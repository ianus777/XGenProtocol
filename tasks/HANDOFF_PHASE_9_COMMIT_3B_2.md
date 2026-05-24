# Handoff — Phase 9 Commit 3b-2 (Scenarios 5 + 6 + B1 honesty test)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-24 (Flipped Status ACTIVE → COMPLETED v1.1 at Phase 9 Commit 3b-2 ship per the bridge's own §9 DoD; load-bearing purpose closes with the J-110 commit. Body §1–§8 stays authoritative as historical record of HANDOFF-at-authoring-time per the bridge-handoff-retention discipline (anti-tempfile-deletion-of-decision-records, D-065 honest-behaviour-over-polite-behaviour; sibling-shape to `tasks/HANDOFF_PERSISTENCE_AMENDMENT_REWALK.md` v1.1 flip at J-107 + `tasks/HANDOFF_TRACK_1_SESSION_CLOSE_2026-05-24.md` v1.1 flip at J-108). Previous v1.0 content stands authoritative for historical record.) Previous v1.0: 2026-05-24 (Authored at J-108 session close as bridge to next session per Joe-lock at session-close pacing decision (Option 1 — defer to fresh session, four grounds: split-session discipline sibling to topo-sort J-097 → J-098; test-discovery-loop pattern from Commit 2a; worst-split-point risk; D-077 forward-drift sustainability on session-pacing precedent). Sibling-shape to `tasks/HANDOFF_PERSISTENCE_AMENDMENT_REWALK.md` precedent at J-107 + `tasks/HANDOFF_TRACK_1_SESSION_CLOSE_2026-05-24.md` bridge-handoff precedent at same milestone. Per Rule 0 + D-065 + D-074 + D-077 discipline.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Purpose

Bridge the J-108 persistence-amendment milestone-close session-arc (Commit 4 `a9ac371` pushed to remote) and the Phase 9 Commit 3b-2 authoring session-arc (next-active for Clair). This HANDOFF carries the session-open framing + scope-correction + discipline-notes that need to be ack-ed at the next session-open per Rule 0 reading-sequence discipline before Commit 3b-2 authoring begins.

The Phase 9 task file (`tasks/FEDERATION_PROPAGATION_PHASE_9.md` Status ACTIVE v1.1, RESUMED at J-108) is the canonical record for Commit 3b-2 scope; this HANDOFF complements rather than supersedes. Body §1–§6 of the task file's §3.0 v1.1 + original §3 Commit 4 sub-section stay authoritative for Scenarios 5 + 6 + B1 detail.

---

## §2 — Session-open framing for next session (mandatory reads per Rule 0)

1. **CLAUDE.md PLAY block** — already updated at J-108 to "Phase 9 Commit 3b-2-equivalent RESUMES" framing.
2. **JOURNAL J-108 entry** — milestone-close entry; nine sub-sections including grep guardrail scope discipline codification (sub-section 8).
3. **This HANDOFF** — session-open ack + scope-correction + discipline-notes (below).
4. **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** §3.0 v1.1 (revised five-commit shape) + original §3 Commit 4 sub-section (Scenarios 5 + 6 + B1 detail).
5. **`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`** §2.5 (Scenario 5 detail) + §2.6 (Scenario 6 detail).
6. **`xgen-node/src/tests/phase9_harness.rs`** — public API surface + in-process harness contract.
7. **Sibling-shape test files** — `xgen-node/src/tests/phase9_two_node_smoke.rs` (Scenario 1 precedent) + `xgen-node/src/tests/phase9_three_node_anti_transitivity.rs` (Scenario 2 precedent) + `xgen-node/src/tests/phase9_drop_and_recover.rs` (Scenario 3 precedent).

---

## §3 — Session-open ack: J-108 prose simplification (D-077 third worked instance at narrative-authoring layer)

At J-108 milestone-close authoring, prose enumeration of Phase 9 remaining scope used a simplified form:

> "Phase 9 Commit 3b-2-equivalent next-active for Clair with Scenarios 2 + compounds C2/C5/C7/C9/C10 (C3 dropped per Phase 9 §3.0 v1.1); ~4-6 atomic commits in their own sequence"

Joe surfaced the Scenario 3 + C3 missing-from-enumeration question at post-J-108 clarification round. Answer: (a) Scenario 3 closed at this milestone per Q4(a) lock + pre-existing-Joe-lock-not-silent-drop for C3 (already dropped at Phase 9 §3.0 v1.0 → v1.1 amendment per Lock #1).

**Broader scope simplification surfaced at this HANDOFF authoring** (post-clarification): the J-108 enumeration also omitted Scenarios 4, 5, 6 + B1 honesty test + the four-commit-sequence shape itself. Sibling-shape to J-108's file-count simplification (10-vs-12 frame) but at scope-enumeration layer rather than file-count layer.

**Joe-locked Option 1 at session-close pacing question**: carry-forward at session-open framing rather than fix-up atom. Grounds:
- Sibling-shape to D-076 v1 → v1.1 amendment lifecycle pattern (v1 prose stays as historical record at v1-authoring time; v1.1 amends forward without rewriting v1).
- J-100's Step-2-bis fix-up atom precedent is wrong-shape — that precedent fixed a *file presence* slip; this surface is *enumeration accuracy in already-shipped prose* (different category).
- J-101's grep-and-quote retrospective discipline ("21 mentions; seventh instance") is closer precedent — handled inline in next entry, not via separate fix-up atom.

**D-077 third worked instance at narrative-authoring layer**:
- **First worked instance** (code-fix layer): Commit 3 prospective sweep at `a677244` — abort-fold + identity-registry-persist + space-event-store-persist closed atomically per D-077 backward-coherence audit; federation-registry-persist audited and confirmed safe via downstream production path.
- **Second worked instance** (narrative-authoring layer, file-count shape): J-108 prose 10-vs-12 file count correction at staging time.
- **Third worked instance** (narrative-authoring layer, scope-enumeration shape): this HANDOFF authoring's recognition of J-108 scope enumeration simplification.

Three instances within ~24h of D-077 promotion (J-107) makes the pattern durable across three boundaries (code-fix layer + narrative-authoring file-count layer + narrative-authoring scope-enumeration layer). Pattern lineage: D-070 + D-075 + D-076 v1 → v1.1 + Rule 0 + D-077 + grep guardrail scope discipline (J-108 sub-section 8) form the surfacing-gap-becomes-codified-discipline family.

**Phase 9 milestone close at Commit 3b-5 will rewrite the framing comprehensively per D-074** (CLAUDE.md PLAY block + JOURNAL J-### entry + ROADMAP visual tree all get accurate four-commit-sequence framing at milestone close). No fix-up atom at this HANDOFF; J-108 prose stays as honest-historical-record.

---

## §4 — Four-commit-sequence-lock (canonical-but-provisional per §3.0 v1.1)

§3.0 v1.1 of `tasks/FEDERATION_PROPAGATION_PHASE_9.md` says "**Revised five-commit shape**" canonically (line 66). With Commit 3b-1 collapsed into persistence-amendment milestone close per Q4(a) lock, **remaining = exactly 4 commits**:

| Commit | Scope | New file(s) | DoD anchor |
|---|---|---|---|
| **3b-2** | Scenarios 5 + 6 + B1 honesty test | `phase9_unknown_signer_first_contact.rs`, `phase9_federation_relationship_rejection.rs` | Original §3 Commit 4 (preserved) |
| **3b-3** | Compound C2 (F-5 anti-transitivity under push queue depth; C3 dropped) | `phase9_compound_c2_anti_transitivity_at_load.rs` | Original §3 Commit 5 with C3 dropped per §3.0 Lock #1 |
| **3b-4** | NodeRuntime-level Scenario 4 + C5 + C7 + C9 + C10 | 5 files in `xgen-core/src/tests/` (Clair picks paths) | Original §3 Commit 6 unchanged |
| **3b-5** | Milestone close per D-074 | Atomic multi-file commit | Original §3 Commit 7 |

**Provisional hedge per §3.0 line 86** (D-065 + D-071 surfacing-gaps-open-sub-milestones discipline): "*The five-commit shape above is the plan if nothing surfaces; the real shape will probably be longer.*" Commit 3b crosses eleven bug-catalogue items (M1, M2, M3, M4-dropped, M5, M7, M9, M10, M11, M12, M14); likely candidates for surface-as-sub-milestone include M4 (race window), M9 (HeldPending double-drain under load — C10 tests directly), M11 (F-3 drain-time approximation bound — C9 tests). Phase 7.5 cold-start, bidirectional `federation_nodes`, topological-sort wire-order non-determinism, and persistence-amendment all opened as sub-milestones during Phase 9 Commit 3b's pre-collapse arc. The four-commit lock holds canonically; the actual shape will be longer if any of M4/M9/M11 (or other gaps not yet surfaced) opens as its own audit → design → impl arc per D-071.

---

## §5 — Commit 3b-2 scope detail (Scenarios 5 + 6 + B1)

Authoritative scope at `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3 Commit 4 (preserved) + survey-findings §2.5 + §2.6. Summary for session-open reading:

### §5.1 Scenario 5 — Unknown-signer first-contact

**Owns**: F-10 (Phase 6 HeldPending generalisation). **Cross-surfaces**: F-10a (30 s timeout), 4006 error code, `pending_identity_replication` counter.

**Load-bearing case (binary level)**: identity arrives 1ms before timeout (resolves) AND 1ms after timeout (must NOT resolve).

**Observation**: `xgen-node_state.json::pending_identity_replication` polling (Phase 6 added it specifically for F-10 observability); sub-second timing via tracing event timestamps.

**Honesty assertion (findings §2.5 sub-item C)**: `pending_identity_replication` decrements within 100 ms of identity-replicate hook firing — proves the hook ran, not periodic sweep. X ≤ 100 ms is the load-bearing window (the hook is synchronous within `handle_identity_replicate_msg`; the sweep task runs every 5 s).

**File location**: `xgen-node/src/tests/phase9_unknown_signer_first_contact.rs` (new file).

### §5.2 Scenario 6 — Federation-relationship rejection

**Owns**: F-3 (Phase 7 verification gate). **Cross-surfaces**: `SpaceState.federation_nodes`, Lock B1 skip-rule.

**Setup**: Node X federates with B at session level but NOT in any of B's Spaces' `federation_nodes`. X attempts to push event for Space S.

**Honesty assertions (findings §2.6 sub-item C, three load-bearing)**:
1. Rejection log line includes exactly `event = "f3_reject"` (G2 trace event from J-092) with `reason = federation_relationship_missing`, peer X, Space S.
2. Event E NOT in B's DAG (verify via `xgen-node_state.json::spaces[].event_count`).
3. B's local fan-out NOT invoked for E (no `fanout_delivered` G3 trace event with E's event_id).

**File location**: `xgen-node/src/tests/phase9_federation_relationship_rejection.rs` (new file).

### §5.3 Lock B1 honesty test (binary-level)

**Negative assertion** per Phase 7 precedent (`state_federation_add_skips_f3_check` integration test at `xgen-node/src/tests/federation_relationship_integration.rs:279`): X sends `state.federation_add` event. Assert outcome is NOT `federation_relationship_missing`.

Per findings §2.6: HeldPending due to F-10 unknown-signer is orthogonal to F-3 skip; the assertion isolates F-3 (NOT a positive accept assertion — the event may HeldPending downstream for unrelated reasons; the assertion only proves F-3 skipped per Lock B1).

**Location**: folded into `phase9_federation_relationship_rejection.rs` as a second test function in the same file (binary-level B1 honesty test elevates the existing NodeRuntime-level integration test to deployment scope; one file, two test functions is the sibling-shape pattern).

---

## §6 — Uniform in-process harness per Lock #2

All Commit 3b-2 tests use `xgen-node/src/tests/phase9_harness::InProcessNode` per §3.0 Lock #2 (uniform in-process harness across Commit 3b scenarios; Scenario 1's J-093 precedent). Public API surface inventory at `phase9_harness.rs:67-381`:

- `spawn_in_process_node` / `spawn_in_process_node_with_state` — node spawn helpers
- `federate(peer_a, peer_b)` — bilateral federation setup
- `register_identity(key)` — identity registration helper
- `submit_locally(event) -> DispatchOutcome` — locally-originated event submission
- `ingest(event)` — direct event ingestion (bypasses dispatch_event)
- `has_event`, `has_space`, `has_federation_peer`, `dag_tips` — state introspection
- `wait_for_event`, `wait_for_space`, `wait_for_federation_peer` — async wait helpers with timeout

**Honesty discipline per Lock #2**: each file's doc-comment names what the in-process shape doesn't exercise. Examples to follow:
- Scenario 3's `shutdown_tx.send(true)` clean shutdown does NOT exercise "process crashed mid-write" failure mode (documented in `phase9_drop_and_recover.rs` doc-comment).
- Scenario 2's in-process three-Node shape does NOT exercise per-event provenance metadata at the runtime store level (documented in `phase9_three_node_anti_transitivity.rs` doc-comment).

**Scenario 5 + 6 + B1 in-process limitations to name in doc-comments**:
- Scenario 5: identity-replication-timing within process boundary doesn't exercise cross-process clock skew or network-stack latency variation that real two-Node TCP would.
- Scenario 6: in-process Node X session establishment via `federate()` doesn't exercise WS handshake under adversarial peer-side behaviour (malformed frames, slow-loris, etc.); F-3 reject path is fully exercised but the WS framing is in-process trusted.

---

## §7 — Joe-lock checkpoint anchors for Commit 3b-2

§3.0 v1.1 records five Joe-lock checkpoints across the Commit 3b arc. No specific Pre-Commit-3b-2 checkpoint is named (the five checkpoints cover Pre-/Post-3b-1, Pre-3b-3, Pre-3b-4, Pre-3b-5). Implementation may proceed without a pre-3b-2 surface unless one of the following triggers fires:

- **Trigger 1** (Rule 3 / Rule 6): test authoring surfaces a finding that belongs at an upstream amendment layer (Phase 4 / Phase 5 / Phase 6 / Phase 7 sub-amendment shape — sibling to Phase 7's B3 amendment + persistence-amendment's drain-without-persist gap). STOP and report.
- **Trigger 2** (D-077): backward-coherence audit at test-authoring time surfaces an implicit-dependency-on-current-behaviour pattern at any of F-10's silent-discard / conditional-mutation surfaces. Surface to Joe via discipline-note before locking the test shape.
- **Trigger 3** (harness gap): if `phase9_harness` API is insufficient for Scenario 5 or 6's required setup (e.g. identity-replicate-with-timing-control not exposed), STOP per Rule 6 — harness extension is structural rework requiring Joe-lock per §5.2 refinement-vs-rework distinction.

Sibling-shape to the unsurfaced Pre-Commit-2a-equivalent at persistence-amendment milestone (Q1+Q2+Q3 walked cleanly without a checkpoint surfacing; the implementation surface produced two D-077 surfaces post-hoc rather than at runbook-locking time, both addressed at Commit 3 prospective sweep).

---

## §8 — Chat Claude standby queue notes (for next sibling milestone's runbook §6.7 amendment)

Two discipline data points recorded at J-108 session close + this HANDOFF authoring, both for next-sibling-milestone runbook authoring to encode:

1. **Canonical-record-source-of-truth lookup discipline**: when citing Phase 9 scope, read `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3.0 of the task file as authoritative; J-091 is historical snapshot at survey close, not current scope. Chat Claude's Track 1 clarification-reply cited J-091 Q4 Lock as canonical; §3.0 v1.1 was already authoritative. Same shape as Track-1-six-site-enumeration that missed app.rs (Chat Claude reading wrong canonical-record source). Forward discipline: identify the authoritative canonical-record source for the surface in question; J-NNN entries are historical snapshots, task files are current scope.

2. **Clair-grep-authoritative-over-runbook-estimate**: at Commit 4 staging, Clair's `grep -rn 'J-NNN' .` was authoritative over Chat-Claude-enumerated freeze-site list (runtime.rs has three sites at 74/216/832 not one at "line 181" estimate; app.rs:2768 site was missed in Track 1 four-site enumeration). Forward discipline for next sibling milestone runbook §6.7 + §6.8 amendment: "Clair greps for J-NNN at Commit 4 staging time; runbook estimate is best-effort, grep is canonical." Sibling-shape to D-077's bidirectional sustainability discipline — author's estimate is forward-projection; executor's grep is backward-coherence audit at staging.

Both items flagged for the next runbook author to encode at §6.7 + §6.8 amendment.

---

## §9 — DoD for this HANDOFF (when it flips ACTIVE → COMPLETED)

This HANDOFF flips Status: ACTIVE → COMPLETED v1.1 at Commit 3b-2 ship per the established HANDOFF lifecycle pattern (sibling-shape to `tasks/HANDOFF_PERSISTENCE_AMENDMENT_REWALK.md` which flipped at Track 1 ship `b9a30da`). The bridge's load-bearing purpose closes when the next session's Commit 3b-2 atomic-commit lands on remote with its own J-### entry.

Header `Last updated` chains a Commit 3b-2 ship reference at the flip; body §1–§8 stays authoritative as historical record of HANDOFF-at-authoring-time per the bridge-handoff-retention discipline (anti-tempfile-deletion-of-decision-records, D-065 honest-behaviour-over-polite-behaviour).

**Atomic-commit shape for Commit 3b-2**: two new test files + `xgen-node/src/tests/mod.rs` module registration + JOURNAL J-### entry + this HANDOFF Status flip per D-074 milestone-close-includes-JOURNAL discipline. Estimated five-file atomic commit. Per-commit-cadence per §3.0 Lock #3.

---

## §10 — Cross-references

- `CLAUDE.md` — PLAY block "Phase 9 Commit 3b-2-equivalent RESUMES" + header chain at J-108.
- `JOURNAL.md` — J-108 milestone-close entry (eight sub-sections including grep guardrail scope discipline at sub-section 8).
- `tasks/FEDERATION_PROPAGATION_PHASE_9.md` — Status ACTIVE v1.1, RESUMED at J-108; §3.0 v1.1 + original §3 Commit 4 sub-section authoritative.
- `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` — §2.5 + §2.6 + new M16 catalogue row at J-108.
- `docs/xgen_federation_propagation_design.md` — §6.4.4 persistence-amendment subsection + §15 row [J-108] persistence-amendment row.
- `DECISIONS.md` — D-074 (milestone-close-includes-JOURNAL), D-077 (bidirectional sustainability discipline).
- `tasks/HANDOFF_PERSISTENCE_AMENDMENT_REWALK.md` (COMPLETED v1.1) — precedent for HANDOFF-as-session-bridge shape.
- `tasks/HANDOFF_TRACK_1_SESSION_CLOSE_2026-05-24.md` (COMPLETED v1.1) — precedent for mid-Track-1 session-close bridge.
- `tasks/HANDOFF_TOPOSORT_RUNBOOK_AUTHORING.md` (COMPLETED) — precedent for Chat-Claude-to-Clair work-item HANDOFF.

Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + grep guardrail scope discipline.
