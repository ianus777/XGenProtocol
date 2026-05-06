# Appendix F — CLI Reference and Usage Examples
> **Status:** ACTIVE  
> **Last updated:** 2026-05-06  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1  

This appendix provides the complete CLI reference for `xgen-node` and `xgen-client`, followed by real-world usage examples covering common operator and user workflows. It is the authoritative reference for CLI syntax. The Rust source doc comments in `xgen-node/src/main.rs` and `xgen-client/src/main.rs` MUST match this appendix exactly (D-028).

---

## F.1 Configuration file reference

### xgen-node_config.toml

Full example with all supported fields:

```toml
[node]
listen = "ws://127.0.0.1:8080/xgen"
local_mode = true

[paths]
keypair_path = "./xgen-node_keypair.enc"

[logging]
level = "info"
```

| Field | Section | Description |
|---|---|---|
| `listen` | `[node]` | WebSocket endpoint the Node binds to |
| `local_mode` | `[node]` | `true` = no Trust Assertion required for registration |
| `keypair_path` | `[paths]` | Path to the encrypted keypair file. Default: `./xgen-node_keypair.enc` |
| `level` | `[logging]` | Log verbosity: `off` / `error` / `warn` / `info` / `debug` / `trace`. Default: `info` |

All other data paths (`spaces/`, `logs/`, `audit/`) are derived automatically from the Node's working directory and are not configurable (D-035).

### xgen-client_config.toml

```toml
[client]
node_url = "ws://127.0.0.1:8080/xgen"

[paths]
keypair_path = "./xgen-client_keypair.enc"
state_path = "./xgen-client_state.json"

[logging]
level = "info"
```

| Field | Section | Description |
|---|---|---|
| `node_url` | `[client]` | Default Node endpoint — used when `--node` is not provided |
| `keypair_path` | `[paths]` | Path to the encrypted keypair file |
| `state_path` | `[paths]` | Path to the client state file |
| `level` | `[logging]` | Log verbosity: same values as Node |

---

## F.2 xgen-node — Complete command reference

```
xgen-node [OPTIONS] [COMMAND]
```

When invoked with no subcommand, starts the Node in foreground mode.

**Global options:**

| Option | Short | Description |
|---|---|---|
| `--config <path>` | `-c` | Config file path. Default: `./xgen-node_config.toml` |
| `--local` | | Start in Local Node mode regardless of config |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version and build info |

**Subcommands:**

| Command | Description |
|---|---|
| `init` | Generate keypair and default config in current directory |
| `status` | Print current Node status from state file |
| `connections` | List all connected clients and federated peers |
| `spaces` | List all hosted Spaces with Rooms and event counts |
| `peers` | List all known federated peer Nodes |
| `identity list` | List all registered Identities |
| `version` | Print version and build metadata |

---

## F.3 xgen-client — Complete command reference

```
xgen-client [OPTIONS] <COMMAND>
```

**Global options:**

| Option | Short | Description |
|---|---|---|
| `--node <endpoint>` | `-n` | Node WebSocket endpoint. Overrides config. |
| `--config <path>` | `-c` | Config file path. Default: `./xgen-client_config.toml` |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version and build info |

**Subcommands:**

| Command | Arguments | Description |
|---|---|---|
| `init` | — | Generate keypair and default config in current directory |
| `whoami` | — | Print local Identity ID and state. No Node connection required. |
| `register` | `--name <name>` | Register this Identity on the Node |
| `create-space` | `--name <name>` | Create a new Space. Caller becomes owner. |
| `create-room` | `--space <id>` `--name <name>` | Create a Room in a Space |
| `invite` | `--space <id>` `--identity <id>` `--role <role>` | Invite an Identity to a Space |
| `join` | `--space <id>` | Join a Space (accept an invite or join an open Space) |
| `send` | `--space <id>` `--room <id>` `--text <text>` | Send a text message to a Room |
| `spaces` | — | List Spaces this Identity has joined |
| `rooms` | `--space <id>` | List Rooms in a Space |
| `members` | `--space <id>` | List members of a Space |
| `federate` | `--space <id>` `--peer <endpoint>` | Initiate federation for a Space with a peer Node |
| `smoke-test` | `--node-a <ep>` `--node-b <ep>` | Run the Phase 1 17-step smoke test |
| `version` | — | Print version and build metadata |

