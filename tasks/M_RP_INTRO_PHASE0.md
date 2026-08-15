# M-RP-INTRO Phase-0 — the DM welcome intro: the surface was ruled, the payload never was
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-15  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS FILE IS

🎯 **THIS IS A PHASE-0 (`D-071`), NOT AN EXECUTION.** Audit → design → runbook → implement. **NO CODE. NO
RUNBOOK. NOTHING IS BUILT FROM THIS DOCUMENT** until §3's open decision is Joe-locked and a runbook is
authored against the ruling.

📌 **PROVENANCE.** `M-RP-INTRO` was FILED at **J-701** with a design conversation banked into
`docs/ROADMAP.md`'s node `Owes:`. ⚠️ **Its trigger — *"Leg C-bis lands"* — FIRED AT J-716 and the milestone
still had no Phase-0 at J-731**, having survived an entire milestone (`M-RP-MEMBER-ACT`). *A trigger that has
fired is a defect.* Joe ruled the sequencing at J-731: **close `M-RP-MEMBER-ACT` first, mint this
immediately after.** This is that document.

🛑 **AND THE SESSION KICKOFF THAT COMMISSIONED IT CARRIED THREE FALSE CLAIMS ABOUT THE WIRE.** They are
corrected in §2 and they **change the shape of §3's fork**. The kickoff's own instruction — *audit the claim
against source before it becomes an instruction, INCLUDING claims made by this kickoff* — is what caught
them. **This file's §2 is the audit; nothing here is inherited.**

