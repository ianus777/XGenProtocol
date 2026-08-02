# M-RP-XGID-SLOT-RETYPE — the identifier slots that regressed to `String` after the retrofit arc closed
> **Status**: ACTIVE  
> Version: 1.4  
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

🛑 **⚠️ THIS SECTION'S HEADLINE IS FALSE AND IS CORRECTED AT §2b (v1.1). KEPT NOT ERASED (`D-131`).** The **third category already exists**, fully worked and locked, at **Pass 3 §4.3** and **Pass 4 §4.2 / §4.3** — documents this Phase-0 did not open. **The observation below is sound; the conclusion drawn from it is not.** 🔑 ***Written after reading Pass 4 §4.1.a and never opening Pass 3 §4.3 — a claim narrower than the thing it describes, in a document authored the same day it names that species.***

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

🛑 **⚠️ CORRECTED AT v1.1: THE RULE DOES REACH IT, BY NAME. See §2b.** *The sentence above is kept because the observation that produced it — that these are boundary slots with a working projection — turned out to be exactly right; only the inference that no rule covered them was wrong (`D-131`).*

⚠️ **AND THAT IS THE BETTER EXPLANATION FOR A TWO-MONTH SILENT REGRESSION THAN CARELESSNESS.** `D-136` §2's own second corroboration was *"`address_book.rs` contains ZERO `IdentityXgid` — the type was never in the room."* **A binary rule offered no bucket for a boundary slot, so a whole subsystem was written outside it without anyone choosing to.**

### §2b — 🛑 THE CORRECTION, AND IT IS A BETTER FINDING THAN THE ONE IT REPLACES: THE CATEGORY EXISTS AND ITS PROMOTION WAS PARKED WITH A TRIGGER NOBODY OWNED

**Read at v1.1 from `tasks/archive/`, which §2 had not opened:**

| where | what it locks |
|---|---|
| **Pass 3 §4.3** *Format-boundary preservation (wire OR persistence)* | *"function signatures that consume format-derived identifiers keep `String` / `&str` / `Option<String>` / `HashMap<String, _>` slots **as-is at the format boundary**. Conversion to typed XGIDs happens at the format/in-memory boundary (**one projection per direction**)."* Its affected-slot list names **"(general) any `TransportMessage::*` variant field carrying identifier-shaped strings"** |
| **Pass 4 §4.2** | The same rule **extended to client-side serialisation surfaces** — Tauri IPC, pipe JSON, stdout |
| **Pass 4 §4.3** *Option α* | **clap `Args` structs keep all 16 identifier-shaped `String` slots as `String` at the parse boundary; the dispatcher arm projects** via `Xgid::new(s) → XxxXgid::from_xgid(…)`. **`app.rs`'s `*Args` structs, by name** |

⇒ 🔒 **`admin_ops.rs`'s CLI args and result structs, and every `TransportMessage` variant field, are INSIDE an existing lock.** §2's *"a file the rule does not reach"* is wrong; **the rule reaches, and the file complies.**

🛑 **AND HERE IS THE ACTUAL DEFECT, WHICH IS SHARPER THAN A MISSING CATEGORY: THE RULE WAS NEVER PROMOTED TO A `D`, DELIBERATELY, AND THE TRIGGER THAT WOULD PROMOTE IT WAS LEFT WITHOUT AN OWNER.**

- Pass 3 §4.3 reasoning 3: *"**Flagged-not-promoted as candidate D-NNN-δ** at this design close (two instances…; **three-instance threshold opens at Pass 4** if a client-side serialisation-format slot instantiates)."*
- Pass 4 §4.2.3 / IMPL §12.4: **"`D-NNN-format-boundary` promotion-watch STAYS OPEN"** — three structurally-distinct instances across two Pass-arcs, held open because `D-077` multi-Pass-arc durability was not yet met. **Promotion trigger, written down: *"fourth structurally-distinct instance at Pass 5 OR cross-milestone closes the gap."***

