# M10.5 — The M10-Routed Carve-Outs (MP-C-16 re-run · MP-F6 fold · MP-C-06 re-home) — Implementation Runbook
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Status & lineage

The M10.5 implementation runbook — executes the Joe-LOCKED design `tasks/M10_5_CARVEOUTS_DESIGN.md`
v1.0 (`d1c32fa`, J-373; decisions M10.5-D1..D8) off the Phase-0 audit `tasks/M10_5_CARVEOUTS_AUDIT.md`
v1.0 (`6947a78`; findings M10.5-A1..A8). The final M10 sub-arc — **all-green, no carve-out → M10.5
closes → M10 closes.** A fix-and-rerun arc (loop-to-green, D-065 / MP-R1-D10).

This runbook confirms the four design-§8 groundings to file:line (the C1a read-surface, the C1b
pre-check insertion point + RejectInfo shape, the C1c emit surface, the D6 harness rails) and lays out
the commit plan. **No DECISIONS change** (M10.5-D# arc-local, D-069). **Commit cadence (D-074):** each
arc is a separate commit; Clair's code precedes the Chat doc-bridge per arc; Joe pushes.

**Entry (Rule 0):** CLAUDE.md PLAY → JOURNAL J-373 → design → audit → this runbook →
`tasks/MP_findings.md` (MP-F13/MP-F6/MP-C-06).

**Baseline:** `cargo test --workspace` **1397/0** (post-M10.4); clippy clean default + all-features.

---

## 1. The locked decisions → commit map

| Decision | Lock | Lands in |
|---|---|---|
| **D1 (C1a)** | MP-C-16 witness enrichment — per-Space `home_node` query on both nodes (assert D3 flip-on-both) | Commit 1 |
| **D2 (C1b)** | dispatch-level `banned` pre-check in `dispatch_event`, returning `Rejected` for a banned `membership.join` | Commit 2 |
| **D3 (C1b)** | reject = PermissionDenied-class (4000-unmapped); **no new wire code, no ch3 edit** | Commit 2 |
| **D4 (C1c)** | thin client `home_changed` emit (single-hop to new home; `new_home_node_id` ← `SessionState.node_id`) | Commit 3b (emit) |
| **D5 (C1c)** | escape (client fan-out) **OUT** — flagged, not built | — (fence) |
| **D6 (C1c)** | harness re-home rails (key continuity + per-phase node retarget) — **rails-first** | Commit 3a (rails) |
| **D7** | witness set (C1a box-gated flip-on-both / C1b dispatch unit RED-on-revert / C1c harness re-home witness) | per arc |
| **D8** | sequence **C1a → C1b → C1c** (cheapest-confirming first), loop-to-green | this plan |

---

## 2. Sequencing & commit plan (M10.5-D8)

**C1a → C1b → C1c**, cheapest-confirming first. Each is a discrete commit; box-gated steps run on a
freed box (loop-on-fault). Within C1c, **rails-first** (3a) then the thin emit (3b) then the witness.

| # | Commit | Crate(s) | Box? | Gate |
|---|---|---|---|---|
| **1** | C1a — MP-C-16 witness enrichment (per-Space home query → assert flip-on-both) | `xgen-mptest` (+ a thin read surface only if needed, §3) | **box-gated RUN** | observe green → flip MP-F13 RESOLVED |
| **2** | C1b — MP-F6 dispatch-level banned pre-check + RED-on-revert unit | `xgen-core` | box-free | `cargo test --workspace` green; unit RED-on-revert |
| **3a** | C1c rails — harness re-home rails (keypair reuse + per-phase node retarget) | `xgen-mptest` (test-crate) | box-free build; box-gated to exercise | rails compile + a smoke drives a re-home |
| **3b** | C1c emit — thin client `home_changed` emit on re-home | `xgen-client` (+ a 1-field source, §5.3) | box-free | emit builds/signs/sends; client units |
| **3c** | C1c witness — the harness re-home witness (rides 3a) | `xgen-mptest` | **box-gated RUN** | re-home green; RED-on-revert |
| **close** | Chat doc-bridge (per-arc + final close) | docs | — | all-green, no carve-out → M10/M10.5 close |

**Loop-to-green (D-065/MP-R1-D10):** a faced fault gets fixed + rerun, not papered over. A box-gated
RUN that surfaces a fault loops back into the same arc's code.

---

## 3. Commit 1 — C1a: MP-C-16 witness enrichment (M10.5-D1)

**The fix is in (audit A1, design §2) — nothing to build on the fix path.** A freshly-created Space's
signed `content["home_node"]` is the source node's pubkey id (M10.4 Shape B), so Site 1
(`admin_ops.rs:2096`, `MIG_6010`) + Site 2 (`exchange.rs:717`, `6009`) pass and `apply_space_migrate`
flips `home_node` to the destination pubkey (`state.rs:1161`). C1a only enriches the witness.

**The witness today.** `mp_r2_fixed::mp_c_16_live_migration_space_rehomes`
(`xgen-mptest/tests/mp_r2_fixed.rs:302`) asserts `require_ok` of `migration initiate` (implicit via the
`[[migration]]` director step) + **Space-present-on-B only** (`!tb.event_ids_for_space(&space).is_empty()`,
:314-318). The doc-comment (:320-322) flags the gap: D3 `home_node`-flip-on-both is unasserted.

**The enrichment (D1):** assert, post-migration, that **both** A's copy and B's copy of the Space report
`home_node` = B's pubkey id.

**Read-surface grounding (confirm-at-pickup CP-1).** The harness drives actors over `AicontrolClient`
and the `ScenarioOutcome` carries per-node `transcripts` (`.events`) but **no resolved per-Space
`home_node`**. Two options, in preference order:
1. **Existing read, no new surface (preferred).** Check whether an existing read exposes a Space's
   resolved `home_node`: `admin_ops::space_list_hosted` → `HostedSpaceSummary` (does it carry
   `home_node`? — grep `xgen-node/src/admin_ops.rs`), or a client `members`/`ai status` projection. If
   one does and is drivable over aicontrol, the witness queries both nodes through it.
2. **Minimal query surface (only if none exists).** Add the smallest per-Space `home_node` read the
   witness needs — **test-crate-side if at all possible** (e.g. read the destination node's on-disk
   Space store / state in the harness); a thin aicontrol/admin read on the node **only if** the node
   must expose it. Keep it minimal (no new operator UX); record the choice in the commit message.

**DoD (Commit 1).** The enriched witness asserts D3 (`require_ok` + `home_node` = B's pubkey on **both**
nodes). Box-gated — runs on a freed box (`cargo build -p xgen-node --features harness-control && cargo
build -p xgen-client && cargo test -p xgen-mptest --test mp_r2_fixed -- --ignored --nocapture`);
**observe green, loop-on-fault.** On observed green, the Chat doc-bridge flips **MP-F13 RESOLVED**
(no unobserved-result claim, J-352) + the matrix MP-C-16 row. Fast suite unaffected (the test stays
`#[ignore]`).

---

## 4. Commit 2 — C1b: MP-F6 dispatch-level banned pre-check (M10.5-D2, M10.5-D3)

**The site (audit A3, design §3, line-drift corrected).** The apply-swallow is `let _ =
state.apply_event(&event, &my_node_id)` at **`runtime.rs:748`** (in `ingest_event` @585; the brief's
`:691` is stale → now `store.append`). `ingest_event` returns `()`, so a banned join is
accepted-but-inert (`is_ok=true`), excluded only at `derive_resolved` via `apply_join`'s `banned`
consult (`state.rs:1003`). The sweep is clean (audit A4): the `:748` silence is load-bearing **backward
for replay only** → fix at the dispatch reply, not the apply core.

**Insertion point (confirmed).** The banned pre-check goes in `dispatch_event`'s **"Step 4 — Semantic
pre-checks (post-validation)"** block (`runtime.rs:1278`-onward), the sibling of the AI-role check
(:1280), the PG-13 tier-gate (:1312), and the invite-ceiling gate (:1342). Place it among these
post-validation semantic gates, **before** the `ingest_event` call. It runs after Step 3 validation
(so the event is well-formed + the Space exists) and after the MP-F3 dedup gate (:1272).

**The check (D2).** For a `membership.join` event, consult the target Space's `banned` set (the same set
`apply_join` reads at `state.rs:1003`) and return `DispatchOutcome::Rejected(...)` if the joiner is
banned. Shape:

