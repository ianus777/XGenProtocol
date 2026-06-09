# MP-F1b — cross-node DM convergence (membership-driven DM federation) — DESIGN

> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-09  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The design for **MP-F1b** on the **Option-2 lock (Joe, J-332)**: ship (iii) membership-driven DM
federation **harness-scoped**, with the gate-B production gap **routed** to a named discovery arc, not
built around. Executes the authoritative (iii)/A–E (re-supplied verbatim from J-327) +
`tasks/MP_F1B_DM_FEDERATION_AUDIT.md` (Phase-0, commit `bfa0535`, the gate-B verdict).

Phase-0's verdict stands and is *why* Option 2 is right: gate-B fails the production case (no
production path resolves a not-yet-replicated stranger's `home_node`), there is **no small
augmentation** (so option 1 would explode scope in the wrong container), and (i)/(ii) either discard
correct work or breach 3.16.1. So: ship the correct, composable half (sub-lock A population), record
MP-C-07 cross-node with an honest boundary, and route the discovery gap as its own arc.

**Three honesty conditions (Joe, part of the arc — not optional):** (1) ship sub-lock A's
membership-apply population; (2) record MP-C-07 cross-node honestly (boundary, not a bare ✅); (3) the
discovery gap is its own named arc. Locked as F1B-D1 / F1B-D4 / F1B-D5 below.

Authored before the runbook; locked with Joe first (D-071).

---

## 2. The lock (Option 2)

> **Gate-B fork LOCKED (Joe, J-332): Option 2 — (iii) harness-scoped + route the discovery gap.**

