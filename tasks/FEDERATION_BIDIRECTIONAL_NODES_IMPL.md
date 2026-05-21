# Federation Bidirectional `federation_nodes` Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-21 (Implementation runbook authored same-session as the design-phase close at `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md`. Four-commit sequence for Clair: doc-pass, origin-aware applier + plumbing + unit tests, Phase 9 Scenario 1 resurrection, milestone close. Wire-format-neutral. Pass-1-neutral. Sibling-in-shape to `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md`. Status flips ACTIVE → COMPLETED at the milestone-close commit (Commit 4) per the established implementation-runbook lifecycle. Per D-069 (canonical document) + D-074 (milestone-close commits MUST include JOURNAL.md) + D-075 (the protocol-design principle the locks instantiate).)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

This is the **implementation runbook** for the bidirectional `federation_nodes` phase of the Federation Event Propagation milestone. The design phase closed 2026-05-21 with three Joe-locks captured at `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` (Status ACTIVE v1.0 — flips COMPLETED in Commit 1 of this runbook): **Q1 Reading (i)** (one event, asymmetric interpretation), **Shape A** (origin-aware applier, wire format unchanged), **sub-option A.1** (re-derive on load — native fit verified at design close).

This document is Clair-facing — it sequences code-level work across four atomic commits.

The design task file is authoritative on **what** to build and **why**. This runbook is authoritative on **how** to ship it, **in what order**, with **what verification at each step**.

### 1.1 Reading order on session start

1. This document, §2 (sequence overview) — get the shape of the four commits.
2. Design task file §3 (Q1 lock) + §4 (Shape A + A.1 locks) + §4.2 (verification result) — re-read the three locks before touching code.
3. DECISIONS.md D-075 — the protocol principle the implementation instantiates.
4. Audit doc `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` §3 (the mechanism, code-verified) — refresh the file:line references for the code surfaces touched in Commit 2.
5. Canonical design doc `docs/xgen_federation_propagation_design.md` §6.4 (Phase 7 F-3 framework) — sibling subsection §6.4.1 is Phase 7.5's; §6.4.2 is the slot for this phase's summary that Commit 1 fills.
6. Then back to this document, §3 onward, for per-commit work.

### 1.2 Latitude

Implementation-internal decisions (test helper shapes, internal function organisation, code-comment phrasing within the verbatim-block guidance below, test fixture builders) are Clair's latitude. Wire-format-visible or signature-visible decisions require Joe-lock — pause and ask.

Concrete starting suggestions in this runbook (unit test names, code-comment wording, parameter ordering on the new `my_node_id`) are exactly that: starting points. Clair may revise if a cleaner option surfaces during implementation, with the constraint that the **vantage-aware applier semantic locked at design §4.1 is preserved verbatim**. The semantic of the fix is locked; the surface code shape around it is Clair's call.

---

## 2. Sequence overview

Four atomic commits, in this order. Each commit is shippable in isolation (workspace `cargo test` passes at each step). Hard ordering is documented in design task file §5.3 — Commit 2 must precede Commit 3 (the integration test passes only after the applier fix); Commit 1 must precede Commit 2 (the canonical record reflects locked design before code references it); Commit 4 must be last (milestone-close housekeeping happens after all code has shipped and verified).

| # | Commit | Scope | Code? | Test count change |
|---|---|---|---|---|
| 1 | Doc-pass | Canonical design doc §6.4.2 + §15 row; design task file flipped COMPLETED; audit doc flipped COMPLETED | No | 571 → 571 (no code) |
| 2 | Origin-aware applier + plumbing + unit tests | `apply_federation_add` signature + body; `apply_event` dispatch; two `NodeRuntime::ingest_event` call sites; six suggested unit tests | Yes | 571 → 571 + N (new unit tests) |
| 3 | Phase 9 Scenario 1 resurrection | Remove `#[ignore]` from `phase9_two_node_smoke.rs::scenario_1_two_node_push_smoke`; verify scenario passes against the fixed applier | Yes (annotation only) | 571 + N → 572 + N (one ignored → one passing) |
| 4 | Milestone close | CLAUDE.md PLAY block flip; ROADMAP.md state flip; JOURNAL.md entry; this runbook Status ACTIVE → COMPLETED; design task file Status preserved at COMPLETED (flipped in Commit 1); audit doc Status preserved at COMPLETED (flipped in Commit 1) | No | unchanged from Commit 3 |

**Test-count discipline.** N is not pre-locked. Each commit's DoD requires actual `cargo test --workspace` output quoting the new count. Do not invent numbers (CLAUDE.md Rule 5).

