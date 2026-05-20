# XGID Adoption v1 — Design Walkthrough Record
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-20  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Provenance disclosure

This document is a **retrospective design record**, created on 2026-05-20 during Phase 1 canonical sources drafting, after the design walkthrough that produced its content had already closed in the same session. It is not a live design document that tracked the walkthrough as it happened.

The walkthrough itself ran across two sessions (2026-05-20):

- **Session A** — opened Q1 through Q4 partial; ROADMAP.md updated mid-session to absorb locked decisions and the restructured design-phase shape; paused before walkthrough close.
- **Session B** (this session) — resumed at Q4 final framing; closed Q4(a), Q4(b), Q5, Q6; began Phase 1 canonical sources drafting (DECISIONS.md D-072 + D-073, `docs/xgen_appendix_j_en.md`, Ch3 §3.0, `tasks/XGID_ADOPTION_IMPL.md`, normative pointers in `docs/xgen_aicontrol_implementation.md` + `docs/xgen_appendix_f_en.md`); recognised mid-Phase-1 that a paired DESIGN file would match Phase 7.5's established DESIGN/IMPL pairing convention; created this file as a retrospective record of the walkthrough's path.

This file's purpose is to capture the **design process** — the path the walkthrough took, the candidates that were considered, the principles that informed each lock, the mid-session restructure rationale. Outcomes (the locked decisions themselves) live in their authoritative homes (`DECISIONS.md`, `docs/xgen_appendix_j_en.md`, `docs/xgen_ch3_specification.md` §3.0). This file does not re-state outcomes; it records why the outcomes have the shape they do.

The Status flips directly to COMPLETED on creation because the walkthrough is already closed at file-creation time. Per D-065 (honest behaviour over polite behaviour), this is named explicitly rather than pretending the file tracked the walkthrough live.

---

## Walkthrough scope

The XGID Adoption design walkthrough opened to design a project-wide identifier type discipline for XGen Protocol. The walkthrough scope was bounded to six questions Q1–Q6, sequenced so that earlier locks set the frame for later ones. Each question carried into the next with sub-frames; later questions could revisit earlier locks only through explicit re-opening.

The six questions, in walkthrough order:

- **Q1 — Vocabulary scope.** Which protocol concepts qualify as first-class XGID-typed identifiers? Which do not?
- **Q2 — Type-system shape.** How is XGID realised in the Rust reference implementation?
- **Q3 — Adoption discipline.** How does the codebase transition from `String`-typed XGID fields to typed flavour wrappers?
- **Q4 — Wire-format invariance.** What guarantees does the protocol make about XGID stability across the federation wire and the AI control / batch JSONL wire?
- **Q5 — Immutability framing for Ch3.** How does Ch3 state "XGIDs are immutable per object" without inviting renaming proposals?
- **Q6 — Field-name-vs-type discipline canonicalisation.** Where does the §5.6 precedent ("the field name carries the role, the type carries the contract") live as a project-wide rule?

The walkthrough did not pre-enumerate the six questions; they emerged sequentially as Q1's locks made Q2's frame visible, and so on. Q5 and Q6 were not on the agenda when Q1 opened; they surfaced as questions once Q4's wire-format work made adjacent concerns visible.

---

## Q1 — Vocabulary scope — LOCKED

**Decision shape.** Six XGID flavours at v1, organised into two families. No more, no fewer.

- **Hash-anchored family** — Event, Space, Room, TrustAssertion.
- **Principal family** — Node, Identity.

**Candidates considered and rejected at Q1:**

- *Seventh flavour for `session_id`.* Rejected — session_id is structurally an Event XGID with an ephemeral lifecycle. Promoting it to a top-level flavour would expand the closed family for a lifecycle property that the field name + surrounding code context already encode. Filed as a sub-axis in Appendix J §J.7.
- *Composite XGID for `trust_assertion_id`.* Rejected — composing (asserter, subject) structure into the identifier itself would have broken URI grammar invariance. Plain hash-anchored XGID, with the composite structure encoded in the assertion payload, not the identifier.
- *XGID flavour for bootstrap discovery URIs.* Rejected — bootstrap URIs are operational network addresses, not protocol-object identifiers. Distinct surface.

**Boundary clarifications confirmed at Q1 (not XGIDs):**

