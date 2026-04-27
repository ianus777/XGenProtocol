// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

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
    types::{Event, FederationMessage, IdentityMessage, TransportMessage},
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
    /// WebSocket-level ping from peer (tungstenite already replied with pong).
    Ping(Vec<u8>),
    /// WebSocket-level pong (response to our ping).
    Pong(Vec<u8>),
    /// The peer closed the WebSocket connection.
    Closed,
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

    /// Receive the next message from the WebSocket.
    /// Silently skips text and raw frames (XGen only uses binary).
    pub async fn recv(&mut self) -> Result<Inbound, TransportError> {
        loop {
            let ws_msg = self
                .ws
                .next()
                .await
                .ok_or(TransportError::Closed)
                .map_err(|e| e)?
                .map_err(TransportError::WebSocket)?;

            match ws_msg {
                Message::Binary(data) => {
                    let frame = decode_frame(&data)?;
                    let value: serde_json::Value = serde_json::from_slice(&frame.payload)?;
                    let type_str = value["type"].as_str().unwrap_or("");
                    return if type_str.starts_with("transport.") {
                        let tm: TransportMessage = serde_json::from_value(value)?;
                        Ok(Inbound::Transport(tm))
                    } else if type_str.starts_with("federation.") {
                        let fm: FederationMessage = serde_json::from_value(value)?;
                        Ok(Inbound::Federation(fm))
                    } else if type_str.starts_with("identity.") {
                        let im: IdentityMessage = serde_json::from_value(value)?;
                        Ok(Inbound::Identity(im))
                    } else {
                        let ev: Event = serde_json::from_value(value)?;
                        Ok(Inbound::Event(ev))
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
    pub async fn server_authenticate(&mut self) -> Result<String, TransportError> {
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
    /// Returns the `identity_id` echoed by the server on success.
    pub async fn client_authenticate(
        &mut self,
        signing_key: &SigningKey,
    ) -> Result<String, TransportError> {
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
            Inbound::Transport(TransportMessage::AuthOk { identity_id, .. }) => Ok(identity_id),
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
