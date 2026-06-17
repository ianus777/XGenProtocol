# Design + Runbook — Thin-verb Arc 4: `thread`×3 (MP-C-13 / PG-08)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Status

Phase-0 complete ([THREAD_VERB_AUDIT.md](THREAD_VERB_AUDIT.md) v1.0). F-TH-1..4
**Joe-LOCKED** (2026-06-10) by recommendation. Design + runbook folded; greenlit
to impl. The last thin-verb arc → after close, the **R1 rerun**.

## 2. The four locks (Joe, 2026-06-10)

| Fork | Lock |
|------|------|
| **TH-D1 — verb shape** | `thread` **subcommand group** (`create`/`resolve`/`archive`), mirror `ai` (`ClientCommand::Thread(ThreadArgs{command: ThreadCommand})`). **D-092: 4 dispatch arms** — one outer `Thread(args)` arm per dispatcher (main · run-path · batch · aicontrol) inner-routing the 3 actions, exactly as `ai`. |
| **TH-D2 — oracle** | **both-halves**: positive (3 thread events converge, asserted via transcript) + enforcement (a non-admin member's `thread resolve` → ChangeInfo `PermissionDenied` → assert-the-reject, inherits MP-F5). |
| **TH-D3 — topology** | single-node (Layer-5c resolution is node-local). |
| **TH-D4 — positive observation** | transcript (the 3 thread events present + converged on every node); the harness has no `ThreadState` projection. Layer-5c winner-selection is unit-proven. |

No DECISIONS change (TH-D# arc-local, D-069; D-092 already promoted).

## 3. Change surface (mirror `ai` subcommand group; FOUR dispatch arms)

1. **clap** (app.rs): `ThreadArgs { #[command(subcommand)] command: ThreadCommand }`; `ThreadCommand` (Subcommand) = `Create(ThreadCreateArgs)` / `Resolve(ThreadStatusArgs)` / `Archive(ThreadStatusArgs)`.
   - `ThreadCreateArgs { space, room, title: Option<String>, auth_tier_min: u32 (default 1) }`.
   - `ThreadStatusArgs { space, room, thread }`.
   - `ClientCommand::Thread(ThreadArgs)`.
2. **ops** (ops.rs): `thread_create` (build ThreadCreate, sign, derive `thread_id = thread_id_from_event_id(create_event_id)` [pub], send-confirm → `ThreadCreateResult { event_id, thread_id, space_id, room_id }`); `thread_resolve` / `thread_archive` (build ThreadResolved/Archived with `--thread`, send-confirm → `ThreadStatusResult { event_id, thread_id, space_id, room_id }`). All via `apply_single_event_confirm` (MP-F5 site). `prev_events` = `get_dag_tips(space)`.
3. **shims** (app.rs): `cmd_thread_create` / `cmd_thread_resolve` / `cmd_thread_archive` (mirror `cmd_ai_*`).
4. **4 dispatch arms**: main.rs · app.rs run-path · batch.rs · aicontrol.rs — each one outer `Thread(args)` arm matching `args.command` → the 3 sub-actions (mirror the `ai` arms verbatim).

**Wire-neutral** (builders shipped Arc E). Authoring: ThreadCreate = Room membership + tier (Tier-1 member OK); resolve/archive = ChangeInfo (Admin+).

## 4. Witness — MP-C-13 (C5, single-node) + RED-on-revert

Scenario (2 actors): alice (owner) creates S + room → invites bob (member) → bob
joins room → alice `thread create` (bind thread_id) → alice `thread resolve` →
alice `thread archive`; bob attempts `thread resolve` on the same thread.

**Oracle (TH-D2, both halves):**
- **positive (TH-D4):** alice's create + resolve + archive events all present in
  the node's cooperative event set (converged; final status deterministically
  Archived via Layer-5c);
- **enforcement (assert-the-reject, MP-F5):** bob's `thread resolve` reply is an
  Error with `reject_code` (ChangeInfo PermissionDenied → **4000**, pin
  empirically — MP-A-20/MP-C-08 precedent, MP-F2-followon) + `event_id`; bob's
  resolve event absent everywhere.

**RED-on-revert:** neuter `ops::thread_resolve`/`_archive` (or, for the positive
half, `ops::thread_create`) — the simplest faithful revert: make the enforcement
op not reach the gate so the positive events don't converge OR bob's reject
doesn't fire. Concretely: revert `ops::thread_archive` to a no-op-shaped event →
the archive event is absent → positive convergence assert (archive present) RED.
(Both halves have a genuine revert; pick per impl — the enforcement half's revert
= bob's resolve no longer rejected.)

## 5. Runbook (single commit)

1. clap (`ThreadArgs`/`ThreadCommand` + 2 arg structs + `ClientCommand::Thread`) + `ops::thread_{create,resolve,archive}` + Results + 3 `cmd_thread_*` shims + **4 dispatch arms**.
2. `MP-C-13/*` batch (alice owner: create-space, create-room, invite bob, thread create/resolve/archive; bob: join room, thread resolve [rejected]) + manifest (single-node; waits: bob joins after invite; alice's thread ops sequential; bob's resolve after thread created; export thread_id from alice's create).
3. `mp_r1_c5::mp_c_13_*` runner: positive (3 thread events on node) + enforcement (bob resolve assert-the-reject, reject_code pinned + absent).
4. Appendix F `thread create`/`resolve`/`archive` entries.

**Verification:** build 0 + clippy clean (default + `--all-features` + `--features harness-control`); fast suite green; MP-C-13 heavy GREEN; **empirically pin** bob's resolve reject_code; RED-on-revert demonstrated.

**DoD:**
- [x] 3 `thread` sub-verbs (subcommand group + ops + 3 shims + **4** dispatch arms incl. aicontrol).
- [x] MP-C-13 GREEN: positive (create/resolve/archive events converge on node) + enforcement (member bob's resolve assert-the-reject, reject_code **4000** + absent).
- [x] RED-on-revert demonstrated (neuter `thread_archive` send → archive event missing from cooperative set → positive convergence assert RED; restored → GREEN).
- [x] Appendix F `thread create`/`resolve`/`archive` entries.
- [x] build 0 + clippy clean (default + `--all-features`) + suites green.
- [ ] Matrix MP-C-13 → ✅ (**Chat**). After close → R1 rerun.

**Empirical (MP-C-13 heavy GREEN):** create/resolve/archive converged on node;
member bob's resolve `reject_code=4000` (ChangeInfo PermissionDenied) + absent.
Pre-fold gate clear; the arc stayed thin (subcommand group, 4 arms up front; one
clippy nit — needless lifetime on a shim helper — caught + fixed pre-commit).
PermissionDenied → 4000 confirmed on the unmapped-variant ledger (MP-F2-followon).

(No "commit pushed" item. Clair's code commit precedes Chat's doc-bridge.)

---

Per D-065 + D-069 + D-071 + D-074 + D-092. MP-R1-D9 (assert-the-reject, MP-F5) +
MP-R1-D10 (loop-to-green) govern. TH-D# arc-local. MP-F6 noted (no re-route).
