# M-RP-IDENTITY-RESOLUTION Leg A — the `not_found` id list
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

🔒 **LOCKED BY JOE 2026-08-01 (J-646) — *"lock"*.** ✅ **EXECUTED AND CLOSED 2026-08-01 (J-647) — see §0.** ⚠️ *v1.1 carried the "amend only by further Joe ruling" clause and the "locked and idle, Clair not stood up" note; both are DISCHARGED — she was stood up and the leg ran. Superseded, kept not erased (`D-131`).*

---

## §0 — ✅ EXECUTION RECORD (J-647, 2026-08-01)

**Implemented by Clair from this file at v1.1. Every gate RE-DRIVEN by Chat — no figure read off her report.**

🔑 **HER ADVERSARIAL READ FOUND NOTHING TO STOP ON, AND SHE SAID SO EXPLICITLY** — the runbook was consistent with itself and with the Phase-0 (§4's type against G-A7's sibling, §7's floor against Phase-0 §8, §2's two-file scope against §8's leg split). ✅ **After J-643, where two of her three flags were defects in Chat's own documents, a clean read is a result worth recording rather than an absence.**

| gate | re-driven result |
|---|---|
| `git diff --stat` | **2 files, +23 / −0** |
| `cargo test --workspace` | **1588 / 0 / 62 across 56 result lines**, 0 errors, 0 warnings · **`Compiling xgen-client` present ⇒ live, not cached** |
| `svelte-check` | **0 errors / 34 warnings / 15 files** — the floor exactly |
| `git ls-files --eol` | **`i/lf` on BOTH.** `ops.rs` `w/crlf` is pre-existing autocrlf; the committed form is unchanged |
| construction sites | **still exactly ONE** (now `:2904`) |
| counter retained | ✅ `report.not_found += 1` at `:2929`, push at `:2935` — **additive** |

📌 **LINE NUMBERS IN §6 AND §8 BELOW ARE PRE-CHANGE** (`:2893` / `:2917`); after the edit the literal is `:2904` and the push `:2935`. **Kept as written — they were correct instructions at authoring time and back-editing them would destroy the audit trail.**

✅ **ADJACENT FILES OPENED — THE J-643 DISCIPLINE, BECAUSE A DIFF CANNOT SHOW AN ERASER ONE LAYER DOWN:** `FillMembersOutcome` (`ops.rs:2951`) carries plain `Serialize` with **no `skip`, no `rename_all`** · `desktop.rs:673` returns the outcome **whole**, no reconstruction or narrowing · **no equality assertion and no JSON-shape test on `FillReport` exists anywhere**. 🔑 **NO ERASER FOUND — unlike J-643, the change is not undone below.**

🛑 **AND THE HONEST LIMIT HELD: THIS LEG IS COMPILE-VERIFIED ONLY.** The push has no unit test and cannot get one without a live node. **Leg F is the first behaviour verification.** **1588 unchanged is the CORRECT result.**

📌 **Parent Phase-0:** `tasks/M_RP_IDENTITY_RESOLUTION.md` **v1.4** — read §1 (grounding), §3 (the four states) and §6 (G-A, now CLOSED) before this file. 🛑 **RUNBOOK-AS-GROUND-TRUTH IS A FAILURE MODE.**

🔑 **SESSION-OPEN READING ORDER FOR CLAIR (MANDATORY):** `CLAUDE.md` PLAY head → `JOURNAL.md` J-646 → the Phase-0 §1/§3/§6 → **then this file in full.** ⚠️ **This runbook is item 4, not item 1.**

---

## §1 — What this leg does, in one sentence

**`FillReport` gains `not_found_ids: Vec<IdentityXgid>`, populated at the one site that already knows which lookup failed, and the TypeScript mirror gains the matching field.** Nothing reads it yet.

🔑 **WHY IT IS FIRST: it is the precondition for every render rule in the milestone.** §4's dimming and §5's hiding both need to know *which* members returned `not_found`, and today the client is told only *how many*. **Legs B and C cannot start until this lands.**

---

## §2 — Files (exactly two)

| # | File | Change |
|---|---|---|
| 1 | `xgen-client/src/ops.rs` | one struct field + one push + doc comment |
| 2 | `ui/common/lib/stores/address-book.svelte.ts` | one interface field + doc comment |

🛑 **NO OTHER FILE.** Not `desktop.rs` (the command already returns `FillMembersOutcome` whole) · not `address_book.rs` · not `members-panel.svelte` · not `app_client.svelte`.

⚠️ **`app_client.svelte:183` DISCARDS `outcome.fill` TODAY** — `addressBook.setResult(sid, outcome.roster, book)`. **That is correct for this leg and must not be "fixed" here.** Wiring the fill half into the store is **Leg B's** job, beside the render rules it feeds. Touching it here mixes a cargo-floor leg with a `svelte-check` leg and makes a regression unattributable.

---

## §3 — ✅ Grounding (measured 2026-08-01 at `e9cde04`)

**G-A1 — `FillReport` has exactly ONE construction site and ONE increment.**
- construction: `ops.rs:2893` — a struct literal naming all four fields.
- increment: `ops.rs:2917` — `report.not_found += 1;`

**G-A2 — 🔑 THE ID IS ALREADY IN SCOPE AT THE INCREMENT.** `:2912` is `for id in &to_fetch`, and `:2913` passes that same `id` to `identity_get_on`. **No lookup, no re-derivation, no new plumbing — the value is one identifier away.**

**G-A3 — no test constructs a `FillReport`.** A repo-wide search for `FillReport {` returns the single production literal. ⇒ **adding a field breaks no test.**

**G-A4 — `Serialize` is already derived** (`ops.rs:2772`) and `desktop.rs:668` returns `FillMembersOutcome` whole. ⇒ **the field crosses the Tauri boundary with no command change.**

**G-A5 — the failure modes are exhaustive within a completed fill.** `:2913` is `identity_get_on(...).await?` — **the `?` aborts the entire fill on a transport error**, and the shell runs `setFailed`. ⇒ inside a fill that returns `Ok`, **every id is either fetched or `not_found`**; there is no per-id "asked and never heard back".

**G-A6 — 🔒 `not_found` MEANS ERASED, and this is locked elsewhere.** `D-127` (cited in `tasks/M13_CLIENT_IDENTITY_LOOKUP_WIDENING.md:54`): *a revoked Identity returns its record WITH `revoked` set, never `identity.not_found`; `not_found` is reserved for erasure.* ⚠️ **Independent corroboration, from a lock that predates this milestone, of the Phase-0's §1 G3** — the conclusion Joe's *"how does Carol say hello"* question forced.

**G-A7 — the sibling type in the same returned struct is typed.** `MemberEntry.identity_id: IdentityXgid` (`ops.rs:2617`) · `MembersResult.owner_id: IdentityXgid` (`:2630`). **The consumer will match `not_found_ids` against roster rows, so the roster's type is the one to match.**

---

## §4 — 🔒 The type: `Vec<IdentityXgid>` (Joe, locked 2026-08-01 — option X1)

🔒 **`not_found_ids: Vec<IdentityXgid>`.** Built at the push site with the macro's documented parse-boundary constructor:

```rust
IdentityXgid::from_xgid(Xgid::new(id.clone()))
```

**Why, in the two lenses:**
- ① **User-visible: NOTHING, under every option considered.** `Xgid` is `#[serde(transparent)]` (`base.rs:35`) and so is every flavour (`flavours.rs:131`); the doc calls the wire form *a plain JSON string, Appendix J §J.5 invariance 2*. TypeScript already sees `MemberEntry.identity_id: string`. **The bytes are identical whether the field is `String` or `IdentityXgid`.**
- ② **Resource: one line and one field, no signature changes, no test churn** — and it is **the only option that survives the fix in §5.** Once `M-RP-XGID-SLOT-RETYPE` lands, this field needs **no rework**; `Vec<String>` would have to be changed twice.

⚠️ **THE CONSTRUCTOR CALL LOOKS UGLY AND THAT IS DELIBERATE.** It re-upgrades a value that was downgraded eight lines earlier at `:2734` / `:2742`. **That ugliness is the §5 defect being visible rather than invisible**, and it deletes itself when the retype lands.

---

## §5 — 🛑 A KNOWN DEFECT IS BEING WORKED AROUND, NOT FIXED (Joe: *"keep the bug and correct it at the closest proper opportunity"*)

🛑 **`D-136` — a completed sweep is not a standing rule.** The XGID Retrofit arc closed 2026-05-29 having retyped all of `xgen-client`; `SeenRecord`, `FetchedIdentity` and `FillReport` were then written in **`String`** on 2026-07-25, two months later, in the same crate — and `MemberEntry`, written 3 days after the arc closed, is correctly typed. **`String` compiles, so nothing caught it.**

🔒 **FILED AS `M-RP-XGID-SLOT-RETYPE` 🟡 PENDING (J-645).** ⚠️ **NOT part of this leg and MUST NOT be started inside it.** `D-071` — a subsystem audit precedes the dependent milestone; folding a retype into a leg whose whole point is one attributable field is how a regression becomes untraceable.

📌 **REQUIRED — a code comment at the wrap site naming the milestone**, so the re-upgrade reads to the next author as *a known defect being worked around*, not as clumsiness. **This is a DoD item, not a nicety.**

---

## §6 — The changes, exactly

### Change 1 — `ops.rs`, the field (at `:2779`, after `not_found`)

Add, with a doc comment recording **what it is** and **why the type is what it is**:

```rust
/// The identities that returned `identity.not_found` in THIS fill — the
/// list behind the `not_found` count. The panel needs the ids, not the
/// tally: hiding an erased member while dimming an unresolved one is a
/// per-row decision (`M-RP-IDENTITY-RESOLUTION` §4/§5), and a count
/// cannot drive it.
///
/// Under `D-127` a `not_found` reply means **erased**, not revoked — a
/// revoked identity returns its record with `revoked` set. Combined with
/// validation step 11 (`exchange.rs:208-210`), an id in this list names a
/// member whose events the node now REJECTS.
pub not_found_ids: Vec<IdentityXgid>,
```

### Change 2 — `ops.rs:2893`, the literal

Add `not_found_ids: Vec::new(),` to the struct literal. ⚠️ **The literal names every field explicitly — there is no `..Default::default()` — so omitting it will not compile.** That is the desired behaviour.

### Change 3 — `ops.rs:2917`, the push

The `else` arm becomes **both** statements — the counter is **kept**, not replaced:

```rust
} else {
    report.not_found += 1;
    // D-136 / M-RP-XGID-SLOT-RETYPE: `to_fetch` is Vec<String> only because
    // observed_identities downgrades typed ids (:2734/:2742) to feed the
    // String-keyed AddressBook — a post-retrofit regression, filed not fixed.
    // This re-wrap goes away when that milestone lands; the field type is
    // already correct and needs no rework.
    report.not_found_ids.push(IdentityXgid::from_xgid(Xgid::new(id.clone())));
}
```

🛑 **THE COUNTER STAYS.** `not_found` is a public field with existing readers assumed; **this leg is additive.** A reader that treats `not_found_ids.len()` as authoritative is Leg B's business, not this one.

📌 **Imports: BOTH TYPES ARE ALREADY IN SCOPE — measured, not assumed.** `ops.rs:26` is `use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid};`. ⇒ **no import change is needed, and adding one is a defect.**

### Change 4 — `address-book.svelte.ts:51-56`, the mirror

```ts
  not_found_ids: string[];
```

with a comment noting it is `Vec<IdentityXgid>` Rust-side and a bare string over the wire (serde-transparent), and that **nothing reads it until Leg B**.

🔑 **WHY THE MIRROR IS IN THIS LEG AND NOT LEG B (Joe, boundary option ②).** The interface's own doc comment calls it *"ops.rs `FillReport`"* — **it is a declared mirror, and a mirror that is stale for a whole leg is the exact defect this milestone has already caught three times.** A type-only field with zero readers cannot regress anything.

---

## §7 — 🛑 Verification, and an honest statement of its limit

### What CAN be verified

| Gate | Expectation |
|---|---|
| `cargo build --workspace` | clean |
| `cargo test` | **1588 / 0 / 62 × 56 — UNCHANGED** |
| `svelte-check` | **0 errors / 34 warnings / 15 files — UNCHANGED** |
| `git diff --stat` | **2 files** |
| line endings | `ops.rs` and the `.ts` are **LF in the index**; confirm with `git ls-files --eol`, **not** a worktree grep (the J-643 trap) |

### 🛑 What CANNOT be verified, stated so it is never later read as done

🛑 **THE PUSH AT `:2917` HAS NO UNIT TEST AND CANNOT GET ONE WITHOUT A LIVE NODE.** The existing `not_found` test (`ops.rs:3829`, `absorb_fetch_notfound_is_skipped_without_poisoning_the_book`) covers **`absorb_fetch`**, which is pure and **never touches `FillReport`** — the report is built by the caller. The increment sits inside `fill_from_events`, downstream of `ensure_connected` and `identity_get_on`.

🔒 **Joe locked T-a: ship it untested rather than refactor `fill_from_events`** — the function carrying the `session.conn` re-entrancy invariant that J-586 already got burned by.

⇒ **LEG A IS COMPILE-VERIFIED ONLY. IT IS NOT BEHAVIOUR-VERIFIED.** The first leg that can verify it is **Leg F** (two clients, a real erased member).

⚠️ **CONSEQUENCE TO EXPECT AND TO STATE IN THE COMMIT: the cargo count will not move.** **1588 unchanged is CORRECT, not a failed run** — but unstated it reads exactly like a leg that did nothing.

📌 **`A PROBE THAT CANNOT FAIL IS NOT EVIDENCE.`** Do **not** add a test that asserts `not_found_ids.is_empty()` on a fresh `FillReport`, or that the literal compiles. **Those pass by construction and would make an unverified leg look verified** — worse than the honest gap.

---

## §8 — DoD

- [x] `not_found_ids: Vec<IdentityXgid>` added to `FillReport` with its doc comment — ✅ `:2780-2790`
- [x] the `:2893` literal names the new field — ✅ now `:2908`, named-fields, no `..Default`
- [x] the push added at `:2917`, **`report.not_found += 1` retained** — ✅ counter `:2929`, push `:2935`
- [x] **the `D-136` / `M-RP-XGID-SLOT-RETYPE` comment is present at the wrap site** (§5 — a DoD item) — ✅ `:2930-2934`
- [x] **no import line changed** — ✅ `ops.rs:26` untouched
- [x] `not_found_ids: string[]` added to the TS `FillReport` with its comment — ✅ `:55-59`
- [x] `cargo test` re-run: **1588 / 0 / 62 × 56, unchanged** — ✅ **and the non-delta is the expected result, not a failed run**
- [x] `svelte-check` re-run: **0 / 34 / 15, unchanged** — ✅
- [x] `git diff --stat` = **exactly 2 files** — ✅ +23 / −0
- [x] `git ls-files --eol` on both files — ✅ **`i/lf` both, index form unchanged**
- [x] **§7's limit restated in the close**: compile-verified only; Leg F is the first behaviour verification — ✅ §0, J-647, ROADMAP and the PLAY block
- [x] Records: JOURNAL + `CLAUDE.md` PLAY + ROADMAP + this runbook in one commit (`D-074`) — ✅ J-647

⚠️ **"Commit pushed" is deliberately NOT a DoD line.** `Status: COMPLETED` in this header is the real signal.

---

## §9 — Seats

**Clair** implements from this file **once Joe locks it**. **Chat re-drives every gate in §7 — no figure is read off the implementation report** (the J-643 rule, which caught a renderer defect that the report could not have shown). **Clair does not close her own leg.**

🔑 **AND THE REVERSE HELD AT J-643: two of the three defects she flagged were in Chat's documents.** If anything in this runbook contradicts itself or contradicts the Phase-0, **say so and stop** — a verification gate that disagrees with its own DoD stops the implementer at the last step, and that has already happened once on this milestone's parent.

✅ **AT THIS LEG SHE FOUND NOTHING TO STOP ON AND SAID SO EXPLICITLY.** 📌 **A clean adversarial read is a RESULT, not an absence** — recorded so the seat is not judged only by the arcs where it catches something.

---

## §10 — 📌 FILED AT THE CLOSE, NOT FIXED

🛑 **THE `Vec<IdentityXgid>` WIRE SHAPE HAS NO WITNESS — ONLY A SCALAR ONE.** §4 justifies the type partly on *"serde-transparent ⇒ TypeScript sees `string`"*. The standing witness for that property is **`ops_result_struct_serde_transparent_wire_invariance` (`ops.rs:3438`, the Pass 4 T2 gate) — and it covers SCALAR `IdentityXgid` slots only** (`CreateSpaceResult`). **There is no witness for a `Vec<IdentityXgid>`.**

✅ **The `string[]` mirror is still correct** — `Vec<T>` serialises as a JSON array of `T`, and `T` is transparent. ⚠️ **But that is INFERENCE FROM SERDE SEMANTICS, not a measured property of this field**, and citing T2 as its witness would be a claim narrower than the thing it describes — the species this milestone has now caught five times.

📌 **A Vec-level wire witness would be a LEGITIMATE test** — it could genuinely fail if a serde attribute were added later, so it is **not** a probe-that-cannot-fail and §7's ban does not reach it. **It is excluded only by scope:** it would move cargo to 1589 and contradict Joe's locked **T-a** and the two-file boundary.

⇒ **FILED FOR LEG B**, which reads the field and can assert the shape **against a real consumer** rather than in a vacuum.
