# XGen `--aicontrol` — Reference Implementation Specification
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools. This document consolidates the architectural commitment recorded in DECISIONS.md D-066 with the technical detail originally drafted as the Chat Claude addendum inside `tasks/BATCH_FLAG_review.md`, extended to cover both binaries.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

> **XGID discipline notice.** XGID discipline (DECISIONS.md D-072, `docs/xgen_appendix_j_en.md`) applies to all identifiers carried by the `--aicontrol` wire format documented here. The five wire-format invariances of Ch3 §3.0.3 — field names, field types (string), canonical form, URI grammar, and string-equality semantics — bind this surface equally to the federation wire. As of **Retrofit Pass 4**, the xgen-client AI-resident surface carries identifier slots (`ai_identity_id` on `EventContext`, the per-Space pacing key) as typed XGID flavours **in memory** (`IdentityXgid` / `SpaceXgid`) and **plain `String` on the wire** via serde-transparency. This is Pass 4's typed-XGID annotation scope only — the M7 `--aicontrol` v1 protocol redesign is a separate milestone and does not ride on this annotation.  

---

## What this document is — and what it is not

This document specifies `--aicontrol`, the AI-driver control surface for the XGen reference Node and Client implementations. **It is not part of the XGen Protocol.**

The XGen Protocol is what travels on the wire between XGen participants: between a Client and its home Node, between two federated Nodes, between MLS group members. Chapter 3 (`docs/xgen_ch3_specification.md`) is the authoritative spec for that protocol. Appendix I (`docs/xgen_appendix_i_en.md`) is the authoritative spec for the data structures carried on it.

`--aicontrol` is none of these. It is a local control channel between an AI driver and a specific `xgen-node.exe` or `xgen-client.exe` instance running on the same machine, carried on a Windows named pipe. It never reaches any XGen wire. A network monitor watching XGen federation traffic would never see an `--aicontrol` message — there is nothing to see, because `--aicontrol` does not cross the network.

A compliant XGen Node or Client implemented in a different language, by a different team, may ship a completely different AI-control surface — gRPC, REST, an MCP server, raw stdin/stdout, or no AI-control surface at all — and remain fully protocol-compliant. The XGen Protocol does not constrain how a vendor builds their local automation surface.

This document specifies one such surface — the reference implementation's. It belongs alongside Chapter 4 (Implementation) and Appendix F (CLI Reference), not alongside Chapter 3 (Specification) or Appendix I (Data Structures).

The architectural commitment to split `--batch` (preserved verbatim for humans and human-readable automation) from `--aicontrol` (new AI-driver surface) is recorded in **DECISIONS.md D-066**. The discipline rules governing how this document evolves are recorded in **DECISIONS.md D-069**. This document is the technical specification under those decisions.

---

## 1. Background and architectural commitment

### 1.1 Two flags, two audiences

The current `--batch` surface (shipped in M2 for the Node, M1 for the Client) is a fire-and-forget script runner. A `.xgb` file is a static sequence of commands; the driver runs them, collects `OK\n` / `ERROR: <message>\n` per line, exits. Plain-text replies make `--batch` easy to read in a terminal, easy to pipe into `tail -f`, easy for a human operator to copy-paste from documentation into a shell.

That same plain-text shape is the worst case for an AI driver. AI drivers work best with structured data. AI drivers want persistent sessions, not per-command process spawns. AI drivers want to observe events in real time, not poll history. AI drivers want named bindings so a multi-step scenario doesn't require log-scraping to find the IDs of objects it just created. AI drivers want lifecycle-aware error replies that include the instance's current state, not just the lowest-level error string.

Rather than evolve `--batch` to meet AI-driver needs (degrading the human-readability that was its original design goal), the architectural commitment is to introduce a **second flag** designed from the start for AI drivers:

- **`--batch`** — preserved verbatim. Plain-text replies. Fire-and-forget script-runner shape. The surface humans, shell scripts, and CI pipelines use today.
- **`--aicontrol`** — new. Persistent control session. JSONL command/reply protocol. Named bindings. Real-time event observation. Lifecycle-aware errors. The surface AI drivers (and MCP servers, and any tool wearing the same shape) use.

Both flags ship on both binaries. Both flags are read-write — `--batch` was extended to read-write on the Client in M1; the Node side received only a read-only subset in M2 and needs the write path filled in (M6 in the current roadmap). `--aicontrol` is read-write on both binaries from day one.

### 1.2 Why this split is structural, not stylistic

The boundary protects three things:

**Boundary 1 — Backward compatibility for humans.** Every operator who has a shell script, a CI job, a personal automation snippet calling `xgen-client --batch script.xgb` or `xgen-node --batch verify.xgb` continues to work without change. The format they expect (plain-text replies, file-driven invocation, exit codes) is preserved verbatim.

