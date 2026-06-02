# M7-events `.events` integration test — Design-lite (EIT-D1–D5 locked)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Locks the five decisions raised by `tasks/M7_EVENTS_INTEGRATION_TEST_AUDIT.md` (v1.0). Joe delegated the calls ("by your recommendations", 2026-06-02); recorded here as the locked set with the recommended choice and its basis. Arc-local `EIT-D#` per D-069. Test-only; no production code beyond the EIT-D4 visibility bump. The commit plan (§ runbook-fold) is included since the work is two tests + a close.

## §0 · Audit correction (D-065, honest framing)

The audit's §4/§5 assumed the client-seam test would reuse `sync_safety_net` to bring up a **Node + Space membership**. Verified against the tree: **`xgen-client` does not depend on `xgen-node`** (deps: `xgen-core` only), and `sync_safety_net.rs` is a **stub WS server built from `xgen-core::transport` primitives** (auth handshake, then silent) — not a real `NodeRuntime`. Consequence: the client seam needs **no membership fixture**. The client handler forwards whatever `Inbound::Event` arrives, filtered only by its own subscribe `Filter`; entitlement/membership filtering is the Node's job and is exercised on the **node** seam. This narrows EIT-D3 and keeps the client test inside xgen-client's existing dep set. The audit body stays as the as-found record (audit-at-lock-time precedent).

---

## Locked decisions

**EIT-D1 — test homes (symmetric, internal `src/tests/`).**
- Node seam → `xgen-node/src/tests/events_pipe_integration.rs` (+ `mod.rs` registration), reusing the real-WS NodeRuntime harness pattern (`federation_push_integration` / `identity_integration` shape).
- Client seam → `xgen-client/src/tests/events_pipe_integration.rs` (NEW `src/tests/` dir for the client crate + `#[cfg(test)] mod tests;` in `lib.rs`), reusing the `sync_safety_net` `xgen-core`-stub-WS pattern.
- *Basis:* both internal `src/tests/` keeps the handler at `pub(crate)` (no public-API exposure), mirrors the established node convention, and avoids the external-`tests/` `pub` widening. Cross-crate is unnecessary once §0 removes the real-Node requirement on the client side.

**EIT-D2 — assertion scope (tight v1).**
- Client: happy path (`subscribe` → stub-WS auth → emitted `Inbound::Event` → bare `Event` JSONL on the duplex) + one negative (an event the subscribe `Filter` excludes is **not** forwarded). The `nodes`-reject and not-ready paths stay where they already pass (inline unit tests) — not duplicated.
- Node: happy path (real ingest/dispatch → `apply_fanout` → registered observer → JSONL) + one negative (a non-matching event yields no observer push, since `apply_fanout` filters before send).
- `unsubscribe`/prune-on-close (node) and session-count/guard (client) stay covered by the existing inline duplex tests; not re-asserted at integration level. WS-loss exit deferred (not v1).

**EIT-D3 — fixtures.**
- Client: real `data_dir` with keypair (`xgen-client_keypair.enc`) + `client_state` whose `home_node` = the stub server's bound `127.0.0.1:0` URL. Stub WS = the `sync_safety_net` server pattern, extended to **emit one `Inbound::Event` after auth** instead of going silent. **No membership** (per §0).
- Node: reuse a node-harness builder to stand up a `NodeRuntime` with the identity as a member of one Space; drive a real event so `apply_fanout` fires. Membership/fan-out lives here, where it is real.

**EIT-D4 — visibility.** `handle_events_connection` → `pub(crate)` in **both** files (internal `src/tests/` reach). For the client, `mod events_pipe` stays as-is (no `pub mod` needed). Minimal, additive.

**EIT-D5 — determinism.** Bounded `tokio::time::timeout` on every JSONL read (no sleeps). Client stub emits the Event immediately post-auth so `conn.recv()` resolves promptly; node side uses the `federation_push` harness wait pattern. Node test `#[serial_test::serial(node_observers)]` (process-global `OnceLock` registry).

---

## Commit plan (runbook folded in)

- **C1 — client seam test** (the flagged gap, priority 1): NEW `xgen-client/src/tests/events_pipe_integration.rs` + `mod tests;` wiring + client `handle_events_connection` → `pub(crate)`. Stub-WS-emits-Event fixture; happy + filtered-out negative.
- **C2 — node seam join test**: NEW `xgen-node/src/tests/events_pipe_integration.rs` + `mod.rs` + node `handle_events_connection` → `pub(crate)`. Real ingest→fanout→observer→JSONL; happy + non-match negative; serial on `node_observers`.
- **C3 — close (D-074 atomic)**: this doc + audit → COMPLETED; `docs/xgen_aicontrol_implementation.md` §3 C5 component-test-boundary note → "closed by integration test"; ROADMAP `.events`-test deferral struck + version bump; CLAUDE PLAY → next; JOURNAL close entry.

Each code commit: `cargo test --workspace` + `cargo build --workspace --all-targets` + `cargo clippy -D warnings`. No DECISIONS.md change (EIT-D# arc-local, D-069). No Joe-lock checkpoint mid-stream — the locks above fully pin the shape; Rule 3 stop-and-surface still applies if the tree contradicts a lock at pickup.

## Confirm-at-pickup (D-078)

- The exact `xgen-core` server-side auth call the stub WS uses (the `sync_safety_net` server already does it — copy that path, don't invent).
- The node-harness builder that seats a member + drives an event into `apply_fanout` (reuse `federation_push_integration` / `identity_integration` scaffolding rather than a fresh one).
- Whether the client `data_dir` state helper (`load_client_state` / keypair load) has a test-fixture writer already (else write the two files directly).

## Definition of Done

Per audit §7 (unchanged): both seam tests pass deterministically in-process; workspace green (count recorded at close); build 0/0; clippy clean; docs atomic per D-074; the "client↔node `.events` integration test" deferral struck from the M7-events / completion lists.
