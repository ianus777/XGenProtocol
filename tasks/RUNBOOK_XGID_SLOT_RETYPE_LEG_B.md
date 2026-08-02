# M-RP-XGID-SLOT-RETYPE Leg B — the four address-book slots, the map key, and the three sites that disappear with them
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

✅ **LEG B IS IMPLEMENTED, RE-DRIVEN AND CLOSED (J-665). Commit `c2975f3` [Clair, code] + the doc bridge [Chat]. Every number below was re-measured by Chat under Rule 5; not one entered the record on report.**

🛑 **AND ONE OF THIS RUNBOOK'S OWN INSTRUCTIONS WAS WRONG — B4 POINTED AT THE WIRE.** §B4 listed `ops.rs:3257/:3261` as *"`FetchedIdentity` fixtures — wrap"*. **They are the `IdentityMessage::Record` WIRE variant's fields**, which stay `String` under `D-137` §1 clause 3. **Clair refused the instruction and flagged it (Rule 6).** ⚠️ *Had she absorbed it she would have retyped a wire enum field — a defect at the exact boundary this milestone exists to respect.* Kept not erased (`D-131`); the mechanism is recorded at J-665.

🔒 **LOCKED BY JOE 2026-08-02 (*"locked and go"*), AS AUTHORED — no corrections raised. Chat wrote it; Clair implements from THIS version and does not close her own leg (D-123).**

📌 **RENAMED AT LOCK: *"the three address-book structs"* → *"the four address-book slots"*.** Phase-0 §6's name counts structs, and **only two of the three carry work** (`FillReport` contributes zero, §1). **The unit that is true of this leg is the SLOT.** Old name annotated, not erased, at `tasks/M_RP_XGID_SLOT_RETYPE.md` §6 (`D-131`).

🔒 **PARENT:** `tasks/M_RP_XGID_SLOT_RETYPE.md` v1.4 §6 (Leg B) · §3a (the 88-slot hand-read) · §7 (the narrowing) · `DECISIONS.md` **D-136** and **D-137**.

---

## §1 — What this leg is, in one paragraph

**Four `String` identifier slots become typed XGID flavours, the address book's `BTreeMap` key becomes typed with them, and three call sites that exist only to bridge the two forms disappear.** It is the first code written since J-657 and the only leg of this milestone on `M-RP-IDENTITY-RESOLUTION` Leg D's critical path (Phase-0 §7).

**IN SCOPE — four slots:**

| file | line at authoring | slot | → |
|---|---|---|---|
| `xgen-client/src/address_book.rs` | `:80` | `SeenRecord.identity_id` | `IdentityXgid` |
| `xgen-client/src/address_book.rs` | `:89` | `SeenRecord.home_node` | `NodeXgid` |
| `xgen-client/src/ops.rs` | `:428` | `FetchedIdentity.identity_id` | `IdentityXgid` |
| `xgen-client/src/ops.rs` | `:432` | `FetchedIdentity.home_node` | `NodeXgid` |

**AND THE MAP KEY, WHICH IS THE POINT AND NOT A SIDE EFFECT:**

| `xgen-client/src/address_book.rs` | `:174` | `AddressBook.records: BTreeMap<String, SeenRecord>` | `BTreeMap<IdentityXgid, SeenRecord>` |
|---|---|---|---|

🔑 **THE KEY IS *WHY* `SeenRecord` FAILS D-137 §1 CLAUSE 3.** The struct sits at a persistence edge *and* a Tauri IPC edge, so its slots look BOUNDARY. They are not: **a `BTreeMap<String, SeenRecord>` is internal state holding the external form**, which is exactly the half of clause 3 that stops BOUNDARY becoming a loophole. **Retyping the fields and leaving the key is a half-fix that would satisfy the grep gate and not the rule.**

📌 **`FillReport` (`ops.rs:2773`) CONTRIBUTES ZERO SLOTS.** Its identifier slot `not_found_ids` is already `Vec<IdentityXgid>` from `M-RP-IDENTITY-RESOLUTION` Leg A. **The leg's name in Phase-0 §6 says three structs; only two carry work.**

