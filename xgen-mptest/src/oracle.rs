// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Convergence / rejection oracle (M9-D4 / C2 — Checkpoint #2 surface).
//!
//! The cross-process analogue of the in-process `RoomState` `Eq` oracle. Two
//! readings per node:
//!
//! 1. **State projection** (the primary convergence signal — the `RoomState` Eq
//!    analogue): the resolved per-Space membership, read via the `members` verb
//!    on a client connected to each node. Convergence ⇒ the membership **set**
//!    (`identity_id → role`) and `owner_id` are equal across all nodes hosting
//!    the Space.
//! 2. **`.events` transcript** (the ordered applied-events record, collected by
//!    [`crate::events::EventCollector`]): events arrive in different orders on
//!    different nodes (that is exactly what convergence tolerates), so the
//!    transcript is compared as the **set of applied `event_id`s** per Space —
//!    same events present, regardless of arrival order. Rejection scenarios
//!    assert the offending `event_id` is **absent** from every node's set.
//!
//! ## Proposed equality definition (for Joe-lock at Checkpoint #2)
//! - **Membership equality** = equal `owner_id` + equal set of
//!   `(identity_id, role)` pairs. `joined_at` is **excluded** (it legitimately
//!   differs across nodes — each stamps on local apply); `invited_by` is
//!   **excluded** from the convergence key for the same reason (it can be read
//!   into the capture for diagnostics but is not part of the Eq).
//! - **Transcript convergence** = equal set of `event_id`s for the Space across
//!   nodes (order-independent — the resolved *state* is the convergence claim,
//!   not arrival order).
//! - **Rejection** = the offending `event_id` is in **no** node's transcript
//!   **and** absent from every node's membership (never applied anywhere).
//!
//! These three are the C2 decision; the comparators below implement the
//! proposal and are easy to retune once locked.

use std::collections::{BTreeMap, BTreeSet};

use crate::batch::ActorRun;

/// A lightweight projection of one streamed `Event` JSON value — only the
/// fields the oracle + capture need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRecord {
    pub event_type: String,
    pub event_id: Option<String>,
    pub sender: String,
    pub space_id: String,
    pub room_id: String,
}

impl EventRecord {
    /// The Space this event belongs to, mirroring `Event::effective_space_id()`:
    /// a create event carries an **empty** `space_id` and *is* the Space (its
    /// `event_id` is the space id); every other event carries the parent Space in
    /// `space_id`. Without this, `state.space_create` would attribute to "" and
    /// never match its own Space.
    pub fn effective_space_id(&self) -> Option<&str> {
        if self.space_id.is_empty() {
            self.event_id.as_deref()
        } else {
            Some(&self.space_id)
        }
    }

    /// True if this event is federation-bootstrap infrastructure, excluded from
    /// the Space-scoped **cooperative** convergence comparison (MP-R1-D7).
    ///
    /// The set is the canonical `xgen_core::INFRA_EVENT_KINDS` (MP-F14 — the
    /// string-keyed mirror of `EventType::is_federation_infra`, single source of
    /// truth shared with the client `get_dag_tips` cooperative-frontier fix). We
    /// hold a wire `type` **string** here (`EventRecord.event_type`), so we match
    /// against the string list rather than the typed predicate.
    ///
    /// Why `state.federation_add` is excluded: the G-6 / M9.2 `add-peer` mechanism
    /// writes it **directionally and asymmetrically** (D-075) — the initiating node
    /// (A) records the relationship in its `FederationRegistry` only (no DAG event),
    /// while the receiving node (B) materializes a `state.federation_add` in the
    /// Space DAG on receipt. So a converged cross-node Space shows an expected
    /// **A:0 / B:1** distribution — directional, not symmetric, and **not** a lost
    /// event (both registries hold the active link; membership still converges).
    /// Excluding it scopes the transcript oracle to cooperative content; membership
    /// convergence stays the positive "federation-formed + state-converged"
    /// assertion. Do **not** "fix" this back to a symmetric model.
    pub fn is_infra(&self) -> bool {
        xgen_core::INFRA_EVENT_KINDS.contains(&self.event_type.as_str())
    }

