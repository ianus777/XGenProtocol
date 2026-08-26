# M-SPACE-ADMISSION Leg G Phase-0 — the rejoin anchor, and the gate that never learned about rejoining
> **Status**: ACTIVE  
> Version: 1.5  
> Date: Aug 2026  
> **Last updated**: 2026-08-26  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS FILE IS

The `D-071` Phase-0 for **Leg G — the rejoin anchor verb**, the last leg of `M-SPACE-ADMISSION — who may join a Space, and how a leaver comes back`.

📌 **Chat wrote this file BEFORE asking for anything it did not already have.** Joe locked two things at session open (2026-08-24) and they are recorded at their own sites: **the leg is scheduled** (§1) and **the wire shape is option ②** (§4). Everything else in this file is Chat's seat and is closed here, not routed. `D-123`'s named failure — Chat gating its own authoring on an unasked Joe decision — is what this file exists to avoid.

🛑 **NO PRODUCT CODE. Reads only.** Zero `.rs`, zero `.ts`, zero `.svelte`, zero `ui/**` written by this pass.

📌 **Every claim below carries its site, and every site was OPENED against source at `ce3060e`** (`D-152` clause 1, `D-153`). Where a claim is a source trace rather than a run, it says so in the same sentence.

---

## §1 — 🔒 THE SCHEDULING LOCK

🔒 **Joe, 2026-08-24: Leg G is scheduled, in the shape *make it work*, carrying *make it honest* inside it.**

📌 **The alternatives put up and refused:** ① `3048` alone — the node stops lying and starts refusing, but she still cannot return; ③ neither, and the slot goes to the round-2 whole-codebase audit. **Both remain available as descopes if the leg over-runs; ① is now `G-2` and ships independently of the rest.**

---

## §2 — 🛑 THE GROUNDING, AND IT IS WORSE THAN THE ANCHOR PROBLEM

### §2.1 — G-1 — THE GATE IMPLEMENTS TWO OF ITS PREDICATE'S THREE TERMS

The master Phase-0 §15.4 specifies the gate as: **admit iff** the Space resolves `open` · **or** the sender holds a `pending_invites` entry · **or the sender is a FORMER MEMBER (§15.5)**.

✅ **MEASURED at `xgen-core/src/node/runtime.rs:1620-1621`:**

```
if space.admission != ADMISSION_OPEN
    && !space.pending_invites.contains_key(&event.sender)
```

**Two terms. The third is absent.** ✅ **`left_at` has ZERO non-test occurrences in `runtime.rs`** — three hits, all inside `#[cfg(test)]` (`:3720`, `:3751`, `:5904`), each an applier-level assertion about the members map, none touching the gate.

✅ **AND THE APPLIER DOES IMPLEMENT IT.** `apply_join` (`xgen-core/src/space/state.rs:1214`) carries the three-way gate, in its own comment: *record ABSENT → fall through · record PRESENT → AlreadyMember · record PRESENT but DEPARTED → fall through — **this is the rejoin***.

🔑 **⇒ THE DISPATCH GATE REFUSES HER `3047` BEFORE THE APPLIER THAT WOULD ADMIT HER IS EVER REACHED.**

🛑 **SCOPE, STATED PRECISELY AND NOT ONE WORD WIDER.** The gate only fires when `admission != open`. ✅ **Both DM constructors pin `invite` unconditionally at construction** (Leg B; `state.rs` `from_dm_space_create` / `from_dm_space_create_node`), and construction happens at **fold time from the create event**, so this reaches **DMs created before Leg B as well as after**. ⇒ **the defect covers EVERY DM, plus any regular Space whose owner has closed it. A regular Space left at the default `open` is unaffected** — the gate never fires there, which is why the arc's own rejoin work looked complete.

⚠️ **`D-154`①, `Q-2`(a) and `§15.5` all say a former member is re-admitted WITHOUT an invite. Today, in a DM, she is refused outright.** *`Q-2`(a) was ruled precisely because `apply_invite` bars all DM invites, so without it a DM departure is irreversible — and the gate reinstated the irreversibility from a different line.*

