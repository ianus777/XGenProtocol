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
        last_local_events: Default::default(),
    }
}

/// MP-F7-D3/D4 — the rejoin-anchor fallback decision. When `get_dag_tips` yields
/// nothing (a just-left non-member starved by member-gated sync), anchor the
/// rejoin after this client's own persisted last local event for the Space (its
/// leave) → a linear `j→lv→rj` chain not concurrent with the leave on
/// `membership:{space}:identity`. Absent (true first join / fresh / cleared
/// state) → the create root, exactly as before. Best-effort: never an error.
fn rejoin_anchor_or_root(state: &xgen_common::state::ClientState, space: &str) -> Vec<String> {
    match state.last_local_events.get(space) {
        Some(anchor) => vec![anchor.clone()],
        None => vec![space.to_string()],
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
            // BOUNDARY (`EventConfirm::Rejected.event_id`, wire) → INTERNAL:
            // project once here (D-137 §1 clause 3).
            event_id: EventXgid::from_xgid(Xgid::new(event_id)),
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
    pub event_id: EventXgid,
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
    pub space_id: SpaceXgid,
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
        // `KnownSpace.space_id` is the external `String` form; project at the
        // result boundary (D-137 §1 clause 3, one projection per direction).
        space_id: SpaceXgid::from_xgid(Xgid::new(space.space_id)),
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
        last_local_events: Default::default(),
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

// ── identity_get ────────────────────────────────────────────────────────────────

/// A faithful projection of the `identity.record` wire response (spec 3.6.7).
///
/// Mirrors the SEVEN fields the wire actually carries — and ONLY those.
/// `identity.record` deliberately does NOT carry `update_version`, `revoked`,
/// or `trust_assertion` (measured `1fd594c`, re-confirmed `167055d`; code and
/// Appendix I §IV.1 agree — this is decided, not drift). This type is the
/// fetch boundary and must never *claim* data the wire did not deliver.
///
/// The address book's book-local fields for the wire-absent locked rules
/// (§5 V2 / revocation-on-encounter, §6 not-renewed) live on `SeenRecord`,
/// NOT here (M-RP-ADDRESS-BOOK Leg D, runbook §2 wire ceiling). `M13 Client
/// Identity Lookup Widening` widens the wire later; when it lands, those
/// fields become field-mapping on top of this type rather than new design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedIdentity {
    pub identity_id: IdentityXgid,
    pub display_name: Option<String>,
    pub registered_at: String,
    pub devices: Vec<xgen_core::wire::types::IdentityDeviceEntry>,
    pub home_node: NodeXgid,
    /// AI declaration (spec 3.6.10). Serde-defaults to `false` when the wire
    /// omits it, matching the human-record byte-identity rule (`types.rs:468`).
    pub is_ai: bool,
    pub ai_capabilities: Option<xgen_common::wire::AiCapabilities>,
}

/// Pure response parser for `identity_get`, split out so the
/// Record / NotFound / unexpected branching is unit-testable without a live
/// Node — the `members_projection` pattern (network shell thin, substance pure).
fn parse_identity_get_response(
    inbound: xgen_core::transport::connection::Inbound,
) -> Result<Option<FetchedIdentity>> {
    use xgen_core::{transport::connection::Inbound, wire::types::IdentityMessage};
    match inbound {
        Inbound::Identity(IdentityMessage::Record {
            identity_id,
            display_name,
            registered_at,
            devices,
            home_node,
            is_ai,
            ai_capabilities,
            ..
        }) => Ok(Some(FetchedIdentity {
            // The one projection per direction (D-137 §1): the wire delivers
            // `identity_id` / `home_node` as `String`; they cross into the typed
            // form HERE, at the fetch boundary, and are typed everywhere after.
            identity_id: IdentityXgid::from_xgid(Xgid::new(identity_id)),
            display_name,
            registered_at,
            devices,
            home_node: NodeXgid::from_xgid(Xgid::new(home_node)),
            is_ai,
            ai_capabilities,
        })),
        // NotFound is a NORMAL outcome — an identity the node has never seen.
        // `Ok(None)`, never an error (runbook §4 Step 1). It also carries no
        // "revoked" meaning: a revoked identity still returns its Record
        // (D-127); NotFound is reserved for never-existed / erased.
        Inbound::Identity(IdentityMessage::NotFound { .. }) => Ok(None),
        other => anyhow::bail!("unexpected response to identity.get: {:?}", other),
    }
}

/// How long to wait for the `identity.record`/`identity.not_found` reply to one
/// `identity.get` before giving up on it (M-RP-MEMBERS Leg A-bis / T1).
///
/// `conn.recv()` had no timeout, so a node that accepted the request and never
/// answered hung the fetch loop — and thus a background address-book fill — for
/// the life of the process. 10 s by the shape analogue `resident::SEND_ACK_TIMEOUT`
/// (one request, one reply, over the socket). This is the FIRST home for the
/// identity.get recv policy, so it is a named local constant rather than a reuse
/// of `SEND_ACK_TIMEOUT`: identity.get and the resident's send-ack correlation
/// are distinct policies that merely share a value, and coupling them to one
/// constant would let tuning one silently retune the other (the inverse D-067).
const IDENTITY_GET_RECV_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(10);

/// Send one `identity.get` on an ALREADY-OPEN connection and parse the reply.
///
/// The connection is neither opened nor closed here — the caller owns its
/// lifecycle. This is what lets [`fill_from_space`] batch many lookups on a
/// single connection (the `drain_space_events` pattern) rather than a
/// connect+goodbye per identity: `goodbye` closes the WebSocket
/// (`connection.rs`), and `ensure_connected` reuses a `Some` connection
/// without reconnecting, so a goodbye between lookups would strand the loop.
async fn identity_get_on(
    conn: &mut crate::session::ClientConnection,
    identity_id: &str,
) -> Result<Option<FetchedIdentity>> {
    use xgen_core::wire::types::IdentityMessage;
    let msg = IdentityMessage::Get {
        protocol_version: "0.1".to_string(),
        identity_id: identity_id.to_string(),
    };
    conn.send_identity(&msg)
        .await
        .context("failed to send identity.get")?;
    // Bound the reply wait (Leg A-bis / T1): a node that accepts the request
    // and never answers must not hang the fetch loop forever. The `?` on the
    // loop caller (`fill_from_events`) means the FIRST timeout aborts the whole
    // loop, so a dead node costs one bound (~10 s), not N (§2a).
    let inbound = match tokio::time::timeout(IDENTITY_GET_RECV_TIMEOUT, conn.recv()).await {
        Ok(r) => r.context("no response from Node")?,
        Err(_elapsed) => anyhow::bail!(
            "identity.get timed out after {}s",
            IDENTITY_GET_RECV_TIMEOUT.as_secs()
        ),
    };
    parse_identity_get_response(inbound)
}

/// Fetch an Identity record from the home Node (spec 3.6.7).
///
/// **Precondition:** `ctx.session.identity` loaded by the dispatcher
/// (`SessionState::ensure_identity`); `ensure_connected` authenticates as that
/// identity. Mirrors `ops::register`'s request/response shape.
///
/// - `identity.record` ⇒ `Ok(Some(FetchedIdentity))`.
/// - `identity.not_found` ⇒ `Ok(None)` — a normal outcome, not an error.
/// - anything else ⇒ `bail!`.
///
/// Best-effort `goodbye` on completion (M5 one-shot semantics).
pub async fn identity_get(
    ctx: &mut OpContext<'_>,
    identity_id: &str,
) -> Result<Option<FetchedIdentity>> {
    let conn = ctx.session.ensure_connected(ctx.node_override).await?;
    let result = identity_get_on(conn, identity_id).await;
    // Courtesy goodbye — best-effort, errors swallowed (matches register).
    let _ = conn.goodbye("client_disconnect").await;
    result
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

    // The client's own dial-endpoint record stays a transport URL.
    let home_node_url = ctx
        .node_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| ctx.session.home_node.clone());

    // M10.4 (MP-F13, Shape B): connect first so the AuthOk node_id echo is
    // captured, then write the Node's pubkey node_id — NOT the transport URL —
    // into the Space's signed content["home_node"] (every consumer + both
    // migration gates expect the pubkey). Connect-before-build is required:
    // the value depends on the connection actually used (honours
    // --node-override). Refuse rather than silently writing a URL.
    ctx.session.ensure_connected(ctx.node_override).await?;
    let home_node_id = ctx.session.node_id.clone().ok_or_else(|| {
        anyhow::anyhow!(
            "home Node did not advertise its node_id (older protocol?); refusing to \
             create a Space with a transport URL as home_node"
        )
    })?
    // `SessionState.node_id` is now `Option<NodeXgid>`; project back to the
    // `String` form the rest of this function (KnownSpace, the signed
    // `content["home_node"]`, tracing) already carries — D-137, one
    // projection per direction, mirroring the space_id projection below.
    .as_str()
    .to_string();

    // Build + sign the space_create event locally so the assigned IDs are
    // available before any network work.
    let space_ev = sign_event(
        build_space_create_event(&signing_key, &args.name, None, args.auth_tier, &home_node_id, None, false),
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
    let mut state = load_or_default_state(ctx.data_dir, &identity_id, &home_node_url);
    state.spaces.push(xgen_common::state::KnownSpace {
        space_id: space_id.clone(),
        name: args.name.clone(),
        node_endpoint: home_node_url.clone(),
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

/// The stable client-state label + create-if-absent key for the user's `self`
/// thread (M11, D-021). One source of truth shared by `create_dm_space` (which
/// applies it when invitee == creator) and `self_open` (which scans for it).
pub(crate) const SELF_THREAD_LABEL: &str = "self";

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

    // The client's own dial-endpoint record stays a transport URL.
    let home_node_url = ctx
        .node_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| ctx.session.home_node.clone());

    // M10.4 (MP-F13, Shape B): connect first to capture the AuthOk node_id echo,
    // then write the Node's pubkey node_id (not the transport URL) into the DM
    // Space's signed content["home_node"]. Refuse rather than write a URL.
    ctx.session.ensure_connected(ctx.node_override).await?;
    let home_node_id = ctx.session.node_id.clone().ok_or_else(|| {
        anyhow::anyhow!(
            "home Node did not advertise its node_id (older protocol?); refusing to \
             create a DM Space with a transport URL as home_node"
        )
    })?
    // See create_space: `Option<NodeXgid>` projected to `String` here so the
    // function body stays String-typed (D-137, one projection per direction).
    .as_str()
    .to_string();

    // 1) Root: state.dm_space_create — its event_id IS the space_id.
    let dm_ev = sign_event(
        build_dm_space_create_event(&signing_key, &args.invitee, &home_node_id),
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
    let mut state = load_or_default_state(ctx.data_dir, &identity_id, &home_node_url);
    // M11 (D-021): a self-DM (invitee == creator) is labelled "self" so the raw
    // `--invitee <own-id>` floor and the `self` verb converge on one stable label
    // (M11-C4). The label is the offline create-if-absent key for `ops::self_open`.
    let space_name = if args.invitee == identity_id {
        SELF_THREAD_LABEL.to_string()
    } else {
        format!("DM with {}", args.invitee)
    };
    state.spaces.push(xgen_common::state::KnownSpace {
        space_id: space_id.clone(),
        name: space_name,
        node_endpoint: home_node_url.clone(),
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

// ── self (M11) ──────────────────────────────────────────────────────────────

/// Result of `ops::self_open` (M11, D-021). The user's `self` thread Space + its
/// dm Room, and whether this call created it (vs opened an existing one).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfThreadResult {
    pub space_id: SpaceXgid,
    pub room_id: RoomXgid,
    pub created: bool,
}

/// Open the user's `self` thread (M11, D-021), creating it if absent.
///
/// `self` is a single-member personal thread — a DM to the user's own identity
/// (shape B), reusing the existing keypair (not a second account). It
/// auto-resolves the session identity as the sole party (no typed id, M11-D5)
/// and is idempotent: an existing `self` thread (the owned KnownSpace labelled
/// [`SELF_THREAD_LABEL`]) is returned offline, with no network round-trip;
/// otherwise the DM create chain runs (invitee = self) and `create_dm_space`
/// records it with the `"self"` label.
///
/// Reach (M11-D2): the thread is **Node-resident, not device-local** — reachable
/// from any client authenticated as the user (their own devices), which see it by
/// syncing the user's member-Spaces from the home Node. Never federated
/// (`DmFederationNotAllowed`).
pub async fn self_open(ctx: &mut OpContext<'_>) -> Result<SelfThreadResult> {
    // Auto-resolve the session identity = the self party (no typed id, M11-D5).
    let identity_id = {
        let id = ctx.session.identity.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "identity not loaded; dispatcher must call SessionState::ensure_identity first"
            )
        })?;
        id.identity_id.as_str().to_string()
    };
    let home_node_url = ctx
        .node_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| ctx.session.home_node.clone());

    // Create-if-absent (M11-D5): the self thread is the owned KnownSpace labelled
    // "self". Found → open it offline (no connection). The raw floor and this verb
    // share the label, so a thread made via `create-dm-space --invitee <own-id>`
    // is recognised here too.
    let state = load_or_default_state(ctx.data_dir, &identity_id, &home_node_url);
    if let Some(known) = state
        .spaces
        .iter()
        .find(|s| s.role == "owner" && s.name == SELF_THREAD_LABEL)
    {
        let room_id = known.rooms.first().map(|r| r.room_id.clone()).unwrap_or_default();
        return Ok(SelfThreadResult {
            space_id: SpaceXgid::from_xgid(Xgid::new(known.space_id.clone())),
            room_id: RoomXgid::from_xgid(Xgid::new(room_id)),
            created: false,
        });
    }

    // Absent → create the self-DM (invitee = self). `create_dm_space` labels the
    // KnownSpace "self" when invitee == creator (one core, no drift, M11-C2).
    let args = crate::app::CreateDmSpaceArgs { invitee: identity_id };
    let r = create_dm_space(ctx, &args).await?;
    Ok(SelfThreadResult { space_id: r.space_id, room_id: r.room_id, created: true })
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

// ── thread (create / resolve / archive) ──────────────────────────────────────

/// Result of `ops::thread_create`. `thread_id` is the conceptual Thread id
/// (`xgen://thread/sha256:`), derived from the signed create event's id — this is
/// the value `thread resolve` / `thread archive` reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadCreateResult {
    pub event_id: EventXgid,
    pub thread_id: String,
    pub space_id: SpaceXgid,
    pub room_id: RoomXgid,
}

/// Result of `ops::thread_resolve` / `ops::thread_archive`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadStatusResult {
    pub event_id: EventXgid,
    pub thread_id: String,
    pub space_id: SpaceXgid,
    pub room_id: RoomXgid,
}

