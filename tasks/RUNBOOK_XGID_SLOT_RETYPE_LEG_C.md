# M-RP-XGID-SLOT-RETYPE Leg C — the remainder: ten slots in six files, across two crates
> **Status**: COMPLETED  
> Version: 1.3  
> Date: Aug 2026  
> **Last updated**: 2026-08-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

✅ **LEG C IS IMPLEMENTED, RE-DRIVEN AND CLOSED (J-669). Commit `b99e202` [Clair, 13 files] + the doc bridge [Chat]. Every number below was re-measured by Chat under Rule 5; not one entered the record on report.**

📌 **ONE CORRECTION TO THE v1.2 AMENDMENT AND TO CLAIR'S OWN HAND-BACK: THE AICONTROL PROJECTION SITS AT `aicontrol.rs:575`, NOT `:572`.** §0 said `:572` because that is where it sat **before** she wrote the explanatory comment above it. **The record should carry the line that is actually there.** *Cosmetic, and recorded anyway — a pointer that is three lines out is how the `:122` error started.*

🔒 **LOCKED BY JOE 2026-08-02 (*"locked"*), AS AUTHORED — no corrections raised (J-668). Chat wrote it; Clair implements from THIS version and does not close her own leg (D-123).**

---

## §0 — 🛑 v1.2 AMENDMENT: R4 FIRED, THE RULING IS TAKEN, AND FOUR OF THIS RUNBOOK'S OWN LINES WERE WRONG

🔒 **R4 FIRED AND CLAIR STOPPED. THAT IS THE RULE WORKING, NOT THE LEG FAILING.** She retyped all ten slots, ran `cargo check --workspace --all-targets`, and the **compiler found three consumers outside `xgen-client` that no grep could see** — all three chain `.identity_id` straight off `client_authenticate(...)` under different local names, so nothing matching `AuthOutcome` or `auth.identity_id` ever appeared:

| crate | site | wants |
|---|---|---|
| `xgen-mptest` | `injector.rs:275` → `authed_as: Option<String>` (`:90`) | a `String` |
| `xgen-mptest` | `wireactor.rs:63` → `identity_id: String` (`:38`) | a `String` |
| `xgen-node` | `transport/mod.rs:57` — a `#[cfg(test)]` `assert_eq!` | a `String` |

✅ **`AuthOutcome.node_id` is CLEAN** — no cross-crate reader. **The entire leak is `identity_id`.**

### 🔒 THE RULING — (A) IN ITS MINIMAL FORM. CHAT'S, UNDER `D-123`, REVERSIBLE ON ONE WORD.

**Joe, restating the seat boundary rather than answering: *"mine are appearance and architecture; yours are technicalities and anything else."*** ⚠️ *Routing this to him was UNDER-STEPPING — the recurring seat error, second instance this arc.*

🔒 **FIX THE THREE SITES BY PROJECTION AT THE CONSUMER, CHANGING NO TYPE IN EITHER CRATE:** `….identity_id.as_str().to_string()`.

- 🔑 **That is `D-137`'s own shape, not an exception to it** — a consumer that wants the external form takes it through a **named projection at its own edge**. **Nothing in `xgen-node` or `xgen-mptest` changes type; three call-site lines change.**
- 🔑 **All three consumers are TEST INFRASTRUCTURE.** `xgen-mptest` is a test crate and the `xgen-node` site is `#[cfg(test)]`. **No production code in either crate reads this field.**
- 🛑 **(B) — dropping `AuthOutcome.identity_id` from the leg — IS REFUSED, and the reason is the milestone's own thesis:** it would leave a slot the hand-read classified INTERNAL as `String` **because a test harness wanted a `String`**, and *a slot does not become BOUNDARY by sitting near something that wants the external form* (`D-137` §1 clause 3). **That is the exact defect this milestone exists to remove, shipped knowingly, in the crate with the widest reach.**

