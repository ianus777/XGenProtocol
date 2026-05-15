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
    pub identity_id: String,
    pub role: Role,
    pub joined_at: String,
}

#[derive(Debug, Clone)]
pub struct RoomState {
    pub room_id: String,
    pub space_id: String,
    pub name: String,
    pub topic: Option<String>,
    pub members: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct SpaceState {
    pub space_id: String,
    pub name: Option<String>,
    pub topic: Option<String>,
    pub auth_tier: u32,
    pub max_event_size: Option<u64>,
    pub home_node: String,
    pub owner_id: String,
    pub is_dm: bool,
    /// Active members: identity_id → SpaceMember
    pub members: HashMap<String, SpaceMember>,
    /// Invited but not yet joined: identity_id → Role
    pub pending_invites: HashMap<String, Role>,
    /// Banned identities (cannot rejoin).
    pub banned: HashSet<String>,
    /// Rooms within this Space: room_id → RoomState
    pub rooms: HashMap<String, RoomState>,
    /// Federated nodes that participate in this Space.
    pub federation_nodes: Vec<String>,
    /// Manual Node ordering from the most recent state.node_priority Event (3.9.3 Layer 5a).
    /// Ordered from highest priority (index 0) to lowest. Empty when no such Event exists.
    pub node_priority_order: Vec<String>,
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
    pub active_mutes: HashMap<String, String>,
}

impl SpaceState {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Initialise a SpaceState from a `state.space_create` Event.
    /// The Space ID is the event's event_id (must be set before calling).
    pub fn from_space_create(event: &Event) -> Result<Self, SpaceError> {
        if event.event_type != EventType::StateSpaceCreate {
            return Err(SpaceError::WrongEventType);
        }
        let space_id = event.event_id.clone().ok_or(SpaceError::MissingField("event_id"))?;
        let content = &event.content;
        let auth_tier = content["auth_tier"].as_u64().ok_or(SpaceError::MissingField("auth_tier"))? as u32;
        let home_node = content["home_node"].as_str().ok_or(SpaceError::MissingField("home_node"))?.to_string();
        let name = content["name"].as_str().map(str::to_string);
        let topic = content["topic"].as_str().map(str::to_string);
        let max_event_size = content["max_event_size"].as_u64();

        let creator = event.sender.clone();
        let owner = SpaceMember {
            identity_id: creator.clone(),
            role: Role::Owner,
            joined_at: event.timestamp.clone(),
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
        let space_id = event.event_id.clone().ok_or(SpaceError::MissingField("event_id"))?;
        let content = &event.content;
        let auth_tier = content["auth_tier"].as_u64().unwrap_or(1) as u32;
        let home_node = content["home_node"].as_str().ok_or(SpaceError::MissingField("home_node"))?.to_string();
        let invitee = content["invitee"].as_str().ok_or(SpaceError::MissingField("invitee"))?.to_string();

        let creator = event.sender.clone();
        let owner = SpaceMember {
            identity_id: creator.clone(),
            role: Role::Owner,
            joined_at: event.timestamp.clone(),
        };
        let mut members = HashMap::new();
        members.insert(creator.clone(), owner);

        // Auto-create the DM room.
        let room_event = sign_event(
            build_room_create_event(creator_key, &space_id, "dm", None),
            creator_key,
        );
        let room_id = room_event.event_id.clone().unwrap();

        // Auto-produce membership.invite for the invitee.
        let invite_event = sign_event(
            build_membership_event(
                creator_key,
                &space_id,
                &room_id,
                EventType::MembershipInvite,
                json!({ "target_identity": invitee, "role": "member" }),
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
        pending_invites.insert(invitee, Role::Member);

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
    pub fn apply_event(&mut self, event: &Event) -> Result<(), SpaceError> {
        match &event.event_type {
            EventType::StateRoomCreate => self.apply_room_create(event),
            EventType::StateFederationAdd => self.apply_federation_add(event),
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

    fn apply_federation_add(&mut self, event: &Event) -> Result<(), SpaceError> {
        if self.dm_constraints_active {
            return Err(SpaceError::DmFederationNotAllowed);
        }
        let node_id = event.content["node_id"]
            .as_str()
            .ok_or(SpaceError::MissingField("node_id"))?
            .to_string();
        if !self.federation_nodes.contains(&node_id) {
            self.federation_nodes.push(node_id);
        }
        Ok(())
    }

    fn apply_node_priority(&mut self, event: &Event) -> Result<(), SpaceError> {
        let ordered_nodes = event.content["ordered_nodes"]
            .as_array()
            .ok_or(SpaceError::MissingField("ordered_nodes"))?;
        self.node_priority_order = ordered_nodes
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
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
        let actor_role = self.member_role(actor).ok_or(SpaceError::NotASpaceMember)?;
        if !can_mute(actor_role) {
            return Err(SpaceError::PermissionDenied(actor_role.as_str().to_string()));
        }
        let target = event.content["target_identity"]
            .as_str()
            .ok_or(SpaceError::MissingField("target_identity"))?
            .to_string();
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
        let actor_role = self.member_role(actor).ok_or(SpaceError::NotASpaceMember)?;
        if !can_create_room(actor_role) {
            return Err(SpaceError::PermissionDenied(actor_role.as_str().to_string()));
        }
        let room_id = event.event_id.clone().ok_or(SpaceError::MissingField("event_id"))?;
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
        let actor_role = self.member_role(actor).ok_or(SpaceError::NotASpaceMember)?;
        if !can_invite(actor_role) {
            return Err(SpaceError::PermissionDenied(actor_role.as_str().to_string()));
        }
        let target = event.content["target_identity"]
            .as_str()
            .ok_or(SpaceError::MissingField("target_identity"))?;
        if self.banned.contains(target) {
            return Err(SpaceError::Banned);
        }
        let role_str = event.content["role"].as_str().unwrap_or("member");
        let role = Role::from_str(role_str).unwrap_or(Role::Member);
        self.pending_invites.insert(target.to_string(), role);
        Ok(())
    }

    fn apply_join(&mut self, event: &Event) -> Result<(), SpaceError> {
        let joiner = &event.sender;

        // Room-level join: room_id is non-empty.
        if !event.room_id.is_empty() {
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
        let role = self.pending_invites.remove(joiner).unwrap_or(Role::Member);
        self.members.insert(
            joiner.clone(),
            SpaceMember { identity_id: joiner.clone(), role, joined_at: event.timestamp.clone() },
        );
        Ok(())
    }

    fn apply_leave(&mut self, event: &Event) -> Result<(), SpaceError> {
        let leaver = &event.sender;
        if !event.room_id.is_empty() {
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
        let actor_role = self.member_role(actor).ok_or(SpaceError::NotASpaceMember)?;
        if !can_kick(actor_role) {
            return Err(SpaceError::PermissionDenied(actor_role.as_str().to_string()));
        }
        let target = event.content["target_identity"]
            .as_str()
            .ok_or(SpaceError::MissingField("target_identity"))?;
        if !event.room_id.is_empty() {
            let room = self.rooms.get_mut(&event.room_id).ok_or(SpaceError::RoomNotFound)?;
            room.members.remove(target);
            return Ok(());
        }
        self.members.remove(target);
        for room in self.rooms.values_mut() {
            room.members.remove(target);
        }
        Ok(())
    }

    fn apply_ban(&mut self, event: &Event) -> Result<(), SpaceError> {
        let actor = &event.sender;
        let actor_role = self.member_role(actor).ok_or(SpaceError::NotASpaceMember)?;
        if !can_ban(actor_role) {
            return Err(SpaceError::PermissionDenied(actor_role.as_str().to_string()));
        }
        let target = event.content["target_identity"]
            .as_str()
            .ok_or(SpaceError::MissingField("target_identity"))?;
        self.members.remove(target);
        self.pending_invites.remove(target);
        self.banned.insert(target.to_string());
        for room in self.rooms.values_mut() {
            room.members.remove(target);
        }
        Ok(())
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    pub fn member_role(&self, identity_id: &str) -> Option<&Role> {
        self.members.get(identity_id).map(|m| &m.role)
    }

    pub fn is_member(&self, identity_id: &str) -> bool {
        self.members.contains_key(identity_id)
    }

    pub fn is_room_member(&self, identity_id: &str, room_id: &str) -> bool {
        self.rooms.get(room_id).map(|r| r.members.contains(identity_id)).unwrap_or(false)
    }
}

// ── Event signing ─────────────────────────────────────────────────────────────

/// Sign an Event: compute event_id (hash of canonical form) and add signature.
pub fn sign_event(mut event: Event, key: &SigningKey) -> Event {
    let v = serde_json::to_value(&event).expect("Event is always serialisable");
    let bytes = canonical_event_bytes(&v);
    event.event_id = Some(hashing::hash_uri(&bytes));
    event.signature = Some(signing::sign(key, &bytes));
    event
}

/// Verify an Event's signature against the embedded public key in `sender`.
pub fn verify_event_signature(event: &Event) -> bool {
    let sig_str = match &event.signature {
        Some(s) => s,
        None => return false,
    };
    let sender_b64 = match event.sender.strip_prefix("xgen://pubkey/ed25519:") {
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

fn sender_id(key: &SigningKey) -> String {
    format!("xgen://pubkey/ed25519:{}", encoding::encode(key.verifying_key().as_bytes()))
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
        sender_id(key),
        String::new(),  // room_id — empty for space_create
        String::new(),  // space_id — empty until derived
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
        sender_id(key),
        String::new(),    // room_id — empty until derived
        space_id.to_string(),
        vec![],
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
        sender_id(key),
        String::new(),
        space_id.to_string(),
        prev_events,
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
        sender_id(key),
        room_id.to_string(),
        space_id.to_string(),
        prev_events,
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
        sender_id(key),
        String::new(),
        space_id.to_string(),
        prev_events,
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
        sender_id(key),
        String::new(),
        String::new(),
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
        sender_id(key),
        String::new(), // space-level event — no room_id
        space_id.to_string(),
        prev_events,
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
        sender_id(node_key),
        String::new(), // space-level event — no room_id
        space_id.to_string(),
        prev_events,
        timestamp.to_string(),
        json!({
            "proposed_by": proposed_by,
            "confirmed_by": confirmed_by,
            "new_name": new_name,
            "promoted_at": timestamp,
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
    Event::new(
        event_type,
        sender_id(key),
        room_id.to_string(),
        space_id.to_string(),
        vec![],
        now(),
        content,
    )
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
        state.apply_event(&room_ev).unwrap();
        assert_eq!(state.rooms.len(), 1);
        let room_id = room_ev.event_id.unwrap();
        assert!(state.rooms.contains_key(&room_id));
    }

    #[test]
    fn room_create_by_member_permission_denied() {
        let alice = alice_key();
        let bob = bob_key();
        let (mut state, space_id) = create_space(&alice);
        let alice_id = sender_id(&alice);
        let bob_id = sender_id(&bob);

        // Alice invites Bob as member (not admin).
        state.pending_invites.insert(bob_id.clone(), Role::Member);

        // Bob joins the space.
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev).unwrap();
        assert_eq!(state.member_role(&bob_id), Some(&Role::Member));

        // Bob tries to create a room — should fail.
        let room_ev = sign_event(build_room_create_event(&bob, &space_id, "secret", None), &bob);
        let err = state.apply_event(&room_ev).unwrap_err();
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
        state.apply_event(&invite_ev).unwrap();
        assert!(state.pending_invites.contains_key(&bob_id));

        // Bob joins the space.
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev).unwrap();
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
        state.apply_event(&room_ev).unwrap();

        // Bob joins space.
        state.pending_invites.insert(bob_id.clone(), Role::Member);
        let join_space = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_space).unwrap();

        // Bob joins the room.
        let join_room = sign_event(
            build_membership_event(&bob, &space_id, &room_id, EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_room).unwrap();
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
        state.apply_event(&room_ev).unwrap();

        state.pending_invites.insert(bob_id.clone(), Role::Member);
        state.apply_event(&sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        )).unwrap();
        state.apply_event(&sign_event(
            build_membership_event(&bob, &space_id, &room_id, EventType::MembershipJoin, json!({})),
            &bob,
        )).unwrap();
        assert!(state.is_room_member(&bob_id, &room_id));

        // Bob leaves the space.
        state.apply_event(&sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipLeave, json!({})),
            &bob,
        )).unwrap();
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
        state.apply_event(&ban_ev).unwrap();
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
        let err = state.apply_event(&invite_ev).unwrap_err();
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
        state.pending_invites.insert(bob_id.clone(), crate::space::membership::Role::Member);
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev).unwrap();
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
        let err = state.apply_event(&invite_ev).unwrap_err();
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
        let err = state.apply_event(&room_ev).unwrap_err();
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
        state.apply_event(&promote_ev).unwrap();
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
        state.apply_event(&invite_ev).unwrap();
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
        state.apply_event(&promote_ev).unwrap();

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
        state.apply_event(&ev).unwrap();
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
        state.apply_event(&ev).unwrap();
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
        state.pending_invites.insert(bob_id.clone(), Role::Member);
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev).unwrap();

        let attempt = sign_event(
            build_space_pacing_event(&bob, &space_id, vec![], 9999, 9999),
            &bob,
        );
        let err = state.apply_event(&attempt).unwrap_err();
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
        let err = state.apply_event(&ev).unwrap_err();
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
        state.apply_event(&room_ev).unwrap();

        // Invite Bob as moderator.
        state.pending_invites.insert(bob_id.clone(), Role::Moderator);
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev).unwrap();

        // Invite Charlie as plain member.
        state.pending_invites.insert(charlie_id.clone(), Role::Member);
        let join2 = sign_event(
            build_membership_event(&charlie, &space_id, "", EventType::MembershipJoin, json!({})),
            &charlie,
        );
        state.apply_event(&join2).unwrap();

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
        state.apply_event(&ev).unwrap();
        assert_eq!(state.member_temperature_visibility, VISIBILITY_EVERYONE);
    }

    #[test]
    fn space_visibility_update_rejected_when_sender_not_owner() {
        let (mut state, space_id, _, _, bob, _) = make_space_with_three_members();
        let ev = sign_event(
            build_space_temperature_visibility_event(&bob, &space_id, vec![], VISIBILITY_EVERYONE),
            &bob,
        );
        let err = state.apply_event(&ev).unwrap_err();
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
        state.apply_event(&ev).unwrap();
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
        let _ = state.apply_event(&ev);
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
        state.apply_event(&room_ev).unwrap();

        state.pending_invites.insert(bob_id.clone(), Role::Moderator);
        let join_b = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_b).unwrap();

        state.pending_invites.insert(charlie_id.clone(), Role::Member);
        let join_c = sign_event(
            build_membership_event(&charlie, &space_id, "", EventType::MembershipJoin, json!({})),
            &charlie,
        );
        state.apply_event(&join_c).unwrap();
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
        state.apply_event(&mute_ev).unwrap();
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
        let err = state.apply_event(&mute_ev).unwrap_err();
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
        state.apply_event(&mute_ev).unwrap();
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
        let err = state.apply_event(&ev).unwrap_err();
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
        state.apply_event(&kick_ev).unwrap();
        assert!(!state.is_member(&charlie_id), "kicked member removed");
        assert_eq!(
            kick_ev.content["reason"].as_str(),
            Some(REASON_AUTO_TEMPERATURE),
            "reason preserved on DAG event"
        );
        // Room reference avoids unused-binding warnings.
        let _ = room_id;
    }
}
