# M10.5 — The M10-Routed Carve-Outs (MP-C-16 re-run · MP-F6 fold · MP-C-06 re-home) — D-071 Phase-0 Audit
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

The M10.5 D-071 Phase-0 audit — the final M10 sub-arc. Grounds the three M10-routed carve-outs
against live `main` (D-078; grep + file:line), surfacing gaps where grounding contradicts the brief
(D-065). Entry per Rule 0: CLAUDE.md PLAY → JOURNAL J-372 → `tasks/M10_5_CARVEOUTS_PHASE0_BRIEF.md`
→ `tasks/MP_findings.md` (MP-F13/MP-F6/MP-C-06) → `tasks/M10_4_HOME_NODE_DISCOVERY_DESIGN.md`
(shipped Layer-1 fix) → `tasks/M8_5_C_S5_REBIND_DESIGN.md` (the `home_changed` receive side).

Three calls already Joe-LOCKED by-recomms (J-372): **C1a** MP-C-16 = verification re-run,
loop-on-fault; **C1b** MP-F6 = bounded fold; **C1c** MP-C-06 = narrow-first-with-escape. This audit
grounds the rails for all three and delivers a recommendation on the load-bearing call — the MP-C-06
**build-vs-escape**.

**Doc-only; no code; no DECISIONS change.** Next: design → Joe-lock → runbook → impl → close.

---

## 1. The three axes (character differs sharply — the whole framing)

| Axis | Character | Production work | Test/harness work |
|---|---|---|---|
| §3.1 MP-C-16 / MP-F13 | **verification re-run** (fix shipped M10.4) | none (fix is in) | a box-gated re-run **+ a witness enrichment** |
| §3.2 MP-F6 | **bounded node-side fold** | a small dispatch-level pre-check | a RED-on-revert unit |
| §3.3 MP-C-06 | **a re-home witness** (the iteration risk) | a **thin** client emit | a genuine harness lift (the real cost) |

---

## 2. Grounding method

Read against live `main` (post-M10.4, `cargo test --workspace` 1397/0 baseline). Every anchor below
is a confirmed file:line on HEAD; where the brief/finding cited a stale line, the drift is recorded as
a finding (D-065/D-078). No box this session → the box-gated re-runs (MP-C-16, the MP-C-06 witness) are
grounded by **code-path reading**, not observed green; the observed-green step is the M10.5 impl/RUN.

---

## 3. Per-axis grounding

### 3.1 — MP-C-16 / MP-F13: the verification re-run (C1a)

**The shipped Layer-1 fix (M10.4, J-371) — confirmed in live code.** The client now learns the
connected Node's pubkey `node_id` from the `AuthOk` echo and stashes it:

- `SessionState.node_id: Option<String>` (`xgen-client/src/session.rs:80`), captured from the AuthOk
  echo in `ensure_connected` (`session.rs:150`: `self.node_id = auth.node_id;`) — **tied to the
  connection actually used**, so a `--node` override is honoured automatically (the M10.4-D5 §5.iii
  interaction, confirmed).
- `create_space` / `create_dm_space` write the pubkey node_id (not the WS URL) into the signed
  `content["home_node"]` (M10.4 C2, `xgen-client/src/ops.rs`).

**Both stall sites clear by construction — confirmed.** With a freshly-created Space's `home_node` =
the source node's pubkey id:

- **Site 1 — `migration initiate` homed-here precondition (MIG_6010).** `admin_ops.rs:2096`:
  `Some(st) if st.home_node.as_str() == rt.node_id.as_str() => {}` else `MIG_6010`. Pubkey ==
  `rt.node_id` → passes. (`migration_initiate` fn at `admin_ops.rs:2081`; `MIG_6011` absent-branch at
  :2104.)
- **Site 2 — cutover authority gate (6009).** `exchange.rs:717`:
  `Some(s) if event.sender.as_str() == s.home_node.as_str() => {}` else `SpaceMigrateAuthority`. The
  migrate is signed under the source (old home) pubkey == `home_node` → passes.
- **Applier re-check + flip.** `apply_space_migrate` (`xgen-core/src/space/state.rs`): defensive
  re-check `event.sender != self.home_node` → `PermissionDenied` (`state.rs:1158`), then
  `self.home_node = destination` (`state.rs:1161`) — flips to the destination pubkey.