⚠️ **AND THE HONEST COST, STATED RATHER THAN SOLD: V6 GETS WEAKER.** It was *"zero `xgen-node` / `xgen-mptest` / `xgen-common`"* — an absolute, which is what made it checkable. It now has to **name the three lines**, or it stops being a check. **See the amended V6.** *R4's letter says **signature** change and a one-line projection is not one — but V6 was written as an absolute, and stopping on it was right.*

### 🛑 FOUR LINES OF THIS RUNBOOK WERE WRONG. ALL FOUR ARE CHAT'S; ALL FOUR ARE CORRECTED BELOW AND KEPT NOT ERASED (`D-131`).

1. **§8.3 SAID THE COMPILER IS THE AUTHORITY AND IT WAS RIGHT.** §2/§§3 recorded *"`AuthOutcome` 3 refs, one file"* from `grep`. **The compiler found three more, in two gated crates.** 🔑 ***A reference count is only as wide as the name you searched for — and a value chained straight off its producer carries no name at all.***
2. **THE AICONTROL PROJECTION IS `aicontrol.rs:572`, NOT `:122`** (§2c, C-1, R2, V5). `:122` is a `String → String` move into `ErrorBody`; the real `EventXgid → String` seam is `event_id: vr.event_id.clone()` at **`:572`**. **Third line in this milestone where a runbook of mine pointed at the wrong place.**
3. **BOTH `node_id` SLOTS ARE `Option<String>`** ⇒ the target is **`Option<NodeXgid>`**, not `NodeXgid`. §1b's table wrote the bare flavour. `SessionState.node_id` is assigned from `auth.node_id` at `session.rs:165`, so **the two retype together**.
4. **§1b/V7's FILE LIST WAS INCOMPLETE.** `OutboundRequest` is **constructed at `desktop.rs:354`** from the Tauri command's `String` params, and `app.rs:3391` reads `short_id(&r.target_event_id)`. Both are **`xgen-client`**, so R4 never applied — **forced consequences of a locked retype, correctly absorbed rather than refused** (the J-665 pattern). `ai_service.rs:258` joins them under this amendment.

### ✅ AND WHAT SHE GOT RIGHT THAT THIS RUNBOOK ASKED FOR

**§8.2 told her to check my two `EventXgid` guesses at their WRITERS rather than trusting the names. She did, and all three hold:** `RedactResult.target_event_id` ← `RedactArgs.target` · `ThreadState.origin_event` ← `event.event_id`, already bound as `event_xgid` at `state.rs:916` (**assigned directly, no projection needed**) · `VerbReject`/`SendOutcome.event_id` ← `EventConfirm::Rejected.event_id`, which is BOUNDARY-wire and **stays `String`**, projecting at each writer.

### The five sites this amendment adds to C-1

| # | site | edit |
|---|---|---|
| 1 | `xgen-mptest/src/injector.rs:275` | `.as_str().to_string()` |
| 2 | `xgen-mptest/src/wireactor.rs:63` | `.as_str().to_string()` |
| 3 | `xgen-node/src/transport/mod.rs:57` | `.as_str()` in the `assert_eq!` |
| 4 | `xgen-client/src/ai_service.rs:258` | project into `SessionContext.identity_id: Option<String>` |
| 5 | `xgen-client/src/ops.rs:3201` | `assert_eq!(r.space_id.as_str(), "…")` |

🛑 **NOTHING ELSE IN `xgen-node` OR `xgen-mptest` MAY BE TOUCHED.** If a fourth site appears there, **R4 fires again and the leg stops again** — the ruling covers the three sites the compiler named, not the crates.

---

🔒 **PARENT:** `tasks/M_RP_XGID_SLOT_RETYPE.md` §3a (the 88-slot hand-read) · §6 (Leg C) · §9 (out of scope) · `DECISIONS.md` **D-136** and **D-137**. **Predecessor:** `tasks/RUNBOOK_XGID_SLOT_RETYPE_LEG_B.md` v1.2 COMPLETED (J-665) — read its §2 and §9 before this one; every trap it names is live here and two of them are worse.

