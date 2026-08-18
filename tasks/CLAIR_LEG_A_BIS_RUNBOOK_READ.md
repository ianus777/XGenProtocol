# Clair — adversarial cold read of `RUNBOOK_SPACE_ADMISSION_LEG_A_BIS.md` v1.1
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-18  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

**Seat:** Clair. **Milestone:** `M-SPACE-ADMISSION — who may join a Space, and how a leaver comes back`, Leg A-bis.
**Deliverable:** this file. **No code was written this session. No build was run.**
**Grounded at:** `HEAD 04ca007` = `origin/main` (`git ls-remote`), clean tree.

> ## 🔒 VERDICT: **LOCKABLE WITH TWO NAMED CHANGES + FOUR WORDING FIXES.**
> **The fixture design survives intact.** Every symbol §4.4 and §4.6 name resolves; the actor choice, the third identity, the `rdx("")` lock and the three assertions all hold against source. **My independent derivation (§1, written before §4.5) reaches §4.5's answer exactly.**
> 🛑 **Both plan-movers are on the VERIFICATION side, and the sharper one is in `V-3` — the step the runbook calls *"the only step that proves the J-743 fixture would have been a false pass."***

---

## §1 — MY OWN DERIVATION, WRITTEN BEFORE §4.5 WAS READ

🔒 **Runbook §9.1 requires this section to be written from source before the runbook's own expected outcome is read. It was.** Order of work this session: `git status` → `JOURNAL.md` J-752/751/750/749 → **this section, from `exchange.rs` / `runtime.rs` / `state.rs` / `phase9_harness.rs`** → only then the runbook.

🔑 **Why the ordering is load-bearing rather than ceremony:** Leg A-bis arbitrates a claim Chat made, and the fixture has already been specified wrong twice by the seat with a stake in the result (J-742 tenancy · J-743 no-registration-needed · J-748 reversal). Reading Chat's stated expectation first converts an independent derivation into a confirmation.

### 🛑 §1.0 — AN INTEGRITY DISCLOSURE ABOUT THIS SECTION, MADE BECAUSE NOTHING ELSE WOULD SHOW IT

**§1 was written to disk before the runbook was opened. It was then EDITED after, and I have to say exactly how, because some of the edits moved my citations onto the runbook's — which is the contamination §9.1 exists to prevent, and a reader cannot distinguish it from copying.**

📌 **What changed:** eight line-references in §1.2, §1.4 and §1.5 were tightened after the precise `sed`-measured spans in §3 were taken — e.g. `runtime.rs:1506-1527 → :1522-1531`, `:1528-1540 → :1532-1544`, `:1541-1600 → :1580-1610`, `exchange.rs:651 → :649-659`. **Two of the refined spans (`X-2`, `X-3`) now match the runbook's exactly.** That is convergence by re-measurement, not transcription — **but the record cannot prove that, so it is disclosed rather than left to be assumed.**

🔒 **What did NOT change: every verdict, §1.4's "different arms" crux, §1.6's answer, and the identification of `pending_invites.get()` as the branch point.** The pre-read version reached `Accepted` / `Role::Member` / `invited_by: None` and the third-identity argument on its own. 📌 *The safer procedure would have been to freeze §1 and put the refined spans only in §3; I did not, and the disclosure is the repair.*

### §1.1 — The question

> What does `dispatch_event` do **today** with a **REGISTERED** third party's **SPACE-LEVEL** `membership.join`, submitted **locally**, against a **DM** it is not party to?

### §1.2 — The trace, gate by gate

Entry: `NodeRuntime::dispatch_event` — `xgen-core/src/node/runtime.rs:1120` — with `origin = EventOrigin::LocallySubmitted`, `peer_node_id = None`.

