# M-RP-PANEL-INERT — `entity-panel` gains a non-interactive mode
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-26  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHY THIS EXISTS, AND WHY IT SHIPS BEFORE LEG B

Opened 2026-07-26. **Clair stopped at the top of M-RP-MEMBERS Leg B rather than reconcile a contradiction silently, and she was right to.**

M-RP-MEMBERS Phase-0 §G2 said R7 follows the `spaces-panel` / `rooms-panel` template *"literally"*. That was read as **`entity-panel` is a neutral list renderer**. **It is an interactive single-select listbox.**

| Measured at `a0752e5` | |
|---|---|
| `entity-panel.svelte:138` | `<ul role="listbox" aria-multiselectable="false">` |
| `:146` | `<li role="option">` |
| `:150-151` | `onclick={() => selectAt(i)}` · `onkeydown={onKey}` |
| `:91` | `selectAt` writes `selected = it.descriptor.id` **unconditionally**, *before* `onActivate` |
| `skin.css:2604-2608` | `.entity-panel-option { cursor: pointer }` |
| `skin.css:2609-2612` | `:focus-visible` ring |
| props | `items · title · badge · collapsible · collapsed · selected · onActivate · emptyText · id` — **no `readonly`, no `inert`** |

🔑 **R1/R2 ARE CORRECT ONLY BECAUSE THEY CLOSE A LOOP R7 CANNOT.** `onActivate → selection.set(regionId, …)` (`rooms-panel:49`, `spaces-panel:50`) writes the global selection bus, and that value flows back into their `selected`. **The prop is the OUTPUT of an interaction.** R7 is forbidden from writing the bus (Leg B **L15**), so its `selected` is `$derived` one-way ⇒ a click writes the child's copy at `:91`, **nothing propagates back**, and the wrong highlight **sticks until the roster changes**. In a group room, where the highlight must be `null`, **one click manufactures one**.

⇒ Without this milestone, R7 would ship six affordances promising interactions Leg B does not wire, and would **assert something false about who the user is talking to**. That is the **N-097** shape — a live affordance lit from nothing — moved from the data layer to the interaction layer.

---

## §1 — SCOPE

**TOUCHED:**

| File | Change |
|---|---|
| `ui/core/lib/components/data-dependent/entity-panel.svelte` | one prop + conditionals across `:136-152` |
| `ui/assets/skin.css` | 🔓 **JOE'S FILE** — one sibling class, see §3 |

**NOT TOUCHED:** `entity-item.svelte` · `rooms-panel.svelte` · `spaces-panel.svelte` · any `.rs` · anything in `ui/common/lib/components/widgets/`.

**Floor:** `svelte-check`, baseline **0 err / 34 warn / 15 files** (re-measured 2026-07-26 at `a0752e5`).

📌 **Its own commit, though it moves the same floor as Leg B.** Unlike Leg A-quater the split is not about floors — it is about keeping a change to a **shared core composite** attributable on its own. `entity-panel` has three real consumers plus five sampler sites; a regression there must not be hidden inside a new-widget commit.

---

## §2 — THE CHANGE

New prop, **defaulting to today's behaviour**:

```
interactive?: boolean   // default true
```

Conditional across `:136-152`:

| Element | `interactive` (default) | `!interactive` |
|---|---|---|
| `<ul>` `role` | `listbox` | `list` |
| `<ul>` `aria-multiselectable` | `"false"` | omitted |
| `<li>` `role` | `option` | `listitem` |
| `<li>` `class` | `entity-panel-option` | the §3 sibling |
| `<li>` `aria-selected` | as today | omitted |
| `<li>` `tabindex` | roving | omitted |
| `<li>` `onclick` / `onkeydown` | as today | **not wired** |

🔒 **`selected` STILL FLOWS TO `EntityItem` (`:159`).** The point is not to remove the highlight — it is to remove every path by which the highlight can be **written from inside the component**. Inert means the row is a *render of state*, never a *producer* of it.

🔑 **THE ARIA CHANGE IS THE LOAD-BEARING HALF, NOT THE CURSOR.** `role="listbox"`/`role="option"` is a **contract**. Suppressing cursor and hover while keeping the roles would leave the panel announcing itself to a screen reader as a single-select listbox with a selectable option per member — **lying to assistive technology specifically, and lying harder than it lies visually**, because a sighted user at least sees nothing happen.

**Default `true` ⇒ zero behaviour change** for `rooms-panel:65`, `spaces-panel:62`, and the five `app_sampler.svelte` sites. That default holding **is** the regression test, and the sampler already exercises `entity-panel` five ways — **the harness is already standing.**

---

## §3 — 🔓 THE ONE PIECE THAT IS JOE'S

Clair wired the class name **`.entity-panel-listitem`** (the `role="listitem"` parallel to `.entity-panel-option`). ⚠️ **`skin.css` is Joe's under D-123 and is never folded into a Chat or Clair commit.** Clair wires the class name; **Joe writes any rule.**

🔒 **THE CLASS NAME IS DELEGATED (Joe, 2026-07-26: *"it is ok as it is"*)** — Clair picks it, no walked appearance decision.

