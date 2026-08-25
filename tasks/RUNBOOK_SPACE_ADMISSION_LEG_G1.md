# RUNBOOK — M-SPACE-ADMISSION Leg G-1: the gate's third term
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-25  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS LEG IS

**`M-SPACE-ADMISSION` Leg G-1 — the gate's third term.** The dispatch-level admission gate implements two of the three terms its own design specifies, so **a returning member is refused `3047` before the applier that would admit her is ever reached.** This leg adds the missing term.

📌 **Phase-0:** `tasks/M_SPACE_ADMISSION_LEGG_PHASE0.md` v1.1 §2.1. **Design:** `tasks/M_SPACE_ADMISSION_PHASE0.md` v3.0 §15.4 (annotated at the site) + `D-154`①②③ + `Q-2`(a).

🛑 **THIS RUNBOOK IS CLAIR'S. 🔒 LOCKED BY JOE 2026-08-24 — IMPLEMENT FROM THIS VERSION, IN A SESSION OPENED BY HER OWN KICKOFF.** Deviations are **reported, never absorbed** (Rule 6) — this arc's implementing seat found seven specification defects and absorbed none, and this document is written by the seat that produced all seven.

🔒 **EVERY CITATION BELOW IS ANCHORED AT `f66a2cb`** (`D-152` clause 1). **Where a line number is given it is a convenience; the INVARIANT ANCHOR is the quoted comment or symbol, and that is what Clair matches on.** ✅ **VERIFIED AT LOCK: `git diff --name-only f66a2cb..8e41273` lists FIVE files and NOT ONE is a `.rs`** — every citation here is still live against `HEAD`. 🔑 ***That is a measurement, not an assumption: a file:line written into a document without its tree has a half-life measured in commits.***

---

## §1 — SCOPE

| | |
|---|---|
| **Files that may change** | `xgen-core/src/node/runtime.rs` (the predicate) · `xgen-node/src/tests/space_admission_gate.rs` (the tests) |
| **Files that must NOT change** | `xgen-core/src/space/state.rs` · `xgen-node/src/fanout.rs` · `xgen-client/**` · `xgen-common/**` · `docs/**` · `ui/**` |
| **Wire surface** | **NONE.** No new reject code, no new variant, no ch3 edit. `3047` is already live and already named. |
| **Expected diff** | ~3 production lines plus a comment block; one new test function plus its controls. |

🛑 **NO NEW REJECT CODE.** The refusal this leg REMOVES is `3047`'s; the refusals it must preserve are `3047`'s and the banned pre-check's. **`3048` is `Leg G-2` and is not in this leg.**

---

## §2 — GROUNDING. **OPEN EACH SITE BEFORE YOU EDIT IT (`D-153`).**

| # | fact | anchor |
|---|---|---|
| **G-1** | The gate's predicate is `space.admission != ADMISSION_OPEN && !space.pending_invites.contains_key(&event.sender)` | `runtime.rs:1620-1621`, inside the block whose banner reads *"M-SPACE-ADMISSION Leg D (D-1) — THE ADMISSION GATE, and it runs BEFORE the expiry check because it asks the prior question"* |
| **G-2** | The enclosing condition is `if origin == EventOrigin::LocallySubmitted && event.room_id.as_str().is_empty()` | `runtime.rs:1580` |
| **G-3** | **`left_at` has ZERO non-test occurrences in `runtime.rs`** — three hits, all `#[cfg(test)]` | `runtime.rs:3720`, `:3751`, `:5904` |
| **G-4** | `apply_join` DOES carry the term, and its spelling is `self.members.get(joiner).is_some_and(\|m\| m.is_present())` under the comment *"the three-way gate"* | `state.rs:1214`, the Space-level branch |
| **G-5** | `SpaceMember::is_present()` is `self.left_at.is_none()`, documented as *"the single definition of the predicate every reader of `SpaceState::members` gates on — `D-067`: one fact, one place"* | `state.rs:121` |
| **G-6** | 🔒 **THE BANNED PRE-CHECK ALREADY RUNS, AND IT RUNS FIRST.** `if space.banned.contains(&event.sender)` sits at `runtime.rs:1523`, **ABOVE** the `LocallySubmitted` block at `:1580`, under the banner *"MP-F6 (M10.5-D2/D3) — dispatch-level banned pre-check"* | `runtime.rs:1505-1531` |
| **G-7** | **A KICK IS NOT A BAN.** `apply_kick` calls `mark_departed` and **does not touch `self.banned`**; only `apply_ban` and `apply_node_eject` insert there | `state.rs`, `fn apply_kick` |
| **G-8** | Both DM constructors pin `admission = invite` unconditionally, at **fold** time | `state.rs` `from_dm_space_create` / `from_dm_space_create_node` |

