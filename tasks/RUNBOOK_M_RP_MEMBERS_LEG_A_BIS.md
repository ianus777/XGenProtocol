# M-RP-MEMBERS Leg A-bis — bound the fill, and give the roster a caller
> **Status**: ACTIVE  
> Version: 1.0  
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

### Step 2 — the timeouts (per §2's lock)

Per-step, **never an overall cap** — an overall cap aborts legitimate work on a large Space (Phase-0 §4c-i). Bound: the **connect**, and **each** `identity.get` `recv()`. Default 5 s each, config-tunable. ⇒ worst case `5 + 5 + 5N`, **always terminating**.

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
- [ ] Roster returned live on 9222: `fill_space_records` → a `MembersResult` whose members match the seeded Space
- [ ] Re-entrancy still green: two consecutive calls, second `touched > 0, fetched 0`, no connection error

*(Per the standing rule, "commit pushed" is deliberately NOT a DoD item.)*

---

## §5 — ⚠️ RE-OPEN-ON-BUILD CLAUSE (mandatory, D-122)

If anything here contradicts the Phase-0 or the code — a private function that will not extract cleanly, a missing derive, a timeout that cannot sit where §2 puts it — **stop and report; do not absorb**.

📌 **This whole leg exists because the last runbook's grounding missed two things.** Expect the clause to fire on **Step 1's extraction** (the re-entrancy wrapper is easy to get subtly wrong) and on **Step 3's `Serialize`**.

---

## §6 — Handoff

**Leg B** (store + `members-panel` + the 7th `CLIENT_PLUGINS` row) is written **after this lands**, against the **measured** command signature — not against a predicted one. Writing it now would mean specifying a widget against a verb that does not exist, which is the exact failure this leg was created by.

**Leg B carries four Phase-0 bindings:** self is a **fixture** (always present · first · filter-immune, resolved from `selfState` never the book) · the roster crosses **`Option`-shaped, never a bare array** · any **count derives from the roster, never rendered rows** · **live `membership.*` refresh** (§5), with the cold fill as the start only.
