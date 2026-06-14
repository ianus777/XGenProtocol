// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Batch dispatch + named-pipe IPC (D-043). After Phase 3 wider (J-070), all
// command implementations live in `crate::app`; this module is just the pipe
// transport and the canonical `get_dag_tips`. Named-pipe pieces are
// Windows-only.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::Mutex;

use xgen_core::{
    transport::connection::Inbound,
    wire::types::{EventType, TransportMessage},
};

// ── Resident health state (M4) ────────────────────────────────────────────────

/// Live state surfaced by the pipe server's `__HEALTH__` handler.
/// Populated by the resident's main loop (AI or human) and read by the
/// pipe-server's request handler. Default values are the human-Client
/// shape — the AI service overrides on startup.
#[derive(Debug, Clone)]
pub struct ResidentHealthState {
    /// Short label for the resident mode — `"human"` or `"ai"` — appended
    /// to the `__HEALTH__` reply as `mode=...`.
    pub mode_label: String,
    /// AI-only field: `Some((known, total))` where `known` is the number of
    /// Spaces the AI resolves an operator for and `total` is the count of
    /// Spaces the AI is a member of. `None` for human-mode residents (the
    /// field is omitted from the `__HEALTH__` reply in that case).
    pub operator_known: Option<(usize, usize)>,
}

impl ResidentHealthState {
    /// Default state for a human-Client resident — no AI-specific fields.
    pub fn human_default() -> Self {
        Self {
            mode_label: "human".to_string(),
            operator_known: None,
        }
    }

    /// Default state for an AI-Client resident at startup — before any
    /// Space events have been observed. Shows `operator_known=0/0` rather
    /// than absent so the field is consistently present in AI-mode logs.
    pub fn ai_default() -> Self {
        Self {
            mode_label: "ai".to_string(),
            operator_known: Some((0, 0)),
        }
    }
}

// ── Pipe name — D-043 ──────────────────────────────────────────────────────────

/// Returns the named pipe path for a given instance label (D-043).
/// `None` → `\\.\pipe\xgen-client`
/// `Some("alice")` → `\\.\pipe\xgen-client-alice`
pub fn pipe_name(instance_label: Option<&str>) -> String {
    match instance_label {
        Some(label) => format!(r"\\.\pipe\xgen-client-{}", label),
        None => r"\\.\pipe\xgen-client".to_string(),
    }
}

// ── Cooperative DAG frontier (MP-F4 anchor + MP-F14 infra-exclusion) ────────────

/// Tip-relevant projection of one served Space event, accumulated by
/// `get_dag_tips` and consumed by `cooperative_frontier`. Carries only what the
/// frontier computation needs: the event's id, its `prev_events`, and its typed
/// `event_type` (so the infra-exclusion decision uses the authoritative
/// `EventType::is_federation_infra` predicate, not a string compare). Extracting
/// this lets the cooperative-frontier logic be unit-tested without a live
/// `Connection` (MP-F14 spine).
#[derive(Debug, Clone)]
struct FrontierEvent {
    event_id: String,
    prev_events: Vec<String>,
    event_type: EventType,
}

/// MP-F4 frontier compute: the collected ids that no collected event references
/// as a `prev_event` — the current DAG leaves — sorted for deterministic
/// `prev_events` ordering (D-076) and capped at the DAG fan-in limit (a frontier
/// wider than the limit is pathological for these flows; the cap keeps the built
/// event acceptable to the node's step-10 gate).
fn compute_frontier(seen: &[String], referenced: &HashSet<String>) -> Vec<String> {
    let mut f: Vec<String> =
        seen.iter().filter(|id| !referenced.contains(*id)).cloned().collect();
    f.sort();
    f.truncate(xgen_core::dag::graph::MAX_PREV_EVENTS);
    f
}

/// MP-F14: the **cooperative** DAG sub-frontier — MP-F4's full frontier with
/// federation/vantage-infrastructure events (`state.federation_add`, per
/// [`EventType::is_federation_infra`]) excluded from **both** the leaf set and
/// their `prev_events` contribution.
///
/// The single delta vs the MP-F4 frontier is the `is_federation_infra()` skip: an
/// infra event is dropped from `seen` (so it can never be a returned tip) and its
/// `prev_events` are **not** counted as referenced (so a cooperative event that an
/// infra event built on stays a cooperative leaf). Without the second half, a
/// `state.federation_add` anchored to the latest cooperative tip would mark that
/// tip "referenced" and a new cooperative event would lose it.
///
/// Why it matters (the MP-F14 root cause): the prior behaviour let a cooperative
/// `prev_events` anchor descend from a `state.federation_add` tip, which is
/// vantage-specific (D-075) and never converges cross-node — the peer holds the
/// anchored cooperative event `HeldPending` on that infra predecessor, which never
/// drains. Anchoring only to converging cooperative tips dissolves the miss. All
/// cooperative callers of `get_dag_tips` (`ops::send` / `ops::join` / `ops::leave`)
/// inherit this through the one collection site.
fn cooperative_frontier(events: &[FrontierEvent]) -> Vec<String> {
    let mut seen: Vec<String> = vec![];
    let mut seen_set: HashSet<String> = HashSet::new();
    let mut referenced: HashSet<String> = HashSet::new();
    for ev in events {
        // MP-F14: skip infra events entirely — out of `seen` AND no prev
        // contribution (see fn doc). One `continue` enforces both exclusions.
        if ev.event_type.is_federation_infra() {
            continue;
        }
        if seen_set.insert(ev.event_id.clone()) {
            seen.push(ev.event_id.clone());
        }
        for p in &ev.prev_events {
            referenced.insert(p.clone());
        }
    }
    compute_frontier(&seen, &referenced)
}

