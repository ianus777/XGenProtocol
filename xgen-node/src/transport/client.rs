// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// WebSocket client — connect outbound to a peer Node (spec 3.3.1).
// Phase 1 / Local Node mode: ws:// only, no TLS.

use std::net::SocketAddr;

use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream};

use super::connection::{Connection, TransportError};

/// Connect to a Node WebSocket endpoint by socket address.
/// Returns a raw (unauthenticated) `Connection`. The caller must run
/// `client_authenticate()` before treating it as ACTIVE.
pub async fn connect(
    addr: SocketAddr,
) -> Result<Connection<MaybeTlsStream<TcpStream>>, TransportError> {
    let url = format!("ws://{}/", addr);
    let (ws, _response) = connect_async(url)
        .await
        .map_err(TransportError::WebSocket)?;
    Ok(Connection::new(ws))
}

/// Connect to a Node WebSocket endpoint by URL string (ws:// or wss://).
/// Returns a raw (unauthenticated) `Connection`. The caller must run
/// `client_authenticate()` before treating it as ACTIVE.
pub async fn connect_url(url: &str) -> Result<Connection<MaybeTlsStream<TcpStream>>, TransportError> {
    let (ws, _response) = connect_async(url)
        .await
        .map_err(TransportError::WebSocket)?;
    Ok(Connection::new(ws))
}
