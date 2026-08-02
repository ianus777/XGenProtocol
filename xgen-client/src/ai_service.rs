// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! AI Client resident mode (M4, spec 3.6.10). Sibling of `service::run`,
//! dispatched when the CLI receives `--ai-mode --service`.
//!
//! Composition (mirrors `service::run`):
//!   - own tracing subscriber + per-run log file
//!   - PID file write (so `--pid` finds this resident)
//!   - named-pipe server with extended `__HEALTH__` reply identifying mode
//!     and operator-known count (M4 §7)
//!   - best-effort sustained WS to the home Node, with the AI runtime loop
//!     on the inbound side: receive event → apply to local SpaceState →
//!     dispatch to plugin → emit reply under pacing + mute constraints
//!
//! Differs from `service::run` in exactly two places:
//!   1. The `AiBehavior` plugin named in `[ai] plugin = "..."` is loaded.
//!   2. The WS receive loop calls the plugin per inbound Event and emits
//!      replies through the same WS connection, gated on pacing and mute.
//!
//! Per the locked M4 architecture (§4 lifecycle, §6 pacing, §8 operator
//! control plane), this resident:
//!   - does NOT auto-join Spaces (impl decision #4 — manual join)
//!   - drops (does not queue) replies that pacing rejects (§6 — "honest
//!     behaviour over polite behaviour")
//!   - exposes no operator command surface (§8)
//!   - takes no temperature actions (§7)

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{anyhow, Context, Result};
use chrono::{SecondsFormat, Utc};
use tokio::sync::Mutex;

use xgen_common::{
    build_info,
    event_trace::{trace_event, write_session_footer, write_session_header, EventDirection, ExitReason, SessionContext, SpaceRole},
    xgid::{IdentityXgid, SpaceXgid, Xgid},
};
use xgen_core::{
    identity::registration::identity_id_from_key,
    message::exchange::build_message_text_event,
    resolution::{derive::conflicts_in_log, derive_resolved, state_key_for_event},
    space::state::{sign_event, SpaceState},
    transport::client::connect_url,
    wire::types::{Event, EventType, TransportMessage},
};

use crate::{
    ai_behavior::{AiBehavior, EchoPlugin, EventContext},
    app,
    batch::ResidentHealthState,
};

// ── Logging init ──────────────────────────────────────────────────────────────

fn init_logging(data_dir: &std::path::Path, log_level_override: Option<&str>) {
    let log_dir = data_dir.join("logs");
    std::fs::create_dir_all(&log_dir).expect("failed to create logs/");
    let now = chrono::Local::now();
    let log_filename = format!("xgen-client_{}.log", now.format("%Y-%m-%d_%H-%M-%S"));
    let log_path = log_dir.join(&log_filename);
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("failed to open log file");

    use tracing_subscriber::{fmt, EnvFilter};
    // D-068 — flag > env (XGEN_LOG) > config (`[logging].level`) > "debug".
    // Pre-J-079 this path fell back to `EnvFilter::new("debug")`, silently
    // ignoring config. The convergence ships in J-079 commit 3.
    let config_path = data_dir.join("xgen-client_config.toml");
    let config_level = app::read_config_log_level(&config_path);
    let env_filter = EnvFilter::new(xgen_common::precedence::resolve_log_level(
        log_level_override,
        config_level.as_deref(),
    ));
    fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_ansi(false)
        .with_writer(log_file)
        .init();
}

// ── Plugin loader ─────────────────────────────────────────────────────────────

/// Instantiate the plugin named in `[ai] plugin = "..."`. M4 ships exactly
/// one plugin (`"echo"`); unknown names cause the resident to refuse to
/// start. Future milestones add additional impls.
fn load_plugin(plugin_name: &str) -> Result<Box<dyn AiBehavior>> {
    match plugin_name {
        "echo" => Ok(Box::new(EchoPlugin::new())),
        other => Err(anyhow!(
            "unknown AI plugin {:?} — M4 ships only \"echo\"",
            other
        )),
    }
}

// ── Outbound pacing tracker ───────────────────────────────────────────────────