**Two pre-existing flakes carried forward** (from CLAUDE.md): precedence env-var race (D-068, commit 3e2f311); `reconnect_with_existing_tip_small_delta_delivered` (Phase 3 test under workspace parallelism). If either fires during this milestone's verification, retry once to confirm flake signature; do not treat as regression.

---

## 3. Commit 1 — Doc-pass commit

### 3.1 Scope

Documentation only. No code changes, no test changes. The purpose: make the canonical design doc and the two task-file Status headers reflect the Joe-locked state of the bidirectional `federation_nodes` phase before any implementation work begins. This is the canonical-document discipline from D-069 + the same-commit discipline from D-074 applied in advance.

### 3.2 Files touched

- `docs/xgen_federation_propagation_design.md` — add §6.4.2 sibling subsection (sibling to §6.4.1 which is Phase 7.5's); add §15 Implementation Complete table row for the bidirectional `federation_nodes` phase.
- `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` — header Status flipped ACTIVE → COMPLETED; Last updated bumped to commit date.
- `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` — header Status flipped ACTIVE → COMPLETED; Last updated bumped to commit date (audit's role as input to the design phase is over once the design closes; canonical record preserved).
- This runbook (`tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`) — header Last updated bumped (Status stays ACTIVE; flips at Commit 4).

### 3.3 §6.4.2 content sketch

The canonical design doc's §6.4 currently covers Phase 7's F-3 framework with Lock A1 (data source: `SpaceState.federation_nodes`), Lock B1 (skip for `state.federation_add`), and Lock B2 (self-establishing tightening deferred). §6.4.1 is Phase 7.5's sibling subsection (cold-start bootstrap closure with P7.5-A through P7.5-D). §6.4.2 is this phase's sibling subsection — same depth, same prose style.

Required content in §6.4.2:

1. **One-paragraph framing.** This phase closes the bidirectional `federation_nodes` finding surfaced during Phase 9 Commit 3a Scenario 1 diagnostic run. The finding: `apply_federation_add` reads `content.node_id` verbatim from every vantage, so receiver-side `federation_nodes` ends up populated with the wrong Node (the receiver itself instead of the asserter). F-3 then rejects every post-bootstrap event because the wire-authenticated peer is not in `federation_nodes`. Reference the design task file for the full Q1+Q2 framing and the rejected alternatives B/C/D.
2. **Lock summary.** Three locks: (a) Q1 Reading (i) — `state.federation_add` is one event recording one party's act, `federation_nodes` is a derived projection with vantage-aware applier logic; (b) Shape A — origin-aware applier with new `my_node_id: &str` parameter on `apply_federation_add`, wire format unchanged; (c) sub-option A.1 — re-derive on load (native fit because `SpaceState` is non-persisted per `NodeRuntime::new` initialising `spaces: HashMap::new()`).
3. **Code surfaces.** Name the three files touched in Commit 2 (`xgen-core/src/space/state.rs` applier + tests; `xgen-core/src/node/runtime.rs` two `ingest_event` call sites). Name the unit-level regression lock (six unit tests) and the integration-level regression lock (Phase 9 Scenario 1 resurrection at Commit 3).
4. **Cross-references.** D-075 (the protocol-design principle); design task file (the locks + reasoning); audit doc (the code-grounded mechanism). Three documents, one authority each.
5. **What this phase does NOT change.** Wire format unchanged. `EventOrigin` enum unchanged. `state.federation_add` content schema unchanged. Pass-1-neutral. Existing federation_add events on disk (test fixtures, dev-build state files) stay valid as-authored.

Length: six-to-eight paragraphs. Read §6.4 + §6.4.1 first and match their tone.

### 3.4 §15 Implementation Complete table row

§15 currently records Phases 1–8 (J-082 through J-089) plus Phase 7.5 (added by Phase 7.5's Commit 1 doc-pass). Add a "Bidirectional federation_nodes" row in chronological position (between Phase 7.5 and where Phase 9 will eventually land). Format matching existing rows: phase identifier | one-line description | commit count | test delta | journal entry reference.

Journal entry reference will not exist yet at Commit 1 time (the JOURNAL entry is written in Commit 4). Use `J-NNN+` placeholder or omit; the Commit 4 commit updates to the actual journal number when known. Match Phase 7.5's `J-094+`-style placeholder if that convention is in effect.

### 3.5 Audit doc Status flip rationale

The audit doc has been ACTIVE since 2026-05-21 because it was the input to the design phase. With the design phase closed and this implementation runbook ACTIVE, the audit doc's role transitions from "active input" to "historical canonical record of the finding." Status flips ACTIVE → COMPLETED in this commit; content unchanged.

This mirrors the sibling pattern: `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` flipped ACTIVE → COMPLETED in Phase 7.5's implementation runbook Commit 1. The flip belongs in the implementation runbook's first commit because that's when the design's role as "input" ends.

### 3.6 DoD for Commit 1

- [ ] §6.4.2 added to `docs/xgen_federation_propagation_design.md`, content per §3.3 above.
- [ ] §15 Implementation Complete row added.
- [ ] `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` header Status flipped ACTIVE → COMPLETED; Last updated bumped.
- [ ] `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` header Status flipped ACTIVE → COMPLETED; Last updated bumped.
- [ ] This runbook's header Last updated bumped (Status stays ACTIVE).
- [ ] `cargo test --workspace` passes with unchanged test count (no code touched).
- [ ] Commit message names this as "Commit 1 of 4 — doc-pass for bidirectional federation_nodes."
- [ ] No JOURNAL.md edit in this commit (Commit 4 carries the JOURNAL entry per D-074).

---

## 4. Commit 2 — Origin-aware applier + plumbing + unit tests

### 4.1 Scope

The fix itself. Implements Q1 Reading (i) + Shape A + sub-option A.1 in code.

### 4.2 Files touched

- `xgen-core/src/space/state.rs` — `apply_federation_add` signature + body; `SpaceState::apply_event` dispatch; six suggested unit tests in `mod tests`.
- `xgen-core/src/node/runtime.rs` — two `ingest_event` call sites where `SpaceState::apply_event` is invoked must pass `&self.node_id` through.

That's it. Three test files in `xgen-node/src/tests/` already touch federation_add events (`federation_push_integration.rs`, `cold_start_bootstrap_integration.rs`, `federation_relationship_integration.rs`) but they construct fixtures manually that already produce the correct `federation_nodes` content; Commit 2 does NOT need to touch them. (Whether any of them now exhibit different behaviour against the fixed applier is verified by `cargo test --workspace` — if a test fails, it was relying on the bug, and Commit 2's diff includes the test fix.)

### 4.3 `apply_federation_add` — the change

Pre-Commit-2 code (`xgen-core/src/space/state.rs:351-363`):

```rust
fn apply_federation_add(&mut self, event: &Event) -> Result<(), SpaceError> {
    if self.dm_constraints_active {
        return Err(SpaceError::DmFederationNotAllowed);
    }
    let node_id = event.content["node_id"]
        .as_str()
        .ok_or(SpaceError::MissingField("node_id"))?
        .to_string();
    if !self.federation_nodes.contains(&node_id) {
        self.federation_nodes.push(node_id);
    }
    Ok(())
}
```

Post-Commit-2 code (target):

```rust
fn apply_federation_add(
    &mut self,
    event: &Event,
    my_node_id: &str,
) -> Result<(), SpaceError> {
    if self.dm_constraints_active {
        return Err(SpaceError::DmFederationNotAllowed);
    }
    let content_node_id = event.content["node_id"]
        .as_str()
        .ok_or(SpaceError::MissingField("node_id"))?;
    // D-075 vantage-aware applier (locked at bidirectional federation_nodes
    // design phase 2026-05-21; see tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md §4.1).
    //
    // state.federation_add records one party's act: "asserter (sender) approves
    // other-party (content.node_id) as federation peer for this Space." The
    // event is asymmetric by construction. The applier reconstructs the
    // relevant peer from local vantage:
    //   - If I am content.node_id (someone else's federation_add naming me),
    //     the relevant peer to add is event.sender (the asserter).
    //   - Else (my own federation_add naming someone else, OR an unrelated
    //     federation_add I'm observing as a third-party with multi-Space
    //     visibility), the relevant peer is content.node_id.
    //
    // Both branches are needed: A authors a federation_add(B); both A and B
    // ingest it. A falls into the else branch (content.node_id=B, my=A); B
    // falls into the if branch (content.node_id=B, my=B). Both end with the
    // other party in federation_nodes; symmetric outcome through asymmetric
    // branches, driven by my_node_id.
    let peer_to_add = if content_node_id == my_node_id {
        event.sender.as_str()
    } else {
        content_node_id
    };
    let peer_string = peer_to_add.to_string();
    if !self.federation_nodes.contains(&peer_string) {
        self.federation_nodes.push(peer_string);
    }
    Ok(())
}
```

The verbatim code-comment block is locked content per the design phase. Phrasing may be edited for clarity, but the structural elements must be preserved: (a) D-075 reference, (b) design task file reference at §4.1, (c) one-line semantic statement of "event records one party's act," (d) explanation of both branches with concrete A/B example, (e) emphasis that vantage drives the branch selection. The block is load-bearing for future readers tracing the applier back to the principle.

### 4.4 `SpaceState::apply_event` dispatch — signature change

Pre-Commit-2 code (around `xgen-core/src/space/state.rs:354`):

```rust
pub fn apply_event(&mut self, event: &Event) -> Result<(), SpaceError> {
    match &event.event_type {
        EventType::StateRoomCreate => self.apply_room_create(event),
        EventType::StateFederationAdd => self.apply_federation_add(event),
        // ... other arms
    }
}
```

Post-Commit-2 code (target):

```rust
pub fn apply_event(&mut self, event: &Event, my_node_id: &str) -> Result<(), SpaceError> {
    match &event.event_type {
        EventType::StateRoomCreate => self.apply_room_create(event),
        EventType::StateFederationAdd => self.apply_federation_add(event, my_node_id),
        // ... other arms unchanged (other appliers do not need my_node_id at v1)
    }
}
```

Most existing arms (`apply_room_create`, `apply_invite`, `apply_join`, `apply_leave`, `apply_kick`, `apply_ban`, etc.) ignore the new parameter. That is intentional and matches D-075's scope statement: only `apply_federation_add` currently needs vantage-awareness; other appliers gain nothing by threading the parameter through. If Clair finds it cleaner to thread through anyway for uniformity (no opinion either way) — that's latitude. If a different applier surfaces a vantage-aware need in a future milestone, the parameter is already threaded.

### 4.5 `NodeRuntime::ingest_event` — two call sites to update

Pre-Commit-2 code (`xgen-core/src/node/runtime.rs` around lines 180-200, both `state.apply_event(&event)` call sites):

```rust
// Site 1 — around line 189
if let Some(state) = self.spaces.get_mut(&event.space_id) {
    state.apply_event(&event).ok();
}

// Site 2 — around line 197
state.apply_event(&event).ok();
```

Post-Commit-2 code (target):

```rust
// Site 1
if let Some(state) = self.spaces.get_mut(&event.space_id) {
    state.apply_event(&event, &self.node_id).ok();
}

// Site 2
state.apply_event(&event, &self.node_id).ok();
```

Both call sites have `self: &mut NodeRuntime` in scope, so `&self.node_id` is available. Pass it through. The `NodeRuntime.node_id: String` field is constructed at `NodeRuntime::new` from the keypair (line 127-130 of runtime.rs) and is the canonical "this Node's URI" used elsewhere in the runtime.

There may be additional `apply_event` call sites surfaced by `cargo build`. The complete list is exhaustively findable by grep against `apply_event(`; the runbook calls out the two production sites in `NodeRuntime::ingest_event` as the load-bearing ones. Test fixture call sites in `xgen-core/src/space/state.rs::tests` get updated in the same Commit 2 as Clair's test-pass work — many tests construct a `SpaceState` directly and apply events to it; the test fixtures pass an in-test `my_node_id_for_test` string (suggest a helper `fn local_node_id() -> &'static str` returning a fixed `"xgen://pubkey/ed25519:LOCAL"` test value).

Sites outside `xgen-core` (i.e. anywhere `xgen-node` or `xgen-client` calls `apply_event` directly) need the same treatment. The complete call-site enumeration is left to `cargo build` — every unfixed site fails compilation with a clear missing-argument error; the runbook does not enumerate them speculatively because the call-site count may have shifted since runbook authoring.

### 4.6 Six suggested unit tests

Add to `xgen-core/src/space/state.rs::mod tests`:

1. **`apply_federation_add_my_event_adds_content_node_id`** — A's vantage (A authors the event; A applies it). Setup: my_node_id = A_id; event with sender=A_id, content.node_id=B_id. Assertion: federation_nodes contains B_id, not A_id.

2. **`apply_federation_add_peer_event_adds_sender`** — B's vantage (B receives the event A authored; B applies it). Setup: my_node_id = B_id; event with sender=A_id, content.node_id=B_id. Assertion: federation_nodes contains A_id, not B_id. **This is the regression lock for the bug** — pre-fix code would put B_id in federation_nodes; post-fix code puts A_id.

3. **`apply_federation_add_two_vantages_mirror`** — both vantages applied to the same event end with mirrored federation_nodes. Setup: identical event; apply once with my_node_id=A_id, once with my_node_id=B_id (against two SpaceStates). Assertion: state_a.federation_nodes = [B_id]; state_b.federation_nodes = [A_id]. The two HashSets are mirrors of each other. This is the unit-level statement of D-075's "A and B end with mirrored federation_nodes through asymmetric branches" property.

4. **`apply_federation_add_third_party_observer_adds_content_node_id`** — third-party vantage (someone with multi-Space visibility observes a federation_add between A and B). Setup: my_node_id = C_id; event with sender=A_id, content.node_id=B_id. Assertion: federation_nodes contains B_id (the else-branch case, both branches need explicit coverage). This case may not occur in current production (third-party observers of federation_add events are unusual), but the test pins the else-branch behaviour and prevents regressions if the protocol surface widens.

5. **`apply_federation_add_dm_constraint_preserved`** — DM Spaces still reject federation_add. Setup: DM SpaceState with dm_constraints_active=true; my_node_id and event arbitrary. Assertion: returns `Err(SpaceError::DmFederationNotAllowed)`; federation_nodes unchanged. Regression lock for the DM constraint that pre-Commit-2 code already enforced.

6. **`apply_federation_add_missing_field_rejected`** — missing content.node_id field still rejected. Setup: event with content = json!({"session_id": "..."}); my_node_id arbitrary. Assertion: returns `Err(SpaceError::MissingField("node_id"))`. Regression lock for the structural field-missing check.

Six tests is the suggested count. Clair may add more if a code-coverage gap surfaces during implementation (e.g. an idempotency test for double-apply of the same event — `apply_federation_add` then `apply_federation_add` again on the same SpaceState produces unchanged federation_nodes because of the `.contains` check). The six listed are the minimum for both-vantage coverage + the load-bearing regression lock (#2 + #3) + the existing behaviour preservation (#5 + #6).

### 4.7 DoD for Commit 2

- [ ] `apply_federation_add` signature gains `my_node_id: &str` parameter.
- [ ] `apply_federation_add` body implements vantage-aware branching per §4.3.
- [ ] Verbatim code-comment block at the applier site cites D-075 + design task file §4.1; structural elements per §4.3 preserved.
- [ ] `SpaceState::apply_event` signature gains `my_node_id: &str` parameter; dispatch threads it to `apply_federation_add`.
- [ ] Both `NodeRuntime::ingest_event` call sites pass `&self.node_id` through.
- [ ] All other call sites to `apply_event` updated (compilation-driven enumeration).
- [ ] Six unit tests added per §4.6; named per the suggestions.
- [ ] `cargo test --workspace` passes; quote actual output with new test count in JOURNAL entry (Commit 4 picks up the number).
- [ ] No regressions in the existing 571 tests (any test that fails was relying on the bug — investigate and fix in the same commit; do not silently skip).
- [ ] Commit message names this as "Commit 2 of 4 — origin-aware applier + plumbing + unit tests for bidirectional federation_nodes."

### 4.8 What NOT to do in Commit 2

- **Do not lift `#[ignore]` from `phase9_two_node_smoke.rs`.** That's Commit 3's job. Lifting it in Commit 2 mixes scopes and makes the regression-witness-becomes-regression-lock transition harder to read.
- **Do not write the JOURNAL entry yet.** Per CLAUDE.md Rule 4, JOURNAL.md is written last (Commit 4). If Clair is tempted to draft the entry during Commit 2 because the work is fresh, save it as a local note for Commit 4 instead.
- **Do not retype any `String` field to `NodeXgid`.** Pass 1 work is not in scope. The `my_node_id: &str` parameter is deliberately `&str` per design task file §5.5 — when Pass 3 widens dispatch to XGID flavours, the parameter widens naturally; v1 keeps it at `&str` for surface neutrality.
- **Do not touch the `state.federation_add` content schema, the builder, or the wire format.** Shape D was rejected at design close; the wire is untouched.
- **Do not refactor adjacent appliers to take `my_node_id`.** D-075 names the pattern as a default for future vantage-aware appliers but does not trigger a retroactive sweep. Only `apply_federation_add` needs vantage-awareness at v1.

---

## 5. Commit 3 — Phase 9 Scenario 1 resurrection

### 5.1 Scope

`#[ignore]` removed from `xgen-node/src/tests/phase9_two_node_smoke.rs::scenario_1_two_node_push_smoke`. The scenario, authored in full during Phase 9 Commit 3a and held in place as the regression witness for the bug, now becomes the activating regression lock at integration level (Commit 2's unit tests are the regression lock at unit level).

### 5.2 Files touched

- `xgen-node/src/tests/phase9_two_node_smoke.rs` — remove the `#[ignore]` attribute from `scenario_1_two_node_push_smoke`. Update or remove the doc comment that named the bug (the bug is fixed; the doc comment text is stale).

That's it. One annotation removed; one doc comment edited. No other code touched.

### 5.3 Verification

`cargo test --workspace -- --include-ignored` should now show one fewer ignored test and one more passing test, and the workspace test count rises by one against Commit 2's count.

More importantly: run `cargo test --workspace --test phase9_two_node_smoke` (or whatever the test-binary name resolves to) directly to confirm the scenario passes. If it fails, **stop and report** — Commit 2's applier fix should be sufficient to make this scenario green; if it's not, the fix has a gap and Commit 2 needs revising rather than Commit 3 shipping with a still-broken scenario.

### 5.4 What to do if the scenario fails post-Commit-2

Per CLAUDE.md Rule 3 (stop and report when a tool fails) and Rule 7 (Definition of Done is a checklist, not a formality):

1. **Stop immediately.** Do not lift `#[ignore]` if the scenario fails — that ships a broken test.
2. **Report the failure to Joe with actual output.** Paste the failing test's output verbatim; do not paraphrase.
3. **Diagnose what Commit 2 missed.** The most likely shapes: (a) a third `apply_event` call site missed in Commit 2 that bypasses the new `my_node_id` plumbing on the federation receive path; (b) the federation push path uses a state-mutation helper that isn't `apply_event` and therefore doesn't honour the fix; (c) a test fixture in the scenario manually pre-populates `federation_nodes` with the buggy value, which now produces incorrect-but-different state with the fixed applier.
4. **Fix Commit 2's gap.** Either amend Commit 2 (if not yet pushed) or land a Commit 2.5 with the gap closure. Do not silently fold the fix into Commit 3 — atomicity matters for `git log` readability.

### 5.5 DoD for Commit 3

- [ ] `#[ignore]` attribute removed from `scenario_1_two_node_push_smoke`.
- [ ] Doc comment that named the bug updated or removed (bug is fixed; stale text is misleading).
- [ ] `cargo test --workspace` passes (now including Scenario 1).
- [ ] Test count is N+1 against Commit 2's count (one previously-ignored test now running and passing). Quote actual output.
- [ ] Commit message names this as "Commit 3 of 4 — Phase 9 Scenario 1 resurrection (regression lock activated)."
- [ ] No JOURNAL.md edit in this commit (Commit 4 carries the JOURNAL entry).

---

## 6. Commit 4 — Milestone close

### 6.1 Scope

Cross-doc state-flip housekeeping per D-074. No code, no tests. The work is "make the canonical record reflect that this milestone is done."

### 6.2 Files touched

- `JOURNAL.md` — new entry (next available J-number) recording the milestone. Per D-074, this is the same-commit JOURNAL entry; do not defer to a separate retrospective commit (the Phase 7.5 J-094 incident is the precedent that lockdowns this rule).
- `CLAUDE.md` — PLAY block flip (the bidirectional `federation_nodes` block flips from 🟢 PLAY → ✅ DONE; the Phase 9 PLAY block re-emerges as 🟢 PLAY with Commit 3b as next-active for Clair); header Last updated bumped.
- `docs/ROADMAP.md` — Visual structure tree's bidirectional-federation_nodes-phase cluster: implementation row flipped 🟢 → ✅; Phase 9 resume row flipped 🟡 → 🟢. Past section gains the implementation-shipped entry. Present section updates to reflect Phase 9 Commit 3b next-active. Header Last updated bumped + version bumped (1.10 → 1.11 expected, but use whatever's next).
- `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` (this runbook) — header Status flipped ACTIVE → COMPLETED; Last updated bumped to commit date.
- `tasks/FEDERATION_PROPAGATION_PHASE_9.md` — header Last updated bumped; PLAY-block-equivalent paragraph updated to reflect Commit 3b as now-active. (Status stays ACTIVE per the existing Phase 9 lifecycle.)

### 6.3 JOURNAL entry content sketch

Next available J-number. Title: "Bidirectional `federation_nodes` fix shipped — four-commit sequence." Content:

1. **What shipped.** Three locks (Q1+Shape A+A.1) implemented across four atomic commits per `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`. Commits 1+2+3+4 (cite hashes when available). Test count delta (quote actual output from Commit 2 + Commit 3 verification).
2. **The locked design.** One-paragraph recap pointing at design task file §3 + §4 and D-075. Future readers should be able to land on the JOURNAL entry and follow the documents from there without re-reading the audit.
3. **Verification.** Quote the `cargo test --workspace` output (full pass at Commit 3 close), the previously-ignored scenario count flipped, and any flake activity observed.
4. **Implementation discoveries.** Anything that surfaced during Commit 2 that the design didn't anticipate (a third call site, a test fixture quirk, a code-comment ambiguity that needed expansion). If nothing surfaced, say so explicitly — silence about implementation findings is itself data per Rule 1.
5. **Downstream unblocks.** Phase 9 Commit 3b is now-active for Clair (link `tasks/FEDERATION_PROPAGATION_PHASE_9.md`). M6 (new) implementation + XGID Retrofit Pass 1 implementation remain blocked behind Phase 9 closure (no change in their state).
6. **Discipline notes.** D-074 worked instance (JOURNAL same-commit). D-075 first production code instance (the `apply_federation_add` verbatim comment block). Sibling-shape recurrence (this is the third instance of "dependent work surfaces gap → audit → design → implementation → resume" after J-081 and Phase 7.5).

### 6.4 CLAUDE.md PLAY block flip

The current PLAY block describes the bidirectional `federation_nodes` design phase as next-active and points at the audit doc. Post-Commit-4 state:

- The bidirectional `federation_nodes` block flips to ✅ DONE in-flight (sibling to existing ✅ DONE-IN-FLIGHT entries in CLAUDE.md). Brief summary: four commits, design + implementation closed, JOURNAL J-NNN, test count delta.
- A new PLAY block describes **Phase 9 Commit 3b** as next-active for Clair. Entry point: `tasks/FEDERATION_PROPAGATION_PHASE_9.md`. Scope: Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10 per the existing Phase 9 task file.
- The M6 (new) PENDING block and the Pass 1 PENDING block both stay PENDING (still blocked behind Federation Event Propagation milestone closure, which now needs Phase 9 to ship before flipping).
- Header Last updated bumped to Commit 4 date.

### 6.5 ROADMAP.md flips

The Visual structure tree's bidirectional federation_nodes phase cluster:

```
└── ✅ Bidirectional federation_nodes phase (sibling to Phase 7.5)
    ├── ✅ Audit phase (canonical doc shipped 2026-05-21 at tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md v1.0 → COMPLETED at impl Commit 1)
    ├── ✅ Design phase (Q1 Reading (i) + Shape A + A.1 locked 2026-05-21; design task file at tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md v1.0 → COMPLETED at impl Commit 1; D-075 promoted)
    ├── ✅ Implementation (Clair shipped four-commit sequence; runbook at tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md v1.0 → COMPLETED at Commit 4)
    └── 🟢 Phase 9 resume (Scenario 1 #[ignore] lifted at impl Commit 3; Commit 3b now next-active)
```

The Federation Event Propagation milestone header in the tree stays "🟡 Phase 9 PAUSED" because Phase 9 itself hasn't closed yet (Commit 3b is still in-flight after this milestone). The milestone fully flips when Phase 9 closes — separate future commit.

Past section: add the implementation-shipped one-paragraph entry (paragraph length, sibling to other ✅ DONE Past entries in the section). The design-phase Past entry from v1.10 stands authoritative; the implementation-shipped Past entry is its sibling.

Present section: replace the "🟢 Bidirectional federation_nodes implementation" paragraph with a "🟢 Federation Event Propagation Phase 9 Commit 3b" paragraph describing the resume scope.

### 6.6 DoD for Commit 4

- [ ] JOURNAL.md entry written per §6.3 with actual `cargo test --workspace` output quoted (not paraphrased — Rule 2).
- [ ] CLAUDE.md PLAY block flipped per §6.4. Header Last updated bumped.
- [ ] ROADMAP.md tree + Past section + Present section + header bumped per §6.5.
- [ ] This runbook's header Status flipped ACTIVE → COMPLETED; Last updated bumped.
- [ ] `tasks/FEDERATION_PROPAGATION_PHASE_9.md` header bumped to reflect Commit 3b is now-active.
- [ ] All five files touched in one atomic commit per D-074. **JOURNAL.md MUST be in the changed-files list** — this is the D-074 discipline check.
- [ ] Commit message names this as "Commit 4 of 4 — bidirectional federation_nodes milestone close."
- [ ] No code touched. No tests touched. If anything in the codebase is unstable at this point, Commit 4 does not ship — back-fill the stability fix in Commit 3 or a Commit 3.5 first.

---

## 7. Test-count discipline

Per CLAUDE.md Rule 5: never invent numbers. Each commit's verification step requires `cargo test --workspace` actual output. The runbook's expected deltas:

- **Commit 1:** 571 → 571 (no code). Workspace pass required.
- **Commit 2:** 571 → 571 + N. N is the number of new unit tests added (suggested 6, but Clair may add more if coverage gaps surface). Workspace pass required.
- **Commit 3:** 571 + N → 572 + N. The +1 is Phase 9 Scenario 1, previously `#[ignore]`d, now passing. Workspace pass required (including the previously-ignored test).
- **Commit 4:** unchanged from Commit 3. Workspace pass required (sanity check that the housekeeping commit didn't break anything).

The actual numbers go in the JOURNAL entry (Commit 4) and in CLAUDE.md's header / PLAY block. Do not pre-fill these in this runbook — Clair fills them based on real output at each commit's verification step.

---

## 8. Risk surface and known mitigations

### 8.1 Compilation cascade from `apply_event` signature change

Adding a parameter to `apply_event` produces compilation errors at every call site. The runbook names the two production sites in `NodeRuntime::ingest_event` and notes that test fixtures will surface via `cargo build`. The mitigation is: let `cargo build` enumerate the call sites; fix each one explicitly (do not paper over with `_my_node_id` placeholder strings — every call site should pass the real local Node ID, since test fixtures construct SpaceState directly and have a known Node identity).

If a call site that isn't yet identified surfaces post-Commit-2 verification, Clair amends Commit 2 (if not yet pushed) or ships a Commit 2.5 cleanup. The runbook does not pre-enumerate all call sites because the count may have shifted since runbook authoring.

### 8.2 Pre-existing test fixtures relying on the bug

`xgen-node/src/tests/cold_start_bootstrap_integration.rs` (per audit doc §2.1) constructs `state.federation_add` fixtures manually with `fed_add(content.node_id=peer_a_id)` — that is, the fixture pre-populates the *expected post-fix value* directly, sidestepping the bug. These tests are not relying on the bug; they're relying on the test author's reasoning about what the fix should produce. They should pass against Commit 2's fix without modification.

`xgen-node/src/tests/federation_push_integration.rs` (per audit doc §2.2) uses a one-sided harness — B is a wire reader, not a real NodeRuntime — so B's `apply_federation_add` never ran in these tests. They should pass against Commit 2's fix without modification.

`xgen-node/src/tests/federation_relationship_integration.rs` (per Phase 7 J-088) constructs `build_node_with_alice_member` helpers that set up federation_peer state by ingesting `state.federation_add` events through the normal applier path. These tests **do** exercise the buggy applier. Verify against Commit 2's fix — they should pass because the test setup that produces correct A-side state ("Alice has Bob as federation peer") is exactly what the fix produces correctly. If they fail, the failure is informative and the test (not the fix) needs revising.

### 8.3 Phase 9 Scenario 1 itself

The scenario was authored by Clair during Phase 9 Commit 3a. It is the canonical bidirectional bootstrap regression witness. Post-Commit-2, the scenario should pass when `#[ignore]` lifts (Commit 3). If it doesn't, see §5.4 for the stop-and-report path.

### 8.4 Workspace flakes

Two known pre-existing flakes (CLAUDE.md, runbook §2). If either fires during verification, retry once. Do not treat as regression unless the flake fires consistently (e.g. 3+ times in 5 runs).

---

## 9. Cross-references

- **Design task file** `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` — what to build and why (the three locks Q1+Shape A+A.1).
- **Audit doc** `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` — code-grounded mechanism evidence (file:line references for the surfaces touched in Commit 2).
- **DECISIONS.md D-075** — the protocol-design principle this implementation instantiates. Verbatim code-comment block in Commit 2 cites D-075 by name.
- **DECISIONS.md D-074** — milestone-close commits MUST include JOURNAL.md. Applies to Commit 4 of this runbook.
- **DECISIONS.md D-069** — canonical-document rule. The audit doc + design task file + this runbook + D-075 + canonical design doc §6.4.2 form the five-document chain.
- **`docs/xgen_federation_propagation_design.md`** §6.4 (Phase 7 F-3 framework) — Commit 1 adds §6.4.2 sibling subsection.
- **`tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md`** (Status COMPLETED v1.0) — sibling implementation runbook for shape precedent (five commits at Phase 7.5; four commits here because the scope is smaller).
- **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** (Status ACTIVE v1.0) — Phase 9 task file. Scope intact post-Commit-4; Commit 3b becomes next-active per §6.5.
- **CLAUDE.md** — operational state. PLAY block flips per §6.4 at Commit 4.
- **`docs/ROADMAP.md`** — navigation map. Tree + Past + Present + header updates per §6.5 at Commit 4.

---

*End of implementation runbook. Status flips ACTIVE → COMPLETED in Commit 4 per the established implementation-runbook lifecycle (sibling to `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md` v1.0 → COMPLETED at Phase 7.5 milestone close). Locked content above is preserved as authoritative record of the four-commit sequence.*  