**OUT OF SCOPE, EXPLICITLY:** the 13 Leg C slots · the 12 function-parameter slots (Phase-0 §9 — Pass 2/3/4 territory) · `M13` · anything in `ui/**`.

---

## §2 — Grounding, measured 2026-08-02 at `e95fa65` (HEAD = origin/main, tree clean, gate PASS 88)

**Every claim below was read from the file, not recalled. Line numbers are as-at authoring and WILL shift — the anchors in §3 are text, never line numbers.**

### §2a — 🛑 THERE ARE THREE SITES, NOT TWO. The session kickoff named two.

The kickoff's *"THE TWO DOWNGRADES MUST DISAPPEAR, NOT MOVE"* is **narrower than the thing it describes** — the project's named recurring species, this time on the kickoff itself. There is a **third site**, and it is a *compensating re-wrap* rather than a downgrade:

```
ops.rs:2734    ids.insert(e.sender.as_str().to_string());              // downgrade
ops.rs:2742    ids.insert(m.identity_id.as_str().to_string());          // downgrade
ops.rs:2935    report.not_found_ids.push(IdentityXgid::from_xgid(Xgid::new(id.clone())));   // re-wrap
```

🔑 **`:2935` CARRIES A COMMENT NAMING THIS MILESTONE AND PREDICTING ITS OWN REMOVAL** — *"This re-wrap goes away when that milestone lands; the field type is already correct and needs no rework."* **The comment goes with the code.** 🛑 **All three vanish together or none do:** `:2734`/`:2742` exist only to feed the `String`-keyed map, and `:2935` exists only to undo them.

✅ **BOTH DOWNGRADE SOURCES ARE ALREADY TYPED — the downgrade is pure loss, not a projection:**
- `Event.sender: IdentityXgid` (`xgen-common/src/wire.rs:482`)
- `MemberEntry.identity_id: IdentityXgid` (`ops.rs:2617`)

### §2b — ✅ THE FLAVOURS SUPPORT EVERYTHING THIS LEG NEEDS. Read at `xgen-common/src/xgid/`.

`declare_flavour!` (`flavours.rs:125-195`) gives every flavour:

- `#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]` ⇒ **`Ord` makes a `BTreeMap` key legal.**
- `#[serde(transparent)]` over `Xgid(String)` (`base.rs:32-36`) ⇒ **the serialised form is a plain JSON string, byte-identical to today's.**
- `impl Borrow<str>` (`flavours.rs:190-194`) ⇒ **`BTreeMap<IdentityXgid, V>::get(&str)` compiles with no wrapper allocation.**
- `impl Deref<Target = Xgid>` ⇒ `.as_str()` reaches through.

### §2c — ✅ THE RETYPE IS INVISIBLE ON DISK AND ACROSS TAURI IPC — which is why it does not move the boundary

`SeenRecord` is `Serialize + Deserialize` and reaches **two** format edges:

- **on disk** — `AddressBook::save` / `::load` (`address_book.rs:313-332`), pretty JSON;
- **across Tauri IPC** — `get_address_book` returns the whole `AddressBook` to the webview (`desktop.rs:637-642`).

🔑 **Under `#[serde(transparent)]` both emit exactly the bytes they emit today.** ⇒ **zero `ui/**` change and the `svelte-check` floor must not move.** 🛑 **AND THAT IS AN ASSERTION UNTIL IT IS A TEST:** the one thing that can break silently is **`BTreeMap` *key* deserialisation through a double newtype** (`IdentityXgid` → `Xgid` → `String`), which travels a different serde path from a value. **V3 exists to reach it. Do not argue it; run it.**

⚠️ **RECORD THIS IN THE CODE, so a later reader does not "fix" it back:** the struct is at a boundary and is still INTERNAL, because clause 3's second half disqualifies it. A doc comment on `records` naming D-137 is part of B1.