| # | Gate | Site | Verdict | Why |
|---|---|---|---|---|
| a | `space_id_of` resolves | `runtime.rs:1144-1149` | PASS | join carries a non-empty `space_id` |
| b | Step 1 — Space known locally | `runtime.rs:1168-1173` | PASS | the DM Space exists on this Node |
| c | Step 2 — F-3 federation relationship | `runtime.rs:1191` | **NOT REACHED** | guarded by `if let Some(peer) = peer_node_id`; local submission passes `None` |
| d | Step 8 — `event_id` == canonical hash | `exchange.rs:527-538` | PASS iff built/signed correctly | structural |
| e | Step 8.5 — future-skew ceiling | `exchange.rs:545-565` | PASS | fixture timestamps are `now`-ish |
| f | Step 10 — DAG structure | `exchange.rs:569-572` → `:601` | PASS iff ≥1 `prev_events` | `MembershipJoin` is **not** a DAG root |
| g | Step 9 — predecessors present | `exchange.rs:582-598` | PASS iff `prev_events` are in the store, else **HeldPending** | fixture must chain from a real tip |
| h | **Step 11a — sender is a registered Identity** | `exchange.rs:625-634` | **PASS *because the actor is registered*** | `node_authored` false, `fed_add_via_federation` false ⇒ unregistered would be `HeldPending{missing_identity}` |
| i | Step 11b — sender is a Space member | `exchange.rs:637-681` | **SKIPPED** | `MembershipJoin` ∈ `skip_membership` (`:649-659`) |
| j | Step 12 — signature verifies | `exchange.rs:684-686` | PASS | actor signs with its own key |
| k | Node-authority gates | `exchange.rs:692-725` | NOT REACHED | wrong event types |
| l | Step 13 — permission | `exchange.rs:728-735` | **SKIPPED** | same `skip_membership` |
| m | Dedup-at-dispatch | `runtime.rs:1391-1395` | PASS | fresh `event_id` |
| n | AI role violation (3041) | `runtime.rs:1398-1409` | NOT REACHED | gated on `is_space_creation` |
| o | `check_ai_capability` (3042) | `runtime.rs:1412-1414` → `exchange.rs:254` | **PASS, no-op** | `!record.is_ai` ⇒ early `Ok(())` (`exchange.rs:265-267`); `make_identity_record` sets `is_ai: false` |
| p | AI operator target/permission | `runtime.rs:1415-1430` | NOT REACHED | wrong event types |
| q | Invite over-ceiling (3045) | `runtime.rs:1462-1502` | NOT REACHED | gated on `MembershipInvite` |
| r | **Banned pre-check** | `runtime.rs:1522-1531` | **PASS** | intruder is not in `space.banned` |
| s | **Tier gate — `verify_tier_assertion`** | `runtime.rs:1532-1544` | **PASS as a no-op** | §1.3 |
| t | **Invite-expiry gate (3044)** | `runtime.rs:1580-1610` | **NOT ENTERED** | §1.4 — *the crux* |
| u | Thread tier gates | `runtime.rs:1626-1655` | NOT REACHED | wrong event type |
| v | `new_joiner` detection | `runtime.rs:1657-1672` | `Some(intruder)` | `space.is_member` false pre-ingest |
| w | `ingest_event` → `apply_join` | `runtime.rs:1758` → `state.rs:998` | **THE INSERT** | §1.5 |

### §1.3 — Gate (s), the tier gate: it passes, and only because two unrelated constants agree

`joiner_tier = identity_registry.get(&event.sender).map(assertion_tier_of).unwrap_or(1)` (`runtime.rs:1532-1536`).

- `assertion_tier_of` — `runtime.rs:241-246` — returns **1** on `trust_assertion: None`.
- `make_identity_record` — `phase9_harness.rs:1263-1277` — sets `trust_assertion: None`.
- `space.auth_tier` — `from_dm_space_create_node` reads `content["auth_tier"].as_u64().unwrap_or(1)` (`state.rs:504`); `build_dm_space_create_event` writes `"auth_tier": 1` (`state.rs:1824`).
- `verify_tier_assertion(1, 1)` — `tiers.rs:158-166` — `1 < 1` false ⇒ `Ok(())`.

⚠️ **Neither constant is pinned by anything the fixture controls.** If either moves, the join rejects `3030 tier_mismatch` and **the test fails with a message about tiers while claiming to be about admission** — a wrong diagnosis, delivered silently and permanently.

### §1.4 — Gate (t): NOT ENTERED — and *which arm is not taken* is the whole point

Both outer conditions hold: `origin == LocallySubmitted` ✓ and `event.room_id.as_str().is_empty()` ✓ (`runtime.rs:1580`). The gate is then entered only through `if let Some(pi) = space.pending_invites.get(&event.sender)` (`runtime.rs:1586`).

**The intruder has no pending invite** ⇒ `None` ⇒ **the entire block is skipped; nothing is evaluated.**

