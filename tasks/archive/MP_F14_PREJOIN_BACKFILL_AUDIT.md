# MP-F14 — regular-Space pre-join-message backfill — D-071 Phase-0 audit
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

The D-071 Phase-0 audit for **MP-F14**, the **sole MP-R3 fix-phase gate item** (J-351). MP-F14 is
**gap-2** of the J-333 / MP-F1b / MP-F11 regular-Space federation family: a member who joins a Space
**after** an existing post never receives that post. It is **R3-grade** (core multiparty-federation
protocol — a member silently missing content), has **no later-milestone home**, and R3 is the last
round → a **fix-it gate item, NOT a carve-out** (only MP-C-16 carves, on its genuine M10+ MP-F13
blocker). Terminal = **(a) GREEN-on-rerun**; a Joe-route is unavailable (nowhere to route).

**This is grounding only — no code, no design locks.** It grounds the gap to file:line against live
`main`, contrasts it precisely with the shipped C5/MP-F11 fix (so the two are not conflated), pins the
J-333 hole-safety surface, and names the coverage enrichment. The design (`MP_F14_PREJOIN_BACKFILL_DESIGN.md`)
locks the fix shape on Joe's call; the runbook follows.

**Method (the MP-R2/R3 bar):** surface-and-route (D-065/D-084); **pin-by-observation BEFORE routing**.
The finding was pinned by observation across 8 RUN-#1 runs (J-351); the *exact* failing trace
(§3) is a design/runbook pin-by-observation deliverable, not a static guess — exactly the MP-F9
runbook-exec-step-1 precedent (the audit grounds the surfaces; the observed re-trace confirms which
one fires). The route (fix-it gate item) is already locked; the fix *site* is what the observed trace
pins.

---

## 1. The finding (recap, from `MP_findings.md` MP-F14 + J-351)

- **Surfaced:** MP-C-14 (4-node star+mesh), MP-R3 box-gated RUN #1, `mp_r3_topology`.
- **Symptom (pinned ×8):** membership always converges, but ~60% of default-settle runs leave
  **exactly one** cooperative event stuck on n0 — always **a0's `p0`** (the creator's post, authored
  before the open-join leaves join), never a leaf join. A consistent single victim ⇒ a structural
  gap, not a settle-race.
