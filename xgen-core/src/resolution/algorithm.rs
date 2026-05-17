// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// State resolution algorithm — seven-layer priority stack (spec 3.9.3).
//
// Every Node independently applies the same algorithm to the same Event log and
// reaches the same winner. This is the convergence guarantee (3.9.2):
// conflict resolution is a pure function of the Event content.

use std::collections::{HashMap, HashSet};

use crate::{
    space::{membership::Role, state::SpaceState},
    wire::types::{Event, EventType},
};

use super::ResolutionError;

/// Resolve a set of conflicting Events — Events that share the same state key
/// and have no causal ordering with each other.
///
/// Returns a reference to the winning Event. The losing Events are NOT removed
/// from the DAG — the caller only updates the state snapshot with the winner.
///
/// # Arguments
/// * `conflicts` — two or more Events that compete for the same state key.
///   Must be non-empty.
/// * `space_state` — current SpaceState snapshot of the containing Space.
/// * `identity_home_nodes` — map of identity_id → home_node_id, used for
///   Layers 3, 5a, and 5b. The caller provides this from the identity registry.
///
/// # Errors
/// Returns `ResolutionError::StateConflictUnresolvable` only if `conflicts` is empty.
/// Layer 5c (lexicographic backstop) guarantees a winner for any non-empty set.
pub fn resolve<'a>(
    conflicts: &'a [Event],
    space_state: &SpaceState,
    identity_home_nodes: &HashMap<String, String>,
) -> Result<&'a Event, ResolutionError> {
    if conflicts.is_empty() {
        return Err(ResolutionError::StateConflictUnresolvable);
    }
    if conflicts.len() == 1 {
        return Ok(&conflicts[0]);
    }

    // Layer 1 — EventType hardcoded priority table (spec 3.9.3 Layer 1).
    if let Some(winner) = layer1_event_type_priority(conflicts) {
        return Ok(winner);
    }

    // Layer 2 — Auth Tier of producing Node (spec 3.9.3 Layer 2).
    // All events in the same Space share the same auth_tier — always tied.
    // This layer is future-proofing for cross-tier Space scenarios.
    let _ = space_state.auth_tier; // acknowledged; always tied in Phase 2 Tier 1

    // Layer 3 — Home Node assertion for Identity's own state (spec 3.9.3 Layer 3).
    if let Some(winner) = layer3_home_node_assertion(conflicts, space_state, identity_home_nodes) {
        return Ok(winner);
    }

    // Layer 4 — Role within Space (spec 3.9.3 Layer 4).
    if let Some(winner) = layer4_role_priority(conflicts, space_state) {
        return Ok(winner);
    }

    // Layer 5a — Manual Node ordering via state.node_priority (spec 3.9.3 Layer 5a).
    if let Some(winner) = layer5a_node_priority(conflicts, space_state, identity_home_nodes) {
        return Ok(winner);
    }

    // Layer 5b — Federation recency (spec 3.9.3 Layer 5b).
    if let Some(winner) = layer5b_federation_recency(conflicts, space_state, identity_home_nodes) {
        return Ok(winner);
    }

    // Layer 5c — Lexicographic event_id backstop (spec 3.9.3 Layer 5c).
    // Always produces a unique winner — event_ids are content hashes and therefore unique.
    layer5c_lexicographic_backstop(conflicts)
}

// ── Layer 1 — EventType hardcoded priority table ──────────────────────────────

fn is_membership_event(t: &EventType) -> bool {
    matches!(
        t,
        EventType::MembershipBan
            | EventType::MembershipKick
            | EventType::MembershipLeave
            | EventType::MembershipJoin
            | EventType::MembershipInvite
    )
}

