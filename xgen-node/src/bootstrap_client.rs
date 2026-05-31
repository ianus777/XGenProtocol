// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Bootstrap client send-path (bootstrap-client D-071 arc, C2 — the
//! load-bearing commit). Drives the `BootstrapMessage` wire types over the
//! normal framed transport (BC-D3 — **NOT** HTTP; the only HTTP in bootstrap
//! is the directory-fetch, D-051, which is OUT of A3 scope).
//!
//! **Pin 1 (J-192 lock):** reuse the existing `Connection` primitives —
//! `connect_url` + `send_bootstrap` + `recv` — with **no transport
//! challenge-response auth**. Spec §3.14.3 says the Bootstrap Node verifies
//! the *message* signature; there is no bootstrap transport handshake (unlike
//! federation, which mandates mutual transport auth). The orchestration lives
//! here in xgen-node; the pure sign/verify logic is in
//! `xgen_core::bootstrap::signing` (xgen-core stays transport-pure).
//!
//! **Pin 2 (J-192 lock):** the `register_ack` / `keepalive_ack` signature is
//! verified against the operator-supplied / stored `bootstrap_id` (and the
//! ack's `node_id` field is asserted to match it) — never the ack's
//! self-declared id.
//!
//! **Pin 3 (J-192 lock):** the C4 keepalive scheduler (a separate
//! `bootstrap_keepalive.rs`, reconnect.rs shape) reuses these functions —
//! `keepalive_bootstrap` is shaped to return the refreshed TTL the scheduler
//! stores.
//!
//! No verb wiring this commit (verbs = C3). These functions are exercised by
//! the integration tests against an in-process stub bootstrap responder.

use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};

use xgen_common::xgid::NodeXgid;
use xgen_core::bootstrap::registration_store::{BootstrapRegistration, BootstrapSelfInfo};
use xgen_core::bootstrap::signing::{sign_bootstrap, verify_bootstrap_signed, BootstrapSignError};
use xgen_core::transport::client::connect_url;
use xgen_core::transport::connection::{Connection, Inbound};
use xgen_core::wire::types::BootstrapMessage;

const PROTOCOL_VERSION: &str = "0.1";

/// Bootstrap directory entry TTL (spec §3.14.3, WD-25): 7 days. A successful
/// `register` / `keepalive` resets the entry's expiry to `now + this`.
const BOOTSTRAP_TTL_SECONDS: i64 = 7 * 24 * 60 * 60;

/// Max seconds to wait for a bootstrap node's ack (sibling to the federation
/// handshake's 15 s `WAIT_TIMEOUT_SECS`).
const ACK_TIMEOUT_SECS: u64 = 15;

