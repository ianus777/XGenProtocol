# M-RP2.16 — `date` (di·A, atomic `<input>` date-input family fold)

> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-28  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

Author + skin the **eleventh** `core` component, `date` — the next atomic di after `range`
(N-038 catalogue row *date / time*), resuming the di-atomic series after the M-RP3 sampler arc.
Built, tuned, and CDP-verified **in the sampler** (D-097), not wired into the real shells.

**Joe-locked at the design walk (2026-06-28):**

1. **FOLD** — the five date-input siblings fold into one `date` component (the `textfield`
   fold again, not the `range` case).
2. **Whitelist = all five:** `date | time | datetime-local | month | week` (default `date`).
3. **Value:** plain string `bind:value`; empty = `''`; getter `{ type, value }`.
4. **Skin:** own `.date` key, default `type='date'`.
5. **Milestone number:** M-RP2.16.

---

## 1. Why a fold (the decision record, for the N-entry)

Sharpened D-096 criterion (root + value-type + **shared skin/surface**, the N-042 amendment):

- **Root** — all five are `<input type=…>`. Shared.
- **Value-type** — plain `bind:value` binds the `.value` **string** for every one
  (`"2026-06-28"` / `"13:45"` / `"2026-06-28T13:45"` / `"2026-06"` / `"2026-W26"`). All
  **string**. Shared — the discriminator that kept `number` separate (numeric) does not bite here.
- **Skin/surface** — identical authored box (bg/border/`--ctl-h`/text/focus/calendar-picker
  indicator) + identical prop surface. They differ **only** in UA-supplied picker chrome
  (calendar / clock / both) — exactly the `textfield` situation (UA validation/keyboard/masking).

Passes the sharpened criterion cleanly → **fold**. Contrast: `number` folds-fail on value-type,
`range` folds-fail on disjoint skin. **No `DECISIONS.md` fold-entry** — this *applies* D-096
(passes the existing sharpened criterion); no amendment.

**Honest counter (aired, not fold-breaking):** each type's string is a different structured
*format*, so a consumer must know `type` to interpret `value`. Resolved exactly as `textfield`
did — the getter carries `{ type, value }`, so `type` travels with the value through the
N-024 registry.

---

## 2. Phase-0 references (read before authoring)

- `ui/core/lib/components/data-independent/textfield.svelte` — the fold exemplar (constrained
  `type` prop, string `bind:value`, `{type,value}` getter, zero `<style>`).
- `ui/core/lib/components/data-independent/number.svelte` — value-type discriminator precedent
  (empty=`null`; own atomic).
- `ui/core/lib/components/data-independent/range.svelte` — sharpened-criterion precedent
  (own atomic on disjoint skin) + the pseudo-element verify method (N-042).
- `ui/assets/skin.css` — `.textfield` / `.number` / `.textarea` / `.select` keys (assemble-from-L2
  precedent); `color-scheme: dark` on `:root` (N-043, added **for** this family).
- `ui/docs/xgen-ui-notes.md` N-038 (track order), N-039 (textfield fold), N-042 (D-096 amendment),
  N-043 (picker-chrome pre-empt), N-045 (sampler populated).
- `DECISIONS.md` D-096 (fold criterion + amendment), D-097 / D-098 (sampler test-bed).

---

## 3. Component spec — `ui/core/lib/components/data-independent/date.svelte`

Root IS the native `<input type=…>`. Zero local `<style>` (all appearance is skin).

**Props** (TS union for `type`; enforcement = union only, no runtime guard — the D-096 / N-039
precedent: out-of-whitelist degrades safely as the browser normalizes unknown `type`):

