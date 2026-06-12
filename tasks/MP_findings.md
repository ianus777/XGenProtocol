# Multiparty-tests — Findings
> **Status**: ACTIVE  
> Version: 1.18  
> Date: Jun 2026  
> **Last updated**: 2026-06-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

Routed defects surfaced by Multiparty-tests runs (MP-R1 onward). Per MP-R1-D6 (surface-and-route,
not patch *inside the test tranche* — the M9 D-065/D-084 discipline): a scenario FAIL that is a real
system defect is recorded here, **not** patched mid-tranche, and routes to its own fix-arc. Each
entry: symptom, grounded divergence/rejection point, contrast proof, code anchor, route. Mirrors
`tasks/M9_findings.md`.

**Fix-phase note (J-322, Joe-directed).** MP-R1 does **not** close at C7. The surfaced backlog
(these findings + the BLOCKED capability gaps) is now treated as the **purpose** of the test pass,
not a deferral: production code gets fixed, then R1 is re-run, looping toward all-green. Findings
are therefore moving from *routed/deferred* to *actively-worked fix-arcs* (each its own D-071
Phase-0, protocol-change discipline). Strategy for the BLOCKED set (fix-all-now vs catalogue across
rounds) is pending the BLOCKED-sizing pass; see J-322.

**MP-R2 fix-phase note (J-344, Joe-LOCKED) — loop-to-green, BOUNDED gate.** MP-R2's box-gated RUN
completed (bench → (a) scale-sweep → (b) fixed-N + witnesses); the spawn-scale floor is established
(MP-C-05 GREEN to 64 clients, no break-point) and **every *drivable* protocol property is GREEN**.
The RUN surfaced **four findings — MP-F7 / MP-F8 / MP-F9 / MP-F10**. Per the R1 precedent (J-322),
MP-R2 does **not** close at the RUN: it enters a **fix-phase that loops to green and re-runs**, then
closes. **BOUNDED gate criterion (the named terminus — so the loop cannot run open-ended):** the
fix-phase gate is scoped to **exactly {MP-F7, MP-F8, MP-F9, MP-F10}** and nothing else. Each reaches
one of two terminal states — **(a) GREEN on rerun** (fixed + the smoke passes), or **(b)
Joe-routed-with-reason** (a deliberate Joe-locked deferral; e.g. if MP-F9's Phase-0 proves a deep
protocol arc → **carry it to R3 as a named dependency**, an *allowed terminal state*, not a failure).
When all four are terminal, the **R2 rerun** (re-run the affected smokes — MP-C-11, MP-C-16, the C3
rows — to green-to-criterion; R1's `a9fbd98` precedent) gates the **true close**.

**Principle (Joe-stated; D-065 / D-077-aligned):** *face every bug that **occurs** — always* (see,
ground, record, route), but *gate on the **bounded** set — deliberately*. Facing ≠ fix-before-proceed:
a faced bug enters the record with a conscious disposition (fix-now / route / carry). **Newly-occurring
bugs — including any the rerun surfaces — are faced-and-routed** to their natural homes (Round-2 audit
/ R3 / own arc) but do **NOT** re-open or extend the R2 gate; the gate's scope is frozen at the four.
The work (the backlog) never ends; the **gate** closes. Marking R2 green = "this floor holds, build on
it" (R1-style, which itself shipped with MP-C-06 deliberately carried).

**Sequence (priority):** **MP-F9** (load-bearing, possibly-protocol; carries **MP-F10**, same C3
machinery) → **MP-F8** (migration aicontrol exposure, build task) → **MP-F7** (churn oracle,
test-authoring). **MP-A-07 intensity** = accepted-as-liveness-witness for R2; the design/runbook-§8
intensity-*curve* → **R3** (a build-divergence, not a fix-phase gate item). **UPDATE (J-346): MP-F9 + MP-F10 IMPLEMENTED + terminal on their defined surfaces** (C2 `8f8e79d` in-session ordered identity delivery; C3 `1216365` dependency-ordered director). MP-F9's design F-3/Design-Z hypothesis was falsified by exec-step-1 (terminal-A-extended re-locked); Smoke 1 GREEN ×2. The MP-A-01(ii) Smoke-2 RED was pinned (bounded throwaway node-side diagnostic on C) to a **distinct F-3 federation-relationship gap** (regular-Space content catch-up onto a late third node) → new finding **MP-F11**, faced-and-**routed → R3 as a named dependency** (does NOT re-open the gate; gate frozen at four — D-065/D-077). **Gate now = {MP-F8, MP-F7}** (F9/F10 CLOSED). Orthogonal departed-signer breadcrumb → **MP-F12** (own home). **Next-active = MP-F8** (migration aicontrol exposure, build task) → MP-F7 → R2 rerun → true close. **UPDATE (J-347): MP-F8 CLOSED on its defined surface** — fence LOCKED **unfenced** (MIG-D1; the route-note's "fenced like add-peer/clock" was a conflation — migration is a production admin verb, sibling to the unfenced `federation initiate`), the one aicontrol arm shipped (C1; xgen-node 290/0, default-build dispatch unit GREEN, M9.2 fence prime invariant held). The box-gated MP-C-16 witness proved C1 (verb resolves + *executes*) but stayed RED on a **distinct** home_node namespace mismatch (signed `home_node` = WS URL, `migration_initiate` expects the pubkey `node_id`) → new finding **MP-F13** (J-278/F1B-D5 family), **routed → R3** (does NOT re-open the gate; gate frozen). **Gate now = {MP-F7}** (sole remaining). Appendix F out (existing node verb, not a new client CLI verb). **Next-active = MP-F7** (churn oracle, test-authoring) → R2 rerun → true close.

**UPDATE (J-348): MP-F7 GREEN-on-rerun — the bounded gate is FULLY TERMINAL; MP-R2 ✅ CLOSED.** MP-F7 kind pinned by observation = **(b) a real leave→rejoin convergence fault** (the rejoin anchored to the `space_create` root → concurrent with the leave on `membership:{space}:a1` → `derive_resolved` elects leave via the deterministic Layer-1 leave>join priority → a1 non-member); fixed via **Fork A client-side causal anchoring** (C1 `8358bb5` — a `ClientState.last_local_events` map; `ops::leave` persists the leave's event_id, `ops::join` reads it on the `get_dag_tips`-empty fallback so the rejoin descends from its own leave; best-effort — absent anchor degrades to root = first-join unchanged). D-076 spine proven RED-on-revert in xgen-core. Witness: **MP-C-11 GREEN-on-rerun** (churn-sweep 4 rungs 2→8 clients, break_point=None, incl. the rung-0 floor that was the deterministic LogicFault). **All four gate items terminal:** MP-F7 GREEN-on-rerun · MP-F8 CLOSED · MP-F9 terminal · MP-F10 terminal. **MP-R2 close criterion CERTIFIED: all-green-except-{MP-C-16, MP-A-01(ii)}, both R3-routed as named dependencies** (MP-C-16→MP-F13, MP-A-01(ii)→MP-F11) — the R1 "all-green-except" shape; MP-C-11 was fixed (GREEN-on-rerun), not a third carve-out. **MP-R2 ✅ CLOSED (J-348).** Carried to R3 as named dependencies: MP-F11 (regular-Space late-third-node F-3 catch-up) + MP-F13 (home_node NodeXgid namespace, J-278 family). Routed to own homes: MP-F12 (departed-signer). R3 inherits the loop-to-green rerun character (J-344). **Next-active = MP-R3** (capstone: max the box bears, chaos overlay; its own D-071 Phase-0).

**Cross-round discipline (Joe, J-344):** this **loop-to-green-with-a-bounded-gate is the established
MP round-close pattern** — R1 (J-322) → R2 (J-344) → **R3 will inherit the same rerun character**.
**MP-R3 RUN #1 → fix-phase note (J-351, Joe-directed) — loop-to-green, BOUNDED gate = {MP-F14}.** MP-R3's box-gated RUN #1 completed on the freed box (re-bench FIRST → node ceiling **~1384** processes, replacing the inherited 1288/1562; box bears the ≤64-client climb with headroom, no rung-floor recalibration). **Result: all-green-except-{MP-C-16, MP-C-14}.** Load-bearing confirmations all landed: **MP-F11 RESOLVED** (MP-A-01(ii) red→green 3/3, hook placement correct, no pin-fallback), MP-A-08 green-with-boundary 4/4, MP-R3-CHAOS 3/3, MP-A-06 convergence-on-winner 4/4 (candidate payload confirmed), the MP-C-05 / MP-C-11 climbs + MP-A-07 / MP-A-18 curves all break-point-free. **MP-C-16** stays red-with-reason (MP-F13, M10+ — the expected home_node WS-URL-vs-pubkey mismatch). The RUN surfaced **one new finding — MP-F14** (regular-Space pre-join-message backfill, the MP-F11/MP-F1b/J-333 family, pinned-by-observation, gap-2 distinct from the C5/MP-F11 establish-path catch-up). Per the R1/R2 precedent the round does **not** close at the RUN: it enters a **fix-phase, BOUNDED gate = exactly {MP-F14}**. MP-F14 is **R3-grade** (core multiparty-federation protocol — no later-milestone home, and R3 is the last round) → a **fix-it gate item**, NOT a carve-out (only MP-C-16 carves, on its genuine M10+ MP-F13 blocker). Terminal for MP-F14 = **(a) GREEN-on-rerun** (the rerun-to-criterion gates the close); a (b) Joe-route is unavailable (nowhere to route — last round). **Newly-surfaced bugs face-and-route to their homes but do NOT extend the gate** (frozen at {MP-F14}). When MP-F14 is terminal → **R3 rerun** to all-green-except-MP-C-16 → MP-R3 close + the consolidated R1+R2+R3 ledger. Coverage breadcrumb (not a defect): the MP-C-14 smoke under-exercises leaf-authored content (leaf sends race their joins, land nowhere) → MP-F14's arc enriches it. **Next-active = MP-F14 D-071 Phase-0** (Clair audit).

---

## MP-F1 — DM cross-node does not converge + DM messages not transcript-observable

- **Surfaced:** MP-C-07 (DM space across nodes), C4 (J-319). Status (see UPDATE J-328 below): **OPEN — routed.**
- **Repro:** `docs/tests/multiparty_scenarios/MP-C-07/` + the known-FAIL smoke
  `mp_r1_c4::mp_c_07_dm_across_nodes_converges` (stays RED until the fix-arc). Run with
  `--test-threads=1` against a `--features harness-control` node build.
- **Policy:** surface-and-route (MP-R1-D6); both facets are binary changes → out of MP-R1 scope.

**UPDATE (J-328) — MP-F1 split into MP-F1a (facet-2, RESOLVED) + MP-F1b (facet-1, OPEN).** The C4 grounding pass (J-327) established the two facets are *independent* problems, not one. **Facet 2 is RESOLVED by MP-F1a (J-328).** Its real root cause was not a DM message filter but **client-side fire-and-forget delivery**: `create_dm_space` sent its 3 events on one connection then `goodbye`d; the RST discarded events 2–3, and the client never awaited the D-070 `EventAccepted` it already received — so the DM's room/invite/message events never reached the node's `.events`. MP-F1a's send-confirm retrofit (a verb awaits each event's node outcome before the next send / before goodbye; F1A-D1..D6, J-327/J-328) lands them. Witnessed by the single-node `MP-C-07-LOCAL` delivery scenario (GREEN, J-328): all 3 create-dm events + the invitee's space-join + room-join land. **Facet 1 (cross-node DM convergence) is now MP-F1b — OPEN.** Resolution Joe-LOCKED = (iii) membership-driven DM federation (a DM's `federation_nodes` = its members' home nodes, populated at membership-apply; J-327); Phase-0 next (gate B = home-node resolvability proves first). **A new node-side defect surfaced while building the witness → MP-F4 (below):** facet-2 delivery is fixed, but single-node *2-party message convergence* is still blocked because the invitee's room-join is dropped during node-side membership resolution.

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