    /// Extract from a raw `.events` JSON value (the serialized `Event`; note the
    /// event type is the `"type"` field per `#[serde(rename = "type")]`).
    pub fn from_value(v: &serde_json::Value) -> Option<EventRecord> {
        Some(EventRecord {
            event_type: v.get("type")?.as_str()?.to_string(),
            event_id: v.get("event_id").and_then(|x| x.as_str()).map(String::from),
            sender: v.get("sender").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            space_id: v.get("space_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            room_id: v.get("room_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        })
    }
}

/// One node's ordered `.events` transcript (arrival order).
#[derive(Debug, Clone, Default)]
pub struct Transcript {
    pub node: String,
    pub events: Vec<EventRecord>,
}

impl Transcript {
    /// Build from a node's collected raw event values (skips any that don't
    /// parse as an Event — defensive).
    pub fn from_values(node: &str, values: &[serde_json::Value]) -> Transcript {
        Transcript {
            node: node.to_string(),
            events: values.iter().filter_map(EventRecord::from_value).collect(),
        }
    }

    /// The set of **all** `event_id`s applied for a given Space (raw — includes
    /// infra). Uses the **effective** space id, so the `state.space_create` event
    /// (whose own `event_id` is the space id, empty `space_id` field) is counted.
    pub fn event_ids_for_space(&self, space_id: &str) -> BTreeSet<String> {
        self.events
            .iter()
            .filter(|e| e.effective_space_id() == Some(space_id))
            .filter_map(|e| e.event_id.clone())
            .collect()
    }

    /// The set of **cooperative** `event_id`s for a Space — [`event_ids_for_space`]
    /// minus federation-bootstrap infra (see [`EventRecord::is_infra`]). This is the
    /// cross-node convergence key (MP-R1-D7): the harness G-6 / M9.2 add-peer
    /// mechanism writes `state.federation_add` asymmetrically (A:0 registry-only /
    /// B:1 materialized), so the raw set diverges on benign infra while the
    /// cooperative content matches.
    ///
    /// [`event_ids_for_space`]: Transcript::event_ids_for_space
    pub fn cooperative_event_ids_for_space(&self, space_id: &str) -> BTreeSet<String> {
        self.events
            .iter()
            .filter(|e| e.effective_space_id() == Some(space_id))
            .filter(|e| !e.is_infra())
            .filter_map(|e| e.event_id.clone())
            .collect()
    }

    /// `true` if `event_id` appears anywhere in this transcript.
    pub fn contains_event(&self, event_id: &str) -> bool {
        self.events
            .iter()
            .any(|e| e.event_id.as_deref() == Some(event_id))
    }
}

/// The resolved membership projection of a Space on one node, from a `members`
/// reply's `data`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipProjection {
    pub node: String,
    pub space_id: String,
    pub owner_id: String,
    /// The convergence key: `identity_id → role`. `joined_at`/`invited_by`
    /// excluded (node-local, legitimately divergent).
    pub members: BTreeMap<String, String>,
    /// MP-R3-D4d — the **topology node** this view was read from (`None` for the
    /// R1/R2 per-actor path). When set, [`per_node_projections`] collapses
    /// multiple actors on one node to a single per-node projection, so the
    /// convergence comparison is node-to-node and a churning actor that is
    /// mid-leave at sample time (returns no view) cannot break the ≥2 precondition.
    pub node_label: Option<String>,
}

impl MembershipProjection {
    /// Build from a `members` reply `data` object.
    pub fn from_members_data(node: &str, data: &serde_json::Value) -> Option<MembershipProjection> {
        let space_id = data.get("space_id")?.as_str()?.to_string();
        let owner_id = data.get("owner_id")?.as_str()?.to_string();
        let mut members = BTreeMap::new();
        for m in data.get("members")?.as_array()? {
            let id = m.get("identity_id")?.as_str()?.to_string();
            let role = m
                .get("role")
                .map(|r| r.to_string().trim_matches('"').to_string())
                .unwrap_or_default();
            members.insert(id, role);
        }
        Some(MembershipProjection {
            node: node.to_string(),
            space_id,
            owner_id,
            members,
            node_label: None,
        })
    }

