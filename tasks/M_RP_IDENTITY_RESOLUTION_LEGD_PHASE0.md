# M-RP-IDENTITY-RESOLUTION Leg D — Tier-1 fetch on join — Phase-0
> **Status**: COMPLETED  
> Version: 1.3  
> Date: Aug 2026  
> **Last updated**: 2026-08-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

**This is the design close for Leg D only.** `tasks/M_RP_IDENTITY_RESOLUTION.md` v1.16 remains the milestone Phase-0; this document does not replace it and does not restate it.

🛑 **IT IS NOT A RUNBOOK AND MUST NOT BE IMPLEMENTED FROM.** ✅ **All four questions in §§3–6 were LOCKED 2026-08-03 — A3 · B2 · C1 · D1.** The runbook (`tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_D.md`) is authored **from those locks**, and Clair implements from **that**, never from this. ⚠️ *v1.0 read "four questions are open"; true when written. Superseded, kept not erased (`D-131`).*

🔒 **WHY IT EXISTS AT ALL:** `M_RP_IDENTITY_RESOLUTION.md` §7 prices Leg D as *"a new Tauri command + a merge-one-record setter."* **Measured, that is narrower than the thing it describes** — the named recurring defect class, live in the record. §§1–6 are what the measurement found.

🔑 **WHAT LEG D IS FOR, IN ONE LINE:** it is the **half that closes `G-B`**. Leg E is discharged (J-670). `G-B` closes on Leg D **and** Leg E together and on nothing else (`N-168`), so Leg D is the only leg that can close it — and the milestone must not close before it, or §4 ships a promise it cannot keep.

---

## §1 — Grounding

**All measurements below taken 2026-08-03 at `aae60be`** (= `git ls-remote origin main`, tree clean). **Read windows are stated**, per J-670's addendum forward rule: *a partial read of a producer is not a producer check.*

### §1.1 — Floors, and which were MEASURED here

| floor | value | provenance |
|---|---|---|
| **cargo** | **1595 / 0 / 62 × 56 terminators** | ✅ **MEASURED at `aae60be` this session.** `FAILED` case-sensitive = 0; `^error` = 0. **Not inherited** — the last measurement was J-669 and zero `.rs` was touched in the Leg C arc |
| **svelte-check** | 0 / 34 / 15 | ⚠️ **INHERITED** — Clair at `87307e8`. **D-ii re-measures before its first edit** |
| **xgid-slot-gate** | **PASS 74** (65 B / 5 D / 3 I / 1 U) | ✅ RUN at `aae60be` |
| **sampler catalogue** | 435 | inherited, out of scope for Leg D |

### §1.2 — The Rust half (windows stated)

| # | fact | citation | window read |
|---|---|---|---|
| **G-D1** | `FetchedIdentity` — the seven wire fields, and its doc comment states the wire ceiling outright | `ops.rs:431` (doc from `:417`) | 415–470 |
| **G-D2** | `pub async fn identity_get(ctx, identity_id: &str) -> Result<Option<FetchedIdentity>>` — one-shot: `ensure_connected` → `identity_get_on` → best-effort `goodbye` | `ops.rs:539` (doc from `:528`) | 495–585 |
| **G-D3** | 🔑 **`identity_get_on` (`ops.rs:502`) takes `&mut ClientConnection` and its OWN doc comment says the caller owns the lifecycle *precisely so lookups can batch on one connection*.** It is **private** (`async fn`, no `pub`) | `ops.rs:496–501`, `:502` | 495–585 |
| **G-D4** | **18 `#[tauri::command]` in `desktop.rs`; NONE fetches an identity.** Re-counted, not inherited | `desktop.rs` | full-file grep |
| **G-D5** | 🛑 **`fill_space_records` does more than fetch — it `AddressBook::load`s, runs the fill, and `book.save()`s UNCONDITIONALLY.** There is **no resident book** between commands | `desktop.rs:671–713` | 660–720 |
| **G-D6** | `absorb_fetch` routes a `Some` through the **version-aware `merge`**; `None` (`identity.not_found`) is a deliberate skip, never a placeholder | `ops.rs:2827` (doc from `:2817`) | 2815–2870 |
| **G-D7** | `FillLock` serialises fills because *"the loser of a read-modify-write race would otherwise silently discard resolved records"* — the file's own words | `desktop.rs:678–680` | 660–720 |
| **G-D8** | ⚠️ **The `ensure_connected` dead-`conn` mine is documented and NOT fixed:** every `goodbye` site leaves `session.conn = Some(dead)`; `fill_from_space` self-cleans as a workaround and the blanket fix is called *"its own arc"* | `ops.rs:2857–2867` | 2815–2870 |
| **G-D9** | `identity_get` takes `identity_id: &str`, **not `IdentityXgid`** | `ops.rs:541` | 495–585 |

