# Storage-Engine Substitution — Design
> **Status**: COMPLETED  
> Version: 1.0  
> Date: June 2026  
> **Last updated**: 2026-06-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Basis

The in-milestone substitution step (Joe call, D-065 — the milestone cannot honestly close until it lands). C1–C4 shipped a registry-constructible, durable `xgen-store-sqlite` engine through the compile-time spine, but selection does **not** yet thread into per-Space construction: `NodeRuntime`'s three `…or_insert_with(|| Box::new(InMemoryEventStore::new()))` sites (`xgen-core/src/node/runtime.rs:265 / 432 / 723`) still build the vanilla RAM store, so a node with `asserts_tier=2` + `sqlite` passes the SE-D4 gate while writing every Space to RAM (silent false-durability). This design closes that.

It is the **§4.12.1 "per-Space SQLite" shape J-228 struck as drift** (Ch4 §4.12 as-built banner + "Known Tradeoffs"), now grounded against live code and locked before any factory threading. SE-SUB-D# arc-local (D-069); promotions evaluated at milestone close. Gates Clair's substitution commit. Baseline suite **1020/0/1**.

**Live-code constraints that shaped the locks.**
- Construction sites live in **`xgen-core`, which is I/O-free and config-blind by design** (Ch4 §4.2.1 — no `tokio`, no filesystem). The engine table + factory + SE-D4 gate live in `xgen-node/src/storage_engine.rs`; the engine is a third crate.
- C4 `SqliteEngine::open(SqliteSettings{ path })` → one file = one `events` table; append-seq = `COUNT(*)` (**global-per-file**, so the SE-D1 durable-seq contract holds only when one file == one Space).
- Vanilla already persists one `sha256_<hex>.json` per Space under `spaces_dir` via `xgen-node/src/app.rs::space_file_name` — a per-Space-file model **and** a filesystem-safe encoding already in production. Crucially, that JSON layer is the vanilla **durability**, and it sits in `xgen-node`'s app layer **outside** the `EventStore` trait (J-228 ES-D2: the trait/`InMemoryEventStore` is RAM-only; `persist_event` + `replay_spaces_from_dir` bolt durability on top).

---

## §2 Locked decisions (SE-SUB-D1 … SE-SUB-D5; D6 in §3)

### SE-SUB-D1 — Granularity: per-Space DB file (Q1, the hinge)
One `sha256_<hex>.db` per Space under the engine's directory. The 1:1 swap of the vanilla per-Space JSON.
- **C4 engine unchanged** — one file = one `events` table = one Space ⇒ the existing `COUNT(*)` append-seq stays correct (SE-D1 contract holds for free). A single-DB-with-`space_id`-column shape would break that seq (needs per-Space `WHERE space_id`), require a space filter on every query, and collide with the per-Space `Box<dyn EventStore>` ownership (J-228 seam) — heavier rework for no v1 gain.
- **Deletion / GDPR** — per-Space physical isolation keeps a future right-to-be-forgotten arc to a unit file-delete + in-memory drop; a single-DB `DELETE WHERE space_id` would violate the append-only invariant and not reclaim space without `VACUUM`. (Not built here — no caller; the point is not to foreclose it.)
- Single-DB stays available **later as a distinct engine variant**, not the v1 default. Cost accepted: N connections/handles at hundreds of Spaces — exactly the "install a heavier / differently-granular engine" operator-contract case (Appendix L vanilla-ceiling note).

### SE-SUB-D2 — File layout: `[storage.sqlite].dir` (Q2)
The engine's settings carry a **directory root**, not a single file. Per-Space path = `<dir>/<stem>.db`.
- Mechanism (keeps C4 untouched): the **host closure** (xgen-node, SE-SUB-D5) reads `[storage.sqlite].dir`, encodes the space_id to a stem (SE-SUB-D3), and **templates a per-Space `EngineSettings { path = <dir>/<stem>.db }`** before calling the engine's `open`. The engine still receives and validates a `path` (SE-D5 intact — the engine owns/validates its own settings; the host only fills the per-Space path into them). `SqliteSettings { path }` need not change.
- `dir` absent → default to the host's resolved `spaces_dir` (`app.rs::resolve_spaces_dir`), so sqlite and vanilla share the Pattern-A Space root, differing only by extension.

