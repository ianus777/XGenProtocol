# XGen UI — Notes
> **Status**: ACTIVE  
> Version: 0.1  
> Date: May 2026  
> **Last updated**: 2026-05-16  
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

## How to use this file

- New notes go under the current date heading, indexed `N-NNN` continuing the numbering.
- A note that crystallises into a decision is marked with a forward pointer (`→ D-NNN`) and left in place — do not delete.
- A note that is superseded or no longer relevant is marked `SUPERSEDED` (or `DROPPED`) with one-line reason, and left in place.
- The file is append-only in spirit. The full history is the record.
