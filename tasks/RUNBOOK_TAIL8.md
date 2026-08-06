# RUNBOOK — M-RP-TAIL8: the unresolved-row fallback shows a short tail, not the whole key
> **Status**: COMPLETED  
> Version: 1.4  
> Date: Aug 2026  
> **Last updated**: 2026-08-06  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — What this is, and who holds it

Discharges `M_RP_MEMBERS.md` **§6a**, open since J-643: Joe locked **tail-8** at J-588 and the build renders the whole key, letting CSS clip it. First application of **`D-142`**.

**Seats:** design + records **Chat** · implementation **Clair** · the push **Joe**.
**Scope: surfaces A1 + A2 only** — `members-panel.svelte:96` (member row) and `:82` (self row). **A3 latent, A4 a separate milestone** — see `D-142`'s table.

⚠️ **PROVENANCE: DELEGATED.** Joe: *"let's go by your recomms finaly."* Mechanism **M3** below was **Chat's recommendation, not a walked appearance decision** (`D-127` shape, `D-141`). **Recorded as delegated so a later revisit reads it correctly.**

🔒 **JOE'S OWN LOCKS, uttered and quoted:** the visible form is **`…` + the last 8** (*"agree tail() with '…' (ascii 0133) as prefix"*) · the scope is **A1+A2 with the rule open to widen** (*"let us keep it as a1 and a2, with possibility to widen/expand usage"*).

---

## §2 — GROUNDED, measured at `1ea59de`

| # | fact | site |
|---|---|---|
| **G1** | `const tail = (xgid) => xgid.split('/').pop() \|\| xgid` | `members-panel.svelte:30` |
| **G2** | two call sites, both in that file: member row `:96`, **self row `:82`** | — |
| **G3** | XGID = `xgen://pubkey/ed25519:<43>` (65 chars) ⇒ `tail()` returns `ed25519:` + all 43 | J-657's derivation, re-checked |
| **G4** | `.ei-name` clips: `overflow:hidden` · `text-overflow:ellipsis` · `white-space:nowrap` | `skin.css:2475-2482` |
| **G5** | `deriveInitials` takes the **name** path first; the xgid fallback at `:94` is unreachable from this surface because the widget always passes a name | `entity-avatar.svelte:83-95` |
| **G6** | `SeenRecord.display_name` is **optional** ⇒ a fully-resolved member can have no name | `address-book.svelte.ts:75` |
| **G7** | `unresolved` and *"the name is a fallback"* are **different predicates** — `toDescriptor` resolves the name independently of how `unresolved` is computed | `members-panel.svelte:96` vs `:128-135` |

🔑 **G6 + G7 are why the skin cannot carry this rule and the widget must.** Four row types reach a tail-rendered name and only one of them carries `data-unresolved="unasked"`. **The widget is the only place that knows it fell back.**

---

## §3 — THE CHANGE: two files, two lines

### 3.1 — `ui/common/lib/components/widgets/members-panel.svelte`

**Replace the helper at `:28-30`.** 🛑 **RENAME IT — do not keep the name `tail`.** A function called `tail` that returns `…gMpQaXrB` is false at its own name, and *a token carrying two meanings* is the defect class this arc has hit four times. Both call sites are in this file.

```ts
// D-142: where no display name is known, an identity shows `…` + the last 8 characters of its
// key — never the raw long pubkey. The ellipsis is part of the STRING, not the skin: only this
// widget knows the name is a fallback (a resolved member may simply have no display_name, and
// an erased member may still carry a cached one), and no CSS selector can express that.
const tail8 = (xgid: string) => (xgid ? `\u2026${xgid.slice(-8)}` : '');
```

- `:82` → `selfState.identity.display_name ?? tail8(selfState.identity.identity_id ?? '')`
- `:96` → `name: rec?.display_name ?? tail8(m.identity_id)`

