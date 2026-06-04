# XGen Protocol — Arc E (Primitive Completion) Design
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Scope + locked decisions

Design beat for Arc E, backed by `tasks/ARC_E_PRIMITIVES_AUDIT.md` v1.0. **Scope: PG-03 (TrustAssertion) + PG-08 (Thread); Role model OUT.** Sequence: PG-03 → PG-08 → close. All `AE-D#` are **arc-local** (D-069) — no DECISIONS.md change at open; promotion eval at close (D-074). Doc-only — suite at J-244's **1060**/0/2, not re-run.

| ID | Decision | Lock |
|---|---|---|
| AE-D1 | TA wire schema authority + drift | ch3 §3.8.4 is **wire-authoritative**; AppC class reconciled to it at close |
| AE-D2 | TA canonical sign/verify | Reuse `canonical_event_bytes`; Ed25519 verify against `issuer`; `TrustAssertionXgid::from_assertion` = hash of canonical bytes |
| AE-D3 | Registration steps 5–7 activation | **Full 7-check `validate_assertion` fn, real**, against a minimal config-backed trusted-issuer set + clock; Local Node still bypasses (§3.8.8); exercised by synthetic-issuer tests |
| AE-D4 | TA scope line | Struct + validation + synthetic issuer **IN**; live Auth Module service **OUT** (Tier 2–4 institutional) |
| AE-D5 | Pass-2 framing | **Arc-E-local**, cross-referenced to the `flavours.rs` Pass-2 note (not a formal XGID Retrofit Pass) |
| AE-D6 | Thread event set | **3 events**: `thread.create` + `thread.resolved` + `thread.archived`; `ThreadStatus = {Open, Resolved, Archived}` |
| AE-D7 | Thread state + convergence | `ThreadState` in `SpaceState`; `state_key_for_event` arms for resolve/archive; appliers convergence-clean (ride M8 `derive_resolved`) |
| AE-D8 | Thread flavour stance | **Stay conceptual** `xgen://thread/sha256:` — **no `ThreadXgid`** in Arc E (consistent with AppC / Phase-3) |
| AE-D9 | Thread `auth_tier_min` gate | Reuse the PG-13 join-gate path; narrow-not-widen vs parent Room; honest no-op until PG-03 + a real module |
| AE-D10 | Protocol/client line | Per-Room-type Thread behaviour + notifications → **client/UI milestone**, not Arc E |

