# M-RP-ICON-ADOPT — Icon adoption / glyph consolidation
> **Status**: PENDING  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Backlog milestone (not the current task — parked behind the M-RP6.1 frame arc). Goal: consolidate **every** glyph in the project into **one bank**, so a glyph is a **skin token** — named by the component, drawn by the skin, replaceable by a theme, licensed at build time.

**⚠️ v1.0 supersedes v0.1 on two counts.** v0.1's §1 was **wrong about where the glyphs live**, and its §3 theory is now **settled by measurement, not opinion** — a CDP probe on the real client (2026-07-12) answered every open question, and three of them dissolved. The model is locked as **D-108**; the platform dependency is **D-109**. Canonical model doc: **`ui/docs/xgen-css-layer-model.md`**.

---

## 1. Inventory — MEASURED (2026-07-12), correcting v0.1

**⚠️ v0.1 said the glyphs were *"currently inline `<svg>` inside their field components."* They are not, and never were.** A grep of every live `.svelte` outside `node_modules` for `<svg|<path|viewBox` returns **`icon.svelte` (7) and `app_sampler.svelte` (2, a demo data-URI) — nothing else.** **Zero inline `<svg>` in any field component.** They all live in `skin.css`.

That correction changes the milestone's shape: this is **not** "extract inline SVG from components." It is **"reconcile four mechanisms across two layers."**

**21 distinct glyphs, 4 mechanisms:**

| # | Mechanism | Where | Glyphs |
|---|---|---|---|
| **A** | `<path d>` from a TS registry, **fill**, `--icon-tint` | `icons.ts` → `icon.svelte` | **3** — `caret-down` `dot` `square` |
| **B** | **`mask-image`** data-URI, currentColor / background-color, mostly **stroke** | `skin.css` — **11 declarations, 10 distinct** | `--eye` `--eye-off` `--star` `--drop` `--tri`×2 `--tri-open`×2 `--chip-x` `--tag-gear` `--pal` `--drop-i` |
| **C** | **`background-image`** data-URI, **colour baked into the URI** (`%23e6e6e6`) | `skin.css` — **7** | `textfield[type=]` ×5 (search/email/url/tel/password) · `select` arrow · `--ea-spark` |
| **D** | **`.svg` file** as `src` | `ui/assets/img-placeholder.svg` | 1 — *and it is **duplicated** as an inline data-URI at `app_sampler.svelte:402`* |
| **—** | **OUT OF SCOPE** — OS/window icons | `xgen-{client,node,sampler}/icons/icon.ico`, `logo/*.ico` | named here so they are not re-litigated |

**🔑 THE FINDING THAT DECIDES THE MILESTONE — every skin.css glyph token is declared INSIDE its own component's class selector. None at `:root`.**

```css
.password-field { --eye: … }    .combobox { --tri: … }
.chip           { --chip-x: … } .section  { --tri: … }   ← the DUPLICATE
```

`skin.css` says it deliberately: *"icon-data vars scoped here (no global token)."* **Two consequences, both measured:**

1. **`--tri` / `--tri-open` are declared TWICE** (combobox 1232-33, section 1829-30). The section's own comment says *"REUSES combobox's masked glyphs"* — **and then re-declares them.** *The loss this milestone exists to prevent has already happened.*
2. **A component-scoped custom property is a private variable, not a theme surface.** A theme author cannot redraw *"the eye"* — they must know **which component scopes it**, and redefine each shared glyph **N times**. **Component-scoping half-defeated the skinnability it was meant to serve.** Scale to a 40-glyph set and a theme becomes unwritable.

## 2. The locked model (D-108)

Full specification: **`ui/docs/xgen-css-layer-model.md`** §2–§3. In brief:

> **A glyph is a SKIN TOKEN. `core` owns the NAME (identity = content); the skin owns the SHAPE (geometry = appearance).**

