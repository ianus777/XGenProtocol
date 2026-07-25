# Runbook — M-RP-ADDRESS-BOOK Leg D: build the client-side address book
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-25  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Seat and scope

**CLAIR implements this runbook verbatim.** Every name, field and file path below is locked by Joe (D-123: taxonomy and architecture are his). Where this runbook says "decided by Chat", the choice is technical and reversible — flag it rather than silently changing it.

**Phase-0 is LOCKED and is NOT re-litigated here.** The four locks live in `tasks/M_RP_ADDRESS_BOOK.md` §4–§7. This runbook converts them into ordered steps. If a step appears to contradict the Phase-0, **stop and report** — do not reconcile it yourself.

**Grounded at HEAD `1fd594c`.** Every line number below was measured at that commit. If a seam has moved, re-measure before building; do not trust a stale pointer.

---

## §1 — Locked names (Joe, 2026-07-24)

- **Module:** `xgen-client/src/address_book.rs` (register in `xgen-client/src/lib.rs`)
- **Types:** `AddressBook` (the book) · `SeenRecord` (one entry)
- **Storage file:** `xgen-client_address_book.json`, in the instance data dir beside `xgen-client_state.json`

📌 **`SeenRecord` deliberately does not echo `IdentityRecord`.** The node type is the authority; `SeenRecord` is this client's projection plus book-local fields. Distinct names are what stop the projection being mistaken for the source of truth.

---

## §2 — ⚠️ THE WIRE CEILING — READ BEFORE BUILDING ANYTHING

**Measured at `1fd594c`, code and spec in agreement** (`xgen-core/src/wire/types.rs:455-473`; Appendix I §IV.1, `docs/xgen_appendix_i_en.md:478-489`).

`identity.get` → `identity.record` is the **only** client-facing identity payload, and it carries **exactly**:

`identity_id` · `display_name` (Opt) · `registered_at` · `devices` · `home_node` · `is_ai` (Opt) · `ai_capabilities` (Opt)

It does **NOT** carry `update_version`, `revoked` / `revoked_at`, or `trust_assertion`.

The node confirms this (`xgen-node/src/app.rs:3538-3551`): the lookup returns `Some(record)` for a **revoked** identity exactly as for a valid one, and omits the flag. Revocation is enforced in one place only — `xgen-node/src/app.rs:1539`, `is_revoked` at session-open, denying the revoked identity its **own** login. A third party looking that identity up learns nothing.

**CONSEQUENCE — Joe locked Option C (2026-07-24):** build all six steps, and drive the three wire-dependent rules from **book-internal seeds** rather than from live wire data.

| Locked rule | Data source in Leg D |
|---|---|
| §4 fill F1 ∪ F2 | ✅ live wire — `display_name`, `is_ai` arrive |
| §7 own-file storage | ✅ book-local |
| §6 E1 + E2 + E3 erasure | ✅ book-local (`last_seen`) |
| §5 V2 higher `update_version` wins | ⚠️ **seeded** — field is book-local, no wire source |
| §5 revocation-on-encounter | ⚠️ **seeded** — no wire source |
| §6 not-renewed badge | ⚠️ **seeded** — no wire source |

🔑 **THE POINT OF OPTION C: the logic is written and TESTED now, so the day `identity.record` widens it becomes field-mapping, not new design.** Do not simulate the missing fields on the wire, do not invent a side-channel, and do not quietly drop the three rules. Build them against seeded book state, and mark each seeded test with the reason.

⚠️ **NEVER populate a wire-absent field with a guess.** `update_version` defaults to `0`, `revoked` to `false`, `trust_assertion` to `None` on anything fetched. A fetched record must never *claim* an identity is valid — it simply has no opinion, and §6's display rule must treat absence as unknown, not as fine.

🔒 **FILED AS A MILESTONE: `M13 Client Identity Lookup Widening`** (`tasks/M13_CLIENT_IDENTITY_LOOKUP_WIDENING.md`, Status PENDING). It converts every seeded rule below into a live wire path. ⚠️ **D-127 decided there: a revoked Identity returns its record WITH `revoked` set, never `not_found`** — `not_found` stays reserved for erasure. Build Step 4/5/6 so that flag arriving from the wire needs no redesign. 📌 **Original filing note:** widening `identity.record` is a protocol change (Appendix I + Ch3 + node + client, and a look at whether federation replication needs the same). It is **out of this arc** and awaits a milestone name from Joe.

