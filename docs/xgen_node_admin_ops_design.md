# XGen Node Admin Operations Design (M6)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-18 (F-9 documentation correction at §4.2 — Stage-6 federation propagation sub-bullet replaced with forward-reference to the canonical Federation Event Propagation design doc per Pass 3 of that milestone's design phase)  
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

For operational reality, this is a gap. A Node operator wanting to accept a federation request, revoke an Identity, register with a Bootstrap Node, or rotate an Auth Module today has no protocol-level surface; the only paths are direct database editing or restarting with manually edited config files. Neither is appropriate.

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

**2.6.1 Connection authority — Joe-lock #1a.** OS-user-equals-operator. The `xgen-node --batch` pipe inherits OS-level access control from the Node process's user. No protocol-level authentication on the pipe itself. M7 may revisit this when remote-driver scenarios (MCP servers running as different OS users) actually surface.

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
    actor            : TEXT          -- identity_id URI of the initiating operator
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

**Why `args_hash` rather than full args.** Some verb args contain potentially sensitive data (target identity IDs that may later need GDPR removal, etc.). Hashing keeps the audit verifiable (you can re-hash a candidate args block and check match) without storing the data itself. Operators concerned about strict non-repudiation can opt into full-args storage via a future config flag; that opt-in is out of M6 scope.

**Export.** The `audit export` verb (§6.A6) materializes a JSONL slice from the SQLite source for SIEM ingestion. Query (`audit query`) reads directly from SQLite.

**2.6.5 Failure semantics — Joe-lock #5.** Best-effort with honest reporting. Partial state is left in place on mid-verb failure; the verb returns a structured error indicating the stage where it failed. The operator decides recovery via category-specific verbs.

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
- Failure: `ERROR <CODE>: <message>\n` where `<CODE>` is a structured error code

Error codes follow a per-category namespace:

| Prefix | Category |
|---|---|
| `FED_*` | Federation management |
| `AUTH_*` | Auth Module management |
| `BOOT_*` | Bootstrap configuration |
| `SPACE_*` | Space/Room operator actions |
| `IDENT_*` | Identity registry administration |
| `LOG_*`, `AUDIT_*` | Logging and audit administration |
| `PLUGIN_*` | Plugin management |
| `GENERIC_*` | Verb-agnostic (bad args, internal error, etc.) |

Specific error codes are enumerated per verb in §6 (TBD — Block 4). The `--aicontrol` JSONL shape (M7) will carry the same codes in structured form (`{"error": {"code": "FED_3041", "message": "..."}}`), so the codes defined in M6 propagate forward without renaming.

---

## 3. The `EventAccepted` protocol addition

This is the only protocol-level change M6 ships. Everything else is reference-implementation surface.

### 3.1 Wire shape

```rust
pub enum TransportMessage {
    // ... existing variants ...
    Error { event_id: String, reason: String, /* ... */ },
    EventAccepted { event_id: String, accepted_at: String },  // NEW in M6
    Goodbye { reason: String },
    // ...
}
```

Field semantics:

- `event_id` — the hash URI of the accepted event (the same `event_id` the originator sees on the `Event` they sent).
- `accepted_at` — RFC 3339 UTC timestamp of acceptance. Recorded for trace/audit purposes; not used for any timing-sensitive logic.

### 3.2 When the Node MUST send it

After the inbound event clears the full validation pipeline (Ch3 §3.7) AND has been successfully written to the Node's event store, AND BEFORE local fan-out (`apply_fanout`) begins.

This boundary is the G2 semantic: validated, persisted, ack sent, then async propagation proceeds. Sending `EventAccepted` before persistence is forbidden by the spec; sending it after fan-out completes is permitted but discouraged (latency cost without semantic gain).

### 3.3 When the Node MUST NOT send it

- If validation fails — `TransportMessage::Error` is sent instead, citing the failure stage.
- If persistence fails — `TransportMessage::Error` is sent, citing `persist` stage. `EventAccepted` is never speculatively sent then retracted.
- If the inbound message is not an event (e.g. a `TransportMessage::Goodbye`). The accept signal applies only to DAG-event submissions.

