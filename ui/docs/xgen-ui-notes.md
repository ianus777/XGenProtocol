# XGen UI — Notes
> **Status**: ACTIVE  
> Version: 0.50  
> Date: May 2026  
> **Last updated**: 2026-07-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Light chronological notes on UI design and adjacent topics. Lower ceremony than `xgen-ui-design-brainstorm.md` (deprecated, kept as inspiration): each entry dated, free-form, no fixed problem/direction/open-questions scaffolding. Notes graduate into Ch6, DECISIONS.md, or a proper instruction file when they mature. Resolved items are not deleted — they stay with a forward pointer (e.g. `→ D-NNN`) so the record remains readable.

---

## 2026-05-15

### N-001 — CLI-first binaries with UI envelopes

Review whether `xgen-client.exe` and `xgen-node.exe` can be used as pure CLI binaries in addition to their normal UI-embedded mode. Today the CLI surface already exists (`whoami`, `status`, `--batch`, `--service` for headless Node) and library-first architecture is mandated (CLAUDE.md, D-037). Open question: is one binary serving both modes the right shape, or is a derivative CLI-only build preferable?

Analogy: FFmpeg has a stable CLI core with various UI front-ends built around it. Initial thought was that XGen UI extensions would be derivatives of the UI-embedded `.exe`. Refined thought: the same `.exe` may already be usable in pure CLI mode, given the library-first design — worth reviewing rather than assumed.

Likely implications if pursued:
- Explicit mode selection (`--cli` / `--headless` flag, or auto-detect from stdin/stdout redirection) so a CLI invocation in a script does not flash a Tauri window
- Clean exit codes and stdout/stderr discipline maintained across both modes
- Documentation: which subcommands are CLI-safe vs which assume an interactive session

Not urgent. For record now; review when UI work resumes.

### N-002 — Adversarial / misuse simulation suite (post-UI)

After the UI is built, build a simulation testing suite that goes beyond happy-path protocol correctness. Scope: node↔client interaction under irregular and hostile usage. Categories include:

- Privilege escalation attempts — regular user attempting actions reserved for admins or owners; spoofed sender fields; replay attacks on permission-changing events
- Out-of-context commands — commands valid in one state issued in another (e.g. send to a Room before joining; promote a DM that is already promoted)
- Malformed and adversarial inputs — overlong fields, control characters, unicode edge cases, malformed JSON, oversized batches, recursive structures
- Weird combinations — concurrent state-changing events that should not coexist; rapid join/leave/ban cycles; federation handshake interleaved with admin commands

Goal is hardening, not feature coverage. Separate from `stress-complete` (which is correctness under load) and from `smoke-ph2` (which is happy-path verification). Output is a list of error situations that the protocol handles cleanly and a list of failures that need fixing.

Depends on: UI track far enough along that a UI client can be driven adversarially, and a stable Auth Tier 1 implementation for the privilege model.

### N-003 — AI users in the XGen network

**Promoted to D-059 (2026-05-15).** See `DECISIONS.md` for the authoritative version. Notes below preserved as the discussion record.

Discussion completed 2026-05-15. Direction below; targets a future DECISIONS.md entry once the protocol surface is detailed.

**Core position.** AI is a first-class XGen Identity. Same shape as a human Identity — one keypair, one identity_id, one display name, one member-list presence, one DM relationship model. Different in declared capabilities and in some asymmetric behavioural rules. The target feeling: addressing an AI member is like addressing a knowledgeable human member who happens to be in the room — not like invoking a tool.

**Identity shape.**
- New field `is_ai: bool` on the Identity record, defaulting to `false`.
- Declared at `identity.register`. Immutable after registration. A human identity cannot later flip to AI or vice versa.
- Replicated alongside the rest of the Identity (extends Layer 15 / D-049).
- One AI deployment = one identity. Two different AI deployments are two different accountable actors with distinct keypairs and reputations.
- The same AI identity may be a member in many spaces and rooms, just like a human "famous member" can be.

**Capabilities pattern (door closed for now, future-proofed).** AI identities carry an open-enum set of capability flags. Phase 2 defines a minimal set with safe defaults. Future phases extend the set; old Nodes ignore unknown capabilities gracefully (same principle as `meta_atts` and the vanilla Node model).

Initial set:
- `dm_initiate: false` — AI cannot **create** a new DM space with another identity. AI can freely **send into** DM spaces a human has already opened (covers reminders, follow-ups, scheduled check-ins).
- `spontaneous_post: false` — governed by per-room permission (see below); default behaviour is response-only.
- Future capabilities reserved without specification — the field is the door.

Protocol-level: hard-enforced. A Node MUST reject events from `is_ai=true` identities that violate declared capabilities. Cleaner than soft enforcement — the audit log proves compliance.

**Invitation and accountability.**
- AI does not appear in a space by coincidence — it is invited (`membership.invite`) by a space owner or admin, like a human member.
- The inviter is recorded permanently in the DAG (existing protocol behaviour). This carries accountability — if the AI misbehaves, the inviter is on record.
- Beyond the inviter, an explicit **operator** role is recorded for the AI's lifecycle in the space. The operator is responsible for the AI's ongoing behaviour. Initially the operator equals the inviter; the inviter can delegate operator rights to another identity via a delegation event. Operator is mutable; inviter is not.

**Tier.** No special tier for AI. The AI inherits the tier requirement of whichever space it is invited into — if a national-health-care space requires Tier 4, an AI member of that space must also satisfy Tier 4. Verification of an AI's tier follows the same Auth Module mechanism as for humans; what counts as "verification" for an AI is the operator's institutional credentials (deferred specification).

