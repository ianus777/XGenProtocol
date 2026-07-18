// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! The shared client resident spine (M-RP6.6, D1).
//!
//! Both the headless `--service` resident (`service::run_ws_loop`) and the Tauri
//! desktop resident (`desktop::run_startup`) drive the SAME connect → authenticate
//! → drain cycle from here — not two forks (D-056 shared command layer). The one
//! thing that differs between them is what happens on each lifecycle transition:
//! the desktop shell emits `xgen-client-state-changed` for the UI; the service
//! stays headless. That difference is injected as a **lifecycle sink** closure
//! (`FnMut(ClientLifecycleState)`), so the spine itself is UI-agnostic and lives
//! in the client crate — GPL core `Connection` is never touched (D3).
//!
//! Leg A (this) ships the single-session spine + real lifecycle. Leg B wraps it
//! in a reconnect loop with backoff (see `run_resident` / `Backoff`, added there).
//! Leg C threads byte/RTT counters through the drain via a stream-layer
//! `CountingStream` (the §0 Path-A interposer) — the `Inbound::Pong` arm below is
//! the reserved RTT seam.

use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use std::time::{Duration, Instant};

use anyhow::Context;
use ed25519_dalek::SigningKey;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::sync::watch;

use xgen_core::transport::connection::{Connection, Inbound};

use crate::lifecycle::ClientLifecycleState;

// ── Traffic accounting (Leg C, §0 Path A) ─────────────────────────────────────
//
// Byte counts are observed at the STREAM layer by a client-crate `CountingStream`
// interposed below the WebSocket handshake — GPL core `Connection` is never
// touched (D3, revised note). Honest scope: ALL socket bytes this resident
// session, the auth handshake included (the wrap happens before authenticate).
// Counts are cumulative across reconnects (the same `TrafficCounters` is shared
// by every session's stream). RTT is measured resident-side: the drain pings on
// an interval and times the matching `Inbound::Pong` — no core change.

/// How often the resident pings to measure RTT (and, as a side effect, bounds
/// how long a silently-dead peer goes undetected — a passive drain with no
/// keepalive only notices an abrupt kill when the OS finally delivers the RST).
const PING_INTERVAL: Duration = Duration::from_secs(10);

/// `u64::MAX` in the `rtt_ms` atomic means "no pong observed yet" → `None` on the
/// wire (absent-not-zero, D4). A real 0 ms RTT is not representable, which is fine
/// — sub-millisecond loopback rounds to 0 and reads as a real value; only the
/// sentinel maps to absent.
const RTT_NONE: u64 = u64::MAX;

/// Shared, cheaply-cloneable traffic counters. Every `CountingStream` and the
/// `get_conn_stats` command hold a clone of the SAME atomics (the `Pacing`
/// managed-state shape). `Relaxed` ordering is correct: these are independent
/// monotonic counters read for display, not a synchronisation signal.
#[derive(Clone)]
pub struct TrafficCounters {
    bytes_in: Arc<AtomicU64>,
    bytes_out: Arc<AtomicU64>,
    rtt_ms: Arc<AtomicU64>,
}

impl Default for TrafficCounters {
    fn default() -> Self {
        Self {
            bytes_in: Arc::new(AtomicU64::new(0)),
            bytes_out: Arc::new(AtomicU64::new(0)),
            rtt_ms: Arc::new(AtomicU64::new(RTT_NONE)),
        }
    }
}

impl TrafficCounters {
    fn add_in(&self, n: u64) {
        self.bytes_in.fetch_add(n, Ordering::Relaxed);
    }
    fn add_out(&self, n: u64) {
        self.bytes_out.fetch_add(n, Ordering::Relaxed);
    }
    fn set_rtt(&self, ms: u64) {
        // Never store the sentinel as a real value (a genuine u64::MAX ms RTT is
        // ~584 million years — safe to clamp).
        self.rtt_ms.store(ms.min(RTT_NONE - 1), Ordering::Relaxed);
    }
    /// A serialisable snapshot for `get_conn_stats`. Field names are snake_case
    /// verbatim — the frontend `ConnTraffic` reads them with no mapping layer.
    pub fn snapshot(&self) -> ConnTraffic {
        let rtt = self.rtt_ms.load(Ordering::Relaxed);
        ConnTraffic {
            bytes_in: self.bytes_in.load(Ordering::Relaxed),
            bytes_out: self.bytes_out.load(Ordering::Relaxed),
            rtt_ms: (rtt != RTT_NONE).then_some(rtt),
        }
    }
}

/// The `get_conn_stats` return shape. `rtt_ms: None` renders ABSENT (D4/N-060),
/// never a fabricated 0. Mirrored verbatim by `selfState.ConnTraffic` (snake_case).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnTraffic {
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub rtt_ms: Option<u64>,
}

