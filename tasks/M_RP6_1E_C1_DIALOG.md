# M-RP6.1e-C1 — `dialog` core (native `<dialog>` + `showModal()`) build runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

For Clair. **First of three steps** in M-RP6.1e-C (Phase-0 J-488 / D-107 / `docs/xgen-client-frame-phase0.md` §4.1, §6, §10.4; the C-split locked this session). Per-component design **locked by Joe** ("lock", this session).

**The C-split (Joe-locked):**

- **C1 — `dialog` `core`** ← *this runbook*. Component only, **no Rust**. Verify **sampler 9422**.
- **C2 — `get_about_info`** — `xgen-common::about` + `build.rs` metadata + the Tauri read command. Verify **real client 9222**.
- **C3 — Help→About assembly** — Help menu + dialog mount + logo assets + the N-086 verdict. Verify **real client 9222**.

C1 ships **no About, no Help menu, no Rust**. It is the reusable modal primitive that C3 consumes. Design captured here; **no code at lock time** (Rule 1/5).

---

## 1. Goal

Build `dialog` — the **31st `core`** component, a **di composite** modal container, flagged as a gap since J-432.

The headline: the **native `<dialog>` element + `showModal()`** supplies top-layer stacking, `::backdrop`, focus trap, background inert, and Esc-dismiss **for free**. This is precisely why `dialog` does **not** need the W-2 owned-popup behaviour machine (`combobox` / `tag-select` / `color-picker` / `entity-context-menu` / `menu` all hand-roll one because no native element gave it to them). Do **not** build a machine here.

---

## 2. Locked design

### 2.1 Root + tier (axis A)

- **Root** `<dialog class="dialog" use:envelope>` — native element.
- **Tier:** `core`, **di composite**. It composes a real `button` child (the Close button) which self-registers as `…__close` → a `dialog` cell yields **multiple** registry entries (the N-054 composite-multiply precedent).
- **Precedent:** `section` (M-RP2.31) — native root + header + `children` body slot. Same shape.

### 2.2 Modality (axis B) — `showModal()` only

- **v1 is modal-only.** Open = `el.showModal()`. Close = `el.close()`.
- **No `modal` prop.** A prop with exactly one legal value is a lie (D-065). Non-modal (`show()`, no backdrop, no trap) is **deferred** until a real consumer asks.
- **Never set the `open` *attribute*.** `<dialog open>` is the **non-modal** path — no top layer, no `::backdrop`, no focus trap. Setting it would silently give us a worse dialog that *looks* like it works. The attribute is native-owned output; the component drives the element through its **methods** only.

### 2.3 Prop surface (axis C)

| Prop | Type | Notes |
|---|---|---|
| `title` | `string` | header text |
| `open` | `boolean` **$bindable** | see §2.4 — the load-bearing axis |
| `closeLabel?` | `string` | default **`"Close"`** (Joe: *not* "OK") |
| `onClose?` | `() => void` | fired after the dialog has closed (any path: button, Esc) |
| `children` | snippet | the body slot |
| `id` | `string` | envelope id |

**Structure:** header (`title`) → `children` body slot → footer holding the single Close `button` (id `…__close`).

**Deferred (D-065):** multi-action footers (OK / Cancel / Apply), a `footer` snippet override, non-modal, backdrop-click dismiss (§2.6), draggable/resizable dialogs. About needs **one** button; inventing a footer-actions API with no second consumer is speculative design.

### 2.4 Open-state reconciliation (axis D) — **the one non-obvious axis; read this twice**

Native `<dialog>` **owns its own open state**. `showModal()` / `close()` are imperative, and **Esc fires `cancel` → `close` without consulting the prop.** A naive `$bindable open` therefore *lies* the moment the user presses Esc: the element is closed, the prop still says `true`, and the next `open = true` is a no-op because the prop never changed.

Three rules close this:

1. **Prop → element.** An `$effect` on `open` calls `showModal()` / `close()`. **Guard against re-entry:** only act when the element disagrees, i.e. `if (el.open !== open)`. Without the guard the effect and the event listener ping-pong.
2. **Element → prop.** A `close` **event listener** writes `open = false` **back into the binding**. This is what makes **Esc honest** — it is not optional polish, it is what keeps the bindable from lying. (`cancel` fires first on Esc and is preventable; we do **not** prevent it. Listening to `close` alone is sufficient and covers every close path.)
3. **`onClose?`** fires from the same `close` listener, after the state has settled.