/// Per-Space drop-on-throttle tracker for AI replies (M4 §6 "Pacing").
///
/// Separate from `xgen-client::pacing::PacingManager` because their policies
/// differ: PacingManager queues throttled messages and releases them later;
/// the AI service intentionally drops them (the moment has passed). Mixing
/// the two through PacingManager's `attempt_send` API would leave ghost
/// queue entries that distort subsequent decisions.
///
/// "Honest behaviour over polite behaviour" — recurring XGen principle named
/// at M4 v0.2→v0.3 review (see D-065 / J-077): when a system can either
/// misrepresent its current state (queueing — "I'll deliver this thought
/// eventually") or honestly reflect it (dropping — "I had something to say
/// then; the moment passed"), XGen picks honest.
#[derive(Default)]
struct AiPacingTracker {
    /// Last actual send timestamp per Space, epoch ms.
    last_send_at_ms: HashMap<SpaceXgid, u64>,
}

impl AiPacingTracker {
    fn new() -> Self {
        Self::default()
    }

    /// Decide whether to send right now. On `true`, the manager records the
    /// send. On `false`, it does not — the caller drops the reply.
    fn try_send(&mut self, space_id: &str, ai_pacing_ms: u64, now_ms: u64) -> bool {
        if ai_pacing_ms == 0 {
            // Cap of zero disables pacing for AIs in this Space (spec 3.7.12.1).
            self.last_send_at_ms
                .insert(SpaceXgid::from_xgid(Xgid::new(space_id.to_string())), now_ms);
            return true;
        }
        let last = self.last_send_at_ms.get(space_id).copied().unwrap_or(0);
        // Clock-skew safe (spec 3.7.12 / Ch6 §6.14.6).
        let elapsed = now_ms.saturating_sub(last);
        if last == 0 || elapsed >= ai_pacing_ms {
            self.last_send_at_ms
                .insert(SpaceXgid::from_xgid(Xgid::new(space_id.to_string())), now_ms);
            true
        } else {
            false
        }
    }
}

// ── AI runtime loop ───────────────────────────────────────────────────────────

