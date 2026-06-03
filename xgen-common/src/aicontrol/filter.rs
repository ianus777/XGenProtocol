// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M7-events arc — the subscription `Filter`, its `parse`, and the shared pure
//! `matches` predicate (EV-D4, v1.1; AC-D3b grammar).
//!
//! Pure substrate (D-065): no transport, no pipe, no runtime. The events-pipe
//! servers (C4 node / C5 client) parse the `subscribe` payload into a [`Filter`]
//! and apply [`matches`] — the **single source of truth** for the grammar
//! (D-067), used by both the node (filter-before-send) and the client
//! (filter-at-drain).
//!
//! **Three dimensions, AND-across / OR-within (AC-D3b):**
//! - `spaces` — empty == all; else the event's effective Space ∈ `spaces`.
//! - `event_types` — empty == all; else the event type matches one entry
//!   (two wildcard forms: bare `*` and a trailing `<family>.*`).
//! - `nodes` — empty == all; else `filter.nodes ∩ event_nodes ≠ ∅`.
//!
//! **The `nodes` source is caller-supplied (EV-D4 v1.1).** A pure
//! `matches(&Filter, &Event)` cannot honor the `nodes` dimension: an `Event`
//! carries no uniform node field — node-authored events put the node in
//! `sender`, `state.federation_add` in `content["node_id"]`, and every other
//! event's node association is the Space's `home_node`, which is *runtime
//! state*, not event data (EV-D5's "provenance + runtime state"). So the
//! predicate takes `event_nodes: &[NodeXgid]`, the set of nodes the event
//! involves, which the **caller** derives: the C3 node side from runtime
//! (`SpaceState.home_node` + `federation_nodes` + sender-if-Node +
//! `content["node_id"]`); the client passes `&[]` (and rejects `nodes` filters
//! at the C5 call site, so the arm is vacuously "all" there). This keeps one
//! pure shared predicate while honoring the runtime-sourced dimension.
//!
//! **Entitlement is the ceiling, never an escalation vector:** out-of-entitlement
//! `spaces`/`nodes` entries are *inert* (match nothing, never error). Only a
//! structurally malformed filter (unknown field, wrong type, illegal wildcard
//! form, unknown exact type) is a `BAD_ARGUMENT` — raised by [`parse`] before
//! streaming, the `subscribe` being the first message.

use serde::Deserialize;

use super::codes::{ControlCode, ControlError};
use crate::wire::{Event, EventType};
use crate::xgid::{NodeXgid, SpaceXgid};

/// A parsed subscription filter (AC-D3b). All three dimensions default to empty
/// (== no restriction). `deny_unknown_fields` makes a stray key a `BAD_ARGUMENT`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filter {
    /// Restrict to these Spaces (empty == all entitled Spaces).
    #[serde(default)]
    pub spaces: Vec<SpaceXgid>,
    /// Restrict to these event types (empty == all). Each entry is `*`, a
    /// `<family>.*` wildcard, or an exact known `EventType` wire string.
    #[serde(default)]
    pub event_types: Vec<String>,
    /// Restrict to events involving these nodes (empty == all). Node-side only;
    /// the client rejects a non-empty `nodes` at its call site (C5).
    #[serde(default)]
    pub nodes: Vec<NodeXgid>,
}

/// Parse a `subscribe` filter payload, validating it fully before any streaming
/// starts. Errors map to the substrate `BAD_ARGUMENT` control code:
/// - unknown field / wrong type → serde (`deny_unknown_fields` + typing),
/// - illegal wildcard form or unknown exact event type → the token check.
pub fn parse(payload: serde_json::Value) -> Result<Filter, ControlError> {
    let filter: Filter = serde_json::from_value(payload).map_err(|e| {
        ControlError::new(
            ControlCode::BadArgument,
            format!("malformed subscribe filter: {e}"),
        )
    })?;
    for entry in &filter.event_types {
        validate_event_type_token(entry)?;
    }
    Ok(filter)
}

