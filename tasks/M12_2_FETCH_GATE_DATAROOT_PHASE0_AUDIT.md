# M12.2 — Fetch verb + `--attach` polish + F6 size gate + F9 data-root: D-071 Phase-0 Audit
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

The D-071 Phase-0 grounding audit for **M12.2** — the second M12 sub-arc. M12's design is
Joe-LOCKED (`tasks/M12_ATTACHMENTS_DESIGN.md` v1.1, M12-D1..D10); **J-381 set M12.2's *scope*
(M12-D10)**; this Phase-0 grounds the *mechanics* so Chat/Joe can lock the M12.2 design.

**M12.2 scope** = the four work-items the design + the kickoff name:
1. the **fetch CLI verb** (the headline — unblocks the full self-thread e2e the M12.1 close named);
2. **`--attach` surface polish** (multi-file, attach-only, mime detection);
3. the **F6 blob-size gate** at transfer/ingest (`blob_too_large`, the reserved `10002`);
4. the **F9 data-root posture shift** (M12-D7: default-outside-install + `--data-dir` override +
   startup validation, touching every node file + `--instance`).

**Audit only.** Grounds every seam to file:line on `main`, surfaces the forks (FK-1..FK-6) with
recommendations, returns a verdict. **No design, no runbook, no code** — those follow (audit →
Chat/Joe design → Joe-lock → Clair runbook → impl). Recommendations are inputs to the design
lock, not pre-decisions (D-065).

**Grounded against `main` @ `60cfd8f`** (HEAD, J-382 close; tree clean). Every anchor below was
**re-confirmed by reading production code this session** (D-078 — the M12.1 runbook's anchors at
`5e96ad7` drifted across the C1–C5 commits; lines re-grounded, not trusted blind).

**Verdict: GO.** Every M12.2 seam is grounded and each work-item is well-precedented: the fetch
verb is a textbook D-092 4-arm verb-add over an **already-built** `ops::fetch_attachments`;
`--attach` polish is **surface-only** (the builder, content schema, and fetch reader are already
plural); F6 is a small gate at a seam the M12.1 code **explicitly flagged for M12.2** (the
reserved `10002` + the discarded `BlobUploadBegin.size`); F9 is a **concentrated-code /
total-operational** shift (one default-resolution site + the config-default rooting + a new flag).
The full self-thread e2e the M12.1 close deferred **is reachable** via `xgen-mptest` once the
fetch verb is a `ClientCommand` (see §5). The six forks are genuine design calls, not blockers.

**Two grounding sharpenings surface (D-065, routed to design, neither overturns a lock):**
- **F9's blast radius is concentrated, not a per-file sweep.** Every node file already hangs off
  one `data_dir` root; the code change is `resolve_data_dir` + the `NodeConfig::default` exe_dir
  rooting + a `--data-dir` flag — ~3 sites, not the ~14 `data_dir.join(...)` consumers (they
  inherit). The *operational* impact (where data lives) is total; the *code* impact is small. The
  real risk is the config-default + the existing-install migration (FK-5), not a wide sweep.
- **There is no "immutable Space" type** to hang the F6 tighter-override on. The only `immutable`
  in `state.rs` is the `e2e_encryption` set-once property; the natural home for a per-Space tighter
  blob cap is a create-time stored field like the **dormant `max_event_size`** (M12-A-04).

---

## §2 Findings (M12.2-A-##)

### M12.2-A-01 — Fetch verb: `ops::fetch_attachments` is fully built but has ZERO callers; the verb-add is a textbook D-092 4-arm wrap. (WI-1; LOAD-BEARING — the e2e unblock)

`ops::fetch_attachments` (`xgen-client/src/ops.rs:1891`) is complete + C4-shipped: it syncs a
Room's `message.file` events (paginated exactly like `ops::history`, `ops.rs:1915-1959`), extracts
each `Descriptor`, fetches the ciphertext by `blob_ref` over WS (`conn.fetch_blob`), decrypts under
the per-blob key, **verifies `plaintext_hash`** (the client-side W3 integrity check,
`ops.rs:1976` — errors rather than writing bad bytes), and writes the plaintext to
`out_dir/<filename>`. Its args/result are **hand-built structs** (`FetchAttachmentsArgs` —
`#[derive(Debug, Clone)]`, *not* clap `Args` — `ops.rs:1862`; `FetchAttachmentsResult` /
`FetchedAttachment` `Serialize`, `ops.rs:1870/1878`).

