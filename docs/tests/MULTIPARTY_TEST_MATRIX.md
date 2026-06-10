# Multiparty Test Matrix — Scenario Catalogue & Results
> **Status**: ACTIVE  
> Version: 1.13  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this file is

The single **what-we-test + result** record for the strategic multiparty campaign. Companion
to `tasks/M9_MULTIPARTY_HARNESS_AUDIT.md` + `tasks/M9_MULTIPARTY_HARNESS_DESIGN.md`: the audit/
design define **how** the harness is built; this defines **what** it exercises, in actor-
narrative form, with a result per scenario.

- **Scenarios are authored now** (they shape what the harness must support) and **results fill
  in during the Multiparty-tests runs** (R1 → R2 → R3, audit §6.2).
- Supersedes the DEPRECATED `MULTIPARTY_S1..S5_*.md` spec files as the live scenario source.
- **Living document** — scenarios are added as design + runs surface them.
- **M9 Round-0 status:** the `xgen-mptest` harness is built (M9 C1–C5, closed J-307); MP-C-02 +
  MP-A-05 ran green against the real binaries (single-node). The other 35 stay PENDING for the
  Multiparty-tests milestone. Open findings live in `tasks/M9_findings.md`.

---

## 2. Conventions

**IDs.** `MP-C-##` cooperative / realistic · `MP-A-##` adversarial / break-the-system.

**Each scenario records:** Narrative · Expected · Oracle (M9-D4: `.events` transcripts + `state`
query) · Round · Batch (saved aicontrol file(s)) · Result (PENDING → PASS/FAIL + run ref).

**Result legend.** ✅ PASS · ❌ FAIL → routed finding (`MP_findings.md`) · PENDING (seeded, not yet
run) · 🚧 BLOCKED — no authoring/harness capability; untested, not a defect, not closed.

**aicontrol batch files (saved artifacts, M9-D8).** The harness drives `--aicontrol` (persistent
JSONL): one `Command` envelope per line — `{"cmd": "...", "args": {...}, "id": "..."}`.
Verbs/bindings: `register`→`identity_id` · `create-space`/`create-dm-space`→`space_id` ·
`create-room`→`room_id` · `invite`→`event_id` (requires `role`) · `join`→`space_id` (takes
`space` + optional `room`; **no** `invite_event` — chaining is the node's pending-invite
bootstrap + `prev_events`) · `send`→`event_id` (requires `room`). In one connection the binary's
own `bind` + `$name` chains per-connection. Batches are saved, versioned files under
`docs/tests/multiparty_scenarios/<ID>/`, one `.jsonl` per actor + a `manifest.toml` (actor → node
assignment, batch, `[[exports]]`, `[[waits]]` ordering edges). The harness feeds lines verbatim
(after `{{}}` substitution) — no ad-hoc inline generation.

**Cross-actor values (M9-D8).** Per-connection `bind`/`$` cannot cross actors → cross-actor
values use a `{{key}}` placeholder the orchestrator fills from a prior actor's **exported** reply
field. Data-dependency auto-orders; non-data ordering uses a manifest `[[waits]]` edge.

**Wire-malformation vs logic-attacks.** Forged-signature / malformed-frame / equivocation cannot
be valid envelopes → they run through the **M9-D6 raw-wire injector**, not batch files.
Logic-attacks (expired invite, tier-gate, unauthorized join) **are** batch-expressible: the batch
sends, the Result asserts the expected rejection **code/category** (`Category`: protocol /
lifecycle / argument / connection / timeout / permission).

---

## 3. Cooperative / realistic family (`MP-C-##`)

> **Cross-node prerequisite (M9 finding F2) — RESOLVED (M9.2).** The fresh-peer
> federation surface shipped (M9.2 fenced `federation add-peer` / `initiate`); MP-R1 encodes the
> G-6 bootstrap in `runner::run_scenario` (MP-R1-D1a). True cross-node MP-C-02 / MP-C-03 now run
> A↔B (C4). Remaining cross-node rows (MP-C-04 / MP-C-14, 3-node) stay R2.

### MP-C-01 — multi-client local fan-out (S1)
- **Narrative:** Alice + Carol register on Node A · Alice creates Space S · invites Carol · both post.
- **Expected:** each sees the other's messages; S converges on A.
- **Oracle:** per-client `.events` + `state` compare. **Round:** R1 · **Batch:** `MP-C-01/{alice,carol}.jsonl` + `manifest.toml` · **Result:** ✅ PASS (MP-R1 C5, `mp_r1_c5::mp_c_01_local_fanout_converges`) — single-node local fan-out: alice + carol both members of S on Node A; the two client views agree `{alice:owner, carol:member}`; both posts present in Node A's cooperative event set.

### MP-C-02 — invite & join (S2/INV) [✅ PASS — true cross-node A↔B]
- **Narrative:** Alice creates S + a room · invites Bob (`role:member`) · Bob joins (pending-invite bootstrap, no `invite_event` arg) · both post.
- **Expected:** Bob a member; S converges; both views agree `{alice:owner, bob:member}`. (INV bootstrap, M8.5-B.)
- **Oracle:** membership equal across views; per-Space cooperative `.events` id-set matches (`state.federation_add` excluded, MP-R1-D7).
- **✅ Result (MP-R1 C4, `mp_r1_c4::mp_c_02_invite_join_converges_cross_node`):** PASS — **true cross-node A↔B**: alice@A invites; bob@B joins; membership + cooperative DAG content converge on both nodes (the G-6 bootstrap, MP-R1-D1a). Promoted from the M9 Round-0 single-node smoke (which the promotion retired).
- **Round:** R1 · **Batch:** `MP-C-02/{alice,bob}.jsonl` + `manifest.toml` (§5, committed)

### MP-C-03 — concurrent send under conflict (S2)
- **Narrative:** Alice (A) + Bob (B), members of S, both `send` at one frontier · nodes federate.
- **Expected:** both retained; resolved order byte-identical A+B (M8).
- **Oracle:** cooperative `.events` SET equality (both messages retained on both nodes) + membership converge — the R1 floor. Byte-identical RESOLVED ORDER (M8) is the R2 check. **Round:** R1 → R2 · **Batch:** `MP-C-03/*` · **Result:** ✅ PASS (MP-R1 C4, `mp_r1_c4::mp_c_03_concurrent_send_both_retained`) — true cross-node A↔B; both messages retained + converge.

