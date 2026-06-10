# Handoff — Thin-verb Arc 1: `create-space --auth-tier` (D-071 Phase-0)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. For Clair — what this is

The first of the four thin-verb arcs (order Joe-LOCKED at J-334:
**auth-tier → ban → room_update → thread×3**). This note is the grounding
checklist for the **D-071 Phase-0 audit** — it points at where to look and
what to answer; it does **not** pre-decide anything (the verdict + the design
forks are yours to ground and Joe's to lock). Authored by Chat ahead of the
arc per the handoff pattern; consume it (Status → COMPLETED) when the Phase-0
ships.

**Rule 0 entry order:** CLAUDE.md PLAY → JOURNAL J-334 → this handoff →
`docs/tests/MULTIPARTY_TEST_MATRIX.md` (MP-A-03 row) → `xgen-client/src/ops.rs`
(`create_space`, the verb-add pattern) → `tasks/MP_R1_DETERMINISTIC_DESIGN.md`
§11 (D10) + §10 (D9, the oracle path-split).

Deliverable: `tasks/AUTH_TIER_VERB_AUDIT.md` v1.0 (ACTIVE), then design →
Joe-lock → runbook → impl → close. Appendix F (`docs/xgen_appendix_f_en.md`)
is a required close deliverable (J-323). The arc flips **MP-A-03** with a
genuine RED-on-revert witness.

---

## 1. Arc scope (grounded at the lock)

Unblocks **MP-A-03** (tier-gate join refusal, PG-13). The xgen-core builder
already exists and already takes the tier:

- `build_space_create_event(signing_key, name, _, auth_tier, home_node, _, _)`
  — `auth_tier` is the **4th positional arg**.
- `ops::create_space` (xgen-client/src/ops.rs) currently passes the literal
  **`1`** in that slot (the matrix cites ops.rs:357 — confirm by reading, line
  numbers drift).

Verb-add shape (mirror the existing verbs in ops.rs / app.rs / batch.rs):
1. add `--auth-tier <u8>` to `CreateSpaceArgs` (clap, in `app.rs`) — **default 1**
   so current behaviour is byte-identical when the flag is absent;
2. thread `args.auth_tier` into the `build_space_create_event` 4th arg;
3. the `batch::dispatch_line` arm for `create-space` (carry the new field);
4. the CLI shim (`app::cmd_create_space` or equivalent) + `--help`.

No wire-format change is expected (auth_tier already rides the event content);
**confirm** that and flag if not (a wire change would need a separate Joe-lock).

---

## 2. The three Phase-0 pivots (the grounding work)

These were flagged at the J-334 lock. The audit must answer all three —
they decide whether MP-A-03 greens or routes a finding, and they shape the
oracle before any code is written (not mid-impl).

### Pivot (a) — gate-teeth [the pivot that decides green-vs-route]
The matrix says the PG-13 join-gate is "a genuine Tier-1 no-op today"
(cites runtime.rs:1155 — grep, don't trust the line). **Ground:** does the
PG-13 gate actually enforce at `auth_tier ≥ 2` (refuse a join that lacks the
required tier claim), or is it inert at every tier?
- If it has teeth → MP-A-03 can green once a Tier-2 space is creatable.
- If it is inert → creating the Tier-2 space is authorable but the *refusal*
  never fires → MP-A-03 **routes a finding** (surface-and-route, D-065), and
  the verb still ships (it's correct; the gate is the gap). Either outcome is
  fine — the point is to know **before** committing to "MP-A-03 will pass."

### Pivot (b) — creation cap
Does the node authorize *creating* a Tier ≥ 2 space, or does it accept whatever
tier the client signs into the create event? Trace the create-event ingest /
validate path for any tier check. If uncapped, that is **not** an auth-tier-arc
defect — record it as a breadcrumb for the **M10 auth-module** era (where tiered
attestation actually lands) and move on. Do not widen this arc to add a cap.

### Pivot (c) — oracle shape [decide before authoring the scenario]
MP-A-03's matrix expectation is "refusal multiparty-visible + converged;
`category=permission`." Per **MP-R1-D9** (design §10), `category=permission`
is **NOT batch-observable** — the rejected client op is fire-and-forget
(`send_event` + goodbye, no recv), so the node `Error` never reaches the batch
reply. So decide now which oracle MP-A-03 uses:
- **wire-category** (assert `category=permission` / the wire code) → needs the
  C7 injector/`WireActor` recv path, like MP-A-05/15; **or**
- **effect-absence** (offending join absent everywhere + protected state
  unchanged) → the MP-A-02/04/20 Option-A paired-oracle treatment.
Pick the one the rails actually support and state it in the design; don't
discover it mid-impl.

---

## 3. Phase-0 DoD

- [ ] `tasks/AUTH_TIER_VERB_AUDIT.md` v1.0 ACTIVE authored, grounded against live main.
- [ ] Verb-add surface enumerated (the 4 sites in §1), wire-neutrality confirmed (or flagged).
- [ ] Pivot (a) gate-teeth: verdict grounded — green-eligible or routes-a-finding.
- [ ] Pivot (b) creation-cap: present/absent grounded; if absent, breadcrumb to M10 (not widened here).
- [ ] Pivot (c) oracle: wire-category vs effect-absence chosen + justified against D-9.
- [ ] MP-A-03 RED-on-revert witness plan stated (what is RED when the verb is reverted).
- [ ] Design forks framed for Joe-lock; no Joe-lock item pre-decided.

(Per the standing convention: no "commit pushed" checklist item — `Status: COMPLETED`
is the shipped signal. Clair's code/arc-doc commit precedes Chat's doc-bridge.)

---

Per D-065 + D-069 + D-071 + D-074 + D-076 + D-077.