- **Source of truth:** `ui/assets/icons/*.svg` + `icons.manifest.json` (**hand**; carries licence per glyph). **Never ships.**
- **Generated:** `ui/assets/glyphs.generated.css` → `:root { --glyph-gear: path('…'); --glyph-gear-url: url("data:…") }` — **the bank, the runtime default**; and `icons.generated.ts` → `type IconName = 'gear' | …` — **names only, no geometry.**
- **Component path:** `<Icon name="gear"/>` → `<path>` with **no `d` attribute** + inline `--g: var(--glyph-gear)`; **ONE** skin rule: `.icon path { d: var(--g) }`.
- **Native-root path** (`<select>`, `<input>` — no child to hang a path on, N-020): `background-image: var(--glyph-x-url)`.
- **Theme override:** a later `:root` redeclaration. **Identical mechanism to `--accent2`.**

## 3. Theory — SETTLED by measurement (was §3 "open")

Every v0.1 question, and what the probe did to it. Evidence table: `xgen-css-layer-model.md` §5.

| v0.1 question | Status |
|---|---|
| **§3a** Shape-skinnability: registry `d`-strings **(A)** vs CSS `mask-image` **(B)** vs CSS `d: path()` **(C)** | **✅ (C), and the framing was moot.** v0.1 asked *"is per-theme glyph replacement a goal?"* — **route (B) was already shipped for 13 glyphs**, without a decision. **CSS `d:` works in WebView2** (measured), **and `d: var(--token)` resolves**, so shape can live in the skin **on a real `<path>` element** — real DOM, per-path control, CDP-inspectable, *and* theme-replaceable. **(C) gives what (A) and (B) each gave only half of.** |
| **§3b** Multi-colour glyphs (the palette mark) break the single-tint model — baked fills / multi-token / **reclassify as `image` (D-096)**? | **✅ DISSOLVED. It stays an `icon`.** N `<path>` children, each `d: var(--glyph-pal-N)` **and its own fill token** — measured: two paths, two geometries, two independent fills, from one generic rule. **A mask can never do this.** **D-096 is NOT re-opened.** |
| **§3c** Add a `stroke` mode to `icon`, or re-author stroke glyphs as fill? | **✅ DISSOLVED. Neither.** With real `<path>` children, `fill` / `stroke` / `stroke-width` are **ordinary skin properties** on `.icon path`. **`icon` gains no new prop.** |
| **§3d** Stateful glyphs (eye / eye-off) | **Unchanged and trivial:** two names in the bank; the **component** swaps `name` off its own state. The skin never toggles state. |
| **§3e** Chevron/caret: fold the CSS triangle into the registry, or leave it? | **✅ FOLD.** `--tri` / `--tri-open` become ordinary `:root` tokens. **The duplicate dies as a side-effect.** |
| **§3f** Provenance / licence (BSL→GPL audit gate) | **✅ STRUCTURAL, not periodic.** Licence + source live in `icons.manifest.json`, **per glyph**. **A glyph with no licence entry fails the build.** No audit can forget what the compiler enforces. |

**❌ One idea raised mid-walk and DROPPED — the `d`-attribute fallback.** The probe showed CSS `d:` **overrides** a present `d` attribute (leg 4), which made "ship geometry as an attribute *and* let the skin override it" *possible*. **It is not right.** It would be **two defaults for one glyph** — a second source of truth for geometry (**D-067 drift wearing a safety vest**) — hedging against a browser that cannot occur (Tauri is always Chromium; **D-109**). **Geometry lives in the skin. Only in the skin.**

## 4. Milestone shape

