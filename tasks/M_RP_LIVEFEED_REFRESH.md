# M-RP-LIVEFEED-REFRESH — the live event router behind the members and rooms panels
> **Status**: ACTIVE  
> Version: 1.15  
> Date: Jul 2026  
> **Last updated**: 2026-08-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

**This is the Phase-0 for the one mechanism that keeps client panels current after they first load.** It exists because two panels were measured frozen on the same day, for the same reason, with the same events already arriving unread.

🔑 **THE ORIGIN — JOE'S, AND IT IS A WIDENING, NOT ONE OF THE FOUR OPTIONS.** `M_RP_MEMBERS.md` §8a put four options for the frozen R7 roster. Joe took none of them: *"we have to build it properly and in this way that the mechanism will use also for rooms panel … if we will have such mechanism, it will have sense to use it intensively."* That is **§8a option (A) with a second consumer**, and it is recorded as its own decision rather than folded into A, so no later reader concludes Joe picked A.

**IT IS NOT** a general event bus, a subscription registry, or a plugin surface. It is a **routing seam with two named, measured consumers**. A third consumer is a future amendment, not a design goal (§4).

**IT IS NOT** a replacement for the cold-start fill. §4 draws that boundary and it is load-bearing.

---

## §1 — Grounding (measured 2026-07-26 at `0466cb2`, HEAD = origin, tree clean)

| Fact | Code | Measured result |
|---|---|---|
| the roster's only writers | `app_client.svelte:173/:176/:183/:187` | all four inside `loadMembers` |
| `loadMembers`' only caller | `app_client.svelte:169` | the `$effect` on `roomLatch.effectiveSpaceId` (`:168`) |
| the spaces/rooms tree's only writer | `app_client.svelte:567` | **`setSpaces` has exactly ONE caller in the entire codebase**, inside the startup block |
| the spaces store's surface | `ui/common/lib/stores/spaces-state.svelte.ts` (53 lines) | **one getter + `setSpaces` (whole-list replace).** No delta setters |
| the address-book store's surface | `ui/common/lib/stores/address-book.svelte.ts` (145 lines) | `setInflight · setResult · setFailed · reset`. No `addMember`, no `removeMember` |
| what arrives live | `app_client.svelte:551-552` — `xgen-event` → `ingest.push` | **R5's store only.** Nothing bridges `ingest` to any other store |
| `membership` in the shell | grep, `app_client.svelte` | **zero occurrences** |
| the events that would refresh both | `xgen-common/src/wire.rs:37-58` | `StateSpaceCreate · StateDmSpaceCreate · StateRoomCreate · StateRoomUpdate · StateSpaceUpdate` + the **eight** `Membership*` variants |
| do those events actually reach the client | `fanout.rs:244` ← `app.rs:2020` | **YES.** `apply_fanout` runs once per accepted locally-submitted event **before any event-type branch** and is generic over the event; the only `event_type` match inside it is `derive_event_nodes`, the observer *filter* dimension |
| the connection lifecycle | `self-state.svelte.ts:94/:106`, fed by `xgen-client-state-changed` | **live and reactive** — `READY` / `DISCONNECTED` observable in the frontend. This is §5's hook |

⇒ **THE ROOMS CASE IS WORSE THAN THE MEMBERS CASE.** Members reload on a Space change. **Spaces and rooms reload only on application restart.** The rooms panel is the navigation surface, so this is arguably the larger user-visible defect of the two.

📌 **AND THE CODE ALREADY SAID SO.** `app_client.svelte:564-565` reads *"Static per session (no live push until the resident, M-RP6.6)."* **M-RP6.6 — client resident: live connection + traffic accounting — CLOSED at J-543.** The precondition expired and nobody returned to the comment. **This is a defect class, not an instance:** a deferral written as a code comment has no owner and no trigger. Filed at §9.

---

## §2 — 🔒 DECISION 1: THE ROUTING SHAPE — LOCKED (Chat, DELEGATED by Joe 2026-07-26)

⚠️ **PROVENANCE: DELEGATED** — *"it is on you, my knowledge is limited"*. Recorded as delegated, **not** as a walked architectural lock. 📌 **Why delegation is safe here and was not safe at `M_RP_MEMBERS.md` §6:** this decision has **zero user-visible surface**. Nothing about it reaches a screen. It is invisible plumbing, and if it is wrong the cost is readability, not a lie told to a user. **Re-open freely.**

**Two shapes were on the table:**

**(i) ONE SHELL-OWNED ROUTER — LOCKED.** A single `$effect`-free handler in `app_client.svelte`, hung on the **existing** `xgen-event` listener (`:551`), that switches on event type and calls the owning store's setter.
- Matches the pattern already in place three times over — the roomLatch feeder, the gaps feeder and the members feeder are all shell-owned writers over pure `$state` stores (W-3: `$common` imports no shell dependency).
- **"Which code handled this event" stays answerable by reading one function.**
- Cost: adding a consumer touches the router. With two consumers that is correct; it is the thing to revisit at four.

**(ii) EVERY STORE EXPOSES `apply(event)`, SHELL FORWARDS EVERYTHING TO EVERYONE — REJECTED.** Cheaper to extend, but every store then has to know the whole event vocabulary, and a bug becomes a hunt across N stores. It also invites a store to quietly start handling an event nobody assigned to it.

🔒 **AND THE SINGLE-WRITER RULE CARRIES OVER.** The router calls the same setters the fill calls. It does **not** get privileged setters of its own, and it does **not** write `ingest`. R5 keeps `ingest` (`:552`) unchanged.

---

## §3 — 🔒 DECISION 2: THE MILESTONE NAME — LOCKED (Joe, 2026-07-26)

