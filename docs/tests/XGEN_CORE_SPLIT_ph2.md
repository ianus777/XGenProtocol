# XGen Protocol — xgen-core Crate Split
> **Status:** PENDING  
> Version: 1.0  
> Date: May 2026  
> **Last updated:** 2026-05-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Extract all shared protocol logic from `xgen-node` into a new `xgen-core` crate. This is the prerequisite task for Phase 2 protocol implementation. It must be completed before any Phase 2 protocol code is written.

**Decision references:** D-022 (xgen-core library split), D-029 (xgen-client → xgen-node dependency, temporary Phase 1 arrangement replaced here).

---

## Why This Must Be Done First

Currently all protocol logic lives in `xgen-node/src/`. `xgen-client` imports it directly via `xgen-node = { path = "../xgen-node" }` (D-029 — a temporary Phase 1 arrangement). This creates two problems:

1. `xgen-client` depends on a binary crate's library — coupling that was always intended to be temporary
2. Any Phase 2 protocol code written now goes into `xgen-node`, increasing the cost of the eventual split

After this task: `xgen-core` holds all shared protocol logic. Both `xgen-node` and `xgen-client` are thin shells that depend on `xgen-core`. Third-party developers can build XGen-compatible implementations by depending on `xgen-core` alone.

---

## License

`xgen-core` is **GPL-2.0-or-later from day one** — not BSL 1.1. This is intentional. It is the public protocol library that the XGen ecosystem builds on. See D-022.

Every source file in `xgen-core/src/` carries this header:

```rust
// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.
```

`xgen-node`, `xgen-client`, and `xgen-common` retain BSL 1.1 headers unchanged.

---

## Current State

```
xgen-common/src/
  lib.rs
  wire.rs          ← Event, EventType — shared serde types
  state.rs         ← ApplicationState enum
  event_trace.rs   ← global event tracing interface
  build_info.rs    ← version constants

xgen-node/src/
  main.rs          ← thin CLI entry point
  lib.rs           ← all protocol logic re-exported
  lifecycle.rs     ← UI lifecycle state machine (Tauri-specific)
  crypto/
    mod.rs
    encoding.rs    ← base64url encode/decode
    hashing.rs     ← SHA-256 Event ID derivation
    signing.rs     ← Ed25519 sign and verify
  wire/
    mod.rs
    types.rs       ← all message type structs (serde)
    canonical.rs   ← canonical JSON form for signing
    framing.rs     ← transport frame encode/decode
    validation.rs  ← Event validation pipeline steps 1–7
  dag/
    mod.rs
    store.rs       ← append-only Event store
    graph.rs       ← DAG tips, prev_events tracking
    pending.rs     ← pending buffer for missing predecessors
  transport/
    mod.rs
    server.rs      ← WebSocket server (Node-specific)
    client.rs      ← outbound WebSocket connection
    connection.rs  ← connection lifecycle, 4 phases
    auth.rs        ← challenge-response authentication
  node/
    mod.rs
    announcement.rs ← node announcement production and verification
    runtime.rs      ← NodeRuntime — orchestrates all modules
  federation/
    mod.rs
    handshake.rs   ← federation state machine
    registry.rs    ← federation relationship registry
  identity/
    mod.rs
    keypair.rs     ← Ed25519 keypair generation and encrypted storage
    registration.rs ← 8-step Identity registration pipeline
    registry.rs    ← Identity record store
  space/
    mod.rs
    state.rs       ← Space and Room state machine
    membership.rs  ← membership role and permission logic
  message/
    mod.rs
    exchange.rs    ← validation steps 8–13 and accept_event
  tests/
    mod.rs
    smoke.rs       ← Phase 1 17-step smoke test

xgen-client/src/
  main.rs          ← thin CLI entry point
  lib.rs
  commands.rs      ← CLI command implementations
```

---

## Target State