**State this file was written against:** `HEAD` `90447f9` **= `origin/main` by `git ls-remote`**, clean tree.
🛑 **All three apps DOWN, MEASURED, not assumed** — `netstat` shows zero listeners on 9222 / 9322 / 9422 /
5173 / 5175, and no `xgen*` process exists. *(The `msedgewebview2` processes on the machine belong to other
applications; the Tauri host is only ours when a CDP port answers — `N-196`'s discipline, applied.)*

**Floors — stated, deliberately NOT re-run (this pass is reads only, zero `.rs`, zero `ui/**`):**
svelte-check **0/34/15**. 🛑 **The component catalogue is recorded UNMEASURED** — the harness that produces
it was not located this session, and a number that has not been driven does not enter this file. 🛑 **`cargo`
IS NOT A FLOOR FOR A UI PHASE-0** — an identical result over zero `.rs` is a **scope argument, not a
measurement**. 🛑 **NO REGISTRY NUMBER IS CARRIED** — `N-184` / `N-190` / `N-194` and J-731's stale-screen
finding (recorded *"7 Spaces, 3 DMs"*, measured **8 Spaces, 4 DMs**) make every registry count unusable as a
floor unless its screen is recorded with it.

---

## §1 — THE INHERITED DESIGN, AND WHAT SURVIVES THE AUDIT

`docs/ROADMAP.md`'s `M-RP-INTRO` node banked **an argument, not merely a conclusion**. Re-read at
`90447f9` and assessed line by line:

| # | the banked claim | verdict |
|---|---|---|
| **I1** | the intro ships as **the OPENING MESSAGE in message chrome**, never as system chrome | ✅ **STANDS, and the reasoning is the load-bearing part** — an intro rendered as system chrome is stranger-authored content **in the system's voice on first contact**, which is `D-113` **S-5**'s no-trust-chrome rule and the classic unsolicited-first-contact vector. **Privacy does not mitigate it**: the threat is one sender and one target in private, with no third party to notice |
| **I2** | as the opening message it is **attributed, in the DAG, redactable, blockable and reportable**, and **symmetry is free** (the initiator sees their own intro as message one) | ✅ **STANDS** — and §2's grounding strengthens it: the first send already produces exactly such an event |
| **I3** | **rich rendering must not put a `WidgetMount` on the wire** (`N-172`) — sender sends **DATA**, receiver renders with a widget it **already trusts**, unknown template falls back to plain text | ✅ **STANDS AND IS THE STRONGEST CONSTRAINT IN THE MILESTONE.** It is also the constraint §3's fork is most likely to spend by accident |
| **I4** | a **PUBLISHED** intro visible before first contact **has no home** — *"`IdentityRecord` carries `display_name` and `is_ai` and nothing else, so that is a new federated world-readable field and a different feature"* | ⚠️ **THE CONCLUSION SURVIVES; THE SENTENCE IS FALSE — see `G-6`.** The record carries **twelve** fields, one of which is free-form JSON. **A published intro may need no new field at all.** ⇒ **§4, ROUTED TO JOE, and explicitly OUT OF THIS MILESTONE'S SCOPE** |
| **I5** | *"needs Leg C-bis's first-send path to exist before it has anything to attach to"* | ✅ **STANDS, and the path is now grounded rather than assumed — `G-7`** |

🔑 **THE SURFACE QUESTION IS RULED AND IS NOT RE-OPENED BY THIS FILE.** J-701 ruled it **by argument, not by
taste**, and `I1`/`I2` survive the audit intact. 🛑 **WHAT J-701 NEVER DID WAS PRICE THE PAYLOAD.** That is
§3, and it is the entire content of this Phase-0.

---

## §2 — GROUNDING, MEASURED AT `90447f9`

### §2.1 — The wire, and the three kickoff claims that fell

| # | fact | site | note |
|---|---|---|---|
| **G-1** | 🛑 **`EventType` carries 59 VARIANTS, not sixteen** | `xgen-common/src/wire.rs:33` | The kickoff cited *"types.rs:964-968, SIXTEEN values"*. **That range is a TEST ARRAY** (`event_type_from_str_all_variants`) inside `xgen-core/src/wire/types.rs`, a file that only **RE-EXPORTS** the type (`:20`). ⚠️ **A test enumeration is not a type definition** — and this one is a *subset*, so it looks complete and reads complete |
| **G-1a** | the enum carries **NO `#[derive(Serialize, Deserialize)]`** and no `rename` attributes — `as_str` / `from_str` are **hand-written** | `wire.rs:29-31`, `:241`, `:248` | ⇒ a new variant is **three hand edits**, not one line. The file says so in a comment: *"Do NOT re-add derive"* |
| **G-2** | ✅ **`message.*` IS exactly FOUR** — `text · file · reaction · redact` | `wire.rs:34-37` | The kickoff's conclusion holds even though its census did not. Independently confirmed by `M-RP-MEDIA`'s own node |
| **G-3** | ✅ **`intro` HAS ZERO HITS** in `xgen-common/src/wire.rs` and `xgen-core/src/wire/types.rs` | grep | There is no `message.intro`. **This claim survives** |
| **G-4** | 🛑 **`Event.content` IS `serde_json::Value` — UNCONSTRAINED.** There is also `meta_atts: Option<Value>` | `xgen-common/src/wire.rs:487`, `:490` | The kickoff's headline — *"THE WIRE IS ONE FIELD WIDE"* — **is false about the wire.** It is true only about **today's producer** |
| **G-4a** | 🔑 **`MessageTextContent` HAS ZERO PRODUCTION READERS.** Every real reader indexes `content["text"]` **directly on the `Value`** | `ops.rs:2110`, `ai_behavior.rs:107`; the struct at `types.rs:36` appears only in its own round-trip test `:1150-1153` | ⇒ *"that is the whole content of a `message.text` event"* describes **a struct nothing uses** |
| **G-4b** | 🛑 **ZERO `deny_unknown_fields` IN THE ENTIRE CODEBASE** | grep across `xgen-common` + `xgen-core` | ⇒ **an additive content key passes through every shipped reader untouched.** This is measured, not inferred from serde's defaults |
| **G-5** | 🔑 **`EventType::Unknown(String)` EXISTS, AND AN UNRECOGNISED TYPE IS ACCEPTED-AS-OPAQUE** — structurally valid, **stored and relayed byte-identically** (FC-D3) — **and NEVER APPLIED** | enum `wire.rs:158`; validation step 6 `xgen-core/src/wire/validation.rs:109-115`; the apply chokepoint `xgen-core/src/space/state.rs:650-654`, arm `EventType::Unknown(_) => Ok(())` at `:654` (FC-D6) | 🛑 **THE FACT THAT DECIDES §3, AND THE KICKOFF DID NOT HAVE IT.** A new event type is **not rejected** by an older peer. It is **carried and silently ignored** — which is a *worse* user-visible outcome than rejection, because nothing anywhere reports a failure |

🔑 **THE SPECIES, AGAIN, AND IT BIT THE DOCUMENT THAT WARNED ABOUT IT: A CLAIM NARROWER THAN THE THING IT
DESCRIBES, REUSED AS IF COMPLETE.** `G-1` is a test read as a type. `G-4a` is a projection struct read as a
schema. **Both were internally consistent and both would have survived any re-read of the kickoff.** They
fell only when `xgen-common/src/wire.rs` and `validation.rs` were opened. *Fifth-plus instance in this arc.*

### §2.2 — The first-send path: what an intro would attach to

| # | fact | site |
|---|---|---|
| **G-7** | the C-bis send sequence, **measured**: `create_dm_space(counterpart)` → `roomLatch.latch(room_id)` → `spaceLatch.latch(space_id)` → `echo.send(space_id, room_id, text)` → `dmDraft.clear()` | `composer-panel.svelte:151-168` |
| **G-7a** | 🔑 **THE CLIENT→NODE SEND SEAM IS ONE `String` WIDE** — `send_message(space_id, room_id, text)` | `xgen-client/src/desktop.rs:305-310` |
| **G-7b** | and the event it builds is hardcoded: `EventType::MessageText` with `json!({ "text": text })` | `xgen-core/src/message/exchange.rs:1020`, `:1032`, called from `resident.rs:451` |
| **G-7c** | ⇒ **the first message in a DM is an ordinary `message.text`, and there is NO OTHER ROUTE from the webview to the wire** | derived from G-7 / G-7a / G-7b |
| **G-8** | `dm-intro` **is client-local and touches nothing on the wire** — mounted only while a draft is active, and `streamMessages = dmDraft.active ? [] : messages` | `stream-panel.svelte:160`, `:182-184`, `:231` |
| **G-8a** | ⇒ **the existing intro page VANISHES the instant the draft clears.** It is a *pre-send* affordance; `M-RP-INTRO` is a *post-send* artefact. **They are two different things wearing one word, and this file does not merge them** | derived |

🛑 **`G-7a` IS THE COST NOBODY HAD PRICED.** Even with a `message.intro` variant in the enum, **the client
physically cannot emit it**: the Tauri command, the resident's outbound queue and the builder are all
`text: String` end to end. **Option (b) in §3 is not "an enum arm + node validation" — it is a new command, a
new resident path, a new builder, and a spec chapter.**

### §2.3 — The render socket

| # | fact | site |
|---|---|---|
| **G-9** | ✅ **`bodyExtras` needs NO NEW MECHANISM.** It is a client-side `WidgetMount[]` resolved at render, built from **LOCAL** state, and `W-13` **DROPS** an id the host cannot resolve | `message.svelte:85` (`resolveMounts`), `stream-panel.svelte:129`, `:143` |
| **G-9a** | ⚠️ **`N-172`'s SOCKET CENSUS IS STALE — IT IS FOUR SOCKETS, NOT THREE.** The note's table lists `message-stream.background`, `message.bodyExtras`, `message.details`; **`stream-panel`'s `above` socket, tenanted by `dm-intro`, is missing** | `stream-panel.svelte:153-160`, `:182`; note at `ui/docs/xgen-ui-notes.md:3526` |
| **G-9b** | 🔑 **`N-172`'s WARNING NAMES THIS MILESTONE BY NAME:** *"the first tenant that renders SOMEONE ELSE'S content — a DM welcome intro, a poll result, an AI resident's diagram — is the first one facing content it did not originate"* | `xgen-ui-notes.md:3526` |

🔒 **THE RENDER HALF IS SOLVED AND THE MILESTONE IS NOT ABOUT IT.** The socket exists, is correct, and drops
unknown ids. **The whole question is the payload**, which is why §3 is the only open decision in this file.

### §2.4 — The identity record

| # | fact | site |
|---|---|---|
| **G-6** | 🛑 **`IdentityRecord` CARRIES TWELVE FIELDS, NOT TWO AND NOT SIX** — `identity_id · display_name · is_ai · ai_capabilities · registered_at · trust_assertion · devices · home_node · update_version · revoked · revoked_at · revocation_reason` | `xgen-core/src/identity/registry.rs:32` |
| **G-6a** | 🔓 **`trust_assertion` IS `Option<serde_json::Value>` — A FREE-FORM JSON VALUE ON A FEDERATED, WORLD-READABLE IDENTITY RECORD** | same |

🔑 **THE ARGUMENT IN `I4` SURVIVES — none of the twelve is an intro field.** 🛑 **BUT ITS PREMISE DOES NOT:**
*"that is a new federated world-readable field"* is **not established**, because `G-6a` is a namespace, not a
field. ⇒ **§4.** ⚠️ **AND THE KICKOFF'S OWN CORRECTION TO THIS CLAIM WAS ITSELF TOO NARROW** (it named six
fields, not twelve) — *the recurring species one layer deeper, inside the sentence written to fix it.*

