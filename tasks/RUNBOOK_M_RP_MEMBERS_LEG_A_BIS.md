# M-RP-MEMBERS Leg A-bis — bound the fill, and give the roster a caller
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-25  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Why this leg exists at all

**It was not in the plan.** Leg A shipped two commands and Leg B was to be pure frontend. Grounding Leg B's data path found **two blockers, both Rust, both Chat's misses:**

1. ⚠️ **THE WIDGET HAS NO WAY TO GET THE ROSTER.** Leg A shipped `get_address_book` (the whole book, unscoped) and `fill_space_records` (returns a `FillReport` — **counts, not members**). Phase-0 §4-B locks the panel to *the members of the latched Space*. **There are 18 registered commands and not one returns a roster.** Leg B cannot be written without this.
2. ⚠️ **THE FILL IS UNBOUNDED** (Phase-0 §4c-i). Without a bound, ③ *"I am waiting for the others"* is **not sayable**, and the `FillLock` mutex Chat added in Leg A can **deadlock the feature for the life of the process**.

**Scope:** Rust only. Moves the **cargo** floor. ⚠️ Leg B stays pure frontend and moves **svelte-check** — that split is the whole reason these are separate legs, and **two commits, not one**.

**Out of scope:** any `ui/**` · any store or widget · the `CLIENT_PLUGINS` row · `skin.css` · anything from M13.

---

## §1 — Grounding (measured 2026-07-25 at `79f4175`)

- **`ops::members` is `pub`** (`ops.rs:2661`) and takes `&crate::app::MembersArgs { space: String }`. The desktop shell **can** call it.
- ⚠️ **But `drain_space_events` (`:2522`), `members_projection` (`:2622`), `observed_identities` (`:2700`) and `partition_observed` (`:2736`) are all PRIVATE to `ops`.** The shell cannot compose them; it can only call whole verbs.
- 🔑 **AND THAT IS THE TRAP: `ops::members` DRAINS, AND `ops::fill_from_space` DRAINS.** Calling both = **two full drains of the same Space DAG**, back to back. Phase-0 §4 explicitly congratulated the design on the opposite — *"F1 and F2 READ THE SAME DRAIN"* — so a double drain is **a regression in reasoning even though it would work**, and it is **user-visible**: it roughly doubles the cold-start window, which is exactly the time ③ is on screen.
- **`FillReport`** = `candidates · fetched · not_found · touched`. No member list.
- 🔒 **`fill_from_space` is re-entrant BY DESIGN (J-586, expensive to learn):** the wrapper clears `ctx.session.conn = None` on **every** exit, including `?`-skipped paths. **Any new entry point needs the same discipline** — this is the single most likely thing to get wrong here.
- **Unbounded paths:** `session.rs:138` → `connect_url` → `connect_async(url).await` (no timeout) · `identity_get_on` → `conn.recv().await` (no timeout). **Bounded:** `drain_space_events` via `sync_completion_timeout` (5 s, `[sync] completion_timeout_seconds`).
- **`reanchor_space`** already uses `Duration::from_secs(5)` for `get_dag_tips` — **5 s is this codebase's existing number, in two places.**

---

## §2 — 🔓 OPEN FOR JOE: THE TIMEOUT'S BLAST RADIUS (structural — D-123 rider ②)

The two unbounded waits live in **shared** code: `ensure_connected` (`session.rs`) and `identity_get_on` (`ops.rs`). Bounding them there fixes the defect **for every caller** — the desktop shell, `--batch`, and `--aicontrol` alike.

**That reach is a decision, not an implementation detail, so it is named rather than taken.**

- **T1 — bound them where they live (`ensure_connected`, `identity_get_on`).** ① *User-visible:* an unbounded network wait is a defect for **every** binary, not just this widget; a hung `--batch` today never returns either. ⚠️ It also **changes shipped behaviour on a slow link** — a run that used to succeed after 30 s now fails at 5. ② *Cost:* smallest; two wraps; config-tunable via the `[sync]` precedent.
- **T2 — bound only inside the fill path**, leaving shared code untouched. ① *User-visible:* identical for R7; `--batch` stays unbounded. ② *Cost:* duplicated logic, and the defect stays live everywhere else.

**Chat recommends T1 with configurable values defaulting to 5 s**, precisely because the defect is not R7's. ⚠️ **But the behaviour change on slow links is a real trade and it is Joe's**, not Chat's, to accept.

🔒 **LOCKED — T1 (Joe, 2026-07-25).** ⚠️ **PROVENANCE: DELEGATED** — *"t1 as you recommend"*. Joe adopted Chat's recommendation without walking the options. **Recorded prominently because this one has REACH:** it changes shipped connection behaviour for `xgen-client` **desktop, `--batch` and `--aicontrol` alike**. If a future long-running batch begins failing on a slow link, **this is the line that explains why**, and re-opening it costs a config value, not a redesign.