🔒 **The empty guard is load-bearing and is not tidiness.** `:82` passes `?? ''`. Without the guard an absent self identity renders a bare `…` — **a row that looks like a person whose name is one ellipsis.** Today it renders empty; that behaviour is preserved exactly.
📌 `\u2026` is written as the escape, not the literal character, so the intent survives any editor or encoding round-trip.

### 3.2 — `ui/core/lib/components/data-dependent/entity-avatar.svelte`

**At `:84`**, strip **leading** non-letter/non-digit characters before deriving initials, so a fallback name yields `7W` and not `…7`.

```ts
const n = (nm ?? '').trim().replace(/^[^\p{L}\p{N}]+/u, '');
```

⚠️ **LEADING ONLY, AND VIA `\p{L}`/`\p{N}` — BOTH DELIBERATE.** A blanket `[^a-z0-9]` strip would empty a CJK or Cyrillic name (`李明` → no initials at all) and a non-anchored strip would eat interior punctuation. *The obvious one-liner is the wrong one-liner here.*
📌 A name that is **entirely** punctuation now falls through to the xgid path at `:94` rather than producing empty initials — a strict improvement, and stated so it is not read as an accident.

### 3.3 — `ui/sampler/src/app_sampler.svelte`

🛑 **The fixture AND its comment change together, in the same commit.** `:290-300` states *"⚠️ TAIL LENGTH IS LOAD-BEARING … a short fixture tail would not clip, and would show a more legible string than the product can produce."* **After this change that premise is false** — the product's own string is 9 characters and does not clip. Fixtures at `:303`/`:304` carry 43-char tails and become fixtures for behaviour that no longer exists.

⇒ both fixture names become `…` + their last 8; the comment is **rewritten to the new premise**, and the superseded sentence is **kept and marked** (`D-131`), not deleted. *`N-109`'s defect is a comment that was true when written and load-bearing for the next reader.*

🛑 **v1.0's §3.3 WAS UNRUNNABLE AND V3's EXPECTED VALUE WAS WRONG — BOTH CHAT'S, BOTH FOUND BY CLAIR BEFORE A LINE WAS COMMITTED. Corrected at v1.1; the superseded instruction is kept above (`D-131`).**

**① The two fixtures end in the SAME 8 characters.** Measured: `:303` → `gMpQaXrB`, `:304` → `gMpQaXrB`. Applied literally, v1.0 produced **two identical rows** in the one place whose purpose is to show them apart. 🔑 ***An instruction that destroys the property the surface exists to demonstrate.***

**② V3's expected `7W` came from the wrong end — and worse than "first-8 vs tail-8".** `7WGuWOqU` is what Leg F **saw on screen**, visible *only because `.ei-name` clips LEFT-anchored*. It is the **head** of that key. **The tail of the real Leg F key was never observed and cannot be known.** ⚠️ ***The fifth instance in this arc of head and tail being conflated — and this one sat inside a GATE, where it would have read as a measurement.***

🔒 **RULED (Chat, `D-123` — sampler fixtures are test data, not appearance): OPTION A, WITH THE DERIVATION ORDER CORRECTED. THE GATE IS DERIVED FROM THE FIXTURE; A FIXTURE IS NEVER CHOSEN TO MAKE A PRE-WRITTEN GATE TRUE.**

| fixture | key (43 chars) | renders | avatar |
|---|---|---|---|
| `:303` **unasked** | `7WGuWOqU4kM2vN8pLdA3QmR47bTfHs1YnE6cZkW5jU0` | `…cZkW5jU0` | **`CZ`** |
| `:304` **erased** | `Zk9WbT5cH1sYnE6f7QmR4xK2vN8pLdA3jU0gMpQaXrB` — **unchanged** | `…gMpQaXrB` | **`GM`** |

📌 **Only ONE fixture changes**, so the diff shows exactly what moved. 📌 **`7WGuWOqU` sits at the HEAD of the new key — where Leg F actually observed those characters** — so the fixture stays recognisably related to the real row **without pretending we know its tail.**