### MP-C-04 — federation topology, transitive path (S3)
- **Narrative:** 3 Nodes A-B-C · Space on A · members on A,B,C · A posts.
- **Expected:** delivery per the locked F-5/D-089 pairwise model; convergence on all three.
- **Oracle:** `state`+`.events` across A/B/C. **Round:** R2 · **Batch:** `MP-C-04/*` · **Result:** PENDING (cross-node, gated on F2)

### MP-C-05 — sustained n×n chat (S4)
- **Narrative:** N nodes × M clients, sustained interleaved posting for a window.
- **Expected:** loss-free at resolution; all projections converge; no hang.
- **Oracle:** final-state convergence + liveness; capture RSS/thread curves. **Round:** R2 → R3 · **Batch:** generated per ramp · **Result:** PENDING

### MP-C-06 — identity re-home (S5)
- **Narrative:** Bob on B, member of S · re-homes to C, same identity · posts from C.
- **Expected:** identity + membership continuous; post from C reaches S (S5 `re_registration` + `identity.home_changed`, M8.5-C).
- **Oracle:** identity continuity + membership preserved; `.events` shows Bob@C. **Round:** R1 · **Batch:** `MP-C-06/*` · **Result:** 🚧 BLOCKED — harness-capability gap + incomplete production feature. (1) **No key continuity:** each actor is a fresh `--init` client with its own keypair (`runner.rs:214`), no keypair-relocation mechanism → can't place one identity on two nodes. (2) **aicontrol hardcodes `node_override:None`** (`aicontrol.rs:360`), so a client can't retarget B→C. (3) **Production re-home is itself incomplete** — M8.5-C shipped the `re_registration` flag + `identity.home_changed` applier, but the client `home_changed` broadcast was deferred (J-278 CP-5 / J-279, "re-home notify" arc, never built). A genuine re-home (same identity, B→C, post from C) is unauthorable AND not wired end-to-end. Out of MP-R1 scope (design §8); revisit when re-home notify ships + the harness gains keypair-relocation / per-command `--node`. (Capability gap, not a defect — not routed to `MP_findings.md`.)

### MP-C-07 — DM private space across nodes
- **Narrative:** Alice (A) `create-dm-space` with Bob (B) · Bob joins (DM seeds Bob as a pending invite, not a member — `from_dm_space_create`) · both exchange messages.
- **Expected:** single-homed DM space, both parties converge, no third-party visibility.
- **Oracle:** `.events`+`state` both; absence on a non-party node. **Round:** R1 · **Batch:** `MP-C-07/*` · **Result:** ❌ FAIL → routed finding (MP-R1 C4, `mp_r1_c4::mp_c_07_dm_across_nodes_converges`). DM cross-node does **not** converge: (facet-1) Bob's `membership.join` applies on B but never propagates B→A (alice's view stays `{alice:owner}`) — DM-specific (MP-C-02 propagates B→A under the identical federation); (facet-2, open) DM `message.text` events are created (send returns an `event_id`) but absent from both nodes' `.events` — not transcript-observable. Routed per MP-R1-D6 (binary change → out of scope); `MP_findings.md` entry authored at C8. **UPDATE (J-331):** facet-2 RESOLVED (MP-F1a / J-328) + single-node 2-party convergence RESOLVED (MP-F4 / J-331 — `get_dag_tips` true-frontier anchor + A1 keying), both witnessed GREEN at the single-node `MP-C-07-LOCAL` (delivery-only → a3+b4 converge on Node A). The **federated** MP-C-07 (cross-node) stays ❌ KNOWN-FAIL → facet-1 = **MP-F1b** ((iii) membership-driven DM federation; gate-B feasibility proves first). **UPDATE (J-333):** the **federated** MP-C-07 is now **✅ harness-green-with-boundary** — MP-F1b shipped (Design Z, `9b4ab8b`): `federation_nodes` populated from **parties = members ∪ pending invitees** (the counterparty is in the set from create → the bootstrap `membership.join` passes F-3 with **no skip**, F-3 still blocks 3rd parties — hole-closed unit) + a `repopulate_dm_federation_after_identity` hook drains the F-3-held join on a late-arriving record (D-076 by inheritance). Both a3 + b4 converge A↔B, **stable ×3**; RED-on-revert genuine. **No production witness claimed** (F1B-D4) — convergence to a not-yet-discoverable counterparty is deferred behind the routed F1B-D5 identity→home-node discovery arc (ROADMAP near-future). Invariant E amended (members → parties) + promoted → D-091.

### MP-C-08 — multi-room space + per-room overrides (PG-12)
- **Narrative:** Alice creates S + multiple rooms · sets a per-room `Deny` override · members post per room.
- **Expected:** posts honor per-room overrides; each room converges independently; override enforced + converged.
- **Oracle:** per-room `state` + `.events`. **Round:** R1 · **Batch:** `MP-C-08/*` · **Result:** 🚧 BLOCKED — no client authoring verb for `state.room_update` (the per-room `Deny` override, PG-12). The primitive is an xgen-core builder only (`build_room_update_event`); authoring was deferred to the UI pass (Arc D PG-12 close), and adding a verb is out of MP-R1 scope (design §8 / MP-R1-D6). Capability gap, not a defect (the path was never exercised) — not routed to `MP_findings.md`. Kept in the R1 set as unfinished, not closed. **✅ UPDATE (J-338): PASS — room_update verb SHIPPED (70a80a6).** Thin-verb arc 3 (`room_update`, J-338): the client `room_update` verb ships over the existing `build_room_update_event` + `apply_room_update` (wholesale-replace applier — Arc D CP-3) + the `check_permission` per-room override gate (PG-12 HAS teeth — `PermissionDenied` on a `Deny` override at validate, exchange.rs:820–833, unit-proven exchange.rs:2464). Scenario `mp_r1_c5::mp_c_08_*`: alice creates S + room1 (open) + room2 (carries `(Moderator,SendMessages)→Deny`), invites bob as moderator, bob posts both. **Positive / per-room independence:** room1 post accepted + converges; the override present in room2's resolved state. **Enforcement (assert-the-reject, the MP-F5 inheritance):** bob's room2 post → `ops::send` → `apply_single_event_confirm` → `reject_code=4000` (PermissionDenied unmapped, pinned to observed — the MP-A-20 precedent; MP-F2-followon) + `event_id`, post absent everywhere. RED-on-revert genuine (author no overrides → room2 post accepted Ok → RED). Pre-fold gate cleared: `invite --role moderator` seats Moderator (`apply_invite` `Role::from_str` → `apply_join` seats the invited role). Appendix F `room_update` entry carries the wholesale-replace note (RU-D1).

