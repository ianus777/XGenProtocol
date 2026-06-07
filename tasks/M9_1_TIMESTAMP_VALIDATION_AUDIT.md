# M9.1 — Event Timestamp-Bound Validation (F1 / gap G6) — Phase-0 Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose

The D-071 Phase-0 audit for **M9.1**, a sub-branch of M9 (✅ CLOSED J-307) that fixes finding
**F1 / gap G6** from `tasks/M9_findings.md`: `validate_event` has **no timestamp-bound check**, so
an event with an arbitrary timestamp is silently accepted (surfaced live by the M9 injector at
MP-A-15). M9.1 is a **protocol fix** to shipping validation logic — distinct from its sibling
**M9.2** (harness-enablement seams, F2/F3/F4, test tooling). Doc-only; no code; no DECISIONS
change expected (M9.1-D# arc-local per D-069). Feeds the M9.1 design → runbook → Clair.

---

## 2. Gates (D-071)

- **M9 CLOSED (J-307)** — the harness surfaced F1 as a real defect (Checkpoint #3 "silently
  accepted" case); routed here, NOT patched under M9 (D-065/D-084).
- Suite **1262/0/8**; build 0; clippy clean (default + `--all-features`).

---

## 3. Grounding

### 3.1 The validation boundary (M9.1-A1)
`validate_event` / `validate_steps_8_13` (xgen-node/src/message/exchange.rs, the F-4 13-step
inbound pipeline, in `dispatch_event`/`process_inbound`) checks: event_id integrity (step 8),
causal predecessors / prev_events (step 9, unknown → HeldPending), sender registration + Space/
Room membership (step 11), signature (step 12). **No step inspects the event timestamp** —
confirmed across steps 8–13. That is the gap.

### 3.2 D-076 non-interference (M9.1-A2)
D-076 = **wire-order determinism**: ordering/resolution is by wire-order + the DAG, **not** by
timestamp. So a timestamp bound must be **admission-only** — it may reject an event at the gate,
but it must **never** feed `state_key_for_event`, the resolver, or any ordering decision. Adding
a bound is wire-format-neutral as long as accepted events flow through the existing path
unchanged.

### 3.3 Clock + expiry precedent (M9.1-A3)
The codebase already has a clock and expiry-window patterns to reuse: `self.clock.now_utc()`
(D-090); the invite admission gates 3044/3045 on `valid_until` (INV / INV-EXP); the Trust
Assertion `valid_until` (AE-D1). M9.1 is the same shape applied to the event's own timestamp.

### 3.4 The INV-EXP lesson (M9.1-A4 — the crux)
INV-EXP (J-298) established that admission gates run **iff `origin == LocallySubmitted`** and
**skip on `ReceivedViaFederation`** — because a federated event already passed its origin node's
gate, and re-checking on the drain path (against a *different* node's clock/state) breaks
convergence and federation catch-up. **This is the central constraint on M9.1:** a timestamp
bound evaluated against a local clock is **clock-dependent**, so if it ran on every node it could
make node A accept and node B reject the *same* event → divergence. The INV-EXP/F-5/D-089
pairwise-trust shape is the precedent for resolving this.

### 3.5 The injector nuance (M9.1-A5 — names the real question)
The M9 injector (MP-A-15) connects as a **peer/client** and sends a skewed event — i.e. it
arrives as `ReceivedViaFederation`-class, not `LocallySubmitted`. So an INV-EXP-style
**locally-submitted-only** gate would close the *local* hole (a lying local client / skewed local
clock) but would **not** reject the injector's federated skewed event. Catching the injector
means bounding federation-received timestamps too — which reopens the convergence risk of 3.4.
**This tension is the design phase's core decision, not pre-resolved here.**

---

## 4. Forks for the design phase

| Fork | Question | Lean (to confirm in design) |
|------|----------|------------------------------|
| **M9.1-F-A** | Which bound? | **Future-skew ceiling** (reject timestamps too far *ahead* of local now). Far-**past** is legitimate — old events arrive via federation catch-up + replay-from-disk in an append-only log; rejecting them would break both. (A monotonicity-vs-`prev_events` rule is an alternative/addition — clock-free, convergence-safe — worth weighing.) |
| **M9.1-F-B** | Origin-gating (the crux) | The INV-EXP shape (gate `LocallySubmitted` only) is convergence-safe but misses the federated injector (3.5). Options: (1) **local-only** gate + treat a skewed *federated* event as the forwarding peer's trust responsibility (F-5/D-089 — consistent with M9's "compromised/malicious peer = out of scope" ledger); (2) **both origins** with a bound chosen wide enough that honest clock-skew never flips the decision (convergence-safe by margin); (3) a **clock-free monotonicity** rule that needs no origin-gating. **The Joe-lock.** |
| **M9.1-F-C** | Convergence safety | Whatever bound is chosen MUST yield the **same accept/reject on every honest node** (or be origin-gated so only one node decides). A divergence-repro test is mandatory (two nodes, skewed clocks, same event → same verdict). |
| **M9.1-F-D** | Where + how | A new admission check in `validate_steps_8_13`, reusing `self.clock` (D-090). **Must not** touch ordering (M9.1-A2 / D-076). |
| **M9.1-F-E** | The window value | The skew tolerance is a **named parameter**, not a magic constant; default chosen against realistic federation/clock-skew (not tuned to pass a test — parameters are the spec). |

---

## 5. Scope boundary

- M9.1 fixes **only** the timestamp-bound admission gap. It does **not** touch resolution/
  ordering (D-076), the invite-expiry gates (already INV-EXP), or any sibling finding.
- **Honest boundary (D-065):** if the locked design is local-origin-only (M9.1-F-B option 1),
  the audit must state plainly that a *federated* skewed event from a malicious trusted peer
  remains admitted by design (peer-trust, F-5/D-089) — closing the local hole, not the
  trusted-peer one. That is a legitimate scope line, but it must be **named**, because the M9
  injector exercised exactly that path.

---

## 6. Next-active

**M9.1 design phase** — lock M9.1-F-A…F-E (F-B is the crux); the convergence-safety repro
(M9.1-F-C) gates the design. Then runbook → Clair → close.

**Entry point (Rule 0):** CLAUDE PLAY → JOURNAL J-308 → this audit §3 + §4 →
`tasks/M9_findings.md` (F1).

Per D-065 + D-069 + D-071 + D-074 + D-078 + D-090.