### SE-SUB-D3 — Encoding: one encoder, no drift (Q3)
The filesystem-safe `Xgid`→stem function is **single-source** (D-067). Grounding refines the pre-lock "lift to a shared crate" mechanism (honest note, D-065): a cross-crate lift is **not needed** — both consumers (the vanilla `persist_event`/`read_persisted_events` path and the new sqlite per-Space closure) live in **`xgen-node`**, so they share the existing `app.rs::space_file_name` directly. The lock intent (one encoder, no second copy) is met without moving it.
- Refactor: generalise `space_file_name(space_id) -> "<…>.json"` into `space_file_stem(space_id) -> "sha256_<hex>"` (extract-hex; fallback `replace(['/',':','.'],"_")`), and let callers append the extension (`.json` for vanilla, `.db` for sqlite). Touches the two vanilla call sites + the replay `*.json` scan filter.
- Reversibility note: since space_ids are always `xgen://hash/sha256:<hex>`, the stem `sha256_<hex>` reverses cleanly to the space_id — load-bearing for engine-mode startup enumeration (§3).

### SE-SUB-D4 — Lifecycle: eager-open, no silent vanilla fallback (Q4)
Lazy `or_insert_with` cannot return a `Result`, but an engine `open` can fail (disk/permissions). The store is opened via the factory at a point where failure is handled, and an open failure **never silently produces a vanilla RAM store under an engine selection** (that would re-introduce the false-durability this milestone exists to kill).
- A single birth path: an `ensure_store(space_id)` helper that calls the SE-SUB-D5 factory. On `Err`: loud + map per caller — `dispatch_event` → `Rejected` (the Space cannot accept events rather than RAM-shadow them), `ingest_event` → loud log-and-skip (rides its existing `(a).iii.α` vigilance posture, but must NOT fall back to vanilla), replay → skip-Space loud. Vanilla's closure is infallible, so today's behaviour is byte-for-byte unchanged.
- Eager intent: provision the engine store at the create/replay sites that own the lifecycle, so a first-touch in `dispatch_event` is normally already-populated. Exact eager-site list + whether `ingest_event` gains a `Result` vs keeps log-and-skip = confirm-at-pickup (§5).

### SE-SUB-D5 — Carrier: factory closure on `NodeRuntime` (Q0)
`NodeRuntime` gains an injected per-Space factory; xgen-core stays I/O-free *as a crate* (the I/O lives inside an injected `dyn Fn` returning a `dyn EventStore`; xgen-core never names a file).
- `type StoreFactory = Box<dyn Fn(&SpaceXgid) -> Result<Box<dyn EventStore + Send + Sync>, StoreInitError> + Send + Sync>` (home: xgen-core, next to `EventStore`). `NodeRuntime::new` defaults to the vanilla closure `|_| Ok(Box::new(InMemoryEventStore::new()))` — behaviour-neutral, so every existing xgen-core constructor/test is unaffected (C2-style).
- xgen-node, after the SE-D4 gate yields a `StorageSelection`, builds the engine closure (capturing the table `EngineFactory` + resolved `[storage.sqlite]` settings + `dir` + the `space_file_stem` encoder) and installs it (`NodeRuntime::with_store_factory(...)` or setter). The three sites call `ensure_store` → the factory.

---

## §3 SE-SUB-D6 — durability-authority handover (Scope B, LOCKED)

**The consequence of SE-SUB-D5, surfaced honestly (D-065 / Rule 3); Joe-locked Scope B 2026-06-03.** The four lock questions framed substitution as "thread the factory so the live store is the engine." Grounding shows that is necessary but **not sufficient**: vanilla durability is the app-layer JSON (`persist_event` at `app.rs:2371`/etc. + `replay_spaces_from_dir` JSON scan), which runs **independent of the store type**. Thread sqlite as the live store and leave that layer untouched and you get:
- a redundant JSON shadow written on every accept (double-write), **and**
- `replay_spaces_from_dir` (scans `*.json`) remains the real recovery source — so on restart the node rehydrates from JSON, not from the engine, and re-`append`s into the already-populated `.db` (swallowed `DuplicateEventId` noise). The engine's durability is real but **not authoritative** — false-durability survives by a different route.