🛑 **THE TRIGGER FIRED AND NOBODY RETURNED. `M6` — the node admin write-path (`admin_ops.rs`, closed J-197) — is a *cross-milestone, structurally-distinct* fourth instance: an admin named-pipe boundary.** ⚠️ *A trigger that has fired is a defect by this project's own standing convention, and this one has been fired and unnoticed since J-197.*

🔑 **SO THE MECHANISM IS `D-136`'s OWN THESIS AT A LEVEL `D-136` DID NOT REACH: not merely *a completed sweep is not a standing rule*, but ***a promotion-watch with a named trigger is worth nothing if nobody owns the trigger.*** The convention was invisible not because it was unwritten, but because it was written **in two archived design docs and deliberately kept out of `DECISIONS.md`.***

⚠️ **AND `D-136` §2 HAS THE SAME GAP.** It quotes Pass 4 §4.1.a's binary classification as *the* locked rule and **never cites Pass 3 §4.3**. 🔑 ***A claim narrower than the thing it describes — inside the entry that names that species.*** 📌 *Not a fault in `D-136`'s conclusion: `SeenRecord` fails the three-way rule exactly as it failed the two-way one (see below). The gap is in the rule as quoted, not in the verdict.*

✅ **`SeenRecord` STILL FAILS, AND NOW FOR A SHARPER REASON.** §4.3 puts the projection **at** the format/in-memory boundary, one per direction. **A `BTreeMap<String, SeenRecord>` sits on the IN-MEMORY side of that boundary** ⇒ it is internal state holding the external form, which is precisely what §4.3 forbids. **The defect that minted `D-136` is untouched by this correction.**

### §2a — 🔒 RULED 2026-08-02 (Joe, *"go by your recommendations"*): A BOUNDARY SLOT MAY STAY `String`, AND IT BECOMES A NAMED THIRD CATEGORY

- **① User-visible impact:** **NONE, under either answer.** This is internal type discipline; no user of XGen sees, feels or comes to believe anything different. 📌 *`D-121`'s legal answer, and it is the true one here — stated plainly rather than dressed in a manufactured UX rationale.*
- **② Resource cost:** *accepting* — near zero in code (the projection exists), plus the cost of **writing the category down**, which is the entire point. *Rejecting* — `FromStr` on eight flavour newtypes, every CLI arg struct retyped, and a deserialisation story for each.

🔒 **THE THIRD CATEGORY, AS IT ENTERS THE RECORD: a BOUNDARY slot holds the external form of an identifier at a parse or serialisation edge, and stays `String` IF AND ONLY IF a named projection converts it at the boundary and no internal state holds the `String` form.** 🛑 **The second half is what stops this becoming an excuse** — `SeenRecord` would fail it, because a `BTreeMap<String, SeenRecord>` *is* internal state holding the external form.

🔒 **RESTATED AT v1.1, AND THE RULING GETS CHEAPER AND STRONGER: THIS IS NOT A NEW CATEGORY — IT IS THE PROMOTION OF `D-NNN-format-boundary`, WHOSE OWN TRIGGER HAS ALREADY FIRED (§2b).** ✅ **Joe's wording and Pass 3 §4.3's are the same rule reached independently** — *one projection per direction, at the format/in-memory boundary* — which is corroboration, not coincidence. ⚠️ *The formulation above was Chat's reconstruction of a lock it had not read; the canonical wording is Pass 3 §4.3's and that is what should be promoted. Kept not erased (`D-131`).*

---

## §3 — 🛑 THE CLASSIFICATION OF THE 88 IS A HYPOTHESIS, NOT A MEASUREMENT

The four-way bucketing (**30 `clap`-boundary · 47 serde-wire · 41 internal · 12 fn-param**) came from a **heuristic**: nearest enclosing `struct` plus a backward scan for a `derive`. It is not a read.

🛑 **AND IT IS ALREADY KNOWN TO UNDERCOUNT ITS OWN LARGEST BUCKET.** The inventory pass showed the boundary category reaches further than the `clap` derive does:

- **~25 `admin_ops.rs` `*Result` structs** (`FederationAcceptResult`, `SpaceUnbanResult`, `IdentityRevokeResult`, …) are **serde outputs to the admin pipe** — the same species, on the way out rather than in.
- **`xgen-client/src/app.rs`'s `BanArgs` · `LeaveArgs` · `RedactArgs` · `RoomsArgs` · `MembersArgs` · `RoomUpdateArgs` · `ThreadCreateArgs` · `ThreadStatusArgs`** are **client CLI argument structs** that the derive-scan did not catch as `clap`.
- **5 sites resolved to no struct at all** (`wire/types.rs` among them, plus `connection.rs`, `fanout.rs`, `envelope.rs` ×2) — almost certainly **enum variant fields**, which the heuristic cannot see. 🛑 **⚠️ CORRECTED AT v1.1: IT IS 12 SLOTS, NOT 5** — `wire/types.rs` ×8 · `envelope.rs` ×2 · `connection.rs` ×1 · `fanout.rs` ×1. **Chat read FOUR FILE-GROUPS as FIVE SLOTS.** ✅ **The guess was right about the cause**: they are `TransportMessage` **enum variant fields**, invisible to a struct-scan by construction. 🔑 ***Third undercount by the same author inside one milestone — which is the argument for the hand-read, not against it.*** Kept not erased (`D-131`).

⚠️ **Chat reported "30 boundary slots" to Joe before this inventory ran, and the real boundary set is larger. Corrected here rather than left standing** — ***a claim narrower than the thing it describes, reused as if complete***, is this project's named recurring species, and it appeared inside the very pass that measures it.

🔒 **⇒ LEG 0 IS NOT COMPLETE AT THIS DOCUMENT. It owes a HAND-VERIFIED classification of all 88**, each slot assigned INTERNAL / BOUNDARY / DESCRIPTIVE **by reading the struct and its consumers**, not by pattern. **No leg that changes a type may open before that read lands.**

✅ **DISCHARGED AT v1.2 — §3a.**

---

## §3a — ✅ THE HAND-READ, 88 OF 88 (v1.2)

**Method: each slot judged against Pass 3 §4.3's actual test — is this the external form AT a format edge, with the projection one step away — not against the heuristic buckets of §3.** Sibling evidence used the way `D-136` used it: **does the same file, or the same struct, choose differently?**

### 🔒 BOUNDARY — stays `String` under an EXISTING lock · 65

| slots | where | lock |
|---|---|---|
| **30** | `admin_ops.rs` CLI args + `*Result` admin-pipe structs | Pass 4 §4.3 α + §4.2 |
| **16** | `app.rs` `*Args` (`BanArgs` · `LeaveArgs` · `RedactArgs` · `RoomsArgs` · `MembersArgs` · `RoomUpdateArgs` · `ThreadCreateArgs` · `ThreadStatusArgs`) | Pass 4 §4.3 α, **by name** |
| **8** | `wire/types.rs` — `TransportMessage` variant fields | Pass 3 §4.3, **by name** |
| **2** | `desktop.rs` `SelfStateInfo` | Pass 4 §4.2 — Tauri IPC |
| **1** | `batch.rs` `FrontierEvent.event_id` | Pass 4 §4.2 — pipe JSON |
| **3** | `protocol_audit.rs` `ProtocolAuditEntry` ×2 + `ProtocolAuditSink.node_id` | Pass 3 §4.3 — persistence |
| **2** | `audit.rs` `AuditEntry.actor` + `AuditQueryFilter.actor` | Pass 3 §4.3 — persistence (`xgen-node_audit.db`) |
| **1** | `fanout.rs:63` | wire |
| **1** | `connection.rs:114` (variant field) | wire |
| **1** | `trust_assertion.rs` `TrustAssertion.identity_id` | the **signed canonical form** — its own doc says *"reproduces the signed form"* |

