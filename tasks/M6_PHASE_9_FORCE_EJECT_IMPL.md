# M6 Phase 9 — A4 force-eject + node-unban (`membership.node_eject` / `node_unban`)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-30  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

The last M6-scoped admin verb: A4 `space force-eject` (A4-D1), plus its reversible
counterpart `space unban` (added by the 1A lock). This is the only M6 verb that
emits a Space-DAG event, so it ran as a **design beat first** (J-158/J-159), then
implementation. Authoritative spec: design §6.A4 + the A4-D1 sub-design (this file).

## Design locks (Joe-locked at the A4-D1 beat)

- **EventTypes:** new `membership.node_eject` + `membership.node_unban` — Node-
  signed (sender = home Node keypair), Space-scoped, non-root DAG events,
  content `{ target_identity, reason? }`.
- **Authority (the deferred signing-identity question):** signature + `sender ==
  space.home_node`, NOT member role. Reuses the existing Node-authored-event
  pattern (B3): both are added to `skip_membership` and to the **node-authored
  sender-registration skip** (the Node keypair is not a registered Identity), with
  a dedicated post-signature gate. A forged eject from any other signer →
  `ExchangeError::NodeEjectAuthority` (wire **3043**). Self-contained: `home_node`
  is replicated Space state, so federated peers validate identically.
- **1A — eject + ban + reversible.** `node_eject` removes the target (+ rooms +
  pending invites) **and** bans (`SpaceState.banned`); `node_unban` lifts the ban
  (rejoin allowed; membership not auto-restored). **Gate that 1A passed:** an
  unban path did NOT exist (`banned` was insert-only) — so "reversible" was added
  as a real mechanism (`node_unban`), not assumed. Resolution: `node_eject` gets
  **top Layer-1 precedence** (Node authority supersedes member governance — a
  concurrent member join/kick can't defeat an eject); `node_unban` takes no
  dominance (eject→unban is causal, resolved by DAG/timestamp order).
- **2A — dedicated `NodeEjectAuthority` + wire 3043** (not reused `PermissionDenied`).
- **Option A propagation (J-159).** The verb dispatches (live in-memory state
  updates immediately → target removed+banned, auth gate enforces at once) +
  persists to the event store. Connected clients / federated peers pick it up via
  the **existing sync path**, not a live push (honest, D-065; sibling to A1
  `defederate`'s no-network-goodbye). A `process_inbound` refactor for live
  fan-out (Option B) is a deliberate future follow-up.

## What shipped

**Core (xgen-core):**
- `wire.rs`: `MembershipNodeEject` / `MembershipNodeUnban` variants (+ as_str/from_str).
- `space/state.rs`: `apply_node_eject` (remove + ban) / `apply_node_unban` (lift); dispatch arms.
- `message/exchange.rs`: both added to `skip_membership` + the node-authored sender-registration skip; `ExchangeError::NodeEjectAuthority` (wire 3043); the post-signature `sender == home_node` gate.
- `resolution/{state_key,algorithm}.rs`: target-keyed membership state + `node_eject` top precedence.

**Node (xgen-node):**
- `admin_ops.rs`: `space_force_eject` (SPACE_8001 not-hosted / 8002 not-member / 8003 already-removed / 8004 dispatch-persist) + `space_unban` (8001/8003); both build+sign the Node event with current tips, dispatch `LocallySubmitted`, persist via `app::persist_event`, audit (DESTRUCTIVE, `correlation_id` = emitted `event_id`). `app::resolve_spaces_dir` helper. clap `SpaceCommand::{ForceEject,Unban}`; pipe dispatch arms.

**Docs (Chat-Claude-owned cross-file touches):** Ch3 §3.3 EventType registry (×2) + §3.9 error table (3043), v0.4→0.5; Appendix I event table (×2), v1.4→1.5.

## Validation-path fix (caught by the wire-gate test)

The wire reject test surfaced that the F-10 unknown-signer `HeldPending` (sender
not a registered Identity) fired for the Node keypair before the authority gate.
Fixed: `node_eject`/`node_unban` skip the sender-registration hold (Node-authored,
exactly like federation_add). Real validation-path fix, not just a test fix.

## Verification (close)

- `cargo test --workspace`: **724** passed / 0 failed (699 lib: 63 client + 35 common + 469 core + 132 node; + 25 integration). +6 vs Phase-10's 718 (+4 core: 3 apply + 1 wire-gate; +2 node verb tests).
- clippy `--workspace --lib --tests --all-features -- -D warnings`: clean. build `--workspace --all-targets`: 0 errors.

## Reserved for Joe

The canonical **design-doc §5.1 / §6.A4** amendments (force-eject + unban shipped;
A4-D1 sub-design closed) + the four D-071 arc stubs. (Ch3/Appendix I spec touches
were Chat-Claude-owned and are done.) Option B (live fan-out) SHIPPED J-160 — see
the Option B section below; the canonical §6.A4 note (Option A → Option B) stays
with Joe's §5.1/§6.A4 amendments.

## Option B — live fan-out — SHIPPED (J-160)

The deliberate follow-up to Option A. When `space force-eject` / `space unban`
dispatch + persist from the admin pipe, they now **also** push the Node-authored
`membership.node_eject` / `node_unban` LIVE — fan-out to the Space's connected
member clients + a federation push to the Space's federated peers — the same path
a client-submitted event takes (`process_inbound` → `apply_fanout` +
`apply_federation_push`). Sync remains the backstop for offline/lagging peers.

- **Wiring (mirrors the runtime/federation_registry threading).** `AdminContext`
  gains `client_senders` + `federation_peer_senders` (`Option`, with
  `with_client_senders` / `with_federation_senders` builders). `start_pipe_server`
  threads the two `Arc`s it now holds through `dispatch_line` → `dispatch_admin`,
  which attaches them to the ctx. `None` (file-only verbs / unit tests) → sync-only
  (the Option-A baseline is preserved).
- **Hook point.** `emit_node_membership_event` (the shared eject/unban helper):
  after persist it builds a `FanoutRequest { event, new_joiner: None }` and calls
  `apply_fanout` (author = the Node id projected to `IdentityXgid`, used only to
  exclude — the Node is never a client recipient) + `apply_federation_push`
  (`LocallySubmitted` → eligible per F-5).
- **Best-effort after persist (D-070 honesty).** A fan-out/push failure does NOT
  roll back the eject — the event is already in the DAG + on disk.
- **Recipient nuance (honest, D-065).** `apply_fanout` collects recipients from the
  Space's *current* members, and `dispatch_event` already removed the target. So
  the ejected target's own session is NOT in the live push — it learns via sync,
  exactly as for a member-initiated kick. Remaining members + federated peers get
  it live. (The handoff's plan-step-3 expectation that the target's own session
  receives it was corrected here against the actual `apply_fanout` semantics.)
- **Verification.** +2 node lib tests (live fan-out to a remaining-member client +
  federation peer for eject AND unban; sync-only-without-senders). `cargo test
  --workspace`: **726** passed / 0 failed (701 lib: 63 client + 35 common + 469
  core + 134 node; + 25 integration). clippy clean; build all-targets 0 errors.

## M6 status after this phase

**M6's admin write-path is COMPLETE** — all backed verbs shipped (A6 5 + A5 4 +
A1 2 + A4 3 [`list-hosted` + `force-eject` + `unban`] + A7 2 = **16 verbs**). The
~18 verbs designed against absent subsystems remain routed to four post-M6 D-071
arcs (federation-admin-control, bootstrap-client, auth-module-registry,
protocol-audit-log) + node-policy.

---

*End of Phase 9 (A4 force-eject) plan.*
