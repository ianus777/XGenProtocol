# RUNBOOK — M-RP-TAIL8: the unresolved-row fallback shows a short tail, not the whole key
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-05  
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

---

## §4 — WHAT IS NOT TOUCHED

`skin.css` (**no rule is added — M3 needs none**) · `entity-item.svelte` · `entity-panel.svelte` · any prop signature · `stream/derive.ts` · `message.svelte` · any `.rs` · the wire · stored records.

---

## §5 — GATES

| # | gate | passes when |
|---|---|---|
| **V1** | the rendered member row | exactly `…` + 8 characters; the ellipsis verified as **`E2 80 A6`** at the byte level, not by eye |
| **V2** | **the double-ellipsis gate** | `scrollWidth <= clientWidth` on `.ei-name` ⇒ `text-overflow` never fires ⇒ no `…7WGuWOq…`. **The panel width is RECORDED with the result** — *"it fits today"* is a claim about one desk |
| **V3** | avatar initials | **`7W`** — read from the live DOM. 🛑 **`…7` is the failure this gate exists to catch** |
| **V4** | **the self row** (`:82`) | rendered and read; and with an absent self identity the row is **empty, not `…`** |
| **V5** | the control | a **resolved** row in the same eval, name and initials **unchanged** — *a probe returning the same answer for differentiated inputs cannot fail* |
| **V6** | a **non-Latin** name in the sampler | initials still derive (the `\p{L}` guard did its job) — **RED on reverting 3.2 to `[^a-z0-9]`** |
| **V7** | floors | `svelte-check` **re-measured before the first edit**, Δ explained · catalogue **435** · slot gate PASS 74 · `cargo` **not run**, stated |

🔒 **V2 IS WRITTEN AT THE LAYOUT LAYER ON PURPOSE (`D-140`)** — *"the ellipsis is not doubled"* is a claim about **layout**, and a string-length assertion cannot decide it.
🔒 **V6 IS THE ONE THAT MUST BE PROVEN ABLE TO FAIL** — a gate that has never failed is not known to work (J-655).

---

## §6 — COMMIT SHAPE

**One commit, Clair.** 3 files, `svelte-check` only, zero `.rs`. *No floor split is needed: nothing here moves cargo, and splitting a three-file frontend change makes the `svelte-check` Δ unattributable rather than clearer.*

Then Chat's doc bridge: JOURNAL + `CLAUDE.md` PLAY + `docs/ROADMAP.md` + `M_RP_MEMBERS.md` §6a discharged + this runbook + the Phase-0 (`D-074`). **Joe pushes both.**

---

## §7 — DoD

- [ ] `svelte-check` baseline re-measured **before** the first edit, not inherited from J-673
- [ ] 3.1, 3.2, 3.3 landed; helper **renamed**, both call sites updated
- [ ] V1–V7 driven **on the committed tree**, V2 recording its panel width, V6 proven RED on revert
- [ ] `M_RP_MEMBERS.md` §6a marked **discharged**, with the date and the commit
- [ ] the close records ***"`D-142` applied at the roster"***, never *"`D-142` applied"*
- [ ] Records in one commit (`D-074`)

---

## §8 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

1. **The `\p{L}` regex is written, not run.** Unicode property escapes need the `u` flag and a modern target; if the build rejects it the fix is `[^A-Za-z0-9]` **plus an explicit non-Latin carve-out** — *not* a silent downgrade to `[^a-z0-9]`, which is the trap V6 exists to catch.
2. **`deriveInitials` has no known test.** The corpus searched was `ui/**` for `deriveInitials` and `initials`; if a test asserts today's `ED` it goes RED and **moves with the change** rather than being worked around.
3. **V2 assumes 9 characters fit the members column.** Believed from Leg F's measurement of a 250 px column, **not re-measured for the new string.** If it does not fit, M3 still holds and the doubled ellipsis becomes a real finding.
4. **This closes one of the surfaces Finding 5 named, not the inconsistency.** The feed still renders `slice(-6)` and message rows still render the full XGID. **Stated here so the close cannot overstate itself.**
