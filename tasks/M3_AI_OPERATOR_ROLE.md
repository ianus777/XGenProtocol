# M3 — AI Operator Role & Delegation
> **Status**: PENDING  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-17 (created at M2 close-out — J-074 sequel; architecture locked by Joe 2026-05-16)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this exists

M2 (J-074) closed the per-binary deployment story. M3 is the first M-numbered milestone that lands new **protocol semantics**, not just plumbing: the operator role for AI Identities inside Spaces, the delegation/revoke event handlers that move it, and the fall-upward resolution that guarantees responsibility never goes orphan.

Phase 2 (J-065) shipped the wire types for `state.ai_operator_delegate` / `state.ai_operator_revoke`, the `is_ai` / `ai_capabilities` registration shape, and one enforcement gate (`check_ai_capability` blocks `state.dm_space_create` when an AI lacks `dm_initiate`). What's missing is everything that turns the operator role from a serialisable shape into a live, resolvable, federation-propagating piece of Space state.

M3 also adds the minimum Client CLI surface (`init --ai`, `ai delegate`, `ai revoke`) needed to test the protocol end-to-end without a separate AI binary. The AI Client *binary* (a long-running daemon that consumes these primitives) is out of scope and deferred to a later milestone.

---

## Architectural foundation — LOCKED (per Joe, 2026-05-16)

The following are not open for re-design during M3. Implementation choices (data structure shapes, function signatures, naming) remain to be settled, but the architecture below is fixed.

### Operator role model

Operator is **its own role**, distinct from member and admin. Sits between member and admin in *privilege scope*, not in the hierarchy:

| Role | Scope |
|---|---|
| Member | Baseline Space participation |
| **Operator** | Member + special privileges over a specific AI (protocol-enforced privileges accrue later as features grow; M3 has none) |
| Admin | Space-wide privileges |
| Owner | Space-wide privileges + structural authority |

Operator is per-(AI, Space), not per-Identity. The same human Identity can be operator of one AI in one Space, a plain member in another Space, and an admin in a third.

### Delegation flow

1. Admin (or owner) chooses a current Space member they want to operate AI-X.
2. Admin issues `state.ai_operator_delegate` naming that member as `new_operator_identity_id`.
3. Once the event is DAG-accepted, that member holds the operator role for AI-X in Space S.

There is no "transfer" mechanism where the current operator signs over to the next. M3's signer model puts delegate authority entirely in {admin, owner}; the previous operator's consent is not required.

### Signer rules (LOCKED)

| Event | Valid signer | Other constraints |
|---|---|---|
| `membership.invite` | owner OR admin | (unchanged from spec 3.7.8) |
| `state.ai_operator_delegate` | owner OR admin | `new_operator_identity_id` MUST be a current Space member; `ai_identity_id` MUST be a current Space member with `is_ai = true` |
| `state.ai_operator_revoke` | owner OR admin | `ai_identity_id` MUST be a current Space member with `is_ai = true` |

**Operator never signs in their operator capacity in M3.** Operator privileges that require operator-signed events arrive in future milestones, layered on M3's resolution function.

### Fall-upward resolution principle

The current operator for AI-X in Space S is computed on demand. SpaceState stores explicit delegations and the membership chain; resolution walks upward when the stored value is unreachable:

```
resolve_operator(space, ai_id) -> identity_id:
    1. If a stored delegation exists for ai_id AND the delegate is a current Space member:
         return that delegate.
    2. Else if the AI's recorded inviter (membership.invite.sender) is a current Space member:
         return the inviter.
    3. Else:
         return the Space owner.
```

Two-step fallback. Owner is always present (or the Space is abandoned). **No orphan state is reachable.**

Operator is therefore a *resolved value, not just a stored one*. The stored delegation may name an identity who has since left or been kicked from the Space — the resolution function transparently skips past such records. Revoke explicitly clears the stored delegation, collapsing resolution to step 2 or 3.

### AI-owned Space

**Rejected in v1.** An AI Identity MUST NOT be a Space owner. Pragmatic deferral, not architectural impossibility — revisit when a real use case appears.

### Operator privileges

**None protocol-enforced in M3.** The role exists, assignment is DAG-recorded, fall-upward resolution works. Practical operator privileges (DM command surface, audit access, AI silencing, capability override, etc.) emerge from real usage and future AI capabilities — layered on top of M3, not part of it. When they do land, they will be "is this signer the current *resolved* operator?" checks, not "did this signer sign a delegate event?" checks.

