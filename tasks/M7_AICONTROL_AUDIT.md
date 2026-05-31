# M7 `--aicontrol` — Phase-0 Audit (drift-reconciliation + §12 triage)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this exists

M7 `--aicontrol` wraps an *existing, now-shipped* surface (`xgen-node-lib::admin_ops::*` + `xgen-client-lib::ops::*`) in an AI-shape JSONL protocol. The canonical doc `docs/xgen_aicontrol_implementation.md` (v1.1, last touched 2026-05-29) was written **before M6 shipped its admin write-path** — so its §7 verb sketches and §12.1 "M6 deliverables" gates predate the reality they describe.

This is **not** a does-the-subsystem-exist audit (the five D-071 audits were that). It is **drift-reconciliation + open-items triage**: (a) inventory what both binaries actually expose today, (b) diff the M7 doc against it, (c) triage §12 into already-closed vs genuinely-open. Evidence checked against the live tree 2026-05-31: `docs/xgen_aicontrol_implementation.md`, `xgen-node/src/admin_ops.rs` (6047 lines), `xgen-client/src/ops.rs` (1555 lines).

## Finding 1 — §12.1 ("M6 deliverables") is effectively CLOSED by M6 shipping

The doc gates these "before M6 goes ACTIVE." M6 is **done** (J-197), so they're resolved or relocated:

| §12.1 item | Status now |
|---|---|
| Verb-set enumeration + schemas | **DONE** — shipped in `admin_ops.rs` (the full clap surface, ~28 admin verbs + 8 M2 reads; see Finding 2) |
| Privilege model | **RESOLVED at M6** — OS-user-equals-administrator, session-scoped, no per-verb gating (D-082; admin-ops design §2.6.1/§2.6.2). The "is this enough for write-path?" question stays open **but is M7's** (§12.2 pipe-auth) |
| `admin_ops::*` shape | **DONE** — shipped, three-dispatcher pattern realised |
| Audit-trail integration per verb | **DONE** — `record_action` + A6 SQLite trail + §3.11.8 protocol log; **`ActorVia::AiControl` ("aicontrol") already exists** in the enum (Finding 4) |
| Live-reload semantics in full | **NOT M7-`--aicontrol`** — `config-reload` is still `NOT_IMPLEMENTED`; it belongs to the separate **M7-standalone (live config reload)** milestone (ROADMAP). The doc's §11 "live-reload is M6 scope" is itself drift |

→ **§12.1's "gate before M6 ACTIVE" framing is moot.** The audit's job on §12.1 is to mark it superseded and route live-reload to M7-standalone, not to re-deliver it.

## Finding 2 — Node verb surface drift (the load-bearing one)

**Shipped reality** (`admin_ops.rs`, clap enums): verbs are **two-token, category-grouped** under an `AdminCommand` enum:
- `audit` query/export/archive · `log` set-level/show-level · `identity` show/revoke/set-trust-expiry/manage-replica · `federation` list/defederate/accept/reject/initiate/set-policy/show-policy (7) · `space` list-hosted/audit-events/audit-rebuild/force-eject/unban/set-node-policy/show-node-policy (7) · `plugin` list/status (2) · `auth-module` list/register/revoke/set-tiers/test (5) · `bootstrap` show/register/deregister/set-info/set-tiers (5).

**The doc's §7 sketches a flat, hyphen-joined namespace with stale names and wrong counts:**

| §7 sketch | Shipped | Drift |
|---|---|---|
| `federate-accept/reject/add/remove/policy`, `defederation-signal` | `federation accept/reject/initiate/defederate/set-policy/show-policy/list` | name-style; `add`→`initiate`, `remove`→`defederate`, `policy`→`set-policy`+`show-policy` (split); `defederation-signal` **not shipped** |
| `auth-module-add/revoke/set-tiers` | `auth-module register/revoke/set-tiers/list/test` | `add`→`register`; doc missed `list`+`test` |
| `bootstrap-register/deregister/set-info` | `bootstrap register/deregister/set-info/show/set-tiers` | doc missed `show`+`set-tiers`; `set-info` args are endpoint/region/capability, **not** the doc's display_name/description/contact |
| `space-force-eject/set-policy/migrate-start` | `space force-eject/set-node-policy/show-node-policy/unban/list-hosted/audit-events/audit-rebuild` | `set-policy`→`set-node-policy` (+`show-node-policy`); `migrate-start` **not shipped** (A4-D2 deferred); doc missed unban/audit-events/audit-rebuild |
| `identity-revoke/set-expiry/replicate` | `identity show/revoke/set-trust-expiry/manage-replica` | `set-expiry`→`set-trust-expiry`, `replicate`→`manage-replica`; doc missed `show` |
| `audit-rotate/query`, `log-set-level`, `config-reload` | `audit query/export/archive` + `log set-level/show-level` | `audit-rotate`→`archive`; doc missed `export`/`show-level`; `config-reload` → M7-standalone |
| `plugin-load/configure/unload/status` | `plugin list/status` only | doc over-specified writes; A7-D1 defers them until a 2nd plugin exists |

→ **The load-bearing M7 design lock this surfaces: the `cmd` verb-exposure model.** The shipped surface is two-token (`federation accept`). M7's JSONL `cmd` field must decide its mapping — space-joined (`"federation accept"`), hyphen (`"federation-accept"`), or nested (`{cmd:"federation", sub:"accept"}`) — **driven by the shipped clap reality, not the doc's stale flat sketch.** Until that's locked, §7's names can't be trusted as the verb list. (The client side `register`/`send` etc. are single-token, so the model must cover both shapes coherently.)

