# M7-events arc — Implementation Runbook (Clair build plan)
> **Status**: ACTIVE  
> Version: 1.4  
> Date: Jun 2026  
> **Last updated**: 2026-06-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

The Clair-facing build plan for the M7-events arc under the locked `EV-D1`–`EV-D6` (`tasks/M7_EVENTS_DESIGN.md`). Adds the client + node `.events` pipes deferred from M7 `--aicontrol` v1 (J-205), on top of the gating Node multi-connection-per-identity fan-out change. Grounded against the live tree per the Phase-0 audit (`tasks/M7_EVENTS_AUDIT.md`).

**Discipline:** D-065 (adapter, not feature — events-pipe specialness lives at its own consumer, the fan-out core stays kind-agnostic); D-066 (`--batch` untouched; `pipe.rs` / `batch.rs` not touched); D-067 (no-drift — one shared predicate); D-074 (atomic close including JOURNAL); D-078 (confirm-at-pickup). `git add <file>` per file, never `git add .`. DoD has no "commit pushed" item.

---

## Grounding (live-tree facts this plan relies on)

- `xgen-node/src/fanout.rs`: `ClientSenders = Arc<Mutex<HashMap<IdentityXgid, mpsc::Sender<OutboundMsg>>>>` (single-sender-per-identity, Phase-1 comment). `apply_fanout(req, author_id, runtime, client_senders)` — collects `space.members.keys()`, `senders.get(rid) → try_send`, cap-1024 drop (`fanout_dropped_channel_full`). Sibling `FederationPeerSenders` untouched (EV-D5).
- `xgen-node/src/app.rs`: `handle_connection` client branch — `mpsc::channel(1024)` + `senders.insert(identity_id, out_tx)` (overwrite); disconnect `senders.remove(&identity_id)`. `apply_fanout` called from **4 sites**: client recv (`app.rs:1395`) + federation catch-up drain (`app.rs:1798`) + F-2 loop (`app.rs:1938`, both in `run_federation_session_post_handshake`) + A4 force-eject live fan-out (`admin_ops.rs:3688`, Option B / J-160).
- `apply_fanout` is the **superset chokepoint** — every accepted event (local + federation-received) passes through it. Federation push (`apply_federation_push`) is a sibling outbound path that adds nothing the observer needs.
- The M7 substrate `xgen-common::aicontrol` (envelope/codes/cmd/bindings/timeout) is the home for the new `Filter` type + predicate. Both binaries depend on it.
- `state.event_subscriptions` currently ships honest `0` both binaries (M7 v1, no events pipe).

---

## Sequence

| Step | Scope | Checkpoint |
|---|---|---|
| **Checkpoint** | Code-trace the retype touch set + `ConnId` mint points | 🔒 **Joe-lock before C1** |
| **C1** | `ConnId` + `ClientSenders` retype (alias + `apply_fanout` body) + register/remove rewrite + test-fixture sweep | ✅ checkpoint CLOSED |
| **C2** | `Filter` + `parse` + `matches` substrate (`xgen-common::aicontrol`) | ✅ SHIPPED J-208 (EV-D4 v1.1: 3-param `matches`) |
| **C3** | Node observer registry in `apply_fanout` (filter-before-send) + shared subscription registry + node `state` count | ✅ SHIPPED J-209 (Shape β — process-global, not threaded) |
| **C4** | Node `.events` pipe surface (subscribe/filter/drain/prune) + `nodes` filter | ✅ SHIPPED J-210 (`events_pipe.rs`; pipe = `{aicontrol}.events`) |
| **C5** | Client `.events` pipe (second WS + surface + at-drain filter) + client `state` count | — |
| **C6** | Close (D-074 atomic, doc-only) | — |

One checkpoint — the retype is the only load-bearing node-mechanism change; C2–C5 are adapter work over it (the design's adapter-after-retype thesis, EV-D2/EV-D3).

---

## Checkpoint — before C1 (Joe-lock) — ✅ CLOSED 2026-06-01 (J-206 arc)

Code-traced not guessed (sibling to the FAC checkpoint-#2 inbound-site trace). Locked findings:

1. **Touch set complete — 5 production value-shape sites.** `fanout.rs:54` (alias) + `fanout.rs:179` `senders.get(rid)` + `fanout.rs:208` `senders.get(joiner_id)` + `app.rs:1263` `senders.insert` + `app.rs:1431` `senders.remove`. These are the only production insert/remove sites on `client_senders`. (`federation_session.rs:337 senders.get` is the *peer* map — EV-D5, out of scope.)
2. **`apply_fanout` has 4 callers, not 3 (catch).** Client recv `app.rs:1395` + federation catch-up `app.rs:1798` + F-2 loop `app.rs:1938` + **A4 force-eject live fan-out `admin_ops.rs:3688`** (Option B / J-160). Irrelevant to C1 — the retype leaves `apply_fanout`'s signature unchanged (callers pass `&client_senders` as today). Load-bearing for **C3**: EV-D6's observer-registry param threads through **4** callers.
3. **`ConnId` home = `xgen-common`; single mint at `handle_connection`** (where `mpsc::channel(1024)` is created, ~`app.rs:1255`). The C5 events-pipe second-WS authenticates as the same identity and lands in the same client branch → one mint covers both the primary WS and the events WS; no separate registration path.
4. **Test-fixture sweep (C1):** `admin_ops.rs:5694` (direct `.insert`) + `fanout.rs:558` `install_sender` helper + the 3 fanout test setups (576 / 686 / 737) move to the Vec shape.
5. **Prime-invariant regression names:** `single_connection_fanout_unchanged` + `two_connections_same_identity_both_receive`.

---

## C1 — `ClientSenders` retype (gating change)

**Scope (xgen-node + `ConnId` home):**
- `ConnId(u64)` newtype + the global `AtomicU64` mint helper (EV-D1).
- `ClientSenders` value → `Vec<(ConnId, mpsc::Sender<OutboundMsg>)>` (EV-D2).
- Register → push `(conn_id, tx)`, create key if absent (no overwrite). Remove → drop matching `conn_id`; prune the identity key when its Vec empties.
- `apply_fanout`: the recipient `.get(rid)` and joiner `.get(joiner_id)` lookups → iterate the Vec, `try_send` to each; per-`(rid, conn_id)` `fanout_delivered` / `fanout_dropped_channel_full` traces preserved. **Signature unchanged** (still takes `&ClientSenders`), so the 4 `apply_fanout` callers are NOT touched in C1 — the caller update is C3's (EV-D6 observer param). `conn_id` is minted in `handle_connection` and threaded to register/remove only.

**Prime invariant (mandatory regression):** with one connection per identity, fan-out is byte-for-byte today. The whole existing fan-out + federation suite stays green; add an explicit `two_connections_same_identity_both_receive` test proving the new capability and a `single_connection_fanout_unchanged` lock.

**Verification (Rule 2):** `cargo test --workspace`; `cargo build --workspace --all-targets`; clippy `-D warnings`. No events pipe yet — this is a pure mechanism change.

---

## C2 — filter substrate (`xgen-common::aicontrol`)

**Scope (pure, no wiring):**
- `Filter { spaces: Vec<SpaceXgid>, event_types: Vec<String>, nodes: Vec<NodeXgid> }` (empty == all, per AC-D3b).
- `parse` from the `subscribe` payload — malformed (unknown field / wrong type / illegal wildcard form) → the `BAD_ARGUMENT` control error (substrate `codes.rs`).
- `matches(&Filter, &Event, event_nodes: &[NodeXgid]) -> bool` (EV-D4 **v1.1** shared predicate, D-067; the literal 2-param form was unimplementable for `nodes` — see design EV-D4 v1.1): AND-across / OR-within; two wildcard forms `*` and trailing `<family>.*` (strip only the `*`, keep the `.` → segment boundary; raw prefix on `EventType::as_str()`); `spaces` arm via the canonical `Event::effective_space_id()` (create-event resolution — empty `space_id` → `event_id`); `nodes` arm = `filter.nodes ∩ event_nodes ≠ ∅`, with `event_nodes` **caller-supplied**. C2 ships + unit-tests the predicate with synthetic `event_nodes`; the runtime derivation is C3's. **Confirm-at-pickup (D-078) — RESOLVED:** `EventType::as_str()` is uniform `family.suffix` (single dot), prefix predicate sound; exact entries fail-closed via `EventType::from_str`.
- `nodes`-on-client rejection is enforced at the *client* call site (C5), not in `matches` (the client passes `event_nodes = &[]`).

**Verification:** substrate unit tests (grammar table: empty==all, AND/OR, both wildcards, illegal wildcards → error, entitlement-narrows). `cargo test -p xgen-common`.

---

## C3 — node observer registry + subscription registry + node `state` — ✅ SHIPPED J-209

**As-built — Shape β (process-global), Joe-locked at C3 pickup.** The registry is a **process-global `OnceLock<NodeObservers>`** (`fanout::node_observers()`), **not** a param threaded through the fan-out callers. This is the J-166 protocol-audit precedent (a process-global registry consulted in hot async fan-out paths "NOT threaded through the ~N hot async signatures"). Consequence: `apply_fanout`'s **signature is unchanged**, the 4 call sites + their enclosing fns (`handle_connection` / `run_federation_session_post_handshake` / `admin_ops` force-eject) are **untouched**, and `NodeAdminDeps`/`AdminContext` are **untouched** — the global is read directly. Honors EV-D6's "single source of truth, process-wide, reachable by both servers" via the global rather than threading. Prime invariant is automatic: uninit/empty global ⇒ no observer sends ⇒ today byte-for-byte.

**Scope (xgen-node):**
- `NodeObservers = Arc<Mutex<Vec<(ConnId, Filter, mpsc::Sender<OutboundMsg>)>>>` behind `static NODE_OBSERVERS: OnceLock<…>` + `pub fn node_observers()` accessor (lazily-empty) (EV-D3 + EV-D6 single source of truth).
- `apply_fanout` reads the global **after** the member loop: derive `event_nodes` from runtime (`derive_event_nodes` — `SpaceState.home_node` + `federation_nodes` + `content["node_id"]` + sender-for-node-signed [`node_eject`/`node_unban`/`federation_add` only; `node_priority` excluded — its refs are `content["ordered_nodes"]`], EV-D4 v1.1 + the C3 source-4-narrow lock), then iterate observers and **filter-before-send** (`matches(filter, event, &event_nodes)`) → `try_send(Event)` to matching observers (EV-D4 A). Cap-1024 drop discipline + a per-observer `observer_delivered`/`observer_dropped_channel_full` trace. Senders lock dropped before the observer lock (no lock-order hazard).
- **Converged the space resolver (no lasting two-copy drift):** `xgen-node::fanout::event_space_id` now delegates to the canonical `Event::effective_space_id()` (C2, `xgen-common::wire`).
- Node command-pipe `state` reads `node_observers().lock().await.len()` → live `state.event_subscriptions` (EV-D6 process-wide count). Empty registry = `0` = today.

**Verification (shipped):** observer receives a matching fanned event, not a filtered-out one (`spaces`-scoped + serial-grouped for global isolation); `derive_event_nodes` four-source coverage; `state` count reflects a pushed observer (=1) and is `0` when empty; full suite **922**/0/1. `cargo test --workspace`; build all-targets 0/0; clippy `-D warnings` clean.

---

## C4 — node `.events` pipe surface — ✅ SHIPPED J-210

**As-built (xgen-node, NEW `events_pipe.rs`, sister to `aicontrol.rs`; `pipe.rs`/`--batch` untouched, D-066):**
- A second resident spawn at `run_node` (inside the existing `#[cfg(windows)]` pipe block, alongside the `--batch` + `.aicontrol` spawns; own `watch` receiver cloned before the batch spawn moves `rx`). **No deps** — the events server only touches the C3 process-global `fanout::node_observers()` (Shape β) + `NODE_LIFECYCLE`; nothing threaded.
- **Pipe name (confirm-at-pickup #5 RESOLVED):** `events_pipe_name(batch) = "{aicontrol_pipe(batch)}.events"` = `…\<base>.aicontrol.events` — namespaced under the aicontrol surface.
- Per-connection: accept → first message MUST be `subscribe` (`parse_subscribe` → a `subscribe` command whose `args` are the AC-D3b `Filter`; non-JSON/no-`cmd` → `MALFORMED_COMMAND`, wrong verb / bad filter → `BAD_ARGUMENT`, all replied **before** streaming, then close) → mint `ConnId` → push `(conn_id, filter, sender)` into the global registry → `subscribe` ack → drain the channel: forward `OutboundMsg::Event` as **bare Event JSONL** (filtering is `apply_fanout`'s job, C3 — the handler forwards whatever lands); ignore `HistoryBatch`/`SyncComplete` (**live-only, Q2**) → `unsubscribe` line **or** connection close prunes the entry.
- **`handle_events_connection<S>` is generic over the stream** (not `#[cfg(windows)]`), so subscribe → stream → prune is tested over `tokio::io::duplex` without a real pipe (the `process_inbound` generic-over-`S` pattern, J-086); only `start_events_server` is `#[cfg(windows)]` (named pipe, D-043). `unsubscribe` is best-effort (`read_line` not cancel-safe in the `select!`); connection close (EOF) is the reliable prune.
- `nodes` filter honored (Node dimension meaningful here — applied in `apply_fanout`'s observer loop via `matches` with the C3 runtime-derived `event_nodes`).

**Verification (shipped):** `parse_subscribe` valid/empty/wrong-verb/non-JSON/bad-filter; `events_pipe_name` suffix; duplex round-trip (subscribe → ack → forwarded Event JSONL → close prunes the registry); malformed-subscribe replies `BAD_ARGUMENT` and registers nothing. Handler tests serial-grouped on `node_observers`. Full suite **930**/0/1; build all-targets 0/0; clippy `-D warnings` clean.

---

## C5 — client `.events` pipe

**Scope (xgen-client, sister to `aicontrol.rs`):**
- The client resident opens a **second same-identity WS** to its home Node (reuse `client_authenticate`; EV-D3 client side — rides the C1 retype, so the primary AI loop's sender is not clobbered). **Confirm-at-pickup (D-078):** the spawn site (`service.rs`/`desktop.rs`/`ai_service.rs`) + reuse of the existing connect/auth path.
- `.events` pipe surface: accept → first-message `subscribe` (parse `Filter`) → tail the second-WS's inbound events → **filter-at-drain** (`matches`) → forward matches as JSONL → close.
- `nodes` present on the client → `BAD_ARGUMENT` (loud, EV-D4).
- Client `state.event_subscriptions` = count of active `.events` sessions (EV-D6).

**Verification:** subscribe-then-receive over the second WS; `nodes` → `BAD_ARGUMENT`; client `state` count. `cargo test -p xgen-client`.

---

## C6 — close (D-074 atomic, doc-only)

- `docs/xgen_aicontrol_implementation.md` §3 events → SHIPPED banners (client + node), as-built deltas vs the deferred spec (live-only no-history per Q2; process-wide count per EV-D6; node observer-grain per EV-D3; `FederationPeerSenders` out per EV-D5).
- `tasks/M7_EVENTS_AUDIT.md` + `tasks/M7_EVENTS_DESIGN.md` + this runbook → COMPLETED.
- ROADMAP M7-events row ✅ + version; CLAUDE PLAY → next milestone; JOURNAL close entry (same commit).
- No DECISIONS.md change expected (EV-D# arc-local, D-069) unless a lock is promoted at close (Joe's call).

---

## Hard rules

- **Adapter, not feature (D-065).** Events-pipe specialness (live-only, filtering) lives at the events-pipe consumer; `apply_fanout`'s member fan-out stays kind-agnostic; the node observer is the one node-internal addition (in arc scope). STOP on any urge to tap the accept/persist chokepoint (out of v1 — EV-D3 grain).
- **`--batch` untouched (D-066);** `pipe.rs` / `batch.rs` not edited. The `.events` pipe is a new sister surface.
- **Prime invariant (C1):** Vec-of-one = today byte-for-byte. Empty observer registry / no subscriptions ⇒ `state` reads `0` ⇒ today.
- **Split-triggers:** if any commit trips >600 lines or a natural family seam, surface it at the push-gate.

---

## Confirm-at-pickup (D-078)

1. `ConnId` home — `xgen-common` (verify no unwanted dep) vs per-binary.
2. The `matches` prefix predicate against the real `EventType::as_str()` strings.
3. Client second-WS spawn site + reuse of the connect/auth path.
4. The `nodes`-filter "involves a node" predicate source (event provenance fields).
5. ~~The `.events` pipe-name suffix convention (alongside `.aicontrol`).~~ **RESOLVED at C4:** `{aicontrol_pipe}.events` = `…\<base>.aicontrol.events` (namespaced under the aicontrol surface).

---

## Cross-refs

- `tasks/M7_EVENTS_DESIGN.md` (EV-D1–EV-D6) + `tasks/M7_EVENTS_AUDIT.md` (Phase-0 map).
- `tasks/M7_AICONTROL_DESIGN.md` §AC-D3b (filter grammar) + §AC-D3c (`state` schema).
- `docs/xgen_aicontrol_implementation.md` §3 (events spec).
- `xgen-node/src/fanout.rs`, `xgen-node/src/app.rs`, `xgen-common/src/aicontrol/`.
- JOURNAL J-203 (Q1/Q2/Q3) + J-205 (M7 v1 close).
- DECISIONS.md: D-065, D-066, D-067, D-069, D-074, D-078, D-082.

---

*Runbook ACTIVE. Checkpoint fires before Clair Commit 1. Clair stood down until the checkpoint closes.*
