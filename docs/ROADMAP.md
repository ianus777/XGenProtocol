# XGen Protocol — Project Roadmap
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-20 (Phase 7.5 design phase closed 2026-05-19 via J-093 walkthrough — all four framework decisions P7.5-A through P7.5-D `[JOE-LOCK: locked 2026-05-19]` at `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` (now COMPLETED v1.0 after Commit 1 of the implementation runbook ships). Phase 7.5 implementation runbook authored at `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md` (ACTIVE v1.0, five-commit sequence). Present section flipped from "Phase 9 LOCKED, awaiting Clair pickup" to "Phase 7.5 implementation ready for Clair pickup; Phase 9 PAUSED at Commit 3 boundary". XGID concept entry in Near future unchanged — still sequenced AFTER Phase 7.5 implementation closure and BEFORE Phase 9 Commit 3 resumption. Past section gains Phase 7.5 design entry. No code changes; 519 tests unchanged.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this document is

The canonical coarse-grained view of where the XGen Protocol project has been, where it is now, and where it is going. One status per track, one line or one short paragraph per item, written so a reader can answer "where are we" without reading the project's full history.

**This document complements, does not replace.** Detailed progress lives in `JOURNAL.md` (contemporaneous record), settled architectural calls live in `DECISIONS.md` (numbered decisions), session-state operational guidance lives in `CLAUDE.md` (what Claude Code should read on the next session), specifications live in `docs/xgen_ch*.md` and `docs/xgen_appendix_*.md`. ROADMAP.md sits above all of these as the navigation map between them.

**This document mirrors reality, not aspiration.** When a milestone is descoped (M6 multiparty → M9 Multiparty Redesign), it moves rather than disappears. When new work surfaces (Propagation Reliability Audit opened mid-project, Federation Event Propagation milestone added after the audit surfaced a gap), it lands here the moment it's recognised. The roadmap is not a plan-from-the-start that the project is executing; it is a living record of what the project now knows it needs.

**Detail is revealed empirically.** Future tracks are sketched briefly; their detail accumulates as they approach. The roadmap does not pretend to know what M9 will involve at the level of M5's specification when M5 was active. This is honest pacing: the unknown stays unknown until it has to be known.

---

## Status legend

| Symbol | State | Meaning |
|---|---|---|
| 🟢 | PLAY | Active work right now. The project is playing this track. |
| ✅ | DONE | Completed and shipped. Settled history. |
| 🟡 | PENDING | Designed or scoped, ready to start, no blockers. |
| ⏸️ | POSTPONED | Paused with a known resume condition. Will resume when condition is met. |
| ❌ | CANCELLED | Stopped. Will not ship. Different from deprecated — no replacement, just won't happen. |
| ⬛ | DEPRECATED | Superseded by named replacement. Named replacement always cited. |

## Update discipline — mandatory

**This document MUST be updated whenever a milestone or phase reaches a state change** — without exception, without deferral, without "I'll do it next session." State changes that trigger an update include:

- A track moves from 🟡 PENDING to 🟢 PLAY (work starts)
- A track moves from 🟢 PLAY to ✅ DONE (work ships)
- A track moves from 🟢 PLAY to ⏸️ POSTPONED (work pauses with known resume condition)
- A track is added to the roadmap (newly recognised work surfaces)
- A track moves to ⬛ DEPRECATED with a named replacement
- A track moves to ❌ CANCELLED
- A phase within a multi-phase milestone closes (e.g. M6 Phase 0 → M6 Phase 1)
- An audit closes
- A design conversation locks decisions that change downstream scope
- A canonical document is created or supersedes another

The update happens **in the same commit** as the work that produced the state change — not in a follow-on commit, not in a separate housekeeping pass. If a milestone closes and ROADMAP.md still shows it as PLAY, the milestone has not actually closed. This is the same discipline that applies to CLAUDE.md and per-file `Status:` headers: the canonical record must reflect reality at every moment.

**This document and `CLAUDE.md` are updated together.** Two anchor points for the same reality. If one moves, the other moves. The same commit touches both. Drift between them is a discipline failure that must be corrected immediately.

Detail-level varies by state: settled work gets one line referencing JOURNAL/tag; active work gets a paragraph; near future gets a paragraph if soon, one line if not yet; far future stays brief. Detail accumulates as a track approaches and is reduced when the track settles (active paragraph → done one-liner).

---

## Past — settled

The arc the project has already traversed. Detail is intentionally compact; the canonical record of each item lives in the JOURNAL entries and tags cited.

### Phase 1 — minimal working implementation

✅ **Phase 1 complete** (April 2026, J-029, tag `v0.10.3`, 173 tests). Ten protocol layers shipped (crypto → wire format → DAG event store → WebSocket transport → Node identity → federation handshake → identity registration → Space/Room protocol → message exchange → smoke test). 17-step smoke test passes over real TCP against two live Node processes. The CLI surface (init, status, connections, spaces, peers, identity list, whoami) shipped on both binaries.

### Phase 2 — full protocol + crate split

✅ **Phase 2 protocol complete** (May 2026, J-058, 300 tests). Nine additional protocol layers shipped (layers 11–19, covering Auth Modules, advanced federation, state migration wire shapes, Bootstrap discovery, MLS wire shapes, audit-log facility). `smoke-ph2` runs 60/60 against two live Nodes. The xgen-core crate split (D-022, D-044) landed at J-045, establishing the dual-licence boundary (BSL 1.1 thin shells, GPL-2.0-or-later xgen-core library). One transport-layer bug (D-056, recv() routing collision between DAG Events and control messages on shared type-prefix strings) discovered and fixed during the live run.

✅ **Full integration stress test** (J-059, 6/6 PASS, 14.6s, 300 tests). Three-node topology (A:9080, B:9081, C:9082 + Bootstrap), all six scenarios pass. Two bugs found and fixed during live run (stack overflow in large async fn, B↔C federation recv hang). Comm record archived at `docs/tests/stress_complete_events.json`.

### Post-Phase-2 protocol additions

✅ **AI Identity, Pacing, and Temperature** (J-065, 387 tests, D-059/D-060/D-061). AI Identity extension to the registration pipeline (`is_ai` + `ai_capabilities` immutability, three error codes 3040/3041/3042, operator delegation EventTypes). Per-Space pacing rules with PacingManager and four edge-case handling. Temperature property foundation with reserved `meta_atts` keys, visibility filter, mute EventType with `auto_temperature` reason recognition, NoOpTemperaturePlugin reference implementation.

### M-series binary consolidation and refactor

✅ **M1 Binary Consolidation** (J-068–J-073, 391 tests). Four binaries collapsed to two (`xgen-node`, `xgen-client`), Tauri compiled into both per D-062, library-first dispatch per D-063, all 19 fundamental flags wired, Client `--service` headless resident operational.

✅ **M2 Node Pipe Server** (J-074, 391 tests). Node-side pipe surface with six flags real-implemented, four control commands (`--ping`/`--health`/`--stop`/`--reload-config`), read-only batch subset (7 verbs).

✅ **M3 AI Operator Role** (J-075, 411 tests). Per-(AI, Space) operator role distinct from member role, three-step fall-upward resolution algorithm, 3041 `ai_role_violation` enforcement, Client CLI `init --ai`/`ai delegate`/`ai revoke`/`ai status` subcommands, two-Node federation smoke verification.

✅ **M4 AI Client Binary** (J-077, 429 tests). AI Client as `xgen-client --ai-mode --service` resident mode, AiBehavior plugin trait with EchoPlugin reference implementation, AiPacingTracker for drop-on-throttle pacing, `__HEALTH__` extended with AI-mode lines, D-065 captures the recurring "honest behaviour over polite behaviour" principle.

✅ **M5 ops::* refactor** (J-078, 435 tests, 12 atomic commits, 17/17 smoke PASS, D-067). Every user-facing `xgen-client` verb (13 total) routes through single `xgen-client-lib::ops::<verb>` function. Three dispatchers (CLI arm, batch driver, pipe arm) became thin shims. The drift surface that produced F-003/F-004 in J-067 architecturally eliminated.

### Audits

✅ **CLI Flag Precedence Audit** (J-079, 463 tests, 5 atomic commits, D-068). What started as one named bug (`xgen-node --port`) surfaced five distinct violations. Two helpers (`xgen_common::precedence::resolve_setting<T>` and `resolve_log_level`) installed; every log-level resolution in the codebase now routes through one function. Drift surface eliminated architecturally.

✅ **J-080 carry-over pass** (468 tests, 3 atomic commits). Three of four J-079/M4 carry-overs closed (`--quiet` gating, init config schema-validity, short-lived CLI log path). Item 4 (`cmd_create_space` optimistic-ack UX) escalated to M6 design phase after investigation revealed it was not a Client-side UX bug but a missing protocol primitive — surfaced the accept-signal gap that drove M6 Pass 3 framework decisions.

✅ **Propagation Reliability Audit** (J-081, 468 tests, 1 atomic commit, no tests added — pure code-trace audit). Five-stage lifecycle walk; four of five sections surfaced drift between spec and code. Federation propagation (Stage 6) found to be architecturally absent in production. Validation asymmetry in `process_inbound` surfaced as separate correctness concern. `TransportMessage::Error` wire shape lacks `event_id` — refuting a multi-session-old assumption. Canonical document `docs/xgen_propagation_reliability.md` records full findings. The audit found what it was opened to find, and one substantial unexpected thing besides.

### M6 design phase

✅ **M6 Phase 0 — design phase closed 2026-05-18** (468 tests, 2 atomic commits during Phase 0, no code changes). Three-pass design phase: Pass 1 (Client `--batch` audit, `tasks/CLIENT_BATCH_AUDIT_M6.md`), Pass 2 (verb category sketches + Joe-lock proposals, `tasks/NODE_ADMIN_PASS2_PROPOSALS.md`, now DEPRECATED), Pass 3 (twelve framework decisions locked, canonical design doc shipped at `docs/xgen_node_admin_ops_design.md`). Key decisions: accept signal as first-class protocol primitive (`TransportMessage::EventAccepted` with G2 semantic), six Joe-lock items (connection authority, authorisation proof, live-reload bucket, audit shape and storage, failure semantics, verb naming convention), three discussion threads (phase ordering, missing categories, error format). D-070 candidate ("two events of equal importance, opposite direction") drafted in design doc §9, awaiting promotion. Block 4 (verb-by-verb walks) deferred to a future session, can run parallel to other work.

### Federation Event Propagation design phase — Pass 2

✅ **Federation Event Propagation Pass 2 closed 2026-05-18** (468 tests, single-session work, no code changes). Ten framework decisions Joe-locked in same-day conversation: F-1 hybrid push direction (push for steady state, pull for gap recovery) + F-1a tip exchange at handshake + F-1b drop-on-peer-down + F-1c Node-implementation per-peer record. F-2 long-lived continuous session + F-2 lifecycle boundaries + F-2a one WebSocket per pair bidirectional. F-3 event signature + federation relationship verification (two-check ingestion gate). F-4 unified validation core + per-event-type post-validation handlers + F-4a 30s HeldPending uniform + F-4b structural pre-checks before / semantic after. F-5 transitive federation locked-out v1 (v2 evolution path documented). F-6 `sync_complete` folded in (`SyncComplete { since, new_tip }`) + F-6a wire-shape details + F-6b 5s configurable safety-net (not protocol-fixed). F-7 response-size pagination folded in + F-7a 1000 events default (`[sync].batch_size`, not protocol-fixed). F-8 + F-9 documentation corrections deferred to Pass 3 same-commit. F-10 HeldPending extended for unknown-signer Identity case + F-10a same timeout policy as F-4a. Work shipped across `docs/xgen_federation_propagation_design.md` main doc (v0.6) plus three addenda (`_F7_addendum.md`, `_F8_F9_addendum.md`, `_F10_addendum.md`). Addenda exist because full-file rewrite per F-item became disproportionately expensive once the doc grew past ~70KB; Pass 3 consolidates them.

### Federation Event Propagation implementation — Phase 4

✅ **Federation Event Propagation implementation Phase 4 SHIPPED 2026-05-19** (J-085, 491 tests). F-1 federation event push + F-1b drop-on-peer-down + F-5 origin gating per design doc §4 + §4.5 + §8.5 + runbook §3.4 + §3.4.1. The "missing mechanism" verdict from the Propagation Reliability Audit (J-081 §2: Stage 6 federation propagation architecturally absent in production) is closed — federation push now exists as a production mechanism. Three implementation Joe-locks captured ahead of code in the §3.4.1 doc-pass commit: Q1 `EventOrigin` enum as runtime parameter (rejected wrapper struct and `#[serde(skip)]` on the wire-shape Event); Q2 `FederationPeerSenders` shape as `Arc<Mutex<HashMap<peer_node_id, Sender<OutboundMsg>>>>` mirroring `ClientSenders` (rejected co-location with shared_spaces — `SpaceState.federation_nodes` stays single source of truth); Q3 reuse of `process_inbound` with two-comment overload documentation (semantic overload of `identity_id` accepting Identity URI for clients OR Node URI for federation sessions, documented at function definition + federation call site). Plus R12-R15 Clair-latitude items: R12 registry lifecycle (register on handshake-ACTIVE, deregister on session exit), R13 try_send semantics (non-blocking, drop on channel-full per F-1b), R14 drop-on-peer-down log line, R15 origin attach at entry points. Forward-looking note in the EventOrigin doc comment: enum is extensible to future `ReceivedViaAdminInjection` (M6) and `ReceivedViaBackfill` (hypothetical replay tooling) variants. Structural changes: new `EventOrigin` enum in `xgen-core::node::runtime`; new `FederationPeerSenders` type in `xgen-node/src/fanout.rs`; new `apply_federation_push` function in `xgen-node/src/federation_session.rs` (sibling of `apply_fanout`, not a wrapper; F-5 guard at the top); `dispatch_event` + `process_inbound` signatures gain `origin` parameter; Phase 3's R1 plug-in point in `handle_federation_incoming` is now operational (out_tx registered on ACTIVE, outbound arm drains pushed events to the wire, inbound `Inbound::Event(_)` routes through `process_inbound`+`apply_fanout`+`apply_federation_push` with F-5 short-circuit, deregister on loop exit). Tests: 488 → 491 (+3 integration tests in new `federation_push_integration.rs`: `alice_post_propagates_to_bob_via_federation_push` covers the full push path end-to-end; `f5_anti_transitivity_received_via_federation_event_not_pushed` regression-locks the F-5 §8.5 guard via direct call; `f1b_drop_on_peer_down_no_panic` verifies graceful handling when peers are absent from the registry). Phase 5 (F-1c per-peer record + reconnect scheduling — `run_initiating` gains its first production caller in xgen-node/src/) is next-active. Two intermittent flakes disclosed in the JOURNAL per Rule 2: the pre-existing precedence env-var race (from J-079, ~10-20% workspace runs); the `reconnect_with_existing_tip_small_delta_delivered` Phase 3 test flake under increased Phase-4 parallelism (~10% workspace runs, 0% isolated). Both are flagged as known intermittent flakes; the fix path for both is the same shape (`#[serial_test::serial]` or controlled test parallelism).

### Federation Event Propagation implementation — Phase 3

✅ **Federation Event Propagation implementation Phase 3 SHIPPED 2026-05-19** (J-084, 488 tests). F-1a federation handshake reshape to bilateral tip exchange per design doc §4.4 + runbook `tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.3 + §3.3.1. The pre-F-1a flow (handshake → `space.join_request` → `state.federation_add` → history dump → `goodbye`-close) is replaced by F-1a tip exchange: bilateral `tips: BTreeMap<String, String>` on `Hello` + `Capabilities`, both sides stream per-Space delta via new `compute_federation_delta_for_space` helper + `SyncComplete` terminator, session stays open as the persistent F-2 push channel. Eight Joe-locks captured before code started (§3.3 Option 3 wire shape, §3.3.1 Locks 1-7: Option A full migration / a-i symmetry rule for `state.federation_add` / R1 unused outbound mpsc / R2 sibling delta helper / R3 informational `new_tip` semantic / R4 sorted-by-`space_id` ordering / R5 bilateral helper with initiator-side production caller in Phase 5); both lock-sets shipped in a §3.3.1 doc-pass commit ahead of the code commit per Pass-2-style ride-along discipline. New module `xgen-node/src/federation_session.rs` with `stream_federation_delta` orchestrator (sorted iteration, a-i symmetry, whole-batch SyncComplete). `handle_federation_incoming` refactored: drop JoinRequest receipt + drop inline state.federation_add build + drop goodbye + call `stream_federation_delta` + enter F-2 long-lived continuous session loop with R1-locked Phase-4-prep outbound mpsc. Seven `xgen-client` `run_initiating` call sites migrated to Option A full-migration shape (drop paired JoinRequest, add `BTreeMap::new()` tips, change recv-loop terminator from `Goodbye` to `SyncComplete` with `Goodbye`/`Closed` fallback for pre-F-1a peers). Tests: 480 → 488 (+8: 4 wire-layer in `xgen-core/src/federation/handshake.rs` covering tips-round-trip + tampered-tips + pre-F-1a wire-shape back-compat for both Hello and Capabilities; 4 integration in `xgen-node-lib` including new `federation_delta_integration.rs` file with brand-new + reconnect + pre-F-1a compat scenarios, plus `bilateral_tips_propagate_through_handshake` in existing `federation_integration.rs`). Phase 4 (federation event push, F-1 + F-1b + F-5) is next-active — the persistent F-2 session is now operational and Phase 4's push code plugs into the R1-prepared outbound mpsc.

### Federation Event Propagation implementation — Phase 2

✅ **Federation Event Propagation implementation Phase 2 SHIPPED 2026-05-18** (J-083, 480 tests). F-4 `process_inbound` validation pipeline unification per design doc §7. The pre-F-4 three-path asymmetry (audit J-081 §3.2: Path A messages → full 13-step pipeline; Path B `MembershipJoin` → direct ingest with no signature verification; Path C other state events → direct ingest with no signature verification) closes architecturally. New types: `ValidationOutcome` enum (`Validated` / `HeldPending(Vec<String>)` / `Rejected(ExchangeError)`) and `validate_event` function in `xgen-core/src/message/exchange.rs` (non-mutating, uniform structural and crypto checks, sub-check coverage table documented). New orchestrator: `DispatchOutcome` enum + `NodeRuntime::dispatch_event` + `drain_pending_uniform` in `xgen-core/src/node/runtime.rs` (F-4 §7.7 pipeline shape: structural pre-check → federation-relationship placeholder (Phase 7) → validation core → semantic pre-checks (AI role/capability/operator per §7.6) → ingest → drain). `process_inbound`'s three-arm event match collapsed to one `dispatch_event` call + `match outcome` block. HeldPending now applies to all event families with the same 30s timeout (F-4a — `PENDING_TIMEOUT_SECS = 30` in `xgen-core/src/dag/pending.rs` unchanged; pre-F-4 only `accept_message` populated the buffer). `ExchangeError` gains `Clone` (additive). `accept_message` preserved unchanged for smoke.rs test backward-compat — production traffic routes through `dispatch_event`. Tests: 476 → 480 (+4 in `xgen-node-lib::fanout::tests`: `f4_path_a_message_unknown_predecessor_held_pending_then_drains` regression-locks Path A; `f4_path_b_join_unknown_predecessor_held_pending_then_drains` closes audit Scenario-A non-message for Path B; `f4_path_c_state_unknown_predecessor_held_pending_then_drains` closes Scenario-A for Path C; `f4_rejects_bad_signature_on_membership_join` regression-locks the audit's HIGH-severity vulnerability vector). The audit's HIGH-severity precondition for Phase 4 (federation push) is now met. Coordination with M6 (new) Phase 2 preserved — rejection sites consistent across event families; M6 wires the wire-layer signal in its own milestone.

### Federation Event Propagation implementation — Phase 1

✅ **Federation Event Propagation implementation Phase 1 SHIPPED 2026-05-18** (J-082, 476 tests). F-6 (`transport.sync_complete`) + F-7 (response-size pagination) shipped together as one coordinated wire-protocol change per design doc §9 + §10. `TransportMessage::SyncRequest::limit: Option<u32>` and new `TransportMessage::SyncComplete { since, new_tip, continue_from }` variants with `#[serde(skip_serializing_if = "Option::is_none")]` for backward-compat. `collect_sync_history` signature changed to `(runtime, requester_id, since, limit) -> (Vec<Event>, Option<String>)` honouring `limit` and emitting `continue_from`. Four production callers migrated from quiet-time / hardcoded-deadline to SyncComplete-driven pagination loops with F-6b safety-net timeout: `xgen-client/src/batch.rs:get_dag_tips`, `xgen-client/src/ai_service.rs:224`, `xgen-client/src/ops.rs:721` (`history`), `xgen-client/src/ops.rs:939` (`ai_status`). Cross-Space behaviour Clair-locked as whole-batch (rationale at `xgen-node/src/fanout.rs:177-201`). `[sync]` config section on both binaries with `completion_timeout_seconds` (default 5) and `batch_size` (default 1000), `#[serde(default)]` so existing on-disk configs keep parsing. Tests: 468 → 476 (+7 unit in `xgen-node-lib::fanout` covering wire roundtrip and pagination semantics, +1 integration in `xgen-client/tests/sync_safety_net.rs` covering the safety-net error path). Federation Event Propagation implementation track moved from Near future to Present in this same commit.

### Federation Event Propagation design phase — Pass 3

✅ **Federation Event Propagation Pass 3 closed 2026-05-18** (468 tests, single same-day session following Pass 2, no code changes). Design phase closed; canonical document shipped; runbook handed off. Five deliverables in one coordinated commit: (1) canonical design doc consolidated to v1.0 ACTIVE at `docs/xgen_federation_propagation_design.md` — three Pass-2 addenda folded in as §10 (F-7), §11 (F-8), §12 (F-9), §13 (F-10), addendum files deleted, version bumped from v0.6, Status flipped from PENDING to ACTIVE; (2) all `[JOE-LOCK]` markers walked from "confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3" to final form `[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`; (3) F-8 corrections applied to `docs/xgen_ch4_implementation.md` §4.11.3 + §4.12.3 (forward-references to canonical design doc; located by content match against unique phrases rather than the audit's stale line numbers); (4) F-9 correction applied to `docs/xgen_node_admin_ops_design.md` §4.2 (Federation propagation Stage-6 sub-bullet now a forward-reference); (5) implementation runbook created at `tasks/FEDERATION_PROPAGATION_COMPLETION.md` (Status: ACTIVE, v1.0) — nine phases with hard-locked Phase 2 → Phase 4 ordering. Pass 2 task file (`tasks/FEDERATION_PROPAGATION_DESIGN.md`) and Pass 3 task file (`tasks/FEDERATION_PROPAGATION_PASS_3.md`) both flipped to COMPLETED in the same commit. Federation Event Propagation milestone block in CLAUDE.md flipped from 🟡 PENDING to 🟢 ACTIVE; implementation track in this document went 🟡 PENDING in Near future.

### Federation Event Propagation implementation — Phase 9 survey

✅ **Federation Event Propagation Phase 9 survey closed 2026-05-19** (J-090 deliverable shipped; finalised via Joe-lock in J-091, no code changes, 519 tests unchanged). Pre-implementation subsystem audit per `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY.md` v2.0. Canonical deliverable at `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` (Status flipped PENDING → COMPLETED v1.1 at J-091 close with all four §8 locks recorded inline). Survey recommended **12 Phase 9 scenarios** — 6 baseline (push smoke, anti-transitivity, drop-and-recover, validation-asymmetry regression, unknown-signer first-contact, federation-relationship rejection) + 6 compounds (C2 anti-transitivity at queue depth, C3 F-3 during F-1a recovery, C5 validation asymmetry under load, C7 pagination at boundary, C9 F-3 drain-time hazard, C10 identity-replicate hook under contention). Four compounds (C1, C4, C6, C8) deferred to follow-on federation-stress milestone (blocked on clock-injection seam or improbable bug shapes). Five structural gaps surfaced (G1 `xgen-node_state.json::peers` hard-coded to `vec![]`, G2 no stable trace events for F-1/F-3 paths, G3 no fan-out trace event — all three close as Phase 9 precondition per Q2 locked option a; G4 audit log for F-3 reject + G5 drop-peer affordance defer to M6 per Q2 locked option c). Flake-handling locked at Q3 option (c) — fix both pre-existing flakes (precedence env-var race + reconnect_with_existing_tip_small_delta_delivered) as precondition, escalate to underlying-race investigation only if Phase 9 implementation surfaces new flakes. 14-entry failure-mode catalogue (11 HIGH severity; 11 caught by Phase 9 set; M6, M8, M13 flagged for Client-Side Consequences Audit post-milestone). Survey discipline followed D-071: subsystem audit precedes dependent milestone phase, same pattern as J-081 → Federation design phase. Joe-lock + COMPLETED-flip + Phase 9 implementation task file authoring all happen in J-091, in the same commit as this entry.

### Federation Event Propagation implementation — Phase 7.5 design phase

✅ **Federation Event Propagation Phase 7.5 design phase closed 2026-05-19** (J-093 walkthrough, 519 tests unchanged, no code changes). Cold-start bootstrap design phase surfaced during Phase 9 Commit 3 harness setup when Scenario 1 ("two-Node push smoke") walked into failure-mode catalogue M5 ("F-3 brand-new federation bootstrap dead-locked") exactly where the Phase 9 survey predicted. Root cause: Phase 7's F-3 gate rejects `state.space_create` arriving via federation because `SpaceState.federation_nodes` lookup returns None when Space S doesn't exist locally yet; Phase 7's B1 skip rule only covers `state.federation_add`, not Space-create. Phase 9 stood down at Commit 3 boundary; J-092's Commits 1 + 2 (observability preconditions + flake fixes) stay shipped because both are pure infrastructure. Four framework decisions `[JOE-LOCK: locked 2026-05-19]` at `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` (Status: ACTIVE v1.0 — flips to COMPLETED in Commit 1 of the implementation runbook): **P7.5-A** narrow F-3 + F-4-step-1 skip for `state.space_create` and `state.dm_space_create` EventTypes (sibling to Phase 7 B1), with new `SpaceLocalMetadata` sibling structure in `xgen-common` carrying `introducer_node_id: Option<String>` field (name locked through any future XGID-typing pass) populated at federation Space-create ingestion for DoS-surface triage; **P7.5-B** third HeldPending trigger "missing federation relationship for (peer, space)" resolved by idempotent `state.federation_add` arrival hook, new error code `4007 federation_relationship_timeout`, combination semantics with F-4a (predecessor) and F-10 (Identity) via existing struct-variant Option fields, precedence ranking predecessor (4002) > federation-relationship (4007) > Identity (4006) because federation-relationship is the most upstream blocker; **P7.5-C** per-trigger timeout with predecessor + Identity staying at 30s and federation-relationship defaulting to 180s with new `[sync].federation_relationship_timeout_seconds` config field (brings F-10a's v2 evolution path forward to v1); **P7.5-D** new `pending_federation_relationship: usize` counter in `xgen-node_state.json` plus existing `f3_reject` trace event extended with disposition field (`rejected` vs `held_pending`) — trace event NOT renamed, introducer field NOT exposed in state file. The design phase added four substantive walkthrough findings beyond the draft: §5.3 DoS-surface paragraph with `SpaceLocalMetadata` sibling structure; §6.3 timeout precedence swap (4007 outranks 4006); §6.3 idempotent-hook clarification + two-stage cascade case; §7.3 timeout default raised from 120s to 180s. Implementation runbook authored same-day at `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md` (ACTIVE v1.0) with five-commit sequence: doc-pass, F-3 + F-4 step 1 skip + `SpaceLocalMetadata`, HeldPending third trigger + config + error code + observability, NodeRuntime-level integration tests (six scenarios), milestone close. Phase 9 remains PAUSED at Commit 3 boundary pending Phase 7.5 implementation closure + XGID concept work (sequenced between). Discipline note: the D-071 audit-precedes-dependent pattern extends one level finer to "design gaps surface during dependent work and close before the dependent work proceeds" — Phase 7.5 is the formal recognition of that pattern.

### Named decisions promoted to DECISIONS.md

✅ **D-070 promoted 2026-05-18** (468 tests, same-day post-Pass-3, no code changes). "Two events of equal importance, opposite direction" — named protocol principle. The original draft in `docs/xgen_node_admin_ops_design.md` §9 framed the principle as "EventAccepted exists, symmetric to Error." The Propagation Reliability Audit (J-081 §5) found that framing was necessary but not sufficient: `TransportMessage::Error` lacked an `event_id` field at all, meaning even with both Error and a future EventAccepted, the originator couldn't correlate either signal back to a specific event. D-070's DECISIONS.md entry incorporates the corrected post-audit framing: BOTH (1) both directions of outcome exist (acceptance + rejection signals), AND (2) both directions carry the envelope-level correlation identifier (`event_id: Option<String>` on `TransportMessage`). Without (2), (1) is hollow. M6 §9 draft preserved as historical record; DECISIONS.md D-070 is the canonical authoritative form. M6 (new) Phase 2 implements both halves in coordinated work with Federation Event Propagation milestone's F-4 rejection-site changes.

✅ **D-071 promoted 2026-05-18** (468 tests, same-day post-D-070, no code changes). "Subsystem audits precede dependent milestones" — project-management principle. Every future milestone whose correctness depends on a load-bearing subsystem MUST include a subsystem audit as part of its Phase 0 (design phase). Audits produce code-grounded canonical documents and surface gaps that may need to close as preconditions of the milestone rather than as parallel work. The pattern emerged organically during the Propagation Reliability Audit (J-081), where findings consistently exceeded the audit's nominal scope (four HIGH-severity findings across five sections) and the audit became Pass 1 input for two downstream design phases (M6 Phase 0, Federation Event Propagation Phase 0). D-071 pairs with D-069: audit phase (D-071) → design phase (D-069) → implementation phase, each producing a canonical artefact. Sibling to D-065 and D-070 — D-065 and D-070 are protocol-design principles; D-071 is the project-management analogue. The shared theme across all three: don't let assumed-state substitute for verified-state.

### Deprecated tracks

⬛ **M6 (original) Multiparty baseline pass** — descoped 2026-05-17. Original M6 plan (run the full Multiparty suite S1–S5 twice through present `--batch` to fill the "A" baseline column) descoped because the binary state had shifted post-J-079, the metric-protocol applicability needed reconfirmation, and the bigger problem M6 was meant to solve had grown to span both binaries. **Replaced by M9 Multiparty Redesign** at the end of the M-series trunk; the M6 slot is reused for the Node admin write path. Affected task files (`tasks/MULTIPARTY_S1_tauri_rerun.md`, `tasks/MULTIPARTY_S2_to_S5_present_pass.md`) carry the DEPRECATED status with the M9 pointer.

⬛ **`tasks/NODE_ADMIN_PASS2_PROPOSALS.md`** — superseded by `docs/xgen_node_admin_ops_design.md`. The Pass 2 proposals file was the working document for Pass 3's lock-decisions; once the design doc shipped with the framework decisions filled in, the proposals file lost its operational role. Kept in `tasks/` as historical predecessor per D-069's canonical-document rule.

---

## Present — playing now

The track or tracks the project is actively working on right now. Detail-level here is the most granular in the document — what's in flight, what's blocking what, what the next concrete step is.

🟢 **Federation Event Propagation implementation — Phase 7.5 Cold-Start Bootstrap implementation ready for Clair pickup; Phase 9 PAUSED at Commit 3 boundary.** Phases 1-8 closed J-082..J-089 (519 tests at Phase 8 close). Phase 9 LOCKED in J-091 with `tasks/FEDERATION_PROPAGATION_PHASE_9.md` (ACTIVE v1.0); J-092 shipped Commits 1 + 2 (observability preconditions G1+G2+G3 + `#[serial_test::serial]` flake fixes); J-093 stood down Commit 3 when Scenario 1 harness setup walked into failure-mode catalogue M5 ("F-3 brand-new federation bootstrap dead-locked") exactly where the Phase 9 survey predicted. **Phase 7.5 design phase closed J-093 same session** with four `[JOE-LOCK: locked 2026-05-19]` framework decisions; **implementation runbook authored 2026-05-20** at `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md` (Status: ACTIVE, v1.0). Five-commit sequence: (1) doc-pass adding canonical design doc §6.4.1 + §15 row, design task file flipped COMPLETED; (2) F-3 + F-4 step 1 skip for `state.space_create` + `state.dm_space_create` plus new `SpaceLocalMetadata` sibling structure in `xgen-common` with `introducer_node_id` field; (3) HeldPending third trigger "missing federation relationship for (peer, space)" + idempotent `state.federation_add` arrival hook + `drain_pending_by_federation_relationship` + new `[sync].federation_relationship_timeout_seconds` config field (180s default) + new error code `4007 federation_relationship_timeout` + `pending_federation_relationship` counter + `f3_reject` trace event extended with disposition field; (4) NodeRuntime-level integration tests for six scenarios (cold-start end-to-end, mid-bootstrap drop/resume, F-10 + Phase-7.5 combination, two-stage cascade, timeout precedence at 4007 + 4002, negative regression); (5) milestone-internal close commit. After Phase 7.5 ships: **XGID concept work is next-active** per Near future first-in-queue (sequenced between Phase 7.5 close and Phase 9 resume) so test code, trace events, and observability fields use XGID from the start rather than being retrofitted. Then Phase 9 resumes from Commit 3 boundary with Scenario 1 working as originally designed; after Phase 9 ships, milestone flips PLAY → DONE and M6 (new) unblocks. J-092's Commits 1 + 2 (observability + flake fixes) stay shipped through this sequence — pure infrastructure, neither depends on the broken cold-start path. **Entry point: `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md` §3 Commit 1 (doc-pass).**

---

## Near future — designed or scoped, awaiting work

Tracks that are ready to start. Each has known shape, known scope, known dependencies. Listed in roughly the order they will be picked up, though that order is not strictly locked — parallelism is possible between independent tracks.

🟡 **XGID — first-class unique identifier concept.** Named protocol-vocabulary primitive covering all unique IDs in XGen (Event IDs, Space IDs, Room IDs, Identity IDs, Node IDs, TrustAssertion IDs). All flavours unified under one term; underlying construction (content-derived hash for hash-anchored objects, public-key for cryptographic principals) is an implementation detail of XGID, not exposed in the user-facing vocabulary. Three deliverables: (1) canonical definition section in Ch3 introducing XGID before any event/Space/Room sections reference it, with explicit clarification that XGIDs are immutable per object (a Space's XGID is its founding event's hash and never changes as the Space's state evolves); (2) type-system adoption in code via newtype or type alias in `xgen-common`, used in new code from adoption point forward, wire-format field names (`event_id`, `space_id`, etc.) unchanged; (3) DECISIONS.md entry locking name + all-encompassing scope + medium-adoption discipline (no wire-format changes, ever). Parent-type surface is small: `Primitive` + `SignedPrimitive` in `xgen-common` cover Event/Node/TrustAssertion; Space/Room/Identity are derived from an Event's XGID and inherit through the parent; Ed25519-pubkey IDs (Node ID, Identity ID) get separate typing at their pubkey surface — ~3 touch points to type the whole protocol. Originated during Phase 7.5 design walkthrough when "content-derived ID" terminology misled in conversation; brand coherence and type-safety benefits judged worthwhile. **Sequenced after Phase 7.5 closure, before Phase 9 Commit 3 resumption** so test code and observability fields use XGID from the start rather than being retrofitted. Historical journal entries not retroactively rewritten — same rule as Mr Code → Code Claude.

🟡 **M6 Block 4 — verb-by-verb walks.** Pass 3 closed the framework decisions; Block 4 walks all seven verb categories (~35 verbs total) to confirm names, lock argument schemas, lock result schemas, and apply the two-token naming convention. Fills in §6 of the design doc which is currently stubbed per category. Chat Claude + Joe work. Independent of Federation completion — can run in parallel.

🟡 **M6 (new) Node admin write path — implementation.** After Federation Event Propagation implementation AND Block 4 close, M6 implementation starts. Phase plan (locked in M6 Pass 3): Phase 1 client gap patches (optional, possibly zero commits), Phase 2 `admin_ops::*` scaffolding + envelope-level `event_id` + `EventAccepted` wire shape, Phase 3 read-only completions, Phase 4 logging/audit admin (audit primitive lands here), Phase 5 identity registry admin, Phase 6 Bootstrap configuration (smaller, before Federation), Phase 7 Federation management, Phase 8 Auth Module management (may defer if §3.6 revocation cascade is spec-gap), Phase 9 Space/Room operator actions (signing-identity sub-design first), Phase 10 plugin management.

---

## Far future — specced, not yet scheduled

Tracks the project knows about but has not committed timing or shape to. Sketched briefly; detail accumulates if and when they approach.

🟡 **M7 — `--aicontrol` v1 covering both binaries.** AI-driver control surface as a sister flag to `--batch` (per D-066). Canonical design at `docs/xgen_aicontrol_implementation.md`. Reuses `xgen-client-lib::ops::*` and `xgen-node-lib::admin_ops::*` as command-implementation layers; adds JSONL command/reply protocol, persistent sessions, named bindings, real-time event observation, lifecycle-aware errors. Scope spans both binaries from day one.

🟡 **M7 standalone — live config reload.** Originally folded into M6, pulled out at Pass 1 of M6 Phase 0. Realises the `--reload-config` Node verb that today returns honest `NOT_IMPLEMENTED`. Field-bucket already locked (reloadable / restart-required / forbidden) in M6 Phase 0 Pass 3 Joe-lock #2; M7 implements the mechanism.

🟡 **M8 — multiparty improved pass with A/B metrics.** Re-runs the multiparty suite against the post-M7 binary state, fills in the "B" column of the A/B comparison the original M6 was meant to start. Shape and scope re-design at the moment M8 actually starts; the framing today is a placeholder.

🟡 **M9 — Multiparty Redesign.** Inherits the work the original M6 was meant to do, redesigned to measure both binaries' read-write surfaces (`--batch` and `--aicontrol`) rather than the original Client-only `--batch` A/B framing. The metric set in `tasks/BATCH_FLAG_review.md` §"Baseline metrics protocol" is retained as a starting point; M9's design phase may revise it.

### Parallel workstreams

🟡 **Federation stress follow-on.** Stub task file at `tasks/FEDERATION_STRESS_FOLLOWON.md` (Status: PENDING, v1.0). Inherits the four Phase 9 compounds deferred from the Federation Event Propagation milestone (C1, C4, C6, C8) plus the clock-injection seam they require. Blocked on the seam landing as separate infrastructure work; independent of the M-series trunk. Created in J-091 alongside the Phase 9 implementation task file so the deferred scope has a documented home rather than living only in survey-findings prose.

🟡 **D3 — MLS operationalisation.** Wire shape already specced (Ch3 §3.10, Appendix I Part X.6); openmls integration pending. Runs as an independent parallel workstream alongside the M-series per D-066. Timing is open.

⏸️ **UI Phase 2 visual merge — postponed at element-modelling step (J-033).** The Tauri scaffolding, lifecycle state machines, named pipe IPC, `--instance` segregation, `--batch` flag, and xgen-core crate split landed during Sessions 14–18. The deeper visual merge (chat mockups' visual treatment onto Miss Design's semantic structure) is postponed. **Resume condition:** confirmed absent-element list in `ui/docs/xgen-ui-design-brainstorm.md` (Points 2 and 3) reconciled with Ch3's authoritative event taxonomy, plus Run 3 design briefing drafted. Until those gate, no visual merge work begins.

### Open areas (specced but unimplemented)

🟡 **State migration depth.** Wire shape specced (Ch3 §3.12, Layer 14); deep testing pending. Folded into M8 conceptually but timing is open.

🟡 **Federation depth (post-completion).** Once the Federation Event Propagation milestone closes the foundational gap, deeper federation work surfaces: N-Node topologies, defederation flow polish, reputation merge across peers. Folded into M8 conceptually but separate from the foundational work.

🟡 **`self` account.** Local-only synthetic Identity, accessible from any client (D-021 area). Deferred; design work has not started.

🟡 **Registry file encryption.** Identity and federation registries at rest. Deferred.

🟡 **DPI resistance.** Investigation only at this stage (D-023). Phase 3 area.

🟡 **Slovak translation pass.** Suspended during active English development; single pass after full document completion. Deferred until English documentation reaches a stable end-state.

---

## Cross-cutting

A few items don't fit cleanly in past / present / near future / far future because they are continuous rather than milestone-shaped. Recorded here for visibility.

🟢 **Design discipline (D-069).** Every milestone Phase 0 must be Joe-locked before the implementing phase starts. Delegated technical drafts must self-flag open items. Canonical-document rule: each major surface gets one authoritative document, others point at it. M6's Phase 0 was the first milestone to follow this discipline end-to-end; Federation Event Propagation's design phase (Pass 2 just closed, Pass 3 next) is the second instance of the same pattern.

✅ **Audit-precedes-dependency discipline (D-071).** Every future milestone's Phase 0 includes a subsystem audit of whatever the milestone depends on. The Propagation Reliability Audit (J-081) established the pattern; D-071 names the discipline. Pairs with D-069: audit phase → design phase → implementation phase, each producing a canonical artefact. Sibling to D-065 and D-070. Promoted to DECISIONS.md 2026-05-18.

🟢 **Honest behaviour over polite behaviour (D-065).** Protocol-design principle. When the system can choose between a behaviour that misrepresents its state and one that honestly reflects it, XGen picks honest. Surfaces in multiple places: AI Client drop-on-throttle pacing, Node event rejection clarity, mute semantics, M6 accept-signal design, Federation Pass 2's `sync_complete` lock (F-6 chose explicit signal over silent quiet-time heuristic citing D-065), Federation Pass 2's pagination lock (F-7 chose explicit cursor over "felt incomplete" heuristic citing D-065).

✅ **Two events of equal importance, opposite direction (D-070).** Protocol-design principle. When the protocol exposes a signal from one party to another about the outcome of an action, both directions of outcome (acceptance and rejection) must be exposed with equal first-class status, AND both directions must carry the envelope-level correlation identifier so the originator can correlate the signal to the action it sent. Sibling to D-065. Promoted to DECISIONS.md 2026-05-18 with corrected post-audit framing (both halves — existence AND correlation — are load-bearing).

🟢 **Honest longer work over fast shortcuts.** Project-management principle. When project work surfaces a real gap, the default response is to close the gap properly, even if that delays downstream work. Locked during the audit's federation finding discussion; informs all milestone-sequencing calls. Pairs with the audit-precedes-dependency discipline above. Federation Pass 2 invoked it three times — to fold in F-6 (sync_complete) rather than defer, to fold in F-7 (pagination) rather than defer, and to fold in F-10 (HeldPending generalisation) rather than reject.

---

## How to read this document

A reader landing here for the first time should be able to answer three questions in under a minute:

1. **What has the project shipped?** Past section, scanned by reading the bold milestone names.
2. **What is being worked on right now?** Present section.
3. **What's next?** Near future, top-of-list.

A reader returning after a gap can scan for state-changes by looking for symbols that have moved (a 🟢 PLAY that became ✅ DONE, a 🟡 PENDING that became 🟢 PLAY, a new 🟡 entry that wasn't there before). The roadmap is meant to be scannable, not exhaustively read.

For any item the reader wants more detail on, the canonical source is named (JOURNAL entry, DECISIONS reference, design doc, task file). ROADMAP.md is the map; the territory lives elsewhere.

---

*End of roadmap.*  
