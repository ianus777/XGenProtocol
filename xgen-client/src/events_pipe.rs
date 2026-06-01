// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M7-events arc C5 — the client `.events` pipe surface.
//!
//! A second resident named-pipe server (sister to `aicontrol.rs`; `pipe.rs` /
//! `--batch` untouched, D-066) that lets a local driver observe the events this
//! Client receives from its home Node. Per `.events` connection the handler:
//!
//! 1. reads the mandatory first `subscribe` (its `args` are the AC-D3b
//!    [`Filter`]); `nodes` present → `BAD_ARGUMENT` (the `nodes` dimension is
//!    Node-only, EV-D4 — rejected loudly, not silently ignored);
//! 2. opens a **second same-identity WebSocket** to the home Node (EV-D3
//!    client side) — this rides the C1 `ClientSenders` multi-connection retype,
//!    so the AI resident's primary WS sender is not clobbered;
//! 3. tails the second WS's inbound events and **filters at the drain**
//!    (`matches`, with `event_nodes = &[]` since the client has no runtime node
//!    provenance), forwarding matches as bare Event JSONL until `unsubscribe`
//!    or connection close.
//!
//! Member-scoped by construction: the second WS receives only this Identity's
//! member-Space fan-out (entitlement is the ceiling). Live-only (Q2): only
//! `Inbound::Event` is forwarded — history is the command pipe's job.
//!
//! `state.event_subscriptions` (EV-D6) is the process-wide count of active
//! `.events` sessions ([`active_session_count`]). The client has no
//! `apply_fanout` registry, so the cross-cutting state is just the count —
//! each session is self-contained in its handler task (a full
//! `(ConnId, Filter, ws)` registry would only be needed if something iterated
//! sessions, which nothing does in v1).
//!
//! `handle_events_connection` is generic over the pipe stream so the
//! subscribe-parse / `nodes`-rejection / not-ready paths are testable over an
//! in-memory duplex; only `start_events_server` is `#[cfg(windows)]` (named
//! pipe, D-043).

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use xgen_common::aicontrol::{filter, matches, parse_command, ControlCode, ControlError, Filter, Reply};
use xgen_common::wire::Event;
use xgen_core::transport::client::connect_url;
use xgen_core::transport::connection::Inbound;

use crate::aicontrol::{aicontrol_pipe_name, instance_lifecycle};

static EVENTS_SESSIONS: AtomicUsize = AtomicUsize::new(0);

/// Process-wide count of active client `.events` sessions (EV-D6 — feeds
/// client `state.event_subscriptions`).
pub fn active_session_count() -> usize {
    EVENTS_SESSIONS.load(Ordering::Relaxed)
}

/// RAII guard: increments the active-session count on construction and
/// decrements on drop, so every exit path (close, WS loss, error) restores it.
struct SessionGuard;

impl SessionGuard {
    fn new() -> Self {
        EVENTS_SESSIONS.fetch_add(1, Ordering::Relaxed);
        SessionGuard
    }
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        EVENTS_SESSIONS.fetch_sub(1, Ordering::Relaxed);
    }
}

/// The client `.events` pipe name: the `.aicontrol` pipe name plus a `.events`
/// suffix (mirrors the node convention, C4: `…\<base>.aicontrol.events`).
pub fn events_pipe_name(batch_pipe_name: &str) -> String {
    format!("{}.events", aicontrol_pipe_name(batch_pipe_name))
}

/// Parse the mandatory first message: a `subscribe` command whose `args` are
/// the AC-D3b filter. Non-JSON / no `cmd` → `MALFORMED_COMMAND`; a different
/// verb or a malformed filter → `BAD_ARGUMENT`.
fn parse_subscribe(line: &str) -> Result<Filter, ControlError> {
    let command = parse_command(line)?;
    if command.cmd != "subscribe" {
        return Err(ControlError::new(
            ControlCode::BadArgument,
            format!(
                "first message on the .events pipe must be `subscribe`, got {:?}",
                command.cmd
            ),
        ));
    }
    filter::parse(serde_json::Value::Object(command.args))
}