🔑 **G-6 IS THE LOAD-BEARING ONE AND IT DECIDES THE SHAPE OF THE EDIT.** A banned identity is refused **before** the admission gate is reached, so **the new term needs no ban clause.** 🛑 **ADDING ONE WOULD BE A SECOND SOURCE OF TRUTH FOR *is this person banned* — `D-067`'s exact target — and it would be the third such site in one function.** ✅ **It is asserted by a CONTROL (§4, `V-3`), not by a code path.**

🔑 **G-7 + G-6 TOGETHER GIVE THE CORRECT BEHAVIOUR FOR FREE:** a **kicked** member is departed and not banned ⇒ the new term admits her (`D-154`②③ — a kick is remembered, a ban bars) · a **banned** or **node-ejected** member is departed **and** banned ⇒ `:1523` refuses her first. **Nothing in this leg has to know the difference.**

---

## §3 — THE EDIT

### G1-1 — the third term

🔒 **The refusal condition gains one conjunct: the sender must not be a RETAINED DEPARTED member.**

```
if space.admission != ADMISSION_OPEN
    && !space.pending_invites.contains_key(&event.sender)
    && !space.members.get(&event.sender).is_some_and(|m| !m.is_present())
{
```

🔒 **THE SPELLING IS `apply_join`'s, DELIBERATELY AND NOT INCIDENTALLY.** `G-4` and `G-5` establish `is_present()` as the single definition. **Do not write `left_at.is_some()` inline, do not add a `has_departed()` accessor, and do not reach for `is_member`** — `is_member` answers the present-tense question and would be `false` for both a former member and a stranger, which is precisely the distinction this term exists to draw. ***Two spellings of one predicate is how the two sites drift apart.***

🔒 **THE COMMENT IS PART OF THE EDIT, NOT DECORATION.** It must state, in the source:

1. **What the term admits:** a member whose record is retained and marked departed — `D-154`①, `Q-2`(a): *a former member is re-admitted without an invite.*
2. **Why there is no ban clause:** the banned pre-check at the *"MP-F6 … dispatch-level banned pre-check"* banner runs earlier in the same function; **a second one here would be a second source of truth.**
3. **Why a kick needs no special case:** `apply_kick` marks and does not ban, so `D-154`②③ fall out of the two checks already present.
4. **That the term is `!is_present()`, NOT `contains_key`:** a *present* member falls through to the applier, which answers `AlreadyMember`.

🛑 **ANNOTATION AT THE SITE (`D-131`, J-775, 2026-08-25) — POINT 4 IS FALSE ON BOTH PATHS, AND §6 OF THIS SAME DOCUMENT SAYS SO. IT WAS REFUSED BY THE IMPLEMENTING SEAT AND NEVER REACHED THE SOURCE.** ✅ **Traced by Clair and re-driven by Chat:** in an **invite-only** Space all three conjuncts hold for a present member, so **this gate refuses her `3047` and she never reaches the applier**; in an **open** Space she does reach it, and `apply_join`'s `Err(AlreadyMember)` is discarded at **`runtime.rs:867`** (`let _ = state.apply_event(&event, &my_node_id)`), so **she is answered `Accepted`.** 🔑 **The shipped comment says the true thing instead: the term is `!is_present()`, a present member is deliberately NOT admitted, and §6's consequence is recorded at the site with its reason.** 🔑 ***This is the same shape as the arc's `§2c` defect — a document refuted by its own later paragraph — and it is the SECOND time that shape has produced a defect here. Neither fell to a re-read; both fell to reading against a compiler.***