⚠️ **The `:302` control (`Bob Lee`, resolved) is UNCHANGED.** It is V5's subject, and *a fixture that moves is not a control.*

🛑 **v1.1's TABLE ABOVE IS ITSELF WRONG AND IS SUPERSEDED AT v1.2 — CHAT'S THIRD DEFECT IN THIS DOCUMENT, FOUND BY CLAIR ON EXECUTION. Kept, not deleted (`D-131`).** It carries a *"key (43 chars)"* column **for a field that has never held a key**, and says *"only one fixture changes"* while also claiming the unchanged one renders `…gMpQaXrB`. **Both cannot be true.**

✅ **MEASURED BY CLAIR, AND AIRTIGHT:** `epUnresolved` is passed **verbatim** to `<EntityPanel>` (`app_sampler.svelte:1013`); `entity-item` renders `descriptor.name ?? descriptor.id` **verbatim** (`:73`, `:133`); **there is no `tail8`, `slice` or any transform anywhere in the sampler path** — the only `tail8` lives in `members-panel`, **which the sampler does not mount** (its sole reference is a comment at `:308`). ⇒ **the fixture `name` field has ALWAYS stored a PRE-RENDERED string; today's `ed25519:9xK2…` is `tail()`'s OUTPUT, not an identity_id.** *Chat described the field wrongly and then built a table on the description.*

🔒 **RULED (Chat): SHORT FORM ONLY. TWO FIXTURE LINES MOVE, NOT ONE — "erased unchanged" is RETRACTED.**

| fixture | `name` before | `name` after | renders | avatar |
|---|---|---|---|---|
| `:303` **unasked** | `ed25519:9xK2…gMpQaXrB` | **`…cZkW5jU0`** | `…cZkW5jU0` | **`CZ`** |
| `:304` **erased** | `ed25519:Zk9W…gMpQaXrB` | **`…gMpQaXrB`** | `…gMpQaXrB` | **`GM`** |

🔒 **`id` UNCHANGED on both, and `:302` UNCHANGED.** ⚠️ **NO full key goes into the unasked `id`:** *"recognisably related to Leg F"* was **decoration, not load-bearing**, and `descriptor.id` feeds the **avatar SEED** — changing it moves the swatch colour with **no gate covering that**. ***A cosmetic flourish must not buy an unmeasured change.*** The `7WGuWOqU` lineage lives in the comment.

🔒 **AND NO `tail8` TRANSFORM IS ADDED TO THE SAMPLER.** It would re-implement product logic in a fixture harness and truncate the `Bob Lee` control. **The sampler renders what it is given; that is its contract.**

🔑 **THE DERIVATION RULE, SHARPENED BY THIS FINDING — FORM vs VALUE:** the **SURFACE dictates the fixture's FORM** (the sampler renders verbatim, so a fixture must hold the rendered string — a fact, not a choice); the **FIXTURE dictates the gate's VALUE** (`CZ`/`GM` are READ OFF the chosen strings, never picked first and made true). ***The rule forbids reverse-engineering a VALUE; it never governed FORM.***

🔒 **WHAT THE REWRITTEN `:290-300` COMMENT MAY AND MAY NOT CLAIM.** **MAY:** the fixtures carry the same **FORM** the product now produces (`…` + 8), the two tails are **distinct on purpose** so the unresolved states are distinguishable, and the unasked tail comes from a 43-char key whose **head** is `7WGuWOqU` — the characters Leg F actually observed on screen. **MUST NOT:** claim the sampler mirrors the product's **transform**. It has none. It mirrors the product's **output**.

---

## §4 — WHAT IS NOT TOUCHED

`skin.css` (**no rule is added — M3 needs none**) · `entity-item.svelte` · `entity-panel.svelte` · any prop signature · `stream/derive.ts` · `message.svelte` · any `.rs` · the wire · stored records.

---

## §5 — GATES

