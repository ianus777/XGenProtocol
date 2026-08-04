# RUNBOOK — M-RP-IDENTITY-RESOLUTION Leg D — Tier-1 fetch on join
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, the seat, and what it is NOT

**Implementation runbook for `M-RP-IDENTITY-RESOLUTION` Leg D.** Authored by Chat from the four locks in `tasks/M_RP_IDENTITY_RESOLUTION_LEGD_PHASE0.md` v1.1 §§3–6 (Joe, 2026-08-03: **A3 · B2 · C1 · D1**).

🔒 **SEATS (`D-123`).** Joe: architecture and appearance; locks; **pushes all commits**. Chat: this runbook, grounding, measurement, records, verification, the technical rulings. **Clair: implements from this document and does NOT close her own leg** — she hands back with the numbers and stops.

🛑 **THIS RUNBOOK IS NOT GROUND TRUTH.** Session-open reading order is `CLAUDE.md` PLAY head → latest JOURNAL entry → ACTIVE handoffs in `tasks/` → **then** this. Runbook-as-ground-truth is a named failure mode.

🔑 **READ §7 BEFORE IMPLEMENTING ANY STEP.** It names where this document is most likely wrong. ⚠️ **It is NOT a census of its errors — it is only the doubts its author already had.** Four of Chat's lines were wrong last arc and Clair caught three; the fourth and fifth fell to producer checks. **Refuse any step whose cited producer does not say what this document says it says.**

⚠️ **LEG D IS COMPILE- AND TYPE-VERIFIED ONLY.** A joiner requires an inbound `membership.join` from a second identity. **Every positive case is Leg F's.** A store driven by hand is a probe that cannot fail and is **not** admissible as evidence here.

---

## §1 — Grounding

`tasks/M_RP_IDENTITY_RESOLUTION_LEGD_PHASE0.md` §1 (G-D1…G-D17) is the grounding for this leg and is **not restated**. The table below carries **only what was measured additionally on 2026-08-03 at `aae60be`**, with read windows stated.

| # | fact | citation | window |
|---|---|---|---|
| **R-G1** | `struct FillLock(tokio::sync::Mutex<()>);` — declared at `:95`, `.manage(...)`-d at `:979` | `desktop.rs:95`, `:979` | grep |
| **R-G2** | `invoke_handler` list — 18 names, `fill_space_records` last, **no trailing comma** on the final entry | `desktop.rs:1012–1031` | 1006–1075 |
| **R-G3** | `SeenRecord` — the book's own narrower type, **deliberately NOT `FetchedIdentity`**; its doc says so outright. `identity_id: IdentityXgid`, `home_node: NodeXgid`, five book-local fields | `address_book.rs:80` (doc from `:70`) | 60–158 |
| **R-G4** | `SeenRecord::from_fetched(&FetchedIdentity, last_seen)` — the **single producer** of a book record from a wire fetch; the three wire-absent fields take no-opinion defaults | `address_book.rs:123` (doc from `:117`) | 60–158 |
| **R-G5** | `AddressBook::merge` is **version-aware** — a fetched record (`update_version 0`) never displaces a seeded higher version | `address_book.rs:240`, `ops.rs:2822–2826` | grep + 2815–2870 |
| **R-G6** | T8 `tauri_command_return_serde_transparent_to_js_frontend` asserts against **`PacingState`** and is about that type | `desktop.rs:1050` (doc from `:1045`) | 1006–1075 |
| **R-G7** | 🛑 **`app_client.svelte` has NO `selfId` and no read of self's identity id.** The only `identity_id` in the whole file is `:264`, the entry being built | `app_client.svelte` | full-file grep |
| **R-G8** | The `_book` TS mirror's value type is `SeenRecord` (the store says so in its own doc comment) | `address-book.svelte.ts:70–83` | full file |

📌 **`address_book.rs` line drift, annotated not repaired (`D-131`):** `M_RP_IDENTITY_RESOLUTION.md` §5b cites `remove` at `:253` and `evict_older_than` at `:285`; measured **`:267`** and **`:299`**. The claim they support — *every caller is a test* — is unaffected.

---

