# XGen Protocol — Arc G (Jurisdictional Namespacing, PG-04) Design
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Frame

Design for PG-04, backed by `ARC_G_JURISDICTIONAL_AUDIT.md` v1.0. Forks settled in discussion (2026-06-04): (1) carrier = Space + Node; (2) declared field, not address-namespace; (3) declaration + inert policy hook **(b)**, implementation-side; (4) hard scope fence. Keystone audit finding (AG-A2): the federation gate already exists (`policy_permits`, two-site-enforced, default-permit) — arc G adds a restrictive dimension, not a new chokepoint.

Protocol half is deliberately thin (one Space field + a MAY clause + a MUST-NOT). The behaviour is node policy (implementation). AG-D# are arc-local pending close (D-069).

## §2 — Locked design decisions (AG-D#)

**AG-D1 — Carrier: `SpaceState.jurisdiction: Option<String>`, set-once at create.** Declared in `state.space_create` content; read by the three `SpaceState` constructors (`from_space_create`, `from_dm_space_create`, `from_dm_space_create_node`). **No mutation event, no applier arm, no `state_key_for_event` arm** — a legal domain is fixed at Space birth (a silent post-hoc change is exactly the integrity hazard to avoid). Node-side jurisdiction is **operator config** (AG-D5), not protocol state — so the protocol carrier is Space-only (fork 1 realised: Space declares, Node policy acts).

**AG-D2 — Repr: optional open string.** `Option<String>`, absent ⇒ undeclared. Operator-meaningful (ISO 3166-1 alpha-2 conventionally, e.g. `"SK"`, `"EU"`), but **not enumerated or validated** — sibling to `regulatory_compliance` / `member_temperature_visibility` (open value, preserved verbatim, forward-compat). No `JurisdictionXgid`, no address-prefix (AG-A4 — would break Appendix J §J.5 invariance and touch every flavour).

**AG-D3 — Convergence: rides M8 for free.** Set-once ⇒ identical across every arrival permutation; no conflict class. `SpaceState`'s `PartialEq`/`Eq` (the M8 `derive_resolved` oracle) covers the new field automatically (additive). Add the field to all three constructors; a convergence pin asserts the field survives a permuted rebuild unchanged.

**AG-D4 — Builder: extend the single canonical `build_space_create_event`.** Add `jurisdiction: Option<&str>`; content carries `"jurisdiction"` only when `Some` (absent key = undeclared, mirrors `topic`). One honest builder over a sibling/setter. Cost is mechanical: existing call sites pass `None` (no judgment, just a sweep). **DM Spaces declare no jurisdiction** — `build_dm_space_create_event` unchanged, DM constructors set `None` (jurisdiction is a deployment-level declaration; a 1:1 DM is not a "government deployment").

**AG-D5 — Implementation hook: extend `FederationPolicy`, AND-compose a sibling predicate.** New `FederationPolicy.allowed_jurisdictions: Option<Vec<String>>` — **restrictive-only**, `None` ⇒ no jurisdiction restriction (permit-all; the PRIME INVARIANT preserved). New pure fn `jurisdiction_permits(policy: Option<&FederationPolicy>, space_jurisdiction: Option<&str>) -> bool`, **AND-composed** with the existing `policy_permits` at both enforcement sites — outbound `federation_session.rs:315` and inbound `app.rs:2421`. Keeping `policy_permits`'s signature/tests untouched and adding a single-responsibility sibling is cleaner than widening the existing predicate. Semantics (mirror `allowed_spaces`):
  - `None` allowed_jurisdictions ⇒ `true` (no restriction).
  - `Some(set)` ⇒ `true` iff the Space declares a jurisdiction ∈ `set`. An **undeclared** Space (`None` jurisdiction) under a restrictive set ⇒ `false` (not-in-allow-list, same as a Space absent from `allowed_spaces`).
  - `Deny` mode is already handled by `policy_permits`; the AND-compose means a `Deny` peer is blocked regardless of jurisdiction.
  Honest **default-permit no-op** until an operator sets `allowed_jurisdictions` (the PG-13 pattern — live gate, dormant value).

