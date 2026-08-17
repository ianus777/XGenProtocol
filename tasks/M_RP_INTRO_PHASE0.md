# M-RP-INTRO Phase-0 — the DM welcome intro: the surface was ruled, the payload never was
> **Status**: COMPLETED  
> Version: 1.6  
> Date: Aug 2026  
> **Last updated**: 2026-08-17  
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
| **G-4c** | 🔑 **BOTH `content` AND `meta_atts` ARE INSIDE `EVENT_FIELD_ORDER` ⇒ BOTH ARE CANONICALISED, SIGNED, AND CONTRIBUTE TO `event_id`** | `xgen-common/src/canonical.rs:26-36`; nested objects sorted lexicographically and recursively, `:13` | ✅ **There is NO integrity gap on either surface.** An additive key is covered by the signature wherever it lands — which is why the choice between them is a *semantic* question, not a security one |
| **G-4d** | 🔑 **`meta_atts` ALREADY HAS A NAMESPACE SPEC AND `content` HAS NONE** — `xgen.*` reserved for protocol, third-party keys reverse-domain, lowercase snake_case, max 128 chars, **and "values are strings; structured values are JSON-encoded strings, NOT nested objects"** | spec 3.1.3, `CLAUDE.md:1241-1249`; live precedent `xgen.room_temperature` / `xgen.member_temperature` at `xgen-common/src/wire.rs:617`, `:621`, asserted by `meta_atts_keys_are_reserved_xgen_namespace` (`types.rs:1766`) | 🛑 **THE ROW THAT OPENS §3.1.** `meta_atts` is a *governed* surface; `content` is a *free* one |
| **G-4e** | ⚠️ **`meta_atts` IS `None` AT EVERY PRODUCER SITE MEASURED** — 26 hits repo-wide, and every construction site writes `None` or `"{}"` | `state_machine.rs:271`, `events_pipe.rs:276`/`:369`, `fanout.rs:2047`, `wire.rs:522`, `admin_ops.rs:626`/`:4141`/`:4634`, `audit.rs` (a SQLite column, a different object) | ⇒ **the reserved temperature keys are DECLARED and, at these sites, never written.** An intro would be **the first thing the client ever puts in `meta_atts`** — an unfed branch becoming fed (`N-091`), which is a cost to name, not a blocker |
| **G-4f** | ✅ **THE PARTITION IS CLOSED BY PREDICATE, NOT BY SUSPECT LIST: `Event` HAS EXACTLY TWO FREE-FORM `Value` FIELDS AND NO THIRD.** Walked all ten fields: eight are `String` / `Vec` / flavour-typed XGID / `EventType`; only `content` and `meta_atts` are `Value` | `xgen-common/src/wire.rs:475-493` | 🔑 **§3.1's d1-vs-d2 IS A PARTITION AND NOT A CENSUS.** *Checked because §9 named a missing option as this file's likeliest defect and §3.1 had just proved that concern correct one level down — not because a third surface was suspected.* 🛑 `event_id` and `signature` are excluded from `EVENT_FIELD_ORDER` **by design** (they are derived FROM the canonical bytes) and are not additive surfaces |
| **G-4g** | 🔑 **AN UNKNOWN KEY INSIDE `content` IS CANONICALISED AND SIGNED — PROVEN AT THE FUNCTION, NOT ASSUMED FROM `G-4c`.** `canonical_event_json` walks `EVENT_FIELD_ORDER` **at the top level only**, then hands each field to `canonical_value`, which **recurses and sorts EVERY object key lexicographically** — it has no allowlist and cannot skip a key it does not recognise | `xgen-common/src/canonical.rs:62-77` (`canonical_event_json`) → `:80-95` (`canonical_value`); consumed by `canonical_event_bytes` `:40`, which `verify_event_signature` uses (`xgen-core/src/space/state.rs:1316`, `:1338`) | ✅ **THE PROPERTY (d1) RESTS ON, AND IT IS STRONGER THAN "IT SURVIVES": AN INTERMEDIARY THAT STRIPPED THE INTRO KEY WOULD BREAK THE SIGNATURE.** The additive key is not merely tolerated in transit — **removing it is DETECTABLE TAMPERING.** ⚠️ *`G-4c` established that `content` is signed; it did NOT establish that an unrecognised key inside it is. That is a different claim and it needed its own read* |
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
| **G-9b-corr** | 🛑 **ANNOTATION AT THE SITE (`D-131`, 2026-08-15, from Clair's runbook read) — `G-9`'s REGISTRY LINE NUMBER WAS WRONG HERE AND WRONG AGAIN IN THE RUNBOOK.** This file cited the `bodyExtras` registry at **`:143`**; runbook v1.0 then cited **`:135`**. **MEASURED: `const widgets = { 'send-status': SendStatus };` is at `:145`.** 🔑 **THREE DISTINCT WRONG NUMBERS FOR ONE LINE ACROSS TWO DOCUMENTS** — the line was never re-opened; each document re-typed it from the previous one. ***A pointer copied forward is not a pointer measured**, and this is the arc's species reduced to its smallest possible form: a single integer.* ✅ The surrounding CLAIM (one object literal, one tenant, `W-13` drops unknown ids) is CONFIRMED and unaffected | `stream-panel.svelte:145` |
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

  > 🛑 **ANNOTATION AT THE SITE (`D-145`, J-750, 2026-08-17) — THE CLAUSE ABOVE IS FALSE, AND THIS FILE IS WHERE IT WAS MINTED. IT IS ALSO NOT A MERE STALENESS: IT MUTATED THE CLAIM IT CAME FROM.** ✅ **MEASURED — THREE Round-2 documents exist and ALL THREE ARE TERMINAL:** `tasks/archive/ROUND_2_AUDIT.md` **COMPLETE v1.3** (Pass 1, J-267, 2026-06-05, verdict GO) · `tasks/archive/ROUND_2_CHECKPOINT_AUDIT.md` **COMPLETED v1.0** (J-357, 2026-06-12, GO) · `tasks/archive/ROUND_2_FINAL_GATE_AUDIT.md` **COMPLETED v1.1** (Pass 2, J-390, 2026-06-17, GO). 🔑 **EVERY RECORD CALLS IT A *PRE-UI* GATE — it gates UI's START, and J-390 closes with *"the pre-UI chain is fully discharged; next-active = UI"*.** ⇒ ***"gates UI COMPLETION" is a STRONGER AND DIFFERENT claim that no record has ever made***, and it was minted here, carried a 🔒, and propagated into nine session kickoffs. **`docs/ROADMAP.md:290` was correct the whole time.** *Superseded text kept above per `D-145`.*

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

🔒 **THIS IS A PROPOSAL, NOT A DECISION** (`D-123` §③). **Nothing is built until Joe rules.**

### 🔒 **RULED — (d). JOE, 2026-08-15.**

**The intro ships as a `message.text` event whose `content` carries the human-readable `text` AND an additive
namespaced key alongside it.** Options **(a)**, **(b)** and **(c)** are refused.

🔑 **WHAT THE RULING BUYS, RESTATED SO A RUNBOOK CANNOT DRIFT FROM IT:** the plain `text` stays
**load-bearing**, so **every** client — old, third-party, or one that has never heard of an intro — renders
something true. The rich form is **additive and ignorable** (`G-4b`), and it is **chosen by the receiver from
a widget it already has** (`G-9`), so **`I3` / `N-172` holds structurally rather than by discipline.**

⚠️ **AND WHAT IT COSTS, KEPT IN THE RECORD RATHER THAN LOST AT THE MOMENT OF AGREEMENT:** (d) **is the
protocol plane.** `M-RP-INTRO` was FILED as an RP (UI) milestone and **this ruling ends that.** The seam at
`G-7a` — `send_message(space_id, room_id, text)`, one `String` wide end to end — **must widen**, which means
`desktop.rs`, the resident's outbound path, and `build_message_text_event`. **That is real Rust work in a
milestone filed as UI**, it is smaller than **(b)**'s and it is not zero. 🛑 **A runbook that treats this as a
frontend leg is reading the wrong milestone.**

🔓 **A `D` MAY BE OWED AND THE NUMBER IS JOE'S TO MINT.** This ruling touches the wire and establishes a
**precedent for every future additive payload**, not just this one — which is exactly the shape that earns a
`D` rather than a 🔒 in a task file. **Recorded here with a 🔒 in the meantime** (§10 DoD item 1). *A ruling
that sets a convention and lives only in one milestone's task file is a convention that will be rediscovered
and re-argued.*

⚠️ **THE PARTITION QUESTION WAS NOT ACADEMIC, AND §3.1 IS THE PROOF.** §9 flagged that the fork might be a
census rather than a partition and named a fifth option as the likeliest defect in this file. **Within the
hour, grounding the ruled option found an un-presented sub-fork INSIDE it** — §3.1. 🛑 **The ruling above stands
and is not re-opened; the option set it chose from was, on the measured evidence, incomplete at a level
below the one presented.** *The species is the arc's own, in Chat's own document, inside the option Chat
recommended.* **§9's cold read is therefore MORE owed, not less.**

---

## §3.1 — 🔓 THE SUB-FORK INSIDE (d): **WHICH SURFACE CARRIES THE KEY.** **JOE'S, UNRESOLVED, NAMED AS THE WIRE.**

🛑 **§3 SAID "AN ADDITIVE CONTENT KEY" AND THEREBY COLLAPSED TWO SURFACES INTO ONE PHRASE.** `G-4` named
**both** `content` and `meta_atts`; the option text named only the first. **The ruling was taken on a
description narrower than the thing it describes** — and that is the defect this arc keeps producing, so it
is surfaced rather than silently resolved by whoever writes the runbook.

✅ **WHAT IS SETTLED AND DOES NOT NEED RULING:** `G-4c` — **both surfaces are canonicalised and signed and
both contribute to `event_id`.** There is **no integrity difference**, and no security argument decides this.
**Both degrade identically** on a client that does not know the key. ⇒ **① user-visible impact is IDENTICAL
across d1 and d2**, stated plainly rather than manufactured. **② no tier consequence**, for the same reason
given in §3.

**③ resource cost, and the semantics, are the only axes that separate them.**

#### **(d1) — the key lives in `content`.**

- **Semantics:** an intro's payload **is the message's content**; `text` is its fallback rendering. **The two
  belong to one object.**
- **Nested objects are native** — no encoding trick.
- 🛑 **COST: `content` HAS NO NAMESPACE RULES AT ALL (`G-4d`)** ⇒ this milestone would be **inventing a
  content-key convention**, which is a **ch3 addition and an architectural precedent**, not a local choice.

#### **(d2) — the key lives in `meta_atts`.**

- ✅ **The namespace convention ALREADY EXISTS and is already asserted by a test (`G-4d`)** — `xgen.intro`
  needs no new rule, and `xgen.room_temperature` is the live precedent.
- 🛑 **COST 1, AND IT IS THE `D-143` SHAPE ONE LAYER DOWN:** spec 3.1.3 says **values are STRINGS, and
  structured values are JSON-ENCODED STRINGS, not nested objects.** ⇒ a structured intro is **JSON inside a
  JSON string** — a parse contract wrapped in an encoding trick. ⚠️ *It is **declared** rather than smuggled,
  which is what separates it from (c); but the resemblance is close enough that it must be seen, not
  discovered.*
- ⚠️ **COST 2:** `meta_atts` is *metadata about an event*. **An intro is not metadata about a message; it is
  the message.** Putting it there is a category choice, made once, that every later payload inherits.
- ⚠️ **COST 3 (`G-4e`):** it would be **the first thing the client ever writes into `meta_atts`.** Not a
  blocker — named so it is priced, not discovered.

### 🔓 CHAT'S RECOMMENDATION — **(d1)**, and the reason is the one that outranks convenience

**(d2) is cheaper today and wrong tomorrow.** Its whole advantage is that a namespace rule already exists —
but that rule was written for **string-valued annotations**, and paying for it with **JSON-in-a-string** buys
the convention by breaking the thing the convention is for. **(d1) costs a new ch3 convention and gets the
semantics right**: the intro is content, `text` is its fallback, and the two live in one object.

🔒 **AND IF (d1) IS TAKEN, THE CONVENTION SHOULD BORROW `meta_atts`' RULES RATHER THAN INVENT NEW ONES** —
`xgen.*` reserved, reverse-domain for third parties, lowercase snake_case. *A second namespace grammar in one
event is how `D-122`'s vocabulary drift starts.* **Chat proposes this; it is part of the same ruling and not
a separate one.**

🛑 **PROPOSAL, NOT DECISION.** ⚠️ **AND THE KEY'S NAME IS NOT DECIDED BY EITHER OPTION** — naming is Joe's
outright (`D-123`), and this file proposes none.

### 🔒 **§3.1 RULED — (d1). JOE, 2026-08-15, *"as you recommend"*.**

**The additive key lives in `content`, alongside `text`.** `meta_atts` is **not** the carrier. 🔒 **And the
recommendation was taken WHOLE, so its rider is part of the lock: the new content-key convention BORROWS
`meta_atts`' grammar rather than inventing a second one** — `xgen.*` reserved for protocol, reverse-domain
for third parties, lowercase snake_case segments, dots as separators. 🔑 **It borrows the GRAMMAR and NOT the
value rule:** spec 3.1.3's *"values are strings, structured values are JSON-encoded strings"* is the
constraint that disqualified (d2) and it **does not travel** — in `content`, **nested objects are native.**
*Carrying it across would import the defect while leaving the benefit behind.*

✅ **THE PARTITION WAS CLOSED BEFORE THE LOCK WAS RECORDED, NOT AFTER — `G-4f`.** §9 named a missing option
as this file's likeliest defect; §3.1 proved that concern correct one level down. So the same question was
asked again **by predicate over all ten `Event` fields rather than against a suspect list**: **exactly two are
free-form `Value`, and there is no third.** *A census is not a partition — twice in this arc — and this one
was made a partition on purpose.*

✅ **AND (d1) GAINED A PROPERTY IT WAS NOT ARGUED ON — `G-4g`.** `canonical_value` **recurses into `content`
and sorts every key with no allowlist**, so an unrecognised key is **inside the signed bytes**. ⇒ **an
intermediary that stripped the intro key would BREAK THE SIGNATURE.** The additive key is not merely
tolerated in transit — **removing it is detectable tampering.** ⚠️ *`G-4c` said `content` is signed; it did
not say an unknown key inside it is. Different claim, own read — and it landed in (d1)'s favour, which is
exactly when a claim most needs checking rather than least.*

🔓 **STILL OPEN AND STILL JOE'S: THE KEY'S NAME.** `D-123` puts naming on Joe's side outright and this file
proposes none. **Candidates are scaffolded below rather than left blank (`D-138`) — they are shapes to react
to, not a recommendation**, and Joe's own reason for the chosen name is recorded beside it when he gives one.

| candidate | shape | note |
|---|---|---|
| `xgen.intro` | flat, matches `xgen.room_temperature`'s form | shortest; says what it is |
| `xgen.card` | names the artefact, not the occasion | survives if intros are ever sent outside first contact |
| `xgen.intro.v1` | version in the key | ⚠️ **Chat's note, not a preference: a version in the KEY means v2 is a NEW key that old readers drop entirely**, where a version INSIDE the value degrades. The choice is Joe's; the consequence is stated so it is not discovered later |

🛑 **NO LEG OPENS AND NO RUNBOOK IS AUTHORED UNTIL THE NAME IS GIVEN** — leg 2 writes it onto the wire, and
**a name placed by Chat and later changed is a wire change, not a rename.**

### 🔒 **THE KEY IS NAMED — `xgen.intro.v1`. JOE, 2026-08-15.**

🔑 **JOE'S REASON, RECORDED WITH THE NAME BECAUSE IT IS LOAD-BEARING:** *"could we have one time in the
future another intro? maybe yes. so this `.v1` suffix is ok, mainly for future `.v2` or `.v3` case."* ⇒ **the
suffix is bought DELIBERATELY, for a successor that is expected rather than merely possible.**

✅ **GRAMMAR FIT, CHECKED AGAINST THE BORROWED RULE RATHER THAN EYEBALLED:** three lowercase segments, dots
as separators, no hyphens, 13 chars against a 128 limit, `xgen.` reserved prefix. **Conforms.**

🛑 **AND THIS CORRECTS CHAT'S OWN WARNING, WHICH WAS NARROWER THAN THE THING IT DESCRIBED — THE ARC'S SPECIES,
IN THE NOTE WRITTEN TO PREVENT A LATER DISCOVERY.** The candidate table above says a version in the key means
*"v2 is a NEW key that old readers drop entirely"*. **True of the KEY. FALSE of what the user sees** — and the
sentence reads as though the intro vanishes. ✅ **Under (d), `text` is always present and load-bearing**
(`G-4b`), so a v1-only client meeting a **v2** intro drops the unknown key and **still renders the plain
sentence.** ⇒ **the degradation is rich → plain, NEVER rich → nothing.** *The warning was written before the
candidate list was reconciled against the ruling two sections above it — and the row is kept unedited, with
this correction beneath it, because rewriting it would hide that Joe chose under a bleaker description than
the truth.* 🔑 **Joe's choice is SAFER than Chat's own note implied.**

🔒 **THE CONSEQUENCE THAT IS NOW BINDING, AND IT IS THE PRICE OF KEY-VERSIONING:** ***`text` MUST REMAIN
LOAD-BEARING FOR EVERY FUTURE VERSION OF THIS KEY, FOREVER.*** It is the ONLY thing that makes a versioned
key degrade instead of disappear. 🛑 **A future `xgen.intro.v2` that moves the human-readable sentence OUT of
`text` and into the key silently converts every older reader's experience from "plainer" to "empty"** — and
nothing anywhere would report it (`G-5`'s shape, arrived at from the opposite direction). **Written into this
file now so that the milestone which mints `v2` inherits it as a constraint rather than rediscovering it as a
defect** (`N-109`: the leg that lifts a limit owns the sweep).

🔓 **NOT DECIDED HERE, AND NAMED SO IT IS NOT ASSUMED:** whether a future `v2` sender ALSO writes `v1`
alongside it. That is the successor milestone's question, it costs nothing to leave open, and the `text`
fallback means it is an optimisation rather than a correctness gate.

✅ **LEG 0-ter DISCHARGED. EVERY GATE IN §3 / §3.1 IS NOW RULED.**

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

✅ **EVERY GATE IS RULED — §3 (d) · §3.1 (d1) · the key `xgen.intro.v1`.** The leg split below **survives all
three rulings unchanged**. 🛑 **A RUNBOOK MAY NOW BE AUTHORED**, and §9's cold read is recommended against it
before any code (`F9`: *can each gate be RUN, in the order written, from the seat that owns it?*).

| leg | what it does | state |
|---|---|---|
| **0** | 🔓 **Joe rules §3.** No code. **This leg is the gate** | ✅ **DONE 2026-08-15 — (d)** |
| **0-bis** | 🔓 **Joe rules §3.1 (d1 vs d2)** | ✅ **DONE 2026-08-15 — (d1), with `meta_atts`' grammar borrowed and its value rule NOT** |
| **0-ter** | 🔓 **Joe NAMES the key** | ✅ **DONE 2026-08-15 — `xgen.intro.v1`, versioned deliberately for an expected successor** |
| **1** | the intro's **wording and appearance** — Joe's (`D-123`, `D-138`: mechanism Chat's, **values Joe's**, **the scaffold ships with plausible values and never blanks**) | 🟡 PENDING — **runbook authorable** |
| **2** | 🛑 **the WIRE seam — REAL RUST, and it was invisible while the milestone was filed as UI.** Widen `send_message` (`G-7a`), the resident outbound path and `build_message_text_event`, **plus the ch3 content-key convention** (d1) | 🟡 PENDING — **runbook authorable** |
| **3** | the render path: the `bodyExtras` (or `details`) tenant, receiver-chosen (`G-9`, `I3`) | 🟡 PENDING — **runbook authorable** |
| **4** | live verify, two identities, Chat re-drives every gate (Rule 5) | 🟡 PENDING |
| **5** | records + close (`D-074`) | ✅ **DONE — J-735** |

🛑 **LEG 2 IS NOT THE LEG THIS MILESTONE WAS FILED WITH.** v1.0's row 2 read *"the send path: whatever §3
rules"* — true, and **narrower than the thing it describes**: §3's ruling makes it Rust in three files.
**Rewritten at the site rather than left to be discovered by whoever opens the runbook.**

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

  > 🛑 **ANNOTATION AT THE SITE (`D-145`, J-750, 2026-08-17): FALSE — see the annotation at `:166`. All three Round-2 documents are terminal and GO (J-267 · J-357 · J-390); the gate is PRE-UI and was discharged before UI began.**
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

1. §3 is **Joe-ruled** — ✅ **(d), 2026-08-15** — **§3.1 is Joe-ruled** — ✅ **(d1), 2026-08-15** — **and the key
   is NAMED** — ✅ **`xgen.intro.v1`, 2026-08-15.** The ruling is recorded in `DECISIONS.md` **if it is a `D`**
   (Joe mints the number and the wording), or in this file with a 🔒 if it is not. 🔓 **Chat's reading is that
   (d)+(d1) EARNS a `D`** — it establishes **the project's first content-key namespace convention**, binding
   on every future additive payload and not on this milestone alone.
1-bis. 🔒 **`text` REMAINS LOAD-BEARING** — no leg, and no future version of `xgen.intro.*`, moves the
   human-readable sentence out of `content.text`. **This is what makes the versioned key degrade rather than
   disappear**, and it is a DoD item because it is invisible until the moment it is violated.
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

---

## §11 — ✅ CLOSED (J-735, 2026-08-15)

**All five legs done.** The three gates left open at J-734 — `V-11` (the sender sees their own intro),
`V-12 ③` (two mounts on one row) and `V-7` (Joe's eyes) — were driven on a **real DM between two
identities**, with `status: accepted` and a real `event_id`. Runbook **v1.4 COMPLETED** carries the figures;
they are not restated here.

### 🔒 WHAT THIS PHASE-0 GOT RIGHT, AND THE ONE THING IT DID NOT

✅ **§2.3's central claim held exactly as written** — *"the render half is solved and the milestone is not
about it"*. The socket needed no new mechanism, `W-13` dropped unknown ids, and the degradation path the
whole **(d)** ruling was chosen for was **tested rather than argued**.

🛑 **§2.3 SAID *"the `bodyExtras` (or `details`) tenant"* AND NEVER ASKED WHICH SHAPE THE LANE WAS FOR.**
`bodyExtras` was built by `M-RP6.9` as the **reactions/tags lane** — *"adding the 4th tag does not move the
row"*, fixtures drawn as `👍 3 · 🎉 1 · 🚀 2`. A 600-character blurb is a **prose block**, and the two are
incompatible shapes. **The runbook picked one of the two options this file offered, and neither document
asked the question. That is Chat's.** ⇒ **placement ships PROVISIONAL, discharger `M-RP-INTRO-CANVAS`.**
*A provisional that points nowhere is a defect with an alibi.*

🔑 **AND THE PAYLOAD QUESTION THIS FILE EXISTS FOR TURNS OUT TO HAVE HAD A SECOND HALF.** §0 says the
surface was ruled and *"the payload never was"* — true. **What neither this file nor J-701 asked is WHERE
THE PAYLOAD IS AUTHORED.** Joe's recall at J-735: *"i needed to have some settable intro canvas … it has to
be settable through settings. it is plugin, am i right?"* — **he is right, and `N-172` had already named
"a DM welcome intro" as a canvas tenant in the very conversation this Phase-0 audits.** What shipped is
**hand-typed per DM**; what was intended is **composed once in Settings**. They compose (Settings holds the
default, the composer overrides), and **the wire contract is unchanged by either** — which is why the close
stands and `M-RP-INTRO-CANVAS` builds on this floor rather than replacing it.

### 📌 CARRIED OUT OF THIS MILESTONE
- 🟡 **`M-RP-INTRO-CANVAS` — the settable welcome canvas: intro as a settings-hosted plugin** (name Joe's).
  `host: client · delivery: compiled · surface: none` (`D-112`); `settingsComponent` on `D-120`'s **shipped**
  mechanism; two mounts (`S-2`). 🔒 **Bound by `N-172`: the wire carries DATA, never a widget id, never
  markup — the receiver picks the widget.** ⚠️ **No `{@html}`** (§7.3): the payload is authored by someone
  the recipient has never met.
- 🟡 **`M-INTRO-POLICY` — trigger FIRED on this close.** Protocol + node + client, **not** a UI leg; `D-143`
  stands. **Phase-0 OWED.** *A trigger that has fired with no Phase-0 is a defect — that is how this
  milestone started.*
- 🔓 **Joe's, still open:** the DM-draft-only asymmetry (Clair's) · `HEADLINE_MAX 120` / `BLURB_MAX 600`
  (`D-138`, provisional) · the `blurb` → `about` rename (**37 sites / 6 files** (case-insensitive LINES — state the metric; other metrics give 28/37/50) — **corrected J-738, Clair F-2: the 35/5 OMITTED `ui/assets/skin.css`, and that file is the FAILURE MODE — `.message-intro-blurb` is the rule matching the class `message-intro.svelte:68` emits, so renaming the field without the selector reverts the blurb to the browser default 16px, the exact defect J-735 fixed. It also crosses a seat boundary: `skin.css` is Joe`s.**; **ch3 unaffected** — it names the
  key, never the fields) · `trust_assertion` · `N-197`'s wording.
- 🔒 **Round-2 whole-codebase audit still GATES UI COMPLETION**, and bites harder now that **(d)** made this
  protocol-plane.

  > 🛑 **ANNOTATION AT THE SITE (`D-145`, J-750, 2026-08-17): FALSE — see the annotation at `:166`. The gate is PRE-UI, closed GO at J-390, and this file is the origin of the *"UI COMPLETION"* mutation.**