/// The AI service's inner loop. Connects to the home Node, authenticates,
/// requests Space history, replays events into local SpaceStates, and on
/// each inbound `message.text` invokes the plugin. Replies are emitted on
/// the same WS connection, gated by pacing + mute.
///
/// Returns Ok on a clean exit (Goodbye / Closed). Err on any setup failure
/// (config / keypair / connect / auth) — the pipe server keeps running so
/// operators can `--stop` the process even if the WS never came up.
async fn run_ai_loop(
    data_dir: PathBuf,
    health_state: Arc<Mutex<ResidentHealthState>>,
    node_override: Option<String>,
) -> Result<()> {
    let config_path = data_dir.join("xgen-client_config.toml");
    // M8 C9: honor the `--node` flag (flag > config `[client].node`, per D-068
    // precedence) — previously this passed `None`, so `--ai-mode --service`
    // silently ignored `--node` and always used the config default
    // (`init --ai` writes `ws://127.0.0.1:8080/xgen`). See MULTIPARTY_S8_findings.md.
    let node = app::resolve_node(node_override.as_deref(), &config_path);
    let keypair_path = app::resolve_keypair_path(&config_path);
    let signing_key = app::load_keypair(&keypair_path)?;

    // M4: AI-mode requires [ai] section with `plugin = "..."` in config.
    let ai_section = app::load_ai_section(&config_path).ok_or_else(|| {
        anyhow!(
            "ai-mode requires [ai] section in {}; run `xgen-client init --ai` first",
            config_path.display()
        )
    })?;
    if !ai_section.is_ai {
        anyhow::bail!(
            "ai-mode requires `[ai] is_ai = true` in config; {} has is_ai = false",
            config_path.display()
        );
    }
    let plugin_name = ai_section
        .plugin
        .as_deref()
        .ok_or_else(|| anyhow!("ai-mode requires `[ai] plugin = \"...\"` in config"))?;
    let mut plugin = load_plugin(plugin_name)?;
    let mention_token = ai_section
        .behavior
        .as_ref()
        .and_then(|b| b.mention_token.clone());

    let ai_identity_id = IdentityXgid::from_xgid(Xgid::new(identity_id_from_key(&signing_key)));
    tracing::info!(
        plugin = plugin.name(),
        mention_token = ?mention_token,
        identity_id = %ai_identity_id,
        "ai-service: plugin loaded"
    );

    tracing::info!(home_node = %node, "ai-service: connecting to home Node");
    let connect_result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        connect_url(&node),
    )
    .await;
    let mut conn = match connect_result {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => anyhow::bail!("WS connect failed: {e}"),
        Err(_) => anyhow::bail!("WS connect timed out after 10 s"),
    };

    let auth_id = conn
        .client_authenticate(&signing_key)
        .await
        .context("WS authentication failed")?
        .identity_id;
    tracing::info!(identity_id = %auth_id, "ai-service: authenticated");

    // Request the history of every Space this AI is a member of.
    // F-6 / F-7: limit None → responder applies [sync].batch_size; if the
    // history exceeds one page the dispatch loop below chases continue_from
    // cursors via follow-up sync_requests. The initial-catch-up boundary
    // surfaces as a SyncComplete with continue_from: None.
    let sync_req = TransportMessage::SyncRequest {
        protocol_version: "0.1".to_string(),
        since: String::new(),
        limit: None,
    };
    conn.send_transport(&sync_req)
        .await
        .context("failed to send sync_request")?;

    // Per-Space SpaceState reconstructed from received events. Replies
    // chain to the most-recent event observed in the Space.
    let mut spaces: HashMap<String, SpaceState> = HashMap::new();
    // R2-F01 C2 — the per-Space accumulated event log that backs the node's
    // ingest gate (conflicts_in_log → derive_resolved rebuild; else
    // incremental). Retained for the resident's lifetime (D-065 honest residue:
    // a new allocation bounded by Space activity — the cost of giving the AI
    // loop the node's exact resolution discipline).
    let mut space_logs: HashMap<String, Vec<Event>> = HashMap::new();
    let mut last_event_in_space: HashMap<String, String> = HashMap::new();
    let mut pacing = AiPacingTracker::new();

    let session_ctx = SessionContext {
        // `auth_id` is `AuthOutcome.identity_id`, now `IdentityXgid`; project to
        // SessionContext's `String` form at the consumer (Leg C).
        identity_id: Some(auth_id.as_str().to_string()),
        role: Some(SpaceRole::Member),
        space_id: None,
    };

    use xgen_core::transport::connection::Inbound;
    loop {
        let inbound = match conn.recv().await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(reason = %e, "ai-service: WS recv error — connection lost");
                break;
            }
        };

        match inbound {
            Inbound::Event(event) => {
                if let Some(id) = &event.event_id {
                    trace_event(&event, EventDirection::In, &session_ctx);

                    // Determine which Space this event belongs to.
                    let space_id = if event.space_id.is_empty() {
                        id.as_str().to_string()
                    } else {
                        event.space_id.as_str().to_string()
                    };

                    // R2-F01 C2 — accumulate the per-Space log first (the event
                    // is now part of `range(0)`, exactly as the node appends
                    // before deriving in `NodeRuntime::ingest_event`), then
                    // apply with the node's ingest discipline.
                    space_logs.entry(space_id.clone()).or_default().push(event.clone());

                    let is_create = matches!(
                        event.event_type,
                        EventType::StateSpaceCreate | EventType::StateDmSpaceCreate
                    );
                    if is_create {
                        if matches!(event.event_type, EventType::StateSpaceCreate) {
                            // Rebuild the snapshot from the accumulated log via
                            // `derive_resolved` (mirrors the node's ingest_event
                            // create arm — convergent, and picks up any
                            // out-of-order children already in the log). Empty
                            // `identity_home_nodes` + vantage `""` per F01-D2/D3.
                            if let Some(s) = derive_resolved(
                                space_logs[&space_id].clone(),
                                "",
                                &HashMap::new(),
                            ) {
                                spaces.insert(space_id.clone(), s);
                            }
                        }
                        // state.dm_space_create stays skipped (CP-2a): the AI
                        // loop has no creator key and DM handling is out of C2
                        // scope. AI shouldn't be in someone else's DM Space with
                        // sync access — if it happens, skip rather than crash.
                    } else if let Some(state) = spaces.get_mut(&space_id) {
                        // The node's SR-D1 gate: a state-keyed event that
                        // genuinely conflicts in the accumulated log forces a
                        // full convergent rebuild (SR-D2); messages and
                        // non-conflicting state events take the incremental
                        // fast path, byte-for-byte today's behaviour.
                        apply_or_rebuild(&space_logs[&space_id], state, &event);
                    }

                    // Track the most recent event so replies can chain.
                    last_event_in_space.insert(space_id.clone(), id.as_str().to_string());

                    // Plugin dispatch for message.text events only — all
                    // other event types are state-affecting and the plugin
                    // doesn't react to them. (Other plugins MAY react;
                    // EchoPlugin doesn't.)
                    let plugin_reply = {
                        let ctx = EventContext {
                            event: &event,
                            ai_identity_id: &ai_identity_id,
                            mention_token: mention_token.as_deref(),
                        };
                        plugin.on_event(&ctx)
                    };

                    if let Some(reply_text) = plugin_reply {
                        if let Err(e) = maybe_emit_reply(
                            &mut conn,
                            &signing_key,
                            ai_identity_id.as_str(),
                            &event,
                            &space_id,
                            &reply_text,
                            &spaces,
                            &mut pacing,
                            &mut last_event_in_space,
                            &session_ctx,
                        )
                        .await
                        {
                            tracing::warn!(reason = %e, "ai-service: reply emission failed");
                        }
                    }

                    // Update health state — operator known per Space.
                    refresh_health_operator_counts(
                        &spaces,
                        ai_identity_id.as_str(),
                        &health_state,
                    )
                    .await;
                }
            }
            Inbound::Transport(TransportMessage::SyncComplete {
                continue_from,
                new_tip,
                since,
                ..
            }) => {
                // F-6 / F-7: explicit end-of-batch signal. If continue_from
                // is Some, more catch-up history remains — chase it with a
                // follow-up sync_request keyed on the cursor. If None, the
                // AI is caught up; the loop continues running for live
                // federation events.
                match continue_from {
                    Some(cursor) => {
                        tracing::debug!(
                            cursor = %cursor,
                            since = %since,
                            new_tip = %new_tip,
                            "ai-service: sync_complete with continue_from — paginating"
                        );
                        let follow_up = TransportMessage::SyncRequest {
                            protocol_version: "0.1".to_string(),
                            since: cursor,
                            limit: None,
                        };
                        if let Err(e) = conn.send_transport(&follow_up).await {
                            tracing::warn!(reason = %e, "ai-service: pagination sync_request failed");
                            break;
                        }
                    }
                    None => {
                        tracing::info!(
                            new_tip = %new_tip,
                            "ai-service: initial catch-up complete"
                        );
                    }
                }
            }
            Inbound::Transport(TransportMessage::Goodbye { .. }) | Inbound::Closed => {
                tracing::info!("ai-service: WS closed — exiting loop");
                break;
            }
            _ => {} // ignore other transport messages, pings, etc.
        }
    }

    let _ = conn.goodbye("ai_service_shutdown").await;
    Ok(())
}

