# Storage-Engine / Plugin-Framework Milestone — Phase 0 Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: June 2026  
> **Last updated**: 2026-06-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose & scope

Phase 0 backing audit for the storage-engine milestone, scoped wider than "add an engine": it
delivers the **compile-time plugin/module spine** plus the **first plugin** (`xgen-store-sqlite`)
through it, behind the `EventStore` trait shipped at J-228 (D-080). The milestone realises §L.7's
"engine-as-module" seam and §L.8's Tier-2–4 conformance, and is the **first `system·node` slot
instance** of the by-trade module-framework stance.

This audit grounds the plan's premises against live source and hunts the stale premise before
design locks (project principle: *subsystem audits precede dependent milestones*). Doc-only; no
code. Suite stands at J-229's **999**/0/1, not re-run.

## 2. The locked bundle being audited

Confirmed with Joe across this session (decisions reached by reasoning, not yet given D-numbers —
SE-D# to be locked in the design phase):

- **Compile-time plugins, v1.** A plugin = a Rust crate compiled into the binary. No runtime
  loading. "Anyone can write one" = publish a conforming crate; adoption = add dep + feature +
  rebuild.
- **Source as the universal cross-platform format.** Ship source, compile per target; `#[cfg]`
  guards per-OS bits inside one codebase. No `.dll`/`.so` matrix, no ABI, no per-OS artifacts.
- **Dynamic native loading REJECTED on security grounds** (see §8). Worst case = identity-key
  theft in-process on a no-anonymity, key-holding Node. Author identity-tiering = accountability,
  not containment — a future-arc layer, not a v1 substitute.
- **GUID handshake.** `kind-GUID` (shared per slot, generated once, copied by implementers) +
  `impl-GUID` (per-plugin, developer-generated). Plain UUIDv4. Own newtypes (`ModuleKindId`,
  `ModuleImplId`) — never `Xgid` (local, dev-assigned, never federates).
- **Metadata authoritative, location never trusted, reject-unknown-loud.** Descriptor const in the
  plugin's own code; no manifest file (compile-time has no host↔plugin gap to bridge).
- **Settings in node config** (`[storage.<engine>]`), not sidecars. Plugin owns + validates its
  own settings schema; host passes through; loud on failure.
- **Tier→engine durability gate, present from T1.** Engine descriptor advertises an
  assurance/durability class; the Node refuses to start if the selected engine under-delivers for
  the tier it serves. Low bar at T1, binding at T2–4.
- **Spine host-neutral; node `EventStore` slot node-scoped.** Client slots are a future milestone
  that inherits the spine for free (own trait, own registry in `xgen-client`); not built now, not
  walled off.
- **Dual naming kept:** *module* = system, *plugin* = ui; `kind` carries the distinction in code;
  one unified handshake mechanism.

## 3. The seam we build on (J-228, grounded)

`xgen-core/src/dag/store.rs` — `trait EventStore { append / get→owned / range(since_seq) /
contains / len }`, `&dyn` swap boundary (ES-D5). `range` is **append-seq, not causal** (R1).
Vanilla backend `InMemoryEventStore` (`HashMap` index + `Vec<EventXgid>` order). Durability floor
in `xgen-node` (`atomic_write.rs`, `persist_event → io::Result`, quarantine; D-084). Engines are
explicitly later opt-in modules behind this trait; D-080 unchanged.

## 4. Grounded findings (live source, main tree)

### 4.1 Trait-fit — the trait is a *data seam*, not an *engine seam*. NEEDS-DESIGN.
`EventStore` (store.rs:46) is **sync**, with **no constructor, no lifecycle (open/close), no
transaction/batch boundary, no `descriptor()`**. An engine (SQLite/redb) needs all four. The clean
shape is a **sibling trait** (e.g. `StorageEngine: EventStore` adding `open(settings)->Result<Self>`
+ `descriptor()`), leaving `EventStore` the lean data seam consumers already use. Exact shape =
design phase's central question. Sub-points to resolve: (a) sync vs async — SQLite/`rusqlite` is
sync, fits today's sync trait; an async engine later would force the question, so v1 stays sync;
(b) **durable append-seq** — vanilla rebuilds `order` in RAM on replay (process-local seq); an
engine MUST persist its own seq or `range(since_seq)` breaks across restart. Real contract item.

### 4.2 Owner/consumer topology — one structural-reach change. CONFIRMED.
Consumers already take `&dyn EventStore` (`graph.rs:82`, `pending.rs:210/240/284/347` — five sites,
matching J-228). **Owners are concrete:** `dag/mod.rs:41 store: InMemoryEventStore` (in `RoomDag`)
and `NodeRuntime.stores: HashMap<_, InMemoryEventStore>` (J-228 §5.1, chosen concrete). Runtime
engine selection forces the owner fields to **`Box<dyn EventStore>`** — the single change with
reach. Everything else is additive behind the existing `&dyn` boundary.

### 4.3 Tier model — the gate's input is a SET, not a scalar, and is bootstrap-scoped. NEEDS-JOE-LOCK.
**The plan's "the Node's asserted tier" premise is STALE.** There is no singular node tier. Grounded:
- `bootstrap.auth_tiers_served: Vec<u8>` (`admin_ops.rs:2525/2847`; surfaced `aicontrol.rs:237`) —
  a **set** of tiers a bootstrap node advertises serving; config example `auth_tiers_served = [2, 3]`
  (`app.rs:3805`).
- Per-auth-module `accepted_tiers: Vec<AuthTier>` (`admin_ops.rs:2112/2250`) — tiers a module accepts.

So the durability gate cannot read "the tier." Two referent questions for Joe:
1. **Which signal?** "Tiers served/accepted" (advertisement to others) is arguably a *different
   thing* from "durability my own storage must guarantee." §L.8 says a Node that **asserts** T2–4
   must run the engine — "asserts" maps to served/accepted. Candidate: gate input = **max over the
   tiers the node serves/accepts**, engine must meet that floor.
2. **Config home?** `auth_tiers_served` is **bootstrap-scoped** today; a non-bootstrap Node may
   never set it. The gate needs a tier signal that exists on every Node, or an explicit "this Node
   asserts up to tier N" config. Likely a small new field, not a reuse.

This is the milestone's one genuine design-risk catch. Recommend resolving it early in design.

### 4.4 Dependency & config state. CONFIRMED.
- `uuid` (v1, `v4`) is in **`xgen-core` only**; **absent from `xgen-common`/`xgen-node`/
  `xgen-client`**. Host-neutral descriptor types belong in `xgen-common` → adds `uuid` there
  (trivial). Alternative: park them in `xgen-core` (uuid present) at the cost of host-neutrality.
  Design pick (SE-D# candidate).
- `NodeConfig` at `app.rs:80`; **no `[storage]` section, no `storage_engine` field** today. New
  config section required + its classification in M7-standalone's reload table (almost certainly
  **restart-required** — swapping the live store mid-run is unsound).

## 5. Verdicts summary

| # | Premise | Verdict |
|---|---------|---------|
| 4.1 | `EventStore` suffices for an engine | **NEEDS-DESIGN** — needs factory/lifecycle/descriptor sibling trait; durable seq contract |
| 4.2 | Owners boxable for runtime selection | **CONFIRMED** — `Box<dyn EventStore>` for `RoomDag.store` + `NodeRuntime.stores`; one reach change |
| 4.3 | "Node's asserted tier" exists | **STALE → NEEDS-JOE-LOCK** — it's a set, bootstrap-scoped; define the gate's tier input |
| 4.4 | uuid + config ready | **CONFIRMED** — uuid in core only; no `[storage]` section yet |

## 6. Open questions → design phase (SE-D# candidates)

- **SE-D1** Engine contract shape: sibling `StorageEngine: EventStore` (`open`/`descriptor`) vs
  widening `EventStore`. Sync v1. Durable append-seq.
- **SE-D2** Descriptor/identity types + home: `ModuleKindId`/`ModuleImplId`/`AssuranceClass`/
  `Descriptor` in `xgen-common` (host-neutral, +uuid) vs `xgen-core`.
- **SE-D3** Registry: feature-assembled table in `xgen-node`; explicit per-feature `register(...)`
  (recommended) vs `inventory`/`linkme` auto-collect (rejected — link-time magic, less auditable).
- **SE-D4** Tier→engine gate: input signal + config home (the 4.3 lock) + assurance-class taxonomy.
- **SE-D5** Settings contract: `[storage.<engine>]` passthrough; plugin-owned schema + loud validation.
- **SE-D6** Owner boxing: `Box<dyn EventStore>` migration of `RoomDag.store` + `NodeRuntime.stores`.
- **SE-D7** Plugin-author source-conformance spec (the "what a conforming crate must contain" doc).
- **SE-D8** Capability advertisement of running engine/assurance (may be light/deferred).

## 7. Code inventory (for the runbook)

**New:** descriptor/identity types (`xgen-common`, +uuid); engine contract (sibling trait,
`xgen-core`); node registry + tier-gate (`xgen-node`); `xgen-store-sqlite` plugin crate (workspace
member, deps `xgen-core`+`xgen-common`, `rusqlite`, durable seq).
**Modified:** owner fields → `Box<dyn EventStore>` (`dag/mod.rs`, `NodeRuntime`); `NodeConfig`
`[storage]` section + reload-table classification; tier-signal config field (per 4.3).
**Verify-in-runbook:** the durable-seq persistence path; reload classification; whether any
non-bootstrap Node has a tier signal at all.

## 8. Security record — why dynamic was rejected (so a future arc inherits the threat model)

A runtime-loaded native plugin runs **in-process with full Node privileges** — no sandbox. On a
verified-identity, no-anonymity, **key-holding** Node (worst at T4), a malicious or supply-chain-
compromised binary can steal the identity key and impersonate an accountable participant, or
silently tamper with the append-only log. The GUID/descriptor handshake verifies *declaration
honesty*, never *behaviour* — it cannot catch a binary that declares correctly and acts
maliciously. Mitigations each cost the goal: signing/allowlist re-adds a gatekeeper (anti-
philosophy); Wasm sandbox contains but fights native-engine speed + direct disk (wrong for a
storage slot); author identity-tiering gives **attribution, not containment**. Compile-time has
**no loading boundary → no loading vulnerability**, and trust = "built from readable source",
matching the project's open-source trust model. A future dynamic/Wasm arc must carry its own threat
model (Wasm-sandbox-or-signing) before a key-holding Node loads foreign code; the parked sidecar/
scan/index/settings/author-tier designs belong to that arc. UI plugins (`kind=plugin`, `host=client`)
are the better future Wasm candidate; system modules (`host=node`) stay compiled-in.

## 9. Next-active

**Design phase** — `tasks/STORAGE_ENGINE_DESIGN.md`: lock SE-D1–SE-D8, **resolving the 4.3 tier-gate
input first** (it gates SE-D4 and the gate's config field). Then runbook → spine + registry → owner
boxing → `xgen-store-sqlite` → close (D-074: Appendix L update, Ch4 §4.12, ROADMAP/CLAUDE/JOURNAL;
graduate the module-framework candidate-D if framework scope warrants). Clair stood down until the
runbook closes.

Per Rule 0 + Rule 3 + D-065 + D-069 + D-074 + D-078 + D-080.
