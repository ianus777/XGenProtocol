# M9.2 — Harness-Enablement Seams (F2 + F3 + F4) — Implementation Runbook

> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose + DoD

Executes the J-313 Joe-LOCKED design (`tasks/M9_2_HARNESS_SEAMS_DESIGN.md`, M9.2-D1..D5). Adds the
three fenced harness-enablement seams: **F2** peer-URL seeding, **F3** clock-advance, **F4** raw-frame
injection. Clair implements.

**DoD:** the fence test passes (verbs absent in default build, present under `--features
harness-control`); `cargo build --workspace` (default) = 0 **and** `cargo build -p xgen-node
--features harness-control` = 0; clippy clean both; the three §4 smokes pass `--features
harness-control` (and are `#[ignore]` so the fast suite stays **1269/0/8**); AUDIT/DESIGN/IMPL →
COMPLETED at close.

---

## 2. Grounded surfaces (confirmed against live `main` — do not re-discover)

- **Fence target:** `xgen-node/Cargo.toml` currently has `[features] store-sqlite = [...]` only;
  `[dependencies]` has `xgen-common = { path = "../xgen-common" }` (normal dep, no features) →
  `harness-control = ["xgen-common/mock-clock"]` forwards cleanly.
- **aicontrol dispatch:** verbs are strings dispatched via a clap `admin_ops::AdminCli` → `AdminCommand`
  → the `match cmd.as_str()` arm in `aicontrol.rs:339+` (e.g. `"federation initiate" =>
  cap!(admin_ops::federation_initiate(&mut ctx, de(args)?))`). Verb classification consts
  `FEDERATION_VERBS` (aicontrol.rs:148), `verb_tier` (:151), `primary_field` (:162).
- **F2 setter:** `NodeRuntime::record_peer_url(&mut self, node_id: &str, url: String)`
  (`runtime.rs:464`) — inserts into the `pub peer_urls: HashMap<NodeXgid,String>` (:242). The handler
  calls this; the existing `federation initiate` then targets the now-known peer.
- **F3 clock:** `NodeRuntime::set_clock(&mut self, Arc<dyn Clock>)` (`runtime.rs:380`), `clock(&self) ->
  Arc<dyn Clock>` (:387). `MockClock` (`xgen-common/src/clock.rs`) has `new()` + `advance(&self,
  Duration)` (interior mutability) — **no `set` yet**. Binary builds the runtime at
  `app.rs:680` (`let mut runtime = NodeRuntime::new(signing_key.clone());`); the binary never calls
  `set_clock` today (default `RealClock`).
- **F4 client:** `xgen-mptest` already depends on `tokio-tungstenite` 0.21 ("connect") and already has
  a raw-wire injector (`xgen-mptest/src/injector.rs`) that opens the real `ws://` transport — C3
  extends it; no new crate/dep. `dial.rs:20` already documents the F3 gap.
- **aicontrol client (for smokes):** `xgen-mptest`'s `AicontrolClient::connect(&node.aicontrol_pipe,
  …)` (`aicontrol.rs`/`bench.rs:179`) sends verb lines to a spawned binary.

---

## 3. Commit plan (D5 — three work commits; all production seams `#[cfg(feature = "harness-control")]`)

### C1 — the fence + F2 `federation add-peer`
1. `xgen-node/Cargo.toml` `[features]`: add `harness-control = ["xgen-common/mock-clock"]`.
2. `admin_ops.rs` (cfg-gated): add a `federation add-peer` subcommand to the clap `AdminCli` /
   `AdminCommand` (args: `node_id`, `url`) + `pub fn federation_add_peer(ctx, args)` calling
   `ctx`'s runtime `record_peer_url(&node_id, url)`. Returns the seeded `peer_node_id`.
3. `aicontrol.rs` (cfg-gated): dispatch arm `"federation add-peer" => cap!(admin_ops::federation_add_peer(&mut ctx, de(args)?))`;
   add `"federation add-peer"` to `FEDERATION_VERBS` (→ `verb_tier` Federation) and `primary_field`
   (`"peer_node_id"`).

