# M6 node-policy — Implementation Runbook (Clair)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Implementation runbook for the *node-policy* arc — the fifth and final M6 deferral (`space set-node-policy` / `space show-node-policy`). Executes the locked design in `tasks/M6_NODE_POLICY_DESIGN.md` (NP-D1–D6). Fork X (inert store, no enforcement) makes this the smallest verb arc: **2 commits, no Joe-lock checkpoint** — the design fully pins the data layer and there is no enforcement seam to code-trace.

**Entry point (Rule 0):** CLAUDE.md PLAY block → latest JOURNAL entry → this runbook → the design doc. Then proceed to C1 directly (no checkpoint gates C1).

## Locked design (the spec for this runbook)

- **`NodePolicy { auto_moderation: bool, action_threshold: Option<f64> }`** (NP-D2). `Default` = `{ false, None }` = today byte-for-byte. `action_threshold` is `[0.0, 1.0]`, only meaningful when `auto_moderation`.
- **Store: `NodePolicyStore { policies: HashMap<SpaceXgid, NodePolicy> }`** (NP-D4) — `FederationPolicyStore` sibling, no default-fallback. On-disk `xgen-node_node_policy.json`.
- **Fork X** (NP-D3): store-only; **nothing in the running Node reads it** this arc. The verbs are the *sole* consumer.
- **Authority** (NP-D1): Node-operator only; hosted-Spaces-only (`home_node == self`); non-propagating; touches no governance / AI-operator state.
- **Codes** (NP-D6): reuse `SPACE_8001` (not hosted here); new `SPACE_8005` (invalid policy — `action_threshold` outside `[0.0, 1.0]`).
- **Absent == disabled** (NP-D2): missing entry and `{ false, None }` are indistinguishable; `show` on an unset hosted Space returns the default.

## Confirm-at-pickup (D-078 — verify against the live tree, don't guess)

1. **Store location** — proposed `xgen-core/src/space/node_policy.rs` (it is `SpaceXgid`-keyed, so the `space/` module is the natural home, sibling to `state.rs`). `federation_policy.rs` lives in `federation/` only because it is `NodeXgid`-keyed. Confirm `space/` reads right; if not, `federation/` is the fallback. Declare in the chosen `mod.rs`.
2. **`RegistryError` import path** — `federation_policy.rs` uses `use super::registry::RegistryError;`. From `space/` the path differs (`crate::federation::registry::RegistryError`). Confirm the canonical re-export and reuse it (no new error type — D-067).
3. **Hosted-here check** — reuse the `force-eject` authority pattern (`event.sender == self.home_node`, `state.rs:~675`); confirm the exact accessor the admin surface already uses for "is this Space hosted by me."
4. **`set` arg style** — `set-node-policy <space_id> [--auto-moderation <bool>] [--action-threshold <f64>]` is a **full set** (mirrors `federation set-policy`, not a partial patch): omitted `--auto-moderation` → `false`, omitted `--action-threshold` → `None`. Confirm the clap bool-value style against existing verbs.

---

## C1 — store + both verbs + threading + tests

