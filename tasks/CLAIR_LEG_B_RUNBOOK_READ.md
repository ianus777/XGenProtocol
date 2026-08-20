# CLAIR — adversarial cold read of `tasks/RUNBOOK_SPACE_ADMISSION_LEG_B.md` v1.0
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-19  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS IS, AND THE VERDICT

An adversarial cold read of the **Leg B runbook v1.0** (PENDING, not locked) against the source at **`3876950`**. **No product code was written.** The runbook was not edited — Chat folds findings in after re-driving each independently.

✅ **STATE, MEASURED AT OPEN:** `git rev-parse HEAD` = **`3876950c2f00f2a00b14745fff1fbbd0e3cd1c2a`**; `git ls-remote origin refs/heads/main` = **the same hash**. Tree dirty exactly as the kickoff described — `tasks/M_SPACE_ADMISSION_PHASE0.md` modified (v2.0 → v2.1), `tasks/RUNBOOK_SPACE_ADMISSION_LEG_B.md` untracked. Both read from disk.

### 🎯 VERDICT: **GO WITH FINDINGS.**

**The BUILD is sound.** The field, the parse idiom, the DM pin, the four tests and V-3's negative control are all correct, and §4.4's design is better than the code it sits beside. **Every defect below is in the runbook's REASONS, its SITE LIST, or its MECHANISM — not in what it asks for.**

**Three findings block the lock** (F-1, F-2, F-3). All three are text-only fixes; none changes a line of the intended implementation.

| # | finding | blocks lock? |
|---|---|---|
| **F-1** | §15.1's "OPEN" item is `D-149`, **locked 2026-08-16**. §0 instructs Clair on a premise four records contradict | 🛑 **YES** |
| **F-2** | A **fourth file** must change and no section names it: `xgen-core/src/wire/types.rs:14-22` | 🛑 **YES** |
| **F-3** | §4.4's tests 2/3/4 need content injection; **neither builder accepts it**, both nearest precedents fail this way, and tests 3/4 **pass while proving nothing** if done wrong | 🛑 **YES** |
| **F-4** | §8.2 cites a **`§4.6` that does not exist** | note |
| **F-5** | **`J-757` does not exist**; two documents disagree about what produced Phase-0 v2.1 | note |
| **F-6** | §4.1 files the constants under the **Temperature** banner and cites task-file locks where every neighbour cites a **spec §** | note |
| **F-7** | The **1604 baseline is carried, not measured**, and the runbook does not say so | note |

---

## §1 — MY DERIVATION, WRITTEN BEFORE JUDGING §4 OR §5 (§8.1's ordering, honoured)

Derived from `xgen-core/src/space/state.rs` and `xgen-common/src/wire.rs` at `3876950`, before reading §4 as an instruction.

**Leg B must touch FOUR files:**

| # | file | why |
|---|---|---|
| 1 | `xgen-common/src/wire.rs` | declare `ADMISSION_OPEN` / `ADMISSION_INVITE` / `DEFAULT_ADMISSION` beside the three sibling defaults |
| 2 | 🛑 **`xgen-core/src/wire/types.rs:14-22`** | **the `pub use xgen_common::wire::{…}` list is EXPLICIT AND HAND-MAINTAINED, not a glob.** Without adding the three names here, `crate::wire::types::DEFAULT_ADMISSION` **does not exist** |
| 3 | `xgen-core/src/space/state.rs` | the `use crate::wire::types::{…}` list (`:32-36`) · the field after `threads:257` · the parse in `from_space_create` · the pin in both DM constructors · four tests |
| 4 | `xgen-core/src/resolution/algorithm.rs:414` | `simple_space_state`, a `#[cfg(test)]` helper (`#[cfg(test)]` opens `:329`), `is_dm: false` ⇒ takes the default |

**My literal census — the honest partition, four `SpaceState` struct literals workspace-wide:**

`state.rs:312` · `state.rs:443` · `state.rs:559` · **`resolution/algorithm.rs:414`**

✅ **Exactly one lies outside §2's list, and it is inside `xgen-core`** ⇒ **§7 trigger ① does NOT fire** and **V-4's "ZERO `xgen-node`, ZERO `xgen-client`" HOLDS.** ✅ No `Self { … }` construction inside `impl SpaceState` (`:260-1299`), no multi-line literal form, no destructuring pattern, no `impl Default for SpaceState`.