/// A byte-counting `AsyncRead`/`AsyncWrite` wrapper. It delegates every read/write
/// to the inner stream and tallies the bytes into the shared counters. `S: Unpin`
/// (a `TcpStream` is), so it is itself `Unpin` and needs no pin projection.
pub struct CountingStream<S> {
    inner: S,
    counters: TrafficCounters,
}

impl<S> CountingStream<S> {
    pub fn new(inner: S, counters: TrafficCounters) -> Self {
        Self { inner, counters }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for CountingStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        let before = buf.filled().len();
        let r = Pin::new(&mut this.inner).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = &r {
            let n = buf.filled().len().saturating_sub(before);
            if n > 0 {
                this.counters.add_in(n as u64);
            }
        }
        r
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for CountingStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        let r = Pin::new(&mut this.inner).poll_write(cx, buf);
        if let Poll::Ready(Ok(n)) = &r {
            this.counters.add_out(*n as u64);
        }
        r
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}

/// The `ws://host:port/path` authority (`host:port`) for the raw TCP dial. Phase 1
/// is ws:// only (no TLS) — a `wss://` URL would dial TCP here and then fail the
/// plaintext handshake against a TLS endpoint; counting over TLS is a documented
/// future concern (the whole transport's wss:// story, per the spec).
fn ws_authority(url: &str) -> anyhow::Result<&str> {
    let rest = url
        .strip_prefix("ws://")
        .or_else(|| url.strip_prefix("wss://"))
        .with_context(|| format!("not a ws(s) URL: {url}"))?;
    Ok(rest.split('/').next().unwrap_or(rest))
}

/// Connect to a Node WS endpoint with a `CountingStream` interposed below the WS
/// handshake (§0 Path A). Mirrors `xgen_core::transport::client::connect_url`'s
/// three lines but hand-dials so the counter sits on the socket — GPL core
/// `connect_url`/`Connection` stay untouched (D3). Returns a `Connection` over the
/// counted stream; the same `Connection` API (`client_authenticate`/`recv`/`ping`/
/// `goodbye`) drives it unchanged.
async fn connect_counted(
    url: &str,
    counters: &TrafficCounters,
) -> anyhow::Result<Connection<CountingStream<TcpStream>>> {
    let authority = ws_authority(url)?;
    let tcp = TcpStream::connect(authority)
        .await
        .with_context(|| format!("TCP connect to {authority}"))?;
    let counted = CountingStream::new(tcp, counters.clone());
    let (ws, _resp) = tokio_tungstenite::client_async(url, counted)
        .await
        .context("WS client handshake")?;
    Ok(Connection::new(ws))
}

/// How long to wait for the initial WS connect before giving up on one attempt.
/// Matches the pre-M-RP6.6 `service::run_ws_loop` timeout (10 s); the desktop
/// scaffold used 2 s — the longer value is kept so a briefly-slow node does not
/// read as DISCONNECTED on the first dial.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// The outcome of one connect → authenticate → drain session. The reconnect
/// wrapper (Leg B) branches on this: `Disconnected` means "we had a live session
/// that dropped" (reset the backoff, reconnect); `ConnectFailed` / `AuthFailed`
/// mean "we never reached READY" (grow the backoff); `ShutdownRequested` means
/// "stop the resident entirely" (break the loop, no reconnect).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEnd {
    /// The watch channel signalled shutdown mid-session; `goodbye` was attempted.
    ShutdownRequested,
    /// The socket dropped or the peer closed (recv `Err` / `Inbound::Closed`)
    /// AFTER we reached READY.
    Disconnected,
    /// The dial failed or timed out; READY was never reached.
    ConnectFailed,
    /// Authentication failed; READY was never reached.
    AuthFailed,
}

impl SessionEnd {
    /// Whether this session reached READY before ending. The reconnect wrapper
    /// resets the backoff only after a session that actually connected — a
    /// flapping node should not reset the schedule on every failed dial.
    pub fn reached_ready(self) -> bool {
        matches!(self, SessionEnd::Disconnected)
    }
}

