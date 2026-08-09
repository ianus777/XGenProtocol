# M-RP-MEMBER-ACT Leg C-bis — the member with no DM: creation, the first send, and the erased identity
> **Status**: ACTIVE  
> Version: 1.7  
> Date: Aug 2026  
> **Last updated**: 2026-08-09  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS FILE IS, AND WHY IT EXISTS BEFORE ITS DESIGN DOES

📌 **§0 DESCRIBES v1.0 AND IS KEPT AS WRITTEN.** ⚠️ **AS OF v1.2 THE FILE IS NO LONGER ONLY A DEBT
REGISTER** — §5–§7 are the Phase-0 the paragraph below says is still owed, written at J-703. **Kept, not
rewritten** (`D-131`): the paragraph is true about why the file was CREATED, and the annotation is what makes
it true about what the file now IS. 🛑 **What still stands from it verbatim: nothing outside §1's ruling and
§5's 🔒-marked lines is locked.**

🛑 **THIS IS A DEBT REGISTER, NOT A DESIGN.** It was created at Leg C's close (J-700) for one reason: `N-109`
requires that a limitation shipped by one milestone be written into the **DoD of the milestone that lifts
it**, not left as folklore. Leg C shipped a **dead control** and two **blocked measurements**; without this
file they would have had nowhere to land except a JOURNAL entry nobody re-reads.

⚠️ **NOTHING BELOW IS LOCKED.** No option is chosen, no approach is endorsed. Everything here is an
**obligation inherited from Leg C**, plus the ground truth needed to design against it. **The design pass is
a separate exercise and needs its own Phase-0** (`D-071`: subsystem audits precede dependent milestones).

📌 **Leg C closed at `6a6c066`** — C-1 `524d4f7` (`selectOnActivate`), C-2 `b5d0908` (`roomLatch.latch()`),
C-3 `6a6c066` (R7 acts). All three gates green; see `RUNBOOK_MEMBER_ACT_LEG_C.md` **v1.2 COMPLETED** §8.

