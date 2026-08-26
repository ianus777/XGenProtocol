# RUNBOOK — M-SPACE-ADMISSION Leg G-2: `3048 rejoin_not_anchored`, the loud refusal
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-25  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS LEG IS

**`M-SPACE-ADMISSION` Leg G-2 — `3048 rejoin_not_anchored`, the loud refusal.** `G-1` made rejoins reachable. **That is exactly what made the accept-then-drop reachable in production for the first time**, and this leg closes it: **the node refuses a rejoin it cannot anchor, instead of answering `Accepted` and letting the fold discard the join.**

📌 **Phase-0:** `tasks/M_SPACE_ADMISSION_LEGG_PHASE0.md` v1.2 §2.2 + §5. **Design:** `tasks/M_SPACE_ADMISSION_PHASE0.md` v3.0 §15.6.

🔑 **THE DEFECT IS ALREADY ASSERTED BY A SHIPPED, GREEN TEST:** `xgen-core/src/resolution/derive.rs:498`, `convergence_mp_f7_rejoin_anchored_at_root_is_dropped`. **This leg does not discover the drop. It makes the sender hear about it.**

🛑 **THIS RUNBOOK IS CLAIR'S. 🔒 LOCKED BY JOE 2026-08-25 — IMPLEMENT FROM THIS VERSION, IN A SESSION OPENED BY HER OWN KICKOFF.** Deviations are **reported, never absorbed** (Rule 6). 🔒 **§3.1's rejoin-only scope was read and LOCKED as written: the gate fires only for a returning member, and the residue it leaves is a KNOWN, TESTED boundary (`V-3`), not an oversight.** 🔒 **Citations anchored at `d80636c`; the invariant anchor is the quoted comment or symbol, never the line number.** ✅ **VERIFIED AT LOCK: `HEAD` is still `d80636c` and NO `.rs` has moved since** — every site below is live against the tree you will edit.

---

## §1 — 🔑 THE ONE FACT THAT SHAPES THE WHOLE LEG: THE NODE ALREADY COMPUTES THIS BOOLEAN

✅ **`runtime.rs:853`, inside `ingest_event`:**

```
let conflict = state_key_for_event(&event).is_some()
    && conflicts_in_log(&event, &store.range(0).unwrap_or_default());
```

**When `conflict` is true the node throws away the incremental apply and rebuilds the Space from `derive_resolved` — the rebuild that drops her join.**

🔑 ***THE NODE COMPUTES, IN SO MANY WORDS, THAT HER JOIN IS CONCURRENT WITH HER OWN DEPARTURE — AND THEN REBUILDS A SPACE WITHOUT HER, HAVING ALREADY REPLIED `Accepted`.*** **The knowledge exists. It arrives one step too late to reach her.** ⇒ **this leg does not invent a predicate; it moves an existing one earlier, to the reply path.**

📌 **This is `M-1`'s shape once more** — *a check that lives only past the reply point is a silent no-op on the answer path* — and it is the third time this arc has met it.

---

## §2 — GROUNDING. **OPEN EACH SITE BEFORE YOU EDIT (`D-153`).**

| # | fact | anchor |
|---|---|---|
| **W-1** | `pub fn conflicts_in_log(incoming: &Event, log: &[Event]) -> bool` — *"same state key AND no causal ordering"*, **ancestry-aware via `build_ancestors`, NOT direct-parent-only `conflicts_with`** | `xgen-core/src/resolution/derive.rs:260` |
| **W-2** | 🛑 **ITS CONTRACT: *"`log` MUST contain `incoming`"*** — the transitive-ancestry computation needs it present | same doc comment |
| **W-3** | 🛑 **IT RETURNS `false` FOR AN EVENT WITH NO `event_id`.** `event_id_owned(incoming)` → `None` → early `return false` | `derive.rs`, `fn conflicts_in_log` first arm |
| **W-4** | `MembershipJoin` and `MembershipLeave` share one state key when `room_id` is empty; `Invite`/`Ban`/`NodeEject`/`NodeUnban` key on the **target** in the same `membership` category ⇒ **all of them compete on one key for one identity** | `xgen-core/src/resolution/state_key.rs:44` ff |
| **W-5** | `algorithm.rs:146-147` picks `MembershipLeave` over `MembershipJoin` on a frontier of two ⇒ **the leave wins and the join is the loser the rebuild excludes** | `xgen-core/src/resolution/algorithm.rs` |
| **W-6** | 🔒 **`self.stores` IS REACHABLE FROM `dispatch_event`.** `pub stores: HashMap<SpaceXgid, Box<dyn EventStore + Send + Sync>>` on the Node; `dispatch_event(&mut self, …)` | `runtime.rs:307`, `:1120` |
| **W-7** | 🔒 **`validate_event` RUNS FIRST, AND ITS STEP 8 REQUIRES `event_id`.** Called at `runtime.rs:1315`; step 8 is *event_id matches canonical content hash*, and an absent `event_id` is `Rejected` there | `runtime.rs:1315` · `xgen-core/src/message/exchange.rs:549-558` |
| **W-8** | The admission gate block is `origin == EventOrigin::LocallySubmitted && event.room_id.as_str().is_empty()`, and `G-1`'s departed term now sits in its predicate | `runtime.rs`, the *"Leg D (D-1) — THE ADMISSION GATE"* banner |

