# M-RP-LIVEFEED-REFRESH — the live event router behind the members and rooms panels
> **Status**: ACTIVE  
> Version: 1.10  
> Date: Jul 2026  
> **Last updated**: 2026-07-27  
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

**CHAT PROPOSES (R4 for the stream + R1 for the panels).** 🔓 **RULING: OPEN.** ⚠️ **Leg C does not open until it lands**, but Legs A and B are unaffected and can proceed.

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

| Event | Consumer store | Effect |
|---|---|---|
| `MembershipJoin` | address-book | add the member to the roster; resolve the name (book, else xgid tail) |
| `MembershipLeave · Kick · Ban · NodeEject` | address-book | remove the member from the roster |
| `MembershipInvite · Mute · NodeUnban` | — | **v1: ignored, deliberately.** Neither changes who is in the room *now*. Recorded so the omission reads as a choice |
| `StateSpaceCreate · StateDmSpaceCreate` | spaces-state | add the Space to the tree |
| `StateRoomCreate` | spaces-state | add the room under its Space |
| `StateRoomUpdate · StateSpaceUpdate` | spaces-state | update the existing entry in place |

⚠️ **SCOPING IS A CORRECTNESS REQUIREMENT, NOT AN OPTIMISATION.** A `membership.*` event for a Space the user is not currently scoped to must not touch the roster on screen. The scope check is `roomLatch.effectiveSpaceId`, the same one the fill uses.

⚠️ **YOU NEVER RECEIVE YOUR OWN EVENT.** `fanout.rs:301-307` excludes the author **by identity**, and it is generic — it applies to `membership.join` exactly as it applies to a message (re-measured 2026-07-26). ⇒ **a roster built purely from the live channel would be missing you, invisibly.** This is why the self row is a fixture (`M_RP_MEMBERS.md` §4c), and it is now independently required by two unrelated arguments.

📌 **`spacesState` needs delta setters it does not have** — it is 53 lines with a single whole-list replace. That is a real, small addition, named here so Leg B's scope is not a surprise.

---

## §7 — Legs

**Leg 0 — Phase-0.** This document. ⚠️ **Read by a second reader against §6's event table and `wire.rs` before any runbook opens.** No code.

**Leg A — the router + the members consumer.** The `membership.*` branch on the existing `xgen-event` listener plus `addMember`/`removeMember` on the address-book store. **Surface: `ui/client/src/app_client.svelte`, `ui/common/lib/stores/address-book.svelte.ts`.** Frontend only; moves the **`svelte-check`** floor (baseline 0 err / 34 warn / 15 files). **Satisfies `M_RP_MEMBERS.md` §5 and unblocks its Leg C.**

**Leg B — the spaces/rooms consumer.** The `state.*` branch plus delta setters on `spaces-state`. **Surface: `ui/client/src/app_client.svelte`, `ui/common/lib/stores/spaces-state.svelte.ts`.** Moves **`svelte-check`**. 📌 **A and B are split deliberately: the second consumer is what proves the seam rather than asserting it.** One commit spanning both makes a regression ambiguous.

**Leg C — the reconnect rule.** 🔓 **Gated on §5.** **Surface: `ui/client/src/app_client.svelte`** (an `$effect` on `selfState.connection`).

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
- [ ] §5's reconnect rule **exercised**, once ruled — **Leg C + Leg D**
- [ ] `svelte-check` floor re-**measured** on the final tree, no new warnings — **Legs A, B, C**
- [ ] registry count **measured** with composition, churn returns to baseline — **Leg D**

---

## §9 — Filed, NOT fixed

- ⚠️ **MEASUREMENT TRAP FOUND 2026-07-26 — `.claude/worktrees/` IS A STALE-CODE DECOY, SAME CLASS AS THE REPO-LOCAL `target/`.** A repo-wide `*.rs` search returns **eight** worktree copies of the tree before the live one, and they carry an **OLD LAYOUT**: `TransportMessage` sits at `xgen-node/src/wire/types.rs` there versus **`xgen-core/src/wire/types.rs` live**. ⇒ a grep that trusts first-hit lands in dead code at a path that no longer exists. **Exclude `.claude` from every repo-wide search.**

- ⚠️ **A DEFERRAL WRITTEN AS A CODE COMMENT HAS NO OWNER AND NO TRIGGER.** `app_client.svelte:564-565` deferred live spaces push to M-RP6.6; M-RP6.6 closed at J-543 and nothing brought the comment back into view. **Proposal, Joe's to rule:** a deferral that survives a milestone belongs in ROADMAP or DECISIONS, not in a comment. Not fixed here.
- `MembershipInvite · Mute · NodeUnban` unhandled by choice (§6).
- No flap guard on §5's R1 reconnect re-fill.
- A third consumer would justify revisiting §2's one-router shape; two does not.

---

## §10 — Handoff

**Blocked on Joe:** §5 (the reconnect rule, gates Leg C only).
**Not blocked:** Legs A and B, once a runbook is written and locked. §3 is locked.
**Chat owes:** the §0b registry composition model (carried from the M-RP-MEMBERS arc, needs a live client).
