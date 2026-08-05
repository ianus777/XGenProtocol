# M-RP-TAIL8 — the unresolved-row fallback shows a short tail, not the whole key — Phase-0
> **Status**: ACTIVE  
> Version: 1.2  
> Date: Aug 2026  
> **Last updated**: 2026-08-05  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is not

**It is:** the discharge of `M_RP_MEMBERS.md` **§6a** — the lock-versus-build gap open since J-643. Joe locked **tail-8** at J-588 (`M_RP_MEMBERS.md:476`); the build renders the whole key and lets CSS clip it.

🔒 **THIS MILESTONE OPENS NO DECISION ABOUT WHETHER TO DO IT.** J-588 is a live Joe lock, and J-677 established that the *"keep `tail()`"* line which appeared to overturn it was **never uttered by Joe** (`D-141`). ⇒ **the leg is EXECUTION, and only its FORM and SCOPE are open.**

**It is NOT:** `D-126`'s word form (deferred by Joe at J-588, untouched) · the `entity-context-menu` deferral · the feed's own truncation unless §4 says so · anything on the wire · any `.rs`.

🔒 **JOE'S LOCK, 2026-08-05:** the visible form is **`…` + the last 8 characters** — `…7WGuWOqU`. The ellipsis is **U+2026 HORIZONTAL ELLIPSIS** (Alt+0133), the single character, `E2 80 A6` in UTF-8 — **not three periods.**

---

## §1 — Grounded, measured at `1ea59de`, not inherited

| fact | site |
|---|---|
| `const tail = (xgid) => xgid.split('/').pop() \|\| xgid` | `members-panel.svelte:30` |
| XGID shape `xgen://pubkey/ed25519:<43 chars>` ⇒ `tail()` returns **`ed25519:` + all 43** | mint sites; J-657's derivation |
| `.ei-name` clips: `overflow:hidden` · `text-overflow:ellipsis` · `white-space:nowrap` | `skin.css:2475-2482` |
| the row looks short **only because CSS clips it**, at whatever width the panel has | measured Leg F: `ed25519:7WGuWOqU…` |

🛑 **`tail()` HAS TWO CALL SITES, NOT ONE — AND THE SECOND IS SELF.**

- `:96` — `name: rec?.display_name ?? tail(m.identity_id)` — the member row.
- `:82` — `name: selfState.identity.display_name ?? tail(selfState.identity.identity_id ?? '')` — **the SELF panel.**

⇒ **changing `tail()` changes how Joe's own row renders when he has no `display_name`.** *Filed as a scope fact for §3, not assumed away.*

---

## §2 — 🛑 THE FINDING THIS PHASE-0 EXISTS FOR: THE ELLIPSIS LEAKS INTO THE AVATAR

**`entity-avatar.svelte:83-95`, read whole:**

```
deriveInitials(nm, xgid):
  if (nm non-empty)  → 2 graphemes of nm            ← the NAME path
  else               → xgid alphanumerics.slice(-2) ← the xgid fallback
```

🔑 **`members-panel` ALWAYS passes a name** (`:96` — `display_name ?? tail(...)`) ⇒ **`nm` is never empty for an unresolved row** ⇒ **the xgid fallback at `:94` is NEVER REACHED on this surface.**

⇒ today: `nm = "ed25519:7WGu…"` → one word → first 2 graphemes → **`ED`**.

⚠️ **AND THIS CORRECTS RUNBOOK §8 FINDING 5's STATED MECHANISM (annotated, not repaired — `D-131`).** It reads *"the avatar initials derive from the literal string `ed25519`"*, which suggests a dedicated algorithm-initials path. **There is none.** It is the **ordinary name path fed a name that begins `ed25519:`.** *The effect is identical; the mechanism is not — and the difference is exactly what makes the next line visible.*

🛑 **CONSEQUENCE OF THE LOCKED FORM: `nm = "…7WGuWOqU"` → first 2 graphemes → `…7`.** **The avatar would render an ellipsis as a person's initial.**

🔑 ***A change locked at one surface reaches a second surface through a shared prop, and nobody looking at either file alone would see it.***

---

## §3 — 🔓 OPEN, AND JOE'S UNDER `D-123` ②: how the avatar is kept out of it

`D-123` ②: *a technical decision that acquires appearance consequences stops being Chat's.* **This one did, so it comes back named rather than resolved.**

