// Copyright (c) 2026 Jozef NiÅ¾nanskÃ½ / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Pending buffer for Events whose dependencies are not yet known.
//
// An Event held here may be waiting on any combination of:
// 1. Missing predecessors â€” one or more entries in `prev_events` are not yet
//    in the local store (spec 3.2.5; F-4 unified buffering, runbook Â§3.2).
// 2. Missing signer Identity â€” the event's `sender` is not yet in the local
//    Identity registry (F-10 generalisation, runbook Â§3.6 + Â§3.6.1 Lock A2 /
//    Lock B1). This case is hit at federation first-contact when the peer
//    pushes events whose authors' Identity records are still in flight via
//    replication.
// 3. Missing federation relationship for (peer, space) â€” the inbound
//    federation peer is not yet recorded in `SpaceState.federation_nodes`
//    for the target Space (Phase 7.5 Â§6 â€” third trigger). Resolved by an
//    idempotent `state.federation_add` arrival hook fired in xgen-node's
//    inbound ingestion path. Held-not-bypassed posture: F-3 is deferred,
//    not weakened â€” events sit in the buffer until F-3's data source is
//    populated, then re-validate cleanly.
//
// The Event is held until ALL outstanding dependencies are resolved, at which
// point it is returned to the caller for processing. The caller drives the
// arrival hooks:
//   - `resolve(predecessor_id, store, id_registry)` on predecessor arrival
//   - `resolve_identity(identity_id, store, id_registry)` on Identity arrival
//   - `resolve_federation_relationship(peer, space, store, id_registry)` on
//     `state.federation_add` ingestion for (peer, space)
//
// Events that remain pending beyond their applicable timeout are discarded
// (spec 3.9.6; predecessor + Identity â†’ 30 s per F-4a + F-10a; federation-
// relationship â†’ 180 s default per Phase 7.5 Â§7, configurable). Effective
// per-entry timeout = max of the per-trigger timeouts for the still-missing
// dependencies (entries waiting on federation get the longer window even if
// also waiting on predecessor/Identity, so bootstrap streams have headroom
// for federation_add to land).
//
// The predecessor-code-wins precedence (4002 > 4007 > 4006) for events with
// multiple missing dependencies at timeout time lives at the caller's emit
// site, not here (runbook Â§3.6.1 Lock D sub-rule + Phase 7.5 Â§6.3 extension).

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, SpaceXgid, Xgid};

use crate::identity::registry::IdentityRegistry;
use crate::wire::types::Event;

use super::store::EventStore;

/// Default timeout for entries waiting on predecessor and/or Identity arrival
/// (spec 3.9.6, WD-08; F-4a + F-10a both at 30 s uniform per runbook Â§3.5.1
/// and Â§3.6.1).
pub const PENDING_TIMEOUT_SECS: u64 = 30;

/// Phase 7.5 Â§7 â€” default timeout for entries waiting on federation-relationship
/// arrival. Materially different timing characteristics from predecessor /
/// Identity timeouts: bootstrap streams can take tens of seconds to deliver
/// the topologically-last `state.federation_add` across realistic WAN
/// latency. Configurable via `[sync].federation_relationship_timeout_seconds`.
pub const FEDERATION_RELATIONSHIP_TIMEOUT_SECS: u64 = 180;

/// Returned by drain_timed_out for each discarded entry. The caller uses this
/// to choose between 4002 (predecessor_timeout), 4007 (federation_relationship_timeout),
/// and 4006 (identity_record_timeout) per the runbook Â§3.6.1 Lock D sub-rule
/// extended by Phase 7.5 Â§6.3 â€” predecessor > federation-relationship > Identity.
#[derive(Debug, Clone)]
pub struct TimedOut {
    pub event_id: EventXgid,
    /// Predecessor event_ids that were still missing at time of timeout.
    /// Empty when the event was not waiting on predecessors.
    pub missing_predecessors: Vec<EventXgid>,
    /// Identity_id that was still missing at time of timeout, if any. None
    /// when the event was not waiting on Identity.
    pub missing_identity: Option<IdentityXgid>,
    /// (peer_node_id, space_id) for the federation relationship that was
    /// still missing at time of timeout, if any. None when the event was
    /// not waiting on federation-relationship arrival. Phase 7.5 §6 third
    /// trigger.
    pub missing_federation_relationship: Option<(NodeXgid, SpaceXgid)>,
}

/// Internal per-event book-keeping. The struct stays private; callers see
/// (Event, Vec<predecessor_id>, Option<identity_id>, Option<(peer, space)>)
/// shapes via add() / TimedOut.
struct BufferedEntry {
    event: Event,
    received_at: Instant,
    /// The identity_id this event was waiting on at add() time, if any.
    /// Reset to None when the identity arrives (so a subsequent
    /// predecessor-arrival resolution can confirm "identity already
    /// satisfied" without re-querying the registry).
    missing_identity: Option<IdentityXgid>,
    /// Phase 7.5 §6 — (peer_node_id, space_id) this event was waiting on at
    /// add() time for federation-relationship arrival. Reset to None when
    /// the relationship arrival hook fires for the matching pair.
    missing_federation_relationship: Option<(NodeXgid, SpaceXgid)>,
}

