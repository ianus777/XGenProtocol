// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! # xgen-mptest — strategic multiparty test harness (M9)
//!
//! Executes the J-305 Joe-LOCKED design (`tasks/M9_MULTIPARTY_HARNESS_DESIGN.md`,
//! M9-D1…M9-D9). The orchestrator **spawns the real built binaries** as separate
//! OS processes (M9-D3), drives each actor through its `--aicontrol` JSONL pipe
//! (M9-D1), and observes convergence/rejection through `.events`/`state`
//! (M9-D4). It does **not** link the binaries — black-box over the real
//! deployment surface, the in-process → real-binary crossing the existing
//! `phase9_harness` cannot do.
//!
//! ## Hard constraints (D-065 / D-084 / runbook §5)
//! - The harness **drives + observes** the binaries; it never patches them. A
//!   real defect the harness surfaces is a **finding routed to a fix-arc**,
//!   never patched under the M9 banner.
//! - The raw-wire injector (C4) is **test-only**, lives here, never ships.
//! - Heavy entry points (anything that spawns a process) are `#[ignore]` /
//!   out-of-band; the fast unit suite must not spawn processes.
//!
//! ## C1 modules (this commit — process lifecycle + drive)
//! - [`binloc`]   — locate the built `.exe`s (M9-D3 exe-location strategy).
//! - [`process`]  — [`process::ManagedProcess`] spawn + kill-on-drop + the
//!   instance-label namespacing that gives per-run isolation.
//! - [`wire`]     — harness-side `Command` (serialize) / `Reply` (deserialize)
//!   mirroring the `xgen-common::aicontrol` envelope (drift-locked by test).
//! - [`aicontrol`]— the named-pipe JSONL client (connect, send, read replies).
//! - [`manifest`] — `manifest.toml` schema + parse (M9-D8).
//! - [`resolve`]  — cross-actor `{{key}}` registry + substitution (M9-D8).
//! - [`batch`]    — the per-actor batch runner (read `.jsonl`, resolve, drive).
//!
//! ## C2 modules (oracle + capture)
//! - [`events`]  — the `.events` observation-pipe collector (live-only, attaches
//!   at scenario start; M9-D4).
//! - [`oracle`]  — convergence (membership + transcript) / rejection comparators
//!   — the cross-process `RoomState` `Eq` analogue (Checkpoint #2 surface).
//! - [`capture`] — capture-by-default artifact-dir writer (§3.4).
//!
//! ## C3 modules (round dial + micro-benchmark)
//! - [`dial`]     — the round dial (M9-D7 scale knobs) + clock-mode grounding.
//! - [`resource`] — per-process RSS + thread-count sampling.
//! - [`bench`]    — the spawn micro-benchmark → box-ceiling report.
//!
//! ## C4 module (raw-wire injector)
//! - [`injector`] — the test-only hostile wire-client (M9-D6, F-F): crafts +
//!   corrupts `Event`s with `xgen-core` builders and sends them over the real
//!   transport to a node's `Server`. Never ships (Checkpoint #3 surface).
//!
//! ## C5 support
//! - [`wireactor`] — a legitimate persistent wire-client (register + submit
//!   signed events); the cooperative counterpart to [`injector`]. The harness
//!   holds the actor's key, which lets MP-A-05 forge a *member's* identity to
//!   isolate the step-12 signature check.
//!
//! C5 (Round-0 smokes), C6 (close) build on top.
//!
//! ## MP-R1 (Multiparty-tests Round 1 — deterministic correctness floor)
//! - [`runner`] — the generic [`runner::run_scenario`] orchestrator + the G-6
//!   federation bootstrap (MP-R1-D1 / C1) that generalizes the hand-wired
//!   Round-0 smokes over N nodes/actors.

pub mod aicontrol;
pub mod batch;
pub mod bench;
pub mod binloc;
pub mod capture;
pub mod dial;
pub mod events;
pub mod injector;
pub mod manifest;
pub mod oracle;
pub mod process;
pub mod resolve;
pub mod resource;
pub mod runner;
pub mod wire;
pub mod wireactor;

/// Harness-wide error type. Thin alias over [`anyhow::Error`] so every layer
/// can `?`-propagate freely; the harness is test-only, so rich typed errors buy
/// little over a good message + context.
pub type Result<T> = anyhow::Result<T>;
