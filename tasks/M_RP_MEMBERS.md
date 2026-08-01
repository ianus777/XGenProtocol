# M-RP-MEMBERS — the R7 members widget over the address book
> **Status**: ACTIVE  
> Version: 1.16  
> Date: Jul 2026  
> **Last updated**: 2026-08-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

**This is the address book's first caller.** `M-RP-ADDRESS-BOOK` (J-586) built the book and shipped it with **zero callers**: the widget is the thing that makes it run in normal operation.

**It is NOT:** the book itself (built, CLOSED) · a contact system (layer ③, `identity_private`, unbuilt and unspecified — Ch2 defines no acquisition flow at all) · presence (layer ④, unbuilt) · M13 wire widening (PENDING, separate) · a trust or moderation surface.

**Seats (D-123).** §4 (scope), §6 (unresolved-row form) and §7 (row shape) are **Joe's** — they are taxonomy, structure and appearance. Everything else in this document is Chat's: grounding, measurement, leg structure, verification. Chat proposes on Joe's side; proposing is not deciding.

---

## §1 — Grounding (measured 2026-07-25 at `fa27121`, HEAD, tree clean)

*Positive control (N-163): 257 `.rs` files scanned excluding `target/` and `.claude/`, **809** `pub fn` matches.*

**G1 — the book has never run.** `fill_from_space|AddressBook::|address_book::` → **43 hits, in exactly two files**: `xgen-client/src/address_book.rs` (22) and `xgen-client/src/ops.rs` (21). `app.rs` is **not** among them ⇒ there is no CLI verb either, not merely no UI caller. `xgen-client_address_book.json` exists **nowhere** in the AppData tree (`%APPDATA%` + `%LOCALAPPDATA%`, recursive) — the file has never been created in normal operation.

**G2 — the R7 substrate is complete, and the pattern is literal.** `ui/common/lib/components/widgets/spaces-panel.svelte` (63 lines) and `rooms-panel.svelte` (66) are the template, three parts each: a `$common` store, a `CLIENT_PLUGINS` descriptor (`kind:'system'`, `surface:'region'`, `regionId`), and a thin widget mapping a protocol row → `EntityDescriptor` → `EntityPanel`. `members` is **already** in `REGION_IDS` (`ui/client/src/layout-default.ts:18`), already titled `'R7 · Members'` (:32), already a leaf in the default layout (:110). It renders `RegionPlaceholder` today because **no `CLIENT_PLUGINS` row claims `regionId: 'members'`** — this milestone adds the **7th** `surface:'region'` system row. Frontend cost ≈ 70 lines of Svelte.

⚠️ **G2 IS WRONG WHERE IT MATTERS MOST — CORRECTED 2026-07-26 (Clair, found while opening Leg B; re-verified by Chat).** *"The pattern is literal"* was read as **`entity-panel` is a neutral list renderer**. **It is not. It is an interactive single-select listbox.** Measured: `entity-panel.svelte` is `<ul role="listbox">` (:138) of `<li role="option">` (:146) with `onclick={() => selectAt(i)}` (:150) and `onkeydown={onKey}` (:151); `selectAt` writes `selected = it.descriptor.id` **unconditionally** at :91, *before* calling `onActivate`. Skin: `.entity-panel-option { cursor: pointer }` (skin.css:2604-2608) and a `:focus-visible` ring (:2609-2612). **There is no `readonly`/`inert` prop** — the prop list is `items · title · badge · collapsible · collapsed · selected · onActivate · emptyText · id`.

🔑 **AND R1/R2 ARE ONLY CORRECT BECAUSE THEY CLOSE A LOOP R7 CANNOT.** `onActivate → selection.set(regionId, …)` (`rooms-panel:49`, `spaces-panel:50`) feeds the global bus, whose value flows back into their `selected`. **R7 is forbidden from writing that bus (Leg B runbook L15)**, so `selected` is `` one-way — a click writes the child's copy at :91, nothing propagates back, and **the wrong highlight STICKS until the roster changes.** In a group room, where the highlight must be `null`, **one click manufactures one.**

⇒ 🔒 **RESOLVED BY A NEW MILESTONE, NOT BY A WORKAROUND IN LEG B: `M-RP-PANEL-INERT` — `entity-panel` gains a non-interactive mode** (`tasks/M_RP_PANEL_INERT.md`). **It ships BEFORE Leg B.** ⚠️ *Chat first recommended reimplementing the list inside R7 (~70 lines) to avoid changing `entity-panel` — having read §1's "do not touch entity-panel" as a statement about COST when it is a statement about SCOPE. Joe asked whether that was not already decided; measurement then showed the `entity-panel` change is ~10 lines. **The avoidance was seven times the size of the thing avoided.***

**G3 — the cost is Rust, and it is a command, not a capability.** `xgen-client/src/desktop.rs` exposes **16** `#[tauri::command]`s (`get_state` · `get_pacing_state` · `get_conn_stats` · `get_resident_status` · `resume_resident` · `send_message` · `get_substitutions` · `set_substitutions` · `get_ui_state` · `set_ui_state` · `get_window_geometry` · `apply_window_geometry` · `get_about_info` · `get_self_state` · `get_spaces` · `quit`). **None touches the address book.** The widget cannot see the book from the frontend at all.

**G4 — the async-op precedent already exists and fits exactly.** `fill_from_space` is `async` + network; `get_spaces` is a sync local-file read, so it is *not* the precedent. `reanchor_space` (`desktop.rs:369`) is: build `SessionState::new(node, data_dir)` from `DataDir` + `ConfigPath` → `ensure_identity` → `ensure_connected` → run → `goodbye`. That is the shape `fill_from_space` wants, **and it satisfies the §4 off-critical-path lock for free**, because a Tauri command is already off the render path.

**G5 — `fill_from_space` is re-entrant by design (J-586) and must be left alone.** The wrapper clears `ctx.session.conn = None` on **every** exit including `?`-skipped error paths. Callers must add **no** connection management of their own, and the clears must not be tidied away.

**G6 — ⚠️ the name R7 can show is the BOTTOM of a three-layer chain, and the other two are unreachable.** Ch2 §User Representation (L557–580) locks the override chain **contact alias → Space nickname → global display name**. The book holds `display_name` = the **global** name only. The alias lives in `identity_private` (layer ③, 0 code hits). The Space nickname is **SPECIFIED-BUT-UNBUILT** — and `MemberEntry` (`ops.rs:2593`) carries `identity_id · role · joined_at · invited_by`, **no name of any kind**. ⇒ **R7 v1 renders the last resort of the chain for everybody.** Correct and honest, recorded here so nobody later reads a rendered name as one the user chose.

📌 **G6 WORDING CORRECTED v1.3** (Clair, Target 1 precision note; D-124 amendment ③ at `DECISIONS.md:4656-4660`, re-verified by Chat). Ch2 **assigns** the Space nickname a home; the field does not exist. D-124's amendment warns in terms about exactly this word choice — *"the measurement was correct; the word was not"* — so v1.2's *"lives in the Space membership record"* read more **built** than the code is.

📌 **AND IT IS NOT A COLLISION WITH D-124 (Clair, Target 1, CONFIRMED; Chat re-verified `DECISIONS.md:4645-4654`).** Ch2's **table** has FOUR layers; its **override chain** has THREE. The contact note is excluded by Ch2 itself — *"not a display name … it does not replace any label"* — and D-124's amendment states the same three-layer precedence. G6 describes the **chain** and is right to list three.

**G7 — the wire ceiling, four rules (runbook `RUNBOOK_ADDRESS_BOOK.md` §2, J-587).** `identity.record` carries seven fields and none of `update_version`, `revoked`/`revoked_at`, `trust_assertion`. Code and spec agree (`wire/types.rs:455-473`; Appendix I §IV.1) — decided, not drift. ⇒ the widget **cannot** show revoked · tier · assertion lapsed · a renamed identity. All four go live at **M13 Client Identity Lookup Widening** (PENDING). No part of M13 is built here.

---

## §2 — The measured surface the widget consumes

```
ops::fill_from_space(ctx, &mut book, space) -> FillReport
  { candidates, fetched, not_found, touched }

ops::members(ctx, &MembersArgs) -> MembersResult
  { space_id, is_dm, owner_id, members: Vec<MemberEntry>, events_replayed }
MemberEntry { identity_id, role, joined_at, invited_by: Option }

AddressBook::{ new, load, save, get, iter, len, is_empty, contains,
               insert, merge, touch, remove, clear, erase_file,
               evict_older_than }              (data_dir-based)
SeenRecord   { identity_id, display_name: Option, is_ai, home_node,
               last_seen, update_version, revoked, trust_assertion: Option }
SeenRecord::trust_lapsed(now) -> Option<bool>   (None = no opinion)
```

📌 `registered_at` is deliberately NOT stored. 📌 `ops::members` derives membership **client-side** by causal replay of the same drain F1/F2 read — no transport change, no second fetch.

---

## §3 — What v1 may honestly show

**MAY:** `display_name` (Option — absent is a real case) · `is_ai` · `home_node` · `last_seen`. And, if §4 locks B or C, **`role` and `joined_at`**, which are *not in the book at all* and arrive free with the `members` call.

**MAY NOT:** revoked · tier · assertion lapsed · renamed identity (§1 G7).

🔒 **THE HARD DISPLAY RULE — THE BOOK STORES OBSERVATIONS, NOT CURRENT TRUTH.** Every record means *as of `last_seen`, this was the state*. A cached `revoked: false` is true only as of then. ⇒ **staleness and absence BOTH render as UNKNOWN, never as fine.** A widget implying "everyone here is valid" would be lying, and lying in the exact direction the no-anonymity core exists to prevent.

⚠️ **CONCRETE CONSEQUENCE — DO NOT FEED `flags.revoked`.** `entity-avatar` already draws a revoked badge from `EntityFlags.revoked`, and the book's `revoked` is `false` on **every wire-filled** record because the wire never sets it. Feeding it would light a **shipped affordance from a constant false** — the N-097 shape that stranded `entity-item.selected` at M-RP6.1g, and the same rule this project used to keep `tabs` out of renderer A. ⇒ **v1 feeds `flags.isAi` and nothing else**; `revoked` stays unfed until M13.

📌 **PRECISION, v1.3 (Clair, Target 4; BOTH line numbers reproduced by Chat).** `revoked = true` does occur in `address_book.rs` at **L365** (struct literal) and **L508** (assignment) — both inside `#[cfg(test)]`, which begins at **L335**. They are the §5 merge-on-encounter seeds, **not wire records**. "Constant false" is a claim about the **production wire path the widget consumes**, and it holds. ⚠️ *Chat's own first probe searched only `revoked: true` and missed L508 — which would have made a correct report look non-reproducing. The N-099 family: a check whose pattern cannot see its subject still returns an answer, and the answer is the flattering one.*

