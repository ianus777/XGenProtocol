// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Space and Room state derivation from the Event DAG (spec 3.7.1–3.7.9).
//
// SpaceState is derived by processing State Events in causal order.
// For Phase 1 (no concurrent state changes), the most recent event wins.
//
// Event building helpers are also here so tests can exercise the full pipeline.

use std::collections::{HashMap, HashSet};

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use rand::{rngs::OsRng, RngCore};
use serde_json::{json, Value};
use thiserror::Error;
use xgen_common::{
    wire::{empty_room_xgid, empty_space_xgid},
    xgid::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid},
};

use crate::{
    crypto::{encoding, hashing, signing},
    space::membership::{
        can_ban, can_create_room, can_invite, can_kick, can_mute, Effect, Role, RoomPermission,
    },
    wire::{
        canonical::canonical_event_bytes,
        types::{
            Event, EventType, ThreadStatus, DEFAULT_AI_PACING_MS, DEFAULT_HUMAN_PACING_MS,
            DEFAULT_MEMBER_TEMPERATURE_VISIBILITY, VISIBILITY_EVERYONE, VISIBILITY_MODERATOR,
            VISIBILITY_SELF_ONLY,
        },
    },
};

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpaceError {
    #[error("permission denied for role {0}")]
    PermissionDenied(String),
    #[error("identity is not a space member")]
    NotASpaceMember,
    #[error("identity is not a room member")]
    NotARoomMember,
    #[error("identity is already a member")]
    AlreadyMember,
    #[error("identity is banned")]
    Banned,
    #[error("identity has not been invited")]
    NotInvited,
    #[error("room not found")]
    RoomNotFound,
    #[error("missing required content field: {0}")]
    MissingField(&'static str),
    #[error("invalid event type for this operation")]
    WrongEventType,
    /// DM Space constraint violations (3.16.1).
    #[error("invitations are disabled in DM Spaces")]
    DmInvitationNotAllowed,
    #[error("DM Spaces may only have one Room")]
    DmSecondRoomNotAllowed,
    #[error("federation is disabled in DM Spaces")]
    DmFederationNotAllowed,
}

// ── Data structures ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceMember {
    pub identity_id: IdentityXgid,
    pub role: Role,
    pub joined_at: String,
    /// Identity that signed the `membership.invite` event admitting this member.
    /// `None` for the Space owner and any member admitted without an explicit invite
    /// (e.g. founding members of pre-M3 Spaces replayed from disk). Used by
    /// `resolve_operator` step 2 (spec 3.6.10.6).
    pub invited_by: Option<IdentityXgid>,
}

/// Pending-invite record (3.7.8). Carries the inviter alongside the assigned
/// role so `apply_join` can populate `SpaceMember.invited_by` for resolution
/// step 2 of `resolve_operator` (spec 3.6.10.6).
///
/// M8.5-B (INV-D6) — `valid_until` carries the invite's tier-graded validity
/// deadline (absolute RFC-3339 timestamp, TrustAssertion §3.8.4 semantics). It
/// is read from the `membership.invite` event content by `apply_invite`, so it
/// is deterministic across Nodes and convergence-neutral (no `state_key` of its
/// own — it rides the `pending_invites` map's `PartialEq`). `None` for invites
/// that carry no `valid_until` (the seeded DM invite, pre-M8.5-B invites): such
/// an invite never expires (the join-acceptance / read gates only bite when the
/// value is present and past). The number is filled by the inviter-side cascade
/// (C2); the Node only reads and enforces it here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingInvite {
    pub role: Role,
    pub invited_by: Option<IdentityXgid>,
    pub valid_until: Option<String>,
}

impl PendingInvite {
    /// Convenience for tests and pre-M3 replay paths that don't have an inviter.
    pub fn from_role(role: Role) -> Self {
        Self { role, invited_by: None, valid_until: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomState {
    pub room_id: RoomXgid,
    pub space_id: SpaceXgid,
    pub name: String,
    pub topic: Option<String>,
    pub members: HashSet<IdentityXgid>,
    /// Per-Room × per-Role permission overrides (PG-12-min, Arc D PM-D4). An
    /// absent key means "inherit the membership-default `can_X`"; a present key
    /// forces `Allow`/`Deny` for that `(Role, RoomPermission)` pair in this Room.
    /// Carried by `state.room_update` events (`apply_room_update`, replace
    /// semantics) and consulted by `check_permission`'s override layer. Empty by
    /// default — membership-only behaviour until an override is explicitly set.
    /// A `HashMap` compares order-independently, so it is a correct M8
    /// convergence oracle under `RoomState`'s `PartialEq`/`Eq`.
    pub permission_overrides: HashMap<(Role, RoomPermission), Effect>,
    /// MLS group epoch for this Room (Arc H PG-05, AH-D3). `None` until a
    /// `state.mls_group_init` event sets the genesis epoch (`Some(0)`); advanced
    /// on membership change in an E2E Space (epoch-advance is AH-D4 / C2). This is
    /// the Node-readable opaque epoch *counter* only — **no key material** lives
    /// Node-side (§3.10.1). Non-E2E Rooms keep `None`. Order-independent scalar,
    /// so it rides `RoomState`'s `PartialEq`/`Eq` M8 oracle additively.
    pub mls_epoch: Option<u64>,
    /// Canonical winning `mls.commit` for the current epoch (M8.7 CC-D5). The
    /// Node-readable *identity* of the commit that won resolution — **no key
    /// material**. `None` until the first `mls.commit`. The `mls_epoch` counter
    /// alone is vacuous under a concurrent commit-race: two honest commits both
    /// advance `N → N+1`, so the counter converges to `N+1` under either fold
    /// order with or without resolution. Which commit is *canonical* is the real
    /// divergence, and it is unobservable in the counter — this field records it.
    /// `apply_mls_commit` sets it; because `derive_resolved` admits only the
    /// resolved winner to the applied set (via the `(room, target_epoch)`
    /// conflict domain — `state_key_for_event`'s `MlsCommit` arm), the tip
    /// records the winner and the loser is excluded. Order-independent scalar, so
    /// it rides `RoomState`'s `PartialEq`/`Eq` M8 convergence oracle additively
    /// (mirrors `mls_epoch`). `RoomState` is unserialized (rebuilt from the log
    /// via `derive_resolved`), so this needs no persistence migration.
    pub mls_commit_tip: Option<EventXgid>,
}

/// Derived state of a Thread within a Space (Arc E PG-08, AE-D7). Keyed by the
/// conceptual Thread id (`xgen://thread/sha256:<hash of the create event>` —
/// AE-D8, no `ThreadXgid`). `created_by` reconciles the ch2 Thread anatomy.
/// A Thread is flat under one Room (no nesting) and is never deleted.
///
/// `PartialEq`/`Eq` so `SpaceState`'s convergence oracle (M8 `derive_resolved`)
/// compares thread state order-independently across arrival permutations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadState {
    /// Conceptual Thread id (`xgen://thread/sha256:`); also the `threads` map key.
    pub id: String,
    /// Parent Room (one Room, flat — no nesting).
    pub room_id: RoomXgid,
    /// The Identity that created the Thread.
    pub created_by: IdentityXgid,
    /// RFC 3339 creation timestamp (the create event's timestamp).
    pub created_at: String,
    /// Optional title.
    pub title: Option<String>,
    /// Lifecycle status (Open on create; Resolved/Archived are terminal).
    pub status: ThreadStatus,
    /// Minimum Auth Tier to participate (narrow-not-widen vs the Room/Space; AE-D9).
    pub auth_tier_min: u32,
    /// The `thread.create` event id this Thread derives from.
    pub origin_event: String,
}

// `PartialEq`/`Eq` added at M8 C1 (state-resolution convergence) — the
// convergence property tests assert that derive_resolved yields the *same*
// SpaceState across every arrival permutation. `PartialEq` over the HashMap/
// HashSet fields is order-independent (set/map equality), which is the correct
// "converged on identical state" oracle — a serde byte-compare would be
// polluted by non-deterministic HashMap iteration order. Purely additive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceState {
    pub space_id: SpaceXgid,
    pub name: Option<String>,
    pub topic: Option<String>,
    pub auth_tier: u32,
    pub max_event_size: Option<u64>,
    pub home_node: NodeXgid,
    pub owner_id: IdentityXgid,
    pub is_dm: bool,
    /// Declared legal jurisdiction of this Space (Arc G PG-04, AG-D1/D2).
    /// Set **once** at creation from `state.space_create` content; there is no
    /// mutation event, no applier arm and no `state_key_for_event` arm — a legal
    /// domain is fixed at Space birth (a silent post-hoc change is the integrity
    /// hazard to avoid). `None` ⇒ undeclared. An optional open string
    /// (ISO 3166-1 alpha-2 conventionally, e.g. `"SK"`/`"EU"`), preserved
    /// verbatim and never enumerated/validated — sibling to
    /// `member_temperature_visibility`. DM Spaces always declare `None` (AG-D4).
    /// Rides M8 for free: set-once ⇒ no conflict class, and `SpaceState`'s
    /// `PartialEq`/`Eq` (the `derive_resolved` oracle) covers it additively
    /// (AG-D3). The live federation containment hook is Arc G C2.
    pub jurisdiction: Option<String>,
    /// Whether this Space uses end-to-end (MLS) encryption for `message.*`
    /// content (Arc H PG-05, AH-D2). Set **once** at creation from
    /// `state.space_create` content; like `jurisdiction` there is no mutation
    /// event, no applier arm and no `state_key_for_event` arm — §3.10.8 makes it
    /// immutable after Space creation (a Space cannot be retroactively encrypted
    /// or decrypted). **Default `false` (OFF)** and **uniform** (no DM exception
    /// — read identically by all three constructors). Default-OFF is the honest
    /// dormant-but-correct posture (D-065): PG-05 closes interface-locked (real
    /// RFC 9420 crypto is D3), so defaulting Spaces *on* would stake the default
    /// Space's security on crypto that does not yet exist — the default-ON flip
    /// is a D3 decision, not Arc H's. Rides M8 for free (set-once ⇒ no conflict
    /// class; `SpaceState`'s `PartialEq`/`Eq` covers it additively, AG-D3 shape).
    pub e2e_encryption: bool,
    /// Active members: identity_id → SpaceMember
    pub members: HashMap<IdentityXgid, SpaceMember>,
    /// Invited but not yet joined: identity_id → PendingInvite (carries role + inviter).
    pub pending_invites: HashMap<IdentityXgid, PendingInvite>,
    /// Operator delegations for AI members (spec 3.6.10.6). Key = `ai_identity_id`,
    /// value = currently-delegated operator's identity_id. Absence means "no
    /// explicit delegation; resolution falls through to inviter, then owner."
    /// Updated by `state.ai_operator_delegate` / `state.ai_operator_revoke`.
    pub ai_operator_delegations: HashMap<IdentityXgid, IdentityXgid>,
    /// Banned identities (cannot rejoin).
    pub banned: HashSet<IdentityXgid>,
    /// Rooms within this Space: room_id → RoomState
    pub rooms: HashMap<RoomXgid, RoomState>,
    /// Federated nodes that participate in this Space.
    pub federation_nodes: Vec<NodeXgid>,
    /// Manual Node ordering from the most recent state.node_priority Event (3.9.3 Layer 5a).
    /// Ordered from highest priority (index 0) to lowest. Empty when no such Event exists.
    pub node_priority_order: Vec<NodeXgid>,
    /// DM Space constraints active (3.16.1). True for DM Spaces until state.dm_promote is applied.
    pub dm_constraints_active: bool,
    /// Minimum send interval (ms) for members with `is_ai = false` (spec 3.7.12.1).
    /// Phase 2 default is `500` when absent from `state.space_create`.
    pub human_pacing_ms: u64,
    /// Minimum send interval (ms) for members with `is_ai = true` (spec 3.7.12.1).
    /// Phase 2 default is `2000` when absent from `state.space_create`.
    pub ai_pacing_ms: u64,
    /// Visibility setting for `xgen.member_temperature` (spec 3.7.13.3).
    /// One of `moderator` (default), `everyone`, `self_only`. Open enum —
    /// unknown values are treated as `moderator` at enforcement time.
    pub member_temperature_visibility: String,
    /// Currently active mutes (spec 3.7.8). Key: target identity_id.
    /// Value: RFC 3339 `cooldown_until` timestamp. Members with an entry MUST
    /// NOT be permitted to post `message.*` Events until the timestamp passes.
    pub active_mutes: HashMap<IdentityXgid, String>,
    /// Threads within this Space (Arc E PG-08, AE-D7): thread_id → ThreadState.
    /// Conceptual `String` key (`xgen://thread/sha256:`, AE-D8). Order-independent
    /// `HashMap` — a correct M8 convergence oracle.
    pub threads: HashMap<String, ThreadState>,
}

impl SpaceState {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Initialise a SpaceState from a `state.space_create` Event.
    /// The Space ID is the event's event_id (must be set before calling).
    pub fn from_space_create(event: &Event) -> Result<Self, SpaceError> {
        if event.event_type != EventType::StateSpaceCreate {
            return Err(SpaceError::WrongEventType);
        }
        // The event's event_id IS the Space's identifier; cross-flavour wrap
        // EventXgid → SpaceXgid (Appendix J §J.2 — same underlying hash bytes,
        // different protocol-object role).
        let event_xgid = event.event_id.clone().ok_or(SpaceError::MissingField("event_id"))?;
        let space_id = SpaceXgid::from_xgid(event_xgid.into_xgid());
        let content = &event.content;
        let auth_tier = content["auth_tier"].as_u64().ok_or(SpaceError::MissingField("auth_tier"))? as u32;
        // Pass 2 widens content extraction to typed parse; the wrap collapses then.
        let home_node = NodeXgid::from_xgid(Xgid::new(
            content["home_node"]
                .as_str()
                .ok_or(SpaceError::MissingField("home_node"))?
                .to_string(),
        ));
        let name = content["name"].as_str().map(str::to_string);
        let topic = content["topic"].as_str().map(str::to_string);
        // Set-once jurisdiction (Arc G PG-04, AG-D1). Absent ⇒ undeclared.
        let jurisdiction = content["jurisdiction"].as_str().map(str::to_string);
        // Set-once E2E encryption mode (Arc H PG-05, AH-D2). Absent ⇒ false (OFF).
        let e2e_encryption = content["e2e_encryption"].as_bool().unwrap_or(false);
        let max_event_size = content["max_event_size"].as_u64();

        let creator = event.sender.clone();
        let owner = SpaceMember {
            identity_id: creator.clone(),
            role: Role::Owner,
            joined_at: event.timestamp.clone(),
            invited_by: None,
        };
        let mut members = HashMap::new();
        members.insert(creator.clone(), owner);

        let human_pacing_ms = content["human_pacing_ms"]
            .as_u64()
            .unwrap_or(DEFAULT_HUMAN_PACING_MS);
        let ai_pacing_ms = content["ai_pacing_ms"]
            .as_u64()
            .unwrap_or(DEFAULT_AI_PACING_MS);
        let member_temperature_visibility = content["member_temperature_visibility"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| DEFAULT_MEMBER_TEMPERATURE_VISIBILITY.to_string());

        Ok(SpaceState {
            space_id,
            name,
            topic,
            auth_tier,
            max_event_size,
            home_node,
            owner_id: creator,
            is_dm: false,
            jurisdiction,
            e2e_encryption,
            members,
            pending_invites: HashMap::new(),
            ai_operator_delegations: HashMap::new(),
            banned: HashSet::new(),
            rooms: HashMap::new(),
            federation_nodes: Vec::new(),
            node_priority_order: Vec::new(),
            dm_constraints_active: false,
            human_pacing_ms,
            ai_pacing_ms,
            member_temperature_visibility,
            active_mutes: HashMap::new(),
            threads: HashMap::new(),
        })
    }

    /// Initialise a DM SpaceState from a `state.dm_space_create` Event.
    /// Returns the SpaceState AND the automatically produced auto-room Event
    /// and membership.invite Event for the invitee (pre-signed with the creator's key).
    pub fn from_dm_space_create(
        event: &Event,
        creator_key: &SigningKey,
    ) -> Result<(Self, Event, Event), SpaceError> {
        if event.event_type != EventType::StateDmSpaceCreate {
            return Err(SpaceError::WrongEventType);
        }
        // Cross-flavour wrap EventXgid → SpaceXgid (same hash bytes).
        let event_xgid = event.event_id.clone().ok_or(SpaceError::MissingField("event_id"))?;
        let space_id = SpaceXgid::from_xgid(event_xgid.into_xgid());
        let content = &event.content;
        let auth_tier = content["auth_tier"].as_u64().unwrap_or(1) as u32;
        // Pass 2 widens content extraction to typed parse; the wraps collapse then.
        let home_node = NodeXgid::from_xgid(Xgid::new(
            content["home_node"]
                .as_str()
                .ok_or(SpaceError::MissingField("home_node"))?
                .to_string(),
        ));
        let invitee = IdentityXgid::from_xgid(Xgid::new(
            content["invitee"]
                .as_str()
                .ok_or(SpaceError::MissingField("invitee"))?
                .to_string(),
        ));
        // Set-once E2E mode, read uniformly from content (Arc H AH-D2). Unlike
        // `jurisdiction` (which DM Spaces hard-set to `None`), e2e is uniform —
        // no DM exception — so a DM Space honours a declared `e2e_encryption`
        // exactly like a regular Space (default OFF; no current caller declares
        // it for a DM, so it stays false until one does).
        let e2e_encryption = content["e2e_encryption"].as_bool().unwrap_or(false);

        let creator = event.sender.clone();
        let owner = SpaceMember {
            identity_id: creator.clone(),
            role: Role::Owner,
            joined_at: event.timestamp.clone(),
            invited_by: None,
        };
        let mut members = HashMap::new();
        members.insert(creator.clone(), owner);

        // Auto-create the DM room.
        let room_event = sign_event(
            build_room_create_event(creator_key, space_id.as_str(), "dm", None),
            creator_key,
        );
        let room_event_xgid = room_event.event_id.clone().unwrap();
        // Cross-flavour wrap EventXgid → RoomXgid (the room-create event's event_id IS the Room's identifier).
        let room_id = RoomXgid::from_xgid(room_event_xgid.into_xgid());

        // Auto-produce membership.invite for the invitee.
        let invite_event = sign_event(
            build_membership_event(
                creator_key,
                space_id.as_str(),
                room_id.as_str(),
                EventType::MembershipInvite,
                json!({ "target_identity": invitee.as_str(), "role": "member" }),
            ),
            creator_key,
        );

        let mut room = RoomState {
            room_id: room_id.clone(),
            space_id: space_id.clone(),
            name: "dm".to_string(),
            topic: None,
            members: HashSet::new(),
            permission_overrides: HashMap::new(),
            mls_epoch: None,
            mls_commit_tip: None,
        };
        room.members.insert(creator.clone());

        let mut pending_invites = HashMap::new();
        // M11-D1 (self thread): skip seeding a self-invite when invitee == creator.
        // A self-DM is a clean single-member room by construction — the creator is
        // already the Owner member + dm-Room member, so no second party is pending.
        // Constructor-only is the entire applier delta (apply_join already
        // short-circuits AlreadyMember). The auto-invite event is still built and
        // returned; `ops::create_dm_space` still sends a self-targeted invite that
        // the node swallows under DM constraints — an inert residue, by design.
        if invitee != creator {
            // The DM creator (Space owner) is the implicit inviter of the second member.
            pending_invites.insert(
                invitee,
                PendingInvite { role: Role::Member, invited_by: Some(creator.clone()), valid_until: None },
            );
        }

        let mut rooms = HashMap::new();
        rooms.insert(room_id, room);

        let human_pacing_ms = content["human_pacing_ms"]
            .as_u64()
            .unwrap_or(DEFAULT_HUMAN_PACING_MS);
        let ai_pacing_ms = content["ai_pacing_ms"]
            .as_u64()
            .unwrap_or(DEFAULT_AI_PACING_MS);

        let state = SpaceState {
            space_id,
            name: None,
            topic: None,
            auth_tier,
            max_event_size: None,
            home_node,
            owner_id: creator,
            is_dm: true,
            // DM Spaces declare no jurisdiction (AG-D4) — a 1:1 DM is not a
            // government deployment.
            jurisdiction: None,
            e2e_encryption,
            members,
            pending_invites,
            ai_operator_delegations: HashMap::new(),
            banned: HashSet::new(),
            rooms,
            federation_nodes: Vec::new(),
            node_priority_order: Vec::new(),
            dm_constraints_active: true,
            human_pacing_ms,
            ai_pacing_ms,
            member_temperature_visibility: DEFAULT_MEMBER_TEMPERATURE_VISIBILITY.to_string(),
            active_mutes: HashMap::new(),
            threads: HashMap::new(),
        };

        Ok((state, room_event, invite_event))
    }

