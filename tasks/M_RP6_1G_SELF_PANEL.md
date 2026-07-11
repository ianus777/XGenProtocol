# M-RP6.1g — R3 Self / connection: the first real system widget
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Canonical runbook for **M-RP6.1g**. Design walked + **Joe-locked 2026-07-11** (D1–D7). Chat's lane: this runbook. Clair's lane: implementation + design closes.

---

## 0. ⚠️ GROUNDING CORRECTIONS — READ BEFORE THE DESIGN

**Placed first deliberately (the J-497 precedent).** The design in `docs/xgen-client-frame-phase0.md` §6, and the pointer in `CLAUDE.md`'s PLAY block, were written **before** the code was grounded. **Three of their assumptions are false.** They are corrected here, and the canonical records move in the doc-bridge (commit 2) — **until then, this runbook supersedes them.**

| pre-lock claim | ground truth (read 2026-07-11) |
|---|---|
| *"a scoped `app.emit('self-state', …)` push + a webview listen"* | **The push ALREADY SHIPS.** `xgen-client/src/desktop.rs::emit_state()` fires `app.emit("xgen-client-state-changed", …)` on **every** lifecycle transition, and `ui/client/src/app_client.svelte` **already** `listen`s to it and already feeds the `status-bar`. **A second channel would be a second surface — the D-067 drift this project exists to kill. NO NEW EMIT IS BUILT.** |
| *"a `get_self_state` read verb"* (implying new projection logic) | **`ops::whoami` already exists** — sync, no network, `{identity_id, display_name, home_node, spaces_joined}` read from `xgen-client_state.json`. And **`session::ClientIdentity::load(keypair_path)` derives the identity XGID from the keypair alone** — no registration, no node. The new Tauri command is a **thin shell wrapper** (the `get_about_info` shape), **not** a new projection. |
| *"stop the node → the connection led flips via the push"* | **NOT RUNNABLE — do not attempt it and do not claim it.** `run_startup` does **one** 2-second `connect_async`, **drops the stream** (`Ok(Ok(_stream))`), emits `Ready`, and never touches the socket again. **There is no resident.** Killing the node changes nothing in a running client. The live flip needs a sustained WS + reconnect loop → filed as **M-RP6.6 client resident** (D7). |

**Data availability, MEASURED on the real machine (`%LOCALAPPDATA%\XGenProtocol`) — this is what R3 can honestly render today:**

| field | source | today |
|---|---|---|
| `identity_id` (XGID) | keypair → `ClientIdentity::load` | ✅ **available** (`xgen-client_keypair.enc` exists) |
| `display_name` / `home_node` / `spaces_joined` | `xgen-client_state.json` → `ops::whoami` | ❌ **absent — the file does not exist.** The Tauri shell has never `register`ed |
| Track-A `status` (emoji/text) | node `state.status/<xgid>` | ❌ **no client op exists at all** |
| connection lifecycle | `get_state` + the existing emit | ✅ **already shipped end-to-end** |

**Nothing is faked to cover a gap (D-065 / W-8).** The unregistered case is rendered **honestly**.

---

## 1. Scope

