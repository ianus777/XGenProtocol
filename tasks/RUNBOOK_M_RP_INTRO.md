# M-RP-INTRO Runbook — the DM welcome intro: one additive key, two seams, and a fallback that must be tested rather than assumed
> **Status**: PENDING  
> Version: 1.0  
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

---

## §1 — 🔑 THE GROUNDING THIS RUNBOOK ADDS, BEYOND THE PHASE-0

The Phase-0 grounded the *decision*. These were measured for the *implementation* and are new here.

| # | fact | site | why it changes the plan |
|---|---|---|---|
| **B-1** | 🛑 **`build_message_text_event` HAS ~60 CALL SITES** across `exchange.rs`, `runtime.rs`, `app.rs`, `fanout.rs`, `ops.rs`, `derive.rs`, `connection.rs`, `ai_service.rs`, `resident.rs` | grep, `xgen-core` + `xgen-client` + `xgen-node` | ⇒ **WIDENING ITS SIGNATURE CHURNS ~60 SITES, almost all tests.** §3.1 takes the delegating-overload route instead: **zero call-site churn** |
| **B-2** | ✅ **only THREE call sites are production**: `resident.rs:451` (the client send path), `ai_service.rs:489` (AI reply), `ops.rs:1996` (CLI send) | same | ⇒ **the blast radius of a behaviour change is three sites, not sixty.** The other ~57 are fixtures |
| **B-3** | 🔑 **THE TWIN PATTERN EXISTS AND IS DOCUMENTED IN-FILE** — `build_message_file_event` and `build_message_redact_event` are each *"Twin of…"* with a doc comment naming the reuse | `exchange.rs:944-950`, `:983-989` | ⚠️ **BUT BOTH TWINS CARRY A DIFFERENT `EventType`. Ours reuses `EventType::MessageText` and widens CONTENT.** ⇒ **this is a NEW sub-pattern, not the existing one**, and §3.1's doc comment must say so rather than claim a precedent it does not have |
| **B-4** | 🔑 **`OutboundRequest` IS `{ space_id, room_id, text, reply }` and has exactly THREE construction sites** — `desktop.rs:355` (production), `resident.rs:711` (the definition), `resident.rs:1565` (a test) | `resident.rs:711-728` | ⇒ **adding a field is a three-site change.** Its doc comment states the design reason: *"the caller hands over intent, not an event"* (`D-067`) — **an intro payload IS intent, so it belongs here** |
| **B-5** | 🔑 **`projectEvent` IS A PURE FUNCTION IN ITS OWN UNIT-TESTED MODULE**, and it is the SINGLE site where an inbound `message.text` becomes a `MessageDescriptor` | `ui/common/lib/components/widgets/stream/derive.ts:69-97`; tests `derive.test.ts` | ⇒ **the whole read half is ONE pure function + ONE registry line.** \|\| 🔑 **AND IT IS M1′-DRIVABLE** — a Vite eval can `await import('/@fs/E:/…/stream/derive.ts')` and execute it for real, with **no disk write and no consent**. §5 uses this |
| **B-6** | ✅ **`projectEvent` READS `e.content?.text` DEFENSIVELY ALREADY** — `typeof e.content?.text === 'string' ? … : ''` | `derive.ts:81` | ⇒ **the `text` fallback is already correct and needs no change.** The intro key is purely additive here |
| **B-7** | 🔑 **THE `bodyExtras` REGISTRY IS ONE OBJECT LITERAL WITH ONE TENANT** — `const widgets = { 'send-status': SendStatus }` — and **W-13 DROPS an id it cannot resolve** | `stream-panel.svelte:135`, `mounts.ts:51` | ⇒ **registering the intro widget is one line**, and **the drop-unknown path IS the degradation path**. §5 tests it rather than trusting it |
| **B-8** | ✅ **`WidgetMount` is `{ widgetId, props?, mountKey? }`**, and `props` is `Record<string, unknown>` — 🛑 **NOTHING TYPE-CHECKS THAT A MOUNT SUPPLIES WHAT ITS WIDGET NEEDS** | `types.ts:53-71`; the warning is `send-status.svelte:37`'s own | ⇒ **the intro widget must tolerate a malformed/absent prop bag at runtime**, because the type system will not catch it |
| **B-9** | ⚠️ **`send_message` REJECTS AN EMPTY `text` BEFORE QUEUEING** — `if text.trim().is_empty() { return SendOutcome::failed("empty message") }` | `desktop.rs:313` | 🔑 **THIS GUARD IS WHAT MAKES 1-bis ENFORCEABLE AT THE SEAM**: an intro with no sentence cannot be sent at all. **Do not weaken it to let a text-less intro through** — that is exactly the failure 1-bis forbids |

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