**M10.4 C3 already witnessed Site 1 + Site 2 (at the applier level) RED-on-revert** (J-371). The
end-to-end box-gated MP-C-16 re-home was the explicit M10.4-D3 deferral to M10.5. So C1a's production
side is a **read-confirmed re-run** — there is nothing to build on the fix path.

**FINDING M10.5-A2 (witness gap — C1a is "re-run + a witness enrichment", not a pure re-run).** The
box-gated test `mp_r2_fixed::mp_c_16_live_migration_space_rehomes` (`xgen-mptest/tests/mp_r2_fixed.rs:302`)
**under-asserts the D3 shape**. It asserts:
- `require_ok` of `migration initiate` (implicit — the director `require_ok`s the `[[migration]]` step;
  test doc-comment :306-310), and
- **Space-present-on-destination-B** only: `!tb.event_ids_for_space(&space).is_empty()` (:314-318).

It does **not** assert the D3 (J-370) `home_node-flip-on-both`. The test's own doc-comment flags this
verbatim (:320-322): *"home_node-flip-on-both is the box-gated RUN enrichment — needs a per-Space home
query."* No per-Space `home_node` read surface is driven by the test today. So C1a requires a **witness
enrichment**: a per-Space home query on both A and B (post-migration: A's copy + B's copy both report
`home_node` = B's pubkey) to assert the full D3 shape. The migration driving mechanism itself is built
(the `[[migration]]` director step — `MigrationPlan`, `runner.rs:210`; the MP-F8 aicontrol arm shipped
J-347).

**Box-gated, not observable this session.** The test spawns 2 real `--features harness-control` nodes +
a client (federated ⇒ Mock clock + harness-control build). It cannot be RUN in a doc-only Phase-0; the
observed green is the M10.5 impl/RUN step. **Loop-on-fault (C1a)** applies once it runs.

**Verdict §3.1:** the fix flows end-to-end by construction; C1a = stand up the box-gated re-run + the
per-Space-home-query witness enrichment (to assert flip-on-both) → observe green → flip **MP-F13
RESOLVED** (on observed green, no unobserved-result claim — J-352). The enrichment is the only build on
this axis.

### 3.2 — MP-F6: the bounded node-side fold (C1b)

**Line-drift correction (D-065/D-078).** The brief/finding cite `runtime.rs:691` for the
`let _ = …apply…` swallow. On live HEAD that line is **`store.append`**; the actual apply-swallow has
drifted to **`runtime.rs:748`** (M10.4 + MP-F11 added lines since J-337/J-338). Corrected anchors below.

**The apply-site sweep (every `let _ =` in `runtime.rs`, the apply core is `ingest_event` @585, reached
via `dispatch_event` @1001):**

| Line | Site | Disposition |
|---|---|---|
| `runtime.rs:245` | `let _ = tier;` | a no-op param binding — not an apply. Out. |
| `runtime.rs:546` | `let _ = graph.add_event(ev, ...)` (in `rehydrate_space_from_store`) | **silent** graph-add on cold-start rebuild from a trusted store. See A4. |
| `runtime.rs:676` | `match graph.add_event(&event, ...) { Err(e) => tracing::error!(...) }` (in `ingest_event`) | **already loud** — logs the error + continues (the Phase-7-B3 federation_add-bootstrap case, documented :684-686). The sibling of :546, made loud. |
| `runtime.rs:693` | `let _ = store.append(event.clone());` | a **documented** ignore-duplicate (Q1 narrow-scope / candidate D-NNN bidirectional-sustainability future-walk, :689-692). Not an apply-error swallow. Out of MP-F6 scope. |
| `runtime.rs:748` | **`let _ = state.apply_event(&event, &my_node_id);`** | **THE MP-F6 site** — the membership-apply swallow. |
| `runtime.rs:2872` | `let _ = node.dispatch_event(...)` | a `#[cfg(test)]` line. Out. |

**The MP-F6 mechanism (confirmed).** `dispatch_event` (`runtime.rs:1001`) returns a `DispatchOutcome`
and already carries the `DispatchOutcome::Rejected(RejectInfo{ code, name, reason })` reply machinery
(MP-F2, e.g. :1028, :1050). After validation it calls `ingest_event`, whose apply at :748 is
`let _ = state.apply_event(...)` — and `ingest_event` returns `()` (`pub fn ingest_event(&mut self,
event: Event)` @585). So a banned identity's re-join is **accepted-but-inert**: the apply error is
discarded at :748, and even if it weren't, `ingest_event`'s `()` return can't propagate it back to
`dispatch_event`'s reply. There is **no dispatch-level `banned` pre-check** (grep `banned` in
`runtime.rs` → only test lines :5499/:5500/:5516). The end-state is correct because `apply_join`
consults `banned` at `state.rs:1003` (`if self.banned.contains(joiner) { return Err(Banned) }`) — the
second gate — so `derive_resolved` excludes the banned joiner. The dishonesty is in the **reply**
(`is_ok=true` for an event resolution will drop).

