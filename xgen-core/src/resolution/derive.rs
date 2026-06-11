// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Resolving derivation — turn a Space's Event log into a convergent SpaceState
// snapshot (spec 3.9.2 / 3.9.7).
//
// This is the M8 (state-resolution convergence) core: the driver that wires the
// already-built seven-layer `resolve()` onto the apply path. The algorithm
// itself (`resolution/algorithm.rs`) is NOT redesigned here — this module only
// orchestrates `topological_sort` + `find_conflicts` + `resolve` + the per-event
// `SpaceState::apply_event` into the §3.9.2 statement:
//
//     snapshot ≡ fold(resolve(causal_sort(log)))
//
// At C1 (this commit) `derive_resolved` has **no production caller** — it sits
// usable and proven, exactly the way `resolve` sat unused before M8. C2 wires it
// into `node/runtime.rs` (`rehydrate_space_from_store` + the `ingest_event`
// conflict gate). Client apply sites stay untouched (SR-D3).
//
// ── CP-A (auth basis during rebuild) — LOCKED: unconflicted-state basis ──────
//
// `resolve()` consults the SpaceState snapshot for Layer 4 (role), Layer 5a
// (node priority) and Layer 5b (federation recency). During a rebuild there is
// no "current" snapshot to consult — the snapshot is what we are computing. The
// v1 rule (Matrix state-res-v2's shape) breaks the circularity by deriving an
// **unconflicted-state basis** first: fold only the events whose state key has
// *no* conflict, then resolve each conflict set against that fixed basis, then
// fold the winners + unconflicted events into the final snapshot. The basis can
// never depend on a conflict outcome, so resolution is well-founded.
//
// Whether the basis *choice* (unconflicted vs current-best-effort snapshot) is
// load-bearing is an empirical question the M8 membership-conflict tests answer.
// Finding (this commit): it is **not** load-bearing for the membership core.
// Layer 1 (ban>join) reads no snapshot state at all; Layer 5c (lexicographic)
// reads none either; Layer 4 (role) reads member roles, but in every M8
// scenario those roles are fixed by *unconflicted* invite/join events, so the
// unconflicted basis and a best-effort snapshot agree on them — and therefore on
// the winner. No scenario resolves a membership conflict against state that is
// itself conflicted. So **SR-D7 is NOT promoted** (per the pickup conditional:
// "promote to SR-D7 only if the membership tests show it's load-bearing"). The
// unconflicted basis stays the v1 rule on first-principles grounds (it is the
// only circularity-free choice), recorded here rather than locked as a decision.

use std::collections::{HashMap, HashSet};

use crate::{
    node::runtime::topological_sort,
    space::state::SpaceState,
    wire::types::{Event, EventType},
};

use super::{algorithm::resolve, conflict::find_conflicts, state_key::state_key_for_event};

/// Derive the convergent `SpaceState` for a Space from its full Event log.
///
/// Order-independent by construction: the log is topologically sorted (causal
/// order, deterministic lexicographic tie-break) before anything else, so any
/// permutation of `events` yields the same result. Conflicting Events that
/// *lose* resolution are skipped (their state mutation is not applied) but are
/// **not** removed from the caller's DAG — the loser stays in the log
/// permanently (§3.9.7); this function only decides which winners shape the
/// snapshot.
///
/// Returns `None` when `events` contains no Space-create root (mirrors
/// `NodeRuntime::rehydrate_space_from_store`'s early return — there is no Space
/// to build a state for).
///
/// # Arguments
/// * `events` — the Space's full Event log, in any order.
/// * `my_node_id` — this Node's id, threaded to `apply_event` for the D-075
///   vantage-aware `apply_federation_add`.
/// * `identity_home_nodes` — identity_id → home_node_id map for resolution
///   Layers 3 / 5a / 5b. C2 sources this from the identity registry.
pub fn derive_resolved(
    events: Vec<Event>,
    my_node_id: &str,
    identity_home_nodes: &HashMap<String, String>,
) -> Option<SpaceState> {
    // Causal order first — this is what makes the whole pipeline order-independent.
    let sorted = topological_sort(events);

    // Transitive causal ancestry, computed over the sorted log. Needed because a
    // true conflict (spec §3.9.1) is "same state key AND no causal ordering" —
    // but `find_conflicts` only groups by state key (its own causal check,
    // `conflicts_with`, is the incremental-gate path and is direct-parent only).
    // A `membership.invite X → membership.join X` chain shares a state key yet is
    // causally ordered: NOT a conflict, both must apply (the join consumes the
    // invite's role). So we restrict each key-group to its causal *frontier*
    // before resolving. (Unoptimised O(N·ancestors) closure — this only runs on
    // the rebuild path, where conflicts are rare; incremental is deferred per
    // SR-D2.)
    let ancestors = build_ancestors(&sorted);

    // Group by state key, then keep only the frontier of each group — the
    // events with no same-key descendant within the group. Frontier events are
    // pairwise non-ancestor ⇒ mutually concurrent ⇒ a genuine conflict set. A
    // frontier of size 1 means the key resolves by plain causal succession (no
    // conflict) and is left to fold normally.
    let conflict_sets: Vec<Vec<Event>> = find_conflicts(&sorted)
        .iter()
        .filter_map(|(_key, group)| {
            let front = frontier_of(group, &ancestors);
            if front.len() >= 2 {
                Some(front.iter().map(|e| (*e).clone()).collect())
            } else {
                None
            }
        })
        .collect();

    // Every frontier-conflict event — winners and losers alike — is excluded
    // from the unconflicted-state basis (CP-A). Non-frontier same-key events are
    // NOT contested and stay in the basis (they apply normally).
    let conflicted_ids: HashSet<String> = conflict_sets
        .iter()
        .flat_map(|set| set.iter().filter_map(event_id_owned))
        .collect();

    // Phase 1 — the unconflicted-state basis: fold every Event whose state key
    // is NOT contested. `resolve()` reads roles / node-priority / federation
    // recency from this fixed snapshot.
    let basis = fold_skipping(&sorted, &conflicted_ids, my_node_id)?;

    // Phase 2 — resolve each concurrent conflict set against the basis; collect
    // the losers.
    let mut losers: HashSet<String> = HashSet::new();
    for set in &conflict_sets {
        // `resolve` only errors on an empty set, which a frontier of size >= 2
        // never is; on the (unreachable) Err we leave the set intact (no losers
        // recorded) rather than panic.
        if let Ok(winner) = resolve(set, &basis, identity_home_nodes) {
            let winner_id = event_id_owned(winner);
            for ev in set {
                let id = event_id_owned(ev);
                if id != winner_id {
                    if let Some(id) = id {
                        losers.insert(id);
                    }
                }
            }
        }
    }

    // Phase 3 — fold the resolved log: every Event except the losers. Winners
    // and unconflicted Events apply in causal order; losers are skipped.
    fold_skipping(&sorted, &losers, my_node_id)
}

