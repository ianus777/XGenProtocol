# M6 Client `members` Command — Membership-Source Design (deferred)
> **Status**: PENDING  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this exists

During M6 Phase 1, the `members --space <id>` Client command — documented in Appendix F §F.3 (command tables) and §F.5.6 (worked example showing pubkey + display name + role + "registered Nm ago", marked `network=No`) — was found to have **no local data source**. `xgen-client_state.json` / `KnownSpace` persist no per-member data; only a Node-side `member_count`. So `members` cannot be produced as the zero-network local read that R1 assumed.

This was surfaced independently by Clair's implementation-time schema check and by the design review — implementation-time divergence the Propagation Reliability Audit explicitly left to this point. `rooms` shipped as Phase 1 (R1); `members` was carved out into this design beat. **Decision: Option 1 (defer `members`), locked 2026-05-29.**

## The decision this design must make

**What is `members`?** Two readings, with different architectures:

### Shape A — Authoritative (Node-query)
`members` queries the home Node for the Space's current membership (pubkey / display name / role / registered-at). Source of truth = the Node's view of the Space DAG.
- **Pros:** always correct and current; no client state growth; matches how membership authority actually works.
- **Cons:** network round-trip (contradicts Appendix F's `network=No`); needs a Node query path; closer in shape to a Phase-7-style network command.
- (Option 2 from the scope elicitation.)

### Shape B — Cached local view (state-schema expansion)
Add `members: Vec<KnownMember>` to `KnownSpace`, populated during join / invite / history-replay, so `members` becomes a true local read.
- **Pros:** genuine zero-network read; preserves Appendix F's `network=No`.
- **Cons:** a write-path / state-tracking change (bigger than a Phase-1 trivium); **inherits a coherence obligation** — the cache must stay consistent with:
  - federation-delivered membership events (members joining/leaving on other Nodes),
  - the revoke-doesn't-cascade rule (A5-D1 / A2-D1) — a revoked identity's membership goes inert but is not retroactively purged,
  - history replay / reconnect deltas.
  - Stale-member and missed-revocation bugs live here.
- (Option 3 from the scope elicitation.)

### Lean (NOT locked — Joe decides at design time)
Authoritative Node-query (Shape A), **or a hybrid:** the client derives membership from DAG events it already holds, and queries the Node only when it knows its local view is incomplete (e.g. immediately post-reconnect, or for a Space it has not fully replayed). The hybrid keeps the common case local while staying correct.

## Sequencing

Sequence **near Phase 7 (Federation management)**, where membership authority and federation-delivered membership events are already in play — the coherence questions Shape B raises are the same surface Phase 7 touches. Not before Phase 2 scaffolding. May be pulled earlier if a client `members` UX is wanted, but the federation-coherence reasoning still applies.

## Dependencies / cross-refs

- **Appendix F** §F.3 (command tables) + §F.5.6 (worked example) — annotated 2026-05-29 as deferred / "target shape — not yet implemented". The chosen shape realises §F.5.6 and lets the `network=` marking be set truthfully.
- **If Shape A:** needs a Node-side membership-query path (relate to A4 `space list-hosted` and the Node's membership reads).
- **If Shape B:** `KnownSpace` schema change in `xgen-client_state.json`; relate to the join / invite / history-replay write paths.
- **Membership model:** Ch2 / Ch3 membership events (`membership.invite`, `membership.join`, …); revoke semantics A5-D1 / A2-D1.

## Definition of Done (this design task)

Flips to ACTIVE when picked up. Produces a locked shape decision (A / B / hybrid) — recorded in DECISIONS.md or inline per D-069 — then an implementation task file. Until then: do **not** implement `members`; Appendix F §F.5.6 stays marked "target shape — not yet implemented".

---

*End of design note.*
