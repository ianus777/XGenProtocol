// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Node-side MLS group state tracking per Room (spec 3.10.2, 3.10.4–3.10.6).
//
// The Node tracks MLS group state from the Node's perspective only:
//   - Which epoch the Room's MLS group is currently on
//   - Which members are in the group
//   - Which device IDs are associated with each member
//
// The Node does NOT track key material — epochs are opaque counters.
// The Node uses epoch tracking to detect stale Welcome messages.

use std::collections::{HashMap, HashSet};

// ── Group state ───────────────────────────────────────────────────────────────

/// Node-side MLS group state for one Room (spec 3.10.2).
/// The Node does not hold any key material — epoch is an opaque counter.
#[derive(Debug, Clone)]
pub struct MlsGroupState {
    /// Room ID this group is associated with.
    pub room_id: String,
    /// Current MLS epoch (advances on every membership change).
    pub epoch: u64,
    /// Active member identity_ids (not device_ids).
    pub members: HashSet<String>,
    /// Known device_ids per member identity_id.
    pub devices: HashMap<String, Vec<String>>,
}

impl MlsGroupState {
    pub fn new(room_id: &str, creator_id: &str) -> Self {
        let mut members = HashSet::new();
        members.insert(creator_id.to_string());
        Self {
            room_id: room_id.to_string(),
            epoch: 0,
            members,
            devices: HashMap::new(),
        }
    }

    /// Advance the epoch. Called when a Commit is processed (member join or leave).
    pub fn advance_epoch(&mut self) {
        self.epoch += 1;
    }

    /// Add a member to the group tracking. Does NOT advance the epoch.
    /// Epoch must be advanced separately via a Commit.
    pub fn add_member(&mut self, identity_id: &str) {
        self.members.insert(identity_id.to_string());
    }

    /// Remove a member from the group tracking. Does NOT advance the epoch.
    pub fn remove_member(&mut self, identity_id: &str) {
        self.members.remove(identity_id);
        self.devices.remove(identity_id);
    }

    pub fn is_member(&self, identity_id: &str) -> bool {
        self.members.contains(identity_id)
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

// ── Group registry ─────────────────────────────────────────────────────────────

/// Node-wide registry of MLS group states, keyed by room_id.
#[derive(Debug, Default)]
pub struct MlsGroupRegistry {
    groups: HashMap<String, MlsGroupState>,
}

impl MlsGroupRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, state: MlsGroupState) {
        self.groups.insert(state.room_id.clone(), state);
    }

    pub fn get(&self, room_id: &str) -> Option<&MlsGroupState> {
        self.groups.get(room_id)
    }

    pub fn get_mut(&mut self, room_id: &str) -> Option<&mut MlsGroupState> {
        self.groups.get_mut(room_id)
    }

    pub fn contains(&self, room_id: &str) -> bool {
        self.groups.contains_key(room_id)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_advances_on_member_join() {
        let mut group = MlsGroupState::new("room1", "alice");
        assert_eq!(group.epoch, 0);

        // Bob joins — epoch must advance after Commit.
        group.add_member("bob");
        group.advance_epoch();
        assert_eq!(group.epoch, 1);
        assert!(group.is_member("alice"));
        assert!(group.is_member("bob"));
    }

    #[test]
    fn epoch_advances_on_member_remove() {
        let mut group = MlsGroupState::new("room1", "alice");
        group.add_member("bob");
        group.advance_epoch(); // epoch 1 (bob joined)

        group.remove_member("bob");
        group.advance_epoch(); // epoch 2 (bob removed)
        assert_eq!(group.epoch, 2);
        assert!(group.is_member("alice"));
        assert!(!group.is_member("bob"));
    }
}
