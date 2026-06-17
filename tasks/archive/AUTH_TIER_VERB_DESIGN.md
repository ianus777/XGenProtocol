# Design + Runbook — Thin-verb Arc 1: `create-space --auth-tier` (MP-A-03 / PG-13)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Status

Phase-0 grounding complete ([AUTH_TIER_VERB_AUDIT.md](AUTH_TIER_VERB_AUDIT.md) v1.0).
The four design forks are **Joe-LOCKED** (2026-06-10, chat). Design + runbook are
folded into one doc per thin-arc sizing (D10/J-323 — the builder, applier, and
PG-13 gate all already shipped; only the client verb is missing).

> **SHIPPED with a scope split (Joe-LOCK 2026-06-10, Option 2).** The **verb
> ships now** (the 3 touches in §3 + the print). The **MP-A-03 batch witness is
> deferred to MP-F5** — impl-time grounding empirically falsified the AT-D1
> effect-absence oracle premise: post-MP-F1a/MP-F2 a rejected client op surfaces
> an `Error` (no `event_id` field; code in free-text `message`), so the C6 batch
> oracle is broken on HEAD (`mp_a_02`/`mp_a_04` RED, untouched by this arc; J-321
> PASS rows stale). The C6-oracle reconciliation is the named finding **MP-F5**
> (next arc, ahead of ban — ban inherits the same reject-oracle dependency).
> **MP-A-03 this arc = verb shipped; batch witness pending MP-F5;** node teeth
> covered by `pg13_tier1_join_into_tier2_space_rejected_3030`. §4 (batch witness)
> + §5 step 4 are therefore **deferred to MP-F5**; the verb's own DoD (§5 steps
> 1–3, 5–6 + build/clippy) stands. See [AUTH_TIER_VERB_AUDIT.md](AUTH_TIER_VERB_AUDIT.md)
> §3(c) SUPERSEDED note + the MP-F5 Phase-0 audit.

---

## 2. The four locks (Joe, 2026-06-10)

| Fork | Lock | Effect |
|------|------|--------|
| **AT-D1 — oracle** | **effect-absence (Option-A paired, C6 batch)** | MP-A-03 = MP-A-02/04/20 treatment: bob's join absent everywhere + bob never a member, converged. `category=permission`/wire-3030 "why" is C7-deferred (matrix note, sibling to MP-A-02's 3045 note). |
| **AT-D2 — witness tier** | **`--auth-tier 2`** | Smallest informative demonstrator; Tiers 3/4 add no coverage, only a larger integer. |
| **AT-D3 — cosmetic** | **print-only** | Echo the tier in the `cmd_create_space` success print. **No** `CreateSpaceResult` field, **no** wire/struct change. |
| **AT-D4 — creation-cap breadcrumb** | **record to M10 at close** | Pivot (b): create-event ingest is uncapped (`from_space_create` stores `auth_tier` verbatim; no validate-path gate). Breadcrumb to the M10 auth-module era; **not** built in-arc. |

