# M12.2a — Fetch verb + `--attach` polish + F6 size gate: Implementation runbook

> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Purpose & status

The Clair-authored M12.2a implementation runbook — the **blob-feature trio** (the design §3
slice). Executes the Joe-LOCKED M12.2 design (`tasks/M12_2_DESIGN.md` v1.0, M12.2-D1..D6) on the
M12.2 D-071 Phase-0 audit (`tasks/M12_2_FETCH_GATE_DATAROOT_PHASE0_AUDIT.md`, GO; findings
M12.2-A-01..05; ledger L1–L25; forks FK-1..FK-6 locked by-recomms).

**M12.2a = the three blob-feature work-items (D2 + D3 + D4) + the full self-thread e2e** that
discharges the M12.1 honest boundary. **The F9 data-root posture shift (D5 + D6) is M12.2b — its
own runbook, explicitly OUT of this one** (M12.2-D1 split).

- **D2 — fetch CLI verb.** `ops::fetch_attachments` is built but has **zero callers** (C4) — a
  textbook D-092 4-arm verb-add over the built op; no core/wire change.
- **D3 — `--attach` surface polish.** Surface-only: multi-file `--attach` + attach-only sends
  (`text` optional on `SendArgs`); client-side only, no core/wire change.
- **D4 — F6 blob size gate.** Node-authoritative reject at `BlobUploadBegin.size` → reserved
  `BlobError` `10002 blob_too_large`; ceiling = a flat `[node]` config field; client pre-checks.

D-071 arc discipline: this runbook → **Joe-lock** → implement (C1→C4, spine-first) → Chat
doc-bridge → M12.2a close → M12.2b (F9). **No code precedes the runbook lock.** Decisions are
arc-local (D-069). Two-seat commit discipline: Clair commits **code only**; Chat authors the
canonical-record doc-bridge; Joe reassembles the D-074 atomic at push.

**Grounded against `main` @ `5330476`** (HEAD, J-383; tree clean). Every anchor in §2 was
**re-confirmed by reading production code this session** (D-078). The audit's anchors were
@ `60cfd8f`; the J-382/J-383 commits were **doc-only** (no code), so all code anchors hold — the
only drift is an immaterial **+1** on the `FetchAttachments*` structs (now `ops.rs:1863/1871/1879`,
audit said `:1862/.../:1878`). Re-confirmed, not trusted blind.

---

## §2 Grounding ledger (trio seams, re-confirmed to file:line on `main` @ `5330476`)

