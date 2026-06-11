// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Injector-actor dispatch (MP-R1-D1 / C7).
//!
//! The raw-wire hostile counterpart to [`crate::batch::run_actor`]. An actor
//! flagged `kind = "injector"` in the manifest does **not** spawn a client
//! process; it speaks the transport directly to its node's `ws://` and — the
//! whole point of C7 — **recvs the node's `Error` frame** (the fire-and-forget
//! batch client cannot, MP-R1-D9). It runs as a concurrent task on the **shared
//! [`Registry`]** alongside the batch actors + the federation/clock director, so
//! it can import a batch actor's exported value (e.g. MP-A-16 targets alice's
//! Space via `{{space_id}}`).
//!
//! ## The attack-directive batch
//! The injector's `.jsonl` is a sequence of attack directives interpreted here
//! (not a client CLI batch): `{"attack": "<kind>", "args": {...}, "id": "..."}`.
//! Member-context directives (`register` / `create_space` / `create_room`) build
//! a persistent [`WireActor`] (the injector's own Space, where it is owner ⇒
//! member); the attack directives then submit a crafted Event and capture the
//! outcome:
//!
//! - `forged_signature` — one-shot (`inject_event`, synthetic Space): step-12
//!   signature rejection (MP-A-05).
//! - `duplicate_id` — submit the same `event_id` twice (MP-A-09): DAG dedup.
//! - `missing_parent` — `prev_events` reference an absent parent (MP-A-10):
//!   Step-9 HeldPending.
//! - `clock_skew` — a 2099 timestamp (MP-A-15): Step-8.5 future-skew → wire 3046.
//! - `fabricated_invite` — a `membership.join` whose `prev_events` reference a
//!   never-issued invite (MP-A-16): Step-9 HeldPending, no membership.
//! - `malformed_frame` — a raw truncated frame (MP-A-12): parse-boundary reject.
//!
//! The C7-specific grounding rule (D-065): where an `Error` frame is the expected
//! rejection signal (forged-signature, clock-skew) the smoke asserts it arrived
//! with the expected code; where the expected outcome is HeldPending/dedup the
//! signal is **absence** (the crafted event never lands) — both captured here so
//! the smoke can assert per-row. If the node *accepts* something it should reject
//! (esp. the fabricated predecessor / expired invite) the absence assertion fails
//! → a real finding to route, not absorb.

use std::collections::HashMap;
use std::time::Duration;

use ed25519_dalek::SigningKey;
use serde_json::Value;
use xgen_common::wire::{Event, EventType};

use crate::batch::BatchLine;
use crate::injector::{
    build_clock_skew_event, build_fabricated_invite_join, build_forged_signature_event,
    build_member_membership_event, build_member_message, build_missing_parent_message, fresh_key,
    inject_event, inject_malformed_frame,
};
use crate::resolve::{placeholders_in, substitute, Registry};
use crate::wireactor::WireActor;
use crate::Result;
use anyhow::{anyhow, bail, Context};

/// How long to listen for the node's `Error` frame after a hostile submit.
const RECV_ERROR_WINDOW: Duration = Duration::from_millis(800);

/// One injector attack's captured outcome.
#[derive(Debug, Clone)]
pub struct InjectionRecord {
    /// The directive `id` (correlates the smoke's assertion to the directive).
    pub id: Option<String>,
    /// The attack kind (e.g. `"clock_skew"`).
    pub attack: String,
    /// The crafted event's `event_id` — the absent-from-transcript oracle target
    /// (`None` for `malformed_frame`, which carries no Event).
    pub event_id: Option<String>,
    /// The node's `Error` reply `(code, string)` if one was recv'd on the wire
    /// (the category-level "why", MP-R1-D9). `None` for HeldPending/dedup/close
    /// outcomes that emit no `Error` frame.
    pub error_reply: Option<(u32, String)>,
    /// For `malformed_frame`: did the node close the connection (rejected the
    /// frame cleanly) rather than hang?
    pub frame_closed: Option<bool>,
}

/// The outcome of driving one injector actor's attack-directive batch.
#[derive(Debug, Clone)]
pub struct InjectorRun {
    pub actor: String,
    pub records: Vec<InjectionRecord>,
}

impl InjectorRun {
    /// The record for a given directive id, if present.
    pub fn record_for(&self, id: &str) -> Option<&InjectionRecord> {
        self.records.iter().find(|r| r.id.as_deref() == Some(id))
    }
}

