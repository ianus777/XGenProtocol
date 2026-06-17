# XGen Protocol — Arc F (Space Migration Subsystem, PG-11) Implementation Runbook
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Frame

Runbook for PG-11 (arc F), gated by `ARC_F_MIGRATION_AUDIT.md` v1.0 + `ARC_F_MIGRATION_DESIGN.md` v1.0 (AF-D1–D8 Joe-locked 2026-06-04; AF-D8 = lean (a), reuse the applier on peers, confirm at C2). Two commits + a doc-only close.

**Shape:** wire a built-and-tested core (`xgen-core/src/migration/`) into a node driver. The one novel piece is the `home_node` authority-anchor flip at cutover (AF-D1/D2) — it lands in C1 with its own convergence + authority tests. AF-D# arc-local (D-069).

**Honesty (D-065):** mechanism ships real + end-to-end tested. Dormant by design: destination admission checks 6003/6004/6005 (AF-D6); automatic source teardown — operator-gated only, never auto (AF-D5).

**One writer per file per commit.** C1 is xgen-core (lib-clean); C2 is xgen-node. Clair owns the Rust; Joe pushes; Claude commits but never pushes.

## §2 — C1: core completion + cutover applier (xgen-core, lib-clean)

Goal: the pure state-machine sequencing + the cutover applier exist and are proven, with no node driver yet.

**Steps.**
1. **`xgen-core/src/migration/state_machine.rs`** — add a pure `fn transition(current: &MigrationState, msg: MigrationMsgKind) -> Result<MigrationState, MigrationError>` enforcing Idle→Negotiating→Transferring→Verifying→Complete/Failed; out-of-sequence ⇒ `WrongState` (6006). **CP-1 LOCKED (C1):** `MigrationMsgKind` = a small local enum (`Request`/`Propose`/`Accept`/`Reject`/`TransferComplete`/`Verified`/`VerificationFailed`/`Failed`), **not** `EventType` — the 12 migration messages are transport messages (only `state.space_migrate` is a DAG `EventType`), so reusing `EventType` would mis-model them. `transition` guards sequence only; the payload checks stay in the existing handlers (no re-doing).
2. **`xgen-core/src/space/state.rs`** — add `EventType::StateSpaceMigrate => self.apply_space_migrate(event)` to `apply_event` (currently falls through). Implement `apply_space_migrate`: set `self.home_node = NodeXgid(content["destination_node_id"])`; **idempotent** (no-op if already equal); **no `state_key_for_event` arm** (AF-D2 — causally-terminal singleton, not LWW-keyed).
3. Confirm the validate-before-apply ordering holds on the dispatch path so the migrate event validates under the *old* `home_node` (`exchange.rs:629`) before the applier installs the new one (AF-D1). No change expected — just assert it in a test.

**Tests (C1).** `transition` happy sequence + each `WrongState` rejection · `apply_space_migrate` flips `home_node` source→dest · idempotent re-apply is a no-op · **convergence pin**: a permuted `derive_resolved` replay including the cutover yields identical `SpaceState.home_node` + identical ordering of post-cutover events · **AF-D2 self-protection**: a second source-signed `state.space_migrate` after cutover fails the `sender == home_node` authority check.

**Gate (C1).** `cargo test -p xgen-core` green (+N over 1121 in scope) · `cargo build --workspace --all-targets` 0 · `cargo clippy --workspace --lib --tests -- -D warnings` clean (default **and** `--all-features`).

## §3 — C2: node driver (xgen-node)

**CP-2/3/4 RESOLVED at C2 pickup (code-traced, not guessed):**

