# Appendix L — EventStore: Node Durable Storage in XGen Protocol

> **Status**: ACTIVE  
> Version: 1.0  
> Date: June 2026  
> **Last updated**: 2026-06-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## L.1 Introduction

This appendix is the canonical expository home for the **EventStore** — the Node's durable storage service for Space history. The architectural commitment is recorded in `DECISIONS.md` D-080 (the storage shape) and D-084 (the write-path failure contract); the milestone that built it is the Durable EventStore milestone (`tasks/EVENTSTORE_AUDIT.md`, `…_DESIGN.md`, `…_IMPL.md`, JOURNAL J-227/J-228). This appendix explains the *as-built* shape: the trait seam, the vanilla file backend, the minimal durability floor, and the boundary at which an operator should install a storage engine module.

The design rests on one separation: **the durable-storage requirement is normative; the engine that satisfies it is by-trade.** D-080 requires every Node to have durable storage and keeps the backing engine pluggable-but-never-absent; it explicitly rejects a fully hand-rolled raw-file append engine as the default. What ships at this milestone is the **trait** (the swap seam) plus a **vanilla file backend** carrying a small no-data-loss floor — enough for modest Nodes, with an honest contract to install an engine module when load grows.

A reader who only needs the rule reads D-080 + D-084. A reader who needs to understand or extend the storage layer reads here.

---

## L.2 The shape: one trait, three primitives

The store is reached through the `EventStore` trait (`xgen-core/src/dag/store.rs`, ES-D1). It abstracts the per-Space **store index** and is the swap boundary (ES-D5): consumer functions take `&dyn EventStore`, so a future engine backend can be substituted without touching them.

The surface is deliberately tiny — three primitives plus two queries:

- `append(&mut self, Event) -> Result<(), StoreError>` — append one event; errors if it has no `event_id` or the id already exists (dedup is part of the contract).
- `get(&self, &EventXgid) -> Result<Option<Event>, StoreError>` — point lookup by id.
- `range(&self, since_seq: u64) -> Result<Vec<Event>, StoreError>` — every event with append-sequence `>= since_seq`, in append order.
- `contains(&self, &EventXgid) -> bool` and `len(&self) -> usize` (with a default `is_empty`).

Two contract notes matter:

**Owned returns.** `get` and `range` return owned `Event`s, not borrows. This keeps the trait engine-agnostic — an on-disk engine cannot hand out a borrow into its storage. The vanilla in-memory backend clones to satisfy the contract.

**`range` is by append sequence, not causal order (R1).** A monotonic per-store counter assigns each appended event the next sequence number; `range(since_seq)` returns the suffix from that sequence onward, in append order. This is the primitive a future engine backend's incremental fetch is built on. It is emphatically **not** causal/topological order: the federation sync path (`collect_sync_history` + topo-sort) composes causal order against a *peer's frontier* at a layer above the store and does not route through `range`. A peer's last-known point is a causal frontier (DAG tips), not a local append-sequence number — conflating the two would be a correctness error.

---

## L.3 The vanilla file backend

The default backend (`InMemoryEventStore`, ES-D2) is what a Node runs in production with no engine module installed. It is two parts:

**In-memory index.** A `HashMap<EventXgid, Event>` for lookup, plus a `Vec<EventXgid>` (`order`) recording append sequence. Because the store is append-only (no removal), `order` is contiguous and monotonic, so `range(since_seq)` is an O(1) suffix slice (`order[since_seq..]`) rather than a scan.

**On-disk persistence.** The durable copy lives in the Node's `spaces_dir` as one JSON file per Space. The filename derives from the Space id: `xgen://hash/sha256:<h>` becomes `sha256_<h>.json` (other id shapes have `/ : .` replaced by `_`). The file is a plain JSON array of `Event` objects — human-readable, trivially backed up (one file per Space), and trivially inspected.

Persistence is a **whole-file rewrite per accepted event**: `persist_event` reads the existing array, appends the new event (after a dedup check by `event_id`), and writes the whole array back. This is simple and correct; its cost is discussed in §L.6.

The in-memory index is the **runtime authority**: an event is "accepted" once it is in the index. The on-disk file is the durable record that lets the index be rebuilt on the next start.

---

## L.4 The durability floor

The floor is a *minimal no-data-loss* property — **not** a storage engine. It has three parts.

**F-1 — atomic write (`xgen-node/src/atomic_write.rs`).** A plain `fs::write` truncates the live file before writing the new bytes, so a crash mid-write leaves a partial JSON array and the whole Space becomes unreadable. `atomic_write(path, bytes)` closes that window:

1. write a sibling temp file `<path>.tmp`;
2. `File::sync_all` — flush the bytes to the device, not just the OS cache;
3. `fs::rename(tmp, path)` — atomic replace.

The rename is atomic on both POSIX (`rename(2)`) and Windows (`std::fs::rename` issues `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`). A crash therefore only ever damages the throwaway `.tmp`; the live file flips from the previous complete version to the new complete version in one indivisible step. The fsync-before-rename ordering is load-bearing — without step 2 the OS may report "written" while bytes sit in cache, and a power cut could leave the renamed file empty.

