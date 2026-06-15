# M12 — Attachments: Design (Joe-LOCKED)
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-15  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Purpose & status

The Joe-LOCKED M12 design, authored by Chat at the design-lock (J-381) after discussing the
five audit-teed design inputs and locking each by-recomms. Sits on the M12 Phase-0 audit
(`tasks/M12_ATTACHMENTS_PHASE0_AUDIT.md`, GO, findings M12-A-01..09) and the framing brief
(`tasks/M12_ATTACHMENTS_PHASE0_BRIEF.md` v1.1, forks F1–F9).

D-071 arc discipline: this design → Clair authors the M12.1 runbook → implement → Chat
doc-bridge per arc → close. **No code precedes the runbook; the runbook does not precede this
lock.** Decisions are arc-local (D-069) unless explicitly marked a DECISIONS.md promotion
candidate (M12-D6 is).

---

## §2 Locked decisions (M12-D1..D10)

### M12-D1 — Byte-transfer = chunked base64 over WS (the gate; M12-A-01; channel corrected J-382/R-2)

The client↔node channel is **WebSocket** (`home_node` is a `ws://` URL; sends ride
`Connection::send_event_confirmed`); WS frames carry JSON/text, so raw file bytes can't ride
them. **Locked: chunked base64** — a net-new transfer sub-protocol carried as **WS
`TransportMessage` variants** (begin/chunk/end + fetch, routed at the node WS message loop): the
byte stream (ciphertext, per M12-D5) is base64-chunked into bounded frames, reassembled on the
far side. Symmetric for upload (client→home) and fetch (home→client).