### §1.3 — The frontend half (windows stated)

| # | fact | citation | window read |
|---|---|---|---|
| **G-D10** | `_book` has **exactly one writer**, `setResult`, and it **replaces WHOLESALE** | `address-book.svelte.ts:148` | full file (191 lines) |
| **G-D11** | 🔑 **`unresolved: true` is stamped at `:187` and THERE IS NO CLEARING PATH.** Swept both directions across the whole file: `:23` `:58` `:98` are comments, `:36` is the type declaration. **`:187` is the only assignment** | `address-book.svelte.ts:187` | full file |
| **G-D12** | `_notFound` is written by **four** setters only — `setInflight :137` · `setResult :147` · `setFailed :155` · `reset :163`. **All are fill-path.** ⇒ a live joiner can never reach `_notFound` | `address-book.svelte.ts` | full file |
| **G-D13** | 🛑 **`setResult` carries a late-response guard** (`spaceId !== _spaceId ⇒ discard`), and `addMember`/`removeMember` carry the same as `R4` | `:144` · `:184` · `:192` | full file |
| **G-D14** | 🛑 **`toDescriptor` branches on `m.unresolved` BEFORE it reads the book:** `flags: m.unresolved ? {} : { isAi: rec?.is_ai ?? false }` | `members-panel.svelte:101` | 86–118 |
| **G-D15** | `routeMembershipEvent` is **synchronous**; the `membership.join` arm calls `addressBook.addMember(...)` and returns | `app_client.svelte:252`, `:263` | 248–296 |
| **G-D16** | `tauriInvoke(cmd, args)` is a **module-scope** async helper — it resolves at router scope. The bare `invoke` at `:651` does **not** (it is an `onMount`-local destructured import; the J-670 lesson ②, already annotated at `:239–240`) | `app_client.svelte:814` | grep + 248–296 |
| **G-D17** | `entity-item` takes `unresolved?: 'unasked' \| 'erased'` — a **string enum**, not the store's boolean. `members-panel` maps between them (Leg B) | `entity-item.svelte:57`, `:126` | grep |

---

## §2 — 🛑 THE THREE FINDINGS THAT CHANGE THE PRICE

### §2a — The Rust half is SMALLER than §7 states, and its cost sits somewhere else

`ops::identity_get` already exists, public, one-shot (G-D2). The command is a **thin wrapper reusing `fill_space_records`'s session preamble**. There is **no new `ops::` verb** in the minimal shape.

⇒ **§7's *"a new Tauri command, so Rust, and it moves the cargo floor"* is right about the crate and wrong about where the work is.** The work is not the fetch. It is §4 (persistence) and §5 (the lock) — **neither of which §7 mentions.**

### §2b — 🛑 THE AI BADGE DOES NOT LIGHT WHEN THE RECORD LANDS. IT LIGHTS WHEN `unresolved` CLEARS.

The session kickoff carried: *"`members-panel.svelte:101` reads `flags` from the BOOK, so writing the record lights the AI badge with no mirror change."*

**MEASURED (G-D14): FALSE.** `:101` tests `m.unresolved` **first**. While the marker stands, `flags` is `{}` and the book record is **never read**.

⇒ **§7's stated point of the whole leg — *"resolves with their name AND their AI badge; the badge is the point, a §3.6.10 obligation"* — does not land from the `_book` merge alone.** The `unresolved` clear is not cosmetic dimming; **it is the gate on the badge.**

🔑 **The kickoff's CONCLUSION still stands — no new `MemberEntry` mirror field is needed — and its REASON does not.** They are separable, which is the J-670 addendum's exact species. Recorded, not silently repaired (`D-131`).

### §2c — 🛑 THE `unresolved` GREP TRAP HAS **THREE** CONCEPTS, NOT TWO

