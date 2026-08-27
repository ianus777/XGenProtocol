# RUNBOOK — M-SPACE-ADMISSION Leg G-4: the client anchor selection
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-26  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS LEG IS

**`M-SPACE-ADMISSION` Leg G-4 — the client anchor selection.** `ops::join` stops looking for an invite that a rejoiner does not have and instead anchors her `membership.join` on **her own membership events in the batch the node already serves her** (Leg G-3). Refusal or absence falls back to today's behaviour, unchanged.

📌 **Phase-0:** `tasks/M_SPACE_ADMISSION_LEGG_PHASE0.md` v1.5 §2.4 + §5. 🔒 **Anchor commit `2926b84`** — every `file:line` below was measured on it (`D-152` clause 1).

🎯 **THIS IS THE LEG THAT MAKES THE ARC PAY.** G-1 admits her at the gate, G-2 refuses her honestly when she cannot be anchored, G-3 serves her the material — **and she still cannot come back from a fresh install.** After G-4 she can.

🛑 **AND IT IS THE FIRST LEG OF THE ARC WHERE ANYTHING RUNS AGAINST A LIVE NODE, A WIRE AND A SECOND IDENTITY.** `3048` has never been observed on a wire. 🔒 **RULED (Joe, 2026-08-26): the live run RIDES THIS LEG, and its home is `xgen-mptest` — see §5b and §8 `OD-1`.**

🛑 **THIS RUNBOOK IS CLAIR'S. IMPLEMENT FROM THIS VERSION, IN A SESSION OPENED BY HER OWN KICKOFF.** Deviations are **reported, never absorbed** (Rule 6). ⚠️ **Where this runbook and any chat message disagree, THIS DOCUMENT WINS — and the disagreement is itself a finding.**

---

## §1 — 🔑 THE FINDING THAT SHAPES THE LEG: THE PREDICATE THAT DECIDES A CORRECT ANCHOR ALREADY LIVES IN `xgen-core`, AND THE CLIENT ALREADY IMPORTS IT

✅ **MEASURED — `xgen-core/src/resolution/state_key.rs:44`, `state_key_for_event`, re-exported at `resolution/mod.rs:17`.** A space-level `membership.join` by her keys `membership:{space}:{her}`. **The events that can collide with it are exactly the events that produce that same key:**

| type | key field read | site |
|---|---|---|
| `MembershipJoin` / `MembershipLeave` | `sender`, **scope-aware** (`room_id` empty ⇒ room-agnostic) | `state_key.rs:53-57` |
| `MembershipKick` | `content["target_identity"]`, **scope-aware** | `state_key.rs:65-68` |
| `MembershipInvite` / `Ban` / `NodeEject` / `NodeUnban` | `content["target_identity"]`, **always space-level** | `state_key.rs:77-86` |

✅ **`xgen-client` ALREADY DEPENDS ON IT AND ALREADY CALLS IT** — `xgen-client/src/ai_service.rs:50` imports `resolution::{derive::conflicts_in_log, derive_resolved, state_key_for_event}` and uses it at `:547`.

🔑 **⇒ THE CLIENT MUST NOT WRITE ITS OWN *does this event name me?* PREDICATE, AND THIS LEG DOES NOT.** G-3's `bootstrap_event_names_requester` (`xgen-node/src/fanout.rs:750`) is a **disclosure** test on the node's seat; what the client needs is a **collision** test, and `state_key_for_event` is that test — **the same function `conflicts_in_log` (`xgen-core/src/resolution/derive.rs:261`) uses to decide `3048`.**

🔒 ***The client selects its anchor with the very function the node will judge it by. One fact, one place (`D-067`) — by construction, not by mirroring.***

🛑 **AND THE ALTERNATIVE IS A REAL DEFECT, NOT A STYLE PREFERENCE.** A hand-written *names me* union on the client would have to re-derive the scope rules; a room-level kick of her does **not** collide with a space-level rejoin (different key), and a client that anchored on one would build a longer chain that still leaves the space-level pair concurrent — ***a plausible, non-empty, wrong anchor, and every leave-based test would pass.*** `N-197`'s shape, one crate over.

---

## §2 — GROUNDING. **OPEN EACH SITE BEFORE YOU EDIT (`D-153`).**

