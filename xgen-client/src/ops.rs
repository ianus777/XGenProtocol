// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Shared command implementations for all xgen-client dispatchers (M5, D-067).
//!
//! Each function takes `&mut OpContext<'_>` plus command-specific args and
//! returns `Result<<Verb>Result>` where the result struct carries pure data.
//! The CLI dispatcher (`app::cmd_*` shims) formats results for stdout; the
//! pipe dispatcher (`batch::dispatch_line` arms) drives the D-066-frozen
//! `OK\n` / `ERROR: …\n` pipe shape and discards data; a future
//! `--aicontrol` surface (M7) will format the same results as JSONL.
//!
//! Per-verb migrations land as atomic commits per the M5 contract (see
//! `tasks/M5_OPS_REFACTOR.md` §3 and `tasks/BATCH_FLAG_review.md` Chat
//! Claude addendum §7). This file grows by one function per commit until
//! all 13 verbs are routed through it.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid};

use crate::session::SessionState;

/// Per-command execution context. Constructed by each dispatcher per
/// invocation in M5; the same instance will be reused across commands
/// in a persistent `--aicontrol` connection in M7.
pub struct OpContext<'a> {
    pub session: &'a mut SessionState,
    pub data_dir: &'a Path,
    pub node_override: Option<&'a str>,
}

/// F-6b safety-net timeout for `sync_request` callers. Resolved from
/// `[sync].completion_timeout_seconds` in `xgen-client_config.toml` (default 5s).
/// Used by every site that issues a `SyncRequest` and waits for a matching
/// `SyncComplete`.
///
/// Takes `&Path` (not `&OpContext`) so callers can resolve the timeout
/// independently of the `&mut session` borrow that `ensure_connected`
/// requires.
fn sync_completion_timeout(data_dir: &Path) -> tokio::time::Duration {
    let cfg = data_dir.join("xgen-client_config.toml");
    let s = crate::app::load_sync_section(&cfg);
    tokio::time::Duration::from_secs(s.completion_timeout_seconds)
}

/// Ops-internal: load `xgen-client_state.json` if it exists, otherwise
/// construct a minimal default from the already-cached `identity_id`.
/// The single source of truth for "read-or-init state file" across
/// every state-writing `ops::*` function. The pre-M5 keypair-path
/// variant in `app.rs` was deleted in commit 12 (this verb being
/// state-write but no other consumers needing a keypair-load).
fn load_or_default_state(
    data_dir: &Path,
    identity_id: &str,
    home_node: &str,
) -> xgen_common::state::ClientState {
    use xgen_common::{build_info, state::ClientState};
    let path = data_dir.join("xgen-client_state.json");
    if let Ok(s) = std::fs::read_to_string(&path) {
        if let Ok(state) = serde_json::from_str::<ClientState>(&s) {
            return state;
        }
    }
    ClientState {
        identity_id: identity_id.to_string(),
        display_name: String::new(),
        version: build_info::VERSION.to_string(),
        build: build_info::GIT_HASH.to_string(),
        home_node: home_node.to_string(),
        updated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        spaces: vec![],
    }
}

/// Apply the single-event send-confirm policy (MP-F1a F1A-D3/D4 — the §3 table
/// "single" class). Every single-event op calls this with the outcome of
/// [`Connection::send_event_confirmed`](xgen_core::transport::connection::Connection::send_event_confirmed):
///
/// - `Accepted` → proceed (the node validated + durably persisted the event).
/// - `TimedOut` → **warn + proceed**: a lone event's lost-vs-HeldPending is
///   irreducibly ambiguous without a held-signal (F1A-D5), so the op returns
///   ok-unconfirmed rather than failing a possibly-delivered send.
/// - `Rejected { code, reason, event_id }` (this op's own event) → **`Err`** of a
///   structured [`VerbReject`]: the node deterministically rejected it. This is
///   the CP-1 contract change — node rejections finally reach the command/batch
///   layer (closes the MP-R1-D9 / J-081 §5 "batch ops are write-only, rejections
///   invisible" gap). **MP-F5:** the reject is carried as a typed `VerbReject`
///   (not flattened to anyhow text) so the aicontrol layer can surface the wire
///   `code` + `event_id` as structured reply fields rather than burying them in
///   the message.
/// - transport failure (`Err`) → `Err`: the connection broke mid-confirm.
///
/// `verb` names the op (kebab CLI form) for the warning / error message.
fn apply_single_event_confirm(
    outcome: Result<
        xgen_core::transport::connection::EventConfirm,
        xgen_core::transport::connection::TransportError,
    >,
    verb: &str,
) -> Result<()> {
    use xgen_core::transport::connection::EventConfirm;
    match outcome {
        Ok(EventConfirm::Accepted) => Ok(()),
        Ok(EventConfirm::TimedOut) => {
            tracing::warn!(
                verb,
                "event sent but not node-confirmed within the sync-completion timeout — \
                 proceeding (a single event's lost-vs-HeldPending is irreducibly ambiguous \
                 without a held-signal; F1A-D5)"
            );
            Ok(())
        }
        Ok(EventConfirm::Rejected {
            code,
            reason,
            event_id,
        }) => Err(anyhow::Error::new(VerbReject {
            verb: verb.to_string(),
            code,
            event_id,
            reason,
        })),
        Err(e) => {
            Err(anyhow::Error::new(e).context(format!("{verb}: send-confirm transport error")))
        }
    }
}

/// A node rejection of a single-event op, carried **structurally** so the
/// aicontrol layer can surface the wire `code` + `event_id` as reply fields
/// (MP-F5-D1) instead of flattening to free text. The aicontrol `Ok(Err(_))`
/// arm downcasts an op's `anyhow::Error` to this; a non-`VerbReject` anyhow keeps
/// the generic `ClientVerb` path. `Display` is byte-identical to the pre-MP-F5
/// `bail!` text, so the CLI/`{e:#}` message is unchanged.
#[derive(Debug, Clone)]
pub struct VerbReject {
    /// The op's kebab CLI name (e.g. `"join"`).
    pub verb: String,
    /// The node's wire reject code (e.g. 3030 `tier_mismatch`).
    pub code: u32,
    /// The rejected event's id (the correlation key).
    pub event_id: String,
    /// The node's wire reason string.
    pub reason: String,
}

impl std::fmt::Display for VerbReject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} rejected by node (code {}): {}",
            self.verb, self.code, self.reason
        )
    }
}

impl std::error::Error for VerbReject {}

// ── whoami ────────────────────────────────────────────────────────────────────

/// Result of `ops::whoami`. Flat field-by-field so any dispatcher can format
/// it for its own output channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoamiResult {
    pub identity_id: IdentityXgid,
    pub display_name: String,
    pub home_node: NodeXgid,
    pub spaces_joined: usize,
}

/// Read the on-disk client state and project the whoami subset.
///
/// Pure data extraction — no println, no pipe writes. Replaces the data
/// half of the historical `cmd_whoami` (the println half lives in the
/// CLI shim in `app.rs`).
pub fn whoami(ctx: &mut OpContext<'_>) -> Result<WhoamiResult> {
    let state = crate::app::load_client_state(ctx.data_dir)?;
    Ok(WhoamiResult {
        identity_id: IdentityXgid::from_xgid(Xgid::new(state.identity_id)),
        display_name: state.display_name,
        home_node: NodeXgid::from_xgid(Xgid::new(state.home_node)),
        spaces_joined: state.spaces.len(),
    })
}

// ── status ────────────────────────────────────────────────────────────────────

/// Result of `ops::status`. Wider field set than `WhoamiResult` — includes
/// `version` and the age (in seconds) of the on-disk state file so the
/// CLI shim can format the staleness warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResult {
    pub identity_id: IdentityXgid,
    pub display_name: String,
    pub version: String,
    pub home_node: NodeXgid,
    pub spaces_joined: usize,
    pub state_file_age_seconds: i64,
}

/// Read the on-disk client state and compute the staleness age.
pub fn status(ctx: &mut OpContext<'_>) -> Result<StatusResult> {
    let state = crate::app::load_client_state(ctx.data_dir)?;
    let age = crate::app::age_seconds(&state.updated_at);
    Ok(StatusResult {
        identity_id: IdentityXgid::from_xgid(Xgid::new(state.identity_id)),
        display_name: state.display_name,
        version: state.version,
        home_node: NodeXgid::from_xgid(Xgid::new(state.home_node)),
        spaces_joined: state.spaces.len(),
        state_file_age_seconds: age,
    })
}

// ── spaces ────────────────────────────────────────────────────────────────────

/// Result of `ops::spaces`. Carries the known-Space list straight from
/// disk; the CLI shim formats the indented per-Space / per-Room printout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacesResult {
    pub spaces: Vec<xgen_common::state::KnownSpace>,
}

/// Read the on-disk client state and return the known-Space list verbatim.
pub fn spaces(ctx: &mut OpContext<'_>) -> Result<SpacesResult> {
    let state = crate::app::load_client_state(ctx.data_dir)?;
    Ok(SpacesResult {
        spaces: state.spaces,
    })
}

// ── rooms ───────────────────────────────────────────────────────────────────────

/// Result of `ops::rooms`. Carries the Room list for one Space straight from
/// disk, plus the Space's human-readable name for the CLI shim's heading. A
/// zero-network local read, same shape as `spaces` (M6 Phase 1, R1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomsResult {
    pub space_id: String,
    pub space_name: String,
    pub rooms: Vec<xgen_common::state::KnownRoom>,
}

/// Read the on-disk client state and return the Room list for the named Space.
/// Errors if no known Space matches `args.space`.
pub fn rooms(ctx: &mut OpContext<'_>, args: &crate::app::RoomsArgs) -> Result<RoomsResult> {
    let state = crate::app::load_client_state(ctx.data_dir)?;
    let space = state
        .spaces
        .into_iter()
        .find(|s| s.space_id == args.space)
        .ok_or_else(|| anyhow::anyhow!("no known Space with ID {}", args.space))?;
    Ok(RoomsResult {
        space_id: space.space_id,
        space_name: space.name,
        rooms: space.rooms,
    })
}

// ── register ──────────────────────────────────────────────────────────────────

/// Result of `ops::register`. Carries the printable fields plus the
/// Node-reported `registered_at` timestamp and an `is_ai` flag projecting
/// the M3 AI-Identity declaration that was sent on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResult {
    pub identity_id: IdentityXgid,
    pub display_name: String,
    pub home_node: NodeXgid,
    pub registered_at: String,
    /// True if the registration declared `is_ai = true` (M3, D-059).
    pub is_ai: bool,
}

