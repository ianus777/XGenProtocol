# M-RP-IDENTITY-RESOLUTION — what a member row shows before the client knows who it is
> **Status**: ACTIVE  
> Version: 1.1  
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
| ④ | **Never asked, or asked and never heard back** | **no answer of any kind** | ✅ **probably — this says nothing about the node** | ✅ **yes, fully** |

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

📌 **CONSEQUENCE TO OWN, NOT A BLOCKER: hiding ③ makes the panel disagree with the DAG.** An erased member is still a member by causal replay — no leave, no kick — and their historical messages still render in the stream (G9). ⇒ **you can see messages from someone the panel says is not there.** **Chat's read: acceptable** — the panel answers *who is here now*, and someone who cannot sign an event is not meaningfully here — **but it is a real divergence between two truths and is recorded as chosen, not discovered later as a bug.**

---

## §6 — 🛑 TWO CAPABILITY GAPS: THE RULE IS NOT IMPLEMENTABLE TODAY

**Neither is a design question. Both are measured absences, and each blocks half of §4/§5.**

### G-A — THE CLIENT CANNOT TELL ③ FROM ④

Both end as *no record in `_book`*. `FillReport` carries `not_found` as a **count** (`ops.rs:2779`), never a list of ids. ⇒ **the client knows three lookups failed and never which three.**

⇒ **Hiding ③ while dimming ④ requires the ids.** A `Vec<IdentityXgid>` alongside the count, surfaced through `fill_space_records`. **Rust; moves the cargo floor.**

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

**Leg A — the `not_found` id list.** `FillReport` carries the ids; surfaced through `fill_space_records`. **Rust; moves the cargo floor.** ⇒ closes **G-A**.

**Leg B — the render rules.** `data-unresolved` on `.entity-item`; ③ filtered from the rendered list; the store distinguishes the two. **Frontend; moves `svelte-check`.** ⚠️ **Gated on Leg A** — without the ids there is nothing to branch on.

**Leg C — the skin.** The dimmed treatment in `ui/assets/skin.css`. ⚠️ **JOE'S FILE.** Chat supplies the hook and measures; **the values are his**.

**Leg D — Tier-1 fetch on join.** 🔓 Gated on §7. New Tauri command + merge-one setter. **Moves the cargo floor.**

**Leg E — the refresh trigger.** 🔓 Gated on **G-B** and on the parent's §5. ⚠️ **The milestone must not close before this**, or §4 ships a promise it cannot keep.

**Leg F — live verify + records.** Two clients, a real join, a real `not_found`. ⚠️ **A store driven by hand is a probe that cannot fail.**

📌 **A/B/C/D split by floor deliberately** — A and D move cargo, B moves `svelte-check`, C moves neither. One commit spanning them makes a regression unattributable.

---

## §9 — Filed, NOT fixed

- **`M_RP_MEMBERS.md` §6a — the `tail-8` lock-versus-build gap.** Joe locked *tail-8*; `tail()` returns the whole final path segment and the CSS clips the **left**, so every unresolved row reads `ed25519:…` — the constant head kept, the distinguishing bytes discarded. ⚠️ **This milestone makes ④ rows more visible, so the gap becomes more visible with it.** Not fixed here.
- **`entity-avatar.svelte:125` collapses `isAi`'s third state** — `data-ai={flags.isAi || undefined}`, so `false` and absent render identically (J-643 §5-iv). **Leg A of `M-RP-LIVEFEED-REFRESH` made the store honest; the renderer still collapses it.** 🔓 Joe's, and it is the same family as §4.
- **③ cannot be told from *erased* vs *never replicated here*** — `not_found` does not distinguish them. **`revoked` on the wire is M13.** Until then, "hidden" is the same treatment for both.
- **`role` / `joined_at` / `invited_by` arrive free and are discarded** (G5, `M_RP_MEMBERS.md` §7, delegated lock). 📌 **A ④ row could honestly show `role`** — protocol-derived, no lookup needed. **Not proposed here**; recorded because this is the first milestone where it would have a use.
- **The visit card (Tier 2) is undesigned**, and its scope rule may be answered by §2's frame. **Its own pass.**

---

## §10 — DoD

- [x] §4 ④ treatment locked by Joe — ✅ **done 2026-08-01, (B)**
- [x] §5 ③ hiding locked, **and the count question C1/C2/C3 ruled** — ✅ **C1 locked 2026-08-01, with a trigger: the first milestone rendering a member count re-opens it**
- [ ] §7 Tier-1 fetch ruled by Joe
- [ ] **G-A closed** — the client can name which members returned `not_found`
- [ ] **G-B closed** — a refresh trigger that actually fires, named and built
- [ ] cargo floor re-measured on every Rust leg, delta explained
- [ ] `svelte-check` floor re-measured on every frontend leg, delta explained
- [ ] **Live-verified, EXERCISED not asserted** — two clients, a real join, a real `not_found`; ③ hidden, ④ dimmed, both resolving on refresh
- [ ] Records: JOURNAL + CLAUDE.md PLAY + ROADMAP + this doc in one commit (D-074)

---

## §11 — Handoff

🔒 **LOCKED:** §2's tier frame (Joe) · §4's ④ treatment — dimmed, own selector (Joe).

🔓 **JOE'S, OPEN:** §7 **Tier-1 fetch on join** (Chat recommends it ships) · §6's **refresh trigger**, which overlaps the parent's §5 reconnect rule and should probably be decided once for both · the `skin.css` values · §5's **DAG-divergence read** (Chat recorded it ACCEPTABLE AND CHOSEN; if Joe disagrees it becomes a real fork and needs writing as one).

⚠️ **TWO ITEMS STRUCK FROM THIS LIST AT v1.1 — ANNOTATED, NOT DELETED (`D-131`). THIS SECTION WAS STALE AGAINST THE BODY OF ITS OWN DOCUMENT.** ① *"the milestone **ID and title** (Rule 8)"* — 🔒 **LOCKED `M-RP-IDENTITY-RESOLUTION` (Joe, 2026-08-01, J-644).** ② *"§5's **count** question (C1 / C2 / C3 — Chat recommends C3, or defer to the first milestone that renders a count)"* — 🔒 **C1 WAS ALREADY LOCKED AT §5 on 2026-08-01, with a trigger.** 🔑 **§5 recorded the lock and §10 ticked it while §11 still listed it OPEN — the sections were never read against each other.** ⚠️ **Same defect species as J-642's too-narrow citation sweep and J-643's §5-iii self-contradiction: a section narrower or staler than the thing it describes, in adjacent text.** Caught on the filing pass, recorded rather than quietly repaired.

⚠️ **NOTHING HERE IS BUILDABLE UNTIL G-A IS CLOSED.** Leg A is the precondition for every render rule in §4 and §5. **Leg 0 is complete; Leg A needs a runbook.**

📌 **This milestone did not exist before 2026-08-01.** It was surfaced by `M-RP-LIVEFEED-REFRESH` Leg A creating the ④ state, and shaped almost entirely by Joe's questions — the two-tier frame, the *"what can be done with such a member"* test that made §5 structural rather than cosmetic, and the *"how does Carol say hello"* question that retracted a false Chat claim (G3) and corrected the whole taxonomy.
