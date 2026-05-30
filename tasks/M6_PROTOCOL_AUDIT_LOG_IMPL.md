# Protocol-Audit-Log — Implementation Runbook (D-071 arc)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-30  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Framing

Implementation runbook for the **protocol-audit-log** D-071 arc. Design is locked at
`tasks/M6_PROTOCOL_AUDIT_LOG_DESIGN.md` v1.0 (PAL-D1/D2/D3, J-164); audit (the
reality map) at `tasks/M6_PROTOCOL_AUDIT_LOG_AUDIT.md`; spec at
`docs/xgen_ch3_specification.md` §3.11.8; verb spec at
`docs/xgen_node_admin_ops_design.md` §6.A4 (A4-D3).

Reading order at pickup (Rule 0): CLAUDE.md PLAY → latest JOURNAL → this runbook
§1–§2 → the design doc → §3+ per commit.

**The load-bearing work is the writer side** (Commit 1) — the protocol-pipeline
hook + store. The reader and rebuild are comparatively mechanical.

**PAL-D1 site refinement (code-trace at authoring, J-165).** The single writer hook
goes **inside `xgen-node::app::persist_event`**, the true persist chokepoint — every
accept path funnels through it (`process_inbound` Accepted arm + the two drain loops
+ the M6 admin write-path), and it already has a per-`event_id` dedup guard.
`replay_spaces_from_dir` uses `runtime.ingest_event` (in-memory) and does **not**
call `persist_event`, so a persist-layer hook never re-fires on restart — no
duplicate entries, no replay-skip logic needed. This is the no-drift-surface choice
(D-067): one site no future author can forget. Confirmed by Joe (J-165) over the
"beside-it in process_inbound" alternative.

**Doc-pass folded into authoring (this atomic, J-165, Chat Claude):** runbook shipped
+ design doc ACTIVE → COMPLETED v1.1 + CLAUDE PLAY + ROADMAP + JOURNAL. Clair's
sequence below starts at Commit 1 (writer).

## §2 Sequence overview

| Commit | Scope | Crate(s) | Class |
|---|---|---|---|
| **1 — Writer** | `ProtocolAuditEntry` + monthly JSONL store; hook inside `persist_event` (thread audit-dir + node_id); 11-EventType match; loud failure | xgen-node | LOAD-BEARING |
| **2 — Reader** | `space audit-events` (`admin_ops` + clap `SpaceCommand::AuditEvents` + pipe arm); read-time space filter; cross-month pagination | xgen-node | mechanical |
| **3 — Rebuild** | `space audit-rebuild` (PAL-D3) — replay DAG → (re)generate entries, dedup; one Space / all | xgen-node | mechanical |
| **4 — Close** | arc DONE; runbook COMPLETED; `audit-events`+`audit-rebuild` → SHIPPED in §6.A4 + backing audit; PLAY → federation-admin-control | docs | close |

**Joe-lock checkpoints:**
- **#1 (pre-Commit-1):** the `persist_event` signature change shape + **the 11-EventType list and per-type field mapping approved by name** (§4 table; D-078 production-grounded enumeration — confirm each type+field exists against current `Event` content before code).
- **#2 (post-Commit-1 / pre-Commit-2):** writer drift check (replay-no-dup + loud-failure tests green; 11-only-types verified).
- **#3 (Commit 3):** PAL-D3 `space audit-rebuild` behaviour — scope (one/all), dedup approach, no startup-reconcile in v1.

**Verification rigour:** every commit — `cargo test --workspace` green, `cargo clippy --workspace --lib --tests -- -D warnings` clean, `cargo build --workspace --all-targets` 0 errors. Commit 1 (milestone-bearing) + Commit 4 also run isolated re-runs of the new tests.

**DoD note (per project discipline):** no checklist contains "commit pushed" (unflippable inside the commit that pushes); `Status: COMPLETED` is the real signal. ROADMAP + CLAUDE update in the same commit as any state change (D-074 atomic).

## §3 Scope guards

In: `ProtocolAuditEntry`, monthly JSONL store, the `persist_event` writer hook (11
types), `space audit-events` reader, `space audit-rebuild`. Out (do NOT broaden
without Joe-lock): hash-chain tamper-evidence; the Auth-Module audit log; automatic
/ startup rebuild; changing `persist_event`'s existing silent event-write best-effort
(only the *audit* write is loud per PAL-D2).

## §4 Commit 1 — Writer side (LOAD-BEARING)