**Where my derivation and §4 agree:** the `unwrap_or_else` idiom, the insertion anchors, the unconditional DM pin, the absence of validation, and the four tests. **Where they differ:** files 2 and 4 above, and the mechanism for getting content into a create event (F-2, F-3).

---

## §2 — FINDINGS

### 🛑 F-1 — §15.1's "OPEN" ITEM IS `D-149`, LOCKED THREE SESSIONS AGO. THE RUNBOOK INSTRUCTS ON A PREMISE FOUR RECORDS CONTRADICT. **BLOCKS THE LOCK.**

**Runbook §0, in a 🛑 block:**

> *"§15.1's unrecognised-value semantics and §15.3's who may change `admission` are **Joe's and unruled** … write no validation … A constructor that rejects an unrecognised value would be implementing an unruled decision."*

🛑 **The first item is ruled. Four records say so, and one of them is the Phase-0 this runbook cites as its locked design.**

| record | says |
|---|---|
| `DECISIONS.md` **`D-149`** (2026-08-16, milestone `M-SPACE-ADMISSION`, spec ref ch3 §3.7.14.2) | *"a field that GATES an action fails **CLOSED**"* — and admission is a gate |
| **Phase-0 §6.2 heading**, marked 🔒 | *"Q2 RULED (Joe, 2026-08-16) … absent ⇒ `open`, **present-and-unknown ⇒ `invite`**"* |
| **Phase-0 §6 heading** | *"**ALL EIGHT RULED** … **NOTHING HERE IS OPEN.**"* |
| **ch3 §3.7.14.2** (normative, `:2841`) | *"**Present but unrecognised means `invite`.** … an admission rule is a gate rather than a display preference"* |

**And Phase-0 §6.8 names `D-149` by number as the decision that binds *"absent ⇒ open, present-and-unknown ⇒ invite"*.**

⇒ **Phase-0 §15.1 re-opens, as a fresh 🔓 question with a fresh recommendation, a decision its own §6.2 marks 🔒 RULED and its own §6.8 cites by D-number.** §15.8 repeats it. The runbook inherits it into an instruction.

🔑 **The recommendation in §15.1 is IDENTICAL to the ruling — which is exactly why nobody caught it.** Reading the recommendation alone gives no signal; the only way to see it is to ask whether it had already been ruled. *This is J-513's species (a resolved decision that keeps advertising itself as open), and J-750's discriminator applies: **the answer is on disk, so it is a measurement and it is Chat's, not Joe's.***

✅ **WHAT SURVIVES — AND THE CODE IS UNAFFECTED.** The ruling is a **use-time** interpretation, not a parse-time normalisation. Three independent grounds:

1. **`D-149`'s own two precedents both interpret at USE, never at parse** — `should_include_member_temperature` matches at enforcement (`state.rs:1779`, `VISIBILITY_MODERATOR | _ =>`), and the invite-expiry gate is `.unwrap_or(true); // unparseable ⇒ fail-closed` at `runtime.rs:1591`. **Both store the value verbatim.**
2. **ch3 §3.7.14.2 says the ABSENCE rule is *"resolved at parse time"* and conspicuously says no such thing of the unrecognised rule.**
3. **Phase-0 §15.4's gate predicate reads *"the Space **resolves** to `open`"*** — resolution at the gate.

⇒ **§4.3 storing verbatim is CORRECT, and §0's "write no validation" is CORRECT.** 🛑 **Only the stated reason is false** — and *an instruction kept for a false reason is one that will be discarded the first time someone checks it* (J-750's F2, verbatim). **The kickoff routes the implementer to `D-149`**, so this will be checked.

**What the fix must say instead:** the item is **ruled** (`D-149`, ch3 §3.7.14.2); admission is a **gate** and an unrecognised value is read as `invite` **at the gate (Leg D)**; Leg B **stores verbatim so the gate can apply it**. Same code, true reason.