### 📌 DESCRIPTIVE — not an XGID at all · 5

- **`envelope.rs` ×3** (`Command.id` `:57`, and `:144` / `:153`) — *"Driver-supplied correlation id, echoed verbatim into the reply."* **A correlation token, not an identifier of anything in the protocol.**
- **`audit.rs` `AuditEntry.correlation_id`** — same shape.
- 🔑 **`audit.rs` `AuditEntry.target`** — *"Verb-specific target (peer_node_id, identity_id, …)"*. **POLYMORPHIC: it holds a different flavour depending on the verb, so NO single flavour can type it.** ⚠️ **This is a fourth answer the rule does not currently name, and `admin_ops.rs` has the same shape at `:609` / `:574`.** *Filed, not invented into the rule here.*

### 🛑 INTERNAL — a real regression; retype · 17

| slots | struct | why it is internal |
|---|---|---|
| **2** | `address_book.rs` `SeenRecord` (`identity_id`, `home_node`) | the `D-136` case. `BTreeMap<String, SeenRecord>` is in-memory state holding the external form |
| **7** | `ops.rs` `FetchedIdentity` ×2 · `ThreadCreateResult` · `ThreadStatusResult` · `RedactResult` · `VerbReject` · `RoomsResult` | 🔑 **`ops.rs` carries 68 typed field declarations — including `identity_id: IdentityXgid` and `home_node: NodeXgid` at `:186/:188`, `:214/:217`, `:288/:290` — while `FetchedIdentity` at `:428/:432` declares THE SAME TWO FIELD NAMES as `String`.** Pass 4 §4.1.a's own scope |
| **3** | `resident.rs` `OutboundRequest` ×2 + `SendOutcome.event_id` | its own doc: *"A queued outbound message … the caller hands over intent"* — an **in-memory queue**, no format edge |
| **2** | `connection.rs` `AuthOutcome` | derives **`Debug, Clone, PartialEq, Eq` and NO serde** ⇒ not a format struct; it is the in-memory result consumed by `ops::create_space` |
| **1** | `session.rs` `SessionState.node_id` | in-memory session state |
| **2** | `state.rs` `ThreadState.id` + `.origin_event` | 🛑 **see below** |

🔑 **`ThreadState` IS A STRONGER INSTANCE THAN THE ONE THAT MINTED `D-136` — THAT WAS SAME-FILE; THIS IS SAME-STRUCT:**

```rust
pub id: String,               // "Conceptual Thread id (xgen://thread/sha256:)"  <- ThreadXgid EXISTS
pub room_id: RoomXgid,        // typed
pub created_by: IdentityXgid, // typed
pub origin_event: String,     // "The thread.create event id"                    <- EventXgid EXISTS
```

**Four identifier slots in one struct; two typed, two `String`; both missing flavours exist; and the struct derives no serde at all** (`Debug, Clone, PartialEq, Eq`) ⇒ **pure in-memory Space state, not a format boundary.** ⚠️ *`state.rs` mentions `Xgid` 187 times — the type was emphatically in the room.*

### ⚠️ UNREAD · 1 — named rather than guessed

- **`envelope.rs` `ErrorBody.event_id`** (`:132`) — sits among correlation ids but is named `event_id`. **Whether it carries a real `EventXgid` or another correlation token needs its producer read. NOT classified here.**

**65 + 5 + 17 + 1 = 88.** ✅ **Reconciled against the sweep total.**

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

🔒 **④ IS RESTATED AT v1.1 AND IT IS NO LONGER "WRITE A NEW RULE": IT IS *PROMOTE `D-NNN-format-boundary` TO A REAL `D`* (§2b).** 🔑 **That is strictly better than a fresh `CLAUDE.md` convention, because the rule is already worked, already worded, already has its instance table, and already has a fired trigger** — what it lacks is a home outside `tasks/archive/`. ⚠️ **And it repairs the cause rather than the symptom: the convention was invisible because it was deliberately kept out of `DECISIONS.md`, not because nobody had written it.**

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

