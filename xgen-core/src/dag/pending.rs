// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Pending buffer for Events whose dependencies are not yet known.
//
// An Event held here may be waiting on either or both of:
// 1. Missing predecessors — one or more entries in `prev_events` are not yet
//    in the local store (spec 3.2.5; F-4 unified buffering, runbook §3.2).
// 2. Missing signer Identity — the event's `sender` is not yet in the local
//    Identity registry (F-10 generalisation, runbook §3.6 + §3.6.1 Lock A2 /
//    Lock B1). This case is hit at federation first-contact when the peer
//    pushes events whose authors' Identity records are still in flight via
//    replication.
//
// The Event is held until ALL outstanding dependencies are resolved, at which
// point it is returned to the caller for processing. The caller drives both
// arrival hooks:
//   - `resolve(predecessor_id, store, id_registry)` on predecessor arrival
//   - `resolve_identity(identity_id, store, id_registry)` on Identity arrival
//
// Events that remain pending beyond PENDING_TIMEOUT_SECS are discarded
// (3.9.6, error 4002 predecessor_timeout / 4006 identity_record_timeout). The
// caller drives the sweep via drain_timed_out(); the predecessor-code-wins
// sub-rule for the both-missing case lives at the caller's emit site, not
// here (runbook §3.6.1 Lock D sub-rule).

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::identity::registry::IdentityRegistry;
use crate::wire::types::Event;

use super::store::EventStore;

/// How long a pending Event may wait for its missing dependencies before it is
/// discarded (spec 3.9.6, WD-08; F-4a + F-10a both at 30 s uniform per
/// runbook §3.5.1 and §3.6.1).
pub const PENDING_TIMEOUT_SECS: u64 = 30;

/// Returned by drain_timed_out for each discarded entry. The caller uses this
/// to choose between 4002 (predecessor_timeout) and 4006 (identity_record_timeout)
/// per the runbook §3.6.1 Lock D sub-rule, and to emit the WARN log line.
#[derive(Debug, Clone)]
pub struct TimedOut {
    pub event_id: String,
    /// Predecessor event_ids that were still missing at time of timeout.
    /// Empty when the event was waiting only on Identity arrival.
    pub missing_predecessors: Vec<String>,
    /// Identity_id that was still missing at time of timeout, if any. None
    /// when the event was waiting only on predecessors.
    pub missing_identity: Option<String>,
}

/// Internal per-event book-keeping. The struct stays private; callers see
/// (Event, Vec<predecessor_id>, Option<identity_id>) shapes via add() /
/// TimedOut.
struct BufferedEntry {
    event: Event,
    received_at: Instant,
    /// The identity_id this event was waiting on at add() time, if any.
    /// Reset to None when the identity arrives (so a subsequent
    /// predecessor-arrival resolution can confirm "identity already
    /// satisfied" without re-querying the registry).
    missing_identity: Option<String>,
}

/// Holds Events that are waiting for one or more missing dependencies
/// (predecessors and/or signer Identity).
pub struct PendingBuffer {
    /// Pending Events keyed by their own event_id, with metadata.
    events: HashMap<String, BufferedEntry>,
    /// For each missing predecessor ID, the set of pending event_ids waiting
    /// for it. Reverse index — kept in sync with `events[*].event.prev_events`.
    waiting_for: HashMap<String, HashSet<String>>,
    /// For each missing identity_id, the set of pending event_ids waiting for
    /// it. Phase 6 (runbook §3.6.1 Lock A2 — per-PendingBuffer secondary
    /// index with cross-Space fan-out driven by NodeRuntime). Reverse index
    /// — kept in sync with `events[*].missing_identity`.
    waiting_for_identity: HashMap<String, HashSet<String>>,
}

