// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Event batch transfer utilities (spec 3.12.4–3.12.5).
//
// Provides:
//   - BATCH_SIZE         — implementation-defined batch size (D-050)
//   - batch_events       — split Events into fixed-size batches for transfer
//   - compute_batch_hash — SHA-256 of concatenated event_ids in a batch
//   - identify_tail      — events produced after the migration.accept snapshot

use std::collections::HashSet;

use crate::{crypto::hashing, wire::types::Event};

/// Number of Events per migration.event_batch message (D-050, spec 3.12.4).
pub const BATCH_SIZE: usize = 100;

/// Split `events` into batches of up to `BATCH_SIZE` each, preserving order.
/// The caller must ensure `events` is already in causal (topological) order.
pub fn batch_events(events: Vec<Event>) -> Vec<Vec<Event>> {
    events
        .chunks(BATCH_SIZE)
        .map(|chunk| chunk.to_vec())
        .collect()
}

/// Compute the batch_hash for a slice of Events.
///
/// Hash input: concatenation of all event_ids in the batch, in order.
/// Events with no event_id (unsigned) are skipped.
pub fn compute_batch_hash(events: &[Event]) -> String {
    // Pass 1 Commit 4: event_id is Option<EventXgid>; project to &str for the
    // canonical concatenation.
    let ids: String = events
        .iter()
        .filter_map(|e| e.event_id.as_ref().map(|x| x.as_str()))
        .collect::<Vec<_>>()
        .join("");
    hashing::hash_uri(ids.as_bytes())
}

/// Return Events from `all_events` whose event_id is NOT in `snapshot_ids`.
/// Used by the source Node to identify tail Events — new Events produced
/// after `migration.accept` that were not included in the main transfer.
pub fn identify_tail<'a>(all_events: &'a [Event], snapshot_ids: &HashSet<String>) -> Vec<&'a Event> {
    all_events
        .iter()
        .filter(|e| {
            e.event_id
                .as_ref()
                .map(|id| !snapshot_ids.contains(id.as_str()))
                .unwrap_or(false)
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        identity::keypair,
        space::state::{build_space_create_event, sign_event},
    };

    fn make_event(sender_key: &ed25519_dalek::SigningKey, space_id: &str) -> Event {
        let mut ev = build_space_create_event(sender_key, "test", None, 1, space_id);
        ev = sign_event(ev, sender_key);
        ev
    }

    #[test]
    fn batch_events_splits_correctly() {
        let key = keypair::generate();
        let events: Vec<Event> = (0..250).map(|_| make_event(&key, "")).collect();
        let batches = batch_events(events);
        // 250 events → 2 full batches + 1 partial
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].len(), BATCH_SIZE);
        assert_eq!(batches[1].len(), BATCH_SIZE);
        assert_eq!(batches[2].len(), 50);
    }

    #[test]
    fn batch_events_fewer_than_batch_size() {
        let key = keypair::generate();
        let events: Vec<Event> = (0..5).map(|_| make_event(&key, "")).collect();
        let batches = batch_events(events);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 5);
    }

    #[test]
    fn compute_batch_hash_is_deterministic() {
        let key = keypair::generate();
        let events: Vec<Event> = (0..3).map(|_| make_event(&key, "")).collect();
        let h1 = compute_batch_hash(&events);
        let h2 = compute_batch_hash(&events);
        assert_eq!(h1, h2);
        assert!(h1.starts_with("xgen://hash/sha256:"));
    }

    #[test]
    fn identify_tail_returns_new_events() {
        let key = keypair::generate();
        let main_events: Vec<Event> = (0..3).map(|_| make_event(&key, "")).collect();
        let snapshot: HashSet<String> = main_events
            .iter()
            .filter_map(|e| e.event_id.as_ref().map(|x| x.as_str().to_string()))
            .collect();

        let tail_event = make_event(&key, "");
        let all_events: Vec<Event> = main_events.iter().cloned().chain([tail_event]).collect();

        let tail = identify_tail(&all_events, &snapshot);
        assert_eq!(tail.len(), 1);
        // The tail event is not in the snapshot.
        assert!(!snapshot.contains(
            tail[0].event_id.as_ref().map(|e| e.as_str()).unwrap_or("")
        ));
    }

    #[test]
    fn identify_tail_empty_when_no_new_events() {
        let key = keypair::generate();
        let events: Vec<Event> = (0..3).map(|_| make_event(&key, "")).collect();
        let snapshot: HashSet<String> = events
            .iter()
            .filter_map(|e| e.event_id.as_ref().map(|x| x.as_str().to_string()))
            .collect();
        let tail = identify_tail(&events, &snapshot);
        assert!(tail.is_empty());
    }
}
