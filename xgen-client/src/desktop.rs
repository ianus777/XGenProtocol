// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Tauri desktop shell for xgen-client (D-062, D-063). Migrated into the
//! library crate by M1 Phase 2a so the single `xgen-client` binary can
//! initialise the UI when launched without `--service`.
//!
//! 2a status: runs first-run detection, pipe server, and the existing
//! auto-connect lifecycle scaffold (Initialising → Setup OR Connecting →
//! Authenticating → Ready/Disconnected). The full long-lived client resident
//! (sustained WS to home Node, history sync, real-time fan-out) is wired in
//! 2b / M3.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow, WindowEvent};

use crate::app;
use crate::lifecycle::{make_state_event, ClientLifecycleState, ClientStateEvent};
use crate::pacing::{PacingManager, PacingState};
use crate::temperature::TemperatureUpdate;
use xgen_common::event_trace::{write_session_footer, write_session_header, ExitReason};
use xgen_common::xgid::{RoomXgid, SpaceXgid, Xgid};

// ── Shared state ───────────────────────────────────────────────────────────────

struct CurrentState(Arc<Mutex<ClientStateEvent>>);

/// Holds the watch sender so the quit command can signal the pipe server.
struct PipeShutdown(tokio::sync::watch::Sender<bool>);

/// Outbound pacing queue manager (Ch6 §6.14). The Tauri webview reads the
/// current snapshot via `get_pacing_state`.
struct Pacing(Arc<Mutex<PacingManager>>);

/// Live connection traffic counters (M-RP6.6 Leg C). Shares the SAME atomics as
/// the resident's `CountingStream` + RTT tracker; the webview reads a snapshot via
/// `get_conn_stats`. Mirrors the `Pacing` managed-state shape.
struct TrafficStats(crate::resident::TrafficCounters);

/// Live resident reconnect status (M-RP6.3 Leg A). Shares the SAME atomics as
/// `run_resident`; the webview reads a snapshot via `get_resident_status`. The
/// `TrafficStats` managed-state shape — one pattern, not a new mechanism (D-067).
struct ResidentStatusState(crate::resident::ResidentStatus);

/// The resume channel (M-RP6.3 Leg A, D4). A generation counter the resident
/// parks on once the retry cap is reached; `resume_resident` bumps it. Sibling of
/// `PipeShutdown` — the same `watch::Sender` managed-state shape, so the resident
/// can `select!` on it exactly the way it already does on shutdown.
struct ResidentResume(tokio::sync::watch::Sender<u64>);

/// The outbound message queue (M-RP6.3 Leg B). `send_message` pushes onto this;
/// the resident's drain pops, writes and correlates. Bounded, so a UI that
/// somehow floods it gets backpressure rather than unbounded memory.
///
/// Every way a message can fail to reach the wire surfaces as an honest `failed`,
/// never as a silent drop (D6) — but that took two mechanisms, not one, and Leg B
/// shipped only the first:
/// - a **full** queue fails at `try_send`, immediately;
/// - an **undrained** queue (nothing polls this receiver between sessions) fails
///   at `send_message`'s bounded wait, `SEND_QUEUE_TIMEOUT`;
/// - a **written** send is resolved by the drain — ack, sweep, or teardown;
/// - an **abandoned** request, popped after that wait expired, is discarded and
///   logged rather than written.
struct Outbound(tokio::sync::mpsc::Sender<crate::resident::OutboundRequest>);

/// The resident's DAG frontier (M-RP6.3 Leg B, B-5). Shared with the drain, which
/// updates it from live ingest and from our own sends; read by `send_message` to
/// decide whether a control-plane re-anchor is needed first.
struct Tips(crate::resident::TipTracker);

/// Resolved path to `xgen-client_config.toml` for this instance (M-RP4.2).
/// Held as managed state so `get_substitutions` reads the live config without
/// re-deriving the data-dir — which depends on the `--instance` label resolved
/// at launch. Mirrors the `CurrentState`/`Pacing` managed-state pattern.
struct ConfigPath(PathBuf);

/// Resolved Tier-1 data directory for this instance (M-RP6.1e-C2). Held as
/// managed state so `get_about_info` reports the real path without re-deriving
/// it as `config_path.parent()` — a derivation that would silently break if the
/// config filename ever moved. Sibling of `ConfigPath`.
struct DataDir(PathBuf);

/// Serialises concurrent `fill_space_records` invocations (M-RP-MEMBERS Leg A,
/// runbook §3). Two fills overlapping (the user clicking through Spaces quickly)
/// would each `load` the book, each `save`, and the loser's resolved records
/// would be silently discarded — so rows sit as pubkey stubs longer than they
/// should, with an invisible cause. Held across load → fill → save. A
/// `tokio::Mutex` (not `std`) because the guard is held across `.await`.
struct FillLock(tokio::sync::Mutex<()>);

/// The UI-state store filename (M-RP6.1k). Sibling of `xgen-client_config.toml` in the data dir;
/// resolved via the managed `DataDir` so the path is never re-derived (D-114). Deliberately NOT
/// config — persistent, and NOT wiped by D-101 clean-slate-on-start.
const UI_STATE_FILE: &str = "xgen-client_uistate.json";

fn emit_state(app: &AppHandle, state: ClientLifecycleState) {
    let canonical = state.as_canonical();
    tracing::info!(lifecycle_state = canonical, "lifecycle transition");
    let payload = make_state_event(state);
    if let Ok(mut stored) = app.state::<CurrentState>().0.lock() {
        *stored = payload.clone();
    }
    let _ = app.emit("xgen-client-state-changed", &payload);
}

// ── Window geometry save/restore (M-RP6.1k, Leg C — D-114 typed carve-out, D-115) ───────────────
//
// Only Rust can read a monitor work area or apply a window rect before the webview is shown, so
// geometry is the one part of the UI-state store Rust owns. Physical px throughout (D-115) — no DPR
// conversion. The pure logic (parse / read-modify-write merge / clamp decision) lives in `app.rs` and
// is unit-tested; this half is the Tauri glue that queries monitors + the live window.

/// Throttle for move/resize saves: at most one disk write per ~500ms so a drag doesn't hammer the
/// file, while a crash loses at most the last ~500ms. `CloseRequested` bypasses this (authoritative).
static LAST_GEOM_WRITE: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);

fn geometry_throttle_ok() -> bool {
    let mut last = LAST_GEOM_WRITE.lock().unwrap();
    let now = std::time::Instant::now();
    let ok = last.map_or(true, |t| now.duration_since(t) >= std::time::Duration::from_millis(500));
    if ok {
        *last = Some(now);
    }
    ok
}

/// The live window's current outer rect (physical px) + maximized flag, or None if it can't be read.
fn current_geometry(win: &WebviewWindow) -> Option<app::WindowGeometry> {
    let maximized = win.is_maximized().unwrap_or(false);
    let pos = win.outer_position().ok()?;
    let size = win.outer_size().ok()?;
    Some(app::WindowGeometry {
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
        maximized,
    })
}

