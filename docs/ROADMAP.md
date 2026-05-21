# XGen Protocol — Project Roadmap
> **Status**: ACTIVE  
> Version: 1.5  
> Date: May 2026  
> **Last updated**: 2026-05-21 (Pass 1 title rename "xgen-common core types" → "core data structures" in both the Visual structure tree and the Near future prose paragraph. Surfaced during Pass 1 runbook authoring when reconnaissance of the actual struct locations showed `SpaceState`, `FederationRegistry`, `IdentityRegistry`, `PendingBuffer` live in `xgen-core`, not `xgen-common`, contradicting the "xgen-common core types" section title. Joe-locked Option B (Pass 1 = core data structures regardless of crate, deliberately spanning xgen-common and xgen-core) over Option A (Pass 1 = xgen-common only with structures sliding to Pass 2). Reasoning: (1) Phase 2 doc-tree sweep's coordination flag pins Appx C + Appx I to Pass 1 in one commit set, splitting the data structures across two Passes would split Appx I documentation in a way the canonical-document rule (D-069) prohibits; (2) Appx I atomicity matters more than crate purity; (3) the honest seam analysis favours retyping both sides of the data-structure / algorithm boundary atomically, with Pass 2's seams moving outward to the algorithm layer (validate_event, dispatch_event, registry method APIs, accept_message). Shipped same-commit as `tasks/XGID_RETROFIT_PASS_1_IMPL.md` skeleton creation per the title-must-reflect-runbook-scope discipline. Previous v1.4 drift-reconciliation content (Phase 9 tree row + Visual structure guardrail) stands authoritative — see prose Present section.)  
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

## Visual structure — nested view

A structural at-a-glance of the project's milestone hierarchy. Past entries are collapsed to milestone-level depth; Present, Near future, parallel workstreams, open areas, the discipline cluster, and cross-cutting principles are shown at full nesting. The tree is a navigation aid — the prose sections below remain canonical for detail. If the tree and prose ever disagree, prose wins; the disagreement is itself a discipline failure that the next state-change commit must reconcile.

**Same-commit discipline applies to the tree, no exceptions.** When updating ROADMAP.md for any state change, the tree above MUST be updated in the same edit, even if the prose change feels small. The tree's value is at-a-glance correctness; if it's stale, it actively misleads rather than just being unhelpful. Drift between tree and prose is treated as the same shape of discipline failure as drift between ROADMAP.md and CLAUDE.md.

