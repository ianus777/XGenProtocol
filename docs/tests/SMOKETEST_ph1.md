# XGen Protocol — Phase 1 Smoke Test with Logging
> **Status:** COMPLETED  
> **Last updated:** 2026-05-06  
> Document type: Instructions for Claude Code  
> Date: April 2026  
> Prepared by: JozefN  
> Purpose: Re-run Phase 1 smoke test with full debug logging active. Verify global Event tracing interface (D-033) is working. Collect log files for review.

---

## Overview

This is a re-run of the Phase 1 smoke test, with the global Event tracing interface now active. The goal is:

1. Confirm the smoke test still passes end-to-end
2. Confirm log files are produced in the correct locations
3. Confirm Event pairing works — same `event_id` appears as `Outbound` in the client log and `Inbound` in the Node log
4. Confirm sensitive message content never appears in any log file
5. Collect all log files and report results

---

## Prerequisites — verify before starting

- [ ] `xgen-node.exe` and `xgen-client.exe` are built from latest source (including Fix 17 — `event_trace` in `xgen-common`)
- [ ] `test/node_a/xgen-node_config.toml` has `[logging]` section with `level = "debug"` (change from `"info"` for this test)
- [ ] `test/node_b/xgen-node_config.toml` has `[logging]` section with `level = "debug"` (change from `"info"` for this test)
- [ ] `test/node_a/logs/` folder exists (created automatically on first run if not)
- [ ] `test/node_b/logs/` folder exists (created automatically on first run if not)
- [ ] No stale state from previous sessions — delete these files if they exist:
  - `test/node_a/xgen-node_state.json`
  - `test/node_a/xgen-node_identities.db`
  - `test/node_b/xgen-node_state.json`
  - `test/node_b/xgen-node_identities.db`

> **Why debug level?** This test specifically validates that the global Event tracing interface produces output. At `info` level, Event-level entries are suppressed. After this test, restore both configs to `level = "info"`.

---

## Test environment

| Component | Location | Port |
|---|---|---|
| Node A | `test/node_a/` | `ws://127.0.0.1:8080/xgen` |
| Node B | `test/node_b/` | `ws://127.0.0.1:8081/xgen` |
| Client 1 | registers on Node A | — |
| Client 2 | registers on Node B | — |
| Binaries | `bin/xgen-node.exe`, `bin/xgen-client.exe` | — |

All commands below are run from the `bin/` directory where the executables live. Open four separate terminal windows.

---

## Step 1 — Start Node A

**Terminal 1 — from `bin/` directory:**
```
xgen-node --config ..\test\node_a\xgen-node_config.toml
```

Expected console output:
```
----------------------------------------
  xgen-node  vX.X.X  (commit)
  Built: ...
  XGen Protocol — Phase 1
----------------------------------------

Node ID:    xgen://pubkey/ed25519:...
Endpoint:   ws://127.0.0.1:8080/xgen
Mode:       local
Identities: 0 registered

Listening on ws://127.0.0.1:8080/xgen — press Ctrl+C to stop
```

Expected first log line in `test/node_a/logs/xgen-node_YYYY-MM-DD_HH-MM-SS.log`:
```
YYYY-MM-DD HH:MM:SS.mmm [INFO ] xgen_node: Log file opened: ...
YYYY-MM-DD HH:MM:SS.mmm [INFO ] xgen_node: Node started node_id=xgen://pubkey/ed25519:... endpoint=ws://127.0.0.1:8080/xgen
```

**Record Node A's `node_id`** — you will need it for the federation step.

---

## Step 2 — Start Node B

**Terminal 2 — from `bin/` directory:**
```
xgen-node --config ..\test\node_b\xgen-node_config.toml
```

Same expected output as Node A, on port 8081.

**Record Node B's `node_id`** — you will need it for the federation step.

---

## Step 3 — Register Client 1 on Node A

**Terminal 3 — from `bin/` directory:**
```
xgen-client register --node ws://127.0.0.1:8080/xgen --display-name "TestUser1"
```

Expected console output:
```
Registered successfully.
Identity ID: xgen://pubkey/ed25519:...
```

Expected Node A log entries:
```
[INFO ] xgen_node: Client authenticated identity_id=xgen://pubkey/ed25519:...
[INFO ] xgen_node: Identity registered identity_id=xgen://pubkey/ed25519:...
```

