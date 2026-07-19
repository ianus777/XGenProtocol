# M-RP-TYPECHECK — a type gate for `ui/**`
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-19  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS MILESTONE IS

`ui/**` runs no typecheck. `npm test` and both `vite build`s prove RUNTIME and BUNDLING and nothing
else — **a type error cannot fail any gate on this project** (N-138). This milestone admits a typecheck
to the gate set, burns the backlog it exposes to zero, and proves the gate can go red.

It ships **ZERO appearance** and **ZERO `.rs`**.

**Taken BEFORE Leg D2 on Joe's sequencing call.** D2 is a new store, a new component, and an
id-stitching path where a local id is replaced by a real `event_id` at outcome — the largest
TypeScript surface this arc has produced, and D2's store is `$common`, which sits inside the
never-checked half.

---

## §1 — GROUNDING (Phase-0, J-557). CONFIRMED vs UNCONFIRMED, kept apart on purpose.

### §1.1 N-138 verified against the codebase, not inherited — TRUE, and it undercounts

All three `ui/**/package.json` read (`client`, `node`, `sampler` — the only ones). No `svelte-check`,
no `tsc`, no `check` script. **And no `typescript` dependency anywhere in the tree.**

Three things N-138's filed proposal assumes that are **not true**:

- It proposes `svelte-check --tsconfig ./tsconfig.json` "to the three `ui` packages". **There is
  exactly ONE tsconfig in the whole `ui/` tree — `ui/common/tsconfig.json`.** `client`, `node`,
  `sampler` and `core` have none. The proposal is not executable as written.
- **That one tsconfig cannot resolve today.** It has no `paths`, and `common/lib/.../derive.ts`
  imports `$common/…` and `$core/…`. The aliases live **only** in the three `vite.config.js` files.
- **There is no `svelte.config.js` anywhere.** svelte-check's vite-config fallback failed from every
  angle attempted; it only ran once given one.

### §1.2 🔑 THE FINDING THAT RESHAPED THE MILESTONE — "the three packages" is the wrong unit

`lang="ts"` counted across all 69 `.svelte` files (excluding `templates/` and `backup/`, a dead
mockup archive that must never enter a check):

| package | typed / total `.svelte` |
|---|---|
| `ui/core`    | **46 / 46** |
| `ui/common`  | **11 / 11** |
| `ui/client`  | 1 / 8 |
| `ui/node`    | 0 / 1 |
| `ui/sampler` | 0 / 3 |

**The shells are plain JS. The libraries are 100% typed.** So the unit is not the three packages —
it is **the library (`ui/core` + `ui/common`), the one thing with no `package.json`, no
`vite.config.js` and no `node_modules` of its own.**

A svelte-check run over `ui/client` returned **0 errors**. That was NOT accepted on its face (N-139):
a positive control — a deliberate error injected into `about-dialog.svelte`, run, reverted — proved
the probe reads the files, and that the 7 untyped shell components have nothing to typecheck.
*The zero was honest and nearly meaningless.*

### §1.3 THE BASELINE — measured before designing, per the kickoff's binding

Probe outside the repo (`C:\cargo-targets\XGenProtocol\tc-probe`) plus temporary
`svelte.config.js` / `vite.config.js` / `tsconfig.json` at `ui/` and a `node_modules` junction.
Project strictness (`strict: true`, `verbatimModuleSyntax`, `isolatedModules` — lifted from
`ui/common/tsconfig.json`):

> **`ui/core` + `ui/common` → 13 errors, 34 warnings, 22 files.**

Of the 13: **10 confirmed-shape, 3 UNCONFIRMED** (see §1.5). `.ts`-only with tests excluded: **1**.

**Not a ratchet. This is a milestone that ends at zero.** The 10 have **THREE root causes:**

| # | root cause | sites | shape of fix |
|---|---|---|---|
| A | `component?: Component` in `common/lib/plugins/registry.ts` — bare `Component` means *takes no props*; every entry is a region widget with a required `regionId` | **6** | one type |
| B | narrowing lost across a closure / `let` — `core/lib/components/layout/mutate.ts:95`, `core/lib/components/layout/region-shell.svelte:254` | **2** | `const split = node` after the guard |
| C | attribute typings — `textfield.svelte:76` (`autocomplete` → `FullAutoFill`), `meter.svelte:75` (`name` not in `HTMLMeterAttributes`) | **2 → ⚠️ ACTUALLY 3** | two lines |

