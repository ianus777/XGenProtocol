# M-SPACE-ADMISSION Leg D Phase-0 — the admission gate: the first thing that ever reads `admission`
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-23  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — WHAT THIS LEG IS

**Legs B and C built the field and the mutation event. NOTHING HAS EVER READ THE VALUE.** `state.rs:283` says so in its own doc comment: ***"Nothing reads this field until Leg D."*** ⇒ **Leg D is the leg where `admission` starts meaning something.**

📌 **Written under `D-155`:** every §§ routed to Joe below is stated as a situation, not a site. The mechanics live in this document; the questions do not.

---

## §2 — 🛑 THE HOLE, MEASURED AT `1623eb6`

**`MembershipJoin` is in `skip_membership` (`exchange.rs:670-680`)** ⇒ **step 11 (`:681`) and step 13 (`:750`) are BOTH skipped for a join.** `validate_event` never adjudicates one.

**The only join gate that exists is at dispatch time — `runtime.rs:1580-1613`** — and its shape is the finding:

```
1586:  if let Some(pi) = space.pending_invites.get(&event.sender) {
```

🔑 ***THE GATE IS INSIDE THE `if let`.*** It adjudicates **the expiry of an invite that exists**. **No pending invite ⇒ the block does not run ⇒ the join falls through and is APPLIED.** ⇒ **anyone may join any Space today, and that is not a gap in enforcement — there is no enforcement to have a gap in.**

⚠️ **The comment at `:1563-1565` already says this out loud** — *"an open join (no pending invite at all) is untouched"* — **so the hole is documented, deliberate, and has simply been waiting for its admission value.**

---

## §3 — THE GATE'S SHAPE

**Same block, same `origin == LocallySubmitted` condition, same fail-closed posture as `3044`.** The new check runs **before** the expiry check and answers a different question:

| | question | today |
|---|---|---|
| **`3044`** | *your invite — is it still valid?* | ✅ built |
| **LEG D** | *do you need one at all?* | ❌ nothing |

🔒 **Federation posture INHERITED, not re-argued:** on `ReceivedViaFederation` the gate is **SKIPPED, not rejected** — `:1567-1579` — a peer trusts the home node's admission decision. **Re-adjudicating would re-create the aged-Space divergence that comment exists to record.**

📌 **Room joins are out of scope by the existing condition** (`event.room_id.as_str().is_empty()`): a room join is gated by Space membership, not by admission.

---

## §4 — 🛑 `F-3` — AND MEASURING IT FOUND THE DOC COMMENT IS FALSE

`state.rs:348-351`:
```
let admission = content["admission"].as_str().map(str::to_string)
    .unwrap_or_else(|| DEFAULT_ADMISSION.to_string());
```

**`:344-347` claims the constructor keeps *absent* and *present-but-unrecognised* apart, and that collapsing them is exactly what it refuses to do.** ⚠️ **`as_str()` RETURNS `None` FOR ANY PRESENT NON-STRING** — a number, a bool, an object. ⇒ ***the collapse the comment forbids is performed by the line the comment sits on, for every non-string value.***

🛑 **AND IT COLLAPSES IN THE PERMISSIVE DIRECTION.** `DEFAULT_ADMISSION` = `ADMISSION_OPEN` (`wire.rs:776`, `:748`) ⇒ **`{"admission": 5}` creates an OPEN Space.** Once §3's gate exists, **a malformed value is the difference between a closed Space and an open one.**

🔒 **RULED (a′), Joe:** a named three-state parse — **Absent / Valid / Malformed** — malformed **stored as raw JSON text**, **only Absent taking the default**.
🔒 **TAKEN BY CHAT:** **64-byte cap, char-boundary truncation.** 🔑 **The load-bearing constraint is not the number — every node must truncate IDENTICALLY or the stored value diverges.** Convergence measured safe at J-759 (serde_json 1.0.149, no indexmap edge ⇒ `BTreeMap`, deterministic).

⇒ **the gate treats Malformed as `invite`** — fail-closed, matching `3044`'s `unwrap_or(true)` at `:1591`.

---

## §5 — 🔒 `D-154` LANDS HERE: `state.rs:1112`