⚠️ **v1.1 (J-702) RECORDED `J-692`'s RULING AND NOTHING ELSE.** ✅ **SUPERSEDED BY v1.2 (J-703), WHICH ADDS
THE PHASE-0** — §5 (design, with Joe's locks) · §6 (legs) · §7 (a stale canonical claim found by the pass),
and §4 renumbered to §8 so the DoD stays last. **The runbook is the remaining deliverable.**

### 📌 **GROUNDING RE-MEASURED AT `24c3409` (J-702) — THE LEG IS SMALLER THAN THE DEBT REGISTER ASSUMED**

| # | fact | site |
|---|---|---|
| **G1** | `create_dm_space(invitee: String) -> CreateDmSpaceResult` is **already a Tauri command** | `xgen-client/src/desktop.rs:796`, registered `:1138` |
| **G2** | result carries `space_id · room_id · event_id · invitee · owner_identity_id` ⇒ **`roomLatch.latch(room_id)` needs no resolution step and no round trip** | `xgen-client/src/ops.rs:827` |
| **G3** | three-event causal chain signed and sent in order; *"ordering is the correctness contract"*; all-or-nothing abort is **built and tested** | `xgen-client/src/ops.rs:838-843` |
| **G4** | 🔑 **`create_dm_space` has ZERO hits anywhere in `ui/`** | repo-wide `ui/**/*.{ts,svelte}` |

🔑 **G4 NAMES THE LEG: this is a FRONTEND leg — no Rust, no protocol, no wire — and it is the FIRST caller
of a verb exposed since M7C and never once invoked from the webview.**

⚠️ **BUT THE VERB SIGNS AND SENDS ⇒ IT NEEDS A LIVE NODE AND IT CAN FAIL** (unlike the on-disk `get_spaces`).
**The draft's send path must say so honestly — `D-065`.**

---

## §1 — THE FOUR OWED ITEMS

### **OWED-1 — 🛑 THE DEAD CONTROL. This is the one with a user-visible cost TODAY.**

**Leg C ships a member row with no existing DM as clickable and doing NOTHING** (`RUNBOOK_MEMBER_ACT_LEG_C.md`
§5-b). Measured live at C-4: bus, latch, `rowCount` and registry all unmoved on such a click — a **silent,
complete no-op**.

⚠️ **A row that looks clickable and does nothing is a dead control** — `6.1j` and `D-113`'s correction both
bite. It was accepted **only** because it is named, owned, and time-bound **to this leg**.

🔒 **THE OBLIGATION: this leg does not close while that click is still silent.** Whatever `J-692` is ruled
to be — open a creation affordance, do nothing but say so, or refuse the click outright — **the outcome must
be that the row no longer lies about being actionable.** ⚠️ **It is legitimate for the answer to be *"the
row stops being clickable"*;** that also discharges the debt. What is NOT legitimate is closing this leg
with the no-op still in place.

📌 ~~**`J-692` IS THE BLOCKER AND IT IS JOE'S.** *What should a click on a member with no existing DM do?* —
open, unruled since 2026-08-06. **Chat does not answer it.**~~

### 🔒 **`J-692` IS RULED — OPTION B, THE DRAFT VIEW (Joe, 2026-08-08, recorded J-702).**

**A click on a member with no existing DM opens a Discord-like DRAFT view:** R5 shows an **empty stream
carrying a prepared page**, R6's composer is **LIVE**, and the DM Space is **materialised on FIRST SEND**.
📌 **The affordance is LMC, not the context menu** — Joe refused RMC as the primary route at J-701,
*"this has to be the most straight path"*.

⇒ **`OWED-1`'s route is now named.** The row stops being a dead control not by ceasing to be clickable but
by the click **doing the thing it looks like it does.** ⚠️ **THE DEBT IS NOT DISCHARGED BY THE RULING** —
it is discharged by the implementation, verified on the live client (§8).

🛑 **THE RULING'S SCOPE IS THE CLICK.** It says nothing about what the prepared page contains (appearance,
**Joe's**, `D-123`), whether a draft survives navigation, or what a failed create does. Those are this
leg's Phase-0 questions and are **carried to Joe, not assumed** (`D-121`).

### **OWED-2 — `OQ5`: DM creation to an ERASED identity (re-sited here at J-690)**

Leg C ships the erased DM **counterpart** row clickable (`L-4` = `OQ-C2` = **E-a**, Joe): clicking re-enters
the DM you are already in. Verified live at C-4 — bus takes `kind: 'identity'`, latch unmoved, **no state
created**, no crash, erased marker and tail-8 both still rendering.

🛑 **THAT IS THE *EXISTING*-DM CASE ONLY.** Creating a *new* DM **to an identity that is erased from the
registry** is untouched and unruled.

⚠️ **JOE'S STANDING CONDITION ON `L-4`, CARRIED FORWARD VERBATIM:** what an erased counterpart's row
*shows* depends on **retention** — history-expiry and auth-tier rules. **Those do not exist in code.** Leg C
was explicitly forbidden to fake an archive, an expiry, or any retention state it cannot honour, and shipped
under that constraint. 🔒 **The milestone that builds retention owes this row its behaviour** — and if that
milestone lands before this one, **this obligation moves there.**

### **OWED-3 — the partial first send**

Out of scope for Leg C by `L-7`. Untouched, undesigned. Named here so it is not rediscovered.

### **OWED-4 — 🔓 `§6` leg 5: BLOCKED BY FIXTURE, AND STILL UNRULED**

**The behaviour:** clicking a member in a group room moves the latch to the DM ⇒ `scope` changes
(`members-panel:46`, `roomLatch.effectiveSpaceId`) ⇒ **R7 re-renders to the DM's roster: two people.**
Correct by every rule the project has; it is also **the panel you just clicked in replacing itself.**

🛑 **IT COULD NOT BE MEASURED AT C-4, AND THE REASON IS THE FIXTURE, NOT THE CODE.** Measured on Joe's
client state at `6a6c066`:

| fact | measured |
|---|---|
| Spaces carrying a `counterpart` (i.e. DMs) | **exactly one** |
| that counterpart | **erased** (present in `notFoundIds`) |
| an erased NON-counterpart member in a group room | **hidden** — `§5a` E2 protects it only *inside* the DM, where `counterpart` is non-null |
| ⇒ rendered group-room rows with an existing DM | **ZERO** |

⇒ **no reachable input produces leg 5.** `§6` leg 1's full sentence (*group room → click → R5 shows that
DM*) is blocked for the same reason and was driven only in its in-DM form (= `§6` leg 4).

🔒 **IT STAYS UNRULED — `D-146`: a lock waits until its stated cost expires, and nothing has expired.**
Ruling now would be a judgement about something **nobody has yet seen on screen.**

📌 **THE OBLIGATION IS SHARED, DELIBERATELY.** The first of **this leg** (which creates DMs) or
**`M-RP-PEOPLE`** (which surfaces people independent of Space) to land makes a second, non-erased DM
reachable. **Whichever lands first owes the measurement, and owes SHOWING IT TO JOE before anyone rules.**

---

## §2 — GROUND TRUTH INHERITED (re-measure; do not trust these lines)

🛑 **CITE THE SITE, RE-MEASURE THE LINE.** Leg C's own runbook staled its `members-panel` line numbers
during its own execution — C-3 shifted them by roughly +11 and +40. **Every number here is a starting
point.**

| # | fact | site |
|---|---|---|
| **B1** | `findDmRoom` — the named local lookup, field match on `counterpart`, self excluded explicitly | `members-panel.svelte` |
| **B2** | `onMemberActivate` — `latch()` then `selection.set()`, two independent stores, NOT ordering-dependent | `members-panel.svelte` |
| **B3** | `KnownSpace.counterpart` — *"DM counterpart XGID, or the session identity for the self thread. `null` for a Space."* | `spaces-state.svelte.ts:32-34` |
| **B4** | `roomLatch.latch(roomId)` — the direct writer; **three** writers total (`note`, `clear`, `latch`) | `room-latch.svelte.ts` |
| **B5** | `entity-panel` `selectOnActivate` — default `true`; R7 passes `false` to keep its highlight DERIVED | `entity-panel.svelte` |
| **B6** | The erased-row composition problem — two marks stack on one string | `N-168` (FILED, NOT FIXED) |
| **B7** | `latch()` unconditional / `selection.set()` guarded — the half-apply shape | `N-171` (FILED, NOT FIXED) |

