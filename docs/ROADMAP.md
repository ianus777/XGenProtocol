# XGen Protocol — Project Roadmap
> **Status**: ACTIVE  
> Version: 6.25  
> Date: May 2026  
> **Last updated**: 2026-07-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this document is

The canonical coarse-grained view of where the XGen Protocol project has been, where it is now, and where it is going. One status per track, one line or one short paragraph per item, written so a reader can answer "where are we" without reading the project's full history.

**This document complements, does not replace.** Detailed progress lives in `JOURNAL.md` (contemporaneous record), settled architectural calls live in `DECISIONS.md` (numbered decisions), session-state operational guidance lives in `CLAUDE.md` (what Claude Code should read on the next session), specifications live in `docs/xgen_ch*.md` and `docs/xgen_appendix_*.md`. ROADMAP.md sits above all of these as the navigation map between them.

**This document mirrors reality, not aspiration.** When a milestone is descoped (M6 multiparty → the M9 multiparty track), it moves rather than disappears. When new work surfaces (Propagation Reliability Audit opened mid-project, Federation Event Propagation milestone added after the audit surfaced a gap), it lands here the moment it's recognised. The roadmap is not a plan-from-the-start that the project is executing; it is a living record of what the project now knows it needs.

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
├── ✅ **Federation Event Propagation** — F-1…F-10 wire + validation · J-119
│   ├── ✅ **Pass 2 design** — 10 F-items locked · J-119
│   ├── ✅ **Pass 3 design** — canonical doc + runbook · J-119
│   ├── ✅ **Phase 1** — F-6 + F-7 wire shape · J-119
│   ├── ✅ **Phase 2** — F-4 validation unification · J-119
│   ├── ✅ **Phase 3** — F-1a tip exchange · J-119
│   ├── ✅ **Phase 4** — F-1 federation push · J-119
│   ├── ✅ **Phase 5** — F-1c per-peer record · J-119
│   ├── ✅ **Phase 6** — F-10 unknown-signer hold · J-119
│   ├── ✅ **Phase 7** — F-3 federation-relationship gate · J-119
│   │   └── ✅ **Phase 7 B3 amendment** — predecessor-chain + step-11 closure · tasks/archive/FEDERATION_PROPAGATION_PHASE_7_B3_AMENDMENT.md
│   ├── ✅ **Phase 7.5** — cold-start bootstrap · J-119
│   │   ├── ✅ **design** — 4 framework decisions · J-093
│   │   └── ✅ **implementation** — 5 commits · J-103
│   ├── ✅ **Phase 9** — deployment integration · J-119
│   │   ├── ✅ **survey** — 14 failure-mode catalogue · J-119
│   │   ├── ✅ **Commit 1** — G1 observability · J-119
│   │   ├── ✅ **Commit 2** — flake fixes · J-119
│   │   ├── ✅ **Commit 3a** — harness + Scenario 1 regression · J-119
│   │   ├── ✅ **Commit 3b-2** — Scenario 2 · J-110
│   │   ├── ✅ **Commit 3b-3-pre** — harness extension · J-111
│   │   ├── ✅ **Commit 3b-3** — Compound C2 · J-112
│   │   ├── ✅ **Commit 3b-4 runbook** · J-114 (J-109, J-113 — never written, see J-603)
│   │   ├── ✅ **Commit 3b-4** — NodeRuntime-level · J-118 (J-113 — never written, see J-603)
│   │   └── ✅ **milestone close** · J-119
│   ├── ✅ **Persistence-amendment sub-milestone** · J-104
│   │   ├── ✅ **audit** · J-105
│   │   ├── ✅ **design** — four Joe-locks · J-107
│   │   ├── ✅ **runbook** · J-106
│   │   ├── ✅ **Track 1 re-walk amendments** · J-107
│   │   ├── ✅ **implementation** — Commit 1 doc-pass `0ca29e6` · J-107
│   │   └── ✅ **milestone close** — Commit 4 · J-108
│   ├── ✅ **Bidirectional federation_nodes** · J-096
│   │   ├── ✅ **audit** · J-096
│   │   ├── ✅ **design** — Q1 Reading (i) + Shape A · J-096
│   │   └── ✅ **implementation** — 4 commits · J-096
│   └── ✅ **Topological-sort wire-order determinism** · J-096
│       ├── ✅ **audit** · J-096
│       ├── ✅ **design** — three locks · J-098 — never written, see J-603
│       ├── ✅ **runbook** · J-100
│       ├── ✅ **implementation** — five-commit sequence · J-101
│       │   ├── ✅ **Step 1** — Joe-lock conversion · J-098 — never written, see J-603
│       │   ├── ✅ **Step 2** — canonical-record amendments + Rule 0 · J-099
│       │   ├── ✅ **Step 3** — runbook v1.0 → v1.1 · J-100
│       │   ├── ✅ **Commit 1** — doc-pass · J-101
│       │   ├── ✅ **Commit 2** — determinism layer `0543a86` · J-101
│       │   ├── ✅ **Commit 2a** — causality layer `4a6fd74`, Path B · J-101
│       │   ├── ✅ **Commit 3** — Phase 9 Scenario 1 second `#[ignore]` · J-101
│       │   └── ✅ **Commit 4** — milestone close per D-074 · J-101
│       └── ✅ **Phase 9 Commit 3b arc** — completed under the close · J-110
│
├── ✅ **XGID Adoption v1** · J-095
│   ├── ✅ **design walkthrough** — Q1–Q6 locked · J-095
│   ├── ✅ **Phase 1 canonical sources** — 8-artefact atomic · J-095
│   ├── ✅ **Phase 2 doc-tree sweep** — classification table · J-095
│   │   ├── ✅ **Scope-A-vs-B pre-walk lock** · J-095
│   │   ├── ✅ **SK appendix housekeeping** · J-095
│   │   └── ✅ **23-doc classification walk** — 6 groups A–F · J-095
│   └── ✅ **implementation** — 2 production commits · J-095
│       ├── ✅ **Commit 1** `c95584a` — xgen-common XGID types · J-095
│       ├── ✅ **Commit 2** `24a255b` — SpaceLocalMetadata · J-095
│       ├── ✅ **hygiene** `904441b` — workspace clippy · J-095
│       └── ✅ **milestone close** · J-095
│
├── ✅ **XGID Retrofit Pass series** — all 5 passes closed · J-122 → J-148
│   ├── ✅ **Pass 1** — core data structures · J-122
│   │   ├── ✅ **Commit 1** `403ef3f` — canonical-form module · J-122
│   │   ├── ✅ **Commit 2** `8a94dee` — convenience constructors · J-122
│   │   ├── ✅ **Commit 3** `75e81b4` — xgen-common data-structures · J-122
│   │   ├── ✅ **Commit 4** `774fe9d` — xgen-core data-structures · J-122
│   │   ├── ✅ **Commit 4a** `4895446` — xgen-core test-fixtures · J-122
│   │   ├── ✅ **Commit 5** `096162e` — Appendix C + I · J-122
│   │   ├── ✅ **J-121 hygiene atom** `1dd909e` · J-121
│   │   └── ✅ **Commit 6** — milestone close · J-122
│   ├── ✅ **Pass 2** — xgen-core, code-only · J-126
│   │   ├── ✅ **design** — single principle locked · J-123
│   │   ├── ✅ **runbook** · J-124
│   │   ├── ✅ **Commit 1** — doc-pass `5892e9e` · J-125
│   │   ├── ✅ **Commit 2** `22765a0` — five surfaces atomic · J-126
│   │   ├── ✅ **Commit 2a** `58b94a5` — test-fixture projection sweep · J-126
│   │   ├── ✅ **Commit 3** — milestone close · J-126
│   │   ├── ✅ **(code) validate_event / ValidationOutcome** · J-126
│   │   ├── ✅ **(code) NodeRuntime::dispatch_event / DispatchOutcome** · J-126
│   │   ├── ✅ **(code) PendingBuffer arrival hooks** · J-126
│   │   ├── ✅ **(code) FederationRegistry / IdentityRegistry** · J-126
│   │   └── ✅ **(code) accept_message signature** · J-126
│   ├── ✅ **Pass 3** — xgen-node + Appendix D · J-138
│   │   ├── ✅ **design** · J-127
│   │   ├── ✅ **runbook** · J-128 (J-129)
│   │   ├── ✅ **J-130 drift-fix atom** · J-130
│   │   ├── ✅ **Commit 1** — doc-pass · J-131
│   │   ├── ✅ **J-132 Path-(iii) amend-in-place** · J-132
│   │   ├── ✅ **J-133 design §2 v1.2 → v1.3** · J-133
│   │   ├── ✅ **J-134 design §2 v1.3 → v1.4** · J-134
│   │   ├── ✅ **J-135 runbook v1.2 → v1.3** · J-135
│   │   ├── ✅ **Commit 2** `67fb48d` · J-136
│   │   ├── ✅ **Commit 2a** `0cdf0ad` · J-137
│   │   ├── ✅ **milestone close** · J-138
│   │   ├── ✅ **(code) federation_session.rs** · J-138
│   │   ├── ✅ **(code) fanout.rs Sender maps** · J-138
│   │   ├── ✅ **(code) app.rs handler slots** · J-138
│   │   ├── ✅ **(code) reconnect.rs spawned fns** · J-138
│   │   ├── ✅ **(code) NodeRuntime per-space HashMap keys** · J-138
│   │   ├── ✅ **(doc) Appendix D classifications** · J-138
│   │   └── ✅ **per-surface tests T1–T11 + fixture sweep** · J-137
│   ├── ✅ **Pass 4** · J-146
│   │   ├── ✅ **design** — v1.0 → v1.2 · J-140
│   │   ├── ✅ **runbook** · J-141
│   │   ├── ✅ **Commit 1** `3869d4c` · J-145 (J-142, J-143)
│   │   ├── ✅ **Surface #2 CLI dispatcher** — app.rs · J-145
│   │   ├── ✅ **Surface #3 batch pipe dispatch** — batch.rs · J-145
│   │   ├── ✅ **Surface #4 Tauri shell** — desktop.rs · J-145
│   │   ├── ✅ **Surface #5 session state** — session.rs · J-145
│   │   ├── ✅ **Surface #6 AI resident** — ai_service.rs · J-145
│   │   ├── ✅ **Surface #7 pacing + temperature** · J-145
│   │   ├── ✅ **(doc) Appendix F client-side** · J-145
│   │   ├── ✅ **(doc) xgen_aicontrol_implementation.md** · J-145
│   │   └── ✅ **(doc) Ch6 §6.15 client spec** · J-145
│   └── ✅ **Pass 5** — confirm-clean, 1 trace fix · J-148
│       ├── ✅ **(code) test fixture builders** · J-148
│       ├── ✅ **(code) integration test helpers** · J-148
│       ├── ✅ **(code) trace event field types** · J-148
│       ├── ✅ **(code) log line formatters** · J-148
│       ├── ✅ **(code) debug/Display impls** · J-148
│       ├── ✅ **(doc) Appendix G log line convention** · J-148
│       └── ✅ **wire-format invariance promise** — promoted at close · J-148
│
├── 🟡 **M-series trunk**
│   ├── ✅ **M1** — binary consolidation · J-073
│   ├── ✅ **M2** — node pipe server · J-074
│   ├── ✅ **M3** — AI operator role · J-075
│   ├── ✅ **M4** — AI client binary · J-077
│   ├── ✅ **M5** — ops::* refactor · J-078
│   ├── ⬛ **M6 (original)** — multiparty baseline, descoped 2026-05-17 · DECISIONS.md D-069 → M9
│   ├── ✅ **M6 (new)** — node admin write path · J-197
│   │   ├── ✅ **Phase 0 design** — 3 passes + Block 4, 33 verbs · J-197
│   │   ├── ✅ **Phase 1** — R1 `rooms` · J-152
│   │   │   └── 🟡 **`members` verb** — no local data source at Phase 1
│   │   │       ↳ trigger: a local members data source exists
│   │   ├── ✅ **Phase 2** — admin_ops/audit skeletons + EventAccepted wire · J-153 (J-081)
│   │   ├── ⬛ **Phase 3** — collapsed; reads bucketed into category phases · J-153
│   │   ├── ✅ **Phase 4** — A6 logging & audit, 5 verbs · J-154
│   │   ├── ✅ **Phase 5** — A5 identity registry, 4 verbs · J-155
│   │   ├── ✅ **Phase 6** — A3 bootstrap config · J-195 (J-280)
│   │   ├── ✅ **Phase 7** — A1 federation mgmt: `list` + `defederate` · J-156
│   │   │   └── ✅ **remaining 5 verbs** — routed to the federation-admin-control arc · J-178
│   │   ├── ✅ **backing audit** · J-157
│   │   ├── ✅ **Phase 9 read subset** — A4 `space list-hosted` · J-157
│   │   │   ├── ✅ **`audit-events`** — routed to the protocol-audit-log arc · J-169 (J-166, J-167, J-168, J-170)
│   │   │   └── ✅ **force-eject A4-D1** — landed at Option B · J-160
│   │   ├── ✅ **Phase 10** — A7 plugin `list` + `status` · J-158
│   │   ├── ✅ **Phase 9** — A4 force-eject + node-unban · J-159
│   │   ├── ✅ **A4 force-eject Option B** — live fan-out · J-160
│   │   ├── ✅ **D-071 arc audit** · J-161
│   │   ├── ✅ **D-071 arc design stubs** — 4 scaffolds opened · J-162
│   │   ├── ✅ **M6 held doc work cleared** · J-163
│   │   ├── ✅ **protocol-audit-log arc** · J-169
│   │   ├── ✅ **federation-admin-control 2a** · J-178
│   │   ├── ✅ **federation-admin-control 2b** — policy · J-183
│   │   ├── ✅ **auth-module-registry (A2)** · J-185 (J-189)
│   │   ├── ✅ **bootstrap-client (A3)** · J-190
│   │   └── ✅ **node-policy** — 5th/final D-071 deferral · J-196
│   ├── ✅ **M7 family** · J-205 → J-226
│   │   ├── ✅ **M7 --aicontrol v1** — command pipes · J-205 (J-201, J-202, J-204)
│   │   ├── ✅ **M7-events arc** — `.events` observation pipes · J-212 (J-203, J-207, J-208, J-209, J-210, J-211, J-229)
│   │   ├── ✅ **M7-completion cluster** · J-223 (J-217, J-218, J-219, J-220, J-221, J-222)
│   │   │   ├── 🟡 **plugin-write verbs** ↳ trigger: a second plugin exists
│   │   │   ├── 🟡 **pipelined handler / CONCURRENT_COMMAND_NOT_ALLOWED** ↳ trigger: none — filed, not scheduled
│   │   │   ├── 🟡 **`migrate-start`** ↳ trigger: the migration subsystem opens
│   │   │   ├── 🟡 **per-driver-identity control plane** ↳ trigger: the privilege-model arc opens
│   │   │   └── ✅ **live config reload** — routed to M7-standalone · J-226
│   │   └── ✅ **M7-standalone** — live config reload · J-226
│   ├── ✅ **M8** — state-resolution convergence · J-241
│   ├── ✅ **Durable EventStore** — trait + vanilla impl · J-228
│   ├── ✅ **Storage-Engine / Plugin-Framework** — C1–C5 + S shipped · J-232
│   ├── 🟡 **Protocol gap-closure arcs** — PG register
│   │   ├── ✅ **Arc A** — doc-drift · J-233
│   │   ├── ✅ **Arc B** — forward-compat / PG-09 · J-235
│   │   ├── ✅ **Arc C** — borrowed the M8 slot · J-241
│   │   ├── ✅ **Arc D** — privilege model / PG-13 + PG-12-min · J-244
│   │   ├── ✅ **Arc E** — primitive completion / PG-03 + PG-08 · J-248
│   │   ├── ✅ **Arc F** — space migration / PG-11 · J-252
│   │   ├── ✅ **Arc G** — jurisdictional namespacing / PG-04 · J-250
│   │   ├── 🟡 **Arc H** — E2E encryption / PG-05
│   │   │   ├── ✅ **design + interface lock** · J-257
│   │   │   └── 🟡 **PG-05 implementation** — real MLS on the live event path
│   │   │       ↳ trigger: D3 — RFC 9420 / openmls in the build
│   │   └── 🟡 **Arc I** — GDPR erasure / PG-02
│   │       ├── ✅ **design + D-088** · J-253
│   │       ├── 🟡 **content erasure** — crypto-shred ↳ trigger: PG-05 implementation ships
│   │       └── 🟡 **identity orphaning** ↳ trigger: none — PG-05-independent; rides the Tier-1 auth-module rebuild
│   ├── ✅ **M8.5** — finalization: INV + F-5 + S5 rebind · J-279
│   ├── ✅ **M8.6** — federation stress, clock-injection · J-294
│   │   └── ✅ **INV-EXP** — invite-expiry replay-gate fix · J-298 (J-296)
│   ├── ✅ **M8.7** — concurrent-commit resolution, R · J-302 (J-299, J-300, J-301)
│   │   └── 🟡 **S + home-DS serialization + loser-rebuild** — folded into the production openmls-client arc
│   │       ↳ trigger: D3 — RFC 9420 / openmls in the build
│   ├── ✅ **M9** — strategic multiparty test harness · J-307
│   │   ├── ✅ **M9.1** — event timestamp-bound validation · J-311
│   │   └── ✅ **M9.2** — harness-enablement seams · J-315
│   ├── ✅ **Multiparty tests** — R1 + R2 + R3 · J-356 · ledger at tasks/HANDOFF_MP_R3.md §3
│   │   └── 🟡 **MP-C-06 + MP-C-16** — re-home capability gap ↳ trigger: M10 lands
│   ├── ✅ **M10** — auth module reference set, Tier-1 · J-375
│   ├── ✅ **M11** — `self` thread, D-021 · J-378
│   └── ✅ **M12** — attachments · J-379 (J-380, J-381, J-382, J-384, J-385, J-386, J-387, J-388, J-389)
│
├── ✅ **Pre-UI documentation-optimization phase** · J-396
├── ✅ **Appendix F/I audit-against-code** · J-398 (J-397)
│   └── ✅ **AI-F17** — IdentityMessage::Record gains is_ai · J-400 (J-401)
├── ✅ **Round 2** — final pre-UI whole-codebase gate, GO · J-390
│   └── 🟡 **gap register** — Open 1/13 (PG-02) ↳ trigger: Arc I closes
│
├── 🟡 **UI** — clean-table rebuild, the post-M10 endpoint · J-390
│   ├── ⬛ **UI Phase 2 visual merge** — element-modelling approach abandoned · J-284 → clean-table UI milestone
│   ├── ⬛ **Mockup stock-take + reconcile-to-as-built** — superseded by the component-library-first build · J-284
│   ├── 🟢 **UI component-library / substrate** — RP track, D-095 `ui/{client,node,common,core,assets}` · J-403
│   │   ├── ✅ **M-RP2 di atomics** — 2.3–2.21, 2.30–2.31a; 27 `core` components · J-403 (J-405, J-407, J-410, J-412, J-413, J-414, J-415, J-416, J-417, J-418, J-419, J-420, J-425, J-426, J-427, J-430, J-431, J-432, J-457, J-458, J-459, J-460)
│   │   ├── ✅ **M-RP2 di composites** — 2.22, 2.24–2.29; owned-popup pattern founded · J-434 (J-447, J-448, J-449, J-450, J-451, J-452)
│   │   ├── ✅ **M-RP3 sampler** — `xgen-sampler` test-bed, tabbed by class×arity, static header · J-422 (J-423, J-433, J-466)
│   │   ├── ✅ **M-RP4 processors + widget tier** — 4 processor kinds, D-099/D-102, first two widgets · J-435 (J-441, J-443, J-444, J-445, J-453, J-454, J-455, J-456)
│   │   └── ✅ **M-RP5 dd track** — `entity-avatar` → `entity-panel` → `entity-context-menu` · J-462 (J-463, J-464, J-465, J-467, J-468, J-469)
│   ├── ⏸️ **M-RP-LIVEFEED-REFRESH** — the live event router behind the members and rooms panels · J-598
│   │   ↳ ⏸️ POSTPONED J-620 — Joe ruled the sequencing: **finish M-DOC-ROADTREE first**. Leg A's runbook is ACTIVE and ready; Clair is stood down
│   │   ↳ trigger: M-DOC-ROADTREE closes (Leg G), **or** Joe stands Clair up for Leg A
│   │   ├── ✅ **Leg 0 Phase-0** — routing shape + delta-vs-fill boundary locked; second-reader pass over §6 vs `wire.rs` done, three findings · J-616 (J-598, J-601)
│   │   ├── 🟡 **Leg A router + members consumer** — runbook `tasks/RUNBOOK_LIVEFEED_LEG_A.md` v1.2 **ACTIVE**; **three** files, frontend only · J-618
│   │   │   ↳ ready to open — §5 closed by Chat (it was never Joe's); waiting only on Clair being stood up
│   │   ├── 🟡 **Leg B spaces/rooms consumer**
│   │   │   ↳ trigger: §6a cleared — 9 of 14 `state.*` wire strings still have no row
│   │   ├── 🟡 **Leg C reconnect rule** ↳ trigger: §5 of the milestone ruled (Joe)
│   │   ├── 🟡 **Leg D live verify** — two identities, one observer; `membership.kick` added at v1.11
│   │   └── 🟡 **Leg E records + close**
│   ├── 🟡 **Clean-table UI milestone** — the live UI build
│   │   ↳ trigger: Round-2 audit GO + M10 closed *(transcribed from the UI container)*
│   └── 🟡 **Multi-device arc** — R2-F09 ↳ trigger: the UI prototype exercises device add/remove
│
├── 🟢 **M-DOC-ROADTREE** — the canonical records become state boards · J-598
│   ├── ✅ **Leg 0 Phase-0** — scope ruled BOTH, node format ruled · J-598 (J-600)
│   ├── ✅ **Leg A pause + archive** — M-RP-MEMBERS Leg C paused; `ROADMAP_ARCHIVE_2026-07-26.md` taken · J-598
│   ├── 🟡 **Leg B precondition**
│   │   ├── ✅ **P1 unlinked DONE markers** — 94 → 5, all five resolve · J-599
│   │   └── 🟡 **P2 unresolved refs** — RE-MEASURED J-625, **NOT cleared**: all 11 still have no journal entry
│   │       ↳ ⚠️ Leg C removed `J-029`/`J-067` from this file — **their only trace is `ROADMAP_ARCHIVE_2026-07-26.md`** ⇒ its retention is LOAD-BEARING and Leg G's delete-condition is correctly unmet
│   │       ↳ trigger: **Leg B-ter** runs (six bodies migrated · four recorded as never allocated · `J-029` disposed)
│   ├── ✅ **Leg B-bis journal repair** — executed, verified, CLOSED · J-626 (J-625)
│   │   ↳ ✅ census **358/351 → 353/353 EQUAL** · `J-317`–`J-321` one copy each deleted (**−25,614 B measured** — the runbook's `25,127 B` was a CHARACTER count) · survivors byte-identical to `6863702`
│   │   ↳ ✅ `J-044`→`J-044a`/`J-044b` · `J-045`→`J-045a`/`J-045b` — 🛑 settled by ARTEFACT EVIDENCE, **not** by `D-134` §2's sort-direction premise, which is FALSE for this file (newest-first only to ~L14898, then an original oldest-first block `J-001`→`J-047`)
│   │   ↳ ✅ 11 citations re-pointed individually (`ROADMAP_ARCHIVE:348` held under §4's lock) · `Repaired:` line written · header corrected — span low end read `J-046`, archive reaches `J-001`
│   │   ↳ 📌 its two unmet DoD items were **re-homed to Leg B-ter**, not dropped; `D-133` forbids a leg runbook carrying an `Owes:` line
│   ├── ✅ **Leg B-ter the eleven that resolve nowhere** — **CLOSED AS DISCHARGED J-635** · spawned + titled by Joe J-626
│   │   ↳ 🛑 **THE `24 MISSING` CENSUS IS WRONG AT THE ROOT (J-635)** — it enumerated `## Entry J-nnn` and could not see the **16** old-form `## J-nnn — date — title` headings in `JOURNAL_ARCHIVE.md` (against 353 Entry-form). ✅ **All thirteen of set A `J-021`–`J-033` EXIST and were never missing**; ✅ **`J-029`'s open disposition dissolves — it has an entry**; ⚠️ **`J-171` is the ONE number confirmed absent.** **Eighth instance of this arc's defect class, and it was the premise of a whole leg**
│   │   ↳ ⚠️ **J-634 cited J-603 by LINE NUMBER and every citation went stale by +26 the instant J-634 was prepended to that same file** ⇒ **a line citation into a file you are about to prepend to is self-invalidating; corrected to content identification (J-635)**
│   │   ↳ 🔒 **disposition (Joe, J-635): closed as discharged, successor spawned.** Reduce-to-`J-029` had dissolved; re-scoping onto the twelve was refused — **they are not *the eleven*, and one title over two unlike things is the fault the title rule prevents**
│   │   ↳ scope: **six bodies to migrate** `J-135 J-132 J-131 J-125 J-124 J-123` · **four cited-only with no record anywhere** `J-113 J-109 J-098` (45 refs, no record of its own) `J-067` · **`J-029` disposition open**
│   │   ↳ 🛑 **sole source is `ROADMAP_ARCHIVE_2026-07-26.md`** — Leg C DELETED all six bodies from this file (refs 7→2 · 5→2 · 10→1 · 27→1 · 24→1 · 15→1); §4a required they NOT be deleted until the journal carried them
│   │   ↳ 🛑 **SCOPE ALREADY DISCHARGED AT J-603 (2026-07-28) — TWO DAYS BEFORE THIS LEG WAS SPAWNED AT J-626 (measured J-634).** `JOURNAL.md` carries `## Entry J-603 — six ROADMAP-only records recovered into the journal, and four numbers recorded as never written`, written under **Leg C**. ✅ **Class ①: the six sit under it as `### J-nnn — recovered body` and are SHA256-IDENTICAL to their archive originals** (6,670 · 4,678 · 6,664 · 4,905 · 1,064 · 1,518). ✅ **Class ②: the four are recorded under `### Cited but never written` as never-written rather than reconstructed.** ⚠️ **J-603 also carries a SECOND recovery pass — twelve `Past` entries with no journal record — which this scope statement never mentions and whose status is UNVERIFIED.** 📌 **Identified BY CONTENT: the line numbers this row carried (L842 · L856–878 · L882) went stale by +26 at J-634 and were corrected J-635**
│   │   ↳ 🔑 **found only because `D-135` §1① is locked** — the first predicate `^##\s*(Entry\s*)?J-nnn` could not see `### J-nnn` and returned a confident false negative; the widened re-run found all six in one turn. **Seventh instance of this arc's defect class**
│   │   ↳ 🔓 **DISPOSITION IS JOE'S, NOT TAKEN:** close B-ter as discharged · reduce to `J-029` · or re-scope onto the twelve
│   │   ↳ trigger: none — runnable once runbooked. **It is AUTHORING, not repair**
│   │   ↳ clears: **Leg B P2**
│   ├── ✅ **Leg C `docs/ROADMAP.md`** — 761,422 → 43,741 B; tree kept, prose deleted, five format rules · J-604 (J-602, J-603)
│   ├── ✅ **Leg D `CLAUDE.md` B2** — 65 of 81 blocks archived, 640,645 → 316,680 B (50.6%); D-094 re-applied after a five-week lapse · J-615 (J-606, J-608–J-614)
│   ├── ✅ **Leg E the two-way closure log** — `CLAUDE.md`'s prose head; one closure log in THREE head notations · **CLOSED J-633**
│   │   ↳ ✅ **P3 EXECUTED AND VERIFIED (J-633) — THE LINE IS OUT.** `CLAUDE.md` **382,712 → 256,427 B** (−126,285) · **lines over 20,000: 1 → 0** · `CLAUDE_HISTORY.md` **1,197,988 → 1,325,400 B**, one new `## ` batch at L17 (Leg D pushed to L21, newest-first held) · **payload occurs exactly once and its SHA256 `0af97c2a…` MATCHES the pre-move hash — byte-identity PROVEN** · line endings preserved per file · **both deltas reconcile from independently measured components** · `CLAUDE.md` keeps a 324 B D-094 forward pointer
│   │   ↳ 🛑 **§5 CONTRADICTED F1c AND COULD NOT BE RUN AS WRITTEN** — F1c is *blocks are byte-identical, this heading is the only thing written*, L13 permits *a batch at the top and nothing else edited*, yet §5 mandated annotation at the site **and** §4r's merges. 🔒 **Joe ruled OPTION ①, byte-identical move** ⇒ nothing merged, dropped or annotated in place. ⚠️ **Stated consequence: P2's sixteen verdicts are documentation, not transformation; the archive keeps 16 near-duplicate pairs and dedup was NOT spawned as a leg**
│   │   ↳ 🛑 **§11's SURFACE WORDING WAS BROADER THAN ANYTHING LEG E MEASURED** — it would have swept in four sibling prose lines **and the two live `♻️ Recovery anchor` pointers**; every measurement was inside the 124,299-char line alone ⇒ **P3 moved that line and nothing else**
│   │   ↳ 📘 **RUNBOOKED** `tasks/RUNBOOK_ROADTREE_LEGE.md` **v1.7 ✅ COMPLETED** — the authority throughout. Trigger FIRED J-615; title locked Joe J-618
│   │   ↳ ✅ **P0-BIS · P1 · P2 ALL CLEARED (J-630, 2026-07-31)** — P0-bis closed the direction `D-135` still owed with a predicate sharing no machinery with the head tests: **49 closure marks line-wide, 49 spans holding exactly one each, ZERO with two**; longest zero-mark spans read clean. P1 re-ran green at open **and** close, and **4(d) fired exactly once — on @116,587, the hand verdict already recorded**
│   │   ↳ 🛑 **P2: ALL 16 PAIRS RE-READ ON BOTH SIDES — TWO INHERITED VERDICTS WERE WRONG** ⇒ **11 B ⊃ C · 3 C-side · 2 DIVERGENT**, superseding *8 · 2 · 1*. **J-456 `M-RP4.5` was recorded `B ⊃ C` and is DIVERGENT** — C alone names `transform.ts` and `toEditable`, so *keep B* would have deleted the only place they appear. **J-448 refined: C richer, but B alone carries `CDP-verified`**
│   │   ↳ ✅ all **four** of §4b's named risks confirmed by reading — J-457's hex `#ba7517`, J-459's deferred filter/search rationale, J-460's `box-sizing` evidence all **B-only**; J-454's `temperature-indicator` dd-block and the `ui/docs/`-at-open rule **C-only**. 🔑 **P3 CONSEQUENCE: nothing is a straight delete, and no pair may be resolved by length**
│   │   ↳ 🛑 **J-629 — THE CORRECTIONS THEMSELVES WERE NARROWER THAN THE CLAIMS THEY CORRECTED.** J-627 fixed the runbook and §11 but not §8-E's census; J-628 fixed §1a/§1b/§3.4 but not §5/§6 — `J-627` and `J-628` appeared **exactly once** in the whole task doc. Superseded figures **annotated, never repaired**; **dated `CLAUDE.md` PLAY blocks and `JOURNAL.md` entries are exempt and must NOT be back-edited** ⇒ **a correction is not applied until the whole RECORD SET has been searched — not the section, not the file**
│   │   ↳ ✅ P0 census + P1 assertion pass — heads **A 1+19 · B 51 (4 stubs) · C 25 + 1 emoji-form** = **97 heads, 93 records** (📌 superseding *95 / 91*, J-627); boundaries RE-DERIVED FROM HEAD POSITIONS: A 0–18,426 · B 18,426–**110,488** · C –124,299, sum **124,299**
│   │   ↳ 🛑 **P1 STOPPED ON ITS OWN ASSERTION (J-627)** — §1a had combined C's head count from the OLD boundary 110,516 with the re-derived span 110,488; and a **FOURTH head form** `**<emoji> M-<id>` at 113,632 was invisible to `**M-`
│   │   ↳ ✅ **P1 CLEARED ON RE-RUN (J-628)** — every §3 gate passed, nothing stopped: 1 line · 124,299 · **97 heads** · **93 records** · B **47 + 4 stubs** · C **14 closure-bearing / 12 not** (11 `CLOSED` + 3 `DONE`: J-473 · J-483 · J-485) · A cross-checked by 20 `CLOSED (J-nnn)` marks
│   │   ↳ 🛑 **THAT CLEARANCE IS RETRACTED (J-629) — THREE OF THE 97 WERE NOT HEADS.** `**M-` matched bold **emphasis**: @115,900 and @115,973 inside ONE clause of the record at 115,464, and @119,871 after an **em-dash inside J-485's own sentence** ⇒ **A 20 · B 51 · C 23 = 94 heads · 90 records**, superseding *97 / 93* and *95 / 91*, annotated never erased
│   │   ↳ 🔑 **EVERY §3 GATE PASSED WHILE THAT WAS TRUE** — §3.3 compares the predicate's count to a census figure **produced by the same predicate**, so it detects drift and never error; same disease as §3.4's span-sum, **one line above it** ⇒ 🔒 **`D-135` MINTED (delegated, reversible): test a predicate in BOTH directions; an assertion built from the predicate it checks cannot fail**
│   │   ↳ ✅ **`D-135` LOCKED BY JOE 2026-07-31 (J-631) — no longer reversible.** 📌 It stood **delegated for one day** and **found two wrong inherited verdicts while still provisional**; the *delegated* wording above is its 2026-07-30 state, kept not erased
│   │   ↳ 🛑 **P3 WAS BLOCKED BY THE PREVIOUS SWEEP'S OWN CORRECTIONS (J-632).** Seven live claims still read `97 / 93` — including **§6's Definition of Done, the checklist Leg E is measured by**. Every one had been written by the J-629 sweep as *CORRECTED*, then refuted four hours later by J-630's `94 / 90`, and nothing re-swept the annotations ⇒ **`D-135` §5a DRAFTED (delegated, reversible): sweep for the SUPERSEDED figure AND the REPLACEMENT — an annotation asserting a corrected value decays exactly like the claim it corrects**
│   │   ↳ 🛑 **A LINE CAN BE ANNOTATED AND STILL BE FALSE** — §5 L218 carried a correction about the `13 → 12` move while the live figure needed `12 → 9`. 🔑 **§6 has now been wrong at THREE consecutive passes (J-629 found `24`, J-632 found `97`): it sits past the working sections and is never the section in hand** ⇒ **sweep a figure change mechanically, never from memory**
│   │   ↳ ✅ **REPAIRED, ALL ANNOTATED, NONE ERASED** — §1a row C (**22 genuine + 1 emoji = 23**; span 13,811 unaffected) · §5 (**12 → 9**; closure-bearing **14 never moved**, the J-623 decision untouched) · §6 DoD (**94 · 90**) · `M_DOC` §8-E × 3. **P3 is UNBLOCKED**
│   │   ↳ ✅ **THE SECOND PREDICATE IS THE WHOLE DIFFERENCE** — A had one (20 `CLOSED` marks vs 1+19) → 0 false heads · B had one (J-keys **strictly monotonic J-503→J-445**, which also forbids within-B twins) → 0 false heads · **C had none → all three**
│   │   ↳ 🛑 **§4c COMPLETE — THE ELEVEN COLLISIONS ARE SIXTEEN.** J-478 · J-479 · J-480 · J-483 · J-485 added; the eleven were measured inside part two's two C sub-windows (110,516–111,445 and 122,026–123,912 = **exactly 11**) and **the 10,581 chars between them were never examined** — all five sit in that gap. **Zero collisions within-A, within-B, within-C, A↔B, A↔C**
│   │   ↳ 🛑 **THE PARSE ERROR HID A REVERSED VERDICT, NOT JUST A MISCOUNT** — C's J-485 read **93 chars** (a stub ⇒ *keep B*) while the false head stood; whole it is **1,813** and **richer than B's 906**, so the verdict **inverts**
│   │   ↳ 🔑 **the last narrow predicate widened and its hits read individually** — `**Next-active` × 8 = 1 head (@0) + 7 mid-record; B's `52 / 50` reconciles to **51** (50 delimiters + the seam head @18,426; @85,891 is a backward pointer) ⇒ §1c is now MEASURED, not inherited
│   │   ↳ 🛑 **two runbook defects found by RUNNING it, both ANNOTATED not silently repaired** — ① §3.4's sum-assertion **cannot fail** (consecutive differences always sum to the span) = *the shape of verification*, sharpened to first-0 · zero-duplicates · strictly-ascending ② §1b's stub lengths measure PAYLOAD, P1 measures HEAD-TO-HEAD incl. the 4-char `) / ` delimiter — a missing convention, not a discrepancy
│   │   ↳ 🛑 **NO UNIFORM RECORD HEAD EXISTS** — each region needs its own predicate AND a second independent one to check it; **five successive passes each came from trusting one**, and §3's assert-and-STOP is what finally caught the fifth
│   │   ↳ 🛑 `82` retired (J-622); **16** B↔C collisions (📌 **superseding `11`, J-629**) read pair by pair — the original eleven **8 B ⊃ C · 2 C ⊃ B · 1 divergent**, the five added **4 B ⊃ C · 1 C richer** ⇒ NO mechanical merge; **deletes nothing without a per-pair hand verdict**. 📌 **THAT SPLIT IS ITSELF SUPERSEDED BY `11 · 3 · 2` (J-630) — kept, not erased: the eleven were re-read and J-456 moved from `B ⊃ C` to DIVERGENT, J-448 was refined**
│   │   ↳ ⚠️ identify the line BY CONTENT, not position — it moved L29 → L25 at J-619. ⚠️ **key extraction is UNSOUND for C's 4 forward-looking heads** (first `(J-nnn)` there is a citation, not an identity) — no false collision resulted, all 16 read on both sides, **but not to be reused blind**. ⚠️ **one hand verdict flagged as a judgement: @116,587 opens after a semicolon, admitted on head form**. 🟡 **Next: P3 — the move into `CLAUDE_HISTORY.md`, under §5 and the §4r verdicts**
│   ├── 🟡 **Leg F bidirectional sweep** — is every known work item ON the roadmap, not just is every entry true · J-617
│   ├── ✅ **Leg H what the census could not see** — **TITLED + CLOSED J-636** · spawned J-635 as B-ter's successor · ran before Leg G as required
│   │   ↳ ① **re-run the J-001–J-598 census under `D-135` §1①** — every figure from the old one is suspect ② **resolve `J-171`** — no `#`-heading in either journal; a non-heading form was never tested ③ **verify J-603's second recovery pass, the twelve `Past` entries with no journal record**
│   │   ↳ ✅ **ALL THREE RESOLVED BY MEASUREMENT (J-636); NOTHING AUTHORED**
│   │   ↳ ✅ **① THE CENSUS IS 5, NOT 24.** Three declaring heading forms — `###` 6 · `##` 16 · `## Entry` 613 ⇒ 632 designations. Missing J-001–J-598: **`J-067` · `J-098` · `J-109` · `J-113` · `J-171`**. **False-positive direction tested too** (`D-135` §1②): 19 headings name a number without declaring one, all read, none an entry head ⇒ **set A `J-021`–`J-033` was never a gap, and `J-029` dissolves with it**
│   │   ↳ ✅ **② `J-171` — cited × 9, never written, substance NOT lost.** All nine name it as the session that locked **FAC-D1 · FAC-D1a · FAC-D2** on 2026-05-30; no body anywhere, and its neighbours J-173 / J-174→J-178 exist ⇒ **recorded as cited-but-never-written with a POINTER to `tasks/archive/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md`. NOT reconstructed** — the J-603 treatment
│   │   ↳ ✅ **③ the twelve were never an open task** — already in `JOURNAL.md` verbatim since J-603; `Past — settled` = 103 entries, **91 larger journal entries + 12 recovered = 103**
│   │   ↳ ⚠️ **does NOT claim the archive is safe** — only `Past — settled` was covered; **the other ~470 KB was never audited. That is LEG G's own DoD item**
│   └── 🟡 **Leg G records + close** — delete `ROADMAP_ARCHIVE_2026-07-26.md` iff Leg B cleared · J-617
│       ↳ 🛑 **ORDER-CONSTRAINED: MUST NOT RUN BEFORE LEG B-TER** — that archive is the **sole surviving source** for the six set-B bodies; had Leg G run first they would be gone permanently
│       ↳ 🛑 **THAT JUSTIFICATION IS FALSE (J-634) AND THE CONSTRAINT IS STILL LEFT STANDING.** ✅ The six have been in `JOURNAL.md` **byte-identical since J-603**, SHA256-proven — **Leg G would destroy nothing of the six.** ⚠️ **NOT LIFTED:** lifting permits an irreversible archive deletion and J-603's **twelve-entry second pass is UNVERIFIED** ⇒ 🔓 **Joe's: re-point the constraint from the six to the twelve.** 📌 **A safety constraint whose reason is wrong is not thereby unnecessary**
│       ↳ 🛑 **RE-POINTED, NOT LIFTED (J-635): LEG G MUST NOT RUN BEFORE LEG H.** The six-bodies justification is false and stays annotated; **the constraint survives on a live reason — the twelve are unverified**
│       ↳ ✅ **LEG H RAN AND CLEARED (J-636) — the twelve reason is discharged too.** 🛑 **STILL BLOCKED, on a third and narrower ground: only `Past — settled` was audited; the remaining ~470 KB of that archive never was, and Leg G deletes the whole file** ⇒ **the audit is Leg G's own DoD item.** 🔑 **Three justifications offered, two wrong — the constraint itself right every time**
│       ↳ ⚠️ F and G were `Leg E`/`Leg F` in the task doc and had NO node here until J-617; `E` collided
│
├── ⏸️ **Parallel workstreams**
│   └── ⏸️ **Slovak translation pass** — first touchpoint `xgen_appendix_a_sk.md`
│       ↳ trigger: English docs reach a stable end-state, or need arises
│
├── ⏸️ **Open areas** — deferred, not scheduled
│   ├── ⏸️ **Registry file encryption** — identity/federation registries at rest
│   │   ↳ trigger: none — filed, not scheduled
│   └── ⏸️ **DPI resistance** — D-023 ↳ trigger: Phase 3 opens ⚠️ ambiguous — the M6 Phase 3 is COLLAPSED (J-153)
│
├── ✅ **Discipline / JOURNAL hygiene** — both retrospective gap-closures shipped
│   ├── ✅ **JOURNAL Gap 1** — Phase 7.5 implementation retrospective · J-103
│   └── ✅ **JOURNAL Gap 2** — XGID Adoption v1 design + Phase 1 retrospective · J-102
│
└── **Cross-cutting principles** — standing decisions, continuous not milestone-shaped · DECISIONS.md
```

### How to read this view

The tree above is the **state board**. It is the authoritative current state of the project. Nothing below it overrides it.

**Node format** — one node, one line, structure from nesting:

```
<status> **<name>** — <description> · <canonical record> → <successor>
    ↳ trigger: <condition>
```

| Field | Rule |
|---|---|
| `<status>` | A **leading** symbol from the Status legend. Never parenthetical, never mid-line. |
| `**<name>**` | The milestone or arc identifier. Never bare — always carries a short descriptive title. |
| `· <canonical record>` | Where the work is written down: a `J-nnn` journal entry, a `DECISIONS.md D-nnn`, or a design document under `tasks/`. **A node closing a chain of commits cites the whole chain** — `· J-212 (J-207, J-208, J-209, J-210, J-211)` — not just the last one. |
| `→ <successor>` | On a **closed** node, what it unblocks or hands off to. Written on the closed node so whoever closes a milestone sees what it releases, on the same line. |
| `↳ trigger:` | Required on every 🟡 and ⏸️ **leaf**. `trigger: none — filed, not scheduled` is a legal and complete answer. |

**Six rules govern the tree. They exist because each one was broken at least once.**

- **R-1 — every node leads with a status symbol.** A status written inside parentheses is invisible to `grep '^✅'`.
- **R-2 — a container's status is derived from its children**: all children ✅ ⇒ ✅ · any child 🟢 ⇒ 🟢 · otherwise the weakest live state. The root is exempt. **A milestone with unfinished children is not done.**
- **R-2a — a derived container carries no trigger.** The condition is written once, on the leaf that owns it. Copying it up the tree creates places to go stale.
- **R-3 — a container of non-work carries no status at all**, only a link. A standing decision has a *force*, not a *state*; it is never "in play" and never "done". Standing decisions live in `DECISIONS.md` and are linked from here, not mirrored here.
- **R-4 — if a node needs a qualifier to be true, it needs a child instead.** `✅ … (2 of 7)` is a claim its own symbol contradicts. The finished half keeps ✅ and its link; the unfinished half becomes a 🟡 child with a trigger; the parent derives via R-2.
- **R-5 — a node's descriptive text stays under ~160 characters.** Without a bound, a chronicle simply relocates from prose into a tree row. **R-5a: the bound applies to the description, not to the link chain** — citing records is what the tree is for.

**Where the detail lives.** The tree carries state and a pointer. It does not carry narrative.

| You want | Read |
|---|---|
| what happened, and why | `JOURNAL.md` (live window) · `JOURNAL_ARCHIVE.md` (older) — by the `J-nnn` on the node |
| a standing decision | `DECISIONS.md` — by `D-nnn` |
| an active milestone's design and legs | `tasks/` — the task doc named on the node |
| the multiparty scenario ledger | `tasks/HANDOFF_MP_R3.md` §3 — 37 scenarios, R1+R2+R3 |
| UI component and substrate notes | `ui/docs/xgen-ui-notes.md` — N-numbered, append-only |

**Maintenance.** Same-commit discipline applies to this tree, no exceptions — D-074: a state change moves `docs/ROADMAP.md`, `CLAUDE.md`'s PLAY head, `JOURNAL.md` and the task doc in one commit. When a node closes, the closer writes its `· J-nnn` and its `→ successor` in the same edit. **A trigger that has fired is a defect: the node it guards is stale by definition.**

## Near future — designed or scoped, awaiting work

Ready to start, in order. The pre-UI chain runs first.

⚫ **Appendix F/I audit-against-code — CLOSED (J-398).** AF sub-pass ✅ (J-397, Appendix F v1.13). AI sub-pass ✅ (J-398): Appendix I reconciled to the as-built serializable types + event catalog (v1.6→v1.7; AI-F01–F16 doc-side — thread model, SpaceState/RoomState/IdentityRecord/FederationRelationship fields, PendingInvite, RoomPermission/Effect, 8 transport variants, identity.home_changed, re_registration). Three fundamentals promoted to single-source-of-truth appendices — **M** (Trust Assertions), **N** (Auth-Module/Plugin descriptors), **O** (`--aicontrol` control plane); `event_trace` typed enums folded into **Appendix G** (v1.2). **AI-F17 Joe-routed** (suspected code gap: the wire `identity.record` omits `is_ai`/`ai_capabilities`).

⬛ **Mockup stock-take + reconcile-to-as-built — DEPRECATED (J-403).** The planned reconciliation of the early-May `ui/docs/` mockups/concepts against the as-built surface was superseded by the component-library-first build (the RP track, now active in Present). The May-era mockup docs are stale-but-frozen; the clean-table UI draws from the as-built surface + the component library directly.

🟡 **Production identity→home-node discovery (MP-F13 / F1B-D5).** Routed from MP-F1b (J-333); the stranger-discovery path, distinct from derivation from known parties (D-091).

---

## Far future — specced, not yet scheduled

### UI

🟡 **Clean-table UI milestone.** Live UI built fresh after all pre-UI work (visual-merge approach deprecated); Round-2 GO. Component-library / substrate groundwork underway (RP track — see Present).

🟡 **Multi-device arc (R2-F09).** Device add/remove; D3-gated (AH-D4 epoch-advance).

⬛ **UI Phase 2 visual merge — DEPRECATED (J-284).**

### Streams (post-UI plane)

🟡 **Streams milestone.** Audio/video as a separate real-time plane with its own co-designed UI; relay-vs-SFU unlocked. Non-binding placeholder reserves `stream.*`/`media.*` + the UI stream-slot + capability-advert extensibility.

### Routed topics (flagged, not scheduled)

🟡 **Module-as-policy-bearer (Pattern B)** — flagged J-379.

### Parallel workstreams

⏸️ **Slovak translation pass — POSTPONED.** Suspended during active English development; a single pass after the English documentation reaches a stable end-state, or sooner if the need suddenly arises (lowest priority).

### Open areas (deferred, not scheduled)

⏸️ **Registry file encryption — POSTPONED.** Identity and federation registries at rest; deferred. Candidate **storage/security module** riding the D-080/085 module framework (encryption-at-rest as a module concern) rather than a standalone arc.

⏸️ **DPI resistance — POSTPONED.** Traffic masking / DPI resistance (D-023); Phase-3 area — investigation-only at this stage, resume when Phase 3 opens.

---

## Cross-cutting

A few items don't fit cleanly in past / present / near future / far future because they are continuous rather than milestone-shaped. Recorded here for visibility.

🟢 **Design discipline (D-069).** Every milestone Phase 0 must be Joe-locked before the implementing phase starts. Delegated technical drafts must self-flag open items. Canonical-document rule: each major surface gets one authoritative document, others point at it. M6's Phase 0 was the first milestone to follow this discipline end-to-end; Federation Event Propagation's design phase (Pass 2 just closed, Pass 3 next) is the second instance of the same pattern.

✅ **Audit-precedes-dependency discipline (D-071).** Every future milestone's Phase 0 includes a subsystem audit of whatever the milestone depends on. The Propagation Reliability Audit (J-081) established the pattern; D-071 names the discipline. Pairs with D-069: audit phase → design phase → implementation phase, each producing a canonical artefact. Sibling to D-065 and D-070. Promoted to DECISIONS.md 2026-05-18.

🟢 **Honest behaviour over polite behaviour (D-065).** Protocol-design principle. When the system can choose between a behaviour that misrepresents its state and one that honestly reflects it, XGen picks honest. Surfaces in multiple places: AI Client drop-on-throttle pacing, Node event rejection clarity, mute semantics, M6 accept-signal design, Federation Pass 2's `sync_complete` lock (F-6 chose explicit signal over silent quiet-time heuristic citing D-065), Federation Pass 2's pagination lock (F-7 chose explicit cursor over "felt incomplete" heuristic citing D-065).

✅ **Two events of equal importance, opposite direction (D-070).** Protocol-design principle. When the protocol exposes a signal from one party to another about the outcome of an action, both directions of outcome (acceptance and rejection) must be exposed with equal first-class status, AND both directions must carry the envelope-level correlation identifier so the originator can correlate the signal to the action it sent. Sibling to D-065. Promoted to DECISIONS.md 2026-05-18 with corrected post-audit framing (both halves — existence AND correlation — are load-bearing).

✅ **Bidirectional sustainability discipline (D-077).** Audit/design-phase principle. At every silent-discard, conditional-mutation, or fallible-operation-with-discard pattern, the sustainability question MUST be asked in both directions: forward-drift (what future callers could bypass this guard) AND backward-coherence (what current callers depend on this as a feature). Both answered simultaneously before locking any fix in isolation. Sits at meta-layer above the no-drift-surface discipline family (D-067 + D-070 + D-075 + D-076 v1.1 at protocol layers; D-077 + Rule 0 at meta layer). Origin: J-107 persistence-amendment re-walk — J-105 design phase asked the forward-only sustainability question and locked Q1 at (a).iii.β (`Result<(), GraphError>` ingest_event); Clair's Commit 2 implementation surfaced cross-milestone Phase 7 B3 amendment dependency (B3 federation-bootstrap path implicitly relied on the silent-discard as a feature); bidirectional discipline would have caught it at design time. Resolution Option Y locked: revert (a).iii.β → (a).iii.α, name the discipline as new principle, document broader audit as future-walk material under candidate D-NNN expanded scope. Surface-driven application per D-071 — NOT applied preemptively to all silent-discard sites in the codebase; applied at each fix site as it surfaces. **First worked instance** of D-077 value: Clair's Commit 3 prospective sweep closed three within-Commit-3 audit gaps atomically (abort-fold + identity-registry-persist + space-event-store-persist; federation-registry-persist audited and confirmed safe). Promoted to DECISIONS.md 2026-05-24 at J-107.

✅ **XGID typing is wire-format and persistence-format invariant (D-081).** Data-model / wire-format principle. Retyping a `String` identifier slot to a typed XGID flavour is a pure in-memory change: every flavour is `#[serde(transparent)]` over `Xgid(String)`, so it serialises byte-identically on every boundary (Node↔Node, Node↔Client, AI-control / batch JSONL, on-disk). No XGID Retrofit pass (1–5) changed a single serialized byte. `Display` is the canonical string form; `Debug` reveals the wrapper for diagnostics only. Sibling to D-076 in the wire-format discipline family; realises the XGID Adoption v1 Q4 invariance promise. Promoted to DECISIONS.md 2026-05-29 at J-148 (arc close). Numbered D-081 — D-080 was already taken by the Node-storage EventStore decision.

✅ **"operator" reserved for the AI-operator role; the Node administrator is a distinct infra principal (D-082).** Protocol-vocabulary / naming principle. "operator" = the AI-operator role only (moderator-parallel: operator : AI-identities :: moderator : room + members; fall-upward per D-064) — never an owner/admin alias. The Node admin principal is the **administrator** (prose) / **admin** (code, CLI, error-codes, config — matching `admin_ops`/`AdminContext`/`AdminError`); v1 has no gradation (OS-user-equals-administrator, session-scoped). owner/super-admin reserved as a future sub-tier (M7). A Node administrator auto-administers Spaces it originates/homes but NOT federated-in replicas (hosts-but-doesn't-own); the signing identity for admin-originated Space events is deferred to the A4 sub-design. Sibling to D-073 (naming discipline). Recorded at J-149; scope-refined at J-150 after a corpus audit found "operator" carries four senses — only the runtime admin principal (Sense D) is renamed; the AI-operator role, the wire field names (`operator_display_name` etc.), and the infrastructure "Node operator"/data-controller sense are all kept (an inline facet-specifier disambiguates Sense C where needed).

🟢 **Honest longer work over fast shortcuts.** Project-management principle. When project work surfaces a real gap, the default response is to close the gap properly, even if that delays downstream work. Locked during the audit's federation finding discussion; informs all milestone-sequencing calls. Pairs with the audit-precedes-dependency discipline above. Federation Pass 2 invoked it three times — to fold in F-6 (sync_complete) rather than defer, to fold in F-7 (pagination) rather than defer, and to fold in F-10 (HeldPending generalisation) rather than reject. Within Federation Event Propagation milestone scope as of J-105 the principle has eight recurrences (Phase 7.5; bidirectional; topo-sort design close J-097; runbook landing J-098; re-walk Step 2 J-099; re-walk Step 3 J-100; topo-sort implementation J-101; persistence-amendment sub-amendment milestone surfacing at J-104). Design-close events do NOT increment the count; the count belongs to the milestone-event the recurrence opened.

🟡 **Candidate D-NNN — "Ingest path invariant encoding"** (flagged at J-105, NOT promoted to DECISIONS.md; **scope expanded at J-107 re-walk** to cover five `ingest_event` silents + three drain helpers + M6 reject paths + B3 apply_event dependency). The persistence-amendment design phase locked Q1 at (a).iii.β (type-level Result-returning `ingest_event`); Y-lock revert at J-107 reverted to (a).iii.α (binary-void signature + log-level vigilance) under cross-milestone Phase 7 B3 amendment dependency surfaced at Clair's Commit 2. The candidate D-NNN names the rung-above-(a).iii.α project-level open question without pre-committing the project to a specific shape; rungs above (a).iii.β named explicitly (ValidatedEvent wrapper, sealed traits + visitor pattern, formal verification). Sibling-shape to D-076's v1 → v1.1 progression at different scope: v1 design-close didn't pre-promote the second invariant; v1.1 emerged after design walked it properly. Resolution at J-107 re-walk close: ship (a).iii.α immediately + name D-077 discipline + flag candidate D-NNN with **expanded scope** preserving optionality on the right rung per D-069 audit-vs-design boundary discipline. Future walk triggered when (a) dependent work surfaces a concrete drift instance, OR (b) Joe locks the candidate as worth pursuing on philosophical/strategic grounds independent of a surfacing gap. See JOURNAL J-105 + J-107 entries + `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` v1.2 §8 for the full reasoning trail.

> ✅ **M-RP-SHELF-FRAME — fixed-height shelves — DONE (J-530).** Both shelves (top favourites · bottom system) now hold a FIXED height whether empty or full: `.shelf[data-empty]`'s collapse (`min-height/padding/border → 0`) was neutralised, so an empty favourites strip no longer collapses and shifts the centre grid — a calmer, non-reflowing frame (Joe-locked). Skin-only (`ui/assets/skin.css`, 1 file, PROVISIONAL); zero Rust / component / registry / schema. Measured live 9222 (Rule 5): top **0 → 28px**, bottom **28.8px** unchanged; the 0.8px residual (box-sizing:border-box + `min-height`) accepted against the optical bar (N-128), the exact `height` pin filed-not-taken. The node inherits it free at **M-RP7.7**. → JOURNAL J-530, N-130.

> 🔒 **M-RP-SETTINGS — DESIGN LOCKED (J-534).** The next milestone's Phase-0 is locked (design/records-only, no code): `docs/xgen-settings-phase0.md` v1.0. **ONE Discord-shaped Settings modal** (never a new OS window — D-A: `surface:'window'` reads as a standalone modal area): a left **category menu (~¼) + a content pane** that swaps per selection (compact). The **plugin manager is the `Plugins` category** (`[info][settings][disable][uninstall]` rows — M-RP6.1m); two entry points — the `gear` opens Settings **@ Plugins**, a new **File ▸ Settings** item (above Restart) opens it **@ default**; `plugins-dialog` absorbed. **D-B: J-513 → B** — a plugin ships its own settings component; the declarative `settings_schema` is not built. **`grid-plate`'s backdrop is the settings mechanism's first tenant.** Legs: **A** the Settings shell + Plugins section · **B** the action row · **C** settings mechanism + backdrop. **Leg A ✅ DONE (J-535, `473b991`)** — the one modal stands up, `plugin-list` re-hosted as the Plugins section, About reused as the About section, `plugins-dialog` absorbed; Chat re-drove live (baseline **86**, gear→Plugins, swap, File▸Settings above Restart). ✅ **Leg B DONE (J-537, feat `15c1cd9`)** — the plugin action row + the Settings window: one-line rows, `[info][settings][disable][uninstall]` (one `onAction` seam), `session.disabled` with the `mounted`/`active` split, per-plugin host-tinted icons, **version replaces the badge**, a real `plugin-detail` info view, and Settings-as-a-**window** (`--settings-inset:120px`, own header round Back/×, independent-scroll columns). Chat re-drove every leg live 9222 after a full reload (N-132): baseline **99 === unique 99** quiescent · **closed-modal regression FIXED** (`.dialog[open]:has(.settings)` — N-134) · disable/enable/persist lifecycle **EXACT** (99→install 114→disable 105→reload 105 survives→enable 114→uninstall 99) · vite 175 · npm 77 · cargo **1517/0/62 IDENTICAL by construction** (20 files, 0 `.rs`). ✅ **Leg C CLOSED (J-540, `5f4a6fe` + `8b7ca1a`) → the SETTINGS arc is CLOSED** (the settings mechanism **D-B → D-120 MINTED** + the `grid-plate` backdrop setting B2). Re-driven live 9222 (Rule 5, full reload): baseline **99===99**; `[settings]` **enabled for grid-plate only** → drill “Grid Backdrop” **99→76** → the **real toggle flips the painted `data-pattern` both ways** (N-097), persists, survives reload; `cargo` **1517/0/62 IDENTICAL**, vite 178, npm 77. Swap generalised `detailId`→`drill={id,mode}`, `settings` intercepted locally, `app_client`/`plugin-list` untouched. **One defect caught in live verify** — a dead-button UI, *not a crash* (the persist `$effect` self-invalidated via a read-modify-write; fix `8b7ca1a` = `untrack`) → **N-136**. phase0 §9 + runbook → COMPLETED.) Filed: `M-RP-BACKDROP` (the backdrop-type menu; **type 1 = solid/gradient**, Joe 2026-07-17) · `M-RP-PLUGINS-NODE` · auto-disable-on-incompat · **`M-RP-DIALOG-CHROME` — dialog header/footer-snippet extraction** (J-512 D9, the 2nd `:has()` footer suppression; **ID provisional — Joe to bless, Rule 8**). → JOURNAL J-534 → J-537 · N-134.

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