**⚠️ v1.1 — §8 #4 FIRED, EXACTLY AS IT ANTICIPATED.** Root cause C is **3 sites, not 2**: `password-field.svelte`
forwards `autocomplete` and had to move to `FullAutoFill` as well. The grouping was built by reading five error
sites and was slightly wrong; Clair flagged it rather than absorbing it. *The runbook naming its own weak spot is
what turned this into a flag instead of a silent drift.*

**`meter.name` — FOLDED IN, NOT ESCALATED, AND THE REASONING IS ON THE RECORD.** `name` was never valid HTML on
`<meter>` and is passed by **no call site** (verified independently: 8 `<Meter` usages, all in `app_sampler.svelte`,
none passing `name`). `{name}` therefore always evaluated to `undefined` and the attribute was never emitted.
**Removal is provably zero behaviour change**, so it is not the "a fix changed behaviour → escalate" case Leg B
meant. Correct call, correctly declared.

**Root cause A is not cosmetic.** `component?: Component` currently types the field as
*"a component taking no props"*. Every `CLIENT_PLUGINS` entry is mounted by the shell **with**
a `regionId`. **The plugin registry's own mount contract is untyped** — and `settingsComponent`
(M-RP-SETTINGS Leg B) is declared the same way. Fixing it types the contract; **it is not a cast.**

### §1.4 ⚠️ THE TOOLCHAIN TRAP — a pin, not a preference

`typescript` now resolves to **7.0.2** by default (the native port). **svelte-check 4.7.3 crashes on
it** — `TypeError: Cannot read properties of undefined (reading 'useCaseSensitiveFileNames')`.
Pinning `typescript@5` (5.9.3) fixed it.

**Unpinned, this gate breaks on the next fresh `npm install` — and it breaks LOUDLY, which is the
only reason it is merely annoying rather than dangerous.** Pin `^5`.

### §1.5 ⚠️ UNCONFIRMED — the separator family. DO NOT TREAT AS DEFECTS.

svelte-check reported `separator.svelte:57:16 — <script> was left open`, and consequently
`menu.svelte:35` and `status-bar.svelte:34` — *"Module '…separator.svelte' has no default export"*.

**Chat's Phase-0 report first characterised this as "one malformed file poisoning two importers in
shipped `core`". That was RETRACTED before this runbook was written.** Grounded since:

- `separator.svelte` **compiles clean** — `svelte.compile`, 0 warnings, under **both** the project's
  Svelte (5.55.5) and the probe's (5.56.6).
- No BOM (`3C 73 63` = `<sc`), trailing newline present, 2948 bytes.
- `sb-cell.svelte` carries the **identical** empty `<style></style>` and produced **no** error — so
  "empty style block" is not even a consistent trigger.

**Most likely a probe artifact: the junction served Svelte 5.55.5 while the probe dir held 5.56.6.**
*A number obtained by mixing two versions of the compiler is not a measurement of this codebase.*

→ **✅ RESOLVED AT LEG A — AND THE PREDICTION WAS WRONG ABOUT THE MECHANISM.** Not a version artifact. A **literal
`<style>` tag inside a `//` comment in `<script>`, in a file that also has a real style block**, makes svelte2tsx
read the script as never closed — stripping the default export and failing every importer. **Reproducible on the
pinned 5.55.5.** The compiler is unaffected, which is why it shipped and was CDP-verified while being invisible to
every floor the project had. **🔑 The gate justified itself on its first run, on already-shipped code.** Fixed by
rewording the comment plus a warning left at the point of contact. Latent set swept: **4 in scope**
(`color-picker`, `shelf-face`, `shelf`, `region-tile`) + **3 out of scope** (`ui/client/src`). → **N-144**

### §1.6 ⚠️ THREE SVELTE VERSIONS, MEASURED

`client 5.55.5` · `node 5.55.5` · `sampler 5.56.4`. **The sampler — where `npm test` runs and the
catalogue is verified — is on a different compiler than the two shipped apps.**

**Not this milestone's job to fix. It IS this milestone's job not to make it worse** → §2.5.

### §1.7 ⚠️ N-140 IS LIVE RIGHT NOW