```
1112:  if self.members.contains_key(joiner) { return Err(AlreadyMember) }
1115:  if self.banned.contains(joiner) { return Err(Banned) }
```

**A direct `contains_key` — `E-0` classifies it under `D-3`, so NO accessor ruling reaches it.** Under `(g)` it refuses the rejoin `Q-2`(a) promised, and **`D-154`②③ make `:1115` unreachable for retained banned members** ⇒ ***the ban check dead for exactly the people it exists for.***

🛑 **SUPERSEDED — `D-3` WAS UNBUILDABLE AND THIS §§ SPECIFIED IT ANYWAY (`D-131`, 2026-08-23, J-763). THE CLAIM BELOW IS FALSE AND IS KEPT, NOT DELETED.**

⇒ ~~**`:1112` gates on `left_at.is_none()` in this leg.**~~ 🛑 **`left_at` HAS ZERO OCCURRENCES IN ANY `.rs` FILE** — it exists only in design documents. **`SpaceMember` (`state.rs:86-95`) has four fields — `identity_id`, `role`, `joined_at`, `invited_by` — and NO departure marker.** Creating it **is** `(g)`, which **§7 item 1 of this very document forbids this leg from touching** ⇒ ***§5 specified an edit in terms of a field §7 forbids creating, and the contradiction sat two screens apart in a document Joe locked on Chat's recommendation.***

🔑 **THE DECISIVE EVIDENCE IS NOT THE MISSING FIELD — IT IS THAT `V-3c` CANNOT BE RUN.** With no retained member there is no rejoin to refuse and no retained-banned state ⇒ ***the control `J-762` calls "the one that matters" describes a state the code cannot reach.*** **Written anyway, `D-3` would have been an unfed branch with no possible control — in the leg whose entire point is that a gate whose removal leaves the suite green was never tested.** 📌 **Found by Clair, who reported it and did NOT improvise a substitute (Rule 6).**

✅ **`D-154`'s `:1112` clause MOVES TO LEG E**, where `left_at` is created and `V-3c` becomes reachable. **The ruling is untouched; only its leg changes.** 🛑 **It is a `SpaceState` applier edit, not a Node gate** — a different file and a different failure mode from §3, and the runbook must not let them share a test.

---

## §6 — 🔒 RULED (A) — JOE, 2026-08-22

**Someone tries to join a Space that requires an invite, and they do not have one. What are they told?**

**(A)** — **"You need an invite to join this Space."** They learn the Space exists and that it is closed.
**(B)** — **"You cannot join this Space."** No reason given.
**(C)** — **The same answer they would get if the Space did not exist.** They learn nothing at all.

🎯 **Chat recommends (A).**

**Why:** the person is almost always someone who was given a stale link or mistyped a name, and (A) tells them the one useful thing — *ask someone for an invite*. (B) leaves them guessing. **(C) is the only one that resists someone probing for which Spaces exist — but it costs every honest user a comprehensible answer, and a prober learns the Space exists from a dozen other places anyway.**

⚠️ **Honest caveat:** (A) does confirm the Space is real to anyone who asks. **If you want Space existence to be private, that is a much larger question than this leg** and it should be its own milestone rather than smuggled in through an error string.

🔒 **RULED (A) — Joe, 2026-08-22.** The refusal names the reason: **an invite is required.**

🔑 **AND THE CODE IS NOT THE NEXT FREE NUMBER — IT IS ALREADY RESERVED FOR THIS.** Leg C added a line to `ch3` §3.6.10.10 recording that **`3047` and `3048` are reserved and not yet live** (ROADMAP row, J-760). ⇒ 🔒 **`3047 admission_required`, TAKEN BY CHAT** — first of the two, in the `3040s` membership-authority sub-band beside `3044`/`3045`, whose family it belongs to. **Reversible on one word.**

⚠️ **§6's caveat stands and is NOT discharged by this ruling:** (A) confirms a Space is real to anyone who asks. **Space-existence privacy is a larger question than this leg and must not be smuggled in through an error string.**

---

## §7 — WHAT LEG D DOES NOT DO