This is the load-bearing M3 invariant: the system always knows who the operator is, even if no delegate event has ever been signed for a given (AI, Space) pair. There is no separately-stored "initial operator." The resolution function, given a Space with an AI member and no explicit delegation, returns the inviter recorded in the membership chain — identical to how the operator is resolved at any other time. **Inviter-is-operator is an output of the resolution function, not stored state.**

---

## Cross-references

| Source | Relevance |
|---|---|
| `docs/xgen_ch3_specification.md` §3.6.10 | AI Identity Extension — registration, immutability, capabilities, enforcement model. §3.6.10.6 will be amended by M3. |
| `docs/xgen_ch3_specification.md` §3.7.4 | DM Space rules. Possibly amended depending on `dm_promotion.rs` inventory (Phase 0 below). |
| `DECISIONS.md` D-059 | AI Identity Extension — already shipped Phase 2; M3 builds on this. |
| `DECISIONS.md` D-060 / D-061 | Pacing / Temperature — already shipped Phase 2; not directly touched by M3 but adjacent. |
| `JOURNAL.md` J-065 | Phase 2 AI / Pacing / Temperature implementation. |
| `xgen-core/src/wire/types.rs` | `StateAiOperatorDelegateContent` / `StateAiOperatorRevokeContent` round-trip in place; `EventType::StateAiOperatorDelegate` / `StateAiOperatorRevoke` reserved. |
| `xgen-core/src/message/exchange.rs::check_ai_capability` | The one enforcement gate already wired (blocks `state.dm_space_create` when AI lacks `dm_initiate`). Reference shape for M3's new signer checks. |
| `xgen-core/src/space/state.rs` | `SpaceState`, `SpaceMember`, role tracking. M3 adds fields here. |
| `xgen-core/src/space/dm_promotion.rs` | Phase 0 inventory required — confirm what's already there and whether it intersects M3. |

---

## Scope

### In scope for M3