/// Emit a `message.text` reply to the Room of `inbound_event`, gated by
/// the AI's `active_mutes` entry (if any) and the per-Space `ai_pacing_ms`
/// cap. Returns Ok on either successful send OR honest drop (mute / pacing).
/// Returns Err only on transport-level failures.
#[allow(clippy::too_many_arguments)]
async fn maybe_emit_reply(
    conn: &mut xgen_core::transport::connection::Connection<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    signing_key: &ed25519_dalek::SigningKey,
    ai_identity_id: &str,
    inbound_event: &Event,
    space_id: &str,
    reply_text: &str,
    spaces: &HashMap<String, SpaceState>,
    pacing: &mut AiPacingTracker,
    last_event_in_space: &mut HashMap<String, String>,
    session_ctx: &SessionContext,
) -> Result<()> {
    // Mute check — if this AI is muted in the Space, drop silently with a
    // log line. (Spec 3.7.13.6: auto_temperature mutes follow the same path.)
    if let Some(state) = spaces.get(space_id) {
        if state.active_mutes.contains_key(ai_identity_id) {
            tracing::info!(
                space_id = %space_id,
                "ai-service: dropping reply — AI is currently muted in this Space"
            );
            return Ok(());
        }
    }

    // Resolve ai_pacing_ms from the local SpaceState (Phase 2 default 2000ms).
    let ai_pacing_ms = spaces
        .get(space_id)
        .map(|s| s.ai_pacing_ms)
        .unwrap_or(xgen_common::wire::DEFAULT_AI_PACING_MS);

    let now_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    if !pacing.try_send(space_id, ai_pacing_ms, now_ms) {
        tracing::warn!(
            space_id = %space_id,
            ai_pacing_ms = ai_pacing_ms,
            "ai-service: dropping reply — pacing cap not yet elapsed (honest behaviour over polite behaviour)"
        );
        return Ok(());
    }

    // Use the most recently observed event in this Space as the
    // prev_events anchor. The runtime tracks this in `last_event_in_space`.
    let prev_events = last_event_in_space
        .get(space_id)
        .map(|id| vec![id.clone()])
        .unwrap_or_else(|| vec![space_id.to_string()]);

    let room_id = if inbound_event.room_id.is_empty() {
        // Inbound was a Space-level event (no Room). Reply needs a Room;
        // skip emission and log.
        tracing::info!(
            space_id = %space_id,
            "ai-service: skipping reply — inbound event had no room_id"
        );
        return Ok(());
    } else {
        inbound_event.room_id.clone()
    };

    let reply_event = sign_event(
        build_message_text_event(signing_key, space_id, room_id.as_str(), prev_events, reply_text),
        signing_key,
    );
    trace_event(&reply_event, EventDirection::Out, session_ctx);
    conn.send_event(&reply_event)
        .await
        .context("failed to send AI reply event")?;
    // Record the reply's event_id as the new last-seen so subsequent replies
    // chain after it (we won't receive it back over our own WS, but other
    // events arriving after it will already extend it via their own prev_events).
    if let Some(reply_id) = reply_event.event_id.clone() {
        last_event_in_space.insert(space_id.to_string(), reply_id.as_str().to_string());
    }
    tracing::info!(
        space_id = %space_id,
        room_id = %room_id,
        event_id = %reply_event.event_id.as_deref().map(|x| x.as_str()).unwrap_or("?"),
        "ai-service: reply sent"
    );
    let _ = ai_identity_id; // already used implicitly via signing_key
    Ok(())
}

