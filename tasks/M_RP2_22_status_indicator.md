# M-RP2.22 — `status-indicator` (di·composite; `<div class="status-indicator">` = led + label + optional link)

> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-30  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

Author + skin `status-indicator` — the **seventeenth** `core` component and the **FIRST di
composite**. Root is `<div class="status-indicator">` (composite identity per N-020/N-022: a
`<div class="type">` wrapper composing real child atomics, vs an atomic's native root tag). It composes
the three already-built constituents — **`led`** (required) + **`label`** (required) + an **optional
trailing `link`** — into the general status-row. Binding none; **di** (the caller supplies the
state→colour map, the caption, the link target — the composite interprets no domain structure, N-049).
Built + CDP-verified **in the sampler** (D-097), dropping into the **DI·composite** panel stood up at
M-RP3.2 (J-433). It founds the **composite build pattern** (how `envelope`/debug and child composition
work for a composite) — the precedent for `password-field` / `color-picker` / `file-field` /
`combobox` / `tag-select` / `star-rating`.

**Joe-locked decisions (SI-1…SI-6, this session):**

1. **API — flat pass-through, link as an optional group.** `id`; led → `states` / `state` / `pulse?`;
   label → `caption` (named to avoid colliding with the link's text); link (optional, rendered iff
   `linkHref` set) → `linkHref?` / `linkText?` (default `"Details →"`) / `linkExternal?` /
   `onLinkClick?`. Flat mirrors how a panel author thinks per row.
2. **Registration model — composite aggregate + children self-register under stable ids** (grounded:
   `led`/`label`/`link` each pass their `debug` getter unconditionally and accept `id`, so `envelope`
   registers each child whenever mounted, keyed `id ?? ordinal`). The composite root registers an
   aggregate getter; the children are the **real** atomic components, given composite-derived **stable
   ids** `<id>__led` / `<id>__label` / `<id>__link`. **Zero changes to the three closed atomics** (D-065
   — don't retrofit built components for a new one's convenience); genuinely composed *of* the atomics;
   richer verify (aggregate + each constituent). **SI-1 refinement:** the aggregate getter is
   `{ state, caption, hasLink }` — **`colour` dropped** from it (the composite would otherwise duplicate
   `led`'s `?? "#000000"` sentinel; colour is verified on the `led#…__led` child entry instead — no
   logic duplication, no drift).
3. **Optional link + wiring.** The `<Link>` is rendered **iff `linkHref` is provided** (a genuine
   `{#if linkHref}` — the link sub-element legitimately does not exist when there's no target; this is
   NOT the N-053 panel-mounting case, which is about registry completeness across *tabs*). The composite
   forwards `linkExternal`/`onLinkClick` so the consumer wires OS-browser (`shell.open`) vs SPA route —
   the composite stays dumb (link's N-052 consumer-wiring contract intact). `linkText` default
   `"Details →"`.
4. **Skin (`.status-indicator`, PROVISIONAL).** Flex row, `align-items:center`, `gap`; led + label
   left, the optional link pushed right (`margin-left:auto`) so a panel of N rows aligns its trailing
   links. Assemble from L2; **no new `:root` token if avoidable**. Combined accent proof in one
   component: the link rides `--accent2` (gold↔blue), the led does NOT (caller colour) — verifiable
   side-by-side.
5. **Path + boundary.** `ui/core/lib/components/data-independent/status-indicator.svelte` (di; binding
   none). **Sampler-only** (D-097); migrating the shells' bespoke `.state-dot`+`dotColor`/`isPulsing`
   onto the minimal form (led+label, no link) is a *later* shell step, **not** this milestone.
6. **Sampler + verify.** Three cells in the **DI·composite** panel — `#default` (led+label, no link),
   `#withlink` (led+label+external link), `#pulse` (pulsing led + label + in-app link). **Matrix
   accounting changes** (the headline of the first composite): a composite row registers the composite
   **plus** each child, so the count is no longer 1-per-cell. 3 cells → **11 new registry entries**
   (3 composite + 8 children) → matrix **44 → 55**.

**Milestone M-RP2.22.**

---

## 1. Why this shape (for the N-entry)

`status-indicator` is the **first di composite** — it founds the pattern every later composite reuses.
Three notable firsts: (a) the **composite root** is `<div class="status-indicator">` (the N-020/N-022
`<div class="type">` shape) rather than a native tag — the structural marker that distinguishes
composite from atomic; (b) the **composite-registration model** — the composite registers one aggregate
getter while its **real child atomics self-register** under composite-supplied stable ids, so `ids()`
carries both the row's identity and each constituent (no atomic was modified to achieve this — the
children already register unconditionally); (c) it **multiplies the matrix count** — the first time a
single sampler cell yields more than one registry entry, which the verify and all future composite
accounting must reflect.

It also proves the di/composite quadrant the M-RP3.2 panel reserved is real: a component that interprets
**no** domain structure (the caller hands it the state→colour map, the caption, the link) yet is built
from multiple atomics — the definition of a di composite.

---

## 2. Phase-0 references (read before authoring)

- `ui/core/lib/components/data-independent/led.svelte` — child 1 (required). Props `states`/`state`/
  `pulse?`/`id`; getter `{state, colour}`; resolves `colour = states[state] ?? "#000000"`. Passes its
  `debug` getter unconditionally → self-registers when mounted.
- `ui/core/lib/components/data-independent/label.svelte` — child 2 (required). Prop `text`/`id`; getter
  `{text}`. The composite feeds `caption` → the child's `text`.
- `ui/core/lib/components/data-independent/link.svelte` — child 3 (optional). Props `href`/`text`/
  `onclick?`/`external?`/`disabled?`/`ariaLabel?`/`id`; getter `{text,href,external,disabled}`; accent-
  derived skin (`var(--accent2)`). The composite forwards `linkHref`/`linkText`/`linkExternal`/
  `onLinkClick`.
- `ui/common/lib/components/base/envelope.ts` — registration fires on the `use:` action whenever a
  `debug` getter is passed, keyed `id ?? ordinal`. Basis for decision 2 (children self-register; stable
  ids make that clean).
- `ui/docs/xgen-ui-notes.md` N-049 (status-indicator catalogue lock + the di kinds), N-051 (led), N-052
  (link), N-053 (the DI·composite panel this drops into).
- `ui/docs/xgen-ui-components.md` — the composite schema (`<div class="type">` root + `├──`/`└──`
  members) + the planned `status-indicator` row to promote.
- `DECISIONS.md` D-096 — composite, not a fold question; applies, no amendment.

---

## 3. Component spec — `ui/core/lib/components/data-independent/status-indicator.svelte`

Root IS `<div class="status-indicator">` (the type-class via `envelope`, N-023). Zero local `<style>`.
Imports the three real child atomics.

**Props:**

| prop | type | default | note |
|---|---|---|---|
| `id` | `string` | — | composite id; children take `<id>__led` etc. |
| `states` | `Record<string,string>` | `{}` | → `led` (caller's state→colour map) |
| `state` | `string` | — | → `led` |
| `pulse` | `boolean` | `false` | → `led` |
| `caption` | `string` | `''` | → `label` text |
| `linkHref` | `string` | — | presence renders the trailing link |
| `linkText` | `string` | `'Details →'` | → `link` text |
| `linkExternal` | `boolean` | `false` | → `link` external (safe target+rel) |
| `onLinkClick` | `(e: MouseEvent) => void` | — | → `link` onclick (consumer wiring) |

- **Child id helper:** `const cid = (s) => id ? `${id}__${s}` : undefined;`
- **Aggregate getter:** `const debug = () => ({ state: state ?? null, caption, hasLink: !!linkHref });`
  (colour intentionally omitted — read it on the `led#…__led` child entry).
- **Markup:**
  ```svelte
  <div use:envelope={{ name: 'status-indicator', id, debug }}>
    <Led {states} {state} {pulse} id={cid('led')} />
    <Label text={caption} id={cid('label')} />
    {#if linkHref}
      <Link href={linkHref} text={linkText} external={linkExternal} onclick={onLinkClick} id={cid('link')} />
    {/if}
  </div>
  ```
- **No `$bindable`** (binding none), **no processor seam**, **no Tauri/router import** (the optional
  link's wiring is the consumer's, forwarded through).

---

## 4. Skin spec — add `.status-indicator` to `ui/assets/skin.css`

Own `.status-indicator` key, placed after `.link`. **PROVISIONAL; Joe live-tunes via HMR.**

- `.status-indicator` — `display: flex; align-items: center; gap: var(--sp-2)` (or a literal until a
  spacing token fits); inherits type from context.
- `.status-indicator > .link` — `margin-left: auto` (pushes the trailing link to the row end so a column
  of rows aligns).
- No new `:root` token if avoidable. The led/label/link each already carry their own skin; the composite
  rule only lays them out.

---

## 5. Sampler integration (D-097) — populate the DI·composite panel

In `ui/sampler/src/app_sampler.svelte`: **replace the DI·composite panel's empty-state** with a
`.sampler-body` holding a `status-indicator` row (the DD·atomic / DD·composite panels keep their
empty-states). Import `StatusIndicator`; reuse the existing `ledStates` map
(`{ ON:'#22c55e', OFF:'var(--t4)', ERR:'var(--err)' }`). **3** cells:

| cell `id` | props | shows |
|---|---|---|
| `status-indicator#default` | `states={ledStates}` `state="ON"` `caption="Connected"` | led + label, no link |
| `status-indicator#withlink` | `state="OFF"` `caption="Disconnected"` `linkHref="https://xgen.example/status"` `linkText="Status page"` `linkExternal` | led + label + external link (right-pushed) |
| `status-indicator#pulse` | `state="ERR"` `pulse` `caption="Error"` `linkHref="#logs"` `linkText="View logs →"` | pulsing led + label + in-app link |

**Matrix accounting (first composite — count is no longer 1-per-cell):** `#default` = composite + led +
label (3); `#withlink` = composite + led + label + link (4); `#pulse` = composite + led + label + link
(4). **+11 entries → matrix 44 → 55.**

---

## 6. CDP verification (Chat self-drives — sampler only)

Launch detached minimized; poll 9422 (retry until non-null); **fresh launch** (avoid stale HMR); split
`.click()` from the DOM read by a tick (the J-433 reactive-flush finding); `cdp-debug.ps1 -App sampler
-Mode {state,eval,screenshot}`; teardown (5175/9422 free, 0 orphans).

1. **Count:** `ids().length === 55`. All three `status-indicator#…` present; `led#default__led`,
   `label#default__label`, `led#withlink__led`, `label#withlink__label`, `link#withlink__link`,
   `led#pulse__led`, `label#pulse__label`, `link#pulse__link` present.
2. **Composite aggregate getters:** `status-indicator#default {state:"ON",caption:"Connected",hasLink:false}`;
   `#withlink {state:"OFF",caption:"Disconnected",hasLink:true}`; `#pulse {state:"ERR",caption:"Error",hasLink:true}`.
3. **Child getters (composition proof):** `led#default__led {state:"ON",colour:"#22c55e"}`;
   `label#default__label {text:"Connected"}`; `link#withlink__link {text:"Status page",href:"https://xgen.example/status",external:true,disabled:false}`.
4. **Link-iff-href:** no `link#default__link` in `ids()` and the `#default` row's `.status-indicator`
   has no `<a>` child; `#withlink`/`#pulse` rows each have one `<a class="link">`.
5. **DOM root:** each `status-indicator#…` element is a `DIV` with class `status-indicator`, containing
   a `span.led` + a `label` + (when present) an `a.link`, in that order.
6. **Skin (switch to the DI·composite tab first so the panel is visible):** `.status-indicator` +
   `.status-indicator > .link` parsed + in cascade; the row computed `display:flex`/`align-items:center`;
   the trailing link computed `margin-left` resolves to the auto-pushed offset. **Combined accent proof:**
   `getComputedStyle(link#withlink__link).color` DIFFERS client↔node (gold↔blue) while
   `getComputedStyle(led#default__led)`'s `--led-colour`/background is identical client↔node (caller
   colour, not accent).
7. **Screenshot (eye-check):** the DI·composite tab now shows three rows — green "Connected" (no link),
   grey "Disconnected" + right-aligned "Status page" link, pulsing red "Error" + "View logs →"; flip the
   skin-swap and confirm the links re-theme while the leds do not.

Quote **actual** CDP output in the JOURNAL (Rule 2); never invent (Rule 5).

---

## 7. Records (D-074; written after verification, Rule 4)

- `ui/docs/xgen-ui-notes.md` — **N-054** (the first di composite; the composite root `<div class="type">`
  shape; the composite-registration model — aggregate getter + children self-register under stable ids,
  zero atomic changes; the matrix-multiplication accounting; the SI-1 colour-omission refinement; the
  `{#if linkHref}` vs N-053 distinction). Version bump.
- `ui/docs/xgen-ui-components.md` — promote `status-indicator` from Planned to built (the **first
  composite** registry row + a build-note; render the composite schema `<div class="status-indicator">`
  ├── led ├── label └── link?). Version bump.
- `docs/ROADMAP.md` — M-RP2.22 ✅ on the RP node + Present narrative; version bump; same-commit with CLAUDE.
- `CLAUDE.md` — PLAY → M-RP2.22; prior-PLAY pointer → J-434. Next-active → `select multiple`… (already
  built) / the text-processor engine → dd-components / further composites.
- `JOURNAL.md` — **J-434** (newest-first; real CDP output incl. the 55-count + the child-composition reads).
- `tasks/M_RP2_22_status_indicator.md` — Status → COMPLETED.
- **No `DECISIONS.md` touch** — a composite, applies D-096 (no amendment); the composite-registration
  model is recorded as N-054 (a D-069 promotion-watch when the second composite reuses it).

`.md` header rule: `> **Last updated**:` carries ONLY the date.

---

## 8. Commit plan (two commits, UI pattern `feat` → `docs`; Joe pushes)

**Commit 1 — implementation** (`status-indicator.svelte`, `skin.css`, `app_sampler.svelte`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/core/lib/components/data-independent/status-indicator.svelte
git add ui/assets/skin.css
git add ui/sampler/src/app_sampler.svelte
git status
git commit -m "feat(ui): status-indicator - seventeenth core component, the FIRST di composite (M-RP2.22)" -m "Root <div class=status-indicator> composing led (required) + label (required) + optional trailing link. Flat pass-through props: states/state/pulse? -> led, caption -> label, linkHref?/linkText?/linkExternal?/onLinkClick? -> link (rendered iff linkHref). di - the caller supplies the state->colour map, caption, link; the composite interprets no domain structure. Composite registers an aggregate getter {state,caption,hasLink}; the real child atomics self-register under composite-supplied stable ids (<id>__led/__label/__link) - zero changes to the three closed atomics. Colour omitted from the aggregate (verified on the led child, no sentinel duplication)." -m "Skin .status-indicator = flex row, trailing .link pushed right (margin-left:auto); PROVISIONAL. Built/CDP-verified in the sampler DI-composite panel (D-097): 3 cells, matrix 44->55 (first composite multiplies the count - composite + children per row), aggregate + child getters, link-iff-href, combined accent proof (link rides --accent2 gold<->blue, led does not)."
git push
```

**Commit 2 — records** (`xgen-ui-notes.md`, `xgen-ui-components.md`, `ROADMAP.md`, `CLAUDE.md`,
`JOURNAL.md`, `M_RP2_22_status_indicator.md`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/docs/xgen-ui-notes.md
git add ui/docs/xgen-ui-components.md
git add docs/ROADMAP.md
git add CLAUDE.md
git add JOURNAL.md
git add tasks/M_RP2_22_status_indicator.md
git status
git commit -m "docs(ui): close M-RP2.22 status-indicator - N-054, J-434, records" -m "N-054 (the first di composite: <div class=type> root + the composite-registration model - aggregate getter + children self-register under stable ids, zero atomic changes - + the matrix-multiplication accounting + the SI-1 colour-omission refinement + the {#if linkHref} vs N-053 distinction) + components registry status-indicator promoted (first composite row + schema) + ROADMAP M-RP2.22 done + CLAUDE PLAY -> M-RP2.22." -m "Founds the composite build pattern (precedent for password-field/color-picker/file-field/combobox/tag-select/star-rating). Task -> COMPLETED. No DECISIONS touch (composite, applies D-096; registration model recorded as N-054, D-069 promotion-watch at the second composite)."
git push
```

---

## 9. Definition of Done

- [ ] `status-indicator.svelte` authored to §3 (`<div class="status-indicator">` root; flat props;
      `cid()` stable child ids; imports + composes the real `Led`/`Label`/`Link`; `{#if linkHref}` for
      the optional link; aggregate getter `{state,caption,hasLink}`; no `$bindable`/processor/Tauri import).
- [ ] `.status-indicator` skin added to §4 (flex row, `> .link` `margin-left:auto`); no new `:root` token.
- [ ] DI·composite panel empty-state replaced with the `status-indicator` row + 3 cells (§5); DD panels
      keep their empty-states; matrix 44→55.
- [ ] CDP verification §6 run in the sampler — actual output captured: `ids().length===55`, aggregate +
      child getters, link-iff-href, DOM root composition, `.status-indicator` skin in cascade, the
      combined accent proof (link swaps, led doesn't).
- [ ] N-054 written; components registry `status-indicator` promoted (first composite row + schema);
      ROADMAP + CLAUDE updated same-commit; JOURNAL J-434 written (real CDP output).
- [ ] Task Status → COMPLETED.

(`Status: COMPLETED` is the real signal — no "commit pushed" checklist item.)