1. **`SpaceState` gains the operator state.** A stored-delegations map (per-AI), an authoritative inviter recorded on each `SpaceMember`, and a public `resolve_operator(&self, ai_id) -> Option<String>` method implementing the fall-upward function.
2. **`membership.invite` acceptance** captures the inviter on the resulting `SpaceMember`. If the invitee is an AI (`is_ai = true` on its `IdentityRecord`), the resolution function will return this inviter as initial operator — no separate action needed.
3. **`state.ai_operator_delegate` acceptance.** Validates: signer ∈ {owner, admin}; `ai_identity_id` is a current member with `is_ai = true`; `new_operator_identity_id` is a current member. On success: records the delegation in `SpaceState`. On failure: returns the appropriate error code (see decision #4 below).
4. **`state.ai_operator_revoke` acceptance.** Validates: signer ∈ {owner, admin}; `ai_identity_id` is a current member with `is_ai = true`. On success: clears the stored delegation (so resolution falls through to inviter → owner).
5. **AI-owned Space rejection.** `state.space_create` and `state.dm_space_create` reject when the sender is `is_ai = true`. Error code `3041 ai_role_violation` per locked decision #4.
6. **Client CLI: `init --ai`.** Minimum surface so M3 is testable. `xgen-client init --ai [--cap dm_initiate=true]`. Default capabilities are `dm_initiate=false`, `spontaneous_post=false`. Capability values can be overridden via repeated `--cap key=value`.
7. **Client CLI: `ai delegate` / `ai revoke`.** New subcommand group `ai` with two subcommands: `delegate --space <id> --ai <id> --to <member-id>` and `revoke --space <id> --ai <id>`. Issued by the local Identity (admin or owner of the Space).
8. **`xgen-client whoami` / `status`** surface "AI operator of: …" when the local Identity is the resolved operator for one or more (AI, Space) pairs.
9. **Unit tests** for: each happy path; each signer-validation failure; each membership-validation failure; resolution function with each of the three return cases (stored delegate present, stored delegate gone, no delegate ever recorded).
10. **Two-Node federation smoke** — extends the existing federation test pattern (J-051-era multi-Node tests) with one AI member, one delegate event, one revoke event, verifying the resolved operator matches on both Nodes.
11. **Spec updates** to `docs/xgen_ch3_specification.md` §3.6.10.6:
    - Replace the current "inviter is on record as authorised" framing with the explicit operator role definition.
    - Document the signer rules (LOCKED above).
    - Document the fall-upward resolution algorithm.
    - State the AI-owned-Space rejection explicitly.
    - State the "no protocol-enforced operator privileges in v1" disposition.
12. **DECISIONS.md** entry — one new decision capturing M3's locked architecture so future sessions can find it without diff-archaeology. Suggested ID: D-064.

### Out of scope (deferred)

- **AI Client binary** — a long-running daemon that registers as an AI, joins Spaces, receives events via `run_ws_loop`, responds under pacing rules. M3 lands the primitives; the binary that consumes them is M3+1 or later.
- **`spontaneous_post` Node-side enforcement.** Spec 3.6.10.4 leaves this NOT Node-validated in Phase 2; M3 does not change that.
- **Protocol-enforced operator privileges** (DM command surface, audit access, AI silencing, capability override, etc.). Per the locked architecture: layered on top of M3 when real features need them.
- **Tauri / Svelte UI surface for operator status.** Headless CLI only in M3. UI rendering of "you are operator of X" comes in the resumed UI track.
- **Operator self-transfer** (operator signs over to next operator without admin/owner involvement). Not in M3's signer model; revisit if/when needed.
- **Operator-of-AI cross-Space inheritance.** Operator is strictly per-(AI, Space). No mechanism for "operator everywhere" or "follow me to a new Space."
- **Pacing / temperature plugin math.** Still plugin-owned per J-065. M3 does not touch.
- **Multiparty test redesign.** Still paused; resumes after M3 lands per the M1 task file's standing direction.

---

## Phase 0 — Pre-flight inventory (REQUIRED before any code)

The architectural foundation is locked, but several implementation surfaces need to be inspected before scope is fully concrete. The first session that picks M3 up does Phase 0 *before* writing implementation code:

1. **Capture baseline.** `cargo test --workspace --release` — confirm 391. Quote actual output.
2. **Read `xgen-core/src/space/state.rs` end-to-end.** Map: how is `SpaceMember` shaped today? Does it already record `invited_by`, or does M3 add that field? How are roles (owner/admin/member) tracked — enum, set of admins, role map? Note any existing patterns M3 should follow.
3. **Read `xgen-core/src/space/dm_promotion.rs`.** Confirm what's there. If DM promotion already touches operator semantics in any way, that constrains M3's data shape choices.
4. **Read `xgen-core/src/message/exchange.rs` event-acceptance pipeline.** Where do role events (membership.invite, role changes) currently mutate SpaceState? M3's new event handlers slot into the same pipeline; understand the existing mutation contract before adding new mutations.
5. **Search for any existing operator wiring.** `grep -r "ai_operator" xgen-core/src` confirms whether anything beyond the wire types and EventType variants is in place. (Per my J-074 close-out, the answer is "nothing" — re-verify in case I missed something.)
6. **Identity replication semantics for `is_ai`.** When Node B receives an Identity from Node A via replication (xgen-core/src/identity/replication.rs), does it propagate `is_ai` + `ai_capabilities`? M3's federation smoke depends on this — if it's already wired, good; if not, flag it.

Phase 0 produces a *short* findings note (a few paragraphs in the journal entry, not a separate document) that becomes the basis for the implementation choices in Phase 1.

---

## Implementation decisions — LOCKED (per Joe, 2026-05-17)

The architectural foundation above answers the *what*. These are the *how* details, locked at task-file review time rather than during Phase 1 — same pattern as M2's pre-implementation question batch. The next session implements against these; deviation requires Joe's say-so.

1. **SpaceState data shape:** `ai_operator_delegations: HashMap<String, String>`. Key = `ai_identity_id`, value = currently-delegated operator's `identity_id`; absence means "no explicit delegation; resolution falls through to step 2". Versioned log / per-AI metadata struct add storage cost without changing what `resolve_operator` returns.

2. **`SpaceMember.invited_by: Option<String>`.** Phase 0 confirms whether the field exists. If absent, add it as `Option<String>` — `None` for owner and founding members, `Some` for everyone joined via `membership.invite`. Required by resolution step 2.
    - **Micro-note (Joe, 2026-05-17):** if Phase 0 finds an existing field carrying the same information under a different name (e.g. `inviter`, `invited_via`, `invited_by_id`), **reuse it** rather than renaming. Avoids churn if the field is already there under a slightly different name; the resolution function refers to whatever name exists.

3. **CLI verb shape:** subcommand group — `xgen-client ai delegate` / `xgen-client ai revoke`, not flat verbs. Leaves room for `ai list`, `ai status`, `ai capabilities`, etc. without breaking the verb surface when later milestones land.

4. **Error codes:** reuse existing `3041 ai_role_violation` for new validation failures (wrong signer, target-not-a-member, AI-not-actually-AI). D-059 already defined the slot; finer granularity (3043 / 3044) adds reading load without adding semantic value. `3042 ai_capability_violation` stays reserved for capability-flag enforcement (`dm_initiate` now; future `spontaneous_post`) — not reused for M3's new role/target checks.

5. **`init --ai` capability defaulting:** `--cap key=value` flags, all-false default, no interactive prompt. Matches the existing `--passphrase` pattern (explicit > prompt); keeps the CLI scriptable; default-false matches D-059's restrictive-by-default capability stance.

6. **Federation test depth:** one Node A / Node B setup; one AI; one delegate event; one revoke event; one delegatee-leaves-Space case. Within that setup, all three of the following are verified cross-Node:
    - **Cross-Node delegate.** alice on Node A signs `state.ai_operator_delegate` naming carol; after federation propagation, `resolve_operator(bob_ai)` returns carol on **both** Nodes.
    - **Cross-Node revoke.** alice on Node A signs `state.ai_operator_revoke`; after federation propagation, `resolve_operator(bob_ai)` returns alice (inviter fallback) on **both** Nodes. *(Wire-type symmetry: both delegate and revoke need cross-Node coverage, not just delegate.)*
    - **Fall-upward across federation.** Re-delegate to carol, then kick carol from Node A; without any explicit revoke, `resolve_operator(bob_ai)` returns alice on both Nodes (step 1 transparently skips a delegate who is no longer a member).

7. **`resolve_operator` return shape:** `Option<String>`, not `String` + panic. Defensive; matches the rest of the codebase's preference for not-panicking on theoretically-impossible cases. `None` only fires if a Space has no owner, which would be a structural bug worth surfacing rather than crashing the Node.

---

## Implementation steps (recommended sequence)

### Phase 1 — `SpaceState` shape + resolution function

1. Add `SpaceMember.invited_by: Option<String>` if Phase 0 confirms it's missing.
2. Add `SpaceState.ai_operator_delegations: HashMap<String, String>` (decision #1 names this).
3. Implement `SpaceState::resolve_operator(&self, ai_id: &str) -> Option<String>` per the locked algorithm. Three return cases tested in unit tests.
4. AI-owned-Space rejection: `state.space_create` and `state.dm_space_create` reception in `exchange.rs` reject when sender's `IdentityRecord.is_ai = true`. New unit test per event type.

### Phase 2 — Event acceptance handlers

5. `state.ai_operator_delegate` acceptance: new handler in `exchange.rs` (or wherever the existing role events live, per Phase 0 findings). Validates signer + targets + AI-ness. On success: writes to `ai_operator_delegations`. Returns appropriate error variant on failure (mapped to a wire error code per decision #4).
6. `state.ai_operator_revoke` acceptance: new handler. Validates signer + target. On success: removes from `ai_operator_delegations`. Falls through to inviter / owner via resolution.
7. `membership.invite` acceptance updates `SpaceMember.invited_by` (Phase 1 added the field; Phase 2 populates it).
8. Unit tests for: each handler happy path; each validation failure; resolution function called from inside acceptance to confirm the resolved value is what we expect after each mutation.

### Phase 3 — Client CLI surface

9. `xgen-client init --ai [--cap dm_initiate=true] [--cap spontaneous_post=true]` — emits an AI-flavoured registration request (`is_ai = true` + the capability map). Decision #5 settles capability defaulting.
10. `xgen-client ai delegate --space <id> --ai <id> --to <member-id>` — signs and sends `state.ai_operator_delegate`. Requires the local Identity to be a Space owner or admin.
11. `xgen-client ai revoke --space <id> --ai <id>` — signs and sends `state.ai_operator_revoke`. Same caller requirements.
12. `xgen-client whoami` and `status` surface "AI operator of: <ai_id> in <space_id>" lines when the local Identity resolves as operator for any (AI, Space) pair via the resolution function.

### Phase 4 — Spec + DECISIONS

13. Amend `docs/xgen_ch3_specification.md` §3.6.10.6 per the bullet list in scope item #11.
14. Add `DECISIONS.md` D-064 (or next available) capturing the locked M3 architecture. One paragraph per locked principle, plus the in-scope/out-of-scope split.
15. If Phase 0 finds anything in `dm_promotion.rs` that intersects, add a §3.7.4 amendment.

### Phase 5 — Verification

16. `cargo test --workspace --release` — green at the new test count (391 baseline + however many M3 unit tests land; expected ~410–425).
17. **Two-Node federation smoke** per decision #6. Add as a new test in the existing federation-test pattern. Quotes from the test transcript go in the journal entry.
18. **Three-step manual end-to-end smoke** against running binaries, mirroring the M2 smoke pattern. Sequenced so each step's observable effect is visible (revoke before kick — revoking *after* a kick on the delegatee would have no observable end-state change, since resolution would already have fallen through to inviter):
    - **Setup.** Node A and Node B running. alice (admin/owner) and bob (AI, `dm_initiate=true`) registered on Node A; carol (plain member) registered on Node B. alice creates Space, invites bob (AI) and carol.
    - **Step 1.** With just the invite events on record (no delegate yet), `xgen-client whoami` / `status` resolves alice as operator for bob on both Nodes. Exercises resolution step 2 (inviter fallback) with no stored delegation.
    - **Step 2.** alice runs `ai delegate --space <id> --ai <bob> --to <carol>`. Verify resolved operator is carol on both Nodes. Exercises resolution step 1 (stored-delegation hit) and federation propagation of the delegate event.
    - **Step 3.** alice runs `ai revoke --space <id> --ai <bob>`. Verify resolved operator returns to alice on both Nodes. Observable transition Carol→alice exercises the revoke code path and resolution step 2 (inviter fallback after the stored delegation is cleared).
    - **Optional follow-on** (exercised during implementation if needed): kick carol mid-flight before revoke to verify resolution step 1 transparently skips a delegate who has left the Space (the resolved operator should auto-fall to alice without an explicit revoke).

---

## Definition of Done

- [ ] Phase 0 baseline captured (`cargo test` quoted in journal).
- [ ] Phase 0 inventory done; findings folded into the journal entry.
- [ ] `SpaceState::resolve_operator` implemented and unit-tested (all three resolution cases).
- [ ] `SpaceMember.invited_by` field present and populated by `membership.invite` acceptance.
- [ ] `state.ai_operator_delegate` acceptance handler implemented with all locked signer + target validations; happy path + each failure mode unit-tested.
- [ ] `state.ai_operator_revoke` acceptance handler implemented with all locked signer + target validations; happy path + each failure mode unit-tested.
- [ ] AI-owned-Space rejection live (`state.space_create` / `state.dm_space_create` from `is_ai = true` sender → reject with appropriate error code).
- [ ] `xgen-client init --ai` surface live; AI registration end-to-end against a running Node.
- [ ] `xgen-client ai delegate` and `xgen-client ai revoke` live; both signed by an owner/admin Identity and accepted by the Node.
- [ ] `xgen-client whoami` and `xgen-client status` surface "AI operator of …" lines when applicable.
- [ ] `cargo build --release --workspace` clean (no new warnings beyond M2's 44+1 baseline).
- [ ] `cargo test --workspace --release` green at the new total (expected ~410–425).
- [ ] Two-Node federation smoke runs green; transcript quoted in journal.
- [ ] Three-step manual end-to-end smoke runs green; transcript quoted in journal.
- [ ] `docs/xgen_ch3_specification.md` §3.6.10.6 updated per scope #11.
- [ ] `DECISIONS.md` D-064 (or next available) added.
- [ ] `JOURNAL.md` entry written quoting actual verification output (J-075 if M3 lands in one session).
- [ ] `tasks/M3_AI_OPERATOR_ROLE.md` header flipped from `PENDING` to `COMPLETED`.
- [ ] `CLAUDE.md` updated to reflect M3 done; next session entry point reset (likely the AI Client binary task file, which gets written at M3 close-out).

---

## Behaviour rules reminder (from CLAUDE.md)

- **Rule 1** — Never fabricate results. Real output only.
- **Rule 2** — Show actual output. Quote terminal output verbatim in the journal.
- **Rule 3** — Stop and report when a tool fails.
- **Rule 4** — Write the journal entry last, after verification is confirmed.
- **Rule 5** — Never invent numbers. Test counts from `cargo test` only.
- **Rule 6** — When in doubt, do less and ask. The architectural foundation is locked; the implementation-level decisions are pre-flagged. Anything else that comes up: stop and ask Joe.
- **Rule 7** — Definition of Done is a checklist, not a formality.

If a finding from Phase 0 (e.g. `dm_promotion.rs` already does something that constrains M3's data shape) calls any of the locked architecture into question, **stop and surface it** — do not silently redesign. The architecture is locked precisely because Joe and I worked through the trade-offs together; a unilateral revision in the middle of implementation defeats the point.

---

*End of M3 task file.*
