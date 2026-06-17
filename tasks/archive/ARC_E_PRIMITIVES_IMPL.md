# XGen Protocol — Arc E (Primitive Completion) Implementation Runbook
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Scope + entry

Implementation runbook for Arc E, backed by `tasks/ARC_E_PRIMITIVES_DESIGN.md` v1.0 (AE-D1–D10 locked) + `tasks/ARC_E_PRIMITIVES_AUDIT.md` v1.0. **Scope: PG-03 (TrustAssertion) → PG-08 (Thread) → close.** Role model OUT. Baseline suite **1060**/0/2 (post-J-244). Every commit: `cargo test --workspace` + build all-targets + clippy `-D warnings` (default + `--all-features`), all green on landing. Per-strand blocks; one writer per file per atom (D-074). Clair implements; Joe pushes.

**Reading order at pickup (Rule 0):** CLAUDE.md PLAY → JOURNAL J-245 → this runbook §2/§3 → resolve the block's confirm-at-pickup first.

---

## §2 — Block C1: PG-03 TrustAssertion (the keystone) — ✅ SHIPPED (J-246)

**C1 SHIPPED at J-246 (Clair).** Suite **1093**/0/2 (+33); build all-targets 0; clippy clean (default + `--all-features`). CP-1 resolved (`type` via serde field + `canonical_object_json` with the §3.8.5 TA field order, not `canonical_event_bytes`); CP-2 resolved (`AssertionPolicy` on `NodeRuntime`, no `NodeConfig` dep in core). Wire-code reconciliation (D-065): design-guessed 3006/3007/3008 collide with existing variants → reused 3004/3005/3006/3030 + added **3010**/**3011**; ch3 §3.6.5 table + §3.8 note land **at close**. See JOURNAL J-246.

Lands first (AE §3.1: keystone under PG-13; Thread's tier gate is only honest after it). **Resolve CP-1 + CP-2 before coding.**

### C1 confirm-at-pickup (D-078)
- **CP-1** — the `type: "trust_assertion"` discriminator in the canonical signed form. Read live `canonical.rs::canonical_event_bytes` + how Events carry `type` into canonical bytes; pick serde-tag vs inject-at-canonicalisation so the signed bytes match §3.8.5 field order (`type, tier, issuer, identity_id, issued_at, valid_until, claims`) **exactly**. The signature must be reproducible by any verifier from the struct alone.
- **CP-2** — how `trusted_issuers` + the required-claims policy reach `accept_registration`. Read the live `accept_registration` signature (`registration.rs:193`) + its callers; choose new-param vs config-handle (mirror the M7-standalone CP-1 `Arc<Mutex<NodeConfig>>` precedent). `xgen-core` must not gain a dep on the `xgen-node` `NodeConfig` type — pass a plain set/struct.

### C1 steps
1. **`xgen-common`** — `TrustAssertion` + `TrustClaims` structs (fields per AE-D1 / §3.8.4; `valid_until` not `expires_at`; no `jurisdiction`). `TrustClaims` preserves unknown keys (mirror `AiCapabilities.extra`). Serde round-trip + byte-identity tests.
2. **`xgen-common`** — canonical bytes (reuse `canonical_event_bytes`, signature excluded) + `TrustAssertion::sign(key)` / `verify()` (Ed25519 against `issuer`); land `TrustAssertionXgid::from_assertion` (the `flavours.rs:35` Pass-2 deferral; AE-D5 cross-ref the note). Sign/verify round-trip + tamper-reject tests.
3. **`xgen-core`** — `RegistrationError` += `AssertionIssuerUntrusted` (3006) / `AssertionIdentityMismatch` (3007) / `AssertionClaimsInsufficient` (3008); `validate_assertion(assertion, registering_id, required_tier, trusted_issuers, now)` implementing the 7 §3.8.5 checks (table in design §2.3). Per-check unit tests with a synthetic issuer keypair.
4. **`xgen-core`** — wire `validate_assertion` into `accept_registration` `!local_node` branch (replace the bind-and-drop at :236); persist the validated assertion on `IdentityRecord`. Local Node bypass unchanged. Registration accept/reject tests (each error code) + Local-Node-still-bypasses test.
5. **`xgen-core`** — `assertion_tier_of` (`runtime.rs`, PM-D2) reads the validated `assertion.tier` when present, heuristic fallback otherwise. Pin: a Tier-2 synthetic assertion → join to a `auth_tier=2` Space passes; absent/Tier-1 → the honest no-op holds (guards PG-13 regression).

**C1 honesty (D-065):** validation logic is real + Ed25519-real + exercised by a synthetic issuer; **no live Auth Module ships** (AE-D4). Trusted-list empty by default; Tier-1/Local-Node path dormant-but-correct. Record at close, PG-09-style.

**C1 gate:** workspace tests green (+N), build all-targets 0, clippy clean both feature sets. Joe pushes.

---

## §3 — Block C2: PG-08 Thread — ✅ SHIPPED (J-247)

**C2 SHIPPED at J-247 (Clair).** Suite **1107**/0/2 (+14); build all-targets 0; clippy clean (default + `--all-features`). CP-3 resolved (reuse `ChangeInfo` — no new `RoomPermission`); CP-4 resolved (`prev_events` seed the parent Room, D-076 v1.1). As-built: `state_key_for_event` category refined to shared `"thread.status"` (design's "(EventType, thread_id)" would never conflict); Rooms inherit Space `auth_tier` (no per-Room tier); participation gate on `thread.create`. **Next = close (§4, D-074 doc-only).** See JOURNAL J-247.

Rides the M8 state path. **Resolve CP-3 + CP-4 before coding.**

### C2 confirm-at-pickup (D-078)
- **CP-3** — exact `RoomPermission` for `thread.resolved`/`thread.archived`. Read live Arc-D `membership.rs::RoomPermission` + `check_permission`; reuse `ChangeInfo` vs add `ManageThreads`. Light — design leans `ChangeInfo`.
- **CP-4** — `thread.create` `prev_events` seeding. Heed D-076 v1.1 (the `build_room_create_event` `vec![]` bug); seed correctly so the create event is causally placed in the Room, not a false root.

### C2 steps
1. **`xgen-common`** — 3 `EventType` variants (`thread.create` / `thread.resolved` / `thread.archived`) + `as_str`/`from_str` strings + `ThreadStatus {Open, Resolved, Archived}`. Type round-trip + `from_str` strictness tests.
2. **`xgen-core`** — `ThreadState` in `SpaceState` (design §3.2; includes `created_by`, reconciling the AppC/ch2 anatomy); `apply_event` arms (create inserts Open; resolve/archive mutate status, idempotent); `state_key_for_event` arms for resolve/archive `(EventType, thread_id)`. Applier unit tests.
3. **`xgen-core`** — validation: `thread.create` sender is a Room member + Thread `auth_tier_min ≥ Room auth_tier_min` (narrow-not-widen, reject otherwise); resolve/archive permission-gated (CP-3); per-Thread `auth_tier_min` participation gate reuses the `verify_tier_assertion` path (AE-D9, honest no-op pre-real-module).
4. **`xgen-core`** — `build_thread_create_event` / `_resolved_event` / `_archived_event` builders (CP-4 `prev_events`).
5. **Convergence test** — concurrent `thread.resolved` + `thread.archived` on one thread: assert all arrival permutations converge to an identical snapshot (rides `derive_resolved`; no Layer-1 pair → Layer-5c lexicographic backstop; design §3.2 records resolved-vs-archived has no semantic winner — acceptable, M9-flag only).

**C2 gate:** workspace tests green (+N), build all-targets 0, clippy clean both feature sets. Joe pushes.

---

## §4 — Close (D-074 doc-only, atomic)

- `PROTOCOL_GAP_AUDIT.md` §5: PG-03 ✅ DONE, PG-08 ✅ DONE; update open/done counts; §4-E arc-letter note.
- Appendix C reconcile (AE-D1): `valid_until` not `expires_at`; `jurisdiction` → marked Phase-3; Thread `created_by` row.
- ch3 §3.8: note steps 5–7 now enforced (was "deferred to Phase 2").
- ROADMAP version bump; arc E ⚫.
- JOURNAL close entry; CLAUDE.md PLAY → next-active.
- **AE-D# promotion eval** — likely all stay arc-local (D-069; none is a cross-arc invariant). Honest dormant-but-correct note for PG-03 (AE-A9) + the Thread tier-gate no-op.

**Cannot fold (STOP on drift, named homes):** live Auth Module service → Tier 2–4 institutional · `jurisdiction` / jurisdictional namespacing → PG-04 / arc G · first-class Role object model → privilege-model continuation arc · per-Room-type Thread behaviour + notifications → client/UI milestone.

---

**Runbook complete (v1.0).** C1 (PG-03) → C2 (PG-08) → close. Four confirm-at-pickup (CP-1/CP-2 at C1, CP-3/CP-4 at C2). No blocking Joe-lock remains; design AE-D1–D10 are the locks. Clair stands down until C1 pickup.