**Verify consequence (method finding from J-495, applies directly here):** Svelte 5 flips state **synchronously** but flushes DOM effects later — a same-tick CDP read after dispatching Esc will see the prop change *without* the element having settled. **Read after settle.**

### 2.5 Getter G (axis E)

```
{ title, open }
```

- `title` — from the prop.
- **`open` MUST be read from the DOM (`el.open`), not from the prop.** The registry's job is to report what *rendered*, not what we *intended*. If a future bug desynchronises prop from element, G must expose it, not paper over it (D-065 / Rule 1 in component form).

### 2.6 Skin + children lifetime (axis F)

- **Skin: L2 `skin.css` only.** `.dialog` (surface, radius `--rad`, padding, border, `--s*` background) + `dialog::backdrop` (dim/blur). L1 structural stays **empty**. Passes the remove-the-rule litmus: delete the skin and the native dialog still opens, traps focus, and closes — only the appearance goes. Close button reuses `.button` (no new rule).
- **Children stay mounted when closed.** Native `<dialog>` closed = `display: none`, **not unmounted**. So composed children (`…__close`, and in C3 the About body) **register on mount and stay registered** — unlike `menu-item`, which mounts on popup-open. **Consequence for verify: the registry count is STABLE across open/close; only `open` flips.** Do not expect (or write) an open/close registry delta.
- **Backdrop click does NOT dismiss.** Native doesn't; we don't fake it. Close paths are **the Close button and Esc**. State this plainly.
- **`::backdrop` inherits nothing** from the document — CSS custom properties do **not** cascade into it in all engines. If a `var(--…)` in `dialog::backdrop` renders as nothing, that is the cause; use a literal or re-declare. **Ground it, don't guess** (Rule 6).

### 2.7 Sampler cell

- `dialog` **is** a catalogued `core` component (a general di container, not frame chrome) → it **gets a sampler cell**, under **DI Composites**.
- The cell = a `dialog` mounted **closed** + a plain **sampler-chrome trigger button** to open it (sampler chrome, unregistered — the `stream-scroll` fixture precedent). A modal in the grid would cover the grid; a trigger is the honest fixture.
- Consider a **second cell** exercising `closeLabel` (a non-default label) so the prop isn't a dead branch — Clair's call, note it.
- **Sampler catalogue registry GROWS** (currently **309**). **Measure the real delta via CDP; never predict it** (Rule 5). State the new count and enumerate every new id.

---

## 3. Files to touch (indicative — Clair confirms exact paths)