---

## §3 — WHAT THIS LEG MUST NOT DO

🛑 **IT MUST NOT FAKE RETENTION.** History-expiry and auth-tier rules do not exist in code. No archive
state, no expiry state, no "this conversation has been cleared" that nothing behind it can honour.

🛑 **IT MUST NOT RE-OPEN `L-3`, `L-5` OR `L-9`.** `selectOnActivate` default `true`, R7's highlight stays
**derived**, and no new store or bus. `D-146` — a locked option re-opens when its **stated cost expires**,
not when taste changes, and the supersede must name the expired cost.

🛑 **`M-RP-PEOPLE` IS NOT THIS LEG.** Filed separately; never named `contacts` (`address_book.rs:38`
reserves that word for Ch2's private contact record). ⚠️ `last_seen` must **never** render as the person's
activity.

---

## §5 — THE DESIGN (J-702 / J-703). LOCKED WHERE MARKED 🔒; EVERYTHING ELSE IS THIS FILE'S RECOMMENDATION

### §5.1 🔒 THE DRAFT IS A **RENDER** STATE, NOT A **LATCH** STATE (Joe, 2026-08-08)

`roomLatch` stays honest: **nothing is latched, because nothing exists.** A small sibling store carries the
draft. R5 renders the empty stream + prepared page; R6 gates on `canSend || draft.active`; **R7 is untouched
because `scope` never moves.**

🔑 **WHY NOT A THIRD STATE INSIDE `roomLatch` — THE STORE'S OWN HEADER IS THE ARGUMENT.**
`room-latch.svelte.ts:5-18` exists because reading the bus would grey the composer *while the user was still
looking at the conversation* (D-121, J-559). A state meaning *"no room, but pretend"* would make **`canSend`
lie** — the identical failure, re-introduced by the store built to prevent it. **One predicate, both widgets.**

⚠️ **`L-9` (no new store) WAS LEG C'S LOCK AND DOES NOT BIND C-bis** — but adding a store is a structural
call, so it was **put to Joe and ruled by him**, not assumed (`D-146`: absence of a bar is not permission).

⚠️ **NAME COLLISION, GROUNDED:** `composer-panel.svelte` already uses `draft` for its **local text
variable**. The store must NOT be called `draft`. Recommended: `dmDraft` / `dm-draft.svelte.ts`.

### §5.2 THE SEND SEAM — MEASURED, NOT DESIGNED

`composer-panel.svelte:65-76`:

```
function submit(): void {
  const text = draft.trim();
  const spaceId = roomLatch.effectiveSpaceId;
  const roomId  = roomLatch.effectiveRoomId;
  if (!text || spaceId == null || roomId == null) return;   // ← the early return
  draft = '';
  void echo.send(spaceId, roomId, text);
}
```

🔑 **THE EARLY RETURN IS WHY A DRAFT CANNOT RIDE THE EXISTING PATH BY FAKING A LATCH** — it re-checks at the
moment of action *"because a disabled button is a courtesy, never a guarantee"*. The draft needs its **own
branch**, above the early return, not a synthetic `roomId`.

**The send sequence:**

```
create_dm_space(invitee)          → CreateDmSpaceResult
roomLatch.latch(result.room_id)   → the latch becomes REAL, first time
echo.send(space_id, room_id, text)
dmDraft.clear()
```

✅ **After that instant every shipped mechanism takes over untouched** — no new persistence, no new fill, no
new wire. `G2` guarantees `room_id` comes back in the result, so **there is no resolution step and no round
trip** between create and latch.

### §5.3 🔒 ① THE DRAFT SURVIVES NAVIGATION, KEYED BY COUNTERPART (Joe, 2026-08-08)

Open a draft on Alice, click a room, come back — **the draft is still there, with the typed text.**
**User-visible impact:** losing typed text is the worst small betrayal a chat app commits, and it is silent.
**Resource cost:** one map keyed by `identityId` in the sibling store; no persistence (the client holds no
user data — J-598 ⑥, Joe's lock), so it dies with the session like every other client state.

### §5.4 🔒 ③ A FAILED CREATE LEAVES THE DRAFT OPEN, AND SAYS SO (Joe, 2026-08-08)

`G3`: the verb signs and sends three events in a causal chain and **aborts writing nothing** when the chain
times out (`create_dm_space_aborts_and_writes_no_record_when_chain_times_out` passes) ⇒ **the client is clean
either way.** ⚠️ **BUT IT NEEDS A LIVE NODE AND IT CAN FAIL** — unlike the on-disk `get_spaces`. **`D-065`:
the failure is surfaced, the typed text is kept, and nothing on screen implies the DM exists.**

### §5.5 🔒 ② THE PREPARED PAGE — APPEARANCE, JOE'S (`D-123`), RULED 2026-08-08

> 🔒 **AMENDMENT, JOE, 2026-08-09 (v1.5) — WHERE THE LOCKED COPY LANDS, AND WHEN.** *"make a placeholder, but
> with content. i will check it independently afterwards and maybe i do some updates manually or leave it as
> is."* ⇒ **`dm-intro.svelte` is populated at C-bis-2**, not left blank until Joe authors it. 🔑 **Nothing in
> this § is re-opened: the sentence, the twice-named counterpart and the non-binding canvas are Joe's locks
> from J-703 and are placed VERBATIM.** 🛑 **THE SKIN IS NOT AMENDED** — `skin.css` stays Joe's and stays a
> STOP, so **this §'s truncation rules do NOT ship at C-bis-2**: the page renders with the client's default
> wrapping until Joe skins it. ⚠️ **CONSEQUENCE, NAMED SO IT IS NOT DISCOVERED AS A DEFECT: the 128-byte
> no-space display name (§5.5) WILL overflow horizontally until `overflow-wrap: anywhere` lands, and the
> one-line header clamp does not exist yet.** That is a **known unskinned state with a named discharger
> (Joe)**, not a regression — and it is why C-bis-5's §5.5 gate stays owed rather than being ticked here.

🔒 **THE SENTENCE:** *"This is the start of private direct message stream with {counterpart\_display\_name}."*
🔑 **AND THE WORD *stream* IS LOAD-BEARING, NOT A PARAPHRASE OF DISCORD'S *history*.** J-598 ③ measured that
**the client has no message history at all** — `ingest` is in-memory, capped at `INGEST_CAP = 500`, with a
`dropped` counter; **every start is a cold start.** *"history"* would be a false statement about the product.

🔒 **THE COUNTERPART'S NAME APPEARS TWICE** (heading + sentence), Discord's shape. ⚠️ **PROVISIONAL** —
Joe: *"we will see."*

🔒 **TRUNCATION IS THE WEB'S DEFAULT, NOT AN ELLIPSIS (Joe, 2026-08-08 — SUPERSEDES v1.2's `Darina F…`).**
*"all block text will be word-wrapped, except headers, that will not be and line will be cut off (letters
will be not cut). not programmed, just css behaviour."*