📌 **A consequence for §4.4 test 2.** The runbook frames the `"banana"` assertion as a tripwire *"if a future leg adds [validation]"* — i.e. as a countdown against an unruled future. Under `D-149` it is a **permanent invariant**: the storage layer must stay verbatim *because* the gate needs the raw value. **The test is right; its comment would be wrong**, and a comment asserting a limit that has been lifted is a live false record (`N-109`).

📌 **§15.3's item is the same species and is Leg C's, not Leg B's:** **ch3 §3.7.14.4** states *"`state.space_admission` is permitted from the Space **owner role** … a role-predicate test … not an equality test against a stored owner Identity"* — which answers both the authority question **and** Rider 2. Named here so Leg C does not re-open it either.

---

### 🛑 F-2 — A FOURTH FILE MUST CHANGE AND NO SECTION NAMES IT. **BLOCKS THE LOCK.**

**`xgen-core/src/wire/types.rs:14-22` is an EXPLICIT, HAND-MAINTAINED `pub use` list — not a glob:**

```rust
pub use xgen_common::wire::{
    clamp_temperature, AiCapabilities, Event, EventType, …
    DEFAULT_AI_PACING_MS, DEFAULT_HUMAN_PACING_MS,
    DEFAULT_MEMBER_TEMPERATURE_VISIBILITY, … VISIBILITY_SELF_ONLY,
};
```

And **`state.rs:32-36` reaches its constants through that re-export**, not from `xgen_common` directly:

```rust
use crate::{ … wire::{ types::{ Event, EventType, ThreadStatus,
    DEFAULT_AI_PACING_MS, DEFAULT_HUMAN_PACING_MS,
    DEFAULT_MEMBER_TEMPERATURE_VISIBILITY, VISIBILITY_EVERYONE, … }}};
```

⇒ **adding three constants to `xgen-common/src/wire.rs` does NOT make them reachable at `crate::wire::types::DEFAULT_ADMISSION`.** Two hand-maintained named lists must gain them. §2 names neither; §4.1 and §4.2 name neither; §5's V-4 omits `types.rs` from the expected diff.

⚠️ **It fails LOUDLY (unresolved import) — but the failure mode is not the risk. The risk is the fix.** The tempting local repair is `use xgen_common::wire::DEFAULT_ADMISSION;` directly in `state.rs`, which compiles and **diverges from the convention `types.rs`'s own doc comment states**: *"Re-exported here so all internal code using `crate::wire::types::{…}` continues to compile without change (Fix 17)."* That would leave `admission` the only wire constant in `state.rs` reached by a second path — a small `D-067` surface — **and Leg C's applier and Leg D's gate inherit whichever path Leg B establishes.**

📌 **`algorithm.rs:414` confirms the re-export is the live convention across the crate** — it reads `crate::wire::types::DEFAULT_HUMAN_PACING_MS`, not `xgen_common::`.

**Fix:** add `types.rs` to §2 as **S-0**, add the re-export as the first step of §4.1 (constants and their re-export are one act), and add it to V-4's expected file set.

---

### 🛑 F-3 — TESTS 2/3/4 NEED CONTENT INJECTION; NEITHER BUILDER ACCEPTS IT, AND THE TWO NEAREST PRECEDENTS BOTH FAIL THE EXACT WAY §4.4 WARNS ABOUT. **BLOCKS THE LOCK.**

**Measured:** neither builder has an `admission` parameter and neither takes arbitrary content.

- `build_space_create_event(key, name, topic, auth_tier, home_node, jurisdiction, e2e_encryption)` — fixed `json!`, conditional keys for `topic` / `jurisdiction` / `e2e_encryption` only.
- `build_dm_space_create_event(key, invitee, home_node)` — fixed `json!` of `auth_tier` / `invitee` / `nonce` / `home_node`. **No injection point at all.**

🛑 **AND THE FILE'S TWO NEAREST DM PRECEDENTS BOTH USE THE BARE BUILDER — ONE ADMITTING IN ITS OWN COMMENT THAT IT CANNOT DISCRIMINATE:**

- `dm_space_create_declares_no_jurisdiction` (`state.rs:2038-2049`)
- `dm_space_create_e2e_uniform_default_off` (`state.rs:2073-2086`) — *"No caller declares E2E for a DM today, so both DM views default OFF — **but the field is read, not hard-set**."*

