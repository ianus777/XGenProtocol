# M-RP-IDENTITY-RESOLUTION — what a member row shows before the client knows who it is
> **Status**: ACTIVE  
> Version: 1.19  
> Date: Aug 2026  
> **Last updated**: 2026-08-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

🔒 **ID AND TITLE LOCKED (Joe, 2026-08-01, J-644).** `M-RP-IDENTITY-RESOLUTION` — *what a member row shows before the client knows who it is*. Rule 8 satisfied: the ID carries a short descriptive title everywhere it appears. 📌 **The filename already matched, so no `git mv` was required, and this document is the first thing to cite the ID.** ⚠️ *v1.0 carried this line as **ID IS PROVISIONAL — Chat's working handle**; superseded, kept not erased (`D-131`).*

---

## §0 — What this is, and what it is NOT

**This is the answer to one question: a member is in the room and the client cannot say who they are — what does the panel do?** It exists because `M-RP-LIVEFEED-REFRESH` Leg A (J-643) created a state the panel had never been in: a roster row whose identity was **never looked up at all**.

**It is NOT:** the visit card (Tier 2 — its own design pass, §2) · M13's wire widening (`revoked` · `trust_assertion` · `tier` are wire-absent, PENDING, separate) · the D-126 word form (deferred at J-588, still Joe's) · presence (layer ④, unbuilt) · the `tail-8` lock-versus-build gap (`M_RP_MEMBERS.md` §6a, filed not fixed).

**Seats (D-123).** §4 (④'s treatment) and §5 (③'s treatment + the count) are **Joe's** — appearance and taxonomy. §2's tier frame is **Joe's**, originated by him. Everything else — grounding, the capability gaps, legs, verification — is Chat's.

---

## §1 — Grounding (measured 2026-08-01 at `afd9aa9`, HEAD = origin/main, tree clean)

**G1 — the node answers `identity.get` from its LOCAL registry only.** `xgen-node/src/app.rs:3538-3573`: `rt.identity_registry.get(&identity_id_typed)` → `Record` or `NotFound`. **No proxy, no forwarding, no live federation lookup.**

**G2 — the registry is written by exactly two production paths.** `identity.register` (`app.rs:3505` upsert on re-home · `:3507` register) and **MP-F9 signer replication** (`app.rs:3900` `send_space_signers_in_session`). ⚠️ **F9-D3 is behaviour-hard: the replicated set is the distinct `ev.sender` of shared-Space history — *delta-signers, NOT current members*.**

**G3 — 🔑 A SENDER THE NODE DOES NOT KNOW CANNOT ACT AT ALL.** `xgen-core/src/message/exchange.rs:208-210`, validation step 11:

```rust
if !id_registry.contains(sender) {
    return Err(ExchangeError::UnknownSender);
}
```

⇒ **`identity.not_found` on a current roster member is not a display inconvenience — it means that member's events are being REJECTED.** They cannot post, react, or leave. **They are inert by protocol, not merely by accident.**

⚠️ **AND IT REACHES BACKWARDS: `membership.join` is signed by the JOINER.** A member whose record the node never held could not have joined — the join itself would have failed step 11. ⇒ **a `not_found` current member is one whose record WAS present and is now gone.**

📌 **THIS RETRACTS A CLAIM CHAT MADE ONE TURN EARLIER IN THE SAME CONVERSATION** — that a federated lurker who never authored an event would legitimately return `not_found`. **False:** they could not have joined. **Joe caught it with one question — *"how does Carol say hello if she is not authorized or was erased?"*** — and the retraction is recorded rather than quietly dropped (`D-131`).

**G4 — `membership.join` carries NO content.** `apply_join` (`xgen-core/src/space/state.rs:995-1030`) reads only `event.sender`, `event.room_id`, `event.timestamp`. **There is no `MembershipJoinContent` struct anywhere in the tree.**

**G5 — but the Space state yields four facts with NO identity record needed.** `identity_id · role · joined_at · invited_by`; `role` and `invited_by` come from the consumed `pending_invite` (`state.rs:1019-1021`). 📌 **`M_RP_MEMBERS.md` §7 locked all of these UNFED** — *"arrive free and are deliberately discarded"* — a **delegated** lock, re-openable at the cost of a prop.

**G6 — 🛑 `is_ai` IS THE ONE THING THAT CANNOT BE DERIVED.** It lives only in the identity record (`xgen-core/src/wire/types.rs:468-469`), and the code's own comment calls surfacing it *"the §3.6.10 **transparency requirement**"*. ⇒ **a row without a record cannot satisfy that requirement by any local means.**

**G7 — the client's book has ONE writer.** `ui/common/lib/stores/address-book.svelte.ts:131` — `_book = book ?? {}`, inside `setResult`, driven only by the fill, whose sole trigger is the `$effect` on `roomLatch.effectiveSpaceId`. **Leg A's router never touches `_book`.**

**G8 — the Tauri surface has 18 commands.** `get_address_book` and `fill_space_records` exist (M-RP-MEMBERS Leg A). ⚠️ **There is NO single-identity lookup** ⇒ any fetch-on-join is a **new command, Rust, and it moves the cargo floor** (baseline 1588 / 0 / 62 × 56).

**G9 — the stream does not consult the book.** `stream-panel.svelte:115` renders authors with **no name**, xgid-tail initials, *"same as inbound"*.

---

## §2 — 🔒 THE TWO-TIER FRAME (Joe, 2026-08-01)

🔒 **Joe's, and it is the frame the rest of this document hangs on:** *first the primary hard data that the protocol definition requires — id, type — then the visit card, for the separate reason that we want to know **who** it is.*

| | Tier 1 — the Identity record | Tier 2 — the visit card |
|---|---|---|
| Question it answers | **What must I know to interact with this identity safely?** | **Who is this, as they choose to present?** |
| Carries | `identity_id · home_node · devices · registered_at · is_ai · ai_capabilities · display_name`; **+ M13's `revoked · trust_assertion · tier`** | not designed |
| Obligation | **mandatory** — `is_ai` is a §3.6.10 transparency requirement (G6) | discretionary |
| Scope | **federation-wide, and must be** — `identity.get` works on any id, or the AI guarantee has holes | 🔓 **member-scoped is the candidate line** |

🔑 **THE FRAME MAY DISSOLVE THE VISIT CARD'S OPEN SCOPE RULE.** That verb was filed as *member-scoped, or it becomes a federation-wide identity oracle* — thesis-bearing, needing its own pass. Under the split, **the oracle risk was never about type and home node; it is about the rich presentation layer.** ⇒ *Tier 1 federation-wide, Tier 2 member-scoped* is a defensible line the tension did not previously have.

📌 **`display_name` STAYS IN TIER 1 AND DUPLICATES NOTHING.** Ch2 §User Representation (`docs/xgen_ch2_architecture.md:561-566`) defines four layers — global display name (**Overrides: Nothing**) · Space nickname · contact alias · contact note (*"supplementary, **not a name***"). **The card is not a naming layer at all.** ⚠️ *Chat first proposed that `display_name` would either move to the card or become a Tier-1 duplicate. **Both were wrong — Joe's question, "which field do you think it is duplicating with?", had no answer.** Kept as a correction, not erased.*

🔓 **TIER 2 IS NOT DESIGNED HERE.** It needs its own pass. **This milestone builds Tier 1 behaviour only.**

---

## §3 — The four states, and why the taxonomy is load-bearing

| # | Situation | Did we ask? | Node holds a record? | Can the member act? |
|---|---|---|---|---|
| ① | Record held | yes | ✅ | ✅ |
| ② | Asked, reply not back | asked, waiting | ✅ presumably | ✅ |
| ③ | Asked, `identity.not_found` | **asked, answered "nothing here"** | ❌ | 🛑 **NO — step 11 rejects every event (G3)** |
| ④ | **Never asked** | **no answer of any kind** | ✅ **probably — this says nothing about the node** | ✅ **yes, fully** |

⚠️ **v1.0/v1.1 DEFINED ④ AS *"never asked, OR asked and never heard back"*. THE SECOND HALF CANNOT OCCUR — superseded at v1.2, kept not erased (`D-131`).** `ops.rs:2913` is `identity_get_on(...).await?`: **the `?` aborts the ENTIRE fill on a transport error**, and the shell runs `setFailed`. ⇒ **inside a fill that returns `Ok`, every id is either fetched or `not_found`.** There is no per-id silent timeout. 🔑 **④ has exactly one source: a live `membership.join` that never triggers a fill at all.**

🔑 **③ AND ④ LOOK IDENTICAL AND ARE OPPOSITES.** ③ is a fact about the **identity** — the node has nothing, so they are inert. ④ is a fact about **our connection** — the member is a live participant we merely failed to name. ⚠️ **Rendering them the same is the panel reporting our network fault as someone else's irregularity.**

📌 **④ IS NOT "NEVER AUTHORIZED".** A ④ member authorized normally, on their home node. ⇒ **hiding ④ would let our socket drop delete a real person from a room they are in** — the shape §3's rule forbids: *staleness and absence both render UNKNOWN, never as fine*.

---

## §4 — 🔒 DECISION 1: WHAT ④ LOOKS LIKE — LOCKED (Joe, 2026-08-01)

🔒 **LOCKED: ④ RENDERS, DIMMED, WITH ITS OWN SELECTOR FOR MANUAL TUNING.** Joe's words: *"special visual, dimmed, its own css class for manual tuning access"*.

**The options as they stood:**

- **(A) bare id, inert, no marking.** ① *User-visible:* ④ is indistinguishable from a resolved member whose `display_name` is `None` — a real case. ② *Resource:* zero.
- **🔒 (B) a quiet visual treatment, no words.** ① *User-visible:* the row reads *not settled yet* without asserting anything about the person. ② *Resource:* one token + one state hook.
- **(C) a text label.** ① *User-visible:* explicit. ② *Resource:* opens D-126's wordlist, deferred at J-588.

