# Audit — the members panel: what exists, what was decided, what is still open

> **Status**: ACTIVE  
> Version: 1.3  
> Date: Jul 2026  
> **Last updated**: 2026-07-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this document exists

The members panel has been designed and partly built across roughly fifteen documents written over two months, in three different naming generations. The design of record is 82 KB long and written in the project's shorthand — section numbers, decision codes, journal entry numbers. That shorthand is efficient for continuing work and close to useless for answering the question *"what did we actually decide, and what did we forget?"*

This document answers that question in ordinary language. It invents nothing. Every statement here was read out of a document or measured in the code on 2026-07-31, and the detailed source is named at the end of each section.

**This is a reading, not a new design.** Nothing here changes a decision. Where the record is contradictory or incomplete, it says so rather than smoothing it over.

---

## 1. What the members panel is

It is the list of people in the conversation you are currently reading. It sits in the client's right-hand region, labelled *Members*.

Joe's own statement of its purpose is the clearest thing written about it anywhere, and it was recorded late — it had not been written down at all until the design was most of the way through:

> *"What for is the members' panel/widget? Primary for DM and secondary for access to member's avatar settings/RMC. If you want to communicate with a group of people, you don't select those avatars in members list, but you create room and invite those people into it, by invitation in avatar's RMC."*

So the panel has two jobs:

1. **Click somebody, start a direct message with them.** This is the main reason it exists.
2. **Right-click somebody, get their avatar's menu** — their settings, and the invite-them-to-a-room action.

And it has one job it must **never** acquire: it is **not** a group-selection surface. You do not tick several people and press "message these". Group conversation happens by creating a room and inviting people into it, one at a time, from each person's own menu. This is written into the design as a permanent prohibition, because a members list is the single most obvious place somebody would later add multi-select.

Neither of those two jobs is built yet. Today the panel renders names and nothing responds to a click.

*Source: `tasks/M_RP_MEMBERS.md` §4c.*

---

## 2. What exists today, measured

Three pieces of code are shipped and running:

- **The panel itself** — `ui/common/lib/components/widgets/members-panel.svelte`, 146 lines. It renders rows and is deliberately inert: no click handler, no hover, no cursor change, nothing that hints at an interaction it cannot perform.
- **The store behind it** — `ui/common/lib/stores/address-book.svelte.ts`, 145 lines. It holds the roster and the cached names.
- **The address book on disk** — `xgen-client/src/address_book.rs`, 721 lines. A file of everybody the client has ever seen, with their display name and whether they are an AI.

Two commands connect the frontend to the Rust side: one reads the address book, one triggers a fill of it for a given Space.

**The one thing that is missing is the most important one: the list never updates.** It is filled once, when you enter a Space, and then it is frozen. If somebody joins the room while you are sitting in it, you will not see them. If somebody leaves, they stay on your screen. The only way to refresh it is to leave the Space and come back.

This is not an oversight that was discovered late. It was designed, and then no piece of work was ever assigned to build it — see section 6.

---

## 3. How the panel gets its data, and why that matters

There are two separate sources feeding it, and they behave very differently:

**Who is in the room** comes from the node. The client asks for the Space's whole event history, replays it locally, and works out the membership from that. It is always current at the moment it is fetched, and it is **completely unavailable offline** — there is no cached copy of who is in a room anywhere on the client.

**What those people are called** comes from the address book on disk. That survives restarts and works offline.

The consequence is worth stating plainly: **offline, the client can tell you somebody's name but cannot tell you they are in the room.** This is not a defect; it is a property of the client holding no conversation data of its own.

That last point is a deliberate project-wide position, locked by Joe: the client is a reader-sender and holds no user data. It writes exactly five files — a process id, the address book, config, the keypair, and UI state. Nothing about any conversation is ever written to the client's disk.

*Source: `tasks/M_RP_MEMBERS.md` §4c and §4c-ii; `tasks/M_RP_LIVEFEED_REFRESH.md` §5c and §5f.*

---

## 4. What has been decided

These are grouped by subject rather than by the order they were decided in.

### 4.1 What the panel shows

