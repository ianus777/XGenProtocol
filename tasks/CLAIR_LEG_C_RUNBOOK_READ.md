# Clair cold read — `RUNBOOK_SPACE_ADMISSION_LEG_C.md` v1.0
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-19  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — SCOPE, TREE, AND METHOD

**Subject:** `tasks/RUNBOOK_SPACE_ADMISSION_LEG_C.md` **v1.0** (untracked, working tree) and
`tasks/M_SPACE_ADMISSION_PHASE0.md` **v2.4** §15.3 + §15.3b (modified, working tree).

🔒 **TREE DISCIPLINE (`D-152` clause 1).** `git status` at open shows **exactly two dirty paths, both `.md`**.
⇒ **every `.rs` file in the working tree is byte-identical to `d8a44f6`, and every `.rs` citation below is
a `d8a44f6` citation.** Where I cite `f45bb13` I say so.

**Method.** §7.1 was executed first: `check_permission` and `apply_space_temperature_visibility` were opened
and the non-owner path derived end-to-end **and written to disk before §3 or §4 were read**. That derivation
is reproduced in §1. Nothing below is inherited from the runbook's own measurement notes.

✅ **STATE RE-MEASURED:** `HEAD` `d8a44f6` **= `origin/main` by `git ls-remote origin refs/heads/main`**.
Dirty exactly as the brief predicted. **No product code written. Runbook not edited.**

---

## §1 — §7.1 DERIVATION (independent, written before §3/§4 were read)

**Question: what happens TODAY when a NON-OWNER sends `state.space_temperature_visibility`?**

| step | measured @ `d8a44f6` | result |
|---|---|---|
| override layer | `event_room_permission` (`exchange.rs:786-804`) — the variant is in no arm ⇒ `_ => None` (`:803`) | no override |
| **validation** | `check_permission` (`exchange.rs:807-916`) — the variant is named in **no** arm ⇒ `_ => Ok(())` (**`:914`**) | **PERMITTED** |
| dispatch | `dispatch_event` (`runtime.rs:1120`) — its typed gates are join/banned, invite-expiry, tier, federation. None matches | **PASSES** |
| **persist** | `self.ingest_event(event)` (`runtime.rs:1759`) → `store.append(event.clone())` (`runtime.rs:811`) | **ON DISK** |
| fold routing | `state_key_for_event` — the variant has **zero** hits in `resolution/state_key.rs` ⇒ `None` ⇒ `conflict = false` | incremental arm |
| **fold** | `let _ = state.apply_event(&event, &my_node_id)` (**`runtime.rs:867`**) → `state.rs:650` → `apply_space_temperature_visibility` (`state.rs:785`), whose **first statement** returns `Err(PermissionDenied)` (`state.rs:786-790`) | **ERROR DISCARDED** |
| **reply** | `DispatchOutcome::Accepted { .. }` (`runtime.rs:1795`) | **`Accepted`** |

✅ **M-1 HOLDS, INDEPENDENTLY DERIVED.** Accepted → persisted → replied `Accepted` → dropped by the fold,
error discarded. **The design rests on a true claim.**

🔑 **AND IT IS STRONGER THAN M-1 STATES — THE CODEBASE ALREADY NAMES THIS SPECIES AND ALREADY CHOSE A FIX.**
`runtime.rs:1505-1522` (MP-F6 / M10.5-D2/D3), for `MembershipJoin` + banned, in its own words:
*"accepted-but-inert … the dispatch reply was `Accepted` (is_ok=true) for an event `derive_resolved` will drop.
The end-state stayed correct (resolution is a second gate), but the **reply lied**. Surface the reject HERE
(the reply); the apply-layer silence … stays — it is load-bearing for replay tolerance."*
📌 **MP-F6's fix site is `dispatch_event`, not `check_permission`** — a precedent §4.4 does not cite and
arguably should, because it is the same problem solved once already.

📌 **A SECOND SILENT SIBLING, unasked and not this leg's:** `apply_space_pacing` (`state.rs:765-780`) carries
the identical owner check and is likewise absent from `check_permission` ⇒ **`state.space_pacing` from a
non-owner is the same silent no-op today.** Two siblings, not one. Filed only so no record claims admission
is the first such case.