    /// MP-R3-D4d — tag this projection with the topology node it was read from
    /// (so [`per_node_projections`] can collapse per node). Additive; the R1/R2
    /// per-actor path leaves `node_label` `None`.
    pub fn with_node_label(mut self, label: impl Into<String>) -> MembershipProjection {
        self.node_label = Some(label.into());
        self
    }

    /// Convergence equality (proposal): same owner + same `(identity, role)`
    /// set. Node label is excluded (it is just the vantage).
    pub fn converges_with(&self, other: &MembershipProjection) -> bool {
        self.owner_id == other.owner_id && self.members == other.members
    }
}

/// An oracle verdict with a human-readable detail line.
#[derive(Debug, Clone)]
pub struct OracleVerdict {
    pub pass: bool,
    pub detail: String,
}

impl OracleVerdict {
    pub fn pass(detail: impl Into<String>) -> OracleVerdict {
        OracleVerdict {
            pass: true,
            detail: detail.into(),
        }
    }
    pub fn fail(detail: impl Into<String>) -> OracleVerdict {
        OracleVerdict {
            pass: false,
            detail: detail.into(),
        }
    }
}

/// Convergence verdict: every node's membership projection agrees, and every
/// node's transcript carries the same set of `event_id`s for the Space.
pub fn convergence_verdict(
    projections: &[MembershipProjection],
    transcripts: &[Transcript],
    space_id: &str,
) -> OracleVerdict {
    if projections.len() < 2 {
        return OracleVerdict::fail(format!(
            "convergence needs ≥2 node projections, got {}",
            projections.len()
        ));
    }
    let first = &projections[0];
    for p in &projections[1..] {
        if !first.converges_with(p) {
            return OracleVerdict::fail(format!(
                "membership diverges between node `{}` ({:?}, owner {}) and node `{}` ({:?}, owner {})",
                first.node, first.members, first.owner_id, p.node, p.members, p.owner_id
            ));
        }
    }
    // Cooperative transcript event-id set equality for the Space (infra excluded,
    // MP-R1-D7 — federation-bootstrap events are asymmetric by construction).
    if transcripts.len() >= 2 {
        let first_ids = transcripts[0].cooperative_event_ids_for_space(space_id);
        for t in &transcripts[1..] {
            let ids = t.cooperative_event_ids_for_space(space_id);
            if ids != first_ids {
                let only_a: Vec<_> = first_ids.difference(&ids).cloned().collect();
                let only_b: Vec<_> = ids.difference(&first_ids).cloned().collect();
                return OracleVerdict::fail(format!(
                    "transcript event-id sets diverge for space {space_id} between `{}` and `{}`: \
                     only in `{}`={:?}, only in `{}`={:?}",
                    transcripts[0].node, t.node, transcripts[0].node, only_a, t.node, only_b
                ));
            }
        }
    }
    OracleVerdict::pass(format!(
        "{} nodes converge on membership {:?} (owner {}) for space {space_id}",
        projections.len(),
        first.members,
        first.owner_id
    ))
}

/// MP-R3-D4d — collapse per-actor projections to one per **topology node**
/// (keeping the first projection seen for each `node_label`). Projections with no
/// `node_label` (the R1/R2 per-actor path) are kept as-is. This decouples the
/// convergence comparison from *which* client reported: at capstone churn a
/// churning actor that is mid-leave at sample time simply returns no projection,
/// and a stable reader on the same node carries that node's view — so the
/// ≥2-projection precondition holds on the node count, not the per-actor count.
pub fn per_node_projections(projections: &[MembershipProjection]) -> Vec<MembershipProjection> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out: Vec<MembershipProjection> = Vec::new();
    for p in projections {
        match &p.node_label {
            Some(label) => {
                if seen.insert(label.clone()) {
                    out.push(p.clone());
                }
            }
            None => out.push(p.clone()),
        }
    }
    out
}