**(A) Pass the xgid fallback deliberately** — send the descriptor a name **and** let the avatar take its `:94` xgid path for unresolved rows. ① *User-visible:* row `…7WGuWOqU`, avatar `QU`-style two-character tail — **the two agree, both derived from the same key, neither is punctuation.** ② *Resource:* the largest of the three — a prop or a flag through `entity-panel` → `entity-item` → `entity-avatar`, i.e. the P1 shape again. **Not one function.**

**(B) Ellipsis in the SKIN, not the string** — the component returns bare `7WGuWOqU`; a CSS `::before` on `[data-unresolved] .ei-name` paints the `…`. ① *User-visible:* **identical to what Joe locked** — the reader sees `…7WGuWOqU`. Avatar initials become `7W`. ② *Resource:* one function + one skin rule. 📌 **`D-138`: the rule is Chat's mechanism, its exact form is Joe's.** ⚠️ *A `::before` is not selectable text — a user copying the name gets `7WGuWOqU` without the ellipsis, which is arguably more correct.*

**(C) Accept `…7`** — ship the ellipsis in the string and let the avatar show it. ① *User-visible:* an avatar reading `…7`, **punctuation as a person's initial**. ② *Resource:* zero extra.

**🔓 CHAT RECOMMENDS (B).** It delivers Joe's locked appearance exactly, costs one rule beyond the one-line change, and **keeps the ellipsis where it belongs — it is a rendering signal, not part of the identifier.** *(A) is correct but is a three-file prop change wearing a one-line milestone's name; (C) ships a defect Joe would see on first launch.*

🔒 **LOCKED (Joe, 2026-08-05): (B) — *"exactly right"*.** The component returns bare `7WGuWOqU`; the `…` is painted by the skin. ⇒ avatar initials become **`7W`**, agreeing with the row. **The `…7` failure cannot occur.**

🛑 **(B) IS WITHDRAWN AT v1.2 — IT CANNOT EXPRESS THE RULE, AND THE FAULT IS IN CHAT'S DESCRIPTION, NOT IN JOE'S CHOICE. Annotated, not repaired (`D-131`).** The skin would have to key on `data-unresolved`, but that attribute and *"the name is a fallback"* are **different predicates with partial overlap** — `toDescriptor` resolves the name (`:96`) independently of how `unresolved` is computed (`:128-135`), and `SeenRecord.display_name` is **optional** (`address-book.svelte.ts:75`). **Four row types reach a tail-rendered name; only one carries `unasked`:**

| row | `data-unresolved` | name shown | a `::before` would be |
|---|---|---|---|
| live joiner, not yet fetched | `unasked` | tail | ✅ right |
| erased, **cached name survives** | `erased` | **`DAVE`** | 🛑 **`…DAVE`** |
| erased, no cached record | `erased` | tail | ❌ missed |
| **resolved, person never set a `display_name`** | **absent** | **tail** | ❌ **missed — the common case** |

