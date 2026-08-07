# RUNBOOK — M-RP-MEMBER-ACT Legs A + B — the annotations, then the command surface
> **Status**: COMPLETED  
> Version: 1.3  
> Date: Aug 2026  
> **Last updated**: 2026-08-06  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, the seat, and what it is NOT

Two legs of `M-RP-MEMBER-ACT`, written from `tasks/M_RP_MEMBER_ACT_PHASE0.md` **v1.9** at `99bb266`.

**Leg A** — `D-131` annotations. **Zero code, zero floors.** Three records currently say the opposite of the code this milestone builds on.
**Leg B** — the command surface: **one** Tauri command wrapping `create_dm_space`, plus `OQ8-K3`'s `counterpart` field, **L2's loader unification**, and the one-time backfill including `F-A`'s self case. **Rust. The cargo floor returns.**

🔒 **v1.1 CHANGES, FROM CLAIR'S ADVERSARIAL READ — REPAIRED, NOT ANNOTATED (`D-145`: this document has never been locked, never been executed, and is cited by nothing).**
- 🛑 **§4 gains L2, the loader unification** — her **Finding 1**: the migration was sited on `load_or_default_state` (the WRITE path) while the UI reads through `load_client_state` (`app.rs:4613`, a plain `from_str`). ***As written, the migration never reached the consumer it exists to serve, and Leg C's scan would have produced the exact duplicate K3 was adopted to prevent.***
- 🔧 **§5 gate 4 rewritten** — v1.0 asked for a proof no exposed command could give: it conflated *parses* (deserialisation → all `None`) with *backfilled* (`Some`) and named no mechanism.
- 🔧 **§5.5 / §8.1 gate-5 prediction corrected** — her **Finding 2**: it was **inverted**. The gate does not flag `counterpart`, so the escalation could never have fired.
- 📌 **§8 gains her Finding 3 and her serialisation note.**

🔒 **v1.2 CHANGES, FROM CLAIR'S THIRD (NARROW) READ — REPAIRED, NOT ANNOTATED (`D-145`).**
- 🛑 **§5 gate 4b rewritten AGAIN — v1.1 named an observation the CLI cannot render.** `cmd_spaces` prints `name`/`space_id`/`node_endpoint`/`role` + rooms and **nothing else** (`app.rs:2643-2652`); there is no `--json`; the batch pipe arm discards the result (`batch.rs:378`); and the migrated state is not persisted on a read, so it cannot be read off disk afterwards. 🔑 ***THE CATEGORY ERROR: gate 4b's claim is about a LOADER, so it must assert on the DATA, not on a display surface.***
- 🔧 **§4 L2-a: `private` → `pub(crate)`.** `load_or_default_state` lives in `ops.rs` and `load_client_state` in `app.rs` — **different modules, so a module-private body cannot serve both.** *(`SELF_THREAD_LABEL` is already `pub(crate)`, `ops.rs:765`.)*
- 🔧 **§4 L2-a's table corrected: `load_client_state` has THREE failure modes, not two** — missing (`bail!` with the init/register hint), **read-I/O error (`?` with "failed to read")**, and parse error (`.context("corrupt")`). **All three messages must survive.**
- ✅ **§8 item 1 DISCHARGED** — she walked the signature against both loaders: `Option` carries missing-vs-present, `Result` carries the error distinction, both wrappers fall out cleanly. **The "cannot share a body" worry does not materialise.**
- 📌 **§8 gains the self-arm scoping note.**

⚠️ **AND A CHAT PROPOSAL THAT DIED ON MEASUREMENT, RECORDED SO IT IS NOT RE-PROPOSED:** routing gate 4b through the `--aicontrol` surface, which does serialise `ops::spaces` wholesale (`aicontrol.rs:418-421`). **It is a NAMED PIPE, not a CLI flag** (`aicontrol.rs:8-26`), spawned only from inside a resident client (`ai_service.rs:627`, `desktop.rs:835`), with a token check (`:318`) and **no existing harness that speaks it**. ***More expensive than both options on the table, proposed as "zero new code" — the same untraced-invocation error, one turn after diagnosing it.***

🔒 **LOCKED BY JOE 2026-08-06 AT `bfd7c75` — STATUS `PENDING` → `ACTIVE`, VERSION DELIBERATELY UNCHANGED AT v1.2.** ***The locked artefact is byte-identical to the one Clair read on the third pass;*** a version bump would have made the reviewed document and the authorised document different names for different things. **Clair may now implement. Legs A and B only.**