🔑 **Contrast with the DM counterpart.** `from_dm_space_create_node` seeds the invitee into `pending_invites` with `valid_until: None` (`state.rs:538-545`). A counterpart actor **would** enter the gate, match `None if !space.dm_constraints_active` (`runtime.rs:1603`), find `dm_constraints_active == true`, and fall to `None => {}` — *"DM-seeded invite: exempt by design"* (`runtime.rs:1609`).

⇒ **Both actors reach the join. They reach it through DIFFERENT ARMS.** A counterpart-actor fixture would exercise the invite-expiry exemption and report it as admission. **The third identity is what keeps the subject on the intended gate.**

### §1.5 — The insert

`apply_join` — `state.rs:998-1035`, space-level branch:

- `members.contains_key(joiner)` false ⇒ no `AlreadyMember` (`:1016`)
- `banned.contains(joiner)` false ⇒ no `Banned` (`:1019`)
- `pending_invites.remove(joiner)` ⇒ **`None`** ⇒ `(Role::Member, None)` (`:1022-1025`)
- `members.insert(joiner, SpaceMember { identity_id, role: Role::Member, joined_at: event.timestamp, invited_by: None })` (`:1026-1034`)

### §1.6 — MY ANSWER

🛑 **THE JOIN IS ADMITTED.** `DispatchOutcome::Accepted { new_joiner: Some(intruder), additional_persisted: [] }`, and the DM's `SpaceState.members` gains the intruder as **`Role::Member`**, **`invited_by: None`**.

**No gate between `dispatch_event` entry and the `members` insert asks whether the sender was invited, whether the sender is party to the DM, or whether the Space wants strangers.** The only thing between an arbitrary registered Identity and membership of somebody else's DM is that the Identity must be *registered* — plus knowledge of the `space_id`, which is obscurity, not access control.

**Deployment state:** the harness registers **by construction, not by policy**. `InProcessNode::register_identity` (`phase9_harness.rs:501-506`) calls `NodeRuntime::register_identity` (`runtime.rs:698`) **directly**; a corpus-wide search for `accept_registration` finds exactly **one** production call site — `xgen-node/src/app.rs:3479` — every other hit a test or doc comment. ⇒ **no `AssertionPolicy` is ever consulted in this harness.**

📌 **§1 was written and saved to disk before the runbook was opened.** Everything from §2 on was written after.

✅ **§1.6 and §4.5 AGREE — outcome, role, and `invited_by` all three.** The runbook's expected outcome is confirmed by an independent derivation, which is what §9.1 was for.

---

## §2 — 🛑 `F-1` (PLAN-MOVING): §2 OF THE RUNBOOK IS A **CENSUS**, NOT A PARTITION

**This was §9.2's question 1 and its own §8 item 2's "likeliest defect". It is the defect.**

### §2.1 — The measurement

📌 **Metric, stated:** `return DispatchOutcome::` sites inside `dispatch_event`'s body (`runtime.rs` lines 1120–1758) and `return ValidationOutcome::` sites inside `validate_event`'s body (`exchange.rs` lines 489–737), counted mechanically by line-bounded `awk`, then read individually.

- **`dispatch_event`: 19 non-`Accepted` exit sites.** Two of them (`:1337`, `:1361`) are the funnel for `validate_event`.
- **`validate_event`: 16 non-`Validated` exit sites.**
- ⇒ **≈33 distinct gates** on the path (17 dispatch-local + 16 validation-local).

**Runbook §2 lists eight rows, `X-0`…`X-7` — and `X-0` is not on the path at all** (it is named as a bound, correctly). ⇒ **seven rows for ~33 gates.**

### §2.2 — Which unlisted gates are REACHABLE by this fixture

Most of the 26 unlisted gates are unreachable by event type (`MembershipInvite`, `ThreadCreate`, `StateSpaceCreate`, node-authority) and their absence costs nothing. **Six are reachable, and each fails with a diagnosis that is not about admission:**

