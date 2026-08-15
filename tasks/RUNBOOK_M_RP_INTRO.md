# M-RP-INTRO Runbook — the DM welcome intro: one additive key, two seams, and a fallback that must be tested rather than assumed
> **Status**: PENDING  
> Version: 1.3  
> Date: Aug 2026  
> **Last updated**: 2026-08-15  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS IS, AND WHAT IT IS NOT

**Implements legs 1–3 of `tasks/M_RP_INTRO_PHASE0.md` v1.3.** Leg 4 (verification) is Chat's and is
specified here so Clair can see what her work will be measured against. **Leg 5 (records + close) is NOT in
this runbook** — the Phase-0 §6 row 5 owns the close and this file **cites that row rather than restating
it**, because the same work written twice in two documents neither citing the other is the species that bit
`M-RP-MEMBER-ACT` four times in one arc.

🔒 **THREE JOE-LOCKS GOVERN EVERYTHING BELOW AND NONE IS RE-OPENABLE BY THIS RUNBOOK** (`D-123` Rule 6 —
Clair reports deviations, never silently redesigns a lock):

| lock | ruling | site |
|---|---|---|
| **§3 — (d)** | the intro is a **`message.text`** event whose `content` carries the human-readable **`text`** AND an additive namespaced key. A new `message.intro` event type was **REFUSED** | Phase-0 §3, 2026-08-15 |
| **§3.1 — (d1)** | the key lives in **`content`**, NOT `meta_atts`. The convention borrows `meta_atts`' **grammar** (`xgen.` reserved, reverse-domain third-party, lowercase snake_case) and **NOT its value rule** — nested objects are native in `content` | Phase-0 §3.1, 2026-08-15 |
| **the key** | **`xgen.intro.v1`** — versioned deliberately for a successor Joe expects | Phase-0 §3.1, 2026-08-15 |

🛑 **AND ONE DoD ITEM THAT IS A CONSTRAINT ON EVERY LINE OF CODE BELOW (Phase-0 §10 item 1-bis):**
***`text` MUST REMAIN LOAD-BEARING.*** The human-readable sentence lives in `content.text` and **is never
moved into the key**. It is the only thing that makes the intro degrade **rich → plain** instead of
**rich → nothing** on a client that does not know `xgen.intro.v1`. **Any step below that would leave `text`
empty or decorative is a DEVIATION and gets reported, not absorbed.**

📌 **State this runbook was authored against:** `HEAD` `c3aa044` **= `origin/main` by `git ls-remote`**,
clean tree. Every `G-` reference resolves to `tasks/M_RP_INTRO_PHASE0.md` §2.

### 🛑 v1.1 — WHAT CLAIR'S COLD READ FOUND, AND WHY v1.0 WAS NOT IMPLEMENTABLE

**`tasks/CLAIR_M_RP_INTRO_RUNBOOK_READ.md` returned GO-WITH-FINDINGS: seven findings, five pointer defects.
Chat re-drove every one against source (Rule 5). ALL CONFIRMED.** *Chat's own re-read of v1.0 before
shipping it caught none of them — the fifth consecutive time in this arc that every real defect came from
outside the text.*

| # | finding | v1.1 response |
|---|---|---|
| **F-1** | 🛑 **THE INTRO PAYLOAD HAS NO AUTHOR.** §4.4 said the send path *"passes the intro alongside the text"* and **nothing anywhere produces `headline`/`blurb`** — `dm-draft.svelte.ts:45,47` holds `_counterpart` + `_texts` only; `createDraftDm(counterpart, text)` takes two params; `echo.send(spaceId, roomId, text, at)` has no intro. **Zero occurrences of `headline`/`blurb` as data in all of `ui/`** (the 5 grep hits are unrelated prose). And **all three closures available to Clair are forbidden by this runbook**: derive from `text` (§2.1), ship unfed (§7.6), invent the mechanism (§2.3) | 🛑 **NEW LEG 2a — §3.0.** *The Phase-0 said the render half needs no new mechanism and "the whole question is the payload"; v1.0 then specified the wire seam and the render seam and NOT the payload's producer. "Passes" presumes something authored it* |
| **F-2** | 🔑 **NO VITEST GATE, while Leg 3 edits a unit-tested pure function.** `derive.test.ts` and `mounts.test.ts` cover exactly `projectEvent` and `resolveMounts` | 🔒 **V-8 ADDED, AND CHAT DROVE IT: `npx vitest run` → 9 files / 154 tests / 154 passed, 830 ms**, measured at `f9557ef`, clean tree. 🛑 **A 154-test floor existed for this whole arc and appeared in NO floor statement.** ⚠️ `ui/package.json` has **vitest ^3 and NO test script** — only `check`; the harness is invoked directly |
| **F-3** | the `extras`-overwrites-`text` branch has **no gate** — §3.1 defers the mechanism to Clair, and V-5's two arms both miss it | 🔒 **V-9 ADDED** |
| **F-4/F-5** | **V-0 contradicts §5's header and §8.2** ("Chat drives all of §5" vs "belongs in the implementer's window"), and its **subject set is undefined** — a pre-edit `cargo`/svelte-check cannot "fail", V-2 is impossible before the param exists. V-1 has no stated baseline and *unchanged may be the correct result* | 🔒 **§5.0 REWRITTEN** — V-0's subject set named, ownership resolved, V-1 given a baseline and an expected direction |
| **F-6** | 🛑 **THE SENDER NEVER SEES THEIR OWN INTRO** — own rows come from `echoToDescriptor`, not `projectEvent`, and nothing said so | 🛑 **FIXED, NOT FILED — §4.5.** `I2` argued *"symmetry is free"*; it is free only because own rows are `MessageDescriptor`s too. **Joe's call, scaffolded as fixed** |
| **F-7** | §4.3 never says whether the widget takes `id`/registers, and no registry gate exists either way | ✅ **§4.3 amended + V-10** |
| **census** | **the gate list was a CENSUS, not a partition** — four failure modes unlisted, and V-5's second arm named no surface | 🔒 **§5 rebuilt** |

**Pointer defects — all five confirmed and corrected:**