**Removal.**
- Standard `membership.ban` and `membership.kick` work as for any member.
- Any admin or owner can kick. Moderators can mute. A foreign operator (an admin who is not the AI's operator) may kick when the AI's operator is absent and the AI is causing disturbance — they may understand the malfunction best.
- No special AI-removal mechanism.

**UI.**
- AI member is shown with the same avatar, name, and message-bubble styling as a human member by default.
- A small, unobtrusive **AI badge** marks the member in the member list. Default placement minimal; operator/admin may customise.
- Messages from AI use the same shape as human messages — no "AI response" header, no different bubble. The badge on the avatar / member identity is the only signal.
- Third-party plugins may decorate further (a "the AI is being playful" indicator was floated jokingly but is the kind of thing the module slot system supports).

**Pacing (resolved in N-005-territory, see below).** Space settings declare both `human_pacing_ms` and `ai_pacing_ms`. Clients enforce these as hard space rules (same status as auth tier requirement). See N-005 for the temperature-based dynamic extension.

**Multi-instance same-keypair behaviour.** Same as for a human running two clients with one keypair: both clients' messages enter the DAG, conflicts (if any) are resolved by Layer 12 (D-046). No special protocol handling. Operator concern, not protocol concern. AI is statistically more likely to produce simultaneous outputs (parallel triggers, scheduled jobs) so operators should avoid multi-instance deployments unless needed.

**AI-to-AI interaction.** Not prohibited by the protocol — two AI identities in the same room can address each other via the same rules as human-to-human. Practically rare today and noted with some dread (witnessed design-Claude ⇔ code-Claude exchanges spiral). Left open for the future; revisit when AI maturity changes the calculus.

**Cross-references.**
- Ch1 — Human and Agent Operation (philosophical grounding)
- D-036 — Module identity modes (related but distinct: modules can sign-as-user; AI signs-as-self)
- D-037 — Tier 1 = persistent accountable identity, not civil identity (foundational framing)
- Layer 15 / D-049 — Identity replication (where `is_ai` and capabilities replicate)

**Status.** Direction agreed. Pending: detailed wire-format additions (Identity record fields, capability enum, operator delegation event), Auth Module guidance for AI verification at each tier, UI badge specification in Ch6. Will graduate to a DECISIONS.md entry when those details are written.

### N-004 — Per-space pacing rules (precursor to N-005)

**Promoted to D-060 (2026-05-15).** See `DECISIONS.md` for the authoritative version. Note below preserved as the discussion record.

Space settings declare two pacing values:

- `human_pacing_ms` — minimum interval between messages from a member where `is_ai = false`
- `ai_pacing_ms` — minimum interval between messages from a member where `is_ai = true`

These are **space rules**, on the same level of authority as the space's auth tier requirement, role permissions, and federation list. A client that wants to participate in the space MUST enforce them locally for its own outbound messages.

Client behaviour:
- Outbound message queue. If sending would violate the pacing cap, the message is queued and released when the interval is satisfied.
- For humans: silent throttle (the user doesn't see anything unless they actually exceed; ~500 ms default is invisible to normal typing).
- For AI: visible to the operator (they're tuning a system); the queue and current pacing state are part of the AI client's operational surface.

Defaults (suggested starting values):
- `human_pacing_ms`: 500
- `ai_pacing_ms`: 2000

Different space cultures override: a contemplative space might use human=5000 / ai=30000; a fast-chat space might use human=0 (disabled) / ai=1000.

No Node-side enforcement in Phase 2 — Node trusts clients to follow space rules, same way it trusts clients to respect role permissions client-side before the Node validates server-side. Bad-actor clients can attempt to violate; they show up clearly in timestamps and are kicked by admins.

This note is a precursor to N-005, which extends pacing into a dynamic temperature model.

### N-005 — Room temperature mechanism

**Promoted to D-061 (2026-05-15).** See `DECISIONS.md` for the authoritative version. Note below preserved as the discussion record.

Pacing caps (N-004) are static thresholds. A richer model treats pacing overpasses as a **temperature signal** — a dynamic indicator of room health computed client-side.

**Concept.** Each pacing overpass adds heat to two counters: per-member and per-room. Heat decays over time when no overpasses happen. The system is forgiving of one-off bursts (a heated argument with quick replies cools off naturally) but accumulates against sustained patterns.

**Thresholds and effects (sketch, not final):**
- Member at warm → soft UI warning to the member ("you're moving fast")
- Member at hot → effective pacing cap doubles temporarily for that member
- Member at very hot → auto-kick with cooldown (e.g. 2 hours)
- Room average hot → admins notified ("room heating up")
- Room average very hot → space-wide cool-down (everyone's effective pacing rises)

**Why this is good.**
- Distinguishes occasional human burst (forgiven) from sustained problem (escalated). Static caps treat both identically.
- Self-cooling without admin intervention. Rooms recover on their own. Admins only act when temperature stays elevated — the genuine signal that something is wrong.
- Provides a visible health metric. Temperature correlates with emotional climate, not just message rate. A room can be high-traffic and cool (well-paced volume) or low-traffic and hot (sparse but bursty conflict). Useful for moderation.

**Layer.** Client-side, not protocol-level. The room's home Node is the natural authority — a room "lives somewhere" and temperature is judged where it lives, analogous to criminal jurisdiction. Other federated Nodes may receive temperature information via `meta_atts` on relevant events if a client chooses to surface it; the home Node's computation is authoritative.

**Fairness considerations.**
- Computed on **send timestamp** (client-stamped), not receive timestamp — network jitter must not punish a member whose messages happened to clump in transit.
- Decay parameters tunable per space (decay-rate, threshold values).
- Auto-kick action is logged with reason `auto_temperature` and the cooldown timestamp.

**Open questions.**
- Exact decay model (linear? exponential? half-life?)
- Default threshold values (warm/hot/very-hot)
- Cooldown duration policy (fixed 2h? scales with offence count? expires after good behaviour?)
- Interaction with AI-specific behaviour: AI flooding likely triggers temperature faster; is that fair? (Probably yes — AI is the more dangerous flood source.)
- UI surface specifics: form factor for the indicators (thermometer? coloured ring around avatar? heat-map of recent activity? colour shift on badges?) — design Claude to propose.
- Does temperature survive room restart / Node restart? (Probably not — it's ephemeral, computed from recent history only.)

**UI indicators (confirmed direction).**

Both room-level and member-level temperature are surfaced visually in the UI.

- **Room temperature.** Visible in the room list (advance signal — see which rooms are heating up before entering) and inside the room (header thermometer or equivalent). Visible to all members — the room's collective vibe is shared awareness.
- **Member temperature.** Visible on the member's avatar or member-list entry. Form factor open (design Claude to propose).

**Visibility policy (confirmed default).**
- Room temperature: visible to all members. The room's collective state is shared awareness.
- Member temperature: admins and moderators only by default. The member themselves always sees their own. Public per-member visibility is **configurable per space** — some communities will choose full transparency; the conservative default is moderation-only because publicly visible "Alice is hot" can itself be socially inflammatory.

**AI under the temperature mechanism (confirmed direction).**

AI is subject to temperature, but the escalation differs.

- **Pacing is rigid for AI.** The AI's client cannot exceed `ai_pacing_ms` in a given room — the same way it cannot violate the tier requirement. Enforced as a space rule, not left to operator goodwill.
- **Temperature still accumulates** for AI members. Even with rigid pacing, an AI can heat up if it is consistently posting at the cap or close to it.
- **Escalation differs from humans:**
  - AI at warm → soft signal back to the AI's client ("heating up — slow down")
  - AI at hot → effective pacing cap widens further for that AI (already rigid, now slower)
  - AI at very hot → **temporary mute, not kick.** AI cannot post for a cooldown period. Returns automatically when the cooldown expires. Keeps membership, keeps DM threads, keeps room context.

**Why the asymmetry.** Human overshoot is typically a social signal (rudeness, abuse, malicious intent) that warrants ejection. AI overshoot is typically a capability signal (excited model, fast inference, complex topic) that warrants throttling. Treating both identically would either kick AIs unfairly or fail to discipline humans appropriately. The asymmetry reflects what the temperature is *actually telling you* about each kind of member.

A second-order effect: an AI that frequently hits temperature in a given room is a tuning signal for its operator/admin — the room wants slower contributions than the AI is configured to provide. Consider raising `ai_pacing_ms` for that AI, or have a less excited AI in this room. The mechanism is a feedback loop for configuration, not a punishment.

**Status.** Idea recorded. Expected to expand significantly. May graduate to its own design document if it grows past a single note.

### N-006 — Mind the scope of dispositions involving xgen-client

Retrospective from the J-065 implementation session.

The Pass 2 disposition file `tasks/AI_USERS_AND_PACING_ph2.md` mixed two scopes that should have been separated:

1. **Protocol side** — `xgen-common` types, `xgen-core` state handling, `xgen-node` visibility filtering, the `NoOpTemperaturePlugin`. This is the protocol surface for D-059 / D-060 / D-061 and absolutely needed to exist now so two Nodes can talk about pacing and temperature correctly.
2. **Client side** — `xgen-client/src/pacing.rs`, `xgen-client/src/temperature.rs`, the Tauri commands `get_pacing_state`, the Tauri event `xgen-temperature-update`. These pieces have no consumer yet — the chat UI doesn't exist. They are correct, tested (23 tests in xgen-client-lib), and harmless, but they sit unused until UI work begins.

The code is left in place because removing it would be wasteful (working, tested code that matches Ch6 spec). When the UI work begins, the Rust side is already there waiting.

**The lesson for future dispositions.** When a task involves any combination of protocol + client + UI, discuss scope first before writing the disposition. The default split is:

- One disposition for the **protocol layer** (xgen-common + xgen-core + xgen-node) — run when ready
- A separate disposition for the **client implementation** — run alongside or after UI design is unblocked

This preserves Code Claude's ability to deliver clean, complete tasks without leaving Rust surface area orphaned.

**The Tauri shape contract.** The Rust side of a Tauri command or event is a malleable shape contract. When the Svelte component is built and wants a different field name or extra context, you change the Rust struct, not the Svelte component. Rust adapts to the UI, not the other way around. This means premature Rust surfaces are not a hard lock-in — but they are wasted work until consumed, and may need rework when the actual consumer is designed. Hence: discuss scope first.

**Acknowledged 2026-05-15** (Pass 2 retrospective).

---

## 2026-06-02

### N-007 — Every module needs a UI representation in both apps

Forward note from the Durable EventStore / module-framework discussion (J-227 arc). As the module system grows (EventStore storage engines, auth-tier modules, the temperature plugin, future viewers/projections), keep in mind that **every module will need some UI representation in the UI apps — including *system* modules, not only *display* ones.**

The module-framework taxonomy (settled by-trade; `tasks/EVENTSTORE_DESIGN.md` §8) is `kind ∈ {system, display}` × `host ∈ {node, client}`. Display modules *are* UI by nature. System modules also need a UI surface, a different one:
- **install / enable / disable / select** — which storage engine is active, which auth modules are loaded; surfacing the capability registry (`installed_plugins()`) + the loader's config selection;
- **status / health** — engine running vs vanilla floor; per-Space event count / file size;
- **warnings** — concretely, the vanilla EventStore operator contract ("storage heavy — install the engine module") needs a UI home so it isn't only a log line; likewise any module degrading or failing.

Both apps carry both kinds:
- **node app** — operator/admin UI: manage + monitor system modules (storage, auth, federation policy); node-side display modules (dashboards, audit-log viewers, health panels);
- **client app** — user UI: client-side display modules (themes, message renderers, viewers) *and* settings for client-side system modules (materialisation/index cache, sync).

Not now — UI work is not unblocked. For record so the module-framework milestone + Ch6 inherit it: each module slot should be designed with "where does this show up in the UI, and how does the operator/user manage it?" as a first-class question, not an afterthought. Graduates into Ch6 + the module-framework milestone.

Cross-ref: `tasks/EVENTSTORE_DESIGN.md` §8 (module-framework stance; `kind×host`); the vanilla operator-contract warning that needs a UI home.

---

## 2026-06-18

### N-008 — Node representation in client view

From the UI-concept brainstorm session (client's-eye view as the fundamental lens). How nodes appear to a user working in the client.

- **Home node** — singular, personal, lives with the **avatar** (the user's own infra home; orange-adjacent). Always exactly one.
- **Foreign nodes** — never rendered as standalone node objects. A node appears only as the **host-stamp on a Space** (blue, secondary, on the Space element). Federation is experienced as *reaching Spaces*, not *browsing a node graph*; the node is provenance, not navigation.
- **Corollary** — blue (infra) always has a host object to attach to (avatar or Space); nothing floats.

Open thread carried from the session: a Space whose host node is down is the one moment a foreign node demands visible presence — likely belongs to the temperature/liveness surface (N-005) on the Space element. Parked.

### N-009 — Contact book as the `users`-axis canonical home (client-layer concept)

**Layer note (important).** Contact book / circles / folders are **client-implementation** concepts built over protocol identities — **not protocol objects.** The protocol knows *identities* (verified, federated) and their *visit cards* (N-010); it does not know "contact books." Private **by construction** — purely client-local, no node holds the concept, nobody can pull or push it. May later materialise in the client liteSQL/redb cache (D-080) but remains an implementation artifact. **D-088 tiers govern the visit card (protocol), not the contact book.**

- The **contact book** is the single canonical store of known members (one entry per person; identity-forward, orange). Concrete home of the `users` axis. Conceptually *yours* — your contacts have always been yours (the Gen-X intuition; privacy follows from being client-local by nature).
- Holds **humans and AIs/bots alike** (D-059) — AI entries carry an honest identity-class badge, not a separate store/ghetto. (Badge form factor = open thread.)
- Every member appearance in a Room/Space is a **presence-reference** into the contact book — never a copy, never a fake-primary alias. One canonical entry, many non-duplicating references. (Shortcuts-as-aliases rejected: they imply a false "primary home"; multi-membership is native to the reference/tag model.)
- **Three-state avatar model:** (1) **Contact** — in your book, persisted, full. (2) **Unknown** — verified identity present in a shared room, no book entry; public data fetched on-demand from home node, not persisted. (3) **Self**.
- **Known vs present:** contact book = everyone you know (persists); room roster = who's here *now* (ephemeral, a presence-filtered view). Unknowns = on-demand home-node fetches. No-anonymity holds — unknowns are *verified*, never anonymous; "verified" ≠ "in my book."
- **Unknown → contact promotion:** the on-demand public fetch doubles as the preview the user acts on to save them (state 2 → 1).
- **Per-contact private annotations.** Each contact entry may carry a user-authored, non-shareable KV map (cf. Google Contacts custom fields/notes: "met at X," "prefers email"). This is **your private data about them** — opposite ownership and direction from the visit card's optional tier (which is *their* self-curated public data). Never pulled, never pushed, no tier governs it; the purest client-local data in the model. Keep the two visually distinct: *their bio* vs *your note about them* have different trust properties.
- A contact entry is therefore three clearly-owned strata: (a) a **reference** to the protocol identity, (b) a **cached visit card** (their public data; decays by their tier, N-010), (c) **your private annotations** (yours; never shared).
- **People-only** — the `users`-axis home; Spaces live on the `localities` axis with their own store, not merged (preserves orange/blue separation).
- **Circles** (people-groups) = saved views/filters over the contact book; client-local, never leaves the client.

**Vocabulary:** "contact" = the *person* (identity/orange); "address" would lean *location* (locality/blue) — "contact" keeps the axes verbally clean. Gen-X register: owned, pre-platform contacts; quietly anti-enshittification. **"Contact book" = LOCKED permanent vocabulary** (alongside "Space").

### N-010 — Visit card: two-tier model + tier-relative decay (protocol-grounded)

Every identity has a **visit card** (public profile), in two tiers:

- **Mandatory tier** — system-public, non-optional (verified handle, home-node, identity proof). The no-anonymity floor. **Travels inline** with the identity/envelope (it *is* the verification signature). Never decays; re-fetchable; orthogonal to retention tier.
- **Optional tier** — identity-defined, self-curated (display name, bio, avatar image, etc.). **Pulled on-demand** from the subject's home node (the same act as unknown-resolution, N-009). Ephemeral render-cache permitted (cache ≠ store; time-boxed); never pushed-and-stored.

**Retention is tier-relative, inheriting D-088** (not a separate card rule):
- **T1** — decay-to-zero (ephemeral; the floor).
- **T2/T3** — module-defined decay (half-life set by the Auth Module; the delegated interior, untouched).
- **T4** — no decay; lawful-basis permanent retention (Art.17(3)).

**Decay = stepped/quantized, not a continuous dial.** The gradient is *across* the tiers (zero → module → ∞), monotonic — a gearbox, not a knob. Continuous TTL rejected: it would re-take the module-delegated interior and make erasure timing undefendable.

Decay applies to the **optional-tier card data only**; the mandatory/verification tier is exempt — so "decay" never means the *identity* fades (no-anonymity preserved).

**UI consequence:** a Space's **tier** is a dignity-relevant infra fact ("is my participation kept here?") → render as a quiet **blue property on the Space element**, beside the host-node stamp (N-008). The console status-bar tier glyph already establishes the vocabulary.

**Open doc-owe:** the optional-field set is identity-defined → the card schema needs a stable mandatory core + an open optional region (Appendix/protocol-schema territory, not UI).

**Routed-to-Joe flag (not a UI note):** N-010 reframes **D-088 in the *temporal* dimension** (erasure tiers → decay/longevity classes). May warrant a D-088 amendment or derived decision in `DECISIONS.md`. Flagged for Joe to route; not absorbed silently here.

### N-011 — Entity shape language + avatar model

From the shape-vocabulary brainstorm. Let the entity's visual token be derived from a simple, render-cheap shape system; shape carries the two-axis semantics.

**Shapes (the two axes as geometry):**
- **Identity → circle** (person, organic; identity/orange axis).
- **Locality → square**, rounded corners (container, structural; infra/blue axis).
Principle: *people round, places cornered.* Distinguishable instantly at small size / peripheral vision.

**Inner stack — background ↔ picture is mutually exclusive, never empty:**
- *No picture:* background color **+** initials, **both forced on** (neither can be turned off) — no blank/bare state is possible.
- *Picture set:* picture replaces the background; **initials become toggleable (on/off)** — the only inner-stack switch, available only when a picture exists.

**Customization (separate from toggling):** background color and initials color are **user-customizable**. Defaults are **deterministic seed values from the stable ID** (axis-constrained: identity → orange-family, locality → blue-family). The seed is a stable starting point, not a lock — the user may override. Seed (color + initials source) derives from the stable ID (`identity_id` / `space_id`), **not** the display name, so the visual token stays stable across renames and never flickers per-render.

**First state on account / Space creation:** seed background color + seed-color initials (1–2 letters).

**Avatar-editor control surface (self-documenting via grayed states):**
- *Picture:* upload / remove.
- *Background-color picker:* **grayed when a picture is present** (the picture *is* the background).
- *Initials-color picker:* available.
- *Initials-shown checkbox:* **grayed (locked on) when no picture; enabled when a picture is present.**
The two controls gray under **opposite** conditions — which visually teaches the background↔picture exclusivity without explanatory text. The initials on/off control and the colour customization live in the same avatar-editor cluster; each acts independently.

**Transparency caveat:** when a picture replaces the background, keep a neutral under-fill beneath the clipped image so alpha-PNGs composite onto something, not onto the page behind the shape.

**Decorator separation:** decorators (self-ring, AI identity-class badge per N-009/D-059, temperature/liveness) ride the shape's **outer edge/ring**, never the inner stack. The component has an *inner layer stack* (background/picture/initials) and an *outer ring* for slot decorators — keeps the base/decorator split and 7-slot model intact.

**One component:** conceptually `<Entity kind=identity|locality image? name id>` (Svelte) — clip-shape (circle vs rounded-square via `clip-path`/`border-radius`) and axis seed-colour-family are the only per-kind differences.

**Open / parked:** non-Latin initials (font + script handling — user base is not English-only); rounded-square corner-radius value; a contrast floor / nudge when custom background+initials colours are set too close; the **Room sub-shape** (Room is contained by a Space — should read as the square's child, e.g. inset/smaller square, not a third top-level shape); the **outer-ring** mechanic itself (self-ring, temperature ring) is undesigned.

### N-012 — Accent distinguishes the anchor entity within each axis

Semantic placeholder only. An **accent** is whatever distinguishes an entity from the plain baseline of its kind. No graphic form is assigned here — the treatment (border, ring, glow, weight, etc.) **and** the deeper meaning it may encode are deferred to the CSS / styling pass (normalized vs custom CSS). This note fixes only *which entities are accented and why*.

Accent carries one unified meaning across both axes: **it marks the anchor / primary member of each axis pair.**
- **Locality axis:** accented locality = **Space**; plain locality = **Room**.
- **Identity axis:** accented identity = **self / me**; plain identity = **member** (another identity).

So Space↔Room and self↔member are the *same distinction* applied per axis: the anchor is accented, the contained/other is plain.

**Partially resolves the parked "Room sub-shape" question (N-011):** a Room is not a new shape — it is simply an **un-accented square** (same locality shape, accent absent). Likewise a member is an un-accented circle. No third top-level shape is needed.

A later accent *encoding* (e.g. Space host-provenance per N-008, or tier per N-010; self = "this is you") may attach when the outer-ring mechanic and CSS are designed — but the principle to hold is only that an accent must **distinguish/mean**, not merely decorate. Encoding choice = open.

### N-013 — UI state mirrors user perception, not protocol storage

Load-bearing principle for the client state tree **and** the node's derived UI (resident desktop mode, D-056). Both surfaces obey it.

**Principle:** the UI state model is shaped by what the user *perceives*, not by how the protocol stores or materializes. Where the two diverge, the tree follows the felt model; the protocol mapping is a downstream materialization concern the state model hides.

**Worked case (the one that surfaced it):** a user perceives themselves as both "in a Space" and "in a Room" — membership feels real at both zoom levels. So the UI presents a `members[]`-shaped view at **both** levels, even if underneath Space-membership is a derived rollup of its Rooms' members. Felt-real wins; the rollup is backstage.

**Consequence — UI-scope "container" umbrella (fenced):** *within UI scope only,* Space and Room are treated as **containers** sharing one membership shape (nested: Space holds Rooms). This term is UI-local and explicitly **not** protocol vocabulary — the protocol knows only Space and Room. The shared shape is why "who's here" rendering is written once and reused; the federation-stamp (Space) vs. permission (Room) asymmetry is real but backstage. *(Connects to N-012: a Room is an un-accented Space — same shape, accent absent.)*

**Kills the "localities" drift:** an earlier client-only synonym for joined Spaces — dropped. Canonical nouns (Space/Room) stay; "container" is the only sanctioned UI umbrella, and only inside this scope.

### N-014 — Context-scoped action with swappable presentation

From the invite-affordance brainstorm. An **action** (invite, rename, set-tier, leave-with-confirm, edit visit-card — anything launched from an entity's context menu that needs a little input before firing) is modelled as *intent + required inputs + the event it fires*, **decoupled from how it is presented**. Presentation (modal / docked tool window / floating popover) is a host decision, not baked into the action.

**Why decoupled:** modal-vs-docked-vs-floating is explicitly **a thing to test later**, not decide now (Joe). Keeping presentation swappable makes flipping invite from popover → modal a one-line host change, A/B-able during testing, not a rewrite. The pattern that survives is *context-scoped action, swappable presentation* — "tool window" is just one presentation of it.

**Invite is the first instance.** Invite launches from the **context menu of a Space or Room avatar** (which level = protocol-shape call, Joe's; Space-invite is the federation-bearing, expirable front door that produces `3044 invite_expired`; Room-invite is likely an in-Space permission grant per `RoomPermission`/`Effect`). Open: whether "invite" (Space) and "add" (Room) are one mechanism or two — routed to Joe.

**Launch → outbox loop:** the action surface *fires and dismisses* (optimistic); the **fate** (pending / held-pending / expired / rejected) renders as a **badge on the originating avatar** (the Space/Room it sprang from) — not as its own panel. So: context menu → action surface (collect + fire) → dismiss → outbox status badge on source avatar. (Outbox = the durable-pending state branch from the state-model thread; J-081 carry-over.)

**Parked for testing:** anchored/tethered vs floating; one-at-a-time vs several open in parallel. Deferred deliberately — the swappable-presentation design exists precisely so these stay cheap to change.

### N-015 — In-app console = scrollback + tilde input line (rides the D-056 command layer)

Recorded so it is not forgotten (Joe). The tilde (`~`) key opens an in-app console. This is **not a new subsystem** — it is the in-app *face* of the shared command layer already locked in **D-056** (UI / Console / `--batch` all funnel through one clap parser). The console's input line is the visible mouth of the same CLI the node/client already speaks.

**Architectural consequence:** UI actions should ride the shared command layer, **not bypass it**. The console proves the command layer has a human-facing surface; building UI affordances that sidestep it would fork the command path D-056 deliberately unified.

**Two halves, one component:**
```
console
├── scrollback   → read-only: system messages, event log, command output  (see N-016)
└── input line   → tilde-invoked CLI → shared command layer (D-056)
```

### N-016 — System-message scrollback as the read-only event/rejection log

The read-only text panel of system messages (the console's scrollback half, N-015). It is the **honest, low-level register** of what the client/node is doing — system messages, event log, command output.

**Second home for outbox/federation signals:** a `3044 invite_expired` (or any federation-derived rejection / held-pending signal) renders in **two registers** — a glanceable **badge** on the relevant Space/Room avatar (N-014), *and* a line in the **scrollback**. The badge is the at-a-glance version; the scrollback is the full honest record. Same event, two registers — fits **D-065** (surface gaps rather than paper over them).

**Relation to outbox:** scrollback is a *display register*, not the outbox state itself. The outbox is the durable-pending state branch; the scrollback is one of the surfaces that renders its signals (the other being avatar badges).

### N-017 — Outbox as a UI representation surface (cards awaiting resolution)

**Scope fence (read first):** this is a **UI representation note**. It governs *what events the UI is allowed to draw, and how* — not the data structure, not the event stream, not the outbox's underlying state, not protocol. Events flow regardless; this note is only about representation. Same fence discipline as the "container" umbrella (N-013).

**What the outbox is (UI view):** a front-stage **deck of friendly cards awaiting resolution** — the live, still-unresolved tail of the client's leveled event stream. The exception/degraded-path surface: on the happy path it is mostly empty (a normal message send confirms in milliseconds and never visibly stacks; it only surfaces here on a problem).

**Two registers of the same event stream:**
- **Scrollback console** (N-016) — CLI-honest, terse, coded (`3044 invite_expired`), opt-in (tilde). The machine register. Immutable full record.
- **Outbox card deck** — friendly, front-stage, always in view. The human register.

*Honesty without hostility:* the technical truth is never hidden (it lives in the console — D-065), but it is never forced on the user either. A normal person never has to meet a raw status code.

**Card form (never a raw CLI line):**
```
OutboxCard (awaits resolution)
├── icon         → kind + severity glyph (calm — no shouty words; leans on N-011 shapes)
├── title        → plain language, human  ("Invite to XGen expired")
├── description  → optional one line       ("Send a new one?")
├── accent color → severity as hue, calm palette — NOT a "WARNING" text label
└── action row   → demand only (N-014 action: acknowledge / retry / dismiss)
```
Severity is carried by **icon + accent color**, never a shouty word. The word "error" may not appear at all.

**Two orthogonal axes:**
- **Severity:** info · warn · error  (how it looks / how loud).
- **Interaction:** passive · demand  (does it carry an action?).
They are independent — a demand may be low-severity; an error may be passive.
- *passive* → no / disabled action row, but the card is still fully friendly and legible.
- *demand* → enabled action row → carries the N-014 context-scoped action.

**Aggregation:** transient warnings aggregate **by Space, passive-band only** — a flaky connection emits **one** rolled-up card ("12 messages pending — connection degraded"), never one card per stuck message. Per-message status stays quiet *inside the thread* (grey clock); the outbox carries only the rollup. Aggregate cards may **expand inline** to list members (disclosure, not action). **Demands never aggregate** — each is its own decision.

**Muting = UI representation filter ONLY:** decides which events are *allowed to be drawn as cards*. No reach into data structure, state, or protocol — events always flow, and the **scrollback always records, unmuted** (you can always go to the console and see what you silenced; D-065).
- **Axes:** by **type** (category-wide) · by **source** (per-entity; source = stable entity ID, reuses N-011 avatars — "mute this Space" = add its avatar to a muted set).
- **Per-type policy (three values):** forced-on (unmutable) · default-on-changeable · default-off-changeable.
- **Compose:** a card shows iff *type-not-muted* **AND** *source-not-muted*.
- **Gate (load-bearing):** **demands are forced-on, never mutable.** Interaction type gates mutability — a required action can never be silenced into invisibility. (This is the "some cannot be muted by type or source" rule.)

**Event population deliberately deferred:** the precise list of events that land in the outbox is **discovered empirically during testing**, not designed up front. This note locks the *shape* (card, registers, axes, aggregation, muting); the *population* is open. Designing the container, not enumerating its contents — and not pretending a speculative list is complete (D-065).

**Relation to neighbours:** launch via N-014 context-scoped action → fate renders as outbox card + avatar badge; scrollback (N-016) is the immutable full record of the same stream; icons lean on N-011 shape vocabulary.

**Parked (CSS / test pass):** exact icon-to-event mapping; whether aggregate expand-to-members is always allowed; calm severity palette values; the modal/docked/floating presentation of demand actions (N-014, test-decided).

### N-018 — Dynamic components (`<svelte:component>`) as default composition strategy

We will lean **extensively** on Svelte's dynamic-component feature (`<svelte:component this={...}>`) as the primary way the UI is composed. It is the implementation realization of the *write-once, parameterize-by-kind* instinct that recurs across these notes:
- **N-014** — swappable action presentation (modal / docked / floating) is a host that swaps the presentation component dynamically; the action stays the same, the chrome is a `this={...}` swap. This is *why* presentation stays test-decided and cheap to flip.
- **Entity control panel** — self-panel and home-node panel share one shape (avatar + scoped settings + context menu + badges); one component, dynamic by entity kind.
- **Container list (N-013)** — Spaces and Rooms share one list-item shape; one component, dynamic by container kind / zoom level.
- **Outbox card (N-017)** — one card; the action-row (passive vs demand) is a dynamic slot/component.

**Consequence:** the default answer to "these two things are the same shape with different content" is *one component selected dynamically*, not duplicated components. Keeps surface count low and makes the no-drift discipline (one authoritative shape per surface) enforceable in code, not just docs.

*UI-implementation note; no protocol/data implication.*

### N-019 — Components live in the UI library and are reused, never rebuilt from scratch

**Standing implementation rule (all UI authoring — Clair, Ms Design, any session).**

Every UI element is a **component in the UI library** with a defined structure, imported and reused — **never re-implemented inline or rebuilt from scratch**. Re-creating an element that already exists is a defect, even when it "works."

**Why state it explicitly:** the known AI failure mode is rebuilding a component from scratch *despite* an instruction not to — because it could not reliably *locate* the existing one. A prohibition without a lookup path cannot be followed. The fix is discoverability, not a sterner "don't."

**Our advantage (Rust + Tauri + Svelte):** single-file components (one `.svelte` = one component, greppable `lib/components/` tree, filename = component name); `$lib` alias gives each component one canonical import path; typed Rust/Tauri data boundary discourages divergent rebuilds; dynamic composition (N-018) shrinks the surface that *could* be rebuilt.

**Enforceable discipline:**
1. Maintain a **component index** under `ui/` — one authoritative list: component name · path · props/shape · purpose. (Same no-drift, one-authoritative-source discipline as doc surfaces, D-067/D-070/D-075, applied to components.) Index is populated when the library is laid down (no components yet to index at brainstorm stage).
2. **Before authoring any UI element:** consult the index — exists → import and reuse; genuinely absent → create in the library **and** register in the index, same step.
3. Re-implementing an indexed component inline is a defect to correct.

*UI-implementation rule; no protocol/data implication. Likely graduates to DECISIONS.md once the library/index location is fixed.*

### N-020 — Component envelope: root `<div class="type">`, one identity in three places

Every component's root is a `<div>` (or the appropriate element) carrying a single **type class** = the component's name in **kebab-case** (`OutboxCard` → `class="outbox-card"`). No prefix (`xg-` etc.) — every CSS element in the project is XGen by definition, so a namespace prefix is redundant.

**One identity in three places:** component name = file name = root type class, all agreeing. Given any one, the other two are mechanically derivable — which keeps the N-019 index honest.

**CSS ownership:** a component styles only what is under its own root type class; rules never reach up or sideways. This is the "CSS file responsibility audit" point carried over from the deprecated brainstorm, now with a concrete mechanism. Reinforced by Svelte's default **style-scoping** (styles in a `.svelte` file apply only to that component) — the type-class convention is the human-readable contract; Svelte's scoping is the machine-enforced boundary. Belt and suspenders.

*UI-implementation rule; no protocol/data implication.*

### N-021 — CSS layering: `normalize.css` as adapted layer-zero baseline

Styling applies in an explicit, ordered cascade:
```
layer 0  normalize.css        → cross-browser/webview baseline (the known-zero floor)
layer 1  component structural → function-critical, appearance-neutral CSS only (N-020/N-025), Svelte-scoped
layer 2  skin / theme          → XGen visual identity on top
```
`normalize.css` is the deliberate **point zero** — flatten inconsistency to a known baseline *before* any skin lands. Naming the order is itself the anti-drift move: everyone knows what is foundation and what is skin.

**Status:** not yet vendored in the repo (no `normalize.css` present as of this note). To be brought in **under `ui/`** when the CSS foundation is laid; moving/copying it into place is authorized.

**Adapted, not pristine (expect updates):** a vanilla `normalize.css` predates our situation. We ship into **Tauri webviews** (WebKitGTK / WKWebView / WebView2), not the open browser population — so part of the work is *trimming* defensive rules we don't need and *adding* webview-specific quirks; part is aligning its zero-point with the N-020 envelope + skin needs. Because it becomes a **maintained, modified** baseline, its deviations from upstream must be **recorded where the file lives** (header note or an "XGen deviations from upstream" comment block) — or a future reader assumes it is pristine and a drift trap is born.

**→ Refined by N-031 (2026-06-23):** the single `normalize.css` is realised as a two-file split — pristine `modern-normalize.css` (never edited, version-bumpable) + `xgen-normalize.css` (the adapted deltas, deviations recorded there). "Trim" becomes "override," not deletion.

*UI-implementation rule; no protocol/data implication.*

---

## 2026-06-20

### N-022 — Component taxonomy by data relation: data-independent / data-derived

Components are classified on one axis — relationship to data structures. The `data-` prefix names that axis explicitly (kills "derived" read as *computed state*, and "independent — of what?").

**data-independent** — an articulated control point wrapping a native HTML control. Keyed to an *interaction semantic* (boolean-toggle, single-select, free-text, numeric, action-trigger), not to any data structure. One semantic admits many shapes: a boolean-toggle renders as a classic checkbox *or* an on/off switch. Indexed by semantic so a builder picks an existing shape rather than reinventing one (serves N-019).

**data-derived** — a UI representation (materialization) of a defined data structure. Binding spans **Appendix I / G / O / none** — broader than I alone (console → O+G via D-056; outbox-card → event catalog §I.2). Some data-derived components are **ungrounded**: pure UI constructs with no data implementation (binding = none), recorded as such rather than forced onto a structure.

**Composition is an orthogonal property of data-derived, not a third class** (locked: option a). Folding assembly-depth into the class axis would mix two unrelated axes in one list. Each data-derived row carries a **composed-of** field:
- atomic — `identity-avatar ← IdentityRecord`
- composite — `spaces-panel ← [SpaceState]`, composed-of `container-list-item × N + section-header`

A composite is a **UI-purpose assembly that defines a compact form.**

**composed-of is membership only, never position.** The index answers *what a component is made of*; the component's **structural** CSS (layer 1, N-020/N-025) answers only *function-critical positioning*, all visual layout being skin (layer 2). Keeping layout out of the index keeps the form compact and preserves one-source-per-fact (no-drift).

*Shapes the forthcoming UI component index (N-019); UI-implementation model, no protocol/data implication.*

**Amendment (2026-06-20) — composition applies to both classes.** N-022 originally scoped composition as a property of *data-derived* only. Correction: composition is orthogonal to the data-relation axis and applies to **both** classes. A **data-independent composite** is several native controls assembled into one control point still keyed to a *single* interaction semantic, with **no** data binding. Worked case: a **combobox** is data-independent, keyed to *single-select*, composed-of (free-text input + single-select list + toggle), binding = none. (Same pattern: tag-select → multi-select; star-rating → single-select; password show/hide → secret.) The original "option a" stands — composition is never a third class, it remains a composed-of property — only its reach widens from data-derived-only to both classes.

### N-023 — Component base: shared logic module + thin envelope (composition, not subclassing)

Every component repeats the same envelope mechanics — root element, kebab type-class (N-020), class-merge for pass-through classes, the scoping contract. DRY-ing that is normally inheritance; Svelte has none. So the base is **composition, not subclassing** — two artifacts, both data-relation-agnostic (they serve data-independent *and* data-derived components alike):

**1. Shared logic module** (pure `.ts`, no DOM) — the mutual functions:
- `kebab(name)` — derives the N-020 type-class from the component name; mechanizes the "one identity in three places" derivation once.
- `mergeClasses(typeClass, passthrough)` — the root carries its own type-class **plus** any caller-supplied classes, never overwriting. This is the **"supplies the type-class, never erases it"** guarantee.
- any shared prop/lifecycle helper that genuinely recurs — kept minimal, nothing kind-specific.

**2. Thin envelope** — a Svelte **`use:` action** (`use:envelope={name}`) applied to the component's own root element. Chosen over a `<Base>` wrapper component because:
- thinnest — adds **no** extra DOM node, so it cannot fight Svelte's style-scoping (N-021 layer 1);
- it *augments* an existing element rather than owning one → structurally cannot erase the component's root identity;
- the component still writes its own root (`<div use:envelope={"outbox-card"}>`), keeping the N-020 name=file=class agreement visible at the call site.

**Out of base (explicitly):** CSS (structural lives with each component, appearance-neutral; all appearance is skin — N-025), the N-011 outer-ring/decorator slot (entity-specific, undesigned), anything keyed to data relation or kind. Base is envelope mechanics only.

**Alternative noted, not chosen:** a `<Base>` wrapper component — rejected for now (extra DOM node + class-ownership ambiguity). Revisit only if shared *structure*, not just behaviour, emerges.

**Index placement:** lives at `lib/components/base/` and registers in the component index (N-019) as **foundation/substrate** — not a data-independent or data-derived row, since it is neither; it is the substrate both classes sit on.

*UI-implementation rule; no protocol/data implication. Likely graduates to DECISIONS.md alongside the N-019/N-020 cluster once the library location is fixed.*

### N-025 — CSS: structural vs skin (skinability requires no local appearance)

CSS splits by purpose, and the purpose decides where it may live. **Three sources only:**
- **layer 0 — `normalize.css`** — cross-webview baseline (N-021).
- **component-local structural CSS** — the constrained exception (below).
- **layer 2 — one skin file** — all appearance, keyed by type-class, legally overrides.

**Structural CSS** — rules *without which the component does not function* (a dropdown list must overlay, not displace; a toggle must sit within its field). May live with the component, and must be **appearance-neutral**: positioning / overlay / flow only — never colour, spacing-as-taste, borders, typography.

**Skin CSS** — all appearance. Lives only in the skin file, keyed by type-class (`.combobox { … }`). Never local.

**The test:** *if the skin tried to override this rule, would it break the component?* Yes → structural, may stay local. No → it is appearance → skin, never local. Anything appearance baked locally is a **skinability obstacle** — the skin cannot override what the component hardcoded.

**Supersedes** the earlier "layout lives in the component's own `.css`" wording (N-020/N-021/N-022/N-023): "layout" is not one unit — its load-bearing part is structural (local-ok), its visual part is skin (must externalise).

*UI-implementation rule; no protocol/data implication. Likely graduates to DECISIONS.md with the N-019/N-020/N-021 CSS cluster.*

**→ Operationalised by N-031 (2026-06-23):** the structural-vs-skin test is restated as the remove-the-rule litmus, with the baseline second cut (element-generic → normalize L0 / component-specific → local L1).

---

## 2026-06-21

### N-024 — Debug field-accessibility: dev-only registry riding the envelope

Fills the reserved **N-024** slot (authored 2026-06-21; the number was reserved because the CDP harness + component index already forward-referenced it). The harness (`tasks/CDP_DEBUG_HARNESS.md`) reads two producers that did not yet exist: `window.__XGEN_DEBUG__` (whole-state dump) and per-component `data-debug-id`. N-024 is the **UI-side contract that produces both — dev-only.** Gate cleared: harness built + verified (client 9222 / node 9322; modes eval/state/console).

**Core move — ride the N-023 envelope, never add a second wire.** Every component root already calls `use:envelope`. A **dev-gated branch** in that same action (a) stamps `data-debug-id`, (b) registers the component's live state-getter into `window.__XGEN_DEBUG__`. One root wire; the N-020 type-class feeds the debug-id; production tree-shakes the whole branch (`import.meta.env.DEV`), honouring the harness's non-negotiable release-safety.

**State exposure = explicit getter (decision (a′), LOCKED).** A Svelte `use:` action receives the DOM node, **not** the component's reactive `$state`. So a stateful component hands the envelope a getter; the action never reaches into reactive scope itself. Chosen over: a registry the component pushes to directly (forks the one-wire win), and DOM-only reads (thin — "DOM-accessibility", not "field-accessibility"). The getter is `$state.snapshot(...)`-wrapped so CDP `returnByValue` receives de-proxied JSON.

**Registry is a singleton with methods + per-component isolation — not a bare object.** A bare object of getters would not survive `JSON.stringify` (getters are not invoked) and one throwing component would abort the whole dump and blind the harness. Instead:

```ts
// lib/components/base/debug.ts — installed lazily; no app-entry edit
type Entry = { type: string; get: () => unknown };
const registry = new Map<string, Entry>();

function readOne(id: string) {
  const e = registry.get(id);
  if (!e) return null;
  try { return { type: e.type, state: e.get() }; }
  catch (err) { return { type: e.type, error: String(err) }; } // isolation
}
function ensureInstalled() {
  const w = window as any;
  if (w.__XGEN_DEBUG__) return;
  w.__XGEN_DEBUG__ = {
    ids:      () => [...registry.keys()],
    get:      (id: string) => readOne(id),
    snapshot: () => Object.fromEntries([...registry.keys()].map(id => [id, readOne(id)])),
  };
}
export function register(id: string, type: string, get: () => unknown) { ensureInstalled(); registry.set(id, { type, get }); }
export function unregister(id: string) { registry.delete(id); }
```

**Three read shapes** map to the harness's capabilities: `snapshot()` (whole dump) · `get(id)` (single component — pairs with the `data-debug-id` the harness finds in DOM) · `ids()` (enumeration).

**Envelope branch (dev-gated; the `debug` import is referenced only inside the guard, so it drops in prod):**

```ts
// lib/components/base/envelope.ts (extended)
import { register, unregister } from './debug';   // static — referenced only inside the DEV guard, so it tree-shakes out of prod
let ordinal = 0;
type Param = string | { name: string; id?: string; debug?: () => unknown };

export function envelope(node: HTMLElement, param: Param) {
  const { name, id, debug: getState } =
    typeof param === 'string' ? { name: param, id: undefined, debug: undefined } : param;
  const typeClass = kebab(name);
  node.className = mergeClasses(typeClass, node.className);   // supplies, never erases (N-023)

  if (import.meta.env.DEV && getState) {
    const debugId = `${typeClass}#${id ?? ++ordinal}`;        // N-011 stable id, else ordinal
    node.setAttribute('data-debug-id', debugId);
    register(debugId, typeClass, getState);
    return { destroy() { unregister(debugId); } };
  }
  return {};
}
```

**`data-debug-id` value = `type-class#<id>`** — N-011 stable ID (`identity_id` / `space_id`) where the component is an entity, else a per-type ordinal. One string locates the node in DOM **and** keys the registry — the harness's two read paths converge on it.

**Component opt-in is one greppable line** (N-019 honesty — state exposure is visible at the call site, never hidden):

```svelte
<script lang="ts">
  let { space }: { space: SpaceState } = $props();
  let expanded = $state(false);
  const dbg = () => $state.snapshot({ space, expanded });
</script>
<div use:envelope={{ name: 'spaces-panel', id: space.space_id, debug: dbg }}> … </div>
```

A component with no state passes the string form (`use:envelope={'icon-button'}`) and never appears in the registry — correct: nothing to read.

**Index placement.** This is envelope/substrate behaviour (N-023), not a component — it registers in the component index against the **base/substrate** row as the dev-debug responsibility, never a data-independent/data-derived row.

**Cross-file owe (discharged with this note).** `tasks/CDP_DEBUG_HARNESS.md` had the state read as `JSON.stringify(window.__XGEN_DEBUG__)`; (a′) makes it `…__XGEN_DEBUG__.snapshot()` plus the `get(id)` / `ids()` verbs — amended in harness v1.1 (companion edit, same arc).

**Open / parked:**
- ~~Static vs dynamic import of `debug.ts` inside the guard~~ — **settled static (2026-06-21, Commit C).** A Svelte `use:` action is not async, so the originally-shown `await import('./debug')` could not compile; the **static** import is referenced only inside the `import.meta.env.DEV` guard and tree-shakes out of production builds. Shipped at `ui/common/lib/components/base/` per D-095.
- Whether non-entity components want a stable id (e.g. slot name) rather than an ordinal — revisit if ordinal churn makes harness reads flaky in practice.
- Deep-object render policy for `snapshot()` (shallow + expand) — already parked in the harness doc; a presentation choice, not a contract change.

*UI-implementation rule; no protocol/data implication. Likely graduates to DECISIONS.md alongside the N-019/N-020/N-023 base cluster once the library location is fixed.*

### N-026 — UI source-tree tiers grounded → D-095 (common / core / client / node / assets)

The UI tier structure is now grounded in DECISIONS.md as **D-095**: the `ui/` subtree mirrors the four core crates 1:1.

- `ui/common/` ≈ `xgen-common` — shared **code** substrate (envelope mechanics, helpers); alias `$common`. The N-023 base (`logic.ts` / `envelope.ts` / `debug.ts`) lives here.
- `ui/core/` ≈ `xgen-core` — the **reference component library** (the di / dd components, N-022); built on `common`; alias `$core`.
- `ui/client/`, `ui/node/` ≈ `xgen-client` / `xgen-node` — thin shells composing `core` (rename of `dev_core_ui/{client_ui,node}`).
- `ui/assets/` — shared **static** files (fonts, logos); a distinct axis from `common` (code vs files). "shared" dropped from the name since everything under `ui/` is shared by definition.

**Resolves a recurring drift:** the substrate's home mutated across proposals (inside `dev_core_ui` → hoisted peer → sibling) precisely because the tier structure was never written. D-095 fixes it; the component index (N-019) gains a tier marker (`common` / `core`) so each entry's home is unambiguous.

**Consequence for the base substrate (N-023):** `base/` lands at `ui/common/lib/components/base/`; the N-024 envelope debug-branch ships there. Components built on it live under `ui/core/lib/components/`.

**`common` vs `core` (load-bearing):** a component never lives in `common`; a bare helper never lives in `core`. `common` = shared behaviour both apps depend on; `core` = the sample components built on it.

Physical folder moves + build-wiring repoint land in the restructure commit following this grounding; `dev_core_ui` is retired (the CLI tests it gated are complete). The Ch3 module-architecture OQ that names "Phase 2 client UI structure" is a *different* sense (module-to-UI extensibility) and is **not** resolved by D-095.

*UI-implementation grounding; pointer to D-095. No protocol/data implication.*

### N-027 — Substrate proof: first `core` component built + live CDP registry verified (both apps)

The base substrate (N-023/N-024) and the D-095 tier wiring are now proven end-to-end, not just type-clean. M-RP2.3 built the first real `core` component and drove the full `tauri dev` + CDP loop in **both** apps.

**Built.** `toggle` — `ui/core/lib/components/data-independent/toggle.svelte` (N-022 boolean-toggle; N-020 atomic, root native `<input type="checkbox">`; type-class supplied by `use:envelope`, not hardcoded). Opt-in debug getter `() => $state.snapshot({ checked })` (N-024). First row in the new **Built components** registry of `xgen-ui-components.md` (Status flipped PENDING→ACTIVE).

**Wired + proven.** A throwaway `<Toggle id="demo">` demo instance mounted in both shells via `$core` (`app_client.svelte`, `app_node.svelte`); the Quit / Shut-Down buttons (still the pre-N-020 throwaway `Button.svelte`) kept, since the windows are `decorations:false` and would otherwise lose their only close affordance. Verified live:
- aliases `$common` / `$core` resolve in real Vite builds;
- DOM carries `class="toggle"` + `data-debug-id="toggle#demo"` (the envelope stamp);
- `window.__XGEN_DEBUG__.snapshot()` returns real `{"toggle#demo":{"type":"toggle","state":{"checked":false}}}`;
- flip → re-dump → `{checked:true}` — confirms the getter reads **live reactive scope**, the core N-024 claim;
- client (9222 / :5173) and node (9322 / :5174) both green.

**Tooling fixes shipped alongside (arc-local, not promoted):**
- `run-client.ps1` / `run-node.ps1` — both pointed `$TauriDir` at a non-existent `…/src-tauri`; the Tauri crate is the `xgen-client` / `xgen-node` root itself. Fixed. Added a dev-only `-Debug` switch injecting `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=<9222|9322>` (client uses a clean param block; node reads `$args` to preserve `--service` forwarding).
- `cdp-debug.ps1` `state` mode evaluated the pre-v1.1 bare `JSON.stringify(window.__XGEN_DEBUG__)`, which stringifies the singleton's methods to `{}` once the registry exists. Corrected to `…snapshot()` (the harness doc was already v1.1-correct; only the script had drifted). The harness DoD's last UI-gated box is now ticked for both apps (real-registry path); the release-inert box stays open — no release build was run.

**Deferred (next step, not this arc):** retire the throwaway `Button.svelte` in both shells when the first `core` button lands and replaces the close affordance. Reuse-not-rebuild (N-019) already demonstrated — one `toggle` consumed unchanged by both apps.

*UI-implementation record. No protocol/data implication. The base cluster (N-019/N-020/N-021/N-023/N-024/N-025) remains a candidate to graduate to DECISIONS.md; the `-Debug` / harness specifics stay arc-local (below the D-069 promotion bar).*

---

## 2026-06-22

### N-028 — Second `core` component (`button`) + sampler design-of-record + component layer-phase taxonomy

M-RP2.4 closed (J-405). A deliberate **pipeline-tuning pass** — one component, not a feature sprint — to shake out the repeatable authoring loop (author → wire via `$core` → live-verify in both apps → record) and surface friction while it is cheap.

**Built.** `button` — `ui/core/lib/components/data-independent/button.svelte` (N-022 action-trigger; N-020 atomic, root native `<button>`; type-class supplied by `use:envelope`, not hardcoded; no local CSS). Event-**out** (`onclick`, no `bind`) — the complementary envelope path to the toggle's event-in `bind:checked`. Props: `label` / `onclick` / `disabled` / `id`. N-024 opt-in debug getter `() => $state.snapshot({ clicks, disabled })`.

**Verified live, both apps** (real Vite + `tauri dev` + CDP, not just `tsc`):
- client (9222): `snapshot()` → `{"toggle#demo":{"type":"toggle","state":{"checked":false}},"button#quit":{"type":"button","state":{"clicks":0,"disabled":false}}}`
- node (9322): `{"toggle#demo":{"type":"toggle","state":{"checked":false}},"button#shutdown":{"type":"button","state":{"clicks":0,"disabled":false}}}`
- DOM carries `class="button"` + `data-debug-id="button#{quit,shutdown}"`; clicking Quit / Shut-Down closes the window — the restored close affordance. The registry now holds **two** components from **two** semantics (bind-in + event-out) cleanly side by side — the N-023/N-024 substrate **generalizes**, which was the real question behind the pass.

**Retired.** Both throwaway `ui/{client,node}/src/lib/Button.svelte` (Svelte-4, pre-N-020) deleted; both shells consume the one `core` button via `$core` (N-019 reuse, second instance after `toggle`). The `demo` toggle stays in the shells for now — its retirement is sampler-bound (below).

**Findings (the pipeline-tuning payoff):**
1. **Terminal-action can't self-redump.** The real button's `onclick` exits the app, so the `clicks` 0→1 delta cannot be re-dumped on the same instance — firing it ends the session. The counter still honestly proves event-out registration + that the getter reads `$state` (baseline-observable); the *live-reactive-read* proof is inherited from `toggle` (same envelope path). A non-terminal demo button would show the delta but was declined to avoid re-adding shell cruft we are routing to the sampler.
2. **Pre-skin ≠ bare.** The button renders dark/rounded/padded, not the bare normalize/native control the chat-preview suggested, because each shell already carries a global `button {}` rule that the type-class-less `<button>` inherits. An N-025 wrinkle (appearance living in a global rule rather than the one skin file) for the skin pass to reconcile; not a defect.
3. **Dev-exit WebView2 warning.** Node `-Debug` close prints `[ERROR …window_impl.cc:172] Failed to unregister class Chrome_WidgetWin_0. Error = 1412` — a known-benign WebView2/Chromium teardown log on the dev-only remote-debug path. Watch-item; confirm `-Debug`-generic (not a leak) by running once without `-Debug` if it recurs. Not gating.
4. **Run-script ergonomic.** `run-{client,node}.ps1 -Debug` blocks the terminal (it *is* `tauri dev`); `cdp-debug.ps1` must run in a **separate** terminal. Pasting the dump after a blocking `run-*` in the same block queues it until the app exits (in a subdir) → spurious "not recognized". For future sessions: dump in its own terminal.

**Component layer-phase taxonomy (A/B/C) — new, orthogonal to di/dd.** A component's *phase* is the build-layer its binding demands, decided the moment its binding is filled:
- **A — pure Svelte:** self-contained in the webview, state in runes, nothing crosses out (`toggle`, `button`, most di).
- **B — Svelte + Tauri:** reaches the Tauri IPC boundary but needs **no new Rust logic** — calls existing commands / listens to existing events (color picker → native dialog; temperature → `xgen-temperature-update` + `get_pacing_state`, the unused N-006 surfaces).
- **C — all three layers:** requires **new Rust behaviour in a crate**, not just an IPC call (auth / EventStore *system*-module management UI — anything where Rust does real work the UI merely surfaces). Deferred.

Phase is a property of the component (its binding), not a schedule. Recorded as a **Phase column** in the Built-components registry, beside the `common`/`core` Tier marker (N-026). A component carries *both* coordinates: di/dd class **and** A/B/C phase (`toggle`/`button` = di · A).

**Standing practice — bare chat-preview per component.** During discuss/author, a disposable `all: revert` chat-side preview (structure + interaction + the debug payload it would emit) is a standing authoring aid — the *appearance* register that cannot show skin. Complemented by the in-app sampler (below), the *truth* register where skin actually lives.

**Sampler (settled this session; design-of-record — implementation deferred, M-RP2.5+).** A dedicated dev app whose whole job is exhibiting `core`/`common` components against skins — the proper successor to the retired `dev_core_ui`. Keeps the real shells clean (only real affordances); the `demo` toggle migrates here.
- **Home:** `ui/sampler/` — an explicit **dev-tool dir, exempt from the D-095 1:1 crate-mirror**. *Owe (routed to Joe): a one-line D-095 footnote stating dev-tooling dirs under `ui/` are mirror-exempt, so the tier map stays clean.*
- **Build-phases A/B/C, trigger-driven (not dated):** Phase-A sampler = Vite-only (instant HMR; skin-swap + size/text override). Phase-B = thin Tauri wrap, unlocked by the **first Phase-B component** (color-picker native dialog, or a temperature **synthetic-feed harness** injecting `xgen-temperature-update` so the thermometer is exercisable with no live federation). Phase-C deferred with its components. The sampler's own build-phase gates which component-phases it can host.
- **Two jobs, both designed-for:** the *read* side (N-024 registry — snapshot state out) **and** the *write/inject* side (synthetic feed — push fake state in). Same instinct, opposite direction; design for both now so the temperature case is not a retrofit.
- **IA — matrix-of-record, tabbed-by-phase presentation (N-014 swappable):** the logical IA is the **class × phase** matrix + a Combined/other section. Leading presentation: **phase = the tab axis** (the load-bearing one — it gates sampler-build reachability; gated phases render as **locked/disabled tabs**, cleaner than greyed bands); **di/dd = an in-pane segmented `[All|di|dd]` filter** (default All, so within-phase skin-swap shows both classes; **sub-tabs reserved as a volume-triggered per-pane upgrade**, never a blanket second tab level — di/dd is a label, not a gate, so it does not earn a tab level by default). **Combined tab** = the skinned all-at-once together-gallery (where cross-component skin comparison lives, since per-phase tabs would otherwise hide it) + composites (N-022 combobox / tag-select) + real assemblies (spaces-panel). **Skin + size = global chrome above the tab bar**, persisting across tabs — one skin re-renders every visible component, the single thing the sampler does that the chat-preview cannot.
- **Index-driven (no drift):** the sampler materializes the Built-components registry — a cell renders its chips from the index; an unpopulated cell says "none yet," an unreachable band says "requires sampler Phase B." No pre-built empty panes (D-065).
- **Killer feature:** live skin A↔B swap against one component tree is the only surface that actually *exercises* the N-021/N-025 layered-CSS model (prove skinability + that appearance is fully externalized).

**GPL-overview flag (routed to Joe — not absorbed).** `ui/core/` ≈ `xgen-core` (GPL-2.0-or-later, D-044), so the `core` reference components are **GPL code**. That makes the Built-components registry (`xgen-ui-components.md`) the *licensed-corpus overview catalogue* of record, and the sampler its live visual face (index-driven, so one derives from the other — D-065 no-drift). Possible `DECISIONS.md` touch tying the `ui/core/` = GPL boundary to a catalogue-completeness duty (the lens that owns D-044). Flagged for Joe to route; not silently recorded as settled.

**→ Resolved (Joe, 2026-06-22) — no decision needed.** No GPL question arises during development: created code is locked under the single development license (BSL 1.1, per every file header), and GPL-2.0-or-later becomes effective on project handover per the fundamental records (the BSL→GPL conversion). The `ui/core/` = GPL boundary is therefore a *future-state* property, not a present catalogue-duty trigger — no `DECISIONS.md` touch. The registry-as-catalogue / sampler-as-visual-face framing (D-065 no-drift) stands on its own, independent of licensing.

**Addendum (same session) — Chat self-drove the CDP loop; one automation race found.** After the records above were written, Chat ran the *entire* verification procedure itself via Windows-MCP PowerShell (no Joe relay): launch each app detached (`Start-Process … run-{client,node}.ps1 -Debug -WindowStyle Minimized`) → poll the CDP port via a `TcpClient` loop → `cdp-debug.ps1 -App {client,node} -Mode state` → read the registry → clean up (`Stop-Process xgen-client/xgen-node` + the Vite port owners + the spawned consoles; ports 9222/9322/5173/5174 all confirmed closed afterward, no orphans). **Both dumps reproduced identically** to Joe's hand-runs (`button#quit` / `button#shutdown` → `{clicks:0,disabled:false}` + `toggle#demo`) — independent corroboration of the M-RP2.4 proof, not a new claim.

**Finding — poll-too-early race (CDP port opens before the registry populates).** Node's *first* automated dump returned `null` (`window.__XGEN_DEBUG__ … no registry yet`); a 3-second re-dump returned the correct `button#shutdown`. Cause: the harness waits for the CDP **port** (9322) to open, but WebView2 opens that port *before* the Svelte app has mounted and registered its components. A human running by hand never sees this (typing latency covers the gap); an automated poll fires the dump in the window between port-open and app-mount. **Not a defect — a timing gap in the *procedure*.** Rule for any self-drive harness: poll the port, **then retry the `snapshot()` dump until `__XGEN_DEBUG__` is non-null** (bounded retries), rather than treating port-up as ready. Candidate hardening for `cdp-debug.ps1` itself: a built-in retry-until-non-null with a timeout, so the recipe is correct regardless of who drives it.

**Working-mode implication.** The CDP harness + run scripts + the N-024 registry are drivable by Chat directly (Windows-MCP), so the per-component loop **author → wire → self-verify live in both apps → record** can run end-to-end without Joe relaying dumps — Joe reviews and pushes. A real shift in the three-agent split for the component-buildout arc: Chat's scope on active UI arcs now includes *running* the live CDP verification, not only authoring records from Joe-supplied output. (Joe-locked working preference, not a protocol decision.)

*UI-implementation record. No protocol/data implication. The sampler design likely graduates to its own design doc once it grows past a note; the A/B/C phase taxonomy + the chat-preview practice are candidates for the base-cluster DECISIONS.md graduation alongside N-019/N-020/N-021/N-023/N-024/N-025. The `Chrome_WidgetWin` + run-script ergonomics stay arc-local.*

### N-029 — Third `core` component (`textfield`): string bind-in path proven; CDP input-dispatch verify subtlety

M-RP2.5 closed (J-407). The third `core` component, `textfield` — the **string bind-in** binding shape, completing the trio the substrate now demonstrably generalizes across: toggle (boolean bind-in), button (event-out), textfield (string bind-in).

**Built.** `textfield` — `ui/core/lib/components/data-independent/textfield.svelte` (N-022 free-text single-line; N-020 atomic, root native `<input type="text">`; type-class via `use:envelope`, not hardcoded; no local CSS). `type` is **fixed, not a prop** — the deliberate boundary that keeps one-semantic-per-component (N-022): email/url/tel are constrained-text, password is secret, number is numeric, each its own component; search is a *shape* of free-text, deferred as a variant. Native-state surface only — `value` (`$bindable`, `bind:value`), `placeholder`, `disabled` (inert + skin-greyed), `readonly` (shown/selectable, not editable — distinct from disabled, not greyed), `id`, `pattern` (native `:invalid` for template-mismatch — consumer owns the rule, skin owns the look; no bespoke validation engine), `name`. N-024 getter `() => $state.snapshot({ value })`.

**Processor-ready, not processor-bearing.** A text processor (emoji-combo, pattern formatting) is designed to live **once** in `common` as a `use:` action and layer onto *any* text-bearing tag (`<input>` and `<textarea>` alike) via `use:processor={pairs}` — the field neither contains nor blocks it. The DRY requirement (same processor on textarea, exchangeable pairs) is satisfied by composition, not duplication. Built separately, later. The same logic keeps the clear/copy-button version out of this atomic: that is a `<div class="textfield-group">` composite (textfield + icon-button ×1–2), a future row — a root-tag boundary (the legitimate split), as opposed to the rejected "simple vs stateful twin" split (which would be two rows for one semantic).

**Verified live, both apps** (real Vite + `tauri dev` + CDP; Chat self-drove the loop per the N-028 working mode):
- client (9222): baseline `{"toggle#demo":…,"textfield#demo":{"type":"textfield","state":{"value":""}},"button#quit":…}`; after dispatch → `textfield#demo` `{value:"hello"}`.
- node (9322): baseline `…"textfield#demo":{…"value":""}…"button#shutdown":…`; after dispatch → `{value:"world"}`.
- The registry now holds **three** components across **three** binding shapes side by side. The `value` 0→delta **re-lands the live-reactive-read proof on the bind-in path** — the proof the terminal-action button could not self-redump (N-028 finding 1).

**Finding — CDP input-dispatch subtlety.** Driving `bind:value` from CDP requires a **real dispatched `input` event** (`el.value="…"; el.dispatchEvent(new Event("input",{bubbles:true}))`), not a bare `el.value=` assignment — Svelte's `bind:value` updates the rune from the `input` handler, so a silent property set leaves the rune (and the registry) stale. Sibling-shape to the N-028 poll-race: a correctness detail in the *verify procedure*, not the component. The self-drive orchestration also reconfirmed the N-028 race fix (retry `snapshot()` until `__XGEN_DEBUG__` is non-null — client needed 2 retries, node 1) and cleaned up with zero orphan ports (9222/9322/5173/5174 all closed).

**Expected pre-skin wrinkle (note).** Like the button, the shells likely carry global `input {}` rules, so pre-skin the field is not the bare normalize baseline — reconcile at the first skin pass (N-025).

*UI-implementation record. No protocol/data implication. `textfield` joins the di·A built set; the `use:processor` action + the `textfield-group` composite are queued follow-ons.*

### N-030 — Shape families on the built components (`button` toggle-mode + icon shape; `toggle` checkbox/switch); `label` & `image` as display-di; combobox decomposition

Design conversation following J-407, ahead of `select`. No code yet — records the decisions so they are not re-litigated, and flags that two **already-built** components (`button`, `toggle`) gain additive surface (a retrofit, not new components). Driven throughout by the root-tag lens (N-020) and shape-is-skin (N-019/N-025).

**1. The boolean family splits by root tag, not by look.** One boolean ("is-down/checked") can be materialized several ways; the **root tag** decides which *component* owns each:
- `<input type="checkbox">` → **`toggle`**. Shapes: classic **checkbox**, **switch** (sliding pill), and a checkbox *styled* as a pressable box — all the same `toggle` component, differing only by **skin** (N-019/N-025); the switch additionally carries `role="switch"` so the accessibility tree says "switch" when the skin does.
- `<button>` → **`button` in toggle-mode** (§2). A true button-style toggle (`<button aria-pressed>`) is **not** a shape of `toggle` — different root tag → it belongs to `button`. Tag lens: same tag + same semantic = same component; different tag = different component.

So "the third boolean form" (button-toggle) lives with `button`, not `toggle`. UX-identical to a styled checkbox; the choice between them is *form semantics* — submits a value in a form → checkbox (`toggle`); in-app pressed action → `button` toggle-mode.

**2. `button` retrofit (additive, on a shipped component).** When next touched, `button` (M-RP2.4) gains:
- **`ariaLabel`** (optional) → `aria-label` when set. Required-in-practice for the **icon shape** (an icon-only button has no text name; without a label it is an unnamed control). Icon-button is therefore a **skin shape of `button`**, not a new component — same `<button>` root, same action-trigger semantic.
- **`pressed` / toggle-mode** — one inner boolean ("is down") whose *lifetime* a mode flag governs: **momentary** (default) = true only while pressed, click emits `onclick` (today's behaviour, unchanged); **toggle** = click *latches* the bool (stays until the next click), exposed bind-out as `pressed`. `aria-pressed` is reflected **only in toggle-mode** (it would be a lie on a momentary button). So `button` spans both binding shapes by mode: event-out (momentary) and boolean bind-out (toggle).
- The pressed/down *look* is **skin**, keyed on `[aria-pressed="true"]`; the component exposes state, the skin draws it.
- All three (`ariaLabel`, `pressed`, shape variants) are **additive** — the existing Quit / Shut-Down stay momentary and untouched.

**3. `toggle` retrofit (catalogue only).** No structural change; its shape family (checkbox / switch) is named for the registry and lands as skin variants with the first skin file.

**4. One source, many projections (the read/drive model).** A control's inner boolean is the single source of truth; everything else is a *projection*, never a second copy: the `bind:` prop → the production program reads/writes; `__XGEN_DEBUG__` (N-024) → tooling/harness reads; `aria-*` → assistive tech reads; the skin → the eye reads. ARIA is **output to the accessibility layer**, not the value the app consumes and not a state to maintain independently — reflect it from the bool, never hand-manage it (drift is exactly what ARIA exists to prevent). This is why every component is machine-readable/drivable by design (binding for the app, registry for tooling) — the reason the envelope/debug substrate (N-023/N-024) was built first.

**5. `label` and `image` are display-kind di components (correcting an earlier miscategorisation).** "Has a value" and "is interactive" are different axes. Label and image **carry a value** (label: `text`; image: `src`) plus the universal props (size → skin, `disabled`, `id`) — full components, just **read/display** (value in, no user-driven event-out) rather than input. They are **data-independent** (plain string/path, no protocol schema behind them), so they sit in the **di family** alongside toggle/button/textfield — not a separate "no-value"/structural bucket. Both are first-class near-term basics. (Delphi/JavaFX analogues: `TLabel.Caption`, `ImageView` — value property + universal props.)
- `label` — value `text` + `for` (the `for`/`id` association is the contract it owns); atomic `<label>`. A labelled-field is a *later composite* (label + textfield), after the atomic.
- `image` — value `src` + `alt`; atomic `<img>`. **Phase A** as a primitive (bundled / URL src); the *source* can pull a usage to **Phase B** — a user-picked local path or a node blob needs Tauri (`convertFileSrc` / asset protocol). Same shape as the file dialog: A primitive, B when the source demands it.

**6. Combobox = composite of `textfield` + `datalist`, not `textfield` + `select`.** Three tiers under "choose from options", separated by root tag:
- **`select`** — native `<select>`, the collapsed **dropdown / pick-only** combo box (no typing). Atomic, di·A. The locked next basic.
- **list-box** — expanded `<select size=N>` / `<select multiple>` — a *shape* of `select` (skin), basically free once `select` exists.
- **`combobox`** (type-or-pick) — native basis is **`<input list>` + `<datalist>`**, *not* `<select>` (you cannot nest a `<select>`'s own popup as an input's suggestion list; `<datalist>` is the element built for that). A **di·A composite** (per the N-022 amendment): keyed to single-select, composed-of textfield + datalist, binding = none. A natural early composite — composes `textfield` (built) + a `datalist`, needs nothing new.
- **rich list-view / editable combobox** (custom rows, filtering, columns) — a `<div>`/`<ul>` **data-derived composite**, far later. Native `<select>`/`<option>` hold plain text only; structured rows force the `<div>` root.

**Meta-principle reaffirmed.** Native HTML is starting material, not master: follow it where it gives a clean, accessible primitive; depart where it is limiting (rich combobox → `<div>`; file dialog → Tauri). The root-tag discriminator records *when* we departed (native = atomic, `<div>` = composite); A/B/C records *how far* (Svelte/Tauri/Rust). Phase is honest cost, never a UX cap (Joe: look + intuitive function first; if a basic needs B/C, build it there — the named exception to basics-first).

**Build-order consequence.** `select` remains a queued basic (atomic, di·A). The **`button` retrofit** (`ariaLabel` + `pressed`/toggle-mode) is a near-term pass on a shipped component — best paired with the **first skin file**, where icon / switch / pressed shapes actually render. `label` and `image` join the di basics queue (display-kind). `combobox` + `textfield-group` are the first composites. (Joe's reframe: address the changes to done components before opening the next one — so the `button` retrofit + first skin file now lead the queue, ahead of `select`.)

*UI-implementation record. No protocol/data implication. The `button` retrofit is the first reopen of a shipped component — recorded so it is not lost. Registry candidates when each lands: button shape/mode note, toggle shape note, label/image display-di rows, combobox composite row.*

## 2026-06-23

### N-031 — CSS source stack locked: two-file normalize + per-component construction + one skin; the remove-the-rule litmus; vocabulary saturation

Design conversation following N-030, settling the CSS architecture before the first skin pass. No code yet — locks the model so M-RP2.7 has a fixed target. **Refines N-021** (its single "normalize.css" becomes a two-file split) and **operationalises N-025** (the structural-vs-skin test). Same layering instinct as N-021/N-025.

**The source stack — three global files + one per-component channel (4 sources, ordered cascade):**
```
L0  modern-normalize.css   pristine upstream cleaner          per-tag, global, shared, NEVER edited (stays version-bumpable)
L0  xgen-normalize.css     our adapted element-generic floor  per-tag, global, shared, deviations recorded in-file
L1  <style> in each .svelte   construction / structural       per-component, as-needed, Svelte-scoped, appearance-neutral
L2  skin.css               all appearance                     one file, keyed by type-class, the single removable layer + live-swap target
```
The two L0 files together are the "adapted, maintained baseline" N-021 named; the pristine-import + adapted-deltas split is its upstream-bumpable realisation. **"Trim" becomes "override":** xgen-normalize cannot delete an upstream rule, only re-set the property to neutralise it — genuine deletion would mean forking modern-normalize, which we do not do.

**The process — two questions, in order, for any rule a component needs:**
1. **Remove-the-rule litmus (baseline vs skin).** Delete the rule: does the component break / stop making sense, or just go plain? Breaks → baseline (L0/L1). Only plain → skin (L2). Appearance-as-taste (colour, spacing-as-taste, borders, the look) is always L2.
2. **If baseline — generic vs specific.** Is it the sane floor for a *bare tag*, true of every use of that element regardless of which XGen component wraps it? → normalize (L0), keyed per-tag, shared. Is it about *this* component's internal structure (its parts' positioning / overlay / flow)? → component-local (L1), keyed to the type-class, scoped in the `.svelte`.

L0 + L1 together = the functional skinless app (works and makes sense with zero skin loaded — the design floor); L2 is the only layer removable while leaving a working app. **Construction CSS is the L1 `<style>` block inside the single-file component (N-019), not a separate per-component file, and is frequently empty** — toggle/button/textfield carry none today; it appears only when a component has internal parts needing positioning.

**Normalize is per-tag, not per-component.** Many components on one tag share one floor: `button` + the icon-button shape + a toggle-mode button are all `<button>` → one `button{}` rule in xgen-normalize serves all three. There is no normalize entry per component.

**Saturation (Joe's observation) — the stack converges to "pick from already-defined," each layer by a different mechanism:**
- **L0** saturates fastest/hardest — only ~15 native interactive tags (the di catalogue); once each has its floor, L0 is essentially done; new components rarely add a tag.
- **L1** barely grows — most components have none; a genuinely new internal structure adds a pattern once, and the next component of that shape reuses it (N-019), not a new rule.
- **L2** saturates by **vocabulary, not files** — early skin work *defines* the primitives (token scale, accent tokens, pressed / focus / disabled treatments); afterwards a new component's skin is mostly assembled from existing skin vocabulary. The file grows in *coverage* (more type-classes addressed); its *vocabulary* (tokens + shared treatments) plateaus. N-019 write-once/reuse applied to styling.

**Consequence — the first skin pass (M-RP2.7) is a vocabulary-founding pass, not a quick three-component skin.** Getting the L2 primitives right early makes later components cheap. The duplicated `:root` token block currently in both shells' `app.css` promotes into the skin layer as the named vocabulary; the generic reset bits (`*{box-sizing}`, `button{appearance:none}`, `img`, `p`) consolidate into `xgen-normalize.css` (the adapted baseline, deduped from both shells); appearance (`button{background…}`, `.primary-*`) moves to `skin.css`; `app.css` is gutted to shell chrome only. That closes the N-028/N-029 global `input{}`/`button{}` wrinkle in one pass.

**Open (for the M-RP2.7 design walk / runbook):** `skin.css` home/path (candidate `ui/assets/`, following the `skin-*.css` + `tokens.css` precedent in `ui/templates/skeleton/` and `ui/backup/run_1.0/`); whether `modern-normalize.css` is wired at all today (the shells' hand-rolled `app.css` reset currently does the L0 job); accent gold/blue as one skin + shell-set token vs two skins (ties to N-030 §2).

*UI-implementation record. No protocol/data implication. Refines N-021 (two-file normalize split) and operationalises N-025 (the litmus). Candidate to graduate to DECISIONS.md with the N-019/N-020/N-021/N-025 CSS cluster.*

### N-032 — Display-di component identities locked (conceptual): `label` / `paragraph` / `image`; the edit-vs-render processor axis; parked richtext

Design conversation (conceptual only — none built; all sit behind M-RP2.7 and `select`). Settles the **display half** of the di model: the three built components (`toggle`/`textfield`/`button`) are all *interactive* (input/event, live getter state); these three are **display-kind di** — value-carrying but **read-only** (no event-out, no bind-in of user action; getter exposes the value, nothing live). Identity by root tag stands (N-030); all three are atomics (native root tag).

**`label` — root `<label>`.** A short caption that *names another control*. Chosen over `<span>` (inline, semantically empty — "marks nothing standalone") and over `<p>` (that is a different component, below). **Association (`for=`/nesting) is a COMPOSITE concern, not the atom** — the atom is association-agnostic; the pairing is wired by the group (`textfield-group`), leaning **implicit nesting** (`<label><Control/></label>`, no id generation) over explicit `for`/`id`. A standalone `<label>` with no control is valid-but-inert — a tolerated edge case, since `label` almost always sits beside a control. Block-level is a **skin** concern (`display:block` in L2), not a tag concern.

**`paragraph` — root `<p>`.** A **single paragraph** of prose (NOT multi-paragraph / rich document). Named `paragraph` (1:1 with the tag) over `text` (too generic, reads as a primitive) and `textblock` ("block" oversells single-paragraph). Scalar string value. **Inline-mark formatter SEAM reserved, not built:** the component renders through a transform hook that is the identity/pass-through today; a future limited inline formatter (WordStar/markdown-lineage delimiters — `_x_`, `*x*`, `/x/`) drops into that one hook. Reserve now, decide later: **the delimiter→mark map and an escape char** (literal `_`/`*` in text, e.g. `my_file_name`). Whitelist is **inline only** (`<strong>`/`<em>`/`<br>`-class) — legal inside `<p>`, so no block-tag tension, and sanitization stays trivial. Links (`<a>`) are the one mark that re-introduces risk (href sanitisation) — decide deliberately if added.

**`image` — root `<img>`.** Value = `src` (string — same value-shape as the other string-valued di); `alt` is a **required a11y prop**, not optional (the image equivalent of `label`'s caption). `<img>` is for **content** images (the picture *is* the data); **decorative** imagery is CSS `background-image` = an L2 **skin** concern, not a component. Skin's job on `<img>` is sizing/fit (`object-fit`, dimensions, radius); `src`/`alt` are value/props, not skin.

**Two text processors on the EDIT-vs-RENDER axis** (not "dynamic/static" — that names *when* it runs, not *what side*; edit-vs-render matches the interactive-di / display-di fault line):
- **edit-side** — a live formatter on a bound, editable value (`textarea`/`textfield`), input-side, re-runs as the user types. (Ties to the earmarked `textfield` `use:processor`.)
- **render-side** — a read-only transform that renders a fixed value once for display (`paragraph`).
Both are reusable `common` actions, thin seams now, formatter logic deferred.

**Parked (explicitly NOT these components):** a multi-paragraph **`textblock`/`richtext`** (root `<div>`, multi-block structured content + sanitiser) — a heavier composite-style renderer, only if ever wanted; and the **edit-side processor** for `textarea` (the live counterpart to `paragraph`'s render-side one). Naming `paragraph` for the single-paragraph case deliberately frees `textblock` for this parked richtext.

**Display-di trio (conceptual queue):** `label` (`<label>`, caption) · `paragraph` (`<p>`, single-paragraph prose, formatter seam) · `image` (`<img>`, `src`+`alt`). Opens after M-RP2.7 (first skin pass) and `select` (di·A), per the locked order.

*UI-implementation record. No protocol/data implication. Conceptual only — none built. Candidate entries for the di catalogue; the `paragraph` formatter axis ties to the `select` segmented-shape and `textfield` `use:processor` threads.*

## 2026-06-24

### N-033 — First skin pass shipped: L2 vocabulary founded (M-RP2.7); the `*/`-in-comment parser trap; switch skin-only confirmed

M-RP2.7 closed (J-412) — the first skin pass, implemented + verified live in both apps (Ms Design seat; Chat self-drove the CDP loop). Stands up the N-031 stack and **founds** the L2 token+treatment vocabulary; closes the N-028/N-029 `button{}` wrinkle. No code on the three components (zero-`<style>` invariant held); all work in the three CSS sources + shell wiring.

**Stack stood up (N-031 operationalised).** `modern-normalize.css` relocated to `ui/assets/` (pristine L0 — the npm-package import in the retired `dev_core_ui` template was never the live path; the shells' hand-rolled `app.css` reset was doing L0, now retired). New `xgen-normalize.css` (L0 floor) + one `skin.css` (L2). Both shells: `$assets` Vite alias, `main.js` chain `modern-normalize → xgen-normalize → skin → app.css`, `app.css` gutted to shell chrome + a per-shell `--accent*` alias. `@font-face` now pulls the shared `ui/assets/fonts/` copy (the per-shell `src/assets/` font duplication is no longer load-bearing).

**L2 vocabulary founded (the reusable primitives later components assemble from).** Tokens: semantic palette (canonical here now), radius (`--rad`/`--rad2`) + spacing (`--sp-1..4`) scales, `--ctl-h`, accent-tinted `--focus-ring`, `--motion`. Treatments: `:focus-visible` ring, `:disabled` grey, `:invalid`→`--err`, pressed = `[aria-pressed="true"]` accent fill + `:active` inset bevel, switch = `.toggle[role="switch"]` `appearance:none` + `::before` thumb with `:checked` `translateX`. Component keys `.button`/`.toggle`/`.textfield`. **One shared `skin.css`, per-shell `--accent*`** (Q2): client gold/`--pr`, node blue/`--inf` — the single per-app knob; the semantic state palette (state-dot) stays shared, never accent.

**Wrinkle closed (N-028 finding 2 / N-029).** The generic `button{…appearance…}` rule re-keys off bare `<button>` onto `.button`; a classless `<button>` now renders the normalize-flat floor (verified: `bg rgba(0,0,0,0)`, 0 border/radius/pad), `.button` renders skinned. The dead `primary-*-button` rules retire into the `--accent` treatment.

**Finding — the `*/`-in-comment parser trap.** Both `app.css` header comments listed the palette as `(--s*/--t*/--pr*/--inf*/--ok/--err)`. The `--s*/` substring contains `*/`, which **closes a C-style comment early**; the trailing comment text + the immediately-following `:root{--accent…}` rule were consumed as malformed CSS and dropped (parser recovered at the next rule). Symptom: `--accent` undefined at runtime, while the skin's `var(--accent, var(--pr))` fallbacks masked it as plausible-looking colour. Caught by the CDP verify (the stylesheet map showed app.css starting at `html,body` not `:root`; a `var(--accent, rgb(1,2,3))` sentinel returned the sentinel). Lesson: **never put a `*` adjacent to a `/` inside a CSS `/* */` comment** (token lists, glob-like notation). The skin pass being *verified rather than assumed* is what surfaced it.

**Q5 verdict — switch skin-only (locked).** `appearance:none` + `::before` thumb renders a clean pill+thumb in both apps; the single-engine WebView2/Chromium target removes the historical pseudo-element-on-form-control risk. No L1 construction scaffold needed — `toggle.svelte` stays `<style>`-free, and all three built components remain zero-L1 (the N-031 prediction that L1 stays near-empty, confirmed).

**Tooling.** `cdp-debug.ps1` gained `-Mode screenshot` (`Page.captureScreenshot` → PNG in `temp/`), so a skin pass is self-verifiable visually by Chat (the rendered cascade), complementing the `getComputedStyle` probe (the resolved cascade) — neither is the N-024 registry, which does not see CSS.

*UI-implementation record. No protocol/data implication. L2 vocabulary now founded; `select` and later components assemble skin from it. Candidate to graduate with the N-019/N-020/N-021/N-025/N-031 CSS cluster to DECISIONS.md.*

## 2026-06-25

### N-034 — `select` built + skinned (M-RP2.8): the first content-carrying di component; the `options`-prop precedent

M-RP2.8 closed (J-413) — `select` (di·A, single-select, atomic `<select>`, pick-only) authored and skinned in one pass. First component built *after* the skin stack existed (N-033), so author-and-skin land together; and the first **content-carrying** di component.

**Component.** `ui/core/lib/components/data-independent/select.svelte`. Root `<select use:envelope>` (N-020); `bind:value` string (the bind-in path, after toggle/textfield); native-state `disabled`/`id`/`name`/`required`; getter `() => $state.snapshot({ value })`; zero `<style>`. No `multiple` (separate semantic/shape).

**The `options` precedent (the design point).** toggle/button/textfield carry only native state; `select` is the first di component that carries *content* (its `<option>` list). Locked shape: a single **`options` prop**, accepting `string[]` or `{value,label,disabled?}[]`, normalized internally to `{value,label,disabled}[]` and rendered with `{#each}`. Chosen over slotted children because it (a) keeps the root atomic — no wrapper, N-020; (b) keeps the component data-*independent* (consumer passes a small static set, like a radio group's items); (c) is the same surface the data-derived layer will feed later. Optional `placeholder` → a leading disabled `<option value="">`. **This is the pattern every later content-carrying component (di composites, dd materializations) follows: a normalized data prop in, native markup rendered internally — never markup handed in by the consumer.**

**Skin (assembled, not founded).** `.select` is the first component skinned purely by *assembling* the M-RP2.7 L2 vocabulary (N-019 reuse applied to styling) — `--s`/`--s5` box, `--rad`, `--ctl-h`, `--sp-*` padding, accent-tinted focus ring, disabled grey, `:invalid`→`--err`. The only new asset is the dropdown arrow: `appearance:none` + an inline-SVG `background-image` chevron (right-aligned). A wrapper-and-`::after` arrow was rejected — it would move the root off `<select>` (N-020), and `::after` on a `<select>` is unreliable; background-image keeps the root native and L1 empty (all four built components stay zero-`<style>`). The closed control is skinned; the **open option-list popup is engine-rendered and left native** (Q3) — a classic-CSS limit; `appearance:base-select` / `::picker(select)` (Chromium 135+) is the future full-style path once the pinned WebView2 version is confirmed.

**Verify (N-029 restated for `change`).** Driving `bind:value` over CDP needs a dispatched **`change`** event (`new Event('change',{bubbles:true})`), not a bare `el.value=` — the same lesson as textfield's `input`. CDP-verified both apps: `{value:""}` → `change` → `{value:"beta"}` (client) / `{value:"gamma"}` (node); `appearance:none`, arrow present, eye-checked.

*UI-implementation record. No protocol/data implication. `select` is the fourth built `core` component; the `options`-prop precedent governs content-carrying components going forward. Next: display-di `label` (N-032).*

### N-035 — `label` built + skinned (M-RP2.9): the first DISPLAY-kind di; the read-only component pattern + the render-then-computed-style verify

M-RP2.9 closed (J-414) — `label` (di·A, display-kind, atomic `<label>`, read-only caption) authored and skinned in one pass. The **first display-kind di** — where the four built so far (toggle/button/textfield/select) are *interactive* (input/event, live getter delta), `label` is **value-carrying but read-only**: the display half of the di model (N-032). Founds the pattern `paragraph`/`image` inherit.

**Component.** `ui/core/lib/components/data-independent/label.svelte`. Root `<label use:envelope>` (N-020); value prop **`text`** (plain, **not** `$bindable`); `id`; getter `() => $state.snapshot({ text })`; body is `{text}`; zero `<style>`. The five walk locks (Joe, 2026-06-25):
- **`text`, not `value`** — `value` is the editable/`$bindable` marker everywhere; display-di take a *semantic* value-name (label & paragraph = `text`, image = `src`). Not one shared prop name — a semantic one per component.
- **No `for` on the atomic** — association (`for=`/nesting) is a composite concern (N-032), wired by the group (`textfield-group`, implicit nesting). `id` kept (debug + future nest target). Standalone label = valid-but-inert.
- **Getter registered anyway** — read-only has no user-driven delta, but the registry stays uniform (N-030 §4: the registry is one projection of the value). This *founds the display-di verify pattern*: no event to dispatch — **verify = snapshot returns the passed value + a computed-style probe**, vs the interactive set's dispatch-then-delta.
- **Skin assembles, no new token** — `.label` = `color: var(--t2)` + `font-size: 12px` + `line-height: 1.5`, all existing vocabulary (N-019 reuse). The **`--fs-*` type-size scale is deferred to `paragraph`** (M-RP2.10): a caption alone doesn't justify founding a scale + retro-keying the four shipped skins; `paragraph` (needs body size + line-height) with two text components in hand is the honest trigger. Keeps `select`'s zero-new-token discipline (N-034).
- **`use:envelope` unchanged** — content-agnostic substrate reused verbatim. The only deltas from interactive di: plain prop (no `$bindable`), no handler, render-not-dispatch verify. Confirms the substrate generalizes across the **interactive/display fault line** — the read-only extension of the boolean-in/event-out/string-in/content-carrying generalizations the prior passes each proved.

**Verify (both apps, real `tauri dev` + CDP; Chat self-drove).** Registry: `label#demo` → `{"type":"label","state":{"text":"Demo label"}}` (client 9222 / node 9322), the first display-di registered cleanly beside the four interactive di. Computed-style probe (both): `color: rgb(200, 196, 188)` (=`--t2` #c8c4bc), `font-size: 12px`, `line-height: 18px`. Screenshots both apps eye-checked — the dim caption renders between the select and the toggle button. Clean teardown (9222/9322/5173/5174 free, 0 orphans).

**Finding — computed `display:block` is flex-item blockification, not a skin rule.** The probe returned `display:block` on the rendered `.label`; investigation showed both `<body>` (flex-row) and `<main#core-ui-pane>` (flex-column) are flex containers, so the label is a **flex item** — CSS blockifies a flex item's computed `display` to `block` regardless of its own value (a bare `<label>` appended to the flex `<body>` reported `block` too). The `.label` skin sets **no `display`**; the UA inline default and the N-032 "inline default; block = a skin variant" framing both stand — the block is **environmental** (the shell's layout), not the component forcing it. Recorded so the computed value is not misread as a skin rule when `paragraph`/`image` are verified the same way.

*UI-implementation record. No protocol/data implication. `label` is the fifth built `core` component and the first display-kind di; the `text`-prop + render-then-computed-style verify are the display-di precedents. Next: `paragraph` (root `<p>`, N-032) — founds the `--fs-*` type scale.*

### N-036 — `paragraph` built + skinned (M-RP2.10): the `--fs-*` type scale founded; the render-side formatter seam reserved as a `common` action

M-RP2.10 closed (J-415) — `paragraph` (di·A, display-kind, atomic `<p>`, single paragraph of prose) authored + skinned in one pass, AND the `--fs-*` type-size scale founded (deferred here from M-RP2.9/N-035). Second of the display-di trio (after `label`); reuses the read-only pattern verbatim.

**Component.** `ui/core/lib/components/data-independent/paragraph.svelte`. Root `<p use:envelope>` (N-020); value prop **`text`** (plain, the display-di semantic name, shared with label — image takes `src`); `id`; getter `() => $state.snapshot({ text })`; body the **text node** `{text}`; zero `<style>`. Identical shape to `label` — the read-only display-di pattern (N-035) generalizes unchanged to a second tag.

**Formatter seam (reserved, NOT built — the design point).** N-032 reserved an inline-mark formatter for `paragraph`; locked shape (Joe, design walk): the body is a plain **text node** today (`{text}`), never `{@html}` — safe by default. The future limited inline formatter (`_x_`/`*x*`, whitelist `<strong>`/`<em>`/`<br>`, escape char; links the one risky add) lands as a **`common` `use:render` action** — the render-side counterpart to the edit-side `use:processor` earmarked for textfield/textarea (the EDIT-vs-RENDER axis, N-032). That action owns the delimiter map + whitelist + sanitisation and rewrites the node's content only when applied; the component never opens `{@html}` itself. **Not built now** (D-065 — no empty machinery); the seam is a documented insertion point. This is the precedent for read-only render-side processing across the display-di.

**The `--fs-*` type scale founded (the milestone's second half).** Until now every component hardcoded `font-size: 12px` (four shipped: button/textfield/select/label) — no type vocabulary. Founded in `skin.css` `:root`: **`--fs-1: 12px`** (control/caption) + **`--fs-2: 14px`** (body prose) + **`--lh: 1.5`** (shared line-height). Numeric ascending=larger, matching `--sp-*`/`--rad`; avoids the `--t*` colour-ramp collision. **Pair only — no `--fs-3`/`--fs-4` seed** (D-065: no current consumer; grow the scale when a heading/lead component needs it). The four shipped skins **retro-keyed** in the same pass: `font-size: 12px` → `var(--fs-1)`, `line-height: 1.5` → `var(--lh)` (8 substitutions); components stay zero-`<style>`. This is N-031's "L2 saturates by vocabulary" in action — the type primitive founded once, future text components assemble from it.

**Skin.** `.paragraph` = `font-size: var(--fs-2)` (14px), `color: var(--t)` (the **brightest** text ramp — prose is the *content*, vs label's caption `--t2`), `line-height: var(--lh)`, `margin-block-end: var(--sp-3)` (block rhythm; xgen-normalize zeroes the UA `<p>` margins so all spacing is skin-owned).

**Verify (both apps, real `tauri dev` + CDP; Chat self-drove).** Registry: `paragraph#demo` → `{"type":"paragraph","state":{"text":"Demo paragraph of prose."}}` (client 9222 / node 9322). Computed-style (both): `.paragraph 14px/21px rgb(236, 233, 225)` (=`--fs-2` / `--t`). **Retro-key re-verified non-regressive** — all four still resolve 12px/18px: `.button 12px/18px rgb(200,196,188)` (`--t2`), `.textfield 12px/18px rgb(236,233,225)` (`--t`), `.select 12px/18px rgb(236,233,225)`, `.label 12px/18px rgb(200,196,188)`. Screenshots both apps eye-checked — the paragraph renders visibly larger + brighter than the label caption above it (the content/caption distinction). Clean teardown (0 orphans).

*UI-implementation record. No protocol/data implication. `paragraph` is the sixth built `core` component (second display-di); the `--fs-*` scale + the render-side `use:render` seam are the precedents going forward. Next: `image` (root `<img>`, `src`+required `alt`, N-032) — completes the display-di trio.*

### N-037 — `image` built + skinned (M-RP2.11): the display-di trio complete; first attribute-valued di + first bundled-asset demo

M-RP2.11 closed (J-416) — `image` (di·A, display-kind, atomic `<img>`, `src` + required `alt`) authored + skinned in one pass. **Third and final display-di** — the trio (label/paragraph/image) is now complete.

**Component.** `ui/core/lib/components/data-independent/image.svelte`. Root `<img use:envelope>` (N-020); props `src: string` + `alt: string` (**both required, no default**) + `id`; getter `() => $state.snapshot({ src, alt })`; zero `<style>`. **Structural novelty:** `<img>` is a **void element** — the first display-di whose value lives in an **attribute** (`src`), not a text-node body (label/paragraph put the value in their content). The read-only pattern otherwise carries over verbatim (same envelope, plain props, render + computed-style verify).

**Required `alt` (the design point).** N-032 locks `alt` required; locked shape (Joe): `alt: string` typed non-optional with **no default**, so the consumer must consciously pass something — including `alt=""` for a deliberately decorative image (valid + conscious). The requirement forces the a11y **decision**, it does not forbid empty. A **DEV-only `console.warn`** fires if `alt === undefined` (omitted entirely); no prod throw — an image with a missing alt should still render. `src` likewise required.

**Getter carries two fields.** Unlike label/paragraph's single `{text}`, image registers `{src, alt}` — `alt` being required makes it part of the component's meaningful state, and snapshotting it lets verify confirm the required-alt landed. Precedent: a display-di getter carries the fields the semantic demands, not always one.

**Skin.** `.image { border-radius: var(--rad); }` — an image has no typography; its appearance is intrinsic, so the skin's only job is framing (here the corner radius, matching the rounded control language). **No new token.** Sizing (width/height) is a consumer/layout concern, not the atomic's skin. xgen-normalize already floors `<img>` block + `max-width:100%` + transparent.

**Demo = bundled placeholder asset (first of its kind).** New asset `ui/assets/img-placeholder.svg` (Joe-approved: neutral grey square `#c6c6c6` + light-grey `#e6e6e6` frame/sun/two-peaks glyph — a reusable neutral missing-image placeholder, not just demo throwaway). Imported via the `$assets` alias in both shells (`import Placeholder from '$assets/img-placeholder.svg'`) — the **first component demo backed by a project asset import** rather than an inline literal (label/paragraph used string literals). Vite **inlined** the sub-threshold SVG as a `data:image/svg+xml,…` URI, so the resolved `src` is the data-URI itself (not a file path) — expected for small assets.

**Verify (both apps, real `tauri dev` + CDP; Chat self-drove).** Registry: `image#demo` → `{"type":"image","state":{"src":"data:image/svg+xml,%3csvg…","alt":"Image placeholder"}}` (client 9222 / node 9322). Computed-style (both): `tag IMG`, `border-radius 6px` (=`--rad`), `display block`, `complete true`, `alt "Image placeholder"`. Screenshots both apps eye-checked — the placeholder renders (grey square, light glyph, rounded corners); it stretches to the column width (max-width:100% + flex stretch, no width constraint — expected, sizing deferred). Clean teardown (0 orphans).

*UI-implementation record. No protocol/data implication. `image` is the seventh built `core` component and the third/final display-di — **the display-di trio is complete**. The first attribute-valued di + first asset-backed demo. Next: the **first composites** (`textfield-group` = label + textfield, where `for`/association lands; `combobox` = textfield + datalist) — the di→composite transition.*

### N-038 — di catalogue scoping: textfield `type`-family fold-in, `number` boundary, the processor engine deferred, and the di→processor→dd sequence

Conversation-derived scoping pass (2026-06-25), recording decisions reached before resuming atomic-di builds. No code.

**Atomic / shape / composite boundary (the discriminator restated).** The catalogue row is keyed to a semantic; what makes a *new atomic component* is a distinct root structure or value-type — not a distinct `type` literal. So: string-valued, structurally-identical `<input>` types **fold into one component**; value-type-changing or chrome-adding types are **own atomics**; custom-chromed versions are **composites**.

**`textfield` gains a constrained `type` prop.** Whitelist `text` (default) | `search` | `email` | `url` | `tel` | `password` — all share `<input>` root, string `bind:value`, `.textfield` skin; they differ only in browser-supplied validation/keyboard/masking. One file, one type-class. The `password-field` composite embeds `<textfield type="password">` + an eye toggle. **This reverses N-029** ("type is fixed, separate semantics") — a conscious re-lock; the DECISIONS.md D-entry is promoted when the prop is actually built (decision lands with code). Caveat to verify at build: per-type native quirks (e.g. `maxlength` interaction) — whitelist + a CDP check per type.

**`number` stays its own atomic.** `<input type="number">` is a single element (atomic) — the up/down spinner is a UA pseudo-element, not authored buttons. It's separate from `textfield` because its `bind:value` is **numeric, not string** (type-unstable to fold in). Prop surface: `value`/`min`/`max`/`step` + native passthrough. `step` governs increment + permitted decimals but is validation, not display formatting; **trailing zeros (`3.00`), thousands separators, and currency are NOT native** (a number has no trailing zeros) — they're formatter concerns, out of the atomic. A custom `− [input] +` stepper is a **composite** (own buttons), the atomic/composite split mirroring native `<select>` vs custom dropdown.

**The text-processor engine (sharpened spec, REMAINS DEFERRED — N-029/N-032).** Not random replacement files: one thin `use:processor` seam (mechanism) + per-instance **config** (the rule-set) — reactive, so rules and e.g. decimal places can change on demand. One seam serves three consumers: text morphs/emoji (`textarea`), numeric formatting incl. trailing zeros (`number`), inline marks (`paragraph`, render-side `use:render`). Security shape is non-negotiable when built: the text/number sink writes to `.value` (text, safe by construction); the markup path (`paragraph`) goes through allowlist + a real sanitizer, never `{@html}`, **never regex as the safety boundary** (allowlist > blocklist); externally-supplied patterns get a ReDoS/complexity guard; sensitive configs are **named, reviewed `common` configs** (trusted-vs-arbitrary by provenance). Built only when a consumer needs it (D-065) — earliest natural trigger is `textarea`/`number`.

**Build sequence (vision, not a locked roadmap — per-milestone walks still happen on open).** Finish all atomic di → the text-processor engine (own arc, consumers in hand) → dd-components (on a complete, settled di + processor foundation). Remaining atomic di after `textfield`-`type`: `textarea`, `number`, `range`, `date`/`time`, `color`, `file` (new `bind:files` shape), `select multiple`. Shapes (fold into built): search→textfield, tri-state→toggle, segmented→button/radio. Composites (the composite milestone, not atomic): radio-group, checkbox-group, combobox, tag-select, star-rating, password-field, custom stepper.

*Scoping record. No protocol/data implication, no code. Reverses N-029 (textfield `type` fixed → constrained prop) → D-096 (landed M-RP2.12); sharpens + keeps deferred the N-029/N-032 processor seam. Sets the di→processor→dd track order.*

### N-039 — `textfield` `type` fold built (M-RP2.12): the N-029 reversal lands with code (→ D-096); per-type inset icons; a Svelte-bind verify subtlety

M-RP2.12 closed (J-417) — `textfield` gains a constrained `type` prop, folding the string-input family into one component. N-038's pre-authorised reversal of N-029 now lands with code **→ D-096** (the fold decision; this note carries the icon treatment + the verify finding, which are skin/method, not the decision).

**Component.** `type?: 'text'|'search'|'email'|'url'|'tel'|'password'` (default `'text'`), **TS union only** — no runtime guard, no DEV-warn: an out-of-whitelist value degrades safely (browser → `text`), so a guard would be empty machinery (D-065), and unlike image's required `alt` the type system has a safe native fallback here. Root `<input {type}>`; getter now `{ type, value }` (carries `type` so the configured type is registry-verifiable — the image-`alt` precedent). `textfield.svelte` stays zero-`<style>`. `maxlength` deliberately NOT added — orthogonal to the fold.

**Per-type inset icon (skin, not behaviour).** A very-weak-grey `#e6e6e6` (the `img-placeholder` light grey — lighter than `--t3`/`--t4`, reads as a faint hint) inline-SVG, right-inset in the text cell, keyed by `.textfield[type="…"]` — same mechanism as the `select` arrow; the colour literal lives inside each SVG (`%23e6e6e6`), not a `:root` token. Glyphs: `text` none · `search` magnifier · `email` envelope · `url` link · `tel` rotary-ish phone · `password` `***`. Iconed types carry a right-padding bump (`calc(--sp-4 + --sp-1)`); `text` keeps default padding. The native `search` clear-"x" is suppressed (`::-webkit-search-cancel-button { appearance:none }`) so it doesn't collide with the magnifier.

**Password reveal is NOT here.** The atomic `type="password"` stays pure (masks + static `***` icon). A readable/reveal toggle is interactive chrome → breaks atomicity → ships as the `password-field` composite (D-096, deferred to the composites track).

**Verify (both apps, real `tauri dev` + CDP; Chat self-drove).** Registry: `textfield#demo` → `{type:"text",value:""}` (default holds), `textfield#demo-search` → `{type:"search",value:""}`. `el.type` sweep across all six whitelist values round-trips exactly (browser accepts each). Per-type computed `background-image`: `text` → `none`, the other five → an image present. `bind:value` delta on `type=search` → `{type:"search",value:"find me"/"node find"}` (the string bind path holds on a non-text type). `.textfield:invalid` applies `--err` (rgb 138,42,42) for **both** native email-type-validation and `pattern` — confirmed on clean detached elements; valid email + plain text stay `--s5`. Screenshots both apps: plain field iconless, search field shows the right-inset magnifier (clear-x gone), email field renders red border + envelope. Clean teardown (0 orphans).

**Verify finding (method, worth keeping).** You cannot probe per-`type` native behaviour by mutating `el.type` on a Svelte `bind:value`-owned `<input>` and reading across an event flush — reconciliation + the bind round-trip fight the manual DOM mutation, and one `getComputedStyle(:invalid)` read returned the base border mid-flush (the rendered screenshot + a detached-element test both showed the correct red). Probe per-type via a **detached element** (or an instance authored with that type) + the screenshot; synchronous `el.type`/computed reads with **no event dispatched** are safe (the sweep above).

*UI-implementation record. No protocol/data implication. `textfield` is still the seventh built component (a fold, not a new one) but now covers six input types; the N-029 reversal is recorded as D-096. Next: remaining atomic di (`textarea`/`number`/… per N-038) and, on the composites track, `password-field` with the reveal toggle.*

## 2026-06-26

### N-040 — `textarea` built (M-RP2.13): the eighth `core` component; stand-alone atomic (not a textfield fold); the edit-side `use:processor` seam reserved + processor kept deferred

M-RP2.13 closed (J-418) — `textarea` (di·A, atomic `<textarea>`, multi-line free-text, string `bind:value`) authored + skinned in one pass. The **eighth** `core` component and the next atomic di per the N-038 track order. Root tag is `<textarea>`, not `<input>` → by the N-020 root-tag discriminator a **new atomic component, NOT a `textfield` fold**. The edit-side multi-line counterpart to `paragraph`'s render-side single prose string (N-032 EDIT-vs-RENDER axis): `paragraph` wraps one read-only string visually, `textarea` holds literal `\n`-bearing editable free text.

**Component.** `ui/core/lib/components/data-independent/textarea.svelte`. Root `<textarea use:envelope>` (N-020); string `bind:value` (the bind-in path again, after toggle/textfield/select); getter `() => $state.snapshot({ value })`; zero `<style>`. Prop surface = the `textfield` string-input vocabulary **minus** what `<textarea>` can't carry, **plus** `rows`: keep `value`/`placeholder`/`disabled`/`readonly`/`id`/`name`; **drop `type`** (no such attribute) and **`pattern`** (`<input>`-only — so no `:invalid`-via-pattern path here); **add `rows`** (numeric, default `3` — the one textarea-specific prop, initial visible height). `maxlength` deliberately omitted (mirrors textfield). Getter is **value-only** — `rows` is static config, not user-mutable state (textfield didn't snapshot `placeholder`).

**Processor seam — reserved, NOT built; `textarea` is processor-READY, not the trigger (the milestone's design decision).** N-038 named `textarea`/`number` as the processor's "earliest natural trigger"; the walk resolved that to **defer**, on two locked grounds: (1) the N-038 sequence is locked — *finish ALL atomic di → engine (own arc, all consumers in hand) → dd* — and `textarea` is not the last atomic (`number`/`range`/`date`/`color`/`file`/`select multiple` follow), so building here over-fits the seam to one of the three named consumers; (2) D-065 — the *atomic* is function-complete without it, exactly as `textfield` shipped processor-ready. The header reserves the **edit-side `use:processor`** insertion point (the counterpart to `paragraph`'s render-side `use:render`); nothing built. The "earliest natural trigger" line is a candidacy note, not a commitment — the locked trigger is "all atoms done."

**auto-grow — future skin shape, not built.** The single-engine WebView2/Chromium target affords a pure-CSS path (`field-sizing: content`), reserved as a skin shape (like `select`'s `appearance:base-select`), not authored now (D-065). The atomic ships native fixed-`rows` height + vertical resize.

**Skin (assembled).** Own `.textarea` key, assembled from the M-RP2.7 L2 vocabulary like `.textfield` (per-class clarity > DRY, the `.select` precedent — not a shared `.textfield, .textarea` group). Same box (`--s`/`--s5`/`--rad`/`--t`/`--fs-1`/`--lh`/padding/`:focus-visible`/`:disabled`/`:read-only`); differs in **no `min-height: --ctl-h`** (rows drives height), **`resize: vertical`** (horizontal would break the flex-column width), **no per-type icon machinery**, **no `:invalid`** (no native `pattern`). No new `:root` token.

**Verify (both apps, real `tauri dev` + CDP; Chat self-drove).** Registry baseline both apps: `textarea#demo` → `{value:""}`. Dispatched `input` (textarea fires `input`, not `change` — N-029) with a newline-bearing string → registry `{value:"line one\nline two"}` (client) / `{value:"node line A\nnode line B"}` (node), `lineCount=2` both — the **literal `\n` survives the bind rune to the registry snapshot**, the thing distinguishing it from `textfield`. Computed-style both: tag `TEXTAREA`, `font-size 12px` (=`--fs-1`), `color rgb(236,233,225)` (=`--t`), `resize vertical`, `border-radius 6px` (=`--rad`), bg `rgb(22,24,28)` (=`--s`), border `rgb(52,59,71)` (=`--s5`). Screenshots both apps eye-checked — multi-line box + the vertical resize grabber render, per-shell chrome. Clean teardown (ports 9222/9322/5173/5174 free, 0 orphans).

*UI-implementation record. No protocol/data implication. `textarea` is the eighth built `core` component; the processor stays deferred (N-038 sequence + D-065), `textarea` ships processor-ready. No `DECISIONS.md` touch — the defer is the application of an existing principle, not a new one. Next: remaining atomic di — `number` (own atomic, numeric `bind:value`) per N-038.*

### N-041 — `number` built (M-RP2.14): the ninth `core` component; own atomic (D-096 held — the value-type discriminator); first non-string/non-boolean registry value; processor kept deferred (2nd consumer)

M-RP2.14 closed (J-419) — `number` (di·A, atomic `<input type="number">`, numeric free-entry, numeric `bind:value`) authored + skinned in one pass. The **ninth** `core` component, the next atomic di after `textarea` per N-038. Mechanically the same `<input>` root as `textfield`, but a **distinct atomic, NOT a member of the `textfield` `type` fold** — the boundary D-096 drew is *same root + same VALUE-TYPE*, and `number` breaks the second half: Svelte's `bind:value` on `type="number"` coerces to a **number** (`null` when empty), not a string. Folding it in would force `textfield`'s `value` prop polymorphic (`string | number | null`) and defeat the single-typed contract the fold exists to give. So `number` stays its own atomic; **D-096 held, not amended** (holding the boundary is *applying* the decision). First registry value that is neither boolean (toggle) nor string (everything since): a JSON **number | null**.

**Component.** `ui/core/lib/components/data-independent/number.svelte`. Root `<input type="number" use:envelope>`; `value = $bindable(null)` (type from the lang=ts prop annotation `value?: number | null`); getter `() => $state.snapshot({ value })`; zero `<style>`. Prop surface = the control vocabulary with the numeric bits swapped: keep `value`/`placeholder`/`disabled`/`readonly`/`id`/`name`; **drop `type`** (fixed) and **`pattern`** (ignored on `type=number`); **add `min`/`max`/`step`** (native attributes that shape the control; `step` drives the native-spinner increment — config, not state, not in the getter). The **native spinner is kept** — the UA up/down arrows ARE the atomic's affordance; the custom-button **stepper** is a separate composite (later track), so no `::-webkit-*-spin-button` suppression (contrast M-RP2.12, where the search clear-x WAS suppressed because it collided with our inset icon — here nothing collides and the spinner is wanted).

**Processor seam — reserved, NOT built; the second defer-per-consumer instance.** N-038 names `number` as the processor's **numeric-formatting** consumer. Deferred on the same two grounds as `textarea` (N-040): the N-038 sequence builds the engine in its own arc after *all* atomic di (every consumer in hand), and D-065 keeps the atomic free of empty machinery. Header reserves the edit-side `use:processor` insertion point; nothing built. This is the **second** reserve-and-defer (after `textarea`) — a **D-069 promotion-watch**: if the defer-per-consumer pattern recurs to the four-recurrence bar it graduates to DECISIONS.md; **not yet** (two instances).

**Skin (own `.number`).** Own key, assembled from the M-RP2.7 L2 vocabulary like `.textfield` (the `.select`/`.textarea` per-class precedent). Single-line control — so it **keeps `min-height: --ctl-h`** (unlike `.textarea`) and **keeps `:invalid` → `--err`** (meaningful here via native numeric constraint validation: out-of-`min`/`max`, bad `step` — same treatment `.textfield` uses for email/pattern). No icon machinery, no `resize`, no spinner suppression, no new `:root` token.

**Verify (both apps, real `tauri dev` + CDP; Chat self-drove).** Registry baseline both apps: `number#demo` → `{value:null}` — **empty input snapshots as `null`** (the Lock-2 expectation, confirmed at runtime, not assumed). Dispatched `input` (N-029) with a numeric value → registry carried a **JSON number** (parsed Int32, `isNumber=True`): client `{value:42}`, node `{value:7}` — NOT the string `"42"`. This is the number-distinguishing proof (the analogue of textarea's `\n`-survives-the-rune). `:invalid` probe on the live `.number` (`min 0`/`max 100`): out-of-range `999` → `:invalid` true, computed `border-top-color` `rgb(138,42,42)` (=`--err`); in-range `42` → valid, `rgb(52,59,71)` (=`--s5`). Computed-style both: tag `INPUT`, `type=number`, `min-height 28px` (=`--ctl-h`), `font-size 12px` (=`--fs-1`), `color rgb(236,233,225)` (=`--t`), `border-radius 6px` (=`--rad`). Screenshots both apps — number box renders (native spinner is UA hover/focus-revealed in Chromium). Clean teardown (0 orphans).

**Build/verify nuances (caught live, worth keeping).** (1) **Runes can't carry TS generics or annotations in the plain-JS app shells.** `$state<number | null>(null)` and `let x: number | null = $state(…)` both fail in `app_client.svelte`/`app_node.svelte` (Vite `rune_missing_parentheses` then `js_parse_error`) because those `<script>` blocks are **plain JS, not `lang=ts`** — a whole-app mount failure (empty body, no `__XGEN_DEBUG__`). Fix: bare `$state(null)` in shells; in the lang=ts component use `$bindable(null)` and let the prop-type annotation type it. (2) **Pseudo-class computed-style must be read in a separate CDP task from the dispatch** — a same-task `getComputedStyle` after the `input` event returned the pre-recalc border (`--s5`) even though `matches(':invalid')` was already true; a second eval round-trip (post-flush) returned the correct `--err`. Sibling to N-039's mid-flush caveat: don't read restyled output in the same tick you triggered it. (3) Vite parse-error overlays do **not** auto-dismiss on fix — a `location.reload()` over CDP was needed to recover the apps.

*UI-implementation record. No protocol/data implication. `number` is the ninth built `core` component; the processor stays deferred (2nd consumer, D-069 watch). No `DECISIONS.md` touch — D-096 held (applied, not amended); the defer is the application of N-038 sequence + D-065. Next: remaining atomic di — `range` (own atomic, bounded numeric `bind:value`, slider) per N-038, in a new session.*

## 2026-06-27

### N-042 — `range` built (M-RP2.15): the tenth `core` component; own atomic on the SHARPENED fold criterion (→ D-096 amendment); first pseudo-element-heavy skin; the slider-pseudo verify finding

M-RP2.15 closed (J-420) — `range` (di·A, atomic `<input type="range">`, bounded numeric, slider, numeric `bind:value`) authored + skinned in one pass. The **tenth** `core` component, the next atomic di after `number` per N-038 (catalogue row *numeric (bounded)*). Mechanically the same `<input>` root **and** the same value-type (number) as `number` — so by the *literal* D-096 criterion (root + value-type) it would fold into `number`. It does not: **D-096's criterion is necessary but not sufficient, and `range` is the case that tests it** (→ D-096 **amendment**, the criterion sharpened to root + value-type + **shared skin/surface**).

**Why own atomic (the design point).** `range` shares root + value-type with `number` but diverges on three axes the fold cannot absorb: (1) **skin** — track/thumb `::-webkit-slider-*` pseudo-elements, **zero** shared appearance with `number`'s text box + spinner; (2) **prop surface** — no `placeholder` (never empty), no live `:invalid` (the thumb is clamped, can't go out of range), no `readonly` (native no-op on `type=range`); bounds are the *defining* attribute, not an optional constraint; (3) **interaction/empty model** — clamped drag, **always-valued** vs `number`'s empty=`null`. Folding would put two disjoint skins behind one class and a prop that swaps the whole rendering — the polymorphic-contract problem D-096 prevents, on the *appearance* axis. The sharpened criterion (genuine interchangeability — one skin, one prop surface, a thin switch) is what made the string-input fold good and what `range` fails. Does **not** reopen the `textfield` fold (the string-input family still passes the sharpened test).

**Component.** `ui/core/lib/components/data-independent/range.svelte`. Root `<input type="range" use:envelope>`; `value = $bindable(0)` typed `number` (always present — the clean divergence from `number`'s `number | null`); getter `() => $state.snapshot({ value })` (always a number); zero `<style>`. Prop surface = the numeric control, slider-shaped: keep `value`/`min`/`max`/`step`/`disabled`/`id`/`name`; **drop** `placeholder`, `pattern`, `readonly` (native no-op), `type` (fixed). **No clamping** in the atomic: a consumer setting `min > 0` passes an in-range initial (documented consumer responsibility, exactly as `number` does not clamp). **No processor seam** — `range` is a bounded drag, not free-text/free-number entry, so there are no typed digits to reformat (the numeric-formatting consumer is `number`); this is **not** a third defer-per-consumer instance.

**Skin (own `.range`, first pseudo-element-heavy skin — PROVISIONAL).** `appearance:none` + `-webkit-appearance:none` on the input, then `::-webkit-slider-runnable-track` (a 4px `--s5` groove, pill radius) + `::-webkit-slider-thumb` (16px circle, `margin-top:-6px` to centre on the 4px track, `background: var(--accent, var(--pr))` → per-shell gold/blue, `border: var(--accent2, var(--pr2))`). Vendor-prefixed is fine (single-engine WebView2/Chromium — the toggle-switch `::before` / select-arrow precedent). `:focus-visible` → `--focus-ring` on the control; `:disabled` greys thumb (`--s4`) + track (`--s3`). **No `:invalid`** (clamped), **no `--ctl-h`** (track ~4px / thumb 16px — `.textarea` likewise dropped it), **no new `:root` token**. The **accent fill** (tinted track left of the thumb) is **deferred** — WebKit gives no free fill, it needs a value-driven `linear-gradient`/JS; a future skin shape (D-065).

**Verify (both apps, real `tauri dev` + CDP; Chat self-drove).** Registry both apps: `range#demo` present in `ids()`; baseline node `{value:50}` (the demo seed), client read `{value:56}` (a stray hover/drag during the long mount-poll on the minimized window — still a number, in range; re-driven cleanly below); **`typeof value === "number"`** both — always-valued, never `null`. Dispatched a real **`input`** event (N-029; range fires `input` on drag) → client `{value:75,t:"number"}`, node `{value:25,t:"number"}` — a **JSON number** on the slider bind path (the number-distinguishing proof, the analogue of `number`'s 42/7). Element computed-style both: `{tag:"INPUT",type:"range",appearance:"none",webkitAppearance:"none",width:"160px",cursor:"pointer"}`. Screenshots both apps eye-checked: the slider renders — track groove + **per-shell accent thumb** (gold client at ~75%, blue node at ~25%, matching the dispatched values) + per-shell chrome. Clean teardown (ports 9222/9322/5173/5174 free, 0 orphans).

**Verify finding (method — the first slider exposes it).** `getComputedStyle(el, '::-webkit-slider-thumb')` / `'::-webkit-slider-runnable-track'` does **not** return the authored styles — it returns UA defaults (thumb `background: rgba(0,0,0,0)`, `width: 160px` = the *element* box, not the 16px thumb). These are UA shadow-DOM pseudo-elements; Chromium does not surface author styles on them via `getComputedStyle` (sibling-shaped to N-039/N-041's mid-flush caveats, but a different cause — not a timing issue, a shadow-pseudo limitation). **The pseudo-element skin is verified instead by stylesheet-rule inspection** (walk `document.styleSheets` → `cssRules`, read `.style.cssText` for each `.range…` selector) **plus the screenshot**: all **7** `.range` rules (base + track + thumb + focus + disabled + 2 disabled-pseudo) were confirmed parsed and in the cascade in both apps, and the screenshot confirms the accent thumb actually renders. Going forward, pseudo-element skins are verified via stylesheet-rule presence + screenshot, not `getComputedStyle`.

*UI-implementation record. No protocol/data implication. `range` is the tenth built `core` component; the fold criterion is sharpened (→ D-096 amendment) — root + value-type + shared skin/surface. The pseudo-element skin verify pattern (stylesheet-rule + screenshot, not `getComputedStyle`) is the precedent for the remaining native-chromed atomics. Next: remaining atomic di — `date` (own atomic, structured value / native picker) per N-038.*

### N-043 — `color-scheme: dark` on `:root` (post-M-RP2.15 skin fix): UA-painted native control internals now render dark

Post-M-RP2.15 skin tweak (J-421), no new component. The `number` spinner arrows rendered **light** even in the dark theme: the skin styles each control's **box** (bg/border/text), but the UA paints control *internals* — spinner arrows, scrollbars, and the `date`/`color`/`file` picker chrome still ahead — from the document's **color-scheme**, which defaults to `light` and ignores our box styling. Fix: one declaration **`color-scheme: dark`** on `:root` in `skin.css` (added to the L2 token block). It inherits, so it governs every native control at once; it is the idiomatic dark-app fix and keeps the spinner (M-RP2.14 lock — no suppression). Global, not `.number`-scoped (Q chosen by Joe): the problem is "native chrome paints light in our dark app," which is document-level, so it is fixed once at the vocabulary level rather than re-tripped per native-chromed atomic. Litmus (N-031): remove it → natives go light again but the app still works → L2 skin, clean fit.

**Verify (both apps, real `tauri dev` + CDP; Chat self-drove).** `getComputedStyle(document.documentElement).colorScheme === "dark"` both apps. `toggle` accent-color unaffected (client `rgb(154,106,48)`=`--pr`, node `rgb(42,96,144)`=`--inf`) — the `accent-color`-driven checkbox shape is independent of `color-scheme`. The number **spinner** is hover-only-painted by Chromium (not shown at rest, so not screenshot-capturable without a synthetic hover), but the **scrollbar** — the same UA-painted-chrome mechanism `color-scheme` governs — renders **dark** in both shells in the screenshots, the observable proof the declaration reaches native internals. Clean teardown (0 orphans).

**Spinner sized down (follow-on, same change).** Joe then asked for the spinner ~25% smaller. The UA inner-spin-button is engine-sized; `transform: scale()` is the reliable lever — `.number::-webkit-inner-spin-button { transform: scale(0.75); }` (0.75 = 25% smaller). **Not** suppression — the M-RP2.14 keep-the-spinner lock holds, this only shrinks it. Verified via stylesheet-rule inspection (the rule parsed + in the cascade both apps, same method as the slider pseudos — `getComputedStyle` doesn't surface `::-webkit-inner-spin-button` either); the rendered hover-state size is Joe's eye-check (hover-only-paint, not screenshot-capturable).

*UI-implementation record. No protocol/data implication, no new component, no `DECISIONS.md` touch (a skin vocabulary addition, the N-031 stack). Forward-looking: pre-empts the same light-native problem on `date`/`color`/`file` pickers ahead. Next unchanged: `date` per N-038.*

### N-044 — Sampler scaffold (M-RP3.0): a third Tauri/WebView2 app as the component test-bed; component track paused

M-RP3.0 closed (J-422) — stood up **`xgen-sampler`**, a standalone Tauri/WebView2 app whose sole job is to host, tune, and CDP-verify the `core` library in isolation with a live client↔node skin-swap. New arc **M-RP3**; the di component track is **paused** (resumable). Two decisions locked: **D-097** (test-bed split — components in the sampler, the two shells *together* in the real apps, the sampler's blind spot) and **D-098** (runtime = full Tauri/WebView2 sibling via a **minimal** host, not Vite-in-Chrome).

**What was built.** Frontend `ui/sampler/` (its own Vite+Svelte app: `index.html`, `package.json` = `xgen-sampler-ui`, `vite.config.js` port **5175** with the same `$core`/`$common`/`$assets` aliases, `src/{main.js, app.css, app_sampler.svelte}`) + a **minimal crate** `xgen-sampler/` (`Cargo.toml` = `tauri` + `tauri-build` only, **no protocol deps**; `build.rs`; `tauri.conf.json` devUrl 5175, decorated/resizable 960×820 window; `src/main.rs` = the bare `tauri::Builder::default().run(generate_context!())`; `capabilities/default.json` core-only; icons copied from client) + root `Cargo.toml` workspace member; `run-sampler.ps1` (Vite 5175, `-Debug` → CDP **9422**); `cdp-debug.ps1` taught `sampler`→9422. The crate is deliberately ~6 lines of Rust — `xgen-client` is the *full* client (CLI + protocol, Tauri bolted on via `desktop::run()`); the sampler inherits **none** of that.

**Skin-swap mechanism.** Per-shell accent is three vars (`--accent`/`--accent2`/`--accent-ink` → `--pr*` client / `--inf*` node) over the shared `skin.css`. The sampler's `app.css` defines **both** blocks keyed by `:root[data-shell="client"|"node"]` (default = client) and flips `document.documentElement.dataset.shell` at runtime — one component grid, flip accent, re-theme live. Replaces “run the component in both real shells.”

**Live skin editing (the user's intent).** No fs-plugin / refresh button needed in dev: `run-sampler.ps1` runs `tauri dev`, so Vite **HMR** hot-applies every save to the **canonical** `ui/assets/skin.css` instantly in the WebView2 window. Joe edits the file directly and watches it live; Chat is out of the inner tuning loop and only does the records + commit once a look is settled. A standalone-exe live-reload is a deferred follow-on (D-098).

**Scaffold scope + verify (Chat self-drove, real `tauri dev` + CDP on 9422).** v0 mounts exactly **one** smoke component to prove the chain; the matrix is M-RP3.1. The smoke instance is a `core` `Button id="smoke"` — registry id **`button#smoke`** (envelope keys by component *type*, not app name; the runbook's `sampler#smoke` was a naming slip). Proofs: `location.href = http://localhost:5175/` (loaded from the sampler Vite server in WebView2); `typeof window.__XGEN_DEBUG__ === "object"`; `ids() = ["button#smoke"]` — the `$core` import + `envelope` + the debug registry work end-to-end in the new app (the scaffold's load-bearing proof). Skin-swap: `--accent` resolved **`#9a6a30`** (gold, `--pr`, client default) → after `data-shell="node"` flip **`#2a6090`** (blue, `--inf`). Screenshots both states: the bar (title + `accent:` tag + `swap skin` control) + the `button#smoke` cell render cleanly (the smoke button's base bg is `--s4`, not accent-driven, so the two shots are pixel-identical — the var-resolution is the swap proof; M-RP3.1's matrix adds accent-prominent components like the toggle for the visual). Clean teardown (ports 5175/9422 free, 0 orphans).

*UI-implementation record. No protocol/data implication. New arc M-RP3; component track paused (resumable). The sampler is the test-bed from `date` onward (D-097). Next: M-RP3.1 — populate the class×phase matrix with all 10 built components + the polished skin-swap control.*

### N-045 — Sampler populated (M-RP3.1): the 10 `core` components live in a semantic-group×state grid; the `toggle` has no `disabled` prop (atomic gap surfaced)

M-RP3.1 closed (J-423) — the scaffold's single `button#smoke` became the full tuning surface: **all 10 built `core` components mounted live**, 22 `envelope`-registered instances in a semantic-group×state grid, with a polished client↔node segmented skin-swap. Frontend-only (`ui/sampler/src/app_sampler.svelte` rewrite + `app.css` grid); the `xgen-sampler` crate untouched. No `DECISIONS.md` touch (applies D-097/D-098/N-028).

**IA = semantic-group×state, not class×phase.** All 10 are di·A today (no dd, no Phase B/C), so N-028's class×phase axes are degenerate — v1 groups by **Interactive** (toggle/button/textfield/select/textarea/number/range) and **Display** (label/paragraph/image), each component a row, its **applicable** states as cells. Class/phase columns activate later when dd/B/C exist.

**Ragged state-map (honest, not a forced grid):** default — all 10; disabled — interactive only; invalid — only `textfield` (bad email) + `number` (out-of-range); plus teaching variants (toggle checked/switch, button toggle-mode, textfield password, textarea `\n`). **No focus column** — focus is transient; a static focus cell would be a lie (verified live instead).

**Atomic gap surfaced (the sampler doing its job).** `toggle` exposes only `checked`/`id`/`shape` — **no `disabled` prop** — so `toggle#disabled` is impossible from the sampler without component work (paused). That cell became **`toggle#switch`** (the switch shape) and the gap is logged here: when the di track resumes, `toggle` likely wants a `disabled` pass-through for parity with the other interactive atomics. This is exactly the kind of coverage hole a dedicated exhibit surfaces that demos-in-shells did not.

**Skin-swap = polished segmented control, kept as TOOL CHROME.** A `client | node` segmented control in the bar (styled in the sampler's `app.css`, active segment uses live `--accent`), NOT a sampled `core` component — preserves the N-028 tool-vs-sampled line. Flips `:root[data-shell]`; with accent-prominent cells now present (toggle `accent-color`, the latched toggle-mode button), the two shell screenshots **genuinely differ** (unlike the smoke-only scaffold).

**Detail confirmed:** `envelope` keys the registry by `data-debug-id = "type#id"` and does NOT stamp the raw DOM `id`, so reusing `id="default"` across component types is collision-free (22 unique `type#id` keys, e.g. `toggle#default` vs `button#default`). `image#default` uses an inline data-URI (no network fetch).

**Verify (Chat self-drove, real `tauri dev` + CDP 9422).** `ids().length === 22`, full list exactly the designed matrix (`toggle#default/checked/switch`, `button#default/disabled/toggle`, `textfield#default/disabled/invalid/password`, `select#default/disabled`, `textarea#default/disabled`, `number#default/disabled/invalid`, `range#default/disabled`, `label#default`, `paragraph#default`, `image#default`). Invalid: `number#invalid` + `textfield#invalid` border = `--err` `rgb(138,42,42)`, while `number#default` stays `--s5` `rgb(52,59,71)` (invalid is specific, not blanket). Disabled: `number#disabled`/`button#disabled` `cursor:not-allowed`. Skin-swap: toggle `accent-color` `rgb(154,106,48)` (gold/`--pr`, client) → `rgb(42,96,144)` (blue/`--inf`, node), `--accent` `#9a6a30`↔`#2a6090`. Screenshots both shells render the grid correctly (states + accents) and differ in bytes. Clean teardown (5175/9422 free, 0 orphans).

*UI-implementation record. No protocol/data implication. The sampler is now the live tuning surface for all 10 components. Logged gap: `toggle` lacks `disabled`. Next: resume the component di track — `date` (own atomic, structured value / native picker) per N-038, built/tuned in the sampler (D-097).*

---

## 2026-06-28

### N-046 — `date` built (M-RP2.16): the eleventh `core` component; the date-input family FOLDS into one atomic (the `textfield` fold again, not `range`); the sampler-DoD becomes standing (→ D-097 note)

M-RP2.16 closed (J-425) — `date` (di·A, atomic `<input>` date-input family) authored + skinned in one pass, built/tuned/verified **in the sampler** (D-097). The **eleventh** `core` component, resuming the di-atomic series after the M-RP3 sampler arc; the next atomic di per N-038 (catalogue row *date / time*).

**The fold (the design point).** The five date-input siblings — `date` / `time` / `datetime-local` / `month` / `week` — **fold into one component** via a constrained `type` prop (default `date`). This is the **`textfield` fold again, not the `range` case.** Run through the sharpened D-096 (root + value-type + **shared skin/surface**, the N-042 amendment): (1) **root** — all five are `<input type=…>`, shared; (2) **value-type** — plain `bind:value` binds the element's `.value` **string** for every one (`"2026-06-28"` / `"13:45"` / `"2026-06-28T13:45"` / `"2026-06"` / `"2026-W26"`), all **string** — the discriminator that kept `number` separate (numeric) does not bite; (3) **skin/surface** — identical authored box + identical prop surface, differing **only** in UA-supplied picker chrome (calendar / clock / both), exactly the `textfield` situation (UA validation/keyboard/masking). Passes the sharpened criterion cleanly → fold. **No `DECISIONS.md` fold-entry** — this *applies* D-096, no amendment. (D-096 already pre-named `date`/`color`/`file` among own atomics *vs* `textfield` — "structured value / native chrome"; the **new** question resolved here is the *intra-family* one.)

**Honest counter (aired, not fold-breaking).** Each type's string is a different structured *format*, so a consumer must know `type` to interpret `value`. Resolved exactly as `textfield` did — the getter carries `{ type, value }`, so `type` travels with the value through the N-024 registry.

**Component.** `ui/core/lib/components/data-independent/date.svelte`. Root `<input {type} bind:value use:envelope>`; `value = $bindable('')` typed `string` — **empty = `''`**, always-string (the clean divergence from `number`'s empty=`null`); getter `() => $state.snapshot({ type, value })`; zero `<style>`. Prop surface = the control vocabulary, date-shaped: keep `value`/`disabled`/`readonly`/`id`/`name`; **add** `min`/`max` (native date/time-string shaping attrs) + `step` (native increment — days/seconds/months per type), type-appropriate values the consumer's job (the `number` precedent); **drop** `placeholder` (native date inputs ignore it) + `pattern` (no native `pattern`). Value is plain `bind:value` (string), **not** `bind:valueAsDate` (`Date | null` is serialization-hostile; string is wire-clean) — `valueAsDate` reserved, not built. **No processor seam** — a structured native value, not free-text entry (the numeric-formatting consumer is `number`). The native picker is the affordance; a custom date-picker dropdown is a later **composite**, not this. `readonly` is authored as a native pass-through but **not sampler-exercised** (engine-variable on date inputs) — a flagged build caveat, not a blocker.

**Skin (own `.date`).** Own key assembled from the L2 vocabulary (the `.number`/`.textarea`/`.select` precedent — per-class clarity > DRY): the `.number` box (`min-height:--ctl-h`, `--s` bg, `--s5` border, `--rad`, `--t`, `--fs-1`), `:focus-visible` → `--accent2` border + `--focus-ring`, `:disabled` greyed, `:read-only` → `--s2`/`--s4`, and **`:invalid` kept** → `--err` (native min/max range validation, the `.number` precedent). The `::-webkit-calendar-picker-indicator` is the click target; **color-scheme:dark on :root (N-043, added FOR this family) is now exercised** — it paints the picker popup + glyph dark for free, so the skin only sets the indicator cursor. PROVISIONAL (Joe live-tunes via HMR).

**Sampler.** A `date` row + **7** cells — the five types (`date#default`/`#time`/`#datetime`/`#month`/`#week`, exhibiting the fold) + `date#disabled` + `date#invalid` (min/max out of range). Matrix **22 → 29**.

**Verify (Chat self-drove, real `tauri dev` + CDP 9422, both accents via skin-swap).** `ids().length === 29` (7 `date#`). **Fold proof, registry:** every `date#…` reports component `"type":"date"` (one component) carrying its own input type (`date`/`time`/`datetime-local`/`month`/`week`), value a **string** each. **Fold proof, bind path:** dispatched a real `input` (`"2026-12-31"`, N-029) on `date#default` → bound getter `{type:"date",value:"2026-06-28"}` → `{…,value:"2026-12-31"}` — a JSON **string** round-trips through `bind:value` (the analogue of `number`'s 42/7). Computed-style: `{tag:"INPUT",type:"date",minHeight:"28px"(--ctl-h),fontSize:"12px"(--fs-1),color:"rgb(236,233,225)"(--t),borderRadius:"6px"(--rad)}`. `:invalid` specific: `date#invalid` border `rgb(138,42,42)` (`--err`) + `:invalid` true; `date#default` stays `rgb(52,59,71)` (`--s5`) + `:invalid` false. Pseudo verified by **stylesheet-rule inspection** (N-042 method): all **6** `.date` rules (base + `:focus-visible` + `:disabled` + `:read-only` + `:invalid` + `::-webkit-calendar-picker-indicator`) parsed + in cascade. Skin-swap: `--accent2` `#c28840` (gold, `--pr2`, client) ↔ `#3a7ab0` (blue, `--inf2`, node). Screenshot (client) eye-checked: the row renders all five native pickers (date `31/12/2026`, time `13:45`, datetime `28/06/2026 13:45`, month `June 2026`, week `Week 26, 2026`), disabled greyed, invalid red-bordered, indicators dark-themed. Clean teardown (5175/9422 free, 0 orphans).

**Standing rule (the sampler-DoD, → D-097 note).** From `date` onward a component milestone is **not done** until its sampler row + applicable-state cells are added and CDP-verified **in the sampler** — this **replaces** the old "wire a demo into both real shells" step entirely (the real apps are for integration + two-apps-together interaction only). Recorded canonically as a one-line closing note on **D-097** (its home).

*UI-implementation record. No protocol/data implication. `date` is the eleventh built `core` component; the date-input family folds (applies D-096, no amendment). The sampler-DoD is now standing (→ D-097). Next: remaining atomic di per N-038 — `color` (own atomic, native chrome) / `file` (new `bind:files` shape) / `select multiple`.*

### N-047 — `color` built (M-RP2.17): the twelfth `core` component; a SINGLETON that stands alone (the `range` case, not a fold); the native picker is not skinnable → a `color-picker` composite is logged (#2)

M-RP2.17 closed (J-426) — `color` (di·A, atomic `<input type="color">`, native swatch + picker) authored + skinned in one pass, built/tuned/verified **in the sampler** (D-097, now the standing DoD). The **twelfth** `core` component, the next atomic di per N-038 (catalogue row *color*).

**Why own atomic (the design point).** `color` has **no siblings** (unlike date's five), so the fold test is *sideways* — `color` vs `date`/`range`. Through the sharpened D-096 (root + value-type + **shared skin/surface**, N-042): it shares the `<input>` root **and** value-type (a **string**, `#rrggbb`) with `date` — root + value-type alone would pull toward a date fold, exactly the trap the sharpened criterion exists for — but it diverges on **skin/surface**: a **swatch** (`::-webkit-color-swatch` + `::-webkit-color-swatch-wrapper`), nothing shared with date's text-box + calendar indicator, and a different prop surface (no min/max/step, no `:invalid`). So it is an **own atomic: the `range` case, not the `textfield` case** (shares root + value-type with a sibling but stands alone on disjoint skin). **Applies D-096, no amendment** (no `DECISIONS.md` touch).

**Component.** `ui/core/lib/components/data-independent/color.svelte`. Root `<input type="color" bind:value use:envelope>`; `value = $bindable('#000000')` typed `string` — **always a valid 7-char `#rrggbb`**, the native control has no empty state, so default `#000000`, never `''` (the date divergence) or `null` (the number divergence): the **always-valued** shape, like `range`. Getter `{value}` — **no `type`** (singleton; type fixed), unlike date's `{type,value}`. Prop surface = the **leanest atomic yet**: keep `value`/`disabled`/`id`/`name`; **drop** `placeholder`/`pattern` (n/a), `readonly` (native no-op on color — the range precedent), `min`/`max`/`step` (n/a), `:invalid` (always a valid hex, never invalid), `type` (fixed). **No processor seam** (a swatch pick, not typed entry — the range reasoning). `alpha`/`colorspace` (`#rrggbbaa`) reserved, not built.

**The native picker is not skinnable → composite #2 logged.** The OPEN picker dialog (saturation square / hue slider / eyedropper / hex+RGB fields / preset swatches) is OS/Chromium-painted; the `.color` skin styles **only the closed-state swatch**. A themed custom palette (matching the gold/blue shell) is the deferred **`color-picker` composite** (the `password-field`-off-`textfield` shape) — logged in the components registry's Composites section + the ROADMAP, **not built** here. `color-scheme:dark` (N-043) is largely moot for `color` — the picker dialog is OS-painted, not webview-painted.

**Skin (own `.color`, pseudo-element-heavy like `.range`).** `appearance:none` + `-webkit-appearance:none`, then the swatch pseudos: `::-webkit-color-swatch-wrapper { padding:0 }` (kills native inset) + `::-webkit-color-swatch { border:none; border-radius: calc(var(--rad) - 1px) }` (the colour fill). The swatch is **compact** (fixed 36×24, **no `--ctl-h`** — as `.range`/`.textarea` dropped it; Joe may swap to `--ctl-h` for row-height parity), `--s` box, `--s5` border, `--rad`; `:focus-visible` → `--accent2` border + `--focus-ring`; `:disabled` → `cursor:not-allowed` + `opacity:0.5` (swatch still shows colour). No `:invalid`, no new `:root` token. PROVISIONAL (Joe live-tunes via HMR).

**Sampler.** A `color` row + **2** cells — `color#default` (seed `#9a6a30` gold) + `color#disabled` (seed `#2a6090` blue); no invalid/type variants exist (ragged-honest). Matrix **29 → 31**.

**Verify (Chat self-drove, real `tauri dev` + CDP 9422, both accents via skin-swap).** `ids().length === 31` (2 `color#`); both report component `"type":"color"`, value a `#rrggbb` string. **Bind path:** dispatched a real `input` (`"#123456"`, N-029) on `color#default` → bound getter `{value:"#123456"}` (a string round-trips through `bind:value`). Computed-style: `{tag:"INPUT",type:"color",appearance:"none",webkitAppearance:"none",width:"36px",height:"24px",borderRadius:"6px",cursor:"pointer"}`. Disabled: `color#disabled` `{isDisabled:true,cursor:"not-allowed",opacity:"0.5"}`. Pseudo skin verified by **stylesheet-rule inspection** (N-042 method): all **5** `.color` rules (`.color`, `:focus-visible`, `:disabled`, `::-webkit-color-swatch-wrapper`, `::-webkit-color-swatch`) parsed + in cascade. Skin-swap: `--accent2` `#c28840` (client) ↔ `#3a7ab0` (node). Screenshot (client) eye-checked: both swatches render (`#disabled` dimmed) **and** incidentally caught the native Chromium picker open (saturation square / hue / eyedropper / RGB fields) — the real native dialog. Clean teardown (5175/9422 free, 0 orphans).

**Verify finding (honest — the first interactive native-popup control exposes it).** During the minimized-window CDP session `color#default`'s value **drifted** from its `#9a6a30` seed (read `#419584`, then the dispatched `#123456`, then a stray green at screenshot) — stray pointer events on the swatch kept opening the native picker and changing the colour. `color#disabled` (non-interactive) held its **exact** `#2a6090` seed throughout. That asymmetry is the proof seeding + bind are **correct** (a seeding bug would have broken disabled too); a fresh user load gets `#9a6a30`. Going forward, interactive native-popup controls (`color`, and the `date` pickers) can self-mutate under stray events in a minimized verify window — read the **non-interactive** cell for the stable-seed proof and the **dispatched round-trip** for the bind proof, not the post-session swatch value.

*UI-implementation record. No protocol/data implication. `color` is the twelfth built `core` component; own atomic on the sharpened criterion (the range case), applies D-096 no amendment. Logged: the `color-picker` composite (#2, themed palette, deferred). Next: remaining atomic di per N-038 — `file` (new `bind:files` shape) / `select multiple`.*

### N-048 — `file` built (M-RP2.18): the thirteenth `core` component; the FIRST non-`value` binding (`bind:files` / FileList) — the 4th binding shape, and the first value-type `$state.snapshot` can't serialise; a `file-field` composite is logged

M-RP2.18 closed (J-427) — `file` (di·A, atomic `<input type="file">`, native picker button) authored + skinned in one pass, built/tuned/verified **in the sampler** (D-097). The **thirteenth** `core` component, the next atomic di per N-038 (catalogue row *file-select*).

**The headline is the binding shape, not the fold.** Own atomic is obvious — `<input type="file">` binds a **FileList**, not a string/number/boolean; no fold candidate (date/color differ entirely; no string/number siblings). Applies D-096, no amendment. The real event is that `file` is the **first non-`value` binding** in the library: `bind:files`, the **4th binding shape** after boolean-in (`checked`, toggle) / event-out (`onclick`, button) / string-in (`value`, the whole input family incl. date/color). The substrate (`envelope`/`debug`, N-023/N-024) had been proven across all of those — but every prior binding rode `value`/`checked`/`onclick`. It is also the first value-type **`$state.snapshot` cannot serialise**: a FileList is a live host object, not a plain object/proxy.

**Getter (the design point) — de-FileList.** The getter returns a **plain** view, not the FileList: `{ count, files: [{ name, size, type }] }` — `count: files?.length ?? 0`, `files` mapped via `Array.from(files)`. The **bindable prop carries the live FileList** (the consumer's real value); the getter is the serialisable projection for the N-024 registry / CDP `returnByValue`. (`$state.snapshot(files)` would not flatten a host FileList — the explicit map is required.)

**Component.** `ui/core/lib/components/data-independent/file.svelte`. Root `<input type="file" bind:files use:envelope>`; `files = $bindable(null)` typed `FileList | null` (**empty = `null`**). Prop surface: keep `files`/`accept`/`multiple`/`disabled`/`id`/`name`; **drop** `value` (**unsettable programmatically** — browser security; the consumer reads via the FileList binding, never writes `.value`), `placeholder`/`pattern`/`readonly`/`min`/`max`/`step` (n/a), `type` (fixed). `capture` (mobile camera) reserved, not built. **No processor seam** (a file pick, not typed entry). Selection fires **`change`**, NOT `input` — `bind:files` updates on change.

**The native file row is minimally skinnable → composite logged.** `<input type=file>` renders a **button** + a UA **"No file chosen"** text. The `.file` skin styles the **button pseudo** to match `.button`; the surrounding text is UA-rendered (the element's `color`/`--fs-1` nudge it, but it is not fully controllable) — accepted. A custom drag-drop file row (zone + selected-file list + remove + upload progress) is the deferred **`file-field` / `dropzone` composite** (the `color-picker`/`password-field` shape) — logged in the registry Composites + ROADMAP, not built.

**Skin (own `.file`).** The button pseudo styled to `.button` (`--ctl-h`, `--sp-1 --sp-4` padding, `--s4` bg, `--s5` border, `--rad`, `--t2`, `--fs-1`, pointer) in **both** spellings — `::file-selector-button` (standard) **and** `::-webkit-file-upload-button` (legacy) — as **separate rules** (a selector list with an unknown pseudo drops the whole rule, so they cannot be comma-combined; this is the divergence from the comma-combinable class selectors). `:hover` / `:focus-visible::file-selector-button` → accent border + `--focus-ring`; `:disabled` → greyed + `not-allowed`. The element itself carries `--t2`/`--fs-1` for the UA text. No new `:root` token. PROVISIONAL (Joe live-tunes).

**Sampler.** A `file` row + **3** cells — `file#default` (single) + `file#multiple` (`multiple`) + `file#disabled`. All start `null` (a file is unsettable from markup — honest-empty, no seeded variant). Matrix **31 → 34**.

**Verify (Chat self-drove, real `tauri dev` + CDP 9422, both accents via skin-swap).** `ids().length === 34` (3 `file#`), baselines `{count:0,files:[]}`. **Bind path (the FileList round-trip — the headline proof):** `value` is unsettable, so injected a real file via `DataTransfer` and dispatched **`change`** (file inputs fire `change`, not `input`): `const dt = new DataTransfer(); dt.items.add(new File(['x'],'test.txt',{type:'text/plain'})); el.files = dt.files; el.dispatchEvent(new Event('change',{bubbles:true}))` → bound getter `{count:1,files:[{name:"test.txt",size:1,type:"text/plain"}]}` — **a FileList round-trips through `bind:files`, de-FileLuted to plain metadata** (the substrate's first non-`value` binding, proven). `file#multiple`/`file#disabled` stayed `{count:0,files:[]}` (no injection). `multiple`: `file#multiple.multiple === true`, `file#default === false`. Computed-style: `{tag:"INPUT",type:"file",fontSize:"12px",color:"rgb(200, 196, 188)"(--t2),cursor:"pointer"}`. Disabled: `file#disabled` `{disabled:true,cursor:"not-allowed",opacity:"0.5"}`. Pseudo skin verified by **stylesheet-rule inspection** (N-042 method): all **8** `.file` rules (`.file`, `::file-selector-button`, `::-webkit-file-upload-button`, `::file-selector-button:hover`, `:focus-visible`, `:focus-visible::file-selector-button`, `:disabled`, `:disabled::file-selector-button`) parsed + in cascade. Skin-swap: `--accent2` `#c28840` (client) ↔ `#3a7ab0` (node). Screenshot (client) eye-checked: `file#default` renders the `.button`-styled "Choose File" + **"test.txt"** (the round-tripped name), `file#multiple` shows native "Choose **Files**" (plural), `file#disabled` greyed. (Incidental reconfirm: `color#default` showed its exact `#9a6a30` seed on this fresh load — closing out the N-047 churn as instance-state, not a defect.) Clean teardown (5175/9422 free, 0 orphans).

*UI-implementation record. No protocol/data implication. `file` is the thirteenth built `core` component; own atomic + the first `bind:files`/FileList binding shape (applies D-096, no amendment). Logged: the `file-field`/`dropzone` composite (deferred). Next: the last remaining atomic di per N-038 — `select multiple` — then the text-processor engine, then dd-components.*

## 2026-06-29

### N-049 — `led` (di display-di status light) + `link` (di navigation atomic) + `status-indicator` (di composite) — catalogue concept-lock

A design conversation (Joe) added three components to the di catalogue. **Planning/concept-lock only — nothing built; logged in `ui/docs/xgen-ui-components.md` v0.24, pointing here.** No `DECISIONS.md` touch (arc-local di-vocabulary decisions; D-069 three-instance threshold not met).

**Origin.** Joe saw the real shells carry a bespoke status light — the `.state-dot` driven by `dotColor(state)` + `isPulsing(state)` (READY→`--ok`, DEGRADED→`--pr`, DISCONNECTED→`--err`, …) — and asked whether an LED-style state indicator is in the plan. It was not (the di tables cover the interactive set + the display-di trio label/paragraph/image; the only deferred read-only primitives noted were `<progress>`/`<meter>`/`<output>`). It is a real gap; it slots in along the existing di/dd + atomic/composite axes.

**`led` — di·A, simple display-di (the 4th, after label/paragraph/image).** Atomic **inline `<span class="led">`** status light (no native HTML element exists for a status light; `<span>` is the neutral inline root, chosen primarily for composite use beside a label; `<output>` is avoided — reserved for the deferred dd progress/meter/output primitives). **Key design point — a caller-supplied colour map, not hardcoded states/colours** (the `select` options-prop precedent, N-034: the atomic carries caller-supplied content it does not interpret → fully data-independent). Locked API:
- **`states: Record<string,string>`** — the map, e.g. `{ "ON": "#ff0000", "OFF": "var(--t4)" }`. Values accept **hex OR `var(--token)`** — the consumer chooses to hardcode a colour or ride the skin tokens, so the atomic stays colour-agnostic while the shells can still theme.
- **`state: string`** — the current key; selects which colour shows.
- **`pulse?: boolean`** — optional animation, **orthogonal to colour** (a separate boolean now; a map *value* could later grow to `{colour, pulse}` for per-state pulsing if wanted).
- Resolve: `colour = states[state] ?? "#000000"` — **full black `#000000` is the reserved unknown/undefined sentinel** (Joe-corrected from an earlier grey idea; a transparent dot would *disappear*, so the fallback is an always-visible solid). **Consumers must never map a real state to `#000000`** (the contract lands on the caller; written into the catalogue row + the future `led.svelte` header so it is not a silent trap).
- **`title = state ?? "?"`** — native hover tooltip shows the live key ("ON"/"READY"/…), self-documenting at runtime; a *set-but-unmapped* key still shows in the tooltip (more diagnostic — you see which unknown state slipped through), only *truly-undefined* shows `"?"`. (Replaces a DEV-warn idea — the visible tooltip is better; a DEV-warn could still be added on top if wanted.)
- Getter **`{ state, colour }`** (the resolved pair). `role="img"` + `aria-label={title}` (colour is not the only signal — a standalone `led` stays accessible).
- Skin: `.led` owns **shape only** (size, `border-radius:50%`, pulse `@keyframes`); **colour rides an inline CSS var** set from the prop — clean L2 split (the skin never hardcodes a state colour).

**`link` — di·A, navigation semantic (new catalogue row).** Atomic **`<a href>`** — surfaced because the `status-indicator` wanted an optional "details →" affordance and no link component existed. Value-carrying (a `text` label) **and** navigational (`href`), with optional `onclick` for in-app/SPA routing (jump to a settings section). **Distinct from the existing *button link-styled shape*** (a `<button>` that *looks* like a link but acts via `onclick`, no navigation) — `link` **is** an `<a>`; both are valid and must not be conflated. Full prop surface (`href`/`text`/`target`/`rel`/external-vs-in-app/inert handling — `<a>` has no native `disabled`) is its **own design walk at build time**, not locked here.

**`status-indicator` — di COMPOSITE (not dd).** `<div class="status-indicator">`, keyed *status*, binding none = **`led` + `label` + optional trailing `link`** (the slotted "details →"). **The classification correction (Joe):** an earlier draft put this in the dd table; it is **di**. The di/dd line is *does the component interpret domain data* — here the **caller** supplies each row's `states`/`state`/caption/link (the `combobox`/`password-field`/`select`-options shape); the component binds no domain structure. A node settings/overview panel is simply **N `status-indicator` rows**, the panel author feeding each. It only becomes **dd** if a future component *binds* a domain structure (e.g. a `node-health-panel` bound to the node's health record that derives READY→ok itself) — that is a separate, later thing for the dd table. The shells' `.state-dot` + label becomes the minimal `status-indicator` (led + label, no link) when the shells move to lib components.

**Build order.** `led` (and `link`) land in the di track; sequence: after `select multiple` (the last input-family atomic), then `led` + `link`, then `status-indicator` once both `led` + `label` are in hand. Each is built/tuned/CDP-verified in the sampler (D-097) like every component since `date`.

*UI-catalogue record. No protocol/data implication, nothing built. Adds `led`/`link`/`status-indicator` to the di catalogue (components registry v0.24). Next build remains `select multiple`; `led`/`link`/`status-indicator` queued after.*

### N-050 — `select-multiple` built (M-RP2.19): the fourteenth `core` component and the LAST input-family atomic di; the FIRST plain-array value-type (`bind:value` → `string[]`), empty model `[]` not `null`, getter `{values,count}`; own atomic under the sharpened D-096

M-RP2.19 closed (J-430) — `select-multiple` (di·A, atomic `<select multiple>`) authored + skinned in one pass, built/tuned/verified **in the sampler** (D-097). The **fourteenth** `core` component and the **last input-family atomic di** (N-038); with it the input-family atomic axis is closed.

**Own atomic under the sharpened D-096 — two of three criteria fail.** It shares the `<select>` tag with `select`, so the *literal* tag test would fold it in; it does not. The sharpened criterion (root + value-type + shared skin/surface, N-042) fails on **two** clauses: value-type diverges (**`string[]`** vs `select`'s scalar `string`) **and** skin-surface diverges (a static scrolling **list-box**, not a dropdown that opens a popup). The `range`-vs-`number` logic, doubled. **Applies D-096, no amendment** (no `DECISIONS.md` touch).

**The headline is the value-type: the FIRST plain array.** Every prior binding was a scalar or a host object — boolean-in (`checked`) / event-out (`onclick`) / string-in (`value`) / number / FileList (`bind:files`). `select-multiple` is the **first plain-array** value-type: `bind:value` on `<select multiple>` yields a native **`string[]`** — the **5th binding shape**. No `bind:group` (that is for checkbox/radio sets); `bind:value` is the clean native path. Unlike FileList, a plain array **is** `$state.snapshot`-serialisable (a proxy, not a host object), so the getter is trivial — the new ground is the **empty model**, not serialisation.

**Empty model `[]`, not `null` (the design point).** Single `select`'s empty is `null` (scalar-absent); `select-multiple`'s empty is **`[]`** (empty *set*). An array prop is always an array, so a consumer `.length`/`.map`s with no null-guard. The divergence from the sibling is correct and deliberate — this is the N-038 array-value-type landing.

**Component.** `ui/core/lib/components/data-independent/select-multiple.svelte` (`lang="ts"`). Root `<select multiple {size} bind:value use:envelope>`; `value = $bindable([])` typed `string[]` (**empty = `[]`**). Prop surface: `value`/`options`/`size`/`disabled`/`id`/`name`. **`multiple` is hardcoded** — the component's identity, not a prop. **`size?` default 4** (visible list-box rows — the one genuinely multi-specific knob). **`options` carries over UNCHANGED from `select`** (N-034): the same dual input shape (`string[]` or `{value,label?,disabled?}[]`) normalized via the same `$derived items` — the two siblings stay API-symmetric on options. **No `placeholder`** (a leading empty option is meaningless for a list-box). No processor seam (a pick, not typed entry). Getter `{ values: $state.snapshot(value), count: value.length }` — the `{count, …}` shape mirrors `file`'s for sampler-row consistency.

**Skin (own `.select-multiple`).** A list-box **surface** (`--s4`/`--s5` box, `--rad`, `--fs-1`/`--lh`, `--sp-1` padding) — **no** `appearance:none`/arrow (it is not a dropdown), **no** `--ctl-h` (`size` drives height). Selected rows are accent-tinted via **`.select-multiple option:checked`** → `--accent2`/`--accent-ink`; `:focus-visible` → accent border + `--focus-ring`; `:disabled` → greyed + `not-allowed`. No new `:root` token. The skin shares **nothing** with `.select` beyond the L2 vocabulary (assembled per-class, the `.number`/`.textfield` precedent). PROVISIONAL (Joe live-tunes via HMR).

**Sampler.** A `select-multiple` row + **3** cells — `select-multiple#default` (`value=[]`), `select-multiple#seeded` (`value=['a','c']`), `select-multiple#disabled` (`disabled`, `value=['b']`); all share a small `a/b/c` options array. Unlike `file`, an array **CAN** seed from markup — the `#seeded` cell is the honest array-round-trip seed. Matrix **34 → 37**.

**Verify (Chat self-drove, sampler + CDP 9422, both accents via skin-swap).** `ids().length === 37` (3 `select-multiple#`). **Seed proof (the `[]` empty-model + array shape):** `#default {values:[],count:0}` (empty = `[]`, **not** `null`), `#seeded {values:["a","c"],count:2}`, `#disabled {values:["b"],count:1}`. **Bind path (the array round-trip — the headline proof):** selected rows a+b via DOM + dispatched **`change`** (`<select>` fires `change`) → bound getter `{values:["a","b"],count:2}` — **a `string[]` round-trips through `bind:value`** (the substrate's first plain-array value-type, proven). Element: `tag=SELECT`, `multiple=true`, `size=4` (the default). Pseudo/option skin verified by **stylesheet-rule inspection** (N-042 method): all **5** `.select-multiple` rules (`.select-multiple`, `option`, `option:checked`, `:focus-visible`, `:disabled`) parsed + in cascade. Disabled: `#disabled.disabled === true`. Skin-swap: `--accent2` `#c28840` (client) ↔ `#3a7ab0` (node). Screenshots (both shells eye-checked): the three list-boxes render dark-surface with **accent-tinted selected rows** (gold client / blue node), `#seeded` shows Alpha highlighted + Gamma below the 4-row fold, `#disabled` greyed. **Verify-harness note:** a fresh launch was needed — a stale HMR session first reported `#default`/`#seeded` both as `['a','b']`; teardown + relaunch gave the correct seeds, confirming the prior read was stale dev-state, not a binding defect (the `range`/`color` minimized-window finding family, N-047 shape).

*UI-implementation record. No protocol/data implication. `select-multiple` is the fourteenth built `core` component; own atomic + the first `string[]` array value-type (applies sharpened D-096, no amendment). The input-family atomic axis is now closed. Next: the di catalogue additions `led` + `link` (N-049), then the `status-indicator` composite, then the text-processor engine, then dd-components.*

### N-051 — `led` built (M-RP2.20): the fifteenth `core` component and the FOURTH simple display-di; the FIRST caller-supplied-colour-map + the FIRST data-coloured atomic (colour rides an inline CSS var, not the accent); `#000000` unknown-sentinel contract

M-RP2.20 closed (J-431) — `led` (di·A, atomic inline `<span class="led">` status light) authored + skinned in one pass, built/tuned/verified **in the sampler** (D-097). The **fifteenth** `core` component and the **fourth simple display-di** (after label/paragraph/image, N-032). Executes the N-049 concept-lock verbatim.

**Two firsts.** (1) **Caller-supplied colour map** — the `select` options-prop shape (N-034) applied to a display-di: the atomic carries a `states: Record<string,string>` map it does **not** interpret, picking a colour by the `state` key. Fully data-independent; the shells' bespoke `.state-dot` + `dotColor(state)` switch becomes this, generalised. (2) **First data-coloured atomic** — every prior component's colour came from the skin (`--accent*`/`--t*`/`--err`); `led`'s colour comes from the **prop**, injected as an inline `--led-colour` custom property the `.led` skin reads. The skin owns **shape only**. Consequence: `led` is the **first component whose colour is NOT accent-derived** — the skin-swap re-themes shell chrome but the dots keep their mapped colours (verified, below).

**The contract (lands on the caller).** `colour = states[state] ?? "#000000"` — **full black `#000000` is the reserved unknown/undefined sentinel** (an always-visible solid; a transparent dot would disappear). **Consumers must never map a real state to `#000000`** — written into the `.svelte` header so it is not a silent trap. `title = state ?? "?"` (native tooltip shows the live key; a set-but-unmapped key still shows — diagnostic; only truly-undefined shows `"?"`). `role="img"` + `aria-label={title}` keeps a standalone led accessible (colour is not the only signal). Map values accept **hex OR `var(--token)`** (consumer hardcodes or rides the skin tokens).

**Component.** `ui/core/lib/components/data-independent/led.svelte` (`lang="ts"`). Root `<span use:envelope role="img" aria-label={title} {title} data-pulse={pulse || undefined} style="--led-colour: {colour}">`. Plain props (display-di, **no `$bindable`**): `states` (default `{}`), `state`, `pulse` (default `false`), `id`. Derived: `colour = states[state] ?? '#000000'`, `title = state ?? '?'`. Getter `{ state: state ?? null, colour }` — the resolved `colour` may be a `var(--token)` string (correct; the computed rgb is a separate computed-style concern). No processor seam. **`pulse` via a reflected `data-pulse` attribute** (the `.toggle[role="switch"]` attribute-hook precedent, since `envelope` owns `class`).

**Skin (own `.led`).** Shape only: `display:inline-block`, fixed `10px` round dot (`border-radius:50%`), `background: var(--led-colour, #000000)` (the inline prop var; the `#000000` fallback doubles the sentinel), `vertical-align:middle`. `.led[data-pulse]` → `animation: led-pulse 1.2s ease-in-out infinite`; `@keyframes led-pulse` opacity 1↔0.35. No `:hover`/`:focus`/`:disabled` (non-interactive). Dot size is a skin literal (no new `:root` token unless a second consumer needs it, D-069). PROVISIONAL (Joe live-tunes via HMR).

**Sampler.** A `led` row in the Display section + **4** cells, all sharing `ledStates = { ON:'#22c55e', OFF:'var(--t4)', ERR:'var(--err)' }` (both value kinds): `led#default` (`ON`, green hex), `led#off` (`OFF`, grey token), `led#pulse` (`ERR`, red token, pulsing), `led#unknown` (`???`, the black sentinel). Matrix **37 → 41**. (Display-di — no disabled/invalid states; honest-ragged like the trio.)

**Verify (Chat self-drove, sampler + CDP 9422; real output quoted, Rule 2).**
- Registry (`-Mode state`, fresh launch): `n_ids=41`; `led#default = {"state":"ON","colour":"#22c55e"}`; `led#off = {"state":"OFF","colour":"var(--t4)"}` (the token reference travels in the getter); `led#pulse = {"state":"ERR","colour":"var(--err)"}`; `led#unknown = {"state":"???","colour":"#000000"}` (**the black sentinel for an unmapped key — the contract proof**).
- Computed colour / pulse / a11y / skin / no-accent (one eval): `{"bg":{"def":"rgb(34, 197, 94)","off":"rgb(88, 92, 100)","unk":"rgb(0, 0, 0)"},"pulse":{"pAnim":"led-pulse","dAnim":"none","pData":"true","dData":null},"a11y":{"tag":"SPAN","role":"img","aDef":"ON","aUnk":"???","title":"ON","radius":"50%","disp":"block"},"ledRules":[…,".led",".led[data-pulse]"],"kf":true,"noAccent":{"client":"rgb(34, 197, 94)","node":"rgb(34, 197, 94)"}}`. The inline `--led-colour` drives `.led` background incl. the `var(--t4)` token path (`rgb(88,92,100)`); the sentinel renders `rgb(0,0,0)`; `data-pulse` + `led-pulse` keyframes applied; `.led` + `.led[data-pulse]` + the keyframes (`kf:true`) in cascade. **`noAccent`: `#default` background `rgb(34,197,94)` identical client↔node** — the no-accent-dependency proof, `led` breaks the accent-swap pattern by design.
- Screenshot (eye-checked): four round dots — green / grey / red (pulsing) / **black** (`#unknown` visibly renders, does not vanish).
- Teardown: `0 orphans - ports 9422/5175 free`.

**Finding (display-di, expected).** Computed `display` is `block`, not the skin's `inline-block` — **flex-item blockification** from the sampler cell's flex layout, not a skin rule (the same `label` finding, N-035). Not a defect.

*UI-implementation record. No protocol/data implication. `led` is the fifteenth built `core` component, the fourth simple display-di; first caller-supplied-colour-map + first data-coloured atomic (colour-as-data via inline CSS var, no accent dependency). Applies D-096, no amendment (the map is the N-034 precedent). Next: `link` (navigation `<a href>` atomic, its own design walk) → the `status-indicator` di composite (once `led` + `label` are both in hand) → the text-processor engine → dd-components.*

### N-052 — `link` built (M-RP2.21): the sixteenth `core` component and the FIRST navigation-kind di (atomic `<a href>`); commits the `<a>`-vs-`<button>` split; synthesised `disabled`; bundled-safe `external` rel; returns to accent-derived colour

M-RP2.21 closed (J-432) — `link` (di·A, atomic `<a href>`) authored + skinned in one pass, built/tuned/verified **in the sampler** (D-097). The **sixteenth** `core` component and the **first navigation-kind di** — a new kind alongside interactive (toggle/button/textfield/…) and display (label/paragraph/image/led). Surfaced by the `status-indicator` composite wanting a "details →" affordance (N-049); the prop surface was deferred to a build-time design walk (this session), now locked.

**A new kind, and the `<a>`-vs-`<button>` commit.** `link` neither binds an editable value (interactive) nor is purely read-only (display): it **acts** (navigates) while carrying a `text` label. The standing tension (N-049) was navigation-`<a>` vs an action that merely *looks* like a link (a `<button>` link-styled shape). `link` **is** an `<a>` with a real `href`; the look-alike stays `button`. They must never be conflated.

**Three notable mechanics.** (1) **Synthesised `disabled`** — an `<a>` has no native `disabled`, so `disabled` **drops `href`** (renders a non-navigating `<a>`), sets `aria-disabled="true"` + `tabindex=-1` (non-focusable), and blocks `onclick`; the skin greys via `[aria-disabled]`. The first component to *fake* a native-absent state rather than pass one through. (2) **Bundled-safe `external`** — `external={true}` auto-sets `target="_blank"` **and** `rel="noopener noreferrer"` so the unsafe bare-`_blank` default never reaches a consumer; no raw `target`/`rel` props exposed. (3) **Returns to accent-derived colour** — `.link` `color: var(--accent2)` re-themes gold/blue per shell. This confirms `led`'s caller-supplied colour (N-051) was the deliberate one-off, not a turn.

**Component.** `ui/core/lib/components/data-independent/link.svelte` (`lang="ts"`). Props: `href` (req), `text` (req; `""` allowed for icon-only), `onclick?`, `external?`, `disabled?`, `ariaLabel?`, `id`. Derived `effectiveHref = disabled ? undefined : href`, `target`/`rel` from `external`. Getter `{ text, href, external, disabled }` (carries the **prop** `href` even when `disabled` drops it from the rendered element). DEV-warn when `text===""` && no `ariaLabel` (no accessible name — the `image`-`alt` guard shape). No `$bindable`, no processor seam, **no Tauri/router import**.

**Consumer-wiring (the atomic stays dumb).** Leaving to the OS browser = the consumer's `onclick` → Tauri `shell.open(href)` (a raw `target="_blank"` inside a Tauri WebView can spawn a blank in-app webview, not the system browser); the real `href` is retained for a11y/right-click. An in-app SPA route = the consumer's `onclick` → router. **Opening a modal is NOT a `link`** (no destination) — that is a `button` flipping an open state; logged a future **`modal`/`dialog`** component (native `<dialog>` + `showModal()`, focus-trap, `::backdrop`, Esc-to-close) as the modal *surface*. Icon-only / icon+text need a future **`icon` primitive** (`ariaLabel` is the atomic hook; not faked in the sampler).

**Skin (own `.link`).** `color: var(--accent2)`, `text-decoration: none`, `cursor: pointer`; `:hover` underline; `:focus-visible` the accent focus ring; `[aria-disabled]` greyed (`--t4`) + no underline + default cursor + `pointer-events: none`. No new `:root` token. A compact/short shape + an icon shape are FUTURE skin shapes. PROVISIONAL.

**Sampler.** A `link` row in the Display section + **3** cells: `link#default` (`#settings`, in-webview), `link#external` (`https://xgen.example`, `external`, `ariaLabel`), `link#disabled` (greyed, `href` dropped). Matrix **41 → 44**. (Navigation di — no invalid state; icon-only deferred.)

**Verify (Chat self-drove, sampler + CDP 9422, fresh launch; real output quoted, Rule 2).**
- Registry (`-Mode state`): `n_ids=44`; `link#default {"text":"Settings","href":"#settings","external":false,"disabled":false}`; `link#external {"text":"xgen.example","href":"https://xgen.example","external":true,"disabled":false}`; `link#disabled {"text":"Unavailable","href":"#x","external":false,"disabled":true}`.
- Attributes / skin / accent (one eval): `{"def":{"tag":"A","href":"#settings","target":null,"rel":null,"aria":null},"ext":{"href":"https://xgen.example","target":"_blank","rel":"noopener noreferrer","al":"XGen site (opens externally)"},"dis":{"href":null,"aria":"true","tab":"-1","deco":"none","col":"rgb(88, 92, 100)"},"linkRules":[".link",".link:hover",".link:focus-visible",".link[aria-disabled]"],"accent":{"client":"rgb(194, 136, 64)","node":"rgb(58, 122, 176)"}}`. **`dis.href === null`** is the synthesised-disabled proof (href dropped) + `aria-disabled`/`tabindex=-1`/greyed `--t4`/no underline; external carries `_blank` + the safe `rel` + `aria-label`; all 4 `.link` rules in cascade; **`accent`: `#default` colour gold `rgb(194,136,64)` (client) ↔ blue `rgb(58,122,176)` (node)** — `link` rides the accent (the contrast to `led`).
- Screenshot (eye-checked): three links — accent "Settings", accent "xgen.example", greyed "Unavailable".
- Teardown: `0 orphans - ports 9422/5175 free`.

*UI-implementation record. No protocol/data implication. `link` is the sixteenth built `core` component, the first navigation-kind di; commits `<a>`-vs-`<button>`, synthesises `disabled`, bundles the safe `external` rel, returns to accent-derived colour. Applies D-096, no amendment. Logged: a future `modal`/`dialog` component (modal surface; trigger = `button`) + the future `icon` primitive. Next: the `status-indicator` di composite (led + label + optional link — all three now in hand) → the text-processor engine → dd-components.*

### N-053 — sampler tabbed by class×arity (M-RP3.2): four-panel container (di/dd × atomic/composite); all panels MOUNTED + CSS-hidden, never `{#if}`; pure sampler chrome

M-RP3.2 closed (J-433) — the sampler host (`ui/sampler/`, D-098) restructured from one long vertical scroll into a **four-panel tab container** keyed by the catalogue's class×arity axes. **Pure sampler chrome**: only `app_sampler.svelte` + `app.css`; **no `core`/`common` component touched, `skin.css` untouched**; matrix unchanged at **44**. Its own M-RP3.x sampler milestone (sibling to M-RP3.0/M-RP3.1) so the next component-build (`status-indicator`, M-RP2.22) drops into the already-tabbed di·composite panel.

**Four panels (the class×arity taxonomy).** **DI · atomic** (the current 16 components / 44 cells) · **DI · composite** (empty; first occupant `status-indicator`) · **DD · atomic** (empty) · **DD · composite** (empty). This makes the sampler finally mirror the component-index's own structure (di-atomic / di-composite / dd subsections). Tab labels track the index's *atomic* vocabulary (not *single*). The client/node skin-swap stays **global** tool chrome above the tabs (the existing `.sampler-bar`, untouched); the tab bar sits between it and the panels. The inner kind sub-headers stay inside DI·atomic, promoted to the three di kinds **INTERACTIVE / DISPLAY / NAVIGATION** (`link` moved from under Display to its own NAVIGATION header, aligning with N-049/N-052; no cell added/removed/re-bound). Empty panels carry an explicit `No components yet` placeholder so they read as intentional, not broken.

**The load-bearing decision: all panels stay MOUNTED; inactive hidden via CSS `display:none` (`class:hidden`), NEVER `{#if}`.** `envelope` registers into `window.__XGEN_DEBUG__` **only while a component is mounted** (grounded in `envelope.ts` — registration fires on the `use:` action). A `{#if}`-gated tab would **unmount** its panel and drop those ids from the registry, so `ids()` would read only the active tab and the CDP matrix-count invariant (D-097, the whole verify protocol) would silently break. CSS-hidden ≠ unmounted → the registry stays complete → self-drive is unchanged (no tab-clicking to register everything). A test-bed wants every component alive anyway. **This invariant is the reusable takeaway** — any future sampler partitioning (more tabs, accordions, lazy sections) must hide, never unmount. (Recorded arc-local; a D-069 promotion-watch only if it recurs as a cross-cutting rule.)

**Verify (Chat self-drove, sampler + CDP 9422, fresh launch; real output quoted, Rule 2).**
- Default tab (`-Mode state`): **all 44** instances enumerate with the other three panels mounted-but-hidden.
- Eval 1 (load): `{"n":44,"panels":["block","none","none","none"],"titles":["Interactive","Display","Navigation"],"empties":3,"tabs":["DI · atomic*","DI · composite","DD · atomic","DD · composite"],"navNext":"link"}`.
- Eval 2 (**the anti-`{#if}` proof**): after clicking DI·composite + a reactive-flush tick — `{"nAfterSwitch":44,"panels":["none","block","none","none"],"activeTab":"DI · composite","activeEmptyText":"No components yet","activeEmptyCode":"status-indicator"}`. `ids().length` **stays 44** through the switch (DI·atomic hidden but mounted).
- Eval 3 (skin-swap re-themes the active tab): `{"tabActiveClient":"rgb(154, 106, 48)","tabActiveNode":"rgb(42, 96, 144)"}` (gold ↔ blue).
- Screenshots eye-checked (DI·atomic grid; DI·composite placeholder). Teardown `0 orphans`.

**Harness finding (recorded).** Svelte 5 reactivity flushes effects **after** the current synchronous task — a same-`eval` read of `getComputedStyle`/`:not(.hidden)` right after `.click()` returns the **pre-update** DOM (the first eval-2 attempt read stale `["block","none","none","none"]` post-click — not a defect). Protocol: drive the switch in one `eval`, read panel state in a **separate** `eval` (next CDP call = next tick, effect flushed). Sibling to the N-050 stale-HMR finding, distinct cause (intra-session reactive flush, not stale dev-state).

*UI-implementation record. No protocol/data implication, no component change. Sampler chrome only (D-098). Establishes the all-mounted / CSS-hidden / never-`{#if}` invariant for sampler partitioning. Next: `status-indicator` (M-RP2.22, the first di composite) into the di·composite panel → text-processor → dd.*

### N-054 — `status-indicator` (M-RP2.22): the FIRST di composite — `<div class="status-indicator">` = led + label + optional link; the composite-registration model (aggregate getter + children self-register under stable ids); first cell to multiply the registry count

M-RP2.22 closed (J-434). `status-indicator` is the **seventeenth** `core` component and the **first di composite**. It founds the patterns every later composite reuses.

**Composite identity.** The root IS `<div class="status-indicator">` — the N-020/N-022 composite marker (a `<div class="type">` wrapper via `envelope`), structurally distinct from an atomic whose root is a native tag. It imports + composes the three real child atomics `led` (required) + `label` (required) + optional trailing `link`. **di**: the caller supplies the state→colour map, the caption, the link target; the composite interprets no domain structure (that interpretation is what a dd component does). Flat pass-through API: `states`/`state`/`pulse?`→led, `caption`→label, `linkHref?`/`linkText?`(default `"Details →"`)/`linkExternal?`/`onLinkClick?`→link.

**The composite-registration model (the reusable precedent).** The composite root registers ONE aggregate getter; the **child atomics self-register** — they each pass their own `debug` getter unconditionally, so `envelope` registers them whenever mounted (keyed `id ?? ordinal`). The composite hands them composite-derived **stable ids** `<id>__led`/`<id>__label`/`<id>__link` so the registry reads cleanly, not ordinal-noisy. **Zero changes to the three closed atomics** (D-065 — don't retrofit built components for a new one's convenience). Consequence: a composite row yields **multiple** `ids()` entries (composite + each child) — the **first time one sampler cell is more than one registry entry**, so the matrix count multiplies (this milestone: 3 cells → +11 entries → 44→55). All future composite accounting must reflect this; the matrix is registry-entry count, not cell count.

**Aggregate getter `{state, caption, hasLink}` — `colour` omitted.** The composite reports only what it owns; resolving `colour` would duplicate `led`'s `?? "#000000"` sentinel, so colour is verified on the `led#…__led` child entry instead (no logic duplication, no drift).

**`{#if linkHref}` is NOT the N-053 case.** The optional link is rendered only when `linkHref` is set — a genuine absent sub-element (a status row may carry no detail link). N-053's never-`{#if}` rule is about keeping every COMPONENT mounted for registry completeness across tabs; an absent optional link is correctly absent and registers nothing. Different concern, same syntax — don't conflate.

**Skin (`.status-indicator`, PROVISIONAL).** Flex row, `align-items:center`, `gap:var(--sp-2)`; the optional trailing `.link` is pushed to the row end via `> .link { margin-left:auto }`. No new token; the skin only lays out the composed atomics (each child carries its own skin). **Verify note (D-065):** the computed `margin-left` of the link reads `0px` in the sampler — the `auto` rule IS applied + in cascade, but a content-hugging test cell offers no free space for `auto` to absorb; the right-push manifests only when `.status-indicator` is a full-width row (its real use). Recorded as-is, not faked with a demo width.

**Verify (Chat self-drove, sampler + CDP 9422; real output, Rule 2).** `ids().length===55`; the three aggregate getters (`#default {ON,Connected,hasLink:false}` / `#withlink {OFF,Disconnected,hasLink:true}` / `#pulse {ERR,Error,hasLink:true}`); the child getters present under stable ids (`led#default__led {ON,#22c55e}`, `label#default__label {Connected}`, `link#withlink__link {Status page, external:true}`); link-iff-href (`#default` root holds only `[SPAN.led, LABEL.label]`, no `link#default__link`); root `DIV.status-indicator`; `.status-indicator` + `.status-indicator > .link` in cascade; **combined accent proof** — the link colour swaps gold(`rgb(194,136,64)`)↔blue(`rgb(58,122,176)`) while the led background `rgb(34,197,94)` is identical across shells; screenshot eye-checked; 0 orphans.

*UI-implementation record. No protocol/data implication. The first di composite; founds the composite build pattern (`<div class="type">` root + aggregate-getter / children-self-register / matrix-multiplies accounting). Applies D-096, no amendment. The composite-registration model is the reusable precedent for `password-field`/`color-picker`/`file-field`/`combobox`/`tag-select`/`star-rating` (D-069 promotion-watch at the second composite). Next: the text-processor engine → dd-components.*

### N-055 — Focus is a transient state signal, not brand identity: editable cells lift a neutral border, affordances wear the accent ring

Post-J-434 skin tune (pushed ahead of this record; documented here after the fact). The `:focus-visible` treatment was split off from the brand accent on a **function litmus**.

**Editable cells** — surfaces that *hold* a value the user types or picks (`textfield`, `textarea`, `number`, `date`, `color`, `select`, `select-multiple`) — take a single neutral `border-color: var(--t3)` on `:focus-visible`: one border swap, no shadow ring. The **four validating cells** (`textfield`, `number`, `date`, `select`) escalate that border to `var(--err-bright)` on `:invalid:focus-visible` (`color` + `select-multiple` are always-valued, so they carry no `:invalid` rule).

**Affordances** — controls that *act* rather than hold a value (`button`, `toggle`, `range`, `file`, `link`) — keep the accent-tinted `--focus-ring` box-shadow (`0 0 0 2px var(--s2), 0 0 0 4px var(--accent2, var(--t3))`), so they still flash gold/blue on focus.

**Rationale.** Focus marks *where you are* — a transient interaction state. The accent marks *whose app this is* — a persistent brand fact. Painting every focused field with the accent conflated the two and made a focused text box shout brand. A neutral border-lift reads as "active here"; the accent stays on the things you press. The validating-cell red is the one place focus *should* carry semantic colour — there it is a **state** signal (invalid), not brand.

**Litmus for the next author.** A new editable cell focuses with `--t3` (and `--err-bright` if it validates); a new affordance focuses with `--focus-ring`. Do **not** re-introduce accent focus on a field. `--t3 #8a8880` is a settled neutral-ramp token (with `--t4`/`--t2`/`--t`); only `--err-bright #e64343` is still HMR-tuning (skin.css line 29).

*UI-implementation record. Skin-only — no component or protocol change. PROVISIONAL only on the `--err-bright` value; the principle and the `--t3` choice are settled. Arc-local (D-069 promotion-watch only if the focus-vs-brand split needs to bind beyond skin).*

### N-056 — text-processor engine (M-RP4.0): the forwarded-attachment edit seam; the four-kind taxonomy on two engines; kind 1 transformer built, kinds 2/3/4 codified (→ D-099)

M-RP4.0 closed (J-435). Discharges the longest-standing reserved UI seam (N-029 → N-032 EDIT-vs-RENDER → N-038 → N-040 textarea reserve), exactly as D-065 intended: built when a consumer is in hand, codified so growth is bounded. The text-processor is **not one engine** — the design walk resolved it into a **four-kind taxonomy on two engines**, and set the honest scope **build kind 1, codify all four**.

**The four-kind taxonomy (the codified architecture — canonical in D-099).**

| # | kind | signature | model `T` | engine / side | built? | first consumer |
|---|---|---|---|---|---|---|
| 1 | transformer | `string → string` (live, on `input`) | none | **edit** (the M-RP4.0 attachment) | **BUILT** | `textarea` (`arrowMorph`/`emojiMorph`) |
| 2 | converter | `string ↔ T` (`toString`/`fromString`) | number · Date · phone | **both** (the bridge: `fromString` edit, `toString` render) | reserved | number / date / phone field |
| 3 | filter / guard | `T → T` (idempotent) | the field's own `T` | side-agnostic | reserved (M-RP4.1) | `number` min/max clamp |
| 4 | renderer | `string → safeHTML` | none | **render** (the deferred `use:render`) | reserved | `paragraph` inline marks |

Two facts travel with the table: **(a) kind ⟂ engine** — four kinds, two engines (edit attachment / render `use:render`), and **kind 2 is the bridge** (its two methods sit on opposite sides; native `type=number` can't show a `toString`, so kind 2 needs a *decoupled* text field, `toString` may delegate to `Intl`); **(b) scope** — codify four, build one (D-065).

**The edit seam is a forwarded *attachment*, not a `use:` action (P-1a).** This is the first time the library forwards behaviour from a consumer onto an atomic's internal element without the atomic carrying the logic. A `use:` action only attaches to elements in the component that writes them — a consumer cannot forward one onto an atomic's inner `<textarea>`. So the engine ships as a Svelte 5 **attachment**: `processor(rules)` returns a `createAttachmentKey()`-keyed prop; the atomic spreads `{...rest}` onto its root, so `<Textarea {...processor(x)} />` lands the attachment on the inner element. The atomic carries **no** processing logic — only the generic spread (ready, not containing — D-065). Reactivity = the attachment lifecycle (new `rules` → new object → Svelte cleans up + re-attaches). This resolves the old N-029/N-040 "a consumer simply layers it on" framing, which had assumed `use:` — superseded here.

**Three files (`ui/common/lib/components/processor/`).** `transform.ts` — the pure, DOM-free, framework-free core (the `logic.ts` posture): `TransformRule {find, replace, reversible?}` + `TransformConfig` + `applyRules` (sequential, literal replace-all via split/join, regex-free, total) + `assertSafeRules` + `ProcessorRuleError`. `configs.ts` — named Tier-1 configs `arrowMorph` (`-->→→`, `<--→←`, `=>→⇒`) / `emojiMorph`. `processor.ts` — the **one** framework touch (`svelte/attachments`): the attachment, the caret-preserving value sink, the re-entrancy guard, and a DEV `window.__XGEN_PROC__` pure-core hook (mirrors the envelope `import.meta.env.DEV` idiom, DCE'd in prod).

**Two provenance tiers gate safety (P-3).** Tier-1 (trusted `common` code): full power, gate bypassed. Tier-2 (user/settings data): **serializable literal `{find, replace}` pairs only** — caps (count, length) + a **convergence lint** (reject a pair whose `replace` re-contains its `find`, because the engine re-runs the whole value each keystroke, so `a`→`aa` would loop). Untrusted regex is not representable (literal strings only); a regex rule-kind + its ReDoS guard are reserved for an explicit advanced opt-in. The literal-only subset is what makes runtime-editable rules safe. Settings-backed Tier-2 rules persist as a section of the app's existing global settings file (reserved, **no bespoke rules file**); the engine stays source-agnostic.

**Caret-preserving value sink (P-4) — the build's hard bit.** On `input`, recompute; if changed, set `node.value`, restore the caret to the **transformed-prefix length** (`applyRules(before.slice(0, caret)).length` = old caret + net length-delta of replacements before it), then dispatch a re-entrancy-guarded synthetic `input` so Svelte's `bind:value` syncs. Holds for the dominant case (the user completes a token *at* the caret, so it sits wholly in the prefix); a token straddling the caret is the documented limitation. **CDP cannot drive `:focus`+caret**, so caret behaviour is the one item verified by eyeball/screenshot, not asserted — recorded honestly (D-065).

**Forward-clean naming.** Kind 1's type is `TransformRule`; the future union `ProcessorRule = TransformRule | ConvertRule | ClampRule | RenderRule` is documented in D-099 but **only `TransformRule` exists in code**, so the namespace stays clean when kinds 2/3/4 land. `TransformRule.reversible` is **declared, not implemented** (reserved — no un-morph path built).

**Verify (Chat self-drove, sampler + CDP 9422, fresh launch; real output, Rule 2).** `ids().length===56` (55→56; the attachment adds no registry entry — the host `textarea#processed` is the one id). Transform + binding-sync: set `"a --> b => c"`, dispatch `input`, tick, read — DOM `"a → b ⇒ c"` AND registry `{value:"a → b ⇒ c"}` (the synthetic input synced `bind:value`, not just the DOM). Pure core via the DEV hook: `applyRules("-->", …)==="→"`, sequential `"a --> b => c <-- d"`→`"a → b ⇒ c ← d"`, no-op `"plain text"` unchanged. Provenance guard: `assertSafeRules([{find:"x",replace:"xx"}],{trusted:false})` **throws** `ProcessorRuleError` (convergence), `{trusted:true}` **passes**, `arrowMorph {trusted:false}` **passes** (convergent literals). No-op safety: a token-free value round-trips with exactly **1** input event (no spurious synthetic dispatch). Screenshot: `#processed` renders `1 → 2 ← 3 ⇒ 4`. **Negative control (recorded honestly):** the sibling `textarea#default` — no processor attached — held externally-typed `:) -->` **un-morphed**, proving the attachment is scoped to the one cell and does not leak onto sibling textareas. Teardown `0 orphans`.

*UI-implementation record. No protocol/data implication. Founds the forwarded-attachment edit seam + the two-engines/four-kinds mental model + the two-tier provenance subset. Engine is `common` infra (no catalogue row); `textarea` is the first processor-host. Crystallises into **D-099** (canonical taxonomy). Next: M-RP4.1 (kind-3 number-clamp, on `change`) → kind-2 converter field (decoupled text field; `Intl`) → kind-4 `use:render` (deferred) → dd-components.*

### N-057 — user-owned substitution pairs (M-RP4.2): the source-agnostic rule store; the ` | ` + first-space grammar; presets retired as the live source (→ D-100)

M-RP4.2 closed (J-441). Executes decision 9 of the M-RP4.0 runbook: the kind-1 transformer built at M-RP4.0 had hardcoded named configs (`arrowMorph`/`emojiMorph`); now the rules come from **one user-owned string**, and a store decouples *where the rules come from* from *who consumes them*. `configs.ts` is deleted — it was sample data, never architecture (D-099/N-056). The engine does NOT change — this arc adds the *source* (TOML) and the *plumbing* (parser + reactive store + frontend delivery).

**The grammar (locked, literal, no regex).** The whole list is one string; pairs separated by the literal ` | ` (space-pipe-space); within a pair, split on the **first space** → `find` = before, `replace` = everything after. `find` = no whitespace; `replace` = any string (multi-char, emoji, internal spaces, a lone `|`). The only forbidden token substring is ` | ` itself; blank pairs skipped. The simplest entry that survives the data (`-->`, `<--`, `:)`, `|`) without regex — the literal engine stays literal.

**Source-agnostic store (`$common`).** `parseRules(text) → TransformConfig` (pure, beside `applyRules`) + a reactive `substitutions` store: `setRules(text)` parses, runs `assertSafeRules({trusted:false})` (Tier-2 — config data is user data, so the caps + convergence lint protect against a self-authored loop), and on rejection fails safe (empty + DEV warn). Hosts read `substitutions.rules` and pass it to `processor(...)`; a new string re-runs the attachment lifecycle. The store takes a string from anywhere — the engine stays source-agnostic (D-099 P-3).

**The source duality (Chat/Clair split; this arc crosses the boundary).** Chat owns the `$common` parser + store + sampler rewire (CDP-verifiable, J-436). Clair owns the Rust config struct + Tauri command + client boot hydration (client-only, J-437). Two sources feed the same store: the **real client** via `get_substitutions` (Rust `load_substitutions_section` reads `xgen-client_config.toml [substitutions] rules` → command → store on `onMount`); the **sampler** seeds a literal (D-097: a minimal host, no client config to read).

**Two hand-synced seeds (a documented seam).** The first-run starter pack (J-438, Joe-locked, six pairs `--> → | <-- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒`) lives in two places kept in sync by hand: Rust `DEFAULT_SUBSTITUTIONS_SEED` (`app.rs`, seeded once at config-birth in `cmd_init`, never resurrected after the user clears it — J-438/J-439) and the TS literal in `app_sampler.svelte` (the sampler mirrors the client pack — J-440). Wiring the sampler host to read a shared config is deprecated-UI plumbing set aside for now; the seam is closed properly by **M-RP4.4** (the sampler real config-load arc).

**Tier-2 richer UX deferred.** The per-pair partition + inline warnings (a bad pair drops, the rest survive, with feedback) is M-RP4.3 — the in-app editor needs it; the file-edit read path fails safe whole (D-065).

*UI-implementation record. No protocol/data implication. Founds the source-agnostic rule store + the ` | `/first-space grammar; retires `configs.ts` presets as the live source. Crystallises into **D-100**. Next: M-RP4.4 (sampler real config-load path — closes the two-hand-synced-seeds seam) → M-RP4.3 (editor + write-back) → M-RP4.1 (kind-3 clamp).*

### N-058 — sampler real config-load path (M-RP4.4): the sampler loads substitutions through the real generate→file→load→command chain (not a frontend literal); clean-slate-on-start; the N-057 frontend-literal seam closed

M-RP4.4 closed (J-444). Executes the runbook + **D-101**. The sampler stops seeding a frontend literal and instead runs the **real config-load chain** — the same shape the client uses — so a config-backed component drops into the rewritten client/node UIs with zero reprogramming.

**The chain (contract-shape parity, NOT code reuse — D-098).** On start the sampler host writes a **subset** config (`[substitutions] rules` only, from a seed const) to `xgen-sampler_config.toml` in exe_dir, exposes a `get_substitutions` Tauri command, and the frontend hydrates the `substitutions` store from it on `onMount` (mirroring `app_client.svelte` J-437: `invoke('get_substitutions') → setRules`). The host reimplements a minimal read/write of its config (Clair, J-443) — it can't depend on `xgen-client` (D-098); the stable contract is the component interface (`setRules(string)`), not the plumbing.

**Clean-slate-on-start (D-101, phase-scoped).** Every binary (client, node, sampler host) wipes any found config at launch, regenerates from seed, then reads — config is ephemeral this phase; this suspends J-438 seed-once for the phase (see D-101).

**What this closes, what stays open (the seed seam).** N-057 flagged two hand-synced seeds: the Rust `DEFAULT_SUBSTITUTIONS_SEED` + the **TS literal** in `app_sampler.svelte`. M-RP4.4 **removes the frontend literal** — the sampler loads, it no longer seeds. But the sampler host now carries its **own** Rust seed const (J-443), hand-synced with the client's, so the seam **moves** (frontend-literal → two Rust consts) rather than fully closing. A shared-const crate collapsing the two Rust consts is explicitly **out of scope** (a documented third-copy seam, deferred until justified).

**Async hydration reactivity (verified).** The seed now lands *after* mount (`await invoke`), so the `textarea#processed` cell starts with empty rules and re-attaches when `setRules` resolves — the source-agnostic store + attachment lifecycle (D-099/J-436) carries the post-mount update. CDP-confirmed live.

**Frontend dep.** `app_sampler.svelte` now calls `invoke`, so `@tauri-apps/api: ^2` was added to `ui/sampler/package.json` (matching the client). The Rust host stays minimal (D-098 unaffected).

**Verify (Chat self-drove, sampler + CDP 9422, two fresh launches; real output, Rule 2).** Launch 1 (config absent): the host generated `xgen-sampler_config.toml` in exe_dir with the seed (subset — `[substitutions]` only); `ids().length===56`; live morph from the loaded rules — input `x --> y :) z -- w <3 q <-- r :(` → `x → y 🙂 z ‒ w ❤️ q ← r 🙁` (all six seed pairs), and the registry `textarea#processed` `{value}` carried the morphed string (bind:value synced, not just the DOM). Launch 2 (delete-on-start): pre-seeded a **sentinel** config (`zzz … | qqq …`), relaunched → the file was **wiped + regenerated to the seed**, and the live store reflected the seed not the sentinel — input `zzz --> qqq :) end` → `zzz → qqq 🙂 end` (seed pairs morph, sentinel tokens stay literal). Teardown 0 orphans both times.

*UI-implementation record. Closes the M-RP4.2 frontend-literal seam (N-057); the real config-load path is now the sampler standard for config-backed components. The seed const is still hand-synced across client + sampler-host Rust (shared-const crate out of scope). Next: M-RP4.3 (in-app TOML editor + write-back) → M-RP4.1 (kind-3 number-clamp).*

## 2026-07-02

### N-059 — the `widget` tier: a Level-2 app assembly above the di/dd × atomic/composite grid (concept + name + home Joe-locked; full definition deferred until di-composites are built)

Design discussion (J-445), no code. M-RP4.3 (in-app `[substitutions]` TOML editor) is the first UI unit that is **assembly + behaviour + host I/O**, and the component taxonomy stops at di/dd × atomic/composite — all passive (N-054). There is no tier for a behaviour-carrying assembly. This note locks the *concept* + name + home for a new tier; the **full definition (constraint set + verify home) is deferred** until the di-composite backlog is built, so the boundary is drawn against real composites rather than one specimen.

**The discriminator (passive vs active).** A composite (`status-indicator`, N-054) is **passive**: props in → DOM out + an inspection getter, no side effects, interprets no domain structure. A **widget** is **active**: owns its own state + lifecycle (dirty-tracking, load/save), decides (parse/validate), and may perform host I/O. Litmus: *does it only arrange values it's handed (composite), or own state and act (widget)?* Removing a composite loses a layout; removing a widget loses a behaviour.

**Placement — a new *level*, not a new grid cell.** Level 0 substrate (`common/base`) → Level 1 components (`core`; di/dd × atomic/composite; ceiling = `status-indicator`) → **Level 2 widget** (assembled *from* Level 1). The widget sits a storey **above** the arity axis — atomic → composite → **widget**, not a third rung wedged between atomic and composite. This keeps di/dd/atomic/composite **pure**, the concept-purity we set out to protect.

**Name = `widget`.** "ui-module" in the generic/CS sense; named `widget` to avoid collision with the project's protocol/CLI **modules** + the Tier-1 auth module work. (Honest note per D-065: the term `widget` is **new to the record** as of this session — not previously written in any canonical file; locked here, not recalled.)

**One tier, not two — I/O is Phase, not a class branch.** Behaviour-only vs I/O-carrying is **not** a second tier; it maps onto the existing **Phase** axis (A pure Svelte / B +Tauri / C all three, N-028). A behaviour-only widget is Phase-A; an I/O widget is Phase-B. A single specimen doesn't justify splitting a taxonomy branch (D-069 four-recurrence bar).

**Home = `ui/common` (Joe-lock).** Some widgets will be used in the node app, so the tier lives in the shared substrate mirror, not `ui/client`. (Per-widget sharing scope may still vary; the *tier's* home is `common`.)

**Verify home — provisional, to be locked at full definition.** A widget's defining trait (host I/O + integration) is the sampler's declared blind spot — D-097 cedes integration + host-real behaviour to the real shells. Leaning: the widget's **effectful layer** verifies in the real shell; its **pure/presentational layer** (composed components + skin + validation, I/O stubbed) stays sampler-tunable via HMR + a DEV hook — the N-056 processor precedent (pure core via `__XGEN_PROC__`, caret behaviour eyeballed in the real focused window). The widget self-registers one aggregate getter the composite way (N-054).

**Constraint sketch (input to the full definition).** Composes down only (core + substrate, never reaches to raw native tags for its logic); owns state + lifecycle; host I/O through defined seams only (Tauri commands + `$common` stores, no ad-hoc file access inside the widget); self-registers one aggregate getter; clean mount/unmount (a droppable unit, no cross-widget coupling); skin = L2 only, pure layer separable from effect layer; scoped home + a Phase; surfaces honest phase-limits (e.g. session-only write-back under D-101, D-065).

**Roadmap consequence.** M-RP4.3 is the first widget, so it now waits on the widget definition. Reordered: finish the di-composite backlog as **passive** (`password-field` / `color-picker` / `file-field` / `combobox` / `tag-select` / `star-rating`, N-054) → **widget definition** (this note promoted to a spec) → **M-RP4.3** (first widget instance) → M-RP4.1 (kind-3 clamp); the kind-2 converter field + kind-4 `use:render` slot around as before.

*UI-design note. No protocol/data implication, no component change. Concept + name + placement + home + one-tier-+-Phase Joe-locked (J-445); full definition (constraint set + verify home) deferred until the di-composite backlog is built. Next: a di-composite from the N-054 backlog (Joe's selection).*

### N-060 — `password-field` (M-RP2.23): the 2nd di composite; redact/reveal/caps mechanics + the transparent-icon + no-reflow lessons

The **second di composite** (after `status-indicator`, N-054), 18th `core` component. Root `<div class="password-field">` composes `textfield` (`__field`) + a `button` toggle-mode reveal (`__reveal`); owns `revealed` + `capsLock`; getter `{revealed, hasValue, capsLock}` — boolean `hasValue`, never the value. The N-054 registration model held clean (composite root + children self-registering under `<childtype>#<id>__<slot>`), so the matrix multiplies a flat **+9** (56→65). **D-069 2nd-composite watch: no promotion** — N-054 stays a note.

**Three mechanics.**
- **Secret safety (`redactValue`, Step A additive on `textfield`):** the child field is passed `redactValue`, so its own getter reports `{type, value: null}` — the live secret never reaches `window.__XGEN_DEBUG__`. Composite reports only boolean `hasValue`. CDP-proven: `textfield#default__field` = `{type:"password", value:null}` while composite `hasValue:true`.
- **Reveal = two-stage icon toggle:** `button mode="toggle"`, `bind:pressed={revealed}`; child `type = revealed ? 'text' : 'password'`. The existing toggle-mode button + reflected `aria-pressed` **is** the two-picture toggle — no new component. Glyph = eye / eye-off via scoped `--eye`/`--eye-off` currentColor `mask-image`, swapped on `aria-pressed` (SVG placeholder until the `icon` primitive, N-052).
- **Caps-lock (composite-level, no textfield touch):** keyboard events bubble from the inner `<input>` to the wrapper `<div>`; `onkeyup`/`onkeydown` read `getModifierState('CapsLock')`. Surfaced as `data-caps` on the wrapper.

**Cosmetic evolution (Joe's revision round — the lessons).**
- **Caps hint is skin, not a layout child.** First cut used an optional `label __capswarn` child; it reflowed the block and varied the matrix. Replaced by `data-caps` → (1) red `--err-bright` field border + (2) an overlaid `::after` "Caps Lock is on!" hint, absolutely positioned on the `position:relative` wrapper so it **never** affects footprint. Dropping the child also flattens the matrix (no conditional entry). *Lesson: state feedback belongs in the skin via a reflected data-attr, not an injected element — a layout child is a reflow + a matrix wobble.*
- **Transparent icon-only reveal.** De-chromed `.password-field > .button` (no bg/border/padding/shadow), icon-only, 18px, greys on `:disabled`, keeps the inherited focus ring. Text labels ("Show"/"Hide") were the first cut — replaced by the icon mask so width doesn't track label length.
- **The width-jump.** Toggling password↔text still jumped the field 16px after the icon work. Root cause was **not** `::-ms-reveal` (Edge/WebView2 native control, killed as belt-and-suspenders) but the `textfield`'s reserved **`padding-right:24px`** in password mode (N-039, space for the `***` inset). Suppressing the `***` glyph (`background-image:none`) left the padding; normalizing it to `--sp-2` gave identical width both states (CDP 155/155, jump 0). *Lesson: when suppressing a per-type inset icon, drop its reserved padding too — the icon and its space are two separate rules.* Gap 5px→3px.

**Verify.** Sampler DI·composite panel, 3 cells (default/disabled/revealed); CDP both accents. All proofs real (Rule 2): matrix 56→65 flat, redact null-while-hasValue, reveal type-flip + `aria-pressed` + eye→eye-off mask, caps → `data-caps` + red border + `::after` hint, transparent button geometry, no `***`, no width jump.

**Logged / deferred.** Confirm-password match → future `password-confirm` composite (equality-check leans dd). Strength meter → future dd. Real eye `icon` primitive (N-052) supersedes the mask placeholder when it lands.

*UI-design note. Component + skin change (shipped), no protocol/data implication. → components registry (18th `core`, 2nd di composite). Next: a di-composite from the N-054 backlog (Joe's selection).*

## 2026-07-03

### N-061 — `star-rating` (M-RP2.24): the 3rd di composite; SHAPE B (self-contained, internal stars — not composing child atomics) refines the composite definition; discrete-value + roving-radiogroup + hover-preview

M-RP2.24 closed (J-447). The **third di composite** (after `status-indicator` N-054, `password-field` N-060), 19th `core` component, and the **third di-composite backlog pick** (N-054 list). Its headline is a taxonomy refinement.

**Shape B — a composite that composes NOTHING (the definition refinement).** The first two composites compose real child atomic *components* (led+label+link; textfield+button), which self-register and multiply the matrix. `star-rating` is a `<div class="star-rating">` (the N-020/N-022 composite root-marker via `envelope`) that renders its stars **internally** in an `{#each}` of plain `<span role="radio">` — it composes no child components. So it registers **one** aggregate getter and the matrix multiplies **flat +1 per cell** (3 cells → +3, **65→68**), not the child-multiply of the earlier composites. This **refines the composite definition**: *a di-composite is a `<div class="type">` assembly; composing child atomic components (status/password) is the common case, not a requirement.* → **D-069 promotion-watch** (definition refinement — note only unless it recurs; a fourth composite that also composes-nothing would promote it).

**di + PASSIVE (the backlog bar).** The caller supplies `max`/`value`; the component interprets no domain structure (di). Hover-preview is transient presentational `$state` (the `button :active` precedent), not load/save/validate/host-I/O — so it clears the **widget** bar (N-059) and stays a passive composite, as the backlog mandates. (star-rating was the deliberate FIRST pick precisely because it's the one unambiguously-passive candidate; the stateful trio — combobox/tag-select/color-picker — come last, where they pressure-test the passive/active line before the widget spec.)

**Mechanics.** Value = `number`, `$bindable`, default 0 (= unrated), numeric bind-out; `max` default 5; getter `{value, max}` (Shape B — no children to aggregate). **a11y**: root `role="radiogroup"`; each star `role="radio"` + `aria-checked`; a **roving tabindex** (the checked star, or star 1 when unrated, is the sole tab stop); arrows move+select (selection-follows-focus radiogroup model), Home=1/End=max, with `stars[next-1].focus()` moving focus with the value. **hover-preview + clearable**: `hovered` transient preview (fill target = `hovered || value`, restores on root `mouseleave`); `clearable` default true — clicking the active star zeroes it. `readonly`/`disabled` drop interaction (no tabindex/handlers); readonly stays full-colour (it *displays* a rating), only disabled dims.

**Glyph = mask placeholder (N-052 lineage).** ★ via a currentColor `mask-image` (skin-scoped `--star` SVG data-URI), filled = `--accent2` (re-themes gold/blue per shell), empty = `--t4` — the password-field eye pattern reused; the real `icon` primitive (N-052) supersedes it when it lands. Whole-star only v1; a half-star readonly average is a future shape.

**Verify (Chat self-drove, sampler + CDP 9422, both accents, real output — Rule 2).** `ids().length===68`; `#default {value:0,max:5}` / `#rated {value:3,max:5}` / `#readonly {value:4,max:5}`. Click star 4 → `4`; click again → `0` (clearable). Hover (split read — the Svelte-5 flush finding N-053: `data-filled` re-renders next tick, so preview fill is read in a *separate* eval): `filled:5` while `value:0`, `mouseleave` → `filled:0` (restore, value untouched). Keyboard `#rated`: `3→Right→4→Left×2→2→Home→1→End→5`. a11y: `role=radiogroup` / star `role=radio` / `checkedIdx=2` (aria-checked on the value star). readonly: `tabindex=null`, click no-op (stays 4), `data-readonly="true"`, `aria-disabled=null` (readonly ≠ disabled). Colour: filled gold `rgb(194,136,64)` ↔ blue `rgb(58,122,176)`, empty `--t4 rgb(88,92,100)`; 4 `.star-rating` rules in cascade. Screenshots both accents eye-checked (default empty / rated 3 / readonly 4). Teardown 0 orphans.

**Skin note.** Joe cosmetic-tuned the shipped skin post-build: star `18px` (from 20px), `gap: var(--sp-0)` (from `--sp-1`). `--sp-0` is now an explicit `:root` scale step (`0`) — the **0-gap is intentional**: the star SVG carries its own whitespace, so stars sit adjacent (no dead click-zones) while reading correctly spaced.

**Correction owned (Rule 5).** The runbook DoD first wrote the matrix as `65→66` (a `+1`-total slip); Shape B adds 1 entry × 3 cells = **+3**, so `65→68` is the correct, verified count. Runbook fixed at close.

*UI-design note. Component + skin change (shipped), no protocol/data implication. → components registry (19th `core`, 3rd di composite). Refines the composite definition (Shape-B composes-nothing) — D-069 promotion-watch. Next: a di-composite from the N-054 backlog — `file-field` next by the passive-purity order (Joe's pick).*

## 2026-07-03

### N-062 — `file-field` (M-RP2.25): the 4th di composite; SHAPE A (child-composite — hidden `file` atomic + drop-zone + list); passive slice only (no remove, no progress); outline drop-icon

M-RP2.25 closed (J-448). The **4th di composite** (after status-indicator, password-field, star-rating), **20th `core`**, 4th backlog pick (N-054). Re-exercises the child-composite model — contrast to star-rating's Shape B.

**Shape A + the scope call (Rule 6).** `file-field` composes the real `file` atomic as a **hidden** child input (`__input`, self-registers) driven by a styled drop-zone + a file-list. The deferred spec was "zone + list + remove + progress"; the **passive slice** shipped is zone + list only. **No remove** (a `FileList` is immutable — remove needs a `File[]` model + `DataTransfer` write-back, tag-select territory; logged follow-up). **No progress/upload** (host I/O = widget-tier, N-059, deferred). So it stays FileList-native + passive: drop/pick **replaces** the selection. Matrix multiplies **+2/cell** (composite + child) → 3 cells → **68→74**.

**Mechanics.** `files` `$bindable` (FileList|null); the zone (`role=button`, `tabindex`, Enter/Space → picker) drives the hidden input via a queried ref (`root.querySelector('input[type=file]')` — no atomic change). Drop builds a `DataTransfer` (respects `multiple`; keeps first when single), sets `input.files`, dispatches `change` so the child `bind:files` syncs up. `data-dragging` reflects for the highlight; `disabled` drops interaction. Getter `{count, files:[{name,size,type}]}`.

**Drop-icon (approved in-milestone touch-up).** Outline (stroked, `fill=none`) folder + short down-arrow centered in the folder rect — **info-only, no accent**. Skin-only: `--drop` mask var on `.file-field`, `::before` on `.drop-zone` left of the label, fixed `--t3` (stays neutral even while the zone border/text go accent on drag). Same mask mechanism as the eye/star glyphs (N-052 lineage); the real `icon` primitive supersedes it later.

**Verify (Chat self-drove, sampler + CDP 9422, both accents, real — Rule 2).** `vite build` 140 modules (caught + fixed a name clash: `file.svelte` already aliased `FileField` → sampler import renamed `FileFieldComposite`). `ids()===74`; baseline `{count:0,files:[]}` on composite + child. Drop → `{count:1,files:[{name:"a.txt",size:1,type:"text/plain"}]}`; `!multiple` drop of 2 keeps 1; `#multiple` keeps 2. Enter triggers hidden input (click spy). `dragover` → `data-dragging="true"`, border `--accent2`. Disabled: `tabindex=-1`, `aria-disabled=true`, drop no-op. `::before` 18×18, mask set, bg `--t3 rgb(138,136,128)`. Accent gold`rgb(194,136,64)`↔blue`rgb(58,122,176)`; 4 `.file-field` rules in cascade. Screenshots eye-checked (icon left of label). 0 orphans.

*UI-design note. Component + skin (shipped). → registry (20th `core`, 4th di composite). Re-confirms the child-composite matrix model (+2/cell) alongside star-rating's Shape-B (+1/cell). Next: `combobox` (N-054 backlog, passive-purity order).*

---

## 2026-07-03

### N-063 — `combobox`: native reverted, rebuilt as rich owned-popup

Started native (Path A, `<input list>` + `<datalist>`, passive). Reverted mid-arc: native datalist rows are **text-only and unstyleable** (OS/WebView-drawn) — no rich rows (icon/status), no compact/left-aligned list. Real usage needs rich rows, so native was dropped entirely (not kept as a baseline).

Rebuilt as **owned-popup**: own `<ul role="listbox">`, so everything is styleable — compact, left-aligned, no balloon. Still a **passive di composite**: owns exactly one UI flag, `open` (same order as password-field `revealed` / file-field `data-dragging`), no behaviour contract → **not** a `widget` (N-059). Settled that the widget bar is *behaviour contract*, not *any state* — an earlier over-rigid "owns open → widget" was corrected.

`options` = `{value,label,status?,disabled?,icon?}[]` (back-compat `string[]`); `icon?` declared but **unwired** until an icon primitive exists. Child textfield registers as `<id>__input` (suffix deliberately ≠ password-field's `__field`, so two textfield-bearing composites don't collide on a shared instance id — a real collision caught at CDP: 74→78 instead of 80). ▼ swaps chevron→closed-triangle on `[data-open]` (real `open` makes the swap honest). ▼ is a real `.chev` span (finger cursor scoped to the glyph; click focuses the field/opens).

**Verify (Chat self-drove, CDP 9422, both accents, real — Rule 2).** `ids()===80`; children `textfield#*__input` (no collision). Open-on-focus sets `data-open` + mounts `<ul>`; filter narrows (`"on"`→Online); select sets value + closes + ▼→chevron; disabled composite inert; disabled row (Offline) unselectable; `.chev` cursor `pointer`, click focuses+opens. 0 orphans.

*UI-design note. Component + skin (shipped). → registry (21st `core`, 5th di composite, matrix 74→80). Establishes the reusable **owned-popup pattern** → color-picker will reuse it (native color popup also unstyleable, N-047). Next: `tag-select` (N-054 backlog).*

---

## 2026-07-03

### N-064 — `chip` (M-RP2.27): standalone di token; self-computed colour; the used-internally-without-registration pattern

M-RP2.27 closed (J-450). **22nd `core`**, a standalone di token (atomic-ish `<span class="chip">`, no self-registering child components — the `×` is a raw `<button>`). Built standalone (own registry row + sampler cells) rather than internal-only because it recurs downstream (dd facets, tier/`is_ai` badges, entity tokens) and the registry already reserved the name. **Prerequisite for `tag-select`** (M-RP2.28).

**Self-computed colour (the headline).** `led` (N-034) was caller-supplied; every other di is accent-derived. `chip` is the **first di whose colour is computed from its own content**: `hash(label)` → hue, at a fixed muted S/L band (fill `hsl(h 45% 82%)`, text `hsl(h 55% 30%)`, border `hsl(h 40% 80%)` — never bright/white), injected as inline vars (`--chip-bg/fg/bd`) the `.chip` skin reads (the `--led-colour` mechanism). Deterministic + **shell-independent** (CDP-proven: identical fill under gold and blue).

**The N-064 pattern — standalone component used internally without registration.** `tag-select` will render chip instances via `{#each}` **without** per-instance `envelope` registration (chips are dynamic/data-driven, not fixed structural children). So a component can be *built standalone* (self-registers in its own sampler cells) yet *used internally without self-registration* when instances are dynamic — keeping the consumer's matrix contribution predictable (tag-select stays +2/cell: composite + textfield, chips don't multiply).

**Contract.** `label` = raw stored value (uppercase is display-only, skin `text-transform`; **bold** `font-weight:700`); `removable?` default true (`×` on the right, `×`-only remove, chip body stays inert-selectable); `onRemove?`; getter `{label, removable}`. Long labels ellipsis-truncate (max-width). `×` = masked stroke glyph (`--chip-x`, N-052 lineage).

**Verify (Chat self-drove, sampler + CDP 9422, both accents, real — Rule 2).** `vite build` 142 modules. `ids()===83`; `#default {label:"rust",removable:true}` / `#static {removable:false}` (no `×`) / `#long` (ellipsis). Computed fills differ per label — rust `rgb(244,225,242)` ≠ svelte `rgb(225,233,244)`, both muted; rust fill **identical** under node shell (self-computed, not accent-derived). `×` present/default, absent/static; `×` click fires bound `onRemove` (local spy, internal counter — not surfaced to registry), no throw. Screenshot eye-checked (bold caps, per-label tints). Joe cosmetic: fill L 92→82 (−10%), label bold.

*UI-design note. Component + skin (shipped). → registry (22nd `core`, matrix 80→83). First self-computed-colour di; establishes the N-064 used-internally-without-registration pattern → `tag-select`. Next: `tag-select` (M-RP2.28, the chip consumer).*

**AMENDMENT (M-RP2.28, 2026-07-04):** "used internally without registration" is NOT automatic — `envelope` registers whenever a debug getter is present (id-less instances just get ordinal ids, `chip#1..4`, caught at CDP when tag-select first consumed chip). Made real by an additive `register` prop on `chip` (default true; `register={false}` omits the getter → renders + stamps `.chip` but does not register). tag-select renders chips `register={false}` → matrix stays +2/cell (composite + `__filter`), chips don't multiply. → N-065.

---

## 2026-07-04

### N-065 — `tag-select` (M-RP2.28): the 6th di composite, THE CHIP CONSUMER; multi-select `string[]`; owned-popup reuse; a general width system

M-RP2.28 closed (J-451). **23rd `core`**, **6th di composite**, last of the N-054 backlog. A **completely new component** (own file/logic) that *reuses the owned-popup pattern* (N-063) and *composes* two existing components as children — `Textfield` (the `__filter` query buffer) + `Chip` (the tags, `register={false}`). Not built on combobox (no shared code).

**Model + registration.** `value: string[]` `$bindable`, empty `[]`; getter `{values,count}` (select-multiple precedent). The query is LOCAL `$state` on the child textfield (`__filter` — 3rd distinct textfield suffix after `__field`/`__input`, collision-safe), cleared on pick, NOT the model. Matrix **+2/cell** (composite + `__filter`; chips don't register via N-064) → **83→89** (3 cells: default/max/create).

**Owned popup, two sections.** Own `<ul role=listbox aria-multiselectable>`: top **"Selected (N)"** (all picked, reachable even when the row collapses) + main **"Options"** (`notSelected && matchesQuery` — hide-selected). Pick STAYS OPEN, clears query, refocuses. `allowCreate?` (default false) → Enter on non-empty query, no exact match → create `{value:q,label:q}`. Dedup case-insensitive + silent. `max?` → picks no-op + `[data-full]` dim + input disabled + no open. Backspace on empty query pops the last tag.

**Structure (password-field layout).** Root `.tag-select` = flex ROW = `.tag-field` (the bordered box; anchors the popup; holds the chip row + growing borderless `__filter`) + an **OUTSIDE** manage gear (`.tag-manage`, transparent icon-only, outline cog mask, N-052 lineage) — beside the field, not inside. Gear fires `onManage?` (opt-in); the actual keyword-set editor = widget-tier (N-059), deferred.

**Width system (Joe-locked).** No `width` → `.tag-field` sizes to content (`max-content`), cap at `DEFAULT_CAP=3`. `width` set → fixed field, a **hidden mirror row** gives natural chip widths + a `ResizeObserver` tracks the field, a derived count shows only chips that FULLY fit, the rest → `+N` pill (no half-clipped chips). Deterministic at fixed width (CDP-stable).

**Candidate collection (design note, deferred).** `options` is source-agnostic (N-057). The persistent vocabulary lives in the client config TOML `[tags] keywords = [...]` (seed `["important","work","personal"]`) → Rust loader → `get_tags` command → `$common` store → `options`; write-back (persisting a created tag) is widget-tier (M-RP4.3). NOT built this milestone — sampler passes a literal. A future **dd** `tag-select` fed from a protocol keyword catalog is reserved.

**Verify (Chat self-drove, sampler + CDP 9422, both accents, real — Rule 2).** `vite build` clean. `ids()===89`; children `textfield#{default,max,create}__filter` (no collision); **0 stray chips** (`register={false}` proven, N-064 made real). Getters default `{count:4}`/max `{count:2}`/create `{count:0}`. Freeform create `zzz`→value===label; dedup `ZZZ`→no-op; Backspace-last→0. Popup sections `["Selected (4)","Options"]`, hide-selected (only `later` pickable); pick `later`→count 5, stays open, query cleared. Width: fixed 260 → 2 chips + `+2`, **no clipping**; auto cells fit content (max 295 / create 155). max cap → no-op + `[data-full]` + disabled input. Gear outside `.tag-field` on all 3 cells, mask set, click-safe. `✓` mark gold `rgb(194,136,64)` ↔ blue `rgb(58,122,176)`. 0 orphans. (Note: a stale Svelte HMR state showed a wrong count mid-arc; a clean reload gave the correct measured `cap` — recorded honestly, Rule 1.)

*UI-design note. Component + skin (shipped). → registry (23rd `core`, 6th di composite, matrix 89). Closes the N-054 di-composite backlog. Next: `color-picker` (reuses owned-popup, N-047) → the `widget` definition (N-059→spec) → M-RP4.3.*

---

### N-066 — `color-picker` (M-RP2.29): compact themed picker, `#rrggbbaa`, float-HSV, open-only children

`color-picker` — **7th di composite, 24th `core`**. The themeable, compact answer to native `<input type=color>`, whose popup dialog is OS/Chromium-painted and **unreachable by CSS *or* JS** past the swatch (N-047, reconfirmed live in DevTools: the dialog is a UA-native window, not DOM — no shadow root, nothing to query; dense/compact layout is only possible in an owned popup). Combobox-shaped **owned-popup** (N-063): anchor `textfield` (`__hex`) + live swatch + a **palette** icon in the chevron slot; passive di, owns only `open`; closes on outside-pointerdown (not blur — avoids a race with the in-popup sliders/SV drag).

**Value = canonical `#rrggbbaa`** (8-digit, lowercase, always-valued; default `#000000ff`). **More capable than native** — native emits no alpha at all. Internal source of truth = **HSVA** (`h` 0–360, `s`/`v` 0–100, `a` 0–255); `value` is derived on every change.

**Float-HSV (the fidelity fix).** Rounding HSV to integers made `rgb→hsv→rgb` lossy — seed `#9a6a30ff` came back `#99692fff` (off-by-one per channel), and the first buggy mount wrote the drift **back into the bound sampler state** (sticky until a fresh load). Fix: keep `h`/`s`/`v` as **floats**, round only at display in the HSVA numeric fields. `rgb→hsv(float)→rgb→round` is then lossless for integer rgb inputs. Verified: seeds `#9a6a30ff`/`#2a6090ff` preserved exactly.

**Two guarded effects, no feedback loop.** commit (hsva → `value` + anchor `hexDraft` + `lastHexa` gate; reads hsva only) and parse (user hex edits → hsva, gated by `lastHexa` so reflected writes are no-ops). The gate is what prevents the anchor field and the sliders/SV from ping-ponging. Hex field accepts **6-digit** (pads `ff`); invalid → native `:invalid`, **no commit**.

**Model selector (HEXA/RGBA/HSVA)** swaps the popup **numeric row only** — the SV surface + hue + alpha are HSV-native and identical across models (view-state `model`, reset each mount, **not** published in the getter). Numeric inputs are **raw** `<input>` (no atomic → no registration). **SV surface** is a CSS-gradient `<div>` + positioned thumb (pointer x→S, y→V), **not `<canvas>`** — CDP-readable (N-042 lesson). **Eyedropper** recycles the native `EyeDropper` API (returns `#rrggbb`, keeps current alpha; button hidden when the API is absent — present in WebView2). **8 recent slots**: commit **on close**, dedup, most-recent-first, empty = checkerboard; local `$state` (persistence deferred).

**Matrix accounting — open-only children (correction vs the runbook's 97).** The `__hue`/`__alpha` ranges live **inside the `{#if open}` popup**, so they register **only while the popup is open** — a **live sub-state, like focus**, not a static cell. Stable closed-state count is therefore **+2/cell** (composite + `textfield __hex`) → 2 cells → **89→93**; opening a cell adds its `__hue`/`__alpha` (verified live at **95**). This is the honest deterministic count (combobox/tag-select popups had **no** registered children, so this is the first composite where a registered child is conditionally mounted — treat the open children as a live-verified state, not part of the baseline matrix).

**Skin.** All appearance in `skin.css` (30 `.color-picker*`/`.cp-*` rules); the only inline `style=` are **data-driven values that cannot live in a stylesheet** — current-colour swatch, live SV hue (`--cp-hue`), thumb x/y %, alpha track colour (`--cp-solid`), each recent slot's colour. Zero component `<style>`. (Vite **dev** injects global CSS into one `<style>` tag for HMR — DevTools labels those rules `<style>`; prod `vite build` extracts them to the external `.css`. Not hardcoding.) Cosmetic pass: field sized to content (`12ch`), palette icon **outside** the field on the right (password-field pattern), tight popup line spacing (`--sp-1`), centered model-button text.

**Deferred (D-065):** colorspace attr; persistence of recents + model; alpha-as-% toggle; keyboard nav on the SV surface (pointer-only v1); external-`value`-set-after-mount re-sync (init parses once; passive-di, value is derived output).

*UI-design note. Component + skin (shipped). → registry v0.38 (24th `core`, 7th di composite, matrix 93). **D-069 7th-composite watch: no promotion.** Next: the `widget` tier definition (N-059→spec) → M-RP4.3 (in-app TOML editor + write-back, first widget) → M-RP4.1 (kind-3 number-clamp).*

---

## How to use this file

- New notes go under the current date heading, indexed `N-NNN` continuing the numbering.
- A note that crystallises into a decision is marked with a forward pointer (`→ D-NNN`) and left in place — do not delete.
- A note that is superseded or no longer relevant is marked `SUPERSEDED` (or `DROPPED`) with one-line reason, and left in place.
- The file is append-only in spirit. The full history is the record.
