// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M7-events arc C4 — the node `.events` pipe surface.
//!
//! A second resident named-pipe server (sister to `aicontrol.rs`; `pipe.rs` /
//! `--batch` are untouched, D-066) that lets an operator/AI **observe** the
//! Node's live fan-out. Each connection subscribes once with an AC-D3b
//! [`Filter`], is registered as a node observer in the process-global
//! `fanout::node_observers()` registry (C3, Shape β), and then receives the
//! matching `Event`s as JSONL as the Node fans them out. `unsubscribe` or
//! connection close prunes the entry — this server is the **writer** of the
//! registry the C3 fan-out hub reads.
//!
//! **Live-only (Q2):** the drain forwards only `OutboundMsg::Event`;
//! `HistoryBatch` / `SyncComplete` are ignored — history is the command pipe's
//! job. **Filtering** is done by `apply_fanout` *before* the send (C3, EV-D4 A),
//! so this handler simply forwards whatever lands in its channel. The `nodes`
//! filter dimension is meaningful here (Node-side, EV-D4 v1.1).
//!
//! The pipe transport is Windows-only (D-043); `handle_events_connection` is
//! generic over the stream so the subscribe → stream → prune path is testable
//! over an in-memory duplex without a real pipe (the `process_inbound`
//! generic-over-`S` pattern, J-086).

use xgen_common::aicontrol::{filter, parse_command, ControlCode, ControlError, Filter, Reply};
use xgen_common::conn::ConnId;

use crate::aicontrol::NODE_LIFECYCLE;
use crate::fanout::{node_observers, OutboundMsg};

/// The node `.events` pipe name: the `.aicontrol` pipe name plus a `.events`
/// suffix (resolves the C4 confirm-at-pickup #5 — the events surface is
/// namespaced under the aicontrol surface: `…\<base>.aicontrol.events`).
pub(crate) fn events_pipe_name(batch_pipe_name: &str) -> String {
    format!(
        "{}.events",
        crate::aicontrol::aicontrol_pipe_name(batch_pipe_name)
    )
}

/// Parse the mandatory first message: a `subscribe` command whose `args` are the
/// AC-D3b filter. Non-JSON / no `cmd` → `MALFORMED_COMMAND`; a different verb or
/// a malformed filter → `BAD_ARGUMENT` (all pre-stream, the subscribe being the
/// first message).
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

/// Start the node `.events` pipe server. Independent of the `--batch` and
/// `.aicontrol` servers: its own accept loop on the `.events` pipe name,
/// spawning one observer handler per connection. Stops when `shutdown_rx`
/// delivers `true`.
#[cfg(target_os = "windows")]
pub(crate) async fn start_events_server(
    pipe_name_str: String,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    use tokio::net::windows::named_pipe::ServerOptions;

    tracing::info!(pipe = %pipe_name_str, "node events pipe server starting");
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
                    tracing::error!(error = %e, "node events pipe create failed — server stopping");
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

        tokio::spawn(async move {
            handle_events_connection(server).await;
        });
    }

    tracing::info!(pipe = %pipe_name_str, "node events pipe server stopped");
}

