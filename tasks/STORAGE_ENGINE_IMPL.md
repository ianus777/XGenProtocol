# Storage-Engine / Plugin-Framework — Implementation Runbook

> **Status**: ACTIVE  
> Version: 1.0  
> Date: June 2026  
> **Last updated**: 2026-06-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Basis

Executes `tasks/STORAGE_ENGINE_DESIGN.md` v1.0 (SE-D1 + SE-D4 LOCKED by Joe; SE-D2/D3/D5/D6/D7/D8 authored within the bundle) over `tasks/STORAGE_ENGINE_AUDIT.md` v1.0. First `system·node` slot instance of the module-framework stance, on J-228's `EventStore` trait (**D-080 LOCKED**, unchanged). SE-D# arc-local (D-069). Status flips to in-progress when Clair picks up C1. Baseline suite **999/0/1**.

**Commit-plan shape (honest, D-065).** The spine is **additive** (new types, new trait — green on landing); owner boxing is **inert** (box the vanilla owners, no behaviour change); the registry+gate land **with only vanilla registered** (gate passes at T1); the engine arrives **last, through the finished spine**. Nothing here forces a same-compile break like the J-228 rename did, so the steps separate cleanly. Five code commits + close.

---

## §2 Commit plan

### C1 — spine: descriptor/identity types + `StorageEngine` trait (additive; green)
- **`xgen-common`** (+`uuid` v4 dep — absent today, audit 4.4): `ModuleKindId` / `ModuleImplId` newtypes over UUIDv4 (**never `Xgid`** — local, dev-assigned, never federate); `AssuranceClass` enum (**minimal v1 ladder:** `BestEffort < Durable`, `Ord`, each variant `→ fn satisfies_tier(u8) -> bool`); `Descriptor { kind_id, impl_id, name: &'static str, assurance: AssuranceClass }`. Dual naming: `kind` carries module(system)/plugin(ui).
- **`xgen-core`**: `pub trait StorageEngine: EventStore { fn open(settings: &EngineSettings) -> Result<Self, EngineError> where Self: Sized; fn descriptor() -> Descriptor; }`. `EngineSettings` = opaque (SE-D5 — a `toml::Value`/map newtype the host passes through untyped). **Static trait — NOT `dyn`-compatible by design** (§3 of DESIGN); do **not** add `&self` or try to box `dyn StorageEngine`.
- Tests: UUID newtype round-trip + `!=`-across-kinds; `AssuranceClass` ordering + `satisfies_tier` table; `Descriptor` const constructs. Additive — full suite still 999 + new.

### C2 — owner boxing (SE-D6; inert — no behaviour change)
- `xgen-core/src/dag/mod.rs:41` `RoomDag.store: InMemoryEventStore` → **`Box<dyn EventStore>`**; construction → `Box::new(InMemoryEventStore::new())`.
- `xgen-node` `NodeRuntime.stores: HashMap<_, InMemoryEventStore>` → **`HashMap<_, Box<dyn EventStore>>`**; insertion sites box the vanilla.
- Consumers already take `&dyn EventStore` (J-228 ES-D5) → read sites untouched. `append(&mut self)` works through `Box`. **Confirm-at-pickup (D-078):** any `&mut`-through-`Box` owner-method site that needs `&mut **store` deref care.
- Tests: existing suite green unchanged (this commit must be behaviour-neutral — that *is* its DoD).

### C3 — registry + tier gate + config (vanilla-only; gate passes at T1)
- **`xgen-node`**: `EngineTable` + `fn register<E: StorageEngine + 'static>(&mut EngineTable)` storing `E::descriptor()` + a boxing factory `fn(&EngineSettings) -> Result<Box<dyn EventStore>, EngineError>` closing over `E::open`. Single assembly site. **Confirm-at-pickup (D-078):** `EngineTable` value type (`fn` ptr vs boxed `Fn`).
- **`NodeConfig` (`app.rs:80`):** NEW `[node].asserts_tier: Option<u8>` + NEW `[storage]` section (`storage_engine: Option<String>`, `[storage.<engine>]` sub-tables passed through). Reload-table class = **restart-required** (M7-standalone table; live store-swap unsound).
- **Tier gate (SE-D4):** at config load — derive `floor = max over (bootstrap.auth_tiers_served ∪ all module accepted_tiers)`; `asserts_tier = config.unwrap_or(floor)`; **reject loud if `asserts_tier < floor`** (under-declare). Selection: `storage_engine` → table lookup → **reject-unknown loud** (no silent vanilla fallback); `None` → vanilla default (always present, no feature). Then **`descriptor.assurance.satisfies_tier(asserts_tier)` or refuse to start.**
- Tests: floor-derive from a served/accepted fixture; clamp-up accepted, clamp-below rejected; unknown engine name → loud reject; vanilla(`BestEffort`) passes T1, fails T2; default-when-`None` = vanilla.

