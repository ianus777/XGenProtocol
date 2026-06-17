# M10.4 — Production Identity→Home-Node Discovery (MP-F13) — Phase-0 Framing Brief
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What M10.4 is

Fourth M10 sub-arc. A **named** sub-arc per the J-358 fork-2 lock — depth decided at this mini-Phase-0,
**not** silently absorbed. It owns **MP-F13** (J-278 / F1B-D5 family): production identity→home-node
discovery. The concrete blocker it must clear is **MP-C-16** (live migration — a Space re-homes to a new
node), which re-runs at **M10.5** once M10.4's disposition lands. Opened J-369 (this brief); no code until
the M10.4 design is Joe-locked.

## 1. The gap (grounded from the findings record)

Two layers, one root:

- **Layer 1 — namespace mismatch (the MP-C-16 RED).** The signed `home_node` field is a **WS URL**
  (e.g. `ws://host:port/xgen`), but `migration_initiate` (and any node-resolution consumer) expects the
  peer's **pubkey `node_id` / NodeXgid**. So a migration target named by its signed `home_node` doesn't
  resolve to a dialable/known node identity — MP-C-16 stays RED on this exact mismatch (J-347).
- **Layer 2 — discovery of a not-yet-known identity (F1B-D5).** Even with the namespace reconciled, when a
  node holds only an identity's **pubkey** and not its `IdentityRecord`, there is **no path** to resolve
  where that identity lives. `build_identity_home_nodes` (runtime.rs:~1895) reads `IdentityRecord.home_node`
  from the **local registry** (registry.rs:~47); a stranger's record isn't there. This is the gap F1b
  routed here (the DM-stranger case, J-332/J-333).

These are related but distinct: Layer 1 is a **field-semantics reconciliation**; Layer 2 is a **distributed
discovery problem** (gossip / directory / DHT / home-node-encoded-in-the-XGID — all heavy, open design space).

## 2. Scope — Joe-LOCKED (J-369)

- **Call 1 — depth = NARROW-FIRST WITH ESCAPE.** The Phase-0 grounds the Layer-1 namespace question
  (§3.1). If reconciling `home_node` (URL ↔ NodeXgid) alone clears MP-C-16, **ship that**; keep
  *discovery-of-unknown-identities* (Layer 2) as a **separately-routed arc**, named and placed on the
  horizon, **not** smuggled into a namespace fix. D-065-honest: if a real discovery dependency is
  load-bearing for MP-C-16 (i.e. the narrow fix can't get there), surface it and re-lock depth.
- **Call 2 — MP-C-16 close condition = AIM CLEAN GREEN.** Migration is node→node where both `node_id`s are
  in principle knowable, so target a clean end-to-end green (migration resolves + the Space re-homes), not a
  green-with-boundary. The re-run lands at M10.5 (per the locked sequence). If grounding shows clean green
  is unreachable without Layer-2 discovery, that is itself the escape-hatch signal for Call 1.
- **Call 3 — `home_node` canonical type = AUDIT-GROUNDED (not locked here).** What `home_node` canonically
  *is* (transport URL vs node identity) is the load-bearing Phase-0 grounding (§3.1). Surfaced back to Joe
  only if it contradicts Call 1.

## 3. What the Phase-0 audit must ground (Clair, D-071)

### 3.1 The `home_node` namespace (load-bearing)
- Where `home_node` is **set** (the signed field on registration; what value goes in — URL or node_id) and
  where it is **read** (`build_identity_home_nodes` runtime.rs:~1895; `IdentityRecord.home_node`
  registry.rs:~47; the migration consumer; the DM-federation populate path; the federation **dial** path).
- The **type expectations at each consumer**: does federation dialing need a URL, does migration need a
  NodeXgid, does DM-federation populate need one or the other? Is there a single canonical type the field
  *should* carry, with a projection to the other where needed (e.g. node_id canonical + a `node_id → url`
  resolution via the federation registry / `record_peer_url`)?
- Whether a reconciliation is **wire-affecting** (does `home_node` cross the wire / persist? a type change
  may need migration) or a **read-side projection only**.

### 3.2 The migration path (the MP-C-16 witness)
- `migration_initiate` (admin_ops; the unfenced aicontrol verb shipped MP-F8/J-347) — exactly what it
  expects for the target node and where MP-C-16's `MIG_6010`-flow reply currently stalls on the mismatch.
- The MP-C-16 harness scenario (`mp_r2_fixed::mp_c_16_live_migration_space_rehomes`) — what a clean-green
  end-to-end re-home requires.

### 3.3 The Layer-2 boundary (escape-hatch grounding)
- Confirm whether MP-C-16's clean green is reachable with Layer-1 reconciliation alone (both node_ids
  knowable in the migration flow) — i.e. that Layer-2 discovery is genuinely *not* required for MP-C-16.
- If a discovery dependency is load-bearing, that is the Call-1 escape signal → surface + re-lock; name the
  Layer-2 arc precisely (the F1B-D5 / DM-stranger sibling) for routing.

### 3.4 D-065 / D-078 discipline
Ground symbol definitions in production code (not inferred from call-sites). If grounding contradicts a
locked call (Call 1 narrow-first, Call 2 clean-green), surface it and re-lock before design.

## 4. Out of scope (recorded boundaries)
- **Layer-2 discovery of unknown identities** (gossip/directory/DHT/XGID-encoded home node) — routed, not
  built, unless §3.3 proves it load-bearing for MP-C-16.
- **MP-C-16 re-run** itself → **M10.5** (per the locked post-M10.3 sequence; M10.5 also folds MP-F6 +
  re-runs MP-C-06).
- The F1b DM-stranger convergence boundary (F1B-D4 harness-green-with-boundary) — stays as recorded unless
  Layer-1 reconciliation incidentally clears it (note if so; don't claim it as a target).

## 5. Sub-arc shape (refined at design-lock)
Mini-Phase-0 audit (ground §3) → design (lock the reconciliation shape + the MP-C-16 witness + the Layer-2
route) → Joe-lock → runbook → Clair impl → Chat doc-bridge → close → M10.5.

## 6. Next-active
**Clair opens the M10.4 D-071 Phase-0 audit** — ground §3.1 (the `home_node` namespace, the canonical type,
the per-consumer expectations) + §3.2 (the migration path / MP-C-16) + §3.3 (the Layer-2 escape grounding)
to file:line → design → Joe-lock → runbook. No code until the M10.4 design is Joe-locked.

**Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-369 → this brief → `tasks/MP_findings.md` (MP-F13 / F1B-D5) →
`tasks/M10_AUTH_MODULE_AUDIT.md`.**