#[derive(Debug, Error)]
pub enum BootstrapClientError {
    #[error("connect to bootstrap node failed: {0}")]
    Connect(String),
    #[error("send to bootstrap node failed: {0}")]
    Send(String),
    #[error("receive from bootstrap node failed: {0}")]
    Recv(String),
    #[error("timed out waiting for {0} after {ACK_TIMEOUT_SECS}s")]
    Timeout(&'static str),
    #[error("unexpected reply from bootstrap node (expected {expected})")]
    UnexpectedReply { expected: &'static str },
    #[error("ack verification failed: {0}")]
    AckVerify(#[from] BootstrapSignError),
}

fn now_rfc(now: chrono::DateTime<Utc>) -> String {
    now.to_rfc3339_opts(SecondsFormat::Millis, true)
}

// ── Pure message builders (testable without a socket) ──────────────────────────

/// Build an unsigned `bootstrap.register` frame advertising this Node's
/// self-info. `self_node_id` is the registrant's own id (derived from the Node
/// keypair). Sign with `sign_bootstrap` before sending.
pub fn build_register(
    self_node_id: &NodeXgid,
    self_info: &BootstrapSelfInfo,
    timestamp: String,
) -> BootstrapMessage {
    BootstrapMessage::Register {
        protocol_version: PROTOCOL_VERSION.to_string(),
        node_id: self_node_id.as_str().to_string(),
        endpoint: self_info.endpoint.clone(),
        region: self_info.region.clone(),
        capabilities: self_info.capabilities.clone(),
        timestamp,
        signature: None,
    }
}

/// Build an unsigned `bootstrap.keepalive` frame (refreshes this Node's
/// directory entry before TTL expiry, spec §3.14.7).
pub fn build_keepalive(self_node_id: &NodeXgid, timestamp: String) -> BootstrapMessage {
    BootstrapMessage::Keepalive {
        protocol_version: PROTOCOL_VERSION.to_string(),
        node_id: self_node_id.as_str().to_string(),
        timestamp,
        signature: None,
    }
}

/// Build an unsigned `bootstrap.deregister` frame (explicit removal, §3.14.7).
pub fn build_deregister(self_node_id: &NodeXgid, timestamp: String) -> BootstrapMessage {
    BootstrapMessage::Deregister {
        protocol_version: PROTOCOL_VERSION.to_string(),
        node_id: self_node_id.as_str().to_string(),
        timestamp,
        signature: None,
    }
}

/// The RFC 3339 expiry stamp `now + 7 days` to record after a successful
/// register/keepalive. Exposed so the C4 scheduler stores a consistent value.
pub fn ttl_expiry(now: chrono::DateTime<Utc>) -> String {
    now_rfc(now + chrono::Duration::seconds(BOOTSTRAP_TTL_SECONDS))
}

// ── Socket exchanges (thin) ─────────────────────────────────────────────────────

/// Register this Node with a Bootstrap Node and return the resulting
/// `BootstrapRegistration` record (to be stored by the C3 verb). Connects,
/// sends a signed `Register`, receives + verifies the `RegisterAck` against
/// `bootstrap_id` (Pin 2), and records the ack's `directory_url` + a fresh TTL.
pub async fn register_with_bootstrap(
    url: &str,
    bootstrap_id: &NodeXgid,
    self_node_id: &NodeXgid,
    self_info: &BootstrapSelfInfo,
    node_keypair: &SigningKey,
) -> Result<BootstrapRegistration, BootstrapClientError> {
    let now = Utc::now();
    let registered_at = now_rfc(now);
    let msg = sign_bootstrap(
        build_register(self_node_id, self_info, registered_at.clone()),
        node_keypair,
    );

    let mut conn = connect_url(url)
        .await
        .map_err(|e| BootstrapClientError::Connect(e.to_string()))?;
    conn.send_bootstrap(&msg)
        .await
        .map_err(|e| BootstrapClientError::Send(e.to_string()))?;

    let ack = recv_bootstrap(&mut conn, "register_ack").await?;
    if !matches!(ack, BootstrapMessage::RegisterAck { .. }) {
        return Err(BootstrapClientError::UnexpectedReply { expected: "register_ack" });
    }
    verify_bootstrap_signed(&ack, bootstrap_id.as_str())?;

    let directory_url = match ack {
        BootstrapMessage::RegisterAck { directory_url, .. } => directory_url,
        _ => unreachable!("matched RegisterAck above"),
    };

    Ok(BootstrapRegistration {
        bootstrap_id: bootstrap_id.clone(),
        url: url.to_string(),
        directory_url,
        registered_at,
        expires_at: Some(ttl_expiry(now)),
    })
}

/// Refresh this Node's directory entry at a Bootstrap Node. Connects, sends a
/// signed `Keepalive`, receives + verifies the `KeepaliveAck` against
/// `bootstrap_id` (Pin 2), and returns the refreshed TTL expiry the C4
/// scheduler stores on the registration.
pub async fn keepalive_bootstrap(
    url: &str,
    bootstrap_id: &NodeXgid,
    self_node_id: &NodeXgid,
    node_keypair: &SigningKey,
) -> Result<String, BootstrapClientError> {
    let now = Utc::now();
    let msg = sign_bootstrap(build_keepalive(self_node_id, now_rfc(now)), node_keypair);

    let mut conn = connect_url(url)
        .await
        .map_err(|e| BootstrapClientError::Connect(e.to_string()))?;
    conn.send_bootstrap(&msg)
        .await
        .map_err(|e| BootstrapClientError::Send(e.to_string()))?;

    let ack = recv_bootstrap(&mut conn, "keepalive_ack").await?;
    if !matches!(ack, BootstrapMessage::KeepaliveAck { .. }) {
        return Err(BootstrapClientError::UnexpectedReply { expected: "keepalive_ack" });
    }
    verify_bootstrap_signed(&ack, bootstrap_id.as_str())?;

    Ok(ttl_expiry(now))
}

/// Explicitly remove this Node from a Bootstrap Node's directory (§3.14.7).
/// Fire-and-forget: the spec defines no `deregister_ack`, so this connects,
/// sends a signed `Deregister`, and returns once it is on the wire.
pub async fn deregister_from_bootstrap(
    url: &str,
    self_node_id: &NodeXgid,
    node_keypair: &SigningKey,
) -> Result<(), BootstrapClientError> {
    let now = Utc::now();
    let msg = sign_bootstrap(build_deregister(self_node_id, now_rfc(now)), node_keypair);

    let mut conn = connect_url(url)
        .await
        .map_err(|e| BootstrapClientError::Connect(e.to_string()))?;
    conn.send_bootstrap(&msg)
        .await
        .map_err(|e| BootstrapClientError::Send(e.to_string()))?;
    Ok(())
}

/// Receive the next frame and require it to be a `BootstrapMessage`. Generic
/// over the stream so it is exercisable against any in-process transport.
async fn recv_bootstrap<S>(
    conn: &mut Connection<S>,
    what: &'static str,
) -> Result<BootstrapMessage, BootstrapClientError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let inbound = tokio::time::timeout(Duration::from_secs(ACK_TIMEOUT_SECS), conn.recv())
        .await
        .map_err(|_| BootstrapClientError::Timeout(what))?
        .map_err(|e| BootstrapClientError::Recv(e.to_string()))?;
    match inbound {
        Inbound::Bootstrap(m) => Ok(m),
        _ => Err(BootstrapClientError::UnexpectedReply { expected: what }),
    }
}

// ── Tests (pure builders) ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use xgen_common::xgid::Xgid;

    fn ndx(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }

    #[test]
    fn build_register_carries_self_info() {
        let info = BootstrapSelfInfo {
            endpoint: "wss://self.example.com/xgen".to_string(),
            region: "EU".to_string(),
            capabilities: vec!["xgen.federation".to_string()],
            auth_tiers_served: vec![2, 3],
        };
        let msg = build_register(&ndx("xgen://pubkey/ed25519:self"), &info, "2026-05-31T12:00:00.000Z".to_string());
        match msg {
            BootstrapMessage::Register { node_id, endpoint, region, capabilities, signature, .. } => {
                assert_eq!(node_id, "xgen://pubkey/ed25519:self");
                assert_eq!(endpoint, "wss://self.example.com/xgen");
                assert_eq!(region, "EU");
                assert_eq!(capabilities, vec!["xgen.federation".to_string()]);
                // tiers are NOT on the wire frame (Checkpoint #1(d), Option A).
                assert!(signature.is_none());
            }
            _ => panic!("expected Register"),
        }
    }

    #[test]
    fn build_keepalive_and_deregister_shapes() {
        let id = ndx("xgen://pubkey/ed25519:self");
        assert!(matches!(
            build_keepalive(&id, "t".to_string()),
            BootstrapMessage::Keepalive { .. }
        ));
        assert!(matches!(
            build_deregister(&id, "t".to_string()),
            BootstrapMessage::Deregister { .. }
        ));
    }
}