impl PendingBuffer {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            waiting_for: HashMap::new(),
            waiting_for_identity: HashMap::new(),
        }
    }

    /// Add an Event to the pending buffer.
    ///
    /// `missing_predecessors` is the subset of the Event's `prev_events` that
    /// are not yet in the store. May be empty (e.g. event waiting only on
    /// Identity arrival).
    ///
    /// `missing_identity` is the event's sender identity_id if that identity
    /// is not yet in the local Identity registry; `None` otherwise.
    ///
    /// At least one of the two parameters must indicate a missing dependency
    /// (non-empty predecessors OR Some identity); callers above this layer
    /// (`dispatch_event`) enforce that — this function does not assert it.
    pub fn add(&mut self, event: Event, missing_predecessors: &[String], missing_identity: Option<&str>) {
        let eid = match &event.event_id {
            Some(id) => id.clone(),
            None => return, // cannot buffer an event with no ID
        };
        for mid in missing_predecessors {
            self.waiting_for
                .entry(mid.clone())
                .or_default()
                .insert(eid.clone());
        }
        let missing_identity_owned = missing_identity.map(|s| s.to_string());
        if let Some(ref id) = missing_identity_owned {
            self.waiting_for_identity
                .entry(id.clone())
                .or_default()
                .insert(eid.clone());
        }
        self.events.insert(
            eid,
            BufferedEntry {
                event,
                received_at: Instant::now(),
                missing_identity: missing_identity_owned,
            },
        );
    }

    /// Notify the buffer that `resolved_id` (a predecessor event) has been
    /// added to `store`. Returns all Events that are now fully unblocked —
    /// every prev_event in the store AND the sender Identity present in
    /// `id_registry`. Removes released Events from the buffer.
    ///
    /// Events whose predecessors are now all present but whose
    /// `missing_identity` is still unsatisfied stay in the buffer; they will
    /// release on a future `resolve_identity` call.
    pub fn resolve(
        &mut self,
        resolved_id: &str,
        store: &EventStore,
        id_registry: &IdentityRegistry,
    ) -> Vec<Event> {
        let candidates = match self.waiting_for.remove(resolved_id) {
            Some(c) => c,
            None => return vec![],
        };

        let mut ready = Vec::new();
        for cid in candidates {
            if let Some(released) = self.try_release(&cid, store, id_registry) {
                ready.push(released);
            }
        }
        ready
    }

    /// Notify the buffer that `resolved_identity_id` has been added to
    /// `id_registry`. Returns all Events that are now fully unblocked —
    /// sender Identity present AND every prev_event in `store`. Removes
    /// released Events from the buffer.
    ///
    /// Phase 6 / F-10 arrival hook: called from
    /// `NodeRuntime::drain_pending_by_identity` after an Identity record
    /// lands via replication (runbook §3.6.1 Lock A2).
    pub fn resolve_identity(
        &mut self,
        resolved_identity_id: &str,
        store: &EventStore,
        id_registry: &IdentityRegistry,
    ) -> Vec<Event> {
        let candidates = match self.waiting_for_identity.remove(resolved_identity_id) {
            Some(c) => c,
            None => return vec![],
        };

        // Clear missing_identity on all candidates (the arrival hook fired
        // — that identity IS now in the registry, regardless of whether
        // predecessors are also ready). This way a later predecessor-arrival
        // can release without re-consulting the registry.
        for cid in &candidates {
            if let Some(entry) = self.events.get_mut(cid) {
                entry.missing_identity = None;
            }
        }

        let mut ready = Vec::new();
        for cid in candidates {
            if let Some(released) = self.try_release(&cid, store, id_registry) {
                ready.push(released);
            }
        }
        ready
    }

    /// Try to release a single candidate event. Returns Some(event) if all
    /// its dependencies are now satisfied (predecessors in store AND, if the
    /// entry was buffered with a missing identity, that identity now in
    /// `id_registry`); None if it stays buffered.
    ///
    /// `missing_identity: None` on the entry means "this buffer entry is not
    /// waiting on an identity arrival" — either the identity was already
    /// known at add-time, or the buffer is being driven by a structural-only
    /// layer (e.g. `RoomDag`) that does not perform identity validation. In
    /// that case identity-readiness is implicitly satisfied; only the
    /// predecessor check gates release.
    fn try_release(
        &mut self,
        candidate_event_id: &str,
        store: &EventStore,
        id_registry: &IdentityRegistry,
    ) -> Option<Event> {
        let entry = self.events.get(candidate_event_id)?;
        let all_preds_known = entry.event.prev_events.iter().all(|pid| store.contains(pid));
        let identity_satisfied = match &entry.missing_identity {
            Some(id) => id_registry.contains(id),
            None => true,
        };
        if !all_preds_known || !identity_satisfied {
            return None;
        }
        // All dependencies satisfied — remove from buffer and clean up
        // reverse indices.
        let entry = self.events.remove(candidate_event_id)?;
        for prev_id in &entry.event.prev_events {
            if let Some(waiters) = self.waiting_for.get_mut(prev_id) {
                waiters.remove(candidate_event_id);
            }
        }
        if let Some(ref id) = entry.missing_identity {
            if let Some(waiters) = self.waiting_for_identity.get_mut(id) {
                waiters.remove(candidate_event_id);
            }
        }
        Some(entry.event)
    }

    /// Discard all entries whose `received_at` is more than PENDING_TIMEOUT_SECS
    /// before `now`. Returns one TimedOut per discarded entry so the caller
    /// can emit the WARN log line + the appropriate error code (4002 /
    /// 4006 per the runbook §3.6.1 Lock D sub-rule).
    pub fn drain_timed_out(&mut self, now: Instant) -> Vec<TimedOut> {
        let timeout = Duration::from_secs(PENDING_TIMEOUT_SECS);

        let timed_out_ids: Vec<String> = self
            .events
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.received_at) > timeout)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();
        for eid in timed_out_ids {
            if let Some(entry) = self.events.remove(&eid) {
                // Collect predecessors still in waiting_for (those that did
                // not arrive within the window).
                let missing_predecessors: Vec<String> = entry
                    .event
                    .prev_events
                    .iter()
                    .filter(|pid| {
                        self.waiting_for
                            .get(*pid)
                            .map(|waiters| waiters.contains(&eid))
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect();

                // Remove from all reverse-index entries.
                for prev_id in &entry.event.prev_events {
                    if let Some(waiters) = self.waiting_for.get_mut(prev_id) {
                        waiters.remove(&eid);
                    }
                }
                if let Some(ref id) = entry.missing_identity {
                    if let Some(waiters) = self.waiting_for_identity.get_mut(id) {
                        waiters.remove(&eid);
                    }
                }

                result.push(TimedOut {
                    event_id: eid,
                    missing_predecessors,
                    missing_identity: entry.missing_identity,
                });
            }
        }
        result
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// True if an Event with this event_id is currently buffered.
    pub fn contains(&self, id: &str) -> bool {
        self.events.contains_key(id)
    }

    /// Count of buffered events currently waiting on Identity-record arrival.
    /// Phase 6 / F-10 observability surface — surfaced via
    /// `NodeState.pending_identity_replication` per runbook §3.6.1 Lock C2.
    pub fn pending_identity_count(&self) -> usize {
        self.events
            .values()
            .filter(|entry| entry.missing_identity.is_some())
            .count()
    }
}

impl Default for PendingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::registry::{IdentityRecord, IdentityRegistry};
    use crate::wire::types::{Event, EventType};
    use serde_json::json;

    fn make_event(id: &str, prev: Vec<&str>) -> Event {
        make_event_with_sender(id, prev, "xgen://pubkey/ed25519:sender")
    }

    fn make_event_with_sender(id: &str, prev: Vec<&str>, sender: &str) -> Event {
        let mut ev = Event::new(
            EventType::MessageText,
            sender.to_string(),
            "xgen://hash/sha256:room".to_string(),
            "xgen://hash/sha256:space".to_string(),
            prev.iter().map(|s| s.to_string()).collect(),
            "2026-04-27T12:00:00Z".to_string(),
            json!({}),
        );
        ev.event_id = Some(id.to_string());
        ev
    }

    fn store_with(ids: &[&str]) -> EventStore {
        let mut s = EventStore::new();
        for id in ids {
            let mut ev = Event::new(
                EventType::StateRoomCreate,
                "s".to_string(),
                "r".to_string(),
                "sp".to_string(),
                vec![],
                "2026-04-27T12:00:00Z".to_string(),
                json!({}),
            );
            ev.event_id = Some(id.to_string());
            s.insert(ev).unwrap();
        }
        s
    }

    fn make_identity(id: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: id.to_string(),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-04-27T12:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: "xgen://pubkey/ed25519:home".to_string(),
            update_version: 0,
        }
    }

    fn registry_with(ids: &[&str]) -> IdentityRegistry {
        let mut r = IdentityRegistry::new();
        for id in ids {
            r.upsert(make_identity(id));
        }
        r
    }

    fn registry_with_default_sender() -> IdentityRegistry {
        registry_with(&["xgen://pubkey/ed25519:sender"])
    }

    #[test]
    fn event_released_when_predecessor_arrives() {
        let mut buf = PendingBuffer::new();
        let mut store = EventStore::new();
        let id_registry = registry_with_default_sender();

        // E1 references E0, but E0 is not in store yet.
        let e1 = make_event("id:e1", vec!["id:e0"]);
        buf.add(e1, &["id:e0".to_string()], None);
        assert_eq!(buf.len(), 1);

        // Now E0 arrives and is inserted into the store.
        let mut e0 = Event::new(
            EventType::StateRoomCreate,
            "s".to_string(),
            "r".to_string(),
            "sp".to_string(),
            vec![],
            "2026-04-27T12:00:00Z".to_string(),
            json!({}),
        );
        e0.event_id = Some("id:e0".to_string());
        store.insert(e0).unwrap();

        let ready = buf.resolve("id:e0", &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_deref(), Some("id:e1"));
        assert!(buf.is_empty());
    }

    #[test]
    fn event_with_two_missing_predecessors_waits_for_both() {
        let mut buf = PendingBuffer::new();
        let id_registry = registry_with_default_sender();

        // E2 waits for both E0 and E1.
        let e2 = make_event("id:e2", vec!["id:e0", "id:e1"]);
        buf.add(e2, &["id:e0".to_string(), "id:e1".to_string()], None);

        // E0 arrives — E2 still waits for E1.
        let store_with_e0 = store_with(&["id:e0"]);
        let ready = buf.resolve("id:e0", &store_with_e0, &id_registry);
        assert!(ready.is_empty());
        assert_eq!(buf.len(), 1);

        // E1 arrives — E2 is now fully unblocked.
        let store_with_both = store_with(&["id:e0", "id:e1"]);
        let ready = buf.resolve("id:e1", &store_with_both, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_deref(), Some("id:e2"));
        assert!(buf.is_empty());
    }

    #[test]
    fn multiple_events_waiting_for_same_predecessor() {
        let mut buf = PendingBuffer::new();
        let id_registry = registry_with_default_sender();

        let e1 = make_event("id:e1", vec!["id:e0"]);
        let e2 = make_event("id:e2", vec!["id:e0"]);
        buf.add(e1, &["id:e0".to_string()], None);
        buf.add(e2, &["id:e0".to_string()], None);
        assert_eq!(buf.len(), 2);

        let store = store_with(&["id:e0"]);
        let ready = buf.resolve("id:e0", &store, &id_registry);
        assert_eq!(ready.len(), 2);
        assert!(buf.is_empty());
    }

    #[test]
    fn resolve_unknown_id_returns_empty() {
        let mut buf = PendingBuffer::new();
        let store = EventStore::new();
        let id_registry = IdentityRegistry::new();
        let ready = buf.resolve("id:nobody", &store, &id_registry);
        assert!(ready.is_empty());
    }

    #[test]
    fn contains_returns_correct_state() {
        let mut buf = PendingBuffer::new();
        let e1 = make_event("id:e1", vec!["id:e0"]);
        buf.add(e1, &["id:e0".to_string()], None);
        assert!(buf.contains("id:e1"));
        assert!(!buf.contains("id:e0"));
    }

    // ── Layer 13 — Timeout tests ──────────────────────────────────────────────

    #[test]
    fn pending_event_discarded_after_timeout() {
        let mut buf = PendingBuffer::new();
        let e1 = make_event("id:e1", vec!["id:e0"]);
        buf.add(e1, &["id:e0".to_string()], None);
        assert_eq!(buf.len(), 1);

        let future = Instant::now() + Duration::from_secs(PENDING_TIMEOUT_SECS + 1);
        let discarded = buf.drain_timed_out(future);

        assert_eq!(discarded.len(), 1);
        assert_eq!(discarded[0].event_id, "id:e1");
        assert!(discarded[0].missing_identity.is_none());
        assert!(buf.is_empty());
    }

    #[test]
    fn pending_event_retained_within_timeout() {
        let mut buf = PendingBuffer::new();
        let e1 = make_event("id:e1", vec!["id:e0"]);
        buf.add(e1, &["id:e0".to_string()], None);

        let near_future = Instant::now() + Duration::from_secs(PENDING_TIMEOUT_SECS - 1);
        let discarded = buf.drain_timed_out(near_future);

        assert!(discarded.is_empty());
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn timeout_logs_missing_predecessor_ids() {
        let mut buf = PendingBuffer::new();
        let e3 = make_event("id:e3", vec!["id:e1", "id:e2"]);
        buf.add(e3, &["id:e1".to_string(), "id:e2".to_string()], None);

        let future = Instant::now() + Duration::from_secs(PENDING_TIMEOUT_SECS + 1);
        let mut discarded = buf.drain_timed_out(future);

        assert_eq!(discarded.len(), 1);
        let entry = discarded.remove(0);
        assert_eq!(entry.event_id, "id:e3");

        let mut missing = entry.missing_predecessors.clone();
        missing.sort();
        assert_eq!(missing, vec!["id:e1", "id:e2"]);
        assert!(entry.missing_identity.is_none());
    }

    // ── Phase 6 / F-10 — identity-missing tests ────────────────────────────

    #[test]
    fn event_with_only_missing_identity_held_and_released_on_identity_arrival() {
        let mut buf = PendingBuffer::new();
        let store = EventStore::new();

        let sender = "xgen://pubkey/ed25519:unknown-signer";
        let ev = make_event_with_sender("id:e1", vec![], sender);
        buf.add(ev, &[], Some(sender));
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.pending_identity_count(), 1);

        // Identity registry initially empty — try resolve_identity for an
        // unrelated identity, no release.
        let id_registry_empty = IdentityRegistry::new();
        let ready = buf.resolve_identity("xgen://pubkey/ed25519:other", &store, &id_registry_empty);
        assert!(ready.is_empty());
        assert_eq!(buf.len(), 1);

        // Identity record arrives via replication — fire resolve_identity.
        let id_registry = registry_with(&[sender]);
        let ready = buf.resolve_identity(sender, &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_deref(), Some("id:e1"));
        assert!(buf.is_empty());
        assert_eq!(buf.pending_identity_count(), 0);
    }

    #[test]
    fn event_waiting_on_both_predecessor_and_identity_needs_both_arrivals() {
        let mut buf = PendingBuffer::new();
        let sender = "xgen://pubkey/ed25519:unknown-signer";
        let ev = make_event_with_sender("id:e2", vec!["id:e0"], sender);
        buf.add(ev, &["id:e0".to_string()], Some(sender));
        assert_eq!(buf.len(), 1);

        // Predecessor arrives first — identity still missing → no release.
        let store = store_with(&["id:e0"]);
        let id_registry_empty = IdentityRegistry::new();
        let ready = buf.resolve("id:e0", &store, &id_registry_empty);
        assert!(ready.is_empty(), "predecessor alone is not enough");
        assert_eq!(buf.len(), 1);

        // Identity arrives — both dependencies now satisfied → release.
        let id_registry = registry_with(&[sender]);
        let ready = buf.resolve_identity(sender, &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_deref(), Some("id:e2"));
        assert!(buf.is_empty());
    }

    #[test]
    fn event_waiting_on_both_releases_in_reverse_arrival_order() {
        // Same as above but identity arrives first, then predecessor.
        let mut buf = PendingBuffer::new();
        let sender = "xgen://pubkey/ed25519:unknown-signer";
        let ev = make_event_with_sender("id:e2", vec!["id:e0"], sender);
        buf.add(ev, &["id:e0".to_string()], Some(sender));

        // Identity arrives first — predecessor still missing → no release.
        let store_empty = EventStore::new();
        let id_registry = registry_with(&[sender]);
        let ready = buf.resolve_identity(sender, &store_empty, &id_registry);
        assert!(ready.is_empty(), "identity alone is not enough");
        assert_eq!(buf.len(), 1);
        // missing_identity has been cleared internally — pending_identity_count
        // drops to zero even though the event is still buffered (waiting on
        // the predecessor now).
        assert_eq!(buf.pending_identity_count(), 0);

        // Predecessor arrives — release.
        let store = store_with(&["id:e0"]);
        let ready = buf.resolve("id:e0", &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_deref(), Some("id:e2"));
        assert!(buf.is_empty());
    }

    #[test]
    fn predecessor_resolve_does_not_release_if_identity_still_missing() {
        // Pure-predecessor resolve call with identity still missing → stays
        // buffered. Confirms the predecessor-only resolve path also gates on
        // identity-readiness via try_release.
        let mut buf = PendingBuffer::new();
        let sender = "xgen://pubkey/ed25519:still-missing";
        let ev = make_event_with_sender("id:e1", vec!["id:e0"], sender);
        buf.add(ev, &["id:e0".to_string()], Some(sender));

        let store = store_with(&["id:e0"]);
        let id_registry = IdentityRegistry::new(); // identity still absent
        let ready = buf.resolve("id:e0", &store, &id_registry);
        assert!(ready.is_empty());
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn timeout_records_missing_identity_when_only_identity_was_missing() {
        let mut buf = PendingBuffer::new();
        let sender = "xgen://pubkey/ed25519:never-arrived";
        let ev = make_event_with_sender("id:e1", vec![], sender);
        buf.add(ev, &[], Some(sender));

        let future = Instant::now() + Duration::from_secs(PENDING_TIMEOUT_SECS + 1);
        let mut discarded = buf.drain_timed_out(future);
        assert_eq!(discarded.len(), 1);
        let to = discarded.remove(0);
        assert_eq!(to.event_id, "id:e1");
        assert!(to.missing_predecessors.is_empty());
        assert_eq!(to.missing_identity.as_deref(), Some(sender));
    }

    #[test]
    fn timeout_records_both_when_both_were_missing() {
        // Predecessor-code-wins rule lives at the caller's emit site, not in
        // PendingBuffer — the TimedOut struct carries both fields populated
        // so the caller can pick the right error code per §3.6.1 Lock D.
        let mut buf = PendingBuffer::new();
        let sender = "xgen://pubkey/ed25519:never-arrived";
        let ev = make_event_with_sender("id:e1", vec!["id:e0"], sender);
        buf.add(ev, &["id:e0".to_string()], Some(sender));

        let future = Instant::now() + Duration::from_secs(PENDING_TIMEOUT_SECS + 1);
        let mut discarded = buf.drain_timed_out(future);
        assert_eq!(discarded.len(), 1);
        let to = discarded.remove(0);
        assert_eq!(to.missing_predecessors, vec!["id:e0".to_string()]);
        assert_eq!(to.missing_identity.as_deref(), Some(sender));
    }

    #[test]
    fn pending_identity_count_tracks_correctly_across_add_and_resolve() {
        let mut buf = PendingBuffer::new();
        let s1 = "xgen://pubkey/ed25519:s1";
        let s2 = "xgen://pubkey/ed25519:s2";

        buf.add(make_event_with_sender("id:e1", vec![], s1), &[], Some(s1));
        buf.add(make_event_with_sender("id:e2", vec![], s2), &[], Some(s2));
        buf.add(make_event("id:e3", vec!["id:e0"]), &["id:e0".to_string()], None);
        assert_eq!(buf.pending_identity_count(), 2);
        assert_eq!(buf.len(), 3);

        let store = EventStore::new();
        let id_registry = registry_with(&[s1]);
        let ready = buf.resolve_identity(s1, &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(buf.pending_identity_count(), 1);
        assert_eq!(buf.len(), 2);
    }
}