### §2d — ✅ `NodeXgid` IS THE RIGHT FLAVOUR — CONFIRMED AT THE PRODUCER, NOT INHERITED FROM PASS 4

Phase-0 §8 flags that `SeenRecord.home_node: NodeXgid` *"contradicts a Pass 4 borderline lock BY NAME"*. **Checked at the emitter rather than taken from the lock:** `xgen-node/src/app.rs:3561` builds `identity.record`'s `home_node` as `record.home_node.as_str().to_string()` — **projected down from a typed `NodeXgid` on the node side.** The value that arrives is a `xgen://pubkey/ed25519:…` URI.

📌 **The `"ws://127.0.0.1:8080/xgen"` `home_node` literals in `ops.rs` tests (`:3067`, `:3109`, `:3146`, `:3218`) belong to `ClientState`, a different struct.** **No fixture is lying, and none of them is in this leg.**

### §2e — ✅ THE ACCESSOR SURFACE DOES NOT MOVE, AND ~30 TEST CALL SITES COMPILE UNTOUCHED

`AddressBook::get` / `contains` / `touch` / `remove` take `&str` (`:194`, `:199`, `:245`, `:253`). **`Borrow<str>` keeps every one of them compiling against a typed map.** Measured: **~30 call sites in `address_book.rs` tests and `ops.rs` tests pass string literals** to those four methods. **All of them stay as they are.** See §5 for the call that keeps it that way.

### §2f — 🛑 THE CARGO FLOOR DOES NOT MOVE BY ITSELF. The kickoff asserts it does.

**A pure retype adds no tests.** `1589/0/62 × 56` would come back **identical**, and an identical floor after a Rust leg is the *inverse* signal this project uses (the M-RP6.1i/6.1j *"identical PROVES no-Rust"* leg). ⇒ 🔒 **the floor moves because V3 adds tests, and by exactly that many.** **A delta explained after the fact is not an explained delta.**

---

## §3 — Executable constraints. Every one is a rule Clair can check against the file, not a preference.

- **R1 — THE THREE SITES ARE DELETED, NOT RELOCATED.** After B3 a repo-wide search for `.as_str().to_string()` in `observed_identities` returns **nothing**, and `IdentityXgid::from_xgid(Xgid::new(` does not appear anywhere in `fill_from_events`. 🛑 **A leg that retypes the structs and keeps any of the three has not closed its own defect.**
- **R2 — THE MAP KEY IS TYPED IN THE SAME COMMIT AS THE FIELDS.** `BTreeMap<String, SeenRecord>` must not survive B1.
- **R3 — THE FOUR ACCESSOR SIGNATURES STAY `&str`** (`get` · `contains` · `touch` · `remove`). They are function parameters, and Phase-0 §9 puts function parameters out of scope. See §5.
- **R4 — NO NEW `String` IDENTIFIER SLOT IS INTRODUCED ANYWHERE.** The gate is the check; run it, do not reason about it.
- **R5 — THE WIRE PARAMETER STAYS `&str`.** `identity_get_on(conn, identity_id: &str)` (`ops.rs:495-497`) feeds `IdentityMessage::Get { identity_id: String }` — a BOUNDARY under D-137 §1 clause 3 with the projection at the edge. Call it `identity_get_on(conn, id.as_str())`.
- **R6 — NO `ui/**` FILE IS TOUCHED.** If one appears to need touching, that is a **finding**, not a licence: stop and report it (Rule 6), because it would mean §2c is false.
- **R7 — `FillReport` IS NOT EDITED.** Its type is already correct. The only change near it is the *removal* of `:2935`.
- **R8 — THE MANIFEST AND THE GATE TRAVEL IN THE SAME COMMIT AS THE CODE** (D-074 applied to the gate). See B6.

---

## §4 — The steps

### B0 — 🛑 RE-MEASURE THE FLOOR BEFORE THE FIRST EDIT

