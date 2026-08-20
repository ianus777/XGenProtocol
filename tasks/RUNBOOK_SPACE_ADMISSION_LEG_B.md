# M-SPACE-ADMISSION Leg B Runbook — the field and the create parse: one String, three constructors, and nothing that reads it yet
> **Status**: PENDING  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-18  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS IS, AND WHAT IS LOCKED

**Leg B of `M-SPACE-ADMISSION` — who may join a Space, and how a leaver comes back.** The design is `tasks/M_SPACE_ADMISSION_PHASE0.md` **§15** (v2.2, locked at J-756, corrected at J-757). This runbook implements **§15.1 and §15.2 only**.

| item | state |
|---|---|
| the design | 🔒 **LOCKED** — Phase-0 **v2.2**, §15. Leg B implements §15.1 + §15.2 |
| the field's name | 🔒 `admission` (Q1, Joe, 2026-08-16) |
| the DM pin | 🔒 store `invite` at DM creation (Q4 **(b)**, Joe, 2026-08-16) |
| absent ⇒ `open` | 🔒 `L-E` + §7 |
| 🔒 **THE LOCK** | **LOCKED by Joe 2026-08-18.** The locked content is **v1.1**; **v1.2 is the lock stamp and nothing else** — zero changes to §1–§9, verifiable by diff. **Records J-757.** 🛑 **A LOCK IS OF A VERSION, NOT OF A FILENAME (J-754)** — Clair implements from **v1.2** and from no earlier version |

🛑 **NOTHING READS THE FIELD WHEN THIS LEG IS DONE.** No gate, no mutation event, no client. **That is the point:** the field lands, converges and defaults correctly *before* anything depends on it, so a defect here surfaces as a failed assertion rather than as an admission decision.

🔒 **UNRECOGNISED VALUES ARE RULED — `D-149`, 2026-08-16, IN THIS MILESTONE. v1.0 SAID THEY WERE "JOE'S AND UNRULED" AND THAT WAS FALSE (Clair `F-1`).** The rule: **a field that GATES fails CLOSED; a field that governs DISPLAY takes its DEFAULT.** `admission` gates ⇒ **unrecognised ⇒ behave as `invite`**.

🔑 **AND IT CHANGES NOTHING THIS LEG DOES — FOR A REASON §4.3 MUST CARRY, BECAUSE LEG D READS IT:** both of `D-149`'s own precedents **interpret at USE, not at PARSE** — `should_include_member_temperature` (`state.rs:1759-1784`) and the expiry gate's `.unwrap_or(true)` (`runtime.rs:1591`). ⇒ **the constructor stores the value VERBATIM; fail-closed lives in Leg D's gate.** 🛑 ***Write no validation, no allow-list, no normalisation — not because the question is open, but because the answer is enforced somewhere else.***

🔓 **ONE §15 ITEM IS GENUINELY OPEN AND MUST NOT BE ANTICIPATED: who may change `admission` (§15.3).** It is a permission question, it belongs to Leg C, and nothing in this leg touches it.

---

## §1 — 🔒 THE GOAL. **FROZEN. DO NOT RE-OPEN.**

> **`SpaceState` carries an `admission` value for every Space: `open` for a plain Space unless its create event says otherwise, and `invite` for every DM, pinned at creation and ignoring content.**

📌 **Refinements go in §3 as measurement notes; §1's text does not move.**

---

## §2 — THE SITES. **MEASURED AT `3876950`. OPEN EACH ONE BEFORE EDITING IT.**