| site | fires when | what the implementer sees |
|---|---|---|
| **`runtime.rs:1169`** | the `space_id` the fixture computed is not the one the runtime registered, **or `derive_resolved` returned `None` on the DM create** | `Rejected("space not found")` |
| **`exchange.rs:571`** (via `:601`) | `prev_events` is **empty** — `MembershipJoin` is not a DAG root | `Rejected(DagError("non-root event must reference at least one predecessor"))` |
| **`exchange.rs:594`** → `runtime.rs:1361` | `prev_events` reference events not in the store | **`HeldPending`** — see `F-2` |
| **`exchange.rs:538`** | anything mutated after `sign_event` | `Rejected(EventIdMismatch)` |
| **`exchange.rs:554/561`** | a hardcoded rather than `now_rfc()` timestamp | `Rejected(TimestampOutOfBounds)` |
| **`exchange.rs:686`** | wrong key used to sign | `Rejected(SignatureFailure)` |

🛑 **AND §2 IS NOT EVEN A COMPLETE PARTITION OF THE *AUTHORISATION* GATES:** `check_ai_capability` (`runtime.rs:1412-1414` → `exchange.rs:254`) runs **unconditionally on every validated event** and can reject `3042`. It is a no-op here only because `make_identity_record` sets `is_ai: false` — **the same shape as `X-3`, which §2 did think worth a row.**

### §2.3 — The consequence, and why it is not cosmetic

🔑 **§4.3's precondition list is derived entirely from the `X-` rows, so it asserts nothing about the DAG.** It checks DM-ness, `auth_tier`, non-membership and absence-of-invite — and **not one thing about `prev_events`, the store, or whether the Space was actually registered.** Those are exactly the six reachable unlisted gates.

⇒ **§4.3 gains two lines:**

```
assert!(node.has_space(&space_id).await, "DM SpaceState was not registered — derive_resolved returned None");
assert!(!tips.is_empty(), "no DAG tips — the join would fail on DAG structure, not on admission");
```

*(`has_space` exists — `phase9_harness.rs:566` in the impl block. `tips` must therefore be read before the precondition block rather than at §4.4.)*

### §2.4 — ⚖️ WHAT §2 GETS RIGHT, STATED SO THE FINDING IS NOT OVER-READ

**I found no *authorisation* gate §2 missed except `check_ai_capability`, and I looked mechanically rather than against a suspect list.** As an enumeration of *the gates that decide whether this actor is allowed in*, §2 is sound and its verdicts are correct at every cited span (§3 below).

🛑 **What overreaches is the CLAIM, not the content:** *"Every gate a `membership.join` passes between `dispatch_event` entry and `apply_join`"* and *"the complete set of gates between `dispatch_event` entry and the `members` insert"*. **Narrow the sentence to *the admission gates* and §2 becomes true.** Leave it as written and §4.3 keeps inheriting a blind spot from it — *which is exactly what it did.*

---

## §3 — `X-` AND `H-` ROW VERIFICATION (§9.2 question 2)

**Every cited span opened and read. No row's claim is false.** Four citations are imprecise.

### §3.1 — Rows that hold exactly

| row | span checked | verdict |
|---|---|---|
| `X-1` | `exchange.rs:601-634`; consumer `runtime.rs:1339-1361` | ✅ **exact** — the `if !fed_add_via_federation && !node_authored && !id_registry.contains(sender)` is at `:629` |
| `X-2` | `runtime.rs:1522-1531` | ✅ **exact** |
| `X-3` | `runtime.rs:1532-1544`; `assertion_tier_of` `:241` | ✅ **exact**, and its coincidence-of-two-constants reading is correct — I reached it independently at §1.3 |
| `X-5` | `exchange.rs:649-659` | ✅ **exact** |
| `X-6` | `exchange.rs:914` | ✅ **exact** — `_ => Ok(())` |
| `X-7` | `state.rs:1016`, `:1019`, `:1022-1025` | ✅ **exact, all three** |
| `H-1` | `phase9_harness.rs:516-576` | ✅ `submit_locally` body is `rt.dispatch_event(ev, EventOrigin::LocallySubmitted, None)` |
| `H-2` | `phase9_harness.rs:578-604` | ✅ `ingest` calls `rt.ingest_event`, bypassing `dispatch_event` |
| `H-4` | `phase9_harness.rs:651`; `state.rs:1277`, `:1284` | ✅ **exact, all three** — but see `F-5` |
| `H-5` | `state.rs:496-540`; `runtime.rs:1603-1608` | ✅ holds; the DM-exempt arm is at `:1609` |
| `H-6` | `state.rs:1812-1833`; `phase9_harness.rs:1263-1278` | ✅ **exact** |
| `H-8` | `xgen-node/src/tests/mod.rs` | ✅ hand-maintained `pub mod` list |

