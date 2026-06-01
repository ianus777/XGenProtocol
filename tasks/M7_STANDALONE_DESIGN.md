# M7-standalone — Design Phase (live config reload)
> **Status**: ACTIVE  
> Version: 1.3  
> Date: Jun 2026  
> **Last updated**: 2026-06-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 Status

Design phase on the Phase 0 audit (`tasks/M7_STANDALONE_AUDIT.md` v1.0). Arc-local decisions = **M7S-D#** (D-069).

**All design questions LOCKED (M7S-D1…D6).** DQ-1a (semantic) → Option A; DQ-2…DQ-5 framed + locked with three tightenings; DQ-6 locked **on the read-pattern trace** (§4), not the conservative default. **Next-active: implementation runbook.** No code. Clair stood down until runbook closes.

**Post-lock refinements (runbook v1.1, 2026-06-01):** **F-R2 adopted** — reload writes **no** config (restart-required values are already on disk from the operator's edit; no `toml` re-serialise, no comment loss). **CP-1 resolved** — the field-level no-lie report (M7S-D4) requires a diff baseline, so the resident **retains its startup `NodeConfig` snapshot**; reloadable fields update it, restart-required fields do not (snapshot = what's *running*). Exact handle home = confirm-at-pickup (D-078) at Commit 2.

---

## §1 M7S-D1 (LOCKED) — `local_mode` not safe to hot-reload in principle

`local_mode` → `accept_registration(…, local_node, …)` (`xgen-core/src/identity/registration.rs:193`), step 4–7 (`:235`): `if !local_node { trust_assertion.ok_or(TrustAssertionRequired)? }` — the **trust-admission gate**; admitted identities persist with `trust_assertion: None` (`:143/264`).

- **Fact 1 (decisive):** hot-reload is meaningless without a registry re-validation pass that does not exist — admitted assertion-less identities are never re-evaluated. A perfect seam can't fix this.
- **Fact 2:** federation-add re-reads fresh from disk (`admin_ops.rs:1744`) while registration uses the frozen startup bool — a live edit desyncs gate from registry **in the same instant** (live correctness break, not a preference).

**Lock:** Option A — **permanent Restart-required**, correct on the merits (§2.6.3 was wrong, not demoted). **No follow-on seam arc**; nothing deferred; no dangling "live-seam later" pointer. **Option B rejected** (a ~20-site hot-path refactor unifying the two read patterns is a D-071 subsystem arc, not smuggled into config-reload). §2.6.3 correction text in §3, executes at close (D-074).

---

## §2 Settled v1 shape + report contract

1. **Re-read** disk via `try_load_config` (`app.rs:3217`) in the `__RELOAD_CONFIG__` handler (holds `config_path` + `runtime`, `pipe.rs:719/720`).
2. **All-or-nothing gate** (M7S-D3) — reject the whole reload, apply **nothing**, if either: (a) TOML parse fails (`try_load_config → None`); or (b) the **`[node].listen` `SocketAddr` semantic check** fails. The `SocketAddr` parse is **part of the gate**, not apply-good-warn-bad.
3. **Diff** parsed vs running; **classify** each delta by disposition (§4 buckets).
4. **Apply Reloadable live** — only **`[logging].level`** via `LOG_RELOAD` (A6-D1, `app.rs:363–382`), self-validating.
5. **Restart-required** — already on disk (the operator edits `config.toml` *before* running `--reload-config`, so the new value is persisted by that edit; **F-R2**) → **no write-back** (avoids `toml::to_string_pretty` destroying operator comments); report **pending-restart** field-by-field; do not apply live.
6. **N/A / seed-only** — detected but neither applied nor persisted-for-restart; reported as N/A (never as reloaded or pending-restart).
7. **Structured report** replaces `NOT_IMPLEMENTED`.

