# M11 — `self` Thread: Phase-0 Audit (D-021)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Purpose & method

The D-071 Phase-0 audit for M11 (`self` thread). Read-only, evidence-cited, grounded to
file:line against the canonical `main` tree (clean, in sync with origin at `345a461`). No design,
no code. Agenda = `tasks/M11_SELF_THREAD_PHASE0_BRIEF.md` (the Joe-LOCKED concept + scope). Feeds
the design phase. Honest-over-green (D-065): if grounding contradicted the locked B shape it would
be surfaced and the C fallback flagged — it does not; B is admissible.

The locked concept (do not re-litigate): `self` = a **Node-side, never-federated, never-broadcast
personal thread reusing the user's EXISTING keypair** (single identity — `self` is *you*, not a
second account; D-021). Text-first; M12 attachments inherited. Forks already locked: **F1** target
**B** (self-DM) / fallback **C** (single-member regular Space); **F2** zero new registration; **F3**
text-first.

---

## 1. VERDICT (headline up front)

**B is admissible.** A self-DM — `state.dm_space_create` with `content["invitee"] == creator` —
produces a working, coherent, single-member personal thread **through the existing client path
unchanged**, with **zero new registration**, **zero new protocol surface**, and the DM path's
**hard `DmFederationNotAllowed` non-federation guarantee** intact. The `invitee == creator` edge is
**benign**: no error, no break; the creator is already the sole Owner member and the only DM-room
member the instant the create chain lands, so **no `membership.join` is ever needed**. The only
artifact is a **vestigial `pending_invites[self]` entry** that is never consumed and never errors —
a cosmetic double-state (self is both Owner member *and* pending invitee), addressable as a
design-phase polish call, not a blocker.

**C (single-member regular Space) is NOT needed.** It remains the sanctioned fallback per the brief,
but the admissibility edge that would have forced it does not resist. B is preferred: it carries the
hard DM-non-federation guarantee and the `is_dm` / "Saved Messages" semantics; C is non-federated
only by *default* and re-shapes the concept toward "a separate little account."

**Net protocol/applier delta to ship B: zero.** The remaining work is client-side affordance
(recognise + label + a create/open convenience) + the ch6 descriptive note. See §6, §8.

---

## 2. HEADLINE — the self-DM admissibility edge (decides B vs C)

The one real unknown the brief named: trace `from_dm_space_create` → `apply_join` for
`invitee == creator`. Result: **clean (no error), with one inert double-state**. Full trace below.

### 2.1 The constructor — `from_dm_space_create` (client-side, `state.rs:342`)

For a self-DM (`content["invitee"] == event.sender == creator`):

- `members.insert(creator, Owner)` — `state.rs:382`. Creator is a full **Owner** member.
- `room.members.insert(creator)` — `state.rs:415`. Creator is a member of the auto-created `"dm"`
  Room.
- `pending_invites.insert(invitee /* == creator */, PendingInvite{ Member, invited_by: Some(creator) })`
  — `state.rs:419-422`. **No `invitee == creator` guard** (confirmed; the only error returns are
  `WrongEventType` at `:347` and `MissingField` at `:350/:358/:364`). So the same `IdentityXgid` is
  simultaneously an **Owner member** and a **pending Member invitee**.
- Returns `(state, room_event, invite_event)` — `state.rs:462`. The constructor **completes without
  error**. The auto-`membership.invite` targets the creator.

**Finding H-1:** the constructor admits `invitee == creator` cleanly. The resulting double-state
(`members[self] = Owner` ∧ `pending_invites[self] = Member`) is the only artifact. It is inert (§2.4).

### 2.2 The node-side ingest constructor — `from_dm_space_create_node` (key-less, `state.rs:487`)

The Node ingests the create with the key-less sibling (M7C-D4, J-219). Same shape, same outcome for
`invitee == creator`:

- `members.insert(creator, Owner)` — `state.rs:524`.
- `pending_invites.insert(invitee /* == creator */, …)` — `state.rs:530-533`. **No guard** (only
  `WrongEventType` `:489` / `MissingField` `:493/:500/:506`).
- `rooms` start empty (`state.rs:562`); the auto-`state.room_create` arrives as a separate event.

So on the Node, after the root ingests: creator is Owner member + (vestigial) pending invitee.
**Consistent with the client constructor.** ✓

### 2.3 The auto-room applier — `apply_room_create` (`state.rs:777`)

When the auto-`state.room_create` arrives on the Node, `apply_room_create` inserts the **event
sender** (= creator) into the new Room's members: `members.insert(actor)` — `state.rs:791-792`. So
the creator becomes a member of the `"dm"` Room **with no join** — both client-side (constructor
`:415`) and node-side (this applier). ✓