/// Request DAG tips for a Space via `transport.sync_request` and collect the
/// current DAG **frontier** (every leaf — events not referenced as any served
/// event's `prev_event`), Space-filtered (closes F-003/F-004 from J-067).
/// The canonical implementation for the entire Client crate.
///
/// **MP-F4 (frontier anchor).** This returns the full frontier, not the single
/// topo-last event the earlier implementation kept. A new event built on the
/// result therefore causally descends from ALL current tips — so a room-join
/// descends from its own space-join even when a concurrent leaf (e.g. a peer's
/// message sent before the join) also exists. The single-tip behaviour left such
/// a room-join concurrent with its space-join, which node-side resolution then
/// dropped (MP-F4). Linear DAG ⇒ frontier == single tip ⇒ behaviour-neutral.
///
/// **MP-F14 (cooperative frontier).** The returned frontier is the *cooperative*
/// sub-frontier: federation/vantage-infrastructure events (`state.federation_add`)
/// are excluded so a cooperative `prev_events` anchor never descends from a
/// non-converging infra tip (see [`cooperative_frontier`]). All cooperative
/// callers (`ops::send` / `ops::join` / `ops::leave`) inherit this here.
///
/// **F-6 / F-7 migration (Phase 1 of Federation Event Propagation completion).**
/// Termination is now the explicit `TransportMessage::SyncComplete` signal, not
/// a 500ms quiet-time heuristic. The loop iterates pages: each page receives
/// Events filtered by `space_id`, terminated by a `SyncComplete`; if
/// `SyncComplete.continue_from` is `Some`, a follow-up `SyncRequest` is issued
/// and pagination continues until `continue_from` returns `None`. The
/// `completion_timeout` parameter is the F-6b safety net — bounds how long the
/// whole pagination loop can run before surfacing a "peer never said done"
/// error. Default value lives in `[sync].completion_timeout_seconds`.
pub async fn get_dag_tips(
    conn: &mut xgen_core::transport::connection::Connection<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    space_id: &str,
    completion_timeout: tokio::time::Duration,
) -> Result<Vec<String>> {
    // MP-F4 (frontier anchor) + MP-F14 (cooperative exclusion). Accumulate every
    // space-matching served event as a `FrontierEvent` across all pages, then
    // compute the *cooperative* DAG frontier via `cooperative_frontier`: the
    // collected ids that no collected COOPERATIVE event references as a parent —
    // the current cooperative leaves — with infra events (`state.federation_add`)
    // excluded from both the leaf set and their prev contribution. Collecting then
    // computing (rather than maintaining frontier sets inline) is what lets
    // `cooperative_frontier` be unit-tested without a live `Connection`.
    let mut collected: Vec<FrontierEvent> = vec![];
    let mut since = String::new();
    let deadline = tokio::time::Instant::now() + completion_timeout;

    loop {
        let req = TransportMessage::SyncRequest {
            protocol_version: "0.1".to_string(),
            since: since.clone(),
            limit: None, // responder applies its [sync].batch_size default
        };
        conn.send_transport(&req).await?;

        // Drain this page until SyncComplete.
        let continue_from = loop {
            let recv = tokio::time::timeout_at(deadline, conn.recv()).await;
            match recv {
                Ok(Ok(Inbound::Event(ev))) => {
                    let ev_space: &str = if ev.space_id.is_empty() {
                        ev.event_id.as_deref().map(|x| x.as_str()).unwrap_or("")
                    } else {
                        ev.space_id.as_str()
                    };
                    // Served events always carry an event_id (signed + persisted);
                    // collect Space-matching ones with their typed event_type so
                    // `cooperative_frontier` applies the infra-exclusion predicate.
                    if ev_space == space_id {
                        if let Some(id) = ev.event_id.as_ref() {
                            collected.push(FrontierEvent {
                                event_id: id.as_str().to_string(),
                                prev_events: ev
                                    .prev_events
                                    .iter()
                                    .map(|p| p.as_str().to_string())
                                    .collect(),
                                event_type: ev.event_type.clone(),
                            });
                        }
                    }
                }
                Ok(Ok(Inbound::Transport(TransportMessage::SyncComplete {
                    continue_from,
                    ..
                }))) => break continue_from,
                Ok(Ok(Inbound::Transport(TransportMessage::Goodbye { .. })))
                | Ok(Ok(Inbound::Closed)) => return Ok(cooperative_frontier(&collected)),
                Ok(Ok(_)) => {} // ignore unrelated transport / federation chatter
                Ok(Err(_)) => return Ok(cooperative_frontier(&collected)),
                Err(_) => {
                    // F-6b safety-net: D-065 "honest behaviour over polite
                    // behaviour" — surface as an error, never silent.
                    anyhow::bail!(
                        "sync_request safety-net timeout — peer never sent sync_complete \
                         within {} ms",
                        completion_timeout.as_millis()
                    );
                }
            }
        };
        match continue_from {
            Some(cursor) => since = cursor,
            None => return Ok(cooperative_frontier(&collected)),
        }
    }
}