### MP-C-09 — ban → converge → post-rejected
- **Narrative:** Member Bob is banned by an admin · Bob attempts a post after the ban.
- **Expected:** ban converges on every node; Bob's post-ban event rejected/excluded everywhere (M8 ban-vs-join Layer 1).
- **Oracle:** membership + `.events` exclude Bob's late post on all nodes. **Round:** R1 · **Batch:** `MP-C-09/*` · **Result:** 🚧 BLOCKED — no client authoring verb for member-initiated `membership.ban` (admin ban). The primitive is an xgen-core builder only (`build_membership_event`); authoring was deferred to the UI pass, and adding a verb is out of MP-R1 scope (design §8 / MP-R1-D6). The node-admin `space force-eject` (`membership.node_eject`) is **not** a substitute — it is Node-authority on a different protocol path, not member-admin ban, and substituting it would test the wrong thing under MP-C-09's name. Capability gap, not a defect — not routed to `MP_findings.md`. Kept in the R1 set as unfinished, not closed. **✅ UPDATE (J-337): PASS — ban verb SHIPPED (d2a7a80).** Thin-verb arc 2 (`ban`, J-337): the client `ban` verb ships over the existing `build_membership_event(MembershipBan)` + `apply_ban` cascade + the `can_ban` gate (Admin+ only — Moderator excluded — enforced at `check_permission` validate + `apply_ban`; real teeth). Scenario `mp_r1_c5::mp_c_09_*`: alice bans bob (cascade removes him from members + rooms); bob's post-ban `send` is rejected at validate step-11 (non-member) → **assert-the-reject — the MP-F5 inheritance, on the same `apply_single_event_confirm` path: `reject_code=4000` + `event_id` present** (pinned to the observed code), the post absent on every node, bob excluded from membership (authoritative `{alice:owner}` view). RED-on-revert genuine (neuter `ops::ban` target → bob stays a member → post accepted Ok → RED). Sequencing ban after MP-F5 paid off as planned.

### MP-C-10 — leave & rejoin
- **Narrative:** Bob `leave`s S, later rejoins via a fresh invite.
- **Expected:** leave converges; rejoin admitted; membership timeline consistent across nodes.
- **Oracle:** `state` membership history compare. **Round:** R1 · **Batch:** `MP-C-10/{alice,bob}.jsonl` + `manifest.toml` · **Result:** ✅ PASS (MP-R1 C5, `mp_r1_c5::mp_c_10_leave_and_rejoin_converges`) — true cross-node A↔B: bob joins, `leave`s, is re-invited, and rejoins (each act originating on B and propagating B→A); the final resolved membership converges on both nodes to `{alice:owner, bob:member}` and the cooperative event-id set matches. (The flagged finding-candidate did not diverge — the lifecycle round-trip rides the MP-C-02-proven B→A path; stable across two runs.)

### MP-C-11 — membership churn under load
- **Narrative:** Many joins/leaves interleaved with sustained posting over a window.
- **Expected:** convergence holds throughout; no orphaned members; no lost admitted posts.
- **Oracle:** final-state convergence + member-set equality. **Round:** R2 → R3 · **Batch:** generated per ramp · **Result:** PENDING

### MP-C-12 — E2E-encrypted space content-blindness (S6)
- **Narrative:** N-member Space with `e2e_encryption` ON · members exchange encrypted messages.
- **Expected:** zero plaintext in any node-visible surface; KeyPackage consume + replenish; epoch advance on commit (Arc H).
- **Oracle:** node-side `.events`/store carry ciphertext only; client decrypts. **Round:** R2 · **Batch:** `MP-C-12/*` · **Result:** PENDING

### MP-C-13 — thread create / resolve / archive (Arc E)
- **Narrative:** members create a thread, post, resolve, then archive it.
- **Expected:** thread state transitions converge (rides M8 Layer-5c).
- **Oracle:** `ThreadState` projection equal across nodes. **Round:** R1 · **Batch:** `MP-C-13/*` · **Result:** 🚧 BLOCKED — no client authoring verb for `thread.*` (create / resolve / archive). The primitives are xgen-core builders only (`build_thread_create_event` / `build_thread_resolved_event` / `build_thread_archived_event`); authoring was deferred to the UI pass (Arc E PG-08 close), and adding verbs is out of MP-R1 scope (design §8 / MP-R1-D6). Capability gap, not a defect (the path was never exercised) — not routed to `MP_findings.md`. Kept in the R1 set as unfinished, not closed. **✅ UPDATE (J-339): PASS — thread×3 verbs SHIPPED (8ba23d1).** Thin-verb arc 4 (the last; `thread create`/`resolve`/`archive`, subcommand group mirroring `ai`, 4 dispatch arms per D-092) over the existing Arc E builders + `ThreadState` applier + the `thread.status` shared state-key (M8 Layer-5c). Scenario `mp_r1_c5::mp_c_13_*` (single-node, F-TH-3). **Positive (TH-D4):** owner alice `create`→`resolve`→`archive`; all three events converge in the node's cooperative event set, final status deterministically Archived. *Observation boundary (recorded, not papered):* this row's oracle as-written says `ThreadState` projection equal across nodes, but the harness has **no ThreadState projection rail** (projections are membership-only) — so the witness asserts **transcript-convergence** (the 3 thread events present + id-set matched on every node) and leans on the unit-proven Layer-5c winner-selection (state_key.rs:374) for the projection itself. A named harness-observability boundary, sibling to MP-C-07's harness-green-with-boundary / MP-A-01(ii). **Enforcement (added witness, assert-the-reject, MP-F5 inheritance):** member bob's `thread resolve` refused by the ChangeInfo Admin+ gate → `reject_code=4000` (PermissionDenied unmapped, pinned to observed — the MP-A-20/room_update precedent; MP-F2-followon) + `event_id`, the op absent everywhere. This is the **first MP witness of the thread ChangeInfo authority gate** (MP-C-08 hit only the per-room override layer) — recorded as an *adversarial* assertion (sibling to the MP-A-20 privilege-escalation family) riding this cooperative row, not folded into its stated positive convergence oracle. RED-on-revert genuine (neuter `thread_archive` send → archive event absent → positive convergence RED; restored → GREEN). Appendix F gains the three `thread` entries.