## §2 — The four locks, restated in one line each

| lock | ruling |
|---|---|
| **§3 → A3** | **One-shot `identity_get` per joiner.** The batched form (`identity_get_on`) stays private; A2 is filed with a trigger — **Leg F measures join concurrency alongside the Tier-1 residue** |
| **§4 → B2** | **The command PERSISTS.** `AddressBook::load` → fetch → `absorb_fetch` → `book.save()`, mirroring `fill_space_records` |
| **§5 → C1** | **The command takes the `FillLock`**, first, before any book I/O |
| **§6 → D1** | **A `not_found` result reaches the frontend and pushes to `_notFound`** — the ③ arm |

---

## §3 — Chat's rulings for this leg (`D-123`) — recorded, reversible on one line

`R-D1`…`R-D5` live in the Phase-0 §7 and are not restated. **Two more were taken while authoring this runbook**, both forced by measurement:

- 🔒 **R-D6 — `addMember` RETURNS `boolean`** (true iff the roster actually gained the member). **Reason:** the fetch must not fire when the add no-ops under `R1` (unknown roster), `R3` (already present) or `R4` (scope moved on) — and the router cannot see which happened without duplicating the three rules outside the store, which is the `D-067` drift the rules were centralised to prevent. **Additive:** no existing caller reads the return.
- 🔒 **R-D7 — NO SELF-GUARD IN THE ROUTER. Considered and refused, with its reason, rather than silently omitted.** `app_client.svelte` has no `selfId` (**R-G7**), so guarding self would mean threading self's identity into the shell's event router to save at most one fetch per Space join — and `addMember`'s `R3` idempotency already prevents repeats. ⚠️ **Consequence owned:** if self's own `membership.join` reaches the router, self's record is fetched and cached. **Harmless** — `L4` excludes self at render — and it is named here so Leg F does not report it as a defect.

🔒 **AND ONE THAT CHANGED SINCE THE PHASE-0 WAS WRITTEN.** Phase-0 §8 offered *"extending T8 may make D-i's cargo delta `+0`"*. **Measured (R-G6): T8 is about `PacingState`.** Widening it would make one test answer two questions and blur what a failure means. ⇒ **a NEW test ships and the expected cargo delta is `1595 → 1596`, stated before the run.** *Kept, not erased (`D-131`); the Phase-0 offered it as a possibility, not a claim.*

---

## §4 — D-i — the Rust half (`xgen-client`). **COMMIT ALONE. Moves cargo.**

🛑 **MEASURE THE BASELINE BEFORE THE FIRST EDIT AND DO NOT INHERIT IT.** Expected `1595 / 0 / 62 × 56` — measured at `aae60be` on 2026-08-03. `cargo test` **exceeds the MCP timeout**: run detached, poll in short calls, sum terminators programmatically, grep `FAILED` **case-sensitively** (case-insensitive matches `0 failed`).

### Step D-i-1 — the command, in `xgen-client/src/desktop.rs`

Place it **immediately after `fill_space_records`** (which ends at `:713`) and **before `quit`** (`:715`), so the two book-touching commands sit together.