```css
/* headers — wrap normally, show line 1 only, never a cut glyph */
overflow-wrap: anywhere;
max-height: 1lh;            /* or line-height: 1.4 + max-height: 1.4em */
overflow: hidden;

/* block text — ordinary wrapping; `anywhere` for the no-space 128-char case */
overflow-wrap: anywhere;
```

🔑 **`white-space: nowrap` IS NOT THE MECHANISM AND CANNOT BE.** `nowrap` + `overflow:hidden` clips at a
**PIXEL** boundary and will paint half a glyph. Letting the header **wrap normally** and clamping it to one
line makes the clip land **between lines, not between letters** ⇒ *"letters will be not cut"* holds **by
construction**, with zero JS, zero `text-overflow`, zero `line-clamp` (which would force an ellipsis back).
✅ `overflow-wrap: anywhere` is **already established in the skin** — 3 sites (`1107`, `2070`, `2099`).
⚠️ `1lh` is a 2023-era unit; fine on WebView2, **not measured on Joe's runtime** — the `em` form is the
older-than-the-project equivalent.

⚠️ **WHY `overflow-wrap: anywhere` IS REQUIRED AND NOT DECORATION:** validation rejects only **empty**,
**>128 bytes**, and **control characters** (`registration.rs:515-519`) — **there is no whitespace
requirement** ⇒ a 128-byte single word is a **VALID display name** and will not wrap. Without `anywhere` it
overflows the paragraph horizontally on hostile input. 📌 *Footnote:* `MAX_DISPLAY_NAME_LEN` counts **BYTES**
(`name.len()`), so ~128 ASCII but ~64 Slovak-diacritic and ~42 CJK glyphs.

📌 **THIS DROPS THREE THINGS FROM v1.2:** the `display:inline-block` rule for the in-sentence name, the
"name pinned to one line inside a wrapping sentence" oddity, **and lock ⑥ becomes MOOT ON THIS PAGE** — no
CSS ellipsis means nothing can stack with `tail8`'s leading one. ⚠️ **⑥ STILL STANDS FOR R7**, where it is
already the shipped behaviour.

⚠️ **THE COST, RECORDED WITH OPEN EYES:** a word-boundary cut **drops a whole word silently** —
`Darina Fulopova` renders as `Darina`, which reads as a complete first name rather than a truncation. **Joe
ruled the cost acceptable on the ground that THE CANVAS TEXT IS NON-BINDING, JUST INFORMATIVE** — see §5.6-bis.
📌 **Set a `min-width` on the header so "no word fits ⇒ blank" is unreachable.**

### §5.6 🔑 THE NAMELESS COUNTERPART IS A REAL STATE, AND THE PROTOCOL SAYS SO

