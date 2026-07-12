# XGen UI — CSS Layer Model
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

The canonical, **shipped** CSS layer model for the XGen UI. This doc exists because the model is load-bearing for everything downstream — theming, the glyph bank, component authoring rules — and because **Ch6 §6.2's four-layer CSS architecture (D-057/D-058) no longer describes what shipped** (§4). Where this doc and Ch6 §6.2 disagree, **this doc describes the code**.

Crystallises into **D-108** (the glyph bank) and **D-109** (the platform dependency). Companion to `xgen-region-dock-model.md` (which owns *layout*); this doc owns *appearance*.

---

## 0. The stack — LOCKED (2026-07-12, Joe)

```
theme-*.css            ← custom skin. May redefine --accent2 AND --glyph-gear.
                          Identical mechanism.
─────────────────────
skin.css               ← default skin, hand-written  ┐ ONE LAYER,
glyphs.generated.css   ← default skin, machine-made  ┘ split by WHO WRITES IT
─────────────────────
xgen-normalize / modern-normalize   ← reset, not skin
```

**Read it in three sentences:**

1. **The normalizes are a reset, not a skin.** They carry no visual opinion.
2. **`skin.css` + `glyphs.generated.css` are ONE layer — the default skin.** It ships inside the app, it is not user-facing, and **without it the app is unusable**. The split between the two files is **tooling, not architecture**: one is hand-edited, the other is machine-rewritten, and *you never mix a generated block into a file a human edits live over HMR.*
3. **`theme-*.css` is the override layer.** It loads last and wins by cascade. It may override **anything** in the default skin — a colour, a radius, or a **glyph** — by redeclaring the token.

**The load order is the mechanism.** There is no second machinery: the cascade *is* the override system.

## 1. The layers, one job each

| Layer | File(s) | Who writes it | Owns |
|---|---|---|---|
| **L0** | `ui/assets/modern-normalize.css` | vendored, **pristine** | cross-browser reset. Never edited. |
| **L0.5** | `ui/assets/xgen-normalize.css` | hand | the XGen floor on top of the reset. No colour, no visual opinion. |
| **L1.5** | `ui/assets/glyphs.generated.css` | **generator** | **the glyph bank** — `:root { --glyph-* }`. §2. |
| **L2** | `ui/assets/skin.css` | hand | **all appearance** (N-090). Tokens, colour, type, spacing, every component's look. |
| **shell** | `ui/{client,node,sampler}/src/app.css` | hand | **shell chrome ONLY** (N-031) — the app-frame skeleton + the per-app accent. **Not skin.** |

**Shipped import chain** (`ui/client/src/main.js`, and its node/sampler siblings):

```js
import '$assets/modern-normalize.css';   // L0
import '$assets/xgen-normalize.css';     // L0.5
import '$assets/glyphs.generated.css';   // L1.5  ← the bank
import '$assets/skin.css';               // L2
import './app.css';                       // shell chrome
```

**The bank sits BELOW `skin.css` deliberately.** `skin.css` must be able to override a glyph, and a theme must be able to override both. Defaults always sit under the thing that overrides them.

**⚠️ Why the bank is NOT in `app.css`** — three blockers, and the first two are hard:
1. **There are three `app.css` files** (client / node / sampler). The bank would be **triplicated**, and drift between copies is the `--tri` failure by construction (§2.1).
2. **`app.css` loads AFTER `skin.css`.** Glyph defaults there would sit *above* the skin — `skin.css` could no longer override a glyph. Cascade inverted.
3. **`app.css` is scoped to shell chrome by its own header** (N-031). A glyph is shared appearance, i.e. **skin**.

## 2. The glyph bank (L1.5)

### 2.1 Why it exists — the measured failure it fixes

Before the bank, glyphs lived in **four** mechanisms across **two** layers, with **two** colouring models. Measured 2026-07-12 — **21 distinct glyphs**:

| # | Mechanism | Where | Count |
|---|---|---|---|
| A | `<path d>` from a TS registry, fill, `--icon-tint` | `icons.ts` → `icon.svelte` | **3** (`caret-down`, `dot`, `square`) |
| B | `mask-image` data-URI, currentColor, mostly **stroke** | `skin.css`, **11 declarations / 10 distinct** | `--eye` `--eye-off` `--star` `--drop` `--tri`×2 `--tri-open`×2 `--chip-x` `--tag-gear` `--pal` `--drop-i` |
| C | `background-image` data-URI, **colour baked into the URI** | `skin.css`, **7** | `textfield[type=]` ×5 · `select` arrow · `--ea-spark` |
| D | `.svg` file as `src` | `ui/assets/img-placeholder.svg` | 1 |

**And every skin.css glyph token was declared INSIDE its own component's class selector** — `.password-field { --eye: … }`, `.combobox { --tri: … }`, `.section { --tri: … }`. **None at `:root`.** skin.css said so deliberately: *"icon-data vars scoped here (no global token)."*

**Two consequences, both bad, both measured:**
- **`--tri` / `--tri-open` are declared TWICE** (combobox + section — the section comment even says *"REUSES combobox's masked glyphs"*, then re-declares them). The loss is not hypothetical; it already happened.
- **A component-scoped custom property is a private variable, NOT a theme surface.** A theme author cannot redraw "the eye" — they must know *which component scopes it* and redefine each shared glyph N times. **Component-scoping half-defeated the very skinnability it was meant to serve.**

### 2.2 The bank

> **Promote every glyph to `:root` as a `--glyph-*` token, declared ONCE. Components consume; they never declare.**

```css
/* glyphs.generated.css — GENERATED, DO NOT EDIT */
:root {
  /* gear — lucide, ISC */
  --glyph-gear:     path('M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z');
  --glyph-gear-url: url("data:image/svg+xml,%3Csvg…%3E");
}
```

**Two forms per glyph, because two consumer species exist:**

| form | consumed by | why |
|---|---|---|
| `--glyph-x` = `path('…')` | the CSS **`d:`** property on a `<path>` child | the `icon` component |
| `--glyph-x-url` = `url("data:…")` | `background-image` / `mask` | **native roots** — `<select>`, `<input>` have **no child element** to hang a `<path>` on, and **N-020 forbids wrapping the root** |

`path()` is only consumable by `d:`. **That is why the two forms are not redundant** — and why hand-maintaining them is forbidden (§2.4).

### 2.3 The source of truth

```
ui/assets/icons/*.svg            ← HAND. 24×24 viewBox, geometry only, no colour.
ui/assets/icons/icons.manifest.json
    "gear": { paint:"stroke", sw:2, source:"lucide", license:"ISC", url:"…" }
                   │
             [ codegen ]
        ┌──────────┴──────────┐
        ▼                     ▼
glyphs.generated.css     icons.generated.ts
:root{--glyph-gear:…}    type IconName = 'gear' | 'eye' | …
   ← the SHAPE              ← the NAME (no geometry)
```

> **The split that makes the whole thing work, and it is N-025/N-090 restated:**
> **`core` owns the NAME (identity = content). The skin owns the SHAPE (geometry = appearance).**

A component says *which* glyph. The skin says *what it looks like*. **A component never writes geometry — for the same reason it never writes a colour.**

**Provenance lives in the manifest, per glyph.** A glyph without a licence entry **fails the build** — the BSL→GPL gate becomes structural rather than a periodic audit.

**The `.svg` files never ship.** They are authoring source; the generator bakes them into the default skin.

### 2.4 Guards — how each failure mode dies

| Failure | Dies how |
|---|---|
| Typo in a glyph name | **Build error** (`IconName` union) |
| Two glyphs under one name (`--tri`) | **Build error** (generator rejects duplicates) |
| A glyph ships with no licence | **Build error** (manifest entry required) |
| Someone hand-edits the bank | File is marked GENERATED; regeneration overwrites. It can only overwrite **its own output** — never `skin.css`. |
| A theme deletes a token | `--g` unresolvable → the `<path>` renders **empty** + DEV-warn. **`icon.svelte` already does exactly this** for an unknown name (the W-13 unknown-id-drop precedent). No throw, no new failure mode. |
| A glyph gets redrawn because nobody knew it existed | The **sampler glyph-grid** renders the whole bank from the `IconName` union. *You cannot redraw a glyph you can see.* |