**Reversal recorded (D-065).** The audit (AE-A7) mused that an optional `jurisdiction` slot might be "cheap forward-compat." Design **reverses this — `jurisdiction` is OMITTED now.** TrustAssertion is a SignedPrimitive whose canonical field set is **locked by §3.8.5** (`type, tier, issuer, identity_id, issued_at, valid_until, claims`). Any added field is either (a) inside the signed canonical form — diverging from the spec'd field order + signature contract — or (b) an unsigned side-field, which is wrong for an entity defined by independent verifiability. Neither is "cheap." `jurisdiction` is properly placed when PG-04 (arc G) designs federation jurisdictional namespacing — likely as a `claims` entry or a schema revision. AppC's `jurisdiction` row is reconciled at close (marked Phase-3, per the AppC header's own convention for un-flavoured/future fields).

---

## §2 — PG-03: TrustAssertion design

### 2.1 The struct (`xgen-common`)

New `TrustAssertion` in `xgen-common` (home of wire types; the `TrustAssertionXgid` flavour already lives in `xgen-common/src/xgid/`). Fields **exactly per ch3 §3.8.4**, canonical order per §3.8.5:

```
struct TrustAssertion {
    tier: u32,
    issuer: String,        // pubkey_uri of the Auth Module
    identity_id: String,   // pubkey_uri of the subject Identity
    issued_at: String,     // RFC 3339 UTC
    valid_until: String,   // RFC 3339 UTC  (NOT expires_at — AE-D1)
    claims: TrustClaims,
    signature: String,     // ed25519:KEY:base64url over canonical form
}
struct TrustClaims {       // tier_verified mandatory; rest optional
    tier_verified: bool,
    email_verified: Option<bool>,
    phone_verified: Option<bool>,
    email_hash: Option<String>,
    phone_hash: Option<String>,
    // unknown keys preserved (key-level forward-compat, mirrors AiCapabilities.extra)
}
```

`type: "trust_assertion"` is the canonical-form discriminator (emitted in canonical bytes, per §3.8.5 field order) — represented as a serde tag or injected at canonicalisation, **confirm-at-pickup** (§4). Unknown `claims` keys are preserved round-trip (the Appendix I §I.2 open-namespace stance + the existing `AiCapabilities.extra` precedent).

### 2.2 Canonical form, signing, id (AE-D2)

- **Canonical bytes** = `canonical_event_bytes(&to_value(&assertion_without_signature))` — same machinery as Events (§3.2.4: no whitespace, sorted keys, UTF-8), field order from §3.8.5. The `signature` field is excluded from the signed bytes.
- **Verify**: Ed25519 verify the `signature` against the `issuer` pubkey over the canonical bytes. Reuse the existing `signing.rs` verify path.
- **Id**: `TrustAssertionXgid::from_assertion(&ta)` lands as the thin wrapper the `flavours.rs:35` Pass-2 note deferred — `from_canonical_bytes(&canonical_bytes(&ta))`. This closes the deferral (AE-D5 cross-ref).

### 2.3 The validation function (AE-D3) — activates steps 5–7

New `validate_assertion(assertion, registering_identity_id, required_tier, trusted_issuers, now) -> Result<(), RegistrationError>` implementing all seven §3.8.5 checks:

| Step | Check | Error (code) | Locality |
|---|---|---|---|
| 1 | `issuer` ∈ trusted-issuer set | `AssertionIssuerUntrusted` (new, 3006) | needs trusted-list |
| 2 | signature verifies against `issuer` | `AssertionSignatureInvalid` (3004) | pure-local |
| 3 | `identity_id` == registering Identity | `AssertionIdentityMismatch` (new, 3007) | pure-local |
| 4 | `tier` ≥ required tier | `TierMismatch` → 3030 (reuse `tiers.rs`) | pure-local |
| 5 | `valid_until` in the future | `AssertionExpired` (3005) | pure-local (clock) |
| 6 | `claims.tier_verified == true` | `AssertionClaimsInsufficient` (new, 3008) | pure-local |
| 7 | Node-policy required contact claims present | `AssertionClaimsInsufficient` (3008) | needs Node policy |

Steps 2/3/4/5/6 are **pure-local and always run** once an assertion is present. Steps 1 + 7 consult a **minimal config-backed trusted-issuer list** (`[node].trusted_auth_modules` — a set of pubkeys + optional required-claims policy; **empty by default**). New error variants (3006/3007/3008) extend `RegistrationError`; 3004/3005 stop being dead code.

### 2.4 Registration wiring (the seam)

`accept_registration` (`registration.rs:193`, `!local_node` branch at :233): replace the bind-and-drop

```
let _assertion = trust_assertion.ok_or(TrustAssertionRequired)?;   // step 4 only, today
// Steps 5–7 deferred to Phase 2
```

with: parse the assertion JSON → `TrustAssertion` (tolerant of unknown claim keys), then `validate_assertion(...)`. **Local Node mode still bypasses entirely** (§3.8.8 — the `if !local_node` guard is unchanged; honest no-op posture, AE-A9). `accept_registration` gains a `trusted_issuers` / policy parameter (or reads it from a passed config handle — **confirm-at-pickup**, mirrors the M7-standalone `Arc<Mutex<NodeConfig>>` precedent / CP-1).

### 2.5 Tier source rewire (closes the PG-03 ↔ PG-13 pair)

Arc D's `assertion_tier_of(record)` (`runtime.rs`, PM-D2) reads `record["tier"]` heuristically. With a validated `TrustAssertion` persisted on the `IdentityRecord`, the tier becomes authoritative: `assertion_tier_of` reads the validated `assertion.tier` when present, falling back to the PM-D2 heuristic (`None→1`) only for Local-Node / assertion-absent records. The PG-13 gate now carries a real value at Tier 2–4; Tier-1 stays the honest no-op. **The single PG-03 upgrade site** documented at PM-D2 is honoured here.

### 2.6 Honesty (D-065)

Real validation logic, real Ed25519 verify, real expiry/claims enforcement — **exercised by synthetic-issuer tests** (a test Auth Module keypair signs assertions; the trusted-list is seeded with its pubkey in-test). No live Auth Module ships (AE-D4). Tier-1/Local-Node bypasses, so in today's deployments the path is dormant-but-correct — the same shape Arc D used for the tier-gate. Recorded plainly at close so nothing is overclaimed.

---

## §3 — PG-08: Thread design

### 3.1 Events + status (AE-D6)

Three new `EventType` variants in `wire.rs` (following the existing ~55-variant pattern; `as_str`/`from_str` strings `thread.create` / `thread.resolved` / `thread.archived`):

- **`thread.create`** — origin Event. Content carries `room`, `title` (optional), `auth_tier_min`, initial content. Anchored to a Room; its Event id is the Thread's `origin_event`. Thread id = `xgen://thread/sha256:<hash of canonical create-event>` (AE-D8 conceptual, no flavour).
- **`thread.resolved`** — State Event; transitions status Open→Resolved.
- **`thread.archived`** — State Event; transitions status Open→Archived.

```
enum ThreadStatus { Open, Resolved, Archived }   // open on create
```

A Thread is **never deleted** (ch2) — no `thread.delete`. The absence is the guarantee (mirrors room/space).

### 3.2 State + convergence (AE-D7)

`ThreadState` in `SpaceState` (sibling to `RoomState`), keyed by `thread_id`:

```
struct ThreadState {
    id: String,            // xgen://thread/sha256:
    room_id: String,       // parent Room (one Room, flat — no nesting)
    created_by: String,    // IdentityXgid  (reconciles AppC: created_by present in ch2 anatomy)
    created_at: String,
    title: Option<String>,
    status: ThreadStatus,
    auth_tier_min: u32,
    origin_event: String,
}
```

- **`apply_event` arms** for the three thread events: create → insert `ThreadState{status: Open}`; resolved/archived → mutate `status` (idempotent — re-applying the same transition is a no-op; convergence-clean).
- **`state_key_for_event`**: `thread.resolved`/`thread.archived` → `Some((EventType, thread_id))` so concurrent resolve-vs-archive converges via M8 `derive_resolved`. `thread.create` is a creation event (like `room.create`) — root-ish, not a conflict key.
- **Convergence note**: concurrent `resolved` + `archived` on the same thread is a genuine state-key conflict → resolved by the seven-layer `resolve()` (no hardcoded Layer-1 pair for these; falls to Layer 4 role / Layer 5c lexicographic). Design records that **resolved-vs-archived has no semantic winner** — lexicographic backstop is acceptable (both are terminal read-only states; the distinction is advisory). Flag for M9 only if a deployment needs a policy.

### 3.3 Validation + permission (AE-D6/D9)

- `thread.create`: sender must be a Room member meeting the Room's `auth_tier_min`; the Thread's own `auth_tier_min` **must be ≥ the Room's** (narrow-not-widen, ch2 L660) — reject otherwise (`BAD_ARGUMENT`).
- `thread.resolved`/`thread.archived`: permission-gated via Arc-D `check_permission` — provisionally `ChangeInfo`-class (a moderator/role action), **confirm the exact RoomPermission at pickup** (reuse the Arc-D `RoomPermission` enum; may add a `ManageThreads` variant or fold into `ChangeInfo`).
- **`auth_tier_min` participation gate (AE-D9)**: posting an Event into a Thread checks the joiner's tier ≥ the Thread's `auth_tier_min`, reusing the `verify_tier_assertion` path PG-13 wired. Honest no-op at Tier-1 until PG-03 gives it teeth — which is why PG-03 ships first.

### 3.4 Builder + tests

- `build_thread_create_event` / `build_thread_resolved_event` / `build_thread_archived_event` builders (sign + `prev_events` discipline, mirror `build_room_create_event` — heed D-076 v1.1: a create event's `prev_events` must seed correctly, not `vec![]`).
- Tests: type round-trip; applier create/resolve/archive; convergence (concurrent resolve-vs-archive permutations converge); narrow-not-widen reject; permission gate; `auth_tier_min` participation gate (synthetic, honest-no-op-aware).

### 3.5 Out of scope (AE-D10)

Per-Room-type Thread behaviour (`room.forum` threads-as-flow, `room.announcements` reply-threads, `room.stage` companion thread) and the notification model are **client/UI-milestone** work — the protocol ships the `status` field + event types + the lifecycle, and stays thin (ch2's own statement). Arc E does not implement Room-type-specific thread semantics.

---

## §4 — Sequence, commits, confirm-at-pickup

**Block C1 — PG-03** (`xgen-common` struct + canonical/sign/verify + `TrustAssertionXgid::from_assertion` → `xgen-core` `validate_assertion` + registration wiring + `assertion_tier_of` rewire + new error codes 3006/3007/3008 + synthetic-issuer tests). Heaviest strand; lands first.

**Block C2 — PG-08** (`wire.rs` 3 EventTypes + `ThreadStatus` → `ThreadState` + appliers + `state_key_for_event` arms → validation + permission + `auth_tier_min` gate → builders + tests). Rides the M8 state path.

**Close** — D-074 doc-only: gap-audit §5 PG-03 ✅ / PG-08 ✅; AppC reconcile (`valid_until`, `jurisdiction`→Phase-3, Thread `created_by`); ROADMAP; JOURNAL; AE-D# promotion eval (likely all stay arc-local — none is a cross-arc invariant); honesty note (PG-03/PG-09-style dormant-but-correct).

**Confirm-at-pickup (D-078) — resolve at the relevant block, not blockers:**
- **CP-1** — `type` discriminator representation in the TA canonical form (serde tag vs inject-at-canonicalisation) so the signed bytes match §3.8.5 exactly (C1).
- **CP-2** — how `trusted_issuers` + required-claims policy reach `accept_registration` (new param vs config handle; mirror M7S CP-1 `Arc<Mutex<NodeConfig>>`) (C1).
- **CP-3** — exact `RoomPermission` for resolve/archive (reuse `ChangeInfo` vs new `ManageThreads`) (C2).
- **CP-4** — `thread.create` `prev_events` seeding under D-076 v1.1 (don't repeat the `vec![]` bug) (C2).

**No DECISIONS.md change at open** (AE-D# arc-local, D-069). **No code** until the runbook lands and Joe approves. Clair stands down until pickup.

---

**Design complete (v1.0).** AE-D1–D10 locked (jurisdiction-omit reversal recorded); PG-03 and PG-08 designs detailed; four confirm-at-pickup. Feeds the runbook `tasks/ARC_E_PRIMITIVES_IMPL.md`.
