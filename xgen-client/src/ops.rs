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
        identity::registration::{build_register, build_register_with_ai, sign_register},
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
        build_space_create_event(&signing_key, &args.name, None, 1, &home_node),
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

    // Scoped connection borrow so ctx.session is free for the state write.
    {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;

        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(space_id.clone()),
        };
        trace_event(&space_ev, EventDirection::Out, &session_ctx);

        conn.send_event(&space_ev)
            .await
            .context("failed to send space_create event")?;
        tracing::info!(space_id = %space_id, name = %args.name, "Space created");

        let _ = conn.goodbye("client_disconnect").await;
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

    {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        let session_ctx = SessionContext {
            identity_id: Some(identity_id.clone()),
            role: Some(SpaceRole::Owner),
            space_id: Some(args.space.clone()),
        };
        trace_event(&room_ev, EventDirection::Out, &session_ctx);
        conn.send_event(&room_ev)
            .await
            .context("failed to send room_create event")?;
        let _ = conn.goodbye("client_disconnect").await;
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
                json!({ "target_identity": args.identity, "role": args.role }),
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
        conn.send_event(&invite_ev)
            .await
            .context("failed to send invite event")?;
        let _ = conn.goodbye("client_disconnect").await;
        id_for_result
    };

    Ok(InviteResult {
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        target_identity: IdentityXgid::from_xgid(Xgid::new(args.identity.clone())),
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        role: args.role.clone(),
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
        // Tip-chain the join so it lands after the inviting
        // `membership.invite` and resolve_operator sees the correct
        // invited_by after replay.
        let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
            .await
            .unwrap_or_else(|_| vec![args.space.clone()]);
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
        conn.send_event(&join_ev)
            .await
            .context("failed to send join event")?;
        tracing::info!(space_id = %args.space, "Joined Space");
        let _ = conn.goodbye("client_disconnect").await;
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

// ── send ──────────────────────────────────────────────────────────────────────

/// Result of `ops::send`. The pre-M5 `cmd_send` did not await an ack —
/// "Message sent." just means the event was written to the WebSocket;
/// the Node-side accept happens asynchronously. M5 preserves that
/// shape; M7 may introduce a structured ack path.
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
        conn.send_event(&msg_ev)
            .await
            .context("failed to send message")?;
        tracing::info!(room = %args.room, "Message sent");
        let _ = conn.goodbye("client_disconnect").await;
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
        conn.send_event(&ev)
            .await
            .context("failed to send delegate event")?;
        let _ = conn.goodbye("client_disconnect").await;
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
        conn.send_event(&ev)
            .await
            .context("failed to send revoke event")?;
        let _ = conn.goodbye("client_disconnect").await;
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
    use xgen_core::{
        space::state::SpaceState,
        transport::connection::Inbound,
        wire::types::{Event, EventType, TransportMessage},
    };

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
                        // space_id on the wire; identify via event_id == args.space.
                        let in_space = ev.space_id.as_str() == args.space.as_str()
                            || ev.event_id.as_deref().map(|x| x.as_str())
                                == Some(args.space.as_str());
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

    // Causal replay: build SpaceState from the root, then apply other events
    // in timestamp order (matches the pre-M5 workaround for HashMap-iteration
    // determinism of the Node's EventStore — see J-075 M3 carry-over).
    let space_event = events
        .iter()
        .find(|e| {
            matches!(
                e.event_type,
                EventType::StateSpaceCreate | EventType::StateDmSpaceCreate
            )
        })
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("no state.space_create event observed for {}", args.space)
        })?;

    let mut state = if matches!(space_event.event_type, EventType::StateDmSpaceCreate) {
        anyhow::bail!("ai status against a DM Space is not supported in M3");
    } else {
        SpaceState::from_space_create(&space_event)
            .context("failed to derive SpaceState from observed state.space_create")?
    };

    let mut sorted: Vec<&Event> = events.iter().collect();
    sorted.sort_by(|a, b| {
        let a_root = matches!(
            a.event_type,
            EventType::StateSpaceCreate | EventType::StateDmSpaceCreate
        );
        let b_root = matches!(
            b.event_type,
            EventType::StateSpaceCreate | EventType::StateDmSpaceCreate
        );
        match (a_root, b_root) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.timestamp.cmp(&b.timestamp),
        }
    });
    for ev in sorted {
        if matches!(
            ev.event_type,
            EventType::StateSpaceCreate | EventType::StateDmSpaceCreate
        ) {
            continue;
        }
        // Client vantage: not a Node — pass `""` so `apply_federation_add`'s
        // D-075 vantage check falls into the else branch (verbatim pre-D-075
        // behaviour for non-Node observers).
        let _ = state.apply_event(ev, "");
    }

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
}