```
XGen Protocol
│
├── Federation Event Propagation milestone (🟢 Phase 9 next-active)
│   ├── ✅ Pass 2 design (10 F-items locked)
│   ├── ✅ Pass 3 design (canonical doc + runbook shipped)
│   ├── ✅ Phase 1 implementation (F-6 + F-7 wire shape)
│   ├── ✅ Phase 2 implementation (F-4 validation unification)
│   ├── ✅ Phase 3 implementation (F-1a tip exchange)
│   ├── ✅ Phase 4 implementation (F-1 federation push)
│   ├── ✅ Phase 5 implementation (F-1c per-peer record + reconnect)
│   ├── ✅ Phase 6 implementation (F-10 unknown-signer Identity)
│   ├── ✅ Phase 7 implementation (F-3 federation-relationship gate)
│   │   └── ✅ Phase 7 B3 amendment (predecessor-chain + step-11 closure)
│   ├── ✅ Phase 7.5 milestone (cold-start bootstrap)
│   │   ├── ✅ Phase 7.5 design (4 framework decisions)
│   │   └── ✅ Phase 7.5 implementation (5 commits, JOURNAL gap flagged)
│   ├── ✅ Phase 8 implementation (documentation pass)
│   └── 🟢 Phase 9 implementation (deployment integration tests)
│       ├── ✅ Phase 9 survey (14 failure-mode catalogue + 12 scenarios)
│       ├── ✅ Phase 9 Commit 1 (G1 observability)
│       ├── ✅ Phase 9 Commit 2 (flake fixes)
│       └── 🟢 Phase 9 Commit 3 onwards (6 DoD scenarios — RESUMED 2026-05-20, Clair's next session)
│
├── XGID Adoption v1 milestone (✅ DONE 2026-05-20)
│   ├── ✅ Design walkthrough (Q1–Q6 locked, 2 sessions)
│   ├── ✅ Phase 1 canonical sources (8-artefact atomic commit, a5f3c8b)
│   ├── ✅ Phase 2 doc-tree sweep (classification table, 70e3e5a)
│   │   ├── ✅ Pre-walk Scope-A-vs-Scope-B Joe-lock (Scope B)
│   │   ├── ✅ Pre-walk SK appendix housekeeping (2 files → DEPRECATED)
│   │   └── ✅ 23-doc classification walk (6 groups A–F)
│   └── ✅ Implementation (Clair's 2 production commits + hygiene + close, J-095)
│       ├── ✅ Commit 1 (c95584a) — xgen-common XGID types + 5 invariance tests
│       ├── ✅ Commit 2 (24a255b) — SpaceLocalMetadata.introducer_node_id retype
│       ├── ✅ Hygiene commit (904441b) — workspace clippy under Rust 1.95.0 (NOT XGID code)
│       └── ✅ Milestone-close commit — JOURNAL J-095 + Ch4 pointer + cross-doc flips
│
├── XGID Retrofit Pass series (🟡 Near future, 5 Passes)
│   ├── 🟡 Pass 1 — core data structures (spans xgen-common + xgen-core)
│   │   ├── (code, xgen-common) Event struct field retypes
│   │   ├── (code, xgen-common) SpaceLocalMetadata.space_id retype
│   │   ├── (code, xgen-common) state.rs observability struct retypes
│   │   ├── (code, xgen-core) SpaceState field retypes
│   │   ├── (code, xgen-core) FederationRegistry / IdentityRegistry / PendingBuffer keys
│   │   ├── (code, xgen-common) carry-over: `canonical_event_bytes` module move xgen-core → xgen-common
│   │   ├── (code, xgen-common) carry-over: deferred hash-anchored convenience constructors
│   │   ├── (doc) Appendix C primitive schemas
│   │   ├── (doc) Appendix I data structures
│   │   └── [coordination flag: code + Appx C + Appx I in ONE commit set]
│   ├── 🟡 Pass 2 — xgen-core (code-only, zero doc work)
│   │   ├── (code) validate_event, ValidationOutcome
│   │   ├── (code) NodeRuntime::dispatch_event, DispatchOutcome
│   │   ├── (code) PendingBuffer arrival hooks
│   │   ├── (code) FederationRegistry / IdentityRegistry APIs
│   │   └── (code) accept_message signature
│   ├── 🟡 Pass 3 — xgen-node + Appendix D
│   │   ├── (code) federation_session, fanout, app handlers
│   │   ├── (code) reconnect scheduler
│   │   ├── (code) pipe server admin verbs (post-M6)
│   │   └── (doc) Appendix D storage/privacy field tables
│   ├── 🟡 Pass 4 — xgen-client + AI control docs (heaviest doc-work pass)
│   │   ├── (code) ops::* layer
│   │   ├── (code) AiBehavior trait, EchoPlugin, AiPacingTracker
│   │   ├── (code) session state, batch dispatcher, CLI dispatcher
│   │   ├── (code) AI service, Tauri commands
│   │   ├── (doc) Appendix F Client-side sections — full per-section annotation
│   │   ├── (doc) xgen_aicontrol_implementation.md — full per-section annotation
│   │   └── (doc) Ch6 §6.15 client-side spec
│   └── 🟡 Pass 5 — test fixtures, helpers, remaining surfaces
│       ├── (code) test fixture builders
│       ├── (code) integration test helpers
│       ├── (code) trace event field types
│       ├── (code) log line formatters
│       ├── (code) debug/Display impls
│       ├── (doc) Appendix G log line convention
│       └── [at close: wire-format invariance promise promoted to cross-cutting]
│
├── M-series trunk
│   ├── ✅ M1 Binary Consolidation
│   ├── ✅ M2 Node Pipe Server
│   ├── ✅ M3 AI Operator Role
│   ├── ✅ M4 AI Client Binary
│   ├── ✅ M5 ops::* refactor
│   ├── ⬛ M6 (original) Multiparty baseline — DEPRECATED, replaced by M9
│   ├── 🟢 M6 (new) Node admin write path — Phase 0 design closed
│   │   ├── ✅ Phase 0 design (3 passes, 12 framework decisions locked)
│   │   ├── 🟡 Block 4 — verb-by-verb walks (~35 verbs, parallel-eligible)
│   │   └── 🟡 Phases 1–10 implementation (post-Federation, post-Block 4)
│   ├── 🟡 M7 — --aicontrol v1 covering both binaries
│   ├── 🟡 M7 standalone — live config reload
│   ├── 🟡 M8 — multiparty improved pass with A/B metrics
│   └── 🟡 M9 — Multiparty Redesign
│
├── Parallel workstreams
│   ├── 🟡 Federation stress follow-on (4 deferred Phase 9 compounds)
│   ├── 🟡 D3 — MLS operationalisation
│   ├── ⏸️ UI Phase 2 visual merge (postponed at element-modelling step)
│   └── 🟡 Slovak translation pass (after English documentation stabilises)
│       └── [first concrete touchpoint: xgen_appendix_a_sk.md + _b_sk.md DEPRECATED 2026-05-20]
│
├── Open areas (specced but unimplemented)
│   ├── 🟡 State migration depth (folded into M8)
│   ├── 🟡 Federation depth post-completion (folded into M8)
│   ├── 🟡 `self` account
│   ├── 🟡 Registry file encryption
│   └── 🟡 DPI resistance (Phase 3 area)
│
├── Discipline / JOURNAL hygiene (🟡 small follow-ups)
│   ├── 🟡 JOURNAL Gap 1 — Phase 7.5 implementation retrospective entry
│   │   └── (commits 12cfe5a + aa2433f + 1be7189 + ecbbf19 + 8859093)
│   ├── 🟡 JOURNAL Gap 2 — XGID Adoption v1 design+Phase 1 retrospective entry
│   │   └── (commit a5f3c8b)
│   └── 🟡 D-074 candidate — "every milestone-close commit MUST include JOURNAL.md"
│       └── (sibling to D-069/D-070/D-071, surfaced 2026-05-20 during J-094 cleanup)
│
└── Cross-cutting principles (continuous, not milestone-shaped)
    ├── 🟢 D-069 — Design discipline (canonical-document rule)
    ├── ✅ D-070 — Two events of equal importance, opposite direction
    ├── ✅ D-071 — Audit-precedes-dependency discipline
    ├── ✅ D-072 — XGID Adoption v1
    ├── ✅ D-073 — Field-name-vs-type discipline
    ├── 🟢 D-065 — Honest behaviour over polite behaviour
    └── 🟢 Honest longer work over fast shortcuts (not yet promoted to D-NNN)
```

