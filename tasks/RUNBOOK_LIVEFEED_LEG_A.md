# RUNBOOK — M-RP-LIVEFEED-REFRESH Leg A: the router and the members consumer
> **Status**: COMPLETED  
> Version: 1.5  
> Date: Jul 2026  
> **Last updated**: 2026-08-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and its state

**Leg A of `M-RP-LIVEFEED-REFRESH` — the live event router behind the members and rooms panels.** Parent: `tasks/M_RP_LIVEFEED_REFRESH.md` ~~v1.11~~ **v1.12**.

✅ **EXECUTED AND VERIFIED 2026-08-01 (Clair implemented; Chat re-drove every gate independently — J-643). STATUS → COMPLETED.** Three files, **103+/3−**; `svelte-check` **0 err / 34 warn / 15 files** = the floor exactly; cargo not run; `ingest.push` byte-identical, confirmed **from the diff** (space prefix, no `−`, no re-add). ⚠️ **§5-iii ①'s user-visible claim did NOT survive the re-drive — see the annotation in §5-iii.**

⚠️ **THIS RUNBOOK IS `ACTIVE` AND §5 IS CLOSED (Chat, 2026-07-29).** 🔑 **v1.0 and v1.1 both routed §5 to Joe and BOTH WERE WRONG ON THE SEAT.** Joe asked why it was his; it was not. **Joe owns choices between honest options — `M_RP_MEMBERS.md` §6's word form is one. He does not own whether the client asserts something it does not know.** §5② had one honest answer and three dishonest ones, already determined by §6's own governing rule (*staleness AND absence both render UNKNOWN, never as fine*), by D-065, and by the `revoked`-unfed precedent (N-097). ⚠️ **Presenting four options when one was live is UNDER-STEPPING — the named recurring seat error — and the mechanism was that the finding broke §2 of this same runbook two hours after Chat wrote it.** Routing it to Joe was a way of not owning a scope change of Chat's own. **Recorded rather than absorbed.**

🔒 **PRECONDITION DISCHARGED.** §7 of the parent required a second-reader pass over §6's event table against `wire.rs` before any runbook opens. Done 2026-07-29; three findings landed in the parent at v1.11. **The `membership.*` surface is a closed partition — 8 of 8 — which is what makes Leg A openable while Leg B is not** (~~§6a: the `state.*` half is 5 of 14~~ ⚠️ **SUPERSEDED AT v1.5: the `state.*` half was classified at J-641 — parent §6a-i registers all 17 rows. Leg B is now gated on Joe's B1/B2/B3 scope ruling instead, per §8.**).

📌 **Not blocked by §5 of the parent.** §10 of the parent: *"§5 (the reconnect rule, gates Leg C only)."* Verified by reading §10, not inherited.

---

## §1 — Grounding (measured 2026-07-29 at `a715ccb`, HEAD = origin/main, tree clean)

**The listener already exists and this leg does not create one.** `app_client.svelte:551` registers `listen('xgen-event', …)` and pushes the payload into `ingest` (`:552`). `xgen-client/src/desktop.rs:395` emits the whole `Event` verbatim; `:824` is the drain's sole call site.

**The payload's type field is `type`, not `event_type`.** `Event` carries `#[serde(rename = "type")]` (`xgen-common/src/wire.rs:476`) ⇒ the webview branches on `payload.type` holding a wire string such as `"membership.join"`.

**The store is a pure `$state` store with four setters and no live path.** `ui/common/lib/stores/address-book.svelte.ts` (6,753 B): `setInflight` · `setResult` · `setFailed` · `reset`. No `addMember`, no `removeMember`. The only writers are inside `loadMembers` (`app_client.svelte:173/:176/:183/:187`), whose sole caller is the `$effect` on `roomLatch.effectiveSpaceId` (`:169`).

**`_roster` is `MemberEntry[] | null` and `null` MEANS NOT KNOWN.** The store's own comment: *"an empty array is unreachable (L5: if you are scoped to a room you are in it), so `null` and `[]` are NOT the same render."*

**`MemberEntry` is `{ identity_id, role, joined_at, invited_by }`** — a mirror of the Rust serialisation, no name field. Names resolve through `_book`. The comment records that `role` / `joined_at` *"arrive free and are deliberately discarded by the widget (L10)."*