🔑 **That second test cannot distinguish read-and-defaulted from hard-set, because its content never carries the key. It is `N-197`'s species, live, three lines from where Leg B's tests go — and it is the natural copy target.**

⇒ **§4.4's 🔑 note (*"The content value must be the WRONG one, or the test cannot tell pinning from parsing"*) is exactly right and is the runbook's best line. But §4.4 names the trap and supplies no mechanism**, while §4.2/§4.3 repeatedly instruct *copy the idiom / copy the neighbour* — and the neighbour is defective.

**Failure modes differ per test, and that matters:**

| test | if the mechanism is missed |
|---|---|
| 2 (`present_admission_is_stored_verbatim`) | **cannot be written at all** ⇒ discovered loudly |
| **3 / 4 (DM pins ignoring content)** | 🛑 **the bare builder omits the key, the constructor pins `invite`, the assertion PASSES — and proves nothing.** Green, and worthless |

**Two routes exist and the runbook picks neither:**

- **(A) widen the builders** — this is the file's own convention for set-once create fields (`jurisdiction` and `e2e_encryption` were each added as a builder parameter). 🛑 **Measured cost: 139 `build_space_create_event` callers + 26 `build_dm_space_create_event` callers = 165 sites across `xgen-core` (85), `xgen-node` (61), `xgen-client` (17), `xgen-mptest` (2)** ⇒ **it detonates V-4's "ZERO `xgen-node`, ZERO `xgen-client`".**
- **(B) mutate the unsigned builder output, then sign** — zero blast radius:
  ```rust
  let mut ev = build_dm_space_create_event(&alice, &bob_id, HOME);
  ev.content["admission"] = json!("open");
  let create_ev = sign_event(ev, &alice);
  ```
  ✅ `Event.content` is `pub content: Value` (`wire.rs:487`); `sign_event(mut event: Event, key)` (`state.rs:1304`) takes ownership.

🛑 **The runbook already knows the fact that decides this — and it is filed in the wrong section.** §6 item 3 (*"WHAT THIS LEG DOES NOT CLAIM"*) says *"there is no builder parameter … A create event carrying `admission` can only be hand-built."* **That sentence is what makes §4.4 non-trivial, and §4.4 does not cite it.** An implementer writing tests from §4 need never reach §6.

⚠️ **AND THE ORDERING PRECEDENT IS INVERTED.** The only content mutation in `state.rs`'s test module is `:2791` — inside `tampered_event_fails_verification`, which mutates **AFTER** signing **in order to produce an invalid event**. `sign_event` derives `event_id` from the canonical bytes of the whole event **including content** (`:1304-1311`), so copying that ordering yields an event whose id does not match its content. 🛑 **Neither DM constructor verifies signatures**, so such a fixture is **silently accepted** and the test still passes. **The syntax precedent exists; the ordering precedent is the opposite of what is needed.**

**Fix:** §4.4 names route (B) explicitly, shows the three-line form, states *mutate before signing* and why, and cross-references §6 item 3. **Route (A) should be named and refused in the runbook**, so nobody reaches for the file's own set-once convention and produces a 165-site diff that fails the runbook's own scope gate.

---

### ⚠️ F-4 — §8.2 CITES A `§4.6` THAT DOES NOT EXIST

> **§8.2.** … *"§4.6-style counts (four tests, 1608, 56 suites) are predictions"*

**§4 has §4.1, §4.2, §4.3, §4.4 — and nothing else.** A phantom section reference: **`N-198`'s species**, one section below the runbook's own finding-trigger list, and in the same arc where J-756 caught the identical defect in Phase-0 §15.5 (a §6.9 that does not exist). ✅ **The clause's substance is right and is load-bearing** — it is the only place reconciling §4.4's 🛑 assertion (*"⇒ `cargo` MOVES 1604 → 1608"*) with §5's authority. **Re-point to §4.4.**

---

### ⚠️ F-5 — `J-757` DOES NOT EXIST, AND TWO DOCUMENTS DISAGREE ABOUT WHAT PRODUCED PHASE-0 v2.1

**Measured:** max journal entry is **`## Entry J-756`**. The string `J-757` occurs **once in the repository** — `tasks/M_SPACE_ADMISSION_PHASE0.md:411`, *"🛑 CORRECTED AT v2.1 (J-757)"*.

