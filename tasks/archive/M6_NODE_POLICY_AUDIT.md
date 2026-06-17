# M6 node-policy — Backing Audit (the fifth / final D-071 deferral)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this exists

`node-policy` is the **fifth and final M6 deferral** — the 2 A4 verbs `space set-node-policy` / `space show-node-policy`. It is the only deferral that was **never formally audited**: it was folded into the A4-D1 force-eject session as a pointer, not given its own audit phase. This document is that phase — the J-081 / D-071 read-only reality pass ("subsystem audits precede dependent milestones") applied before the design/impl arc opens, exactly as the four shipped verb arcs each ran.

Evidence checked against the live tree on 2026-05-31: `docs/xgen_ch2_architecture.md`, `docs/xgen_ch3_specification.md`, `docs/xgen_node_admin_ops_design.md`, `xgen-core/src/federation/federation_policy.rs`, `xgen-core/src/space/state.rs`.

## The two verbs (as designed at Block 4)

`docs/xgen_node_admin_ops_design.md` §6.A4 (lines 660–671):

| Verb | Class | Sketched shape | Propagation |
|---|---|---|---|
| `space set-node-policy` | WRITE | `{ space_id, policy: NodePolicy }` — *"auto-mute thresholds, rate caps, etc."*; `SPACE_8001` not-hosted / `SPACE_8020` invalid-policy | **none** — "Node-level enforcement layer, separate from the Space governance DAG; no protocol event" |
| `space show-node-policy` | READ | `{ space_id } → { space_id, policy: NodePolicy }`; `SPACE_8001` | none |