| # | Seam | Location | M12.2a action |
|---|---|---|---|
| **D2 — fetch verb** | | | |
| G1 | `ops::fetch_attachments` (Room-level; paginated like `history`; fetches by `blob_ref`; decrypts; **verifies `plaintext_hash`** → errors not bad bytes; writes `out_dir/<filename>`) | `xgen-client/src/ops.rs:1891` | wrap it — no behaviour change |
| G2 | `FetchAttachmentsArgs { space:String, room:String, out_dir:PathBuf }` (`Debug,Clone` — **not** clap) / `FetchedAttachment` / `FetchAttachmentsResult { space_id, room_id, files }` (`Serialize`) | `ops.rs:1863` / `:1871` / `:1879` | retrofit `#[derive(Args)]` **or** add a clap `FetchArgs` in `app.rs` |
| G3 | callers of `fetch_attachments` = **zero** (only a doc mention) | `m12_blob_roundtrip.rs:17` | confirms the unblock |
| G4 | `ClientCommand` enum; `SelfThread`/`Send`/`History`/`Members` variants | `app.rs:317`; `:358`/`:383`/`:386`/`:391` | add `Fetch(FetchArgs)` variant |
| G5 | **arm 1/4 CLI** — `History` → `app::cmd_history(args,&node,&keypair_path,&data_dir,cli.quiet)` | `main.rs:285-288` (`Send` `:280`, `Members` `:290`) | add a `Fetch` arm → `cmd_fetch` |
| G6 | **arm 2/4 run-path** (`run_batch_file`) — `History` arm | `app.rs:900` (fn); `:1033` (`Send` `:1028`, `Members` `:1038`) | add a `Fetch` arm → `cmd_fetch` |
| G7 | **arm 3/4 batch/pipe** (`dispatch_line`) — `History` → `ops::history` directly, OK/ERROR | `batch.rs:323` (fn); History arm ~`:556` (`Send`/`Members` siblings) | add a `Fetch` arm → `ops::fetch_attachments` (discard result) |
| G8 | **arm 4/4 aicontrol** (`reconstruct_argv`→clap→`dispatch` match) — `History` → `ops::history` → JSON | `aicontrol.rs:181` (`reconstruct_argv`); dispatch `:494` (`Send` `:488`, `Members` `:500`) | add a `Fetch` arm → `ops::fetch_attachments` → `serde_json::to_value` |
| G9 | `cmd_history` wrapper (human output; CLI + run-path arms call it) | `app.rs` (twin of `cmd_send`/`cmd_members`) | author `cmd_fetch` twin |
| G10 | aicontrol aux maps: `verb_tier` (`_ => Write`), `mutates_state_file` (matches list), `primary_field` (`_ => None`) | `aicontrol.rs:139` / `:154` / `:160` | **no entries needed** — `fetch` correctly defaults Write / non-mutating / whole-object (see §3) |
| **D3 — `--attach` polish** | | | |
| G11 | `SendArgs { space:String, room:String, text:String (req), attach:Option<String> }` | `app.rs:646` | `attach: Vec<String>` (repeatable) + `text: Option<String>` |
| G12 | `ops::send` attach branch (single `Descriptor` via `slice::from_ref`; warn-ignore text; mime hardcoded) | `ops.rs:1649-1708` (mime `:1683-1684`) | loop encrypt+upload over the `Vec`; attach-only; require-one guard |
| G13 | `build_message_file_event(... attachments: &[Descriptor])` (slice; plural content) | `xgen-core/.../exchange.rs:949` | already plural — pass the full `Vec` |
| G14 | `reconstruct_argv` arms = `Bool`/`String`/`other`; **no `Value::Array`** | `aicontrol.rs:181-200` | **D-065 gap** — add a `Value::Array` arm (repeat `--flag v` per element) so multi-file `--attach` works over the aicontrol arm (§3) |
| G15 | 4 send arms all route through `ops::send` (G18, M12.1) | `app.rs`/`batch.rs`/`aicontrol.rs` | polish inherits all 4 once `ops::send` + `reconstruct_argv` updated |
| **D4 — F6 gate** | | | |
| G16 | node WS dispatch `BlobUploadBegin { blob_ref, .. }` — **`size` discarded**; comment "F6 … is M12.2" | `xgen-node/src/app.rs:1780-1786` | read `size`; reject `10002` if over ceiling, before buffering |
| G17 | `BlobChunk` accumulate; `BlobUploadEnd` → `BlobStore::put` → `BlobUploadOk`/`10001`; `blob_err(code,name)` closure → `TransportMessage::Error` | `app.rs:1788-1831`; `blob_err` `:1623` | accumulate-check at `BlobChunk` (defends a lying Begin) |
| G18 | `BlobUploadBegin { protocol_version, blob_ref, size: u64 }` (size IS carried) | `xgen-core/src/wire/types.rs:198-202` | the fail-fast hook — no wire change |
| G19 | `BlobError { HashMismatch→10001, MalformedRef, Io }`; `to_wire_code`; **`10002 blob_too_large` reserved** in doc | `xgen-core/src/blob_store.rs:44` / `:63-73` | add `TooLarge` variant + `to_wire_code → 10002` |
| G20 | `[sync].batch_size` precedent: `SyncSection` field + `#[serde(default = "default_sync_batch_size")]` + `default_*` fn; read in `run_node` `:557`; threaded to `handle_connection` (param `:1475`, call `:1380`) | `app.rs:159/169/187/557/1380/1475` | mirror for a `[node].max_blob_bytes` ceiling |
| G21 | `NodeSection` struct (home for the ceiling field) | `app.rs:116-135` | add `max_blob_bytes` (serde-default) |
| G22 | `handle_connection` sig (~18 params incl. `blobs_dir: PathBuf`, `sync_batch_size: usize`); **one** call site `:1380` | `app.rs:1461-1479` / `:1380` | add a `max_blob_bytes: u64` param |
| G23 | `BLOB_CHUNK_BYTES = 128*1024`; `MAX_PAYLOAD_BYTES = 256*1024` | `types.rs:244` / `framing.rs:21` | ceiling must sit well above one chunk |
| G24 | client pre-check home: `ops::send` attach branch, after `encrypt_blob`, before `upload_blob` | `ops.rs:1663-1669` | compare `ciphertext.len()` vs the shared const |
| **Witness home** | | | |
| G25 | `ManagedProcess::{init_and_spawn_node, init_and_spawn_client, spawn_client_reusing_keypair}` (same-identity 2nd client = copy keypair, no re-init); `aicontrol_pipe` + `data_dir` fields; `AicontrolClient::connect` drives `.aicontrol` JSONL | `xgen-mptest/src/process.rs:157/208/251`; `aicontrol.rs` | the real-binary e2e (§6) |
| G26 | `self_open` → `SelfThreadResult { space_id, room_id, created }`; finds the self thread via `client_state.json` (so the 2nd client must **not** re-`self` — fetch by explicit IDs) | `ops.rs:870-908` | client-A `self` → capture IDs → client-B `fetch` by ID |
| G27 | `m12_blob_roundtrip.rs` = the in-process node + raw-WS witness home (`spawn_in_process_node`) | `xgen-node/src/tests/m12_blob_roundtrip.rs` | in-suite F6 reject witness (§6) |
| G28 | Appendix F verb tables: F.0.4 (`history` row `:101`); F.3 reference (`history` `:351`, `self` `:352`) | `docs/xgen_appendix_f_en.md` | add `fetch` rows (J-323 forward rule) — **Chat doc-bridge** |

