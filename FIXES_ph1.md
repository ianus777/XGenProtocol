# XGen Protocol — Phase 1 Consistency Fixes
> **Status:** COMPLETED  
> **Last updated:** 2026-05-06  
> Document type: Fix instructions for Claude Code  
> Applies to: `xgen_ch3_specification.md`, `xgen_ch4_implementation.md`  
> Date: April 2026  
> Prepared by: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

---

## How to use this document

Each fix is numbered and self-contained. For every fix:
1. Read the **File** and **Location** to find the exact place in the document.
2. Read the **Problem** to understand what is wrong.
3. Apply the **Fix** exactly as described.
4. After all fixes are applied, verify the **Verification checklist** at the bottom.

Do not reformat, rewrite, or restructure any section beyond what each fix explicitly requires.

---

## Fix 01 — Corrupted box-drawing characters in transport frame diagram

**File:** `xgen_ch3_specification.md`  
**Location:** Section 3.1.2, the transport frame ASCII table  
**Problem:** The top-left corner of the box-drawing table starts with corrupted bytes (`���─────`) instead of a clean line. The table currently looks like:

```
���─────────────────────────────────────────────────────────────┐
│ Transport frame structure                                   │
```

**Fix:** Replace the entire transport frame ASCII table with the following clean version:

```
┌─────────────────────────────────────────────────────────────┐
│ Transport frame structure                                   │
├──────────┬──────────────────────────────────────────────────┤
│ 1 byte   │ Format identifier length (N)                     │
│ N bytes  │ Format identifier string (UTF-8)                 │
│ 4 bytes  │ Payload length in bytes (unsigned 32-bit int)    │
│ M bytes  │ Serialised message payload                       │
└──────────┴──────────────────────────────────────────────────┘
```

---

## Fix 02 — Corrupted glyph in Tier ceiling table note

**File:** `xgen_ch3_specification.md`  
**Location:** Section 3.1.1, the paragraph beginning "The descending direction is intentional"  
**Problem:** The phrase contains a corrupted character: `"permission updates ��"`.  
**Fix:** Replace `permission updates ��` with `permission updates`.  

The corrected sentence should read:
> Government-tier protocol messages — signed state events, membership changes, permission updates — are rarely larger than 2KB in practice.

---

## Fix 03 — Section status markers not updated

**File:** `xgen_ch3_specification.md`  
**Location:** The `*Status: wip*` line that appears directly below each of the following section headers:
- `### 3.1 Wire Format`
- `### 3.2 Event Specification`
- `### 3.3 Transport Protocol`
- `### 3.4 Federation Handshake`
- `### 3.5 Node Identity Protocol`
- `### 3.6 Identity Registration Protocol`
- `### 3.7 Space & Room Protocol`
- `### 3.8 Auth Module — Tier 1 Specification`

**Problem:** All eight sections carry `*Status: wip*` but are marked `✅ Complete` in the skeleton table at the top of the document.  
**Fix:** For each of the eight sections listed above, replace the line:

```
*Status: wip*
```

with:

```
*Status: complete*
```

Apply this change to all eight sections. Do not change any other text in those sections.

---

## Fix 04 — Clarify `xgen_uri` scope in URI Formats section

**File:** `xgen_ch3_specification.md`  
**Location:** Section 3.1.6, the `xgen_uri` subsection  
**Problem:** The `xgen_uri` type is described with examples (`xgen://identity/...`, `xgen://space/...`, `xgen://node/...`, `xgen://room/...`) that do not correspond to any actual Phase 1 message field. In practice, Phase 1 uses `pubkey_uri` for Identity IDs and Node IDs, and `hash_uri` for Space IDs, Room IDs, and Event IDs. The `xgen_uri` type as described is not directly used in any Phase 1 wire format field, which causes implementer confusion.  
**Fix:** Add the following note directly below the `xgen_uri` grammar block and examples, before the `hash_uri` subsection:

```
> **Phase 1 note:** In Phase 1 protocol messages, `xgen_uri` does not appear as a
> standalone field type. Identity IDs and Node IDs use `pubkey_uri` directly.
> Space IDs, Room IDs, and Event IDs use `hash_uri` directly. The `xgen_uri` wrapper
> form (`xgen://identity/...`, `xgen://space/...`, etc.) is reserved for Phase 2
> contexts such as resource addressing in REST-style management APIs and Bootstrap
> Node directories. Phase 1 implementers do not need to parse or produce `xgen_uri`
> values — only `hash_uri` and `pubkey_uri` appear in Phase 1 wire fields.
```

---

## Fix 05 — Add missing `transport.sync_response` schema

**File:** `xgen_ch3_specification.md`  
**Location:** Section 3.3.6, directly after the `transport.sync_request` JSON schema block  
**Problem:** The `transport.sync_request` message schema is defined but the Node's response — `transport.sync_response` — has no schema. An implementer cannot build the sync mechanism without knowing what the response looks like.  
**Fix:** Insert the following content directly after the `transport.sync_request` schema block (after the paragraph that begins "If the client has no prior Events for a Room..."):

```
**`transport.sync_response`** — sent by the Node in reply to a `transport.sync_request`:

The Node sends all Events that follow `last_event_id` in the specified Room's DAG,
in causal order (parents before children), as individual Event messages on the
active connection. After the last Event has been sent, the Node sends a
`transport.sync_complete` message to signal the end of the sync batch:

```json
{
  "protocol_version": "0.1",
  "type": "transport.sync_complete",
  "room_id": "xgen://hash/sha256:b2c3d4e5...",
  "event_count": 12,
  "timestamp": "2026-04-26T10:01:00.000Z"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `room_id` | hash_uri | yes | The Room this sync batch covers — matches the request |
| `event_count` | integer | yes | Total number of Events sent in this batch — for validation |
| `timestamp` | datetime | yes | When the Node sent this completion marker |

If there are no missed Events (the client is already up to date), the Node sends
`transport.sync_complete` immediately with `event_count: 0`.

If `last_event_id` is unknown to the Node (the referenced Event is not in its log),
the Node sends the full Room history from the DAG root, subject to any Space history
limits. This handles the case where a client's state is too stale to anchor.
```

---

## Fix 06 — Add missing EventTypes to the Phase 1 EventType registry

**File:** `xgen_ch3_specification.md`  
**Location:** Section 3.2.2, the Phase 1 EventType registry tables  
**Problem:** The registry lists `state.room_create` and room-level state events, but is missing EventTypes that are actively used in the Phase 1 protocol: `state.space_create`, `state.dm_space_create`, `state.federation_add`, `state.federation_remove`, and `state.node_priority`. These are all referenced in sections 3.4 and 3.7 but absent from the canonical registry, making the registry incomplete.  
**Fix:** Add two new table rows to the *State events* table in section 3.2.2 — the Space-level state events — and add a new *Federation events* table after the existing State events table. Insert the following:

**Add to the end of the existing State events table** (after `state.room_avatar`):

| EventType | Description |
|---|---|
| `state.space_create` | Space creation — root Event for a Space, establishes auth_tier and home_node |
| `state.dm_space_create` | DM Space creation — two-member variant of Space, auto-creates one Room |
| `state.node_priority` | Space owner declares manual ordering of federated Nodes for conflict resolution |

**Add a new table after the State events table, before the Membership events table:**

*Federation events* — record federation relationship changes in a Space's DAG:

| EventType | Description |
|---|---|
| `state.federation_add` | Records that a new Node has joined the federation for this Space |
| `state.federation_remove` | Records that a Node has left or been removed from federation for this Space |

---

## Fix 07 — Fix membership event scope description (Room → Space)

**File:** `xgen_ch3_specification.md`  
**Location:** Section 3.2.2, the Membership events table header and description  
**Problem:** The membership events table is introduced with the description "record Identity membership transitions in a Room." However, `membership.invite/join/leave/kick/ban` events operate at the **Space** level, not the Room level. Room membership in Phase 1 is derived automatically from Space membership — all Space members have access to all Rooms (section 3.7.9). Describing them as Room-level events is incorrect and will mislead implementers.  
**Fix:** In the Membership events table, change the table header description from:

```
*Membership events* — record Identity membership transitions in a Room:
```

to:

```
*Membership events* — record Identity membership transitions in a Space.
Room membership in Phase 1 is derived from Space membership — a Space member
has access to all Rooms in that Space (see 3.7.9). Private Rooms with
independent membership are Phase 2:
```

---

## Fix 08 — Fix corrupted emoji in Ch4 skeleton table

