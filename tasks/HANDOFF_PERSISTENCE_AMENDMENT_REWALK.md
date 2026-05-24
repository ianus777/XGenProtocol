# Handoff — Persistence Amendment Re-Walk (Y-lock, (a).iii.β → (a).iii.α)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-24 (Status flipped ACTIVE → COMPLETED v1.1 at Track 1 atomic-commit ship per J-107. Substantive scope of the re-walk closed: Y-lock revert (a).iii.β → (a).iii.α landed at Clair's Commit 2 `f4f0e4e`; Commit 2a Q2+Q3 return-vector landed at `c88fd73`; Commit 3 sentinel-tree refinement + verify landed at `a677244` with 8/8 GREEN verification rigour + three within-Commit-3 audit gaps closed atomically per D-077 first worked instance (abort-fold + identity-registry-persist + space-event-store-persist; federation-registry-persist audited and confirmed safe); Track 1 canonical-record amendments shipped at this eight-file atomic commit per D-074 tenth instance. **D-077 promoted** to DECISIONS.md as new principle: bidirectional sustainability discipline at silent-discard / fallible-discard sites. **Track-1-while-Clair-active first project instance** allowed because Track 1 here was record-of-already-locked-decision (Y-lock made in conversation with Clair at session close before Track 1 was authored), not decision-input — directional asymmetry from topo-sort J-099/J-100 Track-1-as-decision-input precedent. Body §1–§6 stays authoritative as historical record of re-walk-at-lock-time per topo-sort + bidirectional + persistence-amendment audit precedents. Bridge-handoff `tasks/HANDOFF_TRACK_1_SESSION_CLOSE_2026-05-24.md` folded into Track 1 atomic as eighth file per anti-tempfile-deletion-of-decision-records discipline; documents the mid-Track-1 session-close bridge mechanism as first project instance of pattern. **Next-active for Clair**: Commit 4 milestone close per runbook §6 — freezes four milestone-close J-NNN sites, flips runbook v1.1 → v1.2 COMPLETED, flips design doc v1.2 → v1.3 header chain, bumps ROADMAP v1.22 → v1.23, flips CLAUDE.md PLAY block to "Phase 9 Commit 3b RESUMES" per Q4(a) Commit-3b-1-collapse lock from J-105. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 (this commit's promotion) discipline.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — What happened

### §1.1 Sequence

1. **J-104** (2026-05-23) — persistence-amendment audit doc shipped; sentinel-tree authored at `xgen-node/src/tests/` (uncommitted; sentinel-state per Q4(a) lock).
2. **J-105** (2026-05-23) — design phase shipped. Four Joe-locks recorded: **Q1 → (a).ii + (a).iii.β + candidate D-NNN flag** (sort-on-replay + `ingest_event` returns `Result<(), GraphError>`); Q2 → (a) return-vector; Q3 → all three drain helpers; Q4 → (a) sentinel-tree in-scope. Sustainability question forced reframing of Q1 from (a).iii.α (log-level) to (a).iii.β (type-level) after "is this future-proof?" challenge surfaced three forward-drift surfaces (a).iii.α doesn't catch.
3. **J-106** (2026-05-23) — implementation runbook shipped at `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` ACTIVE v1.0. Five-commit Clair-facing sequence with six runbook-structural Joe-locks.
4. **Clair pickup at Commit 1** — doc-pass shipped clean (`0ca29e6`, pushed). Standing by for Joe-lock checkpoint #2 (pre-Commit-2 unit-test list).
5. **Joe-lock checkpoint #2** — Clair proposed 5 tests with renames from runbook §4.8 seed. Joe locked all 5 including test #4 (`dispatch_event_logs_and_rejects_when_ingest_event_returns_unknown_prev_event`) with shape (a) mod-tests-internal field mutation.
6. **§4.9 sentinel-tree gap surfaced** — Clair traced `phase9_drop_and_recover.rs` references `spawn_in_process_node_with_state` + `InProcessNode::shutdown_keep_data` not present in `phase9_harness.rs`. Structural rework per §5.2; not Commit 2 scope. Joe-lock: Option C package-scoped Commit 2 verification (NOT `cargo test --workspace`); workspace deferred to Commit 3; §4.9 amendment flagged for Commit 4 J-NNN as within-milestone runbook correction (sibling-shape to J-098/J-099).
7. **Test #4 dropped at Clair's surface** — Clair traced shape (a) structurally infeasible (`validate_event` Step 9 and `graph.add_event` consult same `EventStore.contains()` in single-threaded flow; no interleaved mutation point). Joe locked Option (c) drop. Final 4-test list confirmed: tests 1, 2, 3, 5.
8. **Commit 2 implementation in flight** — Clair shipped 8 of 9 Commit 2 items, hit `cargo test` fail on `node::runtime::phase_7_5_tests::b3_federation_add_via_federation_skips_step_9_predecessor`.
9. **CRITICAL — Phase 7 B3 cross-milestone dependency surfaced.** B3 amendment (J-088, exchange.rs:455-509) explicitly skips `validate_event` Steps 9/11/13 for `state.federation_add` via federation channel (predecessor-chain deadlock). **B3 implicitly relied on silent-discard at `let _ = graph.add_event(...)` as a feature** — `SpaceState.federation_nodes` mutated via `apply_event` even though DAG insert failed. Q1(a).iii.β `?` propagation broke this. Clair stopped per Rule 6.

