# M8.5 — Finalization Phase-0 Audit (INV · F-5 · S5)
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose

Phase-0 subsystem audit for milestone **M8.5** (the finalization box) per D-071.
Grounds the three M8 diagnostic findings routed to M8.5 — **INV** (invitee
membership-bootstrap), **F-5** (federation anti-transitivity), **S5** (identity
re-bind surfaces) — against the live tree, and **frames** the design forks each
carries. This audit decides nothing: forks are framed for the per-item design
sessions to lock with Joe (D-069/D-071). Entry per Rule 0: CLAUDE.md PLAY →
JOURNAL J-269 → `tasks/M8_findings.md` §3/§4/§5 → this audit.

---

## 2. Grounding method

Read against **HEAD `cecb5ee`** (the M8-close commit; M8 ran on `676b9c1`, no
code drift since — doc-only close). Files inspected: `xgen-client/src/ops.rs`
(invite/join/sync read paths), `xgen-client/src/batch.rs` (`get_dag_tips`),
`xgen-node/src/fanout.rs` (`collect_sync_history`), `xgen-node/src/federation_session.rs`
(the F-5 guard), `xgen-node/src/app.rs` (federation origin tagging),
`xgen-client/src/app.rs` (`RegisterArgs`), `xgen-core/src/wire/types.rs` +
`identity/replication.rs` (identity message surfaces). All findings cite a
concrete code anchor (D-078, production-grounded).

---

## 3. Findings — INV (invitee membership-bootstrap)

**M85-A1 — sync is member-gated (root cause).** `collect_sync_history`
(`fanout.rs` ~727) skips every Space the requester is not already a member of:
`if !space.is_member(requester_id.as_str()) { continue; }`. A pending invitee
(seeded into `pending_invites`, not `members`) therefore receives **zero events**
on a `sync_request` — it cannot see the Space's DAG at all, including the invite
that names it. GAP-CONFIRMED; this is the structural cause both layers below
reduce to.

**M85-A2 — Err-only fallback (the contained sub-bug).** `ops::join` (`ops.rs:770`)
chains the join to `get_dag_tips(...).unwrap_or_else(|_| vec![args.space.clone()])`.
The fallback is **`Err`-only**; for the invitee `get_dag_tips` returns
`Ok(vec![])` (no member-visible events match the Space, per A1), so `prev_events`
becomes **empty** — a root-shaped, non-root event the Node gate-rejects. Real and
narrowly fixable (treat `Ok(empty)` like `Err`), but insufficient: see A3.

**M85-A3 — membership-key collision (the real gap).** `membership.invite` keys
its resolution on the **target**; `membership.join` keys on the **sender** —
both land on `membership:{space}:{invitee}`. With no causal link between them
(the invitee can't chain its join off an invite it can't see, A1), the two are
**concurrent on one state key**, and `derive_resolved` Layer 4 elects the Owner's
invite over the Member's join → join dropped → invitee never becomes a member.
Confirmed by the production-faithful test fixtures (`fanout.rs` `setup_three_member_space`,
`ops.rs` `members_projection_*`), which **hand-chain** `invite.prev=[room]` /
`join.prev=[invite_id]` specifically to avoid this — a linkage production cannot
reproduce without first seeing the invite.

**M85-A4 — §4 candidate seam (what a fix must supply).** The acceptance-time
bootstrap (`M8_findings.md` §4) requires two mechanisms, both grounded against A1:
(i) an **invite-authorized read path** so a valid-unexpired-invite holder can
source current sync without prior membership; and (ii) a way for the invitee to
**learn the invite `event_id`** so its join chains causally after the invite (not
concurrent — dissolving A3). Public material only (identity public key + MLS
public KeyPackage). Open design questions Q1/Q2 (§5).

---

## 4. Findings — F-5 (federation anti-transitivity)

