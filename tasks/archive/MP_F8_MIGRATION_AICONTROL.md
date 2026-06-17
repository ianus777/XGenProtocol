# MP-F8 — expose `migration initiate` over `--aicontrol` (UNFENCED) — FOLDED ARC-DOC (audit · design · runbook)

> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Discipline note

A **one-line production-crate change** (`xgen-node`), so audit + design + runbook are **folded into
this single doc** (no three-doc ceremony, Joe-lock). MP-R2 fix-phase gate item (J-344): closing it
unblocks the **MP-C-16** migration row. The fence shape was the one real design fork — **LOCKED
unfenced** (Joe, J-346) — grounded below. **No code until this runbook (§3) is Joe-locked.**

**Scope guard (J-344):** gate item is MP-F8 only — no drift to F7; anything new is faced-and-routed to
its own home, does not re-open the {F8, F7} gate.

---

## 1. Audit — the gap, grounded against live `main`

**Symptom (RUN-empirical, MP-C-16 / `mp_r2_fixed` C6c):** `migration initiate` over `--aicontrol` →
`UNKNOWN_COMMAND`, `category:argument`, "command is not available over --aicontrol". The test's own
`"did you build --features harness-control?"` hint is a **red herring** (the fence is intact — C6a drove
the fenced `add-peer` green).

**Root cause (grounded):** the verb **exists and is wired everywhere except the aicontrol dispatch.**
- `admin_ops::migration_initiate` — `async`, `MigrationInitiateArgs { space_id, destination_id,
  destination_url }` (derives `serde::Deserialize`) → `MigrationInitiateResult` (derives `Serialize`);
  Arc F / PG-11 / AF-D7 (`admin_ops.rs:2081`). Detached `run_source_migration` (propose → transfer →
  verify → cutover) signed by this Node.
- Already on the **CLI + pipe** surface (`pipe.rs:682`, `AdminCommand::Migration(Initiate)`).
- **Never added to the aicontrol `run_verb` match** (`aicontrol.rs:355-404`) → falls to the catch-all
  `_ => UnknownCommand` "command is not available over --aicontrol" (`aicontrol.rs:401-404`).
- **Mechanical cause of the omission:** Arc F (J-252, 2026-06-04) shipped `migration_initiate` **after**
  the M7 aicontrol dispatch was authored (J-204, 2026-06-01), so the verb postdates the match and was
  simply never appended. Not a deliberate fence decision.

**Ctx-completeness (the one honest wrinkle — answered at grounding, NOT deferred):** `migration_initiate`
needs from `ctx`: `require_runtime(..)`, `ctx.federation_peer_senders`, `ctx.federation_policy` +
`federation_policy_path()`, `ctx.config_path` / `ctx.data_dir` (`admin_ops.rs:2086-2140`). The aicontrol
`build_ctx` already threads **all** of these — `.with_runtime` + `.with_federation_senders`
(= `federation_peer_senders`) + `.with_federation_policy`, over `AdminContext::batch(&deps.data_dir,
&deps.config_path, ..)` (`aicontrol.rs build_ctx`). The aicontrol surface already drives runtime-mutating
WRITE verbs through the same ctx (`space force-eject`, `identity revoke`, `federation defederate`). → the
arm works with the existing ctx; **no extra threading.** Genuinely one arm.

---

## 2. Design — LOCKED (Joe, J-346)

**MIG-D1 — UNFENCED** (the fence fork, LOCKED). The M9.2 `#[cfg(feature = "harness-control")]` fence
(`aicontrol.rs:391-400`) is documented for **harness seams only** — peer-spoof (`add-peer`) /
clock-tamper (`clock`), "provably absent from production" (M9.2-D1). `migration initiate` is **not** a
fabrication seam — it is a **real production admin verb**, already unfenced on CLI/pipe, sibling to the
unfenced `federation initiate` (`aicontrol.rs:369`) among the ~33 unfenced production verbs (identity
revoke, space force-eject, federation defederate, …). It joins them **unfenced**. The J-344 route-note's
"fenced like add-peer/clock" conflated a production verb with the harness-only seams (the conflation
caught at grounding, D-065). Fencing it would make migration the **only** production verb fenced out of
aicontrol while reachable on CLI/pipe — an anomaly; and the aicontrol security posture already exposes
destructive production verbs unfenced. *(Either choice drives MP-C-16 — the harness uses a
harness-control build regardless; the locked difference is that migration-over-aicontrol now ships in a
release build too, consistent with its own CLI/pipe exposure.)*

**MIG-D2 — homogeneous arm.** The arm is byte-for-shape identical to the existing async production arms;
`migration_initiate`, `add-peer`, `clock_*` are all `async` and the `cap!` macro
(`$e.await.map(to_val).map_err(..)`) awaits uniformly — no async wrinkle.