**Floors at open — NOT re-measured this session, inherited from J-601 and to be re-measured at close:** cargo **1588 / 0 / 62 × 56** · svelte-check **0 err / 34 warn / 15 files**. ⚠️ **Leg A is frontend-only and must move `svelte-check` only; a cargo delta means the scope was exceeded.**

---

## §2 — Scope: exactly three files

| File | Change |
|---|---|
| `ui/common/lib/stores/address-book.svelte.ts` | `addMember(spaceId, entry)` and `removeMember(spaceId, identityId)` — two new setters, same guard shape as `setResult`; plus the **unresolved marker** §5 requires |
| `ui/client/src/app_client.svelte` | one router function on the existing `:551` listener; the `ingest.push` at `:552` stays byte-identical |
| `ui/common/lib/components/widgets/members-panel.svelte` | **ONE branch** in `toDescriptor`: a member the book was never consulted for carries **no `isAi` claim** rather than `isAi: false` (§5) |

🔑 **THE THIRD FILE IS §5's ANSWER, NOT SCOPE CREEP, AND IT IS UNAVOIDABLE.** The `?? false` is evaluated at render time from `_book`; `MemberEntry` has no `isAi` field, so **the router cannot reach it from the store side at all.** ⚠️ **A two-file Leg A necessarily ships a false claim.** 📌 **Widening §2 is not widening the leg:** the floors moved are still **`svelte-check` only**, and the A/B split that keeps regressions attributable is untouched.

🛑 **NOTHING ELSE.** No `.rs`. No new store. No new Tauri command. No change to `loadMembers`, to the `$effect` at `:169`, or to `members-panel`'s five-state tree, its self fixture, its DM counterpart or its inert contract. **A diff touching a fourth file is out of scope and stops the leg** (§4 of the parent: the router mutates the same store the fill populates, and gets no privileged setters).

---

## §3 — The four rules that are NOT obvious from the parent's §6

🔑 **These are the reason this runbook exists. A correct-looking router that violates any of them passes a naive read.**

### R1 — 🛑 A DELTA ONTO AN UNKNOWN ROSTER IS DROPPED, NEVER PROMOTED

If `_roster === null`, **both setters return without writing.** Adding to `null` would produce `[joiner]` — a roster of exactly one person — and the panel cannot distinguish that from a real one-member Space. **That converts *"I do not know who is here"* into a confident lie**, which is the single worst thing this leg could ship.

Derived, not chosen: §4 of the parent locks *"the router handles deltas onto an already-filled store"* and *"a store with no fill has no business on the router."* R1 is that sentence made executable.

### R2 — 🛑 THE SUBJECT OF THE EVENT IS NOT ALWAYS `sender`

Per the parent's §6 table and §6-i:

- `membership.join` → subject is **`payload.sender`**
- `membership.leave` → subject is **`payload.sender`**
- `membership.kick` · `membership.ban` · `membership.node_eject` → subject is **`payload.content.target_identity`**; `sender` is the moderator or the node

⚠️ **A uniform `sender` read passes `join` and `leave` and silently removes the wrong person on the other three.** ⚠️ **And `target_identity` is a convention, not a type** — only `MembershipMuteContent` declares it (`wire.rs:712-713`), and that is the one event this milestone ignores. **Read it defensively; if it is missing or not a non-empty string, DROP the event and do not fall back to `sender`.** A fallback here is the defect wearing a seatbelt.

### R3 — 🛑 BOTH SETTERS ARE IDEMPOTENT

`addMember` for an `identity_id` already in the roster is a **no-op**, not a duplicate row. `removeMember` for an absent one is a **no-op**, not an error. The client has no replay suppression of its own and the drain makes no exactly-once promise.

### R4 — 🛑 THE SCOPE GUARD IS THE STORE'S OWN `_spaceId`, COMPARED AGAINST `payload.space_id`

An event for a Space the user is not scoped to must not touch the roster (parent §6: *"scoping is a correctness requirement, not an optimisation"*).

🔒 **LOCKED BY CHAT, and the delegation argument is the parent's own §2** — zero user-visible surface, invisible plumbing, re-open freely. The guard lives **inside the two setters**, taking `spaceId` as their first argument and discarding on mismatch, exactly as `setResult` / `setFailed` already do. **Reason: `_spaceId` is written FIRST by `setInflight` and is therefore the late-response reference the store was built around.** Guarding in the router instead would create a second scope authority for the same store, and the two could disagree during a scope change — the shape D-067 exists to forbid.

