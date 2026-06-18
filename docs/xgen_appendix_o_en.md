# XGen Protocol — Appendix O: `--aicontrol` Control-Plane Data Structures
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-18  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Overview

This appendix is the canonical **data-structure reference** for the `--aicontrol` control plane — the JSON-Lines command/reply protocol an AI driver (or any program) speaks to a long-running `xgen-node` / `xgen-client` instance over its control pipe. It is the structural sibling of `xgen_aicontrol_implementation.md`, which is the **behavioural / reference-implementation specification** (the dispatch model, the lifecycle, the events pipe, the worked examples). The split mirrors ch3-vs-Appendix-I: behaviour lives in the implementation spec; the typed shapes live here.

**Source:** `xgen-common/src/aicontrol/` (`envelope`, `codes`, `cmd`, `filter`, `bindings`, `idempotency`, `timeout`, `token`). Design references are arc-local `AC-D#` / `M7C-D#`.

**Wire vs. runtime.** Two of the structures below cross the pipe as JSON (`Command`, `Reply`/`ErrorBody`/`Category`, and the `Filter` payload of a `subscribe`). The rest (`ControlVerb`/`CmdPath`/`CmdResolution`, `Bindings`/`BoundValue`, `IdempotencyStore`, `TimeoutTier`) are the **runtime substrate** that backs the protocol — documented here because they define the protocol's observable semantics (resolution, substitution, dedup, timeout), even though they are not themselves serialised on the wire. Each section states which it is.

**Convention notes:**
- The transport is JSON Lines: exactly one `Command` object per inbound line, one `Reply` object per outbound line (no trailing newline in the value — the pipe writer appends it).
- Argument keys are `snake_case` (§4.1).
- The envelope is **forward-compatible by omission**: neither `Command` nor `Reply` uses `deny_unknown_fields`, so optional fields (`token`, `idempotency_key`, `reject_code`, …) may be absent on older senders and parse cleanly. The one exception is the `Filter` payload, which *does* `deny_unknown_fields` (a stray key is a `BAD_ARGUMENT`).

---

## O.1 `Command` (wire — inbound)

**Source:** `aicontrol/envelope.rs`  
**Spec:** §4.1 / AC-D1  
**Description:** One inbound JSONL command. `cmd` is the only required field; a line that is not valid JSON or lacks a non-empty `cmd` fails with `MALFORMED_COMMAND` (the reply then omits the echoed `cmd`/`id`).

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `cmd` | `String` / string | Req | The command verb / CLI-path string (resolved by §O.6). |
| `args` | `Map<String, Value>` / object | Opt | Named arguments (`snake_case` keys). Defaults to empty so no-arg verbs (`whoami`/`status`/`state`) may omit it. |
| `id` | `Option<String>` / string | Opt | Driver-supplied correlation id, echoed verbatim into the reply. |
| `bind` | `Option<String>` / string | Opt | Names this command's result for later `$`-substitution (§O.8). |
| `token` | `Option<String>` / string | Opt | AC-D4 per-connection control token. A **top-level** field (never inside `args` — an `args` entry would be reconstructed into a `--token` flag). Opaque; carried unchanged. `absent == proceed`; inert in v1. See §O.11. |
| `idempotency_key` | `Option<String>` / string | Opt | AC-D6 idempotency key. Same shape rule as `token` (top-level, opaque, never in `args`). `absent == do-it-over`. See §O.9. |

## O.2 `Reply` (wire — outbound)

**Source:** `aicontrol/envelope.rs`  
**Spec:** §4.2  
**Description:** One outbound JSONL reply. Internally tagged by `status` (`"ok"` / `"error"`).

**`status: "ok"`** — success.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `status` | string | Req | `"ok"`. |
| `cmd` | string | Req | The echoed command verb. |
| `id` | string | Opt | Echoed iff the command supplied one. |
| `data` | object | Req | The command's result payload. |

**`status: "error"`** — failure, carrying a structured `ErrorBody` (§O.3).

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `status` | string | Req | `"error"`. |
| `cmd` | string | Opt | Echoed command verb; **omitted** on a `MALFORMED_COMMAND` (nothing to echo). |
| `id` | string | Opt | Echoed iff supplied. |
| `error` | `ErrorBody` / object | Req | The structured error. See §O.3. |

## O.3 `ErrorBody` (wire)