**MIG-D3 — Appendix F does NOT fire.** Appendix F documents **client CLI verbs** (ban / room_update /
thread); `migration initiate` is a **node admin verb** whose canonical home is
`xgen_node_admin_ops_design.md` §6 + Appendix K — already documented by Arc F. MP-F8 adds an aicontrol
**dispatch arm for an existing verb**, not a new verb → the J-323 rule (CLI-verb → Appendix F) does not
fire. (Confirmed at design-lock.)

---

## 3. Runbook — the one arm (Joe-LOCKED 2026-06-11)

### C1 — add the unfenced aicontrol arm (`xgen-node`, production)

**Change.** One match arm in `run_verb` (`aicontrol.rs`), placed among the **unfenced** production verbs
(natural home: adjacent to `federation initiate` :369, or in the federation/migration grouping before the
fenced block at :391):

```rust
"migration initiate" => cap!(admin_ops::migration_initiate(&mut ctx, de(args)?)),
```

No fence attribute. No ctx change (§1 ctx-completeness). No `admin_ops` change. No clap/CLI/pipe change.

**Named test (D-078):** a dispatch unit (sibling to the existing aicontrol dispatch tests, e.g. the
`UNKNOWN_COMMAND` assertion at `aicontrol.rs:652`) asserting **`"migration initiate"` resolves — does NOT
return `UNKNOWN_COMMAND`** — in a **default** build (no `--features harness-control`). This is the
inverse of the M9.2 fence-test: migration is present in a default build precisely because it is unfenced
(MIG-D1). (A missing/malformed-args case may reach `migration_initiate` and fail downstream — that is
*not* `UNKNOWN_COMMAND`; the test targets dispatch resolution, not a full migration drive.)

**DoD (C1):**
- `cargo build -p xgen-node` 0-error in **both** default **and** `--features harness-control`.
- `cargo clippy -p xgen-node --lib --tests --all-features -- -D warnings` clean.
- `cargo test -p xgen-node` 0-failed; the new dispatch unit GREEN.
- **Prime invariant:** the existing aicontrol dispatch tests + the M9.2 fence test stay green (the
  addition is one unfenced arm; the fenced block and the catch-all are otherwise untouched).
- No production behavior change beyond making the existing `migration_initiate` verb reachable over
  aicontrol.

**Doc note (Chat seat, NOT this commit):** optional one-line entry for `migration initiate` in the
aicontrol verb list of `docs/xgen_aicontrol_implementation.md` — Joe/Chat handle it in the close bridge
(noted here so it isn't lost). Appendix F is out (MIG-D3).

### The witness (box-gated R2 rerun, separate)

MP-C-16 (`mp_r2_fixed` migration row) — `migration initiate` now drivable over aicontrol → the row greens
at the box-gated rerun (re-run the affected smoke to green-to-criterion; **rebuild harness-control AFTER
any `--workspace` build** — the J-315/J-340 clobber fence). This is the gate-close witness; coordinate
with Joe on the box, sibling to the MP-F9/F10 rerun.

---

## 4. Scope / honest boundary / what this does NOT do

- **No code until §3 is Joe-locked.** Production-crate change → arc discipline (even at one line).
- **UNFENCED is locked** (MIG-D1) — do not fence it; do not "fix" the route-note's framing back in.
- **One arm only** — no `admin_ops` / clap / CLI / pipe / ctx change (all already in place). If
  implementation surfaces a ctx gap the grounding missed (§1 says none), **STOP + surface** — do not
  silently thread a new handle.
- **Does NOT** touch F7 or re-open the {F8, F7} gate. Does NOT alter the M9.2 harness-control fence
  (add-peer / clock stay fenced).
- **MP-C-16 greens at the rerun, not at C1** — C1 ships the capability; the box-gated rerun is the
  witness (sibling to the MP-F9/F10 pattern).

**Confirm-at-pickup (D-078) — all answered at grounding, re-verify against live `main` at C1 start:**
- The exact insertion line among the unfenced arms (line numbers shift; place adjacent to `federation
  initiate`).
- `build_ctx` still threads runtime + federation_senders + federation_policy (§1) — the pipe path
  (`pipe.rs:682`) is the working reference for what `migration_initiate` consumes.

---

*Per D-065 (surface, don't carry the route-note's framing) + D-069 (arc-local MIG-D#) + D-071 (runbook
follows the locked design) + D-074 (per-commit atomicity) + D-077 (prime invariant) + D-078
(confirm-at-pickup) + the J-344 BOUNDED-gate criterion + M9.2-D1 (the fence is for harness seams, which
migration is not).*
