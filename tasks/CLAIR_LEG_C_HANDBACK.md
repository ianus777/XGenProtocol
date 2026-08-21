# Clair hand-back — `M-SPACE-ADMISSION` Leg C implementation
> **Status**: COMPLETED
> Version: 1.0
> Date: Aug 2026
> **Last updated**: 2026-08-20
> Language: EN
> Author: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> License: BSL 1.1 (converts to GPL upon project handover)

---

## §0 — SCOPE, TREE, SEAT

Implemented from `tasks/RUNBOOK_SPACE_ADMISSION_LEG_C.md` **v1.2 (LOCKED)** and no earlier version.

🛑 **TREE (`D-152` clause 1).** Session opened at **`a0ccf3a`** = `origin/main` by `git ls-remote`, clean.
`git diff --stat d8a44f6 a0ccf3a -- '*.rs'` is **EMPTY** ⇒ zero Rust drift from the runbook's stated tree, so
§2's `.rs` citations were valid at open — **and each was re-opened and verified anyway, not inherited.**

⚠️ **Every pre-existing `file:line` below was measured at `a0ccf3a` BEFORE my edits.** This leg inserts lines
into six files, so those citations have shifted in the working tree. Citations to code I *added* are to the
working tree. **Not one number in this document should be re-used without re-measuring.**

📌 **Records pending.** This document cites no `J-` number for this session, because none exists yet
(`N-198`).

🛑 **NOT PUSHED. NOT COMMITTED.** Working tree only.

---

## §1 — §7.1 DERIVATION, WRITTEN BEFORE §4 WAS READ

The runbook's §7.1 is binding and was executed first, from source, on this tree.

**Q: what happens TODAY when a NON-OWNER sends `state.space_temperature_visibility`?**

1. `dispatch_event` → `validate_event` (called `runtime.rs:1315`).
2. Step 13 (`exchange.rs:726-734`) runs iff `!skip_membership`; that set is create / join / node-eject /
   node-unban / space-migrate (+ federation-add-via-federation) ⇒ **the sibling DOES reach the check.**
3. `check_permission` (`exchange.rs:807`): the per-Room override layer first — `event_room_permission`
   (`:787`) has **no arm** for the sibling, falls to `_ => None` ⇒ **the override layer never bites.** The
   main match: `StateSpaceTemperatureVisibility` appears **ZERO times in `:807-916`** ⇒ falls to
   `_ => Ok(())` at **`:914`**.
4. ⇒ validation PASSES; the event is persisted and the sender is answered **`Accepted`**.
5. The fold reaches `apply_space_temperature_visibility` (`:785-796`), which **does** check
   `event.sender != self.owner_id` (`:786`) and returns `Err(PermissionDenied)`.
6. **All three PRODUCTION `apply_event` call sites discard it** with `let _ =`:
   `runtime.rs:867` (cfg(test) opens `:2369`) · `derive.rs:231` (`:299`) · `ai_service.rs:553` (`:668`).
   `exchange.rs:1279` / `:2319` are **inside** `#[cfg(test)]` (opens `:1096`) ⇒ not production.

**A: ACCEPTED · PERSISTED · ANSWERED `Accepted` · then SILENTLY DROPPED by the fold, error thrown away.**

✅ **This matches `M-1` exactly. No finding against the runbook on §7.1.** `F-3`'s correction to the call-site
set is confirmed independently on this tree.

**Five things the derivation added that `M-1` does not state:**

1. **The override layer is a second gate and it is inert here.** A new `StateSpaceAdmission` must **not** be
   added to `event_room_permission`: admission is Space-level with no Room dimension, and an arm there would
   let a per-Room `Effect::Allow` grant a **non-owner** the right to change a Space-wide admission policy,
   silently outranking `can_change_admission`. **Deliberate non-change.**
2. **There is exactly ONE permission table.** `check_permission_pub` (`:317`) is a genuine three-line
   delegating re-export; the `_ => Ok(())` at `:393` belongs to `check_ai_operator_targets`, a different
   function. Nothing to mirror, no divergence risk.
