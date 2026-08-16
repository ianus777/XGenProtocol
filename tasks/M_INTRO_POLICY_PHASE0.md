# M-INTRO-POLICY Phase-0 — receiver-side render policy: the mechanism was named, its INPUT was never measured
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-16  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS FILE IS

🎯 **THIS IS A PHASE-0 (`D-071`), NOT AN EXECUTION.** Audit → design → runbook → implement. **NO CODE. NO
RUNBOOK. NOTHING IS BUILT FROM THIS DOCUMENT** until §4's open decisions are Joe-locked and a runbook is
authored against the rulings.

📌 **PROVENANCE.** `M-INTRO-POLICY — receiver-side render policy` was FILED at **J-701** with its design
banked into the `docs/ROADMAP.md` node's `Owes:`. Its trigger — *"M-RP-INTRO lands"* — **FIRED at J-735
(commit `3e1014d`)** and the node was marked *"Phase-0 is OWED; a trigger that has fired with no Phase-0 is a
defect."* **This file clears that standing defect.** *It is exactly how `M-RP-INTRO` itself started, one
milestone earlier — the second instance of one shape, and the reason the ROADMAP now writes the defect into
the node rather than leaving it to be noticed.*

🛑 **AND THE HEADLINE FINDING IS THAT THE MILESTONE'S STATED INPUT DOES NOT REACH THE RECEIVER.** The node's
`Owes:` says *"auth tier is an INPUT to the policy, not the mechanism"*, and treats the obstacle as
*"Tiers 2 to 4 need qualified institutions and do not exist"*. **That is true and it is not the binding
constraint.** §3 measures a second, independent gap: **the sender's tier is not on the wire at all** —
`identity.record`, the only identity lookup response the protocol has, carries **eight fields and no
`trust_assertion`**, while the stored record carries twelve. ⇒ **even if Tier 2 shipped tomorrow, a receiver's
client could not read a sender's tier.** *The conclusion the node draws — a Tier-2-plus gate excludes
everyone today — survives; the reason it gives is the weaker of two, and the stronger one was never written
down.*

🔑 **v1.1 — AND THE DESIGN WALK THAT FOLLOWED v1.0 MOVED MORE THAN THE AUDIT DID.** Joe read §3, and in one
sitting the milestone gained a **plane it did not have**: an ACCEPTANCE gate on the DM invite, which reads the
initiator's IDENTITY rather than the message's CONTENT, is therefore enforceable **at the node, permanently**,
and **does not inherit `D-143`'s expiry date**. §3a is that walk, measured; **Q8 / Q9 / Q10** are its forks;
two locks came out of it. ⚠️ **Three of v1.0's own statements are corrected in place rather than annotated —
this file has never been committed, so `D-131` does not attach and there is nothing a reader could have been
misled by.** *The corrections are named at their sites so the change is visible, not silent.*

**State this file was written against:** `HEAD` `9cbbf54` **= `origin/main` by `git ls-remote`**, clean tree.

**Floors — stated, deliberately NOT re-run (this pass is reads only; zero `.rs`, zero `ui/**`, zero `.ts`):**
cargo **1602 / 0 / 62 × 56 SUITES** · vitest **172 / 172 × 9 FILES** · svelte-check **0 errors / 34 warnings /
15 files**, all carried from J-734's driven figures. 🛑 **The component catalogue is recorded UNMEASURED** —
its harness has still not been located, and a number that has not been driven does not enter this file.
🛑 **NO REGISTRY NUMBER IS CARRIED** (`N-184` / `N-190` / `N-194`: a registry count without its screen is
unusable as a floor). 🛑 **`cargo` IS NOT A FLOOR FOR A READS-ONLY PASS** — an identical result over zero
`.rs` is a scope argument, not a measurement.