🔒 **W-7 IS LOAD-BEARING AND IT IS WHY `W-3` IS NOT A HAZARD HERE.** `conflicts_in_log` **fails OPEN** on an event with no `event_id` — *a check whose failure mode reads exactly like success* (`N-197`). **It is unreachable at this site ONLY because validation already refused such an event 350 lines earlier.** 🛑 **THAT ORDERING IS A DEPENDENCY, NOT AN ASSUMPTION: it is asserted by control `V-4` and must not be re-implemented as a second `event_id` check.** *Same discipline as `G-1`'s absent ban clause.*

---

## §3 — THE EDIT

### G2-1 — the anchor gate

🔒 **PLACEMENT: immediately AFTER the `3047` admission gate, inside the SAME `LocallySubmitted` + empty-`room_id` block.** The order is the questions in their logical sequence: `3047` asks *do you need an invite at all?* · **`3048` asks *does your rejoin follow your own departure?*** · the `3044` expiry check lives inside the pending-invite branch and never sees a rejoiner.

```
// G-2 sketch, not a paste target — Clair writes the real thing against the tree.
let is_rejoin = space
    .members
    .get(&event.sender)
    .is_some_and(|m| !m.is_present());
if is_rejoin {
    let mut log = store.range(0).unwrap_or_default();
    log.push(event.clone());          // W-2: the contract requires it
    if conflicts_in_log(&event, &log) {
        return DispatchOutcome::Rejected(RejectInfo::coded(
            3048,
            "rejoin_not_anchored",
            /* reason: see §3.2 */,
        ));
    }
}
```

🔒 **`is_rejoin` IS `G-1`'s TERM, RE-READ, NOT RE-SPELLED.** Same `is_present()`, same `D-067` single definition. **If the two conditions can share one binding without contorting the block, share it; if that means restructuring the gate, do not — report it.**

### §3.1 — 🔒 THE GUARD IS THE SCOPE DECISION, AND IT IS DELIBERATE

**The check runs ONLY when the sender is a retained departed member.** Two reasons, and both are load-bearing:

1. **COST.** `conflicts_in_log` runs a full `topological_sort` plus `build_ancestors` over the entire Space log. **A rejoin is rare; a message is not.** Gating on `is_rejoin` means **the ordinary path pays nothing** — it does not even reach the store.
2. 🔑 **NAMING HONESTY.** The wire name is `rejoin_not_anchored`. **A gate that also refused a stranger's un-anchored first join would be a code whose name is NARROWER THAN THE THING IT DESCRIBES — this project's most-repeated defect class, on a permanent wire string.** ⇒ **the guard makes the name exactly true.**

⚠️ **THE RESIDUE, NAMED AND NOT TRADED AWAY:** a **first-time** joiner whose join is concurrent with a same-key event (`W-4`) still gets today's silent drop. 📌 **The residue is thin — her client anchors on her invite, which is what `get_invite_bootstrap` exists to fetch — but it is not empty, and it is NOT closed by this leg.** ✅ **It is pinned by control `V-3` so that it is a known, tested boundary rather than an unexamined one.**

### §3.2 — THE REASON STRING

**Technical register, matching `3047`'s neighbour**, and it must say **what she can do**, not only what failed: that the rejoin does not follow her own last membership event, and that her client must fetch that anchor. 📌 **The exact wording is Joe's to override — reject-reason copy is product surface.** 🛑 **Do not block on it: write the technical sentence, and Joe rewrites it in place if he wants to.**

### G2-2 — 🔒 THE ch3 ROW SHIPS IN THIS SAME CHANGE. **THE REGISTER SAYS SO ITSELF.**