📌 **NO OTHER FIELD REPRODUCES THE TRAP (Clair, Target 4, CONFIRMED).** `EntityFlags` is `{ isAi, revoked, isDm, e2e }`; `isDm`/`e2e` are **space** flags and do not apply to an identity row, so `isAi` — fed from the real wire `is_ai` — is the only meaningful flag. The one other structural constant, `update_version = 0` on wire records, is **not displayed** (§3's MAY list omits it; it is an internal merge-ordering signal), so it carries no constant-value render risk.

---

## §4 — DECISION 1: SCOPE — what is R7 a list OF? 🔒 JOE'S (architecture)

ROADMAP L826 says *"R7 is a contact list, not a room roster."* But the book is **global** (one file, everyone ever seen), `fill_from_space` is **Space-scoped**, R1→R2 are scope-chained, and Ch2 §Cross-Space Discoverability (L1859–1863) permits membership visibility **only within a Space**.

**A — the whole book.**
- *User-visible:* everyone ever seen, across all Spaces, in one list. ⚠️ It becomes the first screen in the client that composites identities **across Space boundaries**. 📌 **THREAT MODEL CORRECTED v1.3 — CHAT'S OVERSTATEMENT, CAUGHT BY CLAIR (Target 3b), RE-MEASURED BY CHAT.** v1.2 called this *"the correlation Ch2 §Cross-Space Discoverability exists to prevent, rendered by us."* **That overclaims.** `SeenRecord` carries **no per-Space field whatsoever** — grepping `address_book.rs` for `space` returns **three hits, all doc comments** — so the book is a **flat list of identities the local user has encountered**. A whole-book view therefore exposes *"who I have met"*, **not** *"which Spaces person X belongs to"*, and it is the latter that L1859–1863 protects (a third party correlating **someone else's** memberships). ⇒ A is a **weaker** exposure than v1.2 claimed. Moot under the B-lock, corrected anyway so a future reader re-opening §4 does not inherit an inflated threat model and refuse A for the wrong reason.
- *Cost:* lowest. No latch, no scope, no `members` call.

**B — the members of the latched Space.**
- *User-visible:* in a room, you see who is in it. Scope matches R4/R5/R6 and the panel's own title. Members whose record has not landed still occupy a row (→ §6).
- *Cost:* ≈15 lines over A — the `rooms-panel` D3 latch, already written twice. Adds the `members` call (already-drained events, no transport).

**C — B with an "all" toggle.**
- *User-visible:* both, one control. ⚠️ Two meanings behind one panel; the user must know which is showing.
- *Cost:* B + a toggle + a second empty state + a UI-state key. The toggle is itself a **surface** decision.

**Chat recommends B.** Under A, an eight-Space user gets a *Members* panel beside a stream listing people who are not in that Space, with nothing distinguishing them. A panel labelled *Members* scoped to nothing is an address book — and the address book has no visual home yet. B also keeps the book invisible behind its consumer, which is the ROADMAP's own framing: *the widget is one consumer of the book, not its viewer*.

⚠️ **NAMED AS A COLLISION, NOT TRADED (D-121).** If B locks, **§6 E2 per-entry delete has no UI to delete from** — a locked erasure control with no surface anywhere in the client. Not this milestone's to fix; **filed** against a future Settings / address-book surface rather than absorbed silently.

🔒 **LOCKED — B, the members of the latched Space (Joe, 2026-07-25).** ⚠️ **PROVENANCE: DELEGATED, not a considered lock.** Joe's words were *"lock all by your recomms"* — he adopted Chat's recommendation without walking the options himself. Recorded honestly so a future reader does not cite this as an independently reasoned architectural judgement, and so it is re-openable at low cost if the rendered panel reads wrong.

📌 **The §4 collision stands under the lock:** E2 per-entry delete still has no UI, and the whole-book view still has no home. Carried into §9, not closed.

### §4a — ⚠️ WHICH LATCH? A CHAT DEFECT, FOUND BY CHAT, ONE TURN AFTER THE LOCK

**The lock stands; its implementation note did not.** §4-B originally said *"the `rooms-panel` D3 latch reused verbatim"*. **That is wrong in a way that matters, and grounding `ui/common/lib/stores/room-latch.svelte.ts` said so in its own header:** *"THERE ARE TWO LATCHES AND THE NAME HIDES IT."*

| | What it is | Lifetime | Reachable from R7? |
|---|---|---|---|
| **R2's Space latch** | `let latchedSpaceId` **inside** `rooms-panel.svelte:23` — the Space you last clicked in the tree | widget | ❌ **NO.** It is a bare `let`, not exported. It is not a store. |
| **`roomLatch`** (`$common`) | app-lifetime store; **already exposes `effectiveSpaceId`** — the Space owning the room you are reading | app | ✅ already consumed by R5 (`stream-panel:69-70`), R6 (`composer-panel:68-69`) and the shell (`app_client:153`) |

⇒ **"Reuse R2's latch" was not merely suboptimal, it was NOT POSSIBLE** — copying it into R7 would create a **third** latch, the exact D-067 drift surface `room-latch.svelte.ts` was lifted to prevent, and the J-559 user-impact reversal is the direct precedent.

**And the user-visible argument runs the same way as J-559.** If R7 latched its own Space, you could be reading room X in Space A — stream and composer both on A — while the members panel shows Space B because you clicked B in the tree. **Members of a Space you are not reading, sitting beside a conversation you are.** That is the greyed-out-composer failure wearing a different coat.

🔑 **CHAT'S CORRECTED RECOMMENDATION — B1: R7 scopes off `roomLatch.effectiveSpaceId`.** One predicate, three widgets: the stream, the composer and the member list always describe **the same conversation**.

**The one honest cost, stated rather than hidden:** `effectiveSpaceId` is `null` until a **room** is latched, so **selecting a Space in the tree does not populate R7** — you must open a room. The alternative, **B2**, would lift R2's Space latch into `$common` as a second shared latch: ① *user-visible* — members appear one click earlier, at the price of reintroducing the A-vs-B divergence above; ② *cost* — a new store plus an edit to a shipped widget plus a second writer, against B1's zero new state and one getter read. **B1 recommended; B2 stays available later as a lift, exactly as `roomLatch` itself was one.**

🔓 **OPEN, and small: this is Chat's to implement but the empty-state copy is Joe's** — under B1 the no-scope empty state reads *"Select a room"*, not *"Select a space"*, because the room is what actually scopes it. ⚠️ **SUPERSEDED v1.5 — see §4c: R7 HAS NO EMPTY STATE.**

### §4b — 🔓 THE DM SPACE: REACHABLE, UNADDRESSED, AND JOE'S — D-122 FIRED HERE (Clair, Target 3c)

**The gap, in Clair's words and re-measured by Chat:** §4 never mentions `is_dm`. But `MembersResult` carries **`is_dm`** (`ops.rs:2606`), and `members_projection`'s own doc comment says it **deliberately** covers DM Spaces (*"so `members` covers DM Spaces — unlike `ai_status`, whose DM bail…"*). Under §4a-B1 a DM Space is reachable through the room latch like any other. ⇒ **R7 would render a two-row Members panel, one of which is you.** No correctness break; an unmade decision.

🔑 **CHAT'S ADDITIONAL GROUNDING, WHICH SETTLES THE SUB-QUESTION CLAIR COULD NOT REACH: the address book DOES contain self.** `fill_from_space_inner` (`ops.rs:2822-2875`) has **no self-exclusion anywhere**, and `observed_identities` (`ops.rs:2700-2723`) confirms it at source: F1 inserts every message author, F2 inserts **every projected member**, and you are a member of every Space you are in. `address_book.rs` returns **zero** hits for `self_` / `is_self` / `whoami` / `exclude`. ⇒ **your own XGID is written into `xgen-client_address_book.json`.**

⚠️ **AND v1.3's CONCLUSION FROM THAT WAS WRONG — CORRECTED v1.4, JOE'S QUESTION.** v1.3 said *"your own row resolves to your global display name; it will NOT render as an unresolved tail-8 label"* and treated that as reassurance. **The fact is right; the inference is backwards.** Self being in the book is **not** the thing that makes D1 safe — it is a **D-067 drift surface**, and §9 now carries it. See §9's SELF-IN-THE-BOOK entry.

