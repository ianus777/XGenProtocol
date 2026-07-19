# M-RP6.9 — `bodyExtras`: the per-row message container (`core`, fixture-driven)
> **Status**: COMPLETED  
> Version: 1.4  
> Date: Jul 2026  
> **Last updated**: 2026-07-19  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Seat, grant, and the fence

**Seat.** Chat Claude authored this runbook (Phase-0 grounding + design walk, J-554).
Clair implements from it. Joe locks fundamental architecture, judges appearance
live in the sampler, and pushes every commit. **Ms Design is RETIRED (J-555);
appearance returned to Chat and is specified in §7 — there is no design seat to
consult.**

**Grant (J-547, restated J-552 and again at J-554):** *"mine: appearance +
architectural decisions; yours: all others."* Every lock in §2 was **taken, not
asked**. They are container **mechanics**, not architecture: no protocol, no wire
shape, no store, no federation, no identity. Joe may of course reverse any of
them — but nothing in this runbook waits on a reply.

**⚠️ THE FENCE — BINDING, AND IT IS THE MILESTONE.**
The container renders `WidgetMount[]` and **never learns what its contents mean**.
The sampler passes fixture mounts; a later arc passes real ones; **the component
does not change between those two events.** That is what makes it a container to
complete rather than a stub to replace.

- **Fixtures live in the SAMPLER only.** In the real client the container renders
  **nothing** until something feeds it. A client rendering invented tags is fake
  data on screen — N-091 / D-065.
- **Fixture assets are locally bundled, never remote URLs.** D-111 must not be
  pre-figured even by a fixture. *This milestone ships no assets at all: every
  fixture mount is a text stub. The constraint is recorded so the next author
  inherits it, not because it binds anything here.*
- **NO `ReactionDescriptor`, no wire shape, no protocol, no store, no
  federation, no attribution.** A data shape invented before the protocol exists
  is a shape the protocol then has to satisfy — and **who reacted is identity
  data, on the no-anonymity core: Joe's.**
- **M-RP-REACTIONS is FILED and its discussion is DELIBERATELY DEFERRED** (Joe,
  J-552). It is named here only to fix the *purpose* of this container. Do not
  reopen it.

---

## §1 — Phase-0 grounding (measured at source against HEAD, 2026-07-19)

Everything in this section was **read**, not recalled. File:line given so the
implementer can re-verify before writing a line — the discipline that caught
three wrong runbooks on this arc (J-499, J-548, J-553).

### §1.1 🔑 `cid()` under N mounts × M rows — the headline question, and the answer is **neither**

The kickoff framed it as *"sixty new registered ids or sixty collisions, and
which is not known."* **It is neither. As HEAD stands it is sixty ZEROS.**

- `message.svelte:79` — `const cid = (s: string) => (id ? \`${id}__${s}\` : undefined);`
  The suffixes passed to it are **hardcoded string literals**: `'avatar'`,
  `'name'`, `'body'`. **`cid()` is never applied to a widget mount.**
- `message.svelte:167` — mounts render as `<W {...d.props} />`. **No `id` reaches
  a mount.** A mounted widget is invisible to the registry unless it mints its own.
- `ui/sampler/src/fixture-widget.svelte` header comment says so on purpose: *"It
  deliberately does NOT register with `envelope`: … keeping the stub
  registration-free keeps the registry delta per message cell predictable."*
- `message-stream.svelte:125` — `msgId = (mid) => \`${id}__m-${mid}\``, and
  message ids are event/ULID. **The prefix chain is collision-free by
  construction**; a collision is only reachable by two mounts sharing a suffix
  *within one row*.

**⇒ No `core` defect. Nothing to bring Joe as a defect.** But the answer only
*stays* true once a real tenant arrives, because an interactive reaction **will**
want to register. That is what §2 D-1 secures and §5 V-6/V-7 measure.

### §1.2 ⚠️ What a collision would look like if we ever caused one — the finding worth keeping

`ui/common/lib/components/base/debug.ts`:

- `const registry = new Map<string, Entry>()`; `register()` is `registry.set(id, …)`.
  **A duplicate id silently overwrites. No warning, no throw.**
- `unregister()` is `registry.delete(id)` and runs on the action's `destroy`.
  **When the loser of a collision unmounts, it deletes the survivor's entry too.**
- ⇒ **A collision presents as the registry SHRINKING**, which is
  indistinguishable from a clean unmount and sits directly in N-132's blind spot.
- `envelope.ts:39` — with `name` but **no `id`**, the debug id falls back to
  `` `${typeClass}#${++ordinal}` ``, a **module-level counter**. Unstable across
  remounts ⇒ no baseline can be taken against it.

*This is why D-1 exists. It is not tidiness; it is the difference between a
measurable socket and one whose failure mode is invisible.*

### §1.3 W-13 under runtime membership change — one real defect, in the key

```
key: `${m.widgetId}-${i}`      // message.svelte:73  AND  message-stream.svelte:111
```

**Index-composed.** `details` has only ever been handed a static list, so it has
never bitten. Under runtime add/remove: delete mount 0 of `[A,B,C]` and `B`
re-keys `B-1` → `B-0`, so **Svelte destroys and recreates every mount after the
removal point.** Invisible for a `<span>` stub; for an animated, interactive
reaction it is state loss plus an animation restart on every *unrelated* removal.
**Exactly the "designed against the weak consumer" failure Joe predicted.**

**Late arrival.** `resolvedDetails` is `$derived` over `widgets[m.widgetId]`. It
re-derives when the `widgets` **reference** changes. A consumer mutating the
registry object **in place** gets nothing. D-119 already paid for this shape
(a `$state` `Set` does not react to `.add`/`.delete` — reassign a fresh one);
it is stated here so it is not rediscovered a third time.

### §1.4 ⚠️ The resolve logic is written THREE times and has zero tests

**⚠️ CORRECTED AT v1.2 (Clair, pre-build read).** v1.0/v1.1 said *"written
twice"* and named two sites. **That count was asserted from the two files Chat
happened to open — never from a grep.** *Chat named a count it had not grounded,
which is the rule this project applies everywhere else, pointed the other way.*

**Grep-verified, `widgets[|widgetId]|${m.widgetId}` across `ui/**` excluding
`node_modules`/`worktrees`/`backup`/`templates` — THREE identical copies:**