✅ **`docs/xgen_ch3_specification.md` §3.6.10.10 states the rule in its own words:** *a wire code is allocated in this table in the same change that first emits it*, and it explains why — **the `3046` incident, where a live code missing from the table would have put two unrelated refusals on one wire number.**

**Two edits, both in this leg:**
1. **ADD the `3048` row** to the reject-code table, between `3047` and `3049`, in the shape of its neighbours: what the refusal means, that it is **local-admission only** (`LocallySubmitted`), that it fires **only for a returning member**, and that **the alternative was a reply that lied**.
2. **AMEND the closing prose line** — today it reads that `3048` *remains reserved … and is not yet live*. **Annotate at the site (`D-131`); do not silently rewrite.**

🛑 **THIS CORRECTS MY OWN PLAN, AND THE CORRECTION IS THE REGISTER'S.** `tasks/M_SPACE_ADMISSION_LEGG_PHASE0.md` §5 assigns the ch3 row to `G-5`. **§3.6.10.10 overrides it.** 🔑 ***A register that exists, is authoritative, and is not consulted at the moment of allocation is `C-8`'s exact shape — and it has already cost this arc one deviation and one duplicated leg letter.*** ✅ **The Phase-0 leg table is corrected in the same commit.**

---

## §4 — VERIFICATION

📌 **Tests extend `xgen-node/src/tests/space_admission_gate.rs`** — the gate lives in the same block, and that file's existing shape (through `submit_locally`, asserting on the `DispatchOutcome` **the sender receives**) is exactly right. **`G-1`'s helpers `ingest_invite` / `ingest_membership` / `submit_join` are already there.**

| # | check | requirement |
|---|---|---|
| **V-1** | **THE SUBJECT.** Invite-only Space · bob invited, joins, **leaves** · bob re-submits a join **anchored on the CREATE ROOT** (the fresh-install shape). | **`Rejected`, code `3048`, name `rejoin_not_anchored`.** **AND: bob is still departed afterwards, AND the store did not grow** — the refusal precedes `ingest_event`, so nothing was appended. |
| **V-2** | 🔒 **THE DISCRIMINATOR, AND THE MOST IMPORTANT CHECK IN THIS LEG.** The identical rejoin, **anchored on her own `membership.leave`**. | **`Accepted`, and bob is PRESENT.** 🛑 **Without this, `3048` could be refusing EVERY rejoin and `G-1` would be silently undone while every other test stayed green.** |
| **V-3** | **CONTROL — THE GATE IS REJOIN-ONLY.** A **first-time** invited joiner submits a join anchored at the create root (concurrent with her own invite on the same key, `W-4`). | **NOT `3048`.** Today's outcome, unchanged. 📌 **This pins §3.1's residue as a tested boundary.** |
| **V-4** | **CONTROL — THE ORDERING THAT MAKES `W-3` UNREACHABLE.** Submit a join whose `event_id` is absent or does not match the canonical hash. | **`Rejected` by VALIDATION (step 8), NOT `Accepted` and NOT `3048`.** 🔑 **This observes `W-7` instead of assuming it — the same discipline that made `G-1`'s absent ban clause a decision.** |
| **V-5** | 🛑 **RED-ON-REVERT, AND THE RED IS THE DEFECT ITSELF.** Delete the `G2-1` block; re-run. | **`V-1` must go red by returning `Accepted` — AND the assertion that catches it must be *bob is not a member afterwards*.** 🔑 ***The reverted run is the silent failure, reproduced and written down: the sender is told yes and the fold drops her.*** ✅ **Record the observed outcome, not just the failure.** |
| **V-6** | **`G-1`'s three tests stay GREEN and UNEDITED**, and so do both pre-`G-1` tests in the file. | 🛑 **If any must be weakened, that is a FINDING, reported and never absorbed.** |
| **V-7** | **THE DM CASE.** A DM party who left, rejoining **unanchored**. | **`Rejected 3048`** — and this is the case that matters most, because a DM leaver has no other route back. |

🔒 **FLOOR: cargo is `1644 / 0 / 62 × 56 SUITES` at `d80636c`. IT MUST MOVE.** **Delta measured with `--skip` on the new names plus libtest's own `filtered out`, never arithmetic.** 🛑 **Run detached with a sentinel; `--no-fail-fast`; sum `^test result:` case-sensitively; require `Compiling xgen-core` in the log.** *A detached run's notification exit code is the launcher's, not cargo's.*

📌 **`N-204` applies to your harness: chunk your writes. The ~70-line figure is a WORKING RULE, not a measured threshold — the boundary was never bisected, and the dangerous sibling (a cut that still parses and writes a half-file looking complete) is inferred, not observed.**