**AG-D6 — Operator surface: reuse `federation set-policy`.** `allowed_jurisdictions` is additive serde on `FederationPolicy` (`#[serde(skip_serializing_if = "Option::is_none", default)]`, mirrors `allowed_spaces`), so it persists + round-trips through the existing store. Whether the existing admin/AI `federation_set_policy` write path accepts it for free or needs an arg is CP-2; the CLI authoring surface may ride the ops/UI pass if heavy (sibling to prior arcs' "protocol-mechanism now, user authoring later").

**AG-D7 — Protocol-doc (D-074, split across commits).**
  - **C1:** ch3 `state.space_create` schema gains the optional `jurisdiction` field. AppC Space class gains `jurisdiction` (promote from the Phase-3 note; it stays on `AuthModule` too, AE-D5 intact).
  - **C2:** ch3 gains the **MAY** clause — "a Node MAY refuse to host or relay a Space whose declared jurisdiction lies outside the Node operator's federation policy" (the normative authorisation for the C2 hook; ships in the same commit as the behaviour, D-074).
  - **Close:** the §2.2 **MUST-NOT** — promote ch1 L858's implication to a normative ch3 clause prohibiting any central identity-aggregation point "even optionally" (standalone prohibition, not tied to a code path). Exact ch3 section + wording grounded at the editing commit (mirrors Arc E's §3.6.5 error-table close).

**AG-D8 — Scope (reaffirmed from audit §4).** OUT: active data-residency/geo-enforcement (Tier 2–4 institutional, ch3 §3.11); GDPR erasure (PG-02/arc I); identity-level jurisdiction (stays `Tier4Claims`, AE-D5); bridge trust-tier (§2.3). Arc G ships declaration + the MAY-act seam only.

## §3 — Confirm-at-pickup (D-078)

- **CP-1 (C2) — jurisdiction availability + compose shape.** Confirm the Space's declared `jurisdiction` is reachable at both `policy_permits` sites (`app.rs:2421` inbound, `federation_session.rs:315` outbound). The node holds SpaceState; if `jurisdiction` isn't already in scope there, thread it (read from the rehydrated/derived SpaceState for `sid`/`space_id`). Lock the exact `policy_permits(...) && jurisdiction_permits(...)` placement.
- **CP-2 (C2) — `federation set-policy` plumbing.** Confirm whether the existing `admin_ops::federation_set_policy` + AI/`--aicontrol` write path deserialises `allowed_jurisdictions` for free (FederationPolicy is the payload) or needs an explicit arg/verb surface. If heavy, defer the authoring surface to the ops/UI pass and ship the field + enforcement only (the inert-but-correct half).
- **CP-3 (C1/close) — ch3 homes + wording.** Ground the exact ch3 section for the `state.space_create` field schema (C1), the MAY clause (C2), and the central-aggregation MUST-NOT (close). Mirrors the Arc E error-table grounding-at-edit discipline.

## §4 — Commit plan (feeds the runbook)

- **C1 — protocol half (declaration).** `SpaceState.jurisdiction: Option<String>` + all 3 constructors (DM ⇒ `None`) + `build_space_create_event` `jurisdiction` param (call sites → `None`) + ch3 schema field + AppC Space-class field. Tests: declared-at-create reads back, absent ⇒ `None`, DM ⇒ `None`, M8 convergence pin (field survives permuted rebuild).
- **C2 — implementation half (containment hook).** `FederationPolicy.allowed_jurisdictions` (additive serde) + `jurisdiction_permits` + AND-compose at both enforcement sites (CP-1) + operator plumbing (CP-2) + ch3 MAY clause. Tests: default-permit no-op (no `allowed_jurisdictions` ⇒ unchanged behaviour), restrictive set denies an out-of-jurisdiction Space, undeclared Space denied under a restrictive set, permitted Space passes — at both inbound and outbound sites.
- **Close — D-074 doc-only.** ch3 central-aggregation MUST-NOT + AppC reconcile + gap-audit §5 PG-04 ✅ (Open **3/13** — PG-02/05/11) + ROADMAP + JOURNAL + AG-D# promotion eval.

## §5 — Honesty posture (D-065)

Dormant-but-correct, the PG-09/PG-13 family: the jurisdiction field is real and convergence-safe from C1; the federation hook is **live but a no-op** under default (empty) policy, gaining teeth only when an operator declares `allowed_jurisdictions`. Active data residency (geo-pinned storage, legal attestation) is honestly **not** delivered — it is operator/Tier-2+ infra, fenced out (AG-D8). The protocol grants the right to contain (MAY) and forbids central aggregation (MUST-NOT); it does not promise enforcement it cannot keep.

No DECISIONS change proposed (AG-D# arc-local pending close, D-069). Doc-only — suite unchanged at J-248's 1107/0/2, not re-run.