/// The drain decision (EV-D4 client, filter-at-drain): forward an inbound WS
/// message iff it is an `Event` the filter matches. The client passes
/// `event_nodes = &[]` — it has no runtime node provenance, and a client filter
/// can never carry `nodes` (rejected at subscribe), so the `nodes` arm is
/// vacuously "all". Non-`Event` inbound (Transport / control) is live-only-
/// ignored (Q2).
fn forwardable<'a>(filter: &Filter, inbound: &'a Inbound) -> Option<&'a Event> {
    match inbound {
        Inbound::Event(e) if matches(filter, e, &[]) => Some(e),
        _ => None,
    }
}

/// Start the client `.events` pipe server. Independent of the `--batch` and
/// `.aicontrol` servers: its own accept loop, spawning one observer handler per
/// connection. Stops when `shutdown_rx` delivers `true`.
#[cfg(target_os = "windows")]
pub async fn start_events_server(
    pipe_name_str: String,
    data_dir: PathBuf,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    use tokio::net::windows::named_pipe::ServerOptions;

    tracing::info!(pipe = %pipe_name_str, "client events pipe server starting");
    let mut first = true;

    loop {
        if *shutdown_rx.borrow() {
            break;
        }
        let server = {
            let create = if first {
                first = false;
                ServerOptions::new()
                    .first_pipe_instance(true)
                    .create(&pipe_name_str)
            } else {
                ServerOptions::new().create(&pipe_name_str)
            };
            match create {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(error = %e, "client events pipe create failed — server stopping");
                    break;
                }
            }
        };

        let connected = tokio::select! {
            r = server.connect() => r.is_ok(),
            _ = shutdown_rx.changed() => false,
        };
        if !connected || *shutdown_rx.borrow() {
            break;
        }

        let handler_dir = data_dir.clone();
        tokio::spawn(async move {
            handle_events_connection(server, handler_dir).await;
        });
    }

    tracing::info!(pipe = %pipe_name_str, "client events pipe server stopped");
}

