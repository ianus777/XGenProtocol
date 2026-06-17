# M8.6 — Federation Stress: Phase-0 Audit (clock-injection seam)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose & scope

Phase-0 (D-071) audit opening **M8.6 — Federation stress**. Grounds the **clock-injection seam** against the as-built scheduler / reconnect / F-10 time-read sites, and against the four deferred Phase-9 compounds **C1 / C4 / C6 / C8** (`tasks/FEDERATION_STRESS_FOLLOWON.md`). Output feeds design → Joe-lock → runbook → Clair.

This audit re-grounds against live `main`. The follow-on stub's file:line references are from 2026-05-19 (pre-M6/M7/M8); they are superseded by §3 below. Every claim here is cited file:line against the current tree.

**Activation check.** The stub's go-ACTIVE gates are all satisfied: M6 (admin write-path verbs) shipped, M7 (`--aicontrol`) shipped, and the CSCA-equivalent whole-codebase audit shipped as Round 2 (J-258, GO). M8.6 is correctly ACTIVE-eligible.

## 2. Method

Grep of every wall-clock / monotonic / tokio-timer read across `xgen-common`, `xgen-core`, `xgen-node`, `xgen-client` (patterns `Utc::now()`, `SystemTime::now()`, `Instant::now()`, `tokio::time`), then line-level reading of the two load-bearing surfaces: `xgen-node/src/reconnect.rs` (scheduler + ladder) and `xgen-core/src/dag/pending.rs` (F-10 HeldPending window), plus the `mark_lost` / `mark_active` convergence sites in `xgen-node/src/app.rs`. Suite baseline **1193/0/2**; no code in this phase.

## 3. As-built findings

### 3.1 Headline — both leaf layers already parameterise time

The stub assumed "every wall-clock call site rewires." Not so. The two subsystems the compounds stress **already take time as a function parameter at the leaf**; the seam burden is at a small set of top callers, not pervasive.

- **Reconnect registry** — `FederationRegistry::due_for_reconnect(now)`, `mark_lost(now)`, `update_next_reconnect(next_at)` all take `DateTime<Utc>` (consumed in `reconnect.rs:scheduler_tick` and `app.rs`). The wall-clock read happens at the *caller*, inline.
- **Pending buffer** — `PendingBuffer::drain_timed_out(now: Instant, federation_relationship_timeout: Duration)` already takes the sweep `now` (`pending.rs:~430`). The only production monotonic read is `received_at: Instant::now()` stamped inside `add()` (`pending.rs:192`).

Consequence: the injection points are leaf-callers, and the seam is **surgical**, not architectural-rewrite-shaped.

### 3.2 The injection surface (in-scope)

| # | Site | Domain | Compound(s) | Inject via |
|---|------|--------|-------------|-----------|
| 1 | `xgen-node/src/reconnect.rs:150` — `let now = Utc::now();` (scheduler_tick) | W | C4 ladder | `Clock::now_utc()` |
| 2 | `xgen-node/src/reconnect.rs:106` — `tokio::time::sleep(SCHEDULER_TICK_SECONDS)` (tick cadence) | T | C4 | `tokio::time::pause/advance` (no trait) |
| 3 | `xgen-core/src/dag/pending.rs:192` — `received_at: Instant::now()` (in `add()`) | M | C1, C6 (30 s F-10) | `Clock::now_instant()` |
| 4 | `xgen-node/src/app.rs:2309` — `reg.mark_lost(&peer_node_id, Utc::now())` (all five session-end paths converge) | W | C1, C4 | `Clock::now_utc()` |
| 5 | `xgen-node/src/app.rs:2183` — `let now = Utc::now();` (`mark_active` / `last_connected` stamp; handshake-ACTIVE convergence, both roles) | W | C1, C4, C8 | `Clock::now_utc()` |
| 6 | `xgen-core/src/federation/handshake.rs:426` — `tokio::time::timeout(WAIT_TIMEOUT_SECS, …)` | T | C8 | tokio test clock |

Sites 1/4/5 are W-domain caller reads feeding the already-parameterised registry. Site 3 is the single M-domain production read (the sweep at site-adjacent `drain_timed_out` is already parameterised). Sites 2/6 are T-domain timers already controllable by `tokio::time` test machinery.

