# M6 node-policy — Design (Joe-locked decisions)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Design phase of the *node-policy* arc — the fifth and final M6 deferral (2 verbs: `space set-node-policy` / `space show-node-policy`). Opens on the reality map in `tasks/M6_NODE_POLICY_AUDIT.md`. Decisions are locked **one by one** with Joe; this doc fills in as they close. Arc-local `NP-D#` decisions live here (D-069); none graduate to DECISIONS.md unless global.

This is a discuss-before-code arc: no runbook / code until the open decisions close.

## Decision log

### NP-D1 — Authority boundary — **LOCKED 2026-05-31**

node-policy is set by the **Node operator / admin** (principal #1: OS-user-equals-administrator, the `--batch` runtime; the same authority that signs `force-eject`) and governs the **home Node's own behavior as host/actor**, per hosted Space, **non-propagating**.

**Off-limits** — node-policy does **not** touch:
- **Space governance** (the *owner* / Role model, principal #3): roles, mutes, pacing, display thresholds — DAG-carried, federated, owner-authored.
- **AI-operator delegation** (principal #2: the member-level human accountable for an AI Identity — `SpaceState.ai_operator_delegations`, `state.ai_operator_delegate`/`revoke`) — DAG-carried member state.

**Three principals stay distinct** (one human may hold all three hats; the authorities never merge):
1. **Node operator / admin** — infrastructure; Node-signed; node-policy's principal.
2. **AI-operator** — member-level AI accountability; governance/DAG.
3. **Owner** — Space governance; Role model/DAG.

**Rationale.** Preserves "hosts-but-doesn't-own": the operator sets how *their machine* behaves, never how the *community* is governed — otherwise the Node either silently overrides owner governance (the capture this project is designed against) or creates two sources of truth for one value. **The Node/Space line is the propagate/don't-propagate line:** operational posture is legitimately per-Node (each federated Node may have different resource limits / legal obligations) so it stays local; governance must be federation-consistent so it stays on the DAG. This non-propagation falls out of the boundary for free, and extends the `force-eject` precedent (A4-D1: Node-admin is a distinct first-class authority, not a masqueraded member role).

**Note (naming hazard recorded, D-065).** ch3:2714 calls the `auto_temperature` signer "the home Node's *operator Identity* acting as an automated moderation agent" — "operator" there = principal #1 (Node), brushing against #2 (AI-operator). node-policy binds to #1 only.

### NP-D2 — The `NodePolicy` schema (smallest cluster) — **LOCKED 2026-05-31**

`NodePolicy` v1 is exactly two fields — the home Node's *automated-action posture* for one hosted Space:

```
NodePolicy {
    auto_moderation: bool,          // does THIS Node auto-act for this Space; default false
    action_threshold: Option<f64>,  // [0.0, 1.0] actionable trigger; Some() only meaningful when enabled
}
```

This fills exactly the **3.7.13.6 "actionable threshold" gap** — *when this Node would fire* `auto_temperature` — which the spec currently leaves as the plugin's private choice with no operator surface. It is **distinct from** the owner's *display* thresholds (`temperature_thresholds` on Room metadata, governance/DAG — off-limits per NP-D1).

**Deliberately excluded from v1** (D-065 — no consumer, no spec home, collision risk): `cooldown_override` (3.7.13.6 already gives Ch6 defaults — 2h kick / 15m mute; a second-order decision), and any `rate_cap` / `storage_quota` (would collide with the existing Space-state `max_event_size`; the Block-4 "rate caps, etc." was illustrative invention). The struct is `#[serde(default)]`-extensible — these join the day a real consumer needs them, not speculatively.

**Absent == disabled.** A missing policy and `{ auto_moderation: false, action_threshold: None }` are treated identically (no distinction between "operator hasn't spoken" and "operator said don't"). `Default` = `{ false, None }` = today byte-for-byte, mirroring the federation-policy precedent (`Default` = permit-all = absent). This is the prime-invariant expressed as a value.

### NP-D3 — Fork X (store-only; enforcement deferred) — **LOCKED 2026-05-31**

node-policy ships as an **inert stored value**: `set-node-policy` writes it, `show-node-policy` reads it back, and **nothing in the running Node reads it** this arc. The live consumer that would fire `auto_temperature` at `action_threshold` arrives with the **temperature-plugin arc**, not here.

**Why X, not Y.** The temperature plugin is still a no-op trait (A7, unbuilt) — there is no live consumer to enforce against. Building one now would pull the unbuilt plugin into a 2-verb admin arc (scope explosion + dependency on absent infrastructure). This is the **store-before-consumer** pattern auth-module-registry (A2) and bootstrap-client (A3) both shipped. The NP-D2 schema is **Y-shaped but X-delivered**: forward-compatible with enforcement without requiring it.

**Prime invariant — trivially held.** With no reader and `auto_moderation` defaulting false, no Space behaves differently than today. (Under a future Y arc the mandatory default-inert regression would land at the enforcement commit, D-065.)

### NP-D4 — Per-Space only (no Node-wide default) — **LOCKED 2026-05-31**

node-policy is stored and resolved **per hosted Space** (keyed by `SpaceXgid`); there is **no Node-wide default** posture in v1. A hosted Space without its own entry resolves to absent == disabled (NP-D2) = today.

**Rationale.** Under Fork X the store has no live reader, so a second resolution path (Space-entry → Node-default → hardcoded `{false,None}`) has nothing to exercise it — premature. NP-D2's absent==disabled already supplies a safe fallback. A blanket "apply to all my hosted Spaces" default is operator-ergonomics, not correctness; the `#[serde(default)]`-extensible store grows it later without rework. Defer the Node-wide default to the temperature-plugin arc, where a live consumer makes the convenience-vs-complexity tradeoff concrete. This pins the store to the cleanest shape: `HashMap<SpaceXgid, NodePolicy>` — the `FederationPolicyStore` sibling minus the default-fallback wrinkle.

### NP-D5 — Two modes of one Node-admin authority — **LOCKED 2026-05-31**

node-policy and `force-eject`/`unban` are **two modes of a single Node-admin authority** (principal #1, Node-signed), split by one test — *does the action change shared Space state?*

- **Intervention** (`force-eject`/`unban`): changes shared state (a member removed/restored) → emits a DAG event (`membership.node_eject`/`node_unban`), federates. One-shot, manual.
- **Standing posture** (node-policy): how the home Node itself behaves → Node-local, non-propagating. Inert today (Fork X).

Once the temperature-plugin arc lands, the model reads as one axis: *manual intervention* (`force-eject`) · *standing posture* (node-policy) · *automated intervention* (enforced node-policy firing `auto_temperature` per its posture). node-policy is the standing-rules layer; force-eject the manual one-shot; enforced node-policy the rules firing themselves.

**No merged surface.** `show-node-policy` shows posture only; it does **not** surface eject history (force-eject already correlates via the audit log / `event_id`). Coherence is conceptual; the only concrete shared element is the `SPACE_80xx` code band (NP-D6).

### NP-D6 — Error band — **LOCKED 2026-05-31**

Both verbs slot into the existing `SPACE_80xx` band, contiguous with the shipped force-eject family (`SPACE_8001`–`8004`):
- `SPACE_8001` — Space not hosted here (**reuse**; both verbs need it).
- `SPACE_8005` — invalid policy (`action_threshold` outside `[0.0, 1.0]`) — the only new code; `set-node-policy` only.

Drops the Block-4-guessed `SPACE_8020` (left a gap in the band for no reason). `show-node-policy` needs only `8001`.

## Design status

**All decisions locked (NP-D1–D6).** The data layer is fully pinned by the locks (schema, `HashMap<SpaceXgid, NodePolicy>` store, on-disk `xgen-node_node_policy.json`, `SPACE_8005`), so no mid-runbook data-layer checkpoint is required. Fork X means **no enforcement seam to code-trace** — the risk that justified checkpoints in the 2a/2b/A2/A3 arcs is absent here. Ready to feed the impl runbook (`tasks/M6_NODE_POLICY_IMPL.md`); this doc → COMPLETED at arc close.

## Cross-refs

- `tasks/M6_NODE_POLICY_AUDIT.md` (v1.0) — the reality map this design answers.
- A4-D1 (`force-eject` = distinct first-class Node-admin authority), D-082 lock #4 (hosted-only), D-069 (arc-local decisions), D-065 (surface-don't-guess), A2/A3 (store-before-consumer precedent).
- ch3:2404 / 2714 / 3.7.13.6 (the spec's "home Node automated moderation policy" gesture + the actionable-threshold gap), federation-policy (`Default`-as-prime-invariant sibling).

---

*Design complete — NP-D1–D6 locked. Next: impl runbook (Clair, Phase 2).*
