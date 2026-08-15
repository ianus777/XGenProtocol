# M-RP-MEMBER-ACT — Leg E: the DM home + the R1 filter — Phase-0
> **Status**: COMPLETED  
> Version: 1.6  
> Date: Aug 2026  
> **Last updated**: 2026-08-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS IS, AND THE SENTENCE THAT SIZES IT

Leg E is **`M-RP-MEMBER-ACT`'s last leg**. Closing it closes the milestone. It is a **leg, not a milestone** — J-717 cost a commit to fix exactly that slip.

**One sentence:** *DM Spaces leave R1 (OQ3 = A3, ruled by Joe at J-709), and they need somewhere to live before they lose the tree.*

**No code. No runbook. This is Phase 0 of D-071's four phases.**

---

## §1 — STATE AT OPEN, RE-MEASURED

| item | measured |
|---|---|
| tree | **CLEAN** (`git status --short` empty) |
| `HEAD` | `73b5302daccf026124f7d4165ef93f1ed8f7b7f6` |
| `git ls-remote origin refs/heads/main` | `73b5302…` — **identical**, not the tracking ref |
| latest record | J-717 · ROADMAP v7.04 · notes v1.21 (to N-191) |
| gate | Leg E's `E → C-bis-6` gate **DISCHARGED since J-711** |

**Floors, stated rather than re-run** (this document is reads only; zero `.rs`, zero `ui/**`): `cargo` **1597 / 0 / 62 × 56** · `svelte-check` **0 / 34 / 15** · catalogue **435**.

🛑 **NO REGISTRY NUMBER IS CARRIED.** N-184 (Space-dependent) and N-190 (draft-dependent) between them mean a count is meaningful only against **the same client, the same Space tree, the same draft state**. The kickoff's `168 at seven Spaces` is recorded as **a screen**, not a floor. **Record the screen, or record no number.**

📌 **Apps are DOWN.** Nothing in this document was measured live; every finding below is a read of the tree at `73b5302`, and each one names its file and line so it can be falsified.

---

## §2 — THE AUDIT (grounded at `73b5302`, not recalled)

| surface | file | what it actually does |
|---|---|---|
| **R1 render** | `spaces-panel.svelte:50` | `items = spaces.map(toDescriptor)` — **every** Space, unfiltered |
| **R1 highlight** | `spaces-panel.svelte:58-63` | `selected` `$derived.by` — resolves the latch, returns `undefined` when `s?.counterpart != null` |
| **R2 scope** | `rooms-panel.svelte:30` | `spaceLatch.scopedSpace?.rooms` |
| **R2 highlight** | `rooms-panel.svelte:42` | `roomLatch.effectiveRoomId` |
| **the space latch** | `space-latch.svelte.ts:76` | **three** writers: `note()` (bus-fed, `kind==='space'` only), `clear()`, and **`latch()` — added at C-bis-6** |
| **the store** | `spaces-state.svelte.ts:34` | `KnownSpace` carries `counterpart: string \| null` — **it already ships** |
| **member activation** | `members-panel.svelte:244-300` | `findDm` (`:220`) → latch room **and** space (`:267-268`), `dmDraft.close()`, bus write; **`if (identityId === selfId) return;` at `:249`** |
| **region machinery** | `layout-default.ts:64` | `buildWidgetRegistry` accepts a `regionId` **outside `REGION_IDS`** |
| **install/leaf** | `app_client.svelte:516-527` · `mutate.ts:266` | `insertLeaf` at a defined target — the `connection-stats` precedent, live |

⚠️ **EVERY POINTER IN THIS TABLE WAS RE-MEASURED 2026-08-12 AFTER CLAIR'S `W1`** — v1.0's were **estimated from a whole-file read rather than measured**, and seven were wrong by 3–8 lines. *A file:line asserted without being measured is the `N-180` species at citation scale; a runbook lifting it sends the implementer to the wrong line.* 🔒 **RULE FROM HERE: a `file:line` enters a record only from a tool that printed it.**

