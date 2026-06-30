# M-RP3.2 — sampler tabs (class×arity panels; di·atomic / di·composite / dd·atomic / dd·composite)

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

Restructure the **sampler host** (`ui/sampler/`, D-098) from one long vertical scroll into a
**four-panel tab container** keyed by the catalogue's class×arity axes. Pure sampler chrome — **no
`core`/`common` component touched, no `skin.css` touched** (D-097/D-098 test-bed-only). Isolated as its
own M-RP3.x sampler milestone (sibling to M-RP3.0 scaffold / M-RP3.1 populate) so the next
component-build (M-RP2.22 `status-indicator`) drops cleanly into an already-tabbed panel 2.

**Joe-locked decisions (this session):**

1. **Four tab panels, by class×arity** — mirrors the component-index's own structure (di atomics, di
   composites, dd atomics, dd composites):
   - **DI · atomic** — all 16 currently-built components (44 cells).
   - **DI · composite** — empty now; first occupant is `status-indicator` (M-RP2.22).
   - **DD · atomic** — empty now.
   - **DD · composite** — empty now.
2. **All panels stay MOUNTED; inactive hidden via CSS `display:none` — never `{#if}`** (the load-bearing
   call). `envelope` registers into the flat `window.__XGEN_DEBUG__` **only while a component is mounted**
   (grounded — `envelope.ts` registers on the `use:` action firing). If tabs unmounted inactive panels,
   `ids()` would read only the active tab and the whole matrix-count verify breaks. CSS-hidden panels stay
   mounted → registry stays complete → CDP self-drive is unchanged (no tab-clicking to register
   everything). A test-bed wants everything alive anyway.
3. **Client/node skin-swap stays GLOBAL tool chrome above the tabs** (the existing `.sampler-bar`,
   untouched). The tab bar sits between the skin-swap bar and the panels.
4. **In-panel kind sub-headers retained as the inner grouping.** Tabs are the outer class×arity axis; the
   kind headers stay *inside* the DI·atomic panel. Promote the three di kinds to explicit sub-headers:
   **INTERACTIVE** (toggle…file) / **DISPLAY** (label/paragraph/image/led) / **NAVIGATION** (link). This
   aligns the panel with the catalogue's three di kinds (interactive / display / navigation, N-049/N-052);
   `link` moves from under "Display" to its own NAVIGATION sub-header. **No cell added/removed/re-bound** —
   only a section-title split. Matrix stays **44**.
5. **Empty panels get an explicit empty-state** ("No components yet" + a one-line hint) so the three empty
   panels read as intentional placeholders, not broken/blank.
6. **Canonical tab labels** track the index vocabulary — **DI · atomic / DI · composite / DD · atomic /
   DD · composite** (the index uses *atomic*, not *single*).

**Milestone M-RP3.2.** Matrix unchanged (44 cells, all in DI·atomic). No new component.

---

## 1. Why this shape (for the N-entry)

The sampler is already a long single scroll at 16 components; composites + dd will make it unmanageable.
A class×arity tab container makes the sampler finally reflect the **catalogue's own shape** (the
component-index's di-atomic / di-composite / dd subsections). The one non-obvious engineering call is
**all-mounted, CSS-hidden, never `{#if}`** — it preserves the CDP registry-completeness invariant the
whole verify protocol (D-097, the matrix count) depends on. `{#if}`-gated tabs would silently break
`ids().length` reads; this milestone fixes the taxonomy without touching that invariant.

---

## 2. Phase-0 references (read before authoring)

- `ui/sampler/src/app_sampler.svelte` — the current shell: `.sampler-bar` (skin-swap, stays global) +
  one `.sampler-body` holding `.s-section-title` ("Interactive"/"Display") + `.s-row`s. The restructure
  wraps the body content into four panels and splits the kind headers.
- `ui/sampler/src/app.css` — the sampler chrome stylesheet (`.sampler-bar`, `.sampler-seg`,
  `.sampler-body`, `.s-section-title`, `.s-row`, …). Tab styles are added here; the swap-control + grid
  styles are untouched.