🛑 **THE GATES SPLIT IN TWO, AND v1.0–v1.1 DID NOT SAY SO — CHAT'S FOURTH DEFECT IN THIS DOCUMENT.** The sampler **does not mount `members-panel`** ⇒ **`tail8`, the function §3.1 changes, is executed by NO sampler gate**, and **V4's subject (the self row, `:82`) is unreachable from the sampler.** V1/V3/V5/V6 run against **hardcoded fixture strings**: they prove the **RENDER FORM**, never the **PRODUCER**. 🔑 ***A gate suite that cannot fail on the changed function is not a gate suite for that function.***

🔒 **JOE RULED L1 (2026-08-05, uttered — *"i like rather l1"*): THE MILESTONE CLOSES ON A LIVE CLIENT RUN, NOT ON THE COMMIT.** ⚠️ **INTERACTIVE ⇒ CUSTODY TRANSFERS TO JOE UNDER `D-132`.** *The alternatives were rejected with reasons: L2 (ship sampler-verified, record the producer unverified) is Leg D's shape, which Leg F then had to discharge weeks later; L3 (export `tail8` for a unit test) makes a component's private helper public API to satisfy a test.*

### 5a — SAMPLER-SIDE (Clair, on the committed tree)

| # | gate | passes when |
|---|---|---|
| **V1** | the rendered member row | exactly `…` + 8 characters; the ellipsis verified as **`E2 80 A6`** at the byte level, not by eye |
| **V2** | **the double-ellipsis gate** | `scrollWidth <= clientWidth` on `.ei-name` ⇒ `text-overflow` never fires ⇒ no `…cZkW5j…` (an ellipsis at BOTH ends, from two different mechanisms). **The panel width is RECORDED with the result** — *"it fits today"* is a claim about one desk |
| **V3** | avatar initials | **`CZ`** on the **unasked** row, **`GM`** on the erased row — read from the live DOM, **derived from §3.3's fixtures**. 🛑 **A LEADING `…` among the initials is the failure this gate exists to catch.** ⚠️ *v1.0 expected `7W` — the HEAD of the real Leg F key, not a tail. Corrected at v1.1, see §3.3 (`D-131`).* |
| **V4** | **the self row** (`:82`) | rendered and read; and with an absent self identity the row is **empty, not `…`** |
| **V5** | the control | a **resolved** row in the same eval, name and initials **unchanged** — *a probe returning the same answer for differentiated inputs cannot fail* |
| **V6** | a **non-Latin** name in the sampler | initials still derive (the `\p{L}` guard did its job) — **RED on reverting 3.2 to `[^a-z0-9]`** |
| **V7** | floors | `svelte-check` **re-measured before the first edit**, Δ explained · catalogue **435** · slot gate PASS 74 · `cargo` **not run**, stated |

⚠️ **V2 and V4 ARE NOT SAMPLER GATES.** Clair reports them **NOT DRIVEN, with the reason** — never as passed, never silently omitted. ***A gate whose subject is unreachable has not been satisfied, and the two must never be written the same way*** (the J-673 W8 lesson).

### 5b — PRODUCT-SIDE (the live run, Chat drives, Joe holds custody)

| # | gate | passes when |
|---|---|---|
| **V2** | **the double-ellipsis gate** | on the REAL members column: `scrollWidth <= clientWidth` on `.ei-name` ⇒ `text-overflow` never fires. **The panel width is RECORDED with the result.** 📌 *Trivially satisfied on the sampler at 300 px and therefore NOT evidence about the product.* |
| **V4** | **the self row** (`:82`) | rendered and read; and with an absent self identity the row is **empty, not `…`** |
| **V8** | 🔑 **`tail8` ACTUALLY RUNS** | a real unresolved member row in the client renders `…` + 8 **produced by the function**, not by a fixture. 🛑 **This is the only gate that touches §3.1 at all.** |

