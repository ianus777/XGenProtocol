# Phase-0 Audit — Thin-verb Arc 1: `create-space --auth-tier` (MP-A-03 / PG-13)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The D-071 Phase-0 audit for the **first** thin-verb arc (order Joe-LOCKED at
J-334: auth-tier → ban → room_update → thread×3). It grounds the verb-add
surface against live `main`, answers the three J-334 pivots, and frames the
design forks for Joe-lock. **No code, no design lock pre-decided here** — this
is the reality map the design phase consumes.

Arc goal: ship `create-space --auth-tier <n>` so a Space with `auth_tier ≥ 2`
becomes authorable from the client, which unblocks **MP-A-03** (tier-gate join
refusal, PG-13) with a genuine RED-on-revert witness.

**Grounding note (line drift, per the handoff warning).** The matrix/handoff
cite stale lines; the live anchors found this pass are:
- `ops::create_space` passes the literal `1` at **[ops.rs:400](../xgen-client/src/ops.rs#L400)** (matrix cited 357).
- The PG-13 join-gate lives at **[runtime.rs:1255–1342](../xgen-core/src/node/runtime.rs#L1255)** (matrix cited 1155).

---

## 2. Verb-add surface (grounded — thinner than the 4-site framing)

`build_space_create_event(key, name, topic, auth_tier: u32, home_node, jurisdiction, e2e)`
already takes the tier as its **4th positional arg** ([state.rs:1366](../xgen-core/src/space/state.rs#L1366))
and writes it into `content["auth_tier"]` ([state.rs:1377](../xgen-core/src/space/state.rs#L1377)).
The client currently hardcodes `1` into that slot.

**Live surface — only TWO substantive touches; the dispatch arms are pass-through:**

| # | Site | Change | Notes |
|---|------|--------|-------|
| 1 | `CreateSpaceArgs` ([app.rs:471](../xgen-client/src/app.rs#L471)) | **add** a clap field | `--auth-tier`, **default 1** → absent-flag is byte-identical to today |
| 2 | `ops::create_space` ([ops.rs:400](../xgen-client/src/ops.rs#L400)) | **replace** literal `1` → `args.auth_tier` | the one load-bearing line |
| 3 | `batch::dispatch_line` `CreateSpace` arm ([batch.rs:349](../xgen-client/src/batch.rs#L349)) | **no change** | forwards the whole `&args` struct to `ops::create_space`; clap-parsed field rides along |
| 4 | `cmd_create_space` CLI shim ([app.rs:2158](../xgen-client/src/app.rs#L2158)) | **no change** to threading | forwards `&args`; `--help` auto-generated from the clap doc-comment |

Sites 3 + 4 are pass-throughs because both dispatchers forward the entire
`CreateSpaceArgs` struct rather than per-field — adding a clap field
auto-propagates. So the verb-add is genuinely thin: **one struct field + one
literal swap.** (Optional cosmetic: echo the tier in the `cmd_create_space`
success print / add `auth_tier` to `CreateSpaceResult` — design-phase nicety,
not required for MP-A-03.)

**Type note (correct the handoff's "u8").** The whole chain is **`u32`**:
`build_space_create_event(auth_tier: u32)`, `SpaceState.auth_tier: u32`
([state.rs:190](../xgen-core/src/space/state.rs#L190)),
`verify_tier_assertion(joiner_tier: u32, space.auth_tier: u32)`. The clap field
should be `u32` to avoid a cast (no-drift, D-067). Flagging because the handoff §1
said `u8`.

**Wire-neutrality: CONFIRMED.** `auth_tier` already rides the signed event
content (it has since Phase 1; the canonical signing form includes it). Passing a
different integer changes the *value* of an existing field, not the wire shape —
zero serialization/canonical-form change, no separate Joe-lock needed.

---

## 3. The three Phase-0 pivots

### Pivot (a) — gate-teeth → **HAS TEETH; MP-A-03 is green-eligible (does NOT route a finding)**

The PG-13 join-gate is **live, not decorative.** At dispatch step 4
([runtime.rs:1255–1342](../xgen-core/src/node/runtime.rs#L1255)) a `MembershipJoin`:
1. reads the Space's slot contract `space.auth_tier`;
2. resolves the joiner's tier via `assertion_tier_of(record)` ([runtime.rs:214](../xgen-core/src/node/runtime.rs#L214)) — `None`/unregistered ⇒ tier **1**; `Some(v)` ⇒ the **validated** stored tier (Arc E/PG-03 gave this teeth);
3. calls `verify_tier_assertion(joiner_tier, space.auth_tier)`;
4. on shortfall → `DispatchOutcome::Rejected` carrying **wire 3030 `tier_mismatch`**, and the joiner is **not** added.

The gate is already **unit-proven** with passing tests:
- `pg13_tier1_join_into_tier2_space_rejected_3030` ([runtime.rs:3968](../xgen-core/src/node/runtime.rs#L3968)) — hand-sets `space.auth_tier = 2`, Tier-1 joiner → Rejected 3030 + absent from members.
- `pg13_tier2_join_into_tier2_space_accepted` ([runtime.rs:4008](../xgen-core/src/node/runtime.rs#L4008)) — Tier-2 joiner into Tier-2 Space → admitted.

The "genuine Tier-1 no-op today" framing (matrix + [runtime.rs:1262](../xgen-core/src/node/runtime.rs#L1262)) is precise but narrow: the gate is a no-op **only because no Tier-2 Space is creatable end-to-end** — every Space is `auth_tier=1` and every joiner resolves to 1, so `verify_tier_assertion(1,1)=Ok`. **This arc removes exactly that blocker.** Once `create-space --auth-tier 2` ships, a real Tier-2 Space exists, a Tier-1 joiner is refused (3030), and the gate bites for the first time end-to-end.

**Verdict: green-eligible.** MP-A-03 passes when the verb ships — it does **not** route a finding. (Honest caveat: the unit test proves only `gate-given-hand-set-tier`. The integration witness this arc owns is what proves the full **authoring → `from_space_create` persist → gate** chain; see §6.)

### Pivot (b) — creation cap → **UNCAPPED (confirmed); breadcrumb to M10, do NOT widen**

The node accepts whatever `auth_tier` the client signs into the create event.
Grounded two ways:
- `SpaceState::from_space_create` ([state.rs:272–319](../xgen-core/src/space/state.rs#L272)) reads `content["auth_tier"]` and stores it verbatim — present-or-`MissingField`, **no cap, no policy check** ([state.rs:275](../xgen-core/src/space/state.rs#L275)).
- `validate_event` for `StateSpaceCreate` ([exchange.rs](../xgen-core/src/message/exchange.rs)) has **no `auth_tier` check** — step-11 sender-membership is skipped for creates ([exchange.rs:638](../xgen-core/src/message/exchange.rs#L638)), and the only create-specific gate is the M3 AI-owner rejection ([exchange.rs:267](../xgen-core/src/message/exchange.rs#L267)). No tier gate anywhere on the create path.

So a Tier-1 creator (e.g. alice, no assertion) can create a Tier-2 Space.

**This is NOT an auth-tier-arc defect.** In a real tiered world, creating a
Tier-N Space should require a Tier-N attestation — but tiered attestation only
lands in the **M10 auth-module** era. Two consequences:
- **Breadcrumb (recorded, not built):** *"create-event ingest does not cap `auth_tier` against the creator's own tier; a Tier-1 creator can mint a Tier-N Space. The create-cap question belongs to M10 (Auth Module), which owns per-tier attestation and policy."*
- **Convenient for this arc:** MP-A-03's alice can create a Tier-2 Space without a Tier-2 attestation existing (none do yet). No need to fabricate creator-side tier state — the scenario works on current rails. **Do not widen this arc to add a cap.**

### Pivot (c) — oracle shape → **effect-absence (Option-A paired), C6 batch; recommended**

MP-A-03's matrix expectation is "refusal multiparty-visible + converged;
`category=permission`." Per **MP-R1-D9** (design §10), `category=permission` is
**NOT batch-observable** — the rejected client op is fire-and-forget
(`send_event` + goodbye, no recv), so the node's `Error` frame never reaches the
batch reply. The same property that makes 3045 non-observable for MP-A-02 makes
3030 non-observable for MP-A-03.

> **SUPERSEDED at impl-time (2026-06-10, empirically grounded).** The
> effect-absence premise below assumed the rejected op is fire-and-forget
> (returns `ok + event_id`). **Live `main` falsifies this:** MP-F1a (await-confirm,
> J-328) + MP-F2 (`reject_signal` wiring, J-324) now send an `Error` frame back
> for a locally-submitted reject (node app.rs:2725, carries code + event_id) →
> `send_event_confirmed` → `Rejected` → `apply_single_event_confirm` **bails**, so
> the offending op's aicontrol reply is an **error envelope** (no `event_id`
> field; wire code buried in free-text `message`). Verified by running the
> existing C6 tranche: `mp_a_02` + `mp_a_04` FAIL on HEAD ("reply has no
> `event_id`"), untouched by this arc's changes — the J-321 PASS rows are stale.
> **Joe-LOCK (2026-06-10):** the verb ships now (Option 2); the C6-oracle
> reconciliation is routed as the named finding **MP-F5** (the next arc, ahead of
> ban — ban's `MP-C-09`/`MP-A-14` witnesses inherit the identical reject-oracle
> dependency). MP-A-03 this arc = **verb shipped; batch witness pending MP-F5**;
> the node teeth stay covered by `pg13_tier1_join_into_tier2_space_rejected_3030`.
> The D-9 amendment ("reject IS batch-observable post-MP-F2") is blessed at MP-F5's
> design-lock, not here. The original recommendation is retained below as the
> grounding that surfaced MP-F5.

**Recommendation: effect-absence (Option-A paired oracle)** — the exact
treatment MP-A-02/04/20 already use in the C6 logic-adversarial tranche
(`mp_r1_c6`):
- **offending event absent everywhere:** bob's `membership.join` for the Tier-2 Space is on no node's transcript;
- **protected state unchanged + converged:** S's membership converges to `{alice:owner}` on every node — bob is never a member.

The `category=permission` / wire-3030 "why" is recorded as **C7-deferred** (it
lives on the injector/`WireActor` recv path, sibling to MP-A-05/15) — mirroring
MP-A-02's matrix note about 3045. This keeps the arc thin: MP-A-03 stays a C6
batch scenario; it does **not** need relocation to C7 + a WireActor.

**Alternative (stated for Joe, not recommended):** wire-category — assert
`category=permission`/wire-3030 directly via a C7 injector recv path. Heavier
(MP-A-03 isn't a C7 scenario today); justified only if the category assertion is
deemed load-bearing enough to outweigh the scope-creep. Default to effect-absence.

---

## 4. MP-A-03 RED-on-revert witness plan (J-323 forward rule)

The witness is the new MP-A-03 batch scenario + its paired-absence oracle.

- **GREEN (verb shipped):** alice runs `create-space --auth-tier 2` → real
  Tier-2 S persisted (`from_space_create` reads auth_tier=2). bob attempts to
  join S; the node refuses (3030, Tier-1 < Tier-2). Oracle passes: bob's join
  absent on every node **and** bob not a member of S (membership = `{alice:owner}`,
  converged).
- **RED on revert:** revert site #2 (`args.auth_tier` → literal `1` in
  `ops::create_space`). S is now created as **Tier-1**; bob (Tier-1) joins
  **successfully** → bob's join event is present **and** bob is a member. The
  paired-absence oracle **fails on both halves**. Reverting the single
  load-bearing line flips the scenario RED — a genuine witness, not a
  green-regardless tautology.

(Node-side coverage already exists via `pg13_tier1_join_into_tier2_space_rejected_3030`;
the new integration witness adds the **authoring** half — that the client can now
mint a Tier-2 Space at all.)

---

## 5. Scenario-authoring note for the design phase (not a blocker)

For bob's `membership.join` to reach the step-4 PG-13 gate it must first pass
`validate_event` steps 8–12 — which requires `prev_events` chaining off S's DAG
tip (so bob must *see* S's create). In a single-node C6 batch this is the same
flow MP-A-04 uses (carol references S's room to post). The design/runbook must
wire bob's join `prev_events` to S's tip in the batch manifest. This is the
standard C6 batch shape, not new machinery — flagged so it's grounded before the
runbook, not discovered mid-impl.

---

## 6. Design forks for Joe-lock (none pre-decided)

1. **Oracle (pivot c):** effect-absence (C6 paired) *[audit recommends]* vs
   wire-category (C7 WireActor). → §3(c).
2. **Tier value in the witness:** Tier-2 is the minimal demonstrator. Confirm
   the scenario uses `--auth-tier 2` (vs exercising 3/4) — recommend 2 (smallest
   informative; higher tiers add no coverage, only a larger integer).
3. **Cosmetic surface:** does `CreateSpaceResult` / the CLI success print echo
   the tier? (Nice-to-have; not required for MP-A-03.)
4. **Breadcrumb home (pivot b):** confirm the create-cap breadcrumb is recorded
   to the M10 ledger at close (not built in-arc).

---

## 7. Phase-0 DoD

- [x] `tasks/AUTH_TIER_VERB_AUDIT.md` v1.0 ACTIVE authored, grounded against live main.
- [x] Verb-add surface enumerated (2 substantive sites + 2 pass-throughs), wire-neutrality CONFIRMED.
- [x] Pivot (a) gate-teeth: **green-eligible** (gate has teeth, unit-proven; no-op only for lack of an authorable Tier-2 Space — which this arc fixes).
- [x] Pivot (b) creation-cap: **absent/uncapped** (grounded at `from_space_create` + `validate_event`); breadcrumb to M10, arc not widened.
- [x] Pivot (c) oracle: **effect-absence (C6 paired)** chosen + justified against D-9; wire-category noted as the alternative.
- [x] MP-A-03 RED-on-revert witness plan stated (revert `args.auth_tier`→`1` flips both oracle halves RED).
- [x] Design forks framed for Joe-lock; nothing pre-decided.

**Next:** design phase — lock the oracle fork (§6.1) + the §6.2–§6.4 calls with
Joe, then author the runbook (`tasks/AUTH_TIER_VERB_IMPL.md`), then impl → close.
Appendix F (`docs/xgen_appendix_f_en.md`) is a required close deliverable (J-323).

---

Per D-065 + D-069 + D-071 + D-074 + D-076 + D-077. MP-R1-D9 (oracle path-split)
+ MP-R1-D10 (loop-to-green) govern.