    /// Initialise a DM SpaceState from a `state.dm_space_create` Event **without
    /// the creator's signing key** — the key-less sibling of [`from_space_create`]
    /// (M7C-D4, J-215). Used by observers that did not author the Space: the Node
    /// ingesting a `state.dm_space_create` (the A3 node-init arm) and a Client
    /// replaying a DM Space's history for a read verb (`ops::members`, A1).
    ///
    /// Unlike [`from_dm_space_create`], this does **not** synthesise the auto-room
    /// or the auto-invite — those arrive as separate creator-signed events
    /// (`state.room_create`, `membership.invite`) and apply through the normal
    /// appliers during replay/ingest. It seeds only the Space owner; `rooms` and
    /// `pending_invites` start empty and are populated by the arriving events.
    ///
    /// The DM `invitee` is seeded into `pending_invites` with the creator as
    /// inviter — exactly as [`from_dm_space_create`] does, and for the same
    /// reason: `apply_invite` rejects invites once `dm_constraints_active` is set
    /// (spec 3.16.1), so the auto-`membership.invite` event cannot populate the
    /// pending invite on replay. Seeding it here lets the invitee's
    /// `membership.join` carry the correct `invited_by` (an observer view that
    /// matches the authoring creator's), instead of falling to `apply_join`'s
    /// `invited_by = None` default. The auto-`state.room_create` is **not**
    /// pre-seeded — it arrives as a separate event and applies cleanly as the
    /// first (only) DM Room.
    pub fn from_dm_space_create_node(event: &Event) -> Result<Self, SpaceError> {
        if event.event_type != EventType::StateDmSpaceCreate {
            return Err(SpaceError::WrongEventType);
        }
        // The event's event_id IS the Space's identifier; cross-flavour wrap
        // EventXgid → SpaceXgid (same hash bytes, different protocol-object role).
        let event_xgid = event.event_id.clone().ok_or(SpaceError::MissingField("event_id"))?;
        let space_id = SpaceXgid::from_xgid(event_xgid.into_xgid());
        let content = &event.content;
        let auth_tier = content["auth_tier"].as_u64().unwrap_or(1) as u32;
        let home_node = NodeXgid::from_xgid(Xgid::new(
            content["home_node"]
                .as_str()
                .ok_or(SpaceError::MissingField("home_node"))?
                .to_string(),
        ));
        let invitee = IdentityXgid::from_xgid(Xgid::new(
            content["invitee"]
                .as_str()
                .ok_or(SpaceError::MissingField("invitee"))?
                .to_string(),
        ));
        // Set-once E2E mode, read uniformly from content (Arc H AH-D2). Unlike
        // `jurisdiction` (which DM Spaces hard-set to `None`), e2e is uniform —
        // no DM exception — so a DM Space honours a declared `e2e_encryption`
        // exactly like a regular Space (default OFF; no current caller declares
        // it for a DM, so it stays false until one does).
        let e2e_encryption = content["e2e_encryption"].as_bool().unwrap_or(false);

        let creator = event.sender.clone();
        let owner = SpaceMember {
            identity_id: creator.clone(),
            role: Role::Owner,
            joined_at: event.timestamp.clone(),
            invited_by: None,
        };
        let mut members = HashMap::new();
        members.insert(creator.clone(), owner);

        // The DM creator (Space owner) is the implicit inviter of the second
        // member (mirrors `from_dm_space_create`; apply_invite can't do this
        // under DM constraints).
        let mut pending_invites = HashMap::new();
        // M11-D1 (self thread): no self-invite when invitee == creator — a self-DM
        // is a clean single-member room by construction (mirror of the client ctor).
        if invitee != creator {
            pending_invites.insert(
                invitee,
                PendingInvite { role: Role::Member, invited_by: Some(creator.clone()), valid_until: None },
            );
        }

        let human_pacing_ms = content["human_pacing_ms"]
            .as_u64()
            .unwrap_or(DEFAULT_HUMAN_PACING_MS);
        let ai_pacing_ms = content["ai_pacing_ms"]
            .as_u64()
            .unwrap_or(DEFAULT_AI_PACING_MS);
        let member_temperature_visibility = content["member_temperature_visibility"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| DEFAULT_MEMBER_TEMPERATURE_VISIBILITY.to_string());

        Ok(SpaceState {
            space_id,
            name: None,
            topic: None,
            auth_tier,
            max_event_size: None,
            home_node,
            owner_id: creator,
            is_dm: true,
            // DM Spaces declare no jurisdiction (AG-D4).
            jurisdiction: None,
            e2e_encryption,
            members,
            pending_invites,
            ai_operator_delegations: HashMap::new(),
            banned: HashSet::new(),
            rooms: HashMap::new(),
            federation_nodes: Vec::new(),
            node_priority_order: Vec::new(),
            dm_constraints_active: true,
            human_pacing_ms,
            ai_pacing_ms,
            member_temperature_visibility,
            active_mutes: HashMap::new(),
            threads: HashMap::new(),
        })
    }

    // ── State machine ─────────────────────────────────────────────────────────

    /// Apply a single Event to this SpaceState.
    /// Called in causal order as Events arrive or are replayed.
    ///
    /// `my_node_id` is the local Node's URI (vantage). Only
    /// `apply_federation_add` consults it today (D-075 vantage-aware applier);
    /// other arms ignore the parameter. Non-Node callers (Client-side replay
    /// for display/state) may pass `""` — the empty string cannot match any
    /// real Node URI, so `apply_federation_add`'s `if content.node_id ==
    /// my_node_id` branch never fires and the applier falls into the else
    /// branch (verbatim pre-D-075 behaviour for non-Node observers).
    pub fn apply_event(&mut self, event: &Event, my_node_id: &str) -> Result<(), SpaceError> {
        match &event.event_type {
            EventType::StateRoomCreate => self.apply_room_create(event),
            EventType::StateFederationAdd => self.apply_federation_add(event, my_node_id),
            EventType::MembershipInvite => self.apply_invite(event),
            EventType::MembershipJoin => self.apply_join(event),
            EventType::MembershipLeave => self.apply_leave(event),
            EventType::MembershipKick => self.apply_kick(event),
            EventType::MembershipBan => self.apply_ban(event),
            EventType::MembershipNodeEject => self.apply_node_eject(event),
            EventType::MembershipNodeUnban => self.apply_node_unban(event),
            // Phase 2: update manual Node priority ordering.
            EventType::StateNodePriority => self.apply_node_priority(event),
            // Phase 2: DM Space promotion — lifts DM constraints and sets the space name.
            EventType::StateDmPromote => self.apply_dm_promote(event),
            // Phase 2: owner updates per-Space pacing rules (3.7.12).
            EventType::StateSpacePacing => self.apply_space_pacing(event),
            // Phase 2: owner updates temperature visibility (3.7.13.3).
            EventType::StateSpaceTemperatureVisibility => self.apply_space_temperature_visibility(event),
            // Phase 2: moderator-or-higher mutes a member (3.7.8).
            EventType::MembershipMute => self.apply_mute(event),
            // M3: AI operator delegation (3.6.10.6). Owner/admin only.
            EventType::StateAiOperatorDelegate => self.apply_ai_operator_delegate(event),
            // M3: AI operator revoke (3.6.10.6). Owner/admin only.
            EventType::StateAiOperatorRevoke => self.apply_ai_operator_revoke(event),
            // PG-12-min (Arc D, PM-D3) — state.room_update carries per-Room
            // permission overrides. Given a real applier here (it was an inert
            // SR-F2 no-op); already state-keyed per room, so M8-convergent for
            // free. Room name/topic content stays deferred (SR-F2 remainder).
            EventType::StateRoomUpdate => self.apply_room_update(event),
            // state.space_update remains the SR-F2 no-op (no content schema yet).
            EventType::StateSpaceUpdate => Ok(()),
            // Arc E (PG-08) — Thread lifecycle. Create inserts an Open Thread;
            // resolved/archived set the terminal status (idempotent,
            // convergence-clean via the shared `thread.status` state key).
            EventType::ThreadCreate => self.apply_thread_create(event),
            EventType::ThreadResolved => self.apply_thread_status(event, ThreadStatus::Resolved),
            EventType::ThreadArchived => self.apply_thread_status(event, ThreadStatus::Archived),
            // Arc F (PG-11) — Space Migration cutover (AF-D1/D2). The Node-authored
            // `state.space_migrate` flips the authority anchor `home_node`
            // source→dest. No `state_key_for_event` arm (it returns `None` for
            // migration.* — a causally-terminal singleton, not LWW-keyed).
            EventType::StateSpaceMigrate => self.apply_space_migrate(event),
            // Arc H (PG-05, AH-D3) — MLS group genesis anchor. Sets the target
            // Room's `mls_epoch` to genesis 0 (Node-readable; no key material).
            // Idempotent; unknown-Room no-op. Epoch advances ride AH-D4 (C2).
            EventType::MlsGroupInit => self.apply_mls_group_init(event),
            // Arc H (PG-05, AH-D4) — MLS epoch advance. A client emits `mls.commit`
            // after a resolved membership change in an E2E Space; the Node tracks
            // the new epoch on the Room (opaque counter, no key material).
            EventType::MlsCommit => self.apply_mls_commit(event),
            // PG-09 / FC-D6 — the apply chokepoint. An unrecognised event type is
            // stored + relayed but NEVER applied: no membership/permission/
            // temperature/pacing state mutates. Explicit (not folded into `_`)
            // so the forward-compat no-op is visible at the dispatch.
            EventType::Unknown(_) => Ok(()),
            _ => Ok(()), // unrecognised known events silently ignored
        }
    }

    fn apply_dm_promote(&mut self, event: &Event) -> Result<(), SpaceError> {
        let new_name = event.content["new_name"]
            .as_str()
            .ok_or(SpaceError::MissingField("new_name"))?;
        self.name = Some(new_name.to_string());
        self.dm_constraints_active = false;
        Ok(())
    }

    fn apply_federation_add(
        &mut self,
        event: &Event,
        my_node_id: &str,
    ) -> Result<(), SpaceError> {
        if self.dm_constraints_active {
            return Err(SpaceError::DmFederationNotAllowed);
        }
        let content_node_id = event.content["node_id"]
            .as_str()
            .ok_or(SpaceError::MissingField("node_id"))?;
        // D-075 vantage-aware applier (locked at bidirectional federation_nodes
        // design phase 2026-05-21; see tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md §4.1).
        //
        // state.federation_add records one party's act: "asserter (sender)
        // approves other-party (content.node_id) as federation peer for this
        // Space." The event is asymmetric by construction. The applier
        // reconstructs the relevant peer from local vantage:
        //   - If I am content.node_id (someone else's federation_add naming
        //     me), the relevant peer to add is event.sender (the asserter).
        //   - Else (my own federation_add naming someone else, OR an
        //     unrelated federation_add I'm observing as a third-party with
        //     multi-Space visibility), the relevant peer is content.node_id.
        //
        // Both branches are needed: A authors a federation_add(B); both A and
        // B ingest it. A falls into the else branch (content.node_id=B,
        // my=A); B falls into the if branch (content.node_id=B, my=B). Both
        // end with the other party in federation_nodes; symmetric outcome
        // through asymmetric branches, driven by my_node_id.
        //
        // Sibling derivation: `NodeRuntime::dispatch_event` Step 7's
        // `fed_add_drain_pair` (xgen-core/src/node/runtime.rs ~:631) uses
        // the same vantage rule to key the federation-relationship drain
        // hook. The two sites MUST stay aligned; drift produces buffered
        // events that never drain. Touch one → check the other.
        // Pass 1 retypes federation_nodes to Vec<NodeXgid>; the peer derivation
        // produces &str (one branch from event.sender via Deref, the other from
        // content JSON), then wraps into NodeXgid at the boundary. Pass 2
        // widens callers + content extraction to typed XGIDs; the wrap collapses.
        let peer_to_add: NodeXgid = if content_node_id == my_node_id {
            NodeXgid::from_xgid(Xgid::new(event.sender.as_str().to_string()))
        } else {
            NodeXgid::from_xgid(Xgid::new(content_node_id.to_string()))
        };
        if !self.federation_nodes.contains(&peer_to_add) {
            self.federation_nodes.push(peer_to_add);
        }
        Ok(())
    }

    fn apply_node_priority(&mut self, event: &Event) -> Result<(), SpaceError> {
        let ordered_nodes = event.content["ordered_nodes"]
            .as_array()
            .ok_or(SpaceError::MissingField("ordered_nodes"))?;
        // Pass 2 widens content extraction to typed XGIDs; the wrap collapses then.
        self.node_priority_order = ordered_nodes
            .iter()
            .filter_map(|v| v.as_str().map(|s| NodeXgid::from_xgid(Xgid::new(s.to_string()))))
            .collect();
        Ok(())
    }

    /// Apply a `state.space_pacing` Event (spec 3.7.12.3).
    /// Only the Space owner may update pacing; both fields are required.
    fn apply_space_pacing(&mut self, event: &Event) -> Result<(), SpaceError> {
        if event.sender != self.owner_id {
            return Err(SpaceError::PermissionDenied(
                "state.space_pacing requires owner".to_string(),
            ));
        }
        let human = event.content["human_pacing_ms"]
            .as_u64()
            .ok_or(SpaceError::MissingField("human_pacing_ms"))?;
        let ai = event.content["ai_pacing_ms"]
            .as_u64()
            .ok_or(SpaceError::MissingField("ai_pacing_ms"))?;
        self.human_pacing_ms = human;
        self.ai_pacing_ms = ai;
        Ok(())
    }

    /// Apply a `state.space_temperature_visibility` Event (spec 3.7.13.3).
    /// Only the Space owner may update; value is stored verbatim (open enum),
    /// but unknown values are treated as `moderator` by the visibility filter.
    fn apply_space_temperature_visibility(&mut self, event: &Event) -> Result<(), SpaceError> {
        if event.sender != self.owner_id {
            return Err(SpaceError::PermissionDenied(
                "state.space_temperature_visibility requires owner".to_string(),
            ));
        }
        let value = event.content["member_temperature_visibility"]
            .as_str()
            .ok_or(SpaceError::MissingField("member_temperature_visibility"))?;
        self.member_temperature_visibility = value.to_string();
        Ok(())
    }

    /// Apply a `membership.mute` Event (spec 3.7.8).
    /// Permitted from moderator-or-higher. The `reason` value is accepted as
    /// free text; the reserved `auto_temperature` value (3.7.13.6) follows the
    /// standard mute logic with no additional protocol behaviour beyond audit.
    fn apply_mute(&mut self, event: &Event) -> Result<(), SpaceError> {
        let actor = &event.sender;
        let actor_role = self.member_role(actor.as_str()).ok_or(SpaceError::NotASpaceMember)?;
        if !can_mute(actor_role) {
            return Err(SpaceError::PermissionDenied(actor_role.as_str().to_string()));
        }
        // Pass 2 widens content extraction to typed XGIDs; the wrap collapses then.
        let target = IdentityXgid::from_xgid(Xgid::new(
            event.content["target_identity"]
                .as_str()
                .ok_or(SpaceError::MissingField("target_identity"))?
                .to_string(),
        ));
        let cooldown_until = event.content["cooldown_until"]
            .as_str()
            .ok_or(SpaceError::MissingField("cooldown_until"))?
            .to_string();
        self.active_mutes.insert(target, cooldown_until);
        Ok(())
    }

    fn apply_room_create(&mut self, event: &Event) -> Result<(), SpaceError> {
        if self.dm_constraints_active && !self.rooms.is_empty() {
            return Err(SpaceError::DmSecondRoomNotAllowed);
        }
        let actor = &event.sender;
        let actor_role = self.member_role(actor.as_str()).ok_or(SpaceError::NotASpaceMember)?;
        if !can_create_room(actor_role) {
            return Err(SpaceError::PermissionDenied(actor_role.as_str().to_string()));
        }
        // Cross-flavour wrap EventXgid → RoomXgid (room-create event's event_id IS the Room's identifier).
        let event_xgid = event.event_id.clone().ok_or(SpaceError::MissingField("event_id"))?;
        let room_id = RoomXgid::from_xgid(event_xgid.into_xgid());
        let name = event.content["name"].as_str().unwrap_or("room").to_string();
        let topic = event.content["topic"].as_str().map(str::to_string);
        let mut members = HashSet::new();
        members.insert(actor.clone());
        self.rooms.insert(
            room_id.clone(),
            RoomState {
                room_id,
                space_id: self.space_id.clone(),
                name,
                topic,
                members,
                permission_overrides: HashMap::new(),
                // Genesis MLS epoch is set by a later state.mls_group_init in an
                // E2E Space (AH-D3); a freshly-created Room has none.
                mls_epoch: None,
                // No commit has won resolution yet (M8.7 CC-D5); set by the first
                // resolved `mls.commit` via `apply_mls_commit`.
                mls_commit_tip: None,
            },
        );
        Ok(())
    }

    /// Apply a `state.mls_group_init` Event (Arc H PG-05, AH-D3): anchor the MLS
    /// group genesis for the target Room at epoch 0. The Node-readable epoch
    /// counter only — no key material lives in `SpaceState` (§3.10.1). Idempotent
    /// (re-applying genesis re-sets `Some(0)`); an unknown Room is a silent no-op
    /// (the group-init's `prev_events` causally follow the Room's create, so in a
    /// well-formed DAG the Room is always present by the time this applies). A
    /// `state_key_for_event` arm keys this per-Room so a concurrent re-init
    /// cannot fork (M8 resolves it to one).
    fn apply_mls_group_init(&mut self, event: &Event) -> Result<(), SpaceError> {
        let room_id = RoomXgid::from_xgid(Xgid::new(event.room_id.as_str().to_string()));
        if let Some(room) = self.rooms.get_mut(&room_id) {
            room.mls_epoch = Some(0);
        }
        Ok(())
    }

