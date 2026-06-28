# M-RP2.18 — `file` (di·A, atomic `<input type="file">`, own atomic; first `bind:files`/FileList shape)

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

Author + skin `file` — the **thirteenth** `core` component, the next atomic di after `color`
(N-038 catalogue row *file-select*). Built, tuned, CDP-verified **in the sampler** (D-097). The
headline is **not** fold-vs-not (own atomic is obvious) — it is the **first non-`value` binding**
in the library: `bind:files`, a **FileList** (the 4th binding shape after boolean-in `checked` /
event-out `onclick` / string-in `value`), and the first **non-serializable** value-type.

**Joe-locked at the design walk (2026-06-28):**

1. **OWN ATOMIC**, headlined by the new `bind:files` / FileList shape (applies D-096, no amendment).
2. **Getter** = `{ count, files: [{ name, size, type }] }` — de-FileLists for the registry; the
   bindable prop carries the **live FileList**.
3. **Prop surface:** `accept` / `multiple` / `disabled` / `id` / `name`.
4. **Skin:** own `.file` key, styling the file-button pseudo to match `.button`; the UA
   "No file chosen" text is accepted (minimal control).
5. **Milestone M-RP2.18**, sampler cells `file#default` + `file#multiple` + `file#disabled`.
6. **Horizon item logged:** a `file-field` / `dropzone` **composite** (drag-drop zone, selected-file
   list, remove, upload progress) — deferred, the `color-picker`/`password-field` shape.

---

## 1. Why own atomic + the new shape (for the N-entry)

Own atomic is obvious — `<input type="file">` is unique: it binds a **FileList** (not a string /
number / boolean), with file-button chrome; no fold candidate (date/color differ entirely; no
string/number siblings). Applies D-096, no amendment (no `DECISIONS.md` touch).

The **substrate question** this pass answers: the base substrate (`envelope`/`debug`, N-023/N-024)
has been proven across boolean-in / event-out / string-in / number / FileList-adjacent shapes — but
**every** prior binding rode `value`/`checked`/`onclick`. `file` is the **first `bind:files`**, and
the **first value-type that `$state.snapshot` cannot serialise** (a FileList is a live host object,
not a plain object/proxy). So the getter must **derive** plain metadata. This is the new ground.

---

## 2. Phase-0 references (read before authoring)

- `ui/core/lib/components/data-independent/color.svelte` / `range.svelte` — the own-atomic +
  no-processor-seam + pseudo-element skin precedents (N-042 verify method).
- `ui/assets/skin.css` — the **`.button`** block (the file-button pseudo must match it:
  `min-height:--ctl-h`, `padding:--sp-1 --sp-4`, `margin:--sp-1 --sp-2`, `bg:--s4`, `border:--s5`,
  `--rad`, `color:--t2`, `--fs-1`, `--lh`, `cursor:pointer`) + `.color`/`.range` (pseudo precedent).
- `ui/common/lib/components/base/envelope` + `debug` — confirm the getter runs through
  `window.__XGEN_DEBUG__`; the getter returns a **plain** object (no FileList) so CDP `returnByValue`
  round-trips.
- `ui/docs/xgen-ui-notes.md` N-024 (registry getter), N-038 (track order), N-042 (pseudo verify),
  N-047 (interactive-native-popup verify finding — relevant: read non-interactive cells / dispatch).
- `DECISIONS.md` D-096 (own-atomics list naming `file`).

---

## 3. Component spec — `ui/core/lib/components/data-independent/file.svelte`

Root IS `<input type="file">`. Zero local `<style>`.

**Props:**

| prop | type | default | note |
|---|---|---|---|
| `files` | `FileList \| null` | `null` (`$bindable`) | **`bind:files`** — the live FileList; empty = `null` |
| `accept` | `string` | `undefined` | native type filter (MIME / extension list) |
| `multiple` | `boolean` | `false` | single vs multi-file |
| `disabled` | `boolean` | `false` | inert + skin-greyed |
| `id` | `string` | — | |
| `name` | `string` | — | |

- **Drop** `value`/`placeholder`/`pattern`/`readonly`/`min`/`max`/`step`/`type` (fixed). `capture`
  (mobile camera) reserved, not built.