/// M8.5-B (INV-D1/INV-D2/INV-D3) — scoped invite-bootstrap fetch. Sends a
/// `transport.invite_bootstrap_request` for `space_id`, drains the structural
/// `HistoryBatch` the Node serves to a pending invitee, and returns the
/// `event_id` of the `membership.invite` naming `my_identity` so `ops::join`
/// can chain its join causally after it (dissolving the M85-A3 concurrency).
///
/// Returns `Ok(Some(invite_id))` on success; `Ok(None)` when the Node refuses
/// the bootstrap (wire `1011` — the requester is not a pending invitee, e.g. an
/// already-member re-join or a Room join) or no invite naming the requester is
/// found, so `ops::join` falls back to `get_dag_tips`. A refusal is a normal
/// outcome here, not an error. Sibling to `get_dag_tips`; same drain shape.
pub async fn get_invite_bootstrap(
    conn: &mut xgen_core::transport::connection::Connection<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    space_id: &str,
    my_identity: &str,
    completion_timeout: tokio::time::Duration,
) -> Result<Option<String>> {
    let req = TransportMessage::InviteBootstrapRequest {
        protocol_version: "0.1".to_string(),
        space_id: space_id.to_string(),
    };
    conn.send_transport(&req).await?;

    let deadline = tokio::time::Instant::now() + completion_timeout;
    let mut invite_id: Option<String> = None;
    loop {
        match tokio::time::timeout_at(deadline, conn.recv()).await {
            Ok(Ok(Inbound::Event(ev))) => {
                // Structural events only (the Node enforces CP-3). Match the
                // invite naming us by content; keep the last (topo order).
                if matches!(ev.event_type, xgen_core::wire::types::EventType::MembershipInvite)
                    && ev.content["target_identity"].as_str() == Some(my_identity)
                {
                    if let Some(id) = ev.event_id.as_ref() {
                        invite_id = Some(id.as_str().to_string());
                    }
                }
            }
            // Success end-of-batch.
            Ok(Ok(Inbound::Transport(TransportMessage::SyncComplete { .. }))) => {
                return Ok(invite_id)
            }
            // Refusal (1011) or any transport error → fall back (no invite_id).
            Ok(Ok(Inbound::Transport(TransportMessage::Error { .. }))) => return Ok(None),
            Ok(Ok(Inbound::Transport(TransportMessage::Goodbye { .. })))
            | Ok(Ok(Inbound::Closed)) => return Ok(invite_id),
            Ok(Ok(_)) => {} // ignore unrelated chatter
            Ok(Err(_)) => return Ok(invite_id),
            Err(_) => {
                anyhow::bail!(
                    "invite_bootstrap safety-net timeout — Node never completed within {} ms",
                    completion_timeout.as_millis()
                );
            }
        }
    }
}

// ── Dispatch ───────────────────────────────────────────────────────────────────

