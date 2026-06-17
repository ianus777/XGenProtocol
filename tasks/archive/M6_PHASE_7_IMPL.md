# M6 Phase 7 — A1 Federation management (honest-subset)
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

A1 Federation management (design §6.A1, Appendix K.2.4). Block 4 specified **7
verbs**; this phase ships the **honest subset of 2** that have real backing in
the post-federation-milestone `FederationRegistry`. Authoritative spec:
`docs/xgen_node_admin_ops_design.md` §6.A1 + Appendix K.2.4.

## Reorder + scope decisions (Joe-locked, J-156)

**Two Rule-6 checkpoints fired before any A1/A3 code, both resolved by Joe:**

1. **Phase 6 (A3 Bootstrap config) DEFERRED — reorder to Phase 7 (A1) first.** Recon
   found A3's whole network half is architecturally absent: `bootstrap/client.rs`
   is a 16-line placeholder, **nothing sends `bootstrap.register` in production**,
   and there is no `[bootstrap]` config / registrations store / self-info store.
   The gap is specifically the bootstrap-*client send path* (the server-side
   `directory.rs`/`reputation.rs`/`capability.rs` are real). Implementing A3's
   `federate` re-advertise stage means building an absent client subsystem inside
   a verb phase — the J-081 / D-071 anti-pattern. **A3 re-enters via its own
   D-071 Bootstrap-client audit→design→impl arc; the exact re-slot is Joe's
   pending call.**

2. **A1 ships the honest subset `list` + `defederate` only.** Of the 7 designed
   A1 verbs, only these two have real backing:
   - ✅ `federation list` (READ) — reads `FederationRegistry.all()` + `peer_records`.
   - ✅ `federation defederate` (DESTRUCTIVE) — `remove()` + persist + report shared spaces.
   - ❌ `federation accept` / `reject` — **no admin-approval pending-request queue**
     (federation auto-establishes on handshake; nothing to accept/reject).
   - ❌ `federation set-policy` / `show-policy` — **no policy store, no policy type,
     no enforcement consumer** (a policy nothing reads is a stub).
   - ⚠️ `federation initiate` — would have to admin-gate the handshake (heavy).

   The 5 deferred verbs go to a **post-M6 federation-admin-control subsystem arc**
   (approval queue + policy store + enforcement) under D-071 — the same
   "no half-feature on an immature surface" call as A3 / A7-D1 / A4-D2. Not folded
   into a verb phase (Option 3 rejected as the D-071 anti-pattern).

## Verbs shipped (2)

| Verb | Class | Audited | Backing |
|---|---|---|---|
| `federation list` | READ | no | `FederationRegistry.all()`; paginated (`limit` default 50 / cap 500, `cursor`); `--state` honest (active/all only — no pending/revoked state exists, A1-D2) |
| `federation defederate` | DESTRUCTIVE | **yes** | `FederationRegistry.remove()` on the *live* registry + persist to `xgen-node_federation.json`; reports `cleaned_spaces` (the relationship's `shared_spaces`) |

`FED_3001` invalid state filter · `FED_3004` not federated · `GENERIC_4000`.

## AdminContext extension (P5 precedent)

A1's `defederate` mutates the **live** `FederationRegistry` (so federation paths
stop treating the peer as federated at once), persisted to disk — the same P5
precedent A5 set for `NodeRuntime`. `AdminContext` gains
`federation_registry: Option<Arc<Mutex<FederationRegistry>>>` + builders
`with_runtime` / `with_federation_registry` (replacing the single-purpose
`batch_with_runtime`, which now delegates) + `federation_registry_path()` +
`require_federation_registry`. The pipe server already holds the registry `Arc`
(creates it at startup); it now threads it through `start_pipe_server` →
`dispatch_line` → `dispatch_admin` (a 2nd optional live-state handle alongside
the A5 runtime handle).

## Commit sequence (folded per the M6 cadence)

| # | Scope | Status |
|---|---|---|
| 1 | AdminContext federation handle + builders + `federation_list`/`federation_defederate` verbs + clap `FederationCommand{List,Defederate}` + 2 verb tests | ✅ |
| 2 | thread `Arc<Mutex<FederationRegistry>>` through `start_pipe_server` (app.rs spawn) + `dispatch_line`/`dispatch_admin` + 2 Federation arms + dispatch-routing test; `#[allow(too_many_arguments)]` on the now-8-arg `start_pipe_server` | ✅ |
| 3 | Phase close: this file → COMPLETED + JOURNAL J-156 + CLAUDE PLAY + ROADMAP | ✅ |

## Definition of Done

- [x] `federation list` paginates (limit/cursor), `--state` honest (active/all; pending/revoked match zero); `FED_3001` on bad state; not audited.
- [x] `federation defederate` removes from the **live** registry, persists, reports `cleaned_spaces`; `FED_3004` when not federated; DESTRUCTIVE → audited.
- [x] clap `federation` grouping routes `list`/`defederate` via `dispatch_line`; the M2 read-only allowlist is unchanged.
- [x] AdminContext carries the live `FederationRegistry` handle; pipe threads it from the resident.
- [x] `cargo test --workspace` green (688 lib + 25 integration, 0 failed); clippy `-D warnings` clean; build all-targets 0 errors.

## Verification (close)

- `cargo test --workspace`: **688 lib** (63 client + 35 common + 465 core + 125 node) + 25 integration; 0 failed. +3 node lib vs Phase 5's 685 (2 verb tests + 1 dispatch-routing test). xgen-core unchanged (465) — A1 added no core code.
- clippy `--workspace --lib --tests --all-features -- -D warnings`: clean. build `--workspace --all-targets`: 0 errors.

## Scope honesty (D-065)

- `defederate` removes the federation **relationship record** and reports its
  `shared_spaces`; it does **not** deep-GC replicated Space data (D-022/§3.15)
  nor send a network `federation.goodbye` — both belong to the deferred
  federation-admin-control arc. The peer observes the relationship gone on next
  interaction.
- `list --state pending|revoked` returns zero, honestly: the registry has no
  state field; a recorded relationship is active.
- `--batch` reply stays OK/ERROR (M2-frozen); rich output is M7 `--aicontrol`.

## Pending Joe actions (not done in this phase, by Joe's reservation)

- **Canonical design-doc amendments** to `docs/xgen_node_admin_ops_design.md`
  §5.1 (phase order) + §6.A1 (A1 honest-subset + 5-verb deferral) + §6.A3 (A3
  bootstrap-client deferral) — Joe reserved these for his confirm.
- **The D-071 arc docs**: a Bootstrap-client arc (A3) and a federation-admin-
  control arc (the 5 deferred A1 verbs).
- **Audit-now fork**: whether to run a cheap read-only backing-map audit across
  A1–A7 now (so remaining deferrals are deliberate, not rediscovered
  category-by-category) — Joe's pending call.

## Next

Per Joe's pending decisions above. Candidate next category by backing: **A2 Auth
Module** (Phase 8) or **A4 Space/Room** (Phase 9, design-gated on the
`membership.node_eject` sub-design) — both should get the same backing check
first.

---

*End of Phase 7 plan.*
