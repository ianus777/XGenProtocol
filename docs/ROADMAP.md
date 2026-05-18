# XGen Protocol — Project Roadmap
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-18 (J-082 — Federation Event Propagation implementation Phase 1 SHIPPED: F-6 `sync_complete` wire shape + F-7 pagination + four-call-site migration. 476 tests. Track moved from Near future to Present per the discipline section; Past gains a Phase-1 close entry under Federation Event Propagation implementation.)  
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

### Federation Event Propagation implementation — Phase 1

✅ **Federation Event Propagation implementation Phase 1 SHIPPED 2026-05-18** (J-082, 476 tests). F-6 (`transport.sync_complete`) + F-7 (response-size pagination) shipped together as one coordinated wire-protocol change per design doc §9 + §10. `TransportMessage::SyncRequest::limit: Option<u32>` and new `TransportMessage::SyncComplete { since, new_tip, continue_from }` variants with `#[serde(skip_serializing_if = "Option::is_none")]` for backward-compat. `collect_sync_history` signature changed to `(runtime, requester_id, since, limit) -> (Vec<Event>, Option<String>)` honouring `limit` and emitting `continue_from`. Four production callers migrated from quiet-time / hardcoded-deadline to SyncComplete-driven pagination loops with F-6b safety-net timeout: `xgen-client/src/batch.rs:get_dag_tips`, `xgen-client/src/ai_service.rs:224`, `xgen-client/src/ops.rs:721` (`history`), `xgen-client/src/ops.rs:939` (`ai_status`). Cross-Space behaviour Clair-locked as whole-batch (rationale at `xgen-node/src/fanout.rs:177-201`). `[sync]` config section on both binaries with `completion_timeout_seconds` (default 5) and `batch_size` (default 1000), `#[serde(default)]` so existing on-disk configs keep parsing. Tests: 468 → 476 (+7 unit in `xgen-node-lib::fanout` covering wire roundtrip and pagination semantics, +1 integration in `xgen-client/tests/sync_safety_net.rs` covering the safety-net error path). Federation Event Propagation implementation track moved from Near future to Present in this same commit.

### Federation Event Propagation design phase — Pass 3

