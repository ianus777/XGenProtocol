# MP-F2 — Reject-path wire-code propagation — IMPLEMENTATION RUNBOOK
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-08  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Scope

Executes the Joe-locked design `tasks/MP_F2_REJECT_WIRE_CODE_DESIGN.md` (MP-F2-D1..D6). Production
arc — first real protocol change of the multiparty effort. Full discipline.

**Single atomic commit.** The enum widening `DispatchOutcome::Rejected(String)` →
`Rejected(RejectInfo)` breaks every construction site and consumer in the same compile (the
rename-breaks-all pattern, like the XGID Path-A retypes). It cannot be split — there is no
intermediate state where the workspace compiles. One commit: the widening + all construction sites +
all consumers + the MP-A-15 wire assertion. Then a doc-only close.

---

## 2. The proof obligation (Joe lock-condition)

**The arc is falsifiable on exactly one assertion:** after the fix, MP-A-15's `Error` frame carries
`error_code == 3046` (was 4000). Concretely:

- `xgen-mptest/tests/mp_r1_c7.rs::mp_a_15_clock_skew_rejected` (line ~226-238): the smoke extracts
  `(code, msg)` from `rec.error_reply`; **add `assert_eq!(code, 3046, …)`** (currently it asserts only
  the message substring + an eprintln noting "3046 NOT on the wire"). Update the doc-comment (206-217)
  + the eprintln to state the code is now on the wire.
- **Boundary on the record:** `mp_a_05_forged_signature_rejected` (line 131) keeps
  `assert_eq!(code, 4000, …)` — signature (`SignatureFailure`) is unmapped, stays generic-4000 until
  MP-F2-followon. The test suite itself encodes the MP-F2-D3 boundary; do not change it.

Payoff verification: re-run the MP-A-15 smoke (`--ignored`, harness-control) and confirm `code=3046`
on the wire.

---

## 3. Grounded edit map (against live `main`)

### 3.1 `xgen-core/src/node/runtime.rs` — the type + the 15 construction sites

1. **Add `RejectInfo`** (near the `DispatchOutcome` enum, ~line 102) with `#[derive(Debug, Clone)]`,
   fields `pub code: u32`, `pub name: &'static str`, `pub reason: String`, and constructors
   `generic(impl Into<String>)` → `(4000, "generic", …)`, `coded(code, name, reason)`,
   `from_exchange(&ExchangeError)` → `to_wire_code()` or `(4000, "generic")`.
2. **Change** `Rejected(String)` → `Rejected(RejectInfo)` (line 111).
3. **15 construction sites**, per the design §2.3 category map:
   - **Cat A (ExchangeError → `from_exchange`):** 1086 (`RejectInfo::from_exchange(&err)`; reason stays
     `err.to_string()` — D2 frozen). 1130/1141/1144 (`from_exchange(&e)`).
   - **Cat B (already `(code,name)`):** 1223, 1319 — `RejectInfo::coded(code, name, format!(…))`; the
     reason string is **left byte-identical** (still includes the `({code})` prose — cosmetic, D2).
   - **Cat C (literal code in prose):** 1120 `coded(3041,"ai_role_violation", …)`; 1197+1205
     `coded(3045,"invite_validity_exceeds_max", …)`; 1273+1280 `coded(3044,"invite_expired", …)`;
     1307 `coded(3030,"thread_auth_tier_below_room", …)`. Reason strings byte-identical.
   - **Cat D (generic):** 899, 921, 1031 — `RejectInfo::generic(…)`.
4. **3 drain wildcards UNCHANGED:** 1554, 1638, 1722 (`HeldPending | Rejected(_) => {}` — `_` matches
   the 1-tuple).

### 3.2 `xgen-node` production consumers (3)

- `app.rs:2698` (`process_inbound`, the payoff) — `Rejected(reason)` → `Rejected(info)`; pass
  `info.code` + `&info.reason` to `reject_signal`.
- `app.rs:2386` (`reject_signal`) — add `error_code: u32` param; set `error_code: error_code`
  (delete the hardcoded `4000`, line 2395). Update the doc-comment (2380-2385) — the GENERIC_4000
  rationale is superseded.
- `admin_ops.rs:4024` — `Rejected(why)` body uses `{why}` in a `format!`; change to `why.reason`.

### 3.3 Test consumers (~33 reason-binders) — shadow-rebind, asserts FROZEN

Implementation rule for D-077: **keep every asserted substring byte-identical.** Use a local
shadow-rebind so bodies are unchanged:

- `let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };` → bind `RejectInfo`, then
  `let reason = reason.reason;` (shadows to `String`; body unchanged). `phase9_validation_asymmetry.rs`
  has this line **identical ×15** (279/297/316/362/383/575/593/612/643/661/687/705/724/755/775) →
  one `replace_all`. `phase9_compound_c5_validation_under_load.rs:373` is the same line (1 more).
- `if let DispatchOutcome::Rejected(reason) = &outcome {` → insert `let reason = &reason.reason;`
  after `{`. Sites: runtime.rs 2131/2162/2240/2268; `federation_relationship_integration.rs:357`;
  `phase9_federation_relationship_rejection.rs:440/464/487` (var names `&outcome_fed_add` /
  `&outcome_space_create` / `&outcome_dm`).
- match arms `DispatchOutcome::Rejected(reason|r) => assert!(…)` → wrap:
  `=> { let reason = reason.reason; assert!(…); }`. Sites: runtime.rs 2393/3634/3718/3802/3837/3934/
  3952/4138/4400(`r`)/4501(`r`); `fanout.rs:1905` (`#[cfg(test)]`).
- `matches!(outcome, DispatchOutcome::Rejected(_))` wildcards — **UNCHANGED** (~21 sites incl.
  asymmetry ×15, c5:370, `m8_s7_privilege.rs:194`, `phase_arcf_migration_e2e.rs:183`).

**No Deref on `RejectInfo`** — the codebase rejects `Deref<Target = str>` shortcuts (XGID norm).
Consumers read `.reason`/`.code` explicitly; the shadow-rebind is local ergonomics, not masquerade.

### 3.4 The MP-A-15 payoff (§2)

`xgen-mptest/tests/mp_r1_c7.rs` — add `assert_eq!(code, 3046, …)`; refresh the doc-comment + eprintln.

---

## 4. Definition of Done

- [ ] `cargo build --workspace --all-targets` → 0 (default).
- [ ] `cargo build --workspace --all-targets --features harness-control` → 0.
- [ ] `cargo clippy --workspace --lib --tests -- -D warnings` clean — **default AND
      `--features harness-control`**.
- [ ] Fast suite green (`cargo test --workspace`, the `#[ignore]` smokes excluded) — count grows only
      by the new `dispatch_event` code-assertion unit tests; the ~33 `reason`-binder tests stay green
      (the D-077 regression net — asserted substrings unchanged).
- [ ] New unit asserts in `runtime.rs` tests: each coded gate's `DispatchOutcome::Rejected(info)`
      carries the expected `info.code` (3046 timestamp, 3030 tier, 3045 over-ceiling, 3044
      invite-expired, 3042 ai-cap, 3041 ai-role); one unmapped (signature) asserts `info.code == 4000`
      (pins the MP-F2-D3 boundary).
- [ ] **Payoff:** MP-A-15 smoke (`--ignored`, harness-control, `--test-threads=1`) shows
      `error_code = 3046` on the wire. MP-A-05 still shows 4000 (boundary).
- [ ] D-076 non-interference: grep-confirm no accepted-event ordering/state path reads a
      `Rejected` payload (the Rejected arm returns `FanoutRequest::none()`); no admission move.
- [ ] No "commit pushed" line — Joe pushes.

## 5. Close (doc-only, after the code lands)

Task-doc closes (arc-local, mine): design + this runbook → COMPLETED; `tasks/MP_findings.md` MP-F2 →
resolved (3046 now on the wire; MP-A-05/unmapped → MP-F2-followon); matrix MP-A-15 row rewritten
(drop the "3046 NOT on the wire" / stale-F1 note). **Canonical-record flips (CLAUDE PLAY / JOURNAL /
ROADMAP) are the one-writer bridges — surfaced to Joe for the close atomic, not written here.** No
DECISIONS change (MP-F2-D# arc-local, D-069).

Next: MP-F3, then MP-F1 (flag the facet-2 grounding pass).
