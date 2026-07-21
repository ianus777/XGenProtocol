# M-RP-PROCESSOR-SEED — the starter rule set, the untouched-default migration, the absent-section repair, and the prefix-reachability diagnostic
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-21  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

Four legs. **Legs A and B are ONE user-visible outcome and MUST NOT be split** — shipping the corrected
seed without the migration fixes nobody who already has the bug.

⚠️ **THIS MILESTONE APPLIES A DECISION; IT DOES NOT TAKE ONE.** The seed value was locked by Joe in
**D-100's first AMENDMENT, dated 2026-07-04**, seventeen days before this runbook. Nobody is inventing
a seed here. See §1.1 — the reason this milestone exists at all is that the amendment was never applied
to the code.

**NOT in scope:** the render-side engine, kind 4, `{@html}`, any sanitiser, the wire, the composer
scroll, `core`'s prop-less `Record<string, Component>`. Any diff outside §6 is a Rule-6 flag, not a
tidy-up.

## §1 — Grounding (all of it re-checked against the codebase, not carried from memory)

### §1.1 ⚠️ The decision already existed and the code never received it

`DECISIONS.md` D-100, **AMENDMENT (M-RP4.3, 2026-07-04)**, on disk today:

> The seed's `-->`/`<--` pairs change to `->`/`<-` … **New seed:** `-> → | <- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒`
> … Applies to **both hand-synced Rust seed consts (client `app.rs` + sampler host `main.rs`) and the
> sampler placeholder text.**

It names **three sites**. `git log -S` proves exactly **one** was done: commit `7872a77` changed the
sampler placeholder to `type -> <- :) <3 :( -- to morph`; **neither Rust const has changed since
`7ea70fa`.** For seventeen days D-100 has said one thing and both binaries have done another, while the
sampler's own placeholder instructed users to type tokens the shipped seed does not contain.

🔑 **This is the J-561 class inverted, and worse.** J-561 was a *remembered* gate asserted as current.
This is a **real, locked, written-down decision that nobody read**, so on 2026-07-20 the defect was
rediscovered by Joe typing, Chat proposed dropping `--`, and Joe re-derived his own 4 July decision —
recorded at J-563 as a novel improvement. ***A decision that is not applied is indistinguishable from
one that was never taken, and the record then credits the rediscovery.***
→ **J-563 and D-100 are corrected in place at close (§7), pointer-style. No retroactive rewrite.**

### §1.2 The three sites, counted by grep and not by recollection

| site | state |
|---|---|
| `xgen-client/src/app.rs:155` `DEFAULT_SUBSTITUTIONS_SEED` | **OLD** — must change (Leg A) |
| `xgen-sampler/src/main.rs:30` `DEFAULT_SUBSTITUTIONS_SEED` | **OLD** — must change (Leg A) |
| `ui/sampler/src/app_sampler.svelte:586` placeholder | **ALREADY CORRECT — CHANGE NOTHING** |

⚠️ **The kickoff called Leg A "one `const`, Rust". It is TWO consts in TWO crates**, and the sampler's
own tests assert against its copy, so **both cargo floors move**. *A scope asserted from the file
someone happened to have open — the J-556 / J-560 §1.4 shape, and the reason this table exists.*

All other hits for the seed string are **records** (`ROADMAP` · `CLAUDE.md` · `DECISIONS` · `JOURNAL` ·
three task docs · `ui/docs/xgen-ui-notes.md`). **Historical entries are never rewritten.**

### §1.3 Why the bug exists, stated precisely

Seed S2 holds **both** `--` and `-->`. The edit-side engine rescans the whole field on every keystroke,
so while typing `-->` the buffer passes through `--`, which is itself a rule, and morphs to `‒`
immediately. `-->` never completes → `‒>`.

⇒ **A rule whose `find` has a proper prefix that is ALSO a rule is unreachable by sequential typing.**
Order in the rule list is irrelevant — *typing order* decides. `<--` survived only because `<-` was
not a rule. `transform.ts:60` lints **convergence** and nothing lints **reachability**: `-->` is not
*invalid*, it is *unreachable*.