**New module** `xgen-node/src/protocol_audit.rs` (sibling to `audit.rs`, the A6
SQLite trail — keep them distinct; doc-comment must state "NOT the A6 admin trail").

**`ProtocolAuditEntry`** — one JSON object per line. Universal fields always present:
`ts` (RFC 3339 UTC, ms), `event_type` (String), `event_id` (the hash URI), `node_id`.
Plus EventType-specific summary fields. Suggested shape: the four universal fields as
struct fields + `#[serde(flatten)] extra: serde_json::Map<String, Value>` populated by
a `from_event(event, node_id)` builder. Summary facts only — full Event recovered from
the DAG via `event_id`.

**The 11 EventTypes + per-type fields (§3.11.8) — Joe-lock #1 list:**

| EventType | Extra fields (beyond ts/event_type/event_id/node_id) |
|---|---|
| `membership.join` | identity_id, space_id, approving_node_id |
| `membership.leave` | identity_id, space_id |
| `membership.invite` | inviter_id, invitee_id, space_id |
| `membership.kick` | kicker_id, kicked_id, space_id, reason? |
| `membership.ban` | banner_id, banned_id, space_id, reason? |
| `state.space_create` | creator_id, space_id, auth_tier |
| `state.room_create` | creator_id, room_id, space_id |
| `state.federation_add` | initiating_node_id, receiving_node_id, space_id |
| `state.federation_remove` | node_id, space_id, reason |
| `identity.register` | identity_id, home_node_id, tier_verified |
| `system.key_rotation` | identity_id, old_key_hash, new_key_hash |

Any EventType **not** in this table (e.g. `message.*`, `membership.node_eject`/`node_unban`, `state.dm_*`) is **not** audited. Clair verifies each field name against the live `Event` content structs before coding (D-078); report any mismatch at checkpoint #1 rather than guessing.

**Store.** Append-only, one file per calendar month at `<audit_dir>/protocol_audit_YYYY-MM.jsonl`, where `<audit_dir>` = the Node working-dir `audit/` (derived by convention like `spaces_dir`/`log_path`, D-035 — confirm the exact resolver, sibling to `resolve_spaces_dir`). Month derived from the entry's `ts` (not `now()`), so an event's line lands in its own month's file. Append one line (open in append mode, write `entry_json + "\n"`). `MUST NOT` auto-delete (no rotation-by-deletion; rotation = new file on month boundary only).

**The hook (PAL-D1).** Inside `persist_event`, **after** the dedup guard's early-return
(so only first-write of an `event_id` audits — idempotent by construction) and after
the event file write: if `event.event_type` ∈ the 11, build the entry and append.
Thread `audit_dir: &Path` + `node_id: &str` into `persist_event`'s signature; update
all call sites (`process_inbound` main + drained loop + federation-drain loop + the M6
admin write-path) + test fixtures.

**Loud failure (PAL-D2).** The audit append is best-effort *for protocol liveness* but
**never silent**: on write error, `tracing::error!` + increment a process-global
counter (e.g. `AtomicU64` `protocol_audit_write_failures`). The event still persists;
the gap is recoverable via Commit 3 rebuild. (Contrast: `persist_event`'s existing
event-write stays silent best-effort — unchanged.)

**Tests (xgen-node):**
1. `protocol_audit_entry_serde_jsonl_roundtrip` — a sample across the 11 types serialises to a single valid JSON line with the universal + extra fields.
2. `monthly_file_path_derived_from_entry_ts` — May ts → `protocol_audit_2026-05.jsonl`.
3. `persist_writes_audit_entry_for_listed_eventtype` — persisting a `membership.join` appends one line.
4. `persist_skips_audit_for_unlisted_eventtype` — a `message.*` / `node_eject` persists but writes **no** audit line.
5. `replay_does_not_write_audit_entries` — `replay_spaces_from_dir` over a populated dir leaves the audit file unchanged (no dup-on-restart).
6. `audit_write_failure_is_loud_not_swallowed` — unwritable audit dir → counter increments + error logged, event still persisted.

## §5 Commit 2 — Reader (`space audit-events`)

`admin_ops::space_audit_events(ctx, args)`; clap `SpaceCommand::AuditEvents`; pipe
dispatch arm. **Args** (`SpaceAuditEventsArgs`): `space_id`, `event_type: Option`,
`since/until: Option<String>` (RFC 3339), `limit: Option<usize>` (default 100),
`cursor: Option<String>`. **Result** `SpaceAuditEventsResult { events: Vec<ProtocolAuditEntry>, returned: usize, next_cursor: Option<String> }`.