🔒 **`M-RP-LIVEFEED-REFRESH` (Joe, 2026-07-26).** The milestone-naming rule stands: **the identifier never appears bare, always with a short descriptive title.** The title carried here is *"the live event router behind the members and rooms panels"* — chosen so the identifier and the title do not both say *refresh*. 🔓 **The title alone remains adjustable** (taxonomy is Joe's); the identifier is locked.

⚠️ **ONE RESERVATION, RECORDED NOT ARGUED (Chat, 2026-07-26).** *Feed* already has a meaning in this client: `ingest` is the message feed (R5's store), and §2 explicitly leaves it untouched. A future reader may read `LIVEFEED` as *the message stream*, which is the one thing this milestone is not. **The `-REFRESH` half is the accurate half.** Raised before the name was applied, not after; renaming later costs one file move plus the references in §7 and `M_RP_MEMBERS.md` §8a.

---

## §4 — 🔒 THE DELTA-VS-FILL BOUNDARY — the rule that keeps the mechanism honest

🔒 **AN EVENT MAY CARRY A CHANGE. AN EVENT MAY NOT BE TRUSTED TO CARRY A WHOLE STATE.**

Joe's *"it will have sense to use it intensively"* is right about frequency and must be bounded on **scope**. Once a live seam exists the pull is to route everything through it. The rule:

- **A panel whose correctness needs the full picture still needs a fill.** The router handles deltas onto an already-filled store.
- **The router mutates the SAME store the fill populates — never a parallel copy.** Two sources of truth for one view is D-067, and when they disagree nothing tells you which is right.
- **A store with no fill has no business on the router.** The fill is what makes a delta meaningful.

📌 **Consequence, stated plainly:** this mechanism can never make the cold-start fill unnecessary. Anyone proposing to delete a fill because "the events keep it current" has misread this section.

---

## §5 — 🔓 DECISION 3: THE RECONNECT RULE — OPEN, JOE'S (structure)

🔑 **THE HOLE THIS MECHANISM CREATES, AND THE STRONGEST ARGUMENT FOR BUILDING IT ONCE.** While the connection is down, **no events arrive at all**. Every change that happens in that window is lost — permanently, from the client's point of view. On reconnect the panel is silently wrong, with no signal, which is the exact failure `M_RP_MEMBERS.md` §3 forbids: *"staleness and absence BOTH render as UNKNOWN, never as fine."*

📌 **This hole exists under §8a option (A) too.** Widening to two consumers does not create it — it means solving it **once** instead of twice.

**The hook is measured and exists:** `selfState.connection` (`self-state.svelte.ts:94`) is live and reactive, driven by `xgen-client-state-changed` → `setConnection` (`:106`), with `READY` and `DISCONNECTED` among the eleven enumerated lifecycle states.

### The three options — D-121 three lenses, per option

**(R1) RE-FILL EVERY LIVE CONSUMER ON A TRANSITION INTO `READY`. — Chat proposes.**
- **① User-visible:** the panels are correct again within one fill of coming back. The user sees a brief resolve, then truth.
- **② Tier:** neutral for members. For spaces/rooms it re-reads the local node; no new copy, no new erasure surface.
- **③ Resource:** small — one `$effect` on `selfState.connection`, reusing the fill paths that already exist. Fires on every reconnect, including flapping ones; a flap guard may be wanted and is a later amendment, not a v1 requirement.

**(R2) MARK THE PANELS STALE ON DISCONNECT AND LET THE USER REFRESH.**
- **① User-visible:** honest, but it hands the user a chore the client could have done itself.
- **② Tier:** neutral.
- **③ Resource:** **more than R1, and in Joe's lane** — a stale marker is new appearance work on both panels, plus a refresh affordance. Same shape as §8a option (B)'s hidden cost.

**(R3) DO NOTHING; ACCEPT POST-RECONNECT DRIFT.**
- **① User-visible:** the panel lies quietly, exactly as it does today, but now only after a disconnect — **rarer and therefore harder to notice or report.**
- **② Tier:** neutral.
- **③ Resource:** free, and it forfeits the milestone's reason for existing.

**CHAT PROPOSES (R4 for the stream + R1 for the panels).** ⚠️ *v1.0–v1.14 read "🔓 **RULING: OPEN.** ⚠️ Leg C does not open until it lands"; superseded 2026-08-02, kept not erased (`D-131`).*

🔒 **RULED 2026-08-02 (Joe, J-658) — BUT ONLY HALF OF IT, AND THE HALVES ARE NAMED SO THE OTHER IS NOT READ AS SETTLED.**

- 🔒 **R1 IS LOCKED, FOR THE PANELS.** A re-fill of every live consumer on the transition into `READY`. ⇒ **Leg C is UNGATED and may open.**
- 🔓 **R4 IS STILL OPEN, FOR THE MESSAGE STREAM.** Sync-from-cursor on reconnect, replayed through the router. **It was deliberately NOT bundled** — different consumer, different question, and it carries its own honest cost (a persisted sync cursor, §5a). **Nothing in Leg C may assume it.**

🔑 **WHY THE SPLIT IS REAL AND NOT BOOKKEEPING — §5a ALREADY SAID SO:** *the message stream needs the missed EVENTS* (order and content matter) ⇒ replay; *the panels need current STATE, not history* ⇒ re-fill. ✅ **The ruling takes the re-fill half and leaves the replay half exactly where §5a put it.**

🛑 **AND R1 WAS RULED IN COMPANY WITH A CHILD MILESTONE'S §6, BECAUSE IT DOES NOT STAND ALONE THERE — SEE `M_RP_IDENTITY_RESOLUTION.md` §6b (N-168).** That milestone's G-B failure case is **a long session in ONE Space that never disconnects**, and R1 fires only on a *reconnect* ⇒ ***a trigger that fires only on an exceptional condition cannot discharge a promise made in the ordinary one.*** **R1 is correct and it is not sufficient there**; its sibling ruling (§7's Tier-1 fetch on join) covers the ordinary path. 📌 **Recorded here because R1 is THIS document's decision and a reader must not infer from it that G-B is closed.**

---

### 🔑 §5a — THE BLACKOUT IS BIDIRECTIONAL, AND THE CATCH-UP MECHANISM ALREADY EXISTS (Joe's challenge, 2026-07-26)

**Joe asked: are the user's own events recorded during the outage, and can we resynchronise messages rather than only refresh panels?** Both halves were measured; **the first answer corrects this document, the second one enlarges it.**

🔑 **(1) NOTHING IS RECORDED. THE OUTAGE IS BIDIRECTIONAL, AND §5'S OPENING UNDERSTATED IT.** §5 said *"no events arrive"*. Measured at `resident.rs:705-727`: a send is **queued as intent, not as an event** — `OutboundRequest` carries `{space_id, room_id, text}`, and **the drain builds, signs and anchors the event at write time**, because the signing key and the frontier both live resident-side (D-067). ⇒ **while the session is down the request is never popped, the caller's own `SEND_QUEUE_TIMEOUT` fires, and the user's message never becomes an event at all.** There is **no persistent outbox**. The outcome returns honestly (`SendOutcome.status` = `timed_out` / `failed`, never a false `accepted` — D6), but the content is gone. **Not only does outside news stop arriving; the user's own actions do not happen either.**

🔑 **(2) THE CATCH-UP MECHANISM IS BUILT, TESTED, AND SIMPLY NEVER CALLED BY THE GUI.**

| Piece | Code | State |
|---|---|---|
| the request | `TransportMessage::SyncRequest { since, limit }` | exists |
| the node handler | `app.rs:1752-1784` — replies `HistoryBatch` + `SyncComplete { since, new_tip, continue_from }` | exists |
| the collector | `fanout.rs:478` `collect_sync_history` — **member-only**, paginated, cursor-resumable | exists |
| its tests | `fanout.rs:1303 / 1353 / 1609 / 1624 / 1642` — member-scoping, self-DM, page limit, resume-to-completion, empty-when-caught-up | **passing** |
| who calls it | `ai_service.rs:236`, `batch.rs:191` | **`resident.rs` NEVER issues one** |

⇒ **The GUI client is the one consumer with the catch-up capability available and unwired.** Resync does not need building. It needs **calling**.

⚠️ **THE ONE THING R4 GENUINELY ADDS, MEASURED:** **the GUI client does not persist a sync cursor.** No tip is stored in `resident.rs` (the `new_tip` hits are `ai_service.rs`, in-memory for one run, and node-side handshake/test code). Without a remembered position, `since=""` means **full replay**. Remembering where you were is small, but it is not free, and it is the honest cost of R4.

### (R4) SYNC FROM CURSOR ON RECONNECT, REPLAYED THROUGH THE ROUTER — for the MESSAGE STREAM
- **① User-visible:** the messages sent by others during the blackout actually appear, in order, in place. Not a refreshed panel — the conversation is whole again.
- **② Tier:** neutral. Replays events the user is already a member-scoped recipient of; `collect_sync_history` is member-only by construction and tested to be.
- **③ Resource:** the transport, handler, pagination and tests already exist. Adds a persisted cursor and the reconnect call. Pagination bounds a long outage.

🔑 **AND THE HONEST SPLIT — REPLAY AND RE-FILL ANSWER DIFFERENT QUESTIONS, SO DOING BOTH IS NOT REDUNDANT.** The **message stream needs the missed EVENTS** (order and content matter) ⇒ replay. The **panels need current STATE, not history** (nobody needs to watch Bob join and leave four times) ⇒ re-fill, which is exact and cheap. **Replaying ten thousand events to learn that one member left is the wrong tool**, and §4's rule already says so: a view whose correctness needs the full picture takes a fill.

### 🔓 §5b — THE BLACKOUT MARKER — JOE'S (appearance + structure)

**Joe: *"it is good that a data blackout we announce with system message."* Agreed, with one constraint that is not negotiable.**

🔒 **IT MUST BE CLIENT-LOCAL CHROME, NEVER A DAG EVENT.** Writing an event to represent *"this client's network was down"* would put one client's local condition into the shared append-only record — and it would **federate to everyone else**, for whom it is not true. Precedent for local chrome in the stream exists and is the shape to follow: the day divider and the jump-to-latest pill are both inline, unregistered, and carry no descriptor.

📌 **AND IT IS STRICTLY STRONGER THAN THE STATUS LED, WHICH ALREADY EXISTS.** The LED (`self-state.svelte.ts:94`, eleven enumerated states) tells you the connection state **now**. A marker in the stream tells you **where in the conversation the gap was** — which is the thing a reader needs afterwards, when the LED is green again and the hole is invisible.

🔓 **OPEN.** Copy, form and placement are Joe's. ⚠️ **SCOPE FLAG, RAISED NOT ABSORBED:** message resync and a stream marker are a **third concern**, not a third consumer of §2's router. Whether they ride this milestone or a sibling is Joe's sequencing call, and this document does not assume the answer.

---

🔓 **CARRIED TO A SIBLING MILESTONE (Joe, 2026-07-26): *"ok, we can split if it will better"*.** §5a and §5b describe **message resync and the blackout marker**, which are a third *concern*, not a third consumer of §2's router. They leave this milestone. **This document keeps: the router, the members consumer, the rooms consumer, and the panel half of the reconnect rule (R1).** 🔓 **The sibling's name is Joe's** — proposed working identifier `M-RP-RESYNC — message catch-up after a connection gap`. §5a/§5b stay written here **until that document exists**, then move rather than duplicate (D-067 applied to prose).

---

### 🔑 §5c — THERE IS NO MESSAGE CACHE. MEASURED, BECAUSE JOE BELIEVED THERE WAS ONE

**Joe: *"i thought we have message cache for such events"*. There is not, and the belief is reasonable — two things that look like one are in place.**

**What the client actually persists — four files, and none of them is a message:**

| File | Written by | Contents |
|---|---|---|
| `xgen-client_address_book.json` | `address_book.rs:52` | identity records — **a real cache, and the one Joe is probably thinking of** |
| `xgen-client_config.toml` | `app.rs:2464` | config |
| `xgen-client_keypair.enc` | `app.rs:2463` | the keypair |
| `xgen-client_uistate.json` | `desktop.rs:99` | layout / UI state |

⇒ **NOTHING ABOUT CONVERSATIONS IS ON DISK CLIENT-SIDE.**

**And the other lookalike:** `ui/common/lib/stores/ingest.svelte.ts` is `$state<IngestEvent[]>([])` — **in memory, capped at `INGEST_CAP = 500`, with a `dropped` counter so the UI can never quietly imply completeness (D5).** It is a **session window, not a cache.** It starts empty and dies with the process.

🔑 **THE LARGER FINDING, AND IT REFRAMES THIS WHOLE SECTION: THE BLACKOUT GAP IS A SPECIAL CASE OF A BIGGER ONE — THE CLIENT HAS NO MESSAGE HISTORY AT ALL.** `ingest` starts empty every run, and the startup block (`app_client.svelte:557-572`) fetches self-state, spaces, about-info, substitutions and the address book — **no history fetch of any kind.** ⇒ **every application start is a cold start with an empty conversation**, and the stream shows only what arrived while it was running. A disconnect is simply a *second* hole in a surface that already has a permanent one. ⚠️ **The sibling milestone's real job is therefore larger than "catch up after an outage"** — and `collect_sync_history`'s cursor pagination is exactly the mechanism for both, which is why they belong together.

---

### 🔒 §5d — SYSTEM EVENTS ARE THREAD ENTRIES THAT ARE LOCAL TO ONE USER (Joe, 2026-07-26) — PROMOTE TO `DECISIONS.md`

🔒 **Joe, first statement:** *"i take that every system event is client-local, which sometimes interprets dag events and otherwise internal states."*
🔒 **Joe, clarifying:** *"i meant more as event in the client's message thread. for include it as a proper documented stream events for the node, i think it is not good. cannot imagine such permanent cluster of internal events in public message stream. maybe we can take it as whatsapp 'delete just for user'."*

⇒ **The rule, in its clarified form:** a system event **renders as an entry in the message thread**, it is **local to one user**, and it is **never a node/DAG event** — never signed, anchored, federated, or written to the shared record. Its two sources are an *interpretation* of DAG events ("Bob joined") or an *internal client state* ("the connection was down here").

🔑 **THE WHATSAPP ANALOGY IS THE RIGHT ONE, AND IT SAYS MORE THAN IT LOOKS.** *Delete-for-me* establishes that **the rendered thread is a PER-USER PROJECTION of the shared record, not the shared record itself.** That is a stronger and more useful statement than "system events are local", because it licenses per-user divergence **as a design principle** rather than as an exception — and the reason a blackout marker is honest is exactly that: the outage was true for one client and false for the others, so a shared record has no business carrying it.

⚠️ **FOUR CONSEQUENCES, NAMED NOT ABSORBED. Each is structure, and structure is Joe's.**

1. **THE THREAD BECOMES A MERGE, NOT A PROJECTION OF ONE LIST.** R5 today projects `ingest` — a single source. Local entries make the rendered stream an **ordered merge of two sources**. That is a real change to `message-stream`, not an additive prop.
2. **ORDERING NEEDS A RULE, BECAUSE TWO CLOCKS MEET.** Local entries carry the client's local clock (§5e); DAG events carry their own. The blackout marker happens to order itself (it sits between the last event received and the first one after), but a general local-entry facility does not get that for free.
3. **THE ANALOGY IMPLIES PERSISTENCE THAT DOES NOT EXIST.** WhatsApp's delete-for-me **survives restart**. Ours cannot: per §5c the client persists no messages at all, so a blackout marker would vanish on restart along with the entire conversation. **Consistent today, and a debt the moment the sibling milestone gives messages a home.**
   - 🔒 **RESOLVED (Joe, 2026-07-26): NO KNOWN CONSUMER.** *"i cannot imagine what kind of local client event happens that needs to be saved for the next restart. maybe it can be some type, but i dont know now."* ⇒ **local entries are session-scoped, and v1 needs nothing more.**
   - 🔒 **BUT THE TWO ARE COUPLED, AND THIS IS THE RULE THAT FALLS OUT OF IT: local-entry persistence is decided WITH message persistence, never before it.** If messages ever survive restart and local entries do not, the reopened thread shows **a conversation with an unexplained hole** — the gap persists and the marker explaining it does not. **That is worse than persisting neither**, which is today's coherent state.
4. **ADDITIVE AND SUBTRACTIVE ARE THE SAME FACILITY, AND THIS IS THE USEFUL PART.** A system event is *additive* (something in your thread that is not in the record); delete-for-me is *subtractive* (something in the record that is not in your thread). Both are **per-user divergence from the shared record**, so a **local thread layer** built once serves both. 📌 **If delete-for-me is ever wanted, this is where it lands** — recorded now so it is not rediscovered as a surprise.

📌 **PROPOSED AS `D-130` — drafted, NOT allocated.** Highest existing entry is **D-129** (measured, 136 entries). **Draft title:** *"The rendered message thread is a per-user projection, not the shared record: system entries are local, never DAG events."* Chat drafts; Joe locks the number and the wording. 🔓 **OPEN.**

📌 **AND IT RETIRES A QUESTION BEFORE IT WAS ASKED:** with this rule, "should the blackout marker be an event?" has a general answer instead of a per-case one, and so will the next narration feature.

---

### ⚫ §5f-OPTIONS — THE REJECTED CACHE OPTION SPACE (superseded by §5f's C1 lock; kept so it is not re-derived)

**Joe: *"what about that message cache. i know i think about it but forgot to mention it earlier, i assume."*** ⇒ it was **intended and never specified**. §5c measured that **none exists**. Opened here so it is not lost; it is not decided here.

🔑 **THE FRAMING, AND IT IS THE WHOLE POINT: A CLIENT MESSAGE CACHE IS NOT A PERFORMANCE FEATURE. IT IS A KEY-MANAGEMENT AND ERASURE DECISION.** The user-visible half (faster starts, history, offline reading) is the easy half and everyone agrees on it. **D-121 lens ② is where this is actually decided**, and it fires harder here than anywhere else in the arc:

- **Does crypto-shred remain a real guarantee?** D-093 clause 1 — universal E2E, no protocol escrow, the node content-blind. **A cache of decrypted message text on a user's disk is a copy OUTSIDE the shredding scope.** Destroy every key and that copy still reads.
- **Is one party's erasure-fate silently imposed on another?** D-093 clause 3. **Another member's erasure request cannot reach your local cache.** The federated right-to-be-forgotten problem, which is already this project's hardest open tension, arrives on the client's own disk.

**The option space, stated so it is not re-derived — none of these is chosen here:**
- **(C1) NO CACHE — today.** Every start is cold; history comes from the node via `collect_sync_history`. Shred stays absolute. Costs a fetch on every start and forbids offline reading entirely.
- **(C2) CIPHERTEXT-ONLY CACHE, KEYS IN THE KEYSTORE.** Shred still bites: destroy the keys and the cached bytes are undecryptable, so the guarantee survives. **This is the only option that keeps D-093 clause 1 true while giving a cache at all**, and it mirrors the address book's existing stance (`address_book.rs:168` — a pure cache, delete-and-refetch).
- **(C3) PLAINTEXT CACHE.** Fastest, offline-capable, **and it breaks crypto-shred.** Recorded so the trade is explicit rather than arrived at.
- **(C4) BOUNDED CACHE** — last N per room and/or a TTL. Orthogonal: it caps exposure under any of C1–C3, it does not decide the question.

⚠️ **CHAT DOES NOT RECOMMEND YET, AND THE REASON IS A MEASUREMENT THAT IS OWED.** C2's viability depends on the **per-recipient content-key wrapping model**, which is already on Chat's carried-owed list as *"the live edge of the GDPR-vs-append-only tension"* and is **unmeasured**. Recommending C2 before measuring it would be a claim narrower than the thing it describes — the defect class this arc has already hit three times (§8b of `M_RP_MEMBERS.md`). **Chat measures the key model first; the recommendation follows it.**

📌 **THIS DESERVES ITS OWN PHASE-0, NOT A SUBSECTION.** It touches D-093, the erasure tension and the sibling milestone's history fetch at once. 🔓 **Scope and sequencing are Joe's.**

---

### 🔒 §5f — THE CLIENT MESSAGE CACHE — RESOLVED: NONE. THE CLIENT IS A READER-SENDER (Joe, 2026-07-26)

🔒 **Joe: *"i mean that outbox. dont want to caching messages. as we said, the client is just reader-sender. doesnt hold any users data."*** ⇒ **(C1) NO CACHE. LOCKED.** C2/C3/C4 below are recorded as the rejected option space, not as live alternatives.

🔑 **AND THE LOCK IS STRONGER THAN THE QUESTION IT ANSWERS.** §5f asked *should the client cache messages?* Joe answered a level up: **the client is a reader-sender and holds no user data.** That makes the message-cache answer a **consequence of a standing property**, not a milestone-local trade — and it means §5f's D-093 tension **never has to be adjudicated**, because the copy that would create it never exists. 📌 **The cleanest possible resolution of the hardest question in this arc: the tension is dissolved rather than balanced.**

⚠️ **ONE MEASURED EXCEPTION, SURFACED BECAUSE THE RECORD MUST NOT SAY WHAT THE DISK CONTRADICTS.** The client **does** persist third-party personal data today: `xgen-client_address_book.json` holds, per identity, `display_name · identity_id · home_node · is_ai · last_seen · revoked · trust_assertion` (`address_book.rs:78-111`). That is **other people's data, on the user's disk, outside their erasure reach** — the same shape as the cache concern, already shipped at smaller scale.
- 📌 **A BOUND IS DECLARED:** `T1_DEFAULT_RETENTION_DAYS = 182` (`address_book.rs:66`), explicitly distinguished from the tier renewal TTLs, which run the opposite direction. ⚠️ **This bullet originally read *"It is NOT unbounded"* — corrected: a declared bound is not an enforced one, see the next bullet.**
- ⚠️ **ANSWERED — DECLARED AND TESTED, NEVER WIRED (Chat, measured 2026-07-26 at `c44eb1d`).** `#[cfg(test)] mod tests` opens at `address_book.rs:335`. **Every** occurrence of `evict_older_than` and `T1_DEFAULT_RETENTION_DAYS` outside the declaration (`:66`) and the definition (`:285`) sits at **`:591 · :607 · :615 · :627 · :643` — all inside that test module.** A repo-wide search across every `.rs` in every crate (`.claude` excluded) returns **those seven lines and nothing else**. ⇒ **`evict_older_than` has ZERO production callers.** 🔑 **The 182-day retention is enforced by the test suite and by nothing that ever runs.** 📌 §5f's own sentence stands as written — *a retention policy that nothing enforces is a retention policy in name only* — and it is now **measured, not suspected.**
- 🔑 **AND IT STRENGTHENS H1 RATHER THAN COMPLICATING IT.** H1's lens ③ said the eviction question *"dies with"* the file. It does more than die: **there was never a live bound to lose.** The book on disk is today **unbounded in fact**, so H1 removes an actual unbounded at-rest store of third-party data, not a bounded one. ⚠️ **If H1 is NOT taken, wiring the eviction is not a tidy-up — it is the difference between the record being true and being false.**
- 🔓 **The reconciliation is Joe's:** either the reader-sender rule carries a named exception for identity records, or the address book is held to it. **Recorded, not resolved.**

---

### 🔒 §5g — THE OUTBOX — THIS IS THE THING JOE MEANT (Joe, 2026-07-26)

🔒 **Joe: *"if the messages successfully transfer into node after the blackout ends, cache can be purge from sent messages (not whole enblock)."*** ⇒ **a send-side queue with PER-ITEM purge on acknowledgement.** Distinct from §5f in both direction and risk, and separated here so no later reader collapses them.

🔑 **THE PURGE RULE MAPS ONTO MACHINERY THAT ALREADY EXISTS AND IS ALREADY HONEST.** `SendOutcome.status` (`resident.rs:729-740`) is a **four-way** outcome, not a boolean: `accepted` (node validated **and durably persisted**) · `rejected` (deterministic refusal, with the wire code) · `timed_out` (**genuinely ambiguous** — the node may hold it) · `failed` (never reached the wire).
🔒 **⇒ PURGE ON `accepted`. KEEP ON EVERYTHING ELSE.**

🔑 **AND JOE'S *"not whole enblock"* IS LOAD-BEARING, NOT A PREFERENCE.** A block purge would discard a `timed_out` item alongside an `accepted` one — **and `timed_out` is exactly the state where the user cannot be told what happened.** Per-item purge is what keeps the ambiguity attached to the item it belongs to.

⚠️ **ONE VISIBLE CONSEQUENCE, JOE'S TO CONFIRM.** Today the drain **builds, signs and anchors at write time** — key and frontier are both resident-side (D-067). A surviving outbox therefore stores **intent** (`{space_id, room_id, text}`) and lets the drain construct the event on reconnect, which preserves the single-anchoring-point rule. ⇒ **the message enters the DAG with a POST-RECONNECT anchor and timestamp, not the moment it was typed.** Honest, and probably wanted — but it is user-visible, so it is confirmed, not assumed.

🔑 **THE TIER LENS SEPARATES THE TWO CLEANLY, WHICH IS WHY THE OUTBOX SURVIVES THE LOCK THAT KILLED THE CACHE.** An outbox holds **the user's OWN composed text, on their OWN disk, for a BOUNDED window that ends at delivery.** A read cache would hold **other people's content, indefinitely, outside their erasure reach.** Not the same exposure — and the outbox does not violate *"holds no user data"*, because the data is the user's own and in transit.

🔓 **STILL OPEN — THE ONE JUDGEMENT CALL, AND IT IS ABOUT WHAT A PERSON SHOULD BE ASKED TO DEAL WITH:** are `timed_out` items **retried automatically** on the next reconnect (risking the duplicate the node may already hold), or **surfaced for the user to resend by hand** (ambiguity stays visible, work moves to the user)? 🔓 **Joe's.**

📌 **TRAVELS TO THE SIBLING MILESTONE, AND SHOULD NOT WAIT ON §5f'S GHOST.** The outbox is separable, far cheaper than a cache, and no longer blocked by any key-model measurement — that dependency died with C1.

---

### 🔓 §5h — CAN THE ADDRESS BOOK STOP BEING AT REST? (Joe, 2026-07-26) — TWO CHANGES, AND ONLY ONE NEEDS PROTOCOL

**Joe: *"those data are also temporary cached for performance. or can we just read this data every time when the client starts run from the node? maybe it can be a part of initial handshaking. client will send pubkeys (is it possible?) and node returns visit cards."***

🔑 **MEASURED FIRST: THERE IS NO IDENTITY LOOKUP VERB ON THE WIRE. NONE.** A search of `xgen-common/src/wire.rs` for any identity-fetch request returns **nothing**. Records are not looked up — they are **derived from the Space's DAG by a drain**: `fill_and_members` (`desktop.rs:644-697`) fills the book *"from one Space's live DAG"*, one Space at a time, under a `FillLock`. This independently confirms `M_RP_ADDRESS_BOOK.md` §5's conclusion that no `identity.*` event exists.

⇒ **Joe's *"is it possible?"* answered plainly: NOT TODAY.** *Send pubkeys, receive visit cards* is a **new protocol verb** — wire type, node handler, client caller.

🔑 **BUT THE TWO HALVES OF THE IDEA ARE SEPARABLE, AND THE PRIVACY HALF NEEDS NO PROTOCOL AT ALL.**

**(H1) DROP THE PERSISTENCE, KEEP THE BOOK IN MEMORY FOR THE SESSION.** No wire change, no node change. Delete `xgen-client_address_book.json`; the fill re-derives per Space as it already does.
- ① **User-visible: less than it looks.** `loadMembers` fires on **every** `effectiveSpaceId` change and calls `fill_space_records` **unconditionally** (`app_client.svelte:167-183`) — **the fill already runs every time a Space is entered, persisted book or not.** ⇒ the file is buying name resolution only in the window *before* the first fill returns, and across restarts. **Not nothing, but far less than "performance" suggests.**
- ② **Tier: this is the whole point.** Third-party personal data stops being **at rest**. §5f's exception disappears rather than being reconciled, and *"the client holds no user data"* becomes true on disk, not just in intent.
- ③ **Resource: small and subtractive.** Delete a file path, a load and a save. The `T1_DEFAULT_RETENTION_DAYS` eviction question dies with it — **including the unmeasured half of it**.

**(H2) ADD THE VISIT-CARD VERB.** Client sends the identity list it needs; node returns the records.
- ① **User-visible:** faster and more targeted than draining a whole Space DAG to learn a handful of names.
- ② **Tier — A NEW SURFACE THAT MUST BE SCOPED, NOT AN OPTIMISATION WITH NO COST.** *"Resolve these pubkeys"* tells the node exactly whom the asker is interested in. Inside the user's own Spaces the node learns nothing new (it hosts them). **Unscoped, it becomes an identity-probe oracle for people the asker shares nothing with.** ⇒ it must be **member-scoped by construction**, the way `collect_sync_history` already is (`fanout.rs:478`, member-only, tested at `fanout.rs:1303`).
- ③ **Resource: a real protocol change** — wire, node handler, client caller, tests. Not a client-side tidy-up.

🔑 **CHAT PROPOSES: H1 IS INDEPENDENT AND SHOULD NOT WAIT FOR H2.** H1 delivers the privacy result on its own, today, with a subtraction. H2 is an **optimisation of the fill**, not a prerequisite for it — and it carries a scoping requirement that deserves its own design pass rather than riding a UI arc.

🔓 **Both are Joe's; neither belongs to this milestone.** 📌 H1 fits the address-book arc; H2 is protocol and sits with the node work.

#### §5h-i — "DO WE HAVE TO CHANGE THE PROTOCOL FOR H2?" (Joe, 2026-07-26) — YES, AND THE CLASS OF CHANGE MATTERS

**Three places, and the third is the one that makes this heavier than an app feature:**
1. **The wire types** — `xgen-core/src/wire/types.rs:45` (`TransportMessage`).
2. **The node handler** — the `app.rs` inbound match, beside the `SyncRequest` arm.
3. **THE SPEC CHAPTER.** ⚠️ **In a protocol project the spec IS the deliverable.** A verb that exists in code and not in Ch3 is not part of the protocol — it is an implementation detail that federated peers cannot rely on.

🔑 **BUT THE EXTENSION DISCIPLINE ALREADY EXISTS, AND ONE PRECEDENT IS DIRECTLY ON POINT.** `AuthOk` gained the Node's own `node_id` as an **additive optional field** — *"old nodes omit it, old clients ignore it — no `protocol_version` bump"* (`xgen-core/src/wire/types.rs:68-71`, citing Ch3 §3.0.3). ⇒ **adding a FIELD is cheap and precedented.**

⚠️ **A NEW MESSAGE TYPE IS NOT THE SAME CLASS OF CHANGE.** An old node receiving an unknown request cannot answer it, so the additive-optional argument does not carry across.

🛑 **`NegotiatedCapabilities` MEASURED — IT CANNOT CARRY THIS, FOR TWO INDEPENDENT REASONS (Chat, 2026-07-26 at `c44eb1d`). THE PROPOSAL IS WITHDRAWN.**

⚠️ **AND THE POINTER ITSELF WAS WRONG.** §5h-i cited the shape as living at `xgen-node/src/app.rs:68`. **That line is an `use` statement.** The declaration is **`xgen-core/src/wire/types.rs:305`**. 📌 *An import is not a definition — the same species as the M-RP6.2/6.6 gate-versus-author collapse, caught here before it was built on.*

**The measured shape, in full — it is two scalar strings:**

```rust
pub struct NegotiatedCapabilities {
    pub serialisation: String,
    pub protocol_version: String,
}
```

1. ⚠️ **THERE IS NO FEATURE OR EXTENSION FIELD TO PUT A VERB BEHIND.** Two scalars, both single-valued. 🔑 **And the drop is deliberate, not an omission:** the *declared* side, `FederationCapabilities` (`types.rs:284-291`), **does** carry `extensions: Vec<String>` — and `handshake.rs:298` constructs the negotiated result from `serial` + `neg_version` **only**. ⇒ **extensions are DECLARED and never NEGOTIATED.** Nothing reads a negotiated extension because none survives the handshake.
2. 🛑 **IT IS THE WRONG LINK ENTIRELY. IT IS NODE↔NODE, AND H2 IS CLIENT↔NODE.** `NegotiatedCapabilities` appears only inside `FederationMessage::Capabilities` (`types.rs:344`) — the **federation** handshake between two Nodes. **H2's verb rides `TransportMessage`, the client session.** 📌 Every non-test occurrence is federation code (`handshake.rs:41/:298/:619`, `app.rs:68/:2253`); the remaining five are reconnect and federation-push tests.

🔑 **BUT THE CAPABILITY-FLAG PATTERN DOES EXIST IN THIS CODEBASE — ON A THIRD STRUCT.** `xgen-core/src/bootstrap/capability.rs` writes `"xgen.bootstrap"` into `capabilities.extensions` on a **`NodeAnnouncement`** (`node/announcement.rs:50`) at `:42-43` and reads it back at `:54` and `:107`. ⇒ **the discipline is precedented and working — just not on either the federation handshake or the client session.** ⚠️ **So H2 behind a flag is not impossible; it is UNBUILT on the link H2 needs**, and that is a different and larger statement than §5h-i's *"the obvious mechanism"*.

⇒ 🔓 **THE QUESTION H2 NOW INHERITS, AND IT IS JOE'S:** does the **client session** get a capability-negotiation surface at all? That is a protocol-shaped decision in its own right — **it would be the mechanism by which every future client verb ships**, not a detail of this one. 📌 **Chat proposes it be named and scoped separately from H2 rather than smuggled in as H2's enabling work.**

🔑 **AND THE REAL COST IS PROBABLY FEDERATION, NOT THE VERB.** *"Node returns visit cards"* only works if **that node holds the card.** Records today are derived from a Space DAG the node already hosts. A pubkey-keyed lookup for someone whose home Node is elsewhere is **a federated fetch**, with its own authorization, caching and failure semantics. ⇒ **the verb is the small half; deciding what happens when the answer lives on another Node is the milestone.**

🔒 **THE RESOURCE LENS IS RELAXED (Joe, 2026-07-26): *"i dont mind if we will need to add some cli or other procedural code. we will test it."*** ⇒ **build cost is NOT the objection to H2, and no option below should be trimmed to avoid writing code.** 📌 **The harness supports this and it is not a promise — `xgen-mptest` is a full multiparty rig (runner · injector · oracle · process · binloc, ~30 e2e tests), and Ch3 §3.0.3 already treats the **batch / `--aicontrol` JSONL wire as a protocol-shaped surface**, so a new verb has an obvious CLI binding and an obvious test home.

⚠️ **BUT THE THREE THINGS THAT MAKE H2 A DESIGN PASS DO NOT GET CHEAPER WITH WILLINGNESS TO BUILD.** Naming them so the relaxed cost lens is not mistaken for a green light:
1. 🔓 **THE SCOPE RULE IS THESIS-BEARING** (§5h-ii) — Joe's, and it is the whole decision.
2. 🔓 **FEDERATION SEMANTICS** — what happens when the card lives on another Node. Design, not typing.
3. ✅ **`NegotiatedCapabilities`' actual shape — MEASURED AND CLOSED (above). It cannot carry H2.** ⇒ replaced by a **larger** open item: 🔓 **whether the CLIENT session gets a capability-negotiation surface at all — Joe's, and scoped separately from H2.**

#### 🔓 §5h-ii — DOES H2 TOUCH THE FUNDAMENTAL THESIS? (Joe asked in those terms, 2026-07-26)

⚠️ **PROVENANCE OF THIS SECTION: CHAT'S READING OF THE THESIS FROM THE PROJECT RECORD, NOT A RE-READ OF Ch0–Ch2 AGAINST THE TEXT.** Recorded as a reading. 📌 **Chat owes the chapter read before any of this reaches a spec.**

**The verb itself: NO.** Verified identity is a first-class protocol primitive; a request that says *"here are pubkeys, return the verified cards"* is that primitive being **read**. It creates no identity, weakens none, and adds no way around one. 🔑 **If anything the CURRENT state is the anomaly: identity is foundational, and yet there is NO WAY TO ASK ABOUT AN IDENTITY** — records fall out of draining a Space DAG as a side effect. H2 makes the primitive addressable, which is *closer* to the thesis than today's code.

🔑 **THE SCOPE RULE: YES, AND THIS IS WHERE THE WHOLE ARGUMENT LIVES.** No-anonymity means **within a Space you know who you are talking to.** It does **not** mean **any registered identity is globally queryable by anyone holding a pubkey.** Two different claims. ⚠️ **An unscoped visit-card verb silently converts the first into the second — a federation-wide directory readable by anyone who can harvest or guess keys. That would be a thesis change made by accident, at the wire level.**

⇒ **THE VERB IS THESIS-NEUTRAL; THE SCOPE RULE ON IT IS THESIS-BEARING.** Member-scoped — *you may resolve identities you already share a Space with* — is the thesis implemented faithfully, and it is the shape `collect_sync_history` already uses. Unscoped is a different protocol wearing the same name.

📌 **AND IT IS THE RECURRING SHAPE OF THIS PROJECT'S HARD QUESTIONS.** No-anonymity vs institutional independence · the Discord bridge · right-to-be-forgotten in a federated append-only log — each is the same move: **a mechanism that is fine in itself, and a scope boundary carrying the entire argument.** H2 is a small member of that family, which is exactly why it earns its own design pass instead of riding inside a UI arc.

---

### 🔒 §5e — THE MARKER'S TIMESTAMP (Joe, 2026-07-26: *"yes we have always time stamp for that"*)

✓ Accepted. **One precision that is easy to get silently wrong:** the gap contains **no events**, so its bounds cannot come from event timestamps — there are none to read. They come from the **client's local clock** at the `READY`→`DISCONNECTED` transition and at the return to `READY` (`self-state.svelte.ts:106`). ⚠️ **A marker placed by event timestamp would land at the last message before the outage rather than at the outage**, which is off by exactly the length of the thing it is describing.

---

## §6 — Which events, which consumer

⚠️ **THE ROUTER BRANCHES ON A JSON STRING, NOT ON A RUST NAME.** `Event` carries `#[serde(rename = "type")]` (`xgen-common/src/wire.rs:476`), `desktop.rs:395` emits the whole `Event` verbatim, and `app_client.svelte:551` pushes it into `ingest` untouched ⇒ **the webview sees `payload.type` holding the wire string** (`"membership.join"`, not `MembershipJoin`). The first column below is therefore written as the wire string; the Rust variant is the same row's identity, not the thing the branch tests.

| Event — `payload.type` | Consumer store | ⚠️ Whose identity the row is about | Effect |
|---|---|---|---|
| `membership.join` | address-book | **`sender`** — the joiner signs their own join | add the member to the roster; resolve the name (book, else xgid tail) |
| `membership.leave` | address-book | **`sender`** — the leaver signs their own leave | remove the member from the roster |
| `membership.kick` · `membership.ban` | address-book | **`content.target_identity`** — ⚠️ `sender` is the **MODERATOR** | remove the member from the roster |
| `membership.node_eject` | address-book | **`content.target_identity`** — ⚠️ `sender` is the **NODE** | remove the member from the roster |
| `membership.invite` · `membership.mute` · `membership.node_unban` | — | — | **v1: ignored, deliberately.** None of the three changes who is in the room *now*. Recorded so the omission reads as a choice |
| `state.space_create` · `state.dm_space_create` | spaces-state | — (`space_id`) | add the Space to the tree |
| `state.room_create` | spaces-state | — (`space_id`; ⚠️ `room_id` is the EMPTY STRING — the event **is** the Room, `xgen-common/src/wire.rs:466-467`) | add the room under its Space |
| ~~`state.room_update` · `state.space_update`~~ | ~~spaces-state~~ | — | ⚠️ **SUPERSEDED AT v1.12 — THE ROW WAS FALSE.** It read *"update the existing entry in place"* through v1.11. **There is nothing to update in place.** Grounding and verdicts: **§6a ③** |
| `state.dm_promote` · `state.space_migrate` | **NONE — see §6a ④** | — | ⚠️ **NOT ROUTABLE, AND NOT AN OVERSIGHT.** Both appliers are real and both write a field whose *name* matches a panel field. **No code path joins the two objects** (§6-ii). Listed so a reader of §6 alone does not read their absence as a gap |

🛑 **§6-i — THE SUBJECT FIELD IS NOT UNIFORM ACROSS THE REMOVE ROWS, AND UNTIL v1.11 THIS TABLE COLLAPSED ALL FOUR INTO ONE (Leg 0 second-reader pass, Chat, 2026-07-29).** v1.10 read `MembershipLeave · Kick · Ban · NodeEject` as a single row — *"remove the member from the roster"* — and named **no field at all**. Grounded: `protocol_audit.rs:110-113` puts `identity_id` = `sender` for `leave`; `:121-135` puts `kicked_id` / `banned_id` = **`content.target_identity`** with `sender` recorded separately as `kicker_id` / `banner_id`; `admin_ops.rs:4207` builds `node_eject` as `{ "target_identity": … }` and `emit_node_membership_event` signs it with **`rt.node_keypair`** ⇒ its `sender` is the node. **A router reading `sender` uniformly removes the moderator, or the node, instead of the person who was removed** — and it would pass on `join` and `leave`, which are the first two things Leg D exercises. ⇒ **the defect would have shipped looking correct.** 🔑 **This is the recurring species — a claim narrower than the thing it describes — and it fell the moment `wire.rs` and the node emitters were OPENED, not on any re-read of this document.**

📌 **`target_identity` IS A CONVENTION, NOT A TYPE.** The only struct that declares it is `MembershipMuteContent` (`xgen-common/src/wire.rs:712-713`) — the one membership event this milestone **ignores**. `kick` / `ban` / `node_eject` build the field as raw `serde_json::json!`. ⇒ **the router must read it defensively and drop the event if it is absent**, because no compiler is holding the producers to it.

⚠️ **SCOPING IS A CORRECTNESS REQUIREMENT, NOT AN OPTIMISATION.** A `membership.*` event for a Space the user is not currently scoped to must not touch the roster on screen. The scope check is `roomLatch.effectiveSpaceId`, the same one the fill uses.

⚠️ **YOU NEVER RECEIVE YOUR OWN EVENT.** `fanout.rs:301-307` excludes the author **by identity**, and it is generic — it applies to `membership.join` exactly as it applies to a message (re-measured 2026-07-26). ⇒ **a roster built purely from the live channel would be missing you, invisibly.** This is why the self row is a fixture (`M_RP_MEMBERS.md` §4c), and it is now independently required by two unrelated arguments.

📌 **`spacesState` needs delta setters it does not have** — it is 53 lines with a single whole-list replace. That is a real, small addition, named here so Leg B's scope is not a surprise.

🛑 **§6-ii — THE PANEL DOES NOT READ THE OBJECT THE APPLIERS WRITE (Chat, measured 2026-07-31, at `86065ad`).** **This section exists because two rows of the table above were false in the same way, and the same mistake is available to anyone extending it.**

🔑 **`SpaceState` AND `KnownSpace` ARE DIFFERENT OBJECTS IN DIFFERENT CRATES. THEY SHARE A FIELD NAME AND NOTHING ELSE.**

| | `SpaceState` — what every applier writes | `KnownSpace` — what the panel renders |
|---|---|---|
| declared | `xgen-core/src/space/state.rs` | `xgen-common/src/state.rs:185-199` |
| written by | **every arm of the dispatch** | `xgen-client/src/ops.rs:647` · `:731` · `:953` — **the user's own three local actions, and nothing else** |
| reaches the panel | **never** | `ops::spaces` (`:246-250`) returns `state.spaces` **verbatim off disk**, its own doc calling it *"a zero-network local read"* → `get_spaces` → `spacesState.setSpaces` |

**Measured in both directions:** a census of every `state.spaces` mutation across all four crates (`.claude` and `target` excluded) returns **exactly three write sites**, all in `xgen-client/src/ops.rs`, all user actions. ⚠️ **The `runtime.rs` `node.spaces[…]` hits are the NODE's `SpaceState` map — different crate, different object, and every one of them is inside a test.**

⇒ **THE TWO USER-VISIBLE CONSEQUENCES ARE REAL; THE ROUTING EXPLANATION OF THEM IS NOT.**
- A promoted DM renders as *"DM with &lt;xgid&gt;"* indefinitely — **because `KnownSpace.name` is written once at DM-create (`ops.rs:953`) and by nothing afterwards**, not because an applier went unrouted.
- `node_endpoint` does not follow a migration — **because it is written from `home_node_url`, the client's own local config (`ops.rs:650` · `:956`)**, which was never the Space's home node in the first place.

🔑 **THE RECURRING SPECIES, CAUGHT INSIDE A SINGLE JOURNAL ENTRY.** `J-640` recorded *"the Spaces tree is not a view of shared state — it is the client's own ledger"* **one section below** a verdict asserting that two appliers reach the panel. **Both claims sat in the same entry and neither was read against the other.** It is the same shape as that entry's own `is_dm` finding — *not a live-routing gap; the fill re-derives it identically.* ⚠️ **Neither would have fallen to any re-read of this document. Both fell to opening `ops.rs`.**

🛑 **CONSEQUENCE FOR LEG B, NAMED HERE RATHER THAN DISCOVERED INSIDE IT:** the router writes a **frontend store**; `get_spaces` reads **disk**. ⇒ **any `state.*` routing is session-scoped by construction** — correct until restart, never true — **and that applies to every row, including the three genuine adds**, not only to the two struck above. 🔓 **This is the substance of the B1/B2/B3 scope question, and it is Joe's.**

---

### 🛑 §6a — THE `membership.*` HALF OF THE TABLE IS A PARTITION. THE `state.*` HALF IS NOT. (Leg 0 second-reader pass, Chat, 2026-07-29 · ⚠️ **FRAME WIDENED AND VERDICTS LANDED 2026-07-31 — SEE §6a-i**)

⚠️ **THE HEADING ABOVE IS KEPT AS WRITTEN AND IS NOW NARROWER THAN THE SECTION (`D-131` — annotate, never silently repair).** `state.*` was **not** the right predicate and the pass that ran on 2026-07-31 says so: `wire.rs::as_str()` carries **59 event strings across 11 namespaces** and §6 accounts for 26, so **33 events in nine namespaces were never inside this section's frame at all** — including the three `dm.*` siblings of the one event flagged here as a Spaces-tree suspect. 🔑 **The frame is now *every wire event that can mutate `KnownSpace`* — 17 rows, registered at §6a-i.** 📌 *The census below stands as measured; only its boundary moved.*

**Both halves were checked in the same direction and then in the reverse direction — not *do the names in §6 exist*, but *does §6 name everything the wire carries*. The two halves came back different.**

✅ **`membership.*` — COMPLETE.** `xgen-common/src/wire.rs` carries **exactly 8** `membership.*` strings (`invite · join · leave · kick · ban · node_eject · node_unban · mute`, declared L43-L58, mapped L180-L187, parsed L274-L281). §6 names **all 8** — 1 add, 4 remove, 3 deliberately ignored. ⇒ **Leg A's event surface is closed, and Leg A's runbook may open on it.**

🛑 **`state.*` — 5 OF 14. NINE ARE UNNAMED, AND §6 HAS NO "IGNORED" ROW FOR THEM.** The wire carries fourteen `state.`-prefixed strings; §6 names five. Unnamed: `state.federation_add` · `state.node_priority` · `state.dm_promote` · `state.space_migrate` · `state.ai_operator_delegate` · `state.ai_operator_revoke` · `state.space_pacing` · `state.space_temperature_visibility` · `state.mls_group_init`.

⚠️ **AND ONE OF THEM CANNOT BE FOUND BY READING RUST VARIANT NAMES AT ALL: `MlsGroupInit` → `"state.mls_group_init"`** (`wire.rs:232`). A branch written as `type.startsWith('state.')` catches it; a table written in variant names never sees it. 🔑 **The wire string and the variant name are different namespaces, and §6 was written in the wrong one** — the same root as the `payload.type` note above.

📌 **`state.dm_promote` and `state.space_migrate` are the two that plausibly change the Space tree**, which is Leg B's whole subject. The rest are policy, AI-operator and crypto events. **None of this is measured yet — what is measured is that the table does not account for them.**

⇒ ~~**§6a GATES LEG B's RUNBOOK, NOT LEG A's.** Leg B may not open until every `state.*` string has a row: consumed, or ignored with the reason written.~~ ⚠️ **SUPERSEDED AT v1.12 — THE GATE WAS TWO-WAY AND THE PASS RETURNED FIVE VERDICTS (`D-131`).** The replacement gate is at the end of §6a-i. 🔑 **A census is not a partition — second instance of that species in eight days, and the first one also survived every re-read of the document that carried it.**

---

### ✅ §6a-i — THE VERDICT REGISTER: 17 ROWS, MEASURED (Chat, 2026-07-31, at `86065ad`)

🔑 **THE TEST WAS MADE CONCRETE BEFORE ANY ROW WAS WRITTEN.** The Spaces panel renders **exactly eight fields** — `KnownSpace {space_id · name · node_endpoint · role · rooms}` plus `KnownRoom {room_id · name · joined}` (`ui/common/lib/stores/spaces-state.svelte.ts:20-32`, a **verbatim mirror** of `xgen-common/src/state.rs:185-199`). Every verdict below is the answer to one question: **does this event's applier write one of those eight?** Read out of `xgen-core/src/space/state.rs`. ⚠️ **`#[cfg(test)]` opens at `:1956` — every line cited below is production.**

🔑 **THE FRAME: `wire.rs::as_str()` carries 59 event strings across 11 namespaces. The ones that can touch a `KnownSpace` are `state.*` (14) and `dm.*` (3) — 17.** Both counts re-measured independently of `J-640`, and both reproduced exactly.

#### ① CONSUMED — 3 rows

| Wire string | Applier | Effect |
|---|---|---|
| `state.space_create` · `state.dm_space_create` | ⚠️ **no dispatch arm** — genesis guards at `:266` / `:346` / `:497`; the event **constructs** the `SpaceState` rather than mutating one | add the Space to the tree |
| `state.room_create` | `:601` → `apply_room_create` | add the room under its Space |

⚠️ **"Consumed" here means *the frontend store can express the change*, NOT that the panel's data source tracks it — see §6-ii. Even these three are session-scoped under B1.**

#### ② IGNORED — 7 rows, REVERSE-TESTED, NOT ASSUMED

**Each was checked by naming the field it actually writes. None is one of the eight.**

| Wire string | Dispatch | Applier writes | Panel field? |
|---|---|---|---|
| `state.federation_add` | `:602` | `self.federation_nodes` (`:713`) | no |
| `state.node_priority` | `:611` | `self.node_priority_order` (`:723`) | no |
| `state.space_pacing` | `:615` | `self.human_pacing_ms` · `self.ai_pacing_ms` (`:744` / `:745`) | no |
| `state.space_temperature_visibility` | `:617` | `self.member_temperature_visibility` (`:761`) | no |
| `state.ai_operator_delegate` | `:621` | `self.ai_operator_delegations` (`:1204`) | no |
| `state.ai_operator_revoke` | `:623` | `self.ai_operator_delegations` (`:1226`) | no |
| `state.mls_group_init` | `:645` | `room.mls_epoch` (`:837`) | no |

⚠️ **`state.mls_group_init`'s Rust variant is `MlsGroupInit` — it carries NO `State` prefix.** A table written in variant names never sees it; a branch on `type.startsWith('state.')` does. **The wire string and the variant name are different namespaces**, and this row is the proof.

#### ③ IGNORED (UNBUILT) + 📌 SPEC GAP — 2 rows *(the third verdict, Joe-locked 2026-07-31)*

🛑 **KEPT AS TWO ENTRIES, NOT ONE ROW.** §6 collapsed them and was false about both; collapsing them again — even correctly — would repeat §6-i's own defect on the same table. **They are unbuilt for different reasons.**

| Wire string | Dispatch | State of the code | 📌 Spec gap |
|---|---|---|---|
| `state.space_update` | `:630` — **literally `=> Ok(())`** | ⚠️ **no applier function exists.** Dispatch comment `:629`: *"remains the SR-F2 no-op (no content schema yet)"* | ⚠️ **SPEC-AHEAD-OF-CODE TOO, AND ITS ONLY REFERENCE IS THE PROMISE.** `docs/xgen_appendix_i_en.md:96` says *"Updates Space metadata"* against a dispatch of `=> Ok(())`. **No content schema has ever been written.** 📌 Corroborated by reference count across `docs/`: `state.space_update` = **1** (that line, and nothing else); the other thirteen `state.*` run **7 to 81** — measured 2026-07-31 |
| `state.room_update` | `:628` → `apply_room_update` (`:883`) | ⚠️ **a REAL applier that writes a REAL field** — `room.permission_overrides` (`:904`), read from `content["permission_overrides"]` (`:884`), early-`Ok` when the key is absent (`:886`). **The field it writes is simply not one of the eight.** Dispatch comment `:627`: *"Room name/topic content stays deferred"* | ⚠️ **`docs/xgen_appendix_i_en.md:95` promises this event carries *name* and *topic*. The applier carries neither.** In a protocol project the spec is the deliverable ⇒ **the SPEC IS AHEAD OF THE CODE**, and this is the row §6 leaned on |

#### ④ NO CONSUMER PATH — 2 rows *(new class; the finding of this pass)*

**The applier is real, writes a real field, and the panel still never sees it — because the panel does not read that object at all (§6-ii).**

| Wire string | Dispatch | Applier writes | Why it does not arrive |
|---|---|---|---|
| `state.dm_promote` | `:613` → `apply_dm_promote` (`:659`) | `self.name = Some(…)` (`:663`) ← `content["new_name"]`; also clears `dm_constraints_active` (`:664`) | writes **`SpaceState.name`**. The panel renders **`KnownSpace.name`**, written once at DM-create (`ops.rs:953`) and by nothing afterwards |
| `state.space_migrate` | `:641` → `apply_space_migrate` (`:1159`) | `self.home_node` (`:1174`) ← `content["destination_node_id"]`; idempotent (`:1167`), authority-gated (`:1171`) | writes **`SpaceState.home_node`**. The panel renders **`KnownSpace.node_endpoint`**, written from **`home_node_url` — the client's own local config** (`ops.rs:650` · `:956`) |

🛑 **THIS CLASS IS WHY ③ AND ④ ARE NOT THE SAME VERDICT.** ③ is *the code was never written*. ④ is *the code exists and is wired to a different object*. **Different owners, different fixes** — ③ is owed to whoever writes the content schema; ④ is owed to whoever decides whether the local ledger consumes events (the B1/B2/B3 question, §6-ii).

#### ⑤ NO ARM — 3 rows

| Wire string | Result |
|---|---|
| `dm.promote_propose` · `dm.promote_confirm` · `dm.promote_reject` | ✅ **ZERO arms in the Space state machine**, confirmed by direct search. They are the **negotiation** that culminates in `state.dm_promote`; none of them mutates `SpaceState` at all |

✅ **THE WIDENING CAME BACK CLEAN.** These three entered the frame only because the frame widened past `state.*` — and having entered it, they change nothing. **A negative result, recorded because it was measured rather than assumed.**

#### THE ARITHMETIC CLOSES

**3 + 7 + 2 + 2 + 3 = 17 = 14 `state.*` + 3 `dm.*`.** Every wire string that can reach a `KnownSpace` now carries a verdict and a line number.

🔒 **THE REPLACEMENT GATE: §6a-i DISCHARGES §6a's GATE ON LEG B's RUNBOOK.** Every row is consumed, ignored with its write site named, unbuilt with its spec gap named, pathless with its object named, or armless. ⚠️ **What the register does NOT discharge is §6-ii's consequence** — the router writes memory and `get_spaces` reads disk. 🔓 **Leg B's runbook may be AUTHORED; it may not be LOCKED until the B1/B2/B3 scope is ruled, because the three options produce three different runbooks.**

---

## §7 — Legs

**Leg 0 — Phase-0.** This document. ✅ **SECOND-READER PASS DONE 2026-07-29 (Chat), against `xgen-common/src/wire.rs` + the node emitters, in BOTH directions.** Three findings, all landed in §6 at v1.11: the `payload.type` namespace, §6-i's non-uniform subject field (**would have shipped a correct-looking router**), §6a's `state.*` partition gap. ✅ **CLASSIFICATION PASS DONE 2026-07-31 (Chat), frame widened on Joe's call to *every wire event that can mutate `KnownSpace`* — 17 rows, landed at v1.12 in §6a-i.** Two further findings: §6-ii (**the panel does not read the object the appliers write** — which corrected two verdicts and one of `J-640`'s own headline claims), and verdict class ④ **NO CONSUMER PATH**. **§7's precondition is discharged for Leg A. For Leg B it is discharged for AUTHORING and not for LOCKING** — see §6a-i's replacement gate. No code.

**Leg A — the router + the members consumer.** The `membership.*` branch on the existing `xgen-event` listener plus `addMember`/`removeMember` on the address-book store. **Surface: `ui/client/src/app_client.svelte`, `ui/common/lib/stores/address-book.svelte.ts`.** Frontend only; moves the **`svelte-check`** floor (baseline 0 err / 34 warn / 15 files). **Satisfies `M_RP_MEMBERS.md` §5 and unblocks its Leg C.**

✅ **LEG A DONE — IMPLEMENTED BY CLAIR, RE-DRIVEN BY CHAT, 2026-08-01 (J-643).** ⚠️ **THREE files, not two** — `members-panel.svelte` joined at §5-iii (runbook §2), and the runbook's own §6 still said *two* until v1.5 caught it. **Measured: 3 files, 103+/3−; only 3 lines removed in the whole leg; `svelte-check` `0 errors / 34 warnings / 15 files` = the floor EXACTLY; cargo not run.** 🔒 **R5 held — `ingest.push` verified byte-identical FROM THE DIFF** (space prefix, no `−`, no re-add), not from the file. **R1/R2/R3/R4 each asserted in code and each read back** — R2 in particular reads `content.target_identity` for `kick`/`ban`/`node_eject` **with no `sender` fallback**, which is the arm that would have shipped looking correct. 🛑 **§5-iii ①'s user-visible claim did NOT survive the re-drive — see runbook §5-iv:** `entity-avatar.svelte:125` is `data-ai={flags.isAi || undefined}`, so **`false` and absent render identically** and an AI joining live still appears human. **The store's data is honest now; the RENDERER collapses it, one layer below the fix.** 📌 **Clair implemented the lock exactly — the defect is in the lock.**

**Leg B — the spaces/rooms consumer.** The `state.*` branch plus delta setters on `spaces-state`. **Surface: `ui/client/src/app_client.svelte`, `ui/common/lib/stores/spaces-state.svelte.ts`.** Moves **`svelte-check`**. 📌 **A and B are split deliberately: the second consumer is what proves the seam rather than asserting it.** One commit spanning both makes a regression ambiguous.

**Leg C — the reconnect rule.** 🔒 **UNGATED — §5's R1 IS RULED (Joe, J-658).** ⚠️ *v1.0–v1.14 read "🔓 Gated on §5"; the trigger has FIRED and **a trigger that has fired is a defect**, so it moves in the same commit. Superseded, kept not erased (`D-131`).* **Surface: `ui/client/src/app_client.svelte`** (an `$effect` on `selfState.connection`). 🔓 **R4 is NOT in this leg** — still open, still the stream's.

🛑 **AND THIS LEG NOW SERVES TWO MILESTONES — IT IS ALSO `M_RP_IDENTITY_RESOLUTION.md`'s LEG E. NAMED HERE RATHER THAN DISCOVERED AT IMPLEMENTATION.** Both were filed as *an `$effect` on `selfState.connection`* in this same file. 🔒 **THIS MILESTONE OWNS THE BUILD** (§5 is this document's decision); **that milestone's Leg E CONSUMES AND VERIFIES** and is discharged when this leg lands. ⚠️ *Two seats writing one `$effect` from two runbooks is the one-writer-per-file-per-atom breach — which is why one runbook is authored, not two.*

✅ **GROUNDED 2026-08-02, NOT ASSUMED — LEG C DOES NOT DEPEND ON LEG B.** Leg B builds **delta setters** for live events; Leg C **re-runs fills**. **Different mechanisms ⇒ the ordering between them is free**, and Leg C may precede Leg B if Joe wants the reconnect hook sooner.

🔑 **BUT LEG C HAS TWO HALVES AND THEY ARE NOT THE SAME PRICE — MEASURED:**
- **The members half is FREE.** `loadMembers(sid)` (`app_client.svelte:171`) is already a **named callable**, and the §3.5 late-guard already discards a resolve whose `spaceId` no longer matches ⇒ re-entrancy is already handled.
- **The spaces/rooms half needs an EXTRACTION.** `spacesState.setSpaces(await invoke('get_spaces'))` (`:625`) is an **inline line inside the startup block, not a function**. ⚠️ **And it is a WHOLESALE replace** ⇒ the runbook must establish what it does to selection and to the room latch before it is fired a second time in a session. **`setSpaces` has had exactly ONE caller since it was written; Leg C makes it two.**
- ⇒ 📌 **`M_RP_IDENTITY_RESOLUTION.md`'s Leg E needs ONLY the members half**, which is the free one.

**Leg D — live verify.** **Surface: the real client on 9222, driven over CDP, against the REAL node (`run-node_service.lnk`), with the second identity `--instance bob`.** Two identities, one observer. **Observed:** a real `membership.join` updating the observer's roster **without a re-fill**; a real room create appearing in the rooms panel; the joiner NOT receiving its own join; churn returning the registry to its recorded baseline. **Measured:** the registry count recorded **with its store composition**, never bare.

**Leg E — records + close.** JOURNAL + CLAUDE.md PLAY + ROADMAP + this document, **one commit** (D-074).

📌 **The membership author-exclusion test is NOT in this milestone.** It moves the **cargo** floor while every leg here moves **`svelte-check`**; bundling them is the attribution-mixing the A/B split exists to prevent. It remains 🔓 open and Joe's, as its own small commit or not at all.

---

## §8 — DoD

Applying `M_RP_MEMBERS.md` §8b's rule — **every item below that says "observed", "exercised", "driven" or "measured" names its surface in §7, and every 🔒 in this document has a leg that builds it:**

- [ ] §2's router exists as one function, and no store gained a privileged setter — **Leg A**
- [ ] §4's boundary holds: the router mutates only stores that a fill populates — **Legs A + B**
- [ ] §6's scope check **exercised**: an event for a non-scoped Space leaves the roster untouched — **Leg D**
- [ ] `membership.join` **observed** updating an already-connected client's roster with no re-fill — **Leg D**
- [ ] the joiner **observed** NOT receiving its own join — **Leg D**
- [ ] a real room create **observed** appearing without an application restart — **Leg D**
- [ ] §5's **R1** reconnect re-fill **exercised** — ✅ **ruled 2026-08-02 (J-658)** — **Leg C + Leg D**
- [ ] **BOTH HALVES of Leg C exercised, named separately because they are not the same build** — the members re-fill (`loadMembers`) **and** the spaces/rooms re-fill (the extracted `get_spaces` call) — **Leg C + Leg D**
- [ ] **`setSpaces` fired a SECOND time in one session leaves selection and the room latch intact** — it has had exactly one caller since it was written — **Leg C + Leg D**
- [ ] 🔓 **R4 is NOT in this DoD.** Still open, still the stream's; **no item here may be read as covering it**
- [ ] `svelte-check` floor re-**measured** on the final tree, no new warnings — **Legs A, B, C**
- [ ] registry count **measured** with composition, churn returns to baseline — **Leg D**
- [ ] §6-i's subject field: `membership.kick` **exercised** and the **TARGET** leaves the roster while the **MODERATOR** stays — **Leg D**

🛑 **AND THAT LAST ITEM IS NEW AT v1.11 BECAUSE THE DoD HAD A HOLE THE SIZE OF §6-i.** Every membership item above observes `membership.join` — the one variant whose subject **is** `sender`. ⇒ **a router that read `sender` uniformly would have passed this entire DoD.** 🔑 Applying `M_RP_MEMBERS.md` §8b's rule to itself: **a DoD that only exercises the easy variant is a check that cannot fail on the hard one.** ⚠️ `kick` needs a node-admin op to drive it, not a second client — **that is a real addition to Leg D's surface and it is named here rather than discovered there.**

---

## §9 — Filed, NOT fixed

- ⚠️ **MEASUREMENT TRAP FOUND 2026-07-26 — `.claude/worktrees/` IS A STALE-CODE DECOY, SAME CLASS AS THE REPO-LOCAL `target/`.** A repo-wide `*.rs` search returns **eight** worktree copies of the tree before the live one, and they carry an **OLD LAYOUT**: `TransportMessage` sits at `xgen-node/src/wire/types.rs` there versus **`xgen-core/src/wire/types.rs` live**. ⇒ a grep that trusts first-hit lands in dead code at a path that no longer exists. **Exclude `.claude` from every repo-wide search.**

- ⚠️ **A DEFERRAL WRITTEN AS A CODE COMMENT HAS NO OWNER AND NO TRIGGER.** `app_client.svelte:564-565` deferred live spaces push to M-RP6.6; M-RP6.6 closed at J-543 and nothing brought the comment back into view. **Proposal, Joe's to rule:** a deferral that survives a milestone belongs in ROADMAP or DECISIONS, not in a comment. Not fixed here.
- `MembershipInvite · Mute · NodeUnban` unhandled by choice (§6).
- **No flap guard on §5's R1 reconnect re-fill.** ⚠️ **R1 IS NOW LOCKED (J-658), so this stops being hypothetical** — §5's own option text called a flap guard *"a later amendment, not a v1 requirement"*, and that stands. 📌 **Leg C's runbook must state which it ships**, so the absence is a recorded choice rather than an oversight.
- 🔓 **§5's R4 — SYNC-FROM-CURSOR ON RECONNECT — REMAINS OPEN AND IS NOT IN ANY LEG.** ⚠️ **Filed here explicitly because its sibling R1 was locked on 2026-08-02 and a half-ruled section is exactly how one half gets read as settled.** Its honest cost is named at §5a: **the GUI client persists no sync cursor**, so `since=""` means full replay. **Owner: Joe. Trigger: none named yet** — ⚠️ *and by §9's own first entry, a deferral without a trigger has no owner in practice; this one is at least in a filed list rather than a code comment.*
- A third consumer would justify revisiting §2's one-router shape; two does not.
- 📌 **SPEC GAP — `state.space_update` HAS NO CONTENT SCHEMA (§6a-i ③).** Dispatched to `=> Ok(())`; no applier function has ever existed. **Not this milestone's to fix** — a frontend router cannot invent a schema. Owed by whoever writes it.
- 📌 **SPEC GAP — `docs/xgen_appendix_i_en.md:95` PROMISES `state.room_update` CARRIES *name* AND *topic*; THE APPLIER CARRIES NEITHER (§6a-i ③).** ⚠️ **The spec is ahead of the code, and in a protocol project the spec is the deliverable** ⇒ this is a real divergence, not a code TODO. **Named, not fixed, and not this milestone's.**
- 🛑 **`KnownSpace` HAS NO EVENT-DRIVEN WRITER AT ALL (§6-ii).** Its three writers are the user's own local actions. ⚠️ **`role` is hardcoded `"owner"` at both create sites and `joined: false` has zero production writers** — so a routed room would have to **invent** both. **Filed here because it outlives this milestone under every one of B1/B2/B3.**
- 🛑 **THE RENDERER COLLAPSES THE THIRD STATE, SO LEG A's §5-iii FIX IS INVISIBLE TO THE USER (runbook §5-iv, measured 2026-08-01).** `entity-avatar.svelte:125` — `data-ai={flags.isAi || undefined}` ⇒ **`false` and absent produce identical DOM.** An AI joining live still renders as a human. ✅ **The store no longer ASSERTS `isAi: false` about a person it never looked up** — that stands and is not reverted — **but option D ①'s *"reads as unresolved"* is not delivered and cannot be from inside §2's three files.** 🔓 **Whether the renderer should distinguish it is Joe's**, deferred by him until after Leg A; it is `M_RP_MEMBERS.md` §6's word form, now filed there as its third unresolved-row case alongside §6a.
- 📌 **`M_RP_MEMBERS.md` §6a — THE `tail-8` LOCK-VERSUS-BUILD GAP, filed there at J-643, not fixed.** Joe locked *tail-8*; the shipped `tail()` returns the whole final path segment and `.ei-name` is **LEFT-ANCHORED**, **so the clip takes the WRONG END** — every unresolved row reads `ed25519:AbCd…`, **the constant head kept and the distinguishing bytes discarded.** ⚠️ *This entry previously read "the CSS clips the **left**", the inverse of the truth; corrected 2026-08-01 (J-649), kept not erased (`D-131`). §6a itself was always correct — the error was in the paraphrase.* Not this milestone's.

---

## §10 — Handoff

**Blocked on Joe:**
- 🔓 **§5's R4 — the STREAM's sync-from-cursor replay. Gates NOTHING today; it is in no leg.** ⚠️ *v1.0–v1.14 read "§5 — the reconnect rule. **Gates Leg C only.**" **§5's R1 half was ruled 2026-08-02 (J-658) and Leg C is unblocked; the R4 half was deliberately not bundled.** Superseded, kept not erased (`D-131`).*
- 🔓 **THE B1/B2/B3 SCOPE RULING — NEW AT v1.12, AND IT NOW HAS A MEASURED BASIS IT DID NOT HAVE BEFORE.** §6-ii establishes that **`get_spaces` reads disk and the router writes memory**, so **B1 makes the panel correct until restart and cannot make it true**. **Gates Leg B's runbook LOCK, not its authoring.**
- 🔓 **WHETHER AN UNRESOLVED ROW RENDERS DISTINGUISHABLY — raised by §5-iv, DEFERRED BY JOE until after Leg A (2026-08-01).** ⚠️ **Gates nothing here**; it is `M_RP_MEMBERS.md` §6's word form and is filed there.

**Not blocked:** ✅ **Leg A is DONE** (J-643). **Leg B's runbook may be authored.** §3 is locked.

**Owes:** `M-RP-MEMBERS Leg C live-membership verify` — unblocked by Leg A, **not discharged by it**: Leg A shipped with no CDP and no live run by design, and Leg C's REQUIRED LEG is two clients and a real join. · `M-RP-LIVEFEED-REFRESH Leg D live verify` — where that run actually happens. · 🔒 **`M-RP-IDENTITY-RESOLUTION Leg E refresh trigger` — NEW 2026-08-02: this milestone's Leg C IS that leg's build** (§7 Leg C). 🛑 **AND IT REACHES FURTHER THAN A LEG: that milestone's C-3 skin rule is gated on Leg E ⇒ `M-RP-IDENTITY-RESOLUTION` C-3 IS GATED ON THIS MILESTONE'S LEG C.** Recorded in **both** `Owes:` lines under `D-133`, because a cross-milestone gate written on one side only is a gate that goes stale invisibly on the other.

**Chat owes:** the §0b registry composition model (carried from the M-RP-MEMBERS arc, needs a live client).

📌 **NOT THIS MILESTONE'S, RAISED HERE BECAUSE §6a-i AND LEG A SURFACED THEM:** the two §9 spec gaps (`state.space_update`'s absent schema · Appendix I `:95` vs `apply_room_update`), the `is_dm` provenance-or-state ruling on the members panel (its own milestone, `J-640`), and `M_RP_MEMBERS.md` §6a's `tail-8` lock-versus-build gap (filed there at J-643).