## §3 — LEG 2: THE WIRE SEAM (RUST). CLAIR IMPLEMENTS.

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
   🛑 **Do not write *"Twin of …"* and inherit a precedent this does not have.**

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
4. 🛑 **`desktop.rs:313`'s empty-text guard STAYS EXACTLY AS IT IS** (`B-9`). **An intro with no sentence is
   not sendable, and that is the point.**

### §3.3 — The ch3 convention (documentation, and it is a deliverable)

**File:** `docs/ch3_*` — the content-key namespace convention, borrowed grammar per the §3.1 lock.
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

**File:** `ui/common/lib/components/widgets/stream-panel.svelte:135`
`const widgets = { 'send-status': SendStatus }` **gains `'message-intro': MessageIntro`.** One line (`B-7`).
🔑 **The `above` socket and `dm-intro` are NOT touched** — that is the **pre-send** draft page and this
milestone is the **post-send** artefact (`G-8a`). **Two different things wearing one word; do not merge
them.**

### §4.3 — The widget: `message-intro.svelte`

**New file:** `ui/common/lib/components/widgets/message-intro.svelte`
Renders `headline` / `blurb` from `props.intro`. 🛑 **TEXT NODES ONLY — NO `{@html}`, NO SANITISER, NO
MARKUP PATH** (Phase-0 §7.3). This is a **component, not a processed string**, for exactly the reason
`dm-intro` is: the payload is authored by a person the recipient has never met. **Markup belongs to
`M-RP-PROCESSOR-RENDER`**, which is a separate milestone that *"must not be scoped in a single sitting."*
⚠️ **Bound the rendered length.** An unbounded stranger-authored blurb is a layout weapon on first contact.
**The bound's VALUE is Joe's** (`D-138`); ship a plausible one.

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

| gate | what it proves | how |
|---|---|---|
| **V-0** | **PRE-EDIT POSITIVE CONTROL** — the gates below fail on today's tree | driven **BEFORE** Clair's first edit. 🛑 **A control captured on a pre-change build is a gate on the CHANGE, so it belongs in the IMPLEMENTER's window, not the verifier's afterthought** (`F9`) — *the E-3b sequencing error, not repeated* |
| **V-1** | `cargo` floor moves in the expected direction and nothing else regresses | `cargo test --workspace` **detached** via `Start-Process`, output to a log, poll for terminator lines (it exceeds the MCP timeout) |
| **V-2** | ⚠️ **an added optional Tauri param does not break a webview caller that omits it** | drive `send_message` from the live client with the old argument set. **Assumed by nobody; measured** |
| **V-3** | 🔑 **THE DEGRADATION PATH — the one that must be TESTED, NOT ASSUMED.** An event carrying `xgen.intro.v1` whose widget is **NOT registered** renders **the plain `text`** and drops the mount (W-13) | **M1′ (`B-5`)**: Vite eval `await import('/@fs/…/stream/derive.ts')`, run `projectEvent` on a synthetic event, read back through a window global. **Real execution, no disk, no consent** |
| **V-4** | malformed payloads produce **no mount and no crash** — string, array, `null`, missing members, oversized blurb | same M1′ harness, table-driven |
| **V-5** | 🛑 **1-bis HELD: an event with the key and NO `text` never leaves the client**, and one with `text` and no key is **byte-identical to today's send** | `desktop.rs:313` guard exercised live; canonical bytes compared |
| **V-6** | svelte-check floor **0/34/15** unmoved or improved | `cd ui; npm run check`, launched detached, poll the output file |
| **V-7** | the intro renders in message chrome on the live client, two identities | CDP 9222, `cdp-debug.ps1`. 🛑 **Chat cannot see PNGs — screenshot, name it, ASK JOE TO LOOK** |

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