---
## §3 — Grounding facts (measured `1fd594c`)

Build against these; do not re-derive them.

- **Request/response op pattern** — mirror `ops::register` (`xgen-client/src/ops.rs:307-390`): `let conn = ctx.session.ensure_connected(ctx.node_override).await?;` → `conn.send_identity(&msg).await?` → `match conn.recv().await? { ... }` → best-effort `conn.goodbye("client_disconnect")`.
- **Storage pattern** — mirror `load_client_state` / `write_client_state` (`xgen-client/src/app.rs:4613-4630`): `data_dir.join(<file>)`, `serde_json::to_string_pretty`, plain `std::fs`.
- 🔑 **F1 and F2 read the SAME drained events.** `ops::members` (`ops.rs:2552-2558`) is two lines: `drain_space_events(ctx, space)` then `members_projection(space, &events)`. **F2 needs no transport change** — the client already derives Space membership by causal replay of the DAG it drains. The Phase-0 §4 cost note ("a transport change on `KnownSpace` / `get_spaces`") was **wrong**; corrected J-582. `KnownSpace` (`xgen-common/src/state.rs:185`) has no members field and needs none.
- ⚠️ **`MemberEntry` carries no `display_name`** — membership yields pubkeys and roles only. Every member still needs one `identity.get` to become renderable. That is F2's real cost: N fetches per Space, not a wire change.
- **`last_seen` shape** — RFC-3339 `String`, copied from `FederatedPeer.last_seen_at` (`xgen-common/src/state.rs:118`).
- **Clock harness for §6 E3** — `clock advance` / `clock set` already ship as node admin verbs behind `--features harness-control` (`xgen-node/src/admin_ops.rs:4437`, injected `MockClock`). Grace's aging needs a feature-gated build, not new machinery.
- **Corpus** — `docs/tests/scripts/ADDRESS_BOOK_SEED_CORPUS.md` v1.1 + six `.xgb` scripts, proven runnable from cold (J-582). Five NOW-tier identities; carol v2 and grace are book-file seeds built here.

---

## §4 — Steps

### Step 1 — `ops::identity_get()`

**Why first:** the book cannot fill without it. Measured: `IdentityMessage::Get` / `Record` / `NotFound` appear 9 times in `xgen-client`, **all inside integration-test harness code in `app.rs`** (1909, 6161, 6193). The wire capability is proven; no callable function exists.

**Build** in `xgen-client/src/ops.rs`:

```
pub async fn identity_get(
    ctx: &mut OpContext<'_>,
    identity_id: &str,
) -> Result<Option<FetchedIdentity>>
```

- Send `IdentityMessage::Get { protocol_version: "0.1".into(), identity_id: identity_id.into() }`.
- `Inbound::Identity(IdentityMessage::Record { .. })` ⇒ `Ok(Some(..))`.
- `Inbound::Identity(IdentityMessage::NotFound { .. })` ⇒ `Ok(None)` — **not an error.** An identity the node has never seen is a normal outcome.
- anything else ⇒ `bail!` in the shape `ops::register` uses.

`FetchedIdentity` mirrors **only** the seven fields §2 lists. **Do not add fields the wire does not carry.**

**Tests:** `Record` ⇒ `Some` with `display_name` and `is_ai` preserved · `NotFound` ⇒ `None` · unexpected inbound ⇒ error · `is_ai` absent on the wire ⇒ `false` (serde default, matching the human-record byte-identity rule at `types.rs:468`).

---

### Step 2 — `AddressBook`, `SeenRecord`, and the file

**Build** `xgen-client/src/address_book.rs`; declare it in `lib.rs`.

`SeenRecord` fields:

| Field | Type | Source |
|---|---|---|
| `identity_id` | `String` | wire |
| `display_name` | `Option<String>` | wire |
| `is_ai` | `bool` | wire (serde default `false`) |
| `home_node` | `String` | wire |
| `last_seen` | `String` (RFC-3339) | **book-local**, set on every touch |
| `update_version` | `u64` | **book-local, default 0** — see §2 |
| `revoked` | `bool` | **book-local, default false** — see §2 |
| `trust_assertion` | `Option<Value>` | **book-local, default None** — see §2 |

