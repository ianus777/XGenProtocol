# XGen UI — CSS Layer Model
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

The canonical, **shipped** CSS layer model for the XGen UI. This doc exists because the model is load-bearing for everything downstream — theming, the glyph bank, component authoring rules.

Crystallises into **D-108** (the glyph bank), **D-109** (the platform dependency) and **D-110** (the Space-theme override subset). Companion to `xgen-region-dock-model.md` (which owns *layout*); this doc owns *appearance*.

> **v1.1 (2026-07-12):** the two drifts filed at v1.0 are **CLOSED**. **Ch6 §6.2's CSS Layer Architecture has been REWRITTEN** against this model (Ch6 v0.5, Session 10) — it is no longer stale, and it no longer contradicts N-025/N-031/N-090. **Ch6 §6.3's Space-theme override question is ANSWERED** — **D-110: a Space may re-COLOUR; it may not re-DRAW and may not re-LAYOUT.** §4 and §6 below are updated accordingly, and **§2.2 gains a normative constraint that D-110 imposes back onto the generator.**

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

> **🔑 NORMATIVE (D-110, v1.1) — `--glyph-*-url` MUST be emitted COLOUR-FREE** (a `currentColor` mask; colour supplied by a **separate** colour token). **This is a security requirement, not a style preference.** D-110 permits a Space theme to change a glyph's **colour** but bans it from changing a glyph's **geometry**. A data-URI with colour **baked into it fuses colour and geometry into one token** — so a Space permitted to change that token's colour would thereby be permitted to **redraw** it, and the ban would be **unenforceable on exactly those glyphs**. *This makes the Phase-0 re-emit of the seven baked-colour glyphs (the 5 `textfield[type=]` insets, the `select` arrow, `--ea-spark` — all currently carrying `%23e6e6e6`) **mandatory**, not cosmetic.*

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

## 4. Relationship to Ch6 — the drift is CLOSED (v1.1)

**Ch6 §6.2's CSS Layer Architecture has been rewritten against this model** (Ch6 **v0.5**, Session 10, 2026-07-12). It is no longer stale. Recorded here because the *shape* of the correction is the useful part:

| Ch6 §6.2 as originally written (pre-Phase-1, D-057/D-058) | What shipped — and what Ch6 now says |
|---|---|
| `base.css` → `tokens.css` → `skin-dark.css` → `components/` | resets → **glyph bank** → `skin.css` → (`theme-*.css`), with `app.css` as shell chrome |
| **`tokens.css` is its own layer** | **Never built.** Tokens live in `skin.css`. |
| `skin-dark.css` | `skin.css` — one skin; dark/light is a **theme-layer** concern, not a filename |
| **"Each `.svelte` file carries its own `<style>` block"** | **🔑 THE REVERSAL: component `<style>` is FORBIDDEN** (N-025/N-031/N-090). A component ships **zero** CSS. |
| *(no concept of a glyph layer)* | **L1.5, the glyph bank** (D-108) |

**🔑 The reversal is worth stating as a principle, because it reads backwards until you see it:**

> **The rule that makes skinning TOTAL is the rule that forbids the component from participating in it.**

A component that could style itself would be a **second place appearance lives** — and a skin could then never fully re-skin it. D-058 had it exactly inverted.

**D-057/D-058 are superseded in part, not deleted.** Their *intent* survives intact — the minimal reset (not a generic normalize), the 13px/1.35 root scale, the 4px spacing unit, no hardcoded values in components. Their **file structure and the component-`<style>` rule do not.**

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

## 6. Theming — and the one thing a Space may NOT do (D-110)

**🔑 A theme can redraw ANY glyph (§3.3). That is the feature — and, at Layer 3, the danger.**

Ch6 §6.3's cascade is: XGen default → **application theme** (operator/user) → **Space theme**. Layers 1 and 2 are ours and the user's. **Layer 3 is not — it is declared by a Space OWNER and arrives over the wire** in a `state.space_theme` Event. Unrestricted, a Space owner could redraw a **lock**, a **warning**, a **verified** mark, or the **AI badge** (Ch6 §6.13) — making a hostile Space look trustworthy, or a human look like a bot.

> ### D-110: A Space may **re-COLOUR**. A Space may **not re-DRAW**, and may **not re-LAYOUT**.

- **✅ Colour** (incl. the glyph **tint**, `--icon-tint`) — permitted. *The mark keeps its meaning; only its hue changes.*
- **❌ Geometry** (`--glyph-*`, `--glyph-*-url`) — **banned. The mark IS the meaning.**
- **❌ Layout / metrics** — banned (readability, accessibility, and displacement attacks).
- **❌ Everything else** — **banned by default. Allowlist, never denylist.**

**Enforcement is client-side and has THREE parts — all required.** Full spec: **Ch6 §6.3.2**. In brief: **allowlist the key** · **validate the value AND apply it via `element.style.setProperty()`** (a key allowlist alone is theatre — string-concatenating a stylesheet lets a malicious *value* escape its declaration and inject arbitrary CSS) · **scope it** to the active Space's subtree, never `:root`, never app chrome.

**And it constrains this doc's own generator** — see the normative note in §2.2: `--glyph-*-url` must be **colour-free**, or the colour-yes/geometry-no split is unenforceable.

## 7. Still open — filed, not solved

1. **`theme-*.css` does not exist.** Ch6 §6.3's cascade is **specified and entirely unbuilt** — `state.space_theme` appears in **no Rust, TypeScript, or Svelte** (grepped 2026-07-12). What is locked is that **the bank is SHAPED so the theme layer can override it when it lands**. **No milestone may claim theming works**, and **none may ship a Layer-3 applier that does not implement Ch6 §6.3.2 in full.**
2. **The exact colour-token allowlist** (names + count) — enumerated when the theme layer is built.
3. **Can a user disable Space themes entirely** (accessibility)? *Recommendation: yes, and it is cheap — Layer 3 is a scoped, droppable overlay by construction.*
4. **The WIDER Space-owner-content trust surface** — `url()` fetches, font substitution, module widgets (D-036). **D-110 closes the glyph hole, not the category.** Ch6's, not the glyph bank's. **Flagged, not solved.**
5. **Per-glyph classification** of the 21 (fill / stroke / multi-colour / native-root) + licence-sourcing — the M-RP-ICON-ADOPT Phase-0 output.

---

*End of CSS layer model.*