### MP-C-14 — 4–5 node star + mesh topology
- **Narrative:** A central node + leaves (star), then add cross-links (mesh) · a Space spanning all.
- **Expected:** delivery + convergence consistent under both topologies (pairwise-trust model).
- **Oracle:** `state`+`.events` across all nodes. **Round:** R2 → R3 · **Batch:** generated per topology · **Result:** PENDING (cross-node, gated on F2)

### MP-C-15 — node restart mid-chat + replay (S4 durability)
- **Narrative:** A node hosting S is killed mid-conversation, restarted (replay-from-disk), catches up.
- **Expected:** replayed `SpaceState` byte-identical; zero orphans; rejoins federation + converges.
- **Oracle:** pre/post-restart `state` equality + cross-node convergence. **Round:** R2 · **Batch:** `MP-C-15/*` + orchestrator kill/restart · **Result:** PENDING

### MP-C-16 — live space migration during chat (Arc F)
- **Narrative:** `migration initiate` moves S's `home_node` A→B while members post.
- **Expected:** `home_node` flips on both nodes; in-flight posts not lost; convergence holds across cutover.
- **Oracle:** `home_node` + `state` equality post-cutover. **Round:** R2 · **Batch:** `MP-C-16/*` + migration verb · **Result:** PENDING

---

## 4. Adversarial / break-the-system family (`MP-A-##`)

Logic-attacks → R1 (cheap, deterministic). Volume-attacks → R2/R3. Wire-malformation → M9-D6
raw injector (not batch-expressible). C4 catalogued all six injector attacks with grounded
rejection points (`tasks/M9_findings.md`); MP-A-05 ran live at Round-0.

### MP-A-01 — expired-invite federation replay (logic) [INV-EXP, J-298]
- **Narrative:** Alice invites Bob (14d TTL) · clock advances past `valid_until` · a peer catches up the aged Space + replays invite + join.
- **Expected:** Bob's membership **preserved** on the catching-up peer; gate does not re-reject on federation replay (admission-only).
- **Oracle:** membership equal across nodes. **Round:** R1 · **Batch:** `MP-A-01/{alice,bob}.jsonl` + `manifest.toml` (`[[clock]]`) · **Result:** **(i) ✅ PASS** (MP-R1 C7, `mp_r1_c7::mp_a_01_local_expired_invite_rejected`) — local expired-invite: alice invites bob (1d TTL); the director `clock set`s node A to 2099 *after* the invite (`[[clock]]` with `after=invite_ready` + `publishes=clock_advanced`, the MP-R1-D3 clock→actor ordering primitive); bob's join (gated on `clock_advanced`) hits the 3044 invite-expiry gate (LocallySubmitted) → join absent + bob not a member. **(J-340) oracle migrated to assert-the-reject** (the MP-F5 C6→C7 straggler; test-only `a9fbd98`): reads `reject_code=3044` (invite_expired) + `event_id` off the error envelope, mirroring the blessed C6 form, then asserts the join absent + bob not a member. The J-321-era ok-reply `event_id` extraction had silently staled when MP-F2/F1a/F5 made a locally-submitted reject return an error envelope (the rerun surfaced it; protocol correct throughout — the panic was at extraction, before the membership assert). No production change; noted against the MP-F5-followon. **(ii) ⏳ PENDING** — the federation-replay-preserved regression (the row narrative: an aged-Space *catch-up* does NOT re-reject the historical invited-join — INV-EXP/J-298, membership preserved) needs a cross-node catch-up where B federates AFTER the clock ages the Space; the fixed G-6 bootstrap establishes federation early, so this timing isn't orchestrable on the current rails (out of MP-R1 scope). The property is already proven in-process at J-298; a real-binary repro needs late-federation/catch-up machinery. Not a defect — a harness-timing gap.

### MP-A-02 — over-ceiling / expired invite at submission (logic) [3044/3045]
- **Expected:** rejected; `category=lifecycle`/`argument`. **Round:** R1 · **Batch:** `MP-A-02/{alice,bob}.jsonl` + `manifest.toml` · **Result:** ✅ PASS (MP-R1 C6, `mp_r1_c6::mp_a_02_over_ceiling_invite_rejected`) — alice invites bob with `valid_for_days=9999` (>> Tier-1 14d ceiling); the Node rejects at invite-ingest (3045) so the invite event is absent from every node's transcript and bob never becomes a member. (Oracle = Option A paired rejection: offending event absent + protected state unchanged. The category-level `3045` "why" is not batch-observable — `invite` is fire-and-forget, no recv; that assertion lives on the C7 wire path.) **⚠ UPDATE (J-335): STALE on HEAD.** This PASS is the J-321 C6 record. MP-F5 (J-335) **re-ran this scenario on current HEAD and it FAILS** ("reply has no `event_id`") — post-MP-F2/MP-F1a the rejected `invite` returns an error envelope, not ok+`event_id`, so the fire-and-forget oracle premise above is falsified. Protocol behaviour is correct (the reject fires); the **oracle assertion** is stale. Re-grounding + rewrite is an MP-F5 close deliverable (`tasks/MP_findings.md`). **✅ UPDATE (J-336): RESOLVED — MP-F5 shipped (bee2ede).** Re-run on HEAD with the assert-the-reject oracle → PASS: empirical `reject_code=3045` (over-ceiling invite) surfaced as a field + `event_id`, offending event absent + protected state unchanged. Re-grounded stale→✅.

