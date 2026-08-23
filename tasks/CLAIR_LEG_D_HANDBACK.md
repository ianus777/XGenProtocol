# Leg D hand-back — Clair

> **Status**: ACTIVE
> Version: 1.0
> Date: Aug 2026
> **Last updated**: 2026-08-22
> Language: EN
> Author: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> License: BSL 1.1 (converts to GPL upon project handover)

Implemented from `tasks/RUNBOOK_SPACE_ADMISSION_LEG_D.md` **v1.1 LOCKED**.
Opened at `7abd341` = `origin/main` by `git ls-remote origin refs/heads/main`, tree clean.

🛑 **Not committed, not pushed.** Records, `Status: COMPLETED`, the commit and the push are Chat's and Joe's.

**Two deviations reported, not absorbed (Rule 6). One is blocking.**

---

## §1 — 🛑 BLOCKING: **D-3 IS NOT IMPLEMENTABLE IN THIS LEG**

`D-3` is specified as *"`state.rs:1112` gates on `left_at.is_none()`"*. **`left_at` does not exist.**

### Measured, both directions

- `grep -rn "left_at" --include=*.rs` over the whole tree, excluding `.claude/` and `target/` — **ZERO occurrences.** The token appears only in design documents (`CLAUDE.md`, `DECISIONS.md`, `docs/ROADMAP.md`, `JOURNAL.md`, five `tasks/` files).
- `SpaceMember` (`state.rs:86-95`) has exactly four fields: `identity_id`, `role`, `joined_at`, `invited_by`. No departure marker under any other name — `departed`, `leave_at`, `former_member`, `is_present` all return zero.
- **All four departure appliers REMOVE the member outright**, so no retained record exists to carry a marker: `apply_leave` `:1203` · `apply_kick` `:1230` · `apply_ban` `:1250` · `apply_node_eject` `:1275`.

### Why I did not add it

**Adding `left_at` IS `(g)`, and the Phase-0 excludes `(g)` from this leg in its own words.** §7.1: *"It does not touch `(g)` / `left_at` itself — that is Leg E. §5 is the gate edit only, written so Leg E has a correct site to build on."*

The two cannot both be satisfied: **§5 specifies the edit in terms of a field §7.1 forbids this leg to create.** §5's own reasoning is conditional and says so — *"**Under `(g)`** it refuses the rejoin `Q-2`(a) promised"*.

`(g)`'s cost, measured so the option can be priced rather than guessed: the struct field + **18 `SpaceMember` construction sites** + the four appliers + `E-0`'s **20 `D-3` census sites**, which `J-761` records as needing individual edits under any ruling and whose meaning all changes at once (*"no existing reader's meaning is preserved"*).

### 🔑 The decisive evidence is that D-3's own control cannot be run

§4's **`V-3c`** requires that deleting D-3's guard turns red *"the rejoin test **and** the retained-ban test"*.

**Neither test can exist today.** A leaver is removed from `members`, so:

- there is no retained member for a rejoin to be refused by — `contains_key` is already `false` and the rejoin already succeeds;
- there is no retained **banned** member, so the second half — the one J-762 calls *"the one that matters"*, where a retained banned member is refused `AlreadyMember` instead of `Banned` — describes a state the code cannot reach.

⇒ **D-3 would be a no-op today whose negative control is unrunnable.** Written now it is an unfed branch (`D-065` / `N-091`): a guard nothing can exercise, shipped green, in the leg whose stated point is that *a gate whose removal leaves the suite green was never tested*.

### Dispositions — none taken

| | option | cost |
|---|---|---|
| **(a)** | **Move D-3 to Leg E**, where `(g)` lands. `V-3c` becomes runnable in the leg that creates the state it tests. | Nothing lost — D-3 changes no behaviour today. Leg D closes with three edits, not four. |
| **(b)** | **Widen Leg D to include `(g)`.** | Contradicts Phase-0 §7.1; pulls a 40-site semantic change into a gate leg and makes the cargo delta unattributable between them. |
| **(c)** | Ship a named predicate (`is_present()`) returning `true` today for Leg E to fill in. | An unfed branch with no possible control; and `D-154`'s rejoin **write** defect (`insert` REPLACES, `state.rs:1122`) still lands in Leg E regardless. |

