# M12.1 — Blob store + attachments: Implementation runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-15  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Purpose & status

The Clair-authored M12.1 implementation runbook, the design §3 scope. Executes the Joe-LOCKED
M12 design (`tasks/M12_ATTACHMENTS_DESIGN.md` v1.0, M12-D1..D10) on the M12 Phase-0 audit
(`tasks/M12_ATTACHMENTS_PHASE0_AUDIT.md`, M12-A-01..09).

**M12.1 = the four net-new, self-contained, federation-free pieces** (M12-A-01/02/03/08): a
content-addressed blob store (`blobs_dir` sibling) + a `Descriptor` + `build_message_file_event`
+ a chunked-base64 byte transfer + per-blob encryption + `--attach` into the **`self` thread** as
the headline witness. Intra-home, multi-device, **never federation** → M11/D-021 intact.

D-071 arc discipline: **this runbook → Joe-lock → implement → Chat doc-bridge → M12.1 close.**
**No code precedes Joe's lock of this runbook.** Two findings (§3) re-ground two locked-decision
framings against `main` and need Joe's confirm before the build sequence (§5) is final; the
runbook-level values (§4) need Joe's lock too. Decisions are arc-local (D-069).

**Grounded against `main` @ `5e96ad7`** (tree clean). Every seam below was re-confirmed to
file:line by reading production code this session (D-078 — anchors re-confirmed, not trusted
blind; the audit's line numbers all held except where noted).

---

## §2 Grounding ledger (seams re-confirmed to file:line)

| # | Seam | Location (main @ `5e96ad7`) | M12.1 action |
|---|---|---|---|
| G1 | `PathsSection` (`keypair_path`, `spaces_dir: Option<String>`) | `xgen-node/src/app.rs:135-139` | add `blobs_dir: Option<String>` |
| G2 | `spaces_dir` resolution (`config.paths.spaces_dir … unwrap_or_else(\|\| data_dir.join("spaces"))`) | `xgen-node/src/app.rs:730-735` | mirror: `blobs_dir` default `data_dir.join("blobs")` |
| G3 | `data_dir = exe_dir()` / `exe_dir()/instances/<label>` (no override flag) | `xgen-node/src/main.rs:173-188` | M12.1 stays under today's `data_dir` (M12-D7 — **no** F9 posture shift) |
| G4 | `EventStore` trait = event-only (`append`/`get`/`range`/`contains`/`len`) | `xgen-core/src/dag/store.rs:77-98` | blob store is **net-new + separate** (EventStore untouched) |
| G5 | `EventType::MessageFile` → `"message.file"`; validation-wired identically to `MessageText` | `xgen-common/src/wire.rs:34,171,265`; `xgen-core/src/message/exchange.rs:789-792,843-846` | reuse it (M12-D2/F5); **no new event kind** |
| G6 | `build_message_text_event` (content `json!({"text": text})`); `Event::new(EventType, IdentityXgid, RoomXgid, SpaceXgid, Vec<EventXgid> prev, ts, Value content)` | `xgen-core/src/message/exchange.rs:922-945` (content `:943`) | twin `build_message_file_event` (content `json!({"attachments":[Descriptor]})`) |
| G7 | `ops::send` = the canonical core; builds the text event with **plaintext** content; `get_dag_tips` for `prev_events`; `conn.send_event_confirmed` | `xgen-client/src/ops.rs:1617-1681` | thread `--attach` through here once (M12-A-08); **see Finding R-1** |
| G8 | `ops::self_open` opens/creates the `"self"`-labelled KnownSpace → returns `space_id`+`room_id` | `xgen-client/src/ops.rs:870-908` | witness target |
| G9 | `SendArgs { space, room, text }` (no attach field) | `xgen-client/src/app.rs:646` | add `attach: Option<String>` |
| G10 | `encrypt_message_envelope` / `decrypt_message_envelope` (enc: v2; per-message random `CK`; ChaCha20Poly1305, D-052) | `xgen-core/src/encryption/client_mls.rs:265,306` | blob crypto uses the **same ChaCha20Poly1305 primitive** in a sibling module; per-blob key is **fresh, not epoch-wrapped** (rides the descriptor) — **see Finding R-1** |
| G11 | hash scheme `hash_uri(bytes) → "xgen://hash/sha256:<hex>"` | `xgen-core/src/crypto/hashing.rs:15` | `blob_ref = hash_uri(ciphertext)`, `plaintext_hash = hash_uri(plaintext)` — matches `event_id` (V2) |
| G12 | `ExchangeError` + `to_wire_code` (register: 3041/3042/3043/3046, 6009) | `xgen-core/src/message/exchange.rs:58,130` | blob rejects are a **new parallel type**, NOT `ExchangeError` (M12-D9; V4) |
| G13 | `TransportMessage` enum (Challenge/Auth/AuthOk/Error{event_id}/Goodbye/SyncRequest/SyncComplete/RateLimit/InviteBootstrapRequest/EventAccepted) | `xgen-core/src/wire/types.rs:45-99` | add blob-transfer variants here (the WS channel — **see Finding R-2**) |
| G14 | `Connection<S>` = WS only; `send_event`/`send_event_confirmed`/`recv()`; `Inbound` enum | `xgen-core/src/transport/connection.rs:58-99,162,186` | client↔node transfer rides here (WS) |
| G15 | client connects to node **only** via `connect_url(node)` → `ws://`; `home_node` is a `ws://` URL; no client→node pipe | `xgen-client/src/session.rs:122-160` | confirms **Finding R-2** |
| G16 | node WS post-auth dispatch loop (`Inbound::Transport(TransportMessage::SyncRequest…)` :1657, `InviteBootstrapRequest` :1704, catch-all `Inbound::Transport(_) => {}` :1742, `Inbound::Event → process_inbound` :1744-1751) | `xgen-node/src/app.rs:1592-1751` | route new blob variants here |
| G17 | frame layer (`encode_frame`/`decode_frame`; `MAX_PAYLOAD_BYTES = 256*1024`) | `xgen-core/src/wire/framing.rs:21,45,60` | chunk under the 256 KB frame ceiling (V1) |
| G18 | D-092 four arms for `Send`/`SelfThread`: CLI (`app.rs:976`) · run-path (`app.rs:2654` `ops::send`) · batch (`batch.rs:554` `ops::send`) · aicontrol (`reconstruct_argv` → clap) | `xgen-client/src/{app.rs,batch.rs,aicontrol.rs}` | `--attach` via `ops::send` once → inherits all four |
| G19 | blob store today: **zero hits** for `blobs_dir`/`BlobStore`/`blob_store` across `xgen-node/src` + `xgen-core/src` | (grep, net-new confirmed) | — |
| G20 | named pipe = driver↔process control (`pipe.rs` `__PING__`/`__HEALTH__`/`__STOP__`/`__RELOAD_CONFIG__`/`__BATCH__`; `aicontrol.rs` Command/Reply JSONL) — **not** client↔node | `xgen-node/src/pipe.rs:838-959`; `xgen-{node,client}/src/aicontrol.rs` | carries the `--attach` **path** + control, **not** the bytes — **Finding R-2** |

---

## §3 Findings requiring Joe's lock (D-065 — surfaced, not papered over)

Two grounding findings. **Neither overturns a locked M12-D# *decision*; each corrects an
imprecise *grounding/framing* and sharpens exactly what M12.1 builds + witnesses.** Both are
exactly the kind of thing D-078 grounding is for. Recommendations given; Joe's confirm gates §5.

### Finding R-1 — the crypto-maturity seam (M12-D5 flag 1 made concrete)

**What the design says.** M12-D3 / §3 step 4: the `Descriptor` (which carries the per-blob
`key`) rides **inside the `enc:` E2E content**. M12-D5 flag 1: blob encryption inherits the text
path's crypto maturity **exactly** — "interface now, production crypto when the text path's D3
work lands; **not stronger, not weaker, than the text path**."

**What grounding shows (G7/G10).** `ops::send` builds `message.text` with **plaintext** content
(`json!({"text": text})`, exchange.rs:943); it does **not** call `encrypt_message_envelope`.
Client-side `enc:` live-encryption is D3-fenced (Arc-H C1 Finding 1 — the client holds no epoch
key / MLS group state; that lifecycle rides the eventual production MLS client). So **today the
text body sits in plaintext in the DAG.**

**The consequence for M12.1.** "Inside the `enc:` content" cannot be honoured for the descriptor
in M12.1 **without making attachments stronger than text** (pulling forward the D3 client-encrypt
that `message.text` does not have) — which **violates M12-D5's "not stronger/weaker than text."**

**Recommendation (faithful to M12-D5 flag 1):**
- **Per-blob ENCRYPTION is real M12.1 work** — fresh per-blob key → ChaCha20Poly1305 → ciphertext
  → blob store. This is the W2 ciphertext-at-rest witness; the blob store is content-blind by
  construction (it only ever receives + holds ciphertext, keyed by the ciphertext hash).
- **The `Descriptor` (incl. the per-blob `key`) rides as PLAINTEXT `message.file` content** —
  exactly matching `message.text`'s plaintext body today. The M12-D3/§3 "inside the `enc:`
  envelope" is the **interface shape** that activates with the **shared text-path D3 enc: work**;
  M12.1 does not pull it forward (file == text maturity).
- **Sharpen W2's claim accordingly:** W2 witnesses (a) ciphertext bytes at rest in `blobs_dir` and
  (b) the blob store is content-blind-by-construction. It does **NOT** yet witness
  "node-cannot-reconstruct-the-key" — that is the same D3 maturity as text's plaintext body.
  State this boundary honestly at the witness + at close.

**This is faithful to the lock (M12-D5 flag 1 anticipates exactly this), not a falsification.**
**Lock-ask R-1:** confirm the maturity boundary (blob ciphertext real; descriptor plaintext at
text-maturity; enc:-wrap deferred to the shared text D3) + the sharpened W2 claim.

### Finding R-2 — the transfer channel is WebSocket, not the named pipe (M12-D1 encoding lock stands)

**What the design/audit say.** M12-A-01 + M12-D1 ground the byte transfer on the **named-pipe**
surfaces (`pipe.rs` `__BATCH__` + `aicontrol.rs` JSONL); M12-A-01: "intra-home multi-device
sharing still moves the bytes client→home-node over **this same pipe**." M12-D1 locks **chunked
base64** ("inside the text-line model"; rejecting a length-prefixed binary frame + single-field
base64).

**What grounding shows (G14/G15/G20).** The client reaches the node **only over WebSocket**:
`SessionState::ensure_connected` → `connect_url(node)` → `ws://…` → `client_authenticate`
(session.rs:122-160); `home_node` is a `ws://` URL; events ride `conn.send_event` /
`send_event_confirmed` (connection.rs:162/186). **There is no client→node pipe path.** The named
pipe (`pipe.rs` / `aicontrol.rs`) is a **driver↔process local control channel** — a script/AI
driving the client process, and a node-local admin channel — **never client↔node.**

**The consequence for M12.1.** The blob bytes must reach the node's blob store; the only
client→node channel is **WS**, where events already ride. So the byte transfer is a set of
**new `TransportMessage` variants on WS** (G13/G16), **not** a new pipe command. The audit's
"client→home-node over the pipe" is grounded-imprecise (corrected here per D-078).

**The M12-D1 *encoding* DECISION stands unchanged.** WS is also a framed JSON channel
(`encode_frame` wraps a `serde_json` payload; raw bytes cannot ride a JSON field without base64),
so "chunked base64, not a length-prefixed binary frame" applies to WS verbatim. Only the
**framing mechanism** changes from M12-D1's pipe-shaped sketch ("bounded lines + begin/size +
sentinel") to **WS-message-shaped** (begin/chunk/end + fetch variants). The encoding, the
chunk-bounding, and the symmetric upload/fetch are identical.

**The named pipe's role in M12.1** (G18/G20): it carries the `--attach <path>` argument + small
control (driver→client); the client reads the file from its **local filesystem** and writes a
fetched blob's decrypted plaintext to a **local path**. The pipe carries **no blob bytes** in
M12.1. (A future driver wanting bytes returned over the pipe is an M12.2 surface concern, bounded
base64 in a Command — not M12.1.)

**Lock-ask R-2:** confirm the transfer channel = **WS** (new `TransportMessage` variants), the
M12-D1 chunked-base64 encoding intact, the named pipe = path/control only. (If Joe instead wants a
pipe-borne transfer for a reason outside this grounding, §5 C3 reshapes to pipe-line framing — flag
at lock.)

---

## §4 Runbook-level values for Joe-lock

| Value | Recommendation | Grounding |
|---|---|---|
| **V1 chunk size (M12-D1)** | `BLOB_CHUNK_BYTES = 128 * 1024` (ciphertext bytes per chunk before base64; base64(128 KiB) ≈ 174 KB < 256 KB frame ceiling) | G17 (`MAX_PAYLOAD_BYTES = 256*1024`) |
| **V2 hash algo (`blob_ref` / `plaintext_hash`)** | reuse `xgen_core::crypto::hashing::hash_uri` (SHA-256, `xgen://hash/sha256:<hex>`); `blob_ref = hash_uri(ciphertext)`, `plaintext_hash = hash_uri(plaintext)` | G11 (matches `event_id` scheme) |
| **V3 `Descriptor` + per-blob key shape** | serde struct `Descriptor { blob_ref: String, plaintext_hash: String, key: String, filename: String, mime: String, size: u64 }`; `attachments: Vec<Descriptor>` in `message.file` content. Per-blob key = 32-byte ChaCha20Poly1305 key, base64 in `key`. Blob ciphertext layout `nonce(12) ‖ ChaCha20Poly1305(plaintext)`. New module `xgen-core/src/encryption/blob.rs`: `encrypt_blob(plaintext) -> (key:[u8;32], ciphertext:Vec<u8>)` / `decrypt_blob(key, ciphertext) -> Result<Vec<u8>>` (sibling to `client_mls.rs`, reuses D-052 ChaCha20Poly1305) | G6/G10/G11; bound to M12-D5 envelope (per R-1, `key` plaintext at text-maturity) |
| **V4 blob reject code band (M12-D9)** | new `BlobError` type (xgen-core, transfer/ingest gate; NOT `ExchangeError`/`StoreError`) with `to_wire_code`; **domain 10 = 10000–10999 (attachments/blobs)** — free per the register: `10001 blob_hash_mismatch` (M12.1, W3), `10002 blob_too_large` (F6 → M12.2), `10003 blob_unavailable` (F3 → M12.3). Re-grep the register at build (RC-F-01/M10.1 discipline) before emitting. | G12; CLAUDE.md "Error Code Convention" domain ranges (1xxx-9xxx allocated; "domain 10 = 10000–10999") |

---

## §5 Build sequence (spine-first; written for the recommended R-1/R-2 resolution)

Five code commits, then Chat's doc-bridge close. Per-commit DoD in §7. Each commit is its own
atomic commit (two-seat: I commit code; Chat authors the canonical-record doc-bridge). **Pending
Joe's lock of R-1, R-2, V1–V4** — if R-2 locks pipe-not-WS, C3 reshapes; the rest is unaffected.

- **C1 — blob store + reject type (the store spine).**
  `blobs_dir` on `PathsSection` (G1) + resolution (G2, default `data_dir.join("blobs")`); a
  content-addressed `BlobStore` (recommend xgen-node, node-local filesystem like the spaces dir —
  confirm-at-build) with `put(ciphertext) -> blob_ref` / `get(blob_ref) -> Option<bytes>` /
  `contains(blob_ref)`, content-blind, keyed by `blob_ref = hash_uri(ciphertext)` (V2); `BlobError`
  (V4) with `to_wire_code`. **Spines W2 (ciphertext-at-rest) + W3 (hash-mismatch reject).**
  Unit tests: put/get round-trip; content-address keying; `put` of bytes whose hash ≠ claimed ref
  → `10001`; `get` of a corrupted file → mismatch.

- **C2 — per-blob crypto + `Descriptor` + `build_message_file_event`.**
  `xgen-core/src/encryption/blob.rs` (V3 `encrypt_blob`/`decrypt_blob`, fresh key, ChaCha20Poly1305);
  `Descriptor` struct (V3); `build_message_file_event` twin of `build_message_text_event` (G6),
  content `json!({"attachments":[Descriptor…]})`, reusing `EventType::MessageFile` (G5).
  Unit tests: `encrypt_blob`→`decrypt_blob` round-trip + wrong-key fails; `blob_ref =
  hash_uri(ciphertext)` + `plaintext_hash = hash_uri(plaintext)`; descriptor serde round-trip;
  builder content shape + `event_type == MessageFile`.

- **C3 — chunked-base64 WS transfer (the long pole).** *(Pending R-2 = WS.)*
  New `TransportMessage` variants (G13): `BlobUploadBegin { blob_ref, size }` · `BlobChunk { seq,
  data_b64 }` · `BlobUploadEnd` · `BlobFetchRequest { blob_ref }` (node replies with
  `BlobChunk`* + an end marker). Client-side `Connection` helpers: `upload_blob(ciphertext)` /
  `fetch_blob(blob_ref) -> Vec<u8>`, chunked at `BLOB_CHUNK_BYTES` (V1), base64 per chunk, bounded
  under the 256 KB frame (G17). Node-side: route the new variants in the WS post-auth loop (G16,
  beside `SyncRequest`/`InviteBootstrapRequest`), reassemble → verify `blob_ref` (`10001` on
  mismatch) → `BlobStore::put`; serve fetch from the store (`10003` on miss — reserved; M12.1
  self-thread never misses). **Spines W4 (chunked round-trip fidelity).**
  Integration test (in-process node): multi-chunk upload→fetch reassembles byte-identical;
  ciphertext-only on the wire (the bytes are already ciphertext per M12-D5).

- **C4 — `--attach` on the surface + `ops::send`/fetch wiring.**
  `SendArgs.attach: Option<String>` (G9). In `ops::send` (G7), when `attach` is `Some`: read the
  local file → `encrypt_blob` → `upload_blob` over WS (C3) → build the `message.file` event with
  the `Descriptor` (C2) → `send_event_confirmed` (descriptor content plaintext per R-1). Threaded
  through `ops::send` once → inherits the four D-092 arms (G18). A fetch op (new `ops::fetch_*`, or
  extend the self/history read) for client 2: read the `message.file` event → extract the
  descriptor → `fetch_blob(blob_ref)` → `decrypt_blob(key, …)` → verify `plaintext_hash` (`10001`
  on mismatch) → write to a local path. Witness target = `--attach` into `self_open` (G8).
  Unit/seam tests: `SendArgs` parse; `ops::send` attach branch builds `message.file` not
  `message.text`; fetch verifies `plaintext_hash`.

- **C5 — the §6 witnesses (W1–W5) as integration tests.**
  Two same-identity clients + an in-process node; RED-on-revert recorded for the spine witnesses
  (W1/W2/W3/W4). W5 asserts no federation surface is touched.

**(Chat) doc-bridge + M12.1 close** — canonical-record flips (CLAUDE PLAY, JOURNAL, ROADMAP,
design/audit status), the M12-D5/R-1 maturity boundary recorded, the R-2 channel correction
recorded; M12-D6 stays a flagged DECISIONS promotion candidate (Joe's call, not this arc).

---

## §6 Witnesses (RED-on-revert)

- **W1 — round-trip (headline).** A file attached into the `self` thread by client 1 is fetched
  back **byte-identical** by client 2 (same identity). RED-on-revert: break reassembly (C3) or
  decryption (C2) → mismatch.
- **W2 — content-blindness (per R-1).** The bytes at rest in `blobs_dir` are **ciphertext**;
  plaintext never appears in the store; the store is content-blind by construction. RED-on-revert:
  store plaintext instead of ciphertext → plaintext on disk → RED. *Honest scope:* this witnesses
  ciphertext-at-rest + content-blind-by-construction; it does **not** witness
  node-cannot-reconstruct-the-key (same D3 maturity as text's plaintext body).
- **W3 — content-address integrity.** A corrupted/substituted blob fails the `blob_ref` /
  `plaintext_hash` check → `10001 blob_hash_mismatch`. RED-on-revert: skip the hash check →
  corruption undetected → RED.
- **W4 — transfer fidelity.** The chunked-base64 round-trip reassembles a **multi-chunk** payload
  (file > `BLOB_CHUNK_BYTES`) exactly. RED-on-revert: break chunk ordering/reassembly → RED.
- **W5 — never-federates.** The self-thread attachment path touches no federation surface
  (`DmFederationNotAllowed` wall intact; no `apply_federation_push` fires for the self-DM blob or
  event). M11/D-021 intact.

---

## §7 Definition of Done

**Per-commit gate (each of C1–C5):** `cargo build --workspace` 0-error;
`cargo clippy --workspace --all-targets --all-features -- -D warnings` clean;
`cargo test --workspace` green (baseline **1405/0** + the commit's new tests); for spine commits,
RED-on-revert recorded in the commit message.

**Milestone gate (at C5):** W1–W5 all green; RED-on-revert recorded for W1/W2/W3/W4; W5 asserts
no federation surface touched; the R-1 maturity boundary + R-2 channel correction stated in the
close. (Build target `CARGO_TARGET_DIR=C:/cargo-targets/XGenProtocol`.)

*(No "commit pushed" DoD line — unflippable inside its own commit; `Status: COMPLETED` is the
shipped signal. Joe pushes.)*

---

## §8 Out of scope (later sub-arcs — do NOT pull in)

- **M12.2** — `--attach` surface polish + the F6 blob-size gate at transfer/ingest (`10002`) + the
  full F9 default-outside-install + `--data-dir` override + startup validation (M12-D7).
- **M12.3** — federation fetch-blob-by-hash + the F3 lazy/eager lock (M12-D8) + the Retained
  durability floor + the held-pending/unavailable client signal (`10003`).
- **M12.4** — the `message.redact` content applier + the F2b sender-`Retention` read + crypto-shred
  destroy-to-erase (D3-gated) + the reserved WORM/legal-hold operator/module hook.
- **Shared-with-text D3** — client-side `enc:` live-encryption of `message.*` content (R-1); when
  it lands for text it lands for the `message.file` descriptor in the same shape.

---

## §9 Sequence + entry (Rule 0)

this runbook → **Joe locks R-1 + R-2 + V1–V4 + the §5 sequence** → implement C1→C5 (spine-first)
→ hand Joe the push → Chat doc-bridge → M12.1 close → M12.2.

**Entry (Rule 0):** `CLAUDE.md` PLAY → `JOURNAL.md` J-381 → `tasks/M12_ATTACHMENTS_DESIGN.md`
(§3/§4) → `tasks/M12_ATTACHMENTS_PHASE0_AUDIT.md` (M12-A-01/02/03/08) → this runbook →
`docs/ROADMAP.md` (M12).