📌 **Three adversarial reads preceded this lock and each returned exactly one blocker, each smaller than the last:** v1.0 — the migration sited on a loader the UI never calls · v1.1 — a gate line asserting on a display that cannot render the field · v1.2 — `private` where the modules require `pub(crate)`.

⚠️ **SUPERSEDED BY THE LOCK ABOVE, ANNOTATED NOT DELETED (`D-145`) — the paragraph below was true from v1.0 through v1.2 and is the reason three adversarial reads happened before a line of code was written. It is history, not instruction.**

🛑 **THIS RUNBOOK IS NOT LOCKED.** Per this milestone's own record — Leg 0 found five plan-moving defects and Leg 0-bis found four more, and **Chat's own re-reads passed both times** — Clair reads this adversarially before Joe locks it. **No code is written until it is locked.**

🔒 **SEATS (`D-123`).** Clair implements from a locked runbook. Chat re-drives every verification leg independently (Rule 5) — **numbers Chat did not personally measure do not enter canonical records.** Joe holds all pushes and every appearance/architecture call.

**What these legs do NOT do:** R7 does not become interactive (Leg C) · nothing renders differently · no `oncontextmenu` (Leg D) · no `is_dm`, no DM home (Leg E) · the wire, `xgen-core` and `xgen-node` are untouched.

---

## §1 — GROUNDING, measured at `99bb266`

| # | fact | site |
|---|---|---|
| **B1** | `desktop.rs` exposes **19** `#[tauri::command]`s; neither DM op is among them | `desktop.rs`, `invoke_handler` |
| **B2** | `create_dm_space(ctx, args: &CreateDmSpaceArgs) -> Result<CreateDmSpaceResult>`; `CreateDmSpaceArgs { invitee: String }` | `ops.rs:806-809` |
| **B3** | `KnownSpace { space_id, name, node_endpoint, role, rooms }` — **no `counterpart`, no `is_dm`** | `xgen-common/src/state.rs:185-192` |
| **B4** | TS mirror is **verbatim snake_case, no mapping layer** | `spaces-state.svelte.ts:20-32` |
| **B5** | `load_or_default_state(data_dir, identity_id, home_node)` — **already receives `identity_id`**, and returns early at `:68` on a successful parse. **Callers: seven STATE-WRITE sites** — `:659` `create_space`, `:743`, `:961` `create_dm_space`, `:1038` `self_open`, `:1641`, `:1778` | `ops.rs:59-70` |
| 🛑 **B5a** | **A SECOND, DISJOINT LOADER SERVES THE READ PATH: `load_client_state` — a plain `serde_json::from_str`, no migration hook.** Callers: `ops::whoami :200` · `status :226` · **`spaces :249`** · `rooms :270` · `aicontrol.rs:265` · `events_pipe.rs:234` | `app.rs:4613-4624` |
| 🛑 **B5b** | **AND THE UI IS ON THAT ONE.** `get_spaces` → `ops::spaces` → `load_client_state` ⇒ a migration written only in `load_or_default_state` **is invisible to every UI read** | `desktop.rs:625` |
| **B6** | The DM name is `format!("DM with {}", invitee)`, or the literal `SELF_THREAD_LABEL` (`"self"`) when `invitee == identity_id` | `ops.rs:965-969` |
| **B7** | `create_dm_space` **pushes unconditionally — there is no dedup** | `ops.rs:970` |
| **B8** | `create_space` writes `name: args.name.clone()` — free-form, max 128 chars — into the **same field** the legacy self scan keys on | `ops.rs:660-662`, scan at `:1042` |

🛑 **B9 — AND THE PHASE-0's K3 COST IS UNDERSTATED. ANNOTATE, DO NOT REPAIR (`D-131`).** §5-OQ8 says *"two write sites (`ops.rs:660` normal, `:970` DM)"*. **Measured: five `KnownSpace { … }` literals** — `ops.rs:660` and `:970` (production) plus **three test fixtures**: `desktop.rs:1204`, `ops.rs:3127`, `ops.rs:3164`. 🔑 ***`#[serde(default)]` fixes DESERIALISATION, not struct-literal construction*** — every literal must supply the new field or the crate does not compile. **The production count is right; the compile surface is 5.**

---