🔑 **HOW IT GOT HERE, AND IT IS A NAMED SHAPE.** §15.4 was written when `left_at` did not exist. **J-763 Deviation ① found `D-3` UNBUILDABLE and moved it to Leg E** — correctly. **Leg E then built `left_at`, the four appliers and `apply_join`'s third arm, and nobody returned to the gate.** ⇒ ***a deferral is valid only as long as its premise holds; when the premise dies the deferral dies with it, and it does not quietly inherit a new one*** — `N-095`'s rule, at gate scale, one arc later.

🛑 **AND NO INSTRUMENT COULD HAVE SEEN IT.** ✅ **Thirteen `fn *rejoin*` sites exist across the workspace** (`derive.rs` ×2, `state.rs` ×4, `ops.rs` ×3, `fanout.rs` ×4). **Every one is applier-level, fold-level, fanout-level or client-level. ZERO exercise the dispatch gate with a former member.** ⇒ **the suite is green because the case is unwritten, not because it passes.** *A check that was never written is not a check that passed.*

### §2.2 — G-2 — THE ACCEPT-THEN-DROP IS ASSERTED BY A SHIPPED TEST, NOT INFERRED

✅ **`xgen-core/src/resolution/derive.rs:498` — `convergence_mp_f7_rejoin_anchored_at_root_is_dropped`.** The losing arm of §15.6 is **in the suite, green, asserting the drop.** Its sibling at `:471` asserts the anchored case is a member.

✅ **The chain is measured end to end:** `algorithm.rs:146-147` picks `MembershipLeave` over `MembershipJoin` on a frontier of two; `derive.rs` excludes the loser. **The node answers `Accepted` and the fold drops the join.**

📌 **UPGRADED FROM THE SESSION-OPEN CLAIM.** At session open Chat called this a source trace. **It is stronger than that: the drop is a shipped assertion.** *Recorded because under-claiming a defect is a record error in the same family as over-claiming one.*

🔒 **`3048` IS RESERVED AND NOT LIVE, AND ONLY ONE RECORD SAID SO.** ✅ `3048` and `rejoin_not_anchored` have **ZERO occurrences in any `.rs`** across all four crates. ✅ **ch3 §3.6.10.10 is honest** (`docs/xgen_ch3_specification.md:2199`): *3048 remains reserved for the M-SPACE-ADMISSION causal-anchor invariant and is not yet live.* 🛑 **Three other records asserted the opposite by implication and are repaired by this pass — see §7.**

### §2.3 — G-3 — THE DOOR ALREADY SERVES THE RIGHT SET; ONLY THE KEY IS MISSING