✅ **CLOSED J-661.** `D-137` promoted to `DECISIONS.md` · `xgid-slot-gate.ps1` + `xgid-slot-manifest.tsv` at the repo root · `CLAUDE.md` **Rule 0 gains item (5)**, which is where the trigger finally gets an owner.

🔒 **④ turned out not to be *"write a new rule"* but *promote `D-NNN-format-boundary`*, whose own trigger had fired at `M6` and gone unread for two months.** 🔑 ***A promotion-watch with a named trigger is worth nothing if nobody owns the trigger*** ⇒ a watch needs **(a) a trigger, (b) an owner, (c) a place the owner will actually read**.

✅ **THE GATE WAS EXERCISED, NOT ASSERTED:** clean tree **PASS** (88 — 65/5/17/1) · planted slot **FAIL** on the dirty guard · planted slot with `-AllowDirty` **FAIL** naming `SessionState.gate_test_target_id` · revert in `finally` · post-revert **PASS**.

🛑 **AND THE GATE'S OWN FIRST DEFECT WAS FOUND THE HARD WAY: IT SWEPT THE FILESYSTEM.** A planted slot in a **tracked** file was counted as production and moved the sweep **255 → 256**. Two vectors, and **the tempting one-line fix closes only one**: untracked files die to `git ls-files`; **modified tracked files still read dirty from disk** and are now refused outright unless `-AllowDirty` prints its warning. ⚠️ *`ls-files` alone would NOT have caught the actual incident.*

**Leg B — the four address-book slots.** `SeenRecord` ×2 + `FetchedIdentity` ×2, plus the `BTreeMap<String, _>` key and the sites that disappear with them. **Moves the cargo floor.** 🔒 **THIS IS THE ONLY LEG ON `M-RP-IDENTITY-RESOLUTION` LEG D's CRITICAL PATH.**

🔒 **RUNBOOK AUTHORED AND LOCKED 2026-08-02 (J-664): `tasks/RUNBOOK_XGID_SLOT_RETYPE_LEG_B.md` v1.1 ACTIVE.** Locked as authored, no corrections raised. **Clair implements from it; standing her up is Joe's.**

📌 **RENAMED AT LOCK — *"the three address-book structs"* is superseded, kept not erased (`D-131`).** The old name counts **structs** and only **two of the three** carry work (`FillReport` contributes zero — already `Vec<IdentityXgid>`). **The unit that is true of this leg is the SLOT, and there are four.**

🛑 **AND THE SITE COUNT WAS WRONG IN EVERY RECORD THAT NAMED IT — THERE ARE THREE, NOT TWO.** Beside the two downgrades (`ops.rs:2734` / `:2742`) there is a **compensating re-wrap at `ops.rs:2935`** — `report.not_found_ids.push(IdentityXgid::from_xgid(Xgid::new(id.clone())))` — whose own comment names this milestone and predicts its own removal. 🔑 **All three vanish together or none do:** the downgrades exist only to feed the `String`-keyed map, and the re-wrap exists only to undo them. ⚠️ ***A claim narrower than the thing it describes*** — the named recurring species, this time in a session kickoff, and it survived because every reader re-read the record instead of opening `fill_from_events`.

**Leg C — the remainder**, sized by what Leg 0's hand-read actually finds. 🔓 **MAY BE SPLIT OFF INTO ITS OWN MILESTONE RATHER THAN BLOCKING** — see §7.

✅ **SIZED AT v1.2 BY THE §3a HAND-READ, AND IT IS SMALL: 13 SLOTS IN 5 FILES.** 🛑 *A first draft of this line read "15 slots… Leg B takes the other 2" — **the total reconciled to 17 and the ATTRIBUTION was wrong**, because Leg B's `FetchedIdentity` IS 2 of the 7 `ops.rs` slots. Caught on re-derivation, corrected, kept not erased (`D-131`): ***a sum that reconciles is not a split that is right.***

