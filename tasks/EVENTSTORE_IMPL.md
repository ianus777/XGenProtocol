# EventStore — Implementation Runbook

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

Executes `tasks/EVENTSTORE_DESIGN.md` v1.6 (ES-D1–ES-D6 LOCKED) over `tasks/EVENTSTORE_AUDIT.md` v1.1. Realises **D-080 (LOCKED)** at the seam stage: the `EventStore` **trait** + a **vanilla file backend** with a **minimal durability floor**; DB engines stay later **modules**. No D-080 amendment. Status flips ACTIVE when Clair picks up C1.

**Commit-plan refinement of design §10 (honest, D-065):** design §10 sketched C1 trait / C2 backend / C3 re-route. Renaming the concrete `EventStore` → trait **breaks every consumer in the same compile**, so the xgen-core trait + impl + `range` + consumer re-route are **one atomic commit (C1)**; the xgen-node durability floor is **C2**; then close. Two code commits + close.

---

## §2 Commit plan

### C1 — xgen-core: `EventStore` trait + in-memory backend + `range` + `&dyn` re-route (atomic; must compile)
- **Rename** the concrete `dag/store.rs::EventStore` struct → **`InMemoryEventStore`**.
- **Define** `pub trait EventStore` (ES-D1): `append(&mut self, Event)->Result<(),StoreError>` · `get(&self,&EventXgid)->Result<Option<Event>>` (**owned**, clones) · `range(&self, since_seq: u64)->Result<Vec<Event>>` · `contains(&self,&EventXgid)->bool` · `len(&self)->usize`. In `xgen-core` (SPDX GPL-2.0-or-later, matching the crate).
- **impl `EventStore for InMemoryEventStore`**: `insert`→`append`; add a **monotonic append-seq** counter + an ordered index (e.g. `BTreeMap<u64, EventXgid>`) so `range(since_seq)` is a cheap suffix scan; `get` clones to owned.
- **Re-route the ~5 consumers** to `&dyn EventStore`: `dag/graph.rs`, `dag/pending.rs`, `message/exchange.rs`, `resolution/algorithm.rs`, `node/runtime.rs` (+ the `dag/mod.rs` owner field). Owned-`get` call sites adapt (no more `&Event` borrows from the store).
- Tests: in-module trait/impl tests (append + dedup; owned `get`; `range` suffix-by-seq incl. `since_seq` past the end → empty; `contains`/`len`); existing store tests adapted to the new signatures; whole workspace compiles + green.

### C2 — xgen-node: durability floor (ES-D3) + write-error contract (ES-D4)
- **Atomic write (F-1):** NEW `xgen-node/src/atomic_write.rs` (pure, unit-testable; SPDX BUSL-1.1) — `atomic_write(path, bytes) -> io::Result<()>`: write `<path>.tmp` → `File::sync_all` → `fs::rename(tmp,path)` (Windows atomic-replace via `MoveFileExW(REPLACE_EXISTING)` in std) → **`#[cfg(unix)]` dir-fsync** (explicit cfg split, not a silent skip). `persist_event` calls it.
- **Write-error contract (ES-D4):** `persist_event` returns `Result`; replace `let _ = fs::write(...)`; on `Err` → **loud** `tracing::error!` + propagate; **v1 does NOT block accept/ack**. The 3 call sites (`app.rs:~2244/2281/2554`) log-and-continue. Absorbs the silent-write candidate → real `D-###` at close.
- **Honest-fail on corrupt read (F-2):** `read_persisted_events` + `replay_spaces_from_dir` stop using `.ok()…unwrap_or_default()` / silent `continue`; on parse failure → **loud** + **quarantine** `<file>.corrupt-<unix_ts>` + continue other Spaces (node still comes up).
- **Read-skip mitigation:** `persist_event` serialises from the in-memory store, not by re-reading the file first (removes one whole-file pass; constant-factor, no O(N²) change).
- Tests: atomic-write round-trip; injected mid-write failure leaves the existing file intact; injected `persist_event` error logs-and-continues (no abort); a corrupt Space file is quarantined + remaining Spaces still replay; a valid file is unaffected. `tempfile`-backed `spaces_dir` (J-086 precedent).

### Close — D-074 atomic, doc-only
- NEW `docs/xgen_appendix_l_en.md` — EventStore service reference (as-built): trait + the three primitives, the vanilla file backend + format, the atomic-write durability floor (incl. Windows cfg split), recovery + quarantine, and the **boundary** (vanilla ceiling + operator contract; engine = later module). Graduate the §8 cross-cutting notes here + **Ch4 §4.12** (Tier-2–4 conformance; scale ceiling; module-framework stance). Mandated header.
- **ES-D6:** fix the stale `xgen-core/src/dag/store.rs` header ("Phase 2 = on-disk").
- **DECISIONS.md:** assign the absorbed silent-write contract a real `D-###`; Joe's call on promoting any ES-D# / the module-framework stance.
- **ROADMAP drift correction** (strike the "engine-free hand-rolled / amend D-080" paragraph + fix the stale "in-memory only" current-state claim + the tree line) + Durable EventStore ✅ + version · CLAUDE PLAY flip · JOURNAL close entry. audit + design + this runbook → COMPLETED.

