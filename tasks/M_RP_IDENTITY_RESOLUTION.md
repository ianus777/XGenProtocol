# M-RP-IDENTITY-RESOLUTION — what a member row shows before the client knows who it is
> **Status**: ACTIVE  
> Version: 1.6  
> Date: Aug 2026  
> **Last updated**: 2026-08-01  
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

🔓 **§5a-i — OPEN, AND CREATED BY THIS RULING: DOES THE ERASED COUNTERPART STILL GET THE DM HIGHLIGHT?** `members-panel.svelte:146` passes `selected={counterpart}`, and `counterpart` is read from the roster ⇒ **by default the erased row renders SELECTED.** ① *User-visible:* a struck-through row also carrying the L16 highlight — *"this is who you are talking to"* on someone who cannot reply. ② *Resource:* one condition either way. 🔑 **Chat's recommendation: KEEP the highlight.** The DM counterpart is still who this room is *with*; the mark says they are gone, the highlight says whose room it is, and the two claims do not conflict. ⚠️ **But it is appearance and it is Joe's.**

📌 **§6a's `tail-8` GAP BECOMES VISIBLE HERE FIRST.** If the erased row shows an id fallback under the name, `tail()` returns the whole final segment and `.ei-name` is **LEFT-ANCHORED** — `overflow:hidden; text-overflow:ellipsis; white-space:nowrap` (`skin.css:2452-2458`) — so it keeps the constant head and ellipsises the distinguishing tail, rendering `ed25519:AbCd…` (`M_RP_MEMBERS.md` §6a). ⚠️ *v1.0–v1.5 wrote this as "the CSS clips the **left**" — **the inverse of the truth**; it is left-ANCHORED and clips the RIGHT. A paraphrase of J-618's correct wording, inverted. Superseded, kept not erased (`D-131`).*

---

### §5b — 🛑 THE ROW SHAPE IS BELAYED, AND THE REASON IS THAT ③ IS CURRENTLY UNREACHABLE (Joe, 2026-08-01, J-649)

🔒 **Joe drew a row shape for E2 — struck display name, id beneath, own CSS class — then said: *"if this case is defined somewhere else, belay this one."* ✅ IT IS EFFECTIVELY DEFINED ELSEWHERE, AND THE CASE IS ALSO UNREACHABLE. THE ROW SHAPE IS DROPPED.**

🛑 **WHY ③ CANNOT OCCUR FOR ANYONE YOU HAVE INTERACTED WITH.** Two measured facts compose:
1. **A held identity is NEVER re-fetched** — `partition_observed` (`ops.rs:2764`) routes held ids to *touch*, not *fetch*; the doc at `:2752` says so outright.
2. **A held record is NEVER removed in production** — `remove` (`address_book.rs:253`) and `evict_older_than` (`:285`) exist and **every caller is a test**.

⇒ **once the book holds someone, it holds them forever and never asks again** ⇒ **`identity.not_found` can only fire for an identity whose record was NEVER cached.** 🔑 **For a DM you have actually used, the first fill cached the counterpart — so their erasure is permanently invisible and E2's row never appears.**

📌 **REACHABLE IN EXACTLY ONE SITUATION: a client with no cached record** — fresh install, new device, or a wiped book. **Real, but it is the multi-device case, not the one the row shape was drawn for.**

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

---

## §7 — 🔓 TIER-1 FETCH ON JOIN — CHAT PROPOSES, NOT LOCKED

**Today a live-joined member is never looked up at all** (G7). ⇒ ④ is not an edge case; **it is the default outcome of every live join.**

**Fetch Tier 1 for a joiner** ⇒ ④ collapses to ① / ② / ③ within one round trip, and **G6's transparency requirement is satisfied for live joiners as it already is for filled ones.**

- ① **User-visible:** a joiner appears dimmed for ~200 ms, then resolves with their name **and their AI badge**. **The badge is the point** — not decoration, a §3.6.10 obligation.
- ② **Resource:** a **new Tauri command** (G8 — none exists), so **Rust, and it moves the cargo floor**. Plus a **second writer to `_book`**, which today has exactly one (G7) — ⚠️ **the single-writer property is deliberate and this weakens it.** 📌 *Mitigation: a merge-one-record setter rather than a whole-book replace, so the fill stays the only wholesale writer.*

⚠️ **AND IT DOES NOT MAKE ④ DISAPPEAR.** A fetch that times out **is** ④. ⇒ **§4's dimming is required whether or not §7 ships**; §7 only makes ④ rare instead of universal.

**Chat recommends §7 ships**, because without it the AI-transparency guarantee has a hole that widens every time someone joins a room you are watching. 🔓 **Joe's, because it moves a floor and adds a command.**

---

## §8 — Legs (proposed, not locked)

**Leg 0 — Phase-0.** This document. No code.

