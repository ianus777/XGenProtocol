# M-RP-ADDRESS-BOOK — Seed Corpus (Leg C)
> **Status**: ACTIVE  
> Version: 1.1  
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

Specified against HEAD `47ed16b`. **Executed and verified against HEAD `86c753c` (J-582) — see §7.** Scripts live in `docs/tests/scripts/` (reuse, per Joe).

---

## §1 — The design rule

Each locked rule maps to at least one corpus entry that **fails without it**. Coverage table:

| Locked rule | Entry | Mechanism | Tier |
|---|---|---|---|
| §4 F1 author-on-sight | **alice** — speaks | client `register` + `send` | NOW ✅ |
| §4 F2 membership sweep | **bob** — silent member | client `register` + `join`, never sends | NOW ✅ |
| AI handling (`is_ai`) | **erin** — AI identity | client `init --ai`, then `register` | NOW ✅ |
| not-renewed badge | **frank** — expired assertion | node admin `identity set-trust-expiry` (past) | NOW ✅ |
| §5 revocation-on-encounter | **dave** — revoked | node admin `identity revoke` | NOW ✅ |
| §5 V2 higher `update_version` wins | **carol** — two versions | seed two records into the book file (Option C) | LEG D |
| §6 E3 evict past N | **grace** — aged > 182 d | book `last_seen` field + clock advance | LEG D |

**Seven identities, minimum-complete.** Small enough to inspect by hand; a failing build points at one named decision.

---
## §2 — The two tiers, and why they exist

The corpus splits along a real seam the measurement revealed — **not a shortcut**:

**NOW — node-exercisable (five: alice, bob, dave, erin, frank).** These seed *identities* against the live node registry. The book merely projects them; they need no book to exist. Produced by client `.xgb` scripts + two node-admin operations + one `init` flag. **All five are executed and verified — §7.**

**LEG D — book-internal (two: carol, grace).** These exercise rules that live *inside* the book's own storage, which is unbuilt until Leg D:

- **carol (§5 V2)** — V2 resolves a conflict between two record versions. ⚠️ **MEASURED at `47ed16b`: no surface mutates an existing identity's `display_name`.** Client has 0 `sign_update` refs (cannot emit `identity.update`); node admin has 0 `display_name =` assignments; the `identity.update` wire message exists but **nothing emits it** (J-576 finding). ⇒ **the protocol has no live producer of a second record version.** V2 is correct and forward-looking but presently unexercisable end-to-end. **Option C (Joe-locked):** seed two records for carol (v1 `carol`, v2 `Carol M.`) **directly into the book file** at Leg D, testing the book merge logic (higher version wins) at its true seam — the client-side merge — without a non-existent wire path. The wire-update gap is **filed separately** (identity-update emitter, a future milestone), not worked around here. 📌 **Confirmed live (§7):** every seeded record carries `update_version: 0`, so the field carol v2 must out-rank exists and starts at a known floor.
- **grace (§6 E3)** — eviction needs a `last_seen` timestamp, which is an **address-book field that does not exist until Leg D**. Folding it now would design a slice of the book schema ahead of Leg D's own Phase-0 — D-071 in reverse (seed twice). Deferred. ⚠️ **Precedent for the field shape: `FederatedPeer.last_seen_at: String` (RFC-3339), `xgen-common/src/state.rs:118`** — Leg D copies this shape; aging grace is then a one-line clock advance against a field that finally exists. 📌 **The clock advance already exists (§7):** `clock advance` / `clock set` are shipped admin verbs behind `--features harness-control` (`xgen-node/src/admin_ops.rs:4437`), driving an injected `MockClock`. Leg D needs a feature-gated build, not new machinery.

🔑 **This split is the corpus doing its job** — it reveals which locked rules are node-exercisable and which are book-internal, which is exactly what Leg D's own test plan needs.

---
## §3 — The executable scripts (NOW tier)

One `.xgb` per client role, following the measured syntax (`register --name` · `create-space [--auth_tier]` · `create-room` · `join --space [--room]` · `send --space --room --text`). IDs (space/room hashes) are captured from the setup run's log and pasted into follower scripts — the existing multiparty pattern.

