# Task — Propagation Reliability Audit
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-18 (audit closed at J-081; canonical doc shipped at `docs/xgen_propagation_reliability.md`; 4 of 5 sections found drift; Federation Event Propagation completion milestone opened PENDING; M6 (new) Phase 2 scope adjustment Joe-locked direct)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Verify the propagation reliability mechanism for accepted DAG events across the full event lifecycle in XGen — from originator submission through home Node ingestion through local fan-out through federation propagation through sync catch-up. Produce a code-grounded reference document that future design work can cite, identify any gaps, and either close them or file them as tracked follow-on deliverables.

**This task blocks M6 (new).** M6 (new) ships `TransportMessage::EventAccepted` with G2 semantic ("event is in home Node's authoritative DAG store"). G2's claim is meaningful only if DAG-resident events propagate reliably to the rest of the system. The accept signal cannot honestly land before the reliability mechanism it depends on is verified.

**This task is the aggregate-audit realisation of Pass 3's locked approach.** Pass 3 of M6 Phase 0 considered a per-phase verification gate (verify before Phase 2 ships `EventAccepted`) and chose instead a standalone audit milestone that verifies the full propagation lifecycle in one pass. Joe's framing: *"after all work pack we will look on those events at once."* Path B was locked. This file is its task surface.

---

## §1 — Mandatory reading

Read in this order before touching code. The audit's authority derives from these sources; everything else is supporting context.

| Source | What it gives | Why read it |
|---|---|---|
| `docs/xgen_node_admin_ops_design.md` §3 and §4 | The protocol-level addition (`TransportMessage::EventAccepted`) and the propagation lifecycle model. Specifically §4.3 lists the three Stage-6 questions this audit must answer. | The audit's deliverable scope is defined here. |
| `DECISIONS.md` D-070 (proposed) draft | Currently in §9 of the design doc above; promotion to DECISIONS.md happens separately. The named protocol principle the audit upholds. | Context for why the audit matters at the protocol layer, not just the implementation layer. |
| `DECISIONS.md` D-065 | "Honest behaviour over polite behaviour." Sibling principle. | The propagation reliability question is at root a D-065 question: can the system honestly claim what its accept signal claims? |
| `DECISIONS.md` D-069 | Delegated design discipline; canonical-document rule. | This audit produces a new canonical document (`docs/xgen_propagation_reliability.md`). The shape of that document follows D-069's rules. |
| `tasks/NODE_ADMIN_PASS2_PROPOSALS.md` §"Pass-3 input: missing protocol accept signal" | Clair's own J-080 finding that surfaced the accept-signal gap. Specifically the author-exclusion rationale recorded at `xgen-node/src/fanout.rs:469`. | Useful evidence already in hand for some of the audit's questions. |
| `xgen-node/src/fanout.rs` | The local fan-out implementation; already partially traced in J-080. | Stage 5 of the lifecycle. |
| `xgen-core/src/federation/` (entire directory) | Federation transport and handshake code. The audit's primary investigation surface for Stage 6. | Most of the audit's work happens here. |
| `CLAUDE.md` MANDATORY behaviour rules (top of file) | Rules 1–7. | Apply throughout. Quote real code with file:line references (Rule 2), never speculate when uncertain (Rule 1), ask if ambiguous (Rule 6), write JOURNAL last (Rule 4). |

After reading, the model in your head must be: there are five stages between originator submission and event-fully-propagated; the audit's job is to walk all five with code-grounded evidence and surface any gap.

---

## §2 — The lifecycle being audited

Verbatim from §4.1 of the design doc — restated here so this task file is self-contained.

```
[1] Originator submits Event over WS to home Node
[2] Home Node runs 13-step validation pipeline (Ch3 §3.7)
[3] Home Node writes Event to local event store
[4] Home Node sends TransportMessage::EventAccepted → originator   ← G2 boundary (M6 ships this)
        ╔═════════════════════════════════════════════════════════════════╗
        ║   Stages 5+ are asynchronous from the originator's perspective. ║
        ║   The originator's G2 claim ("event is in home Node's store")   ║
        ║   is true and stable from this point.                           ║
        ╚═════════════════════════════════════════════════════════════════╝
[5] Home Node fans out to other locally-connected members (apply_fanout)
[6] Home Node propagates to federated peer Nodes
[7] Federated peers ingest into their own DAGs and fan out to their members
[8] Disconnected clients catch up on next sync_request
```

The audit confirms or refutes the reliability of stages 5, 6, 7, and 8. Stages 1–4 are M6's work; the audit assumes they will be implemented per the design doc.

---

## §3 — Investigation by stage

Per stage, the audit answers a specific set of questions with code-grounded evidence. "Code-grounded" means: cite the file and line number; quote the actual code; explain what the code does. Speculation, "I think this is how it works," or "this is the standard pattern" are not acceptable.

### §3.1 Stage 5 — Local fan-out

**Already partially traced in J-080.** Re-confirm and extend.

Questions to answer:

1. **What does `apply_fanout` do exactly?** Cite `xgen-node/src/fanout.rs:121-128` or wherever the loop lives.
2. **What happens if a recipient's `ClientSender` channel is full?** Trace the `tx.try_send(...)` failure path.
3. **What happens if a recipient is disconnected at the moment of fan-out?** Are they in the `ClientSenders` map at all, and what does iteration do?
4. **Is there any retry mechanism for failed sends?** Search for "retry," "backoff," "pending" patterns near the fan-out path.
5. **Confirm the author-exclusion rationale.** J-080 found "duplicate-avoidance UX, not protocol-correctness" in the code comment at `fanout.rs:469`. Verify this is the only rationale recorded (no separate documentation says otherwise).

**Expected outcome:** Stage 5 is reasonably well-understood; the audit confirms what J-080 surfaced and documents it formally.

### §3.2 Stage 6 — Node-to-Node federation propagation (PRIMARY)

This is the audit's load-bearing investigation. The design doc §4.3 lists three questions; restated here with investigation hooks.

**Question 1 — Federation send buffering.**

Does the Node buffer outbound federation events across WS reconnects, or are events emitted-during-disconnect lost from the federation path?

Investigation:
- Identify the federation send code path. Likely lives in `xgen-core/src/federation/` or `xgen-node-lib/`.
- Find the call site that pushes accepted events to peer connections.
- Examine: is there a queue/buffer between event acceptance and the WS send call? If yes, is it persistent across reconnects, or in-memory only?
- Identify reconnection behaviour. When a federation WS drops and re-establishes, what happens to pending sends?

**Question 2 — DAG-tip reconciliation between federated peers.**

Is there automated DAG-tip reconciliation between federated peers (sync_request-style at the Node-to-Node layer), or does federation rely purely on real-time push?

Investigation:
- Search for `sync_request`, `DAG tip`, `reconcile`, `catch-up` patterns in `xgen-core/src/federation/` and `xgen-node-lib/`.
- Distinguish: client-to-Node sync_request (well-known, used by reconnecting clients) vs. Node-to-Node reconciliation (the unknown).
- If Node-to-Node reconciliation exists: when does it run (periodic, on-reconnect, on-tip-divergence detected)?
- If it does not exist: document the gap clearly.

**Question 3 — Recovery from peer-DAG gap.**

If a peer Node's DAG ends up missing events (whatever the cause), what mechanism brings it back into sync?

Investigation:
- Consequence question of Q1 and Q2. If Q1 says "no buffering" and Q2 says "no automatic reconciliation," then Q3's answer is "no mechanism" — that is a gap.
- If a mechanism does exist, document the trigger condition, the protocol exchange shape, the timing characteristics.
- If no mechanism exists, propose (as part of this audit's recommendations, not as part of M6 scope) what the mechanism would need to look like.

**Question 4 — Federation peer connection lifecycle.**

Not in the design doc's original three questions but discovered during Pass 3 conversation as a related concern.

Investigation:
- When a federation peer comes online for the first time after some downtime, what is the initial sync behaviour? Does it pull the full DAG it missed? Per-Space? Via tip comparison?
- When a federation peer goes offline mid-stream (sending events to other peers, those peers are now offline), what queues exist on the sender's side?
- Search for stress-test evidence — `cmd_stress_test`'s "Federation Completeness" check (counts `apply_event message.text` in receiving Node's log) suggests the team has empirical instrumentation for incomplete propagation. Are there documented findings from stress runs?

### §3.3 Stage 7 — Federated peer ingestion and re-fan-out

Once a federation peer receives an event from another Node, what happens?

Questions to answer:

1. Does the receiving peer run the full 13-step validation pipeline (`accept_event`) on incoming federation events?
2. If validation fails at the receiving peer, what happens? Is the sender notified? Logged? Dropped silently?
3. After ingestion, does the receiving peer run `apply_fanout` for its own connected members?
4. Does the receiving peer re-propagate to its own federation peers? (transitive federation — does the protocol support this, and is it enabled today?)

### §3.4 Stage 8 — Sync catch-up on reconnect

For disconnected clients reconnecting to their home Node.

Questions to answer:

1. What is the protocol message for sync_request? Cite the wire-level shape.
2. What does the Node return? A complete event list, a DAG-tip diff, something else?
3. What happens if the client's known tip is so far behind that the response would be enormous? Pagination? Time-window limit?
4. What happens if the client's known tip references events the Node no longer has (e.g. compaction)?

### §3.5 `TransportMessage::Error` propagation scope

Already partially answered in Pass 3 discussion (inferred to be originator-only). The audit confirms by inspection.

Questions to answer:

1. Cite the emit sites for `TransportMessage::Error` in `xgen-node-lib/` and `xgen-core/`.
2. Confirm: every emit site sends to the originator's connection only, not to fan-out.
3. Confirm: `Error` is never broadcast, never federated, never enters the DAG.

If any of these is *not* true, document the exception clearly with the rationale.

---

## §4 — Deliverables

The audit produces:

### §4.1 New canonical document

**File:** `docs/xgen_propagation_reliability.md`

**Shape:** Section per stage (5, 6, 7, 8) plus a section on `TransportMessage::Error` scope. Each section answers the questions in §3 above with code-grounded evidence. Each section ends with one of three verdicts:

- **VERIFIED WORKING** — the mechanism exists, is correct as far as the audit can tell, and is consistent with what the design doc claims.
- **GAP IDENTIFIED** — the mechanism does not exist or is incomplete. Description of the gap, severity assessment, recommended fix or follow-on deliverable.
- **PARTIALLY VERIFIED** — some questions answered with confidence, others uncertain. The uncertain pieces are flagged for either further investigation (separate task) or explicit Joe-discussion.

**Document header:** Standard project header per the convention. Status: `ACTIVE` on first write, becomes `COMPLETED` when all findings are recorded and any gaps have been filed.

**Document size:** Substantive but not exhaustive. One page-equivalent per stage section is the target; longer if a stage's investigation surfaces complexity.

### §4.2 Gap deliverables (if any)

For each `GAP IDENTIFIED` finding, file the gap as a tracked follow-on. Three placement options depending on severity:

- **Severity high — gap blocks M6 (new):** the gap must close before M6 (new) goes ACTIVE. Either ships as part of this audit (if small) or becomes its own milestone before M6.
- **Severity medium — gap doesn't block M6 (new) but should be tracked:** filed as its own `tasks/<gap_name>.md` file with a CLAUDE.md note. M6 (new) goes ACTIVE on schedule; the gap closes in a future milestone (M7+ typically).
- **Severity low — gap is acknowledged limitation:** documented in the audit document itself, no follow-on file. Future contributors are warned.

Joe approves the severity classification of each gap before filing.

### §4.3 JOURNAL.md entry

Single entry covering the audit. Standard format: real `cargo test` output if any tests run during the audit (the audit may be pure code reading with no test additions, in which case "no tests added — pure code-trace audit" is the honest line). Cites the new canonical document. Lists any gaps filed.

### §4.4 CLAUDE.md update

After audit closes: flip Propagation Reliability Audit from 🟢 ACTIVE to ✅ COMPLETED in the roadmap. M6 (new) is now unblocked — Chat Claude + Joe can write `tasks/NODE_ADMIN_WRITE_PATH.md` and CLAUDE.md flips M6 (new) to ACTIVE in a subsequent edit.

### §4.5 DECISIONS.md — D-070 promotion

If the audit closes with no gaps that invalidate the D-070 principle, Chat Claude promotes the D-070 draft from §9 of `docs/xgen_node_admin_ops_design.md` into DECISIONS.md as a proper numbered decision. This is a separate atomic action after audit close, not part of the audit itself.

If the audit finds gaps that change the principle's framing, Chat Claude and Joe discuss before promoting; the promoted form may differ from the current draft.

---

## §5 — Process discipline

Specific gates and rules that apply throughout the audit.

### §5.1 No code changes in this milestone

This is an audit, not a fix milestone. The deliverable is a document, not a behaviour change. If a gap is found and a fix is recommended, the fix lands in a separate follow-on milestone — never in this audit's commit history.

The only exception: if during the audit an obvious one-line correctness bug is discovered that has nothing to do with the propagation question (e.g. a typo, a clearly-wrong assertion), Clair surfaces it to Joe and Joe decides whether to fix in this audit or file as a separate item. Do not silently fix.

### §5.2 No verb work, no M6 work

This audit predates M6 (new). Do not begin `admin_ops::*` scaffolding, do not add `TransportMessage::EventAccepted` shape, do not touch Node admin verbs. Those are M6 (new) Phase 2's job — and Phase 2 only begins after this audit closes.

### §5.3 Joe approves the section verdicts

For each of the five stage sections (§3.1 through §3.5), Clair writes the section, drafts the verdict, then pauses for Joe approval before moving to the next section. This mirrors the J-079 audit pattern where Clair gated on Joe approval at each section boundary.

Why this discipline matters: a `VERIFIED WORKING` verdict that turns out to be wrong is worse than a `PARTIALLY VERIFIED` honest one. The pause-for-approval rhythm catches over-claims early.

### §5.4 Honest "I don't know" is required

If a question in §3 cannot be answered with code-grounded evidence, the verdict is `PARTIALLY VERIFIED` with the uncertain piece explicitly named. Do not fill gaps with "the system likely does X" or "the standard pattern would be Y." Either the code says it or it doesn't.

### §5.5 No invented test counts

If the audit runs tests (unlikely; it's a code-reading audit), real `cargo test` output is quoted. If no tests run, the JOURNAL entry says so explicitly: "no tests added — pure code-trace audit."

---

## §6 — Definition of Done

This task is COMPLETED when **all** of the following are true and individually verified:

- [ ] `docs/xgen_propagation_reliability.md` exists in the repository with all five stage sections (§3.1–§3.5) populated.
- [ ] Each stage section ends with one of the three explicit verdicts: `VERIFIED WORKING`, `GAP IDENTIFIED`, or `PARTIALLY VERIFIED`.
- [ ] Every claim in the canonical document is supported by a file:line code citation or by a quoted Node-log or test-output line.
- [ ] Joe has approved each stage section's verdict before the next section was written (per §5.3).
- [ ] If any gaps were identified, they are filed per §4.2: severity classified, placement decided, tracking file created if needed.
- [ ] JOURNAL.md entry written and posted, citing the canonical document and any gap files.
- [ ] CLAUDE.md updated to reflect audit COMPLETED status (per §4.4).
- [ ] `tasks/PROPAGATION_RELIABILITY_AUDIT.md` (this file) header updated to `Status: COMPLETED` with the close date in `Last updated`.

Note on Definition-of-Done discipline: the JOURNAL.md entry being written is on the checklist, but pushing the commit is NOT. The Status header flipping to COMPLETED is the real signal that work shipped; the push happens after that flip and cannot itself be a ticked box inside the same commit (chicken-and-egg).

---

## §7 — What this task is NOT

To prevent scope drift:

- This is NOT a fix milestone. No `apply_fanout` changes. No federation buffer additions. No new transport messages.
- This is NOT a verb design pass. No M6 admin verbs in scope.
- This is NOT a Chapter 3 specification revision. If the audit surfaces something that needs a Ch3 change, the change is filed as a separate item; the audit just documents the need.
- This is NOT a substitute for Joe-lock #5 in M6. Joe-lock #5 (failure semantics) was already locked in Pass 3 (best-effort with honest reporting + `stage` field); the propagation reliability question is the load-bearing primitive #5 depends on, not a re-opening of #5 itself.
- This is NOT a verification of the Client-side accept handling. M6's Phase 2 ships the Client match arm for `EventAccepted`; verifying that ship belongs to M6's own Definition of Done, not this audit.

---

## §8 — Cross-references

| Source | Relevance |
|---|---|
| `docs/xgen_node_admin_ops_design.md` §3, §4, §5.3, §9 | The design doc this audit unblocks. |
| `DECISIONS.md` D-065 | Honest behaviour over polite behaviour — the principle the audit upholds. |
| `DECISIONS.md` D-069 | Delegated design discipline — the canonical-document rule the audit's output follows. |
| `DECISIONS.md` D-070 (proposed) | The named protocol principle the audit upholds; promotion happens after this audit closes. |
| `tasks/CLI_PRECEDENCE_AUDIT.md` | Template for the audit pattern. J-079 was the first audit milestone in the project; this one inherits its discipline. |
| `tasks/NODE_ADMIN_PASS2_PROPOSALS.md` §"Pass-3 input: missing protocol accept signal" | The J-080 finding that surfaced the audit's need. Specifically the author-exclusion rationale at `fanout.rs:469`. |
| `xgen-node/src/fanout.rs` | Stage 5 of the lifecycle; already partially traced. |
| `xgen-core/src/federation/` (directory) | Stage 6 of the lifecycle; the audit's primary surface. |

---

*End of task file.*
