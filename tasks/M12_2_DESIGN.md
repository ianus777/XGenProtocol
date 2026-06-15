# M12.2 — Fetch verb + --attach polish + F6 gate + F9 data-root: Design (Joe-LOCKED)
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

The Joe-LOCKED M12.2 design, authored by Chat at the design-lock (J-383) after discussing the
six audit forks and locking each by-recomms. Sits on the M12.2 D-071 Phase-0 audit
(`tasks/M12_2_FETCH_GATE_DATAROOT_PHASE0_AUDIT.md`, GO, findings M12.2-A-01..05, ledger L1–L25,
forks FK-1..FK-6) and the M12-wide design (`tasks/M12_ATTACHMENTS_DESIGN.md` v1.1, M12-D10 = the
M12.2 scope; M12-D7 = the F9 posture-but-decoupled lock; M12-D9 = the blob reject band).

D-071 arc flow: this design → Clair authors the M12.2a runbook → implement → Chat doc-bridge →
M12.2a close → M12.2b (its own runbook) → M12.2 close. **No code precedes the runbook.**
Decisions are arc-local (D-069). One grounding hinge re-verified by Chat (D-065): `data_dir` is
resolved **before** config load (`try_load_config` runs with `data_dir` already passed in), so a
root-`data_dir` override must be a flag/env, not a config field (M12.2-D5).

---

## §2 Locked decisions (M12.2-D1..D6)

### M12.2-D1 — M12.2 splits: M12.2a (blob-feature trio + e2e) → M12.2b (F9 data-root) (FK-6)

M12.2 splits into two sub-arcs, each its own runbook:

- **M12.2a** — the **blob-feature trio**: fetch verb (D2) + `--attach` polish (D3) + the F6 size
  gate (D4). Lands the **full self-thread e2e** (discharges the M12.1 honest boundary). High
  value, low risk, client-feature-shaped.
- **M12.2b** — the **F9 data-root posture shift** (D5 + D6). A breaking node-ops default change +
  startup validation + legacy handling — orthogonal to the trio and isolated so it doesn't mix a
  node-ops migration concern into the client-feature work, nor delay the e2e.

M12.2a first; M12.2b second. M12.2 closes when both close.

### M12.2-D2 — Fetch CLI verb (FK-1)

`ops::fetch_attachments` is **built but has zero callers** (C4) — so this is a thin **4-arm
verb-add, no core/wire change**. Shape:

- A new `fetch` / `fetch-attachments` `ClientCommand` with a **by-message/thread selector**
  (mirror `history`); it reuses the built op, which already loops **all** attachments in scope.
- **Output to a path/dir**, naming files from the `Descriptor` filename (binary → never stdout).
- Wire all **four D-092 arms** (CLI `main.rs` / run-path `app.rs` / batch `batch.rs` / aicontrol
  `aicontrol.rs` via `reconstruct_argv`) + an **Appendix F** entry (a new client verb).
- `FetchAttachmentsArgs` gets a clap derive (it is hand-built today).

Exact selector grain + the output-dir convention are runbook details; the shape above is locked.

### M12.2-D3 — `--attach` surface polish (FK-2)

**Surface-only** (the C2 builder takes `&[Descriptor]`; the fetch reader loops; mime is
client-side). Lock **both**, together:

- **Multi-file** `--attach` (the `Descriptor` list is already plural end-to-end).
- **Attach-only** sends — make `text` optional on `SendArgs` so a `message.file` can carry no
  `message.text`.

Client-side only; no core/wire change.

### M12.2-D4 — F6 blob size gate (FK-3)