📌 The router still passes `payload.space_id`; it does not read `roomLatch` at all.

---

## §4 — The router

One function in `app_client.svelte`, called from the existing listener. Shape, not code:

1. `ingest.push(payload)` stays exactly where it is and runs unconditionally — **R5's store is untouched** (parent §2's closing lock).
2. Read `payload.type`. If it does not start with `membership.`, return.
3. Switch on the wire string. `membership.invite` · `membership.mute` · `membership.node_unban` → **return, deliberately** (parent §6). An unrecognised `membership.*` string → return.
4. Resolve the subject per R2. Missing or malformed → return.
5. Call `addressBook.addMember(payload.space_id, entry)` or `addressBook.removeMember(payload.space_id, subject)`.

⚠️ **`buildEntry` for the add path must not invent fields.** `identity_id` = subject · `joined_at` = **`payload.timestamp`** (real, on the wire) · `invited_by` = `null` (not carried) · `role` = **`''`**. 🔑 **Empty string, not `'member'`.** Both are discarded by the widget today, but `'member'` is a claim about authority that no wire field supports, and the field is `pub role: String` on the Rust side — a future reader would take it as fetched. **Honest-empty is the D-065 answer.**

---

## §5 (RUNBOOK) — ✅ CLOSED BY CHAT: WHAT DOES A LIVE-JOINED MEMBER LOOK LIKE BEFORE THE BOOK KNOWS THEM?

🛑 **THIS HEADING READ `🔓 OPEN, JOE'S` UNTIL 2026-07-31 (J-639), WHILE §0 AND §5-iii BOTH RECORDED IT CLOSED.** Corrected, not erased. 🔑 **A RUNBOOK IS EXECUTED TOP-DOWN BY SOMEONE WHO WAS NOT IN THE CONVERSATION** — an implementer reaching a `🔓 OPEN, JOE'S` heading either stalls or routes a settled question back to him, which is **the exact under-stepping J-618 ruled against, re-created by the heading of the section that records the ruling.** ⚠️ **AND THIS MILESTONE HAS TWO `§5`s: this one (the live-joined member's face, Chat's, CLOSED) and the PARENT's `§5` (the reconnect rule, 🔓 JOE'S, GENUINELY OPEN, gating Leg C only).** §0 already draws that line; the heading is now explicit so the two cannot be collapsed by a reader who starts here.

⚠️ **v1.0 OF THIS SECTION ASKED A NARROWER QUESTION THAN THE SITUATION CONTAINS, AND ITS RECOMMENDATION IS WITHDRAWN.** It asked only about the NAME. Walking the render path end to end found a **second** consequence that is worse than the first, and a **scope collision** with §2. Same species as §6-i — caught by opening the thing, not by re-reading the text.

### §5-i — THE RENDER PATH, MEASURED 2026-07-29 AT `a390a26`

`toDescriptor` (`members-panel.svelte`) is the only name path for a non-self row:

```
name:  rec?.display_name ?? tail(m.identity_id)
flags: { isAi: rec?.is_ai ?? false }
where  rec  = addressBook.book[m.identity_id]
       tail = (xgid) => xgid.split('/').pop() || xgid
```

A live-added member is **not in `_book`** ⇒ `rec` is `undefined` ⇒ both fallbacks fire.

🛑 **① THE FALLBACK NAME IS NOT A TAIL-8. IT IS THE WHOLE FINAL PATH SEGMENT, AND IT IS CLIPPED FROM THE WRONG END.** An identity id is `xgen://pubkey/ed25519:<base64>` (`auth.rs:133` `PUBKEY_URI_PREFIX + pubkey_b64`; `mod.rs:267` builds the same shape) ⇒ `tail()` returns **`ed25519:<~44 chars>`**. `entity-item.svelte:123` renders it into `.ei-name`, which `skin.css:2452-2458` styles `overflow: hidden; text-overflow: ellipsis; white-space: nowrap` — **left-anchored, no `direction: rtl`.** ⇒ the user sees **`ed25519:AbCd…`**, and 🔑 **the `ed25519:` prefix is IDENTICAL on every identity in the system, so the clip discards the only distinguishing bytes.** ⚠️ **TWO unresolved members are indistinguishable FROM EACH OTHER**, not merely from resolved ones. 📌 **This is a `M_RP_MEMBERS.md` §6 lock-versus-build gap, NOT Leg A's:** §6 locked *tail-8*, which would have kept the distinguishing end; the shipped code keeps the constant head. **Filed here because Joe locked *tail-8* and should know the build does something else. It is not this leg's to fix.**