- **Phase-0 (D-071 audit).** Classify all 21: fill / stroke / multi-colour / native-root. **Licence-source every one** (§3f). **⚠️ MANDATORY, not optional (D-110): re-emit all seven baked-colour glyphs as `currentColor` masks** — the colour/geometry split must be *enforceable* or the Space ban is void on exactly those glyphs. Lock the `Glyph` manifest record shape and the generator contract. *(The model itself is already locked — D-108/D-110. Phase-0 is classification + provenance, not re-litigation.)*
- **Phase-1.** `icons.manifest.json` + the generator; `glyphs.generated.css` + `icons.generated.ts` emitted; the L1.5 import lands in all three `main.js`. **`--tri` dedup falls out.** `icons.ts` retires as a geometry store.
- **Phase-2.** Migrate consumers: the 10 mask glyphs → `<Icon>` / `var(--glyph-*-url)`; the 6 native-root glyphs → `var(--glyph-*-url)`; the `app_sampler.svelte:402` duplicate of `img-placeholder.svg` dies. **Each CDP-verified against its prior render** (N-097: the painted pixel is the leg).
- **Phase-3.** Sampler **glyph-grid** page — the bank renders itself from the `IconName` union, with names + licences. *You cannot redraw a glyph you can see.*
- **Gate:** starts after the frame arc is functional. **Unchanged — this does not jump the queue.**

## 5. ✅ CLOSED (was "Open — for Joe")

Both items raised at v1.0 were **locked the same day**.

1. **✅ The Space-theme glyph-override ban → D-110.** **A Space may re-COLOUR; a Space may not re-DRAW and may not re-LAYOUT.** Colour tokens (including the glyph **tint**) are permitted — *the mark keeps its meaning, only its hue changes*. **Geometry (`--glyph-*`, `--glyph-*-url`) is banned**, as is layout/metrics; **everything not on the allowlist is banned by default.** Specified in **Ch6 §6.3.1 / §6.3.2**, which also answers the second-pass question Ch6 had carried open since Session 1. **Locked before a single line of theming exists** — `state.space_theme` appears in no Rust, TS or Svelte.

   > **🔑 AND IT BINDS THIS MILESTONE.** D-110 imposes a **normative constraint back onto the generator**: **`--glyph-*-url` MUST be emitted COLOUR-FREE** (a `currentColor` mask; colour from a **separate** token). A data-URI with colour **baked in fuses colour and geometry into one token** — so a Space permitted to change its colour would thereby be permitted to **redraw** it. **The re-emit of the seven baked-colour glyphs (the 5 `textfield[type=]` insets, the `select` arrow, `--ea-spark` — all carrying `%23e6e6e6`) is now a SECURITY REQUIREMENT, not a Phase-0 tidy-up.** It moves from "classify per glyph" to **"mandatory for all seven."**

2. **✅ Ch6 §6.2 amended → Ch6 v0.5, Session 10.** The CSS Layer Architecture is **rewritten against the shipped code**: `tokens.css` never existed; `skin-dark.css` → `skin.css`; the glyph bank added as L1.5; and the reversal that matters — **component `<style>` blocks are FORBIDDEN, not required** (N-025/N-031/N-090). D-057/D-058 **superseded in part, not deleted** — their intent survives, their file structure does not.

3. **Generator host** — a Vite plugin vs a standalone `npm run glyphs` prestep. **Still open.** Phase-1 detail; no architectural weight.

## 6. Non-goals / deferred

- **No theming ships here.** `theme-*.css` does not exist; Ch6 §6.3's cascade is specified but unbuilt. This milestone makes the bank **shaped for** a theme layer — **no milestone may claim theming works.**
- **Trust surfaces are NOT this milestone's** — **D-110** (a Space may re-colour, not re-draw) and **D-111** (a client must not fetch a host chosen by someone else; outbound URL resolution is **node-side**) both landed the same day and both bind future code, but neither ships here. *(v1.2 correction: an earlier list named `url()` fetches as an open glyph-adjacent risk. **It is not one** — D-110's colour-only allowlist rejects a `url()` outright. Retracted.)*
- `temperature-indicator` and other post-frame widgets stay separate (⏸️ — mechanism withdrawn at J-502; the node plugin is a no-op, so there is nothing to render).
- OS/window `.ico` icons stay out of scope.

---

*Model locked (D-108/D-109). Phase locks + milestone number pending the Phase-0 classification pass.*
