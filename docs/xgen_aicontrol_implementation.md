# XGen `--aicontrol` — Reference Implementation Specification
> **Status**: ACTIVE  
> Version: 1.6  
> Date: May 2026  
> **Last updated**: 2026-06-02  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools. This document consolidates the architectural commitment recorded in DECISIONS.md D-066 with the technical detail originally drafted as the Chat Claude addendum inside `tasks/BATCH_FLAG_review.md`, extended to cover both binaries.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

> **XGID discipline notice.** XGID discipline (DECISIONS.md D-072, `docs/xgen_appendix_j_en.md`) applies to all identifiers carried by the `--aicontrol` wire format documented here. The five wire-format invariances of Ch3 §3.0.3 — field names, field types (string), canonical form, URI grammar, and string-equality semantics — bind this surface equally to the federation wire. As of **Retrofit Pass 4**, the xgen-client AI-resident surface carries identifier slots (`ai_identity_id` on `EventContext`, the per-Space pacing key) as typed XGID flavours **in memory** (`IdentityXgid` / `SpaceXgid`) and **plain `String` on the wire** via serde-transparency. This is Pass 4's typed-XGID annotation scope only — the M7 `--aicontrol` v1 protocol redesign is a separate milestone and does not ride on this annotation.  

---

> **🟢 IMPLEMENTATION STATUS — M7 `--aicontrol` v1 SHIPPED (command-pipes-only), J-205 (2026-06-01).** The command-pipe surface shipped across three code commits: **C1** shared substrate (`xgen-common/src/aicontrol/` — AC-D2 envelope, AC-D1 `cmd` resolver, §5 bindings, AC-D3d codes, AC-D3a timeouts; J-201), **C2** client command pipe (`xgen-client/src/aicontrol.rs`, wraps `ops::*`; J-202), **C4** node command pipe (`xgen-node/src/aicontrol.rs`, wraps `admin_ops::*`; J-204). Each is a **sister** to the existing `--batch` pipe (D-066: `--batch` untouched) and an **adapter** (D-065: no new business logic, no verbs beyond the shipped `ops::*`/`admin_ops::*`).
>
> **🟢 EVENT PIPE — M7-events arc SHIPPED (J-212, 2026-06-01).** The event-observation pipe of §3 (deferred from M7 v1 at J-203) shipped across five code commits on top of the gating Node multi-connection-per-identity fan-out change: **C1** `ClientSenders` → `Vec<(ConnId, Sender)>` retype (J-207), **C2** the pure `Filter`/`parse`/`matches` substrate in `xgen-common::aicontrol::filter` (J-208), **C3** the node observer registry consulted in `apply_fanout` + node `state` count (J-209), **C4** the node `.events` pipe `events_pipe.rs` — the registry writer (J-210), **C5** the client `.events` pipe — a second same-identity WS riding the C1 retype (J-211). Locks `EV-D1`–`EV-D6` (`tasks/M7_EVENTS_DESIGN.md`, arc-local per D-069). See the §3 SHIPPED banner for as-built deltas. C5's `subscribe`→second-WS→forward happy path — component-tested at ship — is now covered end-to-end by the client + node `.events` integration tests (J-229).
>
> **🟢 M7-COMPLETION CLUSTER SHIPPED (J-223, 2026-06-01).** The `--aicontrol`-shaped remainder of M7 closed across six code commits (locks `M7C-D1`–`M7C-D4`, `tasks/M7C_COMPLETION_DESIGN.md`, arc-local per D-069). **Block A — client-feature (AC-D5 no longer deferred):** `ops::members` (J-217) + the key-less `SpaceState::from_dm_space_create_node` constructor · `ops::leave` (J-218) · `ops::create_dm_space` (J-219) + a node-side `StateDmSpaceCreate` ingest arm. **Block B — hardening:** AC-D4 per-connection token (J-220) · AC-D6 idempotency key (J-221). **Block C:** the `nodes` filter `ordered_nodes` widening (J-222). As-built deltas in §6 / §8 / the resolution map (§12.2). Suite at close: `cargo test --workspace` **965** / 0 / 1.
>
> **As-built deltas vs this spec (D-065 honest):**
> 1. **`CONCURRENT_COMMAND_NOT_ALLOWED` (§8)** is a wired safety-net that is **structurally non-firing in v1** — the sequential per-connection handler reads the next line only after the current reply is written (serial by construction); the rejection path is reserved for a future pipelined handler.
> 2. **Marshaling asymmetry.** The **client** arm reconstructs an argv and reuses the existing clap parser (its `ops::*` Args are all `--flag`); the **node** arm uses `serde_json::from_value` on `Deserialize`-derived `admin_ops::*` Args (node verbs mix **positional** required IDs + **flag** options, so reconstruct-argv does not port). The mechanism differs because the surfaces do.
> 3. **§7.1 node surface.** The node `--aicontrol` surface = the `admin_ops::*` verbs + `state`, **not** the 7 M2 print-only reads (`status`/`connections`/`peers`/`spaces`/`whoami`/`version`/`identity list` are `app::cmd_*` with no structured Result; `state` + the structured admin reads cover the ground).
> 4. **§9 `state`.** Node `state` **drops** `operator_display_name` (not in local config — see `cmd_whoami`); keeps `uptime_seconds`/`active_connections`/`registered_identities`. `event_subscriptions` shipped honest **`0`** on both binaries in M7 v1; the **M7-events arc** (J-209/J-211) made it the **live process-wide subscription count** (node = observer-registry `len()`, client = active `.events`-session count).
>
> Suite at close: `cargo test --workspace` **898** passed / 0 failed / 1 ignored. Locks: `tasks/M7_AICONTROL_DESIGN.md` (AC-D1–AC-D6). Build plan: `tasks/M7_AICONTROL_IMPL.md`.

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
- **`xgen-node --aicontrol`** — Node-administration actions. Federation management, Auth Module management, Bootstrap configuration, Space and Room admin actions, identity registry administration, logging and audit administration, plugin management. The set is larger and substantially new (most verbs do not exist as `--batch` commands today; the M6 milestone in the current roadmap ships the underlying admin write path that `--aicontrol` will wrap).

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