/// Register the client's identity on the home Node.
///
/// **Precondition:** `ctx.session.identity` must be loaded by the
/// dispatcher (`SessionState::ensure_identity`) before this is called.
/// The home Node is resolved from `ctx.node_override`, falling back to
/// `ctx.session.home_node`.
///
/// On success the function:
/// - writes `xgen-client_state.json` with the new identity + empty Space list,
/// - sends a best-effort `goodbye` and lets the connection drop with the
///   session (M5 one-shot semantics).
pub async fn register(
    ctx: &mut OpContext<'_>,
    args: &crate::app::RegisterArgs,
    ai_section: Option<&crate::app::AiSection>,
) -> Result<RegisterResult> {
    use xgen_common::{build_info, state::ClientState};
    use xgen_core::{
        identity::registration::{
            build_register, build_register_with_ai, set_re_registration, sign_register,
        },
        transport::connection::Inbound,
        wire::types::IdentityMessage,
    };

    // Snapshot identity material before borrowing the connection mutably.
    let (signing_key, identity_id) = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        (id.signing_key.clone(), id.identity_id.as_str().to_string())
    };

    // Resolve home_node before borrowing session for the connection.
    let home_node = ctx
        .node_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| ctx.session.home_node.clone());

    // Build registration message — AI-aware per M3 (D-059, spec 3.6.10).
    let (reg_msg, is_ai) = match ai_section {
        Some(ai) if ai.is_ai => {
            let caps = xgen_common::wire::AiCapabilities {
                dm_initiate: ai.capabilities.get("dm_initiate").copied().unwrap_or(false),
                spontaneous_post: ai
                    .capabilities
                    .get("spontaneous_post")
                    .copied()
                    .unwrap_or(false),
                extra: Default::default(),
            };
            (
                build_register_with_ai(&signing_key, Some(args.name.clone()), true, Some(caps)),
                true,
            )
        }
        _ => (build_register(&signing_key, Some(args.name.clone())), false),
    };
    // S5-D1/D2 (C2) — orphan re-registration: stamp the wire flag before signing
    // so it is part of the canonical signed form (spec 3.13.8). The node Step-3
    // bypass + re-home land in C1 (handle_identity_msg). `home_changed` emit is
    // deferred (CP-5: the client holds no Node pubkey ids; new_home_node_id needs
    // a RegisterOk echo surface, a follow-on arc).
    let reg_msg = if args.re_registration {
        set_re_registration(reg_msg, true)
    } else {
        reg_msg
    };
    let reg = sign_register(reg_msg, &signing_key);

    // Scoped connection borrow so `ctx.session` is free for the state write.
    let registered_at = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        conn.send_identity(&reg)
            .await
            .context("failed to send registration")?;
        let ts = match conn.recv().await.context("no response from Node")? {
            Inbound::Identity(IdentityMessage::RegisterOk { registered_at, .. }) => registered_at,
            Inbound::Identity(IdentityMessage::RegisterFail {
                error_code,
                error_string,
                ..
            }) => {
                anyhow::bail!("registration rejected (code {}): {}", error_code, error_string);
            }
            other => anyhow::bail!("unexpected response from Node: {:?}", other),
        };
        // Courtesy goodbye — best-effort, errors swallowed (matches J-077 shape).
        let _ = conn.goodbye("client_disconnect").await;
        ts
    };

    let state = ClientState {
        identity_id: identity_id.clone(),
        display_name: args.name.clone(),
        version: build_info::VERSION.to_string(),
        build: build_info::GIT_HASH.to_string(),
        home_node: home_node.clone(),
        updated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        spaces: vec![],
    };
    crate::app::write_client_state(ctx.data_dir, &state)?;

    Ok(RegisterResult {
        identity_id: IdentityXgid::from_xgid(Xgid::new(identity_id)),
        display_name: args.name.clone(),
        home_node: NodeXgid::from_xgid(Xgid::new(home_node)),
        registered_at,
        is_ai,
    })
}

// ── create-space ──────────────────────────────────────────────────────────────

/// Result of `ops::create_space`. Carries the assigned `space_id`, the
/// originating `event_id`, and the new Space's owner identity for CLI
/// formatting. Result-struct shape matches the task file §3.1 example.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpaceResult {
    pub space_id: SpaceXgid,
    pub event_id: EventXgid,
    pub name: String,
    pub owner_identity_id: IdentityXgid,
}

/// Create a new Space owned by the calling Identity.
///
/// Precondition: `ctx.session.identity` loaded by the dispatcher.
/// Side effects: appends the new `KnownSpace` to `xgen-client_state.json`
/// (creating the state file from the keypair if absent), emits courtesy
/// `goodbye` on success.
pub async fn create_space(
    ctx: &mut OpContext<'_>,
    args: &crate::app::CreateSpaceArgs,
) -> Result<CreateSpaceResult> {
    use xgen_common::event_trace::{
        trace_event, EventDirection, SessionContext, SpaceRole,
    };
    use xgen_core::space::state::{build_space_create_event, sign_event};

    let (signing_key, identity_id) = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        (id.signing_key.clone(), id.identity_id.as_str().to_string())
    };

    let home_node = ctx
        .node_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| ctx.session.home_node.clone());

    // Build + sign the space_create event locally so the assigned IDs are
    // available before any network work.
    let space_ev = sign_event(
        build_space_create_event(&signing_key, &args.name, None, args.auth_tier, &home_node, None, false),
        &signing_key,
    );
    // Event.event_id is Option<EventXgid> (Pass 1-3). Project to String here so the
    // function body (SessionContext, KnownSpace, tracing) stays String-typed; the
    // Result construction re-wraps to the semantically-correct flavour (the
    // space_create event's id IS the space_id).
    let space_id = space_ev
        .event_id
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("signed space_create event missing event_id"))?
        .as_str()
        .to_string();
    let event_id = space_id.clone();

    let sync_timeout = sync_completion_timeout(ctx.data_dir);

    // Scoped connection borrow so ctx.session is free for the state write.
    {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;

        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(space_id.clone()),
        };
        trace_event(&space_ev, EventDirection::Out, &session_ctx);

        // MP-F1a (F1A-D1): submit-and-await — do not goodbye/return until the node
        // confirms (or the single-event timeout policy fires). On Rejected the `?`
        // propagates before the client-state write below, so a rejected create
        // writes no KnownSpace row.
        let outcome = conn.send_event_confirmed(&space_ev, sync_timeout).await;
        let _ = conn.goodbye("client_disconnect").await;
        apply_single_event_confirm(outcome, "create-space")?;
        tracing::info!(space_id = %space_id, name = %args.name, "Space created");
    }

    // Update client state with the new Space via the canonical
    // ops-private helper (the pre-M5 app::load_or_default_client_state
    // variant was deleted in commit 12 — every state-writing path now
    // uses load_or_default_state, which takes the already-cached
    // identity_id rather than re-loading the keypair).
    let mut state = load_or_default_state(ctx.data_dir, &identity_id, &home_node);
    state.spaces.push(xgen_common::state::KnownSpace {
        space_id: space_id.clone(),
        name: args.name.clone(),
        node_endpoint: home_node.clone(),
        role: "owner".to_string(),
        rooms: vec![],
    });
    state.updated_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    crate::app::write_client_state(ctx.data_dir, &state)?;

    Ok(CreateSpaceResult {
        space_id: SpaceXgid::from_xgid(Xgid::new(space_id)),
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        name: args.name.clone(),
        owner_identity_id: IdentityXgid::from_xgid(Xgid::new(identity_id)),
    })
}

// ── create-room ───────────────────────────────────────────────────────────────

/// Result of `ops::create_room`. For `state.room_create` the `event_id`
/// equals the `room_id`; both fields are exposed for caller clarity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoomResult {
    pub room_id: RoomXgid,
    pub event_id: EventXgid,
    pub space_id: SpaceXgid,
    pub name: String,
}

pub async fn create_room(
    ctx: &mut OpContext<'_>,
    args: &crate::app::CreateRoomArgs,
) -> Result<CreateRoomResult> {
    use xgen_common::event_trace::{
        trace_event, EventDirection, SessionContext, SpaceRole,
    };
    use xgen_core::space::state::{build_room_create_event, sign_event};

    let (signing_key, identity_id) = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        (id.signing_key.clone(), id.identity_id.as_str().to_string())
    };

    let home_node = ctx
        .node_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| ctx.session.home_node.clone());

    let room_ev = sign_event(
        build_room_create_event(&signing_key, &args.space, &args.name, None),
        &signing_key,
    );
    // Project EventXgid → String here (see create_space); Result re-wraps the
    // room_create event's id as the room_id.
    let room_id = room_ev
        .event_id
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("signed room_create event missing event_id"))?
        .as_str()
        .to_string();
    let event_id = room_id.clone();

    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(args.space.clone()),
        };
        trace_event(&room_ev, EventDirection::Out, &session_ctx);
        // MP-F1a (F1A-D1): confirm before goodbye (single-event policy).
        let outcome = conn.send_event_confirmed(&room_ev, sync_timeout).await;
        let _ = conn.goodbye("client_disconnect").await;
        apply_single_event_confirm(outcome, "create-room")?;
    }

    // Update state — find the parent Space, append the Room.
    let mut state = load_or_default_state(ctx.data_dir, &identity_id, &home_node);
    if let Some(space) = state.spaces.iter_mut().find(|s| s.space_id == args.space) {
        space.rooms.push(xgen_common::state::KnownRoom {
            room_id: room_id.clone(),
            name: args.name.clone(),
            joined: true,
        });
    }
    state.updated_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    crate::app::write_client_state(ctx.data_dir, &state)?;

    Ok(CreateRoomResult {
        room_id: RoomXgid::from_xgid(Xgid::new(room_id)),
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        name: args.name.clone(),
    })
}

// ── create-dm-space ─────────────────────────────────────────────────────────

/// Result of `ops::create_dm_space` (M7C-D4, A3). Carries the new DM Space's
/// id, the auto-created DM Room, the `dm_space_create` event id, and the invitee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDmSpaceResult {
    pub space_id: SpaceXgid,
    pub room_id: RoomXgid,
    pub event_id: EventXgid,
    pub invitee: IdentityXgid,
    pub owner_identity_id: IdentityXgid,
}