Expected client log entry in `bin/logs/xgen-client_YYYY-MM-DD_HH-MM-SS.log`:
```
[INFO ] xgen_client: Connecting to Node node_url=ws://127.0.0.1:8080/xgen
[INFO ] xgen_client: Authenticated identity_id=xgen://pubkey/ed25519:...
```

**Record Client 1's `identity_id`.**

---

## Step 4 — Register Client 2 on Node B

**Terminal 3 — from `bin/` directory:**
```
xgen-client register --node ws://127.0.0.1:8081/xgen --display-name "TestUser2"
```

Same expected output on Node B.

**Record Client 2's `identity_id`.**

---

## Step 5 — Create Space on Node A

**Terminal 3:**
```
xgen-client create-space --node ws://127.0.0.1:8080/xgen --name "SmokeTestSpace"
```

Expected console output:
```
Space created: SmokeTestSpace
Space ID: xgen://hash/sha256:...
```

Expected Node A log entries:
```
[INFO ] xgen_node: Space created space_id=xgen://hash/sha256:... name=SmokeTestSpace
[DEBUG] xgen_common::event_trace: Event direction=Inbound event_id=xgen://hash/sha256:... event_type=state.space_create sender=xgen://pubkey/ed25519:... space_id=xgen://hash/sha256:... room_id=null timestamp=...
```

**Record the `Space ID`.**

---

## Step 6 — Client 1 joins Space

**Terminal 3:**
```
xgen-client join --node ws://127.0.0.1:8080/xgen --space <Space ID from Step 5>
```

Expected Node A log:
```
[INFO ] xgen_node: Client authenticated identity_id=...
[DEBUG] xgen_common::event_trace: Event direction=Inbound event_id=... event_type=membership.join ...
```

---

## Step 7 — Establish federation between Node A and Node B

**Terminal 3:**
```
xgen-client federate --node ws://127.0.0.1:8080/xgen --space <Space ID> --peer ws://127.0.0.1:8081/xgen
```

Expected Node A log:
```
[INFO ] xgen_node: Federation established peer_node_id=xgen://pubkey/ed25519:... shared_spaces=1
[DEBUG] xgen_common::event_trace: Event direction=Outbound event_id=... event_type=state.federation_add ...
```

Expected Node B log:
```
[INFO ] xgen_node: Federation established peer_node_id=xgen://pubkey/ed25519:...
[DEBUG] xgen_common::event_trace: Event direction=Inbound event_id=... event_type=state.federation_add ...
```

---

## Step 8 — Client 2 joins Space via Node B

**Terminal 3:**
```
xgen-client join --node ws://127.0.0.1:8081/xgen --space <Space ID>
```

Expected Node B log:
```
[DEBUG] xgen_common::event_trace: Event direction=Inbound event_id=... event_type=membership.join sender=<Client2 identity_id> ...
```

---

## Step 9 — Send a message (the pairing verification step)

**Terminal 3 — Client 1 sends a message:**
```
xgen-client send --node ws://127.0.0.1:8080/xgen --space <Space ID> --room RoomA --text "Hello from smoke test"
```

Expected console output:
```
Message sent.
Event ID: xgen://hash/sha256:XXXXXXXXXXXXXXXX...
```

**Record this `Event ID` — this is the pairing key.**

Expected client log (`bin/logs/xgen-client_*.log`):
```
[INFO ] xgen_client: Message sent event_id=xgen://hash/sha256:XXXXXXXXXXXXXXXX... room=RoomA
[DEBUG] xgen_common::event_trace: Event direction=Outbound event_id=xgen://hash/sha256:XXXXXXXXXXXXXXXX... event_type=message.text sender=<Client1 identity_id> space_id=... room_id=... timestamp=...
```

Expected Node A log (`test/node_a/logs/xgen-node_*.log`) — **same event_id, direction=Inbound:**
```
[DEBUG] xgen_common::event_trace: Event direction=Inbound event_id=xgen://hash/sha256:XXXXXXXXXXXXXXXX... event_type=message.text sender=<Client1 identity_id> space_id=... room_id=... timestamp=...
```

Expected Node B log (`test/node_b/logs/xgen-node_*.log`) — **same event_id, federated:**
```
[DEBUG] xgen_common::event_trace: Event direction=Inbound event_id=xgen://hash/sha256:XXXXXXXXXXXXXXXX... event_type=message.text ...
```

---

## Step 10 — Content leak verification

Search all log files for the message text `"Hello from smoke test"`:

```
findstr /s /i "Hello from smoke test" test\node_a\logs\*.log test\node_b\logs\*.log bin\logs\*.log
```

**Expected result: zero matches.** The message content MUST NOT appear in any log file at any level. If it does appear, this is a critical bug in the `trace_event` implementation — the `content` field is being logged and must be removed immediately.

---

## Step 11 — Shut down

Stop Node A and Node B with `Ctrl+C` in their terminal windows.

Expected Node A log final entry:
```
[INFO ] xgen_node: Node shutting down
```

---

## Step 12 — Restore config levels

After the test, restore both node configs to `level = "info"`:

- `test/node_a/xgen-node_config.toml` → `level = "info"`
- `test/node_b/xgen-node_config.toml` → `level = "info"`

---

## Results to report

After completing the test, produce a **Pairing Report** as follows.

---

### Pairing Report

Collect every `Event direction=Outbound` line from the client log and every `Event direction=Inbound` line from both Node logs. Build the following pairing table — one row per event_id:

| event_id (short — first 12 chars after sha256:) | event_type | Client Outbound ✔/✘ | Node A Inbound ✔/✘ | Node B Inbound ✔/✘ | Notes |
|---|---|---|---|---|---|
| a3f9b2c1d4e5 | state.space_create | ✔ | ✔ | — | Only Node A |
| b2c3d4e5f6a7 | membership.join | ✔ | ✔ | — | Client 1 on Node A |
| c3d4e5f6a7b8 | state.federation_add | — | ✔ | ✔ | Node-to-Node only |
| d4e5f6a7b8c9 | membership.join | ✔ | — | ✔ | Client 2 on Node B |
| e5f6a7b8c9d0 | message.text | ✔ | ✔ | ✔ | All three — main test |

*(The rows above are examples — replace with actual event_ids and types from the logs.)*

**What each column means:**
- **Client Outbound ✔** — the client log has `direction=Outbound` for this event_id
- **Node A Inbound ✔** — Node A log has `direction=Inbound` for this event_id
- **Node B Inbound ✔** — Node B log has `direction=Inbound` for this event_id
- **—** — not expected on this Node/client for this EventType
- **✘** — expected but missing — this is a bug

**Expected pairing pattern for this smoke test:**

| EventType | Client Outbound | Node A Inbound | Node B Inbound |
|---|---|---|---|
| `state.space_create` | ✔ (Client 1) | ✔ | — |
| `state.room_create` | ✔ (Client 1) | ✔ | — |
| `membership.join` (Client 1) | ✔ | ✔ | — |
| `state.federation_add` | — | ✔ | ✔ |
| `membership.join` (Client 2) | ✔ | — | ✔ |
| `message.text` | ✔ (Client 1) | ✔ | ✔ |

Any row where an expected ✔ is missing is a bug — flag it with details.

---

### Additional report items

**1. Log files produced — list all files found:**
```
test/node_a/logs/
test/node_b/logs/
bin/logs/
```

**2. Content leak check result:**
- Output of the `findstr` command from Step 10
- Expected: zero matches

**3. Timing deltas — for the `message.text` Event:**
From the three log lines for the same `message.text` event_id, extract the timestamps and compute:
- Client Outbound timestamp
- Node A Inbound timestamp → delta from Client Outbound (network round trip Node A)
- Node B Inbound timestamp → delta from Node A Inbound (federation propagation delay)

Format:
```
Client  Outbound:  2026-04-30 14:35:22.401  (base)
Node A  Inbound:   2026-04-30 14:35:22.418  (+17ms)
Node B  Inbound:   2026-04-30 14:35:22.531  (+113ms from Node A)
```

This establishes the baseline latency profile for Phase 1.

**4. Test outcome:**
- PASS — full pairing table complete, all expected pairs matched, no content leak
- PARTIAL — some pairs missing — list which EventTypes were unpaired and on which Node
- FAIL — describe which step failed and paste the relevant log lines

**5. Any unexpected log entries** — paste anything that looks wrong or surprising.

---

## Files modified by this test

| File | Change |
|---|---|
| `test/node_a/xgen-node_config.toml` | Temporarily set `level = "debug"` — restore to `"info"` after |
| `test/node_b/xgen-node_config.toml` | Temporarily set `level = "debug"` — restore to `"info"` after |
| `test/node_a/logs/` | New log file created per run |
| `test/node_b/logs/` | New log file created per run |
| `bin/logs/` | New client log file created per run |

---

*End of smoke test instructions*