| # | site | registry var |
|---|---|---|
| 1 | `message.svelte:73–76` | `widgets` (`details`) |
| 2 | `message-stream.svelte:111` | `widgets` (`background`) |
| 3 | **`region-shell.svelte:83`** | **`bgWidgets`** (grid backdrop, M-RP-PLATE) |

All three are `` key: `${m.widgetId}-${i}` `` → `component: <reg>[m.widgetId]` →
`props: m.props ?? {}` → `.filter(x => !!x.component)`. **Character-identical
bar the registry variable** — and `region-shell.svelte:80` says so in its own
comment: *"the `message-stream` shape exactly"*. **The code announced it was a
copy and the runbook still missed it**, because §1.4 was written from a memory of
the codebase that predates M-RP-PLATE.

**🔑 NOT IN SCOPE, NAMED SO IT IS NOT RE-LITIGATED: `region-node.svelte:216/225/226`.**
It is a **fourth grep hit and a DIFFERENT rule** — a SINGLE mount resolved by
`node.widgetId` behind an inline `{#if widgets[…]}`, with **no list, no key, no
props spread, no filter**. *A grep returns four; three move. The non-match is
recorded so the next reader does not have to re-derive why.*

`core`'s `resolve.ts` / `resolve.test.ts` are **layout**, unrelated. `core`'s
tested pure modules today are `stream/grouping.ts` and `layout/{mutate,resolve}.ts`.
⇒ There is genuine pure logic to extract, in **three** places, about to gain a
key rule. **`npm test` moves for an honest reason, not an invented one.**

### §1.5 `types.ts:67–69` is wrong today

> ``details`` = the header region (time · temperature · badges · icon-buttons ·
> **send-status led**)

Locked at J-552 (§9.11.5): send-status is **per-row state**, not header chrome —
the same category as a reaction, not the same category as an author name. That
line was written before a composer existed, when nobody had asked the question.
**Corrected here** (D-5), not at Leg D3 — the socket's doc-comment travels with
the socket's build, and the line is wrong *now*.

---

## §2 — The five locks

