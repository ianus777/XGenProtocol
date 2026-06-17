# M9 — Findings & Injector Rejection-Point Record
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this file is

The durable record of what M9 (strategic multiparty test harness, C1–C5, closed J-307) surfaced
about the **system under test** — kept out of the harness commits per D-065/D-084 (M9 builds the
harness and *surfaces* defects; it does **not** patch the binaries). Each finding is routed to a
fix-arc or to the Multiparty-tests milestone. ACTIVE until each is consumed.

---

## 2. Open findings (routed — NOT patched under M9)

**F1 — No timestamp-bound validation (gap G6). ✅ RESOLVED by M9.1 (2026-06-07).** `validate_event`
(the live F-4 unified core, `xgen-core/src/message/exchange.rs` — NOT the deprecated
`validate_steps_8_13` the audit cited) had no timestamp-bound check, so a skewed-but-otherwise-valid
event (MP-A-15 ClockSkew, stamped 2099) was **silently accepted**. **M9.1 added a "Step 8.5"
future-skew admission bound** (`MAX_FUTURE_SKEW_SECS = 300`; new `ExchangeError::TimestampOutOfBounds`
→ wire **3046**; threaded `now` from `self.clock.now_utc()`): an event whose timestamp is more than
5 min ahead of the receiver's `now`, or is unparseable, is rejected at admission. Admission-only —
the timestamp never feeds `state_key_for_event`/the resolver/ordering (D-076). Runs on **both
origins** (origin-blind), convergence-safe by catch-up monotonicity (a future ceiling only grows
headroom as `now` advances). **No past floor** — far-past is legitimate catch-up/replay; coherently
backdated events remain out of scope (the deferred causal-monotonicity territory, M9.1-D1). MP-A-15
now rejects (3046) + absent. See `tasks/M9_1_TIMESTAMP_VALIDATION_{AUDIT,DESIGN,IMPL}.md`.

**F2 — No fresh-peer federation-initiate surface.** Two fresh node binaries cannot be federated
through the external control surfaces: federation initiate is known-peers-only (`FED_3006`), with
no config peer-list and no `--aicontrol` initiate verb. This gates the true cross-node cooperative
scenarios (MP-C-02 two-node, MP-C-03, MP-C-04, MP-C-14). **Route: Multiparty-tests prerequisite**
— likely a small initiate verb (Joe-lock when that milestone opens). Round-0 ran MP-C-02
single-node (real convergence) to prove the machinery without it.

**F3 — MockClock inoperable across the process boundary.** `ClockMode::Mock` is not usable for
the real-binary harness: it needs a non-default `mock-clock` build **and** there is no
clock-advance control surface across the process boundary, so Round-0 ran real-clock. The M8.6
`Clock` seam is therefore **not** operably reused by M9 (see the M9-D5 promotion note). **Route:
Multiparty-tests R1-determinism prerequisite** — needs a clock-advance control surface (and/or a
mock-clock build) for the deterministic round and for MP-A-01 (expired-invite replay).

**F4 — Malformed-frame injection needs raw socket access.** The injector cannot send a truncated/
garbage frame because `Connection`'s `send_bytes`/`encode_frame` are private; only well-formed
`Event`s can be sent. MP-A-12 is therefore code-grounded, not live. **Route: a `pub` raw-send
seam on `Connection` (a binary change → Joe-lock) for the member-context Multiparty-tests runs**,
or a hand-rolled `connect_async` in the harness.

---

## 3. Resolved this close

**R1 — Validation-boundary citation corrected.** The design (§2 / M9-D6) and runbook cited
`ingest_event` (runtime.rs:481) as the rejection boundary. That is the **no-validation**
direct-insert ("caller is responsible"; only guard `None => return` for unsigned). The real
inbound validation boundary is **`validate_event`** (the F-4 13-step, exchange.rs) inside
`dispatch_event`, called by `process_inbound`. Corrected in the design, the runbook, the matrix
(MP-A-05), and Clair's injector module docs. **Closed.**

**Reject-path Error inconsistency (refines J-081) — not a defect, recorded.** Some reject paths
emit `TransportMessage::Error` (e.g. space-not-found → wire `4000`, observed live); others
log-and-drop. The convergence oracle therefore keys on **absence** (event never applied), not on
an Error reply — which is the correct, robust rejection signal. No action.

---

## 4. C4 injector rejection-point table (Checkpoint #3 record, J-307-locked)

| Attack | Rejection point | C4 status |
|--------|-----------------|-----------|
| ForgedSignature | `validate_event` **step 12** (signature). Live at MP-A-05: `Error(4000,"step 12: signature verification failed")`; absent from `.events`. | ✅ live-confirmed (C5 member-context) |
| Malformed | Transport frame-parse (cannot deserialize into `Event`; never reaches `validate_event`). | code-grounded (needs F4 raw-send) |
| DuplicateId | DAG dedup (`graph.add_event`), **after** validation; needs a valid member-context base event. | code-grounded → Multiparty-tests |
| ClockSkew | **✅ F1 / gap G6 — RESOLVED by M9.1** — `validate_event` Step 8.5 future-skew bound (wire 3046) rejects the 2099-dated event at admission; absent thereafter. | resolved (M9.1) |
| Equivocation | **Not a rejection** — two valid conflicting events both apply; M8 resolution converges on one winner (no fork). MP-A-06 = convergence-on-winner. | code-grounded (not absence) |
| ForgedInvite | `validate_event` membership / missing-predecessor, or HeldPending→timeout. | code-grounded → Multiparty-tests |

---

## 5. Micro-benchmark (C3 — box-ceiling data point)

Box: 32 GB / 20-core (Intel Ultra 7 265KF). Measured per spawned `xgen-node`: **~18.4 MB mean
RSS, 3 threads each.** Estimated process ceiling **~1,562** — consistent with the audit §6.1
estimate (~800–1,200 comfortable, stretch ~1,500). The Multiparty-tests R2/R3 numbers are fixed
against this on a freed-up box (Round-0 ran with ~5 GB free).

Per D-065 + D-069 + D-084.
