# Multiparty Test S8 — Findings (M8 / Wave 3 / C7)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this records

The **C7** result of M8 Wave 3 (runbook `tasks/M8_MULTIPARTY_IMPL.md` §5 C7; design §3 S8 row).
S8 rides the **`--ai-mode --service` AI resident** (CP-5: the only path that holds a *live*
membership; `--aicontrol`/`--batch` are one-shot) as a first-class room member, at the
**binary level** — also the proof-of-concept for M9's AI-resident-driven load-test harness.
First baseline (no historical "A"; M8-D3). B stamp: `8b14aa8` (≡ `676b9c1`).

**Headline:** the AI resident is a **real, viable live participant** (connects, authenticates,
holds the scaffold + control pipe + health, uses the A-pure G-ALIGN apply path), **but the
binary end-to-end reply is BLOCKED by a general invitee-join bootstrap bug** (not AI-specific)
plus AI-resident node-resolution friction. Both are **M8-D4 findings / M9 inputs** (a surfaced
weakness is a success); the proper fix is a design question, so it is **recorded, not patched
in-arc**.

---

## What was run (binary, real `.exe` under `test_runs`-style sandbox)

A real node (`xgen-node --local --port 8092`) + a human client (alice) + an AI client (bob,
`init --ai`, plugin `echo`). alice created a Space + Room and invited bob; bob `join`ed Space +
Room; bob started as `--ai-mode --service`; alice sent a message mentioning bob's full identity
URI (Rail-1 trigger). Observed:

| Step | Observation | Verdict |
|---|---|---|
| bob `init --ai` + register | staged AI (`is_ai`, `plugin="echo"`), `register` sent `is_ai=true` | ✅ |
| bob `join` Space + Room | CLI printed "Joined Space/Room" — **but optimistically** | ⚠ (see Finding 1) |
| bob `--ai-mode --service` | first attempt: WS connect **refused** (Finding 2); after config fix: **connected + authenticated + initial catch-up complete** | ✅ after workaround |
| `--health` | `HEALTHY pid=… mode=ai operator_known=0/0` | ✅ liveness; ⚠ `0/0` (downstream of Finding 1) |
| alice sends mention | accepted, in the DAG | ✅ |
| bob replies | **no reply** — resident logged no inbound-event activity | ✘ blocked |
| `members` ground truth | Space has **1 member (owner only)** — bob is **not** a member | ✘ root cause |

---

## Finding 1 (M8-D4 → M9 input) — invitee one-shot `join` bootstrap bug (GENERAL, not AI-specific)

bob's joins were **rejected at the Node**:
```
WARN F-4 validation core rejected event ... event_type=membership.join
  reason=step 10: DAG structural violation — non-root event must reference at least one predecessor
```
Both bob's Space-join and Room-join carried **empty `prev_events`**, so step 10 rejected them.
bob therefore never became a member → no sync history (`new_tip=` empty) → no fan-out → no
reply.

**Mechanism (grounded, `xgen-client/src/ops.rs:770-772`):**
```rust
let prev_events = crate::batch::get_dag_tips(conn, &args.space, sync_timeout)
    .await
    .unwrap_or_else(|_| vec![args.space.clone()]);   // <- fallback fires only on Err
```
`get_dag_tips` issues a `sync_request`; the Node's `collect_sync_history` serves only the
Spaces the requester is **already a full member of**. An **invitee** (pending invite, not yet a
member) gets **`Ok(empty)`** — sync succeeds with zero events. The `vec![space_id]` fallback
fires only on **`Err`**, not empty-`Ok`, so `prev_events = []` → the join is a non-root event
with no predecessor → rejected. **The "Joined Space" CLI output is optimistic** (the M4
create-space ack carry-over: the client prints success on WS-write, not on Node accept).

**Why this is an M9 input, not an in-arc patch:** it is **general** (every invitee one-shot
join via the CLI hits it, human or AI), and the *correct* fix is a design question — should the
Node's sync serve invitees (so the invitee can chain to the invite tip), or should `ops::join`
fall back to `[space_id]` on empty-`Ok` (a client band-aid that references the create root)?
That belongs to M9 (multiparty redesign) / a join-path arc, not a mid-M8 patch of the *measured*
B binary. **It also retroactively validates the CP-4/M8-D6 decision** to home the rigorous
multiparty proofs at the workspace level: the workspace harness uses `ingest` with explicit
`prev_events`, bypassing this buggy client join path — so C2–C7's convergence/enforcement
proofs are unaffected and correct, while the binary client join is where the bug lives.

---

## Finding 2 (M8-D4 → M9-harness friction) — AI resident ignores `--node`; uses `[client].node`

bob's resident first failed to connect: `WS connect failed: … connection ... actively refused
(os error 10061)` — it dialed `ws://127.0.0.1:8080/xgen` while the Node was on 8092.