**Zero callers.** A workspace grep for `fetch_attachments` / `FetchAttachmentsArgs` returns only
the definition + one **doc-comment** mention in `xgen-node/src/tests/m12_blob_roundtrip.rs:17`. No
`ClientCommand` variant, no dispatch arm, no test. The C5 witnesses drove
`Connection::upload_blob/fetch_blob` **directly** (raw WS), bypassing the ops layer — which is
exactly the M12.1 honest boundary ("the thin xgen-client ops glue … has no full ops-level /
self-thread e2e; the real-binary fetch path needs the M12.2 fetch CLI verb"). The op is built; it
is unreachable.

**The verb-add is the promoted D-092 4-arm pattern** (ban/room_update/thread all followed it). The
four arms for a read verb (the `History` precedent):

| Arm | Site (main @ `60cfd8f`) | History dispatches to |
|---|---|---|
| **CLI** (top-level) | `xgen-client/src/main.rs:198` match; History → `:285` | `cmd_history` |
| **run-path** (`run_batch_file`) | `xgen-client/src/app.rs:900` fn, `:959` match; History → `:1033` | `cmd_history` |
| **batch/pipe** (`dispatch_line`) | `xgen-client/src/batch.rs:323` fn; History → `:556` | `ops::history` directly |
| **aicontrol** (`reconstruct_argv`→clap→match) | `reconstruct_argv` `aicontrol.rs:181`; dispatch match `aicontrol.rs:395`; History → `:494` | `ops::history` → JSON `Value` |

So M12.2 adds: a clap `FetchArgs` (or retrofit `FetchAttachmentsArgs` with `#[derive(Args)]`), a
`ClientCommand::Fetch(FetchArgs)` variant (enum at `app.rs:317`), a `cmd_fetch` wrapper (CLI +
run-path arms), and the batch + aicontrol arms calling `ops::fetch_attachments`. The CLI/run-path
arms call `cmd_*` wrappers (human output); batch returns OK/ERROR; aicontrol serialises the
`FetchAttachmentsResult` as JSON (`Serialize` already derived). **No core/wire change** — the op,
the transfer, and the store all exist. Verb shape = **FK-1**.

### M12.2-A-02 — `--attach` polish is surface-only: the builder, content schema, and fetch reader are already plural; mime is hardcoded. (WI-2)

`SendArgs` (`xgen-client/src/app.rs:646`) today: `space: String`, `room: String`, `text: String`
(**required** — `#[arg(long)]`), `attach: Option<String>` (**single file**). The `ops::send`
attach branch (`ops.rs:1649-1708`): reads the file → `encrypt_blob` → `upload_blob` over WS →
builds one `message.file` event from a **single** descriptor (`std::slice::from_ref(&descriptor)`,
`ops.rs:1693`); **warns and ignores `--text`** when `--attach` is given (`ops.rs:1655-1660`,
"combined text + attachment lands at M12.2"); **mime hardcoded** `"application/octet-stream"` with
the comment "mime detection is M12.2 surface polish" (`ops.rs:1683-1684`).

The downstream is **already plural**:
- `build_message_file_event(... attachments: &[Descriptor])` takes a **slice** and serialises
  `json!({ "attachments": attachments })` (`xgen-core/src/message/exchange.rs:949-970`).
- `ops::fetch_attachments` iterates **all** descriptors in `content["attachments"].as_array()`
  (`ops.rs:1931-1939`).

So the M12.2 polish is **client-surface-only**: (a) **multi-file** = `SendArgs.attach:
Vec<String>` (repeatable `--attach`), loop encrypt+upload, pass the full `Vec` to the already-slice
builder; (b) **attach-only** = `text` → optional (drop the `--text ""` requirement); (c)
**combined text + attachment** in one `message.file` (today the warn-and-ignore); (d) **mime
detection** from the filename/extension. None touches core, wire, or the store. Scope = **FK-2**.

### M12.2-A-03 — F6 gate: the seam is flagged + ready; `BlobUploadBegin` carries a discarded `size`; the reserved `10002` is one variant away. (WI-3)

The node-side blob-upload reassembly loop (`xgen-node/src/app.rs:1780-1831`, in the WS post-auth
dispatch) has the F6 seam **explicitly flagged**: at `BlobUploadBegin` (`:1780`) the comment reads
*"F6 blob-size gating is M12.2; M12.1 grows the buffer from the streamed chunks (no claimed-size
allocation)"* (`:1784-1785`). The variant **already carries `size: u64`** (`xgen-core/src/wire/types.rs`,
`BlobUploadBegin { protocol_version, blob_ref, size }`) — currently **discarded** (`..` at `:1782`).
So a fail-fast at Begin (reject before buffering, using the claimed `size`) **and** an
accumulate-and-check at each `BlobChunk` (`:1788`, defends against a lying Begin) are both
available at one site.

