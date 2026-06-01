// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M7 `--aicontrol` shared substrate (Commit C1).
//!
//! The pure, transport-free pieces both binaries need to speak the AI-shape
//! JSONL control protocol:
//!
//! - [`envelope`] — the AC-D2 inbound [`Command`] / outbound [`Reply`] +
//!   the closed [`Category`] set and the flat [`ErrorBody`].
//! - [`cmd`] — the AC-D1 `cmd` resolver (control verbs first, then the
//!   space-split CLI path).
//! - [`bindings`] — the §5 named binding engine + `$`-substitution.
//! - [`codes`] — the AC-D3d control-surface error catalogue + [`ControlError`].
//! - [`timeout`] — the AC-D3a 3-tier per-command timeout rule.
//! - [`filter`] — the M7-events `Filter` + `parse` + the shared pure `matches`
//!   subscription predicate (AC-D3b grammar; EV-D4 v1.1).
//!
//! **Adapter, not feature (D-065).** This module adds *no* business logic: it
//! never calls `ops::*` or `admin_ops::*`. Each binary's `--aicontrol`
//! dispatch arm composes these helpers with its own command layer (C2 client,
//! C4 node). It lives in `xgen-common` because both `xgen-client` and
//! `xgen-node` already depend on it and the pieces are pure (serde + serde_json
//! only) — the runbook §3 lean choice; the per-binary dispatch arm stays in
//! the binary.
//!
//! See `tasks/M7_AICONTROL_DESIGN.md` (AC-D1–AC-D6) and
//! `docs/xgen_aicontrol_implementation.md` v1.2 (§4 protocol, §5 bindings,
//! §8 codes, §10 timeout).

pub mod bindings;
pub mod cmd;
pub mod codes;
pub mod envelope;
pub mod filter;
pub mod timeout;

pub use bindings::{substitute, Bindings, BoundValue};
pub use cmd::{resolve_cmd, CmdPath, CmdResolution, ControlVerb};
pub use codes::{ControlCode, ControlError};
pub use envelope::{parse_command, Category, Command, ErrorBody, Reply};
pub use filter::{matches, parse, Filter};
pub use timeout::{
    resolve_timeout_ms, TimeoutTier, FEDERATION_TIMEOUT_SECS, READ_TIMEOUT_SECS, WRITE_TIMEOUT_SECS,
};