/// Apply a geometry to the live window THROUGH the D-115 clamp: if it is off EVERY current monitor's
/// work area it is NOT applied (the window keeps its current placement) — never restored off-screen
/// (the unplugged-monitor / laptop-vs-ultrawide case). Shared by session restore (`restore_geometry`,
/// applied before show → config default on discard) and named-state load (`apply_window_geometry`).
fn apply_geometry_clamped(win: &WebviewWindow, geom: &app::WindowGeometry) {
    let work_areas: Vec<(i32, i32, u32, u32)> = win
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| {
            let wa = m.work_area();
            (wa.position.x, wa.position.y, wa.size.width, wa.size.height)
        })
        .collect();
    if work_areas.is_empty() || !app::geometry_on_any_workarea(geom, &work_areas) {
        tracing::info!("window geometry is off-screen — not applied (D-115 clamp)");
        return;
    }
    if geom.maximized {
        let _ = win.maximize();
        return;
    }
    let _ = win.set_position(PhysicalPosition::new(geom.x, geom.y));
    let _ = win.set_size(PhysicalSize::new(geom.width, geom.height));
}

/// Restore the SESSION geometry, applied BEFORE `show()` so there is no visible jump (the window is
/// created hidden via `visible:false`). No saved geometry, or an off-screen rect → the config default
/// + centre (via the clamp inside `apply_geometry_clamped`).
fn restore_geometry(win: &WebviewWindow, store_path: &std::path::Path) {
    if let Some(geom) = app::read_session_geometry(store_path) {
        apply_geometry_clamped(win, &geom);
    }
}

/// Persist geometry on move/resize (throttled) and on close (authoritative). `write_session_geometry`
/// is a read-modify-write, so it never clobbers the webview's layout / named states / unknown keys.
fn on_geometry_event(event: &WindowEvent, win: &WebviewWindow, store_path: &std::path::Path) {
    match event {
        WindowEvent::CloseRequested { .. } => {
            if let Some(g) = current_geometry(win) {
                let _ = app::write_session_geometry(store_path, &g);
            }
        }
        WindowEvent::Moved(_) | WindowEvent::Resized(_) => {
            if geometry_throttle_ok() {
                if let Some(g) = current_geometry(win) {
                    let _ = app::write_session_geometry(store_path, &g);
                }
            }
        }
        _ => {}
    }
}

// ── Tauri commands ─────────────────────────────────────────────────────────────

/// Returns the current lifecycle state. Called by Svelte on mount so it gets
/// the live state regardless of when the webview finished loading.
#[tauri::command]
fn get_state(state: tauri::State<CurrentState>) -> ClientStateEvent {
    state.0.lock().unwrap().clone()
}

/// Read-only snapshot of the outbound pacing queue for a Space (Ch6 §6.14.4).
/// Returns one entry per sender that has activity recorded. The Svelte layer
/// projects this into `data-pacing-state` / `--xgen-pacing-*` custom property
/// values (Ch6 §6.14.3, §6.14.4).
#[tauri::command]
fn get_pacing_state(space_id: String, pacing: tauri::State<Pacing>) -> Vec<PacingState> {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    pacing
        .0
        .lock()
        .map(|m| m.snapshots_for_space(&space_id, now_ms))
        .unwrap_or_default()
}

/// Live connection traffic snapshot (M-RP6.6 Leg C): observed bytes in/out this
/// resident session + last ping/pong RTT. `rtt_ms` is `null` before the first pong
/// (absent-not-zero, D4/N-060), never a fabricated 0. The Svelte `selfState.traffic`
/// slot reads these snake_case fields verbatim (no mapping layer). Mirrors the
/// `get_pacing_state` command shape.
#[tauri::command]
fn get_conn_stats(traffic: tauri::State<TrafficStats>) -> crate::resident::ConnTraffic {
    traffic.0.snapshot()
}

/// Live resident reconnect status (M-RP6.3 Leg A): attempt count, the cap, the
/// countdown to the next dial, and whether the resident has PARKED at the cap.
/// The `get_conn_stats` shape — a thin read over shared atomics, folded into the
/// same 2 s poll (no second push channel; a second channel would be a second
/// surface, D-067).
///
/// `next_attempt_in_ms` is `null` whenever there is no countdown — connected,
/// mid-dial, or parked (absent-not-zero, D4/N-060). `max_attempts` /
/// `connect_timeout_ms` / `ping_interval_ms` travel as DATA so Svelte never
/// hardcodes a second copy of a tunable (D5).
#[tauri::command]
fn get_resident_status(
    status: tauri::State<ResidentStatusState>,
) -> crate::resident::ResidentStatusSnapshot {
    status.0.snapshot()
}

/// Re-arm the resident after it parked at the retry cap (M-RP6.3 Leg A, D4).
/// ONE command, THREE callers: the `[Reconnect]` action, window focus, and
/// network-return. The network-detection half stays in the webview deliberately —
/// Rust-side link detection means a new dependency and a per-platform surface,
/// where the WebView already gives `online` / `focus` for free.
///
/// IDEMPOTENT AND GUARDED: a resume is ignored unless the resident is actually
/// terminal, so a focus storm can never reset a live backoff mid-schedule.
/// Returns whether a resume was actually signalled — the honest answer, so the
/// caller cannot report "reconnecting" when nothing happened (D6's spirit: never
/// look like it worked when it did not).
///
/// `source` NAMES THE CALLER, and it is load-bearing rather than decorative:
/// leaving the terminal state is a user-visible transition with three possible
/// causes, and during the Leg-A verify one occurred that could NOT be attributed
/// from the logs — focus/online counters read zero, so the record would have had
/// to guess. An unattributable state change is a debugging hole; every resume now
/// says who asked for it, in dev and in the field alike.
#[tauri::command]
fn resume_resident(
    source: String,
    status: tauri::State<ResidentStatusState>,
    resume: tauri::State<ResidentResume>,
) -> bool {
    if !status.0.is_terminal() {
        tracing::debug!(source = %source, "resident: resume ignored — not terminal");
        return false;
    }
    tracing::info!(source = %source, "resident: resume accepted");
    resume.0.send_modify(|gen| *gen = gen.wrapping_add(1));
    true
}