    /// Apply an `mls.commit` Event (Arc H PG-05, AH-D4 / M8.7 CC-D5): advance the
    /// target Room's tracked MLS epoch to the value declared in the commit, and
    /// record the commit as the Room's canonical commit tip. Node-side this is the
    /// opaque epoch *counter* + the commit *identity* only — the Commit's MLS
    /// payload is never inspected (the DS routes it opaque; §3.10.1). Unknown Room
    /// or absent `epoch` ⇒ silent no-op.
    ///
    /// **The concurrent commit-race is resolved by the `(room, target_epoch)`
    /// conflict domain (M8.7 CC-D2 — `state_key_for_event`'s `MlsCommit` arm).**
    /// Two members committing the same `N → N+1` advance at one frontier are a
    /// genuine state-key conflict; `derive_resolved` admits only the Layer-5c
    /// lexicographic winner to the applied set (CC-D3), so this applier runs for
    /// the winner and the loser is excluded — every federated node converges on
    /// the same `(mls_epoch, mls_commit_tip)`. (The counter alone is vacuous: two
    /// honest commits both land `N+1` under either fold order; the tip records
    /// *which* commit is canonical — CC-D5.) The loser client's rollback-and-
    /// replay (rebuild from the winner, re-commit → epoch N+2) is the production
    /// openmls client's job (L) — R proves convergence-on-winner, not loser
    /// recovery.
    fn apply_mls_commit(&mut self, event: &Event) -> Result<(), SpaceError> {
        let room_id = RoomXgid::from_xgid(Xgid::new(event.room_id.as_str().to_string()));
        if let Some(epoch) = event.content["epoch"].as_u64() {
            if let Some(room) = self.rooms.get_mut(&room_id) {
                room.mls_epoch = Some(epoch);
                room.mls_commit_tip = event.event_id.clone();
            }
        }
        Ok(())
    }

    /// Apply a `state.room_update` carrying per-Room permission overrides
    /// (PG-12-min, Arc D PM-D3). **Replace semantics**: when the event carries a
    /// `permission_overrides` array, it is the Room's *complete* override set and
    /// the prior set is rebuilt wholesale (an empty array clears all overrides).
    /// When the key is absent the event is not about overrides (e.g. a future
    /// name/topic room_update, still deferred) and the set is left untouched.
    /// Unknown role/permission/effect strings skip that entry (forward-compat).
    /// Unknown target room → no-op (silent-accept, sibling to the other state
    /// arms). Authoring authority is gated upstream in `check_permission`'s
    /// existing `StateRoomUpdate` arm (Admin+), so no authority check is needed
    /// here — the applier runs only on already-validated events.
    fn apply_room_update(&mut self, event: &Event) -> Result<(), SpaceError> {
        let arr = match event.content.get("permission_overrides").and_then(Value::as_array) {
            Some(a) => a,
            None => return Ok(()),
        };
        let room = match self.rooms.get_mut(&event.room_id) {
            Some(r) => r,
            None => return Ok(()),
        };
        let mut overrides: HashMap<(Role, RoomPermission), Effect> = HashMap::new();
        for entry in arr {
            let role = entry.get("role").and_then(Value::as_str).and_then(Role::from_str);
            let perm = entry
                .get("permission")
                .and_then(Value::as_str)
                .and_then(RoomPermission::from_str);
            let effect = entry.get("effect").and_then(Value::as_str).and_then(Effect::from_str);
            if let (Some(role), Some(perm), Some(effect)) = (role, perm, effect) {
                overrides.insert((role, perm), effect);
            }
        }
        room.permission_overrides = overrides;
        Ok(())
    }

    /// Apply a `thread.create` (Arc E PG-08, AE-D7). Derives the conceptual
    /// Thread id from the create event's canonical hash (AE-D8) and inserts an
    /// `Open` ThreadState. Authority and tier are validated upstream
    /// (validate_event step 11 enforces Room membership since the event carries
    /// `room_id`; dispatch step 4 enforces narrow-not-widen + the AE-D9
    /// participation gate), so the applier itself is permissive — it runs only on
    /// already-validated events (sibling to `apply_room_update`).
    fn apply_thread_create(&mut self, event: &Event) -> Result<(), SpaceError> {
        let event_xgid = event.event_id.clone().ok_or(SpaceError::MissingField("event_id"))?;
        let origin = event_xgid.as_str().to_string();
        let thread_id = thread_id_from_event_id(&origin);
        let auth_tier_min = event.content["auth_tier_min"].as_u64().unwrap_or(1) as u32;
        let title = event.content["title"].as_str().map(str::to_string);
        self.threads.insert(
            thread_id.clone(),
            ThreadState {
                id: thread_id,
                room_id: event.room_id.clone(),
                created_by: event.sender.clone(),
                created_at: event.timestamp.clone(),
                title,
                status: ThreadStatus::Open,
                auth_tier_min,
                origin_event: origin,
            },
        );
        Ok(())
    }

    /// Apply a `thread.resolved` / `thread.archived` (Arc E PG-08). Looks up the
    /// Thread by `content["thread"]` and sets its terminal status. **Idempotent**
    /// (re-applying the same transition is a no-op) and **convergence-clean**: a
    /// concurrent resolved-vs-archived pair shares the `thread.status` state key
    /// (`state_key.rs`), so M8 `derive_resolved` applies only the resolved winner.
    /// An unknown thread id is a silent no-op (sibling to the other state arms;
    /// the applier runs only on already-validated events).
    fn apply_thread_status(
        &mut self,
        event: &Event,
        status: ThreadStatus,
    ) -> Result<(), SpaceError> {
        let thread_id = match event.content["thread"].as_str() {
            Some(t) => t,
            None => return Ok(()),
        };
        if let Some(thread) = self.threads.get_mut(thread_id) {
            thread.status = status;
        }
        Ok(())
    }

