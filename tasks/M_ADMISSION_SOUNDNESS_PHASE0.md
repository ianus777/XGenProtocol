# M-ADMISSION-SOUNDNESS Phase-0 — driving the ruled admission mechanism as one lived story
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-27  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS FILE IS

Joe's ask, verbatim in substance: **simulate some real situations related to the mechanism, and find out whether its logic is sound.**

`M-SPACE-ADMISSION — who may join a Space, and how a leaver comes back` closed at J-779 with six ruled clauses (`D-154`), an admission model (`D-148`), and every leg gate green. 🔑 **But each leg proved its own term in isolation. Nobody has ever driven the whole story as one sequence and looked at what a person actually meets.**

🛑 **THIS MILESTONE ASSERTS NOTHING IT HAS NOT FIRST OBSERVED.** Its deliverable is a **transcript with a verdict per situation**, not a test suite. Whether any of it becomes a standing gate is a decision taken **after** the observations exist, not before.

📌 **Instrument RULED (Joe, 2026-08-27): option (a)** — a driver in `xgen-mptest`, on the harness that already works, rather than a throwaway PowerShell rig. ⚠️ **The rejected option was cheaper to start and would have been a SECOND instrument; this arc carries three notes (`N-206`, `N-207`, `N-212`) about instruments that lie while looking correct.**

---

## §1 — 🛑 THE GROUNDING PASS ALREADY ANSWERED PART OF THE QUESTION, AND IT DID NOT NEED A SIMULATION TO DO IT

**Two of the mechanism's central clauses have NO EMITTER ON ANY SHIPPED SURFACE.** Measured at `bbd8b3b` by exhaustive grep of the identifier across every `.rs` in the workspace (excluding `target\` and `.claude\worktrees\`).

### ① `membership.kick` — `D-154`② says *kick is remembered*. Nothing can kick.

`EventType::MembershipKick` is **complete on the receiving side**: parsed (`wire.rs:189, :287`), permission-mapped (`exchange.rs:815`), validated (`exchange.rs:897`), resolved with its own precedence (`algorithm.rs:90, :134-142`), state-keyed (`state_key.rs:65`), applied (`state.rs:786 → apply_kick`), disclosed (`fanout.rs:624, :708, :754`) and audited (`protocol_audit.rs:121`).

🛑 **AND EVERY SITE THAT CONSTRUCTS ONE IS A TEST FIXTURE.** No client verb — `ClientCommand`'s 27 variants include `Ban`, **not `Kick`**. No node admin op. ⇒ ***a clause Joe ruled, about an event no user and no operator can cause.***

### ② `state.space_admission` — `D-148` says admission is owner-settable. Nothing can set it.

Same shape, and it goes further. `StateSpaceAdmission` is parsed, content-typed (`wire.rs:728`), state-keyed (`state_key.rs:111`), applied (`state.rs:799 → apply_space_admission`), and **gated** — `3047 admission_required` is live at `runtime.rs:1695`. Leg C's `known_variants()` completeness guard counts it (`wire.rs:853`).

🛑 **No emitter, AND `create-space` carries no `admission` argument** — zero matches for `admission` in the whole client clap surface. ⇒ ***every Space that has ever been created is `open` by `DEFAULT_ADMISSION`, and the `3047` gate cannot fire outside a unit test.***

### What this means, stated carefully

🔑 **THE LOGIC IS SOUND AND THE SURFACE IS ABSENT.** The milestone built a complete, correct, well-tested RECEIVING half for two clauses and no SENDING half for either. Nothing is wrong; **two things are missing, and the milestone closed without either being visible**, because every leg gate constructed its own events directly and never needed a verb.

⚠️ **AND THIS IS THE ARC'S OWN LESSON RECURRING AT MILESTONE SCALE.** `N-194`/`D-156` shape: *a check whose failure mode reads exactly like success is not a check.* Here: **a clause whose only exerciser is a fixture reads exactly like a shipped clause.** Every test passes. The floor is green. And two of six ruled behaviours are unreachable.

📌 **`membership.node_eject` is NOT in this category** — `admin_ops.rs:4169 space_force_eject` and `:4256 space_unban` are real node admin ops with live-fanout tests. Clause ⑥ is reachable.

---

## §2 — GROUNDING: THE INSTRUMENT