### §3.2 — ✅ `H-7` HOLDS — §9.2 question 3, and it is the generous-direction one

**Confirmed independently and by a corpus-wide search rather than by re-reading the row.** `accept_registration` has **exactly one production call site**, `xgen-node/src/app.rs:3479`; every other hit in the repo is a `#[cfg(test)]` block, a doc comment, or `xgen-auth-module/tests/`. `InProcessNode::register_identity` (`:501-506`) calls `NodeRuntime::register_identity(record)` (`runtime.rs:698`) directly.

⇒ **§6.1's bound is correct and is NOT generous.** *This was the question whose wrong answer would have been dangerous, and the answer is that the runbook is right.*

### §3.3 — 🛑 `F-6` (WORDING): four imprecise citations

1. **`X-4` cites `runtime.rs:1583-1610`; its own outer guard is at `:1580`.** The row's *text* quotes both conditions correctly, so nothing is misstated — **but the span begins three lines below the `if origin == LocallySubmitted && room_id.is_empty()` that makes the gate local-only**, and that guard is half the reason `X-4` behaves as it does. **This is the gate `H-5`'s entire third-identity argument rests on.** ⇒ `1580-1610`.
2. **`H-3` cites `runtime.rs:823-832`; the match arm head is at `:829`** and its body runs to `:841`. The cited span is mostly the preceding comment and stops mid-body.
3. **`H-3` also under-describes the mechanism, and it matters:** the create arm does **not** call a constructor — it calls **`derive_resolved(log, …)`**, which *"dispatches the create constructor internally"* (`runtime.rs:822-825`), and inserts a `SpaceState` **only** `if let Some(mut state) = derive_resolved(...)` (`:832`). ⇒ **a `None` from `derive_resolved` silently leaves the Space unregistered, and the join then dies at `runtime.rs:1169` "space not found"** — which is `F-1`'s single most likely wrong diagnosis. **The imprecision and the risk are the same thing.**
4. **§4.6 point 3 cites `state.rs:290-297`; the creator block is `:291-298`** (`let creator = event.sender.clone();` is at `:291`). Claim correct, span off by one.

---

## §4 — 🛑 `F-2` (PLAN-MOVING, AND THE SHARPEST FINDING): `V-3`'s NEGATIVE CONTROL CANNOT DISTINGUISH ITS OWN CAUSE

### §4.1 — The defect

`V-3` runs the fixture with carol **unregistered** and asserts the outcome is **`HeldPending`**, and the runbook calls it ***"the only step that proves the J-battle-743 fixture would have been a false pass"*** and *"without it, `X-1` is a claim in a document."*

🛑 **`DispatchOutcome::HeldPending` is a UNIT VARIANT. It carries no reason.** (`runtime.rs`, `pub enum DispatchOutcome` — `Accepted { .. }` carries two fields, `Rejected(RejectInfo)` carries one, **`HeldPending` carries nothing**.)

**`ValidationOutcome::HeldPending { missing_predecessors, missing_identity }` *does* carry the reason** (`exchange.rs:594`, `:630`) — and `dispatch_event` **discards it** at `runtime.rs:1361`, collapsing both causes into the bare variant. The F-3 path (`runtime.rs:1272`) returns the same bare variant a third time.

⇒ **Three distinct causes produce one indistinguishable value:**

| cause | site | reachable in `V-3`? |
|---|---|---|
| sender not registered | `exchange.rs:630` → `runtime.rs:1361` | ✅ **the one `V-3` intends** |
| **predecessors missing** | `exchange.rs:594` → `runtime.rs:1361` | ✅ **a plain fixture bug** |
| federation relationship missing | `runtime.rs:1272` | ❌ (`peer_node_id = None`) |

🔑 **⇒ A `V-3` RUN WHOSE `prev_events` ARE WRONG RETURNS `HeldPending` AND PASSES.** And the failure is worse than a wasted control: the **positive** test would fail at the same `:1361` with the same bare `HeldPending`, and an implementer reading it would conclude *"X-1 fired, carol isn't registered"* and go add a registration line that is already there.

