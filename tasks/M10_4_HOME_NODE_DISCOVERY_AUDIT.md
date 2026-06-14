# M10.4 — Production Identity→Home-Node Discovery (MP-F13) — D-071 Phase-0 Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this audits

The D-071 Phase-0 grounding for **M10.4** (the fourth M10 sub-arc, owning **MP-F13**:
production identity→home-node discovery). It grounds the brief's §3 to file:line against `main`
and answers the one decisive question the brief turns on (§3.3): **is MP-C-16's clean green
reachable by reconciling the Layer-1 `home_node` namespace alone, or is Layer-2 discovery of an
unknown identity load-bearing?**

Audit only — no code. D-078: every symbol below is grounded in production source, not inferred
from call-sites.

### 0.1 Verdict headline (three findings, frame holds)

- **The namespace mismatch is real and localized.** `home_node` is typed `NodeXgid` everywhere and
  its canonical value is a **pubkey node_id** (`xgen://pubkey/ed25519:…`). Two populating paths
  diverge: the **identity-registry** `home_node` is already a correct node_id; the **Space-created**
  `home_node` is a **WS URL**, written by the client, wrapped into a `NodeXgid` newtype that performs
  no validation. The URL is simply the wrong value in the right-typed field. (**M10.4-A1**)
- **MP-C-16 stalls at TWO pubkey-vs-URL comparison sites, not one.** The `migration initiate`
  homed-here precondition (`MIG_6010`) AND the `validate_event` cutover authority gate (`6009`) both
  compare a pubkey against the Space's URL `home_node`. (**M10.4-A2**)
- **Clean green IS reachable on Layer-1 alone — the escape does NOT fire for MP-C-16.** Every node
  identity in the migration flow is node-side knowable (source self-known; destination
  operator-supplied id+url; cutover sender = source pubkey). Layer-2 discovery of an *unknown*
  identity is genuinely **not required** for MP-C-16. Call-1 narrow-first is confirmed; Call-2 clean
  green is reachable. (**M10.4-A3** — decisive)

One design fork is surfaced, not locked (Call-3 honours "audit-grounded, not locked here"): the
*shape* of the Layer-1 reconciliation has two candidate forms with materially different blast radii
(**M10.4-A4**), plus a persistence-migration footnote (**M10.4-A5**). None contradicts a locked call.

---

## 1. The `home_node` namespace (brief §3.1, load-bearing)

### 1.1 The type contract — canonical value is a pubkey node_id

`home_node` is typed `NodeXgid` at every structural home:

- `IdentityRecord.home_node: NodeXgid` — `xgen-core/src/identity/registry.rs:47`.
- `SpaceState.home_node: NodeXgid` — `xgen-core/src/space/state.rs:192`.
- `WhoamiResult.home_node: NodeXgid` / `StatusResult.home_node: NodeXgid` —
  `xgen-client/src/ops.rs:188,217`.

`NodeXgid` is the key-derived **principal** flavour (`xgen://pubkey/ed25519:<key>`), the same family
as `IdentityXgid` (D-072 / D-083). Its intended value is a node **pubkey id**, not a transport URL —
the registry fixture and the production registration path both populate it as one (§1.2). **No
`NodeXgid` constructor validates the string** (`NodeXgid::from_xgid(Xgid::new(s))` wraps verbatim), so
a URL string can sit inside a `NodeXgid` without a type error — which is exactly how the bug hides.

### 1.2 The two populating paths diverge (the single namespace violation)

**Identity-registry `home_node` = the accepting node's own node_id (CORRECT per the type).**
`accept_registration(.., home_node_id: &str, ..)` sets
`home_node: NodeXgid::from_xgid(Xgid::new(home_node_id))` —
`xgen-core/src/identity/registration.rs:543`; the doc-comment at `:438` states "`home_node_id` is
this Node's own node_id URI." The production caller passes the node's pubkey id:
`accept_registration(.., home_node_id.as_str(), ..)` at `xgen-node/src/app.rs:2914`, where
`home_node_id: NodeXgid` (`app.rs:2517`) originates from `node_id_uri = pubkey_uri(&signing_key)`
(`app.rs:632`) — the node's keypair pubkey URI. The registry fixture confirms the shape:
`"home_node": "xgen://pubkey/ed25519:NODE"` (`registry.rs:288,503`).

