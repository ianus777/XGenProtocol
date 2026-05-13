// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// WebSocket server — accept incoming connections (spec 3.3.1).
// Phase 1 / Local Node mode: ws:// only, localhost only, no TLS.

use std::net::SocketAddr;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;

use xgen_core::transport::connection::{Connection, TransportError};

pub struct Server {
    listener: TcpListener,
}

impl Server {
    /// Bind a WebSocket server on `addr`. Use `0.0.0.0:0` or `127.0.0.1:0` to
    /// let the OS assign a free port (useful in tests).
    pub async fn bind(addr: SocketAddr) -> Result<Self, TransportError> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self { listener })
    }

    /// The address this server is actually listening on (resolves the OS-assigned port).
    pub fn local_addr(&self) -> SocketAddr {
        self.listener.local_addr().expect("listener has local address")
    }

    /// Accept the next incoming TCP connection and upgrade it to a WebSocket.
    /// Returns a raw (unauthenticated) `Connection`. The caller must run
    /// `server_authenticate()` before treating it as ACTIVE.
    pub async fn accept(&mut self) -> Result<Connection<TcpStream>, TransportError> {
        let (tcp, _peer_addr) = self.listener.accept().await?;
        let ws = accept_async(tcp).await.map_err(TransportError::WebSocket)?;
        Ok(Connection::new(ws))
    }
}