## §2 — Chat's rulings for these legs (`D-123`), recorded and reversible on one line

- **R-1** — the command is named **`create_dm_space`**, matching the op. *Not `open_dm`: the command creates, and Leg C's open-or-draft logic lives in the frontend.*
- **R-2** — the command returns the op's `CreateDmSpaceResult` **unchanged**. No new DTO; the `fetch_identity`/Leg D precedent.
- **R-3** — the backfill runs **only on the parse-success path** (`ops.rs:68`). A freshly defaulted state has `spaces: vec![]` and nothing to migrate.
- **R-4** — the backfill is **idempotent and non-destructive**: it writes `counterpart` only where it is `None`. It never rewrites `name`.
- **R-5** — `self_open` is **not touched and not registered**. It stays CLI-reachable and CLI-tested (`OQ6-E2` point 4).

---

## §3 — LEG A — the annotations. **COMMIT ALONE. Zero code, zero floors.**

🔑 **Each is a claim that is FALSE TODAY, annotated at its site, never silently repaired (`D-131`).**

### Step A-1 — `ui/common/lib/components/widgets/members-panel.svelte:11-14`

The comment asserts *"NOT A SELECTION SURFACE … R7 must NEVER call `selection.set()`"*. **J-675 already filed that it overstates its own source, and `L-7` now reverses it deliberately.** Append below line 14, do not delete:

```
  // ⚠️ SUPERSEDED, ANNOTATED NOT REPAIRED (D-131, J-675 + M-RP-MEMBER-ACT L-7).
  // The "must NEVER call selection.set()" rule above is REVERSED by L-7 (2026-08-06, Joe
  // uttered): LMC does BOTH — opens the DM and writes the bus, so R8 shows the member's
  // card. The REASON was inverted, not discarded: J-591 objected that the bus write would
  // be SILENT; under L-7 it is the point. Wiring lands in Leg C — the rows are still inert
  // here. This comment also overstates its own source (J-675); M-RP-PANEL-INERT recorded
  // inertness as DEFERRED, not rejected, so `interactive` is being USED, not overridden.
```

### Step A-2 — `ui/common/lib/stores/selection.svelte.ts:2-3`

The header says `entity-context-menu` **READs** the bus. **It does not — it takes `descriptor` as a PROP and does not import `selection` at all** (G7, `entity-context-menu.svelte:14`). Append after line 6:

```
// ⚠️ CORRECTED, ANNOTATED NOT REPAIRED (D-131, M-RP-MEMBER-ACT Leg A). The line above says
// `entity-context-menu` READS this bus. Measured 2026-08-06: it does NOT. It takes
// `descriptor: EntityDescriptor` as a PROP, is gesture-agnostic, and does not import this
// module. R8 (inspector) is the only consumer. J-591 carries the same claim.
```

### Step A-3 — `tasks/M_RP_MEMBERS.md`, three drifted numbers

At **`:309`** and **`:406`** — both say **18** Tauri commands; it is **19**. At `:309` also `self_open :1002` (it is **`:1019`** — `:1002` is inside the result struct) and `create_dm_space :793` (it is **`:806`**). **Annotate inline at each site; do not edit the original numbers.**

### Step A-4 — the gate

**None.** No `.rs`, no `ui/**` behaviour, no `.svelte` logic. `svelte-check` and cargo are **stated, not re-run**: cargo **1596/0/62 × 56** · svelte-check **0/34/15** · catalogue **435**.

🛑 **Comment-only edits still touch `.svelte` files.** Run `npm run check` once to prove **0/34/15 is unmoved**; if it moves, stop and report.

---

## §4 — LEG B — the command surface + K3. **COMMIT ALONE. Moves cargo.**

### Step B-i-1 — `xgen-common/src/state.rs:185-192`, the field

```rust
pub struct KnownSpace {
    pub space_id: String,
    pub name: String,
    pub node_endpoint: String,
    /// "owner", "admin", "moderator", "member".
    pub role: String,
    pub rooms: Vec<KnownRoom>,
    /// The DM counterpart's XGID, or the session identity for the self thread.
    /// `None` for an ordinary Space. Backfilled once at load from the legacy
    /// `name` (M-RP-MEMBER-ACT OQ8-K3): the label is a DISPLAY string a user can
    /// write, so it must never be a lookup key (D-143 — the cheap option was
    /// unsound). After the migration the name is free to change.
    #[serde(default)]
    pub counterpart: Option<String>,
}
```

