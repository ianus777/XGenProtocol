# M-RP-ADDRESS-BOOK — client-side seen-records, the identity cache the UI reads names from
> **Status**: COMPLETED  
> Version: 1.9  
> Date: Jul 2026  
> **Last updated**: 2026-07-25  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

**THIS IS A PHASE-0 (D-071). ZERO code until Joe locks.**

**IT IS:** the client's local projection of node-authoritative `IdentityRecord`s — the *who-exists* substrate that every name, avatar and member row reads from.

**IT IS NOT:**

- **NOT the private Identity record.** Ch2 §Contact Model defines `identity_private.contacts` — encrypted, synced across devices, **user-authored annotations** (alias, note, `meta-atts`). A separate, later milestone.
- **NOT presence.** Ephemeral, Space-scoped, short TTL. `xgen-status-gap-phase0.md` §3 defers it out-of-band.
- **NOT self-set status** `{emoji, text}` — greenfield, Track A, and it gates all status-bearing UI.
- **NOT the members widget.** That is **M-RP-MEMBERS**, a consumer of this.

⚠️ **Conflating this book with Ch2's contact record would be an architecture error, not a naming one.** One is a cache of **public** records; the other is **encrypted private** annotations. Different owners, different storage, different erasure rules.

---

## §1 — Why it exists, and why it is first

**Every other identity-bearing layer is a decoration on this one.**

- A **contact** (③) is a pubkey plus your annotations. The pubkey resolves to nothing without a record.
- A **presence signal** is identity + space + status. Without a record it is a coloured dot beside a 65-character XGID.
- `xgen-dd-entity-avatar-phase0.md` §2.1 states it outright: *data dwells in the address book*, and the avatar is *a projection of one address-book entry*.

🔑 **AND IT CLOSES C-8 WITHOUT CONTACTS AND WITHOUT PRESENCE.** The four-layer name chain's **bottom** layer is the global display name, which lives in `IdentityRecord` — exactly what this caches. ⇒ **M-RP-INBOUND-NAME unblocks HERE**, one milestone earlier than the roadmap said.

---

## §2 — Grounding (measured at `886dd07`, 2026-07-24)

**CODE.** Positive control: 51 files scanned across `xgen-client/src` + `xgen-common/src`, **175** `pub fn` matches — the grep works.

- `address_book` / `AddressBook` / `seen_record` / `known_identit` — **0 matches.**
- `identity_private` / `contacts` / `contact_list` / `struct Contact` — **0 matches** across `xgen-core`, `xgen-common`, `xgen-client`.
- `presence` — 8 matches, **every one the English word** (*predecessor presence*, *the store reports presence/absence*, *blob presence*). **No presence subsystem exists.**
- `xgen-common/src/state.rs:185` — `KnownSpace {space_id, name, node_endpoint, role, rooms}`; `:195` — `KnownRoom {room_id, name, joined}`. **No identity payload.**
- `xgen-client/src/desktop.rs:601` — `get_spaces` returns `Vec<KnownSpace>` **verbatim (D1)**, read once on mount.
- `xgen-core/src/space/state.rs:74` `SpaceMember` (role + admitting-invite signer) · `:118` `RoomState.members: HashSet<IdentityXgid>` · `:221` `SpaceState.members: HashMap<IdentityXgid, SpaceMember>`. ⇒ **the node holds a full roster; nothing carries it to the client.**
- `xgen-node/src/admin_ops.rs:60` imports `IdentityRecord` / `RegistryError`; A5's mutating verbs cover `identity` ⇒ **identities are creatable from the shipped admin write path.**

**SPEC.** Both lenses agree, so this is **not** the N-164 trap:

- `xgen-status-gap-phase0.md` §2 — *node holds authoritative `IdentityRecord`; client holds its projection (`xgen-client_state.json` + address book = client-side seen-records)*. 🔑 **The book is already named and defined in the record. It is UNBUILT, not unimagined.**
- same §1 — self-set status and presence are *"missing, greenfield"*.
- same §3 — presence is ephemeral, *"likely out-of-band, deferred"*.

---

## §3 — The four layers the docs specify

| # | layer | home | state |
|---|---|---|---|
| ① | public `IdentityRecord` | node — unencrypted, replicated | ✅ built, node-side |
| ② | **address book — seen-records** | client — local projection | ❌ **THIS MILESTONE** |
| ③ | contacts — private annotations | `identity_private` — encrypted, synced | ❌ later milestone |
| ④ | presence | ephemeral, Space-scoped, TTL | ❌ deferred (§3 of status-gap) |