### C2 — F3 clock verb + mock-clock-in-binary + startup install
1. `xgen-common/src/clock.rs` (under the existing `mock-clock`/`cfg(test)` gate): add
   `MockClock::set_now(&self, DateTime<Utc>)` (mirror `advance`'s interior-mutability shape) so
   `clock set` has a primitive. (`advance` already exists.)
2. `app.rs` startup (`~:680`, cfg-gated): after `NodeRuntime::new(...)`, build
   `let mock = Arc::new(MockClock::new()); runtime.set_clock(mock.clone());` and **stash `mock` into the
   admin/app shared state** (a `#[cfg(feature="harness-control")]` field, e.g.
   `AdminContext.mock_clock: Arc<MockClock>`). Default build (no feature) leaves `RealClock` untouched.
   - **⚠ Key subtlety (the one trap):** the clock verb must advance the **same** instance installed at
     startup. `runtime.clock()` returns `Arc<dyn Clock>` and `advance`/`set_now` are **not** on the
     `Clock` trait — so the handler uses the **stashed `Arc<MockClock>` handle**, not a downcast of the
     trait object. Do not add `advance` to the `Clock` trait (keeps the production trait clean).
3. `admin_ops.rs` (cfg-gated): new `clock` category — `clock advance <duration>` / `clock set
   <rfc3339>` subcommands + `pub fn clock_advance/clock_set(ctx, args)` calling
   `ctx.mock_clock.advance(d)` / `.set_now(ts)`.
4. `aicontrol.rs` (cfg-gated): dispatch arms + verb-table entries for the two clock verbs.

### C3 — F4 raw client (xgen-mptest only — NO production change, NO fence)
Extend `xgen-mptest/src/injector.rs` (already holds the ws connection) with a raw byte-level write that
sends a **truncated / malformed frame** (bypassing the typed `send_*`), to exercise the node's frame
parser. Test-crate-only; needs no `harness-control` (the parser is in the normal build) and adds no
`Connection::send_raw` to any production crate.

---

## 4. Proof obligations (the fence test + three smokes)

- **Fence test (D1 — the gate).** Two cfg-split unit tests in `xgen-node`: under
  `#[cfg(not(feature="harness-control"))]`, parsing `"federation add-peer"` / `"clock advance"` via
  `AdminCli` yields **unknown-verb** (the subcommands are cfg'd out); under
  `#[cfg(feature="harness-control")]`, they **parse**. This is the named proof that the surface cannot
  exist in a default/release build.
- **F2 smoke** (`#[ignore]`, xgen-mptest, harness-control build): spawn two fresh nodes A/B;
  `AicontrolClient` `federation add-peer` each direction → `federation initiate` → a Space replicates
  A↔B (the cross-node bootstrap that was impossible before).
- **F3 smoke** (`#[ignore]`): `clock advance 2d` on a running node, observe `now_utc()` moved via a
  time-dependent surface (e.g. an invite `valid_until` boundary flips), deterministically.
- **F4 smoke** (`#[ignore]`): the injector raw-writes a truncated frame → the node rejects it at frame
  parse, no panic, connection closes cleanly (MP-A-12).

---

## 5. Checkpoints (light)

1. **Fence checkpoint (primary):** the fence test passes; grep-confirm every new verb
   registration/dispatch/handler is `#[cfg(feature="harness-control")]`; the F4 path lives only in
   `xgen-mptest` (no `Connection::send_raw` in any production crate).
2. **Build:** `cargo build --workspace --all-targets` (default) = 0 **and** `cargo build -p xgen-node
   --features harness-control --all-targets` = 0.
3. **Clippy:** clean on default and `--features harness-control`.
4. **Suite unchanged:** the smokes are `#[ignore]`; the fast suite stays **1269/0/8**.

---

## 6. At close (doc-only — Chat seat)

Three work commits (C1 / C2 / C3) ship the seams (Clair); then the Chat doc-only close: AUDIT/DESIGN/IMPL
→ COMPLETED, `JOURNAL` (J-314), `ROADMAP` (M9.2 🟢 → ✅ CLOSED; version bump), `CLAUDE` PLAY. No
DECISIONS change (M9.2-D# arc-local, D-069). DoD has **no** "commit pushed" line. Joe pushes all.

**Next-active after close: Multiparty-tests** (runs the harness R1→R2→R3 on the finalized binary; F2
unblocks MP-C-02/03/04/14, F3 unblocks R1 + MP-A-01, F4 unblocks MP-A-12).

**Entry point for Clair (Rule 0):** CLAUDE PLAY → JOURNAL J-313 → this runbook §2 + §3 →
`tasks/M9_2_HARNESS_SEAMS_DESIGN.md` §3 + §4.

Per D-065 + D-069 + D-071 + D-074 + D-078.
