# M10.5 — The M10-Routed Carve-Outs (MP-C-16 re-run · MP-F6 fold · MP-C-06 re-home) — Design
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-14 (J-374 amendment: D4 emit dropped — see §4)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Status & lineage

The M10.5 design — the final M10 sub-arc; when it closes, **M10 closes**. Built on the Phase-0 audit
`tasks/M10_5_CARVEOUTS_AUDIT.md` v1.0 (`6947a78`, ACTIVE), which grounded all three axes against live
`main` (1397/0 baseline). Chat verified the load-bearing groundings (the §3.2 line-drift `:748` /
`:693`; the §3.3 re-registration `push_identity_to_peers` at app.rs:2952) — both hold. Joe locked the
three design calls by-recomms (J-373). Doc-only; no code; no DECISIONS change (M10.5-D# arc-local,
D-069). Next: runbook → impl → close.

The three brief-level calls (J-372) were: **C1a** MP-C-16 = verification re-run, loop-on-fault;
**C1b** MP-F6 = bounded fold; **C1c** MP-C-06 = narrow-first-with-escape. This design locks their
**shapes** + the witness set + the escape disposition.

**Audit headline (the shape Joe should hold):** M10.5's production side is genuinely light across all
three axes — one dispatch-level pre-check (C1b) + one thin client emit (C1c) + one per-Space home query
in a witness (C1a). **The center of gravity is the harness re-home rails (C1c / audit A8)** — keypair
relocation + per-phase node retarget in `xgen-mptest` — bigger than all the production work combined,
and reusable re-home infrastructure.

---

## 1. The three calls — Joe-LOCKED (J-373)

| Call | Lock | Production | Test/harness |
|---|---|---|---|
| **C1a** MP-C-16 | re-run **+ a witness enrichment** (flip-on-both) | none (M10.4 fix is in) | a per-Space home query in the box-gated witness |
| **C1b** MP-F6 | fix shape **(b)** dispatch-level `banned` pre-check; **reuse the PermissionDenied-class reject** (no new wire code, no ch3 edit) | a small `dispatch_event` pre-check | a RED-on-revert unit |
| **C1c** MP-C-06 | **BUILD narrow-first**; **escape (client fan-out) OUT** | a thin client `home_changed` emit | the re-home rails (the real lift) |

---

## 2. C1a — MP-C-16 witness enrichment (M10.5-D1)

**The fix is in (audit A1, Chat-verified).** The M10.4 Shape-B fix flows end-to-end by construction:
a freshly-created Space's signed `content["home_node"]` is the source node's pubkey id (from the
`AuthOk.node_id` echo, `SessionState.node_id`), so Site 1 (`admin_ops.rs:2096`, `MIG_6010`) and Site 2
(`exchange.rs:717`, `6009`) both pass, and `apply_space_migrate` flips `home_node` to the destination
pubkey (`state.rs:1161`). Nothing to build on the fix path.

**The build = the witness enrichment (audit A2).** The box-gated test
`mp_r2_fixed::mp_c_16_live_migration_space_rehomes` (`xgen-mptest/tests/mp_r2_fixed.rs:302`) today
asserts only `require_ok` of `migration initiate` + Space-present-on-destination-B
(`!tb.event_ids_for_space(&space).is_empty()`, :314-318). Its own doc-comment (:320-322) flags the
gap: the D3 (J-370) **`home_node`-flip-on-both** is unasserted because no per-Space `home_node` read
surface is driven.

