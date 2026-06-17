# Phase-0 Audit — Thin-verb Arc 2: `ban` (MP-C-09 + MP-A-14)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The D-071 Phase-0 audit for the **second** thin-verb arc (order Joe-LOCKED:
auth-tier → MP-F5 → **ban** → room_update → thread×3). Grounds the verb-add
surface + the three pivots Joe flagged, frames the forks for Joe-lock. No code,
nothing pre-decided.

Arc goal: ship a client `ban` verb so an admin can author a member-initiated
`membership.ban`, unblocking **MP-C-09** (ban → converge → post-rejected) +
**MP-A-14** (ban-evasion via new identity). Same thin shape as auth-tier — the
builder, the ban-cascade applier, and the `can_ban` gate all already shipped
(`build_membership_event(MembershipBan)` + `apply_ban`, the latter proven on the
MP-F4 ban-vs-join Layer-1 spine); **only the client verb is missing**.

**MP-F5 is the load-bearing inheritance (lean on it explicitly).** This arc is
sequenced *after* MP-F5 precisely so the post-ban-reject witnesses land green on
the **assert-the-reject** oracle (reject_code present + event-absent + state-
unchanged) instead of as fire-and-forget debt. Pivot (b) confirms that inheritance
is real — not assumed.

---

## 2. Verb-add surface (thin — client verb only; mirror `ops::invite`)