/// Result of `ops::redact` (M12.4 / V7) — the sent `message.redact`'s id and the
/// content event it targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactResult {
    pub event_id: EventXgid,
    pub target_event_id: EventXgid,
    pub space_id: SpaceXgid,
    pub room_id: RoomXgid,
}

/// Create a Thread in a Room — `thread.create` (thin-verb arc 4, MP-C-13 / PG-08).
/// The Thread id is derived from the signed event id (`thread_id_from_event_id`,
/// matching `apply_thread_create`); it is returned for `resolve`/`archive`.
pub async fn thread_create(
    ctx: &mut OpContext<'_>,
    args: &crate::app::ThreadCreateArgs,
) -> Result<ThreadCreateResult> {
    use xgen_common::event_trace::{trace_event, EventDirection, SessionContext, SpaceRole};
    use xgen_core::space::state::{build_thread_create_event, sign_event, thread_id_from_event_id};

    let signing_key = thread_signing_key(ctx)?;
    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    let (event_id, thread_id) = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
            .await
            .unwrap_or_else(|_| vec![args.space.clone()]);
        let ev = sign_event(
            build_thread_create_event(
                &signing_key,
                &args.space,
                &args.room,
                prev_events,
                args.title.as_deref(),
                args.auth_tier_min,
            ),
            &signing_key,
        );
        let event_id = ev
            .event_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("signed thread.create event missing event_id"))?
            .as_str()
            .to_string();
        let thread_id = thread_id_from_event_id(&event_id);
        let session_ctx = SessionContext {
            identity_id: None,
            role: Some(SpaceRole::Owner),
            space_id: Some(args.space.clone()),
        };
        trace_event(&ev, EventDirection::Out, &session_ctx);
        let outcome = conn.send_event_confirmed(&ev, sync_timeout).await;
        let _ = conn.goodbye("client_disconnect").await;
        apply_single_event_confirm(outcome, "thread-create")?;
        tracing::info!(space_id = %args.space, room_id = %args.room, thread_id = %thread_id, "Thread created");
        (event_id, thread_id)
    };

    Ok(ThreadCreateResult {
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        thread_id,
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        room_id: RoomXgid::from_xgid(Xgid::new(args.room.clone())),
    })
}

/// Mark a Thread `Resolved` — `thread.resolved` (Admin+ ChangeInfo).
pub async fn thread_resolve(
    ctx: &mut OpContext<'_>,
    args: &crate::app::ThreadStatusArgs,
) -> Result<ThreadStatusResult> {
    thread_status_op(ctx, args, true, "thread-resolve").await
}

/// Mark a Thread `Archived` — `thread.archived` (Admin+ ChangeInfo).
pub async fn thread_archive(
    ctx: &mut OpContext<'_>,
    args: &crate::app::ThreadStatusArgs,
) -> Result<ThreadStatusResult> {
    thread_status_op(ctx, args, false, "thread-archive").await
}