🛑 ***THE CONTROL HAS EXACTLY THE DEFECT IT WAS WRITTEN TO EXPOSE.*** `N-197`'s species — *a check whose failure mode reads exactly like success* — **inside the step the runbook nominates as its proof against precisely that.** It is the same shape as J-743's *"use a fresh unregistered keypair"*, one level up: a correction that reproduces the defect it corrects.

### §4.2 — The fix is cheap, and the codebase already does it

**`PendingBuffer::pending_identity_count()` is the discriminator and it already exists** — `xgen-core/src/dag/pending.rs:556`, `pub`. **`NodeRuntime.pending` is `pub`** (`runtime.rs:312`) and **`InProcessNode.runtime` is `pub`**, so it is reachable through the harness.

✅ **PRECEDENT, NOT INVENTION: `heldpending_identity_integration.rs:245-255` asserts `HeldPending` and then immediately asserts `pending_identity_count()`** — the exact discriminator, in a shipped test, for the exact gate.

⇒ **`V-3` gains one assertion:**

```
assert!(matches!(outcome, DispatchOutcome::HeldPending));
assert_eq!(
    node.runtime.lock().await.pending.get(&sdx(&space_id))
        .map(|b| b.pending_identity_count()).unwrap_or(0),
    1,
    "HeldPending must be on the IDENTITY trigger (X-1), not on a missing predecessor"
);
```

🔒 **AND A SEQUENCING RULE, WHICH IS FREE:** run `V-3` **after `V-1` is green**, on the **same fixture**, with **the registration line as the only delta.** A green positive run proves the DAG setup is sound, so the single changed variable is registration — *which is what makes the control a control.* **The runbook implies the same fixture; it does not say the positive must be green first, and it does not name the ambiguity.**

📌 **`V-3` stays a discarded probe** (§7 item 6 is untouched) — the discriminator is added to the probe, not shipped.

---

## §5 — 🛑 `F-3` (WORDING): `V-2`'s "57 suites" BRANCH IS STRUCTURALLY IMPOSSIBLE

`V-2` reads: *"**57 suites** if a new binary is produced; **56** if it folds into the existing `xgen-node` test binary. 🛑 MEASURE IT — do not predict it."*

**Measured: `xgen-node/src/lib.rs:41-42` is `#[cfg(test)] pub mod tests;`.** A file at `xgen-node/src/tests/<name>.rs` registered via `pub mod` in `src/tests/mod.rs` — which is exactly where §4.1 puts it — compiles into the **existing lib unit-test binary**. It cannot produce a new suite.

✅ **"Measure it, do not predict it" is right and I am not asking for it to be dropped.** 🛑 **But offering 57 as a legitimate outcome is wrong:** **56 is the only correct value, and a reading of 57 means the file was placed somewhere other than §4.1 says** — i.e. it is a *finding*, not an alternative. As written, an implementer who sees 57 has been pre-authorised to accept it.

---

## §6 — 🛑 `F-4` (WORDING): AN EXPIRED OPTION SURVIVING ITS OWN LOCK

**§5 `V-1`** still reads *"→ **1603** (or **1604** with §4.6)"*. **§4.6 is LOCKED and SHIPS** (Joe, 2026-08-17) and **§10 item 5 says `1602` → `1604`** flatly.

⇒ **One document, two expectations, and the 1603 branch died when §4.6 was locked.** Small — **but it is this arc's own named species** (`N-109`: a countdown whose trigger has fired), sitting in the gate table an implementer reads while deciding whether a number is correct. ⇒ `V-1` reads **1604**, full stop.

---

## §7 — 🛑 `F-5` (COMPLETENESS): `H-4` DOES NOT NAME THE ACCESSOR §4.5 ASSERTION 3 NEEDS

`H-4` lists the read-back accessors as `space_state`, `member_role`, `is_member`. **None of them can reach `invited_by`** — `member_role` returns `Option<&Role>` (`state.rs:1277`).

✅ **The assertion IS implementable:** `SpaceState.members` is `pub HashMap<IdentityXgid, SpaceMember>` (`state.rs:221`) and `SpaceMember.invited_by` is `pub Option<IdentityXgid>` ⇒ `state.members.get(&idx(&carol_id)).unwrap().invited_by.is_none()`.

⚠️ **Flagged because §8 item 6 makes "weakening an assertion" a reportable deviation, and an implementer who cannot find an accessor is precisely the person who weakens one.** §4.5 assertion 3 is the one the runbook calls ***"the assertion that names the hole"***. **One line in `H-4` heads it off.**