A repo-wide sweep of `ui/**` (`*.ts`, `*.svelte`, `node_modules` excluded) returns **34 hits across 10 files**, spanning three unrelated meanings:

| concept | files | hits |
|---|---|---|
| **① the roster marker** — this leg's | `address-book.svelte.ts` (6) · `app_sampler.svelte` (6) · `members-panel.svelte` (4) · `entity-item.svelte` (4) · `entity-panel.svelte` (2) · `mounts.ts` (1) · `app_client.svelte` (1, the comment at `:260`) | 24 |
| **② the send-status tone** — a timed-out message | `echo-status.ts` (3) · `send-status.svelte` (2) · `echo-status.test.ts` (1) | 6 |
| **③ 🆕 the DOCK LEAF** — *"never render an unresolved leaf"* | `app_client.svelte` `:451` `:473` `:485` `:501` | 4 |

⚠️ **The kickoff named ② and missed ③ — inside the very file it cited for ①.** ⇒ **SCOPE THE SWEEP BEFORE BELIEVING IT.** A file-level hit count on `app_client.svelte` reads as 5 marker hits; **one** is the marker.

📌 **LEAD, NOT A CLAIM, AND IT IS LEG F'S NOT LEG D'S:** `echo-status` *has* a word for a timed-out terminal state, and `D-126` deferred T3's bounded retry because its terminal state *"has no word"*. Whether that is precedent or coincidence is worth ten minutes at Leg F, when the residue is re-priced. **Not opened here.**

⚠️ **THIS LEAD IS VOID — IT RESTS ON A MISCITATION, CORRECTED 2026-08-05 (J-676), ANNOTATED NOT REPAIRED (`D-131`).** `D-126` **never deferred any retry**: it is the humane pubkey label (`DECISIONS.md:4710`, J-579) and the string `T3` does not appear in its span. `T3` is a **§6b-local option label** in `M_RP_IDENTITY_RESOLUTION.md`, owned by no decision — see that §'s annotation for the full correction. 🔑 **The `echo-status` observation itself survives and is still worth the ten minutes**, but it is precedent for **§6b's vocabulary gap**, not for anything `D-126` decided.

---

## §3 — 🔒 Q1 LOCKED: THE CONNECTION SHAPE — **A3** (Joe, 2026-08-03)

🔒 **LOCKED: A3 — one-shot `identity_get` now; A2's batched form FILED with a trigger.** Ruled together with §§4–6 as **one answer** (*"go by your recomms"*), which is the shape §12 said they had.

⚠️ *The option list and Chat's argument below are the menu the ruling chose from. Kept, not erased (`D-131`).*

`identity_get` opens, authenticates and `goodbye`s **per call** (G-D2). One join = one full cycle. A rejoin storm of N joiners = N cycles. `fill_and_members` (`ops.rs:2995`) batches on **one** connection precisely to avoid this — and the mechanism that lets it, `identity_get_on`, exists but is **private** (G-D3).

- **A1 — one-shot `identity_get`, as it stands.**
  ① *User-visible:* a joiner resolves in one round trip. In a storm, N handshakes + N auths — visible as **slow badge resolution**, never as failure. **Nobody has yet observed a storm; §7's own surface is one joiner at a time.**
  ② *Resource:* a thin `desktop.rs` wrapper. **Zero new `ops::` surface.**
- **A2 — make the batched form public and add a many-shot verb.**
  ① *User-visible:* **identical at N=1.** Only a storm differs, and no storm has been measured.
  ② *Resource:* a new `pub` op, its own tests, **and the `ensure_connected` dead-`conn` mine now applies to a second verb** — G-D8 calls the blanket fix *"its own arc"*, and `fill_from_space` needed a self-cleaning wrapper proven live at J-586 to survive it. **This is not a signature change; it is standing on a documented mine.**
- **A3 — A1 now, A2 FILED with a trigger.**
  ① *User-visible:* same as A1.
  ② *Resource:* A1's, plus one `Owes:` line.

🔑 **CHAT RECOMMENDS A3.** **Leg F is already obliged to measure the Tier-1 residue** (§6b `Owes:` — how often a fetch times out). **The storm frequency is the same measurement on the same surface**, and it is the first surface where either can occur at all. ⇒ **A2 gets priced against a number instead of a fear, and A1 costs nothing if the number says so.**