🔑 **THE BOOK STORES OBSERVATIONS, NOT CURRENT TRUTH (Joe, 2026-07-25).** Every record means *"as of `last_seen`, this was the state."* Two consequences that bind the whole build:

- A cached `revoked = true` is a **historical fact that can never become wrong** — revocation does not un-happen.
- A cached `revoked = false` is **also only "as of then"** ⇒ ⚠️ **staleness and absence must BOTH render as UNKNOWN, never as fine.** This generalises the J-582 badge rule from one field to the whole book, on a principle rather than as a special case.

📌 **`registered_at` is deliberately NOT stored** (trimmed 2026-07-25). It is provenance about *them*, not needed to recognise them, not needed to route to them, and required by no locked rule — so the book does not keep it. Every remaining field earns its place: `identity_id` (key) · `display_name` (the point) · `is_ai` (§3.6.10 transparency, spec-mandated) · `home_node` (routing, to re-fetch) · `last_seen` (§6 E3) · the three wire-absent fields (locked rules, §2). If a surface ever needs `registered_at`, it is one `identity_get` away.
`AddressBook` holds the records keyed by `identity_id` (a `BTreeMap<String, SeenRecord>` — deterministic serialisation order, which makes the file diffable and the tests stable).

**Storage:** load/save `xgen-client_address_book.json` from the instance data dir, mirroring `app.rs:4613-4630`.

⚠️ **A missing file is an EMPTY BOOK, not an error** — unlike `load_client_state`, which bails. §6 E1 erases by deleting the file, so absence is a normal, supported state and the client must keep working through it.

📌 **Decided by Chat (reversible):** a **corrupt** file returns an error and the book refuses to overwrite it. Silently discarding a damaged book and carrying on would be the polite behaviour; D-065 asks for the honest one. Surface it.

**Tests:** roundtrip save→load preserves every field · missing file ⇒ empty book, no error · corrupt file ⇒ error, file untouched on disk · serialisation order is stable across runs.

---

### Step 3 — Fill: F1 ∪ F2, off the critical path

🔒 **LOCKED (Joe, 2026-07-24): the fill runs OFF THE CRITICAL PATH.** The Space opens immediately; records resolve behind and the view updates as they land. **Never block a Space open on the fetch loop** — at N members that is an unbounded network wait in front of a UI action.

🔑 **F1 and F2 are one drain, not two passes.** From a single `drain_space_events(ctx, space)`:

- **F1 (author-on-sight)** — the distinct `sender` values across the **`message.*`** events only. ⚠️ **NOT every drained sender — CORRECTED 2026-07-25 (J-586), Joe ruled Reading B.** Every member authors their own `membership.join`, and those render as C2 system notices, so "any event you render" would put every member in F1, make F2 dead code, and contradict §4's own rationale (*"a member list built on F1 alone shows only talkers"*). Membership and state events DO NOT qualify.
- **F2 (membership sweep)** — the `identity_id` values from `members_projection(space, &events)`.

Union the two sets, subtract the identities the book **already holds** — held means *present*, with **no freshness window**: under the wire ceiling a re-fetch is a provable no-op (always `update_version 0`; nothing mutates `display_name`), so the window lands with M13, when a re-fetch first becomes informative. Fetch the remainder through Step 1 and upsert each, stamping `last_seen`.

🔒 **AND TOUCH `last_seen` ON THE WHOLE OBSERVED SET, NOT ONLY THE FETCHED ONES (Joe, 2026-07-25).** Re-observing a held identity advances its `last_seen` with no re-fetch. ⚠️ **This is the observation contract, not an optimisation:** every record means *"as of `last_seen`, this was the state"*, so a frozen `last_seen` on someone you demonstrably saw today makes the record **lie about its own central claim** — and E3 would then evict a continuously-present member. `SeenRecord` already specified `last_seen` as "set on every touch"; this spells out what Step 3 left implicit.

⚠️ **The union is the point of the lock.** F1 alone misses silent members (bob in the corpus); F2 alone misses authors who have left the Space. Neither is a superset of the other — this is exactly why §4 locked F1 ∪ F2 rather than either one.

📌 **F3 was REFUSED on spec** and stays refused: Ch2 §Cross-Space Discoverability forbids a global membership index. Fill from Spaces this client is in — never sweep the node for identities the user has no relationship with.