🔒 **THE BINDING CONSEQUENCE FOR THIS MILESTONE (Chat's, technical): R7 RESOLVES SELF'S NAME FROM `selfState`, NEVER FROM THE ADDRESS BOOK.** `get_self_state` is **authoritative** (keypair- and state-derived); the book's copy of you is an **observation**, *as of `last_seen`*, and is stale by construction. M-RP-OWN-ROW-NAME already locked `selfState.identity.display_name` as the source for own rows in the stream. If R7 read self from the book instead, then after a display-name change **the same person would render under two different names in two panels at once**, until the next fill. ⇒ one fact, one home.

**D1 — render normally, self included.**
- ① *User-visible:* the panel means **the same thing everywhere** — a DM is a Space with two members, and it shows two members. Both names resolve (above). Mildly redundant beside a room header, never wrong.
- ② *Cost:* **zero.** No branch.

**D2 — suppress self.**
- ① *User-visible:* a *Members* list that omits a member, in the one place the omission is most visible: two people, one shown. And it mints a self-suppression rule R1/R2 do not have, which would then need justifying for group Spaces too.
- ② *Cost:* a filter plus a rule that spreads.

**D3 — R7 empty or special-cased in a DM.**
- ① *User-visible:* an empty panel beside a conversation with people in it **reads as broken** unless it carries explicit copy.
- ② *Cost:* a branch plus copy plus a third empty state.

**Chat recommends D1**, and the recommendation is **unchanged** by the self-in-the-book finding — self still gets a row; only the **source of its name** changes (`selfState`, per the lock above). A panel whose meaning changes with context is a panel the user cannot trust, and self appearing is not a wart — **you are a member**. It is also the only option that costs nothing.

🔒 **§4b IS EFFECTIVELY CLOSED BY §4c (v1.5).** The self-fixture lock **kills D2 and D3 by construction** — self can never be suppressed, and the panel can never be empty — so **D1 stands, but on a stronger footing than it was argued on**: self appears not *because it is a member of this Space* but because it is a **fixture of the panel**. ⚠️ What survives of §4b is only the other half: whether a two-person DM wants a members panel at all — a display question, not a data one.

⚠️ **SUB-QUESTION, FILED NOT DECIDED — should R7 MARK your own row?** **M-RP-OWN-ROW-NAME** locks *"Self"* plus distinct styling for **own rows only**, and it is explicitly a **message/stream** milestone — so it does **not** bind R7. But inventing a second self-marking mechanism here would be a **D-067 drift surface**, one concept with two implementations. **Chat proposes: v1 renders self as an ordinary, unmarked row; the marking question rides whenever OWN-ROW-NAME's styling exists**, so R7 consumes it rather than duplicating it.

🔓 **OPEN — JOE'S.** §4b is scope and taxonomy, his area under D-123. ⚠️ **It gates Leg B only.** Leg A is two Tauri commands that do not care whether a Space is a DM — **Leg A may open before this is answered.**

---

### §4c — 🔒 THE SELF ROW IS A FIXTURE, NOT A MEMBER (Joe, 2026-07-25)

🔒 **LOCKED (Joe):** *"in the list of members … there will be ALWAYS the self avatar. even if the client will be offline. this is the home for the self avatar"* · *"maybe always as the first, doesnt matter any filter"*.

⇒ **Three properties, and they are not the same property:**
1. **ALWAYS PRESENT** — regardless of scope, roster, connection or registration.
2. **ALWAYS FIRST** — ahead of the roster, independent of any sort.
3. ⚠️ **FILTER-IMMUNE — AMENDED 2026-07-25, SEE CONSEQUENCE 2b.** Originally *"survives any future search or filter"*. **Joe has since ruled that an ACTIVE SEARCH SHOWS RESULTS ONLY, WITHOUT SELF.** ⇒ the correct statement is **FIXTURE OF THE RESTING STATE**: self is unconditional while no search is running, and a running search replaces the list rather than adding to it.

🔑 **AND THIS IS THE CAPABILITY BOUNDARY, NOT A PREFERENCE — MEASURED.** `KnownSpace` (`xgen-common/src/state.rs`) is `space_id · name · node_endpoint · role · rooms` — **no members field** — and `drain_space_events` is an `async` op over the connection. ⇒ **there is no cached membership anywhere; offline the roster is unreachable.** The address book still resolves *names* offline, but not *who is in this room*. Meanwhile `get_self_state` is keypair-derived and needs no network. **So offline, self is the ONLY identity R7 can honestly render** — the lock names a constraint the machine already had.

⚠️ **CONSEQUENCE 1 — R7 HAS NO EMPTY STATE.** §4a-B1's *"Select a room"* empty state is **superseded**: the panel always holds at least one row.

⚠️ **CONSEQUENCE 2 — A FIXTURE THAT LOOKS LIKE A RESULT MAKES THE PANEL LIE. 🔒 THE PANEL HAS FIVE STATES, LOCKED (Joe, 2026-07-25):**

⚠️ **It is a TREE, not a 2×2** — with no scope there is no roster to ask about, and *"roster unknown"* is **three causes, not one**. Chat's earlier *"fourth cell"* framing was imprecise and is corrected here.

```
is there a scope?  (roomLatch.effectiveSpaceId)
│
├─ NO  ──────────────────► ①  self only, NO message
│                            (unregistered · no spaces · no room ever
│                             picked · latched room vanished)
│                            ⚠️ THIS ABSORBS OFFLINE
│
└─ YES → is the roster known?
         ├─ KNOWN ─────────► ②  self + all in room
         └─ UNKNOWN, three causes:
              ├─ in flight ────► ③  self + "I am waiting for the others"
              │                    ⚠️ REQUIRES A BOUNDED FILL — see §4c-i
              ├─ failed ────────► ④  self + "I cannot reach the others"
              └─ offline ───────► ⑤  self + "I cannot see the others"
```

🔒 **① UNSCOPED ABSORBS OFFLINE (Joe-confirmed by adopting the tree).** With no room picked there are **no "others" being blocked**; saying *"I cannot reach/see the others"* would blame the network for a scope the user has not chosen — **a second false statement**.

🔒 **③ → ④ IS A TRANSITION, NOT A SEPARATE RENDER** (Joe): *"if it is clear that request failed it changed to failed online appearance"*.

🔑 **CHAT ARGUED FOR SILENCE AT ③ AND JOE'S ANSWER DEFUSED THE OBJECTION RATHER THAN OVERRIDING IT.** Chat's case was warning-fatigue — a warning that flashes on every room open stops being read, and then ⑤ fails when it matters. But *"I am waiting for the others"* is a **status, not a warning**: different register, objection void. The five messages share one voice — the panel speaking about **what it knows about itself**.

📌 **④ vs ⑤ — both wordings are TRUE in both states, so the split is nuance, not correctness.** They are distinguished for the user by the **connection led**, not by the verb. ⚠️ **The thing that would make the distinction load-bearing is a RETRY affordance** — ④ is retryable, ⑤ is not. None exists today, so ④ and ⑤ may later collapse into one state at no cost to truth. Recorded so a future reader knows the split was a choice.

🔑 **AND JOE'S ⑤ WORDING IS BETTER THAN A CAUSE-SPECIFIC ONE WOULD HAVE BEEN.** *"I cannot see the others"* names the **effect and never the cause**, which is why it is correct across every unknown branch. *"You are offline"* would be false in ④; *"could not load"* would be false in ⑤.

⚠️ **REJECTED WORDING, AND WHY — recorded so it is not re-proposed.** *"I cannot see the others **online**"* was Joe's first cut for ④ and reads two ways: *while online* (intended) and *"the others **who are** online"* — **a presence claim**. Presence is layer ④, **unbuilt, with no client path and Space-scoped by design**. The panel must not assert the one thing it has no mechanism for. ⇒ *"I cannot reach the others"* (Joe, final).

🔑 **AND THE PREDICATE IS NOT "OFFLINE" — THIS PART IS CHAT'S AND KEYING IT WRONG REPRODUCES THE LIE.** The connection state (`DISCONNECTED` on the status-bar led) is only **one** cause of an unavailable roster; a drain that fails while **online** (node refuses, Space gone, timeout) leaves the panel in exactly the same position and would render *"only self"* with **no explanation at all**. ⇒ the honest predicate is three-valued **on the roster**, not two-valued on the connection:

- roster **KNOWN** → render it
- roster **UNKNOWN** (never fetched · fetch failed · offline) → **say so**
- roster known-and-empty → **unreachable by construction**: if you are scoped to it, you are in it

🔒 **BINDING DATA SHAPE ON LEG B:** the roster crosses into the widget as an **`Option`-shaped value, never a bare array** — an empty array MUST NOT be conflatable with *not fetched*. The precedent already ships in this arc: `SeenRecord::trust_lapsed(now) -> Option<bool>`, where **`None` means "no opinion"** rather than `false`. *Same discipline, same reason: absence is a value, and collapsing it into emptiness is how a panel comes to assert something nobody measured.*

📌 **The copy is Joe's** — all three messages are recorded **verbatim as given**, not as approximations.

### ⚠️ §4c-i — THE FILL IS UNBOUNDED, AND THE TIMEOUT IS A PRECONDITION OF ③ — NOT A DETAIL

🔑 **JOE'S QUESTION, WHICH RESHAPED THIS SECTION: *"how then you can say in flight?"***

**The frontend knows ③ from the PROMISE, not from the network** — the invoke was fired, the promise is pending, therefore *I asked and have not been answered*. That is observable with or without a timeout. **But the question cuts deeper, and it is right.**

⚠️ **③ IS NOT A KNOWLEDGE STATE.** The panel's knowledge is **binary** — roster KNOWN or NOT KNOWN. ③, ④ and ⑤ are **all "not known"**; what separates them is not information but **context and elapsed time**: ⑤ the led is red and we know why · ④ the attempt has **concluded** in failure · ③ the attempt has **not concluded**.

🔑 **⇒ ③ ONLY MEANS ANYTHING IF THE ATTEMPT IS GUARANTEED TO CONCLUDE.** Unbounded, *"I am waiting for the others"* is **a permanent condition wearing a state's clothes** — and saying *"I am waiting"* **commits the panel to eventually stopping**. ⇒ **the timeout is not a value attached to ③; it is what makes ③ SAYABLE.** *(Chat originally filed it as an open detail. Wrong category, corrected here.)*

🔒 **CONSEQUENCE: IF THE FILL IS NOT BOUNDED, ③ MUST BE DROPPED** and *unknown* collapses to ④/⑤ — a worse panel, but an honest one. **There is no third option in which ③ ships unbounded.**

📌 **And the failure hides itself:** navigating away and back fires a **fresh** fill, so the user experiences *"this room never shows its members"* rather than *"the app is stuck"*. Quieter, and still a lie.

🔑 **⚠️ SCOPE OF THIS SECTION SHRANK (2026-07-25, §5 rewrite).** Written when the roster was assumed to be a **repeated pull**. It is not — §5 measured a live membership channel ⇒ **the fill is the COLD START ONLY.** So ③ bounds **one read at the start of a scope**, not an open-ended subscription, and the timeout polices a single call rather than a standing condition. **Everything below still holds for that one call**, and the `FillLock` deadlock is **unchanged and still real**.

**Measured 2026-07-25:**

| Path | Code | Bounded? |
|---|---|---|
| connect | `session.rs:138` → `connect_url(node).await` → `connect_async(url).await` | ❌ **no timeout** |
| each of the N identity fetches | `identity_get_on` → `conn.recv().await` | ❌ **no timeout** |
| the drain | `sync_completion_timeout` (5 s default, config) | ✅ bounded |

⇒ **Joe's *"if it is clear that request failed"* MAY NEVER BECOME CLEAR.** A hung node leaves ③ *"I am waiting for the others"* on screen indefinitely — **true at t=0 and a lie by duration**, the same failure mode as every other state here, on a clock.

⚠️ **AND IT MAKES CHAT'S OWN LEG A RECOMMENDATION A LIABILITY — owned.** The `FillLock` mutex (Leg A runbook §3, Chat's call) is held across load → fill → save. With an unbounded fill, **one hung call holds the lock for the life of the process and NO further fill can ever run.** That is not a stale message; it is a **silently dead feature**, and it is worse than the read-modify-write race the lock was added to prevent. **The mutex is only safe with a timeout.**

🔒 **FIX — and Chat over-recommended last turn, corrected here.** Chat proposed a Rust timeout **and** a frontend escalation timer as equally necessary. **They are not.** A Rust-side `tokio::time::timeout` around the fill bounds the command, **releases the lock**, and makes the invoke **always resolve or reject** — at which point the frontend simply escalates ③ → ④ **on the rejection**, with no timer of its own. A separate frontend timer guards only against a stall *outside* the fill (the invoke bridge itself), and is **optional belt-and-braces, not a requirement**.

🔓 **OPEN — THE TIMEOUT *VALUES* ARE JOE'S** (the UX question *"how long before waiting becomes failed"*). ⚠️ **The timeout ITSELF is not open** — per the reframe above it is a precondition of ③, and the alternative is dropping ③, not shipping it unbounded.

⚠️ **CHAT'S FIRST PROPOSAL — A 20 s CAP ON THE WHOLE FILL — IS WITHDRAWN. WRONG MECHANISM, not just a wrong number.** Joe: *"a flight is euphemism for a progress which in it industry has time limit … when?"* An overall cap **aborts legitimate work**: a Space with 200 members is a legitimately long fill, and a fixed box would kill it mid-way. What *"in flight"* promises is not that the whole thing fits in a box — it is that **every step concludes**.

🔒 **CHAT'S DEFAULT UNTIL TOLD OTHERWISE — PER-STEP, NOT OVERALL:**