Joe asked whether the protocol permits no display name. **It does, deliberately, and there is a test named
for it.**

| fact | site |
|---|---|
| `IdentityRecord.display_name` is **`Option<String>`** | `xgen-core/src/identity/registry.rs:34` |
| validation sits **inside `if let Some(name)`** ⇒ absent is accepted | `registration.rs:514-520` |
| **empty string is REJECTED** (`DisplayNameInvalid`, 3009) · >128 rejected · control char rejected | `registration.rs:515-519` |
| `no_display_name_accepted` asserts `record.display_name.is_none()` | `registration.rs:1023-1028` |
| `empty_display_name_rejected` | `registration.rs:1005` |

🔑 **THE PROTOCOL DRAWS A TWO-STATE THE UI MUST RESPECT: *no name* IS LEGAL, *empty name* IS NOT.** A blank
label can never arrive ⇒ either a real 1–128-char name or nothing at all ⇒ **`tail8` is a sufficient answer
and nothing more elaborate is needed.** ⚠️ *Reading, not measurement:* in a no-anonymity protocol the
identity is the **key**, not the label; nameless is coherent with the thesis rather than a hole in it.
**Ch1–Ch2 not read this session — mechanism measured, rationale inferred.**

**TWO LIVE CASES REACH THIS PAGE, AND NEITHER IS A CONNECTION OUTAGE** (Joe asked specifically):

1. **LIVE-JOINED** — `MemberEntry.unresolved?: boolean`, *"stamped `true` by `addMember` on a member that
   reached the roster via a LIVE membership delta — i.e. the address book was never consulted for them
   through a fill"* (`address-book.svelte.ts:28-34`, M-RP-LIVEFEED-REFRESH Leg A). **Transient — cleared by
   the next fill.**
2. **NEVER-NAMED** — `display_name: None`. **Permanent until they set one.**

📌 **OUTAGE IS NOT A CASE:** roster and book arrive together from one `fill_space_records` ⇒ no connection,
no roster, R7 shows self only. **You cannot click a member you cannot see.**

📌 **ERASED IS NOT A CASE FOR THIS PAGE:** `members-panel.svelte:148` filters erased members out of R7
**unless they are the DM counterpart**, and a draft has no DM by definition ⇒ **the erased row is never a
draft target.** That is `OWED-2`, and it stays where it is.

🔓 **STILL OPEN, JOE'S:** — ✅ **CLOSED 2026-08-08, DELEGATED TO CHAT** (*"it doesn't need to, if is better
for its purpose and you think so"*). See §5.6-bis.

### §5.6-bis 🔒 THE CANVAS IS NON-BINDING — AND THAT IS TRUE OF THE WHOLE CLIENT, NOT JUST THIS PAGE

🔒 **JOE, 2026-08-08:** *"such a canvas text will be non-binding, just informative."* ⇒ the page is
**ORIENTATION, NOT ATTESTATION**, and a truncation that loses information is therefore not a defect.

🔑 **GREPPED, AND IT GENERALISES FURTHER THAN THE RULING CLAIMED:** across **every `.svelte` in `ui/`, the
FULL XGID OF ANOTHER IDENTITY IS RENDERED NOWHERE.** `self-panel.svelte:38-42` renders a real full XGID —
**only your own.** Everything about someone else is `display_name ?? tail8()` (`members-panel:117`) or **two
initials** (`entity-avatar:97`, `xgid.slice(-2)`). ⇒ **Chat had been holding this page to a standard NO
SURFACE IN THE PRODUCT MEETS**, and the truncation objection was never load-bearing.

**✅ TWO CALLS DELEGATED TO CHAT BY JOE AND TAKEN HERE. BOTH ARE APPEARANCE-ADJACENT AND BOTH ARE CHEAPLY
OVERTURNABLE — marked so deliberately (`D-123`: the seat is about who DECIDES, and Joe handed these over
explicitly rather than by default).**

**① NO XGID ROW UNDER THE NAME.**
- the page's purpose is orientation; an xgid row is the one element that would **look like attestation**
- 🛑 **`D-065`: it would fake backing.** A 44-char key shown with **no fingerprint, no known-good to compare
  against and no trust store** invites a check the product cannot perform ⇒ **showing the key without the
  means to verify it is worse than showing neither**, because it converts a filed gap into an apparent feature
- consistency: this must not be the one exception, least of all on the **first** screen
- 📌 Discord's second line (`darinafulopova`) is a **handle**, and **XGen has no handles** ⇒ the row has
  nothing to hold. **Dropping it is the honest match, not a subtraction.**

**② THE NAMELESS SENTENCE IS IDENTICAL**, with `…a1b2c3d4` substituted. **No branch.** A special wording
would have the page assert something **about them** (*"this person has no name"*) — a judgment an orientation
surface should not make. ⚠️ **Cost, stated:** in the nameless case the page carries `…a1b2c3d4` and little
else. **Thin — but exactly what R7 already gives, so nothing is lost relative to the product.**

### 🔓 FILED, NOT THIS LEG'S, AND NOT DESIGNED — **THE FIRST-CONTACT VERIFICATION GAP**