```rust
/// Fetch ONE Identity record on demand and absorb it into the book
/// (M-RP-IDENTITY-RESOLUTION Leg D, §7 Tier-1 fetch on join).
///
/// Called by the shell's membership router when a live `membership.join`
/// adds a member the fill was never asked about (G7) — the ONLY state in
/// which a roster row exists with no book record.
///
/// Returns the record AS THE BOOK NOW HOLDS IT, not as the wire delivered
/// it: `merge` is version-aware (§5 V2), so a seeded higher-version record
/// out-ranks a fetch and the caller must mirror what won, not what was sent.
///
/// `Ok(None)` ⇔ `identity.not_found` ⇔ state ③ (ERASED under `D-127`) — a
/// normal outcome, never an error. The frontend routes it to `_notFound`.
///
/// Takes the `FillLock` (§3): this is the SECOND writer of the on-disk book
/// and a read-modify-write race with a concurrent fill would silently
/// discard resolved records — the outcome `fill_space_records:678-679`
/// names in its own words.
#[tauri::command]
async fn fetch_identity(
    identity_id: String,
    data: tauri::State<'_, DataDir>,
    config: tauri::State<'_, ConfigPath>,
    lock: tauri::State<'_, FillLock>,
) -> Result<Option<crate::address_book::SeenRecord>, String> {
    let _guard = lock.0.lock().await;

    let data_dir = data.0.clone();
    let config_path = config.0.clone();

    // The `fill_space_records` preamble, unchanged: resolve the node so
    // `home_node` is non-empty (else `ensure_connected` bails), load the
    // identity so the fetch can authenticate.
    let node = app::resolve_node(None, &config_path);
    let mut session = crate::session::SessionState::new(node, data_dir.clone());
    session
        .ensure_identity(&app::resolve_keypair_path(&config_path))
        .map_err(|e| format!("{e:#}"))?;

    let mut book = crate::address_book::AddressBook::load(&data_dir).map_err(|e| format!("{e:#}"))?;
    let mut ctx = crate::ops::OpContext {
        session: &mut session,
        data_dir: &data_dir,
        node_override: None,
    };

    let fetched = crate::ops::identity_get(&mut ctx, &identity_id)
        .await
        .map_err(|e| format!("{e:#}"))?;

    let found = fetched.is_some();
    let now = /* the same RFC-3339 "now" the fill stamps — SEE §7-b */;
    let _absorbed = crate::ops::absorb_fetch(&mut book, fetched, &now);

    // Persist before answering, for the same reason `fill_space_records`
    // does: an absorbed observation that is not saved is work thrown away.
    book.save(&data_dir).map_err(|e| format!("{e:#}"))?;

    Ok(if found {
        book.get(&identity_id).cloned()
    } else {
        None
    })
}
```

🛑 **THE `found` FLAG IS LOAD-BEARING AND IS NOT REDUNDANT WITH `book.get(...)`.** `absorb_fetch` leaves the book **unchanged** on `not_found` (`ops.rs:2837` — *"do NOT poison the book with a placeholder"*), so a pre-existing record would still be returned by `book.get`. **Without `found`, an erased identity that happens to be cached would be reported as resolved and the ③ arm would never fire.**

⚠️ **`absorb_fetch` is currently private** (`fn`, `ops.rs:2827`). Making it `pub(crate)` is the minimal change. **Do NOT make it `pub`** — it is an internal absorption step, not an API.

### Step D-i-2 — register the command

Add `fetch_identity` to `tauri::generate_handler![...]` (`desktop.rs:1012–1031`). ⚠️ **`fill_space_records` is currently the last entry and carries NO trailing comma** (R-G2) — add the comma when appending.

### Step D-i-3 — the witness test

A **new** test in `mod pass_4_commit_1_tests`, beside T8:

```rust
/// Leg D — `fetch_identity`'s return crosses the IPC boundary with its two
/// identifier slots as PLAIN STRINGS. `SeenRecord` carries `IdentityXgid`
/// and `NodeXgid`; if either lost serde-transparency the frontend's
/// `_book` would silently receive nested objects and every name lookup
/// would miss.
#[test]
fn fetch_identity_return_serde_transparent_to_js_frontend() { /* … */ }
```

🔑 **IT MUST BE A TEST THAT CAN FAIL.** Assert the **exact JSON literals** for `identity_id` and `home_node` (the T8 idiom, `desktop.rs:1063–1067`), not `serde_json::Value` equality. Build the `SeenRecord` with an **exhaustive named literal — no `..Default::default()`** — so a future field addition breaks the test rather than passing silently (the J-669 V3-a rule).

⇒ **Expected cargo: `1595 → 1596 / 0 / 62 × 56`.** Any other number is a finding.

### Step D-i-4 — the gate

Run `.\xgid-slot-gate.ps1` with the apps down. **Expected PASS 74, unchanged.** ⚠️ **No new struct is introduced** (the command returns the existing `SeenRecord`; `identity_id: String` is a **function parameter**, not a struct slot). **If the manifest count moves, STOP and report it — it is a finding to be ruled, not a failure to suppress.**

---

## §5 — D-ii — the frontend half (`ui/**`). **COMMIT ALONE. Moves `svelte-check`.**

