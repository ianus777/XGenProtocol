# M-RP6.6 — the client resident: live connection state + traffic accounting
> **Status**: ACTIVE  
> Owes: M-RP6.6-INGEST live-ingest R5 fan-out · M-RP-SKIN ConnStats row-swap  
> Version: 1.3  
> Date: Jul 2026  
> **Last updated**: 2026-07-21  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

**Phase-0 LOCKED (2026-07-18).** Design-walked with Joe; all five decisions ruled by-recomm. This record hands Legs A/B + the Leg-C Rust/data half to Clair; the Leg-C visual row-swap is a handoff stub for Ms Design (§7). Re-ground §0 against `main` before any code (N-116). Nothing here is built yet. *(v1.1: decision labels normalized to the house arc-local form D1..D5; cargo floor corrected 1517 → 1519 per J-541's +2.)*

---

## §0 — One-line statement
Replace the shell's discard-the-stream auto-connect scaffold (`connect_async("ws://127.0.0.1:8080/xgen")` → throw socket away → sleep(150ms) → READY) with a **real long-lived resident**: a sustained `Connection` to the resolved home Node, honest lifecycle state driven by real socket events, reconnect-on-drop, and byte/RTT accounting that lights the ConnStats Speed/Bandwidth rows currently sitting at honest `N/A`. This closes the live half of gate F-1 (read shape closed M-RP6.1g/J-500; write half is M-RP6.3).

## §1 — Grounding verdict (six questions, verified on `main`)
1. **Reuse vs fork.** `service::run` already holds a thin resident (`connect_url` → `client_authenticate` → `loop { conn.recv() }` discard → `goodbye`) but with **no** lifecycle emission (headless), **no** accounting, **no** reconnect, **no** ingest. The desktop resident is a **superset**. → shared spine extracted (D1).
2. **Lifecycle states.** All **11** `ClientLifecycleState` variants are real Rust *and* all 11 are enumerated in `STATE_COLOURS` — matched. **No variant to add.** Work is *wiring* real events to the existing `emit_state`. Today only INITIALISING→CONNECTING→AUTHENTICATING→(fake)→READY/DISCONNECTED ever fire; RECONNECTING + the three DEGRADED_* exist but are never emitted.
3. **Accounting seam.** ConnStats Speed/Bandwidth are **hardcoded literal `'N/A'`** — not a store key waiting. The N/A→live seam is three joined pieces, none existing: Rust counters, a `selfState` store slot, the row swap. Transport hooks present: outbound choke `send_bytes`; inbound `recv()`; RTT via `ping()` + `Inbound::Pong(Vec<u8>)`. Command shape to mirror: `get_pacing_state` → `Pacing` managed-state.
4. **Node resolution.** `app::resolve_node(node_override, &config_path) -> String` exists (D-068 flag>config); `service.rs` already uses it. Desktop's hardcoded URL is the only holdout.
5. **Reconnect.** No client-side backoff scheduler exists anywhere. RECONNECTING must be **built**.
6. **Live ingest.** Clean seam — resident owns the socket but the drain discards. R5 live fan-out is gated on R5 + send (M-RP6.3). **Deferred to its own milestone** (D2 deferred leg).

## §2 — Locked decisions
- **D1 (spine reuse) — LOCKED: extract shared.** A shared `connect → auth → drain` helper both `service.rs` and the desktop resident call (D-056 shared command layer). Not a fork, not a rebuild.
- **D2 (leg split) — LOCKED: A / B / C, ingest deferred.** See §3.
- **D3 (accounting capture point) — LOCKED: stream-layer wrapper.** *(REVISED at J-543/J-545 — the original text read "resident-level wrapper" and is superseded; see §0 Path A of `M_RP6_3_COMPOSER.md` for why.)* The byte seam the design assumed — choking `send_bytes` / `recv()` — turned out to be **unusable**: `send_bytes` is private and `recv()` returns a parsed `Inbound`, so frame length dies inside GPL core. Counters therefore live in a client-crate **`CountingStream` interposed BELOW the WebSocket handshake**; **GPL core `Connection` remains untouched.** Honest label: **"all socket bytes this resident session" — and the auth handshake IS counted**, because the wrap happens *before* `client_authenticate`. *(The earlier "the auth handshake is outside the count" caveat was written against the abandoned seam and is now simply WRONG — deleted rather than softened.)* `send_event_confirmed` internal drains remain outside the count: under M-RP6.3 D2 they run on their own connection, never on the resident socket.
- **D4 (ConnStats rows when live) — LOCKED data contract: real-when-fed, absent-when-no-counter (N-060).** A row renders its real value once the resident feeds it; absent (not a fabricated placeholder) when no counter exists. The *look* of live-vs-idle is Ms Design's call (§7); this fixes only the data rule.
- **D5 (sequencing) — LOCKED: 6.6 before 6.3.** The resident owns the socket; M-RP6.3 (send) writes over it.

## §3 — Leg split
**Leg A — resident + lifecycle (Rust; Clair).**
Real long-lived `Connection` in the Tauri shell via the D1 shared spine. Swap raw `tokio_tungstenite::connect_async` → `xgen_core::transport::connect_url`; node via `resolve_node` (not hardcoded); real `client_authenticate` replacing the 150ms sleep; `goodbye` on quit. Drive `emit_state` from **actual** outcomes: CONNECTING on dial, AUTHENTICATING on challenge, READY on auth_ok, DISCONNECTED on `recv()` Err / drop. DEGRADED_* wired where the transport surfaces the corresponding failure (auth-degraded on re-auth failure; node/federation degraded left inert until a real source exists — no fabricated transitions, N-091).
*DoD:* kill the node live → real DISCONNECTED; relaunch with node up → CONNECTING→AUTHENTICATING→READY off real events; `cargo test` moves (spine unit tests).

**Leg B — reconnect / backoff (Rust; Clair).**
A client-side reconnect scheduler firing RECONNECTING on drop, backing off, and returning to AUTHENTICATING→READY on re-establish. Pure backoff logic unit-tested (no live socket needed for the schedule math).
*DoD:* node killed then restarted → RECONNECTING (pulsing) → READY without app restart; backoff schedule unit-tested.

**Leg C — accounting (Rust + data contract; Clair + me).**
Resident-level byte counters (in/out) + RTT via resident-driven `ping()`/`Pong` match. A `TrafficStats(Arc<Mutex<…>>)` managed state + a `get_conn_stats` Tauri command mirroring `get_pacing_state`. The `selfState` store gains a traffic slot (§4). **Row rendering swap is Ms Design's (§7).**
*DoD:* `get_conn_stats` returns live counts; store slot populated; counters unit-tested; I CDP-verify the rows go N/A→live after reload.

**Deferred leg — live ingest (own milestone).**
Real-time event ingest into R5 fan-out. Gated on R5 + M-RP6.3. Filed, not in 6.6.

## §4 — Leg-C data contract (mine to lock; Clair implements the Rust half)
Store slot on `selfState` (sibling of `connection` / `identity`, same one-source-two-views shape):

- `traffic: ConnTraffic | null` — `null` before the first `get_conn_stats` returns / no-Tauri (browser dev), exactly like `identity`.
- `ConnTraffic` fields (snake_case verbatim from Rust, no mapping layer — D3 drift rule):
  - `bytes_in: number` — cumulative observed inbound bytes this resident session.
  - `bytes_out: number` — cumulative observed outbound bytes.
  - `rtt_ms: number | null` — last ping/pong round-trip; `null` until the first pong.
- **Absent-not-zero (D4):** a metric with no live counter is `null` and the row renders **absent**, never a fabricated `0`. "Speed" (throughput) and "Bandwidth" derive from the byte deltas the resident feeds; how they read on screen is Ms Design's.
- Written by the shell from a `get_conn_stats` poll/push, mirroring `setConnection` / `setIdentity`. DEV CDP handle extends `__XGEN_SELF__` (or a sibling) so the verify pass reads the traffic facet.

## §5 — Lane map
- **Mine (Chat):** this Phase-0 doc; the Leg-C data contract (§4); CDP-verify of all UI-facing effects after a full reload (lifecycle flips, rows N/A→live); doc-bridge + memory hygiene. **Never pushes.**
- **Clair (Code):** Leg A + Leg B + the Leg-C Rust half (shared spine, reconnect scheduler, counters, `get_conn_stats`). Authors the build runbook from this lock.
- **Ms Design:** the ConnStats visual row-swap (§7) — appearance only.
- **Joe:** locked architecture (§2); pushes.

## §6 — Discipline
- Rust-primary → `cargo test` **must** move substantially off **1519/0/62** (the honest signal — N-116 forbids claiming otherwise). Leg B backoff + Leg C counters are testable pure logic.
- CDP: client **9222 only**; re-drive UI-facing effects live after a **full reload** (N-132, never an accumulated dev session). N-099: read the DOM in a separate eval after a settle delay, not the same eval that sets the bus.
- Every milestone ID carries its title (Joe-locked 2026-07-12). Every `.md` write refreshes the header — each `>` line ends in two spaces; `> **Last updated**:` is date-only.
- No backticks in any PowerShell for Joe — one physical line per `git` command, or a here-string.
- `tasks/M_RP_REGION_GEAR.md` (PENDING) stays untouched — deferred behind this milestone.

## §7 — Handoff stub — Ms Design (visual row-swap, Leg C)
When Clair lands Leg C, `connection-stats.svelte` swaps the two literal `'N/A'` rows (`speed`, `bandwidth`) for values read from `selfState.traffic`. **Data rule fixed (D4/§4):** real-when-fed, absent-when-`null` — reuse the existing `hasValue`/N-060 absent-row precedent already used for the identity rows; do **not** render a fabricated `0`. Appearance (units, formatting, live-vs-idle affordance) is Ms Design's to shape. The row array is already the extensible `{label, value}` shape — this is a value-source swap, not a rewrite.

## §8 — Sequencing note
6.6 (this) → 6.3 (composer/send, writes over the resident's socket) → deferred live-ingest leg (R5 fan-out). Gate F-1 live half closes at 6.6+6.3.