The new seed's finds are `->` `<-` `:)` `<3` `:(` `--`. No find is a proper prefix of another
(`-`, `<`, `:` are not rules), so every pair is reachable. **Joe's fix is correct, and §5's V-A2 proves
it by typing rather than by argument.**

### §1.4 The seed has a SECOND job, and it is now a requirement

Joe, 2026-07-21: the pairs must exist from the first run *"also because they are examples how to define
the next ones."*

⇒ **The seed is the only worked example of the D-100 grammar anywhere in the UI.** This converts it from
convenience into a **standing requirement**: it can never be emptied, and a later seat must not "tidy
away" a hardcoded literal that looks like dead data. It also constrains the *content* — the set must
demonstrate the grammar's range, not merely be useful. The locked set does: multi-char `find`
(`->`), emoji `replace` (`🙂`), symbol `replace` (`‒`). **Written into D-100's second amendment (§4) as a
stated property so it survives the person who wrote it.**

⚠️ **This is compatible with "no hardcoded pairs", correctly understood, and that was verified by grep:**
`configs.ts` (the old `arrowMorph`/`emojiMorph` presets) is **not on disk**; there are **zero** hardcoded
rule arrays in live UI code; every path into the store is `setRules(<string>)` sourced from a config file
or the user's own typing. **No pair is ever a live rule from code.** The seed is birth content for a file
the user then owns — D-100's *"owned defaults, not locked presets"*.

### §1.5 The migration is a SKIP, not a rewrite

`clean_slate_config` (`app.rs:445`) already has the whole shape from Leg C:

```rust
let preserved = try_load_substitutions_section(config_path).map(|s| s.rules);
// … remove_file … write_fresh_config()   ← ALREADY writes the NEW seed …
if let Some(rules) = preserved { let _ = write_substitutions_section(config_path, &rules); }
```

**Migrating means NOT re-injecting.** `write_fresh_config` has already written the new seed; the
migration is a comparison and an early return. There is no upgrade path, no string surgery, no parser.

### §1.6 Two historical seeds exist — and only ONE is defective

| | value | commit |
|---|---|---|
| **S1** | `--> → \| <-- ← \| :) 🙂 \| <3 ❤️ \| :( 🙁` | `2cf494f` (M-RP4.2) |
| **S2** | `--> → \| <-- ← \| :) 🙂 \| <3 ❤️ \| :( 🙁 \| -- ‒` | `7ea70fa` (M-RP4.4) — **still shipping** |

🔑 **S1 does not have the bug.** It carries no `--` rule, so nothing is a proper prefix of `-->`, and
typing `-->` under S1 correctly yields `→`. **The defect arrived with S2**, when `--` was added beside a
rule it shadows.

### §1.7 The absent-section conflation — a live defect, found by grep

`SubstitutionsSection` derives `Default` and `rules` carries `#[serde(default)]`, so a config with **no
`[substitutions]` section at all** parses to `Some("")` — indistinguishable from *the user cleared their
pairs*. `clean_slate_config` then re-injects `""` over the freshly seeded pack.

⇒ **A pre-M-RP4.2 config launching on a current build is blanked permanently and never receives the
starter pack — including the grammar example §1.4 says it exists to provide.**