```rust
// MP-F6 (M10.5-D2) — dispatch-level banned pre-check. apply_join silently
// drops a banned re-join (let _ = state.apply_event, runtime.rs:748; ingest
// returns ()), so the reply was Ok for an event resolution will drop. Surface
// the reject HERE (the reply), leaving the apply-layer silence — load-bearing
// for replay tolerance (A4) — untouched.
if matches!(event.event_type, EventType::MembershipJoin) {
    if let Some(space) = self.spaces.get(&space_id) {
        if space.banned.contains(<joiner>) {
            return DispatchOutcome::Rejected(<PermissionDenied-class reject>);
        }
    }
}
```

- **Joiner identity (CP-2).** For a self-`membership.join` the joiner = `event.sender`; confirm against
  `apply_join`'s joiner extraction (`state.rs` ~:990-1009) that the `banned` set is keyed by the same
  identity the pre-check tests (use the identical key type — `IdentityXgid`/index — as `state.banned`).
- **The reject (D3) — reuse PermissionDenied-class, no new wire code.** `RejectInfo::from_exchange`
  maps an **unmapped** `ExchangeError` → `generic` (4000) (`runtime.rs:163-167`), and `PermissionDenied`
  is unmapped (the same 4000 MP-C-09's banned-*send* reject lands as). So the faithful sibling-shape is
  `RejectInfo::from_exchange(&ExchangeError::PermissionDenied("membership.join: identity is banned".into()))`
  → `(4000, "generic", "…banned")`. (`RejectInfo::generic("…banned")` is byte-equivalent today; prefer
  `from_exchange(PermissionDenied)` so that if MP-F2-followon later maps PermissionDenied to a precise
  code, the banned-join reject upgrades for free.) **No 3040s code, no ch3 edit** — wire-code precision
  is MP-F2-followon's home (design §3, D3). Confirm `ExchangeError::PermissionDenied`'s payload shape +
  that `to_wire_code()` returns `None` for it at pickup (CP-2).

**Witness (D7, box-free).** A `dispatch_event` unit in `xgen-core` (sibling to the existing
runtime/membership tests): set up a Space with `bob` banned; `dispatch_event` a `bob` `membership.join`
→ assert `DispatchOutcome::Rejected` with the PermissionDenied-class code (4000). **RED-on-revert:**
remove the pre-check → the join returns `Accepted` (`is_ok`) yet `bob` is absent from the resolved
membership (accepted-but-inert — the exact MP-F6 symptom).

**DoD (Commit 2).** Build 0 + clippy clean (default + all-features); `cargo test --workspace` green
(+1 unit); RED-on-revert recorded. No spec/ch3/DECISIONS touch (the fold stays bounded).

---

## 5. Commit 3 — C1c: MP-C-06 re-home (M10.5-D4, M10.5-D6) — rails-first

### 5.1 Commit 3a — the harness re-home rails (D6, the real lift; build rails-first)

**Why first (design §6):** the C1c witness (3c) rides the rails; nothing else can drive a re-home
without them. **Test-crate only** (`xgen-mptest`) — no production touch.

**The two rails (audit A8, design §4 D6):**

1. **Key continuity** — one keypair across two node connections (register on A → re-register on the new
   home). Today each actor is a fresh `--init` client with its own keypair: `run_scenario`
   (`runner.rs:266`) spawns each via `ManagedProcess::init_and_spawn_client(&bins, &label, &node.url,
   spec.ai_mode, worker_threads)` (`runner.rs:342`), one node per actor (`ActorSpec.node: String`,
   `manifest.rs:136`). The rail = a spawn path that **reuses an existing actor's keypair / data-dir**
   for the re-home phase (not `--init` a fresh one) — so the same `identity_id` re-homes. Confirm the
   `ManagedProcess` spawn API for a non-`--init` (existing-keypair) spawn at pickup (CP-3).
2. **Per-phase node retarget** — a re-home actor switches its target node mid-scenario (A→C). The batch
   path has a per-command `--node` injection (`xgen-client/src/app.rs:914`, `run_batch_file`); the
   aicontrol path (which the harness drives, `AicontrolClient` `runner.rs:350`) hardcodes
   `node_override:None` (`aicontrol.rs:360`). The rail = a per-phase / per-actor node-retarget surface
   for the re-home step. Likely cleanest as a **re-home director step** (a new `[[rehome]]` manifest
   table, sibling to `[[migration]]`/`[[clock]]`/chaos) that, after a gate, spawns/connects the
   **same-keypair** client to the **new** node — modelling the S5 "shut A, reconnect to C" exactly.
   Confirm the director-step plumbing + manifest schema at pickup (CP-3).

These are **reusable re-home test infrastructure** (general, not MP-C-06-specific).

**DoD (3a).** Rails compile (`cargo build -p xgen-mptest --all-targets`); a minimal smoke drives a
same-keypair re-home A→C (the witness 3c is the full assertion). Box-free build; box-gated to exercise.

### 5.2 Commit 3b — the thin client emit (D4)

**Where (confirmed).** In the client `register` op (`xgen-client/src/ops.rs:307`), **after** `RegisterOk`
is received (`ops.rs:375`) and the connection's `goodbye` (or before it — confirm), gated on
`args.re_registration` (`ops.rs:361`). The op already has every input bar one:

- `identity_id` + `signing_key` (`ops.rs:322-328`).
- `new_home_node_url` = the resolved new home (`home_node`, `ops.rs:332-335`).
- `new_home_node_id` = `ctx.session.node_id` (the new home's pubkey from the AuthOk echo, captured in
  `ensure_connected` `session.rs:150`; `--node`-honoured). **This is the M10.4-dissolved CP-5 blocker.**
- `old_home_node_id` = the client's stored prior home (the home being left). Per **D4**, carry the
  record's stored value **as-is** (for a legacy URL-homed record this is a URL — cosmetic-for-convergence;
  the applier does not gate on `old_home_node_id`). Source it from the prior `ClientState.home_node`
  (the value before this re-home overwrites it, `ops.rs:395`) — confirm the read order at pickup.
- `update_version` = **the one open source (CP-4, may need a Joe micro-lock).**

**The emit (D4).** Build → sign → send (single-hop, no fan-out):

```rust
use xgen_core::identity::registration::{build_home_changed, sign_home_changed};
// after RegisterOk, iff args.re_registration:
let hc = build_home_changed(&identity_id, &old_home_node_id, &new_home_node_id,
                            &new_home_node_url, update_version);   // registration.rs:391
let signed = sign_home_changed(hc, &signing_key);                  // registration.rs:412
conn.send_identity(&signed).await?;                                // single-hop to the new home
```

The new home node receives it on `Inbound::IdentityReplicate → HomeChanged`
(`app.rs:2543`) → `handle_identity_home_changed_msg` (`app.rs:3162`) → `verify_home_changed` +
`handle_incoming_home_changed` (`replication.rs:165`). **Node-to-node convergence rides existing
replication** (re-registration's `push_identity_to_peers`, `re_home = re_registration && already`
`app.rs:2928`, spawn `app.rs:2952`) — **no broadcast machinery (D5 escape is OUT).**

**CP-4 — the `update_version` source (the one detail the design's "thin emit" did not pin; surface +
recommend, Joe-call if it forces a wire surface, D-078/D-065).** The node bumps `update_version` on
re-home (`app.rs:2930`, `prior + 1`) and propagates the bumped record via `push_identity_to_peers`, but
**does not echo the new version** and `ClientState` does not track it — so the client cannot today fill
a `home_changed` version that beats the replicated one. Options:
- **(i) RegisterOk echoes the new `update_version`** (additive optional field — the exact sibling of
  M10.4's `AuthOk.node_id` echo; thin, not fan-out). The client emits with the echoed value. **Preferred**
  for faithfulness (the emit can land `Ok(true)` re-point).
- **(ii) Client increments a locally-tracked version** (no wire touch) — but `ClientState` does not
  track `update_version` today, so this adds a tracked field + risks divergence from the node's bump.
- **(iii) Witness-only on observability.** The S5 DoD's bar is "`identity.home_changed` in the peer's
  log" (observable) — which holds even if the emit lands `VersionStale` (the dispatch arm logs it either
  way, `app.rs:3203/3216`), since the **replica re-point is carried by `push_identity_to_peers`** (A7).
  Under (iii) the emit is observability-faithful with **zero new surface**.

