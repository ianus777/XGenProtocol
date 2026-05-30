# HANDOFF — M6 A4 force-eject Option B (live fan-out)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-30  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Jump-on-it summary

M6 A4 force-eject shipped at **J-159** (commit `8fc37ea`) as **Option A**:
`space force-eject` / `space unban` build + sign the `membership.node_eject` /
`membership.node_unban` event with the Node keypair, `dispatch_event` (live
in-memory state → target removed + banned at once), and persist to the on-disk
space log. **Propagation is sync-only** — connected clients and federated peers
pick the event up on their next sync, NOT via a live push.

**Option B = add the live push.** When a force-eject/unban is dispatched from the
admin pipe, also fan it out immediately to currently-connected clients of the
Space AND push it to the Space's federation peers — the same way a client-
submitted event propagates through `process_inbound`. Joe-locked this session as
a deliberate follow-up (J-159). This is the whole task.

**Read first (Rule 0):** CLAUDE.md PLAY block + JOURNAL J-159, then this file.

## Option-A baseline (what exists today — do not re-do)

`xgen-node/src/admin_ops.rs`:
- `space_force_eject` (and `space_unban`) do their pre-checks (`SPACE_8001` not
  hosted here / `SPACE_8002` not a member / `SPACE_8003` already gone-or-banned),
  then call the shared helper **`emit_node_membership_event(ctx, space_id,
  EventType::MembershipNodeEject|NodeUnban, content) -> Result<event_id, AdminError>`**
  which builds the event (Node keypair + current Space tips), signs it,
  `dispatch_event(EventOrigin::LocallySubmitted, None)`, persists the accepted
  event + `additional_persisted` to the on-disk space log (`SPACE_8004` on persist
  failure), and returns the `event_id`. Audited DESTRUCTIVE with
  `correlation_id = event_id` (`record_action_correlated`).
- **`emit_node_membership_event` is exactly where Option B hooks the fan-out** —
  it currently returns after persist with no `apply_fanout` / federation push.
- `AdminContext` already carries `runtime: Arc<Mutex<NodeRuntime>>` +
  `federation_registry` + `identities_path()` + `spaces_dir()`, threaded from the
  pipe server via `start_pipe_server` → `dispatch_line` → `dispatch_admin`.
- The dispatch helper returns **after persist; there is no `apply_fanout` call** —
  that's the gap Option B closes.

## The fan-out machinery to reuse (xgen-node/src/fanout.rs + app.rs)

- `fanout.rs`: `ClientSenders` = `Arc<Mutex<HashMap<IdentityXgid, mpsc::Sender<OutboundMsg>>>>`
  (fanout.rs:54); `FederationPeerSenders` = `Arc<Mutex<HashMap<NodeXgid, mpsc::Sender<OutboundMsg>>>>`
  (fanout.rs:73); `FanoutRequest { event: Option<Event>, new_joiner: Option<..> }`
  (fanout.rs:79); `pub async fn apply_fanout(req: FanoutRequest, author_id:
  &IdentityXgid, runtime: &Arc<Mutex<NodeRuntime>>, client_senders: &ClientSenders)`
  (fanout.rs:128) — it broadcasts `req.event` to all Space members **except
  `author_id`**. Federation push is the Phase-4 path (`compute_federation_delta_for_space`
  + push through `FederationPeerSenders`); see app.rs ~1103 for the call shape next
  to `apply_fanout`.
- `app.rs::process_inbound(...) -> FanoutRequest`: the canonical dispatch + persist
  + build-FanoutRequest path for a client-submitted event; its caller then calls
  `apply_fanout(...)` (~app.rs:1088) and the federation push path (Phase 4,
  `apply_federation_push` / `FederationPeerSenders`, F-5 origin-gated — note
  `LocallySubmitted` events DO get pushed to peers; `ReceivedViaFederation` do not).
- `client_senders` + `federation_peer_senders` are created in `run_node`
  (~app.rs:514 / 519) and threaded to `handle_connection`, but **NOT** to
  `start_pipe_server` today (the pipe server gets `runtime` + `federation_registry`
  only — see the spawn site ~app.rs:722).

