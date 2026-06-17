# M6 Phase 1 — Client gap patches (R1)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

First implementation chunk of M6 (new), per `tasks/HANDOFF_M6_IMPL.md` and the M6 design
`docs/xgen_node_admin_ops_design.md` §5.1. Phase 1 closes the Client-side gap that Pass 1
(`tasks/CLIENT_BATCH_AUDIT_M6.md`) surfaced: two Appendix-F-documented Client commands
(`rooms`, `members`) that were never implemented in code, plus `federate`.

## Scope decision (Joe-locked 2026-05-29)

Pass 1 offered R1/R2/R3. Joe confirmed **R1** (do `rooms` + `members` first), with the
schema-check caveat the audit itself flagged ("`ClientState.spaces[i].members[]` *or the
equivalent — schema check during implementation*"):

- **`rooms`** — **SHIPPED.** A genuine zero-network local read. Data lives in
  `KnownSpace.rooms[]` (`KnownRoom { room_id, name, joined }`), same shape as `spaces`.
- **`members`** — **DEFERRED** (Joe-locked 2026-05-29). The schema check found that
  `xgen-client_state.json` persists **no per-member data** at all (`KnownSpace` =
  `{space_id, name, node_endpoint, role, rooms}`; only a Node-side `member_count` exists).
  Appendix F §F.5.6 shows `members` output with pubkey + display name + role +
  "registered Nm ago" and marks it network=No — but that output cannot be produced from
  disk today. `members` therefore is **not** the trivial local read R1 assumed. It re-enters
  as its own scoped piece needing either (a) a Node query or (b) a `KnownSpace.members[]`
  state-schema expansion populated on join/invite/history replay. Recorded per Rule 6 / D-065.
- **`federate`** — **DEFERRED to M6 Phase 7** (R2, already settled). Co-designed with the
  Node-side federation-management verbs (A1).

## What shipped (`rooms`)

Mirrors `spaces` exactly across the single-source `ops` layer (D-067) and all three dispatchers:

| Site | File | Change |
|---|---|---|
| ops layer (canonical) | `xgen-client/src/ops.rs` | `RoomsResult` struct + `pub fn rooms(ctx, &RoomsArgs) -> Result<RoomsResult>` (finds Space by id, errors if absent) |
| command enum + args | `xgen-client/src/app.rs` | `ClientCommand::Rooms(RoomsArgs)` + `RoomsArgs { space: String }` |
| CLI shim | `xgen-client/src/app.rs` | `cmd_rooms` — formats per Appendix F §F.5.6 |
| dispatcher: CLI arm | `xgen-client/src/main.rs` | `Some(Rooms(args)) => app::cmd_rooms(args, &data_dir)` |
| dispatcher: batch driver | `xgen-client/src/app.rs` (`run_batch_file`) | `Some(Rooms(args)) => cmd_rooms(args, data_dir)` |
| dispatcher: pipe | `xgen-client/src/batch.rs` (`dispatch_line`) | calls `ops::rooms` directly (OK/ERROR only) |
| tests | `xgen-client/src/ops.rs` | `rooms_returns_rooms_for_matching_space` + `rooms_errors_on_unknown_space` |
| doc | `docs/xgen_appendix_f_en.md` | `rooms` shipped; `members` deferred; `federate` → Phase 7; header v1.3 → v1.4 |

## Verification

- `cargo test -p xgen-client --lib`: **63 passed; 0 failed** (was 61; +2 `rooms` tests).
- `cargo clippy -p xgen-client --lib --tests --all-features -- -D warnings`: clean.
- `cargo build --workspace --all-targets`: 0 errors (other `ClientCommand` matches use catch-all arms).
- Live happy path (`rooms --space <known>`):
  ```
  Rooms in Project Alpha (1)

    general
    ID: xgen://hash/sha256:9cb9acbef972
  ```
  matches Appendix F §F.5.6.
- Live error path (`rooms --space <unknown>`): `error: no known Space with ID xgen://hash/sha256:nope`.

## Definition of Done

- [x] `rooms` routes through `ops::rooms` (single source, D-067); all three dispatchers call it.
- [x] `RoomsArgs { space }` clap-wired; `rooms --help` registered on the live binary.
- [x] Output matches Appendix F §F.5.6.
- [x] Unit tests cover match-found + unknown-Space-errors.
- [x] Clippy + workspace build clean.
- [x] `members` deferral + `federate` → Phase 7 recorded in Appendix F.
- [x] JOURNAL entry written with real verification output.

## Next

Phase 2 — `admin_ops::*` scaffolding + `TransportMessage` envelope `event_id` + `EventAccepted`
+ rejection paths (M6 design §5.2; handoff §"Phase 2"). Per-phase task file:
`tasks/M6_PHASE_2_IMPL.md` when opened.

---

*End of Phase 1.*