**It shows the members of the Space you are currently reading a room in** — not everybody you have ever met. The alternative, showing the whole address book, was rejected because a panel labelled *Members* that is scoped to nothing is an address book, and the address book has no home in the interface yet.

**Your own row is always there.** Always, and always first — even when you are offline, even when no room is selected, even before you have registered anywhere. It is a fixture of the panel, not a member of the list. Joe's words: *"in the list of members there will be ALWAYS the self avatar. Even if the client will be offline. This is the home for the self avatar."*

This turned out to be required for a second, unrelated reason nobody anticipated: the system never sends you your own events. You do not receive your own "join" message. So a list built purely from live events would have been missing you, silently, and nobody would have noticed.

**A row that has not resolved to a name shows the last eight characters of the identity instead.** A friendlier word-based label was designed but deliberately deferred.

**Rows are shape-identical to the Spaces and Rooms panels.** The person's role and join date arrive with the data for free, and are deliberately not displayed. This was a choice, not an oversight.

### 4.2 What the panel must never claim

There is a hard rule underneath the whole design: **the address book stores observations, not current truth.** Every entry means *as of the last time I saw them, this was true.* A cached "not revoked" is only true as of then.

So: **staleness and absence both have to render as UNKNOWN, never as fine.** A panel implying "everybody here is valid" would be lying, and lying in exactly the direction the no-anonymity principle exists to prevent.

The concrete consequence: the avatar component can already draw a "revoked" badge, and the address book has a `revoked` field — but the wire never sets it, so it is `false` on every record. Feeding it would light a real warning badge from a constant. It is deliberately left unfed until the identity-lookup widening milestone lands.

Similarly, nothing in this panel may put a presence dot beside a name. Presence is not built, and the status slot on a row is for personal self-expression, not for "online".

### 4.3 The five things the panel can say about itself

This is the part of the design most likely to be lost if it is not read, because it is a shape rather than a rule. The panel does not have two states (has data / has no data). It has five, and they form a tree:

- **No room selected** → just your own row, and **no message at all**. This case absorbs "you are offline", because with no room chosen there are no other people being blocked — blaming the network would be a second false statement.
- **Room selected, membership known** → your row plus everybody in the room.
- **Room selected, still fetching** → *"I am waiting for the others"*
- **Room selected, the fetch failed** → *"I cannot reach the others"*
- **Room selected, no connection** → *"I cannot see the others"*

Those three messages are Joe's own wording, recorded verbatim. The third and fourth are a transition, not two different screens — waiting becomes failed when the attempt concludes.

One wording was explicitly rejected and the reason is worth keeping: *"I cannot see the others online"* reads two ways, and the second reading is a claim about who is present. The panel must not assert the one thing it has no mechanism for.

### 4.4 The rule that makes "I am waiting" honest

If the panel says *"I am waiting for the others"*, it has promised to eventually stop. That is only true if the fetch is guaranteed to end.

It is not, today. Connecting has no timeout. Each individual identity lookup has no timeout. Only the history drain is bounded, at five seconds.

So a hung node leaves *"I am waiting for the others"* on the screen forever — true at the first second, a lie by the tenth minute. And because the panel re-tries whenever you navigate away and back, the user experiences it as *"this room never shows its members"* rather than *"the app is stuck"*.

**The decision recorded is that the timeout is not a detail attached to this state — it is what makes the state sayable at all.** If the fetch cannot be bounded, that message must be dropped rather than shipped unbounded. Bounds are per-step, five seconds each, not one overall cap — because a Space with two hundred members is a legitimately long fetch and an overall cap would kill it mid-way.

There is a second reason this matters, and it is sharper. The fill holds a lock across its whole run. With an unbounded fetch, **one hung call holds that lock for the life of the process and no further fill can ever run** — a silently dead feature, worse than the problem the lock was added to prevent.

*Source: `tasks/M_RP_MEMBERS.md` §4c-i.*

### 4.5 Direct messages — the model behind the panel's primary job

Because clicking a member opens a DM, the DM model had to be settled, and it was, in some depth.