📌 **Not yet a breach** — the Phase-0 edit is uncommitted, so nothing has been pushed citing an unresolvable number. 🛑 **It becomes `d6b7f77`'s defect exactly (J-753) if it lands in a commit that does not create J-757** — and this time there **is** a live downstream reader, because the runbook Clair implements from cites Phase-0 v2.1 as its locked design.

⚠️ **And the two documents disagree:** runbook §0 says *"v2.1, **locked at J-756**"*; Phase-0 §15.1 says *"CORRECTED AT v2.1 (**J-757**)"*. v2.1 cannot both be locked at J-756 and corrected at J-757.

📌 **A third, smaller staleness in the same section:** §15's heading still reads *"EVERY SITE MEASURED AT **`b3ccb77`**"*, while §15.1's corrected row was measured **while grounding Leg B**, at `3876950`. The blanket provenance claim no longer covers every row of its own section.

---

### ⚠️ F-6 — THE CONSTANTS LAND UNDER THE WRONG BANNER AND CITE THE WRONG AUTHORITY

**Banner placement.** `wire.rs` is banner-organised: `// ── Pacing rules (spec 3.7.12) ──` at `:597`, `// ── Temperature property (spec 3.7.13) ──` at `:615`, **and no banner between `:615` and `:641`.** §4.1 inserts *"immediately after `DEFAULT_MEMBER_TEMPERATURE_VISIBILITY` (`:641`)"* ⇒ **the three admission constants land inside the Temperature section**, between the visibility default and `clamp_temperature`. ✅ The *pattern* copied is right; the *section* is not. **A new `// ── Space admission (spec 3.7.14) ──` banner costs one line.**

**Citation.** §4.1 says the doc comments cite **`L-E` and `L-C`** — Phase-0 lock labels. 🛑 **Every constant in this region cites a spec §** and **none cites a task-file lock**: `DEFAULT_HUMAN_PACING_MS` / `DEFAULT_AI_PACING_MS` *(spec 3.7.12.2)* · `META_ATT_ROOM_TEMPERATURE` / `META_ATT_MEMBER_TEMPERATURE` / `VISIBILITY_*` / `DEFAULT_MEMBER_TEMPERATURE_VISIBILITY` *(spec 3.7.13.3)* · `REASON_AUTO_TEMPERATURE` *(spec 3.7.8, 3.7.13.6)*. ✅ **And the spec section exists:** **ch3 §3.7.14 "Space Admission"** (`:2827`), with §3.7.14.2 stating both value meanings and both fallback rules. **Cite the spec; keep the lock labels as a secondary reference if wanted.** *A `wire.rs` constant pointing at a `tasks/` file would be the first in the file, and task files are not normative.*

📌 **One low-severity knock-on:** the `VISIBILITY_*` doc comment reads *"**Permitted values** for `SpaceState.member_temperature_visibility`"*. Copied verbatim, an admission doc comment saying *"Permitted values"* reads as an allow-list while §4.3 stores anything verbatim. **The same tension already exists on the sibling and is consistent with the file**, so this is a wording note, not a defect — but with `D-149` in play (F-1), *"the two values this build interprets"* is truer than *"permitted values"*.

---

### ⚠️ F-7 — THE `1604` BASELINE IS CARRIED, NOT MEASURED, AND THE RUNBOOK DOES NOT SAY SO

§4.4 asserts *"`cargo` MOVES 1604 → 1608"* and V-1 expects `1608`. **The runbook has no baseline-measurement gate** — V-0 is a `cargo check`, not a test run.

✅ **The carry is defensible and I verified it:** `git show --numstat 3876950` touches **`CLAUDE.md`, `DECISIONS.md`, `JOURNAL.md`, `docs/ROADMAP.md`, `tasks/M_SPACE_ADMISSION_PHASE0.md` — zero `.rs`** ⇒ 1604 (measured at `eedfebd`, J-755) carries **by scope**.

⚠️ **But J-755's `V-1b` lesson is one session old and is the opposite habit:** *"both sides of the delta are now measured on the same tree, in the same session, with the same binaries … a figure carried unmeasured for twenty consecutive sessions was one command away from being measured the whole time."* **State the carry and its scope justification, or measure it at V-0.** *Carrying silently is what made the twenty-session gap invisible.*