/// Run ONE connect → authenticate → drain cycle, driving `sink` at each real
/// lifecycle transition. Returns when the socket drops, auth fails, or the
/// `shutdown_rx` watch fires. Pure transport + lifecycle — the caller resolves
/// the node URL and loads the signing key (both callers already have app-module
/// helpers for that), so this fn does no filesystem work and stays testable.
///
/// Lifecycle emitted (all from REAL outcomes — no `sleep(150ms)` placeholder):
///   - `Connecting`     — before the dial.
///   - `Authenticating` — socket up, auth handshake starting.
///   - `Ready`          — auth succeeded.
///
/// **The TERMINAL state is the CALLER's, not this fn's.** `run_session` emits
/// only the live progression above; on ANY end (connect fail, auth fail, or a
/// mid-session drop) it returns the `SessionEnd` and emits nothing further. The
/// caller then decides: Leg A (single session) emits `Disconnected`; Leg B
/// (reconnect) emits `Reconnecting` and retries. Emitting the terminal state
/// here was the original bug — the drain-drop arms returned `Disconnected`
/// without emitting it, so a READY session that dropped went silent while the
/// connect/auth-fail arms (which DID emit) diverged. One owner, no divergence.
///
/// The three `Degraded*` states are intentionally NOT emitted: a single session
/// has no source for node/federation degradation, and a failed (re)auth surfaces
/// as `AuthFailed` → the caller's terminal state, not a sticky degraded state.
/// Emitting one with no real trigger is an unfed branch (N-091). If a real
/// degradation source ever lands, wire it here.
pub async fn run_session<F>(
    node: &str,
    signing_key: &SigningKey,
    traffic: &TrafficCounters,
    shutdown_rx: &mut watch::Receiver<bool>,
    sink: &mut F,
) -> SessionEnd
where
    F: FnMut(ClientLifecycleState) + Send,
{
    sink(ClientLifecycleState::Connecting);

    let mut conn = match tokio::time::timeout(CONNECT_TIMEOUT, connect_counted(node, traffic)).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            tracing::warn!(home_node = %node, reason = %e, "resident: WS connect failed");
            return SessionEnd::ConnectFailed;
        }
        Err(_) => {
            tracing::warn!(home_node = %node, "resident: WS connect timed out");
            return SessionEnd::ConnectFailed;
        }
    };

    sink(ClientLifecycleState::Authenticating);

    match conn.client_authenticate(signing_key).await {
        Ok(out) => {
            tracing::info!(identity_id = %out.identity_id, connected_node = %node, "resident: authenticated");
        }
        Err(e) => {
            tracing::warn!(reason = %e, "resident: WS authentication failed");
            return SessionEnd::AuthFailed;
        }
    }

    sink(ClientLifecycleState::Ready);

    // Drain inbound until the socket drops or shutdown is requested. Events are
    // discarded at this layer — real-time ingest into R5 fan-out is the deferred
    // leg (gated on R5 + M-RP6.3). A periodic ping measures RTT (and bounds
    // dead-peer detection): the interval arm sets `want_ping`, and the ping is
    // issued at the top of the next iteration — OUTSIDE the select — so it never
    // races the `conn.recv()` borrow (both need `&mut conn`).
    let mut ping_interval = tokio::time::interval(PING_INTERVAL);
    ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    ping_interval.tick().await; // consume the immediate first tick
    let mut pending_ping: Option<Instant> = None;
    let mut want_ping = false;

    let end = loop {
        if want_ping {
            want_ping = false;
            match conn.ping().await {
                Ok(()) => pending_ping = Some(Instant::now()),
                Err(e) => {
                    tracing::warn!(reason = %e, "resident: ping failed — connection lost");
                    break SessionEnd::Disconnected;
                }
            }
        }
        tokio::select! {
            recv = conn.recv() => match recv {
                Ok(Inbound::Closed) => break SessionEnd::Disconnected,
                Ok(Inbound::Pong(_data)) => {
                    if let Some(sent) = pending_ping.take() {
                        traffic.set_rtt(sent.elapsed().as_millis() as u64);
                    }
                }
                Ok(_inbound) => {
                    // Deferred: dispatch to R5 fan-out (own milestone).
                }
                Err(e) => {
                    tracing::warn!(reason = %e, "resident: recv error — connection lost");
                    break SessionEnd::Disconnected;
                }
            },
            _ = ping_interval.tick() => {
                want_ping = true;
            }
            _ = shutdown_rx.changed() => {
                break SessionEnd::ShutdownRequested;
            }
        }
    };

    // Best-effort graceful close. On a dropped socket this errors harmlessly; on
    // the shutdown path it races the caller's exit (the desktop `quit` command
    // calls `app.exit(0)` right after signalling), so it is not a hard guarantee.
    let _ = conn.goodbye("client_disconnect").await;

    end
}

// ── Reconnect / backoff (Leg B) ───────────────────────────────────────────────

/// Capped-exponential reconnect backoff steps: `1 << attempt` seconds, capped.
const BACKOFF_MAX_SHIFT: u32 = 5; // 1 << 5 = 32, then capped to BACKOFF_CAP_SECS
const BACKOFF_CAP_SECS: u64 = 30;