**I lean (a)**, on `V-3c` rather than on tidiness: the leg's stated standard is a negative control per gate, and D-3 is the one gate that cannot have one until `(g)` exists.

---

## §2 — ⚠️ SECOND DEVIATION: A LEG D DoD ITEM THAT IS IN NO LEG D DOCUMENT

**The full suite went RED on a test neither the runbook nor the Phase-0 mentions**, and it was supposed to.

```
tests::space_admission_third_party_join::tests::third_party_registered_identity_joins_a_dm_it_is_not_party_to
  an uninvited registered third party's join of a DM is ACCEPTED today;
  got Rejected(RejectInfo { code: 3047, name: "admission_required", ... })
```

**That red is the gate working.** `J-755` wrote the item into Leg D's row in advance, and `docs/ROADMAP.md:396` carries it verbatim:

> *"LEG D INHERITS AN INVERSION DoD ITEM, written in the edit that closed A-bis (`N-109`): after the gate ships, **a GREEN run of the un-edited DM test is a FAILURE OF THE GATE, not a pass**, and **the open-Space companion is NOT touched**."*

and `RUNBOOK_SPACE_ADMISSION_LEG_A_BIS.md:203`:

> *"after Leg D the DM test is edited into its opposite … The DM test is the perishable witness; the companion is the permanent lock."*

🛑 **`grep -in "inversion|third_party|A-bis|perishable|un-edited"` over `RUNBOOK_SPACE_ADMISSION_LEG_D.md` and `M_SPACE_ADMISSION_LEGD_PHASE0.md` returns ZERO.** The obligation was recorded in the leg that created it and did not reach the leg that owes it.

📌 **The species is the arc's own: a requirement written in one document's BEHAVIOUR list and another document's FILE list, never reconciled.** It differs from the usual instance in one way that matters — here the answer *was* written down, correctly, in advance, by the leg that could foresee it. What failed is the hand-off, not the foresight. The instrument that caught it was the full-suite run, not any reading of Leg D's own documents.

### What I did, and why I acted rather than only reporting

**I inverted the DM test** and did **not** touch the companion. Three reasons:

1. The governing instruction is explicit and leaves no choice of form — *"edited into its opposite"*.
2. The alternative is handing back a red suite over a red that the record predicted and welcomed.
3. The runbook's silence is an **omission**, not a prohibition — nothing in Leg D's scope forbids it.

**The four assertions are reversed in place, each carrying the line it replaced** (`D-131`: the hole is the point, and a deleted witness records nothing). Assertion 4 is **unchanged and now means the opposite** — before the gate it read as an indictment (*the counterpart is still absent while a stranger was admitted*), after it as the ordinary state of a DM nobody has accepted. The module doc records what the file asserted before, why it had to be taken before the gate, and that the un-edited test going green would have been a failure.

✅ **The companion `third_party_registered_identity_joins_an_open_space` is UNTOUCHED and green** — through the gate and through both negative controls. That is what makes the DM inversion readable as *the gate closed a DM* rather than *the gate closed everything*.

### 🔓 One thing I did NOT do, because it is not mine

**The inverted test's NAME is now a false statement.** `third_party_registered_identity_joins_a_dm_it_is_not_party_to` asserts, as of this leg, that she does **not** join — and this codebase names tests by OUTCOME (`..._is_rejected_to_the_sender_end_to_end`, `..._is_rejected_3047_to_the_sender`). That is the `N-109` species in a symbol.

