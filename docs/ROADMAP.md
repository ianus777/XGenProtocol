# XGen Protocol — Project Roadmap
> **Status**: ACTIVE  
> Version: 7.15  
> Date: May 2026  
> **Last updated**: 2026-08-13  
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

### Symbol discipline — mandatory

**This document carries the six symbols above and nothing else.** No other icons: not for emphasis, not for severity, not as title decoration. **They hijack attention away from the state column, which is the one thing this document exists to make scannable.**

- **State symbols belong on NODE LINES ONLY**, in first position after the tree glyph. An annotation line (`↳`) carries **no** state symbol — one there reads as a state that does not exist, and it makes the column unsearchable: before this rule landed, a search for a DONE node returned **62 prose matches** alongside the real ones.
- **Whatever an icon would have signalled is written as words.** If a line needs to shout, it shouts in text.
- Tree glyphs (`│ ─ ├ └ ↳`), arrows and enumeration marks are **structure and punctuation, not icons**, and are unaffected.
- **Exception, deliberate and known:** ⚫ CLOSED is in use at one node and is **not** in the table above. **Left as-is for a later review** — recorded rather than quietly legalised or quietly deleted.

## Update discipline — mandatory

**This document MUST be updated whenever a milestone or phase reaches a state change** — without exception, without deferral, without "I'll do it next session." State changes that trigger an update include:

- A track moves from PENDING to PLAY (work starts)
- A track moves from PLAY to DONE (work ships)
- A track moves from PLAY to POSTPONED (work pauses with known resume condition)
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

⚠️ **THAT REDUCTION HALF HAS NOT BEEN HAPPENING, AND IT WAS MEASURED (2026-08-11, J-715).** The tree carries **12 `↳ Owes:` lines totalling 8,395 characters — mean 700, longest 1,417** — and **five of the six longest are the most recently touched nodes**, two of them written by Chat at J-709 and J-710. 🔑 **They grew for a GOOD reason and in the WRONG place:** each long line exists because a fact was *measured* (`home_node` cannot designate a Space · `friend` returns zero hits · all 240 `block` hits are plumbing) and nobody wanted it re-derived — the *"claim narrower than the thing it describes"* defect class, correctly feared. 🛑 **But a NAVIGATION MAP IS NOT A KNOWLEDGE STORE**, and a 1,400-character line is unreadable exactly when it is needed: while scanning.