## Plan (mirror the runtime/federation_registry threading already in place)

1. **Thread the sender maps to the admin layer.** Add `client_senders:
   Option<ClientSenders>` + `federation_peer_senders: Option<FederationPeerSenders>`
   to `AdminContext` + `with_client_senders`/`with_federation_senders` builders
   (same shape as `with_runtime`/`with_federation_registry`). Extend
   `start_pipe_server` (add 2 params; it's already `#[allow(too_many_arguments)]`),
   the spawn call site in `run_node` (clone the existing Arcs, like
   `pipe_runtime`/`pipe_federation_registry`), and `dispatch_line` /
   `dispatch_admin` to pass them through and attach to the ctx.
2. **Reuse the fan-out path (D-067 no-drift).** Have `emit_node_membership_event`
   build a `FanoutRequest { event: Some(accepted_event), new_joiner: None }` after
   persist and call `apply_fanout(req, &author_id, &runtime, &client_senders)` +
   the federation push (mirror app.rs ~1088/1103). Prefer factoring the dispatch +
   persist + build-FanoutRequest + push steps shared with `process_inbound` into one
   helper so the two callers don't drift; if a full `process_inbound` extraction is
   too invasive for this pass, the minimal version is to construct the
   `FanoutRequest` + call `apply_fanout`/federation-push directly in
   `emit_node_membership_event` (it already holds the accepted event).
3. **`author_id` for `apply_fanout`.** The event is Node-authored, so the Node is
   not a client recipient — pass the Node's id (or any non-member id) as
   `author_id`; `apply_fanout` only uses it to *exclude* the author, and the Node
   isn't in `ClientSenders`, so all client members (including the ejected target's
   own session, which is the point — they should see they were ejected) receive it.
   Confirm this is the desired behaviour for the target's own session.
4. **Federation push.** Trigger the Phase-4 federation push for the node_eject so
   federated peers of the Space get it live (LocallySubmitted → eligible per F-5).
5. **Keep the verb result semantics** (D-070 honesty): return `event_id` after
   persist; fan-out/federation are best-effort after persist (a fan-out failure
   does not roll back the eject — log it).

## Verification

- New NodeRuntime/integration test: a `space force-eject` (and `space unban`)
  pushes the `membership.node_eject`/`node_unban` to a registered client sender
  AND a federation peer sender — not just persists. Mirror the Phase-9
  `phase9_*`/`federation_push_integration` harness patterns.
- `cargo test --workspace` stays green (baseline **724** at J-159: 699 lib —
  63 client + 35 common + 469 core + 132 node — + 25 integration); clippy
  `--workspace --lib --tests -D warnings` clean; build `--workspace --all-targets`
  0 errors.

## On close

JOURNAL J-160 + CLAUDE PLAY flip + ROADMAP bump + update
`tasks/M6_PHASE_9_FORCE_EJECT_IMPL.md` ("Option B live fan-out — SHIPPED") +
design §6.A4 note (Option A → Option B). Flip this handoff Status → COMPLETED.
Folded commit per the M6 cadence; **Joe pushes** (never push directly —
[[feedback_push_convention]]). Strict `Last updated` = bare date
([[feedback_last_updated_strict]]).

## Entry-point reading order

1. CLAUDE.md PLAY block + JOURNAL J-159 (Rule 0).
2. This file.
3. `xgen-node/src/admin_ops.rs` — `space_force_eject` / `space_unban` /
   `emit_node_membership_event` (the Option-A baseline + where fan-out hooks).
4. `xgen-node/src/fanout.rs` — `apply_fanout` signature + `FanoutRequest` +
   `ClientSenders`/`FederationPeerSenders` + exclusion semantics.
5. `xgen-node/src/app.rs` — `process_inbound` (extract target) + the
   `apply_fanout` call site + the federation push path + `run_node` sender
   creation + the `start_pipe_server` spawn site.

---

*Everything else in M6 is complete (admin write-path = 16 verbs, J-159). This is
the one deliberate follow-up. The four D-071 subsystem arcs + node-policy + Joe's
design-doc §5.1/§6.A4 amendments are separate, not part of Option B.*