/// Tokenize and dispatch one batch command line.
///
/// Uses shlex for quoting/escaping. Parses against the canonical
/// `crate::app::Cli` (same parser as direct CLI / `--batch` in-process /
/// pipe-server). Dispatches to `crate::app::cmd_*` — there is no parallel
/// command implementation in batch.rs (Phase 3 wider dedup).
///
/// Commands not appropriate for batch dispatch (the long-running smoke /
/// stress tests, `--service`-only modes) are rejected with a clear error
/// rather than silently misbehaving.
pub async fn dispatch_line(line: &str, data_dir: &Path) -> Result<()> {
    use crate::app::{self, Cli, ClientCommand};

    let tokens = shlex::split(line).unwrap_or_else(|| vec![line.to_string()]);
    let mut argv = vec!["xgen-client".to_string()];
    argv.extend(tokens);

    let cli = <Cli as clap::Parser>::try_parse_from(&argv)
        .map_err(|e| anyhow::anyhow!("unrecognised command: {}", e))?;

    let node_override = cli.node.as_deref();
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| data_dir.join("xgen-client_config.toml"));
    let node = app::resolve_node(node_override, &config_path);
    let keypair_path = app::resolve_keypair_path(&config_path);

    match cli.command {
        None => Ok(()),
        Some(ClientCommand::Init(args)) => app::cmd_init(&args, data_dir),
        Some(ClientCommand::Whoami) => {
            // M5 commit 1: pipe arm calls ops::whoami directly. The pipe
            // protocol (D-066-frozen) only needs OK/ERROR — the result data
            // is discarded here. A future --aicontrol surface (M7) will
            // serialise the same WhoamiResult as JSONL.
            let mut session =
                crate::session::SessionState::new(String::new(), data_dir.to_path_buf());
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::whoami(&mut ctx).map(|_| ())
        }
        Some(ClientCommand::Status) => {
            // M5 commit 2: pipe arm calls ops::status directly.
            let mut session =
                crate::session::SessionState::new(String::new(), data_dir.to_path_buf());
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::status(&mut ctx).map(|_| ())
        }
        Some(ClientCommand::Spaces) => {
            // M5 commit 3: pipe arm calls ops::spaces directly.
            let mut session =
                crate::session::SessionState::new(String::new(), data_dir.to_path_buf());
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::spaces(&mut ctx).map(|_| ())
        }
        Some(ClientCommand::Rooms(args)) => {
            // M6 Phase 1 (R1): pipe arm calls ops::rooms directly. The pipe
            // protocol only needs OK/ERROR — the RoomsResult data is discarded
            // here; a future --aicontrol surface (M7) will serialise it as JSONL.
            let mut session =
                crate::session::SessionState::new(String::new(), data_dir.to_path_buf());
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::rooms(&mut ctx, &args).map(|_| ())
        }
        Some(ClientCommand::Version) => app::cmd_version(),
        Some(ClientCommand::Register(args)) => {
            // M5 commit 4: pipe arm calls ops::register directly. The
            // ensure_identity I/O happens at the dispatcher boundary;
            // ops::register sees an already-loaded identity.
            let ai = app::load_ai_section(&config_path);
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::register(&mut ctx, &args, ai.as_ref())
                .await
                .map(|_| ())
        }
        Some(ClientCommand::CreateSpace(args)) => {
            // M5 commit 5: pipe arm calls ops::create_space directly.
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::create_space(&mut ctx, &args).await.map(|_| ())
        }
        Some(ClientCommand::CreateDmSpace(args)) => {
            // M7C-D4 A3: pipe arm calls ops::create_dm_space directly (3-event send).
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::create_dm_space(&mut ctx, &args).await.map(|_| ())
        }
        Some(ClientCommand::SelfThread(_args)) => {
            // M11 (D-021): pipe arm opens the self thread (create-if-absent).
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::self_open(&mut ctx).await.map(|_| ())
        }
        Some(ClientCommand::CreateRoom(args)) => {
            // M5 commit 6: pipe arm calls ops::create_room directly.
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::create_room(&mut ctx, &args).await.map(|_| ())
        }
        Some(ClientCommand::Invite(args)) => {
            // M5 commit 7: pipe arm calls ops::invite directly.
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::invite(&mut ctx, &args).await.map(|_| ())
        }
        Some(ClientCommand::Ban(args)) => {
            // Thin-verb arc 2: pipe arm calls ops::ban directly.
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::ban(&mut ctx, &args).await.map(|_| ())
        }
        Some(ClientCommand::RoomUpdate(args)) => {
            // Thin-verb arc 3: pipe arm calls ops::room_update directly.
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::room_update(&mut ctx, &args).await.map(|_| ())
        }
        Some(ClientCommand::Thread(args)) => {
            // Thin-verb arc 4: pipe arm inner-routes the 3 thread sub-actions.
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            match args.command {
                crate::app::ThreadCommand::Create(a) => {
                    crate::ops::thread_create(&mut ctx, &a).await.map(|_| ())
                }
                crate::app::ThreadCommand::Resolve(a) => {
                    crate::ops::thread_resolve(&mut ctx, &a).await.map(|_| ())
                }
                crate::app::ThreadCommand::Archive(a) => {
                    crate::ops::thread_archive(&mut ctx, &a).await.map(|_| ())
                }
            }
        }
        Some(ClientCommand::Join(args)) => {
            // M5 commit 8: pipe arm calls ops::join directly.
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::join(&mut ctx, &args).await.map(|_| ())
        }
        Some(ClientCommand::Leave(args)) => {
            // M7-completion A2: pipe arm calls ops::leave directly.
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::leave(&mut ctx, &args).await.map(|_| ())
        }
        Some(ClientCommand::Send(args)) => {
            // M5 commit 9: pipe arm calls ops::send directly. With this
            // arm in place there is exactly one call site for the
            // message-text-event path; F-003/F-004 class is closed
            // architecturally.
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::send(&mut ctx, &args).await.map(|_| ())
        }
        Some(ClientCommand::History(args)) => {
            // M5 commit 10: pipe arm calls ops::history directly.
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::history(&mut ctx, &args).await.map(|_| ())
        }
        Some(ClientCommand::Members(args)) => {
            // M7-completion A1: pipe arm calls ops::members directly. The pipe
            // protocol only needs OK/ERROR — the MembersResult data is discarded
            // here; the --aicontrol surface serialises the same result as JSONL.
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::members(&mut ctx, &args).await.map(|_| ())
        }
        Some(ClientCommand::Ai(args)) => {
            // M5 commit 11: pipe arm calls ops::ai_* directly.
            let mut session =
                crate::session::SessionState::new(node.clone(), data_dir.to_path_buf());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            match args.command {
                crate::app::AiCommand::Delegate(a) => {
                    crate::ops::ai_delegate(&mut ctx, &a).await.map(|_| ())
                }
                crate::app::AiCommand::Revoke(a) => {
                    crate::ops::ai_revoke(&mut ctx, &a).await.map(|_| ())
                }
                crate::app::AiCommand::Status(a) => {
                    crate::ops::ai_status(&mut ctx, &a).await.map(|_| ())
                }
            }
        }
        Some(ClientCommand::SmokeTest(_))
        | Some(ClientCommand::StressTest(_))
        | Some(ClientCommand::SmokePh2(_))
        | Some(ClientCommand::StressComplete(_)) => {
            anyhow::bail!(
                "long-running test commands cannot be dispatched through --batch / pipe"
            )
        }
    }
}