The reject type is ready: `BlobError` (`xgen-core/src/blob_store.rs:44`) is the M12-D9 parallel
type; `to_wire_code` (`:63-73`) maps only `HashMismatch → 10001` today, with **`10002
blob_too_large` (F6, M12.2)** and `10003 blob_unavailable` (F3, M12.3) reserved in the doc comment
(`:67-68`). Domain 10 = 10000–10999 (CLAUDE "Error Code Convention"). F6 adds a `BlobError::TooLarge`
variant + the `10002` wire arm + a reject reply at the node loop (the success reply is
`BlobUploadOk`; `blob_err(code, name)` already exists for the UploadEnd 10001 path, `:1814/1826`).
**Re-grep the register at build** (RC-F-01 / M10.1 collision discipline) — `10002` is reserved, not
yet emitted.

**The ceiling source is genuinely net-new (Pattern-A has no map today):**
- The only enforced size limit is `MAX_PAYLOAD_BYTES = 256 * 1024` (`framing.rs:21`), enforced
  reject-before-signature at validation step 1 (M12-A-04). It gates the **descriptor event** (small),
  **not** the blob bytes (chunked at `BLOB_CHUNK_BYTES = 128*1024`, `types.rs:244`, never one frame).
  So the blob gate is genuinely **parallel** to the envelope ceiling, as F6 anticipated.
- `SpaceState.max_event_size: Option<u64>` (`state.rs:191`) is **stored** (parsed from create
  content `state.rs:289`, `None`-defaulted at every other constructor `state.rs:448/564`,
  `algorithm.rs:419`) but **zero enforcement reads** anywhere — stored-but-unenforced.
- **No tier→size table in code** (the §3.1.1 256/64/32/16/8-KB-by-tier model is spec-but-unwired).
- **No "immutable Space" type** — the only `immutable` in `state.rs` (`:211`) is the
  `e2e_encryption` set-once property. F6's "tighter immutable-Space override" would be a per-Space
  **create-time stored cap** (the shape `max_event_size` already has), tighter-only below the
  ceiling — not a Space class.
- Node-config numeric-limit precedent exists: `[sync].batch_size` (`app.rs:170`, a `u32` read at
  `app.rs:557`). Config sections present: Node/Paths/Logging/Sync/Federation/Bootstrap/Storage
  (`app.rs:116/137/151/159/211/228/290`) — a flat `[node]` blob ceiling has a clean home.

Gate **placement + ceiling source** = **FK-3**.

### M12.2-A-04 — F9 data-root: one root, ~14 consumers; the code-touch is concentrated; `data_dir` is chosen *before* config load. (WI-4)

`resolve_data_dir` (`xgen-node/src/main.rs:173-188`) returns `app::exe_dir()` (the install folder),
or `exe_dir()/instances/<label>` under `--instance`. **No `--data-dir` flag.** The Cli flags are
`--config`, `--local`, `--instance` (global), `--port`, `--log-level`, `--check-config`
(`main.rs` Cli struct). `exe_dir()` is `GetModuleFileNameW`-derived on Windows (`app.rs` `exe_dir`).