✅ **Federation Event Propagation Pass 3 closed 2026-05-18** (468 tests, single same-day session following Pass 2, no code changes). Design phase closed; canonical document shipped; runbook handed off. Five deliverables in one coordinated commit: (1) canonical design doc consolidated to v1.0 ACTIVE at `docs/xgen_federation_propagation_design.md` — three Pass-2 addenda folded in as §10 (F-7), §11 (F-8), §12 (F-9), §13 (F-10), addendum files deleted, version bumped from v0.6, Status flipped from PENDING to ACTIVE; (2) all `[JOE-LOCK]` markers walked from "confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3" to final form `[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`; (3) F-8 corrections applied to `docs/xgen_ch4_implementation.md` §4.11.3 + §4.12.3 (forward-references to canonical design doc; located by content match against unique phrases rather than the audit's stale line numbers); (4) F-9 correction applied to `docs/xgen_node_admin_ops_design.md` §4.2 (Federation propagation Stage-6 sub-bullet now a forward-reference); (5) implementation runbook created at `tasks/FEDERATION_PROPAGATION_COMPLETION.md` (Status: ACTIVE, v1.0) — nine phases with hard-locked Phase 2 → Phase 4 ordering. Pass 2 task file (`tasks/FEDERATION_PROPAGATION_DESIGN.md`) and Pass 3 task file (`tasks/FEDERATION_PROPAGATION_PASS_3.md`) both flipped to COMPLETED in the same commit. Federation Event Propagation milestone block in CLAUDE.md flipped from 🟡 PENDING to 🟢 ACTIVE; implementation track in this document went 🟡 PENDING in Near future.

### Named decisions promoted to DECISIONS.md

✅ **D-070 promoted 2026-05-18** (468 tests, same-day post-Pass-3, no code changes). "Two events of equal importance, opposite direction" — named protocol principle. The original draft in `docs/xgen_node_admin_ops_design.md` §9 framed the principle as "EventAccepted exists, symmetric to Error." The Propagation Reliability Audit (J-081 §5) found that framing was necessary but not sufficient: `TransportMessage::Error` lacked an `event_id` field at all, meaning even with both Error and a future EventAccepted, the originator couldn't correlate either signal back to a specific event. D-070's DECISIONS.md entry incorporates the corrected post-audit framing: BOTH (1) both directions of outcome exist (acceptance + rejection signals), AND (2) both directions carry the envelope-level correlation identifier (`event_id: Option<String>` on `TransportMessage`). Without (2), (1) is hollow. M6 §9 draft preserved as historical record; DECISIONS.md D-070 is the canonical authoritative form. M6 (new) Phase 2 implements both halves in coordinated work with Federation Event Propagation milestone's F-4 rejection-site changes.

✅ **D-071 promoted 2026-05-18** (468 tests, same-day post-D-070, no code changes). "Subsystem audits precede dependent milestones" — project-management principle. Every future milestone whose correctness depends on a load-bearing subsystem MUST include a subsystem audit as part of its Phase 0 (design phase). Audits produce code-grounded canonical documents and surface gaps that may need to close as preconditions of the milestone rather than as parallel work. The pattern emerged organically during the Propagation Reliability Audit (J-081), where findings consistently exceeded the audit's nominal scope (four HIGH-severity findings across five sections) and the audit became Pass 1 input for two downstream design phases (M6 Phase 0, Federation Event Propagation Phase 0). D-071 pairs with D-069: audit phase (D-071) → design phase (D-069) → implementation phase, each producing a canonical artefact. Sibling to D-065 and D-070 — D-065 and D-070 are protocol-design principles; D-071 is the project-management analogue. The shared theme across all three: don't let assumed-state substitute for verified-state.

### Deprecated tracks

⬛ **M6 (original) Multiparty baseline pass** — descoped 2026-05-17. Original M6 plan (run the full Multiparty suite S1–S5 twice through present `--batch` to fill the "A" baseline column) descoped because the binary state had shifted post-J-079, the metric-protocol applicability needed reconfirmation, and the bigger problem M6 was meant to solve had grown to span both binaries. **Replaced by M9 Multiparty Redesign** at the end of the M-series trunk; the M6 slot is reused for the Node admin write path. Affected task files (`tasks/MULTIPARTY_S1_tauri_rerun.md`, `tasks/MULTIPARTY_S2_to_S5_present_pass.md`) carry the DEPRECATED status with the M9 pointer.

⬛ **`tasks/NODE_ADMIN_PASS2_PROPOSALS.md`** — superseded by `docs/xgen_node_admin_ops_design.md`. The Pass 2 proposals file was the working document for Pass 3's lock-decisions; once the design doc shipped with the framework decisions filled in, the proposals file lost its operational role. Kept in `tasks/` as historical predecessor per D-069's canonical-document rule.

---

## Present — playing now

The track or tracks the project is actively working on right now. Detail-level here is the most granular in the document — what's in flight, what's blocking what, what the next concrete step is.

🟢 **Federation Event Propagation implementation — Phase 1 SHIPPED (J-082), Phase 2 next-active.** The nine-phase runbook at `tasks/FEDERATION_PROPAGATION_COMPLETION.md` (Status: ACTIVE, v1.0) drives this milestone. **Phase 1** (F-6 `transport.sync_complete` wire shape + F-7 response-size pagination on `collect_sync_history` + four-call-site migration from quiet-time / hardcoded-deadline to SyncComplete-driven pagination loops with F-6b safety-net) closed in J-082 on 2026-05-18. 476 tests passing (+7 unit, +1 integration over the 468-test handoff baseline). Cross-Space SyncComplete behaviour Clair-locked as whole-batch (single cursor across flattened event sequence; rationale at `xgen-node/src/fanout.rs:177-201`). `[sync]` config section on both binaries with `completion_timeout_seconds` default 5 and `batch_size` default 1000. **Next: Phase 2 — `process_inbound` validation pipeline unification (F-4).** Refactor to the F-4 §7.7 dispatcher shape (structural pre-checks → federation-relationship check → unified validation core → semantic pre-checks → event-type-specific handlers → fan-out). HeldPending moves out of `accept_message` into a shared module reachable from all three event families. Hard precondition for Phase 4 (federation event push) per runbook §8 — Phase 4 before Phase 2 lands the audit's HIGH-severity vulnerability vector. Remaining work after Phase 2: Phase 3 (federation handshake reshape to tip exchange, F-1a) → Phase 4 (federation event push, F-1 + F-1b + F-5) → Phase 5 (per-peer record + reconnect scheduling, F-1c) → Phase 6 (HeldPending generalisation for unknown signer Identity, F-10) → Phase 7 (federation-relationship verification gate, F-3 second check) → Phase 8 (documentation pass) → Phase 9 (integration tests for full federation push path). Blocks M6 (new) going ACTIVE.

---

## Near future — designed or scoped, awaiting work

Tracks that are ready to start. Each has known shape, known scope, known dependencies. Listed in roughly the order they will be picked up, though that order is not strictly locked — parallelism is possible between independent tracks.

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