### ⚠️ §2a — CHAT'S RECOMMENDED VALUES WERE WRONG. MEASURED AFTER THE LOCK, BEFORE THE BUILD.

Chat proposed **5 s** for both bounds, reasoning from `reanchor_space`'s `get_dag_tips` (5 s) and `sync_completion_timeout` (5 s). **Both are precedents for a different operation.** The codebase already has named constants for **these two specific operations**, and they say **10**:

| Operation | Existing constant | Value |
|---|---|---|
| **connect** | `resident.rs:211` — `const CONNECT_TIMEOUT` | **10 s** |
| **one request → one reply over the socket** | `resident.rs:785` — `pub const SEND_ACK_TIMEOUT` | **10 s** |
| a multi-message drain | `sync_completion_timeout` | 5 s, configurable |
| `get_dag_tips` request-reply | `desktop.rs:385` | 5 s |

🔒 **CORRECTED DEFAULTS: connect = 10 s · each `identity.get` recv = 10 s.** For **connect** there is no judgement call — a second connect-timeout constant at a different value would be **two homes for one policy** (D-067). For **recv**, both 5 and 10 have precedent; the closest analogue **by shape** is `SEND_ACK_TIMEOUT` — one request, one reply, same socket — so 10.

🔑 **AND THE TOTAL DOES NOT BLOW UP, BECAUSE THE FETCH LOOP ABORTS ON THE FIRST FAILURE.** `identity_get_on(conn, id).await?` propagates with `?`, so a dead node costs **one** timeout, not N. ⇒ realistic worst case to failure = **connect 10 + drain 5 + first fetch 10 ≈ 25 s**, *not* `10N`. A slow-but-alive node with N = 200 costs N × real latency, not N × the bound. ⚠️ **This is load-bearing for ③ and must not be broken:** if a future change makes the loop *continue* past a failed fetch, the worst case becomes `10N` and *"I am waiting for the others"* can run for half an hour. **If that changes, §4c-i's progress note stops being deferrable.**

📌 **Chat's error, named:** the number was picked from **one** precedent without checking whether the codebase already had a constant for **this** operation. It did, twice. *Same defect class as the rest of this arc — a claim narrower than the thing it described.*

📌 If T2 is chosen, record the shared-code defect as **filed, not fixed**, rather than letting it disappear.

---

## §3 — Steps

### Step 1 — one drain, two outputs

Decompose so the fill and the roster projection **share a single drain**:

1. Extract the existing fill body into a private `fill_from_events(ctx, book, space, &events) -> Result<FillReport>` — everything `fill_from_space_inner` does **after** its `drain_space_events` call, unchanged.
2. `fill_from_space_inner` becomes: drain → `fill_from_events`. **Behaviour identical; its tests must not change.**
3. Add `pub async fn fill_and_members(ctx, book, space) -> Result<(FillReport, MembersResult)>` — drain **once**, then `members_projection(space, &events)?` **and** `fill_from_events(...)`.

🔒 **The re-entrancy wrapper applies to the NEW entry point too.** `fill_and_members` must clear `ctx.session.conn = None` on **every** exit, exactly as `fill_from_space` does. ⚠️ **Do not tidy the existing clears; do not add caller-side connection management.**

📌 `fill_from_space` is **kept** — it has tests and it is the honest verb for "just fill." Two entry points over one body is not drift; two *bodies* would be.

### Step 2 — the timeouts (per §2's lock, values per §2a)

Per-step, **never an overall cap** — an overall cap aborts legitimate work on a large Space (Phase-0 §4c-i). Bound the **connect** (`ensure_connected`) and **each** `identity.get` `recv()` (`identity_get_on`), per T1 — **in the shared code where they live**, so `--batch` and `--aicontrol` are fixed too.

🔒 **Defaults 10 s each (§2a), configurable.** ⚠️ **Reuse `resident::CONNECT_TIMEOUT` for the connect rather than minting a second constant** — a second connect timeout at a different value is two homes for one policy.

### Step 3 — the command

Replace `fill_space_records`'s body to call `fill_and_members` and return **both**. ⚠️ **`MembersResult` needs `Serialize` checked** — `FillReport` did not have it (Leg A trap ②), so **assume nothing**; verify and add if missing, additively.

📌 **Command naming is Joe's** if the verb's meaning has changed enough to want a new name; Chat's default is to keep `fill_space_records` and widen its return, since the call site and timing are unchanged.

### Step 4 — register / verify

No new registration if the name is kept. ⚠️ If renamed, **update `invoke_handler!`** — it fails silently from the webview side.

---

## §4 — Definition of Done

