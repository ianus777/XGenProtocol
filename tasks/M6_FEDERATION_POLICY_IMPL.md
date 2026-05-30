# Federation Policy 2b — Implementation Runbook (D-071 arc)
> **Status**: ACTIVE  
> Version: 1.2  
> Date: May 2026  
> **Last updated**: 2026-05-30  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Arc position

Implementation runbook for **federation-admin-control sub-arc 2b (policy)**. Design LOCKED
in `tasks/M6_FEDERATION_POLICY_DESIGN.md` v1.0 (FAC-D3/D4, 2026-05-30). Sibling worked
example: the 2a runbook `tasks/M6_FEDERATION_ADMIN_CONTROL_IMPL.md` (COMPLETED). Executor:
Clair, after each checkpoint fires. 4 commits.

## Locked inputs (from the design doc — do not re-open)

- **FAC-D4 shape:** `FederationPolicy { mode: Allow|Deny, allowed_spaces: Option<Vec<…>> }`.
  `rate_limit` DEFERRED. Sibling store (not a relationship field). `allowed_spaces`
  restrictive-only: effective = `shared_spaces ∩ allowed_spaces`.
- **FAC-D3 sites:** both — outbound `apply_federation_push` + the inbound federated-event
  ingest gate — via one pure `policy_permits(peer, space_id) -> bool`, default-permit.
- **Prime invariant:** absent policy = Allow + all = today, byte-for-byte.

## Checkpoints (Joe-lock)

- **Checkpoint #1 — before Commit 1. LOCKED (2026-05-30).** Concrete types pinned by name
  against the 2a precedents (D-078): `PolicyMode { Allow, Deny }`; `FederationPolicy { mode:
  PolicyMode, allowed_spaces: Option<Vec<SpaceXgid>> }` (`SpaceXgid` confirmed = the element
  type of `FederationRelationship::shared_spaces`); `FederationPolicyStore` inner serde field
  `policies: HashMap<NodeXgid, FederationPolicy>`; API `new`/`set`/`get`/`remove`/`all`/
  `is_empty`/`save`/`load` (`set` = insert-or-replace, named for the `set-policy` verb, vs the
  queue's `add`); on-disk path `data_dir.join("xgen-node_federation_policy.json")` (exact
  sibling to `xgen-node_federation_queue.json`); helper `policy_permits(policy:
  Option<&FederationPolicy>, space_id: &SpaceXgid) -> bool`.
- **Checkpoint #2 — at Commit 2 (LOAD-BEARING). LOCKED (2026-05-30) → Option B.** Clair
  code-traced the inbound site; the original plan (a `dispatch_event` signature change to gate
  inside the xgen-core F-3 ingest) proved to cost **~85 call-site edits (~79 of them test
  fixtures)**, not the ~6 assumed. **Joe-locked Option B:** put the inbound consult **node-side
  in `xgen-node::process_inbound`** (where the node decides what to *do* with an
  already-validated event), leaving `dispatch_event`'s signature **untouched (0 of 85 sites)**.
  Both consults now live in xgen-node (symmetric with the outbound site). This is WITHIN the
  FAC-D3 lock — the design deliberately did not name the inbound site (D-078); the trace named
  it. **D-067 bonus:** a shared xgen-core `space_id_of(&Event)` resolver, reused by
  `dispatch_event` + `apply_federation_push` + `process_inbound`, collapses 3 existing
  duplicate copies. No design-doc change.

## Commits

### Commit 1 — store + types
NEW `xgen-core/src/federation/federation_policy.rs` (sibling to `pending_queue.rs`,
declared in `federation/mod.rs`):
- `PolicyMode { Allow, Deny }` — `#[derive(Default)]` → `Allow`; `#[serde(rename_all="lowercase")]`.
- `FederationPolicy { mode: PolicyMode, allowed_spaces: Option<Vec<SpaceXgid>> }` —
  `SpaceXgid` (checkpoint #1 lock; = `FederationRelationship::shared_spaces` element type,
  imported from `xgen_common::xgid`). `Default` → `{ Allow, None }` (= permit-all, the prime
  invariant as a value).
- `FederationPolicyStore` with inner `#[serde(default)] policies: HashMap<NodeXgid,
  FederationPolicy>` (JSON `{ "policies": { "<peer_node_id>": {...} } }`) — `new`/`set`/`get`/
  `remove`/`all`/`is_empty`/`save`/`load`; reuses `RegistryError`; `save`/`load` take `&Path`.
  Default-absent semantics live in the helper (Commit 2), not the store.
