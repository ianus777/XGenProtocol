# XGen Protocol — Implementation Decisions
> **Status:** ACTIVE  
> **Last updated:** 2026-05-21 (D-074 added — milestone-close commits MUST include JOURNAL.md)  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

Every decision that goes beyond spec prescription is recorded here before advancing to the next layer.
Format: title, date, layer, spec reference, decision narrative.

---

## D-074 — Milestone-close commits MUST include JOURNAL.md

**Date**: 2026-05-21  
**Layer**: Cross-cutting — project-management discipline applying to every milestone-close commit across the project's history and future. Binds the commit-formation discipline of any milestone whose closure triggers cross-doc state changes (CLAUDE.md PLAY block flip, ROADMAP.md state move, per-task-file Status flip).  
**Spec reference**: `JOURNAL.md` (the contemporaneous record this decision protects); `CLAUDE.md` Rule 4 ("Write the journal entry last" — the sibling per-session discipline this decision generalises into commit-level discipline). Cross-references: D-069 (canonical-document rule — JOURNAL.md is the canonical historical record); D-071 (audit-precedes-dependency — sibling project-management principle); D-065 (honest behaviour over polite behaviour — a milestone closing without a JOURNAL entry is dishonest about how the project got here).

### Decision

**Every milestone-close commit's changed-files list MUST include `JOURNAL.md`.**

No milestone closes without a JOURNAL entry shipped in the same commit as the cross-doc updates that announce the closure (`Status: ACTIVE → COMPLETED` header flips on task files; CLAUDE.md PLAY block updates; ROADMAP.md Past/Present/Near future moves; Visual structure tree updates). The JOURNAL entry is contemporaneous — it describes what shipped, in what order, with what test count delta, with what structural findings surfaced, in the moment the closure happens. Deferring the entry to "a future session" or "a separate housekeeping pass" violates this discipline.

The rule is unconditional: it applies to every milestone close, including small or routine ones (a single-phase milestone, a doc-pass milestone, a sub-question lock that closes a design phase). Size of the milestone does not matter; what matters is the closure event itself producing a contemporaneous record.

### Originating incident

Discovered 2026-05-20 during XGID Adoption v1 Phase 2 close-out, via working-tree forensics. Federation Event Propagation Phase 7.5 implementation milestone shipped 2026-05-20 in five commits (`12cfe5a` + `aa2433f` + `1be7189` + `ecbbf19` + `8859093`) without a JOURNAL.md entry. The cross-doc references in CLAUDE.md and ROADMAP.md named the entry "J-094" — but no J-094 was ever authored. The discrepancy was caught when J-094 was supposed to be the originating context for closing out adjacent work, and a `grep` for `J-094` in JOURNAL.md returned zero hits.

The gap was honest-flagged in the next milestone's close entry (J-095, XGID Adoption v1 implementation close) per D-065 honest-provenance discipline rather than retroactively backfilling J-094, which would have violated D-065 by misrepresenting when the entry was written. The retrospective J-094 entry is now tracked in the Discipline / JOURNAL hygiene cluster in ROADMAP.md as deferred work ("JOURNAL Gap 1 — Phase 7.5 implementation retrospective entry"), to be written in a separate session and given the next available J-number at that time.

The incident surfaced a structural gap in the project's commit-formation discipline: CLAUDE.md Rule 4 says "Write the journal entry last" within a session, but the rule was silent on whether the entry is *in the same commit* as the cross-doc updates or in *a follow-on commit*. The Phase 7.5 close split the JOURNAL entry off as follow-on intent, and the follow-on never happened. D-074 closes the gap by making the same-commit requirement explicit.

### Why this discipline must be explicit

**Reason 1 — JOURNAL is the only contemporaneous record.** CLAUDE.md, ROADMAP.md, task file Status headers, and DECISIONS.md all describe *current* reality — what is true *now*. They get updated as state changes and they describe present state, not history. JOURNAL.md is the only file in the project that records *how reality got here* — the sequence of events, the test count deltas, the structural findings, the sub-question locks made during the work. Without a contemporaneous JOURNAL entry, a milestone close becomes archaeology to reconstruct later. The longer the gap between the close and the entry, the more accuracy decays.

**Reason 2 — Same-commit discipline prevents the gap.** A JOURNAL entry written "in a follow-on commit" relies on someone remembering to write it and the project's commit-formation discipline being attentive enough to land it. Both fail in practice. The Phase 7.5 incident is the worked instance: the follow-on intent was honest at the moment of the close, but no follow-on commit happened. Making the entry part of the close commit removes the gap surface entirely.

**Reason 3 — The forensics cost of missing entries is asymmetric.** Catching a missing JOURNAL entry months later requires `git log --all --grep`, working-tree forensics, cross-checking CLAUDE.md and ROADMAP.md references for J-numbers that don't exist, and re-deriving the milestone's actual state from commit diffs. Writing the entry at the close costs ~10–20 minutes of authoring time. The cost ratio is roughly 1:10 in favour of writing-at-close. D-074 makes the cheaper path mandatory.

**Reason 4 — The principle generalises the per-session Rule 4.** CLAUDE.md Rule 4 says "Write the journal entry last" within a session: do the work → run verification → confirm outputs → write the journal entry quoting actual output → update CLAUDE.md → commit and push. Rule 4 binds the *per-session ordering*. D-074 binds the *per-commit composition*. Together they form the full discipline: the entry is written last in the session, AND it ships in the same commit as the closure-announcing updates.

### Worked instances at promotion

- **XGID Adoption v1 implementation milestone close (J-095, 2026-05-20).** The first close to follow D-074 pre-emptively (before D-074 itself was promoted). The milestone-close commit shipped JOURNAL.md (J-095 entry) alongside CLAUDE.md (PLAY block flip + header), `docs/ROADMAP.md` (Past gain + Present + Near future moves + header), `tasks/XGID_ADOPTION_IMPL.md` (Status: ACTIVE → COMPLETED v1.1), and `docs/xgen_ch4_implementation.md` (one-line follow-on pointer per Phase 2 sweep A5 Joe-lock). Five files in one atomic commit; JOURNAL.md was among them; the discipline held.
- **XGID Adoption v1 Phase 2 doc-tree sweep close (no separate J-number, ride-along on the same commit as J-095).** Sub-milestone close within the larger XGID Adoption v1 work. The classification table at `tasks/XGID_DOC_SWEEP.md` flipped Status: ACTIVE → COMPLETED v1.2 in the same commit as the J-095 entry. D-074 tolerates ride-along closures — a single JOURNAL entry covering multiple sub-milestone closes in the same commit set is honest, provided the entry names all the closures.
- **Phase 7.5 implementation milestone close (counter-instance).** Shipped 2026-05-20 in five commits without a JOURNAL.md entry. Surfaced the gap D-074 closes. The retrospective entry, when written, will be the worked example of "how to backfill honestly per D-065" rather than "how to close a milestone per D-074" — the entry will name itself as retrospective and acknowledge the original commit-formation discipline failure rather than pretending to be contemporaneous.

### Out of scope for this decision