## Finding 3 — Client verb surface drift (smaller, but a real coverage gap)

**Shipped `ops::*`** (M5/D-067): `register`, `whoami`, `status`, `spaces`, `rooms`, `create_space`, `create_room`, `invite`, `join`, `send`, `history`, `ai_delegate`, `ai_revoke`, `ai_status` — 14 functions. Most §6 verbs map cleanly.

**But §6 lists three verbs with no `ops::*` backing:** `create-dm-space` (§6.2), `members` (§6.2), `leave` (§6.3). They are not in the shared layer (they may be CLI-only in `app.rs`). Since §6 states "every Client verb routes through `ops::*`", **M7's client side can only wrap what `ops::*` exposes** — these three either need lifting into `ops::*` first or must be marked deferred. Confirm-at-design item.

## Finding 4 — Forward-readiness already in code (M7 reuses, doesn't rebuild)

- **`ActorVia::AiControl` ("aicontrol") already exists** in `admin_ops.rs` alongside `Batch`/`CliDirect` — the audit-actor tag for `--aicontrol` is pre-wired (verified: `ActorVia::AiControl.as_str() == "aicontrol"`).
- **The three-dispatcher pattern is real on both sides** (CLI · `--batch` · `--aicontrol`), exactly as §6/§7 assume — `--aicontrol` is a third caller, not a new implementation.
- **The error model is shipped**: `AdminError { code, stage, message }` with harmonised numeric bands (`GENERIC_4000`, `FED_3xxx`, `AUTHMOD_61xx`, `IDENT_6xxx`, `BOOT_7xxx`, `SPACE_8xxx`, `AUDIT_5xxx`/`LOG_51xx`) + a `Stage` enum (validate/register/persist/federate). This directly feeds the §4.3 lifecycle-aware error envelope.

→ M7 is genuinely adapter work. The substrate exists.

## Finding 5 — What is genuinely OPEN for M7 (the real design agenda)

§12.2's items stand, plus what this audit adds:

**From §12.2 (still open):** per-command default timeouts · event-pipe subscription filter grammar · `state` full schema (both binaries) · control-surface error-code catalogue · pipe-level authentication policy · replay-safety policy (idempotency keys, deferred).

**New, surfaced by this audit:**
- **AC: verb-name exposure model** (Finding 2) — *the first lock*; everything in §7 re-derives from it.
- **AC: reply/error envelope must nest the shipped `AdminError`** (Finding 4) — §4.2/§4.3/§8 must carry BOTH the control-surface code (`UNKNOWN_COMMAND`, `TIMEOUT`, uppercase snake) AND the underlying numeric verb code (`SPACE_8005`) + `stage`. The envelope is the protocol-shaping core, and its richer constraint comes from the **Node** admin surface, not the simpler client ops.
- **AC: client `ops::*` coverage gap** (Finding 3) — resolve create-dm-space/members/leave (lift vs defer).
- **AC: config-reload boundary** — explicitly out of M7-`--aicontrol` core; route to M7-standalone (correct the doc's §11).

## Ordering recommendation (the Q2 "let the audit decide" answer)

**Envelope-first, Node-constraint-dominated, client-first build.** The protocol lock is the §4.2/§4.3/§8 reply+error envelope, and its hard constraints come from the Node admin surface (code+category+stage+lifecycle, audited writes, hosted-here/propagation errors) — so **design the envelope against `admin_ops::*` richness** so it never needs rework. Then **implement client-first**: the stable, simpler `ops::*` is the cheapest place to validate the locked envelope + bindings + `state` end-to-end before applying it to the heavier admin verbs. So neither pure client-first nor pure node-first — the envelope is Node-dominated in *design*, client-first in *build*.

## Routes to design (`tasks/M7_AICONTROL_DESIGN.md`, Joe-reserved)

Lock order suggested: (1) **AC verb-name exposure model** (unblocks §7); (2) **AC reply/error envelope** nesting `AdminError`+stage + control-surface codes; (3) §12.2 mechanicals (timeouts, subscription grammar, `state` schema, error catalogue); (4) pipe-auth policy (the M6-deferred §2.6.1 question, now M7's); (5) client `ops::*` gap (lift/defer); (6) replay-safety (likely defer, as §12.2 leans). Mark §12.1 superseded; route live-reload + config-reload to M7-standalone.

## Cross-refs

- `docs/xgen_aicontrol_implementation.md` (v1.1) — the canonical M7 spec this audit drift-checks; §7 sketches + §12 open-items + §11 live-reload.
- `xgen-node/src/admin_ops.rs` — the shipped Node surface (AdminCommand + 8 subcommand enums; `AdminError`/`Stage`/`ActorVia`).
- `xgen-client/src/ops.rs` — the shipped client surface (14 `ops::*` fns).
- D-066 (the `--batch`/`--aicontrol` split), D-063 (library-first multi-dispatch), D-067 (`ops::*`), D-082 (administrator vs operator), D-069 (open-item flagging), D-071 (audit-precedes discipline, applied here as drift-reconciliation).
- M7-standalone (live config reload) — the correct home for `config-reload`/§11, NOT M7-`--aicontrol`.

---

*End of audit. Next phase (Joe-reserved): `tasks/M7_AICONTROL_DESIGN.md` — lock the verb-name exposure model + envelope first.*
