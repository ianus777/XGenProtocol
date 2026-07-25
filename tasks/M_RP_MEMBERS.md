# M-RP-MEMBERS — the R7 members widget over the address book
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-25  
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

**G3 — the cost is Rust, and it is a command, not a capability.** `xgen-client/src/desktop.rs` exposes **16** `#[tauri::command]`s (`get_state` · `get_pacing_state` · `get_conn_stats` · `get_resident_status` · `resume_resident` · `send_message` · `get_substitutions` · `set_substitutions` · `get_ui_state` · `set_ui_state` · `get_window_geometry` · `apply_window_geometry` · `get_about_info` · `get_self_state` · `get_spaces` · `quit`). **None touches the address book.** The widget cannot see the book from the frontend at all.

**G4 — the async-op precedent already exists and fits exactly.** `fill_from_space` is `async` + network; `get_spaces` is a sync local-file read, so it is *not* the precedent. `reanchor_space` (`desktop.rs:369`) is: build `SessionState::new(node, data_dir)` from `DataDir` + `ConfigPath` → `ensure_identity` → `ensure_connected` → run → `goodbye`. That is the shape `fill_from_space` wants, **and it satisfies the §4 off-critical-path lock for free**, because a Tauri command is already off the render path.

**G5 — `fill_from_space` is re-entrant by design (J-586) and must be left alone.** The wrapper clears `ctx.session.conn = None` on **every** exit including `?`-skipped error paths. Callers must add **no** connection management of their own, and the clears must not be tidied away.

**G6 — ⚠️ the name R7 can show is the BOTTOM of a three-layer chain, and the other two are unbuilt.** Ch2 §User Representation (L557–580) locks the override chain **contact alias → Space nickname → global display name**. The book holds `display_name` = the **global** name only. The alias lives in `identity_private` (layer ③, 0 code hits). The Space nickname lives in the Space membership record — and `MemberEntry` (`ops.rs:2593`) carries `identity_id · role · joined_at · invited_by`, **no name of any kind**. ⇒ **R7 v1 renders the last resort of the chain for everybody.** Correct and honest, recorded here so nobody later reads a rendered name as one the user chose.

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

⚠️ **CONCRETE CONSEQUENCE — DO NOT FEED `flags.revoked`.** `entity-avatar` already draws a revoked badge from `EntityFlags.revoked`, and the book's `revoked` is `false` on **every** record because the wire never sets it. Feeding it would light a **shipped affordance from a constant false** — the N-097 shape that stranded `entity-item.selected` at M-RP6.1g, and the same rule this project used to keep `tabs` out of renderer A. ⇒ **v1 feeds `flags.isAi` and nothing else**; `revoked` stays unfed until M13.

---

## §4 — DECISION 1: SCOPE — what is R7 a list OF? 🔒 JOE'S (architecture)

ROADMAP L824 says *"R7 is a contact list, not a room roster."* But the book is **global** (one file, everyone ever seen), `fill_from_space` is **Space-scoped**, R1→R2 are scope-chained, and Ch2 §Cross-Space Discoverability (L1859–1863) permits membership visibility **only within a Space**.

**A — the whole book.**
- *User-visible:* everyone ever seen, across all Spaces, in one list. ⚠️ It becomes the first screen in the client that composites identities **across Space boundaries**. No protocol rule is broken — it is the user's own local cache — but it is the correlation Ch2 §Cross-Space Discoverability exists to prevent, rendered by us.
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

---

## §5 — DECISION 2: WHEN DOES FILL RUN? 🔒 Chat's, stated for the record

🔒 **Locked upstream (Joe, address-book §4): FILL RUNS OFF THE CRITICAL PATH.** The Space opens **at once**; records resolve behind and the view updates as they land. **Never gate a Space open on the fetch loop** — at N members that is an unbounded network wait in front of a UI action.

**Chat's implementation reading (technical execution, not a Joe call):** the trigger is the same event that already scopes R2 — a **space selection on the bus**. The shell (or the widget) fires `fill_space_records(space_id)` as a fire-and-forget `invoke`; the store re-reads the book when it resolves. The panel renders from `members` **immediately**, with unresolved rows in the §6 form, and rows resolve in place.

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

**Leg B — the store + the widget.** `$common` address-book store + `members-panel.svelte` + the 7th `CLIENT_PLUGINS` row + shell hydration. Frontend only; moves the **svelte-check floor** (baseline 0 err / 34 warn / 15 files).

**Leg C — live CDP verify (9222).** Registry delta · the §5 off-critical-path lock **exercised** · the §6 unresolved-row render, both cases · the honest empty states · churn-returns-to-baseline as the orphan proxy (N-092a: the client bridge is state-only, there is no `domCount` leg).

**Leg D — records + close.** JOURNAL + CLAUDE.md PLAY + ROADMAP + this doc, **one commit** (D-074).

📌 **A and B split deliberately:** A moves the Rust floor, B moves svelte-check. One commit spanning both makes a regression ambiguous.

---

## §9 — Filed, NOT fixed

- **E2 per-entry delete has no UI** (§4 collision). A locked erasure control with no surface.
- **The whole-book view has no home** if §4 locks B.
- **Contact alias (③) and presence (④) remain unbuilt**, and Ch2 specifies **no contact-acquisition flow at all** (grepped `docs/`; zero) — so the top of the G6 name chain cannot be built even if wanted.
- **Per-tier retention N is not derivable** (J-587): `tier` is reachable only through `trust_assertion`, which the wire never carries. Every record evicts on the T1 default. M13's problem, recorded here because a members widget is where a tier would first be visible.

---

## §10 — DoD

- [x] §4, §6, §7 locked by Joe, in place in this document with the lock date — **2026-07-25, all three DELEGATED**
- [ ] Second reader (Clair, not Chat) has read this Phase-0 against Ch2 §User Representation and Ch2 §Cross-Space Discoverability
- [ ] Leg A: cargo floor re-measured, delta explained
- [ ] Leg B: svelte-check floor re-measured, delta explained
- [ ] Leg C: every claim CDP-measured by Chat, on the real client 9222; the off-critical-path lock **exercised**
- [ ] `xgen-client_address_book.json` **exists after a normal session** — the milestone's one-line proof
- [ ] `flags.revoked` verifiably UNFED
- [ ] Records: JOURNAL + PLAY + ROADMAP + this doc in one commit

---

## §11 — Handoff

🔒 **ALL THREE DECISIONS LOCKED 2026-07-25 — §4 B · §6 tail-8 · §7 unfed — ALL THREE DELEGATED.** Joe adopted Chat's recommendations wholesale (*"lock all by your recomms"*) rather than walking them. Under D-123 these three are **his** areas, so the delegation is recorded as provenance on each one: they are decisions of record, but not decisions he examined, and the re-open cost is low on all three (a latch, a label format, a prop).

⚠️ **ONE DoD ITEM STILL OPEN BEFORE ANY RUNBOOK: the second-reader pass.** This document was written by one author interpreting Ch2 while writing against it — the exact condition that produced four defects last arc (J-587), none of which care caught and every one of which a second reader working from the source did. **Chat cannot be its own second reader.** ⇒ **Clair reads this Phase-0 against Ch2 §User Representation (L557–580) and Ch2 §Cross-Space Discoverability (L1859–1863) BEFORE Leg A opens**, and reports against the source, not against this text. Expect D-122 to fire; it fired three times last arc.

**Then:** Leg A (Rust read surface) and Leg B (store + widget) as two separate Clair runbooks — A moves the cargo floor, B moves svelte-check, and one commit spanning both makes a regression ambiguous.