/// Apply the membership EventType priority table (spec 3.9.3 Layer 1):
///   ban  > join, invite, kick
///   kick > join, invite
///   leave > join
///
/// Returns Some(winner) when one Event's type strictly dominates all others
/// AND exactly one Event has that winning type. Otherwise returns None.
fn layer1_event_type_priority<'a>(conflicts: &'a [Event]) -> Option<&'a Event> {
    if !conflicts.iter().all(|e| is_membership_event(&e.event_type)) {
        return None;
    }

    let types: HashSet<&EventType> = conflicts.iter().map(|e| &e.event_type).collect();
    let has = |t: &EventType| types.contains(t);

    let winner_type = if has(&EventType::MembershipBan) {
        // ban beats join, invite, kick — only if at least one of those is present
        if has(&EventType::MembershipJoin)
            || has(&EventType::MembershipInvite)
            || has(&EventType::MembershipKick)
        {
            Some(EventType::MembershipBan)
        } else {
            None // all are bans — layer 1 doesn't distinguish
        }
    } else if has(&EventType::MembershipKick) {
        if has(&EventType::MembershipJoin) || has(&EventType::MembershipInvite) {
            Some(EventType::MembershipKick)
        } else {
            None
        }
    } else if has(&EventType::MembershipLeave) && has(&EventType::MembershipJoin) {
        Some(EventType::MembershipLeave)
    } else {
        None
    }?;

    let winners: Vec<&Event> =
        conflicts.iter().filter(|e| e.event_type == winner_type).collect();
    if winners.len() == 1 { Some(winners[0]) } else { None }
}

// ── Layer 3 — Home Node assertion ────────────────────────────────────────────

/// For conflicts over an Identity's own state (membership.join/leave, key_rotation),
/// the event whose producer's home Node matches the affected Identity's home Node wins.
///
/// "Producer" of an event = the home Node of the event sender (identity_home_nodes lookup).
/// "Affected identity" = the sender for join/leave, or content["target_identity"] for
/// invite/kick/ban.
///
/// Only applies to membership and key-rotation events — Layer 3 is specifically about
/// an Identity's own state, not general shared state like state.room_update.
///
/// If all conflicting events are from the same home Node, or the home Node cannot be
/// determined, returns None (fall through to Layer 4).
fn layer3_home_node_assertion<'a>(
    conflicts: &'a [Event],
    _space_state: &SpaceState,
    identity_home_nodes: &HashMap<String, String>,
) -> Option<&'a Event> {
    // Layer 3 applies only to membership events and system.key_rotation.
    let first = conflicts.first()?;
    let applies = is_membership_event(&first.event_type)
        || matches!(first.event_type, EventType::SystemKeyRotation);
    if !applies {
        return None;
    }

    // Determine the affected identity (same for all events in the conflict set,
    // since they share a state key).
    let affected_identity = affected_identity_for(first);
    let home_node = identity_home_nodes.get(affected_identity)?;

    // Find events produced by the affected identity's home Node.
    // "Produced by" = the sender's home_node matches.
    let from_home_node: Vec<&Event> = conflicts
        .iter()
        .filter(|e| identity_home_nodes.get(&e.sender).as_ref() == Some(&home_node))
        .collect();

    if from_home_node.len() == 1 { Some(from_home_node[0]) } else { None }
}

fn affected_identity_for(event: &Event) -> &str {
    match &event.event_type {
        EventType::MembershipJoin | EventType::MembershipLeave | EventType::SystemKeyRotation => {
            &event.sender
        }
        _ => event.content["target_identity"].as_str().unwrap_or(&event.sender),
    }
}

// ── Layer 4 — Role within Space ───────────────────────────────────────────────

/// The sender with the higher Space role wins.
/// Roles in descending priority: Owner > Admin > Moderator > Member.
///
/// Returns Some(winner) if exactly one event has the highest sender role.
fn layer4_role_priority<'a>(
    conflicts: &'a [Event],
    space_state: &SpaceState,
) -> Option<&'a Event> {
    let role_of = |e: &Event| -> Option<&Role> { space_state.member_role(&e.sender) };

    let max_role = conflicts.iter().filter_map(|e| role_of(e)).max()?;

    let winners: Vec<&Event> = conflicts
        .iter()
        .filter(|e| role_of(e) == Some(max_role))
        .collect();

    if winners.len() == 1 { Some(winners[0]) } else { None }
}

// ── Layer 5a — Manual Node ordering ──────────────────────────────────────────

