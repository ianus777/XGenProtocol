# RUNBOOK — M-SPACE-ADMISSION Leg E-1: the meaning change — `left_at`, the four writes, and all fifty readers in one commit

> **Status**: ACTIVE  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-23  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — LOCK STATE

| | |
|---|---|
| **Phase-0** | `tasks/M_SPACE_ADMISSION_LEGE_PHASE0.md` **v1.1 ACTIVE** — read §4b, §5d, §6e, §7 before this file |
| **Status** | 🔒 **ACTIVE, LOCKED (Joe, 2026-08-23; J-766).** Clair implements from **v1.1** and no earlier revision |
| **Blocking on** | ✅ **NOTHING.** Phase-0 §8 RULED **(a)** — `apply_node_eject` retains and marks. Recorded as **`D-154`⑥**, an amendment at `D-154`'s own table, not a new `D` |
| **Tree** | every citation measured at **`72262f6`** = `origin/main`, tree clean (`D-152`) |
| **Floors in** | cargo **1623 / 0 / 62 × 56 SUITES** · vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15** · catalogue **UNMEASURED** |
| **Rule 5** | Chat re-drives **every** gate from `HEAD`. Nothing in §6 is adopted on report |
| **Rule 6** | Clair reports deviations and never absorbs them. **Leg D's `D-3` was caught this way and the report was correct** |

---

## §1 — WHAT E-1 IS

🎯 **The single commit in which `SpaceState.members` stops being a set of people who are here and becomes a record with a lifecycle.**

🔑 **WHY IT IS ONE COMMIT AND NOT TWO.** `E-0` §5e measured that `SpaceState.members` is **purely present-tense** — *no existing reader's meaning is preserved by `(g)`, so all 50 change at once.* A commit that changes the writes without the readers leaves **a departed member reading as present at fifty sites**, and existing tests assert the opposite **by name** (§2c). ⇒ ***a leg boundary there would be a half-applied migration with a commit in the middle of it.***

## §1b — WHAT E-1 IS NOT

1. ❌ **No history slicing.** Clause ④ is **E-2**, at `fanout.rs:276-289`. E-1 does not touch `fanout.rs` except `:272`'s recipient list.
2. ❌ **No `get_rejoin_anchor`.** Leg G.
3. 🛑 **No change to `build_membership_event`** (`state.rs:2131`). Phase-0 §4b: the empty `prev_events` is the **documented root-adjacent contract**, not a defect. **Emitting a chain there breaks three callers.**
4. ❌ **No `ui/**`, no `xgen-client` appearance.** `ops.rs`'s projection sites are data and are in scope; how a departed member *renders* is Joe's.
5. ❌ **No `C-6`** (`migration/state_machine.rs:233`). Filed non-blocking, another owner.
6. ❌ **No re-litigation** of `(g)`, `(i)`, `Q-2`(a) or any `D-154` clause.

---

## §2 — GROUNDING. **EVERY LINE MEASURED AT `72262f6`.**

### §2a — the structure

`xgen-core/src/space/state.rs:85-95` — `SpaceMember` has **four** fields: `identity_id`, `role`, `joined_at`, `invited_by`. **No departure marker. `left_at` has zero occurrences in any `.rs` file.**

`state.rs:232` — `/// Active members: identity_id → SpaceMember` (**`C-7`**).

### §2b — the four writes and the one rejoin write

| site | today |
|---|---|
| `apply_leave` `state.rs:1195` | `:1203` `self.members.remove(leaver)` → `NotASpaceMember` if absent · `:1206-1208` strips every `RoomState.members` |
| `apply_kick` `state.rs:1212` | `:1230` `self.members.remove(&target)` · `:1231-1233` room strip |
| `apply_ban` `state.rs:1237` | `:1250` `self.members.remove` · `:1251` `pending_invites.remove` · `:1252` `banned.insert` · `:1253-1255` room strip |
| `apply_node_eject` `state.rs:1265` | `:1275` `self.members.remove` · `:1276` `pending_invites.remove` · **`:1277` `self.banned.insert`** · `:1278-1280` room strip — 🔒 **RULED (a): retains and marks (`D-154`⑥)** |
| `apply_join` `state.rs:1155` | `:1173` `contains_key` ⇒ `AlreadyMember` · `:1176` `banned.contains` ⇒ `Banned` · `:1179-1182` `pending_invites.remove` ⇒ `(role, invited_by)`, **absent ⇒ `(Role::Member, None)`** · `:1183-1192` `members.insert(...)` |

