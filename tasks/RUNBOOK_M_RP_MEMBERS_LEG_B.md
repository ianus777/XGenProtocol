# RUNBOOK — M-RP-MEMBERS LEG B: the address-book store and the R7 members widget
> **Status**: ACTIVE  
> Version: 1.5  
> Date: Jul 2026  
> **Last updated**: 2026-07-26  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — STATE AT OPEN, AND ONE SEQUENCING CORRECTION

**Phase-0:** `tasks/M_RP_MEMBERS.md` v1.11 ACTIVE. Legs A ✅ · A-bis ✅ (J-590) · A-ter ✅ (J-591).
**HEAD at authoring:** `2814fec`, tree clean.

**FLOORS — BOTH RE-MEASURED 2026-07-26, NOT INHERITED:**

| Floor | Value | |
|---|---|---|
| `svelte-check` (`cd ui; npm run check`) | **0 err / 34 warn / 15 files** | ✅ re-measured, identical to last known |
| client registry (`__XGEN_DEBUG__.ids().length`) | **149** | ⚠️ total matches; **composition NOT established** — see §0b |
| cargo | 1588 / 0 / 62 across 56 | not re-measured; Leg B touches no Rust |

### ✅ §0a — LEG A-quater — CLOSED (J-595). SHIPPED AS ITS OWN COMMIT BEFORE LEG B.

