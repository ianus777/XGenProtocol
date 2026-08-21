# M-SPACE-ADMISSION Leg C Runbook — the mutation event: a wire type, a state key, a permission that is actually enforced, and a second constructor
> **Status**: COMPLETED  
> Version: 1.3  
> Date: Aug 2026  
> **Last updated**: 2026-08-18  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS IS, AND WHAT IS LOCKED

**Leg C of `M-SPACE-ADMISSION`.** Design: `tasks/M_SPACE_ADMISSION_PHASE0.md` **§15.3 + §15.3b** (v2.4). **Leg B shipped the field; nothing writes it yet. This leg makes it writable.**

| item | state |
|---|---|
| the design | 🔒 **LOCKED** — Phase-0 **v2.4** |
| the wire name | 🔒 `state.space_admission` — **fixed by §6.3's state key, not a fresh naming decision** |
| who may change it | 🔒 **OWNER ONLY**, via a new `can_change_admission` in the permission table (Joe, 2026-08-18, J-759) |
| creation-time setting | 🔒 **a SECOND constructor**, not a widened builder (Joe, same ruling — Clair's `F-3`) |
| the DM refusal | 🔒 §6.4(b) — an admission change in a DM is refused, **at `check_permission` AND at the applier** (Clair `F-1`), carrying **new reject code `3049 admission_immutable`** (Joe, 2026-08-18) |
| plain permission refusal | 🔒 **`RejectInfo { code: 4000, name: "generic" }`** — the unmapped-fallback band; a non-owner attempt is an ordinary permission failure and reads as one (Joe, 2026-08-18) |
| 🔒 **THE LOCK** | **LOCKED by Joe 2026-08-18.** Locked content **v1.1**; **v1.2 is the lock stamp and nothing else** — zero changes to §1–§8, verifiable by diff. **Records J-759.** 🛑 **A LOCK IS OF A VERSION, NOT OF A FILENAME (`D-152` clause 1)** — Clair implements from **v1.2** and no earlier version |

🛑 **STILL NOT DECIDED, AND THIS LEG MUST NOT ANTICIPATE IT: `F-3`'s non-string boundary is LEG D's.** `admission: 42` stores `open` and is indistinguishable from absent; `"banana"` survives. **Leg C stores and refuses; it does not interpret.**

---

## §1 — 🔒 THE GOAL. **FROZEN.**

> **A Space owner can change `admission` after creation, or set it at creation; anyone else who tries is REFUSED IN A WAY THEY CAN SEE; and a DM's pin cannot be changed at all.**

📌 **"In a way they can see" is load-bearing and is the whole reason §4.4 exists.** Refinements go in §3.

---

## §2 — THE SITES. **MEASURED AT `d8a44f6`. OPEN EACH BEFORE EDITING IT.**

| id | site | what is there |
|---|---|---|
| **S-1** | `xgen-common/src/wire.rs:115` | `StateSpaceTemperatureVisibility,` — the variant-declaration neighbourhood |
| **S-2** | `wire.rs:215` | its `as_str` arm |
| **S-3** | `wire.rs:309` | its `from_str` arm |
| **S-4** | ⚠️ `wire.rs:770-791` | **`known_variants()` — a HAND-MAINTAINED `vec!`.** The sibling sits at `:786`. 🛑 **Omission does not fail: it silently drops the variant from the round-trip test** |
| **S-5** | `wire.rs:698-704` | `StateSpaceTemperatureVisibilityContent` — **the content-struct pattern, one `String` field, doc comment naming the open enum** |
| **S-6** | `xgen-core/src/resolution/state_key.rs:95` | `StateSpaceUpdate`'s arm — 🛑 **the shape to copy, because the temperature sibling HAS NO ARM AT ALL (§3 M-2)** |
| **S-7** | `xgen-core/src/space/state.rs:785-796` | `apply_space_temperature_visibility` — the applier idiom **and the defect not to copy** |
| **S-8** | `state.rs:650` | the `apply_event` dispatch match |
| **S-9** | `state.rs:995-998` | `apply_invite`'s DM bar: `if self.dm_constraints_active { return Err(SpaceError::DmInvitationNotAllowed) }` — **the refusal pattern** |
| **S-10** | `state.rs:240` | 🔑 **`dm_constraints_active` is a FIELD on `SpaceState`, not a method** |
| **S-11** | `xgen-core/src/message/exchange.rs:841-915` | `check_permission`'s match, ending `_ => Ok(())` at **`:914`** |
| **S-12** | `xgen-core/src/space/membership.rs:126-163` | the permission table; `can_manage_federation:149-151` is the **Owner-only** form |
| **S-13** | `state.rs:1415-1423` **at `d8a44f6`** | `build_space_create_event` — the constructor the second one is modelled on. 🛑 **v1.0 CITED `:1382`, MEASURED PRE-LEG-B AND NEVER RE-DRIVEN; `:1382` IS NOW BLANK** (Clair `F-5`). ***`D-152` clause 1 broken in the first runbook written after `D-152` was minted*** |
| **S-14** | `xgen-core/src/message/exchange.rs:126-142` | `to_wire_code` — 🛑 **`_ => None` at `:140`, so `PermissionDenied` is UNMAPPED** |
| **S-15** | `xgen-core/src/node/runtime.rs:187-195` | `RejectInfo::from_exchange` — unmapped variants fall to `generic` (`code: 4000`) at `:177-179` |
| **S-16** | `docs/xgen_ch3_specification.md:2185-2194` | 🔒 **THE ERROR-CODE REGISTRY, §3.6.10.10.** `3040`…`3046` assigned; **3000–3999 is the identity domain and the 3040s the membership-authority sub-band.** `3047`/`3048`/`3049` **measured free across code AND docs** |

---

## §3 — MEASUREMENT NOTES

**M-1.** 🛑 **AN APPLIER-ONLY PERMISSION CHECK IS NOT A REFUSAL — IT IS A SILENT NO-OP.** `check_permission` ends `_ => Ok(())` (**S-11**), `StateSpaceTemperatureVisibility` **appears nowhere in it**, and the applier's error is **discarded** at every production call site. ⇒ ***today a non-owner temperature-visibility change is accepted, persisted, answered `Accepted`, and then dropped by the fold with the error thrown away.*** 🔑 **The codebase already has a name for this species: `runtime.rs:1505-1522` calls it *the reply lied*.** 🛑 **Leg C does not copy it.**

🛑 **`M-1`'s EVIDENCE, CORRECTED (Clair `F-3`): v1.0 cited `exchange.rs:1279` and `:2319` as production call sites and BOTH ARE INSIDE `#[cfg(test)]`, which opens at `exchange.rs:1096`.** The real set at `d8a44f6` is **`runtime.rs:867` · `derive.rs:231` · `ai_service.rs:553`**, all `let _ = …apply_event(…)`. **The conclusion is unchanged; the evidence was wrong, and an implementer checking it would have landed in a test module.**

**M-2.** `state_key_for_event` has **no arm for `StateSpaceTemperatureVisibility`**; the only Space-scoped arm is `StateSpaceUpdate` (**S-6**). ⇒ **admission is per-field conflict-resolved and the sibling is not.** Correct under §6.3, but **the arm is written from `StateSpaceUpdate`'s shape, not copied from the twin.**

**M-3.** Chat's first sweep for `fn dm_constraints_active` returned **nothing** and nearly entered the record as a phantom. **It is a FIELD (`state.rs:240`), not a method** — the pattern was narrower than its subject. 📌 **`D-152` clause 2, caught by re-sweeping without the `fn`.**

**M-4.** 🛑 **CORRECTED (Clair `F-4`) — v1.0 INVERTED ITS OWN SOURCE.** `to_wire_code` returns **`None`** for `PermissionDenied` (**S-14**, `_ => None` at `:140`); the reject reaches the sender through `from_exchange`'s **generic fallback** (**S-15**) as **`RejectInfo { code: 4000, name: "generic", reason: <Display> }`**. v1.0 read the source comment's *"4000-unmapped"* as *"wire 4000"*. ✅ **The sender does see a reject** — ⚠️ **but by a code shared with every unmapped variant, including signature failures. Only the reason string distinguishes them.**

**M-5.** 🔒 **REJECT CODES, MEASURED AGAINST THE REGISTRY (S-16) AND RULED (Joe, 2026-08-18):** the **DM refusal** takes a new **`3049 admission_immutable`**; the **plain permission refusal** stays on **`4000 generic`**. 🔑 **The split is deliberate: a client can ACT on *"this is a DM, its admission is fixed"* and cannot act on a parsed English string, whereas *"you lack permission"* reads correctly as an ordinary permission failure.**

**M-6.** 🛑 **AN INSTRUMENT RECONCILIATION THAT WENT THE WRONG WAY FIRST, RECORDED BECAUSE IT NEARLY COST A CODE.** A sweep matching `to_wire_code` / `RejectInfo::coded` returned six assigned codes and **missed `3044`/`3045` entirely** — which are spec-registered (**S-16**) and used 48 and 31 times, assigned on a construction path that pattern does not see. **A broad sweep of bare numerals was the COMPLETE instrument; the narrow one looked more rigorous and saw less.** 🔑 ***It was trusted over the broad sweep because it matched a mechanism already in mind*** — `D-152` clause 2, and the correction ran in both directions inside one conversation.

**M-7.** 🛑 **`known_variants()` HAS NO COMPLETENESS CHECK (Clair `F-2`).** All three consumers — `wire.rs:806`, `:818`, `:864` — **iterate the vec**; none asserts a count or cross-checks a canonical source. ⇒ **deleting a variant makes every loop skip it and all three tests pass.** ✅ **§4.1's claim is therefore TRUE and v1.0's `V-4` was unable to demonstrate it** — see §4.7 and the rebuilt `V-4`.

---

## §4 — THE CHANGE

### §4.1 — the wire type (`xgen-common/src/wire.rs`)

`StateSpaceAdmission` — variant beside **S-1**; `as_str` ⇒ `"state.space_admission"` beside **S-2**; `from_str` beside **S-3**; 🛑 **and ADD IT TO `known_variants()` at S-4** — *the one edit whose omission passes.*

`StateSpaceAdmissionContent { pub admission: String }` on **S-5**'s pattern, doc comment naming the open enum, the permitted values, and 🔑 **that unrecognised values are stored verbatim and interpreted at the GATE (`D-149`), not here.**

### §4.2 — the state key (`state_key.rs`)

An arm on **S-6**'s shape returning `StateKey::new("state.space_admission", space_id)` — 🔒 **§6.3's ruled key. One active value per Space.**

### §4.3 — the permission (`membership.rs`)

```rust
pub fn can_change_admission(role: &Role) -> bool {
    *role == Role::Owner
}
```
On **S-12**'s `can_manage_federation` form, with a doc comment saying **why Owner and not Admin**: widening later is additive, narrowing later removes a capability.

### §4.4 — 🛑 THE ENFORCEMENT, IN BOTH PLACES, AND THIS IS THE LEG'S POINT

**(a) `check_permission` (S-11)** — a `StateSpaceAdmission` arm **before** the `_ => Ok(())` catch-all, on `MembershipInvite`'s exact shape (`exchange.rs:866-875`). 🛑 **AND THE DM CHECK GOES HERE TOO (Clair `F-1`), NOT ONLY IN THE APPLIER:**

1. **`if space.dm_constraints_active` ⇒ refuse with `3049 admission_immutable`** — checked **first**, because a DM's pin binds the owner as well;
2. otherwise `can_change_admission(role)` ⇒ `ExchangeError::PermissionDenied` ⇒ **`4000 generic`** (M-4, M-5).

✅ **Zero cost: `check_permission` already takes `&SpaceState` (`exchange.rs:807`) and `dm_constraints_active` is a `pub` field (`state.rs:240`) — one `if` in an arm this leg creates anyway.**

🛑 **WHY THIS IS THE FINDING OF THE LEG.** v1.0 put the role check in both places and the DM check **in the applier only** ⇒ an **owner** sending an admission change on a DM would pass `can_change_admission`, be persisted, be refused by the applier, have that error **discarded**, and be answered **`Accepted`**. ***§1's second clause — "a DM's pin cannot be changed at all" — would have been delivered by nothing, and the runbook would have reproduced `M-1` on a second axis while claiming to fix it.***

**(b) the applier (S-7, S-8)** — `apply_space_admission`, dispatched from **S-8**, which:
1. **refuses when `dm_constraints_active`** (**S-9**'s pattern, **S-10**: a field) with a **new `SpaceError` variant**;
2. **re-checks `can_change_admission`**;
3. stores the value **verbatim**.

🛑 **(b)'s JUSTIFICATION — CORRECTED AT v1.3 (Clair `F-A`), AND v1.2's CITATION WAS FALSE.** v1.2 said the federated path reaches `check_permission` via **`runtime.rs:1426`**. ⚠️ **That call sits INSIDE `if matches!(event.event_type, StateAiOperatorDelegate | StateAiOperatorRevoke)` opened at `:1416` — `state.space_admission` can NEVER reach it.** ✅ **The claim is nonetheless TRUE by a different route: `validate_event` step 13 calls `check_permission(event, space)?` at `exchange.rs:256`, on every path that validates, plus the dispatch-side call at `:752`.** ⇒ **(b) remains defence-in-depth for REPLAY**, which is the only path that bypasses validation. 🔑 ***A conclusion can be right while its citation is false, and a reader who checks the citation finds a guard that does not apply and reasonably concludes the opposite.***

🛑 **(a) WITHOUT (b) LEAVES REPLAY UNGUARDED. (b) WITHOUT (a) IS THE SILENT NO-OP `M-1` DESCRIBES. BOTH, OR THE LEG HAS NOT DONE ITS JOB.**

⚠️ **AND S-9 IS A MODEL FOR THE SHAPE, NOT FOR THE PLACEMENT (Clair `F-1`): `apply_invite`'s DM bar is applier-only too, so an invite in a DM today is accepted, persisted, answered `Accepted` and silently dropped — the THIRD live instance of `M-1`'s species, and v1.0 offered it as "the refusal pattern" to copy.** 📌 **Not fixed here; filed separately.**

### §4.5 — the second constructor (`state.rs`, S-13)

`build_space_create_event_with_admission(...)` taking the same arguments plus `admission: &str`, emitting it into content. 🛑 **`build_space_create_event` IS NOT TOUCHED — zero of its 140 call sites move.** ✅ **Clair verified the mechanism: the builder returns an UNSIGNED `Event`, so nothing downstream shifts.** 🔑 **This closes a RACE, not typing:** without it an invite-only Space is `open` between create and the mutation event, ***a federated window admitting exactly the strangers this milestone exists to refuse.***

### §4.6 — 🔒 THE `known_variants()` COMPLETENESS TEST (Clair `F-2`)

**A NEW PERMANENT TEST in `wire.rs`:** `assert_eq!(known_variants().len(), N)` with the count stated and a comment saying **bump this deliberately when adding a variant.**

🔑 **WHY IT EXISTS, AND WHY v1.0's `V-4` WAS INCOHERENT.** §4.1 claims that omitting a variant from **S-4** *silently drops the round-trip case*. **That claim is TRUE (M-7) — and v1.0's `V-4` proposed to demonstrate it by deleting the variant and expecting a FAILURE, which cannot happen**: all three consumers iterate the vec, so a deletion is invisible to every one of them. ***v1.0 read a pass as "S-4 is decorative", which is backwards — a pass proves the omission is INVISIBLE, which is exactly what §4.1 said.*** ⇒ **the count assertion converts the silent hole into a caught one, for every future leg**, and `V-4` is rebuilt against it.

### §4.7 — 🔒 THE SPEC ROW (S-16)

`docs/xgen_ch3_specification.md` §3.6.10.10 gains **one registry row**: `| 3049 | admission_immutable | An admission change targeting a DM Space, whose admission is pinned at creation (L-C) |`. 📌 **Chat writes this row; it is records surface, not implementation, and it ships in the same `D-074` commit.**

### §4.8 — the tests. **SEVEN.**

1. owner changes `admission` ⇒ stored.
2. 🔒 **non-owner (Admin) is refused at `check_permission`** with `PermissionDenied` ⇒ `4000 generic`.
3. non-owner is **also** refused by the applier directly ⇒ (b) proven independently of (a).
4. 🔒 **an admission change in a DM is refused AT `check_permission` with `3049`, EVEN FROM THE OWNER** (Clair `F-1`).
5. `state_key_for_event` returns `state.space_admission:{space_id}` — and **two changes to one Space share one key**.
6. `build_space_create_event_with_admission("invite")` ⇒ `from_space_create` yields `invite`.
7. 🔒 **END-TO-END THROUGH `dispatch_event` (Clair `F-7`): a non-owner's admission change returns a REJECTED outcome to the sender — asserted on the OUTCOME, not on `check_permission`'s return.**

🔑 **§4.6 item 7 exists because `M-1`'s defect is a COMPOSITION failure, and six unit tests would have gone GREEN over it.** Each piece can be individually correct while the sender still receives `Accepted`. **`xgen-node/src/tests/` (42 files) is the home; Leg A-bis's `phase9_harness` `InProcessNode` is the model.** 📌 **§4.6's count test is not in this seven — it is a `wire.rs` unit test and lands with `V-4`.**

🛑 **SEVEN TESTS PLUS §4.6's COUNT TEST ⇒ `cargo` 1608 → 1616.** **56 SUITES stays 56.** 📌 **A prediction; §5 decides.** ⚠️ **`1608` is CARRIED from J-758**, where both sides were measured.

📌 **`F-8`, NOTED NOT FIXED: the new `SpaceError` variant is observable only from tests** — (a) refuses first on every live path, so (b)'s variant is reachable in production only via replay. **That is what defence-in-depth means here, and it is stated so no one later reads test-only reachability as dead code.**

---

## §5 — VERIFICATION. **CHAT RE-DRIVES ALL OF THESE (Rule 5).**

| gate | how | expected |
|---|---|---|
| **V-1** | `cargo test --workspace`, **detached**, own exit sentinel, final `test result:` required, summed programmatically | **1616 / 0 / 62**, all seven named **by exact name** plus §4.6's count test. 📌 `1608` carried from J-758 |
| **V-2** | suite count | **56 SUITES** — structural; a change is a §6 FINDING |
| **V-3** | 🔒 **NEGATIVE CONTROL A**, discarded, never committed: delete the `check_permission` arm | **tests 2, 4 AND 7 must FAIL.** 🛑 If any passes, the arm was never load-bearing on that path and `M-1`'s defect has been reproduced. ⚠️ **Test 7 is the one that matters — it is the only one that asserts what the SENDER receives** |
| **V-3b** | 🔒 **NEGATIVE CONTROL C** (Clair `F-1`), discarded: delete **only the DM branch** from §4.4(a), leaving the applier's | **test 4 must FAIL.** 🛑 **If it passes, the DM refusal is applier-only and the leg has reproduced `M-1` on the DM axis — exactly what v1.0 would have shipped** |
| **V-4** | 🔒 **NEGATIVE CONTROL B, REBUILT (Clair `F-2`)**, discarded: delete the variant from `known_variants()` (**S-4**) | **§4.6's COUNT TEST must FAIL, and the three iterating consumers must all still PASS.** 🔑 ***Both halves are the assertion: the count test catches it, and the sweeps demonstrably do not.*** 🛑 **v1.0's V-4 expected the sweeps to fail and they cannot — see M-7** |
| **V-5** | `git diff --numstat` **AND `git ls-files --others --exclude-standard`** | 🛑 **both instruments — `--numstat` alone does not see untracked files (`D-152` clause 2)** |
| **V-6** | `git ls-files --eol` | **`i/lf`** on every touched file; `bareLF=0` |
| **V-7** | grep the diff | **no interpretation of the value** — no `match` on `"open"`/`"invite"`, no allow-list. **Leg C stores and refuses; Leg D reads** |

---

## §6 — FINDING TRIGGERS. **REPORT, NEVER ABSORB.**

① a `V-3`/`V-4` control passes · ② suite count moves · ③ any Leg A-bis or Leg B test goes red · ④ a §2 citation does not hold at the leg's parent — 🛑 **cite the tree (`D-152` clause 1)** · ⑤ `check_permission` turns out to be bypassed on a path §4.4(a) assumed it covered · ⑥ the second constructor cannot emit content without touching the first.

---

## §7 — ORDERING

**§7.1.** 🛑 **BEFORE reading §4, open `check_permission` (`exchange.rs:807-916`) and `apply_space_temperature_visibility` (`state.rs:785-796`) and write down what happens TODAY when a non-owner sends the sibling's event.** Then compare with **M-1**. *If your answer differs, that is a finding about this runbook and it outranks the runbook.*
**§7.2.** §1 is frozen. §4.8's counts are predictions; §5 decides.

---

## §8 — DoD

- [ ] Variant, `as_str`, `from_str`, **`known_variants()`**, **and §4.6's count assertion**
- [ ] Content struct on **S-5**'s pattern
- [ ] `state_key_for_event` arm, §6.3's key
- [ ] `can_change_admission` in the **table**, Owner-only, doc-commented
- [ ] **Enforcement in BOTH `check_permission` and the applier — INCLUDING the DM branch in `check_permission`**
- [ ] DM refusal carrying **`3049 admission_immutable`**; its own `SpaceError` variant in the applier
- [ ] **§4.7's spec row in ch3 §3.6.10.10**
- [ ] `build_space_create_event_with_admission`; **the original builder untouched**
- [ ] Seven tests **including the end-to-end one through `dispatch_event`**; V-1 **1616 / 0 / 62 × 56 SUITES** measured by Chat
- [ ] **V-3, V-3b and V-4 all run, all controls behaved as specified, all reverted** — `git status` clean
- [ ] Records: JOURNAL + `CLAUDE.md` + ROADMAP + `Status: COMPLETED`, **one `D-074` commit**

📌 **"Commit pushed" is not a DoD item.**

---

## §9 — ✅ CLOSE RECORD. **EVERY GATE RE-DRIVEN BY CHAT (Rule 5). J-760, 2026-08-18.**

**Implemented by Clair from v1.2. Seven tracked files +605/−6, plus one new test file (241 lines) and the hand-back.** Zero `xgen-client`, zero `ui/**`.

| gate | measured by Chat | verdict |
|---|---|---|
| **V-1** | detached, own exit sentinel `=0`, summed programmatically over 56 `^test result:` lines | ✅ **1616 / 0 / 62 — exactly the predicted +8**; `Compiling xgen-core` present ⇒ not cached; `FAILED`/`error[`/`panicked`/`warning` **all zero, case-sensitive**; **all eight confirmed BY EXACT NAME** |
| **V-2** | same run | ✅ **56 SUITES**, structural |
| **V-3** | 🔒 Chat's own script: arm neutralised (both branches), **`xgen-core` and `xgen-node` run SEPARATELY** (Clair `F-G`) | ✅ tests 2 and 4 FAILED · 🔑 **and test 7 returned `Accepted { new_joiner: None, additional_persisted: [] }` — `M-1` REPRODUCED LIVE**, with the assertion message naming it: ***the reply lied.*** **Test 3, the applier test, stayed GREEN throughout** ⇒ `F-7` vindicated concretely |
| **V-3b** | only the DM branch neutralised | ✅ **test 4 FAILED and NOTHING ELSE DID** — `non_owner…` still `ok` ⇒ the DM branch is independently load-bearing |
| **V-4** | the `known_variants()` vec entry removed | ✅ **`known_variants_is_complete` FAILED**, and `as_str_and_from_str_are_inverse_for_known` + `known_type_round_trips_byte_identically` **both PASSED** ⇒ ***both halves: the count test catches it and the sweeps demonstrably do not*** |
| **V-5** | `--numstat` **AND** `ls-files --others` | ✅ seven tracked + two untracked, all in `xgen-common`/`xgen-core`/`xgen-node`/`tasks` |
| **V-6** | `git ls-files --eol` | ✅ `i/lf` ×7; the new test file is LF-only, as Leg A-bis's was |
| **V-7** | every `+` line mentioning admission | ✅ **no `match` on the value, no `== "open"/"invite"`, no allow-list** |

📌 **Every control restored SHA256-identical**; `git status` clean of control residue at close.

⚠️ **TWO SCRIPT FAULTS OF CHAT'S, BOTH CAUGHT BY THEIR OWN GUARDS, BOTH WORTH THE LINE:** ① the first V-3 script used **LF here-string anchors against a CRLF file** ⇒ `.Replace` matched nothing; the *mutation-is-a-no-op* assertion threw and the `finally` restored. ② the second used **arm+7 where the target is arm+8** ⇒ the content assertion named the line it actually found and refused. 🔑 ***Without those two guards each script would have run three controls against UNMUTATED source and reported clean passes*** — the exact failure `N-124b` exists to prevent, twice, in one session.

📌 **`F-B` IS NOT A FINDING — §4.7's spec row landed in `a0ccf3a`, which is THIS ARC'S OWN J-759 COMMIT**, pushed at the start of the implementation session. §4.7 said *"Chat writes this row"* and Chat did; Clair correctly reported not having written it.

✅ **`F-C` AND `F-G` ARE DEPARTURES THAT IMPROVED ON THE LETTER, BOTH UPHELD:** §4.8's seven left the applier's DM branch untested, so **both of §4.4(b)'s branches went into test 3** rather than a ninth test; and **`V-3` as written left test 7 unobserved**, because cargo halts after the first failing suite — ***the leg's single most important assertion, invisible to its own control.*** Chat's re-drive adopted the separated form.

📌 **FILED, NOT FIXED:** `F-D` the count assertion is satisfiable by a duplicate · `F-E` `build_membership_event` emits `prev_events: vec![]` and is unusable on the node ingest path · `F-F` three instruments give three counts for `known_variants()`.

🛑 **WHAT WAS NOT DONE, STATED PLAINLY:** nothing ran against a running Node, a wire, or a second identity. **The REPLAY path — the entire justification for §4.4(b) — is not tested.** **`3049` was never observed on a wire.** vitest, svelte-check and the catalogue are carried by scope, not measured.