**No code has been written since J-657 and no floor has been re-measured since.** *"Six entries of no code"* is not the same claim as *"the floor is verified"*, and the whole of §2f depends on the baseline being real.

- `cargo test` **DETACHED**, polled in short calls (it exceeds the MCP timeout; a killed run leaves a measurement-shaped artifact with no final `test result:` line).
- Apps **down** — a running client holds `xgen-client.exe` and the run dies in ~15 s on `failed to remove file`.
- Expected: **1589 / 0 / 62 across 56 terminators**. 🛑 **If it differs, STOP and report before editing anything** — an unreconciled baseline makes every later delta unattributable.

### B1 — `address_book.rs`: the struct, the key, and the doc comment

1. `SeenRecord.identity_id: String` → `IdentityXgid`; `SeenRecord.home_node: String` → `NodeXgid`. **Keep the existing doc comments** and extend `identity_id`'s with *"the book key"* — it now is one, typed.
2. `AddressBook.records: BTreeMap<String, SeenRecord>` → `BTreeMap<IdentityXgid, SeenRecord>`.
3. **Add a doc line on `records` recording why this is INTERNAL despite two format edges** — D-137 §1 clause 3's second half, one sentence. *This is the note that stops a later reader "restoring" `String` on the argument that the struct is serialised.*
4. `from_fetched` (`:121-135`) — `fetched.identity_id.clone()` and `fetched.home_node.clone()` are now typed-to-typed. **No projection appears here.**
5. `insert` / `merge` — `record.identity_id.clone()` as the key is now typed. No other change.
6. Import `IdentityXgid` / `NodeXgid` from `xgen_common::xgid`.

### B2 — `ops.rs`: `FetchedIdentity`

1. `identity_id: String` → `IdentityXgid`; `home_node: String` → `NodeXgid`.
2. `parse_identity_get_response` (`:442-…`) destructures the wire `IdentityMessage::Record` — **its `identity_id` / `home_node` are `String` from the wire and stay `String` there.** **The projection lands HERE, in the constructor, and is the one projection per direction D-137 requires.** Write it as `IdentityXgid::from_xgid(Xgid::new(identity_id))` at the point of construction — **not** in `from_fetched`, and **not** at any consumer.
3. ⚠️ **The struct's doc block calls it *"the fetch boundary"* — that stays true.** The boundary did not move; the *form held after the boundary* changed. If the wording now reads as though the type is the wire form, adjust the sentence — do not delete the paragraph.

### B3 — `ops.rs`: the cascade, and the three sites

1. `observed_identities` (`:2723-2746`) → `Result<Vec<IdentityXgid>>`; `BTreeSet<IdentityXgid>`; **`:2734` becomes `ids.insert(e.sender.clone());`** and **`:2742` becomes `ids.insert(m.identity_id.clone());`**.
2. `partition_observed` (`:2759-2765`) → `(Vec<IdentityXgid>, Vec<IdentityXgid>)`. Its `book.contains(id)` needs `id.as_str()`.
3. `fill_from_events` (`:2885-2941`) — `book.touch(id.as_str(), &now)`; `identity_get_on(conn, id.as_str())`.
4. **`:2930-2936` — delete the re-wrap AND its four-line comment; the push becomes `report.not_found_ids.push(id.clone());`.**
5. Nothing else in `fill_from_space` / `fill_and_members` changes. **The re-entrancy invariant and its `conn` clears are NOT touched** — they are load-bearing and unrelated (J-586, D-129).

### B4 — Fixtures and assertions

**Small and enumerated; do not sweep beyond it.**