    fn apply_invite(&mut self, event: &Event) -> Result<(), SpaceError> {
        if self.dm_constraints_active {
            return Err(SpaceError::DmInvitationNotAllowed);
        }
        let actor = &event.sender;
        let actor_role = self.member_role(actor.as_str()).ok_or(SpaceError::NotASpaceMember)?;
        if !can_invite(actor_role) {
            return Err(SpaceError::PermissionDenied(actor_role.as_str().to_string()));
        }
        // Pass 2 widens content extraction to typed XGIDs; the wrap collapses then.
        let target = IdentityXgid::from_xgid(Xgid::new(
            event.content["target_identity"]
                .as_str()
                .ok_or(SpaceError::MissingField("target_identity"))?
                .to_string(),
        ));
        if self.banned.contains(&target) {
            return Err(SpaceError::Banned);
        }
        let role_str = event.content["role"].as_str().unwrap_or("member");
        let role = Role::from_str(role_str).unwrap_or(Role::Member);
        // M8.5-B (INV-D6): carry the invite's validity deadline (absolute
        // timestamp). Deterministic from content → convergence-neutral. Absent
        // ⇒ no expiry (the join/read gates only bite on a present, past value).
        let valid_until = event.content["valid_until"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        // M3: capture the inviter so resolve_operator can fall back to it.
        self.pending_invites.insert(
            target,
            PendingInvite { role, invited_by: Some(actor.clone()), valid_until },
        );
        Ok(())
    }

    fn apply_join(&mut self, event: &Event) -> Result<(), SpaceError> {
        let joiner = &event.sender;

        // Room-level join: room_id is non-empty.
        if !event.room_id.as_str().is_empty() {
            // Joiner must already be a Space member.
            if !self.members.contains_key(joiner) {
                return Err(SpaceError::NotASpaceMember);
            }
            let room = self.rooms.get_mut(&event.room_id).ok_or(SpaceError::RoomNotFound)?;
            if room.members.contains(joiner) {
                return Err(SpaceError::AlreadyMember);
            }
            room.members.insert(joiner.clone());
            return Ok(());
        }

        // Space-level join: room_id is empty.
        if self.members.contains_key(joiner) {
            return Err(SpaceError::AlreadyMember);
        }
        if self.banned.contains(joiner) {
            return Err(SpaceError::Banned);
        }
        let (role, invited_by) = match self.pending_invites.remove(joiner) {
            Some(pi) => (pi.role, pi.invited_by),
            None => (Role::Member, None),
        };
        self.members.insert(
            joiner.clone(),
            SpaceMember {
                identity_id: joiner.clone(),
                role,
                joined_at: event.timestamp.clone(),
                invited_by,
            },
        );
        Ok(())
    }

    fn apply_leave(&mut self, event: &Event) -> Result<(), SpaceError> {
        let leaver = &event.sender;
        if !event.room_id.as_str().is_empty() {
            // Room-level leave.
            let room = self.rooms.get_mut(&event.room_id).ok_or(SpaceError::RoomNotFound)?;
            room.members.remove(leaver);
            return Ok(());
        }
        if self.members.remove(leaver).is_none() {
            return Err(SpaceError::NotASpaceMember);
        }
        for room in self.rooms.values_mut() {
            room.members.remove(leaver);
        }
        Ok(())
    }

    fn apply_kick(&mut self, event: &Event) -> Result<(), SpaceError> {
        let actor = &event.sender;
        let actor_role = self.member_role(actor.as_str()).ok_or(SpaceError::NotASpaceMember)?;
        if !can_kick(actor_role) {
            return Err(SpaceError::PermissionDenied(actor_role.as_str().to_string()));
        }
        // Pass 2 widens content extraction to typed XGIDs; the wrap collapses then.
        let target = IdentityXgid::from_xgid(Xgid::new(
            event.content["target_identity"]
                .as_str()
                .ok_or(SpaceError::MissingField("target_identity"))?
                .to_string(),
        ));
        if !event.room_id.as_str().is_empty() {
            let room = self.rooms.get_mut(&event.room_id).ok_or(SpaceError::RoomNotFound)?;
            room.members.remove(&target);
            return Ok(());
        }
        self.members.remove(&target);
        for room in self.rooms.values_mut() {
            room.members.remove(&target);
        }
        Ok(())
    }

    fn apply_ban(&mut self, event: &Event) -> Result<(), SpaceError> {
        let actor = &event.sender;
        let actor_role = self.member_role(actor.as_str()).ok_or(SpaceError::NotASpaceMember)?;
        if !can_ban(actor_role) {
            return Err(SpaceError::PermissionDenied(actor_role.as_str().to_string()));
        }
        // Pass 2 widens content extraction to typed XGIDs; the wrap collapses then.
        let target = IdentityXgid::from_xgid(Xgid::new(
            event.content["target_identity"]
                .as_str()
                .ok_or(SpaceError::MissingField("target_identity"))?
                .to_string(),
        ));
        self.members.remove(&target);
        self.pending_invites.remove(&target);
        self.banned.insert(target.clone());
        for room in self.rooms.values_mut() {
            room.members.remove(&target);
        }
        Ok(())
    }

    /// M6 A4-D1: Node-administrator force-eject. Authority is **Node** (sender is
    /// the Space's home Node), not a member role — validate_event gates this
    /// (`sender == home_node`); re-checked here defensively, mirroring
    /// apply_kick/apply_ban's in-apply authority check. Effect: remove the target
    /// from the Space (+ all rooms + pending invites) AND ban them (1A: reversible
    /// via `membership.node_unban`).
    fn apply_node_eject(&mut self, event: &Event) -> Result<(), SpaceError> {
        if event.sender.as_str() != self.home_node.as_str() {
            return Err(SpaceError::PermissionDenied("membership.node_eject".to_string()));
        }
        let target = IdentityXgid::from_xgid(Xgid::new(
            event.content["target_identity"]
                .as_str()
                .ok_or(SpaceError::MissingField("target_identity"))?
                .to_string(),
        ));
        self.members.remove(&target);
        self.pending_invites.remove(&target);
        self.banned.insert(target.clone());
        for room in self.rooms.values_mut() {
            room.members.remove(&target);
        }
        Ok(())
    }

    /// M6 A4-D1: the reversible counterpart to `apply_node_eject` — lifts the ban
    /// (allows rejoin). Does NOT auto-restore membership; the target rejoins
    /// normally. Same Node-authority gate.
    fn apply_node_unban(&mut self, event: &Event) -> Result<(), SpaceError> {
        if event.sender.as_str() != self.home_node.as_str() {
            return Err(SpaceError::PermissionDenied("membership.node_unban".to_string()));
        }
        let target = IdentityXgid::from_xgid(Xgid::new(
            event.content["target_identity"]
                .as_str()
                .ok_or(SpaceError::MissingField("target_identity"))?
                .to_string(),
        ));
        self.banned.remove(&target);
        Ok(())
    }

    /// Arc F (PG-11) — Space Migration cutover applier (AF-D1/D2). Flips the
    /// authority anchor `home_node` from the source Node to the destination Node.
    ///
    /// The `state.space_migrate` event is Node-authored: `sender` is the SOURCE
    /// home Node keypair (`build_space_migrate_event`). It validates under the
    /// *old* `home_node` upstream (validate-before-apply ordering, AF-D1), then
    /// this applier installs the new anchor.
    ///
    /// **Idempotent (AF-D1):** a re-apply when `home_node` is already the
    /// destination is a no-op — checked *before* the authority gate so a legitimate
    /// `derive_resolved` replay (which re-folds the cutover) never trips it.
    ///
    /// **Self-protecting authority transfer (AF-D2):** the defensive
    /// `sender == home_node` gate (mirroring `apply_node_eject`) means that once
    /// the anchor has flipped, the old source is no longer `home_node`, so a
    /// second/competing source-signed migrate fails here by construction. Only the
    /// *current* home Node can migrate onward. No conflict-resolution machinery is
    /// needed — the gate is the protection (causally-terminal singleton).
    fn apply_space_migrate(&mut self, event: &Event) -> Result<(), SpaceError> {
        let destination = NodeXgid::from_xgid(Xgid::new(
            event.content["destination_node_id"]
                .as_str()
                .ok_or(SpaceError::MissingField("destination_node_id"))?
                .to_string(),
        ));
        // Idempotent no-op: already migrated to this destination.
        if self.home_node == destination {
            return Ok(());
        }
        // AF-D2 authority gate: only the current home Node may migrate the Space.
        if event.sender.as_str() != self.home_node.as_str() {
            return Err(SpaceError::PermissionDenied("state.space_migrate".to_string()));
        }
        self.home_node = destination;
        Ok(())
    }

    /// Apply a `state.ai_operator_delegate` Event (spec 3.6.10.6).
    ///
    /// Signer must be owner or admin (re-checked here as defence-in-depth for
    /// replay paths that bypass `validate_steps_8_13`). The target validity
    /// checks — `ai_identity_id` is a Space member with `is_ai = true`, and
    /// `new_operator_identity_id` is a Space member — are owned by the upstream
    /// `exchange.rs` pipeline because the registry is not available here.
    fn apply_ai_operator_delegate(&mut self, event: &Event) -> Result<(), SpaceError> {
        let actor = &event.sender;
        let actor_role = self.member_role(actor.as_str()).ok_or(SpaceError::NotASpaceMember)?;
        if !crate::space::membership::can_delegate_ai_operator(actor_role) {
            return Err(SpaceError::PermissionDenied(actor_role.as_str().to_string()));
        }
        // Pass 2 widens content extraction to typed XGIDs; the wraps collapse then.
        let ai_id = IdentityXgid::from_xgid(Xgid::new(
            event.content["ai_identity_id"]
                .as_str()
                .ok_or(SpaceError::MissingField("ai_identity_id"))?
                .to_string(),
        ));
        let new_op = IdentityXgid::from_xgid(Xgid::new(
            event.content["new_operator_identity_id"]
                .as_str()
                .ok_or(SpaceError::MissingField("new_operator_identity_id"))?
                .to_string(),
        ));
        self.ai_operator_delegations.insert(ai_id, new_op);
        Ok(())
    }

    /// Apply a `state.ai_operator_revoke` Event (spec 3.6.10.6).
    ///
    /// Same signer-role defence-in-depth as delegate. After this returns
    /// successfully, `resolve_operator` falls through to step 2 (inviter)
    /// or step 3 (owner).
    fn apply_ai_operator_revoke(&mut self, event: &Event) -> Result<(), SpaceError> {
        let actor = &event.sender;
        let actor_role = self.member_role(actor.as_str()).ok_or(SpaceError::NotASpaceMember)?;
        if !crate::space::membership::can_delegate_ai_operator(actor_role) {
            return Err(SpaceError::PermissionDenied(actor_role.as_str().to_string()));
        }
        // Pass 2 widens content extraction to typed XGIDs; the wrap collapses then.
        let ai_id = IdentityXgid::from_xgid(Xgid::new(
            event.content["ai_identity_id"]
                .as_str()
                .ok_or(SpaceError::MissingField("ai_identity_id"))?
                .to_string(),
        ));
        self.ai_operator_delegations.remove(&ai_id);
        Ok(())
    }

    /// Resolve the current operator for an AI Identity in this Space
    /// (spec 3.6.10.6, M3 architecture lock 2026-05-16).
    ///
    /// Fall-upward algorithm:
    ///   1. If a stored delegation exists AND the delegate is a current member: return it.
    ///   2. Else if the AI's `invited_by` is a current member: return it.
    ///   3. Else: return the Space owner (always a member of a live Space).
    ///
    /// Returns `None` only when `ai_id` is not a member of this Space, or in
    /// the structural-bug case of an owner who has somehow left. Callers should
    /// not call this for non-AI Identities — the function has no way to verify
    /// the AI flag (no registry access here), so the contract is "caller knows
    /// `ai_id` is an AI member".
    pub fn resolve_operator(&self, ai_id: &str) -> Option<String> {
        // Pass 2 widens this method to take `&IdentityXgid` and return `Option<IdentityXgid>`;
        // the wraps + projections collapse then.
        let ai_xgid = IdentityXgid::from_xgid(Xgid::new(ai_id.to_string()));
        if !self.members.contains_key(&ai_xgid) {
            return None;
        }
        // Step 1 — stored delegation, if delegate is still a Space member.
        if let Some(delegate) = self.ai_operator_delegations.get(&ai_xgid) {
            if self.members.contains_key(delegate) {
                return Some(delegate.as_str().to_string());
            }
        }
        // Step 2 — recorded inviter, if still a Space member.
        if let Some(member) = self.members.get(&ai_xgid) {
            if let Some(inviter) = &member.invited_by {
                if self.members.contains_key(inviter) {
                    return Some(inviter.as_str().to_string());
                }
            }
        }
        // Step 3 — Space owner (defensive `contains_key`; in a live Space the
        // owner is always a member).
        if self.members.contains_key(&self.owner_id) {
            return Some(self.owner_id.as_str().to_string());
        }
        None
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    pub fn member_role(&self, identity_id: &str) -> Option<&Role> {
        // Pass 2 widens this method to take `&IdentityXgid`; the wrap collapses then.
        self.members
            .get(&IdentityXgid::from_xgid(Xgid::new(identity_id.to_string())))
            .map(|m| &m.role)
    }

    pub fn is_member(&self, identity_id: &str) -> bool {
        // Pass 2 widens this method to take `&IdentityXgid`; the wrap collapses then.
        self.members
            .contains_key(&IdentityXgid::from_xgid(Xgid::new(identity_id.to_string())))
    }

    pub fn is_room_member(&self, identity_id: &str, room_id: &str) -> bool {
        // Pass 2 widens this method to take typed XGIDs; the wraps collapse then.
        let id_key = IdentityXgid::from_xgid(Xgid::new(identity_id.to_string()));
        let room_key = RoomXgid::from_xgid(Xgid::new(room_id.to_string()));
        self.rooms
            .get(&room_key)
            .map(|r| r.members.contains(&id_key))
            .unwrap_or(false)
    }
}

// ── Event signing ─────────────────────────────────────────────────────────────

/// Sign an Event: compute event_id (hash of canonical form) and add signature.
pub fn sign_event(mut event: Event, key: &SigningKey) -> Event {
    let v = serde_json::to_value(&event).expect("Event is always serialisable");
    let bytes = canonical_event_bytes(&v);
    // `hashing::hash_uri` returns a String URI; wrap as typed EventXgid for
    // the event envelope. Pass 2 widens `hash_uri` to return EventXgid; the
    // wrap collapses then.
    event.event_id = Some(EventXgid::from_xgid(Xgid::new(hashing::hash_uri(&bytes))));
    event.signature = Some(signing::sign(key, &bytes));
    event
}

/// Verify an Event's signature against the embedded public key in `sender`.
pub fn verify_event_signature(event: &Event) -> bool {
    let sig_str = match &event.signature {
        Some(s) => s,
        None => return false,
    };
    let sender_b64 = match event.sender.as_str().strip_prefix("xgen://pubkey/ed25519:") {
        Some(b) => b,
        None => return false,
    };
    let key_bytes = match encoding::decode(sender_b64) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let arr: [u8; 32] = match key_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let vk = match ed25519_dalek::VerifyingKey::from_bytes(&arr) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let v = serde_json::to_value(event).expect("Event is serialisable");
    let bytes = canonical_event_bytes(&v);
    signing::verify(&vk, &bytes, sig_str).is_ok()
}

// ── Event builders ────────────────────────────────────────────────────────────

fn generate_nonce() -> String {
    let mut bytes = [0u8; 16];
    OsRng.fill_bytes(&mut bytes);
    encoding::encode(&bytes)
}

/// Project a `Vec<String>` of legacy prev_events URIs into a typed
/// `Vec<EventXgid>` for `Event::new`. Pass 2 widens the `prev_events: Vec<String>`
/// parameter on build_*_event functions to typed XGIDs; this helper collapses then.
fn prev_events_to_xgids(prev: Vec<String>) -> Vec<EventXgid> {
    prev.into_iter()
        .map(|s| EventXgid::from_xgid(Xgid::new(s)))
        .collect()
}

/// Build the sender URI string for an Identity-signing keypair (also used
/// for state.dm_promote and other Node-signed events — Event.sender is typed
/// `IdentityXgid` at v1 and does not distinguish Identity-signed from
/// Node-signed senders; D-072 accepted asymmetry).
fn sender_id(key: &SigningKey) -> String {
    format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(key.verifying_key().as_bytes())
    )
}

/// Typed variant of `sender_id` for Event::new construction. Internal helper —
/// keeps build_*_event call sites tight without forcing tests onto typed sender
/// projections. Pass 2 may widen `sender_id` itself; this helper folds in then.
fn sender_xgid(key: &SigningKey) -> IdentityXgid {
    IdentityXgid::from_xgid(Xgid::new(sender_id(key)))
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Build an unsigned `state.space_create` Event.
pub fn build_space_create_event(
    key: &SigningKey,
    name: &str,
    topic: Option<&str>,
    auth_tier: u32,
    home_node: &str,
    jurisdiction: Option<&str>,
    e2e_encryption: bool,
) -> Event {
    let mut content = json!({
        "name": name,
        "auth_tier": auth_tier,
        "nonce": generate_nonce(),
        "home_node": home_node,
    });
    if let Some(t) = topic {
        content["topic"] = json!(t);
    }
    // Set-once jurisdiction (Arc G PG-04, AG-D4) — write the key only when
    // declared, mirroring `topic`. Absent key ⇒ undeclared.
    if let Some(j) = jurisdiction {
        content["jurisdiction"] = json!(j);
    }
    // Set-once E2E mode (Arc H PG-05, AH-D2) — write the key only when ON, so
    // non-E2E Spaces (the default) stay byte-for-byte as before (M8/back-compat
    // clean). Absent key ⇒ OFF, read by the constructors as `false`.
    if e2e_encryption {
        content["e2e_encryption"] = json!(true);
    }
    Event::new(
        EventType::StateSpaceCreate,
        sender_xgid(key),
        empty_room_xgid(),
        empty_space_xgid(), // derived to event_id post-sign
        vec![],
        now(),
        content,
    )
}

/// Build an unsigned `state.room_create` Event.
/// `space_id` is the event_id of the parent `state.space_create`.
/// Derive the conceptual Thread id from a `thread.create` event id (Arc E PG-08,
/// AE-D8): the same SHA-256 digest re-prefixed `xgen://hash/sha256:` →
/// `xgen://thread/sha256:`. A Thread has no `ThreadXgid` flavour — the id is a
/// plain URI string. If the input lacks the hash prefix it is returned unchanged
/// (defensive — accepted events always carry a hash-prefixed event_id).
pub fn thread_id_from_event_id(event_id: &str) -> String {
    match event_id.strip_prefix("xgen://hash/sha256:") {
        Some(hex) => format!("xgen://thread/sha256:{hex}"),
        None => event_id.to_string(),
    }
}

pub fn build_room_create_event(
    key: &SigningKey,
    space_id: &str,
    name: &str,
    topic: Option<&str>,
) -> Event {
    let mut content = json!({
        "name": name,
        "nonce": generate_nonce(),
    });
    if let Some(t) = topic {
        content["topic"] = json!(t);
    }
    Event::new(
        EventType::StateRoomCreate,
        sender_xgid(key),
        empty_room_xgid(), // derived to event_id post-sign
        // Pass 2 widens the `space_id: &str` parameter to `&SpaceXgid`; the wrap collapses then.
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),

        // D-076 v1.1 causal-DAG-respecting order (locked at topological-sort
        // design-phase re-walk Step 2, 2026-05-22, per
        // tasks/FEDERATION_TOPOSORT_DESIGN.md §11 — Path B at event-construction
        // layer).
        //
        // The function's doc-comment above already claims `space_id` is the
        // event_id of the parent state.space_create; the construction is now
        // honest about it. Pre-fix code set this argument to vec![] which
        // produced a DAG-root semantic at the event-DAG layer while the
        // protocol-level parent-child relationship remained tacit in the
        // doc-comment only. Two DAG roots tied at the top of
        // topological_sort_events with event-id-based tie-break placed
        // state.room_create before its protocol-level parent state.space_create
        // in roughly half of nonce rolls, causing receivers to reject the child
        // with "space not found" before the parent landed.
        //
        // Narrow-scope note: this fix is scoped to build_room_create_event only.
        // Sibling event constructors (state.federation_add, membership.*,
        // message.*, etc.) may carry similar prev_events lies; they are NOT
        // audited at this milestone. If dependent work surfaces need, a future
        // audit arc per D-071 covers them.
        //
        // Pass 1 Commit 4 retype: the predecessor entry is typed EventXgid
        // (Pass 2 widens the `space_id: &str` parameter end-to-end). Underlying
        // hash bytes are unchanged.
        vec![EventXgid::from_xgid(Xgid::new(space_id.to_string()))],

        now(),
        content,
    )
}

/// Build an unsigned `state.mls_group_init` Event (Arc H PG-05, AH-D3).
///
/// Anchors "Room R has an MLS group at genesis epoch 0" on the DAG. Emitted by
/// the creating client only when the Space is E2E (`e2e_encryption == true`).
/// `room_create_event_id` is the parent Room's create event, threaded into
/// `prev_events` so the anchor is causally **under** the Room (non-root —
/// `validate_dag_structure` requires ≥1 predecessor; D-076 v1.1 honesty). The
/// `content` is Node-readable (no key material): genesis epoch + a nonce for a
/// unique event_id.
pub fn build_mls_group_init_event(
    key: &SigningKey,
    space_id: &str,
    room_id: &str,
    room_create_event_id: &str,
) -> Event {
    Event::new(
        EventType::MlsGroupInit,
        sender_xgid(key),
        // Pass 2 widens these projections; the wraps collapse then.
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        vec![EventXgid::from_xgid(Xgid::new(room_create_event_id.to_string()))],
        now(),
        json!({
            "epoch": 0,
            "nonce": generate_nonce(),
        }),
    )
}

/// Build an unsigned `mls.commit` Event (Arc H PG-05, AH-D4).
///
/// Emitted by the affected client after a resolved membership change in an E2E
/// Space, declaring the Room's new MLS `epoch`. The Node tracks the epoch
/// (`apply_mls_commit`) and routes the Commit opaque (the `mls_commit` payload is
/// the real MLS Commit message — a base64url blob in Phase 3; omitted here, the
/// Phase-2 interface advances the epoch counter). Non-root; `prev_events`
/// place it causally after the membership/commit it follows.
pub fn build_mls_commit_event(
    key: &SigningKey,
    space_id: &str,
    room_id: &str,
    prev_events: Vec<String>,
    epoch: u64,
) -> Event {
    Event::new(
        EventType::MlsCommit,
        sender_xgid(key),
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events
            .into_iter()
            .map(|s| EventXgid::from_xgid(Xgid::new(s)))
            .collect(),
        now(),
        json!({
            "epoch": epoch,
            "nonce": generate_nonce(),
        }),
    )
}

/// Build an unsigned `mls.key_package` Event (Arc H PG-05, AH-A5 / §3.10.3).
///
/// A client uploads a single-use KeyPackage to its home Node; the Node stores it
/// in its pool (`NodeRuntime::record_key_package`) for distribution when this
/// Identity/device is later added to an MLS group. `identity_id` is the sender
/// (the uploader); the `mls_key_package` blob is opaque to the Node.
pub fn build_mls_key_package_event(
    key: &SigningKey,
    space_id: &str,
    room_id: &str,
    prev_events: Vec<String>,
    device_id: &str,
    mls_key_package: &str,
    valid_until: &str,
) -> Event {
    Event::new(
        EventType::MlsKeyPackage,
        sender_xgid(key),
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events
            .into_iter()
            .map(|s| EventXgid::from_xgid(Xgid::new(s)))
            .collect(),
        now(),
        json!({
            "identity_id": sender_id(key),
            "device_id": device_id,
            "mls_key_package": mls_key_package,
            "uploaded_at": now(),
            "valid_until": valid_until,
        }),
    )
}

/// Build an unsigned `state.room_update` Event carrying a complete per-Room
/// permission-override set (PG-12-min, Arc D PM-D6). `overrides` is the full set
/// for the Room (replace semantics — see `apply_room_update`); an empty slice
/// clears all overrides. `room_id` must be the target Room (non-empty — the
/// override layer and the state-key both require it). Call `sign_event` to
/// compute event_id + signature.
///
/// Protocol-mechanism builder: a user-facing authoring surface (CLI /
/// `--aicontrol` / client command) is out of arc-D scope and rides the UI / ops
/// pass. Mirrors `build_room_create_event`.
pub fn build_room_update_event(
    key: &SigningKey,
    space_id: &str,
    room_id: &str,
    prev_events: Vec<String>,
    overrides: &[(Role, RoomPermission, Effect)],
) -> Event {
    let arr: Vec<Value> = overrides
        .iter()
        .map(|(role, perm, effect)| {
            json!({
                "role": role.as_str(),
                "permission": perm.as_str(),
                "effect": effect.as_str(),
            })
        })
        .collect();
    Event::new(
        EventType::StateRoomUpdate,
        sender_xgid(key),
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events
            .into_iter()
            .map(|s| EventXgid::from_xgid(Xgid::new(s)))
            .collect(),
        now(),
        json!({ "permission_overrides": arr, "nonce": generate_nonce() }),
    )
}

/// Build an unsigned `thread.create` Event (Arc E PG-08). Anchored to a Room:
/// `room_id` is the parent Room, `space_id` the Space. **CP-4 / D-076 v1.1** —
/// `prev_events` MUST place the Thread causally under the Room (never `vec![]`):
/// pass the Room's create-event id (`room_id`) for the first Thread, or the
/// Room's current tip(s). `validate_dag_structure` rejects a `thread.create` with
/// empty `prev_events` (non-root). `auth_tier_min` is the Thread's participation
/// floor (must be ≥ the Room/Space tier — enforced at dispatch step 4). The Thread
/// id derives from the signed event_id via [`thread_id_from_event_id`].
pub fn build_thread_create_event(
    key: &SigningKey,
    space_id: &str,
    room_id: &str,
    prev_events: Vec<String>,
    title: Option<&str>,
    auth_tier_min: u32,
) -> Event {
    let mut content = json!({
        "auth_tier_min": auth_tier_min,
        "nonce": generate_nonce(),
    });
    if let Some(t) = title {
        content["title"] = json!(t);
    }
    Event::new(
        EventType::ThreadCreate,
        sender_xgid(key),
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events_to_xgids(prev_events),
        now(),
        content,
    )
}

/// Build an unsigned `thread.resolved` Event (Arc E PG-08). `thread_id` is the
/// conceptual Thread id (`xgen://thread/sha256:`); `room_id` is the parent Room
/// (carried so step 11 enforces Room membership + step 13 the ChangeInfo gate).
/// `prev_events` chains the transition into the Room's DAG.
pub fn build_thread_resolved_event(
    key: &SigningKey,
    space_id: &str,
    room_id: &str,
    thread_id: &str,
    prev_events: Vec<String>,
) -> Event {
    build_thread_status_event(key, space_id, room_id, thread_id, prev_events, EventType::ThreadResolved)
}

/// Build an unsigned `thread.archived` Event (Arc E PG-08). See
/// [`build_thread_resolved_event`].
pub fn build_thread_archived_event(
    key: &SigningKey,
    space_id: &str,
    room_id: &str,
    thread_id: &str,
    prev_events: Vec<String>,
) -> Event {
    build_thread_status_event(key, space_id, room_id, thread_id, prev_events, EventType::ThreadArchived)
}

fn build_thread_status_event(
    key: &SigningKey,
    space_id: &str,
    room_id: &str,
    thread_id: &str,
    prev_events: Vec<String>,
    event_type: EventType,
) -> Event {
    Event::new(
        event_type,
        sender_xgid(key),
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events_to_xgids(prev_events),
        now(),
        json!({ "thread": thread_id, "nonce": generate_nonce() }),
    )
}

/// Build an unsigned `state.space_temperature_visibility` Event (spec 3.7.13.3).
/// The Space owner uses this to switch the member-temperature visibility setting.
pub fn build_space_temperature_visibility_event(
    key: &SigningKey,
    space_id: &str,
    prev_events: Vec<String>,
    visibility: &str,
) -> Event {
    Event::new(
        EventType::StateSpaceTemperatureVisibility,
        sender_xgid(key),
        empty_room_xgid(),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events_to_xgids(prev_events),
        now(),
        json!({ "member_temperature_visibility": visibility }),
    )
}

/// Build an unsigned `membership.mute` Event (spec 3.7.8).
/// `reason` is free text; use `xgen_common::wire::REASON_AUTO_TEMPERATURE` to
/// mark an automated temperature-driven mute (3.7.13.6).
pub fn build_membership_mute_event(
    key: &SigningKey,
    space_id: &str,
    room_id: &str,
    prev_events: Vec<String>,
    target_identity: &str,
    reason: &str,
    cooldown_until: &str,
) -> Event {
    Event::new(
        EventType::MembershipMute,
        sender_xgid(key),
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events_to_xgids(prev_events),
        now(),
        json!({
            "target_identity": target_identity,
            "reason": reason,
            "cooldown_until": cooldown_until,
        }),
    )
}

/// Decide whether a recipient should see `xgen.member_temperature` for a given
/// subject in the Space (spec 3.7.13.4). The Node applies this when relaying
/// `meta_atts` to subscribed clients.
///
/// - `recipient_id`: the authenticated Identity of the receiving client
/// - `subject_id`: the member whose temperature is being filtered
/// - The current `space.member_temperature_visibility` is read off `space`.
///
/// Unknown visibility values fall back to `moderator` behaviour (spec 3.7.13.3).
pub fn should_include_member_temperature(
    space: &SpaceState,
    recipient_id: &str,
    subject_id: &str,
) -> bool {
    if recipient_id == subject_id {
        return true; // a member always sees their own temperature
    }
    // The explicit `VISIBILITY_MODERATOR |` arm below is documentary — the
    // wildcard already covers it, but spelling it out makes the spec intent
    // (spec 3.7.13.3 — moderator is the default value and the fallback for
    // unknown values) visible in source. `clippy::wildcard_in_or_patterns`
    // would prefer dropping the named constant; the documentary form wins
    // here.
    #[allow(clippy::wildcard_in_or_patterns)]
    match space.member_temperature_visibility.as_str() {
        VISIBILITY_EVERYONE => true,
        VISIBILITY_SELF_ONLY => false,
        // Default (`moderator`) plus any unknown value treated as `moderator`.
        VISIBILITY_MODERATOR | _ => match space.member_role(recipient_id) {
            Some(role) => *role >= Role::Moderator,
            None => false,
        },
    }
}

/// Build an unsigned `state.space_pacing` Event (spec 3.7.12.3).
///
/// The Space owner uses this to update pacing rules. Both fields are required —
/// callers MUST provide explicit values on every update (no partial updates).
pub fn build_space_pacing_event(
    key: &SigningKey,
    space_id: &str,
    prev_events: Vec<String>,
    human_pacing_ms: u64,
    ai_pacing_ms: u64,
) -> Event {
    Event::new(
        EventType::StateSpacePacing,
        sender_xgid(key),
        empty_room_xgid(),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events_to_xgids(prev_events),
        now(),
        json!({
            "human_pacing_ms": human_pacing_ms,
            "ai_pacing_ms": ai_pacing_ms,
        }),
    )
}

/// Build an unsigned `state.dm_space_create` Event.
pub fn build_dm_space_create_event(
    key: &SigningKey,
    invitee: &str,
    home_node: &str,
) -> Event {
    Event::new(
        EventType::StateDmSpaceCreate,
        sender_xgid(key),
        empty_room_xgid(),
        empty_space_xgid(),
        vec![],
        now(),
        json!({
            "auth_tier": 1,
            "invitee": invitee,
            "nonce": generate_nonce(),
            "home_node": home_node,
        }),
    )
}

/// Build an unsigned `state.federation_add` Event (spec 3.4.5).
/// Produced by the Space owner when approving a federation join request.
pub fn build_federation_add_event(
    key: &SigningKey,
    space_id: &str,
    prev_events: Vec<String>,
    peer_node_id: &str,
    session_id: &str,
    negotiated_version: &str,
    negotiated_serialisation: &str,
) -> Event {
    Event::new(
        EventType::StateFederationAdd,
        sender_xgid(key),
        empty_room_xgid(),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events_to_xgids(prev_events),
        now(),
        json!({
            "node_id": peer_node_id,
            "session_id": session_id,
            "negotiated_version": negotiated_version,
            "negotiated_serialisation": negotiated_serialisation,
        }),
    )
}

/// Build an unsigned `state.dm_promote` Event (spec 3.16.3 Step 4).
/// Signed by the Node keypair — `key` must be the Node's keypair, not a member's.
pub fn build_dm_promote_event(
    node_key: &SigningKey,
    space_id: &str,
    prev_events: Vec<String>,
    proposed_by: &str,
    confirmed_by: &str,
    new_name: &str,
    timestamp: &str,
) -> Event {
    Event::new(
        EventType::StateDmPromote,
        sender_xgid(node_key),
        empty_room_xgid(),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events_to_xgids(prev_events),
        timestamp.to_string(),
        json!({
            "proposed_by": proposed_by,
            "confirmed_by": confirmed_by,
            "new_name": new_name,
            "promoted_at": timestamp,
        }),
    )
}

/// Build an unsigned `state.ai_operator_delegate` Event (spec 3.6.10.6).
/// Signer must be a Space owner or admin.
pub fn build_state_ai_operator_delegate_event(
    key: &SigningKey,
    space_id: &str,
    prev_events: Vec<String>,
    ai_identity_id: &str,
    new_operator_identity_id: &str,
) -> Event {
    Event::new(
        EventType::StateAiOperatorDelegate,
        sender_xgid(key),
        empty_room_xgid(),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events_to_xgids(prev_events),
        now(),
        json!({
            "space_id": space_id,
            "ai_identity_id": ai_identity_id,
            "new_operator_identity_id": new_operator_identity_id,
        }),
    )
}

/// Build an unsigned `state.ai_operator_revoke` Event (spec 3.6.10.6).
/// Signer must be a Space owner or admin.
pub fn build_state_ai_operator_revoke_event(
    key: &SigningKey,
    space_id: &str,
    prev_events: Vec<String>,
    ai_identity_id: &str,
) -> Event {
    Event::new(
        EventType::StateAiOperatorRevoke,
        sender_xgid(key),
        empty_room_xgid(),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev_events_to_xgids(prev_events),
        now(),
        json!({
            "space_id": space_id,
            "ai_identity_id": ai_identity_id,
        }),
    )
}

/// Build an unsigned membership Event (invite / join / leave / kick / ban).
pub fn build_membership_event(
    key: &SigningKey,
    space_id: &str,
    room_id: &str,
    event_type: EventType,
    content: Value,
) -> Event {
    // Pass 2 widens space_id/room_id parameters to typed XGIDs; the wraps collapse then.
    // Empty room_id projects to `empty_room_xgid()` to match wire-shape parity with
    // the pre-Pass-1 `String::new()`.
    let room = if room_id.is_empty() {
        empty_room_xgid()
    } else {
        RoomXgid::from_xgid(Xgid::new(room_id.to_string()))
    };
    let space = if space_id.is_empty() {
        empty_space_xgid()
    } else {
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string()))
    };
    Event::new(event_type, sender_xgid(key), room, space, vec![], now(), content)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::keypair;