No DECISIONS change (AT-D# arc-local, D-069).

---

## 3. Change surface (grounded in audit §2)

**Two substantive code touches + one print; dispatch arms are pass-through.**

1. **`CreateSpaceArgs`** ([app.rs:471](../xgen-client/src/app.rs#L471)) — add
   ```rust
   /// Auth Tier slot contract for the Space (joiners must meet this tier). Default 1.
   #[arg(long, default_value_t = 1)]
   pub auth_tier: u32,
   ```
   `u32` matches `build_space_create_event(auth_tier: u32)` + `SpaceState.auth_tier: u32` (no cast, no-drift D-067). `default_value_t = 1` ⇒ absent-flag behaviour byte-identical to today.

2. **`ops::create_space`** ([ops.rs:400](../xgen-client/src/ops.rs#L400)) — replace the literal `1` (4th arg) with `args.auth_tier`:
   ```rust
   build_space_create_event(&signing_key, &args.name, None, args.auth_tier, &home_node, None, false)
   ```

3. **`cmd_create_space`** ([app.rs:2158](../xgen-client/src/app.rs#L2158)) — AT-D3 print-only: add one line to the success block:
   ```rust
   println!("  Auth Tier:  {}", args.auth_tier);
   ```

**No change:** `batch::dispatch_line` `CreateSpace` arm ([batch.rs:349](../xgen-client/src/batch.rs#L349)) + the `cmd_create_space` threading both forward the whole `&args` struct — the new clap field rides along. `--help` auto-generates from the clap doc-comment.

**Wire-neutral** (audit §2): `auth_tier` already rides signed event content; this changes a value, not the wire shape.

---

## 4. Witness — MP-A-03 scenario (AT-D1 + AT-D2)

New C6 batch scenario `MP-A-03/*` + manifest, joining `mp_r1_c6`. Shape (mirror
MP-A-04's single-node batch, paired-rejection oracle):

- **alice** (owner) runs `create-space --auth-tier 2` → real Tier-2 S persisted (`from_space_create` reads `auth_tier=2`).
- **bob** (Tier-1, no assertion → `assertion_tier_of` = 1) submits `membership.join` for S, `prev_events` chained off S's DAG tip (so `validate_event` steps 8–12 pass and dispatch reaches the step-4 PG-13 gate). *(Scenario-authoring note, audit §5: bob's join must reference S's tip — standard C6 batch wiring.)*
- Node refuses at step 4 → `Rejected` wire **3030 `tier_mismatch`**.

**Oracle (effect-absence paired):**
- bob's `membership.join` event **absent** from every node's transcript;
- S's membership **converges to `{alice:owner}`** on every node — bob never a member.

**RED-on-revert (J-323 forward rule):** revert site #2 (`args.auth_tier` → `1`).
S is created Tier-1 → bob (Tier-1) joins **successfully** → join present **and**
bob a member. Both oracle halves flip RED. Genuine witness, not a
green-regardless tautology.

---

## 5. Runbook (single commit)

1. Add the `auth_tier` clap field (§3.1).
2. Thread `args.auth_tier` into `build_space_create_event` (§3.2).
3. Add the print-only line (§3.3, AT-D3).
4. Author the `MP-A-03/*` batch + manifest + `mp_r1_c6::mp_a_03_*` runner asserting the §4 paired oracle.
5. Update the matrix MP-A-03 row 🚧 BLOCKED → ✅ PASS, with the C7-deferred category note (AT-D1).
6. **Appendix F** ([docs/xgen_appendix_f_en.md](../docs/xgen_appendix_f_en.md)) — `create-space` gains the `--auth-tier <n>` arg (default 1); required close deliverable (J-323).

**Verification:**
- `cargo build` 0 + `cargo clippy -- -D warnings` clean (default + `--all-features`).
- Full suite green; the new `mp_a_03_*` runner GREEN.
- RED-on-revert confirmed (§4) — neuter `args.auth_tier` → join admitted → oracle RED; restore → GREEN.

**DoD:**
- [ ] 3 code touches landed (clap field, ops literal swap, print line).
- [ ] `MP-A-03/*` scenario + runner GREEN; paired oracle asserts join-absence + membership-converged-without-bob.
- [ ] RED-on-revert witness demonstrated.
- [ ] Matrix MP-A-03 → ✅ PASS (category-why note C7-deferred).
- [ ] Appendix F updated (`--auth-tier`).
- [ ] M10 create-cap breadcrumb recorded at close (AT-D4).
- [ ] Build 0 + clippy clean + suite green.

(No "commit pushed" item — `Status: COMPLETED` is the shipped signal. Clair's
code commit precedes Chat's doc-bridge.)

---

Per D-065 + D-069 + D-071 + D-074 + D-076 + D-077. MP-R1-D9 (oracle path-split)
+ MP-R1-D10 (loop-to-green) govern. AT-D# arc-local (D-069).