/// Holds Events that are waiting for one or more missing dependencies
/// (predecessors, signer Identity, and/or federation relationship).
pub struct PendingBuffer {
    /// Pending Events keyed by their own event_id, with metadata.
    events: HashMap<EventXgid, BufferedEntry>,
    /// For each missing predecessor ID, the set of pending event_ids waiting
    /// for it. Reverse index — kept in sync with `events[*].event.prev_events`.
    waiting_for: HashMap<EventXgid, HashSet<EventXgid>>,
    /// For each missing identity_id, the set of pending event_ids waiting for
    /// it. Phase 6 (runbook §3.6.1 Lock A2 — per-PendingBuffer secondary
    /// index with cross-Space fan-out driven by NodeRuntime). Reverse index
    /// — kept in sync with `events[*].missing_identity`.
    waiting_for_identity: HashMap<IdentityXgid, HashSet<EventXgid>>,
    /// For each missing (peer_node_id, space_id) pair, the set of pending
    /// event_ids waiting for the federation-relationship arrival. Phase 7.5
    /// §6 — third trigger secondary index, sibling to `waiting_for_identity`.
    /// The cross-Space arrival hook is driven by `NodeRuntime::
    /// drain_pending_by_federation_relationship`.
    waiting_for_federation_relationship: HashMap<(NodeXgid, SpaceXgid), HashSet<EventXgid>>,
}

impl PendingBuffer {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            waiting_for: HashMap::new(),
            waiting_for_identity: HashMap::new(),
            waiting_for_federation_relationship: HashMap::new(),
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
    /// `missing_federation_relationship` is the (peer_node_id, space_id) pair
    /// the event is waiting on for federation-relationship arrival (Phase 7.5
    /// Â§6 third trigger); `None` otherwise.
    ///
    /// At least one of the three parameters must indicate a missing dependency
    /// (non-empty predecessors OR Some identity OR Some federation_relationship);
    /// callers above this layer (`dispatch_event`) enforce that â€” this function
    /// does not assert it.
    pub fn add(
        &mut self,
        event: Event,
        missing_predecessors: &[EventXgid],
        missing_identity: Option<&IdentityXgid>,
        missing_federation_relationship: Option<(NodeXgid, SpaceXgid)>,
    ) {
        // Pass 2 (J-125, design §2.3 Q3.1 + checkpoint #2 Lock-α) — signature
        // retypes: missing_predecessors slice of EventXgid; missing_identity
        // borrowed &IdentityXgid for lookup-side ergonomics; missing_federation_relationship
        // owned (NodeXgid, SpaceXgid) tuple per Lock-α (insert-side; underlying
        // HashMap key is `(NodeXgid, SpaceXgid)` so the owned shape is the
        // natural fit + saves the .clone() on every insert). Bridge wraps
        // (Xgid::new(...)) dropped per Q3.5.
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
        let missing_identity_owned = missing_identity.cloned();
        if let Some(ref id) = missing_identity_owned {
            self.waiting_for_identity
                .entry(id.clone())
                .or_default()
                .insert(eid.clone());
        }
        if let Some(ref pair) = missing_federation_relationship {
            self.waiting_for_federation_relationship
                .entry(pair.clone())
                .or_default()
                .insert(eid.clone());
        }
        self.events.insert(
            eid,
            BufferedEntry {
                event,
                received_at: Instant::now(),
                missing_identity: missing_identity_owned,
                missing_federation_relationship,
            },
        );
    }

