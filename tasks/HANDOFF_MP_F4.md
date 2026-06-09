# Handoff — MP-F4 is the next loop-to-green fix-arc (Joe-locked 2026-06-09)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-09  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

**Resolves the J-328 open question.** Joe locked the next loop-to-green fix-arc order on
2026-06-09: **MP-F4 (node-side DM membership resolution) goes next, ahead of MP-F1b.**

**Why MP-F4 first (rationale, recorded):**
1. On the critical path regardless — single-node 2-party DM message convergence stays blocked until
   MP-F4 even if MP-F1b/(iii) ships; not downstream of the federation decision.
2. De-risks MP-F1b's Phase-0 — F1b grounds (iii)/gate-B against a *correct* single-node DM membership
   model instead of fighting two defects at once. Findings doc says "weigh together, **not a merge**"
   — F4-first honours that.
3. Smaller, local, lower-risk — no federation, no cross-node harness; fix direction half-grounded.
   F1b carries a feasibility gate (gate B, home-node resolvability) that can kill (iii) — do the
   fragile arc second.
4. Concrete green in hand — F4 lets `MP-C-07-LOCAL` graduate from delivery-only to asserting message
   convergence before the cross-node work.

Different code surfaces, so no rework risk: MP-F4 is node-side membership resolution
(`state_key` / bootstrap); MP-F1b (iii) is `federation_nodes` population at membership-apply. Overlap
is conceptual, not line-level.

---

**Clair — open MP-F4's D-071 Phase-0** (`tasks/MP_F4_DM_MEMBERSHIP_AUDIT.md`). Phase-0 must ground,
NOT lock:
- The two candidate fix directions against live code — (a) room-scope the membership `state_key`
  (`xgen-core/src/resolution/state_key.rs:48`, currently `membership:{space}:{sender}`,
  room-agnostic); (b) gate `get_invite_bootstrap` to non-members (`xgen-client/src/batch.rs:179`,
  re-issues the invite after the invitee is already a member). Recommend one with rationale → Joe-lock.
- D-076 ordering check — a `state_key` change touches resolution; prove non-interference or scope it.
- Backward-coherence (D-077) — confirm the MP-C-01 regular-Space contrast still passes after the fix
  (the Node refuses bootstrap once a member, so its room-join chains causally — must not regress).
- The witness flip — define `MP-C-07-LOCAL` going delivery-only → asserting message convergence
  (the RED→GREEN sensitivity witness; must be genuinely RED if the fix is reverted).
- F1b cross-link — note where this resolution surface overlaps (iii)/MP-F1b; flag for F1b Phase-0,
  do not merge.

**Entry point (Rule 0):** CLAUDE PLAY → JOURNAL J-328 → `tasks/MP_findings.md` v1.4 (MP-F4) →
this handoff.

**Standing discipline:** Clair's code commit FIRST, then Chat's doc-bridge — which records this
order-lock + the MP-F4 Phase-0 open together at J-329. Joe pushes; Claude never pushes.

**Consume:** mark `Status: COMPLETED` (or remove) once MP-F4 Phase-0 lands and J-329 captures it.