- **CP-2 → AF-D8a (lean a) LOCKED.** `federation_session::apply_federation_push` already delivers any `LocallySubmitted` DAG event to the Space's `federation_nodes` peers (reads `SpaceState.federation_nodes`; F-5 guards only `ReceivedViaFederation`). The source authors `state.space_migrate` as `LocallySubmitted`, so it rides the existing push; each peer's `apply_space_migrate` flips `home_node`. `migration.federation_notify` is courtesy, subsumed by the applier reuse — the federation-notify step does **not** grow. **Required dependency:** peers only *apply* the pushed migrate if it validates as Node-authored — closed by the exchange.rs wiring (the C1 grounding finding), landed in C2.
- **CP-3 LOCKED.** Verb = `migration initiate <space> --destination-id <id> --destination-url <url>` (sibling to `federation initiate`); `admin_ops::migration_initiate` requires the runtime, validates the Space is homed here (`MIG_6010`/`MIG_6011`), and spawns `migration_driver::run_source_migration` detached. Surface: `--batch` pipe arm (`AdminCommand::Migration(Initiate)`); the shared admin command layer also exposes it to `--aicontrol`. Audited (A6 trail).
- **CP-4 LOCKED.** Destination: `NodeRuntime::ensure_store(&dest_space_id)` (fresh per-Space store via the injected `store_factory` — vanilla or engine), `store.append` per transferred event, then `rehydrate_space_from_store` (rebuilds graph + `SpaceState` via `derive_resolved` over `store.range(0)`). There is **no** separate SQLite "materialization-cache rebuild" primitive — `append` + `range` + `rehydrate_space_from_store` suffice for any engine (the audit's "SQLite rebuild" = `rehydrate_space_from_store`, engine-agnostic via the `EventStore` trait).

**As-built wire code (D-065):** the migrate Node-authority gate (AF-D2) at `validate_event` rejects a non-home-node migrate with new wire **6009 `migration_authority`** (`ExchangeError::SpaceMigrateAuthority`); the applier re-checks defensively. **Supersession (honest, Arc-E pattern):** the C2 commit guessed **6007**, which collided with the spec's existing §3.12.11 `migration_verification_failed`; corrected to **6009** (next free) at close, with the ch3 §3.12.11 table row added.

**Doc reconcile consolidated into the close (D-074):** the ch3 §3.12 / ch4 handler-presence reconcile (this section's step 10 + §4 step 1) lands as one atomic doc-only commit at close, not split across the C2 code commit.

**Steps.**
1. **Dispatch** — route the 12 migration wire messages (`MigrationRequest`/`Propose`/`Accept`/`Reject`/`Failed`/`EventBatch`/`BatchAck`/`TransferComplete`/`Verified`/`VerificationFailed`/`FederationNotify` + the `state.space_migrate` DAG event) to the existing pure handlers, driven through `transition()`.
   - **C1 grounding finding (do not skip):** `state.space_migrate` is **not yet** recognised by `validate_event` (`exchange.rs`). The `node_authored` (≈557), `skip_membership` (≈582), and home-node authority gate (≈624) matches today list only `MembershipNodeEject`/`MembershipNodeUnban`. A migrate dispatched through `validate_event` as-is is rejected at step 10/11 (Node sender unregistered → HeldPending, then NotASpaceMember) and **never reaches** the `sender == home_node` gate. C2 must add `EventType::StateSpaceMigrate` to those three matches (mirroring `node_eject`) so the cutover validates as Node-authored and is gated by `sender == home_node` under the *old* anchor (AF-D1/D2). C1 proves AF-D2 at the applier's defensive gate (`apply_space_migrate`); the wire-gate is C2's to close.
2. **State ownership** — a per-Space `MigrationState` (source side + destination side independent) held on the node; transitions via `transition()`.
3. **Transport** — send/recv via `connection.rs:156 send_migration`.
4. **EventStore bridge (CP-4)** — source export `EventStore::range(0)` → `batch_events`; tail via `identify_tail`; destination `append` into a fresh per-Space store; rebuild SQLite cache.
5. **Cutover** — commit the `state.space_migrate` event to the DAG (built by `build_space_migrate_event`/`handle_verified`); members get `transport.redirect` (emit only; client UX is OUT).
6. **Retention gate (AF-D5)** — on `migration.verified`, source retains its store; no auto-delete. (Teardown is a separate operator action, not wired into this flow.)
7. **Federation-notify (CP-2 / AF-D8)** — per the locked path.
8. **Operator verb (CP-3)** — `admin_ops` migration-initiate (source-side `migration.request` originator), sibling to the federation verbs.
9. **Dormant admission (AF-D6)** — `handle_migration_propose` stays accept-unless-hosting; leave 6003/6004/6005 dormant.
10. **ch4** — reconcile handler-presence (migration handlers now wired). Header refresh.

**Tests (C2).** Two-node end-to-end migration (integration: request→propose→accept→batch+tail→complete→verify→cutover→home_node flipped on both nodes) · retention-after-verified (source store still present) · post-cutover stale-source event rejected · destination rejects already-hosted (6002).

**Gate (C2).** Same green bar as C1; suite up by C1+C2 counts; workspace build all-targets 0 (xgen-node now consumes the wiring).

## §4 — Close (D-074 doc-only)

1. **ch3 §3.12 / ch4** — reconcile: subsystem now implemented (handlers wired, cutover applier present, retention rule honoured). Header refresh.
2. **`tasks/PROTOCOL_GAP_AUDIT.md`** — §5 **PG-11 ✅ DONE** (Arc F); rollup **Open 2 / 13 · Done 10 · NO-GAP 1** (open = PG-02/05); §4-F DONE; Arc-F close note.
3. **`docs/ROADMAP.md`** — Present Arc-F ⚫ CLOSED; live frontier register 10/13. Paired with CLAUDE.md PLAY (same commit).
4. **`JOURNAL.md`** — J-NNN close entry.
5. **Appendix K carry-in** — record the deferred Appendix K federation-block reconciliation (+ sibling doc drift) into the **Round-2 whole-codebase audit** canonical home (its landing, per the Arc-G-close hand-off). Note the `federation show-policy` summary line is already patched, so it is NOT part of the carry-in.
6. **`tasks/ARC_F_MIGRATION_{AUDIT,DESIGN,IMPL}.md`** → COMPLETED v1.1.
7. **AF-D# promotion eval** — AF-D1–D8 expected arc-local (D-069). Record the authority-flip self-protection (AF-D2) + dormant-but-correct posture (AF-D5/D6, D-065).
8. **DECISIONS.md** — no change expected (confirm at eval).

## §5 — Definition of Done

**C1.** `transition()` + `apply_space_migrate` arm (home_node flip, idempotent, no state_key) · C1 tests green incl. convergence pin + AF-D2 self-protection · build/clippy green.

**C2.** CP-2/3/4 resolved + recorded · 12-msg dispatch + per-Space state ownership + transport + EventStore bridge + retention gate + federation-notify + operator verb + dormant admission · ch4 reconcile · two-node e2e green · workspace build all-targets 0 · clippy green.

**Close.** ch3/ch4 reconcile · gap-audit §5 PG-11 ✅ (2/13 open) · ROADMAP+CLAUDE · JOURNAL · Appendix K carry-in recorded into Round-2 home · task docs COMPLETED · AF-D# eval · DECISIONS confirmed.

(Per task convention, "commit pushed" is **not** a DoD item — the `Status: COMPLETED` header + Joe's push are the shipped signal.)