---

## §3 Per-commit DoD
- `cargo test --workspace` (baseline **984/0/1**; each commit ≥ baseline + its new tests)
- `cargo build --workspace --all-targets` 0/0
- `cargo clippy --workspace --lib --tests --all-features -- -D warnings` clean
- Explicit `git add <file>` per file; `git status` before commit; multi-paragraph message via multiple `-m`. Joe pushes. **No "commit pushed" DoD item** — `Status: COMPLETED` is the signal.

---

## §4 Touch points
| File | What | Commit |
|------|------|--------|
| `xgen-core/src/dag/store.rs` | `EventStore` trait + `InMemoryEventStore` + seq/`range` | C1 (header fix at close) |
| `xgen-core/src/dag/mod.rs` | owner field → `InMemoryEventStore`; re-exports | C1 |
| `xgen-core/src/dag/graph.rs` · `dag/pending.rs` · `message/exchange.rs` · `resolution/algorithm.rs` | consumers → `&dyn EventStore`; owned-`get` | C1 |
| `xgen-core/src/node/runtime.rs` | `stores` field (see §5.1) | C1 |
| `xgen-node/src/atomic_write.rs` | **NEW** pure atomic-write helper | C2 |
| `xgen-node/src/lib.rs` | `mod atomic_write;` | C2 |
| `xgen-node/src/app.rs` | `persist_event`→`Result`+atomic+read-skip; `read_persisted_events`/`replay_spaces_from_dir` quarantine; 3 call sites LOUD | C2 |
| `docs/xgen_appendix_l_en.md` · `docs/xgen_ch4_implementation.md` · ROADMAP/CLAUDE/JOURNAL/DECISIONS | close | close |

---

## §5 Confirm-at-pickup (D-078)
1. **`NodeRuntime.stores` type — RESOLVED (A) concrete** (Chat Claude, 2026-06-02; impl-structure call within locked ES-D1/ES-D5, not a Joe-lock). `HashMap<SpaceXgid, InMemoryEventStore>`; the swap seam lives at the **consumer `&dyn EventStore` boundary** (ES-D5), not in `stores`. Rationale: only one impl exists this milestone, so `Box<dyn>` would add heap + vtable indirection and ripple to every insert site for zero current benefit, while `&dyn` at consumers already makes them backend-agnostic. `Box<dyn>` is the engine-module milestone's localized change. Call sites pass `&store` / `&mut store` (unsized coercion to `&dyn` / `&mut dyn`).
2. **`range` / `collect_sync_history` — RESOLVED (A) leave** (Chat Claude, 2026-06-02). Implement + unit-test `range(since_seq)` as the seam primitive only; **`collect_sync_history` keeps its current direct causal traversal — do NOT rewire it onto `range`.** Rationale is *semantic, not just minimal*: `range` is **append-sequence**-ordered (R1 — a local monotonic counter), whereas `collect_sync_history` needs **causal / topo-DAG** order against a peer's causal frontier (DAG tips). A peer's last-known point is a causal frontier, **not** a local append-seq number, so `range`'s `since_seq` does not answer "what does this peer need." Rewiring would be a correctness trap (append order ≠ causal order). `range` stands as a primitive for a future engine-backend's incremental fetch, not for the sync path.
3. **append-seq index shape — Clair's call** (cheap suffix scan is the only requirement). Lean: since the store is **append-only** and the seq is a **contiguous monotonic counter**, an ordered `Vec<EventXgid>` makes `range(since_seq)` a slice (`[since_seq..]`) with O(1) append; `BTreeMap<u64,EventXgid>` is only needed if seqs could be sparse (they can't be). Either is acceptable.
4. **Windows dir-fsync** — confirm std behaviour; encode the `#[cfg(unix)]` split explicitly. Re-read at C2.

## §6 Test strategy
Pure helper (`atomic_write`) unit-tested in isolation. Trait/impl tested in-module. Durability wired via injected failures + a `tempfile`-backed `spaces_dir`. No live-Node e2e required for v1.

## §7 Verification baseline
Suite at **984/0/1** (J-226). Doc-only until C1.

## §8 Next-active
**C1 (Clair)** — xgen-core `EventStore` trait + `InMemoryEventStore` + `range` + `&dyn` re-route. §5.1/§5.2 RESOLVED (both A) — clear to implement. Milestone open committed (J-227).