/// Create a DM Space with a single invitee (M7C-D4, A3) — the one Block-A verb
/// that exceeds pure adapter. The creator builds and signs the three events of
/// the DM's initial causal chain and sends them over ONE connection, in order:
///
/// ```text
///   dm_space_create (root)  ←  state.room_create (auto-room)  ←  membership.invite
/// ```
///
/// **Ordering is the correctness contract** (the A3 invariant): the node rejects
/// a non-create event targeting an unbuilt Space (`runtime.rs` step 1) —
/// room/invite are NOT pending-buffered — so the root MUST arrive first.
/// `process_inbound` is sequential per event, so an in-order single-connection
/// send is sufficient (do NOT parallelize or reorder the sends).
///
/// Membership rides the ROOT: the node's key-less `from_dm_space_create_node`
/// ingest arm seeds `members={creator}` + `pending_invites={invitee}` from the
/// root content. DMs are single-homed (federation disabled), so there is no
/// federation push — the invitee participates by connecting to this home Node.
///
/// Latent constructor issue (D-065): `SpaceState::from_dm_space_create` produces
/// the auto-invite with EMPTY prev_events (root-shaped), which a node would
/// gate-reject (non-root needs a predecessor). A3 takes the constructor's
/// auto-room as-is and **rebuilds the invite tip-chained to the room** at this
/// call site (so it is a well-formed, accepted, causally-chained DAG record,
/// a state no-op via apply_invite's DM-constraint reject). The constructor's
/// empty-prev_events invite is flagged for its own touch — NOT fixed inside A3.
pub async fn create_dm_space(
    ctx: &mut OpContext<'_>,
    args: &crate::app::CreateDmSpaceArgs,
) -> Result<CreateDmSpaceResult> {
    use serde_json::json;
    use xgen_common::event_trace::{trace_event, EventDirection, SessionContext, SpaceRole};
    use xgen_core::{
        space::state::{
            build_dm_space_create_event, build_membership_event, sign_event, SpaceState,
        },
        transport::connection::EventConfirm,
        wire::types::EventType,
    };

    let (signing_key, identity_id) = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        (id.signing_key.clone(), id.identity_id.as_str().to_string())
    };

    let home_node = ctx
        .node_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| ctx.session.home_node.clone());

    // 1) Root: state.dm_space_create — its event_id IS the space_id.
    let dm_ev = sign_event(
        build_dm_space_create_event(&signing_key, &args.invitee, &home_node),
        &signing_key,
    );
    let space_id = dm_ev
        .event_id
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("signed dm_space_create event missing event_id"))?
        .as_str()
        .to_string();

    // 2) Auto-room from the constructor (correctly chained to the space). The
    //    constructor's bundled auto-invite (empty prev_events — latent bug,
    //    D-065) is discarded; the invite is rebuilt tip-chained below.
    let (_state, room_ev, _constructor_invite) =
        SpaceState::from_dm_space_create(&dm_ev, &signing_key)
            .context("failed to derive DM auto-room from dm_space_create")?;
    let room_id = room_ev
        .event_id
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("auto-room event missing event_id"))?
        .as_str()
        .to_string();

    // 3) Invite, tip-chained to the genuine auto-room (dm_space_create ← room ←
    //    invite). prev_events reads the real room event_id (not a literal),
    //    matching how `create_room` chains to its parent Space at construction.
    let mut invite_unsigned = build_membership_event(
        &signing_key,
        &space_id,
        &room_id,
        EventType::MembershipInvite,
        json!({ "target_identity": args.invitee, "role": "member" }),
    );
    invite_unsigned.prev_events = vec![EventXgid::from_xgid(Xgid::new(room_id.clone()))];
    let invite_ev = sign_event(invite_unsigned, &signing_key);

    let sync_timeout = sync_completion_timeout(ctx.data_dir);

    // Send the three events over ONE connection, in order, root-first (A3
    // invariant) — and CONFIRM each before the next (MP-F1a F1A-D1/D4, the §3
    // "chain" class). A chain timeout / transport failure is genuinely-lost (the
    // predecessor was acked-present) → abort + Err (F-5). A reject of the root or
    // room is a real failure → Err. A reject of the auto-invite is by-design-OK:
    // its DM-constraint apply is an internal state no-op the node swallows
    // (empirically it Accepts) → accept-either per F1A-D3. The client-state record
    // is written only AFTER this block, so a failed create writes no success row.
    {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(space_id.clone()),
        };

        // Root + auto-room: each must be node-accepted before the next is sent.
        for (ev, label) in [(&dm_ev, "dm_space_create"), (&room_ev, "auto-room")] {
            trace_event(ev, EventDirection::Out, &session_ctx);
            match conn.send_event_confirmed(ev, sync_timeout).await {
                Ok(EventConfirm::Accepted) => {}
                Ok(EventConfirm::Rejected { code, reason, .. }) => {
                    // MP-F5: the multi-event chain stays text-bailing (out of the
                    // single-event reject-surfacing scope; F1 does not touch it).
                    let _ = conn.goodbye("client_disconnect").await;
                    anyhow::bail!(
                        "create-dm-space: {label} rejected by node (code {code}): {reason}"
                    );
                }
                Ok(EventConfirm::TimedOut) => {
                    let _ = conn.goodbye("client_disconnect").await;
                    anyhow::bail!(
                        "create-dm-space: {label} not confirmed within {} ms — aborting chain \
                         (predecessor was acked-present ⇒ genuinely lost, F-5)",
                        sync_timeout.as_millis()
                    );
                }
                Err(e) => {
                    let _ = conn.goodbye("client_disconnect").await;
                    return Err(anyhow::Error::new(e).context(format!(
                        "create-dm-space: {label} send-confirm transport error"
                    )));
                }
            }
        }

        // Auto-invite: accept-either (Accepted OR Rejected both mean "the node
        // took it", F1A-D3); only an unconfirmed (TimedOut) / transport failure
        // aborts the chain.
        trace_event(&invite_ev, EventDirection::Out, &session_ctx);
        match conn.send_event_confirmed(&invite_ev, sync_timeout).await {
            Ok(EventConfirm::Accepted) | Ok(EventConfirm::Rejected { .. }) => {}
            Ok(EventConfirm::TimedOut) => {
                let _ = conn.goodbye("client_disconnect").await;
                anyhow::bail!(
                    "create-dm-space: auto-invite not confirmed within {} ms — aborting chain",
                    sync_timeout.as_millis()
                );
            }
            Err(e) => {
                let _ = conn.goodbye("client_disconnect").await;
                return Err(anyhow::Error::new(e)
                    .context("create-dm-space: auto-invite send-confirm transport error"));
            }
        }

        tracing::info!(space_id = %space_id, invitee = %args.invitee, "DM Space created");
        let _ = conn.goodbye("client_disconnect").await;
    }

    // Record the DM Space in client state (creator is owner; the DM Room is known).
    let mut state = load_or_default_state(ctx.data_dir, &identity_id, &home_node);
    state.spaces.push(xgen_common::state::KnownSpace {
        space_id: space_id.clone(),
        name: format!("DM with {}", args.invitee),
        node_endpoint: home_node.clone(),
        role: "owner".to_string(),
        rooms: vec![xgen_common::state::KnownRoom {
            room_id: room_id.clone(),
            name: "dm".to_string(),
            joined: true,
        }],
    });
    state.updated_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    crate::app::write_client_state(ctx.data_dir, &state)?;

    Ok(CreateDmSpaceResult {
        space_id: SpaceXgid::from_xgid(Xgid::new(space_id.clone())),
        room_id: RoomXgid::from_xgid(Xgid::new(room_id)),
        // The dm_space_create event's id IS the space_id.
        event_id: EventXgid::from_xgid(Xgid::new(space_id)),
        invitee: IdentityXgid::from_xgid(Xgid::new(args.invitee.clone())),
        owner_identity_id: IdentityXgid::from_xgid(Xgid::new(identity_id)),
    })
}

// ── invite ────────────────────────────────────────────────────────────────────

/// Result of `ops::invite`. Carries the assigned `event_id` plus the
/// target Identity, Space, and role so the CLI shim and any future
/// `--aicontrol` JSONL serialiser have the full picture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteResult {
    pub event_id: EventXgid,
    pub target_identity: IdentityXgid,
    pub space_id: SpaceXgid,
    pub role: String,
}

pub async fn invite(
    ctx: &mut OpContext<'_>,
    args: &crate::app::InviteArgs,
) -> Result<InviteResult> {
    use chrono::{SecondsFormat, Utc};
    use serde_json::json;
    use xgen_common::event_trace::{
        trace_event, EventDirection, SessionContext, SpaceRole,
    };
    use xgen_core::{
        space::state::sign_event,
        wire::types::{Event, EventType},
    };

    let (signing_key, identity_id) = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        (id.signing_key.clone(), id.identity_id.as_str().to_string())
    };

    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    let event_id = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        // Anchor the invite in the current DAG tip so a subsequent join
        // chains correctly (M3: SpaceMember.invited_by flows through the
        // topological replay that ai_status performs). Fall back to the
        // space_id anchor when tip discovery fails.
        let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
            .await
            .unwrap_or_else(|_| vec![args.space.clone()]);
        // M8.5-B (INV-D6) — stamp `valid_until` via the cascade at sign time
        // (the invite is signed here, so the deadline must be in the content
        // before signing — the Node cannot fill it post-hoc). Cascade for C2:
        // individual `--valid-for-days` → protocol default (14d). The
        // node-default tier is deferred (the client has no source for the
        // inviter-node's default); the Node enforces the per-tier ceiling at
        // ingest (wire 3045) as the backstop. Default-stamp-14d (Joe-lock):
        // an invite with no expiry is the unbounded capability INV-D6 prevents,
        // so 14d is the secure default, not merely "what the design says".
        const PROTOCOL_DEFAULT_VALIDITY_DAYS: i64 = 14;
        let validity_days = args
            .valid_for_days
            .map(i64::from)
            .unwrap_or(PROTOCOL_DEFAULT_VALIDITY_DAYS);
        let valid_until = (Utc::now() + chrono::Duration::days(validity_days))
            .to_rfc3339_opts(SecondsFormat::Millis, true);
        let mut content = json!({
            "target_identity": args.identity,
            "role": args.role,
            "valid_until": valid_until,
        });
        // M8.5-B (INV-D5) — optional opaque `note` (message.rich-format body).
        if let Some(note) = &args.note {
            content["note"] = json!(note);
        }
        let invite_ev = sign_event(
            Event::new(
                EventType::MembershipInvite,
                IdentityXgid::from_xgid(Xgid::new(identity_id.clone())),
                RoomXgid::default(),
                SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
                prev_events
                    .into_iter()
                    .map(|e| EventXgid::from_xgid(Xgid::new(e)))
                    .collect(),
                Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                content,
            ),
            &signing_key,
        );
        let session_ctx = SessionContext {
            identity_id: invite_ev
                .event_id
                .as_ref()
                .map(|_| invite_ev.sender.as_str().to_string()),
            role: Some(SpaceRole::Owner),
            space_id: Some(args.space.clone()),
        };
        trace_event(&invite_ev, EventDirection::Out, &session_ctx);
        let id_for_result = invite_ev
            .event_id
            .as_ref()
            .map(|e| e.as_str().to_string())
            .unwrap_or_default();
        // MP-F1a (F1A-D1): confirm before goodbye (single-event policy).
        let outcome = conn.send_event_confirmed(&invite_ev, sync_timeout).await;
        let _ = conn.goodbye("client_disconnect").await;
        apply_single_event_confirm(outcome, "invite")?;
        id_for_result
    };

    Ok(InviteResult {
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        target_identity: IdentityXgid::from_xgid(Xgid::new(args.identity.clone())),
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        role: args.role.clone(),
    })
}