/// Drive one injector actor's attack-directive batch over the raw transport.
///
/// `targets` is the injector's `(node_label, ws_url)` set (MP-R3-D3) — a
/// single-target injector has one entry; a **multi-target** injector
/// (`nodes = […]`, MP-A-06 equivocation) has ≥2. The single-target directives +
/// the member context use `targets[0]` (the primary node); the `equivocate`
/// directive presents conflicting events to named targets, all from one hostile
/// key. `registry` is shared with the batch actors (imports resolve via `{{key}}`,
/// blocking until published).
pub async fn run_injector_actor(
    actor: &str,
    targets: &[(String, String)],
    lines: &[BatchLine],
    registry: &Registry,
    resolve_timeout: Duration,
) -> Result<InjectorRun> {
    let (primary_label, node_url) = targets
        .first()
        .map(|(l, u)| (l.as_str(), u.as_str()))
        .ok_or_else(|| anyhow!("injector `{actor}` has no target node"))?;
    // The persistent member-context client (created lazily on the first directive
    // that needs it). `forged_signature` / `malformed_frame` are one-shot and use
    // their own fresh connections, so they do not require this.
    let mut member: Option<WireActor> = None;
    let mut space_id: Option<String> = None;
    let mut room_id: Option<String> = None;
    // MP-R3-D3: per-target raw-wire connections for the multi-target `equivocate`
    // directive, all sharing the hostile key — keyed by node label. The primary
    // target's connection is the member context (`member` above); the others are
    // opened on demand here.
    let mut fork_conns: HashMap<String, WireActor> = HashMap::new();
    let mut records: Vec<InjectionRecord> = Vec::with_capacity(lines.len());

    for line in lines {
        // Resolve cross-actor `{{key}}` imports (blocks until published).
        let mut resolved: HashMap<String, String> = HashMap::new();
        for key in placeholders_in(&line.raw) {
            let v = registry
                .wait_for(&key, resolve_timeout)
                .await
                .with_context(|| format!("injector `{actor}` resolving `{{{{{key}}}}}`"))?;
            resolved.insert(key, v);
        }
        let resolved_line = substitute(&line.raw, &resolved);
        let directive: Value = serde_json::from_str(&resolved_line)
            .with_context(|| format!("injector `{actor}` bad directive: {resolved_line}"))?;
        let attack = directive
            .get("attack")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("injector `{actor}` directive missing `attack`: {resolved_line}"))?
            .to_string();
        let id = directive.get("id").and_then(Value::as_str).map(String::from);
        // MP-R2-D2 pacing: an optional top-level `after_ms` on the directive paces
        // the injector's send cadence (MP-A-07 flood intensity = the inter-attack
        // delay; absent ⇒ back-to-back, the R1 default). Read off here, never
        // forwarded (these directives are interpreted locally, not sent verbatim).
        if let Some(ms) = directive.get("after_ms").and_then(Value::as_u64) {
            tokio::time::sleep(Duration::from_millis(ms)).await;
        }
        let args = directive.get("args").cloned().unwrap_or(Value::Null);
        let sarg = |k: &str| args.get(k).and_then(Value::as_str).map(String::from);

        let mut rec = InjectionRecord {
            id: id.clone(),
            attack: attack.clone(),
            event_id: None,
            error_reply: None,
            frame_closed: None,
        };

        match attack.as_str() {
            // ── member-context setup ─────────────────────────────────────────
            "register" => {
                let name = sarg("name").unwrap_or_else(|| "mallory".to_string());
                let wa = ensure_member(&mut member, node_url).await?;
                wa.register(&name)
                    .await
                    .with_context(|| format!("injector `{actor}` register"))?;
            }
            "create_space" => {
                let name = sarg("name").unwrap_or_else(|| "INJ".to_string());
                let wa = ensure_member(&mut member, node_url).await?;
                let sid = wa.create_space(&name).await.context("injector create_space")?;
                rec.event_id = Some(sid.clone()); // = the create event id (liveness probe target)
                space_id = Some(sid);
            }
            "create_room" => {
                let name = sarg("name").unwrap_or_else(|| "general".to_string());
                let sid = space_id
                    .clone()
                    .ok_or_else(|| anyhow!("create_room before create_space"))?;
                let wa = member
                    .as_mut()
                    .ok_or_else(|| anyhow!("create_room needs a registered injector"))?;
                let rid = wa.create_room(&sid, &name).await.context("injector create_room")?;
                rec.event_id = Some(rid.clone());
                room_id = Some(rid);
            }

            // ── one-shot attacks (own fresh connection) ──────────────────────
            "forged_signature" => {
                // Step-12 isolation (Round-0 MP-A-05): the claimed sender must be a
                // real member so steps 9–11 pass and only the signature fails. When
                // the injector has established member context (register +
                // create_space + create_room), forge ITS identity against ITS Space
                // (victim = the registered member key). Else fall back to synthetic
                // (rejected earlier — absence-only).
                let attacker = fresh_key();
                let (victim, space, room) = match (member.as_ref(), &space_id, &room_id) {
                    (Some(wa), Some(sid), Some(rid)) => {
                        (wa.key().clone(), sid.clone(), rid.clone())
                    }
                    _ => (
                        fresh_key(),
                        sarg("space").unwrap_or_else(|| "xgen://hash/sha256:SYNTH_SPACE".to_string()),
                        sarg("room").unwrap_or_else(|| "xgen://hash/sha256:SYNTH_ROOM".to_string()),
                    ),
                };
                let forged = build_forged_signature_event(&victim, &attacker, &space, &room);
                rec.event_id = forged.event_id.as_ref().map(|e| e.as_str().to_string());
                let r = inject_event(node_url, &attacker, &forged)
                    .await
                    .context("injector forged_signature")?;
                rec.error_reply = r.error_reply;
            }
            "malformed_frame" => {
                let outcome = inject_malformed_frame(node_url)
                    .await
                    .context("injector malformed_frame")?;
                rec.frame_closed = Some(outcome.closed);
            }

            // ── member-context attacks (persistent WireActor) ────────────────
            "duplicate_id" => {
                let (sid, rid, key) = member_ctx(&member, &space_id, &room_id, &attack)?;
                let ev = build_member_message(&key, &sid, &rid, vec![&sid], "duplicate injection");
                rec.event_id = ev.event_id.as_ref().map(|e| e.as_str().to_string());
                let wa = member.as_mut().unwrap();
                wa.submit(&ev).await.context("injector duplicate first submit")?;
                // Second submit of the SAME event_id — capture any reply.
                rec.error_reply = wa
                    .submit_recv_error(&ev, RECV_ERROR_WINDOW)
                    .await
                    .context("injector duplicate second submit")?;
            }
            "missing_parent" => {
                let (sid, rid, key) = member_ctx(&member, &space_id, &room_id, &attack)?;
                let fake = sarg("fake_parent")
                    .unwrap_or_else(|| "xgen://hash/sha256:FAKE_PARENT_NEVER_EXISTED".to_string());
                let ev = build_missing_parent_message(&key, &sid, &rid, &fake);
                rec.event_id = ev.event_id.as_ref().map(|e| e.as_str().to_string());
                let wa = member.as_mut().unwrap();
                rec.error_reply = wa
                    .submit_recv_error(&ev, RECV_ERROR_WINDOW)
                    .await
                    .context("injector missing_parent submit")?;
            }
            "clock_skew" => {
                let (sid, rid, key) = member_ctx(&member, &space_id, &room_id, &attack)?;
                let ev = build_clock_skew_event(&key, &sid, &rid, vec![&sid]);
                rec.event_id = ev.event_id.as_ref().map(|e| e.as_str().to_string());
                let wa = member.as_mut().unwrap();
                rec.error_reply = wa
                    .submit_recv_error(&ev, RECV_ERROR_WINDOW)
                    .await
                    .context("injector clock_skew submit")?;
            }
            "fabricated_invite" => {
                // Targets a Space the injector is NOT a member of (imported from a
                // batch actor) — a join referencing a never-issued invite.
                let target = sarg("space")
                    .ok_or_else(|| anyhow!("fabricated_invite needs args.space (the target Space)"))?;
                let fake = sarg("fake_invite")
                    .unwrap_or_else(|| "xgen://hash/sha256:FAKE_INVITE_NEVER_ISSUED".to_string());
                let wa = ensure_member(&mut member, node_url).await?;
                let key = wa.key().clone();
                let ev = build_fabricated_invite_join(&key, &target, &fake);
                rec.event_id = ev.event_id.as_ref().map(|e| e.as_str().to_string());
                rec.error_reply = wa
                    .submit_recv_error(&ev, RECV_ERROR_WINDOW)
                    .await
                    .context("injector fabricated_invite submit")?;
            }

            // ── MP-R3-D3 / MP-A-06 — multi-target equivocation ──────────────
            "equivocate" => {
                // Present two conflicting events to two nodes from ONE hostile
                // key (fork-X → node A, fork-Y → node B at one frontier). Both
                // are individually valid; M8 resolution elects a single winner →
                // the oracle (convergence_verdict) asserts convergence-on-winner.
                // Requires a prior `register` (the shared key + identity
                // replication). The payload (fork types + anchor) is directive
                // **data** — pinned by observation at the RUN (R3-D3), not
                // hard-coded; the candidate is a self-membership leave/join fork.
                let key = member
                    .as_ref()
                    .ok_or_else(|| {
                        anyhow!("equivocate needs a registered injector (add `register` first)")
                    })?
                    .key()
                    .clone();
                let space = sarg("space")
                    .ok_or_else(|| anyhow!("equivocate needs args.space (the target Space)"))?;
                let room = sarg("room").unwrap_or_default();
                let anchor = sarg("anchor").ok_or_else(|| {
                    anyhow!("equivocate needs args.anchor (the shared frontier tip)")
                })?;
                let (type_a, to_a) = parse_fork(&args, "fork_a")?;
                let (type_b, to_b) = parse_fork(&args, "fork_b")?;
                let ev_a =
                    build_member_membership_event(&key, type_a, &space, &room, vec![&anchor]);
                let ev_b =
                    build_member_membership_event(&key, type_b, &space, &room, vec![&anchor]);
                // Record fork-a's id as the primary oracle target (both forks
                // should land; the convergence verdict reads both transcripts).
                rec.event_id = ev_a.event_id.as_ref().map(|e| e.as_str().to_string());
                submit_fork(&mut member, &mut fork_conns, targets, primary_label, &key, &to_a, &ev_a)
                    .await?;
                submit_fork(&mut member, &mut fork_conns, targets, primary_label, &key, &to_b, &ev_b)
                    .await?;
            }

            other => bail!("injector `{actor}`: unknown attack `{other}`"),
        }

        records.push(rec);
    }

    Ok(InjectorRun {
        actor: actor.to_string(),
        records,
    })
}