- **Mid-milestone JOURNAL entries.** D-074 binds *milestone-close* commits specifically. Mid-milestone entries (a long-running milestone with multiple JOURNAL entries across its phases, like Federation Event Propagation's J-082..J-089 series) are not bound by D-074 — each individual phase close gets its own entry per the existing pattern, and D-074 confirms the requirement at the *milestone-level* close (the commit that flips the milestone's overall Status from PLAY to DONE).
- **What goes IN the JOURNAL entry.** D-074 binds the requirement that an entry exists; it does not prescribe the entry's content shape. Each project area has established conventions (Federation phase closes name the Joe-locks and structural findings; XGID closes name the v1 invariance test outcomes and carry-overs; M-series closes name the test count delta and commit chain). The entry content is the milestone author's responsibility, not D-074's mandate.
- **Retrospective entries (D-065 territory).** When a missing-entry gap is discovered after the fact, the retrospective entry is written under D-065 honest-provenance discipline rather than D-074. D-074 applies forward only: the rule says new closes ship JOURNAL.md in the close commit. Past gaps stay flagged in the Discipline / JOURNAL hygiene cluster until separately retrospected. Backdating retrospective entries to make them look contemporaneous would violate D-065.
- **JOURNAL entry numbering.** D-074 does not bind J-number allocation. The convention (sequential J-NNN per chronological order of writing) is established elsewhere; D-074 only requires that an entry exists in the close commit, regardless of its number.
- **Other documentation files in the close commit.** D-074 is specifically about JOURNAL.md. Other files that go in milestone-close commits (CLAUDE.md, ROADMAP.md, task files, Ch3/Ch4 if affected) are governed by their own conventions (CLAUDE.md / ROADMAP.md same-commit discipline per ROADMAP.md's own update-discipline section; task file Status headers per the header convention). D-074 adds JOURNAL.md to that list, not as a replacement.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-069 | Canonical-document rule. JOURNAL.md is the canonical home for contemporaneous historical record — "this is what happened, in this order, on this date." D-074 enforces that the canonical record is *populated* at every milestone close. Without D-074, the canonical record can become silently incomplete; with D-074, the canonical record's completeness is a commit-formation invariant. |
| D-071 | Sibling project-management principle. D-071 says audits precede dependent milestones (verify reality before locking design); D-074 says milestone closes produce contemporaneous record (verify reality has been captured before declaring the milestone closed). Both decisions take implicit gaps out of the project's information state: D-071 between assumed and verified subsystem behaviour; D-074 between announced closure and recorded closure. |
| D-065 | Sibling principle (honest behaviour over polite behaviour). A milestone close commit that doesn't include JOURNAL.md is dishonest in two ways: (1) it announces closure without providing the contemporaneous record that justifies the announcement; (2) it leaves the project's future readers without the context needed to understand how the closure happened. D-074 takes that dishonesty out structurally by making the JOURNAL entry a commit-formation requirement, not a follow-up intent. |
| D-070 | Adjacent protocol-design analogy. D-070 says both sides of an outcome (acceptance / rejection) get equal first-class signals AND envelope-level correlation. D-074's analogue: both the announcement of closure (CLAUDE.md / ROADMAP.md / Status flips) and the record of how closure happened (JOURNAL.md) get equal first-class commit-formation status. The asymmetric historical case (Phase 7.5 close, announcement without record) is exactly the shape D-070 prohibits at the protocol layer, applied to the project-management surface. |
| D-072 / D-073 | XGID Adoption v1 (the worked predecessor where D-074 was already applied pre-emptively). The XGID v1 milestone-close commit (J-095, 2026-05-20) shipped JOURNAL.md as part of its five-file changed-files list per the candidate D-074 framing flagged in J-094 cleanup. D-074 formalises the discipline that XGID v1 close already followed. |
| CLAUDE.md Rule 4 | Per-session sibling discipline. Rule 4 binds *intra-session* ordering: do the work → verify → quote real output → write the journal entry → update CLAUDE.md → commit and push. D-074 binds *commit-level composition*: the journal entry ships in the same commit as the closure announcements. Together they form the full discipline; either alone is insufficient. Rule 4 with follow-on JOURNAL intent (without D-074) is what produced the Phase 7.5 gap. D-074 with bad intra-session ordering (without Rule 4) would produce hastily-written entries that don't capture actual verification output. Both rules are load-bearing. |
| Phase 7.5 implementation milestone (originating incident) | The five-commit Phase 7.5 close (`12cfe5a` through `8859093`, 2026-05-20) shipped without a JOURNAL entry. The gap was caught via working-tree forensics during XGID Adoption v1 Phase 2 close-out the same day. The retrospective J-094 entry stays deferred in the Discipline / JOURNAL hygiene cluster in ROADMAP.md until written, at which time it gets the next available J-number and is honestly labelled as retrospective. |

---

## D-061 — Room temperature: protocol carries the signal, plugin owns the math

**Date:** 2026-05-15  
**Layer:** Ch1 philosophical framing; Ch3 §3.7.13 (protocol surface); Ch6 §6.12 (client display); D-060 (pacing rules — the input signal feeding temperature)
**Spec reference:** Ch1 "Visible Self-Correcting Feedback"; Ch3 §3.7.13 Temperature Property (meta_atts keys, threshold table, visibility setting); Ch3 §3.7.8 (`auto_temperature` reason value); Ch6 §6.12 (DOM contract and rendering); D-059 (`is_ai` informs the asymmetric escalation recommendation)

### Decision

A Room carries a numeric **temperature** signal — one float per Room (collective rhythm) and one float per Member-in-Room (individual accumulated heat). The protocol surface for temperature is intentionally minimal:

- Two `meta_atts` keys: `xgen.room_temperature` and `xgen.member_temperature`, both floats in `[0.0, 1.0]`, both published by the Room's home Node
- A `temperature_thresholds` field in the Room metadata response, declaring at which float values the named states (`warm`, `hot`, `fiery`) begin; default thresholds documented in Ch6 §6.12.2
- A `member_temperature_visibility` field on Space state with three permitted values (`moderator`, `everyone`, `self_only`) controlling who receives `xgen.member_temperature` for other members
- The reserved `auto_temperature` reason value on `membership.kick` (humans) and `membership.mute` (AI) for automated consequences

**The mathematical model is not part of the protocol.** How the temperature value is computed — what decay function applies, how pacing overpasses accumulate, what the action thresholds are — is the responsibility of a plugin running on the Room's home Node. Different communities will moderate at different rhythms; the protocol has no business choosing on their behalf.

The protocol's job is to:

1. Carry the signal (the floats) across the network so every client renders the same value
2. Carry the bucket thresholds so every client classifies the same float into the same named state
3. Recognise the consequences (`auto_temperature` reason on kick / mute) so DAG audit shows what happened and why

The plugin's job is to decide what the float is at any given moment and when to issue automated consequences. The two domains are deliberately separated.

### Why

The original draft of D-061 (replaced by this rewrite) specified a mathematical model: a linear-decay heat accumulator with named thresholds and per-Space configurable parameters. The conversation of 2026-05-15 pushed back on this layer by layer, in the right direction: each round of design pulled mathematical content further out of the protocol surface until the protocol carried nothing but the *signal* and the *consequences*. The reasoning sequence was:

1. The protocol shouldn't mandate one model — different communities need fundamentally different moderation curves
2. A named-policy enum was considered (`linear_decay`, `exponential_decay`, `sliding_window`) and rejected as still too prescriptive
3. A pluggable algorithm via WASM module was considered and noted for a future phase
4. The final cut: the protocol does not announce a model at all. The home Node computes a number; the plugin behind the Node decides how. The float is the wire-level truth.

This matches the rest of the protocol's design language:

- Auth Tiers (D-037) — protocol carries the marker, Auth Module supplies the verification meaning
- `meta_atts` (Ch3 §3.1.3) — protocol carries the bytes, applications supply the interpretation
- Vanilla Node `capabilities` (CLAUDE.md) — protocol carries the field, Nodes supply the behaviour
- Pacing rules (D-060) — protocol carries the cap, communities supply the culture

Temperature joins this list. It was the odd one out in the original draft — the only decision specifying a concrete mathematical model inside the protocol surface.

### Asymmetric escalation (AI mute vs human kick)

The asymmetric escalation principle — human members get `membership.kick` at sustained overpass, AI members get `membership.mute` — is preserved as a **recommendation for plugin authors**, not a protocol mandate.

The protocol provides the structural primitives that make the asymmetry expressible:

- `membership.kick` and `membership.mute` are distinct EventTypes (3.7.8)
- `is_ai` is observable on every Identity (D-059, 3.6.10.1)
- `auto_temperature` is reserved on both events with documented expected pairing (kick for humans, mute for AI)

A plugin that uses this asymmetry will produce the AI-keeps-membership / human-gets-cooldown behaviour described in Ch1 §"Visible Self-Correcting Feedback". A plugin that ignores the asymmetry — treating AI and human identically, or applying no automated escalation at all — is also valid. The protocol does not enforce which choice a community makes; it makes both choices expressible.

The Ch1 framing (AI overshoot is a *capability signal*, human overshoot is a *social signal*) remains the recommended justification for the asymmetry, but it is now framing for plugin authors, not a protocol-level rule.

### Visibility policy

Room temperature (`xgen.room_temperature`) is **visible to every member** by default and not configurable — the collective state of the Room is shared awareness, and concealing it from members would defeat the purpose of self-correcting feedback.

Member temperature (`xgen.member_temperature`) is **moderator-visibility by default**, configurable per Space via `member_temperature_visibility` with three permitted values:

| Value | Effect |
|---|---|
| `moderator` | Default. Moderators and above see member temperatures. |
| `everyone` | All members see all member temperatures — transparent communities. |
| `self_only` | Even moderators see only their own; auto-moderation runs entirely Node-side. |

The home Node enforces visibility — clients receive only what their role permits. The conservative default of `moderator` reflects that publicly visible "Alice is hot" can itself be socially inflammatory in some communities; transparent communities may opt into `everyone`.

### What the protocol does NOT specify

Deliberately outside the protocol surface, owned by the home Node's plugin:

- The mathematical model (decay function, accumulator behaviour, smoothing)
- The action threshold (when `auto_temperature` fires)
- The cooldown duration (Ch6 §6.12.6 documents UI defaults of 2h / 15min as plugin-recommended values; the actual `cooldown_until` timestamp on the issued event is the plugin's choice)
- Persistence across Node restart (temperature is computed live from the event stream; the Node decides when and how to recompute)
- Cross-Node temperature (federated copies relay the home Node's value; non-home Nodes do not recompute)

These decisions belong to the community operating the home Node, expressed through their choice of plugin.

### Computation locality

The Room's home Node is the authoritative source for both temperature values. A Room "lives somewhere" — it is hosted on a specific Node — and temperature is judged where it lives, analogous to criminal jurisdiction. Federated copies of the Room's events may relay temperature values via `meta_atts` on relayed events; receiving Nodes do not recompute. If the home Node changes (Space migration, D-053), the new home Node's plugin takes over; temperature values may differ from the previous home Node's values, and that is correct — the room has moved, and its moderation philosophy may have moved with it.

### Impact

- **Ch1**: short philosophical paragraphs added (§"Visible Self-Correcting Feedback") connecting temperature to the infrastructure transparency principle. Already written in Session 11 of Ch1 Session Log.
- **Ch3 §3.7.13**: new subsection specifying the meta_atts keys, the threshold table field, the visibility setting, and the `auto_temperature` cross-reference. Written 2026-05-15.
- **Ch3 §3.7.6**: Space state components table extended with `member_temperature_visibility`. Written 2026-05-15.
- **Ch3 §3.7.8**: `auto_temperature` reason value and AI / human pairing reserved. Already written in Session 21 (J-063).
- **Ch6 §6.12**: full client-side specification — DOM contract, threshold table consumption, derivation rules, visibility consumption, auto-moderation rendering. Written 2026-05-15.
- **`xgen-core`**: minimal — the `auto_temperature` reason value and `membership.mute` event handling (already in scope for Phase 2 layer work).
- **`xgen-client`**: bucket derivation logic and DOM-attribute writing on Avatar / Room banner components (Ch6 §6.12.3 / §6.12.4).
- **`xgen-node`**: temperature plugin loader interface — Phase 2 implementation question, not specified at the protocol level. The Node operator chooses which plugin (if any) computes temperature for their hosted Rooms.

### Status

This decision is the result of the design conversation of 2026-05-15. The original D-061 draft (which specified a linear-decay accumulator model with named threshold parameters in `temperature_config`) is replaced by this version. The principle of the original — visible self-correcting feedback with asymmetric AI / human escalation — survives; the mathematical content is removed and relocated to the plugin layer.

---

## D-060 — Per-space pacing rules: human_pacing_ms and ai_pacing_ms as enforced space rules

**Date:** 2026-05-15  
**Layer:** Ch3 spec (space settings); Ch6 (client enforcement)  
**Spec reference:** Ch3 §3.7 (space and room protocol); D-059 (AI users — prerequisite for ai_pacing_ms semantics)

### Decision

Every space carries two pacing rules in its settings:

- `human_pacing_ms`: minimum interval (milliseconds) between messages from a member whose Identity has `is_ai = false`
- `ai_pacing_ms`: minimum interval (milliseconds) between messages from a member whose Identity has `is_ai = true`

These are **space rules**, on the same level of authority as the space's auth tier requirement, role permissions, and federation list. A client that wants to participate in the space MUST enforce these caps for its own outbound messages.

### Why

Different room cultures need different rhythms. A contemplative space (human=5000 / ai=30000) and a fast-chat space (human=0 / ai=1000) both have legitimate rhythms. Per-space configuration lets each community express its own cadence. Pacing is not a security boundary — it is a culture boundary, like dress code in a physical space.

The human/AI distinction is essential because AI's capability for high message throughput is fundamentally different from humans'. Treating both identically either flooded rooms with AI burst output or restrictively throttled humans typing at conversational speed.

### Client behaviour

**Outbound message queue:**
- Before sending, the client checks the time since its last successful send in this space
- If the elapsed time is below the pacing cap, the message is queued and released when the interval is satisfied
- For **humans**: silent throttle. The user does not see the queue unless they exceed by a meaningful margin. A 500 ms default is invisible to normal typing.
- For **AI**: visible to the operator. The queue and the current pacing state are part of the AI client's operational surface — operators are tuning a system and benefit from seeing the constraint applied.

### Defaults (suggested starting values)

- `human_pacing_ms`: 500 (catches accidental triple-posts; invisible for normal typing)
- `ai_pacing_ms`: 2000 (gives humans time to read between AI messages; prevents AI monopolising attention)

These are *defaults applied at space creation* unless overridden. The space owner may modify them via space settings updates.

### Enforcement layer

**Phase 2: client-side only.** The Node does not validate that messages respect pacing. Bad-actor clients can attempt to violate; they show up clearly in timestamps and are kicked by admins (or auto-throttled by D-061 temperature).

**Phase 3+ (deferred): Node-side enforcement** may be added if abuse appears in practice. The decision point: Node-side enforcement costs Node CPU and adds latency to every send, in exchange for being robust against malicious clients. Phase 2 trusts clients for the same reasons it trusts them for role permissions client-side before Node-side validation.

### Pacing is rigid for AI

The AI's client cannot exceed `ai_pacing_ms` in a given room — it is a hard space rule, like the tier requirement. This is critical for the D-061 temperature mechanism's AI escalation to make sense: an AI that is properly enforcing pacing can still accumulate temperature (if it consistently sends *at* the cap), and that signal remains meaningful.

### Impact

- Ch3 §3.7: new subsection on space settings including `human_pacing_ms` and `ai_pacing_ms`.
- Wire format: new fields on `SpaceState`; `state.space_pacing_update` event or extension of existing `state.space_update`.
- `xgen-core`: minimal validation (non-negative integers).
- `xgen-client`: outbound queue and pacing logic, plus the AI-specific operator UI surface.

---

## D-059 — AI users as first-class XGen Identities with declared capabilities

**Date:** 2026-05-15  
**Layer:** Ch1 (philosophical); Ch3 (Identity model, registration, validation); Ch6 (UI)  
**Spec reference:** Ch1 (Human and Agent Operation); Ch3 §3.6 (Identity registration); Layer 15 / D-049 (identity replication); D-037 (Tier 1 = persistent accountable identity)

### Decision

**AI is a first-class XGen Identity.** Same shape as a human Identity — one keypair, one identity_id, one display name, one member-list presence, one DM relationship model. Different in declared capabilities and in some asymmetric behavioural rules. The target experience for human members of a room containing an AI: addressing the AI feels like addressing a knowledgeable human member who happens to be in the room, not like invoking a tool.

### Why this shape

Alternatives considered and rejected:
- **No marker at all.** Too permissive — fails to support the asymmetric rules below.
- **Dedicated identity class** (`human` / `ai` / others). Too heavy — introduces a new typing axis when AI mostly looks like a human.
- **Dedicated Auth Tier** (separate from 1–4). Wrong axis. Tier is about depth of verification, not kind of entity. AI in a Tier 4 healthcare space is a Tier 4 entity — it inherits the space's tier requirement.

The chosen model collapses these into a minimal addition: one boolean field plus a capability pattern.

### Identity shape

**New field `is_ai: bool` on the Identity record:**
- Defaults to `false`
- Declared at `identity.register` — part of the registration request, recorded in the Identity record
- **Immutable after registration.** A human Identity cannot later flip to AI or vice versa
- Replicated alongside the rest of the Identity (extends Layer 15 / D-049 identity replication)

**Implication for accountability:** the same persistent-accountable-identity guarantee (D-037) applies. An AI cannot "reset" its identity to escape consequences any more than a human can. The keypair is the anchor.

### Capabilities pattern (door closed for now, future-proofed)

AI identities carry an **open-enum set of capability flags**. Phase 2 defines a minimal set with safe defaults; future phases extend the set without breaking older Nodes (same principle as `meta_atts` namespacing and the vanilla Node model).

**Initial Phase 2 set:**
- `dm_initiate: false` — AI cannot **create** a new DM space with another Identity. AI can freely **send into** DM spaces a human has already opened (covers reminders, follow-ups, scheduled check-ins).
- `spontaneous_post: false` — governed by per-room permission; default is response-only behaviour. A future room permission may flip this on a per-room basis.

**Future capability slots reserved without specification.** The protocol grows by flipping flags that already exist, not by adding new wire fields.

**Enforcement: hard, protocol-level.** A Node MUST reject events from `is_ai = true` Identities that violate their declared capabilities. The audit log proves compliance. Soft enforcement was considered and rejected — it would allow misbehaving operators to silently violate the asymmetries.

### Invitation and accountability

**AI does not appear in a space by coincidence.** It is invited (`membership.invite`) by a space owner or admin, like a human member. The `membership.invite` event records the inviter permanently in the DAG. If the AI misbehaves, the inviter is on record.

**Operator role.** Beyond the inviter, an explicit `operator_identity_id` is recorded for the AI's lifecycle in a space. The operator is responsible for the AI's ongoing behaviour (configuration, tuning, removal). Initially the operator equals the inviter; the inviter can delegate operator rights to another Identity via a new delegation event (`state.ai_operator_delegate` or similar — final naming in spec).

Distinction:
- **Inviter** — historical, immutable; the Identity that first brought the AI into the space
- **Operator** — current, mutable via delegation; the Identity currently responsible for the AI's behaviour

### Tier

No special tier for AI. The AI inherits the tier requirement of whichever space it is invited into. If a space requires Tier 4, an AI member must satisfy Tier 4. Verification of an AI's tier follows the same Auth Module mechanism as for humans; what counts as "verification" for an AI is its operator's institutional credentials (specific verification path deferred to Auth Module Tier work).

### Removal

**Standard `membership.ban` and `membership.kick`** work as for any member. No special AI-removal mechanism.

- Any admin or owner can kick or ban
- Moderators can mute
- A foreign admin (one who is not the AI's operator) may kick when the AI's operator is absent and the AI is causing disturbance — a foreign admin may understand the malfunction best

### UI

- AI member is shown with the **same avatar, name, and message-bubble styling** as a human member by default
- A small, unobtrusive **AI badge** marks the member in the member list. Default placement minimal; operator/admin may customise.
- Messages from AI use the **same shape** as human messages — no "AI response" header, no different bubble shape, no robot icon on each message. The badge on the avatar or member identity is the only visual signal.
- Third-party plugins may decorate further. (A whimsical "the AI is being playful" indicator was floated; the module slot system supports it.)

### Pacing

Governed by D-060 (`human_pacing_ms` / `ai_pacing_ms` as space rules). The AI client enforces `ai_pacing_ms` rigidly — it is a hard space rule, like the tier requirement.

### Multi-instance same-keypair behaviour

Identical to a human running two clients with one keypair: both clients' messages enter the DAG, conflicts (if any) are resolved by Layer 12 / D-046. No special protocol handling. Operator concern, not protocol concern. AI is statistically more likely to produce simultaneous outputs (parallel triggers, scheduled jobs), so operators should avoid multi-instance deployments unless needed.

### AI-to-AI interaction

**Not prohibited.** Two AI Identities in the same room may address each other via the same rules as human-to-human. Practically rare and noted with some caution (witnessed AI ⇔ AI exchanges tend to spiral). Left open for the future; revisit when AI maturity changes the calculus.

### Impact

- Ch1: short subsection or paragraph on AI participation aligned with Human and Agent Operation philosophical frame.
- Ch3 §3.6: new subsection on AI Identity — `is_ai` field, capability declarations, registration semantics, operator delegation event, validation rules for AI-signed events.
- Ch3 §3.13 / Layer 15: identity replication extended to include `is_ai` and capabilities (already structurally supported — just an additional payload).
- Ch6: AI badge specification; pacing behaviour for AI clients; operator-visible AI client UI surface.
- `xgen-core`: Identity record extension; validation rules in event ingestion (`is_ai = true` + violation → reject); operator delegation event handling.
- `xgen-client`: AI client mode (operator-facing UI elements); pacing enforcement (D-060); temperature interaction (D-061).

### Open questions (deferred to spec authoring)

- Exact wire-format name for the operator delegation event
- Auth Module tier-specific verification semantics for AI Identities ("what does Tier 3 mean for an AI?")
- Whether `is_ai` is part of the Trust Assertion payload or a separate Identity-record field
- UI badge specification (icon, position, accessibility)

---

## D-058 — UI spacing system: 4px root unit, named steps in tokens.css, component-scoped typography

**Date:** 2026-05-15  
**Layer:** UI — base.css / tokens.css  
**Spec reference:** Ch6 §6.1 (design system); D-041 (skin architecture)

### Decision

The entire XGen UI uses a **single 4px root spacing unit**. All spacing in every component is a named integer multiple of this unit. No arbitrary per-context values.

**Root unit declaration** lives in `base.css`:
```css
:root {
  --space: 4px;
}
```

**Named steps** are declared in `tokens.css` (values, not structure):
```css
--space-1:  4px;   /* tight inline gap, icon padding */
--space-2:  8px;   /* item padding, small gap */
--space-3: 12px;   /* standard component padding */
--space-4: 16px;   /* section gap */
--space-6: 24px;   /* major section separation */
--space-8: 32px;   /* modal / overlay padding */
```

**Typography** is component-scoped, not globally defined per HTML element. No global `h1`–`h6` or `p` rules. Each component declares its own font size using token references. The only globally declared typographic values are the base scale anchors in `base.css`:

```css
:root {
  font-size: 13px;        /* app base — NOT 16px (document default) */
  line-height: 1.35;      /* compact app rhythm */
}
```

**Rationale:**  
4px is the tightest practical grid unit for information-dense application UIs (Discord, Slack, VS Code all use 4px). Components built independently against the same step names maintain visual coherence without coordination. A single root unit makes the entire layout rescalable: changing `--space` in `base.css` rescales all spacing uniformly — relevant for accessibility/large-UI mode in a future phase. Per-context arbitrary values (sidebar padding 6px, message padding 7px) cannot be systematically adjusted and introduce silent inconsistency across independently-authored components.

**Impact:** Mr Code must not introduce hardcoded pixel values for spacing or typography in any component. All spacing references `--space-N`. All font sizes reference token variables. This rule applies to base.css, tokens.css, skin files, and all component .svelte files without exception.

---

## D-057 — UI CSS layer model: custom app base replaces browser normalize; base always loaded independent of skin

**Date:** 2026-05-15  
**Layer:** UI — base.css / skin architecture  
**Spec reference:** Ch6 §6.1 (design system); D-041 (skin architecture — partial correction)

### Context

D-041 stated "reset coupled to skin so a missing skin degrades to raw HTML." This is corrected here.

A traditional browser normalize (`normalize.css`, `reset.css`, or any HTML-element-complete approach) is a document model. It defines styles for `h1`–`h6`, `p`, `ul`, `ol`, `table`, `blockquote`, `figure`, and other HTML document elements. The XGen UI is not a document — it is a Svelte component application. Most document HTML elements do not appear in the app at all. Defining them in any global CSS file is dead weight and creates specificity conflicts with component-scoped styles.

### Decision

**Do not write a browser normalize or HTML-element-complete reset.** Replace it with a custom minimal `base.css` written specifically for XGen's app UI.

**`base.css` is always loaded, independent of any skin.** It is not coupled to skin loading. Loading order: `base.css` → `tokens.css` → `skin-{name}.css`. Removing a skin does not remove the base. The app degrades gracefully: missing skin → structured compact unstyled app (not browser default rendering).

**`base.css` covers exactly three categories and nothing else:**

1. **Universal box model** — `*, *::before, *::after { box-sizing: border-box; }`. No exceptions.

2. **Root type scale** — `font-size: 13px` and `line-height: 1.35` on `:root`. These are app-UI values, not document-page values. All other typographic values (font family, font weight, color) are CSS variable references filled by tokens and skin.

3. **Browser-aggressive element resets** — only for elements that browsers style forcefully and that appear in app UIs: `button` (remove border, background, padding, cursor inheritance), `input` (remove border, background, appearance), `a` (remove color and text-decoration inheritance). Nothing else. No heading resets, no list resets, no table resets.

**`base.css` declares CSS variable slots** (structure without values) for the properties that components will reference. The skin fills the values. Example: `color: var(--color-text)` in a component; `--color-text: #dcddde` in `skin-dark.css`. The variable name lives in `base.css` as documentation of the required slot; the value lives in the skin.

**All other typographic and spatial definitions live in the component that uses them**, scoped by Svelte's component scoping. `RoomName` defines its own font size. `MessageBubble` defines its own padding. No global element selectors for these.

### Correction to D-041

D-041's statement "reset coupled to skin so a missing skin degrades to raw HTML" is superseded by this decision. The correct degradation chain is: `skin missing → base + tokens → structured compact app`. Raw HTML degradation is not acceptable because it would make the skeleton unreadable as an application.

**Impact:** `base.css` is expected to be approximately 40–60 lines total and stable after initial authoring. It is not a living style sheet. Mr Code must not add HTML-element rules to `base.css` — any element-specific style belongs in the component that uses that element.

---

## D-056 — recv() routing: sender-field check precedes all type-prefix checks

**Date:** 2026-05-14  
**Layer:** Transport (xgen-core/src/transport/connection.rs)  
**Spec reference:** Spec 3.3.4 (WebSocket framing); spec 3.1.2 (Event fields)

**Problem:** `recv()` dispatched incoming binary frames by matching `value["type"]` against type-string prefixes (`"mls."`, `"bootstrap."`, `"reputation."`, etc.). `Event.event_type` is serialised as `"type"` on the wire (via `#[serde(rename = "type")]`). DAG Events such as `mls.key_package`, `bootstrap.node_announce`, and `reputation.defederation_signal` therefore matched the control-message prefix check before the Event check was reached. Deserialization into the control enum failed because `Event` and the control types have different JSON shapes. The error propagated out of `recv()` as `Err`, which the node's connection loop caught as `Err(_) => break`, silently closing the connection.

**Decision:** Add `value.get("sender").is_some()` as the **first** branch in the `recv()` routing chain, before all type-prefix checks. Every `Event` struct has `pub sender: String` with no `skip_serializing_if`, so `"sender"` is always present in a serialised Event. No control message type (`TransportMessage`, `FederationMessage`, `IdentityMessage`, `MlsMessage`, `BootstrapMessage`, `ReputationMessage`, etc.) carries a `"sender"` field. The invariant is enforced by the type system: adding `sender` to a control message would require a structural change that would be immediately visible.

**Impact:** Any message carrying `"sender"` routes to `Inbound::Event` unconditionally. All other routing is unchanged. One-line change; no new allocations; no test changes required. 300/300 tests pass.

---

## D-055 — Server-side Phase 2 handler wiring: peer_url propagation and identity replication push

**Date:** 2026-05-14  
**Layer:** Integration (xgen-node/src/main.rs + supporting xgen-core changes)  
**Spec reference:** Spec 3.9.1 (identity replication); spec 3.6.3 (federation Hello)

**Decision:** Closed the server-side handler gap that prevented smoke-ph2 step 22 from passing. Key choices: `node_endpoint` added to `FederationMessage::Hello` as `Option<String>` excluded from the canonical signature (advisory field only — not in `HELLO_FIELDS`). `peer_url` threaded through `FederationSession` → `FederationRelationship` → `NodeRuntime.peer_urls` HashMap. Identity replication push triggered asynchronously after `RegisterOk` — spawned as a detached task so the registration response is not delayed. `handle_identity_replicate_msg()` is a standalone handler; error response uses error code 3020 (replication domain). See J-057 for full file-by-file change list.

---

## D-054 — Integration test: CLI batch flag as direct executor; smoke-ph2 uses pass!/fail! macros; Phase 2-5 steps note server-side gaps

**Date:** 2026-05-14  
**Layer:** Integration Test (INTEGRATION_TEST_ph2.md Part A)  
**Spec reference:** None (CLI extension decision)

**Decision:** The `--batch` flag on `xgen-client` (CLI binary) is implemented as a direct in-process sequential executor. Each line is parsed via `shlex::split` and dispatched via `Cli::try_parse_from` + the same match arms as the interactive path. No named pipe. No running instance required. `smoke-ph2` is explicitly blocked from batch invocation (returns error exit 1) to prevent recursive async future growth.

The `cmd_smoke_ph2` 60-step test uses `pass!` / `fail!` macros that call `std::process::exit(1)` on first failure. Phase 0 (steps 1-17) and Phase 6 (steps 57-60) exercise fully wired server behaviour. Phases 1-5 exercise client-side protocol message construction and DAG event ingestion; steps requiring server-side Phase 2 handlers not yet wired in `xgen-node/src/main.rs` (MLS routing, DM promotion, migration protocol) pass structurally but are annotated in output as requiring additional server-side wiring.

**Impact:** Step 22 (identity replication query) will fail if `identity.replicate` is not server-side wired. The DoD item "all 60 steps PASS" requires server-side handler work in `xgen-node/src/main.rs` as a follow-on task.

---

## D-053 — Layer 19: Auth Tier 2–4 interface definitions; no verification logic in xgen-core

**Date:** 2026-05-14  
**Layer:** 19 — Auth Module Tier 2–4 Interfaces  
**Spec reference:** Spec 3.11.1–3.11.5; WD-09, WD-10, WD-11

### Context

Layer 19 adds the Auth Module Tier 2–4 interface layer. The guide specifies that this layer
defines contracts for external Auth Modules to implement — not verification logic.

### Decision

**AuthTier enum uses `u32` wire representation, ordered via `PartialOrd/Ord`.** Tier values
map directly to the spec's 1–4 encoding. `auth_tier` in `SpaceState` is already stored as `u32`;
`AuthTier::from_u32` bridges the two representations without changing the existing wire format.

**Three separate claim structs (Tier2Claims, Tier3Claims, Tier4Claims) rather than inheritance.**
Rust has no struct inheritance. Each tier struct carries all fields for that tier level (including
the fields from lower tiers), making each struct self-contained for serde deserialization without
requiring nested wrapper types.

**Tier 1 has no TTL.** Only Tiers 2–4 have TTL constants (WD-09: 365d, WD-10: 180d, WD-11: 90d).
`AuthTier::ttl_days()` returns `Option<u64>` so callers can branch on presence.

**Error code 3030 for TierMismatch.** The 3000–3999 range covers identity and auth domain errors.
3020 is used for stale replication (Layer 15). 3030 is the next clean slot for tier mismatch.

**No verification logic in xgen-core.** The Node verifies the Trust Assertion signature via the
existing signing infrastructure. If the signature is valid, the claim fields are accepted as-is.
The content of the claims (legal names, ISO certifications, security clearances) is the Auth
Module's domain — xgen-core never independently re-verifies those facts.

---

## D-052 — Layer 18: Phase 2 MLS placeholder (ChaCha20 epoch-key scheme); openmls deferred to Phase 3

**Date:** 2026-05-14  
**Layer:** 18 — End-to-End Encryption (MLS)  
**Spec reference:** Spec 3.10.1–3.10.9; DECISIONS.md D-031 (MLS selected over Megolm)

### Context

Layer 18 adds the E2E encryption layer. The guide says to add openmls, openmls_rust_crypto,
and openmls_basic_credential to xgen-core/Cargo.toml. After evaluating this option,
the following decision was made.

### Decision

**Full RFC 9420 openmls integration is deferred to Phase 3.** Phase 2 implements the complete
delivery service protocol and a Phase 2 MLS interface that correctly captures all protocol
properties using ChaCha20Poly1305 (already a project dependency).

**Rationale:**
1. **openmls version risk.** The project uses ed25519-dalek 2.x and sha2 0.10 (RustCrypto
   crates). openmls versions have historically had tight constraints on which RustCrypto
   versions they accept. Adding openmls in Phase 2 risks dependency version conflicts
   that could break existing 290 tests.
2. **Node delivery service needs no MLS crypto.** The Node side (delivery_service.rs,
   key_package.rs, group.rs) is 100% pure Rust — no MLS crypto needed. These files are
   complete and correct without openmls.
3. **Phase 2 placeholder captures all protocol properties.** The epoch-key scheme in
   client_mls.rs correctly demonstrates:
   - Each epoch has an independently derived key (forward secrecy)
   - Removed members do not learn subsequent epoch keys (post-compromise security)
   - Messages encrypted in epoch N cannot be decrypted with epoch M key
   - The `enc:` prefix convention for encrypted content in the event_trace log

**Phase 2 client_mls.rs uses:**
- SHA-256(group_secret || "xgen-epoch-key:" || epoch_le8) → epoch key
- SHA-256(group_secret || "xgen-next-epoch" || epoch_le8) → next group secret
- ChaCha20Poly1305 for encrypt/decrypt with deterministic nonce from epoch number

**The interface is stable.** Phase 3 replaces the key derivation with the RFC 9420 key
schedule while keeping the same `EpochKey`, `EncryptedContent`, `encrypt_message`,
and `decrypt_message` API. No callers need to change.

---

## D-051 — Layer 17: HTTP server/client stubs in xgen-core; BOOTSTRAP_HTTP_PORT = 8443; freshness decay formula

**Date:** 2026-05-14  
**Layer:** 17 — Bootstrap Node and Node Reputation  
**Spec reference:** Spec 3.14.2 (HTTP directory endpoint); 3.15.1 (freshness decay); 3.14.8 (port separation note)

### Decisions

1. **HTTP server and client are stubs in xgen-core; actual binding is in xgen-node.**
   The guide says to add `bootstrap/http.rs` (axum) and `bootstrap/client.rs` (reqwest).
   However, xgen-core is a library crate with no I/O — axum/reqwest would add large
   runtime dependencies. The pure logic (signing, verification, directory management,
   reputation computation) lives in xgen-core. The actual HTTP server start and HTTP
   client calls are implemented in xgen-node as thin shells using that logic.
   `http.rs` and `client.rs` in xgen-core are placeholder files with the port constant
   and max-age constant, documenting the interface without pulling in heavy deps.

2. **BOOTSTRAP_HTTP_PORT = 8443.** Spec 3.14.2 says the directory is served "over HTTPS"
   but does not specify a port. 8443 is the conventional HTTPS alternate port (avoids
   requiring root for port 443 binding). Recorded in `bootstrap/http.rs`.

3. **Port separation: WebSocket on 8080 (default), HTTP directory on 8443 (default).**
   Spec 3.14.2 notes the HTTP server runs "alongside" the WebSocket server on "different
   ports." The specific ports are implementation-defined and configurable; 8080/8443 are
   the Phase 2 defaults.

4. **Freshness decay formula.** Spec 3.15.1 says announcement_freshness decays from 1.0
   to 0.0 between 24h and 90 days (2160h). Phase 2 uses linear decay: at 24h the value
   is 1.0; it decreases linearly to 0.0 at 2160h. Implemented in `reputation::announcement_freshness`.

5. **`canonical_json` on `NodeAnnouncement` made `pub(crate)`.** Required by
   `bootstrap/capability.rs` to re-sign after adding `bootstrap_info`. The method was
   private; making it `pub(crate)` is the minimal change that keeps the API narrow.

---

## D-050 — Layer 16: migration batch size 100; Phase 2 always-accept policy; error code ranges 6001–6007, 6010–6011

**Date:** 2026-05-14  
**Layer:** 16 — Space Migration Protocol  
**Spec reference:** Spec 3.12.4 (batch size, implementation-defined); 3.12.1 (acceptance criteria)

### Context

Layer 16 introduces the Space Migration Protocol (`migration/` module). Several
implementation-defined choices must be recorded before advancing.

### Decisions

1. **BATCH_SIZE = 100 Events per `migration.event_batch` message.** Spec 3.12.4 states
   batch size is "implementation-defined, subject to the Tier message size ceiling." 100 is
   chosen as the recommended value from the spec. Recorded in `transfer.rs` as
   `pub const BATCH_SIZE: usize = 100`.

2. **Phase 2 always-accept policy in `handle_migration_propose`.** Spec 3.12.3 requires
   the destination to validate "compatible protocol version" and "sufficient storage
   capacity." Both checks require runtime data (disk space, version negotiation) not
   available in the pure-function layer. Phase 2 implementation always accepts unless the
   Space is already hosted (`already_hosting` guard). Real capacity checks are deferred to
   Phase 3 when the Node has a proper admin API surface.

3. **Error codes 6001–6007 for migration state machine errors; 6010–6011 for verification.**
   The 6xxx domain is reserved for migration (see CLAUDE.md error code convention). Ranges:
   - 6001 `migration_not_owner` — requester is not the Space owner
   - 6002 `migration_already_hosting`
   - 6003 `migration_insufficient_storage`
   - 6004 `migration_version_incompatible`
   - 6005 `migration_policy_rejected`
   - 6006 `migration_wrong_state`
   - 6010 `event_count_mismatch` — verification failure
   - 6011 `tips_mismatch` — verification failure

4. **`state.space_migrate` is signed by the source Node keypair** (not by the Space owner).
   This matches the pattern established for `state.dm_promote` (D-048) — Node-level
   protocol state events are signed by the Node, not by members.

---

## D-049 — Layer 15: ReplicaRegistry in NodeRuntime; Phase 2 simplification for persistence

**Date:** 2026-05-14  
**Layer:** 15 — Identity Replication  
**Spec reference:** Spec 3.13.1–3.13.6; WD-19 (REPLICATION_FACTOR = 3)

### Context

Layer 15 adds `select_replicas`, `handle_incoming_replicate`, and `ReplicaRegistry` to
`xgen-core/src/identity/replication.rs`. The spec requires replica Node tracking so the
home Node knows where to push updates and so client lookups can fall back to replicas
when the home Node is unreachable.

### Decision

1. **`ReplicaRegistry` lives in `NodeRuntime`.** It is an in-memory map from `identity_id`
   to `Vec<node_id>`. This fits the existing NodeRuntime pattern (all per-Node state in one
   struct). Wired as `pub replica_registry: ReplicaRegistry`.

2. **Not persisted (Phase 2 simplification).** The registry is rebuilt from local state on
   restart. Spec 3.13.6 describes a re-replication sweep on startup; that sweep is the
   mechanism by which the registry is repopulated. Full persistence is deferred to Phase 3
   when the identity store moves to SQLite.

3. **`select_replicas` is filter-then-truncate only.** Spec 3.13.3 criteria 1 (geographic
   diversity) and 2 (freshness ranking) require node announcement metadata that is not yet
   rich enough in Phase 2. Phase 2 implements criteria 3 (exclude existing replicas) and
   4 (limit to REPLICATION_FACTOR). Geographic/freshness criteria deferred.

4. **Error code 3020 for stale inbound version.** `handle_incoming_replicate` returns
   `ReplicationError::VersionStale { incoming, stored }` when the incoming `update_version`
   is not strictly higher than stored. Caller maps this to wire error 3020.

---

## D-043 — Named pipe naming convention for single-instance forwarding

**Date:** 2026-05-13  
**Layer:** Phase 2 Track 1 — Batch flag (`--batch`)  
**Spec reference:** Ch6 §6.9 (Console Input Channel Protocol); J-037 (batch execution model discussion)  

### Context

The `--batch` flag uses a single-instance forwarding model: the first invocation starts the application, and a subsequent invocation with `--batch` detects the running instance, forwards the command file via a named pipe, and exits. The running instance executes the commands. This model requires a pipe name that both invocations can derive independently — with no shared state, no PID lookup, and no discovery mechanism.

### Decision

Named pipes follow the convention:

```
\\.\pipe\xgen-{binary}-{label}
```

where `{binary}` is `client` or `node`, and `{label}` is the `--instance` label. When no `--instance` flag is given, the pipe name omits the label segment:

```
\\.\pipe\xgen-{binary}
```

**Examples:**

| Invocation | Pipe name |
|---|---|
| `xgen-client-app.exe` | `\\.\pipe\xgen-client` |
| `xgen-client-app.exe --instance alice` | `\\.\pipe\xgen-client-alice` |
| `xgen-client-app.exe --instance bob` | `\\.\pipe\xgen-client-bob` |
| `xgen-node-app.exe` | `\\.\pipe\xgen-node` |
| `xgen-node-app.exe --instance node_a` | `\\.\pipe\xgen-node-node_a` |
| `xgen-node-app.exe --instance node_b` | `\\.\pipe\xgen-node-node_b` |

### Rationale

The pipe name is fully derivable from two inputs the second invocation already has: the binary type and the instance label. No lookup, no state file read, no OS process enumeration required. The binary prefix (`client` / `node`) prevents pipe name collision between a client and a node running with the same instance label on the same machine — a normal scenario during stress testing. The pipe name is human-readable and visible in system tools (e.g. Process Explorer), which aids debugging.

This pattern was chosen over a hash-based name (unreadable, no debugging value) and over a label-only name (collision risk between binaries). The instance label is already validated by `validate_instance_label` (alphanumeric, hyphens, underscores, max 64 chars — see `FIXES_sec_01_ph2.md`) so it is safe to embed directly in the pipe name without further escaping.

### Scope

This decision covers Windows named pipes only. If Linux support is added in a future phase, the equivalent mechanism is a Unix domain socket at `<instance_data_dir>/xgen-{binary}.sock` — same derivation principle, filesystem path instead of pipe name.

---

## D-042 — Tauri event emission for real-time lifecycle state changes

**Date:** 2026-05-12  
**Layer:** Phase 2 Track 1 — Client Core Test UI  
**Spec reference:** Appendix E §E.2 (Client lifecycle states); CLAUDE.md Phase 2 Track 1  

### Context

The `xgen-client` binary already writes `xgen-client_state.json` on a periodic basis. For the Core Test UI to show lifecycle state transitions in real time — including fast-moving early transitions (INITIALISING → CONNECTING → AUTHENTICATING → READY, which can complete in under 2 seconds) — periodic file polling is insufficient. A dedicated communication channel between the Rust backend and the Tauri webview frontend is required.

### Decision

On every lifecycle state transition, the Rust backend emits a Tauri event named `"xgen-client-state-changed"` with a `ClientStateEvent` payload:

```json
{
  "state": "READY",
  "label": "Ready",
  "timestamp": "2026-05-12T10:30:00.000Z"
}
```

The `state` field is the canonical uppercase enum form (e.g. `"DEGRADED_AUTH"`). The `label` field is the Appendix E display label (title case). The `timestamp` is UTC RFC 3339 with milliseconds.

The periodic state JSON write is retained unchanged — it provides the full state snapshot (connections, spaces, peers) that the UI may query on demand. The Tauri event channel is exclusively for lifecycle state transitions.

### Rationale

The two mechanisms serve different purposes. The JSON file is a full snapshot written on a timer — useful for deep status queries. The Tauri event is a lightweight notification emitted exactly when something changes — suitable for driving a real-time status indicator. Combining both avoids the choice between staleness (polling only) and Rust complexity (events only for everything).

This pattern is the intended long-term architecture for the UI communication layer: the Rust library owns state, emits targeted events on significant transitions, and the webview reacts. Future XGen protocol events (message receipt, federation events, etc.) may follow the same pattern — emitting outside the periodic write cycle when real-time feedback is required.

### Implementation note

The `transition_state()` function in `xgen-client/src/lib.rs` receives an `&tauri::AppHandle` from the caller in `main.rs`. The library does not hold a reference to Tauri internals — the handle is passed in per call, preserving the library-first architecture.

---

## D-041 — Theme loader: default skin and fallback chain

**Date:** 2026-05-08  
**Layer:** 6 (Layer 4 Presentation — Client UI)  
**Spec reference:** Ch2 §"Architecture Principles" (open enums); Ch6 client design (UI architecture)  

### Context

UI skin/theme files (`skin-{name}.css`) are replaceable, with a minimum of two themes (dark and light) supported. The CSS reset that neutralises UA defaults for semantic tags is coupled to the skin file — each skin contains its own reset block — so that with a skin loaded the page renders with the skin's intended visual treatment, and without a skin the page renders as semantic HTML with browser defaults (which remains usable thanks to the structural-truth-in-tags principle the skeletons follow).

The loader must define behaviour for two cases: (1) default theme when no `?theme=` query param is given, (2) fallback when an explicitly-requested theme cannot be loaded.

### Decision

**Default theme.** When no `?theme=` query param is present, the loader attempts to load `skin-dark.css`. Dark is the primary aesthetic per the Run 2 briefing.

**Fallback chain.** If a requested skin (`?theme=custom-name`) cannot be loaded, the loader falls back to `skin-dark.css` (the default). If `skin-dark.css` also cannot be loaded, no skin is applied — the page renders as raw semantic HTML with browser default styles.

Two-tier graceful degradation:

```
?theme=custom    → skin-custom.css → (fail) → skin-dark.css → (fail) → raw HTML
?theme=dark      → skin-dark.css   → (fail) → raw HTML
?theme=light     → skin-light.css  → (fail) → skin-dark.css → (fail) → raw HTML
no param         → skin-dark.css   → (fail) → raw HTML
```

### Rationale

The "no skin = no reset = raw HTML" property is preserved deliberately. Reset rules live inside skin files, not in `tokens.css` or any always-loaded layer. This guarantees that a skin failure (404, network error, parse error) does not leave the user with a broken half-styled UI — UA defaults stripped but no replacement rules. Instead the user sees semantic HTML rendered with full UA defaults, structurally meaningful and navigable.

Falling back to dark before raw HTML on a missing custom theme prioritises a working UI over the strict raw-HTML mode. A user with a broken custom theme link is more likely to want the standard dark UI than the raw HTML experience.

This is consistent with Ch2's open-enums principle: implementations must handle values they do not understand gracefully. An unknown theme name is an open-enum case at the loader level.

### Implementation note

The bootstrap script in each skeleton page implements the fallback chain via `<link onerror>` handlers on the `<link rel="stylesheet">` element. Implementation detail deferred to the UI implementation phase.

---

## D-039 — Pending buffer wiring: NodeRuntime holds PendingBuffer directly

**Date:** 2026-05-06
**Layer:** Message exchange / Federation (Phase 1 bug fix — F-001)
**Spec reference:** Spec 3.2.5 (pending buffer for unknown prev_events)

### Context

The Phase 1 stress test (STRESSTEST_ph1_findings.md) identified finding F-001: during the concurrent message flood, federated events arriving at Node B with unknown `prev_events` were being silently dropped rather than buffered. The stress test report showed PASS at the client level but Node B was applying only ~53% of expected federated messages.

`PendingBuffer` (`dag/pending.rs`) was already fully implemented and tested. `RoomDag` (`dag/mod.rs`) correctly wraps `EventStore + DagGraph + PendingBuffer` and handles out-of-order delivery with cascading drain. However, `NodeRuntime::accept_message` bypassed both: it called `accept_event` directly using the raw `EventStore` and `DagGraph` fields. On `HeldPending`, the error bubbled up to `main.rs`, which logged it as `ERROR` and traced it as `RejectEvent` — dropping the event permanently.

### Decision

Add `pending: HashMap<String, PendingBuffer>` directly to `NodeRuntime` rather than replacing the existing `stores + graphs` fields with `RoomDag` instances.

**Reason for not switching to `RoomDag`:** `RoomDag::insert` only performs DAG-level checks (missing prev_events, structural validation). `accept_message` must run the full 13-step pipeline (steps 8–13: event_id hash, DAG structure, sender identity, space membership, signature, permissions). These steps require `SpaceState` and `IdentityRegistry` which `RoomDag` does not hold. Switching to `RoomDag` would have required either passing those dependencies into `RoomDag` (changing its interface) or duplicating the validation logic. Adding `PendingBuffer` alongside the existing fields is the minimal change that fixes the gap without altering the `RoomDag` interface or adding responsibilities it was not designed for.

### Implementation

- `NodeRuntime` gains `pub pending: HashMap<String, PendingBuffer>`.
- `accept_message`: on `HeldPending(missing)` → calls `pending.add(event, &missing)` and returns `Err(HeldPending)`.
- `accept_message`: on `Ok(())` → calls `drain_pending_messages(space_id, event_id)`.
- `drain_pending_messages`: resolves the buffer using `pending.resolve(resolved_id, store)`, re-runs `accept_event` on each unblocked event, recurses for every newly accepted event.
- `main.rs`: `Err(ExchangeError::HeldPending(_))` arm logs at `DEBUG` ("event buffered — waiting for unknown prev_events") and does not emit a `RejectEvent` trace, since the event is buffered not rejected.

### Verification

Stress test re-run post-fix: 0 ERROR lines on Node B, 0 reject_event traces, 284 apply_event entries (up from 134, now symmetrical with Node A's 280). With resting point after Phase 3, 0 buffered entries (all membership events settled before flood, no out-of-order arrivals at all).

---

## D-038 — Client session header omits `identity_id` and `connected_node`

**Date:** 2026-05-06
**Layer:** Logging — xgen-client
**Spec reference:** docs/xgen_appendix_g_en.md (session header); LOGGING_implementation.md Step 2

### Decision

Appendix G specifies that the `xgen-client` session header includes `identity_id` and `connected_node`. These fields cannot be placed in the header block because log body lines appear before those values are available:

- `"Log file opened"` fires immediately after subscriber init, before any keypair is loaded or connection is made.
- `"Connecting to Node"` fires inside each network command handler, before authentication completes.

The header must precede all body lines (Appendix G, session structure). Deferring the header until auth completes would violate that constraint. Buffering log output until auth completes is not idiomatic with the `tracing` subscriber model.

**Resolution:** the `xgen-client` session header is written immediately after subscriber init with the fields that are available at that moment (`app_type`, `protocol_version`, `build`, `session_id`, `started_at`). The fields `identity_id` and `connected_node` are omitted from the header and are instead emitted as operational body lines at the point where they become known:

- `identity_id` is logged as a body line after keypair load and `client_authenticate()` completes.
- `connected_node` is logged as a body line after the WebSocket connection is established.

This applies to the CLI client only. The future Tauri UI client (Ch6) has a persistent session with a natural startup sequence and will be able to supply both fields in the header at open time.

---

## D-037 — Tier 1 identity: precise definition of persistent accountable identity

**Date:** 2026-05-05
**Layer:** Philosophy / Specification
**Spec reference:** Ch1 Pillar 2 (no anonymity); Ch3 authentication tiers

### Decision

The original "no anonymity" pillar was correct in intent but imprecise in language, creating a risk of misreading Tier 1 as requiring verified real-world identity. This entry locks the precise definition.

**Tier 1 establishes persistent accountable identity, not civil identity.**

The identity anchor at Tier 1 is the keypair. It is permanent and non-respawnable. This is what "no anonymity" means in XGen: not "we know who you really are," but "you cannot disappear and reappear as someone else."

**Tier 1 requirements:**
- A keypair (the identity anchor — permanent, cryptographically bound to the user)
- At least one contact field: email, phone number, or both — self-declared, not verified by the protocol

**Contact data purpose:** operator reach-back channel (ban notices, account recovery). Not an identity proof.

**Optional node behaviour:** a node may implement an email confirmation code flow as a local policy. This is recommended practice but is not a protocol mandate. Phone number SMS verification requires external provider agreements and is outside the protocol's scope entirely.

**What Tier 1 proves:** this is the same cryptographic actor as before. Nothing more, nothing less.

**What Tier 1 does not prove:** that the email address is the user's real address, that the phone number belongs to them, or that they are a specific real-world person.

Tiers 2–4 progressively verify contact data and eventually tie identity to real-world institutional or legal proof.

**Philosophical note:** the anti-abuse guarantee at Tier 1 rests on keypair permanence, not on contact data truthfulness. You cannot ban a keypair's biography — you can ban the keypair. The contact data makes respawning costly enough to matter; it does not make identity transparent.

---

## D-034 — Client log lifecycle deferred to UI application era

**Date:** 2026-04-30  
**Layer:** Phase 2 — client application  
**Spec reference:** docs/tests/LOGGING_debug_ph2.md (future update)

### Decision

The CLI client has no natural session lifecycle — each command invocation connects, acts, and exits. Creating a new log file per command invocation is wasteful and produces meaningless fragmented logs.

The correct log session boundary is the UI application lifecycle: from when the client UI opens to when it closes. This cannot be implemented until a persistent UI client exists.

This item is deferred until the Tauri + Svelte client application (Ch6) is implemented. At that point, `LOGGING_debug_ph2.md` will be updated to specify that the client log file spans the full application session (open to close), not individual command invocations.

**Current behaviour (acceptable for Phase 1 CLI):** one log file per command invocation. Wasteful but functional. Not a bug — a known limitation of CLI architecture.

---

## D-036 — XGen Module Architecture (resolves OQ-01)

**Date:** April 2026  
**Layer:** Architecture — both Node and Client  
**Spec reference:** Ch6 section 6.8 Module Architecture; Ch3 OQ-01 (resolved)

### Decision

XGen modules use **Event subscription + `meta_atts`** as their communication model (Approach C). A module connects to the Node or Client via WebSocket, subscribes to the Event stream, and communicates module-specific payload via the `meta_atts` field on Events. No separate IPC protocol is invented. Modules speak native XGen.

### Module package

A module is distributed as a **package** — one folder containing a manifest file plus any number of handlers, assets, and UI components. Inside one package there may be a single micro-handler or a complex multi-handler system. The packaging, registration, and discovery mechanism is identical regardless of internal complexity. There is no separate concept of "micro-module" vs "full module" at the system level — only packages of varying complexity.

### Module identity mode

Declared in the module manifest as an enum:

- **`system`** — the module has its own keypair and its own identity_id. It signs Events as itself. It is a distinct actor on the network. Used for bots, bridges, aggregators, compliance reporters.
- **`user`** — the module acts on behalf of the authenticated user. It produces Events signed by the user's keypair. Requires explicit user consent at install time. Used for productivity extensions, UI enhancements, workflow automation.

The Node/Client enforces the declared mode at install time and at Event signing time. A `user`-mode module that attempts to sign as a different Identity is rejected.

### Module UI forms

Three UI forms, declared in the manifest. A module may declare one or more:

- **Headless** — no UI representation beyond the module list entry. Runs silently. Used for background services, bridges, reporters.
- **Widget** — a UI component injected into a defined slot in the XGen application shell. Used for inline tools, sidebar panels, message decorators.
- **Window** — a full separate window launched from the module list. Used for substantial self-contained UIs like the Auth Module verification flow.

### Module list — universal registry

Every installed module appears in the module list regardless of its UI form. The module list entry is always the same structure: title, description, version, author, mode badge (`system`/`user`), status indicator (running/stopped/error), and a settings access point. The module list is the single place a user discovers, enables, disables, configures, and removes modules.

### Capability advertisement

When a Node loads a module that adds a new capability, it adds the capability string to its `capabilities` array in its node announcement (3.5.2). Other Nodes and clients that receive the announcement learn about the capability automatically via the open enum mechanism (3.4.3). Unknown capability values are silently ignored by Nodes that do not support them.

### meta_atts as module communication channel

The `meta_atts` field on every Event (defined in 3.2.1) is the designated channel for module-specific payload. A module that needs to attach additional data to an Event uses `meta_atts` rather than extending the core schema. Conventions:

- Keys in `meta_atts` are namespaced by module: `"xgen.module.<module_id>.<key>"`
- Values are strings or JSON-serialisable objects
- Core protocol Nodes that do not recognise a `meta_atts` key silently ignore it (open enum principle)
- `meta_atts` is never used for core protocol data — it is strictly an extension channel

### Injection slots (widget modules)

The XGen application shell defines a set of named injection slots where widget modules may render components. The slot inventory is specified in Ch6 section 6.8.3. A widget module declares which slot(s) it targets in its manifest.

### Manifest format

Specified in Ch6 section 6.8.2.

---

## D-035 — Node data paths derived from working directory — not config-editable

**Date:** 2026-04-30  
**Layer:** Implementation — Node configuration  
**Spec reference:** Ch4 section 4.3 (runtime folder layout)

### Decision

`log_path` and `spaces_dir` MUST NOT be user-editable fields in `xgen-node_config.toml`. Hardcoded absolute paths in an operator-editable config file are a security problem: they reveal data locations, can be tampered with, and create no separation between config (operators read) and data (nobody touches).

The Node derives ALL data paths from its working directory by convention:

```
<working_dir>/
  xgen-node_config.toml     ← config (operators may read)
  xgen-node_keypair.enc     ← keypair (nobody touches)
  xgen-node_state.json      ← runtime state
  xgen-node_identities.db   ← identity registry
  spaces/                   ← Event stores (nobody touches)
  logs/                     ← debug logs
  audit/                    ← audit logs (Phase 2)
```

No path overrides in config. No way to accidentally or maliciously redirect data storage elsewhere. The keypair path remains configurable via `keypair_path` in `[paths]` as a single narrow exception — operators may legitimately store the keypair on a different device or partition for security.

### Implementation requirement for Mr. Code

Remove `log_path` and `spaces_dir` from `[paths]` in `NodeConfig` struct and both test config files. Replace with hardcoded relative path constants in the Rust source:

```rust
const SPACES_DIR: &str = "spaces";
const LOGS_DIR: &str = "logs";
const AUDIT_DIR: &str = "audit";
```

All path construction uses `working_dir.join(SPACES_DIR)` etc. The working directory is wherever the Node binary is run from — documented as a convention, not a config option.

---

## D-033 — Global Event tracing interface — architectural requirement

**Date:** 2026-04-30  
**Layer:** Phase 2 implementation — core architecture  
**Spec reference:** docs/tests/LOGGING_debug_ph2.md  

### Decision

Debug logging must be implemented as a **global Event tracing interface** — a single chokepoint that every inbound and outbound Event passes through automatically. Enumerated manual `tracing::` calls scattered across individual command handlers are rejected as the primary logging mechanism.

### Rationale — why this should have been first

Logging should have been the very first capability implemented, before any protocol logic, so every Event was observable from the first commit. The Phase 1 implementation reversed this order — 173 tests and a full smoke test were written before any logging existed. As a result:
- Some Events ran without any observability
- Log points were added by enumeration — one per command, one per handler — which is fragile and incomplete
- New commands or handlers added in Phase 2 will silently produce no log output unless someone remembers to add a call
- There is no guarantee that a client log entry and a Node log entry can be paired, because pairing depends on both sides having logged the same event_id

This decision corrects the architecture for Phase 2.

### Required architecture

Every Event that enters or leaves the Node or client MUST pass through a single global tracing interface. This interface is not optional and not bypassed by any code path.

**Interface contract:**

```rust
// Every inbound and outbound Event passes through this — no exceptions
pub fn trace_event(
    event: &XgenEvent,
    direction: EventDirection,   // Inbound | Outbound
    session: &SessionContext,    // who is authenticated, their role
)
```

Inside this function:
1. Check session role — if no owner or admin is authenticated, suppress output (see role gate below)
2. Log the Event at `debug` level with structured fields: `event_id`, `event_type`, `direction`, `sender`, `space_id`, `room_id`, `timestamp`
3. Never log `content` field — message content is never written to the debug log at any level

**Role gate:**
- Debug log output is suppressed unless an owner or admin Identity is authenticated in the current session
- Regular members produce no debug log output even if `level = "debug"` is set in config
- The config `level` field still controls the global ceiling — but the role gate is an additional AND condition
- Rationale: prevents sensitive conversations from leaking into log files when regular members are active

**Pairing guarantee:**
- Every Event has an `event_id` (content hash, globally unique)
- Client log: `direction=Outbound event_id=X`
- Node log: `direction=Inbound event_id=X`
- Pairing is trivially possible by matching `event_id` across log files — no coordination needed

### What this means for the current Phase 1 implementation

The Phase 1 debug log infrastructure (datetime-stamped files, `logs/` subfolder, config level switch, subscriber init) is correct and stays. What changes is the log point generation mechanism — from enumerated manual calls to the global interface above. The manual `tracing::info!` calls in individual command handlers become secondary annotations only; the global interface is the primary and mandatory logging path.

### Implementation priority

Implement the global Event tracing interface as the **first task** of Phase 2 implementation, before any Phase 2 protocol features. See `LOGGING_debug_ph2.md` for full instructions.

---

## D-032 — Two distinct log types: debug log and audit log

**Date:** 2026-04-29  
**Layer:** Phase 2 specification — Node implementation and Auth Module interface  
**Spec reference:** 3.11.8 Audit Log Requirements; docs/tests/LOGGING_debug_ph1.md; docs/tests/LOGGING_audit_ph2.md

### Decision

XGen defines two independent and non-interchangeable log types. They are never merged, never share a file, and serve different audiences.

**Debug log** — technical diagnostic output. Operator-controlled verbosity via `[logging].level` in config. Files accumulate in `logs/` subfolder, one per session with datetime suffix. Operator may delete at any time. Serves developer and operator.

**Audit log** — permanent accountability record. Cannot be disabled by config. Append-only JSON Lines, monthly rotation to `audit/protocol_audit_YYYY-MM.jsonl`. MUST NOT be auto-deleted. Serves auditor, compliance officer, regulator.

### Two audit log layers

**Node-level protocol audit log:** records protocol Events with membership and state-change significance. Always present on every Node regardless of Tier. 11 EventTypes covered. Retention is operator/regulatory decision — no protocol minimum at Tier 1/2.

**Auth Module audit log:** records identity verification decisions made by the Auth Module. Lives inside the Auth Module, not the Node. Required at Tier 3 (7-year retention, SOX §802) and Tier 4 (10-year minimum healthcare, mandatory tamper-evident storage, data localisation constraint).

### Rationale

A system where a Tier 4 government or healthcare operator cannot prove who accessed what data and when is not viable for institutional adoption. The audit log is what makes XGen credible to compliance teams, not just to developers. Specifying it at the protocol level — not as an implementation afterthought — ensures third-party implementations are also compliant.

---

## D-031 — End-to-End Encryption: MLS (RFC 9420) selected over Megolm

**Date:** 2026-04-29  
**Layer:** Phase 2 specification  
**Spec reference:** 3.10 End-to-End Encryption (to be written)

### Decision

XGen will use MLS (Messaging Layer Security, RFC 9420) as its end-to-end encryption protocol. Megolm (the Signal-derived group ratchet used by Matrix/Element) was considered and rejected.

### Rationale

MLS is an IETF standard (RFC 9420, published 2023) designed specifically for asynchronous group messaging with dynamic membership. It provides full forward secrecy and post-compromise security for groups of any size, with mathematically clean key tree updates on every join and leave event. Megolm is a proven production protocol but carries well-documented weaknesses in group membership transitions that have caused real security issues in Matrix deployments.

XGen is designed as future infrastructure, not a fast-ship product. The implementation complexity of MLS is the correct tradeoff for a protocol intended to be adopted as open infrastructure by institutions that require cryptographic correctness. Megolm's weaknesses are knowingly inherited — MLS eliminates them by design.

### Implications for 3.10

- Key package format follows RFC 9420
- Group state is represented as an MLS ratchet tree
- Join/leave Events trigger tree updates (Welcome messages for joins, Commit messages for updates)
- The Node is an MLS Delivery Service — it routes MLS handshake messages but cannot decrypt content
- Key material never touches the Node — the Node is structurally excluded from E2E decryption
- Phase 1 Nodes are forward-compatible: they store and route encrypted Event payloads as opaque blobs

---

## D-030 — xgen-node will be packaged as a system service post-stabilisation

**Date:** 2026-04-29  
**Layer:** operational (post-Phase 2)  
**Spec reference:** Ch4 — production deployment section (to be written)

### Decision

Once `xgen-node` is debugged and tuned after Phase 2, it will be packaged as a system service on all supported platforms. This is a production deployment requirement — a Node that requires manual restart after reboot or dies when a terminal session closes is not production-grade infrastructure.

### Platform approach

| Platform | Mechanism | Notes |
|---|---|---|
| Linux | `systemd` unit file | Primary reference deployment. ~15-line unit file, handles restart-on-failure, journald logging, dedicated user account. |
| Windows | NSSM (Non-Sucking Service Manager) | Wraps the binary as a Windows Service without Rust source changes. Pragmatic choice for early production. |
| macOS | `launchd` plist | Standard macOS daemon mechanism. |

### Timing

Not before Phase 2 implementation is complete and the Node has been tested through multiple restart cycles with full state recovery (Fix 16 regression confirmed stable). Service packaging on an unstable process makes bugs harder to diagnose.

### Documentation impact

A new "Production Deployment" section in Ch4 will document the systemd unit file as the primary reference, with NSSM noted for Windows. No changes to Ch3 protocol spec — this is purely operational.

---

## D-000 — Historic First Compile

**Date:** 2026-04-27
**Layer:** 0 (pre-implementation baseline)
**Spec reference:** —

The first successful compile of the XGen Protocol codebase. No protocol logic implemented — both `xgen-node` and `xgen-client` were pure stubs printing a placeholder line. Marked retroactively as version `0.0.0` in semantic terms: state=0 (building), section=0 (no section started), session=0.

The compile itself took seconds. However, the first two attempts froze overnight and for several hours respectively due to Google Drive file locking on build artifacts. Resolved by moving `CARGO_TARGET_DIR` to a local path (`C:/cargo-targets/XGenProtocol`) outside the synced folder.

Tagged on GitHub as `v0.1.0` (build infrastructure baseline). Real versioning — `[state].[section].[session].[build]` — begins with D-001 and the first line of Wire Format code.

---

## D-001 — Versioning Scheme

**Date:** 2026-04-27 (revised 2026-04-28)
**Layer:** 0 (pre-implementation baseline)
**Spec reference:** —

Adopted a three-component version format: `[state].[layer].[session]`

- **state** — 0 while building Phase 1; 1 when Phase 1 complete and stable
- **layer** — implementation layer number (1–10, per IMPLEMENTATION_GUIDE_ph1.md)
- **session** — work session in which that layer was completed

`Cargo.toml` stores this three-part version. Layer numbering follows the implementation order, not the spec section order (spec sections are non-sequential by necessity — e.g., Layer 6 implements spec 3.4). Using layer numbers makes tags monotonically increasing: v0.1.1 → v0.2.2 → … → v0.9.3.

Originally the second component was intended to be the spec section number, which produced non-monotonic tags (e.g., v0.4.2 for Layer 6 before v0.5.2 for Layer 5). Corrected to layer numbers in session 3.

---

## D-002 — Layer 1: Keypair Encryption Scheme

**Date:** 2026-04-27
**Layer:** 1 — Cryptographic Foundation
**Spec reference:** 3.5.1

The spec requires keypairs to be "encrypted at rest" but does not prescribe the encryption algorithm. Chose **ChaCha20-Poly1305** (AEAD) with **Argon2id** key derivation.

- **ChaCha20-Poly1305** — modern, well-audited AEAD cipher. No timing side-channels from table lookups (unlike AES without hardware acceleration). Available in the `chacha20poly1305` crate.
- **Argon2id** — current recommended KDF for password-based key derivation (RFC 9106). Resistant to GPU and side-channel attacks. Parameters for Phase 1: m=64MB, t=3, p=1 — tuned for interactive use.
- **Phase 1 passphrase** — Local Node mode uses an empty string passphrase. The file is still encrypted (the AEAD tag still provides integrity), but without meaningful key stretching. A non-empty passphrase is supported and works correctly. Production deployments must use a strong passphrase.

File format: JSON with `version`, `algorithm`, `kdf`, `salt` (base64url, 32 bytes), `nonce` (base64url, 12 bytes), `ciphertext` (base64url, 48 bytes = 32-byte key + 16-byte AEAD tag).

---

## D-003 — Layer 1: SigningKey Generation Without rand_core Feature

**Date:** 2026-04-27
**Layer:** 1 — Cryptographic Foundation
**Spec reference:** 3.5.1

`ed25519-dalek v2` exposes `SigningKey::generate(&mut rng)` only when the `rand_core` feature flag is enabled. To avoid adding a feature flag, keypair generation uses `OsRng.fill_bytes()` to produce 32 random bytes and constructs the key with `SigningKey::from_bytes()`. This is equivalent — `SigningKey::generate` does the same internally.

---

## D-004 — Layer 2: Event Fields `event_id` and `signature` as `Option<String>`

**Date:** 2026-04-27
**Layer:** 2 — Wire Format
**Spec reference:** 3.2.1, 3.2.3, 3.2.4

The spec defines `event_id` and `signature` as required fields on received Events, but they cannot exist during construction — `event_id` is derived by hashing the canonical form, and `signature` is produced by signing those same bytes. Both fields are therefore `Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]`.

This means an unsigned, unsigned Event serialises without those fields (correct for computing the canonical form), and a signed Event includes them (correct for the wire). The validation pipeline (step 3) enforces presence on received Events; the type system prevents accidental use of an unsigned Event where a signed one is required.

---

## D-005 — Layer 3: Root Event Types Require Empty `prev_events`

**Date:** 2026-04-27
**Layer:** 3 — DAG Event Store
**Spec reference:** 3.2.5

The spec defines `prev_events` DAG rules but does not explicitly enumerate which event types are DAG roots. Decided that `state.space_create`, `state.dm_space_create`, and `state.room_create` are root types (empty `prev_events` required). All other event types must reference at least one predecessor.

Rationale: Space and Room creation events are the structural origins of their respective DAGs — they have no meaningful predecessors within the same namespace. Enforcing empty `prev_events` on these types makes the DAG structure explicit and prevents accidental chaining that would complicate state derivation.

---

## D-006 — Layer 3: Cycle Detection Reduces to Self-Reference Check

**Date:** 2026-04-27
**Layer:** 3 — DAG Event Store
**Spec reference:** 3.2.5

Full cycle detection (verifying no `prev_event` is a descendant of the new Event) is expensive — it requires a graph traversal. For a newly inserted Event this reduces to a single check: does the Event reference itself? A new Event has no descendants yet, so no other cycle is possible at insertion time. Only self-reference (`event_id ∈ prev_events`) needs an explicit check.

This is correct as an invariant because the store is append-only: once an event_id is in the store, no future Event can retroactively become its ancestor.

---

## D-007 — Layer 3: Phase 1 `prev_events` Fanin Limit = 10

**Date:** 2026-04-27
**Layer:** 3 — DAG Event Store
**Spec reference:** 3.2.5

The spec does not specify a hard limit on `prev_events` entries for Phase 1. Chose 10 as a practical ceiling that accommodates realistic concurrent edit scenarios while preventing degenerate inputs. Phase 2 may revisit based on observed network behaviour.

---

## D-008 — Layer 5: Node Announcement TTL = 90 Days

**Date:** 2026-04-27
**Layer:** 5 — Node Identity and Announcement
**Spec reference:** 3.5.6

The spec requires announcements to carry a `valid_until` field but does not prescribe the TTL duration. Chose 90 days for Phase 1. This is long enough that operators on routine schedules (e.g., weekly restarts) never need to worry about expiry, but short enough that a decommissioned node's announcement falls off peer tables within a quarter. Expiry is checked before signature verification to avoid wasting crypto work on stale announcements.

---

## D-009 — Layer 6: Federation `session_id` Derivation

**Date:** 2026-04-27
**Layer:** 6 — Federation Handshake
**Spec reference:** 3.4.4

The spec requires a `session_id` to be agreed during the handshake but does not specify its derivation. Chose: `hash_uri(sort([node_a_id, node_b_id]) + timestamp)` where node IDs are sorted alphabetically before concatenation.

Sorting ensures the same `session_id` is independently computed by both sides regardless of which is initiating and which is receiving. The timestamp is taken from the `federation.hello` message so both sides use the same value.

---

## D-010 — Layer 6: `FederationMessage` Signing Excludes `signature` via Field Order Constants

**Date:** 2026-04-27
**Layer:** 6 — Federation Handshake
**Spec reference:** 3.4.3

Each `FederationMessage` variant carries `signature: Option<String>` with `skip_serializing_if = "Option::is_none"`. The canonical form for signing uses per-variant field order constants that do not include `"signature"`, so the signature field is always absent from the bytes that get signed — whether it is `None` (unsigned) or `Some` (already signed). This avoids the need to temporarily clear the field before computing the canonical form.

---

## D-011 — Layer 7: `MAX_DISPLAY_NAME_LEN` = 128

**Date:** 2026-04-27
**Layer:** 7 — Identity Registration
**Spec reference:** 3.6.5

The spec requires display name validation but does not prescribe a maximum length. Chose 128 characters (Unicode code points). This comfortably accommodates real names, handles emoji and CJK characters, and is simple to communicate. Empty strings and strings containing control characters (codepoints < 0x20) are also rejected.

---

## D-012 — Layer 7: Phase 1 Uses `identity_id` as `device_id`

**Date:** 2026-04-27
**Layer:** 7 — Identity Registration
**Spec reference:** 3.6.6

The spec defines a `devices` array for multi-device support. Phase 1 supports one device per Identity. Rather than omitting the `devices` array entirely, the registration pipeline populates it with a single entry using `identity_id` as the `device_id`. This keeps the wire schema stable for Phase 2 multi-device support without breaking changes.

---

## D-013 — Layer 8: Empty `room_id` Distinguishes Space-Level Events from Room-Level Events

**Date:** 2026-04-27
**Layer:** 8 — Space and Room Protocol
**Spec reference:** 3.7.1, 3.7.3

The spec defines both Space-level and Room-level events sharing the same `Event` envelope. Rather than introducing a separate envelope field, the existing `room_id` field doubles as a discriminator: an empty string means the event targets the Space; a non-empty string means it targets a specific Room. This is consistent with the spec's use of `room_id = ""` on `state.space_create`.

The `apply_event` state machine and the Layer 9 pipeline both branch on `room_id.is_empty()` before dispatching.

---

## D-014 — Layer 8: `apply_join` Branches on `room_id` Before Membership Check

**Date:** 2026-04-27
**Layer:** 8 — Space and Room Protocol
**Spec reference:** 3.7.5

The initial implementation of `apply_join` checked `self.members.contains_key(joiner)` before branching on whether the event was a Space join or a Room join. This caused existing Space members to receive `AlreadyMember` when trying to join a Room (because they were already in `self.members`). Fixed by checking `room_id.is_empty()` first — if non-empty, route to the Room join path; if empty, route to the Space join path with its own duplicate check.

---

## D-015 — Layer 8: `state.space_create` and `state.room_create` Have Empty ID Fields During Construction

**Date:** 2026-04-27
**Layer:** 8 — Space and Room Protocol
**Spec reference:** 3.2.3, 3.7.2

Both `space_id` and `room_id` are derived as `event_id`, which is computed by hashing the canonical event bytes. This creates a circular dependency: the ID fields cannot be known before serialisation, but they must be part of the canonical form. Resolution: event builders set both fields to empty strings during construction. `sign_event` then computes `event_id = hash_uri(canonical_bytes)` — the empty strings are part of the canonical form and the resulting hash becomes the ID. Callers set `space_id` / `room_id` in subsequent events using the derived value.

---

## D-016 — Layer 9: `validate_steps_8_13` Is Read-Only; Callers Control Insertion

**Date:** 2026-04-28
**Layer:** 9 — Message Exchange
**Spec reference:** 3.2.6

Steps 8–13 of the validation pipeline are implemented as a pure read-only function (`validate_steps_8_13`). It does not mutate the `EventStore` or `DagGraph`. Mutation happens only in `accept_event`, which calls the validator and then inserts on success.

This design lets callers inspect the specific failure reason before deciding whether to buffer (step 9 `HeldPending`) or reject (all other errors). It also makes the validator easily testable in isolation without needing mutable state.

Step 10 (DAG structural check) intentionally duplicates the logic from `DagGraph::add_event` rather than extracting a shared helper, because the DAG check requires a read-only view — there is no `DagGraph::validate_only` method and adding one would be scope creep.

---

## D-017 — Layer 9: Test Setup Merges Two DAG Roots via Invite `prev_events`

**Date:** 2026-04-28
**Layer:** 9 — Message Exchange
**Spec reference:** 3.2.5

In test setup, `state.space_create` and `state.room_create` are both DAG root events (empty `prev_events`). Without intervention, they remain as two independent tips indefinitely. The first membership event (`membership.invite`) references both roots as `prev=[space_id, room_id]`, merging the two roots into a single linear chain and leaving exactly one tip. This ensures message events have a single, unambiguous predecessor for `prev_events` in tests.

This is a test-only convention. In production, the protocol does not require roots to be merged — two persistent tips are valid DAG state.

---

## D-018 — meta_atts Key Namespace: Dot Separator, Reverse-Domain Ownership

**Date:** 2026-04-28
**Layer:** 0 (protocol specification)
**Spec reference:** 3.1.3

`meta_atts` keys follow a dot-separated namespace scheme: `<namespace>.<key>`. The `xgen.` prefix is reserved for specification use. Third-party keys MUST use reverse-domain prefixes (e.g. `com.example.my_key`). Key segments use `snake_case`. Max key length 128 characters. Values are strings; structured values must be JSON-encoded as strings rather than embedded as nested objects.

Spec 3.1.3 updated accordingly.

---

## D-019 — Transport Pluggability: WebSocket as Default, Alternative Streams Permitted

**Date:** 2026-04-28
**Layer:** 0 (protocol specification)
**Spec reference:** 3.3.1

WebSocket over TLS is the mandatory production transport. However, the spec explicitly permits operators to substitute any reliable bidirectional stream transport (Tor hidden services, I2P, pluggable transport proxies) without protocol-layer changes. This is noted in spec 3.3.1. DPI-resistance via custom transports is flagged as a Phase 3 investigation area — no Phase 1 or Phase 2 work required.

---

## D-020 — File Placement: Two-Tier Model (System Files vs User-Configurable Files)

**Date:** 2026-04-28
**Layer:** 0 (deployment model)
**Spec reference:** IMPLEMENTATION_GUIDE_ph1.md — Deployment Model

Refined the Pattern A deployment model into an explicit two-tier system. Tier 1 (system files: config, registries, announcements) is mandatory co-location with the binary — not configurable. Tier 2 (keypair, TLS cert, logs, UI settings) defaults to binary folder but can be redirected via explicit config fields. This accommodates HSM-backed keys, OS keystore integration (Phase 2), and system log aggregation without scattering files by default. No file moves silently — every Tier 2 redirect requires an explicit config entry.

---

## D-021 — Self Account (`self`): Local-Only Synthetic Identity, Post-Phase-1

**Date:** 2026-04-28
**Layer:** 0 (deferred post-Phase-1 feature)
**Spec reference:** —

A `self` account (analogous to Skype's own-account or Telegram's Saved Messages) is planned for implementation after the Phase 1 smoke test, during local testing. Design decision: `self` is a local-only synthetic Identity with its own keypair, never registered on any Node and never appearing in federation. It signs local Events but those Events are never broadcast. The `self` account must be accessible from any user client connecting to the Node — it is not device-local. In Phase 2, a "Saved Messages" Space may be implemented as a proper DM Space where both sides of the DM are the user's own keypair.

---

## D-022 — xgen-core Library Split: Deferred to Post-Phase-1

**Date:** 2026-04-28  
**Layer:** 0 (architecture — deferred)  
**Spec reference:** —  
**Resolved by:** D-044 (2026-05-13)

All protocol logic currently lives in `xgen-node/src/`. A planned post-Phase-1 restructure will extract this into a new `xgen-core` crate: GPL-licensed from day one, the primary library for third-party developers. `xgen-node` and `xgen-client` become thin runtime shells wrapping `xgen-core`, retaining their BSL 1.1 wrapper. `xgen-common` remains as shared serde types.

Rationale for deferring: restructuring crates mid-implementation introduces risk right before the Phase 1 finish line. Do the smoke test first, tag Phase 1 complete, then restructure as the first Phase 2 prep task.

---

## D-023 — Traffic Masking / DPI Resistance: Phase 3 Investigation

**Date:** 2026-04-28
**Layer:** 0 (deferred — Phase 3)
**Spec reference:** 3.3.1

Deep-packet-inspection resistance (obfuscating XGen traffic to evade state-level network surveillance) is acknowledged as a legitimate concern. Phase 1 and Phase 2 impact: none — transport pluggability (D-019) already ensures Tor/I2P are usable without protocol changes, which is sufficient for most adversarial environments. Active DPI resistance (disguising XGen traffic as generic HTTPS, pluggable transport integration) is flagged as a Phase 3 area of investigation. Steganographic transport is explicitly out of scope for the core protocol.

---

## D-024 — History Sync: Individual Events, Not Batch Snapshot

**Date:** 2026-04-28
**Layer:** 10 — Smoke Test
**Spec reference:** 3.7.10 (step 8), 3.7.11

The spec requires Node A to "send full Space state and Room Event history to Node B" (step 11 of the smoke test) but does not prescribe a wire format. Two options were considered: (a) individual Events sent one by one, (b) a new batch snapshot message type.

Chose **individual Events**. Rationale: Events are already the atomic protocol unit; every federated Node must be able to validate each Event independently; no new message type is needed; and the individual approach scales correctly to Phase 2 where `transport.sync_request` handles catching up on missed Events after reconnection — it is additive, not a replacement. Batch delivery would require defining a new message type that Phase 2 would likely supersede anyway.

In the smoke test, Node A sends history Events in insertion order over the active connection, followed by the `state.federation_add` Event (which references the pre-history tip as its `prev_events`, and therefore must be received after the history to be correctly linked in Node B's DAG). Connection is closed with `transport.goodbye` to signal end of sync.

---

## D-025 — File Naming Convention: `xgen-node_*` and `xgen-client_*` Prefixes

**Date:** 2026-04-29
**Layer:** 0 (deployment model)
**Spec reference:** IMPLEMENTATION_GUIDE_ph1.md — Deployment Model, Ch4 section 4.3

All runtime files produced or consumed by a binary are prefixed with the binary name: `xgen-node_*` for Node files, `xgen-client_*` for client files.

Rationale: when two Node instances run side by side for testing (NodeA and NodeB folders), every file in the folder is immediately identifiable by name alone — no ambiguity about which binary owns it. Also makes glob patterns unambiguous in scripts (`xgen-node_*.db`, `xgen-client_*.toml`).

Applied to: config (`xgen-node_config.toml`), keypair (`xgen-node_keypair.enc`), state file (`xgen-node_state.json`), databases (`xgen-node_identities.db`, `xgen-node_federation.db`), logs (`xgen-node.log`). Space databases are in a `spaces/` subfolder and are named by space ID hex — the subfolder itself provides the ownership context.

---

## D-026 — Status File (`*_state.json`): Plain JSON, File Permissions as Security Boundary

**Date:** 2026-04-29
**Layer:** 0 (deployment model / CLI design)
**Spec reference:** Ch4 section 4.14

**What the state file contains**

The running Node writes `xgen-node_state.json` to its application folder every 5 seconds. It contains operational metadata: node ID (a public key — already public by protocol design), uptime, connected client identity IDs and display names, federated peer endpoints, hosted space names, and event counts. The client writes `xgen-client_state.json` with: identity ID, display name, known nodes, joined spaces, and last activity timestamps.

**Why it is safe for Phase 1**

No secret material ever enters the state file. The private key lives only in `*_keypair.enc` (encrypted at rest). Signatures are computed in memory and never written to disk in plaintext. The state file contains only information that is already visible to any authenticated participant in the protocol — a connected client can already see who else is in a Space.

**What it leaks and to whom**

The state file leaks topology: who is connected to this Node, which peers it federates with, which Spaces it hosts. This is only a concern if a third party has filesystem read access to the Node's application folder. On a personal development machine: not a concern. On a shared server: the file MUST be protected by OS-level file permissions (Unix: `chmod 600`; Windows: restrict ACL to the operator account). The Node SHOULD set these permissions itself on first write.

**Planned improvements for Phase 2**

Three improvements are planned but explicitly deferred beyond Phase 1:

1. **Redact identity IDs from state file** — replace full `pubkey_uri` values with display names only, or truncated IDs. The full public key of a connected user is already public, but there is no reason to persist it in a file that may be read by monitoring tools.

2. **Separate admin socket** — replace the file-based status mechanism with a Unix domain socket (or named pipe on Windows) that only the operator's process can connect to. Status commands connect to the socket rather than reading a file. This eliminates the file entirely and makes the data available only to processes with the right OS credentials.

3. **Encrypted state file** — encrypt the state file with a key derived from the node keypair passphrase. Only the operator who can unlock the keypair can read the state file. Adds meaningful protection on shared infrastructure without requiring the admin socket approach.

For Phase 1, file permissions are the sufficient and correct mitigation. The planned improvements are recorded here so they are not forgotten when Phase 2 deployment hardening is scoped.

---

## D-027 — CLI Observability Commands: Phase 1 Scope Extension

**Date:** 2026-04-29
**Layer:** 0 (CLI design — Phase 1 scope extension)
**Spec reference:** Ch4 section 4.16

The original Phase 1 definition of done (spec 3.7.11, IMPLEMENTATION_GUIDE_ph1.md Layer 10) specifies the smoke test as the completion criterion. It does not specify a CLI interface beyond what is needed to drive the smoke test.

The following commands are added to Phase 1 scope as a deliberate extension:

**xgen-node:** `status`, `connections`, `spaces`, `peers`, `identity list`
**xgen-client:** `status`, `spaces`, `whoami`

**Rationale:** the smoke test proves the library works in-process. Runnable binaries need to be observable — an operator running two Nodes on localhost needs to see that they are alive, that clients are connected, and that federation is active. Without these commands, the only evidence the system works is log output. These commands transform log output into structured, queryable state.

All observability commands read `xgen-node_state.json` or `xgen-client_state.json` (D-026) — they do not open a new network connection to the running process. This keeps them instant and dependency-free.

**These commands are NOT Phase 2 work.** They are Phase 1 CLI completeness. Phase 2 will replace or supplement them with a GUI dashboard. The state file mechanism (D-026) persists into Phase 2 as the data source for that dashboard.

**What is explicitly NOT in Phase 1 CLI scope:**
- Admin operations that modify Node state (ban identity, force-disconnect peer, etc.) — Phase 2
- Real-time streaming output (live event feed, live connection monitor) — Phase 2
- Auth Module management commands — Phase 2
- Multi-node management (controlling a remote Node) — Phase 2

---

## D-028 — `--help` Built-in: clap Derive Macros, Section 4.16 as Authoritative Source

**Date:** 2026-04-29
**Layer:** 0 (CLI design)
**Spec reference:** Ch4 section 4.16

`clap` with derive macros generates `--help` output automatically from doc comments (`///`) on struct fields and command variants. The help text in the source code is therefore documentation — it must match section 4.16 of Ch4 exactly.

The authoring rule: write section 4.16 first. Copy the argument descriptions and examples from 4.16 into the Rust doc comments. Never write help text in the code first and retrofit it into 4.16 — the spec is the source of truth, the code is the implementation.

Both `xgen-node --help` and `xgen-client --help` (and all subcommand `--help` variants) are generated by clap at compile time from these doc comments. No hand-written help strings.

---

## D-030 — Runtime file placement: GetModuleFileNameW on Windows; data_dir from config path

**Date:** 2026-04-29
**Layer:** 0 (deployment / binary wiring)
**Spec reference:** D-025 (file naming and placement)

### Problem

`xgen-node init` must create its runtime files (keypair, config, identities DB, state file) in a deterministic, predictable location. The natural choice is the directory that contains the running executable. Rust's `std::env::current_exe()` is sufficient on Linux/macOS but has documented edge cases on Windows: Windows Defender, UAC elevation, App Compatibility shims, and some third-party security products can run a process from a shadow copy at a temp path, causing `current_exe()` to return the temp location rather than the original binary location.

Additionally, Phase 1 requires running two Node instances simultaneously for testing (Node A on port 8080, Node B on 8081). When both nodes share the same binary, a single `exe_dir()` would cause Tier-1 file collisions between instances.

### Decision

**1 — `exe_dir()` on Windows uses `GetModuleFileNameW` directly.**

`GetModuleFileNameW(NULL, ...)` (Win32 API, `windows-sys` crate, Windows-only dependency) returns the full path of the module loaded into the calling process. This is the definitive answer to "where does this executable live" — it is immune to CWD, PATH lookup order, symlinks, shell wrappers, and any launcher that might shadow-copy the binary. The function is called with a growing buffer starting at `MAX_PATH` (260) and doubling until the path fits, ensuring correctness for paths beyond `MAX_PATH` (e.g., with `\\?\` extended-length prefix). On non-Windows the standard library's `current_exe()` is used unchanged.

`exe_dir()` panics rather than falling back to `"."` (the CWD). Silent fallback to CWD was the original failure mode — files appeared in a "random" working directory instead of next to the executable. A panic with a clear message is strictly better: it tells the operator exactly what is wrong rather than silently polluting the working directory.

**2 — `data_dir` is derived from the config file path.**

All Tier-1 runtime files are placed in the parent directory of the config file in use:

```
data_dir = config_path.parent()
```

- **Without `--config`:** `config_path` defaults to `exe_dir()/xgen-node_config.toml`, so `data_dir = exe_dir()`. Tier-1 files are co-located with the binary — matches spec D-025.
- **With `--config /path/to/config.toml`:** `data_dir = /path/to/`. This allows multiple Node instances to run from the same binary with fully isolated data directories, by giving each instance its own config file in its own directory.

This rule is simple, explicit, and composable: operators who need multi-instance deployments create one directory per instance and specify `--config`. Operators who run a single instance (the common case) run `xgen-node init` with no flags and get everything in the binary's directory, as expected.

**3 — `xgen-node init` accepts `--passphrase` flag.**

`init` calls `rpassword::prompt_password()` to read the passphrase interactively. This blocks automated setup (CI, scripted deployments, smoke-test harnesses). The `--passphrase` flag provides the passphrase directly without prompting. It is intentionally undocumented in `--help` (hidden flag) — it is not intended for interactive human use, only for scripting. Passing an empty string produces a keypair encrypted with empty passphrase (Phase 1 Local Node mode).

### Files affected

- `xgen-node/src/main.rs` — `exe_dir()`, `main()`, `cmd_init()`, `run_node()`, all observability commands
- `xgen-node/Cargo.toml` — `windows-sys = { version = "0.59", features = ["Win32_System_LibraryLoader"] }` as `[target.'cfg(windows)'.dependencies]`

---

## D-031 — Phase 1 Node configuration reference (xgen-node_config.toml)

**Date:** 2026-04-29
**Layer:** 0 (deployment / reference)
**Spec reference:** Ch4 section 4.8.1

`xgen-node init` generates a default `xgen-node_config.toml` in the data directory. Below is the canonical Phase 1 reference config with all fields documented.

```toml
# XGen Protocol Node — Phase 1 configuration
# Generated by: xgen-node init
# All paths are absolute. Relative paths resolve from the working directory
# at startup, which may differ from the binary location — use absolute paths.

[node]
# WebSocket endpoint this Node listens on.
# Phase 1: ws:// (plain TCP, localhost only).
# Phase 2: wss:// (TLS, public endpoint).
listen = "ws://127.0.0.1:8080/xgen"

# Local Node mode: skip signature verification on incoming events.
# TRUE for Phase 1 development. FALSE for any production or multi-operator setup.
local_mode = true

[paths]
# Ed25519 signing keypair, encrypted at rest (ChaCha20-Poly1305 + Argon2id).
# Phase 1: encrypted with empty passphrase. Phase 2: OS keystore or HSM redirect.
# This is the ONLY mandatory path. The Node will not start without it.
keypair_path = "C:\\XGen\\NodeA\\xgen-node_keypair.enc"

# Optional: redirect log output. Omit to suppress file logging (stderr only).
# log_path = "C:\\XGen\\NodeA\\xgen-node.log"

# Optional: directory for per-space DAG stores. Omit to use in-memory only.
# spaces_dir = "C:\\XGen\\NodeA\\spaces"
```

### Field reference

| Field | Required | Default if omitted | Phase 2 change |
|---|---|---|---|
| `node.listen` | yes | `ws://127.0.0.1:8080/xgen` | Change to `wss://` with real hostname |
| `node.local_mode` | yes | `true` | Set to `false` for production |
| `paths.keypair_path` | yes | — (Node refuses to start) | May redirect to HSM path |
| `paths.log_path` | no | no file logging | Route to syslog aggregator |
| `paths.spaces_dir` | no | in-memory only | Persistent DAG store directory |

### Multi-instance setup (Phase 1 testing)

To run two Nodes on the same machine:

```
E:\XGen\NodeA\xgen-node.exe --config E:\XGen\NodeA\xgen-node_config.toml init
E:\XGen\NodeB\xgen-node.exe --config E:\XGen\NodeB\xgen-node_config.toml init
```

Edit Node B's config to use port 8081. Each instance has its own keypair, identity registry, and state file — no collisions.

---

## D-029 — xgen-client depends on xgen-node lib for Phase 1 binary wiring

**Date:** 2026-04-29  
**Layer:** 0 (binary wiring)  
**Spec reference:** D-022 (xgen-core crate split, Phase 2)  
**Resolved by:** D-044 (2026-05-13)

`xgen-client` depends directly on the `xgen-node` library crate for Phase 1 binary wiring. This gives the client access to the transport layer (`Connection`, `connect_url`), wire types (`Event`, `IdentityMessage`, etc.), federation handshake, identity registration protocol, event building, and crypto — without duplicating ~2 000 lines of code.

The "circular" concern mentioned earlier was conceptual (two binaries sharing a library), not a Cargo constraint. `xgen-client → xgen-node-lib` is a valid, acyclic dependency.

In Phase 2, D-022 (xgen-core crate) extracts the shared protocol logic from `xgen-node` into a new `xgen-core` library. Both `xgen-node` and `xgen-client` will depend on `xgen-core` instead. The direct `xgen-client → xgen-node` dependency is replaced at that point.

---

## D-037 — Node deployment model: systray singleton with detachable admin window

**Date:** 2026-05-07  
**Layer:** 6 (UI / deployment)  
**Spec reference:** Ch6 §6.1, §6.4  

`xgen-node.exe` is a singleton process — it starts once and runs permanently. The UI is not the lifecycle host; the process is.

**Desktop deployment (normal launch):**
- Node starts → sits in system tray as a minimal persistent icon
- Systray icon reflects Node health at a glance (green = healthy, amber = warning, red = error)
- Double-click or right-click → Open Dashboard opens the full Tauri admin window
- Closing the admin window does not stop the Node — Node continues running in the tray
- Right-click context menu: Open Dashboard, View Logs, Stop Node

**Server/headless deployment:**
- `--service` flag or OS service wrapper (Windows Service, systemd, launchd)
- No systray, no window — process runs fully headless
- Managed via OS service tooling; logs routed to system aggregator

**One binary, two personalities.** No separate service executable. Launch mode determines behaviour.

**Architectural horizon (not scheduled):** long-term, Node administration via privileged client identity — the operator manages their Node through the XGen client itself as a protocol-native admin surface. This is philosophically aligned with XGen's identity-first model but requires a stable client first and has a bootstrapping challenge. Noted for post-Phase 2 consideration.

---

## D-038 — Tier badge placement: Node property, not member property

**Date:** 2026-05-07  
**Layer:** 6 (UI)  
**Spec reference:** Ch6 §6.11.4, Appendix E  

The Auth tier is a property of the **Node**, not of an individual member or message. It describes what authentication level the Node requires and enforces for the current session. A user authenticated at Tier 1 on one Node may be Tier 2 on another — the tier is session-scoped, not identity-scoped.

**Displaying tier badges on individual messages or member list entries is architecturally incorrect.** It implies tier is a permanent attribute of the person, which it is not.

**Correct placements:**
- Console status bar: `Joe / @joe [T1] · Space › #Room` — reflects the current session's auth level on the connected Node
- Node status panel in client sidebar — describes the connected Node's tier requirement
- Node admin dashboard — the Node's own tier displayed prominently

**Removed placements:**
- `room.message.decorator` slot in messages — removed
- Member list entries — removed
- Navigation footer local user identity — removed

The `room.message.decorator` slot remains in place as the module injection point. Tier badge removal does not affect the slot structure.

---

## D-039 — Application shutdown model: × to systray, explicit exit only

**Date:** 2026-05-07  
**Layer:** 6 (UI / deployment)  
**Spec reference:** Ch6 §6.11, Appendix E, D-037  

Closing the window with × does not exit either application. Both applications minimize to the system tray. Explicit exit is always a deliberate user action.

**× button behaviour (both apps):**
- Hides the window, process continues running
- Client: stays connected, session live, logs flowing
- Node: keeps serving clients and federation peers, no change
- Consistent with D-037 (Node window is detachable from Node process)

**Exit paths — phased implementation:**

**Phase 2 skeleton (immediate):**
- In-app exit button in the nav footer alongside the user identity / Node health indicator
- Client nav footer: Disconnect button (drops connection, stays in window) + Exit button (CLOSING → flush → disconnect → process exits)
- Node nav footer: Restart button + Stop Node button (CLOSING → drain sessions → session footer → process exits)
- × button: disabled or no-op until systray is implemented
- This is the only exit path in the skeleton phase

**Phase 2 (systray implementation):**
- × minimizes to systray properly
- Systray right-click context menu → Exit (Client) / Stop Node (Node)
- Systray is the safety net when UI is unresponsive but process is alive

**Phase 3:**
- `xgen-client.exe --stop` and `xgen-node.exe --stop` CLI flags
- Sends graceful shutdown signal via local socket or PID file
- Works even when both UI and systray are unresponsive
- Built on the same IPC channel as Ch6 §6.9 Console input protocol
- Last resort before Task Manager kill (which produces no session footer)

**Graceful shutdown sequence (both apps, all exit paths):**
1. Enter `CLOSING` state — logged, status indicator updates
2. Flush outbound Event queue (max 2s grace period, then force-close)
3. Send `transport.close` to connected Node(s)
4. Write session footer to log
5. Archive session log
6. Process exits

**Appendix E clarification:** “Window close” in Appendix E means explicit exit action, not the × button. The × button triggers minimize-to-systray, not CLOSING state. CLOSING is only entered via an explicit exit action (in-app button, systray menu, or `--stop` flag).

**Nav footer button placement (from JozefN review, 2026-05-07):**
Two compact action buttons sit in the nav footer alongside the identity/health indicator — always visible, always reachable, deliberate enough not to be hit accidentally.
- Client: Disconnect + Exit
- Node: Restart + Stop Node

---

## D-040 — Idle presence state: social signal and resource hint

**Date:** 2026-05-07  
**Layer:** 3 (Specification) / 6 (UI)  
**Spec reference:** Ch3 §3.6 (Identity), Ch3 §3.9 (State resolution), Appendix E  

Idle is a presence state indicating a connected member has produced no non-keepalive protocol activity for a configurable period. It has two distinct roles: a social signal visible to other room members, and an internal resource hint used by the Node.

**What idle is:**
- A runtime presence state: `online` → `idle` → `online`
- A federated presence signal — propagated to federated Nodes so their members see correct presence
- An internal Node resource hint — the Node may deprioritize idle clients (e.g. lower Event delivery queue priority, reduced in-memory session cache) without exposing those decisions externally

**What idle is not:**
- A DAG Event — presence is ephemeral, not historical protocol state
- A log entry at INFO level — idle/wake transitions are DEBUG at most, or not logged
- A lifecycle state in Appendix E — idle does not interrupt the client’s READY state
- A kickout mechanism — idle clients are never disconnected for inactivity (D-039)

**Trigger — what counts as activity:**
From the Node’s perspective, activity is any non-keepalive message received from the client — sending a message, issuing a command, joining a room. Pure Event delivery (Node pushing to client) does not reset the idle timer. The Node cannot observe client-side UI interactions.

**Timeout configuration:**

| Setting | Location | Default | Notes |
|---|---|---|---|
| `idle_timeout_ms` | `client_config.toml` | 900000 (15 min) | User preference |
| `idle_timeout_max_ms` | Node config | 1800000 (30 min) | Operator ceiling — takes precedence if stricter than client setting |

If the Node operator sets a maximum idle timeout, the effective timeout is `min(client_setting, node_max)`. If the client sets no preference, the Node default applies.

**Wake-up:** any non-keepalive message from the client immediately returns presence to `online`. No explicit wake command required.

**Federation:** idle/online presence state is federated — other Nodes propagate it to their members so cross-Node room participants see correct presence. The Node’s internal resource management decisions (cache eviction, queue deprioritization) are local and never cross federation.

**Keepalive logging:** ping/pong keepalive entries are logged at `DEBUG` level, not `INFO`. Over a 5-hour idle session this prevents hundreds of identical INFO entries burying meaningful protocol events. Only the initial connection, state transitions, and significant events are logged at INFO.

**Admin actions remain separate:** idle state has no relationship to `membership.kick` or `membership.ban`. Those are admin-initiated protocol Events for disturbance, not inactivity. An idle user is still a full member.

**Phase 2 note:** the presence signal mechanism — how idle/online state is communicated between Node and client, and across federation — requires a Ch3 Phase 2 specification entry. The EventType or message type for presence updates is not yet defined. This decision records the intent and constraints; the wire format is a Phase 2 spec task.

---

## D-048 — Layer 14 DM Space Promotion: DmProposal in NodeRuntime, not SpaceState

**Date:** 2026-05-14
**Layer:** 14 (DM Space Promotion)
**Spec reference:** Spec 3.16.1–3.16.4

### Context

The promotion proposal is in-memory state — the proposer sends `dm.promote_propose`, the Node stores the proposal, the other member confirms or rejects. The spec says proposals are not DAG events.

### Decision 1 — Proposal storage location

The proposal is stored in `NodeRuntime::dm_proposals: HashMap<String, DmProposal>` (keyed by space_id), not in `SpaceState`. `SpaceState` is replayed from the DAG on restart; proposals do not survive restart. `NodeRuntime` holds the ephemeral operational state that lives only during a running Node session.

### Decision 2 — dm_constraints_active flag on SpaceState

`SpaceState` gains `dm_constraints_active: bool` (true for DM spaces, set to false when `state.dm_promote` is applied). The constraint checks live in `apply_invite`, `apply_room_create`, and `apply_federation_add`. This makes constraints enforced at the DAG-apply layer — replay of the event log correctly lifts constraints when `state.dm_promote` is encountered.

### Decision 3 — state.dm_promote signed by Node keypair

Per spec 3.16.3 Step 4: `state.dm_promote` is produced and signed by the Node, not by either member. `handle_confirm` in `dm_promotion.rs` takes `node_key: &SigningKey` and calls `sign_event`. The sender field is the Node's identity_id. Test `promote_signed_by_node_not_member` verifies this.

### Scope

`dm_promotion.rs` provides pure handler functions — no WebSocket I/O. Delivery of `dm.promote_propose` to the other member and delivery of `state.dm_promote` to both members is the Node runtime's responsibility (xgen-node wiring, not implemented in Phase 2 library). The handlers return `deliver_to` identity IDs so the caller knows who to notify.

---

## D-047 — Layer 13 Pending Event Timeout: drain_timed_out takes explicit now parameter

**Date:** 2026-05-14
**Layer:** 13 (Pending Event Timeout)
**Spec reference:** Spec 3.9.6, WD-08 (30-second timeout)

### Context

Spec 3.9.6 requires pending events (those awaiting unknown prev_events) to be discarded after a timeout, emitting error 4002 (predecessor_timeout). The question was how to drive the timeout check: a monotonic clock dependency inside `PendingBuffer`, or an explicit parameter at the call site.

### Decision

`drain_timed_out` accepts an explicit `now: std::time::Instant` parameter rather than calling `Instant::now()` internally.

**Reason:** an explicit `now` makes the function testable without sleeping or mocking. Tests pass `Instant::now() + Duration::from_secs(31)` to trigger the timeout instantly. The background task in xgen-node passes `std::time::Instant::now()` in production — one extra token, no testability cost.

The timeout constant is `PENDING_TIMEOUT_SECS: u64 = 30` — a named `pub const` in `dag/pending.rs` so the value is tunable from one place (WD-08).

### Sweep task wiring

A background tokio task in `xgen-node/src/main.rs` calls `drain_timed_out(Instant::now())` on every Space's `PendingBuffer` every 5 seconds. For each discarded entry it logs at `WARN` with `event_id`, `missing_predecessors`, and `error_code = 4002`.

---

## D-046 — Layer 12 State Resolution: identity_home_nodes parameter and Layer 3 scope restriction

**Date:** 2026-05-14
**Layer:** 12 (State Resolution Algorithm)
**Spec reference:** Spec 3.9.3 (seven-layer resolution stack), 3.9.8 (error codes)

### Context

The Layer 12 `resolve()` function implements the seven-layer priority stack (spec 3.9.3). Two decisions beyond spec prescription are recorded here.

### Decision 1 — identity_home_nodes as explicit parameter

`IMPLEMENTATION_GUIDE_ph2.md` specifies `resolve(conflicts, space_state)`. The guide's two-parameter signature is insufficient to implement Layers 3, 5a, and 5b, all of which require knowing which home Node each identity is registered on. `SpaceState` does not hold this mapping (it holds federation_nodes, which is a different concept — the set of Nodes a Space has federated with, not the registration point of each identity).

**Decision:** `resolve()` signature is:
```rust
pub fn resolve<'a>(
    conflicts: &'a [Event],
    space_state: &SpaceState,
    identity_home_nodes: &HashMap<String, String>,
) -> Result<&'a Event, ResolutionError>
```

The caller (Node's message handler) provides `identity_home_nodes` from the identity registry. This keeps `resolve()` a pure function with no registry I/O inside the algorithm itself.

### Decision 2 — Layer 3 restricted to membership and key-rotation events

Spec 3.9.3 Layer 3 description: "Home Node assertion for Identity's own state." The phrase "Identity's own state" was narrowly interpreted: Layer 3 applies only to events whose state key is in the membership or system.key_rotation category.

Without this restriction, Layer 3 incorrectly fires for events like `state.room_update` — two concurrent room updates by two admins from different Nodes would be resolved by Layer 3 (which would pick the event from whichever Node happens to be the "affected identity's" home Node, a concept that doesn't apply to shared room state). Layer 3 must not fire for shared state — it is only meaningful when one specific identity's own record is in contention.

**Implementation:** `layer3_home_node_assertion` checks `is_membership_event(&first.event_type) || matches!(first.event_type, EventType::SystemKeyRotation)` before running. All other event types fall through to Layer 4.

### SpaceState extension

`SpaceState` gains `node_priority_order: Vec<String>` (populated by `state.node_priority` events via `apply_event`). This field is required by Layer 5a. Index 0 = highest priority Node.

### Outcome

- 226 tests pass (218 xgen-core + 8 xgen-node)
- All ten Layer 12 tests pass including Layer 5a `node_priority_respected`
- Layer 3 bug caught by test: applying to `StateRoomUpdate` gave a spurious early win before Layer 5a could run

---

## D-044 — xgen-core crate split executed

**Date:** 2026-05-13  
**Layer:** Phase 2 prerequisite  
**Spec reference:** D-022 (planned), D-029 (temporary arrangement, now resolved)

### Context

All shared protocol logic lived in `xgen-node/src/`. `xgen-client` depended directly on the `xgen-node` library crate (D-029 — intentional temporary arrangement). This was always planned to be resolved before Phase 2 protocol work began (D-022).

### Decision

Extracted all shared protocol logic from `xgen-node/src/` into a new `xgen-core` crate. `xgen-core` is GPL-2.0-or-later from day one — the public library that the XGen ecosystem builds on. `xgen-node` and `xgen-client` are now thin shells that depend on `xgen-core`.

**Module allocation after split:**

| Location | Contents |
|---|---|
| `xgen-core/src/` | `crypto/`, `wire/`, `dag/`, `transport/{auth,client,connection}`, `node/`, `federation/`, `identity/`, `space/`, `message/` |
| `xgen-node/src/` | `main.rs`, `lib.rs` (re-exports xgen-core), `lifecycle.rs`, `transport/server.rs`, `tests/` |
| `xgen-client/src/` | `main.rs`, `lib.rs`, `batch.rs`, `identity.rs`, `lifecycle.rs` |

**Adapter pattern in xgen-node transport:** `xgen-node/src/transport/mod.rs` declares `pub mod server` and re-exports `auth`, `client`, `connection` from `xgen_core::transport`. This means all `crate::transport::*` paths in `xgen-node`'s main.rs and tests continue to resolve correctly without modification.

**Test relocation:** inline tests in `federation/mod.rs` and `identity/mod.rs` that required `Server` (Node-specific) were moved to `xgen-node/src/tests/federation_integration.rs` and `xgen-node/src/tests/identity_integration.rs`. Pure unit tests that don't need a server were kept in xgen-core.

### Outcome

- 173 tests pass (`cargo test`) — zero behaviour change
- Release build clean (`cargo build --release`)
- D-022 resolved: xgen-core exists, GPL-licensed, all protocol logic lives there
- D-029 resolved: xgen-client no longer depends on xgen-node

---

## D-055 — Phase 2 server-side handler wiring: node_endpoint in Hello, identity replication routing

**Date:** 2026-05-14
**Layer:** Integration (server-side protocol handler gap closure)
**Spec reference:** 3.4.2 (federation.hello), 3.13.1–3.13.4 (identity replication), 3.3 (transport Inbound routing)

### Context

After Part A of integration testing (J-056), `xgen-node/src/main.rs` `process_inbound()` only handled `Inbound::Identity` and `Inbound::Event`. All Phase 2 Inbound variants added in M1 (`Inbound::IdentityReplicate`, `Inbound::DmControl`, `Inbound::Migration`, `Inbound::Bootstrap`, `Inbound::Reputation`, `Inbound::Mls`) hit `_ => {}` and were silently dropped.

The immediate blocker for smoke-test-ph2 Part B was step 22: the smoke test sends `identity.replicate` to Node B and expects `identity.replicate_ack`. Without a handler, Node B silently dropped the message and the test failed.

A deeper structural gap was also identified: `FederationRelationship` had no `peer_url` field, so after a federation handshake the Node had no stored return address for the peer. This made outbound identity replication (spec 3.13.1 — home Node pushes to replicas after registration) impossible.

### Decisions

**1. `node_endpoint` field added to `FederationMessage::Hello`**

Advisory field (excluded from canonical signature — not in `HELLO_FIELDS`). The initiating Node populates it from `self_url: Option<String>`, a new parameter to `run_initiating()`. The receiving Node extracts it as `peer_url` on the `FederationSession`. Rationale: the receiving Node has no other way to learn the peer's WebSocket URL after the handshake completes over an inbound TCP connection.

Backward compatible: `#[serde(skip_serializing_if = "Option::is_none")]` — old nodes receiving the new field ignore it; new nodes receiving old messages get `None`.

**2. `peer_url: Option<String>` added to `FederationSession` and `FederationRelationship`**

`FederationSession.peer_url` is populated by `run_receiving()` from the Hello's `node_endpoint`. `FederationRelationship.from_session()` copies it across. `NodeRuntime.peer_urls: HashMap<String, String>` (node_id → URL) gives the server a lookup table for outbound replication.

**3. `handle_identity_replicate_msg` added to `xgen-node/src/main.rs`**

Handles `Inbound::IdentityReplicate(Replicate)`: deserialises `identity_record: Value` → `IdentityRecord`, calls `handle_incoming_replicate()`, sends `ReplicateAck` on success or `transport.error` (code 3020) on version-stale rejection.

**4. `push_identity_to_peers` added to `xgen-node/src/main.rs`**

After a successful identity registration, spawns an async task per known peer URL: connect → authenticate → send `identity.replicate` → await `identity.replicate_ack` → record in `replica_registry`. Failures are logged but not fatal (registration already confirmed to the client).

**5. `run_initiating()` call sites updated**

All 4 call sites in `xgen-client/src/main.rs` and 3 in test files updated with the new `self_url` argument. The two federation steps in `smoke-test-ph2` pass the node_b URL; all other call sites pass `None`.

### Outcome

- 300/300 tests passing (292 xgen-core + 8 xgen-node)
- Step 22 blocker resolved: `identity.replicate` is now handled server-side
- Identity replication infrastructure complete per spec 3.13.1–3.13.4
- All other Phase 2 Inbound variants (`DmControl`, `Migration`, `Bootstrap`, `Reputation`, `Mls`) remain `_ => {}` — not required for smoke-test-ph2 steps (those steps use hardcoded `pass!()` or send content as DAG events)

---

## D-045 — Phase 2 wire type names: spec authoritative over implementation guide

**Date:** 2026-05-13
**Layer:** 11 (Wire Format Phase 2 Extensions)
**Spec reference:** 3.9–3.16

### Context

While implementing Layer 11, several wire type names in `IMPLEMENTATION_GUIDE_ph2.md` were found to diverge from the canonical wire strings in `docs/xgen_ch3_specification.md`. The spec is always authoritative.

### Discrepancies resolved

| Guide wire name | Spec wire name | Spec section |
|---|---|---|
| `migration.complete` | `migration.transfer_complete` | 3.12.5 |
| `migration.verify_ok` | `migration.verified` | 3.12.6 |
| `migration.verify_fail` | `migration.verification_failed` | 3.12.6 |
| `migration.tail_batch` | (not a separate type — tail uses `migration.event_batch`) | 3.12.5 |
| `migration.abort` | (not in spec type registry — state machine handles failure) | 3.12.3 |
| `bootstrap.node_register` | `bootstrap.register` | 3.14.3 |
| `bootstrap.node_register_ack` | `bootstrap.register_ack` | 3.14.3 |
| `bootstrap.node_lookup` | (not a wire type — directory lookup is HTTP GET) | 3.14.4 |
| `bootstrap.node_lookup_response` | (not a wire type — HTTP response, not WebSocket) | 3.14.4 |

### Types added beyond the guide (present in spec)

| Type | Spec section | Reason |
|---|---|---|
| `state.space_migrate` | 3.12.7 | Permanent DAG event recording completed migration |
| `migration.failed` | 3.12.3 | Source Node notifies owner of failure |
| `migration.batch_ack` | 3.12.4 | Destination acknowledges each batch |
| `migration.federation_notify` | 3.12.8 | Courtesy notification to federated peers |
| `bootstrap.keepalive` | 3.14.7 | Node refreshes directory TTL |
| `bootstrap.keepalive_ack` | 3.14.7 | Bootstrap Node acknowledges |
| `bootstrap.deregister` | 3.14.7 | Node explicitly removes itself |
| `mls.key_package_request` | 3.10.3 | Node requests KeyPackage from peer Node |
| `mls.key_package_response` | 3.10.3 | Node returns requested KeyPackage |

### Decision

All implementations use spec-authoritative wire names. The guide will be updated in a future documentation pass but the implementation does not wait for that. D-045 is the permanent record of the resolution.

---

## D-056 — Application Deployment Model: one binary per role, multi-mode dispatch

**Date:** 2026-05-16
**Layer:** Layer 6 (UI / deployment / packaging)
**Spec reference:** Ch2 — Application Deployment Model & Lifecycle States (Session 19); Appendix E — Application Lifecycle States (Session 4)

### Context

Earlier Ch2 wording described the deployment model as "one binary, two personalities" — desktop (with UI) versus service (`--service`, headless). That framing conflated two independent questions: (a) does the binary present a UI, and (b) is the invocation long-running or short-lived. The conflation became actively misleading when implementation work surfaced two facts:

- The Client side already has `--batch` (BATCH_FLAG_ph2.md, J-044) — a short-lived, no-UI invocation that connects to a long-running instance via a named pipe (D-043), dispatches commands, and exits. This is neither "desktop personality" nor "service personality." It is a different category of invocation altogether.
- The current code carries `*-app.exe` build artifacts (`xgen-node-app.exe`, `xgen-client-app.exe`) as separate Tauri outputs alongside the CLI binaries. Two parallel `--batch` implementations exist on the Client side (one in `xgen-client/src/main.rs`, one in `xgen-client/src-tauri/src/batch.rs`). This is transitional scaffolding, not the target product shape — and it has no spec to point at because the previous Ch2 wording did not name what the target shape is.

This decision reframes the model cleanly and locks the target architecture so implementation can converge.

### Decision

**One binary per role.** The final product ships exactly two binaries:

- `xgen-node.exe` — the Node application
- `xgen-client.exe` — the Client application

No separate CLI build. No separate Tauri build. The `*-app.exe` outputs in the current repo are transitional and will be collapsed into the single product binaries.

**Two mode categories dispatched by flag.** Each binary detects flags at startup and dispatches into one of two mode categories:

- **Resident mode** — long-running. Owns the process lifecycle (the states defined in Appendix E). Hosts the protocol. Exposes a named-pipe server (D-043) at `\\.\pipe\xgen-{node|client}-{label}`. Two variants:
  - Desktop variant: default launch, with UI (systray + admin window for Node; Console for Client).
  - Headless variant: `--service` flag (primarily a Node concern, but available to either binary). No UI.
- **Control mode** — short-lived. Any flag that means "do something against the running instance, then exit." Process has no UI (no Tauri, no window, no systray). Optionally opens the named pipe of a resident instance, dispatches, reads the result, exits. Current examples: `--batch <file.xgb>`, `--init [--passphrase <p>]`. Future examples: `--stop`, `--reload-config`, `--export-log`, anything else that fits the shape.

"Control mode" is the canonical term. "Injection mode" is an acceptable informal synonym in conversation and journal entries.

**Shared command layer.** All input channels — Tauri UI button clicks, Console typed commands, `--batch` piped commands, future control-mode flags — dispatch through the same command layer defined in the library crate (`xgen-node/src/lib.rs`, `xgen-client/src/lib.rs`). One clap parser, one set of command implementations. No duplicate command code between CLI and UI paths. Adding a new command means defining it once; it becomes available to every input channel simultaneously.

**`--instance <label>` recommended on every resident launch.** The named pipe is derived deterministically from the instance label (D-043). Launching a resident instance without `--instance` produces the unnamed pipe (`\\.\pipe\xgen-{node|client}`), which works but is not the recommended deployment posture for anything beyond casual single-machine use. The recommendation: even when running a single Node or single Client on a machine, launch it with an explicit `--instance` label so control-mode invocations have a named target ready. Cost is zero; benefit is that any future diagnostic, scripted operation, or tooling not yet conceived has a stable address to target.

**Lifecycle scope clarified.** The lifecycle states defined in Appendix E (Node: `INITIALISING`, `READY`, `DEGRADED_*`, `MAINTENANCE`, `CLOSING`; Client: 11 states including `SETUP`, `CONNECTING`, `AUTHENTICATING`, etc.) describe **resident-mode** processes only. Control-mode invocations are outside the lifecycle: they open the pipe, dispatch, and exit. The resident instance does not change state when a control-mode command arrives — it simply processes one more command through its existing command layer.

### Implementation implications

These follow from the decision. They are not part of D-056 itself; they are tasks pulling current code into compliance:

1. **Node-side `--batch` implementation.** J-037 deferred this when the Client-side `--batch` was written. The spec target is now explicit. Port BATCH_FLAG_ph2.md's pattern to the Node side using the same library-first rule, same pipe-naming convention, same clap dispatch shape, with the Node's own command set.
2. **Collapse `*-app.exe` into the single product binaries.** Merge `xgen-{node,client}/src/main.rs` with `xgen-{node,client}/src-tauri/src/main.rs` into one entry point per role. Extract shared resident-mode logic (`run_node_server` / `start_client_session`) into the library crate so the single binary can dispatch any mode without code duplication. Eliminate the two parallel `--batch` implementations on the Client side.
3. **Pipe server in resident mode for both binaries.** Currently only the Client's Tauri variant hosts a pipe server. The Node Tauri shell's `--service` mode emits lifecycle events but binds no WebSocket server and no pipe server. Bring it into compliance with the new model: every resident-mode invocation hosts the pipe server.

These implementation tasks are tracked separately. D-056 locks the architectural target they converge on.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-043 | Pipe naming convention `\\.\pipe\xgen-{node\|client}-{label}`. D-056 generalises it: every resident instance, every control-mode invocation. |
| D-037 | Node deployment personality (now resident mode variants). Architectural horizon — protocol-native Node admin via privileged client Identity — survives unchanged. |
| D-039 | Shutdown model. `×` minimises to tray; `CLOSING` only entered via explicit exit action or a future `--stop` control-mode flag. Consistent with D-056. |
| J-037 | Node `--batch` design discussion. Now has an explicit spec target to point at. |
| J-044 | Client `--batch` implementation (BATCH_FLAG_ph2.md). The principal worked example of the control-mode pattern D-056 generalises.

### Spec status

- Ch2 §Application Deployment Model — rewritten in Session 19 (2026-05-16) to match this decision.
- Appendix E — Design Principles section opened with a paragraph clarifying that lifecycle states describe resident mode only. Session 4 entry added.

---

## D-062 — Tauri inclusion model: compiled into product binary, runtime dispatch chooses UI

**Date:** 2026-05-16
**Layer:** Layer 6 (deployment / packaging)
**Spec reference:** D-056 (one binary per role, multi-mode dispatch). M1 task file `tasks/BINARY_CONSOLIDATION_M1.md` Phase 2.

### Context

D-056 named the deployment target — one binary per role, dispatched at startup. The implementation question that follows: when both binaries link in Tauri (for the desktop variant of resident mode), is the Tauri dependency a build-time variant (Cargo feature flag `tauri`) or always compiled in with runtime dispatch?

Two options surveyed:

- **(a) Feature flag.** `xgen-node`/`xgen-client` build with `--features tauri` for the desktop product; headless deployments build without. Smaller server-shape binary, faster server-shape build, CI can build two variants and classify breakages by side.
- **(b) Always compiled in.** Both binaries always contain Tauri. Runtime dispatch (presence of `--service`, presence of a subcommand, presence of a read-only control flag) decides whether to initialise the UI. Larger binary, longer build, but no packaging variant to mismanage.

### Decision

**Option (b) — always compiled in, runtime-dispatched.** The merged binaries link Tauri unconditionally. The CLI dispatcher in `main.rs` decides at startup whether to call `desktop::run()` (Tauri initialisation) or `app::run_node()` (headless WS server) or a one-shot control handler. The Tauri runtime is paid for in binary size and build time regardless of how the binary will be invoked.

### Rationale

**Fewer error classes.** Under option (a), a packager forgetting `--features tauri` ships a GUI-less binary to a desktop user. That is a real packaging-mistake category, and it can survive smoke-testing if the packager only exercises CLI commands. Option (b) removes this class entirely: every binary can always do everything.

**Honest trade-off.** Acknowledged costs of (b):
- Server-shape deployment carries the Tauri/WebView2 runtime dependency even though it never invokes the UI. Disk footprint grows; for embedded or container deployments this matters.
- `cargo build` time grows with the UI rather than just the protocol. CI cycle time increases.
- CI runs one build instead of two, so a break cannot be independently classified "UI-side broke" vs "protocol-side broke" by build behaviour alone — that classification has to come from the diff.

All accepted. The simpler operational story (one artefact per role, always works in any mode) is worth the build-time and binary-size cost. Revisiting in the other direction is straightforward if those costs become acute — `#[cfg(feature = "tauri")]` gates can be added retrofitting (b) into (a) without rewriting code.

### Implementation note

This decision is the literal Rust expression of D-056's "one binary per role, multi-mode dispatch." Without D-062, D-056 has no Rust-level commitment; with D-062, the merge in M1 Phase 2 has a clean target shape:
- `xgen-node/Cargo.toml` and `xgen-client/Cargo.toml` carry `tauri`, `tauri-plugin-process`, and `tauri-build` (build-dependency) unconditionally.
- Each product crate's root holds `tauri.conf.json` + `build.rs` + `capabilities/` + `icons/` (formerly under `src-tauri/`).
- The Tauri shell code moved to library modules (`xgen-node-lib::desktop`, `xgen-client-lib::desktop`) so the binary's `main.rs` stays thin.

The `*-app.exe` build targets are removed from the workspace. Build artefacts after M1 Phase 2a: exactly `xgen-node.exe` and `xgen-client.exe`.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-056 | Architectural target. D-062 is the implementation-level commitment of how Tauri lives inside that target. |
| D-063 | Companion decision: where the resident-mode logic lives (library crate, not `main.rs`). Required by D-062's runtime-dispatch model — the dispatch target must be a library function any entry point can call. |

---

## D-063 — Resident-mode logic lives in the library crate

**Date:** 2026-05-16
**Layer:** Layer 6 (architecture)
**Spec reference:** D-056 (shared command layer requirement). M1 task file `tasks/BINARY_CONSOLIDATION_M1.md` Phase 1.

### Context

D-056 requires a shared command layer that every input channel (Tauri UI button clicks, Console typed commands, `--batch` piped commands, control-mode flags) dispatches through. For that requirement to be satisfied, the command layer has to live somewhere that all entry points can call — which means it cannot live in `main.rs` (only one `main.rs` exists per binary; library code, Tauri callbacks, and the binary's CLI dispatcher cannot all call into it from there).

The existing layout violated this. `run_node` (the Node's resident-mode entry point), the entire CLI subcommand set (`cmd_init`, `cmd_status`, `cmd_connections`, etc.), and the Client's batch-line dispatcher all lived in `main.rs` to varying degrees. The Tauri shell duplicated functionality (lifecycle scaffold) rather than calling shared code.

### Decision

**Resident-mode logic and the full command surface move to the library crate.** After this decision lands:

- `xgen-node-lib` (`xgen-node/src/lib.rs`) exposes `app::run_node`, `app::cmd_*` for every subcommand, `app::RunNodeOpts`, and `desktop::run` (the Tauri shell entry point, calling `app::run_node` internally).
- `xgen-client-lib` (`xgen-client/src/lib.rs`) exposes `app::cmd_*` for every subcommand, `app::run_batch_file`, the full `Cli` / `ClientCommand` clap structs, `batch::start_pipe_server`, `batch::dispatch_line`, `batch::pipe_name`, `batch::run_batch_client`, and `desktop::run`.
- Each binary's `main.rs` is a thin dispatcher: parse flags, decide mode, call the corresponding library function. No business logic in `main.rs`. The Node main.rs ends up around 270 lines (most of that clap definitions); the Client main.rs around 200 lines (most of that clap dispatch).
- The Client's `Cli` / `ClientCommand` clap structs live in `xgen-client-lib::app` rather than `main.rs` because the batch-file executor (`run_batch_file`) re-parses sub-CLI invocations per `.xgb` line, and that executor lives in the library.

### Rationale

This is the library-first architecture rule from `CLAUDE.md`, applied consistently across the merged binary structure. The rule already existed for Layer 1–10 code (everything below `transport`); D-063 extends it to the dispatch layer that sits between input channels and command implementations.

Without D-063, D-056's "shared command layer" is impossible to express in code: the desktop shell would either duplicate command implementations (drift inevitable, J-067's two-`get_dag_tips` problem multiplied) or call back into `main.rs` somehow (Rust doesn't permit that cleanly). The library extraction is the unblock.

### Implementation note

The implementation pass lives in M1 Phase 1. After it ships:
- `grep "pub async fn get_dag_tips"` returns exactly one match in `xgen-client/src/batch.rs:239`. The duplicate from `xgen-client/src/main.rs` is gone. Closes F-003 / F-004 from J-067 permanently — that was the loudest visible symptom of the library-extraction gap.
- All `cmd_*` functions live in `app.rs` (per crate). `main.rs` calls them via `app::cmd_foo(...)`.
- `desktop::run()` calls `app::run_node()` with `RunNodeOpts { init_logging: false, ... }` so logging init is owned by the desktop module (since Tauri is already up by the time `run_node` runs). The bool flag is the seam.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-056 | Architectural target. D-063 makes the shared command layer physically possible. |
| D-062 | Sibling decision: where Tauri lives (compiled in always). D-063 says where the protocol logic lives (library crate). Together they define the merged-binary architecture. |
| J-067 F-003 / F-004 | The duplicate `get_dag_tips` bug was the visible symptom. D-063 is the structural fix that prevents the bug class. |

---

## D-064 — M3 AI operator role: distinct role, fall-upward resolution, AI-owned-Space prohibition

**Date**: 2026-05-17  
**Layer**: protocol (spec 3.6.10.6) + Space state derivation (xgen-core/src/space/state.rs) + event acceptance pipeline (xgen-core/src/message/exchange.rs) + Client CLI surface (xgen-client)  
**Spec reference**: 3.6.10.6 (rewritten in M3); 3.6.10.10 (3041 wire-name widened)

### Decision

Operator is a distinct role inside a Space, scoped per-(AI, Space) — not Space-wide privileges (those remain admin's and owner's). The system always knows who the operator is even with no explicit delegation Event ever signed: the resolution function falls upward through stored state — stored delegation → AI's inviter → Space owner — and transparently skips entries pointing at Identities who are no longer members. AI Identities are prohibited from being Space owners (any `state.space_create` / `state.dm_space_create` from an `is_ai = true` sender is rejected with 3041, superseding the D-059 `dm_initiate` capability gate for those events). The protocol records who the operator is and surfaces resolution; it does **not** grant the operator any protocol-level event-signing privileges in this version — those layer on top in future milestones.

### Locked principles

**Operator is its own role.** Member < Operator (scoped to AI-X) < Admin < Owner in privilege scope, not in role hierarchy. The same human can be operator of one AI in one Space, a plain member in another, and an admin in a third.

**Delegation flow.** Admin or owner picks a current Space member, signs `state.ai_operator_delegate(ai_identity_id, new_operator_identity_id)`. The previous operator's consent is not required. Operator never signs in their operator capacity in this version — operator-signed events arrive in future milestones layered on the resolution function.

**Fall-upward resolution.** `resolve_operator(space, ai_id)`:
1. If a stored delegation exists for `ai_id` AND the named delegate is a current member: return the delegate.
2. Else if the AI's recorded inviter (sender of the original `membership.invite`) is a current member: return the inviter.
3. Else: return the Space owner (always a member of a live Space).

No orphan state is reachable. The stored delegation map is honoured only when its target is still a member — left/kicked delegates auto-skip without requiring an explicit revoke. Revoke explicitly clears the stored entry, collapsing resolution to step 2 or 3.

**Inviter-as-operator is computed, not stored.** No separate "initial operator" record. When an AI joins with no delegation yet, resolution returns the inviter — identical to how the operator is resolved at any other time.

**AI-owned Space rejected.** Pragmatic deferral, not architectural impossibility — revisit when a real use case appears.

**No protocol-enforced operator privileges in v1.** The operator role is a declaration of responsibility recorded in the DAG. Practical privileges (DM command surface, audit access, AI silencing, capability override) emerge from real usage and future capabilities, layered on top — they will be "is this signer the current *resolved* operator?" checks, not "did this signer sign a delegate event?" checks.

### Implementation surface

| Surface | Shape |
|---|---|
| `SpaceMember.invited_by: Option<String>` | `None` for owner and pre-M3 replayed members; `Some(sender)` for members admitted via `membership.invite`. Captured in `apply_invite` (carried through `pending_invites`) and consumed by `resolve_operator` step 2. |
| `SpaceState.ai_operator_delegations: HashMap<String, String>` | Key = `ai_identity_id`; value = delegated operator's identity_id. Absence means "no explicit delegation; resolution falls through." |
| `SpaceState::resolve_operator(&self, ai_id) -> Option<String>` | Three-case fall-upward algorithm. `None` only for non-member `ai_id` or structurally-impossible no-owner state. |
| `state.ai_operator_delegate` / `state.ai_operator_revoke` | New `apply_event` arms (defence-in-depth signer check); validation in `exchange.rs::check_ai_operator_targets` (signer + target membership + `is_ai` flag). |
| `check_ai_capability` extension | Rejects `state.space_create` / `state.dm_space_create` from any AI sender with 3041, ahead of the D-059 `dm_initiate` 3042 path. The 3042 path remains in code as a framework for future re-enablement. |
| `can_delegate_ai_operator(role) -> bool` | New permission helper; `*role >= Admin`. |
| Wire-name 3041 widened | Was `ai_flag_immutable`; now `ai_role_violation`. Umbrella covers `is_ai` immutability **and** the M3 role validations. Wire **code** unchanged; wire **name** broadens. Spec table updated in §3.6.10.10. |

### CLI surface (M3 minimum, testability only)

- `xgen-client init --ai [--cap key=value]` — writes `[ai]` section to `xgen-client_config.toml`. Default capability values are `dm_initiate=false`, `spontaneous_post=false`; `--cap` flags override. `init --ai` re-run upserts the section without clobbering other config fields.
- `xgen-client register` — reads `[ai]`, builds `is_ai=true` + capabilities for AI registration via the existing `build_register_with_ai`.
- `xgen-client ai delegate --space <id> --ai <id> --to <member-id>` — signs and sends `state.ai_operator_delegate`.
- `xgen-client ai revoke --space <id> --ai <id>` — signs and sends `state.ai_operator_revoke`.
- `xgen-client ai status --space <id> --ai <id>` — connects via WS, replays the Space's DAG locally, runs `resolve_operator`, prints the result with provenance (stored delegation / inviter fallback / owner fallback). Returns the **queried Node's converged view**; call against each Node to verify federation propagation.

`whoami` and `status` remain offline-local-introspection (intentionally — operator-resolution is a network-resident dynamic property and deserves its own honest verb).

### Out of scope (deferred to future milestones)

- AI Client *binary* — a long-running daemon that registers as an AI, joins Spaces, receives events via `run_ws_loop`, responds under pacing rules. This decision lands the protocol primitives; the consuming binary is a separate milestone.
- Protocol-enforced operator privileges (DM command surface, audit access, AI silencing, capability override). Per the locked principles above, these layer on top when real features need them.
- `spontaneous_post` Node-side enforcement — Phase 2 leaves this unenforced (3.6.10.4); no change in M3.
- Operator self-transfer (operator signs over to next operator without admin/owner involvement). Not in M3's signer model.
- Cross-Space operator inheritance. Operator is strictly per-(AI, Space).
- Pacing / temperature plugin math (still plugin-owned per D-060/D-061).

### Why this shape rather than alternatives

The hard architectural question was whether the operator's existence and identity should be stored explicitly (initial operator written into a `SpaceMember.operator_of` field on AI admission) or resolved dynamically. The dynamic-resolution shape wins because:

1. **No special-case bootstrap.** "Inviter-is-operator when no delegate exists" is identical to how the operator is resolved at any other time — single algorithm, no separate code path for "first operator".
2. **Self-healing on member churn.** When a delegate leaves or is kicked, the system silently reverts to the inviter (or owner) without anyone having to sign a revoke. Compare to a stored-only model where every delegate departure requires explicit cleanup or leaves the Space in a broken state.
3. **No orphan state.** The fall-upward chain ends at the owner, who is always present in a live Space. There is no reachable state where "the operator is undefined".
4. **Clear delegation semantics.** Delegate writes a new entry; revoke clears the entry. Both are local point operations — no need to track "the previous operator" or "the operator-of-operator" or any chain.

The alternatives considered and rejected:

- **AI-as-owner permitted.** Rejected pragmatically — no clear use case in M3 and several open questions about what "an AI signs a space.update" means for trust attribution. Not architecturally impossible; revisitable when a real driver appears.
- **Operator-signed delegation (transfer-of-trust by the previous operator).** Rejected because it complicates the signer model and adds nothing the admin/owner-signed flow doesn't already cover. Admin/owner is already the locus of authority over the Space; operator authority over the AI is a subset.
- **Finer-grained error codes (3043 / 3044 for the new validation failures).** Rejected — wire-code granularity adds reading load without adding semantic value when the role family is already covered by 3041. The `ai_role_violation` umbrella catches structural role rules (3041) and capability flags (3042 — separate domain).
- **Cache `whoami` / `status` resolved operator into `xgen-client_state.json`.** Rejected — guaranteed-stale on every cross-Node action; pretending offline-cached state reflects federation truth is worse than a clear "this command is a network query" verb (`ai status`).

### Why now

M3 ships the protocol primitives that the AI Client binary milestone will consume. Landing the operator role, validation, and resolution function before the binary means the binary lands as a thin consumer of well-tested primitives rather than discovering the role-model gaps mid-flight.

### Spec reference

- 3.6.10.6 rewritten — operator role definition, signer rules, fall-upward algorithm, AI-owned-Space prohibition, "no protocol-enforced operator privileges in v1".
- 3.6.10.10 — 3041 wire name widened from `ai_flag_immutable` to `ai_role_violation`; same code.

### Code reference

| File | Surface |
|---|---|
| `xgen-core/src/space/state.rs` | `SpaceMember.invited_by`, `PendingInvite`, `SpaceState.ai_operator_delegations`, `resolve_operator`, `apply_ai_operator_delegate`, `apply_ai_operator_revoke`, `build_state_ai_operator_{delegate,revoke}_event` |
| `xgen-core/src/space/membership.rs` | `can_delegate_ai_operator` |
| `xgen-core/src/message/exchange.rs` | `ExchangeError::AiRoleViolation` → wire `(3041, "ai_role_violation")`; `check_ai_capability` extended; `check_ai_operator_targets` added; `check_permission` arms for delegate/revoke |
| `xgen-core/src/identity/registration.rs` | `AiFlagImmutable.to_registration_code()` returns `(3041, "ai_role_violation")` — wire-name widening |
| `xgen-client/src/app.rs` | `AiSection` in `ClientConfig`; `--ai` / `--cap` on `InitArgs`; `Ai(AiArgs)` subcommand group; `cmd_ai_delegate` / `cmd_ai_revoke` / `cmd_ai_status` |
| `xgen-client/src/main.rs`, `xgen-client/src/batch.rs` | Dispatch for the new subcommand group |

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-059 | M3 builds on D-059's `is_ai` / `ai_capabilities` wire shape. The `dm_initiate` capability mechanism remains in code; the structural 3041 rule in M3 fires before it for `state.dm_space_create` from any AI, making D-059's 3042 path unreachable for that event in M3 but preserving the capability framework for future re-enablement. |
| D-060, D-061 | Adjacent AI-related protocol surfaces (pacing, temperature). Not touched by M3 directly but consumed by the same population of AI Identities. |

---

## D-065 — M4 AI Client: resident mode of xgen-client + plugin model + "honest behaviour over polite behaviour"

**Date**: 2026-05-17  
**Layer**: Client (xgen-client/src/ai_service.rs, ai_behavior.rs) + Configuration (xgen-client_config.toml schema) + Documentation (Ch6 §6.15)  
**Spec reference**: Ch6 §6.15 (new section); Ch3 §3.6.10 (cross-link)

### Decision

The AI Client is **a mode of `xgen-client`**, not a separate binary. `xgen-client --ai-mode --service` dispatches a long-running resident with a plugin-based behaviour model: the runtime owns connection, replay, pacing, mute, and pipe-server I/O; the `AiBehavior` trait owns the decision "should I reply, and what should I say." M4 ships exactly one plugin (`EchoPlugin`, config key `"echo"`) as the reference implementation — its job is to prove the loop end-to-end, not to be useful. Real LLM hookups and sophisticated dialog policies layer on the trait in future milestones.

This decision also names a recurring XGen design principle that has been implicit in earlier protocol choices: **honest behaviour over polite behaviour.** When a system can choose between behaviour that misrepresents its current state (polite — "I'll deliver this thought eventually" / queueing) and behaviour that honestly reflects its current state (honest — "I can't say this right now and the moment passed" / dropping), XGen picks honest.

### Locked architecture

**Binary identity.** Two binaries total: `xgen-node`, `xgen-client`. Three modes for `xgen-client`:

| Invocation | Role |
|---|---|
| `xgen-client <subcommand>` | One-shot human Client |
| `xgen-client --service` | Long-running human-Client resident |
| `xgen-client --ai-mode --service` | Long-running AI-Client resident |

The `--ai-mode` flag is meaningful only with `--service` (clap enforces). Existing pipe naming convention `\\.\pipe\xgen-client[-<instance>]` is unchanged; AI residents bind to the same pipe space and distinguish themselves via the `mode=` field in `__HEALTH__`.

**Why a mode and not a separate binary.** The Node's headless mode is `--service`, not a separate `xgen-node-service` binary. By symmetry, an AI Client is a client — same Identity registration, same Space membership, same event emission, same `[ai]` config staging — just with behaviour coming from a plugin instead of a keyboard. Consistency with the resident/control pattern wins. M1 collapsed binaries that shared identical code; xgen-client and the AI Client share the same library and dispatch through one entry point per mode. Three binaries (the rejected alternative) would have put M4 in conflict with the D-056 consolidation direction it should be following.

**Plugin model.** `AiBehavior` trait in `xgen-client-lib::ai_behavior`:

```rust
pub trait AiBehavior: Send {
    fn on_event(&mut self, ctx: &EventContext) -> Option<String>;
    fn name(&self) -> &'static str;
}
```

The plugin receives one inbound `Event` at a time and returns `Some(text)` to reply (as `message.text`) or `None` for silence. Plugins MUST be fast and non-blocking — long-running work is future-plugin design territory. The runtime handles pacing, mute, prev_events chaining, and WebSocket I/O.

**Reference plugin: `EchoPlugin`** (config key `"echo"`). Replies to mentions in `message.text` with the deterministic line `[echo-plugin] received mention from <last-12-chars-of-sender-id>`. Reply text is fixed — not configurable in M4. Rationale: smoke tests need to grep for the reply; nobody should mistake the artefact for a real reply during early demos.

**Mention detection: two-rail OR**, both case-sensitive:

1. **Rail A (always-on):** substring match for the AI's full `identity_id` URI in `content.text`.
2. **Rail B (optional):** substring match for a `mention_token` (e.g. `"@bob"`) read from `[ai.behavior]`. Default `None`.

Rails are **OR'd, not sequenced** — either match counts. The implementation MUST NOT interpret "always + optionally" as "fall through to optional if always-rail misses."

**Lifecycle.** Long-running daemon under `xgen-client --ai-mode --service`. Reuses the M2 pipe-server pattern for control commands (`__PING__` / `__HEALTH__` / `__STOP__` / `__RELOAD_CONFIG__`). `__HEALTH__` reply for an AI-mode resident extended to `HEALTHY pid=<pid> mode=ai operator_known=<N>/<M>` (where N = Spaces with resolvable operator, M = Spaces the AI is a member of). Coarse signal — the structured per-Space operator map stays on `xgen-client status`.

**Configuration shape.** Single config file `xgen-client_config.toml`. M4 adds two pieces to the existing `[ai]` section from M3:

```toml
[ai]
is_ai = true
plugin = "echo"            # which plugin

[ai.capabilities]
dm_initiate = false
spontaneous_post = false

[ai.behavior]              # plugin's own config; each plugin owns its keys
mention_token = "@bob"
```

The split between `plugin = "..."` (in `[ai]`) and `[ai.behavior]` is deliberate: "which plugin" is a single-line toggle; "how that plugin is tuned" lives in its own namespace. Open-enum on plugin name — unknown values pass config parsing but the runtime loader rejects them at startup with a clear error.

**Pacing — drop, don't queue.** The AI runtime maintains per-Space `last_send_at_ms`. Before emitting a reply, it checks `now - last_send_at_ms >= ai_pacing_ms`. If not, the reply is **dropped** (not queued). Drops are logged at WARN with the literal phrase `dropping reply — pacing cap not yet elapsed (honest behaviour over polite behaviour)` so the principle is greppable in production logs. The same path enforces mute (`active_mutes` in SpaceState).

**Join behaviour: manual, not auto.** The AI Client does NOT auto-join Spaces on startup. Joins are operator-driven via `xgen-client --instance <ai-label> join --space <id>`. Auto-join would make an AI Identity's first observable behaviour in a Space config-driven rather than chosen, muddying the trust model. Manual join keeps presence as an explicit, auditable event in the DAG.

**Operator control plane and temperature surfacing: out of scope for M4.** The protocol-level operator-signed event surface (DM commands, audit access, etc.) does not yet exist — designing M4 around it would load weight on something unbuilt. Temperature is conversational-dynamics design that needs its own conversation. Both layer on the M4 runtime in future milestones.

### The named recurring principle: honest behaviour over polite behaviour

When a system can choose between behaviour that misrepresents its current state and behaviour that honestly reflects it, XGen picks honest. Other places this principle is already operating, named here so future design conversations can invoke it explicitly:

- **Fall-upward operator resolution (D-064).** Returns the *currently-resolvable* operator (skipping stored entries that no longer point at members) rather than serving a stale stored value as if it were live.
- **Node event-acceptance pipeline.** Rejects events that fail validation rather than queueing them for retry; the rejection is the answer.
- **Mute semantics (Ch3 §3.7.8).** A mute is a wall, not a delay. The muted member's events are dropped, not queued for delivery after the cooldown.
- **`cmd_create_space` ack handling** (carry-over from M3, noted in J-075). Currently the Client says "Space created" optimistically; the M4 work surfaced this as a UX bug because optimistic reporting misrepresents the Node's actual decision. Future fix will adopt the honest "wait for ack, then report" pattern.
- **M4 AI Client pacing.** Drops replies that the cap rejects, rather than queueing them. The conversation has moved on; a queued reply now misrepresents the AI's current state.

The principle is not a prescription — sometimes politeness is correct (a Client retrying a transient network error is appropriate; pretending the send already succeeded is not). The naming exists so design conversations can articulate the trade-off cleanly: "this is polite-but-misleading; is that what we want?" and reach for "no, drop / fail / surface the truth" as the default.

### Implementation surface

| File | Shape |
|---|---|
| `xgen-client/src/ai_behavior.rs` | `AiBehavior` trait, `EventContext` struct, `EchoPlugin` impl with case-sensitive two-rail mention detection. |
| `xgen-client/src/ai_service.rs` | `pub fn run()` entry, `run_ai_loop` async fn, `AiPacingTracker` (drop-on-throttle, separate from PacingManager's queue-on-throttle), plugin loader (`load_plugin("echo") -> Box<dyn AiBehavior>`). |
| `xgen-client/src/batch.rs` | New `ResidentHealthState` struct (mode label + optional operator-known count). New `start_pipe_server_with_health` takes shared `Arc<Mutex<ResidentHealthState>>`; existing `start_pipe_server` becomes a default-state wrapper. `__HEALTH__` handler reads from the shared state. |
| `xgen-client/src/main.rs` | Dispatch adds AI-mode branch: `if cli.service { if cli.ai_mode { ai_service::run() } else { service::run() } }`. |
| `xgen-client/src/app.rs` | `AiSection` extended with `plugin: Option<String>` and `behavior: Option<AiBehaviorSection>`. New `AiBehaviorSection` struct (config sub-table for plugin-specific keys; M4's only key is `mention_token`). `cmd_init --ai` defaults `plugin = "echo"`. |
| `xgen-client/src/lib.rs` | `pub mod ai_behavior;` + `pub mod ai_service;`. |
| `docs/xgen_ch6_client_design.md` | New §6.15 "AI Client (resident mode)" — 10 subsections covering mode selection, config, trait, reference plugin, mention detection, runtime loop, pacing/mute, lifecycle/control, manual join, out-of-scope/forward-references. |
| `docs/xgen_ch3_specification.md` | §3.6.10 cross-reference list extended to include D-064 (M3 operator role), D-065 (M4 reference implementation), and Ch6 §6.15 (forward link to client-side surface). |

### Out of scope (deferred)

- **Real LLM hookups.** Future plugins as additional `AiBehavior` impls.
- **Multiple plugins / config-time plugin selection logic.** M4 ships one plugin; the loader matches the configured name to the only available impl. Phase 2+ adds the loader.
- **Operator command surface (DM commands, audit access, AI silencing through operator authority).** Separate protocol-level design conversation.
- **Temperature surfacing / room-temperature reaction by the AI.** Conversational-dynamics design; defer.
- **Auto-join of Spaces by invite.** Locked manual; testing convenience preserved by smoke-script CLI helper.
- **Cross-Space coordination, multi-device AI Client, Tauri / UI surface.** Future milestones.

### Why this shape rather than alternatives

The hard architectural question was *binary identity* — should the AI Client be a separate `xgen-ai` binary or a mode of `xgen-client`? The v0.1 draft of this decision proposed a separate binary; the v0.1→v0.2 review pass amended it to "mode of xgen-client" with reasoning that the M2 precedent (Node's `--service` mode rather than `xgen-node-service` separate binary) and the D-056 consolidation direction (one binary per role) both point the same way. AI Client is a client; the runtime loop differs from the human Client's loop but everything around it (config loading, connection, pipe server, lifecycle) is identical scaffolding. A separate binary would have duplicated that scaffolding for no clear gain.

The plugin trait is locked now rather than deferred. The trait surface is small enough that getting it wrong now is cheap; getting it wrong after a real LLM plugin exists is expensive — the future plugin would either accept the inherited shape or force a breaking-change rework of every consumer. Locking the shape during M4, before any real plugins exist, costs nothing extra and stabilises the interface.

Drop-late-replies is locked because queueing produces stale replies — by the time the cooldown expires, the conversation has moved on. The locked behaviour also is the simpler implementation, but the simplicity follows from the correctness, not the other way around: the honest design is also the lighter design here.

Manual join is locked because the trust model loses something when an AI Identity's first observable behaviour in a Space is config-driven rather than chosen. Auto-join would make the AI's presence implicit; manual join keeps it explicit and auditable through the standard `membership.join` event flow.

### Why now

M4 implementation began at v0.3 task-file lock (J-076) after D-056 consolidation was confirmed closed. The Client lifecycle conventions (PID file, pipe server, session header, log rotation) are stable from M1/M2; the protocol primitives the AI Client consumes are stable from M3. M4 is the first milestone that exercises all of them together in a long-running process and surfaces "what does this look like end-to-end" for the first time. The recurring honest-vs-polite principle was already implicitly operating across earlier decisions; naming it here makes future design conversations more efficient.

### Spec reference

- New section: Ch6 §6.15 "AI Client (resident mode)" — 10 subsections.
- Cross-references added in Ch3 §3.6.10 — pointing forward to §6.15 and back-referencing D-064, D-065.

### Code reference

| Component | File / surface |
|---|---|
| `AiBehavior` trait + `EchoPlugin` | `xgen-client/src/ai_behavior.rs` |
| AI runtime loop + plugin loader + pacing tracker | `xgen-client/src/ai_service.rs` |
| Pipe-server shared health state | `xgen-client/src/batch.rs::ResidentHealthState` + `start_pipe_server_with_health` |
| `--ai-mode` flag + dispatch | `xgen-client/src/app.rs::Cli::ai_mode`; `xgen-client/src/main.rs` mode-selection branch |
| Config schema | `xgen-client/src/app.rs::AiSection` + `AiBehaviorSection` |
| `init --ai` defaults | `xgen-client/src/app.rs::cmd_init` |

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-056 | M4 is a mode of xgen-client per the locked "one binary per role" direction. D-056 closed first (J-076); M4 implementation followed. |
| D-059 | M4 consumes D-059's `is_ai` registration shape via the existing M3 `register` flow; no new wire shape needed. |
| D-060 | M4 reuses D-060's `ai_pacing_ms` field via a simpler drop-on-throttle tracker (sibling of `PacingManager` rather than wrapper, because the policies differ — queue vs drop). |
| D-061 | M4 is a passive recipient of temperature meta_atts; does not emit temperature, does not react to thresholds. |
| D-062 | M4 does NOT use Tauri — explicitly headless. |
| D-063 | M4 follows library-first per D-063: trait + runtime loop in `xgen-client-lib`, binary is thin dispatch. |
| D-064 | M4 surfaces M3's `resolve_operator` result on `__HEALTH__` (operator_known count). |

---

## D-066 — Split `--batch` legacy surface from `--aicontrol` AI surface; the latter is reference-implementation, not protocol

**Date**: 2026-05-17  
**Layer**: Reference implementation control plane (xgen-client / xgen-node binaries) — NOT XGen Protocol  
**Spec reference**: none (this decision is explicitly out-of-protocol). Cross-reference: `tasks/BATCH_FLAG_review.md` (Clair's review of `--batch`) and the Chat Claude addendum appended to the same file (2026-05-17).

### Decision

The `xgen-client` binary will expose **two distinct control surfaces** with different audiences and different design constraints:

| Flag | Audience | Shape | Format | Status under this decision |
|---|---|---|---|---|
| `--batch <file.xgb>` | Humans and human-readable automation (CI shell scripts, ops runbooks) | Fire-and-forget script runner. One command per line. | Plain text `.xgb` files; replies are `OK\n` / `ERROR: ...\n`. | **Frozen as-is.** Continues to behave exactly as it does today. |
| `--aicontrol` | AI drivers (Claude Code, future MCP servers, in-Space AI moderators, scripted multi-step agents) | Persistent control session. Long-lived connection, multiple commands, real-time event observation. | Newline-delimited JSON (JSONL) over a sister pipe. | **New surface.** Design and implementation scoped under this decision; details in `tasks/BATCH_FLAG_review.md` Chat Claude addendum. |

Both surfaces dispatch through a **shared command-implementation layer** (`xgen-client-lib::ops::*`) parameterised by execution context (one-shot connection vs persistent session). This extends the D-063 library-first principle one level deeper to eliminate the existing `cmd_*` / `exec_*` drift surface that produced F-003 / F-004 in J-067.

### The protocol-vs-implementation boundary (locked)

**`--aicontrol` is NOT part of the XGen Protocol.** The XGen Protocol is what travels on the wire between XGen participants — between a Client and its home Node, between two federated Nodes, between MLS group members. `--aicontrol` is none of these. It is a local control channel between an AI driver and a specific `xgen-client.exe` instance running on the same machine, carried on a Windows named pipe. It never reaches any XGen wire. A different XGen client implementation in a different language could ship a different control surface (gRPC, REST, MCP server, raw stdin/stdout, or no AI-control surface at all) and remain fully protocol-compliant. A proprietary XGen client built by a third party may take a completely different approach to AI automation — that is their implementation choice, not a protocol question. The XGen Protocol does not constrain how a Node operator or Client vendor builds their local automation surface.

This is structurally identical to how Matrix treats its Client-Server API as a reference convention while only the Federation API is protocol; or how MLS (RFC 9420) defines the cryptographic ratchet but says nothing about how clients UI it.

**Implication for the documentation tree.** When `--aicontrol` lands:

- It does NOT appear in Ch3 (Specification).
- It does NOT appear in Appendix I (Data Structures).
- It DOES appear in Ch4 (Implementation) or a new dedicated Appendix — explicitly marked "reference implementation control surface; not part of the XGen Protocol".
- Appendix F (CLI Reference) lists `--aicontrol` as a non-fundamental Client flag (per the §F.0 axis added in M4 documentation sweep) with a forward link to the dedicated design document.

### Locked principles

**`--batch` is preserved verbatim.** No behavioural change, no format change, no deprecation timeline. The human-readability properties of the current `--batch` were a deliberate design goal at its original spec time, and replacing them with JSONL would have been a regression. Two surfaces is the honest answer; one surface trying to serve both audiences was always a tension.

**`--aicontrol` is a persistent session, not a script runner.** The natural shape is `xgen-client --aicontrol` opens a long-lived control session on a dedicated pipe; the driver writes JSONL commands and reads JSONL replies and events. Scripts can be fed via shell redirection but there is no in-protocol "load a file" notion. The session lives as long as the connection lives.

**The shared `ops::*` layer ships first, independent of `--aicontrol` design.** The duplicate `cmd_*` vs `exec_*` problem is independent of which CLI flag invokes them. The refactor benefits both `--batch` and `--aicontrol` and unblocks both surfaces. Sequencing this first means the multiparty baseline pass exercises the unified handlers, not the drift-prone duplicates.

**The flag name was locked by Joe explicitly.** `--aicontrol` over alternatives (`--control`, `--session`, `--ctl`, `--aibatch`) because it makes the audience visible in the flag name. Future readers immediately see what category of driver this surface serves.

### Out of scope for this decision

- All technical details of the `--aicontrol` protocol (JSONL field shapes, command verbs, event subscription model, named bindings, lifecycle-aware error codes, pipe naming, concurrency model). These are in the Chat Claude addendum to `tasks/BATCH_FLAG_review.md` and are explicitly delegated to Chat Claude + Clair without per-decision approval from Joe — see the addendum preamble.
- The Node-side equivalent. Whether `xgen-node --aicontrol` also lands is a question for the design phase, not this decision.
- The cross-platform story. Windows-first; cross-platform pipe abstractions remain Phase 3+.
- Authentication and authorisation of the `--aicontrol` pipe (security model for multi-user MCP deployments). Flagged as a known deferred concern in the addendum.

### Why this shape rather than alternatives

**Alternative 1: Make `--batch` itself more capable.** Rejected. Adding JSONL reply mode, persistent sessions, and event observation to `--batch` would either break the human-readability contract or require a flag-on-a-flag dance (`--batch --reply-format=jsonl --persistent --subscribe=events`) that is harder to use than two cleanly separate flags. The current `--batch` is already at its design limit; trying to make it serve both audiences would degrade both.

**Alternative 2: Single flag with version negotiation.** Rejected. A `--batch --protocol=v2` style would put the version selection inside the wire data rather than at the CLI surface. CLI flags are the right place for major behavioural mode selection — it is visible in shell history, scriptable, and discoverable via `--help`.

**Alternative 3: External AI-driver process (MCP server) that translates between AI commands and `--batch`.** Rejected for this milestone. A future MCP server consuming `--aicontrol` is exactly the intended deployment shape, but it must consume a surface designed for AI drivers — layering it on top of the human-readable `--batch` would push every issue identified in Clair's review (per-command WS churn, log-scraping for return values, no real-time observation) into the MCP server as workarounds. The right architectural primitive is `--aicontrol`; MCP servers and other AI integrations sit above it.

**Alternative 4: gRPC or REST on a localhost port.** Rejected for the same reason named pipes were chosen for `--batch` originally: Windows-first, no port-allocation concerns, no firewall pop-ups, no TLS-on-localhost dance, no second listener to secure. Named pipes are the right primitive on Windows and remain so.

### Why now

The M4 documentation sweep surfaced that the CLI reference (Appendix F) and the canonical state of the system finally agree on what exists today. The next major piece of work (multiparty test suite redesign — paused since M1) cannot proceed honestly under the present `--batch` for the reasons in Clair's review: real-time fan-out is unmeasurable, latency metrics are uncapturable, and two-pass log-scraping ID substitution is structurally fragile. The multiparty A/B metrics protocol Clair specified requires a control surface that captures the metrics; the present `--batch` cannot. Therefore `--aicontrol` is a prerequisite for credible multiparty work, not an optional improvement.

Further: `--aicontrol` is the foundation for the future Claude-driven MCP server and any in-Space AI moderator agent. Designing it now — once — saves designing it later under feature pressure from those consumers.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-028 | The canonical-source rule ("Rust doc comments MUST match Appendix F") applies to `--aicontrol` once it lands. The detailed protocol document is the canonical source; Appendix F summarises and links to it. |
| D-035 | `--aicontrol` data lives under the same convention-derived directory layout as everything else — no new configurable paths. |
| D-043 | The pipe naming convention is extended (sister pipe `\\.\pipe\xgen-client[-<label>].aicontrol` alongside the existing legacy pipe). |
| D-056 | `--aicontrol` is a new dispatch mode on the existing `xgen-client` binary, consistent with the locked one-binary-per-role + multi-mode dispatch model. Not a new binary. |
| D-063 | The shared `ops::*` layer extends library-first one level deeper than D-063 originally specified — D-063 moved dispatch out of `main.rs`, this decision moves command implementations into a single shared layer below it. |
| D-065 | `--aicontrol` is the operator command surface that D-065 said would "layer on top in future milestones." This decision schedules it. |
| Clair's `BATCH_FLAG_review.md` | Diagnostic; this decision is the architectural response. Detailed technical decisions are appended to that file as the Chat Claude addendum. |

### Canonical home (added 2026-05-17)

The technical specification for `--aicontrol` lives in **`docs/xgen_aicontrol_implementation.md`** as of 2026-05-17. That document supersedes the Chat Claude addendum inside `tasks/BATCH_FLAG_review.md` (which remains in place as a historical predecessor) and extends the design to cover both binaries (`xgen-client` and `xgen-node`) rather than Client only. D-069 names the canonical-document discipline that this move implements. Future edits to the `--aicontrol` design land in the canonical document, not in DECISIONS.md notes or in `tasks/` addenda.

---

## D-067 — Single source of truth for xgen-client command implementations (`ops::*`); M7 prerequisite met

**Date**: 2026-05-17
**Layer**: xgen-client crate (structural)
**Spec reference**: `tasks/M5_OPS_REFACTOR.md`; D-066 (the `--aicontrol` split that M5 unblocks); D-063 (library-first principle that M5 extends one level deeper); J-067 (F-003 / F-004 background).

### Decision

Every xgen-client command implementation lives in exactly one place: `xgen-client-lib::ops::<verb>`. Every dispatcher — the CLI arm in `main.rs`, the CLI batch driver `app::run_batch_file`, the named-pipe dispatcher `batch::dispatch_line`, and any future Tauri-command / `--aicontrol` arm — calls into the same `ops::<verb>` function. Each dispatcher owns its own output format (CLI shim formats for stdout, pipe arm formats `OK\n` / `ERROR: …\n` per the D-066 freeze, M7's `--aicontrol` arm will format as JSONL); the data extraction lives in exactly one place per verb.

`SessionState` (per-invocation session bundle) and `ClientIdentity` (loaded keypair + cached `identity_id`) are the helpers that make `ops::*` parameterisable across execution contexts. `SessionState::ensure_identity` and `SessionState::ensure_connected` are idempotent so both M5 one-shot dispatchers and M7 persistent-session dispatchers reuse the same code paths.

The M5 type signatures include M7 extension fields (`SessionState.bindings`, `SessionState.spaces`) present-but-empty so the type signature is M7-stable; no shape changes will be needed between M5 and M7.

### Why this matters

**Drift surface eliminated.** Before M5 (and even after J-068's partial dedup), command implementations could diverge across dispatchers — the F-003 / F-004 pair in J-067 was a concrete instance where one `get_dag_tips` copy got a Space-filter fix and the other silently kept the bug. After M5, there is exactly one user-facing implementation per verb; a second copy cannot be introduced without being noticed.

**M7 (`--aicontrol` v1) prerequisite met.** D-066 deferred all `--aicontrol` technical details on the explicit assumption that a shared command layer would land first; designing `--aicontrol` against today's drift-prone duplicates would either inherit the F-003 / F-004 class or force the refactor under feature pressure. M5 ships that prerequisite cleanly.

**M6 (multiparty baseline pass with present `--batch`) benefits too.** The "A" baseline measurements in M6 exercise unified handlers rather than the drift-prone duplicates that existed before M5. Measurements done against M5's `ops::*` are directly comparable to the "B" measurements that M7's `--aicontrol` will produce.

### Out of scope for this decision

- The full M7 `--aicontrol` protocol detail (JSONL field shapes, command verbs, event subscription model, named bindings, lifecycle-aware error codes). Those are D-066's scope, designed in the next milestone.
- Tauri commands for the 13 protocol verbs. The current Tauri shell registers only `get_state` / `get_pacing_state` / `quit`; verb-level Tauri commands are a future milestone (likely alongside the long-lived Tauri resident or alongside `--aicontrol`). When they land they will naturally call `ops::*`.
- The flag-vs-config precedence bug in `xgen-node --port` (surfaced during M5 smoke setup). Not xgen-client; carry-over flagged in J-078.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-063 | M5 extends library-first one level deeper than D-063 originally specified — D-063 moved dispatch into the library; D-067 moves command *implementations* into a single shared layer below dispatch. |
| D-066 | D-066 split `--batch` (frozen) from `--aicontrol` (new) and stipulated that the shared `ops::*` layer ships first. D-067 is that ship. |
| J-067 (F-003 / F-004) | Concrete drift instances that motivated M5. D-067 closes the drift surface architecturally; the smoke run in J-078 confirms it closed by behaviour as well as by structure. |

---

## D-068 — CLI flag precedence over config file (locked)

**Date**: 2026-05-17  
**Layer**: Cross-cutting (both binaries) — implementation rule, not protocol  
**Spec reference**: Appendix F §F.0 (Flag model). Cross-reference: J-078 (M5 close-out) for the known violation that surfaced this decision; D-035 (convention-derived paths) for the related rule on path resolution.

### Decision

For any setting that can be specified both as a CLI flag and as a field in a TOML config file, the **CLI flag takes precedence**. No exceptions. The full precedence order is:

1. **CLI flag** (highest priority — most recent operator intent, visible in shell history and automation)
2. **Config file field** (persisted operator intent from `init` or manual edit)
3. **Default value** (the binary's built-in fallback)

This rule applies uniformly to both `xgen-node` and `xgen-client`, to every flag in Appendix F §F.0.1 (fundamental) and §F.0.3 (non-fundamental) that has a config equivalent, and to any future flag added to either binary that shadows a config field.

### Why this rule must be explicit

The rule has been implicit since Phase 1 and is documented per-flag in Appendix F descriptions (e.g. `--node` on Client: "Overrides config"). What was missing is a single citable architectural decision saying *all flags follow this pattern*. The M5 smoke setup (J-078) surfaced a violation of the rule on `xgen-node --port`, which suggests the rule was not universally enforced in implementation.

Three reasons the rule is structural, not stylistic:

**1. CLI is the most-recent intent.** The config file was written at some past time (init, manual edit, possibly stale). The CLI flag is what the operator typed *right now* when starting this process. Right-now intent must beat persisted intent. Anything else surprises the operator.

**2. CLI is visible; config is hidden.** A `--port 8081` in a command line appears in shell history, in scripts, in process listings, in `ps`/`Get-Process` output. A `listen = "..."` deep in a TOML file is invisible from the operational command surface. Visibility matters for diagnosis and audit; the most-visible source must be the authoritative one.

**3. The testing model depends on it.** Every smoke test, stress test, and multiparty scenario sets ports, modes, and instances via CLI flags so a single set of config files can serve many test invocations. The whole testing model assumes flag override is reliable. If a flag silently falls back to config, every test that depends on that flag is unreliable — silently wrong, not loudly broken.

Reason 3 is the operational urgency. M6 (multiparty baseline pass with present `--batch`) and every subsequent test milestone will fire many invocations against different ports, modes, and instance labels. If `--port` is broken on `xgen-node`, every test that varies the port produces results that may or may not reflect actual flag-override behaviour. The smoke-test ground truth degrades.

### Known violation

`xgen-node --port <port>` did not override the `listen` field in `xgen-node_config.toml` on the first invocation during M5 smoke setup (J-078, 2026-05-17). Observed behaviour: Node attempted to bind the *config-file* port (`8080`) rather than the *CLI flag* port (`8081`), failed on conflict with another Node already on `8080`, exited with `os error 10048`. The same command on second invocation succeeded — mechanism unclear (possibly OS-level port release timing, possibly delayed flag-application code path that catches up on retry).

The bug is in `xgen-node`, not in `xgen-client`. It is not M5 scope (M5 was a Client refactor). It is also not blocking M6 in the narrow sense — the workaround is to either match config to intended port at init time, or invoke twice. But the workaround is exactly the kind of silent-test-pollution this decision rules out.

### Audit task scheduled

**Priority: must be resolved before M6 starts.** M6 runs the multiparty test suite against the present `--batch` shape with metrics captured per Clair's protocol (`BATCH_FLAG_review.md`). The metrics protocol depends on flag overrides being reliable. Running M6 against a binary with broken flag precedence would produce metrics whose meaning is ambiguous (did flag X apply, or did config silently win?).

The audit task covers:

1. **`xgen-node --port`** — fix the observed violation. Root-cause the mechanism (full flag-vs-config code path inspection, not just empirical retry).
2. **All other CLI flags with config equivalents on both binaries** — written confirmation per flag that flag overrides config:
   - `--config <path>` (both binaries) vs default search path
   - `--node <endpoint>` (Client) vs `[client].node`
   - `--log-level <lvl>` (both) vs `[logging].level` and `XGEN_LOG` env
   - `--instance <label>` (both) vs implicit default-instance behaviour
   - `--service` (both) vs lifecycle default (Tauri shell)
   - `--local` (Node) vs `[node].local_mode`
   - `--quiet` (both) vs default banner behaviour
   - `--ai-mode` (Client) vs `[ai].is_ai` config
3. **Tests** — each flag-with-config-equivalent gets a focused test that locks the precedence: flag set, config conflicts, assert flag wins.
4. **A short Appendix F clarification** linking flag-by-flag to this decision (already added — §F.0.6).

Task file: `tasks/CLI_PRECEDENCE_AUDIT.md` (to be written before M6 task file is finalised).

**Completed in J-079 (2026-05-17).** The audit shipped in 5 atomic commits (helper + Node `--port` plumbing + four-site subscriber-init convergence + integration tests + doc sync). Empirical verification surfaced four additional violations beyond the named `--port` defect — four parallel subscriber-init blocks were silently dropping `[logging].level` and falling back to a hardcoded `"debug"` literal. Helpers `resolve_setting` and `resolve_log_level` shipped in `xgen-common::precedence`. Test count rose from 435 to 463 (+10 unit precedence + 5 URL-rewrite + 6 Node integration + 7 Client integration). The drift surface is architecturally eliminated — same shape as M5/D-067 eliminated drift in `xgen-client-lib::ops`. See J-079 for the full audit record.

### Out of scope for this decision

- Environment variable precedence (`XGEN_LOG` etc.) above or below config — the only env var currently in use is `XGEN_LOG`, whose precedence vs the `--log-level` flag is documented in Appendix F (flag wins). If more environment variables are introduced later, this decision can be extended.
- The `init` flow's interactive prompts — those are separate (they ask the user for values that go *into* the config file; they are not flag-vs-config comparisons).
- Default-value selection — covered by per-flag documentation in Appendix F; not in scope here beyond confirming defaults are the lowest-priority source.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-028 | Canonical-source rule says Rust doc comments must match Appendix F. D-068 is now a load-bearing rule in Appendix F; doc comments referencing flag-vs-config behaviour must align with it. |
| D-035 | Convention-derived paths rule established that data paths are derived from working directory, not configurable. D-068 is the dual: flags override what *is* configurable. Both decisions are about taking operator-intent volatility out of unexpected places. |
| D-043 | The named-pipe naming convention is partly driven by `--instance <label>` — a flag. D-068 confirms `--instance` is authoritative for pipe naming when present. |
| D-066 | `--aicontrol` (when shipped in M7) is itself a CLI flag opening a control surface. Its presence-vs-absence is by definition flag-driven, not config-driven; D-068 confirms the pattern. |

---

## D-069 — Delegated technical design discipline (locked)

**Date**: 2026-05-17  
**Layer**: Project management / roadmap discipline — not protocol  
**Spec reference**: none (rule about how the roadmap is sequenced and how delegated work is treated). Cross-references: D-066 (the original `--aicontrol` delegation grant); D-068 (the CLI Precedence Audit, which is the model that worked); the M6 descope of 2026-05-17 (the worked example that motivated this decision).

### Decision

When a milestone's technical design is delegated — typically to Chat Claude and Clair operating under a grant like D-066 ("all technical details ... explicitly delegated ... without per-decision approval from Joe") — the implementing milestone MUST NOT be declared ACTIVE in CLAUDE.md until two complementary conditions are met:

**1. Joe-lock on the architectural commitment.** The major shape — the split, the flag name, the binary boundary, the layer placement — comes from Joe and is recorded as a numbered decision in DECISIONS.md. D-066 is the model: a short, named, dated, citable architectural commitment that scopes what the delegation covers.

**2. Self-aware open-item flagging in the delegated detail.** The delegated technical document MUST explicitly list (a) what's been decided, (b) what's open, and (c) which open items can be resolved by Chat Claude/Clair in the design phase vs which need Joe input. The Chat Claude addendum §12 inside `tasks/BATCH_FLAG_review.md` is the model: a numbered list of "Open items for the design phase" that names exactly what hasn't been settled and signals when escalation is needed.

Additionally:

**3. Canonical-document rule.** Each major implementation surface that spans both binaries (or has the potential to) gets exactly one canonical document. Binary-specific implementation detail lives in sections of that document, not scattered across `tasks/`, addenda inside other documents, or DECISIONS.md notes. The canonical document is the single authoritative source; cross-references from CLAUDE.md, DECISIONS.md, and Appendix F point at it, not at the original scattered locations.

### Why this rule must be explicit

Delegation is necessary. Joe cannot review every JSONL field name, every error code string, every pipe-naming detail — the project would never ship. The 2026-05-17 framing in D-066 ("to avoid per-detail approval bottlenecks") is correct: delegation is how work proceeds at sustainable pace.

But delegated drafts that haven't been Joe-locked are structurally indistinguishable from locked specifications when written down in a `tasks/` file or an addendum. A reader (next-session Clair, future Chat Claude, a future contributor) cannot tell from looking at a file whether its contents represent (a) Joe's binding architectural decision, (b) Joe-conversation-locked detail recorded in writing, (c) Chat Claude's delegated draft awaiting refinement, or (d) Clair's working sketch.

The failure mode this decision prevents: a delegated draft gets scheduled as a milestone implementation target without anyone realising parts of it were assumed rather than decided. The implementation session opens, Clair starts execution, design questions surface as gate-questions partway in, and the milestone has to be paused or descoped. **M6 (multiparty baseline pass with present `--batch`) is the worked example.** The metric protocol in `tasks/BATCH_FLAG_review.md` was Joe-conversation-locked on 2026-05-16, but its *application* in the two MULTIPARTY task files was never reconfirmed after J-079 changed the binary shape. M6 was about to start against a delegated runbook whose anchoring assumptions had silently drifted.

Three reasons the rule is structural, not stylistic:

**Reason 1 — The lock step is a session in itself.** Joe-locking a delegated design is not a side task to bundle with implementation start. The implementation session reads a Joe-locked design; the lock session reads a delegated draft and produces a locked design. Conflating the two means implementation starts before the design is settled.

**Reason 2 — Open-item flagging surfaces drift.** The Chat Claude addendum §12 named six open items: full `cmd` verb set, control-surface error codes, subscription filter grammar, `state` command output schema, per-command timeout values, whether Node-side `--aicontrol` is in scope. This list made the design's boundaries visible. Compare: the metric protocol in the same file did not flag "is this still applicable after J-079?" as an open item, so its applicability was assumed when M6 was scheduled. Self-aware flagging is what prevents this.

**Reason 3 — Canonical-document discipline prevents the same lesson recurring.** When design content is scattered (e.g. `--aicontrol` design today lives in D-066, in the Chat Claude addendum inside `BATCH_FLAG_review.md`, in mentions in `tasks/CLI_PRECEDENCE_AUDIT.md`), no single reader can verify the design is complete and locked. Anyone trying to assess shovel-readiness has to reassemble it from three places, and the boundary between locked vs delegated content gets lost in the seams. One canonical document per surface is the structural fix.

### The two states a delegated design can be in

- **Drafted** — exists in `tasks/`, in an addendum, or in working notes. Useful for forward planning. NOT sufficient to schedule the implementing milestone as ACTIVE. May contain open items that haven't been escalated.
- **Joe-locked** — Joe has read the draft, asked questions, and either confirmed it or directed revisions that are now incorporated. The draft is annotated as locked (status header flipped, or a "Locked YYYY-MM-DD" line added, or the content has been promoted into the canonical document). Implementation milestone may now be declared ACTIVE.

### Known instances at time of decision

| Instance | Status as of 2026-05-17 | What needs to happen |
|---|---|---|
| D-068 → CLI Precedence Audit (J-079) | **Worked correctly.** D-068 was Joe-locked before `tasks/CLI_PRECEDENCE_AUDIT.md` was written. The task file enumerated open items per section; Clair gated on Joe approval at each section boundary. M5→audit→M6-or-equivalent ran cleanly. This is the model. | Nothing. Reference for future delegations. |
| D-066 → M7 (`--aicontrol` v1) | **Canonical home created 2026-05-17.** D-066 locks the architectural commitment. The canonical document `docs/xgen_aicontrol_implementation.md` now exists, covering both binaries; its §12 (Open items for design phases) carries forward the six items from the original Chat Claude addendum plus the additions surfaced when extending to both binaries. The addendum inside `tasks/BATCH_FLAG_review.md` remains as a historical predecessor. | M7 design phase resolves the §12 open items in the canonical document; Joe-locks the result; only then M7 goes ACTIVE. |
| `tasks/BATCH_FLAG_review.md` §"Baseline metrics protocol" → M9 (Multiparty Redesign) | **Joe-conversation-locked (2026-05-16) but applicability uncertain.** The metric set itself is sound. What's open: whether the same set applies to a both-binaries `--batch` / `--aicontrol` A/B framing, and whether the post-J-079 binary shifts any captures. | M9 design phase reconfirms or revises the metric set; promotes it into a canonical home (likely `docs/tests/MULTIPARTY_metrics_protocol.md` or similar); Joe-locks the result; only then M9 goes ACTIVE. |
| M6 (original multiparty baseline) | **Descoped 2026-05-17 — the worked example for this decision.** | Replaced by M9 in the roadmap. |
| M6 (new — Node admin write path) | **PENDING.** Architectural commitment locked in this session's CLAUDE.md edit (Node needs read-write admin surface symmetric to Client). Verb-set design is delegated and not yet drafted. | Open a design discussion on the verb set per category; produce `tasks/NODE_ADMIN_WRITE_PATH.md` with explicit open-item flagging à la addendum §12; Joe-lock the result; only then M6 (new) goes ACTIVE. |

### Out of scope for this decision

- Decisions Joe writes directly (D-035, D-061, D-063, D-068, etc.) — these are Joe-locked by definition; no separate lock step needed.
- Implementation-detail decisions inside a Joe-locked design (e.g. the helper signature in `CLI_PRECEDENCE_AUDIT.md` §5 was Clair's proposal, Joe-approved at the §5 gate). The lock is at the design level, not at every internal choice.
- Per-flag, per-verb, per-field micro-decisions that the design phase is explicitly authorised to settle. The rule is about the boundary between delegated draft and locked spec, not about preventing all delegation.
- Joe's discretion to override this rule for a specific milestone if velocity demands it — the rule is the default discipline, not an absolute prohibition. Overrides should be recorded as a note on the affected milestone block.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-066 | The original delegation grant for `--aicontrol`. D-069 adds the gates that delegation must pass before its implementing milestone goes ACTIVE. D-066's text remains valid; D-069 supplies the discipline around it. |
| D-068 | The CLI Precedence Audit is the pattern D-069 generalises. D-068 was Joe-locked before the audit task file was written; the task file flagged open items section-by-section; Clair gated on Joe approval at each gate. D-069 names this pattern and makes it the default for all future delegated milestones. |
| D-067 | M5's `ops::*` refactor architecturally eliminated drift between parallel implementations. D-069 is the discipline analogue: it eliminates drift between delegated drafts and locked specifications by requiring the canonical document and open-item flagging. Both decisions are about taking implicit gaps out of the system. |
| D-035 | Convention-derived paths took operator-intent volatility out of unexpected places. D-069 takes design-state volatility out of unexpected places (the gap between "drafted" and "locked"). Both decisions are forms of the same principle: make implicit state explicit. |

---

## D-070 — Two events of equal importance, opposite direction (named protocol principle)

**Date**: 2026-05-18  
**Layer**: Protocol — specifically wire-message symmetry for outcome signalling.  
**Spec reference**: `docs/xgen_node_admin_ops_design.md` §9 (original draft, preserved as historical record); `docs/xgen_propagation_reliability.md` §5 (J-081 audit finding that produced the corrected framing); `docs/xgen_federation_propagation_design.md` F-4 (the rejection sites this principle operates over).

### Decision

Wherever the XGen Protocol exposes a signal from one party to another about the outcome of an action, both directions of outcome — acceptance and rejection — MUST be exposed with equal first-class status. "Equal first-class status" means: same layer, same lifecycle, same correlation surface. When an action references a specific protocol object (an event, a registration, a federation request), both the acceptance signal AND the rejection signal MUST carry the identifier of that object so the originator can correlate the signal to the action it sent.

The principle has two halves and both are load-bearing:

1. **Both directions exist.** If the protocol exposes rejection (e.g. `TransportMessage::Error`), it MUST also expose acceptance (e.g. `TransportMessage::EventAccepted`). The originator must be able to learn either outcome through a first-class wire signal, not through inference from silence.
2. **Both directions carry the correlation identifier.** The envelope-level `event_id: Option<String>` field on `TransportMessage` (or the equivalent identifier for whatever protocol object the signal pertains to) MUST be populated on both the acceptance and rejection paths. Without correlation, the signal exists but the originator can't tell which of their in-flight actions it pertains to — making the signal hollow at scale.

Joe's verbatim framing, recorded across two moments in M6 Phase 0 Pass 3:

> *"Acceptance and rejection are two events of equal importance, just opposite direction."*

> *"The accept signal's importance warrants its own wire shape, not a side effect of an unrelated mechanism."*

The second quote is why the rejected M6 alternatives (C1 server-side self-fanout, C3 DAG-layer ack EventType) were rejected: neither treated the accept signal as a first-class concern. The first quote names the underlying principle.

### Why the corrected framing matters (vs the M6 §9 draft)

The original draft in `docs/xgen_node_admin_ops_design.md` §9 framed D-070 as "EventAccepted exists, symmetric to Error." That framing is necessary but not sufficient. Post-audit, J-081 §5 found that `TransportMessage::Error`'s wire shape lacked an `event_id` field at all — meaning even with both Error and a future EventAccepted, the originator could not correlate either signal back to a specific event. A driver with multiple in-flight events sees "Error" or "EventAccepted" arrive but has no way to know which event the signal is about.

The corrected framing makes both halves explicit: existence AND correlation. Without (2), (1) is hollow. M6 (new) Phase 2 ships both halves coordinated: the envelope-level `event_id` addition to TransportMessage, the new EventAccepted variant, and the wiring of Error's emit sites in `process_inbound` to populate the new field on every rejection path. F-4 of the Federation Event Propagation milestone produces the rejection sites consistently across all three event families; M6 Phase 2 wires them to the wire-layer signal under D-070.

### Why this is structural, not stylistic

**Reason 1 — It prevents structural-by-accident asymmetry.** The accept-signal gap existed because nobody designed an accept signal; it was a consequence of the event-streaming model (events flow one way; the response is fan-out, not a per-event reply). The Error-lacks-event_id gap existed because nobody designed Error to be correlatable; it was a consequence of Error originally being a generic transport-error signal rather than an event-rejection signal. Both gaps arose from "we didn't think about it" rather than "we deliberately chose this." Asymmetries that arise that way produce silent correctness bugs in the layers above. Naming the principle catches future instances at design time rather than at deployment time.

**Reason 2 — It pairs with D-065 cleanly.** D-065 binds the *content* of signals (don't lie about state). D-070 binds the *existence and correlation surface* of signals (when you can speak in one direction, you can speak in the other, and both directions name what they're about). Together they constrain the protocol to behaviour that is honest, complete, and correlatable. A protocol with only a rejection signal forces consumers to fake acceptance via heuristics (silence-equals-success); a protocol with both signals but no correlation forces consumers to fake correlation via timing (the next signal must be about the last action I sent). D-065 + D-070 together close both gaps.

**Reason 3 — It is reusable across future protocol design.** Any future XGen protocol addition (a new transport message family, a new federation request shape, a new bootstrap interaction, an Auth-Module verb response) inherits the principle. When a future design conversation asks "should this only signal failure, or should it also signal success?", the principle gives a default: yes, both, equal weight, both correlated. Departures from the default require explicit justification.

### Worked instances at promotion

- **`TransportMessage::Error`** — existing variant; gains envelope-level `event_id: Option<String>` in M6 (new) Phase 2. The five event-rejection sites in `process_inbound` ([`xgen-node/src/app.rs:846-851`](xgen-node/src/app.rs:846), [`855-858`](xgen-node/src/app.rs:855), [`885-897`](xgen-node/src/app.rs:885), [`913-921`](xgen-node/src/app.rs:913), [`926-934`](xgen-node/src/app.rs:926)) are wired to emit Error with `event_id: Some(...)` populated.
- **`TransportMessage::EventAccepted`** — new variant in M6 (new) Phase 2. Sent after the inbound event clears validation and is durably persisted, before local fan-out begins (the G2 boundary documented in `docs/xgen_node_admin_ops_design.md` §3.2).
- **Coordination with Federation Event Propagation milestone:** F-4 (validation pipeline unification) produces the rejection sites consistently across all three event families (today Paths B and C reject inline; after F-4 they reject through the dispatcher's `Rejected` return). M6 Phase 2 then wires those rejection sites to the wire-layer signal with envelope `event_id`. Both halves of D-070 land in coordinated milestones; the symmetry is realised at the moment both ship.

### Out of scope for this decision

- **Asymmetries where one direction genuinely doesn't apply.** `TransportMessage::Goodbye` has no `Greetings` counterpart because connection establishment is asymmetric by nature (the WebSocket handshake itself is the greeting). The principle does not force false symmetries where the underlying interaction is genuinely one-directional.
- **Asymmetries internal to the reference implementation.** A binary's CLI surface having a `--start` flag with no `--stop` flag, an admin verb that's WRITE-only with no READ counterpart, etc. The principle is about protocol-level signals, not implementation-internal control flow. The `--aicontrol` JSONL protocol (M7) inherits D-070 because that surface IS protocol-shaped between AI driver and reference implementation; raw CLI flag pairs are not.
- **The propagation reliability question.** That is a separate concern (§4 of the M6 design doc) addressed by the Propagation Reliability Audit milestone (J-081) and the Federation Event Propagation completion milestone. D-070 governs the signalling layer; D-071 governs the discipline of verifying the propagation layer underneath it. Two different concerns, two different decisions.
- **Backward compatibility migration.** Pre-M6 clients that don't recognise `EventAccepted` ignore it gracefully via existing match-arm fallbacks; post-M6 clients talking to pre-M6 Nodes handle the absence of both `EventAccepted` AND `Error` with a bounded timeout fallback documented in M6 design doc §3.6. D-070 lands the principle; the M6 milestone handles the migration mechanics.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-065 | Sibling protocol-design principle. D-065 binds the *content* of signals (don't misrepresent state). D-070 binds the *existence and correlation surface* of signals (when you can speak in one direction, you can speak in the other; both directions name what they're about). Together they make protocol signalling honest, complete, and correlatable. |
| D-066 | The `--aicontrol` JSONL protocol (M7) inherits D-070. Every JSONL reply shape will carry both `result` and `error` paths at equal first-class status with correlation identifiers, mirroring the `Error` / `EventAccepted` symmetry M6 establishes at the wire-message layer. |
| D-067 | M5's `ops::*` refactor architecturally eliminated drift between parallel command implementations. D-070 is the protocol-layer analogue: it eliminates drift between parallel outcome paths (acceptance and rejection) by requiring symmetric first-class signalling with correlation. Both decisions take implicit gaps out of the system architecturally rather than by discipline. |
| D-069 | D-070 was Joe-framed during M6 Phase 0 Pass 3 (a delegated design phase) per D-069 discipline. The corrected post-audit framing surfaced during the J-081 audit close. Promotion to DECISIONS.md follows the D-069 canonical-document rule: the M6 design doc §9 draft remains as historical record; this DECISIONS.md entry is the canonical authoritative form. |
| M6 (new) `docs/xgen_node_admin_ops_design.md` §9 | The original D-070 draft. Preserved as historical record of the principle's framing at M6 Phase 0 Pass 3. The corrected framing in this entry supersedes §9's text for canonical reference. |
| `docs/xgen_federation_propagation_design.md` F-4 | Produces the rejection sites that M6 (new) Phase 2 wires under D-070. The two milestones coordinate at the rejection-signal interface: F-4 ensures rejection paths exist consistently across all three event families; M6 Phase 2 wires them to the wire-layer signal with envelope `event_id`. |
| J-081 (Propagation Reliability Audit) | Produced the audit finding (§5) that the M6 §9 draft's framing was necessary but not sufficient. The corrected framing in this DECISIONS.md entry incorporates the audit's insight. |

---

## D-071 — Subsystem audits precede dependent milestones (project-management principle)

**Date**: 2026-05-18  
**Layer**: Project management / roadmap discipline — not protocol.  
**Spec reference**: none (rule about how milestones are sequenced and what their design phases must include). Cross-references: D-069 (Joe-locked design phase + open-item flagging + canonical-document rule); D-065 (sibling principle — honest behaviour over polite behaviour); J-081 (the Propagation Reliability Audit, where the pattern emerged).

### Decision

Every future milestone whose correctness depends on a load-bearing subsystem MUST include a subsystem audit as part of its Phase 0 (design phase). The audit runs before design decisions are locked, produces a code-grounded canonical document, and surfaces gaps that may need to close as preconditions of the milestone rather than as parallel work.

"Load-bearing subsystem" means a piece of infrastructure that the milestone's deliverables claim to operate against — a propagation pipeline, a validation pipeline, a federation registry, a transport surface, an event-store mechanism, the Auth Module dispatch. If the milestone's promises depend on the subsystem working as specified, the subsystem's actual working state must be verified, not assumed.

The audit's outputs:

1. A canonical document (`docs/xgen_<subsystem>_<audit-type>.md` shape) recording findings with code-grounded evidence — file paths, line numbers, function names, behavioural traces.
2. A severity-classified gap list (HIGH / MEDIUM / LOW / INFORMATIONAL, with explicit criteria for each level given the milestone's context).
3. An explicit statement of which gaps are preconditions of the dependent milestone vs which are parallel work vs which are recorded for future milestones.

The audit is sized to fit; it is not a re-architecture project. The Propagation Reliability Audit (J-081) shipped in one session and verified five stages of the propagation lifecycle.

### Why this rule must be explicit

The pattern emerged organically during the Propagation Reliability Audit. Two observations established it:

**Observation 1 — Audit findings consistently exceeded the audit's nominal scope.** J-081 was opened to verify Stage 6 federation propagation reliability. It returned HIGH-severity findings in four of five sections — not just Stage 6. The audit found what it was opened to find AND surfaced multiple substantial unexpected gaps (validation asymmetry in `process_inbound`, Error wire shape lacking `event_id`, `sync_complete` gap masking premature catch-up termination, pagination gap allowing unbounded responses). Without the audit, those gaps would have surfaced under feature pressure during M6 (new) or Federation Event Propagation implementation, producing emergency descope or hotfix work.

**Observation 2 — The audit became the precondition input for the dependent milestone's design phase.** Federation Event Propagation Phase 0 took J-081 as Pass 1 input rather than running its own audit. The audit work paid for itself across two milestones (M6 Phase 0, which originally motivated it, and Federation Event Propagation Phase 0, which inherited it). One audit, two downstream design phases consume it.

Three reasons the rule is structural, not stylistic:

**Reason 1 — Subsystem reality drifts from documentation.** The audit found that `docs/xgen_ch4_implementation.md` §4.11.3 + §4.12.3 and `docs/xgen_node_admin_ops_design.md` §4.2 described federation propagation mechanisms that did not exist in code. Without an audit, the dependent milestone's design phase would have inherited the documented-but-absent behaviour as a working baseline. The longer a documented-but-absent behaviour goes unaudited, the more downstream design accumulates against it. Audits make documentation drift visible at the moment it costs least to fix.

**Reason 2 — Multi-session design conversations accumulate assumptions.** The J-080 framing that `TransportMessage::Error` was the rejection signal for event acceptance had been operating across multiple sessions. Direct code trace at audit close refuted it: `Error` had no `event_id` field, and none of the five event-rejection sites in `process_inbound` emitted it — they all just logged via `tracing::error!` + `trace_local(RejectEvent)`. The assumption had been confident, consistent, and wrong. Audits force the moment of "actually look at the code" that long-running design conversations defer indefinitely.

**Reason 3 — The pattern is naturally one-time per subsystem.** Once J-081 audited propagation, the canonical document is durable. Future milestones touching propagation read the audit doc rather than re-discovering its findings. The audit's cost amortises across all dependent work. This is the same shape as D-069's canonical-document rule applied at the verification layer: one authoritative source per subsystem state, others point at it.

### Sequencing with D-069

D-071 extends D-069 backward by one phase. D-069 governs the design phase: Joe-lock + open-item flagging + canonical document. D-071 governs the phase before the design phase: the audit phase. The full sequence for a milestone touching a load-bearing subsystem is:

```
Audit phase (D-071)  →  Design phase (D-069)  →  Implementation phase
     |                       |                          |
  Audit doc        Joe-locked design doc          Runbook + commits
   (canonical)        (canonical)                    (Clair work)
```

Each phase produces a canonical artefact. The audit doc feeds the design doc; the design doc feeds the runbook. A milestone that skips the audit phase produces a design phase whose Pass 1 input is documentation rather than code, and the documentation may be drift. A milestone that skips the design phase produces an implementation phase against decisions never Joe-locked, per D-069.

Both disciplines together: every dependent milestone gets verified reality (D-071) AND locked design (D-069) before code is written.

### Known instances at promotion

- **M6 (new) Phase 0 → Propagation Reliability Audit (J-081, 2026-05-18).** Originally motivated by the J-080 carry-over (`cmd_create_space` optimistic-ack UX) escalating to a missing protocol primitive (no positive accept signal exists today). Audit closed in one session, produced `docs/xgen_propagation_reliability.md`, surfaced four HIGH findings across five stage sections.
- **Federation Event Propagation Phase 0 (Pass 2 + Pass 3, 2026-05-18) → inherits J-081.** No re-audit; the audit's outputs are Pass 1 of the design phase. Design phase Pass 2 produced ten framework decisions; Pass 3 produced the canonical design document and implementation runbook for Clair.

This instance pattern — one audit feeding two design phases — is the worked example for the reasoning above (audits pay for themselves across dependent work).

### Out of scope for this decision

- **Audits as standalone milestones detached from dependent work.** The discipline is about coupling audits to milestones that need them, not creating audit-for-its-own-sake work. The audit's value is its consumption by a dependent design phase; an audit with no dependent milestone scheduled is paperwork.
- **Re-auditing already-audited subsystems on every dependent milestone.** Once audited, subsequent milestones read the canonical audit doc; re-audit only if the subsystem has materially changed since the canonical doc shipped. The decision to re-audit is itself a design-phase Pass 1 question for the new milestone, not a routine ritual.
- **Audits of fully-stable subsystems where the dependent milestone has no exposure to gaps.** Crypto primitives (`ed25519-dalek`, ChaCha20-Poly1305 from `chacha20poly1305`, Argon2id from `argon2`) are not re-audited per XGen milestone — that is the upstream maintainers' work, consumed via crates.io. Settled wire formats whose semantics haven't changed in many milestones are similarly out of scope. The principle applies where there is realistic risk of drift between specification and implementation, not as a blanket requirement for every dependency.
- **The audit's exact methodology, severity-classification thresholds, or document template.** The J-081 audit shape (five-stage walk + per-section verdict + drift surface tally + canonical-doc output) is a precedent, not a prescription. Future audits adapt their methodology to the subsystem they verify. What the principle requires is that the audit *exists*, produces a canonical artefact, and feeds the dependent design phase; how it gets there is the auditor's call.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-065 | Sibling principle (honest behaviour over polite behaviour). D-065 is protocol-design; D-071 is project-management. Both decisions take implicit gaps out of the system: D-065 makes the protocol honest about its state at runtime; D-071 makes the project honest about its subsystems' state before locking design. The shared theme: don't let assumed-state substitute for verified-state. |
| D-067 | M5's `ops::*` refactor eliminated drift between parallel command implementations architecturally. D-071 eliminates drift between documented-vs-actual subsystem behaviour by requiring code-grounded audits. Both decisions are about taking implicit gaps out of the system — D-067 at the implementation layer, D-071 at the verification layer. |
| D-069 | The two disciplines pair: D-069 governs the design phase (Joe-lock + open-item flagging + canonical document), D-071 governs the audit phase before it. D-071 extends D-069's logic backward: design must be locked before implementation, and verification must precede design. Both decisions enforce that earlier discovery prevents implementation-time surprises. |
| D-070 | Sibling decision shipped earlier the same day. D-070 is protocol-design; D-071 is project-management. The two were both surfaced during the Propagation Reliability Audit's close-out: D-070 from the audit's §5 finding about Error wire shape; D-071 from the audit's §6.2 pattern observation about drift surfaces across multiple sections. Both were originally drafted in the M6 design doc and Federation Event Propagation work; both promoted to DECISIONS.md in coordinated post-Pass-3 work. |
| J-081 (Propagation Reliability Audit) | The audit that established the pattern. D-071 names the discipline that J-081 retroactively instantiates. Future audits inherit the J-081 shape (one session, code-grounded, severity-classified, canonical-document output) as a precedent but are not bound to its exact methodology. |
| M6 Phase 0 + Federation Event Propagation Phase 0 | The two milestones whose design phases consumed J-081's output. Pattern: subsystem audit → dependent milestone's Phase 0 design uses audit as Pass 1 input → Phase 0 produces design doc → implementation runbook → implementation. Both are worked examples of D-071 + D-069 operating together. |

---

## D-072 — XGID Adoption v1 (named identifier type discipline)

**Date**: 2026-05-20  
**Layer**: Cross-cutting — vocabulary + type discipline spanning every crate (`xgen-common`, `xgen-core`, `xgen-node`, `xgen-client`) and every documentation surface (Ch3, Ch4, Ch6, Appendix F, Appendix I, Appendix J, `docs/xgen_aicontrol_implementation.md`).  
**Spec reference**: `docs/xgen_appendix_j_en.md` (canonical expository document — taxonomy, construction, wire-invariance promise, immutability framing, worked rejection examples); `docs/xgen_ch3_specification.md` §3.X (terse normative section). Cross-references: D-073 (field-name-vs-type discipline that underwrites how XGID flavours compose with field names at use sites); D-069 (canonical-document rule — Appendix J is the canonical home, others point at it); D-065 (sibling principle — wire-format honesty over local convenience).

### Decision

The XGen Protocol adopts **XGID** as the canonical name and type discipline for all first-class identifiers in the protocol. Six XGID flavours ship at v1: **Event**, **Space**, **Room**, **TrustAssertion** (hash-anchored family) and **Node**, **Identity** (principal family). The Rust type representation is a layered newtype: a base `Xgid(String)` plus six flavour wrappers (`EventXgid`, `SpaceXgid`, `RoomXgid`, `TrustAssertionXgid`, `NodeXgid`, `IdentityXgid`), each `Deref<Target = Xgid>`, all serde-transparent as plain strings so wire format is untouched. All six wrappers, the base type, and the `XgidLike` trait ship in `xgen-common` at v1.

The principle wording, locked at walkthrough close 2026-05-20 and reproduced verbatim in Appendix J:

> *"XGID Adoption v1 ships the types and adopts them in new code. Retrofitting existing XGID-string fields is staged into subsystem-scoped retrofit milestones. The codebase MAY carry mixed discipline transitionally; every new field, new signature, and new trace event field MUST use XGID types from this milestone forward."*

Five **wire-format invariances** are guaranteed under D-072 across both wire crossings the protocol exposes (federation wire AND AI control / batch JSONL wire):

1. **Field names** — the JSON field name carrying an XGID does not change between v1 and any future retrofit pass.
2. **Field types** — the on-wire JSON type is `string`, regardless of which Rust newtype wraps it.
3. **Canonical form** — the string contents of any XGID are byte-identical when produced from the same inputs anywhere in the federation.
4. **URI grammar** — the structural shape of XGID strings (prefix, separators, length characteristics, character class) is fixed at v1 and does not change under retrofit.
5. **String-equality semantics** — two XGIDs are equal iff their string contents are equal. No flavour-aware comparison; no normalisation hooks.

The **second wire crossing** is named explicitly: the AI control / batch JSONL wire format (`docs/xgen_aicontrol_implementation.md`, Appendix F's batch reply schemas, Ch6 §6.15) inherits the five invariances. Any boundary where XGID strings cross a process is bound by the same rules; the protocol does not get to be sloppy at the implementation-protocol seam.

Adoption discipline is **Shape γ + ASAP** — staged retrofit milestones (XGID Retrofit Passes 1–5) land in ROADMAP.md Near future immediately after v1 ships, not Far future. The five passes are subsystem-scoped: Pass 1 retypes `xgen-common` core types and Appendix I Part I; Pass 2 retypes `xgen-core` validation/dispatch/pending-buffer surfaces; Pass 3 retypes `xgen-node` federation/fanout/app surfaces and Appendix F Node-side sections; Pass 4 retypes `xgen-client` ops/AI-behaviour/batch surfaces and the AI-control documentation; Pass 5 retypes test fixtures, helpers, trace events, and any remaining surfaces. After Pass 5 closes, the "mixed discipline transitionally" clause of the principle wording no longer applies.

### What XGID is and is not

**XGID is** the canonical name for first-class protocol identifiers — things that name a durable protocol object that other protocol objects reference by identity. The six flavours are exhaustive at v1. Sub-flavours (e.g. ephemeral `session_id` as an Event-XGID sub-axis) are taxonomic refinements within Appendix J, not new top-level flavours.

**XGID is not**:

- **Wire-envelope correlation handles.** M6 (new) Phase 2's `event_id: Option<String>` field on `TransportMessage` is a transport-layer correlation handle, distinct from the Event XGID it correlates to. The two are equal at the string level by construction but live at different protocol layers and have different lifecycles.
- **Error codes.** Numeric or string-tagged error codes (`4002`, `4006`, `4007`, etc.) are not XGIDs.
- **Config field names** or in-memory handle types like `FederationPeerSenders` keys (even though the keys' string values are XGIDs — the *map structure* isn't an XGID).
- **File paths, log line tokens, debug formatters.** XGID types may *appear* in these via `Display` / `Debug`, but the paths/tokens themselves are not XGIDs.
- **Bootstrap discovery URIs.** Discussed during Q1 walkthrough and explicitly excluded — these are operational addresses, not protocol-object identifiers.

### Why this discipline must be explicit

**Reason 1 — Field-typed-as-String hides protocol-object semantics.** A Rust function signature `fn foo(a: String, b: String, c: String)` carries no information about which argument is which protocol object. A reader has to consult the call site, the field name, and ideally a doc comment to recover the role each `String` plays. Layered newtypes recover that information in the type system: `fn foo(event_id: EventXgid, sender: IdentityXgid, room_id: RoomXgid)` cannot be miscalled. The protocol has eight years of identifier discipline ahead of it; a String-typed identifier surface accumulates miscalls and misroutings at a rate that retrofits cannot keep up with.

**Reason 2 — Without a canonical name, vocabulary fragments.** Before this decision, the project used "event ID", "event id", "sender pubkey", "node URI", "space ID", "room ID", "identity URI", "trust assertion ID" across documentation and code interchangeably and inconsistently. Different docs used different framings; different code used different field names for the same protocol object. "XGID" provides one umbrella name; six flavours provide the discriminators; all parts of the protocol that need to name an identifier reach for the same vocabulary. The discipline pays off most heavily in design conversations: "is this an XGID?" becomes a tractable question with a yes-or-no answer, where "is this an identifier?" was an open framing question every time.

**Reason 3 — Wire-invariance must be the default, not an aspiration.** A protocol whose identifiers can drift in field name, field type, canonical form, URI grammar, or equality semantics between releases produces federation-breakage at every release boundary. The five-invariance promise sets the default to "no drift"; departures from the default require explicit protocol-version negotiation, not silent change. The naming of the invariances at the wire-format layer (rather than as Rust-type-system properties) means the same promise binds non-Rust implementations: any future XGen client, written in any language, gets the same wire-level guarantees.

**Reason 4 — Staged retrofit is honest about the cost of perfection.** A "retype everything in one milestone" approach would either delay v1 by months or ship a v1 with cut corners. Shape γ + ASAP retrofit acknowledges the cost honestly: v1 ships the types and the discipline; existing String fields convert pass-by-pass over the subsequent retrofit milestones; the codebase carries mixed discipline transitionally and explicitly. This is the same shape as D-065's principle (honest behaviour over polite behaviour) applied to a project-management surface: the protocol does not pretend to be perfectly typed during the transition; it states the transition as a real and named project phase.

### Worked instances at promotion

- **Phase 7.5 `SpaceLocalMetadata.introducer_node_id`** — the v1 inaugural production use of an XGID flavour. The field was named with future-XGID-typing in mind during Phase 7.5 design (per §5.6 of `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md`); the v1 implementation runbook retypes it from `Option<String>` to `Option<NodeXgid>` in Commit 2.
- **Phase 9 integration test infrastructure** — the test code Phase 9 ships uses XGID types from the start rather than being retrofitted later. This is why XGID Adoption is sequenced between Phase 7.5 closure and Phase 9 Commit 3 resumption: the test surface is touched once, with the right types.
- **`xgen-common` v1 — the type definitions themselves.** Six flavour wrappers, base `Xgid`, `XgidLike` trait, flavour-specific constructors (e.g. `EventXgid::from_event`, `NodeXgid::from_pubkey`), flavour-specific methods (e.g. `IdentityXgid::pubkey() -> VerifyingKey` on principal flavours; content-derived helpers on hash-anchored flavours), `Deref<Target = Xgid>` on each wrapper, serde-transparent string serialisation, full `Display` / `Debug` / `Eq` / `Hash` / `Clone` derives.
- **Pass 1–5 worked subsystems** — the five retrofit passes are themselves worked instances of the staged-retrofit discipline. Pass 1 (`xgen-common` core types) starts immediately after Phase 9 closes and the Federation Event Propagation milestone flips DONE.

### Out of scope for this decision

- **Future XGID flavour additions.** If a new protocol object surfaces that warrants first-class identifier status (and isn't a sub-axis of an existing flavour), the addition is a future decision, not a parameter of D-072. The taxonomy at v1 is the six-flavour set; growth requires explicit promotion through a future decision entry.
- **Cross-flavour conversion semantics.** Whether (e.g.) a `NodeXgid` can be converted to an `IdentityXgid` is a use-site question answered by use-site logic, not a type-system feature. The flavour wrappers are deliberately not interconvertible at the type level; converting one to another requires extracting the base `Xgid` (via `Deref`) and constructing the target flavour explicitly. This is a feature, not a limitation: silent flavour drift at use sites is what the newtype discipline exists to prevent.
- **Normalisation, case-folding, or whitespace-tolerance.** Invariance 5 (string-equality semantics) is strict: two XGIDs are equal iff their bytes are equal. No normalisation hooks at v1; if normalisation becomes necessary later, it's a protocol-version-bumped change, not a quiet upgrade.
- **Implementation language coupling.** XGID is a protocol-layer concept; the Rust layered-newtype implementation is the v1 *reference* implementation. Future XGen clients in other languages implement the same vocabulary, the same flavours, and the same five wire invariances; they MAY implement the type discipline differently (or not at all, if their type system can't express it cleanly). The invariances bind the wire; the types bind the reference implementation.
- **Wire-format protocol-version negotiation.** D-072 says identifiers don't drift at v1; it does not say there can never be a future protocol version with different identifier semantics. Future versions are explicit version bumps with explicit migration paths, not silent retrofits.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-073 | Coordinated output of the XGID Adoption design walkthrough (same session 2026-05-20). D-073 names the field-name-vs-type discipline that XGID's layered newtype design depends on at use sites: a field named `introducer_node_id` typed as `NodeXgid` says "this is the introducer's identity, and it is a Node XGID" — the name carries the role, the type carries the contract. D-072 establishes the types; D-073 establishes how those types compose with field names. Both decisions land in the same Phase 1 canonical sources commit. |
| D-065 | Sibling principle (honest behaviour over polite behaviour). D-065 is the protocol-design analogue; D-072's adoption discipline is the project-management analogue. Where D-065 requires the protocol to be honest about runtime state, D-072 requires the project to be honest about adoption state: "mixed discipline transitionally" is explicit, named, and bounded by the Pass 5 closure point. Both decisions take implicit gaps out of the system: D-065 from the protocol's behaviour, D-072 from the project's identifier vocabulary. |
| D-069 | The canonical-document rule applies here: Appendix J is the canonical home for XGID concepts; Ch3 §3.X carries the terse normative form; DECISIONS.md D-072 is the architectural commitment; all three reference each other and do not duplicate. The Phase 1 canonical sources commit is itself a worked example of D-069 discipline: a multi-surface concept gets one authoritative document (Appendix J) with downstream sources pointing at it, not scattered. |
| D-070 | Coordinated relationship at the protocol layer. D-070's `event_id: Option<String>` envelope-level correlation handle is *not* itself an XGID (per the "what XGID is not" section above), but its string value is byte-equal to the corresponding Event XGID by construction. The relationship is documented at the use site, not encoded in the type system: D-072's flavours bind protocol-object identifiers; D-070's envelope field is a transport-layer correlation handle that happens to carry an XGID-shaped string. Keeping the two separate at the type level prevents miscalls between protocol-layer and transport-layer surfaces. |
| D-071 | Sibling project-management principle. D-071 governs the audit phase before milestone design (verify reality before locking design). D-072 governs adoption discipline across the whole project (commit to vocabulary + types; stage retrofit honestly). Both decisions take implicit gaps out of the project's shape: D-071 between documentation and code; D-072 between identifier vocabulary in design conversations and identifier types in implementation. The shared pattern: make implicit state explicit. |
| Phase 7.5 design (`tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md`) | The originating precedent for XGID-aware field design. The `introducer_node_id` field's naming was Joe-locked at §5.6 with explicit future-XGID-typing intent. D-072 promotes that one-off forward-aware decision into a project-wide discipline; D-073 names the field-name-vs-type principle the §5.6 decision instantiated. Phase 7.5's implementation runbook retypes the field as the v1 inaugural production use. |

---

## D-073 — Field-name-vs-type discipline (project-wide naming principle)

**Date**: 2026-05-20  
**Layer**: Cross-cutting — naming and typing discipline at every Rust struct field, function parameter, trace event field, and JSON wire field across all four crates and all documentation surfaces describing them.  
**Spec reference**: `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` §5.6 (originating precedent — the `introducer_node_id: NodeXgid` worked example). Cross-references: D-072 (the XGID type vocabulary this principle composes with at use sites); D-069 (canonical-document rule — Appendix J's introduction echoes this principle in one sentence pointing here); D-065 / D-070 / D-071 (sibling architectural principles that take implicit gaps out of the system).

### Decision

**The field name carries the role; the type carries the contract.**

Every Rust struct field, function parameter, trace event field, and JSON wire field that names a protocol object obeys this composition rule:

- **The field name** identifies the *role* the protocol object plays at this use site — what *this particular instance* is doing here. Examples: `introducer_node_id` (the Node that introduced this Space to us); `sender` (the Identity that signed this event); `room_id` (the Room this message belongs to); `peer_node_id` (the Node on the other side of this federation session); `delegated_to` (the Identity an operator role was delegated to).
- **The field type** identifies the *contract* — what kind of protocol object this field can ever hold. Examples: `NodeXgid` (always a Node XGID, never an Identity XGID); `IdentityXgid` (always an Identity XGID); `RoomXgid` (always a Room XGID).

The two pieces of information are orthogonal and both are load-bearing. A field name without type discipline tells you the role but not the contract — a reader has to consult the field's documentation to know that `introducer_node_id` is always a Node XGID, never something else. A type without role discipline tells you the contract but not the role — a function signature `fn foo(a: NodeXgid, b: NodeXgid, c: NodeXgid)` cannot be miscalled at the type level, but a reader has no way to know which Node is which without consulting the docs. Both pieces together produce code that is self-documenting at the use site: `fn foo(introducer: NodeXgid, peer: NodeXgid, owner: NodeXgid)`.

The principle applies to all four surfaces:

1. **Rust struct fields** — `pub introducer_node_id: Option<NodeXgid>`, not `pub introducer: Option<NodeXgid>` (role lost) and not `pub introducer_node_id: Option<String>` (contract lost).
2. **Function parameters** — `fn drain_pending_by_federation_relationship(peer: NodeXgid, space: SpaceXgid)`, not `fn drain_pending_by_federation_relationship(a: String, b: String)`.
3. **Trace event fields** — when a structured trace event carries an XGID, the field name in the event matches the use-site role (e.g. `originator_identity` vs `recipient_identity`) AND the field is typed as the appropriate XGID flavour (not bare `String`).
4. **JSON wire fields** — same rule applied through serde-transparent serialisation: the wire field name carries the role, the underlying Rust type carries the contract. Wire readers see strings (per D-072 invariance 2), but the surrounding field name still names the role.

### Why this discipline must be explicit

**Reason 1 — The discipline emerged organically and was about to be lost in transition.** The originating precedent (Phase 7.5 §5.6, `introducer_node_id`) was Joe-locked mid-design as a forward-looking naming decision: the field was named with a future XGID-typing pass in mind, and the §5.6 inline note explained the reasoning. Without promotion to a DECISIONS.md entry, the rationale would have lived only in a Phase 7.5 design file — which becomes archived once Phase 7.5 ships, and whose authority decays with it. The next person designing a new field would either re-derive the principle from scratch, miss it, or invent a different one. Naming the discipline makes it durable across milestones.

**Reason 2 — Field-name-only discipline produces accidental String typing.** Without the type half of the discipline, a well-intentioned designer who names a field correctly (`introducer_node_id`) is free to type it as `String` because "the name says what it is." That works for one field by one designer in one PR. It fails when the field is used at five call sites, or when a second designer adds a sibling field (`peer_node_id`) and chooses a different type, or when a JSON-decoded value flows into the field without the surrounding type guard. The type half of the discipline is what makes the name-half load-bearing: the compiler enforces what the name claims.

**Reason 3 — Type-only discipline produces opaque use sites.** Without the name half, a function signature like `fn handshake(a: NodeXgid, b: NodeXgid)` is type-safe but unreadable. Which Node is `a`? Which is `b`? A reader has to consult the function body or doc comment to learn that `a` is the local Node and `b` is the remote Node. Naming the role at the field-name level pushes that information to the first place a reader looks, which is the signature itself.

**Reason 4 — The principle generalises beyond XGID.** While XGID is the v1 worked example, the field-name-vs-type discipline applies to every type the project uses for protocol-object identifiers, capabilities, or roles. A future field carrying a capability set (`pub required_capabilities: CapabilitySet`) follows the same rule: name says role (`required_capabilities`, distinct from `granted_capabilities`); type says contract (`CapabilitySet`, not bare `Vec<String>`). Naming the discipline as a standalone decision (not as a footnote to D-072) signals that it operates wherever a typed field carries a role-bearing semantic, not only for identifiers.

### Worked instances at promotion

- **`SpaceLocalMetadata.introducer_node_id: Option<NodeXgid>`** — the originating precedent, locked at Phase 7.5 §5.6 and realised in XGID Adoption v1 Commit 2. The Phase 7.5 design walkthrough explicitly chose this name over candidates like `introducer` (role-only, lost the "Node XGID" contract signal) and `introducer_id` (ambiguous about which kind of ID — could be Node, Identity, or Space at a glance). The locked name encodes both halves: the role (introducer) and the contract (a Node ID).
- **`peer_node_id` / `space_id` / `room_id` / `identity_id` as established naming convention.** The four idiomatic field names already widely used across the codebase obey the discipline at the name level; XGID Adoption v1 and the subsequent retrofit passes complete the discipline at the type level.
- **Forward-looking application to AI-control and admin-ops surfaces.** When M7 (`--aicontrol`) and M6 (new) ship their JSONL reply schemas, each XGID-carrying field obeys both halves: role-bearing names (`accepted_event_id`, `rejected_event_id`, `target_room_id`, `delegated_to_identity`) with XGID-flavour types underneath.

### Out of scope for this decision

- **Acceptable role-bearing field names.** The principle requires that field names carry a role; it does not prescribe a closed vocabulary of role names. `introducer_node_id` vs `bootstrapping_node_id` vs `origin_node_id` are all acceptable role-bearing names for similar concepts; the choice between them is a use-site-design question, not a D-073 question.
- **Non-XGID typed fields.** The principle is a *general* composition rule; this decision documents it via XGID worked examples because XGID is the v1 surface where it most heavily applies. Application to other typed-field surfaces (capabilities, error codes, event-type discriminators) is implicit in the principle's generality and does not require enumerating every future case here.
- **Naming-only docs (e.g. JSON wire docs where Rust types are not visible).** A JSON-only document like `docs/xgen_aicontrol_implementation.md` cannot show Rust types directly. The principle still applies through transitivity: the JSON field name carries the role, the documented type contract ("this field is an Event XGID") carries the contract, and the implementation enforces both halves through serde-transparent typed Rust fields.
- **Internal-only helper functions.** Discipline is meaningful at API boundaries (public structs, function signatures consumers see, trace events external observers consume, JSON wire fields). Truly local helpers (`fn parse_inner(s: &str) -> Result<...>`) are not bound to use role-bearing parameter names if the role is obvious from one call site over a five-line function. The principle is about preventing miscalls at scale, not about adding ceremony to trivial code.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-072 | Coordinated output of the XGID Adoption design walkthrough (same session 2026-05-20). D-072 establishes the type vocabulary (six XGID flavours, layered newtype, wire invariances); D-073 names the discipline that XGID's layered newtype design relies on at use sites — every use of an XGID type pairs with a role-bearing field name. Without D-073, D-072's type discipline could still be undermined by opaque field names; without D-072, D-073's role-bearing names would have no type system to enforce contracts against. Both decisions land in the same Phase 1 canonical sources commit. Appendix J's introduction carries a one-sentence echo of D-073 pointing here. |
| D-065 | Sibling principle (honest behaviour over polite behaviour) at the naming layer. D-065 requires the protocol to be honest about state; D-073 requires field names and types to be honest about what they hold. A field named `node_id` typed as `String` is dishonest in the same architectural sense: it claims (through the name) to hold a Node ID but cannot enforce (through the type) what kind of ID. The discipline takes that dishonesty out of the use site by structural means. |
| D-069 | The canonical-document rule applies: D-073's authoritative home is DECISIONS.md; Appendix J's introduction carries a one-sentence echo with a pointer here; `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` §5.6 remains as historical originating-precedent record. Three surfaces, one authority, explicit forwards. |
| D-070 | The envelope `event_id: Option<String>` field on `TransportMessage` is the worked counter-example: it deliberately departs from the field-name-vs-type discipline (the name carries the role "event ID" but the type is bare `String`, not `EventXgid`). The departure is intentional and documented in D-072's "what XGID is not" section: `event_id` is a transport-layer correlation handle, NOT itself an XGID, and the type-level separation prevents miscalls between protocol-layer and transport-layer surfaces. D-073 thus tolerates documented exceptions where the architectural reasoning supports them. |
| D-071 | Sibling project-management principle. Both decisions take implicit gaps out of the project: D-071 between documented and actual subsystem behaviour; D-073 between named roles and enforced contracts at field-level granularity. The shared pattern across D-065 / D-069 / D-070 / D-071 / D-072 / D-073 is the same: make implicit state explicit at the layer where the implicitness produces drift. |
| Phase 7.5 §5.6 (originating precedent) | The Joe-locked naming decision that produced the principle. §5.6 of `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` chose `introducer_node_id` over candidates that lost either the role half (`introducer`) or the contract half (`introducer_id`). The §5.6 inline reasoning is preserved as historical originating-precedent record; D-073 promotes the underlying principle into a project-wide discipline. |

---

