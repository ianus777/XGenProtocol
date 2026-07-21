# M-RP6.1i — `shelf` core (ordered command strip + faces)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-21  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Build the **`shelf`** — the ordered command strip that frames the widget grid (top = user favourites, bottom = system) — and its leaf, **`shelf-face`**. Two new `core` components, verified in the **sampler (9422)**. **No client mount** (that is M-RP6.1j). **No Rust.**

Design source: `docs/xgen-widget-surfaces-phase0.md` §1–§2 (**S-1…S-8, Joe-locked**) · vocabulary: `ui/docs/xgen-region-dock-model.md` §0 (**N-100, LOCKED**) · taxonomy: **D-112**.

---

## 0. Read first (Rule 0) — and the grounding hedge that keeps earning its keep

**Session-open order:** `CLAUDE.md` PLAY block → latest `JOURNAL.md` entry → any ACTIVE handoff in `tasks/` → **then** this runbook. **A runbook is item 4, never item 1.**

> ### 🔑 **If the code contradicts this document, THE CODE WINS — and you flag it.**
> This hedge has now caught **three** real errors (J-500 §5.2's prop shape · J-501's DM-avatar literal · J-499's vitest path). **Flag deviations; do not absorb them** (Rule 6). **Do not invent numbers** (Rule 5) — every count in the DoD is *measured*, none is pre-booked.

**Vocabulary (N-100).** **tile** = a place · **region** = a widget's full content surface · **face** = a widget's compact **handle** on a shelf · **window** · **slot**. This milestone builds the **strip and the faces**. It builds **no region, no tile, no slot.**

---

## 1. What a shelf is — and the three things it is NOT

```
+---------------------------------------+
| native title bar            (OS)      |
+---------------------------------------+
| menu-bar                (frame chrome)|
+---------------------------------------+
| TOP-SHELF               (frame chrome)|  <- user favourites   (mounts EMPTY at 6.1j)
+---------------------------------------+
|                                       |
| WIDGET GRID    (the Layout descriptor)|  <- dockable, M-RP6.1f
|                                       |
+---------------------------------------+
| BOTTOM-SHELF            (frame chrome)|  <- system commands   (mounts DISABLED at 6.1j)
+---------------------------------------+
| status-bar              (frame chrome)|
+---------------------------------------+
```

- **NOT a grid** (S-3). No `leaf`, no `split`, no `tabs`. A 1-D **ordered strip**. Never give it a `Layout` descriptor — that word drags the whole node vocabulary along with it.
- **NOT a status-bar.** A status-bar is **passive display**; a shelf is an **active command surface** → ARIA **`toolbar`** (roving tabindex + arrow traversal).
- **NOT a menu-bar.** No popups, no dismiss policy, no focus-return. **A shelf has no owned-popup machine** — which is precisely why **N-086's shared-W-2 extraction stays NOT-TRIGGERED** (its trigger is the submenu-flyout or a menu needing the portal, J-496).
- **NOT a second dock** (S-4). A face is **icon + click**, never a shrunken widget. *No panels. No forms. No embedded editors.*

**Placement (S-1):** shelves are **frame chrome, OUTSIDE the `Layout` descriptor** — the same structural argument as the menu-bar (D-107): *the controls that govern the grid must not be dockable by the grid.* **They are not regions and not widgets.**

---

## 2. Decisions (Joe-locked 2026-07-12)

### D1 — Root + ARIA
`shelf` root = **`<div role="toolbar" aria-orientation="horizontal">`**, `aria-label` from a prop. **Roving tabindex** across faces (exactly one face has `tabindex=0`).

### D2 — TWO components: `shelf` + `shelf-face`
The **`menu-bar` / `menu-item` shape**, not the `sb-cell` shape.
- **`shelf`** owns the strip, the item list, `activeIndex`, the face refs, and the keyboard machine.
- **`shelf-face`** is the leaf: its own **native `<button>`** root, composing the `icon` core child, self-registering with its own getter.

**Why the face is its own registered component and not an internal non-registering part (N-064 / `sb-cell`):** a face **carries state** (`disabled`, its `commandId`, its roving `active` flag) and each face must be **independently CDP-readable**. `sb-cell` registers nothing because it is a value-less flex group; a face is not.

**⚠️ Why the face does NOT reuse the `button` core (grounded, `button.svelte`):** `button` renders **`{label}` text only** — no children snippet, no icon child, **no `tabindex`**. A face is icon-only, and its tabindex is **owned by the strip**. The shipped precedent for exactly this is **`menu-trigger`** (inside `menu.svelte`): a chrome component that owns its own `<button>` because roving + ARIA role belong to the parent. **Follow it.**

**Ordinals:** these are the next two `core` components. **Do NOT pre-book a number** — `region-shell` took the 32nd slot at J-499; the ordinal is recorded at close, from the measured registry (Rule 5 applies to counts in designs too, surfaces §7).

### D3 — The keyboard machine: copy `menu-bar`'s LINEAR rove, and nothing else
Grounded in `menu-bar.svelte`: `activeIndex` + a `triggers[]` ref array + **ArrowLeft / ArrowRight / Home / End**, wrap-around, `e.preventDefault()`, `focus()` on the target. **~30 lines.**

**Take that, and take *only* that.** The `menu.svelte` half (open/dismiss/outside-click/focus-return/`$effect` listener lifecycle) has **no shelf counterpart** — a face has no popup. **Do not import it, do not adapt it, do not "leave a seam" for it.**

Also standard toolbar behaviour: **Enter / Space activate the focused face** (the `<button>` does this natively — do not re-implement it, and do not `preventDefault` it).

> **⚠️ FILED, NOT BUILT — M-RP-ROVING.** With this milestone, **roving-tabindex reaches its FOURTH independent implementation**: `entity-panel` (listbox) · `menu-bar` (menubar) · `menu` (popup) · `shelf` (toolbar). **That is D-069's four-recurrence bar, met.** **It is NOT extracted here** — a shared `roving` helper touches **three closed, CDP-verified components** and is its own milestone, not a rider on this one. **Clair: do not extract it. Do not refactor `menu-bar` or `entity-panel`.** Copy the ~30 lines and move on; the duplication is deliberate and recorded.

### D4 — An empty shelf is MOUNTED, not absent
The root **always renders** (N-053: never `{#if}` a registered root — the registry must stay complete). An **`[data-empty]`** attribute reflects `items.length === 0`, and **the skin collapses it**.

**Precedent, shipped:** `status.svelte` (J-464) — *expired → mounted-but-empty*, `data-empty` collapses it via the skin, so `expired:true` stays CDP-readable. **Same mechanism, same reason.**

This is what lets the **top shelf mount empty at 6.1j** (pinning is still open — surfaces §6 ④) without a dead control and without a hole in the registry.

### D5 — Faces dispatch COMMANDS, never a bare `onclick` (S-7)
`shelf` takes `onCommand?: (command: string | undefined) => void` — **byte-for-byte the `menu-bar` seam** (grounded: `app_client.svelte` wires `onCommand={runCommand}` into `commandTable`).

**Consequence, and it is the point:** a shelf button and a menu item become **one command with two triggers** — and an accelerator is free. **`shelf` imports no Tauri, no store, no command table.** It emits a `commandId` and forgets.

### D6 — NO badge this milestone (deferred, and the reason is a scar)
S-4 allows an optional badge. **It is not built.**

**Nothing in the client produces a count**, and **unread counts specifically have NO PROTOCOL MECHANISM** (the read-marker gap, J-503 — *binding: no UI milestone may fake it*). A badge prop with no feeder is **exactly the N-097 shape**: `.entity-item[data-selected]` shipped fully skinned and **unreachable by any client code**, discovered a milestone later.

**Do not add a `badge` prop. Do not add a `.face-badge` skin rule. Do not reserve a slot for one.** *(An unfed branch is an unverified branch — the same rule that keeps `tabs` out of renderer A.)* **Trigger:** the first widget that actually produces a number.

### D7 — Disabled faces are HONEST, not dead (this is what makes 6.1j possible)
At **M-RP6.1j** the bottom shelf mounts with its three faces **`disabled: true`**, because **their commands do not exist yet**: `widget.manager` is M-RP6.1l; `layout.save` / `layout.load` act on the **named UI states** that M-RP6.1k's store creates. *(Grounded: `commandTable` today is `{app.exit, help.about}` — nothing else.)*

**A visibly disabled control is not a dead control — it is an honest phase-limit (W-8).** The `self-panel` precedent: it ships `registered:false` and an explicit *not registered* line rather than faking a state it does not have.

> **BINDING, and each line is a DoD item in the milestone that owns it:**
> **M-RP6.1k DoD** — *`diskette` + `load` flip to enabled and dispatch `layout.save` / `layout.load`.*
> **M-RP6.1l DoD** — *`gear` flips to enabled and dispatches `widget.manager`.*
> **No face is enabled before its command exists, and no milestone closes leaving its own face disabled.** The disabled state is a **countdown**, not a resting state.

**This milestone (6.1i) is the sampler**, where faces are **enabled** and the machine is exercised for real.

---

## 3. Scope — files

| file | change |
|---|---|
| `ui/core/lib/components/data-independent/shelf.svelte` | **NEW** — the strip |
| `ui/core/lib/components/data-independent/shelf-face.svelte` | **NEW** — the leaf |
| `ui/core/lib/components/data-independent/icons.ts` | **+3 glyphs** (§6) |
| `ui/assets/icons/*.svg` | **+3 source SVGs** + provenance (§6) |
| `ui/assets/skin.css` | **+`.shelf*` / `.shelf-face*`** (§5) |
| `ui/sampler/src/app_sampler.svelte` | **+ sampler cells** (§7) |

**OUT OF SCOPE — touch none of these:** `ui/client/**` · `ui/node/**` · any Rust · `menu-bar.svelte` / `menu.svelte` / `menu-item.svelte` / `entity-panel.svelte` (**no roving refactor**) · `status-bar.svelte` / `sb-cell.svelte`. **Prove it with `git show --stat` at close** (the J-497/J-499 discipline — scope is *demonstrated*, not asserted).

---

## 4. Contracts

### 4.1 `shelf.svelte`

```ts
type ShelfItemDef = {
  /** Glyph name (an icons.ts key). Required — a face IS an icon. */
  icon: string;
  /** Accessible name. Required — an icon-only button with no label is unreachable. */
  label: string;
  /** The opaque command id dispatched via onCommand on activate. */
  command: string;
  disabled?: boolean;
};

let {
  items = [],                 // ordered, left→right
  position = 'bottom',        // 'top' | 'bottom'  → reflects to data-position
  ariaLabel,                  // the toolbar's accessible name
  onCommand,                  // (command: string | undefined) => void
  id,
}: { ... } = $props();
```

- `activeIndex` = `$state(0)` — the roving position. **Clamp it**: an empty `items` must not focus anything and must not throw.
- **Skip disabled faces when roving?** **NO.** A disabled toolbar button stays **focusable and reachable** (it is the standard `aria-disabled`-style toolbar behaviour, and at 6.1j it is the *only* thing on the bottom shelf — if roving skipped disabled faces, the bottom shelf would be **entirely unreachable by keyboard**). Roving lands on it; activation is inert.
- Reflect **`data-position`** and **`data-empty`** (D4).
- Root carries `use:envelope` (N-023). **No hardcoded `class`. No component `<style>` block** (N-025 / N-031 / N-090).

**Getter G** (task-state only, N-060):
```ts
const debug = () => ({ position, itemCount: items.length, activeIndex });
```

### 4.2 `shelf-face.svelte`

```ts
let {
  icon,                       // glyph name → <Icon name={icon} .../>
  label,                      // → aria-label (the button has no visible text)
  active = false,             // roving flag, OWNED BY shelf → tabindex 0 / -1
  disabled = false,
  onActivate,                 // () => void — the shelf turns this into onCommand(command)
  ref = $bindable(),          // HTMLButtonElement, for the shelf's programmatic focus
  id,
}: { ... } = $props();
```

Markup shape (the `menu-item` pattern, adapted):
```svelte
<button
  bind:this={ref}
  use:envelope={{ name: 'shelf-face', id, debug }}
  type="button"
  aria-label={label}
  tabindex={active ? 0 : -1}
  aria-disabled={disabled || undefined}
  onclick={() => !disabled && onActivate?.()}
>
  <Icon name={icon} id={id ? `${id}__icon` : undefined} />
</button>
```

- **Use `aria-disabled`, NOT native `disabled` — and this is GROUNDED, not a judgement call: `menu-item.svelte`, the sibling in this same family, already does exactly it** (`aria-disabled={disabled || undefined}` + `onclick={() => !disabled && onSelect?.()}`). **Why it matters here more than anywhere else:** a natively-`disabled` `<button>` **is not focusable**, and at **M-RP6.1j every face on the bottom shelf is disabled** — so native `disabled` would leave the strip with **no focusable element at all**: Tab skips from the menu-bar straight past it, the arrow keys do nothing, and **the shelf does not exist for a keyboard user.** It would also make the roving machine **unverifiable in the real frame** (6.1j's entire job), leaving only the sampler proof — and the sampler is D-097's declared blind spot. **A disabled face must stay focusable so its `aria-label` can be read: the disabled state is a countdown, and a countdown nobody can reach communicates nothing.**
  - Therefore: `aria-disabled={disabled || undefined}` · **guard `onclick`** · **guard Enter/Space** (a non-native-disabled `<button>` activates on both, so the guard is not optional) · skin hooks on **`[aria-disabled="true"]`**, not `:disabled`.
  - **One code path, not two.** The face behaves identically whether or not its siblings happen to be disabled.
- Child ids follow the composite convention: `<id>__icon` (N-054).

**Getter G:**
```ts
const debug = () => ({ command, hasIcon: icon != null, disabled, active });
```
*(Pass `command` down as a prop purely so the face can publish it — it is the thing CDP asserts a click dispatched. The face does **not** call `onCommand` itself; it calls `onActivate`, and the **shelf** owns the mapping face→command. One dispatch path.)*

---

## 5. Skin (`skin.css`, L2 only — N-090: gaps, sizing and layout are skin too)

New keys: `.shelf` · `.shelf[data-position="top"]` · `.shelf[data-position="bottom"]` · `.shelf[data-empty]` · `.shelf-face` · `.shelf-face:hover` · `.shelf-face:focus-visible` · **`.shelf-face[aria-disabled="true"]`** *(not `:disabled` — see §4.2)*.

Assemble from the **existing vocabulary** — introduce **no new token** unless the design genuinely lacks one (and if it does, say so):
- surface `--s2`, hairline `--s5` (the `.menu-bar` / `.status-bar` posture) · icon tint `--t3`, disabled `--t4` · hover ride `--s4` (the `menu-trigger` roving highlight precedent) · radius `--rad` · spacing `--sp-1` / `--sp-2`.
- **Accent-NEUTRAL chrome**, exactly like `.menu-bar` / `.status-bar` / `.separator`: the **only** accent carrier is the **keyboard focus ring** (`--accent2`, gold ↔ blue). **Verify this** — an accent-swap must move the focus ring and *nothing else*.
- Top shelf: `border-bottom: 1px solid var(--s5)`. Bottom shelf: `border-top`. (Mirror of the menu-bar/status-bar edges.)
- Items **right-aligned** (S-3: *"mostly empty, items right-aligned"*) — `justify-content: flex-end` on the strip.
- `.shelf[data-empty]` → collapsed (zero height, no border). **This is what the top shelf looks like at 6.1j.**
- Face box: square, icon-sized; `--ctl-h` (28px) is the shipped control height — use it unless the strip reads wrong, in which case **flag it, don't invent a token silently**.
- Mark the block **PROVISIONAL (Joe live-tunes via HMR)** — the `.menu-bar` / `.status-bar` / `.range` precedent.

---

## 6. 🔒 Glyphs — three new, and I am NOT giving you the path data

`icons.ts` today holds **exactly three** glyphs: `caret-down`, `dot`, `square` (grounded — read it). We need three more:

| name | command it will serve | Material Symbols source |
|---|---|---|
| `gear` | `widget.manager` (6.1l) | `settings` |
| `diskette` | `layout.save` / `layout.saveAs` (6.1k) | `save` |
| `load` | `layout.load` (6.1k) | `folder_open` |

**Rules — all four are hard:**

1. **Source: Material Symbols / Material Icons (Apache-2.0)** — licence-clean, **fill-based**, authored on a **24×24 viewBox**, which is exactly the `icons.ts` contract (grounded: its own header says *"every glyph is authored on a 24x24 viewBox and is FILL-based … so `fill` tinting works"*).
2. **🔑 Take the `d` from the REAL SVG. I have deliberately not supplied path strings, and you must not accept one from memory either — mine or yours.** *(Rule 5, extended: **a path string nobody read is a number nobody measured.** A subtly wrong `d` renders as a plausible-looking blob and passes every check that does not look at it.)*
3. **COLOUR-FREE — this is a SECURITY REQUIREMENT, not tidiness (D-110).** A glyph carries **geometry only**; colour arrives from the tint (`currentColor` / `--icon-tint`). **No `fill="#..."` baked into a path, ever.** A token that fuses colour and geometry makes the Space-theme ban **unenforceable on exactly that glyph**.
4. **Provenance, per glyph (D-108).** Save the source `.svg` into `ui/assets/icons/` and record **licence + source URL + the Material name** in a comment beside each entry in `icons.ts`. The generator + `icons.manifest.json` (where *"a glyph with no licence entry fails the build"*) land at **M-RP-ICON-ADOPT** — **write the provenance now so that migration is a move, not an archaeology dig.**

---

## 7. Sampler cells (`app_sampler.svelte`, **DI Composites** tab)

The `status-bar` precedent (J-494): frame chrome is still built and CDP-verified **in the sampler** first, then mounted in the real client. All panels stay **mounted**, never `{#if}` (N-053).

Cells:
- `shelf#bottom` — `position="bottom"`, the **three real faces** (`gear` / `diskette` / `load`), **all enabled**, `onCommand` → a local `lastCommand = $state(...)` the CDP can read back. *(The sampler is a test-bed: enabled here is correct. Disabled-in-the-client is 6.1j's story, and it is a different, honest one.)*
- `shelf#mixed` — one face **`disabled: true`** among enabled ones (the disabled branch, exercised **here**, so 6.1j inherits a proven state rather than a first-ever one).
- `shelf#top-empty` — `position="top"`, **`items: []`** → the **`[data-empty]`** collapse.

---

## 8. Verify — DoD (sampler 9422, **both accents**)

Single-expression `JSON.stringify({…})` evals only (PS 5.1). **Every count measured.**

1. **Registry** — delta **measured, not predicted**; `count === unique === domCount`; **0 orphans both directions** (the sampler *can* express this leg — unlike the client, N-092a).
2. **Getters** — `shelf#bottom` G `{position:"bottom", itemCount:3, activeIndex:0}`; each `shelf-face#…` G `{command, hasIcon:true, disabled, active}`; the `__icon` children registered.
3. **Roving** — `tabindex` array reads `["0","-1","-1"]` → **ArrowRight** → `["-1","0","-1"]` **and `document.activeElement` is face 1** (focus is the point, not the attribute) → **End** → last → **Home** → first. Wrap-around proven.
   **⚠️ N-099: a set/click and its DOM read are SEPARATE evals, and each leg asserts it can SEE its subject before comparing.** A `null === null` match is a **false pass**.
4. **Dispatch** — click face 0 → the sampler's `lastCommand` reads `"widget.manager"`. Enter on the focused face → same. **The commandId, not the click count.**
5. **Disabled** — the disabled face: activation is **inert** (`lastCommand` unchanged **on both click AND Enter** — the Enter guard is the one a native `disabled` would have given for free, so it is the one most likely to be missed), **and it IS keyboard-reachable**: rove onto it and assert `document.activeElement` is that face. *(This is the leg that protects the bottom shelf at 6.1j, where every face is disabled.)*
6. **Empty** — `shelf#top-empty`: root **present in the registry** (D4/N-053), `[data-empty]` reflected, and **the painted box is collapsed** — `getBoundingClientRect().height === 0`. **⚠️ N-097: the painted pixel is the leg, not the attribute.**
7. **Skin** — all `.shelf*` rules in cascade (stylesheet-rule inspection, N-042), **incl. the `[aria-disabled="true"]` rule**; **zero component `<style>`**; **accent-neutral** — inject the accent swap and prove **only the focus ring** moves (gold `#c28840` ↔ blue `#3a7ab0`).
   *(⚠️ Forward note — **M-RP-FOCUS** (filed 2026-07-12) will move `--focus-ring` off `--accent2` onto a new `--focus` token. When it lands, **this leg becomes “NOTHING moves under an accent swap”** — the shelf becomes **fully** accent-neutral. **That is the fix landing, NOT a regression.** The shelf itself needs no change: it rides the token.)*
8. **Glyphs** — each face's `<svg class="icon">` renders **a real `<path>` with a non-empty `d`**, and `getBBox()` is **non-degenerate** (N-097 generalised: for SVG, `getBBox()` / `getTotalLength()`, **not** `getComputedStyle().d`). *This is the leg that catches a mis-copied path.*
9. **Static gates (apps down):** `vite build` clean · `npm test` clean · `cargo test` **unchanged from baseline 1507/0/62** — *which PROVES the no-Rust claim rather than asserting it*.
10. **Scope** — `git show --stat` = the §3 file list, nothing else.
11. **Eye-check** — screenshot both accents; Joe assesses the strip before close.

---

## 9. Bindings (do not cross these, even if it would be convenient)

- **No client mount.** 6.1j owns it.
- **No badge** (D6). **No `Layout` descriptor, no `leaf`/`split`/`tabs`** (S-3). **No minus button** (S-6 — and it is not an ergonomic preference: a delete-on-a-toolbar would require a **second selection concept**, and killing it is what keeps the D-107 selection bus at **one shape, one meaning**).
- **No roving extraction / no refactor of `menu-bar`, `menu`, or `entity-panel`** (D3 — M-RP-ROVING is filed, not this).
- **No new command** invented in `commandTable`, and **no widget** made removable.
- **The shelf imports no Tauri, no store, no protocol type.** It emits a `commandId`.

---

## 10. Close (D-074)

**Two commits.** Clair: **feat, code only** (the §3 files). Chat: **doc-bridge** (JOURNAL + CLAUDE.md + ROADMAP + `ui/docs/xgen-ui-components.md` registry + any N-note). **Joe pushes both. Chat never pushes.**

Chat **re-drives every non-destructive CDP leg itself** before the record is written (**Rule 5** — *a registry number I did not measure does not enter a canonical record*). Deviations from this runbook are **flagged, not absorbed** — including Chat's own.

---

*Runbook. Design locked by Joe 2026-07-12. Sampler-only; no client, no Rust.*