// ── ban ─────────────────────────────────────────────────────────────────────

/// Result of `ops::ban`. Carries the `event_id` plus the banned target and
/// Space, for CLI / `--aicontrol` formatting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanResult {
    pub event_id: EventXgid,
    pub target_identity: IdentityXgid,
    pub space_id: SpaceXgid,
}

/// Ban a member from a Space — Admin+ `membership.ban` (thin-verb arc 2,
/// MP-C-09/MP-A-14). Mirrors [`invite`] end-to-end (build → sign → send-confirm);
/// the only divergences are the event type and a `{target_identity}` content.
/// The `can_ban` gate (Admin+) is enforced node-side at `validate_event` +
/// `apply_ban`; an unauthorised ban is refused (surfaced via the MP-F5 reject
/// path, like any single-event reject). `apply_ban` cascades the removal across
/// every Room (space-level), so this verb takes no `--room`.
pub async fn ban(ctx: &mut OpContext<'_>, args: &crate::app::BanArgs) -> Result<BanResult> {
    use chrono::{SecondsFormat, Utc};
    use serde_json::json;
    use xgen_common::event_trace::{trace_event, EventDirection, SessionContext, SpaceRole};
    use xgen_core::{
        space::state::sign_event,
        wire::types::{Event, EventType},
    };

    let (signing_key, identity_id) = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        (id.signing_key.clone(), id.identity_id.as_str().to_string())
    };

    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    let event_id = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        // Anchor in the current DAG tip (fall back to the space_id anchor), like
        // invite/join — the ban must chain causally so resolution sees it.
        let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
            .await
            .unwrap_or_else(|_| vec![args.space.clone()]);
        let ban_ev = sign_event(
            Event::new(
                EventType::MembershipBan,
                IdentityXgid::from_xgid(Xgid::new(identity_id.clone())),
                RoomXgid::default(),
                SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
                prev_events
                    .into_iter()
                    .map(|e| EventXgid::from_xgid(Xgid::new(e)))
                    .collect(),
                Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                json!({ "target_identity": args.identity }),
            ),
            &signing_key,
        );
        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(args.space.clone()),
        };
        trace_event(&ban_ev, EventDirection::Out, &session_ctx);
        let id_for_result = ban_ev
            .event_id
            .as_ref()
            .map(|e| e.as_str().to_string())
            .unwrap_or_default();
        // MP-F1a (F1A-D1): confirm before goodbye (single-event policy). A node
        // refusal (e.g. non-admin banner) surfaces structurally per MP-F5.
        let outcome = conn.send_event_confirmed(&ban_ev, sync_timeout).await;
        let _ = conn.goodbye("client_disconnect").await;
        apply_single_event_confirm(outcome, "ban")?;
        tracing::info!(space_id = %args.space, target = %args.identity, "Member banned");
        id_for_result
    };

    Ok(BanResult {
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        target_identity: IdentityXgid::from_xgid(Xgid::new(args.identity.clone())),
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
    })
}

// ── room-update ───────────────────────────────────────────────────────────────

/// Result of `ops::room_update`. `override_count` is how many overrides the verb
/// set (the room's COMPLETE set after this update — wholesale-replace, RU-D1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomUpdateResult {
    pub event_id: EventXgid,
    pub space_id: SpaceXgid,
    pub room_id: RoomXgid,
    pub override_count: usize,
}

/// Set a Room's per-Role permission overrides — Admin+ `state.room_update`
/// (thin-verb arc 3, MP-C-08 / PG-12). **Wholesale-replace:** the `--deny` /
/// `--allow` specs become the Room's COMPLETE override set; any override not
/// listed is cleared (`apply_room_update`, Arc D CP-3). Authority is Admin+
/// (`check_permission` `StateRoomUpdate` → `ChangeInfo`); a non-admin update is
/// refused at validation (surfaced via the MP-F5 reject path). Mirrors
/// [`create_room`] for dispatch + ban/invite for send-confirm.
pub async fn room_update(
    ctx: &mut OpContext<'_>,
    args: &crate::app::RoomUpdateArgs,
) -> Result<RoomUpdateResult> {
    use xgen_common::event_trace::{trace_event, EventDirection, SessionContext, SpaceRole};
    use xgen_core::space::{
        membership::{Effect, Role, RoomPermission},
        state::{build_room_update_event, sign_event},
    };

    // Parse `<role>:<permission>` specs into typed overrides. An unparseable spec
    // is a BAD_ARGUMENT-class error (anyhow surfaces it).
    fn parse_spec(spec: &str, effect: Effect) -> Result<(Role, RoomPermission, Effect)> {
        let (role_s, perm_s) = spec.split_once(':').ok_or_else(|| {
            anyhow::anyhow!("override must be <role>:<permission>, got {spec:?}")
        })?;
        let role = Role::from_str(role_s)
            .ok_or_else(|| anyhow::anyhow!("unknown role {role_s:?} in override {spec:?}"))?;
        let perm = RoomPermission::from_str(perm_s).ok_or_else(|| {
            anyhow::anyhow!("unknown permission {perm_s:?} in override {spec:?}")
        })?;
        Ok((role, perm, effect))
    }
    let mut overrides: Vec<(Role, RoomPermission, Effect)> = Vec::new();
    for s in &args.deny {
        overrides.push(parse_spec(s, Effect::Deny)?);
    }
    for s in &args.allow {
        overrides.push(parse_spec(s, Effect::Allow)?);
    }

    let (signing_key, identity_id) = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        (id.signing_key.clone(), id.identity_id.as_str().to_string())
    };

    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    let event_id = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
            .await
            .unwrap_or_else(|_| vec![args.space.clone()]);
        let ev = sign_event(
            build_room_update_event(&signing_key, &args.space, &args.room, prev_events, &overrides),
            &signing_key,
        );
        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(args.space.clone()),
        };
        trace_event(&ev, EventDirection::Out, &session_ctx);
        let id_for_result = ev
            .event_id
            .as_ref()
            .map(|e| e.as_str().to_string())
            .unwrap_or_default();
        let outcome = conn.send_event_confirmed(&ev, sync_timeout).await;
        let _ = conn.goodbye("client_disconnect").await;
        apply_single_event_confirm(outcome, "room-update")?;
        tracing::info!(space_id = %args.space, room_id = %args.room, overrides = overrides.len(), "Room overrides updated");
        id_for_result
    };

    Ok(RoomUpdateResult {
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        room_id: RoomXgid::from_xgid(Xgid::new(args.room.clone())),
        override_count: overrides.len(),
    })
}

// ── join ──────────────────────────────────────────────────────────────────────

/// Result of `ops::join`. `room_id` is `None` when joining the Space
/// itself (the historical `cmd_join` behaviour when `--room` was absent).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinResult {
    pub event_id: EventXgid,
    pub space_id: SpaceXgid,
    pub room_id: Option<RoomXgid>,
}

pub async fn join(
    ctx: &mut OpContext<'_>,
    args: &crate::app::JoinArgs,
) -> Result<JoinResult> {
    use chrono::{SecondsFormat, Utc};
    use serde_json::json;
    use xgen_common::event_trace::{
        trace_event, EventDirection, SessionContext, SpaceRole,
    };
    use xgen_core::{
        space::state::sign_event,
        wire::types::{Event, EventType},
    };

    let (signing_key, identity_id) = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        (id.signing_key.clone(), id.identity_id.as_str().to_string())
    };

    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    let event_id = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        // M8.5-B (INV-D3) — the bootstrap. A pending invitee sources the invite
        // naming it via the scoped structural fetch and chains its join
        // `prev_events=[invite_id]` — causally *after* the invite, so the two
        // are not concurrent on the `membership:{space}:{invitee}` key (this is
        // what dissolves M85-A3; the join is no longer dropped by derive_resolved
        // Layer 4). When the fetch yields no invite (already a member, a Room
        // join, or the Node refuses), fall back to the DAG tip.
        let prev_events = match crate::batch::get_invite_bootstrap(
            conn,
            &args.space,
            &identity_id,
            sync_timeout,
        )
        .await
        {
            Ok(Some(invite_id)) => vec![invite_id],
            _ => {
                // INV-D4 — `get_dag_tips` fallback now treats `Ok(empty)` like
                // `Err`: an empty tip set (e.g. the invitee saw no member-visible
                // events) would otherwise yield empty `prev_events` — a
                // root-shaped non-root event the Node gate-rejects. Anchor to the
                // Space create instead. (The invite-chain above is the primary
                // path; this is the defensive fallback.)
                match crate::batch::get_dag_tips(conn, &args.space, sync_timeout).await {
                    Ok(tips) if !tips.is_empty() => tips,
                    _ => vec![args.space.clone()],
                }
            }
        };
        let join_ev = sign_event(
            Event::new(
                EventType::MembershipJoin,
                IdentityXgid::from_xgid(Xgid::new(identity_id.clone())),
                RoomXgid::from_xgid(Xgid::new(args.room.clone().unwrap_or_default())),
                SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
                prev_events
                    .into_iter()
                    .map(|e| EventXgid::from_xgid(Xgid::new(e)))
                    .collect(),
                Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                json!({}),
            ),
            &signing_key,
        );
        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(args.space.clone()),
        };
        trace_event(&join_ev, EventDirection::Out, &session_ctx);
        let id_for_result = join_ev
            .event_id
            .as_ref()
            .map(|e| e.as_str().to_string())
            .unwrap_or_default();
        // MP-F1a (F1A-D1): confirm before goodbye (single-event policy).
        let outcome = conn.send_event_confirmed(&join_ev, sync_timeout).await;
        let _ = conn.goodbye("client_disconnect").await;
        apply_single_event_confirm(outcome, "join")?;
        tracing::info!(space_id = %args.space, "Joined Space");
        id_for_result
    };

    Ok(JoinResult {
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        room_id: args
            .room
            .clone()
            .map(|r| RoomXgid::from_xgid(Xgid::new(r))),
    })
}

// ── leave ──────────────────────────────────────────────────────────────────────

/// Result of `ops::leave`. Mirrors [`JoinResult`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveResult {
    pub event_id: EventXgid,
    pub space_id: SpaceXgid,
    pub room_id: Option<RoomXgid>,
}

