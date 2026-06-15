// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Connection lifecycle — CONNECT → AUTHENTICATE → ACTIVE → CLOSE (spec 3.3.4).
//
// `Connection<S>` wraps a WebSocketStream and provides:
//   - server_authenticate()  — issue challenge, verify response, send auth_ok/auth_fail
//   - client_authenticate()  — receive challenge, send signed response, receive auth_ok
//   - send_transport() / send_event() — outbound framed messages
//   - recv()  — next inbound message (transport control, Event, or WebSocket signal)
//   - goodbye() — graceful close (spec 3.3.9)
//   - ping()   — WebSocket keepalive ping (spec 3.3.5)

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use futures_util::{SinkExt, StreamExt};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::wire::{
    framing::{decode_frame, encode_frame, FrameError},
    types::{
        BootstrapMessage, DmControlMessage, Event, FederationMessage, IdentityMessage,
        IdentityReplicateMessage, MigrationMessage, MlsMessage, ReputationMessage,
        SpaceControlMessage, TransportMessage,
    },
};

use super::auth::{self, AuthError};

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("frame error: {0}")]
    Frame(#[from] FrameError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("authentication error: {0}")]
    Auth(#[from] AuthError),
    #[error("authentication failed (code {0}): {1}")]
    AuthFailed(u32, String),
    #[error("unexpected message in {0} phase: {1}")]
    UnexpectedMessage(&'static str, String),
    #[error("connection closed by peer")]
    Closed,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// ── Inbound message discriminant ──────────────────────────────────────────────

/// All message kinds that can arrive on a WebSocket connection.
#[derive(Debug)]
pub enum Inbound {
    /// An XGen protocol Event.
    Event(Event),
    /// A transport control message.
    Transport(TransportMessage),
    /// A federation handshake message.
    Federation(FederationMessage),
    /// An identity registration or retrieval message.
    Identity(IdentityMessage),
    /// Identity replication message (identity.replicate / identity.replicate_ack).
    /// Routed before Identity to avoid mis-routing identity.replicate* as IdentityMessage.
    IdentityReplicate(IdentityReplicateMessage),
    /// A space-level control message (join request, etc.).
    Space(SpaceControlMessage),
    /// DM Space promotion control message (dm.*).
    DmControl(DmControlMessage),
    /// Space migration control message (migration.*).
    Migration(MigrationMessage),
    /// Bootstrap Node protocol message (bootstrap.*).
    Bootstrap(BootstrapMessage),
    /// Reputation protocol message (reputation.*).
    Reputation(ReputationMessage),
    /// MLS E2E encryption message (mls.*).
    Mls(MlsMessage),
    /// WebSocket-level ping from peer (tungstenite already replied with pong).
    Ping(Vec<u8>),
    /// WebSocket-level pong (response to our ping).
    Pong(Vec<u8>),
    /// The peer closed the WebSocket connection.
    Closed,
}

// ── Send-confirm outcome (MP-F1a, F1A-D2) ─────────────────────────────────────

/// The node's deterministic outcome for an event submitted via
/// [`Connection::send_event_confirmed`]. The helper *observes* this; the caller
/// (the `ops::*` verb) *applies policy* (MP-F1a F1A-D3/D4): `Accepted` and
/// `Rejected` both mean "the node took the event" → proceed; `TimedOut` is an
/// outcome, not an error, so the op decides per its class (a multi-event chain
/// errors-and-aborts because await-ack already proved the predecessor present,
/// so the timeout is genuinely-lost; a single-event op warns-and-proceeds
/// because its timeout is irreducibly lost-vs-HeldPending — design §F1A-D4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventConfirm {
    /// The node sent `EventAccepted` for this event_id (validated + durably
    /// persisted, D-070 G2 boundary).
    Accepted,
    /// The node sent `Error` for this event_id — a deterministic rejection
    /// (`code` = wire `error_code`, `reason` = wire `error_string`,
    /// `event_id` = the rejected event's id, MP-F5: carried so the client can
    /// surface it structurally in the aicontrol reject reply).
    Rejected {
        code: u32,
        reason: String,
        event_id: String,
    },
    /// No signal correlated to this event_id arrived within the timeout. The
    /// node may have lost the event OR be holding it (HeldPending emits no
    /// signal — the named silent residue, F1A-D5).
    TimedOut,
}

/// Result of `client_authenticate` (M10.4 / MP-F13, Shape B). Carries the
/// echoed `identity_id` (the client's own, as before) plus the Node's `node_id`
/// when the Node advertised it on `AuthOk`. `node_id` is `None` against an older
/// Node that predates the additive echo — the client must NOT fall back to
/// writing a transport URL as a Space `home_node` in that case (see
/// `ops::create_space`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthOutcome {
    pub identity_id: String,
    pub node_id: Option<String>,
}

// ── Connection ────────────────────────────────────────────────────────────────

pub struct Connection<S> {
    ws: WebSocketStream<S>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S> {
    pub fn new(ws: WebSocketStream<S>) -> Self {
        Self { ws }
    }

    // ── Low-level send / recv ─────────────────────────────────────────────────

    async fn send_bytes(&mut self, payload: &[u8]) -> Result<(), TransportError> {
        let frame = encode_frame(payload);
        self.ws
            .send(Message::Binary(frame))
            .await
            .map_err(TransportError::WebSocket)
    }

    /// Send a transport control message.
    pub async fn send_transport(&mut self, msg: &TransportMessage) -> Result<(), TransportError> {
        let json = serde_json::to_vec(msg)?;
        self.send_bytes(&json).await
    }

    /// Send a protocol Event.
    pub async fn send_event(&mut self, event: &Event) -> Result<(), TransportError> {
        let json = serde_json::to_vec(event)?;
        self.send_bytes(&json).await
    }

    /// Send a protocol Event and await the node's deterministic outcome for it
    /// (MP-F1a, F1A-D2). Sends `event`, then drains `recv()` until a transport
    /// `EventAccepted` / `Error` whose `event_id` matches the sent event arrives,
    /// or `timeout` elapses.
    ///
    /// This is the single source of the submit-and-await discipline (F1A-D1):
    /// *a verb does not return / close until each event it sent is
    /// node-confirmed* (or its timeout policy fires). It exists because
    /// fire-and-forget `send_event` + immediate `goodbye` lets an abrupt
    /// connection teardown drop events the node never read (MP-F1 facet-2).
    ///
    /// Correlation is by `event_id` via the centralised
    /// [`TransportMessage::event_id`] accessor (D-067 no-drift — the match lives
    /// in exactly one place). Inbound that does NOT correlate to this event_id
    /// (fan-out `Event`s for other Spaces, ping/pong, signals for a different
    /// in-flight submission) is **skipped** — the helper waits only for *this*
    /// event's outcome. A closed/errored connection surfaces as
    /// [`TransportError`]; a `TimedOut` is returned as an [`EventConfirm`]
    /// outcome (not an error) so the caller applies its op-class policy.
    pub async fn send_event_confirmed(
        &mut self,
        event: &Event,
        timeout: std::time::Duration,
    ) -> Result<EventConfirm, TransportError> {
        // The sent event_id is the correlation key. A signed event always has
        // one; a `None` here is a caller bug (an unsigned event), not a wire
        // condition — surface it rather than wait for a match that can't occur.
        let sent_id = event
            .event_id
            .as_ref()
            .map(|e| e.as_str().to_string())
            .ok_or(TransportError::UnexpectedMessage(
                "send_event_confirmed",
                "event has no event_id (unsigned)".to_string(),
            ))?;

        self.send_event(event).await?;

        let drain = async {
            loop {
                match self.recv().await? {
                    Inbound::Transport(tm) if tm.event_id() == Some(sent_id.as_str()) => {
                        return Ok(match tm {
                            TransportMessage::EventAccepted { .. } => EventConfirm::Accepted,
                            TransportMessage::Error {
                                error_code,
                                error_string,
                                ..
                            } => EventConfirm::Rejected {
                                code: error_code,
                                reason: error_string,
                                // MP-F5: the matched correlation key IS this
                                // Error frame's event_id — surface it.
                                event_id: sent_id.clone(),
                            },
                            // `event_id()` returns Some ONLY for EventAccepted / Error
                            // (wire/types.rs). A variant matched by event_id that is
                            // neither means a new event_id-bearing TransportMessage was
                            // added without extending this arm — trip loudly rather than
                            // silently report a false "node took it" (D-065/B3: a wildcard
                            // Accepted would mis-map uniformly across all 9 retrofitted
                            // ops). Keep this match in lockstep with
                            // TransportMessage::event_id().
                            _ => unreachable!(
                                "send_event_confirmed: event_id matched a \
                                 non-Accepted/Error TransportMessage"
                            ),
                        });
                    }
                    // Closed peer ends the wait with a transport error.
                    Inbound::Closed => return Err(TransportError::Closed),
                    // Anything not correlated to this event_id: keep waiting.
                    _ => continue,
                }
            }
        };

        match tokio::time::timeout(timeout, drain).await {
            Ok(result) => result,
            Err(_) => Ok(EventConfirm::TimedOut),
        }
    }

    /// M12.1 (M12-D1, R-2) — upload a **ciphertext** blob to the node's
    /// content-blind blob store over WS as chunked base64. Sends `BlobUploadBegin`
    /// → `BlobChunk`* (≤ `BLOB_CHUNK_BYTES` raw per chunk, base64-encoded) →
    /// `BlobUploadEnd`, then drains `recv()` until `BlobUploadOk` (returns the
    /// node-confirmed `blob_ref`) / `Error` (a blob reject, e.g. 10001) / timeout.
    /// `ciphertext` is already encrypted (M12-D5); the store never sees plaintext.
    pub async fn upload_blob(
        &mut self,
        blob_ref: &str,
        ciphertext: &[u8],
        timeout: std::time::Duration,
    ) -> Result<String, TransportError> {
        self.send_transport(&TransportMessage::BlobUploadBegin {
            protocol_version: "0.1".to_string(),
            blob_ref: blob_ref.to_string(),
            size: ciphertext.len() as u64,
        })
        .await?;
        for (seq, chunk) in ciphertext.chunks(crate::wire::types::BLOB_CHUNK_BYTES).enumerate() {
            self.send_transport(&TransportMessage::BlobChunk {
                protocol_version: "0.1".to_string(),
                seq: seq as u32,
                data: crate::crypto::encoding::encode(chunk),
            })
            .await?;
        }
        self.send_transport(&TransportMessage::BlobUploadEnd {
            protocol_version: "0.1".to_string(),
            blob_ref: blob_ref.to_string(),
        })
        .await?;

        let drain = async {
            loop {
                match self.recv().await? {
                    Inbound::Transport(TransportMessage::BlobUploadOk { blob_ref, .. }) => {
                        return Ok(blob_ref);
                    }
                    Inbound::Transport(TransportMessage::Error {
                        error_code,
                        error_string,
                        ..
                    }) => {
                        return Err(TransportError::UnexpectedMessage(
                            "upload_blob",
                            format!("blob reject {error_code}: {error_string}"),
                        ));
                    }
                    Inbound::Closed => return Err(TransportError::Closed),
                    _ => continue,
                }
            }
        };
        match tokio::time::timeout(timeout, drain).await {
            Ok(result) => result,
            Err(_) => Err(TransportError::UnexpectedMessage(
                "upload_blob",
                "timed out awaiting BlobUploadOk".to_string(),
            )),
        }
    }

    /// M12.1 (M12-D1, R-2) — fetch a blob's **ciphertext** by content-address over
    /// WS. Sends `BlobFetchRequest`, then reassembles the streamed `BlobChunk`* in
    /// arrival order (a single WS stream preserves order) until `BlobFetchEnd`.
    /// `Error` (miss / mismatch) or timeout surfaces as `TransportError`. The
    /// caller decrypts + verifies `plaintext_hash` (M12-D5 / the C4 W3 check).
    pub async fn fetch_blob(
        &mut self,
        blob_ref: &str,
        timeout: std::time::Duration,
    ) -> Result<Vec<u8>, TransportError> {
        self.send_transport(&TransportMessage::BlobFetchRequest {
            protocol_version: "0.1".to_string(),
            blob_ref: blob_ref.to_string(),
        })
        .await?;

        let drain = async {
            let mut buf: Vec<u8> = Vec::new();
            loop {
                match self.recv().await? {
                    Inbound::Transport(TransportMessage::BlobChunk { data, .. }) => {
                        let bytes = crate::crypto::encoding::decode(&data).map_err(|_| {
                            TransportError::UnexpectedMessage(
                                "fetch_blob",
                                "malformed base64 chunk".to_string(),
                            )
                        })?;
                        buf.extend_from_slice(&bytes);
                    }
                    Inbound::Transport(TransportMessage::BlobFetchEnd { .. }) => return Ok(buf),
                    Inbound::Transport(TransportMessage::Error {
                        error_code,
                        error_string,
                        ..
                    }) => {
                        return Err(TransportError::UnexpectedMessage(
                            "fetch_blob",
                            format!("blob reject {error_code}: {error_string}"),
                        ));
                    }
                    Inbound::Closed => return Err(TransportError::Closed),
                    _ => continue,
                }
            }
        };
        match tokio::time::timeout(timeout, drain).await {
            Ok(result) => result,
            Err(_) => Err(TransportError::UnexpectedMessage(
                "fetch_blob",
                "timed out awaiting blob".to_string(),
            )),
        }
    }

    /// Send a federation handshake message.
    pub async fn send_federation(&mut self, msg: &FederationMessage) -> Result<(), TransportError> {
        let json = serde_json::to_vec(msg)?;
        self.send_bytes(&json).await
    }

    /// Send an identity protocol message.
    pub async fn send_identity(&mut self, msg: &IdentityMessage) -> Result<(), TransportError> {
        let json = serde_json::to_vec(msg)?;
        self.send_bytes(&json).await
    }

    /// Send a space control message.
    pub async fn send_space(&mut self, msg: &SpaceControlMessage) -> Result<(), TransportError> {
        let json = serde_json::to_vec(msg)?;
        self.send_bytes(&json).await
    }

    /// Send an identity replication message (identity.replicate / identity.replicate_ack).
    pub async fn send_identity_replicate(&mut self, msg: &IdentityReplicateMessage) -> Result<(), TransportError> {
        let json = serde_json::to_vec(msg)?;
        self.send_bytes(&json).await
    }

    /// Send a DM Space promotion control message (dm.*).
    pub async fn send_dm_control(&mut self, msg: &DmControlMessage) -> Result<(), TransportError> {
        let json = serde_json::to_vec(msg)?;
        self.send_bytes(&json).await
    }

    /// Send a space migration control message (migration.*).
    pub async fn send_migration(&mut self, msg: &MigrationMessage) -> Result<(), TransportError> {
        let json = serde_json::to_vec(msg)?;
        self.send_bytes(&json).await
    }

    /// Send a Bootstrap Node protocol message (bootstrap.*).
    pub async fn send_bootstrap(&mut self, msg: &BootstrapMessage) -> Result<(), TransportError> {
        let json = serde_json::to_vec(msg)?;
        self.send_bytes(&json).await
    }

    /// Send a reputation protocol message (reputation.*).
    pub async fn send_reputation(&mut self, msg: &ReputationMessage) -> Result<(), TransportError> {
        let json = serde_json::to_vec(msg)?;
        self.send_bytes(&json).await
    }

    /// Send an MLS E2E encryption message (mls.*).
    pub async fn send_mls(&mut self, msg: &MlsMessage) -> Result<(), TransportError> {
        let json = serde_json::to_vec(msg)?;
        self.send_bytes(&json).await
    }

    /// Receive the next message from the WebSocket.
    /// Silently skips text and raw frames (XGen only uses binary).
    pub async fn recv(&mut self) -> Result<Inbound, TransportError> {
        loop {
            let ws_msg = self
                .ws
                .next()
                .await
                .ok_or(TransportError::Closed)?
                .map_err(TransportError::WebSocket)?;

            match ws_msg {
                Message::Binary(data) => {
                    let frame = decode_frame(&data)?;
                    let value: serde_json::Value = serde_json::from_slice(&frame.payload)?;
                    let type_str = value["type"].as_str().unwrap_or("");
                    // DAG Events always carry a "sender" field; control messages (MlsMessage,
                    // BootstrapMessage, ReputationMessage, …) do not.  Check this FIRST so that
                    // event types whose wire prefix overlaps with a control-message prefix
                    // (e.g. "mls.key_package", "bootstrap.node_announce",
                    // "reputation.defederation_signal") are correctly routed to Inbound::Event
                    // instead of being deserialised as the wrong control type and silently
                    // killing the connection.
                    return if value.get("sender").is_some() {
                        Ok(Inbound::Event(serde_json::from_value(value)?))
                    } else if type_str.starts_with("transport.") {
                        Ok(Inbound::Transport(serde_json::from_value(value)?))
                    } else if type_str.starts_with("federation.") {
                        Ok(Inbound::Federation(serde_json::from_value(value)?))
                    } else if type_str == "identity.replicate" || type_str == "identity.replicate_ack" {
                        // Must be checked BEFORE the identity.* prefix — IdentityReplicateMessage
                        // is a separate enum from IdentityMessage; mis-routing it causes a
                        // deserialization error that silently kills the connection.
                        Ok(Inbound::IdentityReplicate(serde_json::from_value(value)?))
                    } else if type_str.starts_with("identity.") {
                        Ok(Inbound::Identity(serde_json::from_value(value)?))
                    } else if type_str.starts_with("space.") {
                        Ok(Inbound::Space(serde_json::from_value(value)?))
                    } else if type_str.starts_with("dm.") {
                        Ok(Inbound::DmControl(serde_json::from_value(value)?))
                    } else if type_str.starts_with("migration.") {
                        Ok(Inbound::Migration(serde_json::from_value(value)?))
                    } else if type_str.starts_with("bootstrap.") {
                        Ok(Inbound::Bootstrap(serde_json::from_value(value)?))
                    } else if type_str.starts_with("reputation.") {
                        Ok(Inbound::Reputation(serde_json::from_value(value)?))
                    } else if type_str.starts_with("mls.") {
                        Ok(Inbound::Mls(serde_json::from_value(value)?))
                    } else {
                        Ok(Inbound::Event(serde_json::from_value(value)?))
                    };
                }
                Message::Ping(data) => return Ok(Inbound::Ping(data)),
                Message::Pong(data) => return Ok(Inbound::Pong(data)),
                Message::Close(_) => return Ok(Inbound::Closed),
                Message::Text(_) | Message::Frame(_) => continue,
            }
        }
    }

    // ── Authentication ────────────────────────────────────────────────────────

    /// **Server-side** authentication (spec 3.3.4 Phase 2).
    /// Send challenge → verify auth response → send auth_ok or auth_fail.
    /// Returns the authenticated `identity_id` on success.
    ///
    /// `node_id` is this Node's own pubkey id (`xgen://pubkey/ed25519:…`), echoed
    /// on `AuthOk` so the client can write it as a Space's `home_node` (M10.4 /
    /// MP-F13, Shape B).
    pub async fn server_authenticate(
        &mut self,
        node_id: &str,
    ) -> Result<String, TransportError> {
        let (issued, challenge_msg) = auth::issue_challenge();
        self.send_transport(&challenge_msg).await?;

        let inbound = self.recv().await?;
        let auth_msg = match inbound {
            Inbound::Transport(ref tm @ TransportMessage::Auth { .. }) => tm.clone(),
            other => {
                return Err(TransportError::UnexpectedMessage(
                    "AUTHENTICATE",
                    format!("{other:?}"),
                ))
            }
        };

        match auth::verify_auth_response(&issued, &auth_msg) {
            Ok(identity_id) => {
                let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
                let ok = TransportMessage::AuthOk {
                    protocol_version: "0.1".to_string(),
                    identity_id: identity_id.clone(),
                    timestamp: ts,
                    node_id: Some(node_id.to_string()),
                };
                self.send_transport(&ok).await?;
                Ok(identity_id)
            }
            Err(e) => {
                let (code, error_string) = e.to_transport_code();
                let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
                let fail = TransportMessage::AuthFail {
                    protocol_version: "0.1".to_string(),
                    error_code: code,
                    error_string: error_string.to_string(),
                    timestamp: ts,
                };
                let _ = self.send_transport(&fail).await;
                Err(TransportError::AuthFailed(code, error_string.to_string()))
            }
        }
    }

    /// **Client-side** authentication (spec 3.3.4 Phase 2).
    /// Receive challenge → sign nonce → send auth → receive auth_ok.
    /// Returns the `identity_id` echoed by the server plus the Node's `node_id`
    /// (M10.4 / MP-F13, Shape B — `None` against a pre-echo Node).
    pub async fn client_authenticate(
        &mut self,
        signing_key: &SigningKey,
    ) -> Result<AuthOutcome, TransportError> {
        // Receive challenge.
        let nonce = match self.recv().await? {
            Inbound::Transport(TransportMessage::Challenge { nonce, .. }) => nonce,
            other => {
                return Err(TransportError::UnexpectedMessage(
                    "AUTHENTICATE",
                    format!("{other:?}"),
                ))
            }
        };

        // Build and send signed response.
        let auth_msg = auth::build_auth_response(&nonce, signing_key);
        self.send_transport(&auth_msg).await?;

        // Receive auth_ok or auth_fail.
        match self.recv().await? {
            Inbound::Transport(TransportMessage::AuthOk {
                identity_id, node_id, ..
            }) => Ok(AuthOutcome { identity_id, node_id }),
            Inbound::Transport(TransportMessage::AuthFail {
                error_code,
                error_string,
                ..
            }) => Err(TransportError::AuthFailed(error_code, error_string)),
            other => Err(TransportError::UnexpectedMessage(
                "AUTHENTICATE",
                format!("{other:?}"),
            )),
        }
    }

    // ── Active phase ──────────────────────────────────────────────────────────

    /// Send a WebSocket-level ping for keepalive (spec 3.3.5).
    /// The peer must respond with a pong within 10 seconds.
    pub async fn ping(&mut self) -> Result<(), TransportError> {
        self.ws
            .send(Message::Ping(vec![]))
            .await
            .map_err(TransportError::WebSocket)
    }

    /// Graceful close — send transport.goodbye then close the WebSocket (spec 3.3.9).
    /// `reason` should be one of: node_shutdown, client_disconnect, idle_timeout, maintenance.
    pub async fn goodbye(&mut self, reason: &str) -> Result<(), TransportError> {
        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let msg = TransportMessage::Goodbye {
            protocol_version: "0.1".to_string(),
            reason: reason.to_string(),
            timestamp: ts,
        };
        self.send_transport(&msg).await?;
        // Close the WebSocket gracefully; ignore any error at this point.
        let _ = self.ws.close(None).await;
        Ok(())
    }
}

// ── MP-F1a (C1) — send_event_confirmed unit tests ─────────────────────────────

#[cfg(test)]
mod send_confirm_tests {
    use super::*;
    use std::time::Duration;
    use tokio::io::DuplexStream;

    /// A connected in-process WS pair (no TCP, no TLS) over `tokio::io::duplex`.
    async fn ws_pair() -> (Connection<DuplexStream>, Connection<DuplexStream>) {
        let (c, s) = tokio::io::duplex(1 << 16);
        let (cw, sw) = tokio::join!(
            tokio_tungstenite::client_async("ws://localhost/", c),
            tokio_tungstenite::accept_async(s),
        );
        (Connection::new(cw.unwrap().0), Connection::new(sw.unwrap()))
    }

    fn signed_event(text: &str) -> Event {
        let key = crate::identity::keypair::generate();
        crate::space::state::sign_event(
            crate::message::exchange::build_message_text_event(&key, "s", "r", vec![], text),
            &key,
        )
    }

    fn event_id_of(ev: &Event) -> String {
        ev.event_id.as_ref().unwrap().as_str().to_string()
    }

    fn accepted(id: String) -> TransportMessage {
        TransportMessage::EventAccepted {
            protocol_version: "0.1".into(),
            event_id: id,
            accepted_at: "2026-06-08T00:00:00.000Z".into(),
        }
    }

    fn error_for(id: String, code: u32, reason: &str) -> TransportMessage {
        TransportMessage::Error {
            protocol_version: "0.1".into(),
            error_code: code,
            error_string: reason.into(),
            timestamp: "2026-06-08T00:00:00.000Z".into(),
            event_id: Some(id),
        }
    }

    // (a) EventAccepted for the sent id → Accepted.
    #[tokio::test]
    async fn confirm_accepted() {
        let (mut client, mut server) = ws_pair().await;
        let ev = signed_event("hi");
        let id = event_id_of(&ev);
        let srv = tokio::spawn(async move {
            let _ = server.recv().await; // the submitted event
            server.send_transport(&accepted(id)).await.unwrap();
            server // keep the connection open until the client has its outcome
        });
        let r = client
            .send_event_confirmed(&ev, Duration::from_secs(2))
            .await
            .unwrap();
        assert_eq!(r, EventConfirm::Accepted);
        let _ = srv.await;
    }

    // (b) Error for the sent id → Rejected { code, reason }.
    #[tokio::test]
    async fn confirm_rejected_carries_code_and_reason() {
        let (mut client, mut server) = ws_pair().await;
        let ev = signed_event("hi");
        let id = event_id_of(&ev);
        let srv = tokio::spawn(async move {
            let _ = server.recv().await;
            server
                .send_transport(&error_for(id, 3046, "event_timestamp_out_of_bounds"))
                .await
                .unwrap();
            server
        });
        let r = client
            .send_event_confirmed(&ev, Duration::from_secs(2))
            .await
            .unwrap();
        assert_eq!(
            r,
            EventConfirm::Rejected {
                code: 3046,
                reason: "event_timestamp_out_of_bounds".into(),
                event_id: event_id_of(&ev),
            }
        );
        let _ = srv.await;
    }

    // (c) No signal within the timeout → TimedOut (an outcome, not an error).
    #[tokio::test]
    async fn confirm_times_out_when_silent() {
        let (mut client, mut server) = ws_pair().await;
        let ev = signed_event("hi");
        let srv = tokio::spawn(async move {
            let _ = server.recv().await; // receive but never reply
            tokio::time::sleep(Duration::from_secs(1)).await; // hold the connection open
            server
        });
        let r = client
            .send_event_confirmed(&ev, Duration::from_millis(200))
            .await
            .unwrap();
        assert_eq!(r, EventConfirm::TimedOut);
        let _ = srv.await;
    }

    // (d) An unrelated Event (different id) before the matching EventAccepted is
    //     skipped — the helper waits only for THIS event's outcome.
    #[tokio::test]
    async fn confirm_skips_unrelated_inbound() {
        let (mut client, mut server) = ws_pair().await;
        let ev = signed_event("mine");
        let id = event_id_of(&ev);
        let other = signed_event("someone-elses");
        let srv = tokio::spawn(async move {
            let _ = server.recv().await;
            server.send_event(&other).await.unwrap(); // unrelated fan-out
            server.send_transport(&accepted(id)).await.unwrap(); // then the real ack
            server
        });
        let r = client
            .send_event_confirmed(&ev, Duration::from_secs(2))
            .await
            .unwrap();
        assert_eq!(r, EventConfirm::Accepted);
        let _ = srv.await;
    }

    // (e) Connection dropped mid-await → TransportError (not a confirm outcome).
    #[tokio::test]
    async fn confirm_errors_when_connection_drops() {
        let (mut client, mut server) = ws_pair().await;
        let ev = signed_event("hi");
        let srv = tokio::spawn(async move {
            let _ = server.recv().await;
            drop(server); // close the peer without replying
        });
        let r = client
            .send_event_confirmed(&ev, Duration::from_secs(2))
            .await;
        assert!(matches!(r, Err(TransportError::Closed) | Err(TransportError::WebSocket(_))));
        let _ = srv.await;
    }

    // ── M12.1 blob transfer (upload_blob / fetch_blob) ──────────────────────────
    #[tokio::test]
    async fn blob_upload_chunks_reassemble_on_the_far_side() {
        // W4 spine: a multi-chunk ciphertext uploads + reassembles byte-identical
        // on the far side. RED-on-revert: dropping / mis-ordering chunk reassembly
        // (or breaking the base64 round-trip) corrupts the far-side buffer.
        let (mut client, mut server) = ws_pair().await;
        let payload = vec![0xABu8; crate::wire::types::BLOB_CHUNK_BYTES * 2 + 7];
        let blob_ref = crate::crypto::hashing::hash_uri(&payload);
        let expected = payload.clone();
        let bref = blob_ref.clone();
        let srv = tokio::spawn(async move {
            let mut buf = Vec::new();
            loop {
                match server.recv().await.unwrap() {
                    Inbound::Transport(TransportMessage::BlobUploadBegin { .. }) => {}
                    Inbound::Transport(TransportMessage::BlobChunk { data, .. }) => {
                        buf.extend_from_slice(&crate::crypto::encoding::decode(&data).unwrap());
                    }
                    Inbound::Transport(TransportMessage::BlobUploadEnd { blob_ref, .. }) => {
                        server
                            .send_transport(&TransportMessage::BlobUploadOk {
                                protocol_version: "0.1".into(),
                                blob_ref,
                            })
                            .await
                            .unwrap();
                        break;
                    }
                    _ => {}
                }
            }
            buf
        });
        let ok_ref = client
            .upload_blob(&blob_ref, &payload, Duration::from_secs(5))
            .await
            .unwrap();
        let reassembled = srv.await.unwrap();
        assert_eq!(ok_ref, bref);
        assert_eq!(reassembled, expected);
        assert!(expected.len() > crate::wire::types::BLOB_CHUNK_BYTES); // genuinely multi-chunk
    }

    #[tokio::test]
    async fn blob_fetch_reassembles_streamed_chunks() {
        let (mut client, mut server) = ws_pair().await;
        let payload = vec![0xCDu8; crate::wire::types::BLOB_CHUNK_BYTES + 100];
        let blob_ref = crate::crypto::hashing::hash_uri(&payload);
        let expected = payload.clone();
        let srv = tokio::spawn(async move {
            match server.recv().await.unwrap() {
                Inbound::Transport(TransportMessage::BlobFetchRequest { blob_ref, .. }) => {
                    for (seq, chunk) in
                        payload.chunks(crate::wire::types::BLOB_CHUNK_BYTES).enumerate()
                    {
                        server
                            .send_transport(&TransportMessage::BlobChunk {
                                protocol_version: "0.1".into(),
                                seq: seq as u32,
                                data: crate::crypto::encoding::encode(chunk),
                            })
                            .await
                            .unwrap();
                    }
                    server
                        .send_transport(&TransportMessage::BlobFetchEnd {
                            protocol_version: "0.1".into(),
                            blob_ref,
                        })
                        .await
                        .unwrap();
                }
                _ => panic!("expected a BlobFetchRequest"),
            }
        });
        let got = client
            .fetch_blob(&blob_ref, Duration::from_secs(5))
            .await
            .unwrap();
        srv.await.unwrap();
        assert_eq!(got, expected);
    }

    #[tokio::test]
    async fn blob_upload_surfaces_node_reject() {
        // A node Error during upload surfaces as a TransportError (no hang).
        let (mut client, mut server) = ws_pair().await;
        let payload = b"small".to_vec();
        let blob_ref = crate::crypto::hashing::hash_uri(&payload);
        let srv = tokio::spawn(async move {
            loop {
                match server.recv().await.unwrap() {
                    Inbound::Transport(TransportMessage::BlobUploadEnd { .. }) => {
                        server
                            .send_transport(&TransportMessage::Error {
                                protocol_version: "0.1".into(),
                                error_code: 10001,
                                error_string: "blob_hash_mismatch".into(),
                                timestamp: "2026-06-15T00:00:00.000Z".into(),
                                event_id: None,
                            })
                            .await
                            .unwrap();
                        break;
                    }
                    Inbound::Closed => break,
                    _ => {}
                }
            }
        });
        let r = client
            .upload_blob(&blob_ref, &payload, Duration::from_secs(5))
            .await;
        assert!(r.is_err());
        let _ = srv.await;
    }
}