/// One observer session: read the mandatory `subscribe`, open a second
/// same-identity WS to the home Node, then forward matching live Events as
/// JSONL until `unsubscribe` or close. Generic over the pipe stream for
/// testability (the WS side uses the concrete `connect_url`).
async fn handle_events_connection<S>(stream: S, data_dir: PathBuf)
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let lifecycle = instance_lifecycle(&data_dir);
    let (reader_half, mut writer_half) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader_half);
    let mut buf = String::new();

    // Reply helper: write an envelope reply line; ignore write errors (the
    // caller returns immediately after an error reply anyway).
    async fn write_reply<W: tokio::io::AsyncWrite + Unpin>(w: &mut W, reply: Reply) {
        use tokio::io::AsyncWriteExt;
        let mut out = reply.to_line();
        out.push('\n');
        let _ = w.write_all(out.as_bytes()).await;
    }

    // ── Message 1 MUST be `subscribe`. ──
    let filter = loop {
        buf.clear();
        match reader.read_line(&mut buf).await {
            Ok(0) => return,
            Ok(_) => {}
            Err(_) => return,
        }
        let line = buf.trim_end_matches('\n').trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        match parse_subscribe(line) {
            Ok(f) => break f,
            Err(ce) => {
                write_reply(&mut writer_half, Reply::error(None, None, ce.into_body(lifecycle))).await;
                return;
            }
        }
    };

    // ── EV-D4 — `nodes` is Node-only; reject loudly on the client. ──
    if !filter.nodes.is_empty() {
        let ce = ControlError::new(
            ControlCode::BadArgument,
            "the `nodes` filter is Node-only and is not valid on a client `.events` subscription",
        );
        write_reply(&mut writer_half, Reply::error(None, None, ce.into_body(lifecycle))).await;
        return;
    }

    // ── Load identity + home node, open the second same-identity WS (EV-D3). ──
    let signing_key = match crate::session::ClientIdentity::load(
        &data_dir.join("xgen-client_keypair.enc"),
    ) {
        Ok(id) => id.signing_key,
        Err(_) => {
            let ce = ControlError::new(
                ControlCode::InstanceNotReady,
                "no client identity is set up (keypair missing)",
            );
            write_reply(&mut writer_half, Reply::error(None, None, ce.into_body(lifecycle))).await;
            return;
        }
    };
    let home_node = match crate::app::load_client_state(&data_dir) {
        Ok(s) => s.home_node,
        Err(_) => {
            let ce = ControlError::new(
                ControlCode::InstanceNotReady,
                "no client state (home node unknown)",
            );
            write_reply(&mut writer_half, Reply::error(None, None, ce.into_body(lifecycle))).await;
            return;
        }
    };
    if home_node.is_empty() {
        let ce = ControlError::new(ControlCode::InstanceNotReady, "home node is not set");
        write_reply(&mut writer_half, Reply::error(None, None, ce.into_body(lifecycle))).await;
        return;
    }

    let mut conn = match connect_url(&home_node).await {
        Ok(c) => c,
        Err(e) => {
            let ce = ControlError::new(
                ControlCode::ConnectionLost,
                format!("failed to open the observer WS to {home_node}: {e}"),
            );
            write_reply(&mut writer_half, Reply::error(None, None, ce.into_body(lifecycle))).await;
            return;
        }
    };
    if let Err(e) = conn.client_authenticate(&signing_key).await {
        let ce = ControlError::new(
            ControlCode::ConnectionLost,
            format!("observer WS authentication failed: {e}"),
        );
        write_reply(&mut writer_half, Reply::error(None, None, ce.into_body(lifecycle))).await;
        return;
    }

    // ── Subscribed. Count the session (decremented on every exit via the
    // guard) and ack. ──
    let _guard = SessionGuard::new();
    write_reply(
        &mut writer_half,
        Reply::ok("subscribe", None, serde_json::json!({ "subscribed": true })),
    )
    .await;

    // ── Stream: tail the second WS, forward matching live Events as JSONL; a
    // pipe `unsubscribe` line or close ends the session. ──
    loop {
        buf.clear();
        tokio::select! {
            biased;
            read = reader.read_line(&mut buf) => {
                match read {
                    Ok(0) => break, // pipe closed
                    Ok(_) => {
                        let line = buf.trim_end_matches('\n').trim_end_matches('\r').trim();
                        if let Ok(cmd) = parse_command(line) {
                            if cmd.cmd == "unsubscribe" {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            inbound = conn.recv() => {
                match inbound {
                    Ok(msg) => {
                        if let Some(ev) = forwardable(&filter, &msg) {
                            if let Ok(mut s) = serde_json::to_string(ev) {
                                s.push('\n');
                                if writer_half.write_all(s.as_bytes()).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(_) => break, // observer WS lost
                }
            }
        }
    }
    // _guard drops here → session count decremented.
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use xgen_common::wire::EventType;
    use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};

    // ── parse_subscribe (pure) ────────────────────────────────────────────

    #[test]
    fn parse_subscribe_accepts_valid() {
        let f = parse_subscribe(r#"{"cmd":"subscribe","args":{"event_types":["message.text"]}}"#)
            .expect("valid subscribe");
        assert_eq!(f.event_types, vec!["message.text".to_string()]);
    }

    #[test]
    fn parse_subscribe_wrong_verb_is_bad_argument() {
        let e = parse_subscribe(r#"{"cmd":"whoami"}"#).unwrap_err();
        assert_eq!(e.code, ControlCode::BadArgument);
    }

    #[test]
    fn parse_subscribe_non_json_is_malformed() {
        let e = parse_subscribe("garbage").unwrap_err();
        assert_eq!(e.code, ControlCode::MalformedCommand);
    }

    #[test]
    fn events_pipe_name_appends_suffix() {
        assert_eq!(
            events_pipe_name(r"\\.\pipe\xgen-client-default"),
            r"\\.\pipe\xgen-client-default.aicontrol.events"
        );
    }

    // ── forwardable (drain decision) ──────────────────────────────────────

    fn stub_event(ty: EventType, space: &str) -> Event {
        Event {
            protocol_version: "0.1".to_string(),
            event_type: ty,
            event_id: Some(EventXgid::from_xgid(Xgid::new("xgen://hash/sha256:EV".to_string()))),
            sender: IdentityXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:A".to_string())),
            room_id: RoomXgid::from_xgid(Xgid::new(String::new())),
            space_id: SpaceXgid::from_xgid(Xgid::new(space.to_string())),
            prev_events: vec![],
            timestamp: "2026-06-01T00:00:00.000Z".to_string(),
            content: serde_json::json!({}),
            meta_atts: None,
            signature: Some("ed25519:S:S".to_string()),
        }
    }

    #[test]
    fn forwardable_matches_event_passing_filter() {
        let f = filter::parse(serde_json::json!({ "event_types": ["message.text"] })).unwrap();
        let inbound = Inbound::Event(stub_event(EventType::MessageText, "xgen://hash/sha256:S"));
        assert!(forwardable(&f, &inbound).is_some());
    }

    #[test]
    fn forwardable_skips_event_filtered_out() {
        let f = filter::parse(serde_json::json!({ "event_types": ["message.text"] })).unwrap();
        let inbound = Inbound::Event(stub_event(EventType::StateRoomCreate, "xgen://hash/sha256:S"));
        assert!(forwardable(&f, &inbound).is_none());
    }

    #[test]
    fn forwardable_skips_non_event_inbound() {
        use xgen_core::wire::types::TransportMessage;
        let f = Filter::default(); // all-pass
        let inbound = Inbound::Transport(TransportMessage::SyncComplete {
            protocol_version: "0.1".to_string(),
            since: String::new(),
            new_tip: String::new(),
            continue_from: None,
        });
        assert!(forwardable(&f, &inbound).is_none(), "live-only: non-Event inbound is ignored");
    }

    // ── handler pre-WS paths over a duplex (no live Node needed) ───────────

    #[tokio::test]
    async fn nodes_filter_on_client_is_bad_argument() {
        let (client, server) = tokio::io::duplex(4096);
        let dir = tempfile::tempdir().unwrap();
        let handler = {
            let d = dir.path().to_path_buf();
            tokio::spawn(async move { handle_events_connection(server, d).await })
        };
        let (cr, mut cw) = tokio::io::split(client);
        let mut creader = BufReader::new(cr);
        cw.write_all(b"{\"cmd\":\"subscribe\",\"args\":{\"nodes\":[\"xgen://pubkey/ed25519:N\"]}}\n")
            .await
            .unwrap();
        let mut reply = String::new();
        creader.read_line(&mut reply).await.unwrap();
        assert!(reply.contains("BAD_ARGUMENT"), "reply: {reply}");
        assert!(reply.contains("Node-only"), "reply: {reply}");
        handler.await.unwrap();
        assert_eq!(active_session_count(), 0, "rejected subscribe must not count");
    }

    #[tokio::test]
    async fn not_ready_when_no_identity() {
        // Empty data_dir → no keypair → INSTANCE_NOT_READY before any WS.
        let (client, server) = tokio::io::duplex(4096);
        let dir = tempfile::tempdir().unwrap();
        let handler = {
            let d = dir.path().to_path_buf();
            tokio::spawn(async move { handle_events_connection(server, d).await })
        };
        let (cr, mut cw) = tokio::io::split(client);
        let mut creader = BufReader::new(cr);
        cw.write_all(b"{\"cmd\":\"subscribe\",\"args\":{}}\n").await.unwrap();
        let mut reply = String::new();
        creader.read_line(&mut reply).await.unwrap();
        assert!(reply.contains("INSTANCE_NOT_READY"), "reply: {reply}");
        handler.await.unwrap();
        assert_eq!(active_session_count(), 0);
    }
}