    // ── Pass 1 Commit 4a test helpers ────────────────────────────────────────
    // Wrap `&str` into the typed XGID flavour at fixture / lookup sites. The
    // test surface predates Pass 1's data-structure retype; rather than rewire
    // every fixture, these helpers project at the call point.
    fn xid(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn nxd(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }
    /// Project an Option<EventXgid> → Option<String> for fixture-extracted
    /// event_ids (`room_ev.event_id.clone().unwrap()` shape).
    fn event_id_str(ev: &Event) -> String {
        ev.event_id
            .as_ref()
            .expect("signed event has event_id")
            .as_str()
            .to_string()
    }

    fn alice_key() -> SigningKey {
        keypair::generate()
    }

    fn bob_key() -> SigningKey {
        keypair::generate()
    }

    const HOME: &str = "xgen://pubkey/ed25519:NODE";

    fn create_space(key: &SigningKey) -> (SpaceState, String) {
        let ev = sign_event(build_space_create_event(key, "Test Space", None, 1, HOME, None, false), key);
        let space_id = event_id_str(&ev);
        let state = SpaceState::from_space_create(&ev).unwrap();
        (state, space_id)
    }

    #[test]
    fn space_create_sets_owner() {
        let key = alice_key();
        let (state, _) = create_space(&key);
        let alice_id = sender_id(&key);
        assert!(state.is_member(alice_id.as_str()));
        assert_eq!(state.member_role(alice_id.as_str()), Some(&Role::Owner));
    }

    #[test]
    fn space_create_event_id_is_space_id() {
        let key = alice_key();
        let (state, space_id) = create_space(&key);
        assert_eq!(state.space_id.as_str(), space_id);
    }

    // ── Arc G PG-04 — jurisdiction (set-once at create, AG-D1/D2/D4) ─────────
    #[test]
    fn space_create_reads_back_declared_jurisdiction() {
        let key = alice_key();
        let ev = sign_event(
            build_space_create_event(&key, "Gov Space", None, 1, HOME, Some("SK"), false),
            &key,
        );
        let state = SpaceState::from_space_create(&ev).unwrap();
        assert_eq!(state.jurisdiction.as_deref(), Some("SK"));
    }

    #[test]
    fn space_create_absent_jurisdiction_is_none() {
        // The canonical `create_space` helper passes `None` — undeclared ⇒ None.
        let key = alice_key();
        let (state, _) = create_space(&key);
        assert!(state.jurisdiction.is_none());
    }

    #[test]
    fn dm_space_create_declares_no_jurisdiction() {
        // DM Spaces always declare None (AG-D4) — through both DM constructors.
        let alice = alice_key();
        let bob = bob_key();
        let bob_id = sender_id(&bob);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &bob_id, HOME), &alice);
        let (keyed, _room_ev, _invite_ev) =
            SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();
        assert!(keyed.jurisdiction.is_none(), "from_dm_space_create ⇒ None");
        let node_view = SpaceState::from_dm_space_create_node(&create_ev).unwrap();
        assert!(node_view.jurisdiction.is_none(), "from_dm_space_create_node ⇒ None");
    }

    // ── Arc H PG-05 — e2e_encryption (set-once, default-OFF, uniform) + group-init ──
    #[test]
    fn space_create_reads_back_e2e_encryption() {
        let key = alice_key();
        let ev = sign_event(
            build_space_create_event(&key, "Secure Space", None, 1, HOME, None, true),
            &key,
        );
        let state = SpaceState::from_space_create(&ev).unwrap();
        assert!(state.e2e_encryption, "e2e_encryption: true must read back true (AH-D2)");
    }

    #[test]
    fn space_create_default_e2e_is_off() {
        // The canonical create_space helper passes false — default OFF (AH-D2).
        let key = alice_key();
        let (state, _) = create_space(&key);
        assert!(!state.e2e_encryption);
    }

    #[test]
    fn dm_space_create_e2e_uniform_default_off() {
        // e2e is read uniformly by the DM constructors (no DM exception, unlike
        // jurisdiction). No caller declares E2E for a DM today, so both DM views
        // default OFF — but the field is read, not hard-set.
        let alice = alice_key();
        let bob = bob_key();
        let bob_id = sender_id(&bob);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &bob_id, HOME), &alice);
        let (keyed, _r, _i) = SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();
        assert!(!keyed.e2e_encryption, "from_dm_space_create ⇒ OFF by default");
        let node_view = SpaceState::from_dm_space_create_node(&create_ev).unwrap();
        assert!(!node_view.e2e_encryption, "from_dm_space_create_node ⇒ OFF by default");
    }

    #[test]
    fn mls_group_init_applier_sets_genesis_epoch() {
        let key = alice_key();
        let create_ev = sign_event(
            build_space_create_event(&key, "Secure", None, 1, HOME, None, true),
            &key,
        );
        let mut state = SpaceState::from_space_create(&create_ev).unwrap();
        let space_id = state.space_id.as_str().to_string();
        let room_ev = sign_event(build_room_create_event(&key, &space_id, "general", None), &key);
        let room_id = room_ev.event_id.clone().unwrap().as_str().to_string();
        state.apply_event(&room_ev, "").unwrap();
        // A fresh Room carries no MLS epoch.
        assert_eq!(state.rooms.values().next().unwrap().mls_epoch, None);

        // state.mls_group_init anchors genesis epoch 0 (AH-D3).
        let init_ev = sign_event(
            build_mls_group_init_event(&key, &space_id, &room_id, &room_id),
            &key,
        );
        state.apply_event(&init_ev, "").unwrap();
        let room_key = RoomXgid::from_xgid(Xgid::new(room_id.clone()));
        assert_eq!(state.rooms.get(&room_key).unwrap().mls_epoch, Some(0), "genesis set");

        // Idempotent re-apply.
        state.apply_event(&init_ev, "").unwrap();
        assert_eq!(state.rooms.get(&room_key).unwrap().mls_epoch, Some(0));
    }

    #[test]
    fn mls_group_init_unknown_room_is_noop() {
        let key = alice_key();
        let (mut state, space_id) = create_space(&key);
        let init_ev = sign_event(
            build_mls_group_init_event(
                &key,
                &space_id,
                "xgen://hash/sha256:nope",
                "xgen://hash/sha256:nope",
            ),
            &key,
        );
        // Unknown target Room ⇒ silent no-op, never an error.
        assert!(state.apply_event(&init_ev, "").is_ok());
    }

    #[test]
    fn mls_commit_advances_room_epoch() {
        let key = alice_key();
        let create_ev = sign_event(
            build_space_create_event(&key, "Secure", None, 1, HOME, None, true),
            &key,
        );
        let mut state = SpaceState::from_space_create(&create_ev).unwrap();
        let space_id = state.space_id.as_str().to_string();
        let room_ev = sign_event(build_room_create_event(&key, &space_id, "general", None), &key);
        let room_id = room_ev.event_id.clone().unwrap().as_str().to_string();
        state.apply_event(&room_ev, "").unwrap();
        let room_key = RoomXgid::from_xgid(Xgid::new(room_id.clone()));

        // genesis 0
        let init_ev = sign_event(
            build_mls_group_init_event(&key, &space_id, &room_id, &room_id),
            &key,
        );
        state.apply_event(&init_ev, "").unwrap();
        assert_eq!(state.rooms.get(&room_key).unwrap().mls_epoch, Some(0));

        // an mls.commit (epoch 1) advances the tracked epoch
        let commit_ev = sign_event(
            build_mls_commit_event(&key, &space_id, &room_id, vec![room_id.clone()], 1),
            &key,
        );
        state.apply_event(&commit_ev, "").unwrap();
        assert_eq!(state.rooms.get(&room_key).unwrap().mls_epoch, Some(1));

        // unknown-room commit ⇒ silent no-op
        let stray = sign_event(
            build_mls_commit_event(&key, &space_id, "xgen://hash/sha256:nope", vec![room_id], 9),
            &key,
        );
        assert!(state.apply_event(&stray, "").is_ok());
        assert_eq!(state.rooms.get(&room_key).unwrap().mls_epoch, Some(1));
    }

    #[test]
    fn room_create_by_owner_succeeds() {
        let key = alice_key();
        let (mut state, space_id) = create_space(&key);
        let room_ev = sign_event(build_room_create_event(&key, &space_id, "general", None), &key);
        state.apply_event(&room_ev, "").unwrap();
        assert_eq!(state.rooms.len(), 1);
        let room_id = event_id_str(&room_ev);
        assert!(state.rooms.contains_key(room_id.as_str()));
    }

    #[test]
    fn room_create_event_records_space_create_as_predecessor() {
        let key = alice_key();
        let (_state, space_id) = create_space(&key);
        let room_ev = build_room_create_event(&key, &space_id, "general", None);
        // Pass 1 Commit 4a — Vec<EventXgid> vs Vec<String>; project for equality check.
        let prev_strs: Vec<&str> =
            room_ev.prev_events.iter().map(|e| e.as_str()).collect();
        assert_eq!(
            prev_strs,
            vec![space_id.as_str()],
            "build_room_create_event must record space_id as the sole predecessor \
             (D-076 v1.1 causal-DAG-respecting order); empty prev_events is the \
             pre-Path-B bug that placed state.room_create as a DAG root and \
             allowed canonical wire orderings where state.room_create preceded \
             its protocol-level parent state.space_create"
        );
    }

    #[test]
    fn room_create_by_member_permission_denied() {
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, space_id) = create_space(&alice);
        let alice_id = sender_id(&alice);
        let bob_id = sender_id(&bob);

        // Alice invites Bob as member (not admin).
        state.pending_invites.insert(xid(&bob_id), PendingInvite::from_role(Role::Member));

        // Bob joins the space.
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev, "").unwrap();
        assert_eq!(state.member_role(bob_id.as_str()), Some(&Role::Member));

        // Bob tries to create a room — should fail.
        let room_ev = sign_event(build_room_create_event(&bob, &space_id, "secret", None), &bob);
        let err = state.apply_event(&room_ev, "").unwrap_err();
        assert!(matches!(err, SpaceError::PermissionDenied(_)));

        // Silence unused variable warning in this test.
        let _ = alice_id;
    }

    #[test]
    fn invite_join_membership_flow() {
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, space_id) = create_space(&alice);
        let bob_id = sender_id(&bob);

        // Alice invites Bob.
        let invite_ev = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            &alice,
        );
        state.apply_event(&invite_ev, "").unwrap();
        assert!(state.pending_invites.contains_key(bob_id.as_str()));

        // Bob joins the space.
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev, "").unwrap();
        assert!(state.is_member(bob_id.as_str()));
        assert_eq!(state.member_role(bob_id.as_str()), Some(&Role::Member));
        assert!(!state.pending_invites.contains_key(bob_id.as_str()));
    }

    // ── PG-12-min (Arc D, C2) — state.room_update override applier ─────────────

    #[test]
    fn apply_room_update_sets_then_replaces_then_clears() {
        let alice = alice_key();
        let (mut state, space_id) = create_space(&alice);
        let room_ev =
            sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        state.apply_event(&room_ev, "").unwrap();
        let room_id = event_id_str(&room_ev);
        let room_key = RoomXgid::from_xgid(Xgid::new(room_id.clone()));
        // Fresh room starts with no overrides.
        assert!(state.rooms.get(&room_key).unwrap().permission_overrides.is_empty());

        // Set one override.
        let u1 = sign_event(
            build_room_update_event(
                &alice,
                &space_id,
                &room_id,
                vec![room_id.clone()],
                &[(Role::Moderator, RoomPermission::SendMessages, Effect::Deny)],
            ),
            &alice,
        );
        state.apply_event(&u1, "").unwrap();
        let r = state.rooms.get(&room_key).unwrap();
        assert_eq!(r.permission_overrides.len(), 1);
        assert_eq!(
            r.permission_overrides.get(&(Role::Moderator, RoomPermission::SendMessages)),
            Some(&Effect::Deny)
        );

        // PM-D3 replace: a second event wholesale-replaces with a different set.
        let u2 = sign_event(
            build_room_update_event(
                &alice,
                &space_id,
                &room_id,
                vec![event_id_str(&u1)],
                &[(Role::Member, RoomPermission::Invite, Effect::Allow)],
            ),
            &alice,
        );
        state.apply_event(&u2, "").unwrap();
        let r = state.rooms.get(&room_key).unwrap();
        assert_eq!(r.permission_overrides.len(), 1, "replace, not merge");
        assert!(!r
            .permission_overrides
            .contains_key(&(Role::Moderator, RoomPermission::SendMessages)));
        assert_eq!(
            r.permission_overrides.get(&(Role::Member, RoomPermission::Invite)),
            Some(&Effect::Allow)
        );

        // Empty set clears.
        let u3 = sign_event(
            build_room_update_event(&alice, &space_id, &room_id, vec![event_id_str(&u2)], &[]),
            &alice,
        );
        state.apply_event(&u3, "").unwrap();
        assert!(state.rooms.get(&room_key).unwrap().permission_overrides.is_empty());
    }

    #[test]
    fn apply_room_update_skips_unknown_and_ignores_absent_key() {
        let alice = alice_key();
        let (mut state, space_id) = create_space(&alice);
        let room_ev =
            sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        state.apply_event(&room_ev, "").unwrap();
        let room_id = event_id_str(&room_ev);
        let room_key = RoomXgid::from_xgid(Xgid::new(room_id.clone()));

        // Seed one real override.
        let u1 = sign_event(
            build_room_update_event(
                &alice,
                &space_id,
                &room_id,
                vec![room_id.clone()],
                &[(Role::Admin, RoomPermission::Ban, Effect::Deny)],
            ),
            &alice,
        );
        state.apply_event(&u1, "").unwrap();
        assert_eq!(state.rooms.get(&room_key).unwrap().permission_overrides.len(), 1);

        // An update whose entries are ALL unknown strings → each skipped
        // (forward-compat, not an error) → replace yields an empty set.
        let mut bad =
            build_room_update_event(&alice, &space_id, &room_id, vec![event_id_str(&u1)], &[]);
        bad.content = json!({ "permission_overrides": [
            { "role": "superuser", "permission": "send_messages", "effect": "deny" },
            { "role": "moderator", "permission": "teleport", "effect": "allow" },
            { "role": "moderator", "permission": "ban", "effect": "maybe" },
        ], "nonce": "n" });
        let bad = sign_event(bad, &alice);
        state.apply_event(&bad, "").unwrap();
        assert!(
            state.rooms.get(&room_key).unwrap().permission_overrides.is_empty(),
            "all-unknown entries skipped → replace yields empty"
        );

        // Re-seed, then an update with NO permission_overrides key (e.g. a future
        // name-only room_update) must leave the override set untouched.
        let u2 = sign_event(
            build_room_update_event(
                &alice,
                &space_id,
                &room_id,
                vec![event_id_str(&bad)],
                &[(Role::Admin, RoomPermission::Ban, Effect::Deny)],
            ),
            &alice,
        );
        state.apply_event(&u2, "").unwrap();
        assert_eq!(state.rooms.get(&room_key).unwrap().permission_overrides.len(), 1);

        let mut name_only =
            build_room_update_event(&alice, &space_id, &room_id, vec![event_id_str(&u2)], &[]);
        name_only.content = json!({ "name": "renamed", "nonce": "n2" });
        let name_only = sign_event(name_only, &alice);
        state.apply_event(&name_only, "").unwrap();
        assert_eq!(
            state.rooms.get(&room_key).unwrap().permission_overrides.len(),
            1,
            "room_update without permission_overrides key must not touch the set"
        );

        // Unknown target room → no-op (does not panic / error).
        let mut ghost =
            build_room_update_event(&alice, &space_id, "xgen://hash/sha256:ghost", vec![], &[]);
        ghost.content = json!({ "permission_overrides": [], "nonce": "n3" });
        let ghost = sign_event(ghost, &alice);
        assert!(state.apply_event(&ghost, "").is_ok());
    }

    #[test]
    fn join_room_after_joining_space() {
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, space_id) = create_space(&alice);
        let bob_id = sender_id(&bob);

        // Alice creates a room.
        let room_ev = sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        let room_id = event_id_str(&room_ev);
        state.apply_event(&room_ev, "").unwrap();

        // Bob joins space.
        state.pending_invites.insert(xid(&bob_id), PendingInvite::from_role(Role::Member));
        let join_space = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_space, "").unwrap();

        // Bob joins the room.
        let join_room = sign_event(
            build_membership_event(&bob, &space_id, &room_id, EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_room, "").unwrap();
        assert!(state.is_room_member(bob_id.as_str(), room_id.as_str()));
    }

    #[test]
    fn leave_removes_from_space_and_all_rooms() {
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, space_id) = create_space(&alice);
        let bob_id = sender_id(&bob);

        // Setup: room + Bob member.
        let room_ev = sign_event(build_room_create_event(&alice, &space_id, "gen", None), &alice);
        let room_id = event_id_str(&room_ev);
        state.apply_event(&room_ev, "").unwrap();

        state.pending_invites.insert(xid(&bob_id), PendingInvite::from_role(Role::Member));
        state.apply_event(&sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        ), "").unwrap();
        state.apply_event(&sign_event(
            build_membership_event(&bob, &space_id, &room_id, EventType::MembershipJoin, json!({})),
            &bob,
        ), "").unwrap();
        assert!(state.is_room_member(bob_id.as_str(), room_id.as_str()));

        // Bob leaves the space.
        state.apply_event(&sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipLeave, json!({})),
            &bob,
        ), "").unwrap();
        assert!(!state.is_member(bob_id.as_str()));
        assert!(!state.is_room_member(bob_id.as_str(), room_id.as_str()));
    }

    #[test]
    fn leave_by_non_member_rejected() {
        // M7C-D3 (A2): a non-member's leave is rejected — apply_leave returns
        // NotASpaceMember (the node's default member-event path is gated on
        // step-11 sender-membership; there is no special leave role).
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, space_id) = create_space(&alice);
        // Bob never joined.
        let leave_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipLeave, json!({})),
            &bob,
        );
        assert_eq!(
            state.apply_event(&leave_ev, "").unwrap_err(),
            SpaceError::NotASpaceMember
        );
    }

    // ── M6 A4-D1 node_eject / node_unban ─────────────────────────────────────

    /// Build a Space homed at `node`'s URI with `bob` as a joined member.
    fn space_homed_at_node_with_bob(
        owner: &SigningKey,
        node: &SigningKey,
        bob: &SigningKey,
    ) -> (SpaceState, String) {
        let node_uri = sender_xgid(node);
        let ev = sign_event(
            build_space_create_event(owner, "Hosted", None, 1, node_uri.as_str(), None, false),
            owner,
        );
        let space_id = event_id_str(&ev);
        let mut state = SpaceState::from_space_create(&ev).unwrap();
        let bob_id = sender_id(bob);
        state.pending_invites.insert(xid(&bob_id), PendingInvite::from_role(Role::Member));
        state
            .apply_event(
                &sign_event(
                    build_membership_event(bob, &space_id, "", EventType::MembershipJoin, json!({})),
                    bob,
                ),
                "",
            )
            .unwrap();
        (state, space_id)
    }

    #[test]
    fn node_eject_removes_member_and_bans() {
        let alice = alice_key();
        let node = keypair::generate();
        let bob = bob_key();
        let (mut state, space_id) = space_homed_at_node_with_bob(&alice, &node, &bob);
        let bob_id = sender_id(&bob);
        assert!(state.is_member(bob_id.as_str()));

        let node_uri = sender_xgid(&node);
        let eject = sign_event(
            build_membership_event(
                &node,
                &space_id,
                "",
                EventType::MembershipNodeEject,
                json!({ "target_identity": bob_id }),
            ),
            &node,
        );
        state.apply_event(&eject, node_uri.as_str()).unwrap();
        assert!(!state.is_member(bob_id.as_str()));
        assert!(state.banned.contains(bob_id.as_str()));
    }

    #[test]
    fn node_unban_lifts_ban_and_allows_rejoin() {
        let alice = alice_key();
        let node = keypair::generate();
        let bob = bob_key();
        let (mut state, space_id) = space_homed_at_node_with_bob(&alice, &node, &bob);
        let bob_id = sender_id(&bob);
        let node_uri = sender_xgid(&node);

        // Eject (removes + bans).
        state
            .apply_event(
                &sign_event(
                    build_membership_event(
                        &node,
                        &space_id,
                        "",
                        EventType::MembershipNodeEject,
                        json!({ "target_identity": bob_id }),
                    ),
                    &node,
                ),
                node_uri.as_str(),
            )
            .unwrap();
        assert!(state.banned.contains(bob_id.as_str()));

        // Unban lifts the ban.
        state
            .apply_event(
                &sign_event(
                    build_membership_event(
                        &node,
                        &space_id,
                        "",
                        EventType::MembershipNodeUnban,
                        json!({ "target_identity": bob_id }),
                    ),
                    &node,
                ),
                node_uri.as_str(),
            )
            .unwrap();
        assert!(!state.banned.contains(bob_id.as_str()));

        // Rejoin now succeeds (ban lifted; not auto-restored).
        state.pending_invites.insert(xid(&bob_id), PendingInvite::from_role(Role::Member));
        state
            .apply_event(
                &sign_event(
                    build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
                    &bob,
                ),
                "",
            )
            .unwrap();
        assert!(state.is_member(bob_id.as_str()));
    }

    #[test]
    fn node_eject_from_non_home_node_rejected_at_apply() {
        let alice = alice_key();
        let node = keypair::generate();
        let bob = bob_key();
        let (mut state, space_id) = space_homed_at_node_with_bob(&alice, &node, &bob);
        let bob_id = sender_id(&bob);

        // Alice (owner, NOT the home node) tries to node_eject → apply-layer
        // PermissionDenied (the wire-layer NodeEjectAuthority gate is in
        // exchange.rs; this is the defensive in-apply check).
        let forged = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipNodeEject,
                json!({ "target_identity": bob_id }),
            ),
            &alice,
        );
        let err = state.apply_event(&forged, "").unwrap_err();
        assert!(matches!(err, SpaceError::PermissionDenied(_)));
        assert!(state.is_member(bob_id.as_str())); // unchanged
    }

    // ── Arc F (PG-11) Space Migration cutover applier (AF-D1/D2) ──────────────

    #[test]
    fn space_migrate_flips_home_node_source_to_dest() {
        use crate::migration::state_machine::build_space_migrate_event;
        let alice = alice_key();
        let source = keypair::generate();
        let bob = bob_key();
        let (mut state, space_id) = space_homed_at_node_with_bob(&alice, &source, &bob);
        assert_eq!(state.home_node.as_str(), sender_id(&source));

        // Source home Node signs the cutover to the destination.
        let dst = "xgen://pubkey/ed25519:DEST1";
        let migrate = build_space_migrate_event(
            &source,
            &space_id,
            dst,
            "wss://dst.example.com/xgen",
            vec![space_id.clone()],
            "2026-06-04T10:00:00.000Z",
        );
        state.apply_event(&migrate, "").unwrap();
        assert_eq!(state.home_node.as_str(), dst, "home_node flipped source→dest");
    }

    #[test]
    fn space_migrate_idempotent_reapply_is_noop() {
        use crate::migration::state_machine::build_space_migrate_event;
        let alice = alice_key();
        let source = keypair::generate();
        let bob = bob_key();
        let (mut state, space_id) = space_homed_at_node_with_bob(&alice, &source, &bob);

        let dst = "xgen://pubkey/ed25519:DEST1";
        let migrate = build_space_migrate_event(
            &source, &space_id, dst, "wss://dst/x", vec![space_id.clone()], "2026-06-04T10:00:00.000Z",
        );
        state.apply_event(&migrate, "").unwrap();
        // Second apply of the same cutover (e.g. a derive_resolved re-fold) is a
        // no-op — and must NOT be rejected by the authority gate.
        state.apply_event(&migrate, "").unwrap();
        assert_eq!(state.home_node.as_str(), dst);
    }

    #[test]
    fn space_migrate_post_cutover_stale_source_rejected() {
        // AF-D2 self-protection: once the anchor has flipped, the old source is no
        // longer home_node, so a second/competing source-signed migrate fails the
        // `sender == home_node` gate by construction.
        use crate::migration::state_machine::build_space_migrate_event;
        let alice = alice_key();
        let source = keypair::generate();
        let bob = bob_key();
        let (mut state, space_id) = space_homed_at_node_with_bob(&alice, &source, &bob);

        let dst1 = "xgen://pubkey/ed25519:DEST1";
        let migrate1 = build_space_migrate_event(
            &source, &space_id, dst1, "wss://dst1/x", vec![space_id.clone()], "2026-06-04T10:00:00.000Z",
        );
        state.apply_event(&migrate1, "").unwrap();
        assert_eq!(state.home_node.as_str(), dst1);

        // The original source tries to migrate the Space onward to a third node.
        let dst2 = "xgen://pubkey/ed25519:DEST2";
        let migrate2 = build_space_migrate_event(
            &source, &space_id, dst2, "wss://dst2/x", vec![space_id.clone()], "2026-06-04T10:05:00.000Z",
        );
        let err = state.apply_event(&migrate2, "").unwrap_err();
        assert!(matches!(err, SpaceError::PermissionDenied(_)), "stale source migrate rejected");
        assert_eq!(state.home_node.as_str(), dst1, "home_node unchanged by the rejected migrate");
    }

    #[test]
    fn space_migrate_url_home_node_stalls_authority_gate() {
        // M10.4 (MP-F13) — Site 2 namespace reconciliation (D2). The cutover
        // authority gate compares the source-signed migrate's sender (a pubkey)
        // against the Space's home_node. The pre-M10.4 real-binary value — a
        // ws:// URL — can never equal the source pubkey, so the gate stalls; a
        // pubkey home_node clears it and the anchor flips. This is the applier
        // twin of validate_event's 6009 gate (the SAME `sender == home_node`
        // equality, exchange.rs:717, which apply_space_migrate re-checks
        // defensively, state.rs:1158). The validate_event-level + end-to-end
        // re-home is the M10.5 box-gated MP-C-16 (D3).
        use crate::migration::state_machine::build_space_migrate_event;
        let alice = alice_key();
        let source = keypair::generate();
        let dst = "xgen://pubkey/ed25519:DEST1";

        // URL home_node (the bug value) → source-signed cutover stalls.
        let url_home = "ws://127.0.0.1:8521/xgen";
        let url_ev = sign_event(
            build_space_create_event(&alice, "Hosted", None, 1, url_home, None, false),
            &alice,
        );
        let url_space_id = event_id_str(&url_ev);
        let mut url_state = SpaceState::from_space_create(&url_ev).unwrap();
        let migrate_url = build_space_migrate_event(
            &source, &url_space_id, dst, "wss://dst/x", vec![url_space_id.clone()],
            "2026-06-14T00:00:00.000Z",
        );
        let err = url_state.apply_event(&migrate_url, "").unwrap_err();
        assert!(
            matches!(err, SpaceError::PermissionDenied(_)),
            "URL home_node must stall the cutover authority gate"
        );
        assert_eq!(url_state.home_node.as_str(), url_home, "rejected migrate leaves home_node unchanged");

        // Pubkey home_node (== source pubkey, the reconciled value) → clears + flips.
        let pubkey_home = sender_id(&source);
        let ok_ev = sign_event(
            build_space_create_event(&alice, "Hosted", None, 1, &pubkey_home, None, false),
            &alice,
        );
        let ok_space_id = event_id_str(&ok_ev);
        let mut ok_state = SpaceState::from_space_create(&ok_ev).unwrap();
        let migrate_ok = build_space_migrate_event(
            &source, &ok_space_id, dst, "wss://dst/x", vec![ok_space_id.clone()],
            "2026-06-14T00:00:00.000Z",
        );
        ok_state.apply_event(&migrate_ok, "").unwrap();
        assert_eq!(ok_state.home_node.as_str(), dst, "pubkey home_node clears the gate and flips to dest");
    }

    #[test]
    fn ban_blocks_rejoin() {
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, space_id) = create_space(&alice);
        let bob_id = sender_id(&bob);

        // Alice bans Bob directly (without prior membership for brevity).
        // Ensure Alice is at least admin first — she's owner, so fine.
        let ban_ev = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipBan,
                json!({ "target_identity": bob_id }),
            ),
            &alice,
        );
        state.apply_event(&ban_ev, "").unwrap();
        assert!(state.banned.contains(bob_id.as_str()));

        // Try to invite Bob again — should fail with Banned.
        let invite_ev = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            &alice,
        );
        let err = state.apply_event(&invite_ev, "").unwrap_err();
        assert_eq!(err, SpaceError::Banned);
    }

    #[test]
    fn sign_event_produces_valid_signature() {
        let key = alice_key();
        let ev = sign_event(build_space_create_event(&key, "Test", None, 1, HOME, None, false), &key);
        assert!(ev.event_id.is_some());
        assert!(ev.signature.is_some());
        assert!(verify_event_signature(&ev));
    }

    #[test]
    fn tampered_event_fails_verification() {
        let key = alice_key();
        let mut ev = sign_event(build_space_create_event(&key, "Test", None, 1, HOME, None, false), &key);
        ev.content["name"] = json!("Tampered");
        assert!(!verify_event_signature(&ev));
    }

    #[test]
    fn dm_space_creates_room_and_invite() {
        let alice = alice_key();
        let bob = bob_key();
        let bob_id = sender_id(&bob);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &bob_id, HOME), &alice);
        let (state, room_ev, invite_ev) =
            SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();
        assert!(state.is_dm);
        assert_eq!(state.rooms.len(), 1);
        assert!(state.pending_invites.contains_key(bob_id.as_str()));
        assert!(room_ev.event_id.is_some());
        assert_eq!(invite_ev.event_type, EventType::MembershipInvite);
    }

    #[test]
    fn from_dm_space_create_auto_invite_has_empty_prev_events_latent_bug() {
        // LATENT CONSTRUCTOR ISSUE (D-065, flagged at M7C A3 / J-219): the
        // bundled auto-invite is built via `build_membership_event`, which sets
        // empty prev_events — but membership.invite is a non-root EventType, so a
        // node gate-rejects it at validate_event step 10 (validate_dag_structure:
        // non-root needs >=1 prev_event). No production caller sent this invite
        // before A3 (from_dm_space_create had no client verb), so the malformed
        // shape was never exercised through dispatch. `ops::create_dm_space`
        // overrides at the call site (rebuilds the invite tip-chained to the
        // auto-room) rather than fixing the constructor here. This test pins the
        // bug so its eventual fix (and the removal of the A3 override) is
        // deliberate, not accidental. When the constructor is fixed to tip-chain
        // the invite, THIS assertion flips and the A3 override can be dropped.
        let alice = alice_key();
        let bob = bob_key();
        let bob_id = sender_id(&bob);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &bob_id, HOME), &alice);
        let (_state, _room_ev, invite_ev) =
            SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();
        assert!(
            invite_ev.prev_events.is_empty(),
            "latent bug witness: auto-invite still has empty prev_events"
        );
    }

    #[test]
    fn from_dm_space_create_node_seeds_owner_and_pending_invite_keyless() {
        // M7C-D4 (A1): the key-less observer/node init seeds the owner + the
        // pending invite (apply_invite can't add it under DM constraints), but
        // NOT the Room — the auto-room arrives as a separate event.
        let alice = alice_key();
        let bob = bob_key();
        let bob_id = sender_id(&bob);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &bob_id, HOME), &alice);
        let state = SpaceState::from_dm_space_create_node(&create_ev).unwrap();
        assert!(state.is_dm);
        assert!(state.dm_constraints_active);
        assert_eq!(state.members.len(), 1, "only the creator is a member at init");
        assert_eq!(state.owner_id.as_str(), sender_id(&alice));
        assert!(state.rooms.is_empty(), "auto-room is a separate event, not pre-seeded");
        assert!(state.pending_invites.contains_key(bob_id.as_str()));
        assert_eq!(
            state.pending_invites.get(bob_id.as_str()).unwrap().invited_by.as_ref().map(|x| x.as_str()),
            Some(sender_id(&alice).as_str())
        );
    }

    #[test]
    fn from_dm_space_create_node_rejects_non_dm_event() {
        let alice = alice_key();
        let ev = sign_event(build_space_create_event(&alice, "Regular", None, 1, HOME, None, false), &alice);
        assert_eq!(
            SpaceState::from_dm_space_create_node(&ev).unwrap_err(),
            SpaceError::WrongEventType
        );
    }

    #[test]
    fn from_dm_space_create_node_then_appliers_admit_invitee_with_inviter() {
        // The whole point of A1's DM coverage: key-less init + the
        // separately-arriving auto-room + auto-invite + the invitee's join
        // replay to the full {owner, invitee} member set with invited_by intact.
        let alice = alice_key();
        let bob = bob_key();
        let bob_id = sender_id(&bob);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &bob_id, HOME), &alice);
        let space_id = event_id_str(&create_ev);
        // The creator-signed auto-room + auto-invite, exactly as the wire carries them.
        let (_authoring, room_ev, invite_ev) =
            SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();

        let mut state = SpaceState::from_dm_space_create_node(&create_ev).unwrap();
        // Auto-room applies (first DM Room).
        state.apply_event(&room_ev, "").unwrap();
        assert_eq!(state.rooms.len(), 1);
        // Auto-invite is rejected under DM constraints (3.16.1) — the pending
        // invite seeded at init is what carries invited_by.
        assert_eq!(state.apply_event(&invite_ev, "").unwrap_err(), SpaceError::DmInvitationNotAllowed);
        // Bob joins (space-level) and is admitted as Member with the seeded inviter.
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev, "").unwrap();
        assert_eq!(state.members.len(), 2);
        let bob_member = state.members.get(bob_id.as_str()).expect("bob is a member");
        assert_eq!(bob_member.role, Role::Member);
        assert_eq!(bob_member.invited_by.as_ref().map(|x| x.as_str()), Some(sender_id(&alice).as_str()));
    }

    // ── M11 self-thread (D-021) witnesses — W1/W2/W3 ─────────────────────────

    #[test]
    fn from_dm_space_create_self_dm_seeds_no_pending_invite() {
        // W1a (M11-D1): a self-DM (invitee == creator) seeds NO pending invite —
        // a clean single-member room. RED-on-revert: drop the guard at state.rs
        // ~419 and the vestigial pending_invites[self] entry returns.
        let alice = alice_key();
        let self_id = sender_id(&alice);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &self_id, HOME), &alice);
        let (state, _room, _invite) = SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();
        assert!(state.is_dm);
        assert_eq!(state.members.len(), 1, "creator is the sole member");
        assert_eq!(state.members.get(self_id.as_str()).unwrap().role, Role::Owner);
        assert!(
            state.pending_invites.is_empty(),
            "self-DM must seed no pending invite (M11-D1)"
        );
    }

    #[test]
    fn from_dm_space_create_node_self_dm_seeds_no_pending_invite() {
        // W1b (M11-D1): the key-less node-init mirror. RED-on-revert genuine at
        // the second guard site (state.rs ~530).
        let alice = alice_key();
        let self_id = sender_id(&alice);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &self_id, HOME), &alice);
        let state = SpaceState::from_dm_space_create_node(&create_ev).unwrap();
        assert!(state.is_dm);
        assert_eq!(state.members.len(), 1);
        assert_eq!(state.owner_id.as_str(), self_id.as_str());
        assert!(
            state.pending_invites.is_empty(),
            "self-DM node init must seed no pending invite (M11-D1)"
        );
    }

    #[test]
    fn self_dm_creator_is_owner_and_dm_room_member() {
        // W2 (functional): post-create the creator is Owner + a member of the
        // auto dm Room with no membership.join — a usable single-member thread.
        // (Message admission under a live registry is the node reach witness,
        // Commit 3 / W4.)
        let alice = alice_key();
        let self_id = sender_id(&alice);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &self_id, HOME), &alice);
        let (state, _room, _invite) = SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();
        assert_eq!(state.members.get(self_id.as_str()).unwrap().role, Role::Owner);
        assert_eq!(state.rooms.len(), 1, "auto dm Room is present");
        let room = state.rooms.values().next().unwrap();
        assert!(
            room.members.contains(&xid(&self_id)),
            "creator auto-joins the dm Room (no membership.join needed)"
        );
    }

    #[test]
    fn self_dm_never_federates() {
        // W3 (non-federation): a self-DM rejects federation_add with the hard
        // DmFederationNotAllowed guard — privacy as a structural property. The
        // party set is degenerate {this_node}; nothing to push.
        let alice = alice_key();
        let self_id = sender_id(&alice);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &self_id, HOME), &alice);
        let (mut state, _room, _invite) =
            SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();
        let event = make_federation_event(&alice, &self_id, state.space_id.as_str());
        let err = state.apply_event(&event, &self_id).unwrap_err();
        assert_eq!(err, SpaceError::DmFederationNotAllowed);
        assert!(state.federation_nodes.is_empty());
    }

    // ── Layer 14 — DM Space Promotion tests ──────────────────────────────────

    fn make_dm_space_with_two_members() -> (SpaceState, String, SigningKey, SigningKey) {
        let alice = alice_key();
        let bob = bob_key();
        let bob_id = sender_id(&bob);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &bob_id, HOME), &alice);
        let space_id = event_id_str(&create_ev);
        let (mut state, _, _) = SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();
        // Bob joins.
        state.pending_invites.insert(xid(&bob_id), PendingInvite::from_role(crate::space::membership::Role::Member));
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev, "").unwrap();
        (state, space_id, alice, bob)
    }

    #[test]
    fn dm_space_rejects_third_member_invite() {
        let (mut state, space_id, alice, _) = make_dm_space_with_two_members();
        let charlie = bob_key(); // reuse generator
        let charlie_id = sender_id(&charlie);
        // Alice (owner) tries to invite a third person — must fail.
        let invite_ev = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": charlie_id, "role": "member" }),
            ),
            &alice,
        );
        let err = state.apply_event(&invite_ev, "").unwrap_err();
        assert_eq!(err, SpaceError::DmInvitationNotAllowed);
    }

    #[test]
    fn dm_space_rejects_second_room() {
        let (mut state, space_id, alice, _) = make_dm_space_with_two_members();
        // Alice (owner) tries to create a second room — DM already has 1 room.
        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "extra", None),
            &alice,
        );
        let err = state.apply_event(&room_ev, "").unwrap_err();
        assert_eq!(err, SpaceError::DmSecondRoomNotAllowed);
    }

    #[test]
    fn dm_constraints_lifted_after_promotion() {
        let (mut state, space_id, alice, bob) = make_dm_space_with_two_members();
        let node_key = alice_key();
        let alice_id = sender_id(&alice);
        let bob_id = sender_id(&bob);
        let ts = "2026-05-14T10:00:00.000Z";

        // Apply state.dm_promote (signed by node, not a member).
        let promote_ev = sign_event(
            build_dm_promote_event(&node_key, &space_id, vec![], &alice_id, &bob_id, "Our Project", ts),
            &node_key,
        );
        state.apply_event(&promote_ev, "").unwrap();
        assert!(!state.dm_constraints_active);
        assert_eq!(state.name.as_deref(), Some("Our Project"));

        // Now an invite should succeed.
        let charlie = bob_key();
        let charlie_id = sender_id(&charlie);
        let invite_ev = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": charlie_id, "role": "member" }),
            ),
            &alice,
        );
        state.apply_event(&invite_ev, "").unwrap();
        assert!(state.pending_invites.contains_key(charlie_id.as_str()));
    }

    #[test]
    fn history_preserved_after_promotion() {
        let (mut state, space_id, alice, bob) = make_dm_space_with_two_members();
        let alice_id = sender_id(&alice);
        let bob_id = sender_id(&bob);
        // Both members and the existing room are present before promotion.
        assert_eq!(state.members.len(), 2);
        assert_eq!(state.rooms.len(), 1);

        let node_key = alice_key();
        let ts = "2026-05-14T10:00:00.000Z";
        let promote_ev = sign_event(
            build_dm_promote_event(&node_key, &space_id, vec![], &alice_id, &bob_id, "Promoted", ts),
            &node_key,
        );
        state.apply_event(&promote_ev, "").unwrap();

        // Post-promotion: members and rooms unchanged.
        assert_eq!(state.members.len(), 2, "members must survive promotion");
        assert_eq!(state.rooms.len(), 1, "rooms must survive promotion");
        assert!(state.is_member(alice_id.as_str()));
        assert!(state.is_member(bob_id.as_str()));
    }

    // ── Pacing rules (spec 3.7.12) ────────────────────────────────────────

    #[test]
    fn space_create_applies_default_pacing_when_absent() {
        let key = alice_key();
        let (state, _) = create_space(&key);
        assert_eq!(state.human_pacing_ms, DEFAULT_HUMAN_PACING_MS);
        assert_eq!(state.ai_pacing_ms, DEFAULT_AI_PACING_MS);
    }

    #[test]
    fn space_create_honours_explicit_pacing_values() {
        // Build a state.space_create with explicit pacing values and verify
        // SpaceState picks them up.
        let key = alice_key();
        let mut ev = build_space_create_event(&key, "Test Space", None, 1, HOME, None, false);
        let obj = ev.content.as_object_mut().unwrap();
        obj.insert("human_pacing_ms".to_string(), json!(1500));
        obj.insert("ai_pacing_ms".to_string(), json!(10_000));
        let ev = sign_event(ev, &key);
        let state = SpaceState::from_space_create(&ev).unwrap();
        assert_eq!(state.human_pacing_ms, 1500);
        assert_eq!(state.ai_pacing_ms, 10_000);
    }

    #[test]
    fn space_pacing_updated_by_owner() {
        let alice = alice_key();
        let (mut state, space_id) = create_space(&alice);
        let ev = sign_event(
            build_space_pacing_event(&alice, &space_id, vec![], 2000, 8000),
            &alice,
        );
        state.apply_event(&ev, "").unwrap();
        assert_eq!(state.human_pacing_ms, 2000);
        assert_eq!(state.ai_pacing_ms, 8000);
    }

    #[test]
    fn space_pacing_zero_disables_class() {
        // 3.7.12.1 — zero is valid and disables pacing for that class.
        let alice = alice_key();
        let (mut state, space_id) = create_space(&alice);
        let ev = sign_event(
            build_space_pacing_event(&alice, &space_id, vec![], 0, 0),
            &alice,
        );
        state.apply_event(&ev, "").unwrap();
        assert_eq!(state.human_pacing_ms, 0);
        assert_eq!(state.ai_pacing_ms, 0);
    }

    #[test]
    fn space_pacing_rejected_when_sender_not_owner() {
        // Alice creates the space; Bob (a member) attempts the update.
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, space_id) = create_space(&alice);
        let bob_id = sender_id(&bob);

        // Bob becomes a member.
        state.pending_invites.insert(xid(&bob_id), PendingInvite::from_role(Role::Member));
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev, "").unwrap();

        let attempt = sign_event(
            build_space_pacing_event(&bob, &space_id, vec![], 9999, 9999),
            &bob,
        );
        let err = state.apply_event(&attempt, "").unwrap_err();
        assert!(matches!(err, SpaceError::PermissionDenied(_)));
        // Pacing values unchanged.
        assert_eq!(state.human_pacing_ms, DEFAULT_HUMAN_PACING_MS);
        assert_eq!(state.ai_pacing_ms, DEFAULT_AI_PACING_MS);
    }

    #[test]
    fn space_pacing_rejected_when_field_missing() {
        let alice = alice_key();
        let (mut state, space_id) = create_space(&alice);
        // Hand-build an event with only one field present.
        let mut ev = Event::new(
            EventType::StateSpacePacing,
            sender_xgid(&alice),
            empty_room_xgid(),
            SpaceXgid::from_xgid(Xgid::new(space_id)),
            vec![],
            now(),
            json!({ "human_pacing_ms": 1000 }), // ai_pacing_ms omitted
        );
        ev = sign_event(ev, &alice);
        let err = state.apply_event(&ev, "").unwrap_err();
        assert!(matches!(err, SpaceError::MissingField("ai_pacing_ms")));
    }

    #[test]
    fn dm_space_create_applies_default_pacing() {
        // DM Spaces inherit the same defaults (spec 3.7.12.6).
        let alice = alice_key();
        let bob = bob_key();
        let bob_id = sender_id(&bob);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &bob_id, HOME), &alice);
        let (state, _, _) = SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();
        assert_eq!(state.human_pacing_ms, DEFAULT_HUMAN_PACING_MS);
        assert_eq!(state.ai_pacing_ms, DEFAULT_AI_PACING_MS);
    }

    // ── Temperature property (spec 3.7.13) ────────────────────────────────

    /// Helper: build a space, invite Bob (as moderator), then Charlie (as member).
    /// Returns (state, space_id, room_id, alice, bob, charlie).
    fn make_space_with_three_members() -> (
        SpaceState,
        String,
        String,
        SigningKey,
        SigningKey,
        SigningKey,
    ) {
        let alice = alice_key();
        let bob = bob_key();
        let charlie = SigningKey::from_bytes(&[3u8; 32]);
        let (mut state, space_id) = create_space(&alice);
        let bob_id = sender_id(&bob);
        let charlie_id = sender_id(&charlie);

        // Room
        let room_ev = sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        let room_id = event_id_str(&room_ev);
        state.apply_event(&room_ev, "").unwrap();

        // Invite Bob as moderator.
        state.pending_invites.insert(xid(&bob_id), PendingInvite::from_role(Role::Moderator));
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev, "").unwrap();

        // Invite Charlie as plain member.
        state.pending_invites.insert(xid(&charlie_id), PendingInvite::from_role(Role::Member));
        let join2 = sign_event(
            build_membership_event(&charlie, &space_id, "", EventType::MembershipJoin, json!({})),
            &charlie,
        );
        state.apply_event(&join2, "").unwrap();

        (state, space_id, room_id, alice, bob, charlie)
    }

    #[test]
    fn space_create_defaults_visibility_to_moderator() {
        let (state, _) = create_space(&alice_key());
        assert_eq!(state.member_temperature_visibility, DEFAULT_MEMBER_TEMPERATURE_VISIBILITY);
        assert_eq!(state.member_temperature_visibility, VISIBILITY_MODERATOR);
    }

    #[test]
    fn space_visibility_updated_by_owner() {
        let alice = alice_key();
        let (mut state, space_id) = create_space(&alice);
        let ev = sign_event(
            build_space_temperature_visibility_event(&alice, &space_id, vec![], VISIBILITY_EVERYONE),
            &alice,
        );
        state.apply_event(&ev, "").unwrap();
        assert_eq!(state.member_temperature_visibility, VISIBILITY_EVERYONE);
    }

    #[test]
    fn space_visibility_update_rejected_when_sender_not_owner() {
        let (mut state, space_id, _, _, bob, _) = make_space_with_three_members();
        let ev = sign_event(
            build_space_temperature_visibility_event(&bob, &space_id, vec![], VISIBILITY_EVERYONE),
            &bob,
        );
        let err = state.apply_event(&ev, "").unwrap_err();
        assert!(matches!(err, SpaceError::PermissionDenied(_)));
        assert_eq!(state.member_temperature_visibility, VISIBILITY_MODERATOR);
    }

    #[test]
    fn space_visibility_unknown_value_stored_verbatim_but_filtered_as_moderator() {
        // Spec 3.7.13.3: open enum — Node accepts unknown values but enforces
        // moderator behaviour at the filter.
        let alice = alice_key();
        let (mut state, space_id) = create_space(&alice);
        let ev = sign_event(
            build_space_temperature_visibility_event(&alice, &space_id, vec![], "future_value"),
            &alice,
        );
        state.apply_event(&ev, "").unwrap();
        assert_eq!(state.member_temperature_visibility, "future_value");
    }

    #[test]
    fn moderator_visibility_filters_correctly() {
        let (state, _, _, alice, bob, charlie) = make_space_with_three_members();
        let alice_id = sender_id(&alice);
        let bob_id = sender_id(&bob);
        let charlie_id = sender_id(&charlie);
        // Default visibility = moderator.
        // Subject sees themselves regardless of recipient role.
        assert!(should_include_member_temperature(&state, &alice_id, &alice_id));
        // Moderator sees a plain member's temperature.
        assert!(should_include_member_temperature(&state, &bob_id, &charlie_id));
        // Owner (Alice) sees Bob's and Charlie's temperatures.
        assert!(should_include_member_temperature(&state, &alice_id, &bob_id));
        assert!(should_include_member_temperature(&state, &alice_id, &charlie_id));
        // A plain member (Charlie) does NOT see Bob's temperature.
        assert!(!should_include_member_temperature(&state, &charlie_id, &bob_id));
    }

    #[test]
    fn everyone_visibility_includes_for_all_members() {
        let alice = alice_key();
        let (mut state, space_id, _, _, bob, charlie) = make_space_with_three_members_for(alice);
        let ev = sign_event(
            build_space_temperature_visibility_event(&bob, &space_id, vec![], VISIBILITY_EVERYONE),
            &bob,
        );
        // Bob is not the owner — would be rejected.
        let _ = state.apply_event(&ev, "");
        // Use the owner path: directly mutate (testing the filter, not the event handler).
        state.member_temperature_visibility = VISIBILITY_EVERYONE.to_string();
        let bob_id = sender_id(&bob);
        let charlie_id = sender_id(&charlie);
        // Charlie (plain member) now sees Bob's temperature.
        assert!(should_include_member_temperature(&state, &charlie_id, &bob_id));
        assert!(should_include_member_temperature(&state, &bob_id, &charlie_id));
    }

    /// Same as make_space_with_three_members but takes alice key as arg (avoids re-call).
    fn make_space_with_three_members_for(
        alice: SigningKey,
    ) -> (SpaceState, String, String, SigningKey, SigningKey, SigningKey) {
        let bob = bob_key();
        let charlie = SigningKey::from_bytes(&[3u8; 32]);
        let ev = sign_event(build_space_create_event(&alice, "Test Space", None, 1, HOME, None, false), &alice);
        let space_id = event_id_str(&ev);
        let mut state = SpaceState::from_space_create(&ev).unwrap();
        let bob_id = sender_id(&bob);
        let charlie_id = sender_id(&charlie);

        let room_ev = sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        let room_id = event_id_str(&room_ev);
        state.apply_event(&room_ev, "").unwrap();

        state.pending_invites.insert(xid(&bob_id), PendingInvite::from_role(Role::Moderator));
        let join_b = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_b, "").unwrap();

        state.pending_invites.insert(xid(&charlie_id), PendingInvite::from_role(Role::Member));
        let join_c = sign_event(
            build_membership_event(&charlie, &space_id, "", EventType::MembershipJoin, json!({})),
            &charlie,
        );
        state.apply_event(&join_c, "").unwrap();
        (state, space_id, room_id, alice, bob, charlie)
    }

    #[test]
    fn self_only_visibility_blocks_moderator_and_owner() {
        let (mut state, _, _, alice, bob, charlie) = make_space_with_three_members();
        state.member_temperature_visibility = VISIBILITY_SELF_ONLY.to_string();
        let alice_id = sender_id(&alice);
        let bob_id = sender_id(&bob);
        let charlie_id = sender_id(&charlie);
        // Subject still sees themselves.
        assert!(should_include_member_temperature(&state, &charlie_id, &charlie_id));
        // Owner does NOT see Charlie's (self_only).
        assert!(!should_include_member_temperature(&state, &alice_id, &charlie_id));
        // Moderator does NOT see Charlie's (self_only).
        assert!(!should_include_member_temperature(&state, &bob_id, &charlie_id));
    }

    #[test]
    fn mute_by_moderator_accepted() {
        let (mut state, space_id, room_id, _, bob, charlie) = make_space_with_three_members();
        let charlie_id = sender_id(&charlie);
        let cooldown = "2026-05-15T14:00:00.000Z";
        let mute_ev = sign_event(
            build_membership_mute_event(
                &bob,
                &space_id,
                &room_id,
                vec![],
                &charlie_id,
                "Disturbing the room",
                cooldown,
            ),
            &bob,
        );
        state.apply_event(&mute_ev, "").unwrap();
        assert_eq!(state.active_mutes.get(charlie_id.as_str()), Some(&cooldown.to_string()));
    }

    #[test]
    fn mute_by_member_rejected() {
        let (mut state, space_id, room_id, _, _, charlie) = make_space_with_three_members();
        let bob_target = sender_id(&bob_key());
        let mute_ev = sign_event(
            build_membership_mute_event(
                &charlie,
                &space_id,
                &room_id,
                vec![],
                &bob_target,
                "Annoyance",
                "2026-05-15T14:00:00.000Z",
            ),
            &charlie,
        );
        let err = state.apply_event(&mute_ev, "").unwrap_err();
        assert!(matches!(err, SpaceError::PermissionDenied(_)));
        assert!(state.active_mutes.is_empty());
    }

    #[test]
    fn mute_with_auto_temperature_reason_is_recognised() {
        use xgen_common::wire::REASON_AUTO_TEMPERATURE;
        // Spec 3.7.13.6 — automated mutes share the standard mute logic; the
        // reason value is preserved on the DAG event for audit but triggers no
        // additional protocol behaviour.
        let (mut state, space_id, room_id, _, bob, charlie) = make_space_with_three_members();
        let charlie_id = sender_id(&charlie);
        let mute_ev = sign_event(
            build_membership_mute_event(
                &bob,
                &space_id,
                &room_id,
                vec![],
                &charlie_id,
                REASON_AUTO_TEMPERATURE,
                "2026-05-15T13:00:00.000Z",
            ),
            &bob,
        );
        state.apply_event(&mute_ev, "").unwrap();
        assert!(state.active_mutes.contains_key(charlie_id.as_str()));
        // Reason value preserved on the DAG event content for audit.
        assert_eq!(
            mute_ev.content["reason"].as_str(),
            Some(REASON_AUTO_TEMPERATURE)
        );
    }

    #[test]
    fn mute_missing_field_rejected() {
        let (mut state, space_id, room_id, _, bob, _) = make_space_with_three_members();
        let mut ev = Event::new(
            EventType::MembershipMute,
            sender_xgid(&bob),
            RoomXgid::from_xgid(Xgid::new(room_id)),
            SpaceXgid::from_xgid(Xgid::new(space_id)),
            vec![],
            now(),
            // Missing cooldown_until.
            json!({"target_identity": "xgen://pubkey/ed25519:X", "reason": "x"}),
        );
        ev = sign_event(ev, &bob);
        let err = state.apply_event(&ev, "").unwrap_err();
        assert!(matches!(err, SpaceError::MissingField("cooldown_until")));
    }

    #[test]
    fn kick_with_auto_temperature_reason_uses_standard_path() {
        // Spec 3.7.13.6: auto_temperature kick follows the standard kick logic;
        // reason is preserved on the DAG event for audit.
        use xgen_common::wire::REASON_AUTO_TEMPERATURE;
        let (mut state, space_id, room_id, _, bob, charlie) = make_space_with_three_members();
        let charlie_id = sender_id(&charlie);
        let kick_ev = sign_event(
            build_membership_event(
                &bob,
                &space_id,
                "",
                EventType::MembershipKick,
                json!({
                    "target_identity": charlie_id.clone(),
                    "reason": REASON_AUTO_TEMPERATURE,
                    "cooldown_until": "2026-05-15T14:00:00.000Z",
                }),
            ),
            &bob,
        );
        state.apply_event(&kick_ev, "").unwrap();
        assert!(!state.is_member(charlie_id.as_str()), "kicked member removed");
        assert_eq!(
            kick_ev.content["reason"].as_str(),
            Some(REASON_AUTO_TEMPERATURE),
            "reason preserved on DAG event"
        );
        // Room reference avoids unused-binding warnings.
        let _ = room_id;
    }

    // ── M3 (spec 3.6.10.6) — operator role, invited_by, resolve_operator ─────

    #[test]
    fn invite_then_join_captures_invited_by() {
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, space_id) = create_space(&alice);
        let alice_id = sender_id(&alice);
        let bob_id = sender_id(&bob);

        let invite_ev = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            &alice,
        );
        state.apply_event(&invite_ev, "").unwrap();
        let pending = state.pending_invites.get(bob_id.as_str()).unwrap();
        assert_eq!(
            pending.invited_by.as_ref().map(|i| i.as_str()),
            Some(alice_id.as_str())
        );

        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev, "").unwrap();
        let bob_member = state.members.get(bob_id.as_str()).unwrap();
        assert_eq!(
            bob_member.invited_by.as_ref().map(|i| i.as_str()),
            Some(alice_id.as_str())
        );
    }

    #[test]
    fn owner_has_no_invited_by() {
        let alice = alice_key();
        let (state, _) = create_space(&alice);
        let alice_id = sender_id(&alice);
        let owner = state.members.get(alice_id.as_str()).unwrap();
        assert!(owner.invited_by.is_none());
    }

    /// Helper for resolve_operator tests: alice owner, bob invited as member,
    /// carol invited as admin, AI 'dave' invited as member by alice. Returns
    /// (state, space_id, alice_id, bob_id, carol_id, dave_id).
    fn make_space_with_ai_member() -> (SpaceState, String, String, String, String, String) {
        let alice = alice_key();
        let bob = bob_key();
        let carol = SigningKey::from_bytes(&[7u8; 32]);
        let dave_ai = SigningKey::from_bytes(&[8u8; 32]);
        let (mut state, space_id) = create_space(&alice);
        let alice_id = sender_id(&alice);
        let bob_id = sender_id(&bob);
        let carol_id = sender_id(&carol);
        let dave_id = sender_id(&dave_ai);

        // Alice invites bob (member), carol (admin), dave (member, AI).
        for (target, role) in [(&bob_id, "member"), (&carol_id, "admin"), (&dave_id, "member")] {
            state
                .apply_event(&sign_event(
                    build_membership_event(
                        &alice,
                        &space_id,
                        "",
                        EventType::MembershipInvite,
                        json!({ "target_identity": target, "role": role }),
                    ),
                    &alice,
                ), "")
                .unwrap();
        }
        for joiner_key in [&bob, &carol, &dave_ai] {
            state
                .apply_event(&sign_event(
                    build_membership_event(joiner_key, &space_id, "", EventType::MembershipJoin, json!({})),
                    joiner_key,
                ), "")
                .unwrap();
        }
        (state, space_id, alice_id, bob_id, carol_id, dave_id)
    }

    #[test]
    fn resolve_operator_falls_back_to_inviter_when_no_delegation() {
        // Step 2 of the fall-upward algorithm: no stored delegation, dave's
        // inviter alice is a member → alice is operator.
        let (state, _, alice_id, _, _, dave_id) = make_space_with_ai_member();
        assert_eq!(state.resolve_operator(&dave_id).as_deref(), Some(alice_id.as_str()));
    }

    #[test]
    fn resolve_operator_returns_delegate_when_stored() {
        // Step 1 of the fall-upward algorithm: delegation hits, delegate is
        // still a member → carol is operator.
        let (mut state, _, _, _, carol_id, dave_id) = make_space_with_ai_member();
        state.ai_operator_delegations.insert(xid(&dave_id), xid(&carol_id));
        assert_eq!(state.resolve_operator(&dave_id).as_deref(), Some(carol_id.as_str()));
    }

    #[test]
    fn resolve_operator_skips_delegate_who_left_falls_back_to_inviter() {
        // Step 1 transparently skips a delegate who is no longer a member.
        // After carol leaves, resolution falls through to step 2 (inviter alice).
        let (mut state, space_id, alice_id, _, carol_id, dave_id) = make_space_with_ai_member();
        state.ai_operator_delegations.insert(xid(&dave_id), xid(&carol_id));

        // carol leaves the Space.
        let carol_key = SigningKey::from_bytes(&[7u8; 32]);
        let leave_ev = sign_event(
            build_membership_event(&carol_key, &space_id, "", EventType::MembershipLeave, json!({})),
            &carol_key,
        );
        state.apply_event(&leave_ev, "").unwrap();
        assert!(!state.is_member(carol_id.as_str()));
        // Delegation record still in place, but resolution skips it.
        assert!(state.ai_operator_delegations.contains_key(dave_id.as_str()));

        assert_eq!(state.resolve_operator(&dave_id).as_deref(), Some(alice_id.as_str()));
    }

    #[test]
    fn resolve_operator_falls_to_owner_when_inviter_gone() {
        // Step 3 of the fall-upward algorithm: with no delegation and the
        // inviter no longer a member, resolution returns the Space owner.
        // Synthesise this by clearing the dave member's invited_by to a
        // non-existent identity.
        let (mut state, _, alice_id, _, _, dave_id) = make_space_with_ai_member();
        if let Some(m) = state.members.get_mut(dave_id.as_str()) {
            m.invited_by = Some(xid("xgen://pubkey/ed25519:GHOST"));
        }
        assert_eq!(state.resolve_operator(&dave_id).as_deref(), Some(alice_id.as_str()));
    }

    #[test]
    fn resolve_operator_returns_none_for_non_member() {
        let (state, _, _, _, _, _) = make_space_with_ai_member();
        assert!(state.resolve_operator("xgen://pubkey/ed25519:STRANGER").is_none());
    }

    #[test]
    fn apply_ai_operator_delegate_writes_delegation() {
        // Build the space with known keys (avoids the hidden-key shape of the
        // make_space_with_ai_member fixture that owns its keypairs internally).
        let owner_key = alice_key();
        let (mut s2, sid) = create_space(&owner_key);
        let admin_key = bob_key();
        let ai_key = SigningKey::from_bytes(&[42u8; 32]);
        let new_op_key = SigningKey::from_bytes(&[43u8; 32]);
        let admin_id = sender_id(&admin_key);
        let ai_id = sender_id(&ai_key);
        let new_op_id = sender_id(&new_op_key);

        for (target, role, joiner) in [
            (&admin_id, "admin", &admin_key),
            (&ai_id, "member", &ai_key),
            (&new_op_id, "member", &new_op_key),
        ] {
            s2.apply_event(&sign_event(
                build_membership_event(
                    &owner_key,
                    &sid,
                    "",
                    EventType::MembershipInvite,
                    json!({ "target_identity": target, "role": role }),
                ),
                &owner_key,
            ), "")
            .unwrap();
            s2.apply_event(&sign_event(
                build_membership_event(joiner, &sid, "", EventType::MembershipJoin, json!({})),
                joiner,
            ), "")
            .unwrap();
        }

        let delegate_ev = sign_event(
            build_state_ai_operator_delegate_event(
                &admin_key, // admin can delegate
                &sid,
                vec![],
                &ai_id,
                &new_op_id,
            ),
            &admin_key,
        );
        s2.apply_event(&delegate_ev, "").unwrap();
        assert_eq!(
            s2.ai_operator_delegations.get(ai_id.as_str()).map(|s| s.as_str()),
            Some(new_op_id.as_str())
        );
    }

    #[test]
    fn apply_ai_operator_delegate_rejects_non_admin_signer() {
        // Defence-in-depth at apply_event time: a moderator-level signer is
        // rejected even if upstream validation somehow let the event through.
        let owner_key = alice_key();
        let (mut state, sid) = create_space(&owner_key);
        let mod_key = bob_key();
        let mod_id = sender_id(&mod_key);

        state
            .apply_event(&sign_event(
                build_membership_event(
                    &owner_key,
                    &sid,
                    "",
                    EventType::MembershipInvite,
                    json!({ "target_identity": mod_id, "role": "moderator" }),
                ),
                &owner_key,
            ), "")
            .unwrap();
        state
            .apply_event(&sign_event(
                build_membership_event(&mod_key, &sid, "", EventType::MembershipJoin, json!({})),
                &mod_key,
            ), "")
            .unwrap();

        let ev = sign_event(
            build_state_ai_operator_delegate_event(
                &mod_key,
                &sid,
                vec![],
                "xgen://pubkey/ed25519:AI",
                "xgen://pubkey/ed25519:OP",
            ),
            &mod_key,
        );
        let err = state.apply_event(&ev, "").unwrap_err();
        assert!(matches!(err, SpaceError::PermissionDenied(_)));
    }

    #[test]
    fn apply_ai_operator_revoke_clears_delegation() {
        let owner_key = alice_key();
        let (mut state, sid) = create_space(&owner_key);
        let owner_id = sender_id(&owner_key);
        let ai_key = SigningKey::from_bytes(&[44u8; 32]);
        let ai_id = sender_id(&ai_key);
        let new_op_key = SigningKey::from_bytes(&[45u8; 32]);
        let new_op_id = sender_id(&new_op_key);

        for (target, role, joiner) in [
            (&ai_id, "member", &ai_key),
            (&new_op_id, "member", &new_op_key),
        ] {
            state
                .apply_event(&sign_event(
                    build_membership_event(
                        &owner_key,
                        &sid,
                        "",
                        EventType::MembershipInvite,
                        json!({ "target_identity": target, "role": role }),
                    ),
                    &owner_key,
                ), "")
                .unwrap();
            state
                .apply_event(&sign_event(
                    build_membership_event(joiner, &sid, "", EventType::MembershipJoin, json!({})),
                    joiner,
                ), "")
                .unwrap();
        }

        state.ai_operator_delegations.insert(xid(&ai_id), xid(&new_op_id));
        let revoke_ev = sign_event(
            build_state_ai_operator_revoke_event(&owner_key, &sid, vec![], &ai_id),
            &owner_key,
        );
        state.apply_event(&revoke_ev, "").unwrap();
        assert!(!state.ai_operator_delegations.contains_key(ai_id.as_str()));
        // Resolution now falls through to step 2 (inviter = owner).
        assert_eq!(state.resolve_operator(&ai_id).as_deref(), Some(owner_id.as_str()));
    }

    // ── D-075 bidirectional federation_nodes vantage-aware applier ───────
    //
    // Six unit tests covering both vantage branches (sender-vantage and
    // content.node_id-vantage), mirror property, third-party observer,
    // DM constraint preservation, and missing-field rejection. The
    // regression lock for the pre-D-075 bidirectional bug is
    // `apply_federation_add_peer_event_adds_sender` + the mirror test.
    // Integration-level regression lock is Phase 9 Scenario 1
    // (`xgen-node/src/tests/phase9_two_node_smoke.rs::scenario_1_two_node_push_smoke`).

    fn make_federation_event(
        asserter_key: &SigningKey,
        peer_id: &str,
        space_id: &str,
    ) -> Event {
        sign_event(
            build_federation_add_event(
                asserter_key,
                space_id,
                vec![],
                peer_id,
                "session-x",
                "1",
                "json",
            ),
            asserter_key,
        )
    }

    #[test]
    fn apply_federation_add_my_event_adds_content_node_id() {
        // Vantage: A (the asserter, my_node_id == sender). Else branch fires.
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, _) = create_space(&alice);
        let a_id = sender_id(&alice);
        let b_id = sender_id(&bob);
        let event = make_federation_event(&alice, &b_id, state.space_id.as_str());
        state.apply_event(&event, &a_id).unwrap();
        assert_eq!(state.federation_nodes, vec![nxd(&b_id)]);
    }

    #[test]
    fn apply_federation_add_peer_event_adds_sender() {
        // Vantage: B (the named other-party, my_node_id == content.node_id).
        // The if branch fires — regression lock for the bidirectional bug.
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, _) = create_space(&alice);
        let a_id = sender_id(&alice);
        let b_id = sender_id(&bob);
        let event = make_federation_event(&alice, &b_id, state.space_id.as_str());
        state.apply_event(&event, &b_id).unwrap();
        assert_eq!(state.federation_nodes, vec![nxd(&a_id)]);
    }

    #[test]
    fn apply_federation_add_two_vantages_mirror() {
        // Apply the SAME event with two SpaceStates, one from A's vantage and
        // one from B's vantage; federation_nodes end up as mirrors of each
        // other. D-075's "asymmetric branches, symmetric outcomes" property.
        let alice = alice_key();
        let bob = bob_key();
        let (state_a_base, _) = create_space(&alice);
        let mut state_a = state_a_base.clone();
        let mut state_b = state_a_base;
        let a_id = sender_id(&alice);
        let b_id = sender_id(&bob);
        let event = make_federation_event(&alice, &b_id, state_a.space_id.as_str());
        state_a.apply_event(&event, &a_id).unwrap();
        state_b.apply_event(&event, &b_id).unwrap();
        assert_eq!(state_a.federation_nodes, vec![nxd(&b_id)]);
        assert_eq!(state_b.federation_nodes, vec![nxd(&a_id)]);
    }

    #[test]
    fn apply_federation_add_third_party_observer_adds_content_node_id() {
        // Vantage: C (third party with multi-Space visibility observing an
        // A↔B federation_add). Else branch fires; relevant peer is
        // content.node_id (B), not sender (A) — observer takes the
        // sender-perspective view, which is the legacy verbatim behaviour
        // preserved for non-Node observers and unrelated third parties.
        let alice = alice_key();
        let bob = bob_key();
        let carol_key = SigningKey::from_bytes(&[77u8; 32]);
        let (mut state, _) = create_space(&alice);
        let b_id = sender_id(&bob);
        let c_id = sender_id(&carol_key);
        let event = make_federation_event(&alice, &b_id, state.space_id.as_str());
        state.apply_event(&event, &c_id).unwrap();
        assert_eq!(state.federation_nodes, vec![nxd(&b_id)]);
    }

    #[test]
    fn apply_federation_add_dm_constraint_preserved() {
        // DM Spaces still reject federation_add regardless of vantage.
        let alice = alice_key();
        let bob = bob_key();
        let bob_id = sender_id(&bob);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &bob_id, HOME), &alice);
        let (mut state, _, _) = SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();
        let a_id = sender_id(&alice);
        let event = make_federation_event(&alice, &bob_id, state.space_id.as_str());
        let err = state.apply_event(&event, &a_id).unwrap_err();
        assert_eq!(err, SpaceError::DmFederationNotAllowed);
        assert!(state.federation_nodes.is_empty());
    }

    #[test]
    fn apply_federation_add_missing_field_rejected() {
        // Missing content.node_id is still rejected with MissingField.
        let alice = alice_key();
        let (mut state, _) = create_space(&alice);
        let a_id = sender_id(&alice);
        // Build an event with content missing the node_id field.
        let event = sign_event(
            Event::new(
                EventType::StateFederationAdd,
                sender_xgid(&alice),
                empty_room_xgid(),
                state.space_id.clone(),
                vec![],
                now(),
                json!({ "session_id": "session-x" }),
            ),
            &alice,
        );
        let err = state.apply_event(&event, &a_id).unwrap_err();
        assert_eq!(err, SpaceError::MissingField("node_id"));
        assert!(state.federation_nodes.is_empty());
    }

    // ── Thread model appliers (Arc E PG-08) ──────────────────────────────────

    /// Owner creates a Space + Room, then a Thread; returns (state, space_id,
    /// room_id, thread_id).
    fn create_thread(key: &SigningKey) -> (SpaceState, String, String, String) {
        let (mut state, space_id) = create_space(key);
        let room_ev = sign_event(build_room_create_event(key, &space_id, "general", None), key);
        state.apply_event(&room_ev, "").unwrap();
        let room_id = event_id_str(&room_ev);
        let thread_ev = sign_event(
            build_thread_create_event(key, &space_id, &room_id, vec![room_id.clone()], Some("Q3 plan"), 1),
            key,
        );
        state.apply_event(&thread_ev, "").unwrap();
        let thread_id = thread_id_from_event_id(&event_id_str(&thread_ev));
        (state, space_id, room_id, thread_id)
    }

    #[test]
    fn thread_create_inserts_open_thread() {
        let key = alice_key();
        let (state, _space_id, room_id, thread_id) = create_thread(&key);
        assert!(thread_id.starts_with("xgen://thread/sha256:"));
        let thread = state.threads.get(&thread_id).expect("thread inserted");
        assert_eq!(thread.status, ThreadStatus::Open);
        assert_eq!(thread.title.as_deref(), Some("Q3 plan"));
        assert_eq!(thread.room_id.as_str(), room_id);
        assert_eq!(thread.auth_tier_min, 1);
        assert_eq!(thread.created_by.as_str(), sender_id(&key).as_str());
    }

    #[test]
    fn thread_resolved_then_archived_transitions_and_is_idempotent() {
        let key = alice_key();
        let (mut state, space_id, room_id, thread_id) = create_thread(&key);

        let resolve = sign_event(
            build_thread_resolved_event(&key, &space_id, &room_id, &thread_id, vec![room_id.clone()]),
            &key,
        );
        state.apply_event(&resolve, "").unwrap();
        assert_eq!(state.threads.get(&thread_id).unwrap().status, ThreadStatus::Resolved);

        // Idempotent — re-applying the same transition is a no-op.
        state.apply_event(&resolve, "").unwrap();
        assert_eq!(state.threads.get(&thread_id).unwrap().status, ThreadStatus::Resolved);

        let archive = sign_event(
            build_thread_archived_event(&key, &space_id, &room_id, &thread_id, vec![room_id.clone()]),
            &key,
        );
        state.apply_event(&archive, "").unwrap();
        assert_eq!(state.threads.get(&thread_id).unwrap().status, ThreadStatus::Archived);
    }

    #[test]
    fn thread_status_on_unknown_thread_is_noop() {
        let key = alice_key();
        let (mut state, space_id, room_id, _thread_id) = create_thread(&key);
        let resolve = sign_event(
            build_thread_resolved_event(
                &key,
                &space_id,
                &room_id,
                "xgen://thread/sha256:doesnotexist",
                vec![room_id.clone()],
            ),
            &key,
        );
        // Silent no-op (sibling to other state arms): Ok, the real thread is untouched.
        assert!(state.apply_event(&resolve, "").is_ok());
        assert_eq!(state.threads.len(), 1);
    }

    #[test]
    fn thread_id_derivation_swaps_hash_prefix() {
        assert_eq!(
            thread_id_from_event_id("xgen://hash/sha256:abc123"),
            "xgen://thread/sha256:abc123"
        );
        // Defensive: a non-hash input is returned unchanged.
        assert_eq!(thread_id_from_event_id("plain"), "plain");
    }
}