## 3. How a glyph reaches the screen

### 3.1 Component path

```svelte
<Icon name="gear" size={16} />
```

1. `name` is typed `IconName` → **a typo is a compile error**, not a blank square.
2. `icon.svelte` renders `<svg class="icon">` with N `<path>` children — **each with NO `d` attribute**.
3. It sets an inline custom property pointing at the token **by name**, built in JS: `style="--g: var(--glyph-gear)"`.
4. The skin holds **ONE rule for every icon in the app**:

```css
.icon path { d: var(--g); fill: var(--icon-tint, currentColor); }
```

**Not one rule per glyph.** Measured (§5): `var()` **does** resolve inside a custom-property value.

Multi-path glyphs emit `--g1`, `--g2`, … — **each `<path>` independently fillable.** A mask can never do this, which is why multi-colour marks stay `icon`s and are **not** demoted to `image` (D-096 stays shut).

### 3.2 Native-root path

No child element exists, and N-020 forbids wrapping the root. These consume the `-url` form:

```css
.select                  { background-image: var(--glyph-caret-down-url); }
.textfield[type="email"] { background-image: var(--glyph-mail-url); }
.password-field .button  { mask: var(--glyph-eye-url) center / contain no-repeat; }
```

**Same bank, same names, one source.** *(The 5 baked-colour `%23e6e6e6` insets get re-emitted as `currentColor` masks so they tint like everything else — per-glyph confirmation is a Phase-0 output, not an assumption here.)*

### 3.3 Theming

A theme is a CSS file loaded **after** the bank:

```css
/* theme-brutalist.css */
:root {
  --accent2:    #ff0000;                  /* recolours the app */
  --glyph-gear: path('M4 4h16v16H4z');    /* redraws EVERY gear */
}
```

**Identical mechanism for a colour and a glyph.** Cascade: later wins. Every consumer follows — the `<Icon>` in the menu, the shelf face, the `<select>` arrow. **No `.svelte` file moves.**

## 4. ⚠️ Relationship to Ch6 §6.2 — a NAMED DRIFT, not a silent one

**Ch6 §6.2 "CSS Layer Architecture" (D-057/D-058) describes a four-layer model that did not survive Phase-1 implementation.** Ch6 is explicit that it is a **first pass** to be corrected by a second pass after Phase-1 experience; this is that correction, and it is recorded rather than quietly applied.

| Ch6 §6.2 (pre-Phase-1) | Shipped (measured 2026-07-12) |
|---|---|
| `base.css` → `tokens.css` → `skin-dark.css` → `components/` | `modern-normalize` → `xgen-normalize` → **`glyphs.generated.css`** → `skin.css` → `app.css` |
| **`tokens.css` is its own layer** | **No `tokens.css` exists.** Tokens live in `skin.css`. |
| `skin-dark.css` | `skin.css` (one skin; the dark/light split is unbuilt) |
| **"Each `.svelte` file carries its own `<style>` block"** | **N-025 forbids component-local CSS. N-031: `app.css` is shell chrome only. N-090: every skinnable setting lives in `skin.css`.** The shipped rule is the *opposite* of Ch6's. |

**⚠️ Ch6 §6.2 is NOT amended by this doc.** Amending a spec chapter is a Joe-lock, and it is filed as an open item (§6). Until then: **this doc describes the code; Ch6 §6.2 describes an intention that the code superseded.** Anyone reading Ch6 §6.2 for the CSS layer model is reading a stale record — that is exactly the D-067 drift surface the project exists to eliminate, and it is now visible instead of latent.

## 5. Evidence — the CDP probe (real client 9222, 2026-07-12)

