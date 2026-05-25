# Phase 9 Commit 3b-4 — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.2  
> Date: May 2026  
> **Last updated**: 2026-05-24 (J-116 — Track 1 sub-amendment of J-115 SHIPPED — D-078 **second** prospective-catch at Pre-Commit-3b-4 Joe-lock checkpoint #2 (template-API-shape surface vs J-115's contract-surface). **Five-file atomic commit per D-074 (sixteenth instance) + Lock #3 per-commit cadence**. Decision count six per Joe's enumeration (the sixth being the explicit no-amend on DECISIONS.md per D-077 backward-coherence audit — both candidate D-NNN-α "prospective-catch count separation" + candidate D-NNN-β "template-compile-check at runbook authoring" stay flagged-not-promoted per D-069; promotion threshold three instances OR Joe-lock). **Cause:** Clair STOPPED per Rule 3 at Pre-Commit-3b-4 checkpoint #2 (second pass against J-115-amended runbook v1.1) when D-078 production-code verification surfaced that the J-115 amended templates referenced API shapes that don't exist in production: `DispatchOutcome::HeldPending(buffer_state)` (production is a unit variant at `xgen-core/src/node/runtime.rs:88-95`); `DispatchOutcome::Rejected { reason }` struct-destructure syntax (production is `Rejected(String)` tuple variant); `SpaceState::dag()` + `SpaceState::event_count()` methods (don't exist; correct equivalents are `rt.stores[&space_id].contains(&event_id)` at `xgen-core/src/dag/store.rs:48` + `rt.stores[&space_id].len()` at `:52`); `&forged_event.event_id` raw (production `Event::event_id: Option<String>` at `xgen-common/src/wire.rs:332`). **Sibling-but-distinct from J-115's catch shape**: J-115 caught contract intent vs production behaviour mismatch (HeldPending vs Rejected outcome); this catch caught template assertion API vs production type shape mismatch (HeldPending unit variant; Rejected tuple syntax; SpaceState method absence). **D-077-shape framing** — forward-sustainability question (what does production do?) was asked at J-115 amendment authoring, but the backward-coherence question (does the proposed template's API-shape compile against production types?) was not. **Joe locked Reading α** (Track 1 sub-amendment of J-115; runbook v1.1 → v1.2 + findings v1.4 → v1.5; sibling-shape to J-099 / J-109 / J-113 / J-115 canonical-record-amendment-first precedent) over Reading β (Clair latitude + JOURNAL divergence; rejected as no-drift-surface anti-pattern) + Reading γ (hybrid bundle into Clair atom; rejected as defeats D-078's prospective-catch purpose at its second application). **Open question ε resolution**: Option ε.iii locked (defer stronger missing_identity-equals-attacker-uri assertion to its own audit-design-impl arc per D-071; stay narrow at this atom). PendingBuffer does not expose `missing_identity` per-entry as a public accessor — `pending_identity_count() >= 1` + per-event `contains()` is the available F-10 observability surface at deployment level. **Variant 2 (4 families) assertion shape** becomes two-part: (a) `assert!(matches!(outcome, DispatchOutcome::HeldPending))` for the outcome signal; (b) `rt.pending[&space_id].contains(event_id)` + `rt.pending[&space_id].pending_identity_count() >= 1` for the F-10 waiting-on-identity property. Stronger per-event missing_identity inspection would require a new public accessor (Option ε.ii); deferred per D-069 audit-vs-design boundary. **Edits in this update — runbook scope**: §4.5 variants 1/3/4 template full rewrite (tuple destructure; `rt.stores[&space_id]` for DAG-membership; `.as_deref().expect(...)` event_id pattern); §4.5 variant 2 (4 families) template full rewrite (drop `DispatchOutcome::HeldPending(buffer_state)` destructure; two-part assertion via outcome match + PendingBuffer inspection); §4.5 variant 2 (state.federation_add) corrections (`event_count() → len()`; event_id `.as_deref()` pattern; `Accepted { .. }` struct match-arm stays correct); §5.1 C5 sample tuple syntax fix; §4.5 new lock-up note at end naming production-grounded API verification at template-authoring as the J-115-missed discipline gap; §7 intro "Eight" → "Nine sub-sections"; new §7.9 "Template-compile-check at runbook authoring (J-116 amendment)" with catch retrospective + candidate D-NNN-β framing + promotion-watch state; §8 cross-references gain J-116 + candidate D-NNN-β; footer ACTIVE v1.1 → v1.2. **Findings v1.4 → v1.5 in parallel**: §2.4.2 "Uniform assertion shape per test for variant row 2" Rust code block full rewrite to match production; source-of-truth pointers gain `xgen-core/src/dag/pending.rs:491,498` (PendingBuffer accessors) + `xgen-core/src/dag/store.rs:48,52` (EventStore accessors). **Honest longer work over fast shortcuts — tenth recurrence** within Federation Event Propagation milestone scope (Phase 7.5 first through J-115 ninth; this sub-amendment tenth). Framing α locked over Framing β — same recurrence-count semantics J-115 used; each prospective catch that delays milestone closure is its own recurrence regardless of which structural layer (contract vs template-API) the catch surfaces at. **Discipline data point for next sibling milestone runbook author**: when the runbook's §4.5 sample code names a production API by signature, the authoring discipline is to verify that signature against production code (grep enum-shape + grep accessor-method-existence) before locking the sample. J-115 amendment authoring did the contract-grounding (forward-sustainability) correctly but skipped the template-compile-check (backward-coherence); candidate D-NNN-β at one instance flagged-not-promoted; future sibling milestones' runbook §7 should record any catches under this shape and update the cumulative count. **No state transitions**: Phase 9 milestone stays PLAY; Federation Event Propagation milestone stays PLAY; M6 (new) + XGID Retrofit Pass 1 stay PENDING. **PLAY block does NOT flip entry-point file** — Clair's next-active stays Phase 9 Commit 3b-4 against further-further-amended runbook v1.2 + findings v1.5; pickup at Joe-lock checkpoint #2 per runbook §2.3 (re-runs D-078 verification against the corrected templates). Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 + grep guardrail scope discipline. Previous J-115 update content stands authoritative — see header chain below.) Previous J-115 update: 2026-05-24 (J-115 — §4.2 + §4.5 + §4.6 + §4.7 + §8 amendments + new §7.8 + four clerical fixes; version bump v1.0 → v1.1. **Cause: D-078 first prospective-catch at Pre-Commit-3b-4 Joe-lock checkpoint #2** — production-code verification surfaced that variant row 2's `forged_sender_with_resign_*` outcome was `HeldPending` not `Rejected(UnknownSender)`. Track 1 amendment landed pre-implementation. **Five substantive amendments**: (1) §4.2 variant 2 row substantive rewrite (HeldPending for 4 families + Validated-then-ingested for state.federation_add per Phase 7 B3 asymmetry); (2) §4.5 assertion template rewritten for variant 2 (HeldPending destructure + missing_identity check + B3 cell Accepted branch); (3) §4.6 header reframed (Joe-locked at J-113, contract refined at J-115); (4) §4.7 module doc-comment table line + new Phase 7 B3 asymmetry paragraph; (5) new §7.8 sub-section: D-078 first prospective-catch retrospective + prospective-vs-retroactive distinction discipline-notes data point. **Four clerical fixes** (Clair surfaced at checkpoint #2 alongside the main finding): §4.4 module path `xgen-core/src/node.rs` → `xgen-core/src/node/mod.rs`; §4.5 template `.await` removed from sync `dispatch_event`; §5.3 C9 doc-comment line-number `runtime.rs:529-535` → `:864-865`; §5.4 C10 line-numbers `app.rs:1592` → `:1695` + `runtime.rs:680+` → `:911`. **§7 intro bumped** from "Seven sub-sections" to "Eight sub-sections" per new §7.8. **§8 cross-references** updated for findings v1.4 + J-115 + amended line-numbers. **D-078's first prospective-catch instance** — the principle was promoted at J-114 specifically to catch this class of gap before Clair writes code; the very next checkpoint #2 instantiated the prospective shape. Sibling-shape to J-099 / J-109 / J-113 but procedurally distinct: those were retroactive catches; this is the first prospective catch. **No state transitions**: Phase 9 milestone stays PLAY; Federation Event Propagation milestone stays PLAY; M6 (new) + XGID Retrofit Pass 1 stay PENDING. Clair's next-active stays Phase 9 Commit 3b-4 against amended contract; runbook stays the entry-point file. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 discipline. Previous J-114 runbook-authoring content stands authoritative — see J-114 entry below.) 2026-05-24 (J-114 — Runbook authored at the Phase 9 Commit 3b-4 runbook-authoring session following J-113 canonical-record amendment of Scenario 4 enumeration from 6×5=30 → 4×5=20 tests. Sibling-in-shape to `tasks/FEDERATION_TOPOSORT_IMPL.md` (COMPLETED v1.2) and `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` (COMPLETED v1.2) — same eight-section shape; same five-Joe-lock-checkpoint posture; same §7 discipline-notes inclusion with precedent-departure self-defense at §7.1. Five Joe-locks carried into the runbook from authoring session: Lock 1 single-commit-with-split-trigger-discipline shape (3b-4 frames as single commit with three split triggers documented; organic split at Joe-lock checkpoint #4 if any trigger fires — sibling-shape to J-111 retrospective 3b-3-pre + 3b-3 split pattern); Lock 2 D-078 promoted to DECISIONS.md (production-grounded test enumeration discipline; three-instance threshold met at J-099/J-109/J-113; sibling-shape to D-076 v1 → v1.1 promotion at J-097 design close); Lock 3 verification rigour 5 isolated + 3 workspace = 8 green runs minimum at the milestone-bearing commit (sibling-shape to topo-sort J-101 verification rigour); Lock 4 family-uniformity + variant-uniformity structural properties as load-bearing per-family assertions (uniform shape across the 20 Scenario 4 tests; same property for C5); Lock 5 §7 discipline-notes section with seven sub-sections (§7.1–§7.7). Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 discipline.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Framing, reading order, latitude, pre-existing flakes carry-forward

### §1.1 What this runbook is

This runbook is the implementation contract between Chat Claude + Joe (who authored the contract amendments at J-113 + the runbook-authoring locks at this session J-114) and Clair (who ships the code). The contract for Scenario 4's 20-test enumeration is locked at `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` v1.3 §2.4 §E + §2.4.1 production-code-verification walkthrough. The contracts for compounds C5, C7, C9, C10 are locked at the same findings doc §3.5, §3.7, §3.9, §3.10 respectively. This runbook does not re-litigate the locked contracts; it makes them concrete at file-and-line level so Clair has minimal ambiguity at commit-authoring time.

Five runbook-structural Joe-locks were added at this runbook-authoring session open (the locks above are recorded in the header for header-chain continuity; this body restates them with reasoning):

1. **Single-commit-with-split-trigger-discipline shape.** Commit 3b-4 frames as a single atomic commit landing all five test families (Scenario 4 + C5 + C7 + C9 + C10). At Joe-lock checkpoint #4 (pre-Commit-3b-4 implementation), three split triggers are walked:
   - **Trigger (a) — non-existent production contract (J-113 shape).** If pre-implementation production-code verification (per D-078 below) surfaces a family asserting against a contract that doesn't exist, that family triggers Track-1 canonical-record amendment FIRST (the J-113 / J-109 / J-099 pattern) before Clair picks up. Sibling-shape to the three prior instances of this pattern.
   - **Trigger (b) — harness extension beyond CounterLayer (J-111 shape).** If pre-implementation surfacing reveals a family needs new harness primitives (counter expansion, layer additions, new in-process accessors), the harness-extension work splits into a pre-commit (3b-4-pre), Compound C-family-X commits land separately, sibling-shape to 3b-3-pre + 3b-3.
   - **Trigger (c) — family-boundary size split.** If Scenario 4 alone exceeds ~600 lines, Scenario 4 splits to its own commit (3b-4-sc4) and compounds land in 3b-4-compounds. If any single compound exceeds ~400 lines, that compound gets its own commit. Natural fault line is family-boundary, not arbitrary line count — honest about where splits would actually happen.
2. **D-078 promoted to DECISIONS.md in this atomic commit.** "Production-grounded test enumeration" — at every test enumeration (named list of test cases that will become a regression lock), the production reject-path inventory MUST be confirmed against current code BEFORE the enumeration is Joe-locked, not retroactively after implementation surfaces drift. Three-instance threshold met (J-099 audit-doc §11 + J-109 survey §2.6 + J-113 survey §2.4); pattern durable per the no-drift-surface discipline family's promotion threshold sibling-shape to D-076 v1 → v1.1. Sibling-shape to D-077 (bidirectional sustainability at silent-discard sites): D-077 is the meta-layer principle; D-078 is its protocol-test-layer sibling. Application surface for Commit 3b-4: at Joe-lock checkpoint #4, each of the five test family enumerations gets the same prospective verification Scenario 4 retroactively got at J-113.
3. **Verification rigour 5 isolated + 3 workspace = 8 green runs minimum** at the milestone-bearing commit (where the test count delta lands). Sibling-shape to topo-sort J-101 Commit 3 verification rigour; sibling-shape to persistence-amendment J-108 verification rigour. If Commit 3b-4 splits per Trigger (a)/(b)/(c) above, the verification rigour applies at the milestone-bearing sub-commit (the one with the largest test-count delta, typically the main test-shipping commit not the pre-commit infrastructure).
4. **Family-uniformity + variant-uniformity as load-bearing assertions** (per findings §2.4.1's two structural properties). For Scenario 4: same validation pipeline regardless of event family (family-uniformity); all 4 variants reject via F-4's same dispatch shape (variant-uniformity within a family). For C5: per-event outcome independence (variant of family-uniformity at scale). These are the regression locks that distinguish "the tests pass" from "the tests prove F-4 is correct"; the runbook §4 + §5 reference these properties explicitly so Clair's implementation can target them.
5. **§7 discipline-notes section** with seven sub-sections (§7.1–§7.7) per the topo-sort + persistence-amendment runbook precedent; precedent-departure self-defense at §7.1.

Three pre-draft code-trace findings from this runbook-authoring session shaped §4/§5/§6 scope:

- **Finding A — `xgen-core` test directory organisation.** Existing pattern: unit tests live in `#[cfg(test)] mod tests` within their source file (e.g., `xgen-core/src/node/runtime.rs::tests` at line 1486+ for NodeRuntime). Integration tests at `xgen-core/tests/` (directory at crate-root for cross-module integration). No `xgen-core/src/tests/` or `xgen-core/src/node/tests/` directories exist. Phase 9 §3 Commit 6 paragraph mentions both candidate locations as "Clair chooses." Runbook §4.4 + §5.1 + §5.2 + §5.3 + §5.4 lock the choice: **NEW directory `xgen-core/src/node/tests/`** mod alongside `runtime.rs`, sibling-shape to `xgen-node/src/tests/` mod structure. Five new test files (one per family) under this new mod. Rationale at §4.4.
- **Finding B — `ExchangeError` Display strings.** Findings §2.4.1 names eight ExchangeError variants by name and Step number. Code at `xgen-core/src/message/exchange.rs:46-87` confirms the eight variants. Display impl at `:90-110` produces the exact `step N: ...` strings. **Substring matching, not exact-equality matching** is the locked assertion shape for the `reason` field (the strings carry the variant payload — DagError(String) contains the specific violation; EventIdMismatch wraps an inner payload; substring matching is robust across payload variations). §4.5 verbatim assertion template.
- **Finding C — `DispatchOutcome` enum reject shape.** `DispatchOutcome::Rejected(String)` — **tuple variant** with single String payload at `xgen-core/src/node/runtime.rs:88-95`. The String payload carries the ExchangeError's Display string per `dispatch_event`'s F-4 reject arm. Assertion pattern: `assert!(matches!(outcome, DispatchOutcome::Rejected(_)))` + `let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() }` + `assert!(reason.contains(EXPECTED_SUBSTRING))`. Verbatim shape locked at §4.5.

### §1.2 Reading order for Clair

Per CLAUDE.md Rule 0 (session-open reading sequence):

1. CLAUDE.md PLAY block (Phase 9 Commit 3b-4 RESUMES — runbook ships at this commit; Clair pickup next session)
2. JOURNAL.md latest entry (J-116 runbook v1.1→v1.2 template-API correction; previously J-115 contract amendment, J-114 runbook-authoring)
3. This runbook §1 → §2 → §3 → §4 → §5 → §6 → §7 → §8
4. `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` v1.3 §2.4 + §2.4.1 (Scenario 4 contract) + §3.5 (C5) + §3.7 (C7) + §3.9 (C9) + §3.10 (C10)
5. `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3 Commit 6 (Phase 9 task framing; defers to findings for substantive contract)
6. DECISIONS.md D-078 (the principle this runbook applies prospectively to all five families)

Then for each Joe-lock checkpoint:

- **Pre-checkpoint #4 (this runbook's load-bearing checkpoint)**: this runbook §3 (commit shape) + §4 (Scenario 4 detail) + §5 (compounds detail) — Clair surfaces the production-code-verification results for each of the five families per D-078, Joe approves the enumeration or amends.
- **Post-commit verification**: this runbook §6 (verification rigour) — 5 isolated + 3 workspace runs minimum.
- **Milestone-close**: this runbook §6.7 + §6.8 anti-drift guardrails — `grep -rn 'J-NNN'` returns zero in canonical sources before commit.

### §1.3 Latitude — what Clair decides

The five Joe-lock checkpoints frame where Clair surfaces to Joe vs ships from runbook directly. Outside those checkpoints, Clair has latitude on:

- Exact test-helper function names (provided the locked test-name pins at §4.3 + §5.x are honoured)
- Per-test module-level doc-comment wording (provided the locked content at §4.6 + §5.x is honoured)
- Local variable naming + small refactoring inside test bodies
- Choice between match-arm vs `if let` for short outcome destructuring
- Order of file creation within a commit (e.g., Scenario 4 first vs compounds first, as long as the commit is green at close)
- Whether to construct forged events via existing builders (e.g., `build_message_text_event`) or via direct struct literals (Clair's call based on what produces the cleanest forgery; doc-comment names the choice)

The locked items at each checkpoint name the Joe-decision content explicitly. Everything else is Clair's call.

### §1.4 Pre-existing flakes carry-forward

Two pre-existing flakes carry forward under workspace parallelism (sibling-shape to all prior Phase 9 commits + persistence-amendment milestone close):

1. **Precedence env-var race** — `xgen-node` precedence test family occasionally interleaves env-var reads across tests. Did NOT fire during J-110, J-111, J-112 verifications.
2. **`reconnect_with_existing_tip_small_delta_delivered`** — `xgen-node` federation reconnect test occasionally races on delta size measurement. Did NOT fire during J-110, J-111, J-112 verifications.

If either flake fires during Commit 3b-4 verification runs, the verification rigour pattern (5 isolated + 3 workspace; if any fails, re-run; if pattern persists, escalate per Rule 3) applies. Neither flake is implicated by Commit 3b-4's test surface (Scenario 4 + C5 + C7 + C9 + C10 are NodeRuntime-level in xgen-core; the flakes live in xgen-node).

---

## §2 — Sequence overview

### §2.1 Single-commit framing + sub-commits if triggered

This runbook frames Commit 3b-4 as a single atomic commit per Joe-lock Lock 1. The five families ship together under a single commit message + JOURNAL entry. The expected file count is ~10 files:

| # | File | Change |
|---|---|---|
| 1 | `xgen-core/src/node/tests/mod.rs` | NEW (declares the five test modules below) |
| 2 | `xgen-core/src/node/tests/phase9_validation_asymmetry.rs` | NEW (Scenario 4 — 20 tests) |
| 3 | `xgen-core/src/node/tests/phase9_compound_c5_validation_under_load.rs` | NEW (C5 — 1 test, ~100 events) |
| 4 | `xgen-core/src/node/tests/phase9_compound_c7_pagination_boundary.rs` | NEW (C7 — 4 tests, N=999/1000/1001/2000) |
| 5 | `xgen-core/src/node/tests/phase9_compound_c9_drain_time_hazard.rs` | NEW (C9 — 1 test) |
| 6 | `xgen-core/src/node/tests/phase9_compound_c10_identity_lock_contention.rs` | NEW (C10 — 1 test) |
| 7 | `xgen-core/src/node.rs` or `xgen-core/src/lib.rs` | EDIT (add `#[cfg(test)] mod tests;` mod declaration — see §4.4 for the exact spot) |
| 8 | `JOURNAL.md` | EDIT (J-115 entry + header chain) |
| 9 | `CLAUDE.md` | EDIT (PLAY block flip from "Commit 3b-4 RESUMES" to "Commit 3b-5 milestone close RESUMES") |
| 10 | `docs/ROADMAP.md` | EDIT (v1.29 → v1.30 + Past entry + header chain) |

**If a split trigger fires at Joe-lock checkpoint #4** (per Lock 1 (a)/(b)/(c) above), the runbook shape morphs as described in §2.2 below. The single-commit framing is the default; the split is the contingency.

### §2.2 Split-trigger contingencies

#### §2.2.1 Trigger (a) — non-existent production contract (J-113 shape)

If pre-implementation production-code verification at Joe-lock checkpoint #4 surfaces that a family (any of Scenario 4 OR C5/C7/C9/C10) asserts against a contract that doesn't exist in the current production code:

1. **Halt Commit 3b-4 implementation.**
2. **Open Track 1 canonical-record amendment session** (Chat Claude + Joe). The amendment surface is the appropriate findings sub-section (§2.4 for Sc4 already amended at J-113; §3.5 / §3.7 / §3.9 / §3.10 for compounds). Sibling-shape to J-113's Reading B procedure: amend canonical source FIRST, then Clair picks up against amended contract.
3. **D-078 application catches this prospectively rather than retroactively** — that's the whole point of D-078's promotion. If a Trigger (a) fires now, the meta-finding's value is demonstrated by catching the gap before code is written, not after.
4. **Sub-commit shape**: Track-1 amendment commit lands first (atomic per D-074); Clair picks up Commit 3b-4 against amended contract.

#### §2.2.2 Trigger (b) — harness extension beyond CounterLayer (J-111 shape)

If pre-implementation surfacing reveals a family needs new harness primitives (e.g., a new tracing Layer beyond CounterLayer + LogBufferLayer, a new in-process accessor on InProcessNode or NodeRuntime, a new in-test helper for forgery construction):

1. **Halt Commit 3b-4 main implementation.**
2. **Split into Commit 3b-4-pre + Commit 3b-4.** 3b-4-pre lands the harness extension (sibling-shape to J-111's 3b-3-pre). 3b-4 lands the tests against the extended harness.
3. **Note**: Commit 3b-4's tests are NodeRuntime-level (no TCP, no federation_session.rs). The CounterLayer wired through `apply_federation_push` (xgen-node-layer) is likely irrelevant to Commit 3b-4. New harness primitives, if needed, are likely on `NodeRuntime` directly (e.g., a fixture for constructing forged events at the xgen-core layer).
4. **Pre-commit verification**: 8 green runs at the pre-commit (sibling-shape to J-111's 3-unit-test verification at 3b-3-pre).

#### §2.2.3 Trigger (c) — family-boundary size split

If at implementation time Scenario 4 alone exceeds ~600 lines (20 tests × ~30 lines/test ≈ 600 lines is the rough estimate; actual may be lower if shared helpers extract cleanly, or higher if forgery construction is verbose):

1. **Split Scenario 4 from compounds.** Commit 3b-4-sc4 lands the 20 Scenario 4 tests + helpers. Commit 3b-4-compounds lands C5 + C7 + C9 + C10.
2. Sibling-shape: J-111 + J-112's two-commit Compound C2 arc; sibling-shape: persistence-amendment Commit 2 + Commit 2a layered atom.

If any single compound exceeds ~400 lines (C7 most likely given the 4 sub-cases with potentially-divergent N-event-builders; C10 second-most-likely given concurrency setup):

1. That compound gets its own commit.
2. Other compounds stay grouped.

#### §2.2.4 Order of trigger evaluation at checkpoint #4

If multiple triggers fire (e.g., Trigger (a) on C9 AND Trigger (c) on Scenario 4), evaluate triggers in this order to determine commit shape:

1. **(a) first** — canonical-record amendments precede everything else; Track 1 amendments are atomic and ship before Clair picks up.
2. **(b) second** — harness extension precedes test implementation; 3b-4-pre lands before 3b-4.
3. **(c) third** — family-boundary splits are within-implementation scope decisions; they apply to the test-shipping commit(s) after (a) and (b) are resolved.

### §2.3 Five Joe-lock checkpoints

Per the sibling-shape topo-sort + persistence-amendment runbook precedent (J-098 + J-106), five Joe-lock checkpoints frame where Clair surfaces to Joe vs ships from runbook directly:

#### Checkpoint #1 — Post-runbook-read drift check

**Trigger**: Clair has read this runbook + reading-order materials (§1.2) and is ready to start implementation. Before any code is written.

**Joe-lock content**: Does the runbook match Clair's understanding of the contract? Specifically:
- Scenario 4's 20-test enumeration per findings §2.4.1
- C5/C7/C9/C10 contracts per findings §3.5/§3.7/§3.9/§3.10
- The single-commit + split-trigger framing per Lock 1
- D-078's prospective application per Lock 2

**Surface to Joe if**: Any reading-order ambiguity, any contract clarity gap, any split-trigger criteria question.

**Fast-path**: If Clair surfaces no questions and the implementation plan is clean, checkpoint #1 closes silently.

#### Checkpoint #2 — Pre-implementation production-code verification per D-078

**Trigger**: Clair has read the runbook + contracts. Before writing test code.

**Joe-lock content** (this is the load-bearing application of D-078):
- For Scenario 4 (already done at J-113): production code at `xgen-core/src/message/exchange.rs:46-87` confirms the 4 ExchangeError variants matched by the 4 forgery variants. **D-078's retroactive trigger; serves as the verification template for compounds.**
- For C5: confirm the validation pipeline's per-event isolation property holds at `dispatch_event`'s F-4 reject arm. Specifically: does any rejection branch mutate state that subsequent events could observe? Quick code-trace at `runtime.rs:317+`. **If a state-leak surface is found, Trigger (a) fires.**
- For C7: confirm `compute_federation_delta_for_space` + `continue_from` pagination logic at `xgen-node/src/fanout.rs` (the pagination is xgen-node-side; C7 is NodeRuntime-level per Phase 9 §3 Commit 6 framing — Clair should verify the assertion target matches: does C7 assert against `compute_federation_delta` directly or against a NodeRuntime-layer wrapper? See §5.3 for the lock.)
- For C9: confirm the drain-time approximation hazard at `xgen-core/src/node/runtime.rs:864-865` matches findings §3.9. The doc-comment at the production site is the canonical contract. **C9 is unique among compounds in that production explicitly accepts a hazard; C9 tests prove the bound.**
- For C10: confirm `handle_identity_replicate_msg` + `drain_pending_by_identity` lock pattern at `xgen-node/src/app.rs:1695` + `xgen-core/src/node/runtime.rs:911` matches findings §3.10. The lock contention surface is xgen-node-side; the drain is xgen-core-side. **C10 may be the most production-coupled compound; verify carefully.**

**Surface to Joe if**: Any family's contract doesn't match production code. **Trigger (a) fires here, not later.**

**Fast-path**: If all five families verify cleanly, checkpoint #2 closes silently; Clair proceeds to implementation.

#### Checkpoint #3 — Pre-implementation harness-extension assessment

**Trigger**: Checkpoint #2 closed. Before writing test code.

**Joe-lock content**: Does any family need new harness primitives beyond what already exists in `xgen-core` test fixtures? Specifically:
- Forged-event constructors (for Scenario 4 — 4 variants × 5 families = 20 distinct construction patterns; if a shared helper extracts cleanly, no harness extension. If not, a forgery-builder helper may be needed).
- NodeRuntime in-process fixtures for C5/C7/C9/C10 (these tests don't use the xgen-node InProcessNode harness; they construct `NodeRuntime` directly at `xgen-core`'s API layer — verify the existing fixtures cover this).
- Concurrency helpers for C10 (3 concurrent peers × 3 concurrent identity-replicates — Clair's call on whether `tokio::spawn` + `JoinSet` is enough, or whether a dedicated test fixture is warranted).

**Surface to Joe if**: A harness extension exceeds ~50 lines OR introduces a new public API on NodeRuntime/xgen-core. **Trigger (b) fires here, not later.**

**Fast-path**: If all harness needs are within-test-file scope (no library-API additions), checkpoint #3 closes silently.

#### Checkpoint #4 — The load-bearing checkpoint (D-078 application surface)

**Trigger**: Checkpoints #1 + #2 + #3 closed. Before writing test code.

**Joe-lock content**: Final Joe-approval of the test-case enumeration BY NAME per D-078. Clair surfaces:
- **Scenario 4 — 20 test names** (already locked at J-113 findings §2.4.1; Clair confirms the names match).
- **C5 — test name(s)** (likely 1 test; the enumeration is "100 mixed valid+forged events"; Clair confirms the 100-event composition: what mix of variant/family pairs?).
- **C7 — 4 test names** (N=999, N=1000, N=1001, N=2000 per findings §3.7).
- **C9 — 1 test name** + the bound assertion target (30s F-4a window).
- **C10 — 1 test name** + the concurrency shape (3 peers × 3 identity-replicates).

**Surface to Joe**: ALWAYS at this checkpoint. The 20-name Scenario 4 enumeration was Joe-locked at J-113; the runbook-authoring session J-114 confirms that lock stands. The compound enumerations are surfaced here for the first time at name-level — Joe-lock by name explicitly. **This is the largest scope of any Joe-lock checkpoint in the 3b arc.**

**No fast-path.** Checkpoint #4 always triggers; Joe approves the enumeration explicitly.

#### Checkpoint #5 — Post-implementation pre-verification

**Trigger**: Tests written; ready for verification.

**Joe-lock content**: Quick Clair surface of any implementation surprises that didn't show at checkpoints #2/#3/#4. Specifically:
- Test that failed to compile or required a contract amendment (Trigger (a) post-hoc; should be rare given checkpoint #2's coverage).
- Harness extension that exceeded the checkpoint #3 estimate (Trigger (b) post-hoc).
- Family that exceeded its line budget (Trigger (c) post-hoc — checkpoint #5 is the last chance to split before commit).

**Surface to Joe if**: Any of the above. Otherwise proceed to verification (5 isolated + 3 workspace = 8 green runs minimum per Lock 3).

**Fast-path**: If implementation was clean, checkpoint #5 closes silently; Clair runs the 8 verification runs and ships.

---

## §3 — Commit 1 (single commit, default) or Commit 1 + Commit 1-pre / Commit 1a / etc. (split contingencies)

This section is the runbook's spine. It describes the single-commit shape that lands all five families. If Trigger (a)/(b)/(c) fires, the §2.2 contingency morphs apply.

### §3.1 Working tree before Commit 3b-4

The state when Clair picks up:
- `git status` shows clean working tree.
- Last commit is the J-113 amendment (atomic five-file commit per D-074 thirteenth instance).
- `cargo test --workspace` baseline: **600 tests passing** (post-J-112). Pre-existing flakes did NOT fire at J-112 verification.

### §3.2 New files Clair creates

| Path | Purpose | Estimated lines |
|---|---|---|
| `xgen-core/src/node/tests/mod.rs` | Module declarations | ~10 |
| `xgen-core/src/node/tests/phase9_validation_asymmetry.rs` | Scenario 4 — 20 tests | ~500-700 |
| `xgen-core/src/node/tests/phase9_compound_c5_validation_under_load.rs` | C5 — 1 test (100 events) | ~150-250 |
| `xgen-core/src/node/tests/phase9_compound_c7_pagination_boundary.rs` | C7 — 4 tests | ~250-400 |
| `xgen-core/src/node/tests/phase9_compound_c9_drain_time_hazard.rs` | C9 — 1 test | ~120-200 |
| `xgen-core/src/node/tests/phase9_compound_c10_identity_lock_contention.rs` | C10 — 1 test | ~200-300 |

Estimated total ~1,200-1,850 lines. Triggers (c) split-discussion: Scenario 4 alone runs ~500-700 lines (under the ~600 split threshold in the lower half of the range; right at the threshold in the upper half — borderline). C7 + C10 sit at ~400 (right at the family-split threshold for compounds). **Clair surfaces line estimate at checkpoint #5 if any family is in split range.**

### §3.3 Modified files Clair creates

| Path | Change |
|---|---|
| `xgen-core/src/node.rs` (or wherever `node` mod is declared inside xgen-core) | Add `#[cfg(test)] mod tests;` line. See §4.4 for the locked spot. |
| `JOURNAL.md` | J-115 entry + header chain |
| `CLAUDE.md` | PLAY block flip from "Commit 3b-4 RESUMES" to "Commit 3b-5 milestone close RESUMES" + header chain |
| `docs/ROADMAP.md` | v1.29 → v1.30 + Past entry + header chain |

### §3.4 Commit message shape

```
Phase 9 Commit 3b-4 — Scenario 4 validation asymmetry (20 tests) + compounds C5/C7/C9/C10 against amended J-113 contract; per-test family-uniformity + variant-uniformity regression lock per findings §2.4.1; ~XX-file atomic commit per D-074 + Lock #3 per-commit cadence; J-115 ships
```

Where `XX` reflects the actual file count (10 if single commit; varies if split per §2.2).

### §3.5 What Clair does NOT do at Commit 3b-4

- **Does NOT touch production code** unless checkpoint #2 surfaces Trigger (a) and a Track-1 amendment ships first. Commit 3b-4 is test-only; production-code changes are out-of-scope.
- **Does NOT promote the Gap G6 production gap** (timestamp-bound validation per findings §4.6). Gap G6 stays flagged-not-promoted per D-071 surface-driven application; promotion happens at a future audit-design-impl arc if Joe locks it.
- **Does NOT modify existing tests.** All 600 baseline tests stay as-is; the 20+1+4+1+1 = 27 new tests add cleanly.
- **Does NOT add new public APIs to NodeRuntime** unless checkpoint #3 surfaces Trigger (b). Test-internal helpers stay test-internal.

### §3.6 Mid-implementation discipline

If during implementation Clair surfaces a previously-unanticipated finding (sibling-shape to J-110's cross-crate trace-event assertion gap, J-111's tracing-test set_global_default exhaustion, J-099's framing-gap surface):

1. **Halt the in-flight test.** Don't paper over the finding.
2. **Surface to Joe** via Rule 3 + Lock #2 honesty discipline.
3. **Walk options.** Findings of this shape have produced D-077, Rule 0, D-078, and the layered-B3 pattern; the project's response to discipline surfaces is to walk them carefully.
4. **Resume implementation against amended understanding.**

This is the "honest longer work over fast shortcuts" discipline at the implementation layer — count inherited at eighth from J-104; not incremented at within-milestone surfaces unless a sub-milestone opens.

---

## §4 — Scenario 4 — Validation asymmetry (20 tests)

This is the largest single test family in Commit 3b-4. Sub-sections walk the file structure, the per-test pattern, the assertion template, and the file location.

### §4.1 Narrow scope — what stays out

Per findings §2.4.1's production reject path inventory + the J-113 amendment:

- **NOT covered**: timestamp-bound forgery (`future-timestamp` / `past-timestamp`) — production has no timestamp variant; Gap G6 flagged-not-promoted.
- **NOT covered**: `NotASpaceMember` / `NotARoomMember(String)` — these are step-11 sub-checks that surface for valid signers with no Space/Room membership; outside F-4 asymmetry scope, covered by permission-policy tests + C5 (load).
- **NOT covered**: `PermissionDenied(String)` (step 13) — event-family-specific permission checks tested at permission-policy layer.
- **NOT covered**: `HeldPending(Vec<String>)` (step 9) — defers rather than rejects; outside Scenario 4's scope (it's Scenario 5/6 territory).
- **NOT covered**: wire-decoding / serialization errors — those are transport-layer tests, not F-4 tests.

The 4 variants × 5 families = 20 tests exactly cover the F-4 reject paths that distinguish family-by-family asymmetry regressions. The 4 variants map 1:1 to ExchangeError variants {SignatureFailure, UnknownSender, EventIdMismatch, DagError}.

### §4.2 The 4 forgery variants — production-grounded mapping

**Variant row 2 (`forged_sender_with_resign_*`) amended at J-115** against post-Phase-6 F-10 reality; see findings v1.4 §2.4.2 for the full walkthrough. The other three variants retain the J-113 mapping.

| Variant | Production outcome | Step | Notes |
|---|---|---|---|
| `bad_signature_*` | `Rejected(SignatureFailure)` | 12 | Substring: `"step 12: signature verification failed"` |
| `forged_sender_with_resign_*` | **`HeldPending { missing_identity: Some(attacker_uri) }` for 4 families** + **`Validated`-then-ingested for state.federation_add** | 11 (F-10) / B3 skip | Phase 6 F-10 amendment (J-087) at exchange.rs:626-632 + Phase 7 B3 asymmetric cell (J-088). See findings v1.4 §2.4.2 for assertion shape per family |
| `mutated_event_id_*` | `Rejected(EventIdMismatch)` | 8 | Substring: `"step 8: event_id does not match canonical content hash"` |
| `malformed_prev_events_*` | `Rejected(DagError(String))` | 10 | Substring: `"step 10: DAG structural violation"` |

**Substring matching** (not exact-equality) is the locked assertion shape for variants 1, 3, 4 — the DagError variant wraps a String payload; the substring matches the prefix regardless of payload. Variant 2 uses destructure-and-field-check shape (HeldPending + missing_identity) for 4 families; Accepted-and-ingestion-check for state.federation_add. See §4.5 verbatim template.

### §4.3 The 5 event families

| Family | Production constructor or struct |
|---|---|
| `message.text` | `MessageText` event struct; constructor via `Event::new` or test helper |
| `membership.join` | `MembershipJoin` event struct |
| `membership.kick` | `MembershipKick` event struct |
| `state.federation_add` | `StateFederationAdd` event struct — **load-bearing for Phase 7.5 §5 narrowness** (F-3 skips this family; F-4 still catches forgery) |
| `state.room_create` | `StateRoomCreate` event struct |

The five families cover: message-layer (text), membership-layer (join + kick), state-federation-layer (federation_add — the Phase 7.5 §5 skip-set witness), state-room-layer (room_create — DAG-root-with-prev-events witness per D-076 v1.1).

### §4.4 File location — `xgen-core/src/node/tests/`

**Locked location**: `xgen-core/src/node/tests/phase9_validation_asymmetry.rs` + sibling files for compounds.

**Rationale per Finding A above**:
- Existing xgen-core test structure puts unit tests in `mod tests` within source files (e.g., `runtime.rs::tests` at line 1486+).
- Phase 9 tests are too long to embed in source files (Sc4 alone is ~500-700 lines; the runtime.rs mod tests would balloon to ~2000+ lines).
- Integration tests at `xgen-core/tests/` (crate-root) are cross-module integration; Phase 9 tests are NodeRuntime-API-level, single-module scope — they belong adjacent to the NodeRuntime source, not at the crate root.
- New `xgen-core/src/node/tests/` mod is sibling-shape to `xgen-node/src/tests/` (where Phase 9 deployment-level tests live).

**Module declaration**: at the bottom of `xgen-core/src/node/mod.rs` (the `node` module is a directory module per J-115 clerical fix; original v1.0 referenced `node.rs` which doesn't exist). Add:

```rust
#[cfg(test)]
mod tests;
```

The `tests` mod expands to `xgen-core/src/node/tests/mod.rs`, which declares the five test files as sub-modules.

### §4.5 Verbatim assertion template

Each Scenario 4 test follows a variant-specific template. **The shape is locked** — Clair's latitude is variable naming, comment wording, and forgery-construction approach. The assertion steps are the regression lock.

**Template for variants 1, 3, 4** (`bad_signature_*`, `mutated_event_id_*`, `malformed_prev_events_*` — the Rejected variants). Production-grounded shape per J-116 amendment:

```rust
#[tokio::test(flavor = "current_thread")]
async fn <VARIANT>_<FAMILY>() {
    // === SETUP ===
    // Construct NodeRuntime with Space S + Alice's Identity registered + Alice as Space member.
    let (mut rt, alice, space_id, peer_id) = setup_runtime_with_alice_in_space().await;
    let baseline_count = rt.stores[&space_id].len();

    // === FORGE ===
    // Construct event of family <FAMILY> with forgery <VARIANT> applied.
    let forged_event = forge_<VARIANT>_<FAMILY>(&alice, &space_id).await;
    let forged_event_id = forged_event
        .event_id
        .as_deref()
        .expect("event_id must be set post-canonicalisation")
        .to_string();

    // === DISPATCH ===
    // dispatch_event is a sync function on NodeRuntime; no .await per J-115 clerical fix.
    let outcome = rt.dispatch_event(
        forged_event.clone(),
        EventOrigin::ReceivedViaFederation,
        Some(peer_id.as_str()),
    );

    // === ASSERT ===
    // (1) Outcome is Rejected — production DispatchOutcome::Rejected is a tuple variant
    //     (single String payload), NOT a struct variant. Tuple destructure per J-116
    //     amendment (J-115 template used struct { reason } syntax that won't compile
    //     against xgen-core/src/node/runtime.rs:88-95).
    assert!(
        matches!(outcome, DispatchOutcome::Rejected(_)),
        "expected Rejected, got {:?}",
        outcome
    );

    // (2) reason matches the ExchangeError variant substring
    let DispatchOutcome::Rejected(reason) = outcome else {
        unreachable!("guarded above")
    };
    assert!(
        reason.contains("<EXPECTED_SUBSTRING_PER_§4.2>"),
        "expected reason to contain '<EXPECTED_SUBSTRING_PER_§4.2>', got '{}'",
        reason
    );

    // (3) Event did NOT land in Space's event store.
    //     Production API: EventStore::contains(&str) -> bool at xgen-core/src/dag/store.rs:48.
    //     NodeRuntime exposes `pub stores: HashMap<String, EventStore>` at runtime.rs:126-137;
    //     SpaceState has neither a `dag()` method nor a `contains_event()` method.
    assert!(
        !rt.stores[&space_id].contains(&forged_event_id),
        "forged event {} should not have landed in DAG",
        forged_event_id
    );

    // (4) Event count unchanged from pre-dispatch baseline.
    //     Production API: EventStore::len() -> usize at xgen-core/src/dag/store.rs:52.
    //     SpaceState has no `event_count()` method.
    assert_eq!(
        rt.stores[&space_id].len(),
        baseline_count,
        "event store length should not have incremented on rejection"
    );
}
```

**Template for variant 2 (`forged_sender_with_resign_*`) — J-116 amended shape**. The 4 families produce HeldPending (unit-variant outcome signal + PendingBuffer inspection for the F-10 waiting-on-identity property); state.federation_add is the asymmetric cell producing Accepted-then-ingest per Phase 7 B3 (see findings v1.5 §2.4.2):

```rust
// For 4 families: message_text, membership_join, membership_kick, state_room_create
#[tokio::test(flavor = "current_thread")]
async fn forged_sender_with_resign_<FAMILY>() {
    let (mut rt, alice, space_id, peer_id) = setup_runtime_with_alice_in_space().await;

    // Forge with attacker key NOT in IdentityRegistry; re-sign with attacker's own key.
    let attacker_key = generate_attacker_keypair();
    let attacker_uri = pubkey_uri(&attacker_key);
    let forged_event = forge_resigned_<FAMILY>(&attacker_key, &space_id).await;
    let forged_event_id = forged_event
        .event_id
        .as_deref()
        .expect("event_id must be set post-canonicalisation")
        .to_string();

    let baseline_store_len = rt.stores[&space_id].len();
    let baseline_pending_identity = rt
        .pending
        .get(&space_id)
        .map(|buf| buf.pending_identity_count())
        .unwrap_or(0);

    let outcome = rt.dispatch_event(
        forged_event.clone(),
        EventOrigin::ReceivedViaFederation,
        Some(peer_id.as_str()),
    );

    // (a) Outcome signal — production DispatchOutcome::HeldPending is a UNIT variant
    //     at xgen-core/src/node/runtime.rs:88-95. The F-10 missing_identity info is
    //     stored in the PendingBuffer at runtime.rs:611-621 inside the F-4 unified
    //     core, NOT carried on the DispatchOutcome itself. J-115 amendment template
    //     used `DispatchOutcome::HeldPending(buffer_state)` destructure that won't
    //     compile; J-116 corrects to unit-variant match + PendingBuffer inspection.
    assert!(
        matches!(outcome, DispatchOutcome::HeldPending),
        "expected HeldPending (F-10 unknown-signer), got {:?}",
        outcome
    );

    // (b) PendingBuffer inspection — F-10 waiting-on-identity property.
    //     APIs: PendingBuffer::contains(&str) -> bool at xgen-core/src/dag/pending.rs:491;
    //           PendingBuffer::pending_identity_count() -> usize at :498.
    //     NodeRuntime exposes `pub pending: HashMap<String, PendingBuffer>` at
    //     runtime.rs:126-137. The two-part assertion proves: (i) the forged event
    //     was buffered (not silently dropped); (ii) the F-10 identity-waiting
    //     counter incremented (not just predecessor-waiting).
    //
    //     Stronger missing_identity == Some(attacker_uri) assertion deferred per
    //     Option ε.iii lock at J-116 — PendingBuffer does not expose per-entry
    //     missing_identity as a public accessor today. Promoting that accessor
    //     would be its own audit-design-impl arc per D-071.
    let buf = rt.pending.get(&space_id)
        .expect("PendingBuffer must exist for the Space after F-10 buffering");
    assert!(
        buf.contains(&forged_event_id),
        "forged event {} must be in PendingBuffer after F-10 HeldPending",
        forged_event_id
    );
    assert!(
        buf.pending_identity_count() >= baseline_pending_identity + 1,
        "pending_identity_count must have incremented by at least 1 (was {}, now {})",
        baseline_pending_identity,
        buf.pending_identity_count()
    );

    // (c) Event did NOT land in DAG — still buffered, not ingested.
    assert!(
        !rt.stores[&space_id].contains(&forged_event_id),
        "forged event {} must not be in EventStore (HeldPending != Accepted)",
        forged_event_id
    );
    assert_eq!(
        rt.stores[&space_id].len(),
        baseline_store_len,
        "event store length must not increment on HeldPending"
    );

    // Note: attacker_uri is unused at this assertion level per Option ε.iii lock.
    // Future audit-design-impl arc may add a PendingBuffer accessor to inspect
    // per-entry missing_identity; at that point the test gains a stronger
    // `assert_eq!(buf.missing_identity_for(&forged_event_id), Some(attacker_uri.as_str()))`
    // assertion. For now, attacker_uri is constructed for documentation +
    // forgery construction; the deployment-level regression lock is the
    // two-part (a)+(b) assertion above.
    let _ = attacker_uri;
}

// For state.federation_add (asymmetric cell per Phase 7 B3)
#[tokio::test(flavor = "current_thread")]
async fn forged_sender_with_resign_state_federation_add() {
    let (mut rt, alice, space_id, peer_id) = setup_runtime_with_alice_in_space().await;

    let attacker_key = generate_attacker_keypair();
    let forged_event = forge_resigned_state_federation_add(&attacker_key, &space_id).await;
    let forged_event_id = forged_event
        .event_id
        .as_deref()
        .expect("event_id must be set post-canonicalisation")
        .to_string();

    let baseline_store_len = rt.stores[&space_id].len();
    let outcome = rt.dispatch_event(
        forged_event.clone(),
        EventOrigin::ReceivedViaFederation,
        Some(peer_id.as_str()),
    );

    // Phase 7 B3 amendment (J-088): federation_add via federation channel SKIPS
    // step 9 + step 11 + step 13; only steps 8 + 10 + 12 fire. Step 12 passes
    // because attacker re-signed with own embedded key. Event INGESTS.
    //
    // This is the security property the test pins: the protocol does NOT prevent
    // an attacker who already has a federation channel from ingesting forged
    // federation_add events via that channel. Authority chain: session handshake
    // + signature self-verification. See findings v1.5 §2.4.2 + J-088.
    //
    // DispatchOutcome::Accepted IS a struct variant per runtime.rs:88-92
    // (Accepted { new_joiner: Option<String>, additional_persisted: Vec<Event> });
    // struct-style match arm with `..` rest pattern compiles correctly.
    assert!(
        matches!(outcome, DispatchOutcome::Accepted { .. }),
        "expected Accepted (Phase 7 B3 asymmetric cell), got {:?}",
        outcome
    );
    assert!(
        rt.stores[&space_id].contains(&forged_event_id),
        "forged federation_add event {} must be in EventStore per Phase 7 B3 design property",
        forged_event_id
    );
    assert_eq!(
        rt.stores[&space_id].len(),
        baseline_store_len + 1,
        "event store length must increment by 1 per Phase 7 B3 design property"
    );
}
```

**The variant-uniformity property** for variants 1+3+4 is proved by all 5 family tests of one variant passing with the same Rejected assertion shape. For variant 2, the matrix is asymmetric (4 HeldPending + 1 Accepted); variant-uniformity becomes "same outcome class within a variant" where outcome class = "defer-or-ingest at unknown-signer per F-4 + Phase 7 B3 layering". The asymmetry IS the outcome class. Family-uniformity for the matrix as a whole is proved by all 4 variants of one family passing with their respective shapes.

**Helper functions** (Clair authors at file-top or in a shared helpers module):

- `setup_runtime_with_alice_in_space() -> (NodeRuntime, Identity, SpaceId, NodeId)` — standard fixture
- `forge_<variant>_<family>(...)` — one per (variant, family) pair
- `generate_attacker_keypair()` + `pubkey_uri(&key) -> String` for variant 2 tests
- OR: a single `forge_event(variant, family, ...) -> Event` dispatch helper — Clair's call based on cleanliness

**Template-compile-check discipline note (J-116 amendment).** The J-115 amendment authoring closed the variant 2 contract gap (HeldPending vs Rejected outcome) but introduced API-shape drift in the assertion templates themselves — `DispatchOutcome::HeldPending(buffer_state)` against a unit variant; `DispatchOutcome::Rejected { reason }` struct syntax against a tuple variant; `SpaceState::dag()` + `SpaceState::event_count()` against methods that don't exist; raw `&forged_event.event_id` against an `Option<String>` field. D-077-shape framing: the forward-sustainability question (what does production do?) was asked at J-115 amendment authoring; the backward-coherence question (does the proposed template's API-shape compile against production types?) was not. Future sibling milestone runbook authors: when the runbook's §4.5 sample code names a production API by signature, verify that signature against production code (grep enum-shape + grep accessor-method-existence) before locking the sample. The verification cost is small relative to the avoided-rework cost when Clair re-surfaces at a subsequent checkpoint #2. Recorded as candidate D-NNN-β "template-compile-check at runbook authoring" at §7.9; one instance now; promotion-watch open per D-069 surface-driven application.

### §4.6 The 20 test names — Joe-locked at J-113, contract refined at J-115

Verbatim from findings v1.4 §2.4.1 (variant-major / family-minor ordering). Names unchanged from J-113; the contract for variant 2 (`forged_sender_with_resign_*`) refined at J-115 against post-Phase-6 F-10 reality per findings v1.4 §2.4.2.

```
bad_signature_message_text
bad_signature_membership_join
bad_signature_membership_kick
bad_signature_state_federation_add
bad_signature_state_room_create
forged_sender_with_resign_message_text
forged_sender_with_resign_membership_join
forged_sender_with_resign_membership_kick
forged_sender_with_resign_state_federation_add
forged_sender_with_resign_state_room_create
mutated_event_id_message_text
mutated_event_id_membership_join
mutated_event_id_membership_kick
mutated_event_id_state_federation_add
mutated_event_id_state_room_create
malformed_prev_events_message_text
malformed_prev_events_membership_join
malformed_prev_events_membership_kick
malformed_prev_events_state_federation_add
malformed_prev_events_state_room_create
```

These names are Joe-locked. Clair MUST NOT rename without surfacing to Joe at checkpoint #4.

### §4.7 Module doc-comment for `phase9_validation_asymmetry.rs`

The verbatim block at the top of the file (Joe-locked at checkpoint #4 if any element diverges):

```rust
//! Phase 9 Scenario 4 — Validation asymmetry regression (NodeRuntime-level).
//!
//! Owns F-4 (Phase 2 pipeline unification). 20 tests across 4 forgery variants × 5 event families.
//!
//! ## Production contract (per findings v1.3 §2.4.1, J-113 amendment)
//!
//! Each test maps 1:1 to a distinct ExchangeError variant at
//! `xgen-core/src/message/exchange.rs:46-87`:
//!
//! | Variant | Production outcome | Step | Notes |
//! |---|---|---|---|
//! | bad_signature_* | Rejected(SignatureFailure) | 12 | substring "step 12: signature verification failed" |
//! | forged_sender_with_resign_* | HeldPending (4 families) + Validated-then-ingested (state.federation_add) | 11 / B3 skip | Phase 6 F-10 (J-087) + Phase 7 B3 (J-088); see §state.federation_add row below |
//! | mutated_event_id_* | Rejected(EventIdMismatch) | 8 | substring "step 8: event_id does not match canonical content hash" |
//! | malformed_prev_events_* | Rejected(DagError(String)) | 10 | substring "step 10: DAG structural violation" |
//!
//! ## Two load-bearing structural properties
//!
//! 1. **Family-uniformity**: same validation pipeline regardless of event family.
//!    Proved by all 4 variant tests of one family passing.
//!
//! 2. **Variant-uniformity within a family**: all 4 variants reject via F-4's same
//!    dispatch shape. Proved by all 5 family tests of one variant passing.
//!
//! ## Not covered (per findings §2.4.1 narrow scope)
//!
//! - Timestamp-bound forgery (Gap G6 flagged-not-promoted; no production variant).
//! - Mutation-without-resign sender forgery (redundant with bad_signature_*).
//! - NotASpaceMember / NotARoomMember / PermissionDenied — outside F-4 scope.
//! - HeldPending — defers, doesn't reject; Scenario 5/6 territory.
//!
//! ## `state.federation_add` row
//!
//! Verifies Phase 7.5 §5 narrowness AND Phase 7 B3 asymmetric cell (J-115 amendment).
//! For variants 1, 3, 4: F-3 skips state.federation_add for missing relationship, but
//! F-4 still catches forgery (Rejected with variant-specific substring). Sibling-shape
//! to C2's narrowness-regression assertion at the F-3 layer (J-112).
//!
//! For variant 2 (`forged_sender_with_resign_state_federation_add`): asymmetric cell.
//! Phase 7 B3 (J-088) skips step 11 sender-registration for federation_add via federation
//! channel; step 12 passes on attacker's self-signature; event INGESTS. This is the locked
//! security property: the protocol does NOT prevent an attacker who already has a federation
//! channel from ingesting forged federation_add events via that channel. Authority chain:
//! session handshake + step-12 signature self-verification.
```

### §4.8 Per-test verification at run

Each test runs in isolation with `cargo test -p xgen-core <test_name>`. The full Scenario 4 set runs with `cargo test -p xgen-core phase9_validation_asymmetry`. Expected runtime: ~5-10 seconds for all 20 tests combined (each test is NodeRuntime construction + 1 dispatch + assertion — no TCP, no federation_session).

---

## §5 — Compounds (C5, C7, C9, C10)

### §5.1 Compound C5 — Validation asymmetry under load

**Contract** (per findings §3.5): Phase 9 baseline scenario 4 tests with one forged event per assertion; C5 tests at scale. Send 100 mixed valid + forged events to NodeRuntime via direct `dispatch_event` calls; assert per-event outcome independence.

**What bug it catches**: catalogue bug M10 (validation asymmetry leaks rejection state across events under load).

**File**: `xgen-core/src/node/tests/phase9_compound_c5_validation_under_load.rs`. One test (likely).

**Test name** (proposed; surface at checkpoint #4): `c5_validation_asymmetry_under_load_100_events_mixed_valid_and_forged`.

**Composition** (proposed; surface at checkpoint #4):
- 50 valid events (mix of families per §4.3) — each should be `DispatchOutcome::Accepted`
- 50 forged events (mix of (variant, family) pairs per §4.2 + §4.3) — each should be `DispatchOutcome::Rejected` with the variant-specific substring

**Order**: events fed in random order via shuffle (seed pinned for determinism; Clair's call on seed value, surface at checkpoint #4).

**Assertion per event** (production-grounded shape per J-116 amendment — `dispatch_event` is sync; `DispatchOutcome::Rejected` is a tuple variant per runtime.rs:88-95):
```rust
for (event, expected) in events_with_expected.iter() {
    let outcome = rt.dispatch_event(
        event.clone(),
        EventOrigin::ReceivedViaFederation,
        Some(peer_id.as_str()),
    );
    match expected {
        Expected::Accepted => assert!(matches!(outcome, DispatchOutcome::Accepted { .. })),
        Expected::Rejected(substring) => {
            assert!(matches!(outcome, DispatchOutcome::Rejected(_)));
            let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
            assert!(reason.contains(substring));
        }
    }
}
```

**Load-bearing structural property** (per §2.4.1 sibling-shape): isolation. No forged event's rejection state affects a subsequent valid event's acceptance. The 100-event sequence proves this by construction — if isolation broke, the next valid event after a forged-event rejection would fail to ingest.

**Estimated lines**: ~150-250.

### §5.2 Compound C7 — `continue_from` pagination at boundary

**Contract** (per findings §3.7): F-1a tip-exchange uses pagination per F-7. Test that delta size + `continue_from` chains work at the boundary. 4 test cases: N=999, N=1000, N=1001, N=2000.

**What bug it catches**: catalogue bug M7 (continue_from pagination loses events at boundary).

**File**: `xgen-node/src/tests/phase9_compound_c7_pagination_boundary.rs` (Joe-locked at checkpoint #2 — C7 lives xgen-node-side because `compute_federation_delta_for_space` is in `xgen-node/src/fanout.rs`; NOT declared in `xgen-core/src/node/tests/mod.rs`). 4 tests.

**Test names** (proposed; surface at checkpoint #4):
- `c7_pagination_n_999_below_boundary`
- `c7_pagination_n_1000_at_boundary_exact`
- `c7_pagination_n_1001_just_above_boundary`
- `c7_pagination_n_2000_double_boundary`

**Open question — where does C7 assert?** Per findings §3.7 "NodeRuntime-level for the assertion sharpness; pair with one end-to-end smoke test (can be folded into one of the deployment scenarios if budget tight)." Two readings:

- **Reading α** — C7 tests at the `compute_federation_delta_for_space` xgen-node-layer function. But that's xgen-node, not xgen-core; doesn't fit `xgen-core/src/node/tests/`. **Inconsistent with the file location lock.**
- **Reading β** — C7 tests at a NodeRuntime-layer API that exposes the delta size computation. **Does this API exist?** Production calls `compute_federation_delta_for_space` from `xgen-node/src/federation_session.rs::run_federation_session`; NodeRuntime doesn't have a `compute_federation_delta` method directly.

**Surface at checkpoint #2 (D-078 application for C7)**: confirm the assertion target. If C7 needs the xgen-node-layer entry, the file location is `xgen-node/src/tests/phase9_compound_c7_pagination_boundary.rs` (not `xgen-core/`). This is a Trigger (a) instance — the contract refines the file location, not the scope.

**Conservative posture**: C7 likely lives at `xgen-node/src/tests/` not `xgen-core/src/node/tests/`. The mod.rs at §4.4 might NOT declare C7. **Clair surfaces this at checkpoint #2 explicitly.**

**Setup per test**:
1. Build NodeRuntime A with Space S containing exactly N events (mix of families).
2. Build NodeRuntime B with no events for S.
3. Federate A and B (or invoke the delta-computation function directly with a fresh receiver context).
4. Assert all N events arrive on B's DAG.

**Estimated lines**: ~250-400 (the N-event-builder may be verbose for N=1000-2000).

### §5.3 Compound C9 — F-3 drain-time approximation hazard

**Contract** (per findings §3.9): A federation event from peer X for Space S buffers in B's PendingBuffer (missing predecessor). While buffered, X is removed from S's `federation_nodes`. Predecessor arrives. Drain re-dispatches with `peer_node_id: None`. Does the event ingest or reject?

**Production hazard disclosed at `xgen-core/src/node/runtime.rs:864-865`** (doc-comment; J-115 line-number correction from v1.0's stale `:529-535` reference which pointed to the Phase 7.5 §6 held-not-bypassed text, not the drain-time hazard text): "a buffered federation event whose peer relationship was torn down within the 30 s HeldPending window slips through" — F-3 not re-checked on drain.

**Test purpose**: verify the hazard's bound (≤ 30s F-4a window). If bound exceeds 30s, that's an unrecorded bug.

**What bug it catches**: hazard-bound violation (the doc-comment claims 30s; the test proves it).

**File**: `xgen-core/src/node/tests/phase9_compound_c9_drain_time_hazard.rs`. 1 test.

**Test name** (proposed; surface at checkpoint #4): `c9_f3_drain_time_approximation_within_30s_window`.

**Sequence**:
1. NodeRuntime B with Space S, Alice's Identity registered, X in `federation_nodes`.
2. X dispatches event E for S with prev_events referencing event-not-yet-in-DAG. F-3 passes (X is in federation_nodes); F-4a buffers on missing predecessor. `DispatchOutcome::HeldPending`.
3. X removed from S's `federation_nodes` (defederation event from operator).
4. Predecessor event P arrives. P ingests, triggers `drain_pending_uniform` for E.
5. Drain re-dispatches E with `peer_node_id: None` per the production approximation.
6. **Assertion**: E ingests (production hazard is an *accepting* approximation). Event count for S incremented by 2 (P + E).
7. **Bound assertion**: total elapsed wall-clock from step 2 to step 5 < 30s. (The 30s bound is F-4a's window; if buffering exceeds 30s, the event would have been dropped before drain.)

**Documented behaviour**: the test documents the accepting approximation; it does NOT test rejection. Findings §3.9 explicitly: "the design doc explicitly accepts it." Test proves the bound, not the policy.

**Estimated lines**: ~120-200.

### §5.4 Compound C10 — Identity-replicate hook serialisation under lock contention

**Contract** (per findings §3.10): `handle_identity_replicate_msg` at `xgen-node/src/app.rs:1695` (J-115 line-number correction from v1.0's stale `:1592`) calls `rt.drain_pending_by_identity` (at `xgen-core/src/node/runtime.rs:911`, J-115 correction from v1.0's stale `:680+`) inside the same runtime-lock critical section as the identity registration. Under high concurrent federation push load (many incoming events for the same Identity), can a hook fire while another hook is mid-flight? Are buffered events for the same Identity drained twice?

**What bug it catches**: catalogue bug M9 (parallel arrivals double-drain) + new bug M14 (lock-contention-induced ordering bug).

**File**: `xgen-node/src/tests/phase9_compound_c10_identity_lock_contention.rs` (Joe-locked at checkpoint #2 — C10 lives xgen-node-side because `handle_identity_replicate_msg` is in `xgen-node/src/app.rs:1695`; NOT declared in `xgen-core/src/node/tests/mod.rs`). 1 test.

**Open question — file location**: C10's lock contention surface is xgen-node-side (`handle_identity_replicate_msg`); the drain helper is xgen-core-side (`drain_pending_by_identity`). Same Reading α vs β question as C7. **Surface at checkpoint #2 per D-078.**

**Conservative posture**: C10 likely lives at `xgen-node/src/tests/` not `xgen-core/src/node/tests/`. The mod.rs at §4.4 might NOT declare C10.

**Test name** (proposed; surface at checkpoint #4): `c10_identity_replicate_hook_serialisation_no_double_drain`.

**Setup**:
1. NodeRuntime B with Space S, Alice's Identity registered, Bob NOT registered, 3 federation_peers (X1, X2, X3) in S's `federation_nodes`.
2. Concurrent producer task: 3 spawned tokio tasks, each one sends a different event from one of (X1, X2, X3) on behalf of Bob to B. All 3 events HeldPend on F-10 (unknown signer Bob).
3. Concurrent identity-replicate: 3 spawned tasks each send a `register_identity(bob)` call to B.

**Assertion**:
- Total events ingested for S after all tasks complete: 3 (each unique event ingests exactly once; no duplicates).
- Total drain hook invocations: ≤ 3 (each invocation processes 0..3 events; total processed across invocations = 3).
- No DAG-rejection from duplicate-ingest.

**Concurrency mechanism**: `tokio::spawn` + `JoinSet`; Clair's call on synchronisation between producer and identity-replicate phases (likely a `tokio::sync::Notify` to ensure all 3 events buffer before the first identity-replicate fires).

**Estimated lines**: ~200-300.

---

## §6 — Verification + DoD + milestone-bearing freezes

### §6.1 Verification rigour at Commit 3b-4

Per Lock 3:
- **5 isolated runs**: `cargo clean` between each, then `cargo test --workspace`.
- **3 workspace runs**: no cargo clean; back-to-back runs.
- **Total: 8 green runs minimum.** Sibling-shape to topo-sort J-101 Commit 3 verification.

Pre-existing flakes (precedence env-var race; `reconnect_with_existing_tip_small_delta_delivered`) MUST NOT fire during any of the 8 runs. If a flake fires:
- Re-run the failing run.
- If the flake recurs in the same pattern, escalate per Rule 3 — surface to Joe before continuing.

### §6.2 Expected test count delta

Baseline: 600 tests at J-112.
Scenario 4 adds 20 tests.
C5 adds 1 test.
C7 adds 4 tests.
C9 adds 1 test.
C10 adds 1 test.
**Total delta**: +27 tests → **627 tests minimum after Commit 3b-4.**

If splits per §2.2 morph the file structure, the test count delta is the same; per-commit deltas split per the contingency.

### §6.3 `cargo test` evidence per Rule 5

Quote the verbatim cargo output in the JOURNAL entry (J-115). Sibling-shape to J-110/J-111/J-112:

```
test result: ok. <COUNT> passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in <TIME>s
```

Sum the per-crate counts; assert total ≥ 627.

### §6.4 DoD checklist for Commit 3b-4

- [ ] 5 new test files at locked locations per §4.4 + §5.x (or amended locations per checkpoint #2 surfacing).
- [ ] 20 Scenario 4 tests by the locked names (§4.6).
- [ ] 1 C5 test + 4 C7 tests + 1 C9 test + 1 C10 test.
- [ ] All honesty assertions per findings §2.4 + §2.4.1 + §3.5 + §3.7 + §3.9 + §3.10 satisfied.
- [ ] Module-level doc-comment present at every new test file per §4.7 template (adapted per family).
- [ ] `cargo test --workspace` 5 isolated + 3 workspace = 8 green runs.
- [ ] Test count quoted per Rule 5; total ≥ 627.
- [ ] No pre-existing flakes fired.
- [ ] J-115 entry written + header chain in JOURNAL.md.
- [ ] CLAUDE.md PLAY block flipped to Commit 3b-5 (milestone close).
- [ ] ROADMAP.md v1.29 → v1.30 with Past entry + header chain.

### §6.5 J-NNN placeholder freeze sites

Placeholders to freeze at Commit 3b-4 ship (sibling-shape to J-101 + J-108 placeholder freeze pattern):

- This runbook's body J-NNN references freeze to J-115 (the Commit 3b-4 ship JOURNAL entry).
- DECISIONS.md D-078 entry body J-NNN references freeze to J-114 (this runbook-authoring entry) — **frozen at this commit, not at 3b-4 ship.**
- Module doc-comments at the 5 new test files: if any reference J-NNN, freeze to J-115.
- Phase 9 §3.0a entry mentioning Commit 3b-4 may gain a "Commit 3b-4 SHIPPED at J-115" annotation (Clair's call at ship time).

### §6.6 Anti-drift guardrail at milestone-bearing commit

Before staging the Commit 3b-4 atomic commit, run:

```
grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md' --include='*.toml'
```

Per J-108's grep guardrail scope discipline codification: this grep MUST return ZERO matches in canonical sources (code/spec/test docs hosting freezable placeholders). Narrative prose in CLAUDE.md + JOURNAL.md (entries that use J-NNN as historical pointer at authoring time) is OUT-of-scope for this grep — the unconstrained grep returns non-zero by design.

### §6.7 Milestone close — what Commit 3b-4 closes vs leaves PLAY

Commit 3b-4 ships the test families. **Commit 3b-5 (the next commit) is the Phase 9 milestone close** per the Phase 9 §3.0a five-commit shape. Commit 3b-4 does NOT close Phase 9; Phase 9 milestone stays PLAY pending Commit 3b-5.

After Commit 3b-5 closes Phase 9, Federation Event Propagation milestone flips PLAY → DONE, and M6 (new) + XGID Retrofit Pass 1 unblock.

---

## §7 — Discipline notes

Nine sub-sections per Lock 5 + J-115 + J-116 amendments, sibling-shape to topo-sort runbook §7 + persistence-amendment runbook §7.

### §7.1 Precedent-departure self-defense for §7 inclusion

Topo-sort runbook (J-098) added §7 over the bidirectional precedent's absence. Persistence-amendment runbook (J-106) followed topo-sort's shape with §7. This runbook follows the trilogy precedent and includes §7. Three grounds per the topo-sort precedent:

1. **Trilogy-internal consistency** outranks one-step-earlier-precedent consistency. Three prior runbooks now ship §7; this runbook joining makes it four. Pattern durable.
2. **Pattern-naming-when-pattern-is-durable.** The Phase 9 Commit 3b arc has produced sibling-shape findings at J-110, J-111, J-112, J-113 (cross-crate trace assertion; tracing-test exhaustion; counter routing; canonical staleness third instance). The Commit 3b-4 implementation will likely surface its own findings; §7 frames where they record.
3. **D-078 needs runbook-visible pointer.** D-078 is promoted in this commit; the runbook is the natural place to make D-078's protocol-test-layer application visible — §7.2.

### §7.2 D-078 promotion + runbook-visible application

**Pattern**: principle stated → implementation surfaces gap → amendment makes the missing dimension explicit → principle's surface in future milestones makes the dimension visible upfront.

**Three-instance threshold**: J-099 (audit-doc §11 amendment; canonical-document staleness at re-walk Step 2) + J-109 (survey §2.6 amendment; same pattern at Phase 9 Commit 3b-2) + J-113 (survey §2.4 amendment; same pattern at Phase 9 Commit 3b-4). Three instances = durable pattern.

**D-078 framing**: "production-grounded test enumeration" — at every test enumeration (named list of test cases that will become a regression lock), the production reject-path inventory MUST be confirmed against current code BEFORE the enumeration is Joe-locked. Application surface: Joe-lock checkpoint #4 of this runbook (and sibling runbooks for future milestones).

**Sibling-shape to D-077**: D-077 names bidirectional sustainability at silent-discard sites (meta-layer). D-078 names production-grounded enumeration at test-enumeration sites (protocol-test-layer). The two are family-siblings, addressing the same "ask the question before locking" discipline at different scopes.

**This runbook applies D-078 prospectively**: §2.3 checkpoint #2 names production-code verification for each of the five families before Joe-lock checkpoint #4. If a family fails verification, Trigger (a) fires and Track-1 amendment ships first — the J-113 pattern caught proactively rather than retroactively.

### §7.3 Sibling-in-shape fifth recurrence count

Phase 7.5 first (J-093/J-094/J-104); bidirectional federation_nodes second (J-096); topological-sort third (J-097/J-098/J-099/J-100/J-101); persistence-amendment sub-amendment fourth (J-104/J-105/J-106/J-107/J-108); **Phase 9 Commit 3b arc fifth** (J-110/J-111/J-112/J-113/J-114/upcoming J-115).

Five recurrences make the audit → design → runbook → implementation → milestone-close shape robust. Phase 9 Commit 3b arc is slightly different in shape: no explicit audit phase (the Phase 9 survey + findings serve that role); no design phase as separate session (the design is the findings doc §8 + the per-scenario contracts in §2/§3). The runbook + implementation + milestone-close are the standard three.

### §7.4 Honest longer work over fast shortcuts — count inheritance

Count inherited at eighth from J-104, NOT incremented at this runbook-authoring close (sibling-shape to topo-sort J-098 + persistence-amendment J-106 inherit-not-increment framing — runbook-authoring is within-milestone event, not a new milestone-surface event).

If Commit 3b-4 implementation surfaces a Trigger (a)/(b)/(c) condition that opens a sub-milestone, that opening would be the ninth recurrence. The runbook authoring itself doesn't increment.

### §7.5 Family-uniformity + variant-uniformity as load-bearing structural properties

These properties are the deep regression locks for Scenario 4 — the reason 20 tests beat 1 test by more than 20×.

**Family-uniformity**: same validation pipeline regardless of event family. A regression that broke F-4's family-blindness (e.g., a special-case introduced for `state.federation_add` that bypassed signature check) would fail multiple tests within one variant's family-set.

**Variant-uniformity within a family**: all 4 variants reject via F-4's same dispatch shape. A regression that handled one variant differently from another (e.g., DagError producing a different outcome shape than EventIdMismatch) would fail multiple tests within one family's variant-set.

**Joint coverage**: the 4×5=20 matrix catches regressions that any 1×1 or 1×5 or 4×1 sub-matrix would miss. This is why the Joe-lock at J-113 preserved the matrix shape even after dropping 2 variants (6→4) — the matrix structure is load-bearing.

### §7.6 The runbook does NOT lock C7 + C10 file location

Per §5.2 + §5.4, C7 and C10 may live at `xgen-node/src/tests/` rather than `xgen-core/src/node/tests/` because their assertion targets are xgen-node-side. The runbook surfaces this at checkpoint #2 per D-078 (production-code verification) rather than locking it at runbook-authoring time, because the assertion target is the variable being verified.

Sibling-shape lesson: where the runbook can't lock without surfacing first, the runbook records the unknown explicitly and routes through D-078's checkpoint #2. This is the runbook's structural application of D-078 — not just "verify the contract" but "where the contract requires verification, route the verification through the checkpoint."

### §7.7 Discipline data point — three-instance threshold + D-078 promotion shape

The promotion of D-078 in this runbook commit (Path B.i per the session's Lock 2 walk) is itself a discipline data point: when does a pattern's repeat-instance count justify promotion to DECISIONS.md?

Sibling-shape precedent: D-076 v1 → v1.1 in-place amendment at J-099 happened when the pattern (principle stated, gap surfaced, amendment makes it explicit) recurred for the second time within the no-drift-surface family. The promotion threshold for that family is two instances because the no-drift-surface family has a one-per-layer shape — two instances within one layer surfaces a conflict, three would be over-engineered.

For D-078, the pattern is "canonical-document staleness at dependent-milestone implementation time." Three instances (J-099 + J-109 + J-113) is the durability threshold for a pattern that doesn't have a one-per-layer shape — three independent recurrences make the pattern's surface visible across multiple milestone types.

This data point worth recording for future pattern-promotion decisions: count three independent instances before promoting unless the pattern has a one-per-layer shape (in which case count two).

### §7.8 D-078 first prospective-catch retrospective (J-115 amendment)

**The catch.** At Pre-Commit-3b-4 Joe-lock checkpoint #2, Clair's D-078 production-code verification surfaced that variant row 2's `forged_sender_with_resign_*` outcome mapped to `Rejected(UnknownSender)` in v1.0 of this runbook + v1.3 of findings, but `validate_event` at `xgen-core/src/message/exchange.rs:626-632` returns `HeldPending` for unknown-signer events per the Phase 6 F-10 amendment (J-087). The `UnknownSender` ExchangeError variant exists in the enum but is only reachable via the legacy `validate_steps_8_13` path, NOT the F-4 path that `dispatch_event` calls.

**Track 1 amendment shipped pre-implementation.** Joe locked Reading B.i (state.federation_add as Validated-then-ingested per Phase 7 B3 asymmetric cell); the six-file atomic commit landed before Clair wrote any test code. Zero test code thrown away; zero retroactive correction needed.

**The prospective-vs-retroactive distinction.** Sibling-shape to J-099 + J-109 + J-113 (the three retroactive instances that established the D-078 promotion threshold) but procedurally distinct:

| Instance | Catch shape | Cost-avoided |
|---|---|---|
| J-099 | Retroactive (Clair halts mid-Commit-3 verification of topo-sort fix) | Commit 2 already shipped against partial contract; canonical record amended at re-walk Step 2 |
| J-109 | Retroactive (Clair halts mid-Pre-Commit-3b-2-equivalent verification) | Pre-implementation but post-contract-locking; survey amended before code |
| J-113 | Retroactive (Clair halts mid-Commit-3b-4 forgery-helper construction) | Test code partially written; survey amended at "forge helper" boundary |
| **J-115 (this)** | **Prospective (Clair halts at Pre-Commit-3b-4 checkpoint #2, the D-078 application surface)** | Zero test code written; canonical record amended before any implementation effort |

The prospective shape is what D-078 was promoted to produce. The fact that the very next milestone after D-078 promotion (J-114) instantiated the prospective shape on its first checkpoint #2 application is the proof D-078's surface is load-bearing.

**Discipline-notes data point for next sibling milestone runbook author.** The prospective catch shape is the principle working as designed. The retroactive catches (J-099 / J-109 / J-113) are what established the pattern; the prospective catch (J-115 / this) is what validates the prevention. Future sibling milestones' runbook §7 should record their own catches under whichever shape applies and update the cumulative count.

**Candidate D-NNN flagged-not-promoted**: "prospective-catch count separation" — if D-078 prospective catches accumulate to 3 instances, a separate count from retroactive-catches may become warranted. One instance now; flag per D-069 audit-vs-design boundary discipline; promotion trigger Joe-lock OR three-instance threshold.

**Honest framing per D-065**: the J-115 first prospective catch absorbed under the existing "honest longer work over fast shortcuts" recurrence count (ninth recurrence) rather than opening a separate prospective-count immediately. Framing α over Framing β — the recurrence count tracks session-arcs that delay milestone closure, and prospective + retroactive both instantiate that discipline. Separating counts at one instance is premature optimisation; absorbing under one count preserves the count's semantic robustness.

### §7.9 Template-compile-check at runbook authoring (J-116 amendment)

**The catch.** At Pre-Commit-3b-4 Joe-lock checkpoint #2 (second pass against J-115-amended runbook v1.1), Clair's D-078 production-code verification surfaced that the J-115 amended assertion templates at §4.5 + findings v1.4 §2.4.2 referenced API shapes that don't exist in production:

| Template reference | Production reality | Source |
|---|---|---|
| `DispatchOutcome::HeldPending(buffer_state)` (struct destructure with payload field access) | `DispatchOutcome::HeldPending` — **UNIT variant**, no payload | `xgen-core/src/node/runtime.rs:88-95` |
| `DispatchOutcome::Rejected { reason }` (struct-style destructure) | `DispatchOutcome::Rejected(String)` — **TUPLE variant** | `xgen-core/src/node/runtime.rs:94` |
| `rt.spaces[&space_id].dag().contains_event(&event_id)` | No `dag()` method on `SpaceState`; correct: `rt.stores[&space_id].contains(&event_id)` | `xgen-core/src/space/state.rs:150+` + `xgen-core/src/dag/store.rs:48` |
| `rt.spaces[&space_id].event_count()` | No `event_count()` method on `SpaceState`; correct: `rt.stores[&space_id].len()` | `xgen-core/src/dag/store.rs:52` |
| `&forged_event.event_id` (raw &str) | `Event::event_id: Option<String>` — needs `.as_deref().expect(...)` | `xgen-common/src/wire.rs:332` |

**Track 1 sub-amendment shipped pre-implementation.** Joe locked Reading α (Track 1 sub-amendment of J-115; runbook v1.1 → v1.2 + findings v1.4 → v1.5; sibling-shape to J-099 / J-109 / J-113 / J-115 canonical-record-amendment-first precedent) over Reading β (Clair latitude + JOURNAL divergence; rejected as no-drift-surface anti-pattern) + Reading γ (hybrid bundle into Clair atom; rejected as defeats D-078's prospective-catch purpose at its second application). Five-file atomic commit per D-074 sixteenth instance landed before Clair wrote any test code. Zero test code thrown away; zero retroactive correction needed.

**Sibling-but-distinct from J-115's catch shape.** J-115 caught contract intent vs production behaviour mismatch (HeldPending vs Rejected outcome — what the test proves). This catch caught template assertion API vs production type shape mismatch (HeldPending unit variant; Rejected tuple syntax; SpaceState method absence — how the test asserts). The two catch shapes are sibling-but-distinct enough that lumping into one candidate would smear semantically-distinct surfaces.

**D-077-shape framing.** Forward-sustainability question (what does production do?) was asked at J-115 amendment authoring; backward-coherence question (does the proposed template's API-shape compile against production types?) was not. This atom closes the missed dimension by rewriting the templates against verified production APIs.

**Open question ε resolution — Option ε.iii locked.** PendingBuffer does not expose `missing_identity` per-entry as a public accessor today. The J-116 amended template asserts F-10 buffering via `pending.contains(event_id)` + `pending.pending_identity_count() >= baseline + 1` — the available deployment-level observability surface. Stronger `missing_identity == Some(attacker_uri)` assertion would require a new public accessor (Option ε.ii); deferred per D-069 audit-vs-design boundary as its own audit-design-impl arc per D-071 if dependent work surfaces need.

**Two distinct candidate D-NNNs flagged-not-promoted per D-069**:

- **Candidate D-NNN-α** — "prospective-catch count separation" (contract-surface). One instance at J-115 (first prospective catch at contract surface). Promotion-watch open per J-115 §7.8.
- **Candidate D-NNN-β** — "template-compile-check at runbook authoring" (template-surface). One instance at J-116 (this catch). Promotion-watch open per this sub-section.

Both at one-instance below the three-instance threshold OR Joe-lock. The catch-shape distinction (contract-surface vs template-surface) is preserved as semantically-distinct framing so neither candidate smears the other's discipline-pattern surface.

**Discipline data point for next sibling milestone runbook author.** When the runbook's §4.5-shape sample code names a production API by signature, the authoring discipline is to verify that signature against production code (grep enum-shape + grep accessor-method-existence) before locking the sample. The verification cost is small relative to the avoided-rework cost when Clair re-surfaces at a subsequent checkpoint #2. J-115 amendment authoring did the contract-grounding (forward-sustainability) correctly but skipped the template-compile-check (backward-coherence); future sibling milestones' runbook §7 should record any catches under this shape and update the cumulative count.

**Honest framing per D-065**: the J-116 second prospective catch absorbed under the existing "honest longer work over fast shortcuts" recurrence count (tenth recurrence) rather than opening a separate template-prospective-count immediately. Same Framing α reasoning J-115 used — each prospective catch that delays milestone closure is its own recurrence regardless of which structural layer (contract vs template-API) the catch surfaces at. Candidate D-NNN-α + D-NNN-β stay flagged-not-promoted as separate-but-similar surfaces; both await three-instance threshold OR Joe-lock per D-069.

---

## §8 — Cross-references

- **Findings v1.5** — `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` v1.5 §2.4 + §2.4.1 (Scenario 4 contract per J-113) + §2.4.2 (variant row 2 F-10 contract per J-115 + template-API rewrite per J-116) + §3.5 (C5) + §3.7 (C7) + §3.9 (C9) + §3.10 (C10).
- **Phase 9 task** — `tasks/FEDERATION_PROPAGATION_PHASE_9.md` v1.1 §3 Commit 6 (Phase 9-side enumeration; defers to findings).
- **DECISIONS.md** — D-078 (promoted at J-114; the principle this runbook applies prospectively; first prospective catch shipped at J-115 per §7.8; second prospective catch shipped at J-116 per §7.9); D-077 (sibling meta-layer discipline; the J-116 catch surfaces the missed backward-coherence dimension at amendment-authoring); D-076 v1.1 (sibling-shape promotion pattern); D-074 (atomic-commit-includes-JOURNAL discipline applied here). NOT amended at J-116 per D-077 explicit-no-amend audit — both candidate D-NNN-α + D-NNN-β stay flagged-not-promoted per D-069.
- **Candidate D-NNNs flagged-not-promoted** — D-NNN-α "prospective-catch count separation" (contract-surface; one instance at J-115; promotion-watch open per §7.8); D-NNN-β "template-compile-check at runbook authoring" (template-surface; one instance at J-116; promotion-watch open per §7.9). Both at three-instance threshold OR Joe-lock per D-069.
- **JOURNAL.md** — J-116 (D-078 second prospective-catch + template-API Track 1 sub-amendment); J-115 (D-078 first prospective-catch + contract-surface Track 1 amendment); J-114 (runbook-authoring + D-078 promotion); J-113 (canonical-record amendment of Scenario 4 enumeration 30 → 20); J-109 (sibling-shape Scenario 6 amendment); J-099 (sibling-shape audit-doc + design-doc §11 amendment); J-088 (Phase 7 B3 amendment origin); J-087 (Phase 6 F-10 generalisation origin).
- **Sibling runbooks** — `tasks/FEDERATION_TOPOSORT_IMPL.md` (COMPLETED v1.2); `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` (COMPLETED v1.2); `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` (COMPLETED v1.1).
- **Production code anchors** —
  - `xgen-core/src/message/exchange.rs:46-87` (ExchangeError enum)
  - `xgen-core/src/message/exchange.rs:545+` (validate_event F-4 unified core)
  - `xgen-core/src/message/exchange.rs:626-632` (Phase 6 F-10 unknown-sender HeldPending return)
  - `xgen-core/src/message/exchange.rs:455-509` (Phase 7 B3 federation_add skip block)
  - `xgen-core/src/node/runtime.rs:317` (dispatch_event entry)
  - `xgen-core/src/node/runtime.rs:334-340` (test-only-reachable doc-comment for validate_steps_8_13 legacy path)
  - `xgen-core/src/node/runtime.rs:864-865` (F-3 drain-time approximation doc-comment, C9's contract source; J-115 line-number correction)
  - `xgen-node/src/app.rs:1695` (handle_identity_replicate_msg, C10's lock surface; J-115 line-number correction)
  - `xgen-core/src/node/runtime.rs:911` (drain_pending_by_identity, C10's hook; J-115 line-number correction)
  - `xgen-core/src/node/runtime.rs:88-95` (DispatchOutcome enum; J-116 template-API verification — HeldPending unit + Rejected(String) tuple + Accepted struct)
  - `xgen-core/src/node/runtime.rs:126-137` (NodeRuntime fields; J-116 — `pub stores: HashMap<String, EventStore>` + `pub pending: HashMap<String, PendingBuffer>`)
  - `xgen-core/src/dag/store.rs:48` (EventStore::contains(&str) -> bool; J-116 — DAG-membership assertion target)
  - `xgen-core/src/dag/store.rs:52` (EventStore::len() -> usize; J-116 — event-count assertion target)
  - `xgen-core/src/dag/pending.rs:491` (PendingBuffer::contains(&str) -> bool; J-116 — F-10 buffer-membership assertion target)
  - `xgen-core/src/dag/pending.rs:498` (PendingBuffer::pending_identity_count() -> usize; J-116 — F-10 waiting-on-identity assertion target)
  - `xgen-common/src/wire.rs:332` (Event::event_id: Option<String>; J-116 — `.as_deref().expect(...)` pattern)
- **CLAUDE.md** — Rules 0/1/3/5 + MANDATORY behaviour rules (Rule 5 cargo-output-quoting applied at §6.3); D-078 application surface at checkpoint #2 per §2.3.

---

**End of runbook.** Status: ACTIVE v1.2. Flips to COMPLETED at Commit 3b-5 milestone close (Phase 9 close), sibling-shape to topo-sort + persistence-amendment runbook lifecycle.