**Tests:** author-only identity enters via F1 · silent member enters via F2 · an identity that is both enters once, not twice · `NotFound` from Step 1 is skipped without poisoning the book · Space open returns before the fetch loop completes.

---

### Step 4 — Freshness: merge on encounter (§5 V2)

**Rule:** on re-encountering an identity, the record with the **higher `update_version` wins**; equal or lower is discarded, not merged field-by-field.

⚠️ **SEEDED, per §2 Option C.** `identity.record` carries no `update_version`, so nothing on the wire can currently produce a second version. Build and test the merge against **book-file seeds**: two records for carol — v1 `carol`, v2 `Carol M.` at a higher `update_version` — written directly into `xgen-client_address_book.json`.

**Mark every test in this step with why it is seeded.** A future reader must not mistake a seeded fixture for a live path.

📌 **V3 (revocation push) is UNAVAILABLE and out of arc** — Leg A proved no node→client identity-push path exists (`7ab743e`).

**Tests:** higher version replaces lower · lower version is discarded, incumbent untouched · equal version is a no-op · merge is whole-record, not per-field · **a record fetched from the wire (version 0) never displaces a seeded higher version.**

---

### Step 5 — Erasure: E1 + E2 + E3 (§6)

- **E1 — wholesale.** Delete `xgen-client_address_book.json`. The book must return empty and keep working (Step 2).
- **E2 — targeted.** Remove one identity by id.
- **E3 — aged.** Evict records whose `last_seen` is older than **N**.

**N is per-tier, Auth-Module-declared, and a FINITE floor — never `∞`.** T1 = **182 d**, a **PROVISIONAL development value** (J-580) to be re-tuned when real Auth Modules exist. Retention **rises** with tier; T4 approaches keep-as-evidence, because safety outranks minimisation.

⚠️ **`TIER*_TTL_DAYS` (`tiers.rs:22-24`) ARE NOT N.** They are assertion-renewal TTLs and they run the **opposite** direction. Do not read them as retention.

⚠️ **"Not renewed" FLAGS, it never EVICTS.** It is derived **locally** from a cached `valid_until` — no push, no node round-trip. Display belongs to **M-RP-MEMBERS**, not here; this step only makes the state derivable.

🔒 **AND THE BADGE MUST RENDER NOTHING ON `None` — this is the J-582 finding, and it inverts the obvious implementation.** The fill path populates `trust_assertion` for **nobody**; `None` is the common case, not the exception. A badge computing `now > valid_until` against a missing assertion would mark **every ordinary identity** as expired — a warning nobody earned and nobody can act on. **Absence is "no opinion", never "expired" and never "fine".**

**grace (E3) is a seeded case:** write a record with `last_seen` older than N, advance the clock via the `--features harness-control` verbs (§3), assert eviction fires.

**Tests:** E1 removes the file and the book reloads empty · E2 removes one and leaves the rest · E3 evicts past N and **keeps a record exactly at the boundary** · `trust_assertion = None` produces no badge state · a per-tier N above T1 retains what T1 would evict.

---

### Step 6 — Load the corpus and assert the five NOW-tier cases

Run `docs/tests/scripts/ADDRESS_BOOK_SEED_CORPUS.md` v1.1 (six `.xgb` + two node-admin ops; §3/§4 of that doc are the exact operations) and assert the book's resulting state:

| Identity | Assert |
|---|---|
| alice | present via **F1** (authored) |
| bob | present via **F2** (silent member, authors nothing) |
| erin | present with `is_ai = true` |
| dave | ⚠️ **seeded** — revoked-on-encounter, per §2 |
| frank | ⚠️ **seeded** — not-renewed badge state, per §2 |

⚠️ **dave and frank cannot be driven from the live node** even though the corpus seeds them there. The node registry holds their revocation and expiry; **`identity.record` carries neither to the client.** Assert them from book-internal seeds and mark them as such. *This corrects an over-broad claim in J-582, which read frank's badge input as reachable after measuring it node-side only.*

**Tests:** all five assertions above · the book survives a full save→load cycle with the corpus loaded · no identity appears twice.

---

## §5 — Definition of Done