**Name chain** (Ch2 §User Representation, locked): contact alias ③ → Space nickname (Space membership record) → global display name ①. The **contact note** is supplementary context and is **never** a name.

---

## §4 — DECISION 1: FILL POLICY — what enters the book?

**F1 — author-on-sight.** Any identity that authors a **message** event you render. ⚠️ **DISAMBIGUATED 2026-07-25 (J-586, Joe ruled Reading B).** As first written this said "an event you render", which is ambiguous: membership events ARE rendered (C2 system notices, J-547) and every member authors their own `membership.join`, so the literal reading put every member in F1, made F2 dead code, and falsified this section's own next line. **Membership and state events do not qualify.**
- *User-visible:* names appear for people who have **spoken**; everyone silent stays an XGID. A member list built on F1 alone shows only talkers.
- *Cost:* smallest. One hook on the event-ingest path.

**F2 — Space-membership sweep.** On opening/joining a Space, cache every member's record. Ch2 permits this explicitly: *"Within a Space, members can see the membership list of that Space."*
- *User-visible:* everyone appears, silent or not. **This is what a members widget needs.**
- *Cost:* ⚠️ **CORRECTED 2026-07-25 (J-582/J-586): NO transport change is needed.** `ops::members` (`ops.rs:2552-2558`) already derives Space membership client-side by causal replay of the drained DAG; `KnownSpace` has no members field and needs none. **F1 and F2 read the SAME drain.** The real cost is N `identity.get` round-trips, because `MemberEntry` carries no `display_name`.
- 📌 **The roster question returns here, demoted.** It is no longer *the members widget's data model*; it is **one fill source for the book**.

**F3 — registry pull.** Everyone the node knows.
- ⚠️ **COLLIDES WITH THE SPEC, not merely with cost.** Ch2 §Cross-Space Discoverability: *a user's Space membership list is not globally disclosed … the network does not expose a global membership index.* Recommended **refused on spec grounds**, not weighed on cost.

**Chat's recommendation:** **F1 ∪ F2**, F3 refused.

🔒 **LOCKED — F1 ∪ F2, F3 refused (Joe, 2026-07-24).**

---

## §5 — DECISION 2: FRESHNESS — a cache of records that mutate

`IdentityRecord` changes: display-name edits, key rotation, Trust Assertion changes, **revocation**. `update_version` (3.6.8) is the ordering signal.

**V1 — cache forever.** *User-visible:* stale names, and the sharp edge — **a revoked identity still rendering as valid**. ⚠️ In a protocol whose core is verified identity, that is a **trust defect, not a cosmetic one**.
**V2 — refresh on encounter,** take the record with the higher `update_version`. *User-visible:* self-healing for active people; a quiet person's record stays as first seen.
**V3 — V2 plus revocation is pushed,** not left waiting for an encounter.

🔒 **LEG A — MEASURED at `7ab743e`, 2026-07-24. ANSWER: NO PUSH PATH EXISTS.**

*Positive control: 254 source `.rs` files, 792 `pub fn` (`target/` and `.claude/worktrees/` excluded).*

- **A live node→client push channel DOES exist.** The client holds a home-Node WebSocket carrying `Inbound::Event` fan-out (`xgen-client/src/events_pipe.rs` header, EV-D3; `xgen-node/src/fanout.rs`). It is real and already carrying traffic.
- **It cannot carry identity.** The entire DAG event space is `message.*` (text · file · reaction · redact), `state.*` (space · room · federation · mls · dm · status · node_priority · ai_operator) and `space.join_request`. **There is no `identity.*` event type.** Identity lives in the *control-message* layer, never in the DAG.
  - ⚠️ **THE ENUMERATION ABOVE IS INCOMPLETE — CORRECTED 2026-07-25 (M-RP-MEMBERS §5).** It **omits the entire `Membership*` family**, which is eight first-class DAG event types: `MembershipInvite · MembershipJoin · MembershipLeave · MembershipKick · MembershipBan · MembershipNodeEject · MembershipNodeUnban · MembershipMute` (`xgen-common/src/wire.rs:43-58`). 🔒 **THE CONCLUSION OF THIS BULLET STANDS** — there is genuinely no `identity.*` event type, so V3 remains a protocol addition and §5's V2 lock is untouched. **It is the list that was wrong, not the finding.**
  - 🔑 **WHY IT MATTERED, RECORDED SO THE COST IS VISIBLE:** M-RP-MEMBERS read this enumeration as the complete event space and concluded *"the roster is a network read"*, designing a **pull-on-scope-change** members widget. Joe challenged the premise; measurement then showed **membership changes are already fanned out to every connected member** (`xgen-node/src/fanout.rs:272,308`) and reach the frontend **unfiltered** (`resident.rs:407` → `app_client.svelte:517`). **A whole milestone was being designed against a missing line in a list.**
  - 📌 **DEFECT CLASS, third occurrence this arc:** *a claim narrower than the thing it described, reused as if complete* — the J-587 family. The bullet answered *"can the DAG carry identity?"* and was **read as** *"what is in the DAG?"*