### C4 — first plugin: `xgen-store-sqlite` through the spine (SE-D5 settings)
- NEW workspace crate `xgen-store-sqlite` (deps `xgen-core` + `xgen-common` + `rusqlite`; license per workspace convention — **confirm at pickup**). `impl StorageEngine`: `descriptor()` const (kind-GUID copied from the slot, impl-GUID generated; `assurance = Durable`); `open(settings)` reads its **own** `[storage.sqlite]` schema (e.g. `path`), **validates loud** (bad settings → `EngineError`, node refuses start — SE-D5), opens the DB. `impl EventStore`: `append`/`get`/`range`/`contains`/`len` over SQLite with a **persisted append-seq** (its own monotonic column — the §2-SE-D1 durable-seq contract; `range(since_seq)` = `WHERE seq > ?`). Register under `#[cfg(feature = "store-sqlite")]` in C3's assembly site.
- **Confirm-at-pickup (D-078):** the durable-seq persistence mechanism (dedicated `seq` column + index vs rowid); on-restart `len`/`range` correctness.
- Tests (behind the feature): round-trip through `&dyn EventStore`; **durable-seq survives a reopen** (append → drop → reopen → `range(0)` returns all in seq order, `len` correct); dedup on duplicate `EventXgid`; gate accepts `Durable` at T2; bad `[storage.sqlite]` → loud `open` failure.

### C5 — SE-D8 capability advert (light)
- Node-state surface reporting the **active engine descriptor + assurance class** (operator-visible; reuse the existing node-state/`ai_status`-style read path). **Federation/wire advert deferred** (durability is local conformance, not a wire contract). Fold into C4's close if it lands in <~30 lines; otherwise its own commit.

### Close — D-074 atomic, doc-only
- **Appendix L** (`docs/xgen_appendix_l_en.md`) — NEW engine-module section (as-built): the spine (descriptor/identity, `StorageEngine` sibling, registry, gate), `xgen-store-sqlite` as slot-instance #1, the `asserts_tier` gate + `AssuranceClass` ladder.
- **Ch4 §4.12** — graduate the module-framework stance note (kind×host taxonomy, by-trade impl, compile-time-only v1) + the SQLite-engine realisation.
- **SE-D7 conformance appendix** — "what a conforming storage-engine crate must contain" (descriptor const + GUID handshake, `StorageEngine` impl, own settings schema + loud validation, durable append-seq, honest `AssuranceClass`).
- **DECISIONS.md** — assign SE-D# the arc's real D-numbers if promoted (Joe's call); **evaluate the module-framework candidate-D** — this is slot-instance **#1**; the three-instance bar is unmet, so **likely stays a candidate** unless Joe judges framework scope warrants early promotion.
- **ROADMAP** (storage-engine ✅ + Present ⚫ + version) · **CLAUDE PLAY** flip · **JOURNAL** close entry. audit + design + this runbook → COMPLETED.

---

## §3 Per-commit DoD
- `cargo test --workspace` (baseline **999/0/1**; each commit ≥ baseline + its new tests; C4 engine tests behind `--features store-sqlite`)
- `cargo build --workspace --all-targets` 0/0 (+ `--features store-sqlite` for C4)
- `cargo clippy --workspace --lib --tests --all-features -- -D warnings` clean
- Explicit `git add <file>` per file; `git status` before commit; multi-paragraph message via multiple `-m`. Joe pushes (GitHub Desktop / PowerShell). **No "commit pushed" DoD item** — `Status: COMPLETED` is the signal.

---

## §4 Touch points
| File | What | Commit |
|------|------|--------|
| `xgen-common/Cargo.toml` | +`uuid` v4 | C1 |
| `xgen-common/src/...` (module spine) | `ModuleKindId`/`ModuleImplId`/`AssuranceClass`/`Descriptor` | C1 |
| `xgen-core/src/dag/store.rs` (or sibling) | `StorageEngine` sibling trait + `EngineSettings`/`EngineError` | C1 |
| `xgen-core/src/dag/mod.rs` | `RoomDag.store` → `Box<dyn EventStore>` | C2 |
| `xgen-node` `NodeRuntime` | `stores` → `HashMap<_, Box<dyn EventStore>>` | C2 |
| `xgen-node` (registry) | `EngineTable` + `register::<E>` + assembly site | C3 (sqlite reg C4) |
| `xgen-node/src/app.rs:80` (`NodeConfig`) | `[node].asserts_tier` + `[storage]` section + reload class | C3 |
| `xgen-node` (config load) | floor-derive + clamp + tier→assurance gate + reject-unknown | C3 |
| `xgen-store-sqlite/*` (NEW crate) | `StorageEngine`+`EventStore` impl, descriptor, durable seq, settings | C4 |
| node-state surface | active engine/assurance advert | C5 |
| Appendix L · Ch4 §4.12 · SE-D7 appendix · DECISIONS · ROADMAP · CLAUDE · JOURNAL | close | Close |

---

## §5 Confirm-at-pickup (D-078) — resolve before the relevant commit
1. **C1** — `EngineTable` value type (`fn` ptr vs `Box<dyn Fn>`); `EngineSettings` concrete shape (`toml::Value` newtype vs map).
2. **C2** — any `&mut`-through-`Box` owner site needing explicit deref.
3. **C3** — `AssuranceClass` enum exact variants confirmed minimal (`BestEffort`/`Durable`); the floor-derive reads both `auth_tiers_served` **and** module `accepted_tiers` (union), not one.
4. **C4** — durable append-seq mechanism in SQLite (column+index vs rowid); reopen correctness; `xgen-store-sqlite` license.

Per Rule 0 + Rule 2 + Rule 3 + D-065 + D-066 + D-067 + D-069 + D-074 + D-078 + D-080.