    /// Notify the buffer that `resolved_id` (a predecessor event) has been
    /// added to `store`. Returns all Events that are now fully unblocked â€”
    /// every prev_event in the store AND the sender Identity present in
    /// `id_registry`. Removes released Events from the buffer.
    ///
    /// Events whose predecessors are now all present but whose
    /// `missing_identity` is still unsatisfied stay in the buffer; they will
    /// release on a future `resolve_identity` call.
    pub fn resolve(
        &mut self,
        resolved_id: &EventXgid,
        store: &EventStore,
        id_registry: &IdentityRegistry,
    ) -> Vec<Event> {
        // Pass 2 (J-125, design §2.3 Q3.2) — signature retypes to &EventXgid;
        // bridge wrap dropped per Q3.5.
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
    /// `id_registry`. Returns all Events that are now fully unblocked â€”
    /// sender Identity present AND every prev_event in `store`. Removes
    /// released Events from the buffer.
    ///
    /// Phase 6 / F-10 arrival hook: called from
    /// `NodeRuntime::drain_pending_by_identity` after an Identity record
    /// lands via replication (runbook Â§3.6.1 Lock A2).
    pub fn resolve_identity(
        &mut self,
        resolved_identity_id: &IdentityXgid,
        store: &EventStore,
        id_registry: &IdentityRegistry,
    ) -> Vec<Event> {
        // Pass 2 (J-125, design §2.3 Q3.3) — signature retypes to &IdentityXgid;
        // bridge wrap dropped per Q3.5.
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

    /// Notify the buffer that the federation relationship for
    /// `(resolved_peer_node_id, resolved_space_id)` has been ingested (a
    /// `state.federation_add` for the pair has landed). Returns all Events
    /// that are now fully unblocked â€” federation relationship satisfied AND
    /// every prev_event in `store` AND sender Identity present. Removes
    /// released Events from the buffer.
    ///
    /// Phase 7.5 Â§6 arrival hook â€” idempotent (mirrors F-10): fires on
    /// every successful federation_add ingestion for (peer, space), not
    /// only the first. Subsequent fires for the same pair are no-ops because
    /// the secondary index has already been drained.
    pub fn resolve_federation_relationship(
        &mut self,
        resolved_peer_node_id: &NodeXgid,
        resolved_space_id: &SpaceXgid,
        store: &EventStore,
        id_registry: &IdentityRegistry,
    ) -> Vec<Event> {
        // Pass 2 (J-125, design §2.3 Q3.4) — signature retypes to (&NodeXgid, &SpaceXgid);
        // bridge wraps dropped per Q3.5. HashMap key is owned tuple; construct
        // owned for the lookup.
        let key = (
            resolved_peer_node_id.clone(),
            resolved_space_id.clone(),
        );
        let candidates = match self.waiting_for_federation_relationship.remove(&key) {
            Some(c) => c,
            None => return vec![],
        };

        // Clear missing_federation_relationship on all candidates (the
        // arrival hook fired â€” the relationship IS now established,
        // regardless of whether predecessors / Identity are also ready).
        for cid in &candidates {
            if let Some(entry) = self.events.get_mut(cid) {
                entry.missing_federation_relationship = None;
            }
        }

        let mut ready = Vec::new();
        for cid in candidates {
            if let Some(released) = self.try_release(&cid, store, id_registry) {
                ready.push(released);
            } else {
                // try_release failed — some other dependency is still
                // outstanding. The entry was added to the buffer with
                // ONLY the federation-relationship trigger (and possibly
                // missing_identity), so the original add() never indexed
                // it in waiting_for[predecessor_id]. If the predecessor
                // is what's now missing (sibling events from the same
                // bootstrap stream that haven't drained yet), the entry
                // would orphan: not in any reverse index, never released
                // by future predecessor arrivals. Re-index against any
                // currently-missing predecessors so drain_pending_uniform
                // can release it later.
                //
                // Phase 7.5 §6 — surfaced during Commit 4 integration
                // tests (B3 amendment §3.1's predecessor-chain race plays
                // out at the buffer-level too: candidates' iteration
                // order through HashSet is non-deterministic, so a child
                // event may be processed before its parent lands in the
                // store).
                self.reindex_after_partial_release(&cid, store);
            }
        }
        ready
    }

    /// Phase 7.5 §6 helper — when `resolve_federation_relationship` clears
    /// the federation trigger but `try_release` fails on a still-missing
    /// predecessor, ensure the entry is properly indexed in
    /// `waiting_for[predecessor]` so future predecessor arrivals can release
    /// it. Called only on the federation-relationship miss path because the
    /// other resolve paths (predecessor / Identity) are driven by arrivals
    /// that align with how their entries were originally indexed.
    fn reindex_after_partial_release(
        &mut self,
        candidate_event_id: &EventXgid,
        store: &EventStore,
    ) {
        let prev_events: Vec<EventXgid> = match self.events.get(candidate_event_id) {
            Some(e) => e.event.prev_events.clone(),
            None => return,
        };
        for prev in prev_events {
            if !store.contains(prev.as_str()) {
                self.waiting_for
                    .entry(prev)
                    .or_default()
                    .insert(candidate_event_id.clone());
            }
        }
    }

    /// Try to release a single candidate event. Returns Some(event) if all
    /// its dependencies are now satisfied (predecessors in store AND, if the
    /// entry was buffered with a missing identity, that identity now in
    /// `id_registry`); None if it stays buffered.
    ///
    /// `missing_identity: None` on the entry means "this buffer entry is not
    /// waiting on an identity arrival" â€” either the identity was already
    /// known at add-time, or the buffer is being driven by a structural-only
    /// layer (e.g. `RoomDag`) that does not perform identity validation. In
    /// that case identity-readiness is implicitly satisfied; only the
    /// predecessor check gates release.
    fn try_release(
        &mut self,
        candidate_event_id: &EventXgid,
        store: &EventStore,
        id_registry: &IdentityRegistry,
    ) -> Option<Event> {
        let entry = self.events.get(candidate_event_id)?;
        let all_preds_known = entry
            .event
            .prev_events
            .iter()
            .all(|pid| store.contains(pid.as_str()));
        let identity_satisfied = match &entry.missing_identity {
            // Pass 2 (J-125, Surface #4 Q4.2) — pass typed reference.
            Some(id) => id_registry.contains(id),
            None => true,
        };
        // Phase 7.5 §6 — federation-relationship readiness mirrors Identity:
        // None means "this entry is no longer waiting on federation arrival"
        // (either the arrival hook cleared it, or the entry never had a
        // federation trigger). When the hook clears the field, we trust it;
        // we do not re-consult SpaceState here because PendingBuffer does
        // not carry a borrow of the spaces map.
        let federation_satisfied = entry.missing_federation_relationship.is_none();
        if !all_preds_known || !identity_satisfied || !federation_satisfied {
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
        if let Some(ref pair) = entry.missing_federation_relationship {
            if let Some(waiters) = self.waiting_for_federation_relationship.get_mut(pair) {
                waiters.remove(candidate_event_id);
            }
        }
        Some(entry.event)
    }

    /// Discard all entries whose effective timeout has elapsed by `now`.
    /// Effective per-entry timeout = the longest applicable timeout among
    /// the entry's still-missing dependencies (Phase 7.5 Â§7):
    ///   - predecessor + Identity â†’ PENDING_TIMEOUT_SECS (30 s)
    ///   - federation-relationship (possibly combined with others) â†’
    ///     `federation_relationship_timeout` (default 180 s, configurable
    ///     via `[sync].federation_relationship_timeout_seconds`)
    ///
    /// Returns one TimedOut per discarded entry so the caller can emit the
    /// WARN log line + the appropriate error code (4002 / 4007 / 4006 per
    /// the runbook Â§3.6.1 Lock D sub-rule extended by Phase 7.5 Â§6.3:
    /// predecessor > federation-relationship > Identity).
    pub fn drain_timed_out(
        &mut self,
        now: Instant,
        federation_relationship_timeout: Duration,
    ) -> Vec<TimedOut> {
        let default_timeout = Duration::from_secs(PENDING_TIMEOUT_SECS);

        let timed_out_ids: Vec<EventXgid> = self
            .events
            .iter()
            .filter(|(_, entry)| {
                // Phase 7.5 §7 — when the entry is on the federation trigger
                // it gets the federation timeout (typically longer for
                // bootstrap streams; operator-configurable). Otherwise the
                // default predecessor/Identity timeout applies. Operators
                // who configure a shorter federation timeout than the
                // default see that shorter value take effect (no implicit
                // max() — the operator's choice wins).
                let effective = if entry.missing_federation_relationship.is_some() {
                    federation_relationship_timeout
                } else {
                    default_timeout
                };
                now.duration_since(entry.received_at) > effective
            })
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();
        for eid in timed_out_ids {
            if let Some(entry) = self.events.remove(&eid) {
                // Collect predecessors still in waiting_for (those that did
                // not arrive within the window).
                let missing_predecessors: Vec<EventXgid> = entry
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
                if let Some(ref pair) = entry.missing_federation_relationship {
                    if let Some(waiters) =
                        self.waiting_for_federation_relationship.get_mut(pair)
                    {
                        waiters.remove(&eid);
                    }
                }

                result.push(TimedOut {
                    event_id: eid,
                    missing_predecessors,
                    missing_identity: entry.missing_identity,
                    missing_federation_relationship: entry.missing_federation_relationship,
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
        // Pass 2 widens this method to take `&EventXgid`; the wrap collapses then.
        self.events
            .contains_key(&EventXgid::from_xgid(Xgid::new(id.to_string())))
    }

    /// Count of buffered events currently waiting on Identity-record arrival.
    /// Phase 6 / F-10 observability surface â€” surfaced via
    /// `NodeState.pending_identity_replication` per runbook Â§3.6.1 Lock C2.
    pub fn pending_identity_count(&self) -> usize {
        self.events
            .values()
            .filter(|entry| entry.missing_identity.is_some())
            .count()
    }

    /// Count of buffered events currently waiting on federation-relationship
    /// arrival. Phase 7.5 Â§8.2 observability surface â€” surfaced via
    /// `NodeState.pending_federation_relationship`.
    pub fn pending_federation_relationship_count(&self) -> usize {
        self.events
            .values()
            .filter(|entry| entry.missing_federation_relationship.is_some())
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

    /// Wrap a `&str` into the typed XGID flavour for test fixture construction.
    fn ev_xgid(s: &str) -> EventXgid {
        EventXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn id_xgid(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn nd_xgid(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn rm_xgid(s: &str) -> RoomXgid {
        RoomXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn sp_xgid(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }

    // Bring RoomXgid into scope for the helper above (other flavours come in
    // via the file-level use).
    use xgen_common::xgid::RoomXgid;

    fn make_event(id: &str, prev: Vec<&str>) -> Event {
        make_event_with_sender(id, prev, "xgen://pubkey/ed25519:sender")
    }

    fn make_event_with_sender(id: &str, prev: Vec<&str>, sender: &str) -> Event {
        let mut ev = Event::new(
            EventType::MessageText,
            id_xgid(sender),
            rm_xgid("xgen://hash/sha256:room"),
            sp_xgid("xgen://hash/sha256:space"),
            prev.iter().map(|s| ev_xgid(s)).collect(),
            "2026-04-27T12:00:00Z".to_string(),
            json!({}),
        );
        ev.event_id = Some(ev_xgid(id));
        ev
    }

    fn store_with(ids: &[&str]) -> EventStore {
        let mut s = EventStore::new();
        for id in ids {
            let mut ev = Event::new(
                EventType::StateSpaceCreate,
                id_xgid("s"),
                rm_xgid("r"),
                sp_xgid("sp"),
                vec![],
                "2026-04-27T12:00:00Z".to_string(),
                json!({}),
            );
            ev.event_id = Some(ev_xgid(id));
            s.insert(ev).unwrap();
        }
        s
    }

    fn make_identity(id: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: id_xgid(id),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-04-27T12:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: nd_xgid("xgen://pubkey/ed25519:home"),
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
        buf.add(e1, &["id:e0".to_string()], None, None);
        assert_eq!(buf.len(), 1);

        // Now E0 arrives and is inserted into the store.
        let mut e0 = Event::new(
            EventType::StateSpaceCreate,
            id_xgid("s"),
            rm_xgid("r"),
            sp_xgid("sp"),
            vec![],
            "2026-04-27T12:00:00Z".to_string(),
            json!({}),
        );
        e0.event_id = Some(ev_xgid("id:e0"));
        store.insert(e0).unwrap();

        let ready = buf.resolve("id:e0", &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_ref().map(|e| e.as_str()), Some("id:e1"));
        assert!(buf.is_empty());
    }

    #[test]
    fn event_with_two_missing_predecessors_waits_for_both() {
        let mut buf = PendingBuffer::new();
        let id_registry = registry_with_default_sender();

        // E2 waits for both E0 and E1.
        let e2 = make_event("id:e2", vec!["id:e0", "id:e1"]);
        buf.add(e2, &["id:e0".to_string(), "id:e1".to_string()], None, None);

        // E0 arrives â€” E2 still waits for E1.
        let store_with_e0 = store_with(&["id:e0"]);
        let ready = buf.resolve("id:e0", &store_with_e0, &id_registry);
        assert!(ready.is_empty());
        assert_eq!(buf.len(), 1);

        // E1 arrives â€” E2 is now fully unblocked.
        let store_with_both = store_with(&["id:e0", "id:e1"]);
        let ready = buf.resolve("id:e1", &store_with_both, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_ref().map(|e| e.as_str()), Some("id:e2"));
        assert!(buf.is_empty());
    }

    #[test]
    fn multiple_events_waiting_for_same_predecessor() {
        let mut buf = PendingBuffer::new();
        let id_registry = registry_with_default_sender();

        let e1 = make_event("id:e1", vec!["id:e0"]);
        let e2 = make_event("id:e2", vec!["id:e0"]);
        buf.add(e1, &["id:e0".to_string()], None, None);
        buf.add(e2, &["id:e0".to_string()], None, None);
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
        buf.add(e1, &["id:e0".to_string()], None, None);
        assert!(buf.contains("id:e1"));
        assert!(!buf.contains("id:e0"));
    }

    // â”€â”€ Layer 13 â€” Timeout tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn pending_event_discarded_after_timeout() {
        let mut buf = PendingBuffer::new();
        let e1 = make_event("id:e1", vec!["id:e0"]);
        buf.add(e1, &["id:e0".to_string()], None, None);
        assert_eq!(buf.len(), 1);

        let future = Instant::now() + Duration::from_secs(PENDING_TIMEOUT_SECS + 1);
        let discarded = buf.drain_timed_out(future, Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS));

        assert_eq!(discarded.len(), 1);
        assert_eq!(discarded[0].event_id.as_str(), "id:e1");
        assert!(discarded[0].missing_identity.is_none());
        assert!(buf.is_empty());
    }

    #[test]
    fn pending_event_retained_within_timeout() {
        let mut buf = PendingBuffer::new();
        let e1 = make_event("id:e1", vec!["id:e0"]);
        buf.add(e1, &["id:e0".to_string()], None, None);

        let near_future = Instant::now() + Duration::from_secs(PENDING_TIMEOUT_SECS - 1);
        let discarded = buf.drain_timed_out(near_future, Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS));

        assert!(discarded.is_empty());
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn timeout_logs_missing_predecessor_ids() {
        let mut buf = PendingBuffer::new();
        let e3 = make_event("id:e3", vec!["id:e1", "id:e2"]);
        buf.add(e3, &["id:e1".to_string(), "id:e2".to_string()], None, None);

        let future = Instant::now() + Duration::from_secs(PENDING_TIMEOUT_SECS + 1);
        let mut discarded = buf.drain_timed_out(future, Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS));

        assert_eq!(discarded.len(), 1);
        let entry = discarded.remove(0);
        assert_eq!(entry.event_id.as_str(), "id:e3");

        let mut missing: Vec<String> = entry
            .missing_predecessors
            .iter()
            .map(|e| e.as_str().to_string())
            .collect();
        missing.sort();
        assert_eq!(missing, vec!["id:e1", "id:e2"]);
        assert!(entry.missing_identity.is_none());
    }

    // â”€â”€ Phase 6 / F-10 â€” identity-missing tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    #[test]
    fn event_with_only_missing_identity_held_and_released_on_identity_arrival() {
        let mut buf = PendingBuffer::new();
        let store = EventStore::new();

        let sender = "xgen://pubkey/ed25519:unknown-signer";
        let ev = make_event_with_sender("id:e1", vec![], sender);
        buf.add(ev, &[], Some(sender), None);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.pending_identity_count(), 1);

        // Identity registry initially empty â€” try resolve_identity for an
        // unrelated identity, no release.
        let id_registry_empty = IdentityRegistry::new();
        let ready = buf.resolve_identity("xgen://pubkey/ed25519:other", &store, &id_registry_empty);
        assert!(ready.is_empty());
        assert_eq!(buf.len(), 1);

        // Identity record arrives via replication â€” fire resolve_identity.
        let id_registry = registry_with(&[sender]);
        let ready = buf.resolve_identity(sender, &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_ref().map(|e| e.as_str()), Some("id:e1"));
        assert!(buf.is_empty());
        assert_eq!(buf.pending_identity_count(), 0);
    }

    #[test]
    fn event_waiting_on_both_predecessor_and_identity_needs_both_arrivals() {
        let mut buf = PendingBuffer::new();
        let sender = "xgen://pubkey/ed25519:unknown-signer";
        let ev = make_event_with_sender("id:e2", vec!["id:e0"], sender);
        buf.add(ev, &["id:e0".to_string()], Some(sender), None);
        assert_eq!(buf.len(), 1);

        // Predecessor arrives first â€” identity still missing â†’ no release.
        let store = store_with(&["id:e0"]);
        let id_registry_empty = IdentityRegistry::new();
        let ready = buf.resolve("id:e0", &store, &id_registry_empty);
        assert!(ready.is_empty(), "predecessor alone is not enough");
        assert_eq!(buf.len(), 1);

        // Identity arrives â€” both dependencies now satisfied â†’ release.
        let id_registry = registry_with(&[sender]);
        let ready = buf.resolve_identity(sender, &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_ref().map(|e| e.as_str()), Some("id:e2"));
        assert!(buf.is_empty());
    }

    #[test]
    fn event_waiting_on_both_releases_in_reverse_arrival_order() {
        // Same as above but identity arrives first, then predecessor.
        let mut buf = PendingBuffer::new();
        let sender = "xgen://pubkey/ed25519:unknown-signer";
        let ev = make_event_with_sender("id:e2", vec!["id:e0"], sender);
        buf.add(ev, &["id:e0".to_string()], Some(sender), None);

        // Identity arrives first â€” predecessor still missing â†’ no release.
        let store_empty = EventStore::new();
        let id_registry = registry_with(&[sender]);
        let ready = buf.resolve_identity(sender, &store_empty, &id_registry);
        assert!(ready.is_empty(), "identity alone is not enough");
        assert_eq!(buf.len(), 1);
        // missing_identity has been cleared internally â€” pending_identity_count
        // drops to zero even though the event is still buffered (waiting on
        // the predecessor now).
        assert_eq!(buf.pending_identity_count(), 0);

        // Predecessor arrives â€” release.
        let store = store_with(&["id:e0"]);
        let ready = buf.resolve("id:e0", &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_ref().map(|e| e.as_str()), Some("id:e2"));
        assert!(buf.is_empty());
    }

    #[test]
    fn predecessor_resolve_does_not_release_if_identity_still_missing() {
        // Pure-predecessor resolve call with identity still missing â†’ stays
        // buffered. Confirms the predecessor-only resolve path also gates on
        // identity-readiness via try_release.
        let mut buf = PendingBuffer::new();
        let sender = "xgen://pubkey/ed25519:still-missing";
        let ev = make_event_with_sender("id:e1", vec!["id:e0"], sender);
        buf.add(ev, &["id:e0".to_string()], Some(sender), None);

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
        buf.add(ev, &[], Some(sender), None);

        let future = Instant::now() + Duration::from_secs(PENDING_TIMEOUT_SECS + 1);
        let mut discarded = buf.drain_timed_out(future, Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS));
        assert_eq!(discarded.len(), 1);
        let to = discarded.remove(0);
        assert_eq!(to.event_id.as_str(), "id:e1");
        assert!(to.missing_predecessors.is_empty());
        assert_eq!(to.missing_identity.as_ref().map(|i| i.as_str()), Some(sender));
    }

    #[test]
    fn timeout_records_both_when_both_were_missing() {
        // Predecessor-code-wins rule lives at the caller's emit site, not in
        // PendingBuffer â€” the TimedOut struct carries both fields populated
        // so the caller can pick the right error code per Â§3.6.1 Lock D.
        let mut buf = PendingBuffer::new();
        let sender = "xgen://pubkey/ed25519:never-arrived";
        let ev = make_event_with_sender("id:e1", vec!["id:e0"], sender);
        buf.add(ev, &["id:e0".to_string()], Some(sender), None);

        let future = Instant::now() + Duration::from_secs(PENDING_TIMEOUT_SECS + 1);
        let mut discarded = buf.drain_timed_out(future, Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS));
        assert_eq!(discarded.len(), 1);
        let to = discarded.remove(0);
        assert_eq!(
            to.missing_predecessors
                .iter()
                .map(|e| e.as_str().to_string())
                .collect::<Vec<_>>(),
            vec!["id:e0".to_string()]
        );
        assert_eq!(to.missing_identity.as_ref().map(|i| i.as_str()), Some(sender));
    }

    #[test]
    fn pending_identity_count_tracks_correctly_across_add_and_resolve() {
        let mut buf = PendingBuffer::new();
        let s1 = "xgen://pubkey/ed25519:s1";
        let s2 = "xgen://pubkey/ed25519:s2";

        buf.add(make_event_with_sender("id:e1", vec![], s1), &[], Some(s1), None);
        buf.add(make_event_with_sender("id:e2", vec![], s2), &[], Some(s2), None);
        buf.add(make_event("id:e3", vec!["id:e0"]), &["id:e0".to_string()], None, None);
        assert_eq!(buf.pending_identity_count(), 2);
        assert_eq!(buf.len(), 3);

        let store = EventStore::new();
        let id_registry = registry_with(&[s1]);
        let ready = buf.resolve_identity(s1, &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(buf.pending_identity_count(), 1);
        assert_eq!(buf.len(), 2);
    }

    // ── Phase 7.5 §6 — federation-relationship trigger tests ──────────────

    const PEER_A: &str = "xgen://pubkey/ed25519:peer_a";
    const PEER_B: &str = "xgen://pubkey/ed25519:peer_b";
    const SPACE_S: &str = "xgen://hash/sha256:space_s";
    const SPACE_T: &str = "xgen://hash/sha256:space_t";

    fn federation_pair(peer: &str, space: &str) -> Option<(String, String)> {
        Some((peer.to_string(), space.to_string()))
    }

    /// Typed pair for asserting on `TimedOut.missing_federation_relationship`
    /// and `BufferedEntry.missing_federation_relationship` post-Pass-1 retype.
    fn federation_pair_typed(peer: &str, space: &str) -> Option<(NodeXgid, SpaceXgid)> {
        Some((nd_xgid(peer), sp_xgid(space)))
    }

    #[test]
    fn event_with_only_missing_federation_relationship_held_and_released_on_arrival() {
        let mut buf = PendingBuffer::new();
        let store = EventStore::new();
        let id_registry = registry_with_default_sender();

        // event has no missing predecessors and a known sender, but the
        // (peer, space) federation relationship is not yet established.
        let ev = make_event("id:e1", vec![]);
        buf.add(ev, &[], None, federation_pair(PEER_A, SPACE_S));
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.pending_federation_relationship_count(), 1);

        // Resolve for a different pair — no release.
        let ready =
            buf.resolve_federation_relationship(PEER_B, SPACE_S, &store, &id_registry);
        assert!(ready.is_empty());
        assert_eq!(buf.len(), 1);

        // Resolve for the right pair — release.
        let ready =
            buf.resolve_federation_relationship(PEER_A, SPACE_S, &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_ref().map(|e| e.as_str()), Some("id:e1"));
        assert!(buf.is_empty());
        assert_eq!(buf.pending_federation_relationship_count(), 0);
    }

    #[test]
    fn resolve_federation_relationship_idempotent_after_drain() {
        // Phase 7.5 §6.3 — arrival hook fires on every federation_add
        // ingestion, not only the first. After first fire drains the
        // matching entries, subsequent fires are no-ops.
        let mut buf = PendingBuffer::new();
        let store = EventStore::new();
        let id_registry = registry_with_default_sender();

        let ev = make_event("id:e1", vec![]);
        buf.add(ev, &[], None, federation_pair(PEER_A, SPACE_S));
        let ready =
            buf.resolve_federation_relationship(PEER_A, SPACE_S, &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert!(buf.is_empty());

        // Second fire — no-op, no panic.
        let ready =
            buf.resolve_federation_relationship(PEER_A, SPACE_S, &store, &id_registry);
        assert!(ready.is_empty());
        assert!(buf.is_empty());
    }

    #[test]
    fn resolve_federation_relationship_for_different_space_does_not_drain() {
        // (PEER_A, SPACE_T) arrival must not release entries waiting on
        // (PEER_A, SPACE_S).
        let mut buf = PendingBuffer::new();
        let store = EventStore::new();
        let id_registry = registry_with_default_sender();

        let ev = make_event("id:e1", vec![]);
        buf.add(ev, &[], None, federation_pair(PEER_A, SPACE_S));

        let ready =
            buf.resolve_federation_relationship(PEER_A, SPACE_T, &store, &id_registry);
        assert!(ready.is_empty(), "different space must not drain");
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn event_waiting_on_identity_and_federation_relationship_needs_both() {
        let mut buf = PendingBuffer::new();
        let store = EventStore::new();
        let sender = "xgen://pubkey/ed25519:unknown-signer";
        let ev = make_event_with_sender("id:e1", vec![], sender);
        buf.add(
            ev,
            &[],
            Some(sender),
            federation_pair(PEER_A, SPACE_S),
        );

        // Identity arrives first — federation still missing → no release.
        let id_registry = registry_with(&[sender]);
        let ready = buf.resolve_identity(sender, &store, &id_registry);
        assert!(ready.is_empty(), "identity alone is not enough");
        assert_eq!(buf.len(), 1);

        // Federation arrives — release.
        let ready =
            buf.resolve_federation_relationship(PEER_A, SPACE_S, &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_ref().map(|e| e.as_str()), Some("id:e1"));
        assert!(buf.is_empty());
    }

    #[test]
    fn event_waiting_on_all_three_triggers_needs_all_three() {
        let mut buf = PendingBuffer::new();
        let sender = "xgen://pubkey/ed25519:unknown-signer";
        let ev = make_event_with_sender("id:e2", vec!["id:e0"], sender);
        buf.add(
            ev,
            &["id:e0".to_string()],
            Some(sender),
            federation_pair(PEER_A, SPACE_S),
        );
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.pending_identity_count(), 1);
        assert_eq!(buf.pending_federation_relationship_count(), 1);

        // Predecessor first — still waiting on identity + federation.
        let store_with_e0 = store_with(&["id:e0"]);
        let id_registry_empty = IdentityRegistry::new();
        let ready = buf.resolve("id:e0", &store_with_e0, &id_registry_empty);
        assert!(ready.is_empty());

        // Identity arrives — still waiting on federation.
        let id_registry = registry_with(&[sender]);
        let ready = buf.resolve_identity(sender, &store_with_e0, &id_registry);
        assert!(ready.is_empty());

        // Federation arrives — release.
        let ready = buf
            .resolve_federation_relationship(PEER_A, SPACE_S, &store_with_e0, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].event_id.as_ref().map(|e| e.as_str()), Some("id:e2"));
    }

    #[test]
    fn timeout_records_federation_relationship_when_only_federation_missing() {
        let mut buf = PendingBuffer::new();
        let ev = make_event("id:e1", vec![]);
        buf.add(ev, &[], None, federation_pair(PEER_A, SPACE_S));

        // Default timeout would have us wait FEDERATION_RELATIONSHIP_TIMEOUT_SECS.
        // The entry waiting on federation gets the longer window.
        let still_within_window =
            Instant::now() + Duration::from_secs(PENDING_TIMEOUT_SECS + 1);
        let discarded = buf.drain_timed_out(
            still_within_window,
            Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS),
        );
        assert!(
            discarded.is_empty(),
            "entry on federation trigger gets the longer window, not the 30 s default"
        );

        // Past the 180 s window — entry times out.
        let past_federation_window = Instant::now()
            + Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS + 1);
        let mut discarded = buf.drain_timed_out(
            past_federation_window,
            Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS),
        );
        assert_eq!(discarded.len(), 1);
        let to = discarded.remove(0);
        assert_eq!(to.event_id.as_str(), "id:e1");
        assert!(to.missing_predecessors.is_empty());
        assert!(to.missing_identity.is_none());
        assert_eq!(
            to.missing_federation_relationship.as_ref(),
            federation_pair_typed(PEER_A, SPACE_S).as_ref()
        );
    }

    #[test]
    fn timeout_uses_short_window_when_no_federation_trigger() {
        let mut buf = PendingBuffer::new();
        let e1 = make_event("id:e1", vec!["id:e0"]);
        buf.add(e1, &["id:e0".to_string()], None, None);

        // Without federation trigger the entry stays on the default 30 s
        // window, even when we pass a long federation timeout.
        let past_default = Instant::now() + Duration::from_secs(PENDING_TIMEOUT_SECS + 1);
        let discarded = buf.drain_timed_out(
            past_default,
            Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS),
        );
        assert_eq!(discarded.len(), 1);
        assert!(discarded[0].missing_federation_relationship.is_none());
    }

    #[test]
    fn timeout_records_all_three_when_all_three_missing() {
        // The TimedOut struct carries all three fields populated when
        // applicable; the precedence-at-emit-site rule (predecessor > federation > Identity)
        // is enforced by the caller in xgen-node::app, not here.
        let mut buf = PendingBuffer::new();
        let sender = "xgen://pubkey/ed25519:never-arrived";
        let ev = make_event_with_sender("id:e1", vec!["id:e0"], sender);
        buf.add(
            ev,
            &["id:e0".to_string()],
            Some(sender),
            federation_pair(PEER_A, SPACE_S),
        );

        let past_federation = Instant::now()
            + Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS + 1);
        let mut discarded = buf.drain_timed_out(
            past_federation,
            Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS),
        );
        assert_eq!(discarded.len(), 1);
        let to = discarded.remove(0);
        assert_eq!(
            to.missing_predecessors
                .iter()
                .map(|e| e.as_str().to_string())
                .collect::<Vec<_>>(),
            vec!["id:e0".to_string()]
        );
        assert_eq!(to.missing_identity.as_ref().map(|i| i.as_str()), Some(sender));
        assert_eq!(
            to.missing_federation_relationship.as_ref(),
            federation_pair_typed(PEER_A, SPACE_S).as_ref()
        );
    }

    #[test]
    fn pending_federation_relationship_count_tracks_across_add_and_resolve() {
        let mut buf = PendingBuffer::new();
        let store = EventStore::new();
        let id_registry = registry_with_default_sender();

        buf.add(
            make_event("id:e1", vec![]),
            &[],
            None,
            federation_pair(PEER_A, SPACE_S),
        );
        buf.add(
            make_event("id:e2", vec![]),
            &[],
            None,
            federation_pair(PEER_A, SPACE_T),
        );
        buf.add(
            make_event("id:e3", vec!["id:e0"]),
            &["id:e0".to_string()],
            None,
            None,
        );
        assert_eq!(buf.pending_federation_relationship_count(), 2);
        assert_eq!(buf.len(), 3);

        // Arrival for (PEER_A, SPACE_S) drains one.
        let ready =
            buf.resolve_federation_relationship(PEER_A, SPACE_S, &store, &id_registry);
        assert_eq!(ready.len(), 1);
        assert_eq!(buf.pending_federation_relationship_count(), 1);
        assert_eq!(buf.len(), 2);
    }
}
