# M-RP2.21 — `link` (di·A, atomic `<a href>`, navigation kind)

> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

Author + skin `link` — the **sixteenth** `core` component and the **first NAVIGATION-kind di** (a new
kind alongside interactive and display). Built, tuned, CDP-verified **in the sampler** (D-097). An
atomic native **`<a href>`**: value-carrying (a `text` label) **and** navigational (`href`). Surfaced by
the `status-indicator` composite wanting a trailing "details →" affordance (N-049); now built as a
first-class atomic. Locked at the M-RP2.21 design walk (this session) — N-049 had deferred the prop
surface to build time.

**Joe-locked decisions:**

1. **Own navigation kind.** Not display-di (it acts), not interactive-input (it carries no editable
   value). The di kinds are now interactive / display / **navigation**. Value via a `text` prop (the
   label/paragraph/image precedent — not slotted children, keeps it atomic + registry-serialisable).
2. **`link` IS an `<a>`** — distinct from the existing *button link-styled shape* (a `<button>` that
   only *looks* like a link, acts via `onclick`, no navigation). Never conflate (N-049). The discriminator
   is real navigation (right-click-open works, it's a destination) vs an action.
3. **Prop surface:** `href` (required), `text` (required; `""` allowed for an icon-only link),
   `onclick?` (passthrough — for in-app/SPA routing; consumer `preventDefault`s + routes, the real
   `href` stays for a11y + right-click-open), `external?` (boolean), `disabled?` (boolean), `ariaLabel?`
   (the accessible name for an icon-only link), `id`.
4. **`external` → safe target+rel.** `external={true}` auto-sets `target="_blank"` +
   `rel="noopener noreferrer"` (consumers shouldn't have to remember `noopener`). No raw `target`/`rel`
   props exposed yet (add later only on real need).
5. **`disabled` (no native `<a>` disabled).** `disabled={true}` **drops `href`** (renders a
   non-navigating `<a>`), sets `aria-disabled="true"` + `tabindex="-1"` (non-focusable), and blocks
   `onclick`; the skin greys it. Keeps library-wide disabled consistency + covers "nav target currently
   unavailable".
6. **Getter `{ text, href, external, disabled }`** (the meaningful state; `href` is the prop value even
   when `disabled` drops it from the rendered element — the verify notes the rendered drop separately).
7. **`role="img"`-free** — an `<a>` is already a link role; no role override. Accessible name = visible
   `text`, or `ariaLabel` when text is empty (icon-only). DEV-warn if `text===""` **and** no `ariaLabel`
   (no accessible name) — the `image`-`alt` guard shape.
8. **`.link` skin = ACCENT-DERIVED** (`color: var(--accent2)`, re-themes gold/blue per shell). `link`
   goes **back to the accent pattern** — `led` was the deliberate caller-supplied-colour exception, not
   a new norm. Underline-on-hover, focus ring, `[aria-disabled]` greyed + no underline + default cursor.
   PROVISIONAL.

**Consumer-wiring notes (the atomic stays dumb — never imports Tauri/router):**
- **Leave to the OS browser:** consumer `onclick` → Tauri `shell.open(href)` (a raw `target="_blank"`
  inside a Tauri WebView can spawn a blank in-app webview rather than the system browser). The real
  `href` is retained for a11y/right-click. `external` styles + sets the safe target/rel; the
  *OS-browser* behaviour is the consumer's `onclick` wiring.
- **In-app SPA route:** consumer `onclick` → router (`preventDefault` + navigate); real `href` kept.
- **Open a modal:** **NOT `link`** — a modal has no destination. That is a `button` (or button icon
  shape) flipping the consumer's `open` state. Using `<a href="#">` to open a modal is the anti-pattern
  (fake href, breaks right-click, lies to a screen reader). Logged at close: a future **`modal`/`dialog`**
  component (native `<dialog>` + `showModal()`, focus-trap, `::backdrop`, Esc-to-close), its own build.
- **Icon-only / icon+text:** `ariaLabel` is the atomic hook that makes an icon-only link accessible; the
  glyph itself needs a future **`icon` primitive** (icon+text = an `icon`+`link` composite). NOT faked in
  the sampler — no placeholder glyph invented mid-build.

**Milestone M-RP2.21**, sampler cells `link#default` + `link#external` + `link#disabled`.

---

## 1. Why this shape (for the N-entry)

`link` is the **first navigation-kind di** — it neither binds an editable value (interactive) nor is
purely read-only (display); it *acts* (navigates) while carrying a label. The design tension it resolves
is the `<a>`-vs-`<button>` one (N-049): navigation is an `<a>` with a real `href`; an action that only
looks like a link is a `button` shape. `link` commits to the `<a>`.

Three notable points: (a) `disabled` is **synthesised** (no native `<a>` disabled) by dropping `href` +
`aria-disabled` + `tabindex=-1` — the first component to fake a native-absent state rather than pass one
through; (b) `external` **bundles the safe `rel`** so the unsafe `target="_blank"` default never reaches
a consumer; (c) `link` returns to the **accent-derived** skin after `led`'s caller-supplied-colour
exception — confirming `led` was the one-off, not a turn.

---

## 2. Phase-0 references (read before authoring)

- `ui/core/lib/components/data-independent/label.svelte` — the read-only display-di pattern (plain
  prop, `$state.snapshot` getter, `use:envelope`, no hardcoded class). `link` is the same substrate
  shape minus read-only (it acts).
- `ui/core/lib/components/data-independent/image.svelte` — the **required-value + DEV-warn** guard
  (`alt` required); `link` reuses it for the `text===""` && no-`ariaLabel` no-accessible-name case.
- `ui/core/lib/components/data-independent/button.svelte` — the action-trigger contrast (`onclick`,
  `ariaLabel`, no navigation); `link` is its navigation counterpart. Do not conflate (N-049).
- `ui/assets/skin.css` — `.button` (the link-styled button shape to stay distinct from), `--accent2`
  (the per-shell gold/blue the `.link` colour rides), the focus-ring + disabled treatments.
- `ui/docs/xgen-ui-notes.md` N-049 (the `link` catalogue lock + the navigation row), N-034 (no map
  here — `link` carries a scalar label, not a caller map).
- `DECISIONS.md` D-096 — `link` is a new kind/own atomic, no fold question; applies D-096, no amendment.

---

## 3. Component spec — `ui/core/lib/components/data-independent/link.svelte`

Root IS `<a class="link">` (the type-class via `envelope`, not hardcoded). Zero local `<style>`.

**Props:**

| prop | type | default | note |
|---|---|---|---|
| `href` | `string` | — (required) | the destination; dropped from the rendered element when `disabled` |
| `text` | `string` | — (required) | the visible label; `""` allowed for an icon-only link |
| `onclick` | `(e: MouseEvent) => void` | — | passthrough for in-app/SPA routing or `shell.open`; blocked when `disabled` |
| `external` | `boolean` | `false` | → `target="_blank"` + `rel="noopener noreferrer"` |
| `disabled` | `boolean` | `false` | → drop `href`, `aria-disabled="true"`, `tabindex="-1"`, block `onclick` |
| `ariaLabel` | `string` | — | accessible name (required-in-spirit for icon-only) |
| `id` | `string` | — | |

- **Derived:** `effectiveHref = disabled ? undefined : href`; `target = external ? '_blank' : undefined`;
  `rel = external ? 'noopener noreferrer' : undefined`.
- **DEV guard:** `if (DEV && text === '' && !ariaLabel) console.warn('[xgen link] icon-only link needs ariaLabel (no accessible name).')` — the `image`-`alt` shape.
- **Getter:** `const debug = () => ({ text, href, external, disabled });`
- **Markup:**
  ```svelte
  <a
    use:envelope={{ name: 'link', id, debug }}
    href={effectiveHref}
    {target}
    {rel}
    aria-label={ariaLabel}
    aria-disabled={disabled || undefined}
    tabindex={disabled ? -1 : undefined}
    onclick={disabled ? (e) => e.preventDefault() : onclick}
  >{text}</a>
  ```
- **No `$bindable`** (navigation carries no editable value), no processor seam, **no Tauri/router import**
  (consumer-wired).

---

## 4. Skin spec — add `.link` to `ui/assets/skin.css`

Own `.link` key, placed after `.led`. **Accent-derived; Joe live-tunes via HMR; PROVISIONAL.**

- `.link` — `color: var(--accent2)`, `text-decoration: none`, `cursor: pointer`, inherits `--fs-*`.
- `.link:hover` — `text-decoration: underline`.
- `.link:focus-visible` — the accent focus ring (the L2 `--focus-ring` treatment).
- `.link[aria-disabled]` — greyed (`color: var(--t4)` or the disabled token), `text-decoration: none`,
  `cursor: default`, `pointer-events: none` (belt-and-braces; `href` is already dropped).
- No new `:root` token. (A compact/short shape and an icon shape are FUTURE skin shapes — not built
  here; icon needs the future `icon` primitive.)

---

## 5. Sampler integration (the standing sampler-DoD, D-097)

Add a `link` row to the **Display** section of `ui/sampler/src/app_sampler.svelte` (plain-JS shell, bare
`$state`), after the `led` row (navigation sits naturally with the display group for now). **3** cells:

| cell `id` | props | shows |
|---|---|---|
| `link#default` | `href="#settings"` `text="Settings"` | in-app/in-webview link, accent-coloured |
| `link#external` | `href="https://xgen.example"` `text="xgen.example"` `external` `ariaLabel="XGen site (opens externally)"` | `target=_blank` + safe `rel`; proves `ariaLabel` lands |
| `link#disabled` | `href="#x"` `text="Unavailable"` `disabled` | greyed, `href` dropped, non-focusable |

Matrix **41 → 44**. (Navigation di — no invalid state; icon-only deferred to the future `icon`.)

---

## 6. CDP verification (Chat self-drives — sampler only)

Launch detached minimized; poll 9422 (retry until non-null); **fresh launch** (avoid stale HMR, the
J-430 finding); `cdp-debug.ps1 -App sampler -Mode {state,eval,screenshot}`; teardown (5175/9422 free,
0 orphans).

1. `ids().length === 44`; all three `link#` present.
2. **Registry getters:** `link#default {text:"Settings",href:"#settings",external:false,disabled:false}`;
   `link#external {text:"xgen.example",href:"https://xgen.example",external:true,disabled:false}`;
   `link#disabled {text:"Unavailable",href:"#x",external:false,disabled:true}` (getter carries the prop
   `href` even though the rendered element drops it).
3. **Element attributes (one eval):**
   - `link#default`: `tagName==="A"`, `getAttribute("href")==="#settings"`, `target===null`, `rel===null`,
     `getAttribute("aria-disabled")===null`.
   - `link#external`: `getAttribute("href")==="https://xgen.example"`, `target==="_blank"`,
     `rel==="noopener noreferrer"`, `getAttribute("aria-label")==="XGen site (opens externally)"`.
   - `link#disabled`: `getAttribute("href")===null` (**dropped — the synthesised-disabled proof**),
     `getAttribute("aria-disabled")==="true"`, `tabindex==="-1"`.
4. **Skin (accent-derived — the contrast to `led`):** `.link` + `.link:hover` + `.link:focus-visible` +
   `.link[aria-disabled]` parsed + in cascade; `getComputedStyle(link#default).color` under
   `[data-shell="client"]` vs `="node"` **DIFFERS** (gold ↔ blue) — `link` rides the accent, unlike `led`.
   `link#disabled` colour = the greyed token, `text-decoration-line` none.
5. **Screenshot (eye-check):** three links — accent-coloured "Settings", accent "xgen.example", greyed
   "Unavailable"; flip skin-swap and confirm the accent colour changes (client gold ↔ node blue).

Quote **actual** CDP output in the JOURNAL (Rule 2); never invent (Rule 5).

---

## 7. Records (D-074; written after verification, Rule 4)

- `ui/docs/xgen-ui-notes.md` — **N-052** (build + the first navigation-kind di + the `<a>`-vs-`<button>`
  commit + synthesised `disabled` + bundled-safe `external` rel + the return to accent-derived colour +
  the consumer-wiring notes: `shell.open` external-to-OS, SPA route, modal=button).
- `docs/ROADMAP.md` — M-RP2.21 ✅ (RP node + Present narrative); version bump; same-commit with CLAUDE.
- `CLAUDE.md` — PLAY → M-RP2.21; prior-PLAY pointer → J-432. Di queue → `status-indicator` (di composite;
  `led` + `label` + optional `link` now all in hand) → text-processor engine → dd.
- `ui/docs/xgen-ui-components.md` — promote `link` from Planned to built (navigation catalogue row + a
  Build-note); **add a `modal`/`dialog` entry to the dd/Composites planned section** (native `<dialog>`,
  the modal surface — trigger is `button`, surface is `dialog`); `status-indicator` stays Planned.
- `JOURNAL.md` — **J-432** (newest-first; real CDP output).
- `tasks/M_RP2_21_link.md` — Status → COMPLETED.
- **No `DECISIONS.md` touch** — a new di kind/own atomic; applies D-096, no amendment.

`.md` header rule: `> **Last updated**:` carries ONLY the date.

---

## 8. Commit plan (two commits, UI pattern `feat` → `docs`; Joe pushes)

**Commit 1 — implementation** (`link.svelte`, `skin.css`, `app_sampler.svelte`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/core/lib/components/data-independent/link.svelte
git add ui/assets/skin.css
git add ui/sampler/src/app_sampler.svelte
git status
git commit -m "feat(ui): link - sixteenth core component, first navigation-kind di, atomic <a href> (M-RP2.21)" -m "Value-carrying (text) AND navigational (href). Props href/text/onclick?/external?/disabled?/ariaLabel?/id. external -> target=_blank + rel=noopener noreferrer (safe rel bundled). disabled -> drop href + aria-disabled + tabindex=-1 + block onclick (synthesised, no native <a> disabled). Getter {text,href,external,disabled}; DEV-warn when text='' and no ariaLabel. Distinct from the button link-styled shape (link IS an <a>); the atomic never imports Tauri/router - shell.open/SPA-route/modal are consumer wiring (modal = button, not link)." -m ".link skin is ACCENT-derived (color var(--accent2), re-themes gold/blue) - link returns to the accent pattern, led was the caller-supplied-colour exception. Built/tuned/CDP-verified in the sampler (D-097): link row + 3 cells (matrix 41->44), attribute checks (external target/rel, disabled drops href + tabindex=-1 + aria-disabled), and the accent-swap colour delta client<->node."
git push
```

**Commit 2 — records** (`xgen-ui-notes.md`, `ROADMAP.md`, `CLAUDE.md`, `xgen-ui-components.md`,
`JOURNAL.md`, `M_RP2_21_link.md`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/docs/xgen-ui-notes.md
git add docs/ROADMAP.md
git add CLAUDE.md
git add ui/docs/xgen-ui-components.md
git add JOURNAL.md
git add tasks/M_RP2_21_link.md
git status
git commit -m "docs(ui): close M-RP2.21 link - N-052, J-432, records" -m "N-052 (first navigation-kind di + <a>-vs-<button> commit + synthesised disabled + bundled-safe external rel + return to accent-derived colour + consumer-wiring notes: shell.open / SPA route / modal=button) + ROADMAP M-RP2.21 done + CLAUDE PLAY -> M-RP2.21 + components registry link promoted + modal/dialog logged as a future component." -m "Di queue now reads status-indicator (di composite; led + label + link all in hand) -> text-processor -> dd. Task -> COMPLETED. No DECISIONS touch (new di kind/own atomic; applies D-096)."
git push
```

---

## 9. Definition of Done

- [x] `link.svelte` authored to §3 (`href`/`text`/`onclick?`/`external?`/`disabled?`/`ariaLabel?`/`id`;
      `effectiveHref`/`target`/`rel` derived; synthesised disabled; getter `{text,href,external,disabled}`;
      DEV-warn no-accessible-name; no Tauri/router import).
- [x] `.link` accent-derived skin added to §4 (`var(--accent2)`, hover underline, focus ring,
      `[aria-disabled]` greyed).
- [x] Sampler row + `default`/`external`/`disabled` cells added (§5) and live (matrix 41→44).
- [x] CDP verification §6 run in the sampler — actual output captured (incl. the disabled-drops-href
      proof + the external target/rel + the accent-swap colour delta).
- [x] N-052 written; ROADMAP + CLAUDE updated same-commit; components registry `link` promoted +
      `modal`/`dialog` logged; JOURNAL J-432 written (real CDP output).
- [x] Task Status → COMPLETED.

(`Status: COMPLETED` is the real signal — no "commit pushed" checklist item.)