Both hosted-Spaces-only (D-082 lock #4 — never federated-in replicas).

## Finding 1 — node-policy has **no protocol-spec backing** (by design)

- The §6.A4 cite *"Spec refs: §2.6.4"* points at the **admin-ops design's own** numbering — §2.6.4 is its *audit-trail-storage* lock (line 124), **not** a ch3 protocol section. ch2 uses prose headers, not `2.6.x`. So the verbs reference no protocol clause.
- **No `NodePolicy` schema exists** in ch2 or ch3. The sketched contents — *"auto-mute thresholds, rate caps"* — match **nothing** in the spec: `grep` for `auto-mute` / `rate cap` / `rate limit` across ch2+ch3 returns **zero hits**. The field set is illustrative invention at Block 4 (the same as-built-delta risk every prior arc hit: `BOOT_70xx→71xx`, `AUTH_20xx→AUTHMOD_61xx`).
- The spec **deliberately** leaves this Node-local. ch3:2404 — the protocol *does not* specify who issues `auto_temperature` moderation; it names "the room's home Node operating an automated moderation policy" only as a *possible* actor and routes the choice to "Space governance … surfaced in Ch6." (Reinforced at ch3:2034 / 2044: `spontaneous_post` enforcement is "client-side and admin-policy, not Node validation.") The one wire-level trace is the hosted-Space registration rejection reason `policy_rejected` (ch3:4012) — present but undefined.

→ **node-policy is intentionally unspecified operator territory.** This is legitimate (the spec pushes moderation policy *out* of the protocol on purpose) — but it means the design phase defines the construct **from near-scratch**, with no spec schema to mirror. Heavier on the *design* side than any prior verb arc.

## Finding 2 — the store is ABSENT; the sibling precedent is clean

- **No `node_policy.rs`** anywhere in `xgen-core/src` (glob `**/*policy*.rs` returns only `federation/federation_policy.rs`); **no `NodePolicy` type**; `state.rs` has no `policy` field. The M6_BACKING_AUDIT "ABSENT" verdict holds.
- **Sibling precedent — `FederationPolicyStore`** (`federation_policy.rs`, 357 lines): `HashMap<NodeXgid, FederationPolicy>` keyed store; `new`/`set`/`get`/`remove`/`all`/`len`/`is_empty`/`save`/`load`; reuses `RegistryError`; pure `policy_permits(Option<&_>, &SpaceXgid)` helper; `Default` = permit-all = the prime-invariant expressed as a value. A node-policy store keyed by `SpaceXgid` is a near-exact structural sibling.
- **Hosted-Space identity** — `SpaceState` (`state.rs:109`) is keyed by `SpaceXgid` and carries `home_node: NodeXgid`. set/show resolve "hosted here" via `home_node == self` (the D-082 lock #4 / `force-eject` authority check already does this at `state.rs:675`).

## Finding 3 — the moderation primitives already exist, but as **Space-DAG governance**, not Node config

The sketch's "auto-mute / rate caps" overlaps a real, *already-built* surface — but on the wrong side of the Node/Space line:

- `SpaceState.human_pacing_ms` / `ai_pacing_ms` (spec 3.7.12.1), `active_mutes`, `member_temperature_visibility` (3.7.13.3) — per-Space **DAG state**.
- `build_membership_mute_event`, `build_space_pacing_event`, `build_space_temperature_visibility_event` (`state.rs`) — emit **Space-governance Events** that federate and live in the DAG.

These are member-facing, owner/moderator-authored, and propagate. A "Node policy" is the opposite: Node-local, operator-authored, **non**-propagating (the §6.A4 sketch is explicit: propagation = none). So node-policy cannot *be* these primitives — at most it could *drive* them (Finding 4, fork Y).

## Finding 4 — the load-bearing design fork (audit routes; design Joe-locks)

Two genuinely different arcs hide behind these 2 verbs. The audit does **not** pick — it surfaces both for the design phase:

- **Fork X — Node-local store, no enforcement consumer (thin).** `NodePolicy` = an operator config blob per hosted Space; the 2 verbs read/write a `SpaceXgid`-keyed sibling store; **nothing reads it yet**. This is the **A2/A3 standalone-store precedent** (auth-module-registry + bootstrap both shipped store+verbs with enforcement deferred to a named future arc). Prime invariant trivially held (no Space behaves differently). Likely ~4 Clair commits, 1 light checkpoint (schema/names). *The honest-minimal reading of "2 verbs."*
- **Fork Y — enforcement-real.** `NodePolicy` *drives* automated moderation: the home Node, per the stored thresholds, emits `auto_temperature` `membership.mute` / pacing Events (ch3:2404). This has a real enforcement site to code-trace (à la federation-policy 2b checkpoint #2 — the pacing/temperature ingest path), couples to the plugin surface, and is materially heavier. *The maximal reading.*

Even before X-vs-Y, the **prior question is what `NodePolicy` contains** — undefined in spec, illustrative in the sketch. That schema is itself the first design Joe-lock.

## What ships, what routes onward

- **Ships in the arc:** `space set-node-policy` + `space show-node-policy` against a new `NodePolicy` store (the A4 row's last 2 ABSENT cells → SHIPPED).
- **Prime invariant:** absent node-policy = today byte-for-byte. Under Fork X, trivial; under Fork Y, the mandatory default-inert regression lands at the enforcement commit (D-065).
- **Routes to design (`tasks/M6_NODE_POLICY_DESIGN.md`):** (1) the `NodePolicy` schema; (2) Fork X vs Y; (3) store shape + on-disk path (sibling: `xgen-node_node_policy.json`); (4) error band (`SPACE_80xx`, harmonised with the shipped A4 `force-eject` codes `SPACE_8001`–`8004`).
- **Not in scope:** `space migrate-as-source` (A4-D2, separate); operating *as* a moderation plugin host (Fork Y's deeper tail, if Y is even chosen).

## Routing / cross-refs

- `tasks/M6_BACKING_AUDIT.md` v1.6 — A4 row: `set/show-node-policy` ABSENT → *node-policy* arc (this audit deepens that row).
- `docs/xgen_node_admin_ops_design.md` §6.A4 — the Block-4 verb sketch (lines 660–671); amended *from* this audit at arc close, with honest as-built deltas (D-065).
- D-071 (subsystem-audits-precede), D-082 lock #4 (hosted-only), D-065 (surface-don't-guess), ch3:2404 (the spec's one Node-moderation gesture).
- Precedent arcs (store+verbs, enforcement deferred): `tasks/M6_AUTH_MODULE_REGISTRY_AUDIT.md`, `tasks/M6_BOOTSTRAP_CLIENT_AUDIT.md`.
- **Next phase (Joe-reserved):** `tasks/M6_NODE_POLICY_DESIGN.md` — lock the schema + Fork X/Y before any runbook.

---

*End of audit.*
