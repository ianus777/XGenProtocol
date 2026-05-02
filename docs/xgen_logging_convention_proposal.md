
# XGen Logging Convention Proposal --- AI-Readable Structured Logs (Draft v0.1)
> Status: ?  
> Version: 1.0 (Derivated at start from the Summary v0.7)  
> Date: May 2026  
> Last edited: May 2026  
> Language: English  
> Author: JozefN (via ChatGPT)  
> License: BSL 1.1 (converts to GPL upon project handover)  

**THIS DOCUMENT IS JUST A PROPOSAL YET --- NOTHING MANDATORY!**

## Purpose

This document defines a logging convention for XGen implementations that produces logs which are:

 - human-readable without tooling machine-readable without schema
 - negotiation deterministic and stable across versions analyzable by
 - generic AI systems without custom parsers

The goal is not aesthetic logging, but forensic-grade observability — logs that allow reconstruction of protocol behavior from first principles.

------------------------------------------------------------------------

## Design Principles

### 1. Event-Centric Correlation

All cross-node pairing is performed using `event_id`.

 - `event_id` is the only globally reliable identifier
 - timestamps are local and non-authoritative
 - log analysis MUST NOT rely on time correlation

A valid XGen log system allows full reconstruction of event flow across nodes using only `event_id`.

------------------------------------------------------------------------

### 2. Session-Scoped Identity

Each log file represents a single execution session of a producer application (Node or Client).

A session header MUST be present at the beginning of each log file and defines the identity and context of the producer.

------------------------------------------------------------------------

### 3. One Event Per Line

Each log line represents exactly one atomic action.

- no multiline entries
- no embedded blocks
- no implicit continuation

This ensures compatibility with:

- streaming analysis
- line-based tools (`grep`, `awk`)
- AI ingestion

------------------------------------------------------------------------

### 4. Key--Value Structure

Log lines MUST use a flat key–value format:

    key=value key=value key=value ...

Rules:

- keys are lowercase `snake_case`
- values are unquoted unless necessary
- no free-text sentences
- no positional meaning

Example:

    timestamp=2026-04-30T17:24:50.123Z direction=OUT action=send_event event_id=xgen://ha

------------------------------------------------------------------------

### 5. Stable Field Names

Field names are part of the logging contract and MUST remain stable across versions.

Core fields:
| Field | Description |
|--|--|
| `timestamp` | RFC 3339 UTC datetime |
| `direction` |	`IN`, `OUT`, or `LOCAL`
| `action` | operation being performed
| `event_id` | unique event identifier

Optional fields (recommended):
| Field | Description |
|--|--|
| `event_type` | EventType string |
| `sender` | identity pubkey URI
| `space_id` | Space identifier
| `room_id` | Room identifier
| `target` | destination node or identity
| `error_code` | protocol error code

Unknown fields MUST be ignored by analyzers.

------------------------------------------------------------------------

### 6. Explicit Semantics

All meaning MUST be explicit.

Forbidden:

    received event
    sending message
    operation successful

Required:

    direction=IN action=receive_event source=node_b event_id=...
    direction=OUT action=send_event target=node_b event_id=...

------------------------------------------------------------------------

### 7. No Decorative Output

Logs MUST NOT contain:

- emojis
- formatting symbols
- natural language commentary
- redundant success messages

Logs are a data structure, not a user interface.

------------------------------------------------------------------------

## Session Header Example

Each log file MUST begin with a session header block.

Example (Node):

    === XGEN SESSION START ===
    app_type: node
    node_id: xgen://pubkey/ed25519:AAA...
    endpoint: 127.0.0.1:8080
    protocol_version: 0.1
    build: 0.6.2-260430-1724
    session_id: 9f3a7c12
    started_at: 2026-04-30T17:24:50.123Z
    ==========================

Example (Client):

    === XGEN SESSION START ===
    app_type: client
    identity_id: xgen://pubkey/ed25519:BBB...
    connected_node: 127.0.0.1:8080
    protocol_version: 0.1
    build: 0.6.2-260430-1724
    session_id: a81c2d44
    started_at: 2026-04-30T17:24:51.002Z
    ==========================

Example of correct end of log session, indicates application's proper shut down:

    === XGEN SESSION END ===

## Header Rules
- MUST appear exactly once per file
- MUST precede all log lines
- MUST be static (not updated during runtime)

------------------------------------------------------------------------

## Example Log Stream

    timestamp=2026-04-30T17:24:50.123Z direction=LOCAL action=create_event event_id=xgen://hash/abc... event_type=state.space_create
    timestamp=2026-04-30T17:24:50.456Z direction=OUT action=send_event target=node_b event_id=xgen://hash/abc...
    timestamp=2026-04-30T17:24:50.789Z direction=IN action=receive_event source=node_a event_id=xgen://hash/abc...
    timestamp=2026-04-30T17:24:51.000Z direction=LOCAL action=store_event event_id=xgen://hash

------------------------------------------------------------------------

## AI Analysis Properties

A log system conforming to this convention enables:

### 1. Deterministic Event Pairing

Group by `event_id` across multiple logs.

### 2. Flow Reconstruction

Rebuild full propagation paths:

- creation
- outbound transmission
- inbound reception
- storage

### 3. Protocol Validation

Verify:

- DAG integrity
- propagation completeness
- validation pipeline correctness

### 4. Tool Independence

Logs can be analyzed by:

- generic AI systems
- simple scripts
- manual inspection

No schema registry or parser library is required.

### Design Rationale

XGen logs follow the same philosophy as the protocol itself:

- deterministic identifiers (event_id)
- explicit semantics
- forward compatibility
- minimal assumptions

The result is a logging system that is:

> self-describing, self-correlating, and self-debuggable

### Non-Goals

This convention intentionally avoids:

- JSON logging (unnecessary complexity)
- binary log formats
- human-oriented formatting
- transport-level tracing duplication

### Future Extensions

Possible future additions:

- `trace_id` for multi-event workflows
- structured error taxonomy mapping to protocol sections
- optional JSON export mode for external systems

All extensions MUST preserve backward compatibility.

------------------------------------------------------------------------

## Summary

This logging convention defines a minimal but strict structure that enables:

- reliable distributed debugging
- independent analysis by multiple AI systems
- long-term maintainability

It is not designed for readability first, but for truthfulness and analyzability.