/// Send a chat message over the RESIDENT socket (M-RP6.3 Leg B, D1).
///
/// The whole point of narrow-B: this multiplexes onto the one live connection
/// rather than dialling per message. It does NOT use `send_event_confirmed` —
/// that owns the drain and would swallow inbound fan-out (D2's binding
/// invariant); the drain writes and correlates instead.
///
/// **Re-anchor first (B-5).** If the Space has not been anchored this session,
/// fetch its DAG tips over a SHORT-LIVED CONTROL CONNECTION (`ops::*` shape,
/// D-056 control-mode) and seed the tracker. Once per session per Space, never
/// per message — that is what keeps `get_dag_tips` off the resident socket while
/// still giving the first message of a session a correct anchor.
///
/// Returns the honest four-way `SendOutcome`. `timed_out` is NOT folded into
/// success or failure: the node may be holding the event, and D6 forbids a
/// message that looks sent when it is not.
#[tauri::command]
async fn send_message(
    space_id: String,
    room_id: String,
    text: String,
    // M-RP-INTRO Leg 2 — the DM welcome intro payload, or `None`/`null` for every
    // ordinary send. `Option` so a caller may omit it; the shipped webview caller
    // passes an explicit `null` rather than relying on that, because whether Tauri
    // treats a MISSING argument as `None` is measured (runbook V-2), not assumed.
    intro: Option<serde_json::Value>,
    app: AppHandle,
) -> crate::resident::SendOutcome {
    use crate::resident::SendOutcome;

    // 🛑 UNCHANGED, AND DELIBERATELY SO (M-RP-INTRO 1-bis / runbook §3.2 item 4).
    // This guard is what makes the load-bearing-`text` rule enforceable at the
    // seam: AN INTRO WITH NO SENTENCE IS NOT SENDABLE, and that is the point. A
    // message whose only human-readable field was replaced by a payload older
    // clients cannot read is precisely the degradation option (d) was chosen to
    // avoid. Do not weaken it to let a text-less intro through.
    if text.trim().is_empty() {
        return SendOutcome::failed("empty message");
    }

    // 1) Re-anchor this Space if the current session has not yet done so.
    let needs_anchor = !app.state::<Tips>().0.is_anchored(&space_id);
    if needs_anchor {
        let data_dir = app.state::<DataDir>().0.clone();
        let config_path = app.state::<ConfigPath>().0.clone();
        match reanchor_space(&data_dir, &config_path, &space_id).await {
            Ok(tips) => {
                // Logged, not silent: the anchor is what causal correctness rests
                // on, and "which tips did we anchor on" is otherwise invisible from
                // outside the process — the same reasoning as `resume_resident`'s
                // `source`. Cheap (once per Space per session) and the first thing
                // anyone debugging a mis-chained message will want.
                tracing::info!(
                    space_id = %space_id,
                    tip_count = tips.len(),
                    tips = ?tips,
                    "send_message: re-anchored from the control plane"
                );
                app.state::<Tips>().0.seed(&space_id, tips);
            }
            Err(e) => {
                // Not fatal: the tracker still holds whatever the drain observed,
                // and an unanchored send falls back to the root anchor exactly as
                // `ops::send` always has. Logged, never silently swallowed.
                tracing::warn!(
                    space_id = %space_id,
                    reason = %format!("{e:#}"),
                    "send_message: re-anchor failed; sending on tracked/fallback tips"
                );
            }
        }
    }

    // 2) Queue it and wait for the drain's verdict.
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    let queued = app
        .state::<Outbound>()
        .0
        .try_send(crate::resident::OutboundRequest {
            // The Tauri command params arrive as the external `String` form
            // (Tauri IPC boundary, Pass 4 §4.2); project to OutboundRequest's
            // typed fields here — D-137, one projection per direction.
            space_id: SpaceXgid::from_xgid(Xgid::new(space_id)),
            room_id: RoomXgid::from_xgid(Xgid::new(room_id)),
            text,
            // M-RP-INTRO Leg 2 — carried VERBATIM, at the SAME projection point,
            // and deliberately not inspected here: the drain owns what content
            // looks like (`OutboundRequest`'s "intent, not an event"), and the
            // content key is named exactly once, there.
            intro,
            reply: reply_tx,
        });
    if let Err(e) = queued {
        return SendOutcome::failed(format!("not queued: {e}"));
    }

    // BOUNDED, and the bound is the whole point (Leg D1). The drain resolves the
    // reply for every request it WRITES — but it only polls `outbound_rx` inside
    // `run_session`, so a request queued during an outage is never popped and had,
    // until this bound existed, no timer over it at all: the promise never
    // resolved and the message was written unannounced whenever the link returned.
    // On expiry `await_send_outcome` reports `failed` — never reached the wire —
    // and the drain discards the request if it surfaces later.
    crate::resident::await_send_outcome(reply_rx, crate::resident::SEND_QUEUE_TIMEOUT).await
}

/// Fetch a Space's DAG tips over a short-lived CONTROL connection (M-RP6.3 D2 —
/// `get_dag_tips` may never run on the resident socket). The `ops::*` shape: its
/// own `SessionState`, its own connect, its own goodbye.
async fn reanchor_space(
    data_dir: &std::path::Path,
    config_path: &std::path::Path,
    space_id: &str,
) -> anyhow::Result<Vec<String>> {
    let node = app::resolve_node(None, config_path);
    let mut session = crate::session::SessionState::new(node, data_dir.to_path_buf());
    session.ensure_identity(&app::resolve_keypair_path(config_path))?;
    let timeout = std::time::Duration::from_secs(5);
    let conn = session.ensure_connected(None).await?;
    let tips = crate::batch::get_dag_tips(conn, space_id, timeout).await?;
    let _ = conn.goodbye("client_disconnect").await;
    Ok(tips)
}

/// Emit a live inbound Event to the webview (M-RP6.3 Leg B — live ingest, the
/// M-RP6.6 deferral closing). One channel, the `xgen-client-state-changed`
/// precedent: the drain calls this, R5 listens.
fn emit_event(app: &AppHandle, ev: &xgen_core::wire::types::Event) {
    let _ = app.emit("xgen-event", ev);
}

/// Emit a temperature update to the Svelte layer (spec 3.7.13, Ch6 §6.12).
/// Invoked by the Node event ingest pipeline when an incoming Event carries
/// `xgen.room_temperature` or `xgen.member_temperature` in its meta_atts. The
/// Svelte layer projects this into `data-temp-state` and the
/// `--xgen-*-temperature` custom properties.
///
/// Not yet called from the current scaffold (no Phase 2 ingest path wired
/// into the Tauri shell yet); reserved as the API surface the ingest will
/// use once it lands.
#[allow(dead_code)]
fn emit_temperature_update(app: &AppHandle, update: &TemperatureUpdate) {
    let _ = app.emit("xgen-temperature-update", update);
}

/// Returns the raw `[substitutions] rules` string from xgen-client_config.toml
/// (M-RP4.2). Called by the Svelte shell on boot; the `$common` processor store
/// parses it (split on ` | `, first space per pair → find | replace) and feeds
/// every processor-host. Empty string when the section or the file is absent.
#[tauri::command]
fn get_substitutions(config: tauri::State<ConfigPath>) -> String {
    app::load_substitutions_section(&config.0).rules
}

/// Writes the raw `[substitutions] rules` string back to xgen-client_config.toml
/// (M-RP4.3 — the effect half of the `substitutions-editor` widget). Symmetric with
/// `get_substitutions`. Returns `Err(String)` (surfaced to the webview) on a
/// read/parse/write failure rather than clobbering the config (D-065).
///
/// ⚠️ BOTH HALVES OF THE OLD NOTE HERE WERE FALSIFIED BY M-RP-PROCESSOR-WIRE and are
/// corrected rather than deleted, because each was true when written:
/// • "the widget's host-injected `onApply` callback invokes this" — FALSE since Leg A.
///   The client reaches this through the **persist seam on the `$common` store**
///   (`store.svelte.ts`), filled once by the shell at boot beside the `get_substitutions`
///   read. `onApply` survives as the **sampler's** live-only channel (D-097/W-8), where
///   the seam is never filled — so the two hosts differ, and the editor's own note
///   derives from `durable` rather than asserting either.
/// • "Session-only under D-101 (clean-slate-on-start re-seeds every launch)" — FALSE
///   since Leg C. **The write is now DURABLE in the client.** `clean_slate_config`
///   captures `[substitutions].rules` before the wipe and re-injects after regeneration:
///   D-101 still wipes the config whole, and only the user's rule TEXT rides across.
///   *Not an exemption from D-101 — the correction of a mis-filing: user-owned content
///   was filed into the config file, and D-101 wipes config files.* See D-101's
///   amendment. J-438 seed-once therefore resumes for this section only: cleared pairs
///   stay cleared. The sampler still clean-slates and is unchanged.
#[tauri::command]
fn set_substitutions(rules: String, config: tauri::State<ConfigPath>) -> std::result::Result<(), String> {
    app::write_substitutions_section(&config.0, &rules).map_err(|e| e.to_string())
}