### Step B-i-2 — the five construction sites (B9)

**Production:** `ops.rs:660` (`create_space`) → `counterpart: None` · `ops.rs:970` (`create_dm_space`) → `counterpart: Some(args.invitee.clone())` — **including the self case, where `invitee == identity_id`.**
**Fixtures:** `desktop.rs:1204`, `ops.rs:3127`, `ops.rs:3164` → `counterpart: None`.

### Step B-i-3 — 🔒 L2: UNIFY THE TWO LOADERS, **THEN** put the backfill in the one that remains

🛑 **THIS STEP EXISTS BECAUSE v1.0 GOT IT WRONG.** The migration was written into `load_or_default_state` alone — the WRITE path. **The UI reads through `load_client_state` (B5a/B5b), so the migration would never have reached Leg C's scan, and the first click on DAVE would have created the duplicate K3 was adopted to prevent.**

🔒 **JOE RULED L2 over L1 (backfill both loaders).** *Two migrations that must stay identical forever, with nothing enforcing it, is a claim that can silently go false* ⇒ **unsound** ⇒ `D-143`.

**L2-a — one loader.** `load_or_default_state` and `load_client_state` become **one function** carrying **one migration**. The two differ today in exactly three ways, and all three must be preserved:

| | `load_or_default_state` (`ops.rs:59`) | `load_client_state` (`app.rs:4613`) |
|---|---|---|
| **missing file** | returns a **default** `ClientState` | 🛑 **`bail!` with the "run init/register first" hint** |
| **read-I/O error** | falls through to the **default** | 🛑 **`Err` via `?`, context "failed to read"** |
| **corrupt / unparseable** | falls through to the **default** | 🛑 **`Err`, context "state file is corrupt"** |
| **needs** | `identity_id`, `home_node` (to build the default) | neither |