⚠️ **There is no surface anywhere in the client that lets a person confirm WHO THEY ARE ABOUT TO DM.** No
full XGID, no fingerprint, no verification affordance, no comparison target. **In a no-anonymity protocol
that is a real gap**, and ① above makes it *more* visible rather than papering it.

🛑 **NO DESIGN, NO NAME MINTED, NO RIDER.** Filed as an observation against `M-RP-INTRO` / `M-INTRO-POLICY`,
both of which are **FILED, NOT SCHEDULED**. 📌 It is recorded here only because **this pass is what surfaced
it**, and a gap found and left unwritten is a gap rediscovered from scratch.

### §5.7 🛑 THE BUTTON CENSUS — WHY THE SKETCH'S CONTROLS DO **NOT** SHIP

Joe's sketch (*taken as a layout sample, not a token spec*) carried `{button-remove-friend}`,
`{button-block}`, `{button-wave}`, `{mutual-friends}`. **Measured across the real crates:**

| token | measured | verdict |
|---|---|---|
| `{avatar}` `{counterpart_display_name}` | `EntityDescriptor` renders both today | ✅ **ships** |
| `{button-remove-friend}` | **`friend` = ZERO hits**; the one match is `conn.rs:13` *"log-friendly"* | 🛑 no friend model exists |
| `{button-block}` | **240 `block` hits, every one DAG/async plumbing** (`unblocked` pending events, blocking channels). No block verb, no blocklist, no wire event | 🛑 nothing behind it |
| `{mutual-friends}` | **18 `mutual` hits, all concurrency prose.** And **`KnownSpace` carries no member list** (`spaces-state.svelte.ts:24-34`) ⇒ rosters exist only for the latched Space | 🛑 not even client-derivable |
| `{button-wave}` | a wave is a **message**; the composer's send path already does it | ⚠️ real, but **NOT this leg** (Joe: *"later elements"*) |

🛑 **SHIPPING THREE GREYED CONTROLS WOULD REPLACE ONE DEAD CONTROL WITH THREE, ON THE FIRST SCREEN A PERSON
SEES OF A STRANGER.** `D-113`'s correction governs: a control is disabled only for a reason **true of that
subject and legible to the user**. *"We never built blocking"* is not that reason — it is `6.1j`'s and
`J-500`'s refusal exactly, and **this leg exists to DELETE a dead control.**

🔑 **AND THE CONTROL SET IS ANSWERING A QUESTION XGEN DOES NOT HAVE.** Remove Friend / Block / Mutual Servers
are all *"who is this stranger and how do I get rid of them"* — the problem an **anonymous** network has.
**Blocking will become real** (J-701 already leans on *blockable, reportable*); it is not real now.
→ **FILED, NOT BUILT: `M-RP-BLOCK`** (the verb, then the button) inherits those seats.

🛑 **THE PAGE IS A COMPONENT, NOT A PROCESSED STRING — BUT §5.8's MOUNT POINT WAS WRONG AND IS SUPERSEDED
BY J-704.** The `background` socket is **NOT** this page's home: `message-stream.svelte:255` renders it inside
`<div class="message-stream-bg" aria-hidden="true">` — **hardcoded** ⇒ the intro would be the **only content
on screen and invisible to assistive tech**, and *"later elements"* (buttons) would be **unreachable**.
📌 **The socket is correct for WALLPAPER and wrong for CONTENT.** 📌 `message.svelte:133-136` closes the other
escape: the whole `system` sub-tree is **one `<Paragraph>`** ⇒ **a system row can carry the SENTENCE but not
the PAGE.** ✅ **THE HOME IS AN `above` `WidgetMount[]` SOCKET ON `stream-panel`**, resolved through
`resolveMounts` exactly as `message-stream:119` and `region-shell:90` already do — see
`tasks/RUNBOOK_MEMBER_ACT_LEG_C_BIS.md` §2 for the live measurement that forced it.

### §5.8 🔒 THE PAGE IS A COMPONENT, NOT A PROCESSED STRING

The socket is `background?: WidgetMount[]` (message-family Phase-0 §9.4, J-481) — it takes a **component**,
not text. Two live tenants prove the seam: `grid-plate.svelte`, `send-status.svelte`.

🛑 **THE "HTML PROCESSOR" ROUTE IS REFUSED HERE, AND THE REASON IS CONCRETE, NOT STYLISTIC.** Kind-4
(`string → safeHTML`) **does not exist**: `use:render` has zero implementations, `sanitize|safeHTML` returns
nothing outside `node_modules`, and `processor/` holds only the **edit-side** kinds 1/2/3. Building it here
would ship `M-RP-PROCESSOR-RENDER`'s engine **with none of its threat model** — and `M-RP-INTRO`, the
milestone that renders **a stranger's authored card**, would inherit it looking pre-approved. That milestone
is **fifth in Joe's own J-564 sequence** and carries his note that it **must not be scoped in a single
sitting**. `D-146`: nothing about that cost has expired.