**File:** `xgen_ch4_implementation.md`  
**Location:** The Chapter 4 Section Skeleton table, row for section 4.6  
**Problem:** The status cell for `4.6 | Cryptographic Primitives` displays as `��` (corrupted emoji) instead of the checkmark used in all other completed rows.  
**Fix:** In the skeleton table, replace the row:

```
| 4.6 | Cryptographic Primitives | �� Complete |
```

with:

```
| 4.6 | Cryptographic Primitives | ✅ Complete |
```

---

## Fix 09 — Add `prev_events` empty-array exception to Event Envelope field table

**File:** `xgen_ch3_specification.md`  
**Location:** Section 3.2.1, the Event Envelope Schema field definitions table, row for `prev_events`  
**Problem:** The field table describes `prev_events` as required with "at least one required except for Room creation Events" — but this exception is only stated in the prose of section 3.2.5, not in the field table itself. A developer reading only the field table will not know that `state.room_create` is allowed to have an empty array.  
**Fix:** In the field definitions table, change the `prev_events` Description cell from:

```
IDs of the Events this Event causally follows — at least one required except for Room creation Events (3.2.5)
```

to:

```
IDs of the Events this Event causally follows. MUST contain at least one entry except for `state.room_create`, where this MUST be an empty array `[]` — it is the DAG root (3.2.5)
```

---

## Fix 10 — Clarify `transport.sync_request` Room-to-Space resolution

**File:** `xgen_ch3_specification.md`  
**Location:** Section 3.3.6, the `transport.sync_request` schema block and its field table  
**Problem:** The `transport.sync_request` message carries `room_id` but no `space_id`. The Ch4 Event store uses one SQLite database per Space. A Node receiving a sync request needs to locate the correct Space database before it can scan for the Room. Either `space_id` should be added to the request, or the resolution mechanism should be documented.  
**Fix:** Add `space_id` as a required field to the `transport.sync_request` schema. Replace the existing schema block with:

```json
{
  "protocol_version": "0.1",
  "type": "transport.sync_request",
  "space_id": "xgen://hash/sha256:c3d4e5f6...",
  "room_id": "xgen://hash/sha256:b2c3d4e5...",
  "last_event_id": "xgen://hash/sha256:a3f9b2c1..."
}
```

And update the field description paragraph that follows to read:

```
The `space_id` field identifies which Space's Event store to query.
The `room_id` field identifies which Room within that Space to sync.
The `last_event_id` field is the Event ID the client last received —
the Node returns all Events that causally follow it in the Room's DAG.
If the client has no prior Events for a Room (first join or fresh install),
it omits `last_event_id` and the Node sends the full Room history from the
DAG root, subject to any history limits declared by the Space.
```

---

## Fix 11 — Add Work Definitions table

**File:** `xgen_ch3_specification.md`  
**Location:** Directly before the `## Chapter 3 — Open Questions` section at the bottom of the document  
**Problem:** Several numeric values throughout Ch3 are explicitly flagged as "work definitions" pending Phase 1 testing validation. They are scattered across multiple sections with no consolidated list, making it impossible to review them all in one pass when Phase 1 testing begins.  
**Fix:** Insert the following new section directly before `## Chapter 3 — Open Questions`:

```markdown
## Chapter 3 — Work Definitions Pending Phase 1 Validation

The following values were established before implementation testing. Each is
explicitly provisional and MUST be reviewed against real-world measurements
during Phase 1 testing. No value should be treated as final until the Phase 1
smoke test has been run and message sizes and timing behaviour observed.

| # | Value | Current setting | Location | Review trigger |
|---|---|---|---|---|
| WD-01 | Tier 1 message size ceiling | 64 KB | 3.1.1 | Measure actual Event sizes in smoke test |
| WD-02 | Tier 2 message size ceiling | 32 KB | 3.1.1 | Measure actual Event sizes in smoke test |
| WD-03 | Tier 3 message size ceiling | 16 KB | 3.1.1 | Measure actual Event sizes in smoke test |
| WD-04 | Tier 4 message size ceiling | 8 KB | 3.1.1 | Measure actual Event sizes in smoke test |
| WD-05 | Keepalive ping interval | 30 seconds | 3.3.5 | Measure connection stability under load |
| WD-06 | Keepalive pong timeout | 10 seconds | 3.3.5 | Measure connection stability under load |
| WD-07 | Reconnection backoff ceiling | 30 seconds | 3.3.6 | Acceptable during smoke test; review under real network conditions |
| WD-08 | Pending Event buffer timeout | 30 seconds | 4.12.3 | Observe DAG sync latency between Nodes in smoke test |
| WD-09 | Trust Assertion TTL | 1 year | 3.8.6 | Review with Auth Module operator before production deployment |
| WD-10 | Node announcement TTL | 90 days | 3.5.6 | Review with network operator before production deployment |
| WD-11 | `prev_events` maximum array length | 10 entries | 3.2.5 | Observe maximum DAG concurrency in Phase 1 and Phase 2 federation tests |
| WD-12 | Federation handshake response timeout | 10 / 15 seconds | 3.4.3 | Observe handshake latency in smoke test |
| WD-13 | Federation re-initiation cooldown after reject | 60 seconds | 3.4.2 | Acceptable during smoke test; review for production |

After Phase 1 smoke test, update this table: replace "work definition" status
with either "confirmed" (value is appropriate) or "revised to X" (value changed).
```