- [ ] `cargo build` clean
- [ ] `cargo test` — floor **1585 / 0 / 62 across 56**; report new totals and **explain every delta**. ⚠️ Detached, poll, sum `test result:` **case-sensitively**; **56 = completeness**. Never run cargo and the dev client together (N-117)
- [ ] **`fill_from_space`'s existing tests pass UNCHANGED** — that is the proof the decomposition is behaviour-preserving, not an assertion that it is
- [ ] Scope-clean: `git show --stat` — **zero `ui/**`**, zero `skin.css`
- [ ] `svelte-check` **not re-measured**, held by scope — say so rather than imply it
- [ ] **ONE DRAIN PROVEN, NOT ASSUMED** — instrument or trace-log the drain and show **one** per `fill_space_records` call, not two
- [ ] **Timeout EXERCISED, not asserted** — point the client at a black-hole endpoint (a listening socket that never replies) and show the command **rejects** at the bound instead of hanging
- [ ] 🔑 **THE LOCK RELEASES AFTER A TIMEOUT** — a timed-out fill followed by a **successful** one, same process. *This is the leg that proves the `FillLock` deadlock is actually closed; without it the mutex is still a live hazard*
- [ ] ⚠️ **THE FETCH LOOP STILL ABORTS ON FIRST FAILURE** (§2a) — assert one timeout, not N. This is what keeps the worst case at ~25 s instead of `10N`, and ③'s honesty depends on it
- [ ] Roster returned live on 9222: `fill_space_records` → a `MembersResult` whose members match the seeded Space
- [ ] Re-entrancy still green: two consecutive calls, second `touched > 0, fetched 0`, no connection error

*(Per the standing rule, "commit pushed" is deliberately NOT a DoD item.)*

---

## §5 — ⚠️ RE-OPEN-ON-BUILD CLAUSE (mandatory, D-122)

If anything here contradicts the Phase-0 or the code — a private function that will not extract cleanly, a missing derive, a timeout that cannot sit where §2 puts it — **stop and report; do not absorb**.

📌 **This whole leg exists because the last runbook's grounding missed two things.** Expect the clause to fire on **Step 1's extraction** (the re-entrancy wrapper is easy to get subtly wrong) and on **Step 3's `Serialize`**.

---

## §6 — Handoff

✅ **CLOSED 2026-07-25 (J-590), commit `2c3144b`.** All DoD items met and **independently re-driven by Chat** — roster live on 9222 (`[FillReport, MembersResult]`, `members:1`, `events_replayed:3`) · warm re-entrancy (`touched:1 / fetched:0`) · timeout exercised (11093 ms, rejected at the bound) · 🔑 **lock released after timeout** (err → ok, same process) · reproduced again under the sanctioned rig.

⚠️ **TWO THINGS THIS RUNBOOK GOT WRONG, RECORDED RATHER THAN QUIETLY FIXED:**

1. ⚠️ **§4's black-hole DoD CANNOT SEE THE DEFECT IT WAS MEANT TO CATCH.** *"A listening socket that never replies"* trips the **connect** bound and passes green. A node that **completes the WS upgrade and then goes dark during auth** still hangs — so this leg's *"the FillLock deadlock is closed"* holds **only for a peer that never upgrades**. ⇒ **`RUNBOOK_M_RP_MEMBERS_LEG_A_TER.md` closes it**, and its DoD demands a harness that upgrades *then* goes silent.
2. ⚠️ **§2a's worst-case arithmetic (≈25 s) omitted auth entirely.** Corrected in A-ter §2b: the fill pays connect + auth **twice**, so the ceiling is `35 + 4A`.

📌 **AND THE FLOOR HELD AT 1585 BECAUSE THIS LEG ADDED NO TESTS** — the unchanged floor proves the decomposition is behaviour-preserving and proves nothing about the new verb or the bounds. Stated, not implied.

**Leg B** (store + `members-panel` + the 7th `CLIENT_PLUGINS` row) follows **after A-ter**, against the now-**measured** signature: a **2-element array**, snake_case fields (`space_id` · `identity_id` · `joined_at` · `invited_by` · `events_replayed`). 🔓 **Whether that tuple becomes a named struct `{ fill, members }` is OPEN and Joe's, and belongs before Leg B's runbook, not after.**

**Leg B carries four Phase-0 bindings:** self is a **fixture** (always present · first · filter-immune, resolved from `selfState` never the book) · the roster crosses **`Option`-shaped, never a bare array** · any **count derives from the roster, never rendered rows** · **live `membership.*` refresh** (§5), with the cold fill as the start only.

⚠️ **AND A FIFTH, LEARNED AT J-590:** the `run-*.ps1` scripts **leak their Vite and their readiness probe cannot tell whose server answered**, so a leaked Vite silently serves a stale bundle. **Leg B is frontend work and would be verified against it, reporting success throughout.** Verify the bundle's provenance before trusting any Leg B measurement.
