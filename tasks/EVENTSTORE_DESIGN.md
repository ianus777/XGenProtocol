# EventStore — Design

> **Status**: ACTIVE  
> Version: 1.7  
> Date: June 2026  
> **Last updated**: 2026-06-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Basis

Realises **D-080 (2026-05-29, LOCKED)** per `tasks/EVENTSTORE_AUDIT.md` v1.1, at the stage that belongs now. **Architectural line (Joe, 2026-06-02):** DB engines (SQLite, redb, …) live **as modules behind the `EventStore` trait — never in core**; core ships a **minimal, non-engine, vanilla-production-viable default backend**. So this milestone = the **trait (seam)** + a **small durability floor on the vanilla file backend**; engines are opt-in modules in a later milestone. No D-080 amendment. ES-D# arc-local (D-069).

**Status:** 🔒 LOCKED (Joe) · ⏸ deferred. **All in-scope ES-D# are now locked; the design is closed → runbook next.**

---

## §2 ES-D1 — `EventStore` trait shape 🔒 LOCKED (Joe, 2026-06-02)

`trait EventStore { append(&mut self, Event)->Result<(),StoreError>; get(&self,&EventXgid)->Result<Option<Event>>; range(&self, since_seq)->Result<Vec<Event>>; contains(&self,&EventXgid)->bool; len(&self)->usize }` in xgen-core. **Owned returns** (engine-agnostic; in-memory impl clones). **`range` = by append sequence (R1)** — monotonic append counter; causal ordering stays in the layer above (`collect_sync_history` + topo-sort); works on the vanilla backend with no engine. **`&dyn` dispatch** at the ~5 consumer sites. Keep `contains`/`len`.

**Trait boundary (design note, honest — not hidden):** the trait abstracts the **store index**; the vanilla backend's *durability* is the (now-hardened, §4) xgen-node file-persistence layer. A future engine module (SQLite/redb) provides an alternate impl that unifies index + durability; **that unification — and any trait widening it needs — is the engine-module milestone's job**, not this one. Minimal now, clean seam for later.

---

## §3 ES-D2 — default (vanilla) backend 🔒 LOCKED

The trait's default impl wraps **what exists today** — the in-memory index (`dag/store.rs::EventStore` → the in-memory `impl`) plus the per-Space JSON file persistence — extended with a **monotonic append-seq counter** for `range` R1. This is the backend a **vanilla node runs in production** without any engine module. No engine, no SQLite, no new DB dep.

---

## §4 ES-D3 — core durability floor 🔒 LOCKED (Joe, 2026-06-02) — *minimal, not an engine*

The vanilla file backend gets a small no-data-loss floor — and **nothing more**:

- **F-1 → atomic write:** write to `<file>.tmp` → `sync_all` → atomic `fs::rename` over the live file. Windows-correct: rename = `MoveFileExW(REPLACE_EXISTING)` via std; **dir-fsync `#[cfg(unix)]` only** (explicit cfg split, not a silent skip).
- **F-2 → honest-fail on corrupt read:** **no** `unwrap_or_default()` silent-empty. On bad parse → **loud** log + **quarantine** `<file>.corrupt-<ts>` + keep the node up; `replay` continues other Spaces.
- **F-4 → fsync** is part of the atomic write.

**Explicitly NOT in core (= engine territory = modules, D-080):** append-log structure, checksums/integrity beyond parse-validity, indexing, query, concurrency machinery, compaction. The line: *don't corrupt, don't silently lose* — not *write a database*.

---

## §5 ES-D4 — write-error contract 🔒 LOCKED

The persistence path returns `Result`; on failure **log loud + propagate** (no more `let _ = fs::write`). **v1 does NOT block accept/ack on persist failure.** Absorbs the long-flagged silent-write candidate → real `D-###` assigned at close.

---

## §6 ES-D5 — call-site re-route 🔒 LOCKED

~5 xgen-core consumers (`dag/graph`, `dag/pending`, `message/exchange`, `resolution/algorithm`, `node/runtime`) move to `&dyn EventStore`; the 3 `persist_event` sites + `read_persisted_events` + the `space audit-rebuild` reader stay on the (hardened) persistence layer. Mechanical.

## §7 ES-D6 — `store.rs` header fix 🔒-at-close
Strike the stale "Phase 2 = replace with an indexed on-disk store" line.

---

## §8 Deferred — storage-engine **module** milestone (Tier 2, NOT here)
SQLite (D-080 reference) / redb as **opt-in modules behind the trait** · per-Space schema · JSON→engine migration (migration subsystem) · engine-grade durability (WAL / integrity / compaction) · trait-boundary unification (index + durability) · full crash-recovery suite. The trait + floor built here are the seam those modules drop into.