| # | fact | site (`2926b84`) |
|---|---|---|
| **G-a** | `get_invite_bootstrap` scans the drain for `MembershipInvite` whose `content["target_identity"]` is the requester and returns `Ok(Option<String>)`; `Error` ⇒ `Ok(None)` | `xgen-client/src/batch.rs:262-309`, match arms `:283-296` |
| **G-b** | The drain already has the full `Event` in hand — **`prev_events` is present and currently discarded** on this path; `get_dag_tips` reads exactly that field at `:216` | `batch.rs:280-290` vs `:210-221` |
| **G-c** | `FrontierEvent {event_id, prev_events, event_type}` and `compute_frontier` / `cooperative_frontier` are **private to `batch.rs`** and unit-testable without a `Connection` (the MP-F14 spine) | `batch.rs:85-89`, `:96-99`, `:123` |
| **G-d** | `compute_frontier` sorts **lexicographically** (`D-076`) then truncates to `MAX_PREV_EVENTS` | `batch.rs:96-101` |
| **G-e** | `MAX_PREV_EVENTS = 10`; the node's step-10 gate refuses more | `xgen-core/src/dag/graph.rs:26`, `:111` |
| **G-f** | `ops::join` takes `Ok(Some(invite_id)) => vec![invite_id]`, everything else ⇒ `get_dag_tips`, then `rejoin_anchor_or_root` | `xgen-client/src/ops.rs:1674-1705` |
| **G-g** | `rejoin_anchor_or_root` reads `state.last_local_events` — **absent on a fresh install ⇒ the create root** | `ops.rs:142-147` |
| **G-h** | The node serves a route-2 requester the creates **plus only the membership events naming her**, topo-sorted | `xgen-node/src/fanout.rs:900-920` |
| **G-i** | `3048` fires iff `conflicts_in_log(join, log)` — same key, neither event a transitive ancestor of the other | `xgen-core/src/node/runtime.rs:1765-1798`; `derive.rs:260-289` |
| **G-j** | 🛑 **The served batch is a DISCOVERY payload, not an authoritative DAG** — its events' `prev_events` mostly point at events NOT in the batch | `fanout.rs:611-614` |

🔑 **`G-j` IS THE ONE THAT DECIDES THE SHAPE.** Her `membership.leave` was anchored on the DAG tips of the moment — messages, other people's events — **none of which are in her batch.** ⇒ **an intra-batch ancestry walk sees almost no edges, and a *pick the topo-last one* rule would silently drop a genuinely concurrent sibling.** The rule in §3 is therefore written to be correct **without** assuming the batch is ancestry-complete.

---

## §3 — 🔒 THE SELECTION RULE

**Given the drained batch and the join the client is about to build:**

1. **Build the prospective key.** `state_key_for_event` on the join as it will be signed — sender = this identity, `space_id` = `args.space`, `room_id` = `args.room` (empty for a Space join). `None` ⇒ select nothing.
2. **Keep every served event whose `state_key_for_event` equals that key.** Nothing else. No type list, no `sender`/`target` union, no `room_id` test — the key already encodes all three.
3. **Drop any kept event that another KEPT event references in `prev_events`.** (Same subtraction as `compute_frontier`; the reference set is built **only from the kept subset**, never from the whole batch.)
4. **Order and cap.** Keep **batch order** — the node emits `topological_sort_events` — and if more than `MAX_PREV_EVENTS` survive, keep the **LAST 10 in batch order**, then sort the result lexicographically for `D-076` wire-order determinism.
5. **Empty ⇒ select nothing** and fall through to today's path.

🛑 **WHY STEP 4 IS NOT `compute_frontier`, AND WHY THAT IS DELIBERATE.** `compute_frontier` truncates **after a lexicographic sort** (`G-d`) — on an over-wide set that keeps an arbitrary ten, and a dropped sibling is a `3048` the user cannot act on. Batch order is topological, so the **last** ten are the ones most likely to descend from the rest. ⚠️ **`compute_frontier` and `cooperative_frontier` MUST NOT BE MODIFIED** — they serve `get_dag_tips` and changing them would alter every cooperative send. **G-4 gets its own function.**

⚠️ **THE RESIDUE, NAMED AND NOT CLOSED.** More than ten mutually-unordered events on her key — roughly five-plus leave/rejoin cycles, or a long ban/unban history — can still truncate to an anchor that leaves a sibling concurrent, and she meets `3048`. **The refusal is honest and re-submittable; the leg does not close this.** ✅ It is pinned by `V-7` so it is a tested boundary rather than an unexamined one.

