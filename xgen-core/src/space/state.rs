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
    space::membership::{can_ban, can_create_room, can_invite, can_kick, can_mute, Role},
    wire::{
        canonical::canonical_event_bytes,
        types::{
            Event, EventType, DEFAULT_AI_PACING_MS, DEFAULT_HUMAN_PACING_MS,
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

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct PendingInvite {
    pub role: Role,
    pub invited_by: Option<IdentityXgid>,
}

impl PendingInvite {
    /// Convenience for tests and pre-M3 replay paths that don't have an inviter.
    pub fn from_role(role: Role) -> Self {
        Self { role, invited_by: None }
    }
}

#[derive(Debug, Clone)]
pub struct RoomState {
    pub room_id: RoomXgid,
    pub space_id: SpaceXgid,
    pub name: String,
    pub topic: Option<String>,
    pub members: HashSet<IdentityXgid>,
}

#[derive(Debug, Clone)]
pub struct SpaceState {
    pub space_id: SpaceXgid,
    pub name: Option<String>,
    pub topic: Option<String>,
    pub auth_tier: u32,
    pub max_event_size: Option<u64>,
    pub home_node: NodeXgid,
    pub owner_id: IdentityXgid,
    pub is_dm: bool,
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
        };
        room.members.insert(creator.clone());

        let mut pending_invites = HashMap::new();
        // The DM creator (Space owner) is the implicit inviter of the second member.
        pending_invites.insert(
            invitee,
            PendingInvite { role: Role::Member, invited_by: Some(creator.clone()) },
        );

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
        };

        Ok((state, room_event, invite_event))
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
            // State updates (accepted silently for forward-compat).
            EventType::StateSpaceUpdate | EventType::StateRoomUpdate => Ok(()),
            _ => Ok(()), // unrecognised events silently ignored
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
            RoomState { room_id, space_id: self.space_id.clone(), name, topic, members },
        );
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
        // M3: capture the inviter so resolve_operator can fall back to it.
        self.pending_invites.insert(
            target,
            PendingInvite { role, invited_by: Some(actor.clone()) },
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

    fn alice_key() -> SigningKey {
        keypair::generate()
    }

    fn bob_key() -> SigningKey {
        keypair::generate()
    }

    const HOME: &str = "xgen://pubkey/ed25519:NODE";

    fn create_space(key: &SigningKey) -> (SpaceState, String) {
        let ev = sign_event(build_space_create_event(key, "Test Space", None, 1, HOME), key);
        let space_id = ev.event_id.clone().unwrap();
        let state = SpaceState::from_space_create(&ev).unwrap();
        (state, space_id)
    }

    #[test]
    fn space_create_sets_owner() {
        let key = alice_key();
        let (state, _) = create_space(&key);
        let alice_id = sender_id(&key);
        assert!(state.is_member(&alice_id));
        assert_eq!(state.member_role(&alice_id), Some(&Role::Owner));
    }

    #[test]
    fn space_create_event_id_is_space_id() {
        let key = alice_key();
        let (state, space_id) = create_space(&key);
        assert_eq!(state.space_id, space_id);
    }

    #[test]
    fn room_create_by_owner_succeeds() {
        let key = alice_key();
        let (mut state, space_id) = create_space(&key);
        let room_ev = sign_event(build_room_create_event(&key, &space_id, "general", None), &key);
        state.apply_event(&room_ev, "").unwrap();
        assert_eq!(state.rooms.len(), 1);
        let room_id = room_ev.event_id.unwrap();
        assert!(state.rooms.contains_key(&room_id));
    }

    #[test]
    fn room_create_event_records_space_create_as_predecessor() {
        let key = alice_key();
        let (_state, space_id) = create_space(&key);
        let room_ev = build_room_create_event(&key, &space_id, "general", None);
        assert_eq!(
            room_ev.prev_events,
            vec![space_id.clone()],
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
        state.pending_invites.insert(bob_id.clone(), PendingInvite::from_role(Role::Member));

        // Bob joins the space.
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev, "").unwrap();
        assert_eq!(state.member_role(&bob_id), Some(&Role::Member));

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
        assert!(state.pending_invites.contains_key(&bob_id));

        // Bob joins the space.
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev, "").unwrap();
        assert!(state.is_member(&bob_id));
        assert_eq!(state.member_role(&bob_id), Some(&Role::Member));
        assert!(!state.pending_invites.contains_key(&bob_id));
    }

    #[test]
    fn join_room_after_joining_space() {
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, space_id) = create_space(&alice);
        let bob_id = sender_id(&bob);

        // Alice creates a room.
        let room_ev = sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        let room_id = room_ev.event_id.clone().unwrap();
        state.apply_event(&room_ev, "").unwrap();

        // Bob joins space.
        state.pending_invites.insert(bob_id.clone(), PendingInvite::from_role(Role::Member));
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
        assert!(state.is_room_member(&bob_id, &room_id));
    }

    #[test]
    fn leave_removes_from_space_and_all_rooms() {
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, space_id) = create_space(&alice);
        let bob_id = sender_id(&bob);

        // Setup: room + Bob member.
        let room_ev = sign_event(build_room_create_event(&alice, &space_id, "gen", None), &alice);
        let room_id = room_ev.event_id.clone().unwrap();
        state.apply_event(&room_ev, "").unwrap();

        state.pending_invites.insert(bob_id.clone(), PendingInvite::from_role(Role::Member));
        state.apply_event(&sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        ), "").unwrap();
        state.apply_event(&sign_event(
            build_membership_event(&bob, &space_id, &room_id, EventType::MembershipJoin, json!({})),
            &bob,
        ), "").unwrap();
        assert!(state.is_room_member(&bob_id, &room_id));

        // Bob leaves the space.
        state.apply_event(&sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipLeave, json!({})),
            &bob,
        ), "").unwrap();
        assert!(!state.is_member(&bob_id));
        assert!(!state.is_room_member(&bob_id, &room_id));
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
        assert!(state.banned.contains(&bob_id));

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
        let ev = sign_event(build_space_create_event(&key, "Test", None, 1, HOME), &key);
        assert!(ev.event_id.is_some());
        assert!(ev.signature.is_some());
        assert!(verify_event_signature(&ev));
    }

    #[test]
    fn tampered_event_fails_verification() {
        let key = alice_key();
        let mut ev = sign_event(build_space_create_event(&key, "Test", None, 1, HOME), &key);
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
        assert!(state.pending_invites.contains_key(&bob_id));
        assert!(room_ev.event_id.is_some());
        assert_eq!(invite_ev.event_type, EventType::MembershipInvite);
    }

    // ── Layer 14 — DM Space Promotion tests ──────────────────────────────────

    fn make_dm_space_with_two_members() -> (SpaceState, String, SigningKey, SigningKey) {
        let alice = alice_key();
        let bob = bob_key();
        let bob_id = sender_id(&bob);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &bob_id, HOME), &alice);
        let space_id = create_ev.event_id.clone().unwrap();
        let (mut state, _, _) = SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();
        // Bob joins.
        state.pending_invites.insert(bob_id.clone(), PendingInvite::from_role(crate::space::membership::Role::Member));
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
        assert!(state.pending_invites.contains_key(&charlie_id));
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
        assert!(state.is_member(&alice_id));
        assert!(state.is_member(&bob_id));
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
        let mut ev = build_space_create_event(&key, "Test Space", None, 1, HOME);
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
        state.pending_invites.insert(bob_id.clone(), PendingInvite::from_role(Role::Member));
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
            sender_id(&alice),
            String::new(),
            space_id,
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
        let room_id = room_ev.event_id.clone().unwrap();
        state.apply_event(&room_ev, "").unwrap();

        // Invite Bob as moderator.
        state.pending_invites.insert(bob_id.clone(), PendingInvite::from_role(Role::Moderator));
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev, "").unwrap();

        // Invite Charlie as plain member.
        state.pending_invites.insert(charlie_id.clone(), PendingInvite::from_role(Role::Member));
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
        let ev = sign_event(build_space_create_event(&alice, "Test Space", None, 1, HOME), &alice);
        let space_id = ev.event_id.clone().unwrap();
        let mut state = SpaceState::from_space_create(&ev).unwrap();
        let bob_id = sender_id(&bob);
        let charlie_id = sender_id(&charlie);

        let room_ev = sign_event(build_room_create_event(&alice, &space_id, "general", None), &alice);
        let room_id = room_ev.event_id.clone().unwrap();
        state.apply_event(&room_ev, "").unwrap();

        state.pending_invites.insert(bob_id.clone(), PendingInvite::from_role(Role::Moderator));
        let join_b = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_b, "").unwrap();

        state.pending_invites.insert(charlie_id.clone(), PendingInvite::from_role(Role::Member));
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
        assert_eq!(state.active_mutes.get(&charlie_id), Some(&cooldown.to_string()));
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
        assert!(state.active_mutes.contains_key(&charlie_id));
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
            sender_id(&bob),
            room_id,
            space_id,
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
        assert!(!state.is_member(&charlie_id), "kicked member removed");
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
        let pending = state.pending_invites.get(&bob_id).unwrap();
        assert_eq!(pending.invited_by.as_deref(), Some(alice_id.as_str()));

        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev, "").unwrap();
        let bob_member = state.members.get(&bob_id).unwrap();
        assert_eq!(bob_member.invited_by.as_deref(), Some(alice_id.as_str()));
    }

    #[test]
    fn owner_has_no_invited_by() {
        let alice = alice_key();
        let (state, _) = create_space(&alice);
        let alice_id = sender_id(&alice);
        let owner = state.members.get(&alice_id).unwrap();
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
        state.ai_operator_delegations.insert(dave_id.clone(), carol_id.clone());
        assert_eq!(state.resolve_operator(&dave_id).as_deref(), Some(carol_id.as_str()));
    }

    #[test]
    fn resolve_operator_skips_delegate_who_left_falls_back_to_inviter() {
        // Step 1 transparently skips a delegate who is no longer a member.
        // After carol leaves, resolution falls through to step 2 (inviter alice).
        let (mut state, space_id, alice_id, _, carol_id, dave_id) = make_space_with_ai_member();
        state.ai_operator_delegations.insert(dave_id.clone(), carol_id.clone());

        // carol leaves the Space.
        let carol_key = SigningKey::from_bytes(&[7u8; 32]);
        let leave_ev = sign_event(
            build_membership_event(&carol_key, &space_id, "", EventType::MembershipLeave, json!({})),
            &carol_key,
        );
        state.apply_event(&leave_ev, "").unwrap();
        assert!(!state.is_member(&carol_id));
        // Delegation record still in place, but resolution skips it.
        assert!(state.ai_operator_delegations.contains_key(&dave_id));

        assert_eq!(state.resolve_operator(&dave_id).as_deref(), Some(alice_id.as_str()));
    }

    #[test]
    fn resolve_operator_falls_to_owner_when_inviter_gone() {
        // Step 3 of the fall-upward algorithm: with no delegation and the
        // inviter no longer a member, resolution returns the Space owner.
        // Synthesise this by clearing the dave member's invited_by to a
        // non-existent identity.
        let (mut state, _, alice_id, _, _, dave_id) = make_space_with_ai_member();
        if let Some(m) = state.members.get_mut(&dave_id) {
            m.invited_by = Some("xgen://pubkey/ed25519:GHOST".to_string());
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
            s2.ai_operator_delegations.get(&ai_id).map(|s| s.as_str()),
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

        state.ai_operator_delegations.insert(ai_id.clone(), new_op_id.clone());
        let revoke_ev = sign_event(
            build_state_ai_operator_revoke_event(&owner_key, &sid, vec![], &ai_id),
            &owner_key,
        );
        state.apply_event(&revoke_ev, "").unwrap();
        assert!(!state.ai_operator_delegations.contains_key(&ai_id));
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
        let event = make_federation_event(&alice, &b_id, &state.space_id);
        state.apply_event(&event, &a_id).unwrap();
        assert_eq!(state.federation_nodes, vec![b_id]);
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
        let event = make_federation_event(&alice, &b_id, &state.space_id);
        state.apply_event(&event, &b_id).unwrap();
        assert_eq!(state.federation_nodes, vec![a_id]);
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
        let event = make_federation_event(&alice, &b_id, &state_a.space_id);
        state_a.apply_event(&event, &a_id).unwrap();
        state_b.apply_event(&event, &b_id).unwrap();
        assert_eq!(state_a.federation_nodes, vec![b_id.clone()]);
        assert_eq!(state_b.federation_nodes, vec![a_id.clone()]);
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
        let event = make_federation_event(&alice, &b_id, &state.space_id);
        state.apply_event(&event, &c_id).unwrap();
        assert_eq!(state.federation_nodes, vec![b_id]);
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
        let event = make_federation_event(&alice, &bob_id, &state.space_id);
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
                sender_id(&alice),
                String::new(),
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
}