### 2.4 `apply_join` — what happens if a self-join is attempted (`state.rs:982`)

A self-join is **never sent** by the existing path (§4 — `create_dm_space` sends only
create + room + invite, no join). But for completeness, were one attempted:

- **Space-level** (`room_id` empty): `if self.members.contains_key(joiner)` → creator IS a member →
  `Err(AlreadyMember)` — `state.rs:1000-1001`. Rejected, harmless (no state mutation).
- **Room-level** (`room_id` set): `if room.members.contains(joiner)` → creator IS already in the dm
  Room → `Err(AlreadyMember)` — `state.rs:992-993`. Same.

So the vestigial `pending_invites[self]` is **never consumed** (apply_join would short-circuit on
`AlreadyMember` before reaching the `pending_invites.remove(joiner)` at `state.rs:1006`). It sits
inert. **No code path chokes on the member-and-pending double-state** (the only consumer of
`pending_invites` is `apply_join` for a *non*-member, plus `repopulate_dm_federation_nodes` which
unions members ∪ pending → `{self}` either way; §5).

**Finding H-2:** the `invitee == creator` edge is benign across the whole applier surface. No error
breaks the create; no downstream applier malfunctions; the self-DM is fully formed (Owner + dm-Room
member) after the create chain, with no join required.

### 2.5 Message admission — can the creator actually post & read?

The usability test. A self-DM is only real if the creator can post a `message.text` to the dm Room
and read it back. Live `validate_event` (`exchange.rs:489`) step 11:

- **Sender registered** — `id_registry.contains(sender)` — `exchange.rs:629`. Creator is registered
  (it's the user's existing identity) → pass (no HeldPending). *(The brief cited `exchange.rs:202-209`
  — that is the **deprecated** `validate_steps_8_13` `UnknownSender` sibling; the live gate is the
  `validate_event` HeldPending-on-unknown-signer at `:629`, cited here per D-078.)*
- **Space member** — `space.is_member(sender)` — `exchange.rs:672`. Creator is Owner → pass.
- **Room member** — `space.is_room_member(sender, room_id)` — `exchange.rs:676`. Creator is in the dm
  Room (§2.3) → pass.
- **Signature** — `verify_event_signature` — `exchange.rs:685`. Creator's own key → pass.

So the creator passes every step-11 gate and **can post** to the self-DM's dm Room. Reading is via
sync (§5). **The self-DM is a usable thread.** ✓

---

## 3. Registration cost (supporting item 1) — CONFIRMED zero

B reuses the user's **already-registered** identity:

- The create is signed by the existing keypair; `validate_event` step 11 finds the signer in
  `IdentityRegistry` (`exchange.rs:629`) → admitted, **no registration step**.
- `create_dm_space` pulls the signing key + identity from the loaded session
  (`ops.rs:670-677`) — the same identity used for every other verb. No new key, no local-registration
  path, no synthetic identity.

**F2 holds (zero new registration).** The "local registration mandatory" clause from D-021 was an
artifact of the abandoned own-keypair option (A) and drops out entirely under B.

---

## 4. DM-creation entry point (supporting item 2) — works UNCHANGED

`CreateDmSpaceArgs` = `{ invitee: String }` only (`app.rs:540-544`) — **no self-guard, no
invitee≠self validation**. A client can pass its own identity_id today: `create-dm-space --invitee
<own-id>`.

`ops::create_dm_space` (`ops.rs:656`) sends a **3-event chain over one connection, root-first**
(`ops.rs:736-804`):

1. **Root** `state.dm_space_create` (`ops.rs:697-700`) — built from `args.invitee` + the home Node's
   echoed pubkey `node_id` (M10.4 Shape B, `ops.rs:685-694`). Must be **Accepted** (`ops.rs:756`).
   For a self-DM: signer registered, DAG root → admitted. ✓
2. **Auto-room** from `from_dm_space_create` (`ops.rs:711-713`). Must be **Accepted**. Creator
   auto-joins the Room via `apply_room_create` (§2.3). ✓
3. **Auto-invite**, rebuilt tip-chained to the room (`ops.rs:721-732`), targets `args.invitee`.
   **Accept-either** — `Ok(Accepted) | Ok(Rejected) => {}` (`ops.rs:786-787`); only `TimedOut` /
   transport error aborts. The node's `apply_invite` returns `DmInvitationNotAllowed`
   (`state.rs:947-948`, fires on `dm_constraints_active` *before* it ever inspects the target), which
   the dispatch path swallows ("empirically it Accepts", `ops.rs:740-742`).