---

## §8 — ✅ §9.2 QUESTION 4: **YES — THERE IS A FOURTH ASSERTION, AND IT IS STRONGER THAN THE THREE**

**Proposed:** after the join, assert **`!state.is_member(&bob_id)`** — and that `state.members.len()` went **1 → 2**.

**Grounded:** `from_dm_space_create_node` seeds **only the creator** into `members` and puts the invitee into `pending_invites` (`state.rs:526-545`). Bob never emits a `membership.join` in this fixture. ⇒ **before: `members == {alice}`. After: `members == {alice, carol}`.**

🔑 **THE FACT THAT NAMES: THE DM'S ACTUAL COUNTERPART IS NOT A MEMBER, AND THE STRANGER IS.** A two-party DM now holds two members, **neither pair being the two parties**, and `is_dm` / `dm_constraints_active` are both still `true`. **Nothing anywhere enforces a DM's two-party invariant** — `apply_join` has no member-count check and no DM branch at all on the space-level arm.

**It meets the question's test on both halves:** it becomes **unobservable after Leg D** (the gate refuses the join, so the state never arises), and **nothing else records it** — assertions 1–3 all describe *carol*, and none of them says anything about what happened to the DM as an object.

📌 **A weaker second candidate, mentioned and not recommended:** `new_joiner == Some(carol_id)` on the `Accepted` payload. It is the fan-out trigger — i.e. the observable proxy for *"she will receive this DM's future traffic"*, which is the actual security consequence. **I do not propose it** — it is a dispatch-layer detail where assertion 2 already carries the state-layer fact, and adding it would put a fan-out claim in a test that does not exercise fan-out.

---

## §9 — §9.2 QUESTION 5: WHAT ELSE IN §4.6 DOES NOT SURVIVE THE SOURCE

**§4.6 was flagged least-trusted at two-for-five. I re-drove all five points. Three hold exactly; one is off by a line; one is ambiguous.**

| point | verdict |
|---|---|
| 1 — alice + carol, both registered, no bob | ✅ sound |
| 2 — `build_space_create_event(&alice_key, "<name>", None, 1, &node.node_id, None, false)`; trailing `false` is `e2e_encryption` | ✅ **EXACT.** Measured `state.rs:1381-1389`: `(key, name, topic: Option<&str>, auth_tier: u32, home_node, jurisdiction: Option<&str>, e2e_encryption: bool)`. The call matches parameter-for-parameter. **Chat's correction is right.** |
| 3 — `from_space_create` seeds the creator as `Role::Owner` | ✅ **TRUE** (`state.rs:291-298`) — span off by one (`F-6.4`). **And the sibling-is-not-the-model lesson is correct**: `phase9_unknown_signer_first_contact.rs` does ingest an invite + join for an already-owner identity through `ingest`, which bypasses `dispatch_event`. |
| 4 — preconditions | ⚠️ **AMBIGUOUS — see below** |
| 5 — subject event + all three assertions unchanged | ✅ sound; carol has no invite in an ordinary Space either, so assertion 3 still holds |

### 🛑 `F-7` (WORDING): §4.6 point 2 says the DM preconditions "still ship" without saying they **INVERT**

§4.6 opens *"IDENTICAL TO §4.2–§4.5 EXCEPT WHERE STATED"*, and §4.3's precondition list contains `assert!(state.is_dm)` and `assert!(state.dm_constraints_active)`. Point 2 then says those *"still ship — they are cheap and they document the test's subject"*.

🛑 **For an ordinary Space they must be asserted `false`** (`from_space_create` hardcodes `is_dm: false` and `dm_constraints_active: false`, `state.rs:320`/`:330` — which point 2 itself cites). **The prose never says "inverted".** A literal reading of "identical except where stated" plus "still ship" produces `assert!(state.is_dm)` in the open-Space test.

📌 **It would fail loudly, so it is a nuisance rather than a hazard** — but it is unclear prose in the section the runbook itself nominates as least-trusted, and one word fixes it.

---

## §10 — ✅ `F9`: CAN EACH §5 GATE BE RUN, IN ORDER, FROM THE SEAT THAT OWNS IT?