`build_membership_event(key, space_id, room_id, EventType::MembershipBan, content)`
([state.rs:1918](../xgen-core/src/space/state.rs#L1918)) already exists; `content`
carries `{"target_identity": <id>}` (read by `apply_ban` at [state.rs:1071](../xgen-core/src/space/state.rs#L1071)).
There is **no** client `ban` verb today (the `MembershipBan` hits in xgen-client
are tests/AI-behaviour, e.g. ops.rs:2257 is a unit test). Mirror `ops::invite`
(the closest sibling — an admin authoring a membership event about a *target*
identity, then send-confirm):

| # | Site | Change |
|---|------|--------|
| 1 | `BanArgs` (new clap struct, app.rs) | `--space <id>` + `--identity <target>` (space-level ban; room dimension omitted — ban cascades all rooms). Mirror `InviteArgs`. |
| 2 | `ops::ban` (new, ops.rs) | build `MembershipBan` with `target_identity`, chain off the Space tip (`get_dag_tips`, like `invite`/`join`), sign, `send_event_confirmed` → `apply_single_event_confirm("ban")` — **the MP-F5 single-event site** (so a refused ban surfaces structurally too). |
| 3 | `ClientCommand::Ban` + CLI shim + batch arm | pass-through `&args` (same as create-space — the dispatch arms forward the struct; `--help` auto). |

**Wire-neutral** (the ban event already rides signed content; no canonical/wire
change). Same "thin verb over an existing builder" shape as auth-tier — risk is
the scenario witnesses, not the verb.

---

## 3. The three pivots

### Pivot (a) — `can_ban`/role gate → **HAS TEETH (real gate, like PG-13)**

`can_ban` ([membership.rs:141](../xgen-core/src/space/membership.rs#L141)) is
**Admin+** — a Moderator cannot ban (unit-pinned: membership.rs:196/201). Enforced
at **two** sites:
- `check_permission` during `validate_event` ([exchange.rs:886](../xgen-core/src/message/exchange.rs#L886), via `RoomPermission::Ban`): non-`can_ban` sender → `ExchangeError::PermissionDenied`;
- `apply_ban` re-checks defensively ([state.rs:1067](../xgen-core/src/space/state.rs#L1067)).

So an unauthorized ban (member/moderator) is **rejected at validation** — a real
gate, not a no-op. (This isn't the MP-C-09/A-14 witness directly — both use an
*admin* banner so the gate passes — but it grounds that the authority model has
teeth, and it's a candidate negative witness if Joe wants one.)

### Pivot (b) — MP-C-09 post-ban-reject inherits MP-F5 → **CONFIRMED REAL (same path)**

`apply_ban` ([state.rs:1064](../xgen-core/src/space/state.rs#L1064)) removes the
target from `members`, `pending_invites`, **and every room's members**, and inserts
into `banned`. So a banned Bob is a non-member. His subsequent post goes through
**`ops::send` → `apply_single_event_confirm("send")`** ([ops.rs:1144](../xgen-client/src/ops.rs#L1144))
— the exact single-event confirm site MP-F5 fixed. The node rejects it at
`validate_event` step-11 (non-member sender); the reject is locally-submitted →
`reject_signal` → `Error` frame → `EventConfirm::Rejected` → `VerbReject` →
`reject_code` + `event_id` surfaced in the aicontrol reply.

**The inheritance is real, on the path MP-F5 fixed** — not a different path.
MP-C-09's post-ban-reject is now **assert-the-reject** observable (the C6/MP-F5
oracle applies). Expected wire code: step-11 non-member → **4000** (the unmapped
variant, same as MP-A-04/MP-A-20 — MP-F2-followon; pin to observed at impl).

### Pivot (c) — MP-A-14 ban-evasion → **green on the enforceable half + M10 breadcrumb (record-behaviour, as J-334 flagged)**

> **SUPERSEDED-IN-PART at impl (mechanism correction, empirically grounded).** The
> "same identity re-join → REFUSED → **assert-the-reject**" framing below is wrong
> on mechanism. `dispatch_event` has **no banned pre-check**; `apply_join`'s
> `Banned` refusal is at apply and is **swallowed** (runtime.rs:691,
> `let _ = state.apply_event(...)`). So a banned re-join is **accepted-but-inert**:
> Ok at dispatch (empirically `is_ok=true`), but dropped at resolution (M8 Layer-1
> ban>join) → bob not re-admitted. A-14's enforceable green is therefore
> **membership-effect-absence** (bob ∉ resolved members), **not** assert-the-reject.
> (MP-C-09's post-ban *send* IS a genuine assert-the-reject — step-11 in
> `validate_event`, not swallowed; that's where the MP-F5 inheritance holds, pivot
> (b).) The SUBSTANCE of pivot (c) is intact (bob not re-admitted; fresh identity
> joins → M10 breadcrumb); only the green's mechanism is corrected. See
> BAN_VERB_DESIGN.md §4/§5.

Ban is keyed per **`IdentityXgid`** (`banned: HashSet<IdentityXgid>`,
[state.rs:230](../xgen-core/src/space/state.rs#L230)). Two distinct behaviours,
both grounded:
- **Same identity re-join → REFUSED.** `apply_join` consults `banned`
  ([state.rs:1003](../xgen-core/src/space/state.rs#L1003)) → `SpaceError::Banned`.
  So the banned identity cannot rejoin — a clean assert-the-reject green
  (inherits MP-F5: the refused re-join surfaces a reject_code + the join is absent
  + Bob stays out).
- **Fresh identity → JOINS (recorded behaviour).** A new keypair is a different
  `IdentityXgid`, not in `banned` → it open-joins normally (Tier-1 open-join, J-275).
  The protocol does **not** link the fresh identity to the banned person — by
  design (pseudonymity). Cross-identity ban-evasion detection is **M10 auth-module
  / reputation** territory, not protocol.

So MP-A-14 lands as: a **green witness** for the per-identity ban enforcement
(banned identity refused) **+ a recorded-behaviour note + M10 breadcrumb** that
cross-identity ban-evasion is out of protocol scope (the matrix's "treated as a
new identity… no automatic re-entry; recorded behaviour" is exactly this). It is
**not** a clean "evasion blocked" pass — and the audit states that up front, as
J-334 anticipated. Do not widen this arc to add cross-identity linkage.

---

## 4. Two-row flip + RED-on-revert (J-323 forward rule — one witness per row)

**MP-C-09** (ban → converge → post-rejected):
- alice (owner) creates S + room, invites bob, bob joins → alice **bans** bob → bob attempts a post.
- **Oracle (MP-F5 assert-the-reject + paired):** bob's post reply is an Error with `reject_code` (≈4000) + `event_id`; the post event is absent on every node; membership excludes bob (ban applied, converged).
- **RED-on-revert:** revert the `ban` verb (no ban) → bob stays a member → his post is accepted (reply Ok, present in `.events`) → the reject-assert + absence + exclusion all flip RED.

**MP-A-14** (ban-evasion via new identity):
- precondition: bob banned (the new verb). bob's **original** identity attempts re-join → refused (`Banned`); bob registers a **fresh** identity → joins.
- **Oracle:** the original-identity re-join is reject-surfaced + absent + bob not re-added (green); the fresh identity joins (recorded behaviour — asserted as "joins as a new member; no cross-identity block", the M10 breadcrumb).
- **RED-on-revert:** revert the `ban` verb → bob's "re-join" is a plain join (no refusal) → the refusal-assert goes RED.

---

## 5. Forks for Joe-lock (recommendations; none pre-decided)

- **F-BAN-1 — MP-C-09 topology.** Single-node (ban + post-ban-reject + local
  membership-excludes-bob) vs cross-node (ban converges A↔B, bob's post excluded on
  both). *Lean: single-node* — the post-ban-reject (the MP-F5 inheritance, the
  reason this arc follows F5) is node-local; cross-node ban-convergence rides the
  already-proven MP-C-02/F1b machinery and adds federation-harness cost for little
  R1-floor gain. Cross-node is available (C5 has C-10) if you want the fuller
  "converge on every node" assertion.
- **F-BAN-2 — MP-A-14 assertion shape.** (a) assert the enforceable green
  (same-identity re-join refused, assert-the-reject) + record the fresh-identity
  join as behaviour + M10 breadcrumb *[lean]*; vs (b) a looser record-only
  scenario. Lean (a) — it has a genuine RED-on-revert and states the cross-identity
  limit honestly.
- **F-BAN-3 — tranche placement.** MP-C-09 → C5 (membership-lifecycle); MP-A-14 →
  C6 (logic-adversarial, the tranche MP-F5 just reconciled). *Lean: as stated* —
  each row lands in its natural tranche; both inherit the MP-F5 assert-the-reject
  oracle.
- **F-BAN-4 — negative gate witness (optional).** Whether to add a moderator-bans
  → `PermissionDenied` witness (pivot a). *Lean: skip for this arc* — MP-C-09/A-14
  don't need it; it's a separate can_ban assertion, not in the two BLOCKED rows.

---

## 6. Phase-0 DoD

- [x] Verb-add surface enumerated (client verb only; mirror `ops::invite`); wire-neutral.
- [x] Pivot (a) can_ban gate: **has teeth** (Admin+, enforced at check_permission + apply_ban).
- [x] Pivot (b) MP-C-09 reject inherits MP-F5: **confirmed real** — banned-sender post routes through `ops::send` → `apply_single_event_confirm` (the MP-F5 site); reject surfaced.
- [x] Pivot (c) MP-A-14: **green on same-identity refusal + M10 breadcrumb** for cross-identity (record-behaviour, not a clean evasion-blocked pass); grounded, not assumed.
- [x] Two-row flip (MP-C-09 + MP-A-14), each with a RED-on-revert witness stated.
- [x] Forks framed for Joe-lock (topology · A-14 shape · tranche placement · optional negative gate); nothing pre-decided.

**Next:** design phase — lock F-BAN-1..4, author the folded runbook, impl → close.
Appendix F gets the `ban` verb entry (close deliverable, J-323). No DECISIONS
change (BAN-D# arc-local, D-069).

---

Per D-065 + D-069 + D-071 + D-074. MP-R1-D9 (amended favorably at MP-F5 — the
assert-the-reject oracle this arc inherits) + MP-R1-D10 (loop-to-green) govern.
