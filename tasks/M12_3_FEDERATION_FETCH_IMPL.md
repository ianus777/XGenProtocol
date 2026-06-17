# M12.3 — Federation fetch-blob-by-hash: Implementation runbook
> **Status**: ACTIVE
> Version: 1.0
> Date: Jun 2026
> **Last updated**: 2026-06-17
> Language: English
> Author: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> License: BSL 1.1 (converts to GPL upon project handover)

---

## §1 Purpose & status

The Clair-authored M12.3 runbook, executing the Joe-LOCKED design
`tasks/M12_3_FEDERATION_FETCH_DESIGN.md` (v1.0, M12.3-D1..D6) on the Phase-0 audit
`tasks/M12_3_FEDERATION_FETCH_PHASE0_AUDIT.md` (v1.0, GO, M12.3-A-01..06). M12.3 is the
**federation** sub-arc of M12: a peer home **B** that holds a federated `message.file` descriptor
but not its ciphertext can **fetch the blob across homes** from a home that does.

D-071 arc discipline: this runbook → **Joe locks the runbook values (§3 + §4)** → implement
spine-first (§5, per-commit, Joe pushes each) → Chat doc-bridge → M12.3 close. **P1/P2/P3 + V1–V5
are ✅ LOCKED (Joe, 2026-06-17, by-recomms; Chat cross-verified the groundings on `main @ e24bef9`,
D-065 — all held).** Decisions are arc-local (D-069). **M12-D6 (universal-E2E protocol-layer) stays
a flagged DECISIONS.md promotion candidate — not this arc.**

**Grounded against `main @ b1a6bb1`** (tree clean, in-suite 1445/0). Every seam below was
re-confirmed to file:line by reading production code this session (D-078 — anchors re-confirmed,
not trusted blind; the design/audit line numbers drifted across M12.2a/M12.2b and are corrected
here).

**The locks (§3 design, do NOT re-litigate):** D1 β multiplex the established federation session
reusing the M12.1 `TransportMessage` blob variants · D2 F3 lazy-default (Retained-eager = named
unbuilt hook) · D3 reserve-the-hook, no `Retention` read · D4 synchronous miss-signal · D5 type
`BlobError::Unavailable` → `10003`, replace the literal · D6 single arc, spine-first commits,
box-gated real-binary two-home + in-suite in-process witnesses.

---

## §2 Grounding ledger (seams re-confirmed to file:line on `main @ b1a6bb1`)