| id | site | what is there today |
|---|---|---|
| **S-1** | `xgen-common/src/wire.rs:597-603` | the `// ── Pacing rules` banner and `DEFAULT_HUMAN_PACING_MS` / `DEFAULT_AI_PACING_MS` |
| **S-2** | `xgen-common/src/wire.rs:631-641` | `VISIBILITY_MODERATOR` / `_EVERYONE` / `_SELF_ONLY` and `DEFAULT_MEMBER_TEMPERATURE_VISIBILITY` — **the named-permitted-values pattern this leg copies** |
| **S-3** | `xgen-core/src/space/state.rs:186-258` | `pub struct SpaceState`, last field `threads` at `:257` |
| **S-4** | `xgen-core/src/space/state.rs:307-310` | `member_temperature_visibility`'s parse — **the exact `unwrap_or_else` idiom to copy** |
| **S-5** | `xgen-core/src/space/state.rs:312-336` | `from_space_create`'s `Ok(SpaceState {` literal, ending `threads: HashMap::new()` at `:335` |
| **S-6** | `xgen-core/src/space/state.rs:443-468` | `from_dm_space_create`'s literal, ending at `:468` |
| **S-7** | `xgen-core/src/space/state.rs:559-583` | `from_dm_space_create_node`'s literal, ending at `:583` |
| **S-8** | 🔑 **`xgen-core/src/wire/types.rs:14-22`** | **a HAND-MAINTAINED `pub use` list, not a glob** (Fix 17). `state.rs:32-36` reaches `DEFAULT_MEMBER_TEMPERATURE_VISIBILITY` **through it**. 🛑 **Adding a constant to `xgen-common` does NOT make `crate::wire::types::DEFAULT_ADMISSION` exist** (Clair `F-2`) |
| **S-9** | **`xgen-core/src/resolution/algorithm.rs:414`** | **the fourth `SpaceState` literal** — a test helper, `simple_space_state`. **Named nowhere in v1.0** |

🛑 **THIS IS THE MEASURED SET, AND v1.0's "TWO FILES BY DESIGN" WAS WRONG — IT IS FOUR:** `xgen-common/src/wire.rs` · `xgen-core/src/wire/types.rs` · `xgen-core/src/space/state.rs` · `xgen-core/src/resolution/algorithm.rs`. ✅ **All four are inside `xgen-common`/`xgen-core`, so V-4's crate scope holds.**

⚠️ **`SpaceState` has NO `Default` impl and NO `..Default::default()` escape**, so **every struct literal must gain the field or the build breaks.** ✅ **That is a FEATURE: the compiler enumerates them.** 📌 **§3's `M-4` records the census as measured; `V-0` is what makes it honest at implementation time.** *The runbook names the sites the DESIGN touches; the compiler names the sites the CHANGE touches, and this runbook no longer pretends the first list is the second.*

---

## §3 — MEASUREMENT NOTES

**M-1.** `SpaceState` derives `Debug, Clone, PartialEq, Eq` (`state.rs:185`) and **is never serialised** — it is folded from the log by `derive_resolved` (`runtime.rs:677` / `:832` / `:857`). ⇒ **a new `String` field is additive for M8 convergence** and needs no migration, exactly as `jurisdiction` and `e2e_encryption` are.

**M-2.** The three defaults `from_space_create` already consumes are declared in **`wire.rs`**, not `state.rs`. 🛑 **Phase-0 v2.0 said `state.rs` has no `const` block and drew the wrong conclusion from it** — the sweep pattern only looked in one file. Corrected at v2.1. ⇒ **`DEFAULT_ADMISSION` goes in `wire.rs` with its siblings.**

**M-3.** `state.rs` currently holds **82 `#[test]` functions**; the file is the natural home for this leg's tests (its existing constructor tests sit at `:2811-2930`).

**M-4.** 🔒 **THE LITERAL CENSUS, MEASURED AT `3876950` BY READING EVERY HIT: FOUR.** `state.rs:312` · `state.rs:443` · `state.rs:559` · `algorithm.rs:414`. ⚠️ **A `grep` for `SpaceState {` returns THIRTEEN** — the other nine are `-> SpaceState {` return types (two in `xgen-node`, which is why the wrong reading looks alarming), one `pub struct`, one `impl`. 🔑 ***Clair's first census returned twelve and she caught it by READING each hit rather than counting them — the same instrument failure class as `N-197`, and the wrong reading happened to be the frightening one, which is the only reason it got a second look.*** 📌 **`V-0` still runs: this census is `HEAD`'s, and the compiler's is the leg's.**