### §1.2 The B3 finding in full

Phase 7 B3 amendment (J-088, locked 2026-05-20 at `xgen-core/src/message/exchange.rs:455-509`) explicitly skips Step 9 (predecessor presence), Step 11 (sender registration + membership), and Step 13 (permission) of `validate_event` for `state.federation_add` events arriving via federation channel. The B3 inline comment names this as "predecessor-chain deadlock" — the federation_add IS the relationship-establishing event whose own predecessors are themselves held on the Phase 7.5 federation-relationship trigger.

What B3 implicitly relied on but did not name in its locked design: `graph.add_event` inside `ingest_event` returns `UnknownPrevEvent` for this event class (because `validate_event` let it through with missing predecessors), and the silent-discard at `let _ = graph.add_event(...)` swallowed it, after which `let _ = store.insert(...)` and `apply_event(...)` ran and mutated `SpaceState.federation_nodes`. **Net pre-Commit-2 behaviour: SpaceState updates correctly, but the event lands in EventStore-but-not-DagGraph — a coherence violation that B3 silently treated as acceptable.**

Q1(a).iii.β replaces the silent-discard with `?` propagation. `SpaceState` never updates. B3 broken.

### §1.3 The J-105 sustainability frame, in retrospect

The J-105 sustainability question that produced (a).iii.β asked: "is (a).iii.α future-proof?" and surfaced three **forward-drift** risks:

1. Future caller bypasses `validate_event`
2. Disk format change
3. Future async-predecessor protocol revision

The question that would have caught B3 is the **backward-coherence** version: "does any current caller in the codebase depend on this silent-discard as a feature?" The J-105 walk asked the forward-only question and missed the backward question.

**This is the discipline lesson surfaced by this milestone**: sustainability questions must be asked in both directions. Forward-drift (future callers) AND backward-coherence (current callers). The bidirectional sustainability discipline is what would have caught B3 at design phase before Q1 locked (a).iii.β.

### §1.4 Five options walked at Joe-lock surface

Five resolution options walked with Joe:

- **(α) Exempt B3 path inside ingest_event** — rejected on principle. Re-introduces conditional silent-discard. Violates Q1(a).iii.β's compiler-forced-honest-handling discipline.
- **(β) Refactor B3 out of ingest_event** — cleanest long-term separation (~80 lines new helper + dispatch_event branching). Acknowledged but expensive.
- **(γ) Accept regression; schedule re-design phase** — honest but maximally expensive (Phase 7 cold-start bootstrap regresses).
- **(δ) Test-only public surface** — rejected (permanent escape hatch).
- **(ε) Tristate `Result<IngestOutcome, GraphError>` with `Ok(DagSkipped)`** — ~40-50 lines preserving compiler-honesty + new tristate variant.
- **(ζ) Ship (ε) + flag broader audit as future work** — ~40-50 lines + docs. Initially considered the right answer.

