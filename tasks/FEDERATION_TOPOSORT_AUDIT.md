# Federation Topological-Sort Wire-Order Non-Determinism Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-22 (All three audit-phase questions locked at design-phase opening session: Q1 at Shape A v1 + sibling Site 1 fix; Q2 at Q2 middle + Q2.γ (forward-binding to Node-to-Client siblings); Q3 at Q3.ii (canonical wire ordering required). New D-NNN drafted for DECISIONS.md promotion at design-phase close, sibling-distinct from D-067 + D-075, pairs with D-070 as no-drift-surface discipline family. Sections updated: §1.1 + §6 intro + §6.1 + §6.2 + §6.3 + §5.1 + §5.2 + §5.3 + §7.5 + §8. Implementation runbook (`tasks/FEDERATION_TOPOSORT_IMPL.md`, to be authored) sits behind the design task file (`tasks/FEDERATION_TOPOSORT_DESIGN.md`, to be authored). Four atomic commits expected per bidirectional precedent: doc-pass; primitive + sibling fix + unit tests; Phase 9 Scenario 1 lift; milestone close per D-074. Sibling shape to bidirectional audit §6.2 (code-verified-yes pattern — lock at audit-doc lifecycle, not carry into design-phase deliberation).)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

This document is the canonical record of a protocol-level audit finding surfaced during Phase 9 Scenario 1's post-D-075 verification (JOURNAL.md J-096 Finding 2): **`topological_sort_events` in `xgen-node/src/fanout.rs` produces non-deterministic wire-order for DAG-root events because it has no tie-break rule for ready siblings AND its input feed comes from `HashMap.values()` iteration. Two `xgen-node` processes with identical Space state produce different federation-delta wire orderings ~50% of runs. When `state.room_create` wins the race against `state.space_create`, B rejects it ("space not found"), cascading rejections through the bootstrap chain and timing Scenario 1 out at the per-event budget.**

It is a subsystem audit per D-071 — produced before any design-phase work begins, so the design walkthrough that follows has a code-grounded shared baseline to walk against. Sibling-in-shape to `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` (Status: COMPLETED v1.0), which preceded the bidirectional `federation_nodes` design phase. The pattern is: dependent work surfaces a load-bearing protocol gap → audit documents the gap with code-grounded evidence and candidate fix shapes → design phase walks the candidates and locks a fix → implementation runbook ships it → dependent work resumes.

This document does not lock a fix. Locking is the design phase's job. This document's job is to surface the gap precisely, ground it in actual code, and frame the option space cleanly enough that the design walkthrough runs efficiently.

### 1.1 Position in the milestone

