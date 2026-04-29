# XGen Protocol — Phase 1 Consistency Fixes
> Document type: Fix instructions for Claude Code  
> Applies to: `xgen_ch3_specification.md`, `xgen_ch4_implementation.md`  
> Date: April 2026  
> Prepared by: JozefN  

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

## Verification checklist

After applying all fixes, verify the following:

- [ ] The transport frame box in 3.1.2 has clean box-drawing characters with no corrupted bytes
- [ ] The phrase "permission updates" in 3.1.1 has no corrupted glyph after it
- [ ] All eight sections 3.1–3.8 show `*Status: complete*` not `*Status: wip*`
- [ ] Section 3.1.6 has the Phase 1 note below the `xgen_uri` examples
- [ ] Section 3.3.6 has both `transport.sync_request` and `transport.sync_response` / `transport.sync_complete` schemas
- [ ] Section 3.2.2 State events table includes `state.space_create`, `state.dm_space_create`, `state.node_priority`
- [ ] Section 3.2.2 has a new Federation events table with `state.federation_add` and `state.federation_remove`
- [ ] Section 3.2.2 membership events description says "Space" not "Room"
- [ ] Ch4 skeleton table row 4.6 shows `✅ Complete`
- [ ] Section 3.2.1 `prev_events` field description mentions the empty array exception for `state.room_create`
- [ ] Section 3.3.6 `transport.sync_request` schema includes `space_id` field
- [ ] The Work Definitions table exists before `## Chapter 3 — Open Questions`

---

## Files modified

| File | Fixes applied |
|---|---|
| `xgen_ch3_specification.md` | 01, 02, 03, 04, 05, 06, 07, 09, 10, 11 |
| `xgen_ch4_implementation.md` | 08 |

---

*End of fix document*