Read-time filtering (PAL-D1): scan the month file(s) covering `since..until` (or the
current month if unbounded), parse each line, keep entries whose `space_id` matches +
optional `event_type`/time filters; paginate by `limit`/`cursor`. Cross-month: iterate
month files in chronological order. **Errors:** `SPACE_8001` Space not hosted/federated
here; `SPACE_8010` bad filter; `GENERIC_4000`. READ → **not** audited (A4-D3).

**Tests:** space_id filter; event_type filter; since/until range; limit+cursor pagination; cross-month read; empty result; bad-filter → 8010.

## §6 Commit 3 — Rebuild (`space audit-rebuild`, PAL-D3)

`admin_ops::space_audit_rebuild(ctx, args)`; clap `SpaceCommand::AuditRebuild`; pipe
arm. **Args** (`SpaceAuditRebuildArgs`): `space_id: Option<String>` (None → all
hosted/federated Spaces), `dry_run: Option<bool>`. **Behaviour:** for each in-scope
Space, read its persisted events, and for each of the 11 types whose `event_id` is
**not** already present in the audit log, append the entry (dedup against existing
log lines → re-runnable/idempotent). Closes PAL-D2 gaps **and** backfills cold-start
Spaces (events predating the log). **Operator-invoked only — no startup/automatic
reconcile in v1** (PAL-D3). **Result** `SpaceAuditRebuildResult { spaces_scanned, entries_added, entries_already_present }`. WRITE → audited in the A6 trail (correlation to the rebuild action). **Errors:** `SPACE_8001` (when a named Space isn't hosted/federated here); `GENERIC_4000`.

**Tests:** gap recovery (remove a line → rebuild restores exactly it); idempotent (rebuild twice → second adds 0); rebuild-all; cold-start backfill (events present, audit empty → all 11-type entries generated); dry_run adds nothing but reports counts.

## §7 Commit 4 — Milestone close

- Runbook Status ACTIVE → COMPLETED v1.0 → v1.1.
- `docs/xgen_node_admin_ops_design.md` §6.A4: `audit-events` SHIPPED; add the `audit-rebuild` verb (A4-D3 sibling). `tasks/M6_BACKING_AUDIT.md`: `audit-events` ABSENT → SHIPPED + `audit-rebuild` row (the protocol-audit-log arc closes).
- CLAUDE PLAY flip → next arc **federation-admin-control**. ROADMAP arc row 🟢 → ✅ + Present flip. JOURNAL milestone-close entry.
- Verification: full workspace green + clippy + build; isolated re-runs of the Commit 1 + Commit 3 tests.

## §8 Discipline notes

- **No-drift-surface (D-067):** the single `persist_event` hook is the whole point — do not add parallel audit-write calls elsewhere; if a new accept path appears, it already routes through `persist_event`.
- **Honest best-effort (D-065 / D-070 / PAL-D2):** audit-write failure is loud (counter + error), never swallowed; the rebuild verb is the honest recovery, not a hidden auto-retry.
- **D-078 production-grounded enumeration:** the 11-type field mapping (§4) is approved by name at checkpoint #1 against live `Event` content — verify, don't assume.
- **A6 vs §3.11.8:** `protocol_audit.rs` is NOT `audit.rs`. Different store (JSONL vs SQLite), different content (protocol Events vs admin-verb actions), different reader. Doc-comment both modules to say so.

## §9 Cross-refs

- Design: `tasks/M6_PROTOCOL_AUDIT_LOG_DESIGN.md` v1.1. Audit: `tasks/M6_PROTOCOL_AUDIT_LOG_AUDIT.md`.
- Spec §3.11.8; verb spec `docs/xgen_node_admin_ops_design.md` §6.A4 (A4-D3) + Appendix K.2.6.
- Code: `xgen-node/src/app.rs` (`persist_event` ~2962, `process_inbound` ~1707, `replay_spaces_from_dir` ~3048, `resolve_spaces_dir` ~2785); `xgen-node/src/audit.rs` (the DISTINCT A6 trail); `xgen-common/src/event_trace.rs` (debug layer).
- D-071 / D-069 / D-065 / D-067 / D-070 / D-074 / D-078.

---

*Implementation runbook. Clair's sequence: Commit 1 (writer) → 2 (reader) → 3 (rebuild) → 4 (close). Doc-pass folded into authoring (J-165).*