### 3.3 Three time domains (non-interconvertible)

The federation surface reads **two clocks plus a timer**; a single `now()` is insufficient.

- **W — wall-clock `chrono::DateTime<Utc>`.** Ladder math + `mark_lost` / `mark_active`. Must be absolute calendar time: `next_reconnect_attempt` is **persisted** as RFC3339 and survives restart (`reconnect.rs` ladder math `now + chrono::Duration::minutes(step)`).
- **M — monotonic `std::time::Instant`.** F-10 HeldPending 30 s window. Must be jump-immune: an NTP step or DST change must not falsely expire or hold a pending event.
- **T — tokio timers.** One recurring 60 s tick loop (`reconnect.rs:106`) + transient one-shot handshake `timeout`s. Already controllable by `tokio::time::pause()/advance()`.

W and M cannot be derived from one another in production (different guarantees); collapsing them would force one to abandon its defining property. **They are point reads, not running cycles** — W/M cost one clock syscall each (W once per tick + per session-end/ACTIVE; M once per pending `add()`). The only scheduled wakeup in the whole surface is the single T-domain 60 s loop. **Resource cost of the seam: negligible.**

### 3.4 Out-of-scope reads — the seam fence

The workspace has ~90 further `Utc::now()` reads. They are **event-timestamp stamping**, not federation-stress timing, and stay OUT of the seam (clean arc, not grab-bag — J-284):
- identity registration timestamps (`identity/registration.rs`), message-exchange `ts` (`message/exchange.rs`), admin-audit stamps (`admin_ops.rs`), fanout `valid_until` (`fanout.rs`), state/announcement/auth stamps.

These do not gate C1/C4/C6/C8 and dragging them in would expand the seam without serving any compound. **Fence = the six sites in §3.2 only** (Q2 to confirm at lock).

### 3.5 Serde — no persisted Clock (stub worry resolved)

The stub flagged "serde behaviour if Clock is in any persisted struct." It is not: `next_reconnect_attempt` persists as an RFC3339 **string**; the backoff cursor is **in-memory only** (Joe-locked option α, `reconnect.rs` `AttemptCursor`). `Arc<dyn Clock>` is threaded through calls, never serialized. **No serde design needed.**

### 3.6 Backoff-cursor restart semantics (C4-relevant)

`AttemptCursor = Arc<Mutex<HashMap<NodeXgid, u32>>>` is transient (option α): on restart the cursor resets; the persisted `next_reconnect_attempt` still governs *when* the first post-restart attempt fires, but ladder progression then follows the fresh cursor (documented "aggressive post-restart probe"). C4's drop/recover-×5 invariant checks must assert against this documented behaviour, not against a persisted-cursor model.

## 4. Design decision — Fork A LOCKED (this session)

Three forks were framed; **Fork A is Joe-locked**:

- **A (LOCKED) — minimal `Clock` trait, two methods (`now_utc` + `now_instant`) + lean on `tokio::time` for the T domain.** Trait feeds sites 1/3/4/5; sites 2/6 use the tokio test clock. Rationale: smallest production change; keeps `xgen-core/dag/pending.rs` tokio-free (it is a pure DAG logic component).
- B (rejected) — migrate `pending.rs` to `tokio::time::Instant` so pause/advance covers M. Rejected: couples a core logic type to tokio.
- C (rejected) — full `Clock` spanning sleep/timeout too. Rejected: heaviest rewire, overkill.