This audit sits between the bidirectional `federation_nodes` milestone closure (2026-05-21, HEAD `827303d`) and the topological-sort implementation runbook (next-active). Phase 9 Scenario 1 was lifted at Commit 3 of the bidirectional milestone and re-stood-down at Commit 4 on this finding (`#[ignore]`-annotated with inline doc comment naming this audit's forward-reference). Phase 9 Commit 3b (Scenarios 2 + 3, plus the compound scenarios) is paused inside the Federation Event Propagation milestone scope until the topological-sort fix lands.

Pass 1 implementation (`tasks/XGID_RETROFIT_PASS_1_IMPL.md`, Status: ACTIVE v2.0) remains downstream of Federation Event Propagation milestone closure. The locked Shape A v1 is Pass-1-neutral; Pass 1 stays at v2.0 unchanged.

**All three audit-phase questions locked 2026-05-22** at the design-phase opening session (see §6 + §6.1 + §6.2 + §6.3): Q3 at Q3.ii (canonical wire ordering required); Q2 at Q2 middle + Q2.γ (fix primitive's contract once; forward-bind to Node-to-Client siblings); Q1 at Shape A v1 + sibling Site 1 fix. New D-NNN lands in DECISIONS.md at design-phase close (number TBD), sibling-distinct from D-067 + D-075, pairs with D-070 as the no-drift-surface discipline family.

### 1.2 Reading order

1. JOURNAL.md J-096 Finding 2 — Clair's diagnostic write-up with verbatim evidence (timestamp comparison, outcome tally, 105-`dispatch_event`-count parity).
2. This document, §3 → §4 → §6 → §7 → §7.5 (the framing-and-options chain).
3. `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` — structural template; matches this audit's section depth and discipline-note framing.
4. `xgen-node/src/fanout.rs` (lines 193-220 + 311-333) — the two sites named in §3.
5. `xgen-core/src/node/runtime.rs` (lines 859-912) — the canonical sibling sort that the federation path's sort should mirror.

---

## 2. Provenance — how the gap surfaced

The bidirectional `federation_nodes` milestone closed 2026-05-21 in a four-commit sequence (J-096). Commit 3 (`f051039`) lifted Phase 9 Scenario 1's `#[ignore]` annotation and added `#[serial_test::serial]`. The first task at Commit 4's session open was to verify the Scenario 1 pass holds across multiple isolated runs before shipping the milestone-close commit.

### 2.1 The four-check sanity sequence

The pre-Commit-4 verification ruled out four upstream causes before settling on the genuine finding:

1. **4/4 pre-reboot runs failed** at exactly ~122s (the 120s per-event budget plus overhead). Hypothesis 1 (single-run transient): ruled out by repeat.
2. **Machine reboot** to clear OS-level state drift (file handles, port reservations, async runtime residue). Post-reboot result was mixed across multiple isolated runs at the original 120s budget — about 50% pass at 5.34s / 50% fail at ~122s, with no middle ground. Hypothesis 2 (OS-level drift) partially helped but did not eliminate the flake.
3. **Clean cargo rebuild** (`cargo clean && cargo test -p xgen-node --lib tests::phase9_two_node_smoke::tests::two_node_federation_push_smoke_100_messages -- --include-ignored`). Hypothesis 3 (stale build artefacts): ruled out — the flake persisted at the same ~50% rate.
4. **Time-pressure hypothesis test.** Three per-event/per-condition budgets in `phase9_two_node_smoke.rs:261` (10s → 60s), `:343` (120s → 300s) and `phase9_harness.rs:380, :386` (10s → 60s each) were temporarily relaxed. 5 isolated runs sampled. Outcome: 4 fail / 1 pass with the failures hitting **exactly** the new 300s ceiling (301.96 / 301.97 / 301.94 / 301.98 — all within ~0.05s of the budget). The relaxation did not reveal slow-but-eventual arrival; it simply delayed the panic by the extra time. Hypothesis 4 (time pressure): ruled out. **Events truly never drain in failing runs.**

### 2.2 The instrumented run that pinpointed `topological_sort_events`

Diagnostic instrumentation was then added on the xgen-node side (visible to `traced_test`, which filters out `xgen_core::*` targets and would have hidden a `tracing::debug!` placed inside `dispatch_event` itself). Two `tracing::warn!` lines bracketed the `dispatch_event` call in `process_inbound` — one before naming `event_type`, `event_id`, `space_id`, `sender`, `peer_node_id_for_f3`, `prev_events`, and the current `federation_nodes` snapshot for the target Space; one after naming the returned `DispatchOutcome` variant.

Three runs (1 pass + 2 fail) gave the conclusive data (verbatim from J-096):

| Run | Result | dispatch_event calls | Accepted | HeldPending | Rejected |
|---|---|---|---|---|---|
| 1 | PASS (5.34s) | 105 | 102 | 3 | 0 |
| 2 | FAIL (121.94s) | 105 | 2 | 101 | 2 |
| 3 | FAIL (121.98s) | 105 | 2 | 101 | 2 |

**Decisive evidence: 105 dispatch calls in all three runs.** Every event reaches `dispatch_event` on B with identical event-type distribution (1 `state.space_create` + 1 `state.room_create` + 1 `state.federation_add` + 1 `membership.invite` + 1 `membership.join` + 100 `message.text`). The bidirectional fix's drain hook is exercised correctly. The divergence is in the **wire order** of events arriving at B.

### 2.3 The wire-order divergence

In the PASS run, the bootstrap arrival sequence on B was `state.space_create` (29.484313Z, accepted) → `state.room_create` (29.491389Z, accepted) → `membership.invite` → `membership.join` → `state.federation_add` → 100 messages. Standard happy path.

In the FAIL run, the sequence was `state.room_create` (35.246931Z, `federation_nodes=(no-space-state)`, **Rejected with "space not found"**) → `state.space_create` (35.247021Z, accepted) → `state.federation_add` (accepted) → `membership.invite` (HeldPending — its `prev_events` reference the now-missing `room_create_id`) → `membership.join` (HeldPending — refs `invite_id` which is buffered) → `message_1` (its `prev_events=[federation_add_id]` are present, but Step 11 fails: "sender is not a member of room 'xgen://hash/sha256:488a901d...'" because Alice's join never applied — **Rejected**) → `message_2..100` (each refs the previous message, the chain is buffered behind the rejected `message_1` — **HeldPending**). Tally checks out exactly: 2 Accepted (`space_create` + `federation_add`) + 2 Rejected (`room_create` + `message_1`) + 101 HeldPending (`invite` + `join` + 99 chained messages) = 105.

### 2.4 Root-cause trace to source

`build_room_create_event` in `xgen-core/src/space/state.rs:797-820` constructs the event with `vec![]` for `prev_events` (DAG-root semantic). `state.space_create` also has empty `prev_events`. Both are roots.

In `compute_federation_delta_for_space` at `xgen-node/src/fanout.rs:311-333`, the local store contents are collected via `store.values().cloned().collect()` (non-deterministic HashMap iteration per Rust's `RandomState`) and then passed to `topological_sort_events` at `xgen-node/src/fanout.rs:193-220`. That function uses a single-pass scan over the input vector, removing events whose predecessors are already emitted; for tied root events (both ready from iteration 1), the algorithm preserves **input order**, which is therefore non-deterministic.

Sibling function `topological_sort` in `xgen-core/src/node/runtime.rs:859-912` (used for in-process ordering, separate code path) uses Kahn's algorithm with explicit `queue_vec.sort()` for stable tie-breaking. The xgen-node-side delta function does not.

### 2.5 Bidirectional fix verified not implicated

All evidence agrees: identical `dispatch_event` call count across pass/fail; the unit-level mirror test `apply_federation_add_two_vantages_mirror` from bidirectional Commit 2 remains green; the divergence is upstream of the dispatcher in delta serialisation. The bidirectional milestone closure (Commit 4, `827303d`) is correct and the fix stands. This audit's finding is a separate pre-existing bug surfaced during verification, NOT implicated by D-075.

---

## 3. The mechanism, code-verified

Two sites in `xgen-node/src/fanout.rs` compound into one symptom. Neither is wrong in isolation.

### 3.1 Site 1 — `compute_federation_delta_for_space:321`

```rust
let store = match rt.stores.get(space_id) {
    Some(s) => s,
    None => return Vec::new(),
};
let all: Vec<Event> = store.values().cloned().collect();   // line ~321
drop(rt);
let sorted = topological_sort_events(all);
```

`EventStore` is `HashMap<String, Event>`. Rust's default hasher is randomized per HashMap instance. Two Nodes with identical Space state produce different input orderings to the sort. **Non-determinism enters here.**

### 3.2 Site 2 — `topological_sort_events:193`

```rust
pub fn topological_sort_events(mut events: Vec<Event>) -> Vec<Event> {
    let mut emitted: HashSet<String> = HashSet::new();
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut changed = true;
    while !events.is_empty() && changed {
        changed = false;
        let mut i = 0;
        while i < events.len() {
            let ready = events[i].prev_events.iter().all(|p| {
                emitted.contains(p) || !events.iter().any(|e| e.event_id.as_deref() == Some(p))
            });
            if ready {
                let ev = events.remove(i);
                if let Some(id) = &ev.event_id { emitted.insert(id.clone()); }
                out.push(ev);
                changed = true;
            } else { i += 1; }
        }
    }
    out.extend(events);
    out
}
```

Single-pass scan: emit events whose predecessors are already emitted. **No tie-break.** Ready siblings (including DAG roots with empty `prev_events`) emit in input-vector order. The function's silent contract is "input is already meaningfully ordered" — but its caller feeds it `HashMap.values()`. **Non-determinism laundered through to wire order.**

### 3.3 Combined outcome

`state.space_create` and `state.room_create` both have empty `prev_events`. Both ready at iteration 1. Relative order = HashMap iteration order = random ~50% per run. See §2.2 evidence table and §2.3 wire-order divergence for the resulting cascade.

### 3.4 What B does (not implicated)

105 `dispatch_event` calls in pass AND fail runs. Bidirectional fix's six unit tests (incl. `apply_federation_add_two_vantages_mirror`) green. **Divergence is entirely upstream of the dispatcher.**

### 3.5 The sibling that does it right

`xgen-core/src/node/runtime.rs::topological_sort` (lines 859-912) — Kahn's algorithm with explicit `queue_vec.sort()` and `next.sort()` at every level. Two topo-sort implementations in the codebase: one canonical, one not. Federation delta path uses the non-canonical one. **D-067 drift surface in flight** — §6 Q2 walks scope.

---

## 4. The relationship to the design as locked

`tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.3.1 R4: "sorted-by-`space_id` cross-Space ordering for determinism." R4 is correct at the cross-Space layer.

**R4 was silent on within-Space ordering.** The dimension was not on Phase 3's design surface — assumed handled by the topo-sort primitive. The primitive doesn't honour that assumption for tied events.

### 4.1 Why the locked design missed it

1. **R4's framing was cross-Space-only.** Within-Space was assumed-handled, not deliberated.
2. **The two topo-sort implementations were not cross-checked.** `xgen-core/src/node/runtime.rs:859-912` already had stable tie-break when Phase 3 began. `xgen-node/src/fanout.rs:193-220` didn't. Asymmetry not surfaced as a design question.
3. **Test surface gap.** No test before Phase 9 Scenario 1 Commit 3 (`f051039`, 2026-05-21) exercised full bootstrap delivery with two `NodeRuntime` instances + non-trivial state. Bug was there the whole time.

### 4.2 The principle revealed

> **Wire-order determinism is a normative protocol property and must be locked explicitly, not assumed to emerge from local primitives.**

Anticipated at the cross-Space level by R4. Not anticipated at the within-Space level. Same "considered one dimension, missed another" shape as the bidirectional audit's §4.

Audit analogue: **All sender-side code paths that produce wire-visible ordering must be canonical, not merely correct.** "Correct" topo-sort respects causality. "Canonical" topo-sort additionally produces byte-identical output for byte-identical input sets.

### 4.3 Catalogue extension

Phase 9 catalogue (J-091, 14 entries) has no row for delta-serialisation-order non-determinism between Nodes. Design phase adds: **"Sender-side wire-order non-determinism between Nodes."**

---

## 5. Scope

### 5.1 In scope for the implementation runbook that follows

- **`topological_sort_events` tie-break (the primitive fix).** Add `events.sort_by(|a, b| a.event_id.cmp(&b.event_id));` at the top of each outer-loop iteration in `xgen-node/src/fanout.rs:193`. Code-comment block at the sort site per §6.1's verbatim shape.
- **`compute_federation_delta_for_space:321` sibling Site 1 fix.** Sort `Vec<Event>` before passing to `topological_sort_events`. Belt-and-braces: explicit canonical-ordering chain end-to-end.
- **Code-comment block at the sort site** citing D-NNN + Appendix J's content-hash framing (per §6.1).
- **Unit tests at the primitive level**, verifying:
  - deterministic output across input permutations;
  - stable tie-break for ready siblings with empty `prev_events`;
  - no-op-equivalence for already-canonically-ordered input.
- **Phase 9 Scenario 1 re-resurrection.** Lift `#[ignore]` again; becomes activating integration-level regression lock for D-NNN.
- **D-NNN promotion to DECISIONS.md** at milestone close, sibling-distinct from D-067 + D-075, pairs with D-070 as the no-drift-surface discipline family.
- **Catalogue row addition** per §4.3 in `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`.
- **Coordinate with all existing Federation Event Propagation milestone locks** — Phase 3 a-i symmetry rule + R4, Phase 4 origin gating, Phase 7 A1 + B1 + B3, Phase 7.5 P7.5-A through P7.5-D, D-075 bidirectional vantage rule. The fix must not regress any of these.

### 5.2 Out of scope for this audit and the implementation runbook

- **`collect_sync_history` (`xgen-node/src/fanout.rs`) — Q3.ii-analogue, flagged for future scheduling.** Node-to-Client wire output; same bug class (HashMap.values() feed at cross-Space iteration). Per Q2.γ forward-binding, Q3.ii applies here "where analogous" and the site should be reviewed when scheduling allows.
- **`apply_fanout` history-push (`xgen-node/src/fanout.rs`) — Q3.ii-analogue, flagged for future scheduling.** Node-to-Client wire output; same bug class.
- **EventStore container type changes (Shape D.1 territory).** Out-of-scope; right home would be a separate milestone on EventStore canonical-iteration discipline if ever scheduled.
- **Audit of every `HashMap` iteration site in the codebase.** Q2 middle locked at primitive layer; Q2 wide considered and rejected (§6.2).
- **Promoting Q3.ii to a spec-normative Ch3 statement.** D-NNN in DECISIONS.md is sufficient at this phase. Spec promotion is its own future decision.
- **Changing `compute_federation_delta_for_space`'s API contract.** Function signature stays unchanged; only the internal `HashMap.values()` feed gets the sort.
- **Test-surface restructuring beyond Phase 9 Scenario 1's regression-witness role.** Phase 9 Commit 3b's scope.
- **Stress-test follow-on coverage.** The four deferred compound scenarios in `tasks/FEDERATION_STRESS_FOLLOWON.md` remain blocked on the clock-injection seam.

### 5.3 Non-scope decisions explicitly recorded

| Item | Why deferred | Where it lands |
|---|---|---|
| Codebase-wide `HashMap.values()` audit | Q2 middle locked at primitive layer; per-site audit beyond the two named Q3.ii-analogues is not in scope | Separate work item if a future milestone needs it |
| Q3.ii spec-normative entry (Ch3 / Ch4 / Appendix) | D-NNN entry is the canonical lock; spec-level promotion is separate doc-pass work | Future doc-pass if a contributor needs spec-level reference |
| `collect_sync_history` + `apply_fanout` fix | Q2.γ forward-binding flags them; out of this milestone per Q2 middle scope discipline | Their own design discussion against their own consumer pressure |
| Migration of any persisted federation deltas | None exist (no production deployments) | Out-of-scope |

---

## 6. Audit-phase questions — ALL LOCKED 2026-05-22

All three audit-phase questions resolved at the design-phase opening session. The audit records them as locked rather than open, matching the bidirectional audit's pattern of recording Q2 as code-verified-yes inline rather than carrying it into design-phase deliberation.

- **Q1 (§6.1)** — Tie-break choice: **Shape A v1 + sibling Site 1 fix.**
- **Q2 (§6.2)** — D-067 audit scope: **Q2 middle + Q2.γ** (fix primitive's contract once; forward-bind to Node-to-Client siblings).
- **Q3 (§6.3)** — Wire-format normative question: **Q3.ii** (canonical wire ordering required).

The implementation runbook walks the locked design forward; this audit's role transitions from "input to design-phase deliberation" to "canonical record of the locked design." Status remains ACTIVE through the implementation runbook ship; flips to COMPLETED at milestone close per the bidirectional audit's lifecycle precedent.

### 6.1 Q1 — Tie-break choice (LOCKED 2026-05-22: Shape A v1 + sibling Site 1 fix)

**Locked answer: Shape A v1 + sibling Site 1 fix.**

- **Tie-break source:** event_id lexicographic sort at `topological_sort_events`, applied to ready siblings at each iteration of the outer loop.
- **Sibling Site 1 fix:** sort the `Vec<Event>` at `compute_federation_delta_for_space:321` before passing to `topological_sort_events`. Belt-and-braces: explicit canonical-ordering chain end-to-end matching Q2 middle's letter (primitive fixed + feed canonical).
- **Pass-1 posture:** v1 — `&str` sort with code-comment block at the sort site flagging Pass 3 retype to `EventXgid`. Pass-1-neutral; preserves Pass 1's Status ACTIVE v2.0 unchanged.

**Reasons recorded at lock:**

- **Q2 middle's letter** wants the primitive fixed at the primitive layer; Shape A v1 does exactly that with minimum footprint.
- **Shape C's duplicate-event_id argument is hypothetical.** Content-hash-derived event_ids cannot collide except through SHA-256 break, which is its own different bug. Shape C's contract-layer explicit-canonicality is achieved by Shape A's code-comment block + D-NNN citation without the cross-crate dep or performance cost.
- **Shape D.1's structural-depth argument is real but overscopes the milestone.** Touches every EventStore consumer for a problem the milestone's surfaced bug doesn't require. D.1's right home would be a separate milestone on EventStore canonical-iteration discipline if ever scheduled.
- **Shape A v2's type-level guarantee is real but couples this milestone to Pass 1's `EventXgid` flavour.** Pass 1 is currently blocked behind Phase 9 + Federation milestone close + this topo-sort milestone; coupling here adds dependency surface. v1 with the wrap-or-comment precedent established by XGID Adoption v1 Commit 2 + bidirectional Commit 2 is the consistent posture.

**Code-comment block at the sort site (verbatim shape; exact phrasing is implementation latitude):**

```rust
// D-NNN wire-order determinism (locked at topological-sort
// design-phase close [date]; sibling-distinct from D-067 at
// code-organisation layer and D-075 at event-model layer; all
// three lock no-drift-surface properties explicitly).
//
// Sort ready siblings by event_id lexicographically. event_id is
// content-hash-derived per Appendix J (xgen_appendix_j_en.md), so
// the sort key is byte-stable across senders with identical Space
// history, which is exactly what D-NNN's "two senders with
// identical state produce byte-identical federation deltas"
// contract obligates.
//
// v1 ships with &str sort; Pass 3 retypes to EventXgid when
// xgen-node-side dispatch widens to XGID flavours. The retype is
// purely type-level; sort semantics unchanged.
```

**D-NNN framing (to land in DECISIONS.md at design-phase close):**

> Wire-order determinism is a sender-side normative property for Node-to-Node federation; two senders with identical Space history produce byte-identical federation deltas modulo signature-bearing fields. Wire ordering is part of the protocol's contract, not implementation latitude. Forward-bound by Q2.γ to Node-to-Client sender output where analogous.

Sibling-distinct from D-067 (code-organisation layer) and D-075 (event-model layer); pairs with D-070 (transport-layer correlation pair) as the no-drift-surface discipline family. First D-NNN to lock a wire-format-normative property explicitly. The binding it creates: future event-design Joe-locks must include "does this event's serialisation produce canonical wire ordering across senders" as a design-phase question.

**The four shapes considered (for completeness):**

- **Shape A v1 + sibling Site 1 fix (LOCKED).** Minimum footprint; Pass-1-neutral; primitive fix + feed canonical end-to-end.
- **Shape A v2 + sibling Site 1 fix (NOT LOCKED).** Type-level guarantee; couples to Pass 1's `EventXgid` flavour. Rejected on dependency-surface grounds.
- **Shape C + sibling Site 1 fix (NOT LOCKED).** Strongest contract; cross-crate dep cost; duplicate-event_id benefit hypothetical. Rejected on cost-vs-benefit grounds.
- **Shape D.1 alone (NOT LOCKED).** Data-layer fix; overscopes milestone. Rejected on milestone-scope grounds; right home would be a separate EventStore-discipline milestone.
- **Shape B + Shape D.2 (DISQUALIFIED earlier).** Disqualified by Q3.ii at lock-time (§6.3).

### 6.2 Q2 — D-067 audit scope (LOCKED 2026-05-22: Q2 middle + Q2.γ)

**Locked answer: Q2 middle + Q2.γ — fix the primitive's contract once; Q3.ii scoped to Node-to-Node federation with explicit forward-binding to Node-to-Client siblings.**

**Fix scope (what's IN this milestone):**

- Fix `topological_sort_events` so it produces canonical output regardless of input ordering. The primitive's contract changes from "respects causality" to "canonical, given a fixed event set."
- Fix the §3.1 sibling site (`compute_federation_delta_for_space`'s `HashMap.values()` feed) so federation delta is Q3.ii-compliant end-to-end.

**Q3.ii forward-binding (what's flagged but OUT of this milestone):**

- `collect_sync_history` (`xgen-node/src/fanout.rs`) — Node-to-Client wire output; same bug class; flagged for future scheduling against its own consumer pressure.
- `apply_fanout` history-push (`xgen-node/src/fanout.rs`) — Node-to-Client wire output; same bug class; flagged for future scheduling.

Both sites are recorded in §5.2 out-of-scope with the Q3.ii-analogue tag. Future Chat Claude + Joe revisiting either site picks up the Q3.ii framing already locked.

**Reasons recorded at lock:**

- **Matches D-067 + D-070 + D-075 locking pattern.** Lock the principle where it's load-bearing today; forward-bind to sibling surfaces; schedule siblings against their own consumer pressure. D-067 was locked at single-source-of-truth for derived state reads (not a codebase-wide HashMap audit). D-070 was locked at transport-layer correlation pair (not at every wire envelope). D-075 was locked at the vantage-aware applier for `state.federation_add` (not at every relationship-shaped event). This audit's Q2 middle + Q2.γ follows the same discipline shape.
- **The primitive fix architecturally satisfies D-067 at the topo-sort surface.** Every future consumer gets canonical output by default. No per-caller drift surface possible at this layer. The drift surface (§3.5) between `topological_sort_events` and the canonical sibling `xgen-core/src/node/runtime.rs::topological_sort` is closed.
- **Q2 wide would couple unrelated scope into a federation milestone.** Client-facing surfaces (`collect_sync_history`, `apply_fanout`) belong in their own design discussion with their own consumer-pressure framing. The project has consistently scoped milestones tightly.
- **Q2 narrow under Q2.α would be honest scope discipline but leaves a discipline-pattern inconsistency.** D-067/D-070/D-075 all forward-bound siblings explicitly. Q2 narrow would lock Q3.ii at federation only with no forward-binding language, which the project's discipline pattern argues against.

**Q2.γ framing recorded:**

Q3.ii applies to Node-to-Node federation today, with explicit forward-binding that the principle applies to Node-to-Client sender output **where analogous** and should be reviewed when scheduling allows. The design doc's D-NNN entry includes the forward-binding language so future event-design or wire-format discussions inherit the principle.

**Shape-space implications under Q2 middle.** The admitted shapes from Q3.ii (Shape A + sibling Site 1 fix; Shape C alone; Shape D.1 alone; Shape A v2 + Site 1 fix) all remain admissible under Q2 middle. Q2 middle doesn't further narrow shapes; it constrains what the implementation must touch (the primitive + Site 1, with sibling sites flagged but out-of-scope).

**The three readings considered (for completeness):**

- **Q2 narrow (NOT LOCKED).** Fix `topological_sort_events` + sibling §3.1 feed only; other known-leaky sites deferred indefinitely. Considered and rejected on grounds of leaving discipline-pattern inconsistency with D-067/D-070/D-075 forward-binding pattern.
- **Q2 middle (LOCKED).** Fix the primitive's contract once; per-site surfacing for known-leaky siblings becomes its own design question outside this milestone.
- **Q2 wide (NOT LOCKED).** Fix every wire-visible-ordering site in one pass. Considered and rejected on grounds of coupling unrelated scope into a federation milestone; client-facing surfaces belong in their own design discussion.

### 6.3 Q3 — Wire-format normative question (LOCKED 2026-05-22: Q3.ii)

**Locked answer: Q3.ii — canonical wire ordering required.** Two senders with identical Space history MUST produce byte-identical federation deltas (modulo signature-bearing fields that vary by author and time). Wire ordering is part of the protocol's contract, not implementation latitude.

**Reasons recorded at lock (Joe-locked 2026-05-22, design-phase opening):**

- **D-067 wire-format analogue.** The project has consistently locked no-drift-surface properties explicitly rather than trusting them to emerge from local primitives. D-068's five-site CLI Audit closure, M5's 13-verb consolidation, D-070's two-events-with-correlation, D-075's vantage-aware applier all instantiate the same posture. A wire-format-determinism property fits the same family; locking it explicitly is in keeping with the rest of the project's discipline.
- **MLS coupling.** Ch3 §3.10 + D3 parallel-workstream milestone require canonical wire ordering at the application layer. Locking Q3.i would surface this as a late-stage discovery, exactly the shape D-071 audit-precedes-dependent-design was created to prevent.
- **Cross-Node debugging benefit is immediate, not forward-only.** "Do these two senders' deltas match byte-for-byte?" becomes a yes/no question available from today, not from MLS landing.
- **Catalogue alignment.** §4.3's catalogue row name ("Sender-side wire-order non-determinism between Nodes") already implicitly assumes Q3.ii framing; locking Q3.ii aligns the lock with the catalogue row.

**The two readings considered:**

- **Q3.i — Per-receiver determinism sufficient (NOT LOCKED).** Each receiver independently runs a deterministic topological sort on the events it accumulates. Two receivers consuming the same events from two different senders end up with the same local DAG (causality preserved) but may have observed the events in different orders at the wire layer. Considered and rejected on grounds that wire ordering becomes implementation latitude, MLS coupling surfaces as late-stage discovery, and the project's no-drift-surface discipline (D-067, D-068, D-070, D-075) consistently locks normative properties at the protocol layer rather than leaving them as receiver-local concerns.
- **Q3.ii — Canonical wire ordering required (LOCKED).** Two senders with identical state MUST produce byte-identical deltas (modulo signature-bearing fields). Wire ordering is itself a protocol invariant.

**New D-NNN to land in DECISIONS.md at design-phase close** (number TBD), sibling-distinct from D-067:
- D-067 at the code-organisation layer (single source of truth for derived state reads).
- D-NNN at the wire-format layer (single source of truth for sender-side wire output).

Both lock no-drift-surface properties explicitly. Pairs with D-070 (transport-layer correlation pair) + D-075 (event-model vantage rule) as the no-drift-surface discipline family. This is the first D-NNN to lock a wire-format-normative property explicitly. The binding it creates: future event-design Joe-locks must include "does this event's serialisation produce canonical wire ordering across senders" as a design-phase question. That cost is deliberate, not incidental.

**Q3.ii narrows admissible shapes per §7.** Admissible: Shape A + sibling Site 1 fix; Shape C alone; Shape D.1 alone; Shape A v2 + Site 1 fix. Disqualified: Shape B (timestamp non-canonical across senders); Shape D.2 (insertion order non-canonical across Nodes). See §7.5 for the post-lock admissibility column.

### 6.4 Subordinate questions

- **Q4** — Replay determinism on Node restart (Shape D only).
- **Q5** — Sort cost profile (Shape C primarily).
- **Q6** — Forward compat with future EventTypes that emit empty `prev_events` (all shapes).
- **Q7** — Confirm catalogue row name per §4.3.

---

## 7. Candidate fix shapes

Four shapes, increasing structural depth. Design phase picks one with Joe-lock.

### 7.1 Shape A — `event_id` sort at topo primitive

**Mechanism.** Add `events.sort_by(|a, b| a.event_id.cmp(&b.event_id));` at the top of each outer-loop iteration in `topological_sort_events`. Optional sibling: sort input at §3.1 before passing (required under Q3.ii).

**Cost.** One line. Sort cost O(n log n) per iteration — negligible at typical delta sizes.

**Benefit.** Smallest footprint. Pure-function property restored at the primitive.

**Pass-1 coupling.** event_id → EventXgid under Pass 1. Two v-options:

- **v1** — `&str` sort at v1, retype to `EventXgid` under Pass 3 with code-comment block flagging the future retype. Pass-1-neutral.
- **v2** — typed `EventXgid` sort from outset. Pass-1-coupled: requires Pass 1's `EventXgid` to land before this shape ships, OR ships with `Xgid::new(event.event_id.clone())` wrap pattern matching bidirectional Commit 2's `SpaceLocalMetadata.introducer_node_id` precedent.

Design phase picks v-option against Pass 1 coordination posture.

**Q3.** Admissible under both readings (Q3.ii requires sibling §3.1 fix).

### 7.2 Shape B — Timestamp sort

**Mechanism.** Same as A but sort key is `event.timestamp`.

**Cost.** Millisecond collisions realistic → secondary tie-break needed (collapses to A or C). Wall-clock non-canonical across senders.

**Benefit.** Semantically meaningful in logs.

**Q3.** Admissible under Q3.i only. **Disqualified under Q3.ii.**

### 7.3 Shape C — Canonical-event-bytes sort

**Mechanism.** Same as A but sort key is canonical wire bytes via `xgen-core::wire::canonical`.

**Cost.** Serialises every event per comparison (cacheable). Cross-crate dependency `xgen-node` → `xgen-core::wire::canonical`.

**Structural note.** Canonical bytes include `event_id` by construction (per Appendix C primitive schema). For distinct-`event_id` events, canonical-bytes lexicographic order ≈ `event_id` lexicographic order. Shape C's additional benefit over Shape A is concentrated in the case of duplicate-`event_id` events (which the protocol does not currently emit but could in principle under future EventType extensions), and in the explicit-canonicality property at the primitive contract layer (a documentation/auditability benefit beyond the wire-output equivalence for current EventTypes).

**Benefit.** Strongest canonical guarantee. Maximal D-067 closure at the primitive.

**Q3.** Admissible under both readings.

### 7.4 Shape D — Container change at EventStore

**D.1 — `BTreeMap<String, Event>` keyed by event_id.** Canonically sorted iteration. HashMap feed at §3.1 becomes deterministic at source. `topological_sort_events` needs no change.

**D.2 — `IndexMap` insertion-ordered.** Deterministic per process; insertion order non-canonical across Nodes. **Disqualified under Q3.ii.**

**Cost (D.1).** Every `EventStore.{get,values,contains,insert}` call site verified for BTreeMap semantics. API-compatible; perf shifts O(1) → O(log n). Replay-on-disk: BTreeMap reconstructs canonical order regardless of disk order.

**Benefit (D.1).** Closes bug class at the data layer. Q2-wide naturally satisfied. Future consumers automatically canonical.

**Pass-1 coupling.** Indirect — EventStore keys are event_id strings.

**Q3.** D.1 admissible under both readings. D.2 disqualified under Q3.ii.

### 7.5 Summary table (post-Q3.ii lock)

| Shape | Tie-break source | Code change | Q3.ii admissibility | D-067 reach | Pass-1 coupling |
|---|---|---|---|---|---|
| A | event_id (lex) | 1 line + sibling §3.1 fix required | ✅ admitted | Partial — primitive only | v1 Pass-1-neutral; v2 coupled |
| B | timestamp | n/a | ❌ disqualified (wall-clock non-canonical across senders) | n/a | n/a |
| C | canonical bytes | 1 line + cross-crate dep | ✅ admitted (strongest) | Partial maximal | Implicit |
| D.1 | event_id at data layer | EventStore container swap | ✅ admitted | Wide | Indirect |
| D.2 | insertion order | n/a | ❌ disqualified (insertion order non-canonical across Nodes) | n/a | n/a |

Four shapes remain admissible under Q3.ii: A (with mandatory sibling §3.1 fix), C, D.1, and A v2 (variant of A). Cost-benefit still distributed across the four. Design phase picks one against the Q1 + Q2 frame.

---

## 8. Phase-9 + milestone implications

### 8.1 Phase 9 Scenario 1 regression witness

Scenario 1 at `xgen-node/src/tests/phase9_two_node_smoke.rs` is `#[ignore]`-annotated with an inline doc comment forward-referencing this audit. When the fix lands, the `#[ignore]` lifts (sibling to bidirectional fix's Commit 3 pattern). The scenario becomes the activating integration-level regression lock for D-NNN — any future change that re-introduces wire-order non-determinism will fail Scenario 1.

### 8.2 Phase 9 Commit 3b

Stays paused inside the Federation Event Propagation milestone scope. Same pause shape as the bidirectional fix. Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10 all unblock when the topological-sort fix lands and Scenario 1 lifts a second time.

### 8.3 Compound scenarios + stress follow-on

The four deferred compound scenarios in `tasks/FEDERATION_STRESS_FOLLOWON.md` (C1 / C4 / C6 / C8) remain independently blocked on the clock-injection seam. Unaffected by this audit.

### 8.4 M6 (new) + Pass 1 dependency chain

M6 (new) Node admin write path remains 🟡 PENDING; Pass 1 of XGID Retrofit remains 🟡 PENDING at Status: ACTIVE v2.0. Both unblock simultaneously when Phase 9 closes (which unblocks when this milestone closes). The dependency chain extends by one node (this milestone), unchanged in shape.

### 8.5 Pass 1 coupling posture

Shape A v1 is Pass-1-neutral. The `&str` sort at v1 + Pass 3 retype-marker code-comment block is the consistent precedent (XGID Adoption v1 Commit 2 `SpaceLocalMetadata.introducer_node_id` + bidirectional Commit 2 `Xgid::new(...)` wrap). Pass 1 runbook stays at v2.0 unchanged.

---

## 9. Discipline notes

This audit is a worked instance of three project-management principles already in DECISIONS.md:

- **D-069 (canonical-document rule).** This document is the canonical record of the topological-sort audit finding. Future references to the finding cite this document.
- **D-071 (subsystem audits precede dependent milestones).** The audit runs before the design phase that fixes the gap. The design phase has this document as its Pass 1 input, mirroring how the bidirectional design phase had `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` as its input.
- **D-074 (milestone-close commits include JOURNAL).** When the design phase closes, its closure commit will include a JOURNAL entry. When the implementation runbook closes, that closure commit will also include a JOURNAL entry.

### 9.1 Fourth recurrence of the audit→design→impl→close pattern

This is the project's fourth instance of the audit-precedes-dependent-work pattern: J-081 Propagation Reliability Audit → Federation Event Propagation design phase (first); Phase 7.5 design task file → Phase 7.5 implementation runbook (second); Bidirectional `federation_nodes` audit → design → implementation arc just closed (third); this audit → topological-sort design → implementation → Phase 9 Scenario 1 re-resurrection (fourth, in flight).

The pattern is now repeating-pattern rather than one-off. Each substantial bug class that survives the unit-test surface and surfaces at the deployment-level integration test becomes its own phase. D-071's natural cadence is visible: integration-level testing exercises code paths that surface protocol gaps invisible to unit testing.

### 9.2 "Honest longer work over fast shortcut" — fourth instance

The fastest path at J-096 Finding 2's surface would have been to ship Commit 4 of the bidirectional milestone with a `#[serial_test::serial]` annotation as a workaround and a hand-wave acknowledgement of the flake ("known intermittent issue, follow up later"). The chosen path — full diagnostic walk, root-cause traced to source, verbatim evidence captured in J-096, separate phase opened per D-071, Scenario 1 re-stood-down as the regression witness for the new fix — is longer than the workaround AND shorter than rediscovering the bug from a production deployment.

The diagnostic specifically found the bug **upstream** of where the bidirectional fix's symptom appeared to land. The honest framing — "the bidirectional fix is verified correct; this is a separate pre-existing bug that the new test surface revealed" — required acknowledging that the first instinct ("the bidirectional fix is incomplete") was wrong. Same shape as the Phase 9 Scenario 1 → bidirectional finding arc that the just-closed milestone resolved.

### 9.3 D-067 finer-grain instance

The bidirectional audit recorded D-067 as the "single source of truth, no drift surface" principle. This audit's Q2 framing is a worked instance of D-067 at a finer grain: two implementations of the same primitive (topological sort) exist in the codebase, one canonical, one not. Two sites are now confirmed leaky on the wire-output-determinism dimension: `topological_sort_events` (the primary find) and `collect_sync_history` (named in §6.2's Q2 audit list, flagged by its own comment block). More may exist; Q2-wide reading would discover them. The design phase decides whether to close the drift surface at the primitive (Q2 middle), at every consumer (Q2 wide), or only at the activating site (Q2 narrow).

### 9.4 Catalogue extension

The Phase 9 failure-mode catalogue (J-091, 14 entries) named protocol-level deadlocks and validation asymmetries. The topological-sort wire-order non-determinism does NOT map cleanly to any single catalogue entry — it is a category of bug that the catalogue did not anticipate. Phase 9's value as a deployment-stress surface is the ability to find this kind of bug too. The catalogue should be extended in the design phase to add the row per §4.3 so the next contributor reading the catalogue finds the category.

---

## 10. Cross-references

### 10.1 Design documents

- **`docs/xgen_federation_propagation_design.md`** (Status: ACTIVE, v1.0) — the canonical Federation Event Propagation design. §6.4 Phase 7 Lock A1 + B1, §6.4.1 Phase 7.5 P7.5-A through P7.5-D, §6.4.2 Phase 8 bidirectional vantage-aware applier (D-075), §15 Implementation Complete table.
- **`tasks/FEDERATION_PROPAGATION_COMPLETION.md`** (Status: ACTIVE, v1.0) — the implementation runbook. §3.3 Phase 3 wire shape, §3.3.1 R4 (sorted-by-`space_id` cross-Space ordering — anticipated cross-Space dimension, did not surface within-Space dimension; see §4 above).
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md`** (Status: COMPLETED v1.0) — structural template for this audit. Same ten-section shape; same discipline-note framing.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md`** (Status: COMPLETED v1.0) — sibling-shape precedent for the next-active design phase. D-075 locked here.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`** (Status: COMPLETED v1.1) — sibling-shape precedent for the implementation runbook that follows the design phase.
- **`tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md`** (Status: COMPLETED v1.0) — earlier sibling-shape audit→design precedent.
- **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** (Status: ACTIVE v1.0) — Phase 9 implementation task file. Scope intact; Scenario 1 re-stood-down on this finding; Commit 3b paused.

### 10.2 Code surfaces

- `xgen-node/src/fanout.rs::topological_sort_events` (lines 193-220) — Site 2 of §3, the non-canonical sort primitive.
- `xgen-node/src/fanout.rs::compute_federation_delta_for_space` (lines 311-333; HashMap feed at ~321) — Site 1 of §3.
- `xgen-node/src/fanout.rs::collect_sync_history` (lines ~225-275) — Q2 audit list, known-leaky on cross-Space iteration.
- `xgen-node/src/fanout.rs::apply_fanout` (lines ~115-200) — Q2 audit list, history-push path.
- `xgen-core/src/node/runtime.rs::topological_sort` (lines 859-912) — **canonical sibling sort precedent.** Kahn's algorithm with explicit `queue_vec.sort()` and `next.sort()` tie-breaking. The reference site for any of Shapes A/B/C.
- `xgen-core/src/node/runtime.rs::all_events` (lines ~920-930) — uses canonical sort; reference site.
- `xgen-core/src/space/state.rs::build_room_create_event` (lines 797-820) — `prev_events: vec![]` (DAG-root semantic).
- `xgen-core/src/space/state.rs::build_space_create_event` — `prev_events: vec![]` (DAG-root semantic).
- `xgen-node/src/tests/phase9_two_node_smoke.rs::two_node_federation_push_smoke_100_messages` — Phase 9 Scenario 1 regression witness (`#[ignore]`-annotated, doc-comment forward-references this audit).
- `xgen-core/src/dag/store.rs` — `EventStore` definition (`HashMap<String, Event>`); relevant for Shape D.

### 10.3 JOURNAL

- **J-096** — the originating record of the finding. Finding 2 (topological-sort discovery section) is the canonical evidence source. Quote verbatim per Rule 2.
- J-088 (Phase 7 closure) — F-3 implementation context.
- J-093 (Phase 7.5 design closure) — sibling precedent for the design-phase pattern.
- J-094 (Phase 7.5 implementation closure) — sibling precedent for the implementation-phase pattern.
- J-095 (XGID Adoption v1 implementation milestone closure) — D-074 first instance.

### 10.4 DECISIONS

- **D-067** — Single source of truth / no drift surface. The principle whose finer-grain instance Q2 walks (§6.2, §9.3).
- **D-068** — Original "drift surface" decision; precursor to D-067's framing.
- **D-069** — Canonical-document rule. This document is the canonical home for the topological-sort audit finding.
- **D-071** — Subsystem audits precede dependent milestones. The discipline this document follows (§9.1).
- **D-074** — Milestone-close commits include JOURNAL. Forward-binding for the design + implementation phases that follow.
- **D-075** — Bidirectional vantage-aware applier rule. Sibling-distinct from this audit's eventual decision; the principle the just-closed bidirectional milestone locked.

---

*End of audit document. Design phase walkthrough is next-active for Chat Claude + Joe. The walkthrough resolves Q1 + Q2 + Q3 (§6.1, §6.2, §6.3) and picks one of the four candidate fix shapes (§7) with Joe-lock. The locked design phase produces its own task file `tasks/FEDERATION_TOPOSORT_DESIGN.md` (sibling to `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md`) that captures the framework decisions and may promote a new D-NNN to DECISIONS.md. After the design phase closes, the implementation runbook `tasks/FEDERATION_TOPOSORT_IMPL.md` is authored (sibling to `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`) and handed off to Clair. After the runbook ships, Phase 9 Scenario 1's `#[ignore]` lifts again and the topological-sort phase closes.*  
