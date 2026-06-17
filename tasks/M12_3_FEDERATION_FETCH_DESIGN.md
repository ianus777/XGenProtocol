# M12.3 — Federation fetch-blob-by-hash: Design (Joe-LOCKED)
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

The Joe-LOCKED M12.3 design, authored by Chat at the design-lock (J-386) after the fork
discussion on Clair's M12.3 D-071 Phase-0 audit. Sits on
`tasks/M12_3_FEDERATION_FETCH_PHASE0_AUDIT.md` (v1.0, GO, findings M12.3-A-01..06, forks
FK-1..FK-6, committed `fee9271`, pushed) and the master M12 design
`tasks/M12_ATTACHMENTS_DESIGN.md` (M12-D8 F3 deferred-to-here, M12-D9 parallel reject type,
M12-D10 M12.3 scope).

M12.3 is the **federation** sub-arc of M12: the cross-home (home↔home) dimension of attachments.
M12.1 built the load-bearing mechanism (per-blob crypto + chunked-base64 WS transfer +
content-blind `BlobStore`) on the **client↔node** channel; M12.2 polished the client surface +
the F6 gate + the F9 data-root. M12.3 makes a blob **reachable across homes**: a peer that holds
the `message.file` descriptor (which already federates) but not the ciphertext can fetch it from
the home that does.

D-071 arc discipline: this design → Clair authors the M12.3 runbook → implement spine-first →
Chat doc-bridge → M12.3 close. **No code precedes the runbook; the runbook does not precede this
lock.** Decisions are arc-local (D-069). **M12-D6 (universal-E2E protocol-layer) stays a flagged
DECISIONS.md promotion candidate — not this arc.**

---

## §2 The grounded gap (audit M12.3-A-01 / A-02; Chat-reverified on `main @ fee9271`)

The `message.file` **descriptor event already crosses homes** — it rides `apply_federation_push`
exactly like `message.text` (eager push at ingest). The **blob bytes do not**: there is no
node↔node blob path. The federation steady-state loop (`xgen-node/src/app.rs`) is a
`tokio::select!` whose inbound `match` has explicit arms for `Event`-bearing fanout and
`IdentityReplicate`, and a catch-all `Ok(_) =>` "silently ignore" (app.rs:2561) — so any blob
`TransportMessage` arriving on a federation session is **dropped**.

Consequence: a peer home **B** federated with **A** in a shared Space receives the descriptor but
not the ciphertext. A client on B reads the event, fetches by `blob_ref` → `BlobStore::get` →
`Ok(None)` → today a **defensive literal** `blob_err(10003, "blob_unavailable")` (app.rs:1919,
the comment already says "typed `BlobError::Unavailable` + lazy fetch-by-hash land at M12.3").

**The load-bearing shape complication (FK-1):** the home↔home conn is a **concurrent
bidirectional Event stream**, not the sequential request/response the client↔node `fetch_blob`
assumes — so the proven fetch shape does not drop in unmodified. **Chat-reverified (D-065):**
routing blob variants into the federation loop is a **clean arm-add** — new `Inbound` arms ahead
of the `Ok(_)` catch-all, with replies on the same `conn` via the existing `OutboundMsg`/`out_rx`
outbound side (app.rs:2567+); blob fetch/reply is its own request/response correlated by
`blob_ref`, riding *alongside* the Event push, not interleaved into DAG admission.

---

## §3 Locked decisions (M12.3-D1..D6)

### M12.3-D1 — Fetch transport = multiplex the federation session (FK-1: β + γ-reuse)

**Locked: β — multiplex the established federation session.** Add the fetch protocol as new
inbound arms on the federation loop's `match` (replacing part of the `Ok(_)` drop), reusing the
**M12.1 `TransportMessage` blob variants** (`BlobFetchRequest` / `BlobChunk` / `BlobFetchEnd`)
on the same channel — the federation relationship + transport + auth already exist, so no
re-handshake and no new conn lifecycle. Replies ride the existing `OutboundMsg`/`out_rx` outbound
side; the exchange is correlated by `blob_ref`, concurrent with (not interleaved into) the Event
push.

- Rejected as the standing shape: **α — an ephemeral side-conn** per fetch (a second conn
  lifecycle + a second handshake on an already-authenticated relationship). **Flagged as the
  documented fallback** if multiplexing the bidirectional stream proves to entangle with Event
  ordering at implementation (the audit's α-fallback note).
- **Runbook-bound open item (not a Joe-lock):** the **"which peer holds it"** resolution — the
  Space's `home_node` / `federation_nodes`, or the event `sender`'s home. The runbook grounds the
  source and picks; this is a resolution-source detail like the M12.1 chunk-size, not a fork.

### M12.3-D2 — F3 lazy/eager: lazy-default; Retained-eager an unbuilt reserved hook (FK-2; = the M12-D8 deferred lock) — **load-bearing**

**Locked: lazy-default.** B fetches the ciphertext **on demand** at first read-miss (the D1
federated fetch), not eagerly at descriptor-receipt. This is the M12-D8 audit-grounded lean made
formal here (M12-D8 deferred the lock to "the M12.3 federation grounding" — this is it). The
**Retained(T4) eager/replicated override** stays **coupled to the F7 durability floor** but is a
**named, unbuilt reserved hook** in M12.3 (see D3) — no eager-replicate path is built this arc.

- Honest caveat (recorded at the lock): lazy carries the D1 fetch-protocol complexity; **eager**
  would reuse the existing push machinery and is the materially smaller build (no fetch protocol,
  no miss signal) at the cost of every-home-stores-every-blob. The lazy lock takes the F3
  storage-efficiency posture deliberately, accepting the fetch-protocol build.

### M12.3-D3 — Retained(T4) durability floor = reserve-the-hook, no Retention read (FK-4; the M12.3↔M12.4 boundary) — **load-bearing**

**Locked: α — reserve-the-hook.** M12.3 builds the lazy fetch + the unavailable signal; the
Retained eager-replicate override is a **named hook (mechanism reserved), with NO `Retention`
read in M12.3.** The first production `Retention` reader stays at **M12.4** per M12-D10.

This resolves the surfaced M12-D8↔M12-D10 tension (M12-D8 couples a Retained eager override
"here"; M12-D10 says M12.4 owns the first reader) **in favour of M12-D10**: mechanism-first, no
erasure/retention enforcement pulled forward into the federation arc, honouring F7's "mark +
reserve the hook, not build-the-vault." Grounding holds: **zero** production `Retention` readers
on `main` (J-380 finding re-confirmed by the audit + Chat). The protocol-layer "Retained =
ciphertext durability floor + erasure refusal" (M12-D6) is unchanged; M12.3 simply does not
*read* `Retention` to drive replication.

### M12.3-D4 — Held-pending / unavailable client signal = synchronous-first (FK-3; item 4)

**Locked: α — synchronous.** On a federated read-miss, B blocks the client's fetch during the
federation round-trip; on timeout-or-genuine-absence it serves the typed `10003 blob_unavailable`
(D5). **No new client wire**; the typed `10003` is the signal.

- Grounding (audit M12.3-A-04): `PendingBuffer` is **event-shaped** (keyed by `EventXgid`,
  node-side DAG admission, fixed timeouts). A blob-miss is a different unit (`blob_ref`) with a
  different waiter (the client). M12-D8 names `PendingBuffer` "the model to extend," but it would
  **mirror, not extend** it, and α is materially simpler for M12.3's scope.
- **Reserved (not built):** β — asynchronous held-pending (B replies "retry"; client
  re-requests; a new client wire mirroring `PendingBuffer`). Named as the seam if/when
  long-latency federated fetches make blocking the client untenable. Mechanism-first, the M12.2a
  "flat-now, Pattern-A-reserved" precedent.

### M12.3-D5 — `10003 blob_unavailable` = type the variant (FK-5; M12-D9) — *a confirm*

Add a typed `BlobError::Unavailable` arm → `to_wire_code` maps it to `Some((10003,
"blob_unavailable"))`; replace the **defensive literal** at app.rs:1919 with the typed path. This
is a **variant-add to the existing `BlobError`** (already the net-new parallel transfer/ingest
error type per M12-D9 — **not** `ExchangeError`), not a new type. First typed emission =
the federated-read miss. RC-F-01 re-grep at build (Chat + audit confirm **10003 free**, domain-10
used only by the blob band — `blob_store.rs` reserves it, no emitter today).

### M12.3-D6 — Sub-arc shape + witness surface (FK-6; M12-D10)

**Locked: single arc** (M12.3 is one coherent mechanism — fetch transport + typed reject +
sync miss-signal). The runbook may still split into ordered commits (spine-first), but no
M12.3a/b milestone split. **Witness posture** (runbook firms the exact set): a **box-gated
real-binary two-home federation witness** for the headline (sibling to the `m12_2a_self_thread_e2e`
path; the F2/F9 harness-control `add-peer`/`initiate` seam already drives federation) **+ an
in-suite in-process witness** for the spine — mirrors the M12.2a in-suite-spine + box-gated-e2e
split.

---

## §4 Witnesses (M12.3; RED-on-revert; runbook firms the set)

- **W1 — federated round-trip (headline).** Homes A + B federated, sharing a Space. A member on
  A authors a `message.file` → the descriptor federates to B (existing eager push) → a member on
  B reads the event + fetches the blob → B miss → B federation-fetches from A → serves
  **byte-identical**. RED-on-revert: neuter the federation fetch arm → B miss → `10003`.
- **W2 — typed unavailable.** A genuine miss (blob nowhere reachable) surfaces the **typed**
  `BlobError::Unavailable` → wire `10003` (replaces the defensive literal). RED-on-revert: the
  pre-typing literal path is the baseline.
- **W3 — content-blind across homes.** The bytes B fetches + stores are ciphertext (M12-D5
  inherited); B is content-blind by construction (inherits M12.1 W2).
- **W4 — never-leak.** A third home **C** not sharing the Space / federation never receives or
  serves the blob (federation-scoped; `federation_nodes` / policy gates hold).
- **W5 — self stays federation-free.** A self-thread blob still never federates (M11/D-021
  intact) — M12.3's federation surface is **additive**, not a weakening of the self path.

---

## §5 Out of scope / reserved (do NOT pull in)

- **M12.4 erasure** — the `message.redact` content applier (F2a, net-new), the F2b sender
  `Retention` read (M12's **first** production `Retention` reader), crypto-shred destroy-to-erase
  (D3-gated), the reserved WORM/legal-hold operator/module hook.
- **`Retention` enforcement / reading** — M12.3 reserves the Retained eager-replicate hook
  (D3); it does not read or enforce `Retention`.
- **Eager / replicated blob federation** — reserved (D2 lazy-default); the Retained-eager
  override is the named hook, unbuilt.
- **Async held-pending client signal** (D4 β) — reserved unless Joe locks it.
- **Pattern-A tier→size / lifetime tables** (F6/F8) — reserved (mechanism-first).
- **The WORM/archival vault** — operator/module responsibility (F7); M12.3 reserves the hook
  only.

---

## §6 Sequence (Rule 0)

this design → Clair authors `tasks/M12_3_*_IMPL.md` (the M12.3 runbook, §3 scope, spine-first)
→ implement → Chat doc-bridge → M12.3 close → **M12.4** (erasure) → M12 close → Round-2 final
pre-UI gate → UI → Streams. No code until the runbook lands + Joe-locks the runbook values.

## §7 Entry (Rule 0)

`CLAUDE.md` PLAY → `JOURNAL.md` J-386 → this design → `tasks/M12_3_FEDERATION_FETCH_PHASE0_AUDIT.md`
(findings + forks) → `tasks/M12_ATTACHMENTS_DESIGN.md` (M12-D8/D9/D10) →
`tasks/M12_1_BLOB_STORE_ATTACH_IMPL.md` (the built primitives) → `docs/ROADMAP.md` (M12).
