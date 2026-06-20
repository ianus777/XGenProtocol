# XGen UI — Notes
> **Status**: ACTIVE  
> Version: 0.9  
> Date: May 2026  
> **Last updated**: 2026-06-18  
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
layer 1  component envelopes   → root .type-class structure (N-020), Svelte-scoped
layer 2  skin / theme          → XGen visual identity on top
```
`normalize.css` is the deliberate **point zero** — flatten inconsistency to a known baseline *before* any skin lands. Naming the order is itself the anti-drift move: everyone knows what is foundation and what is skin.

**Status:** not yet vendored in the repo (no `normalize.css` present as of this note). To be brought in **under `ui/`** when the CSS foundation is laid; moving/copying it into place is authorized.

**Adapted, not pristine (expect updates):** a vanilla `normalize.css` predates our situation. We ship into **Tauri webviews** (WebKitGTK / WKWebView / WebView2), not the open browser population — so part of the work is *trimming* defensive rules we don't need and *adding* webview-specific quirks; part is aligning its zero-point with the N-020 envelope + skin needs. Because it becomes a **maintained, modified** baseline, its deviations from upstream must be **recorded where the file lives** (header note or an "XGen deviations from upstream" comment block) — or a future reader assumes it is pristine and a drift trap is born.

*UI-implementation rule; no protocol/data implication.*

---

## How to use this file

- New notes go under the current date heading, indexed `N-NNN` continuing the numbering.
- A note that crystallises into a decision is marked with a forward pointer (`→ D-NNN`) and left in place — do not delete.
- A note that is superseded or no longer relevant is marked `SUPERSEDED` (or `DROPPED`) with one-line reason, and left in place.
- The file is append-only in spirit. The full history is the record.