- **Discriminator (conservative settle = 10s quiescence):** fail rate drops to ~1/3 but does **not**
  vanish — `p0` stays stuck after 10s of total quiescence → **not a pure settle-race** → a genuine
  intermittent **backfill** gap. (Pin-by-observation caught a wrong route mid-pin: the gap-2
  hypothesis first looked like a settle-race; run-3's 10s-quiesced failure ruled that out.)
- **Mechanism (pinned):** a leaf joining the Space **after** the creator's post needs that post
  backfilled — but it wasn't in the creator's `federation_nodes[S]` at post-send time. **Early-federate
  + late-member-join**, which triggers **no new establish**, so C5/MP-F11's populate-on-establish never
  re-fires.

---

## 2. The grounded gap — structural surfaces (live `main`, to file:line)

The gap is a **send-time-only outbound federation push** with **no re-push when `federation_nodes[S]`
grows on an already-live session**. Five surfaces ground it; together they show why `p0` can never
reach a node that becomes a federation target *after* `p0` was sent.

### 2.1 The outbound push reads `federation_nodes[S]` at the instant of the push — and only then

`apply_federation_push` ([`federation_session.rs:280`](../xgen-node/src/federation_session.rs#L280))
is the regular-Space outbound federation path. It:
1. F-5 anti-transitivity guard — skip if `origin == ReceivedViaFederation`
   ([`:301`](../xgen-node/src/federation_session.rs#L301));
2. reads the Space's `federation_nodes` **once, now**
   ([`:331-337`](../xgen-node/src/federation_session.rs#L331)) — `rt.spaces.get(&space_id) → s.federation_nodes.clone()`;
3. if empty → **return** ([`:338`](../xgen-node/src/federation_session.rs#L338));
4. for each `peer_id` in that snapshot, `try_send(OutboundMsg::Event(event))`
   ([`:380-395`](../xgen-node/src/federation_session.rs#L380)).

**The load-bearing fact:** the push targets are the `federation_nodes[S]` snapshot **at `p0`'s
send-instant**. There is **no record of "`p0` was sent to set X"** and **no mechanism that re-pushes
`p0` when the set later grows.** A node absent from `federation_nodes[S]` when `p0` is pushed never
receives `p0` from this path, ever. (`apply_federation_push` is invoked per live locally-submitted
event from `process_inbound`; it is not replayed.)

### 2.2 How `federation_nodes[S]` grows for a **regular** Space — and what does NOT grow it

Three writers, and a member-join is **not** one of them:

- **`apply_federation_add`** ([`state.rs:655`](../xgen-core/src/space/state.rs#L655)) — applying a
  `state.federation_add` event adds the vantage-derived peer to `federation_nodes`
  ([`:699-701`](../xgen-core/src/space/state.rs#L699)). This is the event-driven populate; it is
  subject to the F-1a / predecessor / mutual-hold ordering (the surface C5 partially addresses, §2.3).
- **`establish_federation_relationship`** ([`runtime.rs:1983`](../xgen-core/src/node/runtime.rs#L1983),
  C5/MP-F11) — records the relationship and pushes the peer into a present non-DM Space's
  `federation_nodes` ([`:1996`](../xgen-core/src/node/runtime.rs#L1996)) **out-of-band of any event**,
  then drains ([`:2001`](../xgen-core/src/node/runtime.rs#L2001)). **It is only called from one site:**
  the receiver/initiator hook inside `stream_federation_delta`
  ([`federation_session.rs:120`](../xgen-node/src/federation_session.rs#L120)), i.e. **at
  federation-session establish**, not on member-join (§2.3).
- **`repopulate_regular_federation_nodes`** ([`runtime.rs:2110`](../xgen-core/src/node/runtime.rs#L2110),
  C5) — re-unions the established-relationship set into `federation_nodes` at the **3 `derive_resolved`
  rebuild sites** ([`:537`](../xgen-core/src/node/runtime.rs#L537),
  [`:692`](../xgen-core/src/node/runtime.rs#L692),
  [`:717`](../xgen-core/src/node/runtime.rs#L717)) so the relationship survives a rebuild. It is **fed
  by the relationship record `establish_federation_relationship` writes** — it does not discover new
  peers; it only preserves what establish already recorded.

**`apply_join` does NOT touch `federation_nodes`.**
([`state.rs:982-1020`](../xgen-core/src/space/state.rs#L982)) — a space-level or room-level join
mutates only `members` / `rooms[].members` / `pending_invites` / `banned`. **So for a regular Space,
a member joining on node `n_k` does not add `n_k` to `federation_nodes[S]` on any node.** Member-join
and federation-target growth are **decoupled** for regular Spaces. (Contrast: a **DM** re-derives
`federation_nodes` from `members ∪ pending_invitees → home_nodes` at every apply via
`repopulate_dm_federation_nodes` ([`runtime.rs:2081`](../xgen-core/src/node/runtime.rs#L2081)) — but
that helper is `dm_constraints_active`-gated and DM-only; it early-returns for regular Spaces. The DM
has membership-driven federation; the regular Space does not.)

### 2.3 Contrast with C5/MP-F11 — what it fixed, and the precise reason it does NOT cover gap-2

C5/MP-F11 (`9ac7780`, R3-D6) generalized Design-Z to regular Spaces **on the federation-establish
path**. Its mechanism (grounded):

- `establish_federation_relationship` fires from `stream_federation_delta`
  ([`federation_session.rs:105-135`](../xgen-core/src/node/runtime.rs), the establish block before the
  per-Space stream loop), which runs inside `run_federation_session_post_handshake`
  ([`app.rs:2177`](../xgen-node/src/app.rs#L2177)). **Both** ends run it: the receiver
  (`SessionRole::Receiver`, [`app.rs:2004-2006`](../xgen-node/src/app.rs#L2004)) and the initiator
  (`SessionRole::Initiator`, [`reconnect.rs:494-496`](../xgen-node/src/reconnect.rs#L494)). So at a
  **session establish**, each side populates its own `federation_nodes[S]` from the established
  relationship **and** streams its own full delta to the peer.
- This breaks the **mutual hold** MP-F11 named: a late peer's content is F-3-held because
  `federation_nodes` lacks the pusher, and the `state.federation_add` that would populate it can
  itself be predecessor-held (it references the held content). Populating from the **established
  relationship** (out-of-band of the event) drains the held **inbound** content.

**Why C5 does NOT cover gap-2 (the load-bearing distinction):**

1. **C5 fires on session establish, not on `federation_nodes` growth over a live session.** MP-C-14 is
   **early-federate**: the n0↔leaf federation sessions are set up around Space-creation (§2.4). The
   late **member-join** is a separate event over the **already-live** session — it triggers **no new
   `stream_federation_delta`**, so `establish_federation_relationship` **never re-fires** for it. This
   is exactly the finding's "no new establish."
2. **C5 drains held INBOUND content; gap-2 is missing OUTBOUND content.** C5's drain releases content a
   peer **pushed to us** and we F-3-held. In MP-C-14, `p0` was authored on **n0** and **never pushed**
   to the late leaf (the leaf's node wasn't in n0's `federation_nodes[S]` at `p0`-send, §2.1) — so
   there is **nothing F-3-held on the leaf to drain.** The content was simply never sent. C5's drain
   has no effect on an event that never left the sender.
3. The C5 hole-closed spine (`mp_f11_third_party_regular_space_content_blocked_by_f3`,
   [`runtime.rs:3047-3083`](../xgen-core/src/node/runtime.rs#L3047)) proves F-3 still blocks
   third parties after C5 — that invariant is the safety surface gap-2's fix must also hold (§4.3).

**Summary:** C5 closed the *late-establish catch-up* path (a new relationship forms → drain the
inbound hold). Gap-2 is the *late-member-join over an existing relationship* path (the set grows, but
the sender never re-pushes its existing content, and no establish re-fires). Different trigger,
different direction.

### 2.4 The MP-C-14 harness timing — where the race lives

The scenario (`mp_r3_topology.rs:39-71`):
- **a0 (owner, @n0):** `register` → `create-space` → `create-room` → `send "a0-hello"` id `p0`
  **`after_ms:40`** ([`:50`](../xgen-mptest/tests/mp_r3_topology.rs#L50)).
- **a1@n1 / a2@n2 / a3@n3 (leaves):** `register` → `join` (open-join, cross-node) → `send "aN-hello"`
  **`after_ms:40`** ([`:53-63`](../xgen-mptest/tests/mp_r3_topology.rs#L53)).
- Federation = `StarPlusMesh` → the generator emits a **full mesh** of `C(4,2)=6`
  `[[federation]]` links ([`sweep.rs:456-471`](../xgen-mptest/src/sweep.rs#L456)).

Harness federation setup (`runner.rs`):
- **Pre-seed (before the drive):** for every link with `after: None` (all 6 here), `add-peer` **both
  directions with EMPTY spaces** ([`runner.rs:376-382`](../xgen-mptest/src/runner.rs#L376)) — records
  the peer URLs; **does not name a Space, does not initiate.**
- **G-6 tail (the director, concurrent with the drive):** the director processes links
  **sequentially** ([`runner.rs:699`](../xgen-mptest/src/runner.rs#L699)); for each early link it
  **waits for the owner's `space_id` export**, then re-`add-peer` on the **`from` side naming S** +
  `node_initiate` from `from` ([`runner.rs:729-736`](../xgen-mptest/src/runner.rs#L729)). (Note the
  early-link branch re-seeds **only the `from` side** naming S — contrast the late-link branch
  ([`:717-722`](../xgen-mptest/src/runner.rs#L717)) which names S on **both** sides.)

**The race:** `p0` fires at `after_ms:40` (≈40 ms after `create-room`), **concurrent** with the
director's `space_id` wait + 6 **sequential** `add-peer`+`initiate` round-trips against real binaries.
`p0` is pushed **far earlier** than the director can establish all 6 links. At `p0`-push-time, n0's
`federation_nodes[S]` contains only whichever leaf nodes the director's establish loop has already
reached. Leaf links that establish **after** `p0`:
- are not in `federation_nodes[S]` at `p0`-push → `p0` not live-pushed to them (§2.1); and
- their establish-stream (if it carries `p0` — see §3) is the only other route, which the observed
  failure says is missed ~60% of the time.

This is the early-post + sequential-establish window the finding pinned: **a row that posts content
in the gap between Space-creation and federation-establish-complete is exactly the structural class
MP-F14 names.**

### 2.5 The local-vs-cross-node new-joiner asymmetry (a precise, citable contrast)

A **local** new joiner *does* get history backfilled: `apply_fanout` sends the Space's full
topologically-sorted history to a `new_joiner` on the **same node**
([`fanout.rs:246-259`](../xgen-node/src/fanout.rs#L246), gated on `req.new_joiner.is_some()`). There
is **no cross-node equivalent**: when a leaf joins on `n_k`, the existing content on n0 is **not**
re-streamed to `n_k`. The backfill primitive exists at the local-fanout layer; the gap is that it has
no federation-layer sibling for a late cross-node member. (`derive_event_nodes`,
[`fanout.rs:177`](../xgen-node/src/fanout.rs#L177), is the `.events`-observer node-set deriver, **not**
the federation push set — federation push targets come solely from `federation_nodes[S]` in §2.1; do
not conflate the two.)

---

## 3. The pin-by-observation item (design/runbook deliverable — name it, don't guess it)

The structural gap (§2) is certain. The **exact MP-C-14 failing path** has two candidate sub-mechanisms
that a static read cannot decide between, because both sides of an establish *should* populate +
stream (§2.3). The design must pin which one fires **by observation** — the same discipline that
pinned the finding (8 runs), and the MP-F9 runbook-exec-step-1 precedent:

- **Candidate (i) — `node_initiate` reuses the pre-seeded session and does NOT re-run the
  post-handshake establish.** The pre-seed ([`runner.rs:376-382`](../xgen-mptest/src/runner.rs#L376))
  opens nothing (empty-space `add-peer`), but if the node's `federation_initiate` on an
  already-known/connected peer no-ops or reuses rather than running a fresh
  `run_federation_session_post_handshake`, then C5's `establish_federation_relationship` **never fires**
  for the S-naming, and `federation_nodes[S]` is populated **only** by the racing `state.federation_add`
  apply (§2.2) — which can be predecessor/F-3-held. This matches the finding's "no new establish
  re-fires" literally. (Ground `admin_ops::federation_initiate` — does it always dial a fresh session?)
- **Candidate (ii) — the establish fires but in an ordering where `p0` is neither streamed nor
  live-pushed.** e.g. the n0↔leaf establish completes **before** `p0` is posted (empty stream), but
  the establish populated n0's `federation_nodes[S]` such that `p0`'s later live push *should* reach
  the leaf — yet the observed failure says it doesn't, implicating an unexpected ordering or a
  one-directional populate.

**The pin (design exec-step-1):** re-run `mp_r3_topology` with a bounded diagnostic that logs, per
run: (a) n0's `federation_nodes[S]` **at the instant `p0` is pushed** (`apply_federation_push`
snapshot, §2.1); (b) the `establish_federation_relationship` firings + their `federation_nodes`
deltas (which links, in what order, relative to `p0`); (c) which node ends **missing `p0`**. This
distinguishes (i) from (ii) and fixes the precise **fix site** before any production change — the
MP-R2/R3 pin-by-observation bar. The route is locked (fix-it gate item); the *site* is what the trace
pins.

---

## 4. Fix surface — candidate directions (NOT locked; for the design beat)

All candidates reuse the shipped Design-Z / MP-F11 machinery (`federation_relationships`,
`establish_federation_relationship`, `repopulate_regular_federation_nodes`,
`drain_pending_by_federation_relationship`). The gap is a **trigger on the live path**, not new
convergence math — so the fix may land **bounded** (machinery reuse), the §6 posture.

### 4.1 Fork A — re-push existing content when `federation_nodes[S]` grows (sender-side, the direct gap)

When a peer enters `federation_nodes[S]` on an already-live session (the §2.2 growth points), stream
that Space's existing history to the newly-added peer — a federation-layer sibling of the local
`new_joiner` backfill (§2.5). Natural homes: the `establish_federation_relationship` site (extend it to
also **re-stream outbound** to the new peer, not only drain inbound), or a hook where `federation_nodes`
grows on the live path. Closes the gap at its root (§2.1: no re-push today). The §3 pin decides whether
the trigger is "on establish re-fire" (if candidate-i, make establish re-fire / cover the growth) or
"on `federation_nodes` growth over a live session" (if candidate-ii).

### 4.2 Fork B — make the late member-join itself establish/stream the Space onto the joiner's node

Generalize the membership-driven federation the DM already has (`repopulate_dm_federation_nodes` at
membership-apply) to a regular-Space sibling that, on a cross-node member-join, establishes the
relationship + streams existing content to the joiner's node. Heavier (touches the regular-Space
membership-apply path); weigh against Fork A's narrower "on relationship growth" trigger.

(The design grounds the §3 pin first, then picks; the audit does not lock the fork — pin-by-observation
before routing the *site*.)

### 4.3 The J-333 hole-safety surface — mandatory, whichever fork

The fix **must not** weaken F-3. The F-3 gate
([`runtime.rs:1057-1071`](../xgen-core/src/node/runtime.rs#L1057), `skip_f3` only for
`StateFederationAdd | StateSpaceCreate | StateDmSpaceCreate`; everything else — incl. `message.text`
(`p0`) and `membership.join` — is held unless the pusher is in `federation_nodes[S]`; `f3_reject` at
[`:1101-1108`](../xgen-core/src/node/runtime.rs#L1101)) is the receiver-side guard that keeps a node
from accepting content from a non-federated peer. The J-333 lesson (an unconditional F-3 skip is a
hole) and the C5 hole-closed spine
([`runtime.rs:3047-3083`](../xgen-core/src/node/runtime.rs#L3047)) require: **only legitimate members /
established relationships get the backfill; a third party stays F-3-blocked.** The MP-F14 fix's spine
(design deliverable) must include a **RED-on-revert hole-closed assertion** mirroring C5's — a
non-member / non-established node must NOT receive the re-pushed content, and must NOT enter
`federation_nodes[S]`. Re-pushing to a node already in `federation_nodes[S]` for a Space it is a
legitimate federation peer of is safe by construction (it is the relationship F-3 already trusts); the
care is to never re-push to (or populate for) a node that is not.

---

## 5. Coverage enrichment (Clair, J-351 — named, not optional)

The MP-C-14 smoke **under-exercises leaf-authored content.** Each leaf's `send` fires at `after_ms:40`
**immediately after** its `join` ([`mp_r3_topology.rs:61-63`](../xgen-mptest/tests/mp_r3_topology.rs#L61)),
so the leaf post **races its own join** — it lands nowhere reliably (the leaf's node is not yet an
established federation target for the other leaves at send-time, the same §2.1/§2.4 race). The
consequence: **only a0's `p0` is cross-node content genuinely under test**, and that is the one event
the convergence oracle catches missing. A *green* MP-C-14 today would **not** prove leaf posts
converge cross-node.

**Enrichment (the arc enriches the smoke so a green exercises leaf posts):** make each leaf's `send`
depend on its `join` having landed (e.g. gate the send on the join reply / a settle, or sequence it
after the join is confirmed converged) so leaf-authored content is **genuine cross-node content under
test**. This is a `xgen-mptest` change (the MP-C-14 template `actor_batch` + possibly the
`StarPlusMesh` generator path, `sweep.rs:456-471`); it tightens the oracle's actual coverage so the
MP-F14 fix's GREEN-on-rerun proves the property it claims (every member sees every post, including
posts authored both before AND after a late join, by the creator AND by leaves). The enrichment ships
with the fix's box-gated witness, not separately.

---

## 6. Scope, posture, and what this is NOT

- **In scope:** the regular-Space pre-join-message backfill gap (§2) + the MP-C-14 coverage enrichment
  (§5). One production-crate fix (the §4 fork, Joe-locked at design) + the test-crate smoke enrichment.
- **NOT this arc:** MP-C-16 / MP-F13 (home_node WS-URL vs pubkey-id; J-278/F1B-D5; M10+ carve-out — a
  *different* root, the audit does not touch it). MP-A-08's transport-level reconnect-deadlock half
  (R3-D2, routed). The residents-multiplexer (R3-D1, routed). DM federation (Design-Z / MP-F1b,
  shipped; the regular path is the additive sibling — DM keeps members∪invitees).
- **R3-grade, no carve-out:** confirmed against the rerun-to-green discipline — MP-F14 has no
  later-milestone home and R3 is the last round, so it is fixed-in-round, terminal = GREEN-on-rerun.
- **Convergence-safety + D-076:** the fix is a federation **delivery** trigger (re-push existing
  content to a legitimate peer); it changes *who receives* an already-resolved event, not the
  resolution/ordering — D-076 discharged-by-inheritance if it reuses the C5 drain/populate (the design
  proves this, sibling to C5's discharge). The spine is RED-on-revert (the §4.3 hole-closed assertion +
  the backfill-drains assertion), `xgen-core`, mirroring C5's `mp_f11_*` spine.
- **Arc-local decisions** carry **MP-F14-D#** tags (D-069); none is expected to clear the
  global-principle bar (no DECISIONS promotion in this arc). The standing promotion candidates
  (loop-to-green-with-a-bounded-gate round-close; pin-by-observation-before-routing) remain Joe's call,
  unchanged here.

---

## 7. Discipline + next step

- Surface-and-route (D-065/D-084); **pin-by-observation BEFORE routing the fix site** (§3). No code in
  Phase-0; no design lock until Joe's call.
- No self-close. Clair's audit (then design) commits **FIRST** (Joe pushes); Chat's doc-bridge is a
  separate commit. Chat never pushes.
- **Next step: `tasks/MP_F14_PREJOIN_BACKFILL_DESIGN.md`** — lock the §4 fork (after the §3
  pin-by-observation re-trace grounds candidate (i) vs (ii)), the J-333 hole-safety spine (§4.3), and
  the §5 coverage enrichment, then the runbook → implement (spine-first, RED-on-revert) → **R3 rerun**
  to all-green-except-MP-C-16 → MP-R3 close + the consolidated R1+R2+R3 ledger (the standing HANDOFF
  deliverable).

**Entry point (Rule 0):** CLAUDE.md PLAY (J-351 RUN-#1/fix-phase head) → JOURNAL J-351 →
`tasks/HANDOFF_MP_R3.md` → `tasks/MP_findings.md` (MP-F14 + the J-351 fix-phase note) → this audit →
`tasks/MP_R3_CAPSTONE_DESIGN.md` §5.1 (the C5/MP-F11 contrast) → `docs/tests/MULTIPARTY_TEST_MATRIX.md`
§6 (the RUN-#1 record).
