# M-RP-XGID-SLOT-RETYPE — the identifier slots that regressed to `String` after the retrofit arc closed
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

**This is the Phase-0 for `M-RP-XGID-SLOT-RETYPE`, the milestone `D-136` §4 names as owing the sweep that entry deliberately did not run.** `D-136` closed with an explicit disclaimer: *"It does not claim the XGID regression's full extent is known. Only the structs on the address-book fill path were measured. No sweep has been run for other post-2026-05-29 structs that regressed the same way."*

✅ **THAT SWEEP HAS NOW RUN — §1.** It is the reason this document exists and the reason the milestone is larger than the three structs that minted it.

🛑 **WHAT THIS IS NOT.** Not a re-audit of the five retrofit passes — `D-136` §4 is explicit that *"the defect is in what a close means, not in the work"*, and that stands. Not a re-opening of Pass 4's classification locks for the code that was in scope on the day they ran. Not `M13`.

---

## §1 — Grounding (measured 2026-08-02 at `4bb7b45`, HEAD = origin/main, tree clean)

### §1a — The scope question was settled before the numbers, because the numbers are meaningless without it

**Read from the five pass documents in `tasks/archive/`, not recalled:**

| pass | scope | closed |
|---|---|---|
| 1 | `xgen-common` + `xgen-core` **data structures** | J-122, 2026-05-26 |
| 2 | `xgen-core` **algorithm-bearing functions** | J-126, 2026-05-27 |
| 3 | `xgen-node` modules | J-138, 2026-05-28 |
| 4 | `xgen-client` — *"42 String identifier slots across 7 subsystems"* | J-146 |
| 5 | trace-field / `Debug` / `Display` sweep | J-148, `7ed4e30`, 2026-05-29 12:24:02 |

⇒ 🔑 **ALL FOUR CRATES WERE SWEPT.** There is no crate the convention never reached, so **every candidate below sits inside territory the arc already covered** and none can be dismissed as out of scope.

### §1b — 🛑 THE PARTITION WAS RUN TWICE, ON TWO DIFFERENT SUBJECTS, AND THE FIRST SUBJECT WAS THE WRONG ONE

`D-136`'s mechanism is regression in code written **after the pass closed**. The obvious proxy is an author-date test against 2026-05-29.

🛑 **IT DOES NOT SEPARATE THE CASES. The close commit `7ed4e30` landed at 12:24:02 on 2026-05-29, and 36 candidates carry that same date** — a day-granularity test cannot tell *written that morning, inside the arc* from *written that afternoon, after it*.

✅ **Re-run as `git merge-base --is-ancestor <blame-sha> 7ed4e30` over the 55 distinct blame commits, the partition came back IDENTICAL: 125 in-arc, 130 post-arc, same file breakdown.**

🔑 ***The date proxy was right, and it was right by luck rather than by construction.*** 📌 *Recorded because a check that COULD have failed and did not is evidence, where a check that cannot fail is not (the J-655 broken-probe lesson). The ancestry test is the one this milestone's later legs must use.*
### §1c — The result

**255 candidate identifier slots repo-wide** (regex over `*.rs` in all four crates, `target/` and `.claude/` excluded per the standing traps).

| | n |
|---|---|
| **in-arc** (ancestor of `7ed4e30`) | **125** |
| **post-arc** | **130** |

🛑 **THE REGRESSION IS MUCH WIDER THAN THE THREE STRUCTS THAT MINTED `D-136`: 19 files, all four crates.** `address_book.rs` contributes **2 of 130**.

**Post-arc, by file:** `xgen-node/src/admin_ops.rs` 63 · `xgen-client/src/app.rs` 16 · `xgen-client/src/ops.rs` 9 · `xgen-core/src/wire/types.rs` 8 · `xgen-client/src/desktop.rs` 5 · `xgen-common/src/aicontrol/envelope.rs` 4 · `xgen-node/src/audit.rs` 4 · `xgen-client/src/resident.rs` 3 · `xgen-node/src/protocol_audit.rs` 3 · `xgen-core/src/transport/connection.rs` 3 · `xgen-client/src/address_book.rs` 2 · `xgen-node/src/migration_driver.rs` 2 · `xgen-core/src/space/state.rs` 2 · then 1 each in `bootstrap_client_integration.rs` · `fanout.rs` · `batch.rs` · `aicontrol.rs` · `session.rs` · `trust_assertion.rs`.

**Excluding function parameters (12, a Pass 2/3/4 subject, not this one) and `clap`-derived structs, the core set is 88 slots across ~59 struct sites.**

---

## §2 — 🔑 THE FINDING: PASS 4's CLASSIFICATION IS BINARY AND THE CODEBASE HAS THREE CATEGORIES

