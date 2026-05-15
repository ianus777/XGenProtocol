# AI Users and Pacing — Phase 2 Implementation Task
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-15 (J-065 — implementation complete, 387 tests pass)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Overview

This task implements three Ch3 protocol additions (D-059, D-060, D-061) in `xgen-core`, `xgen-node`, and `xgen-client`. The three additions are conceptually related but technically independent — each Part below can be implemented and verified separately.

**Spec references (all stable):**
- Ch3 §3.6.10 AI Identity Extension (D-059)
- Ch3 §3.7.12 Pacing Rules on Spaces (D-060)
- Ch3 §3.7.13 Temperature Property (D-061)
- Ch3 §3.7.8 `membership.mute` Event and `auto_temperature` reason
- Ch6 §6.12 Temperature Property (client display)
- Ch6 §6.13 AI Member Badge (client display)
- Ch6 §6.14 Pacing Queue (client behaviour)
- DECISIONS.md D-059, D-060, D-061

**Part order:** Parts A, B, C are independent. Recommended order is A → B → C because B and C build on `is_ai` from A. Each Part has its own Definition of Done; mark each item only when verified with actual output per CLAUDE.md Rules 1–7.

---

## Part A — AI Identity Extension (D-059, Ch3 §3.6.10)

### A.1 Scope

Add the `is_ai` boolean and `ai_capabilities` map to the Identity record. Enforce shape consistency at registration, immutability after registration, and protocol-level capability restrictions on outbound Events from AI Identities.

### A.2 `xgen-common` changes

**File: `xgen-common/src/wire.rs`** — Identity record extension.

Add two fields to the `Identity` struct:

```rust
#[serde(default)]
pub is_ai: bool,
#[serde(default, skip_serializing_if = "Option::is_none")]
pub ai_capabilities: Option<AiCapabilities>,
```