**M-5.** 🔒 **NEITHER BUILDER ACCEPTS AN `admission` KEY** (Clair `F-3`): `build_space_create_event` (`state.rs:1382-1390`) and `build_dm_space_create_event` (`:1812-1816`) both hard-code their content. **Widening them is the file's own convention for set-once fields — and costs `140 + 27 − 2 = 165 CALL SITES ACROSS FOUR CRATES`, measured.** 🔒 **RULED (Joe, 2026-08-18): the tests mutate content BEFORE signing instead (§4.4). Widening becomes LEG C's**, where a client genuinely needs to *set* the value — which is §6 item 3 arriving on schedule rather than as a surprise.

---

## §4 — THE CHANGE

### §4.1 — `xgen-common/src/wire.rs` — the constants

🔑 **A NEW BANNER, NOT UNDER Temperature's.** After `clamp_temperature`'s block, open `// ── Space admission (spec 3.7.14) ──` on the file's existing banner form, then — on **S-2's** `VISIBILITY_*` pattern:

- `pub const ADMISSION_OPEN: &str = "open";`
- `pub const ADMISSION_INVITE: &str = "invite";`
- `pub const DEFAULT_ADMISSION: &str = ADMISSION_OPEN;`

🛑 **DOC COMMENTS CITE `ch3 §3.7.14.2`, THE SPEC, LIKE EVERY NEIGHBOUR IN THIS FILE — NOT task-file locks (Clair's note).** §3.7.14 exists at `docs/xgen_ch3_specification.md:2827` and §3.7.14.2 is the admission property. **`L-E` and `L-C` may be mentioned; the SPEC § is the citation.** 🛑 **No enum. No `is_valid_admission()`** — `D-149` puts the interpretation at USE, and a validator here would move it to parse.

### §4.1b — `xgen-core/src/wire/types.rs` — the re-export (**S-8**)

Add **`ADMISSION_INVITE`, `ADMISSION_OPEN`, `DEFAULT_ADMISSION`** to the `pub use xgen_common::wire{…}` list at `:14-22`, in its existing alphabetical-ish grouping. 🛑 **Internal code then imports from `crate::wire::types::` — NEVER from `xgen_common` directly.** *That is Fix 17's stated convention, and Legs C and D inherit whichever path this leg sets.*

### §4.2 — `xgen-core/src/space/state.rs` — the field

**S-3:** add `pub admission: String,` **after `threads` (`:257`)**, with a doc comment on `member_temperature_visibility`'s model (`:246-249`): open enum, `String`, values `open` / `invite`, **absent at create ⇒ `DEFAULT_ADMISSION`**, **DM ⇒ always `invite`**, and — stated plainly — **nothing reads it until Leg D**.

Then **`cargo check --workspace` and let the compiler enumerate every literal.** 📌 **`M-4` predicts FOUR; the compiler is the authority.** For each one:

- **S-5 `from_space_create`** → parse per §4.3, insert `admission` after `threads:`.
- **S-6 / S-7, the two DM constructors** → **`admission: ADMISSION_INVITE.to_string()`, unconditionally, content NOT consulted** (`L-C`, Q4(b)).
- **S-9 `algorithm.rs:414`** → a plain-Space test helper ⇒ **`DEFAULT_ADMISSION.to_string()`**. ⚠️ **It is imported into `algorithm.rs`, which may need its own `use` line — through `crate::wire::types`, per §4.1b.**
- **any site the compiler names that `M-4` did not** → 🛑 **§7 FINDING ①, reported before it is filled in.**

### §4.3 — the parse, in `from_space_create`

Beside **S-4**, in the same block and the same shape:

```rust
let admission = content["admission"]
    .as_str()
    .map(str::to_string)
    .unwrap_or_else(|| DEFAULT_ADMISSION.to_string());
```

🛑 **`unwrap_or_else`, not `unwrap_or`** — matching `:310` exactly. 🛑 **No trimming, no lowercasing, no membership check.** Absent ⇒ `open`; **present ⇒ stored verbatim, whatever it says.**

📌 **A comment here carries the REASON, because Leg D reads it:** `D-149` puts unrecognised-value handling at **use**, not at parse — `should_include_member_temperature` and the expiry gate both do it that way — so **the constructor's job is fidelity and the gate's job is judgement.**

### §4.4 — the tests. **FOUR, IN `state.rs`'s `mod tests`.**

🔒 **THE INJECTION IDIOM, RULED (Joe, 2026-08-18) — MUTATE BEFORE SIGNING.** Tests 2–4 need a content key **neither builder accepts** (`M-5`). The builders return an **UNSIGNED** `Event`; `sign_event` is applied afterwards at every call site. ⇒

```rust
let mut ev = build_dm_space_create_event(&key, &invitee, node);
ev.content["admission"] = json!("open");   // BEFORE signing — not tampering
let ev = sign_event(ev, &key);
```

🛑 **THE COMMENT IS MANDATORY AND MUST SAY WHICH IDIOM THIS IS.** `state.rs`'s test module contains a mutation that happens **AFTER** signing, deliberately, to produce a **tampered** event for a signature-rejection test (Clair `F-3`). ***These two look alike and mean opposite things; a reader who copies the wrong one writes a test that proves nothing.***

1. `from_space_create_absent_admission_defaults_to_open` — create content **without** the key ⇒ `state.admission == ADMISSION_OPEN`.
2. `from_space_create_present_admission_is_stored_verbatim` — content carries **`"invite"`** ⇒ stored as given. 🔑 **Then a second assertion in the same test with an unrecognised value (`"banana"`) ⇒ stored verbatim too.** ***This is the test that proves §4.3 added no validation — and it must be written to FAIL if a future leg adds one, because that leg needs to see this assertion and change it deliberately.***
3. `from_dm_space_create_pins_invite_ignoring_content` — build the DM create event **with `"admission": "open"` injected per the idiom above** ⇒ `state.admission == ADMISSION_INVITE`. 🔑 **The injected value must be the WRONG one, or the test cannot tell pinning from parsing** — ⚠️ **and the nearest DM precedents use the bare builder, one of which admits in its own comment that it cannot discriminate.**
4. `from_dm_space_create_node_pins_invite_ignoring_content` — same, on the node-side seed.

🛑 **FOUR TESTS ⇒ `cargo` MOVES 1604 → 1608.** **56 SUITES stays 56** — this leg adds no test module.

---

## §5 — VERIFICATION. **CHAT RE-DRIVES ALL OF THESE INDEPENDENTLY (Rule 5).**

| gate | how | expected |
|---|---|---|
| **V-0** | `cargo check --workspace` **before any literal is filled in**, output kept | 🔑 **the compiler's list of every `SpaceState` literal — the census this runbook cannot write.** ⚠️ **Its COUNT is recorded in the close as a measurement, not predicted here** |
| **V-1** | `cargo test --workspace`, detached, own exit sentinel, final `test result:` line required, summed programmatically over `^test result:` | **1608 / 0 / 62**, exit 0, all four tests named in the output, `Compiling` present ⇒ not a cached pass. 📌 **`1604` is CARRIED from J-755, where BOTH SIDES were measured on one tree** — stated, not silently assumed |
| **V-2** | suite count from the same run | **56 SUITES** — 🛑 **structural; a change is a §7 FINDING, not a correction** |
| **V-3** | 🔒 **THE NEGATIVE CONTROL, AND IT IS THE ONE THAT CAN FAIL SILENTLY.** By hand, discarded, **never committed**: change §4.3's `unwrap_or_else` to a hardcoded `"invite"` and re-run test 1 | **test 1 must FAIL.** 🛑 **If it still passes, the fixture never exercised the absent path and the default is unproven.** Revert immediately; `git status` clean before continuing |
| **V-4** | `git diff --numstat` vs the leg's parent | **FOUR files, all in `xgen-common`/`xgen-core`** — `wire.rs`, `wire/types.rs`, `state.rs`, `algorithm.rs`. 🛑 **ZERO `xgen-node`, ZERO `xgen-client`, ZERO `ui/**`** |
| **V-5** | `git ls-files --eol` on every touched file | `i/lf w/lf` throughout |
| **V-6** | `grep` the diff for `admission` | 🛑 **no `match` on the value, no `if admission ==`, no allow-list, anywhere.** *The absence of a reader is this leg's defining property and is checkable* |

📌 **vitest and svelte-check are CARRIED BY SCOPE** (zero `ui/**`) — **stated in the close, not silently skipped.**

---

## §6 — WHAT THIS LEG DOES **NOT** CLAIM

1. **It does not gate anything.** Every Space, `open` or `invite`, admits exactly whom it admits today. **Leg A-bis's two shipped tests must both stay green** — including the DM one that asserts a stranger IS admitted.
2. **It does not decide what an unrecognised value means — `D-149` already did (unrecognised ⇒ fail closed, because `admission` gates).** 🔑 **This leg stores such a value verbatim, and that is not a contradiction: `D-149`'s interpretation happens at USE.** ⚠️ **Between this leg and Leg D, a Space carrying an unrecognised value therefore behaves as `open`** — because nothing reads the field at all. *Stated so Leg D does not discover it as a surprise.*
3. **It does not let anyone SET `admission`** on a plain Space in practice — no client writes the key and there is no builder parameter. **A create event carrying `admission` can only be hand-built.** ⚠️ **That is Leg C's problem and is named here so it is not discovered there.**
4. **It says nothing about federation.** A peer's `SpaceState` gains the same field by the same fold; **no wire message changes.**

---

## §7 — FINDING TRIGGERS. **REPORT, NEVER ABSORB (Rule 6).**

① V-0 enumerates a `SpaceState` literal **outside `xgen-core`** · ② the suite count moves off 56 · ③ any Leg A-bis test goes red · ④ a cited line number in §2 does not hold at the leg's parent commit · ⑤ V-3's control passes · ⑥ the field cannot be added without touching a serialisation path — 🛑 **that would contradict M-1 and is a design-level finding, not an implementation choice.**

---

## §8 — ORDERING

**§8.1.** 🛑 **BEFORE reading §4, run `V-0` and read `from_space_create` (`state.rs:265-337`) whole.** Derive from the source what the parse must look like, **write it down**, then compare with §4.3. *If they differ, that is a finding about this runbook.*
**§8.2.** §1 is **frozen**. §4.4's counts (four tests, 1608, 56 suites) are **predictions**; §5 is what decides. 📌 **v1.0 cited a §4.6 that does not exist — corrected at v1.1 (Clair's note); the phantom is named rather than silently deleted.**