/// Leave a Space (or a single Room within it) — member-initiated
/// `membership.leave` (M7C-D3, A2). Pure adapter: mirrors [`join`] end-to-end
/// (build → sign → send). The node accepts it on signature + step-11
/// sender-membership with no special role — `validate_steps_8_13` special-cases
/// only invite/kick/ban, so leave falls to the default member-event path.
/// `membership.leave` is a non-root event, so it tip-chains exactly like `join`
/// (the empty-`prev_events` `build_membership_event` helper is for root-adjacent
/// callers, not this one). Space-level when `--room` is omitted: `apply_leave`
/// removes the member from the Space and every Room. Like `join`, this now
/// awaits the node's per-event confirm via `send_event_confirmed` (MP-F1a,
/// F1A-D1): the single-event policy warns + proceeds on a confirm-timeout and
/// returns `Err` on a node reject (the D-070 `EventAccepted` / `Error` ack is
/// the positive accept signal the J-080 note anticipated).
pub async fn leave(
    ctx: &mut OpContext<'_>,
    args: &crate::app::LeaveArgs,
) -> Result<LeaveResult> {
    use chrono::{SecondsFormat, Utc};
    use serde_json::json;
    use xgen_common::event_trace::{trace_event, EventDirection, SessionContext, SpaceRole};
    use xgen_core::{
        space::state::sign_event,
        wire::types::{Event, EventType},
    };

    let (signing_key, identity_id) = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        (id.signing_key.clone(), id.identity_id.as_str().to_string())
    };

    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    let event_id = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        // Tip-chain the leave so it lands after the member's prior events
        // (a non-root event with empty prev_events would fail DAG validation).
        let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
            .await
            .unwrap_or_else(|_| vec![args.space.clone()]);
        let leave_ev = sign_event(
            Event::new(
                EventType::MembershipLeave,
                IdentityXgid::from_xgid(Xgid::new(identity_id.clone())),
                RoomXgid::from_xgid(Xgid::new(args.room.clone().unwrap_or_default())),
                SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
                prev_events
                    .into_iter()
                    .map(|e| EventXgid::from_xgid(Xgid::new(e)))
                    .collect(),
                Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                json!({}),
            ),
            &signing_key,
        );
        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(args.space.clone()),
        };
        trace_event(&leave_ev, EventDirection::Out, &session_ctx);
        let id_for_result = leave_ev
            .event_id
            .as_ref()
            .map(|e| e.as_str().to_string())
            .unwrap_or_default();
        // MP-F1a (F1A-D1): confirm before goodbye (single-event policy).
        let outcome = conn.send_event_confirmed(&leave_ev, sync_timeout).await;
        let _ = conn.goodbye("client_disconnect").await;
        apply_single_event_confirm(outcome, "leave")?;
        tracing::info!(space_id = %args.space, "Left Space");
        id_for_result
    };

    Ok(LeaveResult {
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        room_id: args
            .room
            .clone()
            .map(|r| RoomXgid::from_xgid(Xgid::new(r))),
    })
}

// ── send ──────────────────────────────────────────────────────────────────────

/// Result of `ops::send`. MP-F1a (F1A-D1) made `send` await the node's
/// per-event confirm: "Message sent." now prints only after an `EventAccepted`
/// (or after a warn-and-proceed on a confirm-timeout); a node reject surfaces as
/// `Err` (CP-1). The structured ack path the pre-F1a comment deferred to "M7"
/// is this `send_event_confirmed` consume of the D-070 ack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub event_id: EventXgid,
    pub space_id: SpaceXgid,
    pub room_id: RoomXgid,
}

/// Send a text message to a Room.
///
/// The headline M5 migration: `send` is the verb whose duplicated
/// `get_dag_tips` produced F-003/F-004 in J-067. J-068's wider dedup
/// already collapsed the two copies into the single canonical
/// `crate::batch::get_dag_tips` — M5 ships the structural lock: once
/// `ops::send` is the only call site, there is nowhere a second copy
/// could be reintroduced without being noticed.
pub async fn send(
    ctx: &mut OpContext<'_>,
    args: &crate::app::SendArgs,
) -> Result<SendResult> {
    use xgen_common::event_trace::{
        trace_event, EventDirection, SessionContext, SpaceRole,
    };
    use xgen_core::{
        message::exchange::build_message_text_event,
        space::state::sign_event,
    };

    let (signing_key, identity_id) = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        (id.signing_key.clone(), id.identity_id.as_str().to_string())
    };

    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    let event_id = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        // Tip discovery via the canonical (single-source) implementation
        // closes the F-003/F-004 class architecturally: there is nowhere
        // else this can be re-implemented now.
        let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
            .await
            .unwrap_or_else(|_| vec![args.space.clone()]);
        let msg_ev = sign_event(
            build_message_text_event(
                &signing_key,
                &args.space,
                &args.room,
                prev_events,
                &args.text,
            ),
            &signing_key,
        );
        let id_for_result = msg_ev
            .event_id
            .as_ref()
            .map(|e| e.as_str().to_string())
            .unwrap_or_default();
        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(args.space.clone()),
        };
        trace_event(&msg_ev, EventDirection::Out, &session_ctx);
        // MP-F1a (F1A-D1): confirm before goodbye (single-event policy).
        let outcome = conn.send_event_confirmed(&msg_ev, sync_timeout).await;
        let _ = conn.goodbye("client_disconnect").await;
        apply_single_event_confirm(outcome, "send")?;
        tracing::info!(room = %args.room, "Message sent");
        id_for_result
    };

    Ok(SendResult {
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        room_id: RoomXgid::from_xgid(Xgid::new(args.room.clone())),
    })
}

// ── history ───────────────────────────────────────────────────────────────────

/// One message in `HistoryResult.messages`. `sender` is the full
/// `identity_id` (CLI shim truncates with `short_id` for display).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub sender: IdentityXgid,
    pub timestamp: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResult {
    pub space_id: SpaceXgid,
    pub room_id: RoomXgid,
    pub messages: Vec<HistoryMessage>,
}

/// Pull the message history for a Room via `transport.sync_request` and
/// project text messages into `HistoryResult`. Up to `args.limit`
/// messages are returned. F-6 / F-7 migration: termination is the explicit
/// `SyncComplete` signal (with optional pagination via `continue_from`);
/// the 5-second hardcoded deadline is replaced by `[sync].completion_timeout_seconds`
/// as a safety net (default 5s).
pub async fn history(
    ctx: &mut OpContext<'_>,
    args: &crate::app::HistoryArgs,
) -> Result<HistoryResult> {
    use xgen_common::event_trace::{
        trace_event, EventDirection, SessionContext, SpaceRole,
    };
    use xgen_core::{
        transport::connection::Inbound,
        wire::types::{EventType, TransportMessage},
    };

    let identity_id = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        id.identity_id.as_str().to_string()
    };

    let sync_timeout = sync_completion_timeout(ctx.data_dir);

    let mut messages: Vec<HistoryMessage> = Vec::new();
    {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(args.space.clone()),
        };

        let mut since = String::new();
        let deadline = tokio::time::Instant::now() + sync_timeout;
        let mut limit_reached = false;
        'pages: loop {
            let sync_req = TransportMessage::SyncRequest {
                protocol_version: "0.1".to_string(),
                since: since.clone(),
                limit: None,
            };
            conn.send_transport(&sync_req)
                .await
                .context("failed to send sync_request")?;

            // Drain one page until SyncComplete.
            let continue_from = loop {
                match tokio::time::timeout_at(deadline, conn.recv()).await {
                    Ok(Ok(Inbound::Event(ev))) => {
                        trace_event(&ev, EventDirection::In, &session_ctx);
                        if ev.space_id.as_str() == args.space.as_str()
                            && ev.room_id.as_str() == args.room.as_str()
                            && matches!(ev.event_type, EventType::MessageText)
                        {
                            let text = ev.content["text"].as_str().unwrap_or("").to_string();
                            messages.push(HistoryMessage {
                                sender: ev.sender.clone(),
                                timestamp: ev.timestamp.clone(),
                                text,
                            });
                            if messages.len() >= args.limit {
                                limit_reached = true;
                            }
                        }
                    }
                    Ok(Ok(Inbound::Transport(TransportMessage::SyncComplete {
                        continue_from,
                        ..
                    }))) => break continue_from,
                    Ok(Ok(Inbound::Transport(TransportMessage::Goodbye { .. })))
                    | Ok(Ok(Inbound::Closed)) => break 'pages,
                    Ok(Ok(_)) => {}
                    Ok(Err(_)) => break 'pages,
                    Err(_) => {
                        // F-6b safety-net.
                        anyhow::bail!(
                            "sync_request safety-net timeout — peer never sent sync_complete \
                             within {} ms",
                            sync_timeout.as_millis()
                        );
                    }
                }
            };
            if limit_reached {
                break 'pages;
            }
            match continue_from {
                Some(cursor) => since = cursor,
                None => break 'pages,
            }
        }
        let _ = conn.goodbye("client_disconnect").await;
    }

    Ok(HistoryResult {
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        room_id: RoomXgid::from_xgid(Xgid::new(args.room.clone())),
        messages,
    })
}

// ── ai delegate / revoke / status (M3, spec 3.6.10.6) ─────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDelegateResult {
    pub event_id: EventXgid,
    pub space_id: SpaceXgid,
    pub ai_identity_id: IdentityXgid,
    pub new_operator: IdentityXgid,
}

pub async fn ai_delegate(
    ctx: &mut OpContext<'_>,
    args: &crate::app::AiDelegateArgs,
) -> Result<AiDelegateResult> {
    use xgen_common::event_trace::{
        trace_event, EventDirection, SessionContext, SpaceRole,
    };
    use xgen_core::space::state::sign_event;

    let (signing_key, identity_id) = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        (id.signing_key.clone(), id.identity_id.as_str().to_string())
    };

    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    let event_id = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
            .await
            .unwrap_or_else(|_| vec![args.space.clone()]);
        let ev = sign_event(
            xgen_core::space::state::build_state_ai_operator_delegate_event(
                &signing_key,
                &args.space,
                prev_events,
                &args.ai,
                &args.to,
            ),
            &signing_key,
        );
        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(args.space.clone()),
        };
        trace_event(&ev, EventDirection::Out, &session_ctx);
        let id_for_result = ev
            .event_id
            .as_ref()
            .map(|e| e.as_str().to_string())
            .unwrap_or_default();
        // MP-F1a (F1A-D1): confirm before goodbye (single-event policy).
        let outcome = conn.send_event_confirmed(&ev, sync_timeout).await;
        let _ = conn.goodbye("client_disconnect").await;
        apply_single_event_confirm(outcome, "ai-delegate")?;
        id_for_result
    };

    Ok(AiDelegateResult {
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        ai_identity_id: IdentityXgid::from_xgid(Xgid::new(args.ai.clone())),
        new_operator: IdentityXgid::from_xgid(Xgid::new(args.to.clone())),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRevokeResult {
    pub event_id: EventXgid,
    pub space_id: SpaceXgid,
    pub ai_identity_id: IdentityXgid,
}

pub async fn ai_revoke(
    ctx: &mut OpContext<'_>,
    args: &crate::app::AiRevokeArgs,
) -> Result<AiRevokeResult> {
    use xgen_common::event_trace::{
        trace_event, EventDirection, SessionContext, SpaceRole,
    };
    use xgen_core::space::state::sign_event;

    let (signing_key, identity_id) = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        (id.signing_key.clone(), id.identity_id.as_str().to_string())
    };

    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    let event_id = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
            .await
            .unwrap_or_else(|_| vec![args.space.clone()]);
        let ev = sign_event(
            xgen_core::space::state::build_state_ai_operator_revoke_event(
                &signing_key,
                &args.space,
                prev_events,
                &args.ai,
            ),
            &signing_key,
        );
        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(args.space.clone()),
        };
        trace_event(&ev, EventDirection::Out, &session_ctx);
        let id_for_result = ev
            .event_id
            .as_ref()
            .map(|e| e.as_str().to_string())
            .unwrap_or_default();
        // MP-F1a (F1A-D1): confirm before goodbye (single-event policy).
        let outcome = conn.send_event_confirmed(&ev, sync_timeout).await;
        let _ = conn.goodbye("client_disconnect").await;
        apply_single_event_confirm(outcome, "ai-revoke")?;
        id_for_result
    };

    Ok(AiRevokeResult {
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        ai_identity_id: IdentityXgid::from_xgid(Xgid::new(args.ai.clone())),
    })
}

