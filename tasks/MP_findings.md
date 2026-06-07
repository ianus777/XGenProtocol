# Multiparty-tests — Findings
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

Routed defects surfaced by Multiparty-tests runs (MP-R1 onward). Per MP-R1-D6 (surface-and-route,
not patch — the M9 D-065/D-084 discipline): a scenario FAIL that is a real system defect is
recorded here, **not** patched inside the test milestone; it does **not** block the milestone's
close, and routes to its own fix-arc. Each entry: symptom, grounded divergence/rejection point,
contrast proof, code anchor, route. Mirrors `tasks/M9_findings.md`.

---

## MP-F1 — DM cross-node does not converge + DM messages not transcript-observable

- **Surfaced:** MP-C-07 (DM space across nodes), C4 (J-319). Status: **OPEN — routed.**
- **Repro:** `docs/tests/multiparty_scenarios/MP-C-07/` + the known-FAIL smoke
  `mp_r1_c4::mp_c_07_dm_across_nodes_converges` (stays RED until the fix-arc). Run with
  `--test-threads=1` against a `--features harness-control` node build.
- **Policy:** surface-and-route (MP-R1-D6); both facets are binary changes → out of MP-R1 scope.

**Facet 1 — convergence: DM `membership.join` applies on B but never propagates B→A.**
- Evidence: bob-view (node B) = `{alice:owner, bob:member}`; alice-view (node A) = `{alice:owner}`.
  Bob is a member on B, absent on A.
- Contrast proof (rules out federation/timing/harness): MP-C-02 (regular Space) converges B→A under
  the **identical** director / G-6 federation / settle — bob's join reaches A there. So the gap is
  **DM-specific**, not the shared machinery.
- `.events` corroboration: node A transcript = `state.dm_space_create` only; node B =
  `dm_space_create` + `state.federation_add` + 2× `membership.join`. Bob's joins are in B's stream,
  never in A's.
- Not authoring: the initial scenario (bob sends without joining) was corrected first — the DM seeds
  the creator as sole Owner-member and the invitee as a **pending invite** (so the counterparty must
  join, like MP-C-02). The divergence **survives** the corrected join flow, which is what makes it a
  real finding rather than a scenario bug.

**Facet 2 — observability (OPEN QUESTION, needs a dig): DM `message.text` not transcript-observable.**
- Both DM sends return `ok` with an `event_id` (alice a3, bob b4), but neither appears in either
  node's `.events`. Regular-Space messages **do** appear (MP-C-03 asserts both message `event_id`s
  in both nodes' cooperative sets — PASS). So it is DM-specific.
- One grep ruled out an obvious DM filter in `ops::send` / `apply_fanout`. Unresolved between "DM
  messages aren't emitted to the event stream at all" vs "optimistic-ack then rejected/not-applied"
  — a trace for the fix-arc, not a grep answer. **Consequence:** even if Facet 1 is fixed, DM
  message-convergence would not be transcript-observable until this is understood (it changes how any
  future DM scenario can be asserted).

**Code anchor (the DM model that grounds the scenario shape):**
`xgen-core/src/space/state.rs::from_dm_space_create` (state.rs:342) — seeds the creator as sole
Owner-member (`members.insert(creator…)`, :381), puts the invitee in `pending_invites` (:419, not
`members`), and auto-produces a room + invite. DM = pending-invite bootstrap.

**Route:** a DM-cross-node fix-arc (own Phase-0). Scope: trace why DM `membership.join` does not
replicate B→A (contrast the regular-Space join path that does), and characterize Facet 2 (DM
message emission to the event stream). Both are protocol/binary work, outside Multiparty-tests.