- **Revocation is modelled, and it does propagate — node↔node only.** `IdentityRecord` carries `revoked: bool` + `revoked_at` (`xgen-core/src/identity/registry.rs`, 34 hits). `IdentityReplicateMessage::Replicate` pushes the full record plus `update_version` from home Node to replica Nodes (spec 3.13.4, `xgen-core/src/wire/types.rs:715`). **Replica Nodes are pushed to. Clients are not.**
- **Client-side identity access is pull-only:** `identity.get` → `identity.record` / `identity.not_found` (spec 3.6.7). Request/response, one identity at a time.
- **The client crate has zero revocation awareness** — one grep hit across all of `xgen-client`, at `app.rs:946`, and it is an unrelated AI-delegation CLI argument.

📌 **Precedent, not an anomaly:** `state.status` (PROTO-STATUS, `xgen-core/src/status/mod.rs`) is already an identity-scoped, **global**, `update_version`ed object deliberately kept *off* `IdentityRecord` and *off* per-Space DAG resolution. An identity-scoped global object that nothing pushes is an existing shape in this protocol.

⇒ **V3 IS NOT AVAILABLE AT ANY PRICE THIS ARC.** It is not a client feature and not a cache setting; it requires a new node→client identity-notification surface — a **protocol addition**, Joe's, and outside this milestone. §5 therefore chooses between **V1 and V2 only**, and revocation propagation to clients is recorded as a named protocol gap.

**CHAT RECOMMENDS V2.** *User-visible:* names self-heal for anyone still active, and a revoked identity is corrected on next encounter — which is the only moment it can be rendered at all, so the trust defect narrows to *records you never meet again and therefore never display*. *Cost:* the record already arrives on encounter; V2 is one `update_version` comparison on top of V1. **The defect V1 carries is bought off for one integer compare.**

🔒 **LOCKED — V2 refresh-on-encounter (Joe, 2026-07-24).** V3 is a protocol addition (Leg A) and is recorded as a named node→client identity-notification gap, outside this arc.

---

## §6 — DECISION 3: ERASURE — where right-to-be-forgotten lands first

A local cache of **other people's** identity data. This is one of the project's three standing tensions arriving **locally**, ahead of federation.

📌 **Grounded against D-088 (2026-06-04), read this session (N-164).** XGen identity erasure = **orphaning the pubkey↔person binding**: PII (display name, Trust-Assertion attestation) removed, pubkey persists as an anonymous token, every signature keeps verifying, **no Event touched** — `Event.sender` is inside the signed payload (`xgen-common/src/wire.rs:482`), unerasable by construction. The spec's mechanism for cached identity records is **signed deletion notice + federation TTL expiry**, "not guaranteed to propagate instantly" (Appendix D §3.3). ⇒ a client-side cache is exactly the shape TTL expiry exists for.

**E1 — permanent, wholesale clear.** *Cost:* nil. *User-visible:* nothing granular.
**E2 — per-entry delete.** *User-visible:* remove one person by hand.
**E3 — eviction:** records unseen for *N* drop out — the client-side counterpart of the spec's own TTL expiry.

🔒 **LOCKED — E1 + E2 + E3 (Joe, 2026-07-24).** All three. E3 is not a nice-to-have: it is the client's arm of the erasure mechanism the protocol already relies on.
🔒 **N RESOLVED (Joe, 2026-07-24).**

