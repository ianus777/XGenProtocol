# M6 Phase 2 — admin_ops scaffolding + EventAccepted + rejection correlation
> **Status**: COMPLETED  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

The foundational scaffolding phase of M6 (new), per `tasks/HANDOFF_M6_IMPL.md` §"Phase 2",
the canonical design `docs/xgen_node_admin_ops_design.md` §5.2 + §3, and the audit
`docs/xgen_propagation_reliability.md` §6.5. Every subsequent phase (3–10) adds one
category's verbs against this scaffolding.

This phase ships the only **protocol-level** change in M6 (`TransportMessage::EventAccepted`)
plus the reference-implementation skeletons (`admin_ops`, `audit`) the verb phases consume.

## Locked decisions feeding this phase

- **`event_id` realisation — Joe-confirmed 2026-05-29: per-variant + accessor (Option 1).**
  `event_id` is a wire constraint (top-level optional field, audit §6.5 / D-070), not a
  struct mandate. The internally-tagged `TransportMessage` flattens variant fields to the
  top level, so a per-variant field is byte-identical on the wire to an envelope wrapper.
  Chosen over the wrapping-struct refactor (which would rewrite ~81 load-bearing sites
  across auth/sync/federation and lean on serde `flatten` + internally-tagged enum, a known
  sharp edge). The drift-surface objection (design reason #1) is neutralised by reading
  `event_id` through **one accessor** `TransportMessage::event_id(&self) -> Option<&str>` —
  the correlation consumer calls that; the match lives in exactly one place. A future
  cross-cutting envelope field (trace_id, sent_at, …) would justify the wrapper as its own
  scoped refactor with its own test pass — not a rider on this foundational phase.
- **SQLite for audit** (§2.6.4, Joe-lock #3) — `rusqlite` with the `bundled` feature
  (no system sqlite needed on Windows). DB at `<data_dir>/xgen-node_audit.db` (D-035).
- **`admin_ops::*` is the single source** (D-067); `--batch` and future `--aicontrol` (M7)
  both call it. **Terminology** (D-082): administrator/admin, never operator.

## Commit sequence

| # | Scope | Crate(s) | Status |
|---|---|---|---|
| 1 | Wire shape: `EventAccepted` variant + `event_id` on `Error` + `event_id()` accessor + serde/backward-compat tests | xgen-core | ✅ (4 tests; 643 lib green) |
| 2 | `admin_ops` skeleton (`AdminContext`, `AdminError` w/ stage + error-code bands; no verbs yet) + `audit` skeleton (`rusqlite` bundled; `audit_entries` schema; `AuditEntry` + insert/query API) + lib.rs module decls + tests | xgen-node | ✅ (9 tests; rusqlite bundled) |
| 3 | `EventAccepted` emission in `process_inbound` (after persist, before fan-out) + rejection paths populate `Error.event_id` | xgen-node | ✅ (pure `accept_signal`/`reject_signal` helpers + 5 tests; 102 lib) |
| 4 | Client-side `EventAccepted` plumbing — match arms in the receive loops recognise it (pure plumbing per §5.2.4; full waiting/correlation behaviour deferred per-verb) | xgen-client | ✅ (7 receive loops; debug-log recognition) |
| 5 | `pipe::dispatch_line` structural seam for future `admin_ops::*` write-verb routing (read-only allowlist preserved unchanged; no write verbs yet) | xgen-node | ✅ (documented seam) |
| 6 | Phase close: this file → COMPLETED + Ch3 §3.3.10 + Appendix I doc sync + CLAUDE PLAY + JOURNAL J-153 + ROADMAP | docs | ✅ |

Commits are logical, atomic units; Joe commits/pushes (Claude does not push). Verification
runs per commit.

## Wire shape (Commit 1) detail

```rust
// in TransportMessage (xgen-core/src/wire/types.rs)
#[serde(rename = "transport.error")]
Error {
    protocol_version: String,
    error_code: u32,
    error_string: String,
    timestamp: String,
    // M6 §3.3: populated with the rejected event's hash URI when the error pertains
    // to a specific event; None for transport-level errors (malformed framing, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    event_id: Option<String>,
},
/// M6 §3.1/§3.2 — positive acceptance signal, sibling to Error. Sent to the
/// originator after the event is validated AND persisted, before fan-out (G2).
#[serde(rename = "transport.event_accepted")]
EventAccepted {
    protocol_version: String,
    event_id: String,      // always pertains to an event → required, not Option
    accepted_at: String,   // RFC 3339 UTC
},
```

`EventAccepted.event_id` is required (it always pertains to an event); `Error.event_id` is
optional. The accessor normalises both:

```rust
impl TransportMessage {
    pub fn event_id(&self) -> Option<&str> {
        match self {
            TransportMessage::EventAccepted { event_id, .. } => Some(event_id),
            TransportMessage::Error { event_id, .. } => event_id.as_deref(),
            _ => None,
        }
    }
}
```

## Definition of Done

- [x] `EventAccepted` variant + `Error.event_id` + `event_id()` accessor; serde round-trips; pre-M6 `Error` JSON (no `event_id`) still deserialises (backward-compat test).
- [x] `admin_ops` module: `AdminContext`, `AdminError` (stage taxonomy §2.6.5 + error-code-band shape §2.7); compiles; no verbs.
- [x] `audit` module: `rusqlite` bundled; `audit_entries` table created empty on first open; `AuditEntry` insert + query round-trip test.
- [x] `process_inbound` emits `EventAccepted` after persist, before fan-out, with `event_id` = accepted event's id; rejection paths set `Error.event_id` (newly emit `Error` on rejection — J-081 §5 gap closure).
- [x] Client receive loops recognise `EventAccepted` (plumbing; debug-log, no fallthrough-to-unknown).
- [x] `pipe::dispatch_line` seam in place; read-only allowlist unchanged.
- [x] `cargo test --workspace` green (657 lib + 25 integration, 0 failed); clippy `-D warnings` clean; `cargo build --workspace --all-targets` 0 errors.
- [x] Ch3 §3.3.10 + Appendix I gain the `EventAccepted` wire entry + `Error.event_id` note (doc sync).

## Verification (close)

- `cargo test --workspace`: 657 lib (63 client + 35 common + 457 core + 102 node) + 25 integration; 0 failed. +18 lib vs the 639 Phase-2-start baseline: +4 wire (core) + 9 admin_ops/audit (node) + 5 accept/reject-signal (node).
- `cargo clippy --workspace --lib --tests --all-features -- -D warnings`: clean.
- `cargo build --workspace --all-targets`: 0 errors.

## Next

Phase 3 — read-only completions on existing `--batch`; then Phases 4–10 per category.
Phase 9 stays design-gated (`membership.node_eject` sub-design first).

---

*End of Phase 2 plan.*
