# MP-F1b — cross-node DM convergence (membership-driven DM federation) — D-071 PHASE-0 AUDIT

> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-09  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Discipline note

Phase-0 grounding only — **no code, no design lock**. Built on the **authoritative J-327
re-supply** of (iii)/A–E/gate-B (Joe handed the real lock text — §1/§2 are not reconstructed from
memory; the J-327 guardrail required this before authoring). MP-F1b is the cross-node, protocol-design
arc with a kill-gate (sub-lock B); this audit's job is to ground the (iii) population path + run
gate-B's verdict against live `main`. Standing: Phase-0 → design → Joe-lock → runbook → implement →
close. **Honest-over-fast (D-065): if gate-B kills (iii), that is the finding — surfaced, not worked
around.**

---

## 1. The finding (facet-1, authoritative J-327)

Cross-node DM convergence fails because a DM Space's `federation_nodes` stays **empty**. Chain:
a DM Space has `dm_constraints_active=true` → `apply_federation_add` rejects (`DmFederationNotAllowed`,
state.rs:660) → `SpaceState.federation_nodes` never populated → `apply_federation_push` early-returns
→ no DM event federates either direction. **Empirically confirmed (J-327):** every DM event push
logs `federation_nodes=0` (all 7 attempts, both nodes). Only `dm_space_create` reaches both nodes
(it rode the G-6 handshake catch-up). This is a **protocol-design tension, not a patch** — DMs are
constrained from federation by design (3.16.1 / M8.5), yet MP-C-07 expects cross-node convergence.

Routed (J-319) as MP-F1; split (J-327/J-328) — facet-2 (delivery) shipped as MP-F1a; **facet-1 is
MP-F1b** (this).

---

## 2. Resolution = (iii) membership-driven DM federation (Joe-LOCKED J-327)

Over (i) rescope-the-test and (ii) loosen-the-gate. **Leave `DmFederationNotAllowed` fully intact —
no third-party node ever receives DM content.** Add a *separate* rule: a DM's `federation_nodes` =
exactly the home nodes of its **current members**, populated at **membership-apply time** (NOT via
`apply_federation_add`). Symmetric — both parties' home nodes hold the DM, neither privileged;
respects institutional independence; keeps 3.16.1's privacy containment. **MP-C-07 is NOT rescoped —
it is a real code arc.**

**Sub-locks A–E (authoritative J-327):**
- **A — population site:** the membership-apply path (invite-accept / join), **not**
  `apply_federation_add`.
- **B — home-node resolvability = THE feasibility gate:** (iii) only works if, at membership-apply
  time, each node can resolve a joining member → their home node. Plausible under no-anonymity
  identity-verification but **UNCONFIRMED**. Phase-0's first job is to prove this empirically; **it
  can kill (iii).** ← §4.
- **C — leave semantics:** the set shrinks for future events; the historical local copy is the
  standing federated-RTBF tension — acknowledged in the design, **NOT solved in-arc**.
- **D — create vs ongoing:** `dm_space_create` keeps riding the G-6 handshake catch-up; subsequent
  events use the populated set; DAG dedup (MP-F3) covers overlap.
- **E — new invariant (DECISIONS candidate, NOT yet promoted):** *"A DM's federation set is exactly
  its members' home nodes; no other node receives DM content"* — replaces the blunt "DMs never
  federate." Promotes per D-069 once held across the arc.

---

## 3. Grounding — the (iii) population path (live `main`)

### 3.1 Sub-lock A (population site) — feasible, but NOT inside `SpaceState`

- `SpaceState::apply_join` / `apply_invite` (the membership-apply appliers, `space/state.rs`) are
  **pure** — they hold no `IdentityRegistry`, so they cannot resolve a member → home node themselves.
- The registry-aware layer is `NodeRuntime` (`ingest_event` / `dispatch_event`), which already builds
  `identity_id → home_node` via `build_identity_home_nodes(&self.identity_registry)`
  (runtime.rs:1895; `IdentityRecord.home_node`, registry.rs:47) and threads it into `derive_resolved`.
- ⇒ **The (iii) population hook belongs at the `NodeRuntime` membership-apply point** (after a
  membership event applies, read each current member's home node from the registry and set
  `federation_nodes`), **not** in `SpaceState` and **not** in `apply_federation_add`. Sub-lock A is
  feasible *given the registry holds the member's record* — which is exactly gate-B (§4).

### 3.2 The send path that consumes `federation_nodes`

`derive_event_nodes` (fanout.rs:178) assembles recipients = `home_node` + `federation_nodes` (+ other
sources); `apply_federation_push` (federation_session.rs:247) early-returns on the empty set. So once
`federation_nodes` carries the counterparty's home node, the existing push path federates DM events
to it **without touching `DmFederationNotAllowed`** (that gate is on `apply_federation_add`, a
different surface). (iii)'s send half is wired the moment the set is populated.