/// Returns the raw UI-state store JSON (M-RP6.1k, Leg B) from `<data_dir>/xgen-client_uistate.json`.
/// The `get_substitutions` shape — a thin wrapper over the app-layer file read. Rust NEVER parses the
/// blob (D-114: the layout is the webview's, opaque; only `geometry` becomes a typed struct, at Leg C,
/// where the clamp needs it). Empty string when the store is absent OR unreadable, so the webview's
/// `loadLayout()` falls back to `DEFAULT_LAYOUT` (N-095). App-defined command → no capability grant
/// (J-497 — grounded: `get_substitutions` et al. run with none).
#[tauri::command]
fn get_ui_state(data: tauri::State<DataDir>) -> String {
    app::load_ui_state(&data.0.join(UI_STATE_FILE))
}

/// Writes the raw UI-state store JSON back (M-RP6.1k, Leg B — the `set_substitutions` shape). The
/// webview owns the blob and sends it verbatim; Rust writes it verbatim (opaque round-trip, D-114 —
/// unknown keys survive). Returns `Err(String)` to the webview on a write failure rather than losing
/// the store (D-065).
#[tauri::command]
fn set_ui_state(json: String, data: tauri::State<DataDir>) -> std::result::Result<(), String> {
    app::write_ui_state(&data.0.join(UI_STATE_FILE), &json).map_err(|e| e.to_string())
}

/// Returns the live window's current geometry (M-RP6.1k, Leg D) so the webview can embed it in a NAMED
/// UI state on save (§4.2 — a named state carries the arrangement AND the window rect). App-defined
/// command → no capability grant (J-497). None if the window / rect can't be read.
#[tauri::command]
fn get_window_geometry(app: AppHandle) -> Option<app::WindowGeometry> {
    app.get_webview_window("main").and_then(|w| current_geometry(&w))
}

/// Applies a geometry to the live window THROUGH the D-115 clamp (M-RP6.1k, Leg D) — used when loading a
/// named UI state, so "Reading" restores its window rect too, and an off-screen saved rect is ignored
/// rather than throwing the window off-screen. Rust-side set_position/set_size → NO capability grant
/// (the restore path already does this; capabilities gate JS→Rust IPC, not Rust→runtime calls).
#[tauri::command]
fn apply_window_geometry(app: AppHandle, geom: app::WindowGeometry) {
    if let Some(win) = app.get_webview_window("main") {
        apply_geometry_clamped(&win, &geom);
    }
}

/// Returns the "About" environment block for the About dialog (M-RP6.1e-C2).
/// A thin wrapper over `xgen_common::about::collect` — no logic here (the
/// `get_substitutions` shape). Build metadata (`built`/`commit`/`rustc`) comes
/// from `build_info`; the app facts are supplied here because `xgen-common` has
/// no `tauri`/JS dependency:
///   - `version` is **`xgen-client`'s own** `CARGO_PKG_VERSION` — deliberately
///     NOT `build_info::VERSION` (that is xgen-common's; both are 0.10.3 today);
///   - `tauri` = `tauri::VERSION`; `svelte` = the resolved version emitted by
///     this crate's `build.rs` from the committed `package-lock.json`.
/// Paths come from managed state (`DataDir`/`ConfigPath`), not re-derived.
#[tauri::command]
fn get_about_info(
    config: tauri::State<ConfigPath>,
    data: tauri::State<DataDir>,
) -> xgen_common::about::ClientAboutInfo {
    let common = xgen_common::about::collect(xgen_common::about::AboutParams {
        name: "XGen Client".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        link: "https://www.alchemydump.com".to_string(),
        tauri: tauri::VERSION.to_string(),
        svelte: env!("XGEN_SVELTE_VERSION").to_string(),
        data_dir: data.0.display().to_string(),
        config_path: config.0.display().to_string(),
    });
    xgen_common::about::ClientAboutInfo { common }
}

/// The "Self / connection" identity block for R3 (M-RP6.1g). A thin **shell read**
/// (the `get_about_info` shape) — NOT a D-092 four-armed verb: no node round-trip,
/// no mutation, no CLI meaning, so it never grows those arms and never touches
/// `ops.rs`. `NodeSelfState` is deliberately NOT declared (the J-497 `NodeAboutInfo`
/// precedent — a node wrapper today would guess fields with no call site to
/// validate them against; it lands with the node's real Self at M-RP7.x).
#[derive(serde::Serialize)]
struct SelfStateInfo {
    /// `false` when `xgen-client_state.json` is absent (the Tauri shell has never
    /// `register`ed) — rendered honestly, not faked (D-065 / W-8).
    registered: bool,
    /// From the KEYPAIR, so it is present even when unregistered — the field that
    /// proves the command is live (a `null` here means the keypair read is wrong).
    identity_id: Option<String>,
    display_name: Option<String>,
    home_node: Option<String>,
    spaces_joined: usize,
}

/// Read the self-identity + registration facts for R3 (M-RP6.1g, D2). A thin
/// wrapper composing two EXISTING readers — no new projection surface:
///   1. `ClientIdentity::load` derives the identity XGID from the keypair alone
///      (no registration, no node);
///   2. `ops::whoami` reads `xgen-client_state.json` for the registration facts
///      — `Err` (file absent) is NOT an error to the webview, it is
///      `registered: false`.
/// The data dir comes from managed `DataDir` (never re-derived). No `app.emit`,
/// no resident: the live connection flip is the resident's job (M-RP6.6, D7) —
/// this closes only the *read shape* of gate finding F-1.
#[tauri::command]
fn get_self_state(data: tauri::State<DataDir>) -> SelfStateInfo {
    let data_dir = data.0.clone();

    // 1. XGID from the keypair — present even when unregistered.
    let keypair_path = data_dir.join("xgen-client_keypair.enc");
    let keypair_xgid = crate::session::ClientIdentity::load(&keypair_path)
        .ok()
        .map(|ci| ci.identity_id.to_string());

    // 2. Registration facts. `whoami` never touches `session` — it only reads the
    //    on-disk state file — so a throwaway `SessionState` with an EMPTY home_node
    //    satisfies the `&mut` borrow. No config read for a value nothing consumes.
    let mut session = crate::session::SessionState::new(String::new(), data_dir.clone());
    let mut ctx = crate::ops::OpContext {
        session: &mut session,
        data_dir: &data_dir,
        node_override: None,
    };

    match crate::ops::whoami(&mut ctx) {
        Ok(w) => {
            // Prefer the keypair-derived XGID (it IS the identity); `whoami`'s is a
            // cached copy. If they disagree, flag it (Rule 1) rather than paper over
            // it — a `warn!` not a `debug_assert!`, so a stale state file logs the
            // divergence instead of panicking a running UI.
            let cached = w.identity_id.to_string();
            if let Some(kp) = keypair_xgid.as_deref() {
                if kp != cached {
                    tracing::warn!(
                        keypair = %kp,
                        cached = %cached,
                        "get_self_state: keypair XGID differs from state-file XGID"
                    );
                }
            }
            SelfStateInfo {
                registered: true,
                identity_id: keypair_xgid.or(Some(cached)),
                display_name: Some(w.display_name).filter(|s| !s.is_empty()),
                home_node: Some(w.home_node.to_string()).filter(|s| !s.is_empty()),
                spaces_joined: w.spaces_joined,
            }
        }
        Err(_) => SelfStateInfo {
            registered: false,
            identity_id: keypair_xgid,
            display_name: None,
            home_node: None,
            spaces_joined: 0,
        },
    }
}