/// Result of `ops::ai_status`. Carries the resolved operator + the
/// classification label the CLI prints ("stored delegation" / "inviter
/// fallback" / "owner fallback" / "resolved"), plus diagnostic fields
/// the pre-M5 implementation emitted at TRACE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiStatusResult {
    pub space_id: SpaceXgid,
    pub ai_identity_id: IdentityXgid,
    pub node: NodeXgid,
    pub operator: Option<IdentityXgid>,
    pub source: Option<String>,

    // Diagnostic fields (preserved verbatim from pre-M5 tracing::debug).
    pub events_replayed: usize,
    pub members_count: usize,
    pub delegations_count: usize,
    pub owner_id: IdentityXgid,
    pub ai_member_role: Option<String>,
    pub ai_invited_by: Option<IdentityXgid>,
}

pub async fn ai_status(
    ctx: &mut OpContext<'_>,
    args: &crate::app::AiStatusArgs,
) -> Result<AiStatusResult> {
    use xgen_core::{resolution::derive_resolved, wire::types::EventType};

    let _identity_id = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        id.identity_id.as_str().to_string()
    };

    let node = ctx
        .node_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| ctx.session.home_node.clone());

    let events = drain_space_events(ctx, args.space.as_str()).await?;

    // R2-F01 (A-pure): re-derive the resolved SpaceState through the node's own
    // resolution engine (`derive_resolved`), replacing the pre-M8 timestamp-sort
    // + plain `apply_event` replay (the J-075 M3 carry-over). This aligns the
    // client projection with the node's resolved view under concurrency.
    //
    // The DM bail is preserved and runs BEFORE deriving: `ai status` against a
    // DM Space is an operator-resolution scope limit (M3), NOT a convergence
    // concern, and the swap MUST NOT silently enable it (CP-1a).
    let space_event = events
        .iter()
        .find(|e| {
            matches!(
                e.event_type,
                EventType::StateSpaceCreate | EventType::StateDmSpaceCreate
            )
        })
        .ok_or_else(|| {
            anyhow::anyhow!("no state.space_create event observed for {}", args.space)
        })?;
    if matches!(space_event.event_type, EventType::StateDmSpaceCreate) {
        anyhow::bail!("ai status against a DM Space is not supported in M3");
    }

    // Client vantage `""` is threaded to `apply_event` internally (F01-D3 —
    // identical to the prior non-Node behaviour); the empty `identity_home_nodes`
    // map makes Layers 3/5a/5b abstain cleanly (F01-D2) and Layer 5c guarantees a
    // deterministic, self-consistent projection.
    let state = derive_resolved(events.clone(), "", &std::collections::HashMap::new())
        .ok_or_else(|| {
            anyhow::anyhow!("no state.space_create event observed for {}", args.space)
        })?;

    let resolved = state.resolve_operator(&args.ai);
    let (operator, source) = match resolved.as_ref() {
        Some(op) => {
            // SpaceState maps are keyed by typed XGIDs (Pass 2/3); project the
            // String key to `&str` via the `Borrow<str>` additive API to look up.
            // `resolve_operator` returns `Option<String>`, so comparisons project
            // the typed delegation / inviter / owner XGIDs to `&str`.
            let stored = state.ai_operator_delegations.get(args.ai.as_str()).cloned();
            let inviter = state
                .members
                .get(args.ai.as_str())
                .and_then(|m| m.invited_by.clone());
            let label = if stored.as_ref().map(|x| x.as_str()) == Some(op.as_str()) {
                "stored delegation"
            } else if inviter.as_ref().map(|x| x.as_str()) == Some(op.as_str()) {
                "inviter fallback"
            } else if op.as_str() == state.owner_id.as_str() {
                "owner fallback"
            } else {
                "resolved"
            };
            (Some(op.clone()), Some(label.to_string()))
        }
        None => (None, None),
    };

    let ai_member_role = state
        .members
        .get(args.ai.as_str())
        .map(|m| format!("{:?}", m.role));
    let ai_invited_by = state
        .members
        .get(args.ai.as_str())
        .and_then(|m| m.invited_by.clone());

    Ok(AiStatusResult {
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        ai_identity_id: IdentityXgid::from_xgid(Xgid::new(args.ai.clone())),
        node: NodeXgid::from_xgid(Xgid::new(node)),
        operator: operator.map(|o| IdentityXgid::from_xgid(Xgid::new(o))),
        source,
        events_replayed: events.len(),
        members_count: state.members.len(),
        delegations_count: state.ai_operator_delegations.len(),
        // state.owner_id / SpaceMember.invited_by are already typed (Pass 2/3) — direct.
        owner_id: state.owner_id.clone(),
        ai_member_role,
        ai_invited_by,
    })
}

/// Drain a Space's full DAG history from the home Node via paged
/// `sync_request` / `sync_complete` (F-6 / F-7) and return the Events that
/// belong to `space` — matched by `space_id`, or by `event_id` for the create
/// events that carry an empty `space_id` on the wire. Shared by `ai_status` and
/// `members` so there is one drain, no transport drift (D-067). The caller must
/// have ensured the identity is loaded (the connection authenticates with it).
async fn drain_space_events(
    ctx: &mut OpContext<'_>,
    space: &str,
) -> Result<Vec<xgen_core::wire::types::Event>> {
    use xgen_core::{
        transport::connection::Inbound,
        wire::types::{Event, TransportMessage},
    };

    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    let mut events: Vec<Event> = Vec::new();
    {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;

        let mut since = String::new();
        let deadline = tokio::time::Instant::now() + sync_timeout;
        'pages: loop {
            let sync_req = TransportMessage::SyncRequest {
                protocol_version: "0.1".to_string(),
                since: since.clone(),
                limit: None,
            };
            conn.send_transport(&sync_req)
                .await
                .context("failed to send sync_request")?;

            // F-6 / F-7: drain one page of the response, terminated by an
            // explicit SyncComplete (replacing the prior 5s hardcoded
            // deadline-as-completion-signal). The outer 'pages loop chases
            // continue_from cursors until catch-up is complete.
            let continue_from = loop {
                match tokio::time::timeout_at(deadline, conn.recv()).await {
                    Ok(Ok(Inbound::Event(ev))) => {
                        // state.space_create / state.dm_space_create carry empty
                        // space_id on the wire; identify via event_id == space.
                        let in_space = ev.space_id.as_str() == space
                            || ev.event_id.as_deref().map(|x| x.as_str()) == Some(space);
                        if in_space {
                            events.push(ev);
                        }
                    }
                    Ok(Ok(Inbound::Transport(TransportMessage::SyncComplete {
                        continue_from,
                        ..
                    }))) => break continue_from,
                    Ok(Ok(Inbound::Transport(TransportMessage::Goodbye { .. })))
                    | Ok(Ok(Inbound::Closed)) => break 'pages,
                    Ok(Ok(_)) => {}
                    Ok(Err(_)) => break 'pages,
                    Err(_) => {
                        // F-6b safety-net.
                        anyhow::bail!(
                            "sync_request safety-net timeout — peer never sent sync_complete \
                             within {} ms",
                            sync_timeout.as_millis()
                        );
                    }
                }
            };
            match continue_from {
                Some(cursor) => since = cursor,
                None => break 'pages,
            }
        }
        let _ = conn.goodbye("client_disconnect").await;
    }
    Ok(events)
}

/// One member row in [`MembersResult`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberEntry {
    pub identity_id: IdentityXgid,
    pub role: xgen_core::space::membership::Role,
    pub joined_at: String,
    /// Identity that admitted this member (`None` for the owner / un-invited joins).
    pub invited_by: Option<IdentityXgid>,
}

/// Result of `members` — the resolved membership of a Space as observed by the
/// queried Node, derived by causal replay (covers DM Spaces, M7C-D3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembersResult {
    pub space_id: SpaceXgid,
    pub is_dm: bool,
    pub owner_id: IdentityXgid,
    pub members: Vec<MemberEntry>,
    pub events_replayed: usize,
}

/// Pure causal-replay projection: seed a `SpaceState` from the Space's root
/// create event, apply the remaining events in causal order, and project the
/// resolved membership. R2-F01 (A-pure): the projection re-derives through the
/// node's own resolution engine (`derive_resolved`), which dispatches
/// `from_dm_space_create_node` (the key-less M7C-D4 constructor) for DM Spaces
/// vs `from_space_create` otherwise — so `members` covers DM Spaces — unlike
/// `ai_status`, whose DM bail is operator-resolution-specific, not a
/// membership-read limit. Pure over a `&[Event]`, so the projection is
/// unit-tested without a live Node (the drain itself shares `ai_status`'s
/// already-exercised transport path).
fn members_projection(
    space: &str,
    events: &[xgen_core::wire::types::Event],
) -> Result<MembersResult> {
    use xgen_core::resolution::derive_resolved;

    // Re-derive the resolved SpaceState exactly as the node does, replacing the
    // pre-M8 timestamp-sort + plain `apply_event` replay (J-075). `derive_resolved`
    // threads the client vantage `""` to `apply_event` internally (F01-D3), and
    // the empty `identity_home_nodes` map makes Layers 3/5a/5b abstain cleanly
    // (F01-D2); Layer 5c guarantees a deterministic, self-consistent projection.
    let state = derive_resolved(events.to_vec(), "", &std::collections::HashMap::new())
        .ok_or_else(|| anyhow::anyhow!("no state.space_create event observed for {}", space))?;

    let mut members: Vec<MemberEntry> = state
        .members
        .values()
        .map(|m| MemberEntry {
            identity_id: m.identity_id.clone(),
            role: m.role.clone(),
            joined_at: m.joined_at.clone(),
            invited_by: m.invited_by.clone(),
        })
        .collect();
    // Deterministic order (HashMap iteration is unordered).
    members.sort_by(|a, b| a.identity_id.as_str().cmp(b.identity_id.as_str()));

    Ok(MembersResult {
        space_id: SpaceXgid::from_xgid(Xgid::new(space.to_string())),
        is_dm: state.is_dm,
        owner_id: state.owner_id.clone(),
        members,
        events_replayed: events.len(),
    })
}

