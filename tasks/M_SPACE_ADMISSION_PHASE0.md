# M-SPACE-ADMISSION Phase-0 — who may join a Space, and how a leaver comes back
> **Status**: ACTIVE  
> Version: 2.9  
> Date: Aug 2026  
> **Last updated**: 2026-08-22  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS FILE IS

The **D-071 phase-0 audit** for `M-SPACE-ADMISSION — who may join a Space, and how a leaver comes back`.

🔒 **The SHAPE is already locked (Joe, J-741) and this file audits AGAINST it. It does not re-open it.** Owner-settable, two riders, default open. §1 restates the lock so the audit has a fixed target.

📌 **Chat wrote this file FIRST**, with Joe's decisions as open §§ carrying recommendations and `D-121`'s three lenses (① user-visible impact · ② tier consequence · ③ resource cost, in that order). Joe locks after. `D-123`'s named failure mode is Chat gating its own authoring on unasked Joe decisions; this file exists so that cannot happen.

📌 **Every claim below carries its site.** Claims inherited from J-739/J-740/J-741 were **re-driven at `de9a397`** and are marked ✅ RE-DRIVEN; claims taken on the record are marked 📌 CARRIED. **Rule 5 runs both ways** — §3 and §10 record defects this audit found in the canonical record itself.

🛑 **NO PRODUCT CODE. Reads only.** Zero `.rs`, zero `.ts`, zero `.svelte`, zero `ui/**` written this session.

---

## §1 — 🔒 THE LOCK. **AUDIT AGAINST IT; DO NOT RE-OPEN IT.**

| # | the lock (Joe, J-741) |
|---|---|
| **L-A** | **Admission is a SPACE property, not a DM special case.** *"the rejoin event has to have also general spaces, definitely, so this mechanism must be space general."* |
| **L-B** | **OWNER-SETTABLE, not set-once.** The discriminator: `jurisdiction` and `e2e_encryption` are set-once because they **re-label data that already exists — they falsify the past.** Admission is evaluated **once, at a join, and never re-evaluated** ⇒ no past to falsify. |
| **L-C** | **RIDER 1 — the DM PINS the value** (invite-required, not settable there), or the stranger who owns a DM could flip their own DM open. |
| **L-D** | **RIDER 2 — gate on the ROLE PREDICATE, never on `owner_id` equality**, which would add a **third** site to a split-authority problem that has exactly two. |
| **L-E** | **DEFAULT MUST STAY OPEN**, or J-275's model breaks for every existing Space. |
| **L-F** | 📌 A **one-way ratchet** (`open → invite` permitted, `invite → open` refused) is filed as a partition member and **not recommended**. |

---

## §2 — GROUNDING: THE SHIPPED ADMISSION MODEL. **RE-DRIVEN AT `de9a397`.**

### §2.1 — Nothing requires an invite to join any Space

| # | fact | site | status |
|---|---|---|---|
| **G-1** | `MembershipJoin` sits in `skip_membership` ⇒ the membership check (step 11) and the permission check (step 13) are both skipped for a join | `exchange.rs:649-659` | ✅ RE-DRIVEN |
| **G-2** | The invite-expiry gate is `if let Some(pi) = space.pending_invites.get(&event.sender)`, and the comment directly above reads *"an open join (no pending invite at all) is untouched"* | `runtime.rs:1564-1565`, `:1586` | ✅ RE-DRIVEN |
| **G-3** | `apply_join`'s space-level guards are **exactly two** — already-member (`:1016`) and banned (`:1019`) — with an absent invite taking `None => (Role::Member, None)` (`:1024`) | `state.rs:1016`, `:1019`, `:1022-1025` | ✅ RE-DRIVEN |
| **G-4** | `check_permission` gates joins on **nothing** — the catch-all is `_ => Ok(())` | `exchange.rs:914` | ✅ RE-DRIVEN |
| **G-5** | The model is **deliberate and named in the source**: *"A plain Space (`dm_constraints_active = false`; open-join per J-275)"* | `runtime.rs:5580` | 📌 CARRIED (J-741) |

🔑 **⇒ `PendingInvite` is a STATE, not a GATE.** The state claim (an invitee is pending until they emit `membership.join`) is true; the gate claim is false. **There is nothing to spend, on either side.**

### §2.2 — No admission field exists, and there IS a slot for one

| # | fact | site | status |
|---|---|---|---|
| **G-6** | `auth_tier` is **already an admission contract on `SpaceState`** — `verify_tier_assertion(assertion_tier, space_auth_tier)`, a floor checked at join | `tiers.rs:158-168`; field `state.rs:190` | 📌 CARRIED (J-741) |
| **G-7** | ⚠️ **BUT `auth_tier` IS A REQUIRED CREATE FIELD, NOT AN OPTIONAL ONE.** `from_space_create` reads `content["auth_tier"].as_u64().ok_or(SpaceError::MissingField("auth_tier"))?` — **absent is an ERROR.** It is therefore the wrong template for the absent-⇒-default migration L-E requires | `state.rs:275` | ✅ **NEW, THIS SESSION** |
| **G-8** | `member_temperature_visibility` **is** the right create-parse template: `content[...].as_str().map(...).unwrap_or_else(|| DEFAULT...)` — **absent ⇒ derived default, no backfill** | `state.rs:307-310` | ✅ **NEW, THIS SESSION** |
| **G-9** | No admission field exists today: `join_policy` / `open_join` / `discoverable` / `is_public` = **0**; `space_visibility`'s 3 hits are test function names | corpus-wide, `.rs`, `\.claude\` + `\target\` excluded | 📌 CARRIED (J-741) |

### §2.3 — Role predicates already exist and are the L-D vehicle

`can_invite` / `can_kick` / `can_mute` ≥ `Moderator` · `can_ban` / `can_create_room` / `can_change_space_info` ≥ `Admin` · `can_manage_federation` == `Owner` (`membership.rs:128-155`, 📌 CARRIED). **The applier-side idiom is `apply_mute`'s** — `self.member_role(actor.as_str()).ok_or(NotASpaceMember)?` then `if !can_X(actor_role)` (`state.rs:769-773`, ✅ RE-DRIVEN). **That is L-D's vehicle and it already exists.**

---

## §3 — 🛑 THE SIBLING IS RIGHT ON SHAPE AND WRONG ON GATE. **THREE NEW FINDINGS.**

J-741 named `member_temperature_visibility` as the right sibling — *"whose event type, applier arm and resolution arm already exist and work"* (`docs/ROADMAP.md` M-SPACE-ADMISSION node, and `tasks/M_INTRO_POLICY_PHASE0.md` §3a.7 — ⚠️ **NOT `CLAUDE.md`, whose J-741 block says only *"the right sibling is `member_temperature_visibility`"* and makes no arm claim at all.** Chat's first draft of this file misattributed it there and the re-drive caught it; **a wrong site under a correct finding is the F-4 species, Chat's again**). **The naming is correct. Two of the three claimed parts do not survive measurement.**

### §3.1 — 🛑 A-1 — THE SIBLING'S APPLIER IS ONE OF THE TWO `owner_id` SITES RIDER 2 FORBIDS

```
752: fn apply_space_temperature_visibility(&mut self, event: &Event) -> Result<(), SpaceError> {
753:     if event.sender != self.owner_id {
754:         return Err(SpaceError::PermissionDenied(...));
```

✅ **RE-DRIVEN at `state.rs:752-763`.** Its sibling `apply_space_pacing` is identical at `:732-737`. **These two lines — `:733` and `:753` — ARE the two production `owner_id` direct-authority sites Clair measured and L-D names.**

🔑 **⇒ THE TEMPLATE IS THE ANTI-PATTERN.** Copying `apply_space_temperature_visibility` verbatim — the obvious, cheapest, most defensible implementation move, and the one a runbook would naturally specify — **produces exactly the third `owner_id` site Rider 2 was written to prevent.** The admission applier must use the `apply_mute` idiom (§2.3), **not** the sibling's.

⚠️ ***A precedent named for the right reason can still be the wrong thing to copy.*** The sibling was chosen on the **set-once-vs-forward-looking** axis, where it is correct. Nobody asked what its **gate** looked like. **This must be an executable constraint in the runbook, not a note** — the failure mode is silent, passes every test, and is invisible in review because it matches two neighbours.

### §3.2 — 🛑 A-2 — THE SIBLING HAS NO RESOLUTION ARM. THE RECORD SAYS IT DOES.

✅ **MEASURED:** `state_key_for_event` (`resolution/state_key.rs:44-161`) has arms for `MembershipJoin|MembershipLeave` · `MembershipKick` · `MembershipInvite|Ban|NodeEject|NodeUnban` · `StateRoomUpdate` · `StateSpaceUpdate` · `ThreadResolved|ThreadArchived` · `StateNodePriority` · `MlsGroupInit` · `MlsCommit` · `SystemKeyRotation`, then `_ => None`.

🛑 **`StateSpaceTemperatureVisibility` and `StateSpacePacing` appear ZERO times in `state_key.rs`** (pattern `TemperatureVisibility|SpacePacing`, case-sensitive, count **0**). **They fall to `_ => None`.**

📌 **And the one hit in the resolution layer is a TEST FIXTURE** — `algorithm.rs:435` is a `SpaceState` constructor inside `#[cfg(test)]`. **`member_temperature_visibility` has ZERO production presence in the resolution layer.**

⇒ **The sibling is a TWO-part precedent (enum + applier), not a three-part one.** The claim *"resolution arm already exists"* is **false** and is a record correction owed (§10, C-1).

🔑 **WHY THIS MATTERS RATHER THAN BEING PEDANTRY.** No `state_key` arm ⇒ no conflict class ⇒ `conflicts_in_log` returns false ⇒ two concurrent admission settings are **not detected as a conflict** and are applied in fold order. ⚠️ **AND THE FIRST DRAFT OF THIS LINE SAID *"convergence is not broken (fold order is deterministic under `D-076`)"* — THAT IS FALSE, AND CLAIR KILLED IT (J-743).** ✅ **Re-driven at `runtime.rs:851-874`:** `let conflict = state_key_for_event(&event).is_some() && conflicts_in_log(…)` — **`state_key_for_event` is the FIRST short-circuit** ⇒ no arm ⇒ `conflict = false` ⇒ the `else` branch runs `state.apply_event(&event, …)` (`:867`), **incremental and in ARRIVAL ORDER**. `derive_resolved` — the only path that reaches the deterministic topological sort — **is never entered**, and the appliers are **last-writer-wins assignments** (`state.rs:761`). ⇒ ***two nodes receiving two concurrent owner-issued settings in different orders hold DIFFERENT VALUES.*** 📌 **Clair closed her own leg on the reconciliation question: the three production `derive_resolved` sites are `runtime.rs:677` (cold-start restore), `:832` (create-arm) and `:857` (the conflict rebuild) — NOTHING reconciles periodically**, and `replay_spaces_from_dir` sorts topologically (`app.rs:5027`), **so it heals only on RESTART.** The client carries the identical gate (`ai_service.rs:547`). **For a visibility preference that is tolerable. For the gate that decides who may enter, it is not** — see §6.3, which Chat closes rather than routes.

### §3.3 — ⚠️ A-3 — "ORIGIN-INDEPENDENTLY" IS THE WRONG WORD, AND THE RIGHT STORY IS BETTER

`docs/ROADMAP.md` records that an invite-required Space *"refuses a third party **by the admission rule itself, origin-independently**"*.

⚠️ **The existing admission-gate precedent is explicitly ORIGIN-SCOPED and says so at the site.** The invite-expiry gate runs `if origin == EventOrigin::LocallySubmitted && event.room_id.as_str().is_empty()` (`runtime.rs:1580`, ✅ RE-DRIVEN), under a comment reading *"INV-EXP (D-1/D-3, C2) — **admission-only** + injected clock … on `ReceivedViaFederation` it is SKIPPED … **A peer trusts the home node's already-made admission decision and does not re-adjudicate**"* (`:1567-1572`).

✅ **The CONCLUSION survives and the mechanism is sounder than the sentence:** it is not one rule holding on both channels, it is **two rules, one per channel, and the new one fills the gap the old one leaves.** Local submissions → the new admission gate. Federation submissions → F-3's relationship check, which already refuses a peer absent from `federation_nodes`. **Correct the word; keep the design.** (§10, C-2.)

🔑 **AND IT NAMES THE GATE'S HOME.** An admission gate belongs **beside the invite-expiry gate in `dispatch_event`, on the `origin == LocallySubmitted` branch** — an established location, an established convention, and the same fail-closed reject-coded shape (`3044 invite_expired` is the pattern to mirror with a new code).