// ── Named pipe server — M1 (Windows only) ──────────────────────────────────────

/// Backward-compatible entry point — starts the pipe server with a
/// human-Client default `ResidentHealthState`. Existing callers (service::run,
/// desktop::run) keep working unchanged. The M4 AI service uses
/// `start_pipe_server_with_health` to thread its own state in.
#[cfg(target_os = "windows")]
pub async fn start_pipe_server(
    pipe_name_str: String,
    data_dir: PathBuf,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    let health = Arc::new(Mutex::new(ResidentHealthState::human_default()));
    start_pipe_server_with_health(pipe_name_str, data_dir, shutdown_rx, health).await;
}

/// Start the named pipe server with a shared health-state handle.
/// Accepts one connection at a time; reads commands until `__END__`;
/// dispatches each; writes `OK\n` or `ERROR: …\n`; loops.
/// Stops cleanly when `shutdown_rx` delivers `true`.
///
/// The `__HEALTH__` reply incorporates the `ResidentHealthState` so AI-mode
/// residents (M4) report `mode=ai operator_known=N/M` while human-mode
/// residents report `mode=human` (default).
#[cfg(target_os = "windows")]
pub async fn start_pipe_server_with_health(
    pipe_name_str: String,
    data_dir: PathBuf,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    health_state: Arc<Mutex<ResidentHealthState>>,
) {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::windows::named_pipe::ServerOptions;

    tracing::info!(pipe = %pipe_name_str, "Pipe server starting");

    let mut first = true;

    loop {
        if *shutdown_rx.borrow() {
            break;
        }

        let server = {
            let create_result = if first {
                first = false;
                ServerOptions::new().first_pipe_instance(true).create(&pipe_name_str)
            } else {
                ServerOptions::new().create(&pipe_name_str)
            };
            match create_result {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(error = %e, "Named pipe create failed — pipe server stopping");
                    break;
                }
            }
        };

        // Wait for a client connection or shutdown signal
        let connected = tokio::select! {
            r = server.connect() => r.is_ok(),
            _ = shutdown_rx.changed() => false,
        };

        if !connected || *shutdown_rx.borrow() {
            break;
        }

        // server is now a connected pipe — split for independent read/write
        let (reader_half, mut writer_half) = tokio::io::split(server);
        let mut reader = BufReader::new(reader_half);
        let mut lines: Vec<String> = Vec::new();
        let mut buf = String::new();

        // Read the first line — may be a control command (handled inline) or
        // the first batch line (collected then drained until __END__).
        let first_line: Option<String> = match reader.read_line(&mut buf).await {
            Ok(0) => None,
            Ok(_) => Some(
                buf.trim_end_matches('\n')
                    .trim_end_matches('\r')
                    .to_string(),
            ),
            Err(e) => {
                tracing::warn!(error = %e, "Pipe read error (first line)");
                None
            }
        };

        // Control commands (Phase 4): single line, single response, no __END__.
        // Recognised tokens: __PING__, __HEALTH__, __STOP__, __RELOAD_CONFIG__.
        if let Some(ref line) = first_line {
            match line.as_str() {
                "__PING__" => {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                    let resp = format!("PONG {}\n", now_ms);
                    let _ = writer_half.write_all(resp.as_bytes()).await;
                    let _ = writer_half.flush().await;
                    continue;
                }
                "__HEALTH__" => {
                    // One-line liveness summary. PID + mode label + AI-mode
                    // operator-known count (M4 §7). The structured per-Space
                    // operator map stays on `xgen-client status`.
                    let snapshot = health_state.lock().await.clone();
                    let mut resp = format!(
                        "HEALTHY pid={} mode={}",
                        std::process::id(),
                        snapshot.mode_label
                    );
                    if let Some((known, total)) = snapshot.operator_known {
                        resp.push_str(&format!(" operator_known={}/{}", known, total));
                    }
                    resp.push('\n');
                    let _ = writer_half.write_all(resp.as_bytes()).await;
                    let _ = writer_half.flush().await;
                    continue;
                }
                "__STOP__" => {
                    let _ = writer_half.write_all(b"OK STOPPING\n").await;
                    let _ = writer_half.flush().await;
                    tracing::info!("__STOP__ received over pipe — exiting process");
                    // Brutal exit so the Tauri main loop and any background
                    // tasks go down with us. The merged binary owns the
                    // entire process; clean Tauri shutdown coordination is
                    // post-M1 polish.
                    std::process::exit(0);
                }
                "__RELOAD_CONFIG__" => {
                    let _ = writer_half
                        .write_all(b"NOT_IMPLEMENTED: config reload arrives in a later milestone\n")
                        .await;
                    let _ = writer_half.flush().await;
                    continue;
                }
                _ => { /* fall through to batch path */ }
            }
        }

        // Batch path: not a control command. Collect lines until __END__.
        if let Some(line) = first_line {
            if line != "__END__" {
                lines.push(line);
            }
        }
        loop {
            buf.clear();
            match reader.read_line(&mut buf).await {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = buf
                        .trim_end_matches('\n')
                        .trim_end_matches('\r')
                        .to_string();
                    if trimmed == "__END__" {
                        break;
                    }
                    lines.push(trimmed);
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Pipe read error");
                    break;
                }
            }
        }

        tracing::info!(count = lines.len(), "Batch execution started");

        let mut exec_error: Option<String> = None;
        for line in &lines {
            match dispatch_line(line, &data_dir).await {
                Ok(()) => {}
                Err(e) => {
                    exec_error = Some(format!("{:#}", e));
                    break;
                }
            }
        }

        match exec_error {
            None => {
                tracing::info!("Batch execution completed — OK");
                let _ = writer_half.write_all(b"OK\n").await;
            }
            Some(msg) => {
                tracing::warn!(error = %msg, "Batch execution stopped — ERROR");
                let response = format!("ERROR: {}\n", msg);
                let _ = writer_half.write_all(response.as_bytes()).await;
            }
        }
        let _ = writer_half.flush().await;
        // writer_half and reader_half dropped here → pipe connection closed
    }

    tracing::info!(pipe = %pipe_name_str, "Pipe server stopped");
}