| # | fact | site |
|---|---|---|
| **B-1** | `is_structural_bootstrap_type` serves `space_create · dm_space_create · room_create · invite · join · leave · kick · ban · node_eject · node_unban` — **the whole membership chain, content excluded** | `xgen-node/src/fanout.rs:615-638` |
| **B-2** | `collect_invite_bootstrap` refuses on **authorization ONLY**: `space.pending_invites.get(requester_id).ok_or(REFUSED)?`, `REFUSED = (1011, "invite_bootstrap_refused")` | `fanout.rs:744`, `:751` |
| **B-3** | The read-gate below it is the invite's own `valid_until` — **inside the `pending` branch**, so it has nothing to say about a requester who holds no invite | `fanout.rs:753-768` |
| **B-4** | The node arm replies `HistoryBatch` + `SyncComplete`; a refusal goes back as `TransportMessage::Error` carrying the code | `xgen-node/src/app.rs:1795-1836` |
| **B-5** | Wire variant `InviteBootstrapRequest { protocol_version, space_id }`, `#[serde(rename = "transport.invite_bootstrap_request")]` | `xgen-core/src/wire/types.rs:168` |
| **B-6** | **Exactly FOUR non-test sites** name the variant: `wire/types.rs:168` · `xgen-client/src/batch.rs:270` · `xgen-node/src/app.rs:1795`, plus the client test file | corpus-wide, `.rs`, `target\` + `.claude\` excluded |

🔑 **⇒ THE PAYLOAD A REJOINER NEEDS IS ALREADY BEHIND THIS DOOR.** `D-154`④-as-clarified rules that she receives the membership structure of her absence — and **B-1 is that set, already served, to a stranger holding an invite.** *`D-154`④'s own clarification names `collect_invite_bootstrap` as one of the three doors it governs.*

### §2.4 — 🛑 G-4 — THE CLIENT WILL NOT FIND AN ANCHOR IN THAT BATCH, AND THIS IS WHY G-4 IS A LEG

✅ **`get_invite_bootstrap` (`xgen-client/src/batch.rs:262`) scans the batch for exactly one thing:** `MembershipInvite` whose `content["target_identity"]` is the requester, returning its `event_id`.

🔑 **A rejoiner has no such event.** Her invite was **consumed at her first join** (`pending_invites.remove` in `apply_join`), and in a DM `apply_invite` bars invites outright. ⇒ **widening the node's authorization alone changes nothing**: the call returns `Ok(None)`, `ops::join` falls through to `get_dag_tips`, which starves on `is_member` (`fanout.rs:485-487`), and it lands on `rejoin_anchor_or_root` — **exactly today's behaviour.**

⇒ 🔒 **G-3 WITHOUT G-4 IS A NO-OP, AND THE TWO MUST NOT BE ALLOWED TO CLOSE SEPARATELY WITHOUT A GATE THAT SAYS SO.** G-3's DoD asserts the node **serves**; G-4's asserts the client **anchors**. *A leg whose whole effect is invisible until the next leg lands is a leg that will be reported as done and measured as nothing.*

---

## §3 — 🛑 THE MEASUREMENT THAT INVERTED THE DESIGN

**§15.7 of the master Phase-0 states, as an argument for a new node-side verb:** *"An older Node that cannot answer ⇒ the client falls back to exactly today's behaviour."*

🛑 **THAT IS FALSE, AND IT IS FALSE STRUCTURALLY.**

| # | fact | site |
|---|---|---|
| **W-1** | `TransportMessage` is `#[serde(tag = "type")]` | `xgen-core/src/wire/types.rs:48` and the derive above it |
| **W-2** | **`serde(other)` has ZERO occurrences** in `wire/types.rs` or `xgen-common/src/wire.rs` — there is no unknown-variant arm | corpus grep, case-sensitive |
| **W-3** | `recv` routes on the `transport.` prefix into `serde_json::from_value(value)?` — **an unrecognised `transport.*` type is a deserialisation ERROR**, and the function's own comment says mis-routing *silently kills the connection* | `xgen-core/src/transport/connection.rs:443-478` |
| **W-4** | The node's client-connection loop ends `Err(_) => break` | `xgen-node/src/app.rs:2048` |

🔑 **⇒ A NEW `transport.*` VERB SENT TO AN OLDER NODE DOES NOT PRODUCE A REFUSAL. IT DROPS THE SESSION.** From the person's side: the app disconnects, with no message and nothing to retry.

⚠️ **CHAT NEARLY WROTE THE OPPOSITE INTO THE RECOMMENDATION**, on the strength of §15.7's own sentence, in the same session whose kickoff names *"every one was a claim I could have checked and didn't"* as the arc's pattern. **Checked, and it inverted the answer.**

---

## §4 — 🔒 THE WIRE SHAPE. **RULED (Joe, 2026-08-24): ② — ONE MORE KEY ON THE DOOR THAT EXISTS.**

**The two options, as put:**

- **① A NEW DOOR** — `transport.rejoin_anchor_request`, a node-side sibling of `collect_invite_bootstrap`, §15.7 as originally written.
- **② ONE MORE KEY** — widen `collect_invite_bootstrap`'s authorization to admit a **retained former member**; the client selects a different anchor from the same served batch. **No new wire surface.**

**`D-121`, in order.** **① user-visible:** identical against a current node — she is back either way. **They differ only against an OLDER node, and there the difference is total:** under ① her session is dropped (§3); under ② she receives a `1011` refusal and falls back to today's behaviour — **which is the degradation property §15.7 claimed for ① and only ② delivers.** Mixed-version federation is the normal state of this protocol, not an edge case. **② tier consequence: NONE** — nothing is copied, nothing is destroyed, no `T4` durability floor is touched, no erasure fate is imposed on another party. **③ resource:** ② is one authorization clause plus one client selection change; ① is a wire variant, a serde rename, a node arm, a client function, a ch3 registry row and a capability gate the client would have to consult before daring to ask.

🎯 **CHAT RECOMMENDED ②. 🔒 JOE RULED ② (2026-08-24).**