🔒 **THE MECHANISM, ruled by Joe 2026-08-11 — REDUCE ON COMPLETION, NOT ON A CALENDAR.** *A live node keeps its full `Owes`; the length is what stops the next person re-deriving the finding.* **When the node COMPLETES, the reasoning has already done its work** — it is in the JOURNAL and the notes by then — so **the closing commit cuts `Owes` to what is still owed, and points at the `N-nnn` / `J-nnn` that holds the reasoning.** ⚠️ **`M-RP-INTRO`'s own line already shows the target shape** — it cites `N-172` for the widget-on-the-wire finding in a handful of words. 📌 **Deleting the overflow instead of relocating it is NOT the rule** — that is the loss the length was protecting against.

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
│   ├── 🟡 **M-RP-LIVEFEED-REFRESH** — the live event router behind the members and rooms panels · J-598
│   │   ↳ `Owes:` — **§5's R4 (the stream's sync-from-cursor replay) is still OPEN and in no leg** · **Leg B's scope (B1/B2/B3) is Joe's** · **N-169: any caller of `setSpaces`, ever, triggers a members re-fill — recorded J-670, deliberately not fixed; memoising `effectiveSpaceId` is architecture and would silently un-build Leg C** · *`M-RP-IDENTITY-RESOLUTION Leg E refresh trigger` (NEW J-658) — **DISCHARGED J-670**: Leg C landed and Leg E needed no line of its own; C-3 unblocked (both sides, `D-133`)*
│   │   ├── ✅ **Leg 0 Phase-0** — routing shape + delta-vs-fill boundary locked; second-reader pass over §6 vs `wire.rs` done, three findings · J-616 (J-598, J-601)
│   │   ├── ✅ **Leg A router + members consumer** — runbook `tasks/RUNBOOK_LIVEFEED_LEG_A.md` v1.5 **COMPLETED**; **three** files, frontend only · J-618 (J-639)
│   │   ├── 🟡 **Leg B spaces/rooms consumer** · J-641
│   │   │   ↳ trigger: Joe rules Leg B's scope — **B1 / B2 / B3**. §6-ii turned it from a preference into a measured question; the runbook may be AUTHORED, not LOCKED
│   │   ├── ✅ **Leg C reconnect rule** — the `$effect` on `selfState.connection`; `loadSpaces()` extracted, `seenReady` latch + 5000 ms flap guard (provisional) · **CLOSED J-670** (J-658) · `4c50796` · `9983988` · `87307e8` · runbook `tasks/RUNBOOK_LIVEFEED_LEG_C.md` **v1.6 COMPLETED** ⇒ discharges `M-RP-IDENTITY-RESOLUTION` Leg E
│   │   ├── 🟡 **Leg D live verify** — two identities, one observer; `membership.kick` added at v1.11
│   │   │   ↳ trigger: Legs A–C land
│   │   └── 🟡 **Leg E records + close**
│   │       ↳ trigger: Leg D lands
│   ├── ✅ **M-RP-SELECT-ORIENT** — the panels keep saying where you are: R1 and R2 highlight from the latch, not the bus · **CLOSED J-697** · Phase-0 `tasks/M_RP_SELECT_ORIENT_PHASE0.md` v1.3 COMPLETED · runbook `tasks/RUNBOOK_SELECT_ORIENT.md` v1.4 COMPLETED · `517cf94` `d8edd85` `cd53c6d` `62c72f6` · `DECISIONS.md` D-146 + D-147 · supersedes `M-RP6.2` D4 opt-1 → opt-2 · discharges **`OQ-C3`** as C-4 and the `selection.svelte.ts` importer annotation · J-697 (J-692, J-693, J-694, J-695, J-696)
│   │   ↳ Owes: **option D, the two-state highlight — FILED, not refused; its cost is now LARGER, not expired** · at most a **`D-145a` addendum, DEFERRED** — `D-145` reaches non-document records and the operative test is whether anything has been BUILT on the artifact; Chat first argued `D-145` decided the case and later that it left a gap, and the first reading is the right one, so this is an extension rather than a new principle
│   ├── ✅ **M-TOOL-CDP-KEY** — the harness can press a key: `Input.dispatchKeyEvent`, a ten-key table, and a focus probe that names its owner · **CLOSED J-698** · one file `cdp-debug.ps1` · discharges `M-RP-SELECT-ORIENT` `L-12`'s stated limit — the trusted Enter reproduces the synthetic result at `62c72f6` exactly · J-698
│   │   ↳ Owes: **the first `ArrowDown` after a programmatic `.focus()` skips an item** — `activeIndex` is not synced on focus; `entity-panel`'s documented model, now observable through a trusted key, and it belongs to **M-RP-FOCUS**
│   ├── 🟢 **M-RP-MEMBER-ACT** — the members panel acts: LMC opens the DM, RMC opens the menu · **Leg C-bis CLOSED 2026-08-12 at J-716 — the MILESTONE IS NOT CLOSED: Leg E, the DM home, is its last leg and is UNBUILT (corrected at J-717)** · **Leg C CLOSED J-700 — LMC ships; the RMC context menu and Leg C-bis remain** · Phase-0 `tasks/M_RP_MEMBER_ACT_PHASE0.md` v1.11 · Leg C Phase-0 `tasks/M_RP_MEMBER_ACT_LEG_C_PHASE0.md` v1.5 COMPLETED · Leg C runbook `tasks/RUNBOOK_MEMBER_ACT_LEG_C.md` v1.2 COMPLETED · Leg C-bis `tasks/M_RP_MEMBER_ACT_LEG_C_BIS.md` v1.12 COMPLETED · Leg C-bis runbook `tasks/RUNBOOK_MEMBER_ACT_LEG_C_BIS.md` v1.13 COMPLETED · runbook `tasks/RUNBOOK_MEMBER_ACT_LEG_AB.md` v1.3 COMPLETED · `ce82ebe` `8c70d14` `132ce85` `41bf61a` `7305df2` `524d4f7` `b5d0908` `6a6c066` `e0d4d9a` `96a935f` `8601e677` `37c09d7` · `DECISIONS.md` D-143 + D-144 + D-145 · `N-169` + `N-170` + `N-171` + `N-174` + `N-175` + `N-176` + `N-177` + `N-178` + `N-179` + `N-180` + `N-181` + `N-182` · J-709 (J-708, J-707, J-706, J-680, J-681, J-682, J-683, J-685, J-686, J-687, J-688, J-689, J-690, J-691, J-692, J-699, J-700, J-702, J-703, J-704)
│   │   ↳ Owes: **RESTORED AT J-717 — J-716 CUT THIS LINE ON A CLOSURE THAT HAD NOT HAPPENED.** The J-715 reduction rule ties the cut to the commit that COMPLETES the node, and that commit is still ahead: Leg E is unbuilt. **Restored verbatim rather than re-summarised, and what J-716 genuinely discharged is annotated at the end rather than removed.** ANNOTATION AT THE SITE (D-131, J-718): the K2 item further down this line is STALE - KnownSpace.counterpart SHIPPED in Leg B under OQ8-K3 (xgen-common/src/state.rs:198), and Phase-0 section 6's own Leg E row already says so. Discharged, annotated, never deleted. — **`OQ5` re-sited J-690, not answered** — the partial first send and DM-creation-to-an-erased-identity are **Leg C-bis's**, now carried in that file's DoD · **what a never-DM'd click does was RULED at J-702 — option B, the draft view — and Leg C SHIPPED that click as a NO-OP** — a dead control, named and owned, whose removal is `OWED-1` in Leg C-bis's DoD and is now blocked on implementation rather than on Joe · **the RMC context menu is not built** — Leg C shipped LMC only · **section 6 leg 5 could not be measured and stays UNRULED** — clicking a member in a group room would move the latch to the DM and make R7 replace itself with two people, but Joe's client state carries one DM whose counterpart is erased, and an erased non-counterpart member is hidden in a group room, so no rendered group-room row has an existing DM; filed as `OWED-4` against both Leg C-bis and M-RP-PEOPLE, whichever lands first owes the measurement and owes showing it · **Leg E owes the Space-never-clicked case — a DM entered leaves R2 listing the previous Space's rooms (J-694)** · the erased row's retention behaviour is owed by the milestone that builds history-expiry and auth tiers · `N-171` half-apply shipped as written on Joe's delegated ruling, filed not fixed · cross-node invite discovery is a measurement of Chat's · clippy `-D warnings` has four pre-existing errors, not a tracked floor, left untouched — **AND WHAT J-716 GENUINELY DISCHARGED, ANNOTATED HERE RATHER THAN CUT: `OWED-1` discharged live · `OWED-4` measured, shown unruled per `D-146`, then ruled *"correct behaviour"* by Joe · `OQ5`'s two items carried and closed in Leg C-bis's DoD · the RMC context menu re-sited to Leg D, POSTPONED with a measured reason since section 5.7's census returned zero · `N-171` shipped as written on Joe's delegated ruling. STILL LIVE: Leg E's Space-never-clicked case · the erased row's retention · cross-node invite discovery · clippy's four errors. Reasoning at J-716 · J-717 · N-189 · N-190 · N-191 · `tasks/M_RP_MEMBER_ACT_LEG_C_BIS.md` sections 7-bis and 8**
│   │   ↳ trigger: **LEG C-bis IS CLOSED (J-716); THE MILESTONE IS NOT.** C-bis-5 drove eight checklist items green, quiescent registry `168` at a SEVEN-Space tree recorded AS THE TREE (`N-184`), **nothing of Joe's spent: no DM minted, client state `4204 B` unmoved, exact return to baseline.** **CORRECTION, J-717: J-716 marked this node DONE and its trigger *"ALL LEGS DRIVEN"*, and BOTH WERE FALSE** — section 6's leg table makes **Leg E the last leg**, gated on `C-bis-6` since J-710 and unbuilt. **The kickoff sentence *"C-bis-5 closes, then Leg E"* was inherited and never checked against the leg table.** Annotated, not rewritten. **STOP — section 5.5's gate could not pass as written and was RE-WORDED BEFORE IT WAS RUN (Joe, option A)**; the clamp is `skin.css`, Joe's, not landed, so the overflow is a named unskinned state with `M-RP-SKIN` as discharger. **KEY — section 7's owed `D-131` annotation turned on section 7 ITSELF** (`N-189`): the CSS never changed, the STRING did. **WARNING — `M-RP-INTRO`'s trigger *"Leg C-bis lands"* HAS FIRED and it has no Phase-0** — flagged to Joe, not started.
│ │ └── 🟡 **Leg E — the DM home + the R1 filter** — DM Spaces leave R1 and gain somewhere to live · **THE MILESTONE'S LAST LEG. Minted at J-717, which is late: it has been referenced by three records and owned a row in the on-screen table since J-713 while carrying NO NODE** — the dangling-pointer defect J-710 wrote the rule about, committed against the very milestone that wrote it · Phase-0 `tasks/M_RP_MEMBER_ACT_LEG_E_PHASE0.md` v1.1 ACTIVE (J-718) - ten findings, four of them from Joe's recall against records that read as self-consistent · content ruled by Joe at J-709 (**OQ3 = yes, DM Spaces leave R1**) and the three Phase-0 items ruled at J-718 (**H1 the ninth region + the v3 to v4 leaf-injecting migrate · G-c · L2**), all three DELEGATED under D-141 · grounded in Phase-0 section 6 and `tasks/M_RP_MEMBER_ACT_PHASE0.md` `A3` + `F-D` LOCKED AT J-720: E-1 runbook tasks/RUNBOOK_MEMBER_ACT_LEG_E1.md v1.1 ACTIVE, Joe locked it directly WITHOUT an adversarial read (Chat recommended one, Joe declined - recorded as provenance so a later defect shows the read was offered, not overlooked). Leg order is E-1 the DM Spaces widget, then E-2 the re-inject inside loadLayout, then E-3 the R1 filter, then E-5 close. E-4 IS ABSORBED INTO E-1 AND HAS NO CONTENT - L2 builds in E-1 section 3.2 and E-3 removes the DM rows from R1 anyway; the ID is kept, never renumbered, because the runbook and J-718 reference it. E-1 IS DONE at J-721 - the DM Spaces widget ships in 4 files (NEW dm-spaces.svelte, isDmSpace exported from spaces-state as the argued 4th file, one CLIENT_PLUGINS row, REGION_IDS plus REGION_NAMES), DEFAULT_LAYOUT untouched per Joe's option B, and V1 to V8 were DRIVEN GREEN on Joe's live client with svelte-check re-run by Chat at 0/34/15. Three measurement traps sprang and are filed as N-194: the registry delta matched Chat's prediction FOR THE WRONG REASON (168 to 174 on an identical screen with the panel unmounted - the cause was Settings Plugins going from 10 rows to 11, not DM rows), and V3 plus V6 as written both read the wrong thing and would each have filed a FALSE defect against correct code. Chat's own runbook section 4 could not run as written - insertLeaf is not exposed on window - found by RUNNING it, on the leg locked without an adversarial read. Owed onward: the DM rows show pure tail8 on a fresh client until a Space is latched, because the address book fills per-Space; not a defect, L2 was implemented exactly, discharger M-RP-STARTUP or an eager book fill. NEXT is E-2. E-2 PHASE-0 LANDS AT J-722: tasks/M_RP_MEMBER_ACT_LEG_E2_PHASE0.md v1.1 ACTIVE and runbook tasks/RUNBOOK_MEMBER_ACT_LEG_E2.md v1.0 PENDING, deliberately NOT LOCKED pending a Clair adversarial read (E-1 was locked without one and shipped a section 4 command that could not run). The finding is that P2 established TWO loadLayout callers and the leg needed THREE entry points: app_client.svelte:895 handleUistateLoad assigns a persisted layout via migrateLayout and never calls loadLayout, so a named UI state saved before dm-spaces existed would REMOVE the DM home from the running app and the self thread only GUI door with it. The species is the arc recurring one, a claim narrower than the thing it describes, and it survived the adversarial read because the read checked the claim written and not the claim needed. Also measured: the DEFAULT_LAYOUT return needs the re-inject too, so loadLayout gains a SINGLE EXIT; loadLayout is SHELL-local at layout-default.ts:137 and not core as the kickoff stated; neither id list carries a placement and REGION_IDS is 9 against 8 system region plugin rows; the unambiguity premise is guarded by AVAILABLE_CUSTOM gating rather than by :554 alone; insertLeaf is already TOTAL and idempotent so E-2 adds no algebra. Four items ruled by Joe under D-141: a SYSTEM_REGION_PLACEMENT table is the domain, one helper called from the single exit and from :895 with the applyLayout funnel NAMED AND NOT TAKEN as Joe architecture, no persist, and drive the :895 path. Consent for the named-state disk write is still owed at E-2b. Scope is two files, zero core, zero .rs, zero skin.css, DEFAULT_LAYOUT untouched at eight leaves. NEXT is the adversarial read, then Joe locks, then E-2a. THE ADVERSARIAL READ RAN AT J-723: brief tasks/CLAIR_LEG_E2_RUNBOOK_READ.md v1.0, verdict LOCKABLE WITH ONE NAMED CHANGE plus wording; runbook v1.0 to v1.1, Phase-0 v1.1 to v1.2. THE E-2a BUILD SURVIVED INTACT AND THE MOVER WAS ON THE VERIFY SIDE. PM-1: V4 had no route to its own precondition because handleUistateSave snapshots the live layout, which after boot always contains the re-injected home, so an ordinary save holds NINE leaves. Re-driven by Chat it is sharper than Clair stated: V4 as written would have PASSED VACUOUSLY, since loading a nine-leaf state shows the home whether or not the :895 call site exists at all - a probe that cannot fail, certifying the leg entire reason for existing without testing it. Swept into runbook section 5.1 as a seven-step staging route whose positive control is an on-disk read showing 8 leaves. Clair own proposed fix reproduced the E-1 section 4 shape - DEFAULT_LAYOUT is a module import and is not on window - so runbook section 4 now names all THREE unreachable symbols. W-1 the mountedPlugins empty-at-boot risk is FORECLOSED BY CONSTRUCTION and was overstated; W-2 the section 3.3 code block had already dropped the inline N-095 and D-115 comment its own prose said to preserve; W-3 FOUR file:line pointers were wrong and they are Chat, corrected in place, with the empty dccc9b1..HEAD source diff proving mis-measure rather than drift and every EDIT-TARGET pointer exact. Standing lesson: THE BUILD SURVIVED, THE PROOF OF THE BUILD DID NOT - an adversarial read must be pointed at the DoD and the probes explicitly, not only at the design. Also at J-723, N-195: the J-722 PLAY head never landed because a PowerShell char cast above U+FFFF threw and dropped 8 of 13 lines, and the byte-count check could not see it - byte parity proves CRLF integrity, never that content landed. NEXT is Joe locking runbook v1.1, then E-2a. J-724 DRIVES THE ONE GATE WHOSE EXPECTED VALUE THE RUNBOOK ADMITTED IT DID NOT KNOW, AND THE MEASUREMENT CARRIES A CONSEQUENCE NOBODY PRICED. The kickoff said the apps were DOWN and they were UP; a *xgen* process filter returned nothing while 9222 was listening, and /json/version was queried BEFORE attaching, showing the owner is msedgewebview2.exe, Tauri WebView2 host, serving XGen Client at localhost:5173 - the FILTER was wrong, not the port. All reads, nothing mutated, no cleanup owed. Joe live tree measured: root row 267/131/589/213 with a col 1579/421 holding spaces and self, so spaces parent runs col and edge bottom takes the SIBLING branch giving spaces, dm-spaces, self - E-1 live claim confirmed by RE-DRIVING rather than carried, so V2 now has a stated expectation. On disk 757 B with named = 0 states and session.layout = 8 leaves, so section 5.1 step-3 precondition and N-115 +4 both hold. F8-a: insertBeside sibling branch doubles the split and BISECTS the target slot, giving spaces 1579, dm-spaces 1579, self 842, so R1 SPACES HALVES ON FIRST BOOT AFTER E-2a - 760 px to about 380 px with seven Spaces in it. That is NOT covered by the appearance reservation: the spaces/bottom pair is Joe ruling and is right as an ADDRESS, but the 50/50 weight is insertLeaf built-in bisect which nobody chose - a mechanic that forces a default nobody selected is a mechanism question wearing an appearance question clothes. Filed as Phase-0 section 4 item 5, Joe, three options: B-a ship the bisect (zero code, keeps the helper pure composition), B-b re-weight inside the helper (it stops being composition and starts encoding a SIZE OPINION and every future table row inherits what weight), B-c a different address docking under self (zero code, one table value, but it revises Joe own option 1-B placement ruling). NO CHAT RECOMMENDATION deliberately, because the discriminator is how prominent the home should be on a screen Joe looks at, the axis D-121 says Joe is the better judge of. Chat does assert it must be ruled BEFORE the lock, because under B-a the 50/50 tree is persisted by the first grid gesture, so change it later means editing a tree already on Joe disk and not editing a default. It does not block E-2a shape. The runbook was deliberately NOT edited because Clair is mid-read on section 5.1 - editing a document while its reviewer is attacking it makes her report against a moving target; the runbook sweep happens after her hand-back. NEXT is Clair section 5.1 pass, then Joe rules section 4 item 5, then Joe locks, then E-2a. RULED AND SWEPT AT J-725: Joe ruled section 4 item 5 = B-a, ship the bisect (his own ruling, not a D-141 delegation) - after shipment he asserts the visual and edits the height himself. That plan is durable BY CONSTRUCTION: once he drags the seam the M-RP7.5 feeder persists the 9-leaf tree, and every later boot hits insertLeaf already-docked no-op by reference at mutate.ts:269, so the re-inject can never fight his height, on boot or on File Revert alike - the 50/50 only ever appears on a tree that has never been gestured. B-b and B-c recorded as considered and NOT taken, because B-b would have made the helper encode a size opinion. Runbook swept v1.1 to v1.2: section 3.1 carries the measured tree and the expected weights spaces/dm-spaces/self at 1579/1579/842, V2 flips from DISCOVER to ASSERT so a different shape is a Rule 6 report and not a re-baseline, section 7 item 4 is struck as discharged, and a new section 7 item 7 says F8-a halving is EXPECTED and must not be re-litigated as a defect at verify. Section 5.1 was rebuilt around Clair Q2: the control must PRINT the persisted leaf set, not assert absence from a value it never printed, because a read that returns nothing due to hitting the wrong key is indistinguishable from one that correctly found no dm-spaces - N-099 and N-194 turned on the control itself. New step 0 reads the authentic pre-home tree off disk rather than reconstructing it (F8-b), with the hand-splice kept as 1-alt fallback gated on an on-disk assertion, and finding 9 leaves is the fallback firing rather than a failure. NO GRID GESTURE anywhere in steps 1 to 6, because set() does not persist but the fold, resize and move handlers do, and step 6 is now a byte comparison against the step-0 read. Clair held at the read seat and was right to: her Q1 was closed by the J-724 drive rather than by her, and Q2 is what this sweep answers. STILL OWED before E-2a is Joe locking runbook v1.2; consent for the section 5.1 disk write stays his at E-2b. E-2 IS DONE AT J-726: E-2a built by Clair (2 files, +55/-10, zero .rs), E-2b driven by Chat with SEVEN GATES GREEN and V3 honestly UNDRIVEN, E-2c records. The DM home is now placed on EVERY path that produces a layout - boot, File Revert, and a named UI state load. THE FINDING: the positive control caught a defect in the step that defined it - section 5.1 step 3 said to read the saved control at named.<id>.layout and the real path is named.<id>.state.layout, so Chat read hit the wrong key and returned 0 ids. The WORDING is what caught it: under v1.1 assert-no-dm-spaces a wrong-key read returns nothing and PASSES, certifying a control that never reached its object, while under v1.2 PRINT-the-leaf-set it returned 0 rather than 8 and failed instantly - Clair Q2, asked one turn earlier about exactly this, paid for itself against the very step it rewrote. V4 proved the leg whole reason: an 8-leaf control with no dm-spaces at split 1762/842 loaded through the REAL dialog into 1762/1762/1684, the re-inject bisecting the control, predicted before the click, where a no-op would have left 1762/1396/842. The trap deliberately not taken was __XGEN_UISTATE__.load, which is the STORE load and not handleUistateLoad and would have bypassed :895 entirely. V3 is UNDRIVEN and it is Chat defect, third instance of one shape - layout.revert has no reachable route since runCommand is undefined, no bridge carries revert, and the command is deliberately element-absent; :586 is verified by diff and svelte-check and is byte-identical to :709 which IS driven, but that is an ARGUMENT and not a DRIVE, and the DoD says so rather than quietly passing. Discharger filed: a revert() on the __XGEN_LAYOUT__ DEV bridge, its own change. Other gates: V1 nine leaves and widget registered with count 3 and selfFirst false; V2 spaces/dm-spaces/self on the SIBLING branch as asserted, weights 1762/1396/842 because Joe dragged the seam before verify and 1762+1396 is exactly the bisect pair total with self untouched; V5 two load paths give exactly one dm-spaces leaf; V6 session.layout SHA unchanged across both so P-1 holds; V7 registry 184 with count equal to unique, enumerated as J-721 174 plus exactly 10 panel entries; V8 svelte-check 0/34/15 re-run by Chat, catalogue 435 by scope, no cargo claim. Standing form recorded: a runbook code block is a FLOOR for comments and not a ceiling - the implementer owns comment prose and what may not move is the executable text. Joe client returned to baseline and shown, SHA byte-identical, nothing sent and no DM minted. NEXT is E-3, the R1 filter, then E-5 closes the milestone. LEG E-3 PHASE-0 AND RUNBOOK LAND AT J-727: tasks/M_RP_MEMBER_ACT_LEG_E3_PHASE0.md v1.1 ACTIVE and tasks/RUNBOOK_MEMBER_ACT_LEG_E3.md v1.0 PENDING, deliberately NOT LOCKED pending a Clair adversarial read POINTED AT SECTION 5 BEFORE SECTION 3 - at E-2 the build survived both reads and the verify half was still wrong twice, and both were gate defects rather than code defects. F1: the filter goes on items at spaces-panel:50 and NOT on the local handle at :47, because :58-63 resolves the latched id against spaces in order to RECOGNISE a DM and suppress R1 highlight, so against a filtered list a latched DM stops resolving, the counterpart test goes false, and the suppression stops suppressing - the guard would invert while looking untouched. The filter and the suppression read the same list for opposite purposes, one wanting DMs gone and the other needing them present to recognise them, so items filters while selected and onActivate keep the unfiltered list. F2: the aggregate getter must filter or it lies, since debug publishes count as spaces.length, and filtering only items would render 4 rows while reporting 7, so a probe reading 7 after a CORRECT filter would file failure against correct code - the third time this arc a getter nearly produced a false verdict. F3: the three-readers census is STALE and there are NINE read sites across FIVE files - spaces-panel:47, dm-spaces:69/:92/:103, members-panel:156/:227/:285, room-latch:51, space-latch:59 - with rooms-panel reading the store only indirectly via spaceLatch.scopedSpace. The old census was true when written and E-1 plus C-bis widened it, the arc recurring species caught for the FIRST TIME BEFORE it became an instruction rather than after, and it hardens the lock rather than changing the design because EIGHT of the nine sites need DMs PRESENT. F4: N-184 gives two entities per row and 14 are measured for 7 Spaces, so removing 3 DM rows predicts minus 6, written as a hypothesis to TEST since at E-1 a predicted delta matched an observed one with no mechanism in common. Two rulings delegated: R-a folds the revert() DEV-bridge line into E-3 with the TWO-FILE scope stated up front, discharging E-2 undriven V3 and giving E-3 verify a second reachable load path, and it delegates to handleRevertUi so bridge and command are ONE path; S-a ships silent with no empty-state hint, because an explanation for a change only the developer has ever seen is a disclosure written for nobody. Section 5 pass condition is a TRANSITION and not a state, so V0 runs FIRST on the pre-E-3a build and PRINTS the ids of the DM rows rendered in R1. Section 7 names its own load-bearing weak point: F1 is reasoned from source and never observed, and V6 cannot catch it if it is wrong because once both guards hold they are indistinguishable. NEXT is the adversarial read, then Joe locks, then E-3a. THE READ RAN AT J-728: brief tasks/CLAIR_LEG_E3_RUNBOOK_READ.md v1.0, verdict LOCKABLE WITH TWO NAMED VERIFY CHANGES plus wording; runbook v1.0 to v1.1, Phase-0 v1.1 to v1.2. The E-3a BUILD survived intact - every section 1 pointer exact, F1 and F2 and F3 all hold against source, and the visible/spaces split is justified rather than over-shaped. BOTH PLAN-MOVERS WERE GATE DEFECTS, the third consecutive leg where that is true, and all of them Chat. PM-1: V7 could not fail, because loadLayout re-injects unconditionally at its single exit at layout-default.ts:193 and P-1 never persists, so the home is present whether revert() ran or not - and worse than a wasted gate, V7 whole job was to discharge E-2 undriven V3, which is the entire justification for section 3.3 opening app_client.svelte, so it would have retired a debt it never paid. Swept to a TRANSITION: record the on-disk SHA, set a visibly different tree via the non-persisting bare reassignment, CONFIRM the grid actually changed as the control, await revert(), then confirm the live tree restored and the on-disk SHA unchanged. Clair open caveat closed by measurement: five effects exist and none touches layout, and all seven setSessionLayout calls are gesture handlers, so set is safe as the staging vehicle. PM-2: the runbook claimed the central lock was UNGATED and that was FALSE, and it was the most load-bearing sentence in the document - selected depends only on spaceLatch and spaces and never on rendering, so with a DM latched debug().selectedId is null iff F1 is honoured and the DM raw id iff :47 was naively filtered, which makes V6 the F1 DISCRIMINATOR. The author conflated the doubly-caused PAINT with the singly-caused GETTER VALUE, and the mechanism sat in a comment two lines above the code being reasoned about - N-180 at arm length. Retracted rather than hedged per D-111; F1 is GATED and V6 names both expected outcomes in advance. Five wording sharpenings landed: V0 and V1 anchor on the RENDER and not the store since F3 keeps DMs in the store by design and a store-anchored probe would look like failure against correct code; V8 enumerated means NAMING THE SIX REMOVED IDS so a coincidental minus 6 cannot pass; V4 must confirm the latched Space is a real DM; section 3.1 code block now shows all three edits together because showing only the visible line invited exactly F2 failure; and spaceLatch has no direct bridge so do not name a reading you cannot take. Standing lesson three legs deep: four gate defects and zero build defects across two legs, all four the author, and reading section 5 FIRST cold is what surfaced both - an adversarial read starts at the gates. New soft spot named: V7 replacement route was written after the read that fixed it so nobody has attacked it, and its set-safety rests on a measured absence which is evidence rather than proof. NEXT is Joe locking runbook v1.1, then E-3a.
│   │       ↳ Owes: **RULED AT J-718, and three items on this line are CORRECTED IN PLACE (D-131) rather than silently dropped.** **`G13` = G-c** - Chat's J-709 dissolution DID NOT HOLD: apply_dm_promote never touches is_dm and NOTHING ever clears counterpart, so counterpart inherits the staleness it was proposed to escape; the mutable fact already exists in Rust as dm_constraints_active and is simply not in KnownSpace. Filter on counterpart, trigger named: a promote path writes to the client's KnownSpace tree · ANNOTATION (D-131, J-718): **the `N-173` citation here is FALSE and the miscitation is Chat's**, minted in one line at 8daf712 (J-713) - N-173 is the auth-tier versus processor-provenance collision; **the DM-row label is now `N-192`** and its wording stays Joe's · ANNOTATION (D-131, J-718): **`K2` is DISCHARGED** - it shipped in Leg B under OQ8-K3 · ANNOTATION (D-131, J-718): **the Space-never-clicked case (J-694) is TAKEN by C-bis-6** - members-panel:267-268 latches room AND space, measured at J-711 gate 1 · **NEW, and it outranks the rest: the R1 filter STRANDS THE SELF THREAD** - counterpart holds the session identity for the self thread, members-panel:246 refuses a self-click, and OQ6-E2 deleted the self_open command (CLI only) ⇒ the home MUST list the self thread · **the home lists EVERY DM, hides without removing access, and is a VIEW never a STORE** (Joe's T4 constraint, J-709; D-121 lens 2 question 4) · **the filter lives in `spaces-panel`'s `$derived` ONLY — the store retains every DM so the latch, `canSend` and the DM home can still resolve them**
│   │       ↳ trigger: **DISCHARGED since J-711.** The `E to D` gate was re-pointed onto `C-bis-6` at J-710 (Joe, option C) because Leg D had nothing honest to build; `C-bis-6` is green, so **Leg E is unblocked and unbuilt — it is next in front.**
│   ├── 🟡 **M-RP-PEOPLE** — a people-panel over the address book: everyone you know, independent of Space · FILED, NOT SCHEDULED at Joe's word (J-699) — after Leg C and the prepared milestones · feeder is `get_address_book`
│ │ ↳ Owes: **never named `contacts`** — `address_book.rs:38` reserves that word for Ch2's private contact record · **`last_seen` must never render as the person's activity** · separate from R7, not a boolean on it — measured and concluded at J-699 · inherits `OWED-4` jointly with Leg C-bis: whichever lands first makes a second non-erased DM reachable, owes the section 6 leg 5 measurement, and owes showing it to Joe before anyone rules ALSO OWED, MEASURED AT J-719 WHILE RULING THE DM HOME: the address book alone is NOT a superset of your DMs - it is filled per-Space by fill_and_members on latch, so a DM whose Space was never opened may have NO book entry and the person would be unreachable from a book-sourced list; the people list must read book UNION spacesState counterparts - AND presence DOES NOT EXIST in this protocol: 11 hits for presence or online across xgen-common and xgen-core and every one is the English word meaning a field is present, so an online dot is an unfed branch (N-091) until a presence milestone exists - AND the TIER CONSEQUENCE, which gates this milestone rather than decorating it: making the book a PRIMARY surface makes an UNENFORCED retention bound user-visible. Re-measured at J-719, not inherited from J-601: evict_older_than has ZERO production callers, every call site (613, 629, 637, 649, 665) sits inside the test module opening at line 349. Wiring the eviction stops being a tidy-up the moment the book becomes a navigation surface.
│   │   ↳ trigger: none — filed, not scheduled
│   ├── 🟡 **M-RP-STARTUP** — the client comes up somewhere: restore the last-visited room so R1, R2, R5, R6 and R7 open live instead of blank · FILED, NOT SCHEDULED at J-709 · no Phase-0 · grounded in `N-181` · feeder is `xgen-client_uistate.json`, which already exists at 758 B with keys `active`, `named`, `session`, `version`
│   │   ↳ Owes: **`home_node` CANNOT designate a Space, and that is measured** — Joe's `home_node` is one endpoint and all six of his Spaces carry it, all `role: owner`, so it answers which server to connect to and not which conversation to open · **restoring a SPACE alone leaves you half-entered** — `L1`'s deliberate B1 cost means R7 does not populate until a ROOM is latched, so the stream and the composer stay dead and the blank panels only move · **the room is the unit, because it implies its Space** · **fall back only on ABSENT, never on UNREACHABLE, which is Joe's rule** — absent is a local fact known instantly while unreachable is known only after a timeout, so falling through would make the startup destination depend on how fast a node replied; restore unconditionally, show the connection state, converge on reconnect as `app_client:182` already does · **nobody has read what the `active` key currently holds** — do not design on it until someone does · it stays UI state and not user data, a room id and never message content
│   │   ↳ trigger: none — filed, not scheduled
│   ├── 🟡 **M-RP-BLOCK** — the first member-scoped verb that left-click does not already perform · FILED at J-703, NOT SCHEDULED · no Phase-0 · nothing exists behind it today, and that is measured
│   │   ↳ Owes: **the census is the whole finding, and it is a grep, not an opinion** — `friend` returns ZERO hits (the one match is `conn.rs:13`, the word *log-friendly*) · all 240 `block` hits are DAG or async plumbing, with no verb, no blocklist and no wire event · `KnownSpace` carries no member list, so mutual membership is not even client-derivable · **it is protocol and node work before it is ever UI**, which is why it cannot ride a UI leg · **it is Leg D's named discharger** — M-RP-MEMBER-ACT Leg D is POSTPONED until a member-scoped verb exists that left-click does not already perform, and this is the nearest candidate · **the sketch's set answers the ANONYMOUS network's question** — who is this stranger and how do I get rid of them — so XGen may want a different set entirely, and inheriting Discord's three would be inheriting its threat model
│   │   ↳ trigger: none — filed, not scheduled
│   ├── 🟡 **M-RP-WIDGET-SUSPEND** — visibility off means the widget AND its feeds STOP · FILED at J-719 (Joe: *"widget-visibility is false == widget is stopped (or suspended), this have to be universal widget mechanic element"*) · no Phase-0 · nothing built
│ │ ↳ Owes: **the rule exists and does not reach far enough, and that is measured** — widget-tier W-5 already contracts *"wires listeners / observers / store subscriptions on mount and tears them down on unmount"*, but **the feed lives in the SHELL, not the widget**: only 4 widgets own any \ at all (3 of them dialogs) while pp_client.svelte owns **8 effects, 3 listens, 21 invokes**, and loadMembers (:223) is bound to a latch, not to Members being alive ⇒ **unmounting stops the render and NOT the drain** · **W-3 forecloses the easy fix** — a \ widget may not import invoke (pp_client.svelte:58), so the feed cannot follow the widget's lifecycle and the stop must be a **shell-side demand guard** · lands as **W-14** in ui/docs/xgen-widget-tier.md, a universal contract rather than a shell feature · **the feed audit rides IN this milestone** — a suspend milestone that leaves the feeds running delivers nothing · each feed declares **demand-driven** or **app-level**, and app-level **owes a reason** · LOCKED: **the control that hides a region must NOT live in a region** (else you hide the thing that un-hides); D-107's File▸Exit floor is safe **by construction** — the menu-bar is frame chrome OUTSIDE the layout tree · STOP: **DoD-BOUND: M-RP-MEMBER-ACT Leg E-2's re-inject MUST consult this milestone's hidden set** — absent-leaf currently means *saved before the region existed*; this milestone makes it ALSO mean *the user turned it off*, and one observable with two meanings is the G13 shape. **Not live today** (nothing can remove a system leaf), which is why E-2 ships unconditional and the guard is owed HERE (N-182 reserve nothing)
│   │   ↳ trigger: none — filed, not scheduled
│   ├── 🟡 **M-RP-PLUGIN-INSTALL-UI** — the missing entry point: a user cannot discover or install a custom plugin · FILED at J-719 · no Phase-0 · nothing built
│ │ ↳ Owes: **driven live on Joe's client, not read** — Settings ▸ Plugins renders **10 rows, every one system**, and **Connection Stats is absent** because the pane lists installed.active only; **76 registered elements, ZERO matching install/available/add** · the lifecycle SHIPPED at M-RP-CONNSTATS (install · uninstall · disable · leaf injection · persistence) and is reachable **only from the DEV bridge** window.__XGEN_PLUGINS__ (pp_client.svelte:406-408: *"No install UI exists yet … this drives the mechanism for verify"*), which is **import.meta.env.DEV only, dead-code-eliminated in a release build** ⇒ **unreachable by any user, by any route** · WARNING:️ **M-RP6.1m is NOT the discharger and must not be re-opened** — it was ABSORBED and SHIPPED as M-RP-SETTINGS Leg B ( asks/M_RP_SETTINGS_B_ACTION_ROW.md **COMPLETED**), which is why it never had a node; the four action buttons render live and grey correctly · OPEN: **disableable: Joe ruled !isCustom STANDS** (2026-08-12), so the D-113 derive-from-descriptor drift is **accepted knowingly, not filed** — the on/off need is met by M-RP-WIDGET-SUSPEND instead
│   │   ↳ trigger: none — filed, not scheduled
│   ├── 🟡 **M-RP-SKIN** — the appearance pass: every PROVISIONAL marker in the grid arc · FILED long-standing (Joe: *"majority of graphical elements will be changed or updated after ui mechanics completion"*) · **NODE MINTED AT J-719, LATE** · no Phase-0
│ │ ↳ Owes: **it was the ONE hit of the J-717 sweep, and it is Leg E's exact shape** — named as discharger inside a rigger: line and owning a row in the on-screen table, **with no node at all**, exactly the dangling pointer J-710 wrote the rule about · OPEN: **J-710's rule is EXTENDED at J-719 from *trigger condition* to *named discharger***, on the ground that a reader following the pointer finds nothing either way (Chat's reading, Joe's to overturn) · it is the named discharger for the fold chevrons, stripe/grip/triangle sizing, the folded strip's form, --region-pad/--region-seam, D2/D3's three tones, the editor-note wording, the M-RP6.6 ConnStats row-swap, M-RP-FOCUS, and Send-as-icon · WARNING:️ **flagged at J-564: a discharger that only ACCUMULATES stops being a plan** — either take it soon or split out the parts that are ready · STOP: **SKIN ONLY** (skin.css + tokens, N-090/N-025); if it needs a component change **that is a FINDING, not a licence**
│   │   ↳ trigger: none — UI mechanics completion
│   ├── 🟡 **M-RP-INTRO** — the DM welcome intro: whoever opens the conversation sends their own opening card · FILED at J-701, design conversation banked, **no Phase-0 yet** · ships as the OPENING MESSAGE in message chrome, not as system chrome
│   │   ↳ Owes: **the surface question was ruled by argument, not by taste, and the reasoning must survive** — an intro rendered as SYSTEM chrome is stranger-authored content in the system's voice on first contact, which is `D-113` S-5's no-trust-chrome rule and the classic unsolicited-first-contact vector; privacy does not mitigate it because the threat is one sender and one target in private, with no third party to notice · **as the opening message it is attributed, in the DAG, redactable, blockable and reportable, and symmetry is free** because the initiator sees their own intro as message one · **rich rendering must not put a WidgetMount on the wire** — see `N-172`: sender sends DATA, receiver renders with a widget it already trusts, unknown template falls back to plain text · a PUBLISHED intro visible before first contact has no home — `IdentityRecord` carries `display_name` and `is_ai` and nothing else, so that is a new federated world-readable field and a different feature · **needs Leg C-bis's first-send path to exist before it has anything to attach to**
│   │   ↳ trigger: Leg C-bis lands
│   ├── 🟡 **M-INTRO-POLICY** — receiver-side render policy: the sender sends freely, the recipient's home node decides what renders · FILED at J-701 · Joe's design, and it is the one that unlocks system-chrome rendering safely · protocol + node + client, NOT a UI leg
│   │   ↳ Owes: **the filter must be enforced in the CLIENT, not in the node, and this is not a preference** — today a node can read DM content so node-side filtering would work, but after PG-05 (MLS/E2E) the node holds ciphertext and cannot inspect, strip or rewrite anything, so a node-side filter ships with a known expiry date (`D-143`: the cheap route is unsound) · policy is AUTHORED at the receiver's home node and ENFORCED at render in the receiver's client · **`NodePolicy` is the precedent** — per-Space, node-held, admin show/set verbs — but it is admin-side only and **no policy surface reaches the client today**, so a client-facing policy read is new work · **auth tier is an INPUT to the policy, not the mechanism** — `AuthTier` is Tier1 to Tier4 but Tiers 2 to 4 need qualified institutions and do not exist, `build_dm_space_create_event` hardcodes auth_tier 1, and `auth_tier` has zero hits in the client, so a Tier-2-plus gate today excludes everyone · **three open design questions named at J-701** — the default posture for users who are not institutionally homed, whether a filtered intro is silent or disclosed (`D-065` argues disclosed and non-actionable), and the compliance gap that client-side enforcement cannot guarantee against a patched client without a client-attestation subsystem that does not exist · see `N-173`: the word Tier already means two unrelated things
│   │   ↳ trigger: M-RP-INTRO lands, or an institutional deployment forces it earlier
│   ├── 🟡 **M-RP-MEDIA** — audio and video in the message stream · FILED at J-701 · a `bodyExtras` widget over `message.file`, derived-local by construction
│   │   ↳ Owes: **no new message kind and no new event type are needed, and that is measured** — the wire already carries four (`message.text`, `message.file`, `message.reaction`, `message.redact`) and `message.file` content is `attachments` with a MIME field, so media rides MIME · the UI socket exists and is reserved-unfed by `D-065` (`bodyExtras`: *attachments / reactions*) and `MessageKind` stays `text` or `system` · the work is a media widget, not a protocol change · the blob is already in the DAG so the widget is a projection, per `N-172`
│   │   ↳ trigger: none — filed, not scheduled
│   ├── 🟡 **M-STREAM-LIVE** — live audio and video streaming · FILED at J-701 as a MILESTONE FAMILY, deliberately not folded into M-RP-MEDIA
│   │   ↳ Owes: **the DAG is an event log and a stream is a continuous media session** — `message.file` carries blob refs, not transport; there is no SFU, no WebRTC and no signalling anywhere in the build · folding this in with *more message types* would hide a milestone family behind a sentence that sounds like three · interacts with E2E, so it cannot be scoped before PG-05
│   │   ↳ trigger: PG-05 implementation ships (Arc H, D3 openmls in the build)
│   ├── ✅ **M-RP-TAIL8** — the unresolved-row fallback shows a short tail, not the whole key · phase-0 `tasks/M_RP_TAIL8_PHASE0.md` v1.3 · runbook `tasks/RUNBOOK_TAIL8.md` v1.4 · `165b821` · **CLOSED J-679** (J-678) → discharges `M_RP_MEMBERS.md` §6a, open since J-643
│   │   ↳ Owes: **`N-168`** — the erased row's `line-through` runs through the leading `…`; ship-and-file (Joe, `D-141`) · V4's empty-guard sub-case not driven, unreachable by product action
│   ├── ✅ **M-RP-IDENTITY-RESOLUTION** — what a member row shows before the client knows who it is · **CLOSED J-675** (J-644)
│   │   ↳ Owes: **the first milestone that RENDERS A MEMBER COUNT** — re-opens §5's C1 mismatch, at which point C2 and C3 return as live options · **`D-126` T3 re-priced J-675, NOT closed**; the row still carries no failure affordance · **A3's batched `identity_get` returns as a live option with a number** — N=5 serial, 1151 ms for one and 5459 ms for five, linear in N
│   │   ├── ✅ **Leg 0 Phase-0** — the four states, the tier frame, the two capability gaps · J-644
│   │   ├── ✅ **Leg A the `not_found` id list** — `FillReport` gained `not_found_ids: Vec<IdentityXgid>` through `fill_space_records` + the TS mirror · **CLOSED J-647** ⇒ closes **G-A**
│   │   ├── ✅ **Leg B the render rules** — ③ filtered from the rendered list (except the DM counterpart, §5a) · **CLOSED J-653** · runbook `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_B.md` **v1.4 COMPLETED**
│   │   ├── ✅ **Leg C the skin** — the three sub-legs C-1 / C-2 / C-3, all shipped · **CLOSED J-673** (J-650, J-654, J-655, J-672) · `8a650b1` · `03c92cc` · runbooks `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_C.md` v1.3 · `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_C3.md` v1.5
│   │   ├── ✅ **Leg D Tier-1 fetch on join** — `fetch_identity` persists the book and clears `unresolved`, the gate on the AI badge · **CLOSED J-672** (J-658, J-659, J-665, J-671) · `aa7d9c9` · `9901036` · phase-0 `tasks/M_RP_IDENTITY_RESOLUTION_LEGD_PHASE0.md` v1.2 · runbook `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_D.md` v1.1 ⇒ closes **G-B** with Leg E
│   │   ├── ✅ **Leg E the refresh trigger** — R1; **needed NO line of its own** — C-b0 MEASURED that the spaces re-fill cascades into the members fill · **CLOSED J-670** by `M-RP-LIVEFEED-REFRESH` Leg C (J-658, J-654) ⇒ **C-3 unblocked**
│   │   └── ✅ **Leg F live verify + records** — the milestone's FIRST behaviour verification; **all SEVEN obligations discharged against a real second identity** · **CLOSED J-675** · runbook `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_F.md` v1.7 · phase-0 v1.0 · J-653 (J-672, J-673, J-674, J-675)
│   ├── ✅ **M-RP-XGID-SLOT-RETYPE** — the identifier slots that regressed to `String` after the retrofit arc closed · **CLOSED J-669** (J-645, J-658, J-659, J-660, J-661, J-664, J-665, J-668) → `M-RP-THREAD-XGID`
│   │   ├── ✅ **Leg 0 Phase-0** — the sweep, the three rulings, and the hand-verified classification of all 88 · J-659 (J-660) · `tasks/M_RP_XGID_SLOT_RETYPE.md` §3a
│   │   ├── ✅ **Leg A the enforcement mechanism** — `D-137` promoted; `xgid-slot-gate.ps1` and `xgid-slot-manifest.tsv` added; Rule 0 gains item (5) · J-661 · DECISIONS.md D-137
│   │   ├── ✅ **Leg B the four address-book slots** — 4 slots plus the `BTreeMap` key; three `String`/typed bridge sites gone; cargo 1589 to 1592 · J-665 (J-660, J-661, J-664) · runbook `tasks/RUNBOOK_XGID_SLOT_RETYPE_LEG_B.md` **v1.2 COMPLETED**
│   │   ├── ✅ **Leg C the remainder** — 10 slots in 6 files; R4 fired on `AuthOutcome.identity_id` and was ruled minimal-projection · J-669 (J-668) · runbook `tasks/RUNBOOK_XGID_SLOT_RETYPE_LEG_C.md` **v1.3 COMPLETED**
│   │   └── ✅ **Leg D records + close** — the enforcement posture is stated at `tasks/M_RP_XGID_SLOT_RETYPE.md` §0; cargo 1589 to 1595 · J-669 · DECISIONS.md D-136 §3
│   ├── 🟡 **M-RP-THREAD-XGID** — mint a `ThreadXgid` flavour, or rule the three thread id slots DESCRIPTIVE · J-668 (J-669) ↳ trigger: Joe rules; `AE-D8` refused the flavour and the refusal needs reading first
│   ├── 🟡 **M-RP-VIEW-MENU** — a View menu between File and Help, items Address Book and Self Card · J-589 (J-666) · `tasks/M_RP_MEMBERS.md` §4c ↳ trigger: the Address Book UI and the Self Card UI both exist
│   ├── 🟡 **M-RP-WIDGET-SETTINGS** — a settings icon on each widget's tile stripe, deep-linking to that plugin's Settings section · J-666 ↳ trigger: every relevant plugin ships a `settingsComponent` — 1 of 6 today
│   ├── 🟡 **Clean-table UI milestone** — the live UI build
│   │   ↳ trigger: Round-2 audit GO + M10 closed *(transcribed from the UI container)*
│   └── 🟡 **Multi-device arc** — R2-F09 ↳ trigger: the UI prototype exercises device add/remove
│
├── ✅ **M-DOC-ROADTREE** — the canonical records become state boards · **CLOSED J-638** (J-598) → `M-RP-LIVEFEED-REFRESH` · `M-DOC-BACKFILL`
│   ├── ✅ **Leg 0 Phase-0** — scope ruled BOTH, node format ruled · J-598 (J-600)
│   ├── ✅ **Leg A pause + archive** — M-RP-MEMBERS Leg C paused; `ROADMAP_ARCHIVE_2026-07-26.md` taken · J-598
│   ├── ✅ **Leg B precondition**
│   │   ├── ✅ **P1 unlinked DONE markers** — 94 → 5, all five resolve · J-599
│   │   └── ✅ **P2 unresolved refs** — **CLEARED J-638** · J-603 (J-635, J-636)
│   ├── ✅ **Leg B-bis journal repair** — executed, verified, CLOSED · J-626 (J-625)
│   ├── ✅ **Leg B-ter the eleven that resolve nowhere** — **CLOSED AS DISCHARGED J-635** · spawned + titled by Joe J-626
│   │   ↳ trigger: none — runnable once runbooked. **It is AUTHORING, not repair**
│   ├── ✅ **Leg C `docs/ROADMAP.md`** — 761,422 → 43,741 B; tree kept, prose deleted, five format rules · J-604 (J-602, J-603)
│   ├── ✅ **Leg D `CLAUDE.md` B2** — 65 of 81 blocks archived, 640,645 → 316,680 B (50.6%); D-094 re-applied after a five-week lapse · J-615 (J-606, J-608–J-614)
│   ├── ✅ **Leg E the two-way closure log** — `CLAUDE.md`'s prose head; one closure log in THREE head notations · **CLOSED J-633**
│   ├── ✅ **Leg F bidirectional sweep** — is every known work item ON the roadmap, not just is every entry true · **CLOSED J-637** (J-617) → `M-DOC-BACKFILL`
│   ├── ✅ **Leg H what the census could not see** — **TITLED + CLOSED J-636** · spawned J-635 as B-ter's successor · ran before Leg G as required
│   └── ✅ **Leg G records + close** — the ~470 KB audited, the archive deleted, the milestone closed · **J-638** (J-617)
│
├── 🟡 **M-DOC-BACKFILL** — the milestones that were never on the board · J-637
│   ↳ trigger: none — filed, not scheduled
│
├── ⏸️ **Parallel workstreams**
│   └── ⏸️ **Slovak translation pass** — first touchpoint `xgen_appendix_a_sk.md`
│       ↳ trigger: English docs reach a stable end-state, or need arises
│
├── ⏸️ **Open areas** — deferred, not scheduled
│   ├── ⏸️ **Registry file encryption** — identity/federation registries at rest
│   │   ↳ trigger: none — filed, not scheduled
│   └── ⏸️ **DPI resistance** — D-023 ↳ trigger: Phase 3 opens ambiguous — the M6 Phase 3 is COLLAPSED (J-153)
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
| `↳ Owes:` | Optional, and the **only** other legal annotation. Names work a milestone still owes elsewhere — `D-133`. A cross-milestone gate is written on **both** sides or it goes stale invisibly on one. |

**These two are the only annotations the tree admits.** Anything else on a `↳` line is narrative, and narrative belongs in `JOURNAL.md` behind the node's `J-nnn`. **168 such lines — 61 KB — were deleted at J-663 after the file had regrown from 68,890 to 119,889 bytes in eight days.**

**Six rules govern the tree. They exist because each one was broken at least once.**

- **R-1 — every node leads with a status symbol.** A status written inside parentheses is invisible to `grep '^'`.
- **R-2 — a container's status is derived from its children**: all children ⇒ · any child ⇒ · otherwise the weakest live state. The root is exempt. **A milestone with unfinished children is not done.**
- **R-2a — a derived container carries no trigger.** The condition is written once, on the leaf that owns it. Copying it up the tree creates places to go stale.
- **R-3 — a container of non-work carries no status at all**, only a link. A standing decision has a *force*, not a *state*; it is never "in play" and never "done". Standing decisions live in `DECISIONS.md` and are linked from here, not mirrored here.
- **R-4 — if a node needs a qualifier to be true, it needs a child instead.** `… (2 of 7)` is a claim its own symbol contradicts. The finished half keeps and its link; the unfinished half becomes a child with a trigger; the parent derives via R-2.
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

## On screen now, and NOT a bug — what you are looking at, and which leg owns it

**Why this section exists.** Joe raised "DM Spaces are still in the Spaces list" three times across three sessions — not because he forgot the ruling, but because **nothing in the records answered it at a glance from what he could SEE.** A roadmap organised by milestone answers *"what are we building?"*; it does not answer *"I am looking at this right now — is it broken?"* This section is the second index, keyed by the screen.

🛑 **THIS LIST IS NOT A BLANKET EXCUSE, AND THE LAST COLUMN IS WHY.** Every row carries the condition under which the same thing on screen WOULD be a defect. **A row with no such condition does not belong here** — it would be an invitation to wave away real breakage, which is worse than having no list at all.

| What you see | Why it is expected | Owner | It IS a bug if… |
|---|---|---|---|
| **DM Spaces listed in R1** | `OQ3` ruled they leave R1 (J-709); the gate was re-pointed onto C-bis-6 at J-710 and is **discharged**. Unblocked, unbuilt — and as of J-717 it finally has a ROADMAP node. | **Leg E** | a DM row is **highlighted** — C-bis-6 shipped the suppression, so a lit DM row is a regression |
| **R1's DM rows read `DM with xgen://pubkey/e…`** | ANNOTATION (D-131, J-718): the `N-173` filing was FALSE - N-173 is the auth-tier designation collision. **Re-filed as `N-192`** 2026-08-12; ruled L2 (resolve at render time), wording still Joe's | Joe / `N-192` | a **group** Space renders a raw xgid — that would be identity resolution failing, not naming |
| **Your own message shows your raw xgid and a `GC` avatar** | `N-183`. The echo row never resolves its author; `GC` is the key's last two characters. **Pre-existing, newly reachable.** Filed only, at Joe's word. | filed, no node | R7, R8 or the self-panel **also** stop resolving that same identity — then it is resolution, not just the echo path |
| **Messages gone after a restart, with a banner saying so** | **§5's R4, sync-from-cursor replay, is OPEN and in no leg.** The node HAS them — it replayed 9 Space event stores at last launch. The client simply never fetches history, and the banner says so honestly. | `M-RP-LIVEFEED-REFRESH` Owes | messages vanish **within** one session — that is the live stream, not the missing backfill |
| **A DM's member list shows two rows while the debug count says one** | C-bis-7. `memberCount` reports the **fill**; the counterpart row comes from the **Space record**. The disagreement is the TRUTH. | shipped, J-713 | `rowCount` is not **2** in a DM, or `memberCount` has been "fixed" upward — inflating it makes a frontend count masquerade as a wire count |
| **R2 collapses to a single `dm` row inside a DM** | C-bis-6 option A, named in advance so it would not be reported as a defect | shipped, J-711 | R2 shows the **previous** Space's rooms — that is the dangling highlight C-bis-6 closed |
| **The DM failure line looks unstyled** | C-bis-8 ships it **structural only** — `min-width: 0`, `overflow-wrap: anywhere`, no colour, no size, no weight. `.composer-error` is a **`skin.css` selector and `skin.css` is Joe's file**; no leg may touch it. **`N-188`: the node's string repeats itself three times, so the rule must survive a long unbreakable one.** | Joe / `M-RP-SKIN` | it **overflows the tile**, fails to wrap, or migrates a scrollbar — that is structure, and structure is the leg's |
| **A DM failure line still showing beside a healthy node** | **`N-187`.** `_error` clears on `open` or on the next `create` — **a node coming back is neither.** The line reports **your last attempt**, not the node's health. | shipped, J-715 | it survives an `open` or a fresh `create` — **or** if someone wires it to connection state, which would clear a real unretried failure the moment a socket reconnects |
| **`dm-intro` and the composer look unstyled** | `skin.css` and `dm-intro`'s wording are **Joe's files**; no leg may touch them | Joe | — *(appearance only; a layout collapse is a different question and belongs in a leg)* |

📌 **MAINTENANCE, or this decays into folklore:** a row is **added** when a leg ships something visibly incomplete on purpose, and **deleted** by the commit that closes its owner. ⚠️ **A row whose owner has shipped is worse than no row** — it teaches the reader to distrust the whole table.

## Near future — designed or scoped, awaiting work

Ready to start, in order. The pre-UI chain runs first.

⚫ **Appendix F/I audit-against-code — CLOSED (J-398).** AF sub-pass (J-397, Appendix F v1.13). AI sub-pass (J-398): Appendix I reconciled to the as-built serializable types + event catalog (v1.6→v1.7; AI-F01–F16 doc-side — thread model, SpaceState/RoomState/IdentityRecord/FederationRelationship fields, PendingInvite, RoomPermission/Effect, 8 transport variants, identity.home_changed, re_registration). Three fundamentals promoted to single-source-of-truth appendices — **M** (Trust Assertions), **N** (Auth-Module/Plugin descriptors), **O** (`--aicontrol` control plane); `event_trace` typed enums folded into **Appendix G** (v1.2). **AI-F17 Joe-routed** (suspected code gap: the wire `identity.record` omits `is_ai`/`ai_capabilities`).

⬛ **Mockup stock-take + reconcile-to-as-built — DEPRECATED (J-403).** The planned reconciliation of the early-May `ui/docs/` mockups/concepts against the as-built surface was superseded by the component-library-first build (the RP track, now active in Present). The May-era mockup docs are stale-but-frozen; the clean-table UI draws from the as-built surface + the component library directly.

**Production identity→home-node discovery (MP-F13 / F1B-D5).** Routed from MP-F1b (J-333); the stranger-discovery path, distinct from derivation from known parties (D-091).

---

## Far future — specced, not yet scheduled

### UI

**Clean-table UI milestone.** Live UI built fresh after all pre-UI work (visual-merge approach deprecated); Round-2 GO. Component-library / substrate groundwork underway (RP track — see Present).

**Multi-device arc (R2-F09).** Device add/remove; D3-gated (AH-D4 epoch-advance).

⬛ **UI Phase 2 visual merge — DEPRECATED (J-284).**

### Streams (post-UI plane)

**Streams milestone.** Audio/video as a separate real-time plane with its own co-designed UI; relay-vs-SFU unlocked. Non-binding placeholder reserves `stream.*`/`media.*` + the UI stream-slot + capability-advert extensibility.

### Routed topics (flagged, not scheduled)

**Module-as-policy-bearer (Pattern B)** — flagged J-379.

### Parallel workstreams

**Slovak translation pass — POSTPONED.** Suspended during active English development; a single pass after the English documentation reaches a stable end-state, or sooner if the need suddenly arises (lowest priority).

### Open areas (deferred, not scheduled)

**Registry file encryption — POSTPONED.** Identity and federation registries at rest; deferred. Candidate **storage/security module** riding the D-080/085 module framework (encryption-at-rest as a module concern) rather than a standalone arc.

**DPI resistance — POSTPONED.** Traffic masking / DPI resistance (D-023); Phase-3 area — investigation-only at this stage, resume when Phase 3 opens.

---

## Cross-cutting

A few items don't fit cleanly in past / present / near future / far future because they are continuous rather than milestone-shaped. Recorded here for visibility.

**Design discipline (D-069).** Every milestone Phase 0 must be Joe-locked before the implementing phase starts. Delegated technical drafts must self-flag open items. Canonical-document rule: each major surface gets one authoritative document, others point at it. M6's Phase 0 was the first milestone to follow this discipline end-to-end; Federation Event Propagation's design phase (Pass 2 just closed, Pass 3 next) is the second instance of the same pattern.

**Audit-precedes-dependency discipline (D-071).** Every future milestone's Phase 0 includes a subsystem audit of whatever the milestone depends on. The Propagation Reliability Audit (J-081) established the pattern; D-071 names the discipline. Pairs with D-069: audit phase → design phase → implementation phase, each producing a canonical artefact. Sibling to D-065 and D-070. Promoted to DECISIONS.md 2026-05-18.

**Honest behaviour over polite behaviour (D-065).** Protocol-design principle. When the system can choose between a behaviour that misrepresents its state and one that honestly reflects it, XGen picks honest. Surfaces in multiple places: AI Client drop-on-throttle pacing, Node event rejection clarity, mute semantics, M6 accept-signal design, Federation Pass 2's `sync_complete` lock (F-6 chose explicit signal over silent quiet-time heuristic citing D-065), Federation Pass 2's pagination lock (F-7 chose explicit cursor over "felt incomplete" heuristic citing D-065).

**Two events of equal importance, opposite direction (D-070).** Protocol-design principle. When the protocol exposes a signal from one party to another about the outcome of an action, both directions of outcome (acceptance and rejection) must be exposed with equal first-class status, AND both directions must carry the envelope-level correlation identifier so the originator can correlate the signal to the action it sent. Sibling to D-065. Promoted to DECISIONS.md 2026-05-18 with corrected post-audit framing (both halves — existence AND correlation — are load-bearing).

**Bidirectional sustainability discipline (D-077).** Audit/design-phase principle. At every silent-discard, conditional-mutation, or fallible-operation-with-discard pattern, the sustainability question MUST be asked in both directions: forward-drift (what future callers could bypass this guard) AND backward-coherence (what current callers depend on this as a feature). Both answered simultaneously before locking any fix in isolation. Sits at meta-layer above the no-drift-surface discipline family (D-067 + D-070 + D-075 + D-076 v1.1 at protocol layers; D-077 + Rule 0 at meta layer). Origin: J-107 persistence-amendment re-walk — J-105 design phase asked the forward-only sustainability question and locked Q1 at (a).iii.β (`Result<(), GraphError>` ingest_event); Clair's Commit 2 implementation surfaced cross-milestone Phase 7 B3 amendment dependency (B3 federation-bootstrap path implicitly relied on the silent-discard as a feature); bidirectional discipline would have caught it at design time. Resolution Option Y locked: revert (a).iii.β → (a).iii.α, name the discipline as new principle, document broader audit as future-walk material under candidate D-NNN expanded scope. Surface-driven application per D-071 — NOT applied preemptively to all silent-discard sites in the codebase; applied at each fix site as it surfaces. **First worked instance** of D-077 value: Clair's Commit 3 prospective sweep closed three within-Commit-3 audit gaps atomically (abort-fold + identity-registry-persist + space-event-store-persist; federation-registry-persist audited and confirmed safe). Promoted to DECISIONS.md 2026-05-24 at J-107.

**XGID typing is wire-format and persistence-format invariant (D-081).** Data-model / wire-format principle. Retyping a `String` identifier slot to a typed XGID flavour is a pure in-memory change: every flavour is `#[serde(transparent)]` over `Xgid(String)`, so it serialises byte-identically on every boundary (Node↔Node, Node↔Client, AI-control / batch JSONL, on-disk). No XGID Retrofit pass (1–5) changed a single serialized byte. `Display` is the canonical string form; `Debug` reveals the wrapper for diagnostics only. Sibling to D-076 in the wire-format discipline family; realises the XGID Adoption v1 Q4 invariance promise. Promoted to DECISIONS.md 2026-05-29 at J-148 (arc close). Numbered D-081 — D-080 was already taken by the Node-storage EventStore decision.

**"operator" reserved for the AI-operator role; the Node administrator is a distinct infra principal (D-082).** Protocol-vocabulary / naming principle. "operator" = the AI-operator role only (moderator-parallel: operator : AI-identities :: moderator : room + members; fall-upward per D-064) — never an owner/admin alias. The Node admin principal is the **administrator** (prose) / **admin** (code, CLI, error-codes, config — matching `admin_ops`/`AdminContext`/`AdminError`); v1 has no gradation (OS-user-equals-administrator, session-scoped). owner/super-admin reserved as a future sub-tier (M7). A Node administrator auto-administers Spaces it originates/homes but NOT federated-in replicas (hosts-but-doesn't-own); the signing identity for admin-originated Space events is deferred to the A4 sub-design. Sibling to D-073 (naming discipline). Recorded at J-149; scope-refined at J-150 after a corpus audit found "operator" carries four senses — only the runtime admin principal (Sense D) is renamed; the AI-operator role, the wire field names (`operator_display_name` etc.), and the infrastructure "Node operator"/data-controller sense are all kept (an inline facet-specifier disambiguates Sense C where needed).

**When the cheap option is unsound, the proper one is taken even if it is heavier (D-143).** Project-management principle, designated 2026-08-06 at J-683 after operating undesignated since the Federation audit; previously carried by name as *"honest longer work over fast shortcuts"*. When project work surfaces a real gap and the cheap route would leave something **unsound** — a key that can collide, a claim that can go false, a branch that cannot be verified — the gap is closed properly, even if that delays downstream work or pulls later work forward. **The trigger is unsoundness, not effort:** where the cheap option is complete for what it does, D-143 does not fire and D-065's no-empty-machinery application governs instead. Joe drew both edges one hour apart in M-RP-MEMBER-ACT — OQ8 took the heavier K3 because K1 keyed a lookup on a user-writable string; OQ9 refused the heavier option because a const with an honest note asserts nothing that can go false. Designated because D-065 had accumulated three senses, two of which decide "should I do this work now?" in opposite directions. Pairs with the audit-precedes-dependency discipline (D-071) above. Locked during the audit's federation finding discussion; informs all milestone-sequencing calls. Federation Pass 2 invoked it three times — to fold in F-6 (sync_complete) rather than defer, to fold in F-7 (pagination) rather than defer, and to fold in F-10 (HeldPending generalisation) rather than reject. Within Federation Event Propagation milestone scope as of J-105 the principle has eight recurrences (Phase 7.5; bidirectional; topo-sort design close J-097; runbook landing J-098; re-walk Step 2 J-099; re-walk Step 3 J-100; topo-sort implementation J-101; persistence-amendment sub-amendment milestone surfacing at J-104). Design-close events do NOT increment the count; the count belongs to the milestone-event the recurrence opened.

**Owner content and client state copy are different classes (D-144).** Trust-boundary principle at the UI copy surface, locked 2026-08-06 at J-683. **Owner content** — Space name, room name, `topic`, a welcome message — is the owner describing their own place, arrives over the wire, and renders in the entity row's `secondary` slot. **Client state copy** — *"Select a room"*, *"No messages in this room yet"*, *"I cannot reach the others"* — is the client describing **itself**, and is authored by the client and by nothing else: not the Space owner, not the node operator, not a plugin. The user may restyle or re-language their own client; they may not receive these words from a third party. Because `members-panel`'s *"I cannot reach the others"* means the fill failed, an owner who could rewrite it could make a member's client misreport what it knows — what D-065 forbids, committed by a third party through a supported feature. Locked while it is free: `xgen-client` contains zero occurrences of `topic`, and the receptacle (`secondary`) already ships unfed in `core`.

**Candidate D-NNN — "Ingest path invariant encoding"** (flagged at J-105, NOT promoted to DECISIONS.md; **scope expanded at J-107 re-walk** to cover five `ingest_event` silents + three drain helpers + M6 reject paths + B3 apply_event dependency). The persistence-amendment design phase locked Q1 at (a).iii.β (type-level Result-returning `ingest_event`); Y-lock revert at J-107 reverted to (a).iii.α (binary-void signature + log-level vigilance) under cross-milestone Phase 7 B3 amendment dependency surfaced at Clair's Commit 2. The candidate D-NNN names the rung-above-(a).iii.α project-level open question without pre-committing the project to a specific shape; rungs above (a).iii.β named explicitly (ValidatedEvent wrapper, sealed traits + visitor pattern, formal verification). Sibling-shape to D-076's v1 → v1.1 progression at different scope: v1 design-close didn't pre-promote the second invariant; v1.1 emerged after design walked it properly. Resolution at J-107 re-walk close: ship (a).iii.α immediately + name D-077 discipline + flag candidate D-NNN with **expanded scope** preserving optionality on the right rung per D-069 audit-vs-design boundary discipline. Future walk triggered when (a) dependent work surfaces a concrete drift instance, OR (b) Joe locks the candidate as worth pursuing on philosophical/strategic grounds independent of a surfacing gap. See JOURNAL J-105 + J-107 entries + `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` v1.2 §8 for the full reasoning trail.

> ✅ **M-RP-SHELF-FRAME — fixed-height shelves — DONE (J-530).** Both shelves (top favourites · bottom system) now hold a FIXED height whether empty or full: `.shelf[data-empty]`'s collapse (`min-height/padding/border → 0`) was neutralised, so an empty favourites strip no longer collapses and shifts the centre grid — a calmer, non-reflowing frame (Joe-locked). Skin-only (`ui/assets/skin.css`, 1 file, PROVISIONAL); zero Rust / component / registry / schema. Measured live 9222 (Rule 5): top **0 → 28px**, bottom **28.8px** unchanged; the 0.8px residual (box-sizing:border-box + `min-height`) accepted against the optical bar (N-128), the exact `height` pin filed-not-taken. The node inherits it free at **M-RP7.7**. → JOURNAL J-530, N-130.

> **M-RP-SETTINGS — DESIGN LOCKED (J-534).** The next milestone's Phase-0 is locked (design/records-only, no code): `docs/xgen-settings-phase0.md` v1.0. **ONE Discord-shaped Settings modal** (never a new OS window — D-A: `surface:'window'` reads as a standalone modal area): a left **category menu (~¼) + a content pane** that swaps per selection (compact). The **plugin manager is the `Plugins` category** (`[info][settings][disable][uninstall]` rows — M-RP6.1m); two entry points — the `gear` opens Settings **@ Plugins**, a new **File ▸ Settings** item (above Restart) opens it **@ default**; `plugins-dialog` absorbed. **D-B: J-513 → B** — a plugin ships its own settings component; the declarative `settings_schema` is not built. **`grid-plate`'s backdrop is the settings mechanism's first tenant.** Legs: **A** the Settings shell + Plugins section · **B** the action row · **C** settings mechanism + backdrop. **Leg A ✅ DONE (J-535, `473b991`)** — the one modal stands up, `plugin-list` re-hosted as the Plugins section, About reused as the About section, `plugins-dialog` absorbed; Chat re-drove live (baseline **86**, gear→Plugins, swap, File▸Settings above Restart). ✅ **Leg B DONE (J-537, feat `15c1cd9`)** — the plugin action row + the Settings window: one-line rows, `[info][settings][disable][uninstall]` (one `onAction` seam), `session.disabled` with the `mounted`/`active` split, per-plugin host-tinted icons, **version replaces the badge**, a real `plugin-detail` info view, and Settings-as-a-**window** (`--settings-inset:120px`, own header round Back/×, independent-scroll columns). Chat re-drove every leg live 9222 after a full reload (N-132): baseline **99 === unique 99** quiescent · **closed-modal regression FIXED** (`.dialog[open]:has(.settings)` — N-134) · disable/enable/persist lifecycle **EXACT** (99→install 114→disable 105→reload 105 survives→enable 114→uninstall 99) · vite 175 · npm 77 · cargo **1517/0/62 IDENTICAL by construction** (20 files, 0 `.rs`). ✅ **Leg C CLOSED (J-540, `5f4a6fe` + `8b7ca1a`) → the SETTINGS arc is CLOSED** (the settings mechanism **D-B → D-120 MINTED** + the `grid-plate` backdrop setting B2). Re-driven live 9222 (Rule 5, full reload): baseline **99===99**; `[settings]` **enabled for grid-plate only** → drill “Grid Backdrop” **99→76** → the **real toggle flips the painted `data-pattern` both ways** (N-097), persists, survives reload; `cargo` **1517/0/62 IDENTICAL**, vite 178, npm 77. Swap generalised `detailId`→`drill={id,mode}`, `settings` intercepted locally, `app_client`/`plugin-list` untouched. **One defect caught in live verify** — a dead-button UI, *not a crash* (the persist `$effect` self-invalidated via a read-modify-write; fix `8b7ca1a` = `untrack`) → **N-136**. phase0 §9 + runbook → COMPLETED.) Filed: `M-RP-BACKDROP` (the backdrop-type menu; **type 1 = solid/gradient**, Joe 2026-07-17) · `M-RP-PLUGINS-NODE` · auto-disable-on-incompat · **`M-RP-DIALOG-CHROME` — dialog header/footer-snippet extraction** (J-512 D9, the 2nd `:has()` footer suppression; **ID provisional — Joe to bless, Rule 8**). → JOURNAL J-534 → J-537 · N-134.

---

## How to read this document

A reader landing here for the first time should be able to answer three questions in under a minute:

1. **What has the project shipped?** Past section, scanned by reading the bold milestone names.
2. **What is being worked on right now?** Present section.
3. **What's next?** Near future, top-of-list.

A reader returning after a gap can scan for state-changes by looking for symbols that have moved (a PLAY that became DONE, a PENDING that became PLAY, a new entry that wasn't there before). The roadmap is meant to be scannable, not exhaustively read.

For any item the reader wants more detail on, the canonical source is named (JOURNAL entry, DECISIONS reference, design doc, task file). ROADMAP.md is the map; the territory lives elsewhere.

---

*End of roadmap.*  