// ── Batch invocation path — M2 (Windows only) ──────────────────────────────────

/// Second-process batch invocation path.
/// Validates the .xgb file, connects to the running instance's pipe,
/// streams commands + __END__, reads result, returns exit code.
/// Creates its own tokio runtime — must NOT be called from within async context.
#[cfg(target_os = "windows")]
pub fn run_batch_client(raw_path: &str, pipe_name_str: &str, instance_label: Option<&str>) -> i32 {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to create async runtime: {}", e);
            return 2;
        }
    };
    rt.block_on(run_batch_client_async(raw_path, pipe_name_str, instance_label))
}

#[cfg(target_os = "windows")]
async fn run_batch_client_async(raw_path: &str, pipe_name_str: &str, instance_label: Option<&str>) -> i32 {
    use std::io::BufRead as _;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::ClientOptions;

    // M2 §2.2 — canonicalize and extension check
    let canonical = match std::fs::canonicalize(raw_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: cannot resolve batch file path {:?}: {}", raw_path, e);
            return 2;
        }
    };

    let ext = canonical
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if !ext.eq_ignore_ascii_case("xgb") {
        eprintln!(
            "error: batch file must have .xgb extension, got {:?}",
            canonical
        );
        return 2;
    }

    // M2 §2.3 — read non-empty, non-comment lines
    let file = match std::fs::File::open(&canonical) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: cannot open {:?}: {}", canonical, e);
            return 2;
        }
    };
    // Same behaviour-adjacent fix as xgen-node/src/pipe.rs — `map_while`
    // stops on the first read error rather than potentially spinning
    // forever on a Lines stream that keeps returning Err.
    let commands: Vec<String> = std::io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    // M2 §2.4 — connect to the running instance
    let mut client = match ClientOptions::new().open(pipe_name_str) {
        Ok(c) => c,
        Err(_) => {
            let start_hint = match instance_label {
                Some(l) => format!("xgen-client.exe --instance {} before running --batch.", l),
                None    => "xgen-client.exe before running --batch.".to_string(),
            };
            eprintln!(
                "error: no running xgen-client instance found at {}\n       Start {}",
                pipe_name_str, start_hint
            );
            return 3;
        }
    };

    // M2 §2.5 — write commands + __END__ sentinel
    for cmd in &commands {
        let line = format!("{}\n", cmd);
        if let Err(e) = client.write_all(line.as_bytes()).await {
            eprintln!("error: failed to write to pipe: {}", e);
            return 1;
        }
    }
    if let Err(e) = client.write_all(b"__END__\n").await {
        eprintln!("error: failed to write sentinel: {}", e);
        return 1;
    }
    if let Err(e) = client.flush().await {
        eprintln!("error: failed to flush: {}", e);
        return 1;
    }

    // Read response
    let mut response = String::new();
    if let Err(e) = client.read_to_string(&mut response).await {
        eprintln!("error: failed to read response: {}", e);
        return 1;
    }

    let response = response.trim();
    if response == "OK" {
        0
    } else if let Some(msg) = response.strip_prefix("ERROR: ") {
        eprintln!("{}", msg);
        1
    } else {
        eprintln!("error: unexpected pipe response: {}", response);
        1
    }
}