| | measured |
|---|---|
| Harness | `xgen-mptest` — 32 test files, `ManagedProcess::init_and_spawn_node` / `init_and_spawn_client` / `spawn_client_reusing_keypair`, `AicontrolClient` over JSONL pipes |
| Proven cost | `mp_g4_rejoin_e2e` **2 scenarios, 33.07 s** (39 s wall incl. rebuild), re-driven green at `bbd8b3b` this session ⇒ **~16 s per scenario** |
| Gating | `#[ignore = "heavy: …; box-gated RUN"]`. **64 ignored workspace-wide across essentially every mp file** ⇒ the whole e2e layer is opt-in and contributes zero to the `cargo` floor |
| Oracle | `is_present()` drives the `members` verb, which re-derives through the node's own `derive_resolved` ⇒ it answers *did this SURVIVE RESOLUTION*, not *did the node say Accepted*. **That distinction is what `3048` exists to draw and it is the right oracle for this milestone.** |
| Client verbs available | 27, of which useful here: `register` `create-space` `create-room` `invite` (`--role`) `join` `leave` `ban` `members` `rooms` `history` `send` `fetch` |
| Node admin ops | `space_force_eject` · `space_unban` |
| Clock control | `mp_r1_clock.rs` / `m9_2_f3_clock.rs` exist ⚠️ **their reach over invite `valid_until` is UNVERIFIED and is Leg 0's to establish** |

### 🛑 THE INSTRUMENT'S HARD BOUND, STATED BEFORE ANYTHING IS BUILT

**The harness sees the WIRE, not the SCREEN.** It reads JSON replies from `--aicontrol`. It can say *what is served to her*; it cannot say *what she sees*. ⇒ For situations **3** and **4** — the two where the ruled behaviour is an EXPERIENCE — **the driver takes it to the payload and the last step needs Joe's eyes on a running client.** 📌 *Better said now than discovered in a transcript that quietly stops short.*

⚠️ **And the desktop client has NO ROUTE TO `ops::join` AT ALL** (J-740) ⇒ a rejoin cannot be performed from the GUI even manually. **For situations 3 and 4 the observation is: drive to the departed state via the harness, then look at the client.** Whether that is even stageable is Leg 0's, not assumed here.

---

## §3 — THE SITUATIONS, WITH DRIVABILITY MEASURED

| # | the situation | clause | drivable? |
|---|---|---|---|
| **S-1** | She leaves in the morning, comes back that evening | baseline | ✅ **exists** — V-9a |
| **S-2** | An **admin** leaves and returns with no invite | `D-154`① presence not position | ✅ **DRIVABLE — RESTATED AT LOCK.** ~~*needs a post-join promotion verb*~~ was wrong: **no role-mutation event exists in the protocol at all**, so nobody is ever promoted. But `apply_join`'s no-invite arm (`state.rs:1275`) hard-codes `Role::Member` ⇒ alice invites bob as **admin**, bob joins, leaves, rejoins on his own anchor, and **comes back a plain Member.** 🔑 *A returning admin is silently demoted and nothing tells her* — that is clause ① working exactly as ruled, and it is the sharpest thing in the list to look at. |
| **S-3** | She returns and looks at the gap | `D-154`④ content closed, **structure open** | ✅ wire · ⚠️ **screen needs Joe** |
| **S-4** | She was in four rooms; she returns | `D-154`⑤ rooms not restored | ✅ wire (`rooms`) · ⚠️ **screen needs Joe** |
| **S-5** | She is kicked and tries to come back | `D-154`② kick remembered | 🛑 **NOT DRIVABLE — no emitter (§1①). Carried to `M-ADMISSION-SURFACE`.** |
| **S-6** | Kicked, then banned, then tries | `D-154`③ ban follows kick | 🛑 **HALF — `ban` yes, `kick` no. Carried to `M-ADMISSION-SURFACE`.** |
| **S-7** | Node-ejected, then tries; then un-banned | `D-154`⑥ node-eject follows kick and ban | ✅ node admin ops |
| **S-8** | A stranger asks · and she asks with a lapsed invite | `1011` indistinguishability | ✅ **BOTH DRIVABLE — MEASURED AT LOCK.** ~~*lapsing an invite needs clock reach*~~: `invite --valid-for-days` is `Option<u32>` and `ops.rs:1187` computes `valid_until = Utc::now() + days` with **no lower bound** ⇒ **`--valid-for-days 0` stamps an invite that is already spent.** No clock harness, no `3045` collision (that gate is the ceiling). ⚠️ **Leg 1 must OBSERVE the refusal, not assume it** — `valid_until == now` is a boundary, and a short sleep before the join makes it unambiguous. |
| **S-9** | She reinstalls on a new laptop | `M-CLIENT-RESTORE`'s ground truth | ✅ **exists** — V-9b |
| **S-10** | Admission `open` vs `invite`, same approach | `D-148` | 🛑 **NOT DRIVABLE — no emitter, no create flag (§1②). Carried to `M-ADMISSION-SURFACE`.** |