### How to use this view

The tree is read top-down for navigation: scan to the cluster you care about, then drop into the prose Past/Present/Near future/Far future sections below for the detail. Status icons match the legend; nesting depth reflects logical containment, not strict commit order.

Three views fall out naturally:

- **What's settled?** Scan for ✅ marks. Most of Federation Event Propagation, all of XGID Adoption v1, M1–M5, D-070–D-073.
- **What's playing right now?** Scan for 🟢 marks. Phase 9 next-active; M6 (new) framework decisions locked, Block 4 + implementation pending; D-065, D-069, and the unnamed "honest longer work over fast shortcuts" principle apply continuously.
- **What's the live frontier?** Three items are parallel-eligible and ready to pick up: Phase 9 Commit 3 onwards (Clair); XGID Retrofit Pass 1 runbook authoring (Chat Claude); M6 (new) Block 4 verb-by-verb walks (Chat Claude + Joe). Everything else is either done, waiting on one of those, or further out.

Depth-asymmetry note: settled Past clusters (M-series M1–M5, completed Federation phases) show one line per milestone with no internal phase breakdown, mirroring the prose section's principle that detail accumulates as a track approaches and reduces when it settles. Live and Near-future clusters show full internal nesting because that detail is currently load-bearing for navigation.

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

### Federation Event Propagation implementation — Phase 7.5 (Cold-Start Bootstrap)

✅ **Federation Event Propagation Phase 7.5 implementation SHIPPED 2026-05-20** (five commits `12cfe5a` + `aa2433f` + `1be7189` + `ecbbf19` + `8859093`, 519 → 556 tests, +37 across the milestone; **JOURNAL entry not written at milestone-close time** — the cross-doc "J-094" reference originally placed here pointed at an entry that was never authored, a Rule 4 discipline failure surfaced during XGID Adoption v1 Phase 2 close-out 2026-05-20 and confirmed via working-tree forensics; entry deferred to separate retrospective work per D-065 honest-provenance, will take next available J-number when written). Cold-start bootstrap closure shipped per `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md` (Status: COMPLETED v1.0 at milestone close). Five commits: (1) doc-pass adding canonical design doc §6.4.1 + §15 row, design task file flipped COMPLETED; (2) F-3 + F-4 step 1 skip for `state.space_create` + `state.dm_space_create` plus new `SpaceLocalMetadata` sibling structure in `xgen-common` with `introducer_node_id` field; (3) HeldPending third trigger "missing federation relationship for (peer, space)" + idempotent `state.federation_add` arrival hook + `drain_pending_by_federation_relationship` + new `[sync].federation_relationship_timeout_seconds` config field (180s default) + new error code `4007 federation_relationship_timeout` + `pending_federation_relationship` counter + `f3_reject` trace event extended with disposition field; (3.5) **Phase 7 B3 amendment** (`tasks/FEDERATION_PROPAGATION_PHASE_7_B3_AMENDMENT.md`, COMPLETED v1.0, commit `ecbbf19`) closing two latent Phase-7 gaps surfaced by Commit 4 integration tests: predecessor-chain deadlock + step-11 sender-membership rejection for `state.federation_add` arriving via federation channel; sibling-to-B1 framing (not P7.5-A extension) honest about attribution; skip set widened from step-9 + step-11 + step-13 to step-11 in full (both halves) after Q3-overload code trace established `IdentityRegistry::contains` is Identity-only while `verify_event_signature` works for Node URIs via pubkey-URI decoding; step-12 retained as authority anchor; (4) folded into Commit 3.5 — NodeRuntime-level integration tests written + green within the B3 implementation commit (six scenarios: cold-start end-to-end, mid-bootstrap drop/resume, F-10 + Phase-7.5 combination, two-stage cascade marked not-applicable post-B3 since federation_add no longer F-10-buffers via federation channel, timeout precedence at 4007 + 4002, negative regression); (5) milestone-internal close commit (this entry's commit). Architectural surfaces of note: federation-relationship arrival hook lifted from `xgen-node::app::process_inbound` into `dispatch_event` Step 7 so every caller exercises it under the runtime lock (mirror of Phase 6's Identity-hook architecture); `resolve_federation_relationship` gained `reindex_after_partial_release` helper preventing buffer-entry orphan when sibling drain-released events haven't ingested due to HashSet-order non-determinism (a bug shape that only surfaces with three HeldPending triggers, not two); `drain_timed_out` dropped implicit `.max(default_timeout)` so operator's configured federation timeout wins. Test count 519 → 556 across the milestone (+37: +5 B3 unit tests + 7 cold-start integration tests + 25 across Commits 2 + 3 dispatch / pending-buffer / config-field / counter / state-file roundtrip / SpaceLocalMetadata persistence / trace-event-disposition / etc.). Failure-mode catalogue M5 ("F-3 brand-new federation bootstrap dead-locked") structurally closed. Phase 9 stays PAUSED at Commit 3 boundary with XGID Adoption now the active blocker. The B3 mid-implementation Joe-lock walkthrough is recorded discipline-wise as J-081-shape: the D-071 audit-precedes-dependent pattern extending two levels finer ("design gaps surface during implementation testing and close before the implementation proceeds"), Phase 7.5's J-093-shape pattern recursing one more level into Commit-4 integration-test surface.