> **`Owes:` — LEG F MEASURES JOIN CONCURRENCY ALONGSIDE THE TIER-1 RESIDUE. If N-at-once joins are common, A2 returns as a live option; if they are not, A2 is closed with its reason.**

---

## §4 — 🔒 Q2 LOCKED: THE COMMAND PERSISTS — **B2** (Joe, 2026-08-03)

🔒 **LOCKED: B2 — absorb via `absorb_fetch` and `book.save()`, mirroring `fill_space_records`.** B1 was not a live option and was not offered as one.

⚠️ *Kept, not erased (`D-131`).*

**§7 does not ask this.** `identity_get` touches no book; absorption happens via `absorb_fetch` inside the fill (G-D6), and there is **no resident book between Tauri commands** (G-D5) — so "absorb but do not save" is not a third option, it is a no-op.

- **B1 — return `FetchedIdentity` to the webview; do NOT save.**
  ① *User-visible:* 🛑 **A FALSE BADGE, NOT MERELY A LOST ONE.** `setResult` replaces `_book` **wholesale** from `get_address_book` (G-D10) and replaces `_roster` from the fill — **and the fill's rows carry no `unresolved` field.** ⇒ on the next room switch the joiner's marker is `undefined` ⇒ falsy ⇒ `:101` takes the **book branch** with **no record** ⇒ `isAi: rec?.is_ai ?? false`. **An AI joiner renders as HUMAN.** That is `N-097` inverted — the exact trap `members-panel.svelte:97–100` was written to prevent.
  ② *Resource:* zero.
- **B2 — absorb via `absorb_fetch` and `book.save()`, mirroring `fill_space_records`.**
  ① *User-visible:* the badge survives the next fill **and** the next restart; the following fill is one fetch shorter.
  ② *Resource:* load + save the book per join event (disk I/O per join), and **it makes §5 mandatory**.

🔒 **CHAT RECOMMENDS B2, AND STATES PLAINLY THAT B1 IS NOT A LIVE OPTION.** B1 does not trade cost against completeness; **it ships a confident false claim about AI identity**, which is the one thing §3.6.10 and `D-065` both forbid. *The badge is the point of the leg; a badge that goes out — or worse, goes to the wrong value — on the next room switch is not the badge shipping.*

⚠️ **`D-121` lens ① is stated for the SHIPPED PRODUCT, not for today's desk** — the J-654 departure §7 already made deliberately. On one client no joiner exists at all.

---

## §5 — 🔒 Q3 LOCKED: THE JOIN FETCH TAKES THE LOCK — **C1** (Joe, 2026-08-03)

🔒 **LOCKED: C1 — `let _guard = lock.0.lock().await;` first, before any book I/O.** Forced by B2 rather than chosen.

⚠️ *Kept, not erased (`D-131`).*

Live only if §4 resolves B2.

- **C1 — take the lock.**
  ① *User-visible:* a join arriving during a fill waits for the fill. The fill is bounded (`tokio::time::timeout`, Leg A-bis / T1), so the wait is bounded — **a late badge, never a hang.**
  ② *Resource:* one `let _guard = lock.0.lock().await;`.
- **C2 — do not take the lock.**
  ① *User-visible:* a read-modify-write race between the join save and the fill save. **The loser's records are silently discarded** — `desktop.rs:678–679` names this outcome in its own words. Manifests as a member who resolves and then un-resolves for no reason on screen.
  ② *Resource:* zero.

🔑 **CHAT RECOMMENDS C1, AND IT IS FORCED BY B2 RATHER THAN CHOSEN.** The lock exists for exactly this shape; adding a second writer to the same file and not taking it re-opens the defect the lock was built to close.

---

## §6 — 🔒 Q4 LOCKED: THE ③ ARM SHIPS — **D1** (Joe, 2026-08-03)

🔒 **LOCKED: D1 — a null result pushes to `_notFound`.** ⇒ **Leg D is what makes §5a's E2 DM exception reachable at all**, and the §5b annotation in §9 is therefore owed, not optional.

⚠️ *Kept, not erased (`D-131`).*

`identity_get` returns `Ok(None)` for `identity.not_found` — **a normal outcome, not an error** (G-D2). But `_notFound` has four writers and **all four are fill-path** (G-D12). ⇒ **a live joiner who has been ERASED cannot reach `_notFound`.**

