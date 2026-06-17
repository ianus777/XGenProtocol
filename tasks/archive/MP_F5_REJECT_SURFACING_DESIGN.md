# Design + Runbook — MP-F5: client reject-surfacing + C6 reject-oracle reconciliation
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Status

Phase-0 grounding complete ([MP_F5_REJECT_SURFACING_AUDIT.md](MP_F5_REJECT_SURFACING_AUDIT.md) v1.0).
Design + runbook folded (the arc is bounded — finish the MP-F2 surfacing into the
client reply + rewrite the C6 read-path; not an oracle redesign). **Awaiting
Joe-lock** — F2 (ErrorBody shape) + F4 (D-9 amendment) are the crux; F1/F3/F5 are
design-settled with the recommendations below. Production arc (xgen-core transport
+ xgen-client + xgen-common envelope) + xgen-mptest oracle + matrix annotations,
under full protocol-change discipline.

---

## 2. Fork resolutions (F1/F3/F5 recommended; F2/F4 = Joe-lock crux)

### MP-F5-D1 (F1) — thread the structured reject up to aicontrol [recommended]

The wire code + event_id exist at the transport boundary; they're lost when
`apply_single_event_confirm` flattens to anyhow text. Fix, three small edits:

1. **Widen `EventConfirm::Rejected`** ([connection.rs:109](../xgen-core/src/transport/connection.rs#L109))
   `{ code, reason }` → `{ code, reason, event_id: String }`. Source: `sent_id`,
   already in scope at the match ([connection.rs:190](../xgen-core/src/transport/connection.rs#L190)) —
   the `Error` frame's event_id equals the sent id (that's the correlation key).
2. **Typed reject error** in `ops.rs`: `struct VerbReject { code: u32, event_id:
   String, reason: String }` (impl `std::error::Error`). `apply_single_event_confirm`'s
   `Rejected` arm returns `Err(anyhow::Error::new(VerbReject{..}))` instead of
   `bail!(string)`. All **8** single-event ops (create-space/room, invite, join,
   leave, send, ai-delegate/revoke) inherit it through the one helper — no
   per-op change.
3. **aicontrol downcast** ([aicontrol.rs:481](../xgen-client/src/aicontrol.rs#L481)):
   the `Ok(Err(e))` arm does `e.downcast_ref::<VerbReject>()` → `Some` builds a new
   `DispatchError::VerbReject { code, event_id, reason }`; `None` keeps the existing
   `ClientVerb(String)` path (every non-reject anyhow is unchanged). `VerbReject` is
   the anyhow root (no `.context()` on the reject path), so the downcast holds.

`create_dm_space`'s multi-event chain ([ops.rs:672/701](../xgen-client/src/ops.rs#L672))
is **out of scope** (abort-on-failure, no C6/ban witness uses it) — flagged, not touched.

### MP-F5-D2 (F2) — ErrorBody shape [**JOE-LOCK**]

Add to `ErrorBody` (shipping [envelope.rs:106](../xgen-common/src/aicontrol/envelope.rs#L106)
**and** harness mirror [wire.rs:75](../xgen-mptest/src/wire.rs#L75), both
`#[serde(skip_serializing_if=Option::is_none)]` / `#[serde(default)]`):
- `reject_code: Option<u32>` — the **wire** code (3030/3045/…);
- `event_id: Option<String>` — the rejected event's id.

**Recommended (AC-D2-preserving):** keep `code: "GENERIC_4000"` as the
client-surface code and `category: Protocol` — the wire semantics ride the
**additive** `reject_code`. This honours the AC-D3d invariant ("a control code can
never represent a verb error; ops::* → GENERIC_4000/Protocol") while making the
reject structurally observable. `DispatchError::VerbReject::into_body` fills
`reject_code`/`event_id`; `message` keeps the human text.

**Sub-fork (Joe's call):** map `category` from the wire band (3030 → `Permission`)
so MP-A-03's matrix-wished `category=permission` is literally satisfied. **LOCKED:
defer** (Joe, 2026-06-10) — `reject_code == 3030` is a strictly stronger, more
precise assertion than `category=permission`, **so MP-A-03's matrix "category"
clause is satisfied-by-stronger-means (the exact wire code), not dropped**; and a
band→category table is new client surface that would walk into the open
3030-vs-3010 spec drift (MP-F2-followon) the code-based assertion sidesteps. Surface
`reject_code`; category-remap is an optional later enrichment.

**F2 LOCKED (Joe, 2026-06-10):** additive `reject_code` + `event_id`, retain
`code="GENERIC_4000"` / `category=Protocol`. AC-D3d-preserving (the control-plane
wall stays intact; the reject is surfaced through new additive fields, not by
overloading `code`); additive `Option` fields are wire-safe (old readers ignore
them). Genuinely "finish the MP-F2 surfacing," not a reshape.

### MP-F5-D3 (F3) — C6 oracle rewrite [recommended]

`assert_rejected_no_membership` → **assert-the-reject** (replacing the stale
fire-and-forget read):
1. the offending op's reply is `Reply::Error` (`!is_ok()`), and its `error()`
   carries the expected `reject_code` (e.g. 3030 / 3045) + a `Some(event_id)`;
2. **protected state unchanged** — target not in `alice-view` membership;
3. **offending event absent everywhere** — `rejection_verdict(transcripts,
   event_id)` using the `event_id` from the error body (no longer from an `Ok`
   reply's data).
Add a harness accessor (`reply_err_for`/`error_field`) sibling to `reply_field`.
Applies to A-02/04/17/20 + the new MP-A-03. (MP-A-17's no-leak check is unchanged
in spirit — the bogus-space send still errors + S stays `{alice:owner}`.)

### MP-F5-D4 (F4) — D-9 amendment [**BLESSED, Joe 2026-06-10**]

Amend MP-R1-D9 (design §10): *"For **locally-submitted single-event** rejects (the
path F1 fixes), the reject IS batch-observable post-MP-F2 — the node sends the wire
code + event_id (`reject_signal`), and the client surfaces them structurally in the
aicontrol error reply (`reject_code` + `event_id`). The C6 oracle asserts the reject
directly; the pre-MP-F1a/MP-F2 'fire-and-forget, no recv, category-not-observable'
premise is superseded **for that path**."*

**Scope bound (Joe):** the amendment is **scoped to locally-submitted single-event
rejects** — it does **not** over-claim for the multi-event chain (`create_dm_space`,
out of F1 scope) or federated-reject paths (`reject_signal` is locally-submitted-only
gated). Keeps the amendment honest + bounded. Written into the canonical record by
Chat at close.

### MP-F5-D5 (F5) — stale-row annotation [recommended]

Re-ground A-02/04/17/20 matrix rows against the rewritten oracle (note: PASS rows
were stale pre-MP-F2; now assert-the-reject). Flip MP-A-03 → ✅ PASS (the auth-tier
deferred witness greens here). ban's MP-C-09/MP-A-14 consume the rewritten oracle
in **ban's** arc, not here.

---

## 3. Change surface

| # | Crate / file | Change |
|---|---|---|
| 1 | xgen-core `transport/connection.rs` | widen `EventConfirm::Rejected` + carry `event_id` from `sent_id` |
| 2 | xgen-client `ops.rs` | `VerbReject` type; `apply_single_event_confirm` returns it on Rejected |
| 3 | xgen-client `aicontrol.rs` | `DispatchError::VerbReject` + downcast in the `Ok(Err)` arm + `into_body` fills `reject_code`/`event_id` |
| 4 | xgen-common `aicontrol/envelope.rs` | `ErrorBody.reject_code` + `.event_id` (additive, skip-if-none) |
| 5 | xgen-mptest `wire.rs` | mirror the two fields (+ drift-lock test) |
| 6 | xgen-mptest `tests/mp_r1_c6.rs` | rewrite the reject oracle (MP-F5-D3) + add `mp_a_03_*` |
| 7 | `docs/tests/multiparty_scenarios/MP-A-03/*` | new batch (alice `create-space --auth-tier 2` + room; bob `join`) + manifest |
| 8 | matrix | A-02/04/17/20 re-ground; MP-A-03 → ✅ |

Wire-neutral on the **event** path (no Event/canonical change); the `ErrorBody`
additions are additive transport-reply fields (forward-compatible — absent on
pre-MP-F5 peers, `#[serde(default)]`).

## 4. MP-A-03 scenario (greens here)

`MP-A-03/*` batch, C6 (`mp_r1_c6`), mirroring MP-A-04's single-node shape:
- **alice:** register → `create-space --auth-tier 2` (bind `s`) → `create-room`.
- **bob:** register → `join --space {{space_id}}` (Tier-1, `assertion_tier_of`=1).
  bob's join chains off S's tip (`ops::join` invite-bootstrap → DAG-tip fallback);
  the node refuses at step-4 PG-13 → wire **3030**.
- **exports:** bob.b1.identity_id → `bob_identity_id`; alice.a2.space_id →
  `space_id`; alice.a3.event_id → `space_ready`. **wait:** bob.b2 on `space_ready`
  (bob joins after S+room exist — MP-C-02 ordering discipline).

**Oracle (MP-F5-D3):** bob's `join` reply is an `Error` with `reject_code == 3030`
+ `event_id` present; the join event is absent on every node; bob ∉ `alice-view`
membership (S = `{alice:owner}`, converged).

**RED-on-revert (J-323):** revert the auth-tier literal swap (`args.auth_tier`→`1`,
already shipped — so instead neuter MP-F5's surfacing OR set `--auth-tier 1` in the
batch) → S is Tier-1 → bob joins, reply is `Ok`, bob a member → all three oracle
clauses flip RED. (The auth-tier verb's own RED-on-revert is the literal swap;
MP-F5's RED-on-revert is the surfacing — neuter `VerbReject`/`reject_code` → the
op reply loses the structured code → the oracle's clause (1) fails.)

## 5. Runbook (single commit; checkpoint after step 3 if the downcast shape needs a Joe look)

1. Widen `EventConfirm::Rejected` (+ event_id source). Update its 1 match arm + any constructors.
2. `VerbReject` + `apply_single_event_confirm` typed return.
3. `DispatchError::VerbReject` + downcast + `into_body`.
4. `ErrorBody` fields (envelope + mirror) + drift-lock test.
5. Oracle rewrite + harness accessor; rewrite A-02/04/17/20 assertions; add `mp_a_03_*`.
6. `MP-A-03/*` batch + manifest.
7. Matrix: A-02/04/17/20 re-ground + MP-A-03 → ✅ (Chat doc-bridge at close).

**Verification:**
- build 0 + clippy clean (default + `--all-features` + `--features harness-control`).
- fast suite green; **the C6 heavy tranche GREEN on HEAD** (`cargo test -p xgen-mptest --test mp_r1_c6 -- --ignored`: A-02/04/17/20 **and** A-03 all PASS — the regression closed + the new witness lands).
- RED-on-revert demonstrated (neuter the surfacing → A-03 clause (1) RED; restore → GREEN).

**DoD:**
- [x] Structured reject surfaced: the 8 single-event ops (via `apply_single_event_confirm`) carry `reject_code` + `event_id` in the aicontrol error reply.
- [x] `ErrorBody` additive fields (envelope + mirror), forward-compatible, drift-lock test (`reply_error_parses_mp_f5_reject_fields`, incl. the absent/old-reader case).
- [x] C6 oracle rewritten (assert-the-reject); **A-02/04/17/20 GREEN on HEAD again** (the J-321 regression closed).
- [x] MP-A-03 batch + runner GREEN (reject_code 3030 + absent + bob not-a-member).
- [x] RED-on-revert witness: flipping MP-A-03's batch to `auth_tier:1` → bob's join succeeds (reply `Ok`) → the oracle's `.error()` read fails → RED; restored → GREEN.
- [ ] Matrix re-grounded (A-02/04/17/20) + MP-A-03 → ✅ (**Chat** doc-bridge — empirical results below).
- [ ] D-9 amendment recorded (**Chat**); D-9-favorable, scoped to locally-submitted single-event rejects.
- [x] build 0 + clippy clean (default + `--all-features`) + fast suites green (xgen-core 689, xgen-common 140, xgen-client 103, xgen-mptest 73, xgen-node 286) + C6 heavy tranche **5/5 GREEN**.

**Empirical reject_codes (for the matrix re-grounding, observed on HEAD):**
MP-A-02 → **3045** (`invite_validity_exceeds_max`) · MP-A-03 → **3030** (`tier_mismatch`, the auth-tier witness) · MP-A-04 → **4000** (step-11 non-member; unmapped variant) · MP-A-17 → **4000** (wrong-space_id; unmapped) · MP-A-20 → **4000** (`PermissionDenied`/`can_invite`; unmapped). The three 4000s are pinned to the observed value with an MP-F2-followon note (a future remap re-grounds them here). All five assert-the-reject + paired (absent + protected-state-unchanged).

(No "commit pushed" item — `Status: COMPLETED` is the shipped signal. Clair's
code + arc-doc commit precedes Chat's doc-bridge.)

---

Per D-065 + D-069 + D-070 + D-071 + D-074. MP-R1-D9 (amended favorably here, F4) +
MP-R1-D10 (loop-to-green) govern. MP-F5-D# arc-local (D-069); the D-9 amendment is
the one cross-arc record (Chat, at close).