⚠️ **IF THE BORROW CHECKER OBJECTS**, `space` is already a shared borrow used by both neighbouring conditions — **report it rather than restructuring the block**; a restructure of a gate whose ordering is load-bearing is not a mechanical fix.

---

## §4 — VERIFICATION. **EVERY GATE HAS A NEGATIVE CONTROL, AND THE CONTROLS ARE THE POINT.**

📌 **The tests go in `xgen-node/src/tests/space_admission_gate.rs`, and they follow ITS existing shape**: through `submit_locally`, asserting on the `DispatchOutcome` **the sender receives**, plus the membership that followed. 🔑 **Its module doc already states why** — *a gate the sender never hears about admits nobody while telling them they got in.* **The fixture model is `uninvited_join_into_an_invite_only_space_is_rejected_3047_to_the_sender`**: real create path via `build_space_create_event_with_admission`, all identities registered, `space_level_ev` for correct `prev_events` chaining.

| # | check | requirement |
|---|---|---|
| **V-1** | **THE SUBJECT.** Invite-only Space · bob is invited, joins, **leaves** · bob re-submits `membership.join`. | **`Accepted`**, and bob is **present** afterwards: `members.get(bob).is_present()` true, `left_at` **`None`**, `role == Member`, `invited_by == None` (`D-154`① — role RE-DERIVED). |
| **V-2** | **CONTROL A — THE STRANGER.** In the same Space, carol has **never** been a member and holds no invite. | **`Rejected`, code `3047`.** ⚠️ **Without this the term could be a blanket *admit everyone* and V-1 would still pass.** |
| **V-3** | **CONTROL B — THE BANNED FORMER MEMBER.** dave joins, then is **banned**; dave re-submits. | **`Rejected`**, and the refusal is the **banned pre-check's** shape (`PermissionDenied`-class, unmapped `4000`), **NOT `3047`.** 🔑 **This asserts `G-6`'s ORDERING rather than assuming it, and it is what makes the absent ban clause a measured decision instead of an omission.** |
| **V-4** | **CONTROL C — THE KICKED MEMBER.** erin joins, is **kicked** by the owner, re-submits. | **`Accepted`**, present afterwards. **`D-154`②③ — a kick is remembered, a ban bars; the difference must be visible on the answer path.** |
| **V-5** | **CONTROL D — THE OPEN SPACE IS UNTOUCHED.** The existing `third_party_registered_identity_joins_an_open_space` must stay **GREEN and UNEDITED.** 🛑 **ANNOTATION AT THE SITE (`D-131`, J-775): AS WRITTEN THIS WAS UNFALSIFIABLE — that test lives in `xgen-node/src/tests/space_admission_third_party_join.rs:73`, OUTSIDE §1's may-change list, so it is unedited BY CONSTRUCTION and could not have failed.** ✅ **Clair re-seated it on THIS file's own open-space companion, `the_same_uninvited_join_into_an_open_space_is_admitted`, which is genuinely at risk from the edit; green and byte-untouched in both seats' runs.** 🔑 ***A control that cannot fail is not a control.*** | 🛑 **If it must be weakened, that is a FINDING about the term's scope, reported and never absorbed** — `L-E` says a Space at the default `open` is not closed by anything this milestone does. **No such finding was owed: the scope is intact.** |
| **V-6** | **RED-ON-REVERT, RUN AND RECORDED.** Delete the new conjunct; re-run. | **V-1 and V-4 must go RED, and RED WITH `3047`** — not merely fail. 🛑 **A test that goes red for a different reason has not tested this term.** ✅ **Record the observed code, not just the failure.** |
| **V-7** | **THE DM CASE, because it is the whole reason `Q-2`(a) was ruled.** A DM: two parties, one leaves, then rejoins. | **`Accepted`.** 📌 **The DM constructors pin `invite` at FOLD time (`G-8`), so this is the same gate with no extra setup** — and before this leg it is the case where **departure is irreversible for both parties.** |

🔒 **THE FLOOR MUST MOVE.** `cargo` is **1641 / 0 / 62 × 56 SUITES** at `f66a2cb`. **A green run that leaves it at 1641 means the tests were not collected.** 🔒 **The delta is MEASURED, not arithmetic:** run the workspace with `--skip` on the new test names and require it to return **exactly 1641** with libtest's own `filtered out` count matching the number of new tests.