/// Validate one `event_types` token (AC-D3b closed grammar). Legal forms:
/// - bare `*` (all types),
/// - `<family>.*` — exactly one family segment (`[a-z_]+`) then `.*`,
/// - an exact string that is a known [`EventType`] (**fail-closed**: an
///   unknown well-formed exact like `state` or `message.foo` is rejected).
///
/// Everything else (`*.text`, `state.space_*`, `state.*.foo`, `mess*`, `""`)
/// → `BAD_ARGUMENT`. The wildcard's *form* is validated, not its family
/// membership — `foo.*` is well-formed and accepted-but-inert (matches no
/// real type), while unknown *exact* entries fail closed.
fn validate_event_type_token(entry: &str) -> Result<(), ControlError> {
    if entry == "*" {
        return Ok(());
    }
    if let Some(prefix) = entry.strip_suffix(".*") {
        // A single family segment: non-empty, only `[a-z_]` (no embedded dot,
        // no other wildcard) — this is exactly `^[a-z_]+\.\*$`.
        if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
            return Ok(());
        }
        return Err(ControlError::new(
            ControlCode::BadArgument,
            format!("illegal wildcard form {entry:?} — only `*` or `<family>.*` are allowed"),
        ));
    }
    if EventType::from_str(entry).is_some() {
        return Ok(());
    }
    Err(ControlError::new(
        ControlCode::BadArgument,
        format!("unknown or malformed event type {entry:?}"),
    ))
}

/// The shared pure subscription predicate (EV-D4 v1.1, D-067 single source of
/// truth). Returns `true` iff the event passes **every present** dimension
/// (AND-across), each dimension matching when **any** of its entries match
/// (OR-within). An empty dimension imposes no restriction.
///
/// `event_nodes` is the set of nodes the event involves, supplied by the caller
/// (see the module docs). The client passes `&[]`.
pub fn matches(filter: &Filter, event: &Event, event_nodes: &[NodeXgid]) -> bool {
    // spaces — empty == all; else the event's effective Space ∈ spaces.
    if !filter.spaces.is_empty() {
        match event.effective_space_id() {
            Some(sid) => {
                if !filter.spaces.contains(&sid) {
                    return false;
                }
            }
            // No resolvable Space → cannot satisfy a restricted spaces set.
            None => return false,
        }
    }

    // event_types — empty == all; OR-within.
    if !filter.event_types.is_empty() {
        let ty = event.event_type.as_str();
        if !filter.event_types.iter().any(|entry| event_type_matches(entry, ty)) {
            return false;
        }
    }

    // nodes — empty == all; else filter.nodes ∩ event_nodes ≠ ∅.
    if !filter.nodes.is_empty() && !filter.nodes.iter().any(|n| event_nodes.contains(n)) {
        return false;
    }

    true
}