**Space-created `home_node` = the client's WS URL (WRONG per the type).** `SpaceState::from_space_create`
parses `content["home_node"]` verbatim into `NodeXgid` (`xgen-core/src/space/state.rs:277-282`). The
client writes that content field from its session's transport URL:
`build_space_create_event(.., &home_node, ..)` where
`home_node = ctx.node_override.unwrap_or(ctx.session.home_node.clone())`
(`xgen-client/src/ops.rs:449-456`), and `SessionState.home_node` is **explicitly a `ws://` transport
URL** — `xgen-client/src/session.rs:73,174-176,188-189` (the type-stability test asserts
`s.home_node == "ws://127.0.0.1:8080/xgen"` and the doc-comment says "stays `String` (a `ws://`
transport URL, not a Node XGID)"). `ClientState.home_node` persists the URL too
(`session.rs:199-202`).

**So the namespace divergence is confined to `content["home_node"]` on `state.space_create` /
`state.dm_space_create`.** It is signed content (§4), so it cannot be normalized node-side on ingest.

This matches MP-F13's pinned diagnosis verbatim (`tasks/MP_findings.md:379-390`, J-347): the migrated
Space's `space_create.content.home_node = ws://127.0.0.1:8521/xgen` vs A's
`rt.node_id = xgen://pubkey/ed25519:…`; "the in-process Arc-F migration tests set `home_node = node_id`
explicitly… the real-binary client path always writes a URL." (The in-process test fixtures
`space_homed_at_node_with_bob` / `TEST_HOME = "xgen://pubkey/ed25519:NODE"` —
`state.rs:2611`, `ops.rs:2514` — write a pubkey, which is why every unit test passes while the
real-binary path stalls.)

### 1.3 Every `SpaceState.home_node` consumer wants a pubkey; none dials it

Grounded by enumerating all reads of `self.home_node` / `st.home_node` / `s.home_node` across
`xgen-core` + `xgen-node`:

| Consumer | Site | Compares `home_node` against | Wants |
|---|---|---|---|
| `migration_initiate` homed-here precondition | `xgen-node/src/admin_ops.rs:2096` | `rt.node_id` (pubkey) | **pubkey** |
| `validate_event` node_eject/unban authority (→ 3043) | `xgen-core/src/message/exchange.rs:699` | `event.sender` (pubkey) | **pubkey** |
| `validate_event` space_migrate cutover authority (→ 6009) | `xgen-core/src/message/exchange.rs:717` | `event.sender` (pubkey) | **pubkey** |
| `apply_space_migrate` authority (defensive re-check) | `xgen-core/src/space/state.rs:1158` | `event.sender` (pubkey) | **pubkey** |
| `apply_node_eject` authority | `xgen-core/src/space/state.rs:1093` | `event.sender` (pubkey) | **pubkey** |
| `apply_node_unban` authority | `xgen-core/src/space/state.rs:1115` | `event.sender` (pubkey) | **pubkey** |

**Every consumer is an authority/identity equality check that wants the pubkey node_id. No consumer
dials `SpaceState.home_node` as a URL.** The node `state` snapshot reports `record.home_node`
(`app.rs:2991`) from the *identity* registry (already a node_id) and is display-only. This is the
load-bearing structural fact: the URL value serves **no** consumer correctly — it is wrong for all
six, and right for none.

### 1.4 The federation **dial** path is on the node_id→url namespace already (separate, correct)

