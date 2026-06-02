# EventStore — Phase 0 Node Storage Audit
> **Status**: ACTIVE  
> Version: 1.1  
> Date: June 2026  
> **Last updated**: 2026-06-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Purpose & scope

Phase 0 backing audit for the **EventStore** milestone — the **Node Storage Audit** that **D-080 (2026-05-29)** names as its own Phase-0 follow-up. It grounds the current de-facto node storage (on-disk layout, engine in use, access patterns) against D-080's locked contract, before any conformance work. Doc-only, no code.

**D-080 is LOCKED and governs (no amendment).** It is a *tiered* decision, and this milestone realises only the tier that belongs at this stage of node development:

- **Tier 1 — the `EventStore` interface (the trait): `append` · `get` · `range`.** Mandatory, identical per node. **This milestone builds this** (the seam).
- **Tier 2 — the backing engine.** Pluggable, **never absent** — a node always has *a* store. Today that backend is the node's existing storage (in-memory index + per-Space JSON files); it is *not* absent. D-080 names **SQLite as the eventual reference engine**, swappable behind the trait — and per Joe (2026-06-02) that engine is **deferred to a later storage-engine *plugin*** milestone. **We do not wire SQLite into the node core at this stage.**
- **Tier 3 — projections** (search/analytics/admin query). Optional, beside the log. Out of scope.

So the staging is: **trait now (Tier 1) over the current storage as the default backend; durable engine (SQLite reference) later as a plugin (Tier 2)** — entirely inside D-080's "engine pluggable, never absent" frame.

**Drift correction (Rule 3, for an honest baseline):** the ROADMAP near-future paragraph describing an *"engine-free, hand-rolled append-only log; amend D-080; relocate the engine to the client"* has **no JOURNAL or DECISIONS trace**, contradicts D-080, and is **drift** — struck when this milestone opens; D-080 governs.

---

## §2 Current de-facto storage (grounded)

### §2.1 Runtime index — in-memory
`xgen-core/src/dag/store.rs::EventStore` = `HashMap<EventXgid, Event>`, per-Space at `NodeRuntime.stores: HashMap<SpaceXgid, EventStore>` (`runtime.rs:148`). Methods: `insert` (≈append, dedup on `event_id`), `get`, `contains`, `len`, `values` (order **not** guaranteed). The natural **in-memory backend** once the trait is extracted.

### §2.2 Disk layer — raw `std::fs`
`xgen-node/src/app.rs`: `persist_event` (`:3434`) writes **one JSON file per Space** (`space_file_name` → `sha256_<h>.json`), a whole `Vec<Event>` array, **read-modify-rewritten in full on every event** via `std::fs::write` (errors swallowed). `read_persisted_events` (`:3423`) reads it. `replay_spaces_from_dir` (`:3534`) is boot recovery — scan `*.json`, parse, **topological-sort**, `ingest_event` each; parse failure → silent skip. Runs before the listener opens (spec 4.8.5).

Together, §2.1 + §2.2 **are** the node's current Tier-2 backend — the store that is "never absent."

### §2.3 Engine in use
**None** — raw `std::fs` + `serde_json`. (`rusqlite` exists in the workspace, but only for the A6 SQLite *admin trail*, `audit.rs`, J-153/J-154 — unrelated to the event store. A future engine plugin *may* reuse it; this milestone does **not**.)

### §2.4 Access patterns
`append` (`persist_event`) · `get`/`contains` (in-memory) · iterate-all (`values` / `replay`). **No store-level `range(since-DAG-point)` primitive** — federation sync / reconnect catch-up assembles history via `collect_sync_history` over the in-memory store (mechanism to re-read at design), not a store range. D-080's third primitive has no current home.

---

## §3 Gap vs D-080 (staged)