/// Apply one `event_types` token to an event type string. `*` matches all;
/// `<family>.*` strips only the `*` (keeping the `.`) and prefix-matches, so the
/// segment boundary is respected against the uniform `family.suffix` strings;
/// an exact entry compares equal. (Assumes the token passed [`parse`] —
/// `matches` is total, so a hand-built illegal token simply fails to match.)
fn event_type_matches(entry: &str, ty: &str) -> bool {
    if entry == "*" {
        return true;
    }
    if entry.ends_with(".*") {
        // "state.*" -> prefix "state." (keep the dot — segment boundary).
        return ty.starts_with(&entry[..entry.len() - 1]);
    }
    ty == entry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xgid::{IdentityXgid, RoomXgid, Xgid};
    use serde_json::json;

    fn sx(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn nx(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn idx(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }

    /// An event of `ty` in `space` (raw space_id) with a fixed event_id.
    fn ev(ty: EventType, space: &str) -> Event {
        Event {
            protocol_version: "0.1".to_string(),
            event_type: ty,
            event_id: Some(crate::xgid::EventXgid::from_xgid(Xgid::new("xgen://hash/sha256:EV".to_string()))),
            sender: idx("xgen://pubkey/ed25519:ALICE"),
            room_id: RoomXgid::from_xgid(Xgid::new(String::new())),
            space_id: sx(space),
            prev_events: vec![],
            timestamp: "2026-06-01T00:00:00.000Z".to_string(),
            content: json!({}),
            meta_atts: None,
            signature: Some("ed25519:STUB:STUB".to_string()),
        }
    }

    const SPACE_A: &str = "xgen://hash/sha256:SPACEA";
    const SPACE_B: &str = "xgen://hash/sha256:SPACEB";
    const NODE_X: &str = "xgen://pubkey/ed25519:NODEX";
    const NODE_Y: &str = "xgen://pubkey/ed25519:NODEY";

    // ── empty == all ──────────────────────────────────────────────────────

    #[test]
    fn empty_filter_matches_everything() {
        let f = Filter::default();
        let e = ev(EventType::MessageText, SPACE_A);
        assert!(matches(&f, &e, &[]));
        // even with node provenance present, an empty nodes arm imposes nothing.
        assert!(matches(&f, &e, &[nx(NODE_X)]));
    }

    // ── spaces ────────────────────────────────────────────────────────────

    #[test]
    fn spaces_in_set_matches_out_of_set_does_not() {
        let f = parse(json!({ "spaces": [SPACE_A] })).unwrap();
        assert!(matches(&f, &ev(EventType::MessageText, SPACE_A), &[]));
        assert!(!matches(&f, &ev(EventType::MessageText, SPACE_B), &[]));
    }

    #[test]
    fn spaces_out_of_entitlement_is_inert_not_error() {
        // A Space the subscriber is not in is simply never matched — never an error.
        let f = parse(json!({ "spaces": ["xgen://hash/sha256:NOTMINE"] })).unwrap();
        assert!(!matches(&f, &ev(EventType::MessageText, SPACE_A), &[]));
    }

    #[test]
    fn spaces_arm_resolves_create_event_to_its_event_id() {
        // A state.space_create carries an empty space_id and IS the Space
        // (event_id). A spaces:[<event_id>] filter must see it.
        let mut create = ev(EventType::StateSpaceCreate, "");
        let eid = "xgen://hash/sha256:NEWSPACE";
        create.space_id = SpaceXgid::from_xgid(Xgid::new(String::new()));
        create.event_id = Some(crate::xgid::EventXgid::from_xgid(Xgid::new(eid.to_string())));
        assert_eq!(create.effective_space_id().unwrap().as_str(), eid);

        let f = parse(json!({ "spaces": [eid] })).unwrap();
        assert!(matches(&f, &create, &[]), "spaces filter must see the create event's own Space");
        // A filter for a different Space does not match it.
        let f2 = parse(json!({ "spaces": [SPACE_A] })).unwrap();
        assert!(!matches(&f2, &create, &[]));
    }

    // ── event_types: exact, wildcards, OR-within ──────────────────────────

    #[test]
    fn event_types_exact_match_and_non_match() {
        let f = parse(json!({ "event_types": ["message.text"] })).unwrap();
        assert!(matches(&f, &ev(EventType::MessageText, SPACE_A), &[]));
        assert!(!matches(&f, &ev(EventType::StateRoomCreate, SPACE_A), &[]));
    }

    #[test]
    fn event_types_bare_star_matches_all() {
        let f = parse(json!({ "event_types": ["*"] })).unwrap();
        assert!(matches(&f, &ev(EventType::MessageText, SPACE_A), &[]));
        assert!(matches(&f, &ev(EventType::StateRoomCreate, SPACE_A), &[]));
    }

    #[test]
    fn event_types_family_wildcard_respects_segment_boundary() {
        let f = parse(json!({ "event_types": ["state.*"] })).unwrap();
        assert!(matches(&f, &ev(EventType::StateRoomCreate, SPACE_A), &[]));
        assert!(matches(&f, &ev(EventType::StateSpaceCreate, SPACE_A), &[]));
        // message.* must NOT be caught by state.*
        assert!(!matches(&f, &ev(EventType::MessageText, SPACE_A), &[]));
    }

    #[test]
    fn event_types_or_within() {
        let f = parse(json!({ "event_types": ["message.text", "membership.join"] })).unwrap();
        assert!(matches(&f, &ev(EventType::MessageText, SPACE_A), &[]));
        assert!(matches(&f, &ev(EventType::MembershipJoin, SPACE_A), &[]));
        assert!(!matches(&f, &ev(EventType::StateRoomCreate, SPACE_A), &[]));
    }

    #[test]
    fn event_types_unknown_event_filter_semantics() {
        // PG-09 / FC-D5. An unknown event type matches a `*` filter and a
        // matching `<family>.*` wildcard (via its raw stored type string), but
        // not a named known-type filter; and the exact unknown type cannot be
        // named in a filter at all — `parse` fails closed (BAD_ARGUMENT), the
        // from_str-strict half of Shape A that keeps subscriptions fail-closed.
        let unknown = ev(EventType::Unknown("com.acme.widget".to_string()), SPACE_A);

        // `*` matches anything, including unknown.
        assert!(matches(
            &parse(json!({ "event_types": ["*"] })).unwrap(),
            &unknown,
            &[]
        ));
        // A family wildcard matching the unknown type's family matches it...
        assert!(matches(
            &parse(json!({ "event_types": ["com.*"] })).unwrap(),
            &unknown,
            &[]
        ));
        // ...one for a different family does not.
        assert!(!matches(
            &parse(json!({ "event_types": ["state.*"] })).unwrap(),
            &unknown,
            &[]
        ));
        // A named known-type filter never matches an unknown event.
        assert!(!matches(
            &parse(json!({ "event_types": ["message.text"] })).unwrap(),
            &unknown,
            &[]
        ));
        // The exact unknown type cannot be named — parse is fail-closed.
        assert!(parse(json!({ "event_types": ["com.acme.widget"] })).is_err());
    }

    // ── nodes (Option 3 — caller-supplied event_nodes) ────────────────────

    #[test]
    fn nodes_empty_matches_all() {
        let f = parse(json!({})).unwrap();
        assert!(matches(&f, &ev(EventType::MessageText, SPACE_A), &[nx(NODE_X)]));
        assert!(matches(&f, &ev(EventType::MessageText, SPACE_A), &[]));
    }

    #[test]
    fn nodes_intersection_non_empty_matches() {
        let f = parse(json!({ "nodes": [NODE_X] })).unwrap();
        // event involves NODE_X → match.
        assert!(matches(&f, &ev(EventType::MessageText, SPACE_A), &[nx(NODE_X)]));
        // event involves only NODE_Y → no intersection → no match.
        assert!(!matches(&f, &ev(EventType::MessageText, SPACE_A), &[nx(NODE_Y)]));
        // event involves no node → no match against a restricted nodes set.
        assert!(!matches(&f, &ev(EventType::MessageText, SPACE_A), &[]));
    }

    // ── AND-across the three dimensions ───────────────────────────────────

    #[test]
    fn and_across_dimensions() {
        let f = parse(json!({
            "spaces": [SPACE_A],
            "event_types": ["message.text"],
            "nodes": [NODE_X],
        }))
        .unwrap();
        // all three hold → match.
        assert!(matches(&f, &ev(EventType::MessageText, SPACE_A), &[nx(NODE_X)]));
        // wrong space → fail.
        assert!(!matches(&f, &ev(EventType::MessageText, SPACE_B), &[nx(NODE_X)]));
        // wrong type → fail.
        assert!(!matches(&f, &ev(EventType::StateRoomCreate, SPACE_A), &[nx(NODE_X)]));
        // wrong node → fail.
        assert!(!matches(&f, &ev(EventType::MessageText, SPACE_A), &[nx(NODE_Y)]));
    }

    // ── parse: malformed → BAD_ARGUMENT ───────────────────────────────────

    fn assert_bad_arg(payload: serde_json::Value) {
        match parse(payload) {
            Err(e) => assert_eq!(e.code, ControlCode::BadArgument),
            Ok(f) => panic!("expected BAD_ARGUMENT, parsed {f:?}"),
        }
    }

    #[test]
    fn parse_rejects_illegal_wildcards() {
        assert_bad_arg(json!({ "event_types": ["*.text"] }));
        assert_bad_arg(json!({ "event_types": ["state.space_*"] }));
        assert_bad_arg(json!({ "event_types": ["state.*.foo"] }));
        assert_bad_arg(json!({ "event_types": ["mess*"] }));
    }

    #[test]
    fn parse_rejects_unknown_exact_fail_closed() {
        assert_bad_arg(json!({ "event_types": ["state"] })); // bare family, not a type
        assert_bad_arg(json!({ "event_types": ["message.foo"] })); // well-formed but unknown
        assert_bad_arg(json!({ "event_types": [""] })); // empty token
    }

    #[test]
    fn parse_rejects_unknown_field_and_wrong_type() {
        assert_bad_arg(json!({ "space": [SPACE_A] })); // unknown field (deny_unknown_fields)
        assert_bad_arg(json!({ "event_types": "message.text" })); // wrong type (string, not array)
    }

    #[test]
    fn parse_accepts_legal_forms() {
        let f = parse(json!({
            "spaces": [SPACE_A],
            "event_types": ["*", "state.*", "message.text"],
            "nodes": [NODE_X],
        }))
        .unwrap();
        assert_eq!(f.spaces.len(), 1);
        assert_eq!(f.event_types.len(), 3);
        assert_eq!(f.nodes.len(), 1);
    }

    #[test]
    fn parse_empty_payload_is_all_pass() {
        let f = parse(json!({})).unwrap();
        assert!(f.spaces.is_empty() && f.event_types.is_empty() && f.nodes.is_empty());
    }
}