---

## §2 — FINDINGS

### 🛑 F-1 — BLOCKING. §4.4 LEAVES THE **DM** CASE AS EXACTLY THE M-1 SILENT NO-OP IT EXISTS TO ELIMINATE.

§4.4 puts the **role** check in both places and the **DM** check in **(b) only**. Trace the owner-in-a-DM case
against the shipped machinery:

1. Owner sends `state.space_admission` on a DM. `check_permission`'s new arm calls `can_change_admission(Owner)` ⇒ **`Ok`**.
2. Not in `skip_membership` (`exchange.rs:649-658`), so validation runs — and **passes**.
3. `ingest_event` → **persisted** (`runtime.rs:811`).
4. `let _ = state.apply_event(...)` (`runtime.rs:867`) → `apply_space_admission` → DM refusal → **`Err` discarded**.
5. Reply: **`Accepted`**.

⇒ **the owner is told their DM's admission changed, and it did not.** §1's frozen goal has two clauses —
*"anyone else who tries is REFUSED IN A WAY THEY CAN SEE"* **and** *"a DM's pin cannot be changed at all"* —
and §4.4 satisfies the visibility clause for the **role** axis only. **On the DM axis the leg reproduces M-1.**

🔑 **AND IT IS AVOIDABLE AT ZERO ARCHITECTURAL COST: `check_permission` ALREADY HOLDS WHAT IT NEEDS.**
Its signature is `check_permission(event: &Event, space: &SpaceState)` (`exchange.rs:807`), and
**`dm_constraints_active` is a `pub` field** on `SpaceState` (`state.rs:240`). The DM test can sit in the
same new arm as the role test, one `if` above it.

🛑 **AND §4.4(b)1 TELLS THE IMPLEMENTER TO COPY THE DEFECT.** **S-9** is offered as *"the refusal pattern"* —
`apply_invite`'s DM bar (`state.rs:995-998`; fn at `:995`, bar at `:996-998`). **That bar is applier-only.**
`MembershipInvite` **is** in `check_permission` (`exchange.rs:866`) but **only for the role** (`can_invite`);
nothing there consults `dm_constraints_active`. ⇒ **an invite in a DM today is accepted, persisted, answered
`Accepted`, and silently dropped** — a **third live instance** of M-1's species, and it is the very pattern
§4.4(b)1 says to model. **Copying S-9 propagates the defect onto the new event.**

**What changes if it holds:** §4.4(a) gains the DM condition; §4.6 test 4 must assert at the **dispatch** level
(see F-7), not only at the applier; §1's second clause becomes reachable. **Blocks the lock.**

📌 *Not proposed: fixing `apply_invite`. That is the sibling's defect, and riding it in would make Leg C's diff
argue two cases at once — the Phase-0's own stated reason for filing the temperature sibling separately.*

---

### 🛑 F-2 — BLOCKING. **`V-4` CANNOT PRODUCE THE OUTCOME IT DEMANDS, AND ITS FAILURE READING IS BACKWARDS.**

`V-4` requires: *delete the variant from `known_variants()` (S-4) ⇒ **the round-trip test must FAIL***.

**Measured @ `d8a44f6`.** `known_variants()` is `wire.rs:770-791`. It has **exactly three consumers**, and
**all three are bare sweeps with no count assertion anywhere:**

| consumer | shape |
|---|---|
| `wire.rs:806` `known_type_round_trips_byte_identically` | `for t in known_variants() { … }` |
| `wire.rs:818` `as_str_and_from_str_are_inverse_for_known` | `for t in known_variants() { … }` |
| `wire.rs:864` `infra_predicate_and_kinds_name_the_same_set` | `for t in known_variants() { … }` |

⇒ **deleting an entry makes each loop iterate one fewer item and every remaining assertion still passes.**
**The control PASSES. It cannot fail.**

🛑 **AND THE RUNBOOK CONTRADICTS ITSELF ACROSS TWO SECTIONS.** **S-4** and **§4.1** both say — correctly —
*"omission does not fail: it silently drops the variant from the round-trip test"*. **`V-4` demands a loud
failure from the mechanism its own site table calls silent.** Both cannot be true, **and S-4 is the true one.**