⚠️ **V8 needs a member the client cannot resolve** — Leg F's lever (a joiner, or the node down inside the fetch window). **The run plan is written when Joe transfers custody, not guessed at here.**

---

### ✅ 5b — DRIVEN 2026-08-06 BY CHAT ON THE REAL CLIENT (WebView2, CDP 9222), AT `165b821`. ALL THREE GREEN.

🔑 **THE LEVER WAS ALREADY IN JOE'S DATA — NO JOINER, NO NODE-DOWN WINDOW, NO WRITE.** The DM Space `…6af09cd8…` has two members: Joe and `xgen://pubkey/ed25519:L87GVLyV…sno_FWmw`. **That key is LegF-DAVE**, established from the DAG rather than from its name: it is invite target #3 of the eight in `LegF Verification`, the other seven resolve in the address book as LegF-Bob / CAROL / N1–N5, and DAVE **signed two `membership.join` events himself** before being erased. He is absent from `xgen-node_identities.db` **and** from the client address book ⇒ the fill returns `identity.not_found` ⇒ `notFoundIds`. An erased row is normally hidden, **except the DM counterpart** (§5a E2, J-648) ⇒ the row renders, `rec?.display_name` is `undefined`, **`tail8` runs.**

**Two latches, one plan.** Latch 1 `LegF Verification` › `LegF Room` (the control). Latch 2 the DM Space › `dm` (the subject).

| # | result | measured |
|---|---|---|
| **V8** | ✅ **GREEN** | `…sno_FWmw` · len **9** · cp0 **`0x2026`** · bytes **`e2 80 a6`** · `data-unresolved="erased"` · `data-selected="true"` |
| **V2** | ✅ **GREEN** | `scrollWidth 158 === clientWidth 158` · `clipped:false` · `painted:true`, `rectW 158` · 🔒 **panel width 218 px, `.ei-name` 158 px — RECORDED, one desk** |
| **V4** | ✅ **GREEN, PARTIAL BY CONSTRUCTION** | self row first, `Joe`, resolved, weight 600, unmarked. ⚠️ **The empty-guard sub-case was NOT driven — see the amendment below.** |

🔒 **V8 IS PROVEN AS THE FUNCTION'S OUTPUT, NOT A FIXTURE, AND THE PROOF IS IN THE SAME EVAL.** `book` held **9 keys, none ending `sno_FWmw`**; `roster` held 2, one of them DAVE. ⇒ `rec?.display_name` is `undefined` and `tail8(m.identity_id)` is the only producer on that path; the sampler's fixtures are not mounted in the client. 📌 **AND THE ARM IS NAMED (`D-140`):** `m.unresolved` was `undefined` on both rows ⇒ this is the **fill / not-found** arm. The **live-delta** arm (`addMember` → `unresolved:true` → `resolveMember`) reaches the same call site and **was not exercised**.

✅ **THE CONTROL DISCRIMINATES (J-655).** Latch 1: **8 rows**, every one resolved, `data-unresolved:null`, **no `tail8`**, identical 218/158 geometry. DAVE is in that roster too and was correctly **hidden** (erased, not counterpart). *Same probe, different answer ⇒ it can fail.*

📌 **THREE THINGS MEASURED THAT NO GATE ASKED FOR.** `font-weight 500` on the erased row vs `600` on `Joe` — `skin.css:2558` fires, so the string was measured at the weight it ships at · `text-decoration: line-through` present, scoped to the name, highlight intact · **avatar initials `SN`, not `…s`** — §3.2's leading-`\p{L}` strip working on a **product-produced** string, which §5a could only show against fixtures.

🔑 **AND ONE FINDING THE GATES COULD NOT HAVE PRODUCED — §8's ENTRY 8, AND `N-168`.** Joe read the painted screen: **`line-through` runs THROUGH the leading `…`**, so two independent marks stack on one string. **§5b measured the string layer and the layout layer and treated PAINT as following from them.** It does not. 🔒 **Disposition (Joe, DELEGATED — `D-141`): SHIP, AND FILE at `N-168`.**