### 3.3 The MP-F3 dedup interaction (sub-lock D) — composes

`dm_space_create` rides the G-6 catch-up (already on both nodes); subsequent events use the populated
set. Where the two overlap (an event that both rode catch-up and is federation-pushed), MP-F3's
`store.contains(event_id)` dedup gate (`DispatchOutcome::Duplicate` → `FanoutRequest::none()`)
suppresses the re-fan-out. Grounded: sub-lock D composes with the shipped MP-F3.

---

## 4. Gate-B — THE feasibility gate (the load-bearing section)

**The question (sub-lock B):** at DM-membership-apply, can a node resolve the *counterparty's* home
node? (iii) populates `federation_nodes` from members' home nodes; for the **local** member (creator)
that is trivial, but for the **remote** member it requires the counterparty's `IdentityRecord` —
specifically `home_node` — to be resolvable on this node.

**Code-grounded verdict: the only source of a counterparty's `home_node` is this node's
`IdentityRegistry`, populated by identity replication — and nothing carries it at DM-create.**

1. `create_dm_space` (ops.rs) sets the DM root's `home_node` = the **creator's own node**
   (`node_override`, else `session.home_node`). The content carries `target_identity` (the
   invitee's *identity_id* only) — **not** the invitee's home node. `from_dm_space_create` reads
   `content["home_node"]` = the Space's (creator's) home (state.rs:355). So the DM creation flow
   **never learns or carries** the invitee's home node.
2. The only production resolver is `build_identity_home_nodes(&IdentityRegistry)` →
   `IdentityRecord.home_node`. So the invitee's home node is resolvable **iff the invitee's
   `IdentityRecord` is already in this node's registry**.
3. A registry record for a *remote* identity arrives only via **identity replication**
   (`push_identity_to_peers` → `handle_incoming_replicate`, app.rs:2856/2915), which itself runs over
   an **already-established federation relationship**.
4. The `directory_url` surface (admin_ops, bootstrap-client arc) is **node-federation discovery**, not
   an `identity_id → home_node` resolver — it does not close the gap.

**∴ Harness-vs-production split (sub-lock B's kill-gate firing in exactly the case it was written to
catch):**
- **Harness (MP-C-07 federated):** G-6 pre-establishes federation **and** identity replication before
  the DM, *and* the manifest pre-shares the invitee's identity_id (`{{bob_identity_id}}` export). So
  the invitee's `IdentityRecord` (with `home_node`) **is** in the creator's registry at
  membership-apply → (iii) resolves → gate-B **passes in the harness**.
- **Production (fresh DM to a not-yet-related stranger):** the invitee's `IdentityRecord` is **not**
  replicated (no prior federation relationship), and the DM flow carries no home node → the creator's
  node **cannot resolve** the remote member's home node at membership-apply → (iii) **cannot populate**
  the remote member → gate-B **FAILS in production** as the code stands.

This is a chicken-and-egg: (iii) needs the counterparty's home node to *form* the DM's federation set,
but the only way to learn it (replication) presupposes a federation relationship that the DM itself
was supposed to establish.