- Wire-envelope correlation handles (M6 Phase 2's `event_id: Option<String>` on TransportMessage).
- Error codes (`4002`, `4006`, etc.).
- Config field names.
- File paths, log line tokens, debug formatters.

**Why the closure at six.** The walkthrough kept the family closed because an open identifier family — "we'll add new flavours as we need them" — undermines the type discipline's value. Six flavours, well-chosen, do the work; an open list invites accumulation. Adding a seventh flavour in the future requires explicit promotion through a new DECISIONS.md entry, with the barrier deliberately high.

**Output of Q1:** the six-flavour taxonomy, the two-family split, the boundary-case enumeration.

---

## Q2 — Type-system shape — LOCKED

**Decision shape.** Layered newtype — base `Xgid(String)` plus six flavour wrappers each with `Deref<Target = Xgid>`, all serde-transparent as plain strings.

**Candidates considered and rejected at Q2:**

- *Flat — single `Xgid` type with no flavour wrappers.* Rejected — loses type-system enforcement at flavour boundaries. A function expecting an `IdentityXgid` would not be able to refuse a `NodeXgid` at compile time. The whole point of typed identifiers is that miscalls become compile errors; a flat type loses that.
- *Disjoint — six wrappers with no common base.* Rejected — forces common operations (`Display`, `Debug`, `Eq`, `Hash`, `Clone`) to be implemented six times. Forces code that genuinely doesn't care about flavour (e.g. trace logging) to enumerate all six flavours at every use site.
- *Wire-tagged flavours — serialise as `{"flavour": "event", "value": "..."}`.* Rejected — breaks wire-format invariance 2 (field types must be `string`). Adds redundancy to the wire; flavour is already carried by field names per D-073. Promoted later to a Q4(a) rejected-proposal worked example (Case #5).

**XgidLike trait.** Defined alongside the types. Sparingly used: reserved for code that operates over "any XGID" without caring about flavour (trace logging is the canonical use case). Overuse would defeat the point of typed flavours by silently re-flattening them.

**Flavour-specific constructors.** Hide the double-wrap (e.g. `EventXgid::from_event`, `NodeXgid::from_pubkey`). The constructor surface is flavour-specific because the meaning of construction is flavour-specific: hash-anchored takes the object to be identified; principal takes the public key. No "construct any flavour from arbitrary string" constructor — that would let invalid XGIDs into the type system.

**Principal flavours carry pubkey method.** `NodeXgid::pubkey()` and `IdentityXgid::pubkey()` recover the public key from the XGID. At the implementation runbook stage (`tasks/XGID_ADOPTION_IMPL.md`), pinned as parse-fallible (`Result<VerifyingKey, _>`) at v1. The base `Xgid(String)` accepts any string at v1; principal flavours cannot promise more than the construction-source data supports. A future walkthrough may tighten to infallible if parse-on-construction is adopted; deferred.

**Output of Q2:** the layered newtype design, the XgidLike trait scope, the constructor patterns, and the parse-fallible commitment for principal flavours' pubkey access.

---

## Q3 — Adoption discipline — LOCKED

**Decision shape.** Shape γ + ASAP discipline — staged retrofit milestones, with the five passes landing in ROADMAP.md Near future immediately after v1 ships, not Far future.

**Candidates considered and rejected at Q3:**

- *Shape α — retype everything in one mega-milestone before v1 ships.* Rejected — would either delay v1 by months or ship with cut corners. The discipline of touching every XGID-carrying field across four crates and all documentation in one milestone is not honest about the cost.
- *Shape β — staged but deferred.* Retype in subsequent milestones, but place them in Far future without a binding "ASAP" commitment. Rejected — leaves the codebase in mixed discipline indefinitely; the "mixed transitionally" property becomes "mixed permanently" by inattention.
- *Shape γ — staged with ASAP commitment.* Selected. Five passes (1: `xgen-common`, 2: `xgen-core`, 3: `xgen-node`, 4: `xgen-client` + AI-control docs, 5: tests + helpers + trace events + remaining) land in Near future, picked up as the next available work slot after v1 ships.

**Locked principle wording** (verbatim, reproduced in D-072, Appendix J §J.11, Ch3 §3.0.6):

> *"XGID Adoption v1 ships the types and adopts them in new code. Retrofitting existing XGID-string fields is staged into subsystem-scoped retrofit milestones. The codebase MAY carry mixed discipline transitionally; every new field, new signature, and new trace event field MUST use XGID types from this milestone forward."*

**Why "every new field MUST" with "MAY carry mixed discipline transitionally."** The two clauses are deliberately asymmetric. Existing code is allowed to drift behind during the transition (honest about the retrofit cost); new code is bound by the discipline from v1 onward (prevents the retrofit surface from growing). This is the only shape under which the transition has a finite endpoint.

**Output of Q3:** Shape γ + ASAP, the five-pass numbering with subsystem scopes, the locked principle wording.

---

## Q4 — Wire-format invariance — LOCKED in two parts

Q4 closed in two sittings — three-doc placement and five-invariance scope locked in session A, rejected-proposal examples and supplement-d carry-forward locked in session B.

### Q4 session-A locks

**Three-document placement.** Appendix J carries expository long-form; Ch3 §3.0 carries terse normative form; DECISIONS.md D-072 carries architectural commitment. Three surfaces, one authority per surface, no duplication (D-069 canonical-document rule).

**Five-invariance scope:**

1. Field names.
2. Field types (always `string`).
3. Canonical form (byte-identical from same inputs anywhere in federation).
4. URI grammar (prefix, separators, length, character class fixed at v1).
5. String-equality semantics (bytes equal bytes; no normalisation hooks).

**Second wire crossing.** The five invariances bind both the federation wire (Node-to-Node WebSocket) and the AI control / batch JSONL wire (driver-to-implementation named-pipe). Named explicitly in Appendix J §J.5 and Ch3 §3.0.3. Not "two independent promises that happen to coincide" — one promise across both surfaces.

### Q4(a) — Rejected-proposal worked examples — LOCKED

Session-B opener walked the candidate enumeration. Six candidates surfaced (#1–#6); three patterns emerged:

- *Wire-invariance rejections* — proposals that would break one of the five invariances. Strongest fit for §J.X.3 (renamed to §J.9 at drafting time).
- *Boundary clarifications* — Q1 boundary cases (session_id, composite trust_assertion_id, M6 event_id). Live earlier in Appendix J as scope definitions, with "see also" pointer from §J.9.
- *Immutability rejections* — Case #1 ("rename a Space, keep XGID"). Moved to Q5.

**Two rejected proposals locked for §J.9:**

- *Case #5 — "Use the in-memory handle type as the wire type."* Selected as load-bearing: natural-looking proposal that breaks an invariance subtly. Exactly where principle-only fails.
- *Case #2 — "Shorten the URI grammar for compactness."* Selected: compactness arguments recur in protocol design; the rejection reasoning isn't self-evident.

Third slot reserved (J.9.3) for future walkthroughs.

### Q4(b) — Supplement-d carry-forward — LOCKED

The question: do `docs/xgen_aicontrol_implementation.md` and Appendix F get Option C hybrid annotation at v1, or staged with Retrofit Pass 4?

**Three frames considered:**

- *Option C at v1.* Full annotation now. Cost: expands Phase 1 scope, risks piecemeal-doc-discovery problem.
- *Staged Pass 4.* No v1 touch. Cost: readers of the implementation docs have no signal that XGID discipline applies there; new code might be written in `String` and conflict with Pass 4 retroactively.
- *One-line normative pointer at v1, full annotation at Pass 4.* The hybrid that surfaced as a third option during framing.

**Locked: the one-line pointer.** Minimum viable awareness signal — says (1) XGID discipline applies here, (2) full retrofit is scheduled, (3) where to find authority. Cheap to write, cheap to remove when Pass 4 lands. Pointer scope: `docs/xgen_aicontrol_implementation.md` and `docs/xgen_appendix_f_en.md`. Pass 4 confirmed as the AI control / batch JSONL retrofit pass after reading ROADMAP.md Near future.

**Output of Q4:** three-doc placement; five-invariance scope; both-wire-crossings binding; §J.9 with two worked examples + reserved third slot; v1 pointer in two supplement-d docs.

---

## Q5 — Immutability framing for Ch3 — LOCKED

**Decision shape.** Option C — layered. Ch3 §3.0.2 carries the declarative sentence (short, normative); Appendix J §J.4 carries the construction-derived expansion (longer, expository); Appendix J §J.10 carries the worked example (Case #1 "rename a Space").

**Candidates considered and rejected at Q5:**

- *Option A — declarative invariant only.* A single normative sentence in Ch3, no further grounding. Rejected — leaves Ch3 stating "XGIDs are immutable" without explaining why, which invites exactly the proposal we want to prevent ("but couldn't we just rename...").
- *Option B — construction-derived only.* Long explanation grounded in how XGIDs are constructed, placed only in Appendix J. Rejected — Ch3 is normative and needs the rule stated tersely; making readers go to Appendix J to find what the rule even is fails Ch3's purpose.
- *Option C — both, layered.* Selected. Ch3 has the rule; Appendix J has the why. Construction-grounding explains why "rename a Space" doesn't compute (the XGID isn't of the Space's name; it's of the founding event's bytes).

**Why immutability is structural, not policy.** The walkthrough was explicit that the immutability property is a consequence of how XGIDs are constructed, not a rule the protocol enforces by checking it. Hash-anchored XGIDs are immutable because the hash function is deterministic and the founding object is never modified. Principal XGIDs are immutable because the public key is the identity and changing the key means a different principal. There is no `mutate_xgid()` operation that the protocol forbids — there is no such operation, full stop, by construction.

**Output of Q5:** Option C placement; the worked example in §J.10; structural-not-policy framing throughout.

---

## Q6 — Field-name-vs-type discipline canonicalisation — LOCKED

**Decision shape.** Option A — D-073 in DECISIONS.md as the canonical home for the principle, with a single-sentence echo in Appendix J's introduction pointing back to D-073. §5.6 of `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` stays as the originating-precedent record (the `introducer_node_id: NodeXgid` worked example).

**Candidates considered and rejected at Q6:**

- *Option B — Dedicated section in Appendix J.* Rejected — would tie the principle conceptually to XGID. The principle is project-wide (applies to capabilities, error codes, event-type discriminators, anywhere a typed field carries a role-bearing semantic). Placing it in Appendix J understates the scope; future contributors looking for "how do we name fields in this project" wouldn't find it in the obvious place (DECISIONS.md).
- *Option C — Stays inline in Phase 7.5 §5.6.* Rejected — fails D-069 canonical-document rule. A Phase 7.5 design file is a milestone artefact whose authority decays once Phase 7.5 ships. The principle's authority should not decay with it.

**Decision framing scope.** The walkthrough was explicit that D-073 frames narrowly: one decision, one principle, one entry. Future related typing-discipline decisions (parse-on-construction invariants, cross-flavour interaction rules, etc.) get their own D-numbers if they surface. The narrower framing keeps DECISIONS.md entries focused.

**Output of Q6:** D-073 narrow framing; Appendix J introduction echo; §5.6 historical-precedent retention.

---

## Mid-session restructure of the design phase

A mid-walkthrough restructure changed the Phase 1 deliverable shape. The change happened in session A, mid-Q4, and is recorded here because it materially affected the Phase 1 commit scope and the existence of this design file (originally five upfront deliverables; reshaped to canonical-sources-first + doc-tree-sweep-second).

### Original five-upfront deliverable shape

Phase 1 originally planned to ship five deliverables in parallel:

1. Architectural commitment (DECISIONS.md entry).
2. Expository document (Appendix J).
3. Normative section (Ch3 §3.X).
4. Implementation runbook (Clair-facing).
5. Full annotation pass across every affected document at v1.

### Trigger for restructure

Mid-Q4, the walkthrough kept finding additional affected documents piecemeal — Appendix I (touches identifier serialisation), Appendix F (touches CLI parameters carrying identifiers), `docs/xgen_aicontrol_implementation.md` (touches batch reply schemas with identifiers), Ch6 §6.15 (touches client-side AI surface). Each one surfaced as a "and we should also annotate..." add-on. The five-deliverable shape was implicitly assuming we knew the full set of affected docs upfront; the walkthrough was demonstrating we didn't.

### Restructured two-phase shape

Phase 1 narrowed to **canonical sources commit**: DECISIONS.md (D-072 + D-073), Appendix J, Ch3 §3.0, implementation runbook, plus minimal pointers in the two most-affected secondary docs (`xgen_aicontrol_implementation.md`, Appendix F).

Phase 2 became **doc-tree sweep**: a separate task file walks every main doc in `docs/` and produces a classification table — which docs update at v1, which update with a specific retrofit pass, which need no update. Deliverable is the classification table, not the edits themselves.

The inversion (canonical sources first, classification second) avoids the piecemeal-discovery problem by deferring the doc-by-doc question until the canonical sources exist and a uniform classification rule can be applied.

### Why this restructure happened mid-walkthrough

D-065 in action. The five-deliverable shape was a "polite" upfront plan that wasn't honest about how much we knew. Discovering mid-Q4 that we didn't know the full set was the honest signal; restructuring rather than barrelling through was the honest response. Sibling principle: "honest longer work over fast shortcuts" — Phase 2 takes longer than the original plan implied; it lands the right thing.

---

## Phase 1 deliverables — final shape

After restructure, Phase 1 ships:

1. **`DECISIONS.md`** — D-072 (XGID Adoption v1) + D-073 (field-name-vs-type discipline).
2. **`docs/xgen_appendix_j_en.md`** — canonical expository document. Twelve sections (§J.1–§J.12).
3. **`docs/xgen_ch3_specification.md` §3.0** — terse normative section. Six subsections (§3.0.1–§3.0.6).
4. **`tasks/XGID_ADOPTION_IMPL.md`** — Clair-facing implementation runbook. Two-commit plan; `xgen-common` types + invariance tests, then `introducer_node_id` retype.
5. **`docs/xgen_aicontrol_implementation.md`** — normative pointer added near document head.
6. **`docs/xgen_appendix_f_en.md`** — normative pointer added near document head.
7. **`tasks/XGID_ADOPTION_DESIGN.md`** — this file (retrospective design record).
8. **`CLAUDE.md`** — PLAY block refreshed (pending at time of writing).
9. **`docs/ROADMAP.md`** — Present section updated to reflect walkthrough closed + Phase 1 shipped (pending at time of writing).

Phase 1 commits in one atomic commit per session opener's "one commit, confirmed" decision.

---

## Findings deferred to Phase 2 sweep

This section records findings that surfaced during Phase 1 drafting but were deferred to the Phase 2 doc-tree sweep rather than resolved unilaterally. Each finding names the question, the deferral rationale, and what the Phase 2 sweep is expected to answer.

### Finding 1 — Ch6 §6.15 pointer scope (Scope-A vs Scope-B)

**Surfaced.** 2026-05-20, mid-Phase-1, during Q4(b) pointer-set construction. Q4(b) locked one-line normative pointers in `docs/xgen_aicontrol_implementation.md` and `docs/xgen_appendix_f_en.md`. Phase 1 drafting noticed that Ch6 §6.15 is the client-side spec home for AI Client, covering the same AI control / batch JSONL surface area as the two pointer-receiving docs.

**The question.** Does Ch6 §6.15 receive a pointer at v1, or does the existing Ch3 §3.0 normative section already bind it transitively as a spec document?

**Two scopes the question opens:**

- **Scope A** — *Pointer set extends to all AI-control-surface docs.* The five-invariance promise extends to AI control / batch JSONL; readers of any AI-control-surface doc benefit from explicit awareness. Under Scope A, Ch6 §6.15 receives a pointer at v1 alongside the implementation doc and Appendix F.
- **Scope B** — *Implementation docs get pointers; spec docs inherit invariance through Ch3.* The two pointer-receiving docs (`xgen_aicontrol_implementation.md`, Appendix F) are not normatively bound to Ch3 — they are implementation specifications and CLI references. Ch6 §6.15, being a spec document, inherits invariance through Ch3 §3.0's normative sentence without needing a redundant pointer. Under Scope B, Ch6 §6.15 needs no v1 touch.

**Deferral rationale.** The Phase 2 doc-tree sweep is the natural venue for this classification. The sweep walks every doc and applies the Scope-A/B axis uniformly across the document tree; deciding it for Ch6 §6.15 alone at Phase 1 would pre-empt a classification that should be applied consistently.

**What Phase 2 must resolve.** Whether Scope A or Scope B is the project's rule, and apply it uniformly across all spec docs (Ch3, Ch4, Ch5, Ch6, all appendices) and all implementation docs.

**Held at Phase 1.** The two-doc pointer scope as originally locked at Q4(b) — `xgen_aicontrol_implementation.md` and Appendix F only. Ch6 §6.15 receives no Phase 1 edit.

---

## Cross-references

- `DECISIONS.md` D-072 — architectural commitment locked by this walkthrough.
- `DECISIONS.md` D-073 — field-name-vs-type discipline locked by this walkthrough.
- `docs/xgen_appendix_j_en.md` — canonical expository document.
- `docs/xgen_ch3_specification.md` §3.0 — normative section.
- `tasks/XGID_ADOPTION_IMPL.md` — paired implementation runbook (DESIGN/IMPL pairing, matching the Phase 7.5 convention).
- `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` §5.6 — originating precedent for the field-name-vs-type principle.
- `docs/ROADMAP.md` — five Retrofit Passes 1–5 in Near future; Phase 2 doc-tree sweep next-active after this milestone.

---

*End of XGID Adoption v1 Design Walkthrough Record.*  