/// List the resolved membership of a Space (M7C-D3, A1). Drains the Space's DAG
/// history from the queried Node and projects `state.members` after causal
/// replay. Covers DM Spaces.
pub async fn members(
    ctx: &mut OpContext<'_>,
    args: &crate::app::MembersArgs,
) -> Result<MembersResult> {
    let events = drain_space_events(ctx, args.space.as_str()).await?;
    members_projection(args.space.as_str(), &events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use xgen_common::state::ClientState;

    fn write_state(dir: &Path, state: &ClientState) {
        let path = dir.join("xgen-client_state.json");
        fs::write(path, serde_json::to_string_pretty(state).unwrap()).unwrap();
    }

    #[test]
    fn whoami_projects_state_subset() {
        let dir = tempdir().unwrap();
        let state = ClientState {
            identity_id: "xgen://pubkey/ed25519:abc".into(),
            display_name: "alice".into(),
            version: "0.10.3".into(),
            build: "deadbeef".into(),
            home_node: "ws://127.0.0.1:8080/xgen".into(),
            updated_at: "2026-05-17T00:00:00.000Z".into(),
            spaces: vec![],
        };
        write_state(dir.path(), &state);

        let mut session = SessionState::new(String::new(), dir.path().to_path_buf());
        let mut ctx = OpContext {
            session: &mut session,
            data_dir: dir.path(),
            node_override: None,
        };
        let r = whoami(&mut ctx).unwrap();
        assert_eq!(r.identity_id.as_str(), "xgen://pubkey/ed25519:abc");
        assert_eq!(r.display_name, "alice");
        assert_eq!(r.home_node.as_str(), "ws://127.0.0.1:8080/xgen");
        assert_eq!(r.spaces_joined, 0);
    }

    #[test]
    fn whoami_missing_state_file_errors() {
        let dir = tempdir().unwrap();
        let mut session = SessionState::new(String::new(), dir.path().to_path_buf());
        let mut ctx = OpContext {
            session: &mut session,
            data_dir: dir.path(),
            node_override: None,
        };
        let err = whoami(&mut ctx).unwrap_err();
        assert!(err.to_string().contains("state file not found"));
    }

    #[test]
    fn spaces_returns_known_spaces() {
        use xgen_common::state::{KnownRoom, KnownSpace};
        let dir = tempdir().unwrap();
        let state = ClientState {
            identity_id: "xgen://pubkey/ed25519:x".into(),
            display_name: "carol".into(),
            version: "0.10.3".into(),
            build: "feed".into(),
            home_node: "ws://127.0.0.1:8082/xgen".into(),
            updated_at: "2026-05-17T00:00:00.000Z".into(),
            spaces: vec![KnownSpace {
                space_id: "xgen://hash/sha256:abc".into(),
                name: "Test Space".into(),
                node_endpoint: "ws://127.0.0.1:8082/xgen".into(),
                role: "owner".into(),
                rooms: vec![KnownRoom {
                    room_id: "xgen://hash/sha256:def".into(),
                    name: "general".into(),
                    joined: true,
                }],
            }],
        };
        write_state(dir.path(), &state);

        let mut session = SessionState::new(String::new(), dir.path().to_path_buf());
        let mut ctx = OpContext {
            session: &mut session,
            data_dir: dir.path(),
            node_override: None,
        };
        let r = spaces(&mut ctx).unwrap();
        assert_eq!(r.spaces.len(), 1);
        assert_eq!(r.spaces[0].name, "Test Space");
        assert_eq!(r.spaces[0].rooms.len(), 1);
        assert_eq!(r.spaces[0].rooms[0].name, "general");
    }

    fn state_with_one_space() -> ClientState {
        use xgen_common::state::{KnownRoom, KnownSpace};
        ClientState {
            identity_id: "xgen://pubkey/ed25519:x".into(),
            display_name: "carol".into(),
            version: "0.10.3".into(),
            build: "feed".into(),
            home_node: "ws://127.0.0.1:8082/xgen".into(),
            updated_at: "2026-05-17T00:00:00.000Z".into(),
            spaces: vec![KnownSpace {
                space_id: "xgen://hash/sha256:abc".into(),
                name: "Test Space".into(),
                node_endpoint: "ws://127.0.0.1:8082/xgen".into(),
                role: "owner".into(),
                rooms: vec![
                    KnownRoom {
                        room_id: "xgen://hash/sha256:def".into(),
                        name: "general".into(),
                        joined: true,
                    },
                    KnownRoom {
                        room_id: "xgen://hash/sha256:ghi".into(),
                        name: "random".into(),
                        joined: false,
                    },
                ],
            }],
        }
    }

    #[test]
    fn rooms_returns_rooms_for_matching_space() {
        let dir = tempdir().unwrap();
        write_state(dir.path(), &state_with_one_space());

        let mut session = SessionState::new(String::new(), dir.path().to_path_buf());
        let mut ctx = OpContext {
            session: &mut session,
            data_dir: dir.path(),
            node_override: None,
        };
        let args = crate::app::RoomsArgs {
            space: "xgen://hash/sha256:abc".into(),
        };
        let r = rooms(&mut ctx, &args).unwrap();
        assert_eq!(r.space_id, "xgen://hash/sha256:abc");
        assert_eq!(r.space_name, "Test Space");
        assert_eq!(r.rooms.len(), 2);
        assert_eq!(r.rooms[0].name, "general");
        assert_eq!(r.rooms[1].name, "random");
    }

    #[test]
    fn rooms_errors_on_unknown_space() {
        let dir = tempdir().unwrap();
        write_state(dir.path(), &state_with_one_space());

        let mut session = SessionState::new(String::new(), dir.path().to_path_buf());
        let mut ctx = OpContext {
            session: &mut session,
            data_dir: dir.path(),
            node_override: None,
        };
        let args = crate::app::RoomsArgs {
            space: "xgen://hash/sha256:does-not-exist".into(),
        };
        let err = rooms(&mut ctx, &args).unwrap_err();
        assert!(err.to_string().contains("no known Space"));
    }

    #[test]
    fn status_projects_state_with_age() {
        let dir = tempdir().unwrap();
        let state = ClientState {
            identity_id: "xgen://pubkey/ed25519:def".into(),
            display_name: "bob".into(),
            version: "0.10.3".into(),
            build: "cafe".into(),
            home_node: "ws://127.0.0.1:8081/xgen".into(),
            // Far-past timestamp so age is comfortably > 30s and stable.
            updated_at: "2020-01-01T00:00:00.000Z".into(),
            spaces: vec![],
        };
        write_state(dir.path(), &state);

        let mut session = SessionState::new(String::new(), dir.path().to_path_buf());
        let mut ctx = OpContext {
            session: &mut session,
            data_dir: dir.path(),
            node_override: None,
        };
        let r = status(&mut ctx).unwrap();
        assert_eq!(r.identity_id.as_str(), "xgen://pubkey/ed25519:def");
        assert_eq!(r.display_name, "bob");
        assert_eq!(r.version, "0.10.3");
        assert_eq!(r.home_node.as_str(), "ws://127.0.0.1:8081/xgen");
        assert_eq!(r.spaces_joined, 0);
        assert!(r.state_file_age_seconds > 30);
    }
}

#[cfg(test)]
mod pass_4_commit_1_tests {
    //! XGID Retrofit Pass 4 Commit 1 — Surface #1 (M5 Ops Layer) per-surface
    //! tests T1 + T2 (runbook §3.4). T3 lives at xgen-common flavours.rs.
    use super::*;

    fn ix(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn sx(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn ex(s: &str) -> EventXgid {
        EventXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn rx(s: &str) -> RoomXgid {
        RoomXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn nx(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }

    /// T1 — compile-time witness that all 49 String slots across the 13
    /// Result structs + `HistoryMessage` classify correctly per design doc
    /// §4.1.a: every identifier slot accepts a typed XGID, every descriptive
    /// slot accepts a `String`. A misclassification would fail to compile.
    #[test]
    fn ops_result_struct_field_retype_49_slots_compile() {
        let _ = WhoamiResult {
            identity_id: ix("i"),
            display_name: "n".into(),
            home_node: nx("ws://h"),
            spaces_joined: 0,
        };
        let _ = StatusResult {
            identity_id: ix("i"),
            display_name: "n".into(),
            version: "0.1".into(),
            home_node: nx("ws://h"),
            spaces_joined: 0,
            state_file_age_seconds: 0,
        };
        let _ = SpacesResult { spaces: vec![] };
        let _ = RegisterResult {
            identity_id: ix("i"),
            display_name: "n".into(),
            home_node: nx("ws://h"),
            registered_at: "t".into(),
            is_ai: false,
        };
        let _ = CreateSpaceResult {
            space_id: sx("s"),
            event_id: ex("e"),
            name: "n".into(),
            owner_identity_id: ix("i"),
        };
        let _ = CreateRoomResult {
            room_id: rx("r"),
            event_id: ex("e"),
            space_id: sx("s"),
            name: "n".into(),
        };
        let _ = InviteResult {
            event_id: ex("e"),
            target_identity: ix("i"),
            space_id: sx("s"),
            role: "member".into(),
        };
        let _ = JoinResult {
            event_id: ex("e"),
            space_id: sx("s"),
            room_id: Some(rx("r")),
        };
        let _ = SendResult {
            event_id: ex("e"),
            space_id: sx("s"),
            room_id: rx("r"),
        };
        let hm = HistoryMessage {
            sender: ix("i"),
            timestamp: "t".into(),
            text: "hi".into(),
        };
        let _ = HistoryResult {
            space_id: sx("s"),
            room_id: rx("r"),
            messages: vec![hm],
        };
        let _ = AiDelegateResult {
            event_id: ex("e"),
            space_id: sx("s"),
            ai_identity_id: ix("i"),
            new_operator: ix("o"),
        };
        let _ = AiRevokeResult {
            event_id: ex("e"),
            space_id: sx("s"),
            ai_identity_id: ix("i"),
        };
        let _ = AiStatusResult {
            space_id: sx("s"),
            ai_identity_id: ix("i"),
            node: nx("ws://h"),
            operator: Some(ix("o")),
            source: Some("delegation".into()),
            events_replayed: 0,
            members_count: 0,
            delegations_count: 0,
            owner_id: ix("ow"),
            ai_member_role: Some("member".into()),
            ai_invited_by: Some(ix("inv")),
        };
    }

    /// T2 — LOAD-BEARING wire-format invariance witness (Joe-lock checkpoint
    /// #2). Each typed flavour wrapper is `#[serde(transparent)]`, so a
    /// post-Pass-4 Result struct serialises to byte-identical JSON as the
    /// pre-Pass-4 String-field shape: identifier slots appear as plain JSON
    /// strings, never nested objects. A pre-Pass-4 consumer reads the same
    /// bytes.
    #[test]
    fn ops_result_struct_serde_transparent_wire_invariance() {
        let r = CreateSpaceResult {
            space_id: sx("xgen://hash/sha256:abc"),
            event_id: ex("xgen://hash/sha256:evt"),
            name: "General".into(),
            owner_identity_id: ix("xgen://pubkey/ed25519:OWNER"),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(
            json.contains(r#""space_id":"xgen://hash/sha256:abc""#),
            "space_id not a plain string: {json}"
        );
        assert!(
            json.contains(r#""event_id":"xgen://hash/sha256:evt""#),
            "event_id not a plain string: {json}"
        );
        assert!(
            json.contains(r#""owner_identity_id":"xgen://pubkey/ed25519:OWNER""#),
            "owner_identity_id not a plain string: {json}"
        );
        // Round-trips back through the typed shape.
        let back: CreateSpaceResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.space_id.as_str(), "xgen://hash/sha256:abc");
        assert_eq!(back.name, "General");
    }

    // ── members_projection (A1) ───────────────────────────────────────────────
    //
    // The network drain (`drain_space_events`) shares `ai_status`'s already-
    // exercised transport path and needs a live Node; the testable substance of
    // `members` is the pure causal-replay projection, covered here over
    // hand-built event sequences (regular Space, DM Space, unknown Space).

    use xgen_core::identity::{keypair, registration::identity_id_from_key};
    use xgen_core::space::membership::Role;
    use xgen_core::space::state::{
        build_dm_space_create_event, build_membership_event, build_space_create_event, sign_event,
        SpaceState,
    };
    use xgen_core::wire::types::{Event, EventType};

    const TEST_HOME: &str = "xgen://pubkey/ed25519:NODE";

    fn member_with(r: &MembersResult, id: &str) -> MemberEntry {
        r.members
            .iter()
            .find(|m| m.identity_id.as_str() == id)
            .cloned()
            .unwrap_or_else(|| panic!("{id} not in members: {:?}", r.members))
    }

    #[test]
    fn members_projection_regular_space_owner_and_joiner() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let alice_id = identity_id_from_key(&alice);
        let bob_id = identity_id_from_key(&bob);

        let create =
            sign_event(build_space_create_event(&alice, "Team", None, 1, TEST_HOME, None, false), &alice);
        let space_id = create.event_id.clone().unwrap().as_str().to_string();
        // Alice (owner) invites Bob, Bob joins (space-level), tip-chained:
        // space_create ← invite ← join. Production never builds these unlinked
        // (`ops::invite` / `ops::join` tip-chain via `get_dag_tips`); under the
        // ancestry-aware `derive_resolved` (R2-F01) an empty-prev invite+join
        // would be a spurious concurrent conflict on one membership key.
        let mut invite_unsigned = build_membership_event(
            &alice,
            &space_id,
            "",
            EventType::MembershipInvite,
            serde_json::json!({ "target_identity": bob_id, "role": "member" }),
        );
        invite_unsigned.prev_events = vec![ex(&space_id)];
        let invite = sign_event(invite_unsigned, &alice);
        let invite_id = invite.event_id.clone().unwrap().as_str().to_string();
        let mut join_unsigned =
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, serde_json::json!({}));
        join_unsigned.prev_events = vec![ex(&invite_id)];
        let join = sign_event(join_unsigned, &bob);
        let events: Vec<Event> = vec![create, invite, join];

        let r = members_projection(&space_id, &events).unwrap();
        assert!(!r.is_dm);
        assert_eq!(r.owner_id.as_str(), alice_id);
        assert_eq!(r.members.len(), 2);
        assert_eq!(member_with(&r, &alice_id).role, Role::Owner);
        let bob_m = member_with(&r, &bob_id);
        assert_eq!(bob_m.role, Role::Member);
        assert_eq!(bob_m.invited_by.as_ref().map(|x| x.as_str()), Some(alice_id.as_str()));
        assert_eq!(r.events_replayed, 3);
    }

    #[test]
    fn members_projection_dm_space_covers_both_members() {
        // The DM coverage A1 adds over ai_status (which bails on DM). Exercises
        // the key-less from_dm_space_create_node seed + auto-room + the
        // DM-rejected auto-invite + the invitee's join.
        let alice = keypair::generate();
        let bob = keypair::generate();
        let alice_id = identity_id_from_key(&alice);
        let bob_id = identity_id_from_key(&bob);

        let create =
            sign_event(build_dm_space_create_event(&alice, &bob_id, TEST_HOME), &alice);
        let space_id = create.event_id.clone().unwrap().as_str().to_string();
        // Use the constructor's genuine auto-room, but rebuild the auto-invite
        // tip-chained to that room — mirroring `ops::create_dm_space`. The
        // constructor's own auto-invite carries empty prev_events (a known D-065
        // latent bug, out of C1 scope); production rebuilds it instead. Full
        // causal chain: dm_space_create ← room ← invite ← join.
        let (_authoring, room_ev, _constructor_invite) =
            SpaceState::from_dm_space_create(&create, &alice).unwrap();
        let room_id = room_ev.event_id.clone().unwrap().as_str().to_string();
        let mut invite_unsigned = build_membership_event(
            &alice,
            &space_id,
            &room_id,
            EventType::MembershipInvite,
            serde_json::json!({ "target_identity": bob_id, "role": "member" }),
        );
        invite_unsigned.prev_events = vec![ex(&room_id)];
        let invite_ev = sign_event(invite_unsigned, &alice);
        let invite_id = invite_ev.event_id.clone().unwrap().as_str().to_string();
        let mut join_unsigned =
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, serde_json::json!({}));
        join_unsigned.prev_events = vec![ex(&invite_id)];
        let join = sign_event(join_unsigned, &bob);
        let events: Vec<Event> = vec![create, room_ev, invite_ev, join];

        let r = members_projection(&space_id, &events).unwrap();
        assert!(r.is_dm);
        assert_eq!(r.owner_id.as_str(), alice_id);
        assert_eq!(r.members.len(), 2);
        assert_eq!(member_with(&r, &alice_id).role, Role::Owner);
        let bob_m = member_with(&r, &bob_id);
        assert_eq!(bob_m.role, Role::Member);
        // invited_by survives even though the auto-invite is rejected under DM
        // constraints — the pending invite is seeded at construction.
        assert_eq!(bob_m.invited_by.as_ref().map(|x| x.as_str()), Some(alice_id.as_str()));
    }

    #[test]
    fn members_projection_errors_when_no_create_event() {
        let bob = keypair::generate();
        // A stray join with no observed root create event.
        let join = sign_event(
            build_membership_event(&bob, "xgen://hash/sha256:unknown", "", EventType::MembershipJoin, serde_json::json!({})),
            &bob,
        );
        let events: Vec<Event> = vec![join];
        let err = members_projection("xgen://hash/sha256:unknown", &events).unwrap_err();
        assert!(
            err.to_string().contains("no state.space_create event observed"),
            "unexpected error: {err}"
        );
    }

    /// Full-factorial permutations (n ≤ 5) — the §3.9.2 arrival-order harness,
    /// mirrors `resolution::derive`'s own test helper.
    fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
        if items.len() <= 1 {
            return vec![items.to_vec()];
        }
        let mut out = Vec::new();
        for i in 0..items.len() {
            let mut rest = items.to_vec();
            let picked = rest.remove(i);
            for mut p in permutations(&rest) {
                p.insert(0, picked.clone());
                out.push(p);
            }
        }
        out
    }

    /// R2-F01 permutation-convergence proof (Arc-C mirror, client read path).
    /// A concurrent same-key conflict — `join(bob)` vs `ban(bob)`, both
    /// referencing the create root so neither is the other's ancestor — derives
    /// ONE identical membership under every arrival permutation via
    /// `members_projection`, and that membership matches the node engine's
    /// `derive_resolved` winner (Layer 1: ban > join; needs no home-node map).
    /// This is what the pre-M8 timestamp-sort + plain `apply_event` replay could
    /// not guarantee — the gap R2-F01 closes.
    #[test]
    fn members_projection_concurrent_ban_join_converges_under_all_permutations() {
        use xgen_core::resolution::derive_resolved;

        let alice = keypair::generate();
        let bob = keypair::generate();
        let alice_id = identity_id_from_key(&alice);
        let bob_id = identity_id_from_key(&bob);

        let create =
            sign_event(build_space_create_event(&alice, "Team", None, 1, TEST_HOME, None, false), &alice);
        let space_id = create.event_id.clone().unwrap().as_str().to_string();
        // Concurrent: Bob joins, Owner bans Bob — both reference the create root,
        // so neither is a causal ancestor of the other → a genuine conflict on
        // membership:{space}:bob that resolution (not arrival order) must settle.
        let mut join_unsigned =
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, serde_json::json!({}));
        join_unsigned.prev_events = vec![ex(&space_id)];
        let join = sign_event(join_unsigned, &bob);
        let mut ban_unsigned = build_membership_event(
            &alice,
            &space_id,
            "",
            EventType::MembershipBan,
            serde_json::json!({ "target_identity": bob_id }),
        );
        ban_unsigned.prev_events = vec![ex(&space_id)];
        let ban = sign_event(ban_unsigned, &alice);

        let events = vec![create, join, ban];

        // Every arrival permutation derives a byte-identical MembersResult.
        let reference =
            serde_json::to_string(&members_projection(&space_id, &events).unwrap()).unwrap();
        for perm in permutations(&events) {
            let got = members_projection(&space_id, &perm).unwrap();
            assert_eq!(
                serde_json::to_string(&got).unwrap(),
                reference,
                "members_projection must converge to one identical membership under every \
                 arrival permutation (§3.9.2)"
            );
            // Layer 1 winner: ban dominates the concurrent join — Bob is not a member.
            assert!(
                !got.members.iter().any(|m| m.identity_id.as_str() == bob_id),
                "ban must win — Bob must not appear as a member"
            );
            assert!(
                got.members.iter().any(|m| m.identity_id.as_str() == alice_id),
                "owner Alice remains a member"
            );
        }

        // The client read agrees with the node engine: derive_resolved (the same
        // function `members_projection` wraps) elects ban — Bob banned, not a member.
        let node_state = derive_resolved(events.clone(), "", &std::collections::HashMap::new())
            .expect("scenario has a create event");
        assert!(node_state.banned.contains(&ix(&bob_id)), "node winner: Bob is banned");
        assert!(
            !node_state.members.contains_key(&ix(&bob_id)),
            "node winner: Bob is not a member"
        );
    }
}