3. **The apply-layer `let _ =` silence must NOT be "fixed", and the codebase says why.**
   `runtime.rs:1505-1522` (MP-F6 banned-join) names this species — *"the reply lied"* — and records the ruled
   resolution: surface the reject at the dispatch/validation layer, **leave the apply-layer silence, it is
   load-bearing for replay tolerance** (*"a replayed event resolution will drop must not crash replay; audit
   A4"*). §4.4's both-places design is the shape this codebase has already ruled correct once.
4. `skip_membership` had to be checked, or step 13 never runs for the new variant. It does not cover it. ✅
5. `M-4`/`F-4` corroborated from a second independent source: `runtime.rs:1509-1522`'s own comment states
   `to_wire_code` returns `None` for `PermissionDenied` ⇒ 4000-unmapped.

---

## §2 — WHAT SHIPPED

**7 tracked files changed (`+605 / −6`), 1 code file added (241 lines). Zero `ui/**`. Zero `xgen-client`.**

🛑 **The `+605 / −6` is `git diff --numstat` and it does NOT see untracked files** — the new test file and
this hand-back are invisible to it (`D-152` clause 2; J-758's own size-claim correction). **Code total is
`605 + 241 = 846` added lines across 8 files.** A second untracked file, `tasks/CLAIR_LEG_C_HANDBACK.md`,
is this document.

| file | what |
|---|---|
| `xgen-common/src/wire.rs` | `StateSpaceAdmission` variant · `as_str` · `from_str` · **`known_variants()`** · `StateSpaceAdmissionContent` · **§4.6 count test** |
| `xgen-core/src/wire/types.rs` | the hand-maintained `pub use` re-export (Leg B's `F-2`: not a glob) |
| `xgen-core/src/resolution/state_key.rs` | the arm on `StateSpaceUpdate`'s shape + test 5 |
| `xgen-core/src/space/membership.rs` | `can_change_admission` **in the permission table** |
| `xgen-core/src/message/exchange.rs` | `ExchangeError::AdmissionImmutable` · `to_wire_code` → `3049` · **the `check_permission` arm, DM check FIRST** · tests 2 + 4 |
| `xgen-core/src/space/state.rs` | `SpaceError::DmAdmissionNotAllowed` · `apply_space_admission` · dispatch arm · `build_space_create_event_with_admission` · tests 1 + 3 + 6 |
| `xgen-node/src/tests/mod.rs` | module registration |
| `xgen-node/src/tests/space_admission_mutation.rs` **(new)** | **test 7**, end-to-end through `dispatch_event` |

🔑 **The applier's role check is a ROLE PREDICATE, not `event.sender != self.owner_id`.** ch3 §3.7.14.4
requires this explicitly, and both nearest siblings (`apply_space_temperature_visibility`,
`apply_space_pacing`) use the identity-equality form. Copying them would have produced the third divergent
site the Phase-0's Rider 2 exists to prevent. The reason is written at the site.

🔑 **`known_variants()` N = 60.** Measured, not counted by hand: 59 entries before this leg, which equals the
enum's declared variants minus `Unknown(String)`.

---

## §3 — GATES

Every figure below was produced by a detached run with its own exit sentinel; the launcher's exit code was
**never** read as cargo's.

| gate | result |
|---|---|
| **V-0 baseline** | **1608 / 0 / 62 × 56 SUITES**, `CARGO_EXIT=0`, sentinel present, summed programmatically. 🔑 **Measured on this tree, not carried from J-758** — it confirms the carried figure rather than assuming it |
| **V-1** | **1616 / 0 / 62** — ***exactly the predicted +8***. `CARGO_EXIT=0`; `^test result: FAILED` / `error[` / `panicked` / `^warning` **all zero, case-sensitive**; `Compiling xgen-core/common/node` present ⇒ **not a cached pass**. All eight confirmed **BY EXACT NAME** |
| **V-2** | **56 SUITES** — unchanged, structural |
| **V-3** | ✅ **tests 2, 4 AND 7 all FAILED.** See §4 — this is the leg's most valuable measurement |
| **V-3b** | ✅ **test 4 FAILED and NOTHING ELSE DID** — including test 3, whose applier DM assertion still passed, which is precisely what makes V-3b a distinct control from V-3 |
| **V-4** | ✅ **both halves.** The count test FAILED (`left: 59, right: 60`); all **three** iterating sweeps PASSED — demonstrating the omission is invisible to them |
| **V-5** | 7 tracked `+605/−6` (`git diff --numstat`) **+ 2 untracked (`git ls-files --others`)**: the new test file (241 lines) and this hand-back — **both instruments, `D-152` clause 2** |
| **V-6** | all 7 tracked files `i/lf`, **`bareLF=0` on every one**; the new file is **pure LF (CR=0)**, matching its sibling `space_admission_third_party_join.rs` (`w/lf`) |
| **V-7** | ✅ **no interpretation of the value.** Every added production line was read: zero `match` on `"open"`/`"invite"`, zero `if ==` on the value, zero allow-list. The only comparisons are on **role** and on **`dm_constraints_active`** |

📌 **Leg A-bis's two witnesses green** (§6 trigger ③). 📌 **`xgid-slot-gate` PASS 74 (65/5/3/1), unchanged**
— ⚠️ run with `-AllowDirty` on an uncommitted tree, and **the gate's own banner says not to quote its
numbers**; the clean-tree run is Chat's at commit time.

🔒 **Floors carried BY SCOPE, stated rather than skipped:** vitest **172 / 172 × 9 FILES** · svelte-check
**0 / 34 / 15** — zero `ui/**` touched. 🛑 Catalogue **UNMEASURED**.

---

## §4 — V-3 IS THE MEASUREMENT WORTH READING

With the `check_permission` arm deleted, **test 7 failed with:**

> `a non-owner's admission change must be REJECTED to its sender. Got Accepted { new_joiner: None, additional_persisted: [] }.`

🔑 ***That is `M-1` reproduced live, not argued.*** The applier still refused; its error was discarded; the
sender was told `Accepted`. **And test 3 — the applier test — passed throughout.**

⇒ **`F-7` is vindicated concretely: six unit tests would have gone green over exactly this defect.**

And V-3b's failure is equally precise: `unwrap_err()` on an `Ok(())` — with only the DM branch removed, **the
DM's OWNER passes the gate.** That is `F-1`, live.

---

## §5 — FINDINGS. **REPORTED, NOT ABSORBED.**

### 🛑 F-A — my own cold-read `F-6`'s citation does not support its claim, and I nearly shipped it into code

§4.4(b) and Phase-0 §15.3 both say *"the FEDERATED path DOES pass through `check_permission` —
`runtime.rs:1426` calls `check_permission_pub`"*.

**Measured:** that call sits inside `if matches!(event.event_type, StateAiOperatorDelegate |
StateAiOperatorRevoke)` (`runtime.rs:1415-1418`). **`state.space_admission` never reaches it.**

✅ **The claim is nevertheless TRUE, by a different route:** `dispatch_event` calls `validate_event`
unconditionally (`:1315`), and step 13's `skip_membership` set does not cover the new variant.

🛑 **This is `F-4`'s species repeating — right conclusion, wrong evidence — and I committed it once before
catching it:** I wrote the `runtime.rs:1426` citation into `apply_space_admission`'s doc comment, then
corrected it. **That is exactly the J-758 shape: a bad runbook line propagated into code by a faithful
implementer, where it outlives the document that carried it.** The code now states the mechanism I verified
and carries an explicit note that `:1426` is *not* the general federated gate. **The runbook and the Phase-0
still carry the original claim.**

### 🛑 F-B — §4.7's spec row is ALREADY IN THE TREE

It landed at **`a0ccf3a`**, the J-759 records commit: `docs/xgen_ch3_specification.md:2193` carries the
`3049 admission_immutable` row, plus the `4000`-for-plain-refusal sentence and the `3047`/`3048` reserved
line. **The §8 DoD item is satisfied by an existing commit, not by this leg's work.** Not a defect — the
runbook says Chat writes it — but §4.7's *"ships in the same `D-074` commit"* is no longer accurate, and I
did **not** touch ch3.

### ⚠️ F-C — §4.8's seven do NOT cover the applier's DM branch. I bundled rather than added an eighth

Test 3 is specified as *"non-owner is also refused by the applier directly"*, which exercises only the
**role** branch of §4.4(b). Nothing in the seven constructs `SpaceError::DmAdmissionNotAllowed`, so as
specified the leg would ship an error variant **no test ever produces**, and V-3b (which deletes only
`check_permission`'s DM branch) cannot detect its removal.

**What I did:** wrote test 3 with **both** of §4.4(b)'s branches in one test function. **Count unchanged at
eight; cargo lands on the predicted 1616.** Adding an eighth test would have moved it to 1617.

📌 **Reported as a judgement.** It covers §4.4(b)1, which §4.8 under-specified — but it is a reading, and
it is reversible.

### ⚠️ F-D — §4.6's count assertion can be satisfied by a duplicate

`assert_eq!(known_variants().len(), 60)` passes if a future author adds a variant **and** accidentally lists
an existing one twice: the count is right and the new variant is still missing from the sweeps.

**Not built** — §4.6 specifies the count form precisely and I did not widen it. A `HashSet` dedup assertion
in the same test would close it at no cost. **Chat's call.**

### ⚠️ F-E — `build_membership_event` is unusable in a node-harness fixture, and it cost me a test run

`state::build_membership_event` emits **`prev_events: vec![]`**. That is fine for the `state.rs` unit tests
(they call `apply_event` directly and never touch the DAG) and is a **structural violation on the node
path**: `ingest_event` does not apply an unchained non-root event, so my invite and join silently failed to
land and test 7 failed at its **precondition** (`member_role → None`) rather than at its subject.

✅ **The failure was loud and named the right line** — which is the argument for asserting preconditions
rather than assuming them. Fixed by chaining each setup event on freshly-read `dag_tips`. **Worth knowing
before the next node fixture is written.**

### ⚠️ F-F — three instruments gave three counts for `known_variants()`, and only printing the list reconciled them

Hand count **59** · comma-split **61** · enum-declaration grep **60**. The 61 was two extraction artifacts
(`]` and `}`); the 60 counted `Unknown(String)`, which `known_variants()` deliberately omits. **The truth is
59, and my hand count was right — but I was right not to trust it, because two instruments disagreed.**
`N-197`'s species, caught before the number entered the code.

### ⚠️ F-G — V-3 as written leaves test 7 unobserved

`cargo test --workspace` **halts after the first failing suite.** Under V-3, `xgen-core` fails first, so
`xgen-node` never runs and **test 7 — the one V-3's own note calls "the one that matters" — is never
executed.** Following the gate literally would have recorded a two-of-three pass as if it were three.

**I ran test 7 separately under the still-applied control.** V-3 should say `--no-fail-fast`, or name the
targeted second run.

### 📌 F-H — the Phase-0's §15.3 line citations are from an older tree

`wire.rs:168` / `:429` / `:736`, `state.rs:752-768` / `:659-666`, `state_key.rs:44` do not resolve on this
tree; the runbook §2's `d8a44f6` set does. **Both are correct for their own trees** — `D-152` clause 1
working as intended. **I used §2's set and verified every entry.** Noted only so the Phase-0 is not read as
current.

---

## §6 — WHAT I DID **NOT** VERIFY

🛑 **Nothing was exercised against a running Node, a wire, or a second identity.** Test 7 is in-process, one
node, `peer_node_id: None` throughout — it proves the **local dispatch** path returns `Rejected` to its
sender. **It proves nothing about federation**, and the federated coverage in §5 F-A is a **source trace**,
not an observation.

🛑 **The REPLAY path — the entire justification for §4.4(b) — is not tested.** No replayed event was
constructed. The applier's refusal is exercised by direct `apply_event` calls, which is not the same thing.

🛑 **`3049` was never observed on a wire.** It is asserted at `RejectInfo::from_exchange`, one layer short of
the `Error` frame.

🛑 **I did not re-run vitest, svelte-check or the catalogue.** They are carried by scope (zero `ui/**`) and
must not be quoted as measurements from this session.

🛑 **`-AllowDirty` slot-gate numbers are not quotable**, by the gate's own banner.

---

## §7 — DoD AGAINST §8

- [x] Variant, `as_str`, `from_str`, `known_variants()`, and §4.6's count assertion
- [x] Content struct on **S-5**'s pattern
- [x] `state_key_for_event` arm, §6.3's key
- [x] `can_change_admission` in the **table**, Owner-only, doc-commented
- [x] Enforcement in **BOTH** `check_permission` and the applier — **including the DM branch in
      `check_permission`**
- [x] DM refusal carrying `3049 admission_immutable`; its own `SpaceError` variant in the applier
- [x] §4.7's spec row — ⚠️ **already present at `a0ccf3a`; see `F-B`. Not my edit.**
- [x] `build_space_create_event_with_admission`; **the original builder untouched** (asserted in test 6)
- [x] Seven tests including the end-to-end one; **V-1 1616 / 0 / 62 × 56 SUITES** — 🛑 **measured by me;
      Chat re-drives under Rule 5**
- [x] V-3, V-3b and V-4 all run, all controls behaved as specified, **all reverted and SHA256-verified
      against their pre-control bytes**
- [ ] **Records + commit — Chat's and Joe's.** I did not write JOURNAL / `CLAUDE.md` / ROADMAP, did not set
      `Status: COMPLETED` on the runbook, did not commit and did not push.

---

## §8 — VERDICT

## ✅ **IMPLEMENTED. EIGHT FINDINGS, NONE BLOCKING.**

The runbook's build was sound and every design claim in §4 survived implementation. **`F-A` is the one worth
Chat's attention** — a citation I had myself measured as wrong, which I then wrote into a code comment before
catching it, and which still stands in two canonical records. **`F-C` and `F-G` are the two places I departed
from the letter of the document**, both reported above rather than absorbed.