---

## MP-F1b — cross-node DM convergence (membership-driven DM federation)

- **Surfaced:** MP-C-07 (DM space across nodes), facet-1 of the MP-F1 split (J-327/J-328).
  **Status: SHIPPED + CLOSED (J-333, Design Z, `9b4ab8b`).**
- **RESOLUTION (J-333, Design Z).** The J-332 design's §3.2 "no new send code" premise was **falsified** by the live two-node MP-C-07 witness: a DM federates *late* (forms its content relationship at membership-apply, not at the handshake), so the helper alone left bob's join **F-3-held** on A (the receiving federation-relationship gate requires the pusher already in `federation_nodes` — the join is what would populate it), and alice's pre-join message had nowhere to push (the one-shot initiate catch-up streams on `shared_spaces`, never re-streaming on a later set change — Option C grounded absent). The **F1B-D8** spine came back falsified too: `apply_join` open-joins (J-275), there is **no DM 2-party / join-cap gate**, so an unconditional F-3 skip would be a hole. **Design Z (Joe-LOCKED, F-3 fully intact):** populate `federation_nodes` from **parties = members ∪ pending invitees** (for a DM, exactly the 2 seeded parties from create) → the bootstrap join passes F-3 with **no skip** (a non-party's node never enters the set → F-3 still blocks 3rd parties, *no hole* — proven by `mp_f1b_third_party_dm_join_via_federation_blocked_by_f3`), and the creator's pre-join message pushes via the existing path. A `repopulate_dm_federation_after_identity` hook (fired from the identity-replicate handler) re-populates + **drains the F-3-held join** for a late-arriving record — reusing `drain_pending_by_federation_relationship` verbatim (D-076 discharged by inheritance; no empty-set instant — `apply_join` remove+insert are one apply). **Witness:** MP-C-07 cross-node KNOWN-FAIL → **harness-green-with-boundary** (a3+b4 converge A↔B, stable ×3; RED-on-revert genuine; no production witness — F1B-D4). xgen-core 689/0, xgen-node 286/0, clippy clean (default + `--all-features`). **Invariant E amended (members → parties) + promoted → D-091.** Design v1.1 §9 records the full Z reshape. Production identity→home-node discovery (F1B-D5) routed (ROADMAP near-future horizon). Arc docs AUDIT / DESIGN (v1.1) / IMPL (v1.1) → COMPLETED.
- **Phase-0 (Clair, `bfa0535`) — GAP CONFIRMED; gate-B FAILS the production case.** (iii) populates a
  DM Space's `federation_nodes` from the members' home nodes at membership-apply, and this works **in
  the harness** (G-6 pre-seeds the relationship + replicates identities). But **no production path**
  resolves a not-yet-replicated counterparty's `home_node` at DM-membership-apply:
  `build_identity_home_nodes` (runtime.rs:1895) reads `IdentityRecord.home_node` from the local
  registry (registry.rs:47); `dm_space_create` carries the *creator's* home node, not the invitee's.
  A fresh production DM to a stranger whose record has not replicated does not resolve. This is the
  **kill-gate (sub-lock B) firing exactly as J-327 anticipated** — surfaced, not worked around
  (D-065); no *small* augmentation exists (resolving a stranger re-opens identity discovery, broader
  than F1b).
- **Gate-B fork Joe-LOCKED (J-332) = Option 2 — (iii) harness-scoped + route the discovery gap.**
  Over Option 1 ((iii)+augment now — explodes scope into the wrong container) and re-opening (ii)/(i)
  (discard correct work / breach 3.16.1). Why right: **(iii) is correct *given resolved identities*;
  gate-B is the gap between "resolved" and "discoverable"** — the DM-federation-set derivation is the
  right consumer of resolved identities no matter how discovery is later solved, so sub-lock A (the
  population) is the half that stays.
- **F1B-D1..D7 Joe-LOCKED (arc-local, D-069):** D1 population = a single idempotent **NodeRuntime**
  post-membership-apply helper (`repopulate_dm_federation_nodes`), DM-only (not `SpaceState`/pure, not
  `apply_federation_add`/`DmFederationNotAllowed`-intact; re-fires at all rebuild sites — ingest
  create/`derive_resolved` arm + incremental apply arm + cold-start rehydrate; runbook enumerates).
  D2 the **full** members'-home set (invariant E), self-included; push skips self. D3 the gate-B
  boundary lives **as omission** (an unresolvable member is omitted — no crash/guess/fabricated home;
  honest-by-construction). D4 MP-C-07 recorded **with a boundary, not a bare ✅** (no production
  witness — not expressible on G-6 rails, sibling to MP-A-01(ii)). D5 the discovery gap = its own
  named arc ("production identity→home-node discovery"; routed now, placed on the ROADMAP horizon at
  close; F1b routes, does not build). D6 MP-R1-D10 amended → all-green-except-MP-C-06 **with MP-C-07
  harness-green-with-boundary**. D7 leave shrinks the set / `dm_space_create` rides G-6 / invariant E
  = DECISIONS candidate, promote at close.
- **Composition:** the send path already consumes `federation_nodes` (`derive_event_nodes`
  fanout.rs:178 + `apply_federation_push` federation_session.rs:247) — populating the set federates DM
  events both directions with no `DmFederationNotAllowed` change + no new send code. Composes with the
  shipped MP-F4 (reads Space membership, orthogonal to F4's room-scoped `state_key`s; fires after
  `derive_resolved` → F4-correct; reopens nothing). D-076 (derived projection, vantage-aware; no
  ordering surface) + D-077 (`DmFederationNotAllowed` intact; DM-only) discharged-in-design.
- **Witness (for the runbook):** MP-C-07 (`mp_r1_c4`) flips KNOWN-FAIL → harness-green-with-boundary;
  RED-on-revert = revert the population helper → set stays empty → `apply_federation_push`
  early-returns → DM doesn't federate. Plus 4 NodeRuntime units (resolvable set / omit-unresolvable /
  regular-Space-unchanged / leave-shrinks).
- **Route:** `tasks/MP_F1B_DM_FEDERATION_AUDIT.md` (`bfa0535`) + `_DESIGN.md` v1.0 (`bde00dc`) →
  runbook `_IMPL.md` (next, Clair) → implement → close. The discovery gap (D5) routes onward to its
  own arc.

---

## MP-F2 — reject path delivers generic 4000, not the specific 30xx code

- **Surfaced:** MP-A-05 + MP-A-15, C7 (J-321). **Status: RESOLVED (J-324, fix-arc shipped).** Severity: low
  (observability/contract, not security — rejections work + events correctly absent).