/// MP-R3-D4d — convergence verdict computed **per topology node** (the
/// churn-at-scale oracle): collapse the per-actor projections via
/// [`per_node_projections`] first, then run the same [`convergence_verdict`]. Use
/// this for churn rows where actors may be mid-leave at sample time; the per-actor
/// [`convergence_verdict`] stays the default for non-churn rows.
pub fn node_convergence_verdict(
    projections: &[MembershipProjection],
    transcripts: &[Transcript],
    space_id: &str,
) -> OracleVerdict {
    let per_node = per_node_projections(projections);
    convergence_verdict(&per_node, transcripts, space_id)
}

/// Rejection verdict: the offending `event_id` is absent from every node's
/// transcript (it was never applied anywhere). For wire-injection scenarios
/// (MP-A-05) where the forged event is rejected at `ingest_event`.
pub fn rejection_verdict(transcripts: &[Transcript], offending_event_id: &str) -> OracleVerdict {
    for t in transcripts {
        if t.contains_event(offending_event_id) {
            return OracleVerdict::fail(format!(
                "offending event {offending_event_id} WAS applied on node `{}` (expected rejection)",
                t.node
            ));
        }
    }
    OracleVerdict::pass(format!(
        "offending event {offending_event_id} absent from all {} node transcripts (rejected)",
        transcripts.len()
    ))
}

