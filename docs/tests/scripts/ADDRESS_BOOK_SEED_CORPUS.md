# M-RP-ADDRESS-BOOK — Seed Corpus (Leg C)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-24  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is

The seed corpus for the client-side address book (M-RP-ADDRESS-BOOK, Phase-0 locked J-579/J-580). It is **not arbitrary test data**: every identity exists to exercise exactly one locked decision. If a locked rule has no entry that fails without it, the corpus is incomplete.

Locked policy this corpus serves: **§4** F1 ∪ F2 · **§5** V2 refresh-on-encounter · **§6** E1 + E2 + E3 (T1 N = 182 d provisional) + not-renewed badge · **§7** own file.

Measured against HEAD `47ed16b`. Scripts live in `docs/tests/scripts/` (reuse, per Joe).

---

## §1 — The design rule

Each locked rule maps to at least one corpus entry that **fails without it**. Coverage table:

| Locked rule | Entry | Mechanism | Tier |
|---|---|---|---|
| §4 F1 author-on-sight | **alice** — speaks | client `register` + `send` | NOW |
| §4 F2 membership sweep | **bob** — silent member | client `register` + `join`, never sends | NOW |
| AI handling (`is_ai`) | **erin** — AI identity | client config `AiSection.is_ai = true`, then `register` | NOW |
| not-renewed badge | **frank** — expired assertion | node admin `identity_set_trust_expiry` (past) | NOW |
| §5 revocation-on-encounter | **dave** — revoked | node admin `identity_revoke` | NOW |
| §5 V2 higher `update_version` wins | **carol** — two versions | seed two records into the book file (Option C) | LEG D |
| §6 E3 evict past N | **grace** — aged > 182 d | book `last_seen` field + clock advance | LEG D |

**Seven identities, minimum-complete.** Small enough to inspect by hand; a failing build points at one named decision.

---

## §2 — The two tiers, and why they exist

The corpus splits along a real seam the measurement revealed — **not a shortcut**:

**NOW — node-exercisable (five: alice, bob, dave, erin, frank).** These seed *identities* against the live node registry. The book merely projects them; they need no book to exist. Produced by client `.xgb` scripts + two node-admin operations + one config flag.

**LEG D — book-internal (two: carol, grace).** These exercise rules that live *inside* the book's own storage, which is unbuilt until Leg D:
- **carol (§5 V2)** — V2 resolves a conflict between two record versions. ⚠️ **MEASURED at `47ed16b`: no surface mutates an existing identity's `display_name`.** Client has 0 `sign_update` refs (cannot emit `identity.update`); node admin has 0 `display_name =` assignments; the `identity.update` wire message exists but **nothing emits it** (J-576 finding). ⇒ **the protocol has no live producer of a second record version.** V2 is correct and forward-looking but presently unexercisable end-to-end. **Option C (Joe-locked):** seed two records for carol (v1 `carol`, v2 `Carol M.`) **directly into the book file** at Leg D, testing the book's merge logic (higher version wins) at its true seam — the client-side merge — without a non-existent wire path. The wire-update gap is **filed separately** (identity-update emitter, a future milestone), not worked around here.
- **grace (§6 E3)** — eviction needs a `last_seen` timestamp, which is an **address-book field that does not exist until Leg D**. Folding it now would design a slice of the book schema ahead of Leg D's Phase-0 — D-071 in reverse (seed twice). Deferred. ⚠️ **Precedent for the field shape: `FederatedPeer.last_seen_at: String` (RFC-3339), `xgen-common/src/state.rs:118`** — Leg D copies this shape; aging grace is then a one-line clock advance against a field that finally exists.

🔑 **This split is the corpus doing its job** — it reveals which locked rules are node-exercisable and which are book-internal, which is exactly what Leg D's own test plan needs.

---

## §3 — The executable scripts (NOW tier)

One `.xgb` per client role, following the measured syntax (`register --name` · `create-space [--auth_tier]` · `create-room` · `join --space` · `send --space --room --text`). IDs (space/room hashes) are captured from the setup run's log and pasted into follower scripts — the existing multiparty pattern.