🛑 **② AND THE WORSE ONE: `?? false` TURNS *UNKNOWN* INTO *DEFINITELY NOT AN AI*.** `EntityFlags.isAi` is **`isAi?: boolean`** (`ui/core/lib/components/data-dependent/types.ts`) — the type **has** a third state and the widget collapses it. ⇒ **an AI identity that joins live renders with no AI badge, as a human.**

🔑 **THIS IS THE N-097 TRAP, MIRRORED, AND LEG A IS WHAT CREATES IT.** `M-RP-MEMBERS` refused to feed `flags.revoked` precisely because *lighting a shipped affordance from a constant false* is a lie. This is the same defect inverted: **an absent record UNLIGHTS a badge that should be lit.** ⚠️ **It cannot happen today** — the roster only ever arrives from a fill, and the fill always populates the book in the same breath (`app_client.svelte:183-187`). **The roster-without-book state does not currently exist. Leg A invents it.** ⇒ the panel's own governing rule — *staleness and absence both render UNKNOWN, never as fine* — is broken by this leg unless something is done.

🛑 **③ THE SCOPE COLLISION: LEG A AS §2 SCOPES IT CANNOT FIX ②.** The `?? false` is evaluated **at render time, in `members-panel.svelte`**, from `_book`. `MemberEntry` carries no `isAi` field, so **the router cannot influence it from the store side at all.** ⇒ **any honest fix touches a THIRD file, and §2 says a third file stops the leg.** The collision is real and is not resolvable by writing the router more carefully.

### §5-ii — THE OPTIONS, RE-DERIVED

**(A) SHIP AS-IS — two files, accept both consequences.**
- ① **User-visible:** the member appears instantly as `ed25519:…`, indistinguishable from any other unresolved row, **until the room is re-latched** — the fill's only trigger is the `$effect` on `roomLatch.effectiveSpaceId` (`:169`), so on a room nobody leaves, **that is the rest of the session.** ⚠️ **And an AI joiner is rendered as a human for the same duration.**
- ② **Resource:** zero.
- ③ **Tier:** the name half is honest. ⚠️ **The `isAi` half is a false claim the client did not previously make.**

**(B) RE-FETCH `get_address_book` AFTER AN ADD — still two files.**
- ① **User-visible:** resolves name **and** `isAi` **iff the Rust book already holds that identity from an earlier Space.** For a genuinely new person nothing changes — the book is filled by a DAG drain and nobody drained them. ⇒ **helps the case that was already going to resolve, and misses the case that motivates the question.**
- ② **Resource:** one invoke per join plus a whole-book replace into `_book` on the live path — a second writer to a store §4 wants single-writer.
- ③ **Tier:** narrows the false window, does not close it. **A partial fix to a correctness claim is still a correctness claim.**

**(C) FULL `fill_space_records` ON JOIN.** ⛔ A whole Space DAG drain per join — **the fill this milestone exists to avoid**; §4's closing note names this misreading by name. Listed so the rejection is on the record.

**(D) DON'T ASSERT WHAT IS NOT KNOWN — three files.** The widget learns to tell *book consulted, no name* from *book never consulted*, and renders the second as explicitly unresolved with **no `isAi` claim** (`isAi` is already optional — the type needs nothing).
- ① **User-visible:** the joiner appears immediately and **reads as unresolved rather than as a stranger with an odd name and a confident not-an-AI**.
- ② **Resource:** a marker from the router, plus a branch in `members-panel.svelte`. ⚠️ **Breaks §2's two-file scope.**
- ③ **Tier: the only option that asserts nothing false.** 📌 **And its RENDER FORM is not a new question** — it is `M_RP_MEMBERS.md` §6's *distinguishable unresolved rows*, locked at J-588 with the **word form deferred and explicitly Joe's**. D does not open a decision; it **triggers one that has been sitting open since J-588.**

### §5-iii — 🔒 CLOSED: **D — DON'T ASSERT WHAT IS NOT KNOWN. §2 WIDENS TO THREE FILES.**