**Empirical confirmation — status.** J-327 names the empirical two-node experiment as Phase-0's
verdict step. The finding above is an **absence** (no production path resolves a stranger's home node),
which is established by exhaustive reading of the create / content / registry / directory surfaces.
The clean empirical demonstration of the *production* failure is **not expressible on current harness
rails**: G-6 is the only way the harness establishes federation, and it pre-seeds replication; the
manifest pre-shares identity_ids. So a faithful "fresh DM to an unreplicated stranger" scenario needs
**new harness machinery** (know an identity_id WITHOUT having replicated its record — i.e. an
identity-discovery step distinct from G-6). Recommendation: treat the code-grounding as the verdict
(decisive on the absence), and build the empirical witness in the design/impl phase **only if** a
chosen augmentation needs it — sibling to MP-A-01(ii) PENDING (a property proven in-process whose
clean cross-node repro the current rails can't express).

---

## 5. Sub-locks C / D / E — grounding

- **C (leave / federated-RTBF):** confirmed standing tension — a node that held DM events as a member
  keeps its local copy after leaving; (iii) only shrinks the *future* set. Acknowledged, NOT solved
  in-arc (consistent with the J-327 lock). No code surface forces it.
- **D (create vs ongoing):** §3.3 — composes with MP-F3 dedup; `dm_space_create` rides G-6, subsequent
  events use the populated set.
- **E (invariant):** "a DM's federation set is exactly its members' home nodes; no other node receives
  DM content." DECISIONS candidate; promote per D-069 once held across the arc. Phase-0 records it,
  does not promote.

---

## 6. MP-F4 cross-link (J-331) — composes, does not reopen

MP-F4 (shipped, `bc057f8`) fixed the **node-side single-node** DM membership surface: A1 room-scoped
the membership `state_key` + the `get_dag_tips` **frontier** anchor made the room-join causally
descend from the space-join. MP-F1b's (iii) population is a **different** surface (NodeRuntime
membership-apply → `federation_nodes`), and it must **compose** with F4, not reopen it:
- F1b reads **Space** membership (current members' home nodes) — orthogonal to F4's room-scoped
  membership keys (room-level facts don't enter `federation_nodes`).
- F1b adds no new membership `state_key`; it populates a derived field at apply-time → no interaction
  with F4's conflict-domain keying. **Confirm at design:** the population hook reads the *resolved*
  member set (post-`derive_resolved`), so it sees F4's correct space-membership.

---

## 7. D-076 / D-077 checks

- **D-076 (ordering/convergence):** (iii) populates `federation_nodes` — a **derived** projection of
  membership, per the D-075 vantage-aware shape (the J-250 lesson: a relationship-shaped field is a
  derived projection, populated at apply-time, vantage-aware). Population from the *resolved* member
  set is convergence-safe **iff** every node derives the same member set (it does, post-M8) AND every
  node resolves the same member→home mapping (gate-B: only if records are replicated consistently).
  **The D-076 obligation is contingent on gate-B** — flag for design.
- **D-077 (backward-coherence):** `DmFederationNotAllowed` stays intact (no regression to the privacy
  containment); regular-Space federation is untouched (this is DM-only). The set-shrink-on-leave (C)
  is new behaviour, not a regression.

---

## 8. Verdict + Joe-lock asks

**Verdict: GAP CONFIRMED (facet-1 real); but gate-B (sub-lock B) FAILS the production case as the
code stands** — there is no production path that resolves a not-yet-replicated counterparty's
`home_node` at DM-membership-apply (§4). (iii) **works in the harness** (G-6 pre-seeds) but is
**insufficient for production** without an augmentation. This is the kill-gate firing exactly as
sub-lock B anticipated — surfaced, not worked around (D-065).

**The fork this forces (design-phase, Joe-lock):**
- **(iii)+ augment — identity→home resolution.** Keep (iii); add the missing capability: a way for a
  DM creator's node to resolve the invitee's home node before/at membership-apply (e.g. carry the
  invitee's home node in the DM-create/invite content, sourced from an identity-discovery step; or a
  directory that maps identity_id → home_node; or a DM-specific bootstrap). **This re-opens the
  resolution** — it is no longer "pure membership-apply population"; it needs a resolution source. The
  cheapest may be **carrying the resolved home node in the invite/dm-create content** (the inviter
  resolves it once, at create, from whatever discovery gave them the invitee's identity_id) — but that
  presupposes the discovery surfaces home_node, which today it does not.
- **(iii) harness-scoped + defer production** — ship (iii) populating from the registry (works when
  records are replicated), land MP-C-07 green in the harness, and **route the production
  identity→home gap** as its own finding/arc (the identity-discovery capability is broader than
  MP-F1b). Honest: MP-C-07 converges, but the convergence relies on a pre-established relationship
  that production DMs to strangers won't have.
- **Re-open (ii)/(i)** — only if the augmentation cost is judged too high; J-327 already rejected
  these, but the gate-B finding is new information.

**Recommended next step:** Joe locks the fork above. The audit's code-grounding is decisive on the
*absence*; whether to (a) build the identity→home augmentation now, (b) ship harness-scoped + route
the gap, or (c) reconsider, is the design-phase decision. The empirical two-node witness is built in
the design/impl phase against the chosen path (and needs the un-seeded harness machinery only if the
chosen path claims to work without a pre-established relationship).

| Lock | Status |
|---|---|
| Finding (facet-1 federation_nodes empty) | CONFIRMED |
| (iii) population site (A) | feasible at NodeRuntime membership-apply (§3.1) |
| **Gate-B (B)** | **FAILS production as-is (§4) — the fork-forcing finding** |
| C / D / E | grounded (§5); compose / standing-tension / candidate |
| MP-F4 cross-link | composes, does not reopen (§6) |
| D-076 / D-077 | contingent on gate-B / no regression (§7) |

---

## 9. Entry point (Rule 0)

CLAUDE PLAY → JOURNAL (latest) → `tasks/MP_findings.md` (MP-F1 split / MP-F1b) → this audit (§2 the
authoritative (iii)/A–E, §4 the gate-B verdict) → `tasks/MP_R1_DETERMINISTIC_DESIGN.md` §11 (D10). The
gate-B fork (§8) is the design-phase decision; do not author the design off this audit until Joe locks
the fork.

---

*Per D-065 + D-067 + D-069 + D-071 + D-075 + D-076 + D-077.*