**No-lie report contract (M7S-D1/D4 — load-bearing):**
- A field that parsed fine but is **restart-required** (e.g. edited `local_mode`) MUST surface as **pending-restart** — never silently ignored.
- **`REJECTED` MUST always carry a reason** (`parse` / `semantic` / `unknown`). A reasonless rejection is the same silent-lie M7S-D1 forbids.
- A **seed-only** field (e.g. `[bootstrap]`) MUST report **N/A (seed-only; store is truth)** or stay silent — never **pending-restart** (a restart won't apply it either).
- **Report shape (M7S-D4):** one human-readable control line (matches `__HEALTH__`; not JSON): `RELOADED: <fields>; PENDING_RESTART: <fields>; NA: <fields>; REJECTED: <field=reason>`. Only changed deltas listed.

---

## §3 Decision record (M7S-D1…D6, all LOCKED 2026-06-01)

- **M7S-D1** — `[node].local_mode` = **Restart-required**, permanent (not a demotion). Gates trust admission; admitted identities persist unreconciled; reload would desync gate from registry (§1). Option B rejected. No follow-on seam arc. **§2.6.3 correction (executes at close, D-074):**
  > `[node].local_mode` — Restart-required. Gates trust admission (`accept_registration` steps 4–7); admitted identities persist unreconciled, and a live reload would desync the gate from the registry. Not live-reloadable on the merits (M7S-D1).
- **M7S-D2** — Surface: **legacy `pipe.rs __RELOAD_CONFIG__` only** in v1; no `--aicontrol`/`admin_ops` exposure. **`--reload-config` is an operator action, not an AI-driver verb, in v1** (recorded). Single source of truth.
- **M7S-D3** — Validation: re-parse (all-or-nothing) + logging apply-time self-validation + **`[node].listen` `SocketAddr` check as part of the all-or-nothing gate** (fail → whole reload rejected, nothing applied). Other restart-required fields persist as-parsed.
- **M7S-D4** — Report: single control line `RELOADED / PENDING_RESTART / NA / REJECTED(reason)`; **`REJECTED` always carries a reason** (`parse`/`semantic`/`unknown`); pending-restart + N/A mandatory per §2 no-lie contract.
- **M7S-D5** — Scope: **Node-only v1.** Client `[ai.*]` / `[client].node` reload → own follow-on.
- **M7S-D6** — `[sync]` / `[federation]` / `[bootstrap]` classified **on the trace** (§4): `[sync]` = restart-required (b); `[federation].require_approval` = restart-required (b); `[bootstrap]` = **N/A/seed-only (c)**. Conservative default explicitly **not** taken — it would have lied on `[bootstrap]`.

---

## §4 DQ-6 read-pattern trace (the evidence behind M7S-D6)

Each section traced into one of three: **(a) fresh-per-use → already-live · (b) frozen-at-startup → restart-required · (c) seed-only / store-is-truth → N/A.**

- **`[sync]` → (b) restart-required.** `config.sync.batch_size` read once at startup (`app.rs:420`); `config.sync.federation_relationship_timeout_seconds` at `app.rs:841` — both threaded by value into the running sync/listener loops. Tuning values (not an admission gate), so restart applies them cleanly and honestly. No unreconciled-state hazard.
- **`[federation].require_approval` → (b) restart-required.** Read **once** at startup (`app.rs:655 let require_approval = config.federation.require_approval;`) and threaded by value into the handshake handlers (`:1070/1170/1231/1502/1565`). **The `local_mode` verdict was NOT assumed to transfer — traced independently:** it is **not** read-fresh-per-call, so the "already-live no-op" path **(a) is ruled out by evidence**. Restart-required is honest (a restart applies it to future handshakes). *Note (not a v1 concern):* like `local_mode`, a hypothetical live flip would leave already-established peers unreconciled, so it is not a future live-seam candidate either — but for v1 it classifies cleanly as (b).
- **`[bootstrap]` → (c) N/A / seed-only.** Startup loads the JSON store from disk (`app.rs:743–762`) and, when present, uses it verbatim — **no re-merge of `config.bootstrap.*` into an existing store**. The store (`xgen-node_bootstrap.json`) is truth; runtime `register`/`deregister`/`set-info`/`set-tiers` mutate the store, never the config (`app.rs:90–100`). So editing `[bootstrap]` in config has **no live effect and a restart won't apply it either** once the store exists. **Restart-required would be a lie in both M7S-D1 senses** — the report must classify it N/A (seed-only) or stay silent. (Config-seeding is a first-run/seed-path concern, outside live-reload.)

**Why the trace mattered:** the conservative default (restart-required for all three) would have been **right** for `sync`/`federation` but a **lie** for `bootstrap` — and only the read-pattern trace distinguishes them. Locked on evidence.

---

## §5 Next-active

**Implementation runbook** (`tasks/M7_STANDALONE_IMPL.md`): commit plan for the §2 mechanism (re-read → all-or-nothing gate incl. `SocketAddr` → diff/classify → apply logging live → persist restart-required → structured report) + Joe-lock checkpoints. §2.6.3 correction lands at close (D-074). No code until the runbook closes. Clair stood down.
