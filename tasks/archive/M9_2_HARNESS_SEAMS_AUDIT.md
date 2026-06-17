# M9.2 — Harness-Enablement Seams (F2 + F3 + F4) — Phase-0 Audit

> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose

The D-071 Phase-0 audit for **M9.2**, the second sub-branch of M9 (✅ CLOSED J-307). M9.2 adds the
three **harness-enablement seams** the M9 close routed (`tasks/M9_findings.md` F2/F3/F4) so the
unnumbered Multiparty-tests milestone can run the real cross-node batteries. Unlike M9.1 (a
protocol fix), M9.2 adds **test-control surfaces** — the central question is **how to fence them**
so they are real software for the harness but never a deployable production attack surface. Doc-only;
no code; no DECISIONS change expected (M9.2-D# arc-local per D-069). Feeds M9.2 design → runbook →
Clair.

---

## 2. Gates (D-071)

- **M9.1 CLOSED (J-311)** — the F1 protocol fix shipped; the table is clear for the seam work.
- Suite **1269/0/8**; build 0; clippy clean (default + `--all-features`).

---

## 3. Grounding (the three findings against live `main`)

### 3.1 F2 — fresh-peer federation-initiate (M9.2-A1)
**M9-finding refinement (D-065):** the M9 finding read "no `--aicontrol` initiate verb (FED_3006)".
Live `main` is more precise: the aicontrol **`federation initiate` / `federation accept`** verbs DO
exist (`xgen-common/src/aicontrol/cmd.rs`; 180 s `TimeoutTier::Federation`). The real gap is
**peer-URL seeding**: `NodeRuntime.peer_urls: HashMap<NodeXgid, String>` (`runtime.rs:242`) maps a
known peer's `node_id` → its `ws[s]://` URL, and `federation initiate` can only target a peer whose
URL is **already in that map**. There is **no external surface** — no config peer-list, no aicontrol
verb — to seed a *fresh* peer's URL into `peer_urls`. So two fresh `xgen-node` binaries that have
never met cannot bootstrap a federation relationship through the external surfaces. (`FED_3006` /
the registration `3006 auth_module_untrusted` are unrelated codes — the M9 finding conflated them.)
This gates the true cross-node cooperative scenarios (MP-C-02 two-node, MP-C-03/04/14).

> **Erratum (J-314 — DESIGN v1.1 §8):** the mechanism implied here — that seeding `peer_urls` lets `federation initiate` target the peer — is **wrong**. `federation_initiate` reads the `FederationRegistry` relationship (FED_3006 if absent), **not** `peer_urls` (confirmed admin_ops.rs:1728). The corrected F2 mechanism (add-peer **upserts a `FederationRelationship`**) is in DESIGN §8. This audit line is preserved as the original Phase-0 record.

### 3.2 F3 — clock-advance across the process boundary (M9.2-A2)
The clock seam is `Arc<dyn Clock>` injected via `NodeRuntime::set_clock` (`runtime.rs:288` doc),
with `xgen_common::clock::MockClock`. The **`mock-clock` cargo feature exists**
(`xgen-common/Cargo.toml:36 mock-clock = []`) but is wired **dev-dependency-only** — xgen-core and
xgen-node enable it under `[dev-dependencies]` for in-process tests; the **release binary does not
carry it**, and even with it there is **no external (aicontrol) surface to advance a running node's
clock** across the process boundary. So M9's Round-0 ran real-clock (the M9-D5 "Clock not promoted"
note). This gates R1 determinism and MP-A-01 (expired-invite replay), which need a controllable
clock.

### 3.3 F4 — raw/malformed-frame injection (M9.2-A3)
**M9-finding refinement (D-065):** the finding read "`send_bytes`/`encode_frame` are private". Live
`main`: `encode_frame` is actually **`pub`** (`xgen-core/src/wire/framing.rs:45`); the binding
constraint is that **`Connection::send_bytes` is private** (`transport/connection.rs:105`), while all
the typed `send_*` methods (`send_event`, `send_transport`, …) are `pub` and always produce
well-formed frames. So the M9 injector can craft forged/duplicate/skew *Events* (well-formed frames)
but cannot push a **truncated/garbage frame** to exercise the frame parser. This gates MP-A-12
(malformed-frame) as a live attack.