🔑 **`N-201` (Phase-0 §6e): with `left_at: None` in the literal, the EXISTING `insert` performs `D-154` clause ① exactly** — `left_at` cleared, `joined_at` re-stamped, role and `invited_by` re-derived from `pending_invites`. **The replace is not the defect; it IS the ruling.** ⇒ **`E1-5` is a gate edit, not a write edit.**

### §2c — the tests that assert the opposite and MUST be edited

🛑 **These are not incidental. Each asserts *departed ⇒ absent from `members`*, which is exactly what `(g)` inverts.** A run that leaves them untouched and green would mean the change did not land.

| test | site | what it asserts today |
|---|---|---|
| `resolve_operator_skips_delegate_who_left_falls_back_to_inviter` | `state.rs:4179` | 🔑 **the one no accessor ruling reaches** — `resolve_operator` reads `self.members` directly (`D-4`) |
| — | `state.rs:3044` | `!is_member(bob)` after leave |
| — | `state.rs:3117` | `!is_member(bob)` |
| — | `state.rs:4063` | *"kicked member removed"* |
| — | `state.rs:4192` | `!is_member(carol)` |
| — | `runtime.rs:5885` | *"bob has left the DM"* |
| — | `derive.rs:1071` | *"bob not a Space member"* |

⚠️ **THE LIST IS A STARTING SET, NOT A CLOSED ONE.** It was produced by a `git grep` for `is_member` / `members.contains_key` in negative assertions. **Clair enumerates the full set from the compiler and the red run, reports the total, and does not assume this table is complete** (`N-197`: a list that looks authoritative and is not).

🛑 **AND THE EDIT IS SEMANTIC, NOT MECHANICAL.** `!is_member(bob)` **stays true** under `(i)`, because `is_member` gains the `left_at.is_none()` gate. **What changes is `members.contains_key(bob)`, which becomes `true`.** ⇒ ***a test asserting the person is gone should now assert BOTH: `is_member` false AND the record retained with `left_at` set.*** **An edit that only makes red go green has deleted the leg's own evidence.**

### §2d — the four production `SpaceMember` literals

`state.rs:343` · `state.rs:478` · `state.rs:630` (three constructors) · `state.rs:1185` (`apply_join`).
**Test literals:** `algorithm.rs` ×12 (`:408, 507, 531, 537, 559, 588, 594, 633, 639, 708, 752, 758`) · `admin_ops.rs:6061`.

✅ **Adding a field with no `Default` makes the compiler enumerate every site** — *a change that cannot silently miss one.* 📌 Recorded because it is the inverse of `N-197` and is the reason no census is needed for this edit.

---

## §3 — THE EDITS

### 🔒 `E1-1` — the field

`state.rs:85-95`. Add, after `invited_by`:

- `pub left_at: Option<String>` — **`None` = present; `Some(ts)` = departed at `ts`.**
- Doc comment states, in this order: **what it means** · **that `is_member` and `member_role` gate on it** · **that a rejoin CLEARS it (`D-154`①)** · **that the departure HISTORY is NOT here — it is derived from the log at delivery time (Phase-0 §5d(C))**.

🛑 **The last clause is load-bearing.** Without it the next reader adds an absence list to this struct, which is the `former_members` GDPR shape `§6.5` refused.

`left_at: None` at all four production literals (§2d) and every test literal the compiler names.

### 🔒 `E1-2` — `C-7`, the doc comment

`state.rs:232` — `/// Active members` becomes accurate: the map holds **every identity the Space has admitted**, present and departed; **presence is `left_at.is_none()`**.

### 🔒 `E1-3` — `apply_leave` retains and marks

`state.rs:1203`. `self.members.remove(leaver)` becomes a `get_mut` + mark:

- absent ⇒ `Err(SpaceError::NotASpaceMember)` — **unchanged**
- present with `left_at.is_some()` ⇒ ⚠️ **`Err(NotASpaceMember)`.** *A second leave from someone already gone is not a success, and it must NOT overwrite the first `left_at` — that would move a boundary E-2 depends on.*
- present with `left_at.is_none()` ⇒ `left_at = Some(event.timestamp.clone())`

