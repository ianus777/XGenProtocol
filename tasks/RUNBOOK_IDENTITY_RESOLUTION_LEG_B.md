# M-RP-IDENTITY-RESOLUTION Leg B — the render rules
> **Status**: PENDING  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

🛑 **AUTHORED, NOT LOCKED. NOBODY IS IMPLEMENTING THIS FILE.** It becomes Clair's instruction only when Joe says *"lock"* AND stands her up — **two acts** (the J-646 rule: *"locked" reads like "started" to anyone skimming*). Until both, this is a proposal.

📌 **Parent Phase-0:** `tasks/M_RP_IDENTITY_RESOLUTION.md` **v1.7**. 🛑 **RUNBOOK-AS-GROUND-TRUTH IS A FAILURE MODE.**

**SESSION-OPEN READING ORDER (Clair):** ① `CLAUDE.md` PLAY head → ② latest `JOURNAL.md` entry → ③ Phase-0 §3 (the four states) · §4 · §5 · §5a · §5b · §8 B-1…B-4 → ④ **this file**. It is item 4, not item 1.

---

## §1 — What this leg does, in one sentence

**The `not_found` ids that Leg A pushed into the webview currently stop at `app_client.svelte:183`; Leg B carries them into the store, filters state ③ out of the rendered list with §5a's DM exception, and threads a per-row hook down to `entity-item` so the skin can tell ④ from ③.**

🛑 **THIS LEG LANDS SILENTLY.** ④'s dimming and E2's mark are **`skin.css` values, and `skin.css` is Joe's** (Leg C, deliberately separate — Joe, 2026-08-01). **After Leg B, nothing on screen changes except that erased non-DM members disappear from the list.** Written here so the close is not read as a visual failure.

---

## §2 — Files, and the TWO commits

🔒 **THIS LEG SHIPS AS TWO COMMITS, AND THE SPLIT IS NOT COSMETIC.** Phase-0 §8 splits legs **by floor** — *"A and D move cargo, B moves `svelte-check`, C moves neither; one commit spanning them makes a regression unattributable."* But Phase-0 §9 **owes Leg B a `Vec<IdentityXgid>` wire witness**, which is a **cargo test**. ⚠️ **Phase-0 therefore contradicts itself, and this runbook resolves it by splitting the leg rather than by breaking either rule** (Chat, mechanical, D-123; surfaced rather than absorbed, D-065).

| commit | file | floor it moves |
|---|---|---|
| **B-i** | `xgen-client/src/ops.rs` (tests only) | **cargo 1588 → 1589** · `svelte-check` untouched |
| **B-ii** | `ui/common/lib/stores/address-book.svelte.ts` | `svelte-check` only |
| **B-ii** | `ui/client/src/app_client.svelte` | `svelte-check` only |
| **B-ii** | `ui/core/lib/components/data-dependent/entity-item.svelte` | `svelte-check` only |
| **B-ii** | `ui/core/lib/components/data-dependent/entity-panel.svelte` | `svelte-check` only |
| **B-ii** | `ui/common/lib/components/widgets/members-panel.svelte` | `svelte-check` only |

⚠️ **B-i FIRST.** It is independent of every frontend change and it is the obligation with the older date. **Do not batch them.**