1. 🛑 **`stream-panel.svelte:135` → `:145`.** ⚠️ *The Phase-0's `G-9a`/`B-7` said `:143`. **Chat produced THREE distinct wrong numbers for one line across two documents.** The line was never re-opened; it was re-typed from the previous document each time.*
2. **`docs/ch3_*` matched nothing → `docs/xgen_ch3_specification.md`, §3.1.3 at `:332`** (and §3.0.3 wire-format invariance at `:108`).
3. **`exchange.rs:983-989` is the SIGNATURE, not the doc comment** it was cited for — the comment is `:975-982`.
4. 🛑 **B-2 was FALSE. Production call sites are EIGHTEEN, not three** — `xgen-client/src/app.rs` carries **15 above its `#[cfg(test)]` at `:6350`** (plus the import at `:40`), all shipped CLI subcommands. ⚠️ **The delegating-overload plan survives untouched** (the existing signature never changes) — **but the blast-radius sentence used to justify it was wrong by 6×.**
5. **Test-count metrics do not interchange:** Clair counted `projectEvent(`/`resolveMounts(` CALL SITES (17/28 by her instrument, 15/21 by Chat's); the **TEST counts are 21 and 18**. 🛑 **Neither number may be quoted as the other. The floor is 154/154 across 9 files** — the only figure Chat drove.

📌 **AND ONE INSTRUMENT NOTE, RECORDED AS CLAIR'S:** her `grep -c $'\r'` reported CR=302 on an LF file — **the pattern expanded to empty and matched every line, so the count equalled the LINE count**, and a genuine CRLF file shows equal counts too, which is exactly why it read as correct. She caught it on the byte total not moving after `tr -d '\r'`, re-measured with .NET at an absolute path (CR=0 / LF=302), **and did not claim a fix she had not made** (`D-065`). ⇒ **owed as an `N` note; `N-197` is the next free number.**

---

## §1 — 🔑 THE GROUNDING THIS RUNBOOK ADDS, BEYOND THE PHASE-0

The Phase-0 grounded the *decision*. These were measured for the *implementation* and are new here.

| # | fact | site | why it changes the plan |
|---|---|---|---|
| **B-1** | 🛑 **`build_message_text_event` HAS ~60 CALL SITES** across `exchange.rs`, `runtime.rs`, `app.rs`, `fanout.rs`, `ops.rs`, `derive.rs`, `connection.rs`, `ai_service.rs`, `resident.rs` | grep, `xgen-core` + `xgen-client` + `xgen-node` | ⇒ **WIDENING ITS SIGNATURE CHURNS ~60 SITES, almost all tests.** §3.1 takes the delegating-overload route instead: **zero call-site churn** |
| **B-2** | 🛑 **CORRECTED v1.1 — EIGHTEEN call sites are production, not three.** `resident.rs:451` (client send path) · `ai_service.rs:489` (AI reply) · `ops.rs:1996` (CLI send) · **and FIFTEEN in `xgen-client/src/app.rs`, all above its `#[cfg(test)]` at `:6350`** (shipped CLI subcommands) | `app.rs:1762`–`:5970`, boundary `app.rs:6350` | ⚠️ **The plan is unaffected — §3.1's delegating overload leaves the existing signature untouched, so none of the eighteen is edited.** 🛑 **But the justification was wrong by 6×, and it mattered: had §3.1 chosen to CHANGE behaviour in the shared function, the claimed blast radius would have understated it by fifteen shipped paths** |
| **B-3** | 🔑 **THE TWIN PATTERN EXISTS AND IS DOCUMENTED IN-FILE** — `build_message_file_event` and `build_message_redact_event` are each *"Twin of…"* with a doc comment naming the reuse | `exchange.rs:944-950`, `:983-989` | ⚠️ **BUT BOTH TWINS CARRY A DIFFERENT `EventType`. Ours reuses `EventType::MessageText` and widens CONTENT.** ⇒ **this is a NEW sub-pattern, not the existing one**, and §3.1's doc comment must say so rather than claim a precedent it does not have |
| **B-4** | 🔑 **`OutboundRequest` IS `{ space_id, room_id, text, reply }` and has exactly THREE construction sites** — `desktop.rs:355` (production), `resident.rs:711` (the definition), `resident.rs:1565` (a test) | `resident.rs:711-728` | ⇒ **adding a field is a three-site change.** Its doc comment states the design reason: *"the caller hands over intent, not an event"* (`D-067`) — **an intro payload IS intent, so it belongs here** |
| **B-5** | 🔑 **`projectEvent` IS A PURE FUNCTION IN ITS OWN UNIT-TESTED MODULE**, and it is the SINGLE site where an inbound `message.text` becomes a `MessageDescriptor` | `ui/common/lib/components/widgets/stream/derive.ts:69-97`; tests `derive.test.ts` | ⇒ **the whole read half is ONE pure function + ONE registry line.** \|\| 🔑 **AND IT IS M1′-DRIVABLE** — a Vite eval can `await import('/@fs/E:/…/stream/derive.ts')` and execute it for real, with **no disk write and no consent**. §5 uses this |
| **B-6** | ✅ **`projectEvent` READS `e.content?.text` DEFENSIVELY ALREADY** — `typeof e.content?.text === 'string' ? … : ''` | `derive.ts:81` | ⇒ **the `text` fallback is already correct and needs no change.** The intro key is purely additive here |
| **B-7** | 🔑 **THE `bodyExtras` REGISTRY IS ONE OBJECT LITERAL WITH ONE TENANT** — `const widgets = { 'send-status': SendStatus }` — and **W-13 DROPS an id it cannot resolve** | 🛑 **`stream-panel.svelte:145`** (v1.0 said `:135`, the Phase-0 said `:143` — **both wrong, re-typed rather than re-opened**), `mounts.ts:51` | ⇒ **registering the intro widget is one line**, and **the drop-unknown path IS the degradation path**. §5 tests it rather than trusting it |
| **B-8** | ✅ **`WidgetMount` is `{ widgetId, props?, mountKey? }`**, and `props` is `Record<string, unknown>` — 🛑 **NOTHING TYPE-CHECKS THAT A MOUNT SUPPLIES WHAT ITS WIDGET NEEDS** | `types.ts:53-71`; the warning is `send-status.svelte:37`'s own | ⇒ **the intro widget must tolerate a malformed/absent prop bag at runtime**, because the type system will not catch it |
| **B-9** | ⚠️ **`send_message` REJECTS AN EMPTY `text` BEFORE QUEUEING** — `if text.trim().is_empty() { return SendOutcome::failed("empty message") }` | `desktop.rs:313` | 🔑 **THIS GUARD IS WHAT MAKES 1-bis ENFORCEABLE AT THE SEAM**: an intro with no sentence cannot be sent at all. **Do not weaken it to let a text-less intro through** — that is exactly the failure 1-bis forbids |
| **B-10** | 🔑 **NEW v1.1 — THERE IS A PASSING VITEST FLOOR AND THIS ARC NEVER STATED IT: 9 files / 154 tests / 154 passed / 830 ms**, driven by Chat at `f9557ef` on a clean tree. ⚠️ **`ui/package.json` declares `vitest ^3` but has NO test script** — only `check` — so the harness is invoked directly (`npx vitest run`) | `ui/package.json`; 9 `*.test.ts` under `ui/` | ⇒ **V-8.** 🛑 *A floor that exists, passes, and is never stated is indistinguishable from a floor that does not exist — and Leg 3 edits two of the nine files* |
| **B-11** | 🔑 **NEW v1.1 — OWN ROWS DO NOT PASS THROUGH `projectEvent` AT ALL.** `outbound = echo.forRoom(…).map(echoToDescriptor)`, and `echoToDescriptor` hardcodes `bodyExtras: [{ widgetId: 'send-status', … }]` | `stream-panel.svelte:118-133` | ⇒ **§4.1 alone would leave the SENDER blind to their own intro**, contradicting `I2`'s *"symmetry is free"*. **§4.5 is the fix** |
| **B-12** | 🔑 **NEW v1.1 — `EchoMessage` IS `{ localId, spaceId, roomId, text, sentAt, status, eventId?, code?, cause? }`** | `echo-state.svelte.ts:73-85` | ⇒ **the echo record is where an own-row intro must ride** (§4.5), and its `absent-not-empty-string` discipline (`:80-82`) is the shape to copy |

---

## §2 — LEG 1 (JOE'S): THE INTRO'S WORDING, APPEARANCE, AND PAYLOAD SHAPE

🔒 **`D-138` GOVERNS THIS SECTION AND IS STATED IN FULL BECAUSE IT HAS BEEN INVERTED TWICE:** **mechanism is
Chat's, VALUES are Joe's, and the scaffold SHIPS WITH PLAUSIBLE VALUES AND NEVER BLANKS**, because
*"something that does not render cannot be looked at."* ⇒ **Clair implements against the scaffold below.
Nothing waits on Joe.** When Joe replaces a value, it is a **value edit, not a redesign**.

### §2.1 — 🔓 The payload shape — SCAFFOLDED, and Joe's to overturn

```
content: {
  "text": "<the human-readable sentence — ALWAYS PRESENT, 1-bis>",
  "xgen.intro.v1": {
    "headline": "<short line, optional>",
    "blurb":    "<a sentence or two, optional>"
  }
}
```

⚠️ **CHAT'S REASONING, OFFERED SO JOE CAN DISAGREE WITH SOMETHING CONCRETE:** the scaffold is **two optional
string fields and nothing else**. Every field added here is a field that must be **rendered, escaped,
length-bounded and versioned forever**, and `xgen.intro.v2` exists precisely so fields do **not** have to be
guessed right now. 🛑 **NO AVATAR, NO URL, NO IMAGE REF, NO LINK** in v1 — a URL in a first-contact payload
from a stranger is a fetch the recipient did not ask for, and that is `M-INTRO-POLICY`'s problem, not this
milestone's.

🛑 **`text` IS NOT DERIVED FROM THE KEY AND THE KEY IS NOT DERIVED FROM `text`.** They are composed
independently at the composer. *If `text` were generated from `headline`+`blurb`, then removing the key
would remove the sentence — which is 1-bis violated by construction rather than by accident.*

### §2.2 — 🔓 Wording — SCAFFOLDED, Joe's

**The sentence that goes in `text`** must read as a person talking, because it **is** the opening message and
renders as one on every client that has never heard of this key. Scaffold: the composer pre-fills the draft
with the user's own editable sentence; **the user can send a plain message with no intro at all**, and the
intro is **opt-in, never automatic.** 🛑 **An intro the user did not choose to send is stranger-authored
content the user's own client authored for them** — a different object from the one J-701 ruled on.

### §2.3 — 🔓 Appearance — Joe's, `M-RP-SKIN` absorbs the values

**Mechanism (Chat's):** the intro renders as a `bodyExtras` mount on the message row — **inside message
chrome** (`I1`), below the body, outside the header guard. **Values (Joe's):** spacing, border, tone. **The
widget ships with plausible values in its own `<style>` and never blank** (`D-138`); `skin.css` values are
Joe's file and are **not** in Clair's commit.

---

## §3 — LEG 2: THE WIRE SEAM (RUST) + LEG 2a: THE AUTHORING PATH. CLAIR IMPLEMENTS.

### §3.0 — 🛑 LEG 2a: THE PAYLOAD'S PRODUCER — **NEW IN v1.1, AND IT IS THE MILESTONE'S CONTENT**

🛑 **v1.0 SPECIFIED HOW AN INTRO TRAVELS AND HOW IT RENDERS, AND NOT HOW ONE COMES TO EXIST.** F-1.
**Nothing in `ui/` produces `headline` or `blurb`.** This leg is that producer, and it lands **BEFORE** §3.2,
because §3.2's `intro` parameter has nothing to carry until it exists.

**Files:** `ui/common/lib/stores/dm-draft.svelte.ts` · `ui/common/lib/components/widgets/composer-panel.svelte`
· `ui/common/lib/stores/echo-state.svelte.ts`

1. **`dmDraft` gains intro state beside `_texts`.** `_texts` is `Record<string, string>` keyed by counterpart
   (`:47`); the intro is **keyed the same way** so switching counterpart preserves each draft independently —
   *the existing `_texts` behaviour is the specification here, not a new decision.*
2. **The composer gains an opt-in intro affordance.** 🛑 **OPT-IN, NEVER AUTOMATIC** (§2.2): the default DM
   send is **byte-identical to today's** — no key at all, **not an empty one** (`N-182`, §7.6).
3. **`createDraftDm(counterpart, text)` (`composer-panel.svelte:150`) widens** to carry the intro, and
   **`echo.send` (`echo-state.svelte.ts:162`) widens** to pass it to `invoke('send_message', …)`.
4. 🛑 **`text` IS AUTHORED INDEPENDENTLY AND STAYS REQUIRED.** Composing an intro **does not** populate,
   replace or derive `text`. **A user who writes an intro and no sentence cannot send** — `desktop.rs:313`
   enforces it at the seam (`B-9`) and the composer should say so rather than let the seam refuse silently.

🔓 **VALUES ARE JOE'S, MECHANISM IS CHAT'S (`D-138`) — AND THE SCAFFOLD SHIPS PLAUSIBLE AND NEVER BLANK.**
The affordance's *shape* (a disclosure below the composer, two fields, collapsed by default) is scaffolded;
its **wording, placement and appearance are Joe's**. 🛑 **Do NOT file this to `M-RP-SKIN` and wait** — that
inversion has happened twice and the scaffold-not-blank rule is the correction.

⚠️ **THIS LEG IS A SCOPE INCREASE ON v1.0 AND IT IS CHAT'S, NOT A DISCOVERY OF CLAIR'S.** *Recorded here
rather than absorbed silently: the E-4 lesson is that an absorption presented as a discovery is the routing
that should not have happened.*

### §3.1-pre — LEG 2: THE WIRE SEAM (RUST)

🛑 **THIS IS THE LEG THE MILESTONE WAS NOT FILED WITH.** `M-RP-INTRO` sits in the RP (UI) track; **(d) made
this leg real Rust in three crates.** `cargo` is a **real floor from this leg onward** — it was refused entry
as a measurement during Phase-0 because that pass touched zero `.rs`, and that refusal does not carry
forward.

### §3.1 — `xgen-core`: a delegating overload, NOT a widened signature

**File:** `xgen-core/src/message/exchange.rs`

1. **Add** `build_message_text_event_with_extras(key, space_id, room_id, prev_events, text: &str, extras: Option<&Value>)`.
   Body is today's `build_message_text_event` with the content line replaced: start from
   `json!({ "text": text })`, and **when `extras` is `Some`, insert its keys at the TOP LEVEL of `content`**
   (not nested under a wrapper). ⚠️ **`extras` is a map of namespaced keys → values**, so `xgen.intro.v1` is
   supplied by the caller rather than named in core — *core learns the mechanism, not the tenant.*
2. **Rewrite** `build_message_text_event` as a **one-line delegation** with `None`. 🔑 **`B-1` is why:
   ~60 call sites keep their exact current signature and are not touched.**
3. **Doc comment must be honest (`B-3`):** it is **NOT** the `build_message_file_event` twin pattern — those
   twins carry a **different `EventType`**; this one **reuses `EventType::MessageText` and widens content**.
   🛑 **Do not write *"Twin of …"* and inherit a precedent this does not have.** 📌 *The twin doc comments are
   at `exchange.rs:944-950` and `:975-982`; v1.0 cited `:983-989`, which is the redact **signature**, not its
   comment.*

🛑 **REJECT AN `extras` THAT WOULD OVERWRITE `text`.** If a caller passes a key literally named `text`, that
is 1-bis violated at the lowest level. **Clair's choice of mechanism is hers** (debug-assert, silent skip
with a `tracing::warn!`, or a `Result`) — **but it must not silently win over `text`**, and whichever she
picks she states in the deviation report so Chat can gate on it.

### §3.2 — `xgen-client`: intent, not an event

**Files:** `xgen-client/src/resident.rs`, `xgen-client/src/desktop.rs`

1. `OutboundRequest` (`resident.rs:711`) **gains `pub intro: Option<serde_json::Value>`**. `B-4`: three
   construction sites. Its own doc comment already says the caller hands over **intent** — *an intro payload
   is intent.*
2. The drain (`resident.rs:451`) calls **`build_message_text_event_with_extras`**, passing
   `req.intro.as_ref().map(|v| …)` keyed as `xgen.intro.v1`. 🔑 **THE KEY STRING IS NAMED IN EXACTLY ONE
   PLACE IN RUST** — a `pub const` beside the call, not a literal repeated at two sites (`D-122`: a second
   spelling is how drift starts).
3. `send_message` (`desktop.rs:305`) **gains an optional `intro` param** and projects it into
   `OutboundRequest` at the existing projection point (`desktop.rs:353-358`, `D-137` — one projection per
   direction, and the comment there already says so).
   ⚠️ **VERIFY, DO NOT ASSUME: that a Tauri command tolerates an added optional param without breaking
   existing webview callers that omit it.** Chat drives this as gate **V-2**; Clair does not need to prove
   it, but **must not design around an assumption about it either way.**
4. 🛑 **`desktop.rs`'s empty-text guard STAYS EXACTLY AS IT IS** (`B-9`). **An intro with no sentence is
   not sendable, and that is the point.** 📌 *Cited by symbol: `if text.trim().is_empty()` — it was `:313`
   pre-edit and is **`:325`** now, because Leg 2 inserted the `intro` parameter above it.*

### §3.3 — The ch3 convention (documentation, and it is a deliverable)

**File:** 🛑 **`docs/xgen_ch3_specification.md`** — v1.0 said `docs/ch3_*`, which **matches nothing**.
**§3.1.3 Field Naming Conventions is at `:332`; §3.0.3 Wire-format invariance at `:108`.** The content-key
namespace convention, borrowed grammar per the §3.1 lock.
🛑 **IN A PROTOCOL PROJECT THE SPEC IS THE DELIVERABLE, NOT A CHORE.** State: `xgen.` reserved for protocol ·
reverse-domain for third parties · lowercase snake_case segments · **values may be nested objects (this is
where the convention DIVERGES from `meta_atts`' spec 3.1.3, and the divergence must be written down, not
implied)** · **unknown keys are ignored by readers and preserved byte-identically in transit** (`G-4b`,
`G-4g`).
🔑 **RECORD `G-4g` IN THE SPEC:** an unknown content key is **inside the signed canonical bytes**
(`canonical_value` recurses with no allowlist), so **stripping it breaks the signature.** *A future
implementer who does not know this may "helpfully" filter unknown keys at a relay and silently break every
signature that passes through.*

---

## §4 — LEG 3: THE RENDER SEAM (TS / SVELTE). CLAIR IMPLEMENTS.

### §4.1 — Read the key: `derive.ts`

**File:** `ui/common/lib/components/widgets/stream/derive.ts` (`projectEvent`, `:69-97`)

In the `message.text` branch, **after** the existing fields, read `e.content?.['xgen.intro.v1']`. When it is
a non-null object, emit
`bodyExtras: [{ widgetId: 'message-intro', mountKey: 'message-intro', props: { intro } }]`.

🛑 **`body` KEEPS ITS EXISTING LINE UNCHANGED** (`B-6`) — `typeof e.content?.text === 'string' ? … : ''`.
**The intro is purely additive here. If the diff touches the `body` line, that is 1-bis at risk and it gets
reported.**
⚠️ **VALIDATE THE SHAPE AT THE BOUNDARY** — this is wire data authored by a stranger (`B-8`: nothing
type-checks `props`). A non-object, a string, an array, `null`, or unexpected members must produce **no
mount** rather than a broken one. **The row still renders `text`.**
🔑 **THE KEY STRING IS NAMED IN EXACTLY ONE PLACE IN TS**, exported as a const — the mirror of §3.2 item 2.

### §4.2 — Register the widget: `stream-panel.svelte`

**File:** `ui/common/lib/components/widgets/stream-panel.svelte` — the `const widgets = {…}` object literal
🛑 **CITED BY SYMBOL, NOT BY LINE.** *v1.0 said `:135`, the Phase-0 said `:143`, the truth was `:145`, and
after Leg 3 it is `:169`. **Four wrong pointings for one line** — and v1.1 corrected the grounding row and
the correction list while leaving THIS row, the one an implementer actually follows. **A correction applied
to the cited instance rather than to the class is not a correction.***
`const widgets = { 'send-status': SendStatus }` **gains `'message-intro': MessageIntro`.** One line (`B-7`).
🔑 **The `above` socket and `dm-intro` are NOT touched** — that is the **pre-send** draft page and this
milestone is the **post-send** artefact (`G-8a`). **Two different things wearing one word; do not merge
them.**

### §4.3 — The widget: `message-intro.svelte`

**New file:** `ui/common/lib/components/widgets/message-intro.svelte`
Renders `headline` / `blurb` from `props.intro`.
⚠️ **STATE THE REGISTRY CONTRACT EXPLICITLY (F-7):** whether this widget takes an `id`/`regionId` prop, and
whether anything registers it beyond §4.2's one line. **`B-8` is why it matters — `props` is
`Record<string, unknown>` and NOTHING type-checks that a mount supplies what its widget needs**, so a widget
that silently depends on a field no mount passes fails at runtime, on a stranger's first message. **V-10
gates it either way.** 🛑 **TEXT NODES ONLY — NO `{@html}`, NO SANITISER, NO
MARKUP PATH** (Phase-0 §7.3). This is a **component, not a processed string**, for exactly the reason
`dm-intro` is: the payload is authored by a person the recipient has never met. **Markup belongs to
`M-RP-PROCESSOR-RENDER`**, which is a separate milestone that *"must not be scoped in a single sitting."*
⚠️ **Bound the rendered length.** An unbounded stranger-authored blurb is a layout weapon on first contact.
**The bound's VALUE is Joe's** (`D-138`); ship a plausible one.

### §4.5 — 🛑 THE OWN-ROW PATH — **NEW IN v1.1 (F-6 / `B-11`)**

**File:** `ui/common/lib/components/widgets/stream-panel.svelte` — the `echoToDescriptor` function (`:118-133` pre-edit, **`:121` now** — cite the symbol)

🛑 **§4.1 ALONE FIXES THE RECEIVER AND LEAVES THE SENDER BLIND.** Own rows never reach `projectEvent`: they
come from `echo.forRoom(…).map(echoToDescriptor)`, and that function **hardcodes** `bodyExtras` to the
send-status mount. ⇒ **without this step, the person who sends an intro never sees it.**

🔑 **`I2` IS THE REASON THIS IS FIXED RATHER THAN FILED.** J-701's argument was that as the opening message
the intro is *"attributed, in the DAG, redactable, blockable and reportable, **and symmetry is free** because
the initiator sees their own intro as message one."* **Symmetry is free only because own rows are
`MessageDescriptor`s too — it is not free by accident, and it is not free if nobody writes this step.**

1. `EchoMessage` (`echo-state.svelte.ts:73-85`) gains the intro, following the file's own
   **absent-not-empty-string** discipline (`:80-82`).
2. `echoToDescriptor` **appends** the intro mount to `bodyExtras` **alongside** `send-status`. 🛑 **Appends —
   does not replace.** *A send-status LED lost on exactly the rows carrying a first contact would be the
   M-RP6.9 D-5 mistake made backwards.*
3. ⚠️ **Two mounts on one row means `mountKey` uniqueness now matters where it did not before** — V-12 ③.

🔓 **JOE'S CALL, SCAFFOLDED AS FIXED.** The alternative is to file the asymmetry as a stated limit. **Chat
scaffolds the fix because `I2` was ARGUED rather than assumed**, and a milestone that quietly drops one half
of a ruled argument has changed the ruling without saying so.

### §4.4 — Write the key: the composer

**File:** `ui/common/lib/components/widgets/composer-panel.svelte:151-168`
The first-send path (`G-7`: `create_dm_space` → latch → `echo.send` → `dmDraft.clear()`) passes the intro
alongside the text. `echo.send` (`echo-state.svelte.ts:162`) and its `invoke('send_message', …)` widen to
carry it.
🛑 **OPT-IN, NEVER AUTOMATIC** (§2.2). **A send with no intro must produce byte-identical output to today's
send** — no `xgen.intro.v1` key at all, **not an empty one.** *A key nothing writes is a key nobody has
round-tripped (`N-182`); an empty key present on every message is worse — it is a reserved field shipped
fed with nothing.*

---

## §5 — LEG 4: VERIFICATION. **CHAT DRIVES ALL OF IT (Rule 5).**

🔒 **Numbers Chat did not personally measure do not enter the record.** Clair's numbers are cross-checked,
never adopted. **Listed here so Clair can see the target — not for her to run.**

### §5.0 — 🛑 BASELINES AND CONTROLS ARE DIFFERENT ACTS — **REWRITTEN AGAIN IN v1.2**

🛑 **v1.1's FIX REPRODUCED THE DEFECT IT WAS FIXING, FOUR LINES LATER IN ITS OWN PARAGRAPH.** v1.1
correctly excluded `V-1`/`V-6`/`V-8` from V-0's subject set (*a floor cannot fail pre-edit*) and `V-2`
(*impossible before the parameter exists*) — **and then named a subject set of V-3/V-4/V-5/V-9 without walking
it.** Walked in v1.2:

| gate | pre-edit behaviour | verdict |
|---|---|---|
| **V-9** | `build_message_text_event_with_extras` **does not exist** | 🛑 **IMPOSSIBLE** — *the identical disqualifier v1.1 applied to V-2, four lines earlier* |
| **V-3** | `projectEvent` (`derive.ts:69-97`) reads no content key but `text`, so **there is no mount to drop** | 🛑 **PASSES VACUOUSLY** |
| **V-4** | no mount is ever produced, so "no mount and no crash" is trivially true | 🛑 **PASSES VACUOUSLY** |
| **V-5** | the key cannot be attached at all | 🛑 **PASSES VACUOUSLY** |

🔑 **THE HONEST CONCLUSION, STATED RATHER THAN ENGINEERED AROUND: THIS MILESTONE HAS NO PRE-EDIT POSITIVE
CONTROL, AND THAT IS NOT A GAP TO BE CLOSED.** A control must **fail before and pass after**. **Every gate
here tests behaviour that does not exist yet**, so *pre/post is the wrong axis* — and a "control" that passes
vacuously is worse than none, because it produces a green reading that means nothing. ⚠️ ***Asking a check
that cannot fail to serve as a positive control is the exact category error v1.1 named — sixth instance in
this arc, and the second time it landed inside the correction rather than the original.***

🔒 **⇒ THE TWO ACTS ARE SPLIT, BECAUSE v1.0 AND v1.1 CONFLATED THEM UNDER ONE V-NUMBER:**

**🔒 V-0a — BASELINE CAPTURE. PRE-EDIT. CHAT'S. REAL, RUNNABLE, AND OWED.** Three floors, each recorded
**with the commit it was driven at**. *A baseline is not a control: it does not discriminate, it anchors.*

### ✅ V-0a — **DRIVEN AND CAPTURED, 2026-08-15, AT `84285c2` ON A CLEAN TREE**

| floor | measured | note |
|---|---|---|
| **`cargo test --workspace`** | 🔒 **1597 passed / 0 failed / 62 ignored, across 56 suites** | 🛑 **THIS MILESTONE'S `cargo` BASELINE, MEASURED RATHER THAN INHERITED.** ⚠️ *`CLAUDE.md`'s J-676-era entry states **1596**/0/62 × 56 — **+1 since**. Not investigated here; recorded so the delta is visible rather than silently absorbed into a "matches" claim* |
| **`npx vitest run`** (from `ui/`) | 🔒 **9 files / 154 tests / 154 passed / 860 ms** | re-driven at this commit rather than carried from the `f9557ef` reading (830 ms) |
| **`npm run check`** (svelte-check) | 🔒 **0 errors / 34 warnings / 15 files** | 🔑 **DRIVEN, NOT INHERITED** — *this figure had been STATED all session from the kickoff and never once measured by Chat; it is confirmed exact* |

🛑 **TWO DIFFERENT SUITES REPORT `154` AND THEY ARE UNRELATED.** One Rust suite in the workspace reports
`154 passed`, and the vitest total is `154 tests`. ⚠️ ***A number that appears in two floors is a number that
will be quoted from the wrong one.*** **The vitest floor is `154/154 across 9 FILES`; the cargo floor is
`1597/0/62 across 56 SUITES`. Neither figure may be cited without its unit.**

📌 **AND CHAT'S OWN INSTRUMENT LIED DURING THIS CAPTURE, CAUGHT BEFORE IT WAS REPORTED.** A grep for
`^error|FAILED|panicked` over the cargo log returned **59 hits** — which reads as alarming. **PowerShell
`Select-String` is CASE-INSENSITIVE by default, so `FAILED` matched the `0 failed` inside every
`test result: ok` line.** ✅ `^test result: FAILED` returns **zero**. 🔑 ***Same species as Clair's `grep -c
$'\r'` in the same session, from the opposite direction: hers produced a false NEGATIVE about line endings,
Chat's a false POSITIVE about failures — and both instruments returned a plausible number rather than an
obvious error.*** ⇒ **belongs with `N-197`.**

**🔒 V-0b — DISCRIMINATION, POST-EDIT, FOLDED INTO EACH GATE AS AN A/B.** The discriminating form here is
**inside the post-edit build**: run `V-3` with the widget **registered** and **unregistered** and show the
mount counts differ. ***That proves the gate can tell the difference — which is the only thing a control was
ever for.*** Same shape for `V-4` (well-formed vs malformed) and `V-9` (extras with and without a `text`
key).

🛑 **AND THE VACUITY IS RECORDED AT EACH GATE RATHER THAN THE SECTION BEING QUIETLY REWRITTEN A THIRD
TIME** — *a section silently corrected twice is a section whose history no reader can reconstruct.*

🛑 **V-1's BASELINE: `cargo` IS UNMEASURED IN THIS MILESTONE.** It was refused entry during Phase-0 for
touching zero `.rs`, so **there is no inherited number and none may be quoted from memory.** ⚠️ **`unchanged`
MAY BE THE CORRECT RESULT** — new tests add, existing counts hold; *a moved number is not automatically
progress and an unmoved one is not automatically a miss.*

| gate | what it proves | how |
|---|---|---|
| **V-0a** | 🔒 **BASELINE CAPTURE, PRE-EDIT** — `cargo` (**UNMEASURED**), vitest, svelte-check, **each stamped with the commit it was driven at** | Chat, before handing over. 🛑 **NOT a control. It anchors; it does not discriminate** |
| **V-0b** | 🔒 **DISCRIMINATION** — folded into V-3 / V-4 / V-9 as a post-edit A/B, **not run as a separate pre-edit gate** | see each gate |
| **V-1** | `cargo` against **V-0a's captured baseline**, nothing else regresses. ⚠️ **unchanged may be correct** | `cargo test --workspace` **detached** via `Start-Process`, output to a log, poll for terminator lines (it exceeds the MCP timeout) |
| **V-2** | ⚠️ **an added optional Tauri param does not break a webview caller that omits it** | drive `send_message` from the live client with the old argument set. **Assumed by nobody; measured** |
| **V-3** | 🔑 **THE DEGRADATION PATH — the one that must be TESTED, NOT ASSUMED.** An event carrying `xgen.intro.v1` whose widget is **NOT registered** renders **the plain `text`** and drops the mount (W-13). 🔒 **V-0b A/B: run registered AND unregistered in the SAME post-edit build and show the mount counts DIFFER.** 🛑 *Pre-edit this passes VACUOUSLY — `projectEvent` reads no key but `text`, so there is no mount to drop* | **M1′ (`B-5`)**: Vite eval `await import('/@fs/…/stream/derive.ts')`, run `projectEvent` on a synthetic event, read back through a window global. **Real execution, no disk, no consent** |
| **V-4** | malformed payloads produce **no mount and no crash** — string, array, `null`, missing members, oversized blurb. 🔒 **V-0b A/B: well-formed vs malformed in the same build.** 🛑 *Pre-edit: VACUOUS — no mount is ever produced* | same M1′ harness, table-driven |
| **V-5** | 🛑 **1-bis HELD — TWO ARMS, AND THEY ARE DISCHARGED BY DIFFERENT MEANS.** **ARM 1:** an event with the key and NO `text` never leaves the client. **ARM 2:** a send with no key is byte-identical to an ordinary send | 🛑 **v1.3 CORRECTION — v1.2's ARM 2 WAS UNEXECUTABLE AS WRITTEN.** It said *"compare against V-0a's captured pre-edit send"*; **V-0a captured FLOORS, NOT A SEND**, and the pre-edit tree is gone. ✅ **Arm 2 is discharged by `no_extras_leaves_content_byte_identical_to_a_plain_text_event` (`m_rp_intro_extras_tests`), which compares CANONICAL BYTES — STRICTLY STRONGER than the live comparison v1.2 asked for.** *A gate that cannot be run is worse than one left open: it reads as satisfiable.* **ARM 1:** live, `desktop.rs:325`'s guard |
| **V-6** | svelte-check floor **0/34/15** unmoved or improved | `cd ui; npm run check`, launched detached, poll the output file |
| **V-7** | the intro renders in message chrome on the live client, two identities | CDP 9222, `cdp-debug.ps1`. 🛑 **Chat cannot see PNGs — screenshot, name it, ASK JOE TO LOOK** |
| **V-8** | 🔒 **NEW (F-2) — the vitest floor holds: 9 files / 154 tests / 154 passed.** Leg 3 edits **two of those nine** (`derive.test.ts`, `mounts.test.ts`), and Leg 3's new branch must arrive **covered** | `npx vitest run` from `ui/`, detached, poll the log. 🛑 **No npm script exists — invoke it directly** (`B-10`) |
| **V-9** | 🔒 **(F-3) — an `extras` map containing a key literally named `text` DOES NOT overwrite the sentence.** §3.1 leaves the mechanism to Clair, so the gate tests the OUTCOME and not her choice of mechanism. 🔒 **V-0b A/B: extras with and without a `text` key.** 🛑 *Pre-edit: IMPOSSIBLE — the function does not exist* | a `xgen-core` unit test on `build_message_text_event_with_extras` |
| **V-10** | 🔒 **NEW (F-7) — the registry contract, either way.** If `message-intro` takes an `id`, it is fed; if not, that is stated. **A widget mounted with a prop bag nothing type-checks (`B-8`) must not depend on a field no mount supplies** | vitest against `resolveMounts` + the live row |
| **V-11** | 🔒 **NEW (F-6 / `B-11`) — THE SENDER SEES THEIR OWN INTRO.** `I2` claimed symmetry is free; own rows bypass `projectEvent` entirely, so it is free only once §4.5 exists | live client, own row after send |
| **V-12** | 🔒 **NEW (census→partition) — the four unlisted failure modes:** ① `xgen.intro.v1` present on a **non-DM room** message · ② the key present on a **redacted/tombstoned** row · ③ **two** intro mounts on one row (duplicate-key crash shape, M-RP6.9) · ④ an intro on a **grouped continuation** row, where the header guard is suppressed | vitest + M1′ where pure, live where not |

🛑 **THE LIST ABOVE IS NOW ASSERTED AS A PARTITION, NOT A CENSUS — AND THAT ASSERTION IS ITSELF A CLAIM.**
*v1.0's list looked complete and was not; four failure modes were missing and V-5's second arm named no
surface. **Twice in this arc a set that looked complete was not, and once it was inside the very option Chat
recommended.** V-12 closes the four Clair found. **It does not prove there is no fifth.***

🛑 **NO NUMBER ENTERS THE RECORD WITHOUT THE SCREEN IT WAS MEASURED ON.**

### ✅ §5.1 — RE-DRIVE RESULTS (CHAT, Rule 5) — **AT `e76bac8`, CLEAN TREE, 2026-08-15**

🔒 **Every figure below was driven by Chat. Clair's report agreed throughout; her numbers were cross-checked,
not adopted.**

| gate | measured | verdict |
|---|---|---|
| **V-1** `cargo test --workspace` | **1602 / 0 / 62 × 56 suites**, `CARGO_EXIT=0`, anchored `^test result: FAILED` = **0** | ✅ **+5 vs the `84285c2` baseline of 1597**, and the five are named `m_rp_intro_extras_tests::*` |
| **V-8** `npx vitest run` | **9 files / 172 tests / 172 passed**, `VITEST_EXIT=0` | ✅ **+18 vs 154**, all in `derive.test.ts` |
| **V-6** `npm run check` | **0 errors / 34 warnings / 15 files** | ✅ **floor exactly** |
| **V-2** | 🔑 **MEASURED, NOT ASSUMED, AND NOT SIDESTEPPED.** Live `invoke('send_message', …)` **omitting `intro` entirely** and **passing `intro: null`** both returned the same structured rejection (`code 4000, space not found`) — **not a deserialisation error** | ✅ **Tauri maps a MISSING argument for `Option<T>` to `None`.** Clair's explicit `null` is belt-and-braces, **not** a load-bearing workaround |
| **V-3** | declared mounts **1** · registered registry → **1 resolved** · unregistered → **0 resolved** · `AB_differ: true` · **`body: "hello"` in BOTH** | ✅ **THE DEGRADATION PATH IS REAL AND THE A/B DISCRIMINATES** — the property the entire (d) ruling was chosen for, tested rather than argued |
| **V-4** | malformed intro (`'not-an-object'`) → **0 mounts**, `body: "hello"` | ✅ no mount, no crash, **sentence survives** |
| **V-5 arm 1** | 🔑 **`text: ''` + intro → `event_id: NULL`, `"empty message"`** · `text: '   '` + intro → **same** · `text: 'hello'` + intro → **got an `event_id`, reached the node** | ✅ **`event_id: null` PROVES NOTHING WAS EVER BUILT OR SIGNED** — the send died at the guard, not downstream. **An intro with no sentence cannot leave the client; one with a sentence can.** Also confirms Clair's deliberate split: **whitespace-only is stored in the buffer and refused at the wire** |
| **V-5 arm 2** | `no_extras_leaves_content_byte_identical_to_a_plain_text_event` passing | ✅ canonical-byte equality — **stronger than the live comparison v1.2 asked for** |
| **V-9** | `an_extras_key_named_text_never_displaces_the_message_body` passing | ✅ |
| **V-10** | resolved mount id = **`host-message-intro`** | ✅ the registry contract is **fed**, as Clair's report stated |
| **V-12 ①②④** | key on a **non-`message.text`** event → **0 mounts** · a **future `xgen.intro.v2`** → **0 mounts**, body intact · redaction covered in `derive.test.ts` | ✅ **the versioning premise holds: a v1 reader ignores a v2 key and still renders the sentence** |
| **N-182** | no intro → **the `bodyExtras` KEY IS ABSENT ENTIRELY**, not an empty array | ✅ |

🛑 **NOT DRIVEN, AND NOT CLAIMED — THREE GATES REMAIN OPEN:**

| gate | why | needs |
|---|---|---|
| **V-7** | the intro renders in message chrome, live, two identities | 🛑 **Chat cannot see PNGs.** Screenshot → **Joe looks** |
| **V-11** | 🛑 **THE SENDER SEES THEIR OWN INTRO** — `§4.5`'s own-row path | a real DM between two identities. *The bogus-space probe exercised the COMMAND, not a SUCCESSFUL SEND* |
| **V-12 ③** | two mounts on one row (`mountKey` uniqueness) | a real own-row send, so it rides with V-11 |

⚠️ **V-11 IS THE ONE THAT MATTERS MOST OF THE THREE.** It is **the exact half `F-6` was about** — surfaced by
Clair's cold read and fixed by scaffold. ***Fixing something a cold read surfaced and then not proving it is
the shape this arc keeps failing at.*** If `§4.5` is subtly wrong, **the sender of a first-contact message
sees nothing and nothing anywhere reports it.**

🛑 **AND EVERY LINE NUMBER IN THIS RUNBOOK IS NOW STALE BY ITS OWN CHANGES.** `desktop.rs:313` was the
empty-text guard; **it is now `intro: Option<serde_json::Value>` and the guard moved to `:325`.** The
registry went **`:145` → `:169`**; `echoToDescriptor` **`:118-133` → `:121`**. ⚠️ **And §4.2's INSTRUCTION
line still read `:135` throughout Leg 3** — v1.1 corrected the grounding row and the correction list **and
left the row an implementer actually follows**, which is a **fourth** wrong pointing for one line and the
first one in an instruction rather than a reference. 🔑 ***A CORRECTION APPLIED TO THE CITED INSTANCE RATHER
THAN TO THE CLASS IS NOT A CORRECTION.*** ⇒ **cite SYMBOLS, not lines.**

### 🛑 §5.2 — FOUR INSTRUMENT FAILURES IN ONE SESSION, ALL CHAT'S — owed to `N-197`

🔑 **THREE SEATS HIT THE SAME SPECIES IN ONE SESSION**, and `N-197` is therefore much larger than the
line-ending note it started as.

1. `Select-String "FAILED"` over the cargo log — **case-insensitive by default**, so it matched `0 failed`
   inside **every PASSING** `test result: ok` line. **59 false hits, read as alarming.**
2. The bridge-up poll printed `EVAL RESULT: object` **six times** while Chat's own `-match` reported *"not
   yet"* — **`cdp-debug.ps1` writes via `Write-Host`, so `2>&1 | Out-String` captures NOTHING.**
3. The first M1′ probe returned **blank, with no error at all**; the cause was invisible until a `catch` was
   added.
4. 🛑 **Then it threw — because Chat called `projectEvent(ev)` with ONE argument when it takes THREE**
   (`e, selfId, redactedIds`). ***N-194's question was never asked of Chat's own probe: what would this
   return if the code were RIGHT? It would still throw.***

⇒ **THE GENERALISATION, EARNED FROM BOTH DIRECTIONS IN ONE SESSION: a check whose failure mode produces the
same reading as success is not a check** — Clair's `grep -c $'\r'` produced a false NEGATIVE about line
endings; Chat's `FAILED` grep a false POSITIVE about failures; **and both returned a PLAUSIBLE number rather
than an obvious error.**

🛑 **NO NUMBER ENTERS THE RECORD WITHOUT THE SCREEN IT WAS MEASURED ON.** J-731 measured a previous arc's
stated screen to be stale (recorded *"7 Spaces, 3 DMs"*, actually 8 and 4). **Record the screen or record no
number.**
🛑 **THE CATALOGUE IS CURRENTLY UNMEASURED** and stays that way until its harness is located. **Do not write
435 from memory.**

---

## §6 — DEVIATION PROTOCOL (Rule 6)

**Clair reports; she does not absorb.** Report — do not redesign — if any of these is true:

1. A step conflicts with one of the three §0 locks.
2. A step would require touching `body` in `derive.ts:81`, or weakening `desktop.rs:313`.
3. `extras` cannot be merged into `content` without a wrapper (that would change the wire shape Joe ruled).
4. Widening the Tauri command turns out to break omitting callers (**V-2's risk, surfaced early is free**).
5. Any step needs a value that is Joe's and the scaffold has none. 🛑 **`D-138`: ship a plausible value and
   REPORT it. DO NOT BLANK, and do not file it to `M-RP-SKIN` and wait** — *that inversion has happened
   twice and the scaffold-not-blank rule is the correction.*
6. 🔑 **A claim in THIS RUNBOOK does not match source.** *Every real defect in this arc came from outside the
   text — Joe's recall, Clair reading, or the live client. **Chat's own re-reads have never once caught
   one.*** ⚠️ **The `B-` table is Chat's measurement and is exactly the kind of thing that has been wrong
   before: `G-1` and `G-4` in the Phase-0's own kickoff were both internally consistent and both false.**

---

## §7 — WHAT THIS RUNBOOK MUST NOT DO

1. 🛑 **Must not put a `WidgetMount`, `widgetId`, or any render instruction on the wire** (`N-172`, `I3`).
   The wire carries **DATA**; the receiver picks the widget. **`props.intro` is built client-side in
   `derive.ts` from wire data — the wire never names `message-intro`.**
2. 🛑 **Must not render the intro as system chrome** (`I1`, `D-113` S-5) — ruled at J-701 by argument.
3. 🛑 **Must not open `{@html}` or a sanitiser surface** (§4.3).
4. 🛑 **Must not merge with `dm-intro`'s draft page** (`G-8a`).
5. 🛑 **Must not absorb `M-INTRO-POLICY`** — it triggers on *"M-RP-INTRO lands"*, is **protocol + node +
   client, explicitly NOT a UI leg**, and its own lock stands: **the filter is enforced in the CLIENT, not
   the node** (`D-143`).
6. 🛑 **Must not reserve anything unfed** (`N-182`) — no `intro: null`, no empty key on ordinary messages.
7. 🛑 **Must not touch `skin.css`.** ⚠️ *The standing belief that skin.css is never in a Chat commit is
   FALSE (`8a650b1`, `03c92cc`, `36e7a11`, `5222e64`) — but its VALUES are Joe's, and they are not in
   Clair's commit for this leg.*

---

## §8 — DoD

**Legs 1–3 are DONE when:**

1. Every step in §3 and §4 is implemented, **or reported as a deviation under §6.**
2. **Every gate in §5 is driven by Chat** (Rule 5), **each stated with the screen it was measured on.**
3. 🔒 **1-bis verified, not asserted** — V-5.
4. 🔒 **The degradation path verified, not assumed** — V-3. *This is the property the whole (d) ruling was
   chosen for; if it is only reasoned about, the milestone's central claim is untested.*
5. Floors re-measured: `cargo` (**now a real floor**), svelte-check. **Catalogue only if its harness is
   located and driven.**
6. No phase-limit note survives the leg that lifts its limit (`N-109`).
7. **Two-seat commit discipline:** Clair's code commit first, Chat's doc-bridge second, **Joe pushes both.**

🛑 **"Commit pushed" IS NOT A DoD ITEM.** `Status: COMPLETED` in this file's header is the canonical signal.
📌 **The milestone close is Phase-0 §6 row 5 and is NOT restated here.**

---

## §9 — RECOMMENDED: CLAIR'S COLD READ, BEFORE ANY CODE

**Point her at §1's `B-` table FIRST, cold, against source** — before §3/§4 can frame it. The questions:

1. **Does each `B-` row's cited line say what the row claims?** *`G-1` and `G-4` did not, and both read as
   complete.*
2. **Is §5's gate list a partition or a census?** 🔑 ***Twice in this arc a set that looked complete was
   not — and once was inside the very option Chat had recommended.*** The likeliest defect here is **a
   failure mode nobody listed**, not a wrong entry.
3. **§6's `F9` pass:** can each gate be **RUN**, in the order written, **from the seat that owns it**?
4. **Is anything reserved that §7.6 forbids?**

📌 **Standing Clair up is Joe's.**