`:1206-1208`'s room strip is **UNCHANGED** — `D-154`⑤ is already satisfied by it (Phase-0 §4).

### 🔒 `E1-4` — `apply_kick` and `apply_ban` retain and mark (`D-154`②③)

Same shape as `E1-3` at `state.rs:1230` and `state.rs:1250`, marking the **target**, not the sender.
`apply_ban`'s `:1251` `pending_invites.remove`, `:1252` `banned.insert` and room strip are **UNCHANGED** — `self.banned` stays the authority for the ban (`D-154`③).

🔒 **`apply_node_eject` (`state.rs:1275`) — RULED (a) (Joe, 2026-08-23; `D-154`⑥):** the same shape as `apply_kick`, marking `content["target_identity"]`. **`:1276` `pending_invites.remove`, `:1277` `banned.insert` and `:1278-1280`'s room strip are UNCHANGED.** ⚠️ **A doc comment at the site records that this path retains BECAUSE it also bans** (`:1277`) — *the reason must travel with the edit, or the next reader re-derives it.* **The refused arm is struck, not deleted (`D-131`):**

| arm | edit |
|---|---|
| ~~**(a)** *retain and mark*~~ | ✅ **TAKEN** — identical to `apply_kick`; `:1276`'s `pending_invites.remove` unchanged |
| ~~**(b)** *keep removing*~~ | 🛑 **REFUSED** — ~~no edit, and a doc comment at `:1275` states outright that this site deliberately diverges from `apply_leave`/`kick`/`ban`, and why~~ |

✅ **THE DIVERGENCE's ABSENCE IS STILL WRITTEN DOWN AT THE SITE.** *A site that matches its neighbours because it was RULED to is, in the code, indistinguishable from one that matches them by accident — so the doc comment says which.*

🔑 **A FACT MEASURED WHILE WRITING THIS FILE, AND IT LEANS ON §8: `apply_node_eject` ALSO BANS** — `state.rs:1277` `self.banned.insert(target)`. ⇒ ***its end state is `apply_ban`'s end state***, and `D-154`③ already rules that a ban retains. **Retaining for `apply_ban` and removing for `apply_node_eject` would draw a line between two paths that reach the identical state**, which is Chat's strongest ground for arm (a). ✅ **It was put in front of the ruling and not used to make it; Joe ruled (a) on 2026-08-23.** 📌 Kept as written, because *the fact came before the ruling* is part of the record.

### 🔒 `E1-5` — `apply_join`: the gate, and the ban check that comes alive (`D-3`, `V-3c`)

`state.rs:1173`. The bare `contains_key` becomes:

- record absent ⇒ fall through to `:1176`
- record present, **`left_at.is_none()`** ⇒ `Err(AlreadyMember)` — **today's behaviour, preserved**
- record present, **`left_at.is_some()`** ⇒ **fall through** — this is the rejoin

⇒ `:1176`'s `banned.contains` is now reached by a retained banned member. 🔑 ***Without this ordering, `D-154`②③ make the ban check dead code for exactly the people it exists for: `AlreadyMember` instead of `Banned` — a refusal that looks green and is the wrong refusal.***

📌 **`:1183`'s `insert` is UNCHANGED** beyond `E1-1`'s field (`N-201`). **`:1161`'s room-level guard is UNCHANGED** — a room join by a departed member must still fail, and it now fails on `left_at` via `E1-6`'s accessor rather than on absence. ⚠️ **Clair confirms this by reading, not by assuming: `:1161` is a direct `contains_key` and `(i)` does NOT reach it.** ⇒ **`:1161` gets the same three-way treatment as `:1173`**, refusing a departed member.

### 🔒 `E1-6` — the two accessors (`(i)`) — **30 sites carried**

`state.rs:1434` `member_role` and `state.rs:1441` `is_member`: both return the *present-tense* answer — a member with `left_at.is_some()` is **not** a member and has **no** role.

✅ **This alone discharges `C-3`** (`runtime.rs:1717` reads `is_member`; a rejoiner now reads `already_member = false` ⇒ `new_joiner = Some` ⇒ the push fires) **and `C-5`'s second half** (`fanout.rs:488` `collect_sync_history` — a departed member stops pulling history). 📌 **Both are consequences to be VERIFIED (§6 `V-4`), not claims.**

### 🔒 `E1-7` — the twenty direct readers (`D-3`)