**INTERNAL · 17 total, split by leg:**

| leg | slots | where |
|---|---|---|
| **Leg B** | **4** | `address_book.rs` `SeenRecord` ×2 + `ops.rs` `FetchedIdentity` ×2 |
| **Leg C** | **13** | `ops.rs` ×5 (`ThreadCreateResult` · `ThreadStatusResult` · `RedactResult` · `VerbReject` · `RoomsResult`) · `resident.rs` ×3 · `connection.rs` ×2 · `state.rs` ×2 · `session.rs` ×1 |

📌 **And `FillReport` contributes ZERO slots** — its identifier slot is already `Vec<IdentityXgid>` from `M-RP-IDENTITY-RESOLUTION` Leg A. **Leg B's name says three structs and only two of them carry work.**

🔑 **⇒ CHAT RECOMMENDS LEG C IS *NOT* SPLIT OFF.** The split existed to stop a large unknown blocking `M-RP-IDENTITY-RESOLUTION` Leg D; **the unknown is now measured and it is 13 slots**, five of which sit in `ops.rs` — **the same file Leg B already opens.** ⚠️ **Splitting would create a second milestone to carry 13 mechanical retypes and a second cargo-floor move, and would make Leg B and Leg C edit one file from two milestones** — the rider-versus-milestone judgement running the other way. 🔓 **Still Joe's.**

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
- [ ] **All 88 slots HAND-CLASSIFIED**, each assigned INTERNAL / BOUNDARY / DESCRIPTIVE **by reading the struct and its consumers** — **Leg 0** — ✅ **DONE at v1.2, §3a: 65 BOUNDARY · 5 DESCRIPTIVE · 17 INTERNAL · 1 UNREAD-and-named; reconciled to 88**
- [ ] **`ErrorBody.event_id` read to a verdict** — the one slot §3a refused to classify — **Leg 0**
- [ ] **The POLYMORPHIC `target` shape ruled** — `AuditEntry.target` and `admin_ops.rs:574`/`:609` hold a different flavour per verb, so no single flavour can type them. 🔓 **A fourth answer the rule does not name; Joe's** — **Leg A** ⚠️ *superseded by the ticked item above; kept not erased (`D-131`)*
- [ ] The **12** unresolved sites (`wire/types.rs` ×8 · `envelope.rs` ×2 · `connection.rs` · `fanout.rs`) **named** — ✅ **DONE at v1.1: they are `TransportMessage` enum variant fields, covered by Pass 3 §4.3 by name.** ⚠️ *the DoD read "5" at v1.0; superseded, kept not erased (`D-131`)* — **Leg 0**
- [x] **`D-NNN-format-boundary` PROMOTED to a real `D`**, with its fired trigger recorded — ✅ **`D-137`, J-661** — **Leg A**
- [x] The grep gate exists, **runs, and FAILS on a planted `String` identifier slot** — ✅ **J-661: PASS clean · FAIL on the dirty guard · FAIL naming the planted slot · PASS after revert** — **Leg A**
- [x] The written rule lands in `CLAUDE.md` as a standing convention — ✅ **Rule 0 item (5), J-661** — **Leg A**
- [x] **The POLYMORPHIC `target` shape ruled** — ✅ **`D-137` §1: a second independent reason a BOUNDARY slot stays `String`, NOT a fourth category** — **Leg A**
- [ ] `SeenRecord.home_node` is `NodeXgid` — 🔑 **it contradicts a Pass 4 borderline lock BY NAME** (§4.1.a: *"2 NodeXgid for `home_node` ×3"*) — **Leg B**
- [ ] The **THREE** `String`↔typed bridge sites are **GONE, not moved** — `ops.rs:2734` · `:2742` · **`:2935` with its comment** — **Leg B** ⚠️ *this item read "the `ops.rs:2734`/`:2742` downgrade" until v1.4; the third site was found at runbook authoring. Superseded, kept not erased (`D-131`).*
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