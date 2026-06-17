# XGen Protocol — Arc F (Space Migration Subsystem, PG-11) Phase-0 Audit
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Scope and the D-071 gate

Arc F is the Round-1 D-071 Phase-0 audit for **PG-11** (Space Migration subsystem, S2), selected after Arc G closed (J-250). Spec home: ch3 §3.12 (L4190); wire home: `xgen-common/src/wire.rs:66–98`. Self-contained per gap-audit §4-F.

Grounding only — no design locks (AF-D# land in DESIGN), no code. Scope fork locked in discussion (2026-06-04): **wire the whole state machine, no artificial happy-path slice** (the reject/fail/verify branches are already coded; wiring them is marginal).

## §2 — Spec anchor (what PG-11 demands)

ch3 §3.12.2–3.12.8 defines a complete two-sided migration handshake (source + destination Nodes): request → propose → accept/reject → event-batch transfer → batch-ack → transfer-complete → verify → verified/failed → cutover (`state.space_migrate` DAG record) → member redirect → federation-notify. Load-bearing safety rule (§3.12, L4190): **the source Node MUST NOT delete its Space DB immediately** — it retains until the destination confirms `migration.verified`.

## §3 — As-built findings (AF-A#)

**AF-A1 (GAP-CONFIRMED — built core, absent driver; the keystone).** The wire layer is complete: 12 message types + the `state.space_migrate` DAG record (`wire.rs:66–98`), all with string maps + parse arms + round-trip tests (`types.rs:1047–1232`). The **core logic is built** in `xgen-core/src/migration/` (~733 lines): `state_machine.rs` (`MigrationState` Idle→Negotiating→Transferring→Verifying→Complete/Failed; `MigrationError` 6001–6006; source handlers `handle_migration_request`/`handle_migration_reject`/`handle_verified`/`build_space_migrate_event`; destination handlers `handle_migration_propose`/`accept_event_batch`/`abort_destination`), `transfer.rs` (`batch_events`/`compute_batch_hash`/`identify_tail`), `verification.rs` (`verify_transfer`), and a pure-function `full_migration_end_to_end` test. **The node-side driver is wholly absent** — a workspace grep finds **zero** migration dispatch in `xgen-node/src` (no `handle_migration*` calls, no `MigrationState` ownership, no `StateSpaceMigrate` handling). The module doc-comment states it plainly: *the caller (xgen-node) is responsible for sending wire messages and committing events.* This is the M8/Arc-G built-but-unwired pattern: the logic + tests exist; the milestone is the wiring. GAP-CONFIRMED.

**AF-A2 (missing-wiring inventory — the node driver).** xgen-node must gain: (a) dispatch of the 12 migration wire messages to the existing pure handlers; (b) a per-migration `MigrationState` owned on the node (source side + destination side independently) with the state-transition guards that emit `WrongState` (6006) — the enum exists but **no transition function does**; (c) transport send/receive of the messages (`connection.rs:156 send_migration` exists as the send primitive); (d) commit of the `state.space_migrate` cutover event to the Space DAG; (e) the EventStore export/import bridge (AF-A3); (f) the retention gate (AF-A6); (g) federation-notify + `home_node` propagation (AF-A4); (h) the operator surface (AF-A7).

**AF-A3 (EventStore bridge — no trait gap; fork 2 resolved).** `trait EventStore` (`store.rs:77`) exposes `append(event)`, `get(id)`, and `range(since_seq: u64) -> Vec<Event>`. `range(0)` is a full source export; `append` is the destination import — sufficient for the transfer. `transfer.rs` operates on in-memory `Vec<Event>`, so the node bridges EventStore↔transfer (export via `range`, feed `batch_events`; on the destination, `append` the received batches into a **freshly instantiated per-Space store**). Per D-080/J-232, the destination also rebuilds its SQLite materialization cache from the synced log — node-side plumbing, not a trait change. **No EventStore sub-gap.**

**AF-A4 (the load-bearing design question — authority-anchor flip; fork 4).** There is **no `apply_event` arm for `StateSpaceMigrate`** (the only refs are the builder + tests). Yet `home_node` is the Space's **authority anchor**: `exchange.rs:629` admits a `state.*` event only when `sender == space.home_node`. So cutover must, on apply, mutate `SpaceState.home_node` source→destination — **flipping the authority anchor mid-DAG**, via a `state.space_migrate` event the *source* node signs (`build_space_migrate_event` signs with the source key). This raises three design questions for DESIGN (AF-D# candidates), none mechanical:
  - **Authority-transfer semantics:** the source's signature is valid *because it is still home_node at the moment it signs*; applying the event makes the destination home_node going forward. The applier must accept the migrate event under the *old* anchor and install the *new* one atomically.
  - **Convergence (M8):** unlike Arc G's set-once `jurisdiction`, `home_node` **mutates**. Convergence rests on the cutover being a single causally-ordered DAG event (D-076 causal order) whose `prev_events` seed the current tip — needs confirming that `derive_resolved` orders post-cutover events under the new anchor deterministically across replicas.
  - **Post-cutover rejection window:** events signed by the old home_node after cutover, or events arriving at the source after it has handed off, must be handled (forward to new home / reject) — the spec's `transport.redirect` to members is the client-facing half.

**AF-A5 (destination admission stubs — keep dormant, don't fake; fork 2 sibling).** `handle_migration_propose` is "Phase-2: accept unless already-hosting"; the `InsufficientStorage`/`VersionIncompatible`/`PolicyRejected` errors (6003/6004/6005) are *defined but never evaluated*. Real storage-capacity/version introspection is operator infra. → Keep accept-unless-hosted; leave 6003/6004/6005 as **dormant node-policy hooks** (the G/PG-13 honest-no-op posture), don't fabricate capacity checks.

**AF-A6 (retention rule — no teardown today; honest by accident, to be made honest by design).** Nothing deletes a source Space DB on cutover (no teardown code exists), so there is no premature-delete bug — but the §3.12 retention *rule* is also unimplemented. → Retention is a node-side, **operator-gated** teardown that fires only after `migration.verified`; the core never auto-deletes. Dormant-but-correct.

**AF-A7 (operator surface absent).** `migrate-start` was deferred at M7C; no admin/AI verb initiates a migration. → An `admin_ops` verb (source-side `migration.request` originator), sibling to the existing federation/node verbs; rides the `--aicontrol`/`--batch` surfaces via the shared command layer.

## §4 — Scope fence (named homes, STOP on drift)

- **IN:** full node-side driver for all 12 messages + both state-machine sides + EventStore export/import + cutover applier (`home_node` flip) + retention gate + federation-notify + operator initiate verb.
- **OUT — real storage-capacity / version / policy introspection:** dormant hooks only (AF-A5).
- **OUT — auto-deletion of the source DB:** operator-gated teardown only, never automatic (AF-A6).
- **OUT — client-side redirect UX** (`transport.redirect` member handling beyond emitting it): client/UI milestone.
- **OUT — cross-major-version migration negotiation** beyond the `VersionIncompatible` error path.

## §5 — Verdict

PG-11 = **GAP-CONFIRMED** (AF-A1). Shape: **wire a built-and-tested core state machine into a node driver** + plumb EventStore export/import (AF-A3, no trait gap), the retention gate (AF-A6), federation-notify, and the operator verb. The genuinely novel design work is **not** the wiring but **AF-A4 — the authority-anchor (`home_node`) flip at cutover**: there is no applier for it, it mutates the field that gates `state.*` authority, and its convergence + authority-transfer semantics need an AF-D# lock. Medium arc; AF-A4 is the design crux DESIGN must resolve.

No DECISIONS change proposed at audit stage (AF-D# arc-local pending DESIGN, D-069). Doc-only — suite unchanged at J-250's 1121/0/2, not re-run.