🔑 **AND THE DECIDING FACT IS THE TOKEN ITSELF: `{counterpart_display_name}` IS WIRE DATA** — authored by a
person you have never met, up to **128 characters** (`MAX_DISPLAY_NAME_LEN`), appearing **twice**, on first
contact. **Interpolating it into an `{@html}` string is an injection point at the highest-value spot in the
product.** A component makes it a **text node**, escaped by construction, for free.
✅ `{@html}` ships in exactly ONE place today — `icon.svelte:25`, *shape geometry only, XSS-free by
construction* (N-032) — and that precedent covers a **compile-time constant**, which a display name is not.

📌 **THE FILE IS JOE'S**, on the `skin.css` precedent: he writes the markup (structure, order, nesting) and
the skin. **Named as his in the runbook so no implementation commit folds it in.** *"Later elements"* is
additive — a button is one more tag in the same file when its verb exists. **No architecture change, no
migration.**

⚠️ **`N-172` HELD FOR FREE:** the page is **locally composed** and projects the **reader's own** state. It
must render the counterpart's **identity** (name, avatar, xgid — already on screen elsewhere) and **nothing
they authored**. That is `M-RP-INTRO`, filed and undesigned, **and it must not arrive as a rider here.**

---

## §6 — PROPOSED LEGS (Chat's; the split is Chat's seat under `D-123`)

> 🛑 **SWEPT AT v1.6 (J-706) — THIS TABLE WAS A FOUR-LEG SPLIT AND THE RUNBOOK HAS SHIPPED FIVE SINCE v1.0.**
> Clair flagged the drift at the C-bis-2 hand-back. **The runbook governs**
> (`RUNBOOK_MEMBER_ACT_LEG_C_BIS.md` §3); this table was the PROPOSAL that preceded it and had been left
> standing as if it were current. 📌 *Species: **a superseded record that never announced its own
> supersession** — harmless while both are read together, and a trap the moment one is read alone.*
> **The live split is five legs; the statuses below are the record.**