/// Lazily create + cache the injector's persistent member-context client.
async fn ensure_member<'a>(
    member: &'a mut Option<WireActor>,
    node_url: &str,
) -> Result<&'a mut WireActor> {
    if member.is_none() {
        *member = Some(
            WireActor::connect(node_url)
                .await
                .with_context(|| format!("injector connect {node_url}"))?,
        );
    }
    Ok(member.as_mut().unwrap())
}

/// Parse one equivocation fork directive `{ "type": "<event-type>", "to":
/// "<node-label>" }` (MP-R3-D3). The `type` is an `EventType` wire string (the
/// candidate pair is `membership.leave` / `membership.join`); `to` is the node
/// label the fork is presented to. Both come from the directive **data** so the
/// payload is pinned by the scenario author, not hard-coded.
fn parse_fork(args: &Value, name: &str) -> Result<(EventType, String)> {
    let fork = args
        .get(name)
        .ok_or_else(|| anyhow!("equivocate needs args.{name} = {{type, to}}"))?;
    let ty = fork
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("equivocate {name} needs a `type` (event-type wire string)"))?;
    let event_type = EventType::from_str(ty)
        .ok_or_else(|| anyhow!("equivocate {name} has unknown event type `{ty}`"))?;
    let to = fork
        .get("to")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("equivocate {name} needs a `to` node label"))?;
    Ok((event_type, to.to_string()))
}