### **D-1 · The container supplies a stable `id`; the widget may ignore it.**
Mounts render `<W id={…} {...d.props} />`. Passing `id` is **not** the container
learning what its contents mean — `id` is this codebase's universal envelope
contract, the same category as `props`. A widget that ignores it registers
nothing (today's `FixtureWidget` behaviour, **preserved byte-for-byte**); a widget
that uses it lands on the collision-free prefix chain of §1.1.
`{...d.props}` spreads **last**, so a consumer can still override.
**Applies to BOTH sockets** — `details` and `bodyExtras` — for symmetry.
⚠️ It is **measurably inert on `details` today**, because `FixtureWidget` ignores
`id`. That inertness is a **prediction to verify** (V-2), not an assumption.

### **D-2 · `WidgetMount.mountKey?: string` — optional, falls back to `` `${widgetId}-${i}` ``.**
One optional field fixes the Svelte keying churn (§1.3) **and** the registry id
stability (§1.2), for **both** sockets, **without** inventing a
`ReactionDescriptor` or any wire shape. `WidgetMount` already exists in
`types.ts`; this is container mechanics, not protocol. **The fence holds.**
The fallback is **byte-identical to today's key**, so every existing mount keeps
its current key and current behaviour.

### **D-3 · `resolveMounts()` extracted to a pure `core` module.**
`ui/core/lib/components/data-dependent/mounts.ts` + `mounts.test.ts`.
**FOUR call sites after this milestone** — `message.details` · `message.bodyExtras`
(new) · `message-stream.background` · **`region-shell.background`** — one rule,
tested. **⚠️ The third existing copy was found by Clair, not by the runbook
(§1.4). Migrating only the two the runbook originally named would ship a shared
function AND leave a private copy in the live grid renderer — the illusion of
consolidation, which is worse than none: the next person changes the key formula
in one place and region-shell silently keeps the old one.**

### **D-4 · A tombstone drops `bodyExtras`, matching `details`.**
The container **cannot** distinguish an attachment (must vanish with the body)
from a reaction (arguably survives) — the fence forbids it knowing. So: **one
rule, conservative direction.** `bodyExtrasCount` is forced to `0` in the getter,
mirroring the `deleted → detailsCount:0` precedent (J-479).
**FILED, reversible at M-RP-REACTIONS**, where the tenant is known. Recorded
rather than silently settled.

### **D-5 · `types.ts:67–69` corrected in this milestone.** §1.5.

**Appearance is NOT locked in §2 — it is specified in §7** (skin-only,
PROVISIONAL, discharged at `M-RP-SKIN`, judged live by Joe at 9422). What the
strip *looks* like —
the below-body band, its spacing, whether a marker there reads well against the
reserved avatar column, and its appearance on a **grouped continuation row**
where there is no header line above it. **§7 specifies all of it**, and Joe
judges it live at 9422 after Clair ships — a redirect costs one CSS block.

---

## §3 — Implementation steps

Each step is independently green. Do not batch S-1 into S-2.

### S-1 · `mounts.ts` — the pure resolver (D-2 + D-3)

New file `ui/core/lib/components/data-dependent/mounts.ts`:

- `resolveMounts(mounts, widgets, idPrefix?)` → `{ key, id, component, props }[]`.
- `key` = `m.mountKey ?? \`${m.widgetId}-${i}\`` — **the fallback is today's key,
  exactly.**
- `id` = `idPrefix ? \`${idPrefix}${key}\` : undefined`. The caller passes the
  namespaced prefix; the resolver never builds a namespace itself.
- Unknown `widgetId` ⇒ **dropped** (W-13), unchanged.
- Pure. No Svelte, no `$derived`, no DOM.

`mounts.test.ts` covers: drop-unknown · empty/absent list · `mountKey` honoured ·
fallback equals the legacy key · **duplicate `widgetId` gets distinct keys** ·
removal from the middle **leaves the survivors' keys unchanged when `mountKey` is
supplied** (this is the §1.3 defect, asserted) · `idPrefix` absent ⇒ `id`
undefined.

⚠️ **Mutate each new assertion and watch it fail before trusting it** (J-553 U4:
*a test that has never failed is not yet known to be able to*).

### S-2a · Migrate the two `core` message sites onto `resolveMounts()`

`message.svelte` (`details`) and `message-stream.svelte` (`background`).
**Behaviour must be identical.** `details` gains `idPrefix = cid('d-')`;
stream background gains `idPrefix = cid('bg-')`.
**Floors after S-2a: sampler catalogue 386 unchanged, client registry 134
unchanged**, `npm test` up by the S-1 cases only. If the catalogue moves here,
stop — something registered that should not have.

### S-2b · ⚠️ Migrate `region-shell.svelte` — ITS OWN STEP, AFTER S-2a IS GREEN

**This is the live grid renderer. A break blanks the centre of the client — that
exact failure happened at J-499.** Four-line swap, `bgWidgets` registry,
`idPrefix` **undefined** (the backdrop socket mints no ids today and this
milestone does not change that — `resolveMounts` returns `id: undefined` when no
prefix is passed, by construction, S-1).

**🔑 SEQUENCE IS THE POINT, NOT CAUTION FOR ITS OWN SAKE.** J-499's lesson is not
*"be careful with the grid"* — it is ***if the centre blanks, you must know which
swap did it.*** Two migrations landing in one step destroys that. **Do not batch
S-2b into S-2a.**

Guards, both sides of the swap, **both after a FULL RELOAD** (N-132), comparing
**identical numbers, not non-zero ones**: grid backdrop mounted · client registry
**134** · `backgroundMountCount` unchanged. See **V-2b** for the two that make
these mean something.

### S-3 · Render `bodyExtras` in `message.svelte`

- Resolve with `idPrefix = cid('x-')`. Namespaces `d-` / `x-` / `bg-` cannot
  collide with the literal suffixes `avatar` / `name` / `body`.
- Placement: **below the body, INSIDE `.msg-content`, OUTSIDE the
  `{#if !grouped}` block** — that position *is* the milestone. It must render on
  a grouped continuation row.
- **Tombstone guard (D-4):** not rendered when `deleted`.
- Root element: a single container span/div carrying the stable class
  `.msg-body-extras`. Its **appearance is specified in §7** (provisional,
  skin-only, discharged at `M-RP-SKIN`).
- Getter gains `bodyExtrasCount` = **resolved (rendered)** mounts, forced to `0`
  when `deleted`, and `0` on the `system` kind (the Option-A normalisation
  precedent already in the getter).

⚠️ `system` messages have **no** `bodyExtras` render path. The system sub-tree
stays exactly as it is — authorless centred notice, one paragraph.

### S-4 · Sampler fixtures

Two stubs, and the second one is the point:

1. `fixture-widget.svelte` — **unchanged, not touched.** Non-registering. It is
   the control.
2. **`fixture-reg-widget.svelte` — NEW, registering.** Calls `envelope` with the
   `id` the container hands it, and exposes a trivial getter (`{ label }`). This
   is the instrument that answers §1.1's question **by measurement** rather than
   by reading, and it is why the milestone is worth building rather than filing.

Fixture cells (this list determines the predicted catalogue number in §4 — change
the list and re-derive the number **before** driving):

| cell | shape |
|---|---|
| `message#text-body-extras` | not grouped · 3 `bodyExtras` mounts (2 non-reg + 1 reg) |
| `message#text-extras-grouped` | `grouped=true` · 3 mounts — **proves grouping-immunity, the whole reason the socket was chosen** |
| `message#text-extras-unknown` | not grouped · 2 known + 1 unknown id ⇒ drop-unknown in the new socket |
| `message#text-extras-deleted` | `deleted=true` · 2 mounts ⇒ D-4, `bodyExtrasCount:0` |
| `message-stream#stream-extras` | 4 text rows (2 grouped) × 3 mounts = **the N×M bench** |

⚠️ **Every mount is a text stub. Zero assets, zero URLs** (fence).

### S-5 · `types.ts` — D-2 field + D-5 correction

- `WidgetMount` gains `mountKey?: string` with a comment saying what it is for
  (stable identity across runtime add/remove) and what it is **not** (a protocol
  field, an attribution, or anything a reaction will hang meaning on).
- `MessageDescriptor.bodyExtras` loses "RESERVED-UNFED in v1 (D-065)".
- **`details` loses "send-status led"** and `bodyExtras` gains it, with the
  one-line reason (per-row state, not header chrome).

---

## §4 — Predicted floors — stated BEFORE driving

*A floor predicted then measured is worth more than one read off afterwards.*

| gate | now | predicted | why |
|---|---|---|---|
| `cargo test` | 1546/0/62 · 56 terminators | **1546/0/62** | **zero `.rs`. Any move at all is scope leak — stop.** |
| `npm test` | 114 | **114 + `mounts.test.ts` cases** | D-3 extraction |
| vite CLIENT | 192 | **193** | ⚠️ **+1 `core` module in the import graph.** The client renders nothing from this milestone but still *compiles* `mounts.ts`. **Predicted deliberately: an unpredicted client move on a sampler-only milestone reads exactly like scope leak.** |
| vite SAMPLER | 169 | **171** | +1 `mounts.ts`, +1 `fixture-reg-widget.svelte` |
| sampler catalogue | 386 | **419 (+33)** | derived below | 
| client registry | 134 quiescent | **134** | fixtures are sampler-only; the client feeds the socket nothing |

⚠️ **`ui/assets/skin.css` now enters the diff** (§7, seat change J-555). **It
moves NO module count** — CSS rules do not move a module graph, measured at J-550
when a CSS-rules-only commit left vite unchanged. The 193 / 171 predictions above
are unaffected and stand.

**The +33, derived per cell** (non-grouped text cell = `message` + avatar + name +
body = 4 · grouped = `message` + body = 2 · deleted = `message` + avatar + name = 3
· plus **one registering mount each**):

- `text-body-extras` 4 + 1 reg = **5**
- `text-extras-grouped` 2 + 1 reg = **3**
- `text-extras-unknown` 4 + 1 reg = **5**
- `text-extras-deleted` 3 + **0** (D-4 drops the socket ⇒ the reg stub does not
  mount ⇒ **it must not register**) = **3**
- `stream-extras` 1 root + (2 × 4) + (2 × 2) + 4 reg = **17**

⇒ 5 + 3 + 5 + 3 + 17 = **+33 ⇒ 419.**

⚠️ **This supersedes the "≈412" quoted in the J-554 design walk**, which was
derived before the registering stub was added to the fixture set. **419 is the
number to predict.** *Recorded rather than quietly corrected — a floor that
changes between the walk and the runbook must say so, or the next reader cannot
tell a revision from an error.*

⚠️ Both numbers are **arithmetic, not measurement** (N-108). If the drive lands
elsewhere, **explain the difference before adjusting the number.**

---

## §5 — Verification legs

**Baseline discipline:** N-132 — read the quiescent registry/catalogue **only
after a FULL RELOAD**, never on an accumulated dev session. N-099 — a DOM read
after a mutation needs a **second** eval with a settle delay. N-105 — a thrown
eval is **inconclusive**, not a failure; state the state. N-117 — **stop both
apps and check `Get-Process` for `xgen-*` before any cargo command**; `0/0/0`
reads exactly like a clean run.

| leg | what it proves |
|---|---|
| **V-1** | `npm test` green; the new cases counted; **each new assertion shown able to fail** (mutate → red → revert → green). |
| **V-2** | **S-2a is inert.** Catalogue **386** and registry **134** after S-2a, both after a full reload. *This is what makes D-1's "measurably inert on `details`" a measurement instead of a claim.* |
| **V-2b** | 🔑 **S-2b (region-shell) is inert — BOTH HALVES.** (i) **Render half:** backdrop mounted, registry **134**, `backgroundMountCount` **identical** before and after, each read after a full reload. ⚠️ **The probe must be POSITIVELY CONTROLLED (N-139)** — show it returning TRUE for a backdrop known present *before* the swap, or a FALSE afterwards means nothing. (ii) **⚠️ DROP half, and it is the one that matters:** feed the backdrop an **unknown `widgetId`**, confirm `backgroundMountCount` **falls** and nothing throws, then revert. *"Still mounted" proves the socket renders; it does not prove W-13 still works THROUGH the new function — and the filter is exactly where a subtle difference would hide.* |
| **V-3** | `bodyExtras` renders on a **non-grouped** row: DOM present, `bodyExtrasCount:3`. |
| **V-4** | 🔑 **It renders on a GROUPED row.** `data-grouped` set, header absent, `bodyExtrasCount:3`. **This is the milestone's reason for existing** — if this fails, `bodyExtras` was the wrong socket and Joe hears about it before anything else ships. |
| **V-5** | Drop-unknown in the new socket: 3 declared, `bodyExtrasCount:2`, GHOST absent from the DOM. ⚠️ **Positively controlled (N-139): the same probe must return the two KNOWN mounts, or the absence of the third means nothing.** |
| **V-6** | 🔑 **N×M registration measured.** `stream-extras`: 4 rows × 1 registering mount ⇒ **exactly 4 new ids**, all distinct, all matching `…__m-<msgid>__x-<key>`. **Sixty zeros became four ones, on purpose and by construction.** |
| **V-7** | 🔑 **The collision mode proven, not assumed.** Temporarily give two mounts on one row the **same** `mountKey`; **the registry count goes DOWN, not up, and no error is thrown.** Revert. *§1.2's silent-overwrite is now a measured fact, and D-2 is the thing that prevents it.* |
| **V-8** | D-4: `text-extras-deleted` ⇒ tombstone, `bodyExtrasCount:0`, **no `x-` id in the registry**. |
| **V-9** | Runtime membership change: remove the **first** of three mounts with `mountKey` supplied; the surviving two keep **their keys and their registry ids**. ⚠️ Element identity is claimed **within** a mount, never across one. |
| **V-10** | Full-reload quiescent floors: catalogue, client registry, both vite counts, `cargo test`. **State which were SEEN and which were DERIVED** (N-108). |

⚠️ **`count` is not the message count** — `stream-panel` prepends synthetic system
rows. Use `projectedCount` vs `streamCount`. Not directly in scope here (this is a
sampler bench, not the client stream), but it is the trap that catches everyone.

---

## §6 — Deviation rule

Any departure from §3 is allowed **if it is named in §8 with its reason**. A
deviation recorded is a decision; a deviation summarised away is a defect (J-550:
the summary line outranked the deviation line, on the one binding set in
capitals). **The implementer reads this runbook whole before writing a line** and
reports any cross-section contradiction — three on this arc were caught only that
way, and two of those were Chat's.

---

## §7 — Provisional appearance (SPEC, in live vocabulary)

**Seat change, J-555.** This section was a handoff to Ms Design. Her lane came
back with three artifacts that **are not on disk** (`git status` clean; zero grep
hits) and, more importantly, written in the **retired May-2026 mockup
vocabulary**: `--xgen-*` appears **121 times** in `ui/templates/skeleton/tokens.css`
and **ZERO times** in the live `ui/assets/skin.css`. Her mental DOM is
`ol[aria-label="Messages"] > li > article` with `<dl>` details — the
pre-component-library prototype. **⚠️ It would have failed SILENTLY:**
`var(--xgen-msg-extras-lane)` with no declaration resolves to nothing, no error,
no warning — the lane collapses and *"pre-sized so churn wraps instead of jumping
the row"*, the best idea in the handback, quietly does not hold. Appearance
returns to Chat under the **M-RP-SHELF-FRAME (J-530) pattern**: skin-only,
PROVISIONAL, shipped with the mechanics, judged live by Joe in the sampler,
discharged at **M-RP-SKIN**.

**Her REASONING is kept — it is DOM-independent and it is good.** Her NUMBERS and
TOKEN NAMES are discarded: they were measured against a component that is not
there.

### §7.1 ⚠️ THE MEASURED FACT THAT BREAKS HER OPTION A AS SPECIFIED

Option A (proximity tuck) rested on *"`4px` to the body above, `16px`
inter-message gap does the separating"* — a 1:4 ratio. **Measured at
`skin.css:2836`:**

```
.message              { padding: var(--sp-1) var(--sp-2); }   /* 4px 8px */
.message[data-grouped] { padding-top: 0; }
```

⇒ separation between two NON-grouped rows = 4px bottom + 4px top = **8px**, not 16.
⇒ separation into a GROUPED CONTINUATION = 4px bottom + **0** = **4px**.

**So the real ratio on a continuation row is 4px tuck vs 4px to the next row —
1:1. No belonging signal at all, on precisely the row type this milestone exists
for.** *Her conclusion was right; the number it stood on was from another
codebase.*

**FIX, keeping "no new chrome": make the proximity real rather than add a mark.**
A row that OWNS a strip pushes the next row away:

```
.message:has(.msg-body-extras) { padding-bottom: var(--sp-3); }   /* 12px */
```

⇒ 4px tuck vs 12px to the next row = **1:3**, restored, and it holds on grouped
continuations because it is keyed on the strip's presence, not on `[data-grouped]`.
`:has()` is fine in this Chromium/Tauri build. **Option C (hairline spine) stays
the named fallback** if 1:3 still reads ambiguous in a live grouped run.

### §7.2 What M-RP6.9 SHIPS — the container's own rules, nothing else

```
:root { --msg-extras-lane: 28px; }        /* PROVISIONAL — M-RP-SKIN */

.message .msg-body-extras {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--sp-1);
  min-height: var(--msg-extras-lane);
  margin-top: var(--sp-1);                /* the 4px tuck — §7.1 */
  justify-content: flex-start;
}
.message[data-own] .msg-body-extras { justify-content: flex-end; }
.message:has(.msg-body-extras)      { padding-bottom: var(--sp-3); }
```

**Why each choice, so a later reader can overturn it knowingly:**

- **`--msg-extras-lane` lives at `:root`, NOT on `.message`.** The
  `--region-fold-rotate` comment in this same file warns exactly why: *a local
  default on the component out-specifies an inherited value and silently shadows
  any override* — a trap that already cost one re-verify. A theme must be able to
  reach this. **⚠️ Do not "helpfully" add a `.message` local default.**
  (`--msg-deleted` sits on `.message` correctly, because it is COPY, never themed.)
- **`28px`, not her 26px** — it matches the existing `--ctl-h` / avatar-track
  height, so a lane of reaction tiles lines up with the 28px rhythm the rest of
  the skin already keeps. **Still provisional; it is one line.**
- **`min-height`, not `height`.** Her guarantee (churn wraps horizontally, no row
  jump) holds for one lane; **`flex-wrap` means a long reaction run grows to a
  second lane and the row DOES move.** That is honest and unavoidable — the
  alternative is clipping user content. **Named, not hidden:** the guarantee is
  *"adding the 4th tag does not move the row"*, not *"nothing ever moves."*
- **`justify-content: flex-end` on own rows, NOT `row-reverse`.** `.msg-header`
  reverses because it is a two-part unit (name + details). Reversing here would
  reverse the **order of a user's reactions**, which is not a mirror, it is a
  reordering.
- **Motion: reuse `var(--motion)` (120ms) if an entry transition ships at all**,
  rather than her 180ms — the project has already decided its motion feel and one
  fewer magic number is worth more than 60ms. **Deviation from her spec, named.**

### §7.3 ⚠️ WHAT THIS MILESTONE DOES **NOT** SHIP — and why that is the fence

Her send-status design is **recorded here as intent for Leg D3 and MUST NOT be
built now**: one glyph slot on `data-state`, three readings distinct on **shape +
colour + motion** — filled disc (`sent`, persist-faint, `--t3`) · hollow ring
that breathes (`unresolved`, `--warn`) · attention triangle (`not sent`,
`--err-bright`); retry via `data-retry` — `failed` free · `rejected` none ·
`timed_out` warn-then-retry.

**⚠️ A `.msg-body-extras [data-state="…"]` rule in THIS milestone would put a
tenant's state vocabulary inside the container's skin — the fence breached
through CSS rather than through TypeScript.** Ms Design guarded against this
herself (*"keyed only on the container and `[data-own]`/`[data-deleted]`, never
on widget identity"*) and she was right. **Those rules ship with the WIDGET at
Leg D3, in the widget's own block.** Recorded here only so D3 does not
re-derive it.

Likewise **no reaction-tag visuals** — M-RP-REACTIONS, Joe's, deferred.

### §7.4 What Joe judges, and when

The five sampler cells (§3 S-4) do not exist until Clair builds them, so this
appearance cannot be judged before implementation. **Sequence: Clair ships
mechanics + this skin block in one pass → Joe looks at 9422 → redirect freely.**
Because it is skin-only and provisional, a redirect costs **one CSS block, not a
rebuild** — which is what makes approve-after-delivery cheap here, exactly as it
was for the other thirty-odd components.

**Watch for, specifically:** whether 1:3 reads as belonging on a real grouped run
(§7.1) · whether 28px is dead space when the only tenant is one small glyph ·
whether the strip on a mirrored `[data-own]` row reads as the author's or as
interface chrome.

**Still owed, now unowned:** M-RP6.6 ConnStats row-swap · all Leg C + Leg D
appearance · M-RP-SKIN · M-RP-FOCUS.

---

## §8 — Close

**Written by Clair at close (2026-07-19).** Every number below was measured in
this session unless explicitly marked DERIVED.

### §8.1 Floors — predicted vs measured

| gate | predicted | measured | |
|---|---|---|---|
| `cargo test` | 1546/0/62 · 56 terminators | **1546 / 0 / 62 · 56 terminators · exit 0** | ✅ unmoved |
| `npm test` | 114 + the new cases | **132** (114 + 18) | ✅ |
| vite CLIENT | 193 | **193** | ✅ exact |
| vite SAMPLER | 171 | **170** | ⚠️ **−1, explained below — not adjusted** |
| sampler catalogue | 419 | **419** (`count===unique===domCount`) | ✅ exact |
| client registry | 134 quiescent | **134** (`count===unique===domCount`) | ✅ |

**⚠️ The sampler figure MOVED DURING THE SESSION — 172 → 170 — and this paragraph
was rewritten rather than patched.** It first read 172, which was correct until
Joe's late request (§8.3 ⑨) relocated both stubs' appearance into `skin.css` and
deleted their `<style>` blocks. *A close written before the last change is a close
that is wrong about the shipped state.*

**Decomposed by measurement, not by argument.** The governing rule was proven with
a probe: adding one `<style>` block to `fixture-widget.svelte` moved the build
**170 → 171**; removing it returned **170**. ⇒ **a Svelte `<style>` block is its
own module in Vite's graph.**

| step | modules |
|---|---|
| baseline | 169 *(`fixture-widget`'s style module already inside this number)* |
| `+ mounts.ts` | 170 |
| `+ fixture-reg-widget.svelte` | 171 |
| `+` its own style module *(as originally built)* | **172** ⇐ the figure this close first recorded |
| `−` that style block (§8.3 ⑨ relocation) | 171 |
| `−` `fixture-widget`'s style block *(was inside the 169 baseline)* | **170** ✓ |

So §4's **171** was wrong in two directions at once: it under-counted by one (it
never counted the new component's style block, hence 172), and it predates a
relocation that removed two (hence 170). **Two errors of opposite sign, partly
cancelling. The prediction could not have been right, and neither number was
adjusted to fit.**

**Scope**, from `git diff --stat` + `git status --porcelain`: 7 modified, 3 new.
**Zero `.rs`, zero node, zero `layout-default.ts`, zero `templates/`.** `cargo test`
landing byte-identical corroborates it; the diffstat is the direct evidence.

**SEEN vs DERIVED (N-108).** Everything in the table above was SEEN. So were: the
V-2b before/after pairs, every V-3…V-9 getter and id list, the enumerated 17-id
`stream-extras` subtree, and the empty-store fact (no `xgen-client_uistate.json`
beside the running exe). **Nothing load-bearing is DERIVED** — notably §4's +33
per-cell breakdown was predicted arithmetic, but the `stream-extras` cell's own 17
was then *enumerated*, so that row is measured rather than inferred.

### §8.2 🔑 V-7 — the leg that proves a failure mode, and it CORRECTS §1.2

**§1.2 predicted a collision would present as the registry SHRINKING, silently.
It does not.** Driven, on the painted app:

- **Control first** (two registering mounts, DISTINCT `mountKey`): catalogue
  **420**, two distinct ids. Alive.
- **Collision** (the same two mounts, IDENTICAL `mountKey`): the app is **DEAD** —
  `__XGEN_DEBUG__` undefined, `#sampler-root` **0 children**, blank page. Measured
  twice, at separate times.
- **Revert**: catalogue back to **419 exactly**.

Svelte's keyed `{#each}` rejects duplicate keys and **the render dies at mount**,
before two mounts can ever register the same id. The A/B isolates the cause by
construction: the *only* thing that changed between alive-at-420 and dead was the
key duplication.

**⇒ Duplicate `mountKey` is a LOUD failure, not a silent one.** `debug.ts`'s
silent-overwrite property (`Map.set`, and the loser's `unregister` deleting the
survivor) is still true as a property of the registry — but it is **not reachable
through `mountKey`**, because Svelte kills the render first. §1.2's conclusion
that the failure sits in N-132's blind spot does not hold for this route.
*This is the better outcome, and it is only known because the leg was run.*

**✅ CAPTURED ON RE-DRIVE — this section's earlier "not captured" is RETRACTED.**
The first pass recorded the crash but not the error *name*, and said so. On
re-drive it was obtained verbatim:

```
Uncaught Svelte error: each_key_duplicate
Keyed each block has duplicate key `k-reg` at indexes 2 and 3
  in message.svelte in app_sampler.svelte
```

⚠️ **The method is the transferable part, because the obvious approach fails
silently.** The harness's `-Mode console` subscribes to `Runtime.consoleAPICalled`
only, so an **uncaught exception is invisible to it** — a tail across the crash
returned **265 lines and zero matches**, which would have justified writing *"no
error observed"*: true, and completely misleading. What worked: **arm a
`window.onerror` + `console.error` collector FIRST, then trigger the failure by
HMR, never by reload** — a reload wipes the collector, and a tail attaches too
late. *(The underlying harness gap stands: `-Mode console` cannot see
`Runtime.exceptionThrown`. The workaround above does not close it, it routes
around it.)*

**And the correction this bought.** With the name in hand the cause is exact:
Svelte's **keyed-`{#each}` duplicate guard fires upstream of the registry**, so
§1.2's silent overwrite is **NARROWED, not disproven** — it remains true for
mounts in *different* each-blocks (different rows, or `details` vs `bodyExtras`)
that mint the same final id, because the guard is **per block**. What V-7 proves
is narrower and more useful: **a duplicate within ONE row's socket — the case
`mountKey` most plausibly introduces — fails loudly and names the key and the
indexes.** *A good result for D-2: the field this milestone adds is guarded
against its own most likely misuse.*

### §8.3 Deviations (§6) and findings

**① The third resolver copy — the finding was mine, the runbook then followed.**
Reading v1.1 whole before writing a line surfaced that §1.4's "written twice" was
ungrounded: `region-shell.svelte:83` is a third character-identical copy
(M-RP-PLATE). Chat corrected the runbook to **v1.2**, which now *instructs* S-2b.
**So migrating region-shell is not a deviation from the document I built against.**
Recorded this way round on Chat's own instruction, because a year from now the
distinction between "the implementer went off-script" and "the implementer found
the script wrong" is the whole value of the entry.

**② `WidgetMount.mountKey` landed at S-1, not S-5.** S-1 is titled *"the pure
resolver (D-2 + D-3)"* and its suite exercises `mountKey` in seven cases — the
field has to exist for those assertions to be honest rather than merely runtime-
lucky (there is no frontend typecheck, N-138, so a missing field would not have
failed anything). S-5 kept the D-5 doc correction, which is its real content.

**③ `region-shell`'s render line is byte-identical — no `id={b.id}` — and that is
grounded, not lazy.** `grid-plate` **self-defaults `id = 'grid-plate'`** and
registers as a stable, enumerable `grid-plate#grid-plate`. Passing an undefined id
is a no-op today, but it would make the socket one prop away from silently
**renaming a registered widget** the day anyone adds a prefix. S-2b's "no
`idPrefix`" is therefore load-bearing, and the reason is now in the code comment
so the next reader does not have to rediscover it. The two message sockets *do*
pass `id`, per D-1.

**④ V-9 was driven as a four-config A/B across full reloads, not a live in-place
removal.** A1 keyed ×3 → A2 keyed with the first removed: survivor **kept**
`__x-k-reg2`. B1 unkeyed ×3 → B2 unkeyed, same removal: survivor's id **moved**
`fixture.reg-1` → `fixture.reg-0`, old id gone. **The negative control moved, so
A2's stability is a measurement and not a tautology** — §1.3's defect demonstrated
live, D-2 shown to fix it. ⚠️ **What this does NOT prove:** that Svelte preserved
component *instances* across a live removal. No mutable `bodyExtras` fixture
exists, and editing the fixture module HMR-replaces the whole subtree, so instance
identity cannot be probed across the change. Element identity is claimed **within**
a mount, never across one. The pure suite carries the key-contract half.

**⑤ ⚠️ SUPERSEDED BY ⑨ — kept, not deleted, because the reasoning still explains
why ⑨ was worth doing.** This entry read: *"`fixture-reg-widget.svelte` carries its
own copied style rules"* — Svelte scopes styles per component, so the sibling's
`.fixture-widget` could not reach it, and the rules were a deliberate hand-synced
copy. **True when written; false as shipped.** Both `<style>` blocks are gone and
one shared, named block in `skin.css` now serves both (⑨). *The duplication this
entry describes is precisely what Joe's request removed.*

**⑥ Swept a stale claim in `message.svelte`'s header (N-109).** It still read
*"Deferred to later steps: … `bodyExtras` (reserved-unfed, D-065)"* — false the
moment S-3 landed. Rewritten in the same commit as the code that falsified it.

**⑦ My own N-110 repeat, recorded because it nearly entered the record.** The first
V-6 read filtered registry ids with `indexOf("stream-extras__") === 0`. Ids are
`type#id`, so the filter matched nothing and returned `regCount: 0` with
`allDistinct: true` and `allMatchPattern: true` — **both vacuously true on an empty
array, the flattering answer.** Discarded, re-driven with a `READABLE` guard that
returns early when the subject is absent. *The N-099 family: a check that cannot
see its subject still returns an answer.*

**⑧ Emoji in the sampler fixtures (Joe, mid-session — and he corrected my first
pass).** I put them in message *bodies* first; Joe: *"i meant mainly in the
reaction container. by this we will have proper situation and we will see if we
need to edit reactions' appearance settings."* **He was right — a lane judged
against `clip.png` / `+2` text pills says nothing about how it holds a real
reaction.** Strip labels are now reaction-shaped (`👍 3` · `🎉 1` · `🚀 2` on the
message cells, `👍 4` · `❤️ 2` · `👀` on the stream bench); eight bodies carry one
each, spread across render states, **including a skin-tone modifier (`👍🏽`, a
two-codepoint grapheme)** which survives the round trip intact. Verified
**count-neutral** (419 before and after — text content only). **Kept deliberately**
— Joe: *"if we working system, we replace them. but i need them for tuning now."*
`system` notices left emoji-free on purpose: they are protocol-generated text, and
an emoji there would imply the protocol emits one. ⚠️ **This restates the V-5
control labels** — §8.4's `clip.png` / `reg` are now `👍 3` / `🚀 2`; the leg was
re-driven and is unchanged in substance.

**⑨ ⚠️ Both stubs' appearance moved into `skin.css` as named tokens (Joe:
*"can we put both to skin.css as some named variable, nobody will think that they
are some random mistakes"*). This touches `fixture-widget.svelte`, which S-4 called
"unchanged, not touched — it is the control."** Named as a deviation for that
reason. **Why it does not weaken the control:** the twins were identical only by
**hand-synced copies** (⑤); they are now identical **BY CONSTRUCTION**, and the
variable under test is still **REGISTRATION**, carried by the single intentional
`[data-reg]` divergence. Five tokens
(`--fixture-chip-{fs,lh,pad-x,pad-y,radius}`) in a labelled `SAMPLER FIXTURE STUBS`
block whose comment states plainly that it is **not shipped UI** and that
**`M-RP-REACTIONS` deletes it**. **Proven inert by re-measurement:** chips
**29.9 × 14 @ 10px**, header pills **31.7 × 14 @ 10px** — byte-identical across the
move — catalogue **419**, dotted outline on the reg stub only. **Proven LIVE, not
merely declared:** pushing `--fixture-chip-pad-y` to `6px` moved the painted chip
**14 → 26px** and removing it restored **14**, all inside **one synchronous eval**
so no override could outlive the call (N-123). *It also removed a real hazard: two
files holding the same numbers, hand-synced forever, where a drift would have made
one stub read as "the real one" and quietly destroyed the control.*

**⑩ A DEV warn on duplicate `mountKey` was considered and NOT built.** Svelte
already fails loudly and names the key and the indexes (§8.2); a second guard on a
path that **cannot pass silently** is the unfed-branch mistake this project keeps
refusing. **Filed with its trigger:** Svelte's message names `message.svelte` but
not *which row or descriptor* — add the warn the first time that ambiguity costs
someone real time, with a real case behind it.

### §8.4 The legs, in one line each

- **V-1** ✅ 132 green, and **all 18 new assertions shown able to fail** via **8
  guarded source mutations** (filter removed · `mountKey` ignored · id-guard
  removed · props default removed · index dropped from the fallback · filter moved
  before map · absent/empty not defaulted · props discarded). Each mutation had an
  apply-guard; one early attempt did **not** apply and was recorded as
  **inconclusive, not a pass**. Source byte-identical to backup afterwards.
- **V-2** ✅ S-2a inert: catalogue **386**, registry **134**, both after full reloads.
- **V-2b** ✅ **both halves.** Render: `backgroundMountCount` **1 → 1**, plate
  present, `grid-plate#grid-plate` still registered, registry **134 → 134**,
  `leafCount` 8, `droppedCount` 0 — with the probe **positively controlled**
  (shown TRUE before the swap). Drop, *through the new function*: known+unknown →
  **1**, the known mount still rendered and the ghost absent; unknown-only →
  **falls to 0**, plate deregistered, registry 133, **`leafCount` still 8 — the
  grid never blanked**; restored → **exactly 134**. Zero page errors throughout.
- **V-3** ✅ non-grouped: strip in DOM, 3 children, `bodyExtrasCount: 3`.
- **V-4** 🔑 ✅ **grouped: `data-grouped="true"`, header ABSENT, avatar ABSENT, strip
  still renders 3 children, `bodyExtrasCount: 3`.** The socket is grouping-immune
  by position, proven on the painted DOM. **This is the milestone landing.**
- **V-5** ✅ 3 declared → `bodyExtrasCount` **2**, both knowns rendered (`👍 3`,
  `🚀 2` — **the positive control, N-139**: the same probe returning the knowns is
  what makes the ghost's absence mean anything), GHOST absent from the cell.
  *(Labels re-read after ⑧; substance unchanged.)*
- **V-6** 🔑 ✅ **exactly 4** registering ids,
  `fixture-reg#stream-extras__m-sx-{1,2,3,4}__x-fixture.reg-2`, all distinct; the
  16-id subtree + root = **17**, matching §4's derivation. sx-2 and sx-4 carry reg
  mounts while having no avatar/name — **V-4 reconfirmed at the registry level.**
  *Sixty zeros became four ones, on purpose and by construction.*
- **V-7** 🔑 ✅ §8.2 — and it corrects §1.2.
- **V-8** ✅ tombstone: `bodyExtrasCount: 0`, strip absent, body absent, and **zero
  `text-extras-deleted__x-` ids** — meaningful only because the registering stub
  would otherwise have produced one.
- **V-9** ✅ §8.3 ④, with its negative control.
- **V-10** ✅ §8.1.

**Fence check:** the real client renders **0** `.msg-body-extras` — nothing feeds
it, exactly as §0 requires. No `ReactionDescriptor`, no wire shape, no store, no
asset, no URL anywhere in the diff. No `[data-state]` / `[data-retry]` rule in the
skin. `--msg-extras-lane` is declared at `:root` **only**.

### §8.5 ✅ §7.4 DISCHARGED — appearance judged and approved

Joe judged it live at 9422 and approved: *"the appearance as is now, is very good
now. maybe i will do some cosmetic tuning of it in skin.css, but it can be leave as
such."* The block stays **PROVISIONAL**; discharger **`M-RP-SKIN`**.

⚠️ **Recorded in the skin block itself so it is not "fixed" later:** the chip
measures **14px inside a 28px lane** — half-empty **BY CONSTRUCTION**
(`--fixture-chip-pad-y: 0`), **seen and approved**, not an oversight. The same
comment carries the tuning order: **raise the chip's padding before settling the
lane**, or `--msg-extras-lane` gets fitted to a stub that is about to be replaced
by a taller tap target.

### §8.6 ⚠️ A harness finding that bit JOE, not just me

An `each_key_duplicate` overlay from an **intermediate save during S-2a** kept
resurfacing in Joe's sampler **without him touching anything** — twice. Grounded:
only **one** `const cid` exists (line 111) while the overlay cited line **129**, and
`vite build` compiled the tree clean. **The overlay was a ghost.** Cause: a **vite
dev server (PID 30520) outlived repeated kill-and-relaunch cycles of
`xgen-sampler`** and kept re-serving the failed module from its cache. Killing the
dev server cleared it permanently.

**⇒ For N-132 purposes, a "full reload" must mean the DEV SERVER is gone too, not
just the Tauri window.** Every baseline on this arc has been read after an app
restart; nothing said the server underneath could survive one carrying stale
modules. **It also retracts an "unexplained" I recorded earlier this session** — a
CDP endpoint dying while the process lived, which a hard parse error explains, and
which I had labelled unexplained before the parse error surfaced. *Corrected rather
than left standing.*

### §8.7 For Joe (§7.4)

The strip is live in the sampler under **DD Composites →
`message · bodyExtras socket`**, and on the stream bench under **`message-stream ·
bodyExtras N×M bench`**. Screenshot taken at close. The dotted-outline tile is the
registering stub — an instrument, not a design element. Watch the three things §7.4
names: whether 1:3 reads as belonging on the grouped row, whether 28px is dead
space under one small glyph, and how the strip reads on a mirrored `[data-own]` row.

---

## §9 — Definition of Done

- [x] S-1 … S-5 complete (**including S-2b, `region-shell`**); `mounts.ts` pure
      and tested; **`resolveMounts` has FOUR call sites and the tree contains ZERO
      remaining private copies** — proven by grep, not by memory (§1.4).
      *(Grep for the legacy key expression across `ui/**` returns NONE outside
      `mounts.ts`; the four call sites enumerated.)*
- [x] **V-2b green, BOTH halves** — render and drop — with the probe positively
      controlled. **The grid did not blank, and that was measured rather than
      observed in passing** (`leafCount` 8 held even with the backdrop dropped to 0).
- [x] V-1 … V-10 run; results recorded in §8 with SEEN/DERIVED marked.
- [x] **V-4 green** — the socket survives grouping: header and avatar absent,
      strip present with 3 children, `bodyExtrasCount: 3`.
- [x] **V-7 green** — collision mode measured and reverted. ⚠️ **It CORRECTED
      §1.2**: not a silent registry shrink but a hard mount failure (§8.2).
- [x] `cargo test` **1546/0/62 across 56 terminator lines, exit 0, unmoved.**
- [x] Sampler catalogue moved off **386** → **419**, exactly as predicted.
- [x] Client registry **134 quiescent** after a full reload (empty store verified
      on disk: no `xgen-client_uistate.json` beside the running exe).
- [x] `types.ts` D-5 correction shipped; no `ReactionDescriptor`, no wire shape,
      no store, no asset, no URL anywhere in the diff.
- [x] §7 skin block shipped in `ui/assets/skin.css`, **PROVISIONAL**
      with **no `[data-state]` / `[data-retry]` rule** (§7.3 — those are Leg D3's).
- [x] `--msg-extras-lane` declared at **`:root` only**, with no `.message` local
      default (§7.2 shadowing trap).
- [x] `git diff --stat` confirms scope: **zero `.rs`, zero node, zero
      `layout-default.ts`** (7 modified + 3 new).
- [ ] D-074: JOURNAL + CLAUDE.md PLAY + ROADMAP + this file travel in ONE commit.
      *(Chat's to author; not Clair's to write.)*
- [x] `Status: COMPLETED` set in this header at close.
