// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Conflict detection — identifies competing Events for the same state key
// (spec 3.9.1).
//
// Two Events conflict when:
//   1. They share the same StateKey (same logical piece of mutable state).
//   2. Neither is a causal ancestor of the other (no causal ordering in the DAG).
//
// Message Events never conflict — concurrent messages are both displayed.

use std::collections::HashMap;

use crate::wire::types::Event;

use super::state_key::{state_key_for_event, StateKey};

/// Group a slice of Events by state key. Returns only groups containing 2 or more
/// Events — these are the conflict sets that require resolution.
///
/// Events with no state key (message events, etc.) are silently excluded.
///
/// # Arguments
/// * `events` — candidate event slice (all must share the same Space).
pub fn find_conflicts(events: &[Event]) -> Vec<(StateKey, Vec<&Event>)> {
    let mut by_key: HashMap<StateKey, Vec<&Event>> = HashMap::new();

    for event in events {
        if let Some(key) = state_key_for_event(event) {
            by_key.entry(key).or_default().push(event);
        }
    }

    by_key.into_iter().filter(|(_, group)| group.len() >= 2).collect()
}

/// Check whether a specific Event conflicts with any Event in a reference set.
///
/// Returns the conflicting Events from `existing` that share the state key of
/// `incoming` and have no causal ordering with it (i.e. `incoming` is not in
/// their prev_events chain and they are not in `incoming`'s prev_events chain).
///
/// For Phase 2, causal ordering check is simplified: `incoming` conflicts with
/// `existing_event` if neither appears in the other's `prev_events`. The full
/// ancestor check (transitive closure) is left to Layer 12+ integration.
pub fn conflicts_with<'a>(incoming: &Event, existing: &'a [Event]) -> Vec<&'a Event> {
    let incoming_key = match state_key_for_event(incoming) {
        Some(k) => k,
        None => return vec![],
    };

    let incoming_id = match incoming.event_id.as_ref().map(|e| e.as_str()) {
        Some(id) => id,
        None => return vec![],
    };

    existing.iter().filter(|ev| {
        // Same state key?
        state_key_for_event(ev).as_ref() == Some(&incoming_key)
            // Not the same event.
            && ev.event_id.as_ref().map(|e| e.as_str()) != Some(incoming_id)
            // No direct causal ordering: incoming does not reference ev.
            && !incoming.prev_events.iter().any(|p| Some(p.as_str()) == ev.event_id.as_ref().map(|e| e.as_str()))
            // And ev does not reference incoming.
            && !ev.prev_events.iter().any(|p| p.as_str() == incoming_id)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::types::EventType;
    use serde_json::json;

    use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};

    fn ev(event_type: EventType, sender: &str, space: &str, prev: Vec<&str>, content: serde_json::Value) -> Event {
        Event::new(
            event_type,
            IdentityXgid::from_xgid(Xgid::new(sender.to_string())),
            RoomXgid::from_xgid(Xgid::new("".to_string())),
            SpaceXgid::from_xgid(Xgid::new(space.to_string())),
            prev.into_iter()
                .map(|s| EventXgid::from_xgid(Xgid::new(s.to_string())))
                .collect(),
            "2026-01-01T00:00:00.000Z".to_string(),
            content,
        )
    }

    fn with_id(mut e: Event, id: &str) -> Event {
        e.event_id = Some(EventXgid::from_xgid(Xgid::new(id.to_string())));
        e
    }

    #[test]
    fn no_conflicts_for_message_events() {
        let events = vec![
            with_id(ev(EventType::MessageText, "alice", "s", vec!["prev"], json!({"text":"hi"})), "msg1"),
            with_id(ev(EventType::MessageText, "bob", "s", vec!["prev"], json!({"text":"ho"})), "msg2"),
        ];
        assert!(find_conflicts(&events).is_empty());
    }

    #[test]
    fn same_state_key_produces_conflict_group() {
        let e1 = with_id(ev(EventType::MembershipJoin, "alice", "s", vec!["prev1"], json!({})), "ev1");
        let e2 = with_id(ev(EventType::MembershipJoin, "alice", "s", vec!["prev2"], json!({})), "ev2");
        let binding = [e1, e2];
        let groups = find_conflicts(&binding);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].1.len(), 2);
    }

    #[test]
    fn different_state_keys_produce_no_cross_conflict() {
        let e1 = with_id(ev(EventType::MembershipJoin, "alice", "s", vec!["p"], json!({})), "ev1");
        // Bob joining — different identity, different state key
        let e2 = with_id(ev(EventType::MembershipJoin, "bob", "s", vec!["p"], json!({})), "ev2");
        // Both have state keys but different key_fields → separate groups, each size 1
        let binding = [e1, e2];
        let groups = find_conflicts(&binding);
        assert!(groups.is_empty());
    }

    #[test]
    fn conflicts_with_returns_concurrent_events() {
        let existing1 = with_id(ev(EventType::MembershipBan, "admin", "s", vec!["p"], json!({"target_identity":"alice"})), "ban1");
        let existing = vec![existing1];
        let incoming = with_id(ev(EventType::MembershipJoin, "alice", "s", vec!["p"], json!({})), "join1");
        let conflicts = conflicts_with(&incoming, &existing);
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn conflicts_with_excludes_causally_ordered_events() {
        // existing references incoming — incoming is ancestor of existing, so no conflict
        let existing = with_id(ev(EventType::MembershipJoin, "alice", "s", vec!["join1"], json!({})), "join2");
        let incoming = with_id(ev(EventType::MembershipJoin, "alice", "s", vec!["root"], json!({})), "join1");
        let binding = [existing];
        let conflicts = conflicts_with(&incoming, &binding);
        // existing has incoming.event_id ("join1") in its prev_events → causal order → no conflict
        assert!(conflicts.is_empty());
    }
}
