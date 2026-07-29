# RUNBOOK — M-RP-LIVEFEED-REFRESH Leg A: the router and the members consumer
> **Status**: PENDING  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and its state

**Leg A of `M-RP-LIVEFEED-REFRESH` — the live event router behind the members and rooms panels.** Parent: `tasks/M_RP_LIVEFEED_REFRESH.md` v1.11.

⚠️ **THIS RUNBOOK IS `PENDING`, NOT LOCKED.** §5 carries one open question with a real user-visible surface and it is **Joe's**. Clair does not open this document until its Status reads `ACTIVE` and §5 carries a lock. Authored by Chat under D-123: open §§ carry recommendations plus D-121 lenses; Joe locks.

🔒 **PRECONDITION DISCHARGED.** §7 of the parent required a second-reader pass over §6's event table against `wire.rs` before any runbook opens. Done 2026-07-29; three findings landed in the parent at v1.11. **The `membership.*` surface is a closed partition — 8 of 8 — which is what makes Leg A openable while Leg B is not** (§6a: the `state.*` half is 5 of 14).

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

## §2 — Scope: exactly two files

| File | Change |
|---|---|
| `ui/common/lib/stores/address-book.svelte.ts` | `addMember(spaceId, entry)` and `removeMember(spaceId, identityId)` — two new setters, same guard shape as `setResult` |
| `ui/client/src/app_client.svelte` | one router function on the existing `:551` listener; the `ingest.push` at `:552` stays byte-identical |

🛑 **NOTHING ELSE.** No `.rs`. No new store. No new Tauri command. No change to `loadMembers`, to the `$effect` at `:169`, or to `members-panel`. **A diff touching a third file is out of scope and stops the leg** (§4 of the parent: the router mutates the same store the fill populates, and gets no privileged setters).

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

## §5 — 🔓 OPEN, JOE'S: HOW DOES A LIVE-JOINED MEMBER RENDER BEFORE THE BOOK KNOWS THEIR NAME?

**The situation, measured:** `addMember` writes into `_roster`. It does **not** write `_book`, and the joiner will not be in `_book` — the book is filled by `fill_space_records`, which this leg deliberately does not call. The widget resolves `identity_id` → `_book`, else the **tail-8 xgid**.

⚠️ **AND THAT TAIL-8 IS ALREADY OVERLOADED.** `M_RP_MEMBERS.md` §6 records that shipped code collapses `not_found` and no-display-name to the same tail-8 render. **A live-joined member becomes a THIRD case wearing the same face** — and unlike the other two, this one resolves by itself a moment later.

### (A) SHIP THE TAIL-8, FILE THE GAP — Chat recommends
1. **User-visible:** the new member appears **immediately** as `xgid…a1b2c3d4`, and stays that way until the next Space re-latch fires a fill. On a quiet Space that could be the rest of the session.
2. **Resource:** zero. It is what the widget already does.
3. **Tier:** honest — the client is not asserting a name it does not have. Widens an already-filed defect **without deepening it**; the third case is added to `M_RP_MEMBERS.md` §6's open item rather than fixed here.

### (B) FIRE `get_address_book` AFTER AN ADD
1. **User-visible:** the name appears if the Rust book already holds that identity from an earlier Space. **⚠️ For a genuinely new person it changes nothing** — the book is filled by the DAG drain, and nobody drained them yet. ⇒ **helps only the case that was already going to be fine.**
2. **Resource:** one invoke per join, plus a whole-book replace into `_book` on the live path. Third writer to a store §4 wants to keep single-writer.
3. **Tier:** adds an invoke to `$common`'s feeder path for a mostly-empty win.

### (C) FIRE A FULL `fill_space_records` ON JOIN
1. **User-visible:** correct names, always.
2. **Resource:** a full Space DAG drain per join. **This is the fill the milestone exists to avoid**, and parent §4's closing note names exactly this misreading.
3. **Tier:** ⛔ defeats the milestone. Listed so the rejection is on the record, not to be chosen.

🔓 **Joe's.** ⚠️ **This is the only item in Leg A with a user-visible surface. Everything else in this runbook is derived from an existing lock or is invisible plumbing.**

---

## §6 — Verification (Chat drives; Clair does not close her own leg)

- `npm run check` in `ui/` — svelte-check re-measured on the final tree, quoted verbatim, **compared against 0 / 34 / 15**.
- `git diff --stat` shows **exactly two files**.
- `git diff` on `app_client.svelte:551-552` shows `ingest.push` **unchanged**.
- ⚠️ **NO CDP RUN AT LEG A.** Live behaviour is Leg D, against a real node with a second identity. **A store driven by hand through `__XGEN_MEMBERS__` is a probe that cannot fail** — it proves the setters work, which the reviewer can see in the diff, and proves nothing about routing.

---

## §7 — Definition of Done

- [ ] `addMember` / `removeMember` exist on the store, both guarded on `spaceId` (R4)
- [ ] R1 asserted in code: a delta against `_roster === null` returns without writing
- [ ] R2 asserted in code: `kick` / `ban` / `node_eject` read `content.target_identity`, with **no `sender` fallback**
- [ ] R3 asserted in code: add-existing and remove-absent are both no-ops
- [ ] the three ignored `membership.*` strings return explicitly, each with the parent's §6 reason in a comment
- [ ] `ingest.push` byte-identical; no store gained a privileged setter (parent §2)
- [ ] `svelte-check` re-measured and quoted; **cargo NOT re-run — a cargo change means scope was exceeded**
- [ ] exactly two files in the diff
- [ ] §5 locked by Joe **before** this leg opens, and the locked option is what shipped

📌 **`Owes:` on close — `M_RP_MEMBERS.md` §6's third unresolved-row case**, added there whichever way §5 lands.

---

## §8 — Out of scope, named so it is a choice

- **Leg B** — the `state.*` consumer. 🛑 **Gated on the parent's §6a**: 9 of 14 `state.*` strings have no row.
- **Leg C** — the reconnect rule. Gated on the parent's §5, which is Joe's and open.
- **The membership author-exclusion cargo test** — parent §7's closing note: it moves the cargo floor while every leg here moves `svelte-check`. Still 🔓 and Joe's.
- **H1 / H2** — the address book at rest and the visit-card verb (parent §5h). Rust and protocol; neither belongs to this milestone.