- `address_book.rs` test constructors at `:342/:345`, `:359/:362`, `:424/:428`, `:456/:459`, `:529/:533` — wrap the literals.
- `address_book.rs:435` / `:438` — `assert_eq!(seen.identity_id.as_str(), "…")`.
- `ops.rs` `FetchedIdentity` fixtures at `:3257/:3261`, `:3870/:3874`, `:3901/:3905`, `:3997/:4001` — wrap.
- `ops.rs:3268` — `assert_eq!(got.identity_id.as_str(), "…")`.
- ✅ **Helpers already exist — reuse, do not re-invent:** `ix()` / `nx()` at `ops.rs:~3348`. If `address_book.rs` tests need the same, add a local pair in the same shape.
- 🛑 **The ~30 `book.get("…")` / `contains("…")` / `touch("…", …)` / `remove("…")` call sites are NOT edited** (§2e). If any of them fails to compile, `Borrow<str>` is not doing what §2b says — **stop and report**, do not paper it with `.into()`.

### B5 — The tests that move the floor, and they are the point

🔑 **These exist because §2c is currently an argument, and an argument is not a measurement.**

- **V3-a — the book round-trips byte-identically.** Build a book with ≥2 records, `save` to a temp dir, `load`, assert equality **and** assert the JSON text against a literal expected string containing bare `"xgen://pubkey/ed25519:…"` keys. 🛑 **The literal is the test** — a `save`→`load` equality alone would pass even if both sides had changed shape together.
- **V3-b — a pre-retype file still loads.** Feed a hand-written JSON fixture in **today's** on-disk shape (a bare object keyed by XGID URI) to `AddressBook::load` and assert the records come back. **This is the one that proves nobody's existing book file breaks.** *Fed, not asserted — the N-091 discipline applied to the branch that has never once run against a typed key.*
- **V3-c — the `Borrow<str>` lookup path.** Insert a record, then `get`/`contains`/`remove` it **by `&str`**, proving R3 is real rather than incidental.

📌 **Three tests ⇒ the cargo floor moves by 3, to 1592/0/62.** 🛑 **MEASURE IT; DO NOT PREDICT IT.** If it comes back anything else, the delta is unexplained and the leg does not close.

### B6 — The manifest and the gate, same commit

🛑 **THE GATE WILL WARN AND THEN FAIL IF ONLY HALF OF THIS IS DONE.** Read from `xgid-slot-gate.ps1`: **Check 1** fails when the row tally disagrees with the `# EXPECTED:` header (`:114`, `:119`); **Check 3** warns when a manifest row is no longer found (`:142`). ⇒ **dropping rows without moving the header is a FAIL, not a warning.**

1. Delete the four `INTERNAL` rows from `xgid-slot-manifest.tsv` (lines 76-79 at authoring):
   `address_book.rs SeenRecord home_node` · `SeenRecord identity_id` · `ops.rs FetchedIdentity home_node` · `FetchedIdentity identity_id`.
2. Header → `# EXPECTED: BOUNDARY=65 DESCRIPTIVE=5 INTERNAL=13 UNREAD=1 TOTAL=84`.
3. Re-run `.\xgid-slot-gate.ps1` on a **clean tree**. ✅ **Required result: PASS at 84 (65 / 5 / 13 / 1), with NO warning** — the retyped slots no longer match the sweep regex (`:49` requires `: String` or `: Option<String>`), so they leave `$post` and the manifest at the same time.
4. 🛑 **The gate refuses a dirty tree.** Run it after the commit is staged and the tree is clean, **not** mid-edit. `-AllowDirty` exists only to exercise the gate itself.

---

## §5 — 🔒 ONE CALL, CHAT'S, UNDER D-123 — REVERSIBLE ON ONE LINE

**The four `AddressBook` accessors keep `&str` rather than widening to `&IdentityXgid`.**

- **① USER-VISIBLE IMPACT: NONE, EITHER WAY.** Internal type discipline; nobody using XGen sees, feels or comes to believe anything different. *D-121's legal answer, and here it is the true one.*
- **② RESOURCE COST:** keeping `&str` ≈ 12 edited lines. Widening ≈ **30 further call sites**, all in tests, plus a churn pass across two files — for a discipline gain on a surface **Phase-0 §9 already excludes** (function parameters are Pass 2/3/4 territory, deliberately not mixed with struct fields so a cargo delta stays attributable).
- **③ Elegance argues for widening. It ranks third and does not decide.**

🔒 **AND IT IS NOT A HALF-FIX:** D-137's test is that **no internal state holds the external form**. The state is the map, and the map key becomes typed. **A parameter is not state.**

📌 **FILED, NOT BUILT:** widening the accessor surface (and the same question for `AddressBook::iter`'s `&String`) is a clean follow-on. **It is Leg C's neighbour, not its rider, and it belongs to nobody until Joe names it.**

---

## §6 — Verification

| # | check | expected |
|---|---|---|
| **V0** | Floor **before** the first edit (B0) | cargo **1589/0/62 × 56** — reconcile or stop |
| **V1** | `cargo test` after B5, detached, apps down | **1592/0/62**, delta = the three V3 tests, **enumerated not derived** |
| **V2** | `svelte-check` | **0/34/15 UNCHANGED** — the proof of §2c at the TS edge |
| **V3** | The three new tests (B5) | all pass; V3-b asserted against a **hand-written pre-retype fixture** |
| **V4** | `.\xgid-slot-gate.ps1`, clean tree | **PASS at 84** (65 / 5 / 13 / 1), **no WARN** |
| **V5** | R1 by search | zero `.as_str().to_string()` in `observed_identities`; zero `from_xgid(Xgid::new(` in `fill_from_events` |
| **V6** | Scope by diffstat | **`xgen-client/src/address_book.rs` · `xgen-client/src/ops.rs` · `xgid-slot-manifest.tsv` and nothing else in code.** Zero `ui/**`, zero `xgen-node`, zero `xgen-core`, zero `xgen-common` |
| **V7** | Sampler catalogue | **435 UNCHANGED, by scope** (V6 proves it; do not re-drive the sampler for a Rust leg) |

⚠️ **V1 and V3 need the client and node DOWN.** Kill any stale dev server with a tree kill filtered on `XGenProtocol` in the command line — a name-only filter on `node.exe` leaves orphans (J-557).

---

## §7 — DoD

- [x] **B0 baseline re-measured and reconciled to 1589/0/62 × 56** — ✅ exact, clean run, 56 terminators, final `test result:` line present
- [x] `SeenRecord.identity_id` is `IdentityXgid` — ✅ `address_book.rs:82`
- [x] `SeenRecord.home_node` is `NodeXgid` — ✅ `address_book.rs:91`; **discharges the Phase-0 §8 item that contradicts a Pass 4 borderline lock by name**, flavour confirmed at the producer (§2d)
- [x] `FetchedIdentity.identity_id` is `IdentityXgid` · `.home_node` is `NodeXgid` — ✅ `ops.rs:428/:432`
- [x] `AddressBook.records` is `BTreeMap<IdentityXgid, SeenRecord>` **and carries the D-137 note** — ✅ `address_book.rs:176-183`
- [x] **All THREE sites are GONE, not moved** — ✅ V5 scoped to the two functions: `observed_identities` uses typed `.clone()` ×2, `fill_from_events` has **zero** downgrades and **zero** re-wraps; the `:2935` push is `id.clone()` and the four-line comment is gone
- [x] The four accessors still take `&str` and the ~30 test call sites are unedited — ✅ `:203 :208 :259 :267`
- [x] **V3-b run against a hand-written pre-retype on-disk fixture** — ✅ fed, not asserted
- [x] **cargo floor re-measured and the delta explained by name** — ✅ **1592/0/62 × 56**, Δ **+3** = `v3a` · `v3b` · `v3c`, each seen `... ok` in the log, **enumerated not derived**
- [x] `svelte-check` **0/34/15 unchanged** — ✅ re-run by Chat
- [x] **Manifest rows dropped AND `# EXPECTED:` moved to `INTERNAL=13 TOTAL=84`, same commit** — ✅ 1 insertion / 5 deletions, tally re-counted 65/5/13/1 = 84
- [x] **Gate re-run on a clean tree: PASS at 84, no WARN** — ✅
- [x] Scope confirmed by diffstat — ✅ exactly 3 files; zero `ui/**`, `xgen-node`, `xgen-core`, `xgen-common`

🛑 **NOT IN THIS DoD, AND NO ITEM HERE MAY BE READ AS DISCHARGING THEM:** `M-RP-IDENTITY-RESOLUTION` **Leg D** (this leg *unblocks* it; it does not begin it) · that milestone's **Leg E** and **Leg F** (**Leg F is the first behaviour verification of that whole milestone and nothing in it has been exercised against two clients**) · this milestone's **Leg C** (13 slots) and **Leg D** (records + close). 🔒 **Under N-168, G-B closes on Leg D AND Leg E together and no single leg may tick it.**

📌 **`Status: COMPLETED` on this document is the real close signal. "Commit pushed" is never a DoD item.**

---

## §8 — Filed, NOT fixed

✅ **§9's TWO NAMED RISK POINTS ARE NOW MEASURED, NOT ARGUED.** ① The byte-identity claim at the `BTreeMap` key: `v3a` pins the **exact on-disk JSON literal** (bare `"xgen://…"` keys, plain-string `identity_id` / `home_node`) and it round-trips; **and Chat checked the structural reason separately — the `bb3ac6e→c2975f3` diff changes ONLY the two type names, with no field rename and no serde attribute added or removed**, which is *why* the bytes cannot move. ② The floor arithmetic returned **1592**, exactly the prediction, delta named.

📌 **AND AN HONEST LIMIT ON THE PAIR: `v3a` AND `v3b` ARE NOT TWO INDEPENDENT PROOFS.** Because transparency makes the pre- and post-retype bytes **identical**, `v3b`'s "legacy" fixture is byte-identical to what the new code writes. 🔑 **That identity IS the finding — stated as data instead of as an argument** — but `v3b` is best read as a **regression pin** (it keeps the promise explicit if transparency is ever removed) rather than as a second, separate measurement. *Recorded so nobody later cites two proofs where there is one proof and one guard.*

- 📌 **Widening the `AddressBook` accessor surface to `&IdentityXgid`** (§5). ⚠️ **`AddressBook::iter` moved anyway and that is NOT this item** — a `BTreeMap<IdentityXgid, _>` has no `&String` to hand out, so its item type is **forced by the key retype**, not chosen. Documented in place at `address_book.rs:212-218`.
- 📌 **`ops.rs`'s five remaining Leg C slots sit in the file this leg opens** (`ThreadCreateResult` · `ThreadStatusResult` · `RedactResult` · `VerbReject` · `RoomsResult`). ⚠️ **They are NOT taken here.** Phase-0 §6 recommends Leg C is not split off *precisely because* it shares this file — **that argument is about milestone boundaries, not about scope creep inside a leg.**
- 📌 **`envelope.rs` `ErrorBody.event_id`** — the one UNREAD slot of the 88. Its producer still needs reading. **Not this leg.**
- ⚠️ **The kickoff's "two downgrades" (§2a).** Recorded as a finding rather than silently corrected (D-131), because the same species has now appeared in a Phase-0, a `D` entry and a session kickoff inside one arc.

---

## §9 — Handoff

**Clair implements from this document once Joe locks it. Rule 6 stands: an implementer who silently absorbs a bad instruction ships the architect's mistake — flag a deviation, do not absorb it.**

⚠️ **THE TWO PLACES THIS RUNBOOK IS MOST LIKELY TO BE WRONG, NAMED SO THEY GET CHALLENGED RATHER THAN TRUSTED:**

1. **§2c's byte-identity claim at the `BTreeMap` key.** A typed *value* and a typed *map key* travel different serde paths. **V3-a and V3-b exist to break it.** If either fails, the finding is bigger than this leg and the leg stops.
2. **§2f's floor arithmetic.** `1589 → 1592` assumes exactly three new tests and no incidental `#[test]` removal. **Measure; the number in this document is a prediction until V1 returns.**

🔒 **Chat re-drives every measured leg (Rule 5).** No number in the close is taken on report.