/// Shared body for `thread resolve` / `thread archive` (they differ only in the
/// event type, via the builder choice). `resolved == false` ⇒ archive.
async fn thread_status_op(
    ctx: &mut OpContext<'_>,
    args: &crate::app::ThreadStatusArgs,
    resolved: bool,
    verb: &str,
) -> Result<ThreadStatusResult> {
    use xgen_common::event_trace::{trace_event, EventDirection, SessionContext, SpaceRole};
    use xgen_core::space::state::{
        build_thread_archived_event, build_thread_resolved_event, sign_event,
    };

    let signing_key = thread_signing_key(ctx)?;
    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    let event_id = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
            .await
            .unwrap_or_else(|_| vec![args.space.clone()]);
        let unsigned = if resolved {
            build_thread_resolved_event(&signing_key, &args.space, &args.room, &args.thread, prev_events)
        } else {
            build_thread_archived_event(&signing_key, &args.space, &args.room, &args.thread, prev_events)
        };
        let ev = sign_event(unsigned, &signing_key);
        let session_ctx = SessionContext {
            identity_id: None,
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
        apply_single_event_confirm(outcome, verb)?;
        tracing::info!(space_id = %args.space, thread_id = %args.thread, resolved, "Thread status updated");
        id_for_result
    };

    Ok(ThreadStatusResult {
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        thread_id: args.thread.clone(),
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        room_id: RoomXgid::from_xgid(Xgid::new(args.room.clone())),
    })
}

/// Send a `message.redact` targeting `args.target` (M12.4 / V7). Mirrors
/// `thread_status_op`'s single-event shape: tip-anchor → build → sign → confirm.
/// The node-side erasure side-effect (delete the target attachment's blob bytes,
/// subject to the original author's `Retention` — M12.4-D2/D3/D5) fires when the
/// node ingests this event; this op only sends it. The signer is the loaded
/// identity (the redactor); the `SendMessages` / moderation permission gate is
/// the existing one (unchanged).
pub async fn redact(ctx: &mut OpContext<'_>, args: &crate::app::RedactArgs) -> Result<RedactResult> {
    use xgen_common::event_trace::{trace_event, EventDirection, SessionContext, SpaceRole};
    use xgen_core::message::exchange::build_message_redact_event;
    use xgen_core::space::state::sign_event;

    let signing_key = thread_signing_key(ctx)?;
    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    let event_id = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
            .await
            .unwrap_or_else(|_| vec![args.space.clone()]);
        let unsigned =
            build_message_redact_event(&signing_key, &args.space, &args.room, prev_events, &args.target);
        let ev = sign_event(unsigned, &signing_key);
        let session_ctx = SessionContext {
            identity_id: None,
            role: Some(SpaceRole::Member),
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
        apply_single_event_confirm(outcome, "redact")?;
        tracing::info!(space_id = %args.space, target = %args.target, "redact sent");
        id_for_result
    };

    Ok(RedactResult {
        event_id: EventXgid::from_xgid(Xgid::new(event_id)),
        target_event_id: EventXgid::from_xgid(Xgid::new(args.target.clone())),
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        room_id: RoomXgid::from_xgid(Xgid::new(args.room.clone())),
    })
}