- **RESOLUTION (J-324, MP-F2-D1..D6):** `DispatchOutcome::Rejected(String)` widened to
  `Rejected(RejectInfo { code, name, reason })` (D1, 1-tuple-of-struct — minimal blast radius; the
  ~21 `Rejected(_)` wildcards survived unchanged); each of the 15 gates supplies its already-known
  code (D2); `reject_signal` plumbs `info.code` to the `Error` frame, deleting the hardcoded 4000
  (D4); origin gate unchanged (D5). Reason strings FROZEN byte-identical (additive code field) so
  the ~37 reason-assertion tests stayed green (D-077 backward-coherence). **Payoff verified
  end-to-end:** `mp_r1_c7::mp_a_15_clock_skew_rejected` now asserts **wire `error_code == 3046`**
  (was 4000). Build 0 + clippy clean (default + `--features harness-control`); fast suite 0-failed
  (xgen-core 667, xgen-node 72, integration 285). D-076 discharged: the `Rejected` arm returns
  `FanoutRequest::none()`, `info.code` is read only by `reject_signal` (observability) — no
  admission/ordering surface touched. Arc docs `tasks/MP_F2_REJECT_WIRE_CODE_{DESIGN,IMPL}.md`
  → COMPLETED.
- **Residual → MP-F2-followon (named, not absorbed):** (a) the 7 unmapped event-validation variants
  (signature / membership / permission) stay generic-4000 — so **MP-A-05 still delivers 4000**
  (boundary encoded as a test, `mp_a_05_*` asserts `==4000`); closing them needs an
  event-validation code-assignment decision (spec's 3001/3002 signature codes are
  registration-scoped, not event-scoped). (b) the **3030-vs-3010 tier-code spec drift** (code emits
  `3030 tier_mismatch`; spec §3.11.7 lists `3010 auth_tier_insufficient`). (c) optional cosmetic
  prose de-dup (the `(3030)` text left in some reason strings — harmless, not a drift surface).
- **Original symptom (now fixed):** `validate_event` rejections delivered an `Error` frame with
  `error_code=4000` (generic) + the reason in the message string, not the
  `ExchangeError::to_wire_code` value, so a peer/client couldn't programmatically distinguish *why*.
  MP-A-15 → was `(4000, "… exceeds now + 300s skew ceiling")` despite
  `TimestampOutOfBounds::to_wire_code()` = `3046` (exchange.rs:139). Root = a two-boundary code-drop
  (`dispatch_event` flattening to `Rejected(String)` via `err.to_string()`, runtime.rs:1086; +
  `reject_signal` hardcoding 4000, app.rs:2395). MP-F2 was the deferred completion of D-070's
  transport contract (J-081 named refinement).

---

## MP-F3 — node re-fans-out a re-submitted duplicate

- **Surfaced:** MP-A-09, C7 (J-321). **Status: RESOLVED (J-326, fix-arc shipped).** Severity: minor
  (mild fan-out amplification; no state corruption — clients dedup on apply).
- **RESOLUTION (J-326, F3-D1..D5):** new unit variant `DispatchOutcome::Duplicate`; a
  `store.contains(event_id)` dedup gate in `dispatch_event`; `process_inbound` maps `Duplicate` →
  idempotent `EventAccepted` (`LocallySubmitted` only — truthful, it *was* accepted at first ingest)
  + `FanoutRequest::none()`, which suppresses **both** local fan-out and federation push. 7
  production exhaustive arms got a trivial `Duplicate` arm (migration/test sites already had
  catch-alls; all compiler-caught). **Payoff verified end-to-end:** `mp_r1_c7::mp_a_09_*` flipped
  from the tolerant `n >= 1` to **"fanned out exactly once"** (max occurrences in any single
  transcript == 1) and PASSES against a real `--features harness-control` node — the
  harness-measurement-gap note is retired, the assertion is now falsifiable, and MP-A-09's recording
  flips from PASS-on-property + routed-finding to **clean PASS**. Build 0 + clippy clean (default +
  harness-control); xgen-core 669/0 (+2 units F3-D6/D7), xgen-node 286/0 (convergence/M8 net green),
  xgen-mptest 72/0. D-076 discharged: state is a pure function of an identical log (drop vs
  idempotent-reapply → identical convergence); the §6 side-effect-skip audit confirmed all five
  re-run effects idempotent/already-fired. Arc docs `tasks/MP_F3_DEDUP_REFANOUT_{AUDIT,DESIGN,IMPL}.md`
  → COMPLETED.
- **In-arc correction (D-065, the one real finding of the arc):** the locked F3-D1 gate placement
  ("before `validate_event`") was **wrong** and `phase9_compound_c5` caught it — a signature-forgery
  reusing a *stored* event's content+id (event_id = content hash, which **excludes** the signature)
  was mis-classified `Duplicate` instead of `Rejected(step 12)`, losing the forgery signal. "Same
  `event_id`" only means "genuine duplicate" once the event is **confirmed valid**. Fix: moved the
  gate to **after `validate_event` passes**, before the Step-4 semantic gates (still in
  `dispatch_event`, still `store.contains`, still `Duplicate`) — a genuine duplicate re-validates
  cleanly, and placing it before the semantic gates is correct because an already-stored event is
  already fully accepted (gate-drift, e.g. a since-expired invite, must not un-accept it). Recorded
  as design §8a / runbook §4a.
- **Out-of-scope catches (surfaced-not-chased, design §7):** (1) drained events don't fan out
  (`FanoutRequest` carries only the triggering event; drain-recovered events reach the DAG but not
  live-broadcast — members get them via sync; separate seam). (2) re-submit-while-pending isn't
  caught by store-based dedup (the event is in the `PendingBuffer`, not the store) but is already
  benign (`PendingBuffer::add` idempotent by event_id). Store-based dedup correctly scopes to
  already-accepted duplicates.
- **Original symptom (now fixed):** `dispatch_event` returned `Accepted` for a re-submitted
  duplicate → `process_inbound` re-ran `apply_fanout` → members/observers received it twice (DAG/
  store/disk dedup held 3 ways, so applied-once was never in question; the gap was fan-out
  amplification, no state corruption).

---

## MP-F4 — DM invitee's room-join dropped by node-side membership resolution

- **Surfaced:** MP-C-07-LOCAL (single-node DM delivery witness), C2 (J-328). **Status: RESOLVED (J-331, Option C: frontier anchor + A1 keying).**
  Severity: moderate (blocks single-node 2-party DM message convergence; no state corruption).
- **RESOLUTION (J-331, shipped bc057f8 — Option C: frontier anchor + A1 keying).** Root cause was **two compounding defects**, not the single routed one. **(1) Missing causal edge (client):** `ops::join`'s room-join anchored via `get_dag_tips`, which returned the **single topo-last event, not the DAG frontier**; with alice's pre-join message a sibling leaf (MP-C-07-LOCAL sends `a3` before bob joins), the room-join anchored to *it* instead of the space-join → concurrent → dropped. **(2) Room-agnostic key conflation (node):** the A1 finding. **The frontier fix is the finding-closer:** `get_dag_tips` now returns the true DAG frontier (all leaves, sorted, capped at `MAX_PREV_EVENTS`), so a room-join causally descends from its space-join → `apply_join`'s space-membership guard passes. **A1 retained** as latent-conflation defense (scope-aware membership `state_key` for join/leave/kick; D-077-forward — two-device / federation-reorder). **A1-alone was proven insufficient** by an empirical probe (room-join sorts first → guard fails → dropped, ~50% by hash — the MP-F3/J-326 pattern; `apply_join` guards on space membership + single-pass fold in lexicographic topo order). Witnessed: `MP-C-07-LOCAL` flipped delivery-only → **2-party message convergence** (a3+b4 on Node A) GREEN; MP-C-01 contrast GREEN (now concurrency-robust). All-callers / D-076 / D-077 sweep discharged (no single-tip-dependent test broke; no wire / persistence / reason-string change). Build 0 + clippy clean; xgen-core 683/0, xgen-node 286/0, xgen-client 103/0 + integration. The **federated** MP-C-07 (cross-node) stays KNOWN-FAIL → **MP-F1b**. Arc docs AUDIT / DESIGN (v1.2) / IMPL (v1.1) → COMPLETED.

- **Repro:** `docs/tests/multiparty_scenarios/MP-C-07-LOCAL/` — the witness is GREEN on *delivery*
  (events land) but deliberately does **not** assert message convergence, because the invitee never
  resolves as a room member. Node-side; outside MP-F1a's wire-neutral fence (F1A-D5).
- **Mechanism (grounded, Clair C2):**
  - `state_key_for_event` keys a membership event on `membership:{space}:{sender}` — **room-agnostic**
    (`xgen-core/src/resolution/state_key.rs:48`). A space-join and a room-join by the same identity
    therefore share one membership key.
  - `get_invite_bootstrap` (`xgen-client/src/batch.rs:179`) re-returns the invite naming the invitee
    **even after he is already a member** → the invitee's space-join (b2) and room-join (b3) both
    anchor to `[invite_id]` → **concurrent siblings** on the one membership key → `derive_resolved`
    keeps one and drops the other → invitee is a **Space member but not a room member** → his
    `message.text` is rejected at step-11 `NotARoomMember`.
- **Contrast proof (DM-specific):** MP-C-01 (regular Space, same join pattern) PASSES — the Node
  refuses the bootstrap once the requester is already a member, so the room-join chains off the tip
  (causal, not concurrent) and both membership facts survive. So the gap is the DM bootstrap
  re-issue, not the resolution algorithm in general.