⚠️ **THREE failure modes on the read side, not two** (Clair, third read — v1.1's table omitted the read-I/O arm). **All three messages must survive the unification.**

🔒 **RULED (Chat, `D-123`): keep BOTH behaviours behind ONE body.** One **`pub(crate)`** `read_and_migrate(data_dir, identity_id: Option<&str>) -> Result<Option<ClientState>>` performs read → parse → **migrate**; the two existing public fns become thin wrappers that differ **only** in how they handle `None`/`Err`. ⚠️ **`pub(crate)`, NOT private** — `load_or_default_state` is in `ops.rs` and `load_client_state` is in `app.rs`, so a module-private body cannot serve both (Clair, third read). ⚠️ ***Do NOT collapse the error behaviours.*** `load_client_state`'s messages are user-facing and the CLI depends on them; `load_or_default_state`'s default is what lets a first run work at all. **Collapsing them is a silent behaviour change to six call sites and is out of scope.**

⚠️ **`identity_id` is `Option` because the read path does not have one.** When it is `None` the **self arm cannot run** — the peer arm still does. 📌 **That is correct and not a hole:** `get_spaces` renders a list; the self thread's `counterpart` is only needed by a lookup, and every lookup path is a write path that has the identity. **If a later leg needs the self arm on a read, it passes the identity — the seam is there.**

**L2-b — the migration, in `read_and_migrate`, on the parse-success path only (R-3):**

```rust
// One-time K3 migration, in the ONE loader (L2). The parse lives HERE and nowhere
// else — never in a lookup, never in a render path. Idempotent: only fills `None` (R-4).
for sp in &mut state.spaces {
    if sp.counterpart.is_none() {
        sp.counterpart = match identity_id {
            // F-A: the self thread's name is the bare literal "self" — NO "DM with "
            // prefix, so the peer arm yields None, the field scan misses, and
            // create_dm_space (no dedup, :970) mints a SECOND self thread.
            Some(id) if sp.role == "owner" && sp.name == SELF_THREAD_LABEL => {
                Some(id.to_string())
            }
            _ => sp.name.strip_prefix("DM with ").map(str::to_string),
        };
    }
}
```

🛑 **The migrated state is NOT persisted here.** It is written on the next ordinary state-write. **Because the backfill is idempotent and cheap, re-running it on each load is correct and avoids a write on a read path.** *Named so it is not later read as an oversight.*

### Step B-i-4 — the Tauri command, in `desktop.rs`

One `#[tauri::command] async fn create_dm_space(...)` delegating to `ops::create_dm_space`, following the `fetch_identity` precedent verbatim (`M-RP-IDENTITY-RESOLUTION` Leg D, `RUNBOOK_…_LEG_D.md` §4). **Register it in `invoke_handler`. 19 → 20.**

### Step B-i-5 — the witness test

One test proving the backfill's **three arms**: a peer DM (`"DM with xgen://…"` → `Some(xgen://…)`) · **the self thread (`"self"` + `role == "owner"` → `Some(identity_id)`)** · an ordinary Space (`"Engineering"` → `None`). 🔑 **The self arm is the one Leg 0-bis found; it is the arm most likely to be dropped.**

### Step B-i-6 — the TS mirror, `spaces-state.svelte.ts:25-32`

```ts
  /** DM counterpart XGID, or the session identity for the self thread. `null` for a Space. */
  counterpart: string | null;
```
*Carried verbatim as Rust serialises it (B4). `Option<String>` → `string | null`.*

---

## §5 — GATES

**Leg A:** `npm run check` → **0/34/15 unmoved**. Nothing else.

**Leg B, on the committed tree:**
1. `cargo test --workspace` → **≥ 1597** passed (1596 + the witness), **0 failed**, 62 ignored, 56 binaries. **Report the exact triple.**
2. `cargo clippy --workspace --all-targets -- -D warnings` → clean.

🛑 **FALSE, AND ANNOTATED NOT REPAIRED (`D-145`) — CLIPPY HAS NEVER BEEN CLEAN ON THIS CODEBASE.** Clair found it; Chat confirmed it by an independent method (a detached worktree at the lock commit `5ac91ee`, three commits before hers). **Both trees produce the IDENTICAL four errors:** `desktop.rs:126` (`map_or`) · `desktop.rs:192` (collapsible `if`) · `run_startup` 9/7 args (**`:810` at the lock, `:843` after Leg B — shifted +33 by the inserted command, same function**) · `resident.rs:1018` 8/7 args, **a file Leg B never opened**. ⇒ ***Leg B adds ZERO clippy errors.*** 📌 **Clippy is NOT among the tracked floors** (cargo · svelte-check · catalogue), so it gates nothing — but ***this gate asserted a state it had never measured, which is the third time in this document a gate was written against an unchecked assumption*** (4b twice, gate 2 once). **The four lints are left untouched: all sit in functions outside Legs A/B.**
3. `npm run check` → **0/34/15 unmoved** (the mirror adds a field, not an error).
4. 🔑 **THE BACKWARD-COMPAT + MIGRATION PROOF, AND v1.0's VERSION OF THIS GATE WAS NOT EXECUTABLE.** *v1.0 said "load Joe's real state with the built binary and prove `counterpart = Some(…)`" — it conflated **parses** (deserialisation, which yields `None` for every legacy Space) with **backfilled** (`Some`), and named no mechanism. With L2 there IS now one.* **Two distinct proofs, both required:**
   - **4a — deserialisation (a Rust test).** A fixture JSON with **no `counterpart` key** parses into `ClientState` without error. *This is what `#[serde(default)]` buys, and it is the only thing it buys.*
   - **4b — migration through the READ PATH, on real data. A RUST TEST, ASSERTING ON THE RETURNED STRUCT.** **Copy** `%LOCALAPPDATA%\XGenProtocol\xgen-client_state.json` into the test's temp dir (**five Spaces, one `DM with …sno_FWmw`, NO self thread**), call **`ops::spaces`** against it — the same entry `get_spaces` uses (`desktop.rs:625`), reaching the same `load_client_state` — and assert on `SpacesResult.spaces[..].counterpart`. **Expected: `DM with …sno_FWmw` → `Some("xgen://pubkey/ed25519:L87GVLyVH_fvg-5hV0PL1zpf_s4GUPenODusno_FWmw")`, the other four → `None`.**
   - 🔑 **WHY A TEST AND NOT A CLI RUN, WHICH v1.1 DEMANDED:** `cmd_spaces` prints only `name`/`space_id`/`node_endpoint`/`role` + rooms (`app.rs:2643-2652`), there is no `--json`, the batch pipe arm discards the result (`batch.rs:378`), and the migrated state is **not persisted on a read** so it cannot be recovered from disk afterwards. ⇒ ***`counterpart` is not observable through ANY exposed command.*** 🛑 **The claim under test is about a LOADER, so the assertion belongs on the DATA. v1.1 demanded an observation from a display surface — a category error, and the second unexecutable version of this same gate.**
   - 🛑 ***THIS IS THE GATE THAT PROVES L2.*** Before the unification it returned `null` and could not have been driven at all. **If it passes, the migration reaches the consumer K3 exists to serve; if it is skipped, nothing else in this leg proves that.**
   - ⚠️ **Never run against Joe's live file. Copy into the test temp dir; leave the original untouched.**
   - 📌 **Gate 4b does NOT exercise the self arm** — Joe's real state has no self thread. **The self arm is proven only by the witness fixture (B-i-5), and that is its sole proof.**
5. `xgid-slot-gate.ps1` → **PASS, and expect NO manifest change.** 🔑 **v1.0 predicted the opposite and was wrong (Clair's Finding 2).** The gate's identifier regex (`xgid-slot-gate.ps1:49`) is **name-keyed**, and `counterpart` matches none of its suffixes — tested directly: `pub counterpart: Option<String>` → **no match**; `space_id` → match; **`counterpart_id` → match.** ⚠️ ***So the pass is correct in outcome and wrong in reason: `counterpart` is a genuinely XGID-bearing `String` sitting inside the gate's own blind spot.*** **Report the pass; do NOT read it as classification.** Whether `D-137`'s mechanism should catch this is **Joe's**, filed separately, and gates nothing here.

🛑 **Chat re-drives 1–5 independently on the committed tree (Rule 5).** Clair's numbers are cross-checked, never adopted.

---

## §6 — COMMIT SHAPE

**Three commits, `D-074`.** ① Leg A annotations (docs + comments, no floors) · ② Leg B Rust (`state.rs`, `ops.rs`, `desktop.rs` — cargo moves) · ③ the TS mirror (`svelte-check`). **Joe pushes all three.**

---

🔒 **CLOSED 2026-08-06 — LEGS A AND B ARE SHIPPED. THREE COMMITS ON `origin/main`: `ce82ebe` (Leg A) · `8c70d14` (Leg B-i) · `132ce85` (Leg B-ii).** 8 files, **+200/−15**. Implemented by Clair from this runbook at v1.2; **every gate re-driven independently by Chat on the committed tree (Rule 5)** — numbers below are Chat's own, not adopted.

| gate | Chat-measured at `132ce85` |
|---|---|
| cargo | **1597 / 0 / 62 × 56** — baseline + exactly one witness, zero non-`ok` result lines |
| svelte-check | **0 / 34 / 15** — unmoved |
| `xgid-slot-gate` | **PASS, 74 slots (65/5/3/1)**, no manifest change — as §5.5 predicted |
| gate 4a | **PROVEN** — the source file holds **zero** `counterpart` keys and parsed anyway |
| 🔑 gate 4b | **PROVEN ON REAL DATA.** `DM with …sno_FWmw` → `Some("xgen://pubkey/ed25519:L87…sno_FWmw")`; `Engineering`/`Design`/`LegBSpace`/`LegF Verification` → `None`; count 5 |
| commands | **20 registered**; `self_open` **zero diff lines and unregistered** (R-5 held) |
| catalogue | **435 unmoved BY SCOPE** — no `ui/core`, no `ui/sampler` touched. ⚠️ *Not re-run; stated as scope, not as a measurement.* |

🔑 ***GATE 4b IS THE ONE THAT MATTERED, AND IT WAS RE-DRIVEN FROM SCRATCH BY CHAT*** — Clair's throwaway was deleted, so the number could not be inherited. **Before L2 that DM would have returned `None`, Leg C's scan would have missed it, and `create_dm_space` (no dedup, `ops.rs:970`) would have minted a duplicate.** ✅ **Joe's live state untouched throughout** (`LastWriteTime` unchanged at `08/05/2026 07:12:25`); both re-drives ran against copies.

📌 **§8 item 3 realised exactly as designed:** gate 4b exercised the **peer arm only** — Joe has no self thread — so **the self arm's sole proof is the witness fixture.**

---

## §7 — DoD

- [x] Three annotations written; each names what is false and why, and repairs nothing
- [x] `KnownSpace.counterpart` added with `#[serde(default)]`; **all five literals updated**
- [x] 🔒 **L2: the two loaders unified behind one body; BOTH error behaviours preserved** (`bail!` on the read path, default on the write path) — `read_and_migrate` is `pub(crate)`, `app.rs:4621` delegates, all **three** read-side modes intact
- [x] Backfill in the **one** loader, parse-success path only, idempotent, **self arm present and `Option`-guarded**
- [x] `create_dm_space` command registered; **19 → 20**
- [x] Witness test covers **all three** backfill arms
- [x] TS mirror carries `counterpart: string | null`
- [x] 🔑 **Gate 4b driven as a Rust test on a COPY of Joe's real state through `ops::spaces`, asserting on `SpacesResult.spaces[..].counterpart`** — the proof the migration reaches the read path
- [x] All §5 gates driven by Chat on the committed tree, exact numbers recorded — ⚠️ **gate 2 excepted: its "clean" premise was FALSE and is annotated above**
- [x] `self_open` untouched and unregistered
- [x] JOURNAL + CLAUDE.md PLAY + ROADMAP + this runbook updated in the same commit

---

## §8 — 🛑 WHERE THIS RUNBOOK IS MOST LIKELY WRONG

1. ✅ **DISCHARGED (Clair, third read): L2's WRAPPER SPLIT TYPE-CHECKS.** Walked against both loaders: `Option` carries missing-vs-present, `Result` carries the error distinction, both wrappers fall out cleanly. **The "cannot share a body" worry does not materialise** — provided the body is `pub(crate)` and all three read-side failure modes are preserved.
2. ⚠️ **THE SELF ARM CANNOT RUN ON THE READ PATH** (no `identity_id` there). Argued safe because every lookup is a write path — **argued, not measured.** 📌 **Clair's sharpening: NOTHING today uses `counterpart` for self at all** — `self_open` still name-scans at `ops.rs:1043` and R-5 leaves it. **For the seam to matter, Leg C must both move self resolution onto the field AND keep it on a write path.** *A Leg C latent flag, not an A/B blocker.*
3. 🛑 **GATE 4b'S SELF ARM HAS NO REAL-DATA PROOF.** Joe's state has no self thread, so 4b exercises the peer arm only. **The self arm is proven by the witness fixture and by nothing else.** *If the fixture is weak, the arm Leg 0-bis was needed to find is untested.*
4. 🛑 **The backfill parses `"DM with "` — an English literal `create_dm_space` writes.** If the label is ever localised, the migration silently yields `None` for every legacy DM. **One-time, so safe today and unsafe the day someone re-runs it against localised state.** *Filed, not guarded.*
5. ⚠️ **`role == "owner"` is part of the self test, copied from `ops.rs:1042`.** If a self thread can exist with another role, the arm misses it. **Not verified.**
6. ⚠️ **K3 CONTAINS the name-inference unsoundness; it does not REMOVE it (Clair's Finding 3).** Both arms still key on a field `create_space` lets a user write (`ops.rs:662`), so a legacy owner-Space named `self` — or `DM with X` — **gets a spurious `counterpart` at migration.** Bounded, one-time, consistent with `D-143`; **the Phase-0's "removes the unsoundness" phrasing is stronger than the mechanism delivers** and is annotated there.
7. 📌 **`#[serde(default)]` WITHOUT `skip_serializing_if` MEANS EVERY SPACE NOW SERIALISES `"counterpart":null` (Clair).** The wire-shape test at `desktop.rs:1203` survives **only because it uses `assert!(json.contains(…))`, not exact equality.** ***The `get_spaces` payload shape changes for every Space, including ordinary ones.*** Accepted — the TS mirror is `string | null` — but recorded so it is not later found as a surprise.
8. ⚠️ **Not persisting the migrated state is a deliberate call, not a measurement.** A later reader expecting `counterpart` on disk after one launch will not find it until the next write.
9. 🛑 **Leg B ships a command with NO CALLER — Leg C is its first consumer.** *`D-065`'s empty-machinery shape, accepted because §4.3 requires the op reachable before the row is clickable, and it is one leg, not one milestone.*
10. 🔑 **THE HONEST STATE OF THIS DOCUMENT AT v1.2.** Three adversarial reads, three blockers, **each smaller than the last** — a fatal loader split → an unobservable gate line. **v1.2's changes are corrections to text Clair has already read, not new design.** ⚠️ ***But gate 4b has now been written unexecutably TWICE, by the same author, and the second time was one turn after diagnosing the first.*** **If a third version fails, the fault is not the gate.**