---

## Fix 12 — Add missing `rooms` and `members` CLI commands to xgen-client

**File:** `xgen_ch4_implementation.md`  
**Location:** Section 4.16 (CLI Reference), the `xgen-client` commands table  
**Problem:** The CLI reference includes `spaces` (list Spaces the Identity belongs to) but is missing commands to list Rooms within a Space and to list members of a Space. Without these, an operator cannot inspect Space or Room state from the command line. In XGen protocol terminology, users are called **Identities** at the protocol level and **members** at the Space/Room level.  
**Fix:** Add the following commands to the `xgen-client` CLI commands table:

| Command | Arguments | Description |
|---|---|---|
| `rooms` | `<space_id>` | List all Rooms in the specified Space that this Identity is a member of |
| `members` | `<space_id>` | List all Identity IDs and display names currently in the specified Space |

**Usage examples to add:**

```
xgen-client rooms xgen://hash/sha256:a3f9b2c1...
xgen-client members xgen://hash/sha256:a3f9b2c1...
```

**Note to add in the CLI reference prose:** In XGen, the protocol-level term for a user is **Identity**. The human-facing term at Space level is **member**. The `members` command lists all Identities that have a current `membership.join` state in the Space and have not subsequently `membership.leave`d, been `membership.kick`ed, or been `membership.ban`ned.

---

## Fix 13 — ANSI colour output note in CLI reference

**File:** `xgen_ch4_implementation.md`  
**Location:** Section 4.16 (CLI Reference), general notes subsection  
**Problem:** The CLI reference does not document which terminal environments are supported for coloured output. Testing has confirmed that basic ANSI colours (red errors, etc.) render correctly in Windows Terminal and PowerShell.  
**Fix:** Add the following note to the CLI reference section:

```markdown
**ANSI colour output**

The CLI uses ANSI escape codes for coloured output (error messages in red, success
in green, warnings in yellow). Supported terminal environments include Windows
Terminal, PowerShell, and all standard Linux/macOS terminals.

Implementation note (Rust): use the `supports-color` crate for runtime detection.
This crate checks the `TERM` and `COLORTERM` environment variables and calls the
Windows `GetConsoleMode` API with `ENABLE_VIRTUAL_TERMINAL_PROCESSING` where
applicable. If detection returns false, strip escape sequences from output —
never suppress the message text itself, only the colour codes.
```

---

## Fix 14 — DEFERRED pending module architecture decision

**Status:** Deferred by project owner. Full membership lifecycle CLI commands (`invite`, `leave`, `kick`, `ban`, `stop`) will be specified after the XGen module architecture is defined.

**Why deferred:** CLI commands are one expression of a module — a module may also have a UI entry point, or both. The form a module takes (single file, folder, subprocess, shared library), how it registers with the core system, and how it communicates are open architectural questions that must be resolved before locking in any CLI command extension mechanism. Deciding the CLI surface now would constrain the module architecture before it is designed.

**Resolved during:** Ch6 second pass — module architecture section. Once the module form is defined, Fix 14 becomes a straightforward implementation task.

**Tracked as:** open question in Ch6 and DECISIONS.md (to be recorded when the module architecture discussion begins).

---

## Fix 15 — Document the keepalive-as-session model; no separate inactivity timeout