**Mechanism (grounded):** `main.rs:133` dispatches `ai_service::run(data_dir, instance,
log_level)` — the parsed **`--node` flag is never threaded into the resident**. The resident
resolves its home Node from **`[client].node` in config**, and `init --ai`'s default is
`ws://127.0.0.1:8080/xgen`. State's `home_node` (set at register to 8092) is also not used by
the resident's connect. So `--node` is **silently ignored** for `--service`, and the AI loop
dies on connect-refused while the pipe scaffold stays `HEALTHY` (misleadingly). **Workaround
(applied):** edit `[client].node` in the resident's config to the real home Node. M9's harness
must either thread `--node` into the resident or set `[client].node` per instance.

---

## S8 capability verdict + the G-ALIGN claim

- **AI resident as a live participant: VIABLE.** It connects, authenticates, completes initial
  catch-up, and holds a persistent WS + control pipe + health endpoint (the M4 resident is
  real, not a stub — distinct from the human `--service` drain-and-discard).
- **G-ALIGN for the AI's projection: HELD (referenced).** The resident's apply path is the
  R2-F01 C2 A-pure gate — `apply_or_rebuild(... derive_resolved(log, "", &HashMap::new()))`
  with vantage `""` + empty `identity_home_nodes` (`ai_service.rs`) — i.e. the *identical*
  client projection the C2 G-ALIGN proof covers (`apply_or_rebuild_*` unit tests). So once it
  holds membership, its view converges with the Node's like any client.
- **Binary end-to-end fan-out + reply: BLOCKED** by Finding 1 (the invitee never becomes a
  member). Not an AI defect — a general multiparty-membership-bootstrap bug.

Per CP-5, S8 was **not** folded into a scripted S4 variant (the live AI membership path exists);
the block is a *membership-bootstrap* bug upstream of the AI behaviour, recorded as an M9 input.

---

## M9 load-test-harness friction list (Joe's ask — what M9's multiparty-test harness inherits)

1. **`--node` not honored by `--service`** (Finding 2) — set `[client].node` per instance, or
   thread `--node` into `ai_service::run`.
2. **Invitee join bootstrap bug** (Finding 1) — blocks any invitee from becoming a member via
   the one-shot CLI; the harness cannot seat AI (or human) members via `invite`+`join` until
   fixed.
3. **Optimistic CLI acks** — `join`/`create-space` print success on WS-write, not Node-accept
   (M4 carry-over) — the harness must verify membership via `members`, not the CLI exit/print.
4. **No auto-join** — the resident only syncs Spaces it is already a member of; membership must
   pre-exist before the resident's catch-up (chicken-and-egg with Finding 1).
5. **`operator_known=N/M` / `new_tip=` as readiness signals** — both showed `0/0` / empty here
   because bob was never a member; the harness needs a reliable "joined + synced" signal beyond
   `HEALTHY` (which only proves the scaffold is up, not that the AI loop is connected).
6. **Pacing cap (2 s/Space default)** is a throughput limiter — per-Space, not per-AI; N AIs in
   one Space serialize on one clock. Load testing needs `ai_pacing_ms=0` (no CLI surface today)
   or one Space per AI.
7. **Mention trigger** — full identity URI (Rail 1) always works; `@token` (Rail 2) needs
   `[ai.behavior] mention_token` set + a restart (no hot-reload).
8. **EchoPlugin only** — fixed reply text; variable/load payloads need a custom `AiBehavior`.
9. **Reply needs a Room** (room-level message); Space-level events get no reply.

---

## The four metrics (M8-D2)

- **M1 — Delivery.** Blocked (bob not a member → no fan-out). Root cause Finding 1.
- **M2 — Convergence / G-ALIGN.** AI projection is the A-pure path (referenced C2 proof) — held
  in principle; not exercised end-to-end at binary level due to Finding 1.
- **M3 — Integrity.** The Node correctly **rejected** the malformed (empty-prev) joins — no
  corruption; the failure is loud (WARN + ERROR + reject trace), not silent.
- **M4 — Latency (informational; throughput NOT measured).** Not measured (no reply round-trip
  achieved).

---

## CP-5 disposition + CP-4 placement

CP-5 held: the AI-resident live-membership path exists, so S8 is **not** a scripted S4 variant
and **no resident mode was built**. CP-4: S8 is intrinsically binary-level (the resident is a
real process) — it was run as such; the membership-bootstrap block is the finding.

---

## Definition of Done — C7

- [x] Binary AI resident exercised (real node + human + AI clients; `--ai-mode --service`).
- [x] AI resident viability confirmed (connect/auth/scaffold/health; A-pure G-ALIGN apply path
  referenced).
- [x] **Finding 1** recorded — invitee one-shot `join` bootstrap bug (general; empty-prev →
  step-10 reject; optimistic ack) → M9 input.
- [x] **Finding 2** recorded — AI resident ignores `--node`, uses `[client].node` → M9-harness
  friction.
- [x] **M9 load-test-harness friction list** captured (Joe's ask).
- [x] CP-5 disposition recorded (live path exists; not folded into S4; no resident built).
- [x] M1–M4 recorded honestly (Rule 1 — no fabricated reply).

---

*End of MULTIPARTY_S8_findings.md — C7 complete (AI resident viable; binary end-to-end blocked
by a general invitee-join bug → M9 input).*