🛑 **RUN DISCIPLINE (measured the hard way, not folklore):** `cargo test` **exceeds the MCP call budget** — run it **detached**, poll the PID in **separate short calls**, and use a **sentinel**; *a detached run's notification exit code is the LAUNCHER's, not cargo's.* **Sum `^test result:` CASE-SENSITIVELY** and pass **`--no-fail-fast`**, or the run reports a fraction of the suites. **A missing final `test result:` line means the run was truncated, and a truncated run leaves a measurement-shaped artefact.**

---

## §5 — WHAT THIS LEG MUST NOT DO

1. 🛑 **No ban clause in the new term** (§2 `G-6`). It is asserted by `V-3`, not implemented twice.
2. 🛑 **No new accessor on `SpaceMember`.** `is_present()` exists and is the single definition (`D-067`).
3. 🛑 **No touch to `state.rs`.** `apply_join` is already correct; this leg makes the gate agree with it.
4. 🛑 **No `3048`, no `conflicts_in_log`, no anchor work.** That is `G-2` and `G-3`/`G-4`.
5. 🛑 **No edit to `third_party_registered_identity_joins_an_open_space`** (`V-5`).
6. 🛑 **No re-adjudication of a federated join.** The enclosing `LocallySubmitted` condition (`G-2`) stays **structural** — do not add an origin test to the new conjunct.
7. 🛑 **No ch3 edit and no `docs/**` edit.** Records are `G-5`'s.

---

## §6 — NAMED AND DELIBERATELY NOT FIXED

⚠️ **A PRESENT MEMBER RE-JOINING AN INVITE-ONLY SPACE IS REFUSED `3047`, NOT `AlreadyMember`** — she holds no pending invite, `admission != open`, and the new term does not admit her because she is present. **The refusal is CORRECT in outcome and MIS-NAMED in reason.**

🔑 **IT IS NOT FIXED HERE, AND THE REASON IS THE SAFER SIDE OF A KNOWN TRAP:** admitting her past the gate would hand the refusal to `apply_join`, **whose error every production call site discards** (`let _ = ...apply_event`) ⇒ she would be answered **`Accepted`** and dropped. ***A wrong reject code is a smaller defect than a reply that lies.*** 📌 **Filed for `G-5`'s record sweep, not for this leg.**

---

## §7 — DoD

- [x] `G1-1` shipped in `xgen-core/src/node/runtime.rs`, comment carrying all four points of §3 — 🛑 **THREE of the four; point 4 was FALSE and was correctly refused (see the §3 annotation). The comment carries the true statement in its place.**
- [x] `V-1` … `V-7` all run and green, in `xgen-node/src/tests/space_admission_gate.rs` — **as THREE test functions, not one; §1's estimate was not a constraint**
- [x] `V-3` records the OBSERVED refusal shape — **`code: 4000, name: "generic"`, reason *step 13: permission denied … is banned from Space …*. NOT `3047`. Observed via a throwaway flipped-expectation probe, reverted, zero residue.**
- [x] `V-6` run: the conjunct deleted, **all three new tests RED with `RejectInfo { code: 3047, name: "admission_required" }`, both pre-existing tests GREEN** — that split IS the isolation. `Compiling xgen-core` present. Restored; sha256 identical, mtime stamped forward (`N-199`).
- [x] `cargo` **1641 → 1644 / 0 / 62 × 56 SUITES**, delta measured with `--skip` (**exactly 1641**) and libtest's **`filtered out = 3`** cross-checked — ✅ **BOTH SEATS, INDEPENDENTLY (Rule 5)**
- [x] `vitest` and `svelte-check` carried **by scope** — `git diff --name-only` returns exactly two paths, zero `ui/**`, zero `.ts`, zero `.svelte`
- [x] **Three deviations reported, none absorbed (Rule 6) — all three CORRECT, and two of them were defects in THIS DOCUMENT**
- [x] Chat re-drove every number independently from `HEAD` `c94800d` before the close (Rule 5)

📌 **"Commit pushed" is deliberately not a DoD item** — `Status: COMPLETED` in this header is the shipped signal. **Clair never pushes.**