**Boundary 2 — Forward optimisation for AI drivers.** `--aicontrol` is designed against the actual needs of AI drivers as identified in `tasks/BATCH_FLAG_review.md` §1–§6: persistent connection (eliminates F-003/F-004 class of bugs), structured replies (no log-scraping), real-time event observation (the only honest way to measure delivery latency), named bindings (no two-pass script generation), unified command handlers (the `ops::*` refactor shipped in M5), lifecycle-aware errors.

**Boundary 3 — Implementation-not-protocol separation.** The XGen Protocol stays untouched. `--aicontrol` lives entirely below the wire, between an AI driver and a single binary on the same machine. A future MCP server bridging XGen to a chat AI would consume `--aicontrol` as a client; the protocol it speaks to its AI customer (MCP, JSON-RPC, whatever) is the MCP server's choice, not XGen's concern.

### 1.3 Two binaries, symmetric surface

`--aicontrol` applies uniformly to both binaries. The fundamental flag surface specified in Appendix F §F.0.1 has been deliberately symmetric since M1 (D-063); `--aicontrol` follows the same symmetry. What differs between the two binaries is the **verb set**, not the surface shape.

- **`xgen-client --aicontrol`** — Identity-side actions. Register, create-space, create-room, invite, join, send, history, whoami, status, spaces, rooms, members, federate, AI operator management, etc. Roughly the existing CLI subcommand set, translated into JSONL.
- **`xgen-node --aicontrol`** — Node-administration actions. Federation management, Auth Module management, Bootstrap configuration, Space and Room operator actions, identity registry administration, logging and audit administration, plugin management. The set is larger and substantially new (most verbs do not exist as `--batch` commands today; the M6 milestone in the current roadmap ships the underlying admin write path that `--aicontrol` will wrap).

The pipe naming, the JSONL command/reply protocol, the error code shape, the persistent session model, the binding mechanism, the timeout/cancellation rules, the `state` command, the event observation channel — all of these are **shared**. One canonical document, two binary-specific verb sets.

---

## 2. Surface shape

### 2.1 Naming and invocation

**Flag name:** `--aicontrol`. Locked. Visible at the CLI surface of both binaries.

**Mode, not script runner.** Unlike `--batch`, which loads a `.xgb` file and runs it to completion, `--aicontrol` opens a persistent control session over a named pipe. No `--aicontrol <script>` file-loading variant. Scripts are fed via stdin redirection if a non-interactive driver needs them. The session lives as long as the pipe connection lives.

**File extension** `.aib` (for *AI batch*) is reserved by convention for input files driven via redirection, but the runtime does not enforce or check extensions — the pipe sees bytes, not files. This matches the principle that `--aicontrol` is a live session, not a script runner.

**Invocation examples:**

```
xgen-client --aicontrol                          # interactive session on default-instance pipe
xgen-client --instance alice --aicontrol         # session against instance-labelled pipe
xgen-node --aicontrol                            # Node admin session
xgen-node --instance n1 --aicontrol < setup.aib  # non-interactive feed via stdin
```

### 2.2 Pipe naming — sister pipe per binary

The existing `--batch` pipes (M1 for Client, M2 for Node) stay verbatim:

```
\\.\pipe\xgen-client[-<instance>]
\\.\pipe\xgen-node[-<instance>]
```

A new sister pipe lands alongside each binary's legacy pipe for `--aicontrol`:

```
\\.\pipe\xgen-client[-<instance>].aicontrol
\\.\pipe\xgen-node[-<instance>].aicontrol
```

Why sister pipe and not multiplexed-same-pipe:

- The two protocols are genuinely different (line-oriented text vs JSONL). Multiplexing them by first-line sniffing introduces a parse-the-first-byte branch that adds nothing.
- Sister pipe lets `--batch` and `--aicontrol` evolve independently. Future changes to one cannot regress the other.
- An audit-conscious operator can lock down `--aicontrol` access with a more restrictive ACL than `--batch` without affecting the legacy surface.
- Two pipe names per binary to remember is trivial cost; the deployment shape is symmetric (the legacy pipe is already named, the sister pipe just appends `.aicontrol`).

This extends D-043 (named pipe naming convention). Recorded in D-066's relationship-to-other-decisions table.

### 2.3 Concurrency model — strictly serial per connection, multiple connections allowed

One pipe connection = one in-flight command at a time. The driver fires command N, waits for the reply, then fires command N+1. No request IDs, no out-of-order reply matching, no fancy correlation.

