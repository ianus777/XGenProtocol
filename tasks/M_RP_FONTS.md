# M-RP-FONTS — declare the bundled type, and the 403 that hid it
> **Status**: ACTIVE  
> Owes: M-RP-FONTS-WOFF2 — convert the mono variable TTFs to woff2  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-23  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this was, and what it turned out to be

**Scoped as:** declare the fonts already sitting in `ui/assets/fonts/`, because `skin.css` asked for weights
500/600/700 at fourteen sites and italic at two while declaring **one** face (Inter-Regular 400) — so every
bold and every italic in the app was a renderer-synthesised smear.

**Turned out to be:** 🔑 **the fonts had never loaded in ANY dev shell, and not because of anything this**
**milestone changed.** `ui/assets/` sits outside each app's Vite root, so `skin.css`'s `url()` references
resolved to `/@fs/` paths the dev server answered with **403 Forbidden**. The stylesheet itself loaded fine —
it arrives through the `$assets` alias as a module — so the CSS was correct and the files were simply never
served.

⚠️ **The consequence is worse than being broken, because `vite build` emits the fonts correctly.**
***Dev and the built app rendered in different typefaces, and every appearance judgement in this project is
made in dev.*** The dev client had been drawing `system-ui` (Segoe UI) for the project's entire life.

🔑 **And the project had already met this exact constraint.** `ui/sampler/vitest.config.js` carries the fix
with the reasoning written out — *"The suites live in ../common (outside this package's root), so widen
Vite's fs allow-list to the repo root"* (J-491). **It was applied to the test harness and never generalised
to the three dev servers.** *A finding that was written down, fixed once, and not carried across.*

## §1 — Decisions

**D1 — VARIABLE, not a static set.** Chat first recommended declaring all 34 statics; Joe pushed to check the
upstream distributions in `fonts/`, and both families ship variable builds. **4 files, ~1.35 MB**, against
~4 MB for 34 statics and 138 KB for the broken status quo.

🔑 **The deciding argument was not size — it was the trap.** A static subset means the next `font-weight`
nobody declared falls back to a synthesised smear **silently, with no error**. A continuous 100–900 axis makes
every weight real, including stops no static file could provide. ***The person a silent type fallback catches
is whoever is tuning appearance*** — and `skin.css` is Joe's file, tuned live over HMR.

**D2 — the `"XGen UI Sans"` indirection is kept and extended** with `"XGen UI Mono"`. Call sites never name a
foundry, so swapping either family later is a four-line change.

**D3 — mono ships as TTF.** JetBrains Mono 2.304 has no variable woff2. The TTFs work and are uncompressed;
the conversion is a **size** item, not a correctness one → filed, not done.

**D4 — `server.fs.allow` added to client, node AND sampler.** All three render `skin.css`; fixing one would
have left two shells silently drawing a different typeface than they ship.

**D5 — licences ship beside the fonts.** Both families are OFL, which requires the notice to travel with them.
`LICENSE-Inter.txt` · `LICENSE-JetBrainsMono.txt` · `AUTHORS-JetBrainsMono.txt`, the **D-108 glyph-provenance**
precedent applied to type.

## §2 — Verification (real client 9222, sampler 9422)

⚠️ **`document.fonts.check()` returns false for an unloaded face, so "never requested" and "broken" return the
same string.** Every check below was therefore preceded by a forced `document.fonts.load()` — the positive
control that makes a `true` mean something.

- fetch of the font URL: **403 → 200** (and the *pre-existing* `Inter-Regular.woff2` was **also 403**, which is
  what proved the defect predates this milestone)
- all four faces reach status **`loaded`**
- `check('600 16px "XGen UI Sans"')` **true** and `check('italic 700 16px "XGen UI Mono"')` **true** —
  **combinations with no static file on disk, so the variable axis is proven rather than assumed**
- the head-marker notice (*"Showing this session only…"*) computes `font-style: italic` against a **loaded**
  italic face — it had been a skewed Regular
- registry **149** at open · **149 → 156 → 158** on space/room latch

**Floors:** cargo **1553/0/62 across 56 terminator lines — IDENTICAL**, which *proves* no Rust landed ·
vite **202 / 170** · npm **154** · svelte-check **0/34/15** · sampler catalogue **419**.

## §3 — Method notes

- ⚠️ **`Copy-Item` copied nothing and reported success.** `JetBrainsMono[wght].ttf` contains `[wght]`, which
  PowerShell reads as a **wildcard character class**, so the pattern matched no file. `-LiteralPath` fixes it.
  *N-156's shape in the shell: an operation that does nothing and returns cleanly.*
- ⚠️ A `Regex.Replace` with an escaped replacement string **corrupted `skin.css` into one escaped line.** Caught
  by reading the file back rather than trusting the `@font-face` count; reverted with `git checkout` after
  confirming the only dirty tracked file was that one. *Trust the read-back, never the count.*
- An anchor guard fired correctly and wrote nothing when a here-string's LF endings met a CRLF file.

## §4 — Owed

- **M-RP-FONTS-WOFF2** — convert the two mono variable TTFs to woff2 (~400 KB saving). Size, not correctness.
- **Joe ruled the `ui/templates/` tree a DEPRECATED branch kept only for resources (2026-07-22)** — it still
  carries the old pattern (its own `@font-face` over a static Inter copy), and that is **not a defect to fix**.
  ⚠️ Its Inter copy was therefore **deliberately kept**, unlike the client and node orphans which were removed.
  🔑 **Recommended, NOT taken: mark the folder deprecated in place.** A grep of this repo hits it, and an
  instruction to ignore it lives only in a chat — *which is precisely the ACTIVE-header problem of J-568, one
  directory up: a status nobody can read gets rediscovered by whoever looks next.*
- **M-RP-SELF-VARIANTS** — ⚠️ **the three "Self" typeface variants Joe judged were ALL fallback fonts.**
  Variant 2 was never JetBrains Mono (no face was declared); variant 3 was never Inter Italic. ***A
  [👁️ PERCEPTION] verdict is only as good as whether the thing looked at was the subject*** — re-run it.