### 3.4 The unifying observation (M9.2-A4 — the crux)
All three seams **widen what an external actor can make a node do**: seed an arbitrary peer URL
(F2), move a node's clock (F3), push raw bytes onto the wire (F4). Each is exactly the kind of
surface that, left in a release build, becomes an attack vector (peer-spoofing, clock-tampering,
frame-fuzzing-as-DoS). **So the central M9.2 question is the fence, not the feature** — every seam
must be real for the harness yet provably absent from / inert in a production deploy. This mirrors
the M10 mock-module "real software, never a deployable trust anchor" stance.

---

## 4. Forks for the design phase

| Fork | Question | Lean (to confirm in design) |
|------|----------|------------------------------|
| **M9.2-F-A** | F2 fix shape | A peer-URL seeding surface: an aicontrol `federation add-peer <node_id> <url>` verb (seeds `peer_urls`) and/or a config peer-list read at startup. Lean: aicontrol verb (consistent with the D-066 drive surface), fenced. |
| **M9.2-F-B** | F3 fix shape | An aicontrol `clock advance <duration>` / `clock set <ts>` verb that drives the injected `MockClock` on a running binary, plus shipping the `mock-clock` build for the harness. Fenced. |
| **M9.2-F-C** | F4 fix shape | Either a `pub` raw-send seam on `Connection` (e.g. `pub async fn send_raw(&mut self, bytes: &[u8])`) or a hand-rolled `connect_async` wire-client inside `xgen-mptest`. Lean: weigh a fenced `pub` seam vs keeping the rawness entirely inside the test crate (no production-crate surface at all). |
| **M9.2-F-D** | **The fence (the Joe-lock)** | **(1) compile-time cargo feature** (e.g. `harness-control`, off by default, the seams `#[cfg(feature=…)]`-gated → physically absent from release); **(2) runtime dev-flag** (env/CLI `--unsafe-harness-control` gating the verbs at dispatch); **(3) test-crate-only** (rawness never enters a production crate — only viable for F4). Lean: **compile-time feature** — strongest guarantee (the surface cannot exist in a release binary), and it composes with the existing `mock-clock` feature. The Joe-lock. |
| **M9.2-F-E** | Grouping / sequencing | One milestone, but do F2/F3/F4 ship as one commit or three? Lean: three small commits under one fence (each finding is independent; F4 may need no production-crate change at all). |

---

## 5. Scope boundary

- M9.2 adds **only** the three fenced test-control seams; it does **not** run the batteries (that is
  the Multiparty-tests milestone) and does **not** change protocol behaviour.
- **Honest boundary (D-065):** the fence's strength is the whole point. If F-D lands as a **runtime
  dev-flag** (option 2), the audit must state plainly that a misconfigured production deploy could
  expose peer-seeding / clock-tampering / raw-send — whereas a **compile-time feature** (option 1)
  makes the surface physically un-buildable in release. The fencing choice is a security property,
  not a convenience, and must be named as such.
- F2 unblocks cross-node MP-C-02/03/04/14; F3 unblocks R1 determinism + MP-A-01; F4 unblocks MP-A-12.

---

## 6. Next-active

**M9.2 design phase** — lock M9.2-F-A…F-E (**F-D the fence is the crux**); ground the exact aicontrol
verb surface (`xgen-node/src/aicontrol.rs` + `app.rs`), the `set_clock` seam, and the `Connection`
raw-send shape; author the design → runbook → Clair → close.

**Entry point (Rule 0):** CLAUDE PLAY → JOURNAL J-312 → this audit §3 + §4 →
`tasks/M9_findings.md` (F2/F3/F4).

Per D-065 + D-069 + D-071 + D-074 + D-078.
