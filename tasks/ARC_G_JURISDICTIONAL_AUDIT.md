# XGen Protocol — Arc G (Jurisdictional Namespacing, PG-04) Phase-0 Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Scope and the D-071 gate

Arc G is the Round-1 D-071 Phase-0 audit for **PG-04** (federation jurisdictional namespacing), selected after Arc E closed (J-248). It closes the loop Arc E left open: TrustAssertion's `jurisdiction` field was reversed out (AE-D5) and explicitly homed here; AppC marks jurisdiction "Phase-3 / PG-04 / arc G". PG-04 is the **actionable half** of the §2.2 banked tension (no-anonymity vs. government identity demands); the other half is a spec-prohibition clause (this audit, §5).

This is grounding only — no design locks (AG-D# land in DESIGN), no code.

## §2 — Spec anchors (what PG-04 demands)

- **ch1 L727 (Jurisdictional):** "The federation model must support jurisdictional namespacing — a government deployment must be able to enforce local data residency rules without forking the protocol."
- **ch1 L856/L858 (Tension 1):** the protocol spec must **explicitly prohibit** any design pattern that allows a central identity-aggregation point to exist — "even optionally." Federation is the structural defense; a government subpoenaing *a node* gets only that node's records.
- **ch3 §3.11 L3580–3880:** Tier 2–4 data-localisation obligations are N/A-institutional but "tie to PG-04" — arc G is the substrate those obligations later read, not their implementation.

**Reading.** "namespacing" + "without forking" + "federation model" ⇒ the demand is a **declared legal domain that partitions hosting/federation policy**, not a per-identity attribute and not a re-architecture of addressing. "enforce local data residency" cannot mean active geo-enforcement at the protocol layer — XGen has no central anything; real residency needs operator infra (Tier 2–4). The honest protocol reading is **declaration + a MAY-act federation right**, with enforcement living in node policy.

## §3 — As-built findings (AG-A#)

**AG-A1 (GAP-CONFIRMED — the field is identity-only today).** `jurisdiction` exists solely as a `Tier4Claims: String` (`xgen-core/src/auth/tiers.rs:114`) — a claim *about an identity*, scoped to Tier-4 verification. A workspace grep (`xgen-common`/`xgen-core`/`xgen-node`, `*.rs`) finds **no** jurisdiction at Space, Node, or federation-addressing level. Nothing partitions federation by jurisdiction. GAP-CONFIRMED.

**AG-A2 (the chokepoint already exists — the keystone finding).** The federation path already has a pure, two-site-enforced policy gate: `xgen-core/src/federation/federation_policy.rs` — `FederationPolicy { mode: PolicyMode, allowed_spaces: Option<Vec<SpaceXgid>> }` + `policy_permits(policy, space_id) -> bool` (`:153`). It is **per-peer**, **restrictive-only** (narrows the protocol-derived shared set, never widens), and **default-permit** (no stored policy ⇒ `true`, the "PRIME INVARIANT", byte-for-byte as today). It is enforced at **both** federation sites — outbound `apply_federation_push` (`federation_session.rs:247`) and the inbound federated-event ingest gate (FAC-D3). Operator surface already ships: `federation_set_policy` / `federation_show_policy` (`admin_ops.rs:1845/1943`), the M6 AI/admin write path.

  *Consequence:* fork-3b's "inert policy hook" is **not a new gate** — it is one new restrictive-only dimension on the existing `FederationPolicy` (an `allowed_jurisdictions`-shaped narrowing), consulted by the existing `policy_permits` seam against a Space's newly-declared jurisdiction. This mirrors SR-F1 / Arc-D: the seam is built and wired; arc G adds a dimension, not a chokepoint. Default-permit honesty is already established and battle-tested.

**AG-A3 (the carrier — Space, set at create).** A Space is the unit that declares a legal domain (ch1's "government deployment"). `SpaceState` is the home for a declared `jurisdiction`. Node-side jurisdiction is **operator config**, not protocol state, and already has a natural home in the federation-policy store (AG-A2) — so the protocol carrier is Space-only; "Node carries jurisdiction" (fork 1) is realised as operator policy, not a wire field. Set-once at Space create (a legal domain should not silently mutate post-hoc) — to be locked in DESIGN.

**AG-A4 (repr — declared field, not address namespace; fork 2 confirmed).** A true address-level namespace (`xgen://sk.space/…`) would break XGID invariance (Appendix J §J.5 — every flavour serialises as the same plain string) and touch every flavour. A declared `jurisdiction` field on the Space satisfies "namespacing" semantically (a declared domain that partitions federation policy) with **no XGID/wire-format re-architecture**. Confirmed: metadata field, not address-prefix.

**AG-A5 (convergence — rides M8).** A declared, set-once `jurisdiction` on `SpaceState` participates in the M8 convergence oracle (SpaceState `PartialEq`/`Eq`). Set-at-create ⇒ no concurrent-mutation conflict class; carried by the create event, no new state-key. To confirm in DESIGN against the create path (mirrors `space_local_metadata`).

## §4 — Scope fence (named homes, STOP on drift)

- **OUT — active data residency / geo-enforcement:** operator-infra, Tier 2–4 institutional (ch3 §3.11). Arc G ships the declaration + the MAY-act seam only.
- **OUT — GDPR erasure (PG-02 / arc I):** jurisdiction is a substrate erasure may later read; arc G does not touch deletion.
- **OUT — identity-level jurisdiction:** stays the `Tier4Claims` field where it already lives; not re-homed onto the SignedPrimitive (AE-D5 stands).
- **OUT — bridge trust-tier / §2.3:** unrelated future module.

## §5 — The spec-prohibition clause (§2.2 other half)

PG-04 closes the §2.2 tension only *with* an explicit normative prohibition of central identity aggregation (ch1 L858 is an *implication*, not yet a normative MUST-NOT). Audit task: confirm whether ch3 already states this explicitly; if not, add a MUST-NOT clause (likely ch3 federation/identity section) promoting ch1's implication to normative text. This is cheap doc-work and is the philosophically load-bearing half — it rides the arc, gated to DESIGN for exact home + wording.

## §6 — Protocol vs. implementation split (per the discussion)

- **Protocol (thin, normative):** (1) Space `jurisdiction` declared field — ch3 + AppC; (2) a **MAY** clause ("a node MAY refuse to host/relay a Space outside its jurisdiction policy" — a federated protocol cannot compel a sovereign node's hosting decisions); (3) the §5 central-aggregation **MUST-NOT**.
- **Implementation (node policy):** the `allowed_jurisdictions` dimension on `FederationPolicy` + `policy_permits` consulting it, inert-by-default (no jurisdiction restriction ⇒ permit-all, the prime invariant). Operator config via the existing `federation set-policy` surface. Honest no-op until an operator configures it (PG-13 pattern).

## §7 — Verdict

PG-04 = **GAP-CONFIRMED** (AG-A1). Resolvable as a **small protocol half** (one Space field + a MAY clause + a MUST-NOT) over a **pre-built implementation seam** (AG-A2 — extend `FederationPolicy`/`policy_permits`, not a new gate). Forks 1–4 hold; the keystone finding (AG-A2) de-risks fork-3b from "build a hook" to "add a dimension". Ready for DESIGN (AG-D# locks): carrier + set-once semantics (AG-A3), field repr + convergence (AG-A4/A5), the policy-dimension shape + default-permit invariant (AG-A2), spec-clause home + wording (§5), MAY-clause home (§6).

No DECISIONS change proposed at audit stage (AG-D# arc-local pending DESIGN, D-069). Doc-only — suite unchanged at J-248's 1107/0/2, not re-run.