---

## §4 — ✅ F-2's LOCAL-SUBMISSION LEG IS CLOSED. **DRIVEN THIS SESSION.**

**The open question (Clair's own flagged unverified leg, J-741): does `xgen-node` route client submissions as `LocallySubmitted`?** It was cheap, it was unread, and it changes this milestone's priority. **It is now answered: YES.**

| # | fact | site |
|---|---|---|
| **F2-1** | `dispatch_event(event, origin, peer_node_id)` — the F-3 federation-relationship check is `if let Some(peer) = peer_node_id`, under a comment reading *"Runs only for federation-channel events … locally-submitted events skip this check"* | `runtime.rs:1120-1124`, `:1175-1183` |
| **F2-2** | `process_inbound` derives the peer id **from the origin, mechanically**: `EventOrigin::ReceivedViaFederation => Some(...)` / **`EventOrigin::LocallySubmitted => None`** | `xgen-node/src/app.rs:3142-3147` |
| **F2-3** | **`process_inbound` has exactly THREE production call sites.** Two are `handle_federation_incoming` (`app.rs:2432`, `:2600`), both passing `ReceivedViaFederation`. **The third is the client-connection loop inside `run_node` (`app.rs:2004`) and it passes `EventOrigin::LocallySubmitted`**, under the comment *"Origin = LocallySubmitted — client connection is the origination point for federation-push purposes (runbook §3.4.1 R15)"* | `app.rs:2001-2016`, enclosing `pub async fn run_node` at `:558` |
| **F2-4** | **⇒ EVERY event a client submits over its session reaches `dispatch_event` with `peer_node_id = None`, so F-3 is structurally skipped for all of them.** The chain is mechanical, not conditional: no branch, no policy, no exception | derived from F2-1…F2-3 |
| **F2-5** | The client listener's bind address is **operator config** (`config.node.listen`, default `ws://127.0.0.1:8080/xgen`, `app.rs:342`, `:722-731`, `:1235`). **A publicly-hosted node binds publicly by configuration** | `app.rs` as cited |

### 🔑 WHAT THIS IS, STATED PRECISELY AND NOT ONE WORD WIDER

🛑 **⚠️ ANNOTATION AT THE SITE (`D-145`, J-748, 2026-08-16) — THIS SECTION HAS NOW BEEN CORRECTED TWICE IN OPPOSITE DIRECTIONS, AND THE SECOND CORRECTION PARTLY REVERSES THE FIRST. READ THIS BEFORE THE TEXT BELOW.**

✅ **A THIRD GATE EXISTS ON THE LOCAL PATH AND NEITHER SEAT FOUND IT.** `exchange.rs:601-634`, **Step 11, commented *"sender is a registered Identity (universal)"*:** `if !fed_add_via_federation && !node_authored && !id_registry.contains(sender) { return ValidationOutcome::HeldPending { missing_identity: Some(sender.clone()) } }`. **It is NOT origin-scoped** — the branch that consumes it (`runtime.rs:1339-1361`) sits in the common validation path — **and `MembershipJoin` is NOT exempt**: `skip_membership` covers the *membership* check in the block BELOW this one, not this registration check. ⇒ ***an UNREGISTERED keypair's `membership.join` is BUFFERED, not applied***, and is discarded after the 30-second window (`4006`) unless the Identity is registered meanwhile.

🛑 **⇒ `F-1`'s claim that *registration is not required and tenancy is not required* is FALSE, and the original tenancy framing was right.** Clair's F-1 was correct that **AUTHENTICATION** needs only key possession; it over-extended that to **ADMISSION**, which needs registration. *Two different gates, one session boundary between them.*

✅ **AND REGISTRATION IS GATED BY THE AUTH MODULE, MEASURED:** `accept_registration` (`identity/registration.rs:444-512`) skips assertions **entirely** when `local_node`, and otherwise **requires a `trust_assertion`** that `validate_assertion` checks with **Step 1 `if !policy.trusted_issuers.contains(&assertion.issuer) { return Err(AuthModuleUntrusted) }`** (`:231-234`) — and **the default `AssertionPolicy` is EMPTY, *"trust no Auth Module"*** (`runtime.rs:363-369`).

🔑 **⇒ THREE DEPLOYMENT STATES, AND THE HOLE'S SIZE IS DIFFERENT IN EACH:** ① **Local Node mode** (§3.8.8) — assertions bypassed, **anyone may register, the hole is fully open**, and this is the single-user/dev deployment · ② **production, no Auth Module configured — THE DEFAULT** — every registration fails `AuthModuleUntrusted`, **so no new actor can reach the path at all** · ③ **production with a trusted Auth Module** — the actor must present a **valid signed tier assertion for their own Identity**, so ***they are a tier-verified, named person*** — and then they may join any Space on that node **without invitation**, with F-3 never seeing them because they are local.

🎯 **THE FINDING SURVIVES AND ITS SHAPE IS NOW EXACTLY RIGHT: it is a MULTI-TENANCY hole on a node that has admitted real users** — the population who can exploit it is precisely **the node's own legitimate members**, ***which is the population an invite requirement exists for.*** ⚠️ **It does NOT make the hole safe: a verified stranger walking into a DM is still a breach.** 🔑 **But it does make it ATTRIBUTABLE — the no-anonymity pillar is what bounds this hole, and `D-148`'s admission gate is what closes it.**

📌 **SUPERSEDED TEXT FOLLOWS, KEPT PER `D-145`.**

🛑 **"Locally submitted" does not mean "submitted by the operator". ⚠️ AND IT DOES NOT EVEN MEAN "SUBMITTED BY AN IDENTITY HOMED HERE" — CLAIR'S COLD READ (J-743) WIDENED THIS AND CHAT RE-DROVE IT.** `server_authenticate` (`transport/connection.rs:523-576`) is challenge → nonce → `verify_auth_response`, and `verify_auth_response` (`transport/auth.rs:91-123`) **extracts the verifying key FROM the `identity_id` itself** (`parse_identity_id`, `:117`) and checks a signature ⇒ **key possession only, no registry lookup at all.** The sole registry gate is `is_revoked` (`identity/registry.rs:184-189`), which returns **`false` for absent records** — its own doc says *"Unknown identities are not revoked … the auth gate treats absent as not-revoked (Phase 1 local mode admits unregistered keypairs)"* — and the call site (`xgen-node/app.rs:1537-1543`) carries **no `local_mode` conditional.** ⇒ **ANY FRESHLY GENERATED Ed25519 KEYPAIR CAN OPEN AN AUTHENTICATED CLIENT SESSION AGAINST ANY NODE'S LISTENER. Registration is not required; tenancy is not required.** 🔑 ***The hole is not multi-tenancy-conditional — a SINGLE-TENANT node with a public listener has it identically.*** F-3 was written as a **federation** guard and is correct as one. It was never a **tenant-isolation** guard, and nothing else performs that job for a join: G-1…G-4 show the join path has no other space-level gate but already-member and banned.

⇒ **A third party homed on the same node as a DM's parties can submit a `membership.join` for that DM and be admitted as `Role::Member`.** No invite. No permission check. No federation guard.

### ⚠️ THE HONEST BOUNDS ON THIS FINDING — THREE OF THEM, ALL LOAD-BEARING

1. **It is a SOURCE TRACE of the routing, driven to the entry point. It is not a live exploit run against a running node.** Every link is cited above; **no event was submitted.** The next honest step is an executable one (§12, Leg A-bis) and it belongs to the design, not to this audit.
2. **It is not reachable from the desktop client.** `ops::join` has three callers — CLI, AI control plane, pipe — and **zero Tauri commands** (M-2/F-4, J-740/J-741: 20 client + 2 node, none a join). **Reachable today via the CLI or the pipe.** ⚠️ **THIS BOUND AS FIRST WRITTEN SAID *against one's own home node* AND THAT WAS FALSE (J-743)** — no home relationship, no registration and no prior tenancy is required to open the session (see the headline above). **The remaining bound is a CLIENT-BINARY one, not a relationship one**, and it is the weakest of the three.
3. **The attacker needs the `space_id`.** A DM's space id is not published by anything measured here. **That is an obscurity bound, not an access-control one, and it must not be recorded as a mitigation.**

### 🎯 WHAT IT DOES TO THIS MILESTONE'S PRIORITY

**It converts `M-SPACE-ADMISSION` from a feature into a fix.** With the leg closed, the milestone closes a **measured** gap between the shipped tenant model and the shipped guard set, rather than adding a setting. 📌 **Whether that re-sequences it against the UI arc is §6.6 and is Joe's.**

🔒 **RECORD DISCIPLINE:** the ROADMAP node currently reads *"F-2 is a SOURCE TRACE and not a measurement — whether `xgen-node` routes client submissions as `LocallySubmitted` is UNVERIFIED, and until that leg closes the third-party-join hole must not be recorded as a measured security finding."* **That leg has now closed and the node must be updated in the same commit as this file** (§10, C-3). ⚠️ **The clause is discharged for the ROUTING and NOT for the EXPLOIT** — bound 1 above stands and is written into C-3.

---

## §5 — 🛑 THE PREDICATE HAS NO FEEDER: `SpaceState` CANNOT SAY WHO A FORMER MEMBER IS

This is the part of the milestone that is **not** a settings field, and it is the harder half.

| # | fact | site | status |
|---|---|---|---|
| **P-1** | **`apply_leave` leaves NO TOMBSTONE.** It removes the leaver from `members` and from every room, and writes nothing else | `state.rs:1038-1053` | ✅ RE-DRIVEN |
| **P-2** | **The invite was consumed at the first join** — `pending_invites.remove(joiner)` | `state.rs:1022` | ✅ RE-DRIVEN |
| **P-3** | **`apply_invite` bars ALL DM invites unconditionally, as its FIRST statement**, before the target is even read | `state.rs:962-965` | ✅ RE-DRIVEN |
| **P-4** | `SpaceState` carries **no counterpart, invitee, or ex-member field.** Full field list read at `state.rs:186-258`: `space_id · name · topic · auth_tier · max_event_size · home_node · owner_id · is_dm · jurisdiction · e2e_encryption · members · pending_invites · ai_operator_delegations · banned · rooms · federation_nodes · node_priority_order · dm_constraints_active · human_pacing_ms · ai_pacing_ms · member_temperature_visibility · active_mutes · threads` | `state.rs:186-258` | ✅ **NEW, THIS SESSION** |

🔑 **⇒ UNDER INVITE-REQUIRED ADMISSION, A LEAVER IS INDISTINGUISHABLE FROM A STRANGER.** The state that would tell them apart was deleted by the leave, and in a DM the one mechanism that could re-admit them — an invite — is barred outright by P-3.

🛑 **AND ch3 §3.16.1's PROSE ALREADY SAYS SOMETHING THE CODE DOES NOT IMPLEMENT:** *"no third party may be invited"*. **`apply_invite` bars ALL invites, not third-party ones.** The prose describes a target-scoped rule; the code ships a blanket one. **Filed as a spec-vs-build divergence (§10, C-4); it is ch3's to fix and its amendment is its own node, never a rider** — but a consented rejoin needs the bar **relaxed for a returning counterpart**, so this milestone cannot proceed without naming it.

📌 **The obvious repair — a set-once derived field on the `jurisdiction` pattern recording the DM's two parties — is CHAT'S PROPOSAL AND NOT A LOCK.** It is §6.5.

---

## §6 — ✅ THE DECISIONS. **ALL EIGHT RULED (Joe, 2026-08-16, J-744/J-745) PLUS §6.3's BRANCH (J-749). NOTHING HERE IS OPEN.** Each carries `D-121`'s three lenses. 📌 **The heading read *"🔓 THE OPEN DECISIONS. JOE'S, UNRESOLVED, NAMED"* through v1.7 — stale from the moment the rulings landed, and corrected at J-756. *An honesty note that outlives the thing it warned about is a live false record (`N-109`).***

> 📌 **Every option below is written in this file.** J-741's process defect — an option set that existed only in chat could not be audited — is not repeated. **A census is not a partition:** where the set is a partition it says so; where it may not be, it says that instead.

### §6.1 — 🔒 Q1 RULED (Joe, 2026-08-16): **`admission`.** THE FIELD'S NAME

**Options:** (a) `admission` · (b) `join_policy` · (c) `admission_policy` · (d) `open_join` (boolean-shaped).

**① user-visible:** none directly — but it becomes a **wire string** in the event type name (`state.space_<name>`) and therefore **permanent and federated**, and it will surface in any future settings UI label. **② tier consequence: NONE** — admission creates no copy and destroys none; no crypto-shred surface, no T4 durability floor touched, no erasure-fate imposed on another party. **③ resource:** identical across all four.

🎯 **CHAT RECOMMENDS (a) `admission`.** It is **the codebase's own word** — 41 occurrences across `xgen-core`/`xgen-node` comments (*"admission-only"*, INV-EXP, F-3), and `runtime.rs:1567` already calls the invite-expiry gate an *admission* gate. **(b) and (c) invent a synonym for a word the source already uses.** ⚠️ **(d) is rejected on shape, not taste** — see §6.2.

### §6.2 — 🔒 Q2 RULED (Joe, 2026-08-16): **open-enum `String`, values `open` / `invite`; absent ⇒ `open`, present-and-unknown ⇒ `invite`.** ⚠️ **Ruled on the CORRECTED argument (J-743), not the v1.1 one.** THE FIELD'S SHAPE AND VALUE SET

**Options:** (a) **open-enum `String`** on the `member_temperature_visibility` pattern, values `open` (default) / `invite` · (b) **`bool`** (`open_join: true`) · (c) a typed Rust enum.

**① user-visible:** none today (one setting, two states either way). **The difference lands on the FUTURE:** a third value — `request` (ask to join), `tier_gated`, `closed` — is **additive** under (a) and a **breaking wire change** under (b). Joe has already named request-to-join as a live possibility (`L-2`, `M_INTRO_POLICY_PHASE0.md` §3a.7), so the third value is expected rather than merely permitted. **② tier consequence: NONE.** **③ resource:** (a) and (b) are equal to build; (a) costs one `unwrap_or_else` at each of three constructors, exactly as the sibling does. (c) costs a serde impl and **forecloses forward-compat by construction** — an unknown value fails to deserialise.

🎯 **CHAT RECOMMENDS (a), open-enum `String`, values `open` / `invite`, unknown ⇒ treated as `invite`.** 🛑 **⚠️ THE REASON FIRST GIVEN FOR THIS WAS FALSE AND CLAIR INVERTED IT (J-743). THE RECOMMENDATION SURVIVES; THE ARGUMENT UNDER IT DOES NOT, AND JOE MUST NOT RULE ON THE OLD ONE.** The draft claimed the sibling *"treats unknown as `moderator` — its MOST RESTRICTIVE value"*. ✅ **Re-driven at `state.rs:1759-1784`: `moderator` is NOT the most restrictive — `VISIBILITY_SELF_ONLY => false` denies every non-self recipient, while `moderator` admits moderators-and-above.** And the source states the actual rule outright: *"moderator is **the default value** and the fallback for unknown values"* (`:1770-1771`), with `DEFAULT_MEMBER_TEMPERATURE_VISIBILITY = VISIBILITY_MODERATOR` (`wire.rs:641`). ⇒ **the sibling's principle is `unknown ⇒ DEFAULT`, which applied to admission yields `unknown ⇒ open` — the OPPOSITE of the recommendation.** 🔑 **AND THE COLLISION PUT TO JOE WAS THEREFORE MIS-STATED: the sibling's convention and `L-E` point the SAME WAY, both toward `open`.** **The real tension is fail-closed-as-a-security-stance versus the codebase's default-fallback convention** — and admission is the first field where the two diverge, because it is the first one whose fallback is a *gate* rather than a *display rule*. 📌 **A better precedent exists and sits AT THE ADMISSION GATE ITSELF, uncited in the draft (Clair): `runtime.rs:1591` — `.unwrap_or(true); // unparseable ⇒ fail-closed`.** *The right argument was one file away from the wrong one.* ✅ **THE SPLIT STILL STANDS AND IS STILL THE POINT: absent ⇒ `open` · present-and-unknown ⇒ `invite`.** L-E governs **ABSENCE** (every Space that exists today); fail-closed governs a **PRESENT but UNPARSEABLE** value. **They are different facts and must not be collapsed** — but the collision Joe is ruling on is with a *convention*, not with `L-E`.

### §6.3 — ✅ Q3: THE RESOLUTION ARM. **CHAT'S SEAT — CLOSED HERE, NOT ROUTED.**

📌 **Stated rather than routed, per J-618's rule: Joe owns choices between honest options; he does not own whether the system converges on an adjudicated answer or a fold-order artefact.** §3.2 measured that the sibling has no `state_key_for_event` arm.

🔒 **CHAT'S CLOSE: the admission event GETS a `state_key_for_event` arm**, keyed `("state.space_admission", space_id)` — the `StateSpaceUpdate` / `StateNodePriority` shape (one active value per Space).

**Reasoning — UPGRADED AT J-743 FROM AN OBSERVABILITY ARGUMENT TO A CORRECTNESS ONE, ON §3.2's corrected measurement.** Without an arm the event never reaches the resolver at all: it is applied **incrementally, in arrival order, last-writer-wins**, so two nodes can hold **different admission values**. **That is not an unreported conflict; it is a live divergence.** 🛑 **For a visibility preference it is tolerable. For a gate that is evaluated ONCE and IRREVERSIBLY at a join, a divergence window is a CORRECTNESS FAILURE** — the same join is admitted on one node and refused on another, and `L-B`'s *"no past to falsify"* property is exactly what makes it unfixable after the fact. **The cost is one match arm and it is the cheapest item in the milestone.** ⚠️ **The sibling's own missing arm stays out of scope — but §10 C-1's filed follow-up is now URGENT rather than tidy-up: `member_temperature_visibility` and `human/ai_pacing_ms` have a live in-memory divergence TODAY that heals only on restart.** *Its own milestone, never a rider — but it should be filed with that priority, not the one it had this morning.*

🔒 **§6.3's ONE BRANCH — RULED (Joe, 2026-08-16; recorded to disk J-749). A JOIN ADMITTED UNDER A LOSING CONCURRENT ADMISSION VALUE REMAINS VALID.** If two concurrent settings resolve to `open` while a join raced between them, **the join stands.** **Re-adjudicating past joins is exactly the *falsifying the past* property `L-B` says admission does not have** — a join is evaluated **once**, at apply time, against the state then derived.

🔑 **THE RULING EXTENDS `D-148` CLAUSE 7 TO THE CASE CLAUSE 7 DOES NOT NAME.** Clause 7 answers *the owner deliberately changed the setting* — a member admitted while `open` stays a member after `invite`. This answers *the resolver picked a winner after the fact*. **Two different causes, one answer, and the same reason under both: the admission decision is a completed act, and completed acts are not re-opened.**

⚠️ **IT DOES NOT WEAKEN §6.3's OWN CLOSE.** The `state_key_for_event` arm stays **required** — it is what stops two Nodes holding different values **going forward**. **What this ruling refuses is retroactive repair, not convergence.** *A gate that converges late still converges; a gate that re-opens settled joins would falsify the past in order to get there.*

⚠️ **AND THE HONEST RESIDUE, NAMED RATHER THAN ABSORBED:** under this ruling two Nodes that resolved in different orders can end up holding **the same admission value and different membership sets** — the join landed on one and not on the other. 📌 **That is a pre-existing property of the incremental-apply path (§3.2), not one this ruling creates**, and it is `Leg ②`'s subject. **The ruling settles what is CORRECT; it does not assert that two Nodes agree today.**

📌 **⇒ §6 IS NOW FULLY RULED — all eight questions and the branch. No question in §6 is open.**

### §6.4 — 🔒 Q4 RULED (Joe, 2026-08-16): **(b) — STORE `invite` AT DM CREATION.** RIDER 1's MECHANISM — HOW THE DM PINS THE VALUE

**Options:** (a) **DERIVE at gate time** — the gate reads `if space.dm_constraints_active { require_invite } else { <field> }`, and the field is simply never consulted in a DM · (b) **STORE `invite` at DM creation** and refuse the mutation event when `dm_constraints_active` · (c) both.

**① user-visible:** identical in behaviour. **They differ on what a reader SEES:** under (a) a DM's stored admission reads `open` while behaving as `invite` — **an observable that lies**, which is the `G-13` shape this project has named repeatedly. Under (b) the stored value matches the behaviour. **② tier consequence: NONE.** **③ resource:** (a) is one condition; (b) is one constructor line in `from_dm_space_create` **plus** a refusal arm in the applier; (c) is (b) plus a redundant condition.

🎯 **CHAT RECOMMENDS (b).** ⚠️ **And it is not a preference — (a) has a measured failure mode:** `state.dm_promote` flips `dm_constraints_active` to false — **`apply_dm_promote`, `state.rs:659-666`, the assignment at `:664`.** ⚠️ **THE DRAFT CITED `state.rs:238-239`, WHICH IS THE FIELD DECLARATION AND ITS DOC COMMENT, NOT THE FLIP — the claim was true and the site was wrong, and Clair caught it (J-743). That is the `F-4` species, in the section carrying this option's decisive argument, three sections after §3 names the species as Chat's.** **Under (a), a promoted DM silently becomes an OPEN Space at the instant of promotion**, because the field it falls back to still holds the create-time default. **Under (b) it becomes an `invite` Space and the owner may open it deliberately.** 🔑 ***The pin must survive the un-pinning event, and only a stored value does.*** 🛑 **AND A FACT THAT STRENGTHENS (b), ABSENT FROM THE DRAFT AND MEASURED BY CLAIR: `apply_dm_promote` HAS NO PERMISSION GATE AT ALL, and `StateDmPromote` is NOT in `skip_membership` (`exchange.rs:649-659`), so `check_permission`'s `_ => Ok(())` admits any Space member** ⇒ **in a DM, EITHER PARTY can promote.** ⇒ under (a) **the stranger's promotion silently opens the Space**, which is `Rider 1`'s own threat arriving through a different door.

### §6.5 — 🔒 Q5 RULED (Joe, 2026-08-16): **(g) — RETAIN `SpaceMember` WITH A `left_at` MARKER.** ⚠️ **Joe asked which option is QUALITATIVELY superior, which is a different question from the one Chat answered; Chat's (a) was a cost-and-scope recommendation and (g) is the better MODEL.** THE FORMER-MEMBER PREDICATE — WHAT FEEDS IT (§5)

**Options over where the fact lives.** 🛑 **THE DRAFT CALLED THIS SET A PARTITION AND IT WAS A CENSUS — Clair found at least three more homes (J-743), and *a census is not a partition* is now named FOUR times in this arc, this time against the very section written to test for it.** ⚠️ **The set below is the corrected one and is still not claimed to be exhaustive.**

- **(a) A SET-ONCE `dm_parties` FIELD** on `SpaceState`, written at `from_dm_space_create` from the creator + invitee, on the `jurisdiction` pattern (set-once, no mutation event, no applier arm). **A rejoin is admitted iff the joiner is in `dm_parties`.**
- **(b) A MEMBERSHIP TOMBSTONE** — `apply_leave` records the leaver in a `former_members` set. **General to every Space, not DM-specific.**
- **(c) RELAX `apply_invite`'s DM bar** for a target already known to the Space, and re-admit by ordinary invite.
- **(d) DERIVE FROM THE DAG** at gate time — walk the log for a prior `membership.join` by this sender.
- **(e) NO REJOIN** — a leave is terminal and the parties open a new DM.
- 🛑 **(f) RE-SEED `pending_invites` AT LEAVE** — `apply_leave` writes a `PendingInvite { role, invited_by, valid_until }` (`state.rs:99-103`) back for the leaver. **ADDED AT J-743 (Clair). A distinct home: an EXISTING field with EXISTING expiry semantics** — the only candidate that **arrives with the erasure story (b) is criticised for lacking** (`valid_until` bounds it), and **it makes `collect_invite_bootstrap` work for a rejoiner, dissolving §8's item 3 outright.** ⚠️ **Its real cost, stated because a refused option still belongs in the partition: a leave AUTO-MINTS A STANDING RE-ENTRY RIGHT**, which cuts directly against Q11c's *consented* rejoin — the leaver returns without the counterpart consenting again.
- 🛑 **(g) RETAIN `SpaceMember` WITH A `left_at` MARKER** instead of removing from `members`. **ADDED AT J-743 (Clair). Distinct from (b): it changes `members` SEMANTICS**, so **every `is_member` caller inherits it** — including `collect_sync_history`'s gate, which is precisely §8's starvation problem and would be silently altered.
- 📌 **(h) NODE-LOCAL STORAGE** — outside `SpaceState` entirely. **ADDED AT J-743 (Clair); refusable on convergence** (a node-local predicate answers differently on different nodes, the same objection that sinks (d)) — **but it is a home, and a partition names it.**

**① user-visible:** (a)/(b)/(c)/(d) all deliver *"you left, the other party consents, you are back in the same conversation with its history"*. **(e) is a different product**: the record of what was said is unreachable to one party forever, and Q11c's already-named cost — *a leaver needs the other party's consent to see the record again* — becomes *a leaver never sees it*. **⚠️ (b) has a user-visible consequence the others do not: a permanent, federated, never-evicted list of everyone who ever left a Space** — including people who left to get away from it. **② tier consequence:** **NONE for (a)/(c)/(d)/(e).** ⚠️ **(b) is the one option with a real one:** `former_members` is third-party personal data on a federated, replicated object with **no erasure story**, written by an event (`membership.leave`) whose whole purpose is departure. **That is a GDPR-facing surface on the project's own hardest open problem, minted as a side-effect of a rejoin feature.** **③ resource:** (a) ≈ 6 lines + a constructor arm; (b) ≈ 10 lines + an erasure story that does not exist; (c) ≈ 4 lines but scoped to DMs only; (d) is a log walk **inside a hot admission path** and the only option whose cost grows with Space history.

🔒 **RULED (g) — AND THE RULING CORRECTS CHAT'S RECOMMENDATION RATHER THAN OVERRIDING IT.** 🛑 **THE DISQUALIFIER FOR (a), WHICH CHAT DID NOT CATCH UNTIL JOE ASKED THE QUALITATIVE QUESTION: `L-A` SAYS ADMISSION IS SPACE-GENERAL, AND `dm_parties` IS DM-SHAPED DATA.** It answers *can this leaver rejoin their DM* and says **nothing** about how a leaver returns to a 500-member community ⇒ **(a) answers half the milestone's own title and leaves the other half unmodelled — the exact defect that routed this milestone out of `M-INTRO-POLICY` in the first place.** ✅ **WHY (g) IS THE RIGHT MODEL: today `members` conflates *is here* with *was ever here* by FORGETTING, and the forgetting is the bug.** (g) makes membership **a record with a lifecycle** instead of a set, and one source then serves the admission predicate, the sync gate, the audit trail and rejoin — **and `invited_by` survives, which `apply_leave` currently destroys.** 🔑 **AND IT IS THE ONLY OPTION THAT DELIVERS Q11c's WORD *CONSENTED*: `left_at` is a MARKER, NOT A GRANT** — it records that this Identity was a member, never that they may return, so readmission stays a separate explicit act. **(a) and (f) both auto-admit.** ⚠️ **SEEN AGAINST (g) THE OTHERS RESOLVE: (b) is (g)'s fact stored in a SECOND PLACE — two sources of truth for one fact, `D-067`'s exact target, plus the unbounded departures list · (f) reuses `PendingInvite` for a meaning it does not have, *a pending invite nobody issued* — cheap, and it lies in the data model · (a) duplicates into a DM-only field what (g) holds generally · (d)/(h) fail convergence · (e) is a different product.** 🛑 **ANNOTATION AT THE SITE (`D-145`, J-746, 2026-08-16): THE PARAGRAPH BELOW IS FALSE AND IS RETRACTED.** ✅ **`sender` is a TOP-LEVEL ENVELOPE FIELD on `Event`, sibling to `content`, and only `content` is encrypted (`wire.rs:473-494`)** ⇒ **every retained event carries its author's pubkey in PLAINTEXT, and `membership.*` events are themselves retained** ⇒ ***the fact IS retained; `apply_leave` erases the DERIVED SNAPSHOT, not the record.*** **The (g) ruling stands on its own ground — the gate needs the fact AT GATE TIME, which is option (d)'s convergence objection — but this extra argument is WITHDRAWN, and what (g) actually buys here is queryability from state without a log replay.** *Superseded text follows.*

📌 **AND A T4 ARGUMENT FOR (g) THAT JOE'S §6.7 QUESTION PRODUCED, STRONGER THAN THE REJOIN ONE IT WAS RULED ON: `D-093` retains the BYTES and does not retain WHO WAS IN THE ROOM.** Today the protocol **cannot answer *"was this Identity a member when this event was written?"*** — because `apply_leave` erases the record. **For an accountable deployment that is precisely the question a retained record has to survive to answer**, and retaining content while discarding the membership context makes the retention less useful than it looks. **(g) fixes it as a side effect: the membership record becomes part of the retained history rather than a live set that silently rewrites itself.** *Found by Joe asking whether the RATCHET touched T4, which it does not — the question landed on a different option than the one it was aimed at.*

🛑 **THE COST, NOT WAVED AWAY: it is the biggest diff and it changes `is_member` SEMANTICS, so every caller inherits it — including `collect_sync_history`'s gate.** ⚠️ **THAT BLAST RADIUS POINTS TOWARD CORRECTNESS** — those callers read a set that has silently discarded the fact they need — **but `D-071` binds: an `is_member` caller census is a NAMED PREREQUISITE LEG of the rejoin work, not something discovered during implementation.** 📌 **Superseded recommendation, kept because the reasoning is still the record:** It satisfies L-A (Space-general **mechanism**, DM-scoped **data** — a DM has parties, a general Space has none, and the field is simply `None` there), it costs nothing at rest for non-DM Spaces, it is set-once so it rides M8 for free exactly as `jurisdiction` does, and **it stores a fact the DM's own creation event already contains** rather than accumulating a new one over time. 🛑 **(b) is the option that looks most general and is the most expensive**, and its cost is not build time — it is a durable federated record of departures. **(d) and (h) are rejected on convergence: a predicate derived from a partially-synced log, or held node-locally, answers differently on different nodes.** ⚠️ **(f) IS THE STRONGEST CHALLENGER AND CHAT DOES NOT REFUSE IT LIGHTLY** — it is the only option that reuses a shipped field WITH a shipped expiry, and the only one that makes §8 go away rather than answering it. **It is refused on ONE ground and the ground is Joe's to overturn: it converts a leave into a standing re-entry right, so the rejoin stops being CONSENTED** — which is the exact property Q11c was closed on. ⚠️ **(g) is refused on blast radius, not on merit:** changing `members` semantics silently re-points **every** `is_member` caller, including the sync gate this milestone is trying to reason about. *If Joe wants the simplest data model rather than the smallest diff, (g) is the honest alternative and it should be priced properly rather than dismissed.*

⚠️ **ALL FIVE STILL REQUIRE ch3 §3.16.1's INVITE BAR TO BE RE-STATED** (§5, C-4). **The predicate decides WHO may return; it does not by itself make the return legal.**

### §6.6 — 🔒 Q6 RULED (Joe, 2026-08-16): **(c) SPLIT — ship the gate now, defer the rejoin story.** DOES §4 RE-SEQUENCE THIS MILESTONE?

**Options:** (a) **schedule it next**, ahead of the UI arc · (b) **leave it filed**, unscheduled, with §4 recorded · (c) **split**: ship the gate (the fix) now, defer the rejoin story (the feature).

**① user-visible:** (a) and (c) delay every open UI milestone by the length of a protocol + node milestone; **(b) leaves a measured gap open for that same period.** ⚠️ **The gap's real-world exposure today is bounded by reachability (§4, bounds 2 and 3), and by the fact that no node is publicly hosted with untrusted tenants** — which is a fact about the deployment, **not about the code, and it expires the day that changes.** **② tier consequence: NONE.** **③ resource:** (c) is the cheapest path to closing the gap and it is genuinely separable — the gate needs the field, the applier and the dispatch-site check; the rejoin story needs Q5 and a ch3 amendment.

🎯 **CHAT RECOMMENDS (c), and states its own conflict of interest in doing so.** Chat proposed the split; a split that ships the half Chat measured is exactly the shape J-513's *"the schedule was Chat's"* correction was written about. 🛑 **THE SEQUENCING IS JOE'S AND CHAT IS NOT NEUTRAL HERE.**

### §6.7 — 🔒 Q7 RULED (Joe, 2026-08-16): **REFUSED — no ratchet; `open ⇄ invite` both ways.** ⚠️ **Ruled only after Joe sent it back to be checked against T4, which the draft had waved through.** THE RATCHET (L-F)

📌 **A ratchet is a one-way switch: `open → invite` permitted, `invite → open` refused forever.** ① it forecloses re-opening a community that locked itself during an incident — **a real want, permanently refused, with rebuild-and-lose-the-history as the only route back**; ② **NONE — and this is now MEASURED rather than asserted**; ③ one comparison in the applier.

🛑 **⚠️ THE LENS-2 ENTRY IN THE DRAFT SAID *"none"* WITHOUT GROUNDING IT, AND JOE SENT IT BACK: *"can you check it against auth t4? we need that all be accessible even after some members are out."* ✅ CHECKED, AND THE ANSWER HOLDS — BUT THE DRAFT HAD NO BASIS FOR IT.** *`D-121` lens 2 is the one that is easy to answer correctly by reflex and impossible to trust unless it was actually run.*

✅ **`D-093` clause 2, verbatim: *Retained (T4) = a **durability floor on the ciphertext bytes** (don't drop them) **+ an erasure refusal** — NOT a protocol key that can reproduce plaintext.*** ⇒ **T4 "accessible" means the bytes are not dropped and cannot be erased. It is not an access grant.** ✅ **And the layers are disjoint, measured:** the bytes live in the event store and blob store; **`apply_leave` (`state.rs:1038-1053`) mutates `SpaceState.members` and room membership and NOTHING ELSE** — it never reaches `dag/store.rs` or `blob_store.rs`. **Read access is a separate gate: `is_member` at `fanout.rs:488` (`collect_sync_history`).** ⇒ ***leaving, admission and the ratchet delete nothing, so none of them can breach the T4 floor.***

✅ **AND THE RATCHET SPECIFICALLY REDUCES NO ONE'S ACCESS ROUTE:** in a **regular Space**, `admission: invite` still permits invitations (`can_invite` ≥ Moderator, untouched), so a former member can be re-invited — **the ratchet blocks only the reversion to `open`, never a readmission.** In a **DM** the value is pinned regardless, so the ratchet is inert there. 🎯 **⇒ REFUSED, and the deciding argument is that the threat it defends against already has a bigger door: a compromised owner does not need to re-open the Space, they can simply invite the attackers.** ***A permanent restriction that does not remove the threat it is named for.***

🔒 **JOE'S DEFINITION, RECORDED BECAUSE IT IS A REQUIREMENT AND NOT A GLOSS (2026-08-16): *"accessible mean archivable and readable under specific security circumstances."*** ⚠️ **MEASURED AGAINST IT, THE PROJECT MEETS ONE HALF AND HAS NO SEAM FOR THE OTHER:** **ARCHIVABLE ✅ structurally** — nothing deletes on leave, the stores are per-Space and node-local, and `store.range(0)` is reachable to node-local code. **READABLE UNDER SPECIFIC SECURITY CIRCUMSTANCES ❌ NOT BUILT, AND NOT MERELY UNBUILT — THERE IS NO SEAM:** a corpus-wide search for `fn export_` / `fn archive_` / `fn dump_` / `legal_hold` / `worm` (case-insensitive, `*.rs`, `\.claude\` + `\target\` + tests excluded) returns **ZERO hits**, and `admin_ops`'s two `is_member` uses are a Space-filter (`:1077`) and a removal precondition (`:4191`), **not a history read — because no operator history read exists.** 📌 **`D-093` clause 2 RESERVED that capability to the operator/module layer (*"mark + reserve the hook, don't build the vault"*) — the hook does not exist either.** ⚠️ **And even a perfect read path yields CIPHERTEXT: `D-093` clause 1 is universal E2E with no protocol escrow, so "readable" additionally requires key access the protocol deliberately does not provide.** 🛑 **⚠️ ANNOTATION AT THE SITE (`D-145`, J-746, 2026-08-16) — JOE CLARIFIED THE REQUIREMENT AND IT NARROWS WHAT IS ACTUALLY MISSING.** *"just say why the auth module retain this data. that one day can be read if needed and identity could be recorded either. but it depends how the auth module would write data. i think when the communication will be written, the pubkeys will be presented."* ✅ **HE IS RIGHT AND IT IS MEASURED: `sender: IdentityXgid` is a TOP-LEVEL envelope field, sibling to `content`, and only `content` is encrypted (`wire.rs:473-494`) ⇒ IDENTITY IS ALREADY RECORDED WITH EVERY RETAINED EVENT, in plaintext, and it survives crypto-shredding of that event's own content.** 🔑 **⇒ READING A RETAINED ARCHIVE IS A TWO-KEY PROBLEM AND BOTH KEYS LIVE OUTSIDE THE PROTOCOL BY DESIGN: the CONTENT needs decryption keys (`D-093` clause 1 — universal E2E, no protocol escrow, supplied by the accountable deployment at its own tier), and the PUBKEY→PERSON binding needs the Auth Module's own records.** ***That is not a gap; it is institutional independence working — the protocol records WHO in a form nobody can repudiate and deliberately holds neither key.*** ⚠️ **WHAT REMAINS GENUINELY MISSING IS NARROWER THAN THE DRAFT IMPLIED: an operator-facing READ VERB (still zero hits) — not the identity record, which ships.** 📌 **AND THE DEPENDENCY IS JOE'S OWN AND IS `M10`'s: *it depends how the auth module would write data*** — the Auth Module's record format is what makes a pubkey resolvable later, and it is `M10`'s to define, not admission's. **FILED, NOT SOLVED, AND EXPLICITLY NOT THIS MILESTONE'S — admission must not grow it.** Its nearest owners are `M10` (Auth Module Reference Set) and Arc I / PG-02 preservation. *Named here so the requirement has a home rather than an assumption.*

### §6.8 — 🔒 Q8 RULED (Joe, 2026-08-16): **TWO decisions, not one — `D-148` (the admission model) and `D-149` (absent-vs-unrecognised).** Both written to `DECISIONS.md` this session; **the corpus now holds 155 `D` entries, max `D-149`, and the seven duplicate numbers are exactly `D-134`'s seven documented collision splits — no new collision.** THE `D` NUMBER AND WORDING

📌 **Joe's, and Chat makes no recommendation on the number.** What the `D` must bind, on this audit's evidence: **admission is owner-settable and Space-general** · **absent ⇒ open, present-and-unknown ⇒ invite** (§6.2's named collision) · **gate on the role predicate, never `owner_id`** (L-D, and §3.1 shows the trap is live) · **the DM pins by a STORED value that survives `state.dm_promote`** (§6.4). 🔓 **N-197's wording is still owed and is still Joe's** — §4 does not add an instrument failure to it.

---

## §7 — MIGRATION

🔒 **`L-E` is satisfied BY THE PARSE, NOT BY A BACKFILL.** `from_space_create` reads the field as `content["admission"].as_str().map(...).unwrap_or_else(|| DEFAULT_ADMISSION)` — **G-8's measured pattern, byte-for-byte the sibling's.** Every Space that exists today has no key in its `state.space_create` content ⇒ every one derives `open` ⇒ **J-275's model is untouched for every existing Space, and no stored event is ever rewritten.**

📌 **There is no version bump and no migrate function.** A `state.space_create` content key is additive; `EventType::Unknown` accept-as-opaque (`wire.rs:158`) means a Space created by a newer client is carried, not rejected, by an older peer.

⚠️ **But `state.space_admission` events are a different case and it must be said plainly: an OLDER node stores and relays an admission event and NEVER APPLIES IT** — so a mixed-version federation has one Space with two admission values. **That is the accept-as-opaque property working as designed and it is worse than rejection because nothing reports a failure** (the J-732 finding, applied here). 🔓 **Whether admission needs a capability negotiation is the H2 question named at J-601 and is NOT this milestone's** — but it is the first mechanism in the project where accept-as-opaque produces a **security** divergence rather than a rendering one, and it is filed here so the pointer is not dangling.

---

## §8 — THE REJOIN STORY END TO END. **CONVERGENCE MUST BE RE-ARGUED, NOT ASSUMED.**

📌 **Grounded, not recalled:**

| # | fact | site |
|---|---|---|
| **R-1** | `rejoin_anchor_or_root` exists because *"a just-left rejoiner is a non-member, so member-gated sync starves `get_dag_tips`"* | `xgen-client/src/ops.rs:142`, used at `:1699` |
| **R-2** | The primary path is `get_invite_bootstrap` (`ops.rs:1674` → `batch.rs:262`), and `rejoin_anchor_or_root` is its `_ =>` **fallback** | `ops.rs:1674-1699` |
| **R-3** | 🛑 **`collect_invite_bootstrap` REFUSES A NON-INVITEE with `1011 invite_bootstrap_refused`** — asserted by a dedicated test, `collect_invite_bootstrap_refuses_non_invitee_1011` | `fanout.rs:572-577`, `:1485-1498` |
| **R-4** | `collect_sync_history` gates on `is_member`, and `is_member` reads `members` alone | `fanout.rs:485-487`; `state.rs:1284` |

🔑 **⇒ THE TWO PATHS FAIL IN OPPOSITE DIRECTIONS AND A REJOINER FALLS BETWEEN THEM.** A returning ex-member has **no pending invite** (P-2 consumed it) ⇒ `get_invite_bootstrap` returns `1011` ⇒ the client falls back to `rejoin_anchor_or_root`. And they are **not a member** ⇒ `collect_sync_history` serves nothing. **Under today's open-join model the fallback works because the join itself is unconditional. Under invite-required admission it must be re-argued**, and the argument has three parts the design leg owes:

1. **Does the rejoiner's `membership.join` still anchor correctly** when `prev_events` came from the fallback rather than the bootstrap?
2. **What re-issues the consent** — a fresh invite (barred in a DM by P-3), or the §6.5 predicate admitting without one? **These produce different DAGs and the choice is Q5's.**
3. ⚠️ **DOWNGRADED AT J-743 (Clair), AND THE `D-067` INVOCATION IS WITHDRAWN AS UNEARNED.** The draft said a `1011` refusal of a permitted rejoiner is *a gate and a bootstrap disagreeing — the two-sources-of-truth shape `D-067` exists to prevent*. 🛑 **`collect_invite_bootstrap` IS NOT A GATE — it hands out `prev_events`**, and refusing a non-invitee is **designed, with a designed fallback**: `rejoin_anchor_or_root` is reached through the `_ =>` arm at `ops.rs:1699`, whose comment describes the anchor choice deliberately (`:1694-1698`). ⇒ **item 3 COLLAPSES INTO ITEM 1** and is item 1 restated, not a second independent problem. ***Calling one problem two is the inverse of the unification error and it inflates a milestone the same way.*** 📌 **What survives as a real note: the fallback anchors on this client's own last local event or the create root, so the design leg must still confirm that anchor is correct under a gate that can REFUSE the join it anchors.**

🛑 **NAMED, NOT SOLVED. This is the design leg's first item and it is the reason §12 does not put implementation next.**

---

## §9 — WHAT THIS MILESTONE MUST NOT DO

1. 🛑 **It must not re-open §1's lock.** Owner-settable, two riders, default open.
2. 🛑 **It must not copy `apply_space_temperature_visibility`'s gate** (§3.1). **Executable constraint, not a note.**
3. 🛑 **It must not amend ch3.** §3.16.1's invite-bar divergence (C-4) is real and **its amendment is its own node, never a rider** — already Joe's ruling at J-739 for the ch3 work generally.
4. 🛑 **It must not fix the sibling's missing resolution arm** (§3.2). Filed as C-1; its own milestone.
5. 🛑 **It must not record §4 as an exploit.** The routing is measured; the exploit is not run. **The distinction is written into C-3 and must survive into the ROADMAP.**
6. 🛑 **It must not build the DM receiving half.** Join / accept / the pending-invite surface has no node and is Joe's to open (J-740). ⚠️ **This milestone's gate is unreachable from the desktop client until it exists** — that is a sequencing fact for §6.6, not a licence to absorb it.
7. 🛑 **It must not mint a `was-a-member` READ grant.** Q11c closed by re-routing: **leaving SUSPENDS access, a consented rejoin RESTORES it.** §6.5's predicate governs **admission**, never **history access**.

---

## §10 — RECORD CORRECTIONS OWED. **CHAT'S SEAT — NO RULING REQUIRED.**

| # | correction | where |
|---|---|---|
| **C-1** | ✅ **APPLIED.** *"whose event type, applier arm and **resolution arm** already exist"* — **the resolution arm does not exist** (§3.2). Corrected to *"event type and applier arm"* at both sites, with the sibling's missing arm filed as its own finding | `docs/ROADMAP.md` M-SPACE-ADMISSION node · `tasks/M_INTRO_POLICY_PHASE0.md` §3a.7. ⚠️ **`CLAUDE.md` does NOT carry the claim** — verified by a corpus-wide `.md` search for *"resolution arm"* |
| **C-2** | ✅ **APPLIED.** *"origin-independently"* → **two rules, one per channel** (§3.3). The conclusion survives; the word does not | `docs/ROADMAP.md` M-SPACE-ADMISSION node · `tasks/M_INTRO_POLICY_PHASE0.md` §3a.7 |
| **C-3** | ✅ **APPLIED.** *"whether `xgen-node` routes client submissions as `LocallySubmitted` is UNVERIFIED"* — **VERIFIED, `app.rs:2014` + `:3146`** (§4). Replaced with the discharged form **AND its three bounds** — the routing is measured, the exploit is not run | `docs/ROADMAP.md` M-SPACE-ADMISSION node |
| **C-4** | ch3 §3.16.1's *"no third party may be invited"* is **target-scoped prose against a blanket code bar** (`state.rs:963-965`). **File as a spec-vs-build divergence on the ch3 amendment node; do not fix here** | `docs/ch3`, via the ch3-amendment node |
| **C-5** | 📌 **C-4 CHECKED OUT AND IS SHARPER THAN FILED (Clair, J-743).** The row reads `| Invitations | Disabled — no third party may be invited |` at **`docs/xgen_ch3_specification.md:4999`** (✅ re-driven by Chat — the row is there verbatim, one hit corpus-wide). 🔑 **In a 2-member DM the two rules are EXTENSIONALLY EQUIVALENT — they diverge on exactly ONE case: re-inviting the ORIGINAL COUNTERPART, which is the rejoin case.** ⇒ **the divergence is not incidental to this milestone; it is precisely the case this milestone exists to decide** | recorded here; the amendment stays the ch3 node's |

📌 **All four are corrections at the site, not `D-131` annotations** — `D-131` governs **citations proven broken**; these are **claims proven false**, and this arc's standing practice for those is *corrected, not annotated-and-kept* (F-3, J-741).

---

## §11 — FLOORS

🔒 **CARRIED, NOT RE-RUN — this pass wrote zero `.rs`, zero `.ts`, zero `.svelte`, zero `ui/**`:** cargo **1602 / 0 / 62 × 56 SUITES** · vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15**. 🛑 **Sampler catalogue is UNMEASURED** — its harness has never been located. 📌 **`cargo` is not a floor for a reads-only pass**: an identical result over zero `.rs` is a scope argument, not a measurement.

📌 **Census metrics stated:** the `LocallySubmitted` census (§4) is **LINES matching `LocallySubmitted`, case-sensitive, over `*.rs`, with `\.claude\`, `\target\` and `node_modules` excluded** — `\.claude\` holds 8 full source trees `git status` cannot see. The `process_inbound` call-site count (**3 production**) is **call expressions, hand-separated from 50+ doc-comment mentions by reading each hit, NOT by a grep total.**

---

## §12 — 🔒 PROPOSED LEGS. **THE SPLIT IS CHAT'S SEAT (`D-123`); EVERY RULING IN IT IS JOE'S.**

| leg | content | gated on |
|---|---|---|
| **0** | ✅ **THIS FILE.** Audit + §4's closed leg + §10's corrections, committed atomically with the ROADMAP node under `D-074` | — |
| **A** | ✅ **DESIGN LOCK — DONE (J-756). §15 IS THE DESIGN.** §6.1…§6.8 ruled at J-744/J-745, §6.3's branch at J-749, and **four further rulings this session** (Q-2's mechanism, Q-1a, the `3048` invariant, the leg split) | §6 |
| **A-bis** | ✅ **LEG ① SHIPPED AND VERIFIED — J-755, commit `eedfebd`, two files, +330/−0, ZERO production code.** `tasks/RUNBOOK_SPACE_ADMISSION_LEG_A_BIS.md` **v1.5 COMPLETED**. **MEASURED BY CHAT UNDER RULE 5, NOT ADOPTED ON REPORT:** `cargo` **1602 → 1604 / 0 / 62 × 56 SUITES**, and 🔑 **the baseline was MEASURED on this tree rather than carried** — the same workspace run with `--skip space_admission_third_party_join` returns **1602 / 0 / 62 × 56 with exactly 2 filtered out**, so ***the delta is a measurement and not arithmetic against a number carried for twenty sessions.*** 🔒 **V-3, the negative control, RUN AND GREEN (Chat's, a discarded probe, never committed):** the identical fixture with carol UNREGISTERED returns `HeldPending` **and `pending_identity_count` 0 → 1** on a Space whose DAG tips are non-empty ⇒ ***the hold is for a MISSING IDENTITY and cannot be a missing predecessor*** — which `DispatchOutcome::HeldPending`, a UNIT variant, could never have said on its own. ⇒ **`X-1` stops being a claim in a document.** 📌 **What it records:** *a registered third-party Identity is admitted to a DM it is not party to, as `Role::Member`, with no invite — measured in-process on the local dispatch path.* 📌 **Leg ② remains DEFERRED to after `Leg E-0`** and gets its own runbook. **Original row follows.** 🔒 **RUNBOOK NAMED (Joe, 2026-08-16; recorded J-749): `tasks/RUNBOOK_SPACE_ADMISSION_LEG_A_BIS.md`, authored J-749, LEG ① ONLY.** ⚠️ **THE TWO EXECUTABLE LEGS, BOTH STILL OWED — AND ①'s FIXTURE WAS WRONG UNTIL J-748.** 📌 **Leg ② is DEFERRED to after `Leg E-0` and is deliberately NOT in that runbook** — it tests the sibling's divergence, and admission is not built.<br> ① a test that submits a third-party `membership.join` locally against a DM and asserts today's admission, then asserts the new gate refuses it — **the before-assertion is what makes it a regression test rather than a feature test.** 🛑 **THE ACTOR IS A SECOND *REGISTERED* IDENTITY, NOT A FRESH KEYPAIR.** J-743's instruction (*"must use a fresh unregistered keypair or it tests a narrower thing than the hole"*) was **exactly backwards**: `exchange.rs:629` HeldPends an unregistered sender universally, so **a fresh-keypair fixture would assert `HeldPending` and pass while proving nothing** — ***a check whose failure mode reads exactly like success, minted by a correction meant to prevent one.*** 📌 **`phase9_harness`'s `InProcessNode` already supplies the shape; `phase9_unknown_signer_first_contact.rs` is the federation sibling and the model for keypair setup.** ② **a live two-node concurrent-set test for §3.2's divergence** — both Chat and Clair derived it from a measured gate and a measured applier, **and neither ran it**; `phase9_two_node_smoke.rs` is the existing two-node harness | Leg A |
| **B** | ✅ **DONE (J-758).** The field, the constants and the create parse shipped: `admission: String` on `SpaceState`, `ADMISSION_OPEN`/`ADMISSION_INVITE`/`DEFAULT_ADMISSION` in `wire.rs` under their own `spec 3.7.14` banner and re-exported through `wire/types.rs`, `from_space_create` parsing on `member_temperature_visibility`'s idiom, **both DM constructors pinning unconditionally**, and §15.2's fourth literal in `algorithm.rs`. **Four files, +180/−2, all in `xgen-common`/`xgen-core`.** ✅ **cargo 1604 → 1608 / 0 / 62 × 56 SUITES, re-driven by Chat; V-3's negative control fired on the SEMANTIC assertion.** 🛑 **Nothing reads the value — V-6 verified zero `match`, zero `if ==`, zero allow-list.** ⚠️ **`F-3` rides to Leg D: `.as_str()` returns `None` for a present NON-STRING, which therefore stores `open` and is indistinguishable from absent, while `"banana"` survives — `D-149` rules the second case and is SILENT on the first.** Runbook `tasks/RUNBOOK_SPACE_ADMISSION_LEG_B.md` **v1.3 COMPLETED**. **Original row follows.** **THE FIELD + THE CREATE PARSE** — `SpaceState` field, three constructors, `DEFAULT_ADMISSION`, §7's absent-⇒-open derive. **No gate yet: nothing reads it** | Leg A |
| **C** | **THE MUTATION EVENT** — enum + `as_str` + `from_str` (`wire.rs:114-115`, `:215`, `:309`) ⚠️ **plus `known_variants()` at `wire.rs:736-757`, a hand-maintained test vec: omitting the variant there does not fail, it silently drops it from the round-trip test.** *A check whose failure mode reads exactly like success is not a check.* Plus the content struct, the applier arm (**`apply_mute`'s idiom, §3.1**) and §6.3's `state_key_for_event` arm | Leg B |
| **D** | **THE GATE** — in `dispatch_event` beside the invite-expiry gate, on the `origin == LocallySubmitted` branch, fail-closed, new reject code (`3044`'s shape). **This is the leg that closes §4.** 🔒 **AND IT NOW CARRIES `E-0`: `state.rs:1112`'s bare `contains_key` refuses the rejoin `Q-2`(a) promised, and `D-154`①③ require it to gate — otherwise `:1115`'s ban check is DEAD for retained banned members.** 📌 **`F-3`'s cap TAKEN BY CHAT: 64 bytes, char-boundary truncation; every node must truncate identically or the stored value diverges.** 🔒 **DoD ITEM INHERITED FROM LEG A-bis, WRITTEN IN THE EDIT THAT CLOSED IT (`N-109`, J-755): INVERT `third_party_registered_identity_joins_a_dm_it_is_not_party_to`.** Once the gate ships, ***a GREEN run of the UN-EDITED DM test is a FAILURE OF THE GATE, not a pass*** — the test asserts `Accepted` / `Role::Member` / `invited_by: None` for an uninvited third party, which is exactly what the gate exists to refuse. 🛑 **AND THE COMPANION `third_party_registered_identity_joins_an_open_space` IS NOT TOUCHED** — under `D-148` clause 3 an ordinary Space defaults to `open` forever, so it must stay green through Leg D and after it; **if Leg D finds it must be weakened or edited, that is a FINDING about the GATE'S SCOPE** (the gate would be refusing an open join), reported and never absorbed. *A companion edited alongside the thing it was built to outlive was never a control.* 🔒 **AND `3048` RIDES THIS LEG (Joe, 2026-08-18, J-756; §15.6):** the gate must **refuse** a join that is concurrent with a leave on the same state key, rather than accepting it into a fold that will drop it. **Machinery already present — `conflicts_in_log`, `runtime.rs:853`.** | Leg C |
| **E-0** | ✅ **COMPLETE — J-761. `D-071` PREREQUISITE, MINTED BY THE (g) RULING: the `members` READER CENSUS.** 🛑 **THIS ROW READ *"an `is_member` caller census"* UNTIL 2026-08-22 AND THE NAME WAS FALSE (`D-131`, corrected not erased): (g) does not change `is_member` — ***it changes `members***`, and `is_member` is one of FOUR doors, not the census.** ✅ **RESULT: 50 production sites, ALL `CURRENTLY`, `EVER` 0, `INDIFFERENT` 0** — `D-1` 13 · `D-2` 17 · `D-3` 20, plus `D-4` (`resolve_operator`, which NO accessor ruling reaches). **§5's reopen trigger DISCHARGED BY MEASUREMENT ⇒ (i) STANDS.** `tasks/M_SPACE_ADMISSION_E0_PHASE0.md` **v1.2 COMPLETED** | Leg A |
| **E** | **THE REJOIN STORY** — §6.5's `left_at` model, §8's convergence questions, and §8 item 3's surviving anchor note. 🔒 **NOW GOVERNED BY `D-154`'s ~~five~~ SIX clauses** (⑥ added by Joe 2026-08-23, J-766; **`N-2c`, corrected not erased — `D-131`**), and carrying `E-0`'s six open findings (`C-3` mechanical · `C-4` · `C-5` · `C-6` · `C-7` · `F-E`), **all Chat's**. ✅ **SPLIT AND SHIPPED (J-765 … J-770): `E-1` the meaning change · `E-2` clause ④'s slice · `E-3` the close.** 🛑 **`C-3` was NOT mechanical** (J-764) and **`F-E`'s citation was FALSE** (J-765) | Legs A + D + **E-0** |
| ~~**G**~~ | 🛑 **SUPERSEDED BY THE ROW BELOW — `N-2a` (`D-134`: designations are issued unique; this table carried `G` TWICE from 2026-08-18 to 2026-08-23). Struck, not deleted (`D-131`).** 🛑 **AND `N-2b`: this row cited `D-154`③ for *the gap stays closed*, which is clause ④ — ③ is *ban follows kick*. A reader who followed the citation landed on the wrong clause.** ~~THE REJOIN ANCHOR — `get_rejoin_anchor`. `D-154`~~ **④** ~~MAKES THIS LOAD-BEARING, NOT A FORMALITY:~~ *the gap stays closed* requires a per-member history boundary, and **the gap must be MARKED — a silent jump reads as *nothing was said*, which is the direction `D-065` governs. The marker's wording and form are Joe's** | Leg E |
| **G** | 🔒 **THE REJOIN ANCHOR VERB — ITS OWN LEG (Joe, 2026-08-18, J-756), NOT A RIDER ON E.** A node-side `get_rejoin_anchor`, sibling of `collect_invite_bootstrap`: the rejoiner asks the Node for **her own** last membership event and anchors on it, so the causal anchor stops depending on client-local memory. **New wire surface ⇒ Joe's seat ⇒ its own leg**, sequenced after D, **and it may slip without leaving a silent failure behind, because `3048` already made the residue loud** | Legs A + D + E |
| ✅ **`E-3` SHIPPED (J-772)** — ~~**F**~~ | 🛑 **`N-2d` (J-771): `F` and the Leg E Phase-0's `E-3` ARE THE SAME LEG UNDER TWO NAMES.** The Leg E Phase-0 split `E` into `E-1`/`E-2`/`E-3` **without reconciling against this table, which already defined `F` as the close** — a leg list held in two documents, and the second one written without reading the first. **Merged: `E-3` IS the close.** ~~**F** — CLOSE, records, `D-074` atomic commit, `roadmap-format-gate.ps1` exit 0~~ | all |

🔓 **Legs are Chat's proposal; the leg list is Joe's to lock, and §6.6 may cut it at D.**

---

## §13 — ✅ CLAIR'S COLD READ: **RUN AT J-743. GO WITH FINDINGS — SIX, AND TWO OF THEM MOVED PREMISES JOE WAS ABOUT TO RULE ON.**

✅ **EVERY FINDING RE-DRIVEN BY CHAT AGAINST SOURCE BEFORE IT ENTERED THIS FILE (Rule 5, both ways). Nothing adopted on report.**

| # | finding | verdict |
|---|---|---|
| **F-1** | 🛑 **§4's bound is NOT tenancy.** `server_authenticate` is key-possession only (`connection.rs:523-576` → `auth.rs:91-123`, `parse_identity_id` at `:117`); `is_revoked` admits absent records by documented design (`identity/registry.rs:181-189`) and the call site has no `local_mode` conditional (`app.rs:1537-1543`) ⇒ **any fresh keypair opens an authenticated session against any listener; a SINGLE-TENANT node has the hole identically** | ✅ **RE-DRIVEN. CONFIRMS the finding, KILLS both stated bounds on its reach.** Folded into §4 |
| **F-2** | 🛑 **§6.3's convergence parenthetical is FALSE.** The `state_key_for_event` short-circuit (`runtime.rs:852`) means a keyless event never reaches `derive_resolved` — it is applied incrementally in arrival order, last-writer-wins ⇒ **live divergence, healed only on restart** | ✅ **RE-DRIVEN. KILLS the parenthetical, STRENGTHENS the close.** §3.2 and §6.3 rewritten; reason upgraded observability → **correctness** |
| **F-3** | 🛑 **§6.2's premise is INVERTED.** `moderator` is not the most restrictive value (`self_only` is); the source says it is **the DEFAULT** and the fallback for unknowns (`state.rs:1770-1771`) ⇒ the sibling's principle is `unknown ⇒ default`, which yields `unknown ⇒ open` | ✅ **RE-DRIVEN. Recommendation survives, argument replaced, and the collision put to Joe was mis-stated.** Better precedent cited: `runtime.rs:1591` |
| **F-4** | 🛑 **§6.4 cited a doc comment for its load-bearing claim** — the flip is `apply_dm_promote` (`state.rs:659-666`), not `:238-239`. **Claim true, site wrong — the `F-4` species, Chat's again** | ✅ **RE-DRIVEN. Site corrected, and Clair's strengthener folded in: `apply_dm_promote` has NO permission gate and is not in `skip_membership` ⇒ either DM party can promote** |
| **F-5** | 🛑 **§6.5 was a CENSUS claiming to be a partition** — at least three more homes: (f) re-seed `pending_invites`, (g) `left_at` on a retained `SpaceMember`, (h) node-local | ✅ **CONFIRMED. All three added.** *A census is not a partition — FOURTH instance, and this time against the section written to test for it* |
| **F-6** | 🛑 **§8 item 3 is overstated and its `D-067` invocation is unearned** — `collect_invite_bootstrap` hands out `prev_events`, it is not a gate, and its refusal has a designed fallback ⇒ item 3 collapses into item 1 | ✅ **CONFIRMED. Downgraded; the `D-067` claim withdrawn** |

📌 **CLAIR NAMED HER OWN UNVERIFIED LEGS, WHICH IS THE STANDARD:** she ran no exploit either (F-1 is a source trace on the same bound as §4's bound 1), and **F-2's divergence is derived from the measured gate and the measured applier, not from a two-node run** — ***the live concurrent-set test belongs beside `Leg A-bis` and is now written into §12.*** ✅ **And she closed a leg during the read: the exhaustive `derive_resolved` census — 3 production sites, none periodic.**

🔑 **THE SCORE, RECORDED BECAUSE IT KEEPS BEING THE SAME SCORE: Chat's own re-reads have STILL never caught a defect in this arc. Clair's four cold reads have now returned seven, six, six and six** — and this one **corrected two premises that were one ruling away from being locked.**

---

### 📌 THE ORIGINAL BRIEF, KEPT FOR THE RECORD

🔑 **THE EVIDENCE IS UNAMBIGUOUS AND IT IS AGAINST CHAT: Chat's own re-reads have never once caught a defect in this arc. Clair's three cold reads returned 7, then 6, then 6 — and the third moved the milestone.**

**Point her at, in this order:** ① **§4's five-link chain** — the whole priority argument rests on it and Chat drove it alone · ② **§3.1 and §3.2**, which contradict two canonical records · ③ **§5/§8**, the half with no code behind it at all · ④ **§6.5's option set**, explicitly as a partition test: *is there a sixth place the former-member fact could live?* · ⑤ **§6.2's absent-vs-unknown split**, the one place a principle and a lock point in opposite directions.

⚠️ **AND RULE 5 RUNS BOTH WAYS.** At J-741 Chat's re-drive found a real defect in Clair's census. **Adopt nothing; reproduce everything.** 📌 **Every option in this file is ON DISK** — the J-741 process defect is not repeated.

---

## §14 — DoD FOR PHASE-0 (LEG 0)

- [x] This file exists at `tasks/M_SPACE_ADMISSION_PHASE0.md`, header correct, `Status: ACTIVE` — **39,664 B, LF-only, no BOM, verified on disk**
- [x] §4's leg is closed with all five sites cited and all three bounds stated
- [x] §10's C-1, C-2 and C-3 are applied — **to `docs/ROADMAP.md` and `tasks/M_INTRO_POLICY_PHASE0.md`, NOT `CLAUDE.md`, which never carried the claim**; C-4 is filed, not fixed
- [x] The `M-SPACE-ADMISSION` ROADMAP node gains its Phase-0 pointer and two new Owes: rows (§3.1 the gate trap · §5/§8 the feeder and the rejoin split); §3.2 and §4 landed as in-place corrections
- [x] `roadmap-format-gate.ps1` returns exit 0 — **PASS, tree lines 73..450 clean**; ROADMAP v7.27 → **v7.28**; CRLF integrity re-asserted (CR 600 == LF 600, zero CRCR, zero lone LF, no BOM)
- [x] JOURNAL entry written (**J-742**); `CLAUDE.md` PLAY block updated (**head, above J-741; CR 1620 == LF 1620, zero CRCR, zero lone LF, no BOM**); ROADMAP version bumped (**v7.28**) — **one commit, `D-074`**
- [x] Every open § in §6 carries options, three lenses and either a recommendation or an explicit refusal to make one
- [x] **Clair's cold read RUN (J-743), GO WITH FINDINGS, all six re-driven by Chat and folded in at their sites** — §4 widened · §3.2 + §6.3 corrected and upgraded · §6.2's premise replaced · §6.4's site fixed · §6.5's census completed · §8 item 3 downgraded
- [x] 🔒 **§6.1–§6.8 ALL RULED (Joe, 2026-08-16, J-744 + J-745), AND §6.3's ONE BRANCH RULED AND ON DISK (J-749).** ⚠️ **This box was stale for four versions** — the eight rulings landed at J-744/J-745 and v1.4/v1.5/v1.6 each carried the ruling at its own § while leaving the DoD saying nothing had been ruled. **Flipped in Chat's own seat (record accuracy, `D-123`), and the staleness named rather than quietly corrected:** *a DoD item that contradicts the sections above it trains the reader to skip the DoD.* 📌 **Nothing was ever locked against the v1.1 text; the ruling sites all cite the corrected arguments.**

📌 **"Commit pushed" is deliberately not a DoD item** — `Status: COMPLETED` in this header is the shipped signal, and this file stays `ACTIVE` until the milestone closes at Leg F. ✅ **Leg A locked the design in at v2.0 (J-756); §15 is it.**

---

## §15 — 🔒 THE DESIGN. **LEG A's DELIVERABLE. EVERY SITE MEASURED AT `b3ccb77`, NONE RECALLED.**

📌 **Provenance:** §6.1–§6.8 (Joe, 2026-08-16) + §6.3's branch (J-749) + **four rulings taken 2026-08-18 (J-756)**: Q-2's rejoin mechanism **(a)**, Q-1a **(i)+(iii)**, `3048` riding Leg D, and the anchor verb as its own leg. 🛑 **Sections marked 🔓 are NOT ruled and must not be implemented from.**

🛑 **A CORRECTION MADE WHILE WRITING THIS SECTION, RECORDED BECAUSE IT NEARLY SHIPPED:** the session's own recommendation said *"a `former_members` (or `left_at`) set"*, straddling **(b)** and **(g)** as if they were one option. §6.5 **ruled (g) and REFUSED (b) explicitly** — (b) is *"(g)'s fact stored in a SECOND PLACE, `D-067`'s exact target, plus the unbounded departures list"* with a named GDPR surface. ⇒ **the design below is (g) and only (g).** 🔑 ***Reading the ruling at its site is what caught it; re-reading the summary would not have.***

### §15.1 — THE FIELD

| what | where, measured |
|---|---|
| `pub admission: String` | `SpaceState`, `xgen-core/src/space/state.rs:186-258` — **append after `threads` (`:257`)**. Sibling in kind to `member_temperature_visibility` (`:249`): an **open enum**, `String`, no `enum` type |
| `pub const DEFAULT_ADMISSION: &str = ADMISSION_OPEN;` | 🛑 **CORRECTED AT v2.1 (J-757) — `xgen-common/src/wire.rs`, beside the other protocol defaults (`DEFAULT_HUMAN_PACING_MS:600`, `DEFAULT_AI_PACING_MS:603`, `DEFAULT_MEMBER_TEMPERATURE_VISIBILITY:641`), NOT `state.rs`.** ✅ **And the `VISIBILITY_*` pattern applies (`wire.rs:633-637`): the permitted values get named constants too** — `ADMISSION_OPEN = "open"`, `ADMISSION_INVITE = "invite"`, and `DEFAULT_ADMISSION = ADMISSION_OPEN`. 🛑 **v2.0 SAID `state.rs` HAS NO `const` BLOCK AND THAT WAS FALSE:** the sweep used `^\s*(pub )?const `, `state.rs` genuinely has none — **but the three defaults it consumes are DECLARED IN `wire.rs` AND IMPORTED**, so the census answered the question it was built to ask and not the one that mattered. ***A census cannot see what it was not built to enumerate*** — caught by reading `from_space_create`'s body while grounding Leg B, not by re-reading §15 |
| convergence | `SpaceState` derives `Debug, Clone, PartialEq, Eq` (`:185`) and **is never serialised** — it is folded from the log. A `String` field is covered additively by the `derive_resolved` oracle, exactly as `jurisdiction` and `e2e_encryption` are (`:206`, `:219`) |

🛑 **CORRECTED AT v2.2 (J-757) — THIS ITEM WAS NEVER OPEN. `D-149` RULED IT ON 2026-08-16, IN THIS MILESTONE, AND §6.2's OWN 🔒 HEADING SAYS SO: *"absent ⇒ `open`, present-and-unknown ⇒ `invite`"*.** §6.8 names `D-149` by number and §6's heading — **corrected one session earlier** — says *"ALL EIGHT RULED… NOTHING HERE IS OPEN"*. **Four records disagreed with v2.0 and v2.0 was the one that was wrong.**

🛑 **AND THE v2.0 TEXT DID SOMETHING WORSE THAN CALL A RULED ITEM OPEN: IT REINSTATED THE ARGUMENT `D-149` EXPLICITLY RETRACTED.** It recommended fail-closed *"on `member_temperature_visibility`'s unknown ⇒ most-restrictive precedent (`:247-248`)"* — and **`D-149`'s own text names that premise FALSE**: `VISIBILITY_SELF_ONLY` denies every non-self recipient while `moderator` admits moderators-and-above, so the sibling's convention is *unknown ⇒ **default***, which for admission would yield `open`, **the opposite of what it was cited to support.** 🔑 ***The recommendation survived; the argument under it did not — and v2.0 cited the dead argument by line number, in the milestone that killed it.***

🔒 **THE RULE, AS `D-149` STATES IT: a field that GATES fails CLOSED; a field that governs DISPLAY takes its DEFAULT. `admission` gates ⇒ unrecognised ⇒ behave as `invite`.** ✅ **AND THE CONSTRUCTORS ARE UNAFFECTED, WHICH IS WHY THIS SURVIVED SO LONG UNSEEN:** both of `D-149`'s own precedents **interpret at USE, not at PARSE** — `should_include_member_temperature` (`state.rs:1759-1784`) and the expiry gate's `.unwrap_or(true)` (`runtime.rs:1591`). ⇒ **§15.2 stores the value verbatim and §15.4's gate is where fail-closed lives.** 📌 **Same code as v2.0 described; a different reason — and the reason is what Leg D reads.**

### §15.2 — THE CREATE PARSE

**Three constructors — MEASURED, not inherited from §6.2's count:** `from_space_create` **`state.rs:265`** · `from_dm_space_create` **`:342`** · `from_dm_space_create_node` **`:496`**. ✅ **Three is correct.** 📌 **Their three `SpaceState` literals end at `threads: HashMap::new()` — `:335`, `:468`, `:583` — which is the insertion anchor in each.**

✅ **THE IDIOM IS ALREADY IN THE FILE AND IS COPIED, NOT INVENTED:** `member_temperature_visibility` at `state.rs:307-310` is `content["…"].as_str().map(str::to_string).unwrap_or_else(|| DEFAULT_….to_string())` — **the exact shape §7 describes**, sitting three lines above where `admission` will be parsed.

📌 **ADDED AT v2.3 (Clair `F-5`, J-758): THE TWO DM CONSTRUCTORS ARE NOT SYMMETRIC.** At `f45bb13`, `from_dm_space_create` **hard-sets** `member_temperature_visibility` (`:466`) while `from_dm_space_create_node` **parses it from content** (`:554`). **It changes nothing for `admission` — both pin regardless — but it is why the two DM tests must stay SEPARATE** rather than folding into one two-constructor test on the `jurisdiction`/`e2e_encryption` precedent. 🛑 **Both citations are `f45bb13`-anchored per `D-152` clause 1;** post-Leg-B the same lines are `:497` and `:586`.

- `from_space_create` → read `content.admission`, **`unwrap_or_else(|| DEFAULT_ADMISSION.to_string())`** (§7's pattern; absent ⇒ `open`, `L-E`).
- **Both DM constructors → pin `"invite"` unconditionally** (`L-C`, §6.4 ruling **(b)**), **ignoring content**.

### §15.3 — THE MUTATION EVENT

`state.space_admission` — **the wire name is already fixed by §6.3's state key** `("state.space_admission", space_id)`.

| site | measured |
|---|---|
| variant + `as_str` | `xgen-common/src/wire.rs:168` |
| `from_str` | `wire.rs:429` |
| ⚠️ `known_variants()` | `wire.rs:736` — **hand-maintained; omission does not fail, it silently drops the round-trip case.** *A check whose failure mode reads exactly like success is not a check* |
| `state_key_for_event` arm | `xgen-core/src/resolution/state_key.rs:44` — the `StateSpaceUpdate` / `StateNodePriority` shape, **one active value per Space** |
| applier | new `apply_space_admission`, sibling of `apply_space_temperature_visibility` (`state.rs:752-768`) |
| 🔒 DM refusal | the applier **refuses when `dm_constraints_active`** (§6.4(b)). ⚠️ **`apply_dm_promote` (`state.rs:659-666`, the flip at `:664`) has NO permission gate and `StateDmPromote` is not in `skip_membership` — in a DM EITHER PARTY can promote**, which is why the value is stored and not derived: ***the pin must survive the un-pinning event, and only a stored value does*** |

🔒 **RULED (Joe, 2026-08-18, J-759) — §15.3's open item is CLOSED: `admission` is changeable by the OWNER ONLY, via a NEW `can_change_admission(role) -> *role == Role::Owner` in `membership.rs`'s permission table** (`:126-163`, on `can_manage_federation:150`'s form). 🔑 **The table, not an inline check** — *a permission that exists only inside an applier is one a future leg will not find*, and `membership.rs`'s banner is where a reader looks for who-may-do-what.

🛑 **AND THE MECHANISM QUESTION EXPOSED A LIVE `D-151` CLAUSE-1 DEFECT IN THE NEAREST SIBLING.** `check_permission` (`exchange.rs:807-916`) ends `_ => Ok(())` at `:914`, and **`StateSpaceTemperatureVisibility` appears NOWHERE in that function** — its owner check lives only in its applier (`state.rs:786`). ⚠️ **Applier errors are DISCARDED at production call sites** — `let _ = …apply_event(…)` at **`runtime.rs:867` · `derive.rs:231` · `ai_service.rs:553`** (🛑 **CORRECTED at v2.5, Clair `F-3`: v2.4 cited `exchange.rs:1279` and `:2319`, and BOTH are inside `#[cfg(test)]`, which opens at `exchange.rs:1096`. Conclusion unchanged; evidence wrong**). ⇒ ***today a non-owner temperature-visibility change is validated, accepted, persisted, returned `Accepted` to its sender — and then dropped by the fold with the error thrown away.*** 🔒 **Leg C therefore places the check in `check_permission` (where `ExchangeError::PermissionDenied` maps to a wire reject at `runtime.rs:1519-1525`) AND keeps an applier copy as defence-in-depth for the federated/replay path.** 📌 **The sibling's own defect is NOT fixed here — filed separately, because riding it in would make Leg C's diff argue two cases at once.**

🔒 **RULED (Joe, 2026-08-18, J-759) — REJECT CODES, measured against ch3 §3.6.10.10's registry (`xgen_ch3_specification.md:2185-2194`):** the **DM refusal** takes a new **`3049 admission_immutable`**; the **plain non-owner refusal** stays on **`RejectInfo { code: 4000, name: "generic" }`**, the unmapped-fallback band. 🔑 **The split is what a client can ACT on:** *"this is a DM, its admission is fixed"* is a different message to a user than *"you lack permission"*, and the second reads correctly as an ordinary permission failure. 📌 **`3047` (the gate) and `3048` (the invariant) remain reserved; `3040`–`3046` are assigned, and 3000–3999 is the identity domain with the 3040s its membership-authority sub-band.**

🛑 **AND THE DM CHECK GOES IN `check_permission`, NOT ONLY THE APPLIER (Clair `F-1`, J-759).** The first draft put the role check in both places and the DM check in the applier alone ⇒ ***an OWNER changing admission on a DM would pass the role check, be persisted, be refused by the applier, have that error discarded, and be answered `Accepted`*** — **§15.3's own DM rule delivered by nothing, and the fix reproducing the defect it was written against.** ✅ **Zero cost: `check_permission` already takes `&SpaceState` and `dm_constraints_active` is a `pub` field.**

⚠️ **A THIRD LIVE INSTANCE, FOUND THE SAME WAY AND NOT FIXED HERE: `apply_invite`'s DM bar (`state.rs:995-998`) IS APPLIER-ONLY TOO** ⇒ an invite in a DM today is accepted, persisted, answered `Accepted`, and silently dropped. **Filed separately** — riding it in would make Leg C's diff argue two cases at once.

🛑 **CORRECTED AT v2.6 (J-760): v2.5's CITATION WAS FALSE.** It said the federated path reaches `check_permission` via **`runtime.rs:1426`** — **that call sits INSIDE `if matches!(event.event_type, StateAiOperatorDelegate | StateAiOperatorRevoke)` opened at `:1416`, so `state.space_admission` can never reach it.** ✅ **The claim is TRUE by a different route: `validate_event` step 13 calls `check_permission(event, space)?` at `exchange.rs:256`, plus the dispatch-side call at `:752`.** ⇒ the applier copy is defence-in-depth **for REPLAY**, the only path that bypasses validation. 🔑 ***The finding arrived from Clair as a NOTE, was folded into a locked runbook and into this file, and was recorded as "re-driven" — without the cited line ever being opened. See `D-153`.***

🛑 **A SECOND MEASURED ASYMMETRY, AND §6.3's KEY HAS NO LOCAL PRECEDENT:** `state_key_for_event` has **no arm for `StateSpaceTemperatureVisibility`** — the only Space-scoped arm is `StateSpaceUpdate` (`state_key.rs:95`). ⇒ **`admission` will be per-field conflict-resolved and the sibling is not**, which is correct under §6.3 but means **the arm is written from `StateSpaceUpdate`'s shape, not copied from the temperature twin.**

🛑 **AND A THIRD: `known_variants()` (`wire.rs:770-791`) HAS NO COMPLETENESS CHECK.** All three consumers (`:806`, `:818`, `:864`) **iterate** the vec ⇒ **omitting a variant is invisible to every one of them.** Leg C adds a **count assertion** so the silent hole becomes a caught one for every future leg.

### §15.3b — 🔒 SETTING `admission` AT CREATION (Clair's `F-3`, RULED J-759)

**Neither builder takes an `admission` argument, and widening them costs 165 call sites across four crates (measured at `f45bb13`).** 🔒 **RULED: a SECOND constructor, `build_space_create_event_with_admission`**, on the `jurisdiction`/`e2e_encryption` precedent — **zero existing call sites touched.** 🔑 **Its purpose is to close a RACE, not to save typing:** without it a Space is created `open` and only becomes `invite` when the mutation event lands, ***leaving a federated window in which an invite-only Space admits strangers — exactly the class this milestone exists to close.*** 📌 **If the widened single builder is later preferred, it can absorb the second constructor; the reverse is harder.**

### §15.4 — THE GATE

🔒 **SITE, MEASURED: `xgen-core/src/node/runtime.rs`, immediately after the invite-expiry block ends at `:1612`, inside the same `if origin == EventOrigin::LocallySubmitted && event.room_id.as_str().is_empty()` branch opened at `:1580`.** The `MembershipJoin` arm; expiry first, admission second.

🔒 **REJECT CODE `3047`, `admission_refused` — MEASURED FREE.** 3044/3045 are the invite-admission family (`dag/pending.rs:97`), **3046 is taken at `resident.rs:1374`**. §12's *"3044's shape"* meant `RejectInfo::coded`'s shape, **not the number**.

**Predicate — admit iff any of:** the Space resolves to `open` · the sender holds a `pending_invites` entry · **the sender is a FORMER MEMBER (§15.5)**. Otherwise `Rejected(3047)`. **Fail-closed**, `LocallySubmitted` only — a federated join is **not** re-adjudicated (`INV-EXP`'s D-1/D-3 precedent at `:1567-1579`).

### §15.5 — THE REJOIN PREDICATE — §6.5's **(g)** FOR THE FEEDER, AND **Q-2's (a)** FOR THE PREDICATE. 📌 **`Q-2` IS THIS SESSION'S RULING (J-756) AND HAS NO §6 NUMBER — the draft cited a §6.9 that does not exist, which is `N-198`'s species caught in its own design section.**

🔒 **`SpaceMember` gains `pub left_at: Option<String>`** (`state.rs:74-83`, beside `invited_by:82`). 🔒 **`apply_leave` STOPS REMOVING** — `state.rs:1046`'s `self.members.remove(leaver)` becomes a lookup + `left_at = Some(ts)`; the room-membership stripping at `:1049-1051` is unchanged.

🔒 **Q-2 RULED (a) (Joe, 2026-08-18): a former member is re-admitted WITHOUT an invite** — the predicate is `members.get(sender).left_at.is_some()`. 🔑 **This was not a free choice for DMs and the measurement is why:** `apply_invite`'s **first statement** bars all DM invites unconditionally (`state.rs:962-965`), and `L-C` pins DMs to `invite` ⇒ **without (a), Leg D makes DM departure IRREVERSIBLE for both parties — including the DM's own counterpart.** 📌 **`left_at` is a MARKER, NOT A GRANT** (§6.5): it records that this Identity was a member; the return is still a separate explicit act.

✅ **NO BACKFILL, AND THIS IS MEASURED, NOT HOPED: `SpaceState` IS NEVER PERSISTED — IT IS FOLDED FROM THE STORE.** `derive_resolved(store.range(0))` at **`runtime.rs:677`** (cold start, via `replay_spaces_from_dir`), **`:832`** (create) and **`:857`** (conflict rebuild). Leave events are already on disk in every Space ⇒ **`left_at` regenerates itself at the next cold start.** ⚠️ **The one honest limit: regeneration is on REBUILD.** The incremental path (`:866`) mutates live state, so a Node that never restarts carries pre-existing leavers as invisible until it does. **Statable, small, and not a blocker.**

🛑 **`E-0` IS A HARD PREREQUISITE (`D-071`):** (g) changes `is_member` semantics, so **every caller inherits it** — including `collect_sync_history`'s gate at `xgen-node/src/fanout.rs:488`, which is §8's starvation problem. **The census precedes Leg E; it does not surface during it.**

### §15.6 — 🔒 `3048` — THE `Accepted ≠ member` INVARIANT. **RIDES LEG D (Joe, 2026-08-18).**

🔑 **THE DEFECT, MEASURED END TO END.** Under (a) a rejoiner needs no invite ⇒ `get_invite_bootstrap` yields nothing (`xgen-client/src/ops.rs:1674`) ⇒ `get_dag_tips` starves (non-member, `fanout.rs:488`) ⇒ **`rejoin_anchor_or_root` (`ops.rs:142`) becomes the PRIMARY path, not the defensive one its own comment calls it.** With `last_local_events` absent — reinstall, cleared state, **a second device**, a leave taken elsewhere, or the best-effort write failing at `ops.rs:1842-1847`, which **explicitly degrades to root** — the join anchors on the **create root** ⇒ it is concurrent with the leave on `membership:{space}:{identity}` ⇒ frontier ≥ 2 ⇒ a genuine conflict set ⇒ **`algorithm.rs:146-147`'s Layer 1 table picks `MembershipLeave` over `MembershipJoin`** ⇒ `derive.rs:113-119` **excludes the loser.**

🛑 **⇒ THE NODE ACCEPTS, PERSISTS, RETURNS `Accepted` — AND THE FOLD DROPS IT. TWO SOURCES OF TRUTH INSIDE ONE NODE, SILENTLY.**

📌 **NOT created by admission** — the same fold drops the same rejoin today, and `MP-F7` mitigated it client-side. **What Leg D changes is that the mitigation's failure modes stop being an edge case and become the normal path.**

🔒 **THE RULING: the Node REFUSES rather than accepting-then-dropping.** New code **`3048` `rejoin_not_anchored`**, same gate block. ✅ **The machinery already exists** — `conflicts_in_log` is called at `runtime.rs:853`. 🔑 ***It closes `Accepted ≠ member` as an INVARIANT rather than as a case, and it reaches old clients that `§15.7` never can.***

### §15.7 — 🔒 `get_rejoin_anchor` — **LEG G, ITS OWN LEG**

A node-side sibling of `collect_invite_bootstrap` (`xgen-node/src/fanout.rs:572-577`): the rejoiner asks her Node for **her own** last membership event; the join anchors `prev_events=[leave_id]` ⇒ linear `j→lv→rj` ⇒ **frontier of size 1 ⇒ no conflict set ⇒ the fold applies it.** **Authorization is trivial — she asks for her OWN event, no disclosure surface.** An older Node that cannot answer ⇒ the client falls back to exactly today's behaviour. **New wire verb ⇒ Joe's seat ⇒ not smuggled into Leg E.**

### §15.8 — WHAT §15 DELIBERATELY DOES NOT DECIDE

🔓 unrecognised-value semantics — 🛑 **STRUCK AT v2.2: `D-149` RULED IT, and listing it here was the same error twice in one document** · 🛑 **who may change `admission` — CLOSED at v2.4 (J-759): Owner-only, via the permission table** · 📌 Leg ②'s two-node divergence test (deferred past `E-0`) · 📌 §10 C-1's sibling-divergence follow-up (**its own milestone, never a rider**) · 📌 ch3 §3.16.1's invite-bar restatement (§5 C-4) · 🔓 **`F-3`'s non-string boundary — LEG D's, not decided here.**