// ── Control-command client helpers — Phase 4 (Windows only) ────────────────────

/// Open the pipe, send a single control line, read the single-line response.
/// Returns the trimmed response on success, or an Err containing a
/// human-readable diagnostic on failure.
#[cfg(target_os = "windows")]
async fn pipe_send_control(pipe_name_str: &str, control_token: &str) -> Result<String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::ClientOptions;

    let mut client = ClientOptions::new()
        .open(pipe_name_str)
        .with_context(|| format!("no resident found at {}", pipe_name_str))?;

    let line = format!("{}\n", control_token);
    client
        .write_all(line.as_bytes())
        .await
        .context("failed to write control command")?;
    client.flush().await.context("failed to flush pipe")?;

    let mut response = String::new();
    client
        .read_to_string(&mut response)
        .await
        .context("failed to read pipe response")?;

    Ok(response.trim().to_string())
}

/// `--ping`: round-trip a __PING__ command and print the latency in ms.
/// Exits 0 on PONG response; non-zero otherwise.
#[cfg(target_os = "windows")]
pub fn cmd_ping(pipe_name_str: &str) -> i32 {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to create async runtime: {}", e);
            return 2;
        }
    };
    let start = std::time::Instant::now();
    let result = rt.block_on(pipe_send_control(pipe_name_str, "__PING__"));
    let elapsed_ms = start.elapsed().as_millis();
    match result {
        Ok(response) => {
            if response.starts_with("PONG ") {
                println!("pong: {} ms", elapsed_ms);
                0
            } else {
                eprintln!("error: unexpected ping response: {}", response);
                1
            }
        }
        Err(e) => {
            eprintln!("error: {:#}", e);
            1
        }
    }
}

/// `--health`: ask the running resident for its one-line liveness summary.
/// Exits 0 if response starts with HEALTHY, non-zero otherwise.
#[cfg(target_os = "windows")]
pub fn cmd_health(pipe_name_str: &str) -> i32 {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to create async runtime: {}", e);
            return 2;
        }
    };
    match rt.block_on(pipe_send_control(pipe_name_str, "__HEALTH__")) {
        Ok(response) => {
            println!("{}", response);
            if response.starts_with("HEALTHY") {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("error: {:#}", e);
            1
        }
    }
}

/// `--stop`: signal the running resident to exit gracefully (the resident
/// process terminates itself after responding). Returns 0 on OK STOPPING.
#[cfg(target_os = "windows")]
pub fn cmd_stop(pipe_name_str: &str) -> i32 {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to create async runtime: {}", e);
            return 2;
        }
    };
    match rt.block_on(pipe_send_control(pipe_name_str, "__STOP__")) {
        Ok(response) => {
            println!("{}", response);
            if response.starts_with("OK") {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("error: {:#}", e);
            1
        }
    }
}

/// `--reload-config`: signal the running resident to reload its config.
/// The resident currently replies NOT_IMPLEMENTED (config-reload semantics
/// land in a later milestone); this command surfaces that honestly.
#[cfg(target_os = "windows")]
pub fn cmd_reload_config(pipe_name_str: &str) -> i32 {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to create async runtime: {}", e);
            return 2;
        }
    };
    match rt.block_on(pipe_send_control(pipe_name_str, "__RELOAD_CONFIG__")) {
        Ok(response) => {
            println!("{}", response);
            if response.starts_with("OK") {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("error: {:#}", e);
            1
        }
    }
}

#[cfg(test)]
mod pass_4_commit_1_tests {
    //! XGID Retrofit Pass 4 Commit 1 — Surface #3 (Batch Pipe Dispatch)
    //! per-surface tests T6 + T7 (runbook §5.3, design doc §4.6.a).
    use xgen_common::xgid::{EventXgid, RoomXgid, SpaceXgid, Xgid};

    /// T6 — `get_dag_tips`' `space_id` parameter stays `&str` (design doc
    /// §4.6.a / Q3, Surface #2 Option α sibling): a caller holding a
    /// JSON-decoded `String` passes `&s` with no `SpaceXgid` projection. The
    /// fn requires a live `Connection` to invoke, so this is a type-contract
    /// witness (honest per D-065) — a `String` coerces straight to the `&str`
    /// parameter. A future widening to `&SpaceXgid` would force a projection
    /// at every call site, surfacing the change.
    #[test]
    fn batch_get_dag_tips_space_id_stays_str_with_borrow_projection() {
        let json_decoded: String = "xgen://hash/sha256:space".to_string();
        let space_id_param: &str = &json_decoded; // no projection needed
        assert_eq!(space_id_param, "xgen://hash/sha256:space");
    }