**Channel correction (J-382, R-2).** The audit (M12-A-01) and the original lock framed transfer
as "over the pipe / inside the text-line model." Grounding on `main` (Chat-verified) showed the
client reaches the node **only over WebSocket** — there is no client→node pipe; the named pipe is
the control-mode↔resident **driver** channel (the `--attach` CLI invocation path, not the byte
path). **The chunked-base64 *encoding* lock is unchanged** (WS frames are JSON/text too — raw
bytes still can't ride without base64); only the framing surface moved from pipe text-lines to WS
variants. The binary-frame rejection still holds (transfer stays inside the uniform JSON
`TransportMessage` model, not a parallel binary frame type).

- Rejected: **length-prefixed binary frame** — breaks the UTF-8-line invariant both pipe
  surfaces depend on (`read_line`→String); too invasive on a previously clean channel.
- Rejected as the standing shape: **single base64 JSONL field** — a multi-MB single line
  stresses `read_line` + whole-file-in-memory; viable only as an M12.1-only shortcut, but that
  is a reshape later (against the F1 "no later reshape" spirit), so we take chunked from the
  start.
- Bounded per-line size keeps `read_line` healthy and admits progress/backpressure for the F6
  MB-scale ceiling. Chunk size is a runbook constant (a Phase-0 value, not a lock).

### M12-D2 — Message seam = reuse `message.file`; descriptor as content (M12-A-02 / F5)

Reuse the already-validation-wired `EventType::MessageFile` (`"message.file"`); **no new event
kind** (this is the F5 answer — `file.upload` / `message.attachment-meta` are doc-only;
`stream.*` / `media.*` stay a clean doc reservation). Build the missing
`build_message_file_event` (twin of `build_message_text_event`) whose free-form JSON content
carries `attachments: [Descriptor]` (F1 plural list = a content-schema convention, no wire-type
change). A `message.file` event rides the DAG + fanout exactly as `message.text` (no
`SpaceState` apply arm); only the bytes diverge to the blob store.

### M12-D3 — `Descriptor` schema (net-new)

A net-new struct carried in `message.file` content, **inside** the `enc:` E2E envelope (so all
of it is encrypted at rest/in transit) — **end-state**; in M12.1 the descriptor rides as
**plaintext** `message.file` content (matching `message.text` today), the `enc:`-wrap activating
at the shared D3 cutover per M12-D5's maturity boundary (J-382/R-1):

- `blob_ref` — the **ciphertext** hash = the content-address / blob-store key (the node sees
  and addresses only ciphertext, M12-D5/D6).
- `plaintext_hash` — integrity check **after** client-side decrypt.
- `key` — the per-blob symmetric key (M12-D5); meaningful only to the group that can read the
  enclosing `enc:` content.
- `filename`, `mime`, `size` — by-value metadata.

Multi-file = a list of these. Exact field encoding (the key wrapping shape, hash algo naming)
is a runbook detail bound to the M12-D5 envelope; the *shape* above is locked.

### M12-D4 — Blob store = content-addressed `blobs_dir` sibling (M12-A-03)

Net-new content-addressed store (the `EventStore` is event-only; no blob table exists). A new
`blobs_dir` extends `PathsSection` as a sibling of `spaces_dir`, default `<data_dir>/blobs`,
so events + their referenced blobs back up / snapshot / tier **as one unit** (the T4
requirement). Hash-keyed put/get/contains keyed by `blob_ref` (the ciphertext hash). The store
is **content-blind** — it only ever holds ciphertext.

### M12-D5 — Blob encryption = same Arc-H envelope as text (M12-A-01 input 4a)

**Locked: same-as-text.** Per-blob flow mirrors the AH-D1 `enc:`/CK envelope one level out: the
client generates a **fresh per-blob key**, encrypts the bytes, uploads the **ciphertext** (the
node stays content-blind and content-addresses by the ciphertext hash); the per-blob key +
plaintext hash travel in the `Descriptor`, which itself rides inside the `enc:` E2E message
content. The same group (or, for `self`, the user's own devices) that reads the text reads the
blob — **no new key distribution**.

**Flag 1 (inherited maturity, not a weakness):** blob encryption inherits the text path's
crypto maturity exactly — Arc-H's per-message-CK envelope + crypto-shred are demonstration-grade
today; real RFC 9420 HPKE wrapping + the destroy-to-erase storage op are D3-fenced (the M8.7 S+L
production-openmls arc). M12 builds blob encryption in the **same Arc-H shape**: interface now,
production crypto when the text path's D3 work lands. Not stronger, not weaker, than the text
path it mirrors.

**M12.1 maturity boundary (J-382, R-1).** Grounding on `main` (Chat-verified): the text send
path (`ops::send`) ships **plaintext** content today — client-side `enc:` live-encryption is
D3-fenced (the client holds no epoch key). So wrapping the blob `Descriptor` in `enc:` *now*
would make attachments **stronger than text**, violating this decision. Therefore in M12.1
per-blob **byte encryption is real** (ciphertext at rest), but the `Descriptor` — **including the
per-blob key** — rides as **plaintext `message.file` content**, exactly matching `message.text`.
The `enc:`-wrap of the descriptor activates for text *and* blob descriptors **together** at the
shared D3/M8.7 cutover (zero blob-store rework then). The honest **W2** claim is therefore
*"ciphertext-at-rest + store content-blind-by-construction"* — **not** *"the node can't get the
key"* (today the key is in the plaintext descriptor, exactly as text content is plaintext); both
go node-blind together at D3.

### M12-D6 — E2E philosophy = universal at the protocol layer; T4 retain-and-produce reserved to operator/module (input 4b) — **DECISIONS.md PROMOTION CANDIDATE**

The crypto-shred-vs-WORM fork resolved **universal E2E at the protocol layer**: every tier is
crypto-shreddable; **there is no protocol-level escrow key**. Grounded (D-065, J-381): the
Arc-H *text* path is already universal-no-escrow — zero escrow/tier/recovery hits in
`xgen-core/src/encryption/`, and the defended invariant `erasing_wrapped_key_defeats_epoch_holder`
(AH-D1 constraint 2) makes the per-message CK random and never epoch-derived, so destroying the
wrapped key is permanent **even for the epoch holder**. Blobs inherit that posture, keeping text
and attachments symmetric and the node content-blind.

The T4 **retain-and-produce** capability (F7/F8 WORM / legal-hold) is therefore **NOT** a
protocol escrow; it is **reserved to the operator/module layer** — the same hook F7 already
reserves for the WORM/archival backend. If an accountable deployment must produce retained
content, that escrow lives at *its* tier (an institution forking the reference module +
supplying its accountable backend), consistent with institutional-independence and "mark +
reserve the hook, don't build the vault."

**Ripple resolved across F2/F7/F8 in one move:**
- **F2 (erasure):** crypto-shred is a *real* protocol guarantee everywhere — destroy the
  per-blob key → every replica (including unreachable federated homes) is permanently unreadable.
- **F7/F8 (retention/WORM):** at the protocol layer, "Retained (T4)" = a **durability floor on
  the ciphertext bytes** (don't drop them) + an erasure refusal; *producibility* of plaintext is
  an operator/module-tier concern, not a base-protocol key.

**Promotion note:** M12-D6 is principle-shaped (a protocol-wide E2E invariant reinforcing AH-D1
+ D-088), above the arc-local bar. Recorded here as the lock; **flagged for Joe's explicit
DECISIONS.md promotion** (per the established "no DECISIONS change unless Joe promotes" pattern —
not auto-promoted).

### M12-D7 — F9 data-root posture: adopt the convention, decouple from M12.1 (M12-A-03)

Adopt F9's posture (data root **defaults outside** the install/system folder, operator-overridable
to any absolute path/volume, startup-validated: durable, writable, not tmp) — but **decouple it
from the M12.1 slice**. Today `data_dir = exe_dir()` with no override flag; the full shift touches
*every* node file + `--instance` segregation, not just blobs. So:

- **M12.1** uses `blobs_dir` as a `PathsSection` sibling under **today's** `data_dir` (no posture
  shift needed for the self-thread witness).
- The full **default-outside-install + `--data-dir` override + startup validation** lands at
  **M12.2** (or a named node-config sub-step) as a deliberate node-ops convention change, not an
  M12.1 blocker.

### M12-D8 — F3 federation lean: lazy-lean provisional, lock deferred to M12.3

F3 stays **lazy-lean, audit-grounded-not-locked**; the lock is formally **deferred to the M12.3
federation grounding** (M12.1/M12.2 never touch federation). Lazy blob federation is net-new
(push is eager today; no fetch-by-hash exists). The **Retained (T4) eager/replicated override**
stays coupled to the F7 durability floor (can't legal-hold a single droppable copy). The
`HeldPending` / `PendingBuffer` shape is the model to extend for the lazy-miss / unavailable
**client signal** (the carry-over held-pending UX seam).

### M12-D9 — Blob rejects = a new parallel error type (M12-A-09)

Lock the **principle**: blob/transfer rejects (**blob-too-large** F6, **blob-unavailable** F3
lazy miss, **hash-mismatch** content-address integrity) live in a **net-new parallel error
type at the transfer/ingest gate**, NOT `ExchangeError` (which gates the signed-envelope path;
the small descriptor event still rides it). `StoreError` is event-store-internal, also not the
home. The **code band is picked at build time**, grounded against the existing register (the
RC-F-01 / M10.1 wire-code-collision discipline).

### M12-D10 — Sub-arc split (F4) confirmed

- **M12.1** — local blob store (`blobs_dir` sibling, M12-D4) + `Descriptor` (M12-D3) +
  `build_message_file_event` (M12-D2) + chunked-base64 pipe transfer (M12-D1) + same-Arc-H
  blob encryption (M12-D5) + `--attach` into the **`self` thread** = headline witness
  (intra-home, multi-device, **never federation** → M11/D-021 intact).
- **M12.2** — `--attach` surface polish + the 4 D-092 arms + the F6 blob size gate at
  transfer/ingest + the full F9 data-root posture shift (M12-D7).
- **M12.3** — federation: fetch-blob-by-hash protocol + the F3 lazy/eager lock (M12-D8) + the
  Retained durability floor (F7) + the held-pending/unavailable client signal.
- **M12.4** — erasure: build the `message.redact` content applier (F2a, net-new — none today) +
  the F2b sender-`Retention` read (M12's first production reader of the dormant AI-D8
  enforcement; T4/`Retained` refuses) + crypto-shred destroy-to-erase (D3-gated) + the reserved
  WORM/legal-hold operator/module hook (mark + reserve, not build).

---

## §3 The M12.1 slice (shovel-ready; the runbook's scope)

M12.1 composes four net-new but small, self-contained pieces; it is the next runbook. End-to-end
witness path:

1. Client `--attach <path>` into the `self` thread (`ops::self_open` opens/creates the
   `"self"`-labelled self-DM, M11).
2. Client generates a per-blob key, encrypts the bytes (M12-D5), computes `blob_ref` (ciphertext
   hash) + `plaintext_hash`.
3. Client uploads the ciphertext to the home node's blob store over the chunked-base64 transfer
   (M12-D1); node stores content-blind under `blob_ref` in `blobs_dir` (M12-D4).
4. Client sends a `message.file` event whose `enc:` content carries
   `attachments: [Descriptor{blob_ref, plaintext_hash, key, filename, mime, size}]` (M12-D2/D3)
   into the self-DM.
5. A second client authenticated as the **same identity** syncs the event, reads the descriptor
   from the decrypted content, fetches the ciphertext by `blob_ref` over the transfer, decrypts
   with `key`, verifies `plaintext_hash`.

Entirely intra-home; **never federation** (M12-A-06) → M11/D-021 intact. `--attach` threads
through `ops::send` once and inherits all four D-092 arms (M12-A-08).

## §4 Witnesses (M12.1; RED-on-revert; runbook firms the exact set)

- **W1** round-trip: a file attached in the self thread is fetched back byte-identical by a
  second same-identity client (the headline).
- **W2** content-blindness: the bytes at rest in `blobs_dir` are ciphertext — the plaintext
  never appears in the store (revert the per-blob encryption → plaintext on disk → RED).
- **W3** content-address integrity: a corrupted/substituted blob fails the `plaintext_hash` (or
  `blob_ref`) check (hash-mismatch reject, M12-D9).
- **W4** transfer fidelity: the chunked-base64 round-trip reassembles a multi-chunk payload
  exactly (revert the reassembly → RED).
- **W5** never-federates: the self-thread attachment path touches no federation surface
  (`DmFederationNotAllowed` wall intact).

## §5 Out / routed (survive beyond M12 or this design)

- **Pattern-B "module-as-policy-bearer"** — hard limits as `ModulePolicy` switch-bag entries;
  ROADMAP horizon line, not invented in M12 (M12 keeps F6/F8/F9 on Pattern A).
- **WORM/archival backend** — operator/module responsibility; M12.4 reserves the hook only.
- **Carry-over UX** — federation-derived held-pending / unavailable client signal (surfaces at
  M12.3, M12-D8); federation-under-load stress measurement (no scheduled home).
- The precise blob reject **code band** (picked at the M12.1/M12.2 build, M12-D9).

## §6 Sequence (Rule 0)

this design → Clair authors `tasks/M12_1_*_IMPL.md` (the M12.1 runbook, §3 scope) → implement →
Chat doc-bridge → M12.1 close → M12.2 → M12.3 → M12.4 → M12 close → Round-2 final pre-UI gate →
UI → Streams. No code until the runbook lands.

## §7 Entry (Rule 0)

`CLAUDE.md` PLAY → `JOURNAL.md` J-381 → this design → `tasks/M12_ATTACHMENTS_PHASE0_AUDIT.md`
(findings) → `tasks/M12_ATTACHMENTS_PHASE0_BRIEF.md` v1.1 (forks F1–F9) → `docs/ROADMAP.md`
(M12) → `ch3 §3.1.1` + `DECISIONS.md` D-021 / D-056 / D-088 / AH-D1 (context).
