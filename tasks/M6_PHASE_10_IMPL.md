# M6 Phase 10 — A7 Plugin management (2 reads)
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

A7 Plugin management (design §6.A7, Appendix K.2.7) — the 2 READ verbs (A7-D1
already deferred the WRITE verbs `load`/`configure`/`unload` until a second
plugin exists). This is the **last backed verb phase** of M6's write-path.

## Backing reality (verified at pickup, Rule 5)

The Node plugin subsystem is a **bare trait + one compiled-in no-op impl**:
`xgen-node/src/plugins/temperature.rs` defines `TemperaturePlugin` (only
`compute_room_temperature` / `compute_member_temperature` / `thresholds`) and
`NoOpTemperaturePlugin`; `load_default_plugin()` returns the no-op; the module
doc states "the loader / dispatcher mechanism is deferred." There is **no
registry struct and no per-plugin telemetry** (no name/version/status/kind
fields on the trait, no events-consumed / last-activity tracking).

**This differs from the A3/A1-writes/A4-`audit-events` gaps** (which had *no
backing at all*): the temperature plugin genuinely exists and is loaded. So the
2 reads are shippable **honestly but thin** (D-065):

- Added a minimal honest "registry": `crate::plugins::installed_plugins() ->
  Vec<PluginInfo>` — the **static set of plugins compiled into the binary** (one
  today). Not a dynamic loader/lifecycle store (that stays deferred, A7-D1) — a
  compile-time fact, which is what M6 actually has.
- `version` = the Node binary's version (`CARGO_PKG_VERSION`) — plugins are
  compiled in, no independent versioning in M6.
- `events_consumed` / `last_activity` = `None` — no telemetry is tracked in M6
  (honest, not fabricated).

## Verbs shipped (2 — A7-D1)

| Verb | Class | Audited | Backing |
|---|---|---|---|
| `plugin list` | READ | no | `installed_plugins()` (static compiled-in set; 1 entry: the temperature slot, no-op impl) |
| `plugin status <name>` | READ | no | lookup by name; `PLUGIN_9001` if unknown; telemetry fields honest `None` |

Both are pure reads of a compile-time fact — no live runtime handle needed (work
with the plain batch `AdminContext`). `GENERIC_4000` / `PLUGIN_9001`.

## Commit sequence (folded)

| # | Scope | Status |
|---|---|---|
| 1 | `plugins::{PluginInfo, installed_plugins}` + admin_ops `plugin_list`/`plugin_status` + clap `PluginCommand{List,Status}` + 2 verb tests | ✅ |
| 2 | pipe `dispatch_admin` `Plugin::List/Status` arms + dispatch-routing test | ✅ |
| 3 | Phase close: this file + JOURNAL J-158 + CLAUDE PLAY + ROADMAP | ✅ |

## Definition of Done

- [x] `plugin list` enumerates the compiled-in plugins (1 today); not audited.
- [x] `plugin status` returns detail or `PLUGIN_9001`; telemetry fields honest `None`; not audited.
- [x] clap `plugin list`/`plugin status` route via `dispatch_line`; M2 allowlist unchanged.
- [x] `cargo test --workspace` green; clippy `-D warnings` clean; build all-targets 0 errors.

## Verification (close)

- `cargo test --workspace`: **693 lib** (63 client + 35 common + 465 core + 130 node) + 25 integration; 0 failed. +3 node lib vs Phase 9-reads' 690 (2 verb tests + 1 dispatch-routing test). xgen-core unchanged (465).
- clippy `--workspace --lib --tests --all-features -- -D warnings`: clean. build `--workspace --all-targets`: 0 errors.

## Scope honesty (D-065)

- The "registry" is the **static compiled-in plugin set**, not a dynamic
  loader/lifecycle store (deferred, A7-D1). `version` is the binary version;
  `events_consumed`/`last_activity` are `None` (no telemetry in M6). Honest-thin,
  not fabricated. WRITE verbs land with a real plugin-management subsystem.

## M6 write-path status (after this phase)

**M6's backed admin write-path is now COMPLETE at 14 verbs:** A6 (5) + A5 (4) +
A1 subset (2) + A4 subset (1) + A7 (2). The only remaining M6-scoped verb is
**`force-eject`**, gated on the A4-D1 `membership.node_eject` wire sub-design
session (Chat-Claude + Joe). **~19 verbs** route to four post-M6 D-071 subsystem
arcs (federation-admin-control, bootstrap-client, auth-module-registry,
protocol-audit-log) + node-policy.

## Next

- `force-eject` — open the A4-D1 wire/validation sub-design session, then implement.
- Joe's held doc work: canonical §5.1/§6 amendments, the backing-audit A4-row
  correction, the four D-071 arc stubs.

---

*End of Phase 10 plan.*
