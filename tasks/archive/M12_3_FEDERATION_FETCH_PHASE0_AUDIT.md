# M12.3 — Federation fetch-blob-by-hash: D-071 Phase-0 audit
> **Status**: COMPLETED
> Version: 1.0
> Date: Jun 2026
> **Last updated**: 2026-06-17
> Language: English
> Author: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> License: BSL 1.1 (converts to GPL upon project handover)

---

## §1 Purpose & verdict

The Clair-authored M12.3 D-071 Phase-0 audit — the **federation** sub-arc of M12
(M12-D10): the fetch-blob-by-hash protocol + the F3 lazy/eager lock (M12-D8,
deferred to here) + the Retained(T4) durability floor (F7) + the held-pending /
unavailable client signal + the reserved `BlobError` **`10003 blob_unavailable`**.

Grounds the five M12.3 work-items to file:line on `main @ 79c3870` (tree clean,
in-suite 1445/0). Audit-only (D-078 — production code read this session, not
inferred from the runbook); Chat lands the J-386 doc-bridge + canonical flips.

**Verdict: GO.** The gap is precise and bounded. Every prerequisite is built:
the content-addressed `BlobStore`, the `Descriptor`-carrying `message.file`
event (which federates eagerly today, like `message.text`), the chunked-base64
client↔node fetch, and the reserved `10003` slot. M12.3 is a net-new
**home↔home fetch path** + a **client miss-signal** + **typing the reserved
`10003`** — no blocker. Six forks (FK-1..FK-6) are real design calls with a
recommendation each; none gates GO. Two of them (FK-2 F3 lazy/eager, FK-4
Retained-floor placement) are the load-bearing Joe-locks.

**Entry (Rule 0):** `CLAUDE.md` PLAY → `JOURNAL.md` J-385 →
`tasks/M12_ATTACHMENTS_DESIGN.md` (M12-D8 F3, M12-D9 reject type, M12-D10 split)
→ this audit → `tasks/M12_1_BLOB_STORE_ATTACH_IMPL.md` (the built primitives) →
`docs/ROADMAP.md` (M12, F3/F7).

---

## §2 Grounding ledger (the five items, re-confirmed to file:line on `main @ 79c3870`)

| # | Item / seam | Location | Grounded state |
|---|---|---|---|
| L1 | **Descriptor `message.file` event federates eagerly** | `xgen-node/src/federation_session.rs:280` (`apply_federation_push`) + `:84` (`stream_federation_delta`) | A `message.file` event rides `apply_federation_push` (eager push on local-accept, `try_send(OutboundMsg::Event)`) + the F-1a handshake delta exactly as `message.text` — no blob coupling. The **descriptor** already crosses homes. |
| L2 | **Blob bytes do NOT federate** | `xgen-core/src/blob_store.rs` (the whole store); grep: `BlobStore`/`upload_blob`/`fetch_blob`/`BlobUploadBegin` occur **only** in client↔node files (`app.rs`, `ops.rs`, `connection.rs`, `wire/types.rs`, `encryption/blob.rs`, tests) — **zero** in `federation_session.rs` | No node↔node blob path exists. Net-new confirmed. |
| L3 | **The F-2 federation loop drops blob `TransportMessage`s** | `xgen-node/src/app.rs:2512-2565` (steady-state `tokio::select!`) | Inbound arm routes **only** `Inbound::Event` (→`process_inbound`) + `Goodbye`/`IdentityReplicate`/`Ping`/`Pong`/`Closed`; the catch-all is `Ok(_) => { /* silently ignore */ }` (`:2561`). A `BlobFetchRequest`/`BlobChunk` arriving on a federation session is **dropped**. |
| L4 | **The home↔home conn is a concurrent bidirectional Event stream** | `xgen-node/src/app.rs:2310` (`run_federation_session_post_handshake`); `:2512` (select! of `conn.recv()` + `out_rx.recv()`); F-2a = one-WS-per-pair | Not a sequential request/response channel. This is the load-bearing complexity for a lazy fetch (FK-1, A-02). |
| L5 | **Client↔node fetch = a clean SEQUENTIAL request/response** | `xgen-core/src/transport/connection.rs:317` (`fetch_blob`): send `BlobFetchRequest` → `loop { recv → BlobChunk \| BlobFetchEnd \| Error }`; node handler `xgen-node/src/app.rs:1880-1934` | The proven shape M12.1/M12.2a ship. Reusable verbatim *only* on a channel without a concurrent inbound event stream (FK-1). |
| L6 | **The miss seam: `get` → `Ok(None)` → defensive 10003 literal** | `xgen-core/src/blob_store.rs:127` (`get` returns `Ok(None)` on absent); `xgen-node/src/app.rs:1917-1925` (`Ok(None) => send blob_err(10003, "blob_unavailable")`) | Today a client↔node fetch of an absent blob hard-errors the client (`fetch_blob` → `TransportError`, `connection.rs:342-351`). The comment names it: *"typed `BlobError::Unavailable` + lazy fetch-by-hash land at M12.3."* This literal is the seam M12.3 replaces. |
| L7 | **`BlobError` + `to_wire_code` (M12-D9 home)** | `xgen-core/src/blob_store.rs:43-82` | Domain 10 (10000–10999), parallel to `ExchangeError`. `HashMismatch`→10001, `TooLarge`→10002 (both typed+live). **No `Unavailable` arm.** `to_wire_code` doc: *"Reserved (added with its producer): `10003 blob_unavailable` (F3, M12.3)."* |
| L8 | **10003 collision-check (RC-F-01 / M10.1 discipline)** | grep `10003`/`blob_unavailable`/domain-10 literals | 10003 occurs **only** in `blob_store.rs` docs (reserved) + `app.rs:1916/1919` (the defensive literal). Domain 10 used **only** by the blob band (10001/10002 typed, 10003 reserved). **10003 free as a typed variant.** |
| L9 | **`PendingBuffer` / `HeldPending` shape** | `xgen-core/src/dag/pending.rs:88-120` (`BufferedEntry`, `PendingBuffer`); `ValidationOutcome::HeldPending` in `exchange.rs` | Holds DAG **events** keyed by `EventXgid`, **node-side**, on 3 triggers (missing predecessor / signer-identity / federation-relationship), 30s/180s timeout, arrival hooks, timeout→discard+`TimedOut`. The hold-until-resolved + timeout + arrival-hook **pattern** — but event-shaped (A-04). |
| L10 | **`ModulePolicy` / `Retention` — zero production readers** | def `xgen-common/src/trust_assertion.rs:156-236` (`ModulePolicy{erasability:{retention: Erasable\|Retained}}`, on `claims.extra["module_policy"]`, read via `claims.module_policy()`); producer `xgen-auth-module` | grep `.module_policy()`/`Retention::` outside def+tests+issuer = **empty**. The J-380 "zero readers" finding holds. M12.4 building "the first production reader" stays consistent (A-05). |
| L11 | **F7 framing** | `docs/ROADMAP.md` L826 (M12 forks) | "Retained (T4)" at protocol = a **ciphertext durability floor** (don't drop the bytes) + erasure refusal; tiering/offload = a reserved operator/module hook, **vault NOT built** (mark + reserve). |
| L12 | **M12.3 is the FIRST M12 sub-arc to touch federation** | M12-D10; M12.1/M12.2 = self-thread/intra-home (`W5` never-federates) | M12.3 deliberately introduces the federation blob surface. The self path stays federation-free (M11/D-021 intact); a *self-thread* blob still never federates. |

---

## §3 Findings (M12.3-A-01..A-06)

### M12.3-A-01 — The gap, precisely: the descriptor crosses homes, the bytes don't

Grounded across L1/L2/L3. A `message.file` event (the `Descriptor`-carrying
event, M12-D2) federates **eagerly** to every federated home exactly like
`message.text` (`apply_federation_push`, `stream_federation_delta`). The
ciphertext blob it references lives in a **content-blind `BlobStore`** that has
**no node↔node transfer path** — and the steady-state federation loop **silently
drops** any blob `TransportMessage` (`app.rs:2561`).

**Consequence.** A peer home **B** that receives a federated `message.file`
authored at home **A** holds the descriptor (`blob_ref`, per-blob `key`,
`plaintext_hash`) but **not** the ciphertext. A member's client on B that fetches
the blob (client↔node) hits B's `BlobStore::get` → `Ok(None)` → the defensive
`blob_err(10003)` (L6) → the client hard-errors. **M12.3's job: give B a path to
obtain the blob from A (the home that has it) — federation fetch-blob-by-hash.**

### M12.3-A-02 — The home↔home channel is a concurrent bidirectional Event stream, not a sequential fetch channel (the load-bearing complexity)

Grounded L4/L5. The client↔node `fetch_blob` (L5) is a clean sequential
request/response: send `BlobFetchRequest`, then `loop { recv → chunk | end |
error }` — it works because the client↔node loop has **no concurrent inbound
event stream** interleaving the chunks.

The federation session (L4) is the opposite: one persistent `Connection<S>` per
pair (F-2a), driven by a `tokio::select!` of `conn.recv()` (steady-state events
**from** the peer) and `out_rx.recv()` (events to push **to** the peer). If B
sends a `BlobFetchRequest` to A on this shared conn and awaits chunks: (a) A's
chunks **interleave** with steady-state events arriving from A, and (b) B's recv
arm routes `Event`/`Goodbye`/`IdentityReplicate` only — inbound `BlobChunk`s hit
the `Ok(_) => ignore` catch-all. **Both sides' federation loops need new routing,
and B must correlate streamed chunks with its in-flight request.** The proven
sequential `fetch_blob` shape does **not** drop in unmodified. This is the real
cost of *lazy* fetch and the substance of FK-1.

### M12.3-A-03 — F3 lazy vs eager: lazy pays A-02; eager reuses the existing push machinery

Grounded L1/L11 + A-02. The two shapes (FK-2):

- **Eager (replicate):** the authoring home pushes the blob bytes alongside /
  right after the descriptor-event push. The descriptor push **already works**
  (`apply_federation_push`); eager-blob piggybacks on the same trigger. Every
  federated home stores every blob → **no fetch protocol, no miss, simplest
  client UX**. Cost: storage (every home stores every blob) + bandwidth (push
  even if never read on that peer).
- **Lazy:** the peer receives the descriptor only; fetches on demand → pays the
  A-02 fetch protocol + the A-04 miss signal. Saves storage/bandwidth on peers
  that never read the blob.

M12-D8 leans **lazy** (provisional, audit-grounded-not-locked), with the
Retained(T4) eager/replicated override coupled to F7 (A-05). Honest note for the
lock: **lazy is the harder build (A-02); eager is the cheaper build at a heavier
runtime cost.** The lock is formally deferred to *this* grounding — genuinely
open (FK-2).

### M12.3-A-04 — Held-pending / unavailable client signal: PendingBuffer is the precedent, not the object

Grounded L6/L9. `PendingBuffer` (L9) holds DAG **events** keyed by `EventXgid`,
**node-side**, waiting on missing DAG dependencies. A blob-miss differs on both
axes: the **unit** is a *blob* (`blob_ref`, not an event) and the **waiter** is a
*client* (awaiting a byte transfer the node must go fetch from a peer), at the
*transfer* layer, not the *ingest* layer. So the M12-D8 "extend `HeldPending` /
`PendingBuffer`" framing is a **pattern reuse** (hold-until-resolved + timeout +
arrival-hook), not a literal extension of the event buffer.

Today the client sees a hard error on a miss (L6). The two client-signal shapes
(FK-3):

- **Synchronous:** B blocks the client's `fetch_blob` while it federation-fetches
  from A (bounded by timeout → serve the chunks, or 10003). **No new client
  wire** — reuses the existing `fetch_blob` await + a longer timeout.
- **Asynchronous (held-pending):** B replies "held-pending, retry" immediately,
  fetches in the background, the client re-requests. Needs a new held-pending
  `TransportMessage` + client retry loop — *mirroring* (not extending)
  `PendingBuffer`.

### M12.3-A-05 — Retained(T4) durability floor: zero Retention readers; the M12.3↔M12.4 boundary tension

Grounded L10/L11. `Retention { Erasable | Retained }` has **zero production
readers** (L10). M12-D10 reserves "M12's first production reader of the dormant
AI-D8 enforcement (T4/Retained refuses erasure)" for **M12.4**. But M12-D8
couples a **Retained eager/replicated override** to the F7 durability floor **in
M12.3** ("can't legal-hold a single droppable copy").

**The tension:** reading `Retention` to drive eager-for-Retained would make
**M12.3** the first Retention reader — contradicting M12-D10. F7 itself (L11) is
"mark + reserve, not build-the-vault." Resolution is FK-4: keep M12.3
mechanism-first (lazy fetch + unavailable signal), reserve the Retained
eager-replicate override as a **named hook** coupled to F7 (no Retention read in
M12.3); the first Retention reader stays at M12.4 with the rest of the
erasure/retention enforcement. Surfaced so Joe sees the boundary explicitly.

### M12.3-A-06 — `10003 blob_unavailable`: free, already a defensive literal, not yet typed

Grounded L7/L8. Domain 10 is used only by the blob band; **10003 is free** as a
typed variant (RC-F-01 discipline satisfied — re-grep at build). The current
emit at `app.rs:1919` is a **literal tuple** `blob_err(10003, "blob_unavailable")`
— a defensive client↔node reply on `Ok(None)` so the client errors rather than
hangs. `BlobError` has **no `Unavailable` arm** (L7).

M12.3's wire-code work (per M12-D9 — the net-new parallel type at the
transfer/ingest gate is **`BlobError`**, already in place, *not* `ExchangeError`):
add the typed `BlobError::Unavailable` variant + `to_wire_code` →
`Some((10003, "blob_unavailable"))`; replace the `app.rs:1919` defensive literal
with the typed path; emit 10003 only when the (lazy) federation fetch **also**
fails. The **first typed emission site** = the federated-read genuine miss (B
cannot obtain the blob from A). This is a variant-add to an existing type, not a
new error-type creation (FK-5 — a confirm, recorded as a decision-point).

---

## §4 Forks (FK-1..FK-6 — each with a recommendation; Joe locks at design)

### FK-1 — Federation fetch transport shape (A-01/A-02) — *if lazy (FK-2)*

How does B obtain the blob from A over the home↔home relationship?

- **α — dedicated/ephemeral fetch connection (B→A).** Mirrors the proven
  sequential client↔node `fetch_blob`/node-handler pair (L5) verbatim; sidesteps
  the select!-loop multiplexing (A-02). Cost: re-establishing transport/auth for
  the fetch (the full federation handshake is heavier than a blob fetch wants).
- **β — multiplex on the existing established federation session.** Add a
  correlation/request-id to the blob variants; weave a pending-fetch map into the
  select! loop; route blob variants on **both** sides. Reuses the live relationship
  + auth (no re-handshake). Cost: the multiplexing machinery + both-sides routing
  (the A-02 complexity, taken head-on).
- **γ (orthogonal sub-choice) — wire family:** reuse the existing
  `TransportMessage` blob variants on the federation conn (the conn already
  carries `TransportMessage`, e.g. `SyncComplete`) **vs** a net-new
  `FederationMessage::BlobFetch*` family.

**Recommendation: defer the precise shape to design; provisional lean β
(multiplex on the established session) reusing the existing `TransportMessage`
blob variants (γ-reuse)** — the relationship + transport + auth already exist
(L4), so β avoids re-handshaking, and the `TransportMessage` blob variants +
node `BlobFetchRequest` handler (L5/L6) are reusable on the same channel. **Flag
α as the clean fallback** if the select!-loop multiplexing proves too invasive.
Design must also resolve the **"which peer holds it"** resolution (the Space's
`home_node` / `federation_nodes`, or the event `sender`'s home) — surfaced, not
locked.

### FK-2 — F3 lazy vs eager (A-03; = the M12-D8 deferred lock) — **load-bearing**

- **Recommendation: lazy-default + Retained-eager-override-as-reserved-hook**
  (per the M12-D8 lean + the F3 storage-efficiency rationale). **Honest caveat
  for the lock:** lazy carries the A-02/FK-1 fetch-protocol complexity; eager
  reuses the existing push machinery and is the materially smaller build (no
  fetch protocol, no miss signal) at the cost of every-home-stores-every-blob.
  If Joe wants the smallest M12.3, eager-only is the lighter path. The lean
  stays lazy; the lock is genuinely open and is Joe's.

### FK-3 — Held-pending / unavailable client signal (A-04; item 4)

- **α — synchronous** (B blocks the client fetch during the federation round-trip;
  timeout → serve or 10003; no new client wire).
- **β — asynchronous held-pending** (B replies "retry"; client re-requests; new
  wire + client retry; mirrors `PendingBuffer`).

**Recommendation: synchronous-first (α)** — mechanism-first, no new client wire,
consistent with the M12.2a F6 "flat-field-now, Pattern-A-reserved" precedent.
**Reserve β** (the held-pending signal) as a named seam if/when long-latency
federated fetches make blocking the client untenable. Honest note: M12-D8 names
`PendingBuffer` as "the model to extend," leaning β; grounding (A-04) shows
`PendingBuffer` is event-shaped, so β would *mirror* not *extend* it, and α is
materially simpler for M12.3's scope. Joe's call.

### FK-4 — Retained(T4) durability floor placement (A-05; the M12.3↔M12.4 boundary) — **load-bearing**

- **α — reserve-the-hook.** M12.3 builds lazy fetch + the unavailable signal; the
  Retained eager-replicate override is a **named hook** (mechanism reserved), **no
  Retention read** in M12.3; the first Retention reader stays at M12.4.
- **β — live floor.** M12.3 reads `Retention` to drive eager-for-Retained →
  M12.3 becomes the first Retention reader (re-scopes M12.4's "first reader"
  claim).

**Recommendation: α (reserve-the-hook)** — mechanism-first; keeps M12.4 the
first-Retention-reader per M12-D10; avoids pulling erasure/retention enforcement
forward into the federation arc; honors F7's "mark + reserve, not build-the-vault"
(L11). The M12-D8-vs-M12-D10 tension is surfaced (A-05) for Joe's explicit call.

### FK-5 — `10003` typing + the reject home (A-06; M12-D9) — *a confirm, recorded*

- **Recommendation:** add typed `BlobError::Unavailable` → `Some((10003,
  "blob_unavailable"))`; replace the `app.rs:1919` defensive literal with the
  typed path; first typed emission = the federated-read miss. This **extends the
  existing `BlobError`** (already the net-new parallel transfer/ingest type per
  M12-D9 — *not* `ExchangeError`), a variant-add not a new type. Re-grep the
  register at build (10003 free now, L8).

### FK-6 — Sub-arc shape / witness surface (A-01; M12-D10)

M12.3 is the first M12 sub-arc to touch federation (L12). Witness surface:
two-home federated blob read (§5). The witness harness is a sub-choice:

- **α — `xgen-mptest` real-binary two-home federation** (the F2/F9 harness-control
  `add-peer`/`initiate` seam already drives federation; sibling to the
  `m12_2a_self_thread_e2e` real-binary path; box-gated `#[ignore]`).
- **β — in-process two-`NodeRuntime`** (phase9-style; in-suite, no box).

**Recommendation: defer to the runbook; lean a box-gated real-binary witness (α)
for the headline + an in-suite in-process witness (β) for the spine** — mirrors
the M12.2a split (in-suite spine + box-gated e2e). Design/runbook firm the exact
set. Whether M12.3 also re-scopes into M12.3a/b (fetch protocol vs
signal/durability) is a design-phase split call, not locked here.

---

## §5 Witness sketch (RED-on-revert; design/runbook firm the set)

- **W1 — federated round-trip (headline).** Homes A + B federated, sharing a
  Space. A member authors a `message.file` on A → the descriptor event federates
  to B (existing eager push) → a member on B reads the event + fetches the blob →
  B miss → B federation-fetches from A → serves **byte-identical**. RED-on-revert:
  neuter the federation fetch → B miss → `10003 blob_unavailable`.
- **W2 — typed unavailable.** A genuine miss (blob nowhere reachable) surfaces the
  **typed** `BlobError::Unavailable` → wire `10003` (replaces the defensive
  literal). RED-on-revert: the literal path returns a hang/wrong code.
- **W3 — content-blind across homes.** The bytes B fetches + stores are ciphertext
  (M12-D5 inherited); B is content-blind by construction. (Inherits M12.1 W2.)
- **W4 — never-leak.** A third home **C** that does **not** share the Space /
  federation never receives or serves the blob (federation-scoped; the
  `federation_nodes` / policy gates hold).
- **W5 — self stays federation-free.** A *self-thread* blob still never federates
  (M11/D-021 intact, L12) — M12.3's federation surface is additive, not a
  weakening of the self path.

---

## §6 Out of scope (M12.4 / reserved — do NOT pull in)

- **M12.4 erasure** — the `message.redact` content applier (F2a), the F2b sender
  `Retention` read (M12's **first** production Retention reader, A-05),
  crypto-shred destroy-to-erase (D3-gated), the reserved WORM/legal-hold
  operator/module hook.
- **Retention *enforcement*** — M12.3 reserves the Retained eager-replicate hook
  (FK-4 α); it does not read or enforce `Retention`.
- **Pattern-A tier→size / lifetime tables** (F6/F8) — reserved (mechanism-first;
  the M12.2a flat-field precedent).
- **The WORM/archival vault** — operator/module responsibility (F7, L11); M12.3
  reserves the hook only.
- **Async held-pending client signal** (FK-3 β) — reserved unless Joe locks it.

---

## §7 Entry (Rule 0) + handoff

**Entry:** `CLAUDE.md` PLAY → `JOURNAL.md` J-385 → this audit §2 (ledger) + §3
(findings) + §4 (forks) → `tasks/M12_ATTACHMENTS_DESIGN.md` (M12-D8/D9/D10) →
`tasks/M12_1_BLOB_STORE_ATTACH_IMPL.md` (the built primitives) → `docs/ROADMAP.md`
(M12, F3/F7).

**Handoff:** audit-only commit (Clair's seat). **No CLAUDE/JOURNAL/ROADMAP flips**
— that is Chat's J-386 doc-bridge. The two load-bearing Joe-locks at design are
**FK-2** (F3 lazy/eager) and **FK-4** (Retained-floor placement); FK-1/FK-3/FK-6
are shape calls; FK-5 is a confirm. **M12-D6 stays a flagged DECISIONS.md
promotion candidate — not this arc.** Sequence: this audit → Chat J-386 bridge →
design (Chat/Joe, lock FK-1..FK-6) → Joe-lock → Clair runbook → implement → Chat
doc-bridge → M12.3 close → M12.4. No code until the M12.3 design is Joe-locked.