---

## §3 D-065 surfacings (surfaced, not papered over — confirm at lock)

Three grounding findings. **None overturns a locked M12.2-D# decision; each sharpens exactly what
M12.2a builds.** Joe's confirm gates §4/§5.

### S-1 — mime detection is NOT in the locked D3 (audit FK-2(d) was dropped from the lock)

The audit FK-2 *recommended* `(a)+(b)+(d)` = multi-file + attach-only + **mime detection**. The
Joe-LOCKED **M12.2-D3** carries only **"both: multi-file + attach-only"** — mime is absent from the
lock (design §2; J-383 D3). **So M12.2a keeps mime hardcoded** (`"application/octet-stream"`,
`ops.rs:1683-1684`) and does **not** build mime detection. Recorded as reserved (§8). *Lock-ask
S-1:* confirm mime stays out of M12.2a (or fold it in explicitly at lock — it is small, but it is
not in the current D3 lock and I will not pull it in silently).

### S-2 — multi-file `--attach` over the **aicontrol** arm needs a `reconstruct_argv` array arm

`reconstruct_argv` (`aicontrol.rs:181`) maps a JSON `args` object → argv with `Bool`/`String`/`other`
arms; a JSON **array** falls to `other => push("--attach"); push(arr.to_string())` → one bogus
`--attach '["f1","f2"]'`. So a repeatable `--attach` (D3 multi-file) is **broken over the aicontrol
arm** as-is. The CLI / run-path / batch arms parse argv directly and handle a repeatable flag
natively via clap. **Fix (a real sub-item of D3, C3):** add a `Value::Array` arm to
`reconstruct_argv` that pushes `--flag value` **per element**. Keeps D-092's 4th arm honest for
repeatable flags. *Lock-ask S-2:* confirm the `reconstruct_argv` array arm rides C3.

### S-3 — the client F6 pre-check is conservative (UX courtesy, not the boundary)

The client cannot know the operator's actual node ceiling (it is node config). So the client
pre-check (D4 "the client also checks locally") compares against the **shared default const**
(§4 VB), not the live operator ceiling. Consequence: if an operator **raises** the ceiling above
the default, the client pre-check would reject locally a file the node would accept (false-negative
UX); if an operator **lowers** it, the client pre-check passes a file the node rejects — and the
**node-at-`BlobUploadBegin` gate catches it before any chunk is sent** (`size` rides Begin), so the
wasted-upload the pre-check guards against is already near-zero. The node gate is the authoritative
boundary; the client pre-check is courtesy. *Lock-ask S-3:* confirm the client pre-check uses the
shared default const + this stated conservatism (or drop the client pre-check entirely and rely on
the fast node-at-Begin reject — also coherent).

---

## §4 Runbook-level values — recommend; **Joe locks at the runbook lock**