### 3.4 Originator-facing semantics

When an originator receives `EventAccepted { event_id }`, they may claim:

- ✅ The event is in the home Node's authoritative DAG store.
- ✅ The home Node has validated the event (signature, prev_events, state machine).
- ✅ Any subsequent client connecting to this Node and asking for this Space's DAG will see the event.

They may NOT claim:

- ❌ Other members have received the event (fan-out runs after ack; offline members catch up via sync).
- ❌ Federation peers know about the event (federation propagation is asynchronous and proceeds at its own pace).
- ❌ The event is in any DAG other than the home Node's.

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
- **Federation propagation (Stage 6):** Specified in `docs/xgen_federation_propagation_design.md` (Status: ACTIVE, v1.0). That document is the canonical design for federation event push (F-1), the persistent peer session model (F-2), the receive-side validation gate (F-3 + F-4), pairwise propagation (F-5), the `sync_complete` wire shape (F-6), pagination (F-7), and DAG-hole semantics (F-10). Implementation lands in the Federation Event Propagation completion milestone. Until that milestone closes, Node-to-Node federation event propagation does not occur as a production mechanism — the only Node-to-Node delivery today is the one-time history dump that runs at peer-initiated handshake, then the connection closes (J-081 audit §2 records the absent-mechanism state in code-grounded detail). The M6 admin verbs that govern *federation relationships* (federation accept / reject / defederate, per category 6.A1) are unaffected by the propagation-mechanism gap; they manage the relationship records that the propagation design will operate against once implementation lands.
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
Phase 0 — Design (Pass 1 ✅ Pass 2 ✅ Pass 3 ✅ [this document])
Phase 1 — Client gap patches (R1/R2 from Pass 1; R3: may be zero commits)
Phase 2 — admin_ops::* scaffolding + TransportMessage::EventAccepted shape
Phase 3 — Read-only completions on existing --batch
Phase 4 — Logging/audit admin (audit primitive lands here)
Phase 5 — Identity registry admin
Phase 6 — Bootstrap configuration  (smaller category; before Federation per Pass 3 swap)
Phase 7 — Federation management
Phase 8 — Auth Module management  (TBD; may defer if §3.6 revocation cascade is spec-gap)
Phase 9 — Space/Room operator actions  (signing-identity sub-design first)
Phase 10 — Plugin management
```

Phase 1's content is determined by which of Pass 1's R1/R2/R3 recommendations are accepted. R1 is `rooms` + `members` (2 atomic Client commits). R2 defers `federate` to Phase 7. R3 acknowledges Phase 1 may collapse to zero. Block 4 confirms R1/R2/R3 decisions.

### 5.2 What Phase 2 ships

Phase 2 is the foundational scaffolding milestone. It ships:

1. **`xgen-node-lib::admin_ops` module skeleton.** Empty for now; subsequent phases add per-verb functions.
2. **`AdminContext` and `AdminError` types.** Mirror `OpContext` and the Client-side error pattern.
3. **`TransportMessage::EventAccepted` wire shape.** Added to `xgen-common::wire::types::TransportMessage`. Emission site in `xgen-node-lib::accept_event` after persistence, before fan-out.
4. **Client-side `EventAccepted` handling.** Match arm in `xgen-client-lib`'s receive loops. Pure plumbing; behaviour wiring (waiting for accept in `ops::create_space` etc.) is deferred to a later phase as scoped per verb.
5. **`xgen-node-lib::audit` module skeleton.** Storage layer + entry insertion API. Empty `audit_entries` table created on first Node start. No verbs writing audit entries yet (those land per category).
6. **`pipe::dispatch_line` updated to call into `admin_ops::*`** for any new write verb, with the read-only allowlist preserved unchanged. The dispatcher's switch statement grows by phase.

Phase 2 is the prerequisite for every subsequent phase. Phases 3–10 each add one category's verbs against the Phase 2 scaffolding.

### 5.3 The propagation reliability audit milestone

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

## 6. Verb sets per category (TBD — Block 4)

The seven categories' verb-by-verb walks were deferred to Block 4 of Pass 3, scheduled as a separate session. This section's structure is locked; the per-verb content lands when Block 4 runs.

Each category section will contain, per verb:
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

**Phase:** 7. **Approximate verb count:** ~8. **Sketches in Pass 2 §A1.**

*Block 4 — TBD.*

### 6.A2 Auth Module management

**Phase:** 8 (TBD; deferral candidate). **Approximate verb count:** ~5. **Sketches in Pass 2 §A2.**

*Block 4 — TBD. Block 4 must confirm whether §3.6 covers the revocation cascade before Phase 8 commits to M6.*

### 6.A3 Bootstrap configuration

**Phase:** 6. **Approximate verb count:** ~5. **Sketches in Pass 2 §A3.**

*Block 4 — TBD.*

### 6.A4 Space and Room operator actions

**Phase:** 9. **Approximate verb count:** ~6. **Sketches in Pass 2 §A4.**

**Pre-phase blocker:** The signing-identity sub-design. When the Node operator force-ejects a member, what identity signs the resulting `membership.kick` event? The options (new EventType variant signed by Node keypair; existing EventType with `meta_atts` marker; separate admin keypair) need Joe-lock or explicit deferral before Phase 9 starts.

*Block 4 — TBD.*

### 6.A5 Identity registry administration

**Phase:** 5. **Approximate verb count:** ~4. **Sketches in Pass 2 §A5.**

*Block 4 — TBD. Block 4 must address the cascade question (does `identity revoke` force-eject existing memberships?).*

### 6.A6 Logging and audit administration

**Phase:** 4. **Approximate verb count:** ~5. **Sketches in Pass 2 §A6.**

This phase's `audit *` verbs use the schema and storage defined in §2.6.4 above. `audit rotate`, `audit query`, `audit export` against the SQLite source. Phase 4 lands the audit primitive that all subsequent write phases (5–10) consume.

*Block 4 — TBD.*

### 6.A7 Plugin management

**Phase:** 10. **Approximate verb count:** 2–5 depending on plugin set maturity.

*Block 4 — TBD. Block 4 must decide whether the WRITE verbs (load/configure/unload) ship in M6 or wait for a second plugin to exist beyond `NoOpTemperaturePlugin`.*

---

## 7. Out of scope for M6

The following are explicitly NOT in M6 and are deferred to specific named milestones or remain as design open items:

- **Node `--aicontrol` surface.** M7 per D-066. The `admin_ops::*` layer M6 builds will be reused by M7's `--aicontrol` dispatcher.
- **Live config reload.** Standalone M7 milestone. M6 defines the live-reload bucket (§2.6.3) so M6 verbs that touch reloadable fields know the planned future behaviour, but the reload mechanism itself ships in M7.
- **Protocol-level authentication on the `--batch` pipe.** §2.6.1 locks OS-user-equals-operator for M6 v1. M7 may revisit.
- **Per-verb authorisation gating.** §2.6.2 locks session-scoped for M6 v1. M7 may revisit alongside pipe authentication.
- **Full-args (non-hashed) audit storage opt-in.** Future config flag, post-M6.
- **Connection management category.** Disconnect specific clients, rate limits, IP bans. Deferred per Pass 3 Thread 2; re-enters roadmap if operational pain surfaces.
- **DAG / Space-storage administration category.** Compact, vacuum, force-replay, repair. Deferred per Pass 3 Thread 2; correctness of force-re-replay is non-trivial design work.
- **Auth Module Phase 8.** May defer if Block 4 surfaces §3.6 revocation cascade as a spec-gap concern.
- **Plugin Phase 10 WRITE verbs.** May defer if Block 4 confirms plugin set hasn't matured beyond the M3-era no-op temperature plugin.

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
| **D-070 (proposed)** | "Two events of equal importance, opposite direction" as named protocol principle. Drafted in §9 below. |

---

## 9. D-070 (proposed) — "Two events of equal importance, opposite direction" as a named protocol principle

**Status of this section.** Draft. D-070 is recorded here as part of Pass 3's output. Promotion to DECISIONS.md is a separate atomic action (with the D-070 number reserved). The recorded reasoning below reflects Joe's verbatim framing from Pass 3 sub-question 1.

---

### Draft text

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