- **A DM is its own Space**, not a room inside an existing one. Creating it signs and sends three events in order over one connection.
- **Federation is switched off inside DM Spaces** — enforced in code with a real error, not by convention.
- **Opening a DM creates nothing.** Clicking a person opens a draft thread that exists only in memory: no Space, no room, nothing signed, nothing sent. The recipient learns nothing. The irreversible act is the first message, not the click. You can open ten drafts, send to none, and leave zero trace anywhere.
- **There is no client-side backup and there will not be one.** Joe: *"nothing will change. We have backup of the node, that has to be enough."* The accepted cost, recorded so nobody rediscovers it as a bug: with federation off, exactly one copy of a DM exists, on one node. If that node is lost, the conversation is lost for both people.
- **Where a DM lives has been decided and not built: bilateral replication.** The conversation lives on *both* participants' home nodes, synced between exactly those two. Nobody has to register on a stranger's node, nobody depends on the other person's operator, and node loss becomes survivable.

That last decision reversed the earlier lean. The objection to replication had been "two operators can see the content instead of one" — and that is simply false here, because everything is end-to-end encrypted and the node is content-blind. Operators hold ciphertext they cannot read, so the number of nodes holding a DM does not change who can read it.

**Two things bilateral replication needs, and one of them has never been checked:**

1. The DM federation ban has to narrow from "no federation at all" to "federation restricted to the two participants". The existing code comments say this was always the intent — the guard exists to keep *third-party* nodes out, and a participant's own home node is not a third party.
2. The Space's identity would need to derive from the two people's identities, sorted — so that both of them creating it independently produce the same Space, and duplicates become impossible rather than something to reconcile afterwards.

**Point 2 is unmeasured.** Space identities are currently content hashes of the signed root event, and two people signing different roots produce different hashes. Whether the scheme can derive from the identity pair instead **has never been checked**, and it decides whether this is a milestone or a rewrite.

*Source: `tasks/M_RP_MEMBERS.md` §4c-ii.*

---

### 4.6 How the list is displayed — and the switch that was designed but never built

There is a designed setting that lets a panel be shown three ways: **as lines, as avatars, or as a gallery**. It is real, it was walked with Joe on 2026-07-17, and it is written down in `tasks/M_RP_REGION_GEAR.md` — a document about the settings gear on each region tile, which is why no members document mentions it and why the first version of this audit missed it entirely.

The design splits into three layers that must be kept apart:

1. **The gear** — a settings button on every region tile, an exact twin of the one on each plugin row. Cheap, reusable, buildable today.
2. **The host** — each widget supplies its own settings component, mounted in the existing Settings pane. Shipped mechanism; each region opts in one at a time.
3. **The render variants themselves** — the actual lines / avatars / gallery presentations. This is described in the design as **the largest piece**, and it is per-widget work, not part of the gear.

**The design names the Rooms panel as the first tenant, not Members.** Members is mentioned once, in the section's closing line, as a possible later generalisation: *whether "show as" generalises to other entity regions (Spaces, Members) is its own Phase-0 when picked up.* So for the members panel this is **filed and not designed** — the intention is on record, the design is not.

Measured in the code on 2026-07-31: the row component already carries a variant axis of `row`, `card`, `nav` and `inline`, but the panel that renders every list **hardcodes `variant="row"`** for all of its consumers, Members included. The word `gallery` appears **nowhere in the interface code at all**. So today there is one display mode, it is lines, and it is not switchable.

Two consequences worth stating before anyone plans this:

- **Avatars and gallery are new presentation work**, not a flag on something that already exists. The existing `card` variant is not the same thing.
- **The members panel has a locked decision that points the other way.** Its row shape was locked as *shape-identical to the Spaces and Rooms panels, with the extra slots unfed*. A display-mode switch is additive rather than contradictory, but whoever picks it up should know that lock exists and was adopted on recommendation rather than examined.
### 4.7 Where the panel gets its scope — and where that mechanism came from

The members panel scopes itself to the Space that owns the room you are currently reading. It does this by reading a shared value called `effectiveSpaceId`, held in a store called the room latch.