🛑 **`V-4`'s stated interpretation of a pass is therefore also wrong:** *"If it still passes, S-4 is decorative
and the runbook is wrong about it."* **S-4 is not decorative** — omitting the new variant means the new variant
is never round-tripped, a real coverage hole. A pass here is a fact about **the control**, not about S-4.
⇒ ***as written, `V-4`'s only possible outcome would be recorded as a finding against the correct half of the
runbook*** — `N-197`'s shape, inside the gate written to be a control.

**A control that CAN fail, if one is wanted:** keep `known_variants()` intact and **break `from_str` (S-3)** —
`as_str_and_from_str_are_inverse_for_known` (`wire.rs:815`) then fails **for the new variant by name**, which
demonstrates that membership in the list is what buys the coverage. Run both together (omit from the list
**and** break `from_str`) and the suite goes **green** — that pair demonstrates the silence directly, which is
what S-4 actually claims. 📌 Alternatively a one-line `assert_eq!(known_variants().len(), 60)` makes the
omission loud forever — **a design change, so Chat's or Joe's, not mine.**

**What changes if it holds:** `V-4` is rewritten or struck; §8's DoD item *"V-3 and V-4 both run, both controls
FAILED"* is unsatisfiable as worded. **Blocks the lock.**

✅ **`V-3` IS SOUND AND WAS CHECKED:** deleting the `check_permission` arm drops the event to `_ => Ok(())`
(`exchange.rs:914`), so test 2 fails while test 3 (applier, independent) still passes. **`V-3` can fail, and
its expected outcome is precise.**

---

### 🛑 F-3 — BLOCKING (evidence, not conclusion). **M-1 CITES TWO `#[cfg(test)]` SITES AS "PRODUCTION CALL SITES".**

**M-1** states: *"applier errors are **discarded** at `exchange.rs:1279` and `:2319`"*, and §15.3 of the Phase-0
introduces the identical pair with the words **"at production call sites"**.

**Measured @ `d8a44f6`:** `exchange.rs:1096` is `#[cfg(test)]`, `:1098` is `mod tests {`.
⇒ **`:1279` and `:2319` are BOTH inside the test module.**

**The production discard set is different:**

| site | enclosing | status @ `d8a44f6` |
|---|---|---|
| **`runtime.rs:867`** | `ingest_event` (`:704`) — the Node's incremental fold | **PRODUCTION** (`#[cfg(test)]` at `:2369`) |
| **`derive.rs:231`** | `fold_skipping` (`:210`) — the `derive_resolved` rebuild | **PRODUCTION** (`#[cfg(test)]` at `:299`) |
| **`ai_service.rs:553`** | `xgen-client` | **PRODUCTION** (`#[cfg(test)]` at `:668`) |
| `derive.rs:415` · `state.rs:3449` · `exchange.rs:1279` · `:2319` | — | **TEST** |

✅ **The conclusion is unaffected — I derived M-1 independently via `runtime.rs:867` before reading §3.**
🛑 **But an implementer following §7.1 and then checking M-1's citations lands in a test module**, where the
honest reading is *"this is only discarded in tests"* — **the opposite of M-1's point.**

📌 **It also strengthens M-1: `derive.rs:231` is inside the resolution rebuild**, so the applier error is
discarded on **both** the incremental and the convergent path — which is precisely why MP-F6 could say
*"the end-state stayed correct … but the reply lied"*.

**What changes if it holds:** M-1's citation pair is replaced. **Text-only; blocks the lock only because the
runbook is the implementer's map.**

---

### 🛑 F-4 — BLOCKING (mechanism). **`M-4` IS WRONG TWICE, AND INVERTS THE COMMENT IT DRAWS FROM.**

**M-4** states: *"`ExchangeError::PermissionDenied` **does** reach the sender: `runtime.rs:1519-1525` maps it
to a `Rejected` outcome, wire `4000`."*

**① `to_wire_code` does NOT map `PermissionDenied`.** `exchange.rs:130-144` maps exactly five variants
(3042 · 3041 · 3043 · 6009 · 3046) and ends **`_ => None` at `:140`**. `PermissionDenied` falls to `None`.