| gate | runnable? |
|---|---|
| `V-1` cargo, detached + sentinel | ✅ yes — and the detached-with-sentinel instruction is right; `cargo` exceeds the MCP timeout here |
| `V-2` suite count | ✅ yes — see `F-3`; the answer is **56** |
| `V-3` negative control | ⚠️ **runnable, but not conclusive as written** — `F-2`. With the discriminator, ✅ |
| `V-4` diffstat scope | ✅ yes |
| `V-5` `git ls-files --eol` | ✅ yes |

✅ **§4.4's code block compiles as written — every symbol resolves**, checked individually rather than assumed: `spawn_in_process_node` `:751` · `register_identity` `:501` · `ingest` `:578` · `submit_locally` `:516` · `dag_tips` `:605` (returns `Vec<String>`, so `tips.iter().map(|t| edx(t))` is right) · `space_state` `:651` · `has_space` `:566` · `event_id_str` `:83` `pub(crate)` · `idx`/`ndx`/`sdx`/`edx`/`rdx` `:66-81` all `pub(crate)` · `now_rfc` `:1247` · `pubkey_uri` `:1254` · `keypair::generate()` (used at `:781` and in the sibling) · `sign_event` `state.rs:1304` `pub`, and it **sets both `event_id` and `signature`** so Step 8 is satisfied by construction.

---

## §11 — WHAT I DID **NOT** DO

1. 🛑 **I ran no build and no tests.** `cargo`, vitest and svelte-check floors are **carried, not measured**: cargo **1602 / 0 / 62 × 56 SUITES** · vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15**. 🛑 **Catalogue UNMEASURED.**
2. 🛑 **§4.4 is asserted to compile because every symbol resolves — not because it was compiled.** A type error inside the `Event::new` argument list would not have been caught by this read.
3. 🛑 **I did not trace `derive_resolved` end-to-end** for either the DM or the ordinary-Space create. `F-6.3`'s risk (a `None` leaving the Space unregistered) is **reasoned from the `if let Some(...)` at `runtime.rs:832`, not observed.** §2.3's `has_space` precondition is what would surface it.
4. 🛑 **No exploit, no wire, no second node.** Everything here is a source trace, exactly as §6 says of the leg itself.
5. 📌 **I read §2 mechanically in both directions but §3's `H-` rows one direction only** — I confirmed each cited span says what its row claims; **I did not sweep for facts the `H-` rows should have carried and do not.**

---

## §12 — SUMMARY FOR JOE

| # | finding | class | change |
|---|---|---|---|
| **F-1** | §2 is a **census**, not a partition — ~33 gates on the path, 7 rows; six unlisted gates are reachable and fail with a non-admission diagnosis | 🛑 plan-moving | narrow §2's claim to *the admission gates*; add two `has_space` / `!tips.is_empty()` preconditions to §4.3 |
| **F-2** | **`V-3`'s control cannot distinguish its own cause** — `DispatchOutcome::HeldPending` is a unit variant and three causes collapse into it | 🛑 plan-moving | add `pending_identity_count() == 1` (precedent: `heldpending_identity_integration.rs:255`); run `V-3` **after** `V-1` is green, single-line delta |
| **F-3** | `V-2`'s "57 suites" branch is structurally impossible — `#[cfg(test)] pub mod tests` | wording | **56** is the only correct value; 57 is a finding, not an alternative |
| **F-4** | `V-1` still offers `1603` after §4.6 was locked to ship | wording | `1604`, full stop |
| **F-5** | `H-4` does not name the accessor §4.5 assertion 3 needs | completeness | name `state.members[..].invited_by` |
| **F-6** | four imprecise citations; `X-4`'s span omits its own outer guard; `H-3` under-describes `derive_resolved` | wording | `1580-1610`; `823-841`; `291-298`; name `derive_resolved` |
| **F-7** | §4.6's DM preconditions must **invert**, and the prose does not say so | wording | one word |
| **Q4** | a **fourth assertion** is available and is stronger than the three | addition | `!state.is_member(&bob_id)` — the counterpart is out and the stranger is in |

🔒 **Nothing in §4.2–§4.5's fixture design needs to change.** The actor, the third identity, `rdx("")`, the three assertions and the `H-6` tier precondition all hold against source, and my independent §1 reaches §4.5's answer.

🔓 **Everything above is reported, not applied — `D-123` Rule 6. Standing me up to implement is Joe's, and the runbook is Chat's to amend.**

---