/// Recompute the operator-known-count for the resident's health state.
/// Called after every event-applied so `__HEALTH__` reads stay fresh.
async fn refresh_health_operator_counts(
    spaces: &HashMap<String, SpaceState>,
    ai_identity_id: &str,
    health_state: &Arc<Mutex<ResidentHealthState>>,
) {
    let mut known: usize = 0;
    let mut total: usize = 0;
    for (_sid, state) in spaces.iter() {
        if !state.is_member(ai_identity_id) {
            continue;
        }
        total += 1;
        if state.resolve_operator(ai_identity_id).is_some() {
            known += 1;
        }
    }
    let mut hs = health_state.lock().await;
    hs.mode_label = "ai".to_string();
    hs.operator_known = Some((known, total));
}

/// Apply one inbound Event to the running per-Space `SpaceState` with the node's
/// ingest discipline (R2-F01 C2). `log` is the Space's accumulated event log and
/// already includes `ev` (the caller appended it first — the exact shape of
/// `NodeRuntime::ingest_event`, which `append`s before deriving). A state-keyed
/// event that genuinely conflicts in the log forces a full convergent rebuild
/// via `derive_resolved` (SR-D1/SR-D2, ancestry-aware via `conflicts_in_log` —
/// CP-E); messages (no state key) and non-conflicting state events take the
/// incremental `apply_event` fast path, byte-for-byte today's behaviour, so
/// message traffic never pays a log scan. Vantage `""` + an empty
/// `identity_home_nodes` map per F01-D2/D3 (the client is not a Node; CP-3).
/// Pure (no I/O) so it unit-tests without a live Node.
fn apply_or_rebuild(log: &[Event], state: &mut SpaceState, ev: &Event) {
    let conflict = state_key_for_event(ev).is_some() && conflicts_in_log(ev, log);
    if conflict {
        if let Some(s) = derive_resolved(log.to_vec(), "", &HashMap::new()) {
            *state = s;
        }
    } else {
        let _ = state.apply_event(ev, "");
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Headless AI Client resident entry. Owns the tokio runtime; runs until
/// Ctrl+C or `__STOP__` over the pipe.
pub fn run(
    data_dir: PathBuf,
    instance_label: Option<String>,
    log_level_override: Option<String>,
    node_override: Option<String>,
) {
    std::fs::create_dir_all(&data_dir).expect("failed to create data directory");

    init_logging(&data_dir, log_level_override.as_deref());

    let started_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let session_id = format!("{:08x}", rand::random::<u32>());
    write_session_header(
        "client",
        None,
        None,
        None,
        "0.1",
        build_info::VERSION,
        &session_id,
        &started_at,
    );

    app::write_pid_file(&data_dir);

    let pipe_name_str = crate::batch::pipe_name(instance_label.as_deref());

    // Shared health state, populated by the AI loop and read by the pipe
    // server's __HEALTH__ handler. Default-initialised to human-mode shape
    // so the file-level type stays sane even if the AI loop never runs.
    let health_state = Arc::new(Mutex::new(ResidentHealthState::ai_default()));

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime for --ai-mode --service");

    rt.block_on(async move {
        #[cfg(target_os = "windows")]
        let _pipe_shutdown_hold = {
            let (tx, rx) = tokio::sync::watch::channel(false);
            let pipe_data_dir = data_dir.clone();
            let pipe_name = pipe_name_str.clone();
            let pipe_health = health_state.clone();
            let batch_rx = rx.clone();
            tokio::spawn(async move {
                crate::batch::start_pipe_server_with_health(
                    pipe_name,
                    pipe_data_dir,
                    batch_rx,
                    pipe_health,
                )
                .await;
            });
            // --aicontrol sister server (M7 C2) — second independent spawn.
            let ai_dir = data_dir.clone();
            let ai_pipe = crate::aicontrol::aicontrol_pipe_name(&pipe_name_str);
            let state_lock: crate::aicontrol::StateFileLock =
                std::sync::Arc::new(tokio::sync::Mutex::new(()));
            // M7-events C5: the client `.events` observer pipe — clone rx/dir
            // before the aicontrol spawn moves `rx`.
            let events_dir = data_dir.clone();
            let events_pipe = crate::events_pipe::events_pipe_name(&pipe_name_str);
            let events_rx = rx.clone();
            tokio::spawn(async move {
                // expected_token = None: AC-D4 gate inert in v1 (M7C-D1).
                crate::aicontrol::start_aicontrol_server(ai_pipe, ai_dir, rx, state_lock, None).await;
            });
            tokio::spawn(async move {
                crate::events_pipe::start_events_server(events_pipe, events_dir, events_rx).await;
            });
            tx
        };
        #[cfg(not(target_os = "windows"))]
        {
            let _ = pipe_name_str;
            tracing::warn!(
                "ai-service: named pipe server is Windows-only; control flags will fail on this platform"
            );
        }

        let loop_data_dir = data_dir.clone();
        let loop_health = health_state.clone();
        let loop_node = node_override.clone();
        let ai_task = tokio::spawn(async move {
            if let Err(e) = run_ai_loop(loop_data_dir, loop_health, loop_node).await {
                tracing::warn!(reason = %format!("{:#}", e), "ai-service: AI task ended");
            }
        });

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("ai-service: Ctrl+C received — exiting");
            }
            _ = ai_task => {
                tracing::info!("ai-service: AI task ended; pipe server still active — waiting for Ctrl+C or pipe __STOP__");
                let _ = tokio::signal::ctrl_c().await;
                tracing::info!("ai-service: Ctrl+C received — exiting");
            }
        }
    });

    write_session_footer(ExitReason::Shutdown);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pacing_tracker_first_send_passes() {
        let mut p = AiPacingTracker::new();
        assert!(p.try_send("space", 2000, 10_000));
    }

    #[test]
    fn pacing_tracker_second_within_cap_dropped() {
        let mut p = AiPacingTracker::new();
        assert!(p.try_send("space", 2000, 10_000));
        // 500 ms later, still inside the 2000 ms cap → drop.
        assert!(!p.try_send("space", 2000, 10_500));
    }

    #[test]
    fn pacing_tracker_second_after_cap_passes() {
        let mut p = AiPacingTracker::new();
        assert!(p.try_send("space", 2000, 10_000));
        assert!(p.try_send("space", 2000, 12_000));
    }

    #[test]
    fn pacing_tracker_per_space_isolated() {
        let mut p = AiPacingTracker::new();
        assert!(p.try_send("space_a", 2000, 10_000));
        // Space B has its own clock; should pass even though A is throttled.
        assert!(p.try_send("space_b", 2000, 10_500));
    }

    #[test]
    fn pacing_tracker_zero_cap_disables() {
        let mut p = AiPacingTracker::new();
        assert!(p.try_send("space", 0, 10_000));
        assert!(p.try_send("space", 0, 10_001));
        assert!(p.try_send("space", 0, 10_002));
    }

    #[test]
    fn pacing_tracker_clock_skew_safe() {
        let mut p = AiPacingTracker::new();
        assert!(p.try_send("space", 2000, 10_000));
        // Clock goes backward; saturating_sub yields 0 → throttled.
        assert!(!p.try_send("space", 2000, 9_500));
    }

    /// T13 — `AiPacingTracker` per-Space key is `SpaceXgid` (design doc
    /// §4.6.c / Q6.2). The `&str` API resolves the typed key via `Borrow<str>`:
    /// two sends to the same Space XGID within the cap collide on the same
    /// entry (second dropped); a different Space XGID is isolated.
    #[test]
    fn ai_pacing_tracker_per_space_xgid_key() {
        let mut p = AiPacingTracker::new();
        let space = "xgen://hash/sha256:space_a";
        assert!(p.try_send(space, 2000, 10_000), "first send passes");
        assert!(
            !p.try_send(space, 2000, 10_500),
            "same Space XGID within cap → typed key collides, dropped"
        );
        assert!(
            p.try_send("xgen://hash/sha256:space_b", 2000, 10_500),
            "different Space XGID is isolated"
        );
    }

    #[test]
    fn load_plugin_known_name_succeeds() {
        let plugin = load_plugin("echo").unwrap();
        assert_eq!(plugin.name(), "echo");
    }

    #[test]
    fn load_plugin_unknown_name_errors() {
        // Can't .unwrap_err() because Box<dyn AiBehavior> isn't Debug.
        match load_plugin("nonexistent") {
            Ok(_) => panic!("expected error for unknown plugin name"),
            Err(e) => assert!(e.to_string().contains("unknown AI plugin")),
        }
    }

    // ── R2-F01 C2 — apply_or_rebuild gate (pure, no live Node) ────────────────

    use xgen_common::xgid::EventXgid;
    use xgen_core::identity::keypair;
    use xgen_core::space::state::{build_membership_event, build_space_create_event};

    const AI_NODE: &str = "xgen://pubkey/ed25519:NODE";

    fn ex(s: &str) -> EventXgid {
        EventXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn ixid(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }

    /// The C2 proof (F01-D5, Arc-C mirror at the gate level): feeding a
    /// concurrent same-key conflict — `join(bob)` vs `ban(bob)`, both
    /// referencing the create root — through `apply_or_rebuild` in receive
    /// order converges to the byte-identical snapshot a full `derive_resolved`
    /// of the log produces, i.e. the node winner (Layer 1: ban > join). This is
    /// what the pre-M8 receive-order `apply_event` loop could not guarantee.
    #[test]
    fn apply_or_rebuild_concurrent_ban_join_converges_to_node_winner() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let bob_id = identity_id_from_key(&bob);

        let create = sign_event(
            build_space_create_event(&alice, "Team", None, 1, AI_NODE, None, false),
            &alice,
        );
        let space_id = create.event_id.clone().unwrap().as_str().to_string();
        // Concurrent on membership:{space}:bob — both reference the create root.
        let mut join_u = build_membership_event(
            &bob,
            &space_id,
            "",
            EventType::MembershipJoin,
            serde_json::json!({}),
        );
        join_u.prev_events = vec![ex(&space_id)];
        let join = sign_event(join_u, &bob);
        let mut ban_u = build_membership_event(
            &alice,
            &space_id,
            "",
            EventType::MembershipBan,
            serde_json::json!({ "target_identity": bob_id }),
        );
        ban_u.prev_events = vec![ex(&space_id)];
        let ban = sign_event(ban_u, &alice);

        // Simulate the receive loop: seed at create via derive_resolved (the
        // loop's create arm), then accumulate + gate each subsequent event.
        let mut log: Vec<Event> = vec![create.clone()];
        let mut state =
            derive_resolved(log.clone(), "", &HashMap::new()).expect("create seeds state");
        for ev in [join.clone(), ban.clone()] {
            log.push(ev.clone());
            apply_or_rebuild(&log, &mut state, &ev);
        }

        let node_state = derive_resolved(
            vec![create, join, ban],
            "",
            &HashMap::new(),
        )
        .expect("full log resolves");
        assert_eq!(
            state, node_state,
            "the receive-order gate must converge to the node's resolved snapshot"
        );
        assert!(node_state.banned.contains(&ixid(&bob_id)), "ban wins — Bob is banned");
        assert!(
            !state.members.contains_key(&ixid(&bob_id)),
            "Bob must not be a member"
        );
    }

    /// The fast path: a causally-linked `invite → join` (no concurrency) is NOT
    /// a conflict, so the gate takes the incremental `apply_event` at each step
    /// and never rebuilds — yet still equals a full `derive_resolved` of the log
    /// (fast path == replay when there are no conflicts), and Bob joins.
    #[test]
    fn apply_or_rebuild_causal_invite_join_takes_incremental_path() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let bob_id = identity_id_from_key(&bob);

        let create = sign_event(
            build_space_create_event(&alice, "Team", None, 1, AI_NODE, None, false),
            &alice,
        );
        let space_id = create.event_id.clone().unwrap().as_str().to_string();
        let mut invite_u = build_membership_event(
            &alice,
            &space_id,
            "",
            EventType::MembershipInvite,
            serde_json::json!({ "target_identity": bob_id, "role": "member" }),
        );
        invite_u.prev_events = vec![ex(&space_id)];
        let invite = sign_event(invite_u, &alice);
        let invite_id = invite.event_id.clone().unwrap().as_str().to_string();
        let mut join_u = build_membership_event(
            &bob,
            &space_id,
            "",
            EventType::MembershipJoin,
            serde_json::json!({}),
        );
        join_u.prev_events = vec![ex(&invite_id)];
        let join = sign_event(join_u, &bob);

        let mut log: Vec<Event> = vec![create.clone()];
        let mut state =
            derive_resolved(log.clone(), "", &HashMap::new()).expect("create seeds state");
        for ev in [invite.clone(), join.clone()] {
            log.push(ev.clone());
            apply_or_rebuild(&log, &mut state, &ev);
        }

        assert!(
            state.members.contains_key(&ixid(&bob_id)),
            "Bob joined via the incremental fast path"
        );
        assert_eq!(
            state,
            derive_resolved(log, "", &HashMap::new()).expect("log resolves"),
            "no-conflict gate result == full replay"
        );
    }
}