**② The cited site is not a mapping site.** `runtime.rs:1519-1525` is the **MP-F6 banned-join comment plus its
`MembershipJoin`-specific reject** — nothing generic. **The real mechanism is `RejectInfo::from_exchange`
(`runtime.rs:190-195`): `None => Self::generic(err.to_string())` (`:193`) → `RejectInfo { code: 4000,
name: "generic", reason: <Display> }` (`:178`).**

🔑 **And the inversion is literal.** The comment at `runtime.rs:1519-1520` reads
*"PermissionDenied-class (**4000-unmapped**: `to_wire_code` returns None for it, exchange.rs:140)"*.
**"4000-unmapped" means unmapped and therefore falling back to the generic 4000 — M-4 read it as
"wire code 4000".**

✅ **The conclusion survives: the sender does receive `4000` and the reason string.** But the corrected
mechanism carries a consequence the runbook does not:

🛑 **`4000` / `"generic"` IS THE SAME CODE AND NAME AS EVERY OTHER GENERIC REJECT** —
`"event missing event_id"` (`runtime.rs:1147`), `"space not found"` (`:1169`), `"store init failed"` (`:1163`),
and every other `RejectInfo::generic`. **Only the free-text `reason` distinguishes an admission refusal.**
⇒ **§1's *"refused in a way they can see"* is delivered by a STRING, not by a code**, and any test asserting
only `code == 4000` **passes for several wrong reasons as well as the right one** — `N-197` again.

**What changes if it holds:** M-4 is rewritten onto `from_exchange`; §4.4(a)'s *"a wire reject the sender sees
(M-4)"* gains the honest caveat; any dispatch-level assertion keys on `name`/`reason`, not on `code` alone.
**Blocks the lock.**

---

### 🛑 F-5 — BLOCKING. **`S-13` IS AN `f45bb13` CITATION UNDER A HEADER THAT SAYS `d8a44f6`.**

§2's header: *"**MEASURED AT `d8a44f6`.**"* **S-13** cites `state.rs:1382-1390` for `build_space_create_event`.

| tree | `build_space_create_event` |
|---|---|
| `f45bb13` (pre-Leg-B) | **`:1382`** |
| **`d8a44f6` (HEAD)** | **`:1415`** |

**At `d8a44f6`, `state.rs:1382-1390` is `fn prev_events_to_xgids`** — an unrelated XGID projection helper.
⇒ **an implementer opening S-13 as instructed lands on the wrong function.**

🔑 ***This is `D-152` clause 1 — the rule minted two days ago — inside the site table of the first runbook
written after the mint.*** ✅ Every other §2 citation holds at the stated tree (see §3).

**What changes if it holds:** one number. **Blocks the lock because §2 is the map.**

📌 **The same defect is in the parent, and worse — §15.3 of Phase-0 v2.4 MIXES TREES INSIDE ONE SECTION:**
`apply_space_temperature_visibility (state.rs:752-768)` and `apply_dm_promote (state.rs:659-666, the flip at
:664)` are **`f45bb13`** positions (`:752` / `:659` there; `:785` / `:692` at `d8a44f6`), while the same
section's *"its owner check lives only in its applier (`state.rs:786`)"* is a **`d8a44f6`** position.
**Neither states its tree.** ✅ The runbook's **S-7** already carries the corrected `:785-796` — *it fixed the
parent's applier citation and introduced a new stale one at S-13.*

---

### ⚠️ F-6 — NOTE (right conclusion, half-wrong reason). **THE FEDERATED PATH *DOES* PASS THROUGH `check_permission`.**

§4.4(b) justifies the applier re-check as *"defence in depth for the federated and replay paths, **which do not
pass through `check_permission`**"*.

**Measured @ `d8a44f6`:**

- **Federated — FALSE.** `xgen-node/src/app.rs:3209` — `rt.dispatch_event(event.clone(), origin,
  peer_node_id_owned.as_ref())` inside `process_inbound`. Federation events therefore reach
  `validate_event` (`exchange.rs:489`) → `check_permission` (`exchange.rs:731`), and
  **`StateSpaceAdmission` is not in `skip_membership`** (`exchange.rs:649-658` — join, both creates,
  node-eject/unban, migrate, plus `fed_add_via_federation`). ⇒ **it is checked.**
- **Replay — TRUE.** `xgen-node/src/app.rs:5028` — `runtime.ingest_event(event)` inside
  `replay_spaces_from_dir`, **bypassing `dispatch_event` entirely**. Plus `derive.rs:231`'s `fold_skipping`,
  which applies with no permission layer at all.

✅ **The instruction survives — (b) is genuinely load-bearing — on replay and `derive_resolved`, not on
federation.** 📌 Worth correcting because the false half is the half an implementer would use to judge how
hard to defend (b).

---

### ⚠️ F-7 — NOTE, and the one I would most want taken. **ALL SIX TESTS ARE UNIT-LEVEL; NONE ASSERTS THE COMPOSITION M-1 IS ABOUT.**

§4.6's six tests exercise `check_permission` (2), the applier (1, 3, 4, 6) and `state_key_for_event` (5) —
**each in isolation.**

🔑 **But M-1's defect is not that either piece is wrong. Both of the sibling's pieces are individually
correct:** `apply_space_temperature_visibility`'s owner check *works* when called directly, and
`check_permission` *correctly* returns `Ok` for an event it does not name. **The lie is in the composition.**
⇒ **a suite that only checks the pieces reproduces the exact epistemic error that let the sibling ship**, and
would go green on a Leg C carrying F-1's hole.

**Recommended (one test, existing machinery):** a dispatch-level assertion —
`node.dispatch_event(ev, EventOrigin::LocallySubmitted, None)` returns `DispatchOutcome::Rejected` for the
non-owner case, **and (per F-1) for the owner-in-a-DM case**. The pattern is used throughout `runtime.rs`'s own
test module (e.g. `:2765`, `:2979`). 🛑 Per **F-4** it must key on `name`/`reason`, **not on `code == 4000`
alone**, which several unrelated rejects also produce.

📌 **Test-count consequence:** §4.6 predicts `1608 → 1614`. One dispatch-level test makes it **1615**; adding
the DM dispatch case as its own test, **1616**. **A prediction to restate, not a defect.**

---

### ⚠️ F-8 — NOTE. **THE NEW `SpaceError` VARIANT IS UNOBSERVABLE IN PRODUCTION.**

§4.4(b)1 requires *"a **new `SpaceError` variant**"* for the DM refusal, and §8 makes it a DoD item.
**Every production `apply_event` call site discards the error** (F-3's table: `runtime.rs:867`,
`derive.rs:231`, `ai_service.rs:553` — all `let _ =`). ⇒ **the variant is visible only to tests.**

✅ **That is fine and it should still ship** — its job is to stop the mutation, not to carry a message.
🛑 **It must simply not be read as delivering §1's *"refused in a way they can see"*.** Under F-1 that clause is
delivered by `check_permission`; under v1.0 as written, for the DM case, **it is delivered by nothing.**

---

## §3 — WHAT I CHECKED AND FOUND CLEAN

✅ **§2 citations, all verified @ `d8a44f6`, S-13 excepted (F-5):**
**S-1** `wire.rs:115` variant · **S-2** `:215` `as_str` · **S-3** `:309` `from_str` · **S-4** `known_variants()`
`:770-791`, sibling at `:786` · **S-5** `StateSpaceTemperatureVisibilityContent` at `:702` (📌 the cited span
`698-704` clips the doc comment, which opens `:696` — trivial) · **S-6** `state_key.rs:95`
`EventType::StateSpaceUpdate =>` **exact** · **S-7** `state.rs:785-796` **exact** · **S-8** `state.rs:650`
dispatch arm **exact** · **S-9** `apply_invite` at `:995`, DM bar `:996-998` · **S-10** `pub
dm_constraints_active: bool` at `state.rs:240` **exact, and `pub`** · **S-11** match opens `exchange.rs:841`,
`_ => Ok(())` at **`:914`** — **exact, including the number M-1 depends on** · **S-12** `membership.rs:126-163`,
`can_manage_federation` at **`:149-151`** **exact**.

✅ **M-2 holds.** `StateSpaceTemperatureVisibility` has **zero** hits in `resolution/state_key.rs`; the only
Space-scoped arm is `StateSpaceUpdate` (`:95`). Writing the new arm from `StateSpaceUpdate`'s shape is correct.