> **🟢 SHIPPED — M7-events arc (J-207…J-211, 2026-06-01).** The event pipe shipped on both binaries. The original blocker (a dedicated `.events` WS colliding with the Node's one-sender-per-identity `ClientSenders`) was closed by the **C1 retype** `ClientSenders` → `HashMap<IdentityXgid, Vec<(ConnId, Sender)>>` (EV-D2); the rest is adapter work over it. **As-built deltas vs the §3 design below (D-065 honest):**
>
> - **Filter substrate (EV-D4 v1.1).** Grammar is the shared pure `matches(&Filter, &Event, event_nodes: &[NodeXgid]) -> bool` in `xgen-common::aicontrol::filter` (D-067 single source of truth). The lock was **amended v1.0 → v1.1**: the literal 2-param `matches(&Filter, &Event)` was unimplementable for the `nodes` dimension (an `Event` carries no uniform node field), so the runtime-sourced node set is **caller-supplied** — the node derives `event_nodes` from `SpaceState.home_node` + `federation_nodes` + `content["node_id"]` + sender-for-node-signed-types; the **client passes `&[]`** and rejects a non-empty `nodes` filter with `BAD_ARGUMENT`.
> - **Node observation grain (EV-D3 + EV-D5).** The node taps `apply_fanout` (the **superset chokepoint** — every accepted event, local + federation-received) via a **process-global observer registry** (Shape β, J-166 precedent — not a threaded param); `FederationPeerSenders` stays single-sender and is out of scope. Grain is fan-out-*output* (events delivered to members), not the accept/persist chokepoint.
> - **Live-only, no history (Q2).** Subscription is from-now-forward; the `.events` drain forwards only `Event`s (ignores `HistoryBatch`/`SyncComplete`). History stays the command pipe's job. Gaps are visible across reconnect (cap-1024 drop-on-full).
> - **`event_subscriptions` (EV-D6).** Live process-wide count — node = observer-registry `len()`, client = active `.events`-session count (was honest `0` in M7 v1).
> - **Client side (EV-D3).** The client opens a **second same-identity WS** to its home Node (riding the C1 retype), tails it, and filters **at the drain** (`matches`, `event_nodes = &[]`).
> - **Pipe name.** `…\<base>.aicontrol.events` (namespaced under the aicontrol surface), both binaries.
>
> Locks: `tasks/M7_EVENTS_DESIGN.md` (`EV-D1`–`EV-D6`, arc-local per D-069). Build plan: `tasks/M7_EVENTS_IMPL.md`. The §3 design below is the as-designed spec; the deltas above are authoritative where they differ.

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

**Filter grammar (locked, AC-D3b):** **AND across fields, OR within a field** — an event passes iff it matches every present dimension. **Empty == omitted == no restriction** on all three fields (`spaces:[]` means all entitled Spaces, not "match nothing"). **Wildcards (`event_types`): exactly two forms** — bare `*` (all) and a trailing-segment prefix `state.*` (raw prefix on `EventType::as_str()` with the `.` retained so the segment boundary is respected); leading/mid wildcards → `BAD_ARGUMENT`. **Entitlement is the ceiling** — the filter is a *view*, never broadens; an out-of-entitlement `spaces` entry is inert (yields nothing, never errors). **`nodes` is Node-side only** — on the Client → `BAD_ARGUMENT` (loud, not silent). Malformed filter → `BAD_ARGUMENT` before streaming starts.

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
- `stage` (optional, string): the verb failure stage, present only for Node verb errors (`AdminError`). One of the 6 shipped `Stage` values: `validate · authorize · register · persist · notify · federate`.

**Locked (AC-D2) — flat envelope.** Mandatory: `code` · `category` · `message` · `instance_state`. Optional-by-source: `stage` · `hint` · the band code. **`category` is a closed enumerated set** (`protocol · lifecycle · argument · connection · timeout · permission`) and **alone disambiguates the `code` namespace** — the driver branches on `category`, never parses `code`. Per source: a **Node verb error** carries the band code (e.g. `SPACE_8005`) + `stage`, `category: protocol`; a **client verb error** is message-only (`code: GENERIC_4000`, no `stage`, `category: protocol`) because `ops::*` is `anyhow`-based (lossiness accepted, out of M7 scope; the envelope is forward-compatible with structured client errors); a **control-surface error** uses an uppercase-snake code (§8) with its matching category.

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

**SHIPPED (AC-D5, M7-completion cluster Block A, J-217–J-219):** `members`, `leave`, and `create-dm-space` now have `ops::*` backing and are exposed on `--aicontrol`. `members` (lift over the `ai_status` history-drain, covers DM Spaces via the key-less `from_dm_space_create_node` constructor) and `leave` (mirrors `join`) are pure adapter; `create-dm-space` is the one non-pure-adapter verb (M7C-D4) — the client sends the DM's three-event causal chain (`dm_space_create` ← `room_create` ← `invite`, in order over one connection) and the node gained a `StateDmSpaceCreate` ingest arm that builds the DM `SpaceState` key-less from the root. **As-built notes:** membership rides the **root** (`from_dm_space_create_node` seeds `members` + `pending_invites` from `content["invitee"]`), not the auto-invite; DMs are single-homed (federation disabled), so there is no federation push; the invitee-join-across-nodes discovery flow is out of cluster scope (the cluster forms creator-home-Node state only).

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
| `create-dm-space` | `invitee` | — | `space_id`, `room_id`, `event_id`, `invitee` | `space_id` |
| `create-room` | `space`, `name` | `topic` | `room_id`, `event_id` | `room_id` |
| `spaces` | — | — | array of `{space_id, name, role}` | — |
| `rooms` | `space` | — | array of `{room_id, name, topic}` | — |
| `members` | `space` | — | `space_id`, `is_dm`, `owner_id`, `members[]` of `{identity_id, role, invited_by, joined_at}` | — |

### 6.3 Membership verbs

| Verb | Args (required) | Args (optional) | Data on success | Bind value |
|---|---|---|---|---|
| `invite` | `space`, `target_identity` | `role` | `event_id` | `event_id` |
| `join` | `space` | — | `event_id` | `event_id` |
| `leave` | `space` | `room` | `event_id`, `space_id`, `room_id` | `event_id` |

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

**Verb naming is locked by AC-D1: the `cmd` field is the CLI command path minus the binary name** (e.g. `{"cmd":"federation accept"}`), split on the first space into `[category, verb]`. There is no separate AI-control naming layer — the verb list is whatever the shipped clap surface exposes. M6 has shipped; the **as-built** categories below supersede the original flat `federate-accept`-style sketches. (Read-verb names in §7.1 likewise follow the CLI path; the exact M2 read subset reconciles at the M7 runbook.)

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

Accept/reject incoming federation requests, initiate federation with a peer, defederate, set per-peer allow/deny policy. **As shipped (M6):** `federation accept` · `federation reject` · `federation initiate` · `federation defederate` · `federation set-policy` · `federation show-policy` · `federation list`. (`defederation-signal` was not shipped.)

### 7.3 Auth Module management verbs (M6 scope)

Register a new Auth Module, revoke trust, change accepted Tiers, list, connectivity-test. **As shipped (M6):** `auth-module register` · `auth-module revoke` · `auth-module set-tiers` · `auth-module list` · `auth-module test`. (`auth-module` is a single hyphen-internal category token — AC-D1.)

### 7.4 Bootstrap configuration verbs (M6 scope)

Register/deregister with Bootstrap Nodes, change self-info metadata, update advertised tiers, show state. **As shipped (M6):** `bootstrap register` · `bootstrap deregister` · `bootstrap set-info` · `bootstrap show` · `bootstrap set-tiers`. (`set-info` args: endpoint/region/capability.)

### 7.5 Space and Room admin actions (M6 scope)

For Spaces hosted by this Node — force-eject (Node-administrator authority), set/show Node-level policy, unban, list hosted, audit-events, audit-rebuild. **As shipped (M6):** `space force-eject` · `space set-node-policy` · `space show-node-policy` · `space unban` · `space list-hosted` · `space audit-events` · `space audit-rebuild`. (`migrate-start` deferred — A4-D2.)

### 7.6 Identity registry administration verbs (M6 scope)

Show, revoke a registration (with audit trail), update an Identity's stored Trust Assertion expiry, manage replica relationships. **As shipped (M6):** `identity show` · `identity revoke` · `identity set-trust-expiry` · `identity manage-replica`.

### 7.7 Logging and audit administration verbs (M6 scope)

Query/export/archive the audit log, set + show log levels per module at runtime. **As shipped (M6):** `audit query` · `audit export` · `audit archive` · `log set-level` · `log show-level`. **`config-reload` is NOT part of `--aicontrol` core** — it routes to the separate M7-standalone (live config reload) milestone; see §11.

### 7.8 Plugin management verbs (M6 scope)

Query plugin status. **As shipped (M6):** `plugin list` · `plugin status` only. The write verbs (load/configure/unload) are deferred until a second plugin exists (A7-D1). The temperature plugin's runtime surface (per D-061) lands with that arc.

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
| `PERMISSION_DENIED` | permission | The session lacks the authority to execute this command. **Activated for the AC-D4 token gate (M7C-D1, B1)** — returned when a present control token does not match the resident's expected token. The gate ships **inert** in v1 (no expected token configured → `absent==proceed`); a real enforcement source + per-verb gating land with the privilege-model arc. |
| `MALFORMED_COMMAND` | argument | The line is not valid JSON or has no `cmd`. The reply omits the echoed `cmd`/`id` (nothing to echo). |

All control-surface codes are uppercase snake-case strings to distinguish them from numeric protocol codes at a glance. **Locked (AC-D3d):** this catalogue of 9 is **closed for v1** — new codes are deliberate additions. **Invariant:** control-surface codes never use category `protocol`; `protocol` is verb-sourced only (band codes / client `anyhow`). There is **no `INTERNAL` code in v1** — unexpected faults surface as the verb's `GENERIC_4000` (category `protocol`).

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

**Locked (AC-D3c).** `state` is a control verb running **in-process**, composing from **live runtime state** (it sees more than the file-reading `status` op). Two guardrails: **no new instrumentation** (any field above not already tracked is dropped to a follow-up, not built — adapter scope) and **purely local** (read-tier, 5 s — no network round-trip). The confirmed/control-owned core is locked; `connected_since` + per-space `member_count`/`room_count` (Client) and `uptime_seconds`/`active_connections`/`registered_identities` (Node) are **confirm-at-pickup** — kept iff already cheaply available, else dropped. `bindings` + `event_subscriptions` are control-owned and always present.

---

## 10. Timeout and cancellation

- **Per-command timeout** is part of the command's `args` block as an optional `timeout_ms` field. The driver can override per command.
- **Default per command (locked, AC-D3a)** — a 3-tier class rule pinned by name to the shipped constants: **5 s** read/local (`AUTH_MODULE_PROBE_TIMEOUT_SECS`), **30 s** write/network (`PENDING_TIMEOUT_SECS`), **180 s** federation peer interaction (`FEDERATION_RELATIONSHIP_TIMEOUT_SECS`). Standing invariant: the default is always **≥ the verb's own internal timeout** (else the guard masks legitimate slow completion as a false `TIMEOUT`). Tier is class-derived (READ/WRITE + a federation flag); new verbs inherit their tier — no per-verb table. The `timeout_ms` override is honored as-is (no clamp-up) and floor-validated → `BAD_ARGUMENT` on a non-positive value.
- **On timeout**, the instance returns `error.code == "TIMEOUT"`, `error.category == "timeout"`. The command may or may not have actually executed remotely — the timeout is a local guard, not a remote cancel. For idempotent commands (`whoami`, `status`, `state`, all `*-list` reads), retry is safe. For non-idempotent commands (`send`, `create-space`, write-path Node verbs), the driver must reconcile via subsequent state queries.
- **No explicit cancel command.** A driver wanting to cancel an in-flight command closes the pipe connection. The instance treats connection close as cancellation of any in-flight command and cleans up locally; remote side-effects may already have happened.

This is the simplest model that handles the realistic failure modes without inventing a request-tracking layer the driver doesn't need.

---

## 11. Live config reload — the heart of admin-during-runtime

A use case explicitly named in the Node admin design discussion (2026-05-17): the Node operator changes something in `xgen-node_config.toml` while the Node is running and wants the change to take effect without restart.

Today this is partially specified. `--reload-config` exists as a fundamental flag (Appendix F §F.0.1) but the Node returns honest `NOT_IMPLEMENTED` when invoked. **Routing correction (M7 design):** `config-reload` / live-reload is **not part of `--aicontrol` core** — it is its own milestone, **M7-standalone (live config reload)**, and did **not** ship in M6 (still `NOT_IMPLEMENTED`). This section is retained as the co-design sketch; the implementing milestone is M7-standalone, not M7-`--aicontrol`.

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

### 12.1 M6 deliverables (gate before M6 goes ACTIVE) — ✅ SUPERSEDED (M6 shipped, J-197)

> All items below were closed by M6 shipping: verbs enumerated in `admin_ops::*` (§7.2–§7.8 as-built), privilege model = D-082 (OS-user-equals-administrator), audit integration via `ActorVia`, the `admin_ops::*` shape realised. Live-reload is **not** an `--aicontrol` item — it routes to M7-standalone (see §11). The "gate before M6 ACTIVE" framing is moot.

- **Verb-set enumeration.** Final list of verbs in §7.2 through §7.8 with their `args` and `data` schemas. Probably 30+ verbs total.
- **Privilege model.** Which verbs require what proof of Node-operator authority — Node keypair, separate admin keypair, OS-user identity over the pipe, or pipe-access-equals-operator assumption. Today the pipe is unauthenticated on the assumption that pipe-access on the same machine equals operator-authority; whether this holds for write-path verbs is the M6 question.
- **Live-reload semantics in full.** Which fields go in which bucket (live-reloadable vs restart-required), rollback path, validation rules.
- **Audit trail integration per verb.** Which verbs produce which audit-log entries; schema additions if needed beyond §3.11.8.
- **`xgen-node-lib::admin_ops::*` shape.** The shared command implementation layer that all three dispatchers (CLI arm, `--batch` arm, `--aicontrol` arm) call. Same pattern as `xgen-client-lib::ops::*` from M5.

### 12.2 M7 deliverables (gate before M7 goes ACTIVE) — 🔒 RESOLVED (M7 design, `tasks/M7_AICONTROL_DESIGN.md`)

> Resolution map: per-command timeouts → **AC-D3a**; subscription-filter grammar → **AC-D3b**; `state` schema → **AC-D3c**; control-surface error codes → **AC-D3d**; pipe-level auth → **AC-D4** (OS-ACL + the per-connection token, **shipped inert at M7C B1, J-220** — opaque top-level `Command.token`, `absent==proceed`, `Some(invalid)`→`PERMISSION_DENIED`; enforcement source = the privilege-model arc); replay-safety → **AC-D6** (**shipped at M7C B2, J-221** — opaque `Command.idempotency_key`, per-`.aicontrol`-session result-time dedupe, `absent==do-it-over`; FIFO-bounded per-connection store). Plus AC-D1 (`cmd` verb model) + AC-D2 (flat envelope) + **AC-D5 (the three client verbs `members`/`leave`/`create-dm-space` shipped at M7C Block A, J-217–J-219)**. The bullets below are retained as the original framing.

- **Per-command default timeout values.** §10 says "conservative defaults"; each command needs an actual number.
- **Subscription filter grammar.** §3 sketches it; the grammar needs a formal definition (precedence of `spaces` vs `event_types` vs `nodes` filters, wildcard semantics, the empty-filter case).
- **The `state` command's full output schema.** §9 sketches both binary's variants; the schemas need to be locked.
- **Control-surface error codes.** §8 lists the categories and the v1 codes. New codes added as commands are added; the full catalogue is part of M7's lock.
- **Pipe-level authentication policy.** Today's "pipe-access equals operator-authority" assumption is defensible for single-user dev boxes but becomes a question when MCP servers run as different OS users than the human user, or when multiple AI drivers share access to one Identity intentionally and want audit trails per driver. The natural primitive is per-connection authentication via a token established when the AI driver and the human user paired. Recorded here so future work has a starting point; not designed in v1.
- **Replay safety policy.** ~~Today's behaviour: fresh connection = fresh binding namespace, no idempotency keys, do it over.~~ **SHIPPED (M7C B2, J-221):** each command may carry an opaque driver-supplied `idempotency_key`; the per-connection handler remembers recently-seen keys (FIFO-bounded, `DEFAULT_IDEMPOTENCY_CAP = 1024`) and returns the original reply for a duplicate. **Result-time binding:** only a completed, successful command is recorded — an errored/crashed command records nothing, so a replay re-does it. `absent==do-it-over`. Scope is per-`.aicontrol`-session (the store is a per-connection local); widening to per-driver-identity is a placement change (the wire field is unchanged), deferred to the privilege-model arc.

---

## 13. Sequencing

The revised plan after the 2026-05-17 roadmap update:

1. **M5 ✅ (shipped).** `xgen-client-lib::ops::*` refactor — the shared command implementation layer for the Client.
2. **CLI Precedence Audit ✅ (shipped).** Flag-vs-config precedence verified across both binaries.
3. **M6 (new) — Node admin write path** (current PENDING). Ships `xgen-node-lib::admin_ops::*` plus the read-write verb set across federation management, Auth Module management, Bootstrap configuration, Space/Room admin actions, identity registry administration, logging and audit administration, plugin management. The Node-side `--batch` becomes read-write through these verbs; the Node side of `--aicontrol` becomes possible because the underlying surface exists.
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