🔒 **JOE RULED, 2026-08-02:** the milestone **runs to close** — *"we will complete whole M-RP-XGID-SLOT-RETYPE if we can"* — so **Leg C does NOT split into its own milestone** (Phase-0 §6/§7's open fork, closed). And on this runbook's two scope questions, *"both by your recomms"*: **the three thread slots are filed separately, not held in this leg**, and **C-1 and C-2 share one runbook**.

---

## §1 — Scope: TEN slots, not thirteen

### §1a — 🛑 THE THREE THREAD SLOTS ARE OUT, BECAUSE THE FLAVOUR THEY WOULD RETYPE INTO DOES NOT EXIST

**Phase-0 §3a asserts *"`ThreadXgid` and `EventXgid` both exist"*. Measured: there are exactly SEVEN flavours** — `EventXgid · SpaceXgid · RoomXgid · TrustAssertionXgid · NodeXgid · IdentityXgid · AuthModuleXgid` (`xgen-common/src/xgid/flavours.rs`, `declare_flavour!` call sites). **There is no `ThreadXgid`.**

🔑 **AND ITS ABSENCE IS A DECISION, NOT AN OMISSION — the code says so in three places:** *"conceptual, no `ThreadXgid`"* (`xgen-common/src/wire.rs:122`) · *"AE-D8, no `ThreadXgid`"* (`xgen-core/src/space/state.rs:154`) · *"A Thread has no `ThreadXgid` flavour — the id is a `xgen://thread/sha256:`"* (`state.rs:1423`).

⇒ **`ThreadState.id`, `ThreadCreateResult.thread_id` and `ThreadStatusResult.thread_id` have nothing to retype into.** ⚠️ *Phase-0 §3a's claim is wrong and is annotated, not erased (`D-131`) — the fourth Chat claim in this milestone narrower than the thing it describes.*

🔓 **FILED, JOE'S, AND NOT THIS LEG:** mint `ThreadXgid` against a documented refusal, or rule the three slots DESCRIPTIVE. **Minting a flavour is a protocol-shaped decision and must not ride a mechanical retype leg.**

🔒 **THE THREE MANIFEST ROWS STAY, AND THAT IS THE MECHANISM, NOT AN OVERSIGHT.** They remain `INTERNAL` in `xgid-slot-manifest.tsv`, so **every session's Rule-0 gate run keeps them on screen.** *`D-137` §2 requires a watch to have (a) a trigger, (b) an owner, (c) a place the owner will actually read — and the gate is the only artifact in this project re-read every single session.* **A filed line in a task doc is exactly the parking spot that failed for two months.**

### §1b — The ten, and what each becomes

| # | file | struct | field | → |
|---|---|---|---|---|
| 1 | `xgen-client/src/ops.rs` | `RedactResult` | `target_event_id` | `EventXgid` |
| 2 | `xgen-client/src/ops.rs` | `RoomsResult` | `space_id` | `SpaceXgid` |
| 3 | `xgen-client/src/ops.rs` | `VerbReject` | `event_id` | `EventXgid` |
| 4 | `xgen-client/src/resident.rs` | `OutboundRequest` | `space_id` | `SpaceXgid` |
| 5 | `xgen-client/src/resident.rs` | `OutboundRequest` | `room_id` | `RoomXgid` |
| 6 | `xgen-client/src/resident.rs` | `SendOutcome` | `event_id` | `Option<EventXgid>` |
| 7 | `xgen-client/src/session.rs` | `SessionState` | `node_id` | `NodeXgid` |
| 8 | `xgen-core/src/transport/connection.rs` | `AuthOutcome` | `identity_id` | `IdentityXgid` |
| 9 | `xgen-core/src/transport/connection.rs` | `AuthOutcome` | `node_id` | `NodeXgid` |
| 10 | `xgen-core/src/space/state.rs` | `ThreadState` | `origin_event` | `EventXgid` |

🛑 **THIS LEG LEAVES `xgen-client`. Leg B never did.** Slots 8–10 are **`xgen-core`**, which every other crate depends on — so a signature that leaks past the struct reaches `xgen-node` and `xgen-mptest` too. **§3 R4 exists for that.**

---

## §2 — Grounding, measured 2026-08-02 at `36e7a11` (HEAD = origin/main, tree clean, gate PASS 84)

### §2a — ✅ SEVEN OF THE TEN ARE THE SAME-STRUCT SHAPE THAT MINTED `D-136`, AND IT IS DECISIVE

`D-136`'s instance was *same-file*; J-660 found *same-struct*; **these are same-struct with the sibling typed on the line above:**

```rust
pub struct RedactResult {
    pub event_id: EventXgid,        // typed
    pub target_event_id: String,    // <- the regression
    pub space_id: SpaceXgid,        // typed
    pub room_id: RoomXgid,          // typed
}
```

🔑 **THREE OF FOUR IDENTIFIER SLOTS TYPED AND ONE NOT IS NOT A BOUNDARY STRUCT.** If these were format structs under `D-137` clause 3, **every** identifier slot in them would be `String`. That single observation settles the classification for `RedactResult`, `RoomsResult`, `ThreadCreateResult`, `ThreadStatusResult` without appeal to a rule — *the file already voted.*

### §2b — ✅ `RedactResult` AND `RoomsResult` NEVER REACH THE PIPE — the boundary worry is measured away, not argued away

Both derive `Serialize, Deserialize` and both appear in `batch.rs`, which is a **Pass 4 §4.2 pipe-JSON boundary**. ⚠️ **That is the shape that would make them BOUNDARY.** It does not, because `batch.rs` **discards them**: *"the protocol only needs OK/ERROR — the `RoomsResult` data is discarded"* (`batch.rs:382`) and *"the `RedactResult` is discarded here"* (`:584`).

⇒ **their only serialisation is the CLI stdout path**, and 🔑 **under `#[serde(transparent)]` the stdout bytes are unchanged anyway.** *Checked at the consumer rather than inferred from the derive — a struct is not at a boundary because it can be serialised; it is at one because something serialises it.*

### §2c — ✅ `VerbReject` IS PURE IN-MEMORY, AND ITS PROJECTION SITE ALREADY EXISTS

`VerbReject` derives **`Debug, Clone` and no serde at all** ⇒ no format edge, INTERNAL beyond argument. Its `event_id` flows into the **aicontrol envelope** at `aicontrol.rs:88` (`event_id: String`), and **that** is a Pass 4 §4.2 boundary.

🔒 **SO THE ENVELOPE FIELD STAYS `String` AND THE PROJECTION LANDS AT ITS CONSTRUCTION** (`aicontrol.rs:122`, `event_id: Some(event_id)` → `Some(event_id.as_str().to_string())`). **One projection, at the edge, in the direction that crosses it — `D-137`'s canonical wording, applied.**

### §2d — 🔑 `ThreadState.origin_event` IS WRITE-ONLY

Declared `state.rs:176`, assigned `state.rs:931`, and **read nowhere in the repo** — those two lines are every occurrence. ⇒ **the retype is two lines with zero consumers.**

📌 **AND IT RAISES A QUESTION THIS LEG DOES NOT ANSWER: a field nothing reads is a field nobody has round-tripped** (the M-RP6.1k *reserve nothing* finding, in Rust). **Filed in §8. Not deleted here** — removing a field from a `xgen-core` Space-state struct is a different kind of change from retyping one, and mixing them makes the cargo delta unattributable.

### §2e — 🛑 `SendOutcome` IS THE ONE SLOT THAT CROSSES TO THE WEBVIEW, AND IT HAS A TS MIRROR

`desktop.rs:309` returns `crate::resident::SendOutcome` across the Tauri boundary, and the frontend declares it **verbatim**: `ui/common/lib/stores/echo-state.svelte.ts:37`, *"The Rust `SendOutcome` (resident.rs:733) carried VERBATIM — snake_case, no mapping layer."* **28 references across `desktop.rs` + `resident.rs`** — the largest consumer set in the leg.

🔑 **Under `#[serde(transparent)]` `Option<EventXgid>` serialises exactly as `Option<String>` did**, so the TS mirror needs no change and `svelte-check` must not move. 🛑 **THAT IS AN ARGUMENT UNTIL IT IS A TEST — the Leg B §2c lesson, and this is the same claim one shape further out**, because `Option<T>` adds a `None`/`null` case Leg B never exercised. **V3 breaks it deliberately.**

---

## §3 — Executable constraints

- **R1 — NO NEW `String` IDENTIFIER SLOT ANYWHERE.** The gate is the check; run it, do not reason about it.
- **R2 — THE AICONTROL ENVELOPE FIELD STAYS `String`** (`aicontrol.rs:88`). Retyping it is a boundary breach, not a completion. **One projection at its construction, and no other.**
- **R3 — THE THREE THREAD SLOTS ARE NOT TOUCHED, AND THEIR MANIFEST ROWS ARE NOT DROPPED.** If a retype of them looks tempting mid-leg, that is §1a firing, not an opportunity.
- **R4 — 🛑 THE `xgen-core` RETYPES MUST NOT LEAK PAST THEIR STRUCTS.** `AuthOutcome` and `ThreadState` live in a crate every other crate depends on. **If a signature in `xgen-node`, `xgen-mptest` or `xgen-common` has to change to make this compile, STOP AND REPORT** — that is a scope finding, not a chore. (`AuthOutcome` measured at **3 refs, single file**; `ThreadState` at **8 refs across 4 files**, but only the write-only `origin_event` is in scope.)
- **R5 — NO `ui/**` FILE IS TOUCHED.** If one appears to need touching, §2e is false — stop and report.
- **R6 — `SendOutcome`'s FOUR-WAY HONESTY IS NOT TOUCHED.** `status` / `code` / `reason` are DESCRIPTIVE and stay. The `None` arm (`SendOutcome::failed`) must still produce `event_id: null` on the wire, **not** an empty-string `EventXgid` — ⚠️ *`Xgid` derives `Default`, so `EventXgid::default()` is an empty-but-present id, which would be a false claim that a failed send has an event.*
- **R7 — THE MANIFEST TRAVELS IN THE SAME COMMIT AS THE CODE** (`D-074` applied to the gate). See C-5.

---

## §4 — The steps

### C-0 — Re-measure the floor before the first edit

`cargo test` **DETACHED**, polled in short calls; apps **down** (a running client holds `xgen-client.exe`). Expected **1592 / 0 / 62 across 56 terminators**. 🛑 **If it differs, STOP and report** — Leg B's baseline is the control for this leg's delta.

### C-1 — The seven straightforward retypes

`RedactResult.target_event_id` · `RoomsResult.space_id` · `VerbReject.event_id` · `OutboundRequest.space_id`/`.room_id` · `SessionState.node_id` · `ThreadState.origin_event`.

- Projections land **where an external `String` becomes internal**, one per direction. For `RoomsResult.space_id` that is the read of `crate::app::RoomsArgs` (a Pass 4 §4.3 α clap boundary — **the arg stays `String`**).
- `VerbReject` → the aicontrol projection at `aicontrol.rs:122` (R2).
- `ThreadState.origin_event` → two lines (`:176` declaration, `:931` assignment); **no consumers** (§2d).
- Imports: `xgen_common::xgid::{EventXgid, SpaceXgid, RoomXgid, NodeXgid, Xgid}` per file, as needed.

### C-2 — `AuthOutcome` (`xgen-core`)

`identity_id → IdentityXgid`, `node_id → NodeXgid`. 3 refs, one file. **R4 applies: if the change escapes `connection.rs`, report it.**

### C-3 — `SendOutcome.event_id → Option<EventXgid>`

The boundary-crossing slot. **`SendOutcome::failed` keeps `event_id: None`** (R6); `from_confirm` wraps the confirmed id. **Do not touch `status`/`code`/`reason`.**

### C-4 — The tests that move the floor

🔑 **These exist because §2e is an argument, and Leg B's V3 is the precedent that turning it into a test is worth the three tests it costs.**

- **V3-a — `SendOutcome` serialises identically, BOTH arms.** Assert the **exact JSON literal** for a confirmed outcome (`"event_id":"xgen://hash/sha256:…"`) **and** for a failed one (**`"event_id":null`**). 🛑 **The `null` arm is the one that matters** — it is where `Default` would silently substitute an empty string.
- **V3-b — a `RedactResult` / `RoomsResult` round-trip**, proving the CLI stdout shape is byte-unchanged.
- **V3-c — `AuthOutcome` equality still holds** (it derives `PartialEq, Eq`; the retype must not disturb the comparison `ops::create_space` relies on).

📌 **Three tests ⇒ 1592 → 1595. MEASURE IT; DO NOT PREDICT IT.**

### C-5 — Manifest + gate, same commit

1. Drop the **ten** retyped rows from `xgid-slot-manifest.tsv`. **KEEP the three thread rows** (R3, §1a).
2. Header → `# EXPECTED: BOUNDARY=65 DESCRIPTIVE=5 INTERNAL=3 UNREAD=1 TOTAL=74`.
3. Re-run `.\xgid-slot-gate.ps1` on a **clean tree**. ✅ Required: **PASS at 74 (65 / 5 / 3 / 1), no WARN.**
4. 🛑 Dropping rows without moving the header is a **FAIL** on the gate's own tally check (`xgid-slot-gate.ps1:114`), not a warning. The gate refuses a dirty tree — run it staged and clean.

---

## §5 — Verification

| # | check | expected |
|---|---|---|
| **V0** | floor before the first edit | **1592/0/62 × 56** — reconcile or stop |
| **V1** | `cargo test` after C-4, detached, apps down | **1595/0/62**, delta = the three V3 tests, **enumerated not derived** |
| **V2** | `svelte-check` | **0/34/15 UNCHANGED** — §2e proven at the TS edge |
| **V3** | the three new tests | pass; **V3-a asserts the `null` arm explicitly** |
| **V4** | `.\xgid-slot-gate.ps1`, clean tree | **PASS at 74** (65 / 5 / 3 / 1), no WARN |
| **V5** | R2 by read, amended at v1.2 | `aicontrol.rs:88` **and** `ErrorBody.event_id` still `String`; exactly ONE projection, at **`:572`** — **not `:122`**, which is a `String → String` move |
| **V6** | R4, amended at v1.2 (§0) | **EXACTLY THREE lines outside `xgen-client`/`xgen-core`, and they are named:** `xgen-mptest/src/injector.rs:275` · `xgen-mptest/src/wireactor.rs:63` · `xgen-node/src/transport/mod.rs:57`. **No TYPE changes in either crate.** A fourth site ⇒ **STOP** |
| **V7** | scope by diffstat, amended at v1.2 | `ops.rs` · `resident.rs` · `session.rs` · `aicontrol.rs` · `desktop.rs` · `app.rs` · `ai_service.rs` · `connection.rs` · `state.rs` · the three V6 lines · `xgid-slot-manifest.tsv` — **and nothing else**; zero `ui/**` |
| **V8** | sampler catalogue | **435 UNCHANGED, by scope** |

---

## §6 — DoD

- [x] **C-0 baseline reconciled to 1592/0/62 × 56** — ✅ exact, 56 terminators, final `test result:` present
- [x] All **ten** slots typed per §1b — ✅ with `Option<NodeXgid>` on both `node_id` slots (§0 correction 3)
- [x] **The three thread slots UNTOUCHED and their manifest rows RETAINED** — ✅ re-read: the three remaining `INTERNAL` rows are **exactly** `ThreadCreateResult.thread_id` · `ThreadStatusResult.thread_id` · `ThreadState.id`
- [x] `aicontrol.rs:88` still `String`, one projection — ✅ at **`:575`**; `ErrorBody.event_id` untouched
- [x] **`SendOutcome::failed` still emits `event_id: null`** — ✅ **asserted on an exact JSON literal, both arms**; the test can fail
- [x] **No `xgen-common` file changed; `xgen-node`/`xgen-mptest` limited to the three ruled lines** — ✅ all three are consumer projections, **zero type changes** (§0 ruling (A) minimal)
- [x] No `ui/**` file changed; `svelte-check` **0/34/15** — ✅ re-run by Chat
- [x] **cargo floor re-measured and the delta explained by name** — ✅ **1595/0/62 × 56**, Δ **+3** = `redact_result_round_trips_byte_identically` · `send_outcome_serialises_event_id_identically_both_arms` · `auth_outcome_equality_holds_after_retype`, each read `... ok`
- [x] **Manifest rows dropped AND header moved to `INTERNAL=3 TOTAL=74`, same commit** — ✅ tally re-counted 65/5/3/1 = 74
- [x] **Gate: PASS at 74, no WARN** — ✅
- [x] Scope confirmed by diffstat — ✅ 13 files; zero `ui/**`, zero `xgen-common`

🛑 **HONEST LIMIT, CARRIED FORWARD RATHER THAN QUIETLY DROPPED: THIS LEG IS COMPILE- AND TEST-VERIFIED ONLY.** No CDP, no live client, nothing exercised against a running system. Transparency means the wire / disk / Tauri-IPC bytes are unchanged and V2 + V3-a/-b pin that — **but a type discipline leg cannot claim behaviour it never ran.**

🛑 **NOT IN THIS DoD:** Leg D (records + close) · the `ThreadXgid` question (§8) · `envelope.rs` `ErrorBody.event_id`, the milestone's one UNREAD slot · `M-RP-IDENTITY-RESOLUTION` Leg D/E/F · `M13`.

📌 **`Status: COMPLETED` on this document is the close signal. "Commit pushed" is never a DoD item.**

---

## §7 — Filed, NOT fixed

- 🔓 **`ThreadXgid` — mint it, or rule the three thread slots DESCRIPTIVE. JOE'S.** It is a decision against a **documented refusal** (`AE-D8`), so it needs its own grounding pass on why the refusal was made. **Its standing reminder is the three manifest rows, which the gate surfaces every session.**
- 📌 **`ThreadState.origin_event` is WRITE-ONLY** (§2d). *A field nothing reads is a field nobody has round-tripped.* Whether it should exist at all is a `xgen-core` Space-state question, not a retype.
- 📌 **`envelope.rs` `ErrorBody.event_id`** — the one UNREAD slot of the 88; its producer still needs reading. **Leg D or later, and it gates nothing.**
- 📌 **The 12 function-parameter slots** stay out of scope (Phase-0 §9).

---

## §8 — Handoff

**Clair implements from this document once Joe locks it. Rule 6 stands, and it has now paid for itself twice on runbooks of mine (J-516, J-665) — flag a bad instruction, do not absorb it.**

⚠️ **THE THREE PLACES THIS RUNBOOK IS MOST LIKELY TO BE WRONG, NAMED SO THEY GET CHALLENGED:**

1. **§2e's byte-identity claim at `Option<EventXgid>`.** `Option` adds a `null` case Leg B never exercised, and `Xgid: Default` makes an empty-but-present id representable. **V3-a's `null` arm exists to break this.** If it fails, stop — the finding is bigger than the leg.
2. **§1b's flavour choices.** `target_event_id → EventXgid` and `origin_event → EventXgid` are read from the field names and their doc comments, not from a producer. **Check each at its writer** the way J-665 checked `home_node` at `app.rs:3561`; a flavour assigned from a name is a guess wearing a type's clothes.
3. **R4's blast radius.** `AuthOutcome` measured at 3 refs in one file, but that count is `grep`, not `cargo`. **The compiler is the authority.**

🔒 **Chat re-drives every measured leg (Rule 5).** No number in the close is taken on report.