That mechanism was not designed for the members panel. It was invented on 2026-07-17, during the milestone that turned the Spaces and Rooms panels into real widgets, to solve a problem that only appeared once those two panels shared a selection channel: **the Rooms panel both reads the channel, to know which Space it is showing rooms for, and writes to it, when you click a room.** So the moment you click a room, the channel no longer holds a Space, and the panel can no longer find out what it is scoped to. The fix was to latch the last Space selection separately and keep it.

Every later panel that needed to know *"which conversation am I describing"* — the message stream, the composer, and eventually the members list — was built on that latch. Two things follow that are worth knowing before anyone changes it:

- **There are two latches and the name hides it.** The Rooms panel has its own private one, which is a plain local variable and is not reachable from anywhere else. The shared one is a separate store. An earlier draft of the members design said "reuse the Rooms panel's latch", which was not merely suboptimal — it was impossible, and copying it would have created a third.
- **The honest cost of using the shared one:** it is empty until a *room* is opened. Selecting a Space in the tree does not populate the members panel. You have to open a room. That was accepted deliberately, because the alternative made it possible for the members list to describe a different Space from the conversation on screen.

*Source: `tasks/M_RP_MEMBERS.md` §4a; the mechanism's origin is in a 2026-07-17 session transcript, not in any project record.*

### 4.8 Why the Spaces tree and the members list behave so differently

They look like siblings and they are not, and the difference explains several things that otherwise look like bugs.

**The members list is derived from the node.** The client asks for the Space's event history and replays it. Always current when fetched, unavailable offline.

**The Spaces tree is the client's own written record of its own actions.** It is a plain local file read with no network involved at all, and it is written only when *you* create a Space or a room. Nothing else ever writes it.

This was deliberate and was decided on 2026-07-17: the read verb was chosen as a zero-network local read, the rooms were embedded inside it so no second call is needed, and the note at the time said it would be populated by registering and joining. So the tree is not drifting from its design — **it is doing exactly what it was designed to do.** What was never followed through is the consequence: if the tree only records your own actions, then a Space or room created by somebody else can never appear in it, and a live event router that writes only to the in-memory copy would show it until the next restart and then lose it.

Two fields make this concrete, measured on 2026-07-31:

- **`role`** is written in exactly two places in production, and both hardcode `owner`. No path ever writes admin, moderator or member.
- **`joined`** is written in exactly two places in production, and both write `true`. A `false` value exists nowhere outside the test suite.

So of the fields the Spaces panel renders, two are constants that look like data. Anything that adds a Space or room learned from somebody else's event has to decide what those two values are, and today there is no honest answer available.

---

## 5. What was built but never connected

Two capabilities exist, are tested, and have no way to reach them from the interface:

- **Creating a DM Space** — built, has a command-line verb, has a batch verb, has an automation verb. **No Tauri command**, so the client's UI cannot call it.
- **Opening your own self-thread** — built, tested, idempotent (calling it twice does not create two). Same story: no Tauri command.

This matters for planning. The milestone that lights up the members panel's click behaviour is **not** a feature milestone that has to build DM support. It is a command-surface milestone that wraps two existing, tested functions — the same work already done twice for the address book.

An earlier claim in the record said the self-thread was unbuilt and post-dated other work. That claim was inferred from where the item sat on the roadmap and was never checked against the code. It was wrong, and it would have made this look far more expensive than it is.

---

## 6. Why the panel is frozen, and where the fix went

The design said the list should keep itself current from live membership events. Verification was assigned to check that it did. **No piece of work was ever assigned to build it.**

The reason is a habit that has now been named in the record: **scope gets written in terms of files, requirements get written in terms of behaviours, and nobody reconciles the two.** Every leg did exactly what it was told. The verification leg was told to check something nobody had been told to build. This happened three times in three sessions.

The rule that came out of it: before a work plan is locked, walk every locked decision in the document and ask *which leg builds this?*

When the gap surfaced, Joe did not pick from the four options offered. He widened it:

> *"We have to build it properly and in this way that the mechanism will use also for rooms panel, where are all rooms of selected server and needs to be updated. If we will have such mechanism, it will have sense to use it intensively."*