⚠️ **V4's SECOND HALF IS AMENDED, NOT SILENTLY PASSED (`D-131`).** §5a and §5b both read *"with an absent self identity the row must be empty, not `…`"*. **That phrasing conflates two states**, measured at `self-state.svelte.ts:37-38` where both fields are `string | null`:

- `selfState.identity === null` ⇒ `selfDescriptor` is `null` ⇒ **no self row renders at all.** The guard never runs.
- `identity` present with `identity_id: null` ⇒ `tail8('')` → `''` ⇒ **a rendered row with an empty name.** *That* is what the guard protects.

⇒ the second state is **not reachable by any product action** — only by driving `__XGEN_SELF__.setIdentity(…)` from CDP. **NOT DRIVEN, with the reason**, per the §5a rule that a gate whose subject is unreachable has not been satisfied and must never be written like one that was (the J-673 W8 lesson).

🔒 **V2 IS WRITTEN AT THE LAYOUT LAYER ON PURPOSE (`D-140`)** — *"the ellipsis is not doubled"* is a claim about **layout**, and a string-length assertion cannot decide it.
🔒 **V6 IS THE ONE THAT MUST BE PROVEN ABLE TO FAIL** — a gate that has never failed is not known to work (J-655).

---

## §6 — COMMIT SHAPE

**One commit, Clair.** 3 files, `svelte-check` only, zero `.rs`. *No floor split is needed: nothing here moves cargo, and splitting a three-file frontend change makes the `svelte-check` Δ unattributable rather than clearer.*

🛑 **THE COMMIT IS NOT THE CLOSE (L1).** After it lands, Chat re-drives §5a on the committed tree, then §5b runs live under Joe's custody. **The milestone closes after §5b**, and the doc bridge carries both sets of numbers.

Then Chat's doc bridge: JOURNAL + `CLAUDE.md` PLAY + `docs/ROADMAP.md` + `M_RP_MEMBERS.md` §6a discharged + this runbook + the Phase-0 (`D-074`). **Joe pushes both.**

---

## §7 — DoD