**Tier-2–4 conformance note (cross-cutting — capture against the tier model + D-080 when this milestone opens):** a node that asserts **Tier 2–4** guarantees **must run the durable storage engine module** — the vanilla minimal floor (§4) is *not* crash-proof (it is "don't corrupt, don't silently lose," not ACID/WAL) and is insufficient for those tiers. This is a **node-conformance requirement** (normative; D-080 already requires a node to have durable storage), **not** a wire-protocol change and **not** a free-floating implementation detail — the engine and the how are implementation, the *must-have-durable-storage-to-credibly-assert-T2–4* is normative. Durability lands at **T1 too by construction**: one store per Space holds all tiers mixed, so the substrate sits *below* the tiers and cannot be made crash-proof "only for T2–4"; it also serves T1's own keypair-permanence accountability in its own right.

**Vanilla scale ceiling + operator contract (cross-cutting; capture in Ch4 §4.12 + Appendix L at close).** The vanilla file backend is whole-file-replace JSON (§3–§4), which is *correct and crash-safe* but does **not** scale:
- **Append cost is O(N) per event** (re-serialise + re-write the whole Space file), so a Space's lifetime cost is ~O(N²); and the **full history is resident in RAM** (the in-memory index) for the session, with a transient full-serialise spike on every append. Comfortable to roughly **thousands–low-tens-of-thousands of events per Space**; beyond that it gets slow and memory-heavy. A weeks-long session on a busy Space is exactly the case that approaches this.
- **Operator contract (the honest promise):** vanilla is *adequate for modest nodes and tells you when you've outgrown it* — it does **not** silently degrade. A vanilla node SHOULD track cheap signals (per-Space event count / file size; optionally append latency) and emit a **loud "storage heavy — install the durable engine module" warning** past a threshold. "If load gets heavy, install an engine module" is the stated, fair contract. (Mechanism = implementation; the *must-warn-not-silently-degrade* posture is the contract.)
- **Cheap in-scope mitigation (for the runbook, stays no-engine):** the writer serialises from the **in-memory store**, not by re-reading the file first (today's `persist_event` re-reads) — removes one whole-file pass per append. Constant-factor win, does **not** change O(N²); fold into C2.
- **Guard-rail (do NOT cross):** vanilla stays *dumb whole-file*. **No** trimming / rotating / segmenting / incremental-record appends to fake scale — that is the slope straight into the custom append engine D-080 rejected. Efficiency at scale = the **engine module** (a library: SQLite/redb), never hand-rolled in core.

This closes the boundary story: **vanilla = correct + simple + bounded, with an honest "install the engine when heavy" contract; engine module = the scale + crash-proof upgrade a heavy or Tier-2–4 node runs.**

**Module-framework stance (cross-cutting; D-### candidate sibling to D-080 — promote at the module-framework milestone).** The module system is **by-trade implementation, not normative spec.** The sockets (one **narrow trait per slot** — `EventStore`, `TemperaturePlugin`, auth-module, future viewers), the loader, the capability registry, and the **`kind ∈ {system, display}` × `host ∈ {node, client}`** taxonomy are implementation architecture — grown per real slot, kept *out* of the protocol. **Normative is only the minimum other parties rely on:** node-conformance requirements (durable storage exists; T2–4 needs the engine — already D-080), auth-tier module **semantics** (what a tier assertion *means* cross-node — the contract is normative, the implementation by-trade), and federation-visible **capability advertisement** (D-080). Structure = two layers: **narrow per-slot sockets** (compile-time-typed contracts) under **one shared framing** (registry + loader + the kind×host trust/placement policy — e.g. a display module may not mutate source-of-truth; a node-only system module won't load in the client). **Do not spec a module framework**; let it emerge by-trade across real slots and normalise only contracts that actually recurred — sibling to D-080's *contract-normative / engine-by-trade* split, and to "detail revealed empirically" + the three-instance-durability promotion bar. **EventStore here is the first system·node slot instance of this stance.** **UI surface (keep-in-mind → `ui/docs/xgen-ui-notes.md` N-007):** every module needs a UI representation in both apps — *system* modules too (install / enable / select, status / health, and warnings like the vanilla "storage heavy" operator contract need a UI home), not only *display* modules; each slot is designed with "where does this appear in the UI, and how is it managed?" as a first-class question.

---

## §9 Verification
Doc-only. Suite at 984/0/1 (J-226), not re-run. D-080 unchanged.

## §10 Next-active
**Design closed (ES-D1–ES-D6 locked).** Runbook — `tasks/EVENTSTORE_IMPL.md`: **C1** `EventStore` trait + in-memory `impl` + append-seq/`range` · **C2** vanilla file backend behind the trait + ES-D3 floor (atomic write + honest-fail/quarantine) + ES-D4 `Result` contract · **C3** `&dyn` call-site re-route · **close** (D-074 atomic: Appendix L + ES-D6 header fix + ROADMAP drift correction + `D-###` + ROADMAP/CLAUDE/JOURNAL). Clair stood down until the runbook closes.