That became its own milestone — the live event router — because the Rooms panel has the same disease and worse. The Members list at least refreshes when you change Space; **the Spaces and Rooms tree only refreshes when you restart the application.**

The members panel's remaining work is blocked behind that router's first stage.

---

## 7. What is still open

**Waiting on Joe:**

- How long "waiting" should last before it becomes "failed".
- If a DM's creation succeeds but the first message fails, the other person receives an invitation with no message in it. Is that a defect, or a legitimate state meaning *"someone started a conversation with you"*?
- Whether the self-card widget **is** the account settings pane or merely opens it. The record says "maybe", and a maybe is not a design.

**Waiting on measurement (not decisions — nobody can answer these until somebody looks):**

- Can a Space's identity derive from the participant pair? This decides the size of the DM hosting work.
- If one party destroys their keys, does the other party's retained copy survive? This depends on whether content keys are wrapped per-recipient, which has never been checked. It is the live edge of the project's hardest standing tension.
- Can a DM Space be migrated off a dying node at all?
- Does anything reconcile two DM Spaces created for the same pair?
- How does somebody on another node learn they have been invited?

---

## 8. Things I would flag before more work starts

**The three appearance and scope decisions were never actually examined.** What the panel is a list of, what an unresolved row looks like, and what shape a row is — all three are recorded as *delegated*: Joe said *"lock all by your recomms"* and adopted the recommendations without walking them. They are decisions of record, but not decisions anybody judged. The design document itself says to re-open the unresolved-row label freely on sight, because it is the first thing a user sees on a row that has not loaded.

**One design document has been overtaken and still reads as current.** `tasks/M6_CLIENT_MEMBERS_DESIGN.md` is marked PENDING and poses an open choice: should membership come from asking the node, or from caching it locally? That question was answered — the shipped code asks the node and replays the history. The document has never been updated to say so, and it uses none of the vocabulary the later documents use, so a search phrased in current terms does not find it.

**The milestone has no node on the roadmap.** Its paused state is recorded in its own task document and nowhere else.

**One shipped defect, unrelated to the freeze, found on 2026-07-31.** The panel highlights "the person you are talking to" when the Space is a direct message. The flag it reads for that is set once when a Space is created and never changed afterwards. When a DM is promoted into a full Space, the flag stays set — so the panel keeps highlighting somebody, and once there are three or more people it highlights an arbitrary one of them. Whether the fix belongs in the panel or in the protocol layer depends on whether that flag is meant to record *"this was born a DM"* or *"this is a DM"*, and the field carries no comment either way.

---

## 9. Where the detail lives

| Document | What it holds |
|---|---|
| `tasks/M_RP_MEMBERS.md` | The design of record. Everything in sections 1, 4, 5 and 6 above. |
| `tasks/M_RP_ADDRESS_BOOK.md` | The name cache the panel reads from. |
| `tasks/M_RP_PANEL_INERT.md` | Why the list does not respond to clicks. |
| `tasks/M_RP_REGION_GEAR.md` | The lines / avatars / gallery display switch, and the three layers behind it. |
| `tasks/AUDIT_RECORD_FINDABILITY.md` | Why that switch was unfindable, where records actually live, and the one argument that is lost. |
| `tasks/M_RP_LIVEFEED_REFRESH.md` | Where the "keep it current" work went. |
| `tasks/M6_CLIENT_MEMBERS_DESIGN.md` | The original question, now answered elsewhere. Overtaken. |
| `tasks/M13_CLIENT_IDENTITY_LOOKUP_WIDENING.md` | What has to land before revoked / trust / tier can be shown. |
| `tasks/RUNBOOK_M_RP_MEMBERS_LEG_*.md` | How each piece was actually built. |
| `docs/xgen_ch2_architecture.md` | The naming rules and the cross-Space visibility limits the design obeys. |

### Record trail

Phase-0 locked J-588 · read surface J-589 · one drain feeds the roster J-590 · handshake bounded and the DM model J-591 · store and widget designed J-594 · return shape J-595 · inert panel mode J-596 · **panel shipped J-597** · paused J-598. Address book: policy J-579, closed J-586. Live router: J-616, J-618, J-639.
