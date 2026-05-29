# XGen Node Admin Operations Design (M6)
> **Status**: ACTIVE  
> Version: 1.14  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools. This document was produced during Pass 3 of M6 Phase 0 — the Joe-locked design phase that D-069 requires before an implementing milestone is declared ACTIVE.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this document is — and what it is not

This document specifies the **Node admin write path** for the XGen reference Node implementation. It is the canonical design for the `admin_ops::*` library layer in `xgen-node-lib`, the verb set the layer exposes, the audit and accept-signal infrastructure that supports it, and the M6 milestone that ships it.

It is the canonical document for M6 (new) per the D-069 canonical-document rule. Future edits to the M6 design land here, not in `tasks/` addenda or in DECISIONS.md notes.

**This document is NOT part of the XGen Protocol.** The protocol-level changes it does specify — `TransportMessage::EventAccepted` (§3 of this doc, §3.3 of Ch3 when added) — are noted explicitly as protocol additions and will land in Chapter 3 in the implementing commit. The `admin_ops::*` layer, the audit subsystem, the per-binary verb set, the M6 phase plan — all of these are reference-implementation specification, not protocol. They belong alongside Chapter 4 (Implementation) and the `--aicontrol` document, not alongside Chapter 3 (Specification).

The architectural commitment to fill in the Node admin write path is the M6 (new) milestone block recorded in CLAUDE.md. The discipline rules governing how this document evolves are recorded in DECISIONS.md D-069. The canonical-document rule (one authoritative document per major implementation surface) is recorded in D-069 §"Canonical-document rule." This document is that one.

---

## 1. Background

### 1.1 The asymmetry M6 closes

Today, `xgen-client` exposes 15 user-facing verbs (`whoami`, `status`, `spaces`, `register`, `create-space`, `create-room`, `invite`, `join`, `send`, `history`, `version`, `init`, `ai delegate`, `ai revoke`, `ai status`), all of which route through `xgen-client-lib::ops::*` per D-067, all of which are dispatchable via `--batch`.

`xgen-node` exposes 7 read-only verbs over `--batch` (`status`, `connections`, `peers`, `spaces`, `version`, `whoami`, `identity list`). Every other invocation is rejected at `pipe::dispatch_line` with an explicit error. **The Node has no write path through `--batch`.**

For operational reality, this is a gap. A Node administrator wanting to accept a federation request, revoke an Identity, register with a Bootstrap Node, or rotate an Auth Module today has no protocol-level surface; the only paths are direct database editing or restarting with manually edited config files. Neither is appropriate.

M6 (new) closes the asymmetry. It adds the Node admin write path — a library layer (`xgen-node-lib::admin_ops::*`) symmetric to `xgen-client-lib::ops::*`, dispatched through the existing `xgen-node --batch` pipe surface, covering seven verb categories.

### 1.2 What M6 is not

M6 ships the admin write path through `--batch`. M6 does NOT ship `--aicontrol` on the Node side; that is M7's job (per D-066, canonical document `docs/xgen_aicontrol_implementation.md`). When M7 lands the Node `--aicontrol` surface, it will reuse `admin_ops::*` as its command-implementation layer, the same way the Client `--aicontrol` will reuse `ops::*`. M6 builds the layer; M7 adds the surface above it.

M6 also does NOT ship live config reload. That was originally in the Pass 1 phase list and was pulled out at the same time M6 was scoped — live reload is now M7's standalone item, addressable in concert with `--aicontrol` infrastructure.

### 1.3 Phase 0 design provenance

This document is the output of M6 Phase 0 (design phase). The phase ran in three passes:

- **Pass 1** — `tasks/CLIENT_BATCH_AUDIT_M6.md`. Snapshot audit of the Client `--batch` surface; established that the Client side is post-M5 feature-complete and identified three Appendix-F-documented Client commands missing from code (`rooms`, `members`, `federate`).
- **Pass 2** — `tasks/NODE_ADMIN_PASS2_PROPOSALS.md`. Verb category sketches across seven categories (~35 verbs total) plus six Joe-lock items as proposals plus three discussion threads.
- **Pass 3** — this document. Lock decisions across all twelve framework items (6 Joe-locks + 3 threads + 3 accept-signal sub-questions surfaced by Clair's J-080 carry-over). Verb-by-verb walks are Block 4, deferred to a subsequent session and recorded in §6 below.

Pass 2 is marked DEPRECATED by this document. Its content is superseded; it remains in `tasks/` as historical predecessor per D-069 §"Canonical home" pattern.

---

## 2. Architectural principles locked in Pass 3

Six architectural decisions inform every verb design in M6. They are listed once here and referenced by name throughout the rest of the document.

### 2.1 The accept signal is a first-class protocol primitive

**The principle.** When a write verb causes a protocol event to be emitted into XGen's DAG, the originator receives a positive acceptance signal from their home Node — symmetric to the existing rejection signal (`TransportMessage::Error`).

**Joe's framing (recorded verbatim).** *"Acceptance and rejection are two events of equal importance, just opposite direction."*

**This is a named architectural principle.** It is sibling to D-065's "honest behaviour over polite behaviour." Wherever the protocol gives the originator one direction (acceptance OR rejection), it should give them the other. The asymmetry that existed before M6 — `Error` exists, no acceptance signal — was a structural-by-accident, not by-design property; M6 closes it.

**Shape:** `TransportMessage::EventAccepted { event_id, accepted_at }`. New wire message, symmetric with the existing `TransportMessage::Error`. Adds a single new variant to the transport message enum; adds one entry to Ch3 §3.3 and one entry to Appendix I.

**Semantic (G2 — Validated and persisted):**

> A Node MUST send `EventAccepted` to the originator after the event has been successfully validated AND written to the Node's event store. A Node MUST NOT send `EventAccepted` before the write is durably committed. After the originator receives `EventAccepted`, they may claim the event is in the Node's authoritative DAG store.
>
> `EventAccepted` is a two-party synchronous signal between the originator and their home Node. It makes no claim about fan-out to other local members, propagation to federation peers, or eventual receipt by any party other than the home Node itself. Those are downstream concerns of the Node's normal propagation machinery.

**Joe's framing of C2 vs alternatives (recorded verbatim).** *"The accept signal's importance warrants its own wire shape, not a side effect of an unrelated mechanism."* This is why C1 (server-side self-fanout) and C3 (DAG-layer ack EventType) were rejected: neither treats the accept signal as a first-class concern.

**Propagation reliability dependency.** `EventAccepted` G2 makes the claim "this event is in the home Node's authoritative store." This claim is meaningful only if the system reliably propagates DAG-resident events to other members and federation peers. Verification of the propagation reliability mechanism is the subject of a dedicated audit milestone (see §5.3).

### 2.2 Honest behaviour over polite behaviour (D-065)

This is the protocol-level principle named in D-065 and operating in §2.1 above. M6 verb design surfaces it in several places: error-stage reporting (§2.6), failure semantics (best-effort over two-phase commit, §2.6), and `EventAccepted` itself (refusing to ack speculatively).

### 2.3 Single source of truth for command implementations (D-067)

`xgen-node-lib::admin_ops::*` is the canonical implementation of every Node admin verb, mirroring the `xgen-client-lib::ops::*` layer for Client commands. The `--batch` pipe dispatcher in `xgen-node/src/pipe.rs` calls into `admin_ops::*`; the future Node `--aicontrol` arm (M7) will call the same `admin_ops::*` functions. No parallel implementations.

The shape per verb mirrors `ops::*`:

```rust
pub async fn <verb>(
    ctx: &mut AdminContext<'_>,
    args: <Verb>Args,
) -> Result<<Verb>Result, AdminError>
```

where `<Verb>Result` is a pure-data struct (no I/O), and `<Verb>Args` is the clap-parsed input. Dispatchers format the result for their own output channel; `admin_ops::*` itself emits no stdout, no logs, no pipe writes.

### 2.4 CLI flag precedence over config (D-068)

Any setting in M6 verbs that has both a flag form and a config equivalent follows the locked precedence: CLI flag wins over config wins over default. No exceptions. The `xgen-common::precedence::resolve_*` helpers shipped in J-079 are the implementation primitives.

### 2.5 Convention-derived paths (D-035)

Audit storage, registry files, log paths, and per-instance segregation follow the D-035 convention: paths are derived from working directory by convention, not specified in config. The audit database (§2.6.4) lands at `<data_dir>/xgen-node_audit.db`, consistent with how Identity Registry and Federation Registry are already stored.

### 2.6 Verb-result and audit-entry schemas (Pass 3 locks)

The six Joe-lock items closed in Pass 3 produced these schemas and rules:

**2.6.1 Connection authority — Joe-lock #1a.** OS-user-equals-administrator (the administrator is the `--batch` runtime principal — distinct from the AI-operator role and from the infrastructure "Node operator" / data-controller sense; see D-082). The `xgen-node --batch` pipe inherits OS-level access control from the Node process's user. No protocol-level authentication on the pipe itself. M7 may revisit this when remote-driver scenarios (MCP servers running as different OS users) actually surface.

**2.6.2 Authorisation proof — Joe-lock #1b.** Session-scoped. Any connection that can open the pipe (per §2.6.1) can issue any verb. No per-verb gating. Tightly coupled to §2.6.1; if §2.6.1 ever upgrades to token-based or keypair-challenge auth, §2.6.2 naturally upgrades with it.

**2.6.3 Live-reload field bucket — Joe-lock #2.** Conservative bucketing locked in Pass 3:

| Bucket | Fields |
|---|---|
| **Reloadable** (changes apply immediately) | `[logging].level`; `[ai.behavior].*`; `[node].local_mode` |
| **Restart-required** (accepted into persisted config but not active until restart) | `[node].listen`; `[paths].keypair_path`; `[client].node` (Client only); `[ai].plugin`; `[ai].is_ai` |
| **Forbidden** (changes rejected outright; require manual config edit + restart) | (none currently) |

The live-reload *mechanism* lives in M7 (standalone live-reload milestone). M6 defines the buckets so M6 verbs that touch these fields know which behaviour to plan for.

**2.6.4 Audit trail shape and storage — Joe-lock #3.**

Storage: SQLite at `<data_dir>/xgen-node_audit.db`, consistent with the Identity and Federation registries (D-035 convention). One table, `audit_entries`, with indexes on `timestamp`, `actor`, `verb`.

Schema:

```
audit_entries {
    timestamp        : RFC 3339 UTC (TEXT)
    verb             : TEXT          -- e.g. "federation accept", "identity revoke"
    actor            : TEXT          -- identity_id URI of the initiating administrator
    actor_via        : TEXT          -- "batch" | "aicontrol" (M7+) | "cli-direct"
    target           : TEXT NULL     -- verb-specific (peer_node_id, identity_id, etc.)
    args_hash        : TEXT          -- sha256 of canonical-JSON args
    outcome          : TEXT          -- "ok" | "error"
    error_code       : TEXT NULL     -- e.g. "FED_3041"
    error_message    : TEXT NULL
    correlation_id   : TEXT NULL     -- for chaining related entries
    meta_atts        : TEXT          -- JSON map for forward-compat
}
```

**Why `args_hash` rather than full args.** Some verb args contain potentially sensitive data (target identity IDs that may later need GDPR removal, etc.). Hashing keeps the audit verifiable (you can re-hash a candidate args block and check match) without storing the data itself. Administrators concerned about strict non-repudiation can opt into full-args storage via a future config flag; that opt-in is out of M6 scope.

**Export.** The `audit export` verb (§6.A6) materializes a JSONL slice from the SQLite source for SIEM ingestion. Query (`audit query`) reads directly from SQLite.

**2.6.5 Failure semantics — Joe-lock #5.** Best-effort with honest reporting. Partial state is left in place on mid-verb failure; the verb returns a structured error indicating the stage where it failed. The administrator decides recovery via category-specific verbs.

Every error response includes a `stage` field. The canonical stage vocabulary:

- `validate` — input validation failed (malformed args, missing required field)
- `authorize` — privilege/authorisation check failed (M6 v1: always passes per §2.6.2; reserved for M7+)
- `register` — registry/store lookup or write failed
- `persist` — durable persistence to disk failed
- `notify` — downstream notification (fan-out, federation push) failed
- `federate` — federation peer interaction failed

A verb may fail at multiple stages over its lifetime; the error reports the *first* stage where it failed, not all stages it attempted.

This decision is informed by §2.1: the G2 accept signal is the load-bearing primitive that makes "succeeded vs failed" reliably observable for protocol-event-emitting verbs. Without G2, failure-semantics options A/B/C all operated against an absent primitive.

**2.6.6 Verb naming convention — Joe-lock #6.** Two-token, e.g. `federation accept`, `auth-module register`, `bootstrap set-info`. Matches existing Client convention (`ai delegate`, `identity list`). Clap subcommand-grouping is the implementing primitive: each category becomes a `Subcommand` enum variant with its own nested subcommands.

**Implication for `--aicontrol` (M7).** The JSONL command shape for Node `--aicontrol` will be `{"cmd": "federation accept", ...}` (space-separated) or `{"cmd": "federation_accept", ...}` (underscore-substituted) — the canonical form is space-separated; underscore is a transport-friendly alias if needed. That is an M7 decision, not M6.

**Implication for §7 of `docs/xgen_aicontrol_implementation.md`.** The §7 sketches were drafted with hyphenated single-token verb names (`federation-add` etc.). Those names will be aligned to two-token form when M6 ships, as a documentation-only edit. The verbs themselves do not change; only their CLI representation.

### 2.7 Error format on `--batch` plain-text replies — Thread 3

The Node `--batch` pipe protocol reply shape for M6:

- Success: `OK\n` (unchanged from M2)
- Failure: `ERROR: <CODE>: <message>\n` where `<CODE>` is a structured error code

**Spelling note (implementation alignment, J-154).** The Node pipe wraps every batch error in the M2 `ERROR: <body>\n` form (established pre-M6, used uniformly by the read-only verbs). M6 admin verbs supply the body `<CODE>: <message>`, so the reply reads `ERROR: <CODE>: <message>` — one colon after `ERROR`, consistent with every other error on the pipe. The plain-text spelling is deliberately **non-canonical**: the authoritative structured form is the `--aicontrol` JSON (M7) below (`{"error": {"code", "message"}}`). What is load-bearing here is the per-category `<CODE>` being present in the reply; its exact plain-text framing matches the M2 wrapper rather than introducing a second error spelling on the same channel.

Error codes follow a per-category namespace:

| Prefix | Category |
|---|---|
| `FED_*` | Federation management |
| `AUTH_*` | Auth Module management |
| `BOOT_*` | Bootstrap configuration |
| `SPACE_*` | Space/Room admin actions |
| `IDENT_*` | Identity registry administration |
| `LOG_*`, `AUDIT_*` | Logging and audit administration |
| `PLUGIN_*` | Plugin management |
| `GENERIC_*` | Verb-agnostic (bad args, internal error, etc.) |

Specific error codes are enumerated per verb in §6. **Harmonised numeric bands (Block 4):** `AUTH_2xxx`, `FED_3xxx`, `GENERIC_4000`, `AUDIT_5xxx` + `LOG_51xx`, `IDENT_6xxx`, `BOOT_7xxx`, `SPACE_8xxx`, `PLUGIN_9xxx`. Each prefix owns a distinct band; `GENERIC_4000` is the single cross-cutting code. The `--aicontrol` JSONL shape (M7) will carry the same codes in structured form (`{"error": {"code": "FED_3041", "message": "..."}}`), so the codes defined in M6 propagate forward without renaming.

---

## 3. The `EventAccepted` protocol addition

This is the only protocol-level change M6 ships. Everything else is reference-implementation surface.

### 3.1 Wire shape

**Envelope-level correlation primitive.** The locked design (J-081 §6.5, promoted to DECISIONS.md D-070 second half) puts `event_id` at the **`TransportMessage` envelope level**, not on individual variant bodies. The envelope is the base of the transport-message hierarchy; every variant inherits the field. The exact structural realisation in Rust (a wrapping `TransportEnvelope { event_id, body: TransportBody }` struct, or a serde-flattened field on every variant, or a tagged-union pattern) is Clair's latitude at implementation time per the M6 *cleaner is better* principle. The wire-format intent is what matters here: any `TransportMessage` that pertains to a specific protocol event carries the event's identifier at envelope level so the originator can correlate.

In pseudo-Rust:

```rust
// Envelope (conceptual):
pub struct TransportMessage {
    pub event_id: Option<String>,   // populated when message pertains to a specific event
    pub body: TransportBody,
}

pub enum TransportBody {
    // ... existing variants ...
    Error { reason: String, /* ... */ },                  // existing; envelope event_id now populated when rejection pertains to an event
    EventAccepted { accepted_at: String },                 // NEW in M6
    Goodbye { reason: String },
    // ...
}
```

Field semantics:

- Envelope `event_id` — the hash URI of the event this message pertains to (the same `event_id` the originator sees on the `Event` they sent). `None` for transport messages that do not pertain to a specific event (`Goodbye`, transport-level errors not tied to an event, etc.).
- `EventAccepted::accepted_at` — RFC 3339 UTC timestamp of acceptance. Recorded for trace/audit purposes; not used for any timing-sensitive logic.
- `Error::reason` — unchanged from today.

**Why envelope-level rather than per-variant.** Three reasons, all from J-081 §6.5 and D-070's reasoning:

1. The correlation primitive is identical across `EventAccepted` and `Error` paths (both signal outcomes about a specific event). Duplicating the field on both variants would be drift surface; one envelope-level field is the structural fix.
2. Today's `Error` wire shape lacks `event_id` entirely. The fix isn't to add `event_id` to `Error::body`; it's to make the correlation field available to every variant that needs it. Future variants (e.g. a hypothetical `EventDeferred` for HeldPending acknowledgement) inherit the field automatically.
3. D-070's principle requires that acceptance and rejection signals have *equal first-class status, same correlation surface*. Per-variant `event_id` fields would meet (1) (existence) but not (2) (same correlation surface) cleanly. Envelope-level meets both.

### 3.2 When the Node MUST send it

After the inbound event clears the full validation pipeline (Ch3 §3.7) AND has been successfully written to the Node's event store, AND BEFORE local fan-out (`apply_fanout`) begins.

This boundary is the G2 semantic: validated, persisted, ack sent, then async propagation proceeds. Sending `EventAccepted` before persistence is forbidden by the spec; sending it after fan-out completes is permitted but discouraged (latency cost without semantic gain).

The envelope `event_id` (per §3.1) MUST be populated with the accepted event's hash URI on every `EventAccepted` emission.

### 3.3 When the Node MUST NOT send it

- If validation fails — `TransportMessage::Error` is sent instead, citing the failure stage. The envelope `event_id` on `Error` is populated with the rejected event's hash URI so the originator can correlate.
- If persistence fails — `TransportMessage::Error` is sent, citing `persist` stage. `EventAccepted` is never speculatively sent then retracted. Envelope `event_id` populated on the `Error` message.
- If the inbound message is not an event (e.g. a `TransportMessage::Goodbye`). The accept signal applies only to DAG-event submissions. Transport-level errors not tied to a specific event (e.g. malformed framing on the WebSocket itself) leave the envelope `event_id` as `None`.

### 3.4 Originator-facing semantics

When an originator receives `EventAccepted` with envelope `event_id == E.event_id` (where `E` is the event they submitted), they may claim:

- ✅ The event is in the home Node's authoritative DAG store.
- ✅ The home Node has validated the event (signature, prev_events, state machine).
- ✅ Any subsequent client connecting to this Node and asking for this Space's DAG will see the event.

They may NOT claim:

- ❌ Other members have received the event (fan-out runs after ack; offline members catch up via sync).
- ❌ Federation peers know about the event (federation propagation is asynchronous and proceeds at its own pace).
- ❌ The event is in any DAG other than the home Node's.

The envelope `event_id` is the load-bearing primitive for this correlation: a Client with multiple in-flight event submissions matches each incoming `EventAccepted` or `Error` to its originating event by envelope `event_id` value. Without the envelope field, the originator would have to infer correlation from message ordering or timing — the failure mode J-081 §5 surfaced and D-070's second half closes.

### 3.5 Asymmetry with `TransportMessage::Error`

`Error` is sent only to the originator's connection. `Error` does not propagate to other members (a rejection means the event never entered the DAG — there is nothing to fan out).

`EventAccepted` is also sent only to the originator's connection. `EventAccepted` does not propagate either — but the accepted event itself does, via the normal fan-out + federation machinery (§4).

The two signals are wire-level symmetric (both transport messages, both originator-only). The asymmetry is in what happens *after*: rejection has no downstream work; acceptance triggers fan-out + federation. This is correct: rejection has only one stakeholder (the originator); acceptance has many.

### 3.6 Backward compatibility

Pre-M6 Clients that don't recognise `EventAccepted` will ignore it (the existing match arms default to no-op for unrecognised transport messages). This means a pre-M6 Client talking to an M6 Node continues to work — it just doesn't benefit from the accept signal. There is no break in interop.

Post-M6 Clients talking to pre-M6 Nodes will silently never receive `EventAccepted` (the pre-M6 Node doesn't emit it). The post-M6 Client must handle this gracefully — typically by treating the absence of `EventAccepted` *and* the absence of `Error` within a bounded timeout as "Node version-mismatch fallback; assume optimistic" with an explicit log line. This is a known M6/pre-M6 transitional limitation, not a permanent behaviour.

---

## 4. Propagation lifecycle of an accepted event

This section describes how an accepted event reaches the rest of the system. It is non-normative for M6 — the propagation machinery is unchanged by M6 — but is included here because §3 makes claims that depend on this machinery being reliable.

### 4.1 Two-stage model

```
[1] Originator submits Event over WS to home Node
[2] Home Node runs 13-step validation pipeline (Ch3 §3.7)
[3] Home Node writes Event to local event store
[4] Home Node sends TransportMessage::EventAccepted → originator   ← G2 boundary
        ╔═════════════════════════════════════════════════════════════════╗
        ║   Stages 5+ are asynchronous from the originator's perspective. ║
        ║   The originator's G2 claim ("event is in home Node's store")   ║
        ║   is true and stable from this point.                           ║
        ╚═════════════════════════════════════════════════════════════════╝
[5] Home Node fans out to other locally-connected members (apply_fanout)
[6] Home Node propagates to federated peer Nodes
[7] Federated peers ingest into their own DAGs and fan out to their members
[8] Disconnected clients catch up on next sync_request
```

### 4.2 Stages M6 does NOT modify

Stages 5 and 6 use existing machinery:

- **Local fan-out (Stage 5):** `xgen-node-lib::apply_fanout` iterates `ClientSenders` for the Space and pushes the event to currently-connected recipients via non-blocking `tx.try_send(...)`. Author exclusion is preserved (per Clair's J-080 finding: duplicate-avoidance UX, not protocol-correctness). Offline members are *not* delivered to in real time; they catch up via Stage 8.
- **Federation propagation (Stage 6):** Specified in `docs/xgen_federation_propagation_design.md` (Status: ACTIVE, v1.0). That document is the canonical design for federation event push (F-1), the persistent peer session model (F-2), the receive-side validation gate (F-3 + F-4), pairwise propagation (F-5), the `sync_complete` wire shape (F-6), pagination (F-7), and DAG-hole semantics (F-10). **Implementation shipped** across Phases 1-7 of the Federation Event Propagation completion milestone (J-082 through J-088, May 2026). Federation push, the F-2 long-lived session model, F-3 + F-4 + F-10 validation gate, F-5 anti-transitivity guard, F-6 sync_complete + F-7 pagination, and the F-1c reconnect scheduler are all operational. The M6 admin verbs that govern *federation relationships* (federation accept / reject / defederate, per category 6.A1) operate against the now-load-bearing relationship records: `SpaceState.federation_nodes` (per-Space federation membership, source-of-truth for F-3) and `FederationRegistry` (per-peer protocol-level + operational state, including F-1c records). See JOURNAL J-082..J-088 for the per-phase shipped-state record.
- **Sync catch-up (Stage 8):** Clients reconnecting after a gap call `sync_request` for the Space, comparing DAG tips with the Node. The Node serves missing events from its store.

### 4.3 Propagation reliability — audit milestone

Stages 5 and 8 are reasonably well-understood from existing code and tests. Stage 6 (Node-to-Node federation propagation) is the layer whose reliability mechanism has not been verified end-to-end at the design level.

Specific open questions for the audit milestone:

1. **Federation send buffering.** Does the Node buffer outbound federation events across WS reconnects, or are events emitted during disconnect lost from the federation path?
2. **DAG-tip reconciliation.** Is there automated DAG-tip reconciliation between federated peers (sync_request-style at the Node-to-Node layer), or does federation rely purely on real-time push?
3. **Recovery from gap.** If a peer Node's DAG ends up missing events (whatever the cause), what mechanism brings it back into sync?

These questions are the subject of a dedicated audit milestone (see §5.3). M6 (new) does NOT go ACTIVE until that audit closes with these questions answered (or with the gaps explicitly tracked as separate deliverables).

This is the "aggregate audit" approach Joe locked in Pass 3 — *"after all work pack we will look on those events at once"* — rather than gating individual M6 phases on individual verifications.

---

## 5. The M6 phase plan

### 5.1 Phase order (post-Block 3 lock)

```
Phase 0 — Design (Pass 1 ✅ Pass 2 ✅ Pass 3 ✅ [this document] + Block 4 ✅)
Phase 1 — Client gap patches (R1 `rooms` ✅ J-152; `members` deferred; `federate` → Phase 7)
Phase 2 — admin_ops::* scaffolding + TransportMessage::EventAccepted shape ✅ J-153
Phase 3 — Read-only completions on existing --batch  [COLLAPSED — see note below]
Phase 4 — Logging/audit admin (audit primitive lands here)
Phase 5 — Identity registry admin
Phase 6 — Bootstrap configuration  [DEFERRED — backing absent; → bootstrap-client arc]
Phase 7 — Federation management  [HONEST-SUBSET — `list` + `defederate` shipped ✅ J-1xx; 5 verbs → federation-admin-control arc]
Phase 8 — Auth Module management  [DEFERRED — registry absent; → auth-module-registry arc]
Phase 9 — Space/Room admin actions  (A4-D1 Option A locked; detailed signing sub-design opens Phase 9)
Phase 10 — Plugin management  (A7-D1: 2 reads in M6; WRITE verbs deferred)
```

**Backing-map audit (2026-05-29) — phase plan re-scoped.** Implementation (Phases 5–7) revealed that Block 4 designed several verbs against subsystems that do not yet exist in code. A read-only audit across all seven categories (`tasks/M6_BACKING_AUDIT.md`, the D-071 discipline applied reflexively to M6's own write path) mapped every verb to its real backing. Result: **M6 ships ~15 verbs** (the subsystems that exist); **18 verbs route to four post-M6 D-071 subsystem arcs** (each its own audit→design→impl arc, not a verb phase). Per-category outcome:

| Category | Phase | Outcome |
|---|---|---|
| A6 Logging/audit | 4 | **SHIPPED ✅** (J-154) — built its own backing (SQLite `audit_entries` + reload handle) |
| A5 Identity | 5 | **SHIPPED ✅** — fully backed (`IdentityRegistry` + `replication`) |
| A1 Federation | 7 | **HONEST-SUBSET** — `list` + `defederate` ship (backed by `FederationRegistry`); `accept`/`reject`/`set-policy`/`show-policy`/`initiate` → *federation-admin-control* arc (no approval queue / policy store exists) |
| A4 Space/Room | 9 | **SPLIT** — `list-hosted` + `audit-events` ship (backed reads); `force-eject` → A4-D1 wire sub-design session; 2 node-policy verbs → *node-policy* arc (folds into the force-eject session) |
| A7 Plugin | 10 | **SHIPPED ✅** (as scoped, A7-D1) — 2 reads backed |
| A3 Bootstrap | 6 | **DEFERRED** — all 5 → *bootstrap-client* arc (`bootstrap/client.rs` placeholder; no `[bootstrap]` config/store) |
| A2 Auth Module | 8 | **DEFERRED** — all 5 → *auth-module-registry* arc (no Auth Module registry exists; tier-claim types ≠ a registry of trusted modules) |

The four named arcs: *federation-admin-control* (`tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md`), *bootstrap-client* (`tasks/M6_BOOTSTRAP_CLIENT_DESIGN.md`), *auth-module-registry* (`tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md`), *node-policy* (folded into the A4-D1 force-eject session). The deferred verbs stay **specified** in §6 + Appendix K (the design is not lost) but are **not M6-shipping**; they re-enter when their subsystem is built. The pattern is now deliberate: **M6 ships the admin write-path for subsystems that exist; admin surfaces for subsystems that don't are downstream of building those subsystems.** Recorded per Rule 6 / D-065 / D-071.

**Phase 3 collapsed (2026-05-29).** This line predates Block 4. Block 4 (J-151) enumerated the verb set and bucketed every READ verb *with its category* (e.g. `federation list` ships in Phase 7 alongside the federation writes; `identity show` in Phase 5; see Appendix K), so each of the seven category phases (4–10 = A6/A5/A3/A1/A2/A4/A7) ships its reads and writes together. That leaves Phase 3 — a separate "read-only completions" step — with no enumerated verbs. Phase 3 is therefore **collapsed to zero** (sibling to the R3 outcome Phase 1 nearly took); the read surface is completed per-category. Next implementation step after Phase 2 is **Phase 4** (A6 Logging/audit, which lands the audit-write primitive every later phase consumes). Recorded per Rule 6 / D-065.

### 5.2 What Phase 2 ships

Phase 2 is the foundational scaffolding milestone. It ships:

1. **`xgen-node-lib::admin_ops` module skeleton.** Empty for now; subsequent phases add per-verb functions.
2. **`AdminContext` and `AdminError` types.** Mirror `OpContext` and the Client-side error pattern.

   > **`AdminContext` is runtime-aware for live-mutating categories (note added 2026-05-29, A5 design).** The audit verbs (A6) need only file/`data_dir` access, but security-critical mutating verbs must reach the *running* resident's in-memory state, not just disk. `identity revoke` is locked "immediate, Node-local, security-critical" (A5-D1): denial of authentication must take effect against the live resident at once, not on next restart — so A5 verbs mutate the live in-memory registry behind the pipe server's `Arc<Mutex<NodeRuntime>>` and then persist to `xgen-node_identities.db`. `AdminContext` therefore carries a runtime handle (alongside `data_dir`) for these categories. Precedent: A6-D1's `log set-level` already reaches into the live resident via the `tracing-subscriber` reload handle. M7's `--aicontrol` needs the same handle, so this is not throwaway scaffolding. The structural consequence — `AdminContext` is runtime-aware, not merely path-aware — is recorded here so it is a deliberate design property rather than an implicit one; per-category `M6_PHASE_N_IMPL.md` files note which verbs require the live handle.
3. **`TransportMessage::EventAccepted` wire shape.** Added to `xgen-common::wire::types::TransportMessage`. Emission site in `xgen-node-lib::accept_event` after persistence, before fan-out.
4. **Client-side `EventAccepted` handling.** Match arm in `xgen-client-lib`'s receive loops. Pure plumbing; behaviour wiring (waiting for accept in `ops::create_space` etc.) is deferred to a later phase as scoped per verb.
5. **`xgen-node-lib::audit` module skeleton.** Storage layer + entry insertion API. Empty `audit_entries` table created on first Node start. No verbs writing audit entries yet (those land per category).
6. **`pipe::dispatch_line` updated to call into `admin_ops::*`** for any new write verb, with the read-only allowlist preserved unchanged. The dispatcher's switch statement grows by phase.

Phase 2 is the prerequisite for every subsequent phase. Phases 3–10 each add one category's verbs against the Phase 2 scaffolding.

### 5.3 The propagation reliability audit milestone

> **Status (2026-05-29): this audit is CLOSED.** It ran and completed 2026-05-18 — `docs/xgen_propagation_reliability.md` (Status: COMPLETED), all five stages + close-out verdict-locked by Joe. Its §5 finding (the symmetric rejection signal) is locked into Phase 2 as the envelope-level `event_id` (see §3.1). **M6 implementation is unblocked.** The forward-looking text below is preserved as the audit's defining scope.

Between Phase 0 (this document) and Phase 1 of M6, a dedicated audit milestone runs:

**Title:** Propagation Reliability Audit  
**Owner:** Clair  
**Output:** A document (likely `docs/xgen_propagation_reliability.md`) covering:

- Client-to-Node event ingestion path (verified to be working)
- Local fan-out reliability (verified to be working; author-exclusion rationale recorded per Clair's J-080 finding)
- Sync-on-reconnect for disconnected clients (verified to be working)
- Federation send mechanism — Stage 6 of §4.1 — answers to the three questions in §4.3
- Federation reconciliation mechanism — DAG-tip reconciliation between peers
- `TransportMessage::Error` propagation scope (confirmed originator-only)
- Failure modes per stage and recovery mechanisms per stage
- Explicit list of gaps found, severity, recommended fix

**Definition of Done.** All Stage-6 questions answered with code-grounded evidence (not speculation). Findings recorded in the canonical document. Any gap surfaced is filed as a separate tracked deliverable. M6 (new) does not go ACTIVE until this audit closes.

This is the structural realisation of the "two events of equal importance" principle (§2.1): the accept signal makes a load-bearing claim, and the claim's reliability is verified before the signal ships.

---

## 6. Verb sets per category

The seven categories' verb-by-verb walks were completed in Block 4 (2026-05-29). M6 (new) ships **33 admin verbs** across the seven categories; five further verbs are explicitly deferred (`federation signal-defederation`, `space migrate-as-source`, `plugin load` / `configure` / `unload`). The full verb + schema reference is consolidated in **Appendix K** (`xgen_appendix_k_en.md`), a separate corpus appendix.

Each category section contains, per verb:
- Final verb name (two-token per §2.6.6)
- Class (READ / WRITE / DESTRUCTIVE)
- Argument schema (clap struct)
- Result schema (the `<Verb>Result` data shape)
- Error codes (per-category prefix per §2.7)
- Audit-entry implications (target field semantics, args_hash composition)
- Failure stage taxonomy (which stages from §2.6.5 the verb can fail at)
- Propagation interaction (does the verb emit a protocol event that triggers §3/§4?)
- Cross-references to spec sections

### 6.A1 Federation management

**Phase:** 7. **Verb count:** 7 (locked at Block 4; `federation signal-defederation` deferred — A1-D3). **Class mix:** 2 READ + 4 WRITE + 1 DESTRUCTIVE.

The largest category: the Node administrator manages who this Node federates with, on what terms. **All A1 verbs are federation-*relationship* management** (Node↔Node handshakes + `FederationRegistry`). They do **not** emit Space-DAG events — so there is **no `EventAccepted`** in A1; network failures map to the `federate` stage (best-effort, §2.6.5). The accept-signal first applies in A4's force-eject, not here.

**Block 4 locks (A1):**
- **A1-D1 — `federation initiate` is node-level, distinct from Client `federate`.** Two-level model confirmed: `federation initiate` (Node administrator establishes a **node-to-node** peer relationship → `FederationRegistry`, §4.2) vs Client `federate` (a **Space owner** federates a *specific Space* over a relationship → `SpaceState.federation_nodes`). Different actors, different objects; not duplicates.
- **A1-D2 — `federation list` is paginated** (`limit` + `cursor`), consistent with the F-7 pagination in the federation propagation design. A Node federated with hundreds of peers must not return as one text blob.
- **A1-D3 — `federation signal-defederation` is deferred post-M6.** §3.15 makes it reputation-affecting, but the Bootstrap-side consumer of the signal is unspecced/unbuilt (and A3-D1 already deferred the Bootstrap-*server* role). Emitting a signal nothing consumes is not useful in M6; re-enters scope when the Bootstrap reputation surface is designed.

**Common to all A1 verbs:** propagation = Node↔Node relationship (no Space-DAG event, no `EventAccepted`); clap `federation` `Subcommand` (§2.6.6); `FED_*` codes (numeric band harmonised at Block 4 close). `defederate` is reputation-affecting / hard-to-reverse → heavy audit; elevated-privilege gating (Pass 2 #1b) is N/A in v1 (no privilege gradation, §2.6.2; reserved for M7).

#### `federation list` — READ
- **Args** (`FederationListArgs`): `state: Option<FederationState>` (`active | pending | revoked | all`; default `all`), `limit: Option<usize>` (default 50, hard cap 500), `cursor: Option<String>`.
- **Result** `FederationListResult { relationships: Vec<FederationRelationship>, total_matched: usize, returned: usize, next_cursor: Option<String> }`.
- **Error codes:** `FED_3001` invalid state filter; `GENERIC_4000`.
- **Audit:** READ → not audited.
- **Failure stages:** `validate` → `register` (read `FederationRegistry`).
- **Spec refs:** §3.15, §4.2 (`FederationRegistry`), §2.6.4.

#### `federation accept` — WRITE
- **Args** (`FederationAcceptArgs`): `peer_node_id: String`, `endpoint: Option<String>` (if not carried by the pending request).
- **Result** `FederationAcceptResult { peer_node_id: String, federated_at: String, state: FederationState }`.
- **Error codes:** `FED_3002` no pending request for peer; `FED_3003` already federated; `FED_3010` peer unreachable (stage `federate`); `GENERIC_4000`.
- **Audit:** WRITE → entry written. `target` = peer_node_id; `args_hash` over `{peer_node_id, endpoint}`.
- **Failure stages:** `validate` → `register` (write `FederationRegistry`) → `federate` (establish the F-2 session).
- **Propagation:** Node↔Node; no Space-DAG event; no `EventAccepted`.
- **Spec refs:** §3.15, §4.2.

#### `federation reject` — WRITE
- **Args** (`FederationRejectArgs`): `peer_node_id: String`, `reason: Option<String>`.
- **Result** `FederationRejectResult { peer_node_id: String, rejected_at: String }`.
- **Error codes:** `FED_3002` no pending request; `GENERIC_4000`.
- **Audit:** WRITE → entry written.
- **Failure stages:** `validate` → `register` (mark the pending request rejected).
- **Propagation:** Node↔Node; no `EventAccepted`.
- **Spec refs:** §3.15.

#### `federation initiate` — WRITE
- **Args** (`FederationInitiateArgs`): `peer_node_id: String`, `endpoint: String`.
- **Result** `FederationInitiateResult { peer_node_id: String, state: FederationState, initiated_at: String }`.
- **Error codes:** `FED_3003` already federated; `FED_3010` peer unreachable (stage `federate`); `FED_3011` invalid endpoint; `GENERIC_4000`.
- **Audit:** WRITE → entry written.
- **Failure stages:** `validate` → `register` → `federate` (outbound handshake).
- **Propagation:** Node↔Node node-level relationship (distinct from Client Space-level `federate`, A1-D1); no `EventAccepted`.
- **Spec refs:** §3.15, §4.2.

#### `federation defederate` — DESTRUCTIVE
- **Args** (`FederationDefederateArgs`): `peer_node_id: String`, `reason: Option<String>`.
- **Result** `FederationDefederateResult { peer_node_id: String, defederated_at: String, cleaned_spaces: Vec<String> }`.
- **Error codes:** `FED_3004` not federated; `FED_3010` peer unreachable (stage `federate`; **local termination still proceeds** — best-effort notify); `GENERIC_4000`.
- **Audit:** DESTRUCTIVE → entry written (heavy; reputation-affecting, hard to reverse).
- **Failure stages:** `validate` → `register` (terminate relationship + local replica cleanup per D-022 / §3.15) → `federate` (best-effort notify the peer).
- **Propagation:** Node↔Node termination + local cleanup; no Space-DAG event emission; no `EventAccepted`.
- **Spec refs:** D-022, §3.15.

#### `federation set-policy` — WRITE
- **Args** (`FederationSetPolicyArgs`): `peer_node_id: String`, `mode: PolicyMode` (`allow | deny`), `allowed_spaces: Option<Vec<String>>`, `rate_limit: Option<u32>`.
- **Result** `FederationSetPolicyResult { peer_node_id: String, policy: FederationPolicy }`.
- **Error codes:** `FED_3004` unknown peer; `FED_3020` invalid policy; `GENERIC_4000`.
- **Audit:** WRITE → entry written. `target` = peer_node_id; `args_hash` over the policy fields.
- **Failure stages:** `validate` → `register` (write policy).
- **Propagation:** local policy; no event.
- **Spec refs:** §3.15, §4.2.

#### `federation show-policy` — READ
- **Args** (`FederationShowPolicyArgs`): `peer_node_id: Option<String>` (default all).
- **Result** `FederationShowPolicyResult { policies: Vec<FederationPolicyEntry> }`.
- **Error codes:** `GENERIC_4000`.
- **Audit:** READ → not audited.
- **Failure stages:** `validate` → `register` (read).
- **Spec refs:** §3.15, §4.2.

**Deferred (A1-D3):** `federation signal-defederation` — reputation signal to Bootstrap Nodes. Out of M6; re-enters when the Bootstrap reputation-consumption surface is designed (pairs with the deferred Bootstrap-server role, A3-D1).

### 6.A2 Auth Module management

**Phase:** 8 (confirmed; ships in M6 — A2-D1 removes the deferral condition). **Verb count:** 5 (locked at Block 4). **Class mix:** 2 READ + 2 WRITE + 1 DESTRUCTIVE.

Registering and managing the pluggable Auth Modules this Node trusts (Ch3 tiered-auth architecture, §3.6). All A2 verbs are Node-local registry operations; none emit Space-DAG events (no `EventAccepted`). `auth-module test` additionally makes an outbound diagnostic call to the module (A2-D2).

**Block 4 locks (A2):**
- **A2-D1 — `auth-module revoke` is block-only in M6; no cascade** (mirrors A5-D1; resolves the §3.6 gate). Revoking marks the module untrusted → it can no longer issue or validate Trust Assertions going forward. Existing Trust Assertions it issued remain valid until their **natural expiry** but cannot be renewed through the revoked module, so they age out on their own. Retroactive downgrade / invalidation / notification of already-authenticated Identities is **deferred** — same class and same unbuilt dependency as A5-D1 / the A4 signing machinery. This removes A2's deferral condition: **A2 ships in M6 (Phase 8 confirmed)**. M6's consistent rule: revocations do not cascade (identity or auth-module).
- **A2-D2 — `auth-module test` is an ad-hoc health-check in M6.** It sends a test challenge to the module and reports reachability/response; a *formal* health-check protocol message is deferred.
- **A2-D3 — `auth-module list` / `test` are not audited** (reads, A6-D4). `register` / `revoke` / `set-tiers` write audit entries (`revoke` heavy).

**Common to all A2 verbs:** propagation = Node-local (no Space-DAG event, no `EventAccepted`); clap `auth-module` `Subcommand` (§2.6.6); `AUTH_*` codes (numeric band harmonised at Block 4 close). `tiers` values are the modular Tier 1–4 set.

#### `auth-module list` — READ
- **Args** (`AuthModuleListArgs`): none (optional `revoked: Option<bool>` filter).
- **Result** `AuthModuleListResult { modules: Vec<AuthModuleRecord> }` (module_id, url, pubkey fingerprint, accepted_tiers, last_seen, revoked).
- **Error codes:** `GENERIC_4000`.
- **Audit:** READ → not audited.
- **Failure stages:** `validate` → `register` (read).
- **Spec refs:** §3.6, §2.6.4.

#### `auth-module register` — WRITE
- **Args** (`AuthModuleRegisterArgs`): `url: String`, `public_key: String`, `tiers: Vec<u8>`.
- **Result** `AuthModuleRegisterResult { module_id: String, accepted_tiers: Vec<u8>, registered_at: String }`.
- **Error codes:** `AUTH_2001` invalid URL; `AUTH_2002` invalid public key; `AUTH_2003` already registered; `AUTH_2021` invalid tier (out of 1–4); `GENERIC_4000`.
- **Audit:** WRITE → entry written. `target` = module_id; `args_hash` over `{url, public_key, tiers}`.
- **Failure stages:** `validate` → `register` (store record).
- **Propagation:** Node-local registry; no event.
- **Spec refs:** §3.6, §2.6.4.

#### `auth-module revoke` — DESTRUCTIVE
- **Args** (`AuthModuleRevokeArgs`): `module_id: String`, `reason: Option<String>`.
- **Result** `AuthModuleRevokeResult { module_id: String, revoked_at: String, note: String }` (`note` records the block-only semantics: existing assertions age out, no cascade).
- **Error codes:** `AUTH_2004` not registered; `AUTH_2005` already revoked; `GENERIC_4000`.
- **Audit:** DESTRUCTIVE → entry written (heavy).
- **Failure stages:** `validate` → `register` (mark module untrusted).
- **Propagation:** none — block-only, no cascade in M6 (A2-D1).
- **Spec refs:** §3.6 (cascade deferred), §2.6.4.

#### `auth-module set-tiers` — WRITE
- **Args** (`AuthModuleSetTiersArgs`): `module_id: String`, `tiers: Vec<u8>` (each 1–4).
- **Result** `AuthModuleSetTiersResult { module_id: String, accepted_tiers: Vec<u8> }`.
- **Error codes:** `AUTH_2004` not registered; `AUTH_2021` invalid tier; `GENERIC_4000`.
- **Audit:** WRITE → entry written.
- **Failure stages:** `validate` → `register` (registry write).
- **Spec refs:** §3.6, §2.6.4.

#### `auth-module test` — READ
- **Args** (`AuthModuleTestArgs`): `module_id: String`.
- **Result** `AuthModuleTestResult { module_id: String, reachable: bool, response_time_ms: Option<u32>, reported_tiers: Option<Vec<u8>> }`.
- **Error codes:** `AUTH_2004` not registered; `GENERIC_4000`. (Module unreachable is reported in `reachable: false`, not a hard error.)
- **Audit:** READ → not audited (A2-D3).
- **Failure stages:** `validate` → `register` (lookup) → ad-hoc challenge to the module (A2-D2).
- **Propagation:** Node↔AuthModule diagnostic; no event.
- **Spec refs:** §3.6 (ad-hoc health-check; formal protocol message deferred — A2-D2).

### 6.A3 Bootstrap configuration

**Phase:** 6. **Verb count:** 5 (locked at Block 4). **Class mix:** 1 READ + 3 WRITE + 1 DESTRUCTIVE.

Per §3.15, Bootstrap Nodes are the discovery layer. This category manages how this Node participates in Bootstrap discovery. Bootstrap verbs interact with an external Bootstrap Node (Node↔Bootstrap), which is **not** the Space-DAG accept-signal surface (§3/§4): there is **no `EventAccepted`** here; the result reflects the Bootstrap Node's response, and a network failure maps to the `federate` stage (best-effort per §2.6.5).

**Block 4 locks (A3):**
- **A3-D1 — Bootstrap *client*-only in M6.** This Node registers *itself* with Bootstrap Nodes. Operating *as* a Bootstrap Node (accepting inbound `bootstrap.register` from other Nodes) is a separate sub-category, deferred post-M6.
- **A3-D2 — `set-info` / `set-tiers` re-advertise.** After updating local config, both verbs **best-effort re-advertise** the change to currently-registered Bootstrap Nodes (`federate` stage). Per-bootstrap failures are reported in the result's `re_advertised_to` list, not raised as a hard error — the local config update always succeeds even if a re-advertise fails (honest, D-065).
- **A3-D3 — `bootstrap show` is not audited** (read, A6-D4). The 3 WRITE/DESTRUCTIVE verbs write audit entries.

**Common to all A3 verbs:** clap `bootstrap` `Subcommand` (§2.6.6); `BOOT_*` codes (numeric band harmonised at Block 4 close). `auth_tiers_served` values are the modular Tier 1–4 set.

#### `bootstrap show` — READ
- **Args** (`BootstrapShowArgs`): `bootstrap_id: Option<String>` (filter; default = all registrations).
- **Result** `BootstrapShowResult { registrations: Vec<BootstrapRegistration>, bootstrap_info: BootstrapInfo, auth_tiers_served: Vec<u8> }`.
- **Error codes:** `GENERIC_4000`.
- **Audit:** READ → not audited (A3-D3).
- **Failure stages:** `validate` → `register` (read local config).
- **Spec refs:** §3.15 (Bootstrap discovery), §2.6.4.

#### `bootstrap register` — WRITE
- **Args** (`BootstrapRegisterArgs`): `bootstrap_url: String`.
- **Result** `BootstrapRegisterResult { bootstrap_id: String, registered_at: String, advertised_tiers: Vec<u8> }`.
- **Error codes:** `BOOT_7001` invalid URL; `BOOT_7002` already registered; `BOOT_7010` Bootstrap unreachable (stage `federate`); `GENERIC_4000`.
- **Audit:** WRITE → entry written. `target` = bootstrap_url; `args_hash` = sha256(canonical `{bootstrap_url}`).
- **Failure stages:** `validate` → `register` (store record) → `federate` (send `bootstrap.register`, await ack).
- **Propagation:** Node↔Bootstrap interaction; not a Space-DAG event; no `EventAccepted`.
- **Spec refs:** §3.15, §2.6.4.

#### `bootstrap deregister` — DESTRUCTIVE
- **Args** (`BootstrapDeregisterArgs`): `bootstrap_id: String`.
- **Result** `BootstrapDeregisterResult { bootstrap_id: String, deregistered_at: String }`.
- **Error codes:** `BOOT_7003` not registered; `BOOT_7010` Bootstrap unreachable (stage `federate`; local record still removed — best-effort notify); `GENERIC_4000`.
- **Audit:** DESTRUCTIVE → entry written.
- **Failure stages:** `validate` → `register` (remove local record) → `federate` (best-effort notify the Bootstrap Node).
- **Propagation:** Node↔Bootstrap; reversible (re-register).
- **Spec refs:** §3.15, §2.6.4.

#### `bootstrap set-info` — WRITE
- **Args** (`BootstrapSetInfoArgs`): `display_name: Option<String>`, `description: Option<String>`, `contact: Option<String>` (partial update — only supplied fields change).
- **Result** `BootstrapSetInfoResult { bootstrap_info: BootstrapInfo, re_advertised_to: Vec<String> }`.
- **Error codes:** `BOOT_7020` invalid info; `GENERIC_4000`.
- **Audit:** WRITE → entry written. `args_hash` over the supplied fields.
- **Failure stages:** `validate` → `register` (update local config) → `federate` (best-effort re-advertise, A3-D2).
- **Propagation:** Node↔Bootstrap (re-advertise only); no Space-DAG event.
- **Spec refs:** §3.15, §2.6.3 (config field), §2.6.4.

#### `bootstrap set-tiers` — WRITE
- **Args** (`BootstrapSetTiersArgs`): `tiers: Vec<u8>` (the advertised `auth_tiers_served`; each value 1–4).
- **Result** `BootstrapSetTiersResult { auth_tiers_served: Vec<u8>, re_advertised_to: Vec<String> }`.
- **Error codes:** `BOOT_7021` invalid tier (out of 1–4); `GENERIC_4000`.
- **Audit:** WRITE → entry written. `args_hash` over `{tiers}`.
- **Failure stages:** `validate` (tier range) → `register` (update local config) → `federate` (best-effort re-advertise, A3-D2).
- **Propagation:** Node↔Bootstrap (re-advertise only); no Space-DAG event.
- **Spec refs:** §3.15, §2.6.4.

### 6.A4 Space and Room admin actions

**Phase:** 9. **Verb count:** 5 (locked at Block 4; `space migrate-as-source` deferred — A4-D2). **Class mix:** 3 READ + 1 WRITE + 1 DESTRUCTIVE.

Node-admin authority over Spaces **this Node originates / homes** (D-082 lock #4 — never federated-in replicas). Distinct from member-initiated governance: the Node administrator intervenes for legal / safety / operational reasons that supersede the Space's normal governance. **A4 is the only category that emits a Space-DAG event** (`force-eject`); the rest are Node-local policy / reads.

**Block 4 locks (A4):**
- **A4-D1 — `force-eject` signing identity: Option A locked as direction.** The force-eject emits a **new EventType `membership.node_eject`**, **signed by the Node keypair**, protocol-acknowledging Node-admin authority as a *distinct first-class authority* (not a masqueraded member kick) — honest per D-065 / D-070. The **detailed wire/validation sub-design** (exact event shape, the who-may-emit validation rule, Ch3 §3.3 registry entry, Appendix I wire entry, federation-validation interaction) is a **Phase-9 pre-implementation sub-design session** — the same pattern used for the `EventAccepted` shape before Phase 2. Block 4 locks the *direction*; Phase 9 opens with that focused session before code. Rejected: (B) reusing `membership.kick` + a `meta_atts` marker (a Node action wearing member-authority clothing; fragile validator special-case); (C) a separate admin keypair (premature — v1 is OS-user-equals-administrator, §2.6.1, no distinct admin identity yet).
- **A4-D2 — `space migrate-as-source` deferred post-M6.** §3.12 migration is a heavy multi-Node protocol flow; A4's core operational value is `force-eject` + node-policy. Re-enters when §3.12 migration is implemented.
- **A4-D3 — `space audit-events` targets the §3.11.8 *protocol* audit log (Space-scoped).** This is the "verb exposing the protocol log" that A6-D3 flagged as out-of-A6-scope; it lands here. It reads the JSONL protocol-event record filtered by Space / event-type / time — distinct from A6's `audit query` (the SQLite admin trail). READ, not audited.

**Common to all A4 verbs:** hosted-Spaces-only (D-082 lock #4); clap `space` `Subcommand` (§2.6.6); `SPACE_*` codes (numeric band harmonised at Block 4 close).

**`force-eject` and the accept signal.** `force-eject` originates from the `--batch` admin pipe, not a WS client, so **no `EventAccepted` wire message** is emitted — the verb **result** is the pipe-side analog and follows the G2 boundary: it returns after the `membership.node_eject` event is validated and persisted to the event store; local fan-out (§4 Stage 5) and federation propagation (§4 Stage 6) proceed asynchronously. The result carries the `event_id` so the administrator can correlate it in the DAG / audit log.

#### `space list-hosted` — READ
- **Args** (`SpaceListHostedArgs`): none (optional `name_filter: Option<String>`).
- **Result** `SpaceListHostedResult { spaces: Vec<HostedSpaceSummary> }` (space_id, name, member_count, room_count, federated_peers, created_at).
- **Error codes:** `GENERIC_4000`.
- **Audit:** READ → not audited.
- **Failure stages:** `validate` → `register` (read hosted-Space state).
- **Spec refs:** §2.6.4; Ch2 (hosts-but-doesn't-own).

#### `space force-eject` — DESTRUCTIVE
- **Args** (`SpaceForceEjectArgs`): `space_id: String`, `identity_id: String`, `reason: Option<String>`.
- **Result** `SpaceForceEjectResult { space_id: String, identity_id: String, ejected_at: String, event_id: String }`.
- **Error codes:** `SPACE_8001` Space not hosted here; `SPACE_8002` identity not a member; `SPACE_8003` already removed; `SPACE_8004` persist failed (stage `persist`); `GENERIC_4000`.
- **Audit:** DESTRUCTIVE → entry written (heavy, full provenance per §3.11.8). `target` = `{space_id, identity_id}`; `args_hash` over `{space_id, identity_id, reason}`. The emitted `membership.node_eject` `event_id` is recorded in the audit entry's `correlation_id`.
- **Failure stages:** `validate` (hosted? member?) → `register` (authority check, D-082 lock #4) → `persist` (write `membership.node_eject` to event store; G2 boundary) → `notify` (local fan-out) → `federate` (peer propagation). Best-effort after persist per §2.6.5: the verb returns success once persisted; downstream `notify` / `federate` failures are async and do not roll back the eject.
- **Propagation:** **emits a Space-DAG event** (`membership.node_eject`, A4-D1) → normal fan-out + federation (§4). No `EventAccepted` wire message (pipe-originated; verb result is the G2-analog).
- **Spec refs:** A4-D1 (signing, Phase-9 sub-design), §3.11.8 (audit provenance), §3.3 / Appendix I (EventType — Phase 9), §4 (propagation), D-082 lock #4.

#### `space set-node-policy` — WRITE
- **Args** (`SpaceSetNodePolicyArgs`): `space_id: String`, `policy: NodePolicy` (auto-mute thresholds, rate caps, etc.).
- **Result** `SpaceSetNodePolicyResult { space_id: String, policy: NodePolicy }`.
- **Error codes:** `SPACE_8001` not hosted here; `SPACE_8020` invalid policy; `GENERIC_4000`.
- **Audit:** WRITE → entry written.
- **Failure stages:** `validate` → `register` (write Node-level policy store).
- **Propagation:** **none** — Node-level enforcement layer, separate from the Space governance DAG; no protocol event.
- **Spec refs:** §2.6.4 (Node moderation policy is Node-local, not Space-DAG state).

#### `space show-node-policy` — READ
- **Args** (`SpaceShowNodePolicyArgs`): `space_id: String`.
- **Result** `SpaceShowNodePolicyResult { space_id: String, policy: NodePolicy }`.
- **Error codes:** `SPACE_8001` not hosted here; `GENERIC_4000`.
- **Audit:** READ → not audited.
- **Failure stages:** `validate` → `register` (read).
- **Spec refs:** §2.6.4.

#### `space audit-events` — READ
- **Args** (`SpaceAuditEventsArgs`): `space_id: String`, `event_type: Option<String>`, `since: Option<String>` (RFC 3339), `until: Option<String>`, `limit: Option<usize>` (default 100), `cursor: Option<String>`.
- **Result** `SpaceAuditEventsResult { events: Vec<ProtocolAuditEntry>, returned: usize, next_cursor: Option<String> }`.
- **Error codes:** `SPACE_8001` not hosted here; `SPACE_8010` bad filter; `GENERIC_4000`.
- **Audit:** READ → not audited (A4-D3).
- **Failure stages:** `validate` → `register` (scan / filter the §3.11.8 protocol audit log).
- **Propagation:** none.
- **Spec refs:** §3.11.8 (protocol audit log — the A6-D3-anticipated reader), §2.6.4.

**Deferred (A4-D2):** `space migrate-as-source` — §3.12 Space migration trigger. Out of M6; re-enters when §3.12 migration is implemented.

### 6.A5 Identity registry administration

**Phase:** 5. **Verb count:** 4 (locked at Block 4). **Class mix:** 1 READ + 2 WRITE + 1 DESTRUCTIVE. (`identity list` already ships from M2.)

Managing the Identity records this Node stores. All A5 verbs are **Node-local** — they govern this Node's registry, not the Identity's standing on other Nodes (a Node administrator's authority is scoped to what this Node hosts, D-082). None emit a protocol event in M6 (see A5-D1); propagation interaction = none.

**Block 4 locks (A5):**
- **A5-D1 — `identity revoke` is block-only in M6; cascade deferred.** (Resolves the §3.6-escalated cascade question.) Revoke marks the Identity revoked in the registry → authentication / session-open is denied on this Node (immediate, Node-local, security-critical). Existing Space membership rows are left in place but **inert** — a revoked Identity cannot authenticate, so it cannot act on them. The result reports `stale_membership_spaces` (honest, D-065). Force-eject (cascade) is deferred until the A4 signing-identity sub-design exists: cascading means emitting `membership.kick` events whose signer is the unresolved A4 question, and A4 is Phase 9 while A5 is Phase 5. Block-only decouples the phases; cascade lands as a post-A4 follow-up. Consistent across M6: revocations do not cascade (also settles the A2 §3.6 open item).
- **A5-D2 — `identity manage-replica` is thin-scope.** Registry-only: declare and list which Nodes are recorded as holding replicas of an Identity record. Active replication orchestration (pushing / reconciling the actual record across Nodes) is out of M6 — it belongs with the federation-replication model still under the §5.3 audit.
- **A5-D3 — `identity show` is not audited.** Interactive single-record display, not an off-box extraction; follows A6-D4 (pure reads not audited). PII exposure is bounded by §2.6.1 connection authority.

**Common to all A5 verbs:** propagation interaction = **none**; clap `identity` `Subcommand` (§2.6.6); `IDENT_*` codes (numeric band harmonised at Block 4 close). **A5 verbs mutate live runtime state** — `AdminContext` carries the resident runtime handle for this category (the "immediate, security-critical" requirement of A5-D1; see the runtime-awareness note in §5.2).

#### `identity show` — READ
- **Args** (`IdentityShowArgs`): `identity_id: String`.
- **Result** `IdentityShowResult { record: IdentityRecord }` (display name, registration time, Trust Assertion status + expiry, is_ai, capabilities, devices, revoked flag).
- **Error codes:** `IDENT_6001` not found; `GENERIC_4000`.
- **Audit:** READ → not audited (A5-D3).
- **Failure stages:** `validate` → `register` (registry read).
- **Spec refs:** Identity registry, §2.6.4.

#### `identity revoke` — DESTRUCTIVE
- **Args** (`IdentityRevokeArgs`): `identity_id: String`, `reason: Option<String>`.
- **Result** `IdentityRevokeResult { identity_id: String, revoked_at: String, stale_membership_spaces: Vec<String> }`.
- **Error codes:** `IDENT_6001` not found; `IDENT_6002` already revoked; `GENERIC_4000`.
- **Audit:** DESTRUCTIVE → entry written (heavy audit per the §A5 risk note). `target` = identity_id; `args_hash` = sha256(canonical `{identity_id, reason}`).
- **Failure stages:** `validate` → `register` (mark revoked). No `notify` / `federate` in M6 (block-only, A5-D1).
- **Propagation:** none in M6 (cascade deferred).
- **Spec refs:** §3.6 (revocation; cascade deferred), §2.6.4.

#### `identity set-trust-expiry` — WRITE
- **Args** (`IdentitySetTrustExpiryArgs`): `identity_id: String`, `expiry: String` (RFC 3339).
- **Result** `IdentitySetTrustExpiryResult { identity_id: String, previous_expiry: Option<String>, new_expiry: String }`.
- **Error codes:** `IDENT_6001` not found; `IDENT_6010` malformed/invalid expiry; `GENERIC_4000`.
- **Audit:** WRITE → entry written. `target` = identity_id; `args_hash` over `{identity_id, expiry}`.
- **Failure stages:** `validate` → `register` (registry write).
- **Spec refs:** Trust Assertion (identity model), §2.6.4.

#### `identity manage-replica` — WRITE  *(thin-scope; A5-D2)*
- **Args** (`IdentityManageReplicaArgs`): `identity_id: String`, `action: ReplicaAction` (`add | remove | list`), `node_id: Option<String>` (required for add/remove).
- **Result** `IdentityManageReplicaResult { identity_id: String, replicas: Vec<String> }` (the post-action replica-Node list).
- **Error codes:** `IDENT_6001` identity not found; `IDENT_6020` invalid node_id; `IDENT_6021` replica already present / not present; `GENERIC_4000`.
- **Audit:** WRITE → entry written for `add`/`remove` (`target` = identity_id; `args_hash` over `{identity_id, action, node_id}`); `list` is a read → not audited (A5-D3 pattern).
- **Failure stages:** `validate` → `register` (registry write).
- **Propagation:** none — records the relationship only; no active replication push (A5-D2).
- **Spec refs:** Identity replica relationships (federation model), §2.6.4.

### 6.A6 Logging and audit administration

**Phase:** 4. **Verb count:** 5 (locked at Block 4). **Class mix:** 2 WRITE (one DESTRUCTIVE) + 3 READ.

The landing-pad category: Phase 4 lands the audit primitive every subsequent write phase (5–10) consumes. The `audit *` verbs operate on the **admin audit trail** — the SQLite store at `<data_dir>/xgen-node_audit.db` (§2.6.4) — **not** the §3.11.8 protocol audit log (JSONL, auto monthly rotation), which stays spec-managed with no admin verb [A6-D3]. Two distinct logs, two audiences: `audit query` covers admin-action events; an auditor querying protocol/compliance events reads the §3.11.8 log directly on disk. A verb exposing the protocol log is out of M6 scope.

**Block 4 locks (A6):**
- **A6-D1 — `log set-level` is runtime-only.** Applies immediately via the `tracing-subscriber` reload handle (`[logging].level` is Reloadable, §2.6.3); does NOT persist to config (config-write is the M7 live-reload mechanism). The level survives until Node restart.
- **A6-D2 — `audit rotate` → `audit archive` (DESTRUCTIVE).** The Pass-2 `audit rotate` sketch assumed a JSONL-file model; under the locked SQLite store (§2.6.4) it is reframed as `audit archive`: export rows older than a cutoff to a dated file, then prune them from the live table. This is the administrator's growth-management tool — it preserves the data (archived) while honouring "audit must not be silently auto-deleted" (§2.6.4); the prune is an explicit, audited action.
- **A6-D3 — `audit *` targets the SQLite admin trail only** (above).
- **A6-D4 — audit-the-auditor.** WRITE verbs (`log set-level`, `audit archive`) and the data-extracting `audit export` write audit entries; `audit query` and `log show-level` do not (pure reads).

**Common to all A6 verbs:** propagation interaction = **none** (local Node admin; emits no protocol event; no §3/§4 accept-signal). clap shape: `log` and `audit` are each a `Subcommand` enum variant with nested subcommands (§2.6.6). Error-code numeric bands below are A6-local; cross-category numeric harmonisation is confirmed at Block 4 close.

#### `log set-level` — WRITE
- **Args** (`LogSetLevelArgs`): `module: Option<String>` (target module path, e.g. `xgen_node::federation`; default `*` = global), `level: LogLevel` (`error|warn|info|debug|trace`).
- **Result** `LogSetLevelResult { module: String, previous_level: String, new_level: String, applied: bool }`.
- **Error codes:** `LOG_5101` invalid level; `LOG_5102` unknown/unsettable module; `GENERIC_4000` bad args.
- **Audit:** WRITE → entry written. `target` = module (or `*`); `args_hash` = sha256(canonical `{module, level}`).
- **Failure stages:** `validate` (bad level/module) → `register` (apply to the reload handle).
- **Spec refs:** §2.6.3 (Reloadable bucket), Appendix G (logging convention), §3.11.8 (debug log).

#### `log show-level` — READ
- **Args** (`LogShowLevelArgs`): `module: Option<String>` (filter; default = all effective levels).
- **Result** `LogShowLevelResult { levels: Vec<LogLevelEntry { module: String, level: String }> }`.
- **Error codes:** `GENERIC_4000` bad args.
- **Audit:** READ → not audited (A6-D4).
- **Failure stages:** `validate`.
- **Spec refs:** §2.6.3, Appendix G.

#### `audit archive` — WRITE / DESTRUCTIVE  *(was `audit rotate`; A6-D2)*
- **Args** (`AuditArchiveArgs`): `before: String` (RFC 3339; rows with `timestamp < before` are archived then pruned), `output: Option<PathBuf>` (archive file; default `<data_dir>/audit/xgen-node_audit_archive_<ts>.jsonl`).
- **Result** `AuditArchiveResult { archived_count: usize, archive_path: String, oldest_ts: Option<String>, newest_ts: Option<String> }`.
- **Error codes:** `AUDIT_5010` malformed `before`; `AUDIT_5001` archive write failed (stage `persist`); `AUDIT_5002` prune failed after a successful archive (stage `persist`; **fail-safe toward retention** — archive file kept, rows NOT deleted); `GENERIC_4000`.
- **Audit:** WRITE + DESTRUCTIVE → entry written. `target` = archive_path; `args_hash` = sha256(canonical `{before, output}`).
- **Failure stages:** `validate` → `persist` (write archive) → `persist` (prune). Best-effort per §2.6.5: a prune failure after a good archive leaves rows in place and reports `stage = persist`.
- **Spec refs:** §2.6.4 (storage; must-not-auto-delete), §2.6.5 (failure semantics).

#### `audit query` — READ
- **Args** (`AuditQueryArgs`): `actor: Option<String>`, `verb: Option<String>`, `since: Option<String>` (RFC 3339), `until: Option<String>`, `outcome: Option<String>` (`ok|error`), `limit: Option<usize>` (default 100, hard cap 1000).
- **Result** `AuditQueryResult { entries: Vec<AuditEntry>, total_matched: usize, returned: usize }` (`AuditEntry` = the §2.6.4 row).
- **Error codes:** `AUDIT_5010` bad filter (malformed timestamp / unknown outcome); `GENERIC_4000`.
- **Audit:** READ → not audited (A6-D4).
- **Failure stages:** `validate` (filter) → `register` (SQLite read).
- **Spec refs:** §2.6.4.

#### `audit export` — READ *(data-extracting)*
- **Args** (`AuditExportArgs`): the `audit query` filter set + `output: PathBuf`, `format: Option<String>` (default `jsonl`; `csv` reserved).
- **Result** `AuditExportResult { exported_count: usize, output_path: String, format: String }`.
- **Error codes:** `AUDIT_5010` bad filter; `AUDIT_5020` export write failed (stage `persist`); `GENERIC_4000`.
- **Audit:** READ but data-extracting → entry written (A6-D4). `target` = output_path; `args_hash` over the filter + output args.
- **Failure stages:** `validate` → `register` (read) → `persist` (write file).
- **Spec refs:** §2.6.4 (export materialises JSONL from SQLite for SIEM).

### 6.A7 Plugin management

**Phase:** 10. **Verb count:** 2 (locked at Block 4; the WRITE verbs `load` / `configure` / `unload` deferred — A7-D1). **Class mix:** 2 READ.

Managing pluggable in-process modules. Today the only customer is D-061's `NoOpTemperaturePlugin`. **A7-D1: M6 ships the two READ verbs only.** The WRITE verbs (`plugin load`, `plugin configure`, `plugin unload`) have no real customer with a single no-op plugin — load-by-name (binary-compiled in v1), configure (nothing to configure), and unload have nothing meaningful to operate on. They land when a second plugin exists (same "no half-feature on an immature surface" call as A4-D2 / A1-D3). A7 is the smallest category — 2 reads, nothing to audit.

**Common to A7 verbs:** propagation = none; clap `plugin` `Subcommand` (§2.6.6); `PLUGIN_*` codes.

#### `plugin list` — READ
- **Args** (`PluginListArgs`): none.
- **Result** `PluginListResult { plugins: Vec<PluginSummary> }` (name, version, status, kind).
- **Error codes:** `GENERIC_4000`.
- **Audit:** READ → not audited.
- **Failure stages:** `validate` → `register` (read plugin registry / in-process state).
- **Spec refs:** D-061 (plugin subsystem), §2.6.4.

#### `plugin status` — READ
- **Args** (`PluginStatusArgs`): `plugin_name: String`.
- **Result** `PluginStatusResult { name: String, version: String, status: String, kind: String, events_consumed: Option<u64>, last_activity: Option<String> }`.
- **Error codes:** `PLUGIN_9001` unknown plugin; `GENERIC_4000`.
- **Audit:** READ → not audited.
- **Failure stages:** `validate` → `register` (lookup).
- **Spec refs:** D-061, §2.6.4.

**Deferred (A7-D1):** `plugin load` / `plugin configure` / `plugin unload` — land when the plugin set extends beyond the single no-op plugin.

---

## 7. Out of scope for M6

The following are explicitly NOT in M6 and are deferred to specific named milestones or remain as design open items:

- **Node `--aicontrol` surface.** M7 per D-066. The `admin_ops::*` layer M6 builds will be reused by M7's `--aicontrol` dispatcher.
- **Live config reload.** Standalone M7 milestone. M6 defines the live-reload bucket (§2.6.3) so M6 verbs that touch reloadable fields know the planned future behaviour, but the reload mechanism itself ships in M7.
- **Protocol-level authentication on the `--batch` pipe.** §2.6.1 locks OS-user-equals-administrator for M6 v1. M7 may revisit.
- **Per-verb authorisation gating.** §2.6.2 locks session-scoped for M6 v1. M7 may revisit alongside pipe authentication.
- **Full-args (non-hashed) audit storage opt-in.** Future config flag, post-M6.
- **Connection management category.** Disconnect specific clients, rate limits, IP bans. Deferred per Pass 3 Thread 2; re-enters roadmap if operational pain surfaces.
- **DAG / Space-storage administration category.** Compact, vacuum, force-replay, repair. Deferred per Pass 3 Thread 2; correctness of force-re-replay is non-trivial design work.
- **Auth-module / identity revocation cascade.** Retroactive downgrade / invalidation / notification of Identities authenticated via a revoked Auth Module (or directly revoked) is deferred — M6 revocations are block-only (A2-D1 / A5-D1); the cascade depends on the A4 signing machinery.
- **Plugin WRITE verbs (`load` / `configure` / `unload`).** Deferred per A7-D1 — they land when the plugin set extends beyond the single no-op `NoOpTemperaturePlugin`.

---

## 8. Cross-references

| Decision | Relevance to this document |
|---|---|
| **D-035** | Convention-derived path rule; applies to audit storage location and per-instance segregation. |
| **D-043** | Pipe naming convention with `--instance`; M6 verbs that operate against per-instance Node residents inherit this naming. |
| **D-056** | One-binary-per-role model; M6 admin write path is a dispatch mode on existing `xgen-node`, not a new binary. |
| **D-063** | Library-first principle; `admin_ops::*` is the same shape as `ops::*`. |
| **D-065** | Honest behaviour over polite behaviour; named here in §2.1, §2.2, §2.6.5. |
| **D-066** | `--aicontrol` split from `--batch`; M7 reuses `admin_ops::*` per this design. |
| **D-067** | Single source of truth for command implementations; `admin_ops::*` is the Node-side equivalent of `ops::*`. |
| **D-068** | CLI flag > config precedence; applies to any M6 verb argument shadowing a config field. |
| **D-069** | Delegated design discipline; this document is the canonical-document-rule realisation for M6. |
| **D-070** | "Two events of equal importance, opposite direction" — named protocol principle, canonical form in DECISIONS.md (promoted 2026-05-18 with corrected post-audit framing). The principle requires BOTH (1) acceptance and rejection signals exist with equal first-class status, AND (2) both carry the envelope-level correlation identifier. M6 §3 implements both halves: `EventAccepted` provides (1); envelope `event_id` per §3.1 provides (2). §9 below preserves the original Pass-3 draft as historical record. |
| **D-071** | "Subsystem audits precede dependent milestones" — project-management principle, canonical form in DECISIONS.md (promoted 2026-05-18). Names the discipline that produced the Propagation Reliability Audit (J-081) whose findings shaped §3's envelope-level correlation design. |

---

## 9. D-070 — SUPERSEDED by DECISIONS.md D-070 (canonical)

**Status of this section.** SUPERSEDED 2026-05-18.

D-070 ("Two events of equal importance, opposite direction") was originally drafted in this section during Pass 3 of M6 Phase 0. The promotion to a numbered DECISIONS.md entry happened in a same-day post-audit recording session. **The canonical authoritative form of D-070 lives in `DECISIONS.md` (current range D-000 through D-071), not here.**

The canonical form incorporates the corrected post-audit framing surfaced during the Propagation Reliability Audit (J-081 §5). The original Pass-3 framing in this section was:

> *"EventAccepted exists, symmetric to Error."*

Necessary but not sufficient. J-081 §5 found that `TransportMessage::Error` lacked an `event_id` field at all — meaning even with both Error and a future EventAccepted, the originator could not correlate either signal back to a specific event. D-070's DECISIONS.md form makes both halves explicit:

1. **Both directions exist** (acceptance + rejection — the original Pass-3 half).
2. **Both directions carry the envelope-level correlation identifier** (the post-audit half).

Without (2), (1) is hollow. The full reasoning is recorded in DECISIONS.md D-070's "Why the corrected framing matters" section.

The original §9 body that follows below is preserved as historical record of the Pass-3 framing. It is NOT the canonical reference. Future readers should cite DECISIONS.md D-070; this section exists to record how the principle was first surfaced and how the framing evolved.

---

### Original Pass-3 draft text (preserved as historical record)

**Title.** Two events of equal importance, opposite direction (named protocol principle).

**Date.** 2026-05-18 (Pass 3 of M6 Phase 0).

**Layer.** Protocol (Ch3) AND reference-implementation surfaces consuming the protocol.

### Decision

Wherever the XGen Protocol exposes a signal from one party to another about the outcome of an action, both directions of outcome — acceptance and rejection — MUST be exposed with equal first-class status, on the same layer, at the same time.

The principle was applied to close the M6-era asymmetry where rejection had a signal (`TransportMessage::Error`) but acceptance had none. The fix (`TransportMessage::EventAccepted`) restores symmetry: both are transport-layer wire messages, both are originator-only, both fire at the same boundary in the event-acceptance pipeline.

### Joe's verbatim framing

> *"Acceptance and rejection are two events of equal importance, just opposite direction."*

And on choosing C2 over C1 (server-side self-fanout) and C3 (DAG-layer ack EventType):

> *"The accept signal's importance warrants its own wire shape, not a side effect of an unrelated mechanism."*

### Why this is a structural principle, not stylistic

**1. It prevents the structural-by-accident asymmetry that produced the M6 carry-over.** The accept-signal gap existed because nobody designed an accept signal — it was a consequence of the event-streaming model (events flow one way; the response is fan-out, not a per-event reply). Asymmetries that arise from "we didn't think about it" rather than "we deliberately chose this" produce silent correctness bugs in the layers above. Naming the principle makes future asymmetries visible: any design that exposes one direction needs to either expose the other or document why not.

**2. It pairs with D-065 ("honest behaviour over polite behaviour") cleanly.** D-065 says: don't misrepresent the system's state. This principle says: when the protocol could speak the truth in both directions, it must. Together they bind: a protocol with only a rejection signal forces consumers to fake acceptance via heuristics (silence-equals-success); a protocol with both signals lets consumers speak honestly in both directions.

**3. It is reusable across future protocol design.** Any future XGen protocol addition (a new transport message family, a new federation request shape, a new bootstrap interaction) inherits the principle. When future Pass 3 design sessions ask "should this only signal failure, or should it also signal success?", the principle gives a default: yes, both, equal weight. Departures from the default require explicit justification.

### Out of scope

- Asymmetries where one direction genuinely doesn't apply. Example: `TransportMessage::Goodbye` does not have a `TransportMessage::Greetings` equivalent because connection establishment is asymmetric by nature (the WS handshake itself is the greeting). This principle does not force false symmetries where the underlying interaction is genuinely one-directional.
- The propagation reliability question. That is a separate concern (§4.3 of this document) addressed in the audit milestone.
- Asymmetries internal to the reference implementation (a binary's CLI surface having a `--start` flag with no `--stop` flag, etc.). The principle is about protocol-level signals, not implementation-internal control flow.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| **D-065** | Sibling principle. D-065 binds the *content* of signals (don't lie); this binds the *existence* of signals (when you can speak in one direction, you can speak in the other). Together they constrain the protocol to behaviour that is both honest and complete. |
| **D-066** | The `--aicontrol` JSONL protocol (M7) inherits this principle. Every JSONL `reply` shape will carry both `result` and `error` paths at equal first-class status, mirroring the `Error` / `EventAccepted` symmetry M6 establishes. |
| **D-069** | This decision was Joe-locked during a delegated design session (Pass 3 of M6 Phase 0) per the D-069 discipline. The principle's promotion to DECISIONS.md follows the same Joe-lock-then-canonicalize pattern. |

### Why now

Pass 3 of M6 Phase 0 surfaced the missing accept-signal as a structural gap (via Clair's J-080 finding). Closing the gap meant designing `EventAccepted`. Naming the principle that justifies `EventAccepted` makes the design call durable: future readers of `xgen-node-lib`, future protocol additions, future contributors all benefit from a one-line citable principle they can invoke. "Add it because D-070 says so" is shorter than re-deriving the reasoning every time.

---

*End of design document.*