✅ **M-3 holds.** `dm_constraints_active` is a field (`state.rs:240`), not a method — independently confirmed.

✅ **§4.2's key shape is right.** `StateKey::new("state.space_admission", event.space_id…)` matches
`StateSpaceUpdate`'s form and delivers §6.3's *one active value per Space*.

✅ **§4.3 is right, and so is its doc-comment instruction.** `can_manage_federation` (`:149-151`) is the
Owner-only form, and `*role == Role::Owner` is the correct expression: `Role` is ordered, so `>=` would read as
a threshold where the intent is an exact identity.

✅ **§4.5 — the second constructor is genuinely non-invasive; the brief's question is answered YES.**
`build_space_create_event` (`state.rs:1415` @ `d8a44f6`) returns an **unsigned** `Event` built by
`Event::new(...)`; a sibling that adds one key to the `content` JSON needs no change to it and **no change to
any of its 140 call sites**.

✅ **`skip_membership` does not swallow the new type**, so §4.4(a) is genuinely on the local path — **finding
trigger ⑤ does not fire for local submission** (it fires in the inverse direction, as F-6).

✅ **Leg A-bis and Leg B tests are untouched by this leg's shape.** Leg C adds a variant, an arm, a table
function, an applier and a constructor; it changes no existing signature and no existing default, and the
`admission` default remains `open` via Leg B's parse. **Finding trigger ③ does not fire on inspection.**

✅ **§4.6's `56 SUITES` prediction is sound** — all six tests land in existing unit-test modules; no new
integration target is implied, and the suite count counts targets.

✅ **§8's DoD covers every §4 deliverable**, and *"commit pushed is not a DoD item"* is correctly stated.

✅ **§0's scope guard holds.** Nothing in §4 interprets the stored value; **finding trigger ⑥ does not fire**,
and `F-3`'s non-string boundary is correctly left to Leg D.

---

## §4 — WHAT I DID **NOT** VERIFY

🛑 **I ran no `cargo` command and measured no floor.** `1608 / 0 / 62 × 56 SUITES` is **carried from J-758**,
not re-driven here — this was a documents-and-source read. **Do not quote anything above as a floor
measurement.**

🛑 **I did not verify that `derive_resolved` dropping a bad admission event leaves the end state correct.**
The MP-F6 comment asserts *"resolution is a second gate"* for the banned-join case, and `derive.rs:231`'s
discard makes the same shape plausible here — **but I traced it, I did not run it.**

🛑 **I did not check ch3 §3.7.14 for a normative statement about who may change `admission`**, nor whether the
new event type needs a spec edit. **§4 and §8 name no ch3 item, and I do not know whether that is a decision or
an omission.** Flagging, not claiming.

🛑 **F-1's trace is derived from source, not observed at runtime.** Every link is measured, but no DM admission
event was dispatched — the event type does not exist yet.

---

## §5 — VERDICT

## ⚠️ **GO WITH FINDINGS.**

**The leg's shape is right and its design premise is true.** M-1 is correct — I derived it independently before
reading it. §4.1–§4.3, §4.5, the state key, the Owner-only table entry and the second constructor all survive
scrutiny, and every §2 citation but one holds at the stated tree.

**Five findings block the lock, and they are not evenly weighted.**

- **F-1 is the one that matters.** It is not a citation defect; it is the leg failing its own frozen §1 goal on
  the DM axis, **by copying the very pattern (S-9) that carries the defect M-1 names.** The fix is one `if` in
  the arm §4.4(a) already creates.
- **F-2** makes one of the two controls unsatisfiable, and would record its inevitable pass as a finding
  against the correct half of the runbook.
- **F-4** and **F-3** are wrong evidence under right conclusions — the species this arc keeps meeting — and
  **F-4 carries a live consequence**: `4000`/`generic` is not a distinguishing code.
- **F-5** is one number, and it is `D-152` clause 1 in the document written after `D-152`.
- **F-6, F-7, F-8** ride as notes. 📌 **F-7 is the note I would most want taken**, because it is the difference
  between a suite that would catch F-1 and one that would go green over it.

🛑 **I did not edit the runbook.** Chat re-drives every finding before folding it in. **Rule 6: reported, not
absorbed.**