**In:**
1. One new Tauri **read command** `get_self_state` (thin, over `ops::whoami` + `ClientIdentity::load`).
2. One new **`$common` store** `self-state.svelte.ts` — `{ connection, identity }` + the relocated state→colour map. **One channel, two views.**
3. One new **widget** `self-panel.svelte` (`ui/common/lib/components/widgets/`) — the **first real system widget**, `kind: system` (W-13).
4. `layout-default.ts` — `widgetRegistry.self` → `SelfPanel` (**renderer A's first real leaf**).
5. `app_client.svelte` — writes the store; the `status-bar` reads the **same** store.

**Out (do not build):**
- ❌ Any new `app.emit` / any new event channel (D1).
- ❌ Any resident / reconnect / sustained WS (D7 → M-RP6.6).
- ❌ Any Track-A status read path (D6).
- ❌ Any capability-file change (expected; **ground it, do not assume** — see §5.4).
- ❌ Any `ui/core/**` change. R3 composes **shipped** `core` components unmodified.
- ❌ Any `ui/node/**`, any sampler change.

---

## 2. Locked decisions (Joe, 2026-07-11)

- **D1 — No new Rust push.** R3 reads the **existing** `xgen-client-state-changed`. Self-identity is **static per session** (no in-app registration exists), so it is fetched **once on mount** — the `get_about_info` shape. A push for it would be an **unfed branch**: the exact D-065/N-091 argument that kept `tabs` out of renderer A.
- **D2 — One thin command `get_self_state`, reusing `ops::whoami`.** A **shell read command**, NOT a D-092 four-armed verb (no node round-trip, no mutation, no CLI meaning — the `get_about_info` precedent). **No second projection surface.** Return shape:
  ```
  SelfStateInfo {
    registered: bool,                   // false when xgen-client_state.json is absent
    identity_id: Option<String>,        // from the KEYPAIR — present even when unregistered
    display_name: Option<String>,
    home_node: Option<String>,
    spaces_joined: usize,               // 0 when unregistered
  }
  ```
  `ops::whoami` returning `Err` is **not an error to the webview** — it is `registered: false`.
- **D3 — One `$common` store, two views.** `ui/common/lib/stores/self-state.svelte.ts`. The shell **writes** it (the existing `listen` + the two `invoke`s); the widget **reads** it (W-3 — a `common` widget must never import a shell dep). **The `status-bar` also reads it** — so the connection signal has exactly **one source**, literally two views.
- **D4 — R3 is the FIRST REAL SYSTEM WIDGET** (D-103 §1 — *every region is a widget*). Home `ui/common/lib/components/widgets/self-panel.svelte`, `data-tier="widget"` (W-9), composing **shipped** `entity-item` (dd-composite) + `status-indicator` (di-composite) inside a `section`. It replaces the `self` entry in `layout-default.ts`'s `widgetRegistry` — **one registry entry swapped, no rewrite** (exactly what the 6.1f registry was built for).
- **D5 — The selection bus gets its FIRST WRITER.** `entity-item`'s `onActivate` → `selection.set(regionId, descriptor)`. The W-8 honesty note in `selection.svelte.ts` ("there is NO UI writer") is now **retired** — update that comment.
- **D6 — Two honest phase-limits, surfaced not hidden (W-8).** ① Unregistered → the panel renders the **real XGID** + an explicit *not registered* secondary line. **No fake name.** ② The Track-A `status` slot ships **absent** (`status` prop simply not passed), **not faked**.
- **D7 — The F-1 claim is corrected.** 6.1g closes the **read *shape***, **not** the live half. The runnable proof is **relaunch-scoped** (node up → `READY`; node down → relaunch → `DISCONNECTED`). The live flip needs the resident → **M-RP6.6 — client resident** (sustained WS + reconnect + live lifecycle). **That** is the real F-1 close. **Do not smuggle any part of it into this milestone.**

---

## 3. Files

| # | file | new? |
|---|---|---|
| 1 | `xgen-client/src/desktop.rs` | edit — `SelfStateInfo` + `get_self_state` + `invoke_handler` |
| 2 | `ui/common/lib/stores/self-state.svelte.ts` | **new** |
| 3 | `ui/common/lib/components/widgets/self-panel.svelte` | **new** |
| 4 | `ui/common/lib/stores/selection.svelte.ts` | edit — retire the "no writer" W-8 note (D5) |
| 5 | `ui/client/src/layout-default.ts` | edit — `widgetRegistry.self` |
| 6 | `ui/client/src/app_client.svelte` | edit — store writes; `STATE_COLOURS`/`PULSING_STATES` **relocate out** |
| 7 | `ui/assets/skin.css` | edit — `.self-panel*` appearance (**N-090: ALL of it, including gaps/sizing**) |

**No other file is in scope.** Prove it with `git show --stat` (the J-497 discipline — grounded by scope, not asserted).

---

## 4. Rust half — `get_self_state`

Thin wrapper in `desktop.rs`, beside `get_about_info`. It composes **two existing** readers:

1. **XGID** — `crate::session::ClientIdentity::load(&data_dir.join("xgen-client_keypair.enc"))`. `Ok` → the identity XGID. `Err` (no keypair) → `None`. **This works today and is the field that proves the command is live.**
2. **Registration facts** — `crate::ops::whoami(&mut ctx)` with `OpContext { session, data_dir, node_override: None }`. `whoami` **never touches `session`** — it only reads `xgen-client_state.json` — so a `SessionState::new(…)` is constructed purely to satisfy the borrow. `Ok` → `registered: true` + fields. `Err` → `registered: false`, all `None`, `spaces_joined: 0`.

**Paths come from the managed `DataDir`** (M-RP6.1e-C2) — **never re-derived** from the config path.

### 4.1 ⚠️ Rule-6 grounding points — GROUND THESE, DO NOT GUESS

- **`IdentityXgid` → `String`.** `Display`? `.as_str()`? `.to_string()`? **Read the type.** (Chat has been wrong four times this arc by asserting from memory; this is not a place to be fifth.)
- **Module visibility from `desktop.rs`** — `crate::ops` / `crate::session` must be reachable (same crate; check `lib.rs`).
- **`SessionState::new` signature** — its `home_node: String` param. Since `whoami` ignores `session` entirely, pass an **empty string**. **Do NOT invent a config read to fill it** — that would be a second config-read surface for a value nothing consumes.
- **`identity_id` when BOTH sources exist.** Prefer the **keypair-derived** value (it is derived from the key = the actual identity); `whoami`'s is a cached copy. If they can disagree, that is a bug worth a DEV assert — **flag it, don't paper over it.**
- **`SelfStateInfo` home.** Define it in `desktop.rs` (shell-local, `Serialize`). **`NodeSelfState` is deliberately NOT declared** — the J-497 `NodeAboutInfo` precedent: a node wrapper today would guess fields with no call site to validate them against.

---

## 5. Frontend half

### 5.1 The store — `ui/common/lib/stores/self-state.svelte.ts` (new)

- Holds `{ connection: { state, label }, identity: SelfStateInfo | null }`, module-level `$state` (the `selection.svelte.ts` / `processor/store.svelte.ts` precedent — a `.svelte.ts` module).
- Writers: `setConnection(payload)`, `setIdentity(payload)`. Readers: `connection`, `identity` getters.
- **The payload is carried VERBATIM** — snake_case as Rust serialises it. **No rename/mapping layer**: a mapping layer is a drift surface, and the `about-dialog` already reads the raw payload.
- **`STATE_COLOURS` + `PULSING_STATES` RELOCATE HERE** from `app_client.svelte`. **This is forced, not stylistic:** the widget registry mounts a leaf with only `regionId` — **a widget cannot receive shell props** — so anything the widget needs must be store-mediated (W-3). The shell then reads the **same** map for the `status-bar`. **One map, two views.**
  - ⚠️ It is a **pure relocation**. Prove it (the N-090 discipline): re-measure the `status-bar` led's computed colour **before and after** the move — byte-identical, or it is not pure.
  - All **11** lifecycle states stay enumerated. `led`'s unknown sentinel is **BLACK** — a black led means a missed state. **No fallback branch.**
- DEV handle `__XGEN_SELF__` (N-024 idiom), so the verify pass can read both facets.

### 5.2 The widget — `ui/common/lib/components/widgets/self-panel.svelte` (new)

- Props: `{ regionId, id }`. **First ground what `region-node` actually passes to a leaf component** — `region-placeholder.svelte` takes `regionId`. Do not assume more.
- Root: `<div use:envelope={{ name: 'self-panel', id, debug }} data-tier="widget">` (W-9 — the `substitutions-editor` shape).
- Body: a `section` (`title="Self"`, id `<id>__section`) containing:
  - **`entity-item`** `variant="card"`, `id` = `<id>__item`. Descriptor built from the store:
    - `kind: 'identity'`, `id: identity.identity_id` (or a literal `'—'` when even the keypair is absent),
    - `name: identity.display_name ?? undefined` (absent name → `entity-item` falls back to the id; that is its shipped contract),
    - `secondary`: `identity.home_node` when registered, else **`'not registered'`** (D6).
    - **`status` NOT passed** (D6 ②).
    - `onActivate` → `selection.set(regionId, descriptor)` (**D5 — the first writer**).
  - **`status-indicator`** `id` = `<id>__status`, fed from the store: `states` = the relocated map, `state` / `caption` = `connection.state` / `connection.label`, `pulse` = `PULSING_STATES.includes(state)`.
- Aggregate getter G (W-4 — **observable task-state, never payload/secrets**):
  `{ registered, hasIdentity, connectionState, selected }`. **Do NOT publish the display name or XGID in the getter** — they are already CDP-readable off the composed children's own getters (`entity-item#…__item`), so publishing them again is duplication *and* a payload leak (the N-060 `hasValue` precedent).
- **Zero component `<style>`** (N-090/N-094). If you think you need one, ask: *could a skinner want to retune this?* If yes → `skin.css`. No exceptions.

### 5.3 The shell — `app_client.svelte`

- The existing `listen('xgen-client-state-changed')` callback **also** writes `selfState.setConnection(event.payload)`.
- After the existing `invoke('get_state')`, write it to the store too (the pre-listener race is already handled — reuse it, don't re-solve it).
- Add **one** `invoke('get_self_state')` → `selfState.setIdentity(…)`, inside the same `try` (the browser-dev/no-Tauri path must keep working — the `handleQuit` pattern).
- **The `status-bar` now reads the store**, not a shell-local `currentState`. Drop the local mirror — *one source*, which is the whole point of D3.
- `STATE_COLOURS` / `PULSING_STATES` **removed** from this file (relocated, §5.1).

### 5.4 Permissions — ground, do not generalise

`get_self_state` is an **app-defined** command → **expect NO capability grant** (J-497 ground truth: `get_state`/`get_substitutions`/`get_about_info` all run with none). But **the permission model NEVER generalises** — J-495 `core:window:*` **needed** one, J-497 app-defined needed **none**, J-498 a **plugin** command needed one again. **Prove it empirically** (`errCount: 0`), do not argue it.

### 5.5 Skin

All `.self-panel*` appearance in **`skin.css`** — including **gaps, spacing, sizing and layout** (N-090; `app.css` is the app-frame skeleton + the accent knob, **nothing else**).

---

## 6. Verify — REAL CLIENT 9222 (D-097)

The sampler is `tauri` + `tauri-build` only — it has no `get_self_state`, no protocol deps, no frame. It **structurally cannot** host this. **No sampler leg.**

**⚠️ N-092a — the orphan leg is NOT EXPRESSIBLE here.** The client's debug bridge (`ui/common/lib/components/base/debug.ts`) is **state-only** (`id → {type, get}`, **no DOM handle**). `domCount` / "0 orphans both directions" is a **sampler-only** capability. **Do not put it in this runbook's DoD.** The client-expressible proxy is: **drive churn, prove the registry returns EXACTLY to baseline** (N-095b).

| # | leg | what proves it |
|---|---|---|
| V1 | **Registry** | Baseline **30**. Enumerate the new ids; `count === unique`. `section#region-self` (the placeholder) is **gone**; the widget's own entries are present. **MEASURE the number — do not predict it** (Rule 5). |
| V2 | **The command is live** | `invoke('get_self_state')` — **quote the real JSON** (Rule 2). Expected today: `registered: false`, `display_name: null`, **and a REAL keypair-derived `identity_id`**. A `null` identity_id means the keypair read is wrong — the keypair **exists** (measured). |
| V3 | **The same verb, two grounded outcomes** | With the node up, run `xgen-client register` against **the same data dir** (`%LOCALAPPDATA%\XGenProtocol`) so `xgen-client_state.json` appears → re-`invoke` → `registered: true` + a real `display_name` / `home_node`. **This is the proof of the milestone.** If register cannot be run, **record that honestly** and leave V3 open — **do not fabricate it, and do not hand-write a state file to make it pass.** |
| V4 | **One channel, two views** | Node **up** → launch → `status-bar` led AND the widget's led both read `READY` from the **same** store. Node **down** → **relaunch** → both read `DISCONNECTED`. **The live flip is NOT claimed** (D7 — there is no resident). State this limit in the handback. |
| V5 | **Bus — the FIRST WRITER** | Click the self `entity-item` → `__XGEN_SEL__.current` = `{ regionId: 'self', entity: { kind: 'identity', id: '<the real XGID>' } }`. Then `clear()` → `null`. |
| V6 | **Geometry (N-091 — required for any layout-class change)** | The R3 leaf **fills** its cell; the leaf self-scrolls when overfilled; `documentElement.scrollHeight === clientHeight` (no document scroll). Split ratios unchanged. |
| V7 | **Relocation purity** | The `status-bar` led's computed colour is **identical** before/after the `STATE_COLOURS` move. A cross-file move *should* be a no-op — and *"should"* is not a verification (N-090). |
| V8 | **Skin** | `.self-panel*` rules in cascade (stylesheet-rule inspection, N-042). Accent-neutral, or **name the accent carrier** if one exists. **Zero component `<style>` shipped.** |
| V9 | **Churn → baseline** | Push a layout dropping the `self` leaf via `__XGEN_LAYOUT__` (**it is `{current, set}` — NOT `.default`**; Chat pushed `null` into it once and blanked the centre) → registry drops → restore → **returns EXACTLY to the V1 baseline.** |
| V10 | **Build + tests** | `cargo test` workspace (baseline **1507 / 0 / 62**) · `npm test` (baseline **41**) · `vite build` clean · `git show --stat` = the §3 file list, **no `ui/core/**`, no `ui/node/**`, no sampler**. |

**Harness notes (learned, do not rediscover):** single-expression `JSON.stringify({…})` evals only under PS 5.1; a read after a thrown eval is **inconclusive, not a failure**; Svelte 5 flips state synchronously but tears DOM down on the **effect flush** — read after settle; long-running processes (`cargo tauri dev`) hang the MCP server — **Joe launches dev sessions.**

---

## 7. Definition of Done

- [ ] `get_self_state` shipped in `desktop.rs`, registered in `invoke_handler`, reusing `ops::whoami` + `ClientIdentity::load`. **No second projection surface.**
- [ ] **No new `app.emit`**, no new event channel (D1). Grep-provable.
- [ ] `self-state.svelte.ts` store shipped; `STATE_COLOURS`/`PULSING_STATES` relocated; relocation proven **pure** (V7).
- [ ] `self-panel.svelte` widget shipped (`data-tier="widget"`, one aggregate getter, no payload in G, zero `<style>`).
- [ ] `widgetRegistry.self` → `SelfPanel`; the placeholder no longer mounts for `self`.
- [ ] Selection bus has its **first writer**; the W-8 "no writer" note in `selection.svelte.ts` retired.
- [ ] Unregistered state renders **honestly** (real XGID + *not registered*), Track-A status **absent not faked** (D6).
- [ ] V1–V10 green **with real quoted output** (Rule 2). Any leg not run is reported as **not run** (Rule 1) — V3 in particular.
- [ ] **No capability change** — proven by `errCount: 0`, not argued (§5.4).
- [ ] Scope-clean: `git show --stat` = §3 only.
- [ ] Deviations **flagged, not absorbed** (Rule 6) — including any correction to this runbook.

*(D-074: `Status: COMPLETED` is the shipped signal. "Commit pushed" is never a DoD item.)*

---

## 8. Records (doc-bridge — Chat's commit 2, NOT Clair's)

`JOURNAL.md` J-500 · `CLAUDE.md` PLAY · `docs/ROADMAP.md` · `docs/xgen-client-frame-phase0.md` §6 (**the §0 corrections + the D7 F-1 restatement + M-RP6.6 filed**) · `ui/docs/xgen-region-dock-model.md` §5 (bus writer live) · `ui/docs/xgen-ui-notes.md` (N-096+) · `ui/docs/xgen-ui-components.md` (widget registry) · `docs/xgen-widget-surfaces-phase0.md` §7 (the **6.1i–l relabel**, primes deleted).

---

## 9. Rules

**D-074 two-commit close:** Clair's feat = **commit 1, CODE ONLY**. Chat's doc-bridge = **commit 2**. **Joe pushes both. Neither Claude pushes.**
**Rule 5:** every number comes from real output. A registry count that was not measured does not enter the record.
**Rule 6:** flag deviations, don't absorb them — **including deviations from this runbook.**
**GROUNDING BEATS MEMORY:** read the source before asserting anything about it — **including your own DEV handles.**

---

*End of runbook.*