---

## §3 — 🔓 THE OPEN DECISION: WHAT THE INTRO CARRIES. **JOE'S, UNRESOLVED, NAMED AS THE WIRE.**

🔒 **THIS IS ROUTED UNDER `D-123`'s HELD-HARDEST CLAUSE, VERIFIED AT `DECISIONS.md:4602`:** *"anything
touching **identity, the wire, or an irreversible act** goes to Joe UNRESOLVED and NAMED, even when it arrives
dressed as a technical detail."* **It is the wire, and it is named as the wire.** 📌 *This routing was
measured against the rule, not recalled — the J-731 finding was three over-routings in one document, all
three justified by urgencies that were Chat's own errors.*

🔑 **AND THE FORK IS THE MILESTONE'S IDENTITY:** `M-RP-INTRO` is FILED as an **RP (UI) milestone** and
**stops being one** the moment the intro carries anything but a plain string.

### The four options — `D-121`, three lenses, in rank order

**⚠️ LENS ② (TIER CONSEQUENCE) IS *NO TIER CONSEQUENCE* FOR ALL FOUR, STATED ONCE AND NOT MANUFACTURED.**
No option touches crypto-shred (`D-093`), a T4 durability floor, whose-tier-governs, or one party's
erasure-fate imposed on another. **A DM's content tier is settled by the Space, not by the shape of its first
message.** *A manufactured tier rationale is as bad as a manufactured UX one.*