| prop | type | default | note |
|---|---|---|---|
| `type` | `'date' \| 'time' \| 'datetime-local' \| 'month' \| 'week'` | `'date'` | the fold prop |
| `value` | `string` | `''` | string `bind:value`; empty = `''` (always-string, never `null`) |
| `disabled` | `boolean` | `false` | inert + skin-greyed |
| `readonly` | `boolean` | `false` | shown/selectable; **CDP-verify per type at build** (date-input `readonly` support is engine-variable) |
| `min` | `string` | — | native shaping attr (date/time string, type-appropriate — consumer's job) |
| `max` | `string` | — | native shaping attr |
| `step` | `number` | — | native increment (days / seconds / months per type) |
| `id` | `string` | — | |
| `name` | `string` | — | |

- **Drop** `placeholder` (native date inputs ignore it — the format hint shows instead) and
  `pattern` (no native `pattern` on these types).
- **Getter:** `const debug = () => $state.snapshot({ type, value });` — `type` carried so the
  configured type is registry-verifiable (textfield precedent).
- **Value semantics:** plain `bind:value` (string). **NOT** `bind:valueAsDate` (`Date | null`
  is serialization-hostile; string is wire-clean and matches the family). `valueAsDate` is a
  reserved future shape, not built.
- **No processor seam** — structured native value, not free-text/free-number entry (the
  numeric-formatting consumer is `number`; there are no typed digits to reformat here).
- **Native picker is the affordance** — a custom date-picker dropdown is a later **composite**,
  not this.
- `use:envelope={{ name: 'date', id, debug }}`.

---

## 4. Skin spec — add `.date` to `ui/assets/skin.css`

Own `.date` key, assembled from the L2 vocabulary (the `.number` / `.textarea` / `.select`
precedent — per-class clarity > DRY). Place after `.range`, before `.select`. **Baseline below;
Joe live-tunes via HMR (D-098).**

- Box: `min-height: var(--ctl-h)`, `--sp-1`/`--sp-2` padding+margin, `background: var(--s)`,
  `1px solid var(--s5)`, `border-radius: var(--rad)`, `color: var(--t)`, `font-size: var(--fs-1)`,
  `line-height: var(--lh)`, border/box-shadow transition — i.e. the `.number` box.
- `:focus-visible` → `border-color: var(--accent2, var(--t3))` + `box-shadow: var(--focus-ring)`.
- `:disabled` → `--s2` bg / `--t4` text / `cursor: not-allowed`.
- `:read-only` → `--s2` bg / `--s4` border.
- `:invalid` → `border-color: var(--err)` (native min/max range validation — the `.number` precedent).
- `::-webkit-calendar-picker-indicator` → `cursor: pointer`; **verify the indicator is visible on
  the dark box** (color-scheme:dark already darkens the popup; if the trigger glyph reads wrong,
  a `filter` / `opacity` tweak — Joe live-tunes). Pseudo-element → verify by **stylesheet-rule
  inspection + screenshot**, NOT `getComputedStyle` on the pseudo (N-042 method).
- No new `:root` token; no `--ctl-h` change.

---

## 5. Sampler integration (the standing DoD rule — replaces dual-shell demo wiring, D-097)

Add a `date` row to the matrix in `ui/sampler/src/app_sampler.svelte` (+ `app.css` only if the
grid needs a new row container). Plain-JS shell → bind with bare `let dateVal = $state('')`
(no TS annotations — the N-041 gotcha). Import as `Date`-free name to avoid shadowing the global
`Date` (e.g. `import DateField from '$core/.../date.svelte'`).

**Proposed cells (Joe trims at build; ragged state-map per N-045):**

| cell `id` | type | shows |
|---|---|---|
| `date#default` | `date` | baseline (empty=`''`) |
| `date#time` | `time` | the fold (type variant) |
| `date#datetime` | `datetime-local` | the fold |
| `date#month` | `month` | the fold |
| `date#week` | `week` | the fold |
| `date#disabled` | `date` | disabled state |
| `date#invalid` | `date` (min/max out of range) | `:invalid` → `--err` |

Matrix total 22 → ~29 (adjust by however many cells survive the trim).

---

## 6. CDP verification (Chat self-drives — sampler only, both accents via skin-swap)

Launch detached: `Start-Process run-sampler.ps1 -Debug -WindowStyle Minimized`; poll CDP 9422
(retry `snapshot()` until non-null — N-044 race); `cdp-debug.ps1 -App sampler -Mode …`; clean up
(ports 5175/9422 free, 0 orphans).

1. `ids().length` === new matrix total; list includes every `date#…` cell.
2. **Baseline:** `date#default` → `{type:"date", value:""}` (empty=`''`, always-string).
3. **Fold proof — bind path:** dispatch a real `input` event (N-029) with a date string on
   `date#default` → registry carries `{type:"date", value:"2026-06-28"}` (string round-trip — the
   analogue of `number`'s 42/7 / `range`'s 75/25).
4. **Fold proof — type sweep:** each cell's getter reports its own `type`
   (`date`/`time`/`datetime-local`/`month`/`week`) — one component, five types.
5. **Computed-style:** element `INPUT`, `type=date`, `min-height` `--ctl-h`, `--fs-1`, `--t`,
   `border-radius` `--rad` on `.date`.
6. **:invalid:** `date#invalid` border `--err` (`rgb(138,42,42)`); `date#default` stays `--s5`.
7. **`readonly` per type** (the build caveat): confirm behaviour across the five types.
8. **Picker indicator:** `.date` rules parsed + in cascade (stylesheet-rule inspection) + the
   indicator renders dark/visible (screenshot). No `getComputedStyle` on the pseudo (N-042).
9. **Skin-swap:** `--accent2` in the focus-ring flips gold (`--pr`, client) ↔ blue (`--inf`, node);
   both-shell screenshots eye-checked + differ.

Rules 1–5 / 2 / 4 are the load-bearing fold evidence. Quote **actual** CDP output in the JOURNAL
(Rule 2); never invent counts (Rule 5).

---

## 7. Records (D-074 atomic; written after verification, Rule 4)

- `ui/docs/xgen-ui-notes.md` — **N-046** (build + the fold-as-textfield-mirror finding + any verify
  method note).
- `docs/ROADMAP.md` — UI subtree + RP node: M-RP2.16 ✅; version bump; same-commit with CLAUDE.md.
- `CLAUDE.md` — PLAY → M-RP2.16; prior-PLAY pointer J-424 → J-425.
- `ui/docs/xgen-ui-components.md` — promote the *date / time* row to a built entry (M-RP2.16, J-425).
- **Sampler-DoD standing-rule note** (the item Joe held from the kickoff): one line recording that a
  component milestone is not done until its sampler row + applicable-state cells are CDP-verified
  there — this replaces dual-shell demo wiring. **Placement (Joe confirms at records):** a one-line
  closing note on **D-097** (its canonical home) is the recommendation; alternatively a clause in
  N-046. If D-097: this milestone *does* touch `DECISIONS.md` (one line only).
- `JOURNAL.md` — **J-425** (newest-first; quote real CDP output).
- `tasks/M_RP2_16_DATE.md` — Status → COMPLETED.
- **No other `DECISIONS.md` touch** — the fold applies D-096 (passes the sharpened criterion).

`.md` header rule: `> **Last updated**:` carries ONLY the date; "what changed" goes in the version
bump + JOURNAL.

---

## 8. Commit plan (two commits; Joe pushes — Claude never pushes)

Multi-file discipline: each file written to disk (Filesystem) + `get_file_info`-verified before the
next; `git add` per file (never `git add .`); `git status` sanity; multi-`-m` commit.

**Commit 1 — implementation** (files: `date.svelte`, `skin.css`, `app_sampler.svelte`,
`app.css` *if touched*):

```powershell
cd E:\Projects\XGenProtocol
git add ui/core/lib/components/data-independent/date.svelte
git add ui/assets/skin.css
git add ui/sampler/src/app_sampler.svelte
# git add ui/sampler/src/app.css   # only if the grid row needed it
git status
git commit -m "M-RP2.16: date — eleventh core component, date-input family fold (di.A)" -m "Folds date|time|datetime-local|month|week into one <input> atomic via a constrained type prop (default date). String bind:value (empty=''), getter {type,value}. Passes the sharpened D-096 criterion (root + value-type + shared skin/surface) — the textfield fold again, not the range case." -m "Own .date skin key assembled from L2 (.number box + calendar-picker indicator); picker chrome inherits color-scheme:dark (N-043). Built/tuned/CDP-verified in the sampler (D-097): row added, fold proven via string round-trip + per-type getter sweep, skin-swap accent verified both shells."
git push
```

**Commit 2 — records-only** (files: `xgen-ui-notes.md`, `ROADMAP.md`, `CLAUDE.md`,
`xgen-ui-components.md`, `JOURNAL.md`, `M_RP2_16_DATE.md`, and `DECISIONS.md` *iff* the D-097
note lands there):

```powershell
cd E:\Projects\XGenProtocol
git add ui/docs/xgen-ui-notes.md
git add docs/ROADMAP.md
git add CLAUDE.md
git add ui/docs/xgen-ui-components.md
git add JOURNAL.md
git add tasks/M_RP2_16_DATE.md
# git add DECISIONS.md   # only if the sampler-DoD one-liner lands on D-097
git status
git commit -m "M-RP2.16 records: N-046 + ROADMAP + CLAUDE + components registry + J-425" -m "Date fold close. Sampler-DoD made standing: a component milestone isn't done until its sampler row + cells are CDP-verified there (replaces dual-shell demo wiring)." -m "Task -> COMPLETED."
git push
```

*(Final `git add` lists confirmed against the actual touched-file set at close.)*

---

## 9. Definition of Done

- [ ] `date.svelte` authored to §3 (fold prop, string `bind:value`, empty=`''`, `{type,value}`
      getter, zero `<style>`).
- [ ] `.date` skin key added to §4 (own key, picker indicator handled).
- [ ] Sampler row + applicable-state cells added (§5) and live.
- [ ] CDP verification §6 run in the sampler, both accents — actual output captured.
- [ ] Fold evidence landed: baseline `{type:"date",value:""}`, string round-trip on the bind path,
      per-type getter sweep.
- [ ] N-046 written; ROADMAP + CLAUDE updated same-commit; components registry row promoted;
      JOURNAL J-425 written (real CDP output).
- [ ] Sampler-DoD standing-rule note recorded (placement Joe-confirmed).
- [ ] Task Status → COMPLETED.

(`Status: COMPLETED` is the real signal — no "commit pushed" checklist item.)