| # | Seam | Location | M12.3 action |
|---|---|---|---|
| G1 | **The M12.1 `TransportMessage` blob variants** — `BlobUploadBegin{blob_ref,size}` · `BlobChunk{seq,data}` · `BlobUploadEnd{blob_ref}` · `BlobUploadOk{blob_ref}` · `BlobFetchRequest{blob_ref}` · `BlobFetchEnd{blob_ref}` | `xgen-core/src/wire/types.rs:188-238` | Reuse on the federation conn (D1 γ-reuse). Add `space_id: Option<String>` to `BlobFetchRequest` (§3.1; additive `#[serde(default, skip_serializing_if)]`). **`BlobChunk` carries NO `blob_ref`** — the correlation crux (§3.2). |
| G2 | **`BLOB_CHUNK_BYTES = 128 * 1024`** (raw ciphertext bytes per chunk; base64 ≈ 174 KB < 256 KB frame ceiling) | `xgen-core/src/wire/types.rs:244` | Reuse verbatim on the federation serve leg (confirm V1). |
| G3 | **`BlobError` + `to_wire_code`** — `HashMismatch`→10001, `TooLarge`→10002 (both live); `MalformedRef`/`Io`→None; **no `Unavailable` arm**; doc reserves `10003` | `xgen-core/src/blob_store.rs:43-82` | **D5:** add `Unavailable` arm → `Some((10003, "blob_unavailable"))` (C1). |
| G4 | **The defensive `10003` literal** — client↔node `BlobFetchRequest` handler, `Ok(None)` arm sends `blob_err(10003, "blob_unavailable")`; comment: *"typed `BlobError::Unavailable` + lazy fetch-by-hash land at M12.3."* | `xgen-node/src/app.rs:1917-1925` (literal `:1919`); the `blob_err` closure `:1644`; the whole client↔node serve handler `:1880-1934` | **C1:** replace the literal with the typed path. **C2:** rewrite this `Ok(None)` arm to attempt the federation fetch first, serve `10003` only on no-reachable-peer / all-miss / timeout. |
| G5 | **The federation steady-state loop** — `tokio::select!{ biased; r = conn.recv() => match {…}; Some(out_msg) = out_rx.recv() => match {Event\|HistoryBatch\|SyncComplete} }`; inbound arms: `Goodbye`/`Closed`/`Err`→break, `Ping`/`Pong`→noop, `Event`→`process_inbound`+`apply_fanout`+`apply_federation_push`, `IdentityReplicate`→handler, **catch-all `Ok(_) => { silently ignore }`** | `xgen-node/src/app.rs:2512-2598` (select! `:2513`, catch-all `:2561`, outbound `:2567`) | **C2 (the arm-add, D1):** new inbound arms ahead of `Ok(_)` — serve `BlobFetchRequest` from local store; collect `BlobChunk`/`BlobFetchEnd`/blob-`Error` into the pending slot. New `out_rx` arm for `OutboundMsg::BlobFetchRequest`. |
| G6 | **`run_federation_session_post_handshake<S>`** signature + cleanup tail (`federation_peer_senders.remove(peer)` + `mark_lost` on session-end, all 5 exit paths converge `:2601-2630`) | `xgen-node/src/app.rs:2310-2333` (sig); `:2601` (cleanup) | Thread the new `PendingFederationFetches` Arc (§3.2) through the sig; clear `pending[peer]` (fail any waiter) in the cleanup tail. |
| G7 | **`OutboundMsg`** = `#[derive(Debug, Clone)]` enum `{Event, HistoryBatch, SyncComplete}` | `xgen-node/src/fanout.rs:41-56` | **Clone is load-bearing** → the fetch-request injector variant must stay Clone → **no oneshot in `OutboundMsg`** → the oneshot/accumulator lives in the shared `PendingFederationFetches` map (§3.2). Add `OutboundMsg::BlobFetchRequest { blob_ref: String, space_id: Option<String> }` (Clone-able). |
| G8 | **`FederationPeerSenders = Arc<Mutex<HashMap<NodeXgid, mpsc::Sender<OutboundMsg>>>>`** — how any task pushes into a peer's federation session | `xgen-node/src/fanout.rs:98` | The injection channel: B's miss handler looks up the holder peer's `out_tx` here and pushes `OutboundMsg::BlobFetchRequest`. The new `PendingFederationFetches` is its sibling Arc (§3.2). |
| G9 | **`SpaceState.home_node: NodeXgid` + `federation_nodes: Vec<NodeXgid>`** | `xgen-core/src/space/state.rs:192,234` | The "which peer holds it" surface (§3.3): B resolves the Space's federated holders from these. |
| G10 | **`handle_connection`** already threads `federation_peer_senders` + `blobs_dir` + `runtime` + `max_blob_bytes` | `xgen-node/src/app.rs:1477-1492` | The client↔node miss handler (G4) **already has** `federation_peer_senders` + `runtime` + `blobs_dir` in scope — only the new `PendingFederationFetches` Arc needs threading in. |
| G11 | **Client `Connection::fetch_blob(blob_ref, timeout)`** — sends `BlobFetchRequest`, drains `BlobChunk`* until `BlobFetchEnd`, **maps blob-`Error` → `TransportError::UnexpectedMessage`** (a fetch failure); already carries `timeout` | `xgen-core/src/transport/connection.rs:317-364` | **D4 (no new client wire):** add a `space_id: Option<&str>` param so the node can scope the federated resolution (§3.3); widen the outer timeout (§4 V3). The `Error`→fetch-failure mapping already exists → wire `10003` surfaces as a fetch failure, no client app-layer wire change. |
| G12 | **`ops::fetch_attachments`** — the M12.2a fetch verb; has `args.space` + `args.room`; iterates descriptors, calls `conn.fetch_blob(&d.blob_ref, sync_timeout)` (`:2004`); `sync_timeout = sync_completion_timeout(ctx.data_dir)` | `xgen-client/src/ops.rs:1898-2006` | Pass `Some(&args.space)` into `fetch_blob` (the space context for §3.3). The self/single-home case (empty `federation_nodes`) → local-only → W5 unchanged. |
| G13 | **In-process federation harness** — two `NodeRuntime`s over `tokio::io::duplex`, spawned `run_federation_session_post_handshake` per side; existing federation in-process tests | `xgen-node/src/tests/phase9_harness.rs`; `federation_push_integration.rs`; `late_federation_identity_catchup.rs`; `m12_blob_roundtrip.rs` (M12.1 in-process blob) | The **spine in-process witness** target (C2). |
| G14 | **Box-gated real-binary federation harness** — F2/F9 harness-control `add-peer`/`initiate` seam drives federation; the M12.2a sibling | `xgen-mptest/tests/m12_2a_self_thread_e2e.rs`; `xgen-mptest/src` (harness) | The **headline W1 box-gated e2e** target (C3). |
| G15 | **RC-F-01 re-grep (this session):** `10003`/`blob_unavailable`/`BlobError::Unavailable`/domain-10 sweep | grep | `10003` occurs ONLY in `blob_store.rs` docs (reserved) + `app.rs:1916/1919` (the literal). Domain 10 = 10001 (live), 10002 (live), 10003 (free). **Clear to type as `Unavailable`.** Re-grep again at build. |