**File:** `xgen_ch3_specification.md`  
**Location:** Section 3.3.5 (Keepalive), at the end of the section after the existing content  
**Problem:** XGen has no traditional login session or inactivity timeout — authentication is stateless on the Node (challenge-response at connection time, no server-side session table). The keepalive mechanism IS the session health model. This is not documented, which will cause implementers to add unnecessary session timers, and may confuse operators who expect a conventional "you have been logged out due to inactivity" behaviour.  
**Fix:** Add the following subsection at the end of section 3.3.5:

```markdown
**Keepalive as the complete session model**

XGen does not implement a separate inactivity timeout. Authentication in XGen is
stateless on the Node side — the challenge-response at connection time (3.3.4) proves
the client holds the private key, but the Node maintains no session token and no
session table. There is nothing to "expire".

The keepalive mechanism above IS the session health model. If the WebSocket connection
drops — due to network failure, device sleep, or any other cause — the Node detects
this via the missed pong and closes its end. The client detects the dropped connection
and reconnects using the backoff sequence (3.3.6). Re-authentication is instant (a
single challenge-response round trip). From the user's perspective, the client
reconnects transparently in the background — there is no "logged out" state.

Implementers MUST NOT add a separate inactivity timer that closes the connection or
requires the user to re-enter credentials. The correct model is: connection alive =
authenticated. Connection dropped = reconnect and re-authenticate automatically.

The only valid reason to present a credential prompt to the user is if the encrypted
key file cannot be decrypted on startup (wrong passphrase or missing file). That is a
key access failure, not a session expiry.
```

---

## Fix 16 — Node does not restore Space state on restart (critical bug)

**File:** `xgen_ch4_implementation.md` (implementation bug — fix in the Rust source, document the correct behaviour in Ch4)  
**Location:** Node startup sequence — wherever the Node initialises its in-memory state on `xgen-node` launch  
**Problem:** When a Node is shut down and restarted, it loses all Space records. The in-memory Space registry is not reconstructed from the SQLite Event store on startup. As a result, any client that sends an Event referencing a `space_id` from a previous session receives `accept_message failed: step 10: DAG structural violation — space not found`, even though the Space was legitimately created and its Events are still in the database.

This was confirmed by the following test sequence:
1. Session 1: Node A started → Space `TestSpace` created → Node A shut down
2. Session 2: Node A restarted → Client joined `TestSpace` → Client sent message → Node rejected with `space not found`

The join was accepted (Identity lookup worked) but the message was rejected because Space lookup failed. This means the Node accepts Identities from its persisted Identity registry but does not reconstruct its Space registry from the same database — inconsistent persistence behaviour.

**Root cause:** The Node initialises its Space registry as an empty in-memory structure on startup instead of replaying the Event log to reconstruct current state.

**Fix:** Implement full state reconstruction from the SQLite Event log on Node startup. The correct startup sequence is:

```
1. Load and decrypt keypair
2. Read node_config.json
3. Scan the data directory for all Space SQLite databases
4. For each database found:
   a. Open the database
   b. Read all Events in causal order (parents before children)
   c. Apply each Event to reconstruct current state:
      - state.space_create  → register Space in memory
      - state.room_create   → register Room under its Space
      - membership.join     → add Identity to Space membership
      - membership.leave / kick / ban → remove Identity from Space membership
      - state.federation_add / remove → reconstruct federation registry
      - state.node_priority → reconstruct Node priority ordering
      - state.room_name / topic / avatar → update Room state
5. Only after all databases are replayed: open network listener and accept connections
```

The principle is: **the SQLite Event log is the source of truth. In-memory state is always derived from it, never the other way around.** A Node that has replayed its Event log is in exactly the same state as a Node that has been running continuously since genesis.

**Secondary fix:** The Node should also reject a `membership.join` Event for a Space it does not recognise, not silently accept it. Currently the Node accepts the join (because Identity lookup succeeds) but then rejects subsequent messages. The correct behaviour is to fail at join time with a clear `space_not_found` error so the operator knows immediately what is wrong.

**Note for Ch4 documentation:** Add a section to 4.8 (Node startup sequence) explicitly documenting the state reconstruction requirement. The current Ch4 text describes the startup sequence but does not specify that Space state must be replayed from the Event log before the network listener opens. This must be stated as a hard requirement, not an implementation detail.

