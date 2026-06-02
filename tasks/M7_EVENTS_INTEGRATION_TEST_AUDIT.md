# M7-events `.events` integration test — Phase-0 Audit (reality map)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Phase-0 audit for the **client↔node `.events` integration test** — the one piece deferred (not failed) across the whole M7-events arc. It was flagged at C5 (J-211) and again at the arc close (J-212): the client `.events` happy path was shipped **component-tested, not end-to-end**, because `connect_url` is concrete and the full `subscribe → second WS → forward` path was never run against a live Node.

This audit grounds (a) exactly what is and isn't covered today, (b) what test scaffolding already exists to reach the gap, and (c) the decisions the design phase must lock. Doc-only — no code, no test written. Arc-local decision IDs `EIT-D#` per D-069.

---

## 1 · The gap — what is tested vs not (grounded)

Both pipe surfaces are **unit-tested in isolation**:

- **Node** (`xgen-node/src/events_pipe.rs` `mod tests`): `parse_subscribe` ×5, `events_pipe_name`, and two duplex handler tests — `subscribe_then_stream_then_prune_on_close` (pushes an `OutboundMsg::Event` **manually** into the observer channel) and `malformed_subscribe_errors_and_registers_nothing`.
- **Client** (`xgen-client/src/events_pipe.rs` `mod tests`): `parse_subscribe` ×3, `events_pipe_name`, `forwardable` ×3, and two duplex pre-WS paths — `nodes_filter_on_client_is_bad_argument` and `not_ready_when_no_identity`.

What is **never exercised** — the live seam:

1. **Node seam:** a real ingest/dispatch → `apply_fanout` → the `node_observers()` registry → the `.events` handler drain → JSONL out. Today the handler test *injects* into the channel; nothing drives a real event through `apply_fanout` into a registered observer.
2. **Client seam (the flagged gap):** `subscribe` → open the **second same-identity WS** (`connect_url` + `client_authenticate`) → tail it → `forwardable` → JSONL out. The concrete-WS half has **zero** coverage; every client handler test returns before the WS opens.

**Grep-verified (J-081):** no `.events` integration test exists. Every repo reference to `start_events_server` / `events_pipe` / `handle_events_connection` / `aicontrol.events` is one of the two source files or a resident spawn site (`xgen-node/src/{app,lib}.rs`; `xgen-client/src/{service,desktop,ai_service,aicontrol,lib}.rs`).

---

## 2 · Reality map — scaffolding that already reaches the gap

- **Integration tests live in `xgen-node/src/tests/*.rs`** (registered via `mod.rs`), not an external `tests/` dir. The suite already brings up **real in-process WS Nodes**: `federation_integration.rs`, `federation_push_integration.rs`, `identity_integration.rs`, `smoke.rs`, `reconnect_integration.rs`, and `phase9_harness.rs` all carry real-WS markers (`TcpListener` / bind / `127.0.0.1:0` / `ws://127`). `xgen-node/src/transport/server.rs` is the WS server; `identity_integration.rs`/`smoke.rs` already drive `server_authenticate` ↔ `client_authenticate` over a local WS.
- **Client side** has a cross-crate integration home already: `xgen-client/tests/sync_safety_net.rs` (+ `precedence.rs`, `quiet.rs`) brings up a Node and a Client together. `xgen-client/src/session.rs` carries the `connect_url` / `client_authenticate` real-WS markers.
- **Both `handle_events_connection<S>` are generic over the stream.** The named-pipe transport is `#[cfg(windows)]`, but the handler can be driven directly over `tokio::io::duplex` (the J-086 pattern the existing handler tests already use). **So the pipe layer does not need a real named pipe** — the test feeds `subscribe` over a duplex and reads JSONL back, while the *WS* side is real.

Net: the seam is reachable **in-process** by reusing existing harnesses. No new infrastructure, no two-process orchestration required.

---

## 3 · Two approaches