Pass 4 §4.1.a locked: **identifier slots retype to a typed XGID flavour; descriptive slots stay `String`.**

🛑 **THE LARGEST POST-ARC BLOCK IS NEITHER.** `xgen-node/src/admin_ops.rs` holds **63 of the 130**, and they are **CLI argument structs and admin-pipe result structs** — `#[derive(clap::Args, serde::Deserialize)]` on the way in, `serde::Serialize` on the way out. ⇒ they are **boundary parse and boundary projection slots**: the point at which an untyped external string becomes, or stops being, internal state.

✅ **AND THE PROJECTION IS ALREADY BUILT AND DOCUMENTED.** `admin_ops.rs:977`:

```rust
/// Project a wire-format Identity URI to the typed key at the registry boundary.
fn ident_xgid(s: &str) -> IdentityXgid {
    IdentityXgid::from_xgid(Xgid::new(s.to_string()))
}
```

🔑 **THIS IS THE EXACT INVERSE OF THE CASE THAT MINTED `D-136`.** That entry's third corroboration reads: *"The downgrade at `ops.rs:2734`/`:2742` reads as a deliberate seam and is not one — it exists only to feed a `BTreeMap<String, SeenRecord>` that should not have been `String`-keyed."* **Here the seam reads as deliberate and IS deliberate**: the `String` is the external form, the projection is named, and the typed value is what crosses into the registry.

⇒ 🔑 ***`admin_ops.rs` is not a file that broke the rule. It is a file the rule does not reach.***

⚠️ **AND THAT IS THE BETTER EXPLANATION FOR A TWO-MONTH SILENT REGRESSION THAN CARELESSNESS.** `D-136` §2's own second corroboration was *"`address_book.rs` contains ZERO `IdentityXgid` — the type was never in the room."* **A binary rule offered no bucket for a boundary slot, so a whole subsystem was written outside it without anyone choosing to.**

### §2a — 🔒 RULED 2026-08-02 (Joe, *"go by your recommendations"*): A BOUNDARY SLOT MAY STAY `String`, AND IT BECOMES A NAMED THIRD CATEGORY

- **① User-visible impact:** **NONE, under either answer.** This is internal type discipline; no user of XGen sees, feels or comes to believe anything different. 📌 *`D-121`'s legal answer, and it is the true one here — stated plainly rather than dressed in a manufactured UX rationale.*
- **② Resource cost:** *accepting* — near zero in code (the projection exists), plus the cost of **writing the category down**, which is the entire point. *Rejecting* — `FromStr` on eight flavour newtypes, every CLI arg struct retyped, and a deserialisation story for each.

🔒 **THE THIRD CATEGORY, AS IT ENTERS THE RECORD: a BOUNDARY slot holds the external form of an identifier at a parse or serialisation edge, and stays `String` IF AND ONLY IF a named projection converts it at the boundary and no internal state holds the `String` form.** 🛑 **The second half is what stops this becoming an excuse** — `SeenRecord` would fail it, because a `BTreeMap<String, SeenRecord>` *is* internal state holding the external form.
---

## §3 — 🛑 THE CLASSIFICATION OF THE 88 IS A HYPOTHESIS, NOT A MEASUREMENT

The four-way bucketing (**30 `clap`-boundary · 47 serde-wire · 41 internal · 12 fn-param**) came from a **heuristic**: nearest enclosing `struct` plus a backward scan for a `derive`. It is not a read.

🛑 **AND IT IS ALREADY KNOWN TO UNDERCOUNT ITS OWN LARGEST BUCKET.** The inventory pass showed the boundary category reaches further than the `clap` derive does:

- **~25 `admin_ops.rs` `*Result` structs** (`FederationAcceptResult`, `SpaceUnbanResult`, `IdentityRevokeResult`, …) are **serde outputs to the admin pipe** — the same species, on the way out rather than in.
- **`xgen-client/src/app.rs`'s `BanArgs` · `LeaveArgs` · `RedactArgs` · `RoomsArgs` · `MembersArgs` · `RoomUpdateArgs` · `ThreadCreateArgs` · `ThreadStatusArgs`** are **client CLI argument structs** that the derive-scan did not catch as `clap`.
- **5 sites resolved to no struct at all** (`wire/types.rs` among them, plus `connection.rs`, `fanout.rs`, `envelope.rs` ×2) — almost certainly **enum variant fields**, which the heuristic cannot see.

⚠️ **Chat reported "30 boundary slots" to Joe before this inventory ran, and the real boundary set is larger. Corrected here rather than left standing** — ***a claim narrower than the thing it describes, reused as if complete***, is this project's named recurring species, and it appeared inside the very pass that measures it.

