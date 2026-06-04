# XGen Protocol — Arc F (Space Migration Subsystem, PG-11) Design
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Frame

Design for PG-11, backed by `ARC_F_MIGRATION_AUDIT.md` v1.0. Scope locked in discussion (2026-06-04): **wire the whole state machine, no slice**. The arc is mostly wiring a built-and-tested core (`xgen-core/src/migration/`, ~733 lines) into a node driver; the one novel design problem is **AF-A4 — the `home_node` authority-anchor flip at cutover**, resolved at §2 AF-D1/D2. AF-D# are arc-local pending close (D-069).

## §2 — Locked design decisions (AF-D#)

**AF-D1 — Cutover applier (the missing arm).** Add `EventType::StateSpaceMigrate => self.apply_space_migrate(event)` to `SpaceState::apply_event` (currently absent — falls through today). `apply_space_migrate` sets `self.home_node = NodeXgid(content["destination_node_id"])`. **Validate-under-old / apply-installs-new:** the dispatch path validates before it applies, and `exchange.rs:629` admits a `state.*` event iff `sender == space.home_node`; the migrate event is signed by the source Node, which *is* the current `home_node` at sign time, so it validates under the old anchor — then the applier installs the new one. Atomic within the single event application. **Idempotent:** if `home_node` already equals `destination_node_id`, the applier is a no-op (re-delivery / replay safe).

**AF-D2 — Authority transfer is self-protecting; no convergence conflict class.** After the flip, the source Node is no longer `home_node`, so any second or competing source-signed `state.space_migrate` fails the `sender == home_node` check — the authority gate itself forbids conflicting cutovers. Cutover is therefore a **causally-terminal singleton**: the sequential state machine produces exactly one, and its `prev_events` seed the current DAG tip (D-076 causal order), so every replica orders it identically. **No `state_key_for_event` arm** (home_node is not a last-writer-wins field; it flips once under causal order, not via key-keyed resolution). DESIGN ships a **convergence pin**: post-cutover events replay deterministically under the new anchor across permuted arrival (rides M8 `derive_resolved`).

**AF-D3 — State-transition helper lands in core (pure).** The `MigrationState` enum exists but **no transition function does** (AF-A2). Add a pure `fn transition(current: &MigrationState, msg: MigrationMsgKind) -> Result<MigrationState, MigrationError>` to `state_machine.rs` enforcing IDLE→Negotiating→Transferring→Verifying→Complete/Failed and emitting `WrongState` (6006) on an out-of-sequence message. Keeps the sequencing logic pure + unit-testable (sibling to the existing handlers); the node *owns the storage* of current state (AF-D4), the core *owns the rule*.

**AF-D4 — Node driver shape.** xgen-node gains: a per-Space `MigrationState` map (source side + destination side independent); dispatch of the 12 wire messages → the existing pure handlers via `transition()`; transport send/recv (`connection.rs:156 send_migration` is the send primitive); commit of the cutover `state.space_migrate` event to the DAG. **EventStore bridge (AF-A3, no trait gap):** source export = `EventStore::range(0)` → `batch_events`; tail-during-transfer = `identify_tail` (built); destination import = `append` into a **freshly instantiated per-Space store**, then rebuild the SQLite materialization cache (J-232). 

**AF-D5 — Retention gate (operator-gated, never auto).** On `migration.verified` the source **retains** its Space store; the core never auto-deletes (AF-A6). Post-cutover the Space is naturally inert at the source (its `home_node` now points away, so it accepts no new authoritative events). Actual teardown is a separate explicit operator action — not part of the migration flow. Dormant-but-correct (D-065).

**AF-D6 — Destination admission stays dormant (don't fake introspection).** `handle_migration_propose` keeps accept-unless-already-hosting; `InsufficientStorage`/`VersionIncompatible`/`PolicyRejected` (6003/6004/6005) remain **defined-but-dormant node-policy hooks** (AF-A5) — real storage-capacity/version introspection is operator infra, out of scope. Wiring them now would fabricate guarantees the core can't keep.

**AF-D7 — Operator surface.** An `admin_ops` migration-initiate verb (source-side originator of `migration.request`) via the shared command layer, reaching both `--batch` and `--aicontrol` (sibling to the federation verbs). Exact name/args = CP-3.

**AF-D8 — Federation-notify + peer home_node update.** Federated peers must learn the new `home_node`. Two candidate mechanisms: (a) the existing federation push already propagates `state.*` events, so peers receive the `state.space_migrate` event and apply the **same** `apply_space_migrate` flip (AF-D1) — making `MigrationFederationNotify` a courtesy/redundant signal; or (b) `MigrationFederationNotify` is the authoritative peer-update message. Ground which at C2 (CP-2); lean (a) with notify as courtesy, since the DAG event is already the source of truth and re-using the applier keeps one code path.

## §3 — Confirm-at-pickup (D-078)

- **CP-1 (C1)** — `transition()` signature + the `MigrationMsgKind` discriminant it switches on (reuse `EventType`/a small enum?); confirm it composes with the existing handlers without duplicating their checks.
- **CP-2 (C2)** — does the existing federation push deliver `state.space_migrate` to peers (so their applier flips `home_node`), making notify a courtesy (AF-D8a), or is notify authoritative (AF-D8b)? Lock the one path.
- **CP-3 (C2)** — operator verb name/args + surfaces (`--batch`/`--aicontrol`).
- **CP-4 (C2)** — destination fresh per-Space store instantiation + SQLite cache rebuild exact calls (J-232 plugin API).

## §4 — Commit plan (feeds the runbook)

- **C1 — core completion + cutover applier (xgen-core, lib-clean).** Pure `transition()` in `state_machine.rs` + `apply_space_migrate` arm (`home_node` flip, idempotent, AF-D1) + AF-D2 self-protection. Tests: transition sequence + `WrongState`; cutover flips `home_node`; idempotent re-apply; **convergence pin** (post-cutover events order identically under permuted replay); a second source-signed migrate post-cutover fails authority (AF-D2).
- **C2 — node driver (xgen-node).** Dispatch of 12 msgs + per-Space `MigrationState` ownership + transport send/recv + EventStore export/import bridge (CP-4) + retention gate (AF-D5) + federation-notify (CP-2) + operator verb (CP-3) + dormant admission hooks (AF-D6). ch4 handler-presence reconcile. Tests: two-node end-to-end migration (integration), retention-after-verified, post-cutover stale-source rejection.
- **Close — D-074 doc-only.** ch3 §3.12 / ch4 reconcile (handlers now present) + gap-audit §5 PG-11 ✅ (Open **2/13** — PG-02/05) + ROADMAP + JOURNAL + AF-D# promotion eval + **the deferred Appendix K federation-block reconciliation carry-in recorded into the Round-2 whole-codebase audit home** (its canonical landing, per the Arc-G-close hand-off).

## §5 — Honesty posture (D-065)

The migration *mechanism* ships real and tested end-to-end. What is honestly **not** delivered: real storage-capacity/version/policy admission (dormant hooks, AF-D6) and automatic source teardown (operator-gated only, AF-D5). The authority-anchor flip is real and self-protecting (AF-D2) — no fake atomicity, no central coordinator; correctness rests on the existing signature + `home_node` authority check, now reading the post-cutover anchor.

No DECISIONS change proposed (AF-D# arc-local pending close, D-069). Doc-only — suite unchanged at J-250's 1121/0/2, not re-run.