**Source:** `aicontrol/envelope.rs`  
**Spec:** §4.3  
**Description:** The structured error body. Drivers branch on `category`, never parse `code`.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `code` | string | Req | A band code (`SPACE_8005`, …), `GENERIC_4000`, or an uppercase-snake control code (`BAD_ARGUMENT`, …). |
| `category` | `Category` / string | Req | The closed category set (§O.4) — the disambiguator. |
| `message` | string | Req | Human-readable; not for programmatic parsing. |
| `instance_state` | string | Req | The instance's lifecycle state at the time of the error. |
| `stage` | string | Opt | The verb failure stage — present only for Node verb errors; never on control errors. |
| `hint` | string | Opt | A suggested next command, if applicable. |
| `reject_code` | u32 / number | Opt | MP-F5: the node's wire reject code (e.g. 3030 `tier_mismatch`), present only on a locally-submitted single-event verb reject. `code` still stays `GENERIC_4000`; the wire semantics ride here. |
| `event_id` | string | Opt | MP-F5: the rejected event's id (correlation key), present alongside `reject_code`. |

## O.4 `Category` (wire)

**Source:** `aicontrol/envelope.rs`  
**Spec:** AC-D2  
**Description:** The closed `category` enumerated set. Adding a category is a deliberate envelope change, not an ad-hoc string. Serialises lowercase. **Invariant (AC-D3d):** `protocol` is verb-sourced only — control-surface errors never use it.

| Variant | Wire string | Used for |
|---|---|---|
| `Protocol` | `"protocol"` | Verb-sourced errors (band codes / client `anyhow`). Never a control code. |
| `Lifecycle` | `"lifecycle"` | Instance-not-ready and lifecycle faults. |
| `Argument` | `"argument"` | Bad/missing args, unknown command, unknown binding, concurrent command, malformed command. |
| `Connection` | `"connection"` | Lost home-Node connection / missing resource. |
| `Timeout` | `"timeout"` | Per-command window exceeded. |
| `Permission` | `"permission"` | Session lacks authority (reserved/unused in v1). |

## O.5 `ControlCode` catalogue (wire codes)

**Source:** `aicontrol/codes.rs`  
**Spec:** AC-D3d / §8  
**Description:** The v1 control-surface error catalogue — **closed at nine codes**, each mapping to exactly one `Category` (never `Protocol`). Distinct from the numeric protocol error domains (Appendix I §XIII); these are uppercase-snake strings. The substrate type `ControlError { code, message, hint }` stamps `instance_state` into the wire `ErrorBody` via `into_body`.

| Variant | Wire string | Category |
|---|---|---|
| `InstanceNotReady` | `INSTANCE_NOT_READY` | lifecycle |
| `UnknownCommand` | `UNKNOWN_COMMAND` | argument |
| `BadArgument` | `BAD_ARGUMENT` | argument |
| `BindingNotFound` | `BINDING_NOT_FOUND` | argument |
| `ConcurrentCommandNotAllowed` | `CONCURRENT_COMMAND_NOT_ALLOWED` | argument |
| `ConnectionLost` | `CONNECTION_LOST` | connection |
| `Timeout` | `TIMEOUT` | timeout |
| `PermissionDenied` | `PERMISSION_DENIED` | permission |
| `MalformedCommand` | `MALFORMED_COMMAND` | argument |

## O.6 Command resolution (runtime)

**Source:** `aicontrol/cmd.rs`  
**Spec:** AC-D1  
**Description:** How a `cmd` string resolves. Reserved control verbs are matched first; everything else splits on the **first space** (the structural category/verb separator — a hyphen is intra-token and never a split point, which dissolves the `auth-module` collision). The category/verb split lives in the dispatcher, **not** on the wire — the wire carries an opaque command-path string. These types are runtime resolution vocabulary, not serialised.

- **`ControlVerb`** — a reserved verb handled by the control surface itself. v1 set: `State` (the in-process `state` verb, §9). (`subscribe`/`unsubscribe` live on the events pipe and are never resolved here.)
- **`CmdPath`** — a CLI-path command after the split: `category: Option<String>` (`None` for single-token Client verbs; `Some` for `category verb` Node verbs) + `verb: String` (a single hyphenated word, e.g. `set-node-policy`).
- **`CmdResolution`** — `Control(ControlVerb)` or `Cli(CmdPath)`. A `Cli` path the binary has no verb for answers `UNKNOWN_COMMAND`.

## O.7 `Filter` (wire — `subscribe` payload)