/// One observer session: read the mandatory `subscribe`, register the observer
/// in the global registry, then forward matching live `Event`s as JSONL until
/// `unsubscribe` or connection close. Generic over the stream for testability.
///
/// `unsubscribe` is **best-effort** (it ends the session cleanly); connection
/// close is the reliable prune. `read_line` is not cancellation-safe inside the
/// `select!`, so a partial `unsubscribe` line may be garbled and ignored — but
/// EOF (close) is always observed, so the observer is always pruned.
pub(crate) async fn handle_events_connection<S>(stream: S)
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (reader_half, mut writer_half) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader_half);
    let mut buf = String::new();

    // ── Message 1 MUST be `subscribe`. Parse the filter; on malformed, reply
    // with the control error and close before any streaming begins. ──
    let filter = loop {
        buf.clear();
        match reader.read_line(&mut buf).await {
            Ok(0) => return, // closed before subscribing
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
                let mut out = Reply::error(None, None, ce.into_body(NODE_LIFECYCLE)).to_line();
                out.push('\n');
                let _ = writer_half.write_all(out.as_bytes()).await;
                return;
            }
        }
    };

    // ── Register the observer in the process-global registry (C3 writer). ──
    let conn = ConnId::mint();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<OutboundMsg>(1024);
    node_observers().lock().await.push((conn, filter, tx));

    let mut ack = Reply::ok("subscribe", None, serde_json::json!({ "subscribed": true })).to_line();
    ack.push('\n');
    if writer_half.write_all(ack.as_bytes()).await.is_err() {
        node_observers().lock().await.retain(|(c, _, _)| *c != conn);
        return;
    }

    // ── Stream: forward matching live Events as JSONL; an `unsubscribe` line or
    // connection close ends the session. Live-only (Q2): ignore HistoryBatch /
    // SyncComplete. ──
    loop {
        buf.clear();
        tokio::select! {
            biased;
            read = reader.read_line(&mut buf) => {
                match read {
                    Ok(0) => break, // connection closed
                    Ok(_) => {
                        let line = buf.trim_end_matches('\n').trim_end_matches('\r').trim();
                        if let Ok(cmd) = parse_command(line) {
                            if cmd.cmd == "unsubscribe" {
                                break;
                            }
                        }
                        // Any other line is ignored (the events pipe takes only
                        // the initial subscribe and an optional unsubscribe).
                    }
                    Err(_) => break,
                }
            }
            ev = rx.recv() => {
                match ev {
                    Some(OutboundMsg::Event(e)) => {
                        if let Ok(mut s) = serde_json::to_string(&e) {
                            s.push('\n');
                            if writer_half.write_all(s.as_bytes()).await.is_err() {
                                break;
                            }
                        }
                    }
                    // HistoryBatch / SyncComplete — live-only, ignored (Q2).
                    Some(_) => {}
                    None => break, // our sender was dropped (should not happen)
                }
            }
        }
    }

    // ── Prune. ──
    node_observers().lock().await.retain(|(c, _, _)| *c != conn);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};

    // ── parse_subscribe (pure) ────────────────────────────────────────────

    #[test]
    fn parse_subscribe_accepts_valid() {
        let f = parse_subscribe(r#"{"cmd":"subscribe","args":{"event_types":["message.text"]}}"#)
            .expect("valid subscribe");
        assert_eq!(f.event_types, vec!["message.text".to_string()]);
    }

    #[test]
    fn parse_subscribe_empty_filter_is_all() {
        let f = parse_subscribe(r#"{"cmd":"subscribe"}"#).expect("subscribe with no args");
        assert!(f.spaces.is_empty() && f.event_types.is_empty() && f.nodes.is_empty());
    }

    #[test]
    fn parse_subscribe_wrong_verb_is_bad_argument() {
        let e = parse_subscribe(r#"{"cmd":"state"}"#).unwrap_err();
        assert_eq!(e.code, ControlCode::BadArgument);
    }

    #[test]
    fn parse_subscribe_non_json_is_malformed() {
        let e = parse_subscribe("not json").unwrap_err();
        assert_eq!(e.code, ControlCode::MalformedCommand);
    }

    #[test]
    fn parse_subscribe_bad_filter_is_bad_argument() {
        let e = parse_subscribe(r#"{"cmd":"subscribe","args":{"event_types":["*.text"]}}"#)
            .unwrap_err();
        assert_eq!(e.code, ControlCode::BadArgument);
    }

    #[test]
    fn events_pipe_name_appends_suffix() {
        assert_eq!(
            events_pipe_name(r"\\.\pipe\xgen-node"),
            r"\\.\pipe\xgen-node.aicontrol.events"
        );
    }

    // ── handler round-trip over an in-memory duplex ───────────────────────

    fn stub_message_event() -> crate::wire::types::Event {
        crate::wire::types::Event {
            protocol_version: "0.1".to_string(),
            event_type: crate::wire::types::EventType::MessageText,
            event_id: Some(EventXgid::from_xgid(Xgid::new("xgen://hash/sha256:EV".to_string()))),
            sender: IdentityXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:A".to_string())),
            room_id: RoomXgid::from_xgid(Xgid::new(String::new())),
            space_id: SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:S".to_string())),
            prev_events: vec![],
            timestamp: "2026-06-01T00:00:00.000Z".to_string(),
            content: serde_json::json!({}),
            meta_atts: None,
            signature: Some("ed25519:S:S".to_string()),
        }
    }

    #[tokio::test]
    #[serial_test::serial(node_observers)]
    async fn subscribe_then_stream_then_prune_on_close() {
        let (client, server) = tokio::io::duplex(8192);
        let handler = tokio::spawn(async move { handle_events_connection(server).await });

        let (cr, mut cw) = tokio::io::split(client);
        let mut creader = BufReader::new(cr);

        // Subscribe.
        cw.write_all(b"{\"cmd\":\"subscribe\",\"args\":{}}\n")
            .await
            .unwrap();
        let mut ack = String::new();
        creader.read_line(&mut ack).await.unwrap();
        assert!(ack.contains("\"status\":\"ok\""), "ack: {ack}");

        // The handler registered exactly one observer (serial-grouped → isolated).
        let tx = {
            let obs = node_observers().lock().await;
            assert_eq!(obs.len(), 1, "one observer registered");
            obs[0].2.clone()
        };

        // Push a live Event into the observer channel; the handler forwards it
        // as bare Event JSONL (filtering is apply_fanout's job, C3).
        let ev = stub_message_event();
        tx.send(OutboundMsg::Event(ev.clone())).await.unwrap();
        let mut line = String::new();
        creader.read_line(&mut line).await.unwrap();
        let got: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(got["type"], "message.text");
        assert_eq!(got["event_id"], "xgen://hash/sha256:EV");

        // Close the client → the handler prunes the observer.
        drop(cw);
        drop(creader);
        handler.await.unwrap();
        assert!(
            node_observers().lock().await.is_empty(),
            "observer must be pruned on close"
        );
    }

    #[tokio::test]
    #[serial_test::serial(node_observers)]
    async fn malformed_subscribe_errors_and_registers_nothing() {
        let (client, server) = tokio::io::duplex(4096);
        let handler = tokio::spawn(async move { handle_events_connection(server).await });

        let (cr, mut cw) = tokio::io::split(client);
        let mut creader = BufReader::new(cr);

        // Wrong first verb → BAD_ARGUMENT, then the handler closes.
        cw.write_all(b"{\"cmd\":\"state\"}\n").await.unwrap();
        let mut reply = String::new();
        creader.read_line(&mut reply).await.unwrap();
        assert!(reply.contains("BAD_ARGUMENT"), "reply: {reply}");

        handler.await.unwrap();
        assert!(
            node_observers().lock().await.is_empty(),
            "no observer registered on a malformed subscribe"
        );
    }
}