- **Getter (the design point):** de-FileList →
  `const debug = () => ({ count: files?.length ?? 0, files: files ? Array.from(files).map(f => ({ name: f.name, size: f.size, type: f.type })) : [] });`
  — a **plain** object (NOT `$state.snapshot` on the FileList, which won't flatten a host object).
- `bind:files` on the element; `use:envelope={{ name: 'file', id, debug }}`.
- **No processor seam** (a file pick, not typed entry).
- **`value` is unsettable** programmatically (browser security) — documented; the sampler cannot
  pre-seed a file (see §5/§6).

---

## 4. Skin spec — add `.file` to `ui/assets/skin.css`

Own `.file` key. Place after `.color`, before `.select`. **Baseline; Joe live-tunes via HMR.**

- Style the **file-button pseudo** to match `.button`, both spellings for engine coverage:
  `.file::file-selector-button` (standard) **and** `.file::-webkit-file-upload-button` (legacy) —
  `min-height:--ctl-h`, `padding:--sp-1 --sp-4`, `margin-right:--sp-2`, `bg:--s4`, `border:1px solid
  --s5`, `--rad`, `color:--t2`, `--fs-1`, `cursor:pointer`, border/bg transition.
- `:hover`/`:focus-visible` on the button pseudo → accent-tinted border (`--accent2`) where it reads
  cleanly; `:disabled` → greyed + `cursor:not-allowed`.
- The surrounding **"No file chosen"** text is UA-rendered (the `.file` element's `color`/`--fs-1`
  can nudge it, but it is not fully controllable) — **accepted**; a fully custom file row is the
  deferred `file-field` composite.
- No new `:root` token. Pseudo verified by **stylesheet-rule inspection + screenshot** (N-042).

---

## 5. Sampler integration (the standing sampler-DoD, D-097)

Add a `file` row to `ui/sampler/src/app_sampler.svelte` (plain-JS shell, bare `$state` — N-041).
Each cell binds its own `files` state (all start `null` — files are unsettable from markup):

| cell `id` | props | shows |
|---|---|---|
| `file#default` | — | single-file picker, empty "No file chosen" |
| `file#multiple` | `multiple` | multi-file picker |
| `file#disabled` | `disabled` | greyed, inert |

Matrix **31 → 34**. (No invalid/seeded variants — a file can't be pre-seeded; honest-ragged.)

---

## 6. CDP verification (Chat self-drives — sampler only, both accents via skin-swap)

Launch detached (`Start-Process run-sampler.ps1 -Debug -WindowStyle Minimized`); poll CDP 9422
(retry until non-null, N-044); `cdp-debug.ps1 -App sampler -Mode {state,eval,screenshot}`; clean up
(5175/9422 free, 0 orphans).

1. `ids().length === 34`; all three `file#` present, baseline `{count:0, files:[]}` (empty).
2. **Bind path (the FileList round-trip — the headline proof):** `value` is unsettable, so inject a
   real file via `DataTransfer` and dispatch **`change`** (file inputs fire `change`, NOT `input`):
   ```js
   const el = document.querySelector('[data-debug-id="file#default"]');
   const dt = new DataTransfer();
   dt.items.add(new File(['x'], 'test.txt', { type: 'text/plain' }));
   el.files = dt.files;
   el.dispatchEvent(new Event('change', { bubbles: true }));
   ```
   → bound getter `{ count:1, files:[{ name:"test.txt", size:1, type:"text/plain" }] }` — a FileList
   round-trips through `bind:files`, de-FileLuted to plain metadata (the substrate's first non-`value`
   binding proven).
3. **`multiple`:** `file#multiple` element `.multiple === true`; `file#default` `=== false`.
4. **Computed-style:** element `INPUT`/`type=file`; the file-button pseudo skin verified via
   **stylesheet-rule inspection** (N-042 — `getComputedStyle` won't surface `::file-selector-button`):
   `.file::file-selector-button` + `.file::-webkit-file-upload-button` rules parsed + in cascade.
5. **Disabled:** `file#disabled` `.disabled === true` + greyed.
6. **Skin-swap:** `--accent2` flips gold (`--pr2`, client) ↔ blue (`--inf2`, node).
7. **Screenshot (both shells eye-checked):** the file-button renders `.button`-styled; `#multiple`
   present; `#disabled` greyed.

Quote **actual** CDP output in the JOURNAL (Rule 2); never invent (Rule 5).

---

## 7. Records (D-074; written after verification, Rule 4)

- `ui/docs/xgen-ui-notes.md` — **N-048** (build + the `bind:files`/FileList first-non-`value`-binding
  shape + the de-FileList getter + the `change`-not-`input` + DataTransfer verify method + the
  `file-field` composite deferral).
- `docs/ROADMAP.md` — M-RP2.18 ✅ (tree node + RP narrative); version bump; same-commit with CLAUDE.
  Add the **`file-field`/`dropzone` composite** horizon.
- `CLAUDE.md` — PLAY → M-RP2.18; prior-PLAY pointer J-426 → J-427.
- `ui/docs/xgen-ui-components.md` — promote the *file-select* row to built (M-RP2.18, J-427); **add a
  `file-field` entry to the Composites section** (drag-drop + file list, deferred).
- `JOURNAL.md` — **J-427** (newest-first; real CDP output).
- `tasks/M_RP2_18_FILE.md` — Status → COMPLETED.
- **No `DECISIONS.md` touch** — applies D-096 (own atomic; the new binding shape is a substrate
  proof, not a decision); the composite is a logged horizon.

`.md` header rule: `> **Last updated**:` carries ONLY the date.

---

## 8. Commit plan (two commits, UI pattern `feat` → `docs`; Joe pushes)

Multi-file discipline: write each file (Filesystem) + `get_file_info`-verify before the next;
`git add` per file; `git status` sanity; multi-`-m` commit.

**Commit 1 — implementation** (`file.svelte`, `skin.css`, `app_sampler.svelte`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/core/lib/components/data-independent/file.svelte
git add ui/assets/skin.css
git add ui/sampler/src/app_sampler.svelte
git status
git commit -m "feat(ui): file - thirteenth core component, native file picker atomic (M-RP2.18)" -m "Atomic <input type=file>, the first non-value binding: bind:files (a FileList), the 4th binding shape after boolean-in/event-out/string-in. Own atomic (applies D-096, no amendment). Getter de-FileLists to {count, files:[{name,size,type}]}; the bindable prop carries the live FileList. Props: accept/multiple/disabled/id/name." -m "Own .file skin styling the ::file-selector-button / ::-webkit-file-upload-button pseudo to match .button; the UA no-file-chosen text is accepted (a custom file row is the deferred file-field composite). Built/tuned/CDP-verified in the sampler (D-097): file row + 3 cells (matrix 31->34), FileList round-trip via DataTransfer + change event, skin-swap accent gold<->blue."
git push
```

**Commit 2 — records** (`xgen-ui-notes.md`, `ROADMAP.md`, `CLAUDE.md`, `xgen-ui-components.md`,
`JOURNAL.md`, `M_RP2_18_FILE.md`):

```powershell
cd E:\Projects\XGenProtocol
git add ui/docs/xgen-ui-notes.md
git add docs/ROADMAP.md
git add CLAUDE.md
git add ui/docs/xgen-ui-components.md
git add JOURNAL.md
git add tasks/M_RP2_18_FILE.md
git status
git commit -m "docs(ui): close M-RP2.18 file - N-048, J-427, records" -m "N-048 (first bind:files/FileList shape + de-FileList getter + change-not-input/DataTransfer verify method) + ROADMAP M-RP2.18 done + CLAUDE PLAY -> M-RP2.18 + components registry file row promoted." -m "Logged the file-field/dropzone composite horizon (drag-drop + file list, deferred). Task -> COMPLETED. No DECISIONS touch (applies D-096)."
git push
```

*(Final `git add` lists confirmed against `git status` at close.)*

---

## 9. Definition of Done

- [ ] `file.svelte` authored to §3 (`bind:files`, de-FileList getter, props per lock).
- [ ] `.file` button-pseudo skin added to §4 (both pseudo spellings, matches `.button`).
- [ ] Sampler row + `default`/`multiple`/`disabled` cells added (§5) and live.
- [ ] CDP verification §6 run in the sampler, both accents — actual output captured (incl. the
      DataTransfer FileList round-trip).
- [ ] N-048 written; ROADMAP + CLAUDE updated same-commit; components registry row promoted +
      `file-field` composite logged; JOURNAL J-427 written (real CDP output).
- [ ] Task Status → COMPLETED.

(`Status: COMPLETED` is the real signal — no "commit pushed" checklist item.)