#### **(a) — plain `message.text`. The intro is an ordinary first message.**

- **① user-visible:** the intro is a sentence the sender wrote, rendered identically by every client, forever.
  Nothing degrades because nothing is conditional. **Also: nothing is gained over what a user could type.**
- **③ resource:** ~zero. **And the milestone is then nearly empty** — which is an honest outcome, not a
  failure, but it should be named as one before it is chosen.

#### **(b) — a new `message.intro` event type.**

- **① user-visible — and `G-5` INVERTS THE USUAL ASSUMPTION:** an older or third-party client does **not**
  reject it. It **stores and relays it and never applies it** ⇒ 🛑 **the recipient sees NOTHING AT ALL, and
  no error is raised anywhere.** *A DM whose opening message is invisible to part of the network, silently.*
  On a federated protocol whose whole thesis is that you know who you are talking to, **an opening message
  that some peers cannot see is the worst of the four outcomes.**
- **③ resource:** enum variant + hand-written `as_str` + hand-written `from_str` (`G-1a`) · `apply_event` arm
  (`G-5`) · **a new Tauri command, a new resident outbound path and a new builder (`G-7a` — the cost nobody
  had priced)** · **ch3 spec — and in a protocol project the spec IS the deliverable** · node validation.
  ⚠️ **AND IT WOULD GATE ON THE ROUND-2 WHOLE-CODEBASE AUDIT**, which still gates UI completion.

#### **(c) — structured data inside the `text` string.**

- **① user-visible:** any client that does not parse the convention renders **your markup as literal text**.
- **③ resource:** zero build, **permanent parse contract**.
- 🛑 **`D-143`'s SHAPE, NAMED: the cheap route is unsound.** It puts a parse contract on the wire **while
  pretending not to**, and it leaves no version, no namespace and no way to tell an intro from a message that
  merely looks like one. **Chat does not recommend it and records it only so the fork is a partition.**

#### **(d) — `message.text` PLUS AN ADDITIVE CONTENT KEY. 🔑 VISIBLE ONLY BECAUSE `G-4` / `G-4b` FELL.**

The event stays `message.text`; `content` carries `text` **and** a namespaced key alongside it.

- **① user-visible:** every client renders the plain `text` and **ignores what it does not know** (`G-4b`,
  measured). ⇒ **graceful degradation instead of silence.** A client that *does* know renders the rich form
  — chosen **by the receiver, from a widget it already has** (`G-9`), so **`I3` / `N-172` is satisfied
  structurally rather than by discipline.**