🎯 **S-3, S-4 and S-8 are where Chat expects the surprises**, and the reason is the same in all three: **the RULE is defensible and the EXPERIENCE has never been looked at.** S-8 especially — you ruled `1011` yesterday on the oracle argument, and *watching it* is not the same as *ruling it*. **If seeing it changes your mind, that is a legitimate outcome of this milestone, not a failure of it.**

---

## §4 — 🔒 RULED, AND ONE STILL OPEN

**Q-1 — 🔒 RULED **B** (Joe, 2026-08-27): simulate the eight that are reachable; file the two missing surfaces as their own milestone; the transcript names the holes in its own §1, never in a footnote.**

⇒ **`M-ADMISSION-SURFACE` — the verbs that cause a kick and set a Space's admission** (name checked corpus-wide 2026-08-27: **0 occurrences**). It carries **S-5**, **S-6**'s missing half and **S-10**. 🔑 **It is NEW PRODUCT, not verification** — a kick needs its own design and its own ruling about what it looks like to the person kicked, which is why folding it in here would have turned a soundness pass into a feature arc. 📌 **Neither verb is scheduled; both are FILED.**

⚠️ **THE CARRIED COST OF B, NAMED SO IT IS NOT MISTAKEN FOR OVERSIGHT:** this milestone will ship a transcript with two ruled clauses it could not stage. ***A reader a year out must be able to tell a hole that was found from a hole that was missed.***

**Q-2 — 🔓 STILL OPEN: is this scheduled now, or filed and parked?** Chat has no view worth stating; it is entirely what you want next. 📌 Proceeding as **scheduled** on the strength of the Q-1 ruling; say the word and it parks. The alternative on the table is `M-RP-LIVEFEED-REFRESH`, the only half-built arc on the board, which needs one word: **B1, B2 or B3**.

### The old text of this section, kept per `D-065`

Chat offered three arms — **A** build the two verbs first, **B** simulate the reachable eight and file the rest, **C** records-only annotation — and recommended **B** on the two lenses above. ✅ **Joe ruled B.** ❌ A and B-as-C are refused and recorded as refused.

---

## §5 — PROPOSED LEGS. **THE SPLIT IS CHAT'S SEAT (`D-123`); EVERY RULING IN IT IS JOE'S.**

| leg | content | gated on |
|---|---|---|
| **0** | ✅ **COMPLETE — all three measurement items closed at lock, see §3 and §8.** | — |
| **1** | `mp_admission_soundness.rs` — the reachable wire situations: **S-1, S-2, S-7, S-8**. **Each prints a labelled verbatim transcript; assertions only where a clause is unambiguous.** | ✅ Q-1 = B |
| **2** | S-3 and S-4 driven to the payload, plus a staged client for Joe. **The one leg whose verdict is not Chat's to write.** 🛑 **DESIGN CONSTRAINT, MEASURED: the rig must be left ALIVE.** The GUI attaches to a data dir the harness drove, and the node has to still be running when it does — so Leg 2's driver cannot be a test that tears its rig down at the end. | Leg 1 |
| **3** | The verdict document — one section per situation: what she did, what the system did, what she saw, **what reads wrong**. No recommendations; findings routed to rulings. | Legs 1-2 |
| **4** | Records + close. Whether any scenario becomes a standing gate is decided HERE, from evidence, not at Leg 0. | Leg 3 |

---

## §6 — WHAT THIS MILESTONE MUST NOT DO