**Placement = both, node-authoritative.** The node rejects at **`BlobUploadBegin.size`** (the
field is carried-but-discarded today — the fail-fast hook), returning the **reserved `BlobError`
`10002` `blob_too_large`** *before* accepting chunks; the client also checks locally pre-upload
for UX. The node gate is the real one (the node can't trust the client).

**Ceiling source = a flat operator node-config ceiling now** (the `[sync].batch_size` precedent
shape — a new node-config numeric field). The Pattern-A **tier→size table** + the **tighter
immutable-Space override** (F6's full shape) are **reserved as the named Pattern-A enrichment**,
**not built in M12.2** — there is no tier→size map and no immutable-Space type today, and the
gate *mechanism* (enforce a ceiling, reject `10002` at `BlobUploadBegin`) does **not** reshape
when the source later grows from flat-operator to tier-keyed. Mechanism-first; source grows
without rework.

### M12.2-D5 — F9 default location + override (FK-4)

The F9 posture (M12-D7: default **outside** the install folder, operator-overridable,
startup-validated) lands here:

- **Override = a `--data-dir` flag + env var** (forced by the verified before-config-load
  ordering — a config field can't carry the root).
- **Default = a platform data dir** (cleanly outside the install folder), implemented
  **hand-rolled** (`%LOCALAPPDATA%` on Windows / `$XDG_DATA_HOME` (or `~/.local/share`) on
  Linux, with documented fallbacks) to **avoid a new `dirs`-crate dependency**.
- **Startup validation** (net-new): the resolved root must be present-or-creatable, writable,
  and not a temp path — **fail fast at startup** with a clear error.
- `--instance` (`<root>/instances/<label>`) rebases under the **resolved** root.

### M12.2-D6 — F9 existing-data handling (FK-5)

**Leave-as-legacy + named** (the M10.4-D5 precedent): pre-existing `exe_dir()`-rooted
deployments stay where they are, **documented**, with a `--data-dir=<old path>` **named escape**.
**No auto-migration** — moving a live node's data is disproportionate risk for a reference impl.
The new platform-dir default applies to **fresh** deployments.

---

## §3 M12.2a slice (the e2e; first runbook)

The blob-feature trio that discharges the M12.1 boundary. Implementable shape:

- **D2** the `fetch` verb wraps `ops::fetch_attachments` (clap-derive its args; 4 D-092 arms +
  Appendix F).
- **D3** `SendArgs` gains multi-`--attach` + optional `text`; `ops::send` threads the list (the
  builder is already plural).
- **D4** the node enforces the flat config ceiling at `BlobUploadBegin` (reject `10002`); the
  client pre-checks.

**Witness — the full self-thread e2e (discharges the M12.1 named boundary):** via `xgen-mptest`
driving the **real binaries** over `.aicontrol` — `self` → `send --attach <file>` → `fetch`
retrieves it **byte-identical** by a second same-identity client. Reachable once the fetch verb
is a `ClientCommand` (no new crate edge; `xgen-mptest` already spawns `xgen-node.exe` /
`xgen-client.exe`). + W-multi (multi-file round-trip) + W-toolarge (F6 `10002` reject), RED-on-revert.

## §4 M12.2b slice (F9 data-root; second runbook)

The posture shift (D5 + D6), isolated:

- `resolve_data_dir` gains the `--data-dir`/env override + the platform-dir default (hand-rolled)
  + startup validation; the `NodeConfig::default` `exe_dir()` rooting is retired as the *default*
  (legacy deployments keep it via the named `--data-dir` escape).
- Code-touch is **concentrated** (`resolve_data_dir` + the `NodeConfig::default` root + the new
  flag) — the ~14 `data_dir.join(...)` consumers **inherit** unchanged.
- **Witness:** fresh node defaults outside the install folder; `--data-dir` override honored;
  startup validation rejects a non-writable/tmp root; a legacy `exe_dir` layout still starts via
  `--data-dir=<old>`.

## §5 Reserved / out (named, not built in M12.2)

- **Pattern-A tier→size table + per-Space immutable override** (F6's full shape) — the named
  Pattern-A enrichment; the M12.2 gate mechanism accepts it without rework.
- **Auto-migration** of existing `exe_dir` data — declined (D6); the `--data-dir` escape covers it.
- A `dirs`-crate dependency — declined for M12.2 (hand-rolled platform lookup).

## §6 Sequence

this design → Clair authors `tasks/M12_2a_*_IMPL.md` (trio) → implement → Chat doc-bridge →
M12.2a close (the e2e witnessed) → Clair authors `tasks/M12_2b_*_IMPL.md` (F9) → implement →
Chat doc-bridge → M12.2b close → **M12.2 close** → M12.3 (federation) → M12.4 (erasure). No code
until each runbook lands.

## §7 Entry (Rule 0)

`CLAUDE.md` PLAY → `JOURNAL.md` J-383 → this design → `tasks/M12_2_FETCH_GATE_DATAROOT_PHASE0_AUDIT.md`
(findings/ledger/forks) → `tasks/M12_ATTACHMENTS_DESIGN.md` v1.1 (M12-D10/D7/D9) → the M12.1
runbook (COMPLETED) → `docs/ROADMAP.md` (M12).
