# Task — Federation Event Propagation Phase 9 Implementation
> **Status**: ACTIVE  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-24 (J-115 — §3 Commit 6 Scenario 4 paragraph drift-corrected for variant row 2 (`forged_sender_with_resign_*`): outcome description amended from "1:1 to UnknownSender" to "HeldPending for 4 families + Validated-then-ingested for state.federation_add per Phase 7 B3 asymmetric cell." **Cause: D-078 first prospective-catch at Pre-Commit-3b-4 Joe-lock checkpoint #2.** Production-code verification surfaced that `validate_event` at `xgen-core/src/message/exchange.rs:626-632` (Phase 6 F-10 amendment, J-087) returns `ValidationOutcome::HeldPending { missing_identity: Some(sender.clone()) }` for unknown-signer events, NOT `Rejected(UnknownSender)`. The `UnknownSender` ExchangeError variant exists in the enum but is reachable only via the legacy `validate_steps_8_13` path. Phase 7 B3 amendment (J-088) creates the state.federation_add asymmetric cell. **Joe locked Reading B.i** (state.federation_add Validated-then-ingested per Phase 7 B3 design property, security-property doc-comment in test file). Test count (20 tests for Scenario 4) unchanged — only the variant 2 row's outcome contract was refined. Companion source-of-truth amendments at J-115: `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` v1.3 → v1.4 with §2.4 §E variant row 2 substantive rewrite + new §2.4.2 F-10 HeldPending contract sub-block; `tasks/FEDERATION_PROPAGATION_PHASE_9_COMMIT_3B_4_IMPL.md` v1.0 → v1.1 with §4.2 + §4.5 + §4.6 + §4.7 + §8 amendments + new §7.8 + four clerical fixes. **D-078's first prospective-catch instance** — sibling-shape to J-099 + J-109 + J-113 but procedurally distinct: those were retroactive catches (Clair halts mid-implementation); this is the first prospective catch (Clair halts pre-implementation at D-078 verification surface). Per D-077 bidirectional sustainability discipline + Reading B.i lock per J-115 framing. All amendments are clarifications of already-locked state — no version bump (stays ACTIVE v1.1); sibling-shape to J-109 + J-113 no-version-bump framing. Previous content: 2026-05-24 (J-113 — §3 Commit 6 Scenario 4 paragraph drift-corrected from "6 forgery variants × 5 event families = 30 forgery test cases" to "4 forgery variants × 5 event families = 20 forgery test cases" + variant names compressed to match the J-113 amendment. **Cause: canonical-document staleness surfaced at Clair's mid-Commit-3b-4 implementation read** — the original v1.1 enumeration included `mutated-sender` (ambiguous about which attack shape) + `future-timestamp` + `past-timestamp` variants. Production-code verification at `xgen-core/src/message/exchange.rs:46-87` confirmed the ExchangeError enum has 8 variants with no timestamp variant, and that `mutated-sender` collapses into either `bad_signature` (no-resign case, redundant) or `forged_sender_with_resign` (with-resign case, distinct). Amendment locks the 4-variant set with 1:1 mapping to production ExchangeError variants. Sibling-shape to J-109 §3 Commit 4 Scenario 6 drift correction. **Phase 9 task body lives under the original v1.1 seven-commit numbering**; §3.0a maps Clair's per-commit pickup to "Commit 3b-2/3/4/5-equivalent" naming used in companion documents — the Scenario 4 amendment lives in original-numbering Commit 6's sub-section but is reached via §3.0a's "Commit 3b-4" row. Companion source-of-truth amendments at J-113: `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` v1.2 → v1.3 with §2.4 §E table substantive rewrite + new §2.4.1 production-code-verification sub-block + new §4.6 Gap G6 entry (timestamp-bound validation as production gap). Per D-077 bidirectional sustainability discipline + Reading B lock per J-113 framing. All amendments are clarifications + drift-corrections of already-locked state — no version bump (stays ACTIVE v1.1); sibling-shape to J-109's no-version-bump framing and to §3.0a's no-version-bump precedent. Previous content: 2026-05-24 (post-J-109 — §3 Commit 4 Scenario 6 paragraph drift-corrected to Phase 7.5 §6 held-not-bypassed posture (outcome `HeldPending` not `Rejected`; `disposition = "held_pending"` field on existing `f3_reject` G2 trace event; recovery via `drain_pending_by_federation_relationship` arrival hook; 4007 federation_relationship_timeout); §3 Commit 4 DoD baseline corrected from stale "~525 minimum" to "~597 minimum" (against J-108 baseline 592 + ~3-5 new tests for Scenarios 5 + 6 + B1); §3.0a verification-step typo corrected 599 → 592 (Clair caught at Pre-Commit-3b-2-equivalent verification; §3.0a typo was fabricated number in the §3.0a clarification block from this session-arc). All amendments are clarifications + drift-corrections of already-locked state — no version bump (stays ACTIVE v1.1); sibling-shape to §3.0a's no-version-bump framing precedent. Companion source-of-truth amendments at J-109: `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` v1.1 → v1.2 with §2.6 substantive rewrite + new §2.6.1 contract walkthrough; `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY.md` header chain supersession pointer. Per D-077 bidirectional sustainability discipline + Reading B lock per J-109 framing. Previous content: 2026-05-24 (post-J-108 clarification — §3.0a inline amendment block added at the top of §3.0 making Commit 3b-1 CLOSED + Clair-next-active visible at this file's primary entry point per Rule 0. Single-file commit; no state-transition, no JOURNAL event — clarifies already-locked J-108 state. Companion documents (CLAUDE.md PLAY block + JOURNAL J-108 + docs/ROADMAP.md v1.23) already say the right thing; this amendment removes the residual reading-ambiguity at the task-file entry point where the §3.0 revised-five-commit-shape table still showed Commit 3b-1 in the active sequence. Status stays ACTIVE v1.1. Header version NOT bumped per honest-framing — this is a clarification of the J-108 state, not a substantive scope change. Sibling-shape to J-097 §3.0 amendment-landing single-file commit precedent. Previous content: 2026-05-24 (J-108 — Persistence-amendment sub-amendment milestone CLOSED. **Phase 9 Commit 3b-1 collapsed into the persistence-amendment milestone close per Q4(a) lock** (sentinel-tree four files at `xgen-node/src/tests/` — `phase9_harness.rs`, `phase9_three_node_anti_transitivity.rs`, `phase9_drop_and_recover.rs`, `mod.rs` — shipped atomic at the persistence-amendment milestone's Commit 3 `a677244` and crowned at this milestone's Commit 4; Scenario 3 (drop-and-recover with relationship state) transition FAIL → PASS verifies the persistence fix end-to-end at integration level). **Phase 9 RESUMES at Commit 3b-2-equivalent** with remaining scope: Scenarios 2 (three-Node anti-transitivity carried forward as part of Commit 3b-1's sentinel-tree atomic, so verify against §3.0 revised five-commit shape — Scenario 2 may also be considered closed at this milestone if the sentinel-tree file `phase9_three_node_anti_transitivity.rs` activates Scenario 2 alongside Scenario 3) + compound scenarios C2/C3-dropped/C5/C7/C9/C10 per existing Q4 Lock from J-091 + §3.0 revised five-commit shape with C3 dropped. Sub-amendment milestone shape: D-071 fourth project-wide instance (audit `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md` COMPLETED v1.1 at J-105 → design `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` COMPLETED v1.2 at J-108 → impl runbook `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` COMPLETED v1.2 at J-108 → impl shipped Commits 1+2+2a+3 → re-walk Track 1 at J-107 promoting D-077 bidirectional sustainability discipline → milestone close at J-108). Five Joe-locks across milestone Q1→Q4 walk + re-walk Y-lock: Q1 (a).ii + (a).iii.α (reverted from (a).iii.β at J-107 per Y-lock on cross-milestone Phase 7 B3 amendment dependency); Q2 (a) return-vector; Q3 all-three drain helpers; Q4 (a) sentinel-tree in-scope at milestone close (Commit 3b-1 collapses). After Phase 9 Commit 3b-2-equivalent ships, Phase 9 milestone flips PLAY → DONE, Federation Event Propagation milestone flips PLAY → DONE, and M6 (new) + XGID Retrofit Pass 1 unblock simultaneously. Status stays ACTIVE per the existing Phase 9 lifecycle — the umbrella is in-flight until Phase 9's own milestone-close commit ships. Q3 escalation criterion from J-091 still applies. Previous content: 2026-05-23 (§3.0 "Commit 3b status and locks" amendment landed at v1.0 → v1.1. Three Joe-locks at Phase 9 Commit 3b open: (1) **C3 dropped from Phase 9** — defers to `tasks/FEDERATION_STRESS_FOLLOWON.md` alongside C1/C4/C6/C8 because adding `state.federation_remove` is a spec change masquerading as a test fix (D-069 scope-creep shape) and direct state-dir editing papers over the absent revocation path (D-065 honest-behaviour-over-polite-behaviour shape); M4 MEDIUM-severity coverage gap honestly named; (2) **uniform in-process harness across all Commit 3b scenarios** — sibling-shape to Scenario 1's J-093 precedent; each file's doc-comment names what the in-process shape doesn't exercise (Scenario 3's `shutdown_tx.send(true)` clean shutdown does NOT exercise "process crashed mid-write" failure mode); (3) **one J-### per atomic commit** — five entries across the arc; per-commit cadence keeps `cargo test` evidence per Rule 5 and D-074's "JOURNAL.md in same commit" check unambiguous; sibling-shape to J-095..J-101 milestone-arc precedent. Revised five-commit shape: Commit 3b-1 (Scenarios 2+3) + Commit 3b-2 (Scenarios 5+6+B1 honesty) + Commit 3b-3 (Compound C2; C3 dropped) + Commit 3b-4 (NodeRuntime-level Scenario 4 + C5+C7+C9+C10) + Commit 3b-5 (milestone close per D-074). Original §3 Commit 3-7 sequence preserved below as historical authoritative record; §3.0's revised five-commit shape supersedes it for Clair's pickup. Honest-longer-work-over-fast-shortcuts disclosure: Federation Event Propagation milestone has produced seven such recurrences by J-101; Commit 3b may surface further gaps; expected behaviour per D-065 is to open them as sub-milestones rather than paper over. Single-file commit at this amendment landing — no companion files at CLAUDE.md/ROADMAP.md (those still correctly say "Phase 9 Commit 3b ←── HERE"). Previous content: 2026-05-23 (Phase 9 Commit 3b **RESUMED** at this commit. Topological-sort wire-order non-determinism fix LANDED per J-101 milestone close: five-commit Clair-facing sequence shipped under amended D-076 v1.1 — Commit 1 doc-pass (parents of `0543a86`) + Commit 2 determinism layer (`0543a86`) + Commit 2a causality layer (`4a6fd74` — Path B fix at `xgen-core/src/space/state.rs:797` `build_room_create_event` + validator companions unified per D-067 Option E + 17-site dag-test fixture updates under Posture β) + Commit 3 Phase 9 Scenario 1 second `#[ignore]` lift (`b370dc7`) + Commit 4 milestone close per D-074 (this commit's parent atomic commit; eight files). Phase 9 Scenario 1 (`two_node_federation_push_smoke_100_messages`) is now the activating integration-level regression lock for three decisions jointly: D-075 (bidirectional vantage-aware applier); D-076 v1.1 determinism layer (Commit 2's sort fix); D-076 v1.1 causality layer (Commit 2a's Path B fix). Verification at Commit 3 produced 8/8 green runs (5 isolated with `cargo clean` between each + 3 workspace) with no pre-existing flakes firing. **Phase 9 Commit 3b scope unchanged**: Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10 per existing Q4 Lock from J-091; expected ~5-7 atomic commits in their own sequence. Q3 escalation criterion from J-091 still applies. After Phase 9 Commit 3b ships, Phase 9 milestone flips PLAY → DONE, Federation Event Propagation milestone flips PLAY → DONE, and M6 (new) + XGID Retrofit Pass 1 unblock simultaneously. Status stays ACTIVE per the existing Phase 9 lifecycle — the umbrella is in-flight until Phase 9's own milestone-close commit ships. Previous content: 2026-05-21 (Phase 9 PAUSED at Commit 3a boundary — but the reason changes from "behind bidirectional `federation_nodes` fix" (the original Commit 3a stand-down per J-092 sub-entry) to "behind topological-sort wire-order non-determinism fix" (the new stand-down per J-096). Bidirectional `federation_nodes` implementation milestone CLOSED 2026-05-21 across four commits (`e975162` + `a730eda` + `cbceb41` + `f051039` + Commit 4 milestone-close); Scenario 1 (`two_node_federation_push_smoke_100_messages`) originally lifted `#[ignore]` in Commit 3 then re-stood-down in Commit 4 of the same milestone-close commit when verification surfaced a separate pre-existing wire-order non-determinism in `topological_sort_events` (`xgen-node/src/fanout.rs:193`) fed by non-deterministic HashMap iteration in `compute_federation_delta_for_space` (~:321). xgen-node-side diagnostic instrumentation confirmed the bidirectional fix is verified-correct (105 `dispatch_event` calls on B in both pass and fail runs; divergence is upstream wire order). Per Joe-direction the topological-sort finding opens as its own audit → design → impl arc per D-071, sibling-shape to the bidirectional arc just closed. Phase 9 Commit 3b (Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10) stays paused inside milestone scope until the topological-sort fix lands and Scenario 1 lifts a second time. Scope intact: 12 scenarios still in scope; 4 deferred compounds in `tasks/FEDERATION_STRESS_FOLLOWON.md` unaffected. Q3 escalation criterion from J-091 still applies once Phase 9 resumes. Initial: 2026-05-19 — authored from Joe-locked findings in `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` §8.))  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Ship Phase 9 of the Federation Event Propagation milestone — the deployment-level adversarial proof that federation works under conditions that matter. Twelve scenarios across mixed harness shapes; three observability preconditions; one flake-fix precondition; one milestone-close commit. After this task ships, the milestone flips PLAY → DONE and M6 (new) unblocks.

**This task implements the locked findings.** The survey is `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY.md` (COMPLETED). The findings are `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` (COMPLETED, v1.1). The four Joe-locks recorded in the findings §8 are the load-bearing scoping decisions; this task does not re-litigate them.

**Priority anchor (from survey v2.0).** Priority is working functions, not done-mark on roadmap. A milestone that ships green and turns out to have federation bugs three weeks later is a milestone that failed at its real job. Every scenario in this task is designed to *find bugs if they exist*, not to put a green checkmark next to an F-item.

---

## §1 — Mandatory reading

Read in this order before starting implementation.

| Source | What it gives | Why read it |
|---|---|---|
| `CLAUDE.md` MANDATORY behaviour rules | Rules 1–7. | Apply throughout. Quote actual `cargo test` output (Rule 5); never fabricate test counts (Rule 1); stop and report on tool failure (Rule 3); write JOURNAL last (Rule 4). |
| `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` (COMPLETED v1.1) | Full survey findings + four Joe-locks in §8. | Authoritative scope for this task. Per-scenario stress dimensions, honesty assertions, harness choices, observability gaps all locked. |
| `tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.9 | Original Phase 9 scope at runbook handoff. | Reference — but the locked scope in the findings supersedes the runbook's "Two-Node smoke + Three-Node if affordable" language. |
| `docs/xgen_federation_propagation_design.md` v1.0 ACTIVE | All ten F-items locked; §15 Implementation Complete record. | Authoritative protocol behaviour the tests assert against. |
| `xgen-client/src/app.rs` `cmd_stress_complete` (~line 3597+) | The harness precedent for deployment-level scenarios. | All "stress-complete shape" references in this task refer to this implementation. |
| Phase 5/6/7 integration tests (cited in survey §1) | Style + observability patterns for NodeRuntime-level tests. | Phase 9's NodeRuntime-level scenarios (4, C5, C7, C9, C10) extend the patterns shown there. |

---

## §2 — Locked decisions from survey findings §8

These are restated here so this task is self-contained.

**Q1 Lock — 12 scenarios.** 6 baseline (1, 2, 3, 4, 5, 6) + 6 compounds (C2, C3, C5, C7, C9, C10). Deferred: C1, C4, C6, C8 → `federation-stress` follow-on milestone (`tasks/FEDERATION_STRESS_FOLLOWON.md`).

**Q2 Lock — G4 (audit log for F-3) deferred to M6.** Phase 9 uses transient log parsing for F-3 rejection observation. M6 (new) Phase 0 owns the protocol audit-log schema.

**Q3 Lock — Flake fix option (i) first.** Add `#[serial_test::serial]` to both flake sites. Escalation criterion (option (ii) walk-back) defined in §6 of this task.

**Q4 Lock — Multi-commit Phase 9.** Expected shape (~5-7 commits) sequenced in §3 below.

---

## §3 — Commit sequence

### §3.0a — Commit 3b-1 CLOSED; Clair-next-active is Scenarios 5 + 6 + B1 (post-J-108 clarification, 2026-05-24)

**This block sits above the original §3.0 (preserved below) and supersedes the §3.0 revised-five-commit table only on the question of which row Clair picks up.** Per-commit scope detail in the table and in §3 Commit 3-7 sub-sections remains authoritative.

**Commit 3b-1 (Scenarios 2 + 3) is CLOSED.** Shipped at the persistence-amendment milestone's Commit 3 `a677244` (sentinel-tree atomic) and crowned at the milestone's Commit 4 per J-108 Q4(a) lock. Three files on `main`:

- `xgen-node/src/tests/phase9_three_node_anti_transitivity.rs` (NEW at `a677244`) — activates Scenario 2 (Three-Node anti-transitivity).
- `xgen-node/src/tests/phase9_drop_and_recover.rs` (NEW at `a677244`, J-104-authored) — activates Scenario 3 (Drop-and-recover with relationship state). Scenario 3 transition FAIL → PASS verifies the persistence fix end-to-end at integration level.
- `xgen-node/src/tests/phase9_harness.rs` (REFINEMENT at `a677244`) — gained `SavedNodeState` struct + `InProcessNode::shutdown_keep_data` method + `spawn_in_process_node_with_state` free fn to support Scenario 3's drop-and-recover lifecycle.
- `xgen-node/src/tests/mod.rs` (UPDATE at `a677244`) — two new sentinel module declarations.

**Clair's next-active is what the original §3.0 table called "Commit 3b-2"** — Scenarios 5 + 6 + B1 binary-level honesty test, mapped to original §3 Commit 4. For continuity with CLAUDE.md + ROADMAP + JOURNAL J-108 naming, this commit is referred to as **"Commit 3b-2-equivalent"** in those companion documents (the "-equivalent" suffix is the signal that the original numbering compressed when 3b-1 collapsed into the sub-amendment milestone close).

**Remaining commit sequence after the collapse:**

| Original row | What Clair picks up next | New files | Original §3 mapping |
|---|---|---|---|
| ~~3b-1 (Scenarios 2+3)~~ | ✅ CLOSED at `a677244` per J-108 | (already on `main`) | Original Commit 3 minus Scenario 1 |
| **3b-2** *(= Commit 3b-2-equivalent in J-108)* | ← **CLAIR PICKS UP HERE** — Scenarios 5 + 6 + B1 | `phase9_unknown_signer_first_contact.rs`, `phase9_federation_relationship_rejection.rs` | Original Commit 4 |
| 3b-3 | Compound C2 (F-5 anti-transitivity under push queue depth) | `phase9_compound_c2_anti_transitivity_at_load.rs` | Original Commit 5 with C3 dropped per Lock #1 |
| 3b-4 | NodeRuntime-level: Scenario 4 + C5 + C7 + C9 + C10 | 5 files in `xgen-core/src/tests/` | Original Commit 6 unchanged |
| 3b-5 | Milestone close per D-074 | Atomic multi-file commit per existing §3.0 file list | Original Commit 7 |

**Verification that 3b-1 is actually CLOSED on `main`.** Before Clair starts 3b-2-equivalent, confirm the three files exist by running `git ls-files | grep -E 'phase9_(three_node|drop_and_recover|harness)'` from project root. Expected output:

```
xgen-node/src/tests/phase9_drop_and_recover.rs
xgen-node/src/tests/phase9_harness.rs
xgen-node/src/tests/phase9_three_node_anti_transitivity.rs
```

Also confirm `cargo test --workspace` ships green at the J-108 baseline (592 tests per the milestone-close count; CLAUDE.md J-108 entry quotes the cargo output). If the local working tree shows uncommitted versions of these files or any drift from the `a677244` shipped versions, STOP per CLAUDE.md Rule 3 and report to Joe before starting 3b-2-equivalent.

**Note on the Joe-lock checkpoints below.** The five checkpoints in the original §3.0 (Pre-Commit-3b-1, Post-Commit-3b-1, Pre-Commit-3b-3, Pre-Commit-3b-4, Pre-Commit-3b-5) carry forward unchanged — but Pre-Commit-3b-1 and Post-Commit-3b-1 are now historical (closed at J-108). Clair's first live checkpoint is **Pre-Commit-3b-3** (confirm C2 setup mirrors the now-shipped Scenario 2 harness pattern in `phase9_three_node_anti_transitivity.rs`).

**Entry-point chain for Clair's session-open per Rule 0:** CLAUDE.md PLAY block → JOURNAL J-108 entry → this §3.0a block → original §3.0 table for per-commit scope detail → §3 Commit 4 sub-section below for per-scenario stress dimensions and honesty assertions for Scenarios 5 + 6 + B1.

---

### §3.0 — Commit 3b status and locks (revised five-commit shape, supersedes §3's seven-commit shape below)

**Status at this amendment landing (2026-05-23, post-J-101).** Commits 1 + 2 of the original §3 sequence shipped at J-092 (G1/G2/G3 observability + flake serialisation). Scenario 1 of original Commit 3 (Two-Node smoke) shipped at `b370dc7` as part of the topological-sort milestone's Commit 3, with `#[ignore]` lifted and the doc-comment frozen to J-101's five-event chronology + three-decision regression-lock framing. The remainder of original Commits 3-7 is **renamed and re-grouped** as Commits 3b-1 through 3b-5 below. The original §3 Commit 3-7 sub-sections are preserved below as historical authoritative record — their per-scenario stress dimensions, honesty assertions, and DoD checklists remain the source of truth for what each scenario must prove. §3.0 supersedes only the grouping + commit cadence, not the scenario-level scope.

**Three Joe-locks at Commit 3b open** (settled 2026-05-23 in conversation; recorded here per D-069 canonical-document discipline so the locks are not tribal knowledge):

**Lock #1 — C3 dropped from Phase 9.** The original §3 Commit 5 paired C2 + C3; C3 (F-3 rejection during F-1a recovery) required either (a) adding `state.federation_remove` as a new event type (spec change masquerading as a test fix — D-069 scope-creep shape; would have grown C3's scope into its own sub-milestone), (b) direct state-dir editing on A's data while B is down (papers over the absent revocation path — D-065 honest-behaviour-over-polite-behaviour flag), or (c) dropping C3 from Phase 9 and deferring to `tasks/FEDERATION_STRESS_FOLLOWON.md` alongside C1/C4/C6/C8. Joe-locked to (c). M4 (F-3 stale-snapshot race window, MEDIUM severity per findings §6 catalogue) is the named coverage gap; cost is acceptable for milestone closure. The deferred-follow-on milestone gains C3 to its existing C1/C4/C6/C8 list; the per-Space defederation surface that would have made C3 land cleanly is a natural M6 (new) primitive when M6 unblocks.

**Lock #2 — Uniform in-process harness across all Commit 3b scenarios.** Scenario 1's J-093 precedent picked in-process over subprocess (`stress-complete` shape) for fast + deterministic + direct runtime-state inspection. Commit 3b carries the precedent forward uniformly: all Scenarios 2, 3, 5, 6 and Compound C2 use `phase9_harness::InProcessNode`; the NodeRuntime-level Scenario 4 + C5/C7/C9/C10 use direct `dispatch_event` calls per the original §3 Commit 6 plan (no transport at all). Mixed-harness was rejected on maintenance-surface grounds: one harness pattern for binary-shape scenarios is cleaner than mixing in-process + subprocess for one scenario's marginal fidelity gain. **Honesty discipline applies per-file**: each file's doc-comment names what the in-process shape doesn't exercise. Scenario 3's `shutdown_tx.send(true)` clean shutdown does NOT exercise the "process crashed mid-write" failure mode — the survey's original subprocess `child.kill().await` would have. This gap is named in `phase9_drop_and_recover.rs`'s doc-comment as a known coverage limitation; "process-crashed-mid-write" coverage is `federation-stress` follow-on territory if it ever needs explicit testing.

**Lock #3 — One J-### per atomic commit.** The original §3 wording called for one consolidated J-### at Commit 7 close with sub-entries for the prior commits. Joe-locked to per-commit cadence instead: five J-### entries across the arc, one per atomic commit, each quoting its own `cargo test --workspace` output per CLAUDE.md Rule 5, each satisfying D-074's "JOURNAL.md in same commit" check unambiguously. Per-commit cadence is the durable pattern across J-095 (XGID Adoption v1 milestone close), J-096 (bidirectional `federation_nodes` milestone close), J-097..J-101 (topological-sort milestone arc — design close J-097, runbook landing J-098, re-walk Step 2 J-099, Step 3 J-100, implementation close J-101). Consolidated entries lose per-commit `cargo test` evidence and make D-074 ambiguous; per-commit cadence is more entries but cleaner discipline.

**Revised five-commit shape.**

| Commit | Scenarios | New file(s) | Original §3 mapping |
|---|---|---|---|
| 3b-1 | Scenarios 2 + 3 (Three-Node anti-transitivity + Drop-and-recover) | `xgen-node/src/tests/phase9_three_node_anti_transitivity.rs`, `xgen-node/src/tests/phase9_drop_and_recover.rs` | Original Commit 3 minus Scenario 1 (shipped at `b370dc7`) |
| 3b-2 | Scenarios 5 + 6 + B1 binary-level honesty test | `xgen-node/src/tests/phase9_unknown_signer_first_contact.rs`, `xgen-node/src/tests/phase9_federation_relationship_rejection.rs` | Original Commit 4 |
| 3b-3 | Compound C2 (F-5 anti-transitivity under push queue depth) | `xgen-node/src/tests/phase9_compound_c2_anti_transitivity_at_load.rs` | Original Commit 5 with C3 dropped per Lock #1 |
| 3b-4 | NodeRuntime-level: Scenario 4 + C5 + C7 + C9 + C10 | 5 files in `xgen-core/src/tests/` (Clair picks exact path per existing test organisation) | Original Commit 6 unchanged |
| 3b-5 | Milestone close per D-074 | Atomic multi-file commit (see below) | Original Commit 7 |

**Commit 3b-5 atomic-commit file list (per D-074):**
1. `CLAUDE.md` — Federation Event Propagation milestone block flips 🟢 PLAY → ✅ DONE; M6 (new) block flips 🟡 PENDING → 🟢 PLAY (or whatever the natural next-active state is); header `Last updated` bumped.
2. `docs/ROADMAP.md` — same state-transition reflected in Visual structure tree + Past/Present sections + "What's playing" line + header version bump.
3. `docs/xgen_federation_propagation_design.md` §15 — "eight implementation phases shipped" → "nine implementation phases shipped" with Phase 9 line added; header `Last updated` bumped.
4. `tasks/FEDERATION_PROPAGATION_COMPLETION.md` — Status flipped ACTIVE → COMPLETED.
5. `tasks/FEDERATION_PROPAGATION_PHASE_9.md` (this file) — Status flipped ACTIVE → COMPLETED.
6. `tasks/FEDERATION_STRESS_FOLLOWON.md` — created/finalised at PENDING with C1/C3 (new entry per Lock #1)/C4/C6/C8 + clock-injection seam scope + parallelism-investigation if Q3 escalation didn't fire.
7. `JOURNAL.md` — J-### milestone-close entry per Lock #3 cadence (per-commit, not consolidated); references the prior four J-### entries for Commits 3b-1 through 3b-4 by number; quotes final `cargo test --workspace` output.
8. Optional: `tasks/CLIENT_SIDE_CONSEQUENCES_AUDIT.md` placeholder per the J-081-shape precedent flagged in memory; M13 catalogue carry-forward home.

**Honest-longer-work-over-fast-shortcuts disclosure.** Federation Event Propagation milestone has produced seven recurrences of this pattern by J-101 (Phase 7.5 cold-start first; bidirectional `federation_nodes` second; topological-sort design close J-097 third; runbook landing J-098 fourth; re-walk Step 2 J-099 fifth; Step 3 J-100 sixth; implementation close J-101 seventh). Commit 3b crosses eleven bug-catalogue items (M1, M2, M3, M4-dropped, M5, M7, M9, M10, M11, M12, M14). **The five-commit shape above is the plan if nothing surfaces; the real shape will probably be longer.** Expected behaviour per D-065 + D-071: gaps surfaced during implementation open as sub-milestones with their own audit → design → impl arc; they do NOT get papered over inside the commit that surfaced them. Likely candidates: M4 (race window if real — already partially dropped via Lock #1's C3 deferral, but Scenario 2 + C2 may still surface a fragment), M9 (HeldPending double-drain under load — C10 tests this directly), M11 (F-3 drain-time approximation bound — C9 tests the design-disclosed 30 s bound; if it exceeds, that's an unrecorded bug).

**Joe-lock checkpoints for Clair across the arc** (sibling-shape to the topological-sort milestone's four-checkpoint pattern at J-100):

1. **Pre-Commit-3b-1.** Lock the Scenario 3 in-process "drop" implementation shape (`shutdown_tx.send(true)` + respawn vs. some alternative if one surfaces) before writing the file; Joe approves before code lands.
2. **Post-Commit-3b-1.** If Scenario 2 or 3 surfaces a finding that belongs at an upstream amendment layer (Phase 4 / Phase 5 / Phase 7 sub-amendment shape — sibling to Phase 7's B3 amendment), STOP per Rule 3 and report.
3. **Pre-Commit-3b-3.** Confirm C2 setup mirrors Scenario 2's harness pattern; if any divergence is needed (e.g. queue-depth measurement requires new instrumentation), Joe-lock the divergence before implementing.
4. **Pre-Commit-3b-4.** Lock Scenario 4's 30-test-case enumeration (6 forgery variants × 5 event families) by name before writing; Joe approves the enumeration. Scenario 4 is the largest scope of any single commit in this arc and the most likely to surface unanticipated validator-side findings.
5. **Pre-Commit-3b-5 (milestone close).** Final `cargo test --workspace` 5 isolated + 3 workspace = 8 green runs minimum verification rigour, sibling-shape to J-101's Commit 3 verification rigour. Pre-existing flakes (precedence env-var race; `reconnect_with_existing_tip_small_delta_delivered`) must not fire.

**Q3 escalation criterion stays active per §6.** The §6 flake-escalation rule from the original task file applies across all five new commits.

**Verification rigour for Commit 3b-5 milestone close.** Same 5 isolated runs (with `cargo clean` between each) + 3 workspace runs = 8 green runs minimum per J-101 precedent. Pre-existing flakes from `xgen-common::precedence` and `xgen-node::federation_delta_integration` are already serialised at Commit 2 (J-092); they should not fire under the workspace-parallelism shape these runs exercise.

**Cross-reference for Clair's pickup.** Entry point: this file's §3.0 (this section) → §3 Commit 3 + Commit 4 + Commit 5 + Commit 6 + Commit 7 sub-sections below for per-scenario scope detail (preserved authoritative) → `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` for per-scenario honesty assertions + dimensions → `xgen-node/src/tests/phase9_harness.rs` for the in-process harness contract → `xgen-node/src/tests/phase9_two_node_smoke.rs` for Scenario 1's shipped pattern as the sibling-shape template.

---

Phase 9 ships as **5-7 atomic commits**, each independently reviewable, each with its own JOURNAL sub-entry, each with quoted `cargo test` output. Final commit closes the milestone.

### Commit 1 — Observability preconditions (G1 + G2 + G3)

**Scope.** Three observability gaps closed in one commit.

**G1 — `xgen-node_state.json::peers` field.** Currently hard-coded to `vec![]` at [`xgen-node/src/app.rs:1775`](xgen-node/src/app.rs:1775). Read `FederationRegistry::peer_records` (already loaded at startup, wrapped in `Arc<Mutex<>>`), render each `PeerOperationalRecord` into the `FederatedPeer` shape that `xgen-common/src/state.rs::FederatedPeer` already exports. ~50 lines.

**Fields to render per peer:**
- `peer_node_id` — straight pass-through.
- `lost_connection` — straight pass-through.
- `last_successful_session` — straight pass-through (RFC 3339 string, already present).
- `next_reconnect_attempt` — straight pass-through.

If `FederatedPeer` schema in `xgen-common/src/state.rs` is missing fields the test scenarios need, extend the schema (add fields with `#[serde(default)]` for forward-compat per Phase 5 precedent). Schema extensions are CLAUDE.md Rule 6 territory — if any extension feels load-bearing, stop and ask Joe.

**G2 — Stable structured trace events for F-1 push + F-3 reject paths.** Today's `tracing::warn!`/`error!` calls carry free-form message text. Add `event = "..."` field with stable identifier. Affected sites:

| Site | File:line | New trace event name |
|---|---|---|
| F-1 push success | `xgen-node/src/federation_session.rs:201+` | `event = "federation_push_sent"` |
| F-1b drop (queue full) | `xgen-node/src/federation_session.rs:245` | `event = "federation_push_dropped_full"` |
| F-1b drop (peer unregistered) | `xgen-node/src/federation_session.rs:256` | `event = "federation_push_dropped_unregistered"` |
| F-5 guard fired | `xgen-node/src/federation_session.rs:209` | `event = "federation_push_skipped_origin"` |
| F-3 reject | `xgen-core/src/node/runtime.rs:378` | `event = "f3_reject"` |
| Co-located rejection log | `xgen-node/src/app.rs:1441` | `event = "event_rejected", reason = %reason` |
| Validation reject (per F-4) | inside `process_inbound` reject arms | `event = "validation_reject"` |

Each trace event includes structured fields (peer_node_id, space_id, event_id where relevant). Free-form message text is *additionally* allowed for human readability but the `event` field is the load-bearing identifier for Phase 9 tests.

**G3 — Fan-out trace event.** `xgen-node/src/fanout.rs::apply_fanout` success path adds:
- `event = "fanout_delivered", client_id = X, event_id = E` for each successful delivery.
- `event = "fanout_dropped_channel_full", client_id = X, event_id = E` for the existing try_send failure path.

Pairs with Scenario 1's honesty check #2 and Scenario 2's destination-side absence assertion.

**Verification.** `cargo test --workspace` passes (existing 519 tests unchanged). New trace events visible in test logs when run with `XGEN_LOG=info`. Commit message quotes actual test output.

**DoD for Commit 1:**
- [ ] G1 implemented: `xgen-node_state.json::peers` populated from `FederationRegistry::peer_records` at `build_node_state` call.
- [ ] G2 implemented: 7 trace event additions across federation_session.rs, runtime.rs, app.rs with stable `event` field.
- [ ] G3 implemented: 2 trace event additions in fanout.rs.
- [ ] `cargo test --workspace` passes; test count quoted.
- [ ] No public API breakage (Phase 5 + Phase 6 + Phase 7 tests still pass).
- [ ] JOURNAL sub-entry written.

---

### Commit 2 — Flake fixes (option (i))

**Scope.** `#[serial_test::serial]` applied at two sites.

**Flake #1 site.** Four tests in `xgen-common/src/precedence.rs`:
- `resolve_log_level_*` family at lines 148-178 per survey trace.
- Add `serial_test = "..."` to `xgen-common/Cargo.toml` `[dev-dependencies]` if not already present.

**Flake #2 site.** `xgen-node/src/tests/federation_delta_integration.rs` — `#[serial_test::serial]` on the test module or on individual tests in the module. Decision shape: if the module has 1-3 tests, serialise individually; if 4+ tests, the module attribute is cleaner. Verify by counting — `cargo test federation_delta_integration --list` is the canonical source.

**Escalation criterion (Q3 walk-back to option (ii)).** If during Commit 3-N (per-scenario test additions) any new Phase 9 integration test exhibits either:
1. A `127.0.0.1:0` bind race or "address already in use" failure under workspace parallelism that isn't explained by the test's own logic, OR
2. WS frame-ordering inconsistency where the same test passes in isolation but fails under `--workspace`,

then STOP per CLAUDE.md Rule 3. Report to Joe. Walk back to option (ii) — investigate the underlying tokio/WS race. Do NOT silently add more `#[serial_test::serial]` annotations across new tests as a workaround. The diagnostic signal IS Phase 9's deployment stress; suppressing it defeats the purpose.

**Verification.** Run `cargo test --workspace` 10 times. Per CLAUDE.md Rule 5, quote each run's PASS/FAIL outcome. Acceptable: 10/10 PASS. Anything else is a signal to escalate.

**DoD for Commit 2:**
- [ ] `#[serial_test::serial]` applied to both flake sites.
- [ ] Cargo.toml updates if needed.
- [ ] 10 consecutive `cargo test --workspace` runs all PASS; runs quoted.
- [ ] JOURNAL sub-entry written.

---

### Commit 3 — Baseline deployment scenarios (1, 2, 3)

**Scope.** Three deployment-level scenarios via `stress-complete` harness shape.

**Scenario 1 — Two-Node federation push smoke.**
- 2 Nodes spawned as separate `xgen-node` binaries.
- 100 events from Alice on A, 5 event types, mixed payload sizes (100B/10KB/100KB), 2 concurrent clients on A.
- Honesty assertions per findings §2.1 sub-item C:
  - Alice's post timestamp > `handshake_active_at_B_ts`.
  - Each event arrives on B's wire after `handshake_active_at_B_ts`.
  - Each event's `apply_federation_push` invocation observed on A via G2 trace event `federation_push_sent`.
  - F-5 guard did NOT fire (no `federation_push_skipped_origin` trace events for these 100 events).
- File location: `xgen-node/src/tests/phase9_two_node_smoke.rs` (new file).

**Scenario 2 — Three-Node anti-transitivity.**
- 3 Nodes spawned. A↔B and A↔C federated; B↔C explicitly NOT federated.
- 100 events from A, observed at B and C in parallel.
- Source-side honesty (load-bearing per findings §2.2): G2 trace event `federation_push_skipped_origin` fired on B for E-from-A (origin = `ReceivedViaFederation`). Zero peers iterated.
- Destination-side honesty: E appears in C's CommLog with `from=A`, never with `from=B`.
- File location: `xgen-node/src/tests/phase9_three_node_anti_transitivity.rs` (new file).

**Scenario 3 — Drop-and-recover.**
- 2 Nodes spawned. A↔B federated. 10 events queued, drop mid-stream, 2 sequential drop-recover cycles.
- Adapt harness: spawn B as `tokio::process::Child`; `child.kill().await` for drop; respawn with same `data_dir`.
- Honesty assertions per findings §2.3 sub-item C:
  - Assertion 1: R14 log lines (now `event = "federation_push_dropped_unregistered"` per G2) fire on A for all queued events.
  - Assertion 2: F-1a tip-exchange handshake observed on B's startup logs; `peer_records[B].last_successful_session` advances; `state.federation_add` NOT re-streamed in delta.
  - Assertion 3: Bob receives queued events in topological order.
- File location: `xgen-node/src/tests/phase9_drop_and_recover.rs` (new file).

**Verification.** `cargo test --workspace` passes; baseline 519 + Commit 1/2 unchanged + 3 new tests = expected ~522 minimum. Quote actual count.

**DoD for Commit 3:**
- [ ] 3 new test files implemented.
- [ ] All honesty assertions per findings §2.1-§2.3 sub-item C satisfied.
- [ ] `cargo test --workspace` passes; test count quoted.
- [ ] JOURNAL sub-entry written.

---

### Commit 4 — Baseline scenario 5 + 6 + B1 honesty test

**Scope.** Two more baseline scenarios + the Phase 7 B1 honesty test elevated to binary level.

**Scenario 5 — Unknown-signer first-contact.**
- Stress-complete shape: A↔B federated; Bob's Identity on A but NOT replicated to B.
- 4 timing variants × 4 missing variants per findings §2.5 sub-item E — but only the load-bearing case at binary level: identity arrives 1ms before timeout (resolves) and 1ms after timeout (must NOT resolve).
- Observation: `xgen-node_state.json::pending_identity_replication` polling; sub-second timing via tracing event timestamps.
- Honesty: `pending_identity_replication` decrements within 100ms of identity-replicate hook firing (proves hook ran, not periodic sweep).
- File location: `xgen-node/src/tests/phase9_unknown_signer_first_contact.rs` (new file).

**Scenario 6 — Federation-relationship deferral via HeldPending (post-Phase-7.5 §6 contract).**
- Stress-complete shape: Node X federates with B (session-level) but is NOT in any of B's Spaces' `federation_nodes`. X attempts to push event for Space S.
- Per survey findings v1.2 §2.6 + §2.6.1 (J-109 amendment): F-3 fail emits `DispatchOutcome::HeldPending` (NOT `Rejected` — pre-Phase-7.5 outcome), buffers the event on the federation-relationship trigger keyed by (peer, space), and the existing G2 `f3_reject` trace event fires with new `disposition = "held_pending"` field. Three sub-paths under Phase 7.5 §6: defer path (event in HeldPending); recovery path (federation_add arrives, drain hook fires, event ingests); timeout path (4007 federation_relationship_timeout sweep after `[sync].federation_relationship_timeout_seconds` default 180s).
- Compress to load-bearing cases at binary level: never-existed (the canonical case) + asymmetric.
- Honesty assertions per findings v1.2 §2.6 sub-item C (post-Phase-7.5 contract):
  - G2 `f3_reject` trace event fires with all four fields exactly: `disposition = "held_pending"`, `reason = "federation_relationship_missing"`, peer X, Space S.
  - `xgen-node_state.json::pending_federation_relationship` counter increments by 1 within ~100ms of the trace event fire (sub-second window proves counter is wired to the buffer-add site, not a periodic sweep).
  - Event E NOT in B's DAG (`xgen-node_state.json::spaces[S].event_count` unchanged).
  - B's local fan-out NOT invoked for E (no `fanout_delivered` G3 trace event with E's event_id).
  - **Recovery-path assertion (load-bearing for held-not-bypassed posture):** push `state.federation_add(X, S)` to B; within ~100ms drain hook fires, E re-dispatches and ingests, counter decrements, event_count increments, fan-out IS invoked. If this assertion fails but the defer assertions pass, the buffer is a black hole, not a holding cell.
- **Lock B1 + Phase 7.5 §5 honesty test (binary-level, three sibling sub-assertions).** Per Phase 7.5 §5 the skip set widened from Lock B1's single `StateFederationAdd` to three types adding `StateSpaceCreate` + `StateDmSpaceCreate`. X sends each of the three skip-set members; outcome MUST NOT contain `disposition = "held_pending"` AND `reason = "federation_relationship_missing"` (the conjunction distinguishes "skipped F-3" from "deferred via F-3"). Outcome may legitimately be HeldPending on F-10 Identity trigger; the disposition+reason conjunction is the load-bearing negative assertion.
- **Narrowness regression assertion.** X sends `state.room_create` against a Space that exists locally but where X is not federated. room_create is NOT in the Phase 7.5 §5 skip set (the discriminator is "creates the Space it references"). Outcome MUST be `disposition = "held_pending"` on the federation-relationship trigger — i.e. F-3 still fires for room_create. The Phase 7.5 unit test `f3_does_not_skip_state_room_create` at `xgen-core/src/node/runtime.rs:1006-1042` is the upstream regression lock; Phase 9 elevates the same narrowness check to the deployment level.
- File locations: `xgen-node/src/tests/phase9_federation_relationship_rejection.rs` (deferral + recovery path; main test). Optional second file `xgen-node/src/tests/phase9_federation_relationship_skip_set.rs` or fold the skip-set tests into the main file at Clair's call (Joe-lock checkpoint Pre-Commit-3b-2-equivalent).

**Verification.** `cargo test --workspace` passes; expected ~597 minimum (J-108 baseline 592 + ~3-5 new Scenarios 5 + 6 + B1 tests).

**DoD for Commit 4:**
- [ ] 2 new test files (Scenarios 5 + 6 plus B1 binary-level test).
- [ ] All honesty assertions per findings §2.5-§2.6 sub-item C satisfied.
- [ ] `cargo test --workspace` passes; test count quoted.
- [ ] JOURNAL sub-entry written.

---

### Commit 5 — Compound deployment scenarios (C2, C3)

**Scope.** Two compound scenarios at deployment level.

**Compound C2 — F-5 anti-transitivity under push queue depth.**
- Extends Scenario 2 setup: 3 Nodes, A↔B and A↔C federated, B↔C not federated.
- A pushes 100 events to B and C in rapid succession.
- Load-bearing assertion: log every outbound push from B (G2 `federation_push_sent` for B→A, `federation_push_skipped_origin` for B's attempts to forward A-origin events). Assert zero outbound pushes from B that carry an event with origin = `ReceivedViaFederation`. Source-side, not destination-side — catches bug catalogue M5 even if a hypothetical bypass affects only some events.
- File location: `xgen-node/src/tests/phase9_compound_c2_anti_transitivity_at_load.rs` (new file).

**Compound C3 — F-3 rejection during F-1a recovery.**
- Setup: A↔B federate for Space S. B drops (kill binary). While B is down, A applies `state.federation_remove` for itself on S (via client-side action on A — this needs a `state.federation_remove` event-emit code path; verify the event type exists or extend if needed).
- B comes back; A initiates F-1a handshake.
- Honesty assertions: handshake completes (per-peer relationship distinct from per-Space relationship); subsequent A→B push events for S are rejected with `event = "f3_reject"`, `reason = federation_relationship_missing`.
- **Note for Clair:** if `state.federation_remove` event type doesn't exist in the spec/code today, STOP and ask Joe before extending. Adding a new event type is a spec change, not a Phase 9 test addition. Likely alternatives: use `membership.kick` if the framing fits, or scope the test to "A's federation_nodes membership for S revoked via direct state edit on A's data dir while B is down" (lower fidelity but Phase 9-shippable).
- File location: `xgen-node/src/tests/phase9_compound_c3_f3_during_recovery.rs` (new file).

**Verification.** `cargo test --workspace` passes; expected ~527 minimum.

**DoD for Commit 5:**
- [ ] 2 new test files for compounds C2 + C3.
- [ ] If `state.federation_remove` ambiguity surfaces in C3: stopped and asked Joe per Rule 6 BEFORE implementing.
- [ ] `cargo test --workspace` passes; test count quoted.
- [ ] JOURNAL sub-entry written.

---

### Commit 6 — Compound NodeRuntime scenarios (4, C5, C7, C9, C10)

**Scope.** Five NodeRuntime-level scenarios. All in `xgen-core` test directory since they exercise `NodeRuntime::dispatch_event` directly without TCP.

**Scenario 4 — Validation asymmetry regression (NodeRuntime-level).**
- **4 forgery variants × 5 event families = 20 forgery test cases** per findings v1.4 §2.4 sub-item E + §2.4.1 (J-113 amendment) + §2.4.2 (J-115 amendment for variant row 2).
- **Variant 1, 3, 4** (`bad_signature_*`, `mutated_event_id_*`, `malformed_prev_events_*`) map 1:1 to `Rejected(SignatureFailure | EventIdMismatch | DagError)` at the production reject path per findings v1.4 §2.4.1 table.
- **Variant 2** (`forged_sender_with_resign_*`) — J-115 amended shape: 4 families (`message_text`, `membership_join`, `membership_kick`, `state_room_create`) produce `HeldPending { missing_identity: Some(attacker_uri), missing_predecessors: vec![] }` per Phase 6 F-10 amendment (J-087) at exchange.rs:626-632. **`state.federation_add` is the asymmetric cell**: per Phase 7 B3 amendment (J-088, exchange.rs:455-509) federation_add via federation channel skips step 11 sender-registration; step 12 passes on attacker's self-signature; event INGESTS as `Validated`. This is the locked security property — the protocol does NOT prevent an attacker who already has a federation channel from ingesting forged federation_add events. See findings v1.4 §2.4.2 for the full walkthrough and assertion template per family.
- For each test case: construct forged event of the named family with the named forgery applied, call `runtime.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id))`, assert outcome per variant + family contract above. Variants 1, 3, 4 assert `Rejected` with the specific substring; variant 2 (4 families) destructures `HeldPending` and asserts `missing_identity == Some(attacker_uri)`; variant 2 (state.federation_add) asserts `Accepted` + event lands in DAG (security-property regression lock). See findings v1.4 §2.4.2 + runbook §4.5 verbatim templates.
- Three previously-listed variants dropped from this enumeration at J-113: `mutated-sender` no-resign (redundant with `bad_signature`); `future-timestamp` and `past-timestamp` (assert against a contract that doesn't exist in production — see findings v1.4 §4.6 Gap G6 for the production gap framing).
- File location locked at runbook §4.4: `xgen-core/src/node/tests/phase9_validation_asymmetry.rs`. The runbook (`tasks/FEDERATION_PROPAGATION_PHASE_9_COMMIT_3B_4_IMPL.md` v1.1) is the active entry-point file for Clair's pickup.

**Compound C5 — Validation asymmetry under load.**
- 100 mixed valid+forged events fed via `dispatch_event` calls in random order.
- Assert per-event outcome independently: no forged event's rejection state affects a valid event's acceptance.
- File location: `xgen-core/src/tests/phase9_compound_c5_validation_under_load.rs`.

**Compound C7 — `continue_from` pagination at boundary.**
- 4 test cases per findings §3.7: Space with N=999, N=1000, N=1001, N=2000 events.
- Federate, observe delta size; assert all events arrive on the receiving Node's DAG (no boundary loss).
- NodeRuntime-level for the assertion sharpness; pair with one end-to-end smoke test (can be folded into one of the deployment scenarios if budget tight).
- File location: `xgen-core/src/tests/phase9_compound_c7_pagination_boundary.rs`.

**Compound C9 — F-3 drain-time approximation hazard.**
- Setup: federation event from peer X for Space S buffers in B's PendingBuffer (missing predecessor). X removed from S's `federation_nodes`. Predecessor arrives. Drain re-dispatches with `peer_node_id: None`.
- Assert: event ingests (the approximation accepts) — and document the behaviour. Bound assertion: event was buffered for ≤ 30 s (F-4a window). If bound exceeds 30 s, that's an unrecorded bug.
- File location: `xgen-core/src/tests/phase9_compound_c9_drain_time_hazard.rs`.

**Compound C10 — Identity-replicate hook serialisation under lock contention.**
- 3 concurrent federation peers push events for unknown Bob to B; 3 concurrent identity-replicate messages for Bob.
- Assert: no event is drained twice (no duplicate-ingest DAG rejection); each buffered event drains exactly once.
- File location: `xgen-core/src/tests/phase9_compound_c10_identity_lock_contention.rs`.

**Verification.** `cargo test --workspace` passes; expected ~565 minimum (Scenario 4 adds ~30 tests, others add ~5 each).

**DoD for Commit 6:**
- [ ] 5 new test files for Scenario 4 + compounds C5, C7, C9, C10.
- [ ] All honesty assertions per findings §2.4 and §3.5/§3.7/§3.9/§3.10 satisfied.
- [ ] `cargo test --workspace` passes; test count quoted.
- [ ] JOURNAL sub-entry written.

---

### Commit 7 — Milestone close

**Scope.** Final close-out. No new tests; updates to CLAUDE.md + ROADMAP.md + JOURNAL.md + design doc.

**Updates:**
1. **CLAUDE.md.** Federation Event Propagation milestone block flips 🟢 PLAY → ✅ DONE. M6 (new) block flips 🟡 PENDING → ACTIVE (or whatever the natural next state is). Last updated bumped.
2. **ROADMAP.md.** Same state-transition reflected. Last updated bumped.
3. **`docs/xgen_federation_propagation_design.md` §15** updated from "eight implementation phases shipped" to "nine implementation phases shipped" with Phase 9 line added. Last updated bumped on file header.
4. **`tasks/FEDERATION_PROPAGATION_COMPLETION.md`** flipped from ACTIVE to COMPLETED. Last updated bumped.
5. **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** (this file) flipped from ACTIVE to COMPLETED. Last updated bumped.
6. **JOURNAL.md.** J-### consolidated entry covering all 6 prior commits' sub-entries plus close-out. Test count quoted from actual `cargo test --workspace` output. Milestone shipping summary.

**Verification.** Final `cargo test --workspace` run quoted in commit message. CLAUDE.md milestone block reflects DONE state. ROADMAP.md in sync. Design doc §15 lists Phase 9.

**DoD for Commit 7:**
- [ ] CLAUDE.md updated: milestone PLAY → DONE.
- [ ] ROADMAP.md updated: same.
- [ ] Design doc §15 updated: Phase 9 line added.
- [ ] Runbook + this file flipped to COMPLETED.
- [ ] JOURNAL J-### consolidated entry written.
- [ ] Final `cargo test --workspace` output quoted.
- [ ] Joe pushes the commit.

---

## §4 — Aggregate Definition of Done — Phase 9 milestone

These items must hold at Commit 7 close:

- [ ] All 7 commits shipped sequentially per §3.
- [ ] 12 scenarios implemented (6 baseline + 6 compounds: C2, C3, C5, C7, C9, C10).
- [ ] 3 observability gaps closed (G1, G2, G3).
- [ ] 2 pre-existing flakes serialised (option (i) — escalation criterion not triggered, OR escalation triggered and handled per §6).
- [ ] Test count grew from baseline 519 to ≥ 565 (best estimate; actual quoted).
- [ ] All `cargo test --workspace` runs across 7 commits passed; outputs quoted in respective commit messages.
- [ ] Failure-mode catalogue from findings §6: 11 of 14 bugs catalogued as "caught by Phase 9" actually have a Phase 9 test that would detect them. Confirm by inspection at close.
- [ ] JOURNAL.md has a consolidated J-### entry summarising the milestone close, with sub-entries for each of the 7 commits.
- [ ] CLAUDE.md milestone block flipped PLAY → DONE.
- [ ] ROADMAP.md reflects same state.
- [ ] Design doc §15 lists nine shipped phases.
- [ ] `tasks/FEDERATION_STRESS_FOLLOWON.md` exists and is in PENDING state (created at this milestone close per Step 3 of session plan).
- [ ] Client-Side Consequences Audit identified as the next J-081-shape canonical doc (per memory #14); placeholder task file optional but recommended.

**The 4 catalogue bugs NOT caught by Phase 9** (M6, M8, M13 per findings §6) explicitly carry forward to either:
- `federation-stress` follow-on milestone (M6, M8 via deferred compounds C4, C8, C6).
- Client-Side Consequences Audit (M13 — F-1c registry consistency).

This is intentional. The honest framing of milestone close-out names what was and was not proven.

---

## §5 — Coordination with M6 (new)

M6 (new) Phase 2 lands the envelope-level `event_id` on `TransportMessage::Error` per D-070. Phase 9 produces the rejection paths that M6 Phase 2 will wire — specifically the G2 trace events at `f3_reject`, `validation_reject`, and the co-located rejection log at app.rs:1441. M6 Phase 2 will not change the trace events; it adds the wire-layer rejection signal alongside.

**No M6 work in this task.** M6 unblocks at Phase 9 close. The locked Q2 (defer G4 audit-log to M6) is the explicit hand-off point.

---

## §6 — Escalation rules

Beyond CLAUDE.md Rules 1-7 (which always apply), Phase 9 has one escalation rule unique to this task:

**Flake escalation rule (per Q3 lock).** If during any commit in §3, any new Phase 9 integration test exhibits:
1. A `127.0.0.1:0` bind race or "address already in use" failure under `cargo test --workspace` that isn't explained by the test's own logic, OR
2. WS frame-ordering inconsistency where the same test passes in isolation but fails under `--workspace`,

then:
- STOP per Rule 3.
- Report to Joe with: which test, what symptom, what `cargo test --workspace` output (quoted per Rule 2).
- Walk back to option (ii) per Q3 lock — investigate the underlying tokio/WS race shape.
- Do NOT silently add more `#[serial_test::serial]` annotations. Suppressing the diagnostic signal defeats the purpose; Phase 9's deployment stress IS the signal.

This escalation rule is documented here so it's not lost across the multi-commit cadence.

---

## §7 — Cross-references

- **`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY.md`** (COMPLETED v2.0) — the survey task that produced the findings.
- **`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`** (COMPLETED v1.1) — Joe-locked findings; authoritative scope for this task.
- **`tasks/FEDERATION_PROPAGATION_COMPLETION.md`** §3.9 (ACTIVE — flipped to COMPLETED at Commit 7) — original Phase 9 scope at runbook handoff.
- **`tasks/FEDERATION_STRESS_FOLLOWON.md`** (PENDING — created at Commit 7) — deferred compounds C1, C4, C6, C8 + clock injection + parallelism investigation if Q3 escalation didn't fire.
- **`docs/xgen_federation_propagation_design.md`** (ACTIVE v1.0) — canonical design; §15 records nine shipped phases at Commit 7.
- **`docs/xgen_propagation_reliability.md`** (J-081 audit, ARCHIVED) — the audit that motivated the milestone; M13 carries forward to the Client-Side Consequences Audit per its precedent.
- **`DECISIONS.md`** D-065 (honest behaviour over polite behaviour — applied to test results here), D-069 (delegated design discipline + canonical-document rule), D-070 (two events of equal importance), D-071 (subsystem audits precede dependent milestones; Phase 9 survey instantiates D-071 retroactively).
- **CLAUDE.md** — current milestone state, MANDATORY behaviour rules, known-flake state.

---

*End of Phase 9 implementation task file. Implementation starts when Clair picks this up. Milestone closes at Commit 7.*  