| Value | Recommendation | Grounding |
|---|---|---|
| **VA — fetch verb shape (D2)** | Verb `fetch` with clap alias `fetch-attachments` (matches the op). **Room-level** selector `--space <id> --room <id>` (reuse the built op verbatim; FK-1(a); by-event/by-blob reserved). **`--out-dir <path>` required** (explicit, no surprise writes; FK-1). Retrofit `FetchAttachmentsArgs` with `#[derive(Args)]` (add `#[arg(long)]` to its 3 fields; `out_dir: PathBuf` parses from a string) rather than a parallel `FetchArgs` (one struct, no drift). `ClientCommand::Fetch(FetchAttachmentsArgs)`. **Filename collisions within `out_dir`:** keep the built op's `std::fs::write` = **overwrite** (simplest; matches shipped behaviour); suffix-on-collision reserved (§8). | G1/G2/FK-1 |
| **VB — F6 ceiling source + const + default (D4)** | New `[node].max_blob_bytes: u64`, `#[serde(default = "default_max_blob_bytes")]` on `NodeSection` (mirrors `[sync].batch_size`). A shared `pub const DEFAULT_MAX_BLOB_BYTES: u64` in **xgen-core** (next to `BLOB_CHUNK_BYTES`, `wire/types.rs`) referenced by **both** the node default fn **and** the client pre-check. **Default = 16 MiB (`16 * 1024 * 1024`)** — clearly multi-MB (F6 is "MB-scale"), but modest: the node reassembles the **whole ciphertext into one in-memory `Vec`** (`app.rs:1786`), so a huge default is a memory-DoS surface — which is exactly what F6 exists to bound. Operator-raisable. **Joe locks the number** (16 MiB recommended; surface the in-memory-reassembly tradeoff). The ceiling gates **ciphertext** bytes (`BlobUploadBegin.size` = ciphertext per its doc). | G18/G20/G21/G23; CLAUDE "Error Code Convention" |
| **VC — attach-only + require-one (D3)** | `SendArgs.text: Option<String>`. **Ops-level guard** in `ops::send`: if `attach.is_empty() && text.is_none()` → bail `"a send must carry --text or --attach"` (clap `ArgGroup` is awkward across the reconstruct-argv arm; an ops-level check is robust + covers all 4 arms). Attach-only = `attach` non-empty + `text` `None`. **Combined** (`attach` non-empty + `text` `Some`) → preserve today's **warn-and-ignore-text** (FK-2(c) combined-in-one-`message.file` is a content-schema question, **deferred** — §8). | G11/G12/FK-2 |
| **VD — multi-file (D3)** | `SendArgs.attach: Vec<String>` (repeatable `--attach`; clap auto-repeats a `Vec`). `ops::send`: loop each path → read → `encrypt_blob` → client pre-check (VB/S-3) → `upload_blob` → build a `Descriptor`; collect `Vec<Descriptor>`; one `build_message_file_event(&descriptors)` (already a slice) → one `message.file` event → one `event_id`. + the S-2 `reconstruct_argv` array arm. | G11/G12/G13/G14 |

---

## §5 Build sequence (spine-first; written for the recommended S-1..S-3 / VA..VD resolution)

Four code commits, then Chat's doc-bridge close. Per-commit DoD in §7. Each is its own atomic
commit (Clair commits code; Chat authors the canonical-record doc-bridge; Joe reassembles +
pushes). Spine-first = the three trio pieces (the witnesses rest on them) before the witness
commit; within the trio, the **headline unblock (fetch verb) first**.

- **C1 — fetch CLI verb (D2; the headline unblock).**
  Retrofit `FetchAttachmentsArgs` with `#[derive(Args)]` + `#[arg(long)]` on `space`/`room`/`out_dir`
  (+ alias `fetch-attachments`, VA); `ClientCommand::Fetch(FetchAttachmentsArgs)` (G4); a `cmd_fetch`
  wrapper (human output, twin of `cmd_history`, G9) for the CLI (G5) + run-path (G6) arms; the batch
  arm → `ops::fetch_attachments` discard-result OK/ERROR (G7); the aicontrol arm →
  `ops::fetch_attachments` → `serde_json::to_value` (G8). **No aux-map entries** (G10 — `fetch`
  defaults correctly: `_ => Write` tier, non-state-mutating, whole-object `primary_field`).
  **No core/wire change.**
  In-suite tests: clap parse of `fetch --space --room --out-dir`; the verb routes through each of the
  4 arms (seam tests mirroring the `History`/`Members` test pattern).