Define `AiCapabilities` as a struct with the Phase 2 required capability keys (open to future extension via `#[serde(flatten)]` into an inner `BTreeMap<String, Value>` for unknown keys):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiCapabilities {
    pub dm_initiate: bool,
    pub spontaneous_post: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
```

The `extra` map carries unknown capability keys for forward compatibility (Ch3 §3.6.10.3 — older Nodes ignore unknown keys).

### A.3 Registration validation

**File: `xgen-core/src/identity/registration.rs`** — extend the acceptance pipeline.

Per Ch3 §3.6.4, the acceptance pipeline gains a new step 8 (validate `is_ai` / `ai_capabilities` shape consistency). The pre-existing capacity check becomes step 9. Implement the new step exactly per Ch3 §3.6.10.1:

- If `is_ai = true` and `ai_capabilities` is missing, null, or missing a required Phase 2 key (`dm_initiate`, `spontaneous_post`) → reject with error `3040 ai_declaration_invalid`.
- If `is_ai = false` and `ai_capabilities` is non-null → reject with the same error `3040`.
- Otherwise accept.

Error codes (Ch3 §3.6.10.10):

| Code | Name | Wire constant |
|---|---|---|
| `3040` | `ai_declaration_invalid` | `IDENTITY_AI_DECLARATION_INVALID` |
| `3041` | `ai_flag_immutable` | `IDENTITY_AI_FLAG_IMMUTABLE` |
| `3042` | `ai_capability_violation` | `IDENTITY_AI_CAPABILITY_VIOLATION` |

Add these to the existing error code module. All three live in the 3000–3999 identity domain.

### A.4 Immutability enforcement

**File: `xgen-core/src/identity/registration.rs`** — `identity.update` handler.

When processing `identity.update`, if the `changes` object contains the `is_ai` key, reject with error `3041 ai_flag_immutable`. The `ai_capabilities` map MAY be updated; only `is_ai` itself is immutable.

### A.5 Capability enforcement on outbound Events

**File: `xgen-core/src/wire/validation.rs`** — extend event acceptance.

For every Event the Node receives, after signature validation:

1. Look up the sender's Identity record (`sender` field on the Event).
2. If `is_ai = false`, no capability check applies; continue normal validation.
3. If `is_ai = true`, check the Event type against the sender's `ai_capabilities`:

| Event type | Capability required | Action if violated |
|---|---|---|
| `state.dm_space_create` (sender is_ai = true) | `ai_capabilities.dm_initiate = true` | Reject with `3042 ai_capability_violation`, message "dm_initiate disallowed" |

Per Ch3 §3.6.10.4, the `spontaneous_post` capability is NOT Node-validated in Phase 2 — it is a client-side and admin-policy concern. Do not implement Node-side enforcement for `spontaneous_post` in Phase 2.

### A.6 Operator delegation EventTypes

**File: `xgen-common/src/wire.rs`** — add EventType variants.

Add two new variants to `EventType`:

```rust
StateAiOperatorDelegate,  // "state.ai_operator_delegate"
StateAiOperatorRevoke,    // "state.ai_operator_revoke"
```

Extend `as_str()` and `from_str()` for both. Add the corresponding content struct per Ch3 §3.6.10.6:

```rust
pub struct StateAiOperatorDelegateContent {
    pub space_id: String,
    pub ai_identity_id: String,
    pub new_operator_identity_id: String,
}

pub struct StateAiOperatorRevokeContent {
    pub space_id: String,
    pub ai_identity_id: String,
}
```

Wire format and signing follow standard rules. No state machine integration required in Phase 2 beyond accepting and storing these Events — they are accountability records, not protocol-level privilege grants.

### A.7 Replication

**No changes needed** to `xgen-core/src/identity/replicate.rs`. The `is_ai` and `ai_capabilities` fields ride the existing `identity_record` payload in `identity.replicate` messages (Ch3 §3.6.10.9). Verify with a test that a replicated Identity record carries both fields correctly.

### A.8 Definition of Done — Part A

Mark each item only when verified with actual command output (CLAUDE.md Rules 1–7).

- [ ] `xgen-common/src/wire.rs` — `Identity` struct has `is_ai` and `ai_capabilities` fields with correct serde attributes
- [ ] `AiCapabilities` struct defined with `dm_initiate`, `spontaneous_post`, and `extra: BTreeMap` for forward compatibility
- [ ] Error codes 3040, 3041, 3042 added to the existing error code module
- [ ] `EventType::StateAiOperatorDelegate` and `EventType::StateAiOperatorRevoke` added with `as_str()` / `from_str()` coverage
- [ ] Content structs `StateAiOperatorDelegateContent` and `StateAiOperatorRevokeContent` defined
- [ ] Acceptance pipeline step 8 validates `is_ai` / `ai_capabilities` shape consistency
- [ ] `identity.update` rejects `is_ai` changes with error 3041
- [ ] Event validation rejects `state.dm_space_create` from `is_ai = true` Identity without `dm_initiate = true` with error 3042
- [ ] Replication test confirms `is_ai` and `ai_capabilities` survive `identity.replicate` round-trip
- [ ] Unit tests for each rejection path (3040 missing capabilities, 3040 wrong shape, 3041 flag change, 3042 capability violation)
- [ ] `cargo test` passes — quote actual test count from output
- [ ] `cargo build --release` clean on both binaries — no warnings

---

## Part B — Per-Space Pacing Rules (D-060, Ch3 §3.7.12)

### B.1 Scope

Add `human_pacing_ms` and `ai_pacing_ms` fields to Space state. Add the `state.space_pacing` EventType for updates. Implement client-side outbound queue logic per Ch6 §6.14. Phase 2 enforcement is client-side only (Ch3 §3.7.12.4) — no Node validation of incoming Event timestamps.

### B.2 `xgen-common` changes

**File: `xgen-common/src/wire.rs`** — Space state extension.

Add two fields to the `SpaceState` struct:

```rust
#[serde(default = "default_human_pacing_ms")]
pub human_pacing_ms: u64,
#[serde(default = "default_ai_pacing_ms")]
pub ai_pacing_ms: u64,
```

With defaults per Ch3 §3.7.12.2:

```rust
fn default_human_pacing_ms() -> u64 { 500 }
fn default_ai_pacing_ms() -> u64 { 2000 }
```

Both fields are non-negative integers. Zero is valid and disables pacing for that member class.

Add the EventType variant:

```rust
StateSpacePacing,  // "state.space_pacing"
```

Add the content struct:

```rust
pub struct StateSpacePacingContent {
    pub human_pacing_ms: u64,
    pub ai_pacing_ms: u64,
}
```

Both fields are required in the content object (Ch3 §3.7.12.3 — partial updates are not supported).

### B.3 Space creation

**File: `xgen-core/src/space/state.rs`** — extend `state.space_create` handling.

When the Space creation Event is processed:

1. If the `content` object includes `human_pacing_ms`, store it.
2. If absent, apply default `500`.
3. Same for `ai_pacing_ms`, default `2000`.

The fields land in `SpaceState` and are read by subsequent state queries.

### B.4 Space pacing update

**File: `xgen-core/src/space/state.rs`** — handle `state.space_pacing`.

Apply the same Event-dispatch pattern as other state updates:

1. Verify the sender is the Space owner (per Ch3 §3.7.12 Role permission table — only the owner may update pacing).
2. Update both `human_pacing_ms` and `ai_pacing_ms` in the Space state to the values in the Event content.
3. Both fields required; reject if either is missing.

### B.5 Client-side outbound queue

**File: `xgen-client/src/pacing.rs`** — new module.

Implement the outbound queue per Ch6 §6.14.2:

- One queue per (space_id, sender_identity_id) pair, in memory only
- FIFO ordering, no reordering
- On each send attempt:
  - Look up `last_send_at` for the pair (init to `0` if absent)
  - Compute `elapsed = now - last_send_at`
  - Select cap: `is_ai` of sender → `ai_pacing_ms`, else `human_pacing_ms` (Ch6 §6.14.1)
  - If `elapsed >= cap`, send immediately and update `last_send_at`
  - If `elapsed < cap`, enqueue with `release_at = last_send_at + cap`
- A timer fires at `release_at` and releases the message

Edge cases per Ch6 §6.14.6:

- Negative `elapsed` (clock skew) → treat as `0`
- `is_ai` unknown → fall back to `human_pacing_ms`
- Pacing fields absent from Space state → fall back to defaults 500 / 2000
- Cap of zero → pass through immediately

### B.6 Client UI surface

The DOM contract for the pacing queue indicator (Ch6 §6.14.3, §6.14.4) is implemented in `xgen-client-ui/` Svelte components. **This is UI work; do not implement in `xgen-client/` Rust code.** The Rust code's job is to expose the queue state via `invoke()` so the Svelte layer can read it.

Expose these Tauri commands:

- `get_pacing_state(space_id: String) -> PacingState` — returns current queue depth, time-to-next-send, applied cap
- The `PacingState` struct should serialise cleanly to JSON for `data-pacing-state` / `--xgen-pacing-*` consumption

### B.7 Definition of Done — Part B

- [ ] `xgen-common/src/wire.rs` — `SpaceState` has `human_pacing_ms` and `ai_pacing_ms` with serde defaults 500 / 2000
- [ ] `EventType::StateSpacePacing` and `StateSpacePacingContent` defined and wire-tested
- [ ] `state.space_create` populates pacing fields with defaults when absent
- [ ] `state.space_pacing` updates both fields; rejects when sender is not the owner; rejects when either field is missing
- [ ] `xgen-client/src/pacing.rs` outbound queue implemented per Ch6 §6.14.2
- [ ] Queue selects cap based on sender's `is_ai` (Ch6 §6.14.1)
- [ ] Queue handles all four edge cases (clock skew, missing `is_ai`, missing fields, cap-of-zero)
- [ ] Tauri command `get_pacing_state` returns queue snapshot for UI consumption
- [ ] Unit tests for queue behaviour: immediate release, throttled release, sequential burst drain, clock skew, cap-of-zero
- [ ] `cargo test` passes — quote actual test count from output
- [ ] `cargo build --release` clean

---

## Part C — Temperature Property (D-061, Ch3 §3.7.13)

### C.1 Scope

Reserve two `meta_atts` keys for temperature. Add the `temperature_thresholds` field to the Room metadata response. Add the `member_temperature_visibility` field on Space state with the `state.space_temperature_visibility` update EventType. Implement Node-side visibility filtering on outgoing `meta_atts`. Add `membership.mute` event and reserve the `auto_temperature` reason value on `membership.kick` and `membership.mute`.

**Out of scope:** the mathematical model that computes the temperature values. That belongs to a plugin running on the Room's home Node (Ch3 §3.7.13.5, D-061). This task implements the protocol surface and the plumbing; a placeholder no-op plugin is acceptable for Phase 2.

### C.2 `xgen-common` changes — `meta_atts` reserved keys

**File: `xgen-common/src/wire.rs`** — document and validate the reserved keys.

Add to the existing `meta_atts` validation:

- `xgen.room_temperature` — float, MUST be `0.0 ≤ v ≤ 1.0`, clamp on out-of-range
- `xgen.member_temperature` — float, same range constraints

The validation lives in the `meta_atts` parser; out-of-range values are clamped silently (Ch3 §3.7.13.1).

No new struct fields on the `Event` type — `meta_atts` is already an open-enum `BTreeMap<String, Value>`. The temperature keys are simply two of the allowed `xgen.*` keys with their type and range constraints documented.

### C.3 `xgen-common` changes — `membership.mute` Event

**File: `xgen-common/src/wire.rs`** — add EventType variant.

```rust
MembershipMute,  // "membership.mute"
```

Content struct per Ch3 §3.7.8:

```rust
pub struct MembershipMuteContent {
    pub target_identity: String,
    pub reason: String,
    pub cooldown_until: String,  // RFC 3339 timestamp
}
```

The `reason` field is free-text by default but recognises the reserved `auto_temperature` value (Ch3 §3.7.8 "Standard reason values" table).

### C.4 `xgen-common` changes — `member_temperature_visibility` field

**File: `xgen-common/src/wire.rs`** — Space state extension.

Add to `SpaceState`:

```rust
#[serde(default = "default_member_temperature_visibility")]
pub member_temperature_visibility: String,
```

Default:

```rust
fn default_member_temperature_visibility() -> String { "moderator".to_string() }
```

The field is a string (open enum, Ch3 §3.7.13.3). Permitted values: `moderator`, `everyone`, `self_only`. A Node receiving an unknown value treats it as `moderator`.

Add the EventType for updates:

```rust
StateSpaceTemperatureVisibility,  // "state.space_temperature_visibility"
```

Content struct:

```rust
pub struct StateSpaceTemperatureVisibilityContent {
    pub member_temperature_visibility: String,
}
```

### C.5 `xgen-core` — visibility enforcement on outgoing `meta_atts`

**File: `xgen-core/src/transport/server.rs` (or equivalent outbound path)** — filter `xgen.member_temperature` per recipient role.

When an Event with `xgen.member_temperature` is delivered to a subscribed client:

1. Look up the recipient's authenticated Identity in the Space's member list.
2. Look up the Space's `member_temperature_visibility` setting.
3. Apply the filter per Ch3 §3.7.13.4:
   - `moderator`: include only if recipient is moderator-or-higher OR recipient is the subject (the member whose temperature it is)
   - `everyone`: always include
   - `self_only`: include only if recipient is the subject
4. If filtered out, remove the `xgen.member_temperature` key from the outgoing meta_atts entirely (not set to a placeholder value).

`xgen.room_temperature` is always included regardless of role (Ch3 §3.7.13.3).

### C.6 `xgen-core` — Room metadata response with threshold table

**File: `xgen-core/src/space/state.rs` (or equivalent Room metadata handler)** — extend the response.

When the Node sends Room metadata to a connecting client (Ch3 §3.7.7 — Room state), include an optional `temperature_thresholds` object:

```rust
pub struct TemperatureThresholds {
    pub warm: f64,
    pub hot: f64,
    pub fiery: f64,
}
```

The thresholds are supplied by the home Node's temperature plugin (out of scope here). If no plugin is loaded or the plugin does not provide thresholds, omit the field — clients will fall back to Ch6 defaults (Ch3 §3.7.13.2).

Validation when the field is present (per Ch3 §3.7.13.2):

- All three fields required
- `0.0 < warm < hot < fiery ≤ 1.0`
- Invalid table → omit from the response (the Node does not propagate invalid tables)

### C.7 `xgen-core` — `state.space_temperature_visibility` handling

**File: `xgen-core/src/space/state.rs`** — handle the update Event.

Per Ch3 §3.7.13.3:

1. Verify the sender is the Space owner (only the owner may update visibility).
2. Validate the value: one of `moderator`, `everyone`, `self_only`. Unknown values are treated as `moderator` (the Node accepts the Event but applies `moderator` behaviour).
3. Update `member_temperature_visibility` in the Space state.

### C.8 `xgen-core` — `auto_temperature` reason recognition

**File: `xgen-core/src/space/state.rs`** — handle `membership.kick` and `membership.mute` Events.

When processing either Event:

- If `reason == "auto_temperature"`, this is an automated action issued by a temperature plugin. Apply the standard kick / mute logic; no additional protocol behaviour is triggered by the reason value itself.
- The `cooldown_until` field on the Event is the protocol-level cooldown timestamp. The Space's effective cooldown is whatever the issuing party (plugin) decided; the protocol does not re-interpret it.

This is observability and audit, not a new code path. The standard kick / mute logic already applies; the `auto_temperature` reason simply lets DAG readers know the action was automated.

### C.9 Plugin loader stub (placeholder)

**File: `xgen-node/src/plugins/temperature.rs`** — new module, no-op placeholder.

Per Ch3 §3.7.13.5, the temperature plugin interface is not specified at the protocol level. For Phase 2, implement a no-op placeholder:

- A trait `TemperaturePlugin` with two methods: `compute_room_temperature(space_id, room_id) -> Option<f64>` and `compute_member_temperature(space_id, room_id, member_id) -> Option<f64>`
- A `NoOpTemperaturePlugin` implementation that always returns `None`
- A loader function that returns the no-op by default

When the plugin returns `Some(value)`, the Node attaches the corresponding `xgen.*_temperature` key to relevant outbound Events. When `None`, the key is omitted.

The actual plugin selection mechanism (config-driven, dynamic loading, etc.) is a future Phase 2 implementation decision — out of scope here. The trait surface and the no-op placeholder are sufficient for this task.

### C.10 Client-side rendering

The Ch6 §6.12 DOM contract (`data-temp-state`, `--xgen-*-temperature` custom properties) is implemented in `xgen-client-ui/` Svelte components. **This is UI work; do not implement in `xgen-client/` Rust code.**

The Rust code's job is to expose temperature values via Tauri events to the Svelte layer. Add a Tauri event `temperature_update` carrying:

```rust
pub struct TemperatureUpdate {
    pub space_id: String,
    pub room_id: String,
    pub subject_id: String,       // member_id, or "__room__" for room-level
    pub temperature: f64,
    pub state: String,            // derived bucket: "cool" / "warm" / "hot" / "fiery"
}
```

Bucket derivation runs once on receipt (per Ch6 §6.12.3 — not per frame), using either the Node-supplied `temperature_thresholds` or the Ch6 default thresholds (0.25 / 0.5 / 0.75).

### C.11 Definition of Done — Part C

- [ ] `xgen-common` `meta_atts` validation clamps `xgen.room_temperature` and `xgen.member_temperature` to `[0.0, 1.0]`
- [ ] `EventType::MembershipMute` and `MembershipMuteContent` defined with `target_identity`, `reason`, `cooldown_until`
- [ ] `EventType::StateSpaceTemperatureVisibility` and `StateSpaceTemperatureVisibilityContent` defined
- [ ] `SpaceState.member_temperature_visibility` with default `moderator`
- [ ] `TemperatureThresholds` struct defined; validated per Ch3 §3.7.13.2 when present
- [ ] Node-side filter on outgoing `xgen.member_temperature` honours the three visibility values
- [ ] `xgen.room_temperature` is never filtered — always delivered to every Room member
- [ ] `state.space_temperature_visibility` update Event handler: owner-only, value validation, state update
- [ ] `membership.kick` and `membership.mute` accept the `auto_temperature` reason with no special protocol behaviour beyond audit
- [ ] `NoOpTemperaturePlugin` placeholder in `xgen-node` returns `None` for both methods
- [ ] Tauri `temperature_update` event surfaces float + derived state to the UI layer
- [ ] Unit tests: visibility filtering for each of the three values; threshold table validation; reason value handling; member_temperature filtering vs room_temperature non-filtering
- [ ] `cargo test` passes — quote actual test count from output
- [ ] `cargo build --release` clean

---

## Verification — End-to-End

After all three Parts are complete, run the following end-to-end verification. This is a manual or scripted scenario, not an automated test.

1. Start two `xgen-node` instances locally
2. Federate them
3. Register one human Identity (`is_ai = false`) and one AI Identity (`is_ai = true`, `ai_capabilities: { dm_initiate: false, spontaneous_post: false }`) on Node A
4. Create a Space with `human_pacing_ms: 500`, `ai_pacing_ms: 2000`, `member_temperature_visibility: "moderator"`
5. Both Identities join the Space
6. Attempt `state.dm_space_create` from the AI Identity → MUST be rejected with error 3042
7. Attempt `identity.update` changing `is_ai` → MUST be rejected with error 3041
8. Send messages from both Identities respecting pacing — MUST succeed
9. Send messages from human Identity faster than `human_pacing_ms` — client queue MUST delay them silently
10. Send messages from AI Identity faster than `ai_pacing_ms` — client queue MUST delay them; queue state MUST be visible via `get_pacing_state`
11. Update Space pacing via `state.space_pacing` from the Space owner — MUST succeed; non-owner attempt MUST be rejected
12. Update visibility via `state.space_temperature_visibility` from owner — MUST succeed; non-owner attempt MUST be rejected
13. Issue `membership.mute` with `reason = auto_temperature` from a moderator → mute MUST be applied with `cooldown_until`

Document each step's actual output in `JOURNAL.md` per CLAUDE.md Rule 2.

---

## Out of Scope

- The mathematical model for computing temperature values (plugin-owned; see D-061)
- The Phase 3+ Node-side enforcement of pacing (Ch3 §3.7.12.4 defers this)
- The Phase 3+ Node-side enforcement of `spontaneous_post` (Ch3 §3.6.10.4 defers this)
- The Svelte UI components rendering `data-is-ai`, `data-temp-state`, `data-pacing-state` — these live in `xgen-client-ui/` and are tracked separately as Ch6 implementation
- Migration of legacy Spaces that pre-date the new fields — all new fields have serde defaults, so legacy spaces inherit defaults automatically; no migration code is required
- Slovak translation of the new sections — single translation pass after full document completion (CLAUDE.md)

---

## References

- Ch3 §3.6.10 — AI Identity Extension (full spec)
- Ch3 §3.7.8 — `membership.mute` and standard reason values
- Ch3 §3.7.12 — Pacing Rules on Spaces (full spec)
- Ch3 §3.7.13 — Temperature Property (full spec)
- Ch6 §6.12 — Temperature Property (client display)
- Ch6 §6.13 — AI Member Badge (client display)
- Ch6 §6.14 — Pacing Queue (client behaviour)
- DECISIONS.md D-059 — AI users as first-class Identities
- DECISIONS.md D-060 — Per-space pacing rules
- DECISIONS.md D-061 — Room temperature: protocol carries the signal, plugin owns the math
- CLAUDE.md — Behaviour rules 1–7 (MANDATORY)

---

*This file is the Phase 2 implementation task for AI users, pacing, and temperature. Mark Definition of Done items only when verified with actual command output. Write the JOURNAL.md entry after all three Parts are complete and verified.*