The model above is **not** derived from documentation. Every claim below was measured on the real client, non-destructively, returning to an exact baseline (registry **38 → 38**, zero probe nodes).

| Leg | Claim | Measured |
|---|---|---|
| 1 | baseline | `count 38 / unique 38` |
| 2 | CSS `d: path(…)` renders on a `<path>` with **no `d` attribute** | `hasDAttr:false` · bbox **14×14 @ (5,5)** · **`getTotalLength()` = 56** (= 4×14, the true perimeter — the geometry engine, not just the cascade) |
| 3 | **`d: var(--glyph-square)`** — indirection through a `:root` token | identical: 14×14, len 56 |
| 3b | **a later stylesheet redefines the token** | bbox → **20×20 @ (2,2)** · len → **64.72** (= 20 + 2·√500, the exact triangle perimeter) · `fill` preserved |
| 4 | **`d` attribute present AND CSS `d:` set** | attribute still `"M5 5h14v14H5z"` — **rendered geometry is the CSS one. CSS WINS over the attribute.** |
| 5 | **indirection + multi-path + per-path fill**, one generic rule | p1 = 14×14 / len 56 / **magenta** · p2 = 20×20 / len 64.72 / **green** — **`var()` DOES resolve inside a custom-property value** |
| 5b | a theme overrides **through** the indirection | p1 → diamond, len **56 → 56.57** (= 4·√200, exact) · p2 **untouched** · **inline style unchanged** |
| 6 | `-url` form from a `:root` token on a **native root** | resolved on a real `<select>`, 159-char data-URI |
| 7 | teardown | **38 / 38**, 0 probe nodes, var cleared |

**Consequences that fell out of the probe, and each closed an open question:**
- **One skin rule serves the entire icon system.** The `data-glyph` per-glyph-rule fallback is **dead**.
- **Multi-colour glyphs stay `icon`s** (leg 5) — **`xgen-icon-adoption.md` §3b closed; D-096 not re-opened.**
- **Stroke-vs-fill is a skin property, not a component prop** (`.icon path { stroke: … }`) — **§3c dissolved; `icon` gains no new API.**
- **The `d`-attribute-as-fallback idea was DROPPED** (leg 4 made it *possible*, not *right*): it would be a **second source of truth for geometry** — D-067 drift wearing a safety vest — guarding against a browser that cannot occur (§6/D-109). **Geometry lives in the skin. Only in the skin.**

## 6. ⚠️ Open — filed, not solved

1. **Ch6 §6.2 amendment** (§4). The chapter's CSS layer architecture is stale vs the code. Needs a Joe-lock; it is a spec-chapter touch.
2. **🔑 The Space-theme glyph-override ban.** **Ch6 §6.3 already carries the open question *"Which specific CSS tokens may a Space owner override?"*** (and Session 1 filed *"Permitted Space theme override token list"* for the second pass). **This model supplies the first entry on that list, and it is a trust question, not a styling one:** under §3.3 a theme can redraw **any** glyph — including a **lock**, a **warning**, or a **verified** mark. Ch6 §6.3's Layer 3 is a **Space theme declared by the Space owner via a `state.space_theme` EVENT** — i.e. **attacker-supplied CSS arriving over the wire**. **Glyph tokens MUST be excluded from the Space-overridable subset.** App and user themes may redraw glyphs; **a Space may not.** *(The wider question — what else arbitrary Space-owner CSS can do, `url()` fetches, layout displacement — is Ch6's, not the glyph bank's. Flagged, not solved.)*
3. **`theme-*.css` does not exist yet.** Ch6 §6.3's three-layer cascade (app default → user choice → Space override) is **specified but unbuilt**. What is locked here is that **the glyph bank is SHAPED so the theme layer can override it when it lands** — not that theming ships. **No milestone may claim theming works.**
4. **Per-glyph re-emit of the 5 baked-colour `%23e6e6e6` insets** as `currentColor` masks — a Phase-0 classification output (§3.2).

---

*End of CSS layer model.*