🛑 **OUT OF SCOPE, NAMED SO IT IS NOT DRIFTED INTO:** `ui/assets/skin.css` (Joe's, Leg C) · any Rust outside the `ops.rs` test module · `MemberEntry`'s mirror fields · the refresh trigger (G-B, Leg E) · the Tier-1 fetch (§7, Leg D) · `M-RP-XGID-SLOT-RETYPE` (`D-136`, worked around not fixed).

---

## §3 — ✅ Grounding (measured 2026-08-01 at `9cce4a9`, tree clean)

Every line number below was read from the file, not recalled.

- **G-B1 — `FillReport` has exactly five fields and derives `Default`.** `ops.rs:2772-2794`: `candidates` · `fetched` · `not_found` · `not_found_ids: Vec<IdentityXgid>` · `touched`.
- **G-B2 — the T2 gate lives in `mod pass_4_commit_1_tests`** (`ops.rs:3318`), which carries `use super::*` (`:3321`) and the `ix()` / `sx()` / `ex()` helpers (`:3323-3331`). **T2 itself is `:3438-3462`.** ⇒ the new witness needs **no new module and no new helper**.
- **G-B3 — `setResult` has exactly ONE caller.** `app_client.svelte:183`. Grepped across all `*.ts` / `*.svelte` excluding `target`, `.claude`, `node_modules`. ⇒ **the signature change is cheap and total.**
- **G-B4 — the store's shape.** `_isDm` / `_roster` / `_book` / `_phase` at `:92-95`; `setResult(spaceId, roster, book)` at `:132-137`; `setInflight` `:122-127`; `setFailed` `:139-143`; `reset` `:146-150`. **`MemberEntry.unresolved?: boolean` at `:36`** is documented as *"the ONE non-mirror field"*.
- **G-B5 — the TS mirror already carries the ids.** `FillReport.not_found_ids: string[]` at `address-book.svelte.ts:59`; `FillMembersOutcome { fill, roster }` at `:65-67`, whose own comment (`:64`) says the shell reads **`.roster`, not `.1`** — *"not a positional tuple"*.
- **G-B6 — `entity-item`'s prop block is `:30-50`; its root is `:112-120`.** The shipped idiom is `data-selected={selected || undefined}` (`:116`). Aggregate getter `debug()` at `:87-94`.
- **G-B7 — `entity-panel` carries the per-row view-model `EntityItemInput` at `:35-40`** (`descriptor` · `secondary` · `status` · `meta`) and renders it through the `rowBody` snippet at `:152-161`. ⇒ **the hook is a PER-ROW field on `EntityItemInput`, never a panel-level prop.** 📌 *P1 said "a prop through `entity-panel` → `entity-item`"; the panel's own mechanism for per-row data is this type, and using it is what P1 means here.*
- **🛑 G-B8 — A DECLARATION-ORDER HAZARD, AND IT IS THE ONE THING IN THIS LEG THAT WILL BITE SILENTLY.** In `members-panel.svelte`, **`memberDescriptors` is `:106` and `counterpart` is `:123`** — the filter needs `counterpart` and it is declared **seventeen lines later**. `$derived` is lazy, so this may well run; **it is a temporal-dead-zone hazard on first read and must not be left to chance.** ⇒ **Change 5 MOVES `counterpart` above the member list. This is a named step, not a discovery.**
- **G-B9 — `counterpart` is `undefined` outside a DM** (`:123-127` gate on `addressBook.isDm`). ⇒ **`id === counterpart` IS the DM exception**; no separate `is_dm` test is needed in the filter, and none should be added.
- **G-B10 — §5a-i needs no code.** `counterpart` derives from `roster`, and B-1 keeps `_roster` complete, so the erased counterpart is still found at `:125` and still passed as `selected` at `:146`. 🔒 **Joe locked KEEP (2026-08-01). KEEP = zero lines.** ⚠️ *Phase-0 §5a-i priced this as "one condition either way"; that was Chat's and it was wrong — the options are not symmetric. Corrected here, kept not erased (`D-131`).*

---

## §4 — 🔒 THE HOOK CARRIES A VALUE, NOT A PRESENCE (Chat, mechanical — Joe's to overrule at lock)

🛑 **§4 locked `data-unresolved` for ④ (dimmed). §5a's E2 locked a MARK for the erased DM counterpart. THOSE ARE TWO TREATMENTS ON THE SAME ELEMENT, AND A PRESENCE-ONLY ATTRIBUTE CANNOT CARRY TWO STATES.** §5b already refused `data-revoked` as the vehicle (it belongs to M13, and `D-127` separates revoked from erased). ⇒ **a second distinguishable hook is REQUIRED BY LOCKS ALREADY TAKEN.** Only its form was open.

🔒 **FORM: one prop, two values.** `unresolved?: 'unasked' | 'erased'` → `data-unresolved="unasked"` (state ④) / `data-unresolved="erased"` (state ③, DM counterpart only).

🛑 **THE WORD IS `'unasked'`, NOT `'pending'`, AND THE DIFFERENCE IS NOT COSMETIC.** Phase-0 §3 names ② *asked, reply not back* — **that is what "pending" means.** ④ is **never asked**. ⚠️ **If §7's Tier-1 fetch ships as Leg D, ② becomes a real renderable state and will want that word**, and it would already be spent on its opposite. ⇒ **`'pending'` stays free for ②, exactly as §4 kept *"irregular"* free for M13 and §5b refused to squat `data-revoked`.** 📌 *Chat's first draft of this section used `'pending'`; caught before this file was ever committed. **The value strings become literals in Joe's `skin.css`**, so a rename after Leg C would reach into his file — which is why it was worth one turn now.*

- ① **User-visible:** nothing, today — both are skin hooks and the skin is Leg C. **From Leg C on, Joe gets two independent selectors**, and **§4's locked `.entity-item[data-unresolved]` still matches BOTH**, so nothing he locked breaks.
- ② **Resource:** one prop, one attribute, one `debug()` field, threaded through two `core` components.

**Rejected — two booleans (`unresolved` + `erased`).** ① identical on screen. ② two props through two `core` components, **and the type would permit a row that is both never-asked and answered-not-found — a state that cannot exist.** *A type that can express an impossible state is a type that will eventually be handed one.*

⚠️ **THE VALUES ARE THE HOOK, NOT THE APPEARANCE.** `ui/assets/skin.css` **IS JOE'S.** This leg adds the attribute and **writes not one line of CSS.**

---

## §5 — The changes, exactly

### 🔷 COMMIT B-i

#### Change 1 — `ops.rs`, the `Vec` wire witness (in `mod pass_4_commit_1_tests`, immediately AFTER T2 at `:3462`)

**Also append one line to that module's doc-comment (`:3319-3320`)** so the module does not silently grow past its stated scope:

```rust
//! Plus the Vec-level sibling of T2, added for
//! `M-RP-IDENTITY-RESOLUTION` Leg B (Phase-0 §9, owed at J-647).
```

Then the test:

```rust
/// T2-vec — the Vec-level sibling of the wire-invariance witness.
/// `FillReport.not_found_ids` is `Vec<IdentityXgid>`, and each element is a
/// `#[serde(transparent)]` flavour wrapper, so the field must serialise as a
/// plain JSON array of STRINGS, never an array of objects. The TS mirror
/// declares `not_found_ids: string[]` (`address-book.svelte.ts:59`) and that
/// claim had NO witness before this test: T2 covers SCALAR identifier slots
/// only, so citing it for a `Vec` would be a claim narrower than its subject
/// (filed J-647, Phase-0 §9). This test can genuinely fail — adding a serde
/// attribute to the field or to `IdentityXgid` breaks it.
#[test]
fn fill_report_not_found_ids_vec_serde_transparent_wire_invariance() {
    let r = FillReport {
        candidates: 2,
        fetched: 0,
        not_found: 2,
        not_found_ids: vec![
            ix("xgen://pubkey/ed25519:AAA"),
            ix("xgen://pubkey/ed25519:BBB"),
        ],
        touched: 0,
    };
    let json = serde_json::to_string(&r).unwrap();
    assert!(
        json.contains(
            r#""not_found_ids":["xgen://pubkey/ed25519:AAA","xgen://pubkey/ed25519:BBB"]"#
        ),
        "not_found_ids is not a plain array of strings: {json}"
    );
    // An EMPTY vec must still serialise as `[]`, never omitted — the mirror
    // declares the field required and the panel reads it unconditionally.
    let empty = FillReport::default();
    let json_empty = serde_json::to_string(&empty).unwrap();
    assert!(
        json_empty.contains(r#""not_found_ids":[]"#),
        "empty not_found_ids must serialise as []: {json_empty}"
    );
}
```

🔒 **THE STRUCT LITERAL IS EXHAUSTIVE AND NAMED — NO `..Default::default()`.** `FillReport` derives `Default` (`:2772`) and the shortcut would compile, **but an exhaustive literal is what makes this test FAIL TO COMPILE when a field is added**, which is the point of a wire witness. ⚠️ *This mirrors Leg A's own discipline at the production literal.*

⚠️ **THE FIELD ORDER IN THE `contains` ASSERTION IS THE STRUCT'S DECLARATION ORDER.** `serde_json` emits fields in declaration order for a derived `Serialize`; the assertion targets one field's own key-value pair, so it does not depend on neighbours. **Do not reorder the struct.**

**Expected floor:** cargo **1588 → 1589**, `0` failed, `62` ignored, **56 result lines**. 🛑 **`Compiling xgen-client` MUST be present in the output** — the J-647 check that distinguishes *1589 because it ran* from *1589 because it was cached*.

---

### 🔷 COMMIT B-ii

#### Change 2 — `address-book.svelte.ts`: take the outcome whole, keep the id list

**(a) New backing state, beside `:92-95`:**

```ts
/** §5/§5a — the ids that returned `identity.not_found` in the fill that produced the current
 *  roster (state ③, ERASED under `D-127`). Kept BESIDE `_roster`, never stamped onto it:
 *  `MemberEntry` is a mirror of the Rust row with exactly ONE non-mirror field (`unresolved`),
 *  and not-found-ness is a fact about the FILL, not about the member. B-1: `_roster` stays
 *  COMPLETE — the panel decides what to draw. */
let _notFound = $state<string[]>([]);
```

**(b) `setResult` (`:132-137`) takes the outcome whole, NOT a fourth positional argument:**

```ts
setResult(spaceId: string, outcome: FillMembersOutcome, book: AddressBook): void {
  // late-guard UNCHANGED
  _isDm = outcome.roster.is_dm;
  _roster = outcome.roster.members;
  _notFound = outcome.fill.not_found_ids;
  _book = book ?? {};
  _phase = 'ready';
}
```

📌 **WHY THE WHOLE OUTCOME AND NOT A 4TH ARG:** `FillMembersOutcome`'s own comment (`:64`) already refuses positional access — *"the shell reads `.roster`, not `.1`"*. A fourth argument reintroduces exactly what that type exists to prevent, and **every future `fill` field then costs another signature change.**

🛑 **NO `?? []` ON `not_found_ids`.** The TS mirror declares it **required** and Leg A ships it on every path. A defensive `??` here would be **an unfed branch that can never be exercised** (N-091) — and if it ever did fire, silently swallowing a missing field is the failure we would most want to see.

**(c) Public getter, beside the others:**

```ts
/** The ③ ids for the CURRENT roster. Empty is the normal case. */
get notFoundIds(): string[] {
  return _notFound;
},
```

**(d) 🛑 CLEARED ON EVERY PATH THAT INVALIDATES THE ROSTER — three sites, all mandatory:** `setInflight` (`:122-127`), `setFailed` (`:139-143`), `reset` (`:146-150`) each add `_notFound = [];`. ⚠️ **A stale id list outliving its roster would hide a member of the NEXT Space** — the exact shape the late-response guard exists to prevent, one field over.

📌 **`removeMember` does NOT clear it.** A leaver's id lingering in `_notFound` is inert (the filter only ever consults ids that are in the roster). **Recorded as considered, not overlooked.**

#### Change 3 — `app_client.svelte:183`, the discard site

```ts
addressBook.setResult(sid, outcome, book);
```

**One line.** ⚠️ **The surrounding comment at `:178-180` still describes the roster half only — update it to say the fill half now carries the ③ ids.** *A comment left describing the previous contract is this milestone's own recurring defect (N-109 family).*

#### Change 4 — `entity-item.svelte`, the leaf

**(a) Prop, in the block at `:30-50`** — added to both the destructure and the type:

```ts
unresolved,
// …
/** §4/§5a — what the CLIENT knows about this identity, NOT a property of the entity:
 *  `'unasked'` = never looked up (state ④, dimmed) · `'erased'` = `identity.not_found`
 *  (state ③, `D-127`). Absent = resolved. ⚠️ `'pending'` is DELIBERATELY NOT a member
 *  here — it names state ② (asked, reply not back), which is unbuilt and will want the
 *  word if §7 ships. Deliberately NOT on `EntityDescriptor` and NOT on `flags`
 *  (Phase-0 §8 B-2, option P1). */
unresolved?: 'unasked' | 'erased';
```

🛑 **NO DEFAULT VALUE.** `selected = false` has one because it is a boolean state; `unresolved`'s absence **is** the resolved case and `undefined` says so exactly.

**(b) Root attribute at `:116`, beside `data-selected`:**

```svelte
data-unresolved={unresolved}
```

📌 **No `|| undefined` needed** — unlike `selected`, this is already `undefined` when absent. *Do not copy the boolean idiom mechanically; it would be noise here.*

**(c) `debug()` (`:87-94`) gains:**

```ts
unresolved: unresolved ?? null,
```

⚠️ **`?? null`, not the bare value** — the CDP harness reads JSON, and an `undefined` field **vanishes from `JSON.stringify` output**, which reads exactly like a component that does not have the prop. *A probe that cannot distinguish absent-value from absent-feature is not evidence.*

#### Change 5 — `entity-panel.svelte`, the pass-through

**(a) `EntityItemInput` (`:35-40`) gains the field:**

```ts
/** Per-row client-knowledge hook (M-RP-IDENTITY-RESOLUTION §4/§5a). Passed straight to
 *  `entity-item`; the panel never interprets it. */
unresolved?: 'unasked' | 'erased';
```

**(b) `rowBody` (`:152-161`) passes it:**

```svelte
unresolved={item.unresolved}
```

🛑 **NOTHING ELSE IN `entity-panel` CHANGES.** No panel-level prop, no filtering, no ARIA change, **no `interactive` interaction.** The panel is a conduit here; **the day it starts interpreting this value it has taken a decision that belongs to the widget.**

#### Change 6 — `members-panel.svelte`, the filter and the feed

**(a) 🛑 MOVE `counterpart` (`:119-127`, comment block included) ABOVE the member list (`:105`).** G-B8. **Do this first, as its own edit, and re-read the file after it** — the line numbers below shift.

**(b) Replace `memberDescriptors` with a row builder:**

```ts
// §5/§5a — the ③ set for the current roster. `notFoundIds` is a small array; a Set keeps the
// per-row test O(1) and reads as the membership question it is.
const notFound = $derived(new Set(addressBook.notFoundIds));

// State ② members: the roster MINUS self (L4), MINUS state ③ — except the DM counterpart,
// which is NEVER hidden (§5a E2, J-648). ⚠️ `counterpart` is `undefined` outside a DM
// (G-B9), so `=== counterpart` IS the DM exception; do NOT add a separate `isDm` test.
// B-1: `_roster` stays complete — this filters at RENDER, never in the store.
const memberRows = $derived(
  panelState === 'known' && addressBook.roster
    ? addressBook.roster
        .filter((m) => m.identity_id !== selfId)
        .filter((m) => !notFound.has(m.identity_id) || m.identity_id === counterpart)
        .map((m) => ({
          descriptor: toDescriptor(m),
          unresolved: notFound.has(m.identity_id)
            ? ('erased' as const)
            : m.unresolved
              ? ('unasked' as const)
              : undefined,
        }))
    : [],
);
```

🔑 **③ IS TESTED BEFORE ④, AND THE ORDER IS LOAD-BEARING.** A member can be marked `unresolved` by `addMember` (live join) **and** appear in a later fill's `not_found_ids`. **③ is an ANSWER; ④ is the absence of one** — an answer always wins. *Phase-0 §3: they look identical and are opposites.*

**(c) `rows` (`:113-117`) concatenates the new shape:**

```ts
const rows = $derived(
  (selfDescriptor ? [{ descriptor: selfDescriptor }] : []).concat(memberRows),
);
```

🛑 **SELF NEVER CARRIES `unresolved`.** Self is the fixture, present in all five panel states (L2), and never the counterpart (L17). **Leaving the field off self's entry is the correct render, not an omission.**

**(d) `debug()` (`:131-140`) gains one field:**

```ts
erasedHidden: (addressBook.roster ?? []).filter(
  (m) => m.identity_id !== selfId && notFound.has(m.identity_id) && m.identity_id !== counterpart,
).length,
```

✅ **THIS DOES NOT FIRE C1's TRIGGER.** C1 re-opens on *the first milestone that RENDERS a member count*; **a CDP debug aggregate is not a rendered UI count**, and `members-panel` still passes **no `title` and no `badge`** to `EntityPanel` (`:146`), both optional with no default ⇒ **nothing renders a number on screen** (Phase-0 §8 B-4, confirmed at grounding). 📌 *Recorded because `memberCount` vs `rowCount` now diverge here for the first time, and that divergence is C1's accepted mismatch made visible to the harness — which is precisely where it should be visible and nowhere else.*

---

## §6 — Verification

**Static, per commit — every figure RE-DRIVEN by Chat, none read off a report (Rule 5):**

| # | gate | expected |
|---|---|---|
| V1 | `git diff --stat` B-i | **1 file** (`ops.rs`), tests only |
| V2 | `cargo test --workspace` after B-i | **1589 / 0 / 62 across 56 result lines** · `Compiling xgen-client` **present** |
| V3 | `git diff --stat` B-ii | **5 files**, no Rust, no `skin.css` |
| V4 | `cargo test --workspace` after B-ii | **1589 / 0 / 62 UNCHANGED** ⇒ *proves* B-ii shipped no Rust |
| V5 | `svelte-check` | from the floor **0 / 34 / 15**; **any delta explained, not absorbed** |
| V6 | sampler catalogue | **unchanged** — `entity-item` gained an optional prop with no default; **assert it, do not assume it** |
| V7 | `git ls-files --eol` | **`i/lf` on all six files.** ⚠️ `ops.rs` is `w/crlf` in the worktree and `i/lf` in the index — **pre-existing autocrlf, and the committed form must not move** |

**Live (CDP, client 9222) — the one gate that reads the shipped DOM:**

| # | gate | expected |
|---|---|---|
| V8 | `__XGEN_DEBUG__.get('<row id>').state.unresolved` | `null` for a resolved row; **`'unasked'` for a live-joined one** |
| V9 | `[data-unresolved]` on `.entity-item` | attribute **present with a value** on that row, **absent** on resolved rows |
| V10 | members-panel `state.erasedHidden` vs `memberCount` − `rowCount` | consistent |

🛑 **AND THE HONEST LIMIT, STATED SO IT IS NEVER LATER READ AS DONE.** **③ CANNOT BE REACHED IN A NORMAL RUN.** J-649 measured it: a held identity is never re-fetched (`ops.rs:2764`, doc `:2752`) and a held record is never removed in production (`address_book.rs:253` / `:285` — **every caller is a test**) ⇒ **`identity.not_found` fires only for an identity that was NEVER cached.** ⇒ **V8–V10 verify the ④ path and the plumbing; the ③ filter and E2's exception are VERIFIED BY TYPE AND BY READING, NOT BY RUNNING.**

⚠️ **A STORE DRIVEN BY HAND IS A PROBE THAT CANNOT FAIL.** Injecting an id into `_notFound` over CDP would exercise Svelte's reactivity and **nothing else** — it would not test that the id ever arrives. **Do not record such a run as behaviour verification.** 🔒 **Leg F remains the first behaviour verification of this milestone, and it needs two clients, a real join and a real `not_found` — which per J-649 means a client with no cached record.**

---

## §7 — DoD

- [ ] **B-i committed alone**; cargo **1588 → 1589**, `Compiling xgen-client` present (V1, V2)
- [ ] Phase-0 §9's `Vec<IdentityXgid>` witness obligation **struck as DISCHARGED**, in the same commit-pair
- [ ] **B-ii committed alone**; cargo **1589 unchanged** (V4)
- [ ] `svelte-check` re-measured, **delta explained** (V5)
- [ ] Sampler catalogue **asserted unchanged**, not assumed (V6)
- [ ] `git ls-files --eol` **`i/lf` on all six** (V7)
- [ ] `data-unresolved` **read live off the painted DOM**, not inferred from the diff (V8, V9)
- [ ] 🛑 **The close states plainly that ③ was NOT exercised, and why** (J-649)
- [ ] 🛑 **The close states plainly that this leg LANDS SILENTLY** — no dimming, no mark, until Leg C
- [ ] `.md` header updated on every touched document: **Version bumped · `Last updated` = the date CONTENT changed · TWO trailing spaces on every `> ` line**
- [ ] Records: JOURNAL + `CLAUDE.md` PLAY + `docs/ROADMAP.md` + Phase-0 + this file **in one commit** (`D-074`)
- [ ] Citation sweep run **on the NAME, not the version string — and run TWICE**, because the first pass's own fixes create fresh staleness (`D-135` §5a, J-646)

---

## §8 — Seats (D-123)

- **JOE** — locks this runbook; stands Clair up; owns `ui/assets/skin.css` and therefore Leg C; **pushes all commits**. **§4's valued hook is his to overrule at the lock**, and it is cheaper to overrule now than after.
- **CHAT** — authored this file; re-drives **every** gate in §6; owns the records. **Never pushes.**
- **CLAIR** — implements **from this file once it is locked**, in the two-commit order. **She does not close her own leg.**
  - 🔑 **Rule 6 stands: flag a deviation, never absorb it.** At J-516 an implementer who had silently followed a bad runbook instruction would have shipped a `core` → shell dependency. **If any instruction here is wrong, saying so is the job.**
  - 🔑 **A CLEAN ADVERSARIAL READ IS A RESULT, NOT AN ABSENCE** — say so explicitly, as at J-647.

---

## §9 — Filed, NOT fixed (inherited; none of these is Leg B's to close)

- **`M-RP-XGID-SLOT-RETYPE` (`D-136`)** — `SeenRecord` / `FetchedIdentity` / `FillReport` carry `String` identifier slots that should be typed XGIDs. **Worked around, not fixed** (`D-071`).
- **`M_RP_MEMBERS.md` §6a — the `tail-8` lock-versus-build gap.** `.ei-name` is **LEFT-ANCHORED** and clips the **RIGHT**, so every unresolved row keeps the constant `ed25519:` head and loses the distinguishing bytes. ⚠️ **Leg B makes ④ rows more numerous, so the gap becomes more visible with it.** Not fixed here.
- **`entity-avatar.svelte:125` collapses `isAi`'s third state** — `data-ai={flags.isAi || undefined}`, so `false` and absent render identically. Joe's, same family as §4.
- **M13 §3c — erasure is invisible to anyone holding a cached record.** The real defect J-649 uncovered. **Must be designed together with M13's `revoked` + `update_version`, or not at all.**
- **G-B — "the next refresh" does not arrive.** 🛑 **§4's dimming must not SHIP to a user before a refresh trigger exists** — Leg C is where that becomes true, not Leg B.