**Option A — in-process, real-WS, duplex-driven handler (recommended).** Reuse the `xgen-node/src/tests` real-WS harness (node side) and the `sync_safety_net` node+client harness (client side). Drive each `handle_events_connection` over a `duplex`; the WS half is a real local Node. Deterministic with bounded `tokio::time::timeout`; cross-platform (no named pipe); small.

**Option B — full two-process Windows e2e.** Launch real `xgen-node.exe` + `xgen-client.exe` residents, talk to the actual `.events` named pipes. Highest fidelity, but Windows-only, slow, flaky, and outside the established in-process test idiom. **Rejected for v1** — keep as a possible later smoke if a real end-user pipe regression ever surfaces.

Recommendation: **Option A.**

---

## 4 · Proposed shape (for the design phase to confirm)

Two tests of unequal value:

- **Client seam test — the real gap, priority 1.** Fixture: a real `data_dir` with keypair (`xgen-client_keypair.enc`) + client_state whose `home_node` points at a live local Node WS, and Space membership so the Node fans the event to this Identity (entitlement is the ceiling). Steps: spawn the Node; run the client `handle_events_connection` over a duplex; send `subscribe`; cause the Node to emit an event in the member Space; assert it arrives as bare `Event` JSONL on the duplex. Negative companions: an event the filter excludes is **not** forwarded; an event in a non-member Space is **not** received.
- **Node seam test — the join, priority 2.** Drive a real event through the Node so `apply_fanout` pushes to a registered observer handler (duplex-backed), asserting ingest→fanout→JSONL as one flow rather than the two halves the C3 + C4 tests cover separately. `#[serial_test::serial(node_observers)]` (the registry is a process-global `OnceLock`).

Known trivial impl detail (not a decision): `handle_events_connection` is module-private in both files; a cross-module `src/tests/` test needs `pub(crate)`, an external `tests/` test needs `pub`.

---

## 5 · Open decisions for the design phase (`EIT-D#`, Joe-lock)

1. **EIT-D1 — test homes.** Node seam → `xgen-node/src/tests/events_pipe_integration.rs` (+ `mod.rs`). Client seam needs both a Node and the client handler → `xgen-client/tests/` (reuse `sync_safety_net` shape, `pub` visibility) **or** a new `xgen-client/src/tests/` mirroring the node convention. Cross-crate placement is the real call.
2. **EIT-D2 — assertion scope.** Minimal viable = client happy path + the two negatives in §4. Confirm whether `unsubscribe`/prune and observer-WS-loss exit are in v1 or deferred.
3. **EIT-D3 — membership fixture.** How the test establishes the client as a member of a Space on the Node (reuse an existing harness builder vs a purpose-built one). Drives most of the test's size.
4. **EIT-D4 — visibility bump.** `pub(crate)` vs `pub` on `handle_events_connection`, per EIT-D1.
5. **EIT-D5 — determinism.** Bounded `timeout` waits, no sleeps; confirm the fan-out-to-observer delay is observable deterministically (the federation_push harness is the precedent).

---

## 6 · Proposed mini-roadmap

Audit (this doc) → **design-lite** (lock EIT-D1–D5 — likely one short session, the decisions are light) → **runbook** (may fold into design given size) → **implementation** (candidate 2 commits: client seam test, then node seam join test) → **close** (D-074 atomic: this doc + design → COMPLETED, ROADMAP `.events`-test row, CLAUDE PLAY, JOURNAL). Test-only — no production code beyond the EIT-D4 visibility bump.

---

## 7 · Definition of Done (candidate)

- Client seam happy path + the two negatives pass deterministically in-process.
- Node seam ingest→fanout→observer→JSONL join test passes (serial on `node_observers`).
- `cargo test --workspace` green (new count recorded at close); build all-targets 0/0; clippy `-D warnings` clean.
- Canonical docs updated atomically per D-074; `docs/xgen_aicontrol_implementation.md` §3 C5 component-test-boundary note updated to "closed by integration test".
- The arc-named deferral ("client↔node `.events` integration test") struck from the M7-events / completion deferral lists.