**Done (J-595):** `FillMembersOutcome { fill, roster }` shipped — 2 files, +25/−7, `xgen-client` only (`ops.rs` + `desktop.rs`). Both the public `fill_and_members` and the private `fill_and_members_inner` return `Result<FillMembersOutcome>`; `desktop.rs:672` returns `Result<crate::ops::FillMembersOutcome, String>`; the call site at `:696` flows through unchanged (no tuple destructuring). No `#[serde(rename_all)]` (grepped) ⇒ wire keys are `fill` / `roster`, inner fields snake_case, `FillReport`/`MembersResult` unchanged. **cargo HELD 1588 / 0 / 62 across 56** (a rename adds no coverage — the kickoff's "it moves" prediction was wrong, Rule 6); clippy 0 new lints; svelte-check 0/34/15 + registry 149 held by scope. **Leg B rebases on this.**

Joe ruled the return shape: **positional tuple → named struct.** That is a **Rust** change and it **moves the cargo floor**. Phase-0 §8 splits A and B precisely so a regression is attributable: *"A moves the Rust floor, B moves svelte-check. One commit spanning both makes a regression ambiguous."*

⇒ **`FillMembersOutcome` ships as its own leg, its own commit, before Leg B opens.**

**LEG A-quater — the return shape is named before anything binds to it.** Rust only:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct FillMembersOutcome {
    pub fill:   FillReport,
    pub roster: MembersResult,
}
```

- `ops.rs:2951` — `fill_and_members(...) -> Result<FillMembersOutcome>`
- `desktop.rs:672` — `fill_space_records(...) -> Result<FillMembersOutcome, String>`
- `desktop.rs:696` — the single call site, updated
- 🔓 **Names DELEGATED, not locked** (Joe: *"i go with your proposals"*, and *"it is a technicality which is yours"*). `FillReport` and `MembersResult` unchanged.
- ⚠️ **No `#[serde(rename_all)]` on either inner struct** (verified) ⇒ wire keys are exactly `fill` and `roster`, and the inner fields stay snake_case.
- **Zero frontend callers exist**, and no test calls `fill_and_members` directly (only `desktop.rs:696`). ⇒ this is the cheapest it will ever be.

---

### 🔒 §0c — M-RP-PANEL-INERT SHIPS FIRST (added v1.5, 2026-07-26)

⚠️ **§G2 OF THE PHASE-0 WAS WRONG AND LEG B CANNOT PROCEED ON IT.** *"The pattern is literal"* was read as **`entity-panel` is a neutral list renderer**. It is an **interactive single-select listbox**: `<ul role="listbox">` (`:138`) of `<li role="option">` (`:146`), `onclick`/`onkeydown` (`:150-151`), and `selectAt` writing `selected` **unconditionally** at `:91` before `onActivate`. `cursor: pointer` and a focus ring in `skin.css:2604-2612`. **No `readonly`/`inert` prop exists.**

🔑 **R1/R2 ARE CORRECT ONLY BECAUSE THEY CLOSE A LOOP R7 CANNOT** — `onActivate → selection.set()` feeds the bus, which flows back into `selected`. **L15 forbids R7 writing that bus**, so a click's write at `:91` is never corrected: **the wrong highlight sticks**, and in a group room **one click manufactures one**.

⇒ 🔒 **`tasks/M_RP_PANEL_INERT.md` — `entity-panel` gains a non-interactive mode — SHIPS BEFORE LEG B, AS ITS OWN COMMIT.** Leg B then passes `interactive={false}` and touches `entity-panel` not at all.

📌 **Chat recommended the opposite first** — reimplementing the list inside R7 (~70 lines) to avoid a core change — **having read §1's "do not touch entity-panel" as a COST when it is a SCOPE boundary.** The real change is ~10 lines. *The avoidance was seven times the size of the thing avoided.* Joe caught it by asking whether the decision was not already made.

⚠️ **STILL NOT FIXED BY IT:** `.entity-item:hover` (`skin.css:2521`) is entity-item's own skin and survives. **L7's "no hover" is a skin carve-out for Joe regardless** — after PANEL-INERT it is the only one of six left.

---

### ⚠️ §0b — A NAMED GAP LEG B MUST NOT PAPER OVER

The registry reads **149**, matching the recorded floor exactly — but **the composition is not established.** J-563's model was *149 = empty store + the "No spaces yet" placeholder*, and *158 = 149 + 10 entity rows − 1 placeholder*. The live client right now shows **1 space, 1 room, and no placeholder**, which under that model predicts ~150.

⇒ **Leg B's verification must pin the composition, not just the total.** Record the baseline as *"149 with a store containing N spaces / M rooms"*, never as a bare number. J-563's own warning: *or lose a session hunting a +9 that was never a defect.*

📌 **Also inherited and true:** `__XGEN_DEBUG__` is an **API object** with `ids` / `get` / `snapshot`. `Object.keys(__XGEN_DEBUG__).length` is **3 forever** and is not the registry count. The count is `__XGEN_DEBUG__.ids().length`. (N-166 family; cost one near-miss on 2026-07-26.)

---

## §1 — SCOPE

**TOUCHED (frontend only — moves the `svelte-check` floor and nothing else):**

| File | Action |
|---|---|
| `ui/common/lib/stores/address-book.svelte.ts` | **NEW** — the store. No such file exists today (verified) |
| `ui/common/lib/components/widgets/members-panel.svelte` | **NEW** — the widget |
| `ui/common/lib/plugins/registry.ts` | the **7th** `CLIENT_PLUGINS` descriptor (`surface: 'region'`, `regionId: 'members'`) — insert at `:107` list |
| `ui/client/src/app_client.svelte` | shell hydration, mirroring the R5/R6 pattern |
| `ui/client/src/layout-default.ts` | region slot, if the layout requires an entry |

**NOT TOUCHED — AND EACH FOR A STATED REASON:**

- ❌ `ui/common/lib/components/widgets/rooms-panel.svelte` — **§4a: R7 scopes off `roomLatch.effectiveSpaceId`. No new latch.** Copying R2's bare `let latchedSpaceId` would create a **third** latch, the exact D-067 drift surface `room-latch.svelte.ts` was lifted to prevent.
- ❌ any `.rs` — Leg B moves `svelte-check`, not cargo. If a Rust change looks necessary, **STOP and hand back**; it belongs to a Leg A leg.
- ❌ `ui/assets/skin.css` — **Joe's file** (D-123). Never folded into a Chat/Clair commit.
- ❌ `entity-item.svelte` / `entity-avatar.svelte` — consumed as-is.
- ❌ `entity-panel.svelte` — consumed **as it will be after `M-RP-PANEL-INERT`**, i.e. `<EntityPanel … interactive={false} />`. ⚠️ **DO NOT EDIT IT HERE.** See §0c.

---

## §2 — THE LOCKS THIS LEG BINDS TO

Reproduced so Clair implements from the runbook without re-interpreting the Phase-0. **Where these disagree with the Phase-0, the Phase-0 wins and this runbook is wrong.**

🔒 **L1 — SCOPE IS `roomLatch.effectiveSpaceId`** (`$common`, `room-latch.svelte.ts`), already consumed by R5 (`stream-panel:69-70`), R6 (`composer-panel:68-69`) and the shell (`app_client:153`). **Honest cost, stated not hidden:** it is `null` until a **room** is latched, so selecting a Space in the tree does **not** populate R7. That is B1, chosen over B2 deliberately.

🔒 **L2 — SELF IS A FIXTURE, NOT A MEMBER.** Always present · always first · **resolved from `selfState`, NEVER from the book** · included when offline. ⚠️ **AMENDED:** "filter-immune" is **fixture of the RESTING state** — an active search shows **results only, without self** (§4c 2b).

🔒 **L3 — THE ROSTER CROSSES `Option`-SHAPED, NEVER A BARE ARRAY.** An empty array **must not be conflatable** with *not fetched*. Precedent already shipping: `SeenRecord::trust_lapsed(now) -> Option<bool>`, where `None` means *no opinion*. In TS: `roster: MemberEntry[] | null`, `null` = **not known**.

🔒 **L4 — ANY MEMBER COUNT DERIVES FROM THE ROSTER, NEVER FROM RENDERED ROWS.** The self fixture is not a roster entry and must not be counted as one.

🔒 **L5 — THE PREDICATE IS ROSTER-UNKNOWN, NOT "OFFLINE".** A drain that fails **while online** leaves the panel in the same position and would otherwise render *"only self"* with no explanation. Three-valued on the roster: KNOWN → render · UNKNOWN (never fetched · failed · offline) → **say so** · known-and-empty → **unreachable by construction** (if you are scoped to it, you are in it).

🔒 **L6 — THE FILL IS THE COLD START ONLY.** Steady state is the live membership channel (§5). **POLLING IS REFUSED.**

🔒 **L7 — ROWS SHIP INERT.** No click, no hover, no cursor affordance. ⚠️ **On SCOPE grounds, not absence of capability** — `ops::self_open` and `ops::create_dm_space` **both exist and are tested**; only their Tauri commands are missing. Record it that way so the next reader does not "discover" a missing feature.

🔒 **L8 — WIRE FIELDS ARE snake_case:** `space_id` · `identity_id` · `joined_at` · `invited_by` · `not_found` · `events_replayed`.

🔒 **L9 — `flags.revoked` SHIPS UNFED.** `entity-avatar` draws a revoked badge from `EntityFlags.revoked`, and the book's `revoked` is `false` on **every wire-filled record** because the wire never sets it. Feeding it lights a **shipped affordance from a constant false** — the N-097 shape that stranded `entity-item.selected` at M-RP6.1g. ⇒ **v1 feeds `flags.isAi` and nothing else.**

🔒 **L10 — `secondary` / `meta` / `status` SHIP UNFED.** v1 is shape-identical to R1/R2. ⚠️ `status` is the Track A **self-status** slot; **presence is layer ④ and unbuilt — nothing here may put a dot beside a name.** `role` and `joined_at` arrive free and are **deliberately discarded**; that is a choice, not an oversight.

🔒 **L11 — UNRESOLVED ROWS = tail-8** (D-126); word form **deferred**. ⚠️ Cases 2 (not yet fetched, transient) and 3 (`not_found`, permanent for now) **must be distinguishable** — *that* they be distinguishable is the honesty rule and is not negotiable; *how* is appearance and Joe's.

📌 **L12 — R3's avatar is SCAFFOLDING (Joe). Design nothing against it.**

🔒 **L13 — IN A DM, ALL MEMBERS RENDER AND THE COUNTERPART IS HIGHLIGHTED (Joe, 2026-07-26):** *"all members, not reduced list, just highlighted row of member with whom user talks to"* · *"while dm, all members are present but only bob is highlighted"*. ⇒ **no reduction, no branch, no vanishing panel.** The panel keeps ONE rule — *R7 shows who is in this conversation* — and adds information rather than removing it.

🔒 **L14 — THE MECHANISM IS `entity-item.selected`, AND CHAT'S OBJECTION TO IT WAS WRONG.** Chat argued `selected` would be semantically overloaded because the user does not *pick* the counterpart. **Joe: *"when i go to dm someone, i pick him. otherwise the dm thread will not display itself."*** ⇒ opening a DM **is** the selection; the word is being used for exactly what it means. 📌 **And the plumbing already exists** — `entity-panel.svelte:129/148/159` already does `selected={item.descriptor.id === selected}` and passes the per-row boolean into `entity-item`. Feeding it is ONE PROP, not new machinery. ⚠️ *Chat's earlier "`selected` is stranded and unplumbed" was wrong on the second half: the plumbing is complete; only a caller was missing.*

⚠️ **L15 — THE TRAP BESIDE L14. "SELECTED" MEANS TWO DIFFERENT THINGS AND R7 MUST USE THE LOCAL ONE.**

| | What it is | Scope |
|---|---|---|
| `entity-panel.selected` | a **local prop** — which row in THIS panel is highlighted | one panel |
| `selection.current` (`$common/stores/selection.svelte.ts`) | the **global selection bus**, `{ regionId, entity } \| null` — *"ONE active selection across the whole layout"* | whole app |

R1/R2/R3 **write** the bus on activation; **R8 (inspector) and `entity-context-menu` READ it.** ⇒ 🔒 **R7 passes `selected={counterpartId}` as a LOCAL PROP and MUST NOT call `selection.set()`.** If R7 wrote the bus, opening a DM would silently change what R8 displays — a local highlight hijacking an app-wide primitive.

🔒 **L16 — THE HIGHLIGHT IS DM-ONLY.** Fed when `roster.is_dm` is true, with the counterpart's `identity_id`; **`null` in a group room** (Joe: *"when a room thread is displayed, nobody needs to be selected"*). `MembersResult` already carries `is_dm` (`ops.rs:2606`) — no new plumbing.

🔒 **L17 — SELF SHIPS UNMARKED (Joe, 2026-07-26).** Self is present and **first**, and that position is the only thing distinguishing it. **No "Self" label, no second emphasis.** Three reasons: ① *always first* is already a mark · ② **M-RP-OWN-ROW-NAME** owns the *"Self"* label and its styling and is explicitly a **stream** milestone — minting a second self-marking mechanism here is the **D-067** surface · ③ R7 now has a real emphasis mechanism (L14) and a short list must not carry **two visual languages**. ⇒ in a DM the marked row is Bob, so the other is you **by elimination**. 📌 **Self-marking rides whenever OWN-ROW-NAME's styling lands, so R7 CONSUMES it rather than duplicating it.** ⚠️ **PROVENANCE: DELEGATED** — appearance, and it reads differently on screen than in a runbook. **Re-open freely on sight; the change is additive.**

⚠️ **L18 — CONSEQUENCE FOR §7 OF THE PHASE-0, FLAGGED NOT SILENTLY ABSORBED.** §7 locked *"v1 is shape-identical to R1/R2"*. A highlighted row is a small, deliberate **departure** from that. It is additive and it retires a stranded affordance, so it should be an **amendment to §7**, not a conflict resolved in a runbook. 🔓 **Amending §7 is Joe's** — it is an appearance lock in a document he owns.

---

## §3 — THE STORE (`address-book.svelte.ts`)

Mirrors the shape of the existing `$common` stores (`self-state.svelte.ts`, `spaces-state.svelte.ts`).

**State, minimal and Option-honest:**

```
spaceId: string | null        // the scope this state describes
roster:  MemberEntry[] | null // L3: null = NOT KNOWN. [] is unreachable (L5)
book:    Record<identity_id, SeenRecord>
phase:   'idle' | 'inflight' | 'ready' | 'failed'
```

⚠️ **`phase` is NOT knowledge — it is context and elapsed time** (§4c-i). Knowledge is binary and lives in `roster`. `phase` exists only to separate ③ from ④. **Do not derive "roster known" from `phase`; derive it from `roster !== null`.**

**Behaviour:**
1. On `effectiveSpaceId` change → reset (`roster = null`, `phase = 'inflight'`), then invoke `fill_space_records`.
2. On resolve → `book` merged, `roster = outcome.roster.members`, `phase = 'ready'`.
3. On reject → `roster` stays `null`, `phase = 'failed'`. ⚠️ **③ → ④ is a transition, not a separate render** (Joe).
4. On `effectiveSpaceId === null` → reset to `idle`, `roster = null`. **No invoke.**
5. **Late-response guard:** a resolve whose `spaceId` no longer matches the current scope is **discarded**. Without it, switching rooms twice quickly renders Space A's roster under Space B's heading — the §4a divergence through a different door.

⚠️ **NO TIMER.** The Rust `tokio::time::timeout` bounds the fill, releases the `FillLock`, and makes the invoke **always resolve or reject**. The frontend escalates ③ → ④ **on the rejection**. A frontend timer is optional belt-and-braces and is **NOT in scope** (§4c-i, Chat's over-recommendation corrected there).

⚠️ **NO POLLING (L6).** If the roster looks stale, that is the live-channel leg's problem, not a retry loop's.

---

## §4 — THE WIDGET (`members-panel.svelte`) — FIVE STATES, ONE TREE

**It is a tree, not a 2×2.** Copy verbatim; the messages are **Joe's copy, recorded as given, not approximations.**

```
is there a scope? (roomLatch.effectiveSpaceId)
│
├─ NO ───────────────────► ①  self only, NO message      ⚠️ ABSORBS OFFLINE
│
└─ YES → is the roster known? (roster !== null)
         ├─ KNOWN ───────► ②  self + all in room
         └─ UNKNOWN:
              ├─ inflight ► ③  self + "I am waiting for the others"
              ├─ failed ──► ④  self + "I cannot reach the others"
              └─ offline ─► ⑤  self + "I cannot see the others"
```

🔒 **① SHOWS NO MESSAGE.** With no room picked there are no "others" being blocked; *"I cannot reach the others"* would blame the network for a scope the user never chose — **a second false statement.**

⚠️ **REJECTED WORDING, DO NOT RE-PROPOSE:** *"I cannot see the others **online**"* reads two ways, the second being *"the others who ARE online"* — **a presence claim.** Presence is layer ④, unbuilt. ⇒ ④ is *"I cannot reach the others"* (Joe, final).

📌 **④ vs ⑤ is nuance, not correctness** — both wordings are true in both states, distinguished for the user by the **connection led**, not the verb. They may later collapse into one at no cost to truth. Recorded so a future reader knows it was a choice.

**R7 HAS NO EMPTY STATE** (§4c Consequence 1) — the panel always holds at least the self row. §4a-B1's *"Select a room"* empty state is **superseded**.

---

## §5 — REGISTRATION AND HYDRATION

- `registry.ts:107` — 7th descriptor, `surface: 'region'`, `regionId: 'members'`. Follow the `spaces` / `rooms` / `stream` / `composer` descriptors exactly.
- `app_client.svelte` — hydrate as R5/R6 do (`app_client:153` reads `roomLatch` already).
- `region-node.svelte:45` resolves the display title from `CLIENT_PLUGINS` + `REGION_NAMES`. 🔓 **The region's display title is APPEARANCE and is JOE'S.** Do not invent one; if absent, **stop and ask.**

---

## §6 — DEFINITION OF DONE

- [x] Leg A-quater shipped and committed **separately** (§0a) — **CLOSED, J-595, commit `a0752e5`**
- [ ] Leg B rebased on it and binding to `.fill` / `.roster`, not to a tuple
📌 *Split from one box into two on 2026-07-26: the original conflated a condition already true with one that cannot be true until Leg B ships — an unflippable box, which is exactly what the task-file DoD rule exists to prevent. Chat's wording, so Chat amended it.*
- [ ] `svelte-check` run and reported as **err / warn / files**, compared against **0 / 34 / 15**
- [ ] Registry delta recorded **with store composition**, never as a bare total (§0b)
- [ ] All five panel states reachable and each one **observed**, not reasoned about
- [ ] Self row present in **all five**, always first, resolved from `selfState`
- [ ] Roster crosses `Option`-shaped; a `null` roster and an empty roster are **not** the same render
- [ ] Member count derived from the roster, not from rendered rows
- [ ] Late-response guard exercised: switch rooms twice quickly, assert no cross-render
- [ ] `flags.revoked`, `secondary`, `meta`, `status` all confirmed **unfed**
- [ ] Rows inert: no click, no hover, no cursor change
- [ ] No `.rs` file touched; no `skin.css` touched; `rooms-panel.svelte` untouched

⚠️ **DoD does NOT include "commit pushed"** — `Status: COMPLETED` in the header is the shipped signal.

---

## §7 — HANDOFF

**Clair implements from this runbook once Joe locks it.** Leg C (live CDP verify, 9222) and the **required** live-membership leg — two clients, one Space, a real join, asserting the joiner does **not** receive its own `membership.join` — follow Leg B and are **not** in scope here.

✅ **RESOLVED 2026-07-26 — THE TWO THAT BLOCKED CLAIR:**

1. ✅ **§4b, the DM Space.** Joe: *"all members, not reduced list, just highlighted row of member with whom user talks to"*. ⇒ **L13–L16.** Better than all three options Chat offered: it keeps one rule and adds information instead of removing it. **No branch, no vanishing panel, no rule-with-an-exception.**
2. ✅ **Marking your own row.** Joe: *"while dm, all members are present but only bob is highlighted"*. ⇒ **L17, self unmarked.** DELEGATED, re-open on sight.

✅ **ALSO RESOLVED 2026-07-26:**

3. ✅ **The R7 plugin descriptor `name` = `Members`** (Joe, 2026-07-26: *"panel is redundant imho"*). ⚠️ **AND CHAT'S "HOUSE PATTERN" WAS WRONG — IT QUOTED THE TWO EXCEPTIONS.** Measured across `registry.ts`: **`Spaces` · `Rooms` · `Messages` · `Composer` · `Plugin List` · `Grid Backdrop` · `Text Processing` · `Connection Stats` are ALL BARE**; only `Self Panel` (`:110`) and `Inspector Panel` (`:123`) carry the suffix. **8 bare to 2.** Chat sampled the 2 and called it the convention. 📌 Joe: *"inspector and self panel will be changed in this spirit soon"* — the two outliers are the thing to fix, not the rule to follow. 📌 **No prefix mismatch either:** `rooms` is itself a plugin named `Rooms`, so its tile already reads *Rooms*, not *R2 · Rooms* — `Members` sits exactly alongside its siblings. 📌 `layout-default.ts:29` keeps `'R7 · Members'` as the `REGION_NAMES` fallback; `buildTitles` prefers the plugin `name`, so the descriptor value is what renders on the tile.
4. ✅ **Leg A-quater ships NOW, as its own commit** (Joe, *"yes we will commit now"*) — **before** Leg B, so the cargo floor and the svelte-check floor move in separate commits and a regression stays attributable.

🔓 **NOTHING BLOCKS CLAIR.** One amendment remains outstanding but does not gate implementation: ⚠️ **Narrower than Chat first stated:** `layout-default.ts:29` already carries `members: 'R7 · Members'` as the `REGION_NAMES` fallback, and `members` is already a leaf in the default layout. But `buildTitles` prefers a **region plugin's own `name`**, so the descriptor needs one. House pattern: `self → "Self Panel"`, `inspector → "Inspector Panel"`. **One word, not a blank page.**
🔓 **AND ONE AMENDMENT THAT IS JOE'S BECAUSE IT TOUCHES A LOCK HE OWNS:** §7's *"v1 is shape-identical to R1/R2"* needs amending for the highlighted row (**L18**).

📌 **RECORDS DEFECT FOUND WHILE COMPILING THIS LIST, REPORTED NOT SILENTLY FIXED:** Phase-0 §11 (line 583) and §4b (line 154) **disagree** about how open §4b is. §4b was narrowed by the v1.5 self-fixture lock; §11's handoff line was never updated to match. **Amending §11 is Joe's** — it is the Phase-0's own handoff. Left as found.

📌 **Carried, unchanged, not re-asked:** the config milestone's name · Self Card vs Settings › Account · the partial-first-send remainder · the search-matched-nothing copy (the sixth panel state).