/// The known-Space tree for R1/R2 (M-RP6.2, D1). A thin shell read (the
/// `get_self_state` shape) composing the EXISTING, unit-tested `ops::spaces` —
/// NOT a D-092 four-armed verb: no node round-trip, no mutation, no CLI meaning,
/// so it never grows those arms and never touches `ops.rs`. Each `KnownSpace`
/// carries its `rooms` EMBEDDED (`state.rs`), so R2 reads them from this same
/// payload — there is no `get_rooms` UI command. Carried VERBATIM (snake_case as
/// serde writes it; `KnownSpace`/`KnownRoom` are plain-field structs, no XGID
/// newtypes) — no rename/mapping layer (a mapping layer is a drift surface; the
/// self-state precedent). `Err` (state file absent / unregistered) is NOT an
/// error to the webview — `unwrap_or_default()` gives the honest empty list
/// (D-065 / W-8), which R1 renders as its "No spaces yet" empty state.
#[tauri::command]
fn get_spaces(data: tauri::State<DataDir>) -> Vec<xgen_common::state::KnownSpace> {
    let data_dir = data.0.clone();
    // `spaces` never touches `session` (it reads the on-disk state file), so a
    // throwaway `SessionState` with an EMPTY home_node satisfies the `&mut`
    // borrow — the `get_self_state` precedent, no config read for a value
    // nothing consumes.
    let mut session = crate::session::SessionState::new(String::new(), data_dir.clone());
    let mut ctx = crate::ops::OpContext {
        session: &mut session,
        data_dir: &data_dir,
        node_override: None,
    };
    crate::ops::spaces(&mut ctx)
        .map(|r| r.spaces)
        .unwrap_or_default()
}

/// The local address book for R7 (M-RP-MEMBERS Leg A). A pure on-disk read, the
/// `get_spaces` shape — but it needs NO `OpContext`/`SessionState` at all, since
/// `AddressBook::load` takes a `&Path` and never touches the network.
///
/// ⚠️ Returns `Result`, NOT `unwrap_or_default()`. `get_spaces` swallows its
/// `Err` because there `Err` means "state file absent = unregistered", an honest
/// empty render. Here a **missing** book is ALREADY `Ok(empty)` (`load`'s
/// NotFound arm), so the only `Err` is a **corrupt** file — and swallowing that
/// would render corruption as an empty book, the exact "absence renders as fine"
/// failure the Phase-0 §3 display rule forbids (D-065). The corrupt path leaves
/// the file untouched for inspection.
#[tauri::command]
fn get_address_book(
    data: tauri::State<DataDir>,
) -> Result<crate::address_book::AddressBook, String> {
    crate::address_book::AddressBook::load(&data.0).map_err(|e| format!("{e:#}"))
}

/// Fill the address book from one Space's live DAG, persist it, AND return the
/// Space's resolved membership (M-RP-MEMBERS Leg A / Leg A-bis). The
/// `reanchor_space` shape (runbook §1) — NOT the `get_spaces` shape:
/// `fill_and_members` calls `ensure_connected`, so it needs a REAL session with
/// a resolved `home_node`, not the throwaway empty-`home_node` `SessionState`
/// that `whoami`/`spaces` get away with.
///
/// Returns a `FillMembersOutcome` (`{ fill, roster }`) from a SINGLE drain
/// (Leg A-bis): the R7 widget needs the roster to render and the fill to resolve
/// names, and draining twice would roughly double the cold-start window state ③
/// is on screen.
///
/// Fire-and-forget from the frontend, OFF THE CRITICAL PATH (Joe-locked,
/// address-book §4): the Space opens at once and the view populates from the
/// returned roster (names resolving via the freshly-filled book) when this
/// resolves. A Tauri command is already off the render path, so honouring that
/// lock costs nothing here.
///
/// The `FillLock` guard serialises overlapping fills (§3). `fill_and_members` is
/// re-entrant by design and self-cleans its connection on every exit (J-586) —
/// this caller adds no connection management. The underlying dial and each
/// `identity.get` reply are bounded (Leg A-bis / T1), so a black-hole node
/// makes this command REJECT rather than hang the lock forever.
#[tauri::command]
async fn fill_space_records(
    space_id: String,
    data: tauri::State<'_, DataDir>,
    config: tauri::State<'_, ConfigPath>,
    lock: tauri::State<'_, FillLock>,
) -> Result<crate::ops::FillMembersOutcome, String> {
    // Serialise concurrent fills across load → fill → save (§3): the loser of a
    // read-modify-write race would otherwise silently discard resolved records.
    let _guard = lock.0.lock().await;

    let data_dir = data.0.clone();
    let config_path = config.0.clone();

    // A REAL session (the `reanchor_space` way): resolve the node so `home_node`
    // is non-empty (else `ensure_connected` bails), and load the identity so the
    // fill can authenticate.
    let node = app::resolve_node(None, &config_path);
    let mut session = crate::session::SessionState::new(node, data_dir.clone());
    session
        .ensure_identity(&app::resolve_keypair_path(&config_path))
        .map_err(|e| format!("{e:#}"))?;

    let mut book = crate::address_book::AddressBook::load(&data_dir).map_err(|e| format!("{e:#}"))?;
    let mut ctx = crate::ops::OpContext {
        session: &mut session,
        data_dir: &data_dir,
        node_override: None,
    };

    let result = crate::ops::fill_and_members(&mut ctx, &mut book, &space_id).await;

    // Persist UNCONDITIONALLY, before propagating `result` (runbook §2 Step 6):
    // `fill_and_members` takes `&mut book` and applies touches + absorbed fetches
    // AS IT GOES, so on a mid-loop error the book already holds real
    // observations. Discarding them would throw away work that happened and force
    // the next fill to re-fetch. On an early error (nothing mutated) the save is
    // a harmless no-op write. A save FAILURE is a genuine disk error worth
    // surfacing over the fill outcome — the records were not persisted.
    book.save(&data_dir).map_err(|e| format!("{e:#}"))?;

    result.map_err(|e| format!("{e:#}"))
}

/// Fetch ONE Identity record on demand and absorb it into the book
/// (M-RP-IDENTITY-RESOLUTION Leg D, §7 Tier-1 fetch on join).
///
/// Called by the shell's membership router when a live `membership.join`
/// adds a member the fill was never asked about (G7) — the ONLY state in
/// which a roster row exists with no book record.
///
/// Returns the record AS THE BOOK NOW HOLDS IT, not as the wire delivered
/// it: `merge` is version-aware (§5 V2), so a seeded higher-version record
/// out-ranks a fetch and the caller must mirror what won, not what was sent.
///
/// `Ok(None)` ⇔ `identity.not_found` ⇔ state ③ (ERASED under `D-127`) — a
/// normal outcome, never an error. The frontend routes it to `_notFound`.
///
/// Takes the `FillLock` (§3): this is the SECOND writer of the on-disk book
/// and a read-modify-write race with a concurrent fill would silently
/// discard resolved records — the outcome `fill_space_records:678-679`
/// names in its own words.
#[tauri::command]
async fn fetch_identity(
    identity_id: String,
    data: tauri::State<'_, DataDir>,
    config: tauri::State<'_, ConfigPath>,
    lock: tauri::State<'_, FillLock>,
) -> Result<Option<crate::address_book::SeenRecord>, String> {
    let _guard = lock.0.lock().await;

    let data_dir = data.0.clone();
    let config_path = config.0.clone();

    // The `fill_space_records` preamble, unchanged: resolve the node so
    // `home_node` is non-empty (else `ensure_connected` bails), load the
    // identity so the fetch can authenticate.
    let node = app::resolve_node(None, &config_path);
    let mut session = crate::session::SessionState::new(node, data_dir.clone());
    session
        .ensure_identity(&app::resolve_keypair_path(&config_path))
        .map_err(|e| format!("{e:#}"))?;

    let mut book = crate::address_book::AddressBook::load(&data_dir).map_err(|e| format!("{e:#}"))?;
    let mut ctx = crate::ops::OpContext {
        session: &mut session,
        data_dir: &data_dir,
        node_override: None,
    };

    let fetched = crate::ops::identity_get(&mut ctx, &identity_id)
        .await
        .map_err(|e| format!("{e:#}"))?;

    let found = fetched.is_some();
    // The SAME "now" the fill stamps (ops.rs:2913) — an identical RFC-3339
    // millisecond form, so a joiner's `last_seen` is byte-comparable with a
    // fill's `last_seen` and never a second timestamp format in the one field
    // (runbook §7-b: the producer was grounded, not guessed).
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let _absorbed = crate::ops::absorb_fetch(&mut book, fetched, &now);

    // Persist before answering, for the same reason `fill_space_records`
    // does: an absorbed observation that is not saved is work thrown away.
    book.save(&data_dir).map_err(|e| format!("{e:#}"))?;

    // `found` is load-bearing and NOT redundant with `book.get`: `absorb_fetch`
    // leaves the book UNCHANGED on `not_found` (ops.rs:2837), so a pre-existing
    // cached record would still be returned by `book.get` — and reporting an
    // erased identity as resolved would keep the frontend's ③ arm from firing.
    Ok(if found {
        book.get(&identity_id).cloned()
    } else {
        None
    })
}