(iii) is correct *given resolved identities*; gate-B is the gap between "resolved" and "discoverable."
The DM-federation-set derivation (members' home nodes) is the right consumer of resolved identities
**no matter how discovery is later solved** — the discovery arc changes *how identities enter the
registry*, not *how the DM federation set is derived from known ones*. So building sub-lock A now is
not throwaway work; it is the half that stays.

---

## 3. Grounding (live `main`) — the three load-bearing points

### 3.1 The population site (sub-lock A + Joe condition 1)

- `SpaceState::apply_join`/`apply_invite`/… are **pure** — no `IdentityRegistry`, cannot resolve a
  member → home node. So population is **not** in `SpaceState` (Joe condition 1).
- `apply_federation_add` rejects for DMs (`DmFederationNotAllowed`, state.rs:660) — **stays intact**;
  population is **not** there (Joe condition 1). No third-party node ever receives DM content.
- The registry-aware layer is `NodeRuntime`, which already maps `identity_id → home_node` via
  `build_identity_home_nodes(&self.identity_registry)` (runtime.rs:1895; `IdentityRecord.home_node`,
  registry.rs:47). ⇒ **the hook is a NodeRuntime post-membership-apply step** that reads the **resolved**
  member set and the registry.
- **Rebuild resets it:** `derive_resolved` builds a fresh `SpaceState` whose DM `federation_nodes`
  starts empty (the constructors set `Vec::new()`; `apply_federation_add` is rejected). So the hook
  must re-fire after **both** apply paths in `ingest_event` (the create/`derive_resolved` rebuild arm
  **and** the incremental `_ => apply_event` arm), and on cold-start `rehydrate_space_from_store`. A
  **single idempotent helper** called from those sites (no per-site logic → no D-067 drift); the exact
  site list is a runbook item.

### 3.2 The send path consumes `federation_nodes` (no other change needed)

`derive_event_nodes` (fanout.rs:178) = `home_node` + `federation_nodes` (+ sources);
`apply_federation_push` (federation_session.rs:247) early-returns on the empty set and already carries
the F-5 origin skip + per-peer policy filter. So **the moment `federation_nodes` is populated, the
existing push path federates DM events both directions** — no `DmFederationNotAllowed` change, no new
send code.

### 3.3 `federation_nodes` is in the convergence oracle → use the full members' set

`SpaceState` `#[derive(… PartialEq, Eq)]` (state.rs:185) **includes** `federation_nodes` (field at
:234). Within a node the helper is deterministic (resolved members × registry) ⇒ `assert_converges`
safe. Across nodes, the **full members'-home set (self-included)** is computed *identically* on both
nodes (A and B both derive `{home(A-member), home(B-member)}`), so it does not introduce a cross-node
`SpaceState` asymmetry; the push path skips self (a node never push-delivers to its own id). (Per
MP-R1-D7 the cross-node mptest oracle already excludes federation-infra asymmetry and asserts on the
event-id set + membership, so this is belt-and-braces — but the symmetric set is the clean choice and
matches invariant E literally.)

---

## 4. Locked decisions (F1B-D1..D7)

### F1B-D1 — population site: a NodeRuntime post-membership-apply helper (condition 1)
A single idempotent helper (working name `repopulate_dm_federation_nodes(&mut SpaceState, &IdentityRegistry)`)
that, for a **DM Space** (`dm_constraints_active`) only, sets `federation_nodes` = the current
**resolved** members' home nodes from the registry. Called from the `NodeRuntime` apply sites after the
state is derived/applied (ingest create arm + incremental arm + rehydrate; runbook enumerates).
**Not** `SpaceState` (pure), **not** `apply_federation_add` (`DmFederationNotAllowed` intact).
*Considered + rejected:* placing it inside `derive_resolved` (it has `ihn`) — rejected to honour Joe's
NodeRuntime/`build_identity_home_nodes` scoping and to avoid giving the client's empty-`ihn` derive
(R2-F01) DM-federation semantics it has no use for.

### F1B-D2 — the set = members' home nodes (invariant E), full set
`federation_nodes = { home_node(m) : m ∈ resolved members, resolvable }` — the full set per invariant
E ("exactly its members' home nodes"), self-included for cross-node symmetry (§3.3); the push path
skips self (confirm-at-impl that `apply_federation_push`/`derive_event_nodes` self-exclude — they do
not deliver to the local id).

### F1B-D3 — the gate-B boundary lives in code as omission (condition 2, honest-by-construction)
A member whose `home_node` is **not resolvable** (record not in this node's registry) is **omitted**
from `federation_nodes` — no crash, no guess, no fabricated home. That omission **is** the
harness/production boundary: resolvable (harness, G-6-seeded) → federates; unresolvable (production
stranger) → that node simply doesn't receive the DM, deferred behind the discovery arc (F1B-D5). The
code federates to exactly whom it can resolve — never more.

### F1B-D4 — MP-C-07 cross-node recorded with a boundary, NOT a bare ✅ (condition 2)
`MP-C-07` (`mp_r1_c4`) recorded outcome:
> "DM federation forms correctly when members' home nodes are resolvable; **harness-witnessed**;
> production convergence to a not-yet-known counterparty **DEFERRED** behind the identity→home
> discovery arc (F1B-D5)."

Plus: the un-seeded production case is **not expressible on current harness rails** (G-6 always
pre-establishes federation + replication; the manifest pre-shares `{{identity_id}}`) — sibling to
MP-A-01(ii) PENDING — so **no empirical production witness is claimed**. A clean green here would be
the misleading-positive the test-integrity principle forbids.

### F1B-D5 — the discovery gap is its own named arc (condition 3)
Working name **"production identity→home-node discovery"** — the identity-resolution sibling to
M9.2's still-open "real peer discovery" question. Genuinely bigger than F1b (no-anonymity identity
verification + federation discovery, founding-philosophy territory) → its own arc/milestone in the
discovery space, **NOT** folded into loop-to-green. **F1b routes to it; F1b does not build it.**

### F1B-D6 — close-criterion amendment (MP-R1-D10), blessed
MP-R1-D10's "all-green-except-MP-C-06" now reads: **all-green-except-MP-C-06, with MP-C-07-cross-node
harness-green-with-boundary (production convergence deferred behind the discovery arc).** Carried here
as the MP-R1-D10 amendment (blessed like D8/D10); the doc-bridge mirrors it into
`MP_R1_DETERMINISTIC_DESIGN.md` §11.

### F1B-D7 — sub-locks C / D / E
- **C (leave / federated-RTBF):** the helper recomputes the full set at each membership-apply, so a
  **leave shrinks** `federation_nodes` for future events naturally. The historical local copy on a
  departed member's node is the **standing federated-RTBF tension** — acknowledged, **NOT solved
  in-arc**.
- **D (create vs ongoing):** `dm_space_create` keeps riding the G-6 handshake catch-up; subsequent
  events use the populated set; MP-F3's `store.contains` dedup covers the overlap.
- **E (invariant):** "a DM's federation set is exactly its members' home nodes; no other node receives
  DM content." DECISIONS candidate — **promote per D-069 once it holds across the arc** (at close).

---

## 5. Proof plan

- **Unit (xgen-node, NodeRuntime-level):** after a DM membership-apply, `federation_nodes` =
  resolvable members' home nodes; an **unresolvable** member is **omitted** (the F1B-D3 boundary, the
  honest-by-construction proof); a regular Space's `federation_nodes` is **unchanged** (DM-only helper);
  a leave **shrinks** the set (F1B-D7 C).
