# RUNBOOK — M-ADMISSION-SOUNDNESS Leg 1: the four wire situations
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-29  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS LEG IS

🔒 **LOCKED IN PLACE BY JOE, 2026-08-27 (v1.0 PENDING → v1.1 ACTIVE).** No change to the body at lock. It rides a commit with real content rather than taking one of its own.

The first of five legs of `M-ADMISSION-SOUNDNESS — driving the ruled admission mechanism as one lived story`, filed at J-780 against `ed8f789`.

🛑 **THIS LEG DOES NOT TEST. IT OBSERVES.** It builds one new box-gated binary, `xgen-mptest/tests/mp_admission_soundness.rs`, that drives **four real situations** end to end over real binaries and a real wire, and **prints what happened** in a form a person can read.

🔑 **THE DISTINCTION IS THE WHOLE MILESTONE, AND IT IS EASY TO LOSE WHILE WRITING RUST.** ***A test asserts what we already decided. A simulation shows us what we never asked.*** Every instinct in a `tests/` directory pulls toward `assert!`. **Resist it.** Assert only where a clause is unambiguous and its violation would mean the rig is broken; everywhere else, **print and move on**.

📌 **`D-155` in reverse.** The other legs put questions to Joe in the vocabulary of meaning. This one produces the raw material those questions will be built from — so the transcript's labels are written for a person, not for a compiler.

---

## §1 — 🔒 WHAT IS ALREADY RULED (do not re-open)