- **③ resource:** small — a producer change and one reader. **Additive-optional is this protocol's own
  established pattern**, not an invention: `TransportMessage::AuthOk` documents it verbatim (*"Additive
  optional (Ch3 §3.0.3): old nodes omit it, old clients ignore it — no `protocol_version` bump"*,
  `xgen-core/src/wire/types.rs`). ⚠️ **It still needs `G-7a`'s seam widened**, which is real work — smaller
  than (b)'s, larger than zero.
- ⚠️ **AND THE HONEST OBJECTION, FLAGGED RATHER THAN RELIED ON:** (d) **is still the protocol plane**, and it
  puts a parse contract on the wire — *which is exactly what (c) is condemned for.* **The difference is that
  (d)'s contract is additive, namespaced and versionable, and (c)'s is smuggled into a display string.**
  **That distinction is real but it is thin**, and if Joe reads it as a distinction without a difference,
  **(a) is the correct answer and this file says so plainly.**

### 🔓 CHAT'S RECOMMENDATION — **(d)**, with **(a)** as the named fallback

**(d)** keeps `text` load-bearing so **every** client shows something true, and puts the rich form where
`N-172`'s rule already governs it. **(b) is the option that looks most architecturally correct and degrades
worst** — `G-5` is the whole reason, and it was not knowable before this pass. **(a) is not a failure state:**
if Joe wants `M-RP-INTRO` to remain a UI milestone and stay out of the protocol plane entirely, (a) is the
option that does that, and the milestone should then be scoped small and honestly.

🛑 **THIS IS A PROPOSAL, NOT A DECISION** (`D-123` §③). **Nothing is built until Joe rules.**

---

## §4 — 🔓 ROUTED TO JOE, AND DELIBERATELY **OUT OF SCOPE**: the published intro

`G-6a` — **`IdentityRecord.trust_assertion: Option<serde_json::Value>`, a free-form JSON value on a
federated, world-readable identity record.** It means ROADMAP's *"a published intro would be a new federated
world-readable field"* is **not established**: it may be a namespace inside an existing one.

🔒 **CHAT MAKES NO PROPOSAL HERE AND WILL NOT.** This is **identity AND the wire — both held-hardest axes of
`D-123`** — and it is a **different feature** from the one this milestone is scoped to. It is recorded so
that ① the ROADMAP claim stops being asserted as measured, and ② nobody discovers it mid-implementation and
treats it as a licence. ⚠️ **It must not ride `M-RP-INTRO` as a rider.** If it is ever wanted it takes its
own node and its own Phase-0.

---

## §5 — RECORD CORRECTIONS OWED (Chat's seat — no ruling required)

| # | record | correction |
|---|---|---|
| **R-1** | `docs/ROADMAP.md` `M-RP-INTRO` `Owes:` | *"`IdentityRecord` carries `display_name` and `is_ai` and nothing else"* → **twelve fields (`G-6`)**; the argument survives, the sentence does not. Annotate at the site (`D-131`), do not rewrite history |
| **R-2** | `ui/docs/xgen-ui-notes.md` `N-172` socket table | **three sockets → four**: add `stream-panel.above` / `dm-intro` (`G-9a`) |
| **R-3** | this file, §2 | 🔒 **the kickoff's wire claims are corrected HERE and nowhere else yet** — they were never written into a canonical record, so there is nothing else to annotate. *Recorded explicitly so a future reader does not go looking for a correction that was never owed* |

📌 **`R-1` and `R-2` are annotations to existing records, not rewrites** (`D-131`). They travel with this
milestone's first commit under `D-074` — **JOURNAL + CLAUDE.md + ROADMAP + this file in ONE commit.**

---

## §6 — PROPOSED LEGS (Chat's seat under `D-123`; the split is Chat's, the CONTENT of Leg 1 is Joe's)

⚠️ **NO LEG IS OPENED AND NO RUNBOOK IS AUTHORED UNTIL §3 IS RULED.** The leg list below is **conditional on
(d) or (a)**; **if Joe rules (b), this split is void and the milestone is re-scoped as protocol work behind
the Round-2 audit.**

