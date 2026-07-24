# M-RP-ADDRESS-BOOK — client-side seen-records, the identity cache the UI reads names from
> **Status**: ACTIVE  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-24  
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

**F1 — author-on-sight.** Any identity that authors an event you render.
- *User-visible:* names appear for people who have **spoken**; everyone silent stays an XGID. A member list built on F1 alone shows only talkers.
- *Cost:* smallest. One hook on the event-ingest path.

**F2 — Space-membership sweep.** On opening/joining a Space, cache every member's record. Ch2 permits this explicitly: *"Within a Space, members can see the membership list of that Space."*
- *User-visible:* everyone appears, silent or not. **This is what a members widget needs.**
- *Cost:* requires carrying the roster the node already holds (`SpaceState.members`) to the client — a transport change on `KnownSpace` / `get_spaces`.
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
⚠️ **ONE PARAMETER OUTSTANDING, JOE'S:** *N*, the eviction window. Not blocking Phase-0 close; needed before the seed corpus (§8, Leg C) can exercise eviction and before Leg D build. Recorded as the single open value on this lock.

⚠️ **THE COLLISION STILL STANDS, handed over unresolved (D-121):** when an identity is erased **upstream**, does the local book ever *learn*? The only signal the protocol offers is `identity.not_found` on a re-fetch (which V2 performs on encounter) — there is no push. ⇒ this remains **the first place in the project where right-to-be-forgotten lands concretely**; E1+E2+E3 bound the local exposure, they do not close the propagation gap. Not traded away.

---

## §7 — Storage

`xgen-status-gap-phase0.md` §2 names `xgen-client_state.json` as the client's projection home and the address book beside it.

🔒 **LOCKED — its own file (Joe, 2026-07-24).** Buys clean E1 (delete the file = wholesale erasure, verifiable), keeps other people's identity PII out of the client's own state, and interacts correctly with §6's E3.

---

## §8 — The seed corpus is an OUTPUT of this Phase-0

**Populate cannot be specified before the fill policy is locked.** Once it is, the corpus follows directly from §4–§6: how many identities, how many revoked, at least one `is_ai`, several with changed display names, at least one key rotation — because those are the cases the policy must survive. 

⇒ **Seeding before this Phase-0 closes means seeding a plausible-looking set that exercises nothing, then seeding again.** This is D-071 applied, not a new judgement.

---

## §9 — Legs

- **Leg A** — close §5's open measurement (does a revocation push path exist?). **CHAT.** No code.
- **Leg B** — Joe locks §4, §5, §6, §7.
- **Leg C** — seed corpus specified from the locked policy. **CHAT** writes the `--batch` script set.
- **Leg D** — implementation from a locked runbook. **CLAIR.**

---

## §10 — DoD

**[CHAT]**
- [x] §5's revocation-push question measured against code, result recorded
- [x] four decisions locked by Joe and written into this doc (§4 F1 u F2 . §5 V2 . §6 E1+E2+E3 . §7 own file)
- [ ] seed corpus specified and the `--batch` set written
- [ ] runbook authored for Clair

**IMPLEMENTER**
- [ ] book populates per the locked fill policy
- [ ] freshness rule enforced with a test that a stale record loses to a higher `update_version`
- [ ] erasure path exists per §6
- [ ] `cargo` floor holds; `svelte-check` floor holds

---

## §11 — Filed, NOT fixed

- ⚠️ **Ch2 specifies no contact-acquisition flow.** It defines what a contact *is* and where it *lives* — there is no add flow, discovery, or request/accept anywhere in `docs/`. Grepped; zero. **③ cannot be built until that is written.**
- ⚠️ **Joe's mutual "visit card" model diverges from Ch2**, which is emphatic: *"A contact is not a mutual connection … The other person is never notified and never sees your annotations."* An amendment, not a recollection. **Belongs to ③, not here.**
- ⚠️ **Merged-presence display** — collapsing presence from several Spaces into one indicator erases the context boundary Ch2's Space-scoping exists to protect. Not a protocol breach; a user-model question. **Belongs to ④.**
- 📌 The A/B/C roster walk of 2026-07-24 is **superseded** — the roster is not the members widget's data model. It survives only as **F2**, one fill source.

---

## §12 — Handoff

**Leg A measured (no push path). §4–§7 LOCKED by Joe 2026-07-24.** Next action is **Leg C — Chat specifies the seed corpus from the locked policy and writes the `--batch` set** (§8), then **Leg D — Clair builds from a runbook**. ⚠️ **N (§6 eviction window) is the one open parameter, Joe's, needed before Leg C exercises eviction.** **M-RP-MEMBERS** now unblocks at the members-widget step of the agreed order — *after* populate + book build, not before.