**Role values for `--role`:** `owner` / `admin` / `moderator` / `member`

---

## F.4 Usage examples — Node operator workflows

### F.4.1 First-time Node setup

```
mkdir E:\XGen\XGenNode_A
cd E:\XGen\XGenNode_A
xgen-node init
```

Output:
```
Generating keypair...
Passphrase: ********
Confirm:    ********
Keypair saved:  ./xgen-node_keypair.enc
Config saved:   ./xgen-node_config.toml
Node ID: xgen://pubkey/ed25519:Cazue8SnVwub0khckedqjHwtwjP8WLGuJOdj6h1bA68
Run 'xgen-node' to start.
```

Then edit `xgen-node_config.toml` to set the listen address and logging level, then:

```
xgen-node
```

### F.4.2 Check Node status while running

In a second terminal, from the Node's working directory:

```
xgen-node status
```

Output:
```
xgen-node status
================
Node ID:      xgen://pubkey/ed25519:Cazue8...
Version:      0.10.3 (build 260429-2152)
Uptime:       0h 14m 22s
Mode:         Local Node
Endpoint:     ws://127.0.0.1:8080/xgen
Connections:  2 clients, 1 federated peer
Spaces:       1 hosted
Events:       47 total across all spaces
```

### F.4.3 List all registered identities on the Node

```
xgen-node identity list
```

Output:
```
Registered Identities (2)

  xgen://pubkey/ed25519:MiB5Ew...   Alice    registered 14m ago   1 device
  xgen://pubkey/ed25519:GRqRup...   Bob      registered  9m ago   1 device
```

### F.4.4 Check active federation

```
xgen-node peers
```

Output:
```
Federated Peers (1)

  Node ID:     xgen://pubkey/ed25519:kU90as...
  Endpoint:    ws://127.0.0.1:8081/xgen
  State:       ACTIVE
  Spaces:      SmokeTestSpace
  Connected:   14m 20s ago
  Last seen:   3s ago
```

### F.4.5 Enable debug logging temporarily

Change `level = "debug"` in `xgen-node_config.toml` and restart, or use the environment override without restarting:

```
set XGEN_LOG=debug
xgen-node
```

Module-specific debug (federation only):
```
set XGEN_LOG=xgen_node_lib::federation=debug,info
xgen-node
```

Restore to normal:
```
set XGEN_LOG=
xgen-node
```

---

## F.5 Usage examples — Identity and Space workflows

### F.5.1 Set up a new client identity

```
mkdir E:\XGen\XGenClient_Alice
cd E:\XGen\XGenClient_Alice
xgen-client init
```

Output:
```
Generating keypair...
Passphrase: ********
Confirm:    ********
Keypair saved:    ./xgen-client_keypair.enc
Config saved:     ./xgen-client_config.toml
Identity ID: xgen://pubkey/ed25519:MiB5EwRXFRzccw...
Run 'xgen-client register --name "Your Name"' to register on a Node.
```

### F.5.2 Register identity on a Node

```
xgen-client --node ws://127.0.0.1:8080/xgen register --name "Alice"
```

Output:
```
Registered.
Identity ID: xgen://pubkey/ed25519:MiB5EwRXFRzccw...
```

Check local identity state:
```
xgen-client whoami
```

Output:
```
Identity ID:    xgen://pubkey/ed25519:MiB5EwRXFRzccw...
Display name:   Alice
Registered on:  ws://127.0.0.1:8080/xgen
Spaces joined:  0
```