**MockClock shape LOCKED (Joe's offset refinement this session):** the production two-read split is kept (correct + cheap), but the *test* mock uses a single cursor + offsets so one knob drives both derived reads —
```
MockClock { base_utc, base_instant, cursor: Duration }
  advance(d)  → cursor += d
  now_utc()   → base_utc + cursor
  now_instant() → base_instant + cursor
```
A one-line harness helper advances the cursor **and** `tokio::time::advance(d)` in lockstep, so W/M/T move together from a single call. This removes Fork A's only real cost (coordinating two test clocks).

## 5. Compound → seam dependency map

Each compound is deferred from Phase-9 findings; the catalogue bug it hunts is named, the seam sites it needs are listed, harness-shape options are framed (Joe-lock in design per §6 Q4).

- **C1 — F-10 unknown-signer arriving during F-1b drop.** Sites 3 (F-10 window), 4 (mark_lost), 5 (ACTIVE). Bug hunted: M3 (HeldPending survives identity arrival but drain doesn't fire) × M6 (Phase-5 spawn leak) compound — F-1a re-streams the join while HeldPending still holds the prior version → duplicate-ingest hazard. Harness: in-process NodeRuntime vs two real Nodes.
- **C4 — Phase-5 reconnect scheduler under churn (drop/recover ×5 in 10 min).** Sites 1 (ladder now), 2 (tick cadence), 4/5 (lost/active). Bug hunted: M6 spawn-per-peer-per-tick leak + `peer_records`/`relationships` consistency + ladder reset on ACTIVE + cursor invariants (§3.6). Hardest without the seam (15/30/60/120-min ladder = 5 h real-clock).
- **C6 — F-10 identity-arrival hook under parallel arrivals (two identities).** Site 3. Bug hunted: M9 (HeldPending double-drain on parallel identity arrivals). Race-window-sensitive; distinct from Phase-9 C10 (single-identity-multiple-replicate).
- **C8 — bidirectional simultaneous push.** Sites 5 (ACTIVE), 6 (handshake timeout). Bug hunted: M8 (bidirectional simultaneous push deadlocks F-2a session). `try_send` non-blocking makes the bug improbable but not impossible — deserves a real test, not a confidence assertion (D-065).

## 6. Open questions for the design phase (Joe-locks)

- **Q1 — fork.** **LOCKED A** (this session). Recorded, no re-litigation.
- **Q2 — seam fence.** Confirm scope = the six §3.2 sites only; the ~90 stamping reads (§3.4) stay out. *(Lean: confirm.)*
- **Q3 — `Clock` trait mechanics.** `trait Clock` object (`Arc<dyn Clock>`) vs generic param; sync API (both reads are sync, non-blocking). *(Lean: `Arc<dyn Clock>`, sync — matches the existing `Arc`-threaded scheduler params and avoids genericising `scheduler_tick`.)* Crate home: `xgen-common` (per stub + cross-crate use by core + node).
- **Q4 — per-compound harness shape.** For each of C1/C4/C6/C8: in-process `NodeRuntime` vs two real Nodes; observability surface; honesty assertions (Phase-9 survey precedent). To lock per-compound in design.
- **Q5 — new-compound survey.** Phase-9 added C9/C10 from trace; the clock audit may add C11+. §7.

## 7. New-compound survey

No new compound is asserted at audit time (would be invented, not grounded — D-065). One candidate to *survey* in design, not pre-judge: the T-domain tick loop spawns a **detached `tokio::spawn` per due peer per tick** (`reconnect.rs:106`+ Lock B4); C4 already hunts the leak, but a *W-domain ladder collision* — two peers due in the same tick advancing the shared `attempt_cursor` mutex under the seam's controlled clock — may expose an ordering/aliasing case worth a dedicated probe (candidate **C11**, survey-only). Decide in design.

## 8. State & next-active

- Suite **1193/0/2** (audit-only, no code, not re-run).
- No DECISIONS change (M8.6 sequencing + seam shape are arc-local, D-069; promotion-watch the `Clock` trait if it later recurs as a cross-arc pattern).
- **D-074 companion (same commit as this audit):** JOURNAL J-288 (M8.6 Phase-0 OPENED) + CLAUDE.md PLAY flip (next-active → M8.6 design) + ROADMAP version bump.
- **Next-active:** design phase — lock Q2/Q3/Q4 (Q1 locked), author the `Clock` trait + MockClock + threading design, then runbook → Clair. Clair stands down until the M8.6 runbook exists.

---

*End of M8.6 Phase-0 audit. Status: ACTIVE. The seam is surgical (six sites, two leaf layers already parameterised); Fork A + the single-cursor MockClock are Joe-locked; C1/C4/C6/C8 harness shapes are the design phase's job.*  