I left it, because renaming is a naming decision (Joe's, `D-123`) and the current name is cited by `docs/ROADMAP.md`, `JOURNAL.md` and the A-bis runbook. 📌 `J-755`'s baseline used `--skip space_admission_third_party_join`, the **module** path, so a rename of the function would not invalidate that measurement. Routed, not absorbed.

---

## §3 — WHAT SHIPPED

| | edit | files |
|---|---|---|
| **D-1** ✅ | the admission gate at `runtime.rs:1580`'s block, **before** the expiry check, `LocallySubmitted` only, Space-level only | `xgen-core/src/node/runtime.rs` |
| **D-2** ✅ | the named three-state parse + the 64-byte char-boundary cap; **the false doc comment corrected, not deleted** (`D-131`) | `xgen-core/src/space/state.rs`, `xgen-common/src/wire.rs`, `xgen-core/src/wire/types.rs` |
| **D-3** 🛑 | **BLOCKED — §1** | — |
| **D-4** ✅ | `3047 admission_required` minted at the gate; `ch3` §3.6.10.10 gains **its row AND `3046`'s** (`C-8`) | `xgen-core/src/node/runtime.rs`, `docs/xgen_ch3_specification.md` |
| **inversion** ⚠️ | the A-bis DM witness edited into its opposite — §2 | `xgen-node/src/tests/space_admission_third_party_join.rs` |

**Size — stated honestly, because `git diff --numstat` cannot see untracked files:** **577/−56 across 7 tracked files, PLUS 324 lines of new untracked source** (`xgen-node/src/tests/space_admission_gate.rs`), plus this hand-back and `tasks/CLAIR_LEG_D_DERIVATION.md`. **Zero `ui/**`, zero `xgen-client`** — verified by listing tracked-diff and untracked names together, not by the diff alone.

### D-1 — the gate

The predicate is **`space.admission != ADMISSION_OPEN`**, not `== ADMISSION_INVITE`, and the asymmetry is deliberate rather than stylistic. `admission` is an open enum carried as a string, so the set of values this build does not recognise is unbounded and **includes every value a future version may add**. Testing for `open` means the only value that opens the door is the one that says so — Malformed, `invite`, and any future value all gate. That is `D-149`'s fail-closed reading at the gate, and it matches the expiry check's own `unwrap_or(true)` twenty lines below.

Federation is not re-adjudicated: the enclosing block is already `origin == LocallySubmitted`, so a `ReceivedViaFederation` join is **skipped, not rejected** — inherited from `:1567-1579`, not re-argued. Room joins are excluded by the enclosing `room_id.is_empty()`.

### D-2 — the parse

Three states, named in code: **Absent** (`content.get()` is `None`) ⇒ `DEFAULT_ADMISSION` · **Valid** (a JSON string within the cap) ⇒ stored as-is · **Malformed** (present non-string, or a string over the cap) ⇒ stored as the value's raw JSON text, truncated on a `char` boundary.

📌 **`content.get("admission")` replaces `content["admission"]`**, and that is the mechanical heart of `F-3`: indexing returns `Value::Null` for **both** an absent key and an explicit `null`, which is the collapse the old comment forbade while the line beneath it performed. ⚠️ **Consequence worth naming rather than letting it read as incidental: an explicit `{"admission": null}` is now Malformed and therefore gates.** A behaviour change, in the fail-closed direction.

📌 **A Malformed value can never be mistaken for `open`, by construction** — a JSON value's serialised text is a number, `true`/`false`/`null`, a quoted string, an array or an object, and no prefix of any of those is the bare token `open`. So truncation cannot accidentally manufacture a permissive value.

`ADMISSION_MAX_LEN = 64` lives in `xgen-common/src/wire.rs` beside the rest of the family and is re-exported through `xgen-core/src/wire/types.rs` — that list is **hand-maintained, not a glob** (Leg B `F-2`), so the re-export is not automatic.

### D-4 — ⚠️ a minor citation deviation

§3's table gives D-4's files as **`exchange.rs`, `ch3`**. **There is nothing to add to `exchange.rs`.** `to_wire_code` maps `ExchangeError` variants, and `3047`'s siblings — `3044`, `3045`, `3030` — are **not** variants: they are emitted directly at the dispatch site through `RejectInfo::coded`, which takes the code and name as arguments. `3047` follows them exactly.

Adding an `ExchangeError::AdmissionRequired` to satisfy the table would mint a variant nothing constructs — an unfed branch, in the leg that refuses those. **Reported rather than absorbed; the conclusion (3047 is minted and live) is unaffected.**

### `C-8` — sharper than filed, and it moved a second sentence

`3046` is absent from the **table**, but the paragraph beneath it did name it (*"3046 is assigned outside this table"*). So the stated risk was partly mitigated — and **adding the row makes that sentence false**, so both had to move in one edit. The closing paragraph now records why the correction exists and states the rule it encodes: *a wire code is allocated in this table in the same change that first emits it, and a code named only in prose is a code the next reader will not see.* `3046`'s meaning is not re-litigated. `3048` remains reserved.

📌 `docs/xgen_ch3_specification.md` header bumped **v0.58 → v0.59**, `Last updated` **2026-08-22**, per the standing document convention (*"this header MUST be updated on every file edit"*). Byte count unchanged — a same-length digit swap — so it was **read back** rather than inferred from size.

---

## §4 — VERIFICATION

### V-3a — delete D-1's arm ✅ **REPRODUCED**

```
an uninvited join into an invite-only Space must be REJECTED to its sender.
Got Accepted { new_joiner: Some(IdentityXgid(...)), additional_persisted: [] }
```

**That is the pre-Leg-D behaviour, live** — carol admitted to an invite-only Space holding no invite. `leg_d_federation_join_into_invite_only_space_skips_the_admission_gate` and `leg_d_malformed_admission_gates_like_invite` also went red. 🔑 **Both open-Space controls stayed GREEN**, which is what shows the controls are not themselves measuring the gate.

### V-3b — delete D-2's Malformed branch ✅ **REPRODUCED, AND IT FAILED OPEN**

```
assertion `left != right` failed: a present non-string (5) must NOT collapse to
`open` — that is the permissive collapse `F-3` found
  left: "open"
 right: "open"
```

§4 required this be shown to fail **closed, not open** — that removing the branch restores the permissive collapse. The message says `"open"` in both positions. The composition test in `xgen-core` also went red at its precondition, showing the malformed value would reach the gate as `open` and therefore not gate at all.

### V-3c — 🛑 **NOT RUN. UNRUNNABLE.** §1.

### Control hygiene

Both controls used **file backups, never `git checkout`** (the work is uncommitted). Each asserted **on content** — a marker string present *and* the production construct absent — never on a remembered offset. Both restored and verified **SHA256-identical**:

- `runtime.rs` `d17522ef863cb8e672024ef845116052ffcaac9fa578b2e95602166fe724e398`
- `state.rs` `861dc857859884e362063d087696427c20dac21f96ee885078fe1299c6f33317`

📌 **The content guard fired once and was right to.** My first V-3a assertion grepped for `admission_required`, which also matches the test's own `assert_eq!(info.name, ...)` — so it reported *"gate still present"* on a correct mutation and restored the file. Re-cut against a production-only string. **A guard that produces a false abort is doing its job; a guard that cannot tell production from test is not a guard.**

### V-4 — the federation skip ✅ **PROVEN, NOT ASSERTED**

`leg_d_federation_join_into_invite_only_space_skips_the_admission_gate`: the same uninvited join is **Rejected 3047** under `LocallySubmitted` and **Accepted, with carol actually a member**, under `ReceivedViaFederation`.

The local arm runs **first**, and the ordering is load-bearing: the federation arm admits carol, so a federation-first ordering would leave the local arm hitting `AlreadyMember` — *a refusal that looks green and is the wrong refusal.*

### V-5 — floors ✅

**cargo `1616 → 1623 / 0 / 62 × 56 SUITES`.** Detached, own `XGEN_EXIT_SENTINEL=0`, final `test result:` line present, **summed programmatically** over `^test result:`; `FAILED` **case-sensitive 0**, `^error[` 0, `panicked` 0.

- **The `1616` baseline was MEASURED on this tree at open, not carried** — same command, clean tree, `1616 / 0 / 62 × 56`. ⇒ **+7 is a measurement, not arithmetic against a carried figure.**
- **`56 SUITES` unchanged** — the new node test file is a `pub mod` inside the existing `tests` module, so no new binary; a changed suite count would have been a §6 finding.
- ⚠️ **`Compiling xgen-core` is ABSENT from the final log, and the usual not-cached check therefore does not apply here.** The crate had been compiled from the current source by the immediately preceding targeted run. **Two independent facts settle it instead:** three `leg_d_*` tests that exist only in this working tree **executed inside `xgen-core`'s own lib suite**, and the test binary's mtime (`22:54:30`) **post-dates every source file touched** (latest edit `22:54:05`). Stated as a limit of the stock check rather than waved through.
- 📌 **`cargo test --workspace` is the canonical command, and `--all-targets` is NOT the same measurement:** it returns `1616 / 0 / 60 × 49` on the same tree — identical test count, different suite and ignored counts, because it swaps doctest/target selection. Measured, not assumed.

**vitest `172 / 172 × 9 FILES` · svelte-check `0 / 34 / 15` — CARRIED BY SCOPE, not re-run.** Stated rather than skipped: **zero `ui/**`, zero `xgen-client`**, verified by listing tracked-diff names *and* untracked names together (`git diff --name-only` alone cannot see a new file). **Catalogue UNMEASURED.**

### V-6 — every new test, BY EXACT NAME ✅

📌 **A bare `grep admission` finds five of the seven** — two `leg_d_*` names do not contain the word. All seven confirmed present and `ok` in the final log:

**`xgen-core` — `space::state::tests`**
1. `from_space_create_present_non_string_admission_is_malformed_and_fails_closed`
2. `from_space_create_over_cap_admission_is_truncated_on_a_char_boundary`

**`xgen-core` — `node::runtime::persistence_amendment_commit_2a_tests`**
3. `leg_d_federation_join_into_invite_only_space_skips_the_admission_gate`
4. `leg_d_open_space_still_admits_an_uninvited_join`
5. `leg_d_malformed_admission_gates_like_invite`

**`xgen-node` — `tests::space_admission_gate::tests`**
6. `uninvited_join_into_an_invite_only_space_is_rejected_3047_to_the_sender`
7. `the_same_uninvited_join_into_an_open_space_is_admitted`

Plus one non-test helper, `setup_invite_only_space` (`runtime.rs`). **`7` matches the `1616 → 1623` delta exactly.**

### EOL ✅

`git ls-files --eol` on every touched file: **all `i/lf`**, which is what ships. `mod.rs` is `w/crlf`; the two `space_admission_*.rs` siblings are `w/lf`, and the new `space_admission_gate.rs` matches them. `ch3` is `i/lf w/lf` and stayed LF — `CR 0`, no BOM, re-measured after the edit.

---

## §5 — §1 OF THE RUNBOOK: THE DERIVATION

Executed **from source, before §2 or §4 was opened**, and written to `tasks/CLAIR_LEG_D_DERIVATION.md` before the runbook's account was read.

**The derivation and §2 agree on every point.** Every §2 citation was opened before being accepted (`D-153`); none moved. The only refinement is `C-8`'s scope — `3046` has no ROW but *is* named in the paragraph beneath the table — folded into the ch3 edit rather than reported as a disagreement. Full reconciliation table in that file.

**This is the first leg in four where §1 did not find a defect in §2.** Stated plainly rather than skipped: §2 was accurate.

---

## §6 — WHERE THIS IS MOST LIKELY WRONG

Written before anyone else reads it, in the order I would attack it.

1. **§2's inversion is an edit outside my locked scope.** I acted on another locked document's instruction because the runbook was silent and the alternative was a red suite. **If that judgement is wrong, this is where.** The *form* — reverse in place, keep the replaced lines, do not touch the companion — follows the record; the *decision to act at all* is mine.
2. **`ADMISSION_MAX_LEN = 64` and the raw-JSON-text storage are Chat's design, implemented literally.** I did not test what a Malformed value looks like to an operator or a client — only that it is bounded, deterministic, and never equals `open`.
3. **The explicit-`null` behaviour change** (§3) is a consequence of `content.get()`, not an instruction. It is in the fail-closed direction and I named it, but nobody ruled on it.
4. **`apply_space_admission` (`state.rs:854`) was NOT touched, and it treats a non-string differently from the create path** — it returns `MissingField` and refuses the event outright, where the create path stores raw text. Defensible (the mutation path has a sender to refuse to; a create event does not), but **the mutation path also applies NO cap**, so an over-long admission string can still be stored unbounded through `state.space_admission`. **Out of the runbook's stated scope (`state.rs:344-351`); filed, not fixed.**
5. **Nothing ran against a running Node, a wire, or a second identity.** `3047` has never been observed on a wire. The federation arm is `dispatch_event(..., ReceivedViaFederation, None)` in-process — `peer_node_id` is `None` throughout, so **F-3 is never evaluated and this proves nothing about federation** beyond the origin gate itself.
6. **The `V-3a` control deleted the `if` block and left its ~40-line comment standing.** The comment is not compiled, so the mutation is behaviourally complete — but a reader of that control's diff would see less removed than the gate.

---

## §7 — DoD (runbook §5)

- [x] §1 executed **from source, before §4 was opened**, derivation written to disk
- [ ] **D-1 · D-2 · D-4 shipped. D-3 BLOCKED — §1.**
- [x] **V-3a and V-3b reproduced live**, each restored and `sha256`-verified
- [ ] **V-3c not run — unrunnable; §1**
- [x] V-4 · V-5 · V-6
- [x] Deviations reported, not absorbed (Rule 6) — **two, §1 and §2**
- [x] Hand-back at `tasks/CLAIR_LEG_D_HANDBACK.md`

📌 Phase-0 §8's items map the same way; its `C-8` and federation-skip items are discharged, its `state.rs:1112` item is not.

---

## §8 — OPEN, FOR CHAT AND JOE

1. 🛑 **D-3's disposition** — (a) / (b) / (c) in §1. I lean **(a)**. **This is the only thing standing between the leg and its close.**
2. ⚠️ **Whether §2's inversion was mine to make**, and whether the inverted test should be **renamed** (its name now asserts the opposite of what it tests).
3. 📌 The `apply_space_admission` cap asymmetry (§6.4).
4. 📌 The `exchange.rs` citation in the runbook's D-4 row (§3).
5. 📌 `3048` remains reserved; `C-8`'s registry-allocation rule is now stated in `ch3` and could be promoted if it is wanted as a standing rule rather than a local note.

**Nothing was committed. Nothing was pushed. No record file, no `Status: COMPLETED`, no ROADMAP or JOURNAL edit.**

---

## §9 — ONE LATE SWEEP, RECORDED BECAUSE OF WHERE IT WAS

After §2's inversion and after writing §6, a stale claim survived **in the file I had just inverted**: the DM test's own function doc still read *"Records today's behaviour: `Accepted`, `Role::Member`, `invited_by: None`"*. Every assertion beneath it now says the opposite.

📌 **That is the `N-109` species inside the edit that flagged the `N-109` species** — §2 routes the function's *name* to Joe as a claim that outlived its state, and the *doc comment* two lines above it was the same defect, mine, unswept. Corrected; the module doc, the function doc, and all four assertions now agree, and the function name is the only thing left that does not (routed, §2).

The correction is comment-only — verified from the diff, where every changed line in that hunk is a `///` line, rather than asserted. The delivered-tree cargo run below was taken **after** it.