**Chicken-egg (load-bearing for FK-4):** `data_dir = resolve_data_dir(&cli.instance)` runs at
`main.rs:202`, and `config_path = data_dir.join("xgen-node_config.toml")` at `:206` — i.e. the data
root is chosen **before** the config is read. So a `config.paths.*` field **cannot** set `data_dir`
itself (the config lives *under* it). The override must be a **CLI flag** (`--data-dir`) or an env
var, resolved at `resolve_data_dir`. (`config.paths.spaces_dir` / `blobs_dir` *can* redirect those
sub-dirs off the root — `app.rs:741-744/750-753` — but not the root.)

**One root, many consumers, concentrated code-touch.** `data_dir` is threaded through `run_node`
and ~14 files all derive from it via `data_dir.join(...)`:

| File | Site |
|---|---|
| `xgen-node_identities.db` | `app.rs:757` |
| `xgen-node_federation.json` | `app.rs:938` |
| `xgen-node_federation_queue.json` | `app.rs:969` |
| `xgen-node_federation_policy.json` | `app.rs:1000` |
| `xgen-node_bootstrap.json` | `app.rs:1055` |
| `xgen-node_state.json` | `app.rs:1090` |
| `xgen-node_node_policy.json` | `app.rs:1267` |
| `audit/` | `app.rs:913` |
| `logs/` | `app.rs:652` |
| `spaces_dir` (default `<data_dir>/spaces`) | `app.rs:744` |
| `blobs_dir` (default `<data_dir>/blobs`) | `app.rs:753` |
| pid file | `app.rs:1209` |
| `space_local_metadata` | `app.rs:871` |
| `config_path` | `main.rs:206` |

These consumers **do not change** — they already derive from `data_dir`. The code change is
concentrated at: (1) `resolve_data_dir` (`main.rs:173`, the default + the new override + startup
validation), (2) the `--data-dir` flag on the Cli struct, (3) `NodeConfig::default` which roots
*every* path at `exe_dir()` (`app.rs:326` `let dir = exe_dir()`, `:339-340` spaces_dir/blobs_dir) —
so default-config generation (`maybe_write_default_config`, the J-080 carry-over) must root at the
new default. **No platform-data-dir crate** (`dirs` / `directories`) is in deps — the
"platform data dir" option (FK-4) needs a new dependency or a hand-rolled per-OS resolution
(`%APPDATA%` / `$XDG_DATA_HOME` / `~/Library`). **No existing data-dir validation** (writable /
durable / not-tmp) — startup validation is net-new (the `durable` hits in code are all
storage-engine SE-SUB, unrelated).

**`--instance` interaction (FK-5):** today `exe_dir()/instances/<label>`; after the shift it
becomes `<new_default_root>/instances/<label>`. Whether `--data-dir X --instance n1` →
`X/instances/n1` (compose) or the two are mutually exclusive is a fork.