Two vite dev servers alive at Phase-0 (`ui/client` PID 32496, `ui/node` PID 29300, both from 18:43).
**A "full reload" means the dev server process is gone, not just the Tauri window.** Leg A's first
measurement is invalid unless `Get-CimInstance Win32_Process -Filter "Name='node.exe'"` shows no
surviving vite. **Check for node/vite PIDs, not only `xgen-*`.**

N-117 also stands: the dev client HOLDS the exe; stop both apps before any `cargo` command, or a
held exe gives exit 101, **zero terminator lines, 0/0/0** — which reads exactly like a clean run.

---

## §2 — THE DECISIONS (Joe-locked 2026-07-19 under the J-547 standing grant)

### §2.1 UNIT = the library, not the packages
One check over `ui/core` + `ui/common`. The shells are excluded because they are plain JS and
checking them returns a positively-controlled zero (§1.2). `templates/` and `backup/` excluded —
dead mockup archive.

### §2.2 TOOL = `svelte-check`, not `tsc`
**57 of the 84 typed files are `.svelte`** (57 typed `.svelte` + 27 non-test `.ts`; 91 with tests --
counted at hand-over, not estimated); `tsc` cannot see a component. Both root-cause-B bugs sit one
in a `.ts` and one in a `.svelte` — **a `tsc`-only gate would have caught exactly half of one root
cause.**

### §2.3 🔒 HOME = a real `ui/package.json`. FORCED, not preferred.
**Grounded: 14 bare `svelte` / `svelte/attachments` imports live in `core` + `common`** (`Component`,
`Snippet`, `tick`, `untrack`, `createAttachmentKey`). TypeScript resolves those by walking up from
the importing file: `ui/core/node_modules` → `ui/node_modules` → … **Neither holds any packages.** The library
free-rides on three copies of `node_modules` via vite aliases, and **nothing can resolve `svelte`
from `ui/core` or `ui/common` at all.**

⚠️ **v1.1 CORRECTION (Clair, Rule 6):** v1.0 said these directories *"None exist"*. **`ui/core/node_modules` DOES
exist** — a `.vite` cache holding no packages. **Evidence wrong, conclusion right, decision unaffected.** Chat
asserted a non-existence without running a `Test-Path` — the J-556 class a third time on this arc, and in the
runbook of the milestone built to answer it.

The Phase-0 probe faked this with a directory junction. **A junction cannot ship.**

→ **`ui/package.json` is created** — devDeps + the `check` script. It is `private: true` and declares
**NO `workspaces` key** (a workspaces root would change how all three apps install).

⚠️ **NAMED RISK: `ui/node_modules` becomes a walk-up resolution target for the three apps.** Nearer
`node_modules` wins, so the apps keep their own — **but this is asserted, and V5 measures it**
(vite 193/170 unchanged is the direct evidence).

### §2.4 ALIASES = a `paths` block, and the duplication is ACCEPTED AND NAMED
Generating tsconfig from vite is machinery nobody asked for. But `$common`/`$core`/`$assets` will
then exist in **four** places (three `vite.config.js` + one `tsconfig.json`) — a real D-067 surface.

**It is documented rather than pretended away:** a comment on both sides, and **V4 EXERCISES the
drift** — break an alias in `vite.config.js`, watch the check still pass. *A duplication you have
proven can drift is honest; one you have assumed stays in sync is not.*

### §2.5 🔒 SVELTE PINNED EXACTLY TO THE CLIENT'S VERSION
`ui/package.json` declares `svelte` **`5.55.5` exact** — the version the two shipped apps run.

**Reason, and it is §1.5's lesson made structural: checking against a NEWER compiler than ships can
report errors that do not exist in the shipped build.** That is very likely what the separator family
is. A gate that reports defects the product does not have is worse than no gate, because someone will
"fix" them.

`typescript` pinned `^5` (§1.4). `svelte-check` `^4`.

### §2.6 WARNINGS OUT OF THE GATE — filed, not fixed
34 warnings, ~24 a11y (`role` / `tabindex` / click-handler-without-key-handler). Fixing those changes
**markup in shipped, CDP-verified `core` components** — a behaviour arc with an appearance edge, not
a lint pass. **Errors gate at 0; warnings are REPORTED AND COUNTED.**

→ files **`M-RP-A11Y` — a11y warnings on the `core` component library**.

### §2.7 🔒 THE FLOOR — named now, because every future runbook will quote it
> **`svelte-check` — errors/warnings over `ui/core` + `ui/common`. Gate: errors 0.**