1. `ui/core/…/dialog.svelte` — new `core` di composite (§2.1–§2.5); composes `button` (`…__close`); getter G.
2. `ui/assets/skin.css` — L2 `.dialog` + `dialog::backdrop` (§2.6). Confirm the live token names first (Rule 5).
3. `ui/sampler/…` (DI Composites panel) — the `dialog` cell(s) + trigger (§2.7).
4. `ui/docs/xgen-ui-components.md` — registry entry (Chat writes this in the doc-bridge commit, **not** Clair's feat).

**NOT this milestone:** the Help menu · the About dialog · `get_about_info` · `build.rs` · `xgen-common::about` · the logo assets · any Rust · any real-client file. Those are **C2 / C3**. Scope-clean means: **no `xgen-client/**`, no `ui/client/**`, no Rust.**

---

## 4. Verify plan — sampler 9422 (D-097; Rule 2, quote real output)

The sampler is the correct surface: `dialog` is a pure di component with no shell or window effect. Both accents via skin-swap.

1. **Registry** — the `dialog` cell(s) present; `count === unique === domCount`; **0 orphans both directions**; **measure the real total** (309 → N), enumerate each new id (`dialog#…` + its `…__close`).
2. **Getter G exact** — `{ title, open }`; `open` **read from `el.open`** (§2.5), confirmed `false` on mount.
3. **The open path** — click the trigger → `el.open === true`, `el.matches(':modal') === true` (**this is the `showModal()` proof** — a `<dialog open>` attribute would be `open:true` but `:modal` **false**. This single assertion is what separates a correct modal from the silent-downgrade failure in §2.2. Do not skip it).
4. **Top layer + backdrop** — `dialog::backdrop` renders (computed style present); the dialog paints above the grid.
5. **Focus trap** — on open, focus lands **inside** the dialog (native autofocus behaviour); Tab does not escape to the sampler grid behind it.
6. **Esc honesty (the §2.4 proof)** — dispatch Esc → **read after settle** → `el.open === false` **AND** getter G `open === false` **AND** the bound prop is `false`. If the prop still reads `true`, §2.4 rule 2 is not wired.
7. **Close button** — click `…__close` → closed, `onClose` fired once.
8. **Registry stability across open/close (the §2.6 proof)** — count is **identical** open vs closed; `…__close` is registered in **both** states. 0 orphans in both.
9. **Skin** — `.dialog` + `dialog::backdrop` rules in cascade (stylesheet-rule inspection, the N-042 method — `::backdrop` will **not** answer to `getComputedStyle` on the element).
10. **Accent** — client `#c28840` ↔ node `#3a7ab0` under the skin-swap; if the composition carries no accent, say so plainly (the N-087 accent-neutral precedent) rather than inventing a carrier.
11. **Build** — `vite build` clean; quote the module count.

**PS-5.1 gotchas (N-086, they will bite again):** wrap every CDP eval return as a **JSON object** — a bare-string return is mangled by `ConvertTo-Json` and can surface a spurious `EVAL ERROR` even though the side effect fired. Single-line evals only.

---

## 5. Rule-6 confirm points (ground it, don't guess)

- **`::backdrop` and CSS custom properties** (§2.6) — confirm whether `var(--…)` resolves inside `dialog::backdrop` in this WebView2. If it doesn't, use a literal and **say so**.
- **`:modal` support** in this WebView2 (§4 leg 3). If the pseudo-class is unavailable, find another honest proof that `showModal()` (not the `open` attribute) is the path taken — do **not** drop the leg.
- **Envelope on a `<dialog>` root** — `use:envelope` was widened to `Element` at 6.1a (the SVG fix, N-083); `<dialog>` is a plain HTMLElement so this should be a non-event, but confirm the class stamp lands.

---

## 6. Definition of Done

- [ ] `dialog.svelte` authored per §2 — native root, `showModal()`-only, guarded `$effect`, `close`-listener write-back, DOM-read `open` in G.
- [ ] L2 skin (`.dialog` + `dialog::backdrop`); L1 empty; remove-the-rule litmus holds.
- [ ] Sampler cell(s) + trigger; `closeLabel` exercised.
- [ ] `vite build` clean; module count quoted.
- [ ] All 11 verify legs in §4 run on the **real sampler (9422)**, both accents, with **actual quoted output** — including leg 3 (`:modal`), leg 6 (Esc write-back), leg 8 (registry stable across open/close).
- [ ] Registry total **measured** (309 → N), every new id enumerated; `count === unique === domCount`; 0 orphans both directions.
- [ ] Scope-clean: no `xgen-client/**`, no `ui/client/**`, no Rust.
- [ ] Any deviation from this runbook **flagged, not absorbed** (Rule 6).

*(Per the task-file DoD rule: "commit pushed" is deliberately NOT a checklist item — it is unflippable inside the commit that performs the push. `Status: COMPLETED` in the header is the real signal.)*

---

## 7. Close (D-074, two commits)

1. **Clair — feat commit** (code only): `dialog.svelte` + `skin.css` + sampler cell.
2. **Chat — doc-bridge commit**: `JOURNAL.md` (J-series) · `CLAUDE.md` PLAY · `ui/docs/xgen-ui-components.md` (registry, 31st `core`) · `ui/docs/xgen-ui-notes.md` (N-089) · `docs/ROADMAP.md` · `docs/xgen-client-frame-phase0.md` §6/§10.4 (6.1e-C1 ✅) · this file → **COMPLETED**.

Joe pushes both. Chat never pushes.

---

*End of M-RP6.1e-C1 runbook.*