⚠️ **ROOM MEMBERSHIP IS SEPARATE FROM SPACE MEMBERSHIP — the v1.0 defect, corrected here.** `join --space <S>` grants **Space** membership only. `send` requires **Room** membership and the node rejects it otherwise (`code 4000: step 11: sender is not a member of room`). Every follower that sends therefore needs **two** join lines: `join --space <S>` then `join --space <S> --room <R>`. v1.0 omitted the second and erin failed live on it (J-582). The multiparty suite had this right — it carried separate `*_join_room.xgb` scripts.

- `addressbook_seed_alice.xgb` — register alice, create the seed Space + Room, send one message. (F1; also the setup run that mints the IDs.)
- `addressbook_seed_bob.xgb` — register bob, join the Space, **send nothing**. (F2 — bob appears only via membership sweep. **No room join: bob never sends, and F2 reads Space membership.**)
- `addressbook_seed_erin.xgb` — **preceded by `init --ai`** (see below). Register erin, join Space + Room, send one message so she is both AI and F1-visible.
- `addressbook_seed_carol_v1.xgb` — register carol as `carol`, join Space + Room, send. (Her v1 record enters normally; the **v2 rename is Option C at Leg D**, not a script here.)
- `addressbook_seed_dave.xgb` — register dave, join Space + Room, send. (Dave enters normally; his **revocation is a node-admin op**, below, not a client line.)
- `addressbook_seed_frank.xgb` — register frank, join Space + Room, send. (Frank enters normally; his **expired assertion is a node-admin op**.)

⚠️ **erin is staged by a CLI flag, not a hand-edited config — v1.0 was wrong.** `xgen-client init --ai` (`xgen-client/src/app.rs:716`) writes the whole `[ai]` section (`is_ai = true`, `plugin = "echo"`, the capability map); `register` then sends `is_ai = true`. Measured 2026-07-24. There is no `--ai` on `register` itself, which is what v1.0 half-saw.

⚠️ **alice runs in three passes, and cannot be a single batch.** Her own later lines consume the SPACE and ROOM ids her earlier lines mint, and `.xgb` has no substitution. Pass 1 = `register` + `create-space`; pass 2 = `create-room`; pass 3 = `send`. This mirrors the multiparty precedent (`clientA_pass1` / `pass1b` / `pass2`). The committed script holds all four lines because it documents the full intent; the runner splits it.

📌 **Committed scripts keep `<SPACE>` / `<ROOM>` placeholders.** The runner substitutes into **run-copies outside the repo** (J-582 used `C:\xgen-scratch\ab-populate\scripts`), so the committed set stays clean and re-runnable and the working tree stays undirty.

**Node-admin operations (NOW, not client `.xgb` — these are node-side admin verbs, run against the node after the client seeds).** ⚠️ **Invocation surface, located J-582:** the node accepts `--batch` exactly as the client does; each line is parsed by `admin_ops::AdminCli` (clap, `no_binary_name`) in `xgen-node/src/pipe.rs:98` and dispatched into `admin_ops::*`. **Output lands on the resident's stdout, not the caller's.**

```
identity revoke <identity_id> [--reason <text>]
identity set-trust-expiry <identity_id> --expiry <RFC3339>
identity list          # read-only allowlist — how the runner reads the ids back
identity show <id>     # verification read, prints the record as JSON
```

The same two verbs are also wired into `--aicontrol` (`xgen-node/src/aicontrol.rs:362-363`); `--batch` is the path the corpus uses.

---
## §4 — Exact operations, per identity

**alice** (F1, + setup) — three passes, see §3:
```
register --name alice
create-space --name "Address Book Seed"
create-room --space <SPACE> --name general
send --space <SPACE> --room <ROOM> --text "alice-msg-1"
```
(alice is auto-joined to the Room she creates — she needs no `join`.)

**bob** (F2 — silent):
```
register --name bob
join --space <SPACE>
```
(No send, no room join. Bob is a member who never authors — the case F1 alone misses.)

**erin** (AI): `xgen-client --instance <label> init --passphrase= --ai` FIRST, THEN:
```
register --name erin
join --space <SPACE>
join --space <SPACE> --room <ROOM>
send --space <SPACE> --room <ROOM> --text "erin-msg-1"
```

**carol** (V2, v1 only here):
```
register --name carol
join --space <SPACE>
join --space <SPACE> --room <ROOM>
send --space <SPACE> --room <ROOM> --text "carol-msg-1"
```
v2 (`Carol M.`, higher `update_version`) = **Leg D book-file seed (Option C).**

**dave** (revocation):
```
register --name dave
join --space <SPACE>
join --space <SPACE> --room <ROOM>
send --space <SPACE> --room <ROOM> --text "dave-msg-1"
```
THEN node admin: `identity revoke <dave_identity_id> --reason "..."`.