**Leg A — the `not_found` id list.** ✅ **CLOSED J-647.** `FillReport` gained `not_found_ids: Vec<IdentityXgid>`, surfaced through `fill_space_records`; TS mirror `not_found_ids: string[]`. **Runbook `tasks/RUNBOOK_IDENTITY_RESOLUTION_LEG_A.md` v1.2 COMPLETED.** **2 files, +23 / −0.** ⇒ **closes G-A.** 🛑 **COMPILE-VERIFIED ONLY — Leg F is the first behaviour verification.**

⚠️ **v1.0/v1.1 SAID *"Rust; moves the cargo floor."* IT DOES NOT MOVE IT — corrected at v1.2, kept not erased (`D-131`).** The push sits inside `fill_from_events`, downstream of `ensure_connected` and `identity_get_on`; **the existing `not_found` test covers `absorb_fetch`, which is pure and never touches `FillReport`.** ⇒ **the change has no unit test and cannot get one without a live node.** 🔒 **Joe locked T-a (2026-08-01): ship it untested rather than refactor the function carrying the `session.conn` re-entrancy invariant.** ⇒ **cargo stays 1588 / 0 / 62 × 56, and Leg A is COMPILE-VERIFIED ONLY — Leg F is the first leg that can verify it behaviourally.**

🔒 **The TS mirror ships in Leg A too (Joe, boundary option ②)** — `address-book.svelte.ts`'s `FillReport` calls itself a mirror of `ops.rs`, and a mirror left stale for a whole leg is this milestone's own recurring defect. A type-only field with zero readers moves neither floor.

**Leg B — the render rules.** ✅ **UNBLOCKED — Leg A landed the ids (J-647).** **Frontend; moves `svelte-check`.**

🛑 **B-1 — `_roster` STAYS COMPLETE. ③ IS FILTERED AT RENDER, NEVER OUT OF THE STORE.** `M_RP_MEMBERS.md` §4c consequence 3 locked *"any member count derives from the ROSTER, never from rendered rows"*, precisely so an offline panel cannot undercount. ⚠️ **Removing ③ from `_roster` would silently redefine `roster` from "the membership" to "the renderable membership"** — and C1's accepted mismatch would vanish instead of being accepted. ⇒ **the store records WHICH members are ③; the panel decides what to draw.**

🔒 **B-2 — THE HOOK TRAVELS AS A PROP (Joe, locked 2026-08-01 — option P1).** The chain is `members-panel` → `entity-panel` → `entity-item`, and `entity-item`'s root (`:112-119`) carries `data-variant` / `data-kind` / `data-selected` but **no unresolved prop**. ⇒ **a new public prop threaded through TWO `core` components** (also consumed by `self-panel` and the sampler). **Rejected: riding `EntityDescriptor`** — cheaper, but the descriptor describes *the entity* while this is a fact about **our knowledge of it**, repeating the category error §2's tier frame exists to prevent. **Rejected: `flags`** — it feeds `entity-avatar`, and the treatment belongs to the **row**; that route is how `isAi` ended up collapsed at J-643.

⚠️ **B-3 — LEG B MUST WIRE `outcome.fill` INTO THE STORE FIRST.** `app_client.svelte:183` is `addressBook.setResult(sid, outcome.roster, book)` — **the fill half is DISCARDED**, so Leg A's ids reach the webview and stop there. **`setResult`'s signature changes**, and the fill becomes a **second concern** in a store whose delta rules (R1–R4) were deliberately kept in one place. 📌 *Correct for Leg A's boundary; it is Leg B's first job, not a defect.*

📌 **B-4 — C1's MISMATCH BECOMES OBSERVABLE IN THE DEBUG SURFACE, AND THAT IS NOT THE TRIGGER FIRING.** `members-panel.svelte:131-135` already exposes **both** `memberCount: addressBook.roster?.length` **and** `rowCount: rows.length`; after Leg B they diverge. ✅ **Chat's read: C1's trigger says *the first milestone that RENDERS a member count*, and a CDP debug aggregate is not a rendered UI count ⇒ the trigger does NOT fire.** 🔑 **Recorded rather than assumed, because this is the nearest thing to a count that has ever existed.** 📌 **Independently confirmed: `members-panel` passes NO `title` and NO `badge` to `EntityPanel` (`:146`), and both are optional with no default** ⇒ **nothing renders a count anywhere on screen today.**

📌 **Leg B also OWES the `Vec<IdentityXgid>` wire witness** (§9) — it is the first leg with a real consumer to assert the shape against.

**Leg C — the skin.** The dimmed treatment in `ui/assets/skin.css`. ⚠️ **JOE'S FILE.** Chat supplies the hook and measures; **the values are his**.

**Leg D — Tier-1 fetch on join.** 🔓 Gated on §7. New Tauri command + merge-one setter. **Moves the cargo floor.**

**Leg E — the refresh trigger.** 🔓 Gated on **G-B** and on the parent's §5. ⚠️ **The milestone must not close before this**, or §4 ships a promise it cannot keep.