🔒 **RE-LOCKED (Joe, 2026-08-05, DELEGATED — *"let's go by your recomms finaly"*): MECHANISM M3.** *(The mechanisms are renamed M1/M2/M3 at v1.2: (A)/(B)/(C) collided with the A1–A4 SURFACE labels in conversation — the fourth two-meaning token of this arc, and this one was Chat's.)*

- **M1** — a new marker prop through `entity-panel` → `entity-item`. Correct on all four; **3 files, two `core`, a new public prop.**
- **M2** — the withdrawn skin rule. Correct on **1 of 4**.
- ✅ **M3 — the ellipsis lives in the STRING, written by the widget, plus one line in `deriveInitials` stripping LEADING non-letter/non-digit characters.** ① Correct on **all four** by construction; avatar reads **`7W`**. ② **2 files, 2 lines, no new prop.**

🔑 **M1 threads a new public prop through two `core` components to tell the skin something the widget already knew.** M3 puts the decision where the knowledge is. ⚠️ *Chat reached for M1 first because the last thing shipped through that path was a prop — pattern-matching, not reasoning; recorded because the record should show why the cheaper answer arrived second.*

⚠️ **HONEST COST OF M3:** the ellipsis becomes part of `descriptor.name`, so every downstream reader sees it — today the `entity-item` label and the avatar, **both accounted for** — and `entity-avatar` is `core`, so this still touches `core`, with one line instead of a new prop.

⚠️ **HONEST LIMIT ON (B):** it puts a second ellipsis mechanism next to `text-overflow: ellipsis` in the same rule block. §6's gate exists for that. 📌 *Moot under M3 — no skin rule is added at all — but the DOUBLE-ELLIPSIS gate (§6 V2) survives unchanged, because `text-overflow` can still fire on the new string if the column is narrow.*

---

## §4 — 🔓 OPEN, AND JOE'S: does the scope stop at the members panel?

Runbook §8 Finding 5 recorded **three inconsistent truncations of one identity rendering simultaneously.** Measured:

| surface | producer | today |
|---|---|---|
| member row | `tail()` `members-panel.svelte:30` | `ed25519:<43>` clipped |
| **self row** | **the same `tail()`, `:82`** | same |
| feed entry | `shortId()` `stream/derive.ts:33` — `id.slice(-6)` | `Bk9glk` |
| avatar | `deriveInitials` `entity-avatar.svelte:83` | `ED` |

**S1 — members panel only** (both `tail()` call sites; self comes along by construction). ① *User-visible:* the row and the self panel agree; **the feed still says `Bk9glk` for the same person the roster calls `…7WGuWOqU`.** ② *Resource:* one function.
**S2 — unify with the feed** — one shared helper, `shortId` retired. ① *User-visible:* one identity reads the same everywhere. ② *Resource:* a second file, a **shared module decision** (where does it live?), plus `derive.test.ts` — which **asserts `shortId(BOB) === 'b12345'` and would go RED**, so the test moves with it.

**🔓 CHAT RECOMMENDS S1**, and files S2 rather than folding it in: *"its own milestone, never a rider"* is a standing refusal here, and S2 carries a module-location question that is architecture and therefore Joe's. ⚠️ **But S1 leaves the inconsistency Finding 5 named half-closed, and the record must say so rather than reading as a fix.**

🔒 **LOCKED (Joe, 2026-08-05): A1 + A2 ONLY — *"keep a1 and a2, with possibility of widen to other cases, if i see some in the future."*** ⚠️ **AND THE SCOPE QUESTION WAS RESOLVED ONE LEVEL UP, WHICH IS WHY THIS § IS SHORTER THAN IT LOOKS:** Joe narrowed the subject three times — *tail() instead of raw long pubkey* → *when a long raw pubkey is used AS A NAME* → *the name of an identity (member/user), displayed in the UI, not in code or transferred data.* ⇒ **the selection is by WHAT THE STRING STANDS IN FOR, not by which function produced it**, and that cuts away the feed (`shortId` is already short), the `.rs` `short_id` family (already short, and serving rooms/events/nodes), and the avatar (initials, not a key).

🔒 **THE FORM IS NOW A STANDING RULE, `D-142`, NOT THIS MILESTONE'S SCOPE.** Widening needs **no new decision** — Joe points at a surface and the rule already says what to render. ⇒ **this Phase-0 no longer carries S2/S4; `D-142`'s own table carries every known surface with its reason, so *"widen later"* is never read as *"no other cases exist"*.**

🛑 **WHAT A1+A2 ALONE LEAVES ON SCREEN, STATED SO THE CLOSE CANNOT OVERSTATE ITSELF:** the roster shows `…gMpQaXrB` while **every ungrouped message row still shows the full 65-character XGID** (`D-142` A4, LIVE). *The same person, two forms, in one window at one time.* ⇒ 🔒 **the close records *"`D-142` applied at the roster"*, never *"`D-142` applied"*.**

---

## §5 — CHAT'S, RECORDED NOT ROUTED (`D-123`)

- **R-T1 — the sampler fixture and its comment travel with the change.** `app_sampler.svelte:290-300` states *"⚠️ TAIL LENGTH IS LOAD-BEARING … a short fixture tail would not clip, and would show a more legible string than the product can produce."* 🛑 **After this milestone that premise is FALSE — the product's own string is short and does not clip.** The fixture at `:303` hardcodes a 43-char tail and becomes a fixture for behaviour that no longer exists. ⇒ **fixture and comment change in the same commit**, or `N-109`'s defect repeats: a comment that was true when written and is load-bearing for the next reader.
- **R-T2 — the change is to `tail()`'s BODY, not its call sites.** Both sites keep calling it; neither needs to know the form changed. 🛑 **SUPERSEDED AT v1.2: THE HELPER IS RENAMED `tail8` AND BOTH CALL SITES UPDATED.** A function named `tail` that returns `…gMpQaXrB` is false at its own name, and *a token carrying two meanings* is the defect class this arc has hit four times. Both call sites are in the same file; the diff cost is two words.
- **R-T3 — no new exported helper in this leg.** A shared module is S2's decision (§4) and minting one here would pre-empt it.
- **R-T4 — `svelte-check` is re-measured BEFORE the first edit.** The `0/34/15` on record is inherited from J-673 and is not a baseline until re-driven.
- **R-T5 — `cargo` is NOT run and that is stated, not skipped.** Zero `.rs` by scope.

---

## §6 — THE GATES, AND ONE OF THEM MUST BE ABLE TO FAIL

| # | gate | passes when |
|---|---|---|
| **G1** | the rendered row string | exactly `…` + 8 characters, `E2 80 A6` verified **at the byte level**, not by eye |
| **G2** | **the double-ellipsis gate** | `scrollWidth <= clientWidth` on `.ei-name` ⇒ `text-overflow` never fires ⇒ no `…7WGuWOq…`. ⚠️ **Measured at the real panel width, and the width is RECORDED** — *"cannot happen at today's width"* is a claim about today's desk, not the product |
| **G3** | the avatar initials | whatever §3 locks, read from the live DOM — **`…7` is the failure this gate exists to catch** |
| **G4** | **the self panel** (`:82`) | rendered and read, because §1 shows this leg reaches it |
| **G5** | the control | a **resolved** row in the same eval, unchanged — *a probe returning the same answer for differentiated inputs cannot fail* |
| **G6** | floors | `svelte-check` re-measured before and after, Δ explained · catalogue **435** · `cargo` not run, stated |

🛑 **G2 IS THE ONE THAT CAN FAIL, AND IT IS THE REASON IT IS WRITTEN AT THE LAYOUT LAYER RATHER THAN THE STRING LAYER** (`D-140`): the claim *"the ellipsis is not doubled"* lives in **layout**, and a string-length assertion cannot decide it.

---

## §7 — LEGS

- **T-0** — this document. §3 and §4 are Joe's; nothing else is open.
- **T-1** — the runbook, written from the LOCKED §§ only. One function, the sampler fixture + comment, the CDP witness with its control.
- **T-2** — **Clair implements.** `svelte-check` only; zero `.rs`.
- **T-3** — Chat re-drives every gate **on the committed tree**, records, close.

---

## §8 — DoD

- [ ] §3 the avatar collision locked by Joe — ✅ **LOCKED (B), 2026-08-05**
- [ ] §4 the scope locked by Joe — ✅ **LOCKED A1+A2, 2026-08-05; the form generalised to `D-142`**
- [ ] `svelte-check` baseline **re-measured before the first edit**, not inherited
- [ ] G1–G6 driven on the committed tree; G2 records the panel width it was measured at
- [ ] the sampler fixture **and** its `:290-300` comment changed together
- [ ] `M_RP_MEMBERS.md` §6a marked discharged, with the date and the commit
- [ ] Records: JOURNAL + `CLAUDE.md` PLAY + ROADMAP + `M_RP_MEMBERS.md` + this doc in one commit (`D-074`)

---

## §9 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

1. **§2's grapheme trace is read, not run.** `graphemes("…7WGuWOqU").slice(0,2)` is asserted from the source of `deriveInitials`; it has **not been executed**. *If the grapheme splitter treats U+2026 unexpectedly the initial may differ — the finding (punctuation reaches the avatar) survives either way, but the exact two characters are unverified.*
2. **The 8 in "tail-8" is J-588's number and is not re-derived here.** It was locked as *"the cheapest family"*; nothing in this document re-prices whether 8 is the right count.
3. **§4's table claims these are the only truncations.** Corpus at v1.0: `ui/**` for `tail(`, `slice(-N)`, `shortId`. ✅ **CLOSED AT v1.1 (`D-139`): `.rs` WAS then searched**, and it changed the answer — three more producers surfaced (`xgen-client/src/app.rs:4652`, `xgen-node/src/app.rs:4662`, `ai_behavior.rs:136`), one of which contradicted `D-126`'s own grounding. ⚠️ **Still not searched: any surface outside `ui/**` and `*.rs`** — no docs, no scripts, no test fixtures beyond the sampler.
4. **No claim is made that this closes Finding 5.** Under S1 it closes one of three surfaces, and §4 says so explicitly rather than letting the close read as a fix.
5. **§2's `deriveInitials` trace is now moot for the shipped path but not for the record.** Under the locked (B) the component returns bare `7WGuWOqU`, so initials are `7W` and the `…7` case never arises — **but §2's reading of `deriveInitials` (name path first, xgid fallback never reached from this surface) is still asserted from source and not executed.** G3 measures it.
6. 🛑 **`message.svelte:159`'s Label WAS verified after v1.0 and it is unconditional within `{#if !grouped}`** — so A4 is **LIVE**, not latent, and every ungrouped message row renders a 65-character XGID where a name goes. *v1.0 listed this as an unverified doubt; it is now a measured fact and lives in `D-142`'s table. It is NOT this milestone's to fix.*
