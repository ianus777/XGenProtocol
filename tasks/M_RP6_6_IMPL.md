# M-RP6.6 — the client resident: live connection state + traffic accounting — BUILD RUNBOOK
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-18  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

**This is the build runbook for M-RP6.6 — the client resident: live connection state + traffic accounting.** It descends from the Joe-locked Phase-0 `tasks/M_RP6_6_RESIDENT.md` v1.1 (J-542). Spine-first, Leg A → B → C, each leg with its own DoD and its own commit. **NO CODE until Joe locks this runbook** (§0 carries one open decision that changes Leg C's shape). Re-grounded against `main` before authoring (N-116); §1 records what was verified, file:line.

---

## §0 — Read-first + the one open decision

**Rule-0 entry chain for the implementing seat:** `CLAUDE.md` PLAY head (M-RP6.6 🟢 tail) → JOURNAL J-542 → `tasks/M_RP6_6_RESIDENT.md` v1.1 (the locked Phase-0 — the agenda) → THIS runbook. Do not treat this runbook as ground-truth in isolation (Rule 0); the Phase-0 lock is the authority, this is its build decomposition.

### ✅ RULED — Leg C byte accounting = Path A (Chat, 2026-07-18, under the session's technical-autonomy grant; Joe owns visual only)

Joe assigned all technical decisions to Chat this session ("all technical stuff is yours; mine is visual"). The §0 open decision is therefore Chat's call, ruled **Path A** (the `CountingStream` interposer). **D3's note is revised** to match: *"resident-level counters at the **stream** layer — a client-crate `CountingStream` interposed below the WS handshake; GPL core `Connection` stays untouched; honest scope = all socket bytes this resident session, auth handshake **included** (more honest for a 'Bandwidth' row than excluding it)."* The table below is retained as the decision record.

Phase-0 §1(3) recorded the byte seam as *"outbound choke `send_bytes`; inbound `recv()`."* Re-grounded against `main`, that is not reachable under **D3 (GPL core `Connection` UNTOUCHED)**:

- `Connection::send_bytes` is **private** (`connection.rs:147`) — cannot be wrapped without editing GPL core.
- `Connection::recv()` returns a **parsed `Inbound` enum** (`connection.rs:430`), not raw bytes — the frame length is consumed inside core.
- No byte accounting exists in `transport/`.

So the resident cannot honestly observe **bytes** without a decision. Three honest options, my recommendation is **(A)**:

| # | Path | D3 core untouched? | Lights Speed/Bandwidth (§0 headline)? | Cost |
|---|---|---|---|---|
| **A (recommend)** | Client-crate `CountingStream<S>` interposed between TCP dial and WS handshake; spine's connect step becomes `connect_counted()` instead of GPL `connect_url`. | ✅ yes — counter lives in the client crate at the stream layer | ✅ yes — real byte deltas | Revises D3's *mechanism* (wrap the stream, not recv/send) + honest-scope note (counts the auth handshake too — arguably more honest for "Bandwidth"). ws:// only today; wss:// is a named future concern. |
| B | Leg C ships `rtt_ms` live only; `bytes_in`/`bytes_out` stay honestly **absent** (D4 already permits "no counter → absent, never fabricated 0"). Byte counting becomes its own follow-on milestone. | ✅ yes | ❌ no — Speed/Bandwidth stay N/A after 6.6 | Guts the milestone's stated headline (§0). Cleanest scope, but 6.6 no longer does what §0 says. |
| C | Count **messages** in/out (resident counts `recv()` returns + its own sends), relabel the rows. | ✅ yes | ⚠️ partial — a different quantity than bytes | Dishonest if rendered under "bytes"/"Bandwidth". Advise against. |

**Ruled: A.** It is the only path that keeps GPL core untouched *and* delivers the bytes-derived Speed/Bandwidth §0 promises. D3's note is revised as above (stream-layer counter; core untouched; scope includes the handshake). Leg C (§5) is built on **A**. Everything below Leg C (the shared spine, Leg A, Leg B) is unaffected by this decision and is fully grounded.

---

## §1 — Grounding re-confirmation (verified on `main`, file:line)

1. **`service::run_ws_loop` is the thin resident** (`xgen-client/src/service.rs:83`): `app::resolve_node(None, &config_path)` (`:85`) → `connect_url(&node)` with a 10 s timeout (`:93`) → `client_authenticate(&signing_key)` (`:107`) → `loop { conn.recv() }` **discard** (`:117`) → `goodbye("service_shutdown")` (`:130`). **No** `emit_state`, **no** counters, **no** reconnect, **no** ingest. The desktop resident is a superset — same spine, plus lifecycle emission + counters + reconnect. (D1 basis.)
2. **The desktop scaffold to replace** (`xgen-client/src/desktop.rs:472-490`, inside `run_startup`): emits `Connecting` → `tokio_tungstenite::connect_async("ws://127.0.0.1:8080/xgen")` with a 2 s timeout → on Ok, emits `Authenticating`, **`sleep(150ms)`**, `Ready`; on any error, `Disconnected`. The stream is **thrown away** — no auth, no resident loop. Hardcoded URL is the lone `resolve_node` holdout (§1.4).
3. **`emit_state`** (`desktop.rs:57`) is the lifecycle emit path; `run_startup` (`:417`) is spawned once from `.setup()` (`:587`) with the `AppHandle` and `shutdown_rx: watch::Receiver<bool>` (`:421`). All **11** `ClientLifecycleState` variants are real Rust and all 11 enumerated in `STATE_COLOURS` (`self-state.svelte.ts:20-27`) — matched. `RECONNECTING` + the three `DEGRADED_*` exist but are **never emitted** today. Work is *wiring*, not adding variants.
4. **`app::resolve_node(node_override, &config_path) -> String`** exists (D-068 flag>config); `service.rs:85` already uses it. `desktop.rs`'s hardcoded URL is the only holdout.
5. **No reconnect/backoff scheduler exists** anywhere in `xgen-client/src`. Leg B builds it.
6. **Transport surface** (`xgen-core/src/transport/connection.rs`): `recv() -> Result<Inbound, …>` (`:430`) parses frames to `Inbound` (incl. `Inbound::Pong(Vec<u8>)` `:87` / `:481`); `ping()` (`:586`) sends an empty WS ping; `goodbye(reason)` (`:595`); `client_authenticate(&signing_key) -> AuthOutcome` (`:546`); `send_bytes` is **private** (`:147`). `connect_url`/`connect` (`client.rs:19/32`) return `Connection<MaybeTlsStream<TcpStream>>`, stream already wrapped; `Connection::new(ws)` is **public** (`:141`) — the hook the `CountingStream` interposer (Leg C path A) uses.
7. **Command/state mirror shape** (`desktop.rs`): `struct Pacing(Arc<Mutex<PacingManager>>)` (`:38`), managed via `.manage(pacing_manager)` (`:579`), read by `#[tauri::command] fn get_pacing_state(...) -> Vec<PacingState>` (`:171`), registered in `invoke_handler![… get_pacing_state, …]` (`:611`). Leg C's `TrafficStats` + `get_conn_stats` mirror this exactly.
8. **Store slot** (`ui/common/lib/stores/self-state.svelte.ts`): `selfState` exposes `connection` / `identity` getters + `setConnection` / `setIdentity` (`:52-69`); DEV CDP handle `__XGEN_SELF__` (`:74`). Leg C adds a sibling `traffic` facet (§5.4).
9. **Row consumer** (`ui/common/lib/components/widgets/connection-stats.svelte:52-53`): two literal `{ key:'speed', value:'N/A' }` / `{ key:'bandwidth', value:'N/A' }` rows, explicitly annotated as M-RP6.6 reminders. Ms Design swaps these (§7); Clair does not touch this file.

**Cargo floor: 1519/0/62** (J-541's +2 over the stale 1517). Rust-primary → the count **must** move substantially (Leg B backoff + Leg C counters are pure, testable logic). A leg whose DoD leaves the count flat is not honest.

---

## §2 — The shared spine (D1) — extract, then both call it

**New module `xgen-client/src/resident.rs`** (client crate — NOT GPL core; D3 holds). It houses the shared `connect → auth → drain` spine that both `service.rs` (headless) and `desktop.rs` (Tauri) call. Not a fork, not a rebuild (D-056 shared command layer).

**Shape (illustrative — Clair owns the Rust ergonomics):**

- A `run_session(...)` async fn that performs one connect → authenticate → drain cycle and returns a typed `SessionEnd` outcome (`Disconnected` on `recv()` Err / `Closed`; `ShutdownRequested` when `shutdown_rx` fires; `AuthFailed`; `ConnectFailed`). It takes:
  - the resolved node URL (caller passes `resolve_node(None, &config_path)` — §1.4),
  - the signing key,
  - a `shutdown_rx` clone (so drain selects on shutdown → `goodbye` → clean exit),
  - a **lifecycle sink**: a small `Fn(ClientLifecycleState)` (or a trait) the caller supplies. `service.rs` passes a no-op/`tracing` sink (stays headless); `desktop.rs` passes a closure over `emit_state(&app, …)`. This is how the SAME spine drives real lifecycle in the shell and stays silent in the service.
  - (Leg C only) an optional traffic-counter handle (the `CountingStream` `Arc`s + the RTT tracker), `None` for `service.rs` if desired.
- The drain loop `select!`s on `conn.recv()` vs `shutdown_rx.changed()`; on shutdown it emits nothing new, calls `goodbye`, returns `ShutdownRequested`; on `recv()` Err/`Closed` it returns `Disconnected`; `Inbound::Pong` feeds the RTT tracker (Leg C); every other `Inbound` is discarded (ingest deferred).

**Refactor `service::run_ws_loop`** → thin caller of `run_session` with a `tracing`-only sink and no counters. Behaviour unchanged (headless drain-and-discard); this is the regression-lock that proves the extraction is a pure move.

**DoD (spine — folds into Leg A's commit; no standalone commit):** `service.rs` builds and its existing behaviour is unchanged; `run_session` has unit coverage for its outcome mapping where it's pure (outcome-from-inputs); `cargo build -p xgen-client` clean.

---

## §3 — Leg A — resident + lifecycle (Rust)

**Goal:** replace the `desktop.rs:472-490` scaffold with a real long-lived resident driven by the §2 spine, emitting honest lifecycle state from actual socket outcomes.

**Steps:**
1. Land the §2 spine (`resident.rs`) and refactor `service.rs` onto it.
2. In `run_startup`, replace the connect scaffold (lines ~472-490) with a spawned resident task that calls `run_session`, passing an `emit_state`-closure lifecycle sink. Real transitions:
   - `Connecting` on dial start (before `connect_counted`/`connect_url`),
   - `Authenticating` when the socket is up and `client_authenticate` begins,
   - `Ready` on `AuthOutcome` success,
   - `Disconnected` on `recv()` Err / `Closed` / connect failure / auth failure.
   - **`DEGRADED_AUTH`** only if a real re-auth-failure source exists at this leg; **`DEGRADED_NODE` / `DEGRADED_FEDERATION` left inert** — no source exists, and a fabricated transition is an unfed branch (N-091). State in the leg's commit body *why* they stay inert.
3. Node URL via `app::resolve_node(None, &config_path)` — kill the hardcoded string (§1.4).
4. Real `client_authenticate(&signing_key)` replaces the `sleep(150ms)`.
5. `goodbye` on quit — the drain's `shutdown_rx` arm (§2) already does this; confirm the Tauri quit path drives `shutdown_rx`.

**Files:** `xgen-client/src/resident.rs` (new), `xgen-client/src/service.rs` (refactor to caller), `xgen-client/src/desktop.rs` (scaffold → resident spawn). No GPL-core edits. No frontend edits.

**DoD (Leg A — own commit):**
- [ ] Real-node session (both binaries up — client 9222, node 9322): relaunch client with node up → the led/state walks `CONNECTING → AUTHENTICATING → READY` off **real** events (no 150 ms sleep in the path).
- [ ] Kill the node live → state flips to real `DISCONNECTED` (from `recv()` Err, not a timeout guess).
- [ ] Quit the client → `goodbye` observed (node-side log or clean close).
- [ ] `DEGRADED_*` that have no real source are **not** emitted; commit body states why.
- [ ] `cargo test -p xgen-client` builds and passes; spine outcome-mapping unit tests present. Count moves off 1519 or the commit body explains why Leg A alone adds none (lifecycle is I/O-driven, hard to unit-test purely — acceptable for A; B and C must move it).
- [ ] `cargo test` (workspace) green; quote the real `test result:` line (Rule 2/5).

---

## §4 — Leg B — reconnect / backoff (Rust)

**Goal:** on a dropped connection the resident emits `RECONNECTING` (pulsing — it's in `PULSING_STATES`), backs off, and returns to `AUTHENTICATING → READY` on re-establish, **without an app restart**.

**Steps:**
1. **Pure backoff module** (client crate, e.g. `resident::backoff` or a small struct): a deterministic schedule (e.g. capped exponential with a max) exposing `next_delay(attempt) -> Duration` and a reset. **No live socket** — pure `attempt → Duration` math. Unit-test the schedule directly (this is where the count moves).
2. **Reconnect wrapper** around the §2 spine in the desktop resident: an outer loop — on `SessionEnd::Disconnected`, emit `RECONNECTING`, sleep `backoff.next_delay(attempt)`, re-enter `run_session`; on success reset the backoff; on `ShutdownRequested` break the outer loop. `service.rs` MAY opt into the same wrapper or stay single-shot (Clair's call — record it; if service stays single-shot, say so, don't leave it implied).
3. `RECONNECTING` transitions flow through the same `emit_state` sink as Leg A.

**Files:** `xgen-client/src/resident.rs` (backoff + wrapper). No GPL-core, no frontend.

**DoD (Leg B — own commit):**
- [ ] Real-node session: node killed then restarted → state shows `RECONNECTING` (pulsing) then `AUTHENTICATING → READY` with the client process never restarted.
- [ ] Backoff schedule unit-tested as pure logic (bounds, monotonic-until-cap, reset-on-success). Quote the `test result:` line.
- [ ] `cargo test` count **moves** off the Leg-A total (backoff tests are the honest signal).
- [ ] `cargo test` (workspace) green.

---

## §5 — Leg C — accounting (Rust + data contract)

> ✅ **§0 ruled — Path A (`CountingStream` interposer).** Built on A; D3's note revised (stream-layer counter, core untouched, handshake included).

**Goal:** the resident feeds real traffic metrics; `get_conn_stats` exposes them; the `selfState.traffic` slot carries them; the ConnStats Speed/Bandwidth rows go N/A → live (row swap is Ms Design's, §7).

**Steps (Path A):**
1. **`CountingStream<S>`** (client crate, e.g. `resident::counting`): an `AsyncRead + AsyncWrite + Unpin` wrapper over an inner stream that tallies read/written byte counts into an `Arc<AtomicU64>` pair (`in`, `out`). Unit-test the tally (write N, read M → counters read N/M) against an in-memory duplex — pure, no network. **This is where the count moves for Leg C.**
2. **`connect_counted(url) -> (Connection<…>, TrafficCounters)`** (client crate): mirrors `connect_url`'s three lines but dials `TcpStream::connect`, wraps in `CountingStream`, runs `tokio_tungstenite::client_async(url, counting)`, and `Connection::new(ws)`. ws:// only (Phase-1; wss:// is a named future concern — state it in a code comment). The §2 spine's connect step uses this in the desktop path; GPL `connect_url` and core `Connection` stay untouched (D3-honest).
3. **RTT tracker** (client crate): the resident periodically calls `conn.ping()` and records a send timestamp; when the drain loop sees `Inbound::Pong`, it computes `rtt_ms` and stores it in the shared handle. (Pong-without-a-pending-ping is ignored.) Pure timing arithmetic; unit-test the compute given a send/recv instant pair.
4. **`TrafficStats(Arc<Mutex<…>>)` managed state** in `desktop.rs`, mirroring `Pacing` (`:38` / `:579`): holds `bytes_in` / `bytes_out` (read from the `CountingStream` `Arc`s) + `rtt_ms: Option<u64>`. Populated by the resident task.
5. **`#[tauri::command] fn get_conn_stats(traffic: tauri::State<TrafficStats>) -> ConnTrafficDto`** mirroring `get_pacing_state` (`:171`); register in `invoke_handler![…]` (`:611`) and `.manage(…)` (`:579`).
6. **Store slot** (`self-state.svelte.ts`) — the Leg-C data contract, my half, §5.4 below. Clair implements the Rust `ConnTrafficDto` to serialise **exactly** those snake_case field names (no mapping layer — D3 drift rule).
7. **Shell wiring**: `app_client.svelte` polls/pushes `get_conn_stats` into `selfState.setTraffic(...)`, mirroring the `setConnection`/`setIdentity` pattern. (A short interval poll is fine; a push channel is a later refinement — Clair records which.)

**§5.4 — Leg-C data contract (Chat's to lock; Clair implements the Rust half).**
Add to `selfState` a sibling facet of `connection`/`identity`:

- `traffic: ConnTraffic | null` — `null` before the first `get_conn_stats` returns / in browser dev (no Tauri), exactly like `identity`.
- `ConnTraffic` (snake_case **verbatim** from Rust — no rename layer):
  - `bytes_in: number` — cumulative observed inbound bytes this resident session.
  - `bytes_out: number` — cumulative observed outbound bytes this resident session.
  - `rtt_ms: number | null` — last ping/pong round-trip; `null` until the first pong.
- **Absent-not-zero (D4):** a metric with no live counter is `null` and its row renders **absent**, never a fabricated `0`.
- `setTraffic(payload: ConnTraffic)` writer, mirroring `setConnection`/`setIdentity`. DEV handle: `traffic` is reachable via the existing `__XGEN_SELF__` (it's a getter on `selfState`), so the verify pass reads it with no new handle.

**Files:** `xgen-client/src/resident.rs` (+`counting`, RTT), `xgen-client/src/desktop.rs` (`TrafficStats`, `get_conn_stats`, manage+register), `ui/common/lib/stores/self-state.svelte.ts` (`traffic` slot + `setTraffic`), `ui/client/src/app_client.svelte` (poll → `setTraffic`). **NOT** `connection-stats.svelte` (Ms Design, §7). No GPL-core edits.

**DoD (Leg C — own commit):**
- [ ] `get_conn_stats` returns live counts after a real-node session (invoke over CDP, node 9322 up).
- [ ] `selfState.traffic` populated; `__XGEN_SELF__.traffic` reads real `bytes_in`/`bytes_out` + an eventual `rtt_ms`.
- [ ] `CountingStream` tally + RTT compute unit-tested (pure). `cargo test` count **moves**. Quote the `test result:` line.
- [ ] Absent-not-zero honored: a metric with no counter is `null`, not `0` (assert the DTO/serialisation).
- [ ] `cargo test` (workspace) green.
- [ ] Chat CDP-verifies (own lane, §6) the rows go N/A → live after a full reload — **after** Ms Design lands the row swap; if the swap is not yet in, Chat verifies the store facet is live and notes the row swap as Ms Design's open item.

---

## §6 — Verification protocol

- **Ports:** client **9222**, node **9322**. Leg A/B live DoD (*kill the node → the led flips*, *restart → reconnect*) **requires both binaries running** — a real-node session, not a dev-shell-only session (the scaffold's 2 s auto-connect against a dead port only ever proved `DISCONNECTED`).
- **CDP:** client **9222 only**; re-drive every UI-facing effect live after a **full reload** (N-132 — never an accumulated dev session; a stale HMR session inflates the registry). Read the DOM in a **separate eval after a settle**, not the same eval that sets state (N-099).
- **Registry baseline:** read **quiescent** (N-105) and state the store/selection/saved-state context (N-108/N-112/N-115). M-RP6.6 adds **no** new registered components (it's Rust + a store facet + Ms Design's row-value swap) — so the client registry baseline should be **unchanged**; that invariance is itself a verify leg. Confirm the number live; do not quote a stale one.
- **Gates, quoted from real output (Rule 2/5):** `cargo test` (workspace) — **must move off 1519/0/62**; `npm test` (in `ui/sampler`); `vite build` (in `ui/client`). ⚠️ `cargo test` exceeds the MCP timeout — run it **detached** and poll the PID in separate short calls; a killed detached run leaves a truncated, plausible-but-wrong artifact (N-117), so confirm the final `test result:` line is present. ⚠️ A running `tauri dev` client **holds the client exe** → `cargo test` fails to relink; bring the app down before the static gate.
- **Lane ownership of verification:** Chat re-drives every UI-facing leg (lifecycle flips on real connect/disconnect; traffic store N/A → live) on 9222 after a full reload, and owns the doc-bridge. Clair's per-leg DoD is the Rust/build half; the two are not the same pass.

---

## §7 — Handoff stub — Ms Design (visual row-swap, Leg C)

When Clair lands Leg C, `connection-stats.svelte:52-53` swaps the two literal `'N/A'` rows (`speed`, `bandwidth`) for values read from `selfState.traffic`. **Data rule is fixed (D4/§5.4):** real-when-fed, absent-when-`null` — reuse the existing absent-row precedent (N-060) already used for the identity rows (`connection-stats.svelte:41-45`); do **not** render a fabricated `0`. Appearance (units, formatting, live-vs-idle affordance, whether "Speed" is a rate derived from byte deltas over an interval or a raw cumulative) is Ms Design's to shape. The row array is already the extensible `{ key, label, value }` shape — a value-source swap, not a rewrite. RTT may become a third row (`rtt_ms → "Latency"`), Ms Design's call.

---

## §8 — Lanes + discipline

**Session reassignment (Joe, 2026-07-18): "all technical stuff is yours; mine is visual."** For this arc:
- **Chat (me) — all technical:** the §2 spine + Leg A + Leg B + the full Leg-C Rust half (`CountingStream`, RTT, `TrafficStats`, `get_conn_stats`, `app_client` poll) + the §5.4 data contract + the §0 ruling + CDP-verify + the canonical records (CLAUDE.md / JOURNAL / ROADMAP / this task doc). One commit per leg (spine folds into Leg A). **Never pushes.**
- **Joe — visual:** the ConnStats row-swap (§7 — the Ms Design work) + **push**.

*(This supersedes the Phase-0 §5 split for this session only; the durable Phase-0 lane map is unchanged as a record.)*

**Discipline:** every milestone ID carries its title (Rule 8). Every `.md` write refreshes the header — each `>` line ends in two trailing spaces; `> **Last updated**:` is a date only. No backticks in any PowerShell handed to Joe — one physical line per `git` command, or a here-string. DoD checklists never include "commit pushed" (Joe pushes). `tasks/M_RP_REGION_GEAR.md` (PENDING) stays untouched — deferred behind this milestone. `cargo test` **must move** off 1519 across the arc (a runbook whose DoD leaves the count flat is not honest — Leg B backoff + Leg C `CountingStream`/RTT are testable pure logic).

---

## §9 — Master DoD (the arc closes when all are true)

- [x] §0 decision ruled (Chat, Path A — `CountingStream` interposer); Leg C shaped accordingly.
- [x] Spine (`resident.rs`) extracted; `service.rs` refactored onto it, behaviour unchanged (regression-lock: compiles clean, headless drain unchanged).
- [x] Leg A: real lifecycle on a real-node session (`CONNECTING→AUTHENTICATING→READY` off real auth, live `DISCONNECTED` on kill). `goodbye` on quit wired (best-effort; races `app.exit` — not separately observed). `DEGRADED_*` left inert (no real source, N-091). **The emit bug (N-137) was found and fixed here.**
- [x] Leg B: `RECONNECTING → READY` without app restart (same process, proven); backoff unit-tested + confirmed against wall-clock.
- [x] Leg C: `get_conn_stats` live; `selfState.traffic` populated (`__XGEN_SELF__.traffic` = live bytes + real RTT, grows across a ping interval); absent-not-zero honored (unit test: `None` before first pong); `CountingStream` + backoff counters unit-tested.
- [x] `cargo test` (workspace) **1524/0/62 = floor 1519 + 5 new resident tests, exactly**; `0 failed`; final `test result:` line present (no N-117 truncation). `npm test` 77; `vite build` clean.
- [~] Client registry: **not re-measured live** (dev sessions were brought down for the test gate). Unchanged **by construction** — no new envelope-registered component; `connection-stats.svelte` untouched.
- [x] Chat CDP-verified on 9222 (real-node session): lifecycle A/B/C flips on real connect/disconnect/reconnect; `selfState.traffic` live. (The visible ConnStats rows stay `'N/A'` — that swap is Ms Design's, §7, not this milestone.)
- [ ] Ms Design row-swap — **filed as her open leg** (§7). The arc closes on the live store facet; do not hold the close for it (Joe).
- [x] Records bridged (Chat): CLAUDE.md PLAY head, JOURNAL J-543, ROADMAP (v5.10), N-137, this task doc → COMPLETED. **One atomic commit (D-074); Joe pushes.**
- [x] D3 (GPL core `Connection` untouched) + D2 (live ingest out) confirmed against the actual diff — 7 files, zero `xgen-core`.