/// Consult the most recent state.node_priority Event for manual Node ordering.
/// The node listed at the lowest index (index 0 = highest priority) wins.
///
/// The producing node of an event = identity_home_nodes[event.sender].
/// Uses space_state.node_priority_order (populated by state.node_priority events).
fn layer5a_node_priority<'a>(
    conflicts: &'a [Event],
    space_state: &SpaceState,
    identity_home_nodes: &HashMap<String, String>,
) -> Option<&'a Event> {
    if space_state.node_priority_order.is_empty() {
        return None;
    }

    let event_priority = |e: &Event| -> Option<usize> {
        let node = identity_home_nodes.get(&e.sender)?;
        space_state.node_priority_order.iter().position(|n| n == node)
    };

    // Lower index = higher priority.
    let scored: Vec<(usize, &Event)> =
        conflicts.iter().filter_map(|e| event_priority(e).map(|p| (p, e))).collect();

    if scored.is_empty() {
        return None;
    }

    let min_idx = scored.iter().map(|(p, _)| *p).min()?;
    let winners: Vec<&&Event> =
        scored.iter().filter(|(p, _)| *p == min_idx).map(|(_, e)| e).collect();

    if winners.len() == 1 { Some(winners[0]) } else { None }
}

// ── Layer 5b — Federation recency ────────────────────────────────────────────

/// Most recently joined Node wins. Home Node is treated as joined at Space creation
/// (earliest). Nodes appear in space_state.federation_nodes ordered by join time —
/// later index = more recently joined.
///
/// Combined ordering: [home_node, federation_nodes[0], ..., federation_nodes[last]]
/// Higher index = more recently joined = higher priority in this layer.
fn layer5b_federation_recency<'a>(
    conflicts: &'a [Event],
    space_state: &SpaceState,
    identity_home_nodes: &HashMap<String, String>,
) -> Option<&'a Event> {
    let mut ordered = vec![space_state.home_node.clone()];
    ordered.extend(space_state.federation_nodes.iter().cloned());

    let event_recency = |e: &Event| -> Option<usize> {
        let node = identity_home_nodes.get(&e.sender)?;
        ordered.iter().position(|n| n == node)
    };

    let scored: Vec<(usize, &Event)> =
        conflicts.iter().filter_map(|e| event_recency(e).map(|r| (r, e))).collect();

    if scored.is_empty() {
        return None;
    }

    // Higher index = more recently joined = wins.
    let max_idx = scored.iter().map(|(r, _)| *r).max()?;
    let winners: Vec<&&Event> =
        scored.iter().filter(|(r, _)| *r == max_idx).map(|(_, e)| e).collect();

    if winners.len() == 1 { Some(winners[0]) } else { None }
}

// ── Layer 5c — Lexicographic event_id backstop ────────────────────────────────