- **N is per-tier, Auth-Module-declared.** The address book reads an eviction policy off the Auth Module of the Space the identity was seen in — same pattern as D-088's tier-scaled interior. The client is a *consumer* of the policy, never its author. ⚠️ **Interface dependency (structural, Joe's):** the book calls into the Auth Module for retention policy — a client↔module seam, deferred to M10 (Auth Module Reference Set); no reference module declares a retention N today, so the read is later work.
- **Protocol pins the floor: the no-module / T1 fallback is FINITE, never ∞.** Modules raise N upward from there; nothing is required to *lower* it. This keeps T1 — the tier with no retention duty and the real minimisation pressure — from silently caching forever.
- **T1 default N = 182 days (6 months), customisable.** The single value live today; the one Leg C needs to age a seed record past and prove eviction fires.
- **Retention rises monotonically with tier; T4 ≈ "keep" (Ch2: 10–20+ yr) — a large finite value, not a stored ∞.** ∞ is abstract; the code never needs an infinity case. Rationale: in T3/T4 (corporate/legal, government/health) a cached identity record can be **evidence** — safeguarding, security, legal. Evicting it on a hygiene timer destroys accountability material exactly where it is most needed. **Safety/evidence-preservation outranks data-minimisation as the tier rises** (GDPR Art. 17(3) retention-for-legal-claims is the same gradient).
- ⚠️ **`TIER*_TTL_DAYS` (tiers.rs:22–24) ARE NOT ELIGIBLE AS N.** Those are **Trust-Assertion renewal TTLs** (how often a *credential* is re-proven) and run the **opposite** way — T2 365 → T4 90, *shorter* at higher tier. Retention N runs *longer* at higher tier. Same-shaped numbers, opposite meaning: **never reuse the TTL constants for eviction.**
- 📌 **ALL Ns ARE PROVISIONAL DEVELOPMENT VALUES (Joe, 2026-07-24).** Today's figures — the 182-day T1 default and any per-tier defaults the reference modules eventually carry — are **temporary, for development**, to be re-tuned once real Auth Modules exist. Recorded so a future reader does not mistake a placeholder for a considered constant.

📌 **"NOT RENEWED" — a locally-derived annotation, no federation, no push, no protocol change.** `TrustAssertion` carries an explicit **`valid_until`** RFC-3339 date (`xgen-common/src/trust_assertion.rs:265`), and `IdentityRecord.trust_assertion` holds the whole assertion (`registry.rs:45`), which already replicates inside the record §5 caches. ⇒ the client derives *not renewed* = `now > valid_until` by a **date comparison against the clock** — reading the issuer's signed expiry, not re-judging validity itself (Joe's point: the client is not the verifier; the source of truth is the issuer's stamped `valid_until`, which rides *in* the cached record). A node-pushed flag would go stale the instant it was sent and would reintroduce the push-freshness problem Leg A showed the protocol has no path for; a cached date never goes stale. **An expired assertion does NOT evict the record** — the identity stays in the book, flagged. Filed as an annotation-on-record (sibling to `revoked`), **display deferred to M-RP-MEMBERS and Joe's**. ⚠️ **N-164 pending:** confirmed the *type* carries `valid_until` and the *field* holds the assertion; NOT yet confirmed the F1/F2 fill path populates `trust_assertion` (it is `Option` — may arrive `None`). A build-time check for Leg C/D, not a Phase-0 blocker.

⚠️ **THE COLLISION STILL STANDS, handed over unresolved (D-121):** when an identity is erased **upstream**, does the local book ever *learn*? The only signal the protocol offers is `identity.not_found` on a re-fetch (which V2 performs on encounter) — there is no push. ⇒ this remains **the first place in the project where right-to-be-forgotten lands concretely**; E1+E2+E3 bound the local exposure, they do not close the propagation gap. Not traded away.

---

## §7 — Storage

`xgen-status-gap-phase0.md` §2 names `xgen-client_state.json` as the client's projection home and the address book beside it.

🔒 **LOCKED — its own file (Joe, 2026-07-24).** Buys clean E1 (delete the file = wholesale erasure, verifiable), keeps other people's identity PII out of the client's own state, and interacts correctly with §6's E3.

🔒 **NAMES LOCKED (Joe, 2026-07-24).** Taxonomy is Joe's under D-123; these are now fixed and Clair implements them verbatim:

- **Module:** `xgen-client/src/address_book.rs`
- **Types:** `AddressBook` (the book) · `SeenRecord` (one entry)
- **Storage file:** `xgen-client_address_book.json`, beside `xgen-client_state.json`, matching the existing `xgen-client_*` convention

📌 **`SeenRecord` is not `IdentityRecord`.** The node type is the authority; `SeenRecord` is the client's *projection* of it plus book-local fields (`last_seen`, and whatever §6 eviction needs). Keeping the names distinct is what stops the projection being mistaken for the source of truth — the ①/② layer confusion §3 warns about, in type form.