**Recommendation:** scope C1c's witness to **observability** (iii) for the green (matches the S5 DoD +
keeps the emit truly thin/no-wire-touch), and offer (i) the RegisterOk echo as the small faithful
upgrade if Joe wants `home_changed` to be the authoritative re-point. **Surface this to Joe at the
design-confirm — it is the only C1c detail that can touch a wire surface, and it must not silently
escalate the "thin emit" lock.**

**DoD (3b).** The emit builds/signs/sends on re-home; client units cover the build + the
`re_registration`-gated path; no production behaviour change for non-re-home registrations. Build 0 +
clippy clean.

### 5.3 Commit 3c — the harness re-home witness (D7, box-gated)

**The witness.** Same keypair re-homes A→C (via the 3a rails); a post from C reaches Space S; identity
+ membership continuous; `home_changed` observable in the peer's record/log. **RED-on-revert:** neuter
the emit (3b) and/or the keypair-continuity (3a) → the assertion fails. Box-gated (real
`--features harness-control` nodes + Mock clock) → observed green on a freed box, loop-on-fault.

**DoD (3c).** Re-home witness green on the box; RED-on-revert recorded; the matrix MP-C-06 row flips
🚧 → ✅ (or harness-green-with-boundary if the observability-only scope is the locked bar, mirroring
the MP-C-07 F1B-D4 precedent — record the boundary honestly).