/// Transitive causal ancestor set per event_id, built over a topologically
/// sorted slice (ancestors precede descendants, so each event's parents are
/// already resolved). `ancestors[id]` contains every event_id reachable by
/// following `prev_events` transitively.
fn build_ancestors(sorted: &[Event]) -> HashMap<String, HashSet<String>> {
    let mut ancestors: HashMap<String, HashSet<String>> = HashMap::new();
    for ev in sorted {
        let id = match event_id_owned(ev) {
            Some(i) => i,
            None => continue,
        };
        let mut set: HashSet<String> = HashSet::new();
        for prev in &ev.prev_events {
            let pid = prev.as_str().to_string();
            if let Some(prev_anc) = ancestors.get(&pid) {
                set.extend(prev_anc.iter().cloned());
            }
            set.insert(pid);
        }
        ancestors.insert(id, set);
    }
    ancestors
}

/// The causal frontier of a same-key group: events that are NOT a transitive
/// ancestor of any other event in the group. Frontier events are mutually
/// concurrent (none precedes another) and therefore form the genuine conflict
/// set per spec §3.9.1.
fn frontier_of<'a>(
    group: &[&'a Event],
    ancestors: &HashMap<String, HashSet<String>>,
) -> Vec<&'a Event> {
    group
        .iter()
        .copied()
        .filter(|e| {
            let eid = match event_id_owned(e) {
                Some(i) => i,
                None => return false,
            };
            // `e` is on the frontier iff no other group member `f` has `e` among
            // its transitive ancestors.
            !group.iter().any(|f| {
                f.event_id.as_ref().map(|x| x.as_str()) != Some(eid.as_str())
                    && f.event_id
                        .as_ref()
                        .and_then(|fid| ancestors.get(fid.as_str()))
                        .is_some_and(|anc| anc.contains(&eid))
            })
        })
        .collect()
}

/// Fold a causally-sorted Event slice into a `SpaceState`, seeding from the
/// (topologically first) Space-create Event and applying every subsequent Event
/// whose id is NOT in `skip`. Returns `None` if no create Event is present.
///
/// Mirrors `NodeRuntime::rehydrate_space_from_store`'s create-dispatch-then-apply
/// shape — the single difference is the `skip` set.
fn fold_skipping(
    sorted: &[Event],
    skip: &HashSet<String>,
    my_node_id: &str,
) -> Option<SpaceState> {
    let mut state: Option<SpaceState> = None;
    for ev in sorted {
        match &ev.event_type {
            EventType::StateSpaceCreate => {
                state = SpaceState::from_space_create(ev).ok();
            }
            EventType::StateDmSpaceCreate => {
                state = SpaceState::from_dm_space_create_node(ev).ok();
            }
            _ => {
                if let Some(id) = ev.event_id.as_ref().map(|e| e.as_str()) {
                    if skip.contains(id) {
                        continue;
                    }
                }
                if let Some(s) = state.as_mut() {
                    let _ = s.apply_event(ev, my_node_id);
                }
            }
        }
    }
    state
}

/// `event.event_id` projected to an owned `String` (the topo-sort/loser-set key
/// shape). `None` for an unsigned event.
fn event_id_owned(event: &Event) -> Option<String> {
    event.event_id.as_ref().map(|e| e.as_str().to_string())
}

/// Conflict gate for the incremental apply path (SR-D1 / CP-E): does `incoming`
/// genuinely conflict with any Event already in `log`?
///
/// A genuine conflict (spec §3.9.1) is "same state key AND no causal ordering."
/// This is **ancestry-aware**: it reuses the same transitive-closure logic as
/// `derive_resolved` (`build_ancestors`), NOT the direct-parent-only
/// `conflict::conflicts_with`. That distinction is load-bearing — a
/// causally-ordered `invite X → join X` chain shares a state key yet is NOT a
/// conflict; keying the gate off `conflicts_with` would false-positive it and
/// drop the join (exactly the C1 bug). Returns `false` for non-state-keyed
/// Events (messages never conflict) and unsigned Events.
///
/// `log` MUST contain `incoming` (the node appends the Event to the store
/// before gating); the transitive-ancestry computation relies on it being
/// present so `incoming`'s own ancestors are resolved.
pub fn conflicts_in_log(incoming: &Event, log: &[Event]) -> bool {
    let key = match state_key_for_event(incoming) {
        Some(k) => k,
        None => return false,
    };
    let inc_id = match event_id_owned(incoming) {
        Some(i) => i,
        None => return false,
    };

    let sorted = topological_sort(log.to_vec());
    let ancestors = build_ancestors(&sorted);

    sorted.iter().any(|ev| {
        let eid = match event_id_owned(ev) {
            Some(i) => i,
            None => return false,
        };
        if eid == inc_id {
            return false;
        }
        if state_key_for_event(ev).as_ref() != Some(&key) {
            return false;
        }
        // Concurrent ⇔ neither Event is a transitive ancestor of the other.
        let inc_after_e = ancestors.get(&inc_id).is_some_and(|s| s.contains(&eid));
        let e_after_inc = ancestors.get(&eid).is_some_and(|s| s.contains(&inc_id));
        !inc_after_e && !e_after_inc
    })
}