**Three readers of `spacesState.spaces`, re-confirmed** (F-D's premise holds): `spaces-panel:50` · `rooms-panel:30` · `room-latch.svelte.ts` `resolveLatched`, from which `canSend` derives. 🔒 **A3's filter lives in `spaces-panel`'s `$derived` and nowhere else.**

---

## §3 — FINDINGS THAT MOVE THE PLAN

### ✅ F11 — CLAIR'S ADVERSARIAL READ RAN (2026-08-12). THREE PLAN-MOVING, ONE WORDING. **ALL RE-DRIVEN BY CHAT, NONE ADOPTED ON REPORT (Rule 5).**
Brief: `tasks/CLAIR_LEG_E_PHASE0_READ.md` v1.0. Her verdict: **the design is sound** — `F5`, `F4`, `F6` and the `E-1`/`E-3` ordering lock all survived, and she could not break `T2`.

| # | finding | Chat's re-drive |
| --- | --- | --- |
| **P1** | **`E-0` was already discharged on disk** while §5 listed it as a future leg | ✅ confirmed — all six annotations + `N-192` present. 📌 ***And it is the shape `F1` flags against the ROADMAP, committed by this document in the same commit*** |
| **P2** | **the re-inject hook is unspecified, and `onMount` placement re-strands the home via File ▸ Revert** | ✅ confirmed — two `loadLayout()` callers (`app_client.svelte:709` boot, `:586` `handleRevertUi`), **neither persists**; `layout.revert` is a LIVE command ⇒ **reachable, not theoretical.** ⇒ the hook lives **inside `loadLayout()`** |
| **P3** | **the default position lands in two code sites that can drift** | ✅ confirmed, and **resolved by `①`-B (Joe)**: the home is **left out of `DEFAULT_LAYOUT` entirely**, so re-inject is the ONLY placement path — drift structurally impossible, and no future system region needs `DEFAULT_LAYOUT` edited |
| **W1** | **seven `file:line` pointers wrong by 3–8 lines** | ✅ **all seven confirmed wrong.** No code moved `73b5302..HEAD`, so they were wrong when written — **estimated from a whole-file read instead of measured.** Swept at v1.3 |

🔑 **AND SHE DISSOLVED §8 ITEM 2's OWN FEAR (`C3`):** the "large migrate" does not exist — `migrateLayout` short-circuits (`resolve.ts:161`), and `insertLeaf` is **idempotent by construction** (`mutate.ts:266` — *"already docked → no-op (guards double-install)"*, plus a target-missing no-op that cannot bite on a non-removable system target). **Only the hook survives.** ⚠️ *So §8 item 2 was right that the pricing was wrong, and wrong about the direction — it predicted "larger", and the migrate half was smaller while the wiring half was larger.*

🛑 **THE STANDING LESSON, AND IT IS CHAT'S: `W1` IS NOT A TYPO CLASS.** A `file:line` asserted from reading rather than from a tool that printed it is the `N-180` species at citation scale — and a runbook lifting it verbatim sends the implementer to the wrong line. 🔒 **RULE: a `file:line` enters a record only from a tool that printed it.**

⚠️ **AND THE PATTERN HOLDS FOR THE NINTH TIME THIS ARC: every defect in this document came from OUTSIDE it** — four from Joe's recall, four from Clair executing it. **Chat's own re-reads passed every time.**

### 🛑 F10 — `D-121` HAS HAD **THREE** LENSES SINCE 2026-07-26, AND THE MIDDLE ONE IS AUTH TIERS. `CLAUDE.md`'s SIGNPOST STILL SAYS TWO.
`DECISIONS.md:4504` — *"Every question and recommendation is examined through **three** named lenses first: user-visible impact, then **tier consequence**, then resource cost"*, **Amended: 2026-07-26 (lens 3 added)**, minted on J-591's evidence: Joe asked for the four DM-hosting options to be asserted **against T4**, and *"assertion against t4 is fundamental … among user view and resource drain"*.

🛑 **`CLAUDE.md:11` STILL READS *"LEADS WITH TWO NAMED LENSES (Joe, locked 2026-07-19)"* AND NAMES ONLY ① user-visible ② resource cost.** It predates the amendment by a week and was never swept. ⇒ **the session kickoff said two, and §4 of this document as first written said two.** *The head every session reads first is the one carrying the stale copy — `N-109`'s shape at convention scale.* 📌 **`D-131` annotation owed at `CLAUDE.md:11`; `DECISIONS.md` is correct and is not touched.**

**The four questions lens ② asks** (`DECISIONS.md:4518-4526`): does crypto-shred remain real · does a T4 durability floor survive · whose tier governs, deliberately or by accident · is one party's erasure-fate silently imposed on another.

✅ **APPLIED TO ①, THE HONEST ANSWER IS *NO TIER CONSEQUENCE* — AND `D-121` SAYS THAT IS A LEGAL ANSWER** (*"most questions — tooling, probes, records, **widget layout** — have none"*). H1, H2 and H3 are render surfaces over `spacesState`; none moves a byte, creates a copy, or decides whose tier governs. 🔑 **BUT THE LENS WAS NOT IDLE — IT PRODUCED CONSTRAINT 4.** Question 4 is what forbids a home-local persisted list: *a client-side record of who you talk to is a second copy with its own erasure fate.* ⇒ **lens ② does not discriminate between the options; it constrains what any of them may become.** *Stated plainly rather than manufactured — a fabricated tier rationale is as bad as a fabricated UX one.*

### 🛑 F9 — J-709 RECORDED A JOE LOCK ON THIS EXACT SURFACE AND THIS DOCUMENT'S ① OMITTED IT; ITS GROUNDING IS ALSO TOO WIDE
J-709 carries, under a 🔒: *"the dm home has to be where we can save it along t4 requirements, archived"* — **Joe's words, about this leg's surface.** ⚠️ **① as first written carried no trace of it.**

🔑 **AND THE TWO THREADS ARE ONE CONVERSATION, WHICH IS WHY F2's WELD HAPPENED.** `N-173` was born at **J-701** *"while grounding an auth-tier-gated render proposal"* — the same DM-entry design conversation. **The auth-tier material and the DM-home material were in the room together**, and four entries later J-713 attached *"the rename is Joe's"* to the wrong rename. *The collision was not random; it was topical, which is exactly what makes this class of error survive a re-read.*

🛑 **J-709's GROUNDING IS WIDER THAN THE SOURCE — THE ARC'S OWN SPECIES, RE-MEASURED HERE.** It states *"there is no written T4 archival requirement."* **There is a written per-tier retention table**: `docs/xgen_appendix_d_en.md` §6.2 — **Tier 4 = 10 years minimum for healthcare (HDS, SGB V); government jurisdiction-defined**; Tier 3 = 7 years (SOX §802). ✅ **But it governs the NODE's protocol audit log** (`audit/protocol_audit_YYYY-MM.jsonl`, §6.1) and the Auth Module's own log (§6.3) — **not DM content.** Message retention is `§:217`: *no built-in automatic deletion in Phase 1, operator's responsibility.*

✅ **`TIER4_TTL_DAYS = 90` is right as recorded** — `xgen-core/src/auth/tiers.rs:24`, returned by `AuthTier::ttl_days()`, WD-09/10/11: **the attestation re-verification interval, not a retention floor.**

⇒ **the corrected statement:** *there is no written T4 retention requirement for DM CONTENT; T4 minimums exist and govern AUDIT LOGS; message retention is Phase-1 operator responsibility with no automatic deletion.* 📌 **`D-131` annotation owed at `JOURNAL.md` J-709 — a pushed record, annotated at its site, never repaired.**

### 🛑 F1 — `K2` IS ALREADY DISCHARGED, AND THE ROADMAP CONTRADICTS ITS OWN PHASE-0
`KnownSpace.counterpart: Option<String>` ships at `xgen-common/src/state.rs:198` with `#[serde(default)]` and the K3 backfill (`xgen-client/src/ops.rs:88-89`), mirrored at `spaces-state.svelte.ts:34`. §6's Leg E row says so in its own words — *"OQ8-K2 no longer lives here — K3 took the field into Leg B."* **`ROADMAP.md:321`'s `Owes:` still carries it as open**, and this session's kickoff inherited the ROADMAP rather than the Phase-0. ⇒ **open item ④ is closed before it is asked.**

### 🛑 F2 — `N-173` IS MISCITED, AND THE ORIGIN IS ESTABLISHED RATHER THAN ASSUMED: IT IS CHAT'S, MINTED AT J-713
`N-173` (`ui/docs/xgen-ui-notes.md:3548`) is *"Tier-1 / Tier-2 already means two unrelated things"* — `AuthTier` versus the processor provenance axis. **It is not the DM-row rename.** J-709's ruling table (*"N-173 — context supplied; the rename stays Joe's, no action"*) is about the **tier** rename.

🔑 **PROVENANCE, BY `git log -S` AND NOT BY MEMORY: ONE LINE, ONE COMMIT.** `8daf712` (2026-08-11, J-713) added to ROADMAP's on-screen table: *"the rename is filed as `N-173` and is **Joe's**, not a leg's"*. **That is the whole origin.** It spread to `ROADMAP.md:321` when J-717 minted the Leg E node, and into the **session handoff prose** — which is where this session's kickoff item ③ came from. 🛑 **The kickoff did not inherit the ROADMAP; it inherited THIS SEAT.**

🔑 **THE MECHANISM IS THE ONE `J-676` ALREADY NAMED, RE-COMMITTED BY THE SAME SEAT WITH THE CORRECTION ARC ALREADY IN THE RECORD: TWO SENSES OF ONE TOKEN, WELDED BY A CITATION.** There it was `T3` → `D-126`, where *"no word"* meant two different things. Here it is ***"the rename is Joe's"*** meaning **two different renames, four entries apart**. ⚠️ **`D-139` — a claim states the corpus it searched — was written for exactly this, and was not applied to a citation this seat was AUTHORING rather than reading.**

⇒ the DM-row label question is **real, Joe's, and carries no designation at all**. `D-131` annotations owed at **`ROADMAP.md:321`** and **`:455`**, each naming `8daf712` as origin; the eighteen-versus-seventeen lesson applies — **annotate, never search-and-replace**. 📌 **And the substance needs a home:** filed as **`N-192`** (next free; the file runs to `N-191`), so the annotations **RE-POINT rather than merely delete** — *a question with no designation is invisible to every future search, which is `D-139` from the other side.*

### ✅ F3 — J-694's SPACE-NEVER-CLICKED CASE IS ALREADY TAKEN BY C-bis-6
`members-panel.svelte:267-268` calls `roomLatch.latch(dm.room_id)` **and** `spaceLatch.latch(dm.space_id)`. Member activation is the only path that enters a room without clicking its Space, so R2 can no longer list the previous Space's rooms. J-711's gate ① measured it (`R2 count: 1`, row is `dm`). ⇒ **remove it from Leg E's `Owes:`; do not re-verify it as Leg E work.**

### 🛑 F4 — CHAT'S `G13` DISSOLUTION IS HALF WRONG, AND THE FAILING HALF IS THE ONE LEG E RESTS ON
The J-709 proposal: *`counterpart != null` answers the render question; the `dm_space_create` root event answers provenance.* Measured:

- `SpaceState.is_dm` (`xgen-core/src/space/state.rs:194`) is set at construction and **never reassigned** — `is_dm =` returns **zero** hits repo-wide. ✅ The provenance half is right.
- `apply_dm_promote` (`xgen-core/src/space/state.rs:659-665`) sets `self.name` and `self.dm_constraints_active = false` — **it does not touch `is_dm`.** The mutable *"is currently a DM"* fact **already exists in Rust and is named `dm_constraints_active`** (`state.rs:239`). It is **not** in `KnownSpace`.
- 🛑 **Nothing anywhere ever clears `counterpart`.** No client promote path exists. ⇒ `counterpart != null` inherits **exactly** the staleness A2 was rejected for — *"a DM promoted to a real Space stays hidden from the tree forever."*

🔑 **The dissolution did not escape G13; it renamed the field the staleness lives in.** ⚠️ **Unreachable today, and honestly so:** the client's `KnownSpace` tree is written locally at create/join and no promotion reaches it at all — the *name* would be stale too. This is a gap Leg E must not make load-bearing, not a defect Leg E creates. *"Unreachable today" has been the wrong argument five times in this project; it is offered here with its own trigger attached (§4②), not as a dismissal.*

### 🛑 F5 — THE FILTER STRANDS THE SELF THREAD, AND EVERY OTHER DOOR IS ALREADY SHUT
`KnownSpace.counterpart` holds the **session identity** for the self thread (`state.rs:192`, `spaces-state.svelte.ts:32-34`) ⇒ **A3's filter hides the self thread from R1.** And nothing else reaches it:

- `members-panel.svelte:249` — `if (identityId === selfId) return;`, *"self is never a DM target NOR a draft target"*. **A hard no-op.**
- `OQ6-E2` deleted the `self_open` Tauri command. `self_open` is reachable from `app.rs:2835` · `batch.rs:445` · `aicontrol.rs:451` — **CLI only. There is no desktop command.**

⇒ **after Leg E lands as specified, the self thread has no entry point in the product.** 📌 **And `OQ6-E2` is UNBUILT:** E2 was adopted as *"the self row takes the same path as any peer"*; the shipped code deliberately refuses it, with a comment saying so. **Either the DM home lists the self thread, or A3 strands it.** No record carries this.

### 📌 F6 — WHAT C-bis-6'S `F-D` MINIATURE ACTUALLY SHIPPED, READ RATHER THAN QUOTED
It is a **highlight suppression inside `spaces-panel`'s `selected` `$derived`** — a DM latch yields `undefined`. **The row still renders.** A3 changes **`items`**, which removes rows, which moves the entity-panel row count and therefore the registry. ⇒ the miniature proves **the seam** (a `$derived` reading `counterpart` in the right file) and **nothing about the row-removal path or its registry delta.** The kickoff's *"the seam is proven, the surface is not"* is correct; this is its precise version.

### ✅ F7 — THE HOME IS CHEAPER THAN `A3` PRICED IT
A3 called it *"a new region/surface — the largest piece in this milestone by a wide margin."* Grounded, the machinery ships: `buildWidgetRegistry` accepts an unknown `regionId`; `insertLeaf`/`removeRegion` are live D-119 primitives; `connection-stats` is a working runtime region-plugin precedent; `EntityDescriptor.flags.isDm` exists in `core` and `entity-avatar.svelte:59` **already draws the circle for it** — and `spaces-panel.toDescriptor` never sets it. **A DM home is a widget, a descriptor row and a leaf — not new machinery.**

### 🔒 F8 — LEG E'S FLOOR IS WRONG IN §6, AND IT FOLLOWS FROM F1
§6 lists Leg E's floor as **cargo + svelte-check**, because K2 was expected to land here. K2 landed in Leg B. Under §4's recommendations Leg E touches **zero `.rs`** ⇒ **the cargo floor does not return; a `cargo` re-run would be a scope argument, not a measurement.** Floor = **svelte-check**, plus catalogue **iff** `ui/core` is opened (it should not be).

---

## §4 — OPEN, AND JOE'S. Each carries `D-121`'s **THREE** lenses (`F10`): ① user-visible impact per option → ② tier consequence → ③ resource cost.

📌 **Lens ② for every item below is *NO TIER CONSEQUENCE*, stated once here rather than manufactured three times** — all four items are render surfaces and records; none moves a byte, creates a copy or decides whose tier governs. **Its one live output is `①`'s constraint 4.**

### 🔒 ① — **CLOSED 2026-08-12: H1, THE NINTH REGION. PROVENANCE DELEGATED** (*"all by your recomms"*, `D-141`).

🛑 **ANNOTATION AT THE SITE (`D-131`, 2026-08-12, post-ruling grounding): THE *"`v3 → v4` MIGRATE"* HALF OF THIS RULING IS SUPERSEDED. THE RULING ON H1 STANDS; ONLY ITS MECHANISM CHANGES.** It was recommended **without reading `resolve.ts` or `mutate.ts`** — the pricing-before-measuring species §8 item 2 predicted about this very item. Measured since: `migrateLayout` short-circuits at **`if (l.version >= 3) return raw as Layout;`** (`resolve.ts:161`), so a bump is **a one-off that solves this widget only and mints another bump for every future system region**. 🔑 **AND THE GENERAL RULE ALREADY EXISTS IN WRITING AND NOT IN CODE:** `D-114`'s layout rules read *"widgetId durable · drop unknown · **re-inject `system`** · version + migrate"* — **drop unknown is BUILT** (`resolve.ts:11`, reported in `dropped`), **re-inject `system` is NOT** (the only `re-inject` in the tree is `app_client.svelte:548` / `installed.svelte.ts:99`, both the enable-a-disabled-CUSTOM-plugin path). ⇒ ***`M-RP7.1b`'s `migrate` finding again — a path described in the records and implemented nowhere is not a path, it is a plan.*** ✅ **AND THE AMBIGUITY THAT WOULD MAKE RE-INJECTION DANGEROUS DOES NOT EXIST:** a user cannot remove a system region (`app_client.svelte:554` guards `desc.kind !== 'custom'`; W-13 makes system widgets non-removable) ⇒ **a system `regionId` absent from a saved layout can ONLY mean "saved before that region existed"**, so re-inject is unambiguous rather than a guess about intent. ⇒ **`E-2` BECOMES: BUILD `D-114` §9's RE-INJECT RULE** — no version bump, no schema change, idempotent, and every future system region gets it free.

📌 **Recorded as a DELEGATION, not as Joe deriving H1.** `D-141` exists because a one-word approval is easy to over-extend into a ruling its author never made. 🔓 **What stays Joe's and is NOT settled by this: the tile's default position, its size, its row form, and where the self thread sits in the list** — appearance, untouched.

### 🔓 ① WHERE DO DMs LIVE? — the home surface (architecture **and** appearance, both reserved)

🔒 **THE BINDING CONSTRAINTS. The first is from `F5`; the rest are Joe's own, recorded at J-709 and restored here after `F9` found them missing.**

1. **The home must list the SELF THREAD**, or the product loses it (`F5`).
2. 🔒 **The home lists EVERY DM** — not a recent-N, not favourites, not a curated subset. *Joe, J-709: "the dm home has to be where we can save it along t4 requirements, archived."*
3. 🔒 **The R1 filter HIDES a row; it never removes access.** A DM stays fully resolvable through `spacesState` — which is also `F-D`'s mechanical requirement, arrived at from the other direction.
4. 🔒 **THE HOME IS A VIEW, NEVER A STORE.** It derives from `spacesState` and persists nothing of its own. Joe's own lock (J-598): *"the client is just reader-sender, doesn't hold any users data"*; and per J-709 archival custody is the **node's**, with an Auth Module owning at most the **assertion**. ⇒ no home-local list, no pinned set, no recent cache.
5. 🛑 **AND IT SHIPS NO ARCHIVAL CONTROL.** No export, no retain, no retention setting. `§5.7`'s census governs — *no control ships whose verb does not exist* — and none of these verbs exist on any layer. **T4 compatibility is satisfied by constraints 2–4, which cost nothing, not by a control.**

🔑 **WHY THIS SETTLES MORE THAN IT LOOKS: constraints 2–4 are satisfied identically by H1 and H2, so the tier material does NOT decide the surface** — it decides what the surface may not become. *A constraint that rules out a shortcut nobody has proposed yet is worth writing down before someone proposes it.*

**H1 — a ninth region: a `Direct messages` tile, a system region plugin.**
① The full Discord shape: a permanent surface listing every DM plus the self thread, dockable, foldable, drag-placeable like any tile. ⚠️ **The cost is not the tile, it is the persisted layouts:** a saved v3 layout has eight leaves and no slot for a ninth, so **anyone with a saved workspace would not see the home at all** — the exact stranding A3 exists to prevent.
② One widget + one `CLIENT_PLUGINS` row + `REGION_IDS`/`REGION_NAMES` + a `DEFAULT_LAYOUT` leaf + ~~a `v3 → v4` migrate that injects the leaf into persisted layouts~~ — 🛑 **SUPERSEDED, see ①'s annotation: it is `D-114` §9's RE-INJECT rule, not a version bump.** Zero Rust either way. 📌 The migrate is not new machinery — `migrateLayout` was built and **exercised live** at M-RP7.1b, read-path, idempotent, non-destructive.

**H2 — a second section inside R1: the Space tree above, a `Direct messages` group below.**
① Visible immediately to every existing layout, no migration, no stranding. 🛑 **But it does not honour the ruling as worded.** OQ3 asked *"do DM Spaces leave the **Spaces panel**?"* and Joe answered yes; H2 keeps them in that panel and merely regroups them. It also gives R1's suppression rule a second meaning, in the one `$derived` this leg is supposed to keep simple.
② Smallest by a wide margin: one widget edited, no layout work, no migrate.

**H3 — the home is `M-RP-PEOPLE`** (a people panel over the address book).
① The best long-term shape — everyone you know, DM or not. ② Different feeder (`get_address_book`), filed and unscheduled, **and it does not answer this question**: `members-panel.svelte:219` already records that *a book entry is not proof a DM exists*. ⇒ **rejected as Leg E's answer**, and named so E's home does not pre-empt it.

📌 **Chat's recommendation: H1, with the leaf-injecting mechanism as a named part of the leg, not a footnote** — 🛑 **and that mechanism is `D-114` §9's RE-INJECT rule, NOT the `v3 → v4` migrate written here before `resolve.ts` was read (see ①'s annotation).** It is the only option that honours the ruling, the machinery is precedented, and the migrate is the one piece that would otherwise ship the home to a user who never sees it. **H2 is the honest cheap fallback and its conflict with the ruling is stated rather than smoothed.** 🔓 **Appearance — the tile's default position, its size, its row form, the self thread's place in the list — is Joe's and is not proposed here.**

### 🔒 ② — **CLOSED 2026-08-12: G-c. PROVENANCE DELEGATED** (`D-141`). Filter on `counterpart != null`; the promotion gap recorded with a checkable trigger — *a promote path writes to the client's `KnownSpace` tree* — and `dm_constraints_active` named as the answer then.

### 🔓 ② `G13`'s SEMANTICS — reopened by F4, on better ground than J-709 had

**G-a — filter on `counterpart != null` (the J-709 proposal, unchanged).** ① None today. ② Zero. 🛑 Makes a field that nothing ever clears load-bearing in a second place — which is what §5's own warning said not to do.
**G-b — plumb `dm_constraints_active` into `KnownSpace` and filter on that.** ① None today. ② Rust: field + writer + TS mirror + a backfill value for every existing record. 🛑 **And it buys nothing yet** — the client has no promote path, so the new field would be exactly as static as `counterpart`, at the cost of returning the cargo floor to a leg that otherwise has none (F8).
**G-c — filter on `counterpart != null`, and record the promotion gap with a checkable trigger.** ① None today. ② Zero now; the correct answer is **named in advance** — when a promotion path reaches the client, `dm_constraints_active` is the field that answers *"is this currently a DM"*, and `is_dm` never will.

📌 **Chat's recommendation: G-c**, with the trigger written as a fact and not a wish (`N-182`): *a promote path writes to the client's `KnownSpace` tree*. 🔑 **The reframing matters more than the choice:** `is_dm` is **provenance and correct as provenance**; `dm_constraints_active` is the current-state fact and **already exists**; the question was never *"what does `is_dm` mean"* but *"which of the two facts is the client missing"* — and the answer is the second one.

### 🔒 ③ — **CLOSED 2026-08-12: L2, RENDER-TIME, COPIED NOT LIFTED. PROVENANCE DELEGATED** (`D-141`). 🔓 **The fallback WORDING, and whether a row shows the name alone or name plus a discriminator, remain Joe's** — copy, not mechanism.

### 🔓 ③ THE DM-ROW LABEL — corrected off `N-173` (F2), re-filed as `N-192`, and still Joe's

R1's DM rows read `DM with xgen://pubkey/ed25519:…` today — a raw key where a name belongs, and the DM home would inherit it on **every** row.

**L1 — leave it.** ① The home's whole content is raw keys. ② Zero.
**L2 — resolve at render time** from the address book, falling back to `tail8`. ① The home reads like a contact list. ② The resolver exists — `descriptorFromId` (`members-panel.svelte:136`) — but it is **members-panel-local**; a second caller is either a copy or the lift `M-RP-PEOPLE` was named for. 📌 J-508's four-independent-impls bar applies: **this would be the second, so copy is legal and lift is premature.**
**L3 — rename the stored `KnownSpace.name` at creation.** ① Same visible result. 🛑 ② Writes a display string that goes stale the day the person changes their name — and **K3 exists precisely because a label must never be a lookup key (`D-143`)**. Cheap and wrong.

📌 **Chat's recommendation: L2, render-time, copied not lifted.** 🔓 **The wording of the fallback, and whether a DM row shows the name alone or the name plus a discriminator, is Joe's.**

### ✅ ④ `K2` — CLOSED BEFORE ASKING (F1)

`KnownSpace.counterpart` shipped in Leg B under OQ8-K3. **Nothing to fold forward, nothing to defer.** The `Owes:` line that says otherwise is corrected at its site, not deleted.

---

## §5 — PROPOSED SUB-LEGS. Order is argued; no push leaves DMs homeless.

| leg | what | floor | gated on |
|---|---|---|---|
| **E-0** | ✅ **DONE 2026-08-12, commit `6268fba` (J-718)** — the six `D-131` annotations (`ROADMAP.md:318/:321/:455` · `JOURNAL.md` J-709 · `CLAUDE.md`'s `D-121` signpost · `N-173`'s own site) and **`N-192` minted**. 🛑 **Clair must NOT re-annotate.** 📌 *This row said "future leg" while the work shipped in the same commit as this document — `P1`, and the very shape `F1` flags against the ROADMAP* | none | — |
| **E-1** | ✅ **DONE 2026-08-12 (J-721) — V1–V8 DRIVEN GREEN.** **`DM Spaces`** (Joe: *"direct messages we will have in the messages panel"* — it lists **Spaces, not streams**). 4 files: NEW `dm-spaces.svelte` · `isDmSpace` exported from `spaces-state` (the argued 4th, `§7.1`) · one `CLIENT_PLUGINS` row · `REGION_IDS`/`REGION_NAMES`. `DEFAULT_LAYOUT` untouched (①-B). Self thread pinned first — **`selfFirst: false` measured, CORRECT: Joe has no self thread (J-689), §7.2's predicted case** | svelte-check **0/34/15** re-run by Chat | ① ruled |
| **E-2** | 🔒 **`D-114` §9's RE-INJECT RULE, AND IT LIVES INSIDE `loadLayout()`** — **not at `onMount`** (`P2`). Two callers exist — boot (`app_client.svelte:709`) and `handleRevertUi` (`:586`) — **and NEITHER persists the result**, so an `onMount`-only re-inject would leave the disk autosave pre-`DM Spaces` and **File ▸ Revert would drop the home**, which is the exact stranding H1 exists to prevent. `layout.revert` is a LIVE command (M-RP7.5 Leg D), so that path is reachable, not theoretical. ⚠️ `loadLayout` takes no plugin set today and gains one. 📌 **Target `spaces`, edge `bottom`** — verified to give `[spaces, dm-spaces, self]` in Joe's live tree (sibling: the parent already runs `col`) **and** `[spaces, dm-spaces]` under `DEFAULT_LAYOUT` (wrap: parent runs `row`). **One pair, right answer in both trees** | svelte-check | E-1 |
| **E-3** | 🔒 **THE R1 FILTER** — `spaces-panel`'s `items` `$derived` drops `counterpart != null`. **NEVER the store** | svelte-check | ② ruled, **E-1 + E-2 green** |
| **E-4** | 🛑 **ABSORBED INTO `E-1` — THIS LEG HAS NO CONTENT.** `L2` (resolve the label at render, `tail8` fallback) is built in `E-1` §3.2, and `E-3` removes DM rows from R1 entirely, so no label work remains anywhere. **ID kept, never renumbered** — `E-4` is referenced by the runbook, J-718 and the ROADMAP node. 📌 *`P1`'s shape for the THIRD time — a leg describing work that is not its own — and committed by Chat AFTER Clair flagged the identical defect at `E-0`* | — | — |
| **E-5** ✅ **CLOSED J-731** | verify + records + close (`D-074`). 🛑 **THE ROW WAS AUDITED BEFORE IT WAS EXECUTED AND TWO OF ITS THREE WORDS DID NOT SURVIVE.** *verify* — `E-1`, `E-2` and `E-3` each verified themselves under Rule 5, so `E-5` was **not** a fourth pass; the word was legitimate only for **the one path no leg could test**, the `DEFAULT_LAYOUT` wrap branch, driven at `E-5.2` and **PASSED**. *close* — the MILESTONE's §6 carried its own **Leg `F`**, *"Records + close"*, which this document never cited; **`F` is absorbed into `E-5`, ID kept** (`M_RP_MEMBER_ACT_PHASE0.md` §6a). Phase-0 `tasks/M_RP_MEMBER_ACT_LEG_E5_PHASE0.md` v1.1 · read `tasks/CLAIR_LEG_E5_PHASE0_READ.md` v1.0 | — | **E-3** |

🔑 **WHY E-1 PRECEDES E-3, AND IT IS NOT TIDINESS.** A3 removes DMs from the tree; without the home, **every DM is unreachable the moment the filter lands** — and per F5 the self thread is unreachable *permanently*, because no other surface can open it. The home and the filter **ship in the same leg and in that order**, so no commit Joe pushes ever contains the filter without the home.

🛑 **E-3's verification cannot reuse a carried number.** Removing rows moves the registry (F6), and the count is Space-**and**-draft-dependent (N-184, N-190). ⇒ **measure the transition in one sitting on one client — before-filter and after-filter, same tree, same draft state — or record no number.**

⚠️ **A probe whose pass condition is "the DM rows are gone" is an EMPTY RESULT and must be positively controlled (`N-099`)**: prove the probe can see a DM row by reading one **before** the filter, in the same session.

📌 **No send is proposed anywhere in this leg.** Nothing here mints a DM or spends Joe's data. Joe's client state (**4204 B, 2026-08-10 21:08:23**) is read-only throughout.

---

## §6 — NOT TOUCHED

The wire · any protocol event · `xgen-core` · `xgen-node` · `xgen-common` · **`skin.css` (Joe's file, never folded into a Chat or Clair commit)** · `entity-item.svelte` · `entity-panel.svelte` · the DM model itself · `H4` bilateral replication · `M-RP-PEOPLE` · `M-RP-INTRO`.

⚠️ **`M-RP-INTRO`'s trigger fired at J-716 and it still has no Phase-0.** Flagged, not started — it is not Leg E's.

---

## §7 — THE SWEEP J-717 ASKED FOR, RUN AND HONEST

J-717 found Leg E referenced by three records and owning a table row **with no ROADMAP node**, and asked whether it was alone. A first pass over every `M-…` token returned 51 apparent misses — **an artifact**: ROADMAP is *reduced on close* (J-715), so closed milestones are legitimately absent, and the box-drawing characters were mojibake under the default read encoding, which broke the node test. Re-run against J-710's actual rule (**a milestone named in a `trigger:` line must have a node**), reading UTF-8:

| milestone named in a trigger | node lines |
|---|---|
| `M-RP-INTRO` | 1 |
| `M-RP-THREAD-XGID` | 2 |
| `M-RP-VIEW-MENU` | 1 |
| `M-RP-WIDGET-SETTINGS` | 1 |
| **`M-RP-SKIN`** | **0** |

🛑 **ONE HIT, AND IT IS LEG E'S EXACT SHAPE.** `M-RP-SKIN` is named as the discharger inside the `M-RP-MEMBER-ACT` trigger line (`ROADMAP.md:319`), **owns a row in the on-screen table** (`:460`), and is the named discharger for **every `PROVISIONAL` marker in the grid arc** — with **no node**. *Referenced by records, owning a table row, carrying no node: that is the J-717 defect, on a second milestone.*

🔓 **Whether J-710's rule extends from *trigger condition* to *named discharger* is Joe's** — Chat's reading is that it does, because the reader following the pointer finds nothing either way. **Not fixed here:** a new tree line risks `roadmap-format-gate.ps1` and belongs in its own careful edit, which is the same reason J-709 deferred the startup node.

---

## §8 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

1. 🛑 **NOTHING HERE WAS MEASURED LIVE.** The apps are down. Every finding is a read of the tree at `73b5302`. **F5 is the one that most deserves a live check** — that the self thread is unreachable is argued from three call sites, and Joe may have a path in muscle memory that the code does not show.
2. ⚠️ **F7's cost estimate for H1 is the kind of pricing that has been wrong three times in this arc** (OQ1, OQ6, OQ8 — *each time a cost was priced against one leg and never re-checked against the others*). The migrate is the piece most likely to be larger than stated.
3. **The self thread may not exist in Joe's tree at all.** J-689 recorded *"Joe has no self thread"*, and `self_open` is CLI-only. If it never exists, F5 is a **future** stranding rather than a present one — **which does not change the recommendation, but does change how loudly it should be argued.**
4. **This document has not been read by anyone outside its author.** ⚠️ *Seven consecutive arcs: every real defect came from Clair executing a document or Joe looking at a screen. Chat's own re-reads passed every time.* An adversarial read before any lock is worth its cost, and `M-RP-TAIL8` is the evidence.
5. **F4's "unreachable today" is the argument that has been wrong five times here** (N-091 · N-097 · N-099 · N-109 · N-116). It is stated with a trigger attached for that reason, and it should be attacked rather than accepted.