**M10.5-D1 (LOCKED).** Enrich the witness with a **per-Space `home_node` query on both nodes**, asserting
the full D3 shape: post-migration, A's copy **and** B's copy both report `home_node` = B's pubkey id.
The design grounds the read surface at runbook (Clair's confirm): prefer an existing space-info /
transcript read over a net-new verb; if none exposes per-Space `home_node`, add the minimal query
surface the witness needs (test-crate-side if possible; a thin aicontrol/admin read only if the node
must expose it). **Box-gated** (2 real `--features harness-control` nodes + Mock clock) → the observed
green is the M10.5 RUN, **loop-on-fault**. Flip **MP-F13 RESOLVED** on observed green (no
unobserved-result claim, J-352).

---

## 3. C1b — MP-F6 bounded fold (M10.5-D2, M10.5-D3)

**The site (audit A3, line-drift corrected + Chat-verified).** The apply-swallow is
`let _ = state.apply_event(&event, &my_node_id)` at **`runtime.rs:748`** (the brief's `:691` was stale;
`:693` is now `let _ = store.append`). It sits in `ingest_event` (@585), reached via `dispatch_event`
(@1001); `ingest_event` returns `()`, so the apply error is double-swallowed and cannot reach
`dispatch_event`'s reply. A banned identity's re-join is therefore **accepted-but-inert**: `is_ok=true`
to the client, while `apply_join`'s `banned` consult (`state.rs:1003`) excludes them at
`derive_resolved`. The dishonesty is in the **reply**.

**The sweep is clean (audit A4).** D-077-bidirectional across every `let _ =` in `runtime.rs`: the
`:748` silence is load-bearing **backward for replay** (`replay_spaces_from_dir → ingest_event` wants
tolerant apply — a replayed event resolution will drop must not crash replay), **not** for live
dispatch. `:546` (rehydrate graph-add) is benign cold-start (an unlogged sibling of the loud `:676`);
`:693` (store.append duplicate-ignore) is a documented ignore with its own candidate-D-NNN future-walk.
**No swallow is load-bearing elsewhere in the dangerous sense.** Nothing to route.

**M10.5-D2 (LOCKED) — fix shape (b).** Add a **dispatch-level `banned` pre-check** in `dispatch_event`,
before the `ingest_event` call, for `membership.join` events: consult the target Space's `banned` set
and return `DispatchOutcome::Rejected(RejectInfo{ … })` for a banned joiner. Chosen over (a) fallible
`ingest_event`, which would ripple the error into the replay caller (which *depends on* tolerant apply,
A4) and touch `ingest_event`'s many callers. Shape (b) makes the **reply** honest with **zero replay
blast-radius** (the `:748` silence stays, serving replay tolerance) and reuses the existing `RejectInfo`
machinery — exactly "the second gate moved up to the reply."

**M10.5-D3 (LOCKED) — the reject code = reuse PermissionDenied-class; no new wire code.** A banned-join
reject surfaces via the **existing PermissionDenied-class reject** (the same `4000`-unmapped shape
MP-C-09's banned-*send* reject lands as, MP-F2-followon territory) + a "banned" reason string. **No new
3040s code, no ch3 edit.** Rationale (RC-F-01-disciplined + proportionate): ch3's 3040s
membership-authority sub-band is 3040–3045 with no banned-join admission code, and `3046` is already
`TimestampOutOfBounds` in code — assigning a precise `join_banned` code is RC-F-01 work + a wire-band
edit, disproportionate for a LOW-sev reply-honesty fold. **Wire-code precision is the named home of
MP-F2-followon** (the unmapped-variant cleanup), not M10.5. C1b stays a true bounded fold (no spec
touch). If a future arc wants the precise code, it lands there.

---

## 4. C1c — MP-C-06 re-home (M10.5-D4, M10.5-D5, M10.5-D6)

**What MP-C-06 is.** The **S5 identity re-home**: an orphaned Identity re-registers on a new Node with
the **same keypair** (`re_registration:true`), keeping `identity_id` + Space membership; federated peers
must learn the home moved (`identity.home_changed`) so their replica records re-point. Distinct from
MP-C-16 (operator-driven Space *migration*).

**Why narrow-first holds (audit A6/A7, Chat-verified).** The CP-5 blocker is dissolved
(`SessionState.node_id` sources `new_home_node_id`, M10.4); the receive side is fully built
(`handle_incoming_home_changed` replication.rs:165 + dispatch app.rs:3162 + `build`/`sign`/`verify`
registration.rs:391/412/423); and — Chat-verified — **re-registration already fires
`push_identity_to_peers`** (`re_home = re_registration && already`, app.rs:2928; spawn at app.rs:2952),
so in MP-C-06's full-mesh topology the re-homed record reaches the space-member peer via existing
replication. The `home_changed` emit is the spec-faithful **single-hop delta-notification**, not the
convergence mechanism — **thin, not a broadcast arc.**

**M10.5-D4 (AMENDED J-374 — emit DROPPED; MP-C-06 closes on replicate-convergence).** CP-4-condition-2 grounding (Clair) showed the `home_changed` emit is a **structural no-op** in MP-C-06's topology: it is single-hop to the new home C (D4), C already set its record `home_node = C` + bumped `update_version` during the registration it just processed (`registration.rs:543`, app.rs:2928-2940) **before** RegisterOk returns, so the client's later emit arrives `update_version <= existing` → `VersionStale` no-op (`replication.rs:173`); C is the only recipient (no re-forward, A7), while B — the node that needs to learn the home moved — is re-pointed by `push_identity_to_peers` regardless. The emit reaches only the node that already knows; shipping it would be decorative (D-065). **Re-locked (Joe, J-374): drop the production emit from M10.5.** MP-C-06 closes on **replicate-convergence** (A7): the C1c witness asserts B re-points alice's replica to C via `push_identity_to_peers` + identity/membership continuity (alice posts from C → reaches the Space; same `identity_id`; membership preserved) — the real mechanism, not a `home_changed`-observability assertion. The versioned + replica-holder-fanned-out `home_changed` (the form that is **not** a no-op — it needs **both** the RegisterOk version-echo **and** the fan-out to reach B; the echo alone wouldn't reach B) is the named escape (option i), **routed forward**, its own arc. **C1c = harness rails (3a) + replicate-convergence witness (3c); no 3b emit.** M10.5's production footprint is therefore **C1b alone**.

*(superseded — original D4 below, retained for the record:)* **M10.5-D4 (LOCKED) — the thin emit.** Build the client `home_changed` emit: on a re-home,
`build_home_changed` + `sign_home_changed` + `send_identity` (single-hop) to the **new** home node,
sourcing `new_home_node_id` from `SessionState.node_id` (the connection actually used — `--node`-honoured).
No new broadcast/fan-out machinery. The emit's `old_home_node_id` = the client's own
`IdentityRecord.home_node` (the home being left); the applier keys/re-points by `identity_id` +
`new_home_node` + version and does **not** gate on `old_home_node_id`, so for a legacy URL-homed record
(M10.4-D5 leave-as-legacy) carrying a URL there is cosmetic-for-convergence — **the emit carries the
record's stored value as-is, with the legacy case noted** (spec faithfulness; not a convergence risk).

**M10.5-D5 (LOCKED) — the escape is OUT.** A client-side **fan-out** (the client emitting `home_changed`
to each node holding its replica) is only needed when a space-member node is **not** federated with the
new home (so re-registration's `push_identity_to_peers` cannot reach it). That net-new fan-out loop is
the "heavy broadcast arc" the brief warned against. It **does not fire for MP-C-06** (full-mesh
topology). **Flagged, NOT built.** If a future design wants `home_changed` to reach non-co-federated
space-member nodes, that re-locks depth (a client fan-out over `ClientState.spaces[].node_endpoint`) —
its own arc, out of M10.5.

**M10.5-D6 (LOCKED) — the harness re-home rails (the real lift; audit A8).** Net-new `xgen-mptest`
(test-crate) machinery, built **rails-first** (the C1c witness rides them):
- **Key continuity** — one keypair used across two node connections (register on A → re-register on
  the new home). The actor model today binds one fresh `--init` keypair per actor; the rails add a
  keypair-relocation / re-home actor phase so the same identity re-homes.
- **Per-phase node retarget** — a re-home actor switches its target node mid-scenario. The batch path
  has a per-command `--node` (`xgen-client/src/app.rs:914`); the aicontrol path hardcodes
  `node_override:None` (aicontrol.rs:360) and the scenario format binds one node per actor. The rails
  add a per-phase / per-actor node-retarget surface for the re-home step.
- These are reusable re-home test infrastructure (general, not MP-C-06-specific).

---

## 5. Witness set (M10.5-D7, LOCKED; runbook refines)

- **C1a (MP-C-16):** the box-gated `mp_r2_fixed::mp_c_16_live_migration_space_rehomes`, enriched with
  the per-Space home query → assert D3 (`require_ok` + `home_node` = B's pubkey on **both** A and B).
  Box-gated → observed green at the M10.5 RUN; loop-on-fault; flips MP-F13 RESOLVED.
- **C1b (MP-F6):** a `dispatch_event` unit (xgen-core, box-free) — a banned identity's `membership.join`
  returns `DispatchOutcome::Rejected` (honest reply, PermissionDenied-class). RED-on-revert = remove the
  pre-check → `Accepted` (`is_ok=true`) with the joiner absent from the resolved membership
  (accepted-but-inert).
- **C1c (MP-C-06):** the harness re-home witness — the same keypair re-homes A→C, a post from C reaches
  S, identity + membership continuous, `home_changed` observable in the peer's record/log. Needs the D6
  rails first. RED-on-revert = neuter the emit / the keypair-continuity.

---

## 6. Sequencing (M10.5-D8, LOCKED)

**C1a → C1b → C1c.** Cheapest-confirming first: C1a stands up the box-gated MP-C-16 re-run + the small
witness enrichment (confirms the M10.4 fix end-to-end, flips MP-F13). Then C1b (the bounded fold). Then
C1c (the real lift — rails-first, then the thin emit, then the witness). Each is a separate commit;
Clair's code precedes the Chat doc-bridge per arc; Joe pushes. **Loop-to-green** (D-065/MP-R1-D10): a
faced fault gets fixed and rerun, not papered over.

**Close = all-green, no carve-out** (these three *are* the carve-outs, coming home): C1a green
(MP-F13 RESOLVED) + C1b folded (witness GREEN, RED-on-revert) + C1c green (re-home witnessed) →
**M10.5 closes → M10 closes.**

---

## 7. Out of scope (fence)

- The consolidated R1+R2+R3 ledger — rides MP-R3's MP-F14 close (its own track), NOT M10.5.
- Layer-2 production identity→home-node **discovery** of a stranger (M10.4-D4 / F1B-D5) — separately
  routed; never an MP-C-16/MP-C-06 dependency.
- The MP-C-06 escape (client fan-out to non-co-federated space-member nodes) — flagged (M10.5-D5), not
  built unless a future design re-locks depth.
- A precise `join_banned` wire code (M10.5-D3) — routed to MP-F2-followon, not M10.5.
- Legacy URL-homed Space migration (M10.4-D5) — leave-as-legacy.
- The orchestrated one-shot `recover-identity` command (M8.5-C S5-D4) — deferred UX sugar.

---

## 8. Next-active

**Clair authors the runbook** `tasks/M10_5_CARVEOUTS_IMPL.md` — confirm the §2 read-surface for the
C1a per-Space home query + the §3 `dispatch_event` pre-check insertion point + the §4 client emit
surface + the D6 harness rails to file:line → implement (C1a → C1b → C1c, rails-first within C1c) →
Chat doc-bridges per arc → close (all-green, no carve-out → M10.5 closes → M10 closes). No code until
the runbook lands. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-373 → this design →
`tasks/M10_5_CARVEOUTS_AUDIT.md` → `tasks/MP_findings.md` (MP-F13/MP-F6/MP-C-06).**

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-077 + D-078.