- NO run_node wiring this commit (an unused store would trip clippy `-D warnings`) — first
  consumer is Commit 2.
- Tests: serde round-trip, default = permit-all, set/get/remove, save/load.

### Commit 2 — enforcement (LOAD-BEARING; checkpoint #2 LOCKED → Option B)
- Pure helper `policy_permits(policy: Option<&FederationPolicy>, space_id: &SpaceXgid) -> bool`
  in xgen-core: `None` → `true`; `Some{Deny}` → `false`; `Some{Allow, allowed_spaces}` →
  `allowed_spaces.is_none() || contains(space_id)`. No I/O, no drift (D-067).
- Shared xgen-core resolver `space_id_of(&Event) -> &SpaceXgid` (or owned per the field type),
  reused by `dispatch_event` + `apply_federation_push` + `process_inbound` — collapses the 3
  existing duplicate copies (D-067 net improvement).
- `dispatch_event` signature **UNTOUCHED** (Option B — 0 of the ~85 call sites change).
- Wire the **outbound** consult into `apply_federation_push`: skip the push to a peer whose
  policy denies, or whose `allowed_spaces` excludes the event's space.
- Wire the **inbound** consult into **`xgen-node::process_inbound`** (node-side, post-validation):
  drop/refuse the peer's event when policy denies or excludes the space.
- Thread the live `FederationPolicyStore` Arc to both xgen-node sites (load at run_node
  startup, sibling to the 2a queue threading).
- **MANDATORY default-permit regression test (D-065):** absent policy → both sites behave
  byte-for-byte as today; the existing `federate()`-based suite stays green.
- Tests: helper truth table (None/Deny/Allow±spaces); `space_id_of` dedup; outbound skip;
  inbound drop; default-permit regression.

### Commit 3 — verbs
- `admin_ops::federation_set_policy` (WRITE, audited): upserts a `FederationPolicy` for a
  peer (`mode` + optional `allowed_spaces`); unknown-peer handling per pickup (policy may
  pre-exist a relationship → set is allowed even absent a relationship, per the pre-deny
  design intent).
- `admin_ops::federation_show_policy` (READ, not audited): returns the stored policy or the
  default (permit-all) with an explicit "default (no policy set)" marker.
- `FederationCommand::{SetPolicy, ShowPolicy}` clap variants + pipe dispatch arms.
- Thread the policy-store Arc `run_node → start_pipe_server → dispatch_line →
  dispatch_admin` (sibling to the 2a `federation_queue` threading).
- New FED_30xx codes as needed (continue the 2a series; pick at pickup, document by name).
- Tests: set→show round-trip; default shown when unset; audit-trail assertion on set-policy.

### Commit 4 — close (doc-only)
- `docs/xgen_node_admin_ops_design.md` §6.A1 → set-policy/show-policy SHIPPED (honest
  as-built deltas vs the Block-4 sketch, D-065).
- `tasks/M6_BACKING_AUDIT.md` A1 row + summary → both verbs SHIPPED; the 5 deferred A1 verbs
  now fully shipped across 2a+2b.
- `tasks/M6_FEDERATION_POLICY_DESIGN.md` + this runbook → COMPLETED.
- CLAUDE PLAY flip → next D-071 arc (auth-module-registry). ROADMAP 2b row ✅.
- JOURNAL close entry. Full verification + isolated re-runs of the default-permit regression.

## Definition of done (per commit)
- `cargo build --workspace --all-targets` 0 errors / 0 warnings.
- `cargo test --workspace` green (real counts recorded in JOURNAL, Rule 2); new tests named.
- `cargo clippy --workspace --lib --tests --all-features -- -D warnings` clean.
- Default-permit regression green (Commits 2–4).
- Canonical docs updated in the same commit as the state change (D-069).
- (No "commit pushed" checklist item — the COMPLETED header is the signal; Joe pushes.)

## Cross-refs
- Design: `tasks/M6_FEDERATION_POLICY_DESIGN.md` (FAC-D3/D4 LOCKED). Audit:
  `tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md`. 2a precedents:
  `xgen-core/src/federation/pending_queue.rs`, `approval_gate_decision`, the 2a queue
  threading in `pipe.rs`/`admin_ops.rs`. D-065 / D-067 / D-069 / D-074 / D-078.

---

*Runbook ACTIVE. Clair executes from Commit 1 after checkpoint #1.*