/// Submit one equivocation fork to its named target, sharing the hostile `key`
/// (MP-R3-D3). The primary target reuses the member-context connection; other
/// targets get a per-label raw-wire connection (authenticated with the shared
/// key), cached in `fork_conns`.
async fn submit_fork(
    member: &mut Option<WireActor>,
    fork_conns: &mut HashMap<String, WireActor>,
    targets: &[(String, String)],
    primary_label: &str,
    key: &SigningKey,
    to_label: &str,
    ev: &Event,
) -> Result<()> {
    if to_label == primary_label {
        let wa = member
            .as_mut()
            .ok_or_else(|| anyhow!("equivocate fork to primary needs a registered injector"))?;
        return wa.submit(ev).await.context("equivocate submit to primary");
    }
    if !fork_conns.contains_key(to_label) {
        let url = targets
            .iter()
            .find(|(l, _)| l == to_label)
            .map(|(_, u)| u.clone())
            .ok_or_else(|| {
                anyhow!(
                    "equivocate fork targets node `{to_label}` not in the injector's `nodes` list"
                )
            })?;
        let conn = WireActor::connect_with_key(&url, key.clone())
            .await
            .with_context(|| format!("equivocate connect fork target `{to_label}`"))?;
        fork_conns.insert(to_label.to_string(), conn);
    }
    fork_conns
        .get_mut(to_label)
        .expect("just inserted")
        .submit(ev)
        .await
        .context("equivocate submit to fork target")
}

/// Require an established member context (space + room created), returning the
/// space id, room id, and a clone of the injector's signing key (cloned so the
/// event can be built before the subsequent `&mut` submit).
fn member_ctx(
    member: &Option<WireActor>,
    space_id: &Option<String>,
    room_id: &Option<String>,
    attack: &str,
) -> Result<(String, String, ed25519_dalek::SigningKey)> {
    let wa = member
        .as_ref()
        .ok_or_else(|| anyhow!("attack `{attack}` needs a registered injector (add `register` first)"))?;
    let sid = space_id
        .clone()
        .ok_or_else(|| anyhow!("attack `{attack}` needs a member-context Space (add `create_space` first)"))?;
    let rid = room_id
        .clone()
        .ok_or_else(|| anyhow!("attack `{attack}` needs a member-context Room (add `create_room` first)"))?;
    Ok((sid, rid, wa.key().clone()))
}