/// Pure reconnect backoff schedule (unit-tested; no clock, no socket). Produces
/// 1, 2, 4, 8, 16 s then a 30 s cap — fast to recover a briefly-flapping node,
/// calm for a long-down one. Reset after any session that reached READY so a
/// healthy connection that later drops reconnects promptly rather than inheriting
/// a grown delay.
#[derive(Debug, Default)]
pub struct Backoff {
    attempt: u32,
}

impl Backoff {
    /// The next delay, advancing the schedule. Saturating + shift-clamped so an
    /// arbitrarily long down-time can never overflow.
    pub fn next_delay(&mut self) -> Duration {
        let exp = 1u64 << self.attempt.min(BACKOFF_MAX_SHIFT);
        let secs = exp.min(BACKOFF_CAP_SECS);
        self.attempt = self.attempt.saturating_add(1);
        Duration::from_secs(secs)
    }

    /// Reset to the start of the schedule (after a session reached READY).
    pub fn reset(&mut self) {
        self.attempt = 0;
    }
}

/// The long-lived desktop resident: run `run_session` in a reconnect loop. On any
/// non-shutdown end, emit `Reconnecting`, back off, and retry; reset the backoff
/// after a session that reached READY; return on a requested shutdown. This is
/// the ONLY emitter of `Reconnecting` — with it, a dropped connection
/// re-establishes without an app restart and the UI never sticks at a terminal
/// `Disconnected` (that was the Leg-A single-session terminal; Leg B supersedes
/// it). The `sleep` races `shutdown_rx` so a quit during a long backoff wait exits
/// promptly rather than blocking on the full delay.
pub async fn run_resident<F>(
    node: &str,
    signing_key: &SigningKey,
    traffic: &TrafficCounters,
    shutdown_rx: &mut watch::Receiver<bool>,
    sink: &mut F,
) where
    F: FnMut(ClientLifecycleState) + Send,
{
    let mut backoff = Backoff::default();
    loop {
        let outcome = run_session(node, signing_key, traffic, shutdown_rx, sink).await;
        if outcome == SessionEnd::ShutdownRequested || *shutdown_rx.borrow() {
            break;
        }
        if outcome.reached_ready() {
            backoff.reset();
        }
        sink(ClientLifecycleState::Reconnecting);
        let delay = backoff.next_delay();
        tokio::select! {
            _ = tokio::time::sleep(delay) => {}
            _ = shutdown_rx.changed() => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reached_ready_only_on_disconnected() {
        assert!(SessionEnd::Disconnected.reached_ready());
        assert!(!SessionEnd::ConnectFailed.reached_ready());
        assert!(!SessionEnd::AuthFailed.reached_ready());
        assert!(!SessionEnd::ShutdownRequested.reached_ready());
    }

    #[test]
    fn backoff_schedule_is_capped_exponential() {
        let mut b = Backoff::default();
        let secs: Vec<u64> = (0..7).map(|_| b.next_delay().as_secs()).collect();
        assert_eq!(secs, vec![1, 2, 4, 8, 16, 30, 30]);
    }

    #[test]
    fn backoff_reset_restarts_schedule() {
        let mut b = Backoff::default();
        b.next_delay();
        b.next_delay();
        b.next_delay();
        b.reset();
        assert_eq!(b.next_delay().as_secs(), 1);
    }

    #[test]
    fn backoff_never_decreases() {
        let mut b = Backoff::default();
        let mut prev = 0;
        for _ in 0..20 {
            let d = b.next_delay().as_secs();
            assert!(d >= prev);
            prev = d;
        }
    }

    #[tokio::test]
    async fn counting_stream_tallies_and_snapshot_maps_sentinel() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let counters = TrafficCounters::default();
        // Absent-not-zero (D4): no pong yet → rtt_ms is None, never a fabricated 0.
        assert_eq!(counters.snapshot().rtt_ms, None);

        let (a, mut other) = tokio::io::duplex(64);
        let mut counted = CountingStream::new(a, counters.clone());

        // 5 bytes out through the counted stream; 3 bytes back in.
        counted.write_all(&[1, 2, 3, 4, 5]).await.unwrap();
        counted.flush().await.unwrap();
        other.write_all(&[9, 9, 9]).await.unwrap();
        other.flush().await.unwrap();
        let mut buf = [0u8; 3];
        counted.read_exact(&mut buf).await.unwrap();

        let snap = counters.snapshot();
        assert_eq!(snap.bytes_out, 5, "outbound bytes tallied");
        assert_eq!(snap.bytes_in, 3, "inbound bytes tallied");

        // A real RTT reads back as Some.
        counters.set_rtt(42);
        assert_eq!(counters.snapshot().rtt_ms, Some(42));
    }
}