/// The Event whose event_id sorts lower in lexicographic (Unicode code point)
/// order wins. event_ids are SHA-256 content hashes — ungameable and globally
/// unique — so this layer always produces a unique winner (spec 3.9.3 Layer 5c).
fn layer5c_lexicographic_backstop<'a>(
    conflicts: &'a [Event],
) -> Result<&'a Event, ResolutionError> {
    conflicts
        .iter()
        .filter(|e| e.event_id.is_some())
        .min_by(|a, b| {
            a.event_id.as_deref().unwrap().cmp(b.event_id.as_deref().unwrap())
        })
        .ok_or(ResolutionError::ResolutionStackExhausted)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dag::{graph::DagGraph, store::EventStore},
        space::{
            membership::Role,
            state::{build_space_create_event, sign_event, SpaceMember, SpaceState},
        },
        wire::types::EventType,
        identity::keypair,
        crypto::encoding,
    };
    use ed25519_dalek::SigningKey;
    use serde_json::json;

    // ── Test helpers ──────────────────────────────────────────────────────────

    fn gen_key() -> SigningKey {
        keypair::generate()
    }

    fn identity_id(key: &SigningKey) -> String {
        format!("xgen://pubkey/ed25519:{}", encoding::encode(key.verifying_key().as_bytes()))
    }

    fn make_event_with_id(
        event_type: EventType,
        sender: &str,
        space_id: &str,
        content: serde_json::Value,
        event_id: &str,
    ) -> Event {
        let mut ev = Event::new(
            event_type,
            sender.to_string(),
            "".to_string(),
            space_id.to_string(),
            vec!["prev".to_string()],
            "2026-01-01T00:00:00.000Z".to_string(),
            content,
        );
        ev.event_id = Some(event_id.to_string());
        ev
    }

    fn membership_event(
        event_type: EventType,
        sender: &str,
        target: &str,
        space_id: &str,
        event_id: &str,
    ) -> Event {
        let content = if target == sender {
            json!({})
        } else {
            json!({"target_identity": target})
        };
        make_event_with_id(event_type, sender, space_id, content, event_id)
    }

    fn simple_space_state(owner_id: &str, owner_role: Role, home_node: &str) -> SpaceState {
        let mut members = std::collections::HashMap::new();
        members.insert(owner_id.to_string(), SpaceMember {
            identity_id: owner_id.to_string(),
            role: owner_role,
            joined_at: "2026-01-01T00:00:00.000Z".to_string(),
            invited_by: None,
        });
        SpaceState {
            space_id: "space1".to_string(),
            name: Some("Test".to_string()),
            topic: None,
            auth_tier: 1,
            max_event_size: None,
            home_node: home_node.to_string(),
            owner_id: owner_id.to_string(),
            is_dm: false,
            members,
            pending_invites: Default::default(),
            ai_operator_delegations: Default::default(),
            banned: Default::default(),
            rooms: Default::default(),
            federation_nodes: vec![],
            node_priority_order: vec![],
            dm_constraints_active: false,
            human_pacing_ms: crate::wire::types::DEFAULT_HUMAN_PACING_MS,
            ai_pacing_ms: crate::wire::types::DEFAULT_AI_PACING_MS,
            member_temperature_visibility:
                crate::wire::types::DEFAULT_MEMBER_TEMPERATURE_VISIBILITY.to_string(),
            active_mutes: Default::default(),
        }
    }

    fn empty_home_nodes() -> HashMap<String, String> {
        HashMap::new()
    }

    // ── Layer 1 tests ─────────────────────────────────────────────────────────

    #[test]
    fn ban_beats_concurrent_join() {
        let identity = "xgen://pubkey/ed25519:TARGET";
        let admin = "xgen://pubkey/ed25519:ADMIN";
        let space = "space1";

        let join_ev = membership_event(EventType::MembershipJoin, identity, identity, space, "ev_join");
        let ban_ev = membership_event(EventType::MembershipBan, admin, identity, space, "ev_ban");

        let space_state = simple_space_state(admin, Role::Admin, "node1");
        let conflicts = [join_ev, ban_ev];
        let winner = resolve(&conflicts, &space_state, &empty_home_nodes()).unwrap();
        assert_eq!(winner.event_type, EventType::MembershipBan);
    }

    #[test]
    fn kick_beats_invite() {
        let target = "xgen://pubkey/ed25519:TARGET";
        let mod_id = "xgen://pubkey/ed25519:MOD";
        let space = "space1";

        let invite_ev = membership_event(EventType::MembershipInvite, mod_id, target, space, "ev_invite");
        let kick_ev = membership_event(EventType::MembershipKick, mod_id, target, space, "ev_kick");

        let space_state = simple_space_state(mod_id, Role::Moderator, "node1");
        let conflicts = [invite_ev, kick_ev];
        let winner = resolve(&conflicts, &space_state, &empty_home_nodes()).unwrap();
        assert_eq!(winner.event_type, EventType::MembershipKick);
    }

    #[test]
    fn ban_beats_kick() {
        let target = "xgen://pubkey/ed25519:TARGET";
        let admin = "xgen://pubkey/ed25519:ADMIN";
        let space = "space1";

        let kick_ev = membership_event(EventType::MembershipKick, admin, target, space, "ev_kick");
        let ban_ev = membership_event(EventType::MembershipBan, admin, target, space, "ev_ban");

        let space_state = simple_space_state(admin, Role::Admin, "node1");
        let conflicts = [kick_ev, ban_ev];
        let winner = resolve(&conflicts, &space_state, &empty_home_nodes()).unwrap();
        assert_eq!(winner.event_type, EventType::MembershipBan);
    }

    // ── Layer 4 tests ─────────────────────────────────────────────────────────

    #[test]
    fn same_type_falls_through_to_layer4() {
        // Two concurrent state.room_update events — same type, Layer 1 inapplicable.
        let owner_id = "xgen://pubkey/ed25519:OWNER";
        let admin_id = "xgen://pubkey/ed25519:ADMIN";
        let space = "space1";

        let ev_admin = make_event_with_id(EventType::StateRoomUpdate, admin_id, space, json!({}), "ev_admin");
        let ev_owner = make_event_with_id(EventType::StateRoomUpdate, owner_id, space, json!({}), "ev_owner");

        let mut space_state = simple_space_state(owner_id, Role::Owner, "node1");
        space_state.members.insert(admin_id.to_string(), SpaceMember {
            identity_id: admin_id.to_string(),
            role: Role::Admin,
            joined_at: "2026-01-01T00:00:00.000Z".to_string(),
            invited_by: None,
        });

        let conflicts = [ev_admin, ev_owner];
        let winner = resolve(&conflicts, &space_state, &empty_home_nodes()).unwrap();
        // Owner beats Admin in Layer 4.
        assert_eq!(winner.sender, owner_id);
    }

    #[test]
    fn higher_role_wins_same_type_conflict() {
        let admin_id = "xgen://pubkey/ed25519:ADMIN";
        let mod_id = "xgen://pubkey/ed25519:MOD";
        let space = "space1";

        let ev_mod = make_event_with_id(EventType::StateRoomUpdate, mod_id, space, json!({}), "ev_mod");
        let ev_admin = make_event_with_id(EventType::StateRoomUpdate, admin_id, space, json!({}), "ev_admin");

        let owner_id = "xgen://pubkey/ed25519:OWNER";
        let mut space_state = simple_space_state(owner_id, Role::Owner, "node1");
        space_state.members.insert(admin_id.to_string(), SpaceMember {
            identity_id: admin_id.to_string(),
            role: Role::Admin,
            joined_at: "2026-01-01T00:00:00.000Z".to_string(),
            invited_by: None,
        });
        space_state.members.insert(mod_id.to_string(), SpaceMember {
            identity_id: mod_id.to_string(),
            role: Role::Moderator,
            joined_at: "2026-01-01T00:00:00.000Z".to_string(),
            invited_by: None,
        });

        let conflicts = [ev_mod, ev_admin];
        let winner = resolve(&conflicts, &space_state, &empty_home_nodes()).unwrap();
        assert_eq!(winner.sender, admin_id);
    }

    #[test]
    fn owner_beats_admin() {
        let owner_id = "xgen://pubkey/ed25519:OWNER";
        let admin_id = "xgen://pubkey/ed25519:ADMIN";
        let space = "space1";

        let ev_admin = make_event_with_id(EventType::StateSpaceUpdate, admin_id, space, json!({}), "ev_admin");
        let ev_owner = make_event_with_id(EventType::StateSpaceUpdate, owner_id, space, json!({}), "ev_owner");

        let mut space_state = simple_space_state(owner_id, Role::Owner, "node1");
        space_state.members.insert(admin_id.to_string(), SpaceMember {
            identity_id: admin_id.to_string(),
            role: Role::Admin,
            joined_at: "2026-01-01T00:00:00.000Z".to_string(),
            invited_by: None,
        });

        let conflicts = [ev_admin, ev_owner];
        let winner = resolve(&conflicts, &space_state, &empty_home_nodes()).unwrap();
        assert_eq!(winner.sender, owner_id);
    }

    // ── Layer 5a test ─────────────────────────────────────────────────────────

    #[test]
    fn node_priority_respected() {
        let node_a = "xgen://pubkey/ed25519:NODE_A";
        let node_b = "xgen://pubkey/ed25519:NODE_B";

        let alice_id = "xgen://pubkey/ed25519:ALICE";
        let bob_id = "xgen://pubkey/ed25519:BOB";
        let owner_id = "xgen://pubkey/ed25519:OWNER";
        let space = "space1";

        let ev_bob = make_event_with_id(EventType::StateRoomUpdate, bob_id, space, json!({}), "ev_bob");
        let ev_alice = make_event_with_id(EventType::StateRoomUpdate, alice_id, space, json!({}), "ev_alice");

        // Both are Admins — Layer 4 tied.
        let mut space_state = simple_space_state(owner_id, Role::Owner, node_a);
        space_state.members.insert(alice_id.to_string(), SpaceMember {
            identity_id: alice_id.to_string(),
            role: Role::Admin,
            joined_at: "2026-01-01T00:00:00.000Z".to_string(),
            invited_by: None,
        });
        space_state.members.insert(bob_id.to_string(), SpaceMember {
            identity_id: bob_id.to_string(),
            role: Role::Admin,
            joined_at: "2026-01-01T00:00:00.000Z".to_string(),
            invited_by: None,
        });

        // node_a has higher priority (index 0) than node_b (index 1).
        space_state.node_priority_order = vec![node_a.to_string(), node_b.to_string()];

        let mut home_nodes = HashMap::new();
        home_nodes.insert(alice_id.to_string(), node_a.to_string()); // Alice is on node_a
        home_nodes.insert(bob_id.to_string(), node_b.to_string());   // Bob is on node_b

        let conflicts = [ev_bob, ev_alice];
        let winner = resolve(&conflicts, &space_state, &home_nodes).unwrap();
        // Alice's event wins — she's on node_a which has higher priority.
        assert_eq!(winner.sender, alice_id);
    }

    // ── Layer 5c test ─────────────────────────────────────────────────────────

    #[test]
    fn lexicographic_backstop_always_resolves() {
        let alice_id = "xgen://pubkey/ed25519:ALICE";
        let bob_id = "xgen://pubkey/ed25519:BOB";
        let owner_id = "xgen://pubkey/ed25519:OWNER";
        let space = "space1";

        // Use real SHA-256 hash URIs to ensure lexicographic ordering is unambiguous.
        // xgen://hash/sha256:aaa... sorts lower than xgen://hash/sha256:bbb...
        let id_aaa = "xgen://hash/sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1";
        let id_bbb = "xgen://hash/sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb1";

        let ev1 = make_event_with_id(EventType::StateRoomUpdate, alice_id, space, json!({}), id_bbb);
        let ev2 = make_event_with_id(EventType::StateRoomUpdate, bob_id, space, json!({}), id_aaa);

        // Both are Members — no role distinction. No node priority set.
        let mut space_state = simple_space_state(owner_id, Role::Owner, "node1");
        space_state.members.insert(alice_id.to_string(), SpaceMember {
            identity_id: alice_id.to_string(),
            role: Role::Member,
            joined_at: "2026-01-01T00:00:00.000Z".to_string(),
            invited_by: None,
        });
        space_state.members.insert(bob_id.to_string(), SpaceMember {
            identity_id: bob_id.to_string(),
            role: Role::Member,
            joined_at: "2026-01-01T00:00:00.000Z".to_string(),
            invited_by: None,
        });

        let conflicts = [ev1, ev2];
        let winner = resolve(&conflicts, &space_state, &empty_home_nodes()).unwrap();
        // Lower event_id wins — id_aaa < id_bbb, so ev2 (bob's) wins.
        assert_eq!(winner.event_id.as_deref(), Some(id_aaa));
    }

    // ── Loser stays in DAG test ───────────────────────────────────────────────

    #[test]
    fn loser_event_stays_in_dag() {
        let alice_key = gen_key();
        let alice_id = identity_id(&alice_key);
        let bob_key = gen_key();
        let bob_id = identity_id(&bob_key);

        // Build a minimal Space with alice as owner.
        let space_ev = sign_event(build_space_create_event(&alice_key, "Test", None, 1, "node1"), &alice_key);
        let space_id = space_ev.event_id.clone().unwrap();

        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        graph.add_event(&space_ev, &store).unwrap();
        store.insert(space_ev.clone()).unwrap();

        // Create two conflicting state.room_update events with the same prev_event
        // (both reference the space_create root → concurrent, no causal ordering).
        let mut ev_alice = Event::new(
            EventType::StateRoomUpdate,
            alice_id.clone(),
            "room1".to_string(),
            space_id.clone(),
            vec![space_id.clone()],
            "2026-01-01T00:00:01.000Z".to_string(),
            json!({"name": "alice's name"}),
        );
        ev_alice = sign_event(ev_alice, &alice_key);
        let alice_ev_id = ev_alice.event_id.clone().unwrap();

        let mut ev_bob = Event::new(
            EventType::StateRoomUpdate,
            bob_id.clone(),
            "room1".to_string(),
            space_id.clone(),
            vec![space_id.clone()],
            "2026-01-01T00:00:02.000Z".to_string(),
            json!({"name": "bob's name"}),
        );
        ev_bob = sign_event(ev_bob, &bob_key);
        let bob_ev_id = ev_bob.event_id.clone().unwrap();

        // Store both events in DAG (they both pass validation independently).
        graph.add_event(&ev_alice, &store).unwrap();
        store.insert(ev_alice.clone()).unwrap();
        graph.add_event(&ev_bob, &store).unwrap();
        store.insert(ev_bob.clone()).unwrap();

        // Both events are in the DAG.
        assert!(store.contains(&alice_ev_id));
        assert!(store.contains(&bob_ev_id));

        // Run resolution.
        let mut space_state = simple_space_state(&alice_id, Role::Owner, "node1");
        space_state.members.insert(bob_id.clone(), SpaceMember {
            identity_id: bob_id.clone(),
            role: Role::Member,
            joined_at: "2026-01-01T00:00:00.000Z".to_string(),
            invited_by: None,
        });

        let conflicts = [ev_alice.clone(), ev_bob.clone()];
        let winner = resolve(&conflicts, &space_state, &empty_home_nodes()).unwrap();
        let loser_id = if winner.event_id.as_deref() == Some(&alice_ev_id) {
            &bob_ev_id
        } else {
            &alice_ev_id
        };

        // Loser is still in the DAG — it was never deleted.
        assert!(store.contains(loser_id), "loser event must remain in the DAG");
    }

    // ── Determinism test ──────────────────────────────────────────────────────

    #[test]
    fn resolution_is_deterministic() {
        let alice_id = "xgen://pubkey/ed25519:ALICE";
        let bob_id = "xgen://pubkey/ed25519:BOB";
        let owner_id = "xgen://pubkey/ed25519:OWNER";
        let space = "space1";

        let ev_a = make_event_with_id(
            EventType::StateRoomUpdate,
            alice_id,
            space,
            json!({}),
            "xgen://hash/sha256:aaaa0000000000000000000000000000000000000000000000000000000000a",
        );
        let ev_b = make_event_with_id(
            EventType::StateRoomUpdate,
            bob_id,
            space,
            json!({}),
            "xgen://hash/sha256:bbbb0000000000000000000000000000000000000000000000000000000000b",
        );

        let mut space_state = simple_space_state(owner_id, Role::Owner, "node1");
        space_state.members.insert(alice_id.to_string(), SpaceMember {
            identity_id: alice_id.to_string(),
            role: Role::Member,
            joined_at: "2026-01-01T00:00:00.000Z".to_string(),
            invited_by: None,
        });
        space_state.members.insert(bob_id.to_string(), SpaceMember {
            identity_id: bob_id.to_string(),
            role: Role::Member,
            joined_at: "2026-01-01T00:00:00.000Z".to_string(),
            invited_by: None,
        });

        // Order 1: A then B
        let order1 = [ev_a.clone(), ev_b.clone()];
        let winner1 = resolve(&order1, &space_state, &empty_home_nodes()).unwrap();
        // Order 2: B then A
        let order2 = [ev_b.clone(), ev_a.clone()];
        let winner2 = resolve(&order2, &space_state, &empty_home_nodes()).unwrap();

        assert_eq!(
            winner1.event_id, winner2.event_id,
            "resolution must be deterministic regardless of input order"
        );
    }
}
