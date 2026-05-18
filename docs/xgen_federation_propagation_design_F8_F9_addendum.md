# XGen Federation Event Propagation — Design (F-8 + F-9 addendum)

> **Status**: PENDING  
> Version: 0.1  
> Date: May 2026  
> **Last updated**: 2026-05-18 (F-8 and F-9 surfaced and Joe-confirmed in Pass 2 conversation)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools. Pass 2 addendum to `docs/xgen_federation_propagation_design.md`; merged into the canonical document at Pass 3 per the D-069 canonical-document rule.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## About this addendum

Second Pass 2 addendum to `docs/xgen_federation_propagation_design.md`. Covers F-8 (Ch4 correction timing) and F-9 (admin-ops design doc correction timing) together because the two decisions are structurally identical and locked the same way.

The first Pass 2 addendum is `docs/xgen_federation_propagation_design_F7_addendum.md` (F-7 pagination). Pass 3 consolidates all addenda into the canonical design doc.

---

## 11. Framework decision F-8 — Ch4 lines 779 / 825-827 correction timing

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

### 11.1 The question

Audit §4.6 and §6.2 identified two specific stretches in `docs/xgen_ch4_implementation.md` that describe mechanisms that do not exist in code:

- **Line 779** — implementation description of `transport.sync_request` that includes the unimplemented `sync_response` and `sync_complete` reply shapes. Ch3 spec calls for them; Ch4 describes them as if implemented.
- **Lines 825-827** — text describing Node-to-Node sync behaviour that does not exist in production.

Both are factual drifts where Ch4 describes what was intended rather than what exists. The correction itself is not in question; the question is when.

### 11.2 Options considered

**Option 1 — Correct during Pass 2.** Replace the drifted text now with either a reservation note or accurate "today's-behaviour" text. Two Ch4 edits across the milestone — once now, once when implementation lands.

**Option 2 — Correct at Pass 3.** When Pass 3 promotes the design doc to ACTIVE, do the Ch4 correction in the same commit. The Ch4 text gets replaced with text that forward-references the canonical design doc (`xgen_federation_propagation_design.md`) and acknowledges the mechanism is deferred to its implementing milestone.

**Option 3 — Correct at implementation runbook phase.** Leave Ch4 untouched through Pass 2 and Pass 3. The correction is part of the runbook's "documentation pass" step when the actual code lands.

### 11.3 Decision — Option 2 (correct at Pass 3, same commit as design doc ACTIVE promotion)

**The Ch4 correction is performed in the same commit that flips `xgen_federation_propagation_design.md` Status from PENDING to ACTIVE.** The corrected text becomes a forward-reference to the canonical design doc, honest about the implementation state ("specified in the federation propagation design; implementation follows in the corresponding milestone").

**Reasoning recorded.**

1. **Pass 3 is the natural publication moment.** That is when the design doc flips to ACTIVE and gets cross-referenced by everything else. Folding the Ch4 correction into the same commit means the cross-reference (Ch4 → design doc) is alive from the moment the design doc itself becomes authoritative.
2. **"Describes a deferred mechanism" is better than "describes a mechanism that does not exist."** Today's Ch4 text is misleading because it does not say "this isn't built yet." A Pass 3 correction that explicitly forward-references the design doc and acknowledges the deferred state is honest about the project's posture — consistent with D-065 (honest behaviour over polite behaviour).
3. **Pass 2 already has enough scope.** Adding Ch4 edits during Pass 2 means design-discussion turns also include "and now let me edit Ch4" detours. Better to keep Pass 2 focused on decisions and batch the documentation fix at Pass 3.

Option 3 has the appeal of "one move, end state is accurate" but the cost is real: weeks of misleading text in a load-bearing document. Option 1 fragments the Ch4 edits across multiple phases without strong benefit.

### 11.4 Concrete correction sketch

The exact rewrite is the runbook's job at Pass 3 — this section sketches the shape so future readers know what to expect:

- **Line 779 (sync_request implementation description).** Replace text describing the unimplemented reply shapes with a forward-reference: "The protocol-level reply shapes (`sync_response`, `sync_complete`) are specified in Ch3 §3.3.6 and `xgen_federation_propagation_design.md` §9 (F-6). Implementation lands in the federation propagation completion milestone."
- **Lines 825-827 (Node-to-Node sync description).** Replace text describing absent Node-to-Node behaviour with: "Node-to-Node federation event propagation is specified in `xgen_federation_propagation_design.md` (F-1 through F-7). Implementation lands in the federation propagation completion milestone."

The exact phrasing belongs to the Pass-3-and-implementation-runbook author. The principle is: forward-reference the design doc, acknowledge the deferred state, never describe behaviour as if implemented when it is not.

---

## 12. Framework decision F-9 — `xgen_node_admin_ops_design.md` §4.2 correction timing

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

### 12.1 The question

Audit §6.2 identified that `docs/xgen_node_admin_ops_design.md` §4.2 describes Node-to-Node federation in a way that does not match the actual code. The drifted text suggests federation push exists in some form; the audit confirmed it does not.

The structural question is identical to F-8: when is the correction made?

### 12.2 Options considered

The same three options as F-8 — correct during Pass 2, correct at Pass 3, correct at implementation runbook phase — apply here with the same trade-offs.

### 12.3 Decision — Option 2 (correct at Pass 3, same commit as design doc ACTIVE promotion)

**Same as F-8.** The §4.2 correction is performed in the same commit that promotes `xgen_federation_propagation_design.md` from PENDING to ACTIVE. The corrected text becomes a forward-reference to the canonical design doc.

**Reasoning recorded.** Structurally identical to F-8 §11.3:

1. Pass 3 is the natural publication moment for cross-references to the newly canonical design.
2. "Describes a deferred mechanism" is better than "describes a mechanism that does not exist," and consistent with D-065.
3. Pass 2 stays focused on decisions, not documentation-cleanup edits to adjacent docs.

### 12.4 Concrete correction sketch

`xgen_node_admin_ops_design.md` §4.2 currently describes Node-to-Node federation push. The Pass 3 rewrite replaces this with:

- A statement that Node-to-Node federation event propagation is specified in `xgen_federation_propagation_design.md` (F-1 through F-7).
- A statement that implementation lands in the federation propagation completion milestone.
- Where §4.2's surrounding context refers to specific federation-relationship admin verbs (M6 territory), retain those references — they belong in this doc as admin-ops design, and they couple correctly to the federation propagation work.

Exact phrasing is Pass-3 / runbook author's call. Principle is the same as F-8: forward-reference, acknowledge deferred state, never describe behaviour as implemented when it is not.

---

## F-8 and F-9 combined lock state

| F-item | Decision |
|---|---|
| F-8 — Ch4 lines 779 / 825-827 correction timing | Option 2 — correct at Pass 3, same commit as design doc ACTIVE promotion. Forward-reference to canonical design doc. |
| F-9 — `xgen_node_admin_ops_design.md` §4.2 correction timing | Option 2 — same as F-8. Forward-reference to canonical design doc. |

Pass 3 will fold these sections into the canonical design doc and execute the corrections in the same commit. After Pass 3, this addendum file is deleted.

One framework decision remains in Pass 2: **F-10 — DAG hole semantics when validation fails on a federated event with unknown predecessors AND unknown sender Identity.**

---

*End of F-8 + F-9 addendum.*  
