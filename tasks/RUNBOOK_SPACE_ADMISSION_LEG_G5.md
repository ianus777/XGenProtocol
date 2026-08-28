# RUNBOOK — M-SPACE-ADMISSION Leg G-5: the close leg
> **Status**: COMPLETED  
> Version: 1.3  
> Date: Aug 2026  
> **Last updated**: 2026-08-27  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS LEG IS

🔒 **LOCKED IN PLACE BY JOE, 2026-08-27 (v1.0 PENDING → v1.1 ACTIVE).** The four §1 rulings and this leg's sequencing are his; the split, the edits and the verification are Chat's seat (`D-123`). **It rides the close commit — it does not get one of its own** (Joe, 2026-08-27, on Chat's recommendation: this file changes no ROADMAP node state, G-5 was already declared in the G-0 leg table, and its records *are* the close).

The close leg of `M-SPACE-ADMISSION — who may join a Space, and how a leaver comes back`, and it closes the milestone itself, which has run since J-741.

It carries three things and nothing else: 🔧 the **riders** the arc accumulated and named but did not fix, 📌 the **records** that close the milestone under `D-074`, and 🔓 the **hand-offs** — what leaves this arc with a named owner instead of dissolving into the backlog.

🛑 **IT INTRODUCES NO NEW BEHAVIOUR.** Every edit below either corrects a false statement in a comment, or moves one verb into a lock it should already have been under. **No wire string changes. No predicate changes. No new test scenario.**

📌 **Sub-leg ordering is forced: `G-5a` before `G-5c`.** A record that cites a floor must cite the floor the riders produced, not the one they inherited.

---

## §1 — 🔒 THE RULINGS THIS RUNBOOK IS WRITTEN AGAINST (Joe, 2026-08-27)

Four questions were put in `D-155` form at session open. All four are ruled; **none is left open inside this runbook.**

| # | question | 🔒 ruling |
|---|---|---|
| **①** | She reinstalls on a new laptop with her keypair and no local state. What happens? | **C — the app tells her, in her words**, and does the restore behind one deliberate confirmation. **NOT B** (a silent launch-time write to node-side identity state is the one thing a person cannot audit). |
| **②** | Should a stranger and a welcome-back former member hear the same refusal? | **A — ONE word.** `1011 invite_bootstrap_refused` stays undifferentiated. Two words would be a **membership oracle**: anyone holding a pubkey could learn whether that identity was ever in that Space, by reading a refusal. |
| **③** | `3048`'s operator message. | **UNCHANGED.** It is read by an operator or a log, never by a member, and it names the remedy. |
| **④** | Does G-5 close the milestone? | **YES — with ①'s successor FILED, not built.** |

🔑 **① IS A RULING THIS LEG DOES NOT IMPLEMENT, AND THAT IS DELIBERATE.** The desktop client has **no route to `ops::join` at all** (J-740: three callers — CLI, AI control, tests; **no GUI caller**). A UI ruling with no surface to land on would be a `D` locked for a design nobody can see run — the shape refused at J-515. ⇒ ① is filed as **`M-CLIENT-RESTORE — bringing an account back on a new device`** (§6), and G-5 records the measured bound instead of papering over it.

🛑 **② IS RULED AGAINST THE BETTER EXPERIENCE, KNOWINGLY.** A real person whose invite expired is left with a message that tells her nothing. **The honest place to fix that is the invite she is asking for, not the refusal** — filed at §6, not solved here. The refusal's silence is a privacy property, and the caveat is carried rather than traded away (`D-065`).

---

## §2 — GROUNDING. **OPEN EACH SITE BEFORE YOU EDIT (`D-153`).**

Measured 2026-08-27 at `39cf7d3` (= `origin/main` by `git ls-remote`, tree clean). **Every line number below was opened, not recalled.**

| # | site | what is there now |
|---|---|---|
| **A** | `xgen-client/src/aicontrol.rs:154-156` | `fn mutates_state_file` returns true for `register` · `create-space` · `create-room` · `create-dm-space` · `self`. **Five names.** 🛑 **CORRECTED AT CLOSE (`D-131`, 2026-08-27) — ROWS A AND B ARE NOT THE SAME SET, AND PLACING THEM ADJACENTLY AS TWO FIVE-ITEM LISTS IS WHAT PRODUCED §3's FALSE INSTRUCTION.** The classifier holds `self`, which is NOT a direct writer; the writers hold `leave`, which was NOT classified. **The overlap is four, not five.** |
| **B** | `xgen-client/src/ops.rs` | `crate::app::write_client_state` has **five** call sites: `:466` (`register`, fn at `:373`) · `:732` (`create_space`, `:631`) · `:816` (`create_room`, `:754`) · `:1050` (`create_dm_space`, `:870`) · **`:1892` (`leave`, `:1823`)**. |
| **C** | `xgen-core/src/space/state.rs:156-160` | `PendingInvite`'s doc: *"`None` for invites that carry no `valid_until` … **such an invite never expires** (the join-acceptance / read gates only bite when the value is present and past)."* |
| **D** | `xgen-core/src/space/state.rs:1199-1201` | `apply_invite`'s inline comment: *"Absent ⇒ no expiry (the join/read gates only bite on a present, past value)."* |
| **E** | `xgen-core/src/node/runtime.rs:1836-1841` | The join gate: absent `valid_until` on a **non-DM** Space ⇒ `3044 invite_expired — non-DM invite carries no valid_until (malformed/legacy)`. |
| **F** | `xgen-node/src/fanout.rs:849, :897` | The read gate: same rule, same direction — absent/unparseable on a regular Space ⇒ refused. ⚠️ **CORRECTED AT CLOSE (`D-131`, Clair's `D-3`, 2026-08-27): NEITHER CITED LINE IS THE ENFORCEMENT.** `:849` is a comment; `:897` is the `match pending.valid_until` head. The arm that actually returns `Err(REFUSED)` is `:906` in this pre-edit tree (`:915` after `G5-4`). **The claim was true and the sites were not the site** — `D-153`'s shape at citation scale. |
| **G** | `xgen-mptest/tests/mp_g4_rejoin_e2e.rs:45-79` | *"STATUS AT HAND-BACK: **BOTH SCENARIOS FAIL**, AND THAT IS THE FINDING"* · *"§3's precedence is 🔒 Joe-locked"* · *"Not patched here"*. |
| **H** | `xgen-mptest/tests/mp_g4_rejoin_e2e.rs:34-37` | The `V-9c` disarm procedure names `(None, Some(key)) => select_rejoin_anchor(served, key)`. |
| **I** | `xgen-client/src/batch.rs:411-423` | The match arms are `(Some(key), _)` · `(None, Some(id))` · `(None, None)`. |

🛑 **C AND D ARE FALSE FOR A REGULAR SPACE, AND E AND F ARE WHY.** Absent `valid_until` means *never expires* **only on a DM** (`dm_constraints_active`, exempt by construction — `runtime.rs:4914-4917`). On every other Space **both gates refuse it.** The comment states the DM exception as the general rule.

🛑 **G IS AN `N-109`, AND IT IS SITTING IN THE FILE WHOSE WHOLE JOB IS TO BE THE SYSTEM GATE.** Both scenarios are **GREEN** (J-778, re-driven: `2 passed; 0 failed`), and the precedence the header calls Joe-locked was **DELETED at runbook v1.2** — `batch.rs:351` says so in capitals. *A stale honesty note is still a false statement, and it is worse than a missing one, because it was written by someone being careful.*

🛑 **AND H IS WORSE THAN STALE — IT IS UNFOLLOWABLE.** The arm it tells the next operator to edit **does not exist** (site I). `V-9c` is the negative control for the whole leg; **a procedure that cannot be executed is not a control.** ⇒ *A leg that edits a file invalidates its own citations into that file* — the J-778 finding, recurring inside J-778's own artefact.

---

## §3 — THE EDITS

### `G5-1` — `xgen-client/src/aicontrol.rs` — `leave` enters the state-file lock 🔧 **BEHAVIOUR**

Add `"leave"` to the `matches!` at `:155`, in the order it appears in `ops.rs` (after `create-dm-space`, before `self`). Update the doc comment at `:149-153` to name `leave` explicitly.

🛑 **CORRECTED AT CLOSE (`D-131`, Clair's `D-1`, BLOCKING, 2026-08-27). ~~"to say **five verbs**"~~ — THIS INSTRUCTION WOULD HAVE SHIPPED A FALSE COMMENT, IN THE LEG WHOSE WHOLE PURPOSE IS DELETING FALSE COMMENTS.** After adding `leave` the classifier holds **SIX**. *Five* is true of the direct writers and not of the classified verbs, and §2's rows A and B put the two five-item lists side by side as though they were one set. **The correct statement is: SIX classified, FIVE direct writers, overlap four.** 🔑 **And `self` is the reason the two sets differ, non-obviously:** `self_open` (`ops.rs:1087`) never calls `write_client_state` — it **delegates to `create_dm_space`** at `:1123` when the self thread is absent, so it writes conditionally and never by its own hand. ⇒ ***a reader diffing the classifier against a grep would "correct" `self` away — the same blindness that let `leave` escape, pointed the other way.*** That reasoning belongs in the doc comment, and is.

🔑 **WHY IT IS NOT COSMETIC.** `ops::leave` read-modify-writes `xgen-client_state.json` at `:1892`, and today it does so **outside `StateFileLock`**. What it writes is the **MP-F7 leave anchor** — the `last_local_events` entry that `select_rejoin_anchor`'s fallback reads. ⇒ **the exact record G-4 depends on is the one written unserialised.** The classifier's own comment already anticipates this: *"Revisit if `ops::*` grows a new state-file writer."* It grew one and nobody revisited.

🛑 **THIS IS THE ONLY EDIT IN THE LEG THAT CHANGES BEHAVIOUR.** It is a rider on a close leg, ruled so at J-778, and it must be **tested and measured**, not asserted.

### `G5-2` — `xgen-core/src/space/state.rs` — the `valid_until` comment tells the truth 📌 **COMMENT ONLY**

Both sites (C and D). The corrected statement, in the codebase's own vocabulary:

> Absent `valid_until` ⇒ **no expiry on a DM only.** On a regular Space both gates treat an absent value as malformed/legacy and refuse: the join gate with `3044` (`runtime.rs`, the `None if !space.dm_constraints_active` arm of the `pi.valid_until` match), the read gate in `collect_invite_bootstrap` (`fanout.rs`, the same arm of the `pending.valid_until` match). The DM exemption is structural — `dm_constraints_active` forecloses the invite path, so the absence of `valid_until` is the absence of the window it guards, not an omission.

🛑 **CORRECTED AT CLOSE (`D-131`, Clair's `D-2`, BLOCKING, 2026-08-27). ~~`runtime.rs:1836-1841`~~ — A LITERAL LINE CITATION HERE WOULD HAVE BEEN STALED BY `G5-4`, INSIDE THE SAME COMMIT.** `G5-4` inserts five comment lines above the `3048` emit, so the `3044` arm moves **`:1836` → `:1841`** — measured on both trees, not reasoned. 🔑 ***This is the J-778 finding — a leg that edits a file invalidates its own citations into that file — recurring inside G-5's own artefact, in the section that names it.*** ⇒ **anchor on the SYMBOL, no line number (`D-152` clause 1).**

📌 **Annotate, do not silently rewrite (`D-131`)** — the superseded sentence is struck at its site with the date, so a reader who remembers it can see it moved rather than vanished.

### `G5-3` — `xgen-mptest/tests/mp_g4_rejoin_e2e.rs` — the module doc stops lying 📌 **COMMENT ONLY**

Two things, and the second is the one that matters:

1. **§G — the hand-back status block (`:45-79`)** is rewritten as **history, dated and labelled as such**: it recorded a real RED state, the cause it names is correct and is now `D-156`, and the precedence it calls locked was deleted at v1.2. **Both scenarios pass at `39cf7d3`.** The block is kept — *the refutation is worth more than the erasure* (`D-065`) — and marked.
2. 🛑 **§H — the `V-9c` disarm procedure (`:34-37`) is re-pointed onto the shipped match.** Today: replace `(Some(key), _) => select_rejoin_anchor(served, key)` with `(Some(_), _) => vec![]` at `batch.rs:419`. Restore by **file copy**, never `git checkout --` (`N-210`), and require `Compiling xgen-client` before reading any result (`N-212`).

📌 **`V-9c` is not re-run in this leg.** It is a procedure, not a permanently-green test; **making it followable again is the deliverable**, and the leg says so rather than implying a run happened.

### `G5-4` — 📌 **THE TWO STRINGS: RULED UNCHANGED, AND THE RULING IS RECORDED AT THE SITE**

No string is edited. **Two short comments are added so the next reader does not re-open a closed question** (the J-513 lesson: *a resolved question that keeps advertising itself as open trains its readers to distrust the record*).

- `xgen-node/src/fanout.rs` at the `REFUSED` constant (`:824`): ② ruled **A** (Joe, 2026-08-27) — one word for every refusal case, **deliberately**, because differentiating it would make the refusal a membership oracle. The existing comment at `:895` already states the indistinguishability is intentional; this names the ruling and the date.
- `xgen-core/src/node/runtime.rs` at the `3048` emit (`:1803-1811`): ③ ruled **unchanged** (Joe, 2026-08-27) — Chat-drafted, reviewed, kept; operator-facing, names the remedy.

---

## §4 — VERIFICATION

🔒 **Rule 5 stands: Chat re-drives every number independently, on a FORCED REBUILD, with `Compiling` present for the crate before any result is read.**

| id | what it proves | how |
|---|---|---|
| **V-1** | `leave` is classified as a state-file mutator | A unit test in `aicontrol.rs`'s own test module asserting `mutates_state_file("leave")` is true **and** that a verb known not to write — `spaces` — is false. 🛑 **The negative arm is required: a classifier that returns true for everything passes the positive arm.** |
| **V-2** | The classifier matches the code, not a list | The same test asserts the **set**: exactly the five names, no more. ⇒ a sixth writer added later fails this test instead of silently escaping the lock. |
| **V-3** | The floor moved | `cargo test --workspace --no-fail-fast` run **detached**, PID polled in short calls, sentinel present, `^test result:` lines summed **CASE-SENSITIVELY** and programmatically. 🔒 **Expected 1665 → 1667 / 0 / 64 × 57 SUITES.** The delta is confirmed with `--skip` on the new test names, **never by arithmetic**. |
| **V-4** | `G5-2`/`G5-3`/`G5-4` changed no behaviour | By **scope**: `git show --stat` shows only comment hunks in `state.rs`, `mp_g4_rejoin_e2e.rs`, `fanout.rs`, `runtime.rs`. 📌 **An identical count over comment-only hunks is a scope argument, not a measurement** — state it as one. |
| **V-5** | `V-9c` is followable | Open `batch.rs:411-423` and confirm the arm the rewritten procedure names **exists verbatim**. 🛑 Read the file; do not re-derive it from this runbook. |
| **V-6** | The header no longer contradicts the build | `mp_g4_rejoin_e2e.rs` contains no undated present-tense claim that the scenarios fail. Read the painted text, not the diff. |

🔒 **CARRIED BY SCOPE, NOT RE-RUN** (zero `.ts`, zero `.svelte`, zero `ui/**`): vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15**. 🛑 **Sampler catalogue UNMEASURED — its harness has never been located; write no number for it.**

---

## §5 — WHAT THIS LEG MUST NOT DO

1. 🛑 **It must not change any wire string.** `1011` and `3048` are ruled unchanged (§1 ②③).
2. 🛑 **It must not implement ①.** `M-CLIENT-RESTORE` is filed, not scheduled. **No UI, no launch-time branch, no `register` auto-invocation.**
3. 🛑 **It must not widen `mutates_state_file` beyond `leave`.** Over-locking serialises concurrent network reads for no reason; the classifier's own comment says so.
4. 🛑 **It must not re-run `V-9c`, nor imply it was run.**
5. 🛑 **It must not delete the stale hand-back block.** Mark it as history; the refutation is the record (`D-065`).
6. 🛑 **It must not amend ch3.** The standing J-739 ruling governs; the `3048` registry row already shipped at G-2.
7. 🛑 **It must not touch `select_rejoin_anchor`, the gate predicate, or `collect_invite_bootstrap`'s authorization.** Those are G-1…G-4 and they are closed.

---

## §6 — 🔓 WHAT LEAVES THE ARC, WITH OWNERS NAMED

| item | owner | note |
|---|---|---|
| **`M-CLIENT-RESTORE` — bringing an account back on a new device** | **Joe to schedule** · Chat to Phase-0 | ①'s ruling C. **Name checked corpus-wide: 0 collisions.** Depends on the desktop client gaining any route to `ops::join` (J-740). Scope sketch: detect keypair-without-state at launch → one explicit confirmation → the existing `register --re-registration` path. 🛑 **The verb's own name is `re-registration` and it means *re-home onto a NEW node* (ch3 §3.13.8) — it is being reused for *rebuild local state on the SAME node*. Whether that is one mechanism or two is the Phase-0's first question, and it is not answered here.** |
| **The expired-invite dead end** | filed, nobody | ②'s carried caveat: a returning member whose invite expired hears a stranger's refusal and has no path to a new invite. **The fix belongs to the invite, not the refusal.** |
| `D-154`⑥ — a reversed ejection still leaves a durable federated record | filed, nobody | Accepted knowingly at J-766; not this arc's. |
| `D-154`④ — third-party disclosure to a returning member | filed, nobody | Accepted knowingly at J-769. |
| `self.banned` as a permanent federated list of identities | filed, nobody | Named at `D-154`⑥ and ⑤; owed its own look. |
| The `GENERIC_4000` envelope carrying `3048` in `reject_code` | filed, nobody | First sighting at J-778. **`D-070`'s entry, not this leg's.** |
| `N-204` heredoc truncation boundary UNBISECTED · `N-206`/`N-207`/`N-209` | filed, nobody | Instrument notes; carried forward in the kickoff, not in a milestone. |

---

## §7 — THE CLOSE

One commit, `D-074`, after `G5-1`'s floor is measured.

- `JOURNAL.md` → **J-779** — the close entry: the four rulings, the two false comments, the unfollowable control, the floor delta, and the arc's own measurement of itself.
- `docs/ROADMAP.md` → **v7.64**; the `M-SPACE-ADMISSION` node 🟡 → ✅, **REDUCED on completion per Joe's J-715 mechanism**. 📌 **MEASURED ON BOTH SIDES: 18 `↳` lines / 14,584 characters → 9 lines / 5,117 characters.** ⚠️ **THE FIGURE THIS RUNBOOK ORIGINALLY CARRIED — ~~*18 lines across `368-385`, ~25,095 characters*~~ — WAS WRONG, AND CHAT'S OWN (`D-131`, corrected 2026-08-27):** it summed lines `368-401`, **thirty-four lines running into the two NEXT milestones' nodes.** ***An off-by-sixteen-lines slice reads exactly like a measurement*** — which is why the splice asserted its own fence (first line matches `M-SPACE-ADMISSION`, last matches `trigger: SCHEDULED 2026-08-24`) and threw rather than cut on a miss. 🛑 `roadmap-format-gate.ps1` must exit 0 **before** the commit is staged — **re-driven: `PASS — tree lines 73..451 clean`, exit 0.**
- `CLAUDE.md` PLAY head → the G-5 close block. 🛑 **CRLF file** — read back and compare CR against LF after the write; **any literal text goes in a SINGLE-QUOTED here-string (`N-213`)**.
- `tasks/M_SPACE_ADMISSION_LEGG_PHASE0.md` → **COMPLETED**, §5's G-5 row filled.
- `tasks/M_SPACE_ADMISSION_PHASE0.md` → **COMPLETED**, §12's Leg G row closed.
- This file → **COMPLETED**.

---

## §8 — DoD

- [x] `G5-1` applied; `leave` present in `mutates_state_file`; doc comment states **six classified, five direct writers**, and explains `self` (corrected from *five verbs* at close — `D-131`)
- [x] `V-1` and `V-2` written and green, **including the negative arm and the exact-set arm**
- [x] `V-3` re-driven by Chat on a forced rebuild, `Compiling xgen-client` present, sentinel present, delta confirmed with `--skip` — **not arithmetic**
- [x] `G5-2` applied at both sites, superseded sentence struck not deleted (`D-131`)
- [x] `G5-3` applied; hand-back block marked as history; `V-9c` procedure re-pointed onto the shipped match arm
- [x] `V-5` — the named arm opened in `batch.rs` and confirmed verbatim
- [x] `V-6` — the painted module doc carries no present-tense false claim
- [x] `G5-4`'s two ruling comments in place, dated, attributed to Joe
- [x] `V-4` — scope stated as a scope argument, never as a measurement
- [x] §6's seven hand-offs each carry a named owner or an explicit *nobody*
- [x] §7's six documents updated; `roadmap-format-gate.ps1` exit 0; CRLF integrity re-asserted on both CRLF files
- [x] One commit, `D-074`

📌 **"Commit pushed" is deliberately not a DoD item** — `Status: COMPLETED` in this header is the shipped signal.

---

## §9 — 🔓 OPEN DECISIONS INSIDE THIS RUNBOOK

**None.** All four of §1's questions are ruled. 📌 Stated explicitly rather than left absent, so a reader can tell the difference between *nothing was open* and *nobody looked*.