**Finding E-1 (load-bearing):** the auto-invite's swallowed-reject is the behaviour of **every** DM,
self or not — `apply_invite` rejects on the DM constraint regardless of who the target is. A self-DM
exercises the **identical** path; `invitee == creator` introduces **no new failure**. (Normal DMs are
GREEN — MP-C-07 — so this path is proven.) The client-state row ("DM with `<self-id>`",
`ops.rs:808-818`) is written only after the chain confirms.

**No self-join is sent** (the chain is create + room + invite only), so the `AlreadyMember` path of
§2.4 is never even reached.

---

## 5. Reach (supporting item 3) — CONFIRMED a free property

The Node-side claim, grounded not assumed. `collect_sync_history` (`fanout.rs:447`):

```
for (space_id, space) in &rt.spaces {
    if !space.is_member(requester_id) { continue; }   // fanout.rs:457
    ... store.range(0) ... topological_sort_events ...  // fanout.rs:460-463
}
```

Any client that authenticates as the user and sends a `sync_request` (dispatched at `app.rs:1667`)
receives the **full topo-sorted history of every Space the user is a member of** — including the
self-DM (the user is its Owner). No new mechanism; the self-DM rides the existing **member-gated**
sync exactly like any other Space.

**Precision (honest reading, D-065):** "reachable from any client on the Node" resolves to
"reachable from any client **authenticated as the user**" — i.e. all of the user's own devices. The
identity keypair is itself a device-local file (`ClientIdentity::load`, session.rs:60), so this is
*not* "any arbitrary client regardless of identity." What is Node-resident — and therefore shared
across the user's devices — is the **thread** (`rt.spaces` + the per-Space store), not one device's
local copy. This satisfies D-021's "not device-local" in the sense that matters: the user's phone and
laptop both see the same self-DM by syncing from the Node. The design/ch6 note should state this
precisely and not overclaim "any client."

### 5.1 Non-federation guarantee (the privacy property)

Two independent guards, both confirmed:

- **Hard guard:** `apply_federation_add` returns `Err(DmFederationNotAllowed)` whenever
  `dm_constraints_active` (`state.rs:660-661`); the self-DM sets `dm_constraints_active: true`
  (`state.rs:454` / `:565`). No `state.federation_add` can ever populate a self-DM's
  `federation_nodes`. "No third-party node ever receives DM content" (runtime.rs:2105).
- **Degenerate set:** `repopulate_dm_federation_nodes` (`runtime.rs:2101`) derives the federation set
  from **parties' home nodes** (members ∪ pending invitees). For a self-DM all parties are the user,
  whose home node is **this** node → the set is `{this_node}`, which the push path excludes (no
  self-push). Even absent the hard guard there is nothing to push.

The self-DM is **doubly contained** — it never federates and has no remote party to federate to. ✓

---

## 6. Client surface (supporting item 4) — minimal, client-only

- **No existing `self` / "Saved Messages" handling** in `xgen-client` (grep: none).
- **Create:** `create-dm-space --invitee <own-identity-id>` works **today, unchanged** (§4).
- **Post:** the existing `Send` verb (`app.rs:376`) against the dm Room.
- **Read:** the existing `History` verb (`app.rs:379`) against the dm Room.
- **Locate:** the create writes a `KnownSpace` row labelled `"DM with <invitee>"` (`ops.rs:810`); a
  client can recognise `invitee == own-id` and relabel/route it as the self thread.

**Minimal client work for B (all in `xgen-client`, no protocol change):** (a) a convenience to
create-or-open the self thread (recognise/auto-create the self-DM, e.g. a `self` verb or an
auto-provision on first use), and (b) labelling the self-DM as "self" / "Saved Messages" instead of
"DM with `<self>`". Posting/reading already exist. This is affordance + labelling, **not** a new wire
type or applier.

**ch6 deliverable (authored at M11 close, per brief):** a short client-design note — what it is, that
it **reuses the existing identity** (the anchor line), never-federated/never-broadcast by reference to
the DM constraint, attachments as inherited M12 capability, and the boundary (not an account, no new
protocol surface). NOT a ch3 normative edit. ch6 exists at `docs/xgen_ch6_client_design.md`; it has
no `self` section today.

---

## 7. Operator-confidentiality (low priority) — moot for B-vs-C

Whether the user's **own** home-Node operator can read self-DM content (E2E) is independent of the
DM-non-federation guarantee (which only proves *other* nodes never see it). It is **moot for the
B-vs-C decision**: under both B and C the thread lives on the user's own home Node, so the own-node
operator has the same read posture in either shape — it does not discriminate B from C. `is_dm`
Spaces carry `e2e_encryption` uniformly (default OFF; `state.rs:372` / `:514`), so E2E is *available*
to a self-DM exactly as to any Space, but no current caller declares it. Out of M11 scope per the
brief; surfaced here only to confirm it does not bear on the call.