---

## 6. Confirm-at-pickup (D-078 — Clair grounds before each commit)

- **CP-1 (Commit 1):** the per-Space `home_node` read surface — existing read (HostedSpaceSummary /
  members projection) vs a minimal test-crate-side read; prefer no new node surface.
- **CP-2 (Commit 2):** the joiner-identity extraction for `membership.join` (match `apply_join`'s
  key, `state.rs` ~:990-1009) + `ExchangeError::PermissionDenied`'s payload + that `to_wire_code()`
  returns `None` for it (→ 4000-unmapped).
- **CP-3 (Commit 3a):** the `ManagedProcess` existing-keypair (non-`--init`) spawn API + the re-home
  director-step plumbing + the manifest schema for a re-home step.
- **CP-4 (Commit 3b):** the `update_version` source for the emit (§5.2 options i/ii/iii) — **the one
  item that can touch a wire surface; Joe-call if it forces one.** Plus the `old_home_node_id` read
  order (prior `ClientState.home_node`).

None of these reopen a locked decision (D1..D8 hold). CP-4 is the only one with a possible micro-lock.

---

## 7. DoD (per commit) & close

Per commit: build 0-error; clippy `--all-features` clean; `cargo test --workspace` green (box-free
suites); box-gated witnesses run on a freed box with the result recorded (loop-on-fault); RED-on-revert
recorded where the witness is a regression lock. No "commit pushed" line (Joe pushes).

**Close (Chat doc-bridge, after all three arcs green):** flip MP-F13 RESOLVED (on the observed C1a
green) + MP-F6 RESOLVED + MP-C-06 ✅/boundary in `tasks/MP_findings.md` + the matrix; design + audit +
this runbook → COMPLETED; ROADMAP + CLAUDE PLAY + JOURNAL; **M10.5 closes → M10 closes.** Promotion
eval: M10.5-D# arc-local (D-069), re-confirm at close.

---

## 8. Out of scope (fence — design §7)

- The consolidated R1+R2+R3 ledger — rides MP-R3's MP-F14 close, NOT M10.5.
- Layer-2 production identity→home-node **discovery** of a stranger (M10.4-D4 / F1B-D5) — separately
  routed.
- The MP-C-06 escape (client fan-out to non-co-federated space-member nodes) — flagged (D5), not built.
- A precise `join_banned` wire code (D3) — routed to MP-F2-followon.
- Legacy URL-homed Space migration (M10.4-D5) — leave-as-legacy.
- The orchestrated one-shot `recover-identity` command (M8.5-C S5-D4) — deferred UX sugar.

---

## 9. Next-active

**Clair implements Commit 1 (C1a)** — the MP-C-16 witness enrichment (CP-1 first) → box-gated RUN →
observe green → C1b → C1c (rails-first). Chat doc-bridges per arc → close.

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-077 + D-078.
