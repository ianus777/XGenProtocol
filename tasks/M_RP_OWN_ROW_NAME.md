# M-RP-OWN-ROW-NAME — own message rows resolve the self display name, and the Self toggle
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-23  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

**Phase-0 (D-071): design + records. NO CODE until Joe says go.**

- ⚠️ **This is a MESSAGE/STREAM milestone, not a self-widget one.** Joe's correction, J-576: *"it relates to the messages widget, doesnt it?"*
- ⚠️ **ZERO `self-panel.svelte` edits** (locked J-576). ⇒ the connection light stays and D5's FIRST WRITER survives **by construction**.
- ⚠️ **ZERO wire. No `identity.update`. No schema bump. No stored-data migration.**
- **Closes lock #5** (`tasks/M_RP6_3_COMPOSER.md` §9.11.3), UNMET since Leg A (J-569).
- **Does NOT** touch inbound rows — that is `M-RP-INBOUND-NAME`, blocked on the address book.

## §1 — Why it exists

Lock #5: own rows render the author name as the **full 65-char XGID**. Joe's fix (J-574 §11): the name becomes **"Self"** — default, customisable later, visually distinguished. D-124 then established that *"customisable"* means **the toggle**, not renaming, so **the lock closes without resolving how G is edited**.

## §2 — Grounding (measured at `293481a`)

| fact | site |
|---|---|
| own echo carries **no name** | `stream-panel.svelte:115` — `author: { kind:'identity', id: selfId ?? '' }` · *NO name (C-8)* |
| inbound carries **no name** either | `stream/derive.ts:80` — *NO name (C-8)* |
| the row falls back to the id | `message.svelte:69` / `entity-item.svelte:65` — `name ?? id` |
| **the self name IS resolvable** | `self-panel.svelte:43` — `identity?.display_name` ⇒ **"Joe"**, from the `get_self_state` Tauri command |
| the store is reachable | `selfState` is a `$common` store; `stream-panel` is a `common` widget ⇒ **W-3 clean** |

🔑 **Two sources, not two bugs.** The self panel never reads the message plane.

## §3 — C-8: a NAMED EXCEPTION, not a violation

**C-8 (`M_RP6_3_COMPOSER.md` §9.8) currently reads, in part:**

> *Nothing in the client resolves an XGID → display name: `spacesState` carries no members and R7 is unbuilt … nobody fabricates a name map (the J-501 rule: do not invent fields to make a panel look substantial).*

🔑 **That premise is TRUE for other people and FALSE for you.** `get_self_state` is **authoritative**, not a fabricated map. C-8 forbids inventing names; this invents nothing.

**Precedent, in the same file:** `stream-panel.svelte` already carries *"C-4 still governs INBOUND unchanged (project on read, no mirror); this is the narrow, documented outbound exception (§9.11.2)."* ⇒ **same shape, one constraint over.**

### §3.1 — The amendment text (DRAFTED HERE, APPLIED AT CLOSE)

⚠️ **Deliberately NOT applied in Phase-0.** C-8 describes **shipped behaviour**, and it is still accurate until Leg A lands. Amending it now would make the record describe code that does not exist — the inverse of the J-566 defect, not a cure for it. **It is drafted here so it cannot drift**, and lands with the code.

> **AMENDED (M-RP-OWN-ROW-NAME).** C-8 governs **inbound** authors unchanged. **Own rows are a narrow, named exception:** the self display name is resolved from `selfState.identity.display_name` (`get_self_state`), which is **authoritative, not a fabricated map** — the J-501 rule is untouched. Superseded wording is retained above rather than deleted, per the J-574 amendment discipline.

## §4 — The Identity settings section

🔒 **Joe-locked 2026-07-23.** Per **D-C** (`docs/xgen-settings-phase0.md`, 2026-07-16) and **D-067** (*no second home*): the toggle **dwells** in the one Settings modal; the self gate will **link** to it, non-exclusively.

- `SECTIONS` gains `{ key: 'identity', label: 'Identity' }` at **index 0**.
- ⚠️ **CONSEQUENCE, RECORDED AS CHOSEN, NOT INCIDENTAL:** `DEFAULT_SECTION = SECTIONS[0].key`, so **File ▸ Settings will land on Identity instead of About.** A user-visible change to a shipped path.
- **Why "Identity" and not "Account":** the docs rule on it — *"Identity is not an account. An account implies a relationship with a platform."* Discord says Account because Discord **has** accounts. 🔑 *Importing the word imports the model.*
- **Verified free:** no `key: 'identity'` exists; `data-section="identity"` renders clean. (`identity` is also an `EntityDescriptor` kind — a separate namespace, no interaction. Noted so it is not rediscovered.)
- 📌 **Sub-sections inside Identity are UNDECIDED** (Joe: *"maybe it will be enough to have sub-sections similar to discord's"*). **Not a blocker** — the milestone needs the section to exist, not its internal structure.