**Existing-install data (FK-5):** an existing deployment's data sits at `exe_dir()`. After the
default moves outside, that data is "left behind" unless migrated, or unless the operator points
`--data-dir` at the old `exe_dir`. The **M10.4-D5 "leave-as-legacy-and-named"** precedent applies
(don't auto-migrate; document the override). FK-5.

### M12.2-A-05 — The full self-thread e2e is reachable via `xgen-mptest` once the fetch verb is a `ClientCommand`. (WI-5; discharges the M12.1 boundary)

The M12.1 close named the honest boundary: no crate links both `xgen-client` and `xgen-node`
in-process, so the ops-level / self-thread round-trip went unwitnessed. Grounding the witness
homes:

- **No in-process both-crate test.** `xgen-node` does **not** (dev-)depend on `xgen-client`
  (confirmed: no hit in `xgen-node/Cargo.toml`); `xgen-client` tests cannot spawn an `xgen-node`.
  C5 (`m12_blob_roundtrip.rs`) drives a real in-process node (`spawn_in_process_node`, the
  `phase9_harness`) + a **raw WS `Connection`** (`connect_url` + `client_authenticate` +
  `upload_blob`/`fetch_blob`) — it witnesses the transfer *mechanism*, deliberately bypassing the
  ops/CLI glue.
- **`xgen-mptest` is the real-binary harness** (`xgen-mptest/Cargo.toml:8-9`: "TEST-ONLY: spawns the
  real built `xgen-node.exe`/`xgen-client.exe` as separate processes, drives [them]" via the
  `.aicontrol` pipe; `bench.rs` `ManagedProcess::init_and_spawn_node`). It already drives
  `xgen-client.exe` over aicontrol.

So **once the fetch verb is a `ClientCommand`** (so `reconstruct_argv` → clap → the aicontrol match
arm routes it), `xgen-mptest` can drive the real binaries: spawn a node + a client (init'd with a
keypair), `self` (open the self thread) → `send --attach <file>` → `fetch --space … --room … --out-dir
<dir>` → assert the fetched file is **byte-identical** (on disk) and the aicontrol JSON result
(`FetchAttachmentsResult.files[].{path,size}`) matches. "Same-identity second client" in the
real-binary path = the same keypair file (a second client init'd with it, or the same client
re-running fetch). **This is the witness the M12.1 boundary deferred — reachable, no new crate
edge, gated only on the fetch verb being a routable `ClientCommand`.** The M12.2 e2e *adds* the
ops/CLI-layer coverage C5 couldn't reach; the raw-WS C5 witnesses stand for the mechanism.

---

## §3 Grounding ledger (file:line, re-confirmed on `main` @ `60cfd8f`)

| # | Seam | Location | M12.2 relevance |
|---|---|---|---|
| L1 | `ops::fetch_attachments` (built, paginated like `history`, verifies `plaintext_hash`) | `xgen-client/src/ops.rs:1891` | WI-1 — the op the verb wraps |
| L2 | `FetchAttachmentsArgs` (hand-built `Debug/Clone`, **not** clap) / `FetchAttachmentsResult` (`Serialize`) | `ops.rs:1862` / `:1878` | WI-1 — retrofit `Args` or add a clap `FetchArgs` |
| L3 | `fetch_attachments` callers = **zero** (only a doc mention) | `m12_blob_roundtrip.rs:17` | WI-1 — confirms the unblock |
| L4 | `ClientCommand` enum; `Send`/`History`/`Members` variants | `app.rs:317`; `:383`/`:386`/`:391` | WI-1 — add `Fetch` variant |
| L5 | CLI arm (top-level match) — History → `cmd_history` | `main.rs:198`; History `:285` | WI-1 — arm 1/4 |
| L6 | run-path arm (`run_batch_file` match) — History → `cmd_history` | `app.rs:900`/`:959`; History `:1033` | WI-1 — arm 2/4 |
| L7 | batch/pipe arm (`dispatch_line`) — History → `ops::history` | `batch.rs:323`; History `:556` | WI-1 — arm 3/4 |
| L8 | aicontrol arm (`reconstruct_argv`→clap→match) — Send/History/Members → `ops::*`→JSON | `aicontrol.rs:181`/`:395`; `:488`/`:494`/`:500` | WI-1 — arm 4/4 |
| L9 | `cmd_history` / `cmd_send` / `cmd_members` wrappers | `app.rs:2836` / `:2643` / `:2786` | WI-1 — `cmd_fetch` twin |
| L10 | `SendArgs { space, room, text:String(req), attach:Option<String> }` | `app.rs:646` | WI-2 — `attach: Vec`, `text` optional |
| L11 | `ops::send` attach branch (single descriptor; warn-ignore text; hardcoded mime) | `ops.rs:1649-1708` (mime `:1683-1684`) | WI-2 — multi-file loop + mime |
| L12 | `build_message_file_event(... attachments: &[Descriptor])` (slice; plural content) | `exchange.rs:949-970` | WI-2 — multi-file builder-ready |
| L13 | fetch reader loops all `content["attachments"]` | `ops.rs:1931-1939` | WI-2 — multi-file reader-ready |
| L14 | node blob-upload loop; **"F6 … is M12.2"** comment; `BlobUploadBegin{size}` discarded `..` | `app.rs:1780-1831` (`:1784`, `:1782`) | WI-3 — the gate site |
| L15 | `BlobError` + `to_wire_code` (HashMismatch→10001; **10002/10003 reserved**) | `blob_store.rs:44`/`:63-73` | WI-3 — add `TooLarge`→10002 |
| L16 | `MAX_PAYLOAD_BYTES=256KB` (envelope, gates descriptor event) / `BLOB_CHUNK_BYTES=128KB` | `framing.rs:21` / `types.rs:244` | WI-3 — blob gate is parallel |
| L17 | `SpaceState.max_event_size: Option<u64>` stored, **zero enforcement** | `state.rs:191` (parse `:289`) | WI-3 — per-Space tighter-override candidate |
| L18 | `[sync].batch_size` node-config numeric-limit precedent | `app.rs:170` | WI-3 — flat ceiling home |
| L19 | `resolve_data_dir` = `exe_dir()` / `exe_dir()/instances/<label>`; **no `--data-dir`** | `main.rs:173-188` | WI-4 — the default-shift site |
| L20 | `data_dir` chosen at `main.rs:202` **before** config load (`config_path` `:206`) | `main.rs:202`/`:206` | WI-4 — override must be flag/env, not a config field |
| L21 | `NodeConfig::default` roots every path at `exe_dir()` | `app.rs:326`/`:339-340` | WI-4 — default-config must re-root |
| L22 | ~14 `data_dir.join(...)` consumers (table §2-A-04) | `app.rs` (various) | WI-4 — inherit; not touched |
| L23 | no `dirs`/`directories` dep; no existing data-dir validation | (grep) | WI-4 — platform-dir + validation net-new |
| L24 | `xgen-node` does NOT dep `xgen-client` | `xgen-node/Cargo.toml` | WI-5 — no in-process both-crate test |
| L25 | `xgen-mptest` spawns real `.exe`s, drives via `.aicontrol` | `xgen-mptest/Cargo.toml:8-9`; `bench.rs` | WI-5 — the e2e witness home |

---

## §4 Forks (FK-1..FK-6 — Chat/Joe lock at the M12.2 design; recommendations are inputs)

### FK-1 — Fetch verb shape + output

- **Granularity.** The built `ops::fetch_attachments` is **Room-level** (fetches *all* attachments
  in a Room into `out_dir`) — it matches the `ops::history` sync pattern. Options: (a) keep
  Room-level (reuse the built op verbatim); (b) add **by-event** (one message's attachments); (c)
  add **by-`blob_ref`** (one specific blob).
  - *Recommendation:* **(a) Room-level for M12.2** — the op is built + tested-in-shape; expose it.
    By-event/by-blob granularity is a later additive arg if a use-case surfaces (the descriptor
    list is already in the event content, so by-event is a cheap follow-on). Don't build granularity
    the headline witness doesn't need.
- **Output.** The bytes **always** go to a local path (`out_dir/<filename>`) — that is the payload,
  not a fork. The *summary* output: CLI/run-path arms print a human summary (`cmd_fetch`); batch
  returns OK/ERROR; aicontrol serialises `FetchAttachmentsResult` as JSON (`Serialize` already
  derived). So "path vs stdout" resolves cleanly — files to `out_dir`, summary to stdout/JSON.
  - *Recommendation:* default `out_dir` to CWD or a `<data_dir>`-relative dir? **Lean: require
    `--out-dir`** (explicit, no surprise writes), matching the hand-built `FetchAttachmentsArgs.out_dir`.
- **Verb name + arg struct.** `ClientCommand::Fetch(FetchArgs)` with `--space`/`--room`/`--out-dir`.
  Retrofit `FetchAttachmentsArgs` with `#[derive(Args)]` (it already has the fields), or add a clap
  `FetchArgs` in `app.rs` (the convention — `SendArgs` etc. live there) that maps to it. **+ Appendix
  F** (new client verb, the J-323 forward rule) is a close deliverable.

### FK-2 — `--attach` polish scope (both, or staged?)

Four candidate polish items, all surface-only (M12.2-A-02): (a) multi-file (`attach: Vec<String>`);
(b) attach-only (`text` optional); (c) combined text+attachment in one `message.file`; (d) mime
detection.
- *Recommendation:* **(a)+(b)+(d) in M12.2** (multi-file + attach-only + mime — small, the
  builder/reader are plural, the C4 code flagged mime as M12.2). **(c) combined text+attachment**
  is a content-schema question (does `message.file` content carry *both* a `text` and `attachments`,
  or does the client send two events?) — lean **defer or lock explicitly** at design; today's
  warn-and-ignore is honest. Surface the (c) decision; don't smuggle it.

### FK-3 — F6 gate placement + ceiling source

- **Placement.** (a) client pre-upload reject (fail-fast, read file size before encrypt/upload); (b)
  node-ingest reject (authoritative, at `BlobUploadBegin.size` + accumulate-check at `BlobChunk`);
  (c) both.
  - *Recommendation:* **(c) both.** The node **MUST** check (an untrusted client can lie — Begin's
    `size` is a claim; the per-chunk accumulate-check is the real guard); the client **SHOULD**
    check (fail-fast UX, no wasted multi-MB upload). The node gate is the security boundary; the
    client gate is courtesy.
- **Ceiling source.** Pattern-A is "tier ceiling + tighter immutable-Space override", but there is
  **no tier→size map** and **no immutable-Space type** today (M12.2-A-03). Options: (i) a **flat
  node-config blob ceiling** (`[sync].batch_size` precedent — simplest, Pattern-A "operator-keyed"
  floor, defers the tier table); (ii) introduce the **tier→size map** now; (iii) wire the dormant
  per-Space **`max_event_size`** as the tighter override.
  - *Recommendation:* **(i) flat node-config MB ceiling for M12.2** (the operator-keyed Pattern-A
    floor — e.g. a new `[node].max_blob_bytes`), **+ reserve** the tier→size map and the per-Space
    tighter-override (a create-time stored cap, `max_event_size`-shaped) for later. This is a real
    design call — surface whether the design wants the tier table built now (heavier) or the flat
    floor first (lighter, ship F6's teeth without the §3.1.1 model).

### FK-4 — F9 default location + startup validation

- **Default location.** (a) **platform data dir** (`%APPDATA%` / `$XDG_DATA_HOME` / `~/Library`) —
  needs a new `dirs`/`directories` dep or hand-rolled per-OS (no such dep today); (b)
  **explicit-required** (no silent default — the node refuses to start without `--data-dir`/config,
  forcing an operator choice); (c) a **named convention** (e.g. `~/.xgen/` or a documented fixed
  path outside the install folder).
  - *Recommendation:* surface all three; **lean (a) platform data dir** (the standard, least-surprise
    posture F9 describes — "defaults outside the install/system folder") with the new dep accepted,
    OR (c) a named convention if avoiding the dep matters. (b) explicit-required is the safest
    (no accidental wrong-volume default) but the harshest UX. Joe's call — this is the heart of M12-D7.
- **Override + validation.** A `--data-dir <abs path>` CLI flag resolved in `resolve_data_dir`
  (cannot be a config field — chicken-egg, L20) + startup validation (writable, durable, not a tmp
  dir) — both net-new (L23). The `--check-config` flag (`main.rs`) is the precedent for a startup
  validation gate.

### FK-5 — F9 existing-data handling + `--instance` interaction

- **Existing install-folder data** (identities.db, spaces, blobs at `exe_dir()`): (a) auto-migrate
  on first start; (b) **leave-as-legacy-and-named** (the M10.4-D5 precedent — don't auto-migrate;
  document that existing deployments set `--data-dir <old exe_dir>`, or run a one-time migration).
  - *Recommendation:* **(b) leave-as-legacy-and-named.** Auto-migration of a live node's signed DAG
    + registries is risky; the override (`--data-dir <old exe_dir>`) is a clean opt-in, and a named
    one-time `migrate-data-root` step can be a later sub-arc if wanted. State the boundary honestly.
- **`--instance`:** lock whether `--data-dir X --instance n1` composes to `X/instances/n1` (lean:
  compose — `--instance` stays a sub-segregation under whatever root resolves) or they're mutually
  exclusive.

### FK-6 — Sequencing: split F9 as its own sub-step?

F9 (WI-4) is the heaviest item (default-shift + flag + validation + config-default re-root +
existing-data posture + `--instance`), and it is **orthogonal** to the fetch verb / `--attach`
polish / F6 (those are blob-feature work; F9 is node-ops convention). Options: (a) one M12.2 arc
covering all four; (b) **split** — M12.2a = the lighter blob-feature trio (fetch verb + `--attach`
polish + F6 gate), M12.2b = the F9 data-root posture shift.
- *Recommendation:* **lean (b) split.** The blob-feature trio unblocks the named e2e (the headline
  value) and is low-risk + well-precedented; F9 is a deliberate node-ops convention change with its
  own existing-data + platform-dir + validation decisions (FK-4/FK-5) that don't gate the e2e. A
  split lets the e2e land while F9 gets its own focused lock. Surface it; the design decides whether
  the F9 weight justifies a sub-step.

---

## §5 The e2e witness (discharges the M12.1 boundary)

Grounded in M12.2-A-05: the full self-thread e2e is **reachable via `xgen-mptest`** (real binaries
over `.aicontrol`) once the fetch verb is a routable `ClientCommand`. The M12.2 witness set should
include this real-binary round-trip (spawn node + client → `self` → `send --attach` → `fetch` →
byte-identical assert on disk + JSON result) as the discharge of the deferred boundary; the
raw-WS C5 witnesses stand for the transfer mechanism. The design/runbook firm the exact assertions;
no new crate edge is needed.

---

## §6 Routed / out-of-scope (survive beyond M12.2)

- **M12.3** — federation fetch-blob-by-hash + F3 lazy/eager lock (M12-D8) + Retained durability
  floor + the `10003 blob_unavailable` reject + the held-pending/unavailable client signal.
- **M12.4** — `message.redact` content applier + F2b sender-`Retention` read + crypto-shred
  destroy-to-erase (D3-gated) + the reserved WORM/legal-hold operator/module hook.
- **By-event / by-`blob_ref` fetch granularity** — additive later if a use-case surfaces (FK-1).
- **The §3.1.1 tier→size map + per-Space tighter-override** — if FK-3 takes the flat-ceiling lean,
  these are reserved (the map has no code home today).
- **Combined text + attachment in one `message.file`** — a content-schema question (FK-2c).
- **A `migrate-data-root` step** — if FK-5 takes leave-as-legacy, an operator one-time migration is
  a later named sub-arc.
- **Shared-with-text D3** — client-side `enc:` live-encryption of `message.*` content (R-1); when it
  lands for text it lands for the `message.file` descriptor in the same shape.

---

## §7 Design inputs (the audit → design handoff)

M12.2 is shovel-ready once the six forks are locked. The four work-items grounded:
- **Fetch CLI verb** (WI-1, FK-1) — a D-092 4-arm verb-add over the **built** `ops::fetch_attachments`;
  add `ClientCommand::Fetch` + clap args + `cmd_fetch` + 4 arms + Appendix F. The headline unblock.
- **`--attach` polish** (WI-2, FK-2) — surface-only (multi-file `Vec`, optional text, mime); the
  builder/content/reader are already plural.
- **F6 size gate** (WI-3, FK-3) — `BlobError::TooLarge`→`10002` (reserved) + a gate at the flagged
  `BlobUploadBegin.size` seam (+ per-chunk accumulate); the ceiling source is a real design call
  (flat node-config floor vs tier table vs per-Space `max_event_size`).
- **F9 data-root** (WI-4, FK-4/5/6) — concentrated code-touch (`resolve_data_dir` + config-default
  re-root + `--data-dir` flag + startup validation), total operational impact; existing-data =
  leave-as-legacy lean; **candidate for its own M12.2b sub-step** (FK-6).

The full self-thread e2e (WI-5) is reachable via `xgen-mptest` once the fetch verb is a
`ClientCommand` — the M12.1 boundary discharges here.

**Sequence (Rule 0):** this audit → design (Chat/Joe, lock FK-1..FK-6) → Joe-lock → Clair runbook →
implement → Chat doc-bridge → M12.2 close. No code until the M12.2 design is Joe-locked.

## §8 Entry (Rule 0)

`CLAUDE.md` PLAY → `JOURNAL.md` J-382 → `tasks/M12_ATTACHMENTS_DESIGN.md` v1.1 (M12-D10 scope /
M12-D7 F9 / M12-D9 reject band) → `tasks/M12_1_BLOB_STORE_ATTACH_IMPL.md` (COMPLETED — the M12.1
shipped state + the named e2e boundary) → this audit → `tasks/M12_ATTACHMENTS_PHASE0_AUDIT.md`
(the M12-wide grounding ledger) → `docs/ROADMAP.md` (M12).
