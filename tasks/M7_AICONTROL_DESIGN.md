# M7 `--aicontrol` — Design (decision log)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-06-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

The M7 `--aicontrol` design decision log. Locks are `AC-D#` (arc-local per D-069), resolved one-by-one with Joe. Builds on `tasks/M7_AICONTROL_AUDIT.md` (v1.0) and `docs/xgen_aicontrol_implementation.md` (v1.1, the canonical spec this milestone implements). M7 is **adapter work** wrapping the shipped `xgen-node-lib::admin_ops::*` + `xgen-client-lib::ops::*` surfaces in the AI-shape JSONL protocol — not a protocol change. No code until the runbook; Clair stood down.

## Lock order (from the Phase-0 audit)

| # | Decision | Status |
|---|---|---|
| AC-D1 | `cmd` verb-exposure model | 🔒 **LOCKED** (A — space-joined CLI path) |
| AC-D2 | Reply/error envelope | 🔒 **LOCKED** (flat) |
| AC-D3 | §12.2 mechanicals (splits a–d) | 🔒 **LOCKED** (a, b, c, d) |
| AC-D4 | Pipe-level authentication policy (the M6-deferred §2.6.1 question, now M7's) | 🔒 **LOCKED** (OS-ACL-delegated; no in-protocol auth in v1) |
| AC-D5 | Client `ops::*` coverage gap — `create-dm-space` / `members` / `leave` | 🔒 **LOCKED** (defer all three) |
| AC-D6 | Replay-safety / idempotency keys | 🔒 **LOCKED** (defer; do-it-over in v1) |

**Not AC locks (recorded routing):** §12.1 "M6 deliverables" is **superseded** (closed by M6 shipping — audit Finding 1). `config-reload` / the doc's §11 live-reload is **out of M7-`--aicontrol` core** → routes to the separate **M7-standalone (live config reload)** milestone.

---

## AC-D1 — `cmd` verb-exposure model — 🔒 LOCKED (A: space-joined CLI path)

**Decision: the JSONL `cmd` field is the CLI command path minus the binary name.** Verb list = whatever clap exposes; there is no separate AI-control naming layer.

**1. The rule, structurally.** The dispatcher splits `cmd` on the **first space**. Two delimiters, two roles:
- **Space = the category/verb separator (structural).** It is what the dispatcher splits on.
- **Hyphen = intra-token (lexical).** It is part of a multi-word name (`set-node-policy`, `auth-module`) and is never a split point.

This dissolves the `auth-module` collision rather than dodging it: `auth-module register` → `["auth-module", "register"]` unambiguously, because **no token contains a space**. Verified against the shipped surface: every category is single-token except the hyphenated `auth-module`; every verb is a single hyphenated token. So split-on-first-space yields `[category, verb]` (Node) or `[verb]` (Client) in every case.

Examples: `{"cmd":"send", ...}` (client, 1 token) · `{"cmd":"federation accept", ...}` (node, 2 tokens) · `{"cmd":"space set-node-policy", ...}` · `{"cmd":"auth-module register", ...}`.

**2. Why A over C (wire stability).** C (`{"cmd":"federation","sub":"accept"}`) was considered and **rejected on coupling, not overlooked.** Under A the wire carries an opaque command-path string and the category/verb split is a *dispatcher detail*; under C the split lives in the wire envelope, coupling the wire to clap's internal type grouping (`AdminCommand::Federation(FederationCommand::Accept)`). A flat string decoupled from internal type structure matches the project's wire-invariance instinct (§J.5 / the XGID philosophy: the wire carries opaque strings, not internal structure). B (hyphen-joined, `federation-accept`) is out on the `auth-module` delimiter collision.

**3. Reserved-control-verb carve-out.** Control verbs are checked **before** CLI-path resolution. `state` is already reserved by the canonical doc's own §6.1/§7.1 tables (it is not a CLI op) — pre-existing, not a new imposition. `subscribe` / `unsubscribe` live on the events pipe (§3), not the command pipe. **Rule: control verbs first; everything else is CLI-path.**

**Sub-points (confirm-at-design, resolved — not re-litigated):**
- Client token spelling follows the **CLI surface** (`create-space`, hyphen), not the `ops::*` fn name (`create_space`).
- `args` keys stay **snake_case** per §4.1.

**Consequence baked in.** AC-D1 lets us **delete §7's flat `federate-accept`-style names outright** and replace them with "verb list = whatever clap exposes; `cmd` = the CLI path." This is Finding-2 drift dissolved at the root — exactly the audit's mandate. §7 of the canonical doc gets rewritten under this rule when M7's doc-sync lands.

---

## AC-D2 — reply/error envelope — 🔒 LOCKED (flat)

Decisive constraint: the two surfaces report errors differently. **Node `admin_ops::*`** returns structured `AdminError { code, stage, message }` (band code + a 6-value `Stage`); **client `ops::*`** returns **`anyhow::Result`** — free-form message strings, no structured `code`/`stage`. The envelope absorbs both.

**Reply (§4.2).** `ok` replies carry `data` only. **`stage` is error-only** — never present on success.

**Error (§4.3) — flat envelope.**
- **Mandatory (always present):** `code`, `category`, `message`, `instance_state`.
- **Optional-by-source:** `stage`, `hint`, and the band code — present only when the source supplies them.
- **`category` is a closed enumerated set** — `protocol · lifecycle · argument · connection · timeout · permission` — and **alone disambiguates the `code` namespace.** The driver branches on `category`, never parses `code` to learn which namespace it is in. Adding a category is a deliberate envelope change, not an ad-hoc string.
- **`stage`** is synced to the real shipped `Stage`: `validate · authorize · register · persist · notify · federate` (6 variants; `authorize` always-passes in v1 per D-082, reserved for M7+ per-verb gating). This corrects the doc's mistaken 4-variant sketch.

Per-source mapping:

| Source | `code` | `category` | `stage` | `message` |
|---|---|---|---|---|
| Node verb error (`AdminError`) | band code (`SPACE_8005`, `FED_3041`, …) | `protocol` | the failure stage | `AdminError.message` |
| Client verb error (`anyhow`) | `GENERIC_4000` (no band code exists) | `protocol` | — (absent) | the `anyhow` text |
| Control-surface error | uppercase-snake (`BAD_ARGUMENT`, `TIMEOUT`, …) | matching category | — | human description |

**Forward compatibility.** The envelope is the protocol lock and is **forward-compatible with structured client errors**: if `ops::*` later grows a structured `code`/`stage`, the same fields carry them with no envelope change. Today's client message-only mapping is a *source* lossiness, **not a shape limitation**.

**Accept (D-065 honest-scope).** The client→envelope path is lossy on `code`/`stage` because `ops::*` is `anyhow`-based. Structuring client ops errors is a real future improvement, **out of M7 scope** — M7 is an adapter; it does not refactor the client error type.

**Rejected alternative.** Separated/nested (top-level control `code` + nested `protocol:{code,stage}`) — rejected: costs a wrapper code + an extra concept, and the client side has nothing to put in the nest. Flat matches the canonical §4.3 sketch, the shipped `AdminError`'s own documented "serialise code/stage/message" intent, and the AC-D1 wire-minimalism instinct.

---

## AC-D3 — §12.2 mechanicals

Four largely-independent sub-locks.

### AC-D3a — per-command default timeouts — 🔒 LOCKED (3-tier class rule)

A 3-tier rule keyed off verb class, each tier **pinned by name to a shipped constant** (no magic numbers):

| Tier | Default | Constant | Verbs |
|---|---|---|---|
| Read / local | **5 s** | `AUTH_MODULE_PROBE_TIMEOUT_SECS` | `state`, `whoami`, `status`, all `*-list` / `show` / `query` (local store/disk only) |
| Write / network | **30 s** | `PENDING_TIMEOUT_SECS` | home-Node round-trips & writes: `send`, `create-*`, `invite`, `join`, `register`; node writes; `auth-module test` (5 s internal fail-fast sits under the 30 s guard) |
| Federation peer interaction | **180 s** | `FEDERATION_RELATIONSHIP_TIMEOUT_SECS` | `federation initiate` / `accept` — anything doing a cross-Node handshake |

- **Standing invariant: control-surface default ≥ the verb's own internal timeout.** Otherwise the local guard fires before the operation legitimately completes and masks success as a false `TIMEOUT`. This is *why* federation verbs cannot sit in the 30 s tier. Holds by construction because the tiers are the shipped constants.
- **`timeout_ms` driver override** (§10): **honored as-is** (no clamp-up to the tier default — the driver owns the trade-off), **floor-validated** → `BAD_ARGUMENT` on a non-positive / non-numeric value.
- **Class-derived, no per-verb table.** Tier is read off the existing READ/WRITE classification + a federation-interaction flag; **new verbs inherit their tier from their class.**

### AC-D3b — subscription-filter grammar — 🔒 LOCKED

The `subscribe` filter (§3) has three optional fields (`spaces`, `event_types`, `nodes`). Grammar:

- **Combination: AND across fields, OR within a field.** An event passes iff it matches *every present* dimension: `(space ∈ spaces ∨ spaces empty) ∧ (type matches one event_type ∨ none given) ∧ (involves a node ∈ nodes ∨ none given)`.
- **Empty == omitted == no restriction**, uniform on all three fields. `spaces:[]` means "all entitled Spaces," **not** "match nothing" — closes the silent-drop-everything footgun.
- **Wildcards (`event_types`) — exactly two forms (closed):** bare `*` (all types) and a trailing-segment prefix `state.*` (matches any type whose path starts with `state.`). **No leading or mid-pattern wildcards** (`*.text`, `state.*.foo` → `BAD_ARGUMENT`). Patterns match the canonical wire type strings (`EventType::as_str()`). *Confirm-at-design: the exact match predicate is a raw string-prefix on `as_str()` with the trailing `.` retained in the prefix (so the segment boundary is respected) — verify against the real type strings in the runbook.*
- **Entitlement is the ceiling — the filter narrows, never broadens.** Effective subscription = (what the subscriber may already see) ∩ (filter). The filter is a **view**, not an access request: a `spaces` entry for a Space the subscriber is not in (Client) / does not host (Node) is **inert** — it yields nothing and **never errors** on out-of-entitlement. Prevents the filter being an escalation vector.
- **`nodes` is Node-side only.** On the Client → `BAD_ARGUMENT` (loud, not silent — a silently-ignored filter dimension is a footgun).
- **Malformed filter** (unknown field, wrong type, illegal wildcard form) → `BAD_ARGUMENT` on the events pipe **before** streaming starts (the `subscribe` is the first message).

### AC-D3c — `state` full schema (both binaries) — 🔒 LOCKED

**Principle.** `state` is a **control verb running in-process** in the resident binary, so it composes from **live runtime state** — it sees more than the file-reading `status` op (e.g. live home-node connection state). Two guardrails: **no new instrumentation** (any §9 field not already tracked is dropped to a documented follow-up, not built — adapter scope, D-065); and **`state` answers purely locally** (read-tier, 5 s per AC-D3a — never a network round-trip).

**Client `state.data`:**
- *Confirmed available* (`whoami`/`status`/`spaces`): `lifecycle`, `identity_id`, `display_name`, `is_ai`, `home_node` (`NodeXgid`), `version`, `spaces[]`.
- *In-process live*: `home_node_connected`.
- *Control-owned* (M7 always provides): `bindings`, `event_subscriptions`.
- *Confirm-at-pickup* (keep iff already tracked, else drop): `connected_since`, per-space `member_count` / `room_count`.

**Node `state.data`:**
- *Confirmed / store-derivable*: `lifecycle`, `node_id`, `operator_display_name`, `endpoint`, `auth_tiers_served`, `federated_peers` (store len), `hosted_spaces` (store len).
- *Control-owned*: `bindings`, `event_subscriptions`.
- *Confirm-at-pickup*: `uptime_seconds`, `active_connections`, `registered_identities` (keep iff cheap, else drop).

The lock is the **principle + the confirmed/control-owned core**; the confirm-at-pickup fields resolve against the runtime in the runbook (standard D-078). No field promises data the runtime does not already hold.

### AC-D3d — control-surface error catalogue — 🔒 LOCKED

The v1 catalogue is the §8 set of 8 plus one addition (**9 total**), each mapped onto AC-D2's closed `category` set:

| Code | category |
|---|---|
| `INSTANCE_NOT_READY` | lifecycle |
| `UNKNOWN_COMMAND` | argument |
| `BAD_ARGUMENT` | argument |
| `BINDING_NOT_FOUND` | argument |
| `CONCURRENT_COMMAND_NOT_ALLOWED` | argument |
| `CONNECTION_LOST` | connection |
| `TIMEOUT` | timeout |
| `PERMISSION_DENIED` | permission |
| `MALFORMED_COMMAND` *(new)* | argument |

- **`MALFORMED_COMMAND`** covers the pre-parse failure (line is not valid JSON / has no `cmd`) that none of the §8 codes fit — `BAD_ARGUMENT` presumes a parsed command. On this error the reply **omits the echoed `cmd`/`id`** (nothing to echo).
- **Invariant: control-surface codes never use category `protocol`.** `protocol` is **verb-sourced only** (band codes / client `anyhow`). The 6 categories partition cleanly — 5 covered by control codes, `protocol` reserved for verbs.
- **No `INTERNAL` control code in v1** — unexpected faults surface as the verb's `GENERIC_4000` (category `protocol`); revisit only if a control-layer-only fault path appears.
- **Catalogue closed for v1** — like the category set, new codes are deliberate additions, not ad-hoc strings.

---

## AC-D4 — pipe-auth policy — 🔒 LOCKED (OS-ACL-delegated; no in-protocol auth in v1)

The M6-deferred "pipe-access == operator-authority" question (§2.6.1), now M7's.

**Load-bearing framing: M7 introduces no new exposure.** The write-path admin surface is *already* reachable over the unauthenticated `--batch` pipe (M6-shipped). M7's `.aicontrol` sister pipe is the **same** trust posture, not a new hole — so AC-D4 locks the v1 posture + the deferral seam; it does not re-open the M6 decision.

- **Trust model unchanged (D-082):** pipe-access == administrator; the **OS named-pipe ACL is the access control**, not an in-protocol credential. No in-protocol authentication in v1.
- **Sister pipes inherit the legacy `--batch` pipe's default ACL**, independently restrictable (the §2.2 benefit: lock down `--aicontrol` tighter than `--batch` without touching the legacy surface). **Windows pipe-ACL in v1; cross-platform deferred per §14.**
- **Per-surface audit attribution** via `ActorVia::AiControl` ("aicontrol", already in code) — audited verbs over `--aicontrol` are tagged distinctly from `batch`/`cli-direct`. Free.
- **Reserved trio = one named arc, inert in v1:** the `authorize` stage (AC-D2) + `PERMISSION_DENIED` (AC-D3d) + the per-connection token (this). All dormant; they **activate as a unit only when a privilege model exists** (per-verb gating).
- **Per-connection token = named future extension, first-message seam.** For the §12.2 cases (an MCP server running as a *different* OS user; multiple drivers sharing one Identity wanting per-driver audit). The seam: an optional auth handshake as the **first message** on the command pipe (sibling to the events pipe's first-message `subscribe`); **absent == proceed in v1**, so adding it later is backward-compatible by construction.

## AC-D5 — client `ops::*` coverage gap — 🔒 LOCKED (defer all three)

**Decision:** defer `create-dm-space`, `leave`, `members` to a future arc; **v1 `--aicontrol` exposes exactly the 14 shipped `ops::*` verbs** (consistent with AC-D1's "verb list = whatever the surface exposes"). §6 of the canonical doc is corrected in the doc-sync to mark the three deferred with a one-line pointer to the future arc.

**Future arc (the deferral's address):** *client-feature arc — DM-space creation · membership-leave · member-list op.* (Same discipline as node-policy routing enforcement to the temperature-plugin arc — a named home, not a dangling "someday.")

**Per-verb evidence (inherited by the future-arc author; no re-derivation needed):**

| Verb | Status in tree | Lift cost |
|---|---|---|
| `create-dm-space` | net-new — exists only as hand-built events in the multiparty scenario runner; no `ops`, no CLI, no construction path | feature build (DM-create event + `dm_constraints` content shape) |
| `leave` | net-new — **no membership-leave event type in use anywhere** client-side | feature build (new membership-leave construction) |
| `members` | reachable-but-not-an-op — data reconstructable via the `ai_status` event-replay → `SpaceState.members` path (role / `invited_by`) | moderate (new op reusing replay + member-list semantics) |

**Deliberate (D-065 honest-scope):** `members` is the cheap one (the data path already exists) yet is **deferred anyway** — to keep M7 a pure adapter and avoid an odd one-verb partial. Decision on record.

## AC-D6 — replay-safety — 🔒 LOCKED (defer; do-it-over in v1)

**v1 = do-it-over, no idempotency keys** — and coherent, not a silent omission. The rest of the protocol already lets a robust driver recover: strictly-serial-per-connection (§2.3) = no in-flight ambiguity; lifecycle-aware errors (AC-D2) report *what* failed + the instance state; `state` (AC-D3c) reconciles after a non-idempotent failure (the AC-D3a `TIMEOUT` guidance already routes writes to `state` reconciliation). Reads retry for free.

- **Seam (sibling to AC-D4's token):** an optional driver-supplied `idempotency_key` command field; the instance returns the original reply for a duplicate key. **Absent == do-it-over** → backward-compatible by construction.
- **Session-scoped key-memory** if added — the remembered-key set lives in the per-connection session (consistent with the connection-scoped binding namespace; a fresh connection forgets).
- **Routed to the `--aicontrol` hardening arc** (shared address with AC-D4's per-connection token), but **dependency-free**: unlike the token, idempotency keys need no privilege model, so within the hardening arc AC-D6 can land independently of AC-D4's reserved-trio activation.

---

## Cross-refs

- `tasks/M7_AICONTROL_AUDIT.md` (v1.0) — the Phase-0 drift-reconciliation this design acts on (Findings 1–5 + ordering recommendation).
- `docs/xgen_aicontrol_implementation.md` (v1.1) — the canonical M7 spec; §4/§6/§7/§8/§12 are the sections M7 locks + (for §7) rewrites under AC-D1.
- `xgen-node/src/admin_ops.rs` — shipped Node surface (`AdminCommand` + 8 subcommand enums; `AdminError`/`Stage` (6 variants)/`ActorVia::AiControl`; timeouts `AUTH_MODULE_PROBE_TIMEOUT_SECS`=5, `PENDING_TIMEOUT_SECS`=30, `FEDERATION_RELATIONSHIP_TIMEOUT_SECS`=180).
- `xgen-client/src/ops.rs` — shipped client surface (14 `ops::*` fns; `anyhow`-based errors).
- DECISIONS.md: D-066 (`--batch`/`--aicontrol` split), D-063 (library-first multi-dispatch), D-067 (`ops::*`), D-069 (open-item flagging / arc-local IDs), D-082 (administrator vs operator), D-072 / Appendix J §J.5 (XGID wire-invariance, cited in AC-D1).
- M7-standalone (live config reload) — the correct home for `config-reload` / §11.

---

*All AC-D# locked (AC-D1 · AC-D2 · AC-D3a–d · AC-D4 · AC-D5 · AC-D6). Design phase complete. Next: doc-sync (canonical doc §6/§7/§8/§9/§3/§4/§10/§11/§12 under these locks) + implementation runbook for Clair.*