- **F1b cross-link:** this membership-resolution surface overlaps the DM-membership code that
  (iii)/MP-F1b will touch (populate-at-membership-apply). Flag for MP-F1b Phase-0 — weigh fixing
  together; not a merge.
- **Route:** a node-side DM-membership fix-arc (own D-071 Phase-0). Candidate directions (Phase-0
  decides, NOT locked): room-scope the membership `state_key`, or gate `get_invite_bootstrap` to
  non-members. Protocol/binary work, outside Multiparty-tests.

---

## MP-F5 — C6 batch reject-oracle falsified on HEAD (the reject is now SURFACED, not fire-and-forget)

- **Surfaced:** building MP-A-03 (auth-tier arc, J-335) — Clair refused to author MP-A-03's batch witness on a contested premise and **re-ran the existing C6 tranche on HEAD** to ground it. Status: **RESOLVED (J-336, design `323fced` + impl `bee2ede`) — finish-the-surfacing, not an oracle redesign.**
- **Symptom (empirical, current HEAD):** `mp_r1_c6::mp_a_02_over_ceiling_invite_rejected` and `mp_a_04_non_member_send_rejected` — both recorded ✅ PASS at C6 (J-321) — **FAIL on HEAD** ("reply has no `event_id`"). They do **not** touch the auth-tier changes → a pre-existing regression on HEAD, surfaced on this arc's back (exactly what the loop-to-green is built to catch).
- **Root cause (grounded):** **MP-F2** (reject_signal wiring, J-324) + **MP-F1a** (await-confirm, J-328) closed the J-081 §5 reject-signal gap. A locally-submitted rejected event now gets an `Error` frame back — the node `reject_signal` carries `event_id` + code (`xgen-node` app.rs:2725) → the client `send_event_confirmed` maps it to `EventConfirm::Rejected` → `apply_single_event_confirm` **bails** → the offending op returns an **error envelope, not `ok` + `event_id`**. The C6 oracle ("the rejected op returns `ok` + `event_id` regardless — fire-and-forget, no recv") and **MP-R1-D9**'s "category not batch-observable" were both written at **J-321**, *before* MP-F2/MP-F1a landed → **stale**.
- **Favorable reframe (the crux):** the reject is now **batch-observable** — this resolves D-9 in the favorable direction. The node already sends code + `event_id`; the client merely **flattens** the wire 3030 into anyhow free text → `ErrorBody { code: "GENERIC_4000", category: Protocol, message: "…rejected by node (code 3030)…", <no event_id> }` (client aicontrol map, aicontrol.rs:88). So **MP-F5 = finish the MP-F2 surfacing into the client reply, not redesign the oracle.** The harness already captures it (`Reply::Error` + `.error()` exist) — the oracle rewrite is a read-path swap once the fields exist.
- **Tranche-wide, not auth-tier-specific:** every C6 reject scenario inherits the same dependency. **MP-A-02 / MP-A-04 empirically confirmed RED on HEAD** (Clair ran them); **MP-A-17 / MP-A-20 inferred-stale** (same reject path — effect-absence on a fire-and-forget premise) — to be **confirmed in the MP-F5 re-grounding** (a hard close deliverable).
- **Cross-arc (sequencing, J-335):** **ban**'s witnesses (MP-C-09 post-ban-reject, MP-A-14) inherit the **identical** reject-oracle dependency → MP-F5 is sequenced **before ban** so MP-A-03 *and* ban both close with green RED-on-revert witnesses, instead of piling witness-debt onto the R1 rerun. room_update / thread (cooperative) are unaffected. Revised order: auth-tier (shipped) → **MP-F5** → ban → room_update → thread×3 → R1 rerun.
- **MP-A-03 dependency:** the auth-tier **verb shipped** (bf22aaf) — gate-teeth (a) + uncapped-creation (b) held under grounding; its **batch witness greens in MP-F5** (the deferred half). Node teeth meanwhile covered by `pg13_tier1_join_into_tier2_space_rejected_3030` (runtime.rs).
- **Scope (Phase-0 authored; 5 forks for design-lock):** F2 the `ErrorBody` shape (add `event_id` + carry the wire code additively, preserving AC-D2 the client `ops::* → GENERIC_4000/Protocol` map) + F4 the **D-9 amendment** ("reject IS batch-observable post-MP-F2") are the crux; F3 the C6 oracle rewrite (assert-the-reject: wire code recoverable as a field + protected state unchanged + offending event absent) is a read-path swap. The A-02/04/17/20 re-grounding is a hard close deliverable.
- **Repro:** re-run `mp_r1_c6` (MP-A-02 / MP-A-04) on HEAD with a `--features harness-control` node build → FAIL ("reply has no `event_id`").
- **Code anchors:** client aicontrol error map (`xgen-client` aicontrol.rs:88 — `GENERIC_4000`/`Protocol`/no-`event_id`); `apply_single_event_confirm` (`xgen-client` ops.rs — bails on `Rejected`); node `reject_signal` (`xgen-node` app.rs:2725 — sends code + `event_id`); harness `Reply::Error` + `.error()` (already present).
- **Route:** the **MP-F5** fix-arc (own D-071 Phase-0, authored J-335). Production (`xgen-client` reply shape) + `xgen-mptest` oracle; touches **MP-R1-D9** (amendment at design-lock). Next-active = MP-F5 design.
- **RESOLUTION (J-336, `bee2ede`).** Shipped as finish-the-MP-F2-surfacing. **F1:** `EventConfirm::Rejected` widened to carry `event_id` (the matched `sent_id`); a typed `VerbReject` from `apply_single_event_confirm`; aicontrol downcasts it → all **8 single-event ops** inherit (`create_dm_space` multi-event chain out of scope). **F2 (Joe-locked):** `ErrorBody` gains **additive** `reject_code: Option<u32>` + `event_id: Option<String>` (envelope + harness mirror); `code=GENERIC_4000`/`category=Protocol` retained (AC-D3d wall intact); wire drift-lock test added. Category-remap sub-fork **deferred** (`reject_code==3030` is stronger than a `category` remap + sidesteps the 3030-vs-3010 spec drift). **F3:** the C6 oracle rewritten to **assert-the-reject** (`reject_code` field + `event_id` + protected-state-unchanged + offending-absent), via the existing harness `Reply::Error`/`.error()`. **F5 (hard deliverable):** the stale C6 tranche re-grounded against HEAD — **MP-A-02 → 3045 · MP-A-04 → 4000 · MP-A-17 → 4000 · MP-A-20 → 4000** (the three 4000s unmapped → MP-F2-followon) all ✅; **MP-A-03 → 3030** NEW ✅ (the auth-tier verb's deferred batch witness, RED-on-revert genuine). **MP-R1-D9 amended** (`tasks/MP_R1_DETERMINISTIC_DESIGN.md` §10): a locally-submitted single-event reject IS batch-observable post-MP-F2 — scoped to that path (not the multi-event chain / federated-reject). Verify: C6 5/5 GREEN, fast suites green (xgen-core 689 / xgen-common 140 / xgen-client 103 / xgen-mptest 73 / xgen-node 286), clippy clean, Appendix F §F.8 reject-surfacing note. (Doc-bridge for the ship was deferred + folded into the J-337 ban commit — see JOURNAL J-336.)

---

## MP-F6 — `dispatch_event` swallows the join apply-error + has no `banned` pre-check (runtime.rs:691)

- **Surfaced:** the ban arc (J-337), grounding MP-A-14 — a banned identity's re-join returns `is_ok=true` while resolution silently drops it. Routed (J-338) per Joe's standing desk-item, resolved by recommendation (route now rather than leave implicit).
- **Symptom (empirical):** `dispatch_event` applies the `membership.join` via `let _ = …` (runtime.rs:691) — the apply error is discarded — and there is **no `banned` pre-check** at dispatch. A banned bob's re-join is therefore **accepted-but-inert**: the reply is `is_ok=true`, but `derive_resolved` excludes him because `apply_join` consults `banned` (state.rs:1003, ban dominates). The end-state is correct (membership-effect-absence); the dishonesty is in the **reply**, which reports Ok for an event resolution will drop.
- **Why benign here:** for MP-A-14 the resolution layer is a second gate that catches it (ban dominates at resolve), so the inert re-join never affects membership. MP-A-14's green is membership-effect-absence, not a reject (MP-C-09's *send* path is the genuine assert-the-reject).
- **The breadcrumb (the open question):** the swallowed-apply-error shape (`let _ =`) is the same class **MP-F5** addressed at the validate layer; here it is at the **apply** layer, benign only because resolution is a second gate. Open: is the swallow **load-bearing elsewhere** — an apply site where no second gate catches a dropped error? Needs a sweep of `dispatch_event`'s apply sites.
- **Severity:** LOW (no incorrect end-state observed; reply-honesty + latent-elsewhere only).
- **Route:** deferred to **M10** (auth-module / reject-honesty era) or a future R1-rerun pass — **not an R1-rerun blocker**.
- **Code anchors:** `dispatch_event` join apply (`xgen-core` node `runtime.rs:691`, `let _ =`); `apply_join` banned-consult (`state.rs:1003`).
- **Status:** ROUTED (low-sev, deferred M10). Not resolved.

