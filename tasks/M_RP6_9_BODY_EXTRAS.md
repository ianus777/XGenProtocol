# M-RP6.9 — `bodyExtras`: the per-row message container (`core`, fixture-driven)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-19  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Seat, grant, and the fence

**Seat.** Chat Claude authored this runbook (Phase-0 grounding + design walk, J-554).
Clair implements from it. Ms Design owns **all** visual appearance. Joe locks
fundamental architecture and pushes every commit.

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

### §1.4 The resolve logic is written twice and has zero tests

`message.svelte:68–77` and `message-stream.svelte:106–112` are byte-similar. The
`resolve.ts` / `resolve.test.ts` pair in `core` is **layout**, unrelated. `core`'s
tested pure modules today are `stream/grouping.ts` and `layout/{mutate,resolve}.ts`.
⇒ There is genuine pure logic to extract, and it is about to gain a key rule.
**`npm test` moves for an honest reason, not an invented one.**

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
`ui/core/lib/components/data-dependent/mounts.ts` + `mounts.test.ts`. Three call
sites — `message.details`, `message.bodyExtras`, `message-stream.background` —
one rule, tested.

### **D-4 · A tombstone drops `bodyExtras`, matching `details`.**
The container **cannot** distinguish an attachment (must vanish with the body)
from a reaction (arguably survives) — the fence forbids it knowing. So: **one
rule, conservative direction.** `bodyExtrasCount` is forced to `0` in the getter,
mirroring the `deleted → detailsCount:0` precedent (J-479).
**FILED, reversible at M-RP-REACTIONS**, where the tenant is known. Recorded
rather than silently settled.

### **D-5 · `types.ts:67–69` corrected in this milestone.** §1.5.

**Not locked here — sent to Ms Design, entirely:** what the strip *looks* like —
the below-body band, its spacing, whether a marker there reads well against the
reserved avatar column, and its appearance on a **grouped continuation row**
where there is no header line above it. §7 is the handoff.

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

### S-2 · Migrate the three existing call sites onto `resolveMounts()`

`message.svelte` (`details`), `message-stream.svelte` (`background`). **Behaviour
must be identical.** `details` gains `idPrefix = cid('d-')`; background gains
`idPrefix = cid('bg-')`.
**Floors after S-2: sampler catalogue 386 unchanged, client registry 134
unchanged, `npm test` up by the S-1 cases only.** If the catalogue moves here,
stop — something registered that should not have.

### S-3 · Render `bodyExtras` in `message.svelte`

- Resolve with `idPrefix = cid('x-')`. Namespaces `d-` / `x-` / `bg-` cannot
  collide with the literal suffixes `avatar` / `name` / `body`.
- Placement: **below the body, INSIDE `.msg-content`, OUTSIDE the
  `{#if !grouped}` block** — that position *is* the milestone. It must render on
  a grouped continuation row.
- **Tombstone guard (D-4):** not rendered when `deleted`.
- Root element: a single container span/div carrying a stable class for Ms Design
  to key on. **Chat proposes `.msg-body-extras`; the name is structural, the
  styling is hers.** No skin rule ships from this milestone beyond whatever is
  needed to make it not visually broken — and even that is provisional and
  labelled for `M-RP-SKIN`.
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
| sampler catalogue | 386 | **≈412 (+26)** | derived below |
| client registry | 134 quiescent | **134** | fixtures are sampler-only; the client feeds the socket nothing |

**The +26, derived per cell** (non-grouped text cell = `message` + avatar + name +
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
| **V-2** | **S-2 is inert.** Catalogue **386** and registry **134** after S-2, both after a full reload. *This is what makes D-1's "measurably inert on `details`" a measurement instead of a claim.* |
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

## §7 — Ms Design handoff

**Everything below is hers; nothing in it is decided here.**

- The `bodyExtras` strip sits **below the message body**, inside the content
  column, and **renders on grouped continuation rows where there is no header
  line above it**. How it reads in that position is the design question.
- **This socket has TWO future tenants:** the send-status indicator (Leg D3 —
  three states: sent · **unresolved** · not sent) and **reactions**
  (M-RP-REACTIONS — N per row, animated, well above ~16px, added and removed at
  runtime, custom sets). *Design for the second and the first is free.*
- The structural class this milestone emits is `.msg-body-extras`. The
  `.fixture-widget` styling in the sampler is a stub, **not a proposal.**
- **Still owed from earlier legs, unchanged:** M-RP6.6 ConnStats row-swap, and
  **all** Leg C + Leg D appearance.

---

## §8 — Close

*(Written by the implementer at close. Must state: measured floors vs the §4
predictions with any difference EXPLAINED · every §6 deviation with its reason ·
which numbers were SEEN and which DERIVED · the V-7 result, since it is the one
leg that proves a failure mode rather than a feature.)*

---

## §9 — Definition of Done

- [ ] S-1 … S-5 complete; `mounts.ts` pure and tested.
- [ ] V-1 … V-10 run; results recorded in §8 with SEEN/DERIVED marked.
- [ ] **V-4 green** — the socket survives grouping. Without this the milestone has
      not landed, whatever else passed.
- [ ] **V-7 green** — the silent-overwrite collision mode measured and reverted.
- [ ] `cargo test` **1546/0/62 across 56 terminator lines, unmoved.**
- [ ] Sampler catalogue moved off **386** (predicted **419**; a different number
      is acceptable **explained**, not adjusted).
- [ ] Client registry **134 quiescent** after a full reload.
- [ ] `types.ts` D-5 correction shipped; no `ReactionDescriptor`, no wire shape,
      no store, no asset, no URL anywhere in the diff.
- [ ] `git show --stat` confirms scope: **zero `.rs`, zero node, zero
      `layout-default.ts`.**
- [ ] §7 handed to Ms Design.
- [ ] D-074: JOURNAL + CLAUDE.md PLAY + ROADMAP + this file travel in ONE commit.
- [ ] `Status: COMPLETED` set in this header at close.
