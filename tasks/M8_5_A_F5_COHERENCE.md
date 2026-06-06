# M8.5-A — F-5 Federation-Propagation Coherence
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose & the reframe

M8.5-A was opened (Phase-0 audit, `tasks/M8_5_FINALIZATION_AUDIT.md` §4) as a
**propagation-model decision fork** — F-5 transitive (A) vs mesh (B). Grounding
against HEAD `cecb5ee` inverted that premise:

**F-5 is not an open fork. It was decided and locked.** `docs/xgen_federation_propagation_design.md`
**§8.4** locks **Option 1 — no transitive federation in v1** (JOE-LOCKED, May
2026): federation is pairwise; an Event received via federation terminates one
hop from origin. The guard at `federation_session.rs:268` is the §8.5
implementation; `f5_anti_transitivity_received_via_federation_event_not_pushed`
is the regression lock; §8.6 documents the v2 evolution path (Option 3 —
per-peer `transitive_relay` opt-in flag, default off).

So Joe's "B" coincides exactly with the standing, implemented, tested decision.
**This arc decides nothing about propagation.** Its real subject is a
**doc-coherence gap**: the canonical spec (ch3) never absorbed F-5. M8.5-A
closes that gap and corrects the record.

This supersedes audit finding **M85-A6** (see §5).

---

## 2. What is already locked (no change)

- **Decision:** pairwise federation, no transitive relay (federation design §8.4, Option 1).
- **Code:** `apply_federation_push` (`federation_session.rs`) guards on `EventOrigin::ReceivedViaFederation` at `:268` and returns early — received events are not re-pushed.
- **Test:** `f5_anti_transitivity_received_via_federation_event_not_pushed` (`federation_push_integration.rs`) + the deployment-level `phase9_three_node_anti_transitivity` + `phase9_compound_c2_anti_transitivity_at_load`.
- **v2 path:** Option 3 opt-in (federation design §8.6) — forward-compatible, not a v1 behaviour.

Nothing in this list changes in M8.5-A.

## 3. The coherence gap

ch3 — the canonical protocol spec — is **silent** on whether DAG Events received
via federation are re-forwarded. §3.4 (Federation Handshake) covers the
relationship lifecycle (3.4.1–3.4.7) but has **no event-propagation subsection**.
Meanwhile §3.5.5 (Announcement Propagation) *explicitly permits* transitive relay
— for Node-discovery announcements. A reader of ch3 alone could reasonably infer
DAG Events relay the same way. The normative F-5 decision lives only in a design
doc. That is a D-069 canonical-document gap.

The phantom in the M8 finding: there is **no ch3 §3.2 "forward on accept"
premise** (§3.2 is the Event Specification; grep-confirmed absent). The
"S3/S0 transitive assumption" is likewise inverted — the multiparty tests
*assert* anti-transitivity, not transitivity.

---

## 4. The fix — proposed ch3 §3.4.8 (FOR JOE-LOCK)

Additive new subsection appended after 3.4.7 (zero cross-reference breakage;
the alternative — inserting as 3.4.6 and renumbering 3.4.6/3.4.7 → 3.4.7/3.4.8 —
would touch the `(3.4.6)` cross-ref at ch3 line 3177, so append is recommended).
Proposed text:

> #### 3.4.8 Event Propagation over Federation
>
> Once a federation relationship for a Space is ACTIVE (3.4.3), the Node that accepts a DAG Event into that Space's log propagates it **pairwise**: the Event is pushed directly to every Node with which the accepting Node holds an ACTIVE federation relationship for that Space. Propagation is one hop — from the accepting Node to its direct federation peers for the Space.
>
> **Federation is pairwise; there is no transitive relay.** A Node MUST NOT re-forward onward a DAG Event it received *via* federation from a peer. An Event received over a federation relationship is delivered to the receiving Node's local clients (fan-out) and applied to its Space state, but it is **not** pushed to the receiving Node's other federation peers. Consequently, for the participating Nodes of a Space to converge, each must receive every Event directly from the Node that accepted it — in the general case, a full mesh of federation relationships among the Space's participating Nodes.
>
> This is distinct from Announcement Propagation (3.5.5): Node-discovery announcements MAY be relayed transitively, because an announcement is self-certifying discovery data whose authority does not depend on the relaying path. DAG Events MUST NOT be relayed transitively, because transitive relay would extend an Event's authority chain through Nodes the receiver has no direct federation relationship with, weakening the per-Space, per-peer relationship check (3.4.5). Trust in a federation relationship is not transitive.
>
> A future protocol revision MAY introduce opt-in transitive relay (a per-relationship flag, default off) should deployment scale make mesh-relationship cost the limiting factor; this is a forward-compatible extension, not a v1 behaviour. The full decision record and evolution path are at `docs/xgen_federation_propagation_design.md` §8.

Open Joe-lock points: (i) the wording above; (ii) placement (append as 3.4.8
vs insert as 3.4.6 with renumber) — recommend append.

---

## 5. Record corrections (errata, D-065 / D-069)

The phantom §3.2 contradiction propagated from the M8 finding into downstream
records. To correct at close:

- **`tasks/M8_5_FINALIZATION_AUDIT.md` M85-A6 (§4):** amend — strike "contradicting spec §3.2 'forward on accept'"; restate as "F-5 is already locked (federation design §8.4, Option 1); the gap is that ch3 never absorbed it." Bump audit doc version.
- **J-270 canonical records (PLAY / JOURNAL / ROADMAP):** the F-5 lines carry "contradicts spec §3.2". Light correction to "ch3 had not absorbed the locked F-5 decision (federation design §8.4)". These ride the M8.5-A close commit (D-074), not a separate atom.

## 6. DECISIONS.md pointer (FLAGGED, not auto — D-069)

F-5 Option-1 is a cross-cutting protocol invariant (federation authority model)
currently recorded only in a design doc. It arguably clears the D-069 global bar
for a first-class DECISIONS entry ("federation is pairwise; received-via-federation
events are terminal; transitive relay is a forward-compatible v2 opt-in").
**Flagged for Joe's call** — promote at close, or leave the design-doc §8 record
as authoritative with the new ch3 §3.4.8 as the spec-level statement.

---

## 7. Scope fence + close plan

**OUT:** any code change (guard + tests already correct); the Option-3 v2 opt-in
(forward-compat, future revision); INV (M8.5-B) and S5 (M8.5-C).

**Close (doc-only, single commit, D-074):** ch3 §3.4.8 added (Joe-locked text) +
M85-A6 amended + J-270 erratum on PLAY/JOURNAL/ROADMAP F-5 lines + this doc →
COMPLETED + (optional) DECISIONS entry if Joe promotes + J-271. No suite re-run
(doc-only; 1167/0/2 unchanged).

**Next-active after M8.5-A close:** M8.5-B (INV bootstrap) — the headline build,
co-designed per the §4 candidate.

**CLOSED at J-271 (2026-06-05), doc-only.** Executed: ch3 §3.4.8 added (locked
text, appended after 3.4.7) · DECISIONS.md **D-089** synchronized (F-5 promoted to
first-class invariant) · audit M85-A6 corrected · J-270 erratum applied to live
PLAY/ROADMAP F-5 lines (JOURNAL J-270 left as history; correction carried in
J-271). No code (guard + tests already correct); suite 1167/0/2 unchanged.
**Next-active: M8.5-B (INV bootstrap).**

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-078.