**Two honest scopes:**
- **Scope A (factory-only):** thread the factory, leave JSON as-is. Rejected — the clean-looking shortcut that doesn't actually make `asserts_tier=2 + sqlite` true.
- **Scope B (factory + durability-authority handover):** when an engine is active, the engine **owns durability** — the app-layer JSON persist is bypassed, and startup rehydrates per-Space `SpaceState` from `engine.range(0)` instead of the JSON scan. Engine-mode startup enumerates Spaces by scanning `<dir>/*.db` and reversing each `sha256_<hex>` stem to its space_id (SE-SUB-D3 reversibility), opening each engine and replaying its events. Vanilla mode keeps the JSON persist + JSON-scan replay unchanged.

**Locked: Scope B** — it is what "the milestone cannot honestly close until substitution lands" actually requires (honest longer work over fast shortcuts). It extends the touch-surface to `xgen-node`'s `process_inbound` persist call(s) and `replay_spaces_from_dir` (both gate on engine-active). D-084 sibling: the bypass is loud and explicit, not a silent branch.

---

## §4 Touch points (by crate)

| Crate / file | What | Lock |
|---|---|---|
| `xgen-core/src/dag/store.rs` (or sibling) | `StoreFactory` type + `StoreInitError` | D5 |
| `xgen-core/src/node/runtime.rs` | `store_factory` field + `with_store_factory` + `ensure_store`; the 3 sites call it; default = vanilla closure | D4/D5 |
| `xgen-node/src/storage_engine.rs` | build the engine closure from `StorageSelection` + settings | D2/D5 |
| `xgen-node/src/app.rs::space_file_name` | → `space_file_stem` (+ 2 vanilla callers, `*.json` scan) | D3 |
| `xgen-node/src/app.rs` (config) | `[storage.sqlite].dir` read in the closure builder; default `spaces_dir` | D2 |
| `xgen-node/src/app.rs` (`process_inbound` persist + `replay_spaces_from_dir`) | engine-active gate: bypass JSON persist; rehydrate from `engine.range(0)`; engine-mode Space enumeration via `<dir>/*.db` | D6 |
| (rides the substitution commit, per PLAY) | C3 registry GUID compare-by-value fix — instance of Ch4 §4.12.5, not decided here | — |

---

## §5 Confirm-at-pickup (D-078) — resolve before the relevant commit
1. `StoreFactory` / `StoreInitError` exact shape + home (xgen-core); `with_store_factory` constructor vs setter.
2. `ensure_store` + per-site open-failure mapping (`dispatch_event`→`Rejected`, `ingest_event`→loud-skip vs `Result`, replay→skip-loud) — no path silently yields a vanilla store under an engine selection.
3. `space_file_name`→`space_file_stem` generalisation + the two vanilla call sites + the replay `*.json` scan + the new `.db` consumer.
4. `[storage.sqlite].dir` read site + default-to-`spaces_dir`; per-Space `EngineSettings { path }` templating.
5. (D6) the engine-active gate points in `process_inbound`/`replay_spaces_from_dir`; engine-mode startup enumeration + `range(0)` rehydration; double-write avoidance proof.

---

## §6 Out of scope
- **Single-DB / `space_id`-column engine variant** — deferred; a future engine, not a granularity of this one (SE-SUB-D1).
- **Space deletion / right-to-be-forgotten feature** — not built (no caller); SE-SUB-D1 keeps the door open.
- **C5 honest "active engine" advert** — its own step, rides after substitution (PLAY: "active engine advert is only truthful once the engine is actually active").

Per Rule 0 + Rule 3 + D-065 + D-067 + D-069 + D-074 + D-078 + D-080.