Each gains `left_at.is_none()`. **Grouped by what a wrong answer does:**

| group | sites | what breaks without the edit |
|---|---|---|
| 🛑 **privacy** | `fanout.rs:272` (**`C-5`**) | a departed member keeps receiving every event |
| 🛑 **federation + convergence** | `runtime.rs:2312` (**`C-4`**) | her home Node keeps receiving federated DM traffic. ⚠️ `:2301-2303` demands a byte-identical `Vec<NodeXgid>` for `assert_converges` — the sort is **after** the filter, so filtering does not disturb it |
| 🛑 **authority** | `state.rs:1407, 1412, 1417, 1419, 1426` (**`D-4`**, `resolve_operator`) | a departed delegate or inviter resolves as operator. 📌 **`CLAUDE.md:1470` documents this fn as *"transparently skips members who left"* — the record goes false at a line nobody edits unless it is edited here** |
| ⚠️ **projection** | `ops.rs:2573, 2591, 2595, 2606, 2736` · `admin_ops.rs:3460` · `node/app.rs:4045` | departed members in the roster and in every member count |
| ⚠️ **DM logic** | `runtime.rs:2158` · `dm_promotion.rs:80, 130` | a departed party counted as the counterpart. ⚠️ `:80` is `.keys().find()` — the filter goes inside the closure |
| ✅ **gates, done in `E1-5`** | `state.rs:1161, 1173` | — |
| 📌 **out of scope** | `migration/state_machine.rs:233` (**`C-6`**) | filed, not this leg |

