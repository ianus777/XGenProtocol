# MP-F2 — Reject-path wire-code propagation — DESIGN
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-08  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

MP-F2 (routed at MP-R1 C7, J-321; `tasks/MP_findings.md`) is the first **production-crate**
change of the multiparty effort. The finding: event-validation rejections deliver an `Error` wire
frame with `error_code = 4000` (generic) plus the specific reason in the message string, **not** the
`ExchangeError::to_wire_code` value the Node already computed internally. A federated peer or client
cannot programmatically distinguish *why* an event was rejected.

This design executes the three forks greenlit at the Phase-0 audit (J-322 strategy lock MP-R1-D10,
design §11/§12 — fix-and-rerun, loop-to-green):

- **F-1(a)** — widen `DispatchOutcome::Rejected` to carry the structured code as a single source of
  truth (not re-parse a formatted string).
- **F-2** — propagate the already-computed code *this arc only* (closes MP-A-15's 3046). The
  unmapped `ExchangeError` variants + the 3030-vs-3010 spec drift are named as **MP-F2-followon**,
  not absorbed.
- **F-3** — the origin gate (locally-submitted-only emission) is **unchanged**.

Severity: low (observability / wire-contract — rejections already work + events are correctly
absent). Root: a two-boundary code-drop, **not** a wire-shape gap.

Scope: `xgen-core` (`node/runtime.rs`) + `xgen-node` (`app.rs`, `admin_ops.rs`). No protocol wire
shape changes; no admission/ordering surface moves (§6).

---

## 2. Grounding (against live `main`)

### 2.1 The reject path, end-to-end — two lossy boundaries

1. **`validate_event` → `dispatch_event` (the structural carrier).** `validate_event` returns
   `ValidationOutcome::Rejected(ExchangeError)` — fully structured, `to_wire_code()` available
   (exchange.rs:130). `dispatch_event` flattens it at **runtime.rs:1086**:
   `return DispatchOutcome::Rejected(err.to_string())`. The variant is **`DispatchOutcome::Rejected(String)`**
   (runtime.rs:111) — it cannot carry a code; `err.to_wire_code()` is in scope at 1086 but discarded.
   **Primary drop.**

2. **`process_inbound` → wire (the hardcode).** The `Rejected(reason)` arm (app.rs:2698) calls
   `reject_signal` (app.rs:2386), which hardcodes **`error_code: 4000`** (app.rs:2395) because all it
   receives is the opaque `reason: String`. The function's own doc comment names it: *"a structured
   per-reason code taxonomy is a future refinement"* (app.rs:2382-2385).

### 2.2 What is already in place (so the fix is small)

- **The wire frame is ready.** `TransportMessage::Error` already carries `error_code: u32` +
  `error_string` + `event_id: Option<String>` (`xgen-core/src/wire/types.rs:79-92`). **No wire-shape
  change needed** — the field exists; only the value is dropped internally.
- **D-070 correlation half is done.** `reject_signal` already populates `event_id` (the load-bearing
  correlation primitive, D-070 / J-081 §5). MP-F2 is the deferred completion of D-070's
  transport-layer contract: the *specific reject code* was the named refinement.
- **`to_wire_code` covers only 5 of ~13 `ExchangeError` variants** (exchange.rs:130-142). Mapped:
  `AiCapabilityViolation`(3042), `AiRoleViolation`(3041), `NodeEjectAuthority`(3043),
  `SpaceMigrateAuthority`(6009), `TimestampOutOfBounds`(**3046**). Unmapped → `None` → generic:
  `SignatureFailure`, `UnknownSender`, `NotASpaceMember`, `NotARoomMember`, `PermissionDenied`,
  `EventIdMismatch`, `DagError`, `MissingEventId`, `HeldPending`. **Consequence:** MP-A-15 (timestamp)
  *has* a code to surface (3046); **MP-A-05 (signature) does not** — `SignatureFailure` is unmapped,
  so it stays generic-4000 **this arc** (→ MP-F2-followon, §3 D6).

### 2.3 The 15 construction sites (all in `xgen-core/src/node/runtime.rs`)

Every site already "knows" its code; the widening lets each one supply it. Categorised:

| Cat | Sites (line) | What the site has | This-arc action |
|---|---|---|---|
| **A** ExchangeError in scope | 1086 (`validate_event` reject — **primary**), 1130 (`check_ai_capability`), 1141 (`check_ai_operator_targets_pub`), 1144 (`check_permission_pub`) | `err: ExchangeError` → `to_wire_code()` | call `to_wire_code()`; `None` ⇒ generic 4000 |
| **B** already `(code,name)` | 1223 (PG-13 tier gate), 1319 (thread tier gate) | `(code,name) = e.to_wire_code().unwrap_or((3030,"tier_mismatch"))` already computed, **embedded in the string** | move the code into the field |
| **C** raw string, code in prose | 1120 (ai_role 3041), 1197 + 1205 (invite over-ceiling 3045), 1273 + 1280 (invite_expired 3044), 1307 (thread-below-room 3030) | code named in prose only | supply the literal `(code, name)` it already names |
| **D** internal / pre-validate | 899 ("event missing event_id"), 921 ("space not found"), 1031 ("store init failed") | no specific protocol code | generic 4000 (preserves observed behaviour — MP-A-17 saw 4000) |

Confirmed return types (Cat A grounding): `check_ai_capability` (exchange.rs:254),
`check_ai_operator_targets_pub` (:308), `check_permission_pub` (:317) all return
`Result<(), ExchangeError>`.

### 2.4 Consumer blast radius (the real count — corrects the Phase-0 "~10 tests" estimate)

`DispatchOutcome::Rejected` is **constructed only in `runtime.rs`** (the 15 sites above). It is
**consumed** in:

- **3 production consumers:**
  - `app.rs:2698` (`process_inbound` — the payoff) — destructures `Rejected(reason)`; passes the code
    to `reject_signal`.
  - `app.rs:2386` (`reject_signal`) — gains an `error_code` param; populates the field instead of `4000`.
  - `admin_ops.rs:4024` (`Rejected(why)`) — uses `why` in a `format!` error string.
- **~16 test reason-binders** that bind the payload: `phase9_validation_asymmetry.rs` ×15 (e.g.
  `let DispatchOutcome::Rejected(reason) = outcome else {…}; assert!(reason.contains(…))` at 279/297/
  316/362/383/575/593/612/643/661/687/705/724/755/775) + `federation_relationship_integration.rs:357`.
  `fanout.rs:1905` (`#[cfg(test)]`) also binds.
- **~15 `matches!(outcome, DispatchOutcome::Rejected(_))` wildcards** (phase9_validation_asymmetry,
  paired with each binder) + the drain-loop wildcards `HeldPending | Rejected(_) => {}`
  (runtime.rs:1554/1638/1722).

This count drives the variant-shape lock (§3 D1).

---

## 3. Locked decisions (proposed — for Joe-lock)

### MP-F2-D1 (F-1a) — widen the variant; the field is the single source of truth

`DispatchOutcome::Rejected` carries a small struct instead of a bare `String`:

```rust
/// Structured rejection metadata. The `code`/`name` are the protocol wire
/// (code, name) per ExchangeError::to_wire_code or the gate that produced the
/// rejection; `reason` is the human-readable detail. The CODE FIELD is
/// authoritative — consumers read it, never re-parse `reason` (D-067 no-drift).
pub struct RejectInfo {
    pub code: u32,
    pub name: &'static str,
    pub reason: String,
}
// ...
Rejected(RejectInfo),   // 1-tuple carrying the struct
```

**Variant shape = 1-tuple-carrying-a-struct `Rejected(RejectInfo)`, NOT a struct variant
`Rejected { code, name, reason }`.** Grounded reason (§2.4): the ~15 `matches!(_, Rejected(_))`
wildcards and the drain-loop `Rejected(_)` arms **survive unchanged** under a 1-tuple — `_` matches
any inner type. A struct variant would force every wildcard to `Rejected { .. }` (~15 extra
mechanical edits) for no benefit. The 1-tuple touches only the ~16 reason-binders (→ `info.reason`).

Constructors keep the 15 sites terse:

```rust
impl RejectInfo {
    pub fn generic(reason: impl Into<String>) -> Self;                  // (4000, "generic", reason) — Cat D
    pub fn coded(code: u32, name: &'static str, reason: impl Into<String>) -> Self; // Cat B/C
    pub fn from_exchange(err: &ExchangeError) -> Self;                  // Cat A: to_wire_code() or (4000,"generic")
}
```

`#[derive(Debug, Clone)]` on the enum is retained (RejectInfo derives the same); no `PartialEq/Eq`
(consistent with the existing comment at runtime.rs:96-101 — `DispatchOutcome` is matched, not
compared).

### MP-F2-D2 — each site supplies its code; reason strings are FROZEN (additive)

Per the §2.3 category map. Critically, **the `reason` string content is left byte-identical this
arc** — the code field is purely *additive*. Rationale (backward-coherence, D-077): ~16 test
`reason.contains("…")` asserts (e.g. `reason.contains("step 12: signature verification failed")`,
asymmetry.rs:280) stay green only if the string is unchanged. The new authoritative home for the code
is the field; the legacy `(3030)`/`(3045)` prose left inside Cat B/C reason strings is now *cosmetic
duplication*, **not** a drift surface (no consumer re-parses it — D-067 is about re-parsing, which
this design eliminates). Stripping that prose is a cosmetic cleanup → MP-F2-followon (§3 D6c).

### MP-F2-D3 (F-2) — propagate only already-computed codes this arc

`from_exchange` (Cat A) returns the mapped code where `to_wire_code()` is `Some`, else generic 4000.
This **closes MP-A-15** (`TimestampOutOfBounds` → 3046 now on the wire). It does **NOT** invent codes
for the 7 unmapped event-validation variants — `SignatureFailure`, `PermissionDenied`,
`NotASpaceMember`, etc. continue to emit generic 4000 this arc. **MP-A-05 (signature) therefore stays
4000 until MP-F2-followon** — stated plainly, not glossed.

### MP-F2-D4 — `reject_signal` reads the code; `process_inbound` is the payoff

`reject_signal(origin, event_id, error_code: u32, reason, timestamp)` populates
`TransportMessage::Error { error_code, … }` from the param (deletes the hardcoded `4000`,
app.rs:2395). `process_inbound`'s `Rejected(info)` arm passes `info.code` + `&info.reason`
(app.rs:2698-2736). `accept_signal` and the `LocallySubmitted`-only gate are untouched.

### MP-F2-D5 (F-3) — origin gate unchanged

`reject_signal` keeps the `matches!(origin, EventOrigin::LocallySubmitted) && event_id != "(none)"`
gate (app.rs:2392). Federation peers still receive no `Error` on reject. No change.

### MP-F2-D6 — MP-F2-followon (named, NOT absorbed)

A separate arc owns:
- **(a)** Assign protocol wire codes to the 7 unmapped event-validation `ExchangeError` variants
  (`SignatureFailure`, `UnknownSender`, `NotASpaceMember`, `NotARoomMember`, `PermissionDenied`,
  `EventIdMismatch`, `DagError`) — **closes MP-A-05's residual 4000**. Note the spec subtlety: ch3
  §3.6.5's `3001/3002 signature_invalid` are *registration*-scoped; the followon needs a
  code-assignment decision against the **event-validation** error table (ch3 §3.9), collision-checked
  (the 6009/3046 lesson).
- **(b)** Reconcile the **3030-vs-3010 tier-code drift** caught at Phase-0: the code emits
  `3030 tier_mismatch` (runtime.rs:1222/1318) while spec §3.11.7 lists `3010 auth_tier_insufficient`
  and reserves 3000–3099 for identity. MP-F2 propagates whatever the gate computes (3030); deciding
  3030→3010 (or that 3030 is correct) is a spec-reconciliation, not this arc.
- **(c)** *Optional* cosmetic de-dup: strip the `(code)` prose from Cat B/C reason strings now that
  the field is authoritative (updates the affected `reason.contains` asserts).

---

## 4. Change surface

| File | Sites | Change |
|---|---|---|
| `xgen-core/src/node/runtime.rs` | enum (103-112) + 15 construction sites (§2.3) | add `RejectInfo` + constructors; each `Rejected(…)` supplies `RejectInfo` |
| `xgen-core/src/node/runtime.rs` | 3 drain wildcards (1554/1638/1722) | **unchanged** (`Rejected(_)` matches the 1-tuple) |
| `xgen-node/src/app.rs` | `reject_signal` (2386) + `process_inbound` arm (2698) | param + field populate; pass `info.code`/`info.reason` |
| `xgen-node/src/admin_ops.rs` | 4024 | `Rejected(why)` → `{}` over `why.reason` |
| `xgen-core/src/node/tests/phase9_validation_asymmetry.rs` | ~16 reason-binders | `Rejected(reason)` → `Rejected(info)`, `info.reason`; ~15 `matches!(…Rejected(_))` **unchanged** |
| `xgen-node/src/tests/federation_relationship_integration.rs` | 357 | `.reason` access |
| `xgen-node/src/fanout.rs` | 1905 (`#[cfg(test)]`) | `.reason` access |

Untouched: `wire/types.rs` (frame already adequate), `validate_event`/`ExchangeError` (Cat A reads
its existing `to_wire_code`), admission logic, resolution/ordering, `apply_fanout`, federation push,
the migration/handshake error paths (`e.error_code()` at app.rs:2988 is a different error type — out
of scope).

---

## 5. Proof plan

- **Unit (xgen-core, `dispatch_event`):** for each gate that has a code, assert
  `DispatchOutcome::Rejected(info)` carries the expected `info.code` — timestamp → 3046, tier → 3030,
  over-ceiling → 3045, invite-expired → 3044, ai-capability → 3042, ai-role → 3041. For an unmapped
  variant (signature), assert `info.code == 4000` (pins MP-F2-D3's honest boundary so the followon
  can flip it).
- **Wire / smoke (xgen-mptest):** the MP-A-15 repro (`mp_r1_c7::mp_a_15_*`) now asserts the delivered
  `Error.error_code == 3046` (was 4000) — the finding's close condition. MP-A-05 asserts it is **still
  4000** (documents the followon boundary, not a regression).
- **Backward-coherence (D-077):** the full suite stays green with reason strings frozen — the ~16
  `reason.contains(…)` asserts are unchanged by MP-F2-D2. Any pre-existing test asserting `4000` on a
  *now-coded* path is a finding to surface (expected: only the MP-A-15 smoke, intentionally updated).
- **Non-interference (§6):** a grep-confirmed assertion that no accepted event's ordering/state
  changes — the Rejected path returns `FanoutRequest::none()` and never enters the DAG/store.

Suite baseline 72/0 (fast) holds; the count grows only by the new unit asserts.

---

## 6. Safety — D-076 discharge (reject code is admission OUTPUT, never input)

The widening cannot move any admission or ordering surface, by construction:

- **Admission output, not input.** `RejectInfo` is derived from a rejection the gate has **already
  decided**. The 3044/3045/3046/3030 gates compute their verdict (and code) *before* the field
  exists; the field merely carries what was decided. A rejected event never enters the DAG/store —
  `process_inbound`'s Rejected arm returns `FanoutRequest::none()` (app.rs:2737), so there is no
  fan-out, no persist, no state mutation.
- **Never read by ordering/resolution.** `state_key_for_event`, `derive_resolved`, the resolver, and
  the wire-order/timestamp logic (D-076) operate only on **accepted** events. They never inspect a
  `DispatchOutcome::Rejected` payload. The new `code`/`name` field is wire-and-log observability only.
- **Clock-neutral.** Unlike the 3044/3046 *gates* (which read the injected clock, D-090), the
  `RejectInfo` carries a value those gates already produced — it introduces no new clock or
  convergence dependency.

Conclusion: changing what a *rejected* event reports cannot change whether/how an *accepted* event is
admitted or ordered. The D-076 caution is discharged — no ordering surface is in the blast radius.

---

## 7. Scope fence + honest boundary

**In:** the two internal carriers (the `DispatchOutcome::Rejected` shape + `reject_signal`'s
hardcode) and the mechanical supply of each gate's already-known code. **Out:** the `Error` wire
struct (adequate), admission/ordering, federation push, and — explicitly — assigning codes to the 7
unmapped variants + the 3030-vs-3010 reconciliation (MP-F2-followon, D6).

**Honest boundary (D-065):** MP-F2 closes MP-A-15 (3046 on the wire) and makes *every gate that
already computes a code* surface it. It does **not** make every rejection specifically coded —
signature/membership/permission rejections still emit generic 4000 until MP-F2-followon. The
deliverable is "the computed code now reaches the wire," not "a complete reject-code taxonomy."

No DECISIONS change at this design (MP-F2-D# arc-local, D-069). The arc is a production change — full
discipline: this design → Joe-lock → runbook → implement → close.

---

## 8. Entry point (Rule 0)

This design → `tasks/MP_findings.md` (MP-F2) → `tasks/MP_R1_DETERMINISTIC_DESIGN.md` §11/§12 (the
fix-phase strategy + greenlit forks) → the grounded sites in §2.3/§2.4. **Next: Joe-lock**, then the
runbook (`tasks/MP_F2_REJECT_WIRE_CODE_IMPL.md`), then implement, then close. Then MP-F3, then MP-F1
(needs a facet-2 grounding pass before it is designable). Not pushed — Joe pushes.
