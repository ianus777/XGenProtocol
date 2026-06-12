# MP-F14 — regular-Space pre-join-message backfill — design (MP-F14-D1..D7)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

The design beat for **MP-F14**, the **sole MP-R3 fix-phase gate item** (J-351). It consumes the
grounded Phase-0 audit (`tasks/MP_F14_PREJOIN_BACKFILL_AUDIT.md`) and locks the arc's decisions as
**MP-F14-D1..D7** (arc-local, D-069 — none clears the global-principle bar for DECISIONS promotion).
**No code, no runbook** — the runbook (`tasks/MP_F14_PREJOIN_BACKFILL_IMPL.md`) is authored after Joe's
locks; its **exec-step-1 is the re-trace** (§3) that the fork (§4) locks against.

**The route is Joe-locked (J-351):** MP-F14 is **R3-grade, fixed-in-round, NOT a carve-out** (no
later-milestone home, R3 is the last round; only MP-C-16/MP-F13 carves on its genuine M10+ blocker).
Terminal = **(a) GREEN-on-rerun**.

**The MP-F9 discipline is the spine of this design (read it before locking anything).** In the MP-F9
arc, the design's mechanism hypothesis (F-3/Design-Z held-then-drained) was **FALSIFIED by exec-step-1's
observation** (the children were *rejected* "space not found", not held) and the fix was re-locked. So
this design **does not pre-lock the mechanism.** The structural gap is certain (§2); the **exact MP-C-14
failing path** has candidate sub-mechanisms a static read cannot decide between (§3), so the design
**enumerates the forks + the trace→fork mapping + a recommendation**, and the **fork locks AFTER the
exec-step-1 re-trace** (MP-F14-D2). The trace is **authoritative** — if it falsifies the backfill
premise entirely, §8 is the conditional-terminal branch.

**Method (the MP-R2/R3 bar):** surface-and-route (D-065/D-084); **pin-by-observation BEFORE routing the
fix site.** Honest boundaries recorded.

---

## 0.1 — EXEC-STEP-1 RE-LOCK (2026-06-12, Joe-LOCKED): Fork A FALSIFIED → `get_dag_tips` infra-exclusion (MP-F14-D7 fired)

**The box-gated re-trace ran (runbook §2; throwaway diagnostic, reverted — clean tree) and decisively
falsified Fork A, exactly the MP-F9 risk §0/§8 named.** This section is the authoritative re-lock; §4
(Fork A/B) + §5 (the J-333 hole-safety spine) are **superseded for the mechanism** and kept below as the
historical conditional the trace resolved (the §8/D7 branch was taken).

**What the trace showed (grounded across the kept node logs, 2 captures):**
- The victim leaf's node **WAS** in n0's `federation_nodes[S]` at p0's push (established ~2.2 s before
  p0; `fed_nodes={n1,n2,n3}`), p0 **was** pushed (`federation_push_sent` ✓) and **received** on the leaf.
  → **Fork A's premise (missing outbound re-stream on federation_nodes growth) is FALSE**; Fork B (member-
  join-driven federation) is equally off-target. The federation_nodes were correct; delivery happened.
- On the leaf, p0 is **`HeldPending` for a missing causal predecessor** (the F-4a buffer,
  [`runtime.rs:1193`](../xgen-core/src/node/runtime.rs#L1193)) that **never arrives → never drains →
  stuck forever**.
- p0's `prev_events` = the **`get_dag_tips` frontier** ([`xgen-client/src/batch.rs:96`](../xgen-client/src/batch.rs#L96),
  the MP-F4 fix): 5 tips = 3× `membership.join` + `state.room_create` + **one `state.federation_add`**.
  The missing predecessor is that **`state.federation_add`** — a **vantage-specific / directional** infra
  event (D-075) that the convergence oracle **excludes** (MP-R1-D7, [`oracle.rs:47-57`](../xgen-mptest/src/oracle.rs#L47)),
  which is precisely why the verdict flags **only p0**, not the missing federation_add.
- **The ~60 % intermittency = the race:** whether a `state.federation_add` is a `get_dag_tips` frontier
  tip at p0's `after_ms:40` send (concurrent with the G-6 federation establish). `compute_frontier`
  ([`batch.rs:125`](../xgen-client/src/batch.rs#L125)) does **not** filter by event kind.

**The root (re-locked):** **`get_dag_tips` anchors a new cooperative event's `prev_events` to the full
DAG frontier, including vantage-specific `state.federation_add` (infra) tips that never converge cross-
node** → the cooperative event is un-applyable on any peer lacking that federation_add. This is the
**MP-F4 / J-331 causal-frontier family**: MP-F4 widened `get_dag_tips` to the full frontier (to fix room-
join anchoring); that same frontier now over-anchors cooperative content to non-converging infra events.

**MP-F14-D2 RE-LOCKED (Joe, by-recomm) — `get_dag_tips` cooperative-frontier (infra-exclusion).** The fix
excludes infra / vantage-specific event kinds (`state.federation_add`; the oracle's `INFRA_EVENT_KINDS`
is the canonical set) from the frontier `get_dag_tips` returns for a new cooperative event's
`prev_events`, so DAG-anchoring **agrees with the convergence contract** (what does not converge cross-
node is never a cooperative predecessor). **Site:** the collection loop in `get_dag_tips`
([`batch.rs:145-161`](../xgen-client/src/batch.rs#L145)) — the full `Event` (incl. `ev.event_type`) is in
hand there, so skip infra-kind events from **both** `seen` and their contribution to `referenced` (so
`compute_frontier` yields the **cooperative DAG sub-frontier**; not just dropping infra from the final
set, which would wrongly orphan a cooperative tip an infra event referenced). Client-side; one function;
all cooperative callers (`ops::send` / `ops::join` / `ops::leave`) inherit it. **NOT** the federation_nodes
outbound re-stream (Fork A/B superseded).

**MP-F14-D4 RE-LOCKED — the spine re-shapes (the J-333 hole-safety lens NO LONGER applies — this is not
an F-3 path).** The relevant safety is now **"do not drop a real cooperative predecessor"** (the MP-F4
no-regression). Two RED-on-revert spine tests, **client-side** (`xgen-client`; the pure
collect-then-`compute_frontier` is extracted/unit-testable):
- **Spine #1 (fix-proof) — `mp_f14_cooperative_frontier_excludes_federation_add`:** a frontier built over
  a DAG whose current leaves include a `state.federation_add` returns the cooperative tips **without** the
  federation_add → a new cooperative event does not anchor to it (and so applies on a peer lacking it).
  **RED-on-revert:** without the infra-exclusion, the frontier includes the federation_add → the built
  event anchors to it.
- **Spine #2 (MP-F4 no-regression) — `mp_f14_cooperative_frontier_keeps_membership_and_room_tips`:** a
  room-join / message still anchors to **all** cooperative leaves (`membership.join`, `state.room_create`,
  message tips) → MP-F4's concurrent-leaf fix is preserved (only infra is excluded). **RED-on-revert:** an
  over-broad exclusion that drops cooperative tips → the room-join goes concurrent again (the MP-F4
  finding returns).

**MP-F14-D7 — FIRED + recorded (the honest falsification, D-065).** The trace falsified Fork A; the
mechanism re-shaped to the `get_dag_tips` causal-anchoring root. **Gate disposition UNCHANGED:** MP-F14
stays a fix-in-round gate item, terminal = GREEN-on-rerun. **D1/D3/D5/D6 stand** (the certain gap class
"a member silently misses content" holds; D3 box-gated re-trace = done; D5 enrichment unchanged; D6
convergence-safety / scope fence unchanged). **§4 (forks) + §5 (J-333 spine) are superseded for the
mechanism** and retained below as the historical conditional.

**MP-F4 interaction caveat (the one thing the spine must guard):** the fix lives in the MP-F4 frontier
code, so Spine #2 exists specifically to prove the infra-exclusion does **not** re-open MP-F4's room-join
anchoring (it cannot — membership/room events stay in the frontier; only `state.federation_add` is
excluded — but it is proven, not assumed).

---

## 1. The locked route + state at open

- **MP-R3 RUN #1** (J-351): all-green-except-{MP-C-16, MP-C-14}; node ceiling ~1384. MP-F11 RESOLVED
  (MP-A-01(ii) 3/3), MP-A-08/CHAOS/A-06 green. The capstone holds on **one** finding.
- **MP-F14** = the regular-Space pre-join-message backfill gap (MP-C-14): a leaf joining a Space
  **after** the creator's post `p0` never receives `p0`. **Gap-2** of the J-333/MP-F1b/MP-F11 family,
  **distinct from C5/MP-F11** (gap-1 = late-establish catch-up, which MP-A-08 + MP-A-01(ii) prove
  works).
- **Bounded gate = {MP-F14} only.** Newly-surfaced bugs face-and-route to their homes but do **not**
  extend the gate. When MP-F14 is terminal → **R3 rerun** to all-green-except-MP-C-16 → MP-R3 close +
  the consolidated R1+R2+R3 ledger.

---

## 2. MP-F14-D1 — the certain structural gap (LOCKED; the invariant, not the site)

Grounded in the audit §2, and **certain regardless of the trace**: **federation push is send-time-only,
with no outbound re-push/re-stream when `federation_nodes[S]` grows over an already-live session.**

- `apply_federation_push` ([`federation_session.rs:280`](../xgen-node/src/federation_session.rs#L280))
  reads `federation_nodes[S]` **once, at the push instant** ([`:331-337`](../xgen-node/src/federation_session.rs#L331))
  and pushes only to that snapshot. There is no "sent-to-set" record and no replay.
- The **outbound catch-up stream** (n0 → a peer, full history) fires **only** inside
  `stream_federation_delta` at **session post-handshake**
  ([`federation_session.rs:84`](../xgen-node/src/federation_session.rs#L84), run from
  `run_federation_session_post_handshake`, [`app.rs:2177`](../xgen-node/src/app.rs#L2177); both roles —
  receiver [`app.rs:2004`](../xgen-node/src/app.rs#L2004), initiator
  [`reconnect.rs:494`](../xgen-node/src/reconnect.rs#L494)). It is the surrounding session-setup stream
  that delivers existing content; C5's `establish_federation_relationship`
  ([`runtime.rs:1983`](../xgen-core/src/node/runtime.rs#L1983)) only **drains inbound** held content +
  records the relationship.
- `apply_join` ([`state.rs:982-1020`](../xgen-core/src/space/state.rs#L982)) never touches
  `federation_nodes` → for a regular Space, member-join and federation-target growth are **decoupled**.

**The invariant the fix must establish (MP-F14-D1, LOCKED):** *when a node `N`'s `federation_nodes[S]`
for a regular Space gains a legitimate peer `P` after `S` already holds content, `N` ensures `P`
receives `S`'s existing content* — by an outbound catch-up re-stream to `P` (the net-new direction),
**with F-3 intact** (only legitimate established peers; §5). The **site/trigger** of that re-stream is
the fork (§4), conditional on the §3 trace.

**Honest scope of "reuse" (D-065):** the reusable machinery exists — the full-history delta
(`compute_federation_delta_for_space(.., None)`,
[`fanout.rs:605`](../xgen-node/src/fanout.rs#L605)), the relationship record + F-3 authority + inbound
drain (`establish_federation_relationship`), the peer senders (`FederationPeerSenders`), the rebuild
survival (`repopulate_regular_federation_nodes`,
[`runtime.rs:2110`](../xgen-core/src/node/runtime.rs#L2110)). The **net-new** is the **outbound
re-stream TRIGGER on the live growth path** (today the outbound stream fires only at session setup). So
MP-F14 is *not* pure C5 reuse — it is a new trigger wired to reused stream + safety machinery. It may
land **bounded** (the heavy lifting is the trigger placement, which the trace pins), but it is a real
production-crate change.

---

## 3. MP-F14-D3 — the exec-step-1 re-trace (the pin; box-gated) [LOCKED]

The fork (§4) **locks against this trace, not before it** (the MP-F9 exec-step-1 precedent — the
design's hypothesis can be falsified; the trace is authoritative). Joe authorizes a **bounded throwaway
diagnostic** (keep-artifacts; instrumentation reverted after — the MP-F11/MP-F13 pin precedent), run
against the real `mp_r3_topology` scenario.

### 3.1 The diagnostic (precise log points)

Add bounded `tracing` at four points, keyed by `(space_id, event_id, node)` so the trace is correlatable
across the 4 binaries:

1. **`apply_federation_push` push-snapshot** ([`federation_session.rs:331-380`](../xgen-node/src/federation_session.rs#L331)):
   on every push, log `local_node_id`, `event_id` (= `p0`'s id), and the **full `federation_nodes[S]`
   set** at the push instant + the peers actually `try_send`-ed. → *Answers: which peers did `p0` get
   pushed to, and was the missing leaf's node in the set at that instant?*
2. **`establish_federation_relationship` firing** ([`runtime.rs:1983`](../xgen-core/src/node/runtime.rs#L1983)
   + the hook [`federation_session.rs:116-120`](../xgen-node/src/federation_session.rs#L116)): log
   `(space_id, peer, federation_nodes-before, federation_nodes-after, drained_count)` on every call. →
   *Answers: did the missing leaf's establish fire at all (candidate i vs ii)? when, relative to `p0`?
   did the surrounding `stream_federation_delta` outbound stream run?*
3. **`state.federation_add` growth + Step-7 drain pair** ([`runtime.rs:1570`](../xgen-core/src/node/runtime.rs#L1570)
   / [`:1619-1621`](../xgen-core/src/node/runtime.rs#L1619)): log when n0's `federation_nodes[S]` grows
   via a `federation_add` apply (peer added) + the drain firing. → *Answers: is the growth event-driven
   (federation_add) rather than establish-driven, and does it post-date `p0`?*
4. **The miss** (oracle / transcript at settle): log the exact `(node, missing event_id)` the
   convergence verdict flags (it already computes this; surface it). → *Answers: which node ends missing
   `p0`.*

### 3.2 Box-gated vs in-process — and the recommendation

**Recommendation (LOCKED for the pin): the re-trace is BOX-GATED.** The gap is a **real-timing,
multi-binary phenomenon** — `p0` fires at `after_ms:40` concurrent with the director's **sequential,
`space_id`-gated 6-link** establish loop (audit §2.4). An in-process harness cannot faithfully
manufacture that establish-vs-`after_ms:40` race; the finding itself was pinned on the box (8 RUN-#1
runs). So exec-step-1 instruments the production crate + re-runs `mp_r3_topology` (`--features
harness-control`) and reads the trace. (This is the *pin*; the *fix-proof* spine is in-process — §5.3 —
and the *witness* is the box-gated rerun — §6/§4.5. Same split as C5: spine in-process RED-on-revert,
witness box-gated at the RUN.)

The in-process layer is **not** the pin but **is** where the fix's spine lives (a deterministic
structural repro of the invariant: grow `federation_nodes[S]` after content exists, assert the peer
gets the content + a third party stays F-3-blocked — §5).

---

## 4. MP-F14-D2 — the two forks (RECOMMENDED; locks AFTER the §3 re-trace)

Both forks satisfy MP-F14-D1 (outbound catch-up on growth, F-3 intact); they differ in the **trigger
site**. The trace (§3) selects which growth-point is the live one in MP-C-14.

### 4.1 Fork A — outbound re-stream at the relationship/`federation_nodes`-growth point (the rec)

Hook the outbound catch-up to the point where a regular Space's `federation_nodes` gains a peer over a
live session. Concretely: when the relationship (re-)establishes or `federation_nodes[S]` grows on the
live path, **re-stream `S`'s existing content outbound to the newly-added peer** —
`compute_federation_delta_for_space(rt, S, None)` (full history) → the peer's `FederationPeerSenders`
channel, **in addition to** the inbound drain `establish_federation_relationship` already does. Candidate
sites (the trace picks): the establish hook ([`federation_session.rs:116-120`](../xgen-node/src/federation_session.rs#L116))
made to fire on re-initiate over an existing session; or the `state.federation_add` growth/drain-pair
point ([`runtime.rs:1570/1619`](../xgen-core/src/node/runtime.rs#L1570)) gaining an **outbound** sibling
to its inbound drain.

- **Crate split:** the relationship record + F-3 authority + inbound drain stay `xgen-core`
  (`establish_federation_relationship`); the **outbound re-stream is `xgen-node`** (it needs
  `FederationPeerSenders` + `compute_federation_delta_for_space`, which live node-side). The trigger
  may be signalled from `xgen-core` (e.g. "this growth needs an outbound catch-up to peer P") and
  executed `xgen-node`, mirroring the existing inbound `drain → persist` split.
- **Why the rec:** narrowest reuse of the shipped machinery; does **not** re-couple membership↔federation
  for regular Spaces (which they deliberately don't do today, audit §2.2); the trigger is the relationship
  growth (the F-3 authority), so the hole-safety (§5) is checked at exactly the point F-3 already trusts.

### 4.2 Fork B — outbound re-stream driven by the cross-node member-join

On a cross-node member-join for a regular Space, establish + stream existing content to the joiner's
node — a regular-Space sibling of the DM's membership-driven federation
(`repopulate_dm_federation_nodes`). Heavier: touches the regular-Space membership-apply path and
**re-couples membership to federation** for regular Spaces. Reserved for the case where the trace shows
the growth is fundamentally membership-driven and the establish/`federation_add` path does **not**
reliably add the leaf's node (so Fork A has no growth-point to hook).

### 4.3 The trace → fork mapping (how §3 decides)

| Trace observation (§3) | Reading | Fork |
|---|---|---|
| establish (S, missing-leaf) **never fires**; `federation_nodes[S]`←leaf comes (if at all) from a racing `federation_add` post-`p0` | candidate (i): re-initiate reused the pre-seeded session, C5 didn't re-fire | **Fork A**, trigger = the `federation_add` growth point (or make establish re-fire on re-initiate) + add the outbound re-stream |
| establish (S, leaf) **fires after `p0`** but `p0` not carried outbound / leaf still misses `p0` | candidate (ii-b): establish populated `federation_nodes` but no outbound catch-up for existing content | **Fork A**, trigger = the establish hook gains the outbound re-stream |
| establish (S, leaf) **fires before `p0`**, leaf IS in `federation_nodes[S]` at `p0`-push, yet leaf misses `p0` | a **delivery** bug (`try_send` drop / sender unregistered / F-5), not a backfill gap | **§8 falsification branch** — route the delivery bug; re-shape |
| growth is purely membership-driven; no establish/`federation_add` adds the leaf's node | gap-2 is at the membership layer | **Fork B** |

**Recommendation (conditional):** the most likely outcome is **Fork A** (the structural gap is the
missing outbound re-stream on growth; the audit's §2 surfaces all point there). **Locked after the
exec-step-1 trace confirms which trigger site is live** (MP-F14-D2). Joe locks the fork by-recomm in the
MP-R2-D# pattern, against the observed trace.

### 4.4 What both forks reuse vs add (honest)

- **Reuse:** `compute_federation_delta_for_space(.., None)` (full history), `FederationPeerSenders`
  (outbound), `establish_federation_relationship` (relationship record + F-3 authority + inbound drain),
  `repopulate_regular_federation_nodes` (rebuild survival), `drain_pending_by_federation_relationship`.
- **Net-new:** the **outbound re-stream trigger on the live growth path** (today only at session setup).
  ES-D4 persistence of any re-streamed/drained events (sibling to C5's persist after establish,
  [`federation_session.rs`](../xgen-node/src/federation_session.rs)).

### 4.5 Spine-first + RED-on-revert (LOCKED, both forks)

Like C5: the fix's spine is proven **in-process, RED-on-revert, before the box witness**. The witness
(box-gated `mp_r3_topology` rerun, enriched per §6) flips RED→GREEN at the R3 rerun; the runbook is
spine-first.

---

## 5. MP-F14-D4 — the J-333 hole-safety spine (MANDATORY, RED-on-revert) [LOCKED]

The fix **must not weaken F-3.** The F-3 gate
([`runtime.rs:1057-1071`](../xgen-core/src/node/runtime.rs#L1057): `skip_f3` only for
`StateFederationAdd | StateSpaceCreate | StateDmSpaceCreate`; everything else — incl. `message.text`
(`p0`) and `membership.join` — is held unless the pusher is in `federation_nodes[S]`; `f3_reject`
[`:1101-1108`](../xgen-core/src/node/runtime.rs#L1101)) is the receiver-side guard. The J-333 lesson (an
unconditional F-3 skip is a hole) and the C5 hole-closed spine
([`runtime.rs:3048`](../xgen-core/src/node/runtime.rs#L3048),
`mp_f11_third_party_regular_space_content_blocked_by_f3`) require:

**The outbound re-stream targets ONLY legitimate established peers / members; a third party (a node not
in `federation_nodes[S]` as a legitimate relationship) never receives the re-pushed content and never
enters `federation_nodes`.** Re-pushing to a peer that is *already* a legitimate `federation_nodes[S]`
member is safe by construction (it is the relationship F-3 already trusts); the care is never to
re-push to (or populate for) one that is not.

### 5.1 The spine tests (hard deliverable, mirrors C5's `mp_f11_*`)

`xgen-core` (and/or `xgen-node` for the outbound half), deterministic, RED-on-revert:

- **Spine #1 — backfill-on-growth delivers (the fix proof):** a regular Space holds content `c` (posted
  while peer `P` was NOT a federation target); then `P` legitimately enters `federation_nodes[S]`
  (the fork's growth point); assert `P` receives `c` (the outbound re-stream fired). **RED-on-revert:**
  neuter the outbound re-stream → `P` does not get `c` → fails.
- **Spine #2 — hole-closed (the J-333 safety):** establishing/growing for legitimate peer `A` does NOT
  re-stream `S`'s content to a third party `B` (not in `federation_nodes[S]` / not a member); `B` stays
  F-3-blocked and never enters `federation_nodes[S]`. **RED-on-revert:** an over-broad re-stream (to
  all peers / skipping the legitimacy check) → `B` gets `c` → fails. Sibling to
  `mp_f11_third_party_regular_space_content_blocked_by_f3`.

Both go RED when the respective half is neutered (the C5 pattern). **D-076 discharged by inheritance**
(the re-stream delivers an already-resolved event to a legitimate peer — it changes *who receives*, not
ordering/resolution; the delta-compute is the proven canonical full-history path,
[`fanout.rs:615-627`](../xgen-node/src/fanout.rs#L615)).

---

## 6. MP-F14-D5 — the MP-C-14 leaf-content coverage enrichment (LOCKED, xgen-mptest)

The MP-C-14 smoke under-exercises leaf-authored content (audit §5, Clair J-351): each leaf's `send`
fires at `after_ms:40` **immediately after** its `join`
([`mp_r3_topology.rs:61-63`](../xgen-mptest/tests/mp_r3_topology.rs#L61)), so the leaf post **races its
own join** → lands nowhere reliably → **only a0's `p0` is genuine cross-node content under test.** A
green today does not prove leaf posts converge.

**Enrichment (LOCKED, ships with the fix's witness):** make each leaf's `send` depend on its `join`
having **landed** (gate the send on the join reply / sequence it after the join is confirmed) so
leaf-authored content is **genuine cross-node content under test.** `xgen-mptest` change (the MP-C-14
template `actor_batch`, [`mp_r3_topology.rs:39-71`](../xgen-mptest/tests/mp_r3_topology.rs#L39); the
`StarPlusMesh` generator path stays, [`sweep.rs:456-471`](../xgen-mptest/src/sweep.rs#L456)).

**The enriched oracle must assert all three (D-065 — the enrichment ADDS coverage, it must NOT mask the
gap):**
- **(a) creator post BEFORE the late joins** — `p0` (the MP-F14 headline gap; this assertion stays,
  the enrichment must keep `p0` authored before the leaves join, or the gap is papered over);
- **(b) leaf post AFTER its own join** — the under-tested half (genuine leaf-authored cross-node
  content);
- **(c) creator post AFTER all joins** — the steady-state control.

So a GREEN-on-rerun proves *every member sees every post regardless of join order, by the creator AND
by leaves* — not just "the happy path where the race didn't bite." **The `p0`-before-joins shape is
load-bearing: do not relax the timing into a no-race scenario** (that would hide MP-F14, not fix it).

---

## 7. MP-F14-D6 — convergence-safety, D-076, scope fence [LOCKED]

- **Convergence-safety / D-076:** the fix is a federation **delivery** trigger (re-push an
  already-resolved event to a legitimate peer). It changes *who receives*, not *resolution/ordering* →
  D-076 discharged-by-inheritance (the delta-compute reuses the proven canonical full-history path; the
  spine §5 proves it). Production `xgen-core`/`xgen-node` change → protocol-change discipline
  (RED-on-revert spine, F-3 intact).
- **DM untouched:** DMs keep `repopulate_dm_federation_nodes` (members∪invitees); the regular path is
  the **additive sibling** (the C5 posture). The MP-F1b DM spine + J-298 INV-EXP stay byte-unaffected.
- **NOT this arc:** MP-C-16/MP-F13 (home_node WS-URL vs pubkey-id; J-278/F1B-D5; M10+ carve-out — a
  *different* root). MP-A-08's transport-level reconnect-deadlock half (R3-D2, routed). The
  residents-multiplexer (R3-D1, routed). Any newly-surfaced bug at the R3 rerun **faces-and-routes** to
  its home but does **not** extend the {MP-F14} gate (the R2 discipline).

---

## 8. MP-F14-D7 — the conditional terminal (the MP-F9 falsification branch) [LOCKED]

The trace (§3) is **authoritative**. If exec-step-1 **falsifies the backfill premise** — e.g. the trace
shows the missing leaf IS in `federation_nodes[S]` at `p0`-push yet still misses `p0` (a **delivery**
bug: `try_send` drop, unregistered sender, an F-5 mis-fire) rather than a missing re-stream — then:
- the finding is **re-pinned** (a delivery defect, not a backfill gap), the fix **re-shapes** to the
  delivery path, and the design's §4 forks are superseded for that mechanism (MP-F9 precedent: the
  exec-step-1 observation re-locked the mechanism).
- **The gate disposition is unchanged:** MP-F14 stays a fix-in-round gate item, terminal = GREEN-on-rerun
  (the delivery fix is still R3-grade, still no later-milestone home). Only the *mechanism* follows the
  trace.
- Any **orthogonal** bug the trace surfaces (sibling to MP-F12) **faces-and-routes** to its own home;
  it does **not** extend the gate.

This is the honest-by-construction conditional: the design is authored now because the surfaces are
strong enough to scope the work, but the **mechanism lock waits on the observation** — the third
falsified-mechanism-guess risk this family has earned a bar for (MP-F9 ×1, the gap-2-settle-race route
caught mid-pin ×1).

---

## 9. MP-F14-D# ledger + DECISIONS posture

| # | Decision | Status |
|---|---|---|
| MP-F14-D1 | the certain gap = a member silently misses content; the invariant = the missed cooperative content reaches the member. (Original framing "send-time-only push, no outbound re-stream on federation_nodes growth" was the §4 hypothesis; the re-trace re-rooted it to causal-anchoring — see §0.1.) | **LOCKED** (re-rooted §0.1) |
| MP-F14-D2 | **RE-LOCKED (§0.1):** `get_dag_tips` cooperative-frontier — exclude infra/vantage-specific kinds (`state.federation_add`) from the frontier a cooperative event's `prev_events` anchor to (`batch.rs:145-161`). Forks A/B FALSIFIED by the re-trace. | **LOCKED** (re-lock §0.1) |
| MP-F14-D3 | exec-step-1 = box-gated re-trace (the 4-point diagnostic) — **DONE** (Fork A falsified); spine in-process RED-on-revert; witness box-gated | **LOCKED / DONE** |
| MP-F14-D4 | **RE-SHAPED (§0.1):** the spine = `get_dag_tips` infra-exclusion (fix-proof + MP-F4 no-regression), client-side RED-on-revert. The J-333 hole-safety lens NO LONGER applies (not an F-3 path). | **LOCKED** (re-shape §0.1) |
| MP-F14-D5 | the MP-C-14 leaf-content coverage enrichment (gate leaf sends on join; assert (a) pre-join creator / (b) post-join leaf / (c) post-join creator; keep `p0`-before-joins) | **LOCKED** |
| MP-F14-D6 | convergence-safety / D-076 discharged-by-inheritance; DM untouched; scope fence | **LOCKED** |
| MP-F14-D7 | conditional terminal — the trace is authoritative; **FIRED §0.1** (Fork A falsified → re-shaped to the `get_dag_tips` causal-anchoring root); gate disposition unchanged (fix-in-round, GREEN-on-rerun) | **LOCKED / FIRED** |

**All arc-local (D-069). No DECISIONS promotion in this arc.** Standing promotion candidates (Joe's
call, unchanged): the loop-to-green-with-a-bounded-gate round-close discipline (R1 J-322 / R2 J-344 /
R3) and pin-by-observation-before-routing (the MP-R2 bar).

---

## 10. Discipline + next step

- Surface-and-route (D-065/D-084); **pin-by-observation BEFORE locking the mechanism** (§3/§8). No code
  in the design beat; the fork locks after exec-step-1.
- Two-commit close: this design (Clair's seat) commits FIRST (Joe pushes), then Chat's doc-bridge (the
  J-NNN that flips CLAUDE.md PLAY + ROADMAP + MP_findings). Joe pushes; Chat never pushes.
- Honest boundaries recorded: the fork is conditional (the trace decides, §4.3); the re-stream is
  net-new behaviour wired to reused machinery (§2 / §4.4); the enrichment ADDS coverage and must keep
  the `p0`-before-joins gap visible (§6).

**Next step: `tasks/MP_F14_PREJOIN_BACKFILL_IMPL.md`** (the runbook) — after Joe's locks. The runbook
authors: **exec-step-1 = the box-gated re-trace** (§3) → the fork lock (MP-F14-D2, §4.3) → the
spine-first commit (the §5 hole-safety + backfill-on-growth RED-on-revert, in-process) → the outbound
re-stream fix (the trace-selected site) → the §6 coverage enrichment → the box-gated `mp_r3_topology`
witness (enriched) → the **R3 rerun** to all-green-except-MP-C-16 → MP-R3 close + the consolidated
R1+R2+R3 ledger (the standing HANDOFF deliverable).

**Entry point (Rule 0):** CLAUDE.md PLAY (J-351 RUN-#1/fix-phase head) → JOURNAL J-351 →
`tasks/HANDOFF_MP_R3.md` → `tasks/MP_F14_PREJOIN_BACKFILL_AUDIT.md` → this design →
`tasks/MP_R3_CAPSTONE_DESIGN.md` §5.1 (the C5/MP-F11 contrast) →
`docs/tests/MULTIPARTY_TEST_MATRIX.md` §6 (the RUN-#1 record).