🔒 **ALL MET — LEG D CLOSED 2026-07-25 (J-586).** Built by Clair from v1.1; live-verified by Chat at `167055d` + working tree.

- [x] Step 1 — `ops::identity_get()` + `identity_get_on` + pure `parse_identity_get_response`; `NotFound` ⇒ `Ok(None)`
- [x] Step 2 — `AddressBook` + `SeenRecord` in `xgen-client/src/address_book.rs`; `xgen-client_address_book.json`; missing ⇒ empty book, corrupt ⇒ honest error
- [x] Step 3 — F1 ∪ F2 from one drain, off the critical path, `last_seen` touched across the whole observed set
- [x] Step 4 — merge on encounter, higher `update_version` wins (carol seeds, marked)
- [x] Step 5 — E1 + E2 + E3 + `trust_lapsed`; `None` ⇒ no badge (seeds marked)
- [x] Step 6 — corpus loaded, five NOW-tier assertions pass
- [x] Every wire-absent rule tested from a **seed** and **marked with the reason**
- [x] `cargo` floor moved **1553 → 1585 / 0 / 62 across 56** — the honest "Rust landed" signal, verified independently by Chat
- [x] `svelte-check` holds **by scope** (zero frontend touched) — not re-measured, and said so
- [x] Zero `skin.css`; 3 files, `xgen-client` only

🔑 **LIVE ORCHESTRATION PASS — GREEN (Chat, J-586).** Against a real scratch node, with **no caller-side connection management**: cold `candidates 6 / fetched 6 / touched 0` · warm `candidates 0 / touched 6` · warm again **succeeded** (proving the early-return path clears) · a bogus space genuinely errored · a valid fill after that error **succeeded** (proving the error path clears). `last_seen` advanced across passes for all six. bob present having authored no message — **F2 proven live**. erin `is_ai: true` off the wire. Every record `update_version 0 / revoked false / trust_assertion None` — the wire ceiling exactly as §2 specifies.

⚠️ **The live pass found the defect that committed tests could not:** `fill_from_space` was not re-entrant — the fetch loop's closing `goodbye` left `session.conn = Some(dead)`, and the **warm early return** leaked it on the most frequent path of all. Fixed by a wrapper that captures the inner result and clears on **every** exit, including the `?`-skipped error paths.

📌 **"Commit pushed" is deliberately NOT on this list.** `Status: COMPLETED` in the header is the shipped signal; Joe owns the push.
## §6 — Out of scope — do NOT build these here

- **Any UI.** The book is a data layer. Rendering names, badges and rosters is **M-RP-MEMBERS**, which unblocks after this build.
- **Widening `identity.record`.** Now `M13 Client Identity Lookup Widening` (PENDING) — see §2. Do not build any part of it here.
- **An `identity.update` emitter.** Nothing emits it (J-576); that gap is filed separately.
- **③ contacts** — private encrypted annotations. Ch2 specifies **no contact-acquisition flow at all** (grepped; zero). Cannot be built until written.
- **④ presence** — Space-scoped, TTL, explicitly *not stored and not an Event*.
- **D-126 humane pubkey label.** Adopted, non-protocol, display-only. It matters here only because it is the natural placeholder for a row whose record has not landed yet — but the choice is **appearance, and Joe's**.
- **F3 global sweep.** Refused on spec, permanently.

---

## §7 — ⚠️ RE-OPEN-ON-BUILD CLAUSE (mandatory, D-122)

If building any step shows a locked decision to be **unbuildable, more expensive than its lock assumed, or resting on a fact that measurement contradicts** — **STOP AND REPORT.** Do not reconcile it in code, do not route around it, do not widen scope to absorb it.

This clause has already paid for itself twice in this arc: the Phase-0's F2 transport-change cost was wrong (F2 is cheaper), and `identity.record`'s field set makes three locked rules undrivable from the wire (§2). **Both were found by measuring rather than assuming, and both changed the shape of the work.** A third is likelier than not.

---

## §8 — Handoff

**Chat** authored this off the locked Phase-0 (`tasks/M_RP_ADDRESS_BOOK.md` v1.6, §4–§7 locked J-579/J-580). **Clair** implements. **Joe** locks and pushes.

**On completion:** `M-RP-MEMBERS` unblocks. The wire-widening milestone and the `identity.update` emitter remain filed and unscheduled.