🔒 **⇒ LEG 0 IS NOT COMPLETE AT THIS DOCUMENT. It owes a HAND-VERIFIED classification of all 88**, each slot assigned INTERNAL / BOUNDARY / DESCRIPTIVE **by reading the struct and its consumers**, not by pattern. **No leg that changes a type may open before that read lands.**

---

## §4 — 🔒 SCOPE: ENFORCEMENT FIRST (Joe, 2026-08-02, *"go by your recommendations"*)

🔑 **THE ARGUMENT, AND IT IS `D-136`'s OWN THESIS TURNED ON THIS MILESTONE: a retype of 88 slots that ships without enforcement regresses again by construction.** The entry exists to say that a completed sweep is not a standing rule. **A milestone whose whole content is a second completed sweep would be the same mistake, one arc later, performed by the document that catalogues it.**

`D-136` §3 ranks enforcement, strongest first: **① make the wrong form unrepresentable · ② a test that fails on the wrong form · ③ a lint or grep gate · ④ a written standing rule in `CLAUDE.md`.**

| | ① unrepresentable | ② test | ③ grep gate | ④ written rule |
|---|---|---|---|---|
| **① user-visible** | none | none | none | none |
| **② cost** | very high — the boundary category must be expressible in the type system at every edge | moderate; the test must know which slots are which, so **it depends on §3's hand-read** | **low** — the §1 sweep IS the gate, already written | lowest |
| **catches a NEW file?** | yes | only if it enumerates | **yes** | only if read |

🔒 **CHAT RECOMMENDS ③ + ④ TOGETHER, AND ③ IS NEARLY FREE BECAUSE THIS DOCUMENT ALREADY BUILT IT.** The §1 sweep is a repeatable command; pointed at *post-arc slots not on the classified list*, it is exactly the check that would have caught `SeenRecord` in July. ⚠️ **④ alone is what `D-136` calls the weakest and most common choice — acceptable, but *"what is not fine is silence"*.**

🛑 **HONEST LIMIT, STATED RATHER THAN SOLD (the J-657 discipline): a grep gate catches a slot that is `String` and should not be. It CANNOT catch a slot that is correctly `String` at a boundary and wrongly consumed as internal state one function away.** That is the `SeenRecord` defect's actual shape, and **only the hand-read in §3 sees it.** ⇒ ***the gate clears the desk; it does not retire the read.***

---

## §5 — 🔒 M13 ORDERING (Chat, 2026-08-02, under `D-123` — a recommendation made at this document, not earlier)

`ops.rs:423-425` records that when `M13 Client Identity Lookup Widening` lands, its new fields become *field-mapping on top of* `FetchedIdentity` — **a struct in this milestone's core set.**

🔒 **THIS MILESTONE GOES FIRST.** Retype-then-extend ⇒ M13 writes its new fields already typed, and inherits the convention. Extend-then-retype ⇒ this milestone's retype touches fields M13 has just added, and M13 ships new `String` identifier slots in the meantime — **manufacturing the exact regression `D-136` describes, knowingly.**

📌 **Identical in shape to the Leg D argument in `M_RP_IDENTITY_RESOLUTION.md` §8**, and named here so the collision is not discovered at it.
---

## §6 — Legs

**Leg 0 — Phase-0.** This document **plus the hand-verified classification of all 88** (§3). **No code.** 🛑 **The classification is a DELIVERABLE of this leg, not a preamble to the next one.**

**Leg A — the enforcement mechanism.** ③ the grep gate + ④ the written rule in `CLAUDE.md`. **Moves no floor** (a script and a document). 🔑 **FIRST, DELIBERATELY** — so that every later leg lands under the rule instead of beside it.

**Leg B — the three address-book structs.** `FetchedIdentity` · `SeenRecord` · `FillReport`, plus the `BTreeMap<String, _>` key and the `ops.rs:2734`/`:2742` downgrade that disappears with them. **Moves the cargo floor.** 🔒 **THIS IS THE ONLY LEG ON `M-RP-IDENTITY-RESOLUTION` LEG D's CRITICAL PATH.**

**Leg C — the remainder**, sized by what Leg 0's hand-read actually finds. 🔓 **MAY BE SPLIT OFF INTO ITS OWN MILESTONE RATHER THAN BLOCKING** — see §7.

**Leg D — records + close.** JOURNAL + `CLAUDE.md` PLAY + ROADMAP + this document, one commit (`D-074`). 🛑 **Its close must state how the convention is enforced from then on, or state that it is not — `D-136` §3 binds this milestone to its own rule.**

---

## §7 — 🛑 A SEQUENCING ASSUMPTION OF CHAT'S OWN, CORRECTED HERE