**frank** (not-renewed):
```
register --name frank
join --space <SPACE>
join --space <SPACE> --room <ROOM>
send --space <SPACE> --room <ROOM> --text "frank-msg-1"
```
THEN node admin: `identity set-trust-expiry <frank_identity_id> --expiry <past-RFC3339>`.

**grace** (E3): **Leg D only** — book seed with `last_seen` older than N (182 d), via the `FederatedPeer.last_seen_at` field shape; clock advance to trigger eviction.

---

## §5 — Handoff to Leg D (Clair)

The runbook Clair builds from must cover, in the book's own storage layer:

1. **carol v1+v2 book-file seed** (Option C) + a test that the higher `update_version` wins (§5 V2). Seeded records start at `update_version: 0` (measured, §7).
2. **grace aged-record seed** using a `last_seen` field shaped after `state.rs:118` + a clock-advance test that eviction fires past N=182 d (§6 E3). Use the shipped `clock advance` / `clock set` harness verbs (`--features harness-control`).
3. The five NOW-tier identities loaded via the `.xgb` set + two node-admin ops, asserting F1 (alice), F2 (bob), AI (erin), revoked-on-encounter (dave), not-renewed badge derived from the cached expiry (frank).
4. 🔒 **`trust_assertion` IS NORMALLY `None` — ANSWERED, and it is the opposite of the convenient answer.** See §7. The fill path does **not** populate it for ordinary identities; `None` is the common case, not the exception. **The badge logic must treat `None` as "no assertion to judge" and render nothing** — it must not read absence as expiry.

---

## §6 — Provisional-value note

N = 182 d and every per-tier retention figure are **provisional development values** (J-580), to be re-tuned when real Auth Modules exist. grace's "> 182 d" aging tracks whatever N is set to at build time, not a frozen constant.

---
## §7 — POPULATE: executed and verified (J-582, 2026-07-24)

Run against HEAD `86c753c` with the 2026-07-23 debug binaries (measured: 15 commits since the last floor, all `.md`/`.xgb`, zero `.rs` — no rebuild needed). Fully headless and isolated under `C:\xgen-scratch\ab-populate`, outside the repo. Node `--service` on `ws://127.0.0.1:8080/xgen`, `local_mode = true`. **Executed twice**: once discovering the §3 room-join defect, then **cold from scratch against the corrected committed scripts** — the numbers below are the cold run.

**Registry after the run — six records, read via `identity show`:**

| Identity | `is_ai` | `revoked_at` | `update_version` | `trust_assertion` |
|---|---|---|---|---|
| alice | — | — | 0 | `None` |
| bob | — | — | 0 | `None` |
| erin | **true** | — | 0 | `None` |
| carol | — | — | 0 | `None` |
| dave | — | **set** | 0 | `None` |
| frank | — | — | 0 | `{"expiry":"2026-01-15T00:00:00Z"}` |

**DAG after the run — 16 events:** 1 `state.space_create` · 1 `state.room_create` · 9 `membership.join` · 5 `message.text`. **bob authored exactly one event** — his Space join, no message. F2 isolation is clean and observable. alice has no explicit join: hers is implicit at Space creation, matching the multiparty pairing table.

`identity revoke` reported dave revoked with **1 stale membership space** — the A5-D1 honest report, no cascade, exactly as specified.

🔑 **THE N-164 ANSWER, AND IT INVERTS THE ASSUMPTION.** In local mode the F1/F2 fill path leaves `trust_assertion = None` on **every** identity. Frank has one **only because `set_trust_expiry` synthesises a minimal `{"expiry": ...}` when the record has none** (`xgen-core/src/identity/registry.rs:205`; it reported the previous expiry as `(none)`). ⇒ **the not-renewed badge cannot assume a populated assertion.** A badge that computes `now > valid_until` against a missing assertion must render nothing, not "expired" — otherwise every ordinary identity in a local-mode deployment wears an expiry warning it never earned. Filed into §5.4 as a build-time constraint on Leg D.

📌 **Two operational findings, recorded so the next runner does not pay for them again:** node-admin `--batch` output surfaces on the **resident's** stdout, not the invoking process; and `Start-Process -PassThru -RedirectStandardOutput` hangs the MCP PowerShell tool even when the launch itself succeeds — launch detached without `-PassThru`.