| | |
|---|---|
| **Instrument** | 🔒 option **(a)** — a driver in `xgen-mptest`. Not a PowerShell rig. |
| **Q-1** | 🔒 **B** — simulate the reachable situations; the unreachable ones are `M-ADMISSION-SURFACE`'s, filed and unscheduled. |
| **Scope of this leg** | **S-1, S-2, S-7, S-8.** S-3/S-4 are **Leg 2** (they need a live rig and Joe's eyes). S-5/S-6/S-10 are **not drivable at all** — no emitter exists. |
| **Gating** | `#[ignore = "heavy: …; box-gated RUN"]`, like all 32 sibling mp files. |

---

## §2 — GROUNDING. **OPEN EACH SITE BEFORE YOU USE IT (`D-153`).**

Measured 2026-08-27 at `ed8f789` (= `origin/main` by `git ls-remote`, tree clean).

| # | site | what is there |
|---|---|---|
| **A** | `xgen-mptest/tests/mp_g4_rejoin_e2e.rs:201-221` | `rig()` — spawns node + alice + bob, connects both `--aicontrol` pipes, returns the URL. **The pattern to follow; do not invent a second one.** |
| **B** | same file, `:138-152` | `call()` returns the `Reply` whatever it is; `ok()` asserts success. 🔑 **`call` is this leg's default and `ok` is for rig setup only** — a refusal is the OBSERVATION here, not a failure. |
| **C** | same file, `:160-171` | `is_present()` drives the `members` verb, which re-derives through the node's own `derive_resolved` ⇒ it answers *did this SURVIVE RESOLUTION*, not *did the node say Accepted*. **The right oracle.** |
| **D** | `xgen-mptest/tests/c1_lifecycle.rs:36`, `mp_r2_adversarial.rs:49`, `mp_r2_churn.rs:48`, `mp_r2_restart.rs` | `AicontrolClient::connect(&node.aicontrol_pipe, …)` — **the node's own admin pipe, already driven by four sibling tests.** |
| **E** | `xgen-node/src/aicontrol.rs:174` | `"space force-eject" \| "space unban" => Some("event_id")` — **both are real node verbs.** |
| **F** | `xgen-client/src/app.rs:1008-1027` | `InviteArgs`: `role: String`, `valid_for_days: Option<u32>`, `note: Option<String>`. |
| **G** | `xgen-client/src/ops.rs:1182-1188` | `valid_until = Utc::now() + Duration::days(validity_days)`, default 14. 🔑 **NO LOWER BOUND** ⇒ `0` stamps an already-spent invite. `3045` is the **ceiling** and is not in play. |
| **H** | `xgen-core/src/space/state.rs:1273-1276` | `let (role, invited_by) = match self.pending_invites.remove(joiner) { Some(pi) => (pi.role, pi.invited_by), None => (Role::Member, None) };` — **the demotion, and the whole of S-2.** |
| **I** | `xgen-node/src/fanout.rs:824` | the single `1011` refusal constant, shared by every refusal case. |

---

## §3 — THE FOUR SITUATIONS

📌 **One `#[tokio::test]` per situation, one port per situation** (`mp_g4_rejoin_e2e` holds **8590** and **8591** — take **8592-8595**, and say so in a comment so the next file does not collide). Each prints a header line, then the observations, then a footer, all `eprintln!` and all labelled with the situation id.

### `S-1` — she leaves in the morning and comes back in the evening

alice creates a Space and a room, invites bob as **member**, bob joins, sends one message, leaves, rejoins.

⚠️ **ANNOTATION AT THE SITE (`D-131`, J-781, 2026-08-29): THIS BLOCK ASSUMED `rooms` WOULD BE READABLE. IT IS NOT — BEFORE OR AFTER LEAVING.** The reply is not an empty list; it is `no known Space with ID …`. **Not a rejoin effect: `ops::join` never writes client state** — the five `write_client_state` sites in `ops.rs` are `register` · `create_space` · `create_room` · `create_dm_space` · `leave`, and **`join` is not among them** ⇒ ***a member's own client records his departure and never his arrival.*** Clair printed it verbatim rather than dropping it (`D-2`, reported not absorbed). 🛑 **It reaches the GUI** (`desktop.rs:629 get_spaces` reads that same state) **and reshapes Leg 2's S-4 completely.** 🔒 **Leg 2 §7.1 forbids fixing it — the fix would destroy the observation.** *Original text follows.*

**Observe and print:** the rejoin reply verbatim · `is_present` after · bob's `rooms` before leaving and after rejoining · bob's `history` for the Space after rejoining. 📌 **Assert only** that the rejoin was accepted and that he is present — those are `D-154`①'s presence half and V-9a already proves them; everything else is a reading.

### `S-2` — 🔑 the silent demotion, and this is the one to get right

alice creates the Space, invites bob **as `admin`**, bob joins, **and this is confirmed before he leaves** — print his role from `members`. bob leaves. **No new invite is issued.** bob rejoins on his own anchor.

**Observe and print:** bob's role in `members` **before leaving** and **after rejoining** · the rejoin reply verbatim · **anything at all in the reply that mentions a role.**

🛑 **THE POINT OF THIS SITUATION IS THE SILENCE.** `state.rs:1275` says his role comes back `Role::Member`. **What this leg must establish is whether ANYTHING tells him.** ⇒ ***if the transcript shows a rejoin reply identical to S-1's, that identity IS the finding*** — record it as an observation, in plain words, and route nothing.

⚠️ **Do NOT assert the demotion.** If it turns out he keeps `admin`, that is a far bigger finding than a red test, and an `assert_eq!(role, "member")` would report it as a *failure of this leg* rather than as *a contradiction of `D-154`①*. **Print both roles and let the reader judge.**

### `S-7` — node-eject, then a rejoin attempt, then un-ban

alice creates, invites bob, bob joins. Connect a **third** `AicontrolClient` to **`node.aicontrol_pipe`** (site D) and drive **`space force-eject`** naming bob. bob attempts a rejoin. Then drive **`space unban`** and bob attempts again.

**Observe and print:** the eject reply and its `event_id` · both rejoin replies verbatim, **with their codes** · `is_present` at each stage · whether the second attempt succeeds, and if so **what role bob holds afterward** (it feeds S-2's reading).

📌 **`D-154`⑥'s known cost is the thing to look at: retention makes the ejection a durable federated record while `node_eject` is itself reversible.** Print, do not judge.

### `S-8` — a stranger and a returning member with a spent invite

Three approaches to the **same** Space, each printed side by side:
1. **carol**, never invited, never a member, attempts a join.
2. **bob**, invited with **`--valid-for-days 0`** (site G), who then attempts a join after a short deliberate sleep. ⚠️ **`valid_until == now` is a boundary — the sleep is what makes the observation unambiguous, and it must be in the code with a comment saying why.**
3. **bob** with a normal 14-day invite, as the control that the rig works at all.

**Observe and print:** all three replies **verbatim, adjacent, under one header**, so the identical text is visible as text rather than described.

⚠️ **ANNOTATION AT THE SITE (`D-131`, J-781, 2026-08-29): THE `1011` PREMISE BELOW DOES NOT HOLD ON THIS PATH, AND S-8 COULD NEVER HAVE OBSERVED IT.** **`1011` appeared in NONE of the three replies.** It is `collect_invite_bootstrap`'s refusal (`fanout.rs:824`), and **`ops::join` swallows it** — the `_ =>` arm at `ops.rs:1729` — then falls through to `rejoin_anchor_or_root` and submits the join anyway. ⇒ **`1011` is not reachable by a user attempting to join at all**; a joining user receives the join gate's verdict (`3044` / `4000` / ACCEPTED). 🔑 **J-779's ruling STANDS and is unaffected** — it governs the bootstrap door — **but the reasoning Chat gave for it was about what a returning member hears, and a returning member never hears it.** **Chat's defect (`D-1`), reported not absorbed.** *Original text follows.*

🔑 **You ruled `1011` on the oracle argument at J-779 and this is Joe LOOKING at it.** 🛑 **Assert nothing about the strings being equal.** ***If they are identical, the transcript shows it. If they differ, that is a finding and an assertion would have hidden it as a red test.***

---

## §4 — HOW THE TRANSCRIPT MUST READ

🛑 **A wall of JSON is not an observation.** Each situation prints:

```
── S-2 · an admin leaves and comes back ────────────────────────────
   setup      alice created <space>, invited bob as admin
   before     bob role = admin, present = true, rooms = [general]
   action     bob leave
   action     bob join (no new invite; anchored on his own events)
   reply      <verbatim Reply debug>
   after      bob role = ???, present = ???, rooms = ???
   READING    <one plain-English line: what a person in bob's seat would know>
── end S-2 ─────────────────────────────────────────────────────────
```

📌 **The `READING` line is the deliverable.** It is one sentence, written for a person, saying what bob would and would not know. **If it cannot be written honestly from what was printed, the situation did not observe enough — that is itself a finding for the hand-back, not something to paper over with a guess.**

---

## §5 — VERIFICATION

🔒 **Chat re-drives every number independently on a FORCED REBUILD (Rule 5), `Compiling xgen-mptest` present before any result is read.**

| id | what it proves | how |
|---|---|---|
| **V-1** | The suite compiles and is fully box-gated | `cargo test --workspace` ⇒ **+1 SUITE, +4 ignored, +0 passed.** 🛑 **A moved PASSING count means a soundness leg has become a test leg — a FINDING, not a success (§6.5).** |
| **V-2** | The four situations actually run | `cargo test -p xgen-mptest --test mp_admission_soundness -- --ignored --test-threads=1` ⇒ **4 passed**, and the full transcript captured to a file. |
| **V-3** | The transcript is readable | Every situation has a header, a footer, and a **non-empty `READING` line**. Read the captured output; do not infer it from the source. |
| **V-4** | No port collision | Grep the whole `tests/` directory for each port used. `8590`/`8591` are `mp_g4_rejoin_e2e`'s. |
| **V-5** | Nothing outside `xgen-mptest/tests/` changed | `git --no-pager diff --stat` shows **one new file and nothing else**. 🛑 If a helper had to move into `xgen-mptest/src/`, that is a deviation to REPORT, not to absorb. |

🔒 **Carried by scope, stated not measured** (zero `.ts`, `.svelte`, `ui/**`): vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15**. 🛑 **Catalogue UNMEASURED.** Baseline: cargo **1667 / 0 / 64 × 57 SUITES** at `ed8f789`.

---

## §6 — WHAT THIS LEG MUST NOT DO

1. 🛑 **It must not change protocol, node or client behaviour.** Not one line outside `xgen-mptest/tests/`.
2. 🛑 **It must not assert a refusal's text, a role, or an equality between two replies.** Those are the observations. **An assertion turns a finding into a red test and hides it.**
3. 🛑 **It must not drive S-3, S-4, S-5, S-6 or S-10.** The first two are Leg 2; the last three have **no emitter** and are `M-ADMISSION-SURFACE`'s.
4. 🛑 **It must not build a kick or an admission setter**, nor construct those events directly to work around their absence. ⚠️ ***Constructing the event a fixture-style is exactly what hid this gap for the whole of `M-SPACE-ADMISSION`.***
5. 🛑 **It must not move the passing count.** Every scenario `#[ignore]`.
6. 🛑 **It must not tear the rig down early if that costs an observation**, and must not leave stray processes if it does not.
7. 🛑 **It must not quietly narrow a situation to the part that was easy.** A situation that could not be driven as written is a **deviation reported**, not a situation redefined.

---

## §7 — DoD

- [x] `xgen-mptest/tests/mp_admission_soundness.rs` exists; four `#[tokio::test] #[ignore]` scenarios, ports `8592-8595` with a collision comment — **530 lines**
- [x] `rig()` follows `mp_g4_rejoin_e2e`'s pattern; `call()` used for every observed step, `ok()` only for setup
- [x] **S-2 prints bob's role BEFORE and AFTER, and asserts neither** — `admin` → `member`, role-words in the reply `[]`
- [x] **S-7 drives `space force-eject` and `space unban` over the NODE aicontrol pipe**, both rejoin replies printed with codes
- [x] **S-8 prints all three replies verbatim and adjacent**, with the deliberate sleep and its comment
- [x] Every situation has a header, a footer, and a non-empty **`READING`** line — all four **rewritten after the run** (`D-3`)
- [x] `V-1` — `+1 SUITE, +4 ignored, +0 passed`; **1667 / 0 / 68 × 58 SUITES**, re-driven by Chat on a forced rebuild
- [x] `V-2` — **4 passed, 0 failed** under `--ignored`, transcript captured
- [x] `V-4`, `V-5` — **0 port collisions**; `git diff --stat` empty, one new file and nothing else
- [x] Deviations **reported, not absorbed** (Rule 6) — **three, §10**

📌 **"Commit pushed" is deliberately not a DoD item** — `Status: COMPLETED` in this header is the shipped signal.

---

## §8 — 🔓 OPEN DECISIONS INSIDE THIS RUNBOOK

**None.** Everything this leg needs was ruled at J-780 or measured at Leg 0. 📌 Stated explicitly so a reader can tell the difference between *nothing was open* and *nobody looked*.

---

## §9 — ✅ WHAT WAS OBSERVED (J-781)

🔑 **THE LOGIC IS SOUND AND THE MECHANISM IS INVERTED ON A REAL WIRE.**

| situation | what happened |
|---|---|
| **S-1** | bob came back with the same role and the same reach and **still reads the room's history**. ⚠️ But his own client holds **no record of the Space at all** — see the §3 annotation. |
| **S-2** | **`admin` → `member`**, and `role`/`admin`/`member`/`demot`/`privile` appear **nowhere** in the rejoin reply. The reply is **shaped identically to S-1's**. ***The silence is a measurement, not a claim.*** |
| **S-7** | The ban door speaks plainly, naming him and the Space. **Un-banning alone does not put him back** — he stays absent until he asks again, and returns a plain member, with nothing said about the ejection that stays in the record. |
| **S-8** | **(1) carol, never invited → ACCEPTED.** **(2) bob, invite lapsed → REFUSED `3044`, quoting his exact missed deadline.** **(3) bob, live invite → ACCEPTED.** 🔑 ***The invite is not what opens the door; it is the only thing that can be CHECKED, and therefore the only thing that can FAIL.*** |

🔓 **THREE FINDINGS ROUTED TO JOE AT LEG 3, NOT GATED ON LEG 2:** the open door · the spent invite · the silent demotion.

---

## §10 — ⚠️ DEVIATIONS (Rule 6) — **THREE, NONE ABSORBED. TWO ARE CHAT'S.**

| id | seat | what |
|---|---|---|
| **`D-1`** | **Chat** | §3 S-8's **`1011` premise does not hold on this path**. Annotated at its site. Built as written; reported as observing something *adjacent* to what §3 anticipated, **not quietly redefined**. |
| **`D-2`** | **Chat** | §3 S-1 assumed `rooms` would be readable. **`ops::join` never writes client state.** Annotated at its site. Printed verbatim rather than dropped. |
| **`D-3`** | **Clair**, caught by herself | 🔒 **A `READING` LINE WRITTEN BEFORE THE RUN IS A PREDICTION WEARING AN OBSERVATION'S CLOTHES.** All four were drafted before the run; **S-1's was wrong.** Corrected from observed output and **re-run to confirm**; every recorded number is from that second, final run. 📌 **The only defect in this arc so far from the implementing seat rather than the specifying one**, and now a standing constraint in Leg 2's §0. |

📌 `D-5` from G-5 still stands and is not this leg's: **`cargo fmt` is unclean on baseline across 209 files and is not a gate.**