---

## Fix 17 — Move `event_trace` module from `xgen-node` to `xgen-common`

**File:** `xgen-node/src/event_trace.rs` — move to `xgen-common/src/event_trace.rs`  
**Problem:** `event_trace.rs` was placed in `xgen-node/src/` during Priority 0 implementation. The module contains `EventDirection`, `SpaceRole`, `SessionContext`, and `trace_event()` — all of which are shared infrastructure used by both `xgen-node` and `xgen-client`. Shared code belongs in `xgen-common`, not in one of the consuming crates. The current placement means `xgen-client` imports shared tracing infrastructure via a dependency on `xgen-node` rather than on the common crate — which is architecturally wrong and will cause problems when the `xgen-core` crate split (D-022) happens in Phase 2.  

**Fix — four steps:**

**Step 1:** Move the file:
```
xgen-node/src/event_trace.rs  →  xgen-common/src/event_trace.rs
```
File content is unchanged — only the location moves. Remove one import at the top: `use crate::wire::types::Event;` — replace with the correct path from `xgen-common`'s perspective (the `Event` type must be accessible from `xgen-common`, either already defined there or re-exported from `xgen-node`). If `Event` is not yet in `xgen-common`, define a minimal shared `Event` reference type there, or import from `xgen-node` via the crate dependency. Use whichever approach compiles cleanly without circular dependencies.

**Step 2:** Expose from `xgen-common/src/lib.rs`:
```rust
pub mod event_trace;
```

**Step 3:** Update `xgen-node/src/lib.rs` or `main.rs` — change the import from:
```rust
use crate::event_trace::{EventDirection, SessionContext, SpaceRole, trace_event};
```
to:
```rust
use xgen_common::event_trace::{EventDirection, SessionContext, SpaceRole, trace_event};
```

**Step 4:** Update `xgen-client/src/main.rs` — change the import from:
```rust
use xgen_node::event_trace::{EventDirection, SessionContext, SpaceRole, trace_event};
```
to:
```rust
use xgen_common::event_trace::{EventDirection, SessionContext, SpaceRole, trace_event};
```

**Verify:** `cargo test` — 173/173 tests must pass. Confirm `event_trace` no longer exists in `xgen-node/src/`. Confirm both binaries compile and produce log output as before.

---

## Fix 18 — Remove `log_path` and `spaces_dir` from config — derive from working directory

**File:** `xgen-node/src/main.rs`, `test/node_a/xgen-node_config.toml`, `test/node_b/xgen-node_config.toml`  
**Decision record:** D-035  
**Problem:** `log_path` and `spaces_dir` are user-editable fields in `xgen-node_config.toml`. This is a security problem: the fields reveal where sensitive data is stored, can be tampered with to redirect data, and create no separation between config (operators read) and data (nobody should modify directly). Absolute paths like `E:\XGen\XGenNode_A\spaces` in a config file are especially problematic.

**Fix — three steps:**

**Step 1:** Remove `log_path` and `spaces_dir` from the `PathsSection` struct in `xgen-node/src/main.rs`. The `[paths]` section should contain only `keypair_path` — the single legitimate exception because operators may store the keypair on a separate device:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct PathsSection {
    keypair_path: String,   // only configurable path remaining
}
```

Add working-directory-relative constants:

```rust
const SPACES_DIR: &str = "spaces";
const LOGS_DIR: &str = "logs";
const AUDIT_DIR: &str = "audit";
```

All path construction in the Rust source that previously used `config.paths.spaces_dir` or `config.paths.log_path` now uses `working_dir.join(SPACES_DIR)` and `working_dir.join(LOGS_DIR)` respectively.

**Step 2:** Update both test config files to remove the now-invalid fields:

`test/node_a/xgen-node_config.toml`:
```toml
[node]
listen = "ws://127.0.0.1:8080/xgen"
local_mode = true

[paths]
keypair_path = 'test/node_a\xgen-node_keypair.enc'

[logging]
level = "info"
```

`test/node_b/xgen-node_config.toml`:
```toml
[node]
listen = "ws://127.0.0.1:8081/xgen"
local_mode = true

[paths]
keypair_path = 'test/node_b\xgen-node_keypair.enc'