- [x] `svelte-check` baseline re-measured **before** the first edit, not inherited from J-673 — **0 err / 34 warn / 15 files**
- [x] 3.1, 3.2, 3.3 landed; helper **renamed** `tail` → `tail8`, both call sites updated (`165b821`, 3 files, +31/−19)
- [x] V1–V7 driven **on the committed tree** (§5a, Chat on WebView2), V6 proven RED on revert by Clair and re-driven by Chat
- [x] 🔒 **§5b driven live (Joe's L1): V2 · V4 · V8 all green**, V2 recording **panel 218 px / `.ei-name` 158 px**
- [x] `M_RP_MEMBERS.md` §6a marked **DISCHARGED**, with the date and the commit `165b821`
- [x] the close records ***"`D-142` applied at the roster"***, never *"`D-142` applied"*
- [x] Records in one commit (`D-074`)
- [x] ⚠️ **V4's empty-guard sub-case recorded NOT DRIVEN, with its reason** — never as passed, never silently omitted
- [x] 📌 **The paint-layer finding filed at `N-168`**, not absorbed into the close

---

## §8 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

1. ✅ **CLOSED 2026-08-06 — the doubt below is DISCHARGED, annotated not deleted (`D-131`).** `\p{L}`/`\p{N}` demonstrably execute in this WebView2: measured at §5a, and §5b then produced **`SN`** from a product string that begins with `…`. The build did not reject the escapes and the fallback described here was never needed. *Superseded text follows.* — **The `\p{L}` regex is written, not run.** Unicode property escapes need the `u` flag and a modern target; if the build rejects it the fix is `[^A-Za-z0-9]` **plus an explicit non-Latin carve-out** — *not* a silent downgrade to `[^a-z0-9]`, which is the trap V6 exists to catch.
2. 🔑 **v1.0 CARRIED TWO DEFECTS AND CLAIR CAUGHT BOTH BEFORE ANY COMMIT** — §3.3 collapsed two fixtures into one string, and V3's expected value was taken from the wrong end of a key. **Neither would have been caught by re-reading; both were caught by someone outside the text trying to EXECUTE it.** ⚠️ *Recorded here and not only at the close, because the next reader of this runbook should know its gates have already been wrong once.*
3. **`deriveInitials` has no known test.** The corpus searched was `ui/**` for `deriveInitials` and `initials`; if a test asserts today's `ED` it goes RED and **moves with the change** rather than being worked around.
4. **V2 assumes 9 characters fit the members column.** Believed from Leg F's measurement of a 250 px column, **not re-measured for the new string.** If it does not fit, M3 still holds and the doubled ellipsis becomes a real finding.
5. **This closes one of the surfaces Finding 5 named, not the inconsistency.** The feed still renders `slice(-6)` and message rows still render the full XGID. **Stated here so the close cannot overstate itself.**
6. 🛑 **THIS DOCUMENT HAS BEEN WRONG FOUR TIMES AND EVERY CATCH CAME FROM OUTSIDE THE TEXT.** §3.3 collapsed two fixtures into one string (v1.1) · V3's value came from the **head** of a key, not its tail (v1.1) · §3.3's replacement described the fixture field as holding a *key* when it has always held a **pre-rendered name** (v1.2) · §5 presented one gate suite when the sampler **cannot execute the function the milestone changes** (v1.2). ***Chat's own re-reads passed every time; Clair caught all four by trying to RUN the document.*** ⚠️ **Treat these gates as load-bearing but not yet trustworthy.**
7. 🔑 **V3 IS BLIND TO THE THING §3.2 EXISTS FOR — measured by Clair at v1.3, and it is a gate-design finding, not a defect.** Under the naive `[^a-z0-9]`, the shipped fixtures **still render `CZ`/`GM`**: the leading `…` is stripped either way and the ASCII tail survives. ⇒ **the shipped fixtures alone cannot prove the `\p{L}` guard is necessary.** Only **V6's non-Latin case discriminates**, which is precisely why it had to introduce one. 📌 *The guard earns its keep for the LATENT case — a resolved member with a real non-Latin `display_name` — not for anything the fixtures show.* ⚠️ ***A gate that passes under both the correct and the incorrect implementation is not evidence about that implementation.*** 📌 **NARROWED 2026-08-06:** Clair filed this as *"the suite is blind"*; it is narrower than that. **V5's control DOES discriminate** — `Bob Lee` → `BL` shipped vs `BO` naive, because the unanchored strip eats the SPACE. *The blindness is V3's, not the suite's.*
8. 🔑 **AND THE FIFTH CATCH CAME FROM OUTSIDE THE TEXT TOO — THIS TIME FROM THE PAINTED SCREEN, NOT FROM CLAIR.** §5b's gates were written at the **string** layer and the **layout** layer. Both passed. Joe then looked at the render and saw the `line-through` **running through the leading `…`** — two independent marks stacked on one string, which no probe in this harness asks about. ⇒ **`N-168`, filed not fixed, `D-141` DELEGATED.** ⚠️ ***`D-140` says name the layer that decides. This document named two and assumed the third followed. It does not.***
9. 📌 **THE MILESTONE'S OWN SCOPE, RESTATED SO THE CLOSE CANNOT OVERSTATE IT.** What ships is **`D-142` APPLIED AT THE ROSTER** — surfaces **A1 + A2**, both `members-panel` call sites, and the **fill / not-found arm** of them. The **live-delta arm** reaches the same call site untested; **A3 stays latent**; **A4 (message rows, the full 65-character XGID) is LIVE and untouched**; the feed still renders `slice(-6)`. **A close that says *"`D-142` applied"* commits this project's named defect class in the entry that closes it.**