1. 🛑 **It must not change protocol, node or client behaviour.** It observes. A defect it finds is a finding routed to a ruling, **never patched under this milestone's banner** (Rule 6's shape at milestone scale).
2. 🛑 **It must not build the missing kick or admission verbs.** Q-1 ruled **B** ⇒ they are `M-ADMISSION-SURFACE`'s, filed and unscheduled.
3. 🛑 **It must not assert a clause it has not observed**, and must not quietly narrow a situation to the part the harness can see.
4. 🛑 **It must not imply Joe's eyes were on something they were not.** Leg 2's verdict is his or it is absent.
5. 🛑 **It must not re-open `D-148`, `D-154` or `D-156`.** They are the subject, not the question.
6. 🛑 **It must not add to the `cargo` floor without saying so** — every scenario is `#[ignore]` box-gated like its 32 siblings unless a later ruling changes that.

---

## §7 — FLOORS

🔒 Carried at close of `M-SPACE-ADMISSION` (J-779), commit `bbd8b3b`: cargo **1667 / 0 / 64 × 57 SUITES** · vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15**. 🛑 **Catalogue UNMEASURED — no number is written for it.**

📌 **Expected movement: `+1 SUITE`, `+N ignored`, `+0 passed`** — a box-gated binary contributes zero. ⚠️ **A soundness milestone that MOVES the passing count has quietly become a test milestone; if that happens it is a finding, not a success.**

---

## §8 — DoD FOR PHASE-0 (LEG 0)

- [x] §1's two no-emitter findings independently re-driven by opening each cited site, not by re-running the grep. ✅ **`apply_kick` opened at `state.rs:1323-1348`** — the `D-154`② comment and `mark_departed` are both there, receiver complete. ✅ **`apply_space_admission` opened at `state.rs:968-989`** — DM bar, role predicate (explicitly not identity-equality), verbatim store. **Both receivers exist and are careful; neither has a caller outside a fixture.**
- [x] V-9a and V-9b's assertions audited and mapped to `D-154` clauses (`D-071`). ✅ **They assert member-after-join, not-present-after-leave, rejoin `is_ok()`, present-again — through the node's own `derive_resolved`.** ⇒ **clause ①'s PRESENCE half and nothing else**; position, kick, ban, the gap, rooms and node-eject are asserted **nowhere**. 📌 Chat's earlier *"roughly one and a half of six"* was generous; measured, it is **half of one**.
- [x] S-2's promotion verb: exists or does not — measured, named either way. ✅ **MEASURED: no role-mutation event exists in the protocol at all** (`wire.rs` carries eight `membership.*` and no role verb) ⇒ the situation was RESTATED rather than dropped, and is drivable with shipped verbs.
- [x] S-8's clock reach over invite `valid_until`: measured, and if absent, S-8 narrowed **explicitly** rather than silently. ✅ **NOT NEEDED — `--valid-for-days 0` is sufficient (§3).**
- [x] S-3/S-4 stageability for Joe's eyes: measured against J-740's no-GUI-join finding. ✅ **STAGEABLE.** `desktop.rs:988` resolves the data root by the same precedence as the CLI — `--data-dir` > `XGEN_DATA_DIR` > platform default (`D-067`) ⇒ the harness drives the rejoin over `--aicontrol`, then the GUI is launched on **that same instance dir against that same live node**, and Joe looks. 🛑 **The rig must be left alive — recorded as Leg 2's design constraint in §5.**
- [x] Q-1 ruled; this file → `ACTIVE`. 🔓 **Q-2 remains open and is sequencing only — it does not gate Leg 0.**
- [x] ROADMAP node filed with a short descriptive title; name collision re-checked at file time (**checked 2026-08-27: `M-ADMISSION-SOUNDNESS`, `M-ADMISSION-SURFACE`, `M-CLIENT-RESTORE` — 0 occurrences each**). ✅ **ROADMAP v7.65, gate exit 0. 📌 `M-CLIENT-RESTORE` was ruled at J-779 and had been OWED ITS OWN NODE ever since — filed here rather than left living inside a closed milestone's `Owes` list.**

📌 **LEG 0 IS COMPLETE.** Every item measured, and **two of Chat's own three unknowns turned out to be wrong rather than merely unknown** (S-2 and S-8) — recorded in §3 at their sites, not silently corrected.

---

## §9 — 🔓 OPEN DECISIONS INSIDE THIS FILE

**One — Q-2 in §4, and it is sequencing only.** 📌 Stated as a count so a reader can tell the difference between *nothing was open* and *nobody looked*.