🔒 **Chat's, taken 2026-07-29, not delegated and not Joe's to be asked for.** A and B ship the panel making a claim about a person the client cannot support, on a network whose entire premise is that **you know who you are talking to**. ⚠️ **The one place this project has repeatedly refused to cut a corner is exactly here** (`revoked` unfed · `entity-item.selected` stranded · *observations, not current truth*). **`isAi: false` from an absent record would be the first time it did.**

**What ships:**
- A live-added member carries an **unresolved marker** the fill's members do not.
- `toDescriptor` branches on it: **`flags` omits `isAi` entirely** rather than defaulting it. `isAi?: boolean` is already optional — **the type change is nothing, only the collapse is removed.**
- **The NAME is unchanged** — `tail()` as today. 📌 **Finding ① is `M_RP_MEMBERS.md` §6's lock-versus-build gap, not this leg's**, and is filed there, not fixed here.
- 🔑 **D IS A SUBTRACTION. NOTHING NEW APPEARS ON SCREEN**, so §6's still-deferred word form is **not** required and stays deferred and Joe's.

⚠️ **THE ONE THING THAT DOES GO TO JOE, AND IT GATES NOTHING:** he locked **tail-8** at J-588; the build renders the constant `ed25519:` head instead (§5-i ①). **His lock and the shipped code disagree.** It belongs to `M_RP_MEMBERS`, it blocks no leg here, and it is filed rather than silently accepted.

---

### 🛑 §5-iv — ①'s USER-VISIBLE CLAIM IS FALSE. THE RENDERER COLLAPSES THE THIRD STATE ONE LAYER BELOW THE ONE D FIXES. (Chat, measured 2026-08-01 at execution)

**`ui/core/lib/components/data-dependent/entity-avatar.svelte:125`:**

```
data-ai={flags.isAi || undefined}
```

🛑 **`false || undefined` and `undefined || undefined` BOTH EVALUATE TO `undefined` ⇒ THE ATTRIBUTE IS OMITTED EITHER WAY.** The rendered DOM is **identical** whether `isAi` is `false` or absent. ⚠️ **An AI identity joining live still renders as a human, exactly as before this leg.**

⇒ **§5-ii option D ① — *"the joiner … reads as unresolved rather than as a stranger with an odd name and a confident not-an-AI"* — IS NOT DELIVERED, AND CANNOT BE BY ANY CHANGE INSIDE §2's THREE FILES.** `entity-avatar.svelte` is not one of them and widening to it would be a fourth file.

🔑 **§5-iii CONTRADICTED ITSELF AND NOBODY READ THE TWO LINES AGAINST EACH OTHER.** *"D IS A SUBTRACTION. NOTHING NEW APPEARS ON SCREEN"* sits two paragraphs below D ①'s claim that the row **reads** differently. **Both cannot be true of a renderer that already erases the distinction.** ⚠️ **The `?? false` collapse was removed at the DESCRIPTOR layer without measuring that the SAME collapse exists at the DOM layer** — the recurring species again, a fix narrower than the thing it describes, and caught only by opening `entity-avatar.svelte` at re-drive time.

✅ **WHAT THE LEG DOES DELIVER, AND IT IS NOT REVERTED:** the client **no longer asserts `isAi: false` about a person it has never looked up**. The marker distinguishes *book never consulted* from *book consulted, no name*. **The store's data is honest; only the renderer collapses it**, so a future `entity-avatar` that renders the third state gets correct input **with no store change**. 📌 **The implementation matched the lock exactly — the defect is in the lock, not in the code.**

🔓 **WHETHER THE RENDERER SHOULD DISTINGUISH IT IS JOE'S, DEFERRED BY HIM UNTIL AFTER THIS LEG.** It is `M_RP_MEMBERS.md` §6's still-open word form — 📌 **and note that D was chosen partly BECAUSE it was a subtraction that would not trigger that question. It triggers it anyway.**

---

## §6 — Verification (Chat drives; Clair does not close her own leg)