---

## §3 — WHAT I TRIED TO BREAK AND COULD NOT

Recorded because a clean result is a result, and because the runbook should not be re-audited on these.

- ✅ **All seven §2 line citations hold EXACTLY at `3876950`** — S-1 `597-603` (banner `:597`, consts `:600`/`:603`) · S-2 `631-641` · S-3 `186-258` (`threads` at `:257`) · S-4 `307-310` · S-5 `312-336` (`Ok(SpaceState {` at `:312`, `threads` at `:335`) · S-6 `443-468` · S-7 `559-583`. ⇒ **§7 trigger ④ does not fire.** Phase-0 §15.2's constructor sites `:265` / `:342` / `:496` are also exact.
- ✅ **M-1 holds in full.** `SpaceState` derives **`Debug, Clone, PartialEq, Eq`** at `:185` — **no `Serialize`, no `Deserialize`**. No `impl Default for SpaceState` anywhere. **No serialisation of `SpaceState` anywhere in the workspace** (every `json!(space…)` hit is a `space_id` string). ⇒ **§7 trigger ⑥ cannot fire; no migration; additive for the `derive_resolved` convergence oracle.**
- ✅ **M-3 is exact:** `state.rs` holds **82 `#[test]` functions**, and the constructor tests really do sit at `:2811-2930`.
- ✅ **V-4's crate scope holds** — the only literal outside §2 is in `xgen-core` (F-2's `types.rs` is an import list, not a literal).
- ✅ **§6 item 1 holds structurally.** Leg A-bis's two tests live in `xgen-node/src/tests/space_admission_third_party_join.rs` (`:91`, `:229`) and **construct no `SpaceState` literal** ⇒ unaffected by the field.
- ✅ **V-2's `56 SUITES` is sound** — Leg B adds `#[test]` functions to an existing `mod tests`, no new test module.
- ✅ **V-3 is a genuine negative control that can fail.** Test 1 asserts absent ⇒ `ADMISSION_OPEN`; hardcoding `"invite"` in place of the `unwrap_or_else` makes it fail. It targets the default path specifically, which is the path a fixture is most likely to leave unexercised.
- ✅ **§4.4's tests 3/4 are self-controlling BY DESIGN** — putting the *wrong* value in content is what separates pinning from parsing. **This is the runbook's best idea and it catches a defect the neighbouring `e2e` test already has** (F-3). Its only gap is the mechanism.
- ✅ **No identifier collision on `admission`** — every existing occurrence is prose or a comment.
- ✅ **§0's "write no validation" is correct** — for the reason in F-1, not the one given.

---

## §4 — MY OWN INSTRUMENT FAILURE, REPORTED

My first literal census returned **12 hits** for `SpaceState {` across six files, including **two in `xgen-node/src/tests/m8_s2_convergence.rs`** — which would have fired §7 trigger ① and blown V-4. 🛑 **The pattern matched `-> SpaceState {` return-type annotations**, which are not literals. Re-classified by reading every hit rather than counting them, the true partition is **four literals**, and the `xgen-node` hits are function signatures.

🔑 **`N-197` rule ① applied to myself: a census over-matched, and the wrong reading was the ALARMING one rather than the reassuring one — which is the only reason it got a second look.** Had the over-match run the other way it would have entered this document. **Every count above was taken by reading the hits, not by trusting the total.**

---

## §5 — WHAT I DID NOT DO

- 🛑 **No product code.** No file under `xgen-core/`, `xgen-common/`, `xgen-node/`, `xgen-client/` or `ui/` was modified. `git status` shows only this new file added to the two pre-existing working-tree changes.
- 🛑 **No runbook edit.** Findings are for Chat to re-drive and fold (Rule 5, both directions).
- 🛑 **No floors run.** cargo **1604 / 0 / 62 × 56 SUITES** · vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15** — **carried, not measured this session.** Catalogue **UNMEASURED**. These are reads only and moved nothing.
- 🛑 **I did not verify that `cargo check --workspace` compiles today.** V-0's enumeration is predicted from a source census, not from a compiler run. **My four-literal count is what the compiler should report; if it reports otherwise, trust the compiler and treat the difference as a finding against §1 of this document.**