1. **It does not touch `(g)` / `left_at` itself** — that is Leg E. §5 is the *gate* edit only, written so Leg E has a correct site to build on.
2. **It does not re-adjudicate on federation** (§3).
3. **It does not close `E-0`'s six carried findings** — `C-3` mechanical · `C-4` · `C-5` · `C-6` · `C-7` · `F-E`, all Chat's, all Leg E's.
4. **It does not make Space existence private** (§6's caveat).
5. 🛑 **AND IT WAS SILENTLY MISSING A DoD ITEM ANOTHER LEG HAD ALREADY ASSIGNED IT.** `RUNBOOK_SPACE_ADMISSION_LEG_A_BIS.md:203` states ***"after Leg D the DM test is edited into its opposite"***, and `ROADMAP:396` records **"LEG D INHERITS AN INVERSION DoD ITEM … a GREEN run of the un-edited DM test is a FAILURE OF THE GATE, not a pass"**. **Grep over this document and the runbook for `A-bis` / `witness` / `invert` returned ZERO.** ⇒ ***this Phase-0 was written from the code and from `E-0`, and never swept the backlog for work other legs had already assigned to it.*** 🔑 **Same species as `C-8`, one layer up: a register that exists, is authoritative, and is not consulted at the moment of allocation.** ✅ Clair hit it as a red suite, executed the inversion from the A-bis instruction, kept the companion untouched, and **flagged that the decision to act at all was hers.** ✅ **Chat's `V-3a` re-drive then proved the inversion is GATE-DEPENDENT — it goes red when the gate is disarmed** ⇒ *the choice to act was correct, and now measured rather than argued.*

---

## §7b — 🛑 `C-8` — THE WIRE-CODE REGISTRY IS ALREADY WRONG, FOUND WHILE ALLOCATING `3047`

**`3046 event_timestamp_out_of_bounds` is LIVE in code** (`exchange.rs:155`, mapped in `to_wire_code`) **and is ABSENT from `ch3` §3.6.10.10's registry** — the table at `:2185-2193` runs `3040`→`3045`, then jumps to `3049`.

⚠️ **THAT TABLE IS THE INSTRUMENT THAT MAKES *"`3047` is reserved"* CHECKABLE.** A registry missing a live code means ***the next person to allocate reads the table, sees a gap at `3046`, and takes a number already in use*** — and the collision surfaces as **two unrelated refusals sharing one wire code, in production, on somebody else's machine.**

📌 **`C-8` is Chat's.** **Leg D already touches §3.6.10.10 to add its own row (§8), so the missing `3046` row rides that edit** — a registry correction at the moment the registry is open, not a separate errand. 🛑 **It is a RECORD fix only: no code changes, no behaviour changes, and `3046`'s meaning is not re-litigated.**

---

## §8 — DoD

- [ ] The gate at `runtime.rs:1580`'s block, **before** the expiry check, `LocallySubmitted` only, Space-level only
- [ ] `F-3`'s three-state parse at `state.rs:348` + the 64-byte cap, **with the false doc comment at `:344-347` corrected, not deleted** (`D-131`)
- [~] ~~`state.rs:1112` gates on `left_at.is_none()`~~ 🛑 **UNBUILDABLE — MOVED TO LEG E.** See §5's annotation. **Recorded as broken rather than ticked (`D-065`).**
- [x] §6 ruled **(A)**; **`3047 admission_required`** minted and recorded at the site
- [ ] `C-8` — the missing **`3046`** row added to `ch3` §3.6.10.10 in the same edit that adds `3047`'s
- [ ] **A NEGATIVE CONTROL PER GATE** — each must be shown to FAIL when its own arm is removed (`N-194`); a test that passes with the gate deleted is not a test
- [ ] **The federation-skip proven, not asserted** — a `ReceivedViaFederation` join into an `invite` Space is APPLIED
- [ ] Floors re-measured from HEAD, cargo detached with `XGEN_EXIT_SENTINEL=`
- [ ] Records: JOURNAL + `CLAUDE.md` + ROADMAP + `Status: COMPLETED`, one `D-074` commit

📌 **"Commit pushed" is not a DoD item.**