⚠️ **THE CAVEAT, NAMED AND NOT TRADED AWAY.** Under ② the wire string `transport.invite_bootstrap_request` serves a requester who **holds no invite**. ***A name narrower than the thing it describes, reused as if complete, is this project's most-repeated defect class***, and a wire string is permanent and federated. 🔒 **THE REPAIR IS A SPEC-LEVEL RESTATEMENT OF WHAT THE VERB MEANS — *bootstrap for someone entitled to enter* — NOT A WIRE RENAME**, and it lands in `G-3`'s own DoD alongside the doc comment at `wire/types.rs:168`, which today describes the invitee case only.

📌 **FILED, NOT FOLDED: transport-level unknown-variant tolerance.** A unit `Unknown` arm under `#[serde(other)]` so an unrecognised control message is **ignored rather than fatal**. **It is the `H2` capability question named at J-601**, it is larger than this leg, and **it is what would make ① safe.** ⚠️ **It must not ride Leg G** — a transport-layer tolerance change inside a membership leg would make the diff argue two cases at once. **It gets its own node; the pointer is written so it is not dangling.**

---

## §5 — 🔒 THE LEGS. **THE SPLIT IS CHAT'S SEAT (`D-123`); THE SCHEDULING AND THE WIRE SHAPE ARE JOE'S AND ARE RULED ABOVE.**

📌 **The split handed to Joe at session open had five legs. §2.1's finding added one and forced the ordering.** Reported, not absorbed.