---

## §9 — DoD

- [ ] `wire.rs` carries three constants under their own **`spec 3.7.14`** banner, doc-comments citing **ch3 §3.7.14.2**; **no enum, no validator**
- [ ] **`wire/types.rs`'s `pub use` list re-exports all three** — and every consumer imports via `crate::wire::types`
- [ ] `SpaceState.admission` exists with a doc comment naming `L-E`, `L-C` and *nothing reads it until Leg D*
- [ ] `from_space_create` parses on `member_temperature_visibility`'s idiom; **both DM constructors pin, ignoring content**
- [ ] Every literal the compiler enumerates is filled — **`M-4` predicts four; a fifth is a §7 finding**
- [ ] **Tests 2–4 inject content BEFORE signing, with the comment distinguishing that from the tampering idiom**
- [ ] Four tests, V-1 **1608 / 0 / 62 × 56 SUITES** measured by Chat
- [ ] **V-3 run, the control FAILED as required, and the scratch reverted** — `git status` clean
- [ ] V-6: no reader of the value anywhere in the diff
- [ ] Records: JOURNAL + `CLAUDE.md` + ROADMAP + this file's `Status: COMPLETED`, **one `D-074` commit**

📌 **"Commit pushed" is not a DoD item.** `Status: COMPLETED` in this header is the signal.
