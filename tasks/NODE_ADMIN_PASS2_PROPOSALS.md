# Node Admin Write Path — M6 Pass 2 (Verb Categories + Joe-lock Proposals)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-18  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this file is

Pass 2 of M6 Phase 0. It is the **working document** for your review. It contains:

1. **Verb category sketches** — what each of the seven M6 admin-write categories would actually contain, at the granularity of verb names + one-line purpose + write/read classification + risk surface. Not yet locked schemas.

2. **Joe-lock items as proposals** — six items identified in Pass 1, each presented with options and Chat Claude's recommendation. These are *proposals*, not decisions. Lock happens in Pass 3.

3. **Discussion threads flagged inline** — questions where I genuinely don't have enough context to even propose, framed as "needs your input."

Pass 3 takes this file, locks decisions through discussion, and produces the final design doc + task file.

**Per D-069**: this file is explicitly *delegated draft*, not locked specification. Every section that lists open items flags them explicitly so we can tell what's settled from what isn't.

---

## Part A — Verb category sketches

Seven categories. Each section lists the verbs as **proposed names** with one-line purpose, classification (READ / WRITE / DESTRUCTIVE), and any unique concerns. **All names are placeholders pending the naming-convention decision (Joe-lock #6).**

Convention used in this draft: **two-token like Client side** (`federation accept`, `auth-module register`) to match the existing `ai delegate` / `identity list` pattern. Final convention is Joe-lock #6.

### Category A1 — Federation management

The largest category. The Node operator manages who this Node federates with, on what terms.

| Verb (proposed) | Class | Purpose |
|---|---|---|
| `federation list` | READ | List all current federation relationships (active + pending + revoked) |
| `federation accept` | WRITE | Accept an incoming federation request (peer_node_id + endpoint) |
| `federation reject` | WRITE | Reject an incoming federation request with reason |
| `federation initiate` | WRITE | Initiate federation with a peer Node (outbound handshake) |
| `federation defederate` | DESTRUCTIVE | Cleanly terminate federation with a peer (D-022, §3.15 cleanup flow) |
| `federation policy set` | WRITE | Set per-peer allow/deny policy (which Spaces they can join, rate limits, etc.) |
| `federation policy show` | READ | Show current per-peer policy |
| `federation signal defederation` | WRITE | Submit defederation signal to Bootstrap Nodes (§3.15 reputation surface) |

**Risk surface:** `defederate` and `signal defederation` are reputation-affecting and harder to reverse. They warrant elevated privilege (Joe-lock #1b) and must produce audit entries (Joe-lock #3).

**Open items for design phase (Pass 3 may defer or escalate):**
- Does `federation initiate` use the existing `xgen-client federate --peer` shape (which is the Client-side front of federation initiation) or is the Node side genuinely different? *I think:* Client `federate` is for a *Space owner* asking their home Node to federate a Space; Node `federation initiate` is the *Node operator* establishing a node-to-node relationship independent of any specific Space. Different actors, different intent. → Pass 3 confirms.
- Does `federation list` need pagination? Worst case is a Node federated with hundreds of peers; current Client `--batch` returns whole result text in one OK. → Pass 3 decides.
- Bootstrap signal semantics — §3.15 says defederation signals are reputation-affecting; what does a Bootstrap Node *do* with one? Pass 3 may need Joe input on whether this verb is in M6 or deferred.

### Category A2 — Auth Module management

Registering and managing pluggable Auth Modules per Ch3 (the tiered auth architecture).

| Verb (proposed) | Class | Purpose |
|---|---|---|
| `auth-module list` | READ | List registered Auth Modules + accepted Tiers + last-seen status |
| `auth-module register` | WRITE | Register a new Auth Module (URL + public key + accepted Tiers) |
| `auth-module revoke` | DESTRUCTIVE | Revoke trust in an Auth Module |
| `auth-module set-tiers` | WRITE | Change which Tiers an Auth Module is accepted for |
| `auth-module test` | READ | Send a test challenge to a registered Auth Module, report response |

**Risk surface:** `revoke` invalidates trust assertions issued by that module. Any Identity registered via the revoked module needs handling (downgrade? mark invalid? notify?). → Pass 3 needs spec lookup on §3.6 tiered auth or Joe input.

**Open items:**
- M6 protocol-level handling of "Identities authenticated through a now-revoked module" — current spec (§3.6) may not fully cover the revocation cascade. Could be a Pass 3 surface that surfaces a spec gap.
- Auth Module *health check* protocol — does `auth-module test` use a defined protocol message or is it ad-hoc? *I think:* ad-hoc is fine for M6, formalise later.

### Category A3 — Bootstrap configuration

Per §3.15, Bootstrap Nodes are the discovery layer. This category manages how this Node participates in (or operates) Bootstrap discovery.

| Verb (proposed) | Class | Purpose |
|---|---|---|
| `bootstrap show` | READ | Show this Node's current Bootstrap configuration (registered URLs, advertised auth_tiers_served, bootstrap_info) |
| `bootstrap register` | WRITE | Register this Node with a Bootstrap Node (outbound `bootstrap.register` event) |
| `bootstrap deregister` | DESTRUCTIVE | Deregister from a Bootstrap Node |
| `bootstrap set-info` | WRITE | Update `bootstrap_info` metadata (display name, description, contact, etc.) |
| `bootstrap set-tiers` | WRITE | Update advertised `auth_tiers_served` list |

**Risk surface:** Smallest category in M6. Mostly metadata updates. `deregister` is reversible (just re-register).

**Open items:**
- Does this Node *operate* as a Bootstrap Node too (i.e. accept incoming `bootstrap.register` events from other Nodes)? If yes, that's a separate sub-category (Bootstrap *operator* verbs distinct from Bootstrap *client* verbs). For M6 v1: assume this Node is a Bootstrap *client*; Bootstrap *operator* verbs deferred. → Pass 3 confirms.

### Category A4 — Space and Room operator actions

Node-operator authority over Spaces *hosted by this Node*. Distinct from member-initiated actions: a Node operator can intervene for legal/safety/operational reasons that supersede the Space's normal governance.

| Verb (proposed) | Class | Purpose |
|---|---|---|
| `space list-hosted` | READ | List Spaces hosted by this Node (operator view; same as current `spaces` but read-only-write-safe) |
| `space force-eject` | DESTRUCTIVE | Node-operator removes a member from a hosted Space (distinct from member kick) |
| `space set-node-policy` | WRITE | Set Node-level moderation policy on a hosted Space (auto-mute thresholds, etc.) |
| `space show-node-policy` | READ | Show current Node-level policy for a hosted Space |
| `space migrate-as-source` | DESTRUCTIVE | Trigger Space migration from this Node to another (§3.12) |
| `space audit-events` | READ | Query the audit log for a specific Space (filter by event type, time range) |

**Risk surface:** `force-eject` and `migrate-as-source` are protocol-visible operator actions. The Space's members see them happen. Per §3.11.8, both must produce audit entries with full provenance.

**Open items (signing-identity sub-design — Phase 9 flagged in roadmap):**
- **Who signs the resulting protocol event?** When the Node operator force-ejects a member, the resulting `membership.kick` event has to be signed by *someone*. The Space owner didn't sign it. The kicked member certainly didn't. Options:
  - **A.** Signed by the Node keypair, with a new EventType variant (`membership.node_eject`) that protocol-acknowledges Node-operator authority as distinct from member authority.
  - **B.** Signed by the Node keypair using existing `membership.kick` with a `meta_atts` field marking node-operator authority.
  - **C.** Requires a separate admin keypair (signing capability tied to operator identity, not Node identity).
  - This is the **signing-identity sub-design** flagged in Pass 1 for Phase 9 of M6. Needs Joe-lock at Pass 3 OR a defer note ("Phase 9 lands with a TODO; signing-identity sub-design happens as its own conversation").

### Category A5 — Identity registry administration

Managing the Identity records stored by this Node.

| Verb (proposed) | Class | Purpose |
|---|---|---|
| `identity show` | READ | Show full record for a single Identity (display name, registration time, Trust Assertion status, is_ai, capabilities, devices) |
| `identity revoke` | DESTRUCTIVE | Revoke a registration with audit trail; Identity can no longer authenticate on this Node |
| `identity update-trust-expiry` | WRITE | Extend or shorten the stored Trust Assertion expiry for an Identity |
| `identity manage-replica` | WRITE | Manage replica relationships (which Nodes hold copies of this Identity record) |

(`identity list` is the existing M2 read-only verb; not duplicated here.)

**Risk surface:** `revoke` is the most consequential — a revoked Identity loses access to every Space hosted on this Node. Heavy audit trail requirement.

**Open items:**
- Cascade behaviour on `identity revoke`: does the Identity's existing Space memberships get force-ejected automatically (cascade), or does revocation just block future auth (leaving stale member entries)? Spec answer unclear. → Pass 3 either resolves from §3.6 or escalates to Joe.

### Category A6 — Logging and audit administration

Phase 4 in the roadmap — moved earlier than Identity per your Pass 1 reordering so the audit primitive lands before any other write verb. This is the **landing-pad category** for the audit subsystem.

| Verb (proposed) | Class | Purpose |
|---|---|---|
| `log set-level` | WRITE | Set the runtime log level for a module (or `*` for global) without restart |
| `log show-level` | READ | Show current effective log levels |
| `audit rotate` | WRITE | Force audit log rotation now (manual; normally periodic) |
| `audit query` | READ | Query the audit log: filter by actor, verb, time range, success/failure |
| `audit export` | READ | Export filtered audit log slice to a file (the file is what the operator inspects) |

**Risk surface:** `log set-level trace` enables verbose logging that may capture sensitive data. Privilege model (Joe-lock #1) applies.

**Open items (Joe-lock #3 lives here):**
- **Audit log persistence layer.** §3.11.8 names the audit-log surface but does not lock the storage backend. Options:
  - **A. SQLite per Node** (`xgen-node_audit.db` alongside the other registries). Queryable, transactional, structured. Same primitive as the rest of the Node's storage (D-035 convention-derived).
  - **B. Append-only log file** (`logs/audit/xgen-node_audit.jsonl`). Simpler. No transaction semantics. Easier to ship to external SIEM tools.
  - **C. Both.** SQLite for in-Node `audit query`, append-only file for external export. Doubles the write cost but gives both query and stream surfaces.
  - **Chat Claude's lean:** Option A (SQLite). Reasons: (1) we already use SQLite per Node (Identity registry, Federation registry); audit is the same primitive. (2) `audit query` with filters is much cleaner against SQL than a JSONL file scan. (3) External-export use case is served by `audit export` returning a JSONL file *generated from* the SQLite source — best of both worlds.

### Category A7 — Plugin management

Managing pluggable in-process modules. Today the temperature plugin (D-061's `NoOpTemperaturePlugin`) is the only customer.

| Verb (proposed) | Class | Purpose |
|---|---|---|
| `plugin list` | READ | List loaded plugins (name, version, status, type) |
| `plugin load` | WRITE | Load a plugin by name (must already be compiled into the binary in v1) |
| `plugin configure` | WRITE | Update plugin-specific config (per-plugin keys) |
| `plugin unload` | DESTRUCTIVE | Unload a plugin (stops its event consumption) |
| `plugin status` | READ | Show a single plugin's detailed status |

**Risk surface:** Lowest of the seven categories. Plugin surface is small today.

**Open items:**
- **Phase 10 is gated on plugin spec maturity.** The plugin subsystem is currently exactly one plugin (`NoOpTemperaturePlugin`). The "load by name" / "configure" verbs make sense only if the plugin set is meaningfully extensible. If M6 ships with only the no-op temperature plugin still being the only customer, Phase 10 may legitimately collapse to just `plugin list` + `plugin status` (two reads), and the WRITE verbs land when a second plugin appears. → Pass 3 confirms scope.

---

## Part B — Joe-lock items (proposals + recommendations)

Six items. Pass 1 split #1 into 1a/1b and added #5 and #6. Listing in the order they were established.

### Joe-lock #1a — Connection authority (who can connect to the `--batch` pipe)

**The question.** Today's named pipe is unauthenticated. Anyone with OS-level access to open the pipe path can issue any verb (currently limited to the read-only set; M6 expands the consequences). What should the connection-authority model be?

**Options.**

- **A. OS-user-equals-operator (status quo).** The pipe permissions inherit from the Node process's user. Same OS user can connect; different OS user cannot. No protocol-level authentication on the pipe itself.
- **B. Token-based.** Node operator gets a token at init time; pipe connection requires presenting the token in the first line. Token rotation handled out of band.
- **C. Keypair challenge-response.** Like the WS-level client auth: pipe connection presents a public key, Node challenges with a nonce, client signs. Reuses existing crypto primitives.

**Chat Claude's recommendation:** Option A for M6 v1. The pipe is local-only by definition; OS-level access control is the right primitive on Windows. Adding pipe-level auth in M6 is yak-shaving that delays the verb work.

**However** — `--aicontrol` (M7) may need real authentication when MCP servers run as different OS users than the human. That's M7's problem and M7's design phase. For M6, Option A is fine.

**Status:** Recommended; Pass 3 confirms.

### Joe-lock #1b — Authorisation proof (what proves authorisation for a given verb)

**The question.** Once connected (per #1a), what proves the connected party is authorised for verb X specifically? This is per-verb authorisation, not connection authentication.

**Options.**

- **A. Session-scoped.** If you can connect to the pipe (per #1a), you can issue any verb. No per-verb gating.
- **B. Verb-class gating.** READ verbs require less proof than WRITE verbs; DESTRUCTIVE verbs require more. The class is part of the verb's spec.
- **C. Operator-signed envelopes.** Every WRITE/DESTRUCTIVE verb requires the operator to sign a per-verb envelope (replay-resistant nonce, etc.) using the Node operator's identity keypair. Highest assurance.

**Chat Claude's recommendation:** Option A for M6 v1, consistent with #1a's recommendation. Pipe access = authorisation. Operator-signed envelopes (Option C) are the natural shape for `--aicontrol` once MCP servers and remote-driver scenarios surface, but that's M7+.

**Caveat.** If #1a goes to Option B or C (pipe-level auth), then #1b naturally upgrades to at least Option B. The two questions are linked — locking #1a forces hand on #1b.

**Status:** Recommended; Pass 3 confirms.

### Joe-lock #2 — Live-reload field bucket

**The question.** When `config-reload` (M7 milestone) lands, which config fields are reloadable at runtime vs which require restart? This decision must be made *in M6* because some M6 verbs touch fields whose reloadability informs their schema.

**Proposed buckets.**

**Reloadable (changes apply immediately):**
- `[logging].level` — applies on next log emission
- `[ai.behavior].*` — per-plugin tuning (applies on next plugin event)
- `[node].local_mode` — affects new connections only; existing connections unaffected

**Restart-required (changes accepted into persisted config but not active until restart):**
- `[node].listen` — would require rebinding the WS listener; not safely live-reloadable
- `[paths].keypair_path` — keypair is loaded once at startup
- `[client].node` — Client-side; would require WS reconnection
- `[ai].plugin` — plugin loaded once at startup
- `[ai].is_ai` — registration identity, fixed at registration time

**Forbidden (changes rejected outright; require manual config edit + restart):**
- (none currently — every field above is in one of the two buckets)

**Chat Claude's recommendation:** The bucket above. Conservative — only fields where live application is *certainly safe* go into reloadable. Anything that touches a held resource (listener, file handle, plugin instance) requires restart.

**Status:** Recommended; Pass 3 confirms or amends.

### Joe-lock #3 — Audit trail shape + persistence layer

**The question.** Every WRITE/DESTRUCTIVE verb produces an audit entry per §3.11.8. What is the entry's schema and where is it stored?

**Schema proposal:**

```
audit_entry {
    timestamp: RFC 3339 UTC
    verb: String                       // "federation accept", "identity revoke", etc.
    actor: String (identity_id)        // who initiated (operator's keypair URI)
    actor_via: String                  // "batch" | "aicontrol" (M7+) | "cli-direct"
    target: Option<String>             // verb-specific target (peer_node_id, identity_id, space_id, etc.)
    args_hash: String                  // sha256 of canonical-JSON args (full args not stored — PII concern)
    outcome: "ok" | "error"
    error_code: Option<String>         // when outcome = "error"
    error_message: Option<String>      // when outcome = "error"
    correlation_id: Option<String>     // for chaining related entries
    meta_atts: Map<String, String>     // forward-compat
}
```

**Persistence layer proposal (per Category A6 lean):** SQLite at `<data_dir>/xgen-node_audit.db`. Same primitive as Identity and Federation registries (D-035 conventional path). The `audit_entries` table mirrors the schema above; indexes on `timestamp`, `actor`, `verb`.

**Why not append-only file:** Query is the main use case (`audit query --since 7d --actor X --verb federation\\ accept`). SQL beats file-scan for that. Export to a file (for SIEM ingestion) is the secondary use case, served by `audit export` materializing a JSONL slice from SQLite.

**Why include `args_hash` rather than full args:** Some verb args contain potentially sensitive data (target identity IDs that may later need GDPR removal). Hashing keeps the audit verifiable (you can re-hash a candidate args block and check match) without storing the data itself. Operators concerned about non-repudiation can opt into full args via a config flag — out of M6 scope.

**Status:** Recommended; Pass 3 confirms schema fields, persistence layer, and the args-hash trade-off.

### Joe-lock #4 — Verb-set finalisation

**The question.** Are the verb categories above complete? Verbs missing? Verbs that don't belong?

**Process proposal for Pass 3:**

1. Walk each category (A1–A7) verb-by-verb in Pass 3.
2. For each verb: confirm the name (after #6 naming is locked), confirm class (READ/WRITE/DESTRUCTIVE), spot-check args schema concerns, flag any deferral.
3. Result: §7 of `docs/xgen_aicontrol_implementation.md` updates from "sketched category" to "locked verb list" for each section.

**Chat Claude's note:** Total verb count in the sketches above is **~35 verbs**. That's larger than the 30+ estimated in CLAUDE.md but within the same order of magnitude. Pass 3 may collapse some (e.g. merging `policy set` + `policy show` into a single `policy [show|set]` subcommand-with-args). Final count likely 25–35 after Pass 3 trimming.

**Status:** Process is the deliverable; verb list itself locks in Pass 3.

### Joe-lock #5 — Failure semantics

**The question.** What happens when a write verb fails partway through? Example: `federation accept` succeeds at handshake stage but fails at registry-persist stage. Is the partial state left in place? Rolled back? Reported with an explicit recovery verb?

**Options.**

- **A. Best-effort with honest reporting.** Partial state is left in place; the verb returns `ERROR: <stage-where-failed>` with enough detail that the operator can either retry or invoke a recovery verb. No protocol-level transactions.
- **B. Two-phase commit.** Every write verb is internally phase-1 (validate + reserve) then phase-2 (commit). Failure in phase-2 triggers automatic rollback of phase-1 reservations.
- **C. Mixed.** Cheap-to-rollback operations get full transactions; expensive/external-side-effect operations get best-effort + recovery verbs.

**Chat Claude's recommendation:** Option A for M6 v1, with Option C as the future direction. Reasons:

- Many M6 verbs have *external* side effects (federation handshake produces remote-state changes; bootstrap.register produces remote-state changes) that can't be rolled back even if we wanted to.
- Two-phase commit (Option B) is a major engineering surface that delays M6 substantially without a clear customer asking for it.
- Honest behaviour over polite behaviour (D-065 named principle): if the operation failed midway, *say so*; don't pretend it succeeded or fail silently. The operator decides recovery.

**Implication for verb design:** Each verb's error response includes a `stage` field indicating where it failed (`validate`, `register`, `persist`, `notify`, etc.). The operator can then issue category-specific recovery verbs.

**Status:** Recommended; Pass 3 confirms.

### Joe-lock #6 — Verb naming convention

**The question.** `federation-add` vs `federation add` vs `federation.add`. Pick one and apply uniformly across all ~30 verbs.

**Options.**

- **A. Two-token.** `federation accept`, `auth-module register`, `bootstrap set-info`. Matches Client convention (`ai delegate`, `identity list`). Reads naturally. Two tokens = category + action.
- **B. Hyphenated single-token.** `federation-accept`, `auth-module-register`, `bootstrap-set-info`. Matches the sketches in `docs/xgen_aicontrol_implementation.md` §7. Single token = whole verb. Slightly easier to grep.
- **C. Dotted.** `federation.accept`, `auth_module.register`. Matches Event-type naming (`state.federation_add`). Doesn't match any existing CLI subcommand convention.

**Chat Claude's recommendation:** **Option A (two-token)**. Reasons:

1. Matches the existing Client convention — `ai delegate`, `identity list`. Adopting B or C would mean the two binaries' CLI surfaces use different conventions.
2. Reads naturally in CLI invocations: `xgen-node federation accept --peer ...` reads better than `xgen-node federation-accept --peer ...`.
3. Clap's subcommand-grouping is exactly the right primitive: each category becomes a `Subcommand` enum variant with its own nested subcommands. The code structure mirrors the user-facing structure.
4. The `docs/xgen_aicontrol_implementation.md` §7 sketches were Chat Claude's earlier proposal; aligning them to convention A is a small edit, not a design regression.

**Implication if you pick A:** The `--aicontrol` JSONL command shape uses `"cmd": "federation accept"` with a space (or we underscore-substitute: `"cmd": "federation_accept"`) — this is an M7 question, not M6. M6 only deals with the `--batch` plain-text shape, where the space-separated form is natural.

**Status:** Recommended; Pass 3 confirms.

---

## Part C — Discussion threads needing your input

Three threads where I genuinely don't have enough context to propose. Please answer in Pass 3 (or now if you prefer):

### Thread 1 — Phase ordering inside M6

Pass 1 locked the phase order as:

```
Phase 2 — admin_ops::* scaffolding
Phase 3 — Read-only completions
Phase 4 — Logging/audit admin   (audit primitive lands here)
Phase 5 — Identity registry admin
Phase 6 — Federation management
Phase 7 — Bootstrap configuration
Phase 8 — Auth Module management
Phase 9 — Space/Room operator actions (signing-identity sub-design first)
Phase 10 — Plugin management
```

After enumerating the verbs above, **two specific orderings might be worth reconsidering**:

- **Phase 7 (Bootstrap) before Phase 6 (Federation)?** Bootstrap is the smaller category (5 verbs vs ~8); federation depends on Bootstrap for the discovery half. Phase 6→7 has them in the wrong dependency order. Swap → 7 before 6.
- **Phase 8 (Auth Module) deferred to post-M6?** Auth Module revocation cascade is a spec-gap concern (Category A2 open items). If Pass 3 surfaces that §3.6 doesn't fully cover revocation cascade, Auth Module management might legitimately defer until the spec covers it. Pass 3 decides.

### Thread 2 — Scope question: are there *other* categories I'm missing?

The seven categories come from `docs/xgen_aicontrol_implementation.md` §7. Reviewing them again, two possible omissions:

- **Connection management.** Disconnect a specific client connection (force WS close), set per-connection rate limits, ban a connection by IP/identity. None of these are in §7. Should they be?
- **DAG / Space-storage administration.** Compact event store, vacuum SQLite, force re-replay of a Space's DAG, repair a corrupted Space. Operational tools the operator might want. Not in §7.

**Chat Claude's lean:** Defer both. Connection management can wait until M6 has shipped and we see what's actually needed. DAG/storage admin is harder design (correctness of force-re-replay is non-trivial) and the operational pain hasn't surfaced yet.

### Thread 3 — How verbose should error messages be?

Today's pipe protocol replies `OK\n` or `ERROR: <message>\n`. With M6 expanding the verb surface, error messages will get more varied. Question for Pass 3:

- **A. Free-form error strings.** Whatever the verb's implementation produces. Easy. Hard to parse programmatically.
- **B. Structured `ERROR <code>: <message>\n`** — e.g. `ERROR FED_3041: peer rejected handshake; capabilities mismatch\n`. Still plain text, still human-readable, but the leading code is parsable.
- **C. Numeric only.** `ERROR 4001\n` plus a separate verb `error-info 4001` returns the human text. Most parsable, least human-readable.

Note: this is **M6's plain-text shape**. The `--aicontrol` (M7) JSONL shape will have structured errors with `code`/`category`/`message` (per `docs/xgen_aicontrol_implementation.md` §4.3). Question is just M6's shape.

**Chat Claude's lean:** Option B. Adds a small amount of programmatic parsability without compromising human readability.

---

## Status disposition

`ACTIVE` because:

1. The proposals are working content for Pass 3 discussion.
2. Pass 3 either confirms each recommendation (flip to `COMPLETED`) or revises and proceeds to the design doc.
3. After Pass 3, this file becomes `DEPRECATED` with the locked design doc named as the replacement.

The file is here to be marked up. Pass 3's first action is to walk through it section by section.

---

## Pre-Pass 3 reading order (Joe)

If you want to prepare for Pass 3 ahead of the next session, recommended reading:

1. This file, top to bottom.
2. `tasks/CLIENT_BATCH_AUDIT_M6.md` (Pass 1 — short refresher).
3. `docs/xgen_aicontrol_implementation.md` §7.2–§7.8 (the original category sketches I'm working from).
4. `DECISIONS.md` D-066 (the architectural commitment) and D-069 (the discipline this Pass operates under).

No source-code reading needed at this stage — the verb categories don't depend on implementation specifics yet.

---

*End of Pass 2.*