- **C2 — `--attach` polish (D3; surface-only).**
  `SendArgs.attach: Vec<String>` + `text: Option<String>` (G11); `ops::send` loops encrypt+upload over
  the `Vec`, builds `Vec<Descriptor>`, passes the full slice to the already-plural builder (G12/G13,
  VD); attach-only when `text` absent + the require-one guard (VC); combined → warn-and-ignore-text
  preserved; **mime stays hardcoded** (S-1). **+ the S-2 `reconstruct_argv` `Value::Array` arm**
  (G14) so multi-file `--attach` works over the aicontrol arm. The 4 send arms inherit via `ops::send`
  (G15). **No core/wire change.**
  In-suite tests: multi-file `ops::send` builds **one** `message.file` with **N** descriptors;
  attach-only carries no `message.text`; require-one guard rejects an empty send; `reconstruct_argv`
  of `{"attach":["a","b"]}` → `--attach a --attach b`.

- **C3 — F6 size gate (D4; node-authoritative).**
  `BlobError::TooLarge` variant + `to_wire_code → Some((10002,"blob_too_large"))` (G19; **re-grep the
  register before emitting** — RC-F-01/M10.1 discipline; `10002` is reserved, not yet emitted).
  `pub const DEFAULT_MAX_BLOB_BYTES` (xgen-core, VB); `[node].max_blob_bytes` on `NodeSection` +
  `default_max_blob_bytes()` (G21); read in `run_node` (mirror `:557`); thread a `max_blob_bytes: u64`
  param into `handle_connection` (sig `:1475`-style + the one call site `:1380`, G22). **Gate:** at
  `BlobUploadBegin` read `size`; if `size > ceiling` → `blob_err(10002,"blob_too_large")` and do **not**
  start `blob_upload` (G16); at `BlobChunk`, if the accumulating buffer exceeds the ceiling → reply
  `10002` once + drop the in-flight upload (defends a lying Begin, G17). **Client pre-check** in
  `ops::send`: after `encrypt_blob`, if `ciphertext.len() as u64 > DEFAULT_MAX_BLOB_BYTES` → bail
  before `upload_blob` (G24, S-3). **Spine witness W-toolarge (in-suite, RED-on-revert).**
  In-suite tests (m12_blob_roundtrip-style, in-process node + raw WS): an over-ceiling `upload_blob`
  is rejected `10002` at Begin (RED-on-revert: remove the gate → accepted → RED); an at/under-ceiling
  blob still round-trips; the client pre-check unit rejects an over-const ciphertext.

- **C4 — the e2e witnesses (xgen-mptest; box-gated, real binaries).**
  The full self-thread e2e the M12.1 boundary deferred (§6). **Box-gated** (`#[ignore]`, spawns real
  `xgen-node.exe`/`xgen-client.exe`, Windows named-pipe, needs the binaries **built** first) — **flag
  the RUN for Joe** (MP-arc precedent); not claimed green box-free. New test file under
  `xgen-mptest/tests/` (e.g. `m12_2a_self_thread_e2e.rs`).

**(Chat) doc-bridge + M12.2a close** — canonical-record flips (CLAUDE PLAY, JOURNAL, ROADMAP,
design status); the §3 surfacings recorded; **Appendix F gains the `fetch` rows** (F.0.4 + F.3,
G28 — J-323 forward rule, the Chat seat owns the spec doc); M12.2b (F9) opens next.

---

## §6 Witnesses

**In-suite (run in `cargo test --workspace`; count toward the baseline + N):**
- **C1** — fetch verb clap parse + 4-arm routing seam tests.
- **C2** — multi-file builder (1 event / N descriptors); attach-only (no `message.text`); require-one
  guard; `reconstruct_argv` array arm.
- **C3 / W-toolarge (spine, RED-on-revert)** — in-process node + raw WS: an over-ceiling `upload_blob`
  → `10002 blob_too_large` at Begin; at/under-ceiling still round-trips; client pre-check unit. RED-on-revert:
  remove the Begin gate → over-ceiling blob stored → RED.