Dialing does not go through `SpaceState.home_node`. The federation push/dial path operates on
`peer_urls: HashMap<NodeXgid, String>` (node_id → URL), populated by
`record_peer_url(&peer_node_id, url)` (`xgen-node/src/app.rs:2034`) and consumed by the per-peer push
loop (`app.rs:3234-3253`). `SpaceState.federation_nodes: Vec<NodeXgid>` carries pubkey node_ids
(`repopulate_dm_federation_nodes` reads `IdentityRecord.home_node` — a node_id —
`xgen-core/src/node/runtime.rs:2108-2126`; the DM identity-replicate hook reads the same,
`app.rs:3088`). So the federation/identity layer is uniformly on the **node_id** namespace with
node_id→url resolution via the peer registry. `build_identity_home_nodes`
(`runtime.rs:2068-2074`) maps `identity_id → IdentityRecord.home_node.as_str()` (node_ids) into the
`derive_resolved` resolution layers (3/5a/5b) — also unaffected by the Space-side URL bug.

**Implication:** reconciling `SpaceState.home_node` to the pubkey node_id is *consistent with the rest
of the system*, not a divergence. The URL was the anomaly.

### 1.5 Wire-affecting vs read-side (brief §3.1 closing question)

`content["home_node"]` is **signed content** on `state.space_create` / `state.dm_space_create` (it is
inside the canonical-signed event body — `build_space_create_event` writes it before `sign_event`,
`ops.rs:455-456`). Three consequences:

- A node **cannot** normalize it on ingest (mutating signed content breaks the signature — the
  blocked option, consistent with J-347's "(a) node-normalize-on-ingest = blocked").
- Changing **what value the client writes** is *not a wire-schema change* (the field already exists;
  only its value changes from a URL to a node_id) — but it **is wire/behaviour-affecting** in that the
  client must *learn* the node_id to write it (§4).
- Existing persisted Spaces carry a URL `home_node` (**M10.4-A5**) — a value-flip has a
  data-migration footnote.

---

## 2. The migration path — the MP-C-16 stall (brief §3.2)

### 2.1 The two stall sites (both pubkey-vs-URL)

**Site 1 — `migration initiate` homed-here precondition (`MIG_6010`).**
`xgen-node/src/admin_ops.rs:2081-2104` (`migration_initiate`): the precondition is
`Some(st) if st.home_node.as_str() == rt.node_id.as_str() => {}` else `MIG_6010 "Space … is not homed
on this Node"` (admin_ops.rs:2096-2102). On the genuine source node A, `st.home_node` = A's WS URL and
`rt.node_id` = A's pubkey id → never equal → **MIG_6010 fires on the source that actually homes the
Space** (the `Some(st)` branch, not the `None`/absent branch — MP-F13 confirms this is MIG_6010, not
MIG_6011). The verb args are clean: `MigrationInitiateArgs { space_id, destination_id, destination_url }`
(admin_ops.rs:2055-2064) — the operator supplies **both** the destination node_id and url explicitly.

**Site 2 — the cutover authority gate (`6009`).** Even past Site 1, the cutover
`state.space_migrate` event (signed by the source keypair → `event.sender` = source pubkey) is gated
in `validate_event` by `Some(s) if event.sender.as_str() == s.home_node.as_str() => {}` else
`SpaceMigrateAuthority` (wire **6009**) — `xgen-core/src/message/exchange.rs:715-722`. Source pubkey ≠
source URL → **6009**. The applier `apply_space_migrate` re-checks the same gate defensively
(`state.rs:1158`). So a Layer-1 fix that clears only Site 1 would still stall at Site 2; **both** sites
must reconcile to the same namespace.

Both sites resolve cleanly the instant `SpaceState.home_node` carries the node's pubkey node_id:
Site 1 → `pubkey == rt.node_id (pubkey)` ✓; Site 2 → `source-pubkey == home_node (source pubkey)` ✓,
then the applier flips `home_node` to `destination` (`state.rs:1161`, where `destination` derives from
the migrate event's `destination_node_id` content — a pubkey supplied by the operator). Zero
projection needed at the pure applier *if the stored value is the node_id*.

### 2.2 The witness scenario — what clean green requires

`mp_r2_fixed::mp_c_16_live_migration_space_rehomes`
(`xgen-mptest/tests/mp_r2_fixed.rs:300-371`): alice on node `a` registers → `create-space "MP-C-16"`
→ `create-room` → `send "pre-migration"` (all via the **client**, so `content["home_node"]` = a's WS
URL); a `[[migration]]` director step (`from="a"`, `to="b"`, `space_key="space_id"`) drives
`migration initiate`; the test `require_ok`'s the verb (comment at `:306-307`: "the verb + args are
correct, the Space is homed on A, and B is reachable") and asserts the migrated Space is present on
destination B (`:313-318`). The current RED is `require_ok` failing at the MIG_6010 stall (Site 1).
The doc-comment at `:319-322` already flags that **home_node-flip-on-both** is the box-gated RUN
enrichment "needs a per-Space home query."

For **clean green** (Call-2): `migration initiate` must pass homed-here (Site 1), the cutover must
pass validate (Site 2) + apply, the Space must replicate to B, and `home_node` must flip to B on both
nodes. Destination `b`'s node_id + url are both harness-knowable (b's keypair → node_id; b's listen
url) and supplied by the `[[migration]]` director (`to="b"` → the runner resolves b's identity). So
nothing in the migration flow needs to *discover* an unknown identity.

---

## 3. The Layer-2 escape grounding (brief §3.3) — DECISIVE

**Question:** Is MP-C-16's clean green reachable with Layer-1 reconciliation alone, or is Layer-2
discovery of a not-yet-known identity load-bearing?

**Answer: reachable on Layer-1 alone. Layer-2 discovery is NOT required for MP-C-16.** Every node
identity the migration flow touches is node-side knowable without any directory/gossip/DHT lookup:

- **Source identity** — self-known (the source runs `migration initiate`; `rt.node_id` + its own URL).
- **Destination identity** — operator-supplied as explicit args (`destination_id` + `destination_url`,
  admin_ops.rs:2058-2063); in the harness, the `[[migration]]` director resolves `to="b"` to b's id+url.
- **Cutover authority** — `event.sender` is the source's own pubkey (the source signs the cutover);
  the comparison is source-pubkey vs the Space's home_node — both becoming pubkeys once Layer-1 is
  reconciled, no third-party lookup.

This is categorically different from the F1B-D5 / DM-stranger gap (`tasks/MP_findings.md:110-116`,
J-332/J-333), where a node holds **only a counterparty's pubkey and no `IdentityRecord`** and has no
path to resolve where that identity lives. That gap is about discovering an **unknown** identity's
home; the migration flow never holds an unknown identity — both ends are supplied or self-known.

**Therefore Call-1's narrow-first is confirmed and the escape hatch does NOT fire for MP-C-16.** The
Layer-2 discovery arc (gossip / directory / DHT / XGID-encoded home) stays **separately routed**, not
smuggled into the namespace fix.

**Layer-2 arc named for routing:** *production identity→home-node discovery of unknown identities* —
the **F1B-D5 / DM-stranger sibling** (the `repopulate_dm_federation_nodes` F1B-D3 omission boundary,
`runtime.rs:2099-2103`: "a party whose record is NOT in this node's registry is **omitted** … deferred
behind the routed identity→home discovery arc (F1B-D5)"). It is genuinely heavy (open design space)
and **out of M10.4 scope** unless a future arc proves it load-bearing for a *different* witness.

**Boundary note (brief §4):** the F1b DM-stranger convergence boundary (F1B-D4 harness-green-with-
boundary) is **not** incidentally cleared by reconciling the Space `home_node` — F1b's gap is the
unresolvable *counterparty record*, a Layer-2 problem, distinct from the Space-`home_node` value bug.
It stays as recorded.

---

## 4. The reconciliation-shape fork (surfaced, NOT locked — Call-3)

The brief locks Call-1 (narrow-first) and Call-2 (clean green); it explicitly leaves the `home_node`
*canonical-type* question audit-grounded (Call-3), surfaced back only if it contradicts Call-1. It
does **not** contradict — the grounding *confirms* narrow-first. But the *shape* of the Layer-1 fix
has two candidate forms that the design phase must choose between, because one is wire/client-affecting
and one is node-local. Surfacing both (the design locks):

**The correct canonical value is the pubkey node_id** (§1.3). The blocker on writing it: **the client
cannot learn the node's pubkey id on the current wire.** Grounded — the three messages the client
receives during connect/register carry no node identity:
- `TransportMessage::Challenge { protocol_version, nonce, timestamp }` — no node_id (`wire/types.rs:48-52`).
- `TransportMessage::AuthOk { protocol_version, identity_id, timestamp }` — `identity_id` is the
  **client's own**, echoed (`wire/types.rs:64-68`; `client_authenticate` "Returns the `identity_id`
  echoed by the server" = the registrant's, `connection.rs:400,422`).
- `TransportMessage::RegisterOk { protocol_version, identity_id, registered_at }` — no node_id
  (`wire/types.rs:354-358`).

This confirms J-278's banked claim on current code: **the only node id the client learns is the WS URL
it dials.** So:

- **Shape B — client writes the node_id (the type-correct fix).** Add a node_id echo to a
  connect/register message (e.g. a `node_id` field on `AuthOk` or `RegisterOk`), the client stores it
  alongside its URL, and `create_space` writes the node_id into `content["home_node"]`. Then *every*
  consumer (§1.3) and *both* migration sites (§2.1) work with zero projection. **Cost:** a (small,
  additive) **wire surface** — this is precisely the deferred "re-home notify / `register_ok` node-id
  echo" surface flagged at J-278 CP-5. Plus the persistence footnote (§A5).
- **Shape A — node-side read projection (no wire/client change).** Leave the client writing the URL;
  reconcile at the node consumers using the node's self-knowledge of its own (node_id, url) pair. The
  `migration_initiate` homed-here check is a clean one-liner (`st.home_node == rt.node_id ||
  st.home_node == rt.node_url`). **But the cutover authority gate is the catch:** `validate_event`
  (exchange.rs:717) and `apply_space_migrate` (state.rs:1158) are pure / node-context-light, comparing
  `event.sender (pubkey) == s.home_node (URL)`; projecting the *event sender's* pubkey to a URL needs a
  resolver (the peer registry / migration-session `source_node_id`) threaded into a gate that today has
  none. So Shape A reconciles the precondition cheaply but fights the authority gate.

**Audit lean (recommendation, not a lock):** Shape B is the type-faithful fix and resolves all six
consumers + both migration sites uniformly with no projection plumbing, at the cost of a small additive
wire field the protocol already intended (J-278 CP-5). Shape A avoids the wire change but pushes
node_id↔url projection into the signature/authority gates, which is the more fragile surface. The
design phase should weigh "small wire echo now" vs "projection in the authority gates," and may scope
Shape B's echo narrowly (just enough for create_space to write the node_id) without taking on the full
re-home-notify broadcast. **This is a design-lock, not an audit-lock** (Call-3); it does not reopen
Call-1 or Call-2.

---

## 5. Findings register

- **M10.4-A1 (load-bearing) — namespace mismatch is localized to the Space-created `home_node`.**
  `home_node` is canonically a pubkey `NodeXgid` (§1.1); the identity-registry path writes a correct
  node_id (§1.2, registration.rs:438/543, app.rs:632/2914); the Space path writes a WS URL via the
  client (§1.2, ops.rs:449-456, session.rs:73/174-176). Every `SpaceState.home_node` consumer wants the
  pubkey and none dials it as a URL (§1.3); the federation dial path is a separate, already-correct
  node_id→url namespace (§1.4). **The URL is simply the wrong value in a right-typed field.**
- **M10.4-A2 — MP-C-16 stalls at two pubkey-vs-URL sites, not one.** Site 1 `migration initiate`
  homed-here `MIG_6010` (admin_ops.rs:2096); Site 2 `validate_event` cutover authority `6009`
  (exchange.rs:717) + defensive applier re-check (state.rs:1158). Both must reconcile to the same
  namespace; a one-site fix leaves the cutover stalling (§2.1).
- **M10.4-A3 (decisive) — clean green reachable on Layer-1 alone; escape does NOT fire.** All migration
  node identities are node-side knowable (source self-known; destination operator-supplied; cutover
  sender = source pubkey) — Layer-2 discovery of an unknown identity is NOT required for MP-C-16 (§3).
  Call-1 narrow-first confirmed; Call-2 clean green reachable.
- **M10.4-A4 — the reconciliation-shape fork (design-lock, Call-3).** The client cannot learn the node
  pubkey on the current wire (Challenge/AuthOk/RegisterOk carry none — §4). Shape B (client writes
  node_id via a small additive node_id echo = J-278 CP-5) is type-faithful and projection-free; Shape A
  (node-side read projection) avoids the wire change but fights the authority gates. Surfaced for the
  design phase; recommendation = Shape B, narrowly scoped. Does not contradict Call-1/Call-2.
- **M10.4-A5 — persistence-migration footnote.** Existing persisted Spaces carry a URL `home_node`
  (signed content, immutable). A value-flip (Shape B) makes *new* Spaces correct; pre-existing Spaces
  stay URL-homed unless migrated. **Moot for the MP-C-16 witness** (it creates a fresh Space each run),
  but a real production concern the design should name (not necessarily solve in M10.4).

---

## 6. D-065 / D-078 disposition

Grounding **confirms** the locked calls; nothing to re-lock before design:
- **Call-1 (narrow-first-with-escape):** confirmed — Layer-1 reconciliation alone clears MP-C-16; the
  escape does not fire (M10.4-A3). Layer-2 stays separately routed (F1B-D5).
- **Call-2 (aim clean green):** reachable — both node identities knowable end-to-end (§3).
- **Call-3 (`home_node` canonical type = audit-grounded):** the canonical type is the pubkey node_id;
  this is the type the field already declares and every consumer expects, so it does **not** contradict
  Call-1. The open question that remains is the *fix shape* (M10.4-A4), which is a design-lock, not a
  contradiction to surface back as a re-lock.

All symbols grounded in production source (registry.rs / registration.rs / state.rs / exchange.rs /
runtime.rs / admin_ops.rs / app.rs / connection.rs / wire/types.rs / ops.rs / session.rs +
mp_r2_fixed.rs witness); the in-process-test `home_node = node_id` fixtures are flagged as why the bug
never surfaced before the real-binary witness (§1.2).

---

## 7. What the design phase must lock

1. **The reconciliation shape (M10.4-A4):** Shape B (client writes node_id via a small additive
   node_id echo — recommended) vs Shape A (node-side read projection). If Shape B, the **minimal** wire
   surface (which message carries the node_id echo; where the client stashes it; that `create_space`
   writes it into `content["home_node"]`) — scoped narrowly, not the full re-home-notify broadcast.
2. **Both stall sites reconcile to the same namespace (M10.4-A2):** the design must clear Site 1
   (`MIG_6010`) AND Site 2 (`6009` + applier) — confirm the chosen shape resolves the cutover authority
   gate, not just the homed-here precondition.
3. **The MP-C-16 clean-green witness:** what the re-run at M10.5 asserts (verb require_ok +
   home_node-flip-on-both, the per-Space home query the witness doc-comment flags) — the witness lands
   its disposition at M10.5 per the locked sequence; the design names the proof obligation.
4. **The Layer-2 route (M10.4-A3):** record the F1B-D5 / DM-stranger discovery arc on the horizon as
   the named separately-routed sibling; confirm it is NOT a dependency of MP-C-16.
5. **The persistence footnote (M10.4-A5):** state the production posture for pre-existing URL-homed
   Spaces (migrate / leave-as-legacy / out-of-scope) — at minimum named.

**Next:** Chat design bridge → Joe-lock → runbook → Clair impl → MP-C-16 re-run at M10.5. No code
until the M10.4 design is Joe-locked.