⚠️ **MEASURED CORRECTION (J-596, 2026-07-26) — THE PREDICTED FAILURE DOES NOT OCCUR, AND THE REALITY IS BETTER.** This section predicted the inert `<li>` would inherit `list-style: disc` and lose its radius without a rule. **Grounded against the sampler (9422, CDP computed-style): it does not.** `.entity-panel { list-style: none }` sits on the `<ul>` (`skin.css:2596-2597`), and `list-style-type` **inherits**, so the inert `<li>` inherits `none` — **no bullet** (measured `list-style-type: "none"`, identical to `.entity-panel-option`). And the highlight's rounded background lives on the inner `.entity-item` (`border-radius: var(--rad0)`, `skin.css:2440`), **not** on the `<li>` — so the `<li>` carrying no `border-radius` is invisible (a bare `<li>` has no background to clip; measured both li radii `0px`). ⇒ **the inert rows render correctly with the class wired and NO skin rule at all** — no bullet, no pointer cursor (li default is not `pointer`), no focus ring (no `:focus-visible` rule, and no `tabindex` ⇒ not focusable). **The class is a hook; a rule is OPTIONAL (defensive, e.g. an explicit `list-style: none` in case the `<ul>` rule ever changes), not a correctness requirement.** DoD item 7 discharged: the bulleted-row failure was **reported openly** — it did not occur.

📌 **What DOES survive:** `.entity-item:hover` (`skin.css:2521`) — the inner item's own hover background — applies wherever an entity-item renders (see below). It is the one remaining affordance and is **Joe's** (L7 skin carve-out).

📌 **What this does NOT fix:** `.entity-item:hover` (`skin.css:2521`) is **entity-item's own** skin and applies wherever an entity-item renders. A hover background survives this milestone. ⇒ **L7's "no hover" remains a skin carve-out for Joe**, but after this it is the *only* one of six remaining rather than one of six.

---

## §4 — DoD

- [x] `interactive` prop added, defaulting `true`; `:136-152` conditional per §2 (split into two static-role branches — see §6)
- [x] `selected` still reaches `EntityItem` in both modes (sampler: inert selected row shows `[data-selected]`, `firstItemSelected: true`)
- [x] ARIA roles switch (`listbox`/`option` → `list`/`listitem`); no `aria-selected`, no `tabindex` when inert (CDP-measured)
- [x] No click or keydown handler wired when inert — verified in the DOM: a synthetic click on an inert row left `selected` unchanged, while the interactive positive control changed (N-163)
- [x] `rooms-panel`, `spaces-panel` and all five sampler sites **unchanged in behaviour** — driven: all five getters `interactive: true`, all `role="listbox"`, `#spaces` click still mutates `selected`
- [x] `svelte-check` reported as **0 err / 34 warn / 15 files** against **0 / 34 / 15**
- [x] 🔓 Skin sibling class wired (`.entity-panel-listitem`); the bulleted-row failure reported openly — **it did not occur** (§3 measured correction)
- [x] D-074: JOURNAL + CLAUDE.md PLAY + ROADMAP + this doc in ONE commit

⚠️ **DoD never includes "commit pushed"** — `Status: COMPLETED` in the header is the shipped signal.

---

## §5 — PROVENANCE, RECORDED HONESTLY

🔒 **Joe ruled it** after asking whether the decision was not already made. ⚠️ **Chat had recommended the opposite** — reimplementing the list inside R7 (~70 lines) to avoid changing `entity-panel` — **having read Leg B §1's *"do not touch entity-panel"* as a statement about COST when it is a statement about SCOPE.** Measurement then showed the `entity-panel` change is **~10 lines**.

🔑 ***The avoidance was seven times the size of the thing avoided,*** and it would have left R7 structurally divergent from every sibling and thrown away when this landed anyway. **The claim was never measured; it was inherited from a boundary and reused as a price.** Same defect class as the rest of the arc, and it took a question from outside the text to catch it.

🔓 **The name `M-RP-PANEL-INERT` is DELEGATED, not a walked lock.**

---

## §6 — IMPLEMENTATION FINDINGS (J-596)

🔑 **THE `<li>` IS TWO STATIC-ROLE BRANCHES, NOT ONE DYNAMIC-ROLE ELEMENT — forced by svelte-check.** The naive edit — a single `<li role={interactive ? 'option' : 'listitem'}>` with all attributes made conditional — **moved the floor to 0/35/16.** The new warning was `a11y_no_noninteractive_tabindex` on the `<li>`: with a *dynamic* role the a11y analyser cannot prove the element is interactive, so a nonnegative `tabindex` reads as "tabindex on a possibly-noninteractive (`listitem`) element". Fixed by splitting into `{#if interactive}` → `<li role="option" …>` `{:else}` → `<li role="listitem">`, each with a **static** role the analyser can check; the shared `EntityItem` body is single-sourced via a `{#snippet}`. **Not a `svelte-ignore` suppression** — the code is made statically correct, so the floor returns to **0/34/15** honestly.

📌 **DEVIATION FROM §1's TOUCHED LIST (Rule 6, flagged):** the DoD requires the inert mode be *"verified in the DOM"*, which needs an inert instance rendered. §1's TOUCHED named only `entity-panel.svelte` + `skin.css`. Clair added **one** inert fixture cell to `ui/sampler/src/app_sampler.svelte` (`entity-panel#inert`, `interactive={false}`, a one-way literal `selected` = R7's data-driven pattern) — the sampler is the core test-bed (D-097) and the five existing cells are the default-mode regression fixture. Catalogue **419 → 427** (+8 = the panel + its section + 3×(entity-item + entity-avatar)); `count === unique === 427`, no orphans.

📌 **The `debug` getter gained an `interactive` field** — additive, so the verify pass reads the mode directly; no behaviour change.
