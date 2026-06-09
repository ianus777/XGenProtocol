# MP-F4 — DM invitee's room-join dropped by node-side membership resolution — D-071 PHASE-0 AUDIT

> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-09  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Discipline note

Phase-0 grounding only — **no code, no design lock**. This audit grounds the DM membership-resolution
path against live `main`, surfaces the forks with a recommendation, and runs the D-076 / D-077
checks. The design is authored only after Joe locks the forks (§10). MP-F4 is the next
production-crate fix-arc of the loop-to-green (MP-R1-D10), after MP-F2 (J-324) / MP-F3 (J-326) /
MP-F1a (J-328). Same discipline: ground-before-conclude, frozen-string / regression-net awareness,
build 0 + clippy clean (default + `--features harness-control`) at each step.

**Authoring note (three-agent).** Chat Claude opened this document as a structured scaffold (header,
section frame, §1 carried from `tasks/MP_findings.md` v1.4). **v1.1: Clair completed every
`[GROUND]` section against live `main`** and recorded one material divergence from the routed
framing — see §2.3 + §3 + the §10 verdict. Honest-over-fast (D-065): the grounding does **not**
silently confirm the routed mechanism; where the live code contradicts it, that is surfaced and it
changes the fork recommendation.

---

## 1. The finding (recap, `tasks/MP_findings.md` MP-F4 — Clair-grounded at C2, J-328)

- **Surfaced:** `MP-C-07-LOCAL` (single-node DM delivery witness), C2 (J-328). Severity **moderate**
  — blocks single-node 2-party DM **message convergence**; no state corruption.
- **Symptom:** the DM invitee lands as a **Space member but not a room member**, so his
  `message.text` is rejected at step-11 `NotARoomMember`. The `MP-C-07-LOCAL` witness is GREEN on
  *delivery* (all create-dm events + the invitee's space-join + room-join land in Node A's `.events`)
  but deliberately does **not** assert message convergence — because of this gap.
- **Mechanism (as routed):**
  - `state_key_for_event` keys a membership event on `membership:{space}:{sender}` — **room-agnostic**
    (`xgen-core/src/resolution/state_key.rs:48`). A space-join and a room-join by the same identity
    therefore collapse onto **one** membership key.
  - `get_invite_bootstrap` (`xgen-client/src/batch.rs:179`) re-returns the invite naming the invitee
    **even after he is already a member** → the invitee's space-join (b2) and room-join (b3) both
    anchor to `[invite_id]` → **concurrent siblings** on the one membership key → `derive_resolved`
    keeps one and drops the other → member-of-Space-not-room → step-11 `NotARoomMember`.
- **Contrast proof (DM-specific):** MP-C-01 (regular Space, same join pattern) **PASSES**.
- **DM-model anchor:** `xgen-core/src/space/state.rs::from_dm_space_create` (state.rs:342).
- **Route:** node-side DM-membership fix-arc (this Phase-0). Candidate directions: room-scope the
  membership `state_key`, **or** gate `get_invite_bootstrap` to non-members.
- **F1b cross-link:** this resolution surface overlaps the DM-membership code (iii)/MP-F1b will
  touch. Flag for MP-F1b Phase-0 — weigh together; **not a merge**.

---

## 2. Grounding — the DM membership-resolution path (live `main`)

### 2.1 The state-key collision — CONFIRMED

`state_key_for_event` (`xgen-core/src/resolution/state_key.rs:44`) is the **single producer** of every
conflict key. The membership arms today:

| Event kind | Arm | Key tuple | Room dimension? |
|---|---|---|---|
| `MembershipJoin`, `MembershipLeave` | `state_key.rs:48` | `("membership", "{space}:{sender}")` | **none — room-agnostic** |
| `MembershipInvite/Kick/Ban/NodeEject/NodeUnban` | `state_key.rs:57` | `("membership", "{space}:{target_identity}")` | none — room-agnostic |

No membership arm carries a room dimension. (Room-scoped keys exist elsewhere — `StateRoomUpdate`→
`room_id`, `MlsGroupInit`→`room_id`, `MlsCommit`→`{room}:{epoch}` — but **not** for membership.)

The collision, with live key values: bob's **space-join** (`MembershipJoin`, `sender=bob`,
`room_id=""`) → `membership:{space}:bob`. bob's **room-join** (`MembershipJoin`, `sender=bob`,
`room_id=DM-room`) → **also** `membership:{space}:bob`. **Byte-identical key string** — the room_id is
not read by the `Join`/`Leave` arm. Collision confirmed.

**Why room-agnostic exists:** the comment at `state_key.rs:47` ("Sender IS the affected identity for
join/leave") predates room-level membership. `apply_join` later grew a room-level branch
(`state.rs:986`, `room_id` non-empty → `room.members.insert`), but `state_key_for_event` was never
updated to distinguish room-membership from space-membership. **The conflation is a latent gap, not a
DM-specific quirk** — see §2.4.

### 2.2 The concurrent-sibling drop in `derive_resolved` — CONFIRMED (precise mechanism)

`derive_resolved` (`xgen-core/src/resolution/derive.rs:76`) is the resolving core. The drop is **not**
a crude tie-break; it is the §3.9.1 frontier logic:

1. `find_conflicts` (`conflict.rs:44`) groups events by `StateKey`. For key `membership:{space}:bob`
   the group is `{ invite(bob), b2 space-join, b3 room-join }`.
2. `frontier_of` (`derive.rs:179`) restricts the group to its **causal frontier** — events that are
   not a transitive ancestor of another group member. A frontier of size 1 = plain causal succession
   (no conflict, all fold normally). A frontier of size **≥ 2** = mutually-concurrent = a genuine
   conflict set.
3. A genuine conflict set goes to `resolve()` (the seven-layer algorithm); the **loser** is added to
   the `losers` skip-set and is **excluded from the final fold** (`fold_skipping`, `derive.rs:210`).

So the outcome turns entirely on **whether b2 and b3 are causal or concurrent**:

- **Causal** (b3's `prev_events` transitively include b2): frontier of `{invite, b2, b3}` = `{b3}`
  (size 1) → **no conflict** → `fold_skipping` applies invite, b2, **and** b3 in causal order → bob is
  Space member (b2) **and** room member (b3). ✓
- **Concurrent** (b2.prev=`[invite]`, b3.prev=`[invite]` — siblings): frontier = `{b2, b3}` (size 2,
  invite is ancestor of both) → **conflict** → `resolve()` picks one winner, the other is dropped. ✗

Both b2 and b3 are `MembershipJoin` (same type) → **Layer 1 (type-priority) abstains** (no
join-vs-join rule); empty `identity_home_nodes` → Layers 3/5a/5b abstain → **Layer 5c
(lexicographic `event_id`) decides**. Which membership fact survives is therefore *hash-arbitrary*:
- space-join wins → bob Space-member, **not** room-member → `message.text` step-11 `NotARoomMember`
  (the witnessed symptom).
- room-join wins → `apply_join` room-level **guards on space membership** (`state.rs:988`,
  `if !self.members.contains_key(joiner) → Err(NotASpaceMember)`); b2 was skipped → bob is not a
  Space member → the room-join `Err`s (swallowed) → bob in **neither**.

**Key distinction (decides the fix layer):** the room-join b3 **is admitted to the DAG** —
`validate_event` accepts it, it is stored, it appears in `.events`. It is the **resolved view**
(`derive_resolved`) that excludes it. This is *resolution* dropping a fact, **not** *admission*
rejecting it. So the root-cause fix is in **keying** (the conflict domain), not admission.

### 2.3 The bootstrap re-issue — **DIVERGENCE FROM THE ROUTED FRAMING (honest finding)**

The routed mechanism (§1) says the node "re-returns the invite naming the invitee **even after he is
already a member**," and contrasts MP-C-01 where "the Node refuses the bootstrap once the requester is
a member." **Grounding the static node path does not reproduce this premise:**

- `collect_invite_bootstrap` (`xgen-node/src/fanout.rs:541`) authorizes **solely** on
  `space.pending_invites.get(requester_id)` (fanout.rs:550) — `ok_or(REFUSED)` (wire 1011) if absent.
- `apply_join` space-level (`state.rs:1006`) does `self.pending_invites.remove(joiner)` on **every**
  space-join — DM or regular, no exemption.
- The ingest path (`runtime.rs:672`) applies b2 via the **fast incremental path**: `conflicts_in_log(b2)`
  is `false` (the invite is b2's transitive ancestor, b2.prev=`[invite]` — `derive.rs:260` is
  ancestry-aware), so `apply_event(b2)` runs and mutates the **live** `rt.spaces` snapshot.

⇒ After b2 is confirmed (MP-F1a `EventAccepted` sequences b3 after b2), bob is **gone from live
`pending_invites`**, so a *subsequent* `collect_invite_bootstrap` for b3 should **refuse (1011)** in
the DM exactly as in the regular Space. The static node path gates the bootstrap on membership
**identically** for both. So "the bootstrap re-issues to a member" is **not** supported by the
node code as written.

**What this means for the fix.** The proximate source of b3's concurrency (anchoring to `[invite]`
rather than to b2) must therefore be one of:
- **(i)** a live-state **staleness window** — b3's bootstrap reads a snapshot taken before b2's apply
  propagated (less likely given MP-F1a send-confirm sequences b2→b3); or
- **(ii)** the **`get_dag_tips` fallback** (`ops.rs:922`): when the bootstrap refuses (1011) →
  `Ok(None)` → `ops::join` falls back to `get_dag_tips`; if that returns a set that does **not**
  include b2 (DM member-visibility / `collect_sync_history` scoping / timing), `ops::join` finally
  anchors to `vec![args.space]` (the Space root, `ops.rs:924`) → b3 concurrent with b2.

I **cannot disambiguate (i) vs (ii) statically** — it needs a ~30-min runtime trace (capture b3's
`prev_events` and the live `pending_invites` at b3's bootstrap). **This ambiguity is exactly why fix
A2 is under-determined (§4):** A2 ("gate `get_invite_bootstrap` to non-members") targets a re-issue
the node already refuses; if the real concurrency comes from (ii), A2 changes nothing. **Fix A1
(keying) closes the finding regardless of which proximate cause holds.**

> Recorded honestly (D-065): Clair grounded MP-F4 empirically at C2 and may have *observed* the
> re-issue in a trace; the static read here cannot. The two are reconciled by pinning (i) vs (ii) in
> the design phase — but that pin is **not** on the critical path for the recommended fix (A1).

### 2.4 Why MP-C-01 survives — and why it survives *by luck*, not by guarantee

Under the room-agnostic key, MP-C-01's room-join (c3) survives **only because** it ends up causally
*after* c2 (its bootstrap refuses and `get_dag_tips` returns a tip that includes c2 — carol is a full
Space member, so the member-gated sync serves her c2). It is the **same machinery** as the DM; the DM
differs only in that c3/b3's fallback anchor lands concurrent instead of causal.

**Corollary (load-bearing for the fork choice):** the room-agnostic membership key is a **latent
collision that bites any concurrent space-join + room-join of one identity** — a federation replay
reorder, a future client change to the join flow, or any path that makes the two joins siblings would
re-trigger the identical drop in a *regular* Space. The DM is simply the scenario that triggers it
today. This is why the root-cause fix (A1, correct the conflict domain) is honest-longer-work over
the proximate patch (A2, prevent the DM's specific concurrency) — A2 leaves the collision live.

---

## 3. The clean decision point (D-067 — single no-drift location)

The true root is **§2.1 + §2.2**: the membership `state_key` conflates two distinct facts — *space
membership* and *room membership* — into one conflict domain, so concurrent assertions of the two
facts (which are not actually in conflict — a member can be in a space and a room simultaneously) are
forced to elect a single winner and drop the other.

The **minimal correct fix** is to give space-level and room-level membership **distinct conflict
domains** — one location, `state_key_for_event`'s `Join`/`Leave` arm (`state_key.rs:48`). Every other
candidate (gate the bootstrap, force the client anchor) patches the *concurrency* at the proximate
layer while leaving the conflated domain live for the next caller — a D-077 forward-coherence debt.

`state_key_for_event` is the single producer (§2.1) and all 5 production readers consume the key
**opaquely** (§7) — so the no-drift location is unambiguous.

---

## 4. Forks

### F4-A — fix location / mechanism  *(recommend: **A1**)*

- **A1 — room-scope the membership `state_key` for room-level join/leave** (`state_key.rs:48`).
  Likely shape (design-phase, not locked): for `MembershipJoin`/`MembershipLeave`, branch on
  `event.room_id` — empty ⇒ `("membership", "{space}:{sender}")` (unchanged, space-level); non-empty
  ⇒ a room-scoped key (e.g. `("membership", "{space}:{room}:{sender}")` or a distinct category).
  `Invite/Kick/Ban/NodeEject/NodeUnban` stay room-agnostic (keyed on `target_identity`).
  - **Blast radius (grounded):** `state_key_for_event` is the single producer; the 5 production
    readers (`runtime.rs:673`, `conflict.rs` ×3, `derive.rs` ×2 — and notably
    `xgen-client/src/ai_service.rs:544`, the R2-F01 client gate) consume the `StateKey` **opaquely**
    (`==` / HashMap key). **No reader hardcodes the tuple shape; no compiler-caught call sites.** The
    change is purely behavioral — `StateKey` is **never serialized** (§7), so no wire/persistence
    migration.
  - **The one real design-phase obligation (flag, don't resolve here): cross-scope conflict.** A
    space-level `ban`/`kick`/`leave`/`eject` (room-agnostic key) must still dominate a room-level join
    of the same identity (now a *different* key ⇒ no longer a direct conflict). Static reading
    suggests this stays convergent for free: `apply_join` room-level **guards on space membership**
    (`state.rs:988`) and `apply_ban`/`apply_kick` **cascade-remove from all rooms** (`state.rs:1080/1105`)
    — so a banned identity's room-join `Err`s regardless of fold order. The design phase **must prove
    this across all orderings** (ban/kick/leave/eject × room-join); it is the D-076 crux (§5).
- **A2 — gate `get_invite_bootstrap` to non-members** (`batch.rs:179` / `fanout.rs:541`).
  **Under-determined — not recommended.** Per §2.3 the node *already* refuses the bootstrap once a
  member (`pending_invites` cleared on space-join). A2 targets a re-issue the static path does not
  reproduce; if the real concurrency is the `get_dag_tips` fallback (§2.3-ii) A2 is a no-op. It is
  also narrower in the wrong way — it leaves the §2.1 collision live (D-077 forward debt, §6).
- **A3 — client-side anchor fix** (force b3 to chain off b2/tip). Same class as A2: patches the
  concurrency, leaves the conflation. Fragile (timing/visibility-dependent). Not recommended.

**Recommendation: A1.** It is the single no-drift location, robust to the §2.3 proximate-cause
ambiguity, and closes the latent collision (§2.4) rather than the DM symptom only. A2/A3's appeal is
"narrower," but narrower here means "leaves the root live" — the opposite of the MP-F3 lesson.

### F4-B — the witness flip  *(recommend: confirm, with one added check)*

`MP-C-07-LOCAL` is GREEN on delivery-only today. After A1 it must assert **2-party message
convergence**: bob resolves as a room member, his `b4 message.text` is accepted and lands, and **both**
message `event_id`s (alice `a3`, bob `b4`) appear in Node A's cooperative `.events` set. **Sensitivity
witness:** revert A1's key change → b2/b3 become a size-2 frontier again → room-join dropped → b4
step-11 `NotARoomMember` → b4 absent → assertion **genuinely RED**.

> **Added check (grounded):** the witness must assert **both** messages, which means confirming
> alice's `a3` also appears post-MP-F1a. MP-F1a's UPDATE (J-328) reframed the old "DM message not
> transcript-observable" symptom (MP-F1 facet-2 §52–60) as a consequence of the RST-dropped chain,
> now fixed — but the current `MP-C-07-LOCAL` asserts only the create-dm chain + auto-room landing,
> **not** any message. The flip is the first assertion that a DM *message* converges; design phase
> confirms `a3` lands as part of it.

### F4-C — F1b cross-link  *(recommend: flag, do NOT merge)*

Grounded as **different code**: MP-F4 touches `xgen-core/src/resolution/state_key.rs` (the membership
*conflict domain*). MP-F1b / (iii) touches `apply_federation_add` / `federation_nodes` population at
*membership-apply* (`state.rs`). Overlap is conceptual ("DM membership"), not line-level. **Cross-link
for F1b Phase-0:** when F1b populates a DM's `federation_nodes` from "its members," it means **Space**
members; the room-scoped membership facts A1 introduces are room-level and orthogonal to
`federation_nodes`. F1b should ground (iii)/gate-B against the **post-A1** (correct) single-node DM
membership model — which is the J-328-locked reason F4 runs first. Do not pre-build F1b machinery in
F4.

---

## 5. D-076 convergence check

A1 **directly touches the resolution surface** (the conflict domain) — load-bearing, unlike MP-F2/F3
(delivery-only, D-076-trivial).

- **Invariant the design must hold:** *a node's resolved membership is byte-identical for every Space
  and every existing membership conflict, EXCEPT that a room-level join/leave no longer collides with
  a space-level join/leave of the same identity.*
- **A1 proof obligations:**
  1. Room-scoping changes resolution **only** for the space-join↔room-join collision; every existing
     membership conflict (ban>join, owner-invite>admin-invite, lexicographic backstop — the
     `derive.rs` convergence tests) is byte-identical (they exercise *space-level* membership only,
     `room=""`).
  2. **Cross-scope conflict (the crux, §4-A1):** a space-level removal (ban/kick/leave/eject) still
     dominates a room-level join of the same identity under **all** arrival orders. Candidate
     argument: the `apply_join` space-membership guard (`state.rs:988`) + the ban/kick room cascade
     (`state.rs:1080/1105`) make it convergent without a shared key — but this must be *proven*, not
     assumed, since A1 deliberately removes the shared key that previously forced the conflict.
- **Regression net:** `state_key.rs` unit tests, `derive.rs` convergence/permutation tests
  (`assert_converges`), the 285 binary-convergence integration tests, the M8 net, `MP-C-07-LOCAL`
  (flips), MP-C-01 (stays green).

---

## 6. Backward-coherence audit (D-077)

- **Backward:** no current passing scenario depends on the room-agnostic key **merging** space-join +
  room-join onto one fact — the collision *drops* one fact, it never *merged* them, and there is no
  test asserting "a room-join and a space-join resolve to one membership fact." MP-C-01 (the prime
  contrast) stays GREEN under A1: c2 (`room=""`) and c3 (`room=R`) become **distinct** keys, both fold
  — and MP-C-01 becomes *robust* to concurrency instead of luck-dependent (§2.4). The existing
  `state_key.rs` membership tests (`membership_join_state_key_uses_sender`, `..._ban_..._uses_target`,
  `join_and_ban_on_same_target_share_state_key`) stay valid — they are all space-level (`room=""`),
  untouched by the room-non-empty branch.
- **Forward:** A2 (the narrower bootstrap gate) would leave the §2.1 collision live → a future re-join
  / role change / federation reorder re-triggers it (§2.4). A1 closes it. **Forward-coherence favors
  A1** — this is the D-065 honest-longer-work call.

---

## 7. Blast radius + regression nets

- **Code surface (A1):** one function, `state_key_for_event`'s `Join`/`Leave` arm (`state_key.rs:48`)
  — a string-format change gated on `room_id`. **No compiler-caught call sites** (all 5 production
  readers opaque, §4-A1). The R2-F01 client gate (`ai_service.rs:544`) consumes the **same** function,
  so client and node resolution stay consistent — and the client's DM convergence benefits identically.
- **Wire / persistence:** **none.** `StateKey` has no serde / serialize / persist usage (grounded —
  zero hits); it is recomputed from the log on every derive and used only as an in-memory grouping
  key. A1 is behavioral, not a wire or reason-string change. *(Design phase confirms no persistence
  reader exists — expected clean.)*
- **Regression nets:** xgen-core lib, xgen-client lib, integration convergence (285), M8 net,
  `MP-C-07-LOCAL` witness (flips), MP-C-01 contrast (stays green). Build 0 + clippy clean (default +
  `--features harness-control`).

---

## 8. Test / close plan

- **Unit (`derive.rs` / `state_key.rs`):** (a) a space-join and a room-join by the same identity in
  the same Space produce **distinct** state keys (the A1 fix, mirrors the MLS-commit per-epoch key
  tests); (b) a `derive_resolved` permutation harness where the invitee's space-join + room-join are
  concurrent siblings now resolves to bob being **both** Space member and room member under every
  arrival order; reverting A1 drops the room-membership (RED).
- **Cross-scope (D-076 crux):** a permutation harness with a concurrent space-level ban + room-level
  join of one identity converges to "banned, in no room" under every order (proves A1 did not open a
  ban-evasion).
- **`MP-C-07-LOCAL`:** flip delivery-only → 2-party message convergence (both `a3` + `b4` in Node A's
  set); genuinely RED on revert (§F4-B).
- **MP-C-01 regression:** stays GREEN.
- **Federated `MP-C-07` (`mp_r1_c4`):** stays KNOWN-FAIL — F4 closes *single-node* convergence, not
  cross-node (that is MP-F1b). Confirm the annotation does not falsely flip.

---

## 9. Out of scope / surfaced-not-chased (D-065)

- **MP-F1b** (cross-node DM convergence) — F4 does not touch it; federated MP-C-07 stays known-FAIL.
- **The §2.3 proximate-cause trace** (staleness window vs `get_dag_tips` fallback) — informational
  only; A1 fixes the finding regardless, so pinning it is **not** on the critical path. Worth a
  ~30-min trace in the design phase to retire the routed "bootstrap re-issue" framing cleanly, but it
  does not gate A1.
- **The constructor empty-`prev_events` auto-invite latent bug** (`from_dm_space_create`, flagged
  J-219 / ops.rs:578) — out of scope; `create_dm_space` already rebuilds the invite tip-chained.
- **Any other membership-key collision** surfaced while grounding §2 but not on the DM
  message-convergence path — none found; the only conflated pair is space-join↔room-join (and its
  leave sibling).

---

## 10. Recommendation summary + Joe-lock asks

| Fork | Recommendation | Joe-lock? |
|---|---|---|
| **F4-A** fix location / mechanism | **A1 — room-scope the membership `state_key` for room-level join/leave** (root-cause; robust to §2.3 ambiguity; closes the latent collision). A2/A3 rejected: under-determined / leave the conflation live. | **lock** |
| **F4-B** witness flip (`MP-C-07-LOCAL` → 2-party message convergence, both `a3`+`b4`) | confirm (+ confirm `a3` lands post-MP-F1a) | confirm |
| **F4-C** F1b cross-link (different code; flag, not merge) | confirm | confirm |
| **D-076** convergence (hard for A1) | discharge via the **cross-scope-conflict proof** (§5.2) — the design phase's main work; static reading says the `apply_join` guard + ban/kick cascade keep it convergent, must be proven | confirm |
| **D-077** backward + forward coherence | backward clean (MP-C-01 stays green, becomes robust); forward favors A1 | confirm |

**Verdict: GAP CONFIRMED — severity moderate** (blocks single-node 2-party DM message convergence; no
state corruption). Root cause = the membership `state_key` conflating space-membership and
room-membership into one conflict domain (`state_key.rs:48`), so a concurrent space-join + room-join
of one identity is forced to elect one winner and drop the other in `derive_resolved`
(`derive.rs:179` frontier logic). It is *resolution* dropping an *admitted* fact, so the fix is in
keying.

**Honest finding (D-065, the §2.3 divergence):** the routed "bootstrap re-issues the invite to an
already-member invitee" premise — which fix **A2** targets — is **not reproducible from the static
node path**: `collect_invite_bootstrap` already authorizes on `pending_invites`, which the space-join
clears. The real proximate concurrency source (staleness window vs `get_dag_tips` fallback) needs a
runtime trace to pin. This is *why* A2 is under-determined and **A1 is recommended** — A1 closes the
finding whichever proximate cause holds, and it also closes the latent collision A2 would leave live.

**Next:** Joe locks F4-A (A1 is the real decision) → author `tasks/MP_F4_DM_MEMBERSHIP_DESIGN.md`
(the cross-scope-conflict proof is the design's spine) → runbook → implement → close (`MP-C-07-LOCAL`
→ 2-party message-convergence PASS; MP-C-01 stays GREEN; federated MP-C-07 stays known-FAIL). Holding
for Joe-lock per Phase-0 discipline.

---

*Per D-065 + D-067 + D-069 + D-071 + D-076 + D-077.*