Drivers needing concurrency open multiple pipe connections. The long-lived instance accepts repeated connections (per D-043's composability principle); each connection is an independent session with its own variable-binding namespace.

Why strictly serial:

- **Simpler protocol.** No request ID field on every command, no reply demultiplexing in the driver, no "which command failed?" ambiguity.
- **Matches the actual workload shape.** AI drivers think sequentially — "create the Space, then create the Room, then send a message." The natural model is sequential.
- **Event observation runs on its own channel** (§3 below), so events don't serialise behind commands.
- **Drivers wanting parallelism** (e.g. fan multiple sends across different Spaces simultaneously) open multiple connections. The instance handles each in parallel without protocol-level concurrency in any one connection.

---

## 3. Event observation — dedicated event pipe

A third pipe surface per binary, alongside the legacy `--batch` pipe and the `--aicontrol` command pipe:

```
\\.\pipe\xgen-client[-<instance>].events
\\.\pipe\xgen-node[-<instance>].events
```

Why a third pipe rather than multiplexing on the `--aicontrol` command pipe:

- **Back-pressure isolation.** A slow event consumer (e.g. an AI driver doing LLM inference between event batches) blocks only its own event stream, not the command channel.
- **Independent ACL.** Event observation may need a different access policy than command issuance (e.g. an audit module subscribes to events but cannot issue commands).
- **Multiple subscriptions per binary.** The same Identity (or Node operator) can have one command session and zero-to-N event subscriptions, each with its own filter.

**Subscribe/unsubscribe model.** The first message on the event pipe is a subscribe command:

```json
{"cmd":"subscribe","filter":{"spaces":["xgen://hash/sha256:abc...","xgen://hash/sha256:def..."],"event_types":["message.text","state.*"]}}
```

Filter fields:

- `spaces` (optional, array of hash_uri): Empty or omitted means "all Spaces this Identity is a member of" (Client) or "all hosted Spaces" (Node).
- `event_types` (optional, array of strings): Accepts wildcards (`state.*`, `membership.*`, `*`). Omitted means all event types.
- `nodes` (optional, array of pubkey_uri, Node-side only): Restricts the event stream to events involving the named federated peers. Useful for compliance modules monitoring a specific federation relationship.

Subsequent events are streamed as JSONL until the connection closes.

**Event record shape:**

```json
{
  "type": "event",
  "event": { /* full Event object as per Appendix I */ },
  "received_at": "2026-05-17T10:00:00.000Z",
  "space_id": "xgen://hash/sha256:..."
}
```

The `received_at` field is the instance's local-time observation timestamp — distinct from the Event's `timestamp` (sender clock) and from any future home-Node delivery timestamp. The honest delivery-latency metric (sender → observer) is `received_at - event.timestamp`, captured for the first time without log-scraping.

**Non-Event signals** (lifecycle transitions, connection state changes, plugin signals) ride the same pipe with `"type":"signal"`:

```json
{"type":"signal","name":"home_node_disconnected","reason":"transport.goodbye","timestamp":"..."}
{"type":"signal","name":"home_node_reconnected","timestamp":"..."}
{"type":"signal","name":"lifecycle","state":"degraded_federation","timestamp":"..."}
{"type":"signal","name":"federation_request_pending","peer_node_id":"xgen://pubkey/ed25519:...","space_id":"xgen://hash/sha256:...","timestamp":"..."}
```

The last signal in the list above is a Node-side example — a federation request landing while the operator's `--aicontrol` driver is connected lets the driver react in real time. Client-side equivalents include incoming invitations, role changes, mention events.

---

## 4. Command/reply protocol — JSONL

### 4.1 Command shape

Every command from driver to instance is one JSON object on one line, terminated by `\n`:

```json
{"cmd":"create-space","args":{"name":"Test Space"}}
{"cmd":"send","args":{"space":"$space","room":"$room","text":"hello"}}
{"cmd":"federate-add","args":{"peer_node_id":"xgen://pubkey/ed25519:...","peer_endpoint":"wss://node2.example.com/xgen"}}
```

Fields:

- `cmd` (required, string) — the command verb. The set is binary-specific (see §6 and §7) but the field name and shape are uniform.
- `args` (required, object) — the named arguments for the command. Argument names follow `snake_case` to match the rest of the XGen wire format (3.1.3).
- `id` (optional, string) — driver-supplied correlation ID echoed back in the reply. Useful when driver logs need to thread reply to command across the wire. Not used by the instance for routing.
- `bind` (optional, string) — names the result of this command for later substitution (§5).

### 4.2 Reply shape

Every reply is one JSON object on one line:

```json
{"status":"ok","cmd":"create-space","id":"<echoed>","data":{"space_id":"xgen://hash/sha256:...","event_id":"xgen://hash/sha256:..."}}
{"status":"error","cmd":"send","id":"<echoed>","error":{"code":"4002","category":"protocol","message":"...","instance_state":"ready"}}
```

Fields:

- `status` (required, string): `"ok"` or `"error"`. No other values.
- `cmd` (required, string): echoes the command verb.
- `id` (present iff the command included one): echoes the driver's correlation ID.
- `data` (present iff `status == "ok"`, required then): command-specific result fields. The schema per command lives in §6 (Client verbs) and §7 (Node verbs).
- `error` (present iff `status == "error"`, required then): structured error (§4.3).

### 4.3 Error shape — lifecycle-aware

Every error reply includes the instance's current lifecycle state alongside the error itself:

```json
{"error":{
  "code":"INSTANCE_NOT_READY",
  "category":"lifecycle",
  "message":"register first or complete SETUP in the UI",
  "instance_state":"setup",
  "hint":"run: register --name <your name>"
}}
```

Fields:

- `code` (required, string): the error code. Two categories:
  - **Protocol codes** — numeric codes from the existing XGen error domain (e.g. `4002` for predecessor timeout) carried as the numeric value cast to string for uniformity. Display format `E004002` is for human-readable surfaces only (per CLAUDE.md error-code convention); the wire form is the bare number-as-string.
  - **Control-surface codes** — string codes specific to `--aicontrol`: `INSTANCE_NOT_READY`, `UNKNOWN_COMMAND`, `BAD_ARGUMENT`, `BINDING_NOT_FOUND`, `CONCURRENT_COMMAND_NOT_ALLOWED`, `CONNECTION_LOST`, `TIMEOUT`, `PERMISSION_DENIED`. Documented exhaustively in §8.
- `category` (required, string): one of `protocol`, `lifecycle`, `argument`, `connection`, `timeout`, `permission`. Lets the driver branch on broad category without parsing the code.
- `message` (required, string): human-readable description. Not for programmatic parsing.
- `instance_state` (required, string): the instance's current lifecycle state at the time of the error — one of the Appendix E states for the relevant binary. Lets the driver reason about whether to retry, wait, or escalate.
- `hint` (optional, string): a suggested next command if applicable. Free-form but stable enough that drivers can match on it.

---

## 5. Variable bindings — named, mandatory

Implicit `@last_*` convenience bindings (as originally proposed in Clair's review) are not implemented in v1. The risk is unsafe defaults: creating two rooms back-to-back makes `@last_room` resolve to the second, which is rarely what the script wanted.

Decision: **named bindings are mandatory; implicit `@last_*` is not in v1.**

```json
{"cmd":"create-space","args":{"name":"Test"},"bind":"space"}
{"cmd":"create-room","args":{"space":"$space","name":"general"},"bind":"room"}
{"cmd":"send","args":{"space":"$space","room":"$room","text":"hello"}}
```

Rules:

- `bind` (optional, string on the command): names the result of this command. The binding is created when the command succeeds; on failure no binding is written.
- `$<name>` in any argument value: substitutes the named binding before dispatch. Unknown binding → error `BINDING_NOT_FOUND`.
- Bindings are scoped to the pipe connection. New connection = empty binding namespace.
- The `bind` target is whatever the command's primary return value is — `space_id` for `create-space`, `room_id` for `create-room`, `event_id` for `send`, etc. Per-command bind value listed in §6 and §7.
- For composite results (e.g. `create-space` returns both `space_id` and `event_id`), `bind:"foo"` binds `foo` to the primary return; access other fields via `$foo.event_id` dot-notation. Substring substitution inside the JSON value, simple dot notation only, no expressions.

Why not implicit `@last_*`:

- **Determinism for non-interactive scripts.** A script that creates 5 Spaces and 5 Rooms cannot reliably address them via `@last_*`; named bindings make every reference explicit.
- **Forward compatibility.** Adding `@last_*` later as a convenience layer over named bindings is easy; removing implicit `@last_*` after drivers depend on it is hard.
- **Smaller test surface.** Named-only is a simpler protocol to verify than named-plus-implicit.

---

## 6. Client verb set (`xgen-client --aicontrol`)

The Client verb set mirrors the existing `xgen-client` CLI subcommand set, translated into JSONL. Every Client verb routes through `xgen-client-lib::ops::*` (the shared command implementation layer shipped in M5/D-067) — the `--aicontrol` dispatcher is one of three callers, alongside the CLI arm (`main.rs`) and the `--batch` arm (`batch.rs`).

### 6.1 Identity verbs

| Verb | Args (required) | Args (optional) | Data on success | Bind value |
|---|---|---|---|---|
| `register` | `name` | `is_ai`, `capabilities` | `identity_id`, `home_node_id` | `identity_id` |
| `whoami` | — | — | `identity_id`, `name`, `is_ai`, `home_node_id` | — |
| `status` | — | — | full lifecycle and connection state object | — |
| `state` | — | — | session state (§9) | — |

### 6.2 Space and Room verbs

| Verb | Args (required) | Args (optional) | Data on success | Bind value |
|---|---|---|---|---|
| `create-space` | `name` | `auth_tier`, `max_event_size`, `e2e_encryption`, `human_pacing_ms`, `ai_pacing_ms` | `space_id`, `event_id` | `space_id` |
| `create-dm-space` | `invitee` | `auth_tier` | `space_id`, `event_id` | `space_id` |
| `create-room` | `space`, `name` | `topic` | `room_id`, `event_id` | `room_id` |
| `spaces` | — | — | array of `{space_id, name, role}` | — |
| `rooms` | `space` | — | array of `{room_id, name, topic}` | — |
| `members` | `space` | — | array of `{identity_id, role, is_ai}` | — |

### 6.3 Membership verbs

| Verb | Args (required) | Args (optional) | Data on success | Bind value |
|---|---|---|---|---|
| `invite` | `space`, `target_identity` | `role` | `event_id` | `event_id` |
| `join` | `space` | — | `event_id` | `event_id` |
| `leave` | `space` | — | `event_id` | `event_id` |

### 6.4 Message verbs

| Verb | Args (required) | Args (optional) | Data on success | Bind value |
|---|---|---|---|---|
| `send` | `space`, `room`, `text` | `reply_to`, `meta_atts` | `event_id`, `accepted` | `event_id` |
| `history` | `space`, `room` | `limit`, `before` | array of Event objects | — |

### 6.5 AI operator verbs (per §3.6.10.6)

| Verb | Args (required) | Args (optional) | Data on success | Bind value |
|---|---|---|---|---|
| `ai-delegate` | `space`, `ai_identity_id`, `new_operator_identity_id` | — | `event_id` | `event_id` |
| `ai-revoke` | `space`, `ai_identity_id` | — | `event_id` | `event_id` |
| `ai-status` | `space`, `ai_identity_id` | — | `operator_identity_id`, `resolution_step` (delegate / inviter / owner) | — |

Per-command default timeouts are listed in §10. Full per-command schemas (the `data` object on success, the optional argument shapes) are inherited from the existing CLI subcommands — see Appendix F §F.1–§F.9.3 for the reference shapes.

---

## 7. Node verb set (`xgen-node --aicontrol`)

The Node verb set is largely new. Most verbs do not exist as `--batch` commands today (M2 shipped only a read-only subset). The M6 milestone in the current roadmap (Node admin write path) ships the underlying `xgen-node-lib::admin_ops::*` layer; `--aicontrol` is one dispatcher of three (CLI arm, `--batch` arm, `--aicontrol` arm), same pattern as Client side.

This section sketches the verb categories. Final per-verb schemas are deliverables of M6's design phase (§12 below). Listed here so the canonical document carries the both-binaries shape from day one, not so the schemas are locked.

### 7.1 Read-only verbs (M2-shipped, available today via `--batch`, will be available via `--aicontrol` from day one)

| Verb | Description |
|---|---|
| `status` | Lifecycle and connection state object |
| `whoami` | Node ID, operator display name, advertised endpoint |
| `connections` | Currently connected clients and peer Nodes |
| `peers` | Federated peer Nodes by Space |
| `spaces` | Hosted Spaces with member counts |
| `identity-list` | Registered Identities |
| `version` | Build version, protocol version |
| `state` | Session state (§9) |

### 7.2 Federation management verbs (M6 scope)

Accept/reject incoming federation requests, initiate federation with a peer, defederate, set per-peer allow/deny policy, submit defederation signals to Bootstrap Nodes (per §3.15). Verb names sketched as `federate-accept`, `federate-reject`, `federate-add`, `federate-remove`, `federate-policy`, `defederation-signal`. Final schemas in M6 design phase.

### 7.3 Auth Module management verbs (M6 scope)

Register a new Auth Module, revoke trust, change accepted Tiers. Verb names sketched as `auth-module-add`, `auth-module-revoke`, `auth-module-set-tiers`. Final schemas in M6 design phase.

### 7.4 Bootstrap configuration verbs (M6 scope)

Register/deregister with Bootstrap Nodes, change `bootstrap_info` metadata, update advertised `auth_tiers_served`. Verb names sketched as `bootstrap-register`, `bootstrap-deregister`, `bootstrap-set-info`. Final schemas in M6 design phase.

### 7.5 Space and Room operator actions (M6 scope)

For Spaces hosted by this Node — force-eject (Node-operator authority, distinct from member-initiated kick), set Node-level moderation policy on a hosted Space, trigger Space migration as source Node. Verb names sketched as `space-force-eject`, `space-set-policy`, `space-migrate-start`. Final schemas in M6 design phase.

### 7.6 Identity registry administration verbs (M6 scope)

Revoke a registration (with audit trail), update an Identity's stored Trust Assertion expiry, manage replica relationships. Verb names sketched as `identity-revoke`, `identity-set-expiry`, `identity-replicate`. Final schemas in M6 design phase.

### 7.7 Logging and audit administration verbs (M6 scope)

Rotate audit logs, query the audit log (read), set log levels per module at runtime. The real `--reload-config` story lives here — see §11 below. Verb names sketched as `audit-rotate`, `audit-query`, `log-set-level`, `config-reload`. Final schemas in M6 design phase.

### 7.8 Plugin management verbs (M6 scope)

Load a moderation plugin, configure it, unload it, query plugin status. The home of the temperature plugin's runtime surface (per D-061). Verb names sketched as `plugin-load`, `plugin-configure`, `plugin-unload`, `plugin-status`. Final schemas in M6 design phase.

---

## 8. Control-surface error codes

Codes specific to `--aicontrol` (distinct from XGen Protocol error codes in the 1000–9999 ranges):

| Code | Category | Meaning |
|---|---|---|
| `INSTANCE_NOT_READY` | lifecycle | The instance has not finished SETUP / is not in a state to accept this command. `instance_state` field carries the actual state. |
| `UNKNOWN_COMMAND` | argument | The `cmd` verb is not recognised on this binary. |
| `BAD_ARGUMENT` | argument | Required argument missing, type mismatch, or value out of range. `message` carries the specific issue. |
| `BINDING_NOT_FOUND` | argument | A `$<name>` substitution referenced a binding that does not exist in this session. |
| `CONCURRENT_COMMAND_NOT_ALLOWED` | argument | A second command was issued on the same connection while a first command was still in-flight. The driver violated the strictly-serial model (§2.3). |
| `CONNECTION_LOST` | connection | The persistent WebSocket to the home Node (Client) or a required network resource is unavailable. The command may have side-effects pending reconciliation; the driver should poll `state` to detect recovery. |
| `TIMEOUT` | timeout | The command did not complete within its per-command timeout window (§10). For idempotent commands, retry is safe. For non-idempotent commands, the driver must reconcile via subsequent state queries. |
| `PERMISSION_DENIED` | permission | The session lacks the authority to execute this command. Reserved for Node-side admin verbs; specific privilege model is M6 design-phase deliverable. |

All control-surface codes are uppercase snake-case strings to distinguish them from numeric protocol codes at a glance.

---

## 9. The `state` command

`state` returns the instance's structured view of its own current condition. Same verb on both binaries; the `data` object differs slightly.

**Client `state` response:**

```json
{"status":"ok","cmd":"state","data":{
  "lifecycle":"ready",
  "identity_id":"xgen://pubkey/ed25519:...",
  "is_ai":false,
  "home_node":"ws://127.0.0.1:8080/xgen",
  "home_node_connected":true,
  "connected_since":"2026-05-17T10:00:00.000Z",
  "spaces":[{"space_id":"...","role":"owner","member_count":3,"room_count":2}],
  "bindings":{"space":"xgen://hash/sha256:...","room":"xgen://hash/sha256:..."},
  "event_subscriptions":1
}}
```

**Node `state` response:**

```json
{"status":"ok","cmd":"state","data":{
  "lifecycle":"running",
  "node_id":"xgen://pubkey/ed25519:...",
  "operator_display_name":"Example Community Node",
  "endpoint":"wss://node.example.org:8443/xgen",
  "auth_tiers_served":[1],
  "uptime_seconds":86400,
  "active_connections":12,
  "federated_peers":3,
  "hosted_spaces":15,
  "registered_identities":47,
  "bindings":{"peer":"xgen://pubkey/ed25519:..."},
  "event_subscriptions":2
}}
```

Key properties shared by both:

- `bindings` map exposes the current session's binding namespace. Useful for drivers debugging substitution issues.
- `event_subscriptions` count surfaces how many event pipes are currently attached for this Identity (Client) or this Node (Node-side).
- `home_node_connected` (Client) and `active_connections` (Node) distinguish "instance is up" from "instance is connected to / accepting peers." A `ready` lifecycle with `home_node_connected: false` means the Client is healthy but currently network-degraded.

---

## 10. Timeout and cancellation

- **Per-command timeout** is part of the command's `args` block as an optional `timeout_ms` field. The driver can override per command.
- **Default per command** is conservative. The per-verb default values are M6 (Node) and M7 (Client) design-phase deliverables, but the framing default is: 30 seconds for network-touching commands, 5 seconds for state-read commands.
- **On timeout**, the instance returns `error.code == "TIMEOUT"`, `error.category == "timeout"`. The command may or may not have actually executed remotely — the timeout is a local guard, not a remote cancel. For idempotent commands (`whoami`, `status`, `state`, all `*-list` reads), retry is safe. For non-idempotent commands (`send`, `create-space`, write-path Node verbs), the driver must reconcile via subsequent state queries.
- **No explicit cancel command.** A driver wanting to cancel an in-flight command closes the pipe connection. The instance treats connection close as cancellation of any in-flight command and cleans up locally; remote side-effects may already have happened.

This is the simplest model that handles the realistic failure modes without inventing a request-tracking layer the driver doesn't need.

---

## 11. Live config reload — the heart of admin-during-runtime

A use case explicitly named in the Node admin design discussion (2026-05-17): the Node operator changes something in `xgen-node_config.toml` while the Node is running and wants the change to take effect without restart.

Today this is partially specified. `--reload-config` exists as a fundamental flag (Appendix F §F.0.1) but the Node returns honest `NOT_IMPLEMENTED` when invoked. The implementation is M6 scope.

The verb in `--aicontrol` will be (sketched, M6 design-phase deliverable):

```json
{"cmd":"config-reload"}
< {"status":"ok","cmd":"config-reload","data":{
    "reloaded_fields":["logging.level","federation.allow_list"],
    "restart_required_fields":["network.listen","keypair_path"],
    "config_version":"...",
    "config_hash":"sha256:..."
  }}
```

What M6 design phase must settle:

1. **Which config fields are live-reloadable** (changes apply to the running process immediately) vs **restart-required** (changes are accepted into the persisted config but do not affect the running process until the Node restarts).
2. **Rollback path on bad config** — what happens if the new TOML is syntactically valid but operationally wrong (e.g. listen port already taken by another process). Proposed: the reload is a two-phase commit; phase 1 parses and validates, phase 2 swaps. Phase 1 failure returns the validation error without changing running state.
3. **Audit trail integration.** Every `config-reload` produces a protocol audit log entry per §3.11.8.

This verb is the practical embodiment of why M6 must ship before M7: `--aicontrol` is the AI-shape protocol wrapping the admin surface, and the live-reload mechanism is the admin surface. The protocol shape (this section) and the underlying mechanism (M6) are co-designed.

---

## 12. Open items for design phases

Items that need explicit decisions during the M6 (Node admin write path) and M7 (`--aicontrol` v1) design phases. Listed here per D-069's open-item-flagging discipline so the document's boundary between locked and delegated content is visible.

### 12.1 M6 deliverables (gate before M6 goes ACTIVE)

- **Verb-set enumeration.** Final list of verbs in §7.2 through §7.8 with their `args` and `data` schemas. Probably 30+ verbs total.
- **Privilege model.** Which verbs require what proof of Node-operator authority — Node keypair, separate admin keypair, OS-user identity over the pipe, or pipe-access-equals-operator assumption. Today the pipe is unauthenticated on the assumption that pipe-access on the same machine equals operator-authority; whether this holds for write-path verbs is the M6 question.
- **Live-reload semantics in full.** Which fields go in which bucket (live-reloadable vs restart-required), rollback path, validation rules.
- **Audit trail integration per verb.** Which verbs produce which audit-log entries; schema additions if needed beyond §3.11.8.
- **`xgen-node-lib::admin_ops::*` shape.** The shared command implementation layer that all three dispatchers (CLI arm, `--batch` arm, `--aicontrol` arm) call. Same pattern as `xgen-client-lib::ops::*` from M5.

### 12.2 M7 deliverables (gate before M7 goes ACTIVE)

- **Per-command default timeout values.** §10 says "conservative defaults"; each command needs an actual number.
- **Subscription filter grammar.** §3 sketches it; the grammar needs a formal definition (precedence of `spaces` vs `event_types` vs `nodes` filters, wildcard semantics, the empty-filter case).
- **The `state` command's full output schema.** §9 sketches both binary's variants; the schemas need to be locked.
- **Control-surface error codes.** §8 lists the categories and the v1 codes. New codes added as commands are added; the full catalogue is part of M7's lock.
- **Pipe-level authentication policy.** Today's "pipe-access equals operator-authority" assumption is defensible for single-user dev boxes but becomes a question when MCP servers run as different OS users than the human user, or when multiple AI drivers share access to one Identity intentionally and want audit trails per driver. The natural primitive is per-connection authentication via a token established when the AI driver and the human user paired. Recorded here so future work has a starting point; not designed in v1.
- **Replay safety policy.** Today's behaviour: fresh connection = fresh binding namespace, no idempotency keys, do it over. Acceptable for v1 because the typical AI driver is itself robust to per-command failure. If future work needs strong replay safety, the natural extension is **idempotency keys**: each command carries a driver-supplied `idempotency_key`; the instance remembers recently-seen keys and returns the original reply for any duplicate. Not in v1.

---

## 13. Sequencing

The revised plan after the 2026-05-17 roadmap update:

1. **M5 ✅ (shipped).** `xgen-client-lib::ops::*` refactor — the shared command implementation layer for the Client.
2. **CLI Precedence Audit ✅ (shipped).** Flag-vs-config precedence verified across both binaries.
3. **M6 (new) — Node admin write path** (current PENDING). Ships `xgen-node-lib::admin_ops::*` plus the read-write verb set across federation management, Auth Module management, Bootstrap configuration, Space/Room operator actions, identity registry administration, logging and audit administration, plugin management. The Node-side `--batch` becomes read-write through these verbs; the Node side of `--aicontrol` becomes possible because the underlying surface exists.
4. **M7 — `--aicontrol` v1** (covering both binaries). Ships the three-pipe model (legacy `--batch`, command `--aicontrol`, events), persistent control sessions, JSONL command/reply protocol, named bindings (mandatory), lifecycle-aware errors, `state` command, per-command timeout, subscribe/unsubscribe on the events pipe. Client side wraps the M5 `ops::*` layer; Node side wraps the M6 `admin_ops::*` layer.
5. **M8 — Multiparty improved pass** (A/B against present `--batch` baselines, but the present-`--batch` baselines were never captured — the metric protocol may revise the A/B framing; see M9).
6. **M9 — Multiparty Redesign.** Redesigned to measure both binaries' read-write surfaces (`--batch` and `--aicontrol`) against each other, not the original Client-only `--batch` A/B framing.

D3 (MLS) runs as an independent parallel workstream throughout.

---

## 14. Out of scope for this document

- The XGen Protocol itself. Chapter 3 is authoritative for protocol; this document does not modify or extend it.
- The data structures carried on the protocol. Appendix I is authoritative; this document references but does not redefine.
- The fundamental flag surface (`--service`, `--instance`, `--ping`, `--health`, `--stop`, `--batch`, `--aicontrol`, `--log-level`, `--quiet`, `--config`, etc.). Appendix F is authoritative; this document describes how `--aicontrol` works once invoked, not the flag's place in the broader CLI surface.
- The lifecycle state machines for the two binaries. Appendix E is authoritative; this document references the lifecycle states by name but does not redefine the state machine.
- Cross-platform pipe abstractions. Windows-first is fine for Phase 1 / Phase 2 deployment.
- The MCP server bridging XGen to chat AIs. A future MCP server would consume `--aicontrol` as a client; its own protocol to its chat-AI customer is out of scope here.
- Internationalisation of error messages. The `message` field in errors is English; localised display is a client-side concern.

---

## 15. Relationship to other documents

| Document | Relationship |
|---|---|
| `DECISIONS.md` D-066 | The architectural commitment to split `--batch` from `--aicontrol`. This document is the technical specification under that decision. |
| `DECISIONS.md` D-069 | The discipline rule governing how this document evolves: Joe-lock + open-item flagging + canonical-document rule. §12 of this document is the open-item-flagging deliverable. |
| `DECISIONS.md` D-043 | The named-pipe naming convention. §2.2 of this document extends D-043 with the `.aicontrol` and `.events` sister-pipe naming. |
| `DECISIONS.md` D-063 | Library-first dispatch + multi-mode binary shape. The three-dispatchers-one-ops-layer pattern in §6 and §7 is D-063 applied to `--aicontrol`. |
| `DECISIONS.md` D-067 | M5's `xgen-client-lib::ops::*` refactor. The Client-side `--aicontrol` dispatcher is one of three callers of `ops::*`. |
| `tasks/BATCH_FLAG_review.md` | Clair's original review of `--batch` that motivated the split. Six concrete improvement points; this document is the result of acting on points 1–6 by building a new surface rather than mutating the existing one. The Chat Claude addendum inside that file (2026-05-17, Client-only sketch) is the predecessor of this document; this document supersedes the addendum and extends it to both binaries. |
| `docs/xgen_ch4_implementation.md` | Implementation chapter. The shared `ops::*` and (future) `admin_ops::*` layers belong to Ch4's scope; this document specifies the surface that wraps them. |
| `docs/xgen_appendix_e_en.md` | Application Lifecycle States. The `instance_state` field in error replies (§4.3) and the `lifecycle` field in `state` (§9) draw from Appendix E's state names. |
| `docs/xgen_appendix_f_en.md` | CLI Reference. Appendix F §F.0.1 lists `--aicontrol` as a fundamental flag; this document specifies what happens once the flag is invoked. |

---

*End of document. Updates land here, not in `tasks/BATCH_FLAG_review.md` or in DECISIONS.md notes. The canonical-document rule (D-069 §3) is the structural fix that keeps the design from scattering again.*