On POSIX the containing directory is fsynced after the rename so the rename itself is durable across power loss. This is an explicit `#[cfg(unix)]` split, **not** a silent skip: Windows exposes no directory-handle fsync and the `MoveFileExW` metadata write is already ordered. The sibling temp keeps the rename on one filesystem (a cross-filesystem rename is not atomic).

**F-2 — honest-fail on corrupt read.** Reads no longer treat a corrupt file as empty (the old `.ok().unwrap_or_default()`, which would make a Space's whole history silently vanish). On a parse failure the read logs **loud** (`tracing::error!`) and **quarantines** the file to `<file>.corrupt-<unix_ts>`, then continues; the `.corrupt-…` suffix means the replay scan never re-attempts it. The Node still comes up; the operator gets a signal instead of silent data loss. On the write path, a *read I/O error* on an existing file is **propagated** rather than swallowed — an unreadable-but-present file is never overwritten with just the new event (which would lose history).

**D-084 — write failure is loud + propagated, but does not block accept/ack in v1.** `persist_event` returns `io::Result`; on failure it logs loud and propagates, replacing the old swallowed `let _ = fs::write(...)`. In v1 a persist failure does **not** block accepting or acking the event: the in-memory store has already accepted it (runtime authority), F-1 guarantees no corruption, and federation replication + the content-addressed DAG make a lost tail event re-syncable. The call sites **log-and-continue**. See D-084 for the full reasoning and the Tier-2–4 escalation path.

Together: *don't corrupt, don't silently lose* — the honest floor, below the level of a database.

---

## L.5 Recovery and quarantine

On start, the Node replays each `*.json` file in `spaces_dir` back into the in-memory index (`replay_spaces_from_dir`), topologically sorting events before ingest so causal order is restored. A file that fails to parse is quarantined (F-2) and that Space starts fresh until the file is restored; **other Spaces still replay** — one corrupt file never takes the Node down. The protocol-audit rebuild (`space audit-rebuild`) reads the same per-Space files via `read_persisted_events`.

The backstops that make the floor sufficient for a vanilla Node: the DAG is **content-addressed and causally chained**, so corruption is *detectable* (a torn store can lose history but cannot silently forge it), and federation **replicates** the log, so a well-federated Space can re-sync after a local loss.

---

## L.6 The boundary: vanilla ceiling and the operator contract

The vanilla backend is correct and simple but **does not scale**, by design:

- **Append cost is O(N) per event** (re-serialise + rewrite the whole Space file), so a Space's lifetime cost is ~O(N²); and the **full history is resident in RAM** for the session, with a transient full-serialise spike per append. Comfortable to roughly **thousands–low-tens-of-thousands of events per Space**; beyond that it gets slow and memory-heavy. A long-running, busy Space is exactly the case that approaches this.
- **Operator contract (the honest promise):** vanilla is adequate for modest Nodes and *tells you when you've outgrown it* — it must not silently degrade. A vanilla Node SHOULD track cheap signals (per-Space event count / file size; optionally append latency) and emit a **loud "storage heavy — install the durable engine module" warning** past a threshold. "If load gets heavy, install an engine module" is the stated, fair contract.
- **Guard-rail (do not cross):** vanilla stays *dumb whole-file*. No trimming / rotating / segmenting / incremental-record appends to fake scale — that is the slope straight into the custom append engine D-080 rejected. Efficiency at scale is the **engine module**, a library (SQLite/redb), never hand-rolled in core.

---

## L.7 The seam: storage engines as a later module

The `EventStore` trait is the swap boundary. A future storage-engine milestone delivers SQLite (D-080's reference engine) and/or redb (the Rust-native alternative; RocksDB is the heavier write-throughput escape hatch) as **opt-in modules behind the trait** — each an alternative `EventStore` implementation, selected by Node config, advertised as a per-Node capability. The engine unifies index + durability (append becomes O(1), the on-disk store is indexed so history need not be fully resident) and supersedes both the whole-file rewrite and the read-skip optimisation deferred at this milestone.

This follows the project's module-framework stance (a candidate sibling to D-080): the module system is by-trade implementation — narrow per-slot traits under a shared registry/loader, tagged `kind ∈ {system, display}` × `host ∈ {node, client}` — with only the trust- and federation-bearing *contracts* normative. The EventStore is the first **system · node** slot instance of that stance.

---

## L.8 Tier-2–4 conformance

The vanilla floor is *no-data-loss*, not crash-proof in the ACID/WAL sense. A Node that asserts **Tier 2–4** identity guarantees **must run the durable storage engine module** — the vanilla floor alone is insufficient for those tiers. This is a **node-conformance requirement** (normative; D-080 already requires durable storage), not a wire-protocol change: the engine and the *how* are implementation; the *must-have-durable-storage-to-credibly-assert-T2–4* is normative.

Durability lands at **Tier 1 too by construction**: one store per Space holds all tiers mixed, so the substrate sits below the tiers and cannot be made crash-proof "only for T2–4"; it also serves Tier 1's own keypair-permanence accountability in its own right. The escalation lever named in D-084 — a Node tightening to **commit/fsync-before-ack** — is the engine module's territory and a future decision, not a silent drift.

---