**xgen-core:**
- NEW `space/node_policy.rs` (confirm-at-pickup #1): `NodePolicy` (`#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]`) + `NodePolicyStore` (`#[derive(Debug, Default, Serialize, Deserialize)]`, `policies: HashMap<SpaceXgid, NodePolicy>`). Store API mirrors `FederationPolicyStore`: `new` / `set(SpaceXgid, NodePolicy)` insert-or-replace / `get(&SpaceXgid) -> Option<&NodePolicy>` / `remove` / `all` / `len` / `is_empty` / `save(&Path)` / `load(&Path)`; reuse `RegistryError` (confirm-at-pickup #2). The store reports absence faithfully — **absent == disabled lives in the verb/`Default`, not a store-side fallback** (NP-D4, sibling to the federation-policy split). Declare `pub mod node_policy;` in the chosen `mod.rs`.

**xgen-node `admin_ops`:**
- `node_set_policy` (WRITE, **audited**): hosted-here else `SPACE_8001`; if `action_threshold` is `Some(x)` and `x ∉ [0.0, 1.0]` → `SPACE_8005` (pre-audit, pre-write); build the full `NodePolicy`, `set` + `save`. Result `{ space_id, policy }`. `args_hash` over `{ space_id, auto_moderation, action_threshold }`.
- `node_show_policy` (READ, not audited): hosted-here else `SPACE_8001`; return the stored policy or `NodePolicy::default()`. Result `{ space_id, policy }`.
- New code `SPACE_8005` (invalid policy); reuse `SPACE_8001`.
- `AdminContext` gains `node_policy_store: Option<Arc<Mutex<NodePolicyStore>>>` + `with_node_policy_store` + `require_node_policy_store` + `node_policy_store_path()` (`xgen-node_node_policy.json`). Sibling to the A1/A2/A3 threading (D-067).

**xgen-node wiring:**
- clap: `space` `Subcommand` gains `SetNodePolicy` / `ShowNodePolicy` (§2.6.6 two-token naming).
- Threading `run_node → start_pipe_server → dispatch_line → dispatch_admin` + 2 pipe arms + pipe help string.
- Load the store **inside the `#[cfg(windows)]` pipe block** — the verbs are the sole consumer (Fork X; sibling to bootstrap-client C3 / auth-module C3). The non-pipe `run_node` path is unchanged.

**Tests:**
- xgen-core store: set/get/remove roundtrip; save/load roundtrip; `NodePolicy::default() == { false, None }`; `get` on an unset key → `None`.
- xgen-node verbs: set-then-show + persist; show-unset-hosted → default; set bad `action_threshold` → `SPACE_8005`; set/show on a non-hosted Space → `SPACE_8001`.
- **Prime-invariant regression (mandatory, D-065):** empty store + `show` on a hosted Space returns the default `{ false, None }` and touches nothing else; no Space behaves differently than today; the non-pipe path is unchanged.

**Verification (Rule 2 — paste real output):** `cargo test --workspace` (expect ~+8–10); `cargo build --workspace --all-targets` (0/0); `cargo clippy --workspace --lib --tests --all-features -- -D warnings` (clean). Because the verbs *are* the only consumer, the threaded Arc is read by them — no unused-Arc clippy trip.

**C1 DoD:** store + 2 verbs + threading shipped; `SPACE_8005` added; prime-invariant regression green; suite green; build + clippy clean; per-file `git add`; JOURNAL entry; CLAUDE PLAY → C2 next.

---

## C2 — doc-only close (D-074 atomic)

No code. Same-commit atomic close including JOURNAL.md.

- `docs/xgen_node_admin_ops_design.md` §6.A4: SHIPPED banner on `set-node-policy` + `show-node-policy` with **honest as-built deltas** (D-065): the 2-field `{ auto_moderation, action_threshold }` schema (vs the Block-4 "auto-mute thresholds, rate caps, etc."); `SPACE_8005` superseding the guessed `SPACE_8020`; **Fork X — stored, inert, no enforcement this arc**; propagation = none (confirmed). §5.1 Phase-9 line + category row.
- `tasks/M6_BACKING_AUDIT.md`: A4 rows `set-node-policy` / `show-node-policy` ABSENT → **SHIPPED ✅**; summary + consequence sections note **all M6 deferrals now closed** (the four D-071 verb arcs + node-policy) → next is **M7 `--aicontrol`**.
- `tasks/M6_NODE_POLICY_AUDIT.md` + `tasks/M6_NODE_POLICY_DESIGN.md` → **COMPLETED**.
- `docs/ROADMAP.md`: A4 node-policy → ✅; version bump.
- `CLAUDE.md` PLAY: flip → **M7 `--aicontrol`** (reuses `admin_ops::*`).
- `JOURNAL.md`: arc-close entry (retrospective: 2 commits, Fork X, no checkpoint; NP-D1–D6 realised in code; honest as-built deltas).

**Verification:** `cargo test --workspace` unchanged from C1 (doc-only); clippy clean.

**C2 DoD:** all docs updated; audit + design → COMPLETED; ROADMAP + PLAY flipped; JOURNAL entry in the same commit; per-file `git add`. No DECISIONS.md change (NP-D# arc-local, D-069). **Status: COMPLETED** header on this runbook is the close signal (no "commit pushed" checklist item — chicken-and-egg).

---

## Conventions (reminders)

- Explicit `git add <file>` per file — never `git add .`; `git status` sanity-check before commit; multi-paragraph commit messages via multiple `-m`. Joe pushes manually; Claude never pushes.
- `Filesystem:edit_file` for surgical edits (exact-whitespace `oldText`); verify new files via `Filesystem:get_file_info`; write each file to disk before moving to the next (no prose-then-batch, J-098).
- Stop-and-surface (Rule 3): if a confirm-at-pickup item resolves differently than assumed, surface it before proceeding.

## Cross-refs

- `tasks/M6_NODE_POLICY_DESIGN.md` (v1.0, NP-D1–D6) — the locked design this executes.
- `tasks/M6_NODE_POLICY_AUDIT.md` (v1.0) — the reality map.
- `xgen-core/src/federation/federation_policy.rs` — the structural sibling for store + `Default`-as-prime-invariant.
- Precedent runbooks: `tasks/M6_AUTH_MODULE_REGISTRY_IMPL.md`, `tasks/M6_BOOTSTRAP_CLIENT_IMPL.md` (store-before-consumer; `AdminContext` threading shape).

---

*Runbook ready — C1 has no checkpoint; Clair proceeds on pickup.*