// ── Convergence property tests (SR-D5 primary — the §3.9.2 proof) ─────────────
//
// Each scenario constructs a concurrent same-key conflict, then asserts that
// **every arrival permutation** of the log derives a byte-equal SpaceState
// (`PartialEq` is order-independent over the HashMap/HashSet fields), AND that
// the surviving state reflects the winner the targeted resolution layer should
// pick. Together these are the strong-eventual-consistency guarantee: order of
// arrival never changes the converged state.
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use ed25519_dalek::SigningKey;
    use serde_json::{json, Value};
    use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, Xgid};

    use crate::{
        crypto::encoding,
        identity::keypair,
        space::{
            membership::{Effect, Role, RoomPermission},
            state::{
                build_membership_event, build_room_create_event, build_room_update_event,
                build_space_create_event, build_thread_archived_event, build_thread_create_event,
                build_thread_resolved_event, sign_event, thread_id_from_event_id,
            },
        },
        wire::types::{EventType, ThreadStatus},
    };

    const NODE: &str = "xgen://pubkey/ed25519:NODE";

    // ── Fixture builders ──────────────────────────────────────────────────────

    fn kp() -> SigningKey {
        keypair::generate()
    }

    fn id_of(k: &SigningKey) -> String {
        format!("xgen://pubkey/ed25519:{}", encoding::encode(k.verifying_key().as_bytes()))
    }

    fn xid(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }

    fn eid(ev: &Event) -> String {
        ev.event_id.as_ref().expect("signed event has event_id").as_str().to_string()
    }

    /// Signed `state.space_create` rooted at `NODE`, owner = `owner`.
    fn create_space(owner: &SigningKey) -> Event {
        sign_event(build_space_create_event(owner, "Test", None, 1, NODE, None, false), owner)
    }

    /// Signed membership event with explicit `prev_events`.
    fn mem(
        key: &SigningKey,
        space_id: &str,
        etype: EventType,
        content: Value,
        prevs: &[&str],
    ) -> Event {
        let mut ev = build_membership_event(key, space_id, "", etype, content);
        ev.prev_events =
            prevs.iter().map(|p| EventXgid::from_xgid(Xgid::new(p.to_string()))).collect();
        sign_event(ev, key)
    }

    fn empty_ihn() -> HashMap<String, String> {
        HashMap::new()
    }

    /// Full-factorial permutations — used only for small scenarios (n ≤ 5).
    fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
        if items.len() <= 1 {
            return vec![items.to_vec()];
        }
        let mut out = Vec::new();
        for i in 0..items.len() {
            let mut rest = items.to_vec();
            let picked = rest.remove(i);
            for mut p in permutations(&rest) {
                p.insert(0, picked.clone());
                out.push(p);
            }
        }
        out
    }

    /// The §3.9.2 assertion: every arrival permutation derives the identical
    /// SpaceState. Returns the converged state for further per-scenario checks.
    fn assert_converges(
        events: Vec<Event>,
        ihn: &HashMap<String, String>,
    ) -> SpaceState {
        assert!(events.len() <= 5, "full-factorial harness is for small scenarios only");
        let reference =
            derive_resolved(events.clone(), NODE, ihn).expect("scenario has a create event");
        for perm in permutations(&events) {
            let got =
                derive_resolved(perm, NODE, ihn).expect("scenario has a create event");
            assert_eq!(
                got, reference,
                "all arrival permutations must converge to one identical SpaceState (§3.9.2)"
            );
        }
        reference
    }

    /// Independent manual fold (does NOT call `derive_resolved`) — the oracle for
    /// the no-conflict == plain-replay test.
    fn plain_fold(events: &[Event]) -> SpaceState {
        let sorted = topological_sort(events.to_vec());
        let mut state: Option<SpaceState> = None;
        for ev in &sorted {
            match &ev.event_type {
                EventType::StateSpaceCreate => state = SpaceState::from_space_create(ev).ok(),
                EventType::StateDmSpaceCreate => {
                    state = SpaceState::from_dm_space_create_node(ev).ok()
                }
                _ => {
                    if let Some(s) = state.as_mut() {
                        let _ = s.apply_event(ev, NODE);
                    }
                }
            }
        }
        state.expect("create event present")
    }

    // ── Layer 1 — EventType priority (ban > join) ─────────────────────────────

    #[test]
    fn convergence_layer1_ban_beats_concurrent_join() {
        let owner = kp();
        let x = kp();
        let x_id = id_of(&x);

        let create = create_space(&owner);
        let sid = eid(&create);
        // Concurrent: X joins, owner bans X — both reference the create root, so
        // neither is a causal ancestor of the other → conflict on membership:sid:X.
        let join_x = mem(&x, &sid, EventType::MembershipJoin, json!({}), &[&sid]);
        let ban_x = mem(
            &owner,
            &sid,
            EventType::MembershipBan,
            json!({ "target_identity": x_id }),
            &[&sid],
        );

        let state = assert_converges(vec![create, join_x, ban_x], &empty_ihn());

        // Layer 1: ban dominates join regardless of arrival order. (Layer 1 reads
        // no snapshot state — the unconflicted basis is irrelevant here, which is
        // one reason the basis CHOICE is not load-bearing; see CP-A.)
        assert!(state.banned.contains(&xid(&x_id)), "ban must win — X is banned");
        assert!(!state.members.contains_key(&xid(&x_id)), "X must not be a member");
    }

    // ── MP-F7 — leave→rejoin anchoring (the D-076 spine) ──────────────────────
    // A user who leaves then rejoins an open Space must converge to *member*.
    // The rejoin must causally descend from the leave (`prev=[lv]`) so the
    // `membership:{space}:a1` key is a LINEAR chain `j → lv → rj`:
    // `frontier_of` = {rj} → resolve applies the rejoin → a1 is a member, under
    // every arrival ordering. Anchoring the rejoin at the create ROOT instead
    // (the bug: `ops::join`'s `get_dag_tips` fallback fires for a just-left
    // non-member starved of tips by member-gated sync) makes it CONCURRENT with
    // the leave → frontier {lv, rj} → the spec §3.9.3 Layer-1 `leave > join`
    // rule deterministically drops the rejoin → a1 stays non-member (the
    // observed MP-C-11 fault). These two tests are the proof that Fork A's
    // causal *edge* is load-bearing — the MP-F4 A1-falsification guard:
    // key-separation ≠ ordering, the edge is what orders them. (They also show
    // why Fork C is wrong: the Layer-1 leave>join priority is spec'd and correct;
    // the fix removes the concurrency so the rule never fires, it does not change
    // the rule.)

    #[test]
    fn convergence_mp_f7_rejoin_anchored_after_leave_is_member() {
        let owner = kp();
        let a1 = kp();
        let a1_id = id_of(&a1);

        let create = create_space(&owner);
        let sid = eid(&create);
        let join = mem(&a1, &sid, EventType::MembershipJoin, json!({}), &[&sid]);
        let leave = mem(&a1, &sid, EventType::MembershipLeave, json!({}), &[&eid(&join)]);
        // Fork A: the rejoin is anchored AFTER the leave → linear j→lv→rj.
        let rejoin = mem(&a1, &sid, EventType::MembershipJoin, json!({}), &[&eid(&leave)]);

        let state = assert_converges(vec![create, join, leave, rejoin], &empty_ihn());
        assert!(
            state.members.contains_key(&xid(&a1_id)),
            "rejoin anchored after the leave wins (linear j→lv→rj, frontier={{rj}}) — a1 is a member"
        );
    }

    #[test]
    fn convergence_mp_f7_rejoin_anchored_at_root_is_dropped() {
        // RED witness (the bug) — reverting Fork A reproduces exactly this: a
        // root-anchored rejoin is concurrent with the leave → Layer-1 leave>join
        // drops it → a1 non-member. Deterministic (Layer-1 priority, not a
        // lexicographic coin-flip).
        let owner = kp();
        let a1 = kp();
        let a1_id = id_of(&a1);

        let create = create_space(&owner);
        let sid = eid(&create);
        let join = mem(&a1, &sid, EventType::MembershipJoin, json!({}), &[&sid]);
        let leave = mem(&a1, &sid, EventType::MembershipLeave, json!({}), &[&eid(&join)]);
        // The bug: rejoin anchored at the create root → concurrent with the leave.
        let rejoin_root = mem(&a1, &sid, EventType::MembershipJoin, json!({}), &[&sid]);

        let state = assert_converges(vec![create, join, leave, rejoin_root], &empty_ihn());
        assert!(
            !state.members.contains_key(&xid(&a1_id)),
            "root-anchored rejoin is concurrent with the leave and dropped (Layer-1 leave>join) — \
             a1 non-member: the MP-C-11 fault"
        );
    }

    // ── Arc F PG-11 — Space Migration cutover survives a permuted rebuild ──────

    #[test]
    fn convergence_space_migrate_cutover_flips_home_node_under_any_order() {
        // AF-D1/D2 convergence pin: the cutover applier has no state key (it is a
        // causally-terminal singleton), so it folds in causal order regardless of
        // arrival permutation. Every permutation must derive the identical
        // SpaceState — `home_node` flipped to the destination, membership intact.
        use crate::migration::state_machine::build_space_migrate_event;
        let owner = kp();
        let source = kp();
        let bob = kp();
        let source_id = id_of(&source);
        let bob_id = id_of(&bob);

        // Space homed at the SOURCE node, so the source-signed migrate passes the
        // AF-D2 authority gate when the replay reaches it (home_node still source).
        let create = sign_event(
            build_space_create_event(&owner, "Test", None, 1, &source_id, None, false),
            &owner,
        );
        let sid = eid(&create);
        let invite = mem(
            &owner,
            &sid,
            EventType::MembershipInvite,
            json!({ "target_identity": bob_id, "role": "member" }),
            &[&sid],
        );
        let invite_id = eid(&invite);
        let join = mem(&bob, &sid, EventType::MembershipJoin, json!({}), &[&invite_id]);
        let join_id = eid(&join);

        let dst = "xgen://pubkey/ed25519:DEST1";
        let migrate = build_space_migrate_event(
            &source,
            &sid,
            dst,
            "wss://dst.example.com/xgen",
            vec![join_id.clone()],
            "2026-06-04T10:00:00.000Z",
        );

        let state = assert_converges(vec![create, invite, join, migrate], &empty_ihn());
        assert_eq!(
            state.home_node.as_str(),
            dst,
            "cutover flips home_node to the destination under every arrival order"
        );
        assert!(
            state.members.contains_key(&xid(&bob_id)),
            "membership converges alongside the cutover"
        );
    }

    // ── Arc G PG-04 — jurisdiction survives a permuted rebuild (AG-D3) ────────

    #[test]
    fn convergence_jurisdiction_survives_permuted_rebuild() {
        // Set-once create-carried field rides M8 for free: it is fixed at Space
        // birth, so every arrival permutation derives the identical SpaceState
        // (the `PartialEq`/`Eq` oracle covers `jurisdiction` additively). Pin the
        // field unchanged across the full-factorial permutation harness, with a
        // concurrent membership conflict present to exercise a real rebuild.
        let owner = kp();
        let x = kp();
        let x_id = id_of(&x);

        let create = sign_event(
            build_space_create_event(&owner, "Gov Space", None, 1, NODE, Some("SK"), false),
            &owner,
        );
        let sid = eid(&create);
        let join_x = mem(&x, &sid, EventType::MembershipJoin, json!({}), &[&sid]);
        let ban_x = mem(
            &owner,
            &sid,
            EventType::MembershipBan,
            json!({ "target_identity": x_id }),
            &[&sid],
        );

        let state = assert_converges(vec![create, join_x, ban_x], &empty_ihn());
        assert_eq!(
            state.jurisdiction.as_deref(),
            Some("SK"),
            "jurisdiction is set-once and must survive every arrival permutation unchanged (AG-D3)"
        );
    }

    // ── Arc H PG-05 — e2e_encryption survives a permuted rebuild (AH-D2) ──────

    #[test]
    fn convergence_e2e_encryption_survives_permuted_rebuild() {
        // Set-once create-carried bool, same shape as jurisdiction (AG-D3):
        // fixed at Space birth, no applier, no state key — the `PartialEq`/`Eq`
        // convergence oracle covers it additively. Pin it `true` across every
        // arrival permutation, with a concurrent membership conflict forcing a
        // real resolving rebuild.
        let owner = kp();
        let x = kp();
        let x_id = id_of(&x);

        let create = sign_event(
            build_space_create_event(&owner, "Secure Space", None, 1, NODE, None, true),
            &owner,
        );
        let sid = eid(&create);
        let join_x = mem(&x, &sid, EventType::MembershipJoin, json!({}), &[&sid]);
        let ban_x = mem(
            &owner,
            &sid,
            EventType::MembershipBan,
            json!({ "target_identity": x_id }),
            &[&sid],
        );

        let state = assert_converges(vec![create, join_x, ban_x], &empty_ihn());
        assert!(
            state.e2e_encryption,
            "e2e_encryption is set-once and must survive every arrival permutation unchanged (AH-D2)"
        );
    }

    // ── Layer 4 — Role priority (Owner invite > Admin invite) ─────────────────

    #[test]
    fn convergence_layer4_owner_invite_beats_admin_invite() {
        let owner = kp();
        let admin = kp();
        let x = kp();
        let owner_id = id_of(&owner);
        let admin_id = id_of(&admin);
        let x_id = id_of(&x);

        let create = create_space(&owner);
        let sid = eid(&create);
        // Unconflicted setup: owner invites admin (role=admin); admin joins.
        // These fix admin's role in the unconflicted basis.
        let inv_a = mem(
            &owner,
            &sid,
            EventType::MembershipInvite,
            json!({ "target_identity": admin_id, "role": "admin" }),
            &[&sid],
        );
        let inv_a_id = eid(&inv_a);
        let join_a = mem(&admin, &sid, EventType::MembershipJoin, json!({}), &[&inv_a_id]);
        let join_a_id = eid(&join_a);
        // Conflict: owner invites X as admin vs admin invites X as member — both
        // reference join_a, concurrent, same membership:sid:X key. Same type
        // (invite) → Layer 1 abstains; empty home-nodes → Layer 3 abstains;
        // Layer 4 decides: Owner > Admin.
        let inv_x_owner = mem(
            &owner,
            &sid,
            EventType::MembershipInvite,
            json!({ "target_identity": x_id, "role": "admin" }),
            &[&join_a_id],
        );
        let inv_x_admin = mem(
            &admin,
            &sid,
            EventType::MembershipInvite,
            json!({ "target_identity": x_id, "role": "member" }),
            &[&join_a_id],
        );

        let state = assert_converges(
            vec![create, inv_a, join_a, inv_x_owner, inv_x_admin],
            &empty_ihn(),
        );

        let pending = state
            .pending_invites
            .get(&xid(&x_id))
            .expect("X has a pending invite from the winning event");
        assert_eq!(pending.role, Role::Admin, "owner's invite (role=admin) must win");
        assert_eq!(
            pending.invited_by,
            Some(xid(&owner_id)),
            "winning invite was issued by the owner"
        );
    }

    // ── Layer 5c — Lexicographic event_id backstop ────────────────────────────

    #[test]
    fn convergence_layer5c_lexicographic_backstop() {
        let owner = kp();
        let x = kp();
        let x_id = id_of(&x);

        let create = create_space(&owner);
        let sid = eid(&create);
        // Two concurrent invites by the SAME owner for X with different roles.
        // Same sender ⇒ Layers 1/3/4/5a/5b all tie/abstain ⇒ Layer 5c (lower
        // event_id wins) is the decider.
        let inv_admin = mem(
            &owner,
            &sid,
            EventType::MembershipInvite,
            json!({ "target_identity": x_id, "role": "admin" }),
            &[&sid],
        );
        let inv_member = mem(
            &owner,
            &sid,
            EventType::MembershipInvite,
            json!({ "target_identity": x_id, "role": "member" }),
            &[&sid],
        );

        // The lexicographically smaller event_id wins; that event's role survives.
        let expected_role = if eid(&inv_admin) < eid(&inv_member) {
            Role::Admin
        } else {
            Role::Member
        };

        let state = assert_converges(vec![create, inv_admin, inv_member], &empty_ihn());

        let pending = state.pending_invites.get(&xid(&x_id)).expect("X has a pending invite");
        assert_eq!(
            pending.role, expected_role,
            "Layer 5c must select the lower-event_id invite deterministically"
        );
    }

    // ── PG-12-min (Arc D) — concurrent state.room_update overrides converge ────

    #[test]
    fn convergence_room_update_overrides_pick_one_winner() {
        let owner = kp();
        let create = create_space(&owner);
        let sid = eid(&create);
        // build_room_create_event records `sid` as the room's sole predecessor.
        let room = sign_event(build_room_create_event(&owner, &sid, "general", None), &owner);
        let room_id = eid(&room);

        // Two concurrent room_updates on the SAME room (same state key
        // `(state.room_update, room_id)`), each carrying a different override set.
        // Neither is a causal ancestor of the other → a conflict M8 resolves to a
        // single winner; the loser's set is dropped. The applier participates in
        // RoomState's PartialEq oracle, so convergence covers permission_overrides.
        let u_a = sign_event(
            build_room_update_event(
                &owner,
                &sid,
                &room_id,
                vec![room_id.clone()],
                &[(Role::Moderator, RoomPermission::SendMessages, Effect::Deny)],
            ),
            &owner,
        );
        let u_b = sign_event(
            build_room_update_event(
                &owner,
                &sid,
                &room_id,
                vec![room_id.clone()],
                &[(Role::Member, RoomPermission::Invite, Effect::Allow)],
            ),
            &owner,
        );

        let state = assert_converges(vec![create, room, u_a, u_b], &empty_ihn());

        let rk = RoomXgid::from_xgid(Xgid::new(room_id));
        let overrides = &state.rooms.get(&rk).expect("room exists").permission_overrides;
        assert_eq!(overrides.len(), 1, "exactly one room_update wins under resolution");
        let mod_deny = overrides.get(&(Role::Moderator, RoomPermission::SendMessages))
            == Some(&Effect::Deny);
        let mem_allow =
            overrides.get(&(Role::Member, RoomPermission::Invite)) == Some(&Effect::Allow);
        assert!(mod_deny ^ mem_allow, "converged to exactly one of the two override sets");
    }

    // ── Thread model (Arc E PG-08) — concurrent resolved vs archived ──────────

    #[test]
    fn convergence_thread_resolved_vs_archived_picks_one_winner() {
        let owner = kp();
        let create = create_space(&owner);
        let sid = eid(&create);
        let room = sign_event(build_room_create_event(&owner, &sid, "general", None), &owner);
        let room_id = eid(&room);
        let thread_ev = sign_event(
            build_thread_create_event(&owner, &sid, &room_id, vec![room_id.clone()], Some("topic"), 1),
            &owner,
        );
        let thread_id = thread_id_from_event_id(&eid(&thread_ev));

        // Two concurrent transitions on the SAME thread (shared state key
        // `(thread.status, thread_id)`) — neither a causal ancestor of the other.
        // resolved-vs-archived has no Layer-1 pair, so resolve() settles it at
        // Layer-5c (lexicographic); the loser is dropped. The applier participates
        // in ThreadState's PartialEq, so convergence covers `status`.
        let resolve = sign_event(
            build_thread_resolved_event(&owner, &sid, &room_id, &thread_id, vec![thread_id_origin(&thread_ev)]),
            &owner,
        );
        let archive = sign_event(
            build_thread_archived_event(&owner, &sid, &room_id, &thread_id, vec![thread_id_origin(&thread_ev)]),
            &owner,
        );

        let state = assert_converges(vec![create, room, thread_ev, resolve, archive], &empty_ihn());
        let status = state.threads.get(&thread_id).expect("thread exists").status;
        // Converged to exactly one terminal status (which one is advisory — M9
        // flag only; the guarantee is that every Node agrees).
        assert!(
            matches!(status, ThreadStatus::Resolved | ThreadStatus::Archived),
            "thread converged to a terminal status; got {status:?}"
        );
    }

    /// The thread.create event's own id (the resolve/archive predecessor — chains
    /// the transition causally under the Thread's origin).
    fn thread_id_origin(thread_ev: &Event) -> String {
        eid(thread_ev)
    }

    // ── §3.9.5 — split-brain recovery is free ─────────────────────────────────

    #[test]
    fn convergence_split_brain_partial_logs_merge_either_way() {
        let owner = kp();
        let x = kp();
        let x_id = id_of(&x);

        let create = create_space(&owner);
        let sid = eid(&create);
        let join_x = mem(&x, &sid, EventType::MembershipJoin, json!({}), &[&sid]);
        let ban_x = mem(
            &owner,
            &sid,
            EventType::MembershipBan,
            json!({ "target_identity": x_id }),
            &[&sid],
        );

        // Two Nodes each saw a partial view: Node A saw {create, join_x}; Node B
        // saw {ban_x}. When they reconcile, the merge order is arbitrary.
        let merged_ab = vec![create.clone(), join_x.clone(), ban_x.clone()];
        let merged_ba = vec![ban_x, create, join_x];

        let s_ab = derive_resolved(merged_ab, NODE, &empty_ihn()).unwrap();
        let s_ba = derive_resolved(merged_ba, NODE, &empty_ihn()).unwrap();

        assert_eq!(s_ab, s_ba, "split-brain reconciliation converges regardless of merge order");
        assert!(s_ab.banned.contains(&xid(&x_id)), "the ban wins on both Nodes");
    }

    // ── No-conflict path == plain replay; degenerate inputs ───────────────────

    #[test]
    fn no_conflict_derivation_equals_plain_replay() {
        let owner = kp();
        let x = kp();
        let x_id = id_of(&x);

        let create = create_space(&owner);
        let sid = eid(&create);
        let inv_x = mem(
            &owner,
            &sid,
            EventType::MembershipInvite,
            json!({ "target_identity": x_id, "role": "member" }),
            &[&sid],
        );
        let inv_x_id = eid(&inv_x);
        let join_x = mem(&x, &sid, EventType::MembershipJoin, json!({}), &[&inv_x_id]);

        let events = vec![create, inv_x, join_x];

        // With no conflicts, derive_resolved must equal an independent plain fold
        // (it neither drops nor reorders non-conflicting events).
        let resolved = derive_resolved(events.clone(), NODE, &empty_ihn()).unwrap();
        let reference = plain_fold(&events);
        assert_eq!(resolved, reference, "no-conflict derivation must match plain replay");
        assert!(resolved.members.contains_key(&xid(&x_id)), "X joined as a member");
    }

    #[test]
    fn derive_resolved_returns_none_without_a_create_event() {
        // Empty log.
        assert!(derive_resolved(vec![], NODE, &empty_ihn()).is_none());

        // A membership event whose Space-create is absent — nothing to seed from.
        let x = kp();
        let orphan = mem(
            &x,
            "xgen://hash/sha256:nonexistent",
            EventType::MembershipJoin,
            json!({}),
            &["xgen://hash/sha256:nonexistent"],
        );
        assert!(derive_resolved(vec![orphan], NODE, &empty_ihn()).is_none());
    }

    // ── conflicts_in_log gate (CP-E) — ancestry-aware, NOT direct-parent-only ──

    #[test]
    fn gate_flags_concurrent_same_key_events_as_conflict() {
        let owner = kp();
        let x = kp();
        let x_id = id_of(&x);
        let create = create_space(&owner);
        let sid = eid(&create);
        // ban and join target the same key and both reference the create root —
        // genuinely concurrent.
        let join = mem(&x, &sid, EventType::MembershipJoin, json!({}), &[&sid]);
        let ban = mem(
            &owner,
            &sid,
            EventType::MembershipBan,
            json!({ "target_identity": x_id }),
            &[&sid],
        );
        let log = vec![create, join, ban.clone()];
        assert!(conflicts_in_log(&ban, &log), "concurrent same-key ban vs join is a conflict");
    }

    #[test]
    fn gate_ignores_causally_ordered_same_key_events() {
        // This is the C1 trap, asserted at the gate: invite X → join X share a
        // state key but are causally ordered (join references invite) — NOT a
        // conflict. A direct-parent-only check would also pass here, but the
        // point is the gate must NOT flag it.
        let owner = kp();
        let x = kp();
        let x_id = id_of(&x);
        let create = create_space(&owner);
        let sid = eid(&create);
        let invite = mem(
            &owner,
            &sid,
            EventType::MembershipInvite,
            json!({ "target_identity": x_id, "role": "member" }),
            &[&sid],
        );
        let invite_id = eid(&invite);
        let join = mem(&x, &sid, EventType::MembershipJoin, json!({}), &[&invite_id]);
        let log = vec![create, invite, join.clone()];
        assert!(
            !conflicts_in_log(&join, &log),
            "causally-ordered invite → join must NOT be flagged as a conflict"
        );
    }

    #[test]
    fn gate_ignores_non_state_keyed_events() {
        // A message has no state key — it can never conflict, so the gate
        // short-circuits before any log scan.
        let owner = kp();
        let create = create_space(&owner);
        let sid = eid(&create);
        let mut msg = crate::message::exchange::build_message_text_event(
            &owner, &sid, "", vec![sid.clone()], "hi",
        );
        msg = sign_event(msg, &owner);
        let log = vec![create, msg.clone()];
        assert!(!conflicts_in_log(&msg, &log), "message events never conflict");
    }

    // ── MP-F4 (A1 + frontier anchor) — scope-aware membership convergence ──────
    //
    // Resolution-side witnesses for the A1 keying. The end-to-end finding fix
    // (the frontier `get_dag_tips` anchor making a room-join causally descend from
    // its space-join) is witnessed at the binary level by `MP-C-07-LOCAL`
    // (`mp_r1_c5`); here the DAG shapes are modelled directly. The keying
    // *distinctness* (room-scoped vs space-level) is pinned by the
    // `resolution::state_key` unit tests; these add the convergence layer.

    /// Signed room-level membership event (non-empty `room_id`) with explicit prevs.
    fn mem_room(
        key: &SigningKey,
        space_id: &str,
        room_id: &str,
        etype: EventType,
        content: Value,
        prevs: &[&str],
    ) -> Event {
        let mut ev = build_membership_event(key, space_id, room_id, etype, content);
        ev.prev_events =
            prevs.iter().map(|p| EventXgid::from_xgid(Xgid::new(p.to_string()))).collect();
        sign_event(ev, key)
    }

    #[test]
    fn mpf4_causal_room_join_makes_space_and_room_member() {
        // The finding fixed (the frontier anchor's DAG shape): a room-join that
        // causally descends from the space-join resolves to bob being BOTH a Space
        // member and a room member, under every arrival permutation. (Pre-fix the
        // room-join was a concurrent sibling of the space-join and was dropped.)
        let owner = kp();
        let bob = kp();
        let bob_id = id_of(&bob);
        let create = create_space(&owner);
        let sid = eid(&create);
        let room = sign_event(build_room_create_event(&owner, &sid, "general", None), &owner);
        let room_id = eid(&room);
        let invite = mem(
            &owner,
            &sid,
            EventType::MembershipInvite,
            json!({ "target_identity": bob_id, "role": "member" }),
            &[&sid],
        );
        let invite_id = eid(&invite);
        let space_join = mem(&bob, &sid, EventType::MembershipJoin, json!({}), &[&invite_id]);
        let space_join_id = eid(&space_join);
        // Frontier anchor effect: the room-join descends from the space-join.
        let room_join =
            mem_room(&bob, &sid, &room_id, EventType::MembershipJoin, json!({}), &[&space_join_id]);

        let state = assert_converges(vec![create, room, invite, space_join, room_join], &empty_ihn());
        let rk = RoomXgid::from_xgid(Xgid::new(room_id));
        assert!(state.members.contains_key(&xid(&bob_id)), "bob is a Space member");
        assert!(
            state.rooms.get(&rk).expect("room exists").members.contains(&xid(&bob_id)),
            "bob is a room member (causal room-join is not dropped)"
        );
    }

    #[test]
    fn mpf4_cross_scope_ban_dominates_room_join_all_orders() {
        // F4-D2 spine / ban-evasion guard-rail: a space-level ban and a room-level
        // join of one id occupy DIFFERENT key-groups under A1 (ban room-agnostic,
        // room-join room-scoped) → both fold → guard + cascade make the ban
        // dominate under every order → banned, in no room. Proves A1 opened no
        // ban-evasion.
        let owner = kp();
        let bob = kp();
        let bob_id = id_of(&bob);
        let create = create_space(&owner);
        let sid = eid(&create);
        let room = sign_event(build_room_create_event(&owner, &sid, "general", None), &owner);
        let room_id = eid(&room);
        let space_join = mem(&bob, &sid, EventType::MembershipJoin, json!({}), &[&sid]);
        let sj_id = eid(&space_join);
        // ban and room-join are concurrent siblings of the space-join.
        let ban = mem(
            &owner,
            &sid,
            EventType::MembershipBan,
            json!({ "target_identity": bob_id }),
            &[&sj_id],
        );
        let room_join =
            mem_room(&bob, &sid, &room_id, EventType::MembershipJoin, json!({}), &[&sj_id]);

        let state = assert_converges(vec![create, room, space_join, ban, room_join], &empty_ihn());
        let rk = RoomXgid::from_xgid(Xgid::new(room_id));
        assert!(state.banned.contains(&xid(&bob_id)), "ban wins — bob banned under every order");
        assert!(!state.members.contains_key(&xid(&bob_id)), "bob not a Space member");
        assert!(
            !state.rooms.get(&rk).expect("room exists").members.contains(&xid(&bob_id)),
            "bob not a room member — cross-scope removal dominates the room-join"
        );
    }

    #[test]
    fn mpf4_room_kick_vs_room_join_same_room_converges() {
        // Pair 2: a room-kick and a room-join of one id in the SAME room share the
        // room-scoped key (conflict preserved) → resolve() settles to one winner,
        // convergent under every order. (Distinctness from a space-join is pinned
        // by the state_key unit tests; this proves the shared-key conflict still
        // converges.)
        let owner = kp();
        let bob = kp();
        let bob_id = id_of(&bob);
        let create = create_space(&owner);
        let sid = eid(&create);
        let room = sign_event(build_room_create_event(&owner, &sid, "general", None), &owner);
        let room_id = eid(&room);
        let space_join = mem(&bob, &sid, EventType::MembershipJoin, json!({}), &[&sid]);
        let sj_id = eid(&space_join);
        let room_kick = mem_room(
            &owner,
            &sid,
            &room_id,
            EventType::MembershipKick,
            json!({ "target_identity": bob_id }),
            &[&sj_id],
        );
        let room_join =
            mem_room(&bob, &sid, &room_id, EventType::MembershipJoin, json!({}), &[&sj_id]);
        // assert_converges is the witness: every permutation derives one identical
        // SpaceState (the shared room-scoped key forces a single resolved winner).
        assert_converges(vec![create, room, space_join, room_kick, room_join], &empty_ihn());
    }

    #[test]
    fn mpf4_room_leave_vs_room_join_same_room_converges() {
        // Pair 3: a room-leave and a room-join of one id in the SAME room share the
        // room-scoped key (conflict preserved) → convergent under every order.
        let owner = kp();
        let bob = kp();
        let bob_id = id_of(&bob);
        let create = create_space(&owner);
        let sid = eid(&create);
        let room = sign_event(build_room_create_event(&owner, &sid, "general", None), &owner);
        let room_id = eid(&room);
        let space_join = mem(&bob, &sid, EventType::MembershipJoin, json!({}), &[&sid]);
        let sj_id = eid(&space_join);
        let room_leave =
            mem_room(&bob, &sid, &room_id, EventType::MembershipLeave, json!({}), &[&sj_id]);
        let room_join =
            mem_room(&bob, &sid, &room_id, EventType::MembershipJoin, json!({}), &[&sj_id]);
        let _ = bob_id;
        assert_converges(vec![create, room, space_join, room_leave, room_join], &empty_ihn());
    }
}