**TWO numbers, both quoted, always.** A warning count that silently climbs is how the a11y backlog
gets forgotten. It joins cargo / npm / vite / catalogue / registry.

### §2.8 `ui/common/tsconfig.json` IS DELETED, not kept
It is unrunnable today (§1.1) and two tsconfigs over overlapping files is a second source of truth.
**One tsconfig, at `ui/`.** Its `exclude: ["lib/**/*.test.ts"]` is not inherited — see §2.9.

### §2.9 ⚠️ TEST FILES ARE **IN** THE CHECK — and their error count is UNMEASURED
`vitest` lives only in `ui/sampler/node_modules`, so TypeScript cannot resolve it from `core`/`common`
test files — **7 × TS2307 in the probe.** That is precisely why `ui/common/tsconfig.json` excluded
them. With `vitest` declared in `ui/package.json` it resolves, and the test files become checkable.

**They should be checked** — `mutate.test.ts` hand-builds `v2` trees, which is exactly where a wrong
type hides.

**⚠️ BINDING: the error count with tests INCLUDED AND vitest RESOLVABLE HAS NEVER BEEN MEASURED.
Leg A measures it. It is NOT assumed clean, and it is NOT counted inside the 13.** If it opens a
backlog, that is a Leg-A decomposition and a decision, not a silent scope grab.

→ **✅ MEASURED AT LEG A: 0 test-file errors — positively controlled, not assumed.**

---

## §3 — FILES

**Created**
- `ui/package.json` — private, no `workspaces`; devDeps `svelte@5.55.5` (exact), `typescript@^5`,
  `svelte-check@^4`, `vitest@^3`; script `check`.
- `ui/svelte.config.js` — `export default {};` (faithful: no app declares a preprocessor).
- `ui/tsconfig.json` — §2.1 include/exclude + §2.4 `paths`.

**Deleted**
- `ui/common/tsconfig.json` (§2.8).

**Edited (Leg B, ~5 edits over 5 files)**
- `ui/common/lib/plugins/registry.ts` (root cause A)
- `ui/core/lib/components/layout/mutate.ts`, `…/layout/region-shell.svelte` (root cause B)
- `ui/core/lib/components/data-independent/textfield.svelte`, `…/meter.svelte` (root cause C)

**Must NOT be touched**
`**/*.rs` · `ui/sampler/**` · `ui/node/**` · `ui/client/**` · `ui/assets/skin.css` ·
`ui/templates/**` · `ui/backup/**` · any `layout-default.ts`.