---

## §3 Runbook-bound picks — ✅ LOCKED 2026-06-17 (Joe, by-recomms)

The design left three grounded details to the runbook (D1 names the "which peer holds it" item
explicitly as runbook-bound). Each was a **pick + flag**; all three are now **LOCKED by-recomms**
(Joe, 2026-06-17; Chat cross-verified the groundings on `main @ e24bef9`).

### P1 — "Which peer holds it" (the D1 runbook-bound open item) — **recommended pick, flagged**

**Recommended:** on a client↔node `BlobFetchRequest` MISS (`BlobStore::get → Ok(None)`, G4), B
resolves the candidate holder(s) from **the Space's federation set** — read `SpaceState` for the
fetch's `space_id` (P-wire, below): the holders to try = `home_node` ∪ `federation_nodes` (G9),
**intersected with B's live federation sessions** (the keys of `federation_peer_senders`, G8 —
only peers with an ACTIVE session are reachable). Try the Space's `home_node` **first** (the
likeliest single holder / the common single-home case), then the rest, **serialized** (P2). First
byte-identical hit → `BlobStore::put` locally (so future reads hit) + stream to the client. No
reachable Space-federated holder, or all miss/timeout → typed `10003` (D5).

- **Why scoped to the Space's federation set (W4):** B only ever issues a `BlobFetchRequest` to a
  peer it is federated-with **for that Space**. A third home **C** not sharing the Space is never
  queried (no metadata leak of "B wants blob_ref"), and A never serves a fetch it didn't receive
  → W4 holds by construction.
- **Honest caveat (the precise holder):** the blob *actually* lives at the **authoring client's
  home** (= the descriptor event's `sender`'s `home_node` — blobs land where uploaded, the
  client↔node upload). Resolving `sender → home` precisely from only `blob_ref` needs a
  **`blob_ref → descriptor-event` reverse index** — none exists; M12.3 does not build one. The
  Space-federation-set approach finds the blob wherever it lives among the Space's federated homes
  **without that index**, is W4-safe, and is exactly correct for the W1 two-home topology (one
  peer). **Reserved/flagged:** (i) precise `sender → home` reverse-resolution (needs the index);
  (ii) a fan-query refinement if a Space has many federated homes (try-home_node-only is the
  cheapest; fan-across-`federation_nodes` is the thorough). Mechanism-first picks home_node-first-
  then-the-rest, serialized.
- **P-wire (enabling, additive):** add `space_id: Option<String>` to `TransportMessage::BlobFetchRequest`
  (G1) and a `space_id: Option<&str>` param to `Connection::fetch_blob` (G11). The M12.2a fetch
  verb passes `Some(&args.space)` (G12). **M12.1 self-thread / any None caller → local-only**
  (the node attempts no federation fetch when `space_id` is None or the Space's `federation_nodes`
  is empty) → **W5 holds** (the self Space never federates; behaviour byte-identical to today).