**FINDING M10.5-A4 (D-077-bidirectional sweep — the swallow is load-bearing BACKWARD, for replay only).**
Per site:
- **:748 (the MP-F6 apply):** *Forward-drift* — a future state event whose apply must reject (a banned
  join is today's case) is silently accepted-but-inert; the reply lies. *Backward-coherence* — the
  **replay** caller (`replay_spaces_from_dir` → `ingest_event`) wants apply to be **tolerant**: a
  replayed event that resolution will drop must not crash replay (the cold-start path rebuilds the
  convergent snapshot via `derive_resolved` over the resolved log regardless). So the silence is
  **load-bearing backward for replay, NOT for live dispatch.** This split is the whole fix shape.
- **:546 (rehydrate graph-add):** benign cold-start — trusted store, second gate (`derive_resolved`
  rebuild). Same class as the loud :676 but **unlogged** — a minor consistency gap, not load-bearing.
- **:693 (store.append duplicate-ignore):** a deliberate documented ignore; not an apply-error swallow.
  Out of MP-F6 scope (it has its own candidate-D-NNN future-walk).

**No swallow is load-bearing ELSEWHERE in the dangerous sense** (an apply site where a dropped error
escapes a second gate). The sweep finds exactly one genuine reply-dishonesty site (:748), and its
silence is needed only by the replay caller.

**FINDING M10.5-A5 (the honest fix shape — recommend (b)).** Two shapes the brief names:
- **(a) surface the apply-error** — make `ingest_event` (or :748) fallible and propagate to
  `dispatch_event`'s reply. **Rejected:** this ripples the error into the **replay** caller, which
  *depends on* tolerant apply (A4) — it would force replay to handle/ignore the same error, re-widening
  the surface, and touches `ingest_event`'s many callers (live + replay + key-package hook).
- **(b) add a dispatch-level `banned` pre-check** — in `dispatch_event` (before the `ingest_event`
  call), for `membership.join` events, consult the target Space's `banned` set and return
  `DispatchOutcome::Rejected(RejectInfo{...})` for a banned joiner. **Recommended.** It makes the
  **reply** honest (a banned re-join gets a real reject, not accepted-but-inert) with **zero replay
  blast-radius** (the apply-layer silence at :748 stays, serving replay tolerance), reuses the existing
  RejectInfo machinery, and is exactly the "second gate moved up to the reply" the finding describes.

**Open design detail (for the design-lock, not blocking):** the reject **code** for a banned join.
RejectInfo wants `(code, name, reason)` (MP-F2). There is a `SpaceError::Banned` at the apply layer but
no wire reject code is plumbed for a dispatch-level banned reject today; the design picks a code
(reuse an existing 30xx/40xx admission code or assign one) — flag for Joe only if it needs a new wire
slot (RC-F-01 discipline: confirm against ch3 §3.11.7 before assigning).

**Verdict §3.2:** a genuinely bounded fold. Fix shape (b) (dispatch-level banned pre-check) + a
RED-on-revert unit (a banned join → `Rejected`, not accepted-but-inert; revert → accepted-but-inert).
**Nothing load-bearing-elsewhere to route** — the sweep is clean.

### 3.3 — MP-C-06: the re-home witness (C1c — the load-bearing call)

**What MP-C-06 is (pinned).** The **S5 identity re-home** scenario (`MULTIPARTY_S5_client_rebind.md`;
matrix MP-C-06 §"identity re-home (S5)"): an orphaned Identity (its home Node gone) re-registers on a
new Node with the **same keypair** (`re_registration:true`), keeping `identity_id` + Space membership;
federated peers must learn the home moved (`identity.home_changed`) so their replica records re-point.
**Distinct from MP-C-16** (operator-driven Space *migration*, both node_ids supplied — no notify).

**The source blocker is dissolved (M10.4) — confirmed in live code.** CP-5 (M8.5-C v1.2) deferred the
client emit for exactly one reason: the client could not source `new_home_node_id` (it held only the WS
URL). M10.4 added `SessionState.node_id` (`session.rs:80/150`) — the connected Node's pubkey id from
the AuthOk echo. On a re-home, `ensure_connected(Some(new_node_url))` (or the configured new home)
stashes the **new** home's pubkey id → `new_home_node_id` is now sourceable. The single CP-5 blocker is
gone.

**The receive side is fully built (M8.5-C C1) — confirmed.**
- Applier `handle_incoming_home_changed` (`xgen-core/src/identity/replication.rs:165`): version-guard
  (`update_version <= stored` → `VersionStale`/3020) → re-point `home_node` + bump → `upsert`;
  no-prior-record → `Ok(false)` no-op.
- Builder/sign/verify all `pub` in `xgen-core`: `build_home_changed` (`registration.rs:391`),
  `sign_home_changed` (:412), `verify_home_changed` (:423) — reachable from `xgen-client` (depends on
  xgen-core).
- Node dispatch arm `handle_identity_home_changed_msg` (`xgen-node/src/app.rs:3162`), dispatched at
  `app.rs:2543` under `Inbound::IdentityReplicate → IdentityReplicateMessage::HomeChanged`: verifies
  sig (`verify_home_changed`) → applier → persist → **no ack**.

**The client emit does NOT exist — confirmed.** `xgen-client/src/ops.rs:358` comment:
*"`home_changed` emit is [deferred]"*; matrix MP-C-06 records it deferred (J-278 CP-5 / J-279, "re-home
notify" arc, never built); the re-registration integration test (`reregistration_integration.rs:18`)
notes the applier is proven in C1 tests but the client emit is not built.

**FINDING M10.5-A7 (the propagation architecture — grounding sharpens the call FAVORABLY).** The
load-bearing brief question: *does the emit ride existing fan-out or need new broadcast machinery?*
Grounded:
- The node does **NOT re-forward** `home_changed` — `handle_identity_home_changed_msg` applies it
  **locally only** (no onward broadcast). So `home_changed` is a single-hop client→node message each
  receiving node applies.
- **BUT re-registration already propagates the re-homed record.** On `re_registration:true`,
  `accept_registration` re-homes the record and the handler `upsert`s it with a bumped
  `update_version` (`app.rs:2930/2934`), then **spawns `push_identity_to_peers`** (`app.rs:2952`) —
  the same path as a fresh registration. `push_identity_to_peers` sends `identity.replicate` (the full
  record, with the new `home_node` + bumped version) to the **new home's federation peers**;
  `handle_incoming_replicate` on each peer upserts it.
- **Consequence for MP-C-06's topology (full mesh A↔B↔C, S5):** the space-member node C is a peer of
  the new home B → C receives the re-homed record via the existing `identity.replicate` push — **the
  replica re-point on peers already rides existing replication, with no `home_changed` emit at all.**
  The `home_changed` emit is the spec-faithful **delta-notification** (§3.13.8 step 5 + the S5-DoD
  "`identity.home_changed` in Node C's log"), not the sole convergence mechanism in this topology.
- **So the production emit is THIN: a single-hop client `build_home_changed` + `sign_home_changed` +
  `send_identity` to the new home.** No new broadcast/fan-out machinery is required on the production
  side for the MP-C-06 topology.

**The escape trigger (D-065), and why it does not fire for MP-C-06.** A client-side **fan-out** (the
client emitting `home_changed` to *each* node holding its replica — the home nodes of its Spaces, from
`ClientState.spaces[].node_endpoint`) is only required if a space-member node is **not** federated with
the new home (so the re-registration `push_identity_to_peers` cannot reach it). That is a net-new
client fan-out loop = the "heavy broadcast arc" the brief warns against. **It does not fire for MP-C-06**
(full-mesh topology — every space-member node is a peer of the new home). Flag it as the named escape:
*if the design wants home_changed to reach non-co-federated space-member nodes, re-lock depth* (a
client fan-out over known-space endpoints). MP-C-06's witness does not need it.

**FINDING M10.5-A8 (the harness re-home rails — the REAL lift, test-crate not production).** The matrix
records two harness gaps; both confirmed:
- **No key continuity.** Each actor is a fresh `--init` client with its own keypair (the harness has no
  keypair-relocation mechanism; runner actor model binds one keypair per actor). A re-home needs the
  **same** keypair used across two node connections (register on A → re-register on C) — net-new
  harness machinery (a keypair-sharing actor model / a re-home actor phase).
- **Per-phase node retarget.** A re-home actor must switch its target node mid-scenario (A→C). The
  `.xgb`/batch path supports a per-command `--node` injection (`xgen-client/src/app.rs:914`,
  `run_batch_file`), but the **aicontrol** path hardcodes `node_override:None` (matrix anchor
  `aicontrol.rs:360`) and the harness scenario format binds each actor to one node (`node = "a"`). The
  harness needs a per-phase / per-actor node-retarget surface for the re-home step.
- These are `xgen-mptest` (test-crate) changes, **net-new and the larger, more uncertain part of
  MP-C-06** — bigger than the thin production emit. They are also reusable (a keypair-relocation +
  node-retarget rail is general re-home test infrastructure).

**Verdict §3.3:** narrow-first **holds** on the production side (thin emit, source resolved, receive
side built, convergence rides existing replication — no broadcast arc). The genuine cost is the
**harness re-home rails** (test-crate). Escape (a production client fan-out) does **not** fire for the
MP-C-06 topology; it is flagged for the design only if a non-co-federated broadcast requirement is in
scope.

---

## 4. Findings register (M10.5-A#)

| # | Axis | Finding | Verdict / route |
|---|---|---|---|
| **A1** | §3.1 MP-C-16 | M10.4 Layer-1 fix flows end-to-end by construction — both stall sites clear with a pubkey home_node (Site 1 admin_ops.rs:2096; Site 2 exchange.rs:717; applier flip state.rs:1161). | Re-run sound on the production side; nothing to build on the fix path. |
| **A2** | §3.1 MP-C-16 | The box-gated test (mp_r2_fixed.rs:302) under-asserts D3 — checks Space-present-on-B, **not** home_node-flip-on-both (flagged :320-322 "needs a per-Space home query", unbuilt). | C1a = re-run **+ a witness enrichment** (per-Space home query on both nodes). Box-gated → observed-green is the M10.5 RUN. |
| **A3** | §3.2 MP-F6 | Line-drift (D-065/D-078): brief/finding cite runtime.rs:691; live apply-swallow is **:748** (`let _ = state.apply_event`); :693 is `store.append`. | Anchors corrected. |
| **A4** | §3.2 MP-F6 | D-077-bidirectional sweep: the :748 silence is load-bearing **backward for replay** (tolerant apply), NOT for live dispatch. :546 benign cold-start (unlogged sibling of the loud :676); :693 documented duplicate-ignore. No load-bearing-elsewhere swallow. | Sweep clean — nothing to route. |
| **A5** | §3.2 MP-F6 | Fix shape: (a) fallible ingest ripples into replay (which depends on tolerance); (b) dispatch-level `banned` pre-check makes the reply honest with zero replay blast-radius. | **Recommend (b).** Reject-code = design detail (RC-F-01: confirm ch3 §3.11.7). |
| **A6** | §3.3 MP-C-06 | Source resolved: `SessionState.node_id` live (M10.4); receive side fully built (applier replication.rs:165 + dispatch app.rs:3162 + build/sign/verify_home_changed pub registration.rs:391/412/423). Emit does not exist (ops.rs:358). | Emit is a thin client call. |
| **A7** | §3.3 MP-C-06 | Propagation: node does NOT re-forward home_changed (applies locally); **re-registration already fires `push_identity_to_peers`** (app.rs:2934/2952) → in the full-mesh MP-C-06 topology replica convergence rides existing replication. Emit is the spec-faithful single-hop delta-notification, **not** new broadcast machinery. | Narrow-first holds; no broadcast arc. Escape (client fan-out over known-space endpoints) fires only for a non-co-federated topology — flag for design, does not fire for MP-C-06. |
| **A8** | §3.3 MP-C-06 | Harness re-home rails (test-crate, the **real lift**): no key continuity (fresh `--init` keypair per actor) + per-phase node retarget (aicontrol node_override:None aicontrol.rs:360; batch per-command --node exists app.rs:914 but the scenario format binds one node/actor). | The load-bearing build effort, reusable re-home infra. Scope at design. |

No finding reopens a locked decision. A3 is a line-drift correction; A1/A2/A6/A7 confirm the brief
favorably; A4 is a clean sweep; A5/A8 feed the design.

---

## 5. Recommendation — the MP-C-06 build-vs-escape call (the load-bearing one)

**BUILD, narrow-first (C1c holds; the escape does not fire for MP-C-06).**

Grounding sharpens the call decisively favorably: the single CP-5 blocker (the `new_home_node_id`
source) is dissolved in live code (A6); the receive side is fully built (A6); and — the favorable
surprise — **the replica re-point on peers already rides the existing re-registration
`push_identity_to_peers`** in the MP-C-06 full-mesh topology (A7), so the `home_changed` client emit is
a **thin single-hop delta-notification** (`build_home_changed` + `sign_home_changed` + `send_identity`
to the new home), **not** a broadcast arc. The escape (a client fan-out to non-co-federated
space-member nodes) is real but **out of MP-C-06's scope** — flag it for the design, do not build it.

**The genuine cost is the harness re-home rails, not the production emit (A8).** Keypair relocation
(one identity, two nodes) + per-phase node retarget are net-new `xgen-mptest` machinery and the larger,
more uncertain piece. M10.5 is therefore **"two re-runs + a fold + a thin emit + harness re-home
rails"** — not "+ a heavy broadcast arc." The brief's hoped-for shape holds.

**Honest residual for the design to weigh:** the emit's `old_home_node_id` is the client's own
`IdentityRecord.home_node` (the home being left). For a legacy URL-homed record (M10.4-D5
leave-as-legacy) that field is a URL, not a pubkey id; for an M10.4-created record it is a pubkey. The
`home_changed` applier keys/re-points by `identity_id` + `new_home_node` and version (it does not gate
on `old_home_node_id`), so this is cosmetic for convergence — but the design should pin which value the
emit carries (and note the legacy case) for spec faithfulness.

---

## 6. Witness set sketch (design refines; Joe locks)

- **C1a (MP-C-16):** box-gated `mp_r2_fixed::mp_c_16_live_migration_space_rehomes` enriched with the
  per-Space home query → assert D3 (`require_ok` + `home_node` = B's pubkey on **both** A and B). Run on
  a freed box; loop-on-fault. Flip MP-F13 RESOLVED on observed green.
- **C1b (MP-F6):** a `dispatch_event` unit — a banned identity's `membership.join` returns
  `DispatchOutcome::Rejected` (honest reply), RED-on-revert (revert the pre-check → accepted-but-inert
  `Accepted` with the joiner absent from the resolved membership). xgen-core, box-free.
- **C1c (MP-C-06):** the harness re-home witness — same keypair re-homes A→C, post from C reaches S,
  identity + membership continuous; `home_changed` observable in the peer's record/log. Needs the A8
  harness rails first. RED-on-revert = neuter the emit / the keypair-continuity.

---

## 7. Scope fence (OUT of M10.5)

- The consolidated R1+R2+R3 ledger — rides MP-R3's MP-F14 close (its own track), NOT M10.5.
- Layer-2 production identity→home-node **discovery** of a stranger (M10.4-D4 / F1B-D5) — separately
  routed; never an MP-C-16/MP-C-06 dependency.
- The MP-C-06 escape (client fan-out to non-co-federated space-member nodes) — flagged, not built
  unless the design re-locks depth (A7).
- Legacy URL-homed Space migration (M10.4-D5) — leave-as-legacy.
- The orchestrated one-shot `recover-identity` command (M8.5-C S5-D4) — deferred UX sugar.

---

## 8. Next-active

**Design** (`tasks/M10_5_CARVEOUTS_DESIGN.md`) — lock: (C1a) the MP-C-16 witness enrichment shape (the
per-Space home query); (C1b) the MP-F6 fix (dispatch-level `banned` pre-check + reject code); (C1c) the
MP-C-06 build (thin emit) + the harness re-home rails (keypair relocation + per-phase node retarget) +
the witness set; the escape disposition (flagged, not built). → Joe-lock → runbook → impl → close
(all-green, no carve-out → M10.5 closes → **M10 closes**).

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-077 + D-078.
