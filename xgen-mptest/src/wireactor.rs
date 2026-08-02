// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! A legitimate wire actor (C5 support).
//!
//! A small persistent wire-client that authenticates, registers, and submits
//! correctly-signed Events to a real node — the cooperative counterpart to the
//! hostile [`crate::injector`]. The harness **holds the actor's signing key**,
//! which is what lets the adversarial MP-A-05 smoke forge a *member's* identity
//! (claim Alice, sign with the attacker key) and so isolate the step-12
//! signature check (steps 9–11 pass because Alice really is a member here).
//!
//! It speaks the real transport to the real binary (the node validates + ingests
//! everything) — driving/observing, not patching.

use ed25519_dalek::SigningKey;
use tokio_tungstenite::MaybeTlsStream;

use xgen_common::wire::Event;
use xgen_core::identity::registration::{build_register, identity_id_from_key, sign_register};
use xgen_core::space::state::{build_room_create_event, build_space_create_event, sign_event};
use xgen_core::transport::client::connect_url;
use xgen_core::transport::connection::{Connection, Inbound};
use xgen_core::wire::types::IdentityMessage;

use crate::Result;
use anyhow::{anyhow, Context};
use std::time::Duration;

/// A connected, authenticated legitimate actor.
pub struct WireActor {
    conn: Connection<MaybeTlsStream<tokio::net::TcpStream>>,
    key: SigningKey,
    identity_id: String,
    node_url: String,
}

impl WireActor {
    /// Connect to `node_url` and authenticate with a fresh key.
    pub async fn connect(node_url: &str) -> Result<WireActor> {
        let key = xgen_core::identity::keypair::generate();
        Self::connect_with_key(node_url, key).await
    }

    /// Connect to `node_url` and authenticate with a specific key (so the caller
    /// retains it — needed to later forge this actor's identity).
    pub async fn connect_with_key(node_url: &str, key: SigningKey) -> Result<WireActor> {
        let mut conn = connect_url(node_url)
            .await
            .with_context(|| format!("WireActor connect {node_url}"))?;
        let identity_id = conn
            .client_authenticate(&key)
            .await
            .with_context(|| format!("WireActor authenticate {node_url}"))?
            .identity_id;
        Ok(WireActor {
            conn,
            key,
            // `AuthOutcome.identity_id` is now `IdentityXgid`; project to this
            // actor's `String` field at the consumer (M-RP-XGID-SLOT-RETYPE Leg C).
            identity_id: identity_id.as_str().to_string(),
            node_url: node_url.to_string(),
        })
    }

    /// This actor's identity id (`xgen://pubkey/ed25519:...`).
    pub fn identity_id(&self) -> &str {
        &self.identity_id
    }

    /// This actor's signing key (so the caller can craft a forgery claiming it).
    pub fn key(&self) -> &SigningKey {
        &self.key
    }

    /// Register this identity (Local-Node mode: no trust assertion).
    pub async fn register(&mut self, display_name: &str) -> Result<()> {
        let reg = sign_register(build_register(&self.key, Some(display_name.to_string())), &self.key);
        self.conn
            .send_identity(&reg)
            .await
            .context("WireActor send register")?;
        match self.conn.recv().await.context("WireActor recv register reply")? {
            Inbound::Identity(IdentityMessage::RegisterOk { .. }) => Ok(()),
            other => Err(anyhow!(
                "registration of `{display_name}` was not accepted: {other:?}"
            )),
        }
    }

    /// Submit a pre-built signed Event; briefly drain any reply (non-fatal).
    pub async fn submit(&mut self, event: &Event) -> Result<()> {
        self.conn
            .send_event(event)
            .await
            .context("WireActor send_event")?;
        // Drain a possible EventAccepted/Error without blocking the flow.
        let _ = tokio::time::timeout(Duration::from_millis(150), self.conn.recv()).await;
        Ok(())
    }

    /// Submit a pre-built signed Event **without draining the ack** (MP-R3 §6.3
    /// flood firehose) — the send returns as soon as the bytes are written, so a
    /// flood is bounded only by write throughput, not ack latency. Acks pile up
    /// unread on the socket; the connection is dropped at end of flood.
    pub async fn submit_no_drain(&mut self, event: &Event) -> Result<()> {
        self.conn
            .send_event(event)
            .await
            .context("WireActor send_event (no drain)")?;
        Ok(())
    }

    /// Submit a pre-built signed Event and **listen for the Node's `Error`
    /// reply** within `dur`, returning `(error_code, error_string)` if one
    /// arrives (else `None`). This is the C7 wire-path capability the batch
    /// client lacks (fire-and-forget): the raw `Connection` keeps the socket
    /// open, so an adversarial event's rejection frame is observable here.
    pub async fn submit_recv_error(
        &mut self,
        event: &Event,
        dur: Duration,
    ) -> Result<Option<(u32, String)>> {
        use xgen_core::transport::connection::Inbound;
        use xgen_core::wire::types::TransportMessage;
        self.conn
            .send_event(event)
            .await
            .context("WireActor send_event")?;
        let reply = match tokio::time::timeout(dur, self.conn.recv()).await {
            Ok(Ok(Inbound::Transport(TransportMessage::Error {
                error_code,
                error_string,
                ..
            }))) => Some((error_code, error_string)),
            _ => None,
        };
        Ok(reply)
    }

    /// Build + sign + submit a `state.space_create`; returns the new `space_id`
    /// (= the create event's id).
    pub async fn create_space(&mut self, name: &str) -> Result<String> {
        let ev = sign_event(
            build_space_create_event(&self.key, name, None, 1, &self.node_url, None, false),
            &self.key,
        );
        let id = event_id_of(&ev)?;
        self.submit(&ev).await?;
        Ok(id)
    }

    /// Build + sign + submit a `state.room_create`; returns the new `room_id`.
    pub async fn create_room(&mut self, space_id: &str, name: &str) -> Result<String> {
        let ev = sign_event(
            build_room_create_event(&self.key, space_id, name, None),
            &self.key,
        );
        let id = event_id_of(&ev)?;
        self.submit(&ev).await?;
        Ok(id)
    }

    /// Identity id derived from a key without connecting (utility).
    pub fn id_for(key: &SigningKey) -> String {
        identity_id_from_key(key)
    }
}

fn event_id_of(ev: &Event) -> Result<String> {
    ev.event_id
        .as_ref()
        .map(|e| e.as_str().to_string())
        .ok_or_else(|| anyhow!("signed event unexpectedly has no event_id"))
}