- `ui/common/lib/components/base/envelope.ts` — confirms registration fires on the `use:` action while
  mounted (the basis for decision 2: CSS-hidden ≠ unmounted, so the registry stays complete).
- `ui/docs/xgen-ui-components.md` — the test-bed callout block (M-RP3.0/D-097/D-098) + the catalogue
  structure the four panels mirror.
- `ui/docs/xgen-ui-notes.md` — N-044/N-045 (sampler scaffold + populate), N-049/N-052 (the di kinds:
  interactive / display / navigation).

---

## 3. Implementation spec

### 3.1 `ui/sampler/src/app_sampler.svelte`

- **Tab state (bare `$state`, plain-JS shell — N-041, no TS annotations):**
  ```js
  let activeTab = $state('di-atomic'); // 'di-atomic' | 'di-composite' | 'dd-atomic' | 'dd-composite'
  const tabs = [
    { id: 'di-atomic',    label: 'DI · atomic' },
    { id: 'di-composite', label: 'DI · composite' },
    { id: 'dd-atomic',    label: 'DD · atomic' },
    { id: 'dd-composite', label: 'DD · composite' },
  ];
  ```
- **Markup order:** existing `.sampler-bar` (untouched) → new `.sampler-tabs` bar → four
  `.sampler-panel` divs.
- **Tab bar:** a `<div class="sampler-tabs" role="tablist">` of `<button>`s; active gets `class:active`;
  `onclick={() => activeTab = t.id}`. (Tool chrome, like `.sampler-seg` — deliberately NOT a sampled
  `.button`, preserving the N-028 tool-vs-sampled line.)
- **Panels (ALL mounted; inactive hidden via CSS, decision 2):**
  ```svelte
  <div class="sampler-panel" class:hidden={activeTab !== 'di-atomic'}>   … current grid … </div>
  <div class="sampler-panel" class:hidden={activeTab !== 'di-composite'}> {empty-state} </div>
  <div class="sampler-panel" class:hidden={activeTab !== 'dd-atomic'}>    {empty-state} </div>
  <div class="sampler-panel" class:hidden={activeTab !== 'dd-composite'}> {empty-state} </div>
  ```
  Use `class:hidden` (toggling a `display:none` rule), **not** `{#if}` — mount-preserving.
- **DI·atomic panel body** = the existing `.sampler-body` content **verbatim**, with the kind-header
  split (decision 4): `INTERACTIVE` (toggle, button, textfield, select, select-multiple, textarea,
  number, range, date, color, file) → `DISPLAY` (label, paragraph, image, led) → `NAVIGATION` (link).
  Every `.s-row` / `.s-cell` / `id` / `bind:` is unchanged — only the `link` row relocates under a new
  `NAVIGATION` `.s-section-title`. Keep the inner `.sampler-body` wrapper (or apply its layout to the
  panel) so spacing is identical.
- **Empty-state** (the three empty panels) = a small `<div class="s-empty">` with e.g.
  `No components yet` + a muted hint line (`Composite di components land here, starting with
  status-indicator (M-RP2.22).` etc., tuned per panel).
- The skin-swap script (`shell`, `applyShell`, `onMount`) and all imports/bound `$state` are **unchanged**.

### 3.2 `ui/sampler/src/app.css`

Add (after the `.sampler-bar`/`.sampler-seg` block, before or around `.sampler-body`):

- `.sampler-tabs` — a horizontal bar of tab buttons: `display:flex`, `gap`, `padding`, `background:var(--s2)`,
  `border-bottom:1px solid var(--s5)`, sticky under the bar if desired (`position:sticky; top:<bar-height>`)
  — Joe live-tunes via HMR.
- `.sampler-tabs button` — tool-chrome styling consistent with `.sampler-seg button` (muted `--t2`/`--s3`,
  small font, `cursor:pointer`); `.sampler-tabs button.active` → `--accent`/`--accent-ink` (so the active
  tab also re-themes on skin-swap, the `.sampler-seg.active` precedent).
