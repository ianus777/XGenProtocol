// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Space and Room protocol — state derivation and membership (spec 3.7).

pub mod dm_promotion;
pub mod membership;
pub mod node_policy;
pub mod state;

#[cfg(test)]
mod tests {
    use super::state::{
        build_membership_event, build_room_create_event, build_space_create_event,
        sign_event, SpaceState,
    };
    use crate::identity::keypair;
    use serde_json::json;
    use crate::wire::types::EventType;

    const HOME: &str = "xgen://pubkey/ed25519:NODE";

    /// Full Space+Room lifecycle:
    ///   Alice creates Space → creates Room → invites Bob → Bob joins Space →
    ///   Bob joins Room → verify both members appear in state.
    #[test]
    fn full_space_room_lifecycle() {
        let alice_key = keypair::generate();
        let bob_key = keypair::generate();

        let alice_id = format!(
            "xgen://pubkey/ed25519:{}",
            crate::crypto::encoding::encode(alice_key.verifying_key().as_bytes())
        );
        let bob_id = format!(
            "xgen://pubkey/ed25519:{}",
            crate::crypto::encoding::encode(bob_key.verifying_key().as_bytes())
        );

        // Step 1: Alice creates a Space.
        let space_create = sign_event(
            build_space_create_event(&alice_key, "Dev Team", Some("Protocol dev"), 1, HOME, None),
            &alice_key,
        );
        let space_id = space_create
            .event_id
            .as_ref()
            .expect("signed event has event_id")
            .as_str()
            .to_string();
        let mut state = SpaceState::from_space_create(&space_create).unwrap();

        // Step 2: Alice creates a Room.
        let room_create = sign_event(
            build_room_create_event(&alice_key, &space_id, "general", Some("General discussion")),
            &alice_key,
        );
        let room_id = room_create
            .event_id
            .as_ref()
            .expect("signed event has event_id")
            .as_str()
            .to_string();
        state.apply_event(&room_create, "").unwrap();
        assert_eq!(state.rooms.len(), 1);
        assert!(state.rooms.contains_key(room_id.as_str()));

        // Step 3: Alice invites Bob.
        let invite = sign_event(
            build_membership_event(
                &alice_key,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            &alice_key,
        );
        state.apply_event(&invite, "").unwrap();
        assert!(state.pending_invites.contains_key(bob_id.as_str()));

        // Step 4: Bob joins the Space.
        let join_space = sign_event(
            build_membership_event(&bob_key, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob_key,
        );
        state.apply_event(&join_space, "").unwrap();
        assert!(state.is_member(&bob_id));
        assert!(state.is_member(&alice_id));

        // Step 5: Bob joins the Room.
        let join_room = sign_event(
            build_membership_event(&bob_key, &space_id, &room_id, EventType::MembershipJoin, json!({})),
            &bob_key,
        );
        state.apply_event(&join_room, "").unwrap();
        assert!(state.is_room_member(&bob_id, &room_id));
        assert!(state.is_room_member(&alice_id, &room_id));
    }
}