**Leg F — live verify + records.** Two clients, a real join, a real `not_found`. ⚠️ **A store driven by hand is a probe that cannot fail.**

📌 **A/B/C/D split by floor deliberately** — A and D move cargo, B moves `svelte-check`, C moves neither. One commit spanning them makes a regression unattributable.

---

## §9 — Filed, NOT fixed

- 📌 **THE `Vec<IdentityXgid>` WIRE SHAPE HAS NO WITNESS — ONLY A SCALAR ONE (filed J-647).** `ops_result_struct_serde_transparent_wire_invariance` (`ops.rs:3438`, the Pass 4 T2 gate) covers **scalar** `IdentityXgid` slots only. The TS `string[]` mirror is correct — `Vec<T>` serialises as an array of `T`, and `T` is transparent — **but that is inference from serde semantics, not a measured property of this field**, and citing T2 as its witness would be a claim narrower than its subject. ⚠️ **A Vec-level witness would be a LEGITIMATE test** (it could genuinely fail if a serde attribute were added), **excluded from Leg A by scope alone** — it would move cargo to 1589 against the locked T-a. ⇒ **owed by Leg B**, which has a real consumer to assert against.
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
- [ ] §5a-i the DM highlight on an erased counterpart ruled by Joe — Chat recommends KEEP
- [ ] §7 Tier-1 fetch ruled by Joe
- [x] **G-A closed** — ✅ **J-647.** `FillReport.not_found_ids: Vec<IdentityXgid>` ships; the client can name which members returned `not_found`
- [ ] **G-B closed** — a refresh trigger that actually fires, named and built
- [ ] cargo floor re-measured on every Rust leg, delta explained
- [ ] `svelte-check` floor re-measured on every frontend leg, delta explained
- [ ] **Live-verified, EXERCISED not asserted** — two clients, a real join, a real `not_found`; ③ hidden, ④ dimmed, both resolving on refresh
- [ ] Records: JOURNAL + CLAUDE.md PLAY + ROADMAP + this doc in one commit (D-074)

---

## §11 — Handoff

🔒 **LOCKED:** §2's tier frame (Joe) · §4's ④ treatment — dimmed, own selector (Joe) · **§5a's DM exception — E2, the counterpart is never hidden (Joe, J-648)** · **§8 B-2's hook shape — P1, a prop through `entity-panel` → `entity-item` (Joe, J-648)** · Leg A's type / boundary / test posture — X1 · ② · T-a (Joe, J-646).

🔓 **JOE'S, OPEN:** **§5a-i — does the erased DM counterpart still carry the L16 highlight?** (Chat recommends KEEP) · §7 **Tier-1 fetch on join** (Chat recommends it ships) · §6's **refresh trigger**, which overlaps the parent's §5 reconnect rule and should probably be decided once for both · the `skin.css` values — **including E2's mark, which is entirely his** · §5's **DAG-divergence read** (Chat recorded it ACCEPTABLE AND CHOSEN; if Joe disagrees it becomes a real fork and needs writing as one).

⚠️ **TWO ITEMS STRUCK FROM THIS LIST AT v1.1 — ANNOTATED, NOT DELETED (`D-131`). THIS SECTION WAS STALE AGAINST THE BODY OF ITS OWN DOCUMENT.** ① *"the milestone **ID and title** (Rule 8)"* — 🔒 **LOCKED `M-RP-IDENTITY-RESOLUTION` (Joe, 2026-08-01, J-644).** ② *"§5's **count** question (C1 / C2 / C3 — Chat recommends C3, or defer to the first milestone that renders a count)"* — 🔒 **C1 WAS ALREADY LOCKED AT §5 on 2026-08-01, with a trigger.** 🔑 **§5 recorded the lock and §10 ticked it while §11 still listed it OPEN — the sections were never read against each other.** ⚠️ **Same defect species as J-642's too-narrow citation sweep and J-643's §5-iii self-contradiction: a section narrower or staler than the thing it describes, in adjacent text.** Caught on the filing pass, recorded rather than quietly repaired.

✅ **G-A IS CLOSED AND LEG A HAS SHIPPED (J-647).** ⚠️ *v1.0–v1.4 closed this section with "NOTHING HERE IS BUILDABLE UNTIL G-A IS CLOSED — Leg 0 is complete; Leg A needs a runbook"; superseded, kept not erased (`D-131`).* 🛑 **What is still NOT buildable is the part G-B gates: §4's dimming must not SHIP before a refresh trigger exists, or the panel promises a resolution it cannot deliver.** 🟡 **Leg B is next-active and has no runbook.**

📌 **This milestone did not exist before 2026-08-01.** It was surfaced by `M-RP-LIVEFEED-REFRESH` Leg A creating the ④ state, and shaped almost entirely by Joe's questions — the two-tier frame, the *"what can be done with such a member"* test that made §5 structural rather than cosmetic, and the *"how does Carol say hello"* question that retracted a false Chat claim (G3) and corrected the whole taxonomy.