---

## MP-F7 — MP-C-11 churn-oracle / leave-rejoin convergence-precondition fault (first MP-R2-surfaced finding)

- **Surfaced:** MP-C-11 (membership churn under load), the **MP-R2 box-gated RUN — Pass-1 placeholder clients-sweep** (`mp_r2_sweep`, rung 0 = 2 clients, default floors). Deterministic, **confirmed not a flake** — sampled with healthy resources, so the classifier returned `LogicFault` (*not* Ceiling / ceiling-suspect / spawn-flake): the D-065 logic-vs-hardware split, proven here on its **first real failing rung**. Status: **ROUTED — kind-unpinned (NOT recorded as a protocol defect). Journaled at the C7 close.**
- **Kind: AMBIGUOUS** — spans protocol / client-query / test-authoring; deliberately **not** classified as a protocol defect pending the fix-arc Phase-0. Leading hypothesis (Chat, **hypothesis not locked**): the oracle/scenario authoring — a churn oracle that samples mid-flight needs a *quiesce-then-sample* or a churn-tolerant convergence check. Phase-0 pins it.
- **Symptom (empirical, current HEAD):** `mp_r2_sweep` MP-C-11 rung 0 (2 clients) → `LogicFault`, reason `"convergence needs ≥2 node projections, got 1"`. The oracle fails its **≥2-projection precondition**, rather than detecting an actual membership divergence (a real divergence would read `"membership diverges between…"`).
- **Mechanism (grounded, Clair):** `run_scenario` gathers one membership projection per actor (each client's `members` view, `runner.rs:369-386`); `convergence_verdict` requires **≥2** projections (`oracle.rs:234`). At the 2-client rung the churning client **a1** (`register→join→post→leave→rejoin`) returns **no `members` view** at oracle-sample time, so only a0 projects → the ≥2 precondition fails before any divergence check runs.
- **Contrast proof (engine-sound, scenario-specific):** **MP-C-05** (sustained chat) ran the **identical** sweep engine + per-rung oracle machinery to all-green-to-8 in the same Pass-1 — so `run_sweep` / `ScenarioTemplate::generate` / spawn / classify / `SweepResult` and the convergence oracle are sound; the fault is **MP-C-11-scenario-specific** (churn authoring / leave-rejoin / the ≥2-per-actor precondition), not the engine. This is the "scenario CONTENT validated at the first box-gated RUN" the `mp_r2_sweep` doc-comment flagged — MP-C-11 was built box-free, unit-proven only, never run until now.
- **Candidate root causes (for the fix-arc Phase-0 to pin — NOT locked):** (1) a1's client **drops its Space view after `leave`**, so the post-churn `members` query returns empty; (2) the **open `rejoin`** (no fresh invite) doesn't re-establish a queryable view single-node; (3) the **≥2-per-actor oracle precondition is simply wrong** for a churn scenario where a member may legitimately be mid-leave at sample time.
- **Severity:** blocks the **MP-C-11 row only** — rung 0 (2 clients) fails deterministically at the floor, so a higher sweep `max` changes nothing there; MP-C-11's climb is meaningless until this resolves. **Does NOT block** the MP-C-05 clients climb or the R2 break-point-per-axis record (surface-and-route: a LOGIC-FAULT on one axis doesn't gate another's record).
- **Code anchors:** projection gather (`xgen-mptest` `runner.rs:369-386`); ≥2 convergence precondition (`xgen-mptest` `oracle.rs:234`); the MP-C-11 churn scenario/template + a1's `leave→rejoin` flow (scenario authoring).
- **Route:** own fix-arc (own D-071 Phase-0 — **pin the kind first**: protocol vs client-query vs oracle-authoring). Deferred — candidate to fold into the **Round-2 whole-codebase audit**, or an **R2-followon**. Not an R2-RUN blocker. The MP-C-11 matrix row + the C7 RUN-record carry this as *"blocked-at-floor by MP-F7, climb deferred"* (recorded at the C7 close).
- **UPDATE (J-348) — RESOLVED / GREEN-on-rerun (kind (b), Fork A).** Phase-0 pinned the kind **by observation**: **(b) a real leave→rejoin convergence fault**. (a) FALSIFIED — `ops::leave` doesn't touch client state; `members` re-derives from node-drained events, so a1's empty view is *downstream* of being a non-member, not a dropped cache. (c) NOT-root — the oracle correctly flagged genuine non-convergence (a0's authoritative view resolves a1 out). **The DAG smoking gun:** the rejoin `rj` anchored to the `space_create` root (`prev=[space_create]`) → concurrent with the leave `lv` on `membership:{space}:a1` → `derive_resolved` elects leave (deterministic Layer-1 leave>join priority, algorithm.rs:105/146-147) → a1 stays non-member; the `members`=None reply is downstream (a non-member's member-gated sync starves it). Root: `ops::join`'s `get_dag_tips` falls back to `vec![space_id]` (root) when the just-left non-member's member-gated sync returns no tips (the leave anchored to a real tip — a1 was still a member then; the asymmetry is the pin). Mechanically the **MP-F4/J-331 family** (a membership event dropped because bad `prev_events` made it concurrent). **Fix = Fork A client-side causal anchoring (C1 `8358bb5`):** a `ClientState.last_local_events` map (`space_id → event_id`, **separate** from `KnownSpace` — the anchor is causal-DAG bookkeeping, not membership; zero joined-list pollution); `ops::leave` persists the leave's event_id; `ops::join` reads it on the `get_dag_tips`-empty fallback so the rejoin descends from its own leave (linear `j→lv→rj` → `frontier_of={rj}` → a1 member). **Best-effort:** an absent anchor degrades to root (first-join unchanged, old `state.json` loads — the presence-check is both the rejoin distinguisher and the safety fallback). Fork B (node scoped-fetch) declined as primary; Fork C (resolution tiebreak) rejected — would break ban/kick dominance. **D-076 spine proven not assumed** (xgen-core, RED-on-revert genuine): `..._rejoin_anchored_after_leave_is_member` ✓ + `..._rejoin_anchored_at_root_is_dropped` ✓ — across orderings, no new concurrency (framed against the MP-F4 A1-falsification: key-separation ≠ ordering; create the causal edge). **Witness: MP-C-11 GREEN-on-rerun** (churn-sweep 4 rungs 2→8 clients, break_point=None, incl. the rung-0 floor that was the deterministic LogicFault). DoD met: build 0-error, clippy `--all-features` clean, xgen-core 691/0 (+2 spine), xgen-client +3 units, xgen-common backward-compat; prime invariant held. **Oracle-edge note (recorded, not a fix):** the ≥2-all-actors precondition is latently fragile for a churn scenario that *legitimately* ends an actor mid-leave — not this case (a1 ends rejoined), flagged for future churn rows. Arc-doc `tasks/MP_F7_LEAVE_REJOIN_ANCHOR.md` (`b7f0532`) → COMPLETED.
- **Status:** **RESOLVED / GREEN-on-rerun** (kind (b), Fork A, C1 `8358bb5`, J-348). Gate item terminal — **fixed, not routed**. The last MP-R2 gate item; **MP-R2 ✅ CLOSED (J-348)**.

---

## MP-F8 — MP-C-16 `migration initiate` not exposed over `--aicontrol` (harness-undrivable)

- **Surfaced:** MP-C-16 (live migration), the **MP-R2 box-gated RUN — (b)-tranche C6c** (`mp_r2_fixed`). Status: **ROUTED — blocked-on-capability (NOT a protocol defect, NOT a clobber). Journaled at the C7 close.**
- **Symptom (empirical):** `migration initiate` over `--aicontrol` → `UNKNOWN_COMMAND`, `category:argument`, "command is not available over --aicontrol". The test's own `"did you build --features harness-control?"` hint is a **red herring**.
- **NOT the harness-control fence (grounded, Clair):** the fenced verbs `federation add-peer` + `clock` ARE in the node aicontrol dispatch under `#[cfg(feature="harness-control")]` and work — **C6a drove `add-peer` green** (3-node federation converged) → the harness-control binary is intact, fence live. `migration initiate` was simply **never wired into the aicontrol surface at all** → it falls to the not-available catch-all (`xgen-node/src/aicontrol.rs:403`). Migration exists as a node capability (the C6c director calls it) but has **no aicontrol entry point**, so the harness (which drives binaries over aicontrol) cannot reach it.
- **Contrast (migration-specific):** `add-peer` — also fenced, same `cfg` block — is drivable and drove C6a green. So the gap is the **missing migration aicontrol entry**, not the fence and not a clobber.
- **Sibling:** the MP-C-06 / MP-C-08 / MP-C-09 capability-gap family (primitive exists, no drive surface) — same shape, surfaced at RUN instead of build.
- **Severity:** blocks the **MP-C-16 row only**.
- **Route:** expose `migration initiate` over aicontrol (fenced, like `add-peer`/`clock`) — a production-crate (`xgen-node`) build task. Surface-and-route (D-065/D-084); **not a RUN action**. Deferred (C7 scope-call: own small arc, or fold into the migration-exposure work).
- **Code anchors:** aicontrol not-available catch-all (`xgen-node/src/aicontrol.rs:403`); the fenced-verb dispatch block (same file, `#[cfg(feature="harness-control")]`).
- **UPDATE (J-347) — CLOSED on its defined surface (C1 shipped + proven); the MP-C-16 row residual is a distinct finding (MP-F13), routed.** Fence shape **LOCKED = unfenced** (MIG-D1): `migration initiate` is a production admin verb (sibling to the unfenced `federation initiate`, aicontrol.rs:369; already unfenced on CLI/pipe) — the J-344 route-note's "fenced like add-peer/clock" **conflated** it with the harness-only fabrication seams; grounding refined it (D-065). **C1 shipped** — the one unfenced arm `"migration initiate" => cap!(admin_ops::migration_initiate(&mut ctx, de(args)?))`, ctx-complete (aicontrol `build_ctx` already threads runtime + federation_senders + federation_policy + paths; the async wrinkle a non-issue — `cap!` awaits uniformly). DoD met: default + `--features harness-control` builds 0-error, clippy `--all-features` clean, xgen-node 290/0, the default-build dispatch unit GREEN (verb resolves, not UNKNOWN_COMMAND), M9.2 fence-test prime invariant held. **Witness (box-gated MP-C-16 rerun, `mp_r2_fixed::mp_c_16_live_migration_space_rehomes`):** C1 proven — `migration initiate` resolves + *executes* to a real `MIG_6010` migration-flow reply (no longer UNKNOWN_COMMAND). The MP-C-16 row stays RED on a **distinct** mechanism (home_node WS-URL vs pubkey-id namespace mismatch) → recorded as **MP-F13**, routed → R3. **Appendix F out** (MIG-D3): an aicontrol dispatch arm for an *existing* node admin verb (canonical home = `xgen_node_admin_ops_design.md` §6 + Appendix K), not a new client CLI verb — the J-323 rule does not fire. Only doc touch = one line in `docs/xgen_aicontrol_implementation.md`'s verb list (Chat's, this bridge). Arc-doc `tasks/MP_F8_MIGRATION_AICONTROL.md` → COMPLETED.
- **Status:** **RESOLVED / CLOSED on its defined surface** (C1 shipped + proven, J-347). Gate item **CLOSED**. MP-C-16 row residual → **MP-F13** (R3).

---

## MP-F9 — late-federation catch-up does NOT backfill existing Space history (kind-ambiguous)

- **Surfaced:** the C3 late-federation/catch-up machinery (`mp_r2_catchup::late_federation_catch_up_converges`), **MP-R2 RUN (b)-tranche**. The C3 infra (one of R2's three mechanisms, built box-free at J-342) is **RED at first RUN**. **Confirmed deterministic** (isolated re-run ×2, Rule 2). Status: **ROUTED — kind-unpinned (NOT recorded as a protocol defect). Journaled at the C7 close.**
- **Symptom (empirical):** B federates late (the `add-peer` + `initiate` fire, **no deadlock**), but the existing Space history does **not** backfill onto B — B's transcript is **empty (zero events)**.
- **Kind: AMBIGUOUS** — **protocol** (late federation genuinely doesn't backfill existing history — a real capability gap) **or harness** (the C3 late-fed director path doesn't request/trigger the sync). Deliberately **not** classified as a protocol defect pending the fix-arc Phase-0.
- **The open question (for Phase-0):** the normal G-6 bootstrap federates **early** (before any history exists), so this is the **first test of federate-AFTER-history**. Does the protocol's federation-initiate trigger a history sync/backfill onto the new peer (and doesn't), or does the C3 director's late-fed path fail to drive it? Ground the federation-initiate → sync path against live code.
- **Severity:** blocks **both C3 rows** (late-fed catch-up + MP-A-01(ii)). **Potentially load-bearing for R3** — the R3 partition+reconnect storm (MP-A-08) leans on catch-up/convergence-after-heal — **IF the root is protocol**. A near-term priority flag, not a deferral-to-R3.
- **Route:** own fix-arc + **D-071 Phase-0 (pin the kind first)**. If protocol: a late-federation history-backfill capability arc (likely before R3). If harness: the C3 late-fed director path.
- **Code anchors:** the C3 late-fed director path (`xgen-mptest/src/runner.rs` `run_director` federation phase, `FederationLink.after`); the federation-initiate / sync path (`xgen-core`, Phase-0 to ground).
- **UPDATE (J-345) — kind PINNED: PROTOCOL (bounded); Phase-0 + design Joe-LOCKED; terminal-state A (conditional).** Phase-0 (`tasks/MP_F9_LATE_FEDERATION_BACKFILL_AUDIT.md`, `dcfc81a`) traced it decisively: the late-federation backfill path **exists and is correct sender-side** (A's `stream_federation_delta` absent-peer rule → `compute_federation_delta_for_space(None)` = full topo-sorted history; A's `shared_spaces` incl. the late Space threaded through). B **holds all of it** — the decisive datum is *zero* events, not "only `state.space_create`": `space_create` skips F-3 (`runtime.rs:1027-1032`), so the only thing holding the create too is **step-11 sender-registration** — `StateSpaceCreate` is NOT in the `node_authored` exempt set (`exchange.rs:622-633`), so an alice-signed create with alice unknown to B → `HeldPending(missing_identity=alice)`; every backfilled event is alice-signed → all held → empty transcript. **Root: late federation backfills events but NOT the identities that signed them** (`push_identity_to_peers` fires only at registration to then-current peers, `app.rs:2856/3118-3192`; the federation session streams Space-DAG events only, never the registry). A **bounded PROTOCOL gap** — not "no backfill exists." **Design (`tasks/MP_F9_LATE_FEDERATION_BACKFILL_DESIGN.md`) → terminal-state A (fix-in-fix-phase):** the fix composes via *existing* machinery — `handle_identity_replicate_msg` already auto-fires `drain_pending_by_identity` on a record landing (`app.rs:2913/2975`, `runtime.rs:1717`), releasing the F-10-held events for that signer; and disclosure is **already settled** (establish already `record_peer_url`s "to push identity replicas later," `app.rs:1976-1983`; `push_identity_to_peers` already serializes the whole `IdentityRecord` incl. `home_node` to peers at registration — early-federated peers already get later-registered members' records). The **only** gap is the trigger. **NOT MP-F1b's D5** (stranger home-node discovery) — B catches up records of signers whose events it's already receiving. **Design-locked: F9-D1** backlog-push-on-establish (both sides, symmetric) · **F9-D2** generalized trigger (any establish; reuses the reconnect path J-085 → R3/MP-A-08 free) · **F9-D3** signer-set = distinct senders of the backfilled delta (NOT current-members-only; R3-correct). **Conditional:** terminal-A holds iff the confirming traced re-run (runbook exec step 1) nails the verdict (the `state.space_create` HeldPending discriminator) + implementation surfaces no real disclosure-scoping fork → else reverts to route-to-R3 (D-065). Sequencing (ii): runbook authored now, re-run = exec step 1.
- **UPDATE (J-346) — IMPLEMENTED + terminal on its defined surface; the row-residual is a distinct finding (MP-F11), routed.** The MP-F9 design's F-3/Design-Z hypothesis was **FALSIFIED** by the runbook exec-step-1/Task-1 observation: create-F-10-held → its children REJECTED "space not found" at the F-4 step-1 pre-check (`runtime.rs:992`), dropped permanently — not held-then-drained. Corrected mechanism + fix = **in-session ordered identity delivery** (signers' `IdentityReplicate` sent over the session conn *before* the Space-DAG delta; receiver applies them first). A throwaway spike confirmed **8/8 deterministic closure** → Joe re-locked **terminal-A-extended**. **C2 (`8f8e79d`) shipped it** (`send_space_signers_in_session`, both sides; `handle_identity_replicate_msg(.., ack=false)` before the delta stream). In-process green: core 6/6, F9-D3 departed-signer positive, idempotence, prime invariant 289/0, RED-on-revert genuine. **R2 rerun:** Smoke 1 (basic late-fed catch-up) **GREEN ×2** — MP-F9's defined surface (identity catch-up) is **terminal**. Smoke 2 (`mp_a_01_ii`, 3-node aged-invite) RED ×2 — pinned by a bounded node-side diagnostic to a **distinct F-3 federation-relationship gap** (regular-Space content catch-up onto a late third node), NOT an identity-backfill failure → recorded as **MP-F11**, routed → R3. The **third** inferred-mechanism falsified-by-observation this arc (after F-3/Design-Z by Task-1, detached-delivery by the spike).
- **Status:** kind **PROTOCOL** (bounded); IMPLEMENTED (C2 `8f8e79d`, terminal-A-extended); **terminal on its defined surface** (Smoke 1 GREEN ×2 + in-process witnesses). Row-residual (MP-A-01(ii) Smoke 2) = distinct finding **MP-F11** → R3. Gate item **CLOSED** (J-346).

---

## MP-F10 — director phase-ordering deadlock: a federation-link gated on a clock-published key (pure harness)

- **Surfaced:** C3 MP-A-01(ii) (`mp_r2_catchup::mp_a_01_ii_aged_invite_replay`), **MP-R2 RUN (b)-tranche**. **Confirmed deterministic** (isolated re-run ×2, 45s timeout). Status: **ROUTED — pure test-crate machinery (NOT a protocol defect). Journaled at the C7 close.**
- **Symptom (empirical):** 45s timeout — `"waiting for cross-actor key {clock_advanced} (no actor exported it)"`.
- **Mechanism (grounded, Clair, `runner.rs:437-496`):** `run_director` runs phases **sequentially** federation → clock → migration. A late-fed link with `after = Some(clock_advanced)` blocks the **federation phase** on `wait_for(clock_advanced)` (`:452`), but `clock_advanced` is only published in the **later clock phase** (`:494`) → the federation phase waits for a key the clock phase (which runs *after* it) never reaches to publish → **deadlock**. The `"harness-control?"` hint is a red herring (the clock verb is never reached).
- **Kind: PURE HARNESS** (test-crate machinery) — a federation-link-gated-on-a-clock-key cannot work under the fixed phase order. Not a protocol defect.
- **Compounds with MP-F9:** even if the backfill (MP-F9) worked, this deadlock independently blocks MP-A-01(ii)'s clock-aging path.
- **Severity:** blocks the **MP-A-01(ii) row** (aged-invite-replay).
- **Route:** harness reorder/interleave in `xgen-mptest` — e.g. interleave the federation/clock phases, or allow a federation link to be scheduled after a clock step (own test-crate arc). Likely paired with the MP-F9 fix-arc (same C3 machinery).
- **Code anchors:** `run_director` phase sequence (`xgen-mptest/src/runner.rs:437-496`; fed-phase wait `:452`, clock publish `:494`).
- **UPDATE (J-345) — design-locked F10-D1; carried in the MP-F9 arc.** **F10-D1 (Joe-LOCKED):** a **dependency-ordered single-owner director** — order director steps by publish→wait edges (the clock step publishing `clock_advanced` runs *before* the fed link waiting on it), staying sequential + preserving the `&mut nodes` single-owner model → no deadlock. Self-contained, rides the MP-F9 implementation (same C3 machinery). Row coupling: MP-F9 gates **both** C3 rows; **F10-D1 additionally** unblocks the clock-aged Smoke 2 (`mp_a_01_ii`, which needs both fixes).
- **UPDATE (J-346) — IMPLEMENTED + terminal.** **F10-D1 shipped — C3 (`1216365`):** the dependency-ordered single-owner director (order steps by publish→wait edges; the clock step publishing `clock_advanced` runs *before* the fed link waiting on it; sequential, `&mut nodes` single-owner preserved). Proven: Smoke 2 **ran to completion through live clock-aging — no deadlock** (the deadlock was the symptom; it is gone). In-process C3 87/0, RED-on-revert genuine. The residual Smoke 2 RED is MP-F9's row-residual (the F-3 gap, MP-F11), NOT a director-deadlock recurrence.
- **Status:** kind PURE HARNESS; IMPLEMENTED (C3 `1216365`, F10-D1); **terminal** (deadlock fixed, mechanism green). Gate item **CLOSED** (J-346).

---

## MP-F11 — regular-Space content catch-up onto a late-federating third node is F-3-gated (federation-depth)

- **Surfaced:** the MP-R2 fix-phase MP-F9/F10 arc — the box-gated R2 rerun Smoke 2 (`mp_r2_catchup::mp_a_01_ii_aged_invite_replay_preserves_membership`, mp_r2_catchup.rs:202). Pinned by a bounded throwaway node-side diagnostic on node C (J-346). Status: **ROUTED → R3 as a named dependency (Joe-LOCKED J-346). NOT a new gate item; faced-and-routed to its home.**
- **Symptom (empirical):** 3-node aged-invite scenario (A creates/rooms/invites; bob joins; A's clock +2d; C federates late & catches up). Late node C lands only A-node-authored federation events — the entire Space *content* (alice's `state.room_create` + `membership.invite`, and bob's aged `membership.join` b2) is **dropped**, never applied.
- **Mechanism (pinned by observation, not inferred):** all three content events are **F-3-held** on C — `reason="federation_relationship_missing"`, `disposition="held_pending"` (node C log, `event="f3_reject"`) — because C's `SpaceState.federation_nodes` for the Space never includes the pushing peer A, and **no drain ever fires** (`drain_pending_by_federation_relationship` / repopulate never runs on C; b2 `apply_event` count = 0). C lands only create + node-authored federation events.
- **Falsifies all three R2-rerun candidates (the diagnostic's purpose):** NOT **(a)** 3044 origin-gating (reject is `f3_reject`, not `validate_event/invite_expired`; content unrelated to invite-expiry is held too — C's clock is un-aged, 3044 never in play); NOT **(b)** missing bob-record (`federation_relationship_missing`, not `missing_identity`; alice-signed room/invite held too); NOT **(c)** oracle bug (the events genuinely don't land). **A fourth, distinct mechanism** — the **third** inferred-mechanism falsified-by-observation this arc.
- **Why distinct from MP-F9 (gate-honesty):** MP-F9's defined surface = identity catch-up (signers not replicated; J-345 PROTOCOL verdict) — **terminal** (Smoke 1 GREEN ×2 + in-process witnesses; C2 `8f8e79d`). This F-3 gap is **not** an identity-backfill failure. Mechanically it is MP-F1b/Design-Z (federation_nodes population + F-3 + `drain_pending_by_federation_relationship`) — solved there for **DMs**, needed here for a **regular Space late-federating onto a third node**. (Why Smoke 1 passes: 2-node A→B, B's `federation_nodes` includes A. Smoke 2: 3-node late A→C, C's set never gets A.)
- **Severity:** blocks the **MP-A-01(ii) row only** (its real-binary witness). The property stays J-298-proven in-process.
- **Route (Joe-LOCKED J-346):** **R3 as a named dependency** — N-node / late-catch-up federation depth lives in R3 (where MP-A-08 partition+reconnect already sits). F-3 is the surface J-345 deliberately fenced off after Task-1; generalizing Design-Z (D-091 invariant E + the repopulate hook + `drain_pending_by_federation_relationship`) to populate a late peer's `federation_nodes` for shared *regular* Spaces on establish + fire the F-3 drain is scope-expand into that arc, NOT a fix-phase patch (the J-333 "an unconditional F-3 skip would be a hole" lesson makes F-3 changes non-trivial). Precedent: F1B-D5 (production identity→home-node discovery) was routed to a ROADMAP horizon, not fixed in-arc. Honest-alternative (terminal-A-extended-again, generalize Design-Z in-phase) was weighed and **declined** — touches F-3 on a new path after three falsified mechanism-guesses on this family.
- **Code anchors:** the F-3 deferral / `f3_reject` (`xgen-node` runtime.rs); `federation_nodes` population (Design-Z `repopulate_dm_federation_*`, D-091); the late-fed director path (`xgen-mptest` runner.rs).
- **UPDATE (J-351) — RESOLVED / GREEN at MP-R3 RUN #1.** The R3 fix (C5/MP-F11, `9ac7780`) generalized the shipped Design-Z machinery — `federation_relationships` + `establish_federation_relationship` + the rebuild-site repopulate + `drain_pending_by_federation_relationship` — from DM-only to regular Spaces on the federation-establish path (the `xgen-node` `stream_federation_delta` receiver hook), **F-3 intact** (third parties stay blocked; hole-closed spine `mp_f11_third_party_..._blocked_by_f3`). Spine proven RED-on-revert (J-350); the box-gated witness **MP-A-01(ii) (`mp_r2_catchup::mp_a_01_ii_*`) flipped red→green 3/3 at RUN #1** — hook placement correct, no pin-fallback. MP-A-08's relationship-level heal (4/4) rides the same path. **Fixed-in-round, not routed.**
- **Status:** **RESOLVED / GREEN-on-rerun** (R3-D6, C5 `9ac7780`, J-351). MP-A-01(ii) GREEN 3/3. The R2/J-346 named R3 dependency is **discharged**.

---

## MP-F12 — departed-signer post not re-dispatched on the membership-gated re-dispatch path [orthogonal, routed]

- **Surfaced:** flagged in the MP-F9 C2 commit (`8f8e79d`, J-346) while building in-session ordered identity delivery — orthogonal to the F9/F10 mechanisms; NOT exercised by Smoke 1/2. Status: **ROUTED to its own home (Joe-LOCKED J-346). NOT chased in this arc.**
- **Symptom:** a post authored by a since-departed signer is not re-dispatched on the membership-gated re-dispatch path in `drain_pending_by_identity`.
- **Severity:** LOW (breadcrumb; not observed to affect Smoke 1/2 outcomes). Sibling-shape to MP-F6 (a routed seam, not a gate item).
- **Route:** its own finding/home — the peer/identity-discovery space / the `drain_pending_by_identity` membership-gating seam. Not a gate item, not an R2 blocker.
- **Code anchors:** `drain_pending_by_identity` membership-gated re-dispatch (`xgen-node`).
- **Status:** ROUTED (own home), not resolved. Bounded.

---

## MP-F13 — Space `home_node` holds a WS URL, not a node pubkey id (NodeXgid contract violation; J-278 family)

- **Surfaced:** MP-C-16 (live migration), the MP-F8 box-gated witness rerun (`mp_r2_fixed::mp_c_16_live_migration_space_rehomes`). C1 (the MP-F8 aicontrol arm) proven — `migration initiate` resolves + *executes* to a real migration-flow reply. Pinned by a bounded throwaway test-crate diagnostic (J-347, deterministic ×3). Status: **ROUTED → R3 as a named dependency (Joe-LOCKED J-347). Does NOT re-open the {MP-F7} gate.**
- **Symptom (empirical):** `migration_initiate` rejects with **MIG_6010** `"Space … is not homed on this Node"`, `stage="register"`, on the source node A *where the Space is present* (the `Some(st)` branch, `st.home_node != rt.node_id` — NOT MIG_6011/absent).
- **Mechanism (pinned by observation, ×3):** the migrated Space's signed `space_create.content.home_node` = `ws://127.0.0.1:8521/xgen` (A's WS URL); A's `rt.node_id` = `xgen://pubkey/ed25519:…` (A's pubkey id). `migration_initiate` compares the two as strings → **two different identifier namespaces** (ws:// URL vs xgen://pubkey/ id) → never equal → MIG_6010. Every persisted `home_node` across the run is a `ws://…/xgen` URL.
- **Root (the fix-shape fork the string resolves):** `SpaceState.home_node` is typed `NodeXgid` (intended: a node pubkey id), but `ops::create_space` writes it from the client's `session.home_node` — and per **J-278/MP-F1b** the client only ever learns the node's **WS URL**, never its pubkey id. The node stores the signed content **verbatim** (no ingest override; can't rewrite signed content). So `home_node` **universally contains a URL, violating its NodeXgid contract** — and migration is simply the first code path to compare `home_node` against a real node pubkey id and hit the mismatch. `migration_initiate`'s check is **correct against the intended model**; the defect is upstream, in what `home_node` contains.
- **Why never caught before:** the in-process Arc F migration tests set `home_node = node_id` explicitly ("the caller's choice", runtime.rs:2207); the real-binary client path always writes a URL. MP-C-16 is the **first real-binary exercise of Arc F migration with a client-created Space** — the J-344 box-free-built-smoke-first-run class.
- **NOT C1, NOT test-authoring:** C1's arm is proven (verb resolves + executes). The harness can't trivially make `home_node` the pubkey id without solving J-278 (the client doesn't know it). An architectural identity-namespace inconsistency, not a migration bug and not a scenario bug.
- **Severity:** blocks the **MP-C-16 row only**. **Broader flag (D-065):** **any** code comparing `home_node` to a node pubkey id hits the same wall — migration is just where it first surfaced. A scope note for the J-278 arc, not chased here.
- **Route (Joe-LOCKED J-347):** **R3 as a named dependency — the J-278 / F1B-D5 home-node-identity arc** ("production identity→home-node discovery", already routed to a ROADMAP horizon; same root). The root fix is the client writing the node's pubkey id as `home_node`, which requires the client to *learn* it (the J-278 gap). **Fix-shape forks (string-resolved):** (c) deeper J-278 dependency = the root, **the lean/route**; (b) migration-resolves/compares-URL (node compares `home_node` against its own known listen URL) = a bounded near-term symptom-fix that could green MP-C-16 without solving J-278 — **flagged, NOT taken** (papers over the NodeXgid contract violation + leaves the broader inconsistency); (a) node-normalizes-home_node-on-ingest = **blocked** (signed content, node can't rewrite). Same shape as MP-F11.
- **Code anchors:** `migration_initiate` homed check (`xgen-node` admin_ops.rs:2081; MIG_6010 on `st.home_node != rt.node_id`); `ops::create_space(home_node)` writes `content["home_node"]` from `session.home_node` (`xgen-client`); `from_space_create` stores verbatim (`xgen-core`); the in-process test setting `home_node = node_id` (runtime.rs:2207); J-278 / F1B-D5 (client learns the WS URL only).
- **Status:** ROUTED → R3 named dependency (J-278/F1B-D5 family, Joe-LOCKED J-347). MP-C-16 is its first witness row. Does **not** re-open the gate.

---

## MP-F14 — regular-Space pre-join-message backfill: a member joining after a post never receives it (gap-2, MP-F11/MP-F1b family)

- **Surfaced:** MP-C-14 (4–5 node star+mesh), the **MP-R3 box-gated RUN #1** (`mp_r3_topology`). Pinned by observation across 8 runs (J-351). Status: **the sole MP-R3 fix-phase gate item — fix-in-round (R3-grade, no later-milestone home); NOT a carve-out.**
- **Symptom (empirical, pinned ×8):** membership always converges, but ~60% of default-settle runs leave **exactly one** cooperative event stuck on n0 — always **a0's p0** (the creator's post, authored before the open-join leaves join), never a leaf join. A consistent single victim ⇒ structural gap, not a settle-race.
- **Discriminator (conservative settle, stable-for-25 = 10s quiescence):** fail rate drops to ~1/3 but does **not** vanish — p0 stays stuck after 10s of total quiescence. **Not a pure settle-race** → a genuine intermittent backfill gap. (Pin-by-observation caught a wrong route mid-pin: the gap-2 hypothesis first looked like a settle-race on conservative-pass 2/3; run 3's 10s-quiesced failure ruled that out — the MP-R2 bar.)
- **Mechanism (pinned):** a leaf joining the Space **after** the creator's post needs that post backfilled — but it wasn't in the creator's `federation_nodes[S]` at post-send time. This is **gap-2** of the J-333 regular-Space family, **distinct from C5/MP-F11** (gap-1 = late-federate/establish-path catch-up, which rides the tip-exchange — MP-A-08 + MP-A-01(ii) prove that path works). MP-C-14 is **early-federate + late-member-join**, which triggers **no new establish**, so C5's populate-on-establish never re-fires.
- **Why distinct + R3-grade:** not MP-C-16/MP-F13 (home_node/J-278, M10+); not a settle-race (the 10s-quiesced discriminator); not a harness bug (the events genuinely don't land). **Core multiparty-federation protocol** — a member silently missing content — in the same layer C5 just fixed, with **no later-milestone home**. R3 is the last round → nowhere to route → a **fix-it gate item**.
- **Route (the fix-phase):** an **MP-F14 D-071 fix-arc** (own Phase-0 → design → Joe-lock → runbook → implement; the same Design-Z / MP-F11 family — likely **re-stream-on-member-join / re-push-on-`federation_nodes`-update**, with the **J-333 hole-safety** care: only legitimate members get the backfill, third parties stay F-3-blocked). May land bounded (reuses the MP-F11 machinery). **Coverage enrichment (Clair, not a defect):** the MP-C-14 smoke under-exercises leaf-authored content (leaf sends race their joins → land nowhere → only a0's p0 is cross-node content under test) — the arc enriches the smoke so a green exercises leaf posts.
- **Code anchors:** F-3 / `federation_nodes` population + `drain_pending_by_federation_relationship` (Design-Z / MP-F11, `xgen-core`/`xgen-node`); the member-join apply path (no re-push trigger today); the MP-C-14 StarPlusMesh generator (`xgen-mptest` sweep.rs).
- **Status:** the **sole MP-R3 fix-phase gate item** (J-351). Terminal = (a) GREEN-on-rerun (gates the MP-R3 close). MP-C-14 is its witness row.

---

## MP-A-01(ii) — federation-replay membership-preserved (row PENDING — machinery now BUILT but RED at RUN → MP-F9/F10)

- **Status:** PENDING (recorded for completeness; the property is **proven in-process at J-298**,
  INV-EXP close). Not a defect.
- **Why PENDING:** the INV-EXP/J-298 regression guard (an aged invite+join arriving
  `ReceivedViaFederation` skips the 3044 admission gate → membership **preserved**) needs a
  late-federation / catch-up repro — B federating with A *after* A's clock has aged the Space past
  `valid_until`. The G-6 bootstrap establishes federation **early** (before the clock phase), so that
  timing is not reachable on current harness rails.
- **Route:** late-federation/catch-up ordering machinery in `xgen-mptest` (harness, not production),
  or accept the in-process J-298 proof. (MP-A-01(i) local-expiry rejection PASSED at C7.)
- **UPDATE (MP-R2 RUN, b-tranche):** the late-federation/catch-up machinery was **BUILT (C3, J-342)
  and RUN** — so this is no longer "machinery doesn't exist." Both C3 rows are RED at first RUN:
  **MP-F9** (does B receive existing history at all — backfill gap, kind-ambiguous) gates this row,
  and **MP-F10** (the clock-gated federation-link director deadlock) blocks its specific clock-aging
  path. The property remains **J-298-proven in-process**; the real-binary witness awaits MP-F9 + MP-F10.
  Status now: **ROUTED-via-MP-F9/F10** (was PENDING-no-machinery).
- **UPDATE (J-346) — routed → R3 (Joe-LOCKED).** The box-gated R2 rerun + a bounded throwaway node-side diagnostic pinned this row's residual RED to a **distinct F-3 federation-relationship gap** (regular-Space content catch-up onto a late-federating third node — C's `federation_nodes` never gets A, content F-3-held, no drain) → recorded as **MP-F11**, routed to **R3 as a named dependency**. MP-F9 (identity catch-up) + MP-F10 (director deadlock) are both terminal on their defined surfaces; this row's real-binary witness now waits on MP-F11 (R3). Property stays J-298-proven in-process. **UPDATE (J-351) — GREEN at MP-R3 RUN #1.** MP-F11 RESOLVED (R3-D6, C5 `9ac7780`); the real-binary witness `mp_r2_catchup::mp_a_01_ii_*` flipped **red→green 3/3** at RUN #1 (hook placement correct, no pin-fallback). The J-298 in-process proof now has its real-binary confirmation. **Status: GREEN (R3 RUN #1, via MP-F11 RESOLVED).**