### XGID Adoption v1

✅ **XGID Adoption v1 design walkthrough closed + Phase 1 canonical sources commit shipped 2026-05-20** (single session, no test count change — documentation and design only). Walkthrough ran across two same-day sessions: session A opened Q1–Q4 partial and produced the mid-walkthrough design-phase restructure from five-upfront-deliverables to canonical-sources-first / doc-tree-sweep-second; session B closed Q4 final framing + Q5 + Q6 and drafted the Phase 1 canonical sources commit. **Six locked decisions Q1–Q6:** Q1 vocabulary scope (six XGID flavours — Event, Space, Room, TrustAssertion (hash-anchored family) + Node, Identity (principal family); session_id and trust_assertion_id explicitly placed as sub-axes within existing flavours, not new flavours; wire-envelope correlation handles, error codes, config keys, file paths, bootstrap URIs explicitly NOT XGIDs); Q2 type-system shape (layered newtype — base `Xgid(String)` plus six flavour wrappers each `Deref<Target = Xgid>`, all serde-transparent as plain strings; `XgidLike` trait sparingly used; principal flavours carry parse-fallible `pubkey() -> Result<VerifyingKey, _>` at v1, future-tightening to infallible deferred); Q3 adoption discipline (Shape γ + ASAP — staged retrofit, five passes immediately into Near future; principle wording locked verbatim into D-072 + Appendix J §J.11 + Ch3 §3.0.6); Q4 wire-format invariance (three-document placement + five-invariance scope + both wire crossings explicitly named including AI control / batch JSONL as named second wire crossing; §J.9 ships with two worked rejected-proposal examples (Cases #5 "use in-memory handle type as wire type" and #2 "shorten URI grammar for compactness") + reserved third slot; v1 pointers in `xgen_aicontrol_implementation.md` and Appendix F per Q4(b) lock); Q5 immutability framing (Option C layered — Ch3 §3.0.2 declarative sentence + Appendix J §J.4 construction-derived explanation + §J.10 "rename a Space" worked example demonstrating the property is structural, not policy); Q6 field-name-vs-type discipline canonicalisation (Option A — D-073 in DECISIONS.md as canonical home for the principle, one-sentence echo in Appendix J's introduction, Phase 7.5 §5.6 retained as historical originating-precedent record). **Phase 1 canonical sources commit shipped same session** — eight artefacts in one atomic commit: DECISIONS.md D-072 + D-073, `docs/xgen_appendix_j_en.md` (twelve sections), `docs/xgen_ch3_specification.md` §3.0 (six subsections, inserted as new §3.0 before §3.1 Wire Format), `tasks/XGID_ADOPTION_IMPL.md` (two-commit Clair-facing runbook with five required invariance test names pinned: `xgid_serializes_as_plain_string`, `xgid_deserializes_from_plain_string`, `flavour_wrapper_is_serde_transparent`, `event_xgid_roundtrip_through_event_canonical_form`, `node_xgid_roundtrip_through_handshake_message`), one-line normative pointers in `docs/xgen_aicontrol_implementation.md` and `docs/xgen_appendix_f_en.md` ("XGID discipline applies; full annotation pending Retrofit Pass 4"), `tasks/XGID_ADOPTION_DESIGN.md` (retrospective design walkthrough record — Status: COMPLETED from creation with explicit provenance disclosure per D-065 honest-behaviour principle, the document captures the path the walkthrough took and the Phase 2 sweep flag for Ch6 §6.15 Scope-A-vs-Scope-B classification), CLAUDE.md PLAY block refreshed. Mid-session restructure recorded in the design task file as a worked instance of "honest longer work over fast shortcuts" — the original five-upfront-deliverable shape kept finding affected docs piecemeal during Q4 walkthrough, restructure to canonical-sources-first plus dedicated Phase 2 doc-tree sweep absorbed the piecemeal problem by deferring per-doc classification until the canonical sources existed and a uniform classification rule could be applied. Phase 9 stays PAUSED at Commit 3 boundary; XGID Adoption v1 milestone closes when Clair ships the two-commit implementation, then Phase 9 resumes with integration test code using XGID types from start.

✅ **XGID Adoption v1 implementation milestone CLOSED 2026-05-20** (J-095; Clair's two-commit plan shipped per `tasks/XGID_ADOPTION_IMPL.md` flipped to Status: COMPLETED v1.1; test count 556 → 571 across the milestone, +15: 10 in-module flavour tests + 5 wire-format invariance tests). **Commit 1 (`c95584a`)** ships `xgen-common/src/xgid/` module: base `Xgid(String)` newtype `#[serde(transparent)]` + six flavour wrappers (`EventXgid`, `SpaceXgid`, `RoomXgid`, `TrustAssertionXgid`, `NodeXgid`, `IdentityXgid`) each `Deref<Target = Xgid>` and serde-transparent + `XgidLike` trait + `XgidDecodeError` + hash-anchored `from_canonical_bytes(&[u8])` constructors (matching `xgen-core::crypto::hashing::hash_uri` byte-for-byte) + principal `from_pubkey(&VerifyingKey)` infallible + principal `pubkey() -> Result<VerifyingKey, _>` parse-fallible-at-v1. New `xgen-common` deps: `ed25519-dalek = "2"`, `sha2 = "0.10"`, `base64 = "0.21"`, `thiserror = "1"` (all matching xgen-core's pinned versions). Five required invariance tests pinned by name in `xgen-common/tests/xgid_invariance.rs`: `xgid_serializes_as_plain_string`, `xgid_deserializes_from_plain_string`, `flavour_wrapper_is_serde_transparent`, `event_xgid_roundtrip_through_event_canonical_form`, `node_xgid_roundtrip_through_handshake_message`. Plus 10 in-module flavour tests in `flavours.rs` (legacy-format URI byte-equal locks for both hash-anchored and principal flavours, decode rejection paths, Deref chain, into_xgid consumption, XgidLike trait unification). No production code outside xgen-common modified per Commit 1 DoD. **Commit 2 (`24a255b`)** retypes `SpaceLocalMetadata.introducer_node_id` from `Option<String>` to `Option<NodeXgid>` — the v1 inaugural production use of a typed XGID flavour, instantiating D-073's field-name-vs-type discipline at the use site. Three files: `xgen-common/src/space_local.rs` (struct + constructor + strengthened `serde_roundtrip_with_introducer` test acting as wire-format invariance witness including forward-compat from pre-XGID JSON shape — the per-call-site witness for Appendix J §J.5 invariance 2), `xgen-core/src/node/runtime.rs` (production caller wraps wire-authenticated peer ID via `NodeXgid::from_xgid(Xgid::new(peer.to_string()))` at type-boundary entry point, with code comment flagging Retrofit Pass 3 will widen `dispatch_event(peer_node_id: Option<&str>)` to `Option<&NodeXgid>` at which point the wrap collapses; two test assertions updated from `.as_deref()` to `.as_ref().map(|n| n.as_str())` for &str projection back to pre-retyped peer_id strings), `xgen-node/src/tests/cold_start_bootstrap_integration.rs` (same single-line read-side projection). Honest-broadening warning held throughout — six adjacent String-typed XGID fields presented as tempting parallel retypes (`dispatch_event(peer_node_id: Option<&str>)`, `peer_id: String` variables in F-3/F-4 dispatch graph, `space_id: String` in SpaceLocalMetadata and parallel sites) all deliberately left untouched, each belongs to its own Retrofit Pass where the surrounding subsystem retrofit lands. **Hygiene atom (`904441b`, `chore(workspace): clippy cleanups for new toolchain`)** shipped same-session as separate provenance-distinct sibling commit — workspace clippy gate flipped red → green under Rust 1.95.0; 26 files touched across all four crates; +191/-89 LOC. One behaviour-adjacent fix (`filter_map(|l| l.ok())` → `map_while(Result::ok)` on `std::io::Lines` in `xgen-node/src/pipe.rs` + `xgen-client/src/batch.rs`, closing a potential spin-forever loop on persistent read errors). Rest mechanical (lifetime elisions, `or_insert_with` → `or_default`, `vec!` → array literal, `map_or(false, …)` → `is_some_and(…)`, `assert_eq!(x, true)` → `assert!(x)`, `&[x.clone()]` → `std::slice::from_ref(&x)`, `if let Ok(_)` → `.is_ok()`, match-single-arm → `if let`, `.max(N).min(M)` → `.clamp(N, M)`) or `#[allow]` with per-case rationale comments (notable: `clippy::manual_clamp` on `clamp_temperature` because NaN handling is load-bearing — `f64::clamp` propagates NaN, spec requires NaN → 0.0; `clippy::result_large_err` on `HandshakeError` module because real fix is boxing the variant which belongs to a future error-type-size discipline pass; `clippy::wildcard_in_or_patterns` on the `VISIBILITY_MODERATOR | _` arm as documentary form per spec 3.7.13.3; `clippy::needless_range_loop` file-level in `xgen-client/src/app.rs` because integration-test loops index multiple parallel arrays by the same index). Test count unchanged at 556 (no behaviour regression). Hygiene commit's diff contains zero XGID code — honest provenance via separate atom so a future reader running `git log -- xgen-common` will see hygiene and XGID as distinct commits. **Milestone-close commit (this commit)** carries five files: JOURNAL.md (J-095 entry), CLAUDE.md (PLAY block flip + header bump), `docs/ROADMAP.md` (Past gains this entry, Present updated for next-active state, Near future loses the now-shipped XGID Adoption v1 implementation line, header bump), `tasks/XGID_ADOPTION_IMPL.md` (Status: ACTIVE → COMPLETED, Version 1.0 → 1.1), `docs/xgen_ch4_implementation.md` (one-line v1 follow-on pointer per A5 Joe-lock during Phase 2 sweep — Scope-B blockquote shape matching Phase 1 normative pointers in `docs/xgen_aicontrol_implementation.md` and `docs/xgen_appendix_f_en.md`, with header bump riding on top of Phase 8 doc-pass close annotation). **Carry-over to Retrofit Pass 1:** hash-anchored convenience constructors (`EventXgid::from_event`, `SpaceXgid::from_space_create`, `RoomXgid::from_room_create`, `TrustAssertionXgid::from_assertion`) deferred from Commit 1 because their implementation requires `canonical_event_bytes()` which lives in `xgen-core/src/wire/canonical.rs` and is not visible to `xgen-common` — module-level doc comment in `flavours.rs` flags this with full rationale; runbook's "where it is clean to do so" hedge applied; Pass 1 picks up by moving the canonical-form helpers from `xgen-core/src/wire/canonical.rs` to `xgen-common/src/canonical.rs` (with `xgen-core` re-exporting to preserve call sites) as part of the coordinated commit set, then adding the convenience constructors on the same flavour wrappers. Discipline notes (full detail in J-095): Rule 4 in action — milestone-close commit's changed-files list includes JOURNAL.md alongside the cross-doc updates (the candidate sibling principle flagged in J-094 being followed pre-emptively); D-069 canonical-document discipline as sustained pattern (the same coordinated-deliverable shape XGID Adoption v1 used at design close `a5f3c8b` eight artefacts atomically now used at implementation close); B3-shape gap question asked and answered (no gap — XGID is structurally narrower than Phase 7.5, all surfaces exercised by tests written in the same commits).

✅ **XGID Adoption v1 Phase 2 doc-tree sweep closed 2026-05-20** (single session, no test count change — documentation only). Phase 2 walked all 23 in-scope `docs/` files and produced the classification table at `tasks/XGID_DOC_SWEEP.md` (Status: COMPLETED v1.2). **Pre-walk Scope-A-vs-Scope-B Joe-lock: Scope B** — pointers belong in implementation-flavoured docs (consulted tactically without normative context); spec-flavoured docs inherit invariance through Ch3 §3.0 normatively without redundant pointers; design/audit/roadmap/journal/task files take no pointers. Audience analysis that locked Scope B: classification table is read by future Chat Claude sessions and Joe (Clair never reads it directly); pointer text inside docs is read by humans browsing documentation; no code is shaped by pointer presence/absence. Spec-readers are in spec-reading frame and consult Ch3 by structure; tactical readers benefit from pointer signal. Translation rule applied uniformly across six walk groups (A spec chapters, B English appendices, C design docs, D audit/historical, E implementation reference, F project navigation). **Verdict distribution across 23 in-scope docs:** 4 v1-already-shipped (Ch3, Appx F, Appx J, `xgen_aicontrol_implementation.md`); 1 v1-follow-on-pointer (Ch4, A5 Joe-lock during walk picked option (a) over deferring to a Pass); 2 Pass-1 docs (Appx C primitive schemas, Appx I data structures — with coordination flag pinned: Pass 1 code retype + both doc retypes must land in one coordinated commit set to prevent spec drift at Pass 1 close); 1 Pass-3 doc (Appx D storage/privacy field tables); 1 Pass-5 doc (Appx G log line convention); 14 no-update (Ch0, Ch1, Ch2, Ch5, Ch6, Appx A, Appx B, Appx E, Appx H, `xgen_federation_propagation_design.md`, `xgen_node_admin_ops_design.md`, `xgen_lifecycle_states.md`, `xgen_propagation_reliability.md`, ROADMAP.md). **Pass 2 has zero doc work** (xgen-core consumes types defined in xgen-common; surface documented across Appx C + Appx I, both retyping with Pass 1) — flagged for the Pass 2 runbook author. **Pass 4 has zero new doc rows but substantial per-section work** in two already-pointer-tagged docs (Appx F 890 lines + `xgen_aicontrol_implementation.md` 372 lines) — Pass 4's runbook should anticipate it is the heaviest doc-work pass despite no new classification-table rows. **Pre-walk housekeeping:** `docs/xgen_appendix_a_sk.md` and `docs/xgen_appendix_b_sk.md` flipped from ACTIVE to DEPRECATED with future-translation-pass rationale (`Last updated` bumped to 2026-05-20). Both excluded from the classification table per Joe's call. Future Slovak translation pass will retype from completed English docs as a single coordinated effort once English documentation reaches a stable end-state. Observations recorded in the classification table for future Retrofit Pass runbook authors. The Phase 2 deliverable is the classification table; per-doc edits land afterwards under each Retrofit Pass.

### Named decisions promoted to DECISIONS.md

✅ **D-070 promoted 2026-05-18** (468 tests, same-day post-Pass-3, no code changes). "Two events of equal importance, opposite direction" — named protocol principle. The original draft in `docs/xgen_node_admin_ops_design.md` §9 framed the principle as "EventAccepted exists, symmetric to Error." The Propagation Reliability Audit (J-081 §5) found that framing was necessary but not sufficient: `TransportMessage::Error` lacked an `event_id` field at all, meaning even with both Error and a future EventAccepted, the originator couldn't correlate either signal back to a specific event. D-070's DECISIONS.md entry incorporates the corrected post-audit framing: BOTH (1) both directions of outcome exist (acceptance + rejection signals), AND (2) both directions carry the envelope-level correlation identifier (`event_id: Option<String>` on `TransportMessage`). Without (2), (1) is hollow. M6 §9 draft preserved as historical record; DECISIONS.md D-070 is the canonical authoritative form. M6 (new) Phase 2 implements both halves in coordinated work with Federation Event Propagation milestone's F-4 rejection-site changes.

✅ **D-071 promoted 2026-05-18** (468 tests, same-day post-D-070, no code changes). "Subsystem audits precede dependent milestones" — project-management principle. Every future milestone whose correctness depends on a load-bearing subsystem MUST include a subsystem audit as part of its Phase 0 (design phase). Audits produce code-grounded canonical documents and surface gaps that may need to close as preconditions of the milestone rather than as parallel work. The pattern emerged organically during the Propagation Reliability Audit (J-081), where findings consistently exceeded the audit's nominal scope (four HIGH-severity findings across five sections) and the audit became Pass 1 input for two downstream design phases (M6 Phase 0, Federation Event Propagation Phase 0). D-071 pairs with D-069: audit phase (D-071) → design phase (D-069) → implementation phase, each producing a canonical artefact. Sibling to D-065 and D-070 — D-065 and D-070 are protocol-design principles; D-071 is the project-management analogue. The shared theme across all three: don't let assumed-state substitute for verified-state.

### Deprecated tracks

⬛ **M6 (original) Multiparty baseline pass** — descoped 2026-05-17. Original M6 plan (run the full Multiparty suite S1–S5 twice through present `--batch` to fill the "A" baseline column) descoped because the binary state had shifted post-J-079, the metric-protocol applicability needed reconfirmation, and the bigger problem M6 was meant to solve had grown to span both binaries. **Replaced by M9 Multiparty Redesign** at the end of the M-series trunk; the M6 slot is reused for the Node admin write path. Affected task files (`tasks/MULTIPARTY_S1_tauri_rerun.md`, `tasks/MULTIPARTY_S2_to_S5_present_pass.md`) carry the DEPRECATED status with the M9 pointer.

⬛ **`tasks/NODE_ADMIN_PASS2_PROPOSALS.md`** — superseded by `docs/xgen_node_admin_ops_design.md`. The Pass 2 proposals file was the working document for Pass 3's lock-decisions; once the design doc shipped with the framework decisions filled in, the proposals file lost its operational role. Kept in `tasks/` as historical predecessor per D-069's canonical-document rule.

---

## Present — playing now

The track or tracks the project is actively working on right now. Detail-level here is the most granular in the document — what's in flight, what's blocking what, what the next concrete step is.

🟢 **Federation Event Propagation Phase 9 — deployment integration tests RESUMED 2026-05-20.** Phase 9 paused at Commit 3 boundary in J-093 when the cold-start bootstrap dead-lock surfaced (failure-mode catalogue M5); Phase 7.5 closed the gap, then XGID Adoption v1 shipped between J-093 and now (J-095). Phase 9 Commit 3 (baseline deployment scenarios 1–3) is the next active commit, with integration test code now using XGID types from start — `NodeXgid` for federation peer IDs at construction points, `EventXgid` for event hash anchors, etc. — leveraging the v1 type vocabulary the same way any post-milestone consumer would. After Phase 9 ships, Federation Event Propagation milestone flips PLAY → DONE and M6 (new) Node admin write path unblocks for implementation per its Pass 3 design. Task file [tasks/FEDERATION_PROPAGATION_PHASE_9.md](../tasks/FEDERATION_PROPAGATION_PHASE_9.md) remains Status: ACTIVE v1.0; scope intact (12 scenarios — 6 baseline + 6 compounds; 4 deferred compounds in `tasks/FEDERATION_STRESS_FOLLOWON.md` blocked on clock-injection seam). Q3 escalation criterion from J-091 still applies: any new test exhibiting 127.0.0.1:0 bind race or WS frame-ordering inconsistency under workspace parallelism — STOP and walk back to option (ii) on Q3 (investigate underlying race).

🟢 **XGID Retrofit Pass 1 runbook authoring — ACTIVE for Chat Claude 2026-05-20.** Pass 1 (`xgen-common` core types) is the first of five staged retrofit passes per the Q3 Shape γ + ASAP discipline (D-072). Chat Claude authors `tasks/XGID_RETROFIT_PASS_1_IMPL.md` using the classification table at `tasks/XGID_DOC_SWEEP.md` (Status: COMPLETED v1.2) as canonical input. The Pass 1 coordination flag pinned in the classification table is the load-bearing instruction: Pass 1 code retype + Appendix C + Appendix I must land in one coordinated commit set to prevent spec drift at Pass 1 close. The runbook should also incorporate the deferred-convenience-constructors carry-over from XGID Adoption v1 milestone close — move `canonical_event_bytes` from `xgen-core/src/wire/canonical.rs` to `xgen-common/src/canonical.rs` (with `xgen-core` re-exporting to preserve call sites) as part of the Pass 1 commit set, then add the hash-anchored `from_event` / `from_space_create` / `from_room_create` / `from_assertion` convenience constructors on the v1 flavour wrappers as part of the same set (see J-095 "Carry-overs" for the full rationale). After authoring + Joe-lock, Pass 1 implementation is Clair's pickup; no parallel Chat Claude work in scope during implementation.

---

## Near future — designed or scoped, awaiting work

Tracks that are ready to start. Each has known shape, known scope, known dependencies. Listed in roughly the order they will be picked up, though that order is not strictly locked — parallelism is possible between independent tracks.

🟡 **XGID Retrofit Pass 1 — core data structures (spans xgen-common + xgen-core).** First retrofit pass per Shape γ + ASAP discipline (Q3 lock). Scope: retype the XGID-carrying String fields in the foundational protocol data structures, regardless of which crate they live in. This deliberately spans xgen-common (`Event` struct field retypes — `event_id`, `sender`, `room_id`, `space_id`, `prev_events`; `SpaceLocalMetadata.space_id`; the `state.rs` observability structs `NodeState`/`FederatedPeer`/`HostedSpace`/`ConnectedClient`) and xgen-core (`SpaceState` field retypes — `space_id`, `room_id` maps, `federation_nodes` peer IDs, `members` Identity keys, `ai_operator_delegations` Identity pairs, `owner_id`, `home_node`; `FederationRegistry` keys; `IdentityRegistry` keys; `PendingBuffer` map keys). Cross-crate scope is load-bearing: the Phase 2 doc-tree sweep classified Appendix C + Appendix I as Pass 1 deliverables to be shipped in one coordinated commit set, and splitting the data structures across two Passes would split Appx I documentation in a way the canonical-document rule (D-069) prohibits. Plus the canonical-form carry-over from XGID Adoption v1: move `canonical_event_bytes` from `xgen-core/src/wire/canonical.rs` to `xgen-common/src/canonical.rs` (with `xgen-core` re-exporting to preserve call sites), then add the hash-anchored `from_event` / `from_space_create` / `from_room_create` / `from_assertion` convenience constructors on the v1 flavour wrappers. Documentation: Appendix C primitive schemas and Appendix I data structures field tables retyped column-by-column from String to flavour-typed XGIDs. Test fixtures using these structures get updated in this same pass. Pass 1's runbook lives at `tasks/XGID_RETROFIT_PASS_1_IMPL.md`. Sequenced immediately after Phase 9 ships and Federation Event Propagation milestone flips to DONE; interleaves with M6 (new) work rather than waiting for M-series to settle.

🟡 **XGID Retrofit Pass 2 — `xgen-core`.** Second retrofit pass per ASAP discipline. Scope: retype XGID surfaces in `xgen-core` — validation core (`validate_event`, `ValidationOutcome::HeldPending` struct fields), dispatch (`NodeRuntime::dispatch_event`, `DispatchOutcome` variants), pending buffer (`PendingBuffer`'s `missing_predecessors`, `missing_identity`, `missing_federation_relationship` fields and arrival hooks), federation registry types and operational APIs (`mark_active`, `mark_lost`, `peer_records` keys), identity registry methods (`contains`, `get`, `verify_event_signature`'s parameters), `accept_message` signature. Documentation: Appendix I parts covering xgen-core-resident structures retyped per the canonical sources commit's classification.

🟡 **XGID Retrofit Pass 3 — `xgen-node`.** Third retrofit pass. Scope: retype XGID surfaces in `xgen-node` — federation session (`stream_federation_delta`, `handle_federation_incoming`, `compute_federation_delta_for_space`), fanout (`apply_fanout`, `apply_federation_push`, `FederationPeerSenders` keys, `ClientSenders` keys), app (`process_inbound`, `handle_identity_msg`, `handle_identity_replicate_msg`, `handle_incoming_replicate`), reconnect scheduler (`spawn_reconnect_scheduler`, `attempt_reconnect`), pipe server admin verbs (post-M6). Documentation: Appendix F Node-side sections + any Ch4 §4.11 / §4.12 Node-side identifier references retyped per the canonical sources commit's classification.

🟡 **XGID Retrofit Pass 4 — `xgen-client`.** Fourth retrofit pass. Scope: retype XGID surfaces in `xgen-client` — ops layer (every `xgen-client-lib::ops::<verb>` function signature, `ops::*` `Result<<Verb>Result>` types), AI behaviour (`AiBehavior` trait, `EchoPlugin` reference impl, `AiPacingTracker`), session state (`SessionState`, `ClientIdentity`, `ensure_identity`, `ensure_connected`), batch dispatcher (`run_batch_file`, `batch::dispatch_line`), CLI dispatcher (`main.rs` CLI arm), AI service (`ai_service::run`, `__HEALTH__` extension), Tauri commands. Documentation: Appendix F Client-side sections + `docs/xgen_aicontrol_implementation.md` reply schemas + Ch6 §6.15 client-side spec retyped per the canonical sources commit's classification. This pass closes the AI control / batch wire-side of the Q4 invariance promise from documentation-claims-it perspective to documentation-is-it-and-code-matches perspective.

🟡 **XGID Retrofit Pass 5 — test fixtures, helpers, and remaining surfaces.** Fifth and final retrofit pass per ASAP discipline. Scope: retype XGID surfaces in test fixture builders, integration test helpers (`build_node_with_alice_member`, federation-test setup helpers, smoke-test scaffolds), trace event field types, log line formatters, debug/Display impls, and any remaining surfaces surfaced during the earlier passes that didn't naturally fold in. After Pass 5 closes, the codebase has uniform XGID type discipline; the §5.6 principle ("field name carries the role, type carries the contract") is fully realised in code; the Q3 "mixed discipline transitionally" clause no longer applies. ROADMAP.md cross-cutting section gains the wire-format invariance promise as a named principle alongside D-065/D-070/D-071 at Pass 5 close.

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