| leg | what it does | state |
|---|---|---|
| **0** | 🔓 **Joe rules §3.** No code. **This leg is the gate** | 🟡 PENDING |
| **1** | the intro's **wording and appearance** — Joe's (`D-123`, `D-138`: mechanism Chat's, **values Joe's**, **the scaffold ships with plausible values and never blanks**) | 🟡 PENDING, gated on 0 |
| **2** | the send path: whatever §3 rules, produced at first send (`G-7`) | 🟡 PENDING, gated on 0 |
| **3** | the render path: the `bodyExtras` (or `details`) tenant, receiver-chosen (`G-9`, `I3`) | 🟡 PENDING, gated on 0 |
| **4** | live verify, two identities, Chat re-drives every gate (Rule 5) | 🟡 PENDING |
| **5** | records + close (`D-074`) | 🟡 PENDING |

🛑 **EVERY ROW CARRIES A STATE, AND THAT IS DELIBERATE.** `M-RP-MEMBER-ACT`'s §6 leg table had **two states
across eight rows** while four of those legs had shipped — `F1` at J-730 ⇒ ***the document that owned the
milestone's acceptance could not be read to tell you whether it was accepted.*** **This table is a state
board or it is not a leg table.**

🛑 **AND THE `F2` PRE-EMPT: THIS FILE OWNS THE CLOSE, AND NO OTHER DOCUMENT MAY CLAIM IT.** Leg 5 is the
close. Any runbook authored under this milestone **cites this row and does not restate it** — the same work
written twice, in two documents neither citing the other, is the species that bit `M-RP-MEMBER-ACT` **four
times in one arc**.

🛑 **`F9`'s PASS IS BINDING ON EVERY RUNBOOK THIS MILESTONE PRODUCES:** *can each gate be RUN, in the order
written, from the seat that owns it?* — and **a control captured on a pre-change build is a gate on the
CHANGE, so it belongs in the IMPLEMENTER's kickoff**, not in the verifier's.

---

## §7 — WHAT THIS MILESTONE MUST NOT DO

1. 🛑 **MUST NOT put a `WidgetMount`, a `widgetId`, or any receiver-side render instruction on the wire**
   (`I3` / `N-172`). **The sender sends DATA. The receiver picks the widget.** `W-13`'s unknown-id drop
   protects a client from a widget it does **not** have; it protects it from **nothing** inside a widget it
   does.
2. 🛑 **MUST NOT render the intro as system chrome** (`I1`, `D-113` S-5). Ruled at J-701 by argument; not
   re-opened here and not re-openable by a runbook.
3. 🛑 **MUST NOT open `{@html}` or a sanitiser surface.** `dm-intro.svelte` is a **component, not a processed
   string**, precisely because `name` is wire data authored by a person you have never met
   (`M_RP_MEMBER_ACT_LEG_C_BIS.md` §5.8, `N-032`). ⚠️ **An intro is the same object with a longer string.**
   Rich rendering that needs markup belongs to **`M-RP-PROCESSOR-RENDER`**, which is fifth in a Joe-locked
   sequence and is *"the one milestone here that must not be scoped in a single sitting"*.
4. 🛑 **MUST NOT merge with `dm-intro`'s draft page** (`G-8a`). Pre-send affordance and post-send artefact
   are two things wearing one word.
5. 🛑 **MUST NOT absorb `M-INTRO-POLICY`.** That milestone triggers on *"M-RP-INTRO lands"* and is
   **protocol + node + client, explicitly NOT a UI leg**. **This milestone constrains its shape and does not
   pre-empt it.** ⚠️ Its own locked finding stands: **the filter is enforced in the CLIENT, not the node**,
   because after PG-05 the node holds ciphertext and a node-side filter ships with a known expiry date
   (`D-143`).
6. 🛑 **MUST NOT reserve anything for a decision not yet taken** (`N-182`): no unfed key, no blank prop, no
   `intro: null`. **A key nothing writes is a key nobody has round-tripped.**

---

## §8 — OPEN ITEMS CARRIED, NOT THIS MILESTONE'S

📌 Recorded so they are not rediscovered as findings, and so none of them is quietly absorbed:

- 🔒 **Round-2 whole-codebase audit still gates UI completion.** **If §3 lands on (b), `M-RP-INTRO` gates on
  it too.**
- 🛑 **`OWED-2` (DM to an erased identity) and `OWED-3` (the partial first send)** —
  `M_RP_MEMBER_ACT_LEG_C_BIS.md:89` / `:104`, ticked as *re-sited to the retention milestone* — **and that
  milestone HAS NO NODE.** ⚠️ **The one-sided-gate shape, second instance.** `OWED-3` is
  **INTRO-adjacent**: a partial first send is precisely the path an intro would ride. **It is not taken here,
  and it is not silently inherited either.**