---

## §8 — The seed corpus is an OUTPUT of this Phase-0

**Populate cannot be specified before the fill policy is locked.** Once it is, the corpus follows directly from §4–§6: how many identities, how many revoked, at least one `is_ai`, several with changed display names, at least one key rotation — because those are the cases the policy must survive. 

⇒ **Seeding before this Phase-0 closes means seeding a plausible-looking set that exercises nothing, then seeding again.** This is D-071 applied, not a new judgement.

---

## §9 — Legs

- **Leg A** — close §5's open measurement (does a revocation push path exist?). **CHAT.** No code.
- **Leg B** — Joe locks §4, §5, §6, §7.
- **Leg C** — ✅ **DONE (J-581).** Corpus specified: docs/tests/scripts/ADDRESS_BOOK_SEED_CORPUS.md + 6 .xgb. ⚠️ **Two tiers surfaced:** five identities NOW-executable against the node (alice F1 · bob F2 · erin AI · dave revoke · frank not-renewed); **carol (§5 V2)** and **grace (§6 E3)** are **book-internal → Leg D** (V2 has no live producer — nothing emits identity.update at 47ed16b; E3 needs a last_seen field the book does not yet have). **Option C locked for carol:** seed two versions into the book file at Leg D.
- **Leg D** — implementation from a locked runbook. **CLAIR.**

---

## §10 — DoD

**[CHAT]**
- [x] §5's revocation-push question measured against code, result recorded
- [x] four decisions locked by Joe and written into this doc (§4 F1 u F2 . §5 V2 . §6 E1+E2+E3 . §7 own file)
- [x] §6 N resolved: per-tier Auth-Module-declared, finite floor, T1 = 182 d provisional dev default; not-renewed = derived from cached valid_until
- [x] seed corpus specified and the seed set written (Leg C — `docs/tests/scripts/ADDRESS_BOOK_SEED_CORPUS.md` + 6 `.xgb` scripts; carol v2 + grace E3 are Leg-D book-file cases)
- [x] **POPULATE executed and verified (J-582)** — five NOW-tier identities seeded against a live node, cold-run reproducible; corpus corrected to v1.1 (room-join defect, `init --ai`); N-164 answered (`trust_assertion` is normally `None`)
- [x] runbook authored for Clair (`tasks/RUNBOOK_ADDRESS_BOOK.md`, J-583)

**IMPLEMENTER — ALL MET, LEG D CLOSED 2026-07-25 (J-586)**
- [x] book populates per the locked fill policy — F1 ∪ F2 from one drain, off the critical path; **F2 proven live** (bob cached having authored no message)
- [x] freshness rule enforced with a test that a stale record loses to a higher `update_version` (carol seeds, marked per Option C)
- [x] erasure path exists per §6 — E1 + E2 + E3 + `trust_lapsed`; `None` ⇒ no badge
- [x] `cargo` floor **moved 1553 → 1585 / 0 / 62 across 56** (Rust landed, verified independently by Chat); `svelte-check` holds **by scope** — zero frontend touched, not re-measured

---

## §11 — Filed, NOT fixed

- ⚠️ **Ch2 specifies no contact-acquisition flow.** It defines what a contact *is* and where it *lives* — there is no add flow, discovery, or request/accept anywhere in `docs/`. Grepped; zero. **③ cannot be built until that is written.**
- ⚠️ **Joe's mutual "visit card" model diverges from Ch2**, which is emphatic: *"A contact is not a mutual connection … The other person is never notified and never sees your annotations."* An amendment, not a recollection. **Belongs to ③, not here.**
- ⚠️ **Merged-presence display** — collapsing presence from several Spaces into one indicator erases the context boundary Ch2's Space-scoping exists to protect. Not a protocol breach; a user-model question. **Belongs to ④.**
- 📌 The A/B/C roster walk of 2026-07-24 is **superseded** — the roster is not the members widget's data model. It survives only as **F2**, one fill source.
- ⚠️ **THE WIRE CEILING — `identity.record` cannot carry three of the four locked rules (measured `1fd594c`, J-583).** Code (`xgen-core/src/wire/types.rs:455-473`) and spec (Appendix I §IV.1) **agree**: the only client-facing identity payload carries `identity_id`, `display_name`, `registered_at`, `devices`, `home_node`, `is_ai`, `ai_capabilities` — and **no `update_version`, no `revoked`/`revoked_at`, no `trust_assertion`**. The node returns `Some(record)` for a **revoked** identity exactly as for a valid one (`xgen-node/src/app.rs:3538-3551`); revocation is enforced only at session-open against the revoked identity's **own** login (`app.rs:1539`). ⇒ **§5 V2, §5 revocation-on-encounter and §6's not-renewed badge have no wire source.** 🔒 **Joe locked Option C (2026-07-24): build all six, drive those three from book-internal seeds**, so the logic is written and tested and the day the record widens it is field-mapping, not redesign. **Widening `identity.record` is a protocol change** (Appendix I + Ch3 + node + client + a federation-replication check) — **filed as `M13 Client Identity Lookup Widening`** (`tasks/M13_CLIENT_IDENTITY_LOOKUP_WIDENING.md`, Status PENDING, J-584). ⚠️ **D-127 decided there: a revoked Identity returns its record WITH `revoked` set, never `not_found` — `not_found` is reserved for ERASURE.** M13 converts every Option-C seed into a live wire path; neither milestone blocks the other.