---

## §5 — WHAT THIS LEG MUST NOT DO

1. 🛑 **No client change.** `ops::join` still anchors on client-local memory; **making the anchor FETCHABLE is `G-3`/`G-4`.** This leg only makes the failure honest.
2. 🛑 **No second definition of *departed*.** Re-read `G-1`'s term through `is_present()`.
3. 🛑 **No second `event_id` check.** `W-7` is asserted by `V-4`, not re-implemented.
4. 🛑 **Do not widen the gate past rejoins** (§3.1). The name would stop being true.
5. 🛑 **Do not touch `ingest_event`'s conflict gate at `:853`.** It stays exactly as it is — this leg adds a reply, it does not change resolution.
6. 🛑 **Do not touch `conflicts_in_log`, `state_key_for_event`, or `algorithm.rs`.** Resolution is correct; the reply was wrong.
7. 🛑 **No ch3 edit beyond the `3048` row and its prose line** (`G2-2`). The standing *must not amend ch3* ruling (J-739) governs everything else.

🛑 **ANNOTATION AT THE SITE (`D-131`, J-776, 2026-08-25) — ITEM 7 COLLIDES WITH A STANDING CONVENTION AND THE CONVENTION WINS.** ✅ **`CLAUDE.md:1945`, Document Header Convention: *"This header MUST be updated on every file edit"* — standing and unqualified.** **Item 7 was drafted to fence SCOPE and it fenced the HEADER out with it.** ✅ **Clair took the convention and reported the collision rather than absorbing it; `ch3` v0.59 → v0.60 and the Phase-0 v1.2 → v1.3 both stand.** 🔑 ***A prohibition written to bound a change should name what it is NOT bounding; item 7 named a limit and silently swallowed an obligation.***

---

## §6 — DoD

- [x] `G2-1` shipped in `xgen-core/src/node/runtime.rs` (**+108/−4**), guarded on the rejoin term, comment carrying §1's finding and §3.1's two reasons — 📌 **`is_rejoin` HOISTED to a binding above the `3047` gate (Deviation ①): semantics identical, the four deleted lines are exactly the old inline term, and the lost short-circuit costs one `HashMap` lookup rather than the log scan the guard prevents**
- [x] `G2-2`: the `3048` row added to ch3 §3.6.10.10 **in this same change**, and the *reserved / not yet live* sentence **quoted rather than deleted**, with §3.6.10.10's own allocation rule cited as the reason it rode here
- [x] `tasks/M_SPACE_ADMISSION_LEGG_PHASE0.md` §5 corrected — the ch3 row struck from `G-5` and re-homed to `G-2`, **struck rather than deleted so the re-assignment is visible**
- [x] `V-1` … `V-7` run and green; **four new tests, nine in the file (2 pre-`G-1` + 3 `G-1` + 4 new)**
- [x] `V-5` run: gate deleted (34 lines), targeted run RED, sentinel 101 — 🔑 **OBSERVED `Accepted { new_joiner: Some(IdentityXgid(…)) }` with `is_member() == false` immediately afterwards.** 🛑 **WORSE THAN THIS RUNBOOK PREDICTED: the reverted reply does not merely say yes, it NAMES HIM AS THE NEW JOINER while the fold has already dropped him — a positive identity assertion a client could render, cache or announce.** Restored, sha256 verified identical, mtime stamped, next run showed `Compiling xgen-core`
- [x] `cargo` **1644 → 1648 / 0 / 62 × 56 SUITES**; delta measured with `--skip` (**exactly 1644**) and libtest's **`filtered out = 4`** — ✅ **BOTH SEATS INDEPENDENTLY (Rule 5), Chat's on a FORCED REBUILD (`Compiling xgen-core: 1`) after its first re-drive was caught serving a cached binary**
- [x] `vitest` / `svelte-check` carried **by scope**, proven by `git diff --name-only`: four paths, zero `ui/**`, zero `.ts`/`.svelte`
- [x] **Five deviations reported, none absorbed (Rule 6) — all five correct, and TWO were defects on Chat's side** (the kickoff's scope line, and item 7 above)

🛑 **THE LIMIT, RECORDED AND NOT SOFTENED: nothing ran against a live node, a wire, or a second identity. All four tests go through `submit_locally` in-process. `3048` has never been observed on a wire.** 📌 **That bound is `G-4`'s to close, not this leg's to soften.**

📌 **"Commit pushed" is deliberately not a DoD item** — `Status: COMPLETED` in this header is the shipped signal. **Clair never pushes.**