- 🛑 **`OQ5` item 3 (cross-node invite discovery) has NO HOME AT ALL.**
- 🔓 **Still Joe's, now SCAFFOLDED rather than blank (`D-138`, `5222e64`):** `skin.css` values · the DM row
  label wording (`N-192`) · `dm-intro`'s wording.
- 🛑 **`ROADMAP.md`'s `R-2` parent/child derivation is enforced NOWHERE** — `roadmap-format-gate.ps1` does
  **not** check indentation, and two tree lines were mis-indented for an unknown span while the gate passed.
  **Check depth by eye before any ROADMAP commit.**

---

## §9 — RECOMMENDED: CLAIR'S ADVERSARIAL READ, POINTED AT §3 FIRST AND COLD

🔑 **THE EVIDENCE IS UNAMBIGUOUS AND IT IS ABOUT CHAT, NOT ABOUT CLAIR.** Across `M-RP-MEMBER-ACT` Leg E:
**four gate defects, zero build defects, all four Chat's** — and at the E-5 Phase-0 Clair found **three
defects in Chat's own findings**, two of which had been routed to Joe on urgencies that were Chat's errors.
🛑 **Chat's own re-reads passed every single time.** *Every real defect in this arc came from outside the
text — Joe's recall, Clair reading, or the live client. None came from a re-read.*

**The read is recommended with a specific shape:**

1. **§3 FIRST, COLD, BEFORE §1 OR §2 CAN FRAME IT.** The fork is the decision; a reader who has absorbed
   Chat's grounding will inherit Chat's framing of the options along with it.
2. **Then §2 against source.** Every `G-` row cites a file and a line. **The question is not "is this
   plausible" but "does that line say this."** ⚠️ **Ask of every probe: what would this read return if the
   code were RIGHT?** Same answer ⇒ the probe is wrong (`N-194`). Two of Chat's own probes returned **false
   absences** in the previous session alone.
3. **Then §3's partition.** 🛑 **Is (a)/(b)/(c)/(d) a PARTITION or merely a CENSUS?** *A census is not a
   partition* — it has been the finding **twice in this arc**, and **option (d) exists only because a
   previous census was incomplete.** **A fifth option is the most likely defect in this file.**
4. **Then §6/§7 for the `F9` pass** and for anything reserved that §7.6 forbids.

📌 **Standing Clair up is Joe's.**

---

## §10 — DoD

**`M-RP-INTRO` is DONE when, and only when:**

1. §3 is **Joe-ruled** and the ruling is recorded in `DECISIONS.md` **if it is a `D`** (Joe mints the number
   and the wording), or in this file with a 🔒 if it is not.
2. Legs per §6, each verified by **Chat re-driving every gate independently** (Rule 5) — **numbers Chat did
   not personally measure do not enter this record.**
3. Every verification gate **names its surface in scope before the leg opens** — the `§8b` rule from
   `M_RP_MEMBERS.md`: *any DoD item saying "observed" / "exercised" / "driven" / "measured" MUST name its
   surface, and before locking a leg list, walk every 🔒 and ask WHICH LEG BUILDS THIS.*
4. **No phase-limit note survives the leg that lifts its limit** (`N-109`): **when a leg ships a disclosure,
   its REMOVAL enters the DoD of the leg that lifts the limit, in the same edit that adds it.**
5. `R-1` and `R-2` are annotated at their sites.
6. `roadmap-format-gate.ps1` returns **exit 0**, and the ROADMAP node's `Owes:` is **CUT to what is still
   owed, pointing at the record** (J-715) — 🛑 **a closing commit REDUCES a node; `:320` reached 25,396 chars
   by being appended to at close. DO NOT REGROW IT.**
7. **Floors re-measured, not inherited**, and **each stated with the screen it was measured on** — svelte-check
   at minimum; **the catalogue only if its harness is located and driven**; **`cargo` only if `.rs` is
   touched, and never as a scope argument.**

🛑 **"Commit pushed" IS NOT A DoD ITEM** — it is unflippable inside the commit that performs the push.
`Status: COMPLETED` in this file's header is the canonical signal.