| Step | Bound | Status |
|---|---|---|
| `resolve_node` + `ensure_identity` | local file reads | already bounded |
| `connect_url` → `connect_async` | **5 s** | ❌ **missing** |
| `drain_space_events` | 5 s, `[sync] completion_timeout_seconds` | ✅ already there |
| each of N × `identity.get` `recv()` | **5 s** | ❌ **missing** |
| `book.save` | local write | already bounded |

📌 **5 s is not arbitrary — it is the number this codebase already uses twice:** `reanchor_space` bounds `get_dag_tips` at `Duration::from_secs(5)`, and the sync completion timeout defaults to 5. ⇒ **worst case 5 + 5 + 5N, and it always terminates.** The `[sync]` config key is the precedent for making these tunable rather than constants.

⚠️ **AND TERMINATION IS NOT THE WHOLE OF ③'S HONESTY — FILED, NOT BUILT.** For N = 1–2 (every Space that exists today) the cold fill is a second. For N = 200 it is minutes: **terminating, but ③ would sit there saying *"I am waiting"* with nothing moving.** True, and useless. The fix is **progress** — *fetched k of N* — but `fill_space_records` returns **once, at the end**; there is no channel. 🔒 **Deliberately NOT built:** it would be built for a case that does not exist yet. **The trigger is a Space large enough to notice**, and the milestone that hits it should find this note rather than rediscover it as a bug.

🔒 **CONSEQUENCE 2b — RESOLVED, AND NOT THE WAY CHAT FRAMED IT (Joe, 2026-07-25).** *"i meant filter as a temporary state, till you cancel it. if i looking for bob, for me it is search, and in this case are displayed just results without self avatar."*

⚠️ **CHAT HAD POSED THIS AS A STYLING PROBLEM AND IT WAS A STATE PROBLEM.** The open question was framed as *"how do we make a filter-immune self row not look like a match"* — offering either *scope rides visibly on the panel* or *self reads as a fixture*. **Both answers were wrong because the premise was:** self is not shown during a search at all, so nothing needs distinguishing.

🔑 **THE UNDERLYING DISTINCTION, WHICH THE ORIGINAL LOCK COLLAPSED:** *always present* and *filter-immune* were recorded as two of three co-equal properties. They are not co-equal. **Self is a fixture of the RESTING state.** A search is a **temporary mode you enter and cancel**, and while it runs the panel shows **results only** — self included in nothing, because self is not a result. ⇒ the panel never claims Joe matches *"bob"*, and it never has to be styled out of claiming it.

⚠️ **AND THIS CREATES A SIXTH PANEL STATE THAT §4c'S "NO EMPTY STATE" DOES NOT COVER.** Consequence 1 says the panel always holds at least one row — **true in the resting state, FALSE during a search that matches nothing.** ⇒ a search with no matches is a **legitimate empty result**, and it needs its own copy (*"no matches"*, distinct from all five resting states, and 🔓 **the wording is Joe's**). 📌 *Recorded here rather than left for the search milestone to discover as a bug — the resting-state guarantee and the search-mode guarantee are different guarantees.*

📌 **Binds whichever milestone adds a search; no filter ships in v1.**

⚠️ **CONSEQUENCE 3 — BINDING ON LEG B: ANY MEMBER COUNT DERIVES FROM THE ROSTER, NEVER FROM RENDERED ROWS.** Self renders when there is no roster at all, so a count taken off render-truth is wrong **exactly when it matters** (unscoped, offline). This inverts the usual render-truth getter convention **deliberately**: here render-truth and roster-truth diverge, and the roster is the one a human means by *"how many members"*.

📌 **R3 IS NOT A SECOND SELF SURFACE — the D-067 question is WITHDRAWN (Joe, 2026-07-25).** *"in the self panel there is not an avatar in the true sense … just today there it is, but not for long"* — R3's current avatar is **scaffolding**, and the region is to be rebuilt with its own layout and functions (M-RP-SELF-GATE). **The true self avatar is R7's.** ⇒ no milestone may design against R3's present avatar or invest in it.

🔑 **WHAT THE MEMBERS WIDGET IS FOR — JOE, 2026-07-25. THIS IS THE INTERACTION MODEL AND IT WAS NOT PREVIOUSLY WRITTEN DOWN ANYWHERE.**

> *"what for is the members' panel/widget? primary for dm and secondary for access to member's avatar settings/rmc. if you want to communicate with a group of people, you dont select those avatars in members list, but you create room and invite those people into it, by invitation in avatar's rmc."*

⇒ **PRIMARY: left-click a member → DM them.** ⇒ **SECONDARY: right-click → that avatar's context menu** (settings, and **invite-to-room**).

🔑 **AND THE NEGATIVE HALF IS THE LOAD-BEARING HALF: THE MEMBERS LIST IS NOT A GROUP-SELECTION SURFACE.** You do not multi-select avatars to start a group conversation. Group communication is **create a room, then invite** — and the invitation is issued **from the avatar's context menu**, not from the list. ⇒ **NO multi-select, NO checkboxes, NO "message selected" action, ever** — in this widget or any successor. 📌 *Recorded as a prohibition because a members list is the single most obvious place to add multi-select, and every future milestone will be tempted.*

⚠️ **BINDING ON LEG B — AND NONE OF IT SHIPS IN LEG B.** Leg B renders rows. **Left-click, right-click and the context menu are all LATER**, and Leg B must not ship a hover, cursor or affordance that promises any of them.

---

### §4c-ii — 🔒 THE DM MODEL: WHAT A DM IS, WHERE IT LIVES, WHEN IT IS CREATED, AND WHAT IS NOT KEPT

📌 *Established across one exchange with Joe, 2026-07-25/26. Recorded whole because the pieces only make sense together — and because two of Chat's contributions to it were wrong and are kept as such.*

#### The measured facts

| Fact | Where | Status |
|---|---|---|
| A DM is its own **Space**, created by a `dm_space_create` **root** — not a room inside an existing Space | `ops.rs:793` | ✅ measured |
| Creation signs and sends **three events over ONE connection, in order**: `dm_space_create` → `room_create` → `membership.invite`. Ordering is the correctness contract; room/invite are **not** pending-buffered | `ops.rs:767-792` | ✅ measured |
| The new Space records `node_endpoint = ` **the creator's home node** | `ops.rs:956` | ✅ measured |
| **Federation is ENFORCED OFF in DM Spaces** — a real error variant, not a convention | `xgen-core/src/space/state.rs:67` | ✅ measured |
| ⚠️ **NOTHING ROUTES BY `node_endpoint`.** Every op is `ensure_connected(ctx.node_override)`; `node_override` is a **context** field, never derived from a Space. `aicontrol` passes `None` at all 8 sites. The field is written and *printed* (`app.rs:2645`), never read to decide where to connect | `ops.rs` × many | ✅ measured |
| `ops::self_open` and `ops::create_dm_space` both exist, tested — **neither has a Tauri command** (`desktop.rs` has 18, none reach them) | `ops.rs:1002` / `:793` | ✅ measured |
| `state.space_migrate` is a real wire event; identities can **rehome** | `wire.rs:192` · `registry.rs:321` | ✅ measured |
| 🔑 **THE CLIENT PERSISTS NO CONVERSATION CONTENT AT ALL.** `xgen-client` has **zero** dependency on `xgen-store-sqlite` (the node has 8). It writes five files: pid, address book, config, keypair, state, uistate | `xgen-client/Cargo.toml` | ✅ measured |

🔑 **⇒ THE CLIENT IS A PURE VIEW OVER THE NODE'S DAG.** Everything displayed was drained moments earlier and discarded. **This one absence is also why §4c's roster is unreachable offline** — offline history, offline roster and DM durability are **three symptoms of one missing client-side store**, not three problems.

#### Joe's four propositions, corrected