At J-658 Chat sequenced this milestone **ahead of** `M-RP-IDENTITY-RESOLUTION` Leg D, on three reasons that remain sound — **but the unstated fourth premise was that it was SMALL.** The record said *"three structs"*. **It is 88 slots across 59 struct sites in four crates.**

🔑 ***A dependency priced from a record that described three structs is a dependency priced from a sample.*** 📌 *The same species as everything else in this arc, and the only reason it surfaced is that the sweep ran before the runbook rather than after it.*

🔒 **THE ORDERING SURVIVES, ON A NARROWER CLAIM: `M-RP-IDENTITY-RESOLUTION` Leg D is gated on THIS MILESTONE'S LEG B — the three address-book structs — NOT on the whole milestone.** Legs A and B are small and on the path; **Leg C is not, and must not be allowed to block a milestone it has no relationship to.**

> **`Owes:` — `M-RP-IDENTITY-RESOLUTION Leg D Tier-1 fetch on join` is unblocked by LEG B, not by close.** ⚠️ **If Leg 0's hand-read shows Leg C is large, Leg C splits into its own milestone and this one closes at B** — 🔓 **the split is Joe's to name (`D-123`), and the trigger is the hand-read, not a guess.**

---

## §8 — DoD

- [x] The `D-136` §4 sweep RUN, across all four crates, with its partition subject stated — ✅ **2026-08-02, ancestry against `7ed4e30`, 125 in-arc / 130 post-arc**
- [x] §2a's third category ruled — ✅ **boundary slots stay `String` under a named-projection test (Joe, 2026-08-02)**
- [x] §4's enforcement posture ruled — ✅ **③ + ④ (Joe, 2026-08-02)**
- [x] §5's M13 ordering ruled — ✅ **this milestone first (Chat, `D-123`)**
- [ ] **All 88 slots HAND-CLASSIFIED**, each assigned INTERNAL / BOUNDARY / DESCRIPTIVE **by reading the struct and its consumers** — **Leg 0**
- [ ] The 5 unresolved sites (`wire/types.rs`, `connection.rs`, `fanout.rs`, `envelope.rs`) **named** — the heuristic could not see them — **Leg 0**
- [ ] The grep gate exists, **runs, and FAILS on a planted `String` identifier slot** — 🛑 **exercised, not asserted; a gate that has never failed is not known to work** — **Leg A**
- [ ] The written rule lands in `CLAUDE.md` as a standing convention — **Leg A**
- [ ] `SeenRecord.home_node` is `NodeXgid` — 🔑 **it contradicts a Pass 4 borderline lock BY NAME** (§4.1.a: *"2 NodeXgid for `home_node` ×3"*) — **Leg B**
- [ ] The `ops.rs:2734`/`:2742` downgrade is **GONE, not moved** — **Leg B**
- [ ] cargo floor re-measured on every Rust leg, delta explained — **Legs B, C**
- [ ] Records in one commit (`D-074`), and **the close states the enforcement posture** (`D-136` §3) — **Leg D**

---

## §9 — Filed, NOT fixed

- 📌 **12 function-parameter slots are OUT OF SCOPE HERE.** They are signature surfaces — Pass 2 / 3 / 4 territory — and mixing them with struct fields makes a cargo delta unattributable between two different kinds of change.
- 🛑 **`xgen-core/src/wire/types.rs` HOLDS 8 POST-ARC SLOTS AND IS THE WIRE.** ⚠️ **A wire struct's `String` is not automatically a defect** — XGID newtypes are serde-transparent, so either form serialises identically — **but that also means retyping them is free on the wire and the question is purely internal.** **Named for the hand-read; not pre-judged here.**
- 📌 **`M-RP-LIVEFEED-REFRESH` §6-i reads `content.target_identity` as a raw `json!` convention**, not a typed field. **Adjacent to this milestone and not in it.**
- 📌 **`D-136` §3's option ① — make the wrong form unrepresentable — is NOT TAKEN and is not refused forever.** It is the only option that would end the class. **Re-openable once the boundary category has been read rather than hypothesised.**

---

## §10 — Handoff

🔒 **LOCKED:** §2a the boundary third category · §4 enforcement ③ + ④ · §5 M13 ordering · §7's narrowing — `M-RP-IDENTITY-RESOLUTION` Leg D is gated on **Leg B**, not on close.

🔓 **JOE'S, OPEN:** **whether Leg C splits into its own milestone**, and its name if it does — 🛑 **triggered by Leg 0's hand-read, deliberately not guessed at now.**

📌 **This milestone was filed at J-645 from a three-struct finding and opened at J-658. The sweep that justified it had never been run, and running it first is what showed the milestone was mis-sized — in both directions: wider than three structs, and narrower than 130 slots.**