**✅ LOCKED (Joe, 2026-06-17, by-recomms):** Space-federation-set resolution (home_node ∪
federation_nodes ∩ live sessions, home_node-first, serialized); additive `space_id` on
`BlobFetchRequest`; no `blob_ref → event` reverse index (correctly not built). Self/empty-federation
→ local-only → W5.

### P2 — Fetch correlation scheme (by `blob_ref`) — **recommended pick, flagged**

**The crux (audit A-02):** the federation conn is a concurrent bidirectional stream; `BlobChunk`
(G1) carries **no `blob_ref`** (the M12.1 client↔node channel is sequential, so it never needed
one). Concurrent federated fetches would make an interleaved chunk stream ambiguous.

**Recommended: serialize-per-peer federated fetch** — at most **one** in-flight federated fetch
per peer at a time. The shared `PendingFederationFetches = Arc<Mutex<HashMap<NodeXgid, FetchSlot>>>`
(keyed by **peer**) holds the one in-flight `FetchSlot { blob_ref: String, buf: Vec<u8>, waker:
oneshot::Sender<FetchOutcome> }`. The federation loop (G5) knows its own `peer_node_id`, so an
inbound `BlobChunk` (no `blob_ref`) appends unambiguously to `pending[peer].buf`; `BlobFetchEnd`
(carries `blob_ref`) verifies it matches the slot, fires `waker(Ok(buf))`; a blob-band inbound
`Error` fires `waker(Err(Unavailable))`. **No wire change to `BlobChunk`** → "reuse the M12.1
variants" honoured strictly; "correlated by `blob_ref`" honoured (the slot records it; Request/End
carry it; the loop verifies).

- The miss handler (G4): insert `pending[peer] = FetchSlot{…}` (**if one already in flight for
  that peer → wait-or-fail; serialize**); push `OutboundMsg::BlobFetchRequest{blob_ref, space_id}`
  into `federation_peer_senders[peer]`; `timeout(waker_rx)`. On timeout/Err → remove the slot,
  serve `10003`. Session-end (G6 cleanup) clears `pending[peer]` (fails the waiter → `10003`).
- **Flagged limitation + reserved hook:** one federated fetch per peer at a time — concurrent
  client misses to the **same** peer serialize behind the timeout (acceptable: lazy fetch is
  already a slow path; D4 is synchronous/blocking anyway; the same-blob race self-dedups — the
  second waits, then hits B's now-populated store). **Reserved scale hook** (the audit FK-1 β
  "add a correlation/request-id" note): add `blob_ref` to `BlobChunk` for true concurrent
  multiplexing. Mechanism-first; not built.

**✅ LOCKED (Joe, 2026-06-17, by-recomms):** serialize-one-fetch-per-peer; no `BlobChunk` wire
change; reserved scale-hook = add `blob_ref` to `BlobChunk` later.

### P3 — Synchronous fetch timeout (D4 — the bound before serving `10003`) — **recommended pick, flagged**

**Recommended:** the node-side federated-fetch **inner** timeout = `[sync].completion_timeout_seconds`
(the existing knob, default **5 s**, already the basis of `sync_completion_timeout` the fetch op
uses, G12) — **no new config value**. The client↔node `fetch_blob` **outer** timeout (G11) is
widened to **2 × `[sync].completion_timeout_seconds`** so the **inner < outer invariant** holds:
the node always serves the bytes (or the typed `10003`) before the client's outer timeout fires.
A federated fetch is two hops (client→B→A→B→client) vs the one-hop self-thread fetch, so the
client's existing single-hop timeout would be too tight as the outer bound.

- **Flagged limitation + reserved hook:** reuses one config for a two-hop path. **Reserved:**
  `[node].blob_fetch_timeout_seconds` if the federated round-trip needs its own bound under load
  (mechanism-first; flat-now).

**✅ LOCKED (Joe, 2026-06-17, by-recomms):** inner = `[sync].completion_timeout_seconds` (5 s
reuse), outer client `fetch_blob` = 2×; reserved `[node].blob_fetch_timeout_seconds`.

---

## §4 Runbook values — ✅ LOCKED 2026-06-17 (Joe, by-recomms)