🛑 **`ops.rs:2573`, `:2591`, `:2595`, `:2736` are rustfmt-broken chains** (`.members` alone on its line). **A line-oriented grep does not see them** (`F-3`'s species). Clair works from this table, not from a fresh grep.

### 🔒 `E1-8` — the tests

1. **Every test in §2c edited to the two-sided assertion** (§2c's last paragraph): `is_member` false **and** the record retained with `left_at` set.
2. **New: leave retains.** After `membership.leave`, `members.contains_key(bob)` is `true`, `left_at.is_some()`, `is_member(bob)` is `false`.
3. **New: rejoin is `D-154`①.** leave → join ⇒ `left_at` `None`, `joined_at` == the rejoin event's timestamp, **`role == Role::Member` for a departed Owner**, `invited_by == None`.
4. **New: `V-3c`'s state — the control Leg D could not reach.** ban ⇒ retained + marked; a subsequent join is refused **`Banned`**, *not* `AlreadyMember`.
5. **New: double leave.** A second `membership.leave` errors and **does not move `left_at`**.
6. **New: `C-5`.** `fanout.rs:272`'s recipient list excludes a departed member.
7. **New: `C-4`.** `repopulate_dm_federation_nodes` drops a departed party's node, and the resulting `Vec<NodeXgid>` is still sorted.
8. **New: `D-4`.** A delegate who left falls through to the inviter — *i.e. `state.rs:4179` re-expressed against retention rather than deleted.*

🛑 **FIXTURE RULE (Phase-0 §4b — `F-E`'s real content).** A non-root membership event built with `state::build_membership_event` **carries `prev_events: vec![]` by contract** and will fail DAG validation on the node ingest path. **Chain it explicitly** (`ev.prev_events = …`) or build via `Event::new`, following the idiom already stated at `xgen-node/src/tests/space_admission_gate.rs:56` and `space_admission_mutation.rs:50`. **This has cost two runs; it is not a style note.**

---

## §4 — THE ONE STRUCTURAL BINDING

🔒 **`E1-5` (an applier gate) and `E1-6` (an accessor) are BARRED from sharing a test.**

🔑 ***`M-1`'s species, which this arc has now paid for twice: a check that lives only in the applier is a silent no-op on the answer path, and a check that lives only in the accessor is invisible to the applier.*** A unit test calling `apply_event` cannot see the accessor's gate; a test calling `is_member` cannot see `:1173`. **Each needs its own control, and each control must turn something red on its own (§5).**

---

## §5 — NEGATIVE CONTROLS. **EACH DELETED IN TURN; EACH MUST TURN SOMETHING RED.**

| control | disarm | required result |
|---|---|---|
| **V-3a** | revert `E1-5`'s `:1173` gate to a bare `contains_key` | the rejoin test (§3 `E1-8`.3) goes **RED**, and **`V-3c`'s ban test returns `AlreadyMember`** — *the wrong refusal, reproduced on purpose* |
| **V-3b** | revert `E1-6`'s `is_member` gate | `C-3`'s consequence reverses — a rejoiner is detected as an existing member. **At least one test RED** |
| **V-3c** | revert `E1-3`'s retain-and-mark to `remove` | the retention tests go **RED**, and **every §2c test passes again** — 🔑 *which is the leg's own proof that §2c's edits are gate-dependent and not cosmetic* |
| **V-3d** | revert `E1-7`'s `fanout.rs:272` filter | the `C-5` recipient test goes **RED** |
| **V-3e** | revert `E1-7`'s `resolve_operator` filters | the `D-4` test goes **RED** — *the site no accessor ruling reaches, proven reachable only by its own control* |

🛑 **`N-124b` / `N-199` DISCIPLINE ON EVERY CONTROL:**
1. **Assert the mutation changed something, on CONTENT, not on a remembered offset.** *It fired twice in Leg C and once in E-0 and was the only thing standing between a fake control and a clean-looking pass.*
2. **Restore → STAMP mtime to now → REQUIRE `Compiling xgen-core` in the log.** **An absent `Compiling` line is not efficiency; it is an unproven run.**
3. **Restore verified SHA256-identical** — *and a sha256-verified restore is still not a verified restore without (2).*

---

## §6 — VERIFICATION GATES. **CHAT RE-DRIVES ALL OF THEM FROM `HEAD` (Rule 5).**

| gate | what |
|---|---|
| **V-1** | `cargo test --workspace` — **detached**, logged to file, `XGEN_EXIT_SENTINEL=` appended, `^test result:` summed **case-sensitively**. Floor in **1623 / 0 / 62 × 56 SUITES**. **The delta is a MEASUREMENT** — re-run with `--skip` on the delivered tree, never arithmetic (`A-bis`, J-755). **Every new test confirmed BY EXACT NAME** |
| **V-2** | `left_at` occurrence count in `.rs` goes **0 → n**, and every one of `E1-7`'s twenty sites is confirmed edited **individually, by opening it** — not by a grep count that a rustfmt chain can defeat |
| **V-3a…e** | §5, each run separately, each restored per `N-199` |
| **V-4** | **the two claimed free consequences, PROVEN not asserted:** `C-3` — a rejoiner produces `new_joiner: Some(_)` at `runtime.rs:1713`; `C-5b` — `collect_sync_history` serves nothing to a departed member |
| **V-5** | **`V-3c` RUN AT LAST** — the control Leg D could not reach. Its second half is the one that matters: **`Banned`, not `AlreadyMember`** |
| **V-6** | **the §2c set is CLOSED by the compiler and the red run, and its size REPORTED** — not by this document's starting table |
| **V-7** | scope: **zero `ui/**`**, zero `fanout.rs` outside `:272`, **zero change to `build_membership_event`**. vitest and svelte-check carried by scope, stated rather than skipped |

---

## §7 — OPEN AT LOCK

✅ **NOTHING OPEN — this runbook is LOCKED.** ⚠️ **`D-154`⑥ carries ONE undischarged caveat and it is NOT E-1's to close:** retention makes an ejection a durable federated record while `membership.node_eject` is reversible, so *a reversed ejection still leaves the record saying it happened.* **Filed at `D-154`⑥ beside the `self.banned` look that is still owed.**
🔓 **Nothing on Chat's side.** §5d's boundary shape is decided; `F-E` is retired to §3's fixture rule.

---

## §8 — DoD

- [x] **Joe's §8 ruling written into `E1-4`; arm (b) struck, not deleted**
- [ ] `E1-1` … `E1-8` implemented from this file, **no improvisation** — a blocked edit is REPORTED (Rule 6, and Leg D's `D-3` is why)
- [ ] `V-1` … `V-7` re-driven by Chat from `HEAD`, none adopted on report
- [ ] `V-3a` … `V-3e` each run, each red, each restored with `Compiling` observed
- [ ] §2c's full set enumerated, edited **two-sidedly**, and its size reported
- [ ] Phase-0 v1.1 → **v1.2**, this runbook → **COMPLETED**
- [ ] `D-074` atomic commit: code + JOURNAL + CLAUDE.md + ROADMAP + task docs

📌 **"Commit pushed" is not a DoD item.**
