# M-RP4.2 — user-owned substitution pairs: one list in the client TOML → parse → store → processor-hosts

> **Status**: ACTIVE
> Version: 0.1
> Date: Jun 2026
> **Last updated**: 2026-06-30
> Language: English
> Author: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> License: BSL 1.1 (converts to GPL upon project handover)

---

## 0. What this is

Retire the demo presets (`arrowMorph`/`emojiMorph`) and give the user **one** list of
substitution pairs, owned in **settings** (the client TOML), flowing through the kind-1
transformer built at M-RP4.0. The engine does NOT change — this arc adds the *source* (TOML)
and the *plumbing* (parser + reactive store + frontend delivery).

`arrowMorph`/`emojiMorph` were never architecture — they were sample data (D-099/N-056). The
durable model is: **one ordered list of `{find, replace}` pairs, owned by the user.** This is
decision 9 of the M-RP4.0 runbook, now executed.

**Joe-locked decisions (this arc's design walk):**

1. **One source of truth.** A single string of pairs. No themes, no presets, no merging. The
   `configs.ts` named presets are retired as the *source* (kept only if useful as seed/demo data;
   not wired as the live source).
2. **TOML home = a single string.** A new `[substitutions]` section in `xgen-client_config.toml`
   holding ONE string field `rules`, so it mirrors the future one-textarea editor 1:1:
   ```toml
   [substitutions]
   rules = "--> → | <-- ← | :) 🙂 | <3 ❤️ | :((( 🙁🙁🙁"
   ```
   (Not a TOML array — a single string, parsed by the UI.)
3. **The grammar (locked, literal — no regex):**
   - the whole list is one string; pairs separated by the literal **` | `** (space-pipe-space)
   - within a pair: split on the **first space** → `find` = before, `replace` = everything after
   - `find` = any string with no whitespace; `replace` = any string at all (multi-char, emoji,
     phrase-with-internal-spaces, a lone `|`, e.g. `:| 😐` or `brb be right back`)
   - the ONLY forbidden substring in a token is the literal ` | ` (space-pipe-space) itself
   - blank pairs (empty after split) are skipped
4. **Source-agnostic store.** The parser + reactive store live in `$common` and take a string
   from anywhere. The real client feeds it via a Tauri command; the sampler seeds it with a
   literal. The engine stays source-agnostic (D-099 P-3 / decision 9).
5. **Provenance = Tier-2 (`trusted:false`).** Config-file rules are user data: run
   `assertSafeRules({trusted:false})` so the caps + convergence lint actually protect the user
   from a self-authored looping pair (e.g. `a aa`). On a bad set, fail safe (empty + DEV warn);
   the per-pair partition + inline warnings UX is M-RP4.3 (the editor needs it; the file-edit
   path does not).
6. **Live-as-you-type stays.** Still literal convergent pairs → no caret problem, no trigger
   change. The kind-1 attachment is unchanged.
7. **Chat/Clair split (this arc crosses the boundary):** Chat owns the `$common` parser + store +
   sampler rewire (CDP-verifiable). Clair owns the Rust config struct + the Tauri command + the
   `ui/client` boot hydration (client-only; not sampler-verifiable).

**Milestone M-RP4.2** (read path). **M-RP4.3** (the in-app editor + TOML write-back) is the next arc.

---

## 0.1 The grammar — worked example

```
--> → | <-- ← | :) 🙂 | <3 ❤️ | :((( 🙁🙁🙁 | :| 😐 | brb be right back
```

`split(" | ")` →
| raw pair | first space at | find | replace |
|---|---|---|---|
| `--> →` | 3 | `-->` | `→` |
| `<-- ←` | 3 | `<--` | `←` |
| `:) 🙂` | 2 | `:)` | `🙂` |
| `<3 ❤️` | 2 | `<3` | `❤️` |
| `:((( 🙁🙁🙁` | 4 | `:(((` | `🙁🙁🙁` |
| `:| 😐` | 2 | `:|` | `😐` (the lone `|` survives — no surrounding spaces) |
| `brb be right back` | 3 | `brb` | `be right back` (replace keeps internal spaces) |

Then the existing `applyRules` (literal split/join, multi-char `replace` already works) and
`assertSafeRules` run unchanged.

---

## 1. Why this shape (for the N-entry)

Founds the **source-agnostic rule store**: the engine built at M-RP4.0 had hardcoded named
configs; now the rules come from one user-owned string, and the store decouples *where the rules
come from* (TOML via Tauri / sampler literal / future editor) from *who consumes them* (every
processor-host). The ` | ` + first-space grammar is the simplest entry that survives the data
(tokens contain `=>`, `<--`, `:)`, `|`) without regex — the literal engine stays literal.

---

## 2. Phase-0 references (grounded, read before authoring)

**Frontend (Chat):**
- `ui/common/lib/components/processor/transform.ts` — `applyRules` + `assertSafeRules` +
  `TransformRule`/`TransformConfig`. The parser produces a `TransformConfig`; no engine change.
- `ui/common/lib/components/processor/configs.ts` — the presets being retired as the live source.
- `ui/common/lib/components/processor/processor.ts` — `processor(rules, opts)`; hosts will call it
  with `store.rules` instead of a named config.
- `ui/sampler/src/app_sampler.svelte` — currently `{...processor(arrowMorph,{trusted:true})}` on
  `textarea#processed`; rewires to read the store.
- `ui/common/lib/components/base/` — the `$common` module convention for the new store + parser.

**Backend (Clair) — grounded in this session:**
- `xgen-client/src/app.rs` lines 56–208 — `ClientConfig` + per-section structs
  (`ClientSection`/`PathsSection`/`LoggingSection`/`SyncSection`/`AiSection`) + `Default for
  ClientConfig` + `load_sync_section(config_path)`. The `[substitutions]` section is a verbatim
  copy of the `[sync]` precedent.
- `xgen-client/src/desktop.rs` lines 53–96, 254 — `#[tauri::command]` shape (`get_state`,
  `get_pacing_state`, `quit`) + `invoke_handler(tauri::generate_handler![...])`. The new
  `get_substitutions` command registers here. `config_path = data_dir.join("xgen-client_config.toml")`
  is already derived in `run_startup` (line ~140).
- `ui/client/` — the Svelte shell; the boot `invoke('get_substitutions')` → `store.setRules(...)`
  lands here (mount step).

---

## 3. Chat half — `$common` parser + store (CDP-verifiable in the sampler)

### 3a. `parseRules(text: string): TransformConfig` — in `transform.ts` (pure, framework-free)

```ts
// One string → ordered pairs. Pairs split on the literal " | " (space-pipe-space);
// within a pair, split on the FIRST space → find (before) | replace (rest). Blank pairs skipped.
// find = no whitespace; replace = any string (incl. a lone '|', internal spaces, multi-char).
export function parseRules(text: string): TransformConfig {
  const out: TransformConfig = [];
  for (const pair of text.split(' | ')) {
    const i = pair.indexOf(' ');
    if (i < 0) continue;               // no space → not a pair → skip
    const find = pair.slice(0, i);
    const replace = pair.slice(i + 1);
    if (find.length === 0) continue;   // empty find → skip (assertSafeRules also rejects)
    out.push({ find, replace });
  }
  return out;
}
```

Pure (no DOM, no framework) — lives next to `applyRules` so the DEV `__XGEN_PROC__` hook can
expose it for CDP. (The inverse `stringifyRules(config): string` is M-RP4.3, for the editor.)

### 3b. The reactive store — new `$common` module (e.g. `processor/store.svelte.ts`)

```ts
import { parseRules } from './transform';
import { assertSafeRules } from './transform';
import type { TransformConfig } from './transform';

// Source-agnostic: the client feeds raw TOML text via Tauri; the sampler seeds a literal.
let _rules = $state<TransformConfig>([]);

export const substitutions = {
  get rules() { return _rules; },
  // Replace the whole list from a raw string (the one-textarea / TOML shape).
  setRules(text: string) {
    const parsed = parseRules(text);
    try {
      assertSafeRules(parsed, { trusted: false }); // Tier-2: caps + convergence lint
      _rules = parsed;
    } catch (e) {
      if (import.meta.env.DEV) console.warn('[substitutions] rejected rule set:', e);
      _rules = []; // fail safe (M-RP4.3 adds per-pair partition + inline warnings)
    }
  },
};
```

Hosts read `substitutions.rules` and pass it to `processor(...)`. New string → new `TransformConfig`
→ the attachment lifecycle re-runs (D-099). (Exact filename/rune shape per the `$common` convention;
confirm at author time.)

### 3c. Sampler rewire (`app_sampler.svelte`)

- Drop the `arrowMorph` import; import `substitutions` from the store.
- On setup, seed the store with the demo string (proves the parse→store→processor loop without a
  client config): `substitutions.setRules("--> → | <-- ← | :) 🙂 | <3 ❤️ | :((( 🙁🙁🙁")`.
- `textarea#processed` → `{...processor(substitutions.rules, { trusted: true })}` (trusted here
  because the store already validated as Tier-2; the host re-call is on already-vetted rules).
- Matrix stays **56** (no new cell; the cell now sources from the store).

---

## 4. Clair half — Rust config + Tauri command + client hydration (client-only)

### 4a. `xgen-client/src/app.rs` — the `[substitutions]` section (verbatim `[sync]` precedent)

```rust
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SubstitutionsSection {
    /// One string of pairs (see M-RP4.2 grammar): pairs separated by " | ",
    /// each pair "find replace" split on the first space. Default empty.
    #[serde(default)]
    pub rules: String,
}
```
- Add `#[serde(default)] substitutions: SubstitutionsSection` to `ClientConfig`.
- Add `substitutions: SubstitutionsSection::default()` to `Default for ClientConfig`.
- Add `pub fn load_substitutions_section(config_path: &Path) -> SubstitutionsSection`,
  mirroring `load_sync_section` (read file → `toml::from_str::<ClientConfig>` → `.substitutions`,
  default on any error). `#[serde(default)]` keeps every existing on-disk config parsing.

### 4b. `xgen-client/src/desktop.rs` — the Tauri command

```rust
/// Returns the raw substitution-rules string from xgen-client_config.toml
/// (M-RP4.2). The Svelte layer parses it (split on " | ", first-space per pair)
/// and feeds the processor store. Empty string when absent.
#[tauri::command]
fn get_substitutions(/* config_path via managed state or data_dir */) -> String {
    crate::app::load_substitutions_section(&config_path).rules
}
```
- Register in `invoke_handler(tauri::generate_handler![get_state, get_pacing_state, quit,
  get_substitutions])`.
- Supply `config_path`: either manage `data_dir`/`config_path` as Tauri state, or recompute via the
  same `data_dir.join("xgen-client_config.toml")` used in `run_startup`. Clair's call.

### 4c. `ui/client` — boot hydration

On mount (alongside the existing `get_state` invoke), call `invoke('get_substitutions')` →
`substitutions.setRules(result)`. Client-only; the sampler does not have this command.

---

## 5. CDP verification (Chat self-drives — sampler)

Launch detached; poll 5175/9422; fresh launch; split dispatch from read by a tick (J-433);
teardown to 0 orphans. Quote actual output (Rule 2).

1. **Parser (DEV hook):** `__XGEN_PROC__.parseRules("--> → | :| 😐 | brb be right back")` →
   `[{find:"-->",replace:"→"},{find:":|",replace:"😐"},{find:"brb",replace:"be right back"}]`
   (the lone-`|` + internal-spaces proof).
2. **Store seeded:** the sampler seeds the demo string; read the store → the parsed config.
3. **Live morph from store:** type `--> :) <3` into `textarea#processed`, dispatch input, tick,
   read → `→ 🙂 ❤️` in DOM AND registry (the rules came from the store, not a hardcoded config).
4. **Store update re-morphs:** call `substitutions.setRules("foo BAR")` (new list), then a fresh
   morph uses the new rules and NOT the old arrow rules (proves the store is the live source).
5. **Tier-2 guard:** `substitutions.setRules("a aa")` (a convergent-loop pair) → store stays empty
   (or last-good) + DEV warn (the lint fired on user data).
6. **Count unchanged:** `ids().length === 56` (no new cell).

(The Clair half — Rust loader + `get_substitutions` + client hydration — is verified by a Rust
unit test on `load_substitutions_section` and by Joe in the real client; it is NOT sampler-CDP'able.)

---

## 6. Records (D-074; after verification)

- `ui/docs/xgen-ui-notes.md` — **N-057** (the source-agnostic rule store; the ` | ` + first-space
  grammar; presets retired as the live source; Tier-2 on config data; the Chat/Clair split; the
  sampler-seeds-vs-client-invokes source duality). Version bump.
- `DECISIONS.md` — amend/extend **D-099** OR a new **D-100** (the substitution-pairs grammar +
  TOML-single-string home + source-agnostic store). Decide at close which (grammar may be
  arc-local under D-099; the TOML home is a real new decision). `Last updated` bump.
- `docs/ROADMAP.md` — M-RP4.2 ✅; M-RP4.3 🟡 (editor + write-back); version bump; CLAUDE same-commit.
- `CLAUDE.md` — PLAY → M-RP4.2; prior-PLAY pointer; next-active → M-RP4.3.
- `JOURNAL.md` — **J-NNN** (newest-first; real CDP output; note the Clair-half handoff state).
- `ui/docs/xgen-ui-components.md` — `textarea` processor-host note: source is now the user list
  (store), not a preset. Version bump iff edited.
- `xgen-client_config.toml` instances — a sample `[substitutions]` line in the dev instances
  (optional, demo convenience).
- `tasks/M_RP4_2_SUBSTITUTIONS.md` — Status → COMPLETED (when both halves land; if staged, note
  the Chat-half-done / Clair-half-pending state honestly, D-065).

`.md` header rule: `> **Last updated**:` carries ONLY the date.

---

## 7. Commit plan (UI pattern; Joe pushes)

Likely staged because two agents touch it:
- **Chat commit 1 (feat):** `transform.ts` (+`parseRules`), the store module, `app_sampler.svelte`
  rewire. CDP-verified in the sampler.
- **Clair commit (feat):** `xgen-client/src/app.rs` (+`SubstitutionsSection`/`load_substitutions_section`),
  `desktop.rs` (+`get_substitutions`), `ui/client` boot hydration. Rust test + Joe client check.
- **Records commit (docs):** N-057 + D-099/D-100 + ROADMAP + CLAUDE + JOURNAL + components + task.

Exact `git add` lists authored at close per the standing PowerShell discipline (one `git add` per
file; multiple `-m` flags; `$ProgressPreference='SilentlyContinue'`; Joe pushes).

---

## 8. Definition of Done

- [x] `parseRules` in `transform.ts` (pure; ` | ` split + first-space; blanks skipped); DEV hook exposes it.
- [x] `$common` reactive `substitutions` store (`setRules`, Tier-2 `assertSafeRules`, fail-safe).
- [x] Sampler `app_sampler.svelte` rewired: `arrowMorph` retired as source; store seeded; `#processed` reads store; matrix 56.
- [x] CDP §5 run (parser, store-sourced morph, store-update re-morph, Tier-2 guard, count 56) — actual output captured.
- [x] Clair: `SubstitutionsSection` + `load_substitutions_section` (sync-precedent) + Rust unit test on the loader. *(J-437: 4 loader tests green; lib 123→127; `cargo build -p xgen-client` clean.)*
- [~] Clair: `get_substitutions` Tauri command registered; `ui/client` boot hydration. **Code landed + statically gated (J-437; `npx vite build` ✓ 122 modules);** `Joe-verified in the real client` is the remaining gate (NOT yet run — add a `[substitutions]` line by hand; no on-disk dev config in the repo).
- [ ] Delete `configs.ts` (orphaned once presets retired — held as reference through Clair's build; removed at the true close).
- [~] Records: **J-437 + this DoD written (Clair).** N-057 / D-099-vs-D-100 / ROADMAP / CLAUDE / components / task→COMPLETED are the **canonical close (Chat)**, after Joe verifies — per §0 decision 7 and the J-437 handoff.
- [x] Presets retired as the live source; one user-owned list is the only source.
- [ ] Task Status → COMPLETED (or honest staged state if one half lands first).

(`Status: COMPLETED` is the real signal — no "commit pushed" checklist item. If the Chat half ships
before Clair's, record that split honestly rather than marking the milestone closed.)

---

## 9. M-RP4.3 preview (next arc — NOT this one)

The in-app editor: one textarea bound to the rules string, `parseRules` on change,
`assertSafeRules({trusted:false})` with **per-pair partition + inline warnings** (the richer
validation deferred from §0 decision 5), `stringifyRules` for display, and a **Tauri write-back
command** (Clair) persisting to the `[substitutions]` section (the frontend can't write files).
That's where "user puts his preferred pairs in settings" gets its form.