| Value | Recommendation | Grounding |
|---|---|---|
| **V1 — chunk size on the federation serve leg** | reuse `BLOB_CHUNK_BYTES = 128 * 1024` (G2) verbatim; the federation serve arm chunks `bytes.chunks(BLOB_CHUNK_BYTES)` exactly like the client↔node serve (`app.rs:1889`) | G2 |
| **V2 — `10003` typed variant (D5)** | `BlobError::Unavailable` → `to_wire_code` `Some((10003, "blob_unavailable"))`; variant-add to the existing `BlobError` (NOT `ExchangeError`, M12-D9); RC-F-01 re-grep at build (free per G15) | G3, G15 |
| **V3 — timeouts (P3)** | inner = `[sync].completion_timeout_seconds` (5 s default); outer client `fetch_blob` = `2 ×` that | G11, G12, P3 |
| **V4 — injector variant** | `OutboundMsg::BlobFetchRequest { blob_ref: String, space_id: Option<String> }` (Clone-able, G7); the client↔node `out_rx` match (`app.rs:1662`) gets an `unreachable!()` arm (a client conn never injects a federation fetch into its own `out_tx`) | G5, G7 |
| **V5 — pending-fetch registry** | `PendingFederationFetches = Arc<Mutex<HashMap<NodeXgid, FetchSlot>>>`, `FetchSlot { blob_ref: String, buf: Vec<u8>, waker: oneshot::Sender<FetchOutcome> }`, `FetchOutcome = Result<Vec<u8>, ()>` (Err = unavailable/timeout); threaded like `FederationPeerSenders` (created at `run_node`, cloned into `handle_connection` + `run_federation_session_post_handshake`) | G7, G8, G10, P2 |

---

## §5 Build sequence (spine-first; per the §3/§4 recommended resolution)

Three code commits, then Chat's doc-bridge close. Per-commit DoD in §7. Each is its own atomic
commit (two-seat: Clair commits code; Chat authors the canonical-record doc-bridge; **Joe pushes
each**). **Pending Joe's lock of P1/P2/P3 + V1–V5** — if a pick is redirected, the affected commit
reshapes; the spine-first order is unaffected.

- **C1 — type the `10003` reject (D5; the W2 baseline; tiny, no behaviour change).**
  Add `BlobError::Unavailable` (G3) + the `to_wire_code` arm → `Some((10003, "blob_unavailable"))`
  (V2). Replace the defensive literal at `app.rs:1919` (`blob_err(10003, "blob_unavailable")`)
  with the typed path (`BlobError::Unavailable.to_wire_code()` → `blob_err(code, name)`). **No
  behaviour change** — the client↔node `Ok(None)` arm still serves `10003` (self-thread never
  misses); only the *source* of the tuple changes literal → typed. RC-F-01 re-grep first (G15).
  **Spines W2.** Unit test (in `blob_store.rs` tests): `BlobError::Unavailable.to_wire_code() ==
  Some((10003, "blob_unavailable"))`. **RED-on-revert (W2 baseline):** the pre-typing literal is
  the baseline; the typed path is byte-identical on the wire (assert the wire tuple unchanged).

- **C2 — the federation fetch path (D1/D4; the spine + in-process witness).** *(The bulk.)*
  - **Wire (G1, V4):** `space_id: Option<String>` on `TransportMessage::BlobFetchRequest`
    (additive `#[serde(default, skip_serializing_if = "Option::is_none")]`);
    `OutboundMsg::BlobFetchRequest { blob_ref, space_id }` (V4) + the `unreachable!()` arm in the
    client↔node `out_rx` match (`app.rs:1662`).
  - **Client (G11, G12, P1-wire, P3):** `Connection::fetch_blob(blob_ref, space_id: Option<&str>,
    timeout)` — send `BlobFetchRequest{blob_ref, space_id}`; widen the outer timeout (V3);
    `ops::fetch_attachments` passes `Some(&args.space)` (G12).
  - **Registry (V5):** `PendingFederationFetches` created at `run_node`, threaded into
    `handle_connection` (G10) + `run_federation_session_post_handshake` (G6, new param).
  - **Federation loop arm-add (G5, the D1 spine):** ahead of the `Ok(_)` catch-all (`:2561`),
    new inbound arms — (a) `BlobFetchRequest{blob_ref, ..}` from peer → serve from local
    `BlobStore` (mirror the client↔node serve at `app.rs:1880-1933`: stream `BlobChunk`* +
    `BlobFetchEnd` via `conn.send_transport`, or `Error{10003}` on local miss); (b) `BlobChunk`
    → append to `pending[peer].buf` (P2); (c) `BlobFetchEnd{blob_ref}` → take `pending[peer]`,
    verify `blob_ref` matches, `waker(Ok(buf))`; (d) blob-band `Error` → take `pending[peer]`,
    `waker(Err(()))`. New `out_rx` arm → `OutboundMsg::BlobFetchRequest{blob_ref, space_id}` →
    `conn.send_transport(BlobFetchRequest{blob_ref, space_id})`. Cleanup tail (`:2601`) clears
    `pending[peer]` (fail the waiter → `10003`).
  - **Node-internal helper (the testable spine):** extract
    `federation_fetch_blob(blob_ref, space_id, &runtime, &federation_peer_senders,
    &pending_fetches, inner_timeout) -> Result<Vec<u8>, ()>` — resolve the holders (P1: the
    Space's `home_node` ∪ `federation_nodes` ∩ live sessions), serialize (P2: register
    `pending[peer]`, fail-if-in-flight), inject `OutboundMsg::BlobFetchRequest`, `timeout(waker)`,
    on Ok return bytes (caller `BlobStore::put`s + serves). Called from the client↔node miss
    handler AND the in-process witness.
  - **Client↔node miss handler rewrite (G4):** the `Ok(None)` arm at `app.rs:1917` → if
    `space_id` present and the Space has reachable federated holders, `federation_fetch_blob(…)`;
    on Ok → `BlobStore::put` + stream chunks + `BlobFetchEnd` to the client; on Err / no holder →
    `blob_err(10003, "blob_unavailable")` (the typed path from C1). **Spines W1 (in-process),
    W3, W4, W5.**
  - **In-suite in-process witness (G13, the spine gate):** extend the phase9 federation harness —
    two `NodeRuntime`s A + B over a duplex pair, federated, sharing a Space; A's `BlobStore` holds
    ciphertext blob X, B's is empty; spawn both session loops; call B's `federation_fetch_blob(X,
    space=S)` → assert byte-identical to A's bytes; assert the bytes are **ciphertext** (W3 across
    homes) + B is content-blind by construction; a non-Space third home C is never queried (W4).
    **RED-on-revert:** neuter the federation collect arm (or A's serve arm) → B's fetch times out
    → `Unavailable` / `10003`.

- **C3 — box-gated real-binary two-home e2e (headline W1; RUN to Joe).**
  `xgen-mptest` two-home federation (G14, sibling to `m12_2a_self_thread_e2e.rs`; the F2/F9
  harness-control `add-peer`/`initiate` seam drives federation): A + B federated, sharing a Space;
  a member on A authors a `message.file` (descriptor federates to B via the existing eager push);
  a client on B fetches → B miss → B federation-fetches from A → **byte-identical**. Box-gated
  `#[ignore]`. **RED-on-revert:** neuter the federation fetch arm → B miss → `10003`. **The
  box-gated RUN comes back to Joe separately** (the in-suite gate is C2's in-process witness;
  C3 ships the test, the RUN is Joe's box).

**(Chat) doc-bridge + M12.3 close** — canonical-record flips (CLAUDE PLAY, JOURNAL, ROADMAP,
design/audit/runbook status), the P1/P2/P3 picks recorded as-built, the box-gated RUN result
recorded; M12-D6 stays a flagged DECISIONS promotion candidate (Joe's call, not this arc).

---

## §6 Witnesses (RED-on-revert; design §4)

- **W1 — federated round-trip (headline).** A + B federated, sharing a Space; member on A authors
  `message.file` → descriptor federates to B → member on B reads + fetches → B miss →
  B federation-fetches from A → **byte-identical**. *In-process (C2):* via `federation_fetch_blob`.
  *Box-gated real-binary (C3):* end-to-end through both binaries. **RED-on-revert:** neuter the
  federation fetch arm → B miss → `10003`.
- **W2 — typed unavailable.** A genuine miss (no reachable Space-federated holder / all miss)
  surfaces the **typed** `BlobError::Unavailable` → wire `10003` (replaces the literal).
  **RED-on-revert:** the pre-typing literal path is the baseline (C1); wire tuple byte-identical.
- **W3 — content-blind across homes.** The bytes B fetches + stores are ciphertext (M12-D5
  inherited); B is content-blind by construction (inherits M12.1 W2). *In-process (C2).*
- **W4 — never-leak.** A third home **C** not sharing the Space / federation is never queried and
  never serves the blob (the resolution is scoped to the Space's `federation_nodes` ∩ live
  sessions, P1). *In-process (C2): assert C's session sees no `BlobFetchRequest`.*
- **W5 — self stays federation-free.** A self-thread blob still never federates: the self Space's
  `federation_nodes` is empty → `federation_fetch_blob` attempts nothing → the miss path is
  byte-identical to today (M11/D-021 intact; M12.3's federation surface additive). *In-process
  (C2) + asserted at C1 (the typed path on a None/empty-federation miss == today).*

---

## §7 Definition of Done

**Per-commit gate (C1–C3):** `cargo build --workspace` 0-error;
`cargo clippy --workspace --all-targets --all-features -- -D warnings` clean;
`cargo test --workspace` green (baseline **1445/0** + the commit's new in-suite tests; C3's
box-gated e2e is `#[ignore]`, not counted); for the spine commits (C1, C2) RED-on-revert recorded
in the commit message. RC-F-01 re-grep recorded at C1.

**Milestone gate (at C3 / close):** W1–W5 expressed (W1 in-process + box-gated; W2/W3/W4/W5
in-suite); RED-on-revert recorded for W1(in-process)/W2/W4; the P1/P2/P3 picks + the
`space_id`-on-fetch additive field + the serialize-per-peer limitation stated in the close. The
**box-gated two-home e2e RUN comes back to Joe separately** (not an in-suite DoD line). Build
target `CARGO_TARGET_DIR=C:/cargo-targets/XGenProtocol`.

*(No "commit pushed" DoD line — unflippable inside its own commit; `Status: COMPLETED` is the
shipped signal. Joe pushes each commit.)*

---

## §8 Out of scope (M12.4 / reserved — do NOT pull in)

- **M12.4 erasure** — the `message.redact` content applier (F2a), the F2b sender `Retention` read
  (M12's **first** production `Retention` reader), crypto-shred destroy-to-erase (D3-gated), the
  reserved WORM/legal-hold operator/module hook.
- **`Retention` enforcement / reading** — M12.3 reserves the Retained eager-replicate hook (D3);
  it does **not** read or enforce `Retention` (zero production readers, G-confirmed).
- **Eager / replicated blob federation** — reserved (D2 lazy-default); the Retained-eager override
  is the named hook, unbuilt.
- **Async held-pending client signal** (D4 β) — reserved unless Joe locks it.
- **`BlobChunk` request-id for concurrent multiplexing** (P2 scale hook) — reserved; serialize-
  per-peer is the M12.3 mechanism.
- **Precise `sender → home` reverse-resolution** (P1 reserved (i)) — needs a `blob_ref →
  descriptor-event` index; not built.
- **`[node].blob_fetch_timeout_seconds`** (P3 reserved) — reserved; reuse `[sync].completion_timeout_seconds`.
- **Pattern-A tier→size / lifetime tables** (F6/F8) — reserved (mechanism-first).
- **The WORM/archival vault** — operator/module responsibility (F7); M12.3 reserves the hook only.

---

## §9 Sequence + entry (Rule 0)

this runbook → **Joe locks P1/P2/P3 + V1–V5 + the §5 sequence** → implement C1→C3 (spine-first,
per-commit) → hand Joe each push → Chat doc-bridge → M12.3 close → **M12.4** (erasure) → M12 close
→ Round-2 final pre-UI gate → UI → Streams.

**Entry (Rule 0):** `CLAUDE.md` PLAY → `JOURNAL.md` J-386 → `tasks/M12_3_FEDERATION_FETCH_DESIGN.md`
(M12.3-D1..D6) → `tasks/M12_3_FEDERATION_FETCH_PHASE0_AUDIT.md` (forks + findings) →
`tasks/M12_1_BLOB_STORE_ATTACH_IMPL.md` (the built primitives) → `docs/ROADMAP.md` (M12).