/// Create a DM Space with the given invitee (M-RP-MEMBER-ACT Leg B). Wraps the
/// built-and-tested `ops::create_dm_space` — the DM machinery already exists; this
/// is the Tauri surface that makes it reachable from the client UI (the
/// `fetch_identity` precedent). Leg C is its first caller. Returns the op's
/// `CreateDmSpaceResult` unchanged (R-2). Needs a LIVE node — it signs and sends
/// three events — so it resolves the node + identity like `fetch_identity`, not
/// the on-disk `get_spaces` shape.
#[tauri::command]
async fn create_dm_space(
    invitee: String,
    data: tauri::State<'_, DataDir>,
    config: tauri::State<'_, ConfigPath>,
) -> Result<crate::ops::CreateDmSpaceResult, String> {
    let data_dir = data.0.clone();
    let config_path = config.0.clone();

    let node = app::resolve_node(None, &config_path);
    let mut session = crate::session::SessionState::new(node, data_dir.clone());
    session
        .ensure_identity(&app::resolve_keypair_path(&config_path))
        .map_err(|e| format!("{e:#}"))?;

    let mut ctx = crate::ops::OpContext {
        session: &mut session,
        data_dir: &data_dir,
        node_override: None,
    };
    let args = crate::app::CreateDmSpaceArgs { invitee };
    crate::ops::create_dm_space(&mut ctx, &args)
        .await
        .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
fn quit(app: AppHandle) {
    emit_state(&app, ClientLifecycleState::Closing);
    // M-RP6.1k Leg D — save the final window geometry before exiting. Ctrl+Q / File→Exit reach here via
    // app.exit, which does NOT reliably fire WindowEvent::CloseRequested, so without this the
    // on_window_event saver would miss the last position on the keyboard/menu-quit path (the X-button
    // path IS covered by CloseRequested). write_session_geometry is a read-modify-write → it preserves
    // the layout / named states / unknown keys.
    if let Some(win) = app.get_webview_window("main") {
        if let Some(g) = current_geometry(&win) {
            let store_path = app.state::<DataDir>().0.join(UI_STATE_FILE);
            let _ = app::write_session_geometry(&store_path, &g);
        }
    }
    // Signal the pipe server to shut down before exiting.
    let _ = app.state::<PipeShutdown>().0.send(true);
    write_session_footer(ExitReason::Shutdown);
    app.exit(0);
}

// ── Startup sequence ───────────────────────────────────────────────────────────

async fn run_startup(
    app: AppHandle,
    data_dir: PathBuf,
    pipe_name: String,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
    traffic: crate::resident::TrafficCounters,
    status: crate::resident::ResidentStatus,
    resume_rx: tokio::sync::watch::Receiver<u64>,
    outbound_rx: tokio::sync::mpsc::Receiver<crate::resident::OutboundRequest>,
    tips: crate::resident::TipTracker,
) {
    // Always emit INITIALISING first, regardless of first-run state.
    emit_state(&app, ClientLifecycleState::Initialising);

    // M1 §1.1 — start the named pipe server on a dedicated task.
    #[cfg(target_os = "windows")]
    {
        let data_dir_clone = data_dir.clone();
        let pipe_name_clone = pipe_name.clone();
        let rx_clone = shutdown_rx.clone();
        tauri::async_runtime::spawn(async move {
            crate::batch::start_pipe_server(pipe_name_clone, data_dir_clone, rx_clone).await;
        });
        // --aicontrol sister server (M7 C2) — second independent spawn.
        let ai_dir = data_dir.clone();
        let ai_pipe = crate::aicontrol::aicontrol_pipe_name(&pipe_name);
        let ai_rx = shutdown_rx.clone();
        let state_lock: crate::aicontrol::StateFileLock =
            std::sync::Arc::new(tokio::sync::Mutex::new(()));
        tauri::async_runtime::spawn(async move {
            // expected_token = None: AC-D4 gate inert in v1 (M7C-D1).
            crate::aicontrol::start_aicontrol_server(ai_pipe, ai_dir, ai_rx, state_lock, None).await;
        });
        // M7-events C5: the client `.events` observer pipe — second WS to the
        // home Node, filter-at-drain.
        let events_dir = data_dir.clone();
        let events_pipe = crate::events_pipe::events_pipe_name(&pipe_name);
        let events_rx = shutdown_rx.clone();
        tauri::async_runtime::spawn(async move {
            crate::events_pipe::start_events_server(events_pipe, events_dir, events_rx).await;
        });
    }

    let config_path = data_dir.join("xgen-client_config.toml");
    let keypair_path = data_dir.join("xgen-client_keypair.enc");

    // D-101 — clean-slate-on-start (phase-scoped). Config is ephemeral this
    // phase: if one exists, wipe it and regenerate from seed BEFORE the
    // first-run read below.
    //
    // EXCEPT `[substitutions]`, which rides across the wipe (M-RP-PROCESSOR-WIRE
    // Leg C) — not an exemption from D-101 but the correction of a mis-filing:
    // user-owned content is not config. J-438 seed-once therefore resumes for
    // that one section, so cleared pairs stay cleared on relaunch. The other
    // four sections remain ephemeral by design. See `app::clean_slate_config`
    // and DECISIONS.md D-101 for the full why + exit condition.
    app::clean_slate_config(&config_path, &keypair_path);

    // First-run detection: neither config nor keypair exists.
    if !config_path.exists() && !keypair_path.exists() {
        emit_state(&app, ClientLifecycleState::Setup);
        return;
    }

    // M-RP6.6 Leg A — the real long-lived resident. Replaces the discard-the-
    // stream scaffold (connect_async → throw the socket away → sleep(150ms) →
    // Ready). Node via `resolve_node` (D-068, no hardcoded URL), a REAL
    // `client_authenticate`, and lifecycle driven by ACTUAL socket outcomes
    // through the shared `resident::run_session` spine (D1 — the same spine the
    // headless `--service` mode drives, its sink logging instead of emitting).
    // Single-session in Leg A; Leg B wraps this in a reconnect loop with backoff.
    let node = app::resolve_node(None, &config_path);
    let signing_key = match app::load_keypair(&app::resolve_keypair_path(&config_path)) {
        Ok(k) => k,
        Err(e) => {
            tracing::warn!(reason = %format!("{:#}", e), "resident: keypair load failed — cannot authenticate");
            emit_state(&app, ClientLifecycleState::Disconnected);
            return;
        }
    };

    // The lifecycle sink for the shell: emit each live transition to the webview.
    // The service passes a logging sink to the same spine (D1). `app` is borrowed
    // for the resident's lifetime; nothing else uses it after this point.
    let mut shutdown_rx = shutdown_rx;
    let mut resume_rx = resume_rx;
    let mut outbound_rx = outbound_rx;
    // M-RP6.3 Leg B — live ingest. Every inbound Event the drain reads is emitted
    // to the webview here; R5 listens. `app` is borrowed by both closures, which
    // is fine — they are both `FnMut` captured by the same task.
    let ingest_app = app.clone();
    let mut ingest = move |ev: xgen_core::wire::types::Event| emit_event(&ingest_app, &ev);
    let mut io = crate::resident::SessionIo {
        outbound_rx: &mut outbound_rx,
        tips: &tips,
        ingest: &mut ingest,
    };
    let mut sink = |state: ClientLifecycleState| emit_state(&app, state);

    // Leg B — the long-lived reconnect loop. It owns `Reconnecting` and runs until
    // `quit` signals `shutdown_rx`. Supersedes the Leg-A single session + terminal
    // `Disconnected`: a dropped node now re-establishes without an app restart, so
    // the UI cycles Ready → Reconnecting → Connecting → Ready rather than sticking
    // at Disconnected.
    //
    // M-RP6.3 Leg A — the loop is now BOUNDED (D4): after RECONNECT_MAX_ATTEMPTS it
    // emits a terminal DISCONNECTED, publishes `terminal: true` on the status
    // surface, and PARKS on `resume_rx` rather than dialling forever. It still
    // never exits until shutdown, so there is still exactly one loop and one owner
    // of `Reconnecting`.
    crate::resident::run_resident(
        &node,
        &signing_key,
        &traffic,
        &status,
        &mut shutdown_rx,
        &mut resume_rx,
        &mut io,
        &mut sink,
    )
    .await;
    tracing::info!("resident: loop exited (shutdown requested)");
}

// ── Entry point ────────────────────────────────────────────────────────────────

/// Launch the Tauri desktop shell. Returns when the user shuts the app down.
///
/// `data_dir` — Tier-1 runtime files live here. The caller (`main::resolve_data_dir`)
/// resolves the root by precedence `--data-dir` > `XGEN_DATA_DIR` > platform default
/// (`xgen_common::data_dir::resolve_data_root`) — which FAILS FAST, with NO silent
/// `exe_dir()` fallback (M12.2b F9/D5). `--instance <label>`, when set, rebases the
/// instance dir under that root.
/// `instance_label` — for pipe name derivation. None → default pipe name.
pub fn run(
    data_dir: PathBuf,
    instance_label: Option<String>,
    log_level_override: Option<String>,
) {
    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    let log_dir = data_dir.join("logs");
    std::fs::create_dir_all(&log_dir).expect("failed to create logs/");
    let now = chrono::Local::now();
    let log_filename = format!("xgen-client_{}.log", now.format("%Y-%m-%d_%H-%M-%S"));
    let log_path = log_dir.join(&log_filename);
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("failed to open log file");

    use tracing_subscriber::{fmt, EnvFilter};
    // D-068 — flag > env (XGEN_LOG) > config (`[logging].level`) > "debug".
    // Pre-J-079 this path fell back to `EnvFilter::new("debug")`, silently
    // ignoring config. The convergence ships in J-079 commit 3.
    let config_path = data_dir.join("xgen-client_config.toml");
    let config_level = app::read_config_log_level(&config_path);
    let env_filter = EnvFilter::new(xgen_common::precedence::resolve_log_level(
        log_level_override.as_deref(),
        config_level.as_deref(),
    ));
    fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_ansi(false)
        .with_writer(log_file)
        .init();

    let started_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let session_id = format!("{:08x}", rand::random::<u32>());
    write_session_header(
        "client",
        None,
        None,
        None,
        "0.1",
        env!("CARGO_PKG_VERSION"),
        &session_id,
        &started_at,
    );

    // M1 §1.2 — derive pipe name from instance label.
    let pipe_name_str = crate::batch::pipe_name(instance_label.as_deref());

    // Write the PID file so `--pid` can find this resident.
    crate::app::write_pid_file(&data_dir);

    // Shutdown channel: sender stored as Tauri state, receiver passed to pipe server.
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // Initial stored state — Initialising until run_startup determines actual state.
    let initial = make_state_event(ClientLifecycleState::Initialising);
    let shared_state = CurrentState(Arc::new(Mutex::new(initial)));

    let pacing_manager = Pacing(Arc::new(Mutex::new(PacingManager::new())));

    // M-RP6.6 Leg C — one TrafficCounters, shared between the managed state (read
    // by `get_conn_stats`) and the resident (written by its `CountingStream` + RTT
    // tracker). Both hold clones of the same atomics.
    let traffic = crate::resident::TrafficCounters::default();

    // M-RP6.3 Leg A — one ResidentStatus, shared between the managed state (read by
    // `get_resident_status`) and `run_resident` (which writes the attempt count,
    // the countdown deadline and the terminal flag). Both hold clones of the same
    // atomics — the TrafficCounters contract.
    let resident_status = crate::resident::ResidentStatus::default();

    // The resume channel (D4). Starts at generation 0; `resume_resident` bumps it,
    // and the parked resident wakes on the change. A `watch` (not a `Notify`) so
    // the resident can `select!` on it beside `shutdown_rx` with no extra task.
    let (resume_tx, resume_rx) = tokio::sync::watch::channel(0u64);

    // M-RP6.3 Leg B — the outbound message queue + the shared DAG frontier.
    // Bounded at 64: a UI cannot realistically outrun the drain, and if it did,
    // backpressure surfacing as an honest `failed` beats unbounded memory or a
    // silent drop (D6).
    let (outbound_tx, outbound_rx) =
        tokio::sync::mpsc::channel::<crate::resident::OutboundRequest>(64);
    let tips = crate::resident::TipTracker::default();

    // M-RP4.2 — the config path for `get_substitutions`, derived from the same
    // data_dir the startup sequence uses (`run_startup` line ~139).
    let config_path = ConfigPath(data_dir.join("xgen-client_config.toml"));

    // M-RP6.1e-C2 — the data dir itself, so `get_about_info` reports the real
    // path rather than deriving it from the config path.
    let data_dir_state = DataDir(data_dir.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .manage(shared_state)
        .manage(PipeShutdown(shutdown_tx))
        .manage(pacing_manager)
        .manage(TrafficStats(traffic.clone()))
        .manage(ResidentStatusState(resident_status.clone()))
        .manage(ResidentResume(resume_tx))
        .manage(Outbound(outbound_tx))
        .manage(Tips(tips.clone()))
        .manage(config_path)
        .manage(data_dir_state)
        .manage(FillLock(tokio::sync::Mutex::new(())))
        .setup(move |app| {
            let handle = app.handle().clone();
            let dir = data_dir.clone();
            let pn = pipe_name_str.clone();
            let rx = shutdown_rx.clone();
            let tr = traffic.clone();
            let st = resident_status.clone();
            let rr = resume_rx.clone();
            let ob = outbound_rx;
            let tp = tips.clone();
            tauri::async_runtime::spawn(async move {
                run_startup(handle, dir, pn, rx, tr, st, rr, ob, tp).await;
            });

            // M-RP6.1k Leg C — restore window geometry BEFORE the window is shown (created hidden via
            // `visible:false`), then show it, then install the save-on-move/resize/close hook. The
            // clamp keeps it on-screen (D-115). `.or_else` is a defensive fallback in case the main
            // window's label is ever not "main" — the window must ALWAYS be shown.
            let store_path = data_dir.join(UI_STATE_FILE);
            if let Some(win) = app
                .get_webview_window("main")
                .or_else(|| app.webview_windows().into_values().next())
            {
                restore_geometry(&win, &store_path);
                let _ = win.show();
                let saver_win = win.clone();
                win.on_window_event(move |event| {
                    on_geometry_event(event, &saver_win, &store_path);
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            get_pacing_state,
            get_conn_stats,
            get_resident_status,
            resume_resident,
            send_message,
            quit,
            get_substitutions,
            set_substitutions,
            get_ui_state,
            set_ui_state,
            get_window_geometry,
            apply_window_geometry,
            get_about_info,
            get_self_state,
            get_spaces,
            get_address_book,
            fill_space_records,
            fetch_identity,
            create_dm_space
        ])
        .run(tauri::generate_context!())
        .expect("error while running xgen-client desktop shell");

    write_session_footer(ExitReason::Shutdown);
}

#[cfg(test)]
mod pass_4_commit_1_tests {
    //! XGID Retrofit Pass 4 Commit 1 — Surface #4 (Tauri Shell) per-surface
    //! tests T8 + T9 (runbook §6.3, design doc §4.6.b + §4.2 Instance C).
    use super::*;
    use xgen_common::xgid::{IdentityXgid, SpaceXgid, Xgid};

    /// T8 — a Tauri command return (`get_pacing_state` → `Vec<PacingState>`)
    /// crosses the IPC boundary to the JS frontend via serde-transparent
    /// newtypes (§4.2 Instance C): the JS-visible JSON carries identifier
    /// slots as plain strings, not nested objects.
    #[test]
    fn tauri_command_return_serde_transparent_to_js_frontend() {
        let snap = PacingState {
            space_id: SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:S".to_string())),
            sender_identity_id: IdentityXgid::from_xgid(Xgid::new(
                "xgen://pubkey/ed25519:M".to_string(),
            )),
            cap_ms: 2000,
            queue_count: 0,
            time_to_next_send_ms: 0,
            drain_ms: 0,
            is_ai: true,
        };
        let json = serde_json::to_string(&snap).unwrap();
        assert!(json.contains(r#""space_id":"xgen://hash/sha256:S""#), "got {json}");
        assert!(
            json.contains(r#""sender_identity_id":"xgen://pubkey/ed25519:M""#),
            "got {json}"
        );
    }

    /// Leg D — `fetch_identity`'s return (`Option<SeenRecord>`) crosses the IPC
    /// boundary with its two identifier slots as PLAIN STRINGS. `SeenRecord`
    /// carries `IdentityXgid` and `NodeXgid`; if either lost serde-transparency
    /// the frontend's `_book` would silently receive nested objects and every
    /// name lookup would miss. Exhaustive named literal (no `..Default`) so a
    /// future field addition breaks this test rather than passing silently
    /// (the J-669 V3-a rule).
    #[test]
    fn fetch_identity_return_serde_transparent_to_js_frontend() {
        use xgen_common::xgid::NodeXgid;
        let rec = crate::address_book::SeenRecord {
            identity_id: IdentityXgid::from_xgid(Xgid::new(
                "xgen://pubkey/ed25519:M".to_string(),
            )),
            display_name: Some("Ada".to_string()),
            is_ai: true,
            home_node: NodeXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:N".to_string())),
            last_seen: "2026-08-03T12:00:00.000Z".to_string(),
            update_version: 0,
            revoked: false,
            trust_assertion: None,
        };
        let json = serde_json::to_string(&rec).unwrap();
        assert!(
            json.contains(r#""identity_id":"xgen://pubkey/ed25519:M""#),
            "got {json}"
        );
        assert!(
            json.contains(r#""home_node":"xgen://pubkey/ed25519:N""#),
            "got {json}"
        );
    }

    /// T9 — `ClientStateEvent` carries no identifier slots (design doc §4.6.b
    /// drift correction): `state` is an enum, `label`/`timestamp` stay
    /// `String`. `make_state_event` produces descriptive Strings.
    #[test]
    fn lifecycle_state_event_descriptive_slots_stay_string() {
        let ev = make_state_event(ClientLifecycleState::Ready);
        let _label: &String = &ev.label; // descriptive, stays String
        let _timestamp: &String = &ev.timestamp; // descriptive, stays String
        assert_eq!(ev.label, "Ready");
        assert!(matches!(ev.state, ClientLifecycleState::Ready));
    }
}

#[cfg(test)]
mod m_rp6_2_tests {
    //! M-RP6.2 — R1 Spaces + R2 Rooms. The `get_spaces` command returns
    //! `Vec<KnownSpace>` VERBATIM (D1): the JS frontend receives each Space's
    //! `rooms` EMBEDDED (so R2 needs no second fetch) with snake_case keys (the
    //! store carries the shape unmapped). `KnownSpace`/`KnownRoom` are
    //! plain-field structs — no XGID newtypes — so this asserts STRUCTURE
    //! (embedded rooms, key names), not serde-transparency.
    use xgen_common::state::{KnownRoom, KnownSpace};

    /// The `get_spaces` payload serialises each Space with its Rooms nested and
    /// its keys in snake_case — the shape `spaces-state.svelte.ts` mirrors 1:1.
    #[test]
    fn get_spaces_payload_embeds_rooms_snake_case() {
        let payload = vec![KnownSpace {
            space_id: "xgen://hash/sha256:S1".to_string(),
            name: "Engineering".to_string(),
            node_endpoint: "ws://localhost:8080".to_string(),
            role: "member".to_string(),
            rooms: vec![KnownRoom {
                room_id: "xgen://hash/sha256:R1".to_string(),
                name: "general".to_string(),
                joined: true,
            }],
            counterpart: None,
        }];
        let json = serde_json::to_string(&payload).unwrap();
        // Space keys, snake_case, plain-string ids.
        assert!(json.contains(r#""space_id":"xgen://hash/sha256:S1""#), "got {json}");
        assert!(json.contains(r#""role":"member""#), "got {json}");
        // The Room rides EMBEDDED inside the Space (no separate get_rooms, D1).
        assert!(json.contains(r#""rooms":[{"#), "got {json}");
        assert!(json.contains(r#""room_id":"xgen://hash/sha256:R1""#), "got {json}");
        assert!(json.contains(r#""joined":true"#), "got {json}");
    }

    /// An empty tree (unregistered client → `unwrap_or_default()`) serialises to
    /// `[]` — the honest empty list R1 renders as "No spaces yet" (W-8), not an
    /// error the webview must handle.
    #[test]
    fn get_spaces_empty_tree_is_empty_array() {
        let payload: Vec<KnownSpace> = Vec::new();
        assert_eq!(serde_json::to_string(&payload).unwrap(), "[]");
    }
}
