# Storage-Engine / Plugin-Framework — Design

> **Status**: ACTIVE  
> Version: 1.1  
> Date: June 2026  
> **Last updated**: 2026-06-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Basis

Realises §L.7's "engine-as-module" seam + §L.8's Tier-2–4 conformance, per `tasks/STORAGE_ENGINE_AUDIT.md` v1.0, on top of J-228's `EventStore` trait (**D-080 LOCKED**, unchanged). This is the **first `system·node` slot instance** of the by-trade module-framework stance. Scope = the **compile-time plugin/module spine** + the **first plugin** (`xgen-store-sqlite`) through it. SE-D# arc-local (D-069); promotions + the module-framework candidate-D evaluated at close (D-074). Suite at J-229's **999**/0/1; doc-only at open.

**Status:** SE-D1 + SE-D4 🔒 LOCKED (Joe, 2026-06-02). **SE-D2/D3/D5/D6/D7/D8 nodded by Joe (2026-06-02) — full authored set now locked; design CLOSED → runbook ACTIVE (`tasks/STORAGE_ENGINE_IMPL.md`).** `AssuranceClass` kept minimal (`BestEffort < Durable`, extend at engine #2) on Joe's call. One structural bind surfaced at §3 (SE-D1 object-safety) carried to the runbook as confirm-at-pickup.

---

## §2 SE-D1 — engine contract shape 🔒 LOCKED (Joe, 2026-06-02)

**Sibling trait, not a widening.** `EventStore` (J-228) stays the lean data seam the ~5 consumers already use. Engines implement a sibling:

```
trait StorageEngine: EventStore {
    fn open(settings: &EngineSettings) -> Result<Self, EngineError> where Self: Sized;
    fn descriptor() -> Descriptor;            // associated, no &self (§4 SE-D2 types)
}
```

- **Sync, v1 — forward-constraining lock.** `rusqlite` is sync and fits the sync `EventStore` cleanly. This commits the engine trait to **sync now**; an async engine would be a breaking trait change, deferred to its own arc. Joe-nodded explicitly (it constrains a future arc).
- **Durable append-seq = a real contract item.** Vanilla rebuilds `order` in RAM on replay (process-local seq). An engine **MUST persist its own append-seq**, or `range(since_seq)` breaks across restart. Named in the conformance spec (SE-D7).
- **Seq-stability (named design assumption, not silent).** A `since_seq` cursor is valid **only within one backend identity**. Swapping backends renumbers; a held cursor (e.g. a federation peer's) goes stale. v1 is **restart-required swap + peer re-sync**, so this is sound — but it is *banked*, so a future hot-swap arc inherits the constraint rather than discovering it.

---

## §3 SE-D1 structural realisation — object-safety bind ⚠ surfaced for runbook (confirm-at-pickup, D-078)

`open(...) -> Result<Self>` and `descriptor() -> Descriptor` make `StorageEngine` **not `dyn`-compatible** (a `Self` return + an associated fn with no receiver). That is intended, not a defect: `StorageEngine` is a **static (monomorphised) trait**, and the type-erasure to `Box<dyn EventStore>` happens at the **per-engine registration site** (§5 SE-D3):

```
fn register<E: StorageEngine + 'static>(table: &mut EngineTable) { /* stores E::descriptor() + a fn(&EngineSettings)->Result<Box<dyn EventStore>> closing over E::open */ }
```

So the registry holds `descriptor` + a boxing **factory closure**, never `dyn StorageEngine`. The owner fields (§7 SE-D6) hold `Box<dyn EventStore>` — never `dyn StorageEngine`. This is the clean shape; flagged only because the trait-method signatures don't *look* object-safe and a runbook reader must not "fix" them into a `&self`/`dyn` form. **Confirm-at-pickup:** the exact `EngineTable` value type (`fn` ptr vs boxed `Fn`) at the registration site.

---

## §4 SE-D2 — descriptor / identity types + home 🔒 LOCKED (design-author, within bundle)

- `ModuleKindId` + `ModuleImplId` — **own newtypes over UUIDv4** (`uuid` v4). **Never `Xgid`** (Xgid federates + is protocol-assigned; these are local, dev-assigned, never federate). `kind-GUID` shared per slot (generated once, copied by implementers); `impl-GUID` per-plugin (developer-generated).
- `AssuranceClass` — the engine's advertised durability class (taxonomy in §6 SE-D4).
- `Descriptor` — `{ kind_id, impl_id, name, assurance, … }`, a **const in the plugin's own code**. No manifest file (compile-time has no host↔plugin gap to bridge). **Metadata authoritative, location never trusted, reject-unknown-loud.**
- **Home: `xgen-common`** (host-neutral spine → client slots inherit it free in a later milestone) + adds the `uuid` dep there (trivial; absent today per audit 4.4). Parking in `xgen-core` rejected — costs host-neutrality for no gain.
- **Dual naming:** *module* = system (`host=node`), *plugin* = ui (`host=client`); `kind` carries the distinction in code; one unified handshake mechanism.

---

## §5 SE-D3 — registry 🔒 LOCKED (design-author, within bundle)

Feature-assembled `EngineTable` in **`xgen-node`**; **explicit per-feature `register::<E>(&mut table)`** at a single assembly site gated by `#[cfg(feature = "store-sqlite")]`. **Rejected: `inventory`/`linkme` auto-collect** — link-time magic, less auditable, runs against "built from readable source" trust. Selection: `[node].storage_engine = "<name>"` → look up by descriptor → **reject-unknown loud** (no silent fallback to vanilla when a named engine is missing — that would be a lie about durability). Vanilla (`InMemoryEventStore`, J-228) is the **default when no engine selected**, always present, needs no feature.

---

## §6 SE-D4 — tier→engine durability gate 🔒 LOCKED (Joe, 2026-06-02) — *the audit-4.3 catch, resolved*

The audit killed the "the Node's asserted tier" scalar premise: there is no singular node tier — only `bootstrap.auth_tiers_served: Vec<u8>` (a **set**, bootstrap-scoped) + per-module `accepted_tiers`. Resolution:

- **Gate input = explicit `[node].asserts_tier`, clamped.** A new scalar field, present on **every** node (closes the "non-bootstrap node has no signal" gap).
- **Default when unset = floor over (`auth_tiers_served` ∪ all module `accepted_tiers`).** Existing bootstrap configs Just Work; a plain node with no tier signal defaults to **T1**.
- **Settable upward, never below the derived floor.** Config load **derives the floor, then clamps**; `asserts_tier < floor` is a loud start-time reject — a node cannot under-declare below what it actually serves.
- **The gate:** the engine `Descriptor`'s `AssuranceClass` is compared to `asserts_tier`; **Node refuses to start if the selected engine under-delivers.** Low bar at T1 (vanilla floor passes), binding at T2–4 (vanilla fails → must select a durable engine module, per §L.8).
- **Home: `[node]`** — it's a node-posture property; the engine is the *consumer* of the number, not its owner.
- **`AssuranceClass` taxonomy (authored, confirm-shape at runbook):** minimal ordered ladder — `BestEffort` (vanilla floor: don't-corrupt/don't-silently-lose, not crash-proof) < `Durable` (fsync-on-commit, crash-survivable) < … — mapped to a tier floor each class satisfies. Kept deliberately small in v1; extend when a second engine lands.

---

## §7 SE-D5 / SE-D6 / SE-D7 / SE-D8 — authored within bundle

- **SE-D5 settings contract** 🔒 — `[storage.<engine>]` section, **opaque passthrough**: host reads the sub-table and hands it to `StorageEngine::open` untyped; the **plugin owns + validates its own schema** and is **loud on failure** (open returns `EngineError`, Node refuses start). Host never interprets engine settings. New `[storage]` config section required (absent today, audit 4.4); classification in M7-standalone's reload table = **restart-required** (swapping the live store mid-run is unsound).
- **SE-D6 owner boxing** 🔒 — `RoomDag.store` (`dag/mod.rs:41`) + `NodeRuntime.stores` (`HashMap<_, InMemoryEventStore>`) → **`Box<dyn EventStore>`**. The single structural-reach change (audit 4.2); everything else additive behind the existing `&dyn`. Vanilla becomes "the engine you get when none is selected," not a special-cased concrete type.
- **SE-D7 author-conformance spec** 🔒-shape — a "what a conforming crate must contain" doc: descriptor const (kind-GUID copied + impl-GUID generated), `StorageEngine` impl, settings schema + loud validation, **durable append-seq persistence** (the §2 contract item), honest `AssuranceClass` declaration. Content authored in the runbook; **graduates to an Appendix at close** (D-074).
- **SE-D8 capability advertisement** 🔒-light — a node-state surface reporting the **active engine descriptor + assurance class** (operator-visible). **Federation/wire advertisement deferred** to a future arc (no peer needs to know your engine in v1 — durability is a local conformance property, not a wire contract). Light or deferred per runbook judgement.

---

## §8 Security record — pointer (banked at audit §8)

Dynamic native loading was **rejected on security grounds**: a runtime-loaded native plugin runs in-process with full Node privileges → identity-key theft / log tamper on a no-anonymity key-holding Node (worst at T4); the GUID/descriptor handshake verifies *declaration honesty*, never *behaviour*. Compile-time has **no loading boundary → no loading vulnerability**; trust = "built from readable source." A future dynamic/Wasm arc must carry its **own threat model** (Wasm-sandbox-or-signing) before a key-holding Node loads foreign code; UI plugins (`kind=plugin`, `host=client`) are the better Wasm candidate, system modules stay compiled-in. Author identity-tiering = **attribution, not containment** — a future-arc layer, not a v1 substitute. Full record lives in the audit; design banks it so the future arc inherits the model.

---

## §9 Code inventory (for the runbook)

**New:** descriptor/identity types + `AssuranceClass` + `Descriptor` (`xgen-common`, +`uuid`); `StorageEngine` sibling trait (`xgen-core`); `EngineTable` + `register::<E>` + tier-gate + `[node].asserts_tier` clamp (`xgen-node`); `xgen-store-sqlite` plugin crate (workspace member, deps `xgen-core`+`xgen-common`+`rusqlite`, durable seq, `StorageEngine` impl, descriptor const).
**Modified:** owner fields → `Box<dyn EventStore>` (`dag/mod.rs`, `NodeRuntime`); `NodeConfig` (`app.rs:80`) → `[storage]` section + `[node].asserts_tier` field + reload-table classification (restart-required).
**Verify-in-runbook:** durable-seq persistence path in the SQLite engine; the `asserts_tier` derive-and-clamp at config load; `EngineTable` value type (§3 confirm-at-pickup); reject-unknown-loud on a missing named engine; vanilla-default-when-none path unchanged.

---

## §10 Sequence → runbook

1. **Spine** — descriptor/identity types (`xgen-common`, +uuid) → `StorageEngine` sibling trait (`xgen-core`).
2. **Registry + gate** — `EngineTable` + explicit `register::<E>` (`xgen-node`) → `[node].asserts_tier` field + derive-and-clamp + tier→assurance gate.
3. **Owner boxing** — `RoomDag.store` + `NodeRuntime.stores` → `Box<dyn EventStore>` (the one reach change).
4. **First plugin** — `xgen-store-sqlite` crate through the spine (durable seq, descriptor, settings schema, `Durable` assurance).
5. **Settings + advert** — `[storage.<engine>]` passthrough; SE-D8 node-state surface.
6. **Close (D-074 atomic)** — Appendix L update (engine-module section) + Ch4 §4.12 + SE-D7 conformance appendix + ROADMAP/CLAUDE/JOURNAL; **evaluate promoting the module-framework candidate-D** (this is slot-instance #1 of the stance — the three-instance bar; promote only if framework scope warrants).

**Next-active:** runbook `tasks/STORAGE_ENGINE_IMPL.md` (steps 1–6 commit plan + the §3 object-safety confirm-at-pickup + the §6 `AssuranceClass`-shape confirm). Clair stood down until the runbook closes.

Per Rule 0 + Rule 3 + D-065 + D-069 + D-074 + D-078 + D-080.