🔑 **WHY NOT "IRREGULAR" — JOE RAISED IT AND WITHDREW IT HIMSELF.** ④ is our socket dying; the member is registered and active. **Labelling the row *irregular* makes the panel say something about the person when the only thing that went wrong is ours.** ⚠️ **This is the §4c precedent exactly:** *"I cannot see the others"* was chosen over *"you are offline"* because it names the **effect and never the cause**, and *"I cannot see the others online"* was rejected for asserting something the panel has no mechanism for. ⇒ **the word "irregular" stays free for M13, where `revoked` gives a real anomaly to name.**

📌 **THE SHIPPED IDIOM FOR "ITS OWN CLASS" IS A `data-*` HOOK ON THE COMPONENT ROOT**, tuned in `skin.css` as an attribute selector — `entity-item.svelte:116` `data-selected`, `entity-avatar.svelte:125/126` `data-ai` / `data-revoked`, styled at `skin.css:2382` / `:2401`. **Chat's recommendation: `data-unresolved` on `.entity-item`, tuned as `.entity-item[data-unresolved]`.** It gives a standalone selector Joe can tune without touching any other state. 🔓 **If Joe wants a literal `class=` instead, that is his call and costs the same.**

🔒 **AND THE HOOK CARRIES A VALUE, NOT A PRESENCE (J-650).** §4 locked `data-unresolved` for ④; **§5a's E2 then locked a MARK for the erased DM counterpart** — two treatments on the same element — and §5b refused `data-revoked` as the vehicle. **A presence-only attribute cannot carry two states**, so a distinguishable second hook is **required by locks already taken**, not a widening. ⇒ **`unresolved?: 'unasked' | 'erased'` → `data-unresolved="unasked"` (④) / `="erased"` (③).** ✅ **§4's locked selector `.entity-item[data-unresolved]` STILL MATCHES BOTH** — nothing locked here breaks; `[data-unresolved="erased"]` merely narrows. 🔑 **And the shared base selector is the truth, not a compromise: ③ and ④ have one fact in common — we hold no record for this person.** 🛑 **THE WORD IS `'unasked'`, NOT `'pending'` — `pending` names state ② (*asked, reply not back*, §3), which is unbuilt and will want it if §7 ships.** *Chat's first draft spent it on ④; caught before the runbook was ever committed, the same discipline that kept "irregular" free for M13.*

⚠️ **`ui/assets/skin.css` IS JOE'S.** The milestone adds the **hook**; the **values** are his.

🛑 **AND THE DIMMING DOES NOT DISCHARGE G6.** A ④ row still cannot say whether the member is an AI. **Dimming makes the row honest about being unfinished; it does not make the transparency requirement satisfied.** ⇒ **§7's fetch is what actually closes G6**, and §4 without §7 is a nicer-looking gap.

---

## §5 — 🔓 DECISION 2: ③ IS HIDDEN — JOE'S PROPOSAL, ONE THING UNRESOLVED

🔓 **Joe's rule, stated 2026-08-01:** *"not found — doesn't display at all till the next refresh."* Reasoning: *"there is no usage of such members."*

✅ **THE REASONING IS CORRECT AND STRONGER THAN IT LOOKED.** G3 makes it structural: a `not_found` member's events fail step 11. **They cannot post, react or leave. There is genuinely nothing to do with them, and nothing they can do.**

**🛑 THE ONE UNRESOLVED PIECE — THE MEMBER COUNT.** `M_RP_MEMBERS.md` §4c consequence 3 locked: *"any member count derives from the ROSTER, never from rendered rows"* — deliberately, so an offline panel cannot miscount. ⇒ **hidden ③ members are still counted, and the panel says 5 while showing 4.**

- **C1 — count the roster, accept the mismatch.** ① *User-visible:* a number that does not match the rows; **unexplained, it reads as a bug**. ② *Resource:* zero.
- **C2 — count what is shown.** ① *User-visible:* internally consistent. ⚠️ **Contradicts a lock made for a good reason** — under C2 an offline panel undercounts, which is the failure §4c was written to prevent. ② *Resource:* small, plus re-opening a lock.
- **C3 — count the roster, show the difference.** *"4 of 5"* or similar. ① *User-visible:* honest and self-explaining; **the panel says it is not showing everything.** ② *Resource:* a second number plus 🔓 **copy, which is Joe's**.

**Chat recommended C3. 🔒 JOE LOCKED C1 (2026-08-01): count the roster, accept the mismatch.**

🔑 **AND C1 IS CORRECT *BECAUSE* NOTHING RENDERS A COUNT TODAY.** `M_RP_MEMBERS.md` §7 locked `secondary` / `meta` / `status` **unfed**, and no member count ships anywhere in the panel. ⇒ **C1 and "defer it" are the SAME ACTION right now** — both mean *do nothing* — and they diverge only at the moment something renders a number. **C1 costs zero until then, and C3's second number would be built for a display that does not exist.**

🛑 **THEREFORE C1 CARRIES A TRIGGER, AND WITHOUT IT THE LOCK GOES STALE INVISIBLY.** A future milestone adds a count, the panel silently acquires a number that does not match its rows, and **nobody connects it back to this decision** — the `M-RP-REGION-GEAR` shape (work whose record has no home). ⇒ written as an obligation, not an assumption:

> **`Owes:` — the first milestone that RENDERS A MEMBER COUNT must re-open this question.** At that point C1's mismatch becomes user-visible for the first time, and C2 / C3 return as live options. ⚠️ **§4c consequence 3's lock — *"any member count derives from the ROSTER, never from rendered rows"* — stands and is NOT re-opened by C1**; C1 accepts its consequence rather than amending it.

📌 **Chat's C3 recommendation is kept, not erased (`D-131`)** — it was argued on *"the only option where the panel neither misleads nor contradicts itself"*, which remains true **the day a count ships** and is simply not yet in force.

⚠️ **ONE THING C1 DOES NOT DEFER:** ③ members must still be **hidden from the rendered list**. C1 is a ruling about the *count only*. The hiding is §5's rule and it is unaffected.

---

### §5a — 🔒 THE DM EXCEPTION: ③ IS NEVER HIDDEN WHEN IT IS THE DM COUNTERPART (Joe, locked 2026-08-01, J-648 — option E2)

🔒 **LOCKED: in a DM, the counterpart row RENDERS even when erased.** §5's hiding stands everywhere else, unchanged.

🔑 **THE RULE KEYS ON `is_dm`, NOT ON MEMBER COUNT — and that is what makes it a rule rather than a heuristic.** `is_dm` is **provenance**: written `true` once at Space creation (`xgen-core/src/space/state.rs:451`, `:567`) and **never recomputed from current membership**. ⇒ a **group** room that has shrunk to two people is **not** a DM and §5 hides there as locked; a DM stays a DM after its counterpart is erased.

**Why §5's own reasoning does not reach this case.** §5 was locked on *"there is no usage of such members"* — **an argument about group rooms**, where hiding one of twenty is a subtraction. **In a DM the counterpart is not a member of the conversation; they are the other half of it.** Three things compound:

- 🛑 **The stream still renders their messages** (G9). ⇒ **the panel says you are alone in a room whose other half is on screen one panel over.** §5's DAG-divergence was recorded as *acceptable* sized against a group room; **in a DM the divergence is 100% of the counterparty** — a different quantity, not a different degree.
- 🛑 **An erased counterpart means the DM is permanently dead.** Under §1 G3 they cannot sign an event, so they can never reply. **That is the single most useful fact about the room, and hiding the row deletes exactly it.**
- ⚠️ **A panel showing only you, in a room that exists to talk to one person, reads as a bug** — C1's mismatch shape, with no number to explain it.

🔑 **THE RULE THAT GENERALISES, stated so this is not special-pleading: hiding is legitimate when it is a SUBTRACTION. When the hidden member is the entire counterparty, hiding is not a subtraction — it is a false claim that nobody was there.**

**The options as they stood:**
- **(E1) hide uniformly, DMs included.** ① *User-visible:* the DM shows you alone while the stream shows the conversation. ② *Resource:* **zero** — §5 as already locked.
- **🔒 (E2) the DM counterpart is never hidden.** ① *User-visible:* the row persists, visibly not-a-participant; the DM reads as **ended**, not empty. ② *Resource:* one condition in the filter + a `skin.css` treatment. ✅ **NO NEW WORD REQUIRED** — a mark suffices.
- **(E3) hide the row, explain in the panel note.** ① *User-visible:* honest, but the explanation sits **below the list**, not where the missing thing was. ② *Resource:* **the largest.** 🛑 **The `NOTE` table (`members-panel.svelte:62-68`) is Joe's copy and every entry names an effect on US** — *"I am waiting" · "I cannot reach" · "I cannot see"*. **For an erased identity nothing is wrong on our side, so no honest "I…" sentence exists** ⇒ E3 requires a new sentence in a new grammar, which **re-opens D-126's wordlist** (deferred at J-588), plus a sixth panel state or a `message` that stops being a pure function of `panelState`.

⚠️ **CHAT MIS-PRICED E2 AND E3 IN THE FIRST PASS — corrected before the ruling, kept not erased (`D-131`).** Chat said E2 *"needs a word, which re-opens D-126"*. **It does not; a mark suffices. E3 is the option that cannot avoid the wordlist.** ⇒ **E2 is both the more honest AND the cheaper of the two non-trivial options**, so the ruling did not trade cost against honesty.

⚠️ **`ui/assets/skin.css` IS JOE'S.** The milestone adds the hook; **the mark, the strikethrough, the muting — all values — are his.** 📌 *Chat's sketch showed one arbitrary rendering purely so the question "should it show" could be answered by looking at a row; it is not a proposal.*

🔒 **§5a-i — LOCKED: THE ERASED DM COUNTERPART KEEPS THE L16 HIGHLIGHT (Joe, 2026-08-01, J-650).** `members-panel.svelte:146` passes `selected={counterpart}`, and `counterpart` is read from the **roster** (`:123-127`) ⇒ with B-1 keeping `_roster` complete, **the erased row is still found and still renders SELECTED.** ① *User-visible:* a marked row also carrying the L16 highlight — the mark says they are gone, the highlight says whose room this is, and the two claims do not conflict. ② *Resource:* **ZERO — KEEP is the default and costs no code.**

⚠️ **AND THE PRICE WAS STATED WRONG THE FIRST TIME.** v1.5/v1.6 read *"② Resource: one condition either way."* **The options are NOT symmetric:** KEEP is **zero lines** (the highlight already flows from the roster), DROP costs **one condition** plus a second site where ③-ness is tested. Superseded, kept not erased (`D-131`). 📌 *Caught at Leg B's grounding by reading `:123-127`, not by re-deriving this section — the recommendation was right and its cost was wrong.*

📌 **§6a's `tail-8` GAP BECOMES VISIBLE HERE FIRST.** If the erased row shows an id fallback under the name, `tail()` returns the whole final segment and `.ei-name` is **LEFT-ANCHORED** — `overflow:hidden; text-overflow:ellipsis; white-space:nowrap` (`skin.css:2452-2458`) — so it keeps the constant head and ellipsises the distinguishing tail, rendering `ed25519:AbCd…` (`M_RP_MEMBERS.md` §6a). ⚠️ *v1.0–v1.5 wrote this as "the CSS clips the **left**" — **the inverse of the truth**; it is left-ANCHORED and clips the RIGHT. A paraphrase of J-618's correct wording, inverted. Superseded, kept not erased (`D-131`).*

---

### §5b — 🛑 THE ROW SHAPE IS BELAYED, AND THE REASON IS THAT ③ IS CURRENTLY UNREACHABLE (Joe, 2026-08-01, J-649)

🔒 **Joe drew a row shape for E2 — struck display name, id beneath, own CSS class — then said: *"if this case is defined somewhere else, belay this one."* ✅ IT IS EFFECTIVELY DEFINED ELSEWHERE, AND THE CASE IS ALSO UNREACHABLE. THE ROW SHAPE IS DROPPED.**

🛑 **WHY ③ CANNOT OCCUR FOR ANYONE YOU HAVE INTERACTED WITH.** Two measured facts compose:
1. **A held identity is NEVER re-fetched** — `partition_observed` (`ops.rs:2764`) routes held ids to *touch*, not *fetch*; the doc at `:2752` says so outright.
2. **A held record is NEVER removed in production** — `remove` (`address_book.rs:253`) and `evict_older_than` (`:285`) exist and **every caller is a test**.

⇒ **once the book holds someone, it holds them forever and never asks again** ⇒ **`identity.not_found` can only fire for an identity whose record was NEVER cached.** 🔑 **For a DM you have actually used, the first fill cached the counterpart — so their erasure is permanently invisible and E2's row never appears.**

📌 **REACHABLE IN EXACTLY ONE SITUATION: a client with no cached record** — fresh install, new device, or a wiped book. **Real, but it is the multi-device case, not the one the row shape was drawn for.**

🛑 **NARROWED 2026-08-03 BY LEG D's DESIGN CLOSE — THE RULE STANDS, THE EXAMPLE SET DID NOT (`D-131`, kept not erased).** *"Exactly one situation"* enumerated three **installation** states. **A LIVE JOINER IS, BY CONSTRUCTION, NOT IN THE BOOK** — they arrived through a `membership.join` delta, and G7 says the fill was never asked about them. ⇒ once Leg D's Tier-1 fetch ships (§7, locked J-658; §6 D1 locked 2026-08-03), **③ is reachable for any erased live joiner in an ordinary session on an ordinary install.** 🔑 ***The rule that produced the enumeration — a held record is never re-fetched and never removed — is unchanged and still true; the enumeration was a claim narrower than the rule it came from, reused as if complete.*** ⇒ **§5a's E2 DM exception becomes reachable at Leg D**, and Leg F is the first surface that can exercise it. 📌 *Leg D's Phase-0 §6 carries the derivation.*

📌 **CITATION DRIFT, ANNOTATED NOT REPAIRED (`D-131`):** the two producers cited above sit at **`address_book.rs:267`** (`remove`) and **`:299`** (`evict_older_than`) as measured at `aae60be` on 2026-08-03; `:253`/`:285` were true when written and were moved by the `M-RP-XGID-SLOT-RETYPE` legs. **The claim they support — every caller is a test — is unaffected.**

✅ **§5a's E2 LOCK STANDS AND NEEDS NO REWORK.** It is the correct rule *when* the state occurs; only the row's visual design is belayed.

**What the belay drops, and what it therefore does NOT cost:**
- **The two-line row, the id line, and the L1/L2/L3 variant question** — all dropped.
- ✅ **`M_RP_MEMBERS` §7 / L10's *"secondary · meta · status ship UNFED"* lock STAYS CLOSED.** It would have been re-opened to feed the id line.
- ✅ **§6a's `tail-8` gap stays FILED, not a precondition.** It would have had to ship with the id line or the line would be noise.

⚠️ **AND CHAT'S SKETCH WAS WRONG TWICE, WHICH IS WHY A SKETCH MUST BE CHECKED AGAINST THE COMPONENT BEFORE ANYTHING IS LOCKED FROM IT:**
- 🛑 **It drew the `card` layout in a panel that renders `variant="row"`.** `showSecondary = variant === 'card'` (`entity-item:69`) and `.ei-meta` is `flex:none; margin-left:auto` (`skin.css:2469-2471`) ⇒ **under `row` the second field is pinned to the RIGHT EDGE of the same line, not placed below the name.**
- 🛑 **It struck through a display name that CANNOT EXIST.** With no book record, `toDescriptor` already falls back to `tail(m.identity_id)` ⇒ **an erased row has no display name to strike; it shows the xgid tail as its name.**

📌 **THE APPEARANCE VOCABULARY DOES ALREADY EXIST — AND IT IS NOT OURS TO TAKE.** `.entity-avatar[data-revoked]` (`skin.css:2400-2417`) ships greyscale + 0.55 opacity + a diagonal slash. 🛑 **But `revoked` ships UNFED (N-097) and belongs to M13, and `D-127` separates the two states: a revoked identity returns its record WITH `revoked` set; `not_found` is reserved for ERASURE.** ⇒ **reusing `data-revoked` for erasure would make them indistinguishable the day M13 lands.**

🛑 **THE REAL DEFECT THIS UNCOVERED IS NOT THIS MILESTONE'S AND IS FILED AGAINST M13 §3c (J-649): erasure is invisible to anyone holding a cached record.** A freshness window is meaningless while a re-fetch returns nothing new (`ops.rs:2755-2757`) and becomes meaningful only once M13 lands `revoked` + `update_version` ⇒ **the two must be designed together.**

📌 **CONSEQUENCE TO OWN, NOT A BLOCKER: hiding ③ makes the panel disagree with the DAG.** An erased member is still a member by causal replay — no leave, no kick — and their historical messages still render in the stream (G9). ⇒ **you can see messages from someone the panel says is not there.** **Chat's read: acceptable** — the panel answers *who is here now*, and someone who cannot sign an event is not meaningfully here — **but it is a real divergence between two truths and is recorded as chosen, not discovered later as a bug.**

---

## §6 — 🛑 TWO CAPABILITY GAPS: THE RULE IS NOT IMPLEMENTABLE TODAY

**Neither is a design question. Both are measured absences, and each blocks half of §4/§5.**

### G-A — ③ IS UNIDENTIFIABLE (CORRECTED AT v1.2 — THE GAP IS ONE-SIDED, NOT TWO)

⚠️ **v1.0/v1.1 STATED: *"THE CLIENT CANNOT TELL ③ FROM ④ — both end as no record in `_book`."* THAT IS FALSE AS OF J-643, and it was already false when this document was written.** Superseded, kept not erased (`D-131`).

✅ **④ IS ALREADY IDENTIFIABLE.** `addressBook.addMember` stamps `unresolved: true` on every live-joined member (`address-book.svelte.ts:168`), and `members-panel.svelte:101` **already branches on it** — that marker shipped in `M-RP-LIVEFEED-REFRESH` Leg A, the very leg that created state ④.

❌ **③ IS NOT.** It arrives through `setResult`, carries no marker, and is invisible: `FillReport` reports `not_found` as a **count** (`ops.rs:2779`), never a list. ⇒ **the client knows three lookups failed and never which three.**

🔑 **SO THE GAP IS ONE-SIDED, AND CLOSING IT IS SMALLER THAN v1.0 CLAIMED.** ⇒ **hiding ③ requires the ids; dimming ④ requires nothing new.**

🛑 **HOW THE ERROR HAPPENED, RECORDED BECAUSE IT IS THE RECURRING SPECIES:** G-A was reasoned from the state taxonomy and **never re-read against the code J-643 had just shipped one day earlier**. A claim about what the client can distinguish was written without opening the file that does the distinguishing.

⇒ **Leg A carries `not_found_ids` — runbook `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_A.md` v1.2, ✅ COMPLETED (J-647).** ✅ **G-A IS CLOSED: `FillReport.not_found_ids: Vec<IdentityXgid>` ships, and the client can now name which members returned `not_found`.** 📌 **AND IT DID NOT MOVE THE CARGO FLOOR** — 1588 / 0 / 62 × 56 re-measured unchanged, which is the expected result; see §8.

### G-B — "THE NEXT REFRESH" DOES NOT CURRENTLY ARRIVE

The fill's only trigger is a change in `roomLatch.effectiveSpaceId`, de-duped across rooms **within** a Space (G7). ⇒ **in a long session in one Space, the next refresh never comes.** A hidden ③ stays hidden and a dimmed ④ stays dimmed **for the rest of the session**.

🔑 **THIS IS §4c-i's RULE APPLIED ONE LEVEL OUT:** *a transient state is only sayable if the attempt is guaranteed to conclude.* **Dimming says *not yet*, and *not yet* commits the panel to eventually resolving.**

📌 **Joe's position, recorded:** the refresh is *"optical, from the user side — mechanism will be invented"*, and the members panel refresh is **paused** pending `M-RP-LIVEFEED-REFRESH`. ⇒ **the rule may be LOCKED now and IMPLEMENTED behind the mechanism.** ⚠️ **But it must not SHIP before the trigger fires**, or the panel makes a promise it cannot keep — which is the defect this project keeps catching.

**Candidate triggers, none locked:** retry on reconnect (rides the parent's §5, 🔓 Joe's and open) · refresh when the panel gains a row it cannot resolve · a bounded per-row retry. 📌 **Cheapest honest option is the reconnect hook, because that decision is being made anyway.**

🔒 **RULED 2026-08-02 (Joe, J-658) — OPTION T2: THE RECONNECT HOOK (R1) *AND* §7's TIER-1 FETCH ON JOIN, TOGETHER.** ⚠️ *v1.0–v1.12 carried the sentence above as the whole option space; it is not superseded — it is the menu the ruling chose from, and the paragraph following is the finding that removed one item from it. Kept not erased (`D-131`).*

### 🛑 §6b — N-168: THE RECONNECT HOOK ALONE DOES NOT CLOSE G-B, AND ADOPTING IT ALONE WOULD HAVE CLOSED G-B ON PAPER

**Re-grounded before the ruling, not inherited:** `app_client.svelte:168` — `const sid = roomLatch.effectiveSpaceId; // the sole tracked dependency`. One trigger, confirmed. And `selfState.connection` is read in exactly **two** places — `:144` (the status feed) and `:267` (`guardedSend`'s guard) — ⇒ **nothing re-fills on reconnect today.** G-B holds exactly as written.

🔑 **THE FINDING: R1 FIRES ON A TRANSITION INTO `READY`. G-B's FAILURE CASE IS A LONG SESSION IN ONE SPACE THAT NEVER DISCONNECTS.** A member who joins that session is dimmed for the rest of it. ⇒ **R1 discharges §4c-i's promise only in sessions that happen to suffer a network fault** — and a *good* session has none. 🛑 **Had §6 been ruled R1-alone, `docs/ROADMAP.md` would have carried `G-B closed` beside a panel that still promises a resolution it cannot deliver, and C-3 would have shipped on it.**

🔑 **N-168, STATED SO IT OUTLIVES THIS MILESTONE: *A TRIGGER THAT FIRES ONLY ON AN EXCEPTIONAL CONDITION CANNOT DISCHARGE A PROMISE MADE IN THE ORDINARY ONE.*** The reconnect hook is a **recovery** mechanism; §4's dimming is an **ordinary-path** claim. *Pairing them looks complete because both contain the word "refresh".*

⇒ **G-B IS CLOSED BY LEG D *AND* LEG E TOGETHER, NEVER BY EITHER ALONE.** Leg D makes the attempt happen on the ordinary path; Leg E recovers the exceptional one. **Neither leg may record `G-B closed` on its own.**

🛑 **AND THE RESIDUE IS NAMED, WITH AN OWNER AND A TRIGGER, SO IT IS NOT AN UNOWNED DEFERRAL.** A Tier-1 fetch that **times out** is still ④, and nothing retries it. §7 says so itself (*"a fetch that times out **is** ④"*). **T3 — a bounded per-row retry — was the option that literally satisfies §4c-i and it was NOT taken**, because its terminal state has no word and would re-open `D-126` (deferred J-588) for a case nobody has yet seen occur.

> **`Owes:` — LEG F MEASURES HOW OFTEN THE RESIDUE OCCURS, AND T3 IS RE-PRICED THEN.** Leg F is the first two-client run and therefore the first surface on which a real timed-out Tier-1 fetch can happen at all. ⚠️ **If the residue is common, T3 stops being an option and becomes a defect.** 📌 *Written as an obligation rather than an intention — `M_RP_LIVEFEED_REFRESH.md` §8a's own lesson: a deferral written without an owner and a trigger has neither.*

---

## §7 — 🔓 TIER-1 FETCH ON JOIN — CHAT PROPOSES, NOT LOCKED

**Today a live-joined member is never looked up at all** (G7). ⇒ ④ is not an edge case; **it is the default outcome of every live join.**

**Fetch Tier 1 for a joiner** ⇒ ④ collapses to ① / ② / ③ within one round trip, and **G6's transparency requirement is satisfied for live joiners as it already is for filled ones.**

- ① **User-visible:** a joiner appears dimmed for ~200 ms, then resolves with their name **and their AI badge**. **The badge is the point** — not decoration, a §3.6.10 obligation.
- ② **Resource:** a **new Tauri command** (G8 — none exists), so **Rust, and it moves the cargo floor**. Plus a **second writer to `_book`**, which today has exactly one (G7) — ⚠️ **the single-writer property is deliberate and this weakens it.** 📌 *Mitigation: a merge-one-record setter rather than a whole-book replace, so the fill stays the only wholesale writer.*

⚠️ **AND IT DOES NOT MAKE ④ DISAPPEAR.** A fetch that times out **is** ④. ⇒ **§4's dimming is required whether or not §7 ships**; §7 only makes ④ rare instead of universal.

**Chat recommends §7 ships**, because without it the AI-transparency guarantee has a hole that widens every time someone joins a room you are watching. 🔓 **Joe's, because it moves a floor and adds a command.**

🔒 **LOCKED 2026-08-02 (Joe, J-658): §7 SHIPS, AS LEG D.** Ruled together with §6 as one decision (option **T2**) — see §6b. 🔑 **THE REASON IT IS ONE DECISION AND NOT TWO: §7 IS THE ATTEMPT AND §6 IS THE RETRY, AND A RETRY OF AN ATTEMPT THAT NEVER HAPPENS IS NOT A RETRY.** Today no live joiner is looked up at all (G7) ⇒ without §7 the reconnect hook re-runs a fill for a member the fill was never asked about.

⚠️ **D-121 LENS ① WAS STATED FOR THE SHIPPED PRODUCT, NOT FOR TODAY'S DESK — AND THAT IS A DEPARTURE FROM J-654 THAT WAS MADE DELIBERATELY.** J-654's *"no user-facing impact"* was legal because that decision was about **ORDER**, and ③/④ need two clients. **This decision is about whether a promise is kept**, and it becomes visible the day two identities share a room — where §7's own grounding says ④ is *the default outcome of every live join*. 🔑 ***"Unreachable on one desk" is a fact about the test setup, not about the product*** — the argument that has now been wrong five times in this project (N-091 · N-097 · N-099 · N-109 · N-116).

🛑 **AND LEG D DOES NOT OPEN FIRST — SEE §8's Leg D.** `M-RP-XGID-SLOT-RETYPE` (`D-136`) lands ahead of it, standalone.

---

## §8 — Legs (proposed, not locked)

**Leg 0 — Phase-0.** This document. No code.

**Leg A — the `not_found` id list.** ✅ **CLOSED J-647.** `FillReport` gained `not_found_ids: Vec<IdentityXgid>`, surfaced through `fill_space_records`; TS mirror `not_found_ids: string[]`. **Runbook `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_A.md` v1.2 COMPLETED.** **2 files, +23 / −0.** ⇒ **closes G-A.** 🛑 **COMPILE-VERIFIED ONLY — Leg F is the first behaviour verification.**

⚠️ **v1.0/v1.1 SAID *"Rust; moves the cargo floor."* IT DOES NOT MOVE IT — corrected at v1.2, kept not erased (`D-131`).** The push sits inside `fill_from_events`, downstream of `ensure_connected` and `identity_get_on`; **the existing `not_found` test covers `absorb_fetch`, which is pure and never touches `FillReport`.** ⇒ **the change has no unit test and cannot get one without a live node.** 🔒 **Joe locked T-a (2026-08-01): ship it untested rather than refactor the function carrying the `session.conn` re-entrancy invariant.** ⇒ **cargo stays 1588 / 0 / 62 × 56, and Leg A is COMPILE-VERIFIED ONLY — Leg F is the first leg that can verify it behaviourally.**

🔒 **The TS mirror ships in Leg A too (Joe, boundary option ②)** — `address-book.svelte.ts`'s `FillReport` calls itself a mirror of `ops.rs`, and a mirror left stale for a whole leg is this milestone's own recurring defect. A type-only field with zero readers moves neither floor.

**Leg B — the render rules.** ✅ **CLOSED J-653.** Runbook `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_B.md` **v1.4 COMPLETED**. **Two commits: `06c5afe` (B-i, the wire witness, 1 file +40, cargo 1588 → 1589) · `7e06456` (B-ii, the render rules, 5 files +69/−22, zero `.rs`).** Clair implemented from the locked runbook; **Chat re-drove every gate.** 🛑 **AND IT LANDS SILENTLY — the only visible change is that erased non-DM members would disappear from the list, and that state is not currently reachable.**

🛑 **THE HONEST LIMIT, AMENDED MID-LEG (Joe, J-653): NEITHER ③ NOR ④ IS REACHABLE WITH ONE CLIENT, AND ④'s SIDE OF THAT LINE WAS ORIGINALLY DRAWN WRONG.** ③ was known unreachable at J-649. 🔑 **④ is in the same position, measured: `unresolved: true` is set in exactly ONE place** (`address-book.svelte.ts:187`, in `addMember`) **with exactly ONE caller** (`app_client.svelte:210`, the live membership router) ⇒ **it needs an inbound `membership.join` from another identity — the same two-client setup.** ⇒ **V8/V9/V10 reduced to what one client can show; all three positive cases OWED BY LEG F.**

✅ **WHAT WAS PROVEN LIVE:** the prop threads end-to-end through two `core` components — **26 `entity-item` getters (21 sampler + 5 client) all expose the field, all `null`; zero `[data-unresolved]` attributes across all 26.** 🔑 **`fieldPresentOnAll: true` is why Change 4(c) specifies `?? null`** — it separates *absent value* from *absent feature*, and without it the negative gate would be indistinguishable from a component that never received the prop.

🛑 **AND THE SURFACE MATTERS: THE CLIENT IS THE EVIDENCE, THE SAMPLER IS NOT.** The sampler mounts `entity-item` as isolated catalogue instances fed by literals and **has no `members-panel` at all** (measured: zero such ids at 9422) ⇒ **only the client exercises the wired path this leg changed.** *A sampler row proves the component accepts the prop; only a client row proves the path delivers it.* The sampler remains required for **V6 alone**.

✅ **V6 DONE PROPERLY: sampler catalogue 427 → 427, measured as a TRANSITION in one session** — pre-Leg-B `ui/` checked out, HMR reloaded, measured, restored, re-measured. 🔑 **The 328 carried in the PLAY record is from the M-RP6.1 arc and was never a valid baseline for this leg** — ***a stale baseline is worse than none, because it turns a real check into a false one.***

✅ **CLAIR READ IT ADVERSARIALLY BEFORE THE LOCK, AND IT WAS NOT CLEAN (J-651).** Sent in for the read with no authority to code, she returned four findings; **three were Chat's**, and re-driving them surfaced a **fifth** Chat defect she did not catch. 🛑 **The one that mattered: Change 2(b) rendered the §3.5 late-response guard as a bare comment inside a WHOLE-BODY replacement — a literal paste would have silently dropped it.** 🔑 **STANDING RULE EARNED: a whole-body replacement never elides a line behind a comment.** 📌 *Also corrected: G-B8 called the lowest-risk item in the leg "the ONE thing that will bite silently" and pointed at the wrong Change number; four store anchors were off by one; Change 3 mis-described a comment that was already correct. All annotated in the runbook, never deleted (`D-131`).* 🔑 **The read was worth its turn precisely because it was NOT clean — and it ran BEFORE the lock, which is what J-642 exists to say.**

🛑 **AND LEG B CANNOT BE A SINGLE-FLOOR LEG, WHICH THIS SECTION ASSUMED — §8 AND §9 CONTRADICTED EACH OTHER.** §8's split rule says B moves `svelte-check` only; §9 **owes Leg B the `Vec<IdentityXgid>` wire witness**, which is a **cargo** test. ⇒ 🔒 **RESOLVED BY SPLITTING THE LEG, NOT BY BREAKING EITHER RULE (Chat, mechanical, J-650): Leg B ships as TWO commits — B-i the Rust witness ALONE (cargo 1588 → 1589), B-ii the frontend ALONE (`svelte-check` only).** Attribution is preserved and the obligation is discharged inside Leg B as promised. 📌 *Found while authoring the runbook, by reading §8 against §9 rather than either alone.*

🛑 **B-1 — `_roster` STAYS COMPLETE. ③ IS FILTERED AT RENDER, NEVER OUT OF THE STORE.** `M_RP_MEMBERS.md` §4c consequence 3 locked *"any member count derives from the ROSTER, never from rendered rows"*, precisely so an offline panel cannot undercount. ⚠️ **Removing ③ from `_roster` would silently redefine `roster` from "the membership" to "the renderable membership"** — and C1's accepted mismatch would vanish instead of being accepted. ⇒ **the store records WHICH members are ③; the panel decides what to draw.**

🔒 **B-2 — THE HOOK TRAVELS AS A PROP (Joe, locked 2026-08-01 — option P1).** The chain is `members-panel` → `entity-panel` → `entity-item`, and `entity-item`'s root (`:112-119`) carries `data-variant` / `data-kind` / `data-selected` but **no unresolved prop**. ⇒ **a new public prop threaded through TWO `core` components** (also consumed by `self-panel` and the sampler). **Rejected: riding `EntityDescriptor`** — cheaper, but the descriptor describes *the entity* while this is a fact about **our knowledge of it**, repeating the category error §2's tier frame exists to prevent. **Rejected: `flags`** — it feeds `entity-avatar`, and the treatment belongs to the **row**; that route is how `isAi` ended up collapsed at J-643.

⚠️ **B-3 — LEG B MUST WIRE `outcome.fill` INTO THE STORE FIRST.** `app_client.svelte:183` is `addressBook.setResult(sid, outcome.roster, book)` — **the fill half is DISCARDED**, so Leg A's ids reach the webview and stop there. **`setResult`'s signature changes**, and the fill becomes a **second concern** in a store whose delta rules (R1–R4) were deliberately kept in one place. 📌 *Correct for Leg A's boundary; it is Leg B's first job, not a defect.*

📌 **B-4 — C1's MISMATCH BECOMES OBSERVABLE IN THE DEBUG SURFACE, AND THAT IS NOT THE TRIGGER FIRING.** `members-panel.svelte:131-135` already exposes **both** `memberCount: addressBook.roster?.length` **and** `rowCount: rows.length`; after Leg B they diverge. ✅ **Chat's read: C1's trigger says *the first milestone that RENDERS a member count*, and a CDP debug aggregate is not a rendered UI count ⇒ the trigger does NOT fire.** 🔑 **Recorded rather than assumed, because this is the nearest thing to a count that has ever existed.** 📌 **Independently confirmed: `members-panel` passes NO `title` and NO `badge` to `EntityPanel` (`:146`), and both are optional with no default** ⇒ **nothing renders a count anywhere on screen today.**

📌 **Leg B also OWES the `Vec<IdentityXgid>` wire witness** (§9) — it is the first leg with a real consumer to assert the shape against.

**Leg C — the skin.** 🔒 **SPLIT INTO THREE, AND THE SPLIT IS THE RULING (Joe, 2026-08-01, J-654 — option S3).** ✅ **Runbook `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_C.md` v1.1 COMPLETED — C-1 and C-2 implemented, measured and recorded (J-655).**

✅ **LEG C CLOSED 2026-08-04 (J-673) — ALL THREE THIRDS SHIPPED.** C-3 landed at `8a650b1` [Clair, 1 file, +27/−4] plus the comment follow-up `03c92cc` [+6/−4]; runbook `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_C3.md` **v1.5 COMPLETED**. The shared base rule (`.entity-item[data-unresolved] .ei-name`, weight `500`) and the unasked rule (`color: var(--t3)`) sit above the erased rule; the `N-109` obligation filed at J-655 is **paid** — the *"DELIBERATELY ABSENT"* note is retired now that the rule ships. **Eleven gates green, re-driven by Chat on the committed tree:** catalogue **435** and `svelte-check` **0/34/15** both unchanged · cargo **not run**, zero `.rs` by scope · live computed-style reads on the sampler — unasked `.ei-name` **`rgb(138,136,128)` / 500 / no decoration**, the erased row **still marked** with §5a-i's selection bar intact, the control row **untouched at 600**. 🔒 **`D-138`: the mechanism is verified; `500` and `--t3` are Joe's values and unreviewed** — a computed-style read proves a rule applies and cannot prove it looks right. 📌 *Runbook §8 carries what the run found, including a fresh staleness this leg introduced into the very sentence it was correcting.* ⚠️ *v1.11 read "v1.0 AUTHORED, NOT LOCKED"; true when written, killed by Joe's **"chat"** ruling — **the file was never locked and was implemented anyway, by Chat, with Clair reading the diff instead.** Superseded, kept not erased (`D-131`).* 🔒 **LOCKED SEPARATE FROM LEG B — IT DOES NOT RIDE ALONG (Joe, J-650).** 🛑 **CONSEQUENCE, WRITTEN DOWN SO LEG B's CLOSE IS NOT READ AS A FAILURE: LEG B LANDS SILENTLY.** ④'s dimming and E2's mark are both `skin.css` values ⇒ after Leg B the only visible change on screen is that erased non-DM members disappear from the list.

🛑 **THE CONTRADICTION THAT FORCED THE SPLIT, AND IT WAS REAL RATHER THAN A WORDING SLIP.** §11 and Leg B's runbook §9 both say *"§4's dimming must not SHIP before a refresh trigger exists"* — **G-B is open** — while `docs/ROADMAP.md`'s Leg C node read `↳ trigger: Leg B has landed — fired`. ⚠️ **A trigger that has fired is a defect by the standing convention**, and the node carried it beside a 🟡 state. **Two canonical records disagreed about whether the leg was unblocked.**

🔑 **THE GATE IS ASYMMETRIC, AND THAT IS THE WAY OUT — NEITHER RECORD HAD TO BE OVERRULED.** ④'s dimming says ***not yet*** — a **transient** claim, and §4c-i binds a transient claim to eventually conclude. **③'s mark says *gone* — a TERMINAL claim under §1 G3, and it promises nothing.** ***A rule that makes no promise cannot break one.***

| | ships | why |
|---|---|---|
| **C-1** `ui/sampler/src/app_sampler.svelte` | **now** | one `entity-panel#unresolved` cell — resolved control + `unasked` + `erased`, **inert and one-way, mirroring `members-panel.svelte:164`**. **The only surface on which either state can be made to render at all** (both need two clients in the client). Catalogue **427 → 435, MEASURED** — 🛑 *v1.11 predicted 434 on a `1 + 2N` model; a panel costs `2 + 2N`, because `entity-panel` wraps a self-registering `<Section>` (`section.svelte:69`). Superseded, kept not erased (`D-131`)* |
| **C-2** `ui/assets/skin.css` | **now** | **③'s mark ONLY.** Moves **neither floor** |
| **C-3** `ui/assets/skin.css` | ✅ **UNGATED J-670** — Leg E discharged | the bare `[data-unresolved]` base rule **and** `[data-unresolved="unasked"]`. **Not written as an instruction; it does not exist yet** |

🔒 **THE VALUES ARE DELEGATED TO CHAT FOR THIS LEG ONLY, AND THE STANDING RULE IS UNCHANGED (Joe, 2026-08-01).** *"normally i have skin.css, this rule still stays, especially when we build complex components. but those are small elements that are not worthy obvious workflow."* ⇒ **`ui/assets/skin.css` REMAINS JOE'S FILE.** This is a narrow carve-out for two selectors on an existing component, where the round-trip costs more than the decision; **he re-tunes any of it in Notepad++ at any time, with no milestone attached and no runbook required.** ⚠️ *Chat first stated this as "Chat now owns the Leg C default values" and proposed annotating the five documents that say `skin.css` is Joe's — **wider than what was given, and it would have converted a carve-out into a seat change.** Superseded before anything was written, kept not erased (`D-131`).* 🔑 ***A delegation accepted more broadly than it was given is how a seat quietly moves.***

🔒 **③'s MARK: a strikethrough on `.ei-name` (`--t2` text, `--t4` rule, 1px). MARKED, NEVER DIMMED — and that is a lock, not a taste.** §5a-i keeps the L16 highlight on the erased DM counterpart; `[data-selected]` paints it as `box-shadow: inset 2px 0 0` (`skin.css:2526`), and **an `opacity` on the root would composite that bar away with everything else.** 📌 **No `background` either** — the rule sits **after** `[data-selected]` at **equal specificity (0,2,0)**, so any background would silently outrank the selection; the file's own comment at `:2519-2520` already depends on that ordering. 🔑 **Every alternative was foreclosed by an existing lock rather than by preference:** a glyph ⇒ `core` owns the name (`D-108`), a component change · a word ⇒ re-opens `D-126`, deferred at J-588, and §5a's E2 was locked on *"NO NEW WORD REQUIRED"* · `data-revoked` ⇒ M13's, and `D-127` separates revoked from erased. ***Strikethrough is what is left after four locks.***

🛑 **AND THE SAMPLER FIXTURE USES XGID TAILS, NOT NAMES.** With no book record `toDescriptor` already falls back to `tail()` ⇒ **a real ③ row has no display name to strike** (§5b). ***A fixture with a human name would be tuning a case the product cannot produce — §5b's own defect, reproduced inside the surface built to prevent it.*** ⚠️ **Consequence to own: the strikethrough makes §6a's `tail-8` gap MORE visible, not less** — it strikes the constant `ed25519:` head the clip preserves.

📌 **FILED, DELIBERATELY NOT TAKEN: the case for shipping the BASE rule now.** A bare `.entity-item[data-unresolved]` muting `.ei-name` is defensible **today** — a machine identifier currently renders at the same weight and colour as a human display name, a false equivalence **whether or not a refresh ever arrives.** 🛑 **Not taken, because it also lands on ④, and ④ is what the gate holds.** Re-openable at C-3. *Recorded rather than silently decided (`D-065`).*

**Leg D — Tier-1 fetch on join.** 🔒 **UNGATED — §7 RULED, IT SHIPS (Joe, J-658).** ⚠️ *v1.0–v1.12 read "🔓 Gated on §7"; superseded, kept not erased (`D-131`).*

🔒 **DESIGN CLOSED 2026-08-03 — `tasks/M_RP_IDENTITY_RESOLUTION_LEGD_PHASE0.md` v1.1, four locks (Joe): A3 · B2 · C1 · D1.** Runbook `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_D.md` **v1.0 ACTIVE**, authored from those locks; Clair implements from it and does not close her own leg.

🛑 **AND §7's PRICE — *"a new Tauri command + a merge-one-record setter"* — WAS NARROWER THAN THE THING IT DESCRIBES.** Measured at `aae60be`, and this is a live instance of the named defect class sitting in this document:

1. ✅ **The Rust half is SMALLER than stated.** `ops::identity_get` already exists, public, one-shot (`ops.rs:539`, doc from `:528`). The command is a thin wrapper reusing `fill_space_records`'s session preamble; **no new `ops::` verb.**
2. 🛑 **But `fill_space_records` also LOADS AND SAVES the book** (`desktop.rs:671-713`), and there is **no resident book between Tauri commands**. ⇒ **persistence (§4 → B2) and the `FillLock` (§5 → C1) are two decisions §7 does not ask.** A command that returns without saving would let the next `setResult` wholesale-replace `_book` from a disk book that never heard of the joiner — and the fill's rows carry no `unresolved` field, so `members-panel.svelte:101` would take the **book branch with no record** and render `isAi: false`. **An AI joiner as a human — `N-097` inverted.**
3. 🛑 **THE AI BADGE DOES NOT LIGHT WHEN THE RECORD LANDS; IT LIGHTS WHEN `unresolved` CLEARS.** `members-panel.svelte:101` tests `m.unresolved` **before** it reads the book. ⇒ the marker clear is **the gate on the badge**, not cosmetic dimming — and there is **no clearing path today** (`address-book.svelte.ts:187` is the only assignment, swept both directions).
4. 🛑 **A LIVE JOINER CANNOT REACH `_notFound`** — its four writers (`:137` `:147` `:155` `:163`) are all fill-path. Without §6's D1 arm an erased joiner is dimmed forever, which is `G-B`'s own defect re-created inside the leg that closes it.

🔒 **SPLIT BY FLOOR (§8's rule): D-i Rust (cargo `1595 → 1596`, stated before the run) / D-ii frontend (`svelte-check`, re-measured before its first edit).** 🔑 **THE RETURN TYPE IS `Option<SeenRecord>`, not `FetchedIdentity`** — the frontend's `_book` values *are* `SeenRecord`, and returning the wire type would duplicate `SeenRecord::from_fetched` in TS. **No new struct ⇒ the slot gate expects PASS 74 unchanged.**

⚠️ *v1.13–v1.16 read: **"AND IT IS NOT THE NEXT LEG TO OPEN. `M-RP-XGID-SLOT-RETYPE` (`D-136`) LANDS FIRST, STANDALONE."** ✅ **DISCHARGED — that milestone CLOSED at J-669** (74 slots on the manifest, gate PASS, re-run at `aae60be`). **Nothing is in front of Leg D.** Superseded, kept not erased (`D-131`).*

✅ **LEG D CLOSED 2026-08-04 (J-672). `aa7d9c9` D-i [Clair, 2 files, +109/−2] · `9901036` D-ii [Clair, 2 files, +49/−6]; every gate re-driven by Chat on the COMMITTED tree.** `fetch_identity` ships as an 18→19th Tauri command returning `Option<SeenRecord>`, holding the `FillLock`, absorbing through `absorb_fetch` (now `pub(crate)`) and saving the book; `resolveMember` merges one record, **clears `unresolved`**, and routes a null to `_notFound`; `addMember` returns `boolean` so the router fetches only on a real add. **cargo 1595 → 1596** (Δ named, and the new test **proven able to fail**) · **`svelte-check` 0/34/15 → 0/34/15** against a freshly measured baseline · **slot gate PASS 74 unchanged on a CLEAN tree.** 🔑 **THE `now` PRODUCER WAS GROUNDED, NOT GUESSED** — the runbook left it as a marked hole and Clair filled it from `ops.rs:2913`, so no second timestamp format enters `last_seen`. 🔑 **AND B2 PAID TWICE:** persistence is also what makes `resolveMember`'s whole-function scope guard safe — a fetch resolving after a room switch drops its frontend write, but the record is **already on disk**, and §5b guarantees the next fill will not re-ask. **Under B1 it would have been lost outright** — a consequence B2's own derivation never looked at. 📌 *Runbook `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_D.md` **v1.1 COMPLETED** §10; Phase-0 `tasks/M_RP_IDENTITY_RESOLUTION_LEGD_PHASE0.md` **v1.2 COMPLETED**.* ⚠️ **COMPILE- AND TYPE-VERIFIED ONLY — no joiner, no `not_found`, no badge observed.**

✅ **GATE DISCHARGED 2026-08-02 (J-665): `M-RP-XGID-SLOT-RETYPE` LEG B LANDED at `c2975f3`.** `SeenRecord` and `FetchedIdentity` are typed, the address book is `BTreeMap<IdentityXgid, SeenRecord>`, and the three `String`↔typed bridge sites in `ops.rs` are gone. 🔒 **THIS LEG IS NOW OPENABLE** — opening it is Joe's. 📌 *The leg's name there is now **"the four address-book slots"**; "the three address-book structs" below is the superseded name, kept not erased (`D-131`).*

🔒 **NARROWED AT v1.14 (J-659), AFTER THAT MILESTONE'S SWEEP RAN: THIS LEG IS GATED ON ITS **LEG B** — the three address-book structs — NOT ON ITS CLOSE.** ⚠️ **The narrowing is forced by a measurement, not a preference: the record said *three structs*; the sweep found **88 identifier slots across 59 struct sites in four crates**.** 🔑 ***A dependency priced from a record that described three structs was a dependency priced from a sample*** — and gating this leg on all 88 would block it behind work it has no relationship to. ✅ **Legs A (enforcement) and B (the three structs) are small and on the path; Leg C is not.** 📌 *Recorded on both sides under `D-133`; see `tasks/M_RP_XGID_SLOT_RETYPE.md` §7.* The v1.0–v1.12 record offered *"lands first, OR Leg D absorbs it"*; **the fork is taken, not left open.** Three reasons, the third decisive:

1. **`D-071`** — subsystem audits precede dependent milestones, and this is one.
2. Leg D's new command **carries an `identity_id` slot** ⇒ retyping afterwards means touching that command **twice**. Leg A already documented a re-wrap workaround for exactly this (§9).
3. 🔑 **`M-RP-XGID-SLOT-RETYPE` IS A FILED MILESTONE WITH ITS OWN ID.** Folding it into a leg makes it a **rider** — and *"its own milestone, never a rider"* is a standing refusal in this project (`mergeClasses` · `M-RP-ROVING` · the `dialog` footer slot). ⚠️ **Both move cargo**, so absorbing it also makes the delta unattributable **within** the floor, which §8's split-by-floor rule exists to prevent.

**Leg E — the refresh trigger.** ✅ **CLOSED J-670 — AND IT NEEDED NO LINE OF ITS OWN.** Built by `M-RP-LIVEFEED-REFRESH` Leg C (commits `4c50796` + `9983988`). 🔑 **C-b0 MEASURED that the spaces re-fill CASCADES into the members fill** — `roomLatch.effectiveSpaceId` is an unmemoised getter over `spacesState.spaces`, counter **+1** with `sidBefore === sidAfter` **true** ⇒ **the members re-fill this leg needs was already delivered.** *Discharged by measurement, not by argument.* ⇒ ✅ **C-3 IS UNBLOCKED.** 🛑 **It does NOT close G-B — see §6b / N-168; G-B closes on Leg D *and* Leg E together, and Leg D has not landed.** ⚠️ **The milestone still must not close before Leg D**, or §4 ships a promise it cannot keep. 🛑 **N-169 (recorded on the parent, not fixed): ANY caller of `setSpaces`, ever, triggers a members re-fill** — **this leg's discharge DEPENDS on that cascade**, so memoising `effectiveSpaceId` would silently un-build it.

🛑 **AND LEG E IS NOT A NEW BUILD — IT IS `M_RP_LIVEFEED_REFRESH.md` §7's LEG C UNDER A SECOND NAME. NAMED HERE RATHER THAN DISCOVERED AT IMPLEMENTATION.** Both are *an `$effect` on `selfState.connection`* in **`ui/client/src/app_client.svelte`**. Two milestones filed one build.

- 🔒 **THE PARENT OWNS IT.** §5 is the parent's section and the reconnect rule is the parent's decision ⇒ **`M-RP-LIVEFEED-REFRESH` Leg C BUILDS; this Leg E CONSUMES AND VERIFIES**, and is discharged when the parent's Leg C lands. ⚠️ *Two seats writing one `$effect` from two runbooks is the one-writer-per-file-per-atom breach.*
- 🛑 ⇒ **C-3 IS GATED ON A LEG IN A DIFFERENT MILESTONE.** Legal, but it must appear in **both** `Owes:` lines (`D-133`) or one record goes stale invisibly.
- ✅ **GROUNDED, NOT ASSUMED — THE PARENT'S LEG C DOES NOT DEPEND ON ITS LEG B.** Leg B builds **delta setters** for live events; Leg C **re-runs fills**. Different mechanisms. ⇒ the ordering is free.
- 🔑 **AND LEG C HAS TWO HALVES, OF WHICH THIS MILESTONE NEEDS ONLY ONE.** The **members** half is free — `loadMembers(sid)` is already a named callable with the §3.5 late-guard. The **spaces/rooms** half is not: `spacesState.setSpaces(await invoke('get_spaces'))` is an **inline line inside the startup block** (`app_client.svelte:625`), **not a function**, and needs extracting. ⇒ **this milestone's dependency is the cheaper half**, which is worth knowing before the parent's runbook is scoped.

**Leg F — live verify + records.** Two clients, a real join, a real `not_found`. ⚠️ **A store driven by hand is a probe that cannot fail.** 🔒 **AND IT NOW CARRIES THREE OBLIGATIONS MOVED OUT OF LEG B (Joe, J-653):** ① a real join producing `data-unresolved="unasked"` · ② a real `not_found` producing the ③ filter **and §5a's E2 exception** · ③ a populated roster giving `erasedHidden` something to count. 🛑 **Leg F is the FIRST behaviour verification of this milestone — Legs A, B and D are compile- and type-verified only.**

🔒 **AND LEG D ADDED FOUR MORE (J-672) — IT IS THE LEG THAT CREATED THE SURFACE FOR ALL OF THEM:**

- ④ **a joiner resolving to ①/②** — the name lands **and the AI badge lights.** 🔑 *This is the one that proves §2b: the badge is gated on the `unresolved` clear, not on the record arriving, and nothing before Leg F can show which.*
- ⑤ **a joiner resolving to ③** — an erased live joiner reaches `_notFound` and is hidden, or **marked** if they are the DM counterpart. 📌 *§5a's E2 exception is reachable for the first time here — Leg D is what made it so.*
- ⑥ **a joiner whose fetch FAILS or TIMES OUT** — the row stays ④ and **nothing retries it.** This is the T3 residue (`D-126`), and Leg D is what creates the state at all.
- ⑦ **JOIN CONCURRENCY** — how many joins arrive at once. 🔒 *A3 shipped one-shot `identity_get` (one connect/auth/`goodbye` per joiner); if N-at-once joins are common the batched form returns as a live option, and if they are not it is **closed with its reason**. Priced against a number, not a fear.*

📌 **A/B/C/D split by floor deliberately** — A and D move cargo, B moves `svelte-check`, C moves neither. One commit spanning them makes a regression unattributable.

---

## §9 — Filed, NOT fixed

- 📌 **THE `Vec<IdentityXgid>` WIRE SHAPE HAS NO WITNESS — ONLY A SCALAR ONE (filed J-647).** `ops_result_struct_serde_transparent_wire_invariance` (`ops.rs:3438`, the Pass 4 T2 gate) covers **scalar** `IdentityXgid` slots only. The TS `string[]` mirror is correct — `Vec<T>` serialises as an array of `T`, and `T` is transparent — **but that is inference from serde semantics, not a measured property of this field**, and citing T2 as its witness would be a claim narrower than its subject. ⚠️ **A Vec-level witness would be a LEGITIMATE test** (it could genuinely fail if a serde attribute were added), **excluded from Leg A by scope alone** — it would move cargo to 1589 against the locked T-a. ⇒ **owed by Leg B**, which has a real consumer to assert against. 🔒 **DISCHARGED BY LEG B COMMIT B-i (`06c5afe`, J-653)** — `fill_report_not_found_ids_vec_serde_transparent_wire_invariance` ships in `mod pass_4_commit_1_tests` beside T2, with an exhaustive named literal (no `..Default`) and an empty-vec `[]` case. **It shipped ALONE, so the cargo move 1588 → 1589 stays attributable under §8's split rule.** ✅ **OBLIGATION CLOSED.**
- 🛑 **`SeenRecord` / `FetchedIdentity` / `FillReport` CARRY `String` IDENTIFIER SLOTS THAT SHOULD BE TYPED XGIDs — A POST-RETROFIT REGRESSION, FOUND 2026-08-01 (J-645).** The XGID Retrofit arc closed 2026-05-29 having retyped all of `xgen-client`; these three were written in `String` on 2026-07-25, while `MemberEntry` — written 3 days after the arc closed — is correctly `IdentityXgid`. **`SeenRecord.home_node: String` contradicts a Pass 4 borderline lock by name.** 🔑 **The `.as_str().to_string()` downgrade at `ops.rs:2734`/`:2742` is not a deliberate seam — it exists only to feed the `String`-keyed book, and disappears when this is fixed.** ⇒ **`D-136` minted; filed as `M-RP-XGID-SLOT-RETYPE` 🟡 PENDING.** ⚠️ **Leg A works around it with a documented re-wrap and does NOT fix it (`D-071`).** 📌 **Found because Joe recalled the retrofit and asked for the check to be re-run deeper — Chat's first pass sampled one function and concluded the opposite.**
- **`M_RP_MEMBERS.md` §6a — the `tail-8` lock-versus-build gap.** Joe locked *tail-8*; `tail()` returns the whole final path segment and `.ei-name` is **LEFT-ANCHORED** (`overflow:hidden; text-overflow:ellipsis; white-space:nowrap`), **so the clip takes the WRONG END** — every unresolved row reads `ed25519:AbCd…`, the constant head kept and the distinguishing bytes discarded. ⚠️ *v1.0–v1.5 wrote "the CSS clips the **left**", which is the inverse; corrected at v1.6, kept not erased (`D-131`).* ⚠️ **This milestone makes ④ rows more visible, so the gap becomes more visible with it.** Not fixed here.
- **`entity-avatar.svelte:125` collapses `isAi`'s third state** — `data-ai={flags.isAi || undefined}`, so `false` and absent render identically (J-643 §5-iv). **Leg A of `M-RP-LIVEFEED-REFRESH` made the store honest; the renderer still collapses it.** 🔓 Joe's, and it is the same family as §4.
- **③ MEANS ERASED — CORRECTED AT v1.2.** ⚠️ *v1.0/v1.1 said ③ "cannot be told from erased vs never replicated here"; superseded, kept not erased (`D-131`).* 🔒 **`D-127`** (cited at `tasks/M13_CLIENT_IDENTITY_LOOKUP_WIDENING.md:54`) locks that **a revoked Identity returns its record WITH `revoked` set, never `identity.not_found` — `not_found` is reserved for erasure.** ⇒ **independent corroboration, from a lock predating this milestone, of §1 G3's conclusion.** 📌 *What ③ still cannot distinguish is erased-here vs never-replicated-here; that remains open and is M13-adjacent.*
- **`role` / `joined_at` / `invited_by` arrive free and are discarded** (G5, `M_RP_MEMBERS.md` §7, delegated lock). 📌 **A ④ row could honestly show `role`** — protocol-derived, no lookup needed. **Not proposed here**; recorded because this is the first milestone where it would have a use.
- **The visit card (Tier 2) is undesigned**, and its scope rule may be answered by §2's frame. **Its own pass.**

---

## §10 — DoD

- [x] §4 ④ treatment locked by Joe — ✅ **done 2026-08-01, (B)**
- [x] §5 ③ hiding locked, **and the count question C1/C2/C3 ruled** — ✅ **C1 locked 2026-08-01, with a trigger: the first milestone rendering a member count re-opens it**
- [x] §5a the DM-counterpart exception ruled — ✅ **E2 locked 2026-08-01 (J-648): ③ is never hidden when it is the DM counterpart; the rule keys on `is_dm`, not member count**
- [x] §5a-i the DM highlight on an erased counterpart ruled by Joe — ✅ **KEEP locked 2026-08-01 (J-650); it costs ZERO lines, the highlight already flows from the roster**
- [x] §7 Tier-1 fetch ruled by Joe — ✅ **RULED 2026-08-02 (J-658): IT SHIPS, as Leg D**, together with §6 as one decision (option **T2**)
- [x] §6's refresh trigger ruled by Joe — ✅ **RULED 2026-08-02 (J-658): R1**, a re-fill on the transition into `READY`; **the parent's Leg C builds it**
- [x] **G-A closed** — ✅ **J-647.** `FillReport.not_found_ids: Vec<IdentityXgid>` ships; the client can name which members returned `not_found`
- [x] **G-B closed** — ✅ **CLOSED 2026-08-04 (J-672) BY THE PAIR, EXACTLY AS `N-168` REQUIRED.** Leg E (the refresh trigger, R1) discharged J-670 by `M-RP-LIVEFEED-REFRESH` Leg C; **Leg D (the attempt on the ordinary path) landed at `aa7d9c9` + `9901036`.** 🔑 *Neither leg ticked it alone, and the record shows both dates — which is the whole point of §6b's rule.* ⚠️ **The MECHANISM is built and compile-verified; that a refresh actually fires ON SCREEN is Leg F's ⑤/⑥.**
- [ ] **The residue re-priced** — Leg F measures how often a Tier-1 fetch times out; **T3's bounded retry returns as a live option, or becomes a defect** (§6b `Owes:`). 📌 *Now REACHABLE — Leg D created the state; before it, a Tier-1 fetch could not time out because there was no Tier-1 fetch.*
- [x] cargo floor re-measured on every Rust leg, delta explained — ✅ Leg A, Leg D-i (**1595 → 1596**, Δ named, test proven able to fail)
- [x] `svelte-check` floor re-measured on every frontend leg, delta explained — ✅ Leg B, Leg C, Leg D-ii (**0/34/15 → 0/34/15** against a freshly measured baseline, not the inherited figure), **Leg C-3 (0/34/15 unchanged, twice — the rule commit and the comment follow-up)**
- [ ] **Live-verified, EXERCISED not asserted** — two clients, a real join, a real `not_found`; ③ hidden, ④ dimmed, both resolving on refresh. 📌 *Leg C-3 verified the SKIN on the sampler (④'s tone and weight read off the painted DOM, J-673); **the STORE delivering that state to a real client row is still Leg F's**, and the two must not be read as the same claim.*
- [ ] Records: JOURNAL + CLAUDE.md PLAY + ROADMAP + this doc in one commit (D-074)

---

## §11 — Handoff

🔒 **LOCKED:** §2's tier frame (Joe) · §4's ④ treatment — dimmed, own selector (Joe) · **§5a's DM exception — E2, the counterpart is never hidden (Joe, J-648)** · **§8 B-2's hook shape — P1, a prop through `entity-panel` → `entity-item` (Joe, J-648)** · Leg A's type / boundary / test posture — X1 · ② · T-a (Joe, J-646) · **§5a-i — the erased DM counterpart KEEPS the L16 highlight (Joe, J-650)** · **Leg C ships SEPARATE from Leg B (Joe, J-650)** · **LEG C SPLITS — S3: C-1 + C-2 ship now (③ only), C-3 is gated on Leg E (Joe, J-654)** · **this leg's `skin.css` default values are DELEGATED TO CHAT, as a narrow carve-out that leaves the standing rule intact (Joe, J-654)**.

🔓 **JOE'S, OPEN — ONE ITEM LEFT:** §5's **DAG-divergence read** (Chat recorded it ACCEPTABLE AND CHOSEN; if Joe disagrees it becomes a real fork and needs writing as one).

✅ **LEG D's FOUR DESIGN LOCKS ADDED 2026-08-03 (Joe, one answer): A3** one-shot `identity_get`, the batched form filed with a Leg F trigger · **B2** the command persists the book · **C1** it takes the `FillLock` · **D1** a `not_found` reaches `_notFound`. 📌 *Derivations and the option lists they chose from live in `tasks/M_RP_IDENTITY_RESOLUTION_LEGD_PHASE0.md` v1.1 §§3–6; the implementation instructions live in the runbook. Neither is restated here.*

⚠️ *v1.0–v1.12 also listed §7's **Tier-1 fetch** and §6's **refresh trigger** here. **Both were ruled together on 2026-08-02 (J-658) as one decision — option T2: §7 ships as Leg D, and §6's trigger is R1.** The line's own hedge — *"should probably be decided once for both"* — **was right, and it was right for a reason it did not state**: §7 is the ATTEMPT and §6 is the RETRY. ⚠️ **But the pairing it proposed was still not sufficient — R1 alone does not close G-B (§6b / N-168).** Superseded, kept not erased (`D-131`).*

🔒 **AND TWO ITEMS WERE TAKEN BY CHAT UNDER `D-123` RATHER THAN ROUTED, BOTH RECORDED SO THEY CAN BE REVERSED ON ONE LINE:** ① **`M-RP-XGID-SLOT-RETYPE` lands first, standalone** — not absorbed into Leg D (§8 Leg D, three reasons) · ② **the parent's Leg C owns the `$effect`; this Leg E consumes it** (§8 Leg E). 🔑 *Both are sequencing and attribution, not architecture or appearance — the seat where `D-123` names **under-stepping** as the failure mode.*

⚠️ *v1.0–v1.10 listed **"the `skin.css` values — including E2's mark, which is entirely his"** among the OPEN items. **Superseded at v1.11 for THIS LEG ONLY** — Joe delegated the two Leg C selectors to Chat (§8 Leg C) while keeping `skin.css` his as a standing rule and keeping the right to re-tune with no milestone. **The line was correct when written and is now narrower than the arrangement; kept not erased (`D-131`).*** 🔑 **The distinction that matters: what moved is TWO SELECTORS, not the FILE.***

⚠️ **TWO ITEMS STRUCK FROM THIS LIST AT v1.1 — ANNOTATED, NOT DELETED (`D-131`). THIS SECTION WAS STALE AGAINST THE BODY OF ITS OWN DOCUMENT.** ① *"the milestone **ID and title** (Rule 8)"* — 🔒 **LOCKED `M-RP-IDENTITY-RESOLUTION` (Joe, 2026-08-01, J-644).** ② *"§5's **count** question (C1 / C2 / C3 — Chat recommends C3, or defer to the first milestone that renders a count)"* — 🔒 **C1 WAS ALREADY LOCKED AT §5 on 2026-08-01, with a trigger.** 🔑 **§5 recorded the lock and §10 ticked it while §11 still listed it OPEN — the sections were never read against each other.** ⚠️ **Same defect species as J-642's too-narrow citation sweep and J-643's §5-iii self-contradiction: a section narrower or staler than the thing it describes, in adjacent text.** Caught on the filing pass, recorded rather than quietly repaired.

✅ **G-A IS CLOSED AND LEG A HAS SHIPPED (J-647).** ⚠️ *v1.0–v1.4 closed this section with "NOTHING HERE IS BUILDABLE UNTIL G-A IS CLOSED — Leg 0 is complete; Leg A needs a runbook"; superseded, kept not erased (`D-131`).* 🛑 **What is still NOT buildable is the part G-B gates: §4's dimming must not SHIP before a refresh trigger exists, or the panel promises a resolution it cannot deliver.** 🔒 **AND THAT SENTENCE IS NOW A LEG BOUNDARY RATHER THAN A BLANKET HOLD (J-654): it gates C-3 and it does NOT reach ③'s mark, which promises nothing.** 🟡 **Leg C's C-1 + C-2 are SHIPPED (J-655); its runbook is `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_C.md` v1.1 COMPLETED. C-3 remains, gated on Leg E.** ✅ *Leg B closed at J-653; the runbook is COMPLETED.* ⚠️ *v1.11 read "Leg C is next-active as C-1 + C-2; its runbook v1.0 is AUTHORED, NOT LOCKED, and Clair is NOT stood up" — **all three clauses killed within the same day by Joe's "chat" ruling and the implementation that followed.** **FIFTH successive state of this one sentence** (v1.5 · v1.7 · v1.9 · v1.11, and now v1.12); superseded, kept not erased (`D-131`).* 🔑 ***It keeps going stale because it tries to carry a STATE, and a state belongs in a NODE — said at v1.11 and then true again one revision later, which is the strongest possible evidence for it.*** ⚠️ *v1.4–v1.6 closed this line with "and has no runbook" (false from J-650); v1.7–v1.8 with "AUTHORED, NOT LOCKED" (false from J-652); v1.9 with "LOCKED and Clair NOT stood up" (false from J-653, when she implemented it). **Three successive states of the same sentence, each superseded by the act it was waiting for.** All kept not erased (`D-131`).* 📌 *Caught by sweeping this document for gates that have fired before locking anything — J-642's own discipline, applied to the file that cites it.*

📌 **This milestone did not exist before 2026-08-01.** It was surfaced by `M-RP-LIVEFEED-REFRESH` Leg A creating the ④ state, and shaped almost entirely by Joe's questions — the two-tier frame, the *"what can be done with such a member"* test that made §5 structural rather than cosmetic, and the *"how does Carol say hello"* question that retracted a false Chat claim (G3) and corrected the whole taxonomy.