- `.sampler-panel` — the panel wrapper; `.sampler-panel.hidden { display: none; }` (the mount-preserving
  hide — decision 2).
- `.s-empty` — centred muted placeholder (`color:var(--t3)`, padding, small font) for the empty panels.
- **No `:root` token added; `skin.css` untouched.** All values PROVISIONAL (HMR-tuned).

---

## 4. CDP verification (Chat self-drives — sampler only)

Launch detached minimized; poll 9422 (retry until non-null); **fresh launch** (avoid stale HMR, the
J-430 finding); `cdp-debug.ps1 -App sampler -Mode {state,eval,screenshot}`; teardown (5175/9422 free,
0 orphans).

1. **Registry complete with default tab active (THE load-bearing proof, decision 2):** on load (DI·atomic
   active, the other three panels CSS-hidden-but-mounted), `ids().length === 44` — the full matrix
   enumerates **without clicking any tab**. Spot-check a hidden-panel-independent id set is unchanged from
   J-432 (e.g. `link#default`, `file#multiple`, `led#unknown` all present).
2. **Tabs are CSS-hidden, not unmounted (the anti-`{#if}` proof):** one eval — read each panel's
   `getComputedStyle(panel).display`: DI·atomic `!== "none"`, the other three `=== "none"` on load; then
   set `activeTab` via the DOM/click to `di-composite` and re-read — DI·composite now visible, DI·atomic
   `none`; **and `ids().length` is STILL 44** through the switch (mounted regardless of active tab). This
   is the proof that `{#if}` was correctly avoided.
3. **Empty panels:** DI·composite / DD·atomic / DD·composite each contain the `.s-empty` placeholder text
   (queried in the DOM).
4. **Kind sub-headers (decision 4):** the DI·atomic panel shows three `.s-section-title`s —
   `INTERACTIVE` / `DISPLAY` / `NAVIGATION`; `link#*` sits under `NAVIGATION` (DOM order check).
5. **Skin-swap still global across tabs:** flip `[data-shell]` client↔node — the active tab button +
   accent-derived components re-theme; confirm on at least two tabs (`.sampler-tabs button.active`
   `background` differs gold↔blue, the `.sampler-seg` precedent).
6. **Screenshot (eye-check):** the tab bar renders 4 tabs under the skin-swap bar; DI·atomic shows the
   full grid; switch to each empty tab and confirm the placeholder renders (not a blank/broken panel).

Quote **actual** CDP output in the JOURNAL (Rule 2); never invent (Rule 5).

---

## 5. Records (D-074; written after verification, Rule 4)

- `ui/docs/xgen-ui-notes.md` — **N-053** (the tab restructure; the four-panel class×arity taxonomy
  mirroring the catalogue; **the all-mounted / CSS-`display:none` / never-`{#if}` decision and WHY**
  — it preserves the CDP registry-completeness invariant D-097's matrix count depends on; the NAVIGATION
  sub-header promotion). v0.35 → v0.36.
- `ui/docs/xgen-ui-components.md` — update the test-bed callout block to note the sampler is now tabbed by
  class×arity (di·atomic / di·composite / dd·atomic / dd·composite), DI·atomic holding the current 44-cell
  grid. v0.27 → v0.28.
- `docs/ROADMAP.md` — M-RP3.2 ✅ on the RP node + Present narrative; version bump (v4.08 → v4.09);
  same-commit with CLAUDE.
- `CLAUDE.md` — PLAY → M-RP3.2 (sampler tabs); prior-PLAY pointer → J-433. Next-active reads
  `status-indicator` (M-RP2.22, drops into the di·composite panel).
- `JOURNAL.md` — **J-433** (newest-first; real CDP output, incl. the `ids().length===44`-through-switch
  proof).