*(`ui/client/src/about-dialog.svelte` was mutated for the Phase-0 positive control and reverted;
`git status` was verified empty. It is not in this milestone's scope.)*

---

## §4 — LEGS

### Leg A — the gate, measured
Create the three files, delete `ui/common/tsconfig.json`, `npm install` in `ui/`.

**PREDICT FIRST, THEN RUN.** Predictions, from Phase-0 and stated before the run:
- errors **13**, warnings **34**, files **22** — *if the toolchain pin changes nothing*
- **minus 3** if the separator family was the version artifact §1.5 expects → **10**
- test-file errors: **UNMEASURED** (§2.9)

**DECOMPOSE any difference. Do not adjust the prediction to the result.** A floor predicted then
measured is worth more than one read off afterwards (N-108: say which numbers were SEEN and which
were DERIVED).

**⚠️ HAND BACK HERE IF the separator 3 survive** (§1.5) — that is Joe's, not Leg B's.

### Leg B — burn to zero
Three root causes, in order A → B → C. **Root cause A is a contract fix, not a cast** (§1.3).

**⚠️ ZERO RUNTIME BEHAVIOUR CHANGE IS THE GOAL, NOT A GUARANTEE.** If a fix changes behaviour, that
is a **FINDING** — the gate caught something live. **Stop and bring it to Joe.** Do not fold it in.

### Leg C — 🔑 MUTATE THE GATE (J-553 U4). NON-NEGOTIABLE.
Inject a deliberate type error in a **`.ts`** and, separately, in a **`.svelte`**; watch the gate go
red **and name the file**; revert; re-run green.

*A test that has never failed is not yet known to be able to. A green typecheck that cannot go red is
worse than none, because it looks like evidence.* This is the N-139 / N-142 family and **this
milestone is the worst possible place to repeat it.**

### Leg D — floors + records
ROADMAP · JOURNAL · CLAUDE.md PLAY · this doc → `Status: COMPLETED`. **N-138 marked GRADUATED** —
per J-513, a note that files an obligation and never graduates is a note the project has decided to
forget slowly.

### ⚠️ N-142 — TAKEN, IN ITS OWN COMMIT
`cdp-debug.ps1 -Mode console` subscribes to `Runtime.consoleAPICalled` **only**, so an uncaught
exception is invisible to it (265 lines, zero matches across a crash). The fix is a one-line
`Runtime.exceptionThrown` subscription. This milestone touches tooling anyway.

**Its own commit — it must never hide inside the gate's diff.** Verified by *causing* an uncaught
exception and seeing it appear.

---

## §5 — FLOORS, PREDICTED BEFORE DRIVING

| floor | predicted | why |
|---|---|---|
| `cargo test` | **1546 / 0 / 62 across 56 terminator lines — IDENTICAL** | zero `.rs`. **Identical is the DIRECT proof, not a corroboration** |
| `npm test` (sampler) | **132** | a devDependency and a script are not modules |
| `vite build` client | **193** | ← also the direct evidence for the §2.3 walk-up risk |
| `vite build` sampler | **170** | |
| sampler catalogue | **419** | `count === unique === domCount` |
| client registry | **134** quiescent | |
| **svelte-check** | **NEW FLOOR** | errors 0 at close; warnings recorded |

**Every registry/catalogue number states its axes** (N-105 quiescence · N-108 store · N-112 selection ·
N-115 saved-state count · fold state).

⚠️ `cargo test` **exceeds the MCP timeout** — run DETACHED and poll the PID in **separate short
calls**. A long `Start-Sleep` kills the shell and takes the detached run with it, leaving a
truncated log that reads plausible and complete and **wrong**, betrayed only by the missing final
`test result:` line. Grep **case-SENSITIVE** — `FAILED|panicked` case-insensitively matches
`0 failed` (N-117).

---

## §6 — VERIFICATION

| # | leg | evidence |
|---|---|---|
| V1 | **Baseline decomposed** | predicted vs measured, difference explained not absorbed; separator family resolved either way |
| V2 | **Zero errors** | `svelte-check` exit 0, errors 0; warning count recorded as a number |
| V3 | **🔑 The gate goes RED — both kinds** | injected `.ts` error → red + file named; injected `.svelte` error → red + file named; both reverted; re-run green |
| V4 | **Alias drift EXERCISED** (§2.4) | break a `vite.config.js` alias → check still passes → restore. *Proves the duplication is real* |
| V5 | **`ui/node_modules` did not perturb the apps** (§2.3) | vite 193 / 170 unchanged; sampler catalogue 419 |
| V6 | **Zero Rust** | `git diff --stat` has no `.rs` **AND** `cargo test` identical to §5 |
| V7 | **Scope** | `git show --stat` = the §3 file list; no `skin.css`, no shell, no sampler |
| V8 | **Behaviour-neutral** | client launches, registry **134** quiescent, all axes stated |

**⚠️ V3 and V4 are the two legs that make this milestone mean anything.** V2 alone is a green light
that has never been shown capable of turning red.

**N-105 applies to every probe: assert the subject is READABLE before asserting anything about it.**
A selector that cannot see its subject returns a clean-looking nothing — Phase-0 hit this twice
(`ReadAllBytes` on a relative path returned `len=0` for four files that are 1–3 KB).

**N-123 applies to every probe that persists a mutation: the cleanup is PART OF THE PROBE.** Phase-0
left `ui/svelte.config.js`, `ui/vite.config.js`, `ui/tsconfig.json`, a `ui/node_modules` junction and
`ui/client/svelte.config.js` on disk and removed all five; `git status` verified empty afterwards.

---

## §7 — DEFINITION OF DONE

⚠️ **v1.1 — SEAT ERROR IN v1.0's DoD, CAUGHT BY CLAIR (Rule 6).** Items marked **[CHAT]** below were written as
though the implementer owned them. **They do not.** This session's seat division puts every canonical record with
Chat, and the kickoff said so plainly — so the runbook and the kickoff, both authored by Chat, contradicted each
other. **Clair followed the kickoff and wrote no records, which was correct.** *This is the J-499/J-548/J-553/J-556
class — a document disagreeing with another document, findable only by reading both whole — and the fifth on this
arc from Chat.* The split is now explicit so the next milestone that copies this DoD copies it right.

**— IMPLEMENTER (Clair) —**

- [x] `ui/package.json`, `ui/svelte.config.js`, `ui/tsconfig.json` created; `ui/common/tsconfig.json` deleted
- [x] `npm run check` in `ui/` → **errors 0**; warnings recorded as a number → **0 errors / 34 warnings / 15 files, exit 0**
- [x] Baseline **decomposed**, not adjusted (§Leg A); separator family resolved — **neither artifact nor escalation, but a third answer: a real, reproducible svelte2tsx trigger** (→ N-144)
- [x] Test-file error count **measured and stated** (§2.9) → **0, positively controlled**
- [x] **V3 driven both kinds** — gate proven able to go red (`.ts` **and** `.svelte`, both files named), reverted, re-run green
- [x] **V4 driven** — alias drift exercised
- [x] Every §5 floor **predicted then measured** → cargo **1546/0/62 × 56**, npm **132**, vite **193/170**, all SEEN
- [x] `cargo test` **identical**, stated as the direct proof of zero Rust
- [x] Behaviour change surfaced rather than folded — **`meter.name` folded in WITH a proof of zero behaviour change, and declared** (§1.3)
- [x] No probe artifact left on disk; `git status` clean of unintended files
- [x] Deviations flagged, not absorbed — **7 Rule-6 flags, 2 of which corrected this runbook**

**— RECORDS (Chat) —**

- [x] **[CHAT]** N-142 in **its own commit**
- [x] **[CHAT]** N-138 marked **GRADUATED**; `M-RP-A11Y` filed → **N-146**
- [x] **[CHAT]** New notes for the findings → **N-144** (svelte2tsx) · **N-145** (`--ignore` rejected; noise stays)
- [x] **[CHAT]** v1.0's false `node_modules` claim corrected here **and** in the pushed J-557 records
- [x] **[CHAT]** ROADMAP · JOURNAL · CLAUDE.md PLAY · this doc updated together (D-074)
- [x] **[CHAT]** `Status: COMPLETED` on this doc

*(No "commit pushed" item — it is unflippable inside the commit that performs the push. `Status:
COMPLETED` is the signal. Joe pushes.)*

---

## §8 — FOR CLAIR (Rule 6)

**Flag, do not absorb.** Four runbooks on this arc were caught only because the implementer read them
whole first (J-499, J-548, J-553, J-556) — **three of those were Chat's.** This one has already had
one characterisation retracted before hand-over (§1.5).

**Known-weak spots in this document, named so you check them first:**
1. **§1.5** is a prediction, not a finding. If the 3 survive the pin, the runbook is wrong and Leg A
   stops.
2. **§2.3's walk-up risk** is reasoned, not measured. V5 is the measurement.
3. **§2.9's test-file count** is explicitly unmeasured. If it is large, say so — do not quietly
   exclude them to make Leg B look clean.
4. **§1.3's "three root causes"** came from reading five error sites. **If a fix at one site moves a
   count somewhere else, the grouping was wrong — say so.**

⚠️ **And the rule this milestone exists to embody: DO NOT ASSERT A COUNT, A FILE LIST, OR A
"THERE ARE N PLACES THAT DO X" WITHOUT RUNNING THE GREP.** At J-556 a runbook said the resolve logic
was written twice; it was written three times, and the third file's own comment said so. That class
is internally consistent and disagrees with the codebase — **no amount of re-reading finds it. Only
a grep does.**

**N-143: write the close LAST. If the work moves after it, RE-READ it rather than append.**

---

## §9 — OPEN FOR JOE

1. **The separator family** (§1.5) — iff it survives the pin. One of the three is inside shipped,
   CDP-verified `core`.
2. **The three-way Svelte version drift** (§1.6) — filed, not scoped here.
3. **Two vite dev servers were alive at Phase-0** (§1.7). They must be down before Leg A measures.

---

## §10 — AFTER

**Leg D2 — R6 composer + echo store (`$common`)** — design already locked and waiting: the C-4
amendment (§9.11.2), the twelve user-facing locks (§9.11.3), and §9.11.8's finding that outage send
latency is re-anchor + `SEND_QUEUE_TIMEOUT` ≈ 19 s, **not** the 16 s bound alone.
Then **Leg D3 — send-status widget**, second tenant of the `bodyExtras` container M-RP6.9 built.

**Filed by this milestone:** `M-RP-A11Y` — a11y warnings on the `core` component library.