🔒 **PRECEDENCE IS UNCHANGED AND THAT IS LOAD-BEARING.** An invite naming her still wins. ✅ **An invitee's behaviour must be byte-identical** — `V-1` asserts it. A dual-entitled requester (departed **and** re-invited with a live invite) takes the invite: G-3 already ruled her narrowed set keeps the invite naming her, so INV-D2/D3 still works.

---

## §4 — THE EDIT

### G4-1 — `xgen-client/src/batch.rs` — the drain keeps what it already receives

- Add a private `fn select_rejoin_anchor(events: &[FrontierEvent], key: &StateKey) -> Vec<String>` implementing §3 steps 2–5. 🔑 **`FrontierEvent` must carry the whole `Event`'s key material** — extend it, or add a sibling projection struct; **either is acceptable, state which and why in the hand-back.** It must be callable **without a `Connection`**, exactly as `cooperative_frontier` is, so `V-5`/`V-6`/`V-7` are unit tests.
- Widen `get_invite_bootstrap`'s return from `Result<Option<String>>` to **`Result<Vec<String>>`**: invite found ⇒ `vec![invite_id]`; else ⇒ `select_rejoin_anchor(...)`; else ⇒ `vec![]`. Accumulate the projection **in the same drain** — 🛑 **no second request, no second round trip.**
- The `Error` / `Goodbye` / `Closed` / `Err` arms return what has been accumulated so far, mirroring today's arms exactly. A `1011` refusal still yields **empty** ⇒ fall back.
- 🛑 **The function NAME stays `get_invite_bootstrap`.** It is client-internal, and a rename inside a behaviour leg makes the diff argue two cases at once (§4's own ruling shape in the Phase-0). **Its doc comment is rewritten to say what it now returns and why** — the same repair §4 prescribed for the wire verb: *a restatement of meaning, not a rename.*

### G4-2 — `xgen-client/src/ops.rs` — `ops::join` uses it

- `match ... { Ok(ids) if !ids.is_empty() => ids, _ => { …unchanged fallback… } }`. 🔒 **`get_dag_tips` and `rejoin_anchor_or_root` are UNTOUCHED** — they remain the fallback for an old node, a `1011`, an empty selection and a first join.
- Update the block comment at `:1667-1673` to name **both** anchor sources.
- ⚠️ **`rejoin_anchor_or_root`'s doc (`:136-141`) becomes narrower than the truth** the moment G4-1 lands — it says a starved rejoiner lands there, and after this leg she usually does not. **Annotate at the site (`D-131`), do not delete.**

### G4-3 — `xgen-node/src/fanout.rs` — the `NAMED, NOT FIXED` paragraph (rider filed at J-777)

At `:854-863`. **Add:** 🔒 **Joe ruled the sketch correct (2026-08-26)** — a former member holding an **expired** invite is refused and must obtain a fresh one; and the consistency reason: **the `3044` gate at `runtime.rs:1806` is not conditioned on the rejoin flag**, so the `OR` would have opened a door onto a locked gate. 🔒 **Clair's condition holds: the losing arm's reasoning STAYS, rewritten never removed** — the next reader hits the same fork and this is the only place it is visible.

### G4-4 — `xgen-core/src/node/runtime.rs` — the G-2 comment that is narrower than the thing it describes

At `:1707-1709`: *"The `3044` expiry check below lives inside the pending-invite branch and never sees a rejoiner."* 🛑 **TRUE for a rejoiner with no invite, FALSE for one holding an expired one** — she reaches `:1804` and is refused `3044`. **Correct it in place, naming both cases.** ⚠️ **Comment only. The gate's behaviour is Joe-ruled and does not move.**

---

## §5 — VERIFICATION

🛑 **`N-207` FIRST, AT THE FRONT OF EVERY RUN, NOT HALFWAY THROUGH IT.** Stamp the edited `.rs` mtimes forward and **require `Compiling xgen-client` in the log BEFORE reading any number.** A right-looking floor over the other seat's binary invites no second look at all.
🛑 **`N-206`: no inner `&`; detached + sentinel; `--no-fail-fast`; sum `^test result:` CASE-SENSITIVELY.** The notification's exit code is the launcher's.

| # | check |
|---|---|
| **V-1** | 🔒 **AN INVITEE IS BYTE-IDENTICAL.** A pending invitee with a live invite still anchors on `vec![invite_id]`, and the served payload is unchanged. **Both pre-existing `collect_invite_bootstrap` tests green and byte-untouched.** |
| **V-2** | A departed member with **no** invite and **no** local state selects her own last space-level membership event(s) and the node accepts the join. |
| **V-3** | A **kicked** member selects the `membership.kick` naming her (`D-154`②) and is admitted. |
| **V-4** | A **banned** member is refused `1011` at the door ⇒ empty selection ⇒ today's fallback. |
| **V-5** | Unit: a **room-level** kick of her is **NOT** selected for a Space rejoin (different key), and **IS** selected for a rejoin of that room. |
| **V-6** | Unit: a `kick` **she issued** against a third party is **NOT** selected (its key names the third party) — the client-side twin of `N-209`. |
| **V-7** | Unit: >10 surviving candidates ⇒ exactly 10 returned, and they are the **last ten in batch order** (§3 step 4), lexicographically sorted. |
| **V-8** | 🔒 **NEGATIVE CONTROL, RUN SEPARATELY.** Revert **only** step 3 (the reference subtraction) and, separately, **only** step 4's ordering. **Two reverts, two DIFFERENT red sets.** 🔑 ***One revert proving two behaviours is a coincidence, not a control.*** |
| **V-9** | 🎯 **THE LIVE LEG. ⚠️ SUPERSEDED BY §5b AT v1.1 (`D-131`) — the v1.0 text is kept below because it is what the leg was scheduled on, and it was under-specified in a way worth seeing.** ~~*Two identities, a real node process, a real wire: alice creates an invite-only Space, invites bob, bob joins, bob leaves, bob's client state is deleted, bob rejoins. Assert the anchor on the wire, `Accepted`, and `bob.is_member()`. Then, with G4-1 reverted, assert `3048` on the wire.*~~ 🛑 **It described SYSTEM-GATE work inside a table of implementation checks, and named no home** — see §5b for `V-9a` / `V-9b`. |
| **V-10** | `cargo` floor moves from **1654 / 0 / 62 × 56 SUITES**; delta measured three ways (`--skip`, libtest's own `filtered out`, and an independent re-drive). Carried by scope: vitest **172 / 172 × 9 FILES**, svelte-check **0 / 34 / 15**. **Catalogue UNMEASURED — write no number.** 🛑 **AND THE FLOOR DELTA COMES FROM `V-1`…`V-8` ONLY. §5b's scenarios are `#[ignore]` by design and DO NOT MOVE IT** — say so explicitly in the hand-back, or an unmoved live-leg contribution reads as *no Rust landed*, which is the 6.1i/6.1j signal INVERTED and would be believed. |

---

## §5b — 🎯 THE SYSTEM GATE. **RULED (Joe, 2026-08-26) — IT RIDES THIS LEG.**

🛑 **§5 IS AN IMPLEMENTATION GATE; THIS IS A SYSTEM GATE, AND CONFLATING THEM IS WHY v1.0's `V-9` WAS UNDER-SPECIFIED.** §5 asks *does the function return what the runbook says*. **§5b asks whether the composition does** — serialisation, transport routing, the client's own data directory, the node's store, and the order things actually happen in. ***No test in §5 can fail the way §5b can fail, and that is the whole reason it exists.***

### The home, measured

✅ **`xgen-mptest`.** Its own header states the purpose: the orchestrator **spawns the real built binaries as separate OS processes**, drives each actor through its `--aicontrol` JSONL pipe, observes through `.events` / `state`, and **does not link the binaries** — *the in-process → real-binary crossing the existing harness cannot do.*

✅ **`join` is already drivable there** — `ClientCommand::Join` is wired into the aicontrol dispatch at `xgen-client/src/aicontrol.rs:488`, **above** the not-exposed block at `:541`. ✅ **The closest template is `xgen-mptest/tests/m12_3_federation_fetch_e2e.rs`** — alice `create-space` (`:191`) → `invite` (`:193`) → bob `join` (`:206`), all over real binaries. ✅ **Restart / same-data-dir shape:** `xgen-mptest/tests/mp_r2_restart.rs`, `ManagedProcess` + `instance_label`.

🔒 **New file: `xgen-mptest/tests/mp_g4_rejoin_e2e.rs`.** `#[tokio::test]` + `#[ignore = "heavy: …"]`, run out-of-band exactly as its siblings are:

```text
cargo build -p xgen-node && cargo build -p xgen-client
cargo test -p xgen-mptest --test mp_g4_rejoin_e2e -- --ignored --nocapture
```

🛑 **NOT `smoke-ph2`, and the reason is diagnostic, not stylistic.** `cmd_smoke_ph2`'s `fail!` macro calls `std::process::exit(1)` (`xgen-client/src/app.rs:1557-1565`) — it tells you a **step number** and dies. mptest has an oracle and a capture-by-default artifact dir. ***A gate that can only report WHICH step failed is a gate you cannot use to find out WHY.***

### The scenarios

| # | scenario |
|---|---|
| **V-9a** | 🎯 **THE LEG'S SUBJECT — REQUIRED.** alice creates an invite-only Space → invites bob → bob joins → bob leaves → **bob's `last_local_events` entry for that Space is removed from `xgen-client_state.json`** → bob rejoins. ✅ **Assert: `Accepted`, and bob is a member again.** 🔑 **That one map entry IS the fresh-install condition for this leg** — `rejoin_anchor_or_root` (`ops.rs:142-147`) reads exactly it, and its absence is what sent her to the create root. `ClientState.last_local_events` is `#[serde(default)]` (`xgen-common`), so removing the key is a legal state, not a corruption. |
| **V-9b** | ⚠️ **THE TRUE FRESH INSTALL — REQUIRED IF REACHABLE, AND ITS REACHABILITY IS UNMEASURED.** A clean data directory with bob's **same identity**, then rejoin. 🛑 **Whether the harness can re-seat an existing identity into an empty data dir HAS NOT BEEN MEASURED and this runbook does not assume it.** ✅ **Measure it and report.** If it cannot, **file it as a finding with the blocker named** — ***do not silently drop it and do not let `V-9a` stand in for it in the record***, because `V-9a` isolates the anchor path while `V-9b` is the thing a person actually does. |
| **V-9c** | 🛑 **`3048` ON A WIRE — THE FIRST TIME EVER.** Re-run `V-9a` with **G4-1 reverted**. ✅ **Assert the refusal arrives at the client** carrying `3048` / `rejoin_not_anchored`, and **quote the reply verbatim in the hand-back.** 🔑 **This is the leg's most valuable single artefact:** `3048` has existed in the node for two legs and **no client has ever received it**. ***A wire code nothing has ever received is a string, not a behaviour.*** |

### The rules this gate runs under

🛑 **`#[ignore]`, ALWAYS.** mptest's own constraint — *the fast unit suite must not spawn processes*. ⇒ **§5b contributes ZERO to the `cargo` floor**, and `V-10` must say so out loud.
⚠️ **A SPAWN OR CONNECT TIMEOUT IS A FLAKE, NOT A FAILURE** — re-run isolated before recording anything (`mp_r2_restart.rs`'s Rule 2). 🛑 **But a flake re-run until green and then recorded as green is `N-207`'s shape wearing a different hat: state HOW MANY RUNS the recorded result took.**
🛑 **THE HARNESS DRIVES AND OBSERVES; IT NEVER PATCHES.** A real defect surfaced here is **a finding routed to a fix-arc**, never patched under the G-4 banner.
🛑 **PORTS AND LABELS ARE PER-RUN.** Pick a port not used by the sibling tests; `instance_label` gives the data-dir isolation. **Kill-on-drop is `ManagedProcess`'s job — verify no orphan process or port survives the run**, and say so.

---

## §6 — WHAT THIS LEG MUST NOT DO

1. 🛑 **No `transport.*` variant, no wire-format change.** §4 of the Phase-0 ruled ②. The request and the served payload are untouched.
2. 🛑 **No change to `compute_frontier` or `cooperative_frontier`.** They serve every cooperative send.
3. 🛑 **No change to `get_dag_tips`, `rejoin_anchor_or_root`, or `collect_sync_history`.**
4. 🛑 **No node-side behaviour change.** G4-3 and G4-4 are **comment-only**; a behaviour hunk in either file is a deviation, not a rider.
5. 🛑 **No hand-written *does this event name me?* predicate on the client.** §1.
6. 🛑 **It must not widen what the node serves.** G-3 ruled ②; this leg reads the batch it is given.
7. 🛑 **It must not invent a client-side retry of a `3048`.** The reason string tells her to re-anchor; an automatic retry would loop against a gate whose whole job is to be heard.

---

## §7 — DoD

- [ ] G4-1 · G4-2 landed; `V-1`…`V-8` green; `V-8`'s two reverts run separately with different red sets, then restored, sha256-identical, mtimes stamped (`N-199`).
- [ ] G4-3 · G4-4 landed, comment-only, proven by `git show --stat` + a diff read.
- [ ] §5b `V-9a` green; `V-9b` green **or** filed as a finding with its blocker named and measured (never silently dropped).
- [ ] §5b `V-9c` run: `3048` observed on a wire, transcript quoted verbatim in the hand-back.
- [ ] `V-10` floors re-driven **independently by both seats on forced rebuilds**, `Compiling` present in both logs.
- [ ] `rejoin_anchor_or_root`'s doc annotated at the site (`D-131`), not deleted.
- [ ] Every `file:line` written into the source or the hand-back names the tree it was measured on (`D-152` clause 1).
- [ ] Deviations reported, never absorbed (Rule 6) — including any disagreement with this document.

*(No `commit pushed` item: it is unflippable inside the commit that pushes. `Status: COMPLETED` in this header is the canonical shipped signal.)*

---

## §8 — 🔓 OPEN DECISIONS, SITTING INSIDE THE RUNBOOK, BOTH ARMS SPECIFIED

### `OD-1` — ✅ **RULED (Joe, 2026-08-26): ARM A — THE LIVE RUN RIDES THIS LEG, IN `xgen-mptest`.** The question and both arms are kept below; the ruling is recorded at the site, and §5b is the executable form of it.

⚠️ **THE PRICE IN ARM A BELOW WAS WRONG AND IS CORRECTED HERE (`D-131`).** It read *the most expensive verification leg of the arc by a wide margin*, costed as **building a rig**. ✅ **Measured after the fact: there is no rig to build.** `xgen-mptest` spawns the real built `.exe`s as OS processes and drives them over `--aicontrol` JSONL — its own header names the purpose as *the in-process → real-binary crossing the existing harness cannot do* — and `ClientCommand::Join` is already exposed there (`xgen-client/src/aicontrol.rs:488`, above the not-exposed block at `:541`). 🔑 ***The recommendation was right and the reason given for its cost was not; both are recorded, because a recommendation accepted on a wrong price is a decision the record cannot audit.***

### `OD-1` — 🎯 **JOE'S (as originally put). Does the live two-identity wire run ride this leg, or wait?**

*A member left your Space. She reinstalls the app on a new machine — nothing of hers survives locally — and asks to come back. Everything built so far says she can. **Nobody has ever watched it happen.***

- **ARM A — it rides G-4.** ✅ **What you would see:** the arc's standing limit closes here, and `3048` is observed on a wire for the first time. **① user-visible impact:** this is the only arm in which anyone can say *rejoin from a fresh install works* and mean it; every claim so far is about what a function returns in-process. **② tier consequence: NONE** — nothing is copied, nothing destroyed, no `T4` floor touched. **③ cost:** a real node process, two identities, a scripted client run and a deliberate revert — **the most expensive verification leg of the arc by a wide margin**, and the first that can fail for environmental reasons rather than logical ones.
- **ARM B — G-4 ships in-process; the live run becomes its own leg before `G-5` closes.** **①** the arc closes with the same limit it has carried since `G-1`, now on a path a user can actually reach. **③** cheaper now, and the same cost later.

🎯 **CHAT RECOMMENDS ARM A.** ⚠️ **The honest caveat: it will cost the most time of anything in this arc, and it may fail for reasons that are not the code's.** 🔑 **The reason it is still the recommendation: `G-4` is the first leg in which a live rejoin path exists end to end, and a limit carried across four legs closes cheapest at the moment it becomes closable.**

### `OD-2` — **CHAT'S SEAT, RULED HERE, RECORDED SO IT IS VISIBLE.** Truncation order.

**RULED: batch-order-last, in G-4's own function (§3 step 4).** The rejected arm — reuse `compute_frontier` unchanged — is cheaper by one function and **keeps an arbitrary ten**. ⚠️ **Not routed to Joe: this is a mechanism with no user-visible fork** (both arms produce the same experience except in the >10 case, where one is simply likelier to be wrong). *Recorded rather than silently taken, per `D-065`.*

### 🔓 STILL OPEN, OLDER, NOT THIS LEG

- The `1011` reason string — a stranger's refusal and *you need a fresh invite* read identically. **`G-5`.**
- `G-2`'s reject reason string, shipped as drafted, **Joe's to overwrite in place. `G-5`.**
- `apply_invite`'s comment says an absent `valid_until` means no expiry, while both read gates refuse on absent for a regular Space. **Opened halfway, not claimed. `G-5`.**
- `D-154`⑥'s reversible-ejection record · `D-154`④'s third-party disclosure · `self.banned` as a permanent federated list. **Older and nobody's.**