/// Logic-rejection verdict (MP-R1-D4 / C3): assert the reply captured for
/// `command_id` is an **error** with the expected wire `code` (and, if given, the
/// expected `category`). The data already lives in `ActorRun.replies` — this just
/// reads it. The C6/C7 logic-adversarial tranches (MP-A-02/03/04/14/16/17/20,
/// MP-A-15) assert rejection this way.
///
/// The exact `code`/`category` strings a protocol rejection carries on the wire
/// are grounded against live replies by the scenario author (C6/C7); this helper
/// is string-agnostic.
pub fn rejection_category_verdict(
    run: &ActorRun,
    command_id: &str,
    expected_code: &str,
    expected_category: Option<&str>,
) -> OracleVerdict {
    let reply = match run.reply_for(command_id) {
        Some(r) => r,
        None => {
            return OracleVerdict::fail(format!(
                "actor `{}` has no reply for command `{command_id}` (expected rejection `{expected_code}`)",
                run.actor
            ))
        }
    };
    let err = match reply.error() {
        Some(e) => e,
        None => {
            return OracleVerdict::fail(format!(
                "command `{command_id}` SUCCEEDED — expected rejection `{expected_code}`"
            ))
        }
    };
    if err.code != expected_code {
        return OracleVerdict::fail(format!(
            "command `{command_id}` rejected with code `{}` ({}), expected `{expected_code}`",
            err.code, err.category
        ));
    }
    if let Some(cat) = expected_category {
        if err.category != cat {
            return OracleVerdict::fail(format!(
                "command `{command_id}` rejected with category `{}`, expected `{cat}` (code `{}` matched)",
                err.category, err.code
            ));
        }
    }
    OracleVerdict::pass(format!(
        "command `{command_id}` correctly rejected: code `{}`, category `{}`",
        err.code, err.category
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ev(id: &str, ty: &str, space: &str) -> serde_json::Value {
        json!({
            "type": ty,
            "event_id": id,
            "sender": "xgen://pubkey/ed25519:S",
            "space_id": space,
            "room_id": "xgen://hash/sha256:R"
        })
    }

    #[test]
    fn event_record_extracts_type_from_renamed_field() {
        let r = EventRecord::from_value(&ev("E1", "membership.join", "S")).unwrap();
        assert_eq!(r.event_type, "membership.join");
        assert_eq!(r.event_id.as_deref(), Some("E1"));
        assert_eq!(r.space_id, "S");
    }

    #[test]
    fn create_event_attributes_to_its_own_id_via_effective_space() {
        // state.space_create: empty space_id, event_id IS the space id.
        let create = json!({
            "type": "state.space_create",
            "event_id": "SPACE1",
            "sender": "ALICE",
            "space_id": "",
            "room_id": ""
        });
        let r = EventRecord::from_value(&create).unwrap();
        assert_eq!(r.effective_space_id(), Some("SPACE1"));
        let t = Transcript::from_values("a", &[create]);
        assert!(t.event_ids_for_space("SPACE1").contains("SPACE1"));
    }

    #[test]
    fn membership_converges_ignores_node_and_joined_at() {
        let a = MembershipProjection::from_members_data(
            "a",
            &json!({"space_id":"S","owner_id":"ALICE","members":[
                {"identity_id":"ALICE","role":"owner","joined_at":"t1"},
                {"identity_id":"BOB","role":"member","joined_at":"t2"}]}),
        )
        .unwrap();
        let b = MembershipProjection::from_members_data(
            "b",
            &json!({"space_id":"S","owner_id":"ALICE","members":[
                {"identity_id":"BOB","role":"member","joined_at":"DIFFERENT"},
                {"identity_id":"ALICE","role":"owner","joined_at":"ALSO_DIFFERENT"}]}),
        )
        .unwrap();
        assert!(a.converges_with(&b), "membership should converge ignoring joined_at + order");
    }

    #[test]
    fn membership_divergence_detected() {
        let a = MembershipProjection::from_members_data(
            "a",
            &json!({"space_id":"S","owner_id":"ALICE","members":[
                {"identity_id":"ALICE","role":"owner"},{"identity_id":"BOB","role":"member"}]}),
        )
        .unwrap();
        let b = MembershipProjection::from_members_data(
            "b",
            &json!({"space_id":"S","owner_id":"ALICE","members":[
                {"identity_id":"ALICE","role":"owner"}]}),
        )
        .unwrap();
        assert!(!a.converges_with(&b), "missing Bob on b must diverge");
    }

    #[test]
    fn convergence_verdict_passes_on_matching_nodes() {
        let pa = MembershipProjection::from_members_data(
            "a",
            &json!({"space_id":"S","owner_id":"ALICE","members":[
                {"identity_id":"ALICE","role":"owner"},{"identity_id":"BOB","role":"member"}]}),
        )
        .unwrap();
        let pb = MembershipProjection::from_members_data(
            "b",
            &json!({"space_id":"S","owner_id":"ALICE","members":[
                {"identity_id":"BOB","role":"member"},{"identity_id":"ALICE","role":"owner"}]}),
        )
        .unwrap();
        // Same events, different arrival order on each node.
        let ta = Transcript::from_values("a", &[ev("E1", "membership.invite", "S"), ev("E2", "membership.join", "S")]);
        let tb = Transcript::from_values("b", &[ev("E2", "membership.join", "S"), ev("E1", "membership.invite", "S")]);
        let v = convergence_verdict(&[pa, pb], &[ta, tb], "S");
        assert!(v.pass, "{}", v.detail);
    }

    #[test]
    fn convergence_verdict_fails_on_transcript_set_mismatch() {
        let pa = MembershipProjection::from_members_data(
            "a",
            &json!({"space_id":"S","owner_id":"ALICE","members":[{"identity_id":"ALICE","role":"owner"}]}),
        )
        .unwrap();
        let pb = pa.clone();
        let ta = Transcript::from_values("a", &[ev("E1", "x", "S"), ev("E2", "y", "S")]);
        let tb = Transcript::from_values("b", &[ev("E1", "x", "S")]); // missing E2
        let v = convergence_verdict(&[pa, pb], &[ta, tb], "S");
        assert!(!v.pass);
        assert!(v.detail.contains("E2"), "detail: {}", v.detail);
    }

    fn members_ab() -> (MembershipProjection, MembershipProjection) {
        let data = json!({"space_id":"S","owner_id":"ALICE","members":[
            {"identity_id":"ALICE","role":"owner"},{"identity_id":"BOB","role":"member"}]});
        (
            MembershipProjection::from_members_data("a", &data).unwrap(),
            MembershipProjection::from_members_data("b", &data).unwrap(),
        )
    }

    #[test]
    fn cooperative_event_ids_excludes_federation_add() {
        let t = Transcript::from_values(
            "b",
            &[
                ev("E1", "membership.join", "S"),
                ev("FED", "state.federation_add", "S"),
            ],
        );
        // Raw set has both; cooperative set drops the infra event.
        assert!(t.event_ids_for_space("S").contains("FED"));
        let coop = t.cooperative_event_ids_for_space("S");
        assert!(coop.contains("E1"));
        assert!(!coop.contains("FED"), "federation_add must be excluded: {coop:?}");
    }

    #[test]
    fn federation_add_a0_b1_distribution_converges() {
        // MP-R1-D7 directional shape: A's Space DAG holds NO state.federation_add
        // (registry-only add-peer, M9.2-D2′); B materializes exactly ONE on
        // receipt. The cooperative content is identical, so excluding the infra
        // kind makes the Space converge. This test ENCODES the A:0/B:1 intent: if
        // a future change made A also emit it (A:1/B:1) or B stop (A:0/B:0), the
        // Space would still converge — the asymmetry is benign by design, and the
        // exclusion's purpose is documented here even though it would still pass.
        let (pa, pb) = members_ab();
        let ta = Transcript::from_values(
            "a",
            &[ev("E1", "membership.invite", "S"), ev("E2", "membership.join", "S")],
        );
        let tb = Transcript::from_values(
            "b",
            &[
                ev("E1", "membership.invite", "S"),
                ev("E2", "membership.join", "S"),
                ev("FED", "state.federation_add", "S"), // B:1, A:0 — the directional infra event
            ],
        );
        // Sanity: the RAW sets diverge exactly on FED (what we observed live)…
        assert_ne!(
            ta.event_ids_for_space("S"),
            tb.event_ids_for_space("S"),
            "raw sets must diverge on the federation_add (the live symptom)"
        );
        // …and the cooperative verdict converges (FED excluded).
        let v = convergence_verdict(&[pa, pb], &[ta, tb], "S");
        assert!(v.pass, "A:0/B:1 federation_add must converge once excluded: {}", v.detail);
    }

    #[test]
    fn extra_cooperative_event_still_diverges() {
        // Guard: excluding infra must NOT mask a real cooperative divergence — an
        // extra non-infra event on one node still fails convergence.
        let (pa, pb) = members_ab();
        let ta = Transcript::from_values("a", &[ev("E1", "membership.join", "S")]);
        let tb = Transcript::from_values(
            "b",
            &[
                ev("E1", "membership.join", "S"),
                ev("FED", "state.federation_add", "S"), // benign infra — excluded
                ev("E2", "message.text", "S"),          // real cooperative event A lacks
            ],
        );
        let v = convergence_verdict(&[pa, pb], &[ta, tb], "S");
        assert!(!v.pass, "a missing cooperative event E2 must diverge");
        assert!(v.detail.contains("E2"), "divergence should name E2, not FED: {}", v.detail);
    }

    #[test]
    fn per_node_projection_ignores_mid_leave_actor() {
        // MP-R3-D4d: node `a` had two actors — a stable reader (projected) and a
        // churning actor that was mid-leave at sample time (returned NO view, so
        // it is simply absent from `projections`). node `b` has one reader. The
        // per-node collapse yields exactly 2 node projections (a, b), so the
        // ≥2-projection precondition holds on the node count, and the verdict
        // converges — the absent churner does not break it.
        let a_stable = MembershipProjection::from_members_data(
            "a-stable-view",
            &json!({"space_id":"S","owner_id":"ALICE","members":[
                {"identity_id":"ALICE","role":"owner"},{"identity_id":"BOB","role":"member"}]}),
        )
        .unwrap()
        .with_node_label("a");
        let b_reader = MembershipProjection::from_members_data(
            "b-view",
            &json!({"space_id":"S","owner_id":"ALICE","members":[
                {"identity_id":"BOB","role":"member"},{"identity_id":"ALICE","role":"owner"}]}),
        )
        .unwrap()
        .with_node_label("b");
        // The mid-leave churner on `a` is NOT in the list (returned no view).
        let collapsed = per_node_projections(&[a_stable.clone(), b_reader.clone()]);
        assert_eq!(collapsed.len(), 2, "one projection per node (a, b)");
        // And a SECOND stable reader on `a` (e.g. another non-churning client)
        // collapses to one — still 2 node projections, not 3.
        let a_stable2 = a_stable.clone();
        let collapsed2 = per_node_projections(&[a_stable, a_stable2, b_reader.clone()]);
        assert_eq!(collapsed2.len(), 2, "two readers on node a collapse to one");

        let t = vec![
            Transcript::from_values("a", &[ev("E1", "membership.join", "S")]),
            Transcript::from_values("b", &[ev("E1", "membership.join", "S")]),
        ];
        let v = node_convergence_verdict(&collapsed2, &t, "S");
        assert!(v.pass, "per-node convergence holds despite the mid-leave churner: {}", v.detail);
    }

    #[test]
    fn per_actor_oracle_unchanged_for_non_churn() {
        // MP-R3-D4d: untagged (node_label = None) projections — the R1/R2 per-actor
        // path — pass through per_node_projections unchanged, and the default
        // convergence_verdict behaves exactly as before.
        let (pa, pb) = members_ab();
        assert!(pa.node_label.is_none() && pb.node_label.is_none());
        let passthrough = per_node_projections(&[pa.clone(), pb.clone()]);
        assert_eq!(passthrough.len(), 2, "untagged projections pass through");
        let t = vec![
            Transcript::from_values("a", &[ev("E1", "membership.join", "S")]),
            Transcript::from_values("b", &[ev("E1", "membership.join", "S")]),
        ];
        // Same verdict via both the default and the per-node wrapper.
        assert!(convergence_verdict(&[pa.clone(), pb.clone()], &t, "S").pass);
        assert!(node_convergence_verdict(&[pa, pb], &t, "S").pass);
    }

    #[test]
    fn rejection_verdict_passes_when_offender_absent() {
        let ta = Transcript::from_values("a", &[ev("GOOD", "x", "S")]);
        let tb = Transcript::from_values("b", &[ev("GOOD", "x", "S")]);
        let v = rejection_verdict(&[ta, tb], "FORGED");
        assert!(v.pass, "{}", v.detail);
    }

    #[test]
    fn rejection_verdict_fails_when_offender_applied() {
        let ta = Transcript::from_values("a", &[ev("FORGED", "x", "S")]);
        let v = rejection_verdict(&[ta], "FORGED");
        assert!(!v.pass);
        assert!(v.detail.contains("FORGED"));
    }

    fn run_with(id: &str, reply_line: &str) -> ActorRun {
        ActorRun {
            actor: "mallory".to_string(),
            replies: vec![(
                Some(id.to_string()),
                crate::wire::Reply::from_line(reply_line).unwrap(),
            )],
        }
    }

    const REJECT: &str = r#"{"status":"error","cmd":"join","id":"m1","error":{"code":"3044","category":"protocol","message":"invite_expired","instance_state":"ready"}}"#;

    #[test]
    fn rejection_category_passes_on_expected_code_and_category() {
        let run = run_with("m1", REJECT);
        let v = rejection_category_verdict(&run, "m1", "3044", Some("protocol"));
        assert!(v.pass, "{}", v.detail);
        // Code-only (no category) also passes.
        assert!(rejection_category_verdict(&run, "m1", "3044", None).pass);
    }

    #[test]
    fn rejection_category_fails_on_wrong_code() {
        let run = run_with("m1", REJECT);
        let v = rejection_category_verdict(&run, "m1", "3045", None);
        assert!(!v.pass);
        assert!(v.detail.contains("3044") && v.detail.contains("3045"), "{}", v.detail);
    }

    #[test]
    fn rejection_category_fails_on_wrong_category() {
        let run = run_with("m1", REJECT);
        let v = rejection_category_verdict(&run, "m1", "3044", Some("argument"));
        assert!(!v.pass);
        assert!(v.detail.contains("argument"), "{}", v.detail);
    }

    #[test]
    fn rejection_category_fails_when_command_succeeded() {
        let run = run_with(
            "m1",
            r#"{"status":"ok","cmd":"join","id":"m1","data":{"space_id":"S"}}"#,
        );
        let v = rejection_category_verdict(&run, "m1", "3044", None);
        assert!(!v.pass);
        assert!(v.detail.contains("SUCCEEDED"), "{}", v.detail);
    }

    #[test]
    fn rejection_category_fails_when_command_absent() {
        let run = run_with("m1", REJECT);
        let v = rejection_category_verdict(&run, "nope", "3044", None);
        assert!(!v.pass);
        assert!(v.detail.contains("no reply"), "{}", v.detail);
    }
}
