// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

pub mod about;
pub mod aicontrol;
pub mod build_info;
pub mod canonical;
pub mod clock;
pub mod conn;
pub mod data_dir;
pub mod event_trace;
pub mod module;
pub mod precedence;
pub mod space_local;
pub mod state;
pub mod trust_assertion;
pub mod wire;
pub mod xgid;

// XGID Adoption v1 (D-072) — re-export the type vocabulary at the crate root
// so downstream crates write `use xgen_common::{NodeXgid, EventXgid, ...}`
// instead of `use xgen_common::xgid::NodeXgid`.
pub use xgid::{
    AuthModuleXgid, EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, TrustAssertionXgid,
    Xgid, XgidDecodeError, XgidLike,
};

// M8.6 (Federation stress) — the clock-injection seam (Fork A). `Clock` +
// `RealClock` always-public; `MockClock` is feature-gated (`mock-clock`).
pub use clock::{Clock, RealClock};

// M7-events arc (EV-D1) — connection identifier for the multi-connection
// fan-out registry.
pub use conn::ConnId;

// Arc E (PG-03) — the Trust Assertion SignedPrimitive (ch3 §3.8.4 / §3.8.5).
pub use trust_assertion::{
    Erasability, ModuleKind, ModulePolicy, Retention, TrustAssertion, TrustAssertionVerifyError,
    TrustClaims,
};

// Storage-Engine / Plugin-Framework milestone (SE-D2) — host-neutral module /
// plugin descriptor + identity vocabulary, re-exported at the crate root.
pub use module::{AssuranceClass, Descriptor, ModuleImplId, ModuleKindId};