---

## 8. Honest caveats & design-phase calls (D-065)

None of these change the B verdict; all are design-phase polish, not Phase-0 blockers:

1. **Vestigial `pending_invites[self]` double-state (§2.1, §2.4).** Self is both Owner member and
   pending invitee. Inert today (never consumed, never errors). The design phase should pick: (a)
   leave it (zero code, inert); (b) add an `invitee == creator` guard in the constructor(s) that skips
   seeding the self-invite — clean, but touches `from_dm_space_create` + `from_dm_space_create_node`;
   (c) skip emitting the auto-invite event for a self-DM in `create_dm_space`. **Recommend (a) or a
   minimal (b)** — (a) is honest about "B works unchanged"; (b) is the tidy version if the design
   wants the state to read cleanly. Flag, do not pre-decide.
2. **Reach wording (§5).** Do not let ch6/design overclaim "any client." It is "any client
   authenticated as the user." State precisely.
3. **`create-dm-space --invitee <self>` is currently the literal entry.** B ships with **no protocol
   change**, but a bare `create-dm-space --invitee <own-40-char-id>` is a poor UX. The minimal client
   convenience (§6) is the design's call — it is affordance, not mechanism.
4. **The constructor's empty-`prev_events` auto-invite latent bug** (`ops.rs:649-655`, D-065) is
   pre-existing and orthogonal — `create_dm_space` already discards the constructor's invite and
   rebuilds it tip-chained. Not an M11 concern; noted so the design doesn't trip over it.

---

## 9. What the design phase must decide

1. **Confirm B** (this audit's verdict) — or invoke C if Joe disagrees with the double-state
   tolerance (not recommended; B is clean).
2. **The double-state polish** (§8.1): leave / guard-in-constructor / skip-auto-invite.
3. **Client entry shape** (§6): a `self` convenience verb / auto-provision vs literal
   `create-dm-space --invitee <self>`; the "self" / "Saved Messages" label.
4. **The ch6 note** content (close deliverable, brief-specified).
5. **(Out of scope, confirm deferred):** attachments (M12, F3), E2E for self-DM (§7).

---

## 10. Out-of-scope / routed-open (confirmed, not re-opened)

- ch6 descriptive note — close deliverable, not Phase-0 (§6).
- Attachments — M12 (F3 locked text-first).
- Operator-confidentiality / E2E — moot for B-vs-C (§7); deferred.
- Routed-open items unchanged (brief): MP-F12, MP-F2-followon, MP-F15, MP-F16. None bears on M11.

---

## 11. Evidence index (file:line)

| Claim | Site |
|---|---|
| client DM constructor, no `invitee==creator` guard | `xgen-core/src/space/state.rs:342`, members `:382`, room `:415`, pending `:419-422`, returns `:462` |
| node-side key-less DM constructor, no guard | `xgen-core/src/space/state.rs:487`, members `:524`, pending `:530-533` |
| auto-room applier inserts sender into Room | `xgen-core/src/space/state.rs:777`, `:791-792` |
| apply_invite rejects on DM constraint (target-blind) | `xgen-core/src/space/state.rs:946-948` |
| apply_join space-level AlreadyMember | `xgen-core/src/space/state.rs:1000-1001` |
| apply_join room-level AlreadyMember | `xgen-core/src/space/state.rs:992-993` |
| live step-11 registration gate (HeldPending) | `xgen-core/src/message/exchange.rs:489`, `:629` |
| step-11 Space/Room member gates | `xgen-core/src/message/exchange.rs:672`, `:676` |
| deprecated step-11 sibling (brief-cited) | `xgen-core/src/message/exchange.rs:202-217` |
| DmFederationNotAllowed hard guard | `xgen-core/src/space/state.rs:660-661`; `dm_constraints_active: true` `:454`/`:565` |
| repopulate from parties (degenerate self set) | `xgen-core/src/node/runtime.rs:2101-2105` |
| CreateDmSpaceArgs = invitee only, no self-guard | `xgen-client/src/app.rs:540-544` |
| create_dm_space 3-event chain + accept-either invite | `xgen-client/src/ops.rs:656`, chain `:736-804`, invite accept-either `:786-787`, swallow note `:740-742` |
| reuses session identity (zero new registration) | `xgen-client/src/ops.rs:670-677` |
| collect_sync_history member-gated reach | `xgen-node/src/fanout.rs:447`, member filter `:457`; dispatch `xgen-node/src/app.rs:1667` |
| Send / History verbs exist | `xgen-client/src/app.rs:376`, `:379` |
| ch6 client-design chapter (no self section) | `docs/xgen_ch6_client_design.md` |

---

*Phase-0 audit. No code until the M11 design is Joe-locked.*