**Box-gated `#[ignore]` (real binaries over `.aicontrol`; RUN flagged for Joe — discharges the M12.1 boundary):**
- **W-e2e (headline)** — spawn node → `init_and_spawn_client` **A** → drive A over `.aicontrol`:
  `self` (capture `space_id` + `room_id` from the JSON reply) → `send --space <s> --room <r> --attach <file>`
  → `spawn_client_reusing_keypair` **B** (same identity, copies the keypair) → drive B:
  `fetch --space <s> --room <r> --out-dir <dir>` → assert the file at `B.data_dir/<dir>/<filename>` is
  **byte-identical** to the original (and B's JSON `FetchAttachmentsResult.files[].{path,size}` matches).
  B must **not** re-`self` (it has no `client_state.json` self-KnownSpace; it fetches by the explicit
  IDs A returned — the op syncs the room by ID, no KnownSpace lookup; G26). This is the ops/CLI-layer
  coverage C5's raw-WS witness could not reach.
- **W-multi** — A `send` with **two** `--attach` files in one send → B `fetch` retrieves **both**
  byte-identical (exercises D3 multi-file + the S-2 aicontrol array arm end-to-end).
- **W-toolarge-e2e** — A `send --attach <over-ceiling file>` → the `.aicontrol` reply is an
  ERROR carrying `10002` (the real-binary face of the C3 gate).

---

## §7 Definition of Done

**Per-commit gate (each of C1–C4):** `cargo build --workspace` 0-error
(`CARGO_TARGET_DIR=C:/cargo-targets/XGenProtocol`);
`cargo clippy --workspace --all-targets --all-features -- -D warnings` clean;
`cargo test --workspace` green (baseline **1429/0** + the commit's new in-suite tests); for the
spine commit (C3), **RED-on-revert recorded** in the commit message. C4's box-gated `#[ignore]`
tests are **not** in the workspace count — built, run explicitly, and the RUN result flagged for Joe.

**Milestone gate (at C4 / close):** W-toolarge green in-suite (RED-on-revert recorded); the
box-gated W-e2e / W-multi / W-toolarge-e2e **run on a freed box and reported** (the M12.1 boundary
discharged — the real fetch **verb** round-trips byte-identical through the real binaries); the §3
surfacings stated at close; Appendix F `fetch` rows landed (Chat).

*(No "commit pushed" DoD line — unflippable inside its own commit; `Status: COMPLETED` is the
shipped signal. Joe pushes.)*

---

## §8 Out of scope (M12.2b / later sub-arcs — do NOT pull in)

- **F9 data-root posture (M12.2-D5 + D6)** — `--data-dir` flag + env + platform-dir default
  (hand-rolled) + startup validation + leave-as-legacy existing data. **= M12.2b, its own runbook.**
- **mime detection** (audit FK-2(d)) — dropped from the D3 lock (S-1); stays hardcoded. Reserved.
- **Combined text + attachment in one `message.file`** (FK-2(c)) — a content-schema question;
  today's warn-and-ignore-text stands.
- **By-event / by-`blob_ref` fetch granularity** (FK-1) — additive later if a use-case surfaces.
- **Filename-collision suffixing in `out_dir`** — overwrite stands for M12.2a (VA).
- **Pattern-A tier→size table + per-Space immutable override** (F6's full shape) — the named
  Pattern-A enrichment; the M12.2a flat-config gate mechanism accepts it without rework (M12.2-D4).
- **M12.3** federation fetch-by-hash + F3 + `10003 blob_unavailable`; **M12.4** erasure.

---

## §9 Sequence + entry (Rule 0)

this runbook → **Joe locks S-1..S-3 + VA..VD + the §5 sequence** → implement C1→C4 (spine-first)
→ hand Joe the push per commit → Chat doc-bridge → M12.2a close (the e2e witnessed) → M12.2b (F9).

**Entry (Rule 0):** `CLAUDE.md` PLAY → `JOURNAL.md` J-383 → `tasks/M12_2_DESIGN.md` (§2 M12.2-D1..D6 /
§3 M12.2a) → `tasks/M12_2_FETCH_GATE_DATAROOT_PHASE0_AUDIT.md` (FK-1/FK-2/FK-3 + ledger) → this
runbook → `tasks/M12_1_BLOB_STORE_ATTACH_IMPL.md` (COMPLETED — the M12.1 shipped state + the named
e2e boundary) → `docs/ROADMAP.md` (M12).