- **D1 — the merge setter adds to `_notFound` on a null result.**
  ① *User-visible:* an erased joiner is **hidden** per §5, or **marked** per §5a's E2 if they are the DM counterpart. The panel's claim concludes.
  ② *Resource:* one branch in the new setter. **No new store field** — `_notFound` already exists and `members-panel:117` already derives a `Set` from it.
- **D2 — leave it; an erased joiner stays ④, dimmed.**
  ① *User-visible:* 🛑 **a permanently dimmed row whose attempt ALREADY CONCLUDED.** §4c-i binds a transient claim to eventually resolve; here it resolved, and the panel says *not yet* forever. **This is `G-B`'s own defect re-created inside the leg that closes it.**
  ② *Resource:* zero.

🔑 **CHAT RECOMMENDS D1.** D2 is not a cheaper option; it is `G-B` reopened one level down, in the leg whose entire purpose is to close it.

🛑 **AND D1 NARROWS A CLAIM IN THE MILESTONE PHASE-0 — ANNOTATION OWED (`D-131`).** §5b says ③ is *"reachable in exactly one situation: a client with no cached record — fresh install, new device, or a wiped book."* **A live joiner is by construction not in the book.** ⇒ **after Leg D, ③ is reachable for any erased live joiner in an ordinary session.** §5b's *rule* stands (a held record is never re-fetched); its *example set* was narrower than its rule. **Leg D is what makes §5a's E2 exception reachable at all.**

---

## §7 — 🔒 TAKEN BY CHAT UNDER `D-123` — RECORDED, NOT ROUTED

These are sequencing, records and technical execution. **Reversible on one line; that is a statement about Chat's behaviour, not a request for Joe's.** Silence is not consent and none of these is being put up for a lock (J-670's seat lesson, third instance).

- **R-D1 — ✅ EXECUTED AS WRITTEN.** *"This brief lands ALONE, unreferenced; the §8 pointer into `M_RP_IDENTITY_RESOLUTION.md` lands with the lock, in the `D-074` bundle."* **v1.0 shipped alone at `9531a1d` (2026-08-03), pushed.** ⇒ the pointer, the runbook and the records land as a **second** commit, which is the shape the ruling designed. *Reason it was taken: a locked task doc pointing at an unratified design reads as if the design were settled — the J-670 ① failure shape. An orphan file for one turn is cheaper than a canonical record that overclaims.*
  - 🛑 **AND CHAT'S FIRST v1.1 EDIT ANNOTATED THIS LINE AS *"SUPERSEDED BY EVENTS — the lock arrived before the brief was committed (`HEAD` still `aae60be`, the file still untracked), so there was never an unratified state to protect against."* THAT ANNOTATION WAS FALSE AND IS ITSELF ANNOTATED HERE, NOT DELETED (`D-131` — including a correction's correction).** `9531a1d` had already landed and been pushed when it was written. 🔑 ***The `git status` it rested on was read at the START of the turn and trusted for the whole of it — a source of truth consulted narrowly at time T and reused broadly at time T+n.*** The same species as the last arc's five. ⇒ **FORWARD RULE, joining the producer-window rule: a claim about REPOSITORY STATE carries the moment it was read, and any claim about the tree made later in the same turn is RE-MEASURED, not inherited — the tree has a writer who is not Chat.**
- **R-D2 — Leg D SPLITS: D-i (Rust, moves cargo) / D-ii (frontend, moves `svelte-check`).** Forced by §8's split-by-floor rule; a single commit makes the delta unattributable within the floor.
- **R-D3 — the new merge setter carries its OWN scope guard**, mirroring `setResult :144` and `addMember :184`. A fetch fired at join and resolving after a room switch must not clear a marker in a scope the user has left. *(The `_book` half is scope-free — the book is a global cache keyed by identity. Only the roster touch is scoped.)*
- **R-D4 — `routeMembershipEvent` STAYS SYNCHRONOUS.** The fetch is fired as an un-awaited call to a module-scope async helper with its own `catch` — the `loadMembers` idiom (`app_client.svelte:209`, `untrack`ed), using `tauriInvoke` (G-D16), never the `onMount`-local `invoke`.
- **R-D5 — command name: `fetch_identity`.** `get_*` in `desktop.rs` denotes a local read (`get_spaces` is `fn`, a synchronous on-disk read — J-670 ④). This one is `async` and goes to the node; **naming it `get_*` would repeat the exact misreading that produced J-670's fourth error.**