### F.5.3 Create a Space and a Room

```
xgen-client --node ws://127.0.0.1:8080/xgen create-space --name "Project Alpha"
```

Output:
```
Space created.
Space ID: xgen://hash/sha256:9ba66d487573c1f64f1f14976efe9b0f8a016c44df805563cf541bb2d35f094c
```

```
xgen-client --node ws://127.0.0.1:8080/xgen create-room \
  --space xgen://hash/sha256:9ba66d487573... \
  --name "general"
```

Output:
```
Room created.
Room ID: xgen://hash/sha256:9cb9acbef9720c6fb0f1846e773c0d8ea97773db85b5d7976c249557938f7853
```

### F.5.4 Invite another Identity to the Space

Alice invites Bob (Bob's identity_id obtained from Node identity list or out-of-band):

```
xgen-client --node ws://127.0.0.1:8080/xgen invite \
  --space xgen://hash/sha256:9ba66d487573... \
  --identity xgen://pubkey/ed25519:GRqRupb9GudeJu5xiXtWiISwPOYo28nNcbkU1sdn3FM \
  --role member
```

Output:
```
Invite sent.
```

### F.5.5 Accept an invitation and join the Space (Bob's side)

From Bob's client directory, on Node B:

```
xgen-client --node ws://127.0.0.1:8081/xgen join \
  --space xgen://hash/sha256:9ba66d487573...
```

Output:
```
Joined Space: Project Alpha
```

### F.5.6 List Spaces, Rooms, and members

```
xgen-client spaces
```

Output:
```
Spaces (1)

  Project Alpha
  ID:      xgen://hash/sha256:9ba66d487573...
  Node:    ws://127.0.0.1:8080/xgen
  Role:    owner
  Rooms:   1
  Members: 2
```

```
xgen-client rooms --space xgen://hash/sha256:9ba66d487573...
```

Output:
```
Rooms in Project Alpha (1)

  general
  ID: xgen://hash/sha256:9cb9acbef972...
```

```
xgen-client members --space xgen://hash/sha256:9ba66d487573...
```

Output:
```
Members of Project Alpha (2)

  xgen://pubkey/ed25519:MiB5Ew...   Alice   owner    registered 14m ago
  xgen://pubkey/ed25519:GRqRup...   Bob     member   registered  9m ago
```

### F.5.7 Send messages

Alice sends a message:

```
xgen-client --node ws://127.0.0.1:8080/xgen send \
  --space xgen://hash/sha256:9ba66d487573... \
  --room xgen://hash/sha256:9cb9acbef972... \
  --text "Hello Bob, welcome to Project Alpha!"
```

Output:
```
Message sent.
Event ID: xgen://hash/sha256:e97c46b1e8d842a0cedd9862dabc66a82869744e50c2a43a67ef5d4fadf39434
```

Bob replies from Node B:

```
xgen-client --node ws://127.0.0.1:8081/xgen send \
  --space xgen://hash/sha256:9ba66d487573... \
  --room xgen://hash/sha256:9cb9acbef972... \
  --text "Hello Alice! Glad to be here."
```

Output:
```
Message sent.
Event ID: xgen://hash/sha256:9179066b77712b33a010787c3054b924ed8535edae0052260ee70a0f3d0814c4
```

### F.5.8 Establish federation between two Nodes

Alice (Space owner on Node A) initiates federation so Node B can participate in the Space:

```
xgen-client --node ws://127.0.0.1:8080/xgen federate \
  --space xgen://hash/sha256:9ba66d487573... \
  --peer ws://127.0.0.1:8081/xgen
```

Output:
```
Federation initiated.
Peer Node ID: xgen://pubkey/ed25519:kU90as7POYA8w9yRANpsFdYjjP32LJKpyUCQau6qAUA
Events synced: 4
```

---

## F.6 Usage examples — Two-Node full session

This example shows the complete sequence for a real two-Node session from scratch. It mirrors the Phase 1 smoke test.

**Prerequisites:** Node A running on port 8080, Node B running on port 8081. Alice registered on Node A, Bob registered on Node B.

```bash
# -- Alice (Node A) --

# Create the Space
xgen-client --node ws://127.0.0.1:8080/xgen create-space --name "SmokeTestSpace"
# → Space ID: xgen://hash/sha256:9ba66d48...

# Create a Room
xgen-client --node ws://127.0.0.1:8080/xgen create-room \
  --space xgen://hash/sha256:9ba66d48... \
  --name "general"
# → Room ID: xgen://hash/sha256:9cb9acbe...

# Invite Bob
xgen-client --node ws://127.0.0.1:8080/xgen invite \
  --space xgen://hash/sha256:9ba66d48... \
  --identity xgen://pubkey/ed25519:GRqRup... \
  --role member

# Federate with Node B
xgen-client --node ws://127.0.0.1:8080/xgen federate \
  --space xgen://hash/sha256:9ba66d48... \
  --peer ws://127.0.0.1:8081/xgen

# -- Bob (Node B) --

# Join the Space
xgen-client --node ws://127.0.0.1:8081/xgen join \
  --space xgen://hash/sha256:9ba66d48...

# Join the Room
xgen-client --node ws://127.0.0.1:8081/xgen join \
  --space xgen://hash/sha256:9ba66d48... \
  --room xgen://hash/sha256:9cb9acbe...

# -- Exchange messages --

# Alice sends
xgen-client --node ws://127.0.0.1:8080/xgen send \
  --space xgen://hash/sha256:9ba66d48... \
  --room xgen://hash/sha256:9cb9acbe... \
  --text "Hello Bob"

# Bob replies
xgen-client --node ws://127.0.0.1:8081/xgen send \
  --space xgen://hash/sha256:9ba66d48... \
  --room xgen://hash/sha256:9cb9acbe... \
  --text "Hello Alice"
```

---

## F.7 `--help` output requirements

Every command and subcommand MUST produce useful `--help` output generated from Rust doc comments via `clap` derive API. The following format is required for all subcommands:

```
xgen-client-send
Send a text message to a Room

USAGE:
    xgen-client [OPTIONS] send [SEND OPTIONS]

OPTIONS:
    --space <space-id>    Space ID (xgen://hash/sha256:...)
    --room  <room-id>     Room ID (xgen://hash/sha256:...)
    --text  <text>        Message text to send

EXAMPLES:
    xgen-client --node ws://127.0.0.1:8080/xgen send \
        --space xgen://hash/sha256:9ba66d48... \
        --room  xgen://hash/sha256:9cb9acbe... \
        --text "Hello!"

    -h, --help    Print help information
```

Requirements:
- Every argument has a description
- Every subcommand has at least one `EXAMPLES:` entry
- The `EXAMPLES:` section uses real, runnable commands — not placeholders like `<YOUR_SPACE_ID>`
- The examples in `--help` MUST match the examples in this appendix exactly

The canonical source of argument descriptions and examples is this appendix (F.2, F.3, F.4, F.5, F.6). The Rust doc comments that generate `--help` output MUST match this appendix. When this appendix changes, the Rust source MUST be updated to match.

---

## F.8 Session log

### Session 1 — April 2026 (JozefN)
**Covered:** Appendix F written in full covering: F.1 configuration file reference for both binaries including the new `[logging]` section and the removal of `log_path`/`spaces_dir` from config (D-035); F.2 complete xgen-node command reference; F.3 complete xgen-client command reference; F.4 Node operator workflow examples (setup, status, identity list, federation check, debug logging); F.5 Identity and Space workflow examples (init, register, create-space, create-room, invite, join, spaces/rooms/members list, send message, federate); F.6 complete two-Node full session example mirroring the Phase 1 smoke test with real event_ids from J-029; F.7 `--help` output requirements and the canonical source rule (D-028).
