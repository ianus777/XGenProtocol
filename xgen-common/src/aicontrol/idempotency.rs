// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! AC-D6 idempotency store (M7C-D2, B2).
//!
//! A bounded key → completed-reply cache. The `.aicontrol` handler holds one
//! per connection (the per-`.aicontrol`-session scope, M7C-D2): a command that
//! carries an `idempotency_key` and completes **successfully** is recorded here
//! at **result-time**; a later command with the same key returns the recorded
//! reply without re-executing. `absent==do-it-over`.
//!
//! **Scope lives in placement, not the wire.** Per-session today = the store is
//! a per-connection local (dies on disconnect). End-state B widens to
//! per-driver-identity by moving the store to a driver-keyed home — the wire
//! `idempotency_key` field is unchanged. B-subsumable.
//!
//! **Bounded (FIFO).** A long-lived session cannot grow the store without bound:
//! at capacity the oldest key is evicted. An evicted key loses dedup (a replay
//! re-executes) — acceptable, idempotency windows are finite.
//!
//! **Result-time binding (B2 decision, J-221).** Only completed + successful
//! operations are recorded (the caller records on [`super::Reply::is_ok`]); an
//! errored or crashed-mid-flight operation records nothing, so a replay re-does
//! it. **In-flight policy:** there is none to build — the serial `.aicontrol`
//! handler reads the next line only after the current command's dispatch
//! returns, so a same-connection key cannot be replayed while its original is
//! still executing (by the time the replay is read, the key is recorded → it is
//! deduped). A future *pipelined* handler that allows concurrent in-flight
//! commands MUST wait-or-reject an in-flight key (never do-it-over) to preserve
//! at-most-once; that is the pipelined-handler arc's concern, not v1's.

use std::collections::{HashMap, VecDeque};

use super::envelope::Reply;

/// Default per-connection capacity. Bounds growth in a long session; a replay
/// of a key older than this many distinct keys re-executes.
pub const DEFAULT_IDEMPOTENCY_CAP: usize = 1024;

/// A bounded key → recorded-[`Reply`] cache (FIFO eviction). One per
/// `.aicontrol` connection (per-session scope, M7C-D2).
#[derive(Debug)]
pub struct IdempotencyStore {
    cap: usize,
    map: HashMap<String, Reply>,
    order: VecDeque<String>,
}

impl IdempotencyStore {
    pub fn new(cap: usize) -> Self {
        Self {
            cap: cap.max(1),
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// The recorded reply for `key`, if a prior successful command recorded it.
    pub fn get(&self, key: &str) -> Option<Reply> {
        self.map.get(key).cloned()
    }

    /// Record a completed, successful operation's reply under `key` (result-time
    /// binding). First-writer-wins (a re-record of an already-present key is a
    /// no-op — it cannot happen on a serial connection, but stays safe). Evicts
    /// the oldest key when over capacity.
    pub fn record(&mut self, key: String, reply: Reply) {
        if self.map.contains_key(&key) {
            return;
        }
        self.order.push_back(key.clone());
        self.map.insert(key, reply);
        while self.order.len() > self.cap {
            if let Some(evicted) = self.order.pop_front() {
                self.map.remove(&evicted);
            }
        }
    }

    #[cfg(test)]
    fn recorded_count(&self) -> usize {
        self.map.len()
    }
}

impl Default for IdempotencyStore {
    fn default() -> Self {
        Self::new(DEFAULT_IDEMPOTENCY_CAP)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ok(tag: &str) -> Reply {
        Reply::ok("whoami", None, json!({ "tag": tag }))
    }

    #[test]
    fn unrecorded_key_misses() {
        let s = IdempotencyStore::new(8);
        assert!(s.get("k1").is_none());
    }

    #[test]
    fn recorded_key_returns_prior_reply_verbatim() {
        let mut s = IdempotencyStore::new(8);
        s.record("k1".to_string(), ok("first"));
        let got = s.get("k1").expect("recorded key hits");
        // The cached reply round-trips identically (dedupe returns the prior result).
        assert_eq!(got.to_line(), ok("first").to_line());
    }

    #[test]
    fn first_writer_wins_on_duplicate_record() {
        let mut s = IdempotencyStore::new(8);
        s.record("k1".to_string(), ok("first"));
        s.record("k1".to_string(), ok("second"));
        assert_eq!(s.get("k1").unwrap().to_line(), ok("first").to_line());
        assert_eq!(s.recorded_count(), 1);
    }

    #[test]
    fn fifo_eviction_bounds_growth() {
        let mut s = IdempotencyStore::new(2);
        s.record("k1".to_string(), ok("1"));
        s.record("k2".to_string(), ok("2"));
        s.record("k3".to_string(), ok("3")); // evicts k1 (oldest)
        assert_eq!(s.recorded_count(), 2);
        assert!(s.get("k1").is_none(), "oldest key evicted → replay re-executes");
        assert!(s.get("k2").is_some());
        assert!(s.get("k3").is_some());
    }
}
