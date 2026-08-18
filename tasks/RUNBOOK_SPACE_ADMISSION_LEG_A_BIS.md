# M-SPACE-ADMISSION Leg A-bis Runbook — leg ①: the before-assertion, and the fixture that had to be corrected twice to get it
> **Status**: ACTIVE  
> Version: 1.3  
> Date: Aug 2026  
> **Last updated**: 2026-08-18  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS IS, AND WHAT IT IS NOT

**Implements `tasks/M_SPACE_ADMISSION_PHASE0.md` v1.7 §12 Leg A-bis, LEG ① ONLY.** One integration test in
`xgen-node`, asserting **today's** admission behaviour for a third-party `membership.join` submitted
locally against a DM. **No production code. No `SpaceState` field, no event type, no gate** — those are
Legs B, C and D.

🛑 **LEG ② IS NOT IN THIS RUNBOOK AND ITS ABSENCE IS DELIBERATE.** Leg ② is the two-node
concurrent-set divergence test for §3.2. It tests **the sibling's** divergence
(`member_temperature_visibility` / `human_ai_pacing_ms`), because **admission is not built** — there is no
`state.space_admission` event to set concurrently. **It is deferred to after `Leg E-0`** and gets its own
runbook. *Folding it in here would produce a runbook half of which cannot be executed against anything.*

### 🔒 THE ONE THING THAT MAKES THIS LEG URGENT RATHER THAN TIDY

**LEG ① MUST LAND BEFORE LEG D.** Leg D ships the admission gate. Under `D-148` clause 4 **a DM pins
`invite`**, so once the gate exists **a DM can never again be observed admitting an uninvited third party** —
there is no configuration that restores the pre-gate state, and no later session can produce this
measurement. 🔑 ***It is a prerequisite of the fix having a proof, not polish applied after it.***

### 🔒 THE LOCKS THIS RUNBOOK IMPLEMENTS AND MAY NOT RE-OPEN (`D-123` Rule 6)

| lock | ruling | site |
|---|---|---|
| **the actor** | **A SECOND *REGISTERED* IDENTITY, NOT A FRESH KEYPAIR** | Phase-0 §12 A-bis, J-748 |
| **the shape** | a **before-assertion** — assert today's admission first; the after-assertion belongs to Leg D | Phase-0 §12 A-bis |
| **the name** | `tasks/RUNBOOK_SPACE_ADMISSION_LEG_A_BIS.md` | Joe, 2026-08-16 |
| **`D-148` cl. 4** | **a DM pins `invite`** — which is why the before-state is unrecoverable after Leg D | `DECISIONS.md` D-148 |
| **§4.6** | **the companion open-Space test SHIPS** — locked on Chat's recommendation | Joe, 2026-08-17 |
| **§4.1 names** | **KEPT as scaffolded** — file, `mod.rs` line and both test fns stand | Joe, 2026-08-17 |

📌 **State this runbook was authored against:** `HEAD` `f728cb4` **= `origin/main` by `git ls-remote`**,
clean tree. Every `H-` and `X-` row below was measured this session at that commit.

---

## §1 — 🔑 WHY THE FIXTURE WAS WRONG TWICE, STATED ONCE, SO IT IS NOT WRONG A THIRD TIME

**J-742 said the bound was multi-tenancy. J-743 (Clair) said it was not tenancy and the actor needs no
registration. J-748 said tenancy, with a measured reason none of the earlier framings had.**

🛑 **The J-743 instruction — *"must use a fresh unregistered keypair or it tests a narrower thing than the
hole"* — was exactly backwards.** `exchange.rs:601-634` Step 11 HeldPends an **unregistered** sender
universally; `MembershipJoin` is **not** exempt (`skip_membership` covers the *membership* check in the
block **below** it). ⇒ **a fresh-keypair fixture would assert `HeldPending`, go green, and prove nothing
about admission.**

🔑 ***A check whose failure mode reads exactly like success — minted by a correction written to prevent
one.*** **This runbook's entire §2 exists to make that impossible to reproduce: every gate on the path is
enumerated below, with what it does to THIS fixture, so the fixture is built against a partition and not
against a memory.**

---

## §2 — 🔒 THE GATE INVENTORY ON THE LOCAL JOIN PATH. **RE-DRIVEN AT `f728cb4`. THIS IS THE RUNBOOK'S SPINE.**

**Every gate a `membership.join` passes between `dispatch_event` entry and `apply_join`, in order, with its
verdict for this fixture.** 📌 **Metric:** read by opening each cited span, not by grep count.