| leg | content | status |
|---|---|---|
| **C-bis-1** | `stream-panel` becomes a flex column + the empty `above` socket + the `dm-intro` placeholder | ✅ `e0d4d9a` (J-705) |
| **C-bis-2** | the `dmDraft` store + R7's click opens a draft + the fed page + the empty-stream swap | ✅ `96a935f` (J-706) |
| **C-bis-3** | R6 gates on `canSend \|\| dmDraft.active`; the composer's draft branch **above** the early return; the text routes through `dmDraft`; **still no create** | ✅ `8601e677` (J-707) |
| **C-bis-4** | the send sequence (`create_dm_space` → **`loadSpaces`** → `latch` → `echo.send` → clear) + the failure surface | ✅ `37c09d7` (J-708) |
| **C-bis-5** | live CDP verify + `OWED-4` measurement **shown to Joe** + records | 🟡 NEXT |
| **C-bis-6** | **after the create the client ORIENTS** — R1 unselects while in a DM, R2 follows (Joe's rule) | 🟡 |
| **C-bis-7** | **R7 reads `counterpart` from the SPACE RECORD** ⇒ a DM shows **self + counterpart**; the draft-row highlight folds in | 🟡 |

> 🔒 **THE R7 RULE, LOCKED BY JOE 2026-08-09 — R7 SHOWS THE PARTICIPANTS OF THE STREAM YOU ARE LOOKING AT,
> WHENEVER THEY EXIST.** Group room → the roster. Existing DM → **self + counterpart**. Draft → **the group
> roster it was opened from**, because 🛑 **the DM does not exist yet and synthesising two rows would invent a
> roster no fill produced (`D-065`)**. ⚠️ **The draft case is NOT an exception — it is the rule meeting a Space
> that has not been created.** Joe: *"draft member panel state is correct. decision to change members list is
> executed with existing dm stream."*

> 🔓 **OQ3 MAY BE CHEAPER THAN FILED — MEASURED 2026-08-09, NOT YET RULED.** A2/A3's cost was written as
> *"`KnownSpace` gains a field — **Rust** — the cargo floor returns."* **It does not need one:**
> `KnownSpace.counterpart` **already distinguishes a DM today**, verified across all six Spaces in Joe's live
> state (Engineering / Design / LegBSpace / LegF Verification → `NULL`; both DM Spaces → set).
> ⚠️ **`G13` STILL BITES** — `counterpart` would not clear on promotion either, so *"was born a DM"* vs *"is
> a DM"* remains undecided. **But the Rust half of A3's price tag looks removable.** 🛑 **The filter stays
> RENDER-ONLY regardless** (the `resolveLatched` / `canSend` trap already recorded above).

🔑 **VISIBLE FIRST** (Joe's standing brief): C-bis-1 puts the page on screen where he can correct it **while
the milestone is still open**, before any create path exists.

⚠️ **`N-171` IS OPENED BY THIS LEG AND THEREFORE FIXED BY IT** — C-bis-2 touches `onMemberActivate`, so the
lookup moves above `latch()`. The locked write ORDER is untouched.

⚠️ **ZERO RUST.** `G4`: `create_dm_space` has no hits in `ui/` and this leg is its first caller. **`cargo`
must stay 1597/0/62 × 56 IDENTICAL — that is the leg's proof, not its assumption.**

---

## §7 — ⚠️ A STALE CLAIM IN A CANONICAL RECORD, FOUND BY THIS PASS

`CLAUDE.md`'s PLAY head carries J-618's finding that *"the fallback name is not a tail-8: `tail()` returns
the whole final segment (`ed25519:<~44 chars>`) and `skin.css:2452-2458` clips it LEFT-anchored ⇒ two
unresolved members are indistinguishable from each other."*

🛑 **RE-MEASURED AT `24c3409`, IT IS FALSE AGAINST THE CURRENT BUILD:**

- **`tail()` does not exist in `ui/`.** The only helper is `tail8` (`members-panel.svelte:51`) =
  `` `…${xgid.slice(-8)}` ``.
- **Nothing is left-anchored.** `.ei-name` (`ui/assets/skin.css:2474-2481`) is stock `overflow:hidden` +
  `text-overflow: ellipsis` + `white-space: nowrap` — **trailing**. **Zero `direction: rtl`, zero
  `unicode-bidi` in the entire file.** The cited lines do not hold that rule.

⇒ **The build does what Joe locked at J-588 (`D-142`): the last 8 bytes — the distinguishing ones — survive.**
Most likely the M-RP-MEMBERS / M-RP-MEMBER-ACT rewrite of `members-panel` closed the gap and nobody returned
to the entry.

📌 **OWED: read J-618 and annotate at the site (`D-131`)** — not delete, not rewrite. ⚠️ **Chat repeated this
claim TWICE in one session without re-measuring it and reasoned on top of it; the grep that killed it was run
only because Joe asked a question that required it.** *The kickoff's own ① — your re-reads catch almost
nothing — demonstrated inside the session that quotes it.*

---

## §8 — DoD

- [ ] **`OWED-1` discharged** — a member row with no existing DM **no longer presents as actionable while
      doing nothing**, by whichever route `J-692` is ruled. **Verified on the live client, not inferred.**
- [ ] **`OWED-2` addressed or explicitly re-sited** to the retention milestone, named in that milestone's DoD.
- [ ] **`OWED-3`** designed or explicitly deferred with an owner named.
- [ ] **`OWED-4` measured and SHOWN to Joe** if this leg makes a second non-erased DM reachable — **shown
      before any ruling**, per `D-146`.
- [ ] `N-171`'s half-apply corrected **if** `onMemberActivate` is opened by this leg (move the lookup above
      `latch()`; the locked write ORDER is untouched).
- [ ] Floors held: cargo · svelte-check · catalogue · client registry quiescent baseline — **re-measured,
      never inherited.**
- [ ] Joe's client state file **byte-identical**, read before and after.
- [ ] **§5.5 realised:** the sentence verbatim, the name **twice**, **no ellipsis anywhere** — header wraps
      and is clamped to line 1, **no glyph is cut**, paragraphs wrap normally. Confirmed on the **PAINTED
      DOM**, not on the source. ⚠️ **A 128-byte NO-SPACE name is fed** and neither overflows nor blanks the
      header (`overflow-wrap: anywhere` + `min-width`).
- [ ] **§5.6-bis held: NO xgid row, and no full XGID of another identity anywhere on the page.**
- [ ] **§5.6 exercised:** the **nameless counterpart** path driven — the page renders `…a1b2c3d4` and the
      sentence still reads. ⚠️ **Fed, not asserted (`N-091`)** — a branch nobody has run is a branch nobody
      has tested.
- [ ] **§5.4 exercised: the create is made to FAIL** (node down) — the draft stays open, the typed text
      survives, the failure is on screen, and **nothing implies the DM exists**. `D-065`.
- [ ] **§5.3 exercised:** type into a draft → navigate to a room → return → **the text is still there.**
- [ ] **§5.7 held: NO control ships whose verb does not exist.** `btns` on the painted page counted, and the
      count is **justified row by row**, not merely reported.
- [ ] **`cargo` 1597/0/62 × 56 IDENTICAL** — the proof of `G4`'s zero-Rust claim, summed programmatically.
- [ ] **§7 discharged:** J-618 read and **annotated at the site** (`D-131`), or explicitly re-sited with an
      owner named.
- [ ] `dm-intro.svelte` (or its final name) **named as JOE'S FILE in the runbook** — never folded into an
      implementation commit, on the `skin.css` precedent.

⚠️ **THE PROBE DISCIPLINE APPLIES TO EVERY ITEM ABOVE WHOSE PASS CONDITION IS AN EMPTY RESULT** — `N-099`,
`N-169` (`get(id).state.<field>`), `N-170` (`data-debug-id` carries the full `type#id`). **A false negative
reads exactly like a genuine absence: positively control it or do not record it.**

⚠️ **NO "commit pushed" ITEM.** It is unflippable inside the commit that performs the push;
`Status: COMPLETED` is the real signal.