## §5 — The preference key

**Per-device**, on `uiStateStore` (D-124).

- `UiStateBag` is a **BAG** with per-key merge (N-107) ⇒ **adding a key is not a schema change.**
- Precedent is exact — an existing boolean session key documented as *"undefined = never set (the default true applies)"*.
- ⇒ **`version: 1` on `Store` does NOT bump. `migrateLayout` is NOT touched. No migration.**
- **Default: ON** (lock #5's default).

## §6 — Resolution and toggle semantics

| toggle | own row author name renders |
|---|---|
| **ON** (default) | **"Self"** |
| **OFF** | the registered `display_name` ⇒ **"Joe"** |

- 🔒 **ONE GLOBAL PREFERENCE, never per-space** (D-124).
- 🔒 The name is **merged**, never *collapsed* — `collapsed` is a persisted layout field (D-124).
- ⚠️ **Scope is own rows ONLY.** Inbound is untouched (C-8 stands). The self panel card is untouched (§0).

## §7 — Legs

| leg | content | seat |
|---|---|---|
| **Phase-0** | this document | Chat → **Joe locks** |
| **A** | Identity section · preference key · resolution · toggle control | **Clair**, from a locked runbook |
| **B** | the J-575 styling in `skin.css` | ⚠️ **JOE'S — no implementer may write it** |
| **Close** | CDP verification + records + the §3.1 C-8 amendment applied | Chat |

🔒 **A and B ride the SAME milestone** (Joe, 2026-07-23). ⚠️ The colour is **not cosmetic**: J-575 measured the author name at **2.65 : 1 at 10px — below WCAG AA** — and `#E5E5E5` takes it to **14.1 : 1**. Shipping A alone would put *"Self"* on screen in the failing grey: a known defect knowingly left (D-065).

**Leg B values (J-575, carried verbatim):** `font-weight: 600` · `font-style: italic` · `letter-spacing: 0.05em` · `color: #E5E5E5` · ⚠️ **`font-synthesis: none` REQUIRED** — without it a browser skew silently satisfies the italic (N-161).

🔑 **The styling marks YOUR NAME, not the word "Self"** — it applies in **both** toggle states (D-124 ④). It says *this is you*, not *this says Self*.

## §8 — DoD

**IMPLEMENTER (Leg A)**

- [ ] `SECTIONS` gains `identity` at index 0; deep-link lands on it
- [ ] `uiStateStore` session key added; absent ⇒ default ON; survives relaunch
- [ ] `echoToDescriptor` supplies `name` from `selfState.identity.display_name`
- [ ] Toggle control rendered in the Identity section
- [ ] `svelte-check` 0 errors; floors held

**JOE (Leg B)**

- [ ] `skin.css` carries the five declarations, `font-synthesis: none` included

**[CHAT] (Close)**

- [ ] ⚠️ **POSITIVE CONTROL:** BOTH states measured — ON ⇒ `"Self"`, OFF ⇒ `"Joe"`. *"The XGID is absent" and "nothing rendered" are the same string.*
- [ ] Italic proven **differentially** (N-161: advance width vs a bogus family, after `document.fonts.load()`) — a declared stack proves nothing
- [ ] Contrast re-measured; ≥ 4.5:1 confirmed
- [ ] §3.1 C-8 amendment applied to `M_RP6_3_COMPOSER.md` §9.8, superseded wording retained
- [ ] Lock #5 flipped to MET **with its verifier named** (the J-574 discipline)
- [ ] Registry baseline re-read, ⚠️ **stating which axis was counted** (N-155)

## §9 — Filed, NOT fixed

- ⚠️ **Inbound author names remain at 2.65 : 1** after this ships. The failing colour is on the author-name element generally; the fix here is scoped to self by Joe's ruling. ⇒ **N-162 contrast sweep** — measurement Chat's, remedy Joe's.
- ⚠️ **C-8's text names "Ms Design"**, a seat retired at J-568 and superseded by D-123. Left verbatim. ⇒ **twelfth candidate for `M-RP-SEAT-ORPHANS`** (lock #5 was the eleventh; ten was already called a floor).
- ⚠️ **`M-RP-INBOUND-NAME` is larger than first filed** — Ch2 specifies a **four-layer override chain**, not a flat XGID→name lookup. See the D-124 amendment.

## §10 — Handoff

**Phase-0 written. Awaiting Joe's lock, then a runbook for Clair.** ⚠️ **No code until he says go.**