| leg | content | gated on |
|---|---|---|
| **G-0** | ✅ **THIS FILE**, plus §7's record repairs, committed atomically under `D-074` | — |
| **G-1** | ✅ **SHIPPED J-775. `cargo` 1641 → 1644 / 0 / 62 × 56 SUITES, re-driven independently by both seats.** The gate's predicate gained the former-member conjunct in `apply_join`'s own spelling; **a departed member, a kicked member and a DM party who left can all now return, and a banned one is still stopped upstream by the pre-check.** 🔑 **THE LEG WAS SPECIFIED SMALLER THAN THE OBVIOUS VERSION BECAUSE TWO SITES WERE OPENED INSTEAD OF REASONED ABOUT** — the banned pre-check already runs FIRST (`runtime.rs:1523` above `:1580`), and `apply_kick` marks without banning. 🛑 **THREE DEVIATIONS, ALL CORRECT, TWO OF THEM DEFECTS IN THE RUNBOOK: §3's comment point 4 was false on both paths and was refused before it reached the source; `V-5` named a control living outside §1's scope and was therefore unfalsifiable as written.** Both annotated at their sites (`D-131`). Runbook `tasks/RUNBOOK_SPACE_ADMISSION_LEG_G1.md` **v1.2 COMPLETED**. | G-0 |
| **G-2** | ✅ **SHIPPED J-776. `cargo` 1644 → 1648 / 0 / 62 × 56 SUITES, re-driven independently by both seats.** The node now refuses a rejoin it cannot anchor instead of answering `Accepted` and letting the fold drop it. 🔑 **THE LEG INVENTED NO PREDICATE — `ingest_event` ALREADY COMPUTED THIS BOOLEAN at the SR-D1 conflict gate and then rebuilt the Space without her, having already replied; `G-2` moved the existing predicate onto the ANSWER PATH.** 🛑 **AND THE REVERTED REPLY IS WORSE THAN THE DESIGN PREDICTED: `V-5` observed `Accepted { new_joiner: Some(…) }` with `is_member() == false` — not merely a yes, but a POSITIVE IDENTITY ASSERTION about a person the fold had already dropped, which a client could render, cache or announce.** **AND IT SHIPPED THE ch3 §3.6.10.10 `3048` ROW IN THE SAME CHANGE**, with the *reserved / not yet live* sentence **quoted rather than deleted**. 🔑 **That row was assigned to `G-5` when this table was written, and §3.6.10.10 overrides the assignment in its own words: *a wire code is allocated in this table in the same change that first emits it*, with the `3046` incident as the reason — a table showing a gap at a number already in production sends the next allocator onto a live code.** 📌 ***`C-8`'s shape again: a register that exists, is authoritative, and is not consulted at the moment of allocation.*** 🛑 **LIMIT, NOT SOFTENED: nothing ran against a live node, a wire, or a second identity — `3048` has never been observed on a wire, and that bound is `G-4`'s.** Runbook **v1.2 COMPLETED**. | G-1 |
| **G-3** | ✅ **SHIPPED J-777. `cargo` 1648 → 1654 / 0 / 62 × 56 SUITES, re-driven independently by both seats on FORCED REBUILDS.** `collect_invite_bootstrap`'s authorization now admits a retained departed member who is **not banned**; 🔒 **§3 RULED ② (Joe, 2026-08-26): she is served ONLY HER OWN membership events**, because `D-154`④ ruled what a RETURNING member receives and **a requester admitted by the gate but not yet rejoined is not yet returning.** 🛑 **THE FINDING THAT SHAPED IT: the invite requirement at `fanout.rs:751` was doing TWO JOBS — `banned` has zero occurrences in `fanout.rs`, the transport arm defers all authorization to that function, and the dispatch pre-check guards event submission not transport requests, so a banned identity was excluded ONLY as a side effect of holding no invite.** ⇒ **the ban term is REQUIRED here, the exact inverse of `G-1` where it would have been a second source of truth** — ***the same clause is a defect in one leg and a requirement in the other, and the only way to tell which is to open the path.*** 🔑 **AND THE LEG'S OWN FINDING: a naive `sender || target` union would have leaked exactly what ② withholds, because a kick SHE issued names her as `sender` and a third party as `target`** — ***the union asks the requester's question (does this event mention her?) where the ruling asks the event's (whose removal does this event disclose?), and the two agree on every event except the one that matters.*** 🔒 **AND JOE RULED THE SHADOW CORRECT (2026-08-26): §4's prose/sketch fork resolves to the SKETCH.** A former member holding an EXPIRED invite is refused — **because `D-154`① makes the invite the CARRIER OF THE ROLE, and because the `3044` gate at `runtime.rs:1806` is not conditioned on the rejoin flag, so the `OR` would have opened a door onto a locked gate.** 🎯 **Chat had recommended the `OR` and was wrong; recorded as wrong (`D-065`).** 🔓 **What survives is a STRING, not a predicate — `1011` at the door reads exactly like a stranger's refusal; filed for `G-5`.** 🛑 **LIMIT: `G-3` alone changes nothing a user can see — every assertion is about what the NODE SERVES.** Runbook **v1.3 COMPLETED**. | G-1 |
| **G-4** | **THE CLIENT ANCHOR SELECTION.** `ops::join` anchors on her own last `membership.*` event from the served batch instead of on an invite that does not exist; refusal or absence falls back to today. | G-3 |
| **G-5** | **CLOSE.** ~~ch3 registry row for `3048`,~~ ROADMAP, JOURNAL, `CLAUDE.md`, task docs — one commit, `D-074`, `roadmap-format-gate.ps1` exit 0. ⚠️ **The struck item SHIPPED AT `G-2` instead, on §3.6.10.10's own allocation rule; it is struck rather than deleted so the re-assignment is visible to a reader who remembers this table.** | all |

🔒 **ORDERING IS FORCED, NOT PREFERRED: `G-2` BEFORE `G-3`/`G-4`.** The safety net exists before the success path is trusted. **`G-1` alone makes rejoins reachable and therefore makes the accept-then-drop reachable in production for the first time** — shipping `G-1` without `G-2` would widen a live silent failure. ⚠️ **`G-1` and `G-2` may share a commit; they may NOT ship in the other order.**

🔓 **The leg list is Joe's to cut. `G-3`/`G-4` are the descope boundary: stopping after `G-2` leaves a Space that admits her and a node that refuses her honestly when it cannot anchor — a defensible resting state, and the option ① Joe was offered at session open.**

---

## §6 — WHAT THIS LEG MUST NOT DO