### MP-A-03 — tier-gate join refusal (logic) [PG-13]
- **Expected:** join refused; refusal multiparty-visible + converged; `category=permission`. **Round:** R1 · **Batch:** `MP-A-03/*` · **Result:** 🚧 BLOCKED — no client authoring verb to create a Space with `auth_tier ≥ 2`. `create-space` exposes only `--name`; `ops::create_space` passes `auth_tier=1` literally ([ops.rs:357](../../xgen-client/src/ops.rs#L357)) and the PG-13 gate is "a genuine Tier-1 no-op today" (runtime.rs:1155), so the gate can't be triggered. Sibling to the PG-12 authoring deferral (primitive exists, no authoring surface); out of MP-R1 scope (design §8 / MP-R1-D6). Capability gap, not a defect — not routed. Kept in the R1 set as unfinished, not closed. **UPDATE (J-335): verb SHIPPED (bf22aaf) — no longer BLOCKED, but NOT yet green.** The `create-space --auth-tier` verb ships (auth-tier arc 1, J-335) — gate-teeth confirmed (PG-13 HAS teeth, unit-proven `pg13_tier1_join_into_tier2_space_rejected_3030`, runtime.rs:1255–1342, not the cited 1155; creation uncapped → M10 breadcrumb), so a Tier-2 Space is now authorable. The **batch witness is deferred to MP-F5**: building it surfaced that the C6 reject-oracle premise is falsified on HEAD (a rejected op now errors rather than returning ok+`event_id`; see MP-F5 in `tasks/MP_findings.md`). MP-A-03 greens in MP-F5 (sequenced before ban). Node teeth covered meanwhile by the `pg13_*` unit. **✅ UPDATE (J-336): PASS — green (bee2ede).** MP-F5 shipped → the batch witness lands: alice creates a Tier-2 Space, bob (Tier-1) refused at the PG-13 join-gate → assert-the-reject `reject_code=3030` (tier_mismatch) + `event_id`, join absent + bob not a member, converged. RED-on-revert genuine (flip the batch to `auth_tier:1` → bob joins → RED). The auth-tier verb's deferred witness is retired here — no debt onto the R1 rerun.

### MP-A-04 — unauthorized / non-member send (logic)
- **Expected:** rejected; no event admitted to S anywhere. **Round:** R1 · **Batch:** `MP-A-04/{alice,carol}.jsonl` + `manifest.toml` · **Result:** ✅ PASS (MP-R1 C6, `mp_r1_c6::mp_a_04_non_member_send_rejected`) — carol (never a member of S) posts to S's room; the Node rejects (F-4 step-11 sender-membership) so carol's message event is absent from every node's transcript and carol never becomes a member. (Oracle = Option A paired rejection.) **⚠ UPDATE (J-335): STALE on HEAD.** This PASS is the J-321 C6 record; MP-F5 (J-335) re-ran it on HEAD and it **FAILS** ("reply has no `event_id`") — same root cause as MP-A-02 (the reject is now surfaced post-MP-F2/MP-F1a). Behaviour correct, oracle stale; MP-F5 reconciles. **✅ UPDATE (J-336): RESOLVED (bee2ede)** — assert-the-reject PASS; empirical `reject_code=4000` (step-11 non-member; an unmapped variant → MP-F2-followon) + `event_id`, offending absent. Re-grounded stale→✅.

### MP-A-05 — signature / identity forgery (wire) [F-F] [adversarial Round-0 smoke — ✅ PASS]
- **Narrative:** the injector emits an event signed with a key not matching the claimed identity.
- **Expected:** rejected at `validate_event` (F-4 13-step, **step 12** signature check, exchange.rs — *not* `ingest_event`, which is the no-validation direct-insert) on every node; never applied.
- **Oracle:** event absent from all node `.events`.
- **✅ Round-0 result (M9 C5, run `c5_mp_a_05`):** PASS — node returned `Error(4000, "step 12: signature verification failed")`; forged event absent; the legitimate control message applied. Step-12 isolation against Alice's real member-context Space.
- **✅ Result (MP-R1 C7, `mp_r1_c7::mp_a_05_forged_signature_rejected`):** PASS — re-run through `run_scenario` via the new `kind="injector"` dispatch. The injector establishes member context (register + create_space + create_room → owner ⇒ member), forges its own identity (claim the registered member, sign with a fresh attacker key) so steps 9–11 pass and only step 12 fails; node returned `Error(4000, "step 12: signature verification failed")` on the wire (the C7 recv capability); forged event absent from the node transcript.
- **Round:** R1 · **Mechanism:** M9-D6 raw-wire injector (`kind="injector"`, C7)

### MP-A-06 — equivocation / fork attempt (wire) [F-F]
- **Narrative:** a hostile peer presents conflicting events at one frontier to different nodes.
- **Expected:** **not a rejection** — both valid events apply; M8 resolution converges on a single winner; no permanent fork. (Outcome = convergence-on-winner, not absence.) **Round:** R2 · **Mechanism:** injector · **Result:** PENDING

### MP-A-07 — flooding / DoS (volume) [M8.6]
- **Expected:** no hang; local liveness; honest traffic still applies (cap back-pressure, C8). **Round:** R2 → R3 · **Mechanism:** injector high-rate · **Result:** PENDING

### MP-A-08 — partition + reconnect storm (volume) [M8.6]
- **Expected:** convergence after heal; no lost admitted events; no reconnect deadlock. **Round:** R3 · **Mechanism:** orchestrator link control · **Result:** PENDING

### MP-A-09 — duplicate-event_id replay / dedup (wire)
- **Narrative:** the injector re-sends a valid event with the same `event_id`.
- **Expected:** idempotent — DAG dedup (`graph.add_event`, after validation); applied once **and fanned out once**. **Round:** R1 · **Mechanism:** injector (member-context) · **Result:** ✅ PASS (MP-R1 C7 + MP-F3 fix, `mp_r1_c7::mp_a_09_duplicate_fanned_out_exactly_once`) — the same `event_id` re-submitted is applied once (DAG/store/disk dedup, grounded 3 ways) **and fanned out exactly once** (max occurrences in any single transcript == 1). **MP-F3 (RESOLVED J-326):** the former re-fan-out (`dispatch_event` returned `Accepted` for the duplicate → `apply_fanout` re-broadcast → members received 2×) is fixed by a `store.contains(event_id)` dedup gate in `dispatch_event` → `DispatchOutcome::Duplicate` → idempotent `EventAccepted` + `FanoutRequest::none()`. The assertion flipped from the tolerant `n >= 1` (with a measurement-gap note) to the falsifiable exactly-once; the harness measurement-gap is retired and this row is a clean PASS (was PASS-on-property + routed finding).

### MP-A-10 — causal gap / missing-parent (wire)
- **Narrative:** an event arrives whose `prev_events` are absent.
- **Expected:** buffered (HeldPending) then drained on arrival, or dropped — never applied out of causal order. **Round:** R1 · **Mechanism:** injector · **Result:** ✅ PASS (MP-R1 C7, `mp_r1_c7::mp_a_10_missing_parent_held`) — the injector (member context) submits a `message.text` whose `prev_events` reference an absent parent; the Node's Step-9 predecessor check holds it (HeldPending) → the event is absent from the node transcript (never applied out of causal order). (Held defers rather than emitting an `Error` frame, so no wire reply — absence is the rejection signal.)

### MP-A-11 — oversized payload (resource)
- **Expected:** rejected or bounded; node stays live; no OOM. **Round:** R2 · **Mechanism:** injector · **Result:** PENDING

### MP-A-12 — malformed / truncated frame (wire)
- **Expected:** rejected at transport frame-parse (never reaches `validate_event`); node stays live. **Round:** R1 · **Mechanism:** injector (F4 raw-send seam) · **Result:** ✅ PASS (MP-R1 C7, `mp_r1_c7::mp_a_12_malformed_frame_rejected_node_alive`) — the injector writes a raw truncated frame (header declares 64 KiB, zero payload bytes); the Node rejects it at parse and closes the connection cleanly (`frame_closed=true`), then a post-attack legitimate `create_space` lands in the transcript — the node stayed live and serving.

### MP-A-13 — anti-transitivity probe (federation) [F-5/D-089]
- **Narrative:** A→B delivers an event; assert B does **not** re-forward it to C (pairwise, not transitive relay).
- **Expected:** C does not receive via B; the locked full-mesh/pairwise model holds. **Round:** R2 · **Mechanism:** observe `.events` on C · **Result:** PENDING

### MP-A-14 — ban-evasion via new identity (logic)
- **Narrative:** a banned user registers a fresh identity and attempts to rejoin.
- **Expected:** treated as a new identity subject to the same gates (no automatic re-entry); recorded behaviour. **Round:** R1 · **Batch:** `MP-A-14/*` · **Result:** 🚧 BLOCKED — no client authoring verb for member-initiated `membership.ban` (the same gap as MP-C-09), so the "banned user" precondition can't be established. Without the ban the scenario collapses to trivial cases (an uninvited fresh identity simply does an open-join / a non-invited join — no ban-evasion content). The primitive is an xgen-core builder only (`build_membership_event`); authoring deferred to the UI pass; out of MP-R1 scope (design §8 / MP-R1-D6). Capability gap, not a defect — not routed. Kept in the R1 set as unfinished, not closed. **✅ UPDATE (J-337): PASS — green-half + M10 breadcrumb (NOT a clean evasion-blocked pass).** With the `ban` verb shipped (J-337) the precondition is establishable. Scenario `mp_r1_c6::mp_a_14_*`: alice bans bob, then **(a)** banned bob's re-join is refused at resolution — `apply_join` consults `banned` (state.rs:1003), ban dominates, bob is **not** re-admitted (resolved members exclude him); the enforceable green, RED-on-revert genuine (neuter the ban → bob re-admitted → RED). **Mechanism correction (D-065, grounded mid-arc):** the re-join is **accepted-but-inert, NOT assert-the-reject** — `dispatch_event` has no `banned` pre-check and the apply-error is swallowed (`let _ =`, runtime.rs:691), so the re-join reply is `is_ok=true` while resolution silently drops it. The audit's pivot-(c) assert-the-reject framing is **superseded**; A-14's green is **membership-effect-absence**, not a reject (the send path, MP-C-09, is the genuine assert-the-reject). **(b)** a fresh identity (bob2) open-joins as a new principal — ban is per-`IdentityXgid`, cross-identity linkage is out of protocol scope (pseudonymity by design) → recorded as **behaviour + M10 breadcrumb** (cross-identity linkage = auth-module/reputation, not a protocol gate). So A-14 is honest **green-half** (same-identity refusal) **+ breadcrumb** (fresh-identity), not full evasion-blocked coverage.

### MP-A-15 — clock-skew timestamp (wire)
- **Narrative:** the injector sends an event with a far-future / far-past timestamp.
- **Expected:** a future-skewed event is rejected by the M9.1 **Step-8.5 future-skew bound** (`ts > now + MAX_FUTURE_SKEW_SECS`[300] → `TimestampOutOfBounds`, wire **3046**); resolution unaffected (wire-order determinism, D-076); no state corruption. **Round:** R1 · **Mechanism:** injector · **Result:** ✅ PASS (MP-R1 C7, `mp_r1_c7::mp_a_15_clock_skew_rejected`) — the injector submits a 2099-dated event in a member context; the Node rejects it via the M9.1 Step-8.5 future-skew bound (`"... exceeds now + 300s skew ceiling"`) → event absent. **The wire `Error` frame carries `error_code == 3046`** (the smoke asserts it) — closed by **MP-F2 (RESOLVED J-324):** the reject path was widened (`DispatchOutcome::Rejected(RejectInfo)`) so each gate's already-computed `to_wire_code` reaches the wire, replacing the former generic-4000. (History: M9 finding F1 — "no timestamp bound" — was closed by M9.1/J-311; the former "3046 NOT on the wire" reject-path gap was MP-F2, closed J-324. Residual: the 7 unmapped variants stay 4000 → MP-F2-followon, which is why MP-A-05 below still reads 4000.)

### MP-A-16 — forged invite ("never issued") (logic/wire)
- **Narrative:** a join references an invite event that was never issued.
- **Expected:** rejected (missing-predecessor / membership) or HeldPending→timeout; no membership granted. **Round:** R1 · **Mechanism:** injector · **Result:** ✅ PASS (MP-R1 C7, `mp_r1_c7::mp_a_16_fabricated_invite_grants_nothing`) — **reclassified to C7 at C6** (the batch form is mis-premised: XGen Spaces are open-join by default — [runtime.rs:1244](../../xgen-core/src/node/runtime.rs#L1244) / J-275 — so an uninvited *batch* join legitimately succeeds + grants membership, correct behaviour, not a defect; the batch `join` verb takes no invite-reference arg). The genuine attack is injector-only: alice (batch) creates Space S; the injector (NON-member) submits a `membership.join` whose `prev_events` reference a **never-issued invite** → the Node's Step-9 predecessor check holds it (HeldPending) → the join is absent from the transcript AND the injector never becomes a member (S stays `{alice:owner}`). Distinct from an open-join (no fabricated predecessor referenced). Finding-candidate resolved GREEN — the node correctly refuses the fabricated predecessor.

### MP-A-17 — wrong-space_id confusion (logic)
- **Narrative:** an event references a space the actor is not in / does not exist.
- **Expected:** rejected (`Error 4000` space-not-found observed live at C4); no cross-space leakage. **Round:** R1 · **Batch:** `MP-A-17/{alice,carol}.jsonl` + `manifest.toml` · **Result:** ✅ PASS (MP-R1 C6, `mp_r1_c6::mp_a_17_wrong_space_id_no_leak`) — carol sends to a non-existent space; the Node rejects (4000) so the event is absent from every node's transcript and does not leak into the real Space S (S stays exactly `{alice:owner}`). (Oracle = Option A paired rejection: offending absent + S membership unchanged.) **⚠ UPDATE (J-335): likely STALE on HEAD (inferred).** Shares the C6 reject-oracle premise MP-F5 (J-335) found falsified post-MP-F2/MP-F1a (the rejected op now errors rather than returning ok+`event_id`). Not yet re-run; confirming + reconciling is an MP-F5 close deliverable. **✅ UPDATE (J-336): RESOLVED (bee2ede)** — **empirically confirmed** (re-grounding discipline, not assumed): assert-the-reject PASS; `reject_code=4000` (space-not-found; unmapped → MP-F2-followon) + `event_id`, no cross-space leak. Inferred-stale→✅.

### MP-A-18 — connect / disconnect storm (volume) [C4 leak gauge]
- **Expected:** no task/handle leak; node stays live (the M8.6 C4 attempt-gauge property at the binary). **Round:** R2 → R3 · **Mechanism:** orchestrator churn · **Result:** PENDING

### MP-A-19 — slow-loris / held connections (resource)
- **Expected:** held/partial connections do not exhaust the node; honest traffic unaffected. **Round:** R2 · **Mechanism:** injector partial-write · **Result:** PENDING

### MP-A-20 — privilege escalation (logic)
- **Narrative:** a non-admin actor attempts an admin verb (`space set-node-policy`, ban).
- **Expected:** refused; `category=permission`; no state change. **Round:** R1 · **Batch:** `MP-A-20/{alice,bob,carol}.jsonl` + `manifest.toml` · **Result:** ✅ PASS (MP-R1 C6, `mp_r1_c6::mp_a_20_member_invite_refused`) — **as-authored note:** exercised via the role-gate path (a non-privileged **member** bob attempts the owner/admin-gated `invite`, the real `can_invite` server gate), **not** the originally-named node-admin verbs (`set-node-policy`/`ban` are not client-issuable — clap would return `UNKNOWN_COMMAND`, a control-parse error, wrong category, not the property). Same escalation property. bob's escalation-invite of carol is denied (`can_invite`) so it is absent from every node's transcript and carol never becomes a member. (Oracle = Option A effect-absence; `category=permission` is not batch-observable — `invite` is fire-and-forget — and lives on the C7 wire path.) **⚠ UPDATE (J-335): likely STALE on HEAD (inferred).** Same C6 reject-oracle premise MP-F5 (J-335) found falsified post-MP-F2/MP-F1a; not yet re-run, reconciled in MP-F5. **✅ UPDATE (J-336): RESOLVED (bee2ede)** — empirically confirmed: assert-the-reject PASS; `reject_code=4000` (`can_invite` permission denial; unmapped → MP-F2-followon) + `event_id`, carol absent. Inferred-stale→✅.

### MP-A-21 — stale / rollback MLS commit (wire) [M8.7]
- **Narrative:** the injector replays a stale `mls.commit` against an advanced epoch.
- **Expected:** no epoch regression; concurrent-commit resolution holds (`mls_commit_tip`, M8.7). **Round:** R2 · **Mechanism:** injector · **Result:** PENDING

---

## 5. Runnable batches (authoritative on disk)

The runnable MP-C-02 batches are committed at `docs/tests/multiparty_scenarios/MP-C-02/` —
`alice.jsonl` + `bob.jsonl` + `manifest.toml` (M9 C5, commit `6d08859`) — and are the
authoritative shapes. They match the **real** client arg surface, which differs from the early
illustrative seed (corrected here so the catalogue does not teach a false mechanism):

- `invite` requires a `role` arg (e.g. `member`) — no default.
- `join` takes **no** `invite_event` arg — invite-chaining is the node's pending-invite bootstrap
  + `prev_events`, not a join argument (`JoinArgs = {space, room?}`).
- `send` requires a `room` arg — the room id comes from `create-room` (exported as `room_id`),
  not from the `create-space` reply (`{space_id, event_id}` only).
- Cross-actor ordering that data-dependency cannot express (Bob's `join` must follow Alice's
  `invite`) uses a manifest `[[waits]]` edge (`bob.b2` waits for the exported `invite_ready`).

---

## 6. Status roll-up

| Family | Seeded | PASS | FAIL | BLOCKED | PENDING |
|--------|-------:|-----:|-----:|--------:|--------:|
| Cooperative (MP-C) | 16 | 8 | 0 | 1 | 7 |
| Adversarial (MP-A) | 21 | 13 | 0 | 0 | 8 |

**Roll-up recount as of J-339 (loop-to-green verb work complete — all four thin verbs shipped).** The MP-R1 C4–C7 runs + the loop-to-green fix/verb arcs (MP-F1a–F5 + the auth-tier/ban/room_update/thread×3 verbs) flipped the rows below; **all five formerly-BLOCKED verb-gap scenarios are now witnessed** (MP-A-03 / MP-C-09 / MP-A-14 / MP-C-08 / MP-C-13), each with a genuine RED-on-revert inheriting the MP-F5 assert-the-reject oracle. **Cooperative PASS (8):** MP-C-01/02/03/07/08/09/10/13 — of which **MP-C-07** is *harness-green-with-boundary* (no production witness; F1B-D4) and **MP-C-13**'s positive half is transcript-asserted (no ThreadState projection rail; Layer-5c unit-proven) with an added ChangeInfo-teeth enforcement witness (sibling to the MP-A-20 family). **Cooperative BLOCKED (1):** MP-C-06 (re-home) — the **sole remaining BLOCKED**, deferred to M10 (D10). **Adversarial PASS (13):** MP-A-01/02/03/04/05/09/10/12/14/15/16/17/20 — **MP-A-01** is part (i) only (A-01(ii) PENDING harness machinery), **MP-A-14** is green-half + M10 breadcrumb. Remaining PENDING = the R2/R3 volume/scale/topology rows + A-01(ii). **MP-R1 ✅ CLOSED (J-340).** The R1 rerun on final HEAD `a9fbd98` came back all-green-to-criterion: C4 3/3 · C5 6/6 (+MP-C-06 BLOCKED) · C6 6/6 · C7 7/7 · regression 0-failed across all crates. Close criterion certified (MP-R1-D10 + F1B-D6): **all-green-except-MP-C-06, MP-C-07 harness-green-with-boundary.** The one finding the rerun surfaced — **MP-A-01(i)** — was a test-side stale oracle (the MP-F5 C6→C7 straggler), resolved test-only (`a9fbd98`, assert-the-reject migration → wire 3044); protocol correct throughout, no production change. By-design non-greens (expected, not failures): **MP-C-06** (re-home → M10; sole surviving test-debt item) + **MP-A-01(ii)** (late-federation/catch-up harness machinery, J-298-proven in-process). **Operational notes for MP-R2:** (1) **MP-C-10** failed once under peak C5 tranche-parallelism (aicontrol pipe-connect spawn timeout) and PASSED on isolated re-run — a harness process-spawn flake, not a regression. (2) **Binary-clobber hazard:** `cargo test --workspace` rebuilds `xgen-node` default-features over the `harness-control` binary at the pinned target dir → the heavy tranches then fail all-`UNKNOWN_COMMAND` (the J-315 fence-holds signal); run the workspace regression-check BEFORE the harness-control build, or rebuild harness-control after any workspace build, before the heavy tranches. **Next-active = MP-R2** (scale + real-clock; its own D-071 Phase-0). Horizon: MP-R2 → MP-R3 → Round-2 audit (UI gate) → UI → M10 → M11.

**MP-R2 Phase-0 + design Joe-LOCKED (J-341).** The scale + real-clock round opened + cleared its D-071 Phase-0 (`tasks/MP_R2_SCALE_AUDIT.md`) + design (`tasks/MP_R2_SCALE_DESIGN.md`, MP-R2-D1..D6); 6 forks locked by-recomms. **Scenario set: 14 R2 rows (7 cooperative + 7 adversarial)** + MP-A-01(ii) infra-borne; **MP-A-08 confirmed R3** (matrix-authoritative). **The §2 design falsification (D-065):** only `nodes`/`clients` are dial-spawn axes — **rate is net-new pacing** (a per-line `after_ms`; `run_actor` fires back-to-back today) and **connection-churn is net-new orchestrator infra**; **residents multiplexing → R3** (R2 is test-crate-only, one process per logical participant). R2 climbs three mechanisms: spawn-scale / paced-intensity / connection-churn. **Tranches (F-6):** (a) scale/intensity sweep MP-C-05/11 + MP-A-07 · (b) new-capability fixed-N MP-C-04/12/14/15/16 + MP-A-06/11/13/21 · (c) infra: late-federation/catch-up (+ MP-A-01(ii)/MP-C-15/16) + connection-churn (MP-A-18/19). **MP-A-01(ii)** is now a runnable R2 row riding the D5 catch-up machinery (no longer standalone PENDING). **CEILING (D4):** bench-calibrated floors + failed-rung-no-sample = Ceiling-suspect (reverses R1's conservative default). **RUN gate held:** first RUN step = the `bench.rs` box-ceiling benchmark (`XGEN_MPTEST_BENCH_TIERS=10,50,100`), before any sweep. Next-active = Clair (runbook `tasks/MP_R2_SCALE_IMPL.md`).

**Round-0 (M9) complete (J-307):** MP-C-02 (cooperative) + MP-A-05 (adversarial) ✅ PASS against
the real binaries via the `xgen-mptest` harness (single-node — the harness is the machinery, the
proof). The remaining 35 scenarios stay PENDING for the **Multiparty-tests** milestone
(R1 → R2 → R3) on a finalized binary, gated on the open findings in `tasks/M9_findings.md`
(notably the F2 fresh-peer federation-initiate surface for the cross-node cooperative set and the
F3 clock-advance surface for the deterministic round).

Per D-065 + D-069 + D-074.