**M85-A5 — the guard is deliberate and documented.** `federation_session.rs:268`
returns early on `EventOrigin::ReceivedViaFederation` — the first action in the
push function, with a normative doc-comment at L227 ("received via federation
MUST NOT be pushed onward"). Inbound federation events are tagged
`ReceivedViaFederation` at the app.rs ingest arms (≈2050/2059/2190/2199). The
Node is **mesh by explicit design**, not by accident.

**M85-A6 — spec contradiction (the fork).** Spec §3.2 states "forward on accept";
the S3/S0 multiparty scenarios assume transitive (non-adjacent) delivery. The
shipped code does the opposite. This is a genuine **propagation-model decision
fork**, not a bug to patch — recorded as the M8.5-A fork (§5).

> **Amendment (2026-06-05, M8.5-A) — M85-A6 CORRECTED.** Grounding for M8.5-A found **no ch3 §3.2 "forward on accept" premise** (§3.2 is the Event Specification), and the multiparty tests (`phase9_three_node_anti_transitivity`, `phase9_compound_c2_*`) **assert** anti-transitivity rather than assume transitive delivery. F-5 is **not an open fork**: it was decided and JOE-LOCKED May 2026 as **Option 1** (pairwise, no transitive relay) at `docs/xgen_federation_propagation_design.md` §8.4, with the `:268` guard + `f5_anti_transitivity_*` regression test already shipped. The real gap is **doc-coherence** — ch3 never absorbed F-5 (a D-069 issue), not a propagation-model decision. M8.5-A closed it doc-only: **ch3 §3.4.8 added + DECISIONS.md D-089 synchronized**. See `tasks/M8_5_A_F5_COHERENCE.md`. The §6 "Fork F-5" framing below is superseded accordingly.

**M85-A7 — a transitive option has substrate today.** Event-id dedup already
exists (per-Space pending buffer + EventStore `contains`), and `event_id` is a
stable content hash (D-076), so a re-forward path (option A) would have a clean
dedup base. The parts option A would still need: lift/scope the guard for
re-forward, plus a loop/TTL/propagation-policy to bound amplification. Option B
(commit to mesh) needs only a spec §3.2 rewrite. Recorded, not decided.

---

## 5. Findings — S5 (identity re-bind surfaces)

**M85-A8 — register surface incomplete.** `RegisterArgs` (`xgen-client/src/app.rs`)
carries only `--name`; there is no `re_registration` flag on the CLI **or** in the
`identity.register` wire shape (`IdentityMessage::Register`, `wire/types.rs`).

**M85-A9 — `home_changed` absent (2 of 3 surfaces missing).** Only
`identity.replicate` / `identity.replicate_ack` are wired (`IdentityReplicateMessage`).
There is **no** `identity.home_changed` EventType, builder, or applier anywhere
in `xgen-core`. So 1 of the 3 identity-mobility surfaces exists; M8.5 builds the
other 2.

**M85-A10 — identity-level, not Space-DAG (M8-free).** Home-change touches the
`IdentityRegistry` + the replication path — **not** `derive_resolved` / Space
state keys — so it carries no multiparty-convergence surface and no new M8 risk.
Heaviest in *build* shape (new surfaces), lightest in *interaction* shape. Open
question Q3 (§6).

---

## 6. Design forks (framed — NOT locked)

These are for the per-item design sessions to lock with Joe (D-069/D-071). The
audit records leanings only.

- **Fork F-5 (M8.5-A) — propagation model.** **(A) transitive** (re-forward
  received events; needs guard-lift + loop/TTL/propagation-policy; reuses
  existing dedup) **vs (B) commit to mesh** (rewrite spec §3.2; matches built
  code, simplest). **Audit lean: B**, absent a goal of chain-topology federation.
  Decide first — it shapes INV (§4 of findings: INV is the *membership*-boundary
  tip-visibility problem, F-5 the *node*-boundary forwarding problem; settling
  them in one milestone avoids fixing one against an assumption the other
  overturns).
- **INV Q1 (M8.5-B) — invite-authorized read.** Relax `collect_sync_history`'s
  member-gate for a valid-unexpired-invite holder **vs** add a separate
  invite-scoped fetch path. (Gate-relax is smaller; a separate path is cleaner
  to reason about for blindness/authorization.)
- **INV Q2 (M8.5-B) — invite `event_id` delivery.** The invite-as-capability
  carries its own immutable `event_id` to the invitee out-of-band **vs** the Node
  serves it on presentation of the invite. (Determines how the join sources the
  causal anchor that dissolves A3.)
- **S5 Q3 (M8.5-C) — home-change semantics.** Re-registration flow (onboarding a
  new home Node) **vs** a record-update + replication re-point. (Determines
  whether the new surface is a `register` variant or a distinct
  `identity.home_changed` event.)

---

## 7. Scope fence (OUT of M8.5)

Real RFC 9420 MLS crypto (D3, PG-05 interface-locked); multi-device seam (pulled
to a future arc at J-267, R2-F09); the strategic multiparty test **harness**
(that is M9); production load measurement (M9). M8.5 is correctness fixes only.

---

## 8. Suggested sub-arc roadmap + next-active

Each sub-arc runs the standard lifecycle (design → Joe-lock → runbook → Clair →
combined-commit close), D-071.

- **Phase 0 (this audit)** — 3 items grounded, forks framed. ACTIVE.
- **M8.5-A — F-5 fork** — pure protocol decision; lock A vs B with Joe. Decide
  first (shapes INV).
- **M8.5-B — INV bootstrap** — the headline build; co-designed with A per §4
  (membership-boundary vs node-boundary).
- **M8.5-C — S5 surfaces** — build-new, self-contained, identity-level; last.

**Next-active: M8.5-A design** (the F-5 propagation fork) — open with the fork
framing in §6, lock A or B with Joe, then INV (M8.5-B) co-design.

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-078.
