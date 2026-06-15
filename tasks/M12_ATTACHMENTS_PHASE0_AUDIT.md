# M12 — Attachments: D-071 Phase-0 Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-15  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Purpose & scope

The D-071 Phase-0 grounding audit for M12 (attachments). Grounds the nine forks F1–F9
(Joe-LOCKED at J-379) against `main` @ `fdbaa8d` to file:line, working the brief's nine-item
agenda. **Audit only** — surfaces gaps honestly (D-065), does not pre-empt the design locks.
Where a fork's grounding refines or shifts its provisional target, that is recorded and routed
to the design discussion, not silently adjusted.

Agenda map (brief §"Phase-0 audit agenda"): item 1 → M12-A-02 · item 2 → M12-A-03 · item 3 →
M12-A-01 (load-bearing) · item 4 → M12-A-06 · item 5 → M12-A-05 · item 6 → M12-A-04 · item 7 →
M12-A-07 · item 8 → M12-A-08 · item 9 → M12-A-09.

**Verdict: GO.** Every fork is grounded; the load-bearing unknown (pipe byte-transfer,
M12-A-01) is concretely characterized and gates the *transfer-mechanism choice*, not the
architecture. The minimal M12.1 self-thread slice depends only on four small, self-contained,
net-new pieces (M12-A-01/02/03/08) and never touches federation (M12-A-06) → M11/D-021 intact.
Two brief-framing refinements surface (see §4): the attachment kind is **`message.file`** (the
brief's `file.upload` / `message.attachment-meta` are doc-only, zero code hits), and F9's
"default outside the install folder" is a **genuine new convention**, not an extension (today's
data root *is* the install folder, with no override flag).

---

## §2 Findings (M12-A-##)

### M12-A-01 — LOAD-BEARING (agenda item 3 / pipe byte-transfer). The pipe is line-delimited UTF-8 text; no binary / length-prefixed / chunked transfer exists.

Both pipe surfaces are newline-delimited UTF-8 String I/O:

- Legacy `__BATCH__` control pipe (`xgen-node/src/pipe.rs`): `BufReader::read_line(&mut buf)`
  into a `String` (`pipe.rs:838`, `:904`); replies `writer_half.write_all(resp.as_bytes())`
  (`:859`, `:889`, `:959`). Client side reads the file line-by-line (`.lines()`,
  `pipe.rs:1030`) and writes `line.as_bytes()` (`:1054`) terminated by `__END__\n` (`:1059`).
- `--aicontrol` JSONL pipe (`xgen-node/src/aicontrol.rs`, `xgen-client/src/aicontrol.rs`):
  `BufReader::read_line` (`aicontrol.rs:504`) → one `Command` JSON object per line;
  `Reply::to_line()` written via `write_all(out.as_bytes())` (`:528`). Envelope is
  `serde_json` Command/Reply (`aicontrol.rs:192`, `:330`).

There is **no length-prefixed frame, no binary read (`read_exact`/`read_buf` on raw bytes), no
chunked/multi-frame assembly** anywhere in the pipe layer. `read_line` breaks on any embedded
`\n` and assumes UTF-8 — **raw file bytes cannot ride the existing pipe** (binary contains `\n`
and is not valid UTF-8).

**Consequence.** M12's `--attach` byte upload over the pipe (and the symmetric fetch) needs a
net-new transfer mechanism. This gates **even the federation-free M12.1 self-thread slice** —
intra-home multi-device sharing still moves the bytes client→home-node over this same pipe.
Three candidate shapes for design (not locked here):
1. **base64-in-JSONL field** — encode the file into a string field on an existing/new Command;
   simplest, no new framing, but ~33% overhead and whole-file-in-memory (bounded by F6's blob
   ceiling, so memory cost is the design's to bound);
2. **length-prefixed binary frame** — a net-new binary sub-protocol alongside the line protocol
   (a `__UPLOAD__ <len>\n<bytes>` shape); efficient, streamable, but new framing on a
   previously text-only channel;
3. **chunked base64** — multi-line base64 body with a sentinel; a middle path.

This is the design's first decision and the M12.1 long pole. *(D-056/D-043 = the pipe
deployment/naming context; the byte-transfer shape is net-new on top.)*

### M12-A-02 — The message seam is `message.file` (`MessageFile`), a declared, validation-wired, but UN-BUILT event kind.

`EventType::MessageFile` → wire `"message.file"` (`xgen-common/src/wire.rs:34`, `:171`, `:265`).
It is **fully wired into validation** identically to `message.text`: SendMessages permission +
room-membership-gated (`exchange.rs:790`, `:844` — the `event_room_permission` fold and the
`check_permission` match both treat `MessageText | MessageFile | MessageReaction |
MessageRedact` as a unit). But there is **no builder** — only `build_message_text_event`
(`exchange.rs:922`) exists; `build_message_file_event` is absent — and no content shape. Message
content is a free-form `serde_json::Value` (text builder: `json!({ "text": text })`,
`exchange.rs:943`).

The descriptor attaches here as event content: a `message.file` event whose content carries
**`attachments: [Descriptor]`** (F1's plural list = a content-schema convention, not a wire-type
change). The `Descriptor` struct (hash + filename + mime + size) is net-new — no existing
struct carries that tuple.

**Refinement to the brief (D-065).** The brief names `file.upload` and `message.attachment-meta`
as "possible existing descriptor kinds." Both return **zero code hits** across the workspace —
they are ch2 / Phase-9-survey doc breadcrumbs, not code. The real, already-validation-wired home
is `message.file`. Recommend the design **reuse `message.file`** (this is also the F5 namespace
answer — no new event kind needed).

**Ingest/apply seam:** a message event lands in the DAG via `ingest_event` (`runtime.rs:585`) →
`store.append(event)` (`runtime.rs:693`); the validated entry is `dispatch_event`
(`runtime.rs:1001`). Messages are **not** applied to `SpaceState` (no `MessageText`/`MessageFile`
apply arm) — they are DAG content delivered by fanout. So a `message.file` descriptor rides the
DAG + fanout exactly as `message.text` does today; only the bytes diverge (to the blob store).

### M12-A-03 — The node data-root convention exists and is extensible; the blob store is a clean `PathsSection` sibling. Event store is in-memory+JSON by default (SQLite is opt-in); EventStore is event-only.

The data-root convention is real and grounded:
- `data_dir` is the Tier-1 node root (D-025) — every node file is `data_dir.join("xgen-node_*")`
  (`admin_ops.rs:420`–`451`: identities.db, federation.json, auth_modules.json, etc.).
- `spaces_dir` defaults to `<data_dir>/spaces`, overridable via `config.paths.spaces_dir`
  (`app.rs:731`–`734`; `PathsSection`, `app.rs:136`).
- The **default event store is in-memory + JSON durability** under `spaces_dir`
  (`app.rs:753`–`754`, `:788`–`789`). **SQLite is an opt-in `[storage]` engine** (Storage-Engine
  milestone, SE-D3): selected via `config.storage` (`StorageSection`, `app.rs:281`); its per-Space
  DB dir comes from `[storage.<engine>].dir` (default `spaces_dir`, `app.rs:766`–`773`); the
  engine receives a fully-resolved `path` opaquely (`SqliteSettings.path`,
  `xgen-store-sqlite/src/lib.rs:46`, `:72`).
- `EventStore` trait (`xgen-core/src/dag/store.rs:77`: `append`/`get`/`range`/`contains`/`len`)
  is **event-only** — the SQLite impl is a single `events` table (`lib.rs:59`: seq, event_id,
  payload). **No blob/file table exists** → the content-addressed blob store is net-new.

**Blob-store placement.** The blob store extends `PathsSection` as a sibling of `spaces_dir`
— a new `blobs_dir`, default `<data_dir>/blobs` (the brief's `<data_root>/events.db +
<data_root>/blobs/` shape). Backs up / snapshots / tiers as a unit with the event log because
both hang off `data_dir`.

**F9 shift flagged (D-065).** Today `data_dir` resolves to **`exe_dir()`** — the install folder —
via `resolve_data_dir` (`xgen-node/src/main.rs:173`–`186`: `exe_dir()`, or
`exe_dir()/instances/<label>` under `--instance`); `NodeConfig::default()` also roots every path
at `exe_dir()` (`app.rs:317`, `:330`). **There is no `--data-dir` flag and no data-root override
today.** F9's "data root **defaults outside** the install/system folder, operator-overridable to
any absolute path/volume, startup-validated (durable, writable, not tmp)" is therefore a
**genuine new convention**, not a free extension of an existing override. Adopting it shifts the
default node-root posture (and interacts with `--instance` segregation). Honest design input,
not a blocker.

### M12-A-04 — Only a single flat 256 KB frame ceiling is enforced; the §3.1.1 per-tier table and Space `max_event_size` are unwired. Blob gate is a parallel transfer/ingest gate.

- Frame ceiling: `MAX_PAYLOAD_BYTES = 256 * 1024` (`xgen-core/src/wire/framing.rs:21`) — a
  **single flat constant, not tier-derived**. Enforced **reject-before-signature** at validation
  step 1: `if bytes.len() > MAX_PAYLOAD_BYTES` (`xgen-core/src/wire/validation.rs:54`, error at
  `:28`).
- §3.1.1 per-tier table (256/64/32/16/8 KB by tier): **no tier→size map anywhere in code** —
  spec-but-unwired.
- Space `max_event_size`: stored on `SpaceState` (`state.rs:191`), parsed from create content
  (`state.rs:289`, `:317`), defaulted to `None` at every other constructor (`state.rs:448`,
  `:564`; `algorithm.rs:419`). **Zero enforcement comparison** — greps of `exchange.rs`,
  `runtime.rs`, `wire/` for any `max_event_size` read return nothing. **Stored-but-not-enforced.**

**Confirms F6.** The bytes never ride the signed envelope, so the blob size gate is a **parallel
gate at transfer/ingest**, independent of the 256 KB envelope ceiling (the envelope still gates
the descriptor event, which is small). Pattern-A's tier-derived MB-scale ceiling + tighter-only
immutable Space override are both net-new (no tier→size table, no enforced `max_event_size` to
mirror). The design decides whether the blob gate stays fully independent or finally wires the
dormant §3.1.1 envelope enforcement alongside.

### M12-A-05 — Erasure is genuinely net-new; `message.redact` has no applier; zero production readers of retention. F2b would be the first reader of the dormant AI-D8 enforcement.

- `message.redact` (`MessageRedact`) is a **declared, validation-wired kind** (`exchange.rs:792`,
  `:846`) with **no content-erasure applier** — no `MessageRedact` apply arm, no `apply_redact`,
  no content removal/mutation in `state.rs`/`runtime.rs`. The only "tombstone" hits in code are
  **federation-relationship** tombstones (`federation/registry.rs:261`, `pending_queue.rs:48`),
  not content. So there is no existing content-erasure mechanic to inherit — F2 builds it.
- `ModulePolicy` / `Erasability` / `Retention{Erasable, Retained}` are defined
  (`xgen-common/src/trust_assertion.rs:156`–`193`) with accessors `module_policy()` /
  `set_module_policy()` / `module_kind()` / `set_module_kind()` (`:206`–`233`), carried under
  `claims.extra["module_policy"]` (forward-extensible, `erasability` its first member). But a
  grep for any **production reader** — `Retention::Retained`/`Erasable`, `.retention`,
  `.erasability`, `module_policy()` calls — across `xgen-core/src` + `xgen-node/src` (excluding
  tests) returns **nothing**. **Zero production readers** (M10.3 populated + witnessed these in
  tests only; enforcement was D3-gated). **F2b activation makes M12 the first production reader
  of the dormant AI-D8 enforcement**, exactly as J-379 anticipated.
- D-088 crypto-shred substrate (Arc H): the `enc:` v2 per-message wrapped-`CK` envelope exists in
  `xgen-core/src/encryption/client_mls.rs` (wrap/unwrap), but the **destroy-to-erase storage
  operation is fenced** (interface-only per D-088 §"Scope (honest)"; cascade
  `D-088 content-erasure → PG-05 real crypto → D3`).

**Posture (lean per brief).** F2a (tombstone/redaction-only, building the missing mechanic;
cross-federation erasure = request-not-guarantee, the honest D-065 boundary) with **F2b named**
(redaction reads the sender's `Retention`; T4/`Retained` refuses). Both net-new. M12.4 (erasure)
is correctly the last sub-arc — it sits on the most net-new ground.

### M12-A-06 — Federation push is eager; no fetch-by-reference / lazy path exists. The held-pending model is the lazy-miss UX seam.

- Push is **eager**: `apply_federation_push` fires at ingest sites (`app.rs:1777`, `:2183`,
  `:2350`); `compute_federation_delta_for_space` (`fanout.rs:605`) /
  `stream_federation_delta` (`federation_session.rs:84`) carry **signed events only**. Events
  propagate to federated peers on author/ingest.
- The **only** pull-shaped mechanism is `sync_request` for missing *predecessor events* via
  `HeldPending` + `PendingBuffer` (`exchange.rs:196`; `fanout.rs` Paths A/B/C confirm
  out-of-order events go `DispatchOutcome::HeldPending` and drain on arrival). **There is no
  fetch-blob-by-hash path** — nothing requests content-by-reference.

**Consequence for F3.** "Lazy" federation for blob content is net-new: it needs (a) a
fetch-blob-by-hash request/response protocol between nodes, and (b) a held-pending / unavailable
**client signal** for a blob miss. The existing `HeldPending`/`PendingBuffer` shape is the
natural model to extend for (b) — this is also where the brief's carry-over "lazy-fetch miss UX"
surfaces. The self-thread (M12.1) sidesteps all of this (intra-home, never federated). F3 stays
**lazy-lean / audit-grounded-not-locked** — decide at design after the M12.3 federation grounding;
the **Retained (T4) eager/replicated override** (F7 coupling) is also net-new (no durability-floor
mechanism today).

### M12-A-07 — No event/blob-store GC, TTL, or retention lifecycle exists. F7/F8 reclaim + WORM/legal-hold + tiering hook are all net-new.

The store is **append-only with no deletes** — the SQLite `append` derives the next seq from
`COUNT(*)` (`lib.rs:97`–`99`), which is only correct because nothing is ever removed. A
workspace grep for gc/reclaim/ttl/expire/prune/retention lifecycle over events or blobs hits
only unrelated subsystems: MLS KeyPackage expiry/single-use (`encryption/key_package.rs`),
federation reconnect scheduling (`federation/registry.rs`), bootstrap keepalive TTL
(`bootstrap_client.rs`). **None is an event/data-store lifecycle.**

**Consequence.** F7 (tier-retention-aware reclaim/GC: T1 reclaimable+erasable, T4
pinned/legal-held/undeletable WORM-shaped) + F8 (Pattern-A tier TTL, two modes: reclaim-deadline
vs retention-minimum/legal-hold) + the reserved **archive/offload tiering hook** are all net-new
surfaces. Per the brief, M12 **marks + reserves** the WORM/offload hook (operator/module
responsibility); it does not build the vault. The reserved hook is a fresh operator/module seam,
not an extension of anything.

### M12-A-08 — The client Send surface + 4-arm D-092 pattern are in place; `--attach` is net-new on the surface; the `self` verb is the M12.1 witness target.

- `SendArgs { space, room, text }` (`xgen-client/src/app.rs:646`) — **no attach field**;
  `--attach` is net-new on the surface. `ops::send` (`xgen-client/src/ops.rs:1617`) is the
  canonical core (one copy, no drift — MP-F1a awaits the node ack).
- The **4 D-092 dispatch arms** are confirmed present for `Send` and for the `self` verb
  (the sibling witness shape): CLI match arm (`app.rs:1021` Send / `:976` SelfThread), run-path
  `cmd_*` (`app.rs:2637` Send / `:2361` SelfThread), batch (`batch.rs:435` SelfThread + the
  cooperative `get_dag_tips` callers `ops::send`/`join`/`leave` at `batch.rs:121`/`:160`),
  aicontrol (`xgen-client/src/aicontrol.rs`). Any new `--attach` plumbing threads through
  `ops::send` once and inherits all four arms.
- `ops::self_open` (`ops.rs:870`) opens/creates the `"self"`-labelled self-DM (M11). The M12.1
  headline witness = `--attach` into the self thread: client hashes the file → uploads bytes to
  the home node's blob store over the pipe (M12-A-01) → sends a `message.file` descriptor event
  (M12-A-02) into the self-DM → a second same-identity client fetches the bytes back. Entirely
  intra-home; **never federation** (M12-A-06) → M11/D-021 intact.

**F4 split confirmed grounded.** M12.1 (local blob store + descriptor + single-node multi-device
pipe round-trip, self-thread witness) depends only on M12-A-01/02/03/08 — all net-new but small
and self-contained. M12.2 adds the `--attach` surface polish + the 4 arms; M12.3 federation
(M12-A-06); M12.4 erasure (M12-A-05). The seams the brief anticipated are real.

### M12-A-09 — Attachment kind = reuse `message.file`; the streams band is doc-only (clean F5 reservation); a blob reject needs a new parallel sub-band.

- Full EventType namespace grounded (`xgen-common/src/wire.rs:31`–`159`, `as_str`/`from_str` at
  `:168`/`:255`). `message.file` is the natural, already-validation-wired attachment home
  (M12-A-02) — **no new event kind needed** (F5's "reserve the attachment event-kind" resolves
  to "reuse `message.file`").
- `stream.*` / `media.*` prefixes return **zero code hits** — the streams fence is
  ROADMAP/doc-only. F5 is a **clean reservation**; there is no code-level band to collide with.
- **Reject codes.** `ExchangeError` (`exchange.rs:58`) + `to_wire_code()` (`exchange.rs:130`)
  cover the signed-envelope validation path (transport/state bands; the size reject is step 1 at
  `validation.rs:28`). A blob reject — **blob-too-large** (F6), **blob-unavailable** (F3 lazy
  miss), **hash-mismatch** (content-address integrity) — belongs to the **parallel transfer/ingest
  gate**, not the signed-envelope `ExchangeError` path. Recommend a net-new blob/transfer error
  type with its own codes (the design picks the band; note the wire-code-collision discipline —
  ground the chosen band against the existing register, the RC-F-01 / M10.1 lesson). `StoreError`
  (`dag/store.rs`) is event-store-internal and not the right home either.

---

## §3 Per-fork grounding ledger

| Fork | Provisional target (J-379) | Grounding verdict | Finding |
|---|---|---|---|
| **F1** | descriptor = multi-file list `attachments: [Descriptor]` | **CONFIRMED** — message content is free-form JSON; plural list is a content-schema convention, `Descriptor` net-new | M12-A-02 |
| **F2** | F2a tombstone-only lean, F2b retention-read named | **CONFIRMED net-new** — no content-erasure applier; F2b = first reader of zero-reader retention | M12-A-05 |
| **F3** | lazy-lean, audit-grounded-not-locked; Retained→eager | **CONFIRMED net-new** — push eager, no fetch-by-hash/lazy path; HeldPending is the miss-UX seam; durability-floor net-new | M12-A-06 |
| **F4** | open monolithic; M12.1→12.4 split | **CONFIRMED** — M12.1 depends only on A-01/02/03/08; seams real | M12-A-08 |
| **F5** | reserve attachment kind; steer clear of streams | **REFINED** — reuse `message.file` (no new kind); streams band doc-only (clean) | M12-A-02, M12-A-09 |
| **F6** | Pattern-A tier ceiling + tighter Space override; gate at transfer/ingest | **CONFIRMED** — flat 256 KB only; tier-table + `max_event_size` unwired; blob gate is parallel | M12-A-04 |
| **F7** | retention-aware reclaim/GC; WORM/legal-hold; reserved offload hook | **CONFIRMED net-new** — no event/data GC/TTL exists; append-only store | M12-A-07 |
| **F8** | Pattern-A tier TTL, two modes | **CONFIRMED net-new** — no TTL/lifecycle in code | M12-A-07 |
| **F9** | blob store sibling under one durable data root; default outside install folder; operator-overridable, startup-validated; node-config | **CONFIRMED + SHIFT FLAGGED** — data-root convention exists (`data_dir`/`spaces_dir`), blob store is a clean `PathsSection` sibling; BUT today's default `data_dir` = `exe_dir()` (install folder), no override flag → "default outside install folder" is a genuine new convention | M12-A-03 |

**Pattern-A spine (F6/F8/F9) cross-check:** consistent. Size/lifetime/placement are node/spec/
operator concerns; the `ModulePolicy` switch-bag (Pattern B) has **zero production readers**
(M12-A-05), so "deliberately not used for hard ceilings" is structurally clean today — nothing
reads it to loosen anything.

---

## §4 Refinements routed to design (D-065 honesty)

1. **Attachment kind = `message.file`, not `file.upload`/`message.attachment-meta`.** The latter
   two are doc breadcrumbs with zero code presence; `message.file` is the real, validation-wired
   home. (M12-A-02, M12-A-09.)
2. **F9 default-root posture is a real shift.** Today's data root defaults to the install folder
   (`exe_dir()`) with no override flag; F9's "default outside the install/system folder +
   operator-overridable + startup-validated" is a new convention that interacts with `--instance`
   segregation. Not a blocker — a deliberate design adoption to surface, not assume. (M12-A-03.)
3. **Blob rejects live in a new parallel error type, not `ExchangeError`.** The blob gate is
   parallel to the signed-envelope gate; fold the wire-code-collision discipline (RC-F-01 /
   M10.1) into the band choice. (M12-A-09.)

None of these contradict a locked fork; each refines the design's starting point.

## §5 Routed / out-of-scope (survive beyond M12)

- **Pattern-B "module-as-policy-bearer"** — reconsidering hard limits as `ModulePolicy`
  switch-bag entries; the dormant switch-bag (M12-A-05) is the home, but this is a ROADMAP
  horizon line, not invented in M12.
- **WORM/archival backend** — operator/module responsibility; M12 reserves the offload hook only
  (M12-A-07).
- **Carry-over UX** — federation-derived held-pending / unavailable client signal (a lazy blob
  miss surfaces here, M12-A-06); federation-under-load stress measurement (no scheduled home).

---

## §6 Design inputs (the audit → design handoff)

The minimal **M12.1** slice is shovel-ready once the M12-A-01 pipe-transfer fork is decided —
it composes four net-new but small, self-contained pieces:
- **Pipe byte-transfer** (M12-A-01) — the load-bearing fork; pick base64-in-JSONL vs
  length-prefixed binary vs chunked. The M12.1 long pole.
- **`Descriptor` + `message.file` builder** (M12-A-02) — `build_message_file_event` carrying
  `attachments: [Descriptor]`; reuse the validation-wired `message.file` kind.
- **Content-addressed blob store** (M12-A-03) — new `blobs_dir` sibling under `data_dir`;
  hash-keyed put/get; net-new (EventStore is event-only). Decide F9's default-root posture.
- **`--attach` on `SendArgs` + `ops::send`** (M12-A-08) — threads through one core, inherits the
  4 D-092 arms; self-thread is the witness.

Downstream sub-arcs sit on progressively more net-new ground: M12.2 (`--attach` surface polish) →
M12.3 (federation fetch-by-hash + lazy/eager + Retained durability floor, M12-A-06) → M12.4
(erasure: build the redaction mechanic + F2b retention read, M12-A-05). F6/F7/F8 enforcement
(blob ceiling at transfer/ingest, reclaim/GC, TTL, reserved WORM/offload hook) thread across the
sub-arcs per the design's sequencing.

**Sequence (Rule 0):** this audit → design (Chat/Joe) → Joe-lock → Clair runbook → implement
(sub-arc'd M12.1–M12.4) → Chat doc-bridge per arc → close → Round-2 final pre-UI gate → UI →
Streams.

## §7 Entry (Rule 0)

`CLAUDE.md` PLAY → `JOURNAL.md` J-379 → `tasks/M12_ATTACHMENTS_PHASE0_BRIEF.md` (the agenda) →
this audit → `docs/ROADMAP.md` (M12) → `ch3 §3.1.1` (size model) → `DECISIONS.md`
D-021 / D-056 / D-088 (context).