- `tasks/M_RP3_2_sampler_tabs.md` — Status → COMPLETED.
- **No `DECISIONS.md` touch** — sampler chrome, arc-local (D-097/D-098). The all-mounted/never-`{#if}`
  invariant is recorded as N-053; a D-069 promotion-watch only if it recurs as a cross-cutting rule.

`.md` header rule: `> **Last updated**:` carries ONLY the date.

---

## 6. Commit plan (two commits, UI pattern `feat` → `docs`; Joe pushes)

**Commit 1 — implementation** (`app_sampler.svelte`, `app.css`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/sampler/src/app_sampler.svelte
git add ui/sampler/src/app.css
git status
git commit -m "feat(sampler): tab the matrix by class x arity - di/dd x atomic/composite (M-RP3.2)" -m "Four-panel tab container (DI-atomic / DI-composite / DD-atomic / DD-composite) keyed to the catalogue's class x arity axes; client/node skin-swap stays global above the tabs. All panels stay MOUNTED, inactive hidden via CSS display:none (class:hidden), NEVER {#if} - preserves the window.__XGEN_DEBUG__ registry-completeness invariant the CDP matrix count (D-097) depends on. DI-atomic holds the current 16 components / 44 cells with INTERACTIVE / DISPLAY / NAVIGATION sub-headers (link promoted to its own NAVIGATION header); the three empty panels carry an explicit no-components-yet placeholder." -m "Pure sampler chrome (D-098): no core/common component touched, skin.css untouched. Matrix unchanged at 44. CDP-verified: ids().length===44 with default tab active AND through a tab switch (mounted regardless of active tab - the anti-{#if} proof); inactive panels computed display:none; empty-state placeholders present; skin-swap re-themes across tabs."
git push
```

**Commit 2 — records** (`xgen-ui-notes.md`, `xgen-ui-components.md`, `ROADMAP.md`, `CLAUDE.md`,
`JOURNAL.md`, `M_RP3_2_sampler_tabs.md`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/docs/xgen-ui-notes.md
git add ui/docs/xgen-ui-components.md
git add docs/ROADMAP.md
git add CLAUDE.md
git add JOURNAL.md
git add tasks/M_RP3_2_sampler_tabs.md
git status
git commit -m "docs(sampler): close M-RP3.2 sampler tabs - N-053, J-433, records" -m "N-053 (four-panel class x arity taxonomy mirroring the catalogue + the all-mounted / CSS-display:none / never-{#if} decision and why it preserves CDP registry-completeness + the NAVIGATION sub-header promotion) + components test-bed callout updated + ROADMAP M-RP3.2 done + CLAUDE PLAY -> M-RP3.2." -m "Next-active: status-indicator (M-RP2.22, the first di composite) drops into the di-composite panel. Task -> COMPLETED. No DECISIONS touch (sampler chrome, arc-local D-097/D-098)."
git push
```

---

## 7. Definition of Done

- [ ] `app_sampler.svelte` restructured to §3.1: `activeTab` state + `.sampler-tabs` bar + four
      `.sampler-panel`s (`class:hidden`, not `{#if}`); DI·atomic = the existing grid with
      INTERACTIVE/DISPLAY/NAVIGATION sub-headers (link under NAVIGATION); three empty-state panels;
      skin-swap + imports + bound `$state` unchanged.
- [ ] `app.css` gains `.sampler-tabs` / `.sampler-tabs button(.active)` / `.sampler-panel(.hidden)` /
      `.s-empty` (§3.2); `skin.css` untouched; no `:root` token added.
- [ ] CDP verification §4 run in the sampler — actual output captured, incl. **`ids().length===44` with
      the default tab active AND held through a tab switch** (the mount-preserving proof) + inactive panels
      computed `display:none` + empty-state present + skin-swap across tabs.
- [ ] N-053 written; components test-bed callout updated; ROADMAP + CLAUDE updated same-commit;
      JOURNAL J-433 written (real CDP output).
- [ ] Task Status → COMPLETED.

(`Status: COMPLETED` is the real signal — no "commit pushed" checklist item.)