| Tier | D-080 | Current state | This milestone |
|---|---|---|---|
| **1 — interface** (`append`/`get`/`range`) | mandatory trait | **PARTIAL** — in-memory store has append(`insert`)/`get`; **no `range`**; **concrete struct, not a trait** | **builds it** |
| **2 — backing engine** (pluggable, never absent) | SQLite reference, swappable | **present as the current storage** (in-memory + per-Space JSON) — not absent; not yet the durable reference engine | **wraps current storage as the default backend; SQLite engine → later plugin** |
| **3 — projections** | optional | none | out of scope |
| **durability floor** | crash-safe append, integrity, backup | **not met** (see §4) | **engine plugin's job**; optional minimal interim floor is an open fork (§6) |

---

## §4 Findings (current durability state)

- **F-1** non-atomic write — in-place `std::fs::write`; a mid-write crash corrupts the *whole* Space file.
- **F-2** silent-empty on parse failure — read paths `.ok()…unwrap_or_default()` / `continue`; a corrupt file makes the Space's entire history vanish on restart. Compounds F-1.
- **F-3** swallowed write errors — `let _ = std::fs::write(...)`; disk-full/IO error → event in memory but never on disk, silently.
- **F-4** no fsync / durability barrier.
- **F-5** O(n) whole-file rewrite per event.
- **F-6** no write-vs-state ordering invariant.

**Full resolution = the durable engine plugin** (D-080 Tier 2), not a hand-rolled engine here (D-080 rejected that). **F-1 + F-2 are the two data-loss bugs** (whole-Space corruption / silent total loss) — whether to close *just those* with a minimal no-data-loss floor in this milestone, or defer everything to the plugin, is the open fork in §6. **Good (unchanged):** all three persist sites run under the held runtime mutex (no concurrent-writer race); replay topo-sorts and doesn't re-fire the PAL-D1 hook.

---

## §5 Boundary (out of scope — reference only)

Mutable-snapshot stores — `bootstrap/registration_store.rs`, node `NodeState` (`xgen-node_state.json`), `space_local_metadata`, `FederationRegistry` (`*_federation.json`) — JSON-backed, same non-durable `fs::write` family, but **not the event log** and not D-080's subject. (Ch4 §4.9/§4.11/§4.12 already note these are JSON, not SQLite.) Separate follow-on; mapped here so a later pass inherits it.

---

## §6 Open questions → design phase (ES-D#)

**In scope (this milestone — the seam):**
1. **`EventStore` trait shape** — `append`/`get`/`range` signatures + error type + crate home (xgen-core); return ownership; `range(since-DAG-point)` semantics; dispatch (`&dyn` vs generic).
2. **Default backend** — wrap the current storage (in-memory index + per-Space JSON persistence) as the trait's default `impl`, so the node keeps working and tests stay green.
3. **Interim durability fork (F-1/F-2)** — close just the two data-loss bugs with a minimal no-data-loss floor now, or defer *all* durability to the engine plugin? (Joe's call.)
4. **Call-site re-route** — the ~5 xgen-core consumers (`graph`, `pending`, `exchange`, `resolution`, `runtime`) + the 3 `persist_event` sites + `read_persisted_events` + the `space audit-rebuild` reader → through the trait.
5. **`store.rs` header correction** — strike the stale "Phase 2 = replace with an indexed on-disk store" line, at close.

**Deferred to the storage-engine plugin milestone (Tier 2 / SQLite — NOT here):** per-Space SQLite schema · JSON→engine migration (ties to the migration subsystem) · transaction/fsync/WAL policy · full crash-recovery suite.

---

## §7 Verification

Doc-only. Suite at **984/0/1** (J-226), not re-run. **D-080 LOCKED, unchanged — no amendment.** No DECISIONS.md change in Phase 0 (ES-D# arc-local, D-069).

---

## §8 Next-active

**Design phase** — `tasks/EVENTSTORE_DESIGN.md`: lock ES-D# on §6, **trait shape first** (it gates the rest). The engine (SQLite reference) is explicitly a **later plugin**, not this milestone. Then runbook → commits (Clair) → close. Clair stood down until the design closes.
