// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

pub mod aicontrol;
pub mod build_info;
pub mod canonical;
pub mod conn;
pub mod event_trace;
pub mod module;
pub mod precedence;
pub mod space_local;
pub mod state;
pub mod wire;
pub mod xgid;

// XGID Adoption v1 (D-072) — re-export the type vocabulary at the crate root
// so downstream crates write `use xgen_common::{NodeXgid, EventXgid, ...}`
// instead of `use xgen_common::xgid::NodeXgid`.
pub use xgid::{
    AuthModuleXgid, EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, TrustAssertionXgid,
    Xgid, XgidDecodeError, XgidLike,
};

// M7-events arc (EV-D1) — connection identifier for the multi-connection
// fan-out registry.
pub use conn::ConnId;

// Storage-Engine / Plugin-Framework milestone (SE-D2) — host-neutral module /
// plugin descriptor + identity vocabulary, re-exported at the crate root.
pub use module::{AssuranceClass, Descriptor, ModuleImplId, ModuleKindId};