- `npm run check` in `ui/` — svelte-check re-measured on the final tree, quoted verbatim, **compared against 0 / 34 / 15**.
- ~~`git diff --stat` shows **exactly two files**.~~ ⚠️ **SUPERSEDED AT v1.5 — THE COUNT WAS STALE AND CONTRADICTED THIS RUNBOOK'S OWN §2 AND §7 (`D-131`: annotate, never silently repair).** §5-iii widened §2 to **three** files on 2026-07-29; §2 and §7 both moved and **§6 did not**. 🔑 **Same species as J-629: the correction was narrower than the claims it corrected.** ⚠️ **A verification gate contradicting its own Definition of Done stops the implementer at the last step** — caught INDEPENDENTLY by Chat and by Clair on first read. ✅ **CORRECT GATE: `git diff --stat` shows EXACTLY THREE FILES** — measured at execution: **3 files, 103+/3−**.
- `git diff` on `app_client.svelte:551-552` shows `ingest.push` **unchanged**.
- ⚠️ **NO CDP RUN AT LEG A.** Live behaviour is Leg D, against a real node with a second identity. **A store driven by hand through `__XGEN_MEMBERS__` is a probe that cannot fail** — it proves the setters work, which the reviewer can see in the diff, and proves nothing about routing.

---

## §7 — Definition of Done

✅ **ALL TEN VERIFIED BY CHAT ON THE RE-DRIVE, 2026-08-01 (J-643). Clair implemented and did NOT close her own leg.**

- [x] `addMember` / `removeMember` exist on the store, both guarded on `spaceId` (R4) — `:165` / `:173`, **guard form identical to the existing `setResult` / `setFailed` at `:128` / `:137`**
- [x] R1 asserted in code: a delta against `_roster === null` returns without writing — both setters
- [x] R2 asserted in code: `kick` / `ban` / `node_eject` read `content.target_identity`, with **no `sender` fallback** — plus a `typeof` + non-empty check, drop on absence
- [x] R3 asserted in code: add-existing and remove-absent are both no-ops — `.some()` precheck on both sides
- [x] the three ignored `membership.*` strings return explicitly, each with the parent's §6 reason in a comment
- [x] **§5's answer built: a live-added member carries the unresolved marker, and `toDescriptor` OMITS `isAi` for it — never `false`** ⚠️ **built exactly as locked; see §5-iv — the RENDERER collapses it anyway, so ①'s user-visible claim is false**
- [x] `ingest.push` byte-identical; no store gained a privileged setter (parent §2) — verified **from the diff** (space prefix, no `−`, no re-add), not from the file
- [x] `svelte-check` re-measured and quoted — **"svelte-check found 0 errors and 34 warnings in 15 files"** = the floor exactly; **cargo NOT re-run**
- [x] exactly three files in the diff — **3 files, 103+/3−; only 3 lines removed in the whole leg**
- [x] `M_RP_MEMBERS.md` §6 carries the **tail-8 lock-versus-build gap** (§5-i ①), filed not fixed — landed by Chat at J-643

📌 **`Owes:` on close — `M_RP_MEMBERS.md` §6's third unresolved-row case**, added there whichever way §5 lands. ✅ **DISCHARGED at J-643** — and ⚠️ **§5-iv widened what is owed**: the case is no longer only a store-side distinction but a **renderer** question, which is Joe's and deferred by him until after this leg.

---

## §8 — Out of scope, named so it is a choice

- **Leg B** — the `state.*` consumer. ~~🛑 **Gated on the parent's §6a**: 9 of 14 `state.*` strings have no row.~~ ⚠️ **SUPERSEDED AT v1.4 — THAT GATE HAS FIRED AND IS DISCHARGED (`D-131`: annotate, never silently repair).** The classification pass landed at **J-641**: parent §6a-i registers all **17** rows (14 `state.*` + 3 `dm.*`) across five verdict classes. 🔓 **Leg B is now gated on JOE'S B1/B2/B3 SCOPE RULING instead** — parent §6-ii found that **`get_spaces` reads disk while the router writes memory**, so the three options produce three different runbooks. **Leg B's runbook may be AUTHORED, not LOCKED.** 📌 **NONE OF THIS REACHES LEG A** — §4 step 2 returns early on anything that is not `membership.*`, and `spaces-state.svelte.ts` is not one of §2's three files.
- **Leg C** — the reconnect rule. Gated on the parent's §5, which is Joe's and open.
- **The membership author-exclusion cargo test** — parent §7's closing note: it moves the cargo floor while every leg here moves `svelte-check`. Still 🔓 and Joe's.
- **H1 / H2** — the address book at rest and the visit-card verb (parent §5h). Rust and protocol; neither belongs to this milestone.