- **Witness — MP-C-07 cross-node (`mp_r1_c4`) flips KNOWN-FAIL → harness-green-with-boundary:** both
  DM messages converge A↔B in the harness (where G-6 seeds resolution); the recorded outcome carries
  the F1B-D4 boundary text. **RED-on-revert:** revert the population helper → `federation_nodes` stays
  empty → `apply_federation_push` early-returns → DM doesn't federate → MP-C-07 RED. Genuine, not vacuous.
- **No production witness** — recorded honestly per F1B-D4 (not expressible on current rails).
- **D-076 net:** `assert_converges` + the 285 integration + M8 net stay green (DM-only helper; regular
  membership resolution untouched).
- Build 0 + clippy clean (default + `--features harness-control`).

---

## 6. D-076 / D-077 / MP-F4 composition

- **D-076:** `federation_nodes` is a **derived projection** of (resolved members × registry) — the
  D-075 vantage-aware shape (J-250: relationship-shaped fields are derived projections). Within a node
  deterministic; the full-set choice (F1B-D2) keeps it cross-node symmetric where registries agree;
  where they differ (gate-B) the mptest oracle tolerates federation-infra asymmetry (MP-R1-D7). The
  helper reads the **resolved** member set (post-`derive_resolved`/apply) → no ordering surface added.
- **D-077:** `DmFederationNotAllowed` intact (no privacy-containment regression); regular-Space
  federation untouched (DM-only). Set-shrink-on-leave is new behaviour, not a regression.
- **MP-F4 composition (cross-link, confirmed):** F1b reads **Space** membership (`state.members`),
  orthogonal to F4's room-scoped membership `state_key`s; F1b adds **no** `state_key`. The helper fires
  **after** `derive_resolved`/apply, so it reads F4-correct space-membership. Composes; does not reopen
  the shipped frontier-anchor + A1.

---

## 7. Scope fence + honest boundary (D-065)

- **In scope:** sub-lock A's NodeRuntime population helper (F1B-D1/D2/D3) + the MP-C-07 witness flip
  with boundary (F1B-D4) + the MP-R1-D10 amendment (F1B-D6).
- **Out of scope, routed:** production identity→home-node **discovery** (F1B-D5) — its own arc; F1b does
  not build it. The federated-RTBF historical-copy tension (F1B-D7 C) — acknowledged, not solved.
- **Honest boundary:** MP-C-07 converges **in the harness** because G-6 pre-seeds resolution; a
  production DM to a not-yet-known counterparty does **not** converge until the discovery arc lands.
  This is recorded, witnessed-with-boundary, and **not** dressed as a clean cross-node ✅.
- **No DECISIONS change in-arc** (E is a candidate; promote at close per D-069).

---

## 8. Entry point (Rule 0)

CLAUDE PLAY → JOURNAL J-332 (Phase-0 open + gate-B verdict + Option-2 lock + the MP-R1-D10 amendment)
→ `tasks/MP_F1B_DM_FEDERATION_AUDIT.md` (§2 authoritative (iii)/A–E, §4 gate-B verdict) → this design
→ (after Joe-lock) `tasks/MP_F1B_DM_FEDERATION_IMPL.md` (runbook). The population helper (F1B-D1) + the
honest MP-C-07 recording (F1B-D4) are the runbook's deliverables; the discovery arc (F1B-D5) is routed,
not built.

---

*Per D-065 + D-067 + D-069 + D-071 + D-075 + D-076 + D-077.*
