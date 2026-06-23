# XGen UI — M-RP2.6 Runbook: `button` retrofit + `toggle` semantic shape
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-23  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## Purpose

Execute **M-RP2.6** — the first reopen of shipped `core` components (N-030), purely **additive** and **skin-free**. Two files change: `button.svelte` gains `ariaLabel` + a `pressed`/toggle-mode; `toggle.svelte` gains a semantic `shape`. No `$common` change, no protocol/data change, no appearance (all shapes/looks are M-RP2.7 skin). Arc locked at J-409; decisions locked by Joe (this session). No code was written before this runbook (D-071).

Entry order (Rule 0): CLAUDE PLAY → JOURNAL J-409 → this runbook → `ui/docs/xgen-ui-notes.md` (N-030 design, N-031 CSS stack) → `ui/docs/xgen-ui-components.md`.

## Locked decisions (do not re-litigate)

1. **Momentary down-state = CSS `:active`, not a JS pulse.** The inner `pressed` bool is driven only in toggle-mode; momentary leaves it `false`. (Refines N-030's "momentary pulses the bool" — delegated to `:active`, which is M-RP2.7 skin; keeps `pressed` a stable, self-redumpable observable.)
2. **`aria-checked` reflected in switch-shape** on `toggle` (projection of the same `checked` bool — N-030 §4).
3. **Verification wiring:** reuse the existing `toggle#demo` as the switch proof (set `shape="switch"`) + add ONE throwaway toggle-mode `button#demo-toggle` for the `pressed`-latch proof. Net +1 shell element.

All other N-030 retrofit points hold: `ariaLabel`→`aria-label`; `mode` (`momentary` default / `toggle`); `pressed` `$bindable`, latched in toggle-mode; `aria-pressed` only in toggle-mode; getter gains `pressed`; `toggle` `role="switch"` only in switch-shape; Quit/Shut-Down stay momentary, untouched.

## Implementation

### Step 1 — `ui/core/lib/components/data-independent/button.svelte`

Additive prop surface and one toggle branch. Leave the comment block; extend it with one line noting the M-RP2.6 retrofit.

- **Props** — add to the `$props()` destructure and its type:
  - `ariaLabel?: string`
  - `mode?: 'momentary' | 'toggle'` (default `'momentary'`)
  - `pressed?: boolean` as `$bindable(false)`
- **State/logic:**
  - Keep `clicks` `$state(0)`.
  - `handleClick`: `clicks += 1; if (mode === 'toggle') pressed = !pressed; onclick?.();`
- **Debug getter:** `const debug = () => $state.snapshot({ clicks, disabled, pressed });`
- **Template** (root `<button>`):
  - `aria-label={ariaLabel || undefined}`
  - `aria-pressed={mode === 'toggle' ? pressed : undefined}`
  - Keep `{disabled}`, `onclick={handleClick}`, `use:envelope={{ name: 'button', id, debug }}`, `{label}` body.

### Step 2 — `ui/core/lib/components/data-independent/toggle.svelte`

- **Props** — add `shape?: 'checkbox' | 'switch'` (default `'checkbox'`).
- **Getter:** unchanged (`{ checked }` — `shape` is a static prop, not state).
- **Template** (root `<input type="checkbox">`):
  - `role={shape === 'switch' ? 'switch' : undefined}`
  - `aria-checked={shape === 'switch' ? checked : undefined}`
  - Keep `bind:checked`, `use:envelope={{ name: 'toggle', id, debug }}`.
- Extend the comment block with one line on the M-RP2.6 `shape` addition (switch = `role`/`aria-checked` now; visual switch = M-RP2.7 skin keyed on `[role="switch"]`).

### Step 3 — throwaway demos in both shells

In `ui/client/src/app_client.svelte` and `ui/node/src/app_node.svelte`:

- Set `shape="switch"` on the existing `<Toggle … id="demo" />` (the switch/role proof on the existing element).
- Add one throwaway `<Button mode="toggle" bind:pressed={demoPressed} id="demo-toggle" />` (declare `let demoPressed = $state(false);`). Label optional (`label="toggle"`).
- Do NOT touch the real Quit (`button#quit`) / Shut-Down (`button#shutdown`) buttons — they pass no `mode`, stay momentary.

## Verification (CDP self-drive, both apps — Chat working mode N-028)

Per the locked loop: launch detached (`Start-Process run-{client,node}.ps1 -Debug -WindowStyle Minimized`), poll port, **retry `snapshot()` until non-null** (N-028 race), dispatch real events, re-dump, clean up all ports (9222/9322/5173/5174). Driving requires a real dispatched event (N-029).

1. **Baseline dump** (both apps) shows: `toggle#demo` `{checked:false}`, `button#demo-toggle` `{clicks:0,disabled:false,pressed:false}`, plus `textfield#demo` and the real `button#quit`/`button#shutdown`.
2. **Pressed-latch proof (headline):** dispatch a real `click` on `button#demo-toggle` → re-dump → `{clicks:1,disabled:false,pressed:true}`. Second click → `{clicks:2,…,pressed:false}`. This lands the **event-driven self-redump** the terminal Quit/Shut-Down could not (N-028 finding 1).
3. **Switch/role proof:** DOM for `toggle#demo` carries `role="switch"` and `aria-checked="false"`; dispatch a real `change`/click → re-dump `{checked:true}` and DOM `aria-checked="true"`; `role="switch"` persists.
4. **Momentary regression:** `button#quit` / `button#shutdown` carry no `aria-pressed`; clicking still closes the window.
5. Quote ACTUAL dump output in the J-410 close entry (Rules 1/2/5). Ports cleaned, zero orphans.

## Out of scope (M-RP2.7, next)

Stand up the CSS source stack (N-031): `xgen-normalize.css` + the first `skin.css`; render button text/icon · toggle checkbox/switch · the pressed bevel (`[aria-pressed="true"]`); reconcile the global `input{}`/`button{}` wrinkle; found the L2 token vocabulary. Skin keys on `[role="switch"]` (no `data-shape` needed). The icon shape needs `ariaLabel` set by its caller — exercised when the first icon-button skin lands.

## Definition of Done

- [x] `button.svelte`: `ariaLabel`, `mode`, `pressed` ($bindable) added; `handleClick` toggle-branch; getter = `{clicks,disabled,pressed}`; `aria-label` + `aria-pressed` (toggle-mode-only) on root.
- [x] `toggle.svelte`: `shape` added; `role="switch"` + `aria-checked` (switch-shape-only) on root; getter unchanged.
- [x] Both shells: existing `toggle#demo` set to `shape="switch"`; one throwaway `button#demo-toggle` (mode toggle, `bind:pressed`) added; real Quit/Shut-Down untouched.
- [x] Static gate clean in both shells — no `svelte-check`/`tsc` in this toolchain (plain-JS shells), ran as `vite build` (Svelte compiler over every module): 119 modules each, 0 errors / 0 warnings.
- [x] CDP baseline dump reproduced in BOTH apps (J-410: all four components incl. `button#demo-toggle` with `pressed`).
- [x] Pressed-latch delta proven on `button#demo-toggle` (client `clicks 4→5→6` / `pressed false→true→false`; node `2→3→4` same flip — actual output in J-410).
- [x] Switch proof: `toggle#demo` DOM `role="switch"` persists; `aria-checked` reflects `checked` (false→true, matches registry) — both apps, actual output in J-410.
- [x] Momentary regression: Quit/Shut-Down carry no `aria-pressed` (DOM-verified both apps: `ariaPressed:null, hasAttr:false`); close-on-click path untouched by the retrofit (proven M-RP2.4/J-405; not re-clicked — terminal).
- [x] Ports cleaned (9222/9322/5173/5174 = closed), zero `xgen-client`/`xgen-node` orphans.
- [x] Canonical close (D-074): JOURNAL J-410; CLAUDE PLAY (M-RP2.6 ✅ → M-RP2.7 next); `xgen-ui-components.md` (button getter `{clicks,disabled,pressed}`, shape-family prose → built, v0.10); `docs/ROADMAP.md` RP node (v3.90); this runbook → COMPLETED.

> **Note:** "commit pushed" is intentionally NOT a DoD item — it is unflippable inside the commit that performs it (Joe pushes). `Status: COMPLETED` in the header is the real completion signal.