[logging]
level = "info"
```

**Step 3:** Update `xgen-node init` command output and the generated default config template to reflect that `spaces_dir` and `log_path` are no longer configurable. The generated config should only contain `[node]`, `[paths]` (keypair only), and `[logging]`.

**Verify:** `cargo test` 173/173 pass. Start Node A and Node B from their test directories. Confirm `spaces/` and `logs/` are created automatically in the correct working directory. Confirm no absolute paths appear in any config file.

---

## Verification checklist

After applying all fixes, verify the following:

- [ ] Transport frame box in 3.1.2 has clean box-drawing characters, no corrupted bytes
- [ ] Phrase "permission updates" in 3.1.1 has no corrupted glyph after it
- [ ] All eight sections 3.1–3.8 show `*Status: complete*`, not `*Status: wip*`
- [ ] Section 3.1.6 has the Phase 1 note below the `xgen_uri` examples
- [ ] Section 3.3.6 has the `transport.sync_complete` schema after `transport.sync_request`
- [ ] Section 3.2.2 State events table includes `state.space_create`, `state.dm_space_create`, `state.node_priority`
- [ ] Section 3.2.2 has a Federation events table with `state.federation_add` and `state.federation_remove`
- [ ] Section 3.2.2 membership events description says "Space" not "Room"
- [ ] Ch4 skeleton table row 4.6 shows `✅ Complete`
- [ ] Section 3.2.1 `prev_events` field description mentions the empty array exception for `state.room_create`
- [ ] Section 3.3.6 `transport.sync_request` schema includes `space_id` field
- [ ] Work Definitions table (WD-01 through WD-13) exists before `## Chapter 3 — Open Questions`
- [ ] Section 4.16 `xgen-client` table includes `rooms` and `members` commands
- [ ] Section 4.16 includes the ANSI colour output note with `supports-color` crate named
- [ ] Section 3.3.5 ends with the "Keepalive as the complete session model" subsection
- [ ] Node startup replays SQLite Event log before opening network listener (Fix 16 — Rust source)
- [ ] Node rejects `membership.join` for unknown Space with `space_not_found` error (Fix 16 — Rust source)
- [ ] Section 4.8 documents state reconstruction as a hard startup requirement (Fix 16 — Ch4 doc)
- [x] `event_trace` module lives in `xgen-common/src/event_trace.rs`, not in `xgen-node/src/` (Fix 17)
- [x] Both binaries import `event_trace` from `xgen_common::event_trace` (Fix 17)
- [x] `xgen-node/src/event_trace.rs` no longer exists (Fix 17)
- [ ] `log_path` and `spaces_dir` removed from `NodeConfig` struct and all config files (Fix 18)
- [ ] `spaces/`, `logs/`, `audit/` derived from working directory via constants (Fix 18)
- [ ] Only `keypair_path` remains in `[paths]` section (Fix 18)

---

## Files modified

| File | Fixes applied |
|---|---|
| `xgen_ch3_specification.md` | 01, 02, 03, 04, 05, 06, 07, 09, 10, 11, 15 |
| `xgen-node/src/` | 17 (event_trace moved out) |
| `xgen-common/src/` | 17 (event_trace moved in) |

---

## Session log

| Session | Date | Changes |
|---|---|---|
| Session 1 | April 2026 | Fixes 01–11: consistency review of Ch3 Phase 1 vs Ch4 cross-check |
| Session 2 | April 2026 | Fixes 12–15 drafted; Fix 14 (membership lifecycle CLI) deferred by project owner; Fix 13 revised — basic ANSI colours confirmed working on Windows Terminal/PowerShell |
| Session 3 | April 2026 | Fix 16 added — critical bug: Node does not reconstruct Space state from SQLite Event log on restart, confirmed by live test |
| Session 4 | April 2026 | Fix 17 added — `event_trace` module must move from `xgen-node/src/` to `xgen-common/src/` — shared infrastructure belongs in the common crate |
| Session 5 | April 2026 | Fix 17 applied — `event_trace` and `wire.rs` (Event, EventType) moved to `xgen-common`; re-exported from `xgen-node/src/wire/types.rs`; 173/173 tests pass; smoke test with logging confirmed (J-026) |
| Session 6 | April 2026 | Fix 18 added — `log_path` and `spaces_dir` removed from config; all data paths derived from working directory by convention (D-035) |

---

*End of fix document*
