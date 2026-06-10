# Multiparty-tests — Findings
> **Status**: ACTIVE  
> Version: 1.7  
> Date: Jun 2026  
> **Last updated**: 2026-06-09  
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

## MP-A-01(ii) — federation-replay membership-preserved (NOT a finding; PENDING harness machinery)

- **Status:** PENDING (recorded for completeness; the property is **proven in-process at J-298**,
  INV-EXP close). Not a defect.
- **Why PENDING:** the INV-EXP/J-298 regression guard (an aged invite+join arriving
  `ReceivedViaFederation` skips the 3044 admission gate → membership **preserved**) needs a
  late-federation / catch-up repro — B federating with A *after* A's clock has aged the Space past
  `valid_until`. The G-6 bootstrap establishes federation **early** (before the clock phase), so that
  timing is not reachable on current harness rails.
- **Route:** late-federation/catch-up ordering machinery in `xgen-mptest` (harness, not production),
  or accept the in-process J-298 proof. (MP-A-01(i) local-expiry rejection PASSED at C7.)