🛑 **RE-MEASURE `svelte-check` BEFORE THE FIRST EDIT.** The `0 / 34 / 15` on record is **inherited** from `87307e8` and is not a baseline until it is re-run.

### Step D-ii-1 — `ui/common/lib/stores/address-book.svelte.ts`

**(a) `addMember` returns `boolean` (R-D6).** All three early returns become `return false;`; the tail returns `true` after the reassignment. Update the doc comment at `:180–182` to say what the return means.

**(b) A new setter, after `removeMember`:**

```ts
/** Leg D — one member resolved by the Tier-1 fetch on join. `record === null` means the node answered
 *  `identity.not_found` (state ③, ERASED under D-127), NOT that the fetch failed — a rejected invoke never
 *  reaches here. Three writes, and the SECOND is the point:
 *    ① merge ONE record into `_book` — `setResult` stays the only WHOLESALE writer;
 *    ② CLEAR `unresolved` on the roster row — `members-panel:101` tests the marker BEFORE it reads the
 *       book, so without this the record lands and the AI badge stays dark;
 *    ③ on null, push to `_notFound` — otherwise an erased joiner is dimmed forever and §4c-i's promise
 *       never concludes.
 *  Scope-guarded like `setResult`/`addMember`: a fetch fired at join and resolving after a room switch
 *  must not clear a marker in a scope the user has left. */
resolveMember(spaceId: string, identityId: string, record: SeenRecord | null): void {
  if (spaceId !== _spaceId) return; // scope guard — the setResult/addMember idiom
  if (_roster === null) return;     // R1 — no delta onto an unknown roster
  if (record !== null) {
    _book = { ..._book, [identityId]: record };            // ① merge one, never wholesale
  } else if (!_notFound.includes(identityId)) {
    _notFound = [..._notFound, identityId];                // ③ the erased arm, idempotent
  }
  _roster = _roster.map((m) =>                             // ② the marker clear — the leg's whole point
    m.identity_id === identityId ? { ...m, unresolved: false } : m,
  );
},
```

🔑 **② RUNS ON BOTH ARMS, DELIBERATELY.** An erased joiner's attempt **concluded**; leaving `unresolved: true` would keep saying *not yet* about a question already answered. The ③ filter in `members-panel` then hides the row (or marks it, under §5a's E2, if it is the DM counterpart).

⚠️ **`unresolved: false`, NOT `delete`.** `members-panel:101` tests truthiness, so both work today — but the field is a declared optional on `MemberEntry` and an explicit `false` distinguishes *asked and answered* from *never asked*, which is exactly the `?? null` reasoning Leg B's Change 4(c) already applied to the prop.

**(c) `_book`'s reassignment is a fresh object** — Svelte 5 `$state` tracks the reference; a mutation in place would not re-render.

### Step D-ii-2 — `ui/client/src/app_client.svelte`, the `membership.join` arm

`routeMembershipEvent` **stays synchronous** (R-D4). Guard the fetch on `addMember`'s new return (R-D6):

```js
const added = addressBook.addMember(payload.space_id, { /* unchanged */ });
if (added) void fetchJoinerIdentity(payload.space_id, subject);
```

And a module-scope helper beside `loadMembers`:

```js
// Leg D — the Tier-1 fetch on join (§7). Fire-and-forget: the router must not become async, and a failed
// fetch leaves the row at state ④ (dimmed), which is exactly what §4 says ④ is for. `tauriInvoke`, NOT the
// bare `invoke` at :651 — that one is an onMount-local destructured import and does not resolve at this
// scope (the J-670 lesson ②).
async function fetchJoinerIdentity(sid, identityId) {
  try {
    const record = await tauriInvoke('fetch_identity', { identityId });
    addressBook.resolveMember(sid, identityId, record ?? null);
  } catch {
    /* the fetch failed or timed out — the row stays ④. Leg F measures how often (§6b Owes:). */
  }
}
```

⚠️ **`identityId`, camelCase.** Tauri maps snake_case Rust parameters to camelCase JS — the `fill_space_records` call at `:222` passes `{ spaceId: sid }` against a Rust `space_id`. **Follow the existing call, not the Rust signature.**

### Step D-ii-3 — nothing else

**No change to `MemberEntry`'s shape, `members-panel.svelte`, `entity-item.svelte`, `entity-panel.svelte`, or any `skin.css`.**

---

## §6 — Verification gates

| # | gate | expected |
|---|---|---|
| **V0** | cargo floor **before** the first `.rs` edit | **1595 / 0 / 62 × 56** |
| **V1** | cargo after D-i | **1596 / 0 / 62 × 56**; final `test result:` present; `FAILED` **case-sensitive** = 0 |
| **V2** | the Δ enumerated **by name** | exactly `fetch_identity_return_serde_transparent_to_js_frontend … ok` |
| **V3** | 🔑 the new test **proven able to fail** — flip one asserted literal, see it fail, revert | fails, then passes |
| **V4** | `.\xgid-slot-gate.ps1`, apps down, clean tree | **PASS 74 (65 / 5 / 3 / 1)** — any movement is a finding |
| **V5** | `svelte-check` **before** the first `ui/**` edit | re-measured, not inherited |
| **V6** | `svelte-check` after D-ii | Δ from V5 stated and explained |
| **V7** | scope | D-i: `desktop.rs` + `ops.rs` visibility only, **zero `ui/**`**. D-ii: **zero `.rs`**, and within `ui/**` only `address-book.svelte.ts` + `app_client.svelte` |
| **V8** | the split reproduces | D-i's committed tree compiles and tests **alone**, before D-ii exists |

🛑 **NOT VERIFIABLE HERE, AND SAYING SO IS THE GATE:** a real joiner, a real `not_found`, the badge lighting, the ③ filter firing, and the join-concurrency count. **All Leg F's**, and Leg F's obligation list grows by them.

---

## §7 — 🛑 WHERE THIS RUNBOOK IS MOST LIKELY WRONG

⚠️ **THIS IS NOT A CENSUS OF ITS ERRORS. It is only the doubts its author already had.** The errors that matter are the ones not on this list — four of five last arc were caught from **outside** the text, none by re-reading it. **Check the producer, not the name; refuse the step if they disagree.**

- **(a) `crate::address_book::SeenRecord` as a Tauri return type.** It derives `Serialize` (R-G3), which is what the boundary needs — but **no existing command returns it**, and `get_address_book` returns the whole book. **If the type does not cross cleanly, that is a real finding**, not a transcription slip.
- **(b) 🛑 THE `now` TIMESTAMP IS A DELIBERATE HOLE IN §4's CODE.** `absorb_fetch` takes `now: &str` and this document **does not know which helper the fill uses to produce it** — it was not measured. **Find the producer `fill_from_events` uses and use the same one.** A second timestamp format in the same `last_seen` field would be a silent data defect that no gate here can see.
- **(c) `absorb_fetch`'s visibility.** `pub(crate)` is stated as minimal, but it may already be reachable, or the module may not be a sibling. Read it.
- **(d) `book.get(&identity_id)` takes `&str`** (`address_book.rs:203`) while the book key is `IdentityXgid`. It compiles today for the fill's callers; **confirm it, do not assume the `String` parameter passes straight through.**
- **(e) `addMember`'s return type change (R-D6).** Stated as additive on the claim that no caller reads it. **That claim came from a grep of `app_client.svelte`, not of `ui/**` — a reference count is only as wide as the name searched.** Sweep `ui/**` including the sampler before believing it.
- **(f) The `identityId` camelCase mapping** is taken from `:222`'s `{ spaceId: sid }` against Rust `space_id`. One example is one example.
- **(g) 🛑 THE `unresolved` GREP TRAP HAS THREE CONCEPTS**, not two: the roster marker, `echo-status`'s send-status tone, and the **dock leaf** in `app_client.svelte` (`:451 :473 :485 :501`). A file-level hit count on `app_client.svelte` reads as 5 marker hits; **one** is. **Scope the sweep before believing it.**

---

## §8 — Scope: what must NOT be touched

- ❌ `ui/assets/skin.css` — **Joe's.** C-3 (the `[data-unresolved]` base rule + `unasked` variant) is **Leg C's** remaining third, ungated since J-670 and **not this leg**.
- ❌ `setResult` / `setFailed` / `setInflight` / `reset` — their late-guards and `_notFound` clears are unchanged.
- ❌ `removeMember`, `routeMembershipEvent`'s leave/kick/ban/node_eject arms, `ingest.push`, `R4`'s sync-from-cursor replay.
- ❌ `roomLatch.effectiveSpaceId` — 🛑 **`N-169`: it is an unmemoised getter ON PURPOSE.** Any caller of `setSpaces` triggers a members re-fill, and **`M-RP-LIVEFEED-REFRESH` Leg C and identity-resolution Leg E both depend on that cascade firing.** Memoising it without replacing the members trigger silently un-builds both. **It is architecture and it is Joe's.**
- ❌ `entity-item` / `entity-panel` — Leg B already threaded the prop.

---

## §9 — DoD (Leg D)

✅ **LEG D LANDED 2026-08-04. `aa7d9c9` (D-i, Clair, 2 files, +109/−2) · `9901036` (D-ii, Clair, 2 files, +49/−6). Pushed by Joe; every gate RE-DRIVEN by Chat on the COMMITTED tree at `9901036`.**

| # | gate | result, re-driven at `9901036` |
|---|---|---|
| **V0** | cargo before the first `.rs` edit | **1595 / 0 / 62 × 56** — Clair, measured not inherited |
| **V1** | cargo after D-i | ✅ **1596 / 0 / 62 × 56**, `FAILED` case-sensitive **0**, `^error` **0** — Δ exactly **+1** |
| **V2** | the Δ by name | `desktop::pass_4_commit_1_tests::fetch_identity_return_serde_transparent_to_js_frontend ... ok` — the only add |
| **V3** | the test proven able to fail | ✅ literal flipped → **1 failed**, panic at the assertion; reverted, zero leftovers |
| **V4** | slot gate, **CLEAN TREE** | ✅ **PASS 74 (65 / 5 / 3 / 1)** — unchanged. 📌 *Clair's own V4 ran `-AllowDirty` and the script itself says its numbers are not quotable; **this** run is the quotable one, and the guard doing its job is why* |
| **V5** | `svelte-check` before the first `ui/**` edit | **0 / 34 / 15** (255 files) — freshly measured, **not** the inherited `87307e8` figure |
| **V6** | `svelte-check` after D-ii | ✅ **0 / 34 / 15**, Δ **0** |
| **V7** | scope | ✅ D-i = `{desktop.rs, ops.rs}`, zero `ui/**` · D-ii = `{app_client.svelte, address-book.svelte.ts}`, zero `.rs` — disjoint |
| **V8** | the split reproduces | 🔑 **PROVEN, NOT ARGUED — see below** |

🔑 **V8 UPGRADED FROM AN ARGUMENT TO A MEASUREMENT.** Clair reported it *"satisfied by construction"*. Measured: `git diff --name-only aa7d9c9 9901036 -- '*.rs'` returns **empty** — **D-ii changed ZERO `.rs`** ⇒ ***the Rust source at `9901036` is byte-identical to D-i's committed tree***, so V1's re-drive at HEAD **is** a measurement of D-i's tree rather than an inference about it. ✅ **FORWARD FORM, GENERAL: a commit split whose halves touch DISJOINT FILE SETS is PROVEN by diffing the halves for the other half's file types — no checkout, no second full run, and no "by construction" required.** *A split not proven to reproduce the tested tree has tested nothing (J-670); this is the cheap proof.*

✅ **AND THE `svelte-check` Δ IS A REAL ARGUMENT, NOT A NUMBER MATCH:** a new warning in a previously-clean file moves **15 → 16**; a new warning in an already-warning file moves **34 → 35**; and D-ii is **pure addition**, so nothing could have been removed to mask one. **Both figures unchanged ⇒ no new warning.**

- [x] **D-i** — `fetch_identity` + registration + the witness test, **one commit, zero `ui/**`** — `aa7d9c9`
- [x] V0…V4 green; the cargo Δ is **`+1`, named**, and the new test was **proven able to fail**
- [x] **D-ii** — `resolveMember` + `addMember`'s return + the router hook, **one commit, zero `.rs`** — `9901036`
- [x] V5…V8 green; the `svelte-check` Δ stated against a **freshly measured** baseline
- [x] `M_RP_IDENTITY_RESOLUTION.md` §5b annotated — ✅ landed at `304742b`
- [x] `M_RP_IDENTITY_RESOLUTION.md` §8 Leg D updated with the outcome; §9's citations annotated
- [x] **`G-B` TICKED** — Leg E discharged J-670, Leg D landed here; `N-168` satisfied by the pair
- [x] Leg F's obligation list grown by this leg's three unverifiable cases **and** the join-concurrency count
- [x] Records: JOURNAL + `CLAUDE.md` PLAY + `ROADMAP.md` + the milestone doc + this runbook in one commit (`D-074`)
- [x] 🛑 **Clair handed back with the numbers and did not close her own leg. Chat re-drove every gate. Joe pushed.**

---

## §10 — WHAT THE IMPLEMENTATION FOUND THAT THE RUNBOOK DID NOT KNOW

### ✅ §7-b's deliberate hole, filled by grounding

**`chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)`** — the exact producer `fill_from_events` uses at **`ops.rs:2913`**, whose value flows into `absorb_fetch(book, fetched, &now)` at **`:2947`**. 🔑 *The hole was marked rather than guessed precisely so this would be a lookup instead of a coin-flip, and it was.* **No second timestamp format enters `last_seen`.**

### ✅ §7's other six items — all grounded, all held

(a) `SeenRecord` derives `Serialize` (`address_book.rs:79`) and already crosses IPC inside `get_address_book`'s map · (c) `absorb_fetch` was private, now `pub(crate)`, **not `pub`** · (d) `book.get` takes `&str` (`:203`) and `IdentityXgid: Borrow<str>`, so `&String` coerces · (e) 🔑 **`addMember` has exactly ONE call site across all of `ui/**` INCLUDING the sampler** — the sweep §7 demanded, and the `void`→`boolean` change is genuinely additive · (f) the `{ identityId }` camelCase mapping matches `:222`'s precedent · (g) only the roster-marker `unresolved` was touched; the dock leaf and `echo-status` are untouched.

### 🔒 R-D3's PARENTHETICAL IS ANNOTATED; THE CODE STANDS (`D-131`)

Clair flagged a real mismatch: §5-1(b)'s code guards the **whole function** on `spaceId !== _spaceId`, so the `_book` merge is scoped too — while `R-D3`'s aside read *"the `_book` half is scope-free — the book is a global cache keyed by identity."*

🔒 **RULED (Chat): THE CODE IS THE INTENT. `R-D3`'s note was an OBSERVATION about what `_book` is, never an instruction to leave the merge unguarded.** Annotated, not changed.

🔑 **AND CLAIR'S CONSEQUENCE WAS RIGHT IN DIRECTION AND WRONG IN MECHANISM — IN THE SAFE DIRECTION.** She wrote: *"a harmless missed cache entry, re-fetched by the next fill."* **Measured: it is NOT re-fetched.** D-i persisted the record to disk **before** the frontend guard ever ran, so the next `get_address_book` returns it, and §5b's rule — *once the book holds someone it never asks again* — means the fill does not re-fetch at all. ⇒ ***THE WHOLE-FUNCTION GUARD IS SAFE BECAUSE §4 LOCKED B2.*** Under B1 the late-resolving record would have been **lost outright**. 🔑 **That is a SECOND consequence of B2 that nobody priced when B2 was chosen** — the lock is load-bearing in a place its own derivation never looked.

### 📌 EOL, recorded so it is not misread later

`ops.rs`, `app_client.svelte` and `address-book.svelte.ts` are **CRLF in the working tree via `core.autocrlf`** and **LF in the index** (`i/lf` on all four touched files); `desktop.rs` is LF throughout. **Pre-existing, the J-643 shape, not introduced by this leg** — and the index form is what ships. All four diffs are clean localized hunks with no whole-file EOL churn.