Real on this machine: **fourteen** configs carry no `[substitutions]` section, including
`instances\lp-cli\xgen-client_config.toml` (234 bytes, 2026-06-16) and thirteen fixtures under
`bin/instances/m3-*`, `m4-*`, `bin/test_01*` and `test_runs/multiparty_s1_run*`. Verified live —
`xgen-client_config.toml` (Joe), `instances\bob`, and the stale seeded copy under
`C:\cargo-targets\XGenProtocol\debug\` all read **S2**; **zero S1 configs exist here.**

*This is J-562's §3.1 conflation one level up: two states that are the same value at the point of
decision, and only the presence of the section can tell them apart.*

## §2 — Decisions (Joe-locked 2026-07-21, both D-121 lenses stated per option)

**D-a — the migration list is S2 ONLY.**
- *S2 only* — ① repairs the defect we shipped and touches nobody who is working; an S1 install falls
  through to the user-authored arm and is preserved verbatim. ② one const, one comparison, one test.
- *S1+S2* — ① repairs the same defect **and silently changes behaviour on an install with no defect**:
  after migration `--` becomes a rule, so a user who has been typing `-->` for weeks and getting `→`
  starts getting `‒>` — ***the exact defect this milestone removes, newly installed on someone who did
  not have it***, in a file they never touched. ② one extra const, one extra test.
- 🔑 **Cost was not the decider; user impact was.** The bound's principle is *untouched defaults get
  migrated*, but its **purpose** is to repair a defect we shipped. S1 is not defective, and consistency
  is not a reason to change behaviour under someone. ⚠️ **Chat recommended S1+S2 first and reversed it**
  — the first answer was correct about cost and was answering the wrong question.

**D-b — the absent-section repair is IN, as a NAMED FOURTH LEG.** ① an old config receives the starter
pack and its grammar example instead of a permanently empty box. ② one `Option` layer inside the very
function Leg B already opens; deferring means opening it twice. **Named as its own leg — not smuggled
into B.**

**D-c — D-100 gets a SECOND AMENDMENT; no new D-number.** ① no user-facing impact — said plainly.
② identical either way. The migration exists only to finish what the first amendment started; a new
number scatters one idea across two decisions.

**D-d — the reachability check is a NON-THROWING DIAGNOSTIC and must NOT join `assertSafeRules`.**
🔑 **This is the finding that decides Leg D.** `assertSafeRules` **throws**, and
`store.svelte.ts:setRules` **fails safe to empty** on rejection. A user whose list contains `--` and
`-->` would go from *five working rules and one broken one* to **zero substitutions**, explained only by
a `console.warn` stripped from release builds. ***Adding the check to the validator makes the product
strictly worse than the bug it diagnoses.*** It is computed separately, never gates Apply, and is
surfaced beside the rules — the V-C3b shape: make it visible, let the user fix their own data.

## §3 — The bound, written BEFORE the code (⚠️ Joe's, and the reason this milestone is not small)

**This is the first time the project rewrites a user's config on upgrade.** The bound is normative:

1. **BYTE-IDENTITY ONLY.** `preserved == HISTORICAL_SEED_S2`, exact string equality on the raw
   `rules` value. **Never a substring match. Never a prefix/suffix test. Never a normalisation, trim,
   parse-and-compare, or similarity heuristic. Never a rewrite of a value differing by one character.**
2. **AN EXPLICIT LIST OF NAMED CONSTANTS.** The old seed is **kept in the source as a named `const`
   because the constant IS the evidence** — it is what makes the comparison auditable rather than
   magical. It is documented as historical and never used to seed anything.
3. **A CURATED LIST IS NEVER MIGRATED.** Anything that is not byte-identical to a listed historical seed
   is **user-authored** and rides across verbatim, forever. For those users the answer is **Leg D's
   diagnostic**, never a rewrite.
4. **THE MIGRATION NEVER GROWS SILENTLY.** Adding a value to the list is a decision, recorded in D-100,
   with its own line. A future seed change that forgets to append its predecessor is a bug, and V-B4
   exists to make that visible.

## §4 — Legs

### Leg A — the seed (Rust, both crates)
1. `xgen-client/src/app.rs:155` → `-> → | <- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒`
2. `xgen-sampler/src/main.rs:30` → the same string
3. Add beside the client const: `const HISTORICAL_SEED_S2: &str = "--> → | <-- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒";`
   with a doc-comment stating it is **evidence, never seed material** (§3.2)
4. Doc-comment on the live const carrying §1.4 — the seed is also the grammar's worked example and must
   never be emptied
5. `ui/sampler/src/app_sampler.svelte:586` — **DO NOT TOUCH.** Already correct.

⚠️ Existing tests assert the old value (client ≈ `:6479`, `:6504–6560`, `:6689`; sampler `:132–160`).
**Both cargo floors move. Predict each separately and decompose any miss (N-108) — never adjust.**

### Leg B — the migration (Rust, client only)
Insert the discriminator into `clean_slate_config`'s existing `Option` arm. Target table:

| `preserved` | outcome |
|---|---|
| `None` (unreadable) | skip re-inject → fresh seed stands (unchanged, J-562) |
| `Some("")` from an explicit empty `rules` | re-inject `""` → cleared stays cleared (unchanged, J-438) |
| `Some(s)`, `s == HISTORICAL_SEED_S2` | **skip re-inject → the new seed stands** ← the migration |
| `Some(s)` otherwise | re-inject verbatim → user-authored, untouched |

### Leg C — the absent-section repair (Rust, client only)
`try_load_substitutions_section` must report *section absent* distinctly from *rules empty*. Keep
`load_substitutions_section`'s reader behaviour **byte-identical** — it is right to collapse both.
Only `clean_slate_config` may see the distinction, and it treats *absent* like a first run: **skip the
re-inject, let the starter pack stand.**

### Leg D — the reachability diagnostic (TS + editor)
1. Pure, total, non-throwing in `transform.ts`, beside `assertSafeRules` and **not inside it**:
   `findUnreachableRules(rules: TransformConfig): { find: string; shadowedBy: string }[]`
   — a rule is unreachable when another rule's `find` is a **proper prefix** of its own.
2. `substitutions-editor.svelte`: a second `$derived` and a second notice line, **separate from
   `validation`**. It must **never** gate Apply and never enter `assertSafeRules`.
3. Getter unchanged in shape; if a field is added it is task-state, never payload.

⚠️ **Appearance is Joe's.** The notice ships **PROVISIONAL** in `skin.css`, judged live by Joe at 9422,
discharged at **M-RP-SKIN** (the M-RP-SHELF-FRAME pattern, J-555). Class must not collide with the
existing `.subs-note` (the editor's save-note) or `.subs-warn` (the validation warning).

## §5 — Verification

⚠️ **V-A2 IS THE LEG THIS MILESTONE EXISTS FOR AND IT MUST BE KEYSTROKE-BY-KEYSTROKE.** Say it by name:
***a wholesale value-set verifies the transform function, not the feature — and that is precisely how
this shipped broken*** (N-154). Every seed pair is typed **one character at a time** into a live
processor-host, and the value is read after the final keystroke.

- **V-A1** both consts read the new value; `app_sampler.svelte:586` byte-unchanged (`git diff`).
- **V-A2** keystroke-by-keystroke, all six pairs, in the **client composer**
  (`ui/common/lib/components/widgets/composer-panel.svelte:114`):
  `->`→`→` · `<-`→`←` · `:)`→`🙂` · `<3`→`❤️` · `:(`→`🙁` · `--`→`‒`.
  **Positive control:** a non-rule token (`xyz`) typed the same way must come through **unmorphed**, or
  "everything morphed" and "the probe is not reading the field" are the same result (N-139).
  **Negative control:** type `-->` and assert `‒>` — *the old seed's headline pair is now simply
  `--` followed by `>`, and asserting it proves the shadowing relation is gone rather than hidden.*
- **V-B1** config holding S2 verbatim → relaunch → config holds the **new** seed. **Control:** a
  `[logging]` value altered in the same file must be wiped, proving the clean-slate ran at all.
- **V-B2** config holding **user-authored** rules (`zz yy`) → relaunch → **byte-identical**, unmigrated.
- **V-B3** config holding `rules = ""` → relaunch → still `""` (J-438 holds; the migration did not
  widen into cleared lists).
- **V-B4** config holding **S1** → relaunch → **preserved verbatim, NOT migrated** — the direct proof of
  D-a, and the leg that would catch a substring or "close enough" comparison.
- **V-C1** config with **no `[substitutions]` section** → relaunch → **the new starter pack is present**.
  **Control:** the same file with an explicit `rules = ""` must stay empty in the same session, or the
  two states have not been discriminated at all.
- **V-D1** a list containing `--` and `-->` → the notice names the pair **and Apply stays enabled and
  the other rules keep working**. ***The direct proof of D-d — the diagnostic must not blank anything.***
- **V-D2** the locked seed produces **zero** diagnostics. **Control:** V-D1's list produces exactly one.

⚠️ **Apps AND dev servers down before any static gate** — a running dev client holds `xgen-client.exe`
and `cargo test` dies on it (N-117); prove it **by port**, never by process name (N-140).
⚠️ Use a **throwaway data root** (`XGEN_DATA_DIR=E:\_xgen_redrive`) for every destructive B/C leg so
Joe's live config never enters the blast radius. A Tauri first run creates **no config at all** — seed
one with `xgen-client init --data-dir <abs> "--passphrase="` (N-150: PS 5.1 drops an empty-string
argument entirely). Fixtures written no-BOM, **assert first byte 91** (N-151).

## §6 — Files (a diff outside this list is a Rule-6 flag, not a tidy-up)

- `xgen-client/src/app.rs` — Leg A const + historical const, Leg B, Leg C
- `xgen-sampler/src/main.rs` — Leg A const
- `ui/common/lib/components/processor/transform.ts` — Leg D pure function
- `ui/common/lib/components/widgets/substitutions-editor.svelte` — Leg D surface
- `ui/assets/skin.css` — Leg D notice, PROVISIONAL ⚠️ **Joe's file**
- `DECISIONS.md` · `JOURNAL.md` · `CLAUDE.md` · `docs/ROADMAP.md` · this doc · `ui/docs/xgen-ui-notes.md`

**NOT touched:** `app_sampler.svelte` · `composer-panel.svelte` · `store.svelte.ts` · `desktop.rs` ·
anything under `xgen-node`.

## §7 — Floors

Predict each **before** driving; decompose any miss, never adjust (N-108).

| gate | at HEAD `65a1420` | expectation |
|---|---|---|
| cargo | 1549 / 0 / 62 across 56 terminator lines | **MOVES — CLIENT ONLY.** ⚠️ **This row said "both crates" and was WRONG** (Clair, Rule-6). The sampler's count **cannot** move: both its seed tests compare `DEFAULT_SUBSTITUTIONS_SEED` **to itself**, so they prove the write→read round trip and can never detect a wrong value. Measured 3 → 3. 🔑 **The hand-synced seam between the two crates' seeds (N-058) is guarded by nothing but a human reading both files** — see N-158. |
| svelte-check | 0 errors / 34 warnings / 15 files | errors stay 0; **quote both numbers** |
| npm | 142 | **MOVES** — Leg D adds tests |
| vite | 202 client / 170 sampler | client may move by reachability (N-149) |
| catalogue | 419 | |
| client registry | **149** at rest · 156 space selected · **158 room latched** (N-155) | **state which one** |

## §8 — DoD

**IMPLEMENTER**
- [x] Legs A–D per §4; scope proven by `git diff --stat` against §6
- [x] Every verification leg in §5 driven, each with its stated control
- [x] V-A2 driven **keystroke-by-keystroke** — a wholesale value-set does not discharge it
- [x] Floors predicted then measured; misses decomposed in the leg report

**[CHAT]**
- [x] `DECISIONS.md` — D-100 **second amendment**
- [x] **§1.1 correction landed in place** at J-563 and at D-100 — pointer-style, no retroactive rewrite
- [x] JOURNAL · CLAUDE.md PLAY · ROADMAP · this doc, **written before the commit command** (D-074)
- [x] `M-RP-MSG-NEWLINE-WIRE`'s outbound half folded in as a filing (J-565's owed item)

No "commit pushed" item — `Status: COMPLETED` in this header is the milestone's signal, never a leg's.

## §10 — CLOSE (J-567): measured, and what the re-drive changed

**Shipped `a0c4ec9`.** Scope re-verified independently: **6 files, +527/−32, every one named in §6,
nothing outside it**, and `app_sampler.svelte` **absent from the commit** — the "untouched" claim proven
by the diff rather than asserted.

| gate | predicted | **measured** |
|---|---|---|
| cargo | moves | **1553 / 0 / 62 across 56 terminator lines** (from 1549) — `xgen_client_lib` **181**, `xgen_sampler` **3, UNMOVED** |
| svelte-check | errors stay 0 | **0 errors / 34 warnings / 15 files** — identical |
| npm | moves | **154** (from 142) |
| vite | client may move | **202 client / 170 sampler** — unchanged |
| catalogue | — | **419**, `count === unique === domCount` |
| client registry | state which axis | **149 at rest** — quiescent, empty selection, **no room latched** (N-155) |

### 🔑 V-B1 is attributable, on a REAL config, and the control is what makes it evidence

The original V-B1 attribution was lost — the wipe control was set on
`C:\cargo-targets\XGenProtocol\debug\xgen-client_config.toml`, a **stale artifact of an older
data-root era**, not the live config (Clair's own flag, §11). Re-driven against `instances\bob`, the
last un-migrated S2 config on this machine:

| | before | after |
|---|---|---|
| `rules` | `--> → \| <-- ← \| :) 🙂 \| <3 ❤️ \| :( 🙁 \| -- ‒` | **`-> → \| <- ← \| :) 🙂 \| <3 ❤️ \| :( 🙁 \| -- ‒`** |
| `level` **(control)** | `trace` — injected | **`debug`** — back to `ClientConfig::default()` |

***Migration alone proves only that the value changed; the reverted `level` proves the clean-slate ran
and rewrote the file***, so the new seed is the result of a **skipped re-inject** and not of a path that
never touched the config. Without the control, "migrated" and "never read" are the same `rules` line.

⚠️ **The pre-state survives** at `instances\bob\xgen-client_config.bak` (317 bytes, byte-identical,
Joe's request). **It is the only surviving un-migrated S2 config on this machine — do not delete it.**
It is what lets someone who was not here verify this table.

## §11 — What the implementer found that this document got wrong

Four Rule-6 flags, all Chat's, all caught by the implementer reading the runbook against the code:

1. **§7's cargo row** — "both crates" is wrong; corrected in place. 🔑 **The sub-finding is worth more
   than the flag:** the sampler's count *cannot* move, because both its seed tests compare the const to
   itself. **A test that cannot fail is not a floor.** → N-158.
2. **§6 omitted the Leg D test file** while §7 required npm to move — an internal contradiction; flagged
   **before** building rather than resolved silently.
3. **§1.7's example** — flagged as non-existent. ⚠️ **The flag's evidence was itself wrong**: `lp-cli`
   exists (measured twice, at an absolute path under `%LOCALAPPDATA%`). Her *conclusion* was right and
   understated — **fourteen** no-section configs, not one and not thirteen. Corrected by keeping the
   example and fixing the count. 🔑 *Same root cause as her own self-flag: **data-root resolution**,
   twice in one session, in opposite directions* → N-157.
4. **§5's composer anchor** was stale (`:94` → `widgets/composer-panel.svelte:114`).

**And two the implementer raised against herself**, which is the report's most valuable content: the
mis-set wipe control above, and **two cargo baselines discarded before entering anything** (322/0/2
across 12, and 1231 across 44 — one read mid-run, betrayed by a truncated test name where a terminator
belonged). *A baseline discarded before it enters the record costs a session; one that enters it costs
a milestone.*

🔑 **The mechanism worked exactly as §6 intends.** Every one of these arrived as a **flag**, not as a
silent extra file in the diff. *A runbook that cannot be wrong is not the goal; a runbook whose
wrongness is forced into the open is.*

## §12 — Owed, not smuggled in

- **`M-RP-MSG-NEWLINE-WIRE`** — the outbound half is measured (node stored `"text":"…\n…"` verbatim,
  with a control); the **inbound render half needs M-RP6.4 backfill or a second identity**. Still open.
- The third and fourth of the five: `core`'s prop-less `Record<string, Component>` (six sites) and
  `M-RP-COMPOSER-SCROLL`.
- **M-RP-SKIN** keeps accumulating: D2/D3's three tones · the editor note wording (**still no verdict**) ·
  M-RP6.6 ConnStats row-swap · M-RP-FOCUS · Send-as-icon (blocked on a verified glyph) · **and now
  Leg D's notice.**
