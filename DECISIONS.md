# XGen Protocol — Implementation Decisions
> **Status:** ACTIVE  
> **Last updated:** 2026-05-15 (D-061 rewritten in place)  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

Every decision that goes beyond spec prescription is recorded here before advancing to the next layer.
Format: title, date, layer, spec reference, decision narrative.

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

`xgennode.exe` is a singleton process — it starts once and runs permanently. The UI is not the lifecycle host; the process is.

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
- `xgenclient.exe --stop` and `xgennode.exe --stop` CLI flags
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