Then Joe's framing of "expensiveness = code = error-loop risk" produced **Option X vs Option Y**:

- **Option X** (apply bidirectional sustainability broadly to related code surfaces): 80-200 lines across 4-7 sites; 2-4 cascading session-arcs as each silent-discard surface gets walked; multi-site blast radius.
- **Option Y** (revert to (a).iii.α log-level; document broader audit as future work): ~5-10 lines just `tracing::error!` at the silent site; near-zero new error surface; forward-drift risks return but they're hypothetical future-contributor problems, not present-day concrete drift.

**Joe locked Y** on error-loop-risk grounds. The forward-drift risks (a).iii.α doesn't catch get named in documentation as future-walk material; they're future-contributor problems not present-day concrete drift. Bidirectional sustainability discipline gets named in DECISIONS.md as new principle (sibling-shape to D-076 v1.1 / Rule 0 / D-075 origin pattern) but applied surface-driven per D-071, not preemptively.

---

## §2 — What the re-walk changes

### §2.1 Substantive code changes (Track 2 — Clair)

1. **Q1(a).iii.β → Q1(a).iii.α**. `ingest_event` signature stays binary-void: `pub fn ingest_event(&mut self, event: Event)`. No `Result`, no tristate. Signature is unchanged from pre-Commit-2 state.
2. **At the `let _ = graph.add_event(...)` site**: replace with match arm calling `tracing::error!` on `Err` and continuing. Silent-discard becomes loud-discard. B3 keeps working because no structural change.
3. **Q1(a).ii sort-on-replay at `replay_spaces_from_dir`** stays unchanged. Defensive layer holds.
4. **51 test-fixture `.expect()` updates Clair shipped: REVERT** to original `let _ = node.ingest_event(...)`. No Result to expect.
5. **Locked unit test list drops from 4 to 2.** Tests 3 (`replay_spaces_from_dir_topologically_sorts_before_ingest`) and 5 (`topological_sort_publicly_reachable_from_xgen_node`) survive intact. Tests 1, 2, 4 lose their Result-shape regression target and drop.
6. **`topological_sort` re-export to xgen-node** stays per runbook §4.6. Still required for sort-on-replay.
7. **`GraphError` visibility: no action needed.** Already `pub` per Clair's earlier code-trace; never referenced from a public signature now that the Result is gone.

### §2.2 Verbatim code-comment block at the `graph.add_event` call site

Replaces the Checkpoint #4 locked block in the runbook §4.3. Full content:

```rust
// Q1(a).iii.α — tracing::error on graph.add_event failure, continue.
//
// Phase 7.5 persistence-amendment milestone (J-NNN). The Q1 lock at
// design doc §3 was originally (a).iii.β (Result<(), GraphError> at
// the signature, compiler-forced caller handling) but reverted to
// (a).iii.α (log-level vigilance) at implementation when the
// cross-milestone Phase 7 B3 amendment dependency surfaced:
// state.federation_add events arriving via federation channel
// intentionally have missing predecessors (B3 §4.1 predecessor-chain
// deadlock; xgen-core/src/message/exchange.rs:455). The B3 amendment
// relied on this site's silent-discard as a feature — the federation_add
// event lands in EventStore + mutates SpaceState.federation_nodes even
// though graph.add_event returns UnknownPrevEvent. Result-propagation
// would have broken B3 at the SpaceState mutation layer.
//
// FUTURE WORK (candidate D-NNN — "ingest path invariant encoding under
// bidirectional sustainability discipline"): this site, plus the four
// other silent-discard sites in ingest_event (event_id-missing-return,
// store.insert silent, two apply_event silents), plus the three drain
// helpers' silent-discards, plus any reject paths that swallow event-
// acceptance failures, all share the same discipline question: under
// what circumstances may a fallible operation discard its error? The
// audit must be bidirectional — forward-drift (future callers bypass
// upstream validation) AND backward-coherence (current callers depend
// on the silent as a feature). Both questions must be asked at every
// site simultaneously, because closing any one silent in isolation can
// break a cross-milestone semantic dependency (B3 at this milestone is
// the worked example).
//
// Scope of the future walk: re-audit ingest_event's five silents +
// the three drain helpers + the M6 reject paths + B3's apply_event
// dependency, simultaneously, under the bidirectional sustainability
// frame. Do NOT close any one silent in isolation. Promotion of
// candidate D-NNN to D-NNN happens when (a) Joe locks the walk as
// worth pursuing, OR (b) dependent work (M6 admin write path, M8
// federation depth, future cold-start refactor) surfaces a concrete
// drift instance log-level vigilance does not catch.
//
// Rungs above (a).iii.α at the design level (recorded for future-walk
// reference; not promoted at this milestone):
//   - (a).iii.β — Result<(), GraphError> compiler-forced handling
//   - ValidatedEvent wrapper — type-constructor discipline
//   - Sealed traits + visitor pattern — new-caller shape constraint
//   - Formal verification — machine-checked invariants
match graph.add_event(&event, store) {
    Ok(()) => {}
    Err(e) => {
        tracing::error!(
            event = "graph_add_event_failed",
            space_id = %space_id,
            event_id = %event.event_id.as_deref().unwrap_or("(none)"),
            error = %e,
            "graph.add_event returned error; event continues to store + apply_event \
             per (a).iii.α + Phase 7 B3 amendment (federation_add bootstrap case)"
        );
    }
}
```

### §2.3 Verification rigour at Commit 2 — package-scoped per §4.9 gap

NOT `cargo test --workspace`. The sentinel-tree files (`phase9_harness.rs`, `phase9_three_node_anti_transitivity.rs`, `phase9_drop_and_recover.rs`, `mod.rs`) reference `spawn_in_process_node_with_state` + `InProcessNode::shutdown_keep_data` which don't exist until Commit 3's structural rework lands. Workspace-test will fail to compile at the sentinel-tree layer; that's expected per the §4.9 amendment.

Run instead:

- `cargo test -p xgen-core --lib` — must pass clean
- `cargo test -p xgen-node --lib` (filtered to exclude the four sentinel files) — must pass clean
- `cargo test -p xgen-client --lib` — must pass clean
- `cargo test -p xgen-common --lib` — must pass clean
- **`cargo test -p xgen-core --lib b3_federation_add_via_federation_skips_step_9_predecessor`** — must pass green. This was Clair's trigger; it locks the revert correctness.

Workspace verification (full 8 green runs minimum per runbook §5.3) defers to Commit 3 after sentinel-tree structural rework lands. The §4.9 DoD amendment for package-scoped verification at Commit 2 + workspace deferred to Commit 3 gets recorded in Commit 4's milestone-close J-NNN entry as within-milestone runbook correction (sibling-shape to J-098/J-099 honest-framing class).

---

## §3 — What the re-walk amends (Track 1 — Chat Claude + Joe)

Shape-2 in-place amendments to canonical-record artifacts. Sibling-shape to topo-sort J-099 Step-2 eight-file atomic commit + J-100 Step-3 runbook revision. Track 1 lands as one atomic commit per D-074.

### §3.1 Design doc amendment

**File**: `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md`

**Amendments**:

1. **§3 amendment**: record (a).iii.β considered + B3 cross-milestone finding + revert to (a).iii.α + bidirectional sustainability framing. New "Amendment (2026-05-23)" subsection between "Decision" and "Originating incident" (sibling-shape to D-076 v1.1 in-place amendment pattern). Original §3 prose stays authoritative as historical record of design-at-lock-time.

2. **§8 amendment**: candidate D-NNN expanded scope — "ingest path invariant encoding under bidirectional sustainability discipline". Five `ingest_event` silents + three drain helpers + M6 reject paths + B3 apply_event dependency all in scope of the future walk. MUST be walked simultaneously, not piecemeal. Promotion trigger: Joe-lock OR dependent work surfaces concrete drift.