**Source:** `aicontrol/filter.rs`  
**Spec:** AC-D3b / EV-D4 v1.1  
**Description:** A parsed events-pipe subscription filter. Three dimensions, **AND-across / OR-within**; each empty dimension imposes no restriction. `#[serde(deny_unknown_fields)]` — a stray key is a `BAD_ARGUMENT`, validated fully before any streaming starts.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `spaces` | `Vec<SpaceXgid>` / array | Opt | Restrict to these Spaces (empty == all entitled). Out-of-entitlement entries are inert (match nothing), never an error. |
| `event_types` | `Vec<String>` / array | Opt | Restrict to these types (empty == all). Each entry is `*`, a `<family>.*` wildcard (exactly one `[a-z_]+` segment then `.*`), or an exact known `EventType` wire string. Unknown *exact* types fail closed (`BAD_ARGUMENT`); a well-formed `<family>.*` is accepted-but-inert if no real type matches. |
| `nodes` | `Vec<NodeXgid>` / array | Opt | Restrict to events involving these nodes (empty == all). Node-side only; the client rejects a non-empty `nodes` at its call site. The node-set per event is caller-supplied (runtime-derived), as an `Event` carries no uniform node field. |

## O.8 `Bindings` / `BoundValue` (runtime)

**Source:** `aicontrol/bindings.rs`  
**Spec:** §5  
**Description:** The per-connection named-variable namespace backing `$`-substitution. A `bind:"foo"` records a successful command's result; `$foo` resolves to its **primary** return, `$foo.field` to other result fields by dot-notation. Substitution is substring-level inside JSON string values, simple dot-notation only, no expressions. Scoped to the pipe connection (a fresh connection starts empty). An unknown binding or field → `BINDING_NOT_FOUND`. When a whole string value is exactly one token, the bound value's JSON type is preserved (a numeric field substitutes as a number).

- **`BoundValue`** — `primary: Value` (what bare `$name` resolves to) + `fields: Map<String, Value>` (the full result object, for dot-notation).
- **`Bindings`** — the `name → BoundValue` map; `snapshot()` exposes `name → primary` for the `state` reply's `bindings` field (§9).

## O.9 `IdempotencyStore` (runtime)

**Source:** `aicontrol/idempotency.rs`  
**Spec:** AC-D6 / M7C-D2  
**Description:** A bounded `key → completed-Reply` cache, one per `.aicontrol` connection (per-session scope). A command carrying an `idempotency_key` that completes **successfully** is recorded at result-time; a later command with the same key returns the recorded reply without re-executing. `absent == do-it-over`. FIFO eviction at capacity (`DEFAULT_IDEMPOTENCY_CAP = 1024`); an evicted key loses dedup (a replay re-executes). First-writer-wins on a duplicate record. Scope lives in *placement* (per-session now → per-driver later), not the wire — the `idempotency_key` field is unchanged either way.

## O.10 `TimeoutTier` (runtime)

**Source:** `aicontrol/timeout.rs`  
**Spec:** AC-D3a  
**Description:** The per-command timeout class. Tier is derived from a verb's READ/WRITE classification plus a federation-interaction flag — there is no per-verb table. Standing invariant: a tier default is always ≥ the verb's own internal timeout (else the guard masks legitimate slow completion as a false `TIMEOUT`).

| Variant | Default | Applies to |
|---|---|---|
| `Read` | 5 s (`READ_TIMEOUT_SECS`) | Local reads: `state`, `whoami`, `status`, list/show/query. |
| `Write` | 30 s (`WRITE_TIMEOUT_SECS`) | Home-Node round-trip / writes: `send`, `create-*`, `invite`, `join`, `register`, node writes. |
| `Federation` | 180 s (`FEDERATION_TIMEOUT_SECS`) | Cross-Node handshakes: `federation initiate` / `accept`. |

A driver's optional `timeout_ms` (§10, from `args`) is honored as-is (no clamp-up — the driver owns the trade-off) but floor-validated: a positive integer, else `BAD_ARGUMENT`. Absent → the tier default.

## O.11 Control token (wire field + runtime check)

**Source:** `aicontrol/token.rs`  
**Spec:** AC-D4 / M7C-D1  
**Description:** The `Command.token` field (§O.1) is an opaque per-connection control token. `check_token` is the verification seam: `absent == proceed`, and the seam is **inert in v1** (no expected token configured). The field is **B-subsumable** — end-state B's driver-bound credential (a signed capability, a JWT-shaped blob, anything) rides this same field with no wire change; the envelope never parses or normalises it.

---

*End of Appendix O*