1. ✅ **"I can DM any member — known or unknown — of a node I am valid on."** Correct. The DM Space is a root, independent of the room latch and of whatever Space is in scope. `create_dm_space` takes an **identity**, not an existing relationship.
2. ⚠️ **"To DM a member of another node I must be valid on their node" — THE DIRECTION INVERTS.** The DM is hosted by **whoever creates it**. If you initiate, it lives on **your** node and **they** must be valid there; you need nothing from theirs. You need validity on their node only for DMs **they** started.
3. ⚠️ **"With many friends I must be valid on all their home nodes" — only for the DMs they started.** 🔑 **But this exposes the real weakness: NOBODY DECIDES WHERE A DM LIVES — it is a side effect of who clicked first.** Ten friends can scatter across ten hosts by accident, chosen by neither party. ⚠️ **And it makes the duplicate case likely rather than exotic:** both click, two Spaces exist on two nodes, federation is off, and **nothing reconciles them** — a conversation that silently splits in two.
4. ⚠️ **"The system should log me in automatically to his node" — coherent, and NOT what exists.** Three pieces are missing: **routing** (no op reads a Space's `node_endpoint`), **multi-node sessions** (`SessionState` holds ONE connection; ops null it and re-dial), and **auto-auth** (nothing selects a node from a Space and authenticates unprompted). 📌 *The good news is architectural:* **`node_endpoint` already stores exactly what this needs and nothing reads it** — a decision made and never wired, not a gap to design around.

#### 🔒 CREATION IS LAZY — LOCKED (Joe, 2026-07-26)

**LMC on a member → open the DM thread. No confirmation dialog.** Joe's evidence was Discord's own behaviour: opening a never-contacted person shows an **empty thread** (*"This is the beginning of your direct message history with …"*), no modal.

🔑 **THE REASON IT IS SAFE IS THAT THE OPEN IS LAZY, NOT THAT IT IS GUARDED.** Opening sends nothing; the recipient learns nothing. **The irreversible act is the FIRST MESSAGE, not the click.**

**Procedure:**
- **Open** → scan known Spaces for an existing DM with that identity. **Exists** → open normally. **Does not** → open a **DRAFT THREAD**: an in-memory object holding only the target identity — no `space_id`, no `room_id`, nothing signed, nothing sent. Header renders from the address book, which is why it can name someone never messaged.
- **First send** → `create_dm_space` (3 events) → message tip-chained onto the new room (**a 4th**) → draft promoted in place.

⇒ **Opening ten drafts and sending to none leaves ZERO trace, local or remote.**

⚠️ **CHAT PROPOSED A CONFIRMATION DIALOG AND IT WAS THE WRONG SHAPE.** Chat argued the click writes to the DAG so it must be guarded. **The better answer was to stop the click writing at all.** 🔑 *Chat reached for extra MECHANISM where an existing STATE already answered it — the second time in one session (see §4c consequence 2b, posed as a styling problem when it was a state problem). A bias in one direction, not random error.*

⚠️ **OPEN — WHAT A PARTIAL FIRST SEND LEAVES BEHIND.** Lazy creation moves the irreversible act to first-send; **it does NOT make it atomic with the message.** Fails at create → clean, draft survives, retry. **Create succeeds, message fails → a Space exists and the recipient holds an invitation from you with no message in it** — exactly what lazy creation was adopted to prevent, relocated into a crash window. Torn inside the 3-event create → a Space with no room, or a room with no invite. 🔓 **Whether an invite-without-message is a defect or a legitimate state (*"someone started a conversation with you"*) is Joe's, and belongs in the interaction milestone's Phase-0.**

#### 🔒 NO CLIENT-SIDE BACKUP — LOCKED (Joe, 2026-07-26): *"nothing will change. we have backup of the node, that has to be enough"*

The options were **A** leave it (node is the sole store) · **B** client-side archive · **C** node-to-node DM replica (= federation, enforced off).

🔑 **JOE LOCKED A ON THE ARGUMENT THAT SETTLED IT: NEITHER PARTY HAS A COPY, SO THERE IS NO ASYMMETRY TO CORRECT.** ⚠️ **Chat had claimed an asymmetry** — *"her history lives on your node at your discretion"* — implying the host's user holds something. **They do not.** Both parties are pure views; only the node stores. The dependency is on **who controls the node**, which is narrower than possession, and the node's own backup already covers it.

⚠️ **THE ACCEPTED COST, NAMED SO IT IS NOT REDISCOVERED AS A BUG:** federation off means **exactly one copy exists, on one node, with no replica anywhere**. If the host node is lost, the conversation is lost for both parties. `state.space_migrate` is the escape hatch — **a deliberate act taken BEFORE the node is lost**, not resilience.

📌 **AND IF A CLIENT-SIDE STORE IS EVER BUILT, IT IS NOT A DM FEATURE.** It is a **client cache layer**, and offline history, the offline roster (§4c) and DM durability all fall out of it together. **Sizing it as "should DMs be backed up" would size it wrong.**

#### 🔒 HOST ASSIGNMENT — **RESOLVED: H4, BILATERAL REPLICATION (Joe, 2026-07-26): *"h4 definitely"***

⚠️ **PROVENANCE: CONSIDERED.** Joe asked for *"rather proper than cheap"*, then asked for the options **asserted against T4** — and that pass is what decided it. 📌 **The T4 assertion was promoted to a standing project lens the same day — see D-121 lens 3.**

**The options as they stood:**

| | Shape | ① User-visible | ② Tier consequence | ③ Resource |
|---|---|---|---|---|
| **H1** | host by race (status quo) | none today | ⚠️ **compliance regime decided by who clicked first**; T4 floor = one copy | zero now, unbounded later |
| **H2** | deterministic host (lower XGID's node) | duplicates impossible; auto-login is arithmetic | T4 floor still one copy | small |
| **H3** | race + a general discovery mechanism | solves host lookup **and** cross-node invite discovery together | T4 floor still one copy | medium, new subsystem |
| **🔒 H4** | **bilateral replication** — the DM lives on **both** participants' home nodes, synced between exactly those two | nobody registers on a foreign node · nobody depends on the other's operator · node loss survivable | ✅ **two independent ciphertext copies ⇒ a real durability floor**; each node applies **its own tier to its own copy** | largest — a core protocol change |

🔑 **WHAT THE T4 PASS CHANGED, AND IT REVERSED CHAT'S LEAN.** Chat had costed H4 as *"two operators see the content instead of one"* and weighted it heavily. **D-093 clause 1 makes that false: universal E2E, no protocol escrow, the node CONTENT-BLIND at every tier.** Operators hold ciphertext they cannot read ⇒ **the number of nodes holding a DM does not change who can read it.** With the objection gone, three positives stood unopposed:

1. 🔑 **A durability floor with one copy is not a floor.** Retained (T4) means *do not drop these bytes*; every single-home option leaves exactly one copy on one machine.
2. **The erasure model already assumes replicas.** D-093 clause 2: destroy the per-blob key ⇒ *"every replica, **including unreachable federated homes**, is permanently unreadable."* ⇒ **replication does not weaken crypto-shred** — the guarantee is key-destruction, not byte-deletion.
3. **Under H1, whose retention regime governs a whole conversation is answered by who clicked first.** ⚠️ *That is not a decision, it is a race — and it is the same shape D-093 clause 3 forbids at the blob layer: one physical copy carrying two records with two erasure-fates, one policy silently overriding the other.*

⇒ **H4 dissolves host assignment, cross-node discovery and duplicate reconciliation simultaneously** — each party reads and writes their own node, so there is no host to assign, nothing to discover, and no foreign validity to acquire. **It also removes lock A's accepted cost without building a client cache.**

#### ⚠️ WHAT H4 REQUIRES — TWO CHANGES, ONE OF THEM UNMEASURED

1. **`DmFederationNotAllowed` NARROWS** from *"no federation"* to *"federation restricted to the participant set"*. 📌 **The guard's own comments already say this is the intent** — `runtime.rs:2224` *"no **third-party** node ever receives DM content"* and `state.rs:2957` *"privacy as a structural property"*. **A participant's own home node is not a third party**, so the narrowing serves the stated purpose rather than eroding it.
2. 🔑 **A PAIR-DERIVED SPACE IDENTITY.** If `space_id` derives from the two **sorted XGIDs**, both parties creating independently produce the **same id** — the DAGs merge the way DAGs do, and **duplicates become structurally impossible rather than reconciled after the fact.**

⚠️ **UNMEASURED, AND IT DECIDES WHETHER H4 IS A MILESTONE OR A REWRITE:** space ids are `xgen://hash/sha256:…`, content-addressed on the **signed root event** — and two people signing different roots produce different hashes. **Whether the id scheme can derive from the identity pair instead has NOT been checked.** 📌 *Chat offered to measure it before the lock; Joe locked H4 on principle first. The measurement is still owed and is Chat's.*

#### 🔓 STILL OPEN INSIDE H4 — THE ONE PLACE IT MAY YET HAVE A REAL COST

**If Bob crypto-shreds, does Alice's T4-retained copy survive?** This turns on **whether the content key is wrapped per-recipient**: if each party holds their own wrapped key, Bob destroying his does not touch Alice's; if there is one key, **Bob's erasure defeats Alice's retention obligation.**

⚠️ **The wrapping mechanics are UNMEASURED and must not be assumed.** 📌 *Note this tension exists under EVERY option — it is inherent to a two-party record whose parties may sit at different tiers — but **H4 is the only option that gives each party a copy under their own policy to resolve it against.*** This is the live edge of the project's standing GDPR-vs-append-only tension, not a detail of DM hosting.

#### 📌 UNMEASURED — **CHAT'S TO MEASURE, NOT JOE'S TO ANSWER.** Owed before the interaction milestone, not before Leg B

⚠️ **These are MEASUREMENTS, not open decisions.** *They were first filed under 🔓 (Joe's lane) — a mislabel Joe caught. Items 1 and 4 are greps. Items 2 and 3 are greps that MAY promote into Joe decisions depending on what they find; until measured, there is no question to put to him.*

1. **Are DM Spaces eligible for `state.space_migrate`?** DM constraints exist (`runtime.rs:2877`); whether they block migration is unknown. ⚠️ **This decides whether a DM can ever be moved off a dying node** — and under lock A it is the *only* remedy.
2. **Does anything reconcile duplicate DM Spaces** for the same pair on two nodes?
3. **How does an invitee on another node learn of an invitation** they must connect to that node to see?
4. **Does any draft-thread concept exist in the client today?**

⚠️ **NONE of the above blocks Leg B**, which renders rows and wires no interaction.

---

⚠️ **THE SELF ROW AND THE OTHER ROWS DO NOT DIVERGE AFTER ALL.** Chat had recorded an asymmetry — member rows workable, self row not — on the assumption that the self thread was unbuilt. **Measurement (below) shows both verbs exist and both lack only a Tauri command.** ⇒ **no asymmetry to design around.**

🔑 **MEASURED 2026-07-25 — THE VERBS ALREADY EXIST. CHAT'S "THE TARGET DOES NOT EXIST" WAS WRONG, AND THE ERROR IS KEPT BECAUSE IT WAS MADE AND CORRECTED WITHIN ONE EXCHANGE.**

| Verb | Where | CLI | batch | aicontrol | ⚠️ **Tauri** |
|---|---|---|---|---|---|
| `ops::create_dm_space` | `ops.rs:793` | ✅ | ✅ | ✅ | ❌ |
| `ops::self_open` | `ops.rs:1002` | ✅ | ✅ | ✅ | ❌ |

⇒ **BOTH interaction targets are BUILT at the ops level.** `self_open` returns `SelfThreadResult`, is **create-if-absent idempotent**, and carries an integration test (`self_open_creates_then_is_create_if_absent_idempotent`). `SELF_THREAD_LABEL` (`ops.rs:752`) is the shared key, applied by `create_dm_space` when invitee == creator and scanned by `self_open` — **one source of truth, already written.**

⚠️ **WHAT IS MISSING IS THE UI SURFACE, AND ONLY THAT.** `desktop.rs` holds **18** `#[tauri::command]`s and **none** of them reaches either verb. 🔑 **This is the M-RP-MEMBERS LEG A SHAPE EXACTLY — the capability existed and the command did not.**

📌 **CONSEQUENCE FOR THE INTERACTION MILESTONE: IT IS A COMMAND-SURFACE MILESTONE, NOT A FEATURE MILESTONE.** Wrapping two existing, tested `ops::` verbs as Tauri commands — the work Leg A already did twice for the book. **Materially cheaper than "build M11 first"**, which is what Chat's wrong claim would have implied to whoever read it.

⚠️ **CHAT'S ERROR, NAMED:** *"the self thread is D-021 / M11, post-M10, and is not built"* was inferred from the ROADMAP position of M11 and **never checked against the code**. 🔑 *A roadmap says what is planned; it does not say what exists. The claim was one grep from being right and Chat asserted it instead — this arc's defect class, committed by Chat for at least the sixth time, and caught only because the claim was measured immediately after being written.*

🔒 **BINDING ON LEG B — UNCHANGED, BUT FOR THE RIGHT REASON NOW.** The self row and member rows still ship **inert** in Leg B. ⚠️ **The justification is SCOPE (Leg B renders rows), NOT absence of capability.** Leg B must not ship a hover, cursor or affordance promising an interaction it does not wire.

🔒 **THE SELF ROW ACTS, AND IT OPENS A DM (Joe, 2026-07-25).** *"self row in members' region? if so, it opens dm. similar to that in skype"* ⇒ clicking self opens the **self thread**, the Skype shape.

⚠️ **CHAT'S EARLIER READING WAS WRONG AND IS RECORDED AS WRONG, NOT REPLACED.** Chat had read the self row as opening the **Self Card** (the View-menu proposal) and filed it as an unconfirmed reading. It is not the Self Card. *The reading was never confirmed and correctly carried as a reading — this is what that discipline is for.*

🔑 **AND THE RULING'S TARGET EXISTS — SEE THE MEASUREMENT ABOVE.** `ops::self_open` is built, tested and idempotent; only its Tauri command is missing. ⚠️ **Chat initially recorded the opposite** (*"wiring the click would ship a lit affordance with nothing behind it"*) and corrected it on measurement within the same exchange.

🔒 **BINDING ON LEG B: THE SELF ROW SHIPS INERT — ON SCOPE GROUNDS.** No click handler, no hover or cursor affordance implying one. **The design is decided here; only the wiring waits.** ⇒ the interaction milestone inherits this ruling and must find it rather than reinvent it — and since both verbs already exist, **lighting the row is a command-surface step, not a redesign.**

🔒 **§4b NO LONGER BLOCKS LEG B (2026-07-25).** Its data half closed by construction at §4c; its remaining half was *"does a two-person DM want a members panel at all"*. Under §4c the panel renders **self + roster** everywhere, so a DM resolves by the ordinary rule — you and them — with **no special case to write**. ⇒ nothing about §4b needs answering before Leg B; if Joe later wants a DM to suppress or restyle the panel, that is an additive display change, not a prerequisite.

---

## §5 — DECISION 2: WHEN DOES FILL RUN? 🔒 Chat's, stated for the record

🔒 **Locked upstream (Joe, address-book §4): FILL RUNS OFF THE CRITICAL PATH.** The Space opens **at once**; records resolve behind and the view updates as they land. **Never gate a Space open on the fetch loop** — at N members that is an unbounded network wait in front of a UI action.

**Chat's implementation reading — ⚠️ REWRITTEN, THE PREMISE WAS WRONG (Joe, 2026-07-25).** This section previously said the trigger is a change in `roomLatch.effectiveSpaceId` and stopped there — **a pull-on-scope-change model**. Joe challenged it: *"the member list has to be dynamic, every time when someone appear in the room … or in some rate list checks who is in the scoped room."* **He is right, and the live mechanism already exists.**

🔑 **MEASURED END TO END, 2026-07-25 — MEMBERSHIP CHANGES ARE ALREADY PUSHED TO CONNECTED CLIENTS, AND NOTHING FILTERS THEM.**

| Link | Code | Result |
|---|---|---|
| membership is a DAG event | `xgen-common/src/wire.rs:43-58` | **eight variants** — `MembershipInvite · Join · Leave · Kick · Ban · NodeEject · NodeUnban · Mute` |
| recipients | `xgen-node/src/fanout.rs:272` — `space.members.keys()` | **every member of the Space** |
| joiner catch-up | `fanout.rs:277` — only when `req.new_joiner.is_some()` | **additive history, NOT the delivery rule** |
| delivery | `fanout.rs:308` — `senders.get(rid)`, every connection | already-connected members included |
| the `:217` type match | `derive_event_nodes` | **the observer FILTER dimension — not an exclusion** |
| resident receive | `xgen-client/src/resident.rs:395` | no `event_type` match |
| resident forward | `resident.rs:407` — `(io.ingest)(ev)` | **verbatim** |
| shell | `ui/client/src/app_client.svelte:517` — `ingest.push(event.payload)` | **verbatim** |

⇒ **a `membership.*` event in a Space you belong to already reaches the frontend today, while connected. The roster can self-maintain.**

🔒 **THEREFORE THE FILL IS THE COLD START, NOT THE MECHANISM.**
- **Cold start** — on a change in `roomLatch.effectiveSpaceId` (§4a): one `fill_space_records`, off the critical path. ⚠️ **De-duplicate**: `effectiveSpaceId` does not change when moving between rooms **within** a Space, so fire on the **Space** transition only.
- **Steady state** — a `membership.*` event for the scoped Space refreshes the roster. No second fetch of the whole DAG; the event carries the change.

🔒 **POLLING IS REFUSED.** Joe offered *"or in some rate list checks"* as the alternative. The push path makes it unnecessary, and a poll would be **a second source of truth for something already delivered** — D-067, and it would also burn N `identity.get` round trips on a timer.

🔑 **AND THE ONE EXCLUSION IN THE FAN-OUT INDEPENDENTLY JUSTIFIES §4c'S FIXTURE LOCK.** `fanout.rs:302` excludes the **author by identity** — *"an author's own connections do not see their own posted event echoed"* — and **every member authors their own `membership.join`** (J-586). ⇒ **you never receive your own join**, so a roster built purely from the live channel would be **missing you, invisibly**. Joe locked the self row as a fixture on offline-honesty grounds; it turns out to be **independently required** by the fan-out. *Two unrelated arguments, one answer.* 📌 The cold read is unaffected — `ops::members` replays the drained DAG, which contains your own join.

⚠️ **WHAT IS **NOT** PROVEN, AND IS NOT CLAIMED: this path is traced STATICALLY.** No membership event has been **observed arriving live** — the Leg A live pass exercised only the fill. ⇒ **Leg B's DoD carries it as a required leg, EXERCISED not asserted** (two clients, a shared Space, a real join). *A traced path is a hypothesis with good sources.*

⚠️ **AND THE CLAIM THAT MISLED THIS SECTION IS STILL IN THE RECORD.** `tasks/M_RP_ADDRESS_BOOK.md` §5 (Leg A, J-579) enumerates the DAG event space as *"`message.*`, `state.*` and `space.join_request`"* — **the entire `Membership*` family is omitted**. Its *conclusion* (no `identity.*` event exists) is **correct**; its *enumeration* is not, and Chat inherited it here as *"the roster is a network read"* without checking. **Corrected in place at the source doc.** 📌 Same defect class as J-587 and as §4a's latch: **a claim narrower than the thing it described, reused as if complete — third occurrence this arc.**

⇒ **the off-critical-path lock is EXERCISED, not asserted** (Leg C): open a Space, assert the panel paints before the fill returns, then assert rows resolve afterwards.

---

## §6 — DECISION 3: WHAT AN UNRESOLVED ROW LOOKS LIKE 🔒 JOE'S (appearance)

Three distinct truths, and conflating them is the trap:
1. **Member known, record held** → name (or, if `display_name` is `None`, the same fallback as 2).
2. **Member known, record not yet fetched** → transient; resolves in seconds.
3. **Member known, `identity.get` returned `not_found`** → permanent for now; the book is deliberately left unpoisoned (`FillReport.not_found`).

📌 **D-126 humane pubkey label is very likely this milestone's first real consumer** — it is the natural render for 2 and 3.

**Chat proposes, does not decide:** **tail-8** for v1 (zero new files, already the pattern in `app.rs`/`ops.rs`; D-126 names it the cheapest family), and the **word form deferred**. Canonical-vs-cosmetic, the wordlist and its language are **open and Joe's** under D-126 and are not settled by this milestone.

⚠️ Whatever the form, 2 and 3 must be **distinguishable** or the panel tells the user a temporary state is permanent. Distinguishing them is appearance; *that* they be distinguishable is the honesty rule and is not negotiable.

🔒 **LOCKED — tail-8 for v1; the D-126 word form DEFERRED (Joe, 2026-07-25).** ⚠️ **PROVENANCE: DELEGATED** (*"lock all by your recomms"*), not a walked appearance decision — and this is the one of the three where that matters most, because it is the first thing a user sees on a row that has not resolved. **Re-open freely on sight.**

📌 **D-126's other openings are NOT settled by this lock:** canonical-vs-cosmetic, the wordlist and its language remain open and Joe's. This milestone consumes the cheapest family and reserves the rest.

### 🛑 §6a — THE LOCK AND THE SHIPPED BUILD DISAGREE: `tail-8` WAS LOCKED, THE CODE RENDERS THE CONSTANT HEAD (filed by `M-RP-LIVEFEED-REFRESH` Leg A, 2026-08-01, J-643 — FILED, NOT FIXED)

**Measured on the render path, not inferred:**

```
toDescriptor (members-panel.svelte):  name: rec?.display_name ?? tail(m.identity_id)
tail = (xgid) => xgid.split('/').pop() || xgid
```

🛑 **`tail()` IS NOT A TAIL-8. IT RETURNS THE WHOLE FINAL PATH SEGMENT, AND THE CLIP THEN TAKES THE WRONG END.** An identity id is `xgen://pubkey/ed25519:<base64>` (`auth.rs:133` · `mod.rs:267`) ⇒ `tail()` yields **`ed25519:<~44 chars>`**. `entity-item.svelte:123` renders it into `.ei-name`, styled `overflow:hidden; text-overflow:ellipsis; white-space:nowrap` (`skin.css:2452-2458`) — **left-anchored, no `direction: rtl`.**

⇒ **The user sees `ed25519:AbCd…`, and the `ed25519:` prefix is IDENTICAL on every identity in the system.** ⚠️ **The clip discards precisely the distinguishing bytes, so TWO unresolved members are indistinguishable FROM EACH OTHER** — not merely from resolved ones. **The locked `tail-8` would have kept the distinguishing end; the build keeps the constant head.**

🔑 **THIS IS A LOCK-VERSUS-BUILD GAP, NOT A DESIGN QUESTION.** Joe locked `tail-8` at J-588 (⚠️ delegated, *"lock all by your recomms"*, and §6 already flags this as the one of the three where delegation matters most). **Nothing re-opened it; the code simply does something else.** 📌 **Filed here rather than fixed because it is `M-RP-MEMBERS`'s, it blocks no leg of `M-RP-LIVEFEED-REFRESH`, and silently repairing a shipped divergence from a Joe lock is not Chat's call.**

⚠️ **AND §6's OWN NON-NEGOTIABLE IS THE THING AT RISK:** *"2 and 3 must be **distinguishable** or the panel tells the user a temporary state is permanent."* **Under the shipped `tail()`, cases 2 and 3 are not distinguishable from each other OR from any other unresolved row.** 🔓 **The fix and its form are Joe's.**

---

## §7 — DECISION 4: ROW SHAPE 🔒 JOE'S (appearance)

`entity-item.svelte` carries the `row · card · nav · inline` variant axis; `entity-panel.svelte:155` hardcodes `variant="row"`. Rows expose `secondary` · `meta` · `status` slots, **all currently UNFED** in R1 and R2 (D6/D-065 — no faked topic, no faked unread).

R7 is the first region with **real** candidates: `role` → `secondary`? `last_seen` → `meta`? Or ship them unfed and keep v1 shape-identical to its siblings.

⚠️ `status` is the Track A **self-status** slot (personal expression, not presence). Presence is layer ④ and **unbuilt** — nothing in this milestone may put a dot beside a name.

🔒 **LOCKED — `secondary` / `meta` / `status` ship UNFED; v1 is shape-identical to R1/R2 (Joe, 2026-07-25).** ⚠️ **PROVENANCE: DELEGATED** (*"lock all by your recomms"*). Adding a slot later is purely additive, so the cost of re-opening is a prop, not a redesign.

📌 **Consequence, stated plainly:** `role` and `joined_at` arrive free with the §4-B `members` call and are then **deliberately not rendered**. That is a real datum discarded on purpose — recorded so the next reader knows it was a choice, not an oversight.

---

## §8 — Legs

**Leg 0 — Phase-0.** This document. Grounding + the three open decisions. ⚠️ **Read against Ch2 §User Representation and Ch6 by a second reader before any runbook is opened** — the J-587 structural fix applied rather than quoted (four defects last arc, one class: a single author writing against a spec they were simultaneously interpreting; every one caught by a second reader working from the source). No code.

**Leg A — the Rust read surface.** Two thin commands in `desktop.rs`: a sync book read (`get_spaces` shape, `DataDir`) and an async fill trigger (`reanchor_space` shape, G4). Moves the **cargo floor** (baseline 1585/0/62 across 56). ⚠️ **D-129 checked and does NOT fire:** each call builds a fresh session and drops it; nothing is made persistent across ops, so `ensure_connected` is untouched. Stated so the next arc knows it was checked, not skipped.

**Leg B — the store + the widget.** `$common` address-book store + `members-panel.svelte` + the 7th `CLIENT_PLUGINS` row + shell hydration. **Scopes off `roomLatch.effectiveSpaceId` (§4a) — no new latch, no edit to `rooms-panel.svelte`.** Frontend only; moves the **svelte-check floor** (baseline 0 err / 34 warn / 15 files).

**Leg C — live CDP verify (9222).** Registry delta · the §5 off-critical-path lock **exercised** · the §6 unresolved-row render, both cases · the honest empty states · churn-returns-to-baseline as the orphan proxy (N-092a: the client bridge is state-only, there is no `domCount` leg).

🔒 **REQUIRED LEG, ADDED 2026-07-25 — LIVE MEMBERSHIP, EXERCISED NOT ASSERTED.** §5's push path is traced **statically only**; no membership event has been observed arriving. ⇒ **two clients, one shared Space, a real join** — assert the already-connected client's roster updates **without a re-fill**. ⚠️ **And assert the author exclusion too:** the **joiner's own** client must NOT receive its own `membership.join`, which is precisely why the self row is a fixture. A verify that only watches the observer proves half the mechanism.

**Leg D — records + close.** JOURNAL + CLAUDE.md PLAY + ROADMAP + this doc, **one commit** (D-074).

📌 **A and B split deliberately:** A moves the Rust floor, B moves svelte-check. One commit spanning both makes a regression ambiguous.

---

## §8a — 🛑 THE BLOCKER: §5's STEADY STATE WAS NEVER BUILT, AND NO LEG WAS EVER ASSIGNED TO BUILD IT 🔓 JOE'S (architecture)

**Found by Clair opening Leg C; verified independently by Chat from the code at `0466cb2`, 2026-07-26.**

**THE MEASUREMENT — four reads, not a report:**

| Claim | Code | Result |
|---|---|---|
| the only `addressBook` writers | `app_client.svelte:173/:176/:183/:187` | **all four inside `loadMembers`** |
| the only `loadMembers` caller | `app_client.svelte:169` | inside the `$effect` whose **sole** tracked dependency is `roomLatch.effectiveSpaceId` (`:168`) |
| what the live event reaches | `app_client.svelte:551-552` — `xgen-event` → `ingest.push` | **R5's store.** Nothing bridges `ingest` → `addressBook` |
| the store's surface | `ui/common/lib/stores/address-book.svelte.ts` (145 lines) | **four methods only** — `setInflight · setResult · setFailed · reset`. No `addMember`, no `removeMember` |
| `membership` anywhere in the shell | grep, `app_client.svelte` | **zero occurrences** |

⇒ **R7's roster NEVER CHANGES after the fill until the room is re-latched.** §5 locked the steady state as the live membership channel; §8's Leg C was assigned to *verify* it; **no leg was ever assigned to build it.** Leg C's REQUIRED LEG asserts a behaviour that does not exist.

📌 **§5's push path is otherwise CONFIRMED, and this is the half that does hold.** Re-measured 2026-07-26: `apply_fanout` (`fanout.rs:244`) is called from `app.rs:2020` once per accepted locally-submitted event, **before any event-type branch**, and is generic over the event — the only `event_type` match inside it is `derive_event_nodes` (the observer *filter* dimension, §5's table). ⇒ **`membership.join` rides exactly the same fan-out path as the `message` test at `fanout.rs:843`, and the author exclusion at `:301-307` applies to it identically.** The event does reach the frontend. **Only the last hop — frontend event → roster — is missing.**

### THE FOUR OPTIONS — D-121 THREE LENSES, PER OPTION

**(A) BUILD THE LIVE HANDLER, THEN VERIFY.** A `membership.*` branch on the **existing** `xgen-event` listener (`:551`), scoped to the current Space, plus `addMember`/`removeMember` on the store.
- **① User-visible:** live *who is here*. A join appears and a leave disappears without re-entering the room.
- **② Tier:** neutral. Reads an event already delivered; adds no persistence, no new copy, no erasure-fate change, no tier race.
- **③ Resource:** real frontend work — eight `Membership*` variants to triage, ordering, dedup, name resolution for a joiner absent from the book. Moves `svelte-check`. Its own commit. **A bridge on an existing transport, not a new transport.**

**(B) DESCOPE — R7 v1 IS A SNAPSHOT.** Accurate as of the last fill, refreshes on re-entry; steady state files as its own milestone.
- **① User-visible:** the panel **silently omits people who are in the room**, and keeps showing people who have left.
- **② Tier:** the departed-member row is the sharper edge — someone who left or was banned still renders as present. Not erasure in the D-093 sense, but the same shape at the UI: one party's state change invisible to another.
- **③ Resource:** free now — **except it is not.** §3's hard rule is *"staleness and absence BOTH render as UNKNOWN, never as fine."* Under B, state ② says *"self + all in room"* when it means *"as of when you opened it"*, with no staleness signal. **That is the exact lie this milestone exists to prevent.** Making B honest needs an as-of-open marker — **new appearance work in Joe's lane, costing more than A.**

**(C) VERIFY WHAT EXISTS, RECORD THE GAP AS UNMET.** Decides nothing; the milestone cannot close with a REQUIRED LEG unmet. **Listed for completeness only.**

**(D) EVENT-TRIGGERED RE-FILL.** On a scoped `membership.*` event, re-run the existing fill.
- **① User-visible:** **identical to A.**
- **② Tier:** the one place D is genuinely worse than A — every membership event re-runs `fill_space_records`, which **re-persists the address-book cache to disk**. More writes of other people's identity records, triggered by other people's actions.
- **③ Resource:** far cheaper — a few lines, no variant handling. But it **contradicts §5's lock**: it spends a drain plus a book fetch to learn what the event already said (a round-trip per join on a busy Space), and it makes the REQUIRED LEG's *"without a re-fill"* **unprovable by construction**.

**CHAT PROPOSES, DOES NOT DECIDE: (A).** It delivers §5's lock, it is a bridge rather than a transport, and it is the only option under which the REQUIRED LEG is provable. **(D) is the honest fallback if A proves too big** — same user outcome, and the price is amending §5's lock **openly** rather than shipping B and letting state ② quietly lie.

🔒 **RULING (Joe, 2026-07-26) — NONE OF THE FOUR. THE MECHANISM IS BUILT PROPERLY AND SHARED WITH THE ROOMS PANEL.** *"we have to build it properly and in this way that the mechanism will use also for rooms panel, where are all rooms of selected server and needs to be updated … if we will have such mechanism, it will have sense to use it intensively."* ⇒ **this is (A) with a second consumer, and it is recorded as its own decision — NOT as Joe picking (A)**, so no later reader collapses the two. ⚠️ **The widening was grounded before it was accepted, not after:** `spacesState.setSpaces` has **exactly one caller in the entire codebase** (`app_client.svelte:567`, the startup block), so the spaces/rooms tree is frozen **for the whole session** — worse than the roster, which at least reloads on a Space change. The `state.*` events that would refresh it already exist (`wire.rs:37-41`) and already arrive.

🔒 **CARRIED OUT OF THIS MILESTONE (Joe, 2026-07-26): its own milestone —** *"we can do it as new milestone if needed"*. Phase-0 written to `tasks/M_RP_LIVEFEED_REFRESH.md` v1.1 ACTIVE — **M-RP-LIVEFEED-REFRESH — the live event router behind the members and rooms panels** (name locked by Joe, §3 there). **M-RP-MEMBERS §5 is satisfied by that milestone's Leg A, and this milestone's Leg C unblocks behind it.**

✅ **THAT TRIGGER HAS FIRED (2026-08-01, J-643): `M-RP-LIVEFEED-REFRESH` LEG A IS BUILT AND VERIFIED** — the router lives on the existing `xgen-event` listener, `addMember`/`removeMember` are on the store, three files, `svelte-check` at the **0/34/15** floor. ⇒ **§5's steady state EXISTS in code for the first time, and §8a's blocker is discharged.** ⚠️ **LEG C IS UNBLOCKED BUT NOT DISCHARGED** — its REQUIRED LEG is *live membership **EXERCISED**, two clients and a real join*, and **Leg A shipped with NO CDP and NO live run by design** (that is `M-RP-LIVEFEED-REFRESH` Leg D). 🔑 **The thing Leg C must prove is still unproven; what changed is that it is now provable.**

📌 **CONSEQUENT SEQUENCING, ALSO JOE'S (proposal, not a decision):** under **A** or **D** the leg list needs a **build leg between B and C** — it moves `svelte-check` while Leg C moves nothing, and folding a build into a verify leg is the same attribution-mixing §8 splits A from B to prevent. **Superseded in practice by the ruling above** — the build leg now lives in `M_RP_LIVEFEED_REFRESH.md` §7 Leg A rather than in this document's §8.

---

## §8b — 🔑 THE META-FINDING: THREE GAPS, ONE SHAPE, AND THE AUTHORING HABIT BEHIND THEM

**This Phase-0 locked three behaviours that no leg was assigned to build:**
1. **§5 steady-state roster refresh** — §8a above.
2. **§6 unresolved rows DISTINGUISHABLE** — shipped code renders `rec?.display_name ?? tail(id)`, so `not_found` (**permanent**) and record-without-a-name collapse to the **same tail-8**; and the **transient** case cannot occur at all, because the fill awaits roster *and* book before a single `setResult`.
3. **The membership author-exclusion test** — §8's REQUIRED LEG demands it; no leg builds it, and it moves the **cargo** floor while the §5 handler moves **svelte-check**, so it cannot be folded into the build leg either.

⇒ **Every leg did what it was told. Leg C was told to verify things nobody was told to build.**

🔑 **THE HABIT — CHAT'S, NAMED SO IT STOPS:** *scope is written in FILES, requirements are written in BEHAVIOURS, and the two are never reconciled.* Three instances in three sessions — PANEL-INERT §1 omitted `app_sampler.svelte` while §4 required a DOM verification that needed it; Leg B §6 required *"all five states observed"* while §7 put observation in Leg C; and Phase-0 §8 assigned Leg C to verify §5, which no leg built.

🔒 **RULE, APPLIED FROM HERE:** any DoD item containing **"observed"**, **"exercised"**, **"driven"** or **"measured"** MUST name its **surface** in the scope section, or the scope section is wrong. And before a leg list is locked, walk **every 🔒 in the document** and ask **WHICH LEG BUILDS THIS.**

📌 **Clair flagged and proceeded correctly all three times.** She has been absorbing a defect that is Chat's to stop making.

---

## §9 — Filed, NOT fixed

- **E2 per-entry delete has no UI** (§4 collision). A locked erasure control with no surface.
- **The whole-book view has no home** if §4 locks B.
- **Contact alias (③) and presence (④) remain unbuilt**, and Ch2 specifies **no contact-acquisition flow at all** (grepped `docs/`; zero) — so the top of the G6 name chain cannot be built even if wanted.
- **Per-tier retention N is not derivable** (J-587): `tier` is reachable only through `trust_assertion`, which the wire never carries. Every record evicts on the T1 default. M13's problem, recorded here because a members widget is where a tier would first be visible.
- 🛑 **§6's THIRD UNRESOLVED-ROW CASE — *book never consulted* — NOW EXISTS, AND IT IS A RENDERER QUESTION, NOT A STORE ONE** (`M-RP-LIVEFEED-REFRESH` Leg A `Owes:`, discharged into this file at J-643). §6 enumerates three truths, all of which assume a member arrived **through a fill**. Leg A created a fourth situation: a member added by a **live `membership.join`**, for whom the address book was **never consulted at all**. ✅ **The store side is done** — `MemberEntry.unresolved` marks it and `toDescriptor` omits the `isAi` claim rather than defaulting it `false`. 🛑 **The RENDER side is not, and cannot be from inside that milestone:** `entity-avatar.svelte:125` is `data-ai={flags.isAi || undefined}`, so **`false` and absent produce identical DOM** — an AI joining live still renders as a human. ⚠️ **§6's distinguishability rule now has THREE cases to separate, not two, and the renderer currently separates none of them.** 🔓 **Joe's, and it is the same question as §6a's** — both are *what does an unresolved row look like*, which is why they are filed together.
- 📌 **NOT RE-VERIFIED BY THE SECOND READER, FLAGGED RATHER THAN ASSUMED (Clair):** §9's *"contact alias ③ = 0 code hits"* and *"no contact-acquisition flow (grepped `docs/`; zero)"* were **outside the five named targets and were not re-run**. They stand on Chat's J-578 measurement alone. Re-measure before either is used to justify a decision.

### ⚠️ SELF IS IN THE ADDRESS BOOK, AND THE LOCKED DESIGN SAYS IT IS NOT — FOUND BY JOE, 2026-07-25

**A locked statement and the shipped code disagree. Measured, both sides:**

| Source | Says |
|---|---|
| `tasks/M_RP_ADDRESS_BOOK.md:125` (§6 erasure, 🔒 **LOCKED**) | *"A local cache of **other people's** identity data."* |
| `tasks/M_RP_ADDRESS_BOOK.md:153` (§7 storage, 🔒 **LOCKED**) | *"keeps **other people's** identity PII out of the client's own state"* |
| `ops.rs:2700-2723` (`observed_identities`, **shipped**) | F1 = every message author · F2 = **every projected member**. **No self-filter.** |
| `address_book.rs` (**shipped**) | **zero** hits for `self_` / `is_self` / `whoami` / `exclude` |

⇒ **you are in your own address book, and two locked sentences assert you are not.** Neither the Leg-D runbook, nor the live pass, nor Clair's second-reader pass, nor Chat caught it. **Joe did, with a four-word question about Chat's terminology.**

**Three consequences, ascending:**
1. **§7's storage rationale no longer describes the file.** Your own PII is in it. Harmless — your data, your disk — but the sentence is false.
2. **§6's erasure reasoning was built on a third-party frame.** E1/E2/E3 still work; erasing your own record is meaningless rather than wrong. E3 never fires on self, since `touch` advances `last_seen` on every fill and you are always a member. Low severity.
3. ⚠️ **THE REAL ONE — A SECOND HOME FOR YOUR OWN DISPLAY NAME.** `get_self_state` is authoritative; the book holds an **observation** of you. **One fact, two sources, and one of them is stale by construction — D-067.** Bound for this milestone by the §4b lock (R7 reads `selfState`), but the rule is a **convention, not a mechanism**: the next consumer that forgets it reintroduces the divergence.

**🔒 LOCKED — S2, EXCLUDE SELF AT FILL (Joe, 2026-07-25).** ⚠️ **PROVENANCE: CONSIDERED, not delegated** — Joe chose the option Chat did **not** recommend, and named its consequence himself in the same breath. ⇒ **the drift becomes structurally impossible rather than a rule people must follow**, which is the stronger guarantee Chat had identified while still recommending the cheaper one.

🔑 **JOE NAMED THE COST S2 CARRIES, AND IT IS A DESIGN CONSEQUENCE, NOT A SIDE EFFECT:** *"that means we need the setting interface to have somewhere else, maybe in settings > account sub menu (in the single place)."* ⇒ with self excluded from the book, **the book can no longer be the surface through which you edit your own record** — there is no record. The self-editing surface moves to **Settings › Account**, and *"in the single place"* is the binding half: **ONE pane, however many entry points** (the J-503 one-pane-two-entry-points precedent).

🔒 **AND THE MENU SHAPE — CORRECTED 2026-07-25 (Joe):** *"i meant: View > Address Book, Self Card is analogous to File > Setting, Exit now."* ⇒ **THE VIEW MENU EXISTS**, holding **Address Book** and **Self Card** — structurally analogous to how File holds Settings and Exit. **Two distinct items, not a submenu.** Address Book opens its own modal; **Self Card opens the self widget** (§4c — it carries more logic than a card).

⚠️ **CHAT READ THIS EXACTLY BACKWARDS AND THE ERROR IS KEPT.** Joe's *"like Settings and Exit"* was an analogy about **structure** — a menu holding two items — and Chat read it as an analogy about **location**, concluding *"the View-menu proposal is dropped, they go in File"*. 🔑 **The reading was filed as a reading and not a lock, which is the only reason it cost one sentence instead of a milestone.** ⇒ *the discipline earned its keep twice in one exchange — here, and at the self-row/Self-Card confusion above.*

🔓 **STILL OPEN:** whether the self widget behind Self Card **is** Settings › Account or merely reaches it. Joe said *"maybe"*. ⚠️ **A "maybe" is not a design and must not harden by repetition.**

📌 **Gates nothing.** Leg A is untouched; Leg B needs only the §4b lock, which already answers the question for R7. **S2 is an amendment to a CLOSED milestone and rides whichever milestone next touches the fill path.**

---

**The options as they stood, kept because the reasoning is load-bearing:**

**🔓 OPEN — JOE'S, because it amends a CLOSED milestone (architecture).**
- **S1 — leave the code; correct the two prose lines.** ① *User-visible:* nothing, **provided** every consumer resolves self from `selfState`. ② *Cost:* **zero code**, two sentence fixes in a COMPLETED doc.
- **S2 — exclude self at fill.** ① *User-visible:* nothing today, and the drift becomes **structurally impossible**. ⚠️ But *"absent from the book"* then becomes a **legitimate** outcome meaning *"it's me"*, which every future name-resolver must branch on — trading one rule for another. ② *Cost:* one filter, **plus** a migration question (existing book files already contain self), **plus** reopening a closed milestone's code.
- **S3 — keep self, mark it with a field.** ① Nothing user-visible. ② Highest — a format change to a shipped file.

**Chat recommends S1 plus the §4b lock**, and says plainly why it is the weaker guarantee: it makes the correct behaviour a **rule people must follow** rather than a shape that cannot be got wrong. S2 is the stronger guarantee and Chat does not recommend it, because *"not in the book"* silently acquiring a second meaning is the kind of overloaded absence this project has been bitten by before. **Recorded as a collision, not traded (D-121).**

📌 **Gates nothing.** Leg A is untouched; Leg B needs only the §4b lock, which already answers the question for R7.

---

## §10 — DoD

- [x] §4, §6, §7 locked by Joe, in place in this document with the lock date — **2026-07-25, all three DELEGATED**
- [x] Second reader (Clair, not Chat) has read this Phase-0 against Ch2 — **done 2026-07-25; 5 targets, 4 CONFIRMED, 1 overstatement (3b), 1 GAP (3c → §4b); every finding re-measured by Chat before entering this document**
- [ ] 📌 **§4b (DM Space) — NO LONGER A GATE.** Closed by construction at §4c; any DM-specific display change is additive, not a prerequisite
- [ ] Leg A: cargo floor re-measured, delta explained
- [ ] Leg B: svelte-check floor re-measured, delta explained
- [ ] Leg C: every claim CDP-measured by Chat, on the real client 9222; the off-critical-path lock **exercised**
- [ ] 🔒 **Live membership EXERCISED** — two clients, one Space, a real join: the connected observer's roster updates with no re-fill, **and** the joiner does not receive its own event (§5 author exclusion)
- [ ] 🔒 **The fill is BOUNDED** — §4c-i: without it, ③ is not sayable and the `FillLock` can deadlock the feature
- [ ] `xgen-client_address_book.json` **exists after a normal session** — the milestone's one-line proof
- [ ] `flags.revoked` verifiably UNFED
- [ ] Records: JOURNAL + PLAY + ROADMAP + this doc in one commit

---

## §11 — Handoff

🔒 **ALL THREE DECISIONS LOCKED 2026-07-25 — §4 B · §6 tail-8 · §7 unfed — ALL THREE DELEGATED.** Joe adopted Chat's recommendations wholesale (*"lock all by your recomms"*) rather than walking them. Under D-123 these three are **his** areas, so the delegation is recorded as provenance on each one: they are decisions of record, but not decisions he examined, and the re-open cost is low on all three (a latch, a label format, a prop).

✅ **SECOND-READER PASS DONE 2026-07-25 (Clair) — the Leg 0 DoD item is ticked.** Five targets: **four CONFIRMED** (G6 vs D-124 · §4a both halves · §4-B vs Ch2 L1859–1863 · `flags.revoked` unfed · the 16-command count), **one overstatement** (§4-A's threat model, 3b), **one GAP** (the DM Space, 3c → the new §4b). ⚠️ **Every finding was re-measured by Chat before it entered this document** — including one of Chat's own probes that was too narrow to see its subject (§3 precision note) and one correction Clair made to Chat's own briefing about the git state, which was right.

🔓 **ONE OPEN ITEM: §4b, the DM Space. Joe's. IT GATES LEG B ONLY.**

**Leg A may open now** — two Tauri commands that do not care whether a Space is a DM. Leg A moves the cargo floor, Leg B moves svelte-check; they are separate runbooks for that reason, and one commit spanning both would make a regression ambiguous.