3. **Header `Last updated` paragraph**: chain a re-walk entry in front of J-105 design-close entry. Reference this HANDOFF.

4. **Version bump**: v1.0 → v1.1 (sibling-shape to topo-sort design doc v1.0 → v1.1 at J-099).

### §3.2 DECISIONS.md new entry — D-077 (or next available)

**New entry**: "D-077 — Sustainability question, both directions (bidirectional sustainability discipline)"

Sub-sections:

- **Decision**: At every silent-discard, conditional-mutation, or fallible-operation-with-discard pattern in the codebase, the sustainability question MUST be asked in both directions: (a) forward-drift — what future callers could bypass this; (b) backward-coherence — what current callers depend on this as a feature. Both questions answered simultaneously before closing any single silent in isolation.
- **Originating incident**: J-105 design phase locked Q1 at (a).iii.β under forward-only sustainability frame; implementation surfaced cross-milestone Phase 7 B3 dependency on the silent-discard as a feature; (a).iii.β reverted to (a).iii.α at implementation; bidirectional discipline named at this milestone.
- **Worked example**: Phase 7 B3 amendment dependency (this milestone). Forward-drift question caught three risks; backward-coherence question would have caught the B3 dependency before (a).iii.β locked. Asked at design phase, the bidirectional question would have surfaced the cross-milestone dependency before code shipped.
- **Application scope**: Surface-driven per D-071. Future audits trigger when (a) Joe locks the walk OR (b) dependent work surfaces concrete drift. NOT pre-applied retroactively across the codebase; applied at audit-phase opening of milestones that touch fallible-discard patterns.
- **Sibling discipline family**: D-067 (code-organisation) + D-070 (transport-layer correlation pair) + D-075 (event-model) + D-076 v1.1 (wire-format) + Rule 0 (session-open). Each principle originated from a discipline failure that surfaced during implementation. D-077 originates from this milestone's cross-milestone B3 dependency surface.
- **Relationship to candidate D-NNN "Ingest path invariant encoding"**: candidate D-NNN is the *application* of D-077's principle to the specific ingest-path silent-discard family. D-077 is the discipline; candidate D-NNN is the first scheduled future-walk under that discipline.

### §3.3 Runbook §4 amendment

**File**: `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md`

**Amendments**:

1. **§4.1 scope**: rewritten to (a).iii.α framing. Narrow scope holds — Q1 covers only `graph.add_event` Result-handling at this site, NOT the other four silent-discard sites. The five silents and three drain helpers + M6 reject paths + B3 dependency all candidate-D-NNN-grouped under D-077 bidirectional sustainability discipline.

2. **§4.2 the signature change**: removed. `ingest_event` signature stays binary-void.

3. **§4.3 the verbatim code-comment block**: replaced with §2.2 content above.

4. **§4.4 GraphError visibility**: amended — no action needed (already pub; never referenced from public signature now).

5. **§4.5 dispatch_event call site**: removed Result-handling. Stays at `self.ingest_event(event);` shape.

6. **§4.6 replay_spaces_from_dir sort-on-replay**: stays unchanged. `topological_sort` re-export still needed.

7. **§4.7 test-fixture caller updates**: reverted. Test fixtures use `let _ = node.ingest_event(...)` per pre-Commit-2 shape.

8. **§4.8 seed unit tests**: 4 → 2 tests. Tests 3 + 5 survive. Tests 1, 2, 4 drop.

9. **§4.9 DoD checklist**: amended for package-scoped verification at Commit 2 (NOT `cargo test --workspace`); workspace verification deferred to Commit 3.

10. **§4.10 anti-drift guardrails**: re-framed for (a).iii.α + bidirectional sustainability future-walk discipline.

11. **§7 discipline notes**: new sub-section §7.8 added — "(a).iii.β → (a).iii.α revert" recording the discipline lesson + naming bidirectional sustainability discipline as origin of D-077.

12. **Header `Last updated` paragraph**: chain a re-walk entry in front of J-106 runbook-authoring entry.

13. **Version bump**: v1.0 → v1.1.