/// Pull the loaded identity's signing key (the three `thread` ops need only the
/// key — the builders derive the sender from it).
fn thread_signing_key(ctx: &OpContext<'_>) -> Result<ed25519_dalek::SigningKey> {
    let id = ctx.session.identity.as_ref().ok_or_else(|| {
        anyhow::anyhow!("identity not loaded; dispatcher must call SessionState::ensure_identity first")
    })?;
    Ok(id.signing_key.clone())
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
    // MP-F7-D3 — captured before the `conn` borrow so the rejoin-anchor fallback
    // can read this client's persisted last local event for the Space.
    let data_dir = ctx.data_dir;
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
                    // MP-F7-D3/D4 — a just-left rejoiner is a non-member, so
                    // member-gated sync starves `get_dag_tips` and we land here.
                    // Anchor after this client's own last local event for the
                    // Space (its leave) when known → the rejoin causally descends
                    // from the leave (linear j→lv→rj), not concurrent with it.
                    // Absent (true first join / fresh state) → the create root,
                    // exactly as before. Best-effort: never an error.
                    _ => rejoin_anchor_or_root(
                        &load_or_default_state(data_dir, &identity_id, ""),
                        &args.space,
                    ),
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
        // MP-F7-D2/D4 — persist this leave as the Space's last local event so a
        // later rejoin (`ops::join`) anchors after it (linear j→lv→rj) instead of
        // the create root. Best-effort bookkeeping: a write failure degrades the
        // rejoin to the root fallback, it does not fail the leave.
        let mut state = load_or_default_state(ctx.data_dir, &identity_id, "");
        state
            .last_local_events
            .insert(args.space.clone(), id_for_result.clone());
        state.updated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        if let Err(e) = crate::app::write_client_state(ctx.data_dir, &state) {
            tracing::warn!(
                space_id = %args.space, error = %e,
                "MP-F7: failed to persist leave anchor (rejoin falls back to root)"
            );
        }
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

/// M12.2a (D3/VC) — pure send-argument validation. A send must carry at least
/// one of `--text` / `--attach`; combining them is an ERROR (lock change,
/// D-065 — silently dropping the user's typed text is quiet data-loss). Pure
/// (no I/O) so it is unit-testable without a node or keypair.
pub(crate) fn validate_send_args(args: &crate::app::SendArgs) -> Result<()> {
    if args.attach.is_empty() && args.text.is_none() {
        anyhow::bail!("a send must carry --text or --attach");
    }
    if !args.attach.is_empty() && args.text.is_some() {
        anyhow::bail!("cannot combine --text and --attach yet");
    }
    Ok(())
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
        crypto::{encoding, hashing::hash_uri},
        encryption::blob::encrypt_blob,
        message::exchange::{build_message_file_event, build_message_text_event, Descriptor},
        space::state::sign_event,
        wire::types::DEFAULT_MAX_BLOB_BYTES,
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

    // M12.2a (D3/VC) — require-one + combined guards, fail-fast before connecting.
    validate_send_args(args)?;

    let event_id = {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;
        // Tip discovery via the canonical (single-source) implementation
        // closes the F-003/F-004 class architecturally: there is nowhere
        // else this can be re-implemented now.
        let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
            .await
            .unwrap_or_else(|_| vec![args.space.clone()]);
        let msg_ev = if !args.attach.is_empty() {
            // M12.2a (D3/VD) — multi-file message.file send. For each file: read
            // → encrypt under a fresh per-blob key (M12-D5) → upload the
            // ciphertext to the home node's content-blind blob store over WS
            // (M12-D1/R-2) → a Descriptor. All descriptors ride one message.file
            // event (the builder is already plural). Per R-1 the Descriptor
            // (incl. the per-blob key) is plaintext content, matching
            // message.text today. (Combined --text guarded out above, VC.)
            let mut descriptors: Vec<Descriptor> = Vec::with_capacity(args.attach.len());
            for path in &args.attach {
                let plaintext = std::fs::read(path)
                    .with_context(|| format!("failed to read --attach file {path}"))?;
                let (blob_key, ciphertext) = encrypt_blob(&plaintext);
                // M12.2a (D4/S-3) — client F6 pre-check (UX courtesy): reject
                // before uploading. Uses the shared default ceiling — the node's
                // BlobUploadBegin gate is the authoritative boundary (it knows
                // the operator's live ceiling; this is conservative).
                if ciphertext.len() as u64 > DEFAULT_MAX_BLOB_BYTES {
                    anyhow::bail!(
                        "attachment {path} is too large ({} bytes ciphertext exceeds the \
                         {DEFAULT_MAX_BLOB_BYTES}-byte ceiling)",
                        ciphertext.len()
                    );
                }
                let blob_ref = hash_uri(&ciphertext);
                let plaintext_hash = hash_uri(&plaintext);
                let confirmed = conn
                    .upload_blob(&blob_ref, &ciphertext, sync_timeout)
                    .await
                    .context("blob upload failed")?;
                if confirmed != blob_ref {
                    anyhow::bail!("node confirmed a different blob_ref than uploaded");
                }
                let filename = std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("attachment")
                    .to_string();
                descriptors.push(Descriptor {
                    blob_ref,
                    plaintext_hash,
                    key: encoding::encode(&blob_key),
                    filename,
                    // mime detection is a future polish item (S-1 — not in the D3 lock).
                    mime: "application/octet-stream".to_string(),
                    size: plaintext.len() as u64,
                });
            }
            sign_event(
                build_message_file_event(
                    &signing_key,
                    &args.space,
                    &args.room,
                    prev_events,
                    &descriptors,
                ),
                &signing_key,
            )
        } else {
            // Text-only path — the require-one guard above guarantees text is Some.
            let text = args.text.as_deref().unwrap_or_default();
            sign_event(
                build_message_text_event(
                    &signing_key,
                    &args.space,
                    &args.room,
                    prev_events,
                    text,
                ),
                &signing_key,
            )
        };
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

// ── fetch_attachments (M12.1) ───────────────────────────────────────────────────

/// M12.2a (D2/VA) — clap-derived so it backs the `fetch` `ClientCommand`
/// directly (one struct, no parallel `FetchArgs` — VA). `--out-dir` is required
/// (VA: no surprise writes). Room-level selector (`--space`/`--room`), the
/// built op's grain.
#[derive(Debug, Clone, clap::Args)]
pub struct FetchAttachmentsArgs {
    /// Space ID
    #[arg(long)]
    pub space: String,
    /// Room ID
    #[arg(long)]
    pub room: String,
    /// Directory to write decrypted attachment files into.
    #[arg(long)]
    pub out_dir: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedAttachment {
    pub filename: String,
    pub blob_ref: String,
    pub size: u64,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchAttachmentsResult {
    pub space_id: SpaceXgid,
    pub room_id: RoomXgid,
    pub files: Vec<FetchedAttachment>,
}

/// M12.1 — fetch every attachment in a Room: sync the `message.file` events,
/// extract each `Descriptor`, fetch the ciphertext by `blob_ref` over WS, decrypt
/// under the per-blob key, verify `plaintext_hash` (the client-side W3 integrity
/// check), and write the plaintext to `out_dir/<filename>`. **Errors** on a
/// `plaintext_hash` mismatch (integrity reject) rather than writing bad bytes.
/// The self-thread witness's read side; a CLI verb is M12.2 surface polish.
pub async fn fetch_attachments(
    ctx: &mut OpContext<'_>,
    args: &FetchAttachmentsArgs,
) -> Result<FetchAttachmentsResult> {
    use xgen_core::{
        crypto::{encoding, hashing::hash_uri},
        encryption::blob::decrypt_blob,
        message::exchange::Descriptor,
        transport::connection::Inbound,
        wire::types::{EventType, TransportMessage},
    };

    let sync_timeout = sync_completion_timeout(ctx.data_dir);
    std::fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("failed to create out_dir {:?}", args.out_dir))?;

    let mut files: Vec<FetchedAttachment> = Vec::new();
    {
        let conn = ctx.session.ensure_connected(ctx.node_override).await?;

        // 1) Sync the room; collect message.file descriptors (paginated like
        // history), tracking each file event's id, and collect message.redact
        // targets for the D6/V8 client tombstone.
        let mut file_events: Vec<(String, Vec<Descriptor>)> = Vec::new();
        let mut redacted: std::collections::HashSet<String> = std::collections::HashSet::new();
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
            let continue_from = loop {
                match tokio::time::timeout_at(deadline, conn.recv()).await {
                    Ok(Ok(Inbound::Event(ev))) => {
                        if ev.space_id.as_str() == args.space.as_str()
                            && ev.room_id.as_str() == args.room.as_str()
                        {
                            match ev.event_type {
                                EventType::MessageFile => {
                                    let evid = ev
                                        .event_id
                                        .as_ref()
                                        .map(|e| e.as_str().to_string())
                                        .unwrap_or_default();
                                    let mut descs = Vec::new();
                                    if let Some(atts) =
                                        ev.content.get("attachments").and_then(|v| v.as_array())
                                    {
                                        for a in atts {
                                            if let Ok(d) =
                                                serde_json::from_value::<Descriptor>(a.clone())
                                            {
                                                descs.push(d);
                                            }
                                        }
                                    }
                                    if !descs.is_empty() {
                                        file_events.push((evid, descs));
                                    }
                                }
                                // M12.4-D6/V8 — minimal client tombstone: a
                                // redacted message.file is not rendered / its blob
                                // not fetched. Collect the redact targets; filter
                                // after the full sync (order-independent).
                                EventType::MessageRedact => {
                                    if let Some(t) = ev
                                        .content
                                        .get("target_event_id")
                                        .and_then(|v| v.as_str())
                                    {
                                        redacted.insert(t.to_string());
                                    }
                                }
                                _ => {}
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
                    Err(_) => anyhow::bail!(
                        "sync_request safety-net timeout — peer never sent sync_complete"
                    ),
                }
            };
            match continue_from {
                Some(cursor) => since = cursor,
                None => break 'pages,
            }
        }

        // M12.4-D6/V8 — drop redacted message.file events: a redacted attachment
        // is not fetched (its blob bytes are erased node-side, M12.4-D4). Flatten
        // the surviving file events' descriptors.
        let descriptors: Vec<Descriptor> = file_events
            .into_iter()
            .filter(|(evid, _)| !redacted.contains(evid))
            .flat_map(|(_, descs)| descs)
            .collect();

        // 2) Fetch + decrypt + verify + write each attachment over the same conn.
        // M12.3-D1/D4 (P1/P3) — pass the Space so the node can lazily fetch a
        // missing blob across homes (federated read), and a 2× outer timeout so
        // the node's inner federated round-trip (bounded by
        // [sync].completion_timeout_seconds) serves the bytes or the typed 10003
        // before this outer timeout fires. A self/single-home Space → local-only.
        let fetch_timeout = sync_timeout * 2;
        for d in &descriptors {
            let ciphertext = conn
                .fetch_blob(&d.blob_ref, Some(args.space.as_str()), fetch_timeout)
                .await
                .with_context(|| format!("fetch_blob failed for {}", d.blob_ref))?;
            let key_bytes = encoding::decode(&d.key)
                .map_err(|_| anyhow::anyhow!("malformed per-blob key in descriptor"))?;
            let key: [u8; 32] = key_bytes
                .as_slice()
                .try_into()
                .map_err(|_| anyhow::anyhow!("per-blob key is not 32 bytes"))?;
            let plaintext = decrypt_blob(&key, &ciphertext)
                .map_err(|e| anyhow::anyhow!("blob decrypt failed: {e}"))?;
            // W3 (client-side): post-decrypt integrity.
            if hash_uri(&plaintext) != d.plaintext_hash {
                anyhow::bail!(
                    "attachment {} failed plaintext_hash integrity check",
                    d.filename
                );
            }
            let path = args.out_dir.join(&d.filename);
            std::fs::write(&path, &plaintext)
                .with_context(|| format!("failed to write {:?}", path))?;
            files.push(FetchedAttachment {
                filename: d.filename.clone(),
                blob_ref: d.blob_ref.clone(),
                size: plaintext.len() as u64,
                path: path.to_string_lossy().to_string(),
            });
        }
        let _ = conn.goodbye("client_disconnect").await;
    }

    Ok(FetchAttachmentsResult {
        space_id: SpaceXgid::from_xgid(Xgid::new(args.space.clone())),
        room_id: RoomXgid::from_xgid(Xgid::new(args.room.clone())),
        files,
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

// ── address-book fill: F1 ∪ F2 (M-RP-ADDRESS-BOOK Leg D, Step 3) ──────────────────

/// Whether an event is a MESSAGE (spoken content) event, whose author qualifies
/// for F1 (author-on-sight).
///
/// 🔒 **Joe ruling B (2026-07-25): F1 is MESSAGE authors, not "any drained
/// sender".** Every member authors their own `membership.join` (and the owner
/// `state.space_create`), and those render as system notices — so counting all
/// senders would put every member in F1, making F2 redundant and contradicting
/// the lock's own rationale (§4: *"a member list built on F1 alone shows only
/// talkers"* / *"everyone silent stays an XGID"*). F1 = people who have
/// **spoken**; membership and state events do NOT qualify.
fn is_message_event(t: &xgen_core::wire::types::EventType) -> bool {
    use xgen_core::wire::types::EventType;
    matches!(
        t,
        EventType::MessageText
            | EventType::MessageFile
            | EventType::MessageReaction
            | EventType::MessageRedact
    )
}

/// The full set of identities OBSERVED in a Space's drained DAG: F1 (distinct
/// authors of MESSAGE events, Joe ruling B) ∪ F2 (projected Space members).
/// Deterministic (`BTreeSet`) order — unit-testable without a live Node (the
/// `members_projection` split).
///
/// ⚠️ **The union is the point of the lock (§4).** F1 alone misses silent
/// members (bob — joined, never spoke); F2 alone misses authors who have left
/// the Space. Neither is a superset of the other.
fn observed_identities(
    space: &str,
    events: &[xgen_core::wire::types::Event],
) -> Result<Vec<IdentityXgid>> {
    use std::collections::BTreeSet;

    let mut ids: BTreeSet<IdentityXgid> = BTreeSet::new();

    // F1 — authors of MESSAGE events (Joe ruling B). `Event.sender` is already
    // an `IdentityXgid` (`wire.rs`), so this is a typed clone, no downgrade.
    for e in events {
        if is_message_event(&e.event_type) {
            ids.insert(e.sender.clone());
        }
    }

    // F2 — current members (causal-replay projection over the same drain).
    // Errors propagate; the caller logs and the background fill retries.
    // `MemberEntry.identity_id` is already an `IdentityXgid`, so again a clone.
    let projected = members_projection(space, events)?;
    for m in &projected.members {
        ids.insert(m.identity_id.clone());
    }

    Ok(ids.into_iter().collect())
}

/// Split the observed set into `(to_fetch, to_touch)`: identities the book does
/// NOT hold — the ones to `identity_get` — versus those it already holds, which
/// are *touched* ([`AddressBook::touch`]) rather than re-fetched.
///
/// "Already held" is membership in the book: a re-fetch is a no-op today
/// (`identity.record` always carries `update_version 0`, and `display_name`/
/// `is_ai` are immutable node-side, runbook §2), so *held* == *held fresh*, and
/// re-observing a held identity only needs its `last_seen` advanced. A freshness
/// window (re-fetch after N) becomes meaningful only once `M13 Client Identity
/// Lookup Widening` makes a re-fetch informative, and lands with it — not
/// reserved here. Pure — unit-testable without a live Node.
fn partition_observed(
    observed: Vec<IdentityXgid>,
    book: &crate::address_book::AddressBook,
) -> (Vec<IdentityXgid>, Vec<IdentityXgid>) {
    // `partition` routes the `true` arm — the unheld — into the first bucket.
    // `book.contains` takes `&str` (D-137 §5: accessors stay `&str`), so the
    // typed id reaches through via `as_str`.
    observed.into_iter().partition(|id| !book.contains(id.as_str()))
}

/// Report from one [`fill_from_space`] pass, for observability + testing.
///
/// `Serialize` was added for M-RP-MEMBERS Leg A: `fill_space_records`
/// (`desktop.rs`) returns this across the Tauri boundary to the webview. Purely
/// additive — no field or behaviour change.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct FillReport {
    /// Observed identities the book did not already hold — the ones fetched.
    pub candidates: usize,
    /// `identity.record` returned and absorbed into the book.
    pub fetched: usize,
    /// `identity.not_found` returned — skipped, book left unpoisoned.
    pub not_found: usize,
    /// The identities that returned `identity.not_found` in THIS fill — the
    /// list behind the `not_found` count. The panel needs the ids, not the
    /// tally: hiding an erased member while dimming an unresolved one is a
    /// per-row decision (`M-RP-IDENTITY-RESOLUTION` §4/§5), and a count
    /// cannot drive it.
    ///
    /// Under `D-127` a `not_found` reply means **erased**, not revoked — a
    /// revoked identity returns its record with `revoked` set. Combined with
    /// validation step 11 (`exchange.rs:208-210`), an id in this list names a
    /// member whose events the node now REJECTS.
    pub not_found_ids: Vec<IdentityXgid>,
    /// Already-held identities re-observed in this drain — their `last_seen`
    /// advanced, no re-fetch (the observation contract, J-584).
    pub touched: usize,
}

/// Absorb one fetch result into the book: `Some` ⇒ upsert (stamping
/// `last_seen`); `None` (`identity.not_found`) ⇒ skip, book unchanged. Returns
/// `true` iff a record was absorbed.
///
/// Pure over the book, so the NotFound-skip is unit-testable without a live
/// Node. The `Some` arm uses the version-aware [`merge`](crate::address_book::AddressBook::merge)
/// (§5 V2): a wire-fetched record (always `update_version 0`) never displaces a
/// seeded higher version. In the live fill this is a plain insert — held
/// identities are never re-fetched — but routing through `merge` keeps the
/// wire-vs-seed precedence correct for Option C and for M13.
fn absorb_fetch(
    book: &mut crate::address_book::AddressBook,
    fetched: Option<FetchedIdentity>,
    now: &str,
) -> bool {
    match fetched {
        Some(f) => {
            book.merge(crate::address_book::SeenRecord::from_fetched(&f, now));
            true
        }
        None => false, // NotFound — do NOT poison the book with a placeholder.
    }
}

/// Fill the address book from a Space's live DAG: drain once, compute F1 ∪ F2
/// minus held, fetch the remainder, upsert each stamping `last_seen`
/// (runbook §4 Step 3). Returns a [`FillReport`]; the caller persists the book.
///
/// ⚠️ **MUST be run OFF THE CRITICAL PATH (Joe-locked).** The Space open must
/// return immediately; the consumer (M-RP-MEMBERS) spawns this behind the open
/// and the view updates as records land. Never gate a Space open on this loop —
/// at N members it is an unbounded network wait in front of a UI action.
///
/// # Re-entrancy invariant — DO NOT "tidy away" the `conn` clears
///
/// 🔑 **`fill_from_space` MUST NOT return with a non-`None` `ctx.session.conn`
/// on ANY path — success, early return, or error.** It is the first `ops::*`
/// verb designed to be called **repeatedly on a live session**, so it must
/// leave the session re-usable.
///
/// **Systemic root (filed for `session.rs`, NOT fixed here):** every one of the
/// ~25 `goodbye` sites in `ops.rs` closes the WebSocket but leaves
/// `session.conn = Some(dead)`, and `ensure_connected` reuses a `Some`
/// connection without detecting that it is closed. This has never bitten
/// because M5/M6 dispatchers are one-shot (a fresh session per command); the
/// blanket fix belongs in `ensure_connected` (detect + reconnect a dead conn),
/// touches every op, and is its own arc — M7's persistent `--aicontrol` session
/// stands on the same mine. Until then, this verb self-cleans: the wrapper
/// below clears `conn` on **every** exit (proven live at J-586 — Pass 2 failed
/// on exactly the exits the mid-clear alone did not cover: the warm early
/// return, and the post-goodbye exit).
pub async fn fill_from_space(
    ctx: &mut OpContext<'_>,
    book: &mut crate::address_book::AddressBook,
    space: &str,
) -> Result<FillReport> {
    let result = fill_from_space_inner(ctx, book, space).await;
    // Exit invariant (see above): never hand the session back with a stale
    // connection, on any path — success, early return, drain error, or fetch
    // error. This single clear is why the verb is re-entrant.
    ctx.session.conn = None;
    result
}

async fn fill_from_space_inner(
    ctx: &mut OpContext<'_>,
    book: &mut crate::address_book::AddressBook,
    space: &str,
) -> Result<FillReport> {
    // Pass 1 — drain the Space DAG ONCE. `drain_space_events` opens, drains,
    // and `goodbye`s (closing the socket), so from here `session.conn` is
    // `Some(dead)`; the wrapper clears it on exit and `fill_from_events` resets
    // it for its own fetch loop.
    let events = drain_space_events(ctx, space).await?;
    fill_from_events(ctx, book, space, &events).await
}

/// Everything a fill does AFTER the drain: compute the observed set (F1 ∪ F2),
/// split held vs unheld, touch the held, fetch the unheld on one reused
/// connection. Operates over already-drained `events` and opens NO drain of its
/// own — so a caller that has already drained the Space DAG for another purpose
/// (`fill_and_members`, which also needs the membership projection off the same
/// events) can fill WITHOUT a second drain of the same DAG (Leg A-bis §1).
///
/// ⚠️ **Resets `ctx.session.conn` for its fetch loop and `goodbye`s at the end,
/// so on return `session.conn` is `Some(dead)` again.** It does NOT clear on
/// exit — the re-entrancy invariant belongs to the PUBLIC wrappers
/// (`fill_from_space`, `fill_and_members`), which clear on every path. Keeping
/// exactly one clear per public entry point is what keeps it auditable.
async fn fill_from_events(
    ctx: &mut OpContext<'_>,
    book: &mut crate::address_book::AddressBook,
    space: &str,
    events: &[xgen_core::wire::types::Event],
) -> Result<FillReport> {
    let observed = observed_identities(space, events)?;
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    // Split observed into unknown (fetch) and already-held (touch).
    let (to_fetch, to_touch) = partition_observed(observed, book);

    // The observation contract (J-584): re-observing a held identity advances
    // its `last_seen` — no fetch (a re-fetch is a no-op under the wire ceiling)
    // — and it keeps E3 from evicting someone who is demonstrably still present.
    for id in &to_touch {
        book.touch(id.as_str(), &now);
    }

    let mut report = FillReport {
        candidates: to_fetch.len(),
        fetched: 0,
        not_found: 0,
        not_found_ids: Vec::new(),
        touched: to_touch.len(),
    };
    if to_fetch.is_empty() {
        // Warm-book steady state — the common path. The wrapper clears the
        // drain's dead connection on return (this used to leak it, J-586).
        return Ok(report);
    }

    // The drain closed its WebSocket; force a fresh connection for the fetch
    // loop (`conn` is a public field; `ensure_connected` only reconnects when
    // it is `None`).
    ctx.session.conn = None;

    // Pass 2 — fetch the unknowns on ONE reused connection, goodbye once.
    let conn = ctx.session.ensure_connected(ctx.node_override).await?;
    for id in &to_fetch {
        let fetched = identity_get_on(conn, id.as_str()).await?;
        if absorb_fetch(book, fetched, &now) {
            report.fetched += 1;
        } else {
            report.not_found += 1;
            report.not_found_ids.push(id.clone());
        }
    }
    let _ = conn.goodbye("client_disconnect").await;

    Ok(report)
}

/// The outcome of [`fill_and_members`]: the [`FillReport`] from the address-book
/// fill and the Space's resolved [`MembersResult`] roster, from a single drain.
///
/// M-RP-MEMBERS Leg A-quater replaced the prior positional
/// `(FillReport, MembersResult)` tuple with this named struct so that the
/// R7 members widget binds to `.fill` / `.roster` rather than to `.0` / `.1`.
/// Purely a return-shape change — the two halves are unchanged. No
/// `#[serde(rename_all)]`, so the wire keys are exactly `fill` and `roster`.
#[derive(Debug, Clone, Serialize)]
pub struct FillMembersOutcome {
    pub fill: FillReport,
    pub roster: MembersResult,
}

/// Fill the address book AND return the Space's resolved membership from a
/// SINGLE drain (M-RP-MEMBERS Leg A-bis). The R7 members widget needs both the
/// roster (to render `state.members`) and the fill (to resolve those members'
/// names from the address book). `members` drains, and `fill_from_space`
/// drains; calling both would drain the same Space DAG twice, back to back —
/// roughly doubling the cold-start window during which state ③ ("waiting for
/// the others") is on screen (Phase-0 §4c-i). This drains ONCE and derives
/// both from the same events.
///
/// The membership projection is derived directly from the drained events
/// (`members_projection`). `fill_from_events`'s own `observed_identities`
/// re-derives that projection internally, but that is a pure in-memory
/// re-derivation — NOT a second drain. The network cost is paid exactly once,
/// which is the whole point of this verb.
///
/// # Re-entrancy invariant — the same discipline as `fill_from_space`
///
/// 🔑 **Clears `ctx.session.conn = None` on EVERY exit** — success, projection
/// error, or fetch error — so the next caller on this live session gets a
/// re-usable connection. The systemic root (every `goodbye` leaves a
/// `Some(dead)` conn, and `ensure_connected` reuses a `Some` blindly) is filed
/// for `session.rs`, NOT fixed here — see `fill_from_space` and D-129.
pub async fn fill_and_members(
    ctx: &mut OpContext<'_>,
    book: &mut crate::address_book::AddressBook,
    space: &str,
) -> Result<FillMembersOutcome> {
    let result = fill_and_members_inner(ctx, book, space).await;
    // Exit invariant (see above): never hand the session back with a stale
    // connection, on any path — success, projection error, drain error, or
    // fetch error. This single clear is why the verb is re-entrant.
    ctx.session.conn = None;
    result
}

async fn fill_and_members_inner(
    ctx: &mut OpContext<'_>,
    book: &mut crate::address_book::AddressBook,
    space: &str,
) -> Result<FillMembersOutcome> {
    // ONE drain feeds BOTH the roster and the fill (Leg A-bis §1).
    let events = drain_space_events(ctx, space).await?;
    let members = members_projection(space, &events)?;
    let report = fill_from_events(ctx, book, space, &events).await?;
    Ok(FillMembersOutcome {
        fill: report,
        roster: members,
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

    // ── MP-F7 — leave→rejoin anchor (client side) ─────────────────────────────
    #[test]
    fn mp_f7_rejoin_anchor_uses_persisted_leave_when_present() {
        let mut state = ClientState::default();
        state
            .last_local_events
            .insert("xgen://hash/sha256:s".into(), "xgen://hash/sha256:lv".into());
        // Present ⇒ anchor after the leave (the rejoin causally descends from it).
        assert_eq!(
            rejoin_anchor_or_root(&state, "xgen://hash/sha256:s"),
            vec!["xgen://hash/sha256:lv".to_string()]
        );
    }

    #[test]
    fn mp_f7_rejoin_anchor_falls_back_to_root_when_absent() {
        // Absent (first join / fresh / cleared state) ⇒ the create root, as today.
        let state = ClientState::default();
        assert_eq!(
            rejoin_anchor_or_root(&state, "xgen://hash/sha256:s"),
            vec!["xgen://hash/sha256:s".to_string()]
        );
    }

    #[test]
    fn mp_f7_client_state_loads_without_last_local_events_field() {
        // D-1 backward-compat (prime invariant): a pre-MP-F7 state.json with no
        // `last_local_events` deserialises (serde default → empty map), and the
        // absent-anchor path still degrades to root.
        let json = r#"{"identity_id":"xgen://pubkey/ed25519:a","display_name":"a",
            "version":"0.10.3","build":"x","home_node":"ws://127.0.0.1:8080/xgen",
            "updated_at":"2026-05-17T00:00:00.000Z","spaces":[]}"#;
        let state: ClientState = serde_json::from_str(json).expect("old state loads");
        assert!(state.last_local_events.is_empty());
        assert_eq!(
            rejoin_anchor_or_root(&state, "xgen://hash/sha256:s"),
            vec!["xgen://hash/sha256:s".to_string()]
        );
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
            last_local_events: Default::default(),
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
            last_local_events: Default::default(),
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
            last_local_events: Default::default(),
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
        assert_eq!(r.space_id.as_str(), "xgen://hash/sha256:abc");
        assert_eq!(r.space_name, "Test Space");
        assert_eq!(r.rooms.len(), 2);
        assert_eq!(r.rooms[0].name, "general");
        assert_eq!(r.rooms[1].name, "random");
    }

    #[test]
    fn redact_result_round_trips_byte_identically() {
        // M-RP-XGID-SLOT-RETYPE Leg C V3-b. `RedactResult`'s identifier slots
        // are typed XGID flavours; under `#[serde(transparent)]` the CLI stdout
        // / pipe-JSON shape is byte-unchanged (bare strings) and a round-trip is
        // stable. `RedactResult` derives no `PartialEq`, so the round-trip is
        // asserted on the serialised text.
        let original = RedactResult {
            event_id: EventXgid::from_xgid(Xgid::new("xgen://hash/sha256:evt".to_string())),
            target_event_id: EventXgid::from_xgid(Xgid::new("xgen://hash/sha256:tgt".to_string())),
            space_id: SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:spc".to_string())),
            room_id: RoomXgid::from_xgid(Xgid::new("xgen://hash/sha256:rm".to_string())),
        };
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(
            json,
            r#"{"event_id":"xgen://hash/sha256:evt","target_event_id":"xgen://hash/sha256:tgt","space_id":"xgen://hash/sha256:spc","room_id":"xgen://hash/sha256:rm"}"#,
        );
        let back: RedactResult = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
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
            last_local_events: Default::default(),
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

    // ── identity_get response parsing (M-RP-ADDRESS-BOOK Leg D, Step 1) ────────
    //
    // The network shell (`ensure_connected` → `send_identity` → `recv` →
    // `goodbye`) shares `register`'s already-exercised transport path and needs
    // a live Node; the testable substance is `parse_identity_get_response`,
    // covered here over hand-built `Inbound` values (the `members_projection`
    // split). Record ⇒ Some · NotFound ⇒ None · unexpected ⇒ error · a
    // wire-omitted `is_ai` ⇒ false.

    use xgen_core::transport::connection::Inbound;
    use xgen_core::wire::types::IdentityMessage;

    #[test]
    fn identity_get_record_maps_to_some_preserving_display_name_and_is_ai() {
        let inbound = Inbound::Identity(IdentityMessage::Record {
            protocol_version: "0.1".into(),
            identity_id: "xgen://pubkey/ed25519:CAROL".into(),
            display_name: Some("Carol".into()),
            registered_at: "2026-07-25T00:00:00Z".into(),
            devices: vec![],
            home_node: "xgen://pubkey/ed25519:NODE".into(),
            is_ai: true,
            ai_capabilities: None,
        });
        let got = parse_identity_get_response(inbound)
            .unwrap()
            .expect("identity.record must map to Some");
        assert_eq!(got.identity_id.as_str(), "xgen://pubkey/ed25519:CAROL");
        assert_eq!(got.display_name.as_deref(), Some("Carol"));
        assert!(got.is_ai, "is_ai must be preserved from the wire");
    }

    #[test]
    fn identity_get_not_found_maps_to_none_not_an_error() {
        // NotFound is a normal outcome (an identity the node has never seen):
        // Ok(None), NOT an error (runbook §4 Step 1).
        let inbound = Inbound::Identity(IdentityMessage::NotFound {
            protocol_version: "0.1".into(),
            identity_id: "xgen://pubkey/ed25519:GHOST".into(),
        });
        let got = parse_identity_get_response(inbound)
            .expect("NotFound must NOT be an error");
        assert!(got.is_none(), "NotFound must map to None");
    }

    #[test]
    fn identity_get_unexpected_inbound_is_an_error() {
        // Any non-identity-lookup response is a protocol violation for this op.
        let inbound = Inbound::Identity(IdentityMessage::RegisterOk {
            protocol_version: "0.1".into(),
            identity_id: "xgen://pubkey/ed25519:X".into(),
            registered_at: "2026-07-25T00:00:00Z".into(),
        });
        assert!(
            parse_identity_get_response(inbound).is_err(),
            "an unexpected inbound must bail, not silently succeed"
        );
    }

    #[test]
    fn identity_get_is_ai_defaults_false_when_wire_omits_it() {
        // A human record: the wire omits `is_ai` entirely
        // (skip_serializing_if = is_false, types.rs:468). Deserialising such a
        // record must default it to false, and the parser must preserve that —
        // otherwise every human would read as an AI.
        let json = r#"{"type":"identity.record","protocol_version":"0.1","identity_id":"xgen://pubkey/ed25519:BOB","registered_at":"2026-07-25T00:00:00Z","devices":[],"home_node":"xgen://pubkey/ed25519:NODE"}"#;
        let rec: IdentityMessage =
            serde_json::from_str(json).expect("human record deserialises with is_ai defaulted");
        let got = parse_identity_get_response(Inbound::Identity(rec))
            .unwrap()
            .expect("Record ⇒ Some");
        assert!(!got.is_ai, "is_ai must default to false when the wire omits it");
        assert_eq!(got.display_name, None, "absent display_name stays None");
    }
}

#[cfg(test)]
mod pass_4_commit_1_tests {
    //! XGID Retrofit Pass 4 Commit 1 — Surface #1 (M5 Ops Layer) per-surface
    //! tests T1 + T2 (runbook §3.4). T3 lives at xgen-common flavours.rs.
    //! Plus the Vec-level sibling of T2, added for
    //! `M-RP-IDENTITY-RESOLUTION` Leg B (Phase-0 §9, owed at J-647).
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

    /// T2-vec — the Vec-level sibling of the wire-invariance witness.
    /// `FillReport.not_found_ids` is `Vec<IdentityXgid>`, and each element is a
    /// `#[serde(transparent)]` flavour wrapper, so the field must serialise as a
    /// plain JSON array of STRINGS, never an array of objects. The TS mirror
    /// declares `not_found_ids: string[]` (`address-book.svelte.ts:59`) and that
    /// claim had NO witness before this test: T2 covers SCALAR identifier slots
    /// only, so citing it for a `Vec` would be a claim narrower than its subject
    /// (filed J-647, Phase-0 §9). This test can genuinely fail — adding a serde
    /// attribute to the field or to `IdentityXgid` breaks it.
    #[test]
    fn fill_report_not_found_ids_vec_serde_transparent_wire_invariance() {
        let r = FillReport {
            candidates: 2,
            fetched: 0,
            not_found: 2,
            not_found_ids: vec![
                ix("xgen://pubkey/ed25519:AAA"),
                ix("xgen://pubkey/ed25519:BBB"),
            ],
            touched: 0,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(
            json.contains(
                r#""not_found_ids":["xgen://pubkey/ed25519:AAA","xgen://pubkey/ed25519:BBB"]"#
            ),
            "not_found_ids is not a plain array of strings: {json}"
        );
        // An EMPTY vec must still serialise as `[]`, never omitted — the mirror
        // declares the field required and the panel reads it unconditionally.
        let empty = FillReport::default();
        let json_empty = serde_json::to_string(&empty).unwrap();
        assert!(
            json_empty.contains(r#""not_found_ids":[]"#),
            "empty not_found_ids must serialise as []: {json_empty}"
        );
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

    // ── address-book fill: F1 ∪ F2 (M-RP-ADDRESS-BOOK Leg D, Step 3) ───────────
    //
    // Pure candidate computation (`fill_candidates`) + the fetch-absorb step
    // (`absorb_fetch`) are unit-tested here over hand-built event sequences —
    // the `members_projection` split (network shell thin, substance pure). The
    // async orchestrator `fill_from_space` shares `members`/`identity_get`'s
    // already-exercised transport path and is exercised live at Step 6.
    //
    // F1 = MESSAGE authors ONLY (Joe ruling B, 2026-07-25). The union is the
    // point of the lock: F1 alone misses silent members (bob), F2 alone misses
    // authors who left (carol) — neither is a superset of the other.

    fn seed_space(owner: &ed25519_dalek::SigningKey) -> (String, Event) {
        let create = sign_event(
            build_space_create_event(owner, "Team", None, 1, TEST_HOME, None, false),
            owner,
        );
        let space_id = create.event_id.clone().unwrap().as_str().to_string();
        (space_id, create)
    }

    /// Invite + join `who` into `space` (space-level), tip-chained onto `tip`.
    /// Returns (invite, join, join_id) so a follow-on event can chain on.
    fn invite_and_join(
        owner: &ed25519_dalek::SigningKey,
        who: &ed25519_dalek::SigningKey,
        space_id: &str,
        tip: &str,
    ) -> (Event, Event, String) {
        let who_id = identity_id_from_key(who);
        let mut invite_u = build_membership_event(
            owner,
            space_id,
            "",
            EventType::MembershipInvite,
            serde_json::json!({ "target_identity": who_id, "role": "member" }),
        );
        invite_u.prev_events = vec![ex(tip)];
        let invite = sign_event(invite_u, owner);
        let invite_id = invite.event_id.clone().unwrap().as_str().to_string();
        let mut join_u =
            build_membership_event(who, space_id, "", EventType::MembershipJoin, serde_json::json!({}));
        join_u.prev_events = vec![ex(&invite_id)];
        let join = sign_event(join_u, who);
        let join_id = join.event_id.clone().unwrap().as_str().to_string();
        (invite, join, join_id)
    }

    #[test]
    fn observed_f2_catches_a_silent_member() {
        // bob joins and never speaks. His only authored event is his join
        // (membership, not a message) ⇒ he can enter ONLY via F2.
        let alice = keypair::generate();
        let bob = keypair::generate();
        let bob_id = identity_id_from_key(&bob);

        let (space_id, create) = seed_space(&alice);
        let (invite, join, _) = invite_and_join(&alice, &bob, &space_id, &space_id);
        let events = vec![create, invite, join];
        assert!(
            !events.iter().any(|e| super::is_message_event(&e.event_type)),
            "guard: this corpus has no message events, so F1 contributes nobody"
        );

        let observed = observed_identities(&space_id, &events).unwrap();
        assert!(observed.contains(&ix(&bob_id)), "silent member bob must enter via F2");
    }

    #[test]
    fn observed_f1_catches_an_author_who_left() {
        // carol joins, SPEAKS, then leaves. F2 no longer lists her (left), but
        // F1 catches her because she posted a message — the case the union
        // exists for, and the one F2 alone would miss.
        use xgen_core::message::exchange::build_message_text_event;
        let alice = keypair::generate();
        let carol = keypair::generate();
        let carol_id = identity_id_from_key(&carol);

        let (space_id, create) = seed_space(&alice);
        let (invite, join, join_id) = invite_and_join(&alice, &carol, &space_id, &space_id);
        // carol speaks — the F1 trigger. room_id "" is fine: the membership
        // projection ignores message events and F1 reads only type + sender.
        let msg = sign_event(
            build_message_text_event(&carol, &space_id, "", vec![join_id.clone()], "carol-msg"),
            &carol,
        );
        let msg_id = msg.event_id.clone().unwrap().as_str().to_string();
        let mut leave_u =
            build_membership_event(&carol, &space_id, "", EventType::MembershipLeave, serde_json::json!({}));
        leave_u.prev_events = vec![ex(&msg_id)];
        let leave = sign_event(leave_u, &carol);
        let events = vec![create, invite, join, msg, leave];

        // F2 has dropped carol…
        let projected = members_projection(&space_id, &events).unwrap();
        assert!(
            !projected.members.iter().any(|m| m.identity_id.as_str() == carol_id),
            "carol left — she must NOT be in the F2 projection"
        );
        // …but F1 catches her.
        let observed = observed_identities(&space_id, &events).unwrap();
        assert!(observed.contains(&ix(&carol_id)), "author-who-left carol must enter via F1");
    }

    #[test]
    fn observed_member_and_author_appears_once() {
        // alice is the owner (F2) AND posts a message (F1). She must appear
        // exactly once — the union is a set, not a concatenation.
        use xgen_core::message::exchange::build_message_text_event;
        let alice = keypair::generate();
        let alice_id = identity_id_from_key(&alice);

        let (space_id, create) = seed_space(&alice);
        let create_id = create.event_id.clone().unwrap().as_str().to_string();
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, "", vec![create_id], "alice-msg"),
            &alice,
        );
        let events = vec![create, msg];

        let observed = observed_identities(&space_id, &events).unwrap();
        assert_eq!(
            observed.iter().filter(|c| c.as_str() == alice_id.as_str()).count(),
            1,
            "an identity that is both an author and a member appears once, not twice"
        );
    }

    #[test]
    fn partition_sends_held_to_touch_and_unheld_to_fetch() {
        // bob is a member, but the book already holds him ⇒ he is NOT re-fetched
        // (a re-fetch is a no-op under the wire ceiling; runbook §2) — he is
        // touched instead (observation contract, J-584).
        let alice = keypair::generate();
        let bob = keypair::generate();
        let bob_id = identity_id_from_key(&bob);

        let (space_id, create) = seed_space(&alice);
        let (invite, join, _) = invite_and_join(&alice, &bob, &space_id, &space_id);
        let events = vec![create, invite, join];

        let mut book = crate::address_book::AddressBook::new();
        book.insert(crate::address_book::SeenRecord::from_fetched(
            &FetchedIdentity {
                identity_id: ix(&bob_id),
                display_name: Some("Bob".into()),
                registered_at: "2026-07-25T00:00:00Z".into(),
                devices: vec![],
                home_node: nx(TEST_HOME),
                is_ai: false,
                ai_capabilities: None,
            },
            "2026-07-25T00:00:00Z",
        ));

        let observed = observed_identities(&space_id, &events).unwrap();
        let (to_fetch, to_touch) = partition_observed(observed, &book);
        assert!(!to_fetch.contains(&ix(&bob_id)), "an already-held identity is not re-fetched");
        assert!(to_touch.contains(&ix(&bob_id)), "an already-held identity is touched instead");
    }

    #[test]
    fn absorb_fetch_notfound_is_skipped_without_poisoning_the_book() {
        // identity.not_found ⇒ Ok(None) ⇒ nothing entered. The book must NOT
        // gain a placeholder for an identity the node has never seen.
        let mut book = crate::address_book::AddressBook::new();
        let absorbed = absorb_fetch(&mut book, None, "2026-07-25T12:00:00Z");
        assert!(!absorbed, "NotFound must not be absorbed");
        assert!(book.is_empty(), "NotFound must not poison the book");
    }

    #[test]
    fn absorb_fetch_some_upserts_stamping_last_seen() {
        let mut book = crate::address_book::AddressBook::new();
        let fetched = FetchedIdentity {
            identity_id: ix("xgen://pubkey/ed25519:DAN"),
            display_name: Some("Dan".into()),
            registered_at: "2026-07-25T00:00:00Z".into(),
            devices: vec![],
            home_node: nx(TEST_HOME),
            is_ai: false,
            ai_capabilities: None,
        };
        let absorbed = absorb_fetch(&mut book, Some(fetched), "2026-07-25T12:00:00Z");
        assert!(absorbed, "a Record must be absorbed");
        let rec = book.get("xgen://pubkey/ed25519:DAN").expect("record present");
        assert_eq!(rec.display_name.as_deref(), Some("Dan"));
        assert_eq!(rec.last_seen, "2026-07-25T12:00:00Z", "last_seen is stamped on absorb");
    }

    // ── Step 6: the corpus assembled, five NOW-tier cases asserted (Option A) ──
    //
    // Committed deterministic assembly of the seed corpus
    // (docs/tests/scripts/ADDRESS_BOOK_SEED_CORPUS.md §4) over the Step-3 event
    // builders. The FETCH is SIMULATED against an in-test node registry — Option
    // A (Joe, 2026-07-25): a mock node would only verify the mock; the LIVE
    // drain→get→absorb round-trip is a separate J-582-style operational pass
    // (Chat's seat), and the corpus→node load was already proven live at J-582.
    //
    // Live-derivable cases (alice F1, bob F2, erin is_ai) come through the fill;
    // dave (revoked) and frank (not-renewed) are ⚠️ SEEDED per §2 Option C —
    // the wire carries neither `revoked` nor `trust_assertion`, so their state
    // is overlaid book-internally, exactly as `M13` would later deliver it.
    #[test]
    fn corpus_assembles_the_five_now_tier_cases_and_round_trips() {
        use crate::address_book::{AddressBook, SeenRecord};
        use std::collections::HashMap;
        use tempfile::tempdir;
        use xgen_core::message::exchange::build_message_text_event;

        let alice = keypair::generate();
        let bob = keypair::generate();
        let erin = keypair::generate();
        let carol = keypair::generate();
        let dave = keypair::generate();
        let frank = keypair::generate();
        let idof = |k: &ed25519_dalek::SigningKey| identity_id_from_key(k);
        let (alice_id, bob_id, erin_id, carol_id, dave_id, frank_id) = (
            idof(&alice), idof(&bob), idof(&erin), idof(&carol), idof(&dave), idof(&frank),
        );

        // ── Build the corpus DAG as a single causal chain. Messages use
        // room_id "" — the membership projection ignores message events and F1
        // reads only type + sender (see Step-3 F1 test).
        let (space_id, create) = seed_space(&alice);
        let mut events = vec![create];

        // bob — silent member (F2 only): invite + join, NO message.
        let (inv, join, bob_jid) = invite_and_join(&alice, &bob, &space_id, &space_id);
        events.extend([inv, join]);

        // erin — AI, speaks (F1 + F2): invite + join + message.
        let (inv, join, erin_jid) = invite_and_join(&alice, &erin, &space_id, &bob_jid);
        events.extend([inv, join]);
        let erin_msg = sign_event(
            build_message_text_event(&erin, &space_id, "", vec![erin_jid], "erin-msg-1"),
            &erin,
        );
        let mut tip = erin_msg.event_id.clone().unwrap().as_str().to_string();
        events.push(erin_msg);

        // alice — owner (F2) AND speaks (F1).
        let alice_msg = sign_event(
            build_message_text_event(&alice, &space_id, "", vec![tip.clone()], "alice-msg-1"),
            &alice,
        );
        tip = alice_msg.event_id.clone().unwrap().as_str().to_string();
        events.push(alice_msg);

        // carol, dave, frank — each joins + speaks (F1 + F2).
        for who in [&carol, &dave, &frank] {
            let (inv, join, jid) = invite_and_join(&alice, who, &space_id, &tip);
            events.extend([inv, join]);
            let msg = sign_event(
                build_message_text_event(who, &space_id, "", vec![jid], "msg-1"),
                who,
            );
            tip = msg.event_id.clone().unwrap().as_str().to_string();
            events.push(msg);
        }

        // ── The observed set is the full F1 ∪ F2 union.
        let mut book = AddressBook::new();
        let candidates = observed_identities(&space_id, &events).unwrap();
        for want in [&alice_id, &bob_id, &erin_id, &carol_id, &dave_id, &frank_id] {
            assert!(candidates.contains(&ix(want)), "every corpus identity is observed: {want}");
        }
        assert!(candidates.contains(&ix(&bob_id)), "bob (silent) enters via F2, not F1");

        // ── Simulate the node registry (Option A) and run the fetch/absorb loop.
        let mk = |ident: &str, name: &str, is_ai: bool| FetchedIdentity {
            identity_id: ix(ident),
            display_name: Some(name.to_string()),
            registered_at: "2026-07-25T00:00:00Z".to_string(),
            devices: vec![],
            home_node: nx(TEST_HOME),
            is_ai,
            ai_capabilities: None,
        };
        let mut registry: HashMap<String, FetchedIdentity> = HashMap::new();
        registry.insert(alice_id.clone(), mk(&alice_id, "alice", false));
        registry.insert(bob_id.clone(), mk(&bob_id, "bob", false));
        registry.insert(erin_id.clone(), mk(&erin_id, "erin", true)); // AI
        registry.insert(carol_id.clone(), mk(&carol_id, "carol", false));
        registry.insert(dave_id.clone(), mk(&dave_id, "dave", false));
        registry.insert(frank_id.clone(), mk(&frank_id, "frank", false));

        let now = "2026-07-25T13:00:00Z";
        for cand in &candidates {
            absorb_fetch(&mut book, registry.get(cand.as_str()).cloned(), now);
        }

        // ── NOW-tier assertions (live-derivable).
        assert!(book.get(&alice_id).is_some(), "alice present via F1 (authored) + F2 (owner)");
        assert!(book.get(&bob_id).is_some(), "bob present via F2 (silent member)");
        assert!(book.get(&erin_id).unwrap().is_ai, "erin present with is_ai = true");

        // ── Option-C seeds (§2): the wire carries neither field, so dave's
        // revocation and frank's lapsed assertion are overlaid book-internally.
        let mut dave_seed = SeenRecord::from_fetched(registry.get(&dave_id).unwrap(), now);
        dave_seed.revoked = true; // SEEDED — revocation-on-encounter (§2 Option C)
        book.insert(dave_seed);
        let mut frank_seed = SeenRecord::from_fetched(registry.get(&frank_id).unwrap(), now);
        frank_seed.trust_assertion = Some(serde_json::json!({ "valid_until": "2026-01-15T00:00:00Z" })); // SEEDED (§2 Option C)
        book.insert(frank_seed);

        assert!(book.get(&dave_id).unwrap().revoked, "dave revoked (SEEDED, §2 Option C)");
        assert_eq!(
            book.get(&frank_id).unwrap().trust_lapsed(now),
            Some(true),
            "frank's not-renewed badge derivable (SEEDED, §2 Option C)"
        );

        // ── Survives a full save→load cycle with the corpus loaded.
        let dir = tempdir().unwrap();
        book.save(dir.path()).unwrap();
        let reloaded = AddressBook::load(dir.path()).unwrap();
        assert_eq!(reloaded, book, "the book survives save→load with the corpus loaded");

        // ── No identity appears twice.
        assert_eq!(book.len(), 6, "six identities, none duplicated");
        let unique: std::collections::HashSet<_> = book.iter().map(|(k, _)| k.clone()).collect();
        assert_eq!(unique.len(), 6, "all six ids distinct");
    }
}

#[cfg(test)]
mod m12_2a_send_validation_tests {
    //! M12.2a C2 (D3/VC) — the pure require-one + combined send-arg guards.
    use super::validate_send_args;
    use crate::app::SendArgs;

    fn args(text: Option<&str>, attach: Vec<&str>) -> SendArgs {
        SendArgs {
            space: "s".to_string(),
            room: "r".to_string(),
            text: text.map(str::to_string),
            attach: attach.into_iter().map(str::to_string).collect(),
        }
    }

    #[test]
    fn text_only_ok() {
        assert!(validate_send_args(&args(Some("hi"), vec![])).is_ok());
    }

    #[test]
    fn attach_only_ok() {
        assert!(validate_send_args(&args(None, vec!["a.png"])).is_ok());
    }

    #[test]
    fn multi_attach_only_ok() {
        assert!(validate_send_args(&args(None, vec!["a.png", "b.png"])).is_ok());
    }

    #[test]
    fn neither_text_nor_attach_errors() {
        // VC require-one.
        assert!(validate_send_args(&args(None, vec![])).is_err());
    }

    #[test]
    fn combined_text_and_attach_errors() {
        // VC lock change (D-065): combined is an error, not warn-and-ignore.
        assert!(validate_send_args(&args(Some("hi"), vec!["a.png"])).is_err());
    }
}