1. 🛑 **It must not add a `transport.*` variant.** §4 ruled ②.
2. 🛑 **It must not add `#[serde(other)]` to `TransportMessage`.** Filed at §4; its own node.
3. 🛑 **It must not amend ch3 beyond the reject-code registry row and the verb's meaning sentence.** The standing *must not amend ch3* ruling (J-739) governs everything else.
4. 🛑 **It must not touch `collect_sync_history`'s `is_member` gate.** A former member is not a member; `D-154`④ governs what she reads, and `E-2` already built it.
5. 🛑 **It must not mint a was-a-member READ grant.** `§9` item 7 of the master Phase-0 binds unchanged: leaving SUSPENDS access, a consented rejoin RESTORES it.
6. 🛑 **It must not re-adjudicate a federated join.** The gate block is already `origin == LocallySubmitted`; the structural skip must stay structural, not become a second condition.
7. 🛑 **It must not fix `is_structural_bootstrap_type`'s membership set.** It is already right for this leg.

---

## §7 — RECORD CORRECTIONS OWED. **CHAT'S SEAT — NO RULING REQUIRED.**

| # | correction | where |
|---|---|---|
| **R-1** | *"AND `3048` RIDES THIS LEG"* on the Leg D row — **it did not.** `3048` has zero `.rs` occurrences; J-762, Leg D's own design entry, never mentions it; J-763 closed Leg D as three of four gates without it. **It fell out at design time and nothing swept the claim.** | `tasks/M_SPACE_ADMISSION_PHASE0.md` §12, Leg D row |
| **R-2** | *"it may slip without leaving a silent failure behind, because `3048` already made the residue loud"* — **the licensing sentence for Leg G's slippage rests on a code that is not live.** | `tasks/M_SPACE_ADMISSION_PHASE0.md` §12, Leg G row · `CLAUDE.md` PLAY · J-765 (historical, NOT rewritten) |
| **R-3** | §15.7's *"an older Node that cannot answer ⇒ the client falls back to exactly today's behaviour"* — **false as measured (§3).** | `tasks/M_SPACE_ADMISSION_PHASE0.md` §15.7 |
| **R-4** | §15.4's predicate names three terms; the shipped gate has two (§2.1). **The section is not wrong — the CODE is — and the row must say which.** | `tasks/M_SPACE_ADMISSION_PHASE0.md` §15.4 |
| **R-5** | The ROADMAP node cites `tasks/M_SPACE_ADMISSION_PHASE0.md` **v2.8** in two places; the file is **v2.9** since J-772. | `docs/ROADMAP.md` M-SPACE-ADMISSION node |

📌 **All five are annotated at the site per `D-131`, none deleted.** 🛑 **J-765's text is NOT rewritten** — a journal entry is a historical record of what was believed on the day, and `D-131` governs citations proven broken, not history.

---

## §8 — FLOORS

🔒 **CARRIED BY SCOPE, NOT RE-RUN — this pass wrote zero `.rs`, zero `.ts`, zero `.svelte`, zero `ui/**`:** cargo **1641 / 0 / 62 × 56 SUITES** · vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15**. 🛑 **Sampler catalogue UNMEASURED** — its harness has never been located; **no number is written for it.**

📌 **`cargo` is not a floor for a reads-only pass**: an identical result over zero `.rs` is a scope argument, not a measurement.

🔒 **G-1 AND G-2 MUST MOVE THE COUNT.** A gate arm and a reject code that leave `1641` unchanged mean the tests were not written. **The delta is MEASURED with `--skip` on the new test names, never arithmetic** — the discipline every leg of this arc has used.

---

## §9 — DoD FOR LEG G-0

- [ ] This file exists at `tasks/M_SPACE_ADMISSION_LEGG_PHASE0.md`, header correct, `Status: ACTIVE`, LF-only, no BOM, verified on disk
- [ ] §7's five corrections applied at their sites, annotated not deleted; J-765 untouched
- [ ] The `M-SPACE-ADMISSION` ROADMAP node's Leg G row states the six legs, the ② ruling and the `3048` correction; `roadmap-format-gate.ps1` returns exit 0
- [ ] `CLAUDE.md` PLAY head updated; CRLF integrity re-asserted (CR count equals LF count, zero CRCR, zero lone LF, no BOM) on both CRLF files
- [ ] JOURNAL entry written; ROADMAP version bumped — **one commit, `D-074`**
- [ ] Every open item in this file is either ruled at its site or explicitly filed with a named owner

📌 **"Commit pushed" is deliberately not a DoD item** — `Status: COMPLETED` in this header is the shipped signal, and this file stays `ACTIVE` until Leg G closes at `G-5`.
