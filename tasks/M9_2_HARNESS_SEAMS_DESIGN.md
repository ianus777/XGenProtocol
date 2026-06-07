# M9.2 — Harness-Enablement Seams (F2 + F3 + F4) — Design

> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose

Execute the M9.2 Phase-0 forks (`tasks/M9_2_HARNESS_SEAMS_AUDIT.md`) as locked decisions. M9.2 adds
the three harness-enablement seams (F2 peer-seeding, F3 clock-advance, F4 raw-send) that unblock the
cross-node Multiparty-tests batteries — **fenced so they can never ship in a release binary**.
Doc-only; no code; no DECISIONS change expected (M9.2-D# arc-local per D-069). Feeds runbook → Clair.

**Locks recap:** D1 the fence = compile-time `harness-control` feature (the crux) · D2 F2 = fenced
aicontrol `federation add-peer` · D3 F3 = fenced aicontrol `clock advance/set` + mock-clock-in-binary ·
D4 F4 = test-crate-only raw client (no production surface, no fence) · D5 one design / three commits.

---

## 2. Grounding (carried from audit + the Cargo/clock plumbing confirmed this session)

- **`mock-clock` is an `xgen-common` cargo feature** (`xgen-common/Cargo.toml:36`), pulled by
  xgen-core/xgen-node **only under `[dev-dependencies]`** → `MockClock` is **not** compiled into a
  normal binary build.
- The binary reads time via `runtime.clock().now_utc()` (default `RealClock`) and **never calls
  `set_clock`** — that seam (`NodeRuntime::set_clock`) is exercised only by in-process tests. So
  driving a *running binary's* clock is genuinely new wiring, not a flag flip.
- F2's `peer_urls` map (`runtime.rs:242`) and the existing `federation initiate`/`accept` aicontrol
  verbs are as the audit found; F4's frame parser lives in the normal build (the raw-send target).
- aicontrol verb handlers live in `xgen-node/src/aicontrol.rs` + `app.rs`.

---

## 3. Locked decisions (M9.2-D1 … M9.2-D5)

### M9.2-D1 — The fence: compile-time `harness-control` feature (F-D, the crux)
A new **off-by-default cargo feature `harness-control`** on `xgen-node`. Every F2/F3 control seam is
`#[cfg(feature = "harness-control")]`-gated — verb registration, dispatch arm, and handler. A default
`cargo build` (release or dev) **physically cannot contain** the seams; the harness builds the binary
with `cargo build --features harness-control`.

- Plumbing: `xgen-node [features] harness-control = ["xgen-common/mock-clock"]` (forwards the existing
  feature so `MockClock` compiles into the binary for D3). The normal `xgen-common` dependency carries
  the forward; the `[dev-dependencies]` mock-clock wiring stays for the in-process test suite.
- **Not** a runtime dev-flag: the security property is *un-buildability* in release, not a guard that a
  misconfig could bypass (§5).

### M9.2-D2 — F2 peer-seeding: fenced aicontrol `federation add-peer` (F-A)
A fenced verb `federation add-peer <node_id> <url>` that inserts `(node_id → url)` into
`NodeRuntime.peer_urls`, after which the existing `federation initiate` can target the now-known peer.
No config peer-list (a second mechanism with its own surface). This lets the harness federate two
fresh binaries that have never met.

### M9.2-D3 — F3 clock-advance: fenced aicontrol `clock advance/set` + mock-clock-in-binary (F-B)
A fenced verb pair `clock advance <duration>` / `clock set <rfc3339>` that drives the binary's
injected `MockClock` via `NodeRuntime::set_clock` (installed at startup **only** under
`harness-control`; the default binary keeps `RealClock`). `harness-control` pulling
`xgen-common/mock-clock` (D1) is what makes `MockClock` available in the binary at all.

### M9.2-D4 — F4 raw-send: test-crate-only raw client (F-C) — **no production surface, no fence**
`xgen-mptest` opens its **own** raw `tokio-tungstenite` socket to the node and writes arbitrary
bytes (truncated/garbage frames) to exercise the node's frame parser. **No `Connection::send_raw` is
added to any production crate**, and F4 needs **no `harness-control` feature** — the frame parser is
present in the normal build, and a test-only crate is inherently un-shippable. F4 is therefore
independent of D1 (the fence covers F2/F3 only).

### M9.2-D5 — Grouping/sequencing (F-E)
One design; the runbook sequences **three small commits**: **C1** the `harness-control` feature + F2
`add-peer` verb; **C2** F3 clock verb + the mock-clock-in-binary plumbing + startup `set_clock`;
**C3** F4 raw client in `xgen-mptest`. C3 is orthogonal (no production change).

---

## 4. The fence guarantee (the security property)

With D1, a release/default build has **no** add-peer verb, **no** clock verb, and **no** `MockClock`
linked — the surfaces are compiled out, not merely guarded. This is strictly stronger than a runtime
dev-flag, which would leave a live (if gated) surface in every shipped binary, one misconfiguration
from exposure. **Proof obligation (M9.2-D1):** a test asserting the verbs are **absent** in a default
build (e.g. the default-feature aicontrol verb table does not contain `federation add-peer` /
`clock advance`), paired with the harness driving them **present** under `--features harness-control`.

---

## 5. Honest boundaries (D-065)

1. **F2 `add-peer` is not production peer-discovery.** It seeds a peer URL *for the harness*. How real
   nodes discover/authorize each other in a deployment is a separate, still-open product question,
   **out of M9.2 scope** — the verb must not be read as that feature.
2. **The fence is a security property, named as such.** Compile-time `harness-control` makes
   peer-seeding / clock-tampering un-buildable in release; that is the whole point of M9.2.
3. **F4 models a hostile peer, not a compromised honest binary** (carried from the M9 injector
   boundary) — the raw client tests the parser's rejection, not an insider threat.

---

## 6. Proof plan (M9.2 proves the seams + the fence — NOT the batteries)

The full R1/R2/R3 batteries are the Multiparty-tests milestone; M9.2 proves each seam works and the
fence holds:

- **Fence test** (D1): default build → the add-peer/clock verbs are absent from the aicontrol surface;
  `--features harness-control` → present. (The §4 obligation.)
- **F2 smoke**: two fresh `xgen-node` binaries (harness-control build), `add-peer` each direction →
  `federation initiate` succeeds → a Space replicates across them (the cross-node bootstrap that was
  impossible before).
- **F3 smoke**: `clock advance 2d` on a running node moves its `now_utc()` (observable via an existing
  time-dependent surface, e.g. an invite-expiry boundary), deterministically.
- **F4 smoke**: the `xgen-mptest` raw client sends a truncated frame → the node rejects it at frame
  parse (MP-A-12), connection survives or closes cleanly, no panic.

These run as `#[ignore]`/out-of-band harness smokes (like M9 Round-0); the fast unit suite stays
1269/0/8.

---

## 7. Scope + next-active

**Change surface:** `xgen-node` (Cargo `[features]`; fenced verb handlers in `aicontrol.rs` + `app.rs`
startup `set_clock`) — all `#[cfg(feature="harness-control")]`; `xgen-mptest` (raw client, C3).
**Untouched:** protocol/validation, ordering, the default binary's behaviour, all other crates.

**Next-active: M9.2 runbook** (`tasks/M9_2_HARNESS_SEAMS_IMPL.md`) — C1 feature + add-peer / C2 clock
verb + mock-clock plumbing / C3 raw client + the §6 smokes + the fence test → Clair → close. Then
**Multiparty-tests**.

**Entry point (Rule 0):** CLAUDE PLAY → JOURNAL J-313 → this design §3 + §4 →
`tasks/M9_2_HARNESS_SEAMS_AUDIT.md` §3 + §4.

Per D-065 + D-069 + D-071 + D-074 + D-078.