- `addressbook_seed_alice.xgb` — register alice, create the seed Space + Room, send one message. (F1; also the setup run that mints the IDs.)
- `addressbook_seed_bob.xgb` — register bob, join the Space, **send nothing**. (F2 — bob appears only via membership sweep.)
- `addressbook_seed_erin.xgb` — **preceded by config**: launch erin's client with `AiSection.is_ai = true` (config, not a CLI flag — measured: client `register` has no `--ai`). Register erin, join, send one message so she is both AI and F1-visible.
- `addressbook_seed_carol_v1.xgb` — register carol as `carol`, join, send. (Her v1 record enters normally; the **v2 rename is Option C at Leg D**, not a script here.)
- `addressbook_seed_dave.xgb` — register dave, join, send. (Dave enters normally; his **revocation is a node-admin op**, §4 below, not a client line.)

**Node-admin operations (NOW, not `.xgb` — these are node-side admin verbs, run against the node after the client seeds):**
- `identity_revoke` on dave's `identity_id` → produces the revoked record §5 must correct on encounter.
- `identity_set_trust_expiry` on frank's `identity_id` with a past `valid_until` → produces the expired-assertion record the not-renewed badge reads.

⚠️ **frank needs a client registration first** (to exist in the registry) before `identity_set_trust_expiry` can target him — add `addressbook_seed_frank.xgb` (register + join + send), then the admin op. *(Six client scripts total: alice, bob, erin, carol_v1, dave, frank.)*

---

## §4 — Exact operations, per identity

**alice** (F1, + setup):
```
register --name alice
create-space --name "Address Book Seed"
create-room --space <SPACE> --name general
send --space <SPACE> --room <ROOM> --text "alice-msg-1"
```

**bob** (F2 — silent):
```
register --name bob
join --space <SPACE>
```
(No send. Bob is a member who never authors — the case F1 alone misses.)

**erin** (AI): config `AiSection.is_ai = true` + `capabilities` as needed, THEN:
```
register --name erin
join --space <SPACE>
send --space <SPACE> --room <ROOM> --text "erin-msg-1"
```

**carol** (V2, v1 only here):
```
register --name carol
join --space <SPACE>
send --space <SPACE> --room <ROOM> --text "carol-msg-1"
```
v2 (`Carol M.`, higher `update_version`) = **Leg D book-file seed (Option C).**

**dave** (revocation):
```
register --name dave
join --space <SPACE>
send --space <SPACE> --room <ROOM> --text "dave-msg-1"
```
THEN node admin: `identity_revoke <dave_identity_id>`.

**frank** (not-renewed):
```
register --name frank
join --space <SPACE>
send --space <SPACE> --room <ROOM> --text "frank-msg-1"
```
THEN node admin: `identity_set_trust_expiry <frank_identity_id> <past-timestamp>`.

**grace** (E3): **Leg D only** — book seed with `last_seen` older than N (182 d), via the `FederatedPeer.last_seen_at` field shape; clock advance to trigger eviction.

---

## §5 — Handoff to Leg D (Clair)

The runbook Clair builds from must cover, in the book's own storage layer:
1. **carol v1+v2 book-file seed** (Option C) + a test that the higher `update_version` wins (§5 V2).
2. **grace aged-record seed** using a `last_seen` field shaped after `state.rs:118` + a clock-advance test that eviction fires past N=182 d (§6 E3).
3. The five NOW-tier identities loaded via the `.xgb` set + two node-admin ops, asserting F1 (alice), F2 (bob), AI (erin), revoked-on-encounter (dave), not-renewed badge derived from `valid_until` (frank).
4. ⚠️ **Confirm the F1/F2 fill path populates `IdentityRecord.trust_assertion`** (it is `Option` — may arrive `None`); frank's badge depends on it (N-164 pending, from J-580).

⚠️ **Node-admin CLI wiring lookup for Leg D:** `identity_revoke` / `identity_set_trust_expiry` are `pub async fn` in `xgen-node/src/admin_ops.rs` (:1031, :1125); their CLI/aicontrol invocation surface is a Leg-D lookup (not located this session).

---

## §6 — Provisional-value note

N = 182 d and every per-tier retention figure are **provisional development values** (J-580), to be re-tuned when real Auth Modules exist. grace's "> 182 d" aging tracks whatever N is set to at build time, not a frozen constant.