| # | gate | site | verdict for this fixture | status |
|---|---|---|---|---|
| **X-0** | `server_authenticate` → `verify_auth_response` → `is_revoked` | `transport/connection.rs:523-576`, `transport/auth.rs:91-123`, `identity/registry.rs:184-189` | 🛑 **NOT ON THE PATH AT ALL IN THIS HARNESS** — `submit_locally` calls `dispatch_event` directly. **Named as a bound in §6, not silently skipped** | ✅ carried (J-743/J-748) |
| **X-1** | **Step 11 — registration.** `if !fed_add_via_federation && !node_authored && !id_registry.contains(sender) { HeldPending }` | `exchange.rs:601-634`, consumer `runtime.rs:1339-1361` | 🔑 **THE GATE THAT DICTATES THE FIXTURE.** Carol **must be registered** or the test asserts `HeldPending` and proves nothing | ✅ **RE-DRIVEN** |
| **X-2** | banned check | `runtime.rs:1522-1531` | Carol is not in `banned` ⇒ passes | ✅ **RE-DRIVEN** |
| **X-3** | 🛑 **THE TIER GATE — NEW THIS SESSION, NAMED IN NO KICKOFF AND IN NO PHASE-0 SECTION.** `verify_tier_assertion(joiner_tier, space.auth_tier)`, where `joiner_tier = identity_registry.get(sender).map(assertion_tier_of).unwrap_or(1)` | `runtime.rs:1532-1544`; `assertion_tier_of` at `:241` | **PASSES AS A NO-OP, AND ONLY BECAUSE TWO SEPARATE FACTS HAPPEN TO AGREE:** `build_dm_space_create_event` writes **`"auth_tier": 1`** (`state.rs:1812-1833`), and `make_identity_record` sets **`trust_assertion: None`** ⇒ `assertion_tier_of` → 1 ⇒ `verify_tier_assertion(1, 1) = Ok`. ⚠️ **Neither fact is guaranteed by anything the fixture asserts** — see `H-6` | ✅ **NEW, THIS SESSION** |
| **X-4** | invite-expiry gate (INV-EXP) — outer guard `if origin == LocallySubmitted && room_id.is_empty()` (`:1580`), then `if let Some(pi) = space.pending_invites.get(&event.sender)` (`:1586`) | `runtime.rs:1580-1610` | **THE INNER `if let` IS FALSE FOR CAROL ⇒ THE WHOLE BLOCK IS SKIPPED.** ⚠️ **AND THAT IS A TRAP, NOT A CONVENIENCE** — see `H-5`. 📌 *Span corrected in v1.2: v1.1 cited `1583-1610`, which began three lines BELOW the gate's own outer guard (Clair `F-6`).* | ✅ **RE-DRIVEN** |
| **X-5** | `skip_membership` — membership + permission steps | `exchange.rs:649-659` | `MembershipJoin` is in the set ⇒ both skipped | ✅ carried (G-1) |
| **X-6** | `check_permission` | `exchange.rs:914` | catch-all `_ => Ok(())` ⇒ gates joins on nothing | ✅ carried (G-4) |
| **X-7** | `apply_join`, space-level arm | `state.rs:1016`, `:1019`, `:1022-1025` | **exactly two guards** (already-member, banned), then `match pending_invites.remove(joiner) { … None => (Role::Member, None) }` ⇒ **`Role::Member`, `invited_by: None`** | ✅ **RE-DRIVEN** |

🛑 **⇒ THE PARTITION CLAIM WAS FALSE AND IS WITHDRAWN (v1.2 — Clair's cold read `F-1`, re-driven by Chat under Rule 5).** v1.1 offered X-0…X-7 as **the complete set of gates between `dispatch_event` entry and the `members` insert**.

✅ **MEASURED — AND THE METRIC IS STATED, BECAUSE THE TWO SEATS GOT TWO NUMBERS AND BOTH ARE RIGHT (v1.3, Clair's second pass):**
- **35 RAW RETURN SITES** — **19** `return DispatchOutcome::` in `runtime.rs:1120-1758` **+ 16** `return ValidationOutcome::` in `xgen-core/src/message/exchange.rs:489-740`.
- **33 DISTINCT GATES** — the same set with **`runtime.rs:1337` and `:1361` counted once, not twice**: they are the `match validate_event(...)` **arms**, the funnel through which all 16 `exchange.rs` outcomes return. **A fixed offset of 2.**

🔑 **BOTH FIGURES ARE CORRECT UNDER THEIR OWN CONVENTION AND NEITHER IS A DISCREPANCY — which is exactly why the convention has to sit next to the figure.** Chat wrote **35** (raw sites); Clair wrote **≈33** (distinct gates); **each document stated its own metric and neither stated the other's.** ⚠️ ***`J-628`'s species: two correct measurements that read as a contradiction because the conventions were never placed side by side*** — cheap to name here, a session to rediscover later. 📌 **§2's rows are GATES, so 33 is the figure this table is a subset of.** 🛑 **Whichever is cited, cite the convention with it — the standing floor rule (`cargo` = × SUITES, `vitest` = × FILES) generalises: a count without its convention cannot be compared to the next one.**

**§2 lists seven.**

🔒 **WHAT §2 ACTUALLY IS, STATED HONESTLY: THE ADMISSION GATES — the checks that decide WHETHER A JOIN IS ADMITTED. It is NOT every gate on the path.** ✅ **As a partition of the admission gates it survived the cold read intact**; what failed was the width of the sentence around it.

⚠️ **THE SIX UNLISTED GATES THAT MATTER, BECAUSE EACH FAILS WITH A DIAGNOSIS THAT IS NOT ABOUT ADMISSION:** `runtime.rs:1169` **space not found** · `exchange.rs:571` **`validate_dag_structure` DagError** — 🔑 **Step 10 runs BEFORE Step 9's predecessor lookup exactly so malformed `prev_events` fail synchronously** · `:532`/`:538` missing / mismatched `event_id` · `:554`/`:561` timestamp out of bounds · `:686` signature failure · and **`check_ai_capability`, which runs unconditionally**. ⇒ **§4.3 gains two DAG preconditions (v1.2), because v1.1's preconditions derived entirely from the X-rows and asserted NOTHING about the DAG.**

🔑 **FIFTH INSTANCE IN THIS ARC OF *a census wearing a partition's clothes* — and the second time inside a section written to prevent it.** §2's own v1.1 text predicted this defect by name, invited Clair to attack it, **and was still wrong.** ***Predicting a failure mode is not the same as being immune to it; the prediction is worth exactly the audit that follows it.***

---

## §3 — 🔑 THE GROUNDING THIS RUNBOOK ADDS BEYOND THE PHASE-0

| # | fact | site | why it changes the plan |
|---|---|---|---|
| **H-1** | 🔑 **`InProcessNode::submit_locally(ev) -> DispatchOutcome` ALREADY IS THE LEG'S SUBJECT PATH** — its body is `rt.dispatch_event(ev, EventOrigin::LocallySubmitted, None)` | `phase9_harness.rs:516-576` | ⇒ **the fixture needs NO new harness and NO hand-rolled `dispatch_event` call.** `peer_node_id = None` is exactly F2-2's mapping, so **F-3 is structurally skipped without the test having to arrange it** |
| **H-2** | ✅ **`ingest(ev)` bypasses `dispatch_event` entirely** (`rt.ingest_event` + persist) and is the sibling's setup primitive | `phase9_harness.rs:578-604` | ⇒ 🔒 **ALL SETUP USES `ingest`; ONLY THE SUBJECT EVENT USES `submit_locally`.** *A setup step that goes through the path under test makes the test's own scaffolding part of the claim* |
| **H-3** | ✅ **`ingest_event` builds `SpaceState` for a DM** — `EventType::StateSpaceCreate \| EventType::StateDmSpaceCreate` share the create-arm | `runtime.rs:823-832` | ⇒ ingesting `build_dm_space_create_event` yields a live DM `SpaceState` with `dm_constraints_active: true` (`state.rs:463`, `:578`) |
| **H-4** | ✅ **The read-back accessors exist:** `space_state(space_id) -> Option<SpaceState>`, and `SpaceState::member_role(&str) -> Option<&Role>` / `is_member(&str)` | `phase9_harness.rs:651`; `state.rs:1277`, `:1284` | ⇒ **the role assertion is a direct read, not an inference from `DispatchOutcome`.** Both are asserted — see §5 |
| **H-5** | 🛑 **THE DM COUNTERPART HAS A PENDING INVITE AND THE INTRUDER DOES NOT — AND X-4 BRANCHES ON EXACTLY THAT.** `from_dm_space_create_node` seeds the invitee as a `PendingInvite`; the DM arm of X-4 (`None => {}`, *"DM-seeded invite: exempt by design"*) exists **for the counterpart** | `state.rs:496-540`; `runtime.rs:1603-1608` | 🔑 ⇒ **IF THE FIXTURE'S ACTOR WERE BOB (the counterpart), X-4's INNER BLOCK WOULD RUN AND THE TEST WOULD BE MEASURING THE INVITE-EXPIRY GATE INSTEAD OF ADMISSION.** ***The third identity is not decoration; it is what keeps the subject on the intended gate.*** **Carol must be neither the creator nor the invitee, and the test must assert that** |
| **H-6** | ⚠️ **X-3 PASSES ON A COINCIDENCE OF TWO CONSTANTS AND NOTHING PINS EITHER** — `"auth_tier": 1` in the DM builder and `trust_assertion: None` in `make_identity_record` | `state.rs:1812-1833`; `phase9_harness.rs:1263-1278` | ⇒ 🔒 **THE FIXTURE ASSERTS `space.auth_tier == 1` BEFORE SUBMITTING.** *If a later change raises the DM default tier, the join would reject with `3030` and the test would fail with a message about admission — a wrong diagnosis, silently, forever.* **One line buys a correct failure message** |
| **H-7** | 🛑 **THE HARNESS DOES NOT GO THROUGH `accept_registration` AT ALL.** `InProcessNode::register_identity` calls `NodeRuntime::register_identity(record)` directly with a `make_identity_record` whose `trust_assertion` is `None` | `phase9_harness.rs:501-506`, `:1263`; cf. `identity/registration.rs:444-512` | 🔑 **⇒ THE ANSWER TO *"WHICH OF J-748's THREE DEPLOYMENT STATES IS THE HARNESS IN?"* IS: NONE OF THEM.** **No `AssertionPolicy` is consulted, because the function that consults it is never called.** ⇒ the fixture registers successfully **by construction, not by policy** — see §6.1, where this is written down as a bound rather than glossed |
| **H-8** | ✅ **The test module list is hand-maintained** — a new file needs a `pub mod` line | `xgen-node/src/tests/mod.rs` | ⚠️ **`mod.rs` is the same species as `known_variants()`: omitting the line does not FAIL, it silently omits the test.** *A check whose failure mode reads exactly like success is not a check.* **The cargo delta in §5 is what catches it** |
| **H-9** | ✅ **The `cargo` floor is `1602 / 0 / 62 × 56 SUITES`**, carried across fourteen consecutive no-code sessions | Phase-0 §11 | ⇒ 🔒 **`cargo test` MUST MOVE.** *This is the inverse of the M-RP6.1i/6.1j leg: there, an identical count PROVED no Rust landed; here, an identical count proves the test did not run* |

---

## §4 — THE FIXTURE. **CLAIR IMPLEMENTS.**

### §4.1 — 🔓 Names — SCAFFOLDED, JOE'S TO OVERRULE (`D-138`: ship a plausible value, never blank)

- **file:** `xgen-node/src/tests/space_admission_third_party_join.rs`
- **`mod.rs` line:** `pub mod space_admission_third_party_join;`
- **test fn (§4.4):** `third_party_registered_identity_joins_a_dm_it_is_not_party_to`
- **test fn (§4.6):** `third_party_registered_identity_joins_an_open_space`

🔒 **KEPT AS SCAFFOLDED (Joe, 2026-08-17).** Naming is Joe's seat; these shipped plausible rather than blank (`D-138`), and they stand.

### §4.2 — Setup — **`ingest` ONLY (H-2)**

1. `let node = spawn_in_process_node().await;`
2. Three keypairs via `keypair::generate()`: **alice** (DM creator), **bob** (DM counterpart), **carol**
   (the third party). Ids via `pubkey_uri`.
3. **Register alice and carol** via `node.register_identity(&key).await`. 🔒 **Carol's registration is the
   X-1 requirement and is the single most load-bearing line in the fixture.**
   📌 **Bob is registered too** — he emits no event, but an unregistered counterpart makes the DM's own
   state a second variable, and *a fixture with one unexplained asymmetry invites the next reader to
   assume it is meaningful.*
4. `let dm_ev = sign_event(build_dm_space_create_event(&alice_key, &bob_id, &node.node_id), &alice_key);`
   `let space_id = event_id_str(&dm_ev);` `node.ingest(dm_ev).await;`

### §4.3 — Preconditions — **ASSERTED, NOT ASSUMED**

Before the subject event, assert on `node.space_state(&space_id).await.expect(…)`:

- `state.is_dm` **and** `state.dm_constraints_active` — *the test claims to be about a DM; it should say so
  to the compiler.*
- 🔒 **`state.auth_tier == 1`** — `H-6`. Without this, an X-3 rejection reads as an admission result.
- 🔒 **`!state.is_member(&carol_id)`** and **`state.pending_invites`** does **not** contain carol — `H-5`.
  ***This is the line that proves the subject is on the admission path and not on the invite-expiry
  path.***
- `state.is_member(&alice_id)` — the DM has its creator, so the Space is real and not an empty shell.
- 🔒 **TWO DAG PRECONDITIONS — NEW IN v1.2, FROM Clair `F-1`.** ① **`node.space_state(&space_id).await.is_some()`** — the Space resolves, or `runtime.rs:1169` rejects with **space-not-found** and the failure reads as if admission refused it. ② **`!tips.is_empty()`** on the `dag_tips` read of §4.4 — `validate_dag_structure` (`exchange.rs:571`, **Step 10, which runs BEFORE Step 9's predecessor lookup**) rejects malformed or empty `prev_events` with a `DagError`. 🔑 **v1.1's preconditions derived entirely from the X-rows and therefore asserted NOTHING about the DAG** — ***a fixture cannot be verified only against the gates its author enumerated.*** 📌 **Two lines, and they convert two of the six unlisted gates from silent misdiagnoses into loud ones.**

### §4.4 — The subject event

```
let tips = node.dag_tips(&space_id).await;
let carol_join = sign_event(
    Event::new(
        EventType::MembershipJoin,
        idx(&carol_id),
        rdx(""),                       // space-level join — room_id MUST be empty (X-4, X-7)
        sdx(&space_id),
        tips.iter().map(|t| edx(t)).collect(),
        now_rfc(),
        json!({}),
    ),
    &carol_key,
);
let outcome = node.submit_locally(carol_join).await;
```

🛑 **`rdx("")` is load-bearing.** A non-empty `room_id` takes `apply_join`'s **room-level** arm, which
**requires existing Space membership** (`state.rs:1002-1005`) — the test would then assert
`NotASpaceMember` and read as if a gate existed.

### §4.5 — The assertions — **THREE, AND EACH ANSWERS A DIFFERENT QUESTION**

1. **`matches!(outcome, DispatchOutcome::Accepted { .. })`** — *the dispatch path admitted it.*
   ⚠️ **`Accepted` carries `{ new_joiner, additional_persisted }`; match with `{ .. }`, never positionally.**
2. **`node.space_state(&space_id).await.unwrap().member_role(&carol_id) == Some(&Role::Member)`** —
   *the state records her as a full member.* 🔑 **Assertion 1 alone is not enough: `Accepted` is an outcome
   of the dispatch, and `Role::Member` is the fact the milestone is about.**
3. **`invited_by` is `None`** on carol's `SpaceMember` — 🔑 ***this is the assertion that names the hole.***
   She is a member **and nobody invited her**, which is `X-7`'s `None => (Role::Member, None)` arm made
   visible. *Without it the test could be satisfied by a build in which some invite mechanism admitted
   her, and the reader could not tell.* 📌 **Accessor (v1.2, Clair `F-5`): `H-4` named only `member_role` and `is_member`, neither of which reaches `invited_by`. Read it off `state.members.get(&carol_id)` — `SpaceState::members` is `pub` (`state.rs:221`) and `SpaceMember::invited_by` is a field (`:74`).**
4. 🔒 **`!state.is_member(&bob_id)` — NEW IN v1.2, FROM Clair `Q4`, AND IT IS THE STRONGEST OF THE FOUR.**
   **The DM's ACTUAL COUNTERPART is not a member while the STRANGER is.** Membership goes
   `{alice} → {alice, carol}` and `is_dm` stays `true`. 🔑 **Assertions 1–3 record that an uninvited party
   was admitted; THIS one records that the two-party invariant of a DM is not enforced by anything at
   all** — the Space is now a three-name DM whose named second party never joined. ⚠️ **Unobservable after
   Leg D, and recorded in no other test.** ***Chat wrote three assertions about the intruder and none
   about the DM; the missing one was about the thing the DM is FOR.***

### §4.6 — 🔒 THE COMPANION TEST — **LOCKED (Joe, 2026-08-17). SHIPS.**

**The DM test above INVERTS at Leg D and is a one-way witness (§0).** A **second test in the same file** runs the identical third-party join against an **ordinary Space** (`build_space_create_event`, `auth_tier` 1), which under `D-148` clause 3 **defaults to `open` forever** ⇒ **it stays green through Leg D and afterwards.**

🔑 **THE ARGUMENT THAT CARRIED IT: after Leg D the DM test is edited into its opposite, and at that moment the project has NO live assertion that open-join still works** — which is `J-275`'s model and `L-E`'s whole ground. ***The DM test is the perishable witness; the companion is the permanent lock.***

**`D-121` as recorded before the lock:** ① **user-visible impact: none** — a test. ② **tier consequence: none** — nothing is copied, nothing erased. ③ **resource: ~30 lines in a file being created anyway.**

#### 🔓 Name — SCAFFOLDED (naming is Joe's; `D-138`)

**test fn:** `third_party_registered_identity_joins_an_open_space`

#### The shape — **IDENTICAL TO §4.2–§4.5 EXCEPT WHERE STATED**

🛑 **TWO CLAIMS IN THE FIRST DRAFT OF THIS SECTION WERE FALSE AND ARE CORRECTED HERE RATHER THAN ANNOTATED** — the section was PENDING and had no downstream reader when they were caught (`D-145`'s test), and **both fell the moment the source was opened, neither to any re-read.** *Chat's own, in the section Chat had just recommended, thirty minutes after writing it.* **They are named because the pattern is the arc's:** ***a claim narrower — or simply other — than the thing it describes.***

1. Setup: **alice** (creator) + **carol** (third party), both registered. 📌 **No bob** — an ordinary Space has no counterpart, and adding one would import the DM's shape into a test whose whole point is that it is not a DM.
2. `build_space_create_event(&alice_key, "<name>", None, 1, &node.node_id, None, false)` → `ingest`. 🛑 **CORRECTED: the trailing `false` is `e2e_encryption`, NOT `is_dm`** (`state.rs:1381-1389` — the seven params are `key`, `name`, `topic`, `auth_tier`, `home_node`, `jurisdiction`, `e2e_encryption`). **`is_dm` is not a parameter at all:** `from_space_create` hardcodes `is_dm: false` and `dm_constraints_active: false` (`state.rs:317`, `:330`). ⇒ **the two preconditions still ship — they are cheap and they document the test's subject — but they are STRUCTURALLY GUARANTEED, not a risk being guarded**, and the runbook must not imply otherwise.
3. 🛑 **CORRECTED, AND THIS ONE WOULD HAVE COST A WRONG SETUP STEP: ALICE DOES NOT NEED TO JOIN.** The first draft said `from_space_create` leaves `members` empty. **It does not** — `state.rs:290-297` builds a `SpaceMember { role: Role::Owner, invited_by: None }` for `event.sender` and inserts it before the struct is returned. ⇒ **the creator is a member with `Role::Owner` the instant the create Event is ingested; no `MembershipJoin` for alice is required or wanted.** ⚠️ **AND THE SIBLING IS NOT THE MODEL HERE:** `phase9_unknown_signer_first_contact.rs` ingests an invite **and** a join for alice **who is already the owner-member** — through `ingest`, which bypasses `dispatch_event`, so whatever `apply_join`'s already-member guard (`state.rs:1016`) returns is swallowed. ***Copying that setup would have added two events that do nothing and one of which is an error nobody sees.*** 🔑 **The general lesson, and it is why §8 item 1 exists: a sibling test is a model for the HARNESS SHAPE, never for the SEMANTICS — it was written to reach a different gate.**
4. Preconditions: `state.auth_tier == 1` (`H-6`), **`state.member_role(&alice_id) == Some(&Role::Owner)`** (the create seeded her — assert what is actually true), `!state.is_member(&carol_id)`, `pending_invites` does not contain carol.
5. Subject event and all three assertions of §4.5, unchanged.

🔒 **AND ITS OWN DoD CLAUSE, WHICH IS THE POINT OF IT: THIS TEST IS NOT TOUCHED BY LEG D.** Leg D inverts the DM test **only**. 🛑 **If Leg D finds it must also weaken or edit this one, that is a FINDING about the gate's scope — the gate would be refusing an open join — and it is reported, never absorbed.** ***A companion that gets edited alongside the thing it was built to outlive was never a control.***

---

## §5 — VERIFICATION. **CHAT DRIVES ALL OF IT (Rule 5).**

| gate | what | expected |
|---|---|---|
| **V-1** | `cargo test --workspace` — **detached, log to file, own exit sentinel appended** (it exceeds the MCP timeout) | 🔒 **MUST MOVE from `1602 / 0 / 62 × 56 SUITES`** → **`1604`** (two tests: §4.4 and §4.6). 📌 *v1.2, Clair `F-4`: v1.1 still offered "1603 or 1604" after §4.6 was locked — **a dead branch left in a gate is a gate that accepts a wrong number.*** ⚠️ **Summed programmatically, never hand-counted; the final `test result:` line must be present** — *a killed detached run leaves a measurement-shaped artefact* |
| **V-2** | the new tests named in the run output, and the **suite** count | 🔒 **56 SUITES — UNCHANGED, AND STRUCTURALLY SO.** `xgen-node/src/lib.rs:42` is `#[cfg(test)] pub mod tests;` ⇒ a file under `src/tests/` compiles into the **existing lib test binary**, not a new one. 🛑 *v1.2, Clair `F-3`: v1.1 offered "**57** if a new binary is produced" and told the driver to measure it — **but 57 is not a possible reading, so offering it as a live branch invited a false alarm and made the gate unable to distinguish a real anomaly from the expected case.*** ✅ **A CHANGED suite count is now a FINDING, reported under §8.** *Stating a unit stays mandatory: cargo = × SUITES* |
| **V-3** | 🔒 **THE NEGATIVE CONTROL — REBUILT IN v1.2 BECAUSE v1.1's VERSION COULD NOT FAIL.** Run the fixture with carol **UNREGISTERED**, once, by hand, and discard it | 🛑 **`DispatchOutcome::HeldPending` IS A UNIT VARIANT — IT CARRIES NO REASON.** `ValidationOutcome::HeldPending { missing_predecessors, missing_identity }` **does**, and `runtime.rs:1341-1361` **binds both, hands them to the `PendingBuffer`, and returns the bare unit variant** ⇒ ***an unregistered sender and a missing predecessor produce the IDENTICAL return value.*** ⚠️ **So v1.1's control would have passed on a fixture with wrong `prev_events`** — **the exact defect it was written to expose.** 🔒 **THE FIX, PRECEDENTED AND NOT INVENTED: assert `pending_identity_count()` INCREMENTED** (`dag/pending.rs:556`, `pub`, reachable via `InProcessNode.runtime`) — **`phase9_unknown_signer_first_contact.rs:91`/`:238` already does exactly this.** ➕ **RUN V-3 ONLY AFTER V-1 IS GREEN**, so a compile or harness fault cannot present as a registration hold |
| **V-4** | ✅ scope, by diffstat | **exactly two files** — the new test file and `mod.rs`. **Zero `ui/**`, zero `xgen-core`, zero `xgen-common`, zero `xgen-client`** ⇒ vitest and svelte-check floors are **carried by scope, not re-run** |
| **V-5** | ✅ `git ls-files --eol` on both files | **`i/lf`** — everything but `CLAUDE.md` and `docs/ROADMAP.md` is LF |

🛑 **A FLOOR IS NEVER CITED WITHOUT ITS UNIT.** cargo = **× SUITES** · vitest = **× FILES**.
🛑 **The sampler catalogue is UNMEASURED and its harness has never been located. DO NOT WRITE 435.**

🔑 **AND THE LESSON V-3 CARRIES OUT OF THIS RUNBOOK, BECAUSE IT IS BIGGER THAN V-3: the discriminator that makes the control real was sitting in `phase9_unknown_signer_first_contact.rs` — THE FILE §0 NAMES AS "READ IT FIRST".** Chat wrote a step, called it *"the only step that proves the J-743 fixture would have been a false pass"*, **and did not check whether that step could itself fail.** ⇒ ***`N-197` rule ② — run the negative control — failed on the step written to enforce `N-197` rule ②.*** 📌 **The finding is Clair's, from the cold read, and it is the sharpest thing either seat produced on this leg.**

---

## §6 — WHAT THIS TEST DOES **NOT** PROVE. **STATED BEFORE IT IS RUN, NOT AFTER.**

📌 *The Phase-0's §4 bound 1 says the routing is a source trace and the exploit is not run. **This leg
discharges that for the LOCAL path and for nothing else**, and the boundary is written here so the ROADMAP
node can be updated with the correct width rather than the generous one.*

1. 🛑 **IT DOES NOT EXERCISE `accept_registration`, AND THEREFORE NONE OF J-748's THREE DEPLOYMENT STATES**
   (`H-7`). The harness registers by calling `NodeRuntime::register_identity` directly. ⇒ **the test proves
   what a REGISTERED Identity can do; it says nothing about how hard it is to become one.** **That is the
   entire content of the hole's SIZE**, and it stays a source finding.
2. 🛑 **IT DOES NOT EXERCISE THE WIRE.** `X-0` — `server_authenticate`, the WebSocket listener,
   `is_revoked` — is not on the harness path. **The session-establishment half of `F-1` remains a source
   trace.**
3. 🛑 **IT IS NOT AN EXPLOIT AGAINST A RUNNING NODE.** In-process, one node, no network. **`§9`'s bar
   against recording §4 as an exploit still binds after this leg lands.**
4. 🛑 **IT PROVES NOTHING ABOUT FEDERATION.** `peer_node_id = None` throughout; F-3 is never evaluated.

🔒 **The honest one-line summary the ROADMAP node should carry:** ***a registered third-party Identity is
admitted to a DM it is not party to, as `Role::Member`, with no invite — measured in-process on the local
dispatch path.***

---

## §7 — WHAT THIS RUNBOOK MUST NOT DO

1. 🛑 **Must not add the gate.** That is Leg D. **A test that ships beside its own fix cannot record a
   before-state.**
2. 🛑 **Must not add a `SpaceState` field or an event type.** Legs B and C.
3. 🛑 **Must not touch `apply_join`, `check_permission`, `skip_membership`, or the INV-EXP block.**
4. 🛑 **Must not "fix" anything it finds.** A finding is reported under §8, not repaired — *an implementer
   who quietly fixes the thing the test was built to witness destroys the witness.*
5. 🛑 **Must not use a fresh unregistered keypair as the actor** (§1).
6. 🛑 **Must not assert `HeldPending` anywhere in shipped code** (V-3 is a discarded probe).
7. 🛑 **Must not add `#[ignore]`, `todo!`, a commented-out after-assertion, or any placeholder for Leg D.**
   🔑 **The inversion is a Leg D DoD item (§10), not a reservation here** — *a key nothing writes is a key
   nobody has round-tripped* (`N-182` / the M-RP6.1k finding).

---

## §8 — DEVIATION PROTOCOL (Rule 6)

**Clair reports; she does not absorb.** Report — do not redesign — if:

1. Any `X-` row in §2 does not say what it claims at its cited span. 🔑 **§2 is Chat's measurement and
   `F-1`…`F-4` in this arc were all Chat's, all internally consistent, and all false.**
2. **§2 is a census rather than a partition** — a gate exists on the path that §2 does not list.
   ***This is the likeliest defect in this document.***
3. The fixture cannot reach `Accepted` for a reason not in §2.
4. `submit_locally` turns out not to be the local path (H-1).
5. A precondition in §4.3 fails — **that is a finding, not a fixture bug**, and it is reported before it is
   worked around.
6. **Any assertion needs to be weakened to make the test pass.** 🛑 ***A green test bought by relaxing an
   assertion is the failure mode this whole leg exists to prevent.***

---

## §9 — 🔒 CLAIR'S COLD READ, BEFORE ANY CODE. **`CLAIR_LEG_A_BIS_RUNBOOK_READ.md`.**

### §9.1 — 🛑 THE INSTRUCTION THAT IS NEW, AND WHY IT IS NOT CEREMONY

🔒 **CLAIR MUST DERIVE LEG ①'s EXPECTED OUTCOME FROM SOURCE AND WRITE IT DOWN *BEFORE* READING §4.5.**
Her read file opens with her own answer to: **what does `dispatch_event` do with a registered third party's
space-level `membership.join` against a DM, today?** — derived from `exchange.rs`, `runtime.rs` and
`state.rs`, with sites. **Only then does she read §4.5 and compare.**

🔒 **AND §1 IS FROZEN ONCE WRITTEN — NEW IN v1.2.** Write it, **verify the bytes landed**, and **do not reopen it** for any reason, including tightening a line-reference that later measurement improves. ⚠️ **THIS IS NOT A REPRIMAND; IT IS A GAP CLAIR FOUND AND DISCLOSED HERSELF.** On the first cold read she refined eight line-refs in §1 after reading the runbook, **two of which then matched the runbook's exactly** — and although **no verdict, crux or answer changed**, ***the record cannot distinguish convergence-by-re-measurement from transcription, and an independence claim that cannot be checked is not an independence claim.*** 🔑 **The fix is procedural, not a matter of restraint: later refinements go in a §3 measurement note, never back into §1.** *A rule that depends on the reader resisting a temptation is not a rule.*

⚠️ **THE REASON, AND IT IS NOT HYPOTHETICAL: LEG A-bis ARBITRATES A CLAIM CHAT MADE, AND THE WRONG FIXTURE
WAS AUTHORED BY THE SEAT WITH A STAKE IN THE RESULT.** Reading Chat's stated expectation first makes an
independent derivation into a confirmation. 📌 **This is a RUNBOOK INSTRUCTION, not a new seat rule** — the
seat rule already exists and it is the cold read itself; this is the one leg where the *order of reading*
inside it is load-bearing.

### §9.2 — The `F9` pass, and the questions

**`F9`: can each gate in §5 be RUN, in the order written, from the seat that owns it?** — plus:

1. **Is §2 a partition or a census?** *Four times in this arc a set that looked complete was not, once
   inside the very section written to test for it.*
2. **Does each `H-` and `X-` row's cited span say what the row claims?**
3. **Is `H-7`'s claim right — does the harness genuinely never call `accept_registration`?** *If it does,
   §6.1's bound is wrong in the generous direction, which is the dangerous one.*
4. **Is there a fourth assertion §4.5 should carry** — a fact about today's behaviour that becomes
   unobservable after Leg D and that nothing else records?
5. **§4.6 is LOCKED and ships — but TWO of its claims were already found false by Chat and corrected in place** (the trailing `false` is `e2e_encryption` not `is_dm`; `from_space_create` DOES seed the creator as `Role::Owner`). 🔑 **Treat the whole of §4.6 as the LEAST-TRUSTED section in this document** — it is the newest, it was written after the recommendation rather than before it, and its error rate so far is two-for-five. **What else in it does not survive the source?**

📌 **Standing Clair up is Joe's.**

---

## §10 — DoD

🔒 **THE COLD READ IS DONE AND ITS FINDINGS ARE FOLDED IN AT v1.2 (records: `J-753`).** `tasks/CLAIR_LEG_A_BIS_RUNBOOK_READ.md`, 32,130 B. **Verdict: LOCKABLE with two plan-movers and four wording fixes** — all six re-driven by Chat under Rule 5 and **all six upheld**. 📌 **The fixture design survived intact**: actor, third identity, `rdx("")`, the three original assertions and the `H-6` tier precondition all hold against source, and **Clair's §1 derivation reached §4.5's answer exactly** (`Accepted` / `Role::Member` / `invited_by: None`). ⚠️ ***Both plan-movers were on the VERIFICATION side, not the fixture side — Chat's fixture was right and Chat's instruments were not.***

📌 **Also upheld, and it is the one whose wrong answer would have been dangerous: `H-7` HOLDS** — exactly **one** production `accept_registration` call site (`app.rs:3479`), everything else tests or comments. **§6.1's bound is correct, not generous.**

**Leg A-bis ① is DONE when:**

1. The test exists, is registered in `mod.rs`, and passes — **or every deviation is reported under §8.**
2. **Every gate in §5 is driven by Chat** (Rule 5), each stated with what it was measured on.
3. 🔒 **V-3's negative control is RUN and RECORDED** — *`X-1` must not remain a claim in a document.*
4. 🔒 **§6's four non-claims are written into the `M-SPACE-ADMISSION` ROADMAP node**, replacing the
   generous form with the measured one.
5. `cargo` re-measured and **moved**; scope proven two files by diffstat. 🔒 **Expected `1602` → `1604`
   (two tests, §4.4 and §4.6).** 🛑 **MEASURED, NEVER DERIVED — and the SUITE count is measured too, not
   predicted.**
6. 🔒 **LEG D's DoD GAINS THE INVERSION ITEM, WRITTEN IN THE SAME EDIT THAT CLOSES THIS LEG** (`N-109`):
   *"invert `third_party_registered_identity_joins_a_dm_it_is_not_party_to` — after the gate ships, a
   GREEN run of the un-edited test is a FAILURE OF THE GATE, not a pass."* 🛑 **Written now, because the
   leg that lifts a limit owes the removal of the note that states it.**
7. Clair's cold read exists at `tasks/CLAIR_LEG_A_BIS_RUNBOOK_READ.md` with §9.1's ordering honoured.

🛑 **"Commit pushed" IS NOT A DoD ITEM.** `Status: COMPLETED` in this file's header is the canonical signal.
📌 **The milestone close is Phase-0 §12 Leg F and is not restated here.**