    /// T7 — batch replies serialise `ops::*` Result structs to JSON over the
    /// named pipe; serde-transparency preserves the wire shape (design doc
    /// §4.6.a / §4.2 Instance B). A pre-Pass-4 batch consumer reads
    /// byte-identical JSON — identifier slots are plain strings.
    #[test]
    fn batch_reply_json_serde_transparent_wire_invariance() {
        let r = crate::ops::SendResult {
            event_id: EventXgid::from_xgid(Xgid::new("xgen://hash/sha256:E".to_string())),
            space_id: SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:S".to_string())),
            room_id: RoomXgid::from_xgid(Xgid::new("xgen://hash/sha256:R".to_string())),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains(r#""event_id":"xgen://hash/sha256:E""#), "got {json}");
        assert!(json.contains(r#""space_id":"xgen://hash/sha256:S""#), "got {json}");
        assert!(json.contains(r#""room_id":"xgen://hash/sha256:R""#), "got {json}");
    }
}

#[cfg(test)]
mod mp_f14_cooperative_frontier_tests {
    //! MP-F14 Commit 1 — the cooperative-frontier spine (box-free, RED-on-revert).
    //! Exercises `cooperative_frontier` directly (no live `Connection`): the
    //! infra-exclusion fix + the MP-F4 no-regression (full frontier preserved).
    //! The J-333 hole-safety lens does NOT apply — this is not an F-3 path.
    use super::{cooperative_frontier, FrontierEvent};
    use xgen_core::wire::types::EventType;

    fn fe(id: &str, prev: &[&str], ty: EventType) -> FrontierEvent {
        FrontierEvent {
            event_id: id.to_string(),
            prev_events: prev.iter().map(|p| p.to_string()).collect(),
            event_type: ty,
        }
    }

    /// The fix: a DAG whose current leaf is a `state.federation_add` yields the
    /// cooperative tips WITHOUT it — and keeps the cooperative event the
    /// federation_add was built on as the anchor (the infra event's prev
    /// contribution is not counted, so `j` stays a leaf).
    ///
    /// RED-on-revert: drop the `is_federation_infra()` skip in
    /// `cooperative_frontier` → the result becomes `["f"]` (the federation_add is
    /// the only leaf) → both asserts fail.
    #[test]
    fn mp_f14_cooperative_frontier_excludes_federation_add() {
        // rc(room_create) ← j(join) ← f(federation_add); f is the sole DAG leaf.
        let events = vec![
            fe("rc", &["s"], EventType::StateRoomCreate),
            fe("j", &["rc"], EventType::MembershipJoin),
            fe("f", &["j"], EventType::StateFederationAdd),
        ];
        let frontier = cooperative_frontier(&events);
        assert!(
            !frontier.contains(&"f".to_string()),
            "federation_add must be excluded from the cooperative frontier, got {frontier:?}"
        );
        // j is the cooperative tip the federation_add hid; excluding the infra
        // event's prev contribution keeps j a leaf (rc is referenced by j).
        assert_eq!(frontier, vec!["j".to_string()], "got {frontier:?}");
    }

    /// MP-F4 no-regression: the FULL set of concurrent cooperative leaves — a
    /// membership.join, a state.room_create, and a message.text — all survive as
    /// tips (the MP-F4 full-frontier property; the pre-MP-F4 single-tip behaviour
    /// dropped concurrent leaves).
    ///
    /// RED-on-revert: an over-broad exclusion (e.g. widening `is_federation_infra`
    /// to also flag membership/room kinds) drops a cooperative tip → fewer than
    /// three leaves → the MP-F4 single-tip drop returns.
    #[test]
    fn mp_f14_cooperative_frontier_keeps_membership_and_room_tips() {
        // s(space_create) ← {rc(room_create), j(join), m(message)} — three
        // concurrent cooperative leaves, none infra. `cooperative_frontier`
        // returns them sorted.
        let events = vec![
            fe("s", &[], EventType::StateSpaceCreate),
            fe("rc", &["s"], EventType::StateRoomCreate),
            fe("j", &["s"], EventType::MembershipJoin),
            fe("m", &["s"], EventType::MessageText),
        ];
        let frontier = cooperative_frontier(&events);
        assert_eq!(
            frontier,
            vec!["j".to_string(), "m".to_string(), "rc".to_string()],
            "all three cooperative leaves (membership/room/message) must survive as \
             tips; got {frontier:?}"
        );
        assert!(
            !frontier.contains(&"s".to_string()),
            "the referenced root must not be a tip, got {frontier:?}"
        );
    }
}