### §3.4 CLAUDE.md PLAY block + header

**Amendments**:

1. **Header `Last updated`**: chain a re-walk entry in front of J-106 entry. Frame: bidirectional sustainability discipline named at this re-walk; (a).iii.β reverted to (a).iii.α; D-077 promoted to DECISIONS.md; Track 1 (this commit) + Track 2 (Clair Commit 2 revised) ship in parallel per topo-sort J-099/J-100 Step 2/Step 3 split precedent.

2. **PLAY block**: stays "Persistence-amendment Commit 1 ✅; Clair pickup at Commit 2 revised ←── HERE". The PLAY block doesn't flip away from Clair-Commit-2 yet — she's still picking up; only the milestone-close commit flips PLAY to Phase 9. Re-walk amendments to canonical record stay alongside PLAY.

### §3.5 JOURNAL.md re-walk entry

**New entry**: J-NNN (next available) — "Persistence-amendment re-walk Step 2: design doc + DECISIONS.md + runbook amended in place; D-077 promoted; bidirectional sustainability discipline named"

Sub-sections (sibling-shape to topo-sort J-099):

1. **Header**: re-walk close
2. **Sub-section 1**: B3 finding retrospective — what surfaced at Clair's Commit 2 implementation
3. **Sub-section 2**: five options walked + Y-lock decision
4. **Sub-section 3**: D-077 promotion + sibling family enumeration
5. **Sub-section 4**: candidate D-NNN expansion (ingest_event + drain helpers + reject paths + B3 apply_event)
6. **Sub-section 5**: discipline pattern naming — bidirectional sustainability question
7. **Sub-section 6**: Track 1 (this commit, amendments) + Track 2 (Clair Commit 2 revised) parallel-ship per topo-sort J-099/J-100 precedent
8. **Sub-section 7**: D-074 application count — tenth instance at this re-walk Track 1 commit
9. **Sub-section 8**: "Honest longer work over fast shortcuts" recurrence count — stays inherited at eighth per within-milestone Shape-2 amendment framing (NOT incremented at this re-walk surface)
10. **Sub-section 9**: discipline-notes data points — three Clair-surfaced findings at Commit 2 implementation (sentinel-tree gap + test #4 infeasibility + B3 dependency); runbook author audit (me) missed all three in their respective ways

### §3.6 ROADMAP.md amendment

**File**: `docs/ROADMAP.md`

**Amendments**:

1. **Version**: v1.21 → v1.22 (sibling-shape to topo-sort v1.14 → v1.15 at J-099 Step 2).
2. **Visual structure tree**: persistence-amendment sub-cluster — Commit 1 ✅ stays; Commit 2 row gets `(revised)` annotation indicating Y-lock revert.
3. **Cross-cutting principles section**: D-077 row added (bidirectional sustainability discipline); candidate D-NNN row updated with expanded scope (ingest_event + drain helpers + reject paths + B3).
4. **Past section**: re-walk entry added under persistence-amendment sub-cluster.
5. **Header `Last updated` paragraph**: chain a re-walk entry.

### §3.7 Atomic commit per D-074

All Track-1 files in ONE atomic commit:

1. `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` — §3 + §8 amendments + header + v1.1
2. `DECISIONS.md` — new D-077 entry
3. `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` — §4 + §7 amendments + header + v1.1
4. `CLAUDE.md` — header bump + PLAY block context update
5. `docs/ROADMAP.md` — v1.21 → v1.22 + tree + Past + cross-cutting + header
6. `JOURNAL.md` — J-NNN re-walk entry
7. `tasks/HANDOFF_PERSISTENCE_AMENDMENT_REWALK.md` — Status flipped ACTIVE → COMPLETED v1.1 at re-walk close

Seven files. Sibling-shape to topo-sort J-099 eight-file atomic commit (one fewer because this HANDOFF lives at re-walk-only layer, not Step-2/Step-3 split).

---

## §4 — Track 1 / Track 2 split

Sibling-shape to topo-sort J-099/J-100 Step-2/Step-3 split. The re-walk has two parallel tracks:

### §4.1 Track 1 — Chat Claude + Joe — canonical-record amendments

**Scope**: §3 amendments above. Atomic commit per D-074.

**Sequence**:
1. New Chat Claude session opens.
2. Per Rule 0: read CLAUDE.md PLAY block + latest JOURNAL entry + this HANDOFF.
3. Author the seven-file atomic commit per §3.
4. Verify all `J-NNN` placeholders consistent across files.
5. Verify `Filesystem:get_file_info` on each file written before moving to the next (per J-098/J-099 prose-then-batch discipline).
6. User pushes via GitHub Desktop or PowerShell.

**Does NOT block**: Track 2. Clair ships Commit 2 revised in parallel; Track 1 lands alongside.

### §4.2 Track 2 — Clair — Commit 2 revised

**Scope**: §2 substantive code changes + §2.3 verification.

**Sequence**:
1. Clair receives the Joe-lock response (already drafted; sent at session-end).
2. Reverts current Commit 2 working tree to pre-(a).iii.β state.
3. Applies the seven substantive lock changes per §2.1.
4. Lands the verbatim code-comment block per §2.2.
5. Runs package-scoped verification per §2.3.
6. Ships Commit 2 revised.
7. User pushes.

**Does NOT block**: Track 1. Track 1's canonical-record amendments land alongside Commit 2 revised.

---

## §5 — Definition of done

- [ ] Track 1 atomic commit ships per §3.7 seven-file atomic commit per D-074
- [ ] Track 2 Commit 2 revised ships per §2 with package-scoped verification per §2.3 green
- [ ] Phase 7 B3 test (`b3_federation_add_via_federation_skips_step_9_predecessor`) passes green at Commit 2 revised — load-bearing signal
- [ ] D-077 promoted to DECISIONS.md with sibling-discipline-family framing per §3.2
- [ ] Candidate D-NNN scope expanded at design doc §8 per §3.1 amendment
- [ ] Verbatim code-comment block at `graph.add_event` site matches §2.2 verbatim content
- [ ] All `J-NNN` placeholders frozen to actual J-numbers at Track 1 ship
- [ ] Runbook §4.9 amendment landed per Commit 4 J-NNN milestone-close entry (deferred to Commit 4; not Track 1's scope)
- [ ] HANDOFF Status flipped ACTIVE → COMPLETED v1.1 at Track 1 commit close
- [ ] **NEXT-ACTIVE for Clair after Track 2 ships**: Joe-lock checkpoint #3 (post-Commit-2 / pre-Commit-2a primitive shape locked)
- [ ] **NEXT-ACTIVE for Chat Claude after Track 1 ships**: standby until Clair's Commit 2a closes

`Status: COMPLETED` header line is the unflippable success signal. Per D-065 + D-067 + D-069 + D-071 + D-074 + D-077 (this re-walk's origin) + Rule 0 + "Honest longer work over fast shortcuts" discipline.

---

## §6 — Cross-references

- `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md` — audit doc (COMPLETED v1.1 at J-105)
- `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` — design doc (ACTIVE v1.0 → v1.1 at Track 1 ship)
- `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` — implementation runbook (ACTIVE v1.0 → v1.1 at Track 1 ship)
- `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md` — sibling-in-shape Step-3 HANDOFF precedent (COMPLETED v1.1 at J-100)
- `DECISIONS.md` D-065 + D-067 + D-069 + D-070 + D-071 + D-074 + D-075 + D-076 v1.1 + (new) D-077
- `JOURNAL.md` J-088 (Phase 7 B3 amendment) + J-093 + J-095..J-106 (the eight precedents this re-walk inherits discipline-pattern from)
- `xgen-core/src/message/exchange.rs:455-509` — Phase 7 B3 amendment code site
- `xgen-core/src/node/runtime.rs:~210` — the `graph.add_event` silent-discard site (where the verbatim code-comment block lands)
- `xgen-node/src/tests/phase9_*.rs` (four sentinel-tree files) — uncommitted sentinel state per Q4(a) lock

---

*End of HANDOFF.*  