🔒 **EVERY CENSUS IN THIS FILE EXCLUDES `\.claude\`, `\target\` AND `node_modules`** (`J-737`: eight stale
source trees hidden by `.gitignore:10` inflate a repo-wide `auth_tier` grep from 107 to 219 — **51%**, and the
inflated number is complete-looking, plausible and wrong). **Every count below states its file pool.**

---

## §1 — THE INHERITED DESIGN, AND WHAT SURVIVES THE AUDIT

`docs/ROADMAP.md`'s `M-INTRO-POLICY` node banks an argument, not merely a conclusion. Re-read at `9cbbf54`
and assessed line by line:

| # | the banked claim | verdict |
|---|---|---|
| **I1** | **the filter is ENFORCED IN THE CLIENT, not the node, and this is not a preference** — today a node can read DM content so node-side filtering would work, but after **PG-05** (MLS/E2E) the node holds ciphertext and cannot inspect, strip or rewrite anything, so a node-side filter ships with a known expiry date (`D-143`: the cheap route is unsound) | ✅ **STANDS, AND IT IS THE STRONGEST CLAIM IN THE MILESTONE.** It is `D-143` applied exactly as designated — the trigger is *unsoundness*, not effort. **A filter that stops working when encryption arrives is a guard whose failure mode is silence** |
| **I2** | policy is **AUTHORED at the receiver's home node** and **ENFORCED at render in the receiver's client** | ⚠️ **THE ENFORCEMENT HALF STANDS; THE AUTHORING HALF IS AMBIGUOUS AND §4 Q1 IS THAT AMBIGUITY.** *"Authored at the receiver's home node"* does not say **by whom** — the node's operator, or the user whose identity is homed there. `NodePolicy`, the cited precedent, is **operator** authority. **These are different products and the difference is not cosmetic** |
| **I3** | **`NodePolicy` is the precedent** — per-Space, node-held, admin show/set verbs | ⚠️ **STANDS AS A STORAGE-SHAPE PRECEDENT ONLY, AND MUCH MORE NARROWLY THAN THE WORD "PRECEDENT" IMPLIES — `G-1a` / `G-1b`.** The store is **INERT by its own module doc: nothing in the running Node reads it.** ⇒ **there is no enforcement path anywhere in this project to copy** |
| **I4** | **no policy surface reaches the client today, so a client-facing policy read is NEW WORK** | ✅ **STANDS, AND `G-5` MAKES IT SHARPER THAN STATED:** there is **no `policy.*` namespace on the wire at all** — 61 `TransportMessage` variants across nine namespaces, zero of them policy. **It is a new wire namespace, not a new field on an existing verb** |
| **I5** | **auth tier is an INPUT to the policy, not the mechanism**; Tiers 2–4 need qualified institutions and do not exist; `build_dm_space_create_event` hardcodes `auth_tier: 1` | ⚠️ **THE CONCLUSION STANDS ON A REASON THE NODE DOES NOT GIVE — `G-2b` / `G-3`.** Two separate gaps, both measured: **(1)** a Space's `auth_tier` is a **join-gate floor**, not a property of a sender, and in a DM it is a **constant 1 by construction**; **(2)** the sender's *own* tier lives in `trust_assertion`, which **`identity.record` does not carry** |
| **I6** | three open design questions from J-701 — the **default posture for the non-institutionally-homed** · **silent vs disclosed** (`D-065` argues disclosed and non-actionable) · **the compliance gap** against a patched client with no attestation subsystem | ✅ **ALL THREE STAND AND ALL THREE ARE STILL OPEN.** Carried into §4 as **Q2 / Q3 / Q4**, unchanged in substance, each now priced against measured ground |
| **I7** | inherited from `M-RP-INTRO-CANVAS`: **PROMINENCE ON UNSOLICITED FIRST CONTACT.** Joe's placement puts the canvas **between header and paragraph**, so the first thing a recipient sees from a stranger is the stranger's composed canvas. `I1` holds (message chrome, attribution directly above) — **whether that should render UNASKED, and under what policy, is this milestone's question** | ✅ **STANDS AND IT IS THE ONLY QUESTION HERE WITH A USER-VISIBLE SURFACE TODAY.** ⚠️ **It is also the only one that can be answered WITHOUT new protocol** — see §4 Q5 and §5's leg split |
| **I8** | `N-173` — the word **"Tier" already means two unrelated things**; do not add a third | ⚠️ **STANDS AND IS WORSE THAN FILED — `G-4a`.** The collision is no longer only in prose: **`data-tier` is a live HTML attribute in `ui/` meaning the widget tier, AND an attribute in template CSS meaning the auth tier.** *Latent, but the attribute name is already spent twice* |

🔑 **THE SURFACE QUESTION IS NOT RE-OPENED HERE.** `M-RP-INTRO` `I1` ruled the intro renders in **message
chrome, never system chrome** (`D-113` S-5), and the ROADMAP node's own framing — *"Joe's design, and it is
the one that unlocks system-chrome rendering safely"* — is the **motivation** for this milestone, not a
licence to pre-empt that ruling. 🛑 **NOTHING IN THIS FILE MOVES THE INTRO INTO SYSTEM CHROME**, and no leg
may.

---

## §2 — GROUNDING: THE PRECEDENT. **MEASURED AT `9cbbf54`.**

**File pool for every census in this section: 464 files** (`*.rs` `*.ts` `*.svelte` `*.js`, worktrees /
`target` / `node_modules` excluded), unless a narrower pool is stated on the row.

| # | fact | site |
|---|---|---|
| **G-1** | ✅ **`NodePolicy` — 47 occurrences across FIVE files, ALL node-side.** Definition **23** (`xgen-core/src/space/node_policy.rs`); readers **24** — `admin_ops.rs` 11 · `pipe.rs` 6 · `app.rs` 4 · `aicontrol.rs` 3, every one in `xgen-node/`. **ZERO in `xgen-client/`. ZERO in `ui/`.** *Independently confirmed twice before this pass (Chat J-737, Clair J-738) and re-driven here* | `xgen-core/src/space/node_policy.rs`, `xgen-node/src/{admin_ops,pipe,app,aicontrol}.rs` |
| **G-1a** | 🛑 **THE PRECEDENT IS INERT, AND ITS OWN MODULE DOC SAYS SO IN CAPITALS.** *"FORK X (NP-D3): the store is INERT this arc — **nothing in the running Node reads it**. Enforcement (an actionable auto-moderation reader) is deferred to the temperature-plugin arc; the two verbs (`space set-node-policy` / `show-node-policy`) are the **sole consumer**."* ⇒ **`NodePolicy` is a schema, a persistence path and two admin verbs. There is no enforcement reader in this codebase to copy** | `node_policy.rs:26-32` (module doc) |
| **G-1b** | 🛑 **AND IT MISMATCHES ON BOTH AXES THAT MATTER HERE. ① KEY: `policies: HashMap<SpaceXgid, NodePolicy>` — per hosted SPACE.** An intro policy is per **RECEIVER IDENTITY**. **② PRINCIPAL: it is "Node-operator authority (principal #1, the `force-eject` signer)", explicitly not owner (#3) and not AI-operator (#2).** An intro policy that filters what **I** see is a **user** preference | `node_policy.rs:81-88`; module doc `:8-16` |
| **G-1c** | ✅ **WHAT DOES TRANSFER, AND IT IS WORTH TAKING:** the **`absent == disabled`** prime invariant (NP-D2) — a missing entry and `{false, None}` are indistinguishable, so an empty store is today byte-for-byte; and the store **never invents a default record**, reporting presence/absence faithfully while the *default* lives in the verb. **That is `N-182` (reserve nothing) realised in a persisted store, and it is directly reusable** | `node_policy.rs:44-56`, `:76-80` |
| **G-1d** | ⚠️ **THE FILE-PATH CONVENTION IS ALSO REUSABLE** — `D-035` naming, `xgen-node_node_policy.json`, sibling store rather than a field on `SpaceState`, on the stated ground that operator-lifecycle state is kept out of the protocol-derived state object | `node_policy.rs:20-24` |

🔑 **THE HONEST SUMMARY OF THE PRECEDENT: IT PROVES THE PROJECT CAN STORE AND ADMINISTER A POLICY. IT PROVES
NOTHING ABOUT ENFORCING ONE, BECAUSE NOTHING HAS EVER ENFORCED THIS ONE.** ⚠️ *Naming it "the precedent"
without that qualifier is the arc's own species — a claim narrower than the thing it describes — and it
would have set a runbook looking for an enforcement path to mirror that does not exist.*

---

## §3 — GROUNDING: THE INPUT. **THE SECTION THAT CHANGES THE MILESTONE.**

| # | fact | site |
|---|---|---|
| **G-2** | ✅ **`auth_tier` — 107 occurrences, 25 files.** *Reproduces J-737's corrected true figure exactly (raw 219 with worktrees; 51% inflation)* | 464-file pool |
| **G-2a** | ✅ **THE DM/SPACE ASYMMETRY IS REAL AND REPRODUCED.** `build_dm_space_create_event(key, invitee, home_node)` — **three parameters, no tier** — writes the literal **`"auth_tier": 1`**. `build_space_create_event(key, name, description, auth_tier, home_node, jurisdiction, secure)` — **seven parameters**, tier threaded from `args.auth_tier`, which is a **real user-facing CLI flag** on `create-space`. **`CreateDmSpaceArgs` carries ONLY `invitee`** | `xgen-core/src/space/state.rs:1811-1829` (literal at **`:1825`**) · `:1382` · `xgen-client/src/ops.rs:669` · `:907` · `xgen-client/src/app.rs:986` |
| **G-2b** | 🛑 **AND THE ASYMMETRY IS NOT THE POINT — `auth_tier` IS THE WRONG OBJECT. IT IS A SPACE'S JOIN-GATE FLOOR, NOT A PROPERTY OF A SENDER.** `verify_tier_assertion(assertion_tier, space_auth_tier)` returns `TierMismatch` **iff `assertion_tier < space_auth_tier`** — a **slot contract checked at admission** (PG-13, wire **3030**). ⇒ **inside a DM, `auth_tier` is a constant `1` by construction and carries ZERO information about who is speaking.** **A receiver-side render policy cannot read it, because it does not describe the sender** | `xgen-core/src/auth/tiers.rs:158-168`, `:127-146` |
| **G-2c** | ✅ **THE TIER THAT VARIES IS `tier_verified`, INSIDE THE SENDER'S TRUST ASSERTION** — `Tier2Claims` / `Tier3Claims` / `Tier4Claims` all lead with it. **That is the input a policy would actually want** | `tiers.rs:70-118` |
| **G-3** | 🛑 **AND IT IS NOT ON THE WIRE. `identity.record` — the ONLY identity lookup response the protocol has — CARRIES EIGHT FIELDS: `protocol_version · identity_id · display_name · registered_at · devices · home_node · is_ai · ai_capabilities`. NO `trust_assertion`. NO tier. NO `revoked`. NO `update_version`** | `xgen-core/src/wire/types.rs:453-471` |
| **G-3a** | 🔑 **THE STORED RECORD CARRIES TWELVE (`M-RP-INTRO` `G-6`): `identity_id · display_name · is_ai · ai_capabilities · registered_at · trust_assertion · devices · home_node · update_version · revoked · revoked_at · revocation_reason`.** ⇒ **THE WIRE PROJECTION IS NARROWER THAN THE STORE, AND THE TIER IS ONE OF THE FOUR FIELDS DROPPED.** *This is not a gap someone forgot to fill; it is a projection someone chose. Widening it is a protocol decision about what an identity publishes about itself* | `xgen-core/src/identity/registry.rs:32` |
| **G-3b** | 🛑 **THE UNFED BRANCH IS ALREADY SHIPPED, IN PRODUCTION, ON EXACTLY THIS INPUT.** `xgen-client/src/address_book.rs` declares **`pub trust_assertion: Option<Value>`** and a reader **`trust_lapsed(&self, now) -> Option<bool>`** — and its own comments record the **J-582** finding: *"`identity.record` carries no `trust_assertion`, so this fires only on a SEEDED record"*, with the guard test asserting **`r.trust_assertion == None`, "an ordinary fetched record has no assertion"**. **21 hits in that file; every non-test value is `None`** | `address_book.rs:105-113`, `:145-160`, `:676-681`; `from_fetched` sets `trust_assertion: None` at `:135` |
| **G-3c** | 🔑 **⇒ THE POLICY'S NAMED INPUT HAS A FIELD, A READER, A TEST AND NO FEEDER. `N-091`'s shape at PROTOCOL scale, and it is measured rather than suspected** | derived from `G-3` / `G-3a` / `G-3b` |
| **G-4** | ✅ **`ui/` IS CLEAN OF ALL OF IT.** 145-file pool (`*.ts` `*.svelte` `*.js` `*.css`, `node_modules` excluded): **`auth_tier` 0 · `authTier` 0 · `trust_assertion` 0 · `trustAssertion` 0 · `NodePolicy` 0.** *`N-173`'s scoped claim re-driven and it holds* | `ui/**` |
| **G-4a** | 🛑 **`tier` RETURNS 106 IN `ui/`, AND EVERY LIVE-PATH HIT IS THE WIDGET AXIS — `data-tier="widget"`.** The **auth** axis survives only in **dead files**: `ui/backup/run_1.0/tokens.css` and `ui/templates/skeleton/tokens.css` carry a full `--xgen-color-tier-1…4` palette and **`.xgen-tier-badge[data-tier="1".."4"]`** rules (D-038 era). ⇒ **`data-tier` is ALREADY SPENT TWICE, on one attribute name, with opposite meanings** | `ui/common/lib/components/widgets/*.svelte` · `ui/templates/skeleton/tokens.css:199-220` · `ui/backup/run_1.0/tokens.css:88-264` |
| **G-5** | 🛑 **THERE IS NO `policy.*` NAMESPACE ON THE WIRE.** The `TransportMessage` / `IdentityMessage` census returns **61 renamed variants** across `bootstrap` · `dm` · `federation` · `identity` · `migration` · `mls` · `reputation` · `space` · `transport`. **Zero policy.** ⇒ a client-facing policy read is **a new namespace**, not a field on an existing verb | `xgen-core/src/wire/types.rs`, all `#[serde(rename = "…")]` |
| **G-6** | ✅ **THE SPEC AGREES AND SAYS IT PLAINLY:** *"in Phase 1, all Spaces are Tier 1, so Layer 2 never produces a winner in Phase 1 deployments."* | `docs/xgen_ch3_specification.md:3172` |
| **G-7** | ✅ **THE NAMESPACE GRAMMAR THIS MILESTONE WOULD INHERIT IS ALREADY IN THE SPEC:** *"the `xgen.` namespace is **reserved** for XGen Protocol specification use"*, with **versioning in the key** — the convention `M-RP-INTRO` locked as `xgen.intro.v1` | `docs/xgen_ch3_specification.md:363`, `:372` |
| **G-8** | 🛑 **AND `M-RP-INTRO`'s OWN DoD ITEM 5 DID NOT LAND — SEE §6 `R-1`.** `N-172`'s socket table still lists **three sockets and one `bodyExtras` tenant**; the shipped build has **four sockets and two tenants** | `ui/docs/xgen-ui-notes.md:3526-3534` |

### 🔑 WHAT §3 MEANS FOR THE MILESTONE, STATED ONCE AND NOT REPEATED

**The ROADMAP node reasons: Tiers 2–4 do not exist ⇒ a Tier-2-plus gate excludes everyone ⇒ the default
posture is an open question.** ✅ **True.** 🛑 **But there is a nearer wall, and it does not move when tiers
ship:** the receiver's client **cannot read a sender's tier at all**, because `identity.record` does not
publish it. ⇒ **`M-INTRO-POLICY` as banked cannot be built end-to-end without a protocol change to what an
identity publishes about itself** — which is **identity AND the wire**, `D-123`'s held-hardest pair, and
therefore **Joe's outright**.

⚠️ *This is not an argument for cancelling the milestone. It is an argument for splitting it — see §5. The
half that needs no new protocol is `I7`'s prominence question, and it is the half with a user on the other
end of it today.*

---

## §3a — GROUNDING: THE DM AS A PRIVATE SPACE, AND THE ACCEPTANCE PLANE. **MEASURED AT `9cbbf54`, 2026-08-16.**

📌 **PROVENANCE: this section exists because Joe walked v1.0 and every step of the walk was a measurement, not
an opinion.** *Five of the fourteen rows below correct or narrow something Chat had said in the same session.*

### §3a.1 — DM autonomy: what is already true, and what is not

| # | fact | site |
|---|---|---|
| **G-9** | ✅ **DM-ness is a DERIVED PROTOCOL FACT, not config.** `is_dm` is a `SpaceState` field set `true` **only** from the `state.dm_space_create` root. **No node setting can set or clear it** | `state.rs:194`, `:450`, `:566` |
| **G-9a** | ✅ **AND THE FIVE §3.16.1 CONSTRAINTS ARE IMPLEMENTED** — in `xgen-core`'s state machine, which the Node runs. ⚠️ **CORRECTS A CHAT FRAMING FROM THIS SESSION:** *"`is_dm` = 0 in `xgen-node/`"* is true and **must not be read as "the constraints are not enforced."** They are | `state.rs:605`, `:2839`, `:2886`; ch3 §3.16.1 |
| **G-9b** | 🛑 **BUT `is_dm` HAS ZERO HITS IN `xgen-node/`, PRODUCTION AND TESTS.** The node's **config / policy layer** — `NodePolicy`, `max_event_size`, storage, rate limits, fanout — **never asks whether a Space is a DM.** ⇒ **every general-Space node setting already applies to DMs today, silently.** **Autonomy is a carve-out to BUILD, not a property to PRESERVE** | `xgen-node/**`, 0 hits |
| **G-10** | 🛑 **THE SPEC DOES NOT SAY DMs ARE PRIVATE FROM MODERATION.** §3.16.1 lists **five** constraints — members 2 · Rooms 1 · federation disabled · invitations disabled · **visibility Private, meaning *not discoverable via Bootstrap Node directory***. **That is DISCOVERY privacy, not CONTENT privacy.** The opening *"private, bilateral nature"* is prose framing, **not a normative row** ⇒ **the autonomy Joe wants is a SIXTH constraint that does not exist: a spec amendment, NOT a conformance fix** | `docs/xgen_ch3_specification.md:4984`, `:4990-5004` |
| **G-10a** | 🛑 **AND THE COLLISION IS LIVE.** §3.7.13.6 has automated moderation instruct the **home Node** to issue signed `membership.kick` / `membership.mute` with `reason = auto_temperature`, **signed by the node operator's identity**. **Nothing in §3.7.13 scopes itself away from DM Spaces**, and in a two-member DM an auto-kick removes one of the two participants | ch3 `:2785-2797` |
| **G-10b** | 🛑 **§3.7.13 WAS WRITTEN FOR ROOMS AND SPACE ROLES, AND A DM HAS NEITHER MEANINGFULLY.** `member_temperature_visibility` defaults to **`moderator`** — *"moderator-or-higher role in the Space"* — and **DM Space state carries that field, initialised to the default, TODAY** ⇒ **the machinery is already present on DM state; it was never scoped away, only never contemplated** | ch3 `:2736-2766`; `state.rs:465`, `:581` |
| **G-10c** | 🔑 **THE SPEC ALREADY DRAWS THE CIPHERTEXT BOUNDARY, ONE SECTION OVER, AND IT SUPPORTS THE CARVE-OUT.** §3.7.13.4: *"The home Node enforces visibility… the client does not implement filtering."* **The exact inverse of `I1`.** Both are right: **temperature is `meta_atts` computed node-side and stays readable on ciphertext; message CONTENT does not.** ⇒ **the boundary is *what the node can still enforce after PG-05*, and the spec is already on the correct side of it in both places** | ch3 `:2767-2776` vs `I1` |
| **G-10d** | ⚠️ **THE TIER OMISSION IS REAL AND SHOULD BE MADE DELIBERATE.** §3.16.1 has **no tier row**; `CreateDmSpaceArgs` carries only `invitee`; the builder hardcodes `1`. ⇒ **DMs are ALREADY autonomous from tier requirements — by OMISSION, not by decision.** An institutional operator reading §3.16.1 could file this as a gap and "fix" it against the privacy rule | ch3 §3.16.1; `state.rs:1825`; `app.rs:986` |
| **G-10e** | 📌 **NOT READ THIS PASS, STATED RATHER THAN IMPLIED:** ch3 §3.7.13.1 / .2 / .7 / .8 and Ch6 §6.12.6. **A DM exclusion amendment must be checked against all five before it is drafted** | — |

### §3a.2 — The cut, the ban, and why the receiver cannot currently do either

| # | fact | site |
|---|---|---|
| **G-11** | 🛑 **THE DM'S OWNER IS THE INITIATOR — `owner_id: creator`** — and ch3 says `membership.ban` is *"sent by **admin or owner**"*. ⇒ **on unsolicited first contact the STRANGER is the owner and the RECIPIENT is a plain member. The one party who needs the cut cannot make it** | `state.rs:449`, `:559`; ch3 `:2451` |
| **G-11a** | 🛑 **AND THERE IS NO USER-LEVEL BLOCK AT ALL:** `blocklist` **0 hits** · `block_identity` **0** · no blocklist, no verb, no wire event. *Independently matches `M-RP-BLOCK`'s node, re-driven here rather than inherited* | 464-file pool |
| **G-12** | ✅ **BUT THE TERMINAL CUT ALREADY EXISTS STRUCTURALLY.** `membership.leave` applies via `apply_leave` at `state.rs:605` **with no DM guard**, and §3.16.1 **disables invitations** ⇒ **the counterpart cannot re-add you. Leave + invitations-disabled = TERMINAL, with no new event type.** What is missing is a **user-reachable verb and a UI**, not a mechanism | `state.rs:605`; ch3 §3.16.1 |
| **G-12a** | 📌 **NOT ESTABLISHED, AND LOAD-BEARING FOR ANY RUNBOOK:** whether a **1-member DM Space** has defined behaviour anywhere, and whether `ops.rs`'s two `MembershipLeave` sites are a reachable client op or test scaffolding. **Both need a read before Leg C-cut is scoped** | `ops.rs`, 2 hits |
| **G-13** | 🛑 **LEAVING CLOSES YOUR OWN READ PATH.** `collect_sync_history` opens `if !space.is_member(requester_id) { continue; }` ⇒ **the node keeps the events and stops serving them to you.** The terminal cut, as it stands, **terminates the victim's access to the record of what was done to them** | `xgen-node/src/fanout.rs:478`, `:485-487` |
| **G-13a** | 🔑 **AND THE PRESERVATION GAP IS RETRIEVAL, NOT STORAGE.** The node already persists Space event stores; `SyncRequest`→`HistoryBatch` is built, tested and **never issued by the GUI**; §5's **R4 sync-from-cursor replay is OPEN and in no leg**. Joe's *"the client is just reader-sender"* lock (J-598) means preservation **cannot** be client-side | `fanout.rs:478`; `M-RP-LIVEFEED-REFRESH` node |
| **G-14** | 🔑 **THE BAN'S HOME IS THE ADDRESS BOOK, AND IT FITS: `SeenRecord` ALREADY CARRIES BOOK-LOCAL FIELDS THE WIRE DOES NOT** — `update_version`, `revoked`, `trust_assertion`, each annotated *"the wire carries no…"*. A `banned` flag joins them with **no wire, no node, no spec change** | `address_book.rs:92-116` |
| **G-14a** | 🛑 **AND THE TRAP: THE EVICTION WOULD SILENTLY UN-BAN.** `evict_older_than` keys on `last_seen` with `T1_DEFAULT_RETENTION_DAYS = 182`. **A banned identity stops being seen by definition**, so their `last_seen` freezes and they become the **most** eligible record ⇒ **the ban expires precisely because it worked.** Cannot fire today (**zero production callers**), and `M-RP-PEOPLE`'s node already records that wiring it becomes owed. 🔒 **RULE: a ban record is not retention-eligible** | `address_book.rs:299`, `:59-66`; test module opens `:335` |

### §3a.3 — Retention: the mechanism is SHIPPED, and Chat's "race" framing was wrong

| # | fact | site |
|---|---|---|
| **G-15** | ✅ **RETENTION IS AN AUTH-MODULE PROPERTY AND IT IS IN PRODUCTION.** `ModulePolicy.erasability.retention: Retention { Erasable, Retained }` rides the **Trust Assertion**; `xgen-auth-module` sets **`Retained` for Tier 4, `Erasable` for T1–T3** | `xgen-common/src/trust_assertion.rs:162-192`; `xgen-auth-module/src/lib.rs:61`, `:81-84` |
| **G-15a** | 🔑 **AND "WHOSE TIER GOVERNS" WAS DECIDED IN M12.4, DELIBERATELY.** `resolve_redact_erasure` reads the **ORIGINAL CONTENT AUTHOR's** retention, **not the redactor's**, because *"retention is a property of the record (`D-093` c2)"*; only an explicit `Retained` blocks — **the legal-hold floor**. M12's **first production `Retention` reader** | `xgen-core/src/node/runtime.rs:134`, `:145`, `:608-639` |
| **G-15b** | 🛑 **⇒ TWO CHAT CLAIMS FROM THIS SESSION ARE WITHDRAWN, NOT SOFTENED.** ① *"whose tier governs is a race"* — **FALSE; it is decided per-record on the author.** ② *"a DM cannot express that it is a T4 conversation"* — **IRRELEVANT: retention rides the AUTHOR'S IDENTITY, per record, never the Space's `auth_tier`.** *Chat put the obligation on the wrong object; Joe's recall was right and the code says so* | derived |
| **G-15c** | 🔑 **AND IT SHARPENS `G-3` RATHER THAN CONTRADICTING IT — THE HALF v1.0 DID NOT STATE.** The retention read comes from the **identity registry, node-local**. ⇒ **THE TIER IS REACHABLE NODE-SIDE AND UNREACHABLE CLIENT-SIDE.** *That is exactly why retention works today and a client-side render policy cannot* | `runtime.rs:134` vs `G-3` |
| **G-15d** | ⚠️ **WHAT SURVIVES, REDUCED, AND IS ARC I'S NOT THIS MILESTONE'S:** `Retained` blocks a **redaction**; **crypto-shred destroys the KEY, not the record** ⇒ a T4 floor made of retained ciphertext with a destroyed key is `D-121` lens ② questions ① and ②. **And `Retained` creates no COPY at the institution** — a DM lives at the *initiator's* node, so a T4 **recipient's** institution still holds nothing | `D-093`, Arc I / PG-02 |

### §3a.4 — The acceptance plane: the option v1.0 did not contain

| # | fact | site |
|---|---|---|
| **G-16** | 🔑 **AN ACCEPTANCE GATE ALREADY HAS A PROTOCOL HOOK: DM CREATION SEEDS A `PendingInvite` FOR THE INVITEE.** **The recipient is PENDING, not a member, until they emit `membership.join`** | `state.rs:416-430` |
| **G-16a** | 🛑 **THE ONE MEASUREMENT THIS PLANE NEEDS BEFORE ANYTHING ELSE, AND IT IS NOT TAKEN: DOES THE CLIENT AUTO-JOIN A DM INVITE?** **If it does, a consent gate exists in the protocol and is being spent silently.** ⚠️ *Note `G-13`'s interaction: a non-joined recipient is not a member, so `collect_sync_history` would serve them nothing — which is circumstantial evidence for auto-join and NOT a measurement* | UNMEASURED |
| **G-17** | 🔑 **AN ACCEPTANCE GATE DOES NOT INHERIT `D-143`'s EXPIRY DATE, AND THAT IS THE WHOLE POINT.** `I1` moves the filter to the client because after PG-05 the node cannot read **content**. An acceptance gate reads the **initiator's identity tier**, from the registry — which `G-15a` proves the node already does. **Identity is not encrypted; content is.** ⇒ **it lives at the node permanently, is ENFORCEABLE rather than advisory, and CLOSES Q4 outright** | derived from `G-15a` / `I1` |
| **G-18** | ✅ **THE REQUIREMENT HAS AN ADDITIVE HOME AND IT WORKS AT T1.** `ModulePolicy` carries `erasability` **plus `extra: BTreeMap<String, Value>`** (mirrored on `Erasability` and `TrustClaims`) ⇒ a `dm_invite_required` key lands with **zero struct change**, the same additive-namespaced shape as `xgen.intro.v1`. **Every tier's Auth Module issues a `ModulePolicy`, so T1 gets it free** | `trust_assertion.rs:162-180` |
| **G-18a** | 📌 **NOT VERIFIED:** whether `extra` is `#[serde(flatten)]`. **Needs a read before a runbook** | — |
| **G-19** | 🛑 **CORRECTION — `space.join_request` IS NOT THE PRECEDENT IT LOOKS LIKE.** Measured: `{ space_id, node_id }`, **node↔node federation** — a *node* asking to join a Space. **There is NO user-level request-to-join verb anywhere on the wire** ⇒ *"ask for a DM invitation"* is **genuinely new protocol**, without the analogue Chat would have claimed | `xgen-core/src/wire/types.rs`, `JoinRequest` |
| **G-20** | ✅ **THE "OPEN DM" vs "ASK FOR DM" AFFORDANCE SPLIT COSTS NOTHING TODAY — ITS FEEDER SHIPPED AT `OQ8`/K3.** `counterpart` is a real Space-record field threaded into the UI: **39 hits in `members-panel.svelte`**, plus `dm-draft` 34 · `ops.rs` 16 · `composer-panel` 12 · `dm-spaces` 10 · `stream-panel` 6 · `spaces-state` 5. **The client can already answer *"do I have a DM with this identity?"* with no new data** | `ops.rs:89`; `ui/**` |
| **G-21** | ⚠️ **A PER-SPACE REGIME HAS AN ENFORCEABILITY HOLE.** `dm_space_create` carries `invitee` and **nothing recording where the initiator found you**, and a DM Space is **not created inside** the group Space. A recipient's node could partly reconstruct co-membership — **but only for Spaces it hosts or replicates**, so it fails silently for group Spaces homed elsewhere. 🔑 ***A regime that holds only when two parties share a host is not a regime; it is a coincidence*** | `state.rs:1811-1829` |
| **G-22** | 📌 **VOCABULARY DRIFT, FILED NOT CHASED:** `TrustClaims.tier_verified` is a **`bool`** in `xgen-common`, `Tier2Claims.tier_verified` is a **`u32`** in `xgen-core/auth/tiers.rs`. **One name, two types, two crates** | `trust_assertion.rs:93`; `tiers.rs:73` |

### 🔒 TWO LOCKS THAT CAME OUT OF THE WALK

🔒 **L-1 — THE GATE IS ON *CREATE*, NEVER ON *OPEN*.** Once a DM exists, consent was already given and it is
openable forever. ***"Open DM" is always correct not by convention but by construction*** — and it is the
privacy rule applied to the affordance: **consent at the door, nothing inside the room, and the door is passed
only once.** ⇒ **no leg may put a policy check on opening an existing conversation.**

🔒 **L-2 — IF A DM-INVITE REQUEST IS EVER MINTED, IT IS STRUCTURALLY BOUNDED IN THE SAME EDIT.** **A request
flow is ITSELF an unsolicited message.** A free-text *reason* or *who I am* field **rebuilds this milestone's
entire problem one layer down, on a new object, arriving BEFORE consent instead of after.** ⇒ **identity only,
or identity plus a hard-capped plain string — no canvas, no `WidgetMount`, no rich form** (`N-172`, §7.2
applied to a new carrier). *Naming it now is cheaper than discovering it.*

---

## §4 — 🔓 THE OPEN DECISIONS. **JOE'S, UNRESOLVED, NAMED.**

🔒 **ROUTED UNDER `D-123`'s HELD-HARDEST CLAUSE** — *"anything touching identity, the wire, or an irreversible
act goes to Joe UNRESOLVED and NAMED, even when it arrives dressed as a technical detail."* **Q1, Q6 and Q7
are identity and the wire. Q2–Q5 are product.**

⚠️ **LENS ② (TIER CONSEQUENCE, `D-121`) IS *NO TIER CONSEQUENCE* FOR Q1–Q5 AND Q7–Q10, STATED ONCE AND NOT
MANUFACTURED.** None of them touches crypto-shred (`D-093`), a T4 durability floor, whose-tier-governs, or
one party's erasure-fate imposed on another. **A render or acceptance policy decides what a reader is shown
or whether a conversation begins; it creates no copy and destroys none.** 🛑 **Q6 IS THE EXCEPTION AND IT IS
REAL — see Q6's own lens ② row.** *A manufactured tier rationale is as bad as a manufactured UX one; so is a
missed one.*

🛑 **AND THE CONDITION ON THAT SENTENCE, WRITTEN IN BECAUSE IT WAS NEARLY MISSED: IT HOLDS ONLY WHILE
RETENTION IS ROUTED OUT.** Joe raised preserving a terminated DM's content, minimally at T4. **`G-15` shows
that is an Auth-Module property that already ships**, and `G-15d` shows what survives belongs to **Arc I /
PG-02**. ⇒ **retention is DELIBERATELY not in this milestone's scope, and the moment it enters, the line above
becomes false.** *Routed out on purpose, not by omission — which is the whole difference.*

---

### 🔓 **Q1 — WHOSE POLICY IS IT? THE RECEIVER USER'S, OR THE RECEIVER'S HOME-NODE OPERATOR'S?**

**This is the question that decides every other one, and the banked text does not answer it.** *"Authored at
the receiver's home node"* is true of both readings. `G-1b` shows the cited precedent is **operator**
authority; `I7`'s question — *should a stranger's canvas render unasked* — reads as a **user** preference.

| | **(A) the USER's preference, stored at their home node** | **(B) the NODE OPERATOR's posture for identities they home** | **(C) BOTH — operator sets a floor, user tightens within it** |
|---|---|---|---|
| **① user-visible** | I decide what strangers may show me; it follows me across devices because it lives at my node. **A setting I can find and change** | My institution decides what I may be shown. **On an institutional deployment this is the point**; on a personal node it is my own node so it collapses to (A) with extra steps | I get a control that sometimes cannot be loosened, and **the reason it is locked comes from somewhere I cannot see**. ⚠️ **`D-065` risk: a control that silently refuses is a system misrepresenting its state** |
| **③ resource** | one policy record per identity · a client↔node read verb (`G-5`, new namespace) · a Settings section on `D-120`'s shipped mechanism | reuses `NodePolicy`'s **shape** (`G-1c`/`G-1d`) and its admin-verb pattern · **no Settings UI at all** — the cheapest to build and the one nobody can operate | (A) + (B) + a **resolution rule** between them, and a resolution rule is a third thing to specify, test and explain |
| **fit with the thesis** | `D-144`: *the user may restyle their own client; they may not receive these words from a third party.* A render policy is closer to client state copy than to owner content | institutional independence is a **growth strategy**, and an operator-only control is the shape an institution asks for | — |

🔓 **CHAT'S RECOMMENDATION — (A), with (B) named as a later, additive layer and NOT built now.** (A) is the
only reading under which `I7` — the question with a live user surface — is answerable at all. **(C) is the
right long-run shape and is `D-065`-hostile until there is a way to tell the user *why* a control is locked**,
which is a disclosure surface that does not exist. 🛑 **Proposal, not decision.**

🔑 **JOE'S REFINEMENT, 2026-08-16 — THE AXIS IS *WHERE IT IS AUTHORED*, AND IT IS THE RIGHT ONE:** *"the
user's preference is decided by user in the client setting; the home-node preference is decided by admin/owner
in the node/space setting."* ✅ **Both halves map onto shipped mechanisms** — the user half onto `D-120`'s
`settingsComponent`, the operator half onto `NodePolicy`'s admin-verb shape.

🛑 **BUT THE *SPACE* HALF OF *"node/space setting"* DOES NOT WORK FOR DMs, AND `G-11` / `G-21` ARE WHY.** A DM
Space is created by the **initiator** and `owner_id: creator` ⇒ **a setting on that Space is the stranger's
setting.** *This inversion has now appeared three times in one session — the ban, the policy, the regime —
and it is one fact wearing three faces: **in a DM, the person you may need protection from owns the room.***
⇒ **the operator half must be keyed per hosted IDENTITY, not per Space** — a key `NodePolicy` does not have.

🔒 **AND A CONSEQUENCE THAT NARROWS Q1 SHARPLY: IF JOE TAKES THE DM-PRIVACY `D`, THE OPERATOR HALF COLLAPSES
FOR DMs ENTIRELY.** A node that may not inspect or moderate a DM has nothing to express a *render* policy
about. ⇒ **under the `D`, Q1 for DMs resolves to (A) by construction**, and the operator layer survives only
on the **acceptance** plane (Q8), where it reads identity rather than content.

### 🔓 **Q2 — THE DEFAULT POSTURE FOR USERS WHO ARE NOT INSTITUTIONALLY HOMED** *(J-701, unchanged)*

**Restated against measured ground:** since `G-3` means **no tier is readable**, a tier-based default is
**unimplementable today regardless of which value it takes.** ⇒ the real fork is **what the default keys on
instead.**

| | **(A) render everything** (today's behaviour) | **(B) render nothing from a non-Space-member stranger until asked** | **(C) render the guaranteed `text`, defer the rich canvas behind one click** |
|---|---|---|---|
| **① user-visible** | first contact from a stranger paints their composed canvas, unasked. **This is what ships today** | a stranger's opening message shows as a plain row until I act. **Safest, and it makes the DM feel broken to the sender**, who has no way to know | I always see the sentence — which `M-RP-INTRO` §1-bis makes **load-bearing forever** — and the canvas is **one deliberate click**. Symmetric for the sender: they still know what was delivered |
| **③ resource** | zero | needs a "stranger" predicate; the nearest honest one is **address-book membership**, which `M-RP-PEOPLE`'s node already measured is **NOT a superset of your DMs** — a DM whose Space was never opened may have no book entry ⇒ **the predicate misfires on real data** | a render-time branch in the intro widget + one persisted preference. **No new predicate, no new wire field** |
| **needs new protocol?** | no | ⚠️ **effectively yes** — a sound stranger predicate needs a membership fact the client does not reliably hold | ✅ **NO** |

🔓 **CHAT'S RECOMMENDATION — (C).** It is the only option that is **implementable today**, it satisfies
`I7` without pre-judging Q1, and it spends nothing that a later tier-aware policy would have to unwind.
🛑 **Proposal, not decision. And ⚠️ its default value — deferred or expanded — is APPEARANCE-adjacent and
Joe's under `D-138`.**

---

### 🔓 **Q3 — IS A FILTERED INTRO SILENT OR DISCLOSED?** *(J-701; `D-065` argues disclosed and non-actionable)*

✅ **`D-065` decides this and this file does not re-argue it — it prices it.** *Honest behaviour over polite:
when the system can choose between a behaviour that misrepresents its state and one that honestly reflects
it, XGen picks honest.* **A silently dropped intro is a message the recipient was sent and does not know
exists.**

🛑 **THE PART `D-065` DOES NOT DECIDE, AND IT IS THE ONE THAT NEEDS RULING: DISCLOSED TO WHOM.**

- **to the RECEIVER** — *"this sender included a card; show it?"* ✅ **Chat recommends. It is `D-144` client
  state copy, authored by the client, and it makes Q2(C) legible rather than mysterious.**
- **to the SENDER** — *"your card was not shown."* 🛑 **Chat recommends AGAINST, and the reason is the
  thesis, not the cost:** on a no-anonymity network, telling a sender what a specific recipient's filter did
  **turns a private preference into an oracle a stranger can probe.** ⚠️ *It also has no wire to travel on —
  `G-5` — so refusing it costs nothing and building it would cost a new verb.*
- 🔒 **NON-ACTIONABLE, per `D-065`'s own framing at J-701:** the disclosure names what happened; it is not a
  Report/Block affordance. **`M-RP-BLOCK` is a separate filed milestone with no protocol behind it, and it
  must not arrive as a rider here.**

---

### 🔓 **Q4 — THE COMPLIANCE GAP: CLIENT-SIDE ENFORCEMENT CANNOT BIND A PATCHED CLIENT** *(J-701, unchanged)*

✅ **STANDS, MEASURED: there is no client-attestation subsystem, and `D-113` S-7 is the nearest thing to one**
(*no `packaged` plugin loads until S-1…S-6 ship*) — **which is a sandbox rule, not an attestation.**

🔑 **CHAT'S READING, AND IT IS A REFRAME RATHER THAN A SOLUTION:** the gap is only a *compliance* gap if the
policy is **(B)** in Q1 — an operator promising something about a user's client. **Under (A) it is not a gap
at all**: a user who patches their own client to see more is exercising a freedom this project explicitly
protects (`D-144`: *the user may restyle or re-language their own client*). ⇒ 🔒 **Q4's severity is
DERIVED FROM Q1 and must not be ruled before it.** ⚠️ **If Joe takes (B) or (C), the honest position is that
the policy is a DEFAULT and not a GUARANTEE, and the records must say so** — *a guarantee that a supported
action can walk around is `D-143`'s unsound shape.*

---

### 🔓 **Q5 — PROMINENCE ON UNSOLICITED FIRST CONTACT** *(inherited from `M-RP-INTRO-CANVAS`, `I7`)*

🔑 **THIS IS THE ONLY QUESTION IN THE MILESTONE WITH A USER ON THE OTHER END OF IT TODAY**, and `M-RP-INTRO`
shipped the condition that raises it: the canvas moves **between header and paragraph** (Joe, J-736), so a
stranger's composed block is the **first** thing rendered from someone you have never met.

✅ **`I1` HOLDS AND IS NOT AT ISSUE** — it is message chrome, with attribution directly above it, in the DAG,
redactable and reportable. **What is at issue is UNASKED RENDERING, which `I1` never spoke to.**

🔒 **Q5 IS ANSWERED BY Q2 AND HAS NO SEPARATE MECHANISM.** Recorded as its own question **only because it is
the milestone's motivating user harm and would otherwise vanish inside a fork about defaults.** ⚠️ **It is
also the reason §5 splits the milestone: Q5 + Q2(C) is buildable now; Q6 is not.**

---

### 🔓 **Q6 — DOES `identity.record` GAIN THE SENDER'S TIER? IDENTITY AND THE WIRE — JOE'S OUTRIGHT.**

🛑 **CHAT MAKES NO RECOMMENDATION ON THIS ONE AND WILL NOT.** It changes **what an identity publishes about
itself to anyone who asks**, on a protocol whose thesis is no-anonymity. **That is a philosophy question
wearing a field's clothes.** The options are recorded so the fork is a partition; the ruling is Joe's.

| | **(i) widen `identity.record` to carry `trust_assertion`** | **(ii) a scoped tier read — member-scoped, like `collect_sync_history`** | **(iii) build nothing; the policy keys on inputs that exist** |
|---|---|---|---|
| **① user-visible** | anyone holding my XGID learns my verification tier | someone already in a Space with me learns it | no tier-aware policy exists; Q2(C) still works |
| **② tier consequence** | 🛑 **REAL AND NAMED: `trust_assertion` is the T2–T4 claim set — legal name, organisation, AML/KYC, security clearance level, jurisdiction (`tiers.rs:70-118`). PUBLISHING IT WHOLESALE PUBLISHES REGULATED PERSONAL DATA.** Widening the projection is a **GDPR surface**, not a field addition | narrower blast radius, **same class of data** | none |
| **③ resource** | one field + spec | a new scoped verb + spec + the scope rule | zero |
| **thesis** | 🔑 **J-598's own reading applies verbatim: no-anonymity means *within a Space you know who you are talking to*, NOT *any identity is globally queryable by anyone holding a pubkey*. An unscoped verb converts the first into the second — a thesis change made by accident, at the wire level** | consistent with that reading | — |

🛑 **AND IT IS ALREADY ROUTED ELSEWHERE, SO IT MUST NOT BE DECIDED TWICE.** `trust_assertion` was **BARRED
from riding `M-RP-INTRO`** and is **still unrouted**; `M-RP-INTRO-CANVAS` carries the **H1/H2 visit-card**
question, and *"there is no identity lookup verb on the wire at all"* is the same wall from the other side.
🔒 **CHAT PROPOSES ONLY THIS: Q6 IS THE SAME QUESTION AS H2 AND SHOULD BE RULED ONCE, IN ONE PLACE, FOR BOTH
MILESTONES.** *Two milestones independently deciding what an identity publishes is how a protocol grows two
answers.*

---

### 🔓 **Q7 — IF A POLICY IS STORED AT A NODE, WHAT NAMESPACE CARRIES IT?** *(the wire — Joe's)*

Only live **if Q1 lands on (A) or (C)** and the policy is not purely client-local. `G-5`: **there is no
`policy.*` namespace.** 📌 **Recorded, deliberately not optioned** — it is downstream of Q1 and pricing it
now would be pricing a decision that may never be taken. ⚠️ **A purely client-local policy needs no wire at
all, and that is a real fourth option for Q1 that §5 Leg A ships by accident** — named here so it is chosen
rather than defaulted into.

---

### 🔓 **Q8 — ACCEPTANCE PLANE OR RENDER PLANE? THE OPTION v1.0 DID NOT CONTAIN.**

🔑 **§9 NAMED A MISSING OPTION AS THIS FILE'S LIKELIEST DEFECT AND IT WAS RIGHT WITHIN THE DAY.** v1.0 asked
only *what renders*. Joe's DM-consent walk produced a second plane: **gate the DM's CREATION, not the
message's rendering.**

| | **render policy** (as banked) | **acceptance policy** (`G-16`–`G-18`) |
|---|---|---|
| reads | message **content** | **initiator's identity** |
| enforced at | client, necessarily (`I1`, `D-143`) | **node, durably** |
| survives PG-05 | yes, by having moved to the client | **yes, natively — identity is not encrypted** |
| Q4's patched-client gap | real | **none — the invite never arrives** |
| tier readable? | ❌ not client-side (`G-3`) | ✅ **yes, node-side, today (`G-15c`)** |
| stops the intro | after arrival | **before the DM exists** |
| ① user-visible | a stranger's card is deferred or annotated | **a stranger cannot open a DM without asking** |
| ③ resource | one render branch + a preference | a policy read + **a new request verb (`G-19`: no precedent)** |

🔓 **CHAT'S RECOMMENDATION — BOTH, ON DIFFERENT LEGS, AND THE RENDER ONE FIRST.** The render deferral (Q2 (C))
is buildable today with **zero protocol** and discharges `I7`/Q5 for the user who has the problem now. The
acceptance gate is **stronger and slower** — it needs `G-16a`'s measurement, a policy read and a new verb.
🛑 **They are not alternatives and must not be traded against each other:** *consent at the door, and a choice
about prominence once inside.* 🛑 **Proposal, not decision.**

⚠️ **AND THE DEFAULT IS Q2 AGAIN, HARDER:** *"accept DMs only from T≥N"* is legitimate, **but as a default it
makes the network unreachable** — T2–T4 do not exist and everyone is T1 (`G-6`). 🔒 **Default must be
accept-all; the gate is OPT-IN.** *That is also the honest institutional story: the institution restricts
itself, not everyone else.*

---

### 🔓 **Q9 — WHERE DOES THE REQUIREMENT LIVE, AND DOES THE FLAG PUBLISH?** *(identity and the wire — Joe's)*

**Joe, 2026-08-16:** *"we can call it dm-invite-requirement or request, and can be a part of each auth module,
even t1."*

✅ **THE HOME IS MEASURED AND FREE: `ModulePolicy.extra` (`G-18`)** — additive, zero struct change, every tier's
Auth Module issues one, and **enforcement needs no wire at all** because the recipient's node reads its own
registry (`G-15c`).

🔑 **THE OPEN HALF IS DISCOVERY, AND IT COLLAPSES Q6 INTO SOMETHING SMALL.** For a client to render *"Open
DM"* vs *"Ask for DM"* it must learn the requirement ⇒ `identity.record` again. **But now the disclosure is
ONE BOOLEAN, not `trust_assertion` wholesale** — no legal name, no clearance, no jurisdiction, **no regulated
data.** ⇒ **Q6 GAINS A THIRD OPTION: publish the requirement flag, publish nothing else.**

🔓 **Chat's recommendation — publish the flag.** It is arguably not a disclosure at all: it is a fact about
**how you may be contacted**, which is the entire purpose of publishing it. **Fallback if Joe prefers to
publish nothing:** the client creates optimistically, the recipient's node rejects, the client then offers to
request — honest, `D-065`-clean, one wasted round trip, **and it leaks the requirement anyway**, which is why
publishing is the cheaper truth. 🛑 **Proposal, not decision.**

📌 **NAMING IS JOE'S, AND THEY ARE TWO OBJECTS.** The **requirement** is the recipient's standing policy; the
**request** is the initiator's act. *Fusing them into one word is how a policy and an event end up sharing a
name and drifting apart later.* The protocol's existing pair is `PendingInvite` / `membership.invite`.

🔒 **`L-2` BINDS ANY REQUEST VERB MINTED UNDER THIS QUESTION.**

---

### 🔓 **Q10 — IS THE REGIME PER-IDENTITY OR PER-SPACE?**

| | **per-identity** | **per-Space** |
|---|---|---|
| whose rule | the recipient's, via their Auth Module `ModulePolicy` | the **Space owner's**: *"members here may / may not DM each other freely"* |
| ① user-visible | protects you **everywhere**, travels with your identity | governs only contact arising **from that Space** |
| ③ resource | `ModulePolicy.extra` — **additive, zero struct change** | **new Space-state field + new EventType + owner-settable** |
| enforceable? | ✅ recipient's node reads its own registry | 🛑 **`G-21`: NO** |

🛑 **`G-21` IS THE DECIDING ROW: A PER-SPACE REGIME CANNOT BIND SOMEONE WHO TAKES YOUR XGID FROM A MEMBER
LIST AND OPENS THE DM FROM ANYWHERE ELSE**, because `dm_space_create` records **nothing about where the
initiator found you**, and partial reconstruction works **only where the recipient's node hosts or replicates
that Space.** ⇒ 🔓 **Chat recommends per-identity, and files per-Space separately** — it is a legitimate
institutional want (*"no unsolicited internal DMs"*) with an **unsolved enforceability question**, so it must
not ride here. 📌 **They compose if both ever exist: either may require a request. Nothing decided now
forecloses the other.**

---

### 🔓 **Q11 — THE DM-PRIVACY `D`. 🛑 IT OUTRANKS Q1–Q10 AND SHOULD BE RULED FIRST.**

**Joe, 2026-08-16:** *"such space is a special mechanism and has to be autonomous from node setting of general
spaces… the dm has to be a private space"*, on the ground that **T1 identity is already bannable, so the
counterpart can cut and ban without third-party intervention — and that holds even at T4.**

🔑 **THE STRONGEST FORM OF THE ARGUMENT, AND IT IS TIER-INDEPENDENT, SO CHAT RECOMMENDS RECORDING IT RATHER
THAN THE TIER FORM:** ***third-party moderation exists to protect people who did not choose to be in the room.
A DM has no bystanders*** — both parties are the entire membership, and either one's exit ends it at zero
cost. **In a group, leaving costs you the group, which is exactly why someone else must be able to act.**

🔒 **THE PROPOSED `D` HAS THREE PARTS, NOT ONE:**

1. **DM Spaces are exempt from node MODERATION / CONTENT policy.** 🛑 **NOT from node RESOURCE policy** —
   `max_event_size`, rate limits, storage stay uniform, because **anyone can open a DM to any identity, so an
   exempt DM is an unmetered write path into a node.** 🔑 **The boundary is not a preference: it is *what the
   node can still enforce on ciphertext after PG-05* — the same test `I1` uses, and the one §3.7.13.4 already
   sits on the correct side of (`G-10c`).**
2. **The terminal cut is `membership.leave`** — protocol, mutual, in the DAG, **and already structurally
   terminal** under §3.16.1's invitations-disabled (`G-12`). **It needs a user-reachable verb, not a new
   event.** 🛑 **`G-13` first: leaving currently closes YOUR OWN read path, and that must be resolved before
   the cut gets a verb** — either the cut preserves a *was-a-member* read grant, or preservation lives
   somewhere other than the Space you left. 🔓 **Chat has NO recommendation; `G-12a` is unread.**
3. **The ban is ADDRESS-BOOK-LOCAL** — unilateral, invisible to the other party, unenforced against arrival and
   **honest about it** (`G-14`). 🔒 **And exempt from retention eviction (`G-14a`).**

🔑 **THE ASYMMETRY IS ITSELF A PRIVACY PROPERTY: the cut is protocol and mutual; the ban is local and tells
the other party nothing.**

⚠️ **TWO QUALIFICATIONS THAT DO NOT DEFEAT IT AND BELONG IN THE RECORD:** ① **a ban is per-key and T1 keys are
free** — self-help is sufficient for **accountability**, not **prevention**; *this is a limit of T1 identity,
not of the privacy rule, and node-side filtering would not stop key-churn either* ⇒ **it argues FOR the rule.**
② **a ban is post-hoc and Q5 is pre-hoc** — first contact is precisely the moment before you can ban, so the
cut disposes of the **repeat**, never the **first**. 🔑 ***That is why Q2 (C) survives this ruling intact:
deferring the canvas is not moderation, it is the receiver's own client deciding what to draw.***

🛑 **DoD CONSEQUENCE, AND IT IS THE ONE THAT MOVES WORK: A PRIVACY RULE JUSTIFIED BY *"they can just cut and
ban"* IS UNSOUND WHILE THEY CANNOT (`G-11`, `G-11a`).** ⇒ **`M-RP-BLOCK` becomes a PREREQUISITE of this `D`,
not an unrelated filed milestone** — and `G-12` re-prices it **downward**: the DM case needs **no new wire
event**, only a leave verb and a book flag.

📌 **AND IT LANDS AS A SPEC AMENDMENT, NOT A CONFORMANCE FIX (`G-10`).** §3.16.1 gains a **sixth constraint**
and §3.7.13 gains an explicit DM exclusion; the **tier omission (`G-10d`) is recorded as deliberate** in the
same edit. **Its own milestone — spec work, never a rider.**

---

## §5 — 🔒 PROPOSED LEGS. **THE SPLIT IS CHAT'S SEAT (`D-123`); EVERY RULING IN IT IS JOE'S.**

🔑 **THE SPLIT'S WHOLE ARGUMENT: ONE HALF OF THIS MILESTONE NEEDS NO NEW PROTOCOL AND HAS A USER WAITING;
THE OTHER HALF NEEDS A RULING ABOUT WHAT AN IDENTITY PUBLISHES.** Shipping them as one milestone means the
first waits on the second. *`D-143` does not fire here — Leg A is **complete for what it does** and asserts
nothing that can go false, which is `D-065`'s no-empty-machinery half.*

| leg | what it does | gate | state |
|---|---|---|---|
| **0-pre** | 🔓 **Joe rules Q11 — the DM-privacy `D`.** 🛑 **It outranks everything below: under it, Q1's operator half collapses for DMs and Q4 narrows.** No code | — | 🟡 **PENDING — Joe** |
| **0** | 🔓 **Joe rules Q1** (whose policy) — **no code. This leg is the gate for every other one** | 0-pre | 🟡 **PENDING — Joe** |
| **0-bis** | 🔓 **Joe rules Q2 + Q3 + Q8** (the default, disclosed-to-whom, and which plane) | Leg 0 | 🟡 **PENDING — Joe** |
| **A** | **the render-time deferral: Q2(C) built client-side.** The intro's rich canvas renders behind one deliberate act; `content.text` always renders. **A `D-120` `settingsComponent` holds the preference.** 🛑 **ZERO Rust, ZERO wire, ZERO spec — and it discharges `I7`/Q5** | 0-bis | 🟡 **PENDING** |
| **B** | **the disclosure copy** — `D-144` client state copy, authored by the client and by nothing else. ⚠️ **Wording is Joe's (`D-138`, scaffolded not blank)** | A | 🟡 **PENDING** |
| **C** | 🔓 **Joe rules Q6** — and **`M-RP-INTRO-CANVAS`'s H1/H2 is the same question; rule once** | — | 🟡 **PENDING — Joe, and NOT gated on A/B** |
| **D** | **the tier-aware policy proper** — policy record, storage, the client↔node read (Q7's new namespace), enforcement at render | C · Q7 | ⏸️ **BLOCKED on C. Do not scope before it** |
| **M** | 🔑 **THE ONE MEASUREMENT, AND IT GATES THE WHOLE ACCEPTANCE PLANE: DOES THE CLIENT AUTO-JOIN A DM INVITE (`G-16a`)?** Plus `G-12a` (1-member DM behaviour; are `ops.rs`'s `MembershipLeave` sites reachable?) and `G-18a` (`extra` flatten). **Reads and one live drive; no code** | — | 🟡 **PENDING — NOT gated on any ruling** |
| **P** | 🔓 **Joe rules Q9 + Q10** (requirement home + publish; per-identity vs per-Space) | 0-pre · M | 🟡 **PENDING — Joe** |
| **G** | **the acceptance gate** — `ModulePolicy.extra` requirement, node-side enforcement at invite, the request verb under `L-2`. 🛑 **Protocol + node + client** | P | ⏸️ **BLOCKED on P** |
| **E** | live verify, two identities, Chat re-drives every gate (Rule 5) | the legs that land | 🟡 **PENDING** |
| **F** | records + close (`D-074`) | E | 🟡 **PENDING** |

🛑 **EVERY ROW CARRIES A STATE, DELIBERATELY** — `M-RP-MEMBER-ACT`'s §6 leg table had two states across eight
rows while four legs had shipped. **This table is a state board or it is not a leg table.**

🔓 **AND THE SPLIT ITSELF IS JOE'S TO OVERTURN.** ⚠️ **If Joe wants `M-INTRO-POLICY` to remain one milestone
that ships only when the tier is readable, that is a legal answer** — and then **Legs A + B should be
re-sited into `M-RP-INTRO-CANVAS`, not dropped**, because the prominence question is live either way and the
canvas milestone is the one touching that render path. *Chat states its recommendation and does not assume it.*

---

## §6 — RECORD CORRECTIONS OWED (Chat's seat — no ruling required)

| # | record | correction |
|---|---|---|
| **R-1** | `ui/docs/xgen-ui-notes.md` `N-172` socket table (`:3530-3534`) | 🛑 **`M-RP-INTRO`'s Phase-0 §5 `R-2` WAS A DoD ITEM (§10 item 5) AND THE MILESTONE CLOSED WITHOUT IT.** Measured: **no annotation exists anywhere in `xgen-ui-notes.md`** for the fourth socket, and the table is now stale **twice** — it lists **three sockets** (missing `stream-panel.above` / `dm-intro`, `G-9a`) **and one `bodyExtras` tenant** when the shipped build has **two** (`send-status` + `message-intro`, driven at J-735 as `childCount: 2`). ⚠️ **A DoD item was ticked and not done, in the one note whose job is to stop a future feature spending the socket's security property.** Annotate at the site (`D-131`), do not rewrite |
| **R-2** | `docs/ROADMAP.md` `M-INTRO-POLICY` node `Owes:` | *"auth tier is an INPUT to the policy"* is **true and incomplete**: it prices the missing tiers and not the missing **wire field** (`G-3`). **Annotate with `G-3` / `G-3b`**, keeping the existing J-738 correction intact |
| **R-3** | `docs/ROADMAP.md` `M-INTRO-POLICY` node `Owes:` | *"`NodePolicy` is the precedent"* → **precedent for the STORAGE SHAPE only; the store is INERT and has no enforcement reader (`G-1a`)**, and it mismatches on key and principal (`G-1b`) |
| **R-4** | `ui/docs/xgen-ui-notes.md` `N-173` | the collision is **no longer prose-only**: `data-tier` is a live attribute meaning **widget tier** in `ui/common/**` and **auth tier** in `ui/templates/skeleton/tokens.css` + `ui/backup/run_1.0/` (`G-4a`). **The note's own claim — *"every `tier` hit in `ui/` is the second axis"* — is FALSE once the template files are in the pool.** ⚠️ **The rename is still Joe's; only the measurement is corrected** |

| **R-5** | `docs/ROADMAP.md` `M-RP-BLOCK` node | 🛑 **RE-PRICED IN BOTH DIRECTIONS AND ITS TRIGGER CHANGES.** The node reads *"it is protocol and node work before it is ever UI"* — **for the DM case that is now measurably wrong**: `G-12` shows leave + invitations-disabled is **already terminal with no new event**, and `G-14` puts the ban in the address book with **no wire**. ⚠️ **And it gains a trigger it has never had** (`trigger: none — filed, not scheduled` today): **`M-RP-BLOCK` is a PREREQUISITE of Q11's `D`**, because the privacy rule's own justification is that the counterpart can cut and ban — and `G-11` shows the recipient currently can do **neither** |
| **R-6** | `ui/docs/xgen-ui-notes.md` | 📌 **`N-197` IS OWED AND ITS WORDING IS JOE'S** — unchanged by this session, restated because §8 is where a reader looks for it. **No new `N` is minted here**; `G-9`–`G-22` live in this file, and promoting any of them is a later call |

📌 **All six are annotations to existing records, not rewrites (`D-131`).** They travel with this
milestone's first commit under `D-074` — **JOURNAL + CLAUDE.md + ROADMAP + this file in ONE commit.**

---

## §7 — WHAT THIS MILESTONE MUST NOT DO

1. 🛑 **MUST NOT move the intro into system chrome.** `M-RP-INTRO` `I1` ruled it by argument (`D-113` S-5:
   stranger-authored content in the system's voice on first contact). **The node's *"unlocks system-chrome
   rendering safely"* is a MOTIVATION, not a licence, and no leg here re-opens that ruling.**
2. 🛑 **MUST NOT put a filter, a policy, a `widgetId` or any receiver-side render instruction ON THE WIRE**
   (`N-172`, `I3`). **The sender sends DATA; the receiver decides rendering.** A policy that travels with a
   message is a sender-controlled render instruction wearing a safety feature's clothes.
3. 🛑 **MUST NOT build a node-side content filter** (`I1`, `D-143`). After PG-05 the node holds ciphertext.
   **A filter with a known expiry date is the unsound cheap route, named.**
4. 🛑 **MUST NOT break `content.text`'s load-bearing guarantee** (`M-RP-INTRO` §10 item 1-bis). A policy may
   defer, collapse or annotate the **rich** form; **it may never suppress the sentence**, because that is the
   only thing making a versioned key degrade rich→plain instead of rich→nothing.
5. 🛑 **MUST NOT reserve anything for a decision not yet taken** (`N-182`): no unfed policy key, no
   `policy: null`, no blank prop, no descriptor field for a tier nothing publishes. ⚠️ **`G-3b` is what this
   rule looks like when it is broken — a field, a reader and a test, fed by nothing, shipped.**
6. 🛑 **MUST NOT absorb `M-RP-BLOCK`, `M-RP-INTRO-CANVAS`, `M-RP-PEOPLE` or the `trust_assertion` routing.**
   Q6 **shares** a ruling with the canvas milestone's H1/H2; it does not take it over.
7. 🛑 **MUST NOT introduce a third meaning for "tier"** (`N-173`, `G-4a`), and **must not reuse the
   `data-tier` attribute**, which is already spent twice.
8. 🛑 **MUST NOT PUT A POLICY CHECK ON *OPENING* AN EXISTING CONVERSATION** (`L-1`). **The gate is on CREATE.**
   Once a DM exists, consent was given; a check on open would re-adjudicate a decision the user already made,
   and would break the shipped `counterpart` affordance (`G-20`) for no gain.
9. 🛑 **MUST NOT MINT A DM-INVITE REQUEST CARRYING FREE TEXT** (`L-2`). **A request flow is itself an
   unsolicited message**, arriving **before** consent rather than after. **Identity only, or identity plus a
   hard-capped plain string — no canvas, no `WidgetMount`, no rich form.**
10. 🛑 **MUST NOT TAKE ON DM RETENTION.** `G-15` shows it is an Auth-Module property that already ships;
   `G-15d` shows what remains open belongs to **Arc I / PG-02**. ⚠️ **The instant retention enters scope,
   §4's *"no tier consequence"* line becomes false** — which is why it is routed out **deliberately**.

---

## §8 — OPEN ITEMS CARRIED, NOT THIS MILESTONE'S

📌 Recorded so they are not rediscovered as findings, and so none is quietly absorbed:

- 🛑 **THE ROUND-2 CONTRADICTION IS STILL UNRESOLVED.** `docs/ROADMAP.md:290` marks Round 2 **✅ GO at J-390**
  (*"final pre-UI whole-codebase gate"*) while the J-735 kickoff carried *"Round-2 still GATES UI
  COMPLETION"*. **Both cannot be current.** ⚠️ **It bites here**: if this milestone reaches Leg D it is
  protocol-plane work, and whether a gate stands in front of it is unknown. **Reconcile in a session that
  starts on it** — not at a tail, and not inside this milestone.
- 🔓 **`N-197`'s wording** — six instrument failures across three seats. Joe's, still owed.
- 🔓 **`F-6`, from Clair: the `bodyExtras` tenant ordinal disagrees across FOUR sites** with no stated
  counting rule. **A counting rule is naming ⇒ Joe's.** *Touches `R-1`'s file; do not fold them.*
- 🔓 **`blurb` → `about`: 37 sites / 6 files, case-insensitive LINES** (state the metric — 28/37/50 are all
  defensible). **Crosses into `skin.css`, which is Joe's**, and the selector `.message-intro-blurb` must move
  **with** the field. **Sequences into `M-RP-INTRO-CANVAS`.**
- 🔓 **`HEADLINE_MAX 120` / `BLURB_MAX 600`** ship PROVISIONAL under `D-138` and are Joe's. ⚠️ **A policy that
  filters content should probably own its bounds — Chat recommends they be re-sited here IF Leg D is ever
  built, and NOT before.**
- 🔓 **The DM-draft-only authoring asymmetry** (Clair's, named as hers) — the reader is permissive, the writer
  is narrow. **Joe's.**
- 🛑 **`git worktree prune` is recommended and un-run.** Four of the eight trees under `.claude/worktrees/`
  are **orphaned**; four are registered. **Do not delete without Joe** — Claude Code may expect them.
- 🛑 **`M-RP-PEOPLE`'s node already records that the address book is NOT a superset of your DMs.** Any leg
  here tempted to use book-membership as a "stranger" predicate must read that first (`Q2(B)`).

---

## §9 — RECOMMENDED: CLAIR'S ADVERSARIAL READ, COLD, POINTED AT §3 FIRST

🔑 **THE EVIDENCE IS ABOUT CHAT, NOT ABOUT CLAIR, AND IT HAS NOW PAID TWICE.** At J-733 a cold read returned
**seven** findings, five of them pointer defects, **all confirmed against source, none caught by Chat's own
re-read of the same file**. At J-738 it returned **six**, **four of them wrong claims**, on documents Chat
had re-read — **and two sat in `skin.css`, the file the close touched last and the only one nobody re-read
afterwards.** 🛑 ***If you are about to verify something by re-reading it, that is not verification.***

**The read is recommended with a specific shape:**

1. **§3 AND §3a FIRST, COLD, BEFORE §1 OR §2 CAN FRAME THEM.** They are the sections that changed the
   milestone. A reader who has absorbed §1's verdict column will inherit Chat's framing of what the
   constraint *is*. 🛑 **§3a IS THE HIGHER-RISK ONE: it was written in a single sitting, from a live design
   conversation, and it CORRECTS FIVE OF CHAT'S OWN SAME-SESSION STATEMENTS — which is exactly the condition
   under which the sixth goes unnoticed.**
2. **Then every `G-` row against source.** Each cites a file and a line. **The question is not "is this
   plausible" but "does that line say this."** ⚠️ **Ask of every count: what would this return if the code
   were RIGHT?** Same answer ⇒ the probe is wrong (`N-194`).
3. 🛑 **THEN THE CENSUSES, AND CHECK THE INCLUDE LIST BEFORE THE COUNT.** The `blurb` census was published
   **three times** (39 → 35 → 37) and **the file it omitted was the failure mode**. **Every count in this
   file states its pool; verify the pool, then the number, then that the number is not contradicted by the
   grouped output it came from** (J-737: a published 39 against a list summing to 50).
4. 🛑 **THEN ASK WHETHER §4 IS A PARTITION OR A CENSUS.** *A census is not a partition* has now been the
   finding **three times in this arc**, and **Q1 grew a fourth option (purely client-local, `Q7`) while this
   file was being written**. **A missing option is the most likely defect here.**
5. **Then §5 and §7 for the `F9` pass:** *can each gate be RUN, in the order written, from the seat that owns
   it?* — and for anything §7.5 forbids.

📌 **Standing Clair up is Joe's.**

---

## §10 — DoD

**`M-INTRO-POLICY — receiver-side render policy` is DONE when, and only when:**

0. **Q11 is Joe-ruled FIRST** — the DM-privacy `D`. 🛑 **It outranks the rest: under it Q1's operator half
   collapses for DMs and Q4 narrows.** 🔓 **Chat's reading is that Q11 EARNS a `D` outright**, and that it is
   **broader than this milestone** — a statement about DM Spaces generally, which therefore **constrains**
   `M-INTRO-POLICY` rather than being produced by it. **Number and wording Joe's.** 🛑 **And it is not DONE
   until `M-RP-BLOCK` gives the recipient a cut and a ban** — a rule justified by *"they can cut and ban"* is
   unsound while `G-11` holds.
1. **Q1 is Joe-ruled** (whose policy), and **Q2 + Q3 + Q8** with it. **Q6 is ruled — or explicitly deferred with
   a named home shared with `M-RP-INTRO-CANVAS`'s H1/H2**, noting `G-18`/Q9 gives it a **third, much smaller
   option** (publish one boolean). 🔓 **Chat's reading is that Q1 EARNS a `D`** — it decides whether a
   user-facing policy layer exists in this project at all, and that binds beyond this milestone. **The number
   and the wording are Joe's.**
1-ter. 🔑 **LEG M IS RUN BEFORE ANY ACCEPTANCE-PLANE LEG IS SCOPED** — `G-16a` (does the client auto-join a DM
   invite?), `G-12a` (1-member DM behaviour; reachable `MembershipLeave`?), `G-18a` (`extra` flatten).
   🛑 **If the client auto-joins, a consent gate exists in the protocol and is being spent silently — and
   every option in Q8 is priced against a fact nobody has checked.**
2. **The `§5` split is Joe-confirmed or Joe-overturned.** 🛑 **If overturned, Legs A + B are RE-SITED into
   `M-RP-INTRO-CANVAS`, not dropped** — the prominence question is live either way.
3. Legs per §5, each verified by **Chat re-driving every gate independently** (Rule 5) — **numbers Chat did
   not personally measure do not enter this record.**
4. Every verification gate **names its surface in scope before the leg opens** (`M_RP_MEMBERS.md` §8b): any
   DoD item saying *observed* / *exercised* / *driven* / *measured* **must name its surface**, and before
   locking a leg list, **walk every 🔒 and ask WHICH LEG BUILDS THIS.**
5. **No phase-limit note survives the leg that lifts its limit** (`N-109`): when a leg ships a disclosure, its
   **REMOVAL enters the DoD of the leg that lifts the limit, in the same edit that adds it.** ⚠️ **Leg B ships
   a disclosure by definition; this item is not decoration here.**
6. **`R-1` … `R-6` are annotated at their sites** (`D-131`). 🛑 **`R-1` IS THE ONE THIS MILESTONE INHERITED
   BECAUSE ITS PREDECESSOR TICKED IT AND DID NOT DO IT. It does not get ticked twice.**
6-bis. 🔒 **`L-1` AND `L-2` SURVIVE INTO EVERY LEG AND EVERY RUNBOOK.** The gate is on **create**, never on
   **open**; and any request verb is **structurally bounded in the edit that mints it**, not in a later one.
7. `roadmap-format-gate.ps1` returns **exit 0**, and the ROADMAP node's `Owes:` is **CUT to what is still
   owed, pointing at the record** (J-715) — 🛑 **a closing commit REDUCES a node. DO NOT REGROW IT.**
8. **Floors re-measured, not inherited**, each stated **with the screen it was measured on** — svelte-check at
   minimum; **the catalogue only if its harness is located and driven**; **`cargo` only if `.rs` is touched,
   and never as a scope argument.**

🛑 **"Commit pushed" IS NOT A DoD ITEM** — it is unflippable inside the commit that performs the push.
`Status: COMPLETED` in this file's header is the canonical signal.