---

## §8 — What changes, by floor

### D-i — Rust (`xgen-client`), moves cargo

1. `desktop.rs` — `#[tauri::command] async fn fetch_identity(identity_id: String, data, config, lock) -> Result<Option<FetchedIdentity>, String>`, reusing `fill_space_records`'s preamble (resolve node → `SessionState::new` → `ensure_identity` → `OpContext`).
2. Conditional on §4 = B2: `AddressBook::load` → `ops::identity_get` → `absorb_fetch` → `book.save()`.
3. Conditional on §5 = C1: `let _guard = lock.0.lock().await;` first.
4. Registration in the `invoke_handler` list.

⚠️ **THE CARGO DELTA WAS AN OPEN MEASUREMENT AND IS NOW SETTLED — SEE THE RUNBOOK §3.** v1.0 offered *"extending `tauri_command_return_serde_transparent_to_js_frontend` (`desktop.rs:1050`) adds no new test count ⇒ the delta may legitimately be `+0`."* **Measured while authoring the runbook: T8 asserts against `PacingState` and is ABOUT that type**, so widening it would make one test answer two questions. ⇒ **a NEW test ships and the expected delta is `1595 → 1596`.** Superseded, kept not erased (`D-131`).

🔑 **AND THE RETURN TYPE IS `Option<SeenRecord>`, NOT `FetchedIdentity`** — settled in the runbook §4. The frontend's `_book` values **are** `SeenRecord`; returning the wire type would force a TS-side reconstruction of `SeenRecord::from_fetched`, duplicating the single producer. **And no new struct means no new manifest slots**, which is why V4 expects PASS 74 unchanged.

⚠️ **RUN THE GATE AFTER D-i.** `FetchedIdentity` is already typed; the command's `identity_id: String` parameter is a **function parameter, not a struct slot**, and whether the manifest counts it is **unmeasured**. **Expect PASS 74 unchanged; if the count moves, that is a FINDING to be ruled, not a failure to be suppressed.**

### D-ii — frontend (`ui/**`), moves `svelte-check`

