# Federation Event Propagation — Design Phase Task

> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-18  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this task is

This task file governs the **design phase** of the Federation Event Propagation completion milestone. The milestone was opened 2026-05-18 at the close of the Propagation Reliability Audit (J-081) and currently sits PENDING in CLAUDE.md. It cannot go ACTIVE until its own Joe-locked design phase produces a canonical design document, per the D-069 discipline rule.

This is **design work, not implementation**. No code changes. No tests added. The deliverable is a canonical design document that Clair (Code Claude) will use as the basis for the implementation runbook later.

This task closes when the canonical design document is shipped and Joe-locked, at which point a separate runbook task file (likely `tasks/FEDERATION_PROPAGATION_COMPLETION.md`) is written for Clair, and CLAUDE.md flips the milestone block to ACTIVE.

---

## 2. Why this milestone exists

The Propagation Reliability Audit (`docs/xgen_propagation_reliability.md`) found that Node-to-Node federation event propagation does not exist as a production mechanism. The federation surface today is one-time history dump on peer-initiated handshake, then connection close. No persistent peer session, no outbound event push, no DAG-tip reconciliation, no gap-recovery mechanism.

Two coordinated audit findings close in this milestone:

1. **§2 Stage 6 — Node-to-Node federation propagation architecturally absent** (HIGH).
2. **§3 sub-finding — `process_inbound` validation asymmetry: Paths B and C skip signature + timestamp verification** (LOW today, HIGH on federation landing — a **precondition**, not parallel work).

The §4 sub-findings (500ms quiet-time fallback in sync-on-reconnect, no pagination on `collect_sync_history`, Ch4 lines 779/825-827 describing absent mechanisms) are related concerns; the design phase decides whether to fold them in.

---

## 3. Three-pass discipline

Mirrors the M6 Phase 0 discipline locked in D-069.

| Pass | Owner | Deliverable | Status |
|---|---|---|---|
| Pass 1 — audit current state | done (J-081) | `docs/xgen_propagation_reliability.md` | ✅ |
| Pass 2 — proposals + Joe-lock markers | Chat Claude + Joe | Working draft of design doc with `[JOE-LOCK]` markers on every framework decision | next |
| Pass 3 — lock framework decisions | Chat Claude + Joe | Canonical design doc, all decisions locked, Pass 2 deprecated | after Pass 2 |
| Runbook | Chat Claude | `tasks/FEDERATION_PROPAGATION_COMPLETION.md` (separate task file) | after Pass 3 |
| ACTIVE flip | Chat Claude | Update CLAUDE.md, ROADMAP.md | when runbook done |
| Implementation | Clair | Code + tests | not part of this task |

---

## 4. Pass 2 — proposed scope

Pass 2 produces a working draft of the canonical design document at `docs/xgen_federation_propagation_design.md`. The draft surfaces design alternatives with trade-offs, marks every framework decision with `[JOE-LOCK]`, and stops short of deciding for Joe.

### 4.1 Sections the design doc should cover

1. **What this document is** — header section per project convention; canonical-document rule per D-069.
2. **Background** — what the audit found; why this milestone exists; what the §2/§3 findings mean concretely.
3. **Scope and non-scope** — what this milestone closes (§2 Stage 6 absence, §3 validation asymmetry as precondition, design doc §4.2 + Ch4 lines 779/825-827 corrections) and what it explicitly defers (M6 admin verbs, the M6 Phase 2 envelope `event_id` work).
4. **Federation push direction** — three options on the table: push-from-home / pull-from-peer / hybrid. Trade-offs for each. `[JOE-LOCK]`.
5. **Session model** — persistent peer sessions vs. periodic reconciliation vs. ephemeral-per-batch. Trade-offs. `[JOE-LOCK]`.
6. **Wire-protocol shape** — what new `TransportMessage` / `FederationMessage` variants or fields are needed; existing handshake reuse vs. new mechanism; identity authority on the federation channel (Node keypair vs. delegated authority). Sketches only; final shapes locked in Pass 3.
7. **DAG reconciliation** — tip exchange, gap recovery, ordering guarantees on receipt, predecessor-unknown handling on the receiving side.
8. **Validation asymmetry closure** — how `process_inbound` Paths B and C gain signature and timestamp verification; whether this is the same code path as Path A or stays separate; this is the **precondition section** and must be addressed in the same milestone.
9. **§4 sub-findings disposition** — for each (500ms fallback, no pagination, Ch4 §implementation description) — fold in, defer, or address documentationally only. `[JOE-LOCK]`.
10. **Documentation correction scope** — which existing doc lines get corrected, when (in this milestone or in the implementation runbook), and how (reserved-section note now vs. full rewrite at Pass 3).
11. **Open questions** — anything that surfaces during Pass 2 that needs more than a `[JOE-LOCK]` flag.

### 4.2 What Pass 2 is NOT

- Not verb-level wire-shape proposals (that's runbook territory).
- Not Rust type design (Clair's latitude, per the M6 precedent — wire shape Joe-locked, internal realisation delegated).
- Not a re-litigation of the audit findings (J-081 is closed; the audit is treated as fact).
- Not test plan design (that's runbook territory).
- Not transitive federation scope (the audit flagged this as MEDIUM-deferred; the design phase may sketch it as future work but does not lock).

### 4.3 Joe-lock items expected to surface in Pass 2

The audit and the structure of the problem suggest the following framework decisions need Joe-locking. The actual list emerges during Pass 2 — these are the candidates known going in:

| # | Decision | Audit reference |
|---|---|---|
| F-1 | Push direction (home-pushes / peer-pulls / hybrid) | §2.5 Q1, Q2 |
| F-2 | Session model (persistent / periodic / ephemeral) | §2.5 Q1, Q4 |
| F-3 | Identity authority on the federation channel (Node keypair / per-Space delegation / something else) | §2.3, §4.5 |
| F-4 | Validation asymmetry — same code path as Path A or separate | §3.3, §3.6 |
| F-5 | Transitive federation — locked-out / locked-in / opt-in (initial spec) | §3.5 |
| F-6 | 500ms quiet-time fallback (§4) — fold in or defer | §4.7 |
| F-7 | No-pagination on `collect_sync_history` (§4) — fold in or defer | §4.7 |
| F-8 | Ch4 lines 779/825-827 correction — now or at Pass 3 | §4.6, §6.2 |
| F-9 | `docs/xgen_node_admin_ops_design.md` §4.2 correction — now or at Pass 3 | §2.6, §6.2 |
| F-10 | What "DAG hole" means when validation fails on a federated event (Scenario A non-message in §3.3) | §3.3 |

Numbering is provisional; Pass 2 may merge, split, or add items.

---

## 5. Operating constraints

### 5.1 The audit is fact

J-081 closed under per-section Joe-approval gate. Its findings are not re-litigated in this design phase. If Pass 2 surfaces a question that contradicts an audit finding, the design phase pauses; the resolution goes back to Joe; the audit is corrected only with explicit Joe authority.

### 5.2 No code changes during the design phase

Pure design work. No `cargo` commands. No file changes to `xgen-*/src/`. The design document is the only deliverable.

### 5.3 Each Pass-2 framework decision arrives separately

Pass 2 does not dump all `[JOE-LOCK]` items at once. Each framework decision is surfaced with its trade-off analysis, Joe responds, the decision is recorded (still `[JOE-LOCK]` in the draft — Pass 3 promotes them all together), Pass 2 moves to the next item. This mirrors the audit's per-section Joe-approval gate and prevents the "wall of decisions" failure mode.

### 5.4 The validation asymmetry is a precondition, not a sub-task

Closing federation push without closing the validation asymmetry would land a vulnerability (per audit §3.6). The design phase treats both as coordinated work in one document. If at any point Pass 2 considers splitting them, that requires explicit Joe-lock.

### 5.5 Header discipline

The design document follows the project header convention. `Status: PENDING` while Pass 2 is in progress; `Status: ACTIVE` at Pass 3 close. Every edit bumps `Last updated`. Two trailing spaces before EOL on every `> ...` line.

---

## 6. Definition of Done

This task completes when ALL of the following are true:

- [ ] Pass 2 working draft of `docs/xgen_federation_propagation_design.md` exists with all framework decisions surfaced and `[JOE-LOCK]` markers in place.
- [ ] Pass 3 has run and every `[JOE-LOCK]` marker has been locked or explicitly deferred to a separate milestone.
- [ ] Pass 3 design doc has `Status: ACTIVE`.
- [ ] D-070 promotion to DECISIONS.md is complete (separate work, but coordinated; the promotion text uses this milestone's framing) — this item may be checked off separately if Joe chooses to sequence D-070 differently.
- [ ] `tasks/FEDERATION_PROPAGATION_COMPLETION.md` (the implementation runbook for Clair) exists.
- [ ] CLAUDE.md milestone block flipped from PENDING to ACTIVE.
- [ ] ROADMAP.md updated in the same commit as the CLAUDE.md flip (per the project's mandatory-update discipline).

**The work is done when ROADMAP.md reflects the milestone as ACTIVE and Clair has a runbook to execute.** Not before. The `Status: COMPLETED` header in this task file is the real ship signal.

---

## 7. Cross-references

- **Audit (input):** `docs/xgen_propagation_reliability.md` — the canonical record of what's missing. Especially §2, §3, §6.4.
- **Documents needing correction:** `docs/xgen_node_admin_ops_design.md` §4.2; `docs/xgen_ch4_implementation.md` lines 779, 825-827.
- **Discipline rule:** D-069 (Joe-locked design phase before ACTIVE; canonical-document rule).
- **Related principles:** D-065 (honest behaviour over polite behaviour); D-070 (proposed: two events of equal importance, opposite direction); project principle "honest longer work over fast shortcuts" (locked 2026-05-18 during audit close).
- **Downstream:** M6 (new) Node admin write path — blocked behind this milestone's ACTIVE flip per CLAUDE.md.

---

*End of task file.*  