---

## §12 — Handoff

**Leg A measured · §4–§7 LOCKED (J-579) · N resolved (J-580) · Leg C DONE (J-581) · POPULATE DONE (J-582) · ✅ LEG-D RUNBOOK AUTHORED (J-583).**

🔒 **NEXT: LEG D — CLAIR IMPLEMENTS `tasks/RUNBOOK_ADDRESS_BOOK.md` v1.0.** Six steps in dependency order: `ops::identity_get()` → book type + own-file storage → F1 ∪ F2 fill → merge on encounter → erasure E1/E2/E3 → corpus load and assert. Names locked (§7). Corpus at `docs/tests/scripts/ADDRESS_BOOK_SEED_CORPUS.md` v1.1, proven runnable from cold.

⚠️ **READ §11's WIRE CEILING BEFORE BUILDING.** Three locked rules have no wire source; Joe locked **Option C** — build them, drive them from book-internal seeds, mark every seeded test with the reason. **Never populate a wire-absent field with a guess, and never let absence read as "fine".**

🔒 **TWO CORRECTIONS THIS ARC, BOTH FROM MEASUREMENT:**

- **F2 is CHEAPER than its lock assumed.** §4's cost note ("a transport change on `KnownSpace` / `get_spaces`") was wrong. `ops::members` (`ops.rs:2552-2558`) already derives Space membership client-side by causal replay of the drained DAG. **F1 and F2 read the SAME drain** — one call, union the two sets, fetch the remainder. F2's real cost is N `identity.get` round-trips, because `MemberEntry` carries no `display_name`.
- ⚠️ **`trust_assertion` is normally `None`** (J-582). The fill path populates it for nobody; frank only has one because `set_trust_expiry` **synthesises** it (`registry.rs:205`). **The not-renewed badge must render nothing on `None`, not "expired"** — otherwise every ordinary identity wears a warning it never earned.

🔒 **FILL TIMING LOCKED (Joe, 2026-07-24): off the critical path.** The Space opens at once; records resolve behind. Blocking the open puts an unbounded network wait in front of a UI action.

📌 **Free for Leg D:** `clock advance` / `clock set` ship behind `--features harness-control` (`admin_ops.rs:4437`, `MockClock`); every seeded record carries `update_version: 0`, a known floor for carol v2.

🔑 **THE BOOK STORES OBSERVATIONS, NOT CURRENT TRUTH (Joe, 2026-07-25).** Each record means *"as of `last_seen`, this was the state."* A cached `revoked = true` can never become wrong; a cached `revoked = false` is **also** only true as of then ⇒ **staleness and absence must BOTH render as UNKNOWN, never as fine.** Generalises the J-582 badge rule to the whole book. 📌 **`registered_at` trimmed from `SeenRecord`** — provenance about them, needed neither to recognise nor to route, required by no locked rule. ⚠️ **The observational framing improves the GDPR footing but does NOT dissolve it** — a record keyed to an identifiable person is still personal data, and the client's user becomes the controller. Reduced, not removed.

📌 **Reputation is deliberately NOT built here.** Stable identity + encounter history is exactly the substrate a reputation system needs, and it would arrive by accident. ⚠️ **On a no-anonymity protocol, reputation attaches to a real, permanent, legally-real person** — mob dynamics and unappealable scores become durable harm to a named human. Ch2-weight, Joe's, and **it must not shape the book's field set.**

**M-RP-MEMBERS** unblocks after the book build. The **`identity.update` emitter** remains filed and unscheduled.