1. `address-book.svelte.ts` — a new merge-one setter (working name `resolveMember(spaceId, identityId, record | null)`), doing **three** things, of which the second is the leg's whole point:
   - merge one `SeenRecord` into `_book` **without** replacing it (`setResult` stays the only wholesale writer — §7's own stated mitigation);
   - **clear `unresolved` on the roster entry** (G-D11 — there is no such path today, and G-D14 is why it matters);
   - **on a null result, push to `_notFound`** (§6 D1).
   All three behind R-D3's scope guard.
2. `app_client.svelte` — the `membership.join` arm fires the fetch after `addMember` (R-D4).
3. **Nothing on `MemberEntry`'s shape.** ✅ Correct — but **for §2b's reason, not the kickoff's.**

📌 **`svelte-check` is re-measured BEFORE the first D-ii edit.** The 0/34/15 in §1.1 is inherited from `87307e8` and is not a baseline until it is re-run.

---

## §9 — Corrections owed to existing records (`D-131` — annotate, never repair silently)

1. **`M_RP_IDENTITY_RESOLUTION.md` §5b cites `address_book.rs:253` (`remove`) and `:285` (`evict_older_than`).** Measured at `aae60be`: **`:267`** and **`:299`**. The retype legs moved them. *The claim they support is unaffected — every caller is still a test — but the citations are stale.*
2. **§2b** — the AI-badge mechanism, above.
3. **§6's narrowing of §5b's ③ reachability**, above.
4. **§2c** — the third `unresolved` concept.

⚠️ **These are annotations, not edits to be made now.** They travel in the `D-074` bundle with the lock, so the corrections and the decisions they inform land together.

---

## §10 — Verification plan (skeleton; the runbook details it)

- **D-i:** cargo re-run against the §1.1 baseline **measured this session**; delta enumerated by name, `FAILED` grepped **case-sensitively**, terminator count summed programmatically. Gate re-run, apps down.
- **D-ii:** `svelte-check` re-measured **before** the first edit and after; delta explained.
- 🛑 **NEITHER HALF IS A BEHAVIOUR VERIFICATION.** Legs A and B were compile- and type-verified only; **Leg D is the same**. A single client produces no joiner (G-D15's arm needs an inbound `membership.join` from another identity), so **every positive case is Leg F's** — and Leg F's obligation list grows by this leg's three: a joiner resolving to ①/②, a joiner resolving to ③, and the storm/residue counts (§3 `Owes:`).
- ⚠️ **A store driven by hand is a probe that cannot fail.** No hand-driven `resolveMember` call is admissible as evidence that the join path works.

---

## §11 — DoD (Leg D)

- [x] §3 connection shape ruled — ✅ **A3 (Joe, 2026-08-03)**
- [x] §4 persistence ruled — ✅ **B2 (Joe, 2026-08-03)**
- [x] §5 `FillLock` ruled — ✅ **C1 (Joe, 2026-08-03)**
- [x] §6 the ③ arm ruled — ✅ **D1 (Joe, 2026-08-03)**
- [x] Runbook authored from the locked answers, **written so Clair can refuse it** — ✅ **`tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_D.md` v1.0 ACTIVE (2026-08-03).** §7 names where it is most likely wrong and states outright that the list is **not** a census of its errors
- [x] D-i lands alone; cargo delta stated **before** the run and explained after — ✅ `aa7d9c9`, **1595 → 1596**, the `+1` predicted in the runbook §3 and the added test named at V2
- [x] D-ii lands alone; `svelte-check` re-measured before the first edit — ✅ `9901036`, V5 **0/34/15** freshly measured, V6 Δ **0**
- [x] xgid-slot-gate re-run after D-i; any manifest movement **ruled, not suppressed** — ✅ **PASS 74 unchanged, CLEAN TREE** (Clair's own run was `-AllowDirty` and not quotable; the guard doing its job is why)
- [x] §9's four annotations applied in the `D-074` bundle — ✅ landed at `304742b`
- [x] **`G-B` TICKED — and only here.** ✅ Leg E discharged J-670; **Leg D landed at `aa7d9c9` + `9901036`** ⇒ `N-168` satisfied by the pair, ticked by neither alone
- [x] Records: JOURNAL + `CLAUDE.md` PLAY + `ROADMAP.md` + `M_RP_IDENTITY_RESOLUTION.md` + this doc in one commit (`D-074`) — ✅ J-672

---

## §12 — Handoff

✅ **ALL FOUR RULED 2026-08-03 (Joe): A3 · B2 · C1 · D1 — taken as ONE answer, which is the shape this section predicted they had.** §5 is forced by §4, §4 by §2b + the `N-097` inversion, §6 by `G-B`'s own logic; **§3 was the only one with genuine slack**, and A3 defers it onto a measurement Leg F already owes.

🔓 **NOTHING IN THIS DOCUMENT REMAINS OPEN FOR JOE.** ⇒ **the runbook `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_D.md` is authored from these four locks**, and Clair implements from it. 🛑 **She does not close her own leg.**

✅ **CLOSED 2026-08-04 (J-672). Leg D landed at `aa7d9c9` (D-i) + `9901036` (D-ii), Clair; every gate re-driven by Chat on the committed tree; `G-B` ticked.** 🔑 **THE FOUR LOCKS ALL PAID:** B2's persistence turned out to be load-bearing in a **second** place its own derivation never looked — it is what makes `resolveMember`'s whole-function scope guard safe, because a late-resolving fetch's record is already on disk and §5b guarantees the fill will not re-ask for it. 📌 *Runbook §10 carries the finding.* ⚠️ **AND THE LEG IS COMPILE- AND TYPE-VERIFIED ONLY — no joiner, no `not_found`, no badge observed. Every positive case is Leg F's, whose list grew by four.**

🔒 **NOT JOE'S, AND NOT BEING PUT UP:** §7's five `R-D` rulings. They are sequencing, naming and records — `D-123`'s Chat seat, where **under-stepping is the named failure mode** (J-618 · J-669 · J-670).

⚠️ **NOTHING IN LEG D TOUCHES APPEARANCE.** `ui/assets/skin.css` remains Joe's; C-3 — the `[data-unresolved]` base rule and `unasked` variant — is **Leg C's** remaining third, ungated since J-670 and **separate from this leg**.

📌 **RAISED ONCE, NOT ACTED ON:** the `CLAUDE.md` PLAY head. It is a 509-line file whose head carries ~169 KB in 139 lines and grew again last arc. **Whether it gets the J-662/J-663 treatment is Joe's.**