```
xgen-common/       ← unchanged
xgen-core/         ← NEW crate (GPL-2.0-or-later)
  Cargo.toml
  src/
    lib.rs
    crypto/        ← moved from xgen-node
    wire/          ← moved from xgen-node
    dag/           ← moved from xgen-node
    transport/
      mod.rs
      client.rs    ← moved from xgen-node
      connection.rs ← moved from xgen-node
      auth.rs      ← moved from xgen-node
      (server.rs stays in xgen-node — Node-specific)
    node/          ← moved from xgen-node
    federation/    ← moved from xgen-node
    identity/      ← moved from xgen-node
    space/         ← moved from xgen-node
    message/       ← moved from xgen-node

xgen-node/         ← thin shell (BSL 1.1)
  Cargo.toml       ← xgen-core dependency added, most crates removed
  src/
    main.rs        ← unchanged
    lib.rs         ← re-exports xgen-core public API
    lifecycle.rs   ← unchanged (Tauri UI lifecycle, Node-specific)
    transport/
      mod.rs
      server.rs    ← stays here — WebSocket server is Node-specific
    tests/         ← stays here — smoke tests run against Node runtime
      mod.rs
      smoke.rs

xgen-client/       ← thin shell (BSL 1.1)
  Cargo.toml       ← xgen-core dependency added, xgen-node dependency removed
  src/
    main.rs        ← unchanged
    lib.rs         ← re-exports xgen-core public API
    commands.rs    ← unchanged
```

---

## Step-by-Step Instructions

### Step 1 — Create the xgen-core crate

Add `xgen-core` to the workspace `Cargo.toml`:

```toml
[workspace]
members = [
    "xgen-common",
    "xgen-core",
    "xgen-node",
    "xgen-client",
]
```

Create `xgen-core/Cargo.toml`:

```toml
[package]
name = "xgen-core"
version = "0.10.3"
edition = "2021"
license = "GPL-2.0-or-later"

[lib]
name = "xgen_core"
path = "src/lib.rs"

[dependencies]
xgen-common = { path = "../xgen-common" }
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = { version = "0.21", default-features = false, features = ["connect"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ed25519-dalek = "2"
sha2 = "0.10"
rand = "0.8"
base64 = "0.21"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
tracing = "0.1"
anyhow = "1"
thiserror = "1"
chacha20poly1305 = "0.10"
argon2 = "0.5"
futures-util = { version = "0.3", default-features = false, features = ["sink", "std"] }

[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.59", features = ["Win32_System_LibraryLoader"] }

[dev-dependencies]
tempfile = "3"
```

Add a GPL-2.0 license file `LICENSE-CORE` in the project root with the standard GPL-2.0-or-later text.

---

### Step 2 — Move modules from xgen-node to xgen-core

Move the following directories from `xgen-node/src/` to `xgen-core/src/` **exactly as-is** — do not modify any logic during the move:

- `crypto/` (all files)
- `wire/` (all files)
- `dag/` (all files)
- `node/` (all files — both `announcement.rs` and `runtime.rs`)
- `federation/` (all files)
- `identity/` (all files)
- `space/` (all files)
- `message/` (all files)

From `transport/`, move only:
- `transport/client.rs`
- `transport/connection.rs`
- `transport/auth.rs`
- `transport/mod.rs` — update to not declare `server` module

**Do NOT move:**
- `transport/server.rs` — stays in `xgen-node/src/transport/`
- `lifecycle.rs` — stays in `xgen-node/src/`
- `tests/` — stays in `xgen-node/src/`
- `main.rs`, `lib.rs` — stays in both binaries

After moving, update all copyright headers in `xgen-core/src/` to GPL-2.0-or-later (see License section above).

---

### Step 3 — Create xgen-core/src/lib.rs

```rust
// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

pub mod crypto;
pub mod wire;
pub mod dag;
pub mod transport;
pub mod node;
pub mod federation;
pub mod identity;
pub mod space;
pub mod message;
```

---

### Step 4 — Update xgen-node

Update `xgen-node/Cargo.toml` — add `xgen-core` dependency, remove crates now provided through `xgen-core`:

```toml
[dependencies]
xgen-common = { path = "../xgen-common" }
xgen-core = { path = "../xgen-core" }
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = { version = "0.21", default-features = false, features = ["connect"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "chrono"] }
anyhow = "1"
thiserror = "1"
clap = { version = "4", features = ["derive"] }
rpassword = "7"
toml = "0.8"
```

Update `xgen-node/src/lib.rs` — replace all internal module declarations with re-exports from `xgen-core`:

```rust
// Re-export xgen-core public API
pub use xgen_core::crypto;
pub use xgen_core::wire;
pub use xgen_core::dag;
pub use xgen_core::transport;
pub use xgen_core::node;
pub use xgen_core::federation;
pub use xgen_core::identity;
pub use xgen_core::space;
pub use xgen_core::message;

// Node-specific modules
pub mod lifecycle;

mod transport_server {
    pub use super::transport_server_impl::*;
}
```

Keep `xgen-node/src/transport/` containing only `server.rs` and a `mod.rs` that declares it. Update `xgen-node/src/lib.rs` to also expose the Node-specific server:

```rust
pub mod server_transport {
    pub use crate::transport_node::server;
}
```

Adjust as needed so that `main.rs` and `tests/smoke.rs` compile without changes to their import paths — update import paths in those files if necessary to point to `xgen_core::` instead of the old local module paths.

---

### Step 5 — Update xgen-client

Update `xgen-client/Cargo.toml` — remove `xgen-node` dependency, add `xgen-core`:

```toml
[dependencies]
xgen-common = { path = "../xgen-common" }
xgen-core = { path = "../xgen-core" }
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = { version = "0.21", default-features = false, features = ["connect"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "chrono"] }
anyhow = "1"
thiserror = "1"
clap = { version = "4", features = ["derive"] }
rpassword = "7"
toml = "0.8"
shlex = "1"
```

Update `xgen-client/src/lib.rs` and `xgen-client/src/commands.rs` — replace all `xgen_node_lib::` import paths with `xgen_core::`.

---

### Step 6 — Verify

**Step 6a — Unit and integration tests**

Run the full test suite. All 173 tests must pass. No new tests are written for this task — the split is a structural refactor only. Zero behaviour change.

```sh
cargo test
```

If tests fail, the cause is always one of: missing import path update, missing `pub` visibility on a moved item, or a module declaration left in the wrong place. Fix systematically — do not change any logic.

Run the in-process smoke test explicitly:

```sh
cargo test smoke
```

**Step 6b — Build release binaries**

Build both binaries in release mode to confirm the full compilation chain is clean:

```sh
cargo build --release
```

Both `xgen-node` and `xgen-client` must compile without warnings or errors.

**Step 6c — Live two-node smoke test**

Run the Phase 1 smoke test against two real running Node processes over TCP — the same verification that confirmed Phase 1 complete (J-029, tag v0.10.3):

```sh
# Terminal 1 — start Node A
xgen-node --instance node-a

# Terminal 2 — start Node B
xgen-node --instance node-b

# Terminal 3 — run smoke test against live nodes
xgen-client smoke-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen
```

All 17 steps must pass. This is the definitive proof that the crate split introduced zero behaviour change.

---

### Step 7 — Record decisions and update CLAUDE.md

Add a DECISIONS.md entry: D-044 — xgen-core crate split executed. Note: D-022 (planned) and D-029 (temporary xgen-client → xgen-node dependency) are both resolved by this task. Update their entries to reference D-044.

Update `CLAUDE.md`:
- Mark D-022 resolved
- Add `xgen-core/` to the Repository Layout section
- Note the GPL license on xgen-core

Add a JOURNAL.md entry for this session.

---

## Definition of Done

- [ ] `xgen-core` crate exists with GPL-2.0-or-later license
- [ ] All protocol modules live in `xgen-core/src/`
- [ ] `xgen-node` depends on `xgen-core`, not its own internal modules
- [ ] `xgen-client` depends on `xgen-core`, not `xgen-node`
- [ ] `transport/server.rs` remains in `xgen-node`
- [ ] `lifecycle.rs` remains in `xgen-node`
- [ ] All 173 tests pass (`cargo test`)
- [ ] In-process smoke test passes (`cargo test smoke`)
- [ ] Release binaries build cleanly (`cargo build --release`)
- [ ] Live two-node smoke test passes (17 steps over real TCP)
- [ ] D-022 and D-029 marked resolved in DECISIONS.md
- [ ] D-044 entry written
- [ ] CLAUDE.md updated
- [ ] JOURNAL.md entry written

---

## What Comes After

Once this task is complete, Phase 2 protocol implementation begins. All new Phase 2 protocol code goes directly into `xgen-core/src/`. See `IMPLEMENTATION_GUIDE_ph2.md` for the Phase 2 layer sequence.
