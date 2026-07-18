# XGen Protocol — Development Journal
> **Status:** ACTIVE  
> **Last updated:** 2026-07-18  

This document is a chronological record of development activity on the XGen Protocol project.
It is intended to establish authorship, timeline, and scope of original work for intellectual
property purposes. Entries are written contemporaneously with the work described.

---

## Entry J-542 — M-RP6.6 (client resident: live connection + traffic accounting) OPENED: Phase-0 grounded against live code, all five decisions Joe-LOCKED by-recomm; next-active = Clair build runbook (Legs A/B + Leg-C Rust half)

**What happened.** M-RP6.6 opened (M-RP6.2 closed J-541). Chat ground the six session-open questions against `main`, presented Phase-0, Joe locked D1–D5 by-recomm. Doc-only; no code. Phase-0 `tasks/M_RP6_6_RESIDENT.md` (v1.1 ACTIVE). No DECISIONS change (D1–D5 arc-local).

**Grounding verdict (verified `main`).** (1) `service::run` already holds a THIN resident (`connect_url`→`client_authenticate`→`loop{recv()}` discard→`goodbye`) with **no** lifecycle emit (headless), **no** accounting, **no** reconnect, **no** ingest — desktop resident is a **superset**, not a copy. (2) All **11** `ClientLifecycleState` variants are real Rust AND all 11 enumerated in `STATE_COLOURS` — matched; work is WIRING real events to the existing `emit_state`, no variant to add. (3) ConnStats Speed/Bandwidth are **hardcoded literal `'N/A'`**, not a store key waiting — the N/A→live seam is three joined pieces (counters · store slot · row swap), none existing; transport hooks present (`send_bytes` choke · `recv()` · `ping()`+`Inbound::Pong`). (4) `app::resolve_node` exists (D-068 flag>config), `service.rs` uses it; desktop's hardcoded `ws://127.0.0.1:8080` is the lone holdout. (5) No reconnect/backoff scheduler anywhere — must be built. (6) Live-ingest seam clean, DEFERRED (gated on R5 + M-RP6.3).

**Decisions Joe-LOCKED (all by-recomm).** D1 spine — extract a shared `connect→auth→drain` helper (D-056); not fork/rebuild. D2 leg split — A resident+lifecycle · B reconnect/backoff · C accounting; live-ingest→own milestone. D3 accounting capture — resident-level wrapper counters, GPL core `Connection` **UNTOUCHED** (honest scope: bytes observed by the resident loop; auth handshake + `send_event_confirmed` internal drains excluded). D4 ConnStats data contract — real-when-fed, absent-when-no-counter (N-060), never a fabricated `0` (the row LOOK is Ms Design's; only the data rule is fixed). D5 sequencing — 6.6 BEFORE 6.3.

**Lane split.** Chat = this Phase-0 + the Leg-C data contract (`selfState.traffic` slot: `ConnTraffic{bytes_in, bytes_out, rtt_ms|null}`, snake-case verbatim) + CDP-verify (lifecycle flips on real connect/disconnect, rows N/A→live). Clair = Legs A/B + Leg-C Rust half (shared spine · backoff · counters · `get_conn_stats` mirroring `get_pacing_state`) + build runbook. Ms Design = the ConnStats visual row-swap (appearance only). Joe locks + pushes.

**Correction (D-065).** Session-open quoted the cargo floor as 1517/0/62; live floor is **1519/0/62** (J-541's +2). Rust-primary → the count MUST move (Leg B backoff + Leg C counters are testable pure logic). Task-doc §6 floor corrected (v1.1).

**Canonical (D-074).** `tasks/M_RP6_6_RESIDENT.md` v1.1 ACTIVE (D-1..D-5→D1..D5 house-normalized · floor 1519); `CLAUDE.md` PLAY head (M-RP6.6 🟡→🟢 PHASE-0 LOCKED, new tail); `docs/ROADMAP.md` v5.08→5.09 (L778 🟡→🟢 · 6.6-precedes-6.3); this JOURNAL J-542. No DECISIONS change.

**Next-active.** Clair authors `tasks/M_RP6_6_IMPL.md` (build runbook, spine-first, Leg A→B→C); Joe-lock before any code → Chat CDP-verify live (full reload, N-132) + doc-bridge → close. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-542 → `tasks/M_RP6_6_RESIDENT.md`.** Not pushed — Joe pushes.

---

## Entry J-541 — M-RP6.2 — R1 Spaces + R2 Rooms on real `KnownSpace` — CLOSED: the two navigation anchors become functional widgets on the live Space tree, the bus gains its 2nd + 3rd writers, and the first cross-region data flow works end-to-end

**Single-seat milestone (Chat: grounded, authored the runbook, Joe locked D1–D8, Chat built + CDP-verified). Code + records ready to commit; Joe pushes.** Back to the grid after the SETTINGS arc. The `spaces` (R1) and `rooms` (R2) `RegionPlaceholder`s are replaced by real `kind: system` widgets on the live `KnownSpace` tree: **select a space in R1 → R2 shows its rooms → R8 inspects either** — the same `{regionId, entity}` selection bus R3→R8 proved (J-500/J-501), now with real state and a real cross-region flow.

### The grounding was the milestone's first output (N-116)

The runbook (v1.0, PENDING) had grounded the read verb / command shape / leaf-mount / entity-panel correctly, but **Anchor 5 (the placeholder swap) was a generation stale** — it described "register a system widget" (the J-500 self/inspector era). Re-confirmed against the tree: since **M-RP6.1l** (the plugin registry, J-513) and **M-RP-CONNSTATS** (the reactive derivation, J-533), a region widget is swapped in by **adding a `PluginDescriptor` to `CLIENT_PLUGINS`**, which `buildWidgetRegistry(installed.mounted)` derives — no `app_client` register line. Code-verified at `installed.svelte.ts:57/63` (both `active`/`mounted` start `...CLIENT_PLUGINS`, so a `kind:'system'` entry needs no install step, no shell wiring). **The record was self-consistent; the code had moved.** Runbook corrected to v1.1 (Anchor 5 + the two consequences: +2 plugin-list rows, and each descriptor's `name` becomes the tile title via `buildTitles`), then Joe locked D1–D8 (v1.2).

### Shipped

- **Leg 0 — Rust `get_spaces`** (`desktop.rs`): a thin shell read (the `get_self_state` shape) composing the existing unit-tested `ops::spaces`; returns `Vec<KnownSpace>` verbatim (rooms embedded, D1 — no `get_rooms` UI command); `Err` → `unwrap_or_default()` → `[]` (honest unregistered empty, W-8). Registered in `invoke_handler`. **No `ops.rs` touch.** + 2 serialize tests (embedded-rooms/snake-case shape; empty→`[]`).
- **Leg A — R1** (`ui/common`): new `stores/spaces-state.svelte.ts` (`$common` store, TS `KnownSpace`/`KnownRoom` interfaces mirrored from the Rust serialisation so `core` imports no protocol type; DEV `__XGEN_SPACES__`) + `widgets/spaces-panel.svelte` (`entity-panel` of spaces; writes `selection.set('spaces', …)`; `selected` bus-derived, D5; the `KnownSpace → EntityDescriptor` projection lives **in the widget**, the self-panel precedent, D7) + `CLIENT_PLUGINS` descriptor (`surface:'region'`, `regionId:'spaces'`) + one `app_client` hydrate line (`spacesState.setSpaces(await invoke('get_spaces'))`).
- **Leg B — R2** (`ui/common`): `widgets/rooms-panel.svelte` (**the one new mechanic, D3**: latches the last SPACE selection from the bus and KEEPS it while the bus holds a room — without it, clicking a room blanks R2's own list; reads the scoped space's embedded `.rooms`, D1; writes `selection.set('rooms', …)`) + `CLIENT_PLUGINS` descriptor. **No new store, no new invoke, no `app_client` change.** The latch `$effect` reads `selection.current` and writes `latchedSpaceId` — never reads it, so no self-invalidating read-modify-write (the N-136 trap avoided by construction).

### Verify — live client 9222, Chat re-drove every leg (Rule 5, N-132 full reload first)

The dev client is **unregistered** (J-500 situation), so `get_spaces` honestly returns `[]` (**V1** ✅, and the "No spaces yet" empty row rendered — **V2** empty ✅). Interactive legs driven on an **injected 2-space tree** via the store's DEV setter (the same boundary the shell hydrate feeds — the widget→bus→cross-region flow is fully real):

- **V0** cargo **1519/0/62** (+2). **V2** R1 `count:2`, two rows. **V3** trusted click → bus `{kind:'space'}`, row **paints** `box-shadow rgb(154,106,48) 2px 0 0 inset` (the pixel, N-097). **V4** (the real proof) space select → **R2 repopulates** + **R8 shows the space**. **V5** (D3 latch) room click → bus `{kind:'room'}`, **R8 follows**, **R2 keeps its list**, **R1 un-highlights** (D4 opt-1) — all four in one read. **V6** two distinct empties ("Select a space" / "No rooms"). **V7** bus purity — a phantom space id no R1 row exists for renders in R8, R2's stale-latch guard falls to "Select a space" (no throw). **V8** Settings → Plugins shows the `Spaces` + `Rooms` rows, `:modal true`.
- **Registry:** quiescent **count===unique===119**, `droppedCount:0`. **M-RP6.2 adds exactly +8** (two `entity-panel` subtrees; a still-placeholder region registers only its tile — `streamEntries:[]`). No leaks.
- **Gates:** vite **183** · npm **77** · cargo **1519/0/62**. **N-097 skin check ✅** (no `.spaces`/`.rooms` in `skin.css`).

### Findings / Rule-6 deviations (Chat's own predictions, corrected against the live tree — N-105)

1. **V8 plugin-list count moved 4→6, NOT the runbook's "3→5".** The pre-M-RP6.2 base was **4** system plugins (self, inspector, plugin-list, **grid-plate**) — grid-plate was added at M-RP-PLATE (J-532) since the runbook re-quoted the stale J-513 "3". The +2 (Spaces/Rooms) is right; the base was stale. *A count re-quoted from an old close is a hypothesis, not a measurement (N-105).*
2. **The `[system]` distinction is the version line + host-tinted icon, not a literal `[system]` text badge** — M-RP-SETTINGS Leg B (J-537) replaced the badge with the version. Rows render `"Spaces v1.0.0"` / `"Rooms v1.0.0"`, `systemCount:6`.
3. **Baseline is 119 on this machine, not the J-540 "99"** (N-108 — count depends on the machine's store/build context). The verified quantity is the **+8 delta**, computed on this same load (placeholder region = 1 tile entry; widget region = 5).
4. **The first V5 room-click missed on a stale coordinate** (285 measured the instant R2 repopulated, before layout settled; the row was at 304). Re-measured and re-clicked — the "re-measure before every gesture; a rect is not a constant" harness rule, live.
5. **D8 `icon` left UNSET** on both descriptors → the documented `plugin-list` fallback `p.icon ?? 'square'`. No verified Material source SVG for a spaces/rooms glyph in-repo, and a `d` path is not fabricated from memory (Rule 5 / D-108) — real glyphs filed to M-RP-ICON-ADOPT / M-RP-SKIN.

**No new D, no new `core`, no new N** (D-103/D-112/W-3 extension; the latch is a D3 realisation). Records: this JOURNAL J-541 · `tasks/M_RP6_2_SPACES_ROOMS.md` v1.3 ACTIVE→COMPLETED (§9 close) · `CLAUDE.md` PLAY · `docs/ROADMAP.md` (M-RP6.2 ✅, R1/R2 no longer placeholders). **Next-active: M-RP6.3 — live messaging (the send verb) / the R5 stream wrap**, then `temperature-indicator` unblocks once a non-no-op node plugin + live activity exist (still ⏸️). `get_spaces` closes only the *read* shape; the live Space push is the resident (M-RP6.6).

## Entry J-540 — M-RP-SETTINGS Leg C CLOSED → the SETTINGS arc is CLOSED: the settings mechanism (D-120) + the grid-plate backdrop setting (B2) shipped, re-driven live, D-120 minted

**Commits `5f4a6fe` (feat, 8 files) + `8b7ca1a` (fix: untrack the persist, 1 file) [Clair, code-only, all `ui/`, on `main`/pushed].** Zero `.rs`, zero `ui/sampler/` (`git show --stat` = 8 `ui/` files + `app_client.svelte`) → `cargo test` **1517/0/62 IDENTICAL by construction**, sampler catalogue **328**. The LAST leg of the SETTINGS arc; the arc (A shell · B action row + window · C mechanism + backdrop) is CLOSED. **D-120 minted** this entry (built-and-looked-at, the D-119 precedent — the reserved number from J-539 lands with the code).

### What shipped (D-B → D-120, B2)

New `$common/lib/stores/backdrop.svelte.ts` (one boolean `pattern`, default true) + new `$common/…/widgets/grid-plate-settings.svelte` (a `core` `Toggle` writing the store, id `grid-plate-settings__toggle`). `grid-plate` reads the store and **paints `data-pattern`** (B2 — no longer fully inert), G `{backgroundLive, pattern}`; `registry.ts` grid-plate row gains `settingsComponent`; `skin.css` `.grid-plate[data-pattern]`=raster / plain otherwise. `settings-dialog.svelte` **drill-in generalised** `detailId → drill={id, mode:'info'|'settings'}` (the J-539 reuse), `settings` **intercepted locally**, generic `{@const C}<C/>` mount — `app_client.handlePluginAction` and `plugin-list` **untouched**. `uistate.svelte.ts` gains a `backdrop` session key + `setSessionBackdrop` (N-107 per-key merge, zero Rust); `app_client` binds `backgroundLive={backdrop.pattern}`, seeds before `loadLayout`, persists via an **`untrack`ed** effect.

### The defect Clair shipped in `5f4a6fe` and caught in live verify → N-136

The backdrop persist `$effect` called `setSessionBackdrop()`, whose read-modify-write **read `_store.session` inside the effect** → it became a dependency → the write self-invalidated it → `effect_update_depth_exceeded` → Svelte halts the render scheduler: `$state` keeps mutating (getters advance, toggle animates natively) but the **DOM freezes**. Symptom = a *dead-button UI*, **not a crash** → it passed every static gate; **only a live drive surfaced it** (Joe caught it on screen). Fix (`8b7ca1a`) = `untrack` the write. **The other persist axes (`installed`/`disabled`/`locked`/`layout`) were safe because they persist from EVENT HANDLERS, not effects** — backdrop was the first persist wired through an effect. → **N-136** (Rule-5 / N-099 family: a failure only a live drive can see). *This is why the leg was not closed on the static gates alone.*

### Measured — Chat re-drove EVERY leg (live 9222, full reload first, N-132; zero errors)

| leg | result |
|---|---|
| baseline | **99 === unique 99**, quiescent — store `backdrop:true` (Clair's toggle test, **cited N-108**), else empty (`installed:[]`, `disabled:[]`, `locked:null`, `active:null`, `named:0`, `sel:null`). 99-family unchanged — nothing new is always-mounted |
| persisted value at first frame | `region-shell` G `backgroundLive:true` + `grid-plate` G `pattern:true` + **painted `data-pattern="true"`** — the persisted choice paints on reload |
| mechanism (D-B) | gear → Settings `:modal true` @ Plugins; `[settings]` **ENABLED for grid-plate ONLY** (self-panel/inspector-panel/plugin-list `aria-disabled`) — descriptor-derived; click → drill `detail:{id:'grid-plate',mode:'settings'}`, header **“Grid Backdrop”**, Back present, `toggle#grid-plate-settings__toggle` registers, **99→76** (list unmounts, settings component swaps in) |
| B2 flip (the REAL toggle, N-097) | true→false: store + `grid-plate` G `pattern` + `region-shell` G `backgroundLive` + **painted `data-pattern` REMOVED** (raster→plain) + input unchecked + `session:false` persisted; false→true restores **`data-pattern="true"`**. **Two-way on the painted DOM** |
| untrack fix | reads return cleanly through the flip — no `effect_update_depth_exceeded`, DOM responsive (the N-136 fix proven) |
| hygiene | Back → **99===99** (toggle unregisters); Close → `open:false`, **99===99** no leak; reload → **99===99, `backdrop:true` survives, painted first-frame** |
| gates | `vite build` **178** · `npm test` **77** · scope 8 files, 0 `.rs`/0 sampler → `cargo test` **1517/0/62 IDENTICAL by construction** |

**⚠️ Two of Chat's read-selectors reported clean-looking nothings before I fixed them (N-110 family, logged not hidden):** `get('settings-body')` missed the `type#id` key (it is `settings#settings-body`), and `ids().indexOf('grid-plate-settings__toggle')` checked exact array membership when the id is `toggle#grid-plate-settings__toggle` — both returned a plausible `null`/`false` until re-queried against the real ids. *A selector that cannot see its subject reports the flattering answer; re-ground it before believing it.* No phantom entered the record.

### The backdrop's final form (Joe, 2026-07-17) — an M-RP-BACKDROP refinement, NOT a Leg-C change

Joe refined the FILED backdrop-type menu (`M-RP-BACKDROP`): of its four visual types, **the first is now `solid/gradient`** (was `solid`). This is a filing refinement to the final menu — it does **not** touch Leg C's shipped one-boolean B2 toggle (a mechanism proof), and the full 4-type enumeration is pinned when M-RP-BACKDROP walks. Recorded against M-RP-BACKDROP (ROADMAP / PLAY / D-120 relationship line).

### Canonical (D-074)

`DECISIONS.md` **+D-120** (settings mechanism = component-per-plugin hosted in the content pane; `settings_schema` superseded); `ui/docs/xgen-ui-notes.md` **+N-136** (the untrack finding); `docs/xgen-settings-phase0.md` **v1.1→v1.2, Status ACTIVE→COMPLETED** (§9 built + verified); runbook `tasks/M_RP_SETTINGS_C_MECHANISM.md` **ACTIVE→COMPLETED**; `CLAUDE.md` PLAY (Leg C + arc CLOSED, baseline 99); `docs/ROADMAP.md` (arc CLOSED + the M-RP-BACKDROP refinement); this JOURNAL J-540. No new `core`.

**Next-active.** The SETTINGS arc is closed. Open follow-ons unchanged: `M-RP-BACKDROP` (the static/generative/data-driven, base-vs-stack type menu — unblocked by this leg, not started; type 1 = solid/gradient) · `M-RP-PLUGIN-INSTALL` (the `+` install affordance) · `M-RP-PLUGINS-NODE` · `M-RP-DIALOG-CHROME` · `M-RP-ROVING` · `M-RP6.6`. Per ROADMAP, the client region arc resumes at **M-RP6.2** (R1 Spaces + R2 Rooms on real `KnownSpace`). Not pushed by Chat — Joe pushes.

---

## Entry J-539 — M-RP-SETTINGS Leg C DESIGN-LOCKED: the settings mechanism (D-B → reserved D-120) + the grid-plate backdrop setting; swap = REUSE the Leg-B drill-in; backdrop = B2 (one painted value)

**Records-only (design). No code, no Rust, no registry change.** The last leg of the SETTINGS arc. Session-open presented the Leg-C Phase-0 — grounded against live code first (N-116), then the mechanic, the forks needing Joe, and the legs — and Joe **locked** all three open decisions.

### Grounding (N-116 — grepped, not assumed)

- **The content pane swaps two ways** (`settings-dialog.svelte`): a sidebar **section swap** (all mounted, inactive `[data-active]`→`display:none`) and, inside the Plugins panel, a **list↔detail `{#if}` swap** driven by `info`→`detailId`, Back→`null`, the shared header owning Back+×. **The `settings` verb is FORWARDED to the shell today** (`onPluginAction`→`app_client.handlePluginAction`, which handles only `uninstall`/`disable`) → a **no-op**.
- **The backdrop is threaded but unpersisted:** `region-shell` takes `background`/`backgroundLive`/`bgWidgets`; `app_client` passes **`backgroundLive={true}` as a hardcoded literal** (the seam Leg C binds); `grid-plate` **accepts `backgroundLive` and IGNORES it** (inert by contract, getter reads it back); **no backdrop choice is persisted anywhere** (reserved for this milestone).
- **`settingsComponent` is unfed:** `PluginDescriptor.settingsComponent?` is `undefined` on every row; `hasSettings = !!settingsComponent` → the `[settings]` button is greyed for all. Confirmed. (Leg B built that button greyed-for-all *on purpose* — Leg C feeds one descriptor and it lights up for that row alone.)

### The three locks (Joe, 2026-07-17)

1. **D-B → reserved `D-120`.** The formal decision (settings mechanism = **component-per-plugin, hosted in the Settings content pane**; the declarative `settings_schema` superseded, not built) takes the number **D-120** — but per the project's decision hygiene (D-117/D-118/D-119: a decision enters `DECISIONS.md` when *built and looked at*, not predicted), the **D-120 entry is minted at Leg C CLOSE**, not now. The reserved number is fixed; the record lands with the code — this session does **not** write `DECISIONS.md` (mirrors J-534, which locked D-A/D-B/D-C without minting numbers).
2. **Swap = REUSE.** Generalize the Leg-B drill-in from `detailId` to a single drill target carrying a **mode** (`{ id, mode: 'info' | 'settings' }`); the `settings` verb is **intercepted LOCALLY** in `settings-dialog` (like `info`), **never forwarded**; `app_client.handlePluginAction` untouched; a generic `{@const C = plugin.settingsComponent}<C />` is the third content-pane target. A second machine would be N-086 wrong-abstraction risk.
3. **Backdrop = B2 (one painted value).** `grid-plate` **stops being fully inert** — it reads **one** value from a new `$common` backdrop store and **branches its render**, so the writer is proven on the **painted DOM** (N-097: a setting that moves nothing on screen is an untested writer), not by getter alone. Being a `$common` widget it cannot import a shell store (W-3) → the value lives in `$common` (settings-component writes, plate reads, N-096); the **shell** mirrors it into `backgroundLive` (replacing the literal) and **persists** it via a new `uistate` session key mirroring `setSessionDisabled` (N-107 per-key merge, **zero Rust**), hydrated before `loadLayout`. **The *look* of the two states is Joe's → M-RP-SKIN.** The full static/generative/data-driven menu stays **`M-RP-BACKDROP`**, NOT this leg.

**Surfaced, not decided silently (both then locked):** whether the settings swap reuses the Leg-B detail-swap machinery (→ **reuse**) and where the minimal-backdrop line sits vs. `M-RP-BACKDROP` (→ **B2**, one painted value; the menu stays filed).

### Canonical (D-074)

`docs/xgen-settings-phase0.md` **v1.0→v1.1** (§2 D-B formalization note, §5 sharpened to B2, §6 Leg-C line, new **§9** the mechanic lock — the runbook's canonical source); runbook `tasks/M_RP_SETTINGS_C_MECHANISM.md` **NEW (v1.0 ACTIVE)** handed to Clair; `CLAUDE.md` PLAY (Leg C 🔒 design-locked → 🟢 PLAY); `docs/ROADMAP.md` v5.05→v5.06; this JOURNAL J-539. **No `DECISIONS.md` change** (D-120 minted at close). No new `core`, no new N.

**Next-active.** Clair implements Leg C from the runbook (design→already-locked): the drill-in generalization + local `settings` interception + generic mount, with `grid-plate` as the one fed tenant; then the `$common` backdrop store + the plate's one painted value + the grid-plate settings component + the shell binding & persistence. Chat re-drives every leg live 9222 after a full reload (N-132, baseline 99); `cargo test` stays 1517/0/62 IDENTICAL. At close: mint **D-120**, close the arc. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-539 + J-537 → this runbook → `docs/xgen-settings-phase0.md` §9.** Not pushed — Joe pushes.

---

## Entry J-538 — M-RP-PLUGIN-INSTALL filed: the Plugins pane action-bar is COMPOSED from cores, not the grid `shelf`; `pane-toolbar` filed as a D-069 extraction candidate

**Records-only (design). No code.** A design walk from Joe's question — are the grid shelves special, or can this bar recycle one? Grounded `shelf` (N-116): it is a grid-tailored `role="toolbar"` of command icon-faces (`onCommand`/S-7, never a bare click), a linear rove over `faces[]`, `position:'top'|'bottom'` = favourites/system, `data-empty` collapse. A search `textfield` is **not** a face and cannot be one without breaking the contract (different keyboard model — own tab stop + caret keys, not arrow-rove). **Verdict: do not recycle `shelf`.** The Plugins pane's top strip (search + info/filter + a `+` install button) is **composed shell-local** from `textfield` + `icon`-buttons; the static-header / scroll-body layout is already the Leg-B settings-content scroll model (a skin, not a component). `pane-toolbar` filed as a **D-069 extraction candidate** — built at the 4th sighting (members/rooms/spaces headers), not at N=1 (N-135). The `+` opens an install dialog over `AVAILABLE_CUSTOM` (grounded: the catalogue exists; D-119's `install()` path works) — the write-side the read-only list lacked.

**Filed milestone: `M-RP-PLUGIN-INSTALL — the "+" install affordance + the plugins pane action-bar`** (ID per Joe's go-ahead 2026-07-17; PENDING; slots after Leg C; gates nothing; rename stays Joe's call). Legs, visible-first: **A** the action-bar composed (search + `+`) as the static header · **B** `+` → install dialog over `AVAILABLE_CUSTOM` (D-119 reuse) · **C** file the `pane-toolbar` candidate + CDP verify. Zero Rust (frontend + the opaque-blob store). **One open for its kickoff:** search = local filter of the visible list vs. catalogue search to install from (decides one control or two). → ROADMAP · CLAUDE.md PLAY · N-135. No new D, no new `core`.

---

## Entry J-537 — M-RP-SETTINGS Leg B CLOSED: the plugin action row + the Settings window; disable/enable/persist lifecycle proven live; the closed-modal regression fixed and verified

**Feat `15c1cd9` [Clair, code-only, 20 files, +602/−74, on `main`]. ⚠️ RECORD CORRECTION (Rule 6): the handoff and the session kickoff both said “not pushed — Joe pushes,” but a fresh `git fetch` resolves `origin/main` to `15c1cd9` — the code IS on the remote. Recorded as pushed, not as pending.** Zero Rust · zero schema · zero sampler · zero `core` component (the only `core` touch is additive glyph-map DATA in `icons.ts`) — `git show --stat` = 20 files, no `.rs`, no `ui/sampler/` → `cargo test` **1517/0/62 IDENTICAL by construction**, sampler catalogue **328 unchanged**. A **heavy visible-first round**: Joe directed the one-line row model, per-plugin icons, the version line, and the entire Settings-**window** chrome live over HMR — the corrections below are **design evolution, not defects**. Chat re-drove every leg on the live client 9222 **after a full reload** (Rule 5 + N-132).

### What shipped

The five Leg-B mechanics, all descriptor-derived and honest (§4 of the base runbook): `PluginDescriptor.settingsComponent?` → `hasSettings` (undefined on every row → `[settings]` greyed for the real reason *“no settings”*); `installed.svelte` **disabled axis** (`disabled` `$state<Set>` + `disable`/`enable`/`isDisabled`/`hydrateDisabled` + a new **`mounted`** view = system + installed-not-disabled) — shell derives registry/titles from `mounted` (disabled custom **unmounts**), `plugin-list` reads `active` (disabled custom stays **listed**); the **action row** `[info][settings][disable][uninstall]` (native `<button>` composing core `Icon`, `aria-disabled`+guarded onclick, one `onAction(id,verb)` seam — W-3 held, `data-verb`/`data-plugin` the CDP hooks); a **leading per-plugin icon** (host-tinted in skin); **`info`** → a real `plugin-detail` `<dl>` view. Persistence via `session.disabled` (N-107 per-key merge, **zero Rust**), `hydrateDisabled` before `loadLayout`.

**Row model + window chrome (Joe, visible-first):** one line per plugin `[icon] Name  vX.Y.Z  … [info][settings][disable][uninstall]`; description + the host·delivery·surface axes **moved out of the row into the `info` view**; the `[system]/[user]` badge **replaced by the version** (new descriptor field `version?`='1.0.0' placeholder, real versions later via D-118). **11 real Material Icons** (Apache-2.0, colour-free per D-110, provenance `.svg` in `ui/assets/icons/`, D-108). Settings became a **window**: own header (round Back `<` + context title + round `×`), a solid window-linked area at `--settings-inset: 120px` (resizes with the main window, equal gap 4 edges), independently scrolling columns, thin scrollbars.

### Measured — Chat re-drove every leg (live 9222, after a full reload)

| leg | result |
|---|---|
| baseline | **99 === unique 99**, quiescent — `sel:null`, `installed:[]`, `disabled:[]`, `named:[]`, unlocked, default layout. **+13 from Leg A's 86** (4 system rows × net +3 [−`__desc` −`__meta` −badge +`__icon` +4 action icons] = +12, +1 header `settings__close`). Enumerated, `count===unique`. |
| ⚠️ closed-modal regression | fresh **and** after open→close: `display:none`, `:modal false`, `open` attr false, not in flow, `rectH:0`, registry back to **99 no leak**. The `.dialog[open]:has(.settings)` scope fix holds — N-134. |
| open @ Plugins | gear (`shelf-face#app-shelf-bottom__0`, `widget.manager`) → `section:'plugins'`, `:modal` **true** (`.matches(':modal')`), `display:flex`, **inset L/T/R/B all 120** (window-linked, equal gap). |
| action-row honesty | 4 system rows: `info` live, `settings`/`disable`/`uninstall` **all `aria-disabled`** — each for a plugin-true reason (no settings / system can't disable / system can't uninstall). `badgeCount:0` (badge gone); rows paint `v1.0.0`. |
| info detail | click info → `plugin-detail` real `<dl>` painted (self-panel: Name/Version/Id/Kind=system/Host=client/Delivery=compiled/Surface=region/Description), header → plugin name + Back; count swaps **99→84** (list unmounts, detail mounts); **Back → 99 exact**. |
| disable/enable/persist/uninstall | **99 → install 114 → disable 105** (widget `region-connection-stats__*` unmounts, row stays listed, `session.disabled:['connection-stats']`) **→ full reload 105** (survives on disk, still unmounted) **→ enable 114** (re-injects) **→ uninstall 99===99** (`csGone`, no leak, no blank). Every transition exact. |
| gates | `vite build` **175** (Leg A 174 + `plugin-detail.svelte`) · `npm test` **77** (4 files) · scope 20 files 0 `.rs`/0 sampler → `cargo test` **1517/0/62 IDENTICAL by construction**. **Cargo NOT re-run live**: the running `tauri dev` client holds `xgen-client.exe` (N-117/J-511); IDENTICAL is guaranteed by the diff being 100% frontend, and Clair ran it real per the handoff §2. Client left pristine (empty store, `sel:null`, 99). |

### Flags carried (Rule 6 — surfaced, not absorbed)

**⚠️ M-RP-DIALOG-CHROME — dialog header/footer-snippet extraction FILED (J-512 D9, 2nd `:has()` footer suppression).** Settings suppresses the stock `dialog` title+footer to own its header/`×`; that is the second `:has()` chrome hack (after the 6.1k footer suppression), so a `dialog`-owned custom-chrome slot is now owed as its **own milestone** (About/UI-state/Settings would share one mechanism). Joe's framing settles the tension: *Settings is a WINDOW, not a common dialog — the divergence is the point*; the extraction is housekeeping, not a blocker. **⚠️ The ID `M-RP-DIALOG-CHROME` is provisional — Joe to bless per Rule 8.** — `version='1.0.0'` is a declared placeholder (real per-plugin versions via D-118 / M-RP-PLUGINS-NODE) — host-tint icons (module red / widget blue; all rows `client`→blue today) + greyed-legible uninstall-on-system are **PROVISIONAL → M-RP-SKIN** (Joe's absent-vs-greyed call) — action buttons are native `<button>`+core `Icon` (core `Button` is text-only; the shelf-face/menu-item precedent) — the closed-modal `[open]`-scope fix is **N-134**.

### Doc-bridge

JOURNAL J-537 (this) + CLAUDE.md PLAY (Leg B CLOSED, measured baseline 99) + ROADMAP (Leg B ✅ → next-active **Leg C** = the settings mechanism D-B + the `grid-plate` backdrop setting in the content pane; M-RP-DIALOG-CHROME filed) + `ui/docs/xgen-ui-notes.md` N-134. **No new `D`, no new `core`.** → next-active **M-RP-SETTINGS Leg C**.

---

## Entry J-536 — M-RP-SETTINGS Leg B: GO — rover extraction DEFERRED (5th instance recorded against M-RP-ROVING); action-row runbook handed to Clair

**Records-only (design/handoff). No code, no Rust, no registry change.** Session-open presented the Leg-B plan — five descriptor-derived mechanics + the feeder contract, runbook `tasks/M_RP_SETTINGS_B_ACTION_ROW.md` — and Joe said **go**. Two things settled at handoff.

### The one open decision — the rover — RESOLVED: defer

Leg A's Settings sidebar is a roving-tabindex list — the arguable **5th** independent rove instance, past D-069's four-recurrence bar (`entity-panel` · `menu-bar` · `menu` · `shelf`, the fourth copied deliberately at 6.1i). **Joe's call: do NOT extract mid-arc.** Leg B adds no new rover, so extracting now would only widen the blast radius across four closed components — the 6.1k `dialog`-footer lesson: a shared-helper refactor buried in a feature arc makes that arc's registry delta unreadable. The 5th-instance pressure is recorded against **M-RP-ROVING — extract the roving-tabindex helper** (its own milestone, never a rider); Leg B proceeds on the shell-local copy. *The extraction is owed — its forcing function is a shared refactor, not this leg.*

### Two appearance calls flagged to Joe's lane (non-blocking)

Both ship PROVISIONAL → M-RP-SKIN and gate nothing: the exact **meta** fields on the row line (§3), and **uninstall on system rows — absent vs. greyed-legible** (§4). The mechanic is built regardless; the look is Joe's.

### Grounding re-confirmed before handoff (N-116)

Grepped live, not remembered: `PluginDescriptor` (`registry.ts`) still carries no `settingsComponent`/`hasSettings`; `installed.svelte.ts` still exposes only `active` (no `disabled`/`mounted`/`disable`/`enable`). Runbook §2's grounding holds — the five mechanics are genuinely unbuilt. Tree clean at open (`0 0` vs `origin/main`; `473b991` + `eac5398` present). Baseline to cite going forward: **86** (quiescent, empty store).

### Handoff

`tasks/M_RP_SETTINGS_B_ACTION_ROW.md` → v1.1, gains a §0.5 Handoff — GO block carrying the rover decision. Clair implements per §3–§5; Chat re-drives every verification leg live 9222 after a full reload; Joe pushes. → CLAUDE.md PLAY · ROADMAP (M-RP-ROVING 5th-instance trigger · SETTINGS Leg B GO).

---

## Entry J-535 — M-RP-SETTINGS Leg A CLOSED: the one Discord-shaped Settings modal stands up; `plugin-list` re-hosted as the Plugins section, About reused as the About section, `plugins-dialog` absorbed

**Commit `473b991` [Clair, code-only, 6 files, +276/−94, on `main`, pushed by Joe]. Zero Rust · zero schema (`version` stays 3) · zero sampler · zero `core`/`common`** (confirmed on the commit: all six files are `ui/client/src` + `ui/assets/skin.css`; `cargo test` **1517/0/62 IDENTICAL by construction**). Clair built; Chat re-drove every leg on the live client 9222 **after a full reload** (Rule 5 + N-132). The running build is `473b991` (About "Built" reads it).

### The one sentence

The **one** Discord-shaped Settings modal now stands up — a category sidebar (~¼) + a content pane that swaps per selection, wrapping the core `dialog`. The read-only `plugin-list` is re-hosted as its **Plugins** section; the About body is extracted (`about-content.svelte`) and reused as its **About** section; both entry points route (gear → Settings @ Plugins, new **File ▸ Settings** → Settings @ default); `plugins-dialog` is **absorbed and gone**.

### What shipped (6 files)

`settings-dialog.svelte` (NEW) — wraps core `dialog`; two-pane; getter `settings#settings-body {section, sectionCount, open}`; deep-link via a `section` prop + an `$effect` landing on `section ?? default` when `open` flips. `about-content.svelte` (NEW) — the About body extracted for two mounts (parametrized by `idPrefix`: default `about` keeps the C3 ids byte-identical; Settings uses `settings-about` → no `data-debug-id` collision, both modals always-mounted). `about-dialog.svelte` — slimmed to `<Dialog><AboutContent/></Dialog>`; Help ▸ About untouched. `app_client.svelte` — `PluginsDialog`→`SettingsDialog`; `settingsOpen`/`settingsSection`; `widget.manager`→Settings@Plugins, new `settings.open`→Settings@default; File = `Settings · Restart · —— · Exit`. `plugins-dialog.svelte` — **removed** (absorbed). `skin.css` — `.settings-*` two-pane block, PROVISIONAL → M-RP-SKIN.

### Measured — Chat re-drove every leg (live 9222, after a full reload)

| leg | result |
|---|---|
| V0 baseline | full reload → **86 === unique 86** · `sel:null` · empty store (no saved-state picker). |
| V1 gear entry | gear (`shelf-face#app-shelf-bottom__0`, `widget.manager`) → `settings#settings-body {section:'plugins', sectionCount:2, open:true}`; `:modal` **true** (open `<dialog>` `.matches(':modal')`, never the attribute); `openId dialog#settings`. |
| V2 plugins-dialog gone | `ids().indexOf('dialog#plugins') < 0` — absorbed; file deleted. |
| V3 sidebar swap | click `[data-section=about]` → `section` `plugins`→`about`; **DOM visibility swapped**: `plugin-list#plugin-list` `offsetParent===null` (hidden), `label#settings-about-name` `offsetParent!==null` (shown); About renders real data (Built `473b991`). |
| V4 close / no leak | `button#settings__close` → `dialog#settings.open false`, no open `<dialog>`, **86 === 86** (clean return, no leak). |
| V5 File entry (source) | `app_client.svelte`: `{label:'Settings', command:'settings.open'}` **directly above** `{label:'Restart'}`; `settings.open` sets `settingsSection=null` (→ default `about`) + opens; gear sets `'plugins'`; `<SettingsDialog bind:open section={settingsSection}/>` mounted; `PluginsDialog` import gone. File menu `itemCount 4` (was 3). |
| V6 About preserved | Help ▸ About intact; the original `about-*` ids (12) unchanged (C3 preserved), `settings-about-*` (12) are the reuse. |
| V7 scope/gates | `git show --stat`: 6 files, no `.rs`/sampler/`core`/`common` → `cargo test` **1517/0/62 IDENTICAL** · `npm test` **77** · `vite build` **174** (was 173; +2 new −1 removed). |

### Delta enumerated (+13; 73 → 86)

−2 (`dialog#plugins`, `button#plugins__close`) + 3 (`dialog#settings`, `button#settings__close`, `settings#settings-body`) + 12 (`settings-about-*`: 1 image + 10 labels + 1 link) = **+13**. The 12 original `about-*` ids remain intact.

### ⚠️ Record correction — J-534's "67" baseline was STALE (Clair caught it; Chat confirms)

J-534 wrote *"Registry unchanged (67, quiescent/empty-store baseline from J-533's arc)"* — a number **re-quoted from the pre-arc era without re-measuring** (the N-105 shape). The true pre-Leg-A live baseline is **73** (69 at J-529 + 4 plate at J-532; connstats adds nothing when uninstalled/empty-store). Chat's post-reload read of **86 === unique 86** with the exact **+13** confirms 73 was the base. **The next milestone cites 86** (quiescent / empty-store / no-selection), not 67.

### Deviations / design flags (Rule 6 — all Clair's, all accepted on Chat's re-drive)

**Second real section = About** (not deferred — `aboutInfo` was already fetched, so it is genuinely ready) — forced the `about-content` extraction + `idPrefix` param; §3.2's proposal made concrete, and it exercises the sidebar swap honestly (N-091). **Sidebar rover = a shell-local linear-rove copy** (selection-follows-focus) — neither core rover fit (`entity-panel` is data-dependent; `shelf` is horizontal transient dispatch); it is chrome, not a new `core` component (catalogue stays 328), so `M-RP-ROVING` stays filed — **noted: arguably a 5th independent rove instance, extraction-pressure for Joe/Chat to weigh.** **Content pane = all sections mounted, inactive `display:none`** (the M-RP3.2 tabbed-sampler + always-mounted-dialog precedent) — swap is CDP-observable via the getter + `offsetParent`. **Kept the stock `dialog` Close footer** — deliberately did NOT reach for `:has()` suppression (that would be the 2nd D9 recurrence → the footer-snippet extraction); Close-at-bottom is honest for a read-only leg, X-in-corner is Joe's via M-RP-SKIN.

### Housekeeping (out of scope, filed)

Stale comment `plugin-list.svelte:26` ("the host (the plugins-dialog)") left in `$common` — sweep in a follow-up if wanted (scope kept tight). Screenshots `temp/cdp-shot-client.png`.

### State

**M-RP-SETTINGS Leg A DONE.** The one Settings modal exists, the plugin manager is its Plugins section, both entry points route, `plugins-dialog` is gone. **Next-active = Leg B** (`tasks/M_RP_SETTINGS_B_ACTION_ROW.md`) — the `[info][settings][disable][uninstall]` action row + leading kind-glyph + `session.disabled` (this IS M-RP6.1m). Then **Leg C** (the settings mechanism, D-B + the `grid-plate` backdrop setting in the content pane). No new D, no new `core`, sampler 328 unchanged. Records: CLAUDE.md PLAY · ROADMAP · this entry. Per D-065 + Rule 5 + Rule 6.

---

## Entry J-534 — M-RP-SETTINGS DESIGN LOCKED: one Discord-shaped Settings modal (Plugins is a section), J-513 → B, no OS window

**Design / records-only, NO CODE.** A design walk this session locked the shape of the next milestone, `M-RP-SETTINGS`. Canonical Phase-0 doc shipped at `docs/xgen-settings-phase0.md` (v1.0, ACTIVE). Registry unchanged (67, quiescent/empty-store baseline from J-533's arc). Clair not engaged; the Leg-A runbook is filed for handoff.

### The one sentence

`M-RP-SETTINGS` stands up **one** Settings surface — an in-DOM modal shaped like Discord's (a left **category menu ~¼ width + a content pane** that swaps per selection, compact; never a new OS window). The **plugin manager is a section inside it** (the `Plugins` category). **Two entry points, one modal:** the `gear` shelf face opens Settings on the **Plugins** section; a new **File ▸ Settings** item (above Restart) opens it on the default section. It proves the per-plugin settings mechanism end-to-end via the grid backdrop, which is `grid-plate`'s own setting.

### Three decisions (Joe-locked this session)

- **D-A — "Settings window" = an in-DOM modal.** D-112 penciled Settings as `surface:'window'`; Joe's reading is that `window` means a **standalone modal area** (the mechanism we already have for About/Plugins), NOT a second OS window. **Grounded:** `xgen-client/tauri.conf.json` has exactly one window; the four "dialogs" (`about-dialog` / `plugins-dialog` / `uistate-{save,load}-dialog`) are in-DOM Svelte modals wrapping the core `dialog`. So there is **no frame arc** — no second Vite entry (the client has one `index.html`), no second CDP target, no new typed Rust geometry struct (D-114/D-115 untouched).
- **D-B — J-513 → B (component-per-plugin).** The J-513 gate (*how a plugin's settings get drawn*) was binding-deferred until the grid works. The grid works. **Resolved: B** — a plugin that has settings ships its own settings component; the modal hosts it (the shipped widget-tier pattern; `substitutions-editor` was its first instance). The declarative `settings_schema` auto-render (Ch6 §6.8.2/§6.8.5, **zero lines exist**) is not built and is superseded as a path (*"it does not need to be yet another widget system"*, Joe). A technical/mechanism choice; the look stays Joe's. *A formal `D`-number is Joe's to bless when Leg C builds it.*
- **D-C — ONE Discord-shaped Settings modal; the gear deep-links to its Plugins section.** Settled across the walk (first draft: gear opens Settings, absorbs `plugins-dialog`; second: two separate modals; **final: ONE modal, plugin manager is a category**). Sections = app-level categories + a **Plugins** category (the `plugin-list` rows + the action row) + per-plugin settings in the content pane. The `gear` (`widget.manager`) opens Settings **on the Plugins section**; **File ▸ Settings** (new `settings.open` command + a new File item **above Restart** — grounded: File is `Restart · —— · Exit` today) opens it on the **default** section. `plugins-dialog.svelte` is **absorbed**. The deep-link is trivial (the modal takes a `section` arg).

### The row model (Joe's compact one-liner vision; appearance is Joe's, mechanics are mine)

`[kind-glyph] Official Name · meta · [info][settings][disable][uninstall]` — icon-buttons with hover tooltips (order Joe-set). Leading red/blue glyph = **module vs widget**, the which-one derived from the `host` axis (`node`=module · `client`=widget). Every button state is **descriptor-derived** and greyed only for a reason true of that plugin (W-13 rendered); a verb never built for anyone ships **absent**, never dead-grey (J-500 / 6.1j). Feeders: **info** = a detail view (built Leg A) · **settings** = the plugin's own `settingsComponent` (greyed until a plugin ships one) · **disable** = a new `session.disabled` set (v1 user-toggle; auto-disable-on-version-incompat is a second feeder needing D-118 semver, future) · **uninstall** = D-119 uninstall (custom-only; system absent/greyed-legible).

### Grounding findings worth the record

- **`grid-plate` is the settings mechanism's first real tenant**, not `substitutions-editor` — the backdrop *is* grid-plate's own setting, so Leg C proves D-B and delivers a minimal backdrop setting in one leg. The full backdrop-type menu (static/generative/data-driven, base-vs-stack) is filed as `M-RP-BACKDROP`, not this arc.
- The current wiring (grounded, N-116): `gear` face → `widget.manager` → `pluginsOpen` → `<PluginsDialog>` hosting the read-only `plugin-list` widget; `__XGEN_PLUGINS__` DEV bridge (install/uninstall + leaf + persist) already lives in `app_client`; `PluginDescriptor` carries `host`/`delivery`/`surface`/`kind` but no `settingsComponent`/`hasSettings` yet.

### Leg roadmap (design-only)

**Leg A** — the Settings shell + Plugins section (the Discord-shaped modal: sidebar ~¼ + content pane; the read-only `plugin-list` as the Plugins section; wire gear → Settings @ Plugins + File ▸ Settings @ default; `plugins-dialog` absorbed; no new verbs, visible-first). **Leg B** — the action row (the Plugins-section rows gain `[info][settings][disable][uninstall]` + kind-glyph; feeders info/disable/uninstall; this IS M-RP6.1m). **Leg C** — the settings mechanism (D-B) + the backdrop setting in the content pane (`grid-plate` first tenant). Each leg: real client 9222 only, Rule 5 re-drive, baseline read quiescent after a full reload (N-132), `cargo test` 1517/0/62 IDENTICAL (all frontend; the opaque-blob store path, D-114).

### State

**M-RP-SETTINGS DESIGN LOCKED.** Next-active = **Leg A** (the Settings shell + Plugins section), runbook `tasks/M_RP_SETTINGS_A_SHELL.md` (Leg B = `tasks/M_RP_SETTINGS_B_ACTION_ROW.md`, PENDING). Filed follow-ons: `M-RP-BACKDROP` (backdrop-type menu) · `M-RP-PLUGINS-NODE` (node-module rows, the red glyph's first real rows) · auto-disable-on-incompat (D-118 semver). Records: Phase-0 doc v1.0 · CLAUDE.md PLAY · ROADMAP. **No new D yet** (D-B awaits its build; D-A/D-C are D-112 readings). Per D-065 + D-069 + Rule 5.

---

## Entry J-533 — M-RP-CONNSTATS CLOSED: the first `kind:'custom'` widget, and the runtime install → dock → uninstall path it forced into being (→ D-119)

**Commits `c747729` (the milestone, 9 files) + `7f24b19` (post-close: Speed/Bandwidth N/A rows, owner override) [Clair, code-only, on main, NOT pushed — Joe pushes with this doc-bridge]. Zero Rust · zero schema (`version` stays 3) · zero sampler** (confirmed on the commits: no `.rs`, no `ui/sampler/`; `cargo test` **1517/0/62 IDENTICAL** by construction). Clair built; Chat re-drove every leg on the live client 9222 **after a full reload** (Rule 5 + N-132). ID `M-RP-CONNSTATS` confirmed by Joe; the runtime-install pattern minted as **D-119**.

### The one sentence

The first `kind:'custom'` widget (`connection-stats`) and, with it, the runtime **install → dock → uninstall** lifecycle that did not exist (the registry was a static `const`): a custom is registered into a now-**reactive** registry, its region **injected** into the layout, **removed without blanking**, and its installed-state **persisted per-device** so a reload re-registers it **before `loadLayout`** instead of W-13-dropping it. The pattern is **D-119**.

### What shipped (9 files + the N/A follow-up)

`installed.svelte.ts` (NEW, `$common`) — the reactive `$state` installed-set + `active` getter (`[...CLIENT_PLUGINS, ...installed customs]`) + install/uninstall/hydrate/isInstalled/ids (N-096, one source several readers). `connection-stats.svelte` (NEW, `$common`) — a compact `{label,value}[]` metric-row list reading `selfState` (D-067, no new channel / no Rust); composes `Led`+`Label`; `rowCount` render-truth; rows absent when the source is null (N-060). `registry.ts` — `AVAILABLE_CUSTOM` (the first runtime-installable rows), separate from `CLIENT_PLUGINS`. `mutate.ts` — `insertLeaf` + a **total** `removeRegion` wrapper (remove-without-blanking = the existing collapse-degenerate); no new algebra. `layout-default.ts` — the `widgetRegistry`/`bgWidgets`/`REGION_TITLES` consts became **pure builders**. `uistate.svelte.ts` — `session.installed?: string[]` + `setSessionInstalled` + the N-107 per-key merge (geometry stays Rust's; writes even `[]`). `app_client.svelte` — reactive registries (`$derived` off `installed.active`); install/uninstall wrappers (set + leaf + persist); boot hydrates the installed-set **BEFORE** `loadLayout`; the `__XGEN_PLUGINS__` DEV bridge. `plugin-list.svelte` — reads `installed.active` so an installed custom shows as a `[user]` row (read-only; the action row is M-RP6.1m). `skin.css` — `.connection-stats` (PROVISIONAL → M-RP-SKIN). **Follow-up `7f24b19`:** two trailing `Speed · N/A` / `Bandwidth · N/A` reminder rows — see the N-091 override below.

### Measured — every leg (live 9222, Chat re-drove after a full reload)

| leg | result |
|---|---|
| V0 baseline | **73 === unique 73** · `installed:[]` · `available:['connection-stats']` · leafCount 8 · `session.installed` absent. |
| V1 install | **73→85 (+12)**, `count===unique`, leafCount 8→9, `droppedCount 0`. **+12 enumerated:** `region-tile#region-connection-stats` + `connection-stats#region-connection-stats` + `led#…__state-led` + **6** value `label#region-connection-stats__{state,registered,endpoint,spaces,speed,bandwidth}` + 3 `label#plugin-list__connection-stats__{name,desc,meta}`. |
| V2 render | widget G `{state:READY, rowCount:6}`; painted rows **State=Ready** (led `rgb(45,122,58)` = `--ok`) · Registered=Yes · Endpoint=`ws://127.0.0.1:8080/xgen` · Spaces joined=0 · Speed=N/A · Bandwidth=N/A. |
| V3 uninstall | **85→73 exact**, leafCount 8, `droppedCount 0`, `docNoScroll:true` — remove-without-blanking. |
| V4 persist (load-bearing) | install → **full reload** → widget **survives** (85, `droppedCount 0`, `installed:[connection-stats]`, renders live "State Ready…") → uninstall → reload → **gone** (73, `installed:[]`). |
| V5 W-13 honesty | a phantom **unregistered** `connection-stats` leaf injected into the layout → `droppedCount:1`, leafCount 8, shell present, `docNoScroll:true`; restored clean. The boot-order dependency is real. |
| V7 gates | no `.rs` → `cargo test` **1517/0/62 IDENTICAL** · `npm test` **77** · `vite build` **173** (was 171; +2 = the two new `$common` files). |

### The +12 vs Clair's +10 — not a discrepancy, the two commits

Clair measured **+10** on `c747729` (4 value rows). The post-close `7f24b19` added the two N/A value rows (+2). The combined tree Chat measured is **+12** — both correct for their commit. No phantom this time; Chat reloaded from the start (N-132 held).

### The live demo reproduced on Chat's own drive

With Joe's node up on `127.0.0.1:8080` (matching the client endpoint), `connection-stats` reflects the REAL connection: State=**Ready**, led green (`--ok`) — and the self-panel + status-bar are green too. **One store, three views** (`selfState`), exactly D3's point. The widget survived a client restart (persisted install). The N/A rows render honest **N/A**, not fabricated data.

### ⚠️ The N-091 override — an owner-sanctioned reversal, recorded not drifted (→ N-133)

The runbook §4.1 said *no placeholder metric rows* (N-091: an unfed row is an unverified branch). After the live demo, **Joe requested** two trailing `Speed · N/A` / `Bandwidth · N/A` reminder rows. This is a **deliberate owner override**, justified because the rows are **honest `N/A`** (no fabricated numbers) rather than an unfed branch feigning data, and the owner is informed the data source does not exist yet (grepped: zero byte/RTT/throughput accounting anywhere, no resident socket — the M-RP6.6 arc). `rowCount → 6` (render-truth). When live-metrics infrastructure lands, each `N/A` becomes a real value; the rows are already there. **Recorded so the reversal is visible, not silent drift** (D-065).

### Deviations (Rule 6 — flagged, not absorbed; all Clair's, all realization details)

**Reactive registries live in `app_client`, not a `$common` store** — §4.5 anticipated it; W-3 forces it (`RegionPlaceholder` is shell-local). The `layout-default` consts became **pure builders**. **`plugin-list.svelte` — a 9th file** beyond §3's list, required by V1's plugin-list row; read-only (consistent with D4, defers the action row to M-RP6.1m); completes the N-096 two-readers-one-source. **Titles made reactive** — else the custom tile shows its raw id, not "Connection Stats". **`removeRegion` wrapper over raw `removeLeaf`** — took the runbook's "or a thin wrapper" branch to keep `mutate.ts`'s public surface **total** (no null leak). **Fresh-`Set` reactivity** — §4.2 said "mutate the set"; Svelte 5 `$state` does not react to `Set.add`/`.delete`, so reassign a fresh `Set` (folded into D-119). **Boot reorder** — hydrate + `installed.hydrate` + lock seed moved before `loadLayout` (D3-required; idempotent, no-Tauri-safe). **The +12 install delta** — the runbook's V1 named "rendered rows + a plugin-list row (+3)" without the region-tile frame or the exact subtree; enumerated above.

### Live-env findings (worth the record)

`connection-stats` correctly flips **Disconnected → Ready** (led `--err` → `--ok`) when the node comes up — one store, three views. **The in-app File ▸ Restart is a no-op under `tauri dev`** (same PID; needs a real process relaunch — the filed M-RP-RESTART caveat). **Speed/Bandwidth are unavailable by INFRASTRUCTURE, not by the widget** (no byte/RTT accounting, no resident socket — one-shot startup connect) — that is the **M-RP6.6** arc; the N/A rows' eventual data home.

### State

**M-RP-CONNSTATS DONE. The install/uninstall half of the plugin lifecycle is closed** — both exemplars are shipped (plate = containment/mount; connection-stats = install→dock→uninstall), and the runtime-install pattern is locked as **D-119**. **Next-active = `M-RP-SETTINGS`** (per J-532's order: plate → connection-stats → settings) — the plugin manager reuses D-119's install path for the real install/uninstall UI (the `[Remove]` action row, M-RP6.1m) **and** delivers the backdrop-setting that binds the plate's `backgroundLive` + a user-chosen/persisted backdrop (the discussed background-type menu lands there). Then, independently, **M-RP6.6** (the resident → transport accounting → metrics push) fills the N/A rows with real numbers. Still filed: `M-RP-SKIN` · M-RP7.7 (node inherits frame+grid) ⏸️ · `M-RP-ROVING` · `M-RP-MOVE-KBD` · `M-RP-RESTART`. Records: **D-119** · ui-notes **N-133** · components registry bump (first `kind:'custom'` + `AVAILABLE_CUSTOM`) · ROADMAP · CLAUDE.md PLAY. Per D-065 + D-074 + Rule 5 + Rule 6.

---

## Entry J-532 — M-RP-PLATE CLOSED: the grid backdrop plate — the dev raster becomes an element, and a hole reveals it while capturing nothing

**Feat `6bd17c8` [Clair, code-only, 6 files, +157/−9, on main, NOT pushed — Joe pushes with this doc-bridge]. Zero Rust · zero schema (`version` stays 3) · zero sampler · zero node** (confirmed on the commit: no `.rs`, no `ui/sampler/`, no `ui/node/`). Clair built; Chat re-drove every verification leg on the live client 9222 (Rule 5).

### The one sentence

The dev raster was promoted from a `.region-shell` CSS `background-image` (paint) to a real **grid-wide background-widget socket + one inert system plate widget** (element) — the exact `message-stream` `background?: WidgetMount[]` socket, one level up — so the backdrop mounts through the plugin registry, shows through every hole/seam/perimeter, and **never captures the pointer** (`pointer-events:none`; the instant a hole is clickable it has an address and D-116 falls).

### What shipped (D1–D4, Chat-locked under §0 autonomy over mechanics; appearance is Joe's)

**D1 — the socket lives ON `region-shell`** (core): `background?: WidgetMount[]` + `backgroundLive?` (default true, threaded into each mount) + a **separate `bgWidgets` registry** (kept apart from the region-id-keyed tile `widgets` map so `grid-plate` can never be mistaken for a leaf). Rendered as an `inset:0 pointer-events:none` backdrop wrapper behind `RegionNode`, drag overlay still last. G gains `backgroundMountCount` + `backgroundLive`. The node inherits it free at M-RP7.7. **D2 — the plate is `kind:'system' · delivery:'compiled' · surface:'none'`** — the promoted dev raster, the **first `surface:'none'` row WITH a `component`** (plugin-list has none; the two-shape split is now real: no-component → shell mounts directly into a dialog; with-component → shell mounts into a named host socket). **D3 — inert now:** one fixed plate, `backgroundLive` exposed-but-unbound (the message-stream "binding deferred" precedent); the live-switchable backdrop + its user setting ride **M-RP-SETTINGS** (gated on the J-513 settings-mechanism collision, unchanged). **D4 — appearance = the current raster**, now an element, PROVISIONAL → M-RP-SKIN. Reserved-nothing held: no descriptor key, no store key, no persistence — the inert plate is a constant `DEFAULT_BACKGROUND` from `layout-default.ts`.

### The load-bearing proof (V3, D-116) — Chat-measured on 9222

Backdrop `pointer-events:none`; `elementFromPoint` at the perimeter → `region-split`, at a tile → `region-tile-body` — **never the plate**. A reactive backdrop is fine; a clickable one would re-open §7.1's lattice argument. It does not.

### Measured — every leg (live 9222, Chat re-drove; Clair's numbers reproduced)

| leg | result |
|---|---|
| V1 registry | quiescent **73 === unique 73** (after a full reload — see the correction below); plate delta **+4** enumerated: `grid-plate#grid-plate` (mount) + `label#plugin-list__grid-plate__{name,desc,meta}` (its plugin-list row). 69 (J-529) + 4 = 73. |
| V2 shows-through | fold `members` → `data-collapsed:width` / `data-fold-mode:across`, **22px strip**, a **230px hole**; hole point → `region-split` (not tile, not plate), `plateCoversHolePoint:true`. Unfold → exact restore (73). |
| V3 pointer (D-116) | backdrop `pointer-events:none`; perimeter→`region-split`, tile→`region-tile-body`. |
| V4 backgroundLive | shell G `backgroundMountCount:1, backgroundLive:true`; plate getter `{backgroundLive:true}`. |
| V5 W-13 drop | unknown id → `backgroundMountCount:0`, plate+backdrop gone, **73→72**, shell + 8 tiles present, no crash; restore → **73**, mountCount 1. |
| V6 raster relocation | `.region-shell` `backgroundImage:none` + `--s5` (`rgb(52,59,71)`) kept; `.grid-plate` carries the radial-gradient. |
| V7 gates | no `.rs` → `cargo test` **1517/0/62 IDENTICAL by construction** · `npm test` **77** · `vite build` **171 modules**. |

### ⚠️ The Rule-5 correction — a stale dev client nearly put MY wrong number in the record (N-132)

Chat's FIRST quiescent read on the long-running `tauri dev` client showed **77**, and Chat drafted it as a deviation — *"Clair's 73 does not reproduce."* **It was the other way round.** A full page reload cleared the count to a stable **73** (`===unique`, six reads): the long-running client had accumulated **4 stale-but-UNIQUE** envelope registrations across HMR reloads (Clair's build + Chat's drive), which is exactly why `count===unique` still held at 77 and the pollution looked clean. **Clair's 73 was correct all along.** The reload was the disambiguator; the phantom 77 never entered the record. *The N-099 family, a new variant: a stale subject returns a self-consistent, flattering answer — and an agent does not get to stamp its own un-reloaded number as canonical while flagging the correct one as non-reproducing.* → **N-132: read a quiescent registry baseline AFTER a full reload, never on an accumulated dev session.**

### Deviations (Rule 6 — flagged, not absorbed; all Clair's, all accepted)

**Registry +4, not the runbook's +1** — the +3 is `grid-plate`'s own plugin-list row (`label#plugin-list__grid-plate__{name,desc,meta}`), the honest **N-096** consequence of registering it as a `kind:'system'` plugin; the plugin manager now shows a 4th row ("Grid Backdrop — [system] · client · compiled · none", Joe-eye-confirmed). Correct per D5; §3 of the runbook did not call it out. **Stacking realized by lifting the tree, not sinking the backdrop** — §4.6's "no explicit z-index needed" hint was wrong on grounding (a positioned `z:0` backdrop paints over non-positioned tiles). Fix: root split `position:relative; z-index:1` (the message-stream rows z:1 shape); `.region-shell` gets only `position:relative` (NOT a stacking context), so the fixed drag-overlay `z:4000` is untouched. A real finding, the right fix. **`--s5` kept on `.region-shell`** (literal §4.6; only image+size moved) → the W-13 drop degrades to a clean `--s5` gap, not a blank. **DEV bridge `setBackground`/`get background`** added to `__XGEN_LAYOUT__` (§5 V5-sanctioned), DCE'd in release. **Plate self-ids to `grid-plate`** (the `self-panel` `id = region-${regionId}` shape) → stable enumerable registration, not envelope's `++ordinal` fallback.

### State

**M-RP-PLATE DONE. The CONTAINMENT/MOUNT half of the plugin lifecycle is closed** — a `surface:'none'` grid-wide widget now has a proven home, and the socket ships FED (D-065/N-091). The dock-engine arc's mechanics remain complete; the plate is the first custom-widget-tier *content* exemplar. **Next-active = connection-stats** (`surface:'region'`, `kind:'custom'`, the first traffic consumer + a natural node widget) — the **install→dock→uninstall** half, which needs a design walk + runbook before impl. Then **M-RP-SETTINGS** (binds `backgroundLive` + the user-chosen/persisted backdrop into exactly this socket). Still filed: `M-RP-SKIN` · M-RP7.7 (node inherits frame+grid) ⏸️ · `M-RP-ROVING` · `M-RP-MOVE-KBD` · `M-RP-RESTART` · `temperature-indicator` ⏸️. **No new D** (§4.5.1 + D-103/D-112/W-12 extension). Records: ui-notes **N-131** + **N-132** · components registry **v0.77** (73) · `docs/xgen-dock-engine-phase0.md` §4.5.1 (⏸️FILED → ✅BUILT) · ROADMAP · CLAUDE.md PLAY. Per D-065 + D-074 + Rule 5 + Rule 6.

---

## Entry J-531 — D-118 LOCKED: the plugin package (one zip + root manifest, universal for node + client); plus the node frame/grid postponed and the client's next exemplar set

**Architecture + naming convention. No code.** A design conversation this session settled how plugins are distributed and installed, and re-pointed the near-term work.

### D-118 — the package

Every out-of-tree plugin — either app, any delivery kind — ships as ONE `.zip` with a manifest at its root. `host` / `delivery` / `surface` / `kind` (D-112) become manifest FIELDS, not separate on-disk shapes — one uniform unit the plugin manager enumerates, trust-badges, installs, removes. The manifest is readable **without executing or unpacking-to-live** (enumerate + trust-badge + route-by-host from the manifest alone). **D-085 unchanged:** a zip is transport, never a load path — `service` spawns out-of-process, `packaged` runs in the no-key sandbox (gated on S-1…S-6, D-113/S-7), nothing is `dlopen`ed into the node; `compiled` stays baked in, so every zip in `plugins\` is `kind: custom` by construction. Two-tier location (`[app_root]` bundled read-only · `[userdata]` user-writable — the app-vs-user lock). Discovery ≠ loading: the manager can list `plugins\` before the sandbox floor exists.

**Naming:** `pg{c|n}_<plugin-id>_r<YYMMDD>.zip` — `pgc` client / `pgn` node (the host axis + routing hint); `r` = single-letter release channel (room for `d`/`b`); date a human label. **No ui-vs-system token** (Joe): system plugins are `compiled` + non-removable and never appear as loose packages, so the namespace is custom-only, and the host badge is already `pgc`/`pgn`. A filename is a label + routing hint, never a trust descriptor — everything trust-relevant lives in the manifest. D-117 stays reserved for the fold axis; this takes D-118.

### Direction re-point (records catching up to the conversation)

- **M-RP7.7 (node inherits the frame + grid) → POSTPONED** until the client UI is complete (Joe). Port a FINISHED frame+grid to the node in ONE pass, not twice — the client still has appearance (M-RP-SKIN), real region content, and Settings ahead of it. The node is “working enough to simulate traffic,” which is what makes the client's connection widget real later. **D1 (a)** + **D2** (File menu = Restart / Shut Down; **hide-to-tray IS the native window X**, no new menu item — the shipped `CloseRequested`→`hide()` intercept becomes reachable the moment D3 flips `decorations:true`) recorded as 7.7 design intent; **D3** (native resizable window, twin-config, N-088 fix, **no status-bar grip** — native frame resizes) and **D4** (retire `app_node.svelte` legacy chrome → placeholder centre) are node-UI, deferred with 7.7.
- **Client next = the two custom-widget exemplars, plate first**, then connection-stats, then the plugin manager (`M-RP-SETTINGS`). Order is architectural: a custom widget is the HARDER, more general case than any system widget (system = always-present, easy; custom = install → registry → layout-inject → uninstall → remove-without-blanking) — prove the lifecycle on one instance before scaling. **`M-RP-PLATE`** (grid-background, `surface:'none'`, inert, no data source) is the most self-contained exemplar and closes the containment/mount question; **connection-stats** (`surface:'region'`, `kind:'custom'`, the first traffic consumer, a natural node widget too) closes install→dock→uninstall. System widgets fill in as live data matures; sub-element widgets (temperature etc.) ride INSIDE their host regions (D-061, §6.2), never as standalone panels — which is why “build all system widgets” is not a clean unit.
- **The open ground the exemplars close:** the plugin ARCHITECTURE is locked (D-112 taxonomy, D-103 region model, D-113/S-7 sandbox sequencing), but the install/uninstall LIFECYCLE and the compiled-LOADING mechanism are explicitly open (D-112) — the registry today is a static TS array (`CLIENT_PLUGINS`) with no runtime install path. That gap is what the exemplars exist to close.
- **`plugins\{client,node}` folder split** agreed (matches the host axis; the node's is the stricter — out-of-process / no-key sandbox only, never its address space).

### Records

DECISIONS **D-118** · CLAUDE.md PLAY (NEXT-ACTIVE re-pointed to M-RP-PLATE; 7.7 postponed; D-118 filed) · ROADMAP (7.7 ⏸, plate promoted, D-118 ref). No code; nothing pushed by Chat (Joe pushes).

---

## Entry J-530 — M-RP-SHELF-FRAME CLOSED: fixed-height shelves — the empty strip holds its frame, and the grid stops reflowing

**Skin-only, 1 file (`ui/assets/skin.css`). Zero Rust · zero component · zero registry change · no schema change.** PROVISIONAL skin (Joe HMR-tunes). Chat authored + drove the CDP verify on the live client 9222 (Rule 5) — a skin edit under the running `tauri dev` session, HMR-applied and measured in place.

### The one sentence

A lock that only collapses an EMPTY shelf is not a frame — neutralising `.shelf[data-empty]` (was `min-height/padding/border → 0`) makes BOTH shelves hold the base `min-height: var(--ctl-h)` + the position hairline whether empty or full, so an empty favourites strip (or pinning into it) never shifts the centre grid.

### What shipped (Joe-locked)

Joe: both shelves fixed height, not collapsible, even when empty — a calmer grid frame. The base `.shelf` rule already gives a fixed frame (`min-height: var(--ctl-h)` + `padding: 0 var(--sp-2)` + a position hairline; top `border-bottom`, bottom `border-top`), and BOTH shelves already share it — so the “button shelf height” Joe asked to measure is not a special number, it is `--ctl-h` + hairline, and “match the button shelf” reduces to “stop collapsing the empty one.” One block removed; `data-empty` stays EMITTED (a skin hook, no JS reader — grepped) but no longer zeroes the box.

### Measured — live 9222, Chat drove (Rule 5)

| leg | before | after (HMR) |
|---|---|---|
| bottom (button) shelf | 28.8px, `data-empty:false` | 28.8px — unchanged |
| top (favourites) shelf | **0px** (collapsed, `data-empty:true`) | **28px**, `data-empty:true` (attr retained), `min-height:28px`, hairline 0.8px |

The collapse is gone: the top strip went 0 → 28px, so the grid no longer reflows between an empty and a populated favourites shelf.

### The sub-pixel finding (accepted, recorded so it is not “fixed” later)

Top 28 vs bottom 28.8 = **0.8px** (one device pixel). Cause: `.shelf` is `box-sizing:border-box` with `min-height: var(--ctl-h)` — the POPULATED bottom shelf's faces push its border-box 0.8px past the 28 min, while the EMPTY top sits exactly at min-height. It is not a collapse (both are fixed); it is a content-vs-min artifact of the border-box. Joe accepted it (the optical bar, N-128 — optically correct is satisfied); it is sub-pixel and DPR-safe (a pinned `28.8px` would freeze this display's scaling). The exact-equality alternative — `.shelf { height: var(--ctl-h) }` → both 28px, faces clamp harmlessly — is filed-not-taken. → N-130.

### Ripples

The node inherits it free at M-RP7.7 (shared skin → both apps get the identical fixed frame). No registry / catalogue / schema impact. No new D. Deviations: none.

### Records

The skin.css shelf docblock is corrected in place (the “`data-empty` collapses the strip … the 6.1j pre-pinning look” note → the fixed-frame note). ui-notes N-130 · ROADMAP · CLAUDE.md PLAY · task file COMPLETED. Feat: skin-only, 1 file — **not pushed (Joe pushes).**

---

## Entry J-529 — M-RP7.6 CLOSED: the grid lock — the lock that only hides buttons is not a lock

**Feat `303faa4` [Clair, code-only, 9 files, +139/−42, pushed]. Zero Rust · zero sampler · no schema change** (`version` stays 3 — `locked` is a `session` key, not a `Layout` field). `cargo test` **1517/0/62 IDENTICAL by construction** (no `.rs` in the diff). `npm test` **77**, `vite build` **170**. Clair built; Chat re-drove every §7 leg on the live client 9222 (Rule 5) — Clair's numbers reproduced exactly.

### The one sentence

The lock that only hides buttons is not a lock — the guard is `if (locked) return` at the top of `handleFold`/`handleResize`/`handleMove` (the layer a command bridge or a stray call cannot walk around), and the element-absent grip / fold-buttons / dead seams are the honesty layer so no live-looking dead control ships. Both halves shipped; the refusal is load-bearing.

### What shipped (D1–D5, Joe-locked, no design authority taken)

**First stateful shelf face (D2):** `shelf-face` gained `pressed?` → `aria-pressed={pressed || undefined}`; getter `{command, hasIcon, disabled, active, pressed}` — a toggle axis distinct from roving `active` (G2, proven: pressed face, `active:false` unchanged). `shelf` `ShelfItemDef` gained `pressed?`, threaded. **`locked` threaded shell → node → tile (D1):** grip + `.region-title-buttons` go `{#if !locked}` element-absent; the seam gate composed to `live && !locked`. **Persistence (D3, N-107):** `uistate.setSessionLocked` mirrors `setSessionLayout`; `persist()` now does a **two-key session merge** (`layout` | `locked`) — geometry (Rust's) and layout both preserved when `locked` writes. **Wiring (D4):** `app_client` `locked` `$state` seeded `session()?.locked ?? false` after hydrate; single `layout.lock` toggle (one bool, one command — D-067); `SHELF_BOTTOM` → `$derived` so the latch tracks `locked`; the three handlers early-return. **Glyph (D5):** PROVISIONAL colour-free `lock` in `icons.ts` (D-108/D-110); shape → M-RP-SKIN. Skin `.shelf-face[aria-pressed="true"]` accent-neutral latch, no glyph swap.

### The load-bearing proof (V1) — re-driven both ways

Locked, driving `__XGEN_LAYOUT__.fold`+`move`+`fold` through the DEV bridge — **which bypasses the chrome and calls the handler directly** — the descriptor was **byte-identical** (sizes `[63,111,826,200]` intact, no `collapsed` introduced). The control is the leg: unlocked, the *same* bridge call **mutated** (`bridgeMutates:true`, introduced `collapsed`), then unfold restored it exactly. A refusal you cannot distinguish from a stuck bridge is not proven — so the mutating control had to run (N-099 family). It did; the refusal is real.

### Measured — every leg (live 9222, Chat re-drove)

| leg | result |
|---|---|
| V2 suppression | locked ⇒ grips **0** / fold-groups **0** / live-seams **0** (7 dead dividers stay). unlocked ⇒ grips **8** / live-seams **7**. |
| V3 bands inert | locked ⇒ bands 0, `dragging:null` (no grip ⇒ no drag can start). |
| V4 face toggle | click ⇒ `pressed:true`, `aria-pressed="true"`, roving `active:false` unchanged. |
| V5 persistence/N-107 | disk `session` = geometry `{402,178,1644,1165}` (Rust's) + layout v3 + `locked` **coexisting** — the frontend write never ate geometry. |
| V6 migrate-tolerance | witnessed at open (N-091) — store had **no `locked` key** → hydrates `false`, grips 8, no crash. |
| V7 content untouched | locked ⇒ count 69, 8 tile bodies, self-panel + inspector registered. |
| V8 registry | quiescent **67 → 69** (+2 = `shelf-face#app-shelf-bottom__3` + its `__icon`), `count===unique===69`; four conditions read (empty store, null selection, nothing folded, zero saved). |
| V9 gates | no `.rs` · `cargo test` **1517/0/62 IDENTICAL** · `npm test` **77** · `vite build` **170**. |

### Deviations (Rule 6 — flagged, not absorbed)

**The seam composition** — §4 literally specified only `onpointerdown={live && !locked ? …}`. Clair composed one `{@const draggable = live && !locked}` driving `data-live` AND all five pointer listeners, because the skin paints the resize cursor + `::before` hit-zone + z-index off `[data-live="true"]` — gating only `onpointerdown` would ship a resize-cursored seam that does nothing, exactly the live-looking dead control D1 forbids. §8 anticipated this ("if the seam live predicate cannot compose `!locked` cleanly, that is a finding"); it composes cleanly and V2 proves it (7 dead dividers, 0 live). A faithful D1+D4 realization, not a divergence. **`UiStateBag.locked?: boolean`** added to the type so the session key is first-class (§6 scope). Minor.

### Corrections to the record

**Module count 169 → 170** — J-528's 169 was stale; the clean tree at HEAD builds 170, so this change is module-neutral. The running figure is 170. **N-108 baseline 67 → 69** (ui-components) — the +2 is the grid-lock face + its icon child; the four-axis citation rule (store · selection · saved · fold) stands. **No new D, no new N** — D-116/N-107 extension exactly as the runbook predicted; the seam composition is a D1 realization, not a discovery (D-065 — no N invented).

### State

M-RP7.6 DONE. **The dock-engine arc's mechanics are complete** — tile frame, fold, splitter resize, drag-to-dock, exact preview, session persistence, and now the lock. **Next-active = M-RP7.7 — node app inherits the frame + grid** (lands after the arc so it inherits a working grid rather than building the frame twice). Still filed: `M-RP-SKIN` (every PROVISIONAL incl. the lock/unlock glyph swap) · `M-RP-PLATE` · `M-RP-ROVING` · `M-RP-MOVE-KBD` · `M-RP-RESTART` · `temperature-indicator` ⏸️. Per D-065 + D-074 + N-107 + Rule 5 + Rule 6.

---

## Entry J-528 — M-RP7.5 CLOSED: the session layout feeder — the grid persists, and N-107 holds one level deeper

**5 files, ZERO Rust** (`git diff --stat`: `uistate.svelte.ts` · `app_client.svelte` · `menu.svelte` · `menu-bar.svelte` · `skin.css`) → `cargo test` **1517/0/62 IDENTICAL by construction**. `npm test` 77, `vite build` 169. Registry **67** quiescent. Clair built; Chat re-drove every leg on the live client 9222 (Rule 5).

### What shipped

The grid finally **persists across a relaunch**. `loadLayout()` already read `session.layout` and migrated it — the load half was shipped; this milestone added the **write half**. `uiStateStore.setSessionLayout(layout)` feeds the live arrangement (fold/resize/move) on a 400ms debounce; `persist()` became the one **N-107-correct write path** — it reads the on-disk `session` FRESH, spreads it, and overrides `layout` **only**, so the frontend write never eats the `geometry` Rust writes. Wired from `handleFold`/`handleResize`/`handleMove`. Companion verbs shipped (in-tree, exposed): **File ▸ Restart** (`app.restart` → `relaunch()`), a **separator**, then **Exit**; and a standalone **`layout.revert`** command (live handler, no UI home yet). The menu gained a **non-registering separator** primitive (`{ separator: true }` → `role=separator`, no id — the roving machine steps over it).

### The N-107 live proof (V1)

The one thing static review can't settle: a frontend `session.layout` write must leave `session.geometry` untouched. Committed a fold → disk `session.layout` gained `collapsed` while `session.geometry` stayed **byte-identical** (x402 y178 w1612 h1087). Neither writer ate the other, at rest and across a write. V2: the fold (and a prior inner-resize `[1714,2286]`) survived `location.reload()`. V3: a valid layout carrying a retired `ghost-retired-xyz` loaded — resolve **dropped the ghost** (registry 66 = 67−1), the shell rendered (no blank), the descriptor kept the id (W-13). Separator: roving steps Restart→Exit→back over it. *(V3 first pass tripped my own BOM probe artifact — `Set-Content -Encoding UTF8` → `EF BB BF` → Rust `get_ui_state` choked → N-095 DEFAULT fallback; re-ran BOM-free. N-124 family: verify the input the probe writes. Self-caught by looking.)*

### N-116 — the record said "never wired"; the code said otherwise (Clair caught it, Rule 6, sixth consecutive milestone)

The runbook §2/§5, ROADMAP L754, dock-engine §10/§13 and J-520 all claimed `tauri-plugin-process` was "declared but never wired" — so Leg C would move `cargo test`. **False.** The plugin was `.plugin()`-wired at `desktop.rs:543` and `process:default` granted at `capabilities/default.json:8`, both since M1 (`c23c06a`). Restart added **zero Rust**; the whole milestone is frontend-only. My grounding failed — I propagated the ROADMAP's claim into the runbook without grepping `desktop.rs`. All four records corrected in this commit.

### Honest partials + filing

**Restart** wiring is correct but **dies on the 2nd invocation under `tauri dev`** — a dev-lifecycle artifact (spawn-self+exit tears down the dev supervisor that owns Vite), not a wiring bug; unproven in a standalone bundle. Joe's ground: Restart's primary user is the agent during verification, exactly where the dev-death bites, so it isn't worth a detour — the grid matters more. Filed **countdown: a standalone-bundle hand-test, Joe discharges** (the bundle has no CDP). **Revert** is a live handler but as-built a visible no-op (continuous feeder ⇒ disk ≈ live ⇒ reload changes nothing); recorded useful semantics = **undo-to-launch** (snapshot the boot layout, restore on Revert — respects the saved workspace, unlike reset-to-default). Not button-driven (no UI home). Both refiled to **`M-RP-RESTART`**. **Reset-to-default** stays filed deep in settings (Joe: "not everyday").

### State

M-RP7.5 DONE. The `session` per-key merge is shipped → **M-RP7.6 (the grid lock) is unblocked** (its `locked` flag lives in `session`). Per D-065 + D-074 + N-107 + N-116 + Rule 5 + Rule 6.

---

## Entry J-527 — M-RP7.4b CLOSED: the exact preview — rehearse the drop, and the reflow lie becomes optical truth

**`ui/core/lib/components/layout/region-shell.svelte` — ONE file. Zero Rust (proven by `git diff --stat` + `cargo test` 1517/0/62 IDENTICAL), zero sampler, no `mutate.ts`/`skin.css` change, no schema change (`version` stays 3). `npm test` 77, `vite build` 169.**

### What shipped

7.4a's preview drew **half of the target as it is NOW** — but `move` does remove → collapse-degenerate → insert, and the *remove* reflows the grid *before* the region lands (N-127), so the preview was up to 120px off. 7.4b computes the preview from a **rehearsal of the move**:

1. **Dry-run** `move(layout, source, target, edge)` on the live descriptor — `move` is pure and total (M-RP7.3), so this has **zero side-effect** (V6: the live layout is byte-identical *while the preview is showing*).
2. **Resolve** the hypothetical tree the same way the render does.
3. **Find the moved leaf's path** and **proportion its weight down that path**, mirroring the renderer's own rules: `flex: {weight} 1 0` → `weight / Σweights × axis`, with a folded-along leaf / shrink-wrapped split taking a fixed `--region-stripe` strip out of the weight pool (`carriesMainAxisWeight`, **reused from resolve.ts, not re-derived** — a concept in the code is not reinvented).

### 🔑 The reflow is fixed — which was the whole point

| case | axis | 7.4a | **7.4b** |
|---|---|---|---|
| `spaces`→`stream` right | left | **60px** | **2px** |
| | width | 38px | **4px** |

The moved region now previews in the **right place**. The strip-exclusion works too: V3 (drop into a column holding a folded strip) landed **top 1px** — without the exclusion the strip's ~22px would have shifted everything.

### ⚠️ The measured finding (N-128) — the runbook's "≤2px floor" was the wrong measurement

The runbook's §1.3 "within 2px" measured **one split level's four column widths**; the preview instead walks a **multi-level path to a TILE**. Weight-proportional math is exact for **weights** and **structurally blind to the fixed gaps** (each tile's 4px margin per side + the inter-tile gap at every level), which accumulate down the path:

| case | reflow axis | gap axis |
|---|---|---|
| V1 (3-level wrap) | left 2 / width 4 | top 7 / **height 14** |
| V2 (2-level sibling) | left 3 / width 10 | top 6 / height 7 |
| V3 (folded, strip-excluded) | left 3 / **top 1** | width 10 / height 8 |

So the honest floor is **reflow-exact, ~2px per split level of accumulated gap** — up to ~14px on a deep wrap, not the flat 2px predicted (Rule 6: the runbook measured the wrong quantity). Chasing the gaps precisely means modeling the margin/seam interaction per child type — the §7 second-model trap; §5 (mount the hypo tree offscreen, measure the real rect) is the truly-exact path and stays filed, unbuilt.

### Resolution — the bar is optical

I reported the measured floor rather than claim the ≤2px the runbook demanded (Rule 3/5/6). Joe: *"we don't need super exact computations and result numbers. if the highlighted rectangles are in optically correct positions and size, i am satisfied."* The proportional preview meets that bar — the reflow (the thing that was visibly wrong at 60px) is exact, and the residual is ~1–14px = **1–4% of the rect**, invisible in use. Shipped; the floor is recorded as-is, not "pixel-perfect."

### MEASURED — every leg re-driven, ground-truth method (Rule 5)

Capture the preview mid-drag → commit the real move → read the moved tile's **fresh** rect (`matches:1` — after a move the tile is a new DOM node, N-125, so a held reference lies) → restore via the saved descriptor. **V1** reflow-heavy: 2px on the reflow axis (was 60px). **V2** sibling: 3–10px. **V3** folded target: strip-exclusion works, top 1px. **V4** wrap (V1) + sibling (V2) both optical. **V5** no-op edge → `previews:0`. **V6** dry-run purity: layout byte-identical while the preview shows. **V7** `npm test` 77 · `vite build` 169 · `cargo test` **1517/0/62 IDENTICAL**. **V8** clean quiescent 67, saved-descriptor probe removed.

**Records:** `region-shell.svelte` (the only code) · dock-engine Phase-0 **v2.3** (§11 row 4b, §13 `M-RP-PREVIEW-EXACT` ⬛ superseded) · `ui/docs/xgen-ui-notes.md` **v0.92** (N-128 + the N-127 correction: conditional 1–120px, not flat 40–100) · `tasks/M_RP7_4B_EXACT_PREVIEW.md` COMPLETED · CLAUDE.md PLAY · ROADMAP. **No new D.**

**Next-active: M-RP7.5 — the session layout feeder** (writes `session.layout`; N-107's per-key merge inside `session`; `M-RP-RESTART` lands with it).

---

## Entry J-526 — M-RP7.4a CLOSED: the division preview — the orange half shows the target, and a measured finding about what it can and cannot promise

**`ui/core/lib/components/layout/region-shell.svelte` + `ui/assets/skin.css` — two files. Zero Rust (proven by `git diff --stat` + `cargo test` 1517/0/62 IDENTICAL), zero sampler, no schema change (`version` stays 3). Detection byte-identical — `npm test` 77, `vite build` 169.**

### What shipped

Joe's note on the shipped M-RP7.4 drag: the orange drop preview should show **the real region about to be created** — half the target, where it will land — not the fixed `f=0.3` edge slab. M-RP7.4 conflated two things in one `bands` array: the **hit targets** (the strips that DECIDE the edge) and **what the user sees**. This milestone splits them.

- The `bands` derive + every `data-edge`/`data-active`/`data-noop`/hit rect stay **byte-identical** — detection (D2) is untouched.
- A new `$derived previewRect` = the drop-half of the hovered tile (`move`'s own 50/50: `top`/`bottom` split the height, `left`/`right` split the width, flush to the target's real edges).
- One new `.region-drop-preview` element, `pointer-events:none` (the D3 correctness lock — it is PAINT, never a hit target), render guard just `drag.edge` (so a no-op edge and a hole suppress it free — D4/D3 inherited, no second check).
- The band's `data-active` highlight (the old slab) now paints nothing — its visible role moved to the preview, so there is one honest indicator, not two.

### 🔑 The measured finding (N-127) — a preview drawn from the pre-move DOM is directional, not exact

V1 confirmed the preview is the **exact** half of the hovered target (TOP → `{368,116,842,331}` = stream's top half to the pixel; RIGHT → `{790,116,421,662}` = its right half). **But V2/V3 — "does the preview match the rendered rect after the drop?" — the runbook's stated pass/fail — do NOT pass:**

| drop | preview (MID) | `spaces` rendered (after) | delta |
|---|---|---|---|
| stream TOP (sibling) | `{368,116,842,331}` | `{266,116,**922**,329}` | width **+80**, left **−102** |
| stream RIGHT (wrap) | `{790,116,421,662}` | `{729,117,**459**,659}` | width **+38**, left **−61** |

**Root cause, grounded on the real client:** `move` **removes the source from its old slot, and that re-flows the whole grid.** `spaces` was the leftmost column (weight 1 of `[1,2,7,2]`); dropping it elsewhere makes the root row `[2,7,2]` over three columns, so the target's column **widens ~80px** and the source lands in that widened half. The preview is drawn from the target's pre-move rect; reality is the post-move (reflowed) rect. **They cannot agree whenever source and target are in different branches — i.e. almost every move.** The rule (N-127): *"draw what the algebra commits" is exact at the descriptor level, but the rendered geometry also depends on the reflow the same mutation triggers elsewhere — so a pre-move preview is honest only for a LOCAL mutation; `move` is non-local.*

**I stopped and reported this rather than fabricate a V2/V3 pass or silently redesign** (Rule 3/5/6). Shown the measured deltas, **Joe accepted the directional preview** — *"i think it works as i want."* Recorded plainly so nobody later "fixes" the ~40–100px as a bug: it is inherent to a pre-move preview under reflow. The exact post-drop rect (apply `move` to a copy, render offscreen, measure) is filed as **`M-RP-PREVIEW-EXACT`** (§13), out of the two-file scope.

### MEASURED — every leg re-driven with the trusted-pointer harness (Rule 5)

**V1** preview = exact half (above). **V2/V3** the finding (above) — the mechanism is correct, the pre-move geometry is what it is. **V4** `elementFromPoint` over the preview centre returns `region-drop-band` (`isPreview:false`) — the preview never captures the pointer (D3). **V5** dragging `rooms` over `self`'s TOP (a no-op edge) → `previews:0`, `activeBands:0` (D4 inherited). **V6** `npm test` 77 · `vite build` 169 · `cargo test` **1517/0/62 IDENTICAL**. **V7** clean quiescent 67, no residue.

### Honest close

The runbook's V2/V3 "sub-pixel match to reality" is not what shipped — it is unachievable under reflow, and the acceptance is the directional preview. V1 (exact half), V4 (no capture), V5 (no preview on no-op/hole) all held; the appearance is PROVISIONAL (M-RP-SKIN). No new D.

**Records:** `region-shell.svelte` · `skin.css` · dock-engine Phase-0 **v2.2** (§11 row 4a, §13 `M-RP-PREVIEW-EXACT`) · `ui/docs/xgen-ui-notes.md` **v0.91** (N-127) · `tasks/M_RP7_4A_DIVISION_PREVIEW.md` COMPLETED · CLAUDE.md PLAY · ROADMAP.

**Next-active: M-RP7.5 — the session layout feeder** (writes `session.layout`; N-107's per-key merge inside `session` becomes load-bearing; `M-RP-RESTART` lands with it).

---

## Entry J-525 — M-RP7.4 CLOSED: drag to dock — the dead grip gets a pointer, and a user rearranges the grid by hand

**`ui/core/lib/components/layout/` + `ui/client/src/app_client.svelte` + `ui/assets/skin.css`. TypeScript + Svelte + PROVISIONAL skin only — zero Rust, zero sampler, no schema change (`version` stays 3). Proven by `git diff --stat` (7 files, no `.rs`/`Cargo.*`) AND `cargo test` 1517/0/62 IDENTICAL.**

### What shipped

The loudest milestone in the arc: the move grip that M-RP7.1 shipped **painted and dead** (`.region-tile-move`, `aria-hidden`, no handler — Joe: *"only with this grip the region can be moved"*) is now ACTIVATED, and a user can rearrange the grid by hand for the first time. **No new tree algebra** — `move` was finished, pure, total and live at M-RP7.3; 7.4 wires trusted pointer gestures onto it.

- **The grip** drops `aria-hidden`, gains `role="button"` + `tabindex="0"` + `onpointerdown`. Keyboard-driven move is its own protocol → filed `M-RP-MOVE-KBD` (D5).
- **`onMoveStart` + `draggingId` threaded** shell → node → tile (the `onFold` shape); `onMove` (→ `handleMove`) is the shell↔app completion callback.
- **ONE grid-level overlay (D1)** — the drag ghost + four `data-edge` bands + an inert centre, mounted last at `z-index: 4000` above every tile. This designs the whole N-119 paint-order class out in one stroke; per-tile overlays would re-fight the seam's z-index battle on every tile.
- **`isMoveNoop` extracted** from `move` — the single predicate both the affordance and the action read.

### 🔒 The five locks held, and D2 is the spine

- **D2 — the band is chosen by HIT-TEST, never geometry.** `elementFromPoint(x,y)` → read `data-edge` off the band. The band rects only POSITION the hit targets; the pointer's edge is read off the thing that paints. **V1 proved it:** mid-drag (button held over `stream`), `elementFromPoint` swept across the four band centres read **`top:stream / bottom:stream / left:stream / right:stream / center:-`** — every centre resolving to the edge the picture shows, no coordinate resolving to another.
- **D3 — a hole offers no band because a band attaches only to a rendered tile.** Not a suppression check — an impossibility. **V4:** with a real hole (fold `spaces` across the root row → the split under-fills), a drag over the hole showed `elementFromPoint → region-split`, **`bandsDrawn: 0`**, release = no-op.
- **D4 — a no-op band does not light.** The trap was computing "is this a no-op?" a second way in the overlay — a parallel model of `move`'s decision that can drift (the N-124 shape). The fix is structural: `move` and the overlay both call **`isMoveNoop`**, so a highlighted band and a committed move cannot disagree by construction (→ N-126). **V5:** dragging `rooms` (already above `self`) over `self` → TOP band `data-noop=true active=no`, BOTTOM band `data-noop=no`.
- **D1** designed out N-119; **D5** filed the keyboard protocol.

### MEASURED — every leg re-driven with the TRUSTED-POINTER harness (Rule 5; the DEV `move()` handle is NOT the proof here)

**Baseline** 67, quiescent, empty store, no selection. **V1** the D2 sweep (above). **V2** trusted drag `spaces` grip → `stream`'s right band: `spaces` left the far-left column (`left ~4`) and appeared right of `stream` (`left 722`, `spacesRightOfStream:true`), registry 67. **V3** a sequence of trusted drags (sibling + relocate): registry **67 / unique 67 / leaf 8 / stampMismatches 0** after each — the N-125 tripwire clean through real gestures. **V4** the hole (above). **V5** the no-op band (above). **V6** teardown total — Esc mid-drag (`dragging:null` at MID), release outside the grid, release on the source's own centre: each left no `data-dragging`, no overlay, registry unchanged. **V7** a sub-threshold press is a click, not a move (no band phase, no `move` call). **V8** `npm test` **77** · `vite build` **169** · `cargo test` **1517/0/62 IDENTICAL**. **V9** clean quiescent 67, no inline residue.

### Deviations (Rule 6) — one, flagged not absorbed

The runbook DoD said *"onMove threaded shell → node → tile."* Grounded, the TILE only needs a START trigger: under D1 the capture loop, the overlay and the `onMove` completion all live at the grid level (`region-shell`), so the tile gets `onMoveStart` and `onMove` is the shell↔app callback. The runbook's wording was off; the code is right — the fourth milestone running where Rule 6 fired on the runbook, and the fourth where the code was the correct party.

**Two honest notes.** (1) The Esc test's synthetic `KeyboardEvent` is untrusted, but `onWinKey` reads `e.key` and does not depend on a UA default, so it validly exercises my teardown wiring (a real trusted Escape takes the same path). (2) At MID the `.region-drag-overlay` DOM read still showed `true` one turn after the state went null — a pre-flush read; the state (`dragging:null`) is authoritative and the overlay was gone after settle (the N-099 read-after-settle family).

**Records:** the 7 code files · dock-engine Phase-0 **v2.1** (§11 row 4 CLOSED, §13 `M-RP-MOVE-KBD`) · `ui/docs/xgen-ui-notes.md` **v0.90** (N-126) · `tasks/M_RP7_4_DRAG_TO_DOCK.md` COMPLETED · CLAUDE.md PLAY · ROADMAP **v4.96**. **No new D.**

**Next-active: M-RP7.5 — the session layout feeder.** The grid finally WRITES `session.layout` (§12) — and then both writers touch `session`, so N-107's per-key merge inside `session` becomes load-bearing. Then M-RP7.6 (the grid lock) · M-RP7.7 (node app inherits the frame + grid).

---

## Entry J-524 — M-RP7.3 CLOSED: the mutation algebra — N-120 discharged, `move` built, and the move exposed N-120's twin in the renderer

**`ui/core/lib/components/layout/` + `ui/client/src/app_client.svelte`. TypeScript + Svelte only — zero Rust, zero sampler, zero `skin.css`, no schema change (`version` stays 3). Proven by `git diff --stat` (7 files, all `.ts`/`.svelte`) AND by `cargo test` 1517/0/62 IDENTICAL.**

### What shipped

`mutate.ts` is now the **complete** pure write algebra for the dock:

- **`resizeSplit(layout, path, aIdx, bIdx, fraction)`** — the N-120 fix. The pair is **two explicit DESCRIPTOR indices** that need not be adjacent; weight moves between exactly those two, the between-entries (ghost weight included) untouched.
- **`foldLeaf(layout, regionId, collapsed)`** — migrated **verbatim** out of the shell (it was already pure and identity-addressed — a move, not a rewrite). Unfold still DELETES the key.
- **`move(layout, sourceLeafId, targetLeafId, edge)`** — remove → collapse-degenerate (cascading) → insert (sibling if the target's parent already runs on the drop axis; else WRAP). **No re-normalise pass** — §6 step 4 was overstated work and is deleted from the record.

`resolve.ts` gained **`srcIndex` on every `ResolvedNode`** (its descriptor index in its parent's children; root `-1`); `region-node` threads `path` from `srcIndex` and reports a seam's pair as descriptor indices. The 12 M-RP7.2 `resizeSplit` cases were **migrated** (not deleted) to the two-index signature; `mutate.test.ts` is now **26 cases** (`npm test` **75**).

### 🔑 N-120 discharged — reached, not argued (V1)

J-519 shipped `resizeSplit` addressing a split by a `path` counted over the **resolved** tree, while `resolve.ts` **drops** — so the instant anything dropped, the resize hit the wrong pair (J-519: a ghost between `spaces` and `rooms` made a rightward drag **HALVE** `spaces`). The fix: a resolved child carries its source index; a resize addresses by it; a move addresses by leaf **identity**.

**V1, real client 9222:** the poisoned layout rebuilt (one unknown `widgetId`, 3 descriptor children → 2 tiles, 1 seam). Dragging the seam RIGHT to enlarge `spaces`: MID-drag (button down) the descriptor read `[1,1,1]`; AFTER release **`[1356, 1000, 644]`** — **`spaces` GREW `1000→1356`, the ghost byte-identical at `1000`, pair total `2000` invariant.** The exact inverse of J-519. *The gesture and the result finally agree, and the live preview stayed in resolved space while only the write crossed to descriptor space.*

### 🔑 The finding was bigger than the runbook (N-125) — `move` exposed N-120's twin in the RENDERER

The first `move` on the real client dropped the registry **67 → 65**, and the painted DOM showed tiles stamped `data-debug-id="region-tile#region-rooms"` **titled "R4 · Room header"** — the content correct, the identity scrambled. Root cause, grounded (`git show HEAD` = M-RP7.2, not mine): `region-node`'s `{#each node.children as child, i (i)}` is **index-keyed**, and `move` is the FIRST mutation to change a split's **child count and order** — fold and resize never did. So index keying reused a tile instance across regionIds, and `use:envelope` stamps `data-debug-id` on MOUNT without re-keying.

**Same family as N-120: a latent index-key defect, unreachable until the first mutation that restructures the tree.** It fails DoD V2 (registry 67 through any move), so the fix was required by the DoD, not scope creep. Fixed with **stable node-identity keys** (`(nodeKey(child))` — a leaf by its widgetId, a split by its subtree's leaf ids). After: every move holds registry **67, unique 67**, every stamp matches its title. *"Unreachable today" has now been wrong six times here (N-091 · N-097 · N-099 · N-109 · N-116 · N-120).*

### MEASURED — Chat re-drove every leg on client 9222 (Rule 5)

**Baseline** 67, quiescent, empty store, no selection, version 3. **V1** N-120 `[1356,1000,644]` (ghost byte-identical). **V2** registry 67/unique 67/leaf 8/dropped 0 through sibling, wrap, collapse, and relocate. **V3** sibling insert: root `[2,7,2]`, col `[rooms,spaces,self]` `[3,3,2]`. **V4** wrap: `row[1,1]` `[rooms,spaces]`, grandparent `[3,1]` untouched. **V5** collapse-degenerate: the rooms/self col vanished, `rooms` is a bare leaf holding the weight-2 slot (`[1,2,7,2]`), members col `[1,1,2]`. **V6** bare-leaf root renders a single "R5 · Message stream" tile (registry 52) — **not a blank centre** (N-095 holds). **V7** folded `spaces` moved into a col split: `collapsed:'width'` survived, fold-mode flipped `along→across` (correct — leaves a hole, §4.1). **V8** unfold DELETES the key. **V9** spaces relocated `(4,30) 119×855 → (1171,245) 253×212` — a region visibly moving. **V10** `npm test` **75** · `vite build` **169** · `cargo test` **1517/0/62 IDENTICAL** (56 terminators, final `test result:` line present — the N-117 truncation trap avoided; a first detached run read `1195/0/60`, the exact "plausible and WRONG" artifact the harness warns of). **V11** clean quiescent 67, no inline residue.

### Deviations (Rule 6) — one in-scope addition, flagged not absorbed

**N-125 is beyond the runbook's four legs.** The runbook did not foresee that `move` would expose the renderer's index keying; but its own DoD (V2: registry 67 through any move) demands the fix, so it is in scope by the DoD, not scope creep. Flagged here and in the task file. Everything else matched the runbook, and the runbook's own corrections held: `resolve.ts` DOES drop (N-120 real), the re-normalise step DID NOT exist (§3.4 correct), the two-index signature IS needed (non-adjacent pair proven in V1 and in a dedicated vitest case leaving the between-entry byte-identical).

**Records:** `resolve.ts` · `region-node.svelte` · `region-shell.svelte` · `mutate.ts` · `mutate.test.ts` · `resolve.test.ts` · `app_client.svelte` (the 7 code files) · dock-engine Phase-0 **v2.0** (§6 re-normalise deleted; §6.1 N-120 discharged + N-125; §11 row 3 CLOSED) · `ui/docs/xgen-ui-notes.md` **v0.89** (N-120 discharge · N-125) · `tasks/M_RP7_3_MUTATION_ALGEBRA.md` COMPLETED · CLAUDE.md PLAY · ROADMAP **v4.95**. **No new D.**

**Next-active: M-RP7.4 — drag to dock.** The algebra gets a pointer: grip + four edge bands per tile, inert centre. A hole is not a drop target (§4.5). ⚠️ N-119's paint-order lesson is inherited — sweep `elementFromPoint` across every drop-band edge.

---

## Entry J-523 — M-RP7.2b CLOSED: the region owns its gap — and the boundary it was locked to fix turned out to be invisible

**Skin only (`ui/assets/skin.css`). No component, no descriptor, no vitest, no Rust. Joe-locked → built → A/B-measured → the justification died → Joe shipped it anyway on the right reason.**

### What shipped

`--region-gap: 4px` is now the **only** spacing token. `--region-pad` and `--region-seam` are **deleted — not renamed, gone.**

- every **TILE** carries `margin: G`
- every **SPLIT** carries **nothing** (and, since J-521, must never paint again — now load-bearing: a nested split's box **overlaps its neighbour's margin**)
- every **SEAM** is a **ZERO-WIDTH** flex child with `−G/2` each side, cancelling the double — **its box lands dead centre of the gap**, so the drag handle survives (a `gap` never was hit-testable; that is why M-RP7.2 had to build a seam element at all)

**`--region-seam-hit` re-derived: `max(4px, calc(var(--region-gap) / 2))`.** N-122's premise is retired — it compensated for a seam that **was** `--region-gap` wide. The seam is now **always** zero-width, so the `::before` **is** the entire grab target. Two floors, both deliberate: **≥ 8px** (a finger, at any gap) and **≥ the gap the user can see**.

### 🔑 THE MILESTONE'S PREMISE WAS A GHOST, AND IT HAD A 🔒 ON IT IN THREE RECORDS

For three journal entries the arc carried: ***"the region-owned gap model fixes the one boundary still wrong — tile↔hole is 0."*** It was in the skin comment, in Phase-0 §4.5.2's table, in the PLAY block, and in **this milestone's own runbook — which Chat wrote before measuring it.**

**The A/B took one eval:** inject the OLD geometry as a `<style>` (tile margin 0 · shell padding 4 · seam 4px), fold a tile **across** to build a real hole, measure, remove the style.

| | NEW | **OLD (injected)** |
|---|---|---|
| folded tile insets | 4 / 4 | **4 / 4** |
| distance to the hole | 774.8px | **774.8px** |
| adjacent-gap census | all **4** | **all 4** |
| perimeter | **4** | **4** |

> ### ⚠️ **IT IS NOT OBSERVABLE, AND THE REASON IS ONE LINE: SINCE J-521, A HOLE AND A GAP ARE THE SAME SURFACE.** Both are `.region-shell`'s backdrop. A tile "butting onto a hole" is **indistinguishable** from a tile with a gap — ***there is nothing on the far side of that gap to be separated from.*** The claim was never wrong about the pixels; it was **about a difference that cannot be seen**, and nobody had tried to look. **The model has ZERO VISUAL DELTA.**

**🔑 FOURTH TIME, SAME SHAPE.** N-118: my *reasoning* can be internally consistent and wrong. J-521: my *architecture* can be. N-116: the *record* can be. **N-124: the *justification* can be** — and a 🔒 icon changed nothing except how confidently it got repeated.

### 🔒 JOE SHIPPED IT ANYWAY, ON THE HONEST REASON — AND THAT REASON WAS AVAILABLE FROM THE START

**It deletes a MECHANISM, not a token.** The perimeter was the *shell's padding*; the inter-tile gap was the *seam element's thickness* — **two mechanisms, two tokens, which had already drifted apart once** (J-517: 1px and 6px; **Joe found it by looking at the screen**). J-520 forced them to *derive* from one number; **they can now never drift, because they no longer exist.** And it is Joe's own model of what a region **is**: *"those gaps are part of each region."* **A gap that lives on the REGION travels with it when `move` relocates it (M-RP7.4); a gap that lives on the GRID exists only where the grid puts a seam.**

### ⚠️ TWO RECORDS WERE MORE PRECISE THAN THEIR MEASUREMENT (N-124a)

**(a) `[1,2,7,2]` WAS NEVER BIT-EXACT.** The arc recorded it as **"EXACT"** at four separate window widths. At full float precision it is **`[1, 2.000112, 7.000224, 2.000112]`** — **and the OLD geometry returns `2.000114`.** A pre-existing Chromium sub-pixel artefact (~0.004px) in **both** models — **which is precisely what proves the margin adds no bias.** (N-121's border broke it by **1.5%**; this is **0.006%**, and it is not new.) *The runbook said "if V1 is not EXACT the model does not ship" — and V1 was not exact. The right answer was not to fail the model; it was to notice the control had never been measured either.*

**(b) THE CLAMP DOES NOT STOP AT "EXACTLY 22px".** Measured **22.29px** — integer-weight rounding (L2), not a defect. Under the OLD geometry the same rounding lands at **21.89px — just BELOW the minimum.** Neither is a bug; the new one at least errs on the safe side of the floor.

> ***A number you rounded before you recorded it is a number you can no longer use as a control.***

### MEASURED — Chat re-drove every leg on the real client 9222 (Rule 5)

**V1** ratios `2.000112` new vs `2.000114` old → **no bias** · **V2** perimeter **4/4/4/4** and **14 adjacent tile pairs all exactly 4**, including pairs crossing a split boundary → **it composes at depth** · **V3** **MID-drag (button still down)** descriptor `[1,2,7,2]` untouched while the tiles had painted 216/118; **AFTER** `[194,106,700,200]` — pair total **300 invariant**, untouched siblings **×100** · **V4** grab zone **9px @ gap 0 · 9px @ gap 4 · 20px @ gap 20**, and the seam wins at **x=113–115, inside the neighbour's border box** (**N-119's `z-index` re-proven by sweeping `elementFromPoint`, not by reading the CSS**) · **V5** registry **67** quiescent; **the zero-width seam registers nothing** · **V6** clamp **22.29px**, `data-collapsed`/`data-fold-mode` **null** — *it stops, it does not fold*; `--region-min` still reads **22px** on the seam · **V7** `npm test` **59** · `vite build` **169** · **zero Rust by scope** (`git diff` = `ui/assets/skin.css`) · **V8** probes removed, no inline style, client reloaded quiescent (**N-123 honoured — the cleanup is part of the probe**) · **V9 Joe drove it by hand** (registry **71** — selection active — and a hand-dragged descriptor `[100,220,680,200]`).

### ⚠️ CHAT'S OWN DEFECT, RECORDED NOT ABSORBED (N-124b)

The skin edit was five line-index splices, and **two of them ran top-down.** An earlier splice shifted the file, and the next one **deleted `min-height: 0` from `.region-shell`** instead of the `padding` line it was aiming at — *the rule that keeps a deep leaf from pushing the scrollbar onto the document.* **It was caught by the grep verification, NOT by the diff.** 🔒 **Splice DESCENDING, always — and guard every splice with an assertion on the line it is about to replace.** *Sibling of J-521's `edit_file` near-miss: **a diff that looks applied is not a render.***

**Records:** `ui/assets/skin.css` (the only code) · dock-engine Phase-0 **v1.9** (§4.5.2 rewritten — the ghost named, the `M-RP-PLATE` bundle retracted in §4.5.1 and §13; §11 gains the **M-RP7.2b** row) · `ui/docs/xgen-ui-notes.md` **v0.88** (**N-124 / N-124a / N-124b**) · `tasks/M_RP7_2B_REGION_GAP.md` **COMPLETED** · CLAUDE.md PLAY · ROADMAP. **No new D.**

**Next-active: M-RP7.3 — the mutation algebra (pure).** 🔒 **It opens by fixing N-120** (Phase-0 §6.1) — a required leg, not a filed item: an index into the **resolved** tree is not an address into the **descriptor**, and *a misaddressed `resize` nudges two integers while a misaddressed `move` relocates a panel into the wrong branch.*

---

## Entry J-522 — The region gets its own edge, and the obvious way to draw it would have silently biased the splitter

**Skin only. No component, no Rust, registry 67. Joe's request; Joe's number (`--region-gap: 4px`, which he set himself after testing 0 and 20).**

---

### 1. The ask

Joe: *"Give each region a 1px very fine smooth border. **When gap is 0, there is no sense of regions, all is melted.**"* — he had just tested the extremes (0 and 20px) and found the layout survives both, then settled on **4px**.

**✅ SHIPPED:** `--region-border: 1px` · `--region-edge: var(--s4)` on `.region-tile`. **Both PROVISIONAL; discharger `M-RP-SKIN`.**

### 2. ⚠️ THE COLOUR COULD NOT BE `--s5` ANY MORE — AND THAT IS J-521's DOING

The old hairlines were `--s5`. **But J-521 made `--s5` THE BACKDROP.** An `--s5` edge would be **invisible at any gap > 0** — the majority case. The edge has to read against **both** its neighbours:

- against the **backdrop** `--s5 #343b47` (whenever gap > 0) → it must be **darker**
- against a **tile body** `--s #16181c` (at gap 0, where two edges meet as one line) → it must be **lighter**

**`--s4 #2a2f38` sits between them and contrasts with each. One colour, both jobs.** ***A token that was correct for a hairline stopped being correct the moment the surface behind it changed.***

---

### 3. 🔑 N-121 — A BORDER WAS THE OBVIOUS ANSWER AND IT BROKE THE ARC'S BASELINE

**A real `border` was tried, and measured before it was believed. The `[1,2,7,2]` ratios came back as `[1, 1.97, 6.90, 1.97]`.**

> ### 🔑 **WHY: with `flex: n 1 0`, flex distributes the FREE space by weight — and then each item's BORDER BOX is `content + 2×border`. A CONSTANT is added to every tile.** Content boxes stay exactly weight-proportional; **border boxes do not.** ***And every rect JS can measure is a border box.***
>
> **⚠️ WORSE THAN COSMETIC: THE SPLITTER COMPUTES ITS DRAG FRACTION FROM THE NEIGHBOURS' `getBoundingClientRect()` WIDTHS.** **A border would have biased the resize arithmetic by `2×border` per tile — systematically, silently, forever.** *The drag would land slightly off where you dropped it and nothing would ever fail.*

**✅ `outline` + `outline-offset: -1px`** — the identical hairline, drawn just inside the edge, with **ZERO layout impact**. **Re-measured: `[1, 2, 7, 2]` EXACT.** `--region-min` still parses to **22**; the folded strip is still **22px** with **zero overflow**; the clamp still stops at **22** without folding.

**⚠️ An inset `box-shadow` was also layout-free and was REJECTED:** it paints **below the children**, so the stripe's own `--s2` background would have covered the top hairline. **An outline paints above them.** *(Flagged: tiles are not focusable today — if one ever needs a focus ring, it must not be an `outline` on this element.)*

---

### 4. 🔑 N-122 — AND JOE'S OWN EXTREME TEST HAD A HOLE IN IT HE COULDN'T SEE

He tried `--region-gap: 0` and reported *"the system withstands it"*. **It renders. But at gap 0 the SEAM ELEMENT IS ZERO PIXELS WIDE** — and `--region-seam-hit` was a **constant 1px**, leaving a **2px** grab target. ***Alive, and ungrabbable by a human.*** *He tested that it didn't break; he didn't test that he could still drag it.*

**✅ THE HIT AREA NOW COMPENSATES FOR THE GAP:** `max(1px, calc((8px - var(--region-gap)) / 2))` → **a CONSTANT grab zone at every setting.**

**MEASURED (`elementFromPoint` swept across the seam at each value):** **gap 4 → 9px · gap 0 → 9px · gap 20 → 22px** — and **the drag COMMITS at gap 0** (MID `[1,2,7,2]` untouched, AFTER `[180,120,700,200]`).

> ***A knob that silently kills a gesture at one end of its range is not a knob.***

**⚠️ `calc()` is safe HERE only because this token is never read by JS.** `getComputedStyle` does **not** resolve `calc()` inside a custom property — it returns the raw token stream, and `parseFloat` gives **`NaN`**. **`--region-min` and `--region-snap` ARE read by JS and must stay plain values** — the exact trap the M-RP7.2 runbook already names, dodged rather than walked into.

---

### 5. ⚠️ N-123 — AND THEN THE INSTRUMENT BROKE JOE'S TOKEN AND HE REPORTED IT AS A BUG IN HIS OWN CSS

**Joe, immediately after: *"something happens, `--region-gap: 4px` doesn't work anymore."*** He had edited the token, HMR had applied it, **and nothing moved.**

**It was not his CSS. It was Chat's probe.** Proving the drag at gap 0 (N-122) required setting the token in **one** CDP call and dragging in the **next** — so the override could not be undone inside a single eval, and was set as an **inline style** on `.region-shell`. ***It was never removed.***

> ### ⚠️ **AND VITE'S HMR HOT-PATCHES CSS WITHOUT A RELOAD — SO THE INLINE OVERRIDE SURVIVED EVERY EDIT JOE MADE.** **Inline beats the stylesheet.** *His 4px was being applied and then silently overridden, indefinitely, by a leftover from a test he never ran — and he went looking for the fault in his own work.*

**🔒 RULES ADDED (`CDP_DEBUG_HARNESS.md`):** a probe that must persist a mutation across calls **owes a cleanup call, and the cleanup is part of the probe** · **any session that touched inline styles ends with `location.reload()`** · **prefer reloading between token variants over inline overrides.**

> ***N-118 said the instrument can lie to me. N-123 says it can lie to JOE — and he has no way to tell that the thing on his screen is not the thing in the file.***

---

### 6. Measured (re-driven, Rule 5)

Ratios **`[1,2,7,2]` exact** · outline **0.8px** `rgb(42,47,56)` at offset **−0.8px** *(1 device pixel at DPR 1.25 — which is what "very fine" wants)* · `--region-min` **22** · folded strip **22px**, overflow **0** · clamp stops at **22**, `data-collapsed` **null** · grab zone **9 / 9 / 22px** at gap **4 / 0 / 20** · drag commits at gap 4 **and** gap 0 · registry **67** · `vite build` **169**.

**Files:** `ui/assets/skin.css` · `ui/docs/xgen-ui-notes.md` **v0.86 → v0.87** (**N-121**, **N-122**, **N-123**) · `tasks/CDP_DEBUG_HARNESS.md` **v1.5 → v1.6** · `CLAUDE.md` · `docs/ROADMAP.md` · `JOURNAL.md`.

**`M-RP7.3 — the mutation algebra (pure)` is still next, and it still opens with N-120.**

---

## Entry J-521 — The perimeter and the seams were two different surfaces; and the dependency Chat claimed yesterday did not exist

**Skin only. No component, no Rust, registry 67. Joe found BOTH by looking at the screen.**

---

### 1. ⚠️ "The not-transparent padding around the whole grid is still there"

J-520 made the gap **uniform at 5px** and Joe still said the perimeter was wrong. **He was right, and the reason was not the width — it was the SURFACE.** Measured, not guessed:

| element | painted |
|---|---|
| `.region-split` (the seams, the holes) | **`rgb(52,59,71)` = `--s5` + the dot raster** |
| `.region-shell` (the perimeter) | **transparent** |
| `.app-center` (behind it) | **transparent** |

> ### 🔑 **SO THE GAP BETWEEN REGIONS SHOWED THE GRID'S OWN BACKDROP, AND THE GAP AROUND THE GRID SHOWED THE ROOM BEHIND THE GRID.**
> **Two different surfaces for what is conceptually one gap.** Joe's words for it were ***"the edge gap is made from outside"*** — **and that is a precise description of the implementation, arrived at from the picture alone.** *He named the mechanism before anyone measured it.*

**✅ FIX (skin only): the backdrop MOVED UP from `.region-split` to `.region-shell`.** **The shell paints · the splits are TRANSPARENT · the tiles are OPAQUE** → **every gap — seam, hole, perimeter — is now the SAME ONE SURFACE**, showing through wherever no tile covers it.

**RE-DRIVEN:** shell `rgb(52,59,71)` + raster · split **transparent** · tile opaque `rgb(22,24,28)` · edge gap **5** · seam **5** · registry **67** · **MID-drag descriptor still `[1,2,7,2]`, AFTER `[176,124,700,200]`** — the splitter is untouched. **Screenshot confirms one continuous raster around and between the regions.**

**🔒 AND THIS IS ALREADY THE `M-RP-PLATE` ARCHITECTURE** — one backdrop behind the whole shell, splits transparent, tiles opaque. **Arrived at early, for free.** That milestone now only has to swap a `background-image` for a real element.

**⚠️ A near-miss worth recording:** the first edit was a two-hunk change; **the second hunk failed to match, so NEITHER applied** — but the *separate* removal from `.region-split` **did** land. For one build **nothing painted the grid surface at all.** *`Filesystem:edit_file` is all-or-nothing PER CALL, not per hunk — and the CDP read is what caught it, not the diff.* **Verify after every skin move; a diff that looks applied is not a render.**

---

### 2. 🔑 RETRACTION — THE DEPENDENCY CHAT CLAIMED YESTERDAY DOES NOT EXIST

**J-520 said, with a lock icon on it:**

> *"`.region-split` PAINTS today, so under the region-owned margin model a nested split's box would paint INTO its neighbour's margin and eat the gap. The model **requires transparent splits — which is exactly what `M-RP-PLATE` does** → **the region-owned gap and the backdrop plate are ONE MILESTONE.**"*

**⚠️ THE PREMISE WAS TRUE. THE CONCLUSION WAS WRONG.** The model requires the splits **not to PAINT** — and **making the splits stop painting is the two-line skin move in §1 above.** It is **not** the plate widget. ***Chat conflated "the splits must not paint" with "the milestone that makes the splits not paint". `M-RP-PLATE` is a SUPERSET, not a prerequisite.***

> ### 🔒 **→ THE BLOCKER IS GONE. JOE'S REGION-OWNED GAP MODEL IS BUILDABLE NOW.**
> A small component change: **every TILE takes `margin: G`** · **the seam becomes a ZERO-WIDTH flex child with `−G/2` margin each side** (cancelling the double) · **`--region-pad` is DELETED — the edge gap BECOMES the region's own margin.** Then **tile↔tile = tile↔edge = tile↔hole = G**, and it **composes at any nesting depth** because a split contributes nothing. **It can land before or after M-RP7.3 — Joe's call. The grid still has priority.**

> ### ⚠️ **AND THE PATTERN IS NOW THREE FOR THREE IN THIS ARC.**
> **N-118: two confident, coherent, wrong diagnoses**, both written into the harness as *"measured"*. **This: a confident, coherent, wrong DEPENDENCY**, written into three canonical records with a 🔒 on it. ***All three were internally consistent. None was checked against the thing it claimed about.***
>
> ***A dependency you have not TRIED TO BREAK is a dependency you have ASSUMED.*** *N-116 said the record can be self-consistent while the code is not. N-118 said my reasoning can be. This says my **architecture** can be — and a lock icon does not make it true.*

---

### 3. Files

`ui/assets/skin.css` (backdrop `.region-split` → `.region-shell`; splits explicitly transparent, with a *do-not-re-add* note) · `docs/xgen-dock-engine-phase0.md` **v1.7 → v1.8** (§4.5 corrected, §4.5.2 retraction) · `CLAUDE.md` · `docs/ROADMAP.md` · `JOURNAL.md`.

**No component, no Rust. `M-RP7.3 — the mutation algebra (pure)` is still next, and it still opens with N-120.**

---

## Entry J-520 — The gap becomes one number, and Joe's margin model turns out to BE the plate milestone; `M-RP-RESTART` filed

**Skin + records only. No component, no Rust, no registry change. Both items raised by Joe; the grid keeps priority — his call and the right one.**

---

### 1. ⚠️ The gap — and the first of his two arrows points at MY bug

Joe sent a screenshot with two red arrows: **no gap between the regions**, and **an edge gap "made from outside"**. He is right on both, and the first one is J-517's doing: **I shipped TWO tokens at TWO values** — `--region-seam: 1px` between tiles, `--region-pad: 6px` at the frame. ***A gap that means one thing at the edge and another between tiles is not a knob; it is a bug with two names.*** **He found it by looking at the screen** — the fourth time in this arc that the "Joe looks at it" slot has caught something no other instrument reached (M-RP7.1's hole · M-RP7.1b's chevron · N-119's hit area · this).

**✅ SHIPPED: `--region-gap: 5px`** (his number). `--region-pad` and `--region-seam` both **derive** from it — they keep separate names only because they are consumed in different places, and **they must never drift apart again.**

**RE-DRIVEN on the real client:** edge gap **5** · seam **5** · tile-to-tile **5** · grab zone **8px**, symmetric, entirely inside the gap · registry **67** · **MID-drag the descriptor still read `[1,2,7,2]`** while the tile moved, **AFTER `[198,102,700,200]`** (sum 1200, pair invariant, siblings ×100) · gap still 5px after the drag · selection 0. **The skin change is behaviour-neutral for the mechanics.**

**`--region-seam-hit` 3px → 1px** (mechanical, §0): it was sized for a **1px** seam. Against a **5px** one it would reach 3px **into each neighbour's edge** and eat clicks there.

---

### 2. 🔑 HIS MODEL IS RIGHT, IT IS NOT WHAT SHIPPED, AND IT TURNS OUT TO **BE** THE PLATE MILESTONE

Joe: *"those gaps are part of each region and are transparent, like a margin on a block element — **they overlap when they meet, not added** to each other. The gap has to be **5px (not 10px)** between both of regions and also between region and edge."*

**⚠️ That is TRUE MARGIN COLLAPSING, and FLEXBOX DOES NOT COLLAPSE MARGINS — they ADD.** It cannot be written the way it reads. **But the result is exactly reachable:**

> **Every TILE carries a full margin `G`. Splits carry NOTHING. The seam becomes a ZERO-WIDTH flex child with `−G/2` margin each side — which cancels the double.**

| boundary | arithmetic | result |
|---|---|---|
| tile ↔ tile | `G` + `−G` + `G` | **G** |
| tile ↔ frame edge | the tile's own margin | **G** — **`--region-pad` DISAPPEARS.** *The edge gap BECOMES the region's own margin. That is literally his model.* |
| tile ↔ **hole / backdrop** | the tile's own margin | **G** — ***today this case is ZERO*** |
| **nested** | the split contributes **nothing**, so the inner tile's own `G` is the whole gap | **G** — **composes at any depth** |

**And the seam's zero-width box lands DEAD CENTRE of the gap**, so the drag survives — still an element, still hit-testable. *(A `gap` never was. That is the whole reason M-RP7.2 had to build a seam element.)*

> ### 🔒 **AND THE FINDING: IT IS NOT A SCHEDULING PREFERENCE. IT IS A DEPENDENCY.**
> **`.region-split` PAINTS today** (`--s5` + the raster). Under this model **a nested split's box OVERLAPS its neighbour's margin — so its background would paint INTO the gap and eat it.** **The model REQUIRES the splits to be transparent** — which is **exactly what `M-RP-PLATE` does** (one plate behind the whole shell; splits stop painting).
>
> ***→ The region-owned gap and the backdrop plate are ONE MILESTONE.*** **`M-RP-PLATE` grew; it did not move.** *I had guessed the sequencing right for the wrong reason; the grep gave me the real one.*

**🔒 WHAT SHIPPED IS AN HONEST DOWN-PAYMENT, NOT THE MODEL.** The two boundaries Joe can **see today** are uniform at 5px. **The third — tile↔hole — is still ZERO, and it is stated in the skin comment, not hidden.** *Seams exist only BETWEEN a split's children, and a hole is not a child.*

---

### 3. ⏸️ `M-RP-RESTART` — File ▸ Restart and Revert. FILED; lands with M-RP7.5.

Joe proposed a **File ▸ Restart** item, *"disabled in states where it cannot be executed — no problem for me."* **The guard is right. But grounding it split the item in two:**

- **`Restart`** — bounce the process. **Real today**: **`tauri-plugin-process` is ALREADY in `xgen-client/Cargo.toml`** — that plugin *is* `restart()`/`exit()`. **⚠️ Though it is DECLARED AND NEVER WIRED** — no init in any `xgen-client/src/*.rs`, no frontend use. *A dependency nothing consumes.* Filed as its own small finding.
- **`Revert`** — *"no automatic save on exit, load last automatic save."* **⚠️ THERE IS NO AUTOMATIC SAVE. The grid does not write `session.layout` until M-RP7.5.**

> ### ⚠️ **SO "RESTART WITHOUT SAVING" AND "RESTART" ARE THE SAME THING TODAY**, and `Revert` would be greyed **100% of the time** — not *"in some moments"*. **A permanently-disabled item is not a guard; it is a promise.** *That is the painted-dead chrome this project keeps refusing (J-500 · 6.1j · and the identical reason M-RP7.6's grid lock cannot ship early: a lock over a verb that does not exist guards nothing.)*

**→ Ship both LIVE, together, with M-RP7.5 — the milestone that creates the state they discard.** Joe's disabled-guard is adopted for the genuine **runtime** cases.

**⚠️ One cost named now so it is not a surprise:** after **M-RP6.3** (the send verb), a restart tears down a live WS that may hold **unsent messages**. *Restart gets more expensive the more the app actually does.*

---

### 4. Files

`ui/assets/skin.css` (`--region-gap`; `--region-seam-hit` 3→1px; the stale J-517 two-knob comment replaced) · `docs/xgen-dock-engine-phase0.md` **v1.6 → v1.7** (§4.5.1 grows, §4.5.2 rewritten, §13 filed) · `CLAUDE.md` · `docs/ROADMAP.md` · `JOURNAL.md`.

**No component, no Rust, registry 67 (re-measured). `M-RP7.3 — the mutation algebra (pure)` is still next, and it still opens with N-120.**

---

## Entry J-519 — M-RP7.2 CLOSED: the seam drags, the descriptor is written once — and the re-drive reached a bug that inverts a resize

**M-RP7.2 — splitter resize on the seam ✅ CLOSED.** Code commit **`9faa38c`** [Clair] — 8 files, +512/−28. **Zero Rust · zero `ui/sampler` · zero `ui/node`** — verified by diffstat and by the identical `cargo test`, not asserted.

**Chat re-drove EVERY leg on the real client (Rule 5). Not one number below was taken on report.**

---

### 1. What shipped

**`mutate.ts`** (new, `core`, pure) — `resizeSplit(layout, path, seamIndex, fraction)`: integer scale-up, pair-only, total temperament (bad path · non-split · out-of-range seam · non-finite fraction → **input unchanged, never throws**), immutable rebuild. **`path: number[]`** threaded through `region-node` (derived; **no schema change**). The **`.region-seam` element** replaces the flex `gap` (a `gap` cannot be hit-tested). The **gesture**: pointer capture, live preview, min-clamp off the skin. **`resolve.ts` exported `isFoldAlong` / `splitShrinkWraps` / `carriesMainAxisWeight`** — one source for tile fold-mode, split shrink-wrap **and** seam liveness.

**All eight locks L1–L8 honoured as written.**

### 2. 🔑 THE CENTRAL CLAIM, PROVEN — AND IT WAS ONLY PROVABLE BECAUSE LEG 0 BUILT THE INSTRUMENT FIRST

> **MID-DRAG (button still down): the descriptor still read `[1,2,7,2]` while the tile had painted from 74px to 176px.**
> **AFTER (released): `[237, 63, 700, 200]`.** Sum **1200**. Pair total **invariant** (`1+2 = 3` → ×100 → `237+63 = 300`). **Untouched siblings exactly ×100** (`7→700`, `2→200`). **No jump on commit.**

***"Preview live, write the descriptor ONCE on release" is indistinguishable from "write on every move" if you can only read after `mouseReleased`.*** J-518's `-MidExpression` is the entire reason this is a measurement and not a hope.

### 3. Re-driven (Rule 5) — every number Chat's own

**Registry 67** — quiescent · empty store · **no selection** · nothing folded · **zero saved UI states**; `count === unique === 67`; **the seam does NOT register** (L8); **returns to exactly 67** after a full fold / shrink-wrap / resize churn. **⚠️ The client Clair handed over had a SELECTION ACTIVE** — the inspector was populated and `paragraph#region-inspector__empty` was gone. **A count read in that state is 71, not 67** (N-112). *Reloaded before measuring; a baseline read in an unstated state is not a baseline.*

`cargo test` **1517 / 0 / 62 — IDENTICAL** (56 terminators, complete run, **zero real failures under a CASE-SENSITIVE grep** — N-117) · `npm test` **59** (was 49) · `vite build` **169** (was 168).

**The hoist works, and the runbook's central trap is closed:** `getComputedStyle(seam).getPropertyValue('--region-min')` → **`"22px"`**. **Chromium resolves the nested `var(--region-stripe)` inside a custom property's computed value** — which was the open question. `--region-stripe` reads **22px on both the seam and a tile**: the hoist is behaviour-neutral. **7 seams.**

**Clamp (L6):** dragged hard into the frame → painted width stopped at **exactly 22px**, mid-drag *and* after; **`data-collapsed: null`, `data-fold-mode: null`**. ***It stops. It does not fold.*** (§4.2.)

**Inert seams (L4), driven not asserted:** folded-**along** → that seam `data-live="false"`, root seams live. **Shrink-wrap** → `data-shrinkwrap="true"`, column **22px**, **both adjoining root seams `false`** — **and the column's OWN internal seam `true`**, because two *width*-folded (across) leaves still carry their **height** weight. **The nuanced case falls out of ONE predicate with no special case.**

**Resize after fold:** the shrink-wrapped column's live seam `[3,1]` → **`[1946, 2054]`** (sum 4000, pair invariant), folds survived. **Fold after resize:** worked.

---

### 4. ⚠️ N-119 — CLAIR'S FINDING, AND THE RUNBOOK WAS ONE PAINT-ORDER FACT SHORT OF WORKING

**The runbook (Chat's) said: expand the seam's hit area with a `::before` overlay. It did not work, and the first drag did nothing.** The next tile **follows the seam in DOM order and painted over the far half of the overlay.**

**Chat re-drove it by DELETING the fix** (`z-index` removed live, `elementFromPoint` swept across the seam): x = 77–80 hit `.region-seam`; **x = 81–84 hit `.region-tile-body`** — ***including the seam's own painted pixel***, which spans 80.35–81.35. **Half of the VISIBLE seam was unclickable.** With `z-index: 1`: 77–84 all seam. **The fix is load-bearing, not cargo-cult.**

> ### 🔑 **`pointer-events` decides WHETHER an element is hit. PAINT ORDER decides WHICH element is hit.** An overlay that reaches into the next box reaches into a box that **paints after it**. **Expanding a hit area is two facts, and the CSS only states one.** **M-RP7.4's drop bands are the same shape of overlay — they inherit this.**

**🔒 Clair's deviation (`z-index: 1`) was CORRECT and is ACCEPTED.** *An expanded hit area that a sibling paints over is not expanded.* **And she found it by LOOKING (`elementFromPoint`), not by reading the CSS** — N-118's own lesson, applied by the implementer to the architect's design. **Second milestone running: Rule 6 caught the runbook, not the code.**

### 5. 🔑 N-120 — CLAIR FILED IT AS "LATENT, OUT OF SCOPE". CHAT REACHED IT. IT INVERTS A RESIZE.

**`region-node` renders the RESOLVED tree and threads `path` from the RESOLVED child index. `resizeSplit` walks the DESCRIPTOR by that path.** `resolve.ts` **drops** unknown leaves, `tabs`, and all-dropped splits. **Any drop, and the two index spaces diverge.**

**Planted a layout with ONE unknown `widgetId`** (`ghost`) — descriptor has 3 children, **2 tiles paint**, **1 seam**, and it sits between `spaces` and `rooms`:

| | before | after dragging that seam RIGHT (to ENLARGE `spaces`) |
|---|---|---|
| descriptor | `[1,1,1]` | **`[1660, 340, 1000]`** — *the **ghost** took 1660* |
| painted `spaces` | 660px | **335px — it HALVED** |
| painted `rooms` | 660px | 986px — it **grew** |

> ### ⚠️ **THE RESIZE DID THE EXACT OPPOSITE OF THE GESTURE**, because the pair actually resized was `ghost ↔ spaces`. **And 55% of the row's weight now belongs to a widget that does not exist** — which **M-RP7.5 would write to DISK**, and which would **reappear eating half the screen** if that widget ever came back.

**🔒 THE ASYMMETRY, PROVEN IN THE SAME POISONED LAYOUT:** folding `spaces` with the ghost present collapsed **`spaces`** — **correct** — because **fold addresses by `regionId`, an IDENTITY.** **Resize addresses by `path`, a POSITION** — and ***a position is only an address in the tree it was counted in.***

> ### 🔑 **THE GENERAL RULE:** ***a derived view may renumber. An index into a derived view is not an address into the source.*** `resolve.ts` **already knows** the descriptor index at the moment it drops — **it just throws it away.**

**⚠️ UNREACHABLE IN TODAY'S BUILD** (all 8 ids registered; `tabs` is never produced — D-116). **It becomes reachable the first time a widget id is retired or renamed and a user loads a saved workspace** — the **W-13 reconcile** case the project explicitly designed for, and M-RP7.1b proved the Load dialog is **three clicks** away. ***"Unreachable today" is the argument that has now been wrong five times here*** (N-091 · N-097 · N-099 · N-109 · N-116).

**→ NOT FILED. It becomes a REQUIRED LEG of `M-RP7.3 — the mutation algebra (pure)`, which is NEXT and which OWNS addressing.** *L5 said the addressing is "paid once" — **if it is wrong, we have paid once for the wrong thing.** A misaddressed resize nudges two integers; **a misaddressed `move` relocates a panel into the wrong branch.** `move` is not built on a broken address.*

### 6. Clair's finding ② — the runbook's premise was misattributed (harmless, and she fixed it better than it was written)

The runbook said *"`resolve.ts` already decides which children get an inline flex — REUSE that value."* **It does not** — that decision lived in `region-node` (shrink-wrap) and `region-tile` (fold-mode). **The INTENT (one source, D-067) was right and the FACT was wrong.** She exported the three predicates from `resolve.ts` and had all three call-sites consume them — **which dedups pre-existing parallel logic rather than adding a fourth copy.** *Better than compliance.*

### 7. Chat's own instrument, strengthened mid-re-drive

**`cdp-debug.ps1` gains `-KeepSelection`.** The harness was clearing the selection before every gesture — so it **avoided** N-118 instead of **testing** it. ***A harness that can only pass is a weak harness.*** **Control, same session:** two drags on the tile body with the clear suppressed → **`mousedown, mousemove, dragstart` — and then nothing. No `mouseup`.** **N-118 reproduced on the wire, with the native drag visible** (it was only *inferred* at J-518). **Subject:** the seam, same conditions, a live 21-char selection → **both consecutive drags committed, no `dragstart`, `mousedown: false`** — because `preventDefault()` on `pointerdown` **suppresses the compat mouse events**; the pointer events carry the drag. **Measured, not assumed.**

### 8. Records

`tasks/M_RP7_2_SPLITTER_RESIZE.md` → **COMPLETED**. Phase-0 → **v1.6**. `xgen-ui-notes.md` → **v0.86** (**N-119**, **N-120**). New tokens on `.region-shell`: `--region-min: var(--region-stripe)` · `--region-snap: 8px` · `--region-seam-hit: 3px`; **`--region-stripe` hoisted from `.region-tile`** (behaviour-neutral, 22/22 verified). **`--region-seam` is now the seam element's thickness — exactly as J-517 predicted: *"one token, both eras."***

**⚠️ THE M-RP7.1 SE CORNER TRIANGLE IS STILL PAINTED-DEAD, AND ITS DISCHARGER SAID "M-RP7.2".** This milestone resized on the **seam**, not the **corner**. **The countdown is NOT discharged — it is RE-POINTED, not quietly dropped.** → **`M-RP-SKIN`** decides whether a corner grip exists at all; **if it stays, it needs a milestone that makes it live.** *A disabled face is a countdown, and a countdown whose milestone came and went without discharging it is exactly the painted-dead chrome this project keeps refusing.*

---

## Entry J-518 — M-RP7.2 leg 0: the trusted-mouse harness lands, and it found the milestone's first real bug before a line of the milestone was written

**M-RP7.2 — splitter resize on the seam: OPENED. Leg 0 (Chat) DONE. Runbook ACTIVE for Clair. No component code, no Rust.**

---

### 1. Why the harness went first, and it was not an ordering preference

The brief said it plainly: *"the arithmetic is the cheapest real mechanic in the arc; **budget for the harness, not the arithmetic.**"* So the harness was **not** written into Clair's runbook. **If it did not work, we would have discovered that AFTER a splitter had shipped that nobody could prove** — and tooling is Chat's lane in any case.

`cdp-debug.ps1` gains **`-Mode click`** and **`-Mode drag`**, driving real `Input.dispatchMouseEvent` (browser-level ⇒ **trusted**, unlike an `eval`-synthesised event, which is `isTrusted:false` and fires no native defaults — J-496).

> ### 🔑 **AND ONE PARAMETER THAT IS NOT A CONVENIENCE: `-MidExpression`, EVALUATED WHILE THE BUTTON IS STILL DOWN.**
> M-RP7.2's central design lock is *"preview live, write the descriptor ONCE, on release."* **That claim is UNPROVABLE if you can only read after `mouseReleased`** — it is indistinguishable from writing on every move. **The mid-drag read IS the proof.** Verified: MID sees the moves and **no `mouseup`**; AFTER sees it.

### 2. CALIBRATED, not assumed — and the calibration target was already-verified behaviour

**Clicked a fold button at its measured `getBoundingClientRect()` centre `(206,42)` and watched `collapsed:"width"` appear.** A wrong coordinate space simply **misses** — so the instrument checks itself against a behaviour M-RP7.1b already proved.

**→ 🔒 Coordinates are CSS pixels, the same space `getBoundingClientRect()` returns. DPR 1.25 does NOT apply.** `isTrusted=true` throughout; `buttons=1` across the moves; **three consecutive drags byte-identical**.

*(A small thing that says something: the second click at the same coordinates did nothing — because **folding MOVES the button** into the rotated strip, to `(15,785)`. The instrument was right and my assumption was wrong. **Re-measure coordinates before every gesture; a rect is not a constant.**)*

### 3. ⚠️ THE HARNESS LIED FIRST — twice — and both lies were HARMLESS-LOOKING

**(a) A HOVER reported `buttons=1`.** I was sending `button:"left"` on *every* event, including button-up moves; Chromium derives from it. Harmless to a splitter (it only listens after `pointerdown`) — **but it would have silently poisoned M-RP7.4, whose drop-band hover must be readable with the button UP.** Fixed: `button:"none"` on a button-up move.

**(b) PowerShell would have emitted `123,5` instead of `123.5`.** A `[double]` stringifies with the **current culture's** decimal separator; on this box that is a comma, which is not JSON, and the CDP frame dies with an error that looks nothing like a locale bug. **Integer coordinates end to end.**

---

### 4. 🔑 N-118 — AND THIS ONE IS NOT A HARNESS BUG. IT IS A BUG IN THE MILESTONE THAT DOES NOT EXIST YET.

**The symptom:** the drag worked perfectly on a fresh page, and then the **second** drag from the same point delivered **three** events — hover-move, `mousedown`, **one** `mousemove` — and then **nothing. No further moves. No `mouseup` at all.** Every CDP ack came back clean.

**The cause:** a drag across selectable text fires `selectstart` and leaves a **SELECTION** behind. The next drag presses **on that selection** — and **Chromium treats a selection as DRAGGABLE CONTENT.** It opens a **native HTML5 drag session**, which takes the mouse and stops delivering `mousemove` and `mouseup` to the page entirely.

> ### ⚠️ **A SPLITTER THAT IS NOT `user-select: none` WILL SELECT THE TEXT UNDER IT ON THE FIRST DRAG — AND THE SECOND DRAG WILL BE SWALLOWED, LEAVING THE TILE STUCK TO THE CURSOR WITH NO `mouseup` TO END IT.**
> **A real user reproduces this by dragging a splitter twice.** → `user-select: none` + `preventDefault()` + `setPointerCapture` are **DoD legs in the runbook, not footnotes.** **M-RP7.4 inherits the trap wholesale** — it drags a *grip*, over a *title stripe*, next to a *title*: all text.

> ### 🔑 **THE EXPENSIVE LESSON, AND IT IS ABOUT ME, NOT THE CODE: I DIAGNOSED IT WRONG TWICE, CONFIDENTLY, AND BOTH WRONG DIAGNOSES WERE *COHERENT*.**
> **First:** *"a CDP ack doesn't mean the renderer ran it — the events are arriving LATE"* → replaced the sleep with an rAF barrier. **Wrong**: a read **two seconds later** showed the events had **never arrived at all.**
> **Then:** *"interleaving `Runtime.evaluate` between moves KILLS the input stream — the instrument is destroying what it measures"* → **and I wrote that into the file as a measured finding.** **Also wrong**: removing the barrier changed nothing.
> **Both stories fit the data. Neither was true.** What actually found it was **reloading the page and watching the symptom vanish** — changing one variable and *looking*, not reasoning harder.
>
> ***N-116 said the RECORD can be self-consistent while the code is not. N-118 says MY OWN REASONING can be self-consistent while being wrong — and it will happily enter the record wearing the word "measured" if nothing is built to disprove it.*** *Both false findings were live in `cdp-debug.ps1` as authoritative comments before the third experiment removed them.*

---

### 5. The runbook — `tasks/M_RP7_2_SPLITTER_RESIZE.md` v1.0, ACTIVE

**Eight design locks taken under §0 autonomy (mechanics are Chat's; only graphical appearance is Joe's):**

| # | lock |
|---|---|
| **L1** | **`mutate.ts` is BORN at 7.2** with `resizeSplit` only; 7.3 adds `move` and pulls `fold` out of the shell. **⚠️ THE ARC TABLE PUT THE ALGEBRA *AFTER* THE FIRST MUTATION, AND THAT CANNOT BE TRUE — the first mutation IS algebra.** The alternative was tree surgery in two places for one milestone. |
| **L2** | **Integers only** (§7 bans floats, and it is right). Resolution comes from an **exact integer scale-up**: `[1,2,7,2]` → ×100 → `[100,200,700,200]`, then **only the dragged pair moves**. *Untouched siblings keep their proportions to the byte; the pair's total is invariant, so nothing else moves.* **Cost, stated: a saved workspace reads the scaled numbers after a drag. That is the price of §7's own lock, and it is mine to pay.** |
| **L3** | **Live preview; integers written ONCE on `pointerup`.** A float never reaches the descriptor even if the user thrashes, and the CDP proof is **one clean diff** instead of sixty. |
| **L4** | **A seam is draggable iff BOTH neighbours carry a main-axis weight.** One derived predicate; folded-along and shrink-wrapped fall out of it with **no special case**. |
| **L5** | **`path: number[]`, derived. NO schema change** (a split id would be a key nothing round-trips — the M-RP6.1k finding). **M-RP7.4's `move` needs the identical addressing, so it is paid once.** |
| **L6** | **The min-clamp reads the skin** (N-090). **It stops; it never auto-folds** (§4.2). |
| **L7** | **No keyboard resize** — it would put a tab stop on **all 7 seams** nobody asked for, *and* dodge the harness this milestone exists to land. Filed. |
| **L8** | **The seam does NOT register.** `.region-split` has no getter either — the seam is its chrome. **Registry stays 67**; proof is the tree diff + painted geometry. |

**⚠️ I over-asked at the top of the session** — three questions to Joe, two of which were **mine to answer** under §0. Retracted and taken. *Autonomy is not a licence to be vague; it is an obligation to decide.*

**Also retracted: a scrollbar-collision risk I NAMED WITHOUT MEASURING.** The invisible hit zone was said to clash with the tile body's scrollbar — **I asserted that; I did not check it.** Grounding cuts both ways: *do not name a risk you have not grounded* (rule ③). The seam keeps the 1px hairline Joe already approved and grows only an **invisible** hit area; if that turns out to need a visible change, **it is a finding, not a licence.**

---

### 6. Files

| file | change |
|---|---|
| `cdp-debug.ps1` | `-Mode click` · `-Mode drag` · `-MidExpression` · `-At/-From/-To/-Steps`; selection clear before every gesture; rAF read barrier; integer-only coords |
| `tasks/M_RP7_2_SPLITTER_RESIZE.md` | **NEW — v1.0, ACTIVE.** Clair's runbook |
| `tasks/CDP_DEBUG_HARNESS.md` | **v1.3 → v1.4** — the trusted-input section |
| `ui/docs/xgen-ui-notes.md` | **v0.84 → v0.85** — **N-118** |
| `CLAUDE.md` · `docs/ROADMAP.md` | M-RP7.2 → 🟢 ACTIVE; leg 0 recorded |

**No `ui/core/**`, no `ui/client/src/**`, no Rust. Registry unchanged (67, re-measured this session).**

---

## Entry J-517 — The space around the regions and the space under them: two knobs, one filed milestone, and a discharger pointed at the wrong place

**Records + skin only. NO component change, NO Rust, NO registry change. Design walk for M-RP7.2 opened; no code written — Joe has not said go.**

---

### 1. Two questions from Joe, and both were really the same question

Joe asked, in order: *"have we a gap possibility between regions and grid edges? I would like to have that due to possible skin customization request — set it to non-zero for development inspection"*, and then: *"the space under regions aka 'the hole' will be customizable, and there will be some static or dynamic visual plate — from solid black to animated reactive colour fractal clouds."*

**They are the same surface.** The perimeter and the hole are both *"grid, but no tile here"* — and once there is a backdrop plate, **they light up together.** That is why the second answer changed the first one, and why they travel in one commit.

---

### 2. 🎛️ The two spacing knobs — GROUNDED FIRST, and the grounding found that neither existed

**Answer to the literal question: NO, and worse than no.**

- **`.region-shell` had no padding at all** — and `.app-center` sets **`padding: 0` explicitly** (J-499's D5). The grid has been welded to the frame edge since renderer A shipped.
- **`.region-split` carried a hardcoded `gap: 1px`** — **not a token, not a knob.** Every record calls the hairline seams *"skin"*; the number was a literal.

Both are now **PROVISIONAL** tokens on `.region-shell`:

| token | what it spaces | value |
|---|---|---|
| **`--region-pad`** | the **grid ↔ frame edge** | **6px** — non-zero **on purpose**, so it is visible for development inspection (Joe's ask) |
| **`--region-seam`** | **between tiles** | **1px** — the same hairline, the same value; now tunable |

> **🔑 `--region-seam` IS NOT A THROWAWAY — IT SURVIVES THE NEXT MILESTONE.** At **M-RP7.2** the flex `gap` **becomes a real seam ELEMENT** (a `gap` cannot be hit-tested — that is the whole reason the splitter needs one) **and this token becomes its thickness.** *One token, both eras, so retuning the seam never means finding two places.*

> ### 🔒 **THE PERIMETER IS NOT A SEAM AND CAN NEVER BECOME ONE.**
> Seams exist **only BETWEEN a split's children** — the `{#each}` interior. **There is no leading or trailing seam.** So the outer edge **cannot** grow a drag cursor or a hit zone at M-RP7.2, structurally. *It costs the splitter milestone nothing, and — the part that matters — **it cannot lie to the user about a drag that does not exist**.*

**Ratios UNAFFECTED, and this was checked rather than assumed:** flex distributes over the **reduced content box**, so `[1,2,7,2]` still holds **exactly** — only the absolute px shrink by `2×pad`. Global **`box-sizing: border-box`** (`xgen-normalize.css:21`, **grepped**) means the padding cannot overflow the frame. *A `content-box` shell with padding would have blown the frame out, and that is not a thing to find out by looking.*

---

### 3. ⏸️ `M-RP-PLATE` — the grid backdrop: an inert, live-switchable plate widget under the tiles. FILED, NOT BUILT.

**Joe's frame is right, and — the finding — the project had already built the mechanism one level down and nobody had connected the two.**

**🔑 GROUNDED, NOT RECALLED:** `message-stream.svelte` has shipped since **J-482** with **`background?: WidgetMount[]`** — an **ARRAY** (so a *stack* of plates is free) · `position: absolute; inset: 0` · **`pointer-events: none`** · unknown-`widgetId` **dropped** (W-13) · and a **`backgroundLive`** switch passed into every mount, whose own comment reads: *"a reactive widget renders frozen when false; a static object ignores it."* Locked at **J-481**. **Chat wallpaper.**

***Joe is describing the same object one level up: grid wallpaper.*** **"Solid black" and "animated reactive fractal clouds" are the SAME SEAM** — one is a `div` with a colour, the other owns a canvas, **and the host never learns the difference.**

**🔑 IT COSTS NO SURFACE — AND THE TAXONOMY HAD ALREADY RULED ON IT.** W-12 (amended): a widget declares **at most one** of `region · shelf · window · none`. A backdrop plate declares **none of them** — which is not a hole in the taxonomy but **`xgen-widget-surfaces-phase0.md` §3.2: *content rendered inside a HOST is not a surface*** — the identical ruling that settled `temperature-indicator` (J-502). → **no tile, no region, no shelf face, no new surface kind, no W-12 conflict.** *The answer was on the shelf; it did not need inventing.*

**🔑 AND THE ONE THING THAT WOULD HAVE BEEN EASY TO GET WRONG: A HOLE CANNOT HOLD ANYTHING.** A hole is **flex leftover space** — **no element, no address, no identity** (which is *precisely* why D-116 holds and §7.1's lattice stayed out). **So the plate is a BACKDROP, not a hole-filler:**

> **ONE plate, `inset: 0`, behind the whole `.region-shell`.** Not one per split — *a cloud that restarts at every split boundary reads as N surfaces, not one.* **The tiles are OPAQUE, so what shows through IS the holes** — identical mechanic to today's background, **promoted from *paint* to *element***. **And it lights the `--region-pad` perimeter for free**, which is what fuses §2 and §3 into one answer.

> ### 🔒 **THE PLATE MAY READ THE POINTER. IT MAY NEVER CAPTURE IT.**
> `pointer-events: none`; a passive listener only. ***The instant a hole becomes clickable it has an ADDRESS*** — and **D-116** (*a target tile is an address*) falls, and **§7.1's lattice argument is live again.** ***A reactive backdrop is fine. A clickable one retires the tree.***

**⚠️ IT RETIRES A §4.5 CLAIM, AND THAT IS THE FINDING.** §4.5 says the raster is *"a BACKGROUND on the split container — zero new DOM, one skin rule."* A CSS background does solid, gradient, pattern, even keyframes. **It cannot do a canvas, a shader, or anything reactive.** The moment the plate is dynamic **it is an ELEMENT.** → **today's dot raster is not the seed of the plate; it is the placeholder the plate REPLACES.**

**🔒 BUT PROMOTED, NOT DELETED:** the dev raster becomes the **first *system* plate widget**, so the socket **ships FED** — *an unfed branch is an unverified branch* (D-065 / N-091). **"Solid black" is then a setting on it, not a special case.**

---

### 4. ⚠️ A DISCHARGER WAS POINTING AT THE WRONG MILESTONE — CORRECTED

The hole raster shipped **PROVISIONAL** with **`M-RP-SKIN`** named as its discharger (J-516). **That is now wrong, and it is wrong in a way that would have cost real work:**

> **`M-RP-SKIN` would TUNE the raster. `M-RP-PLATE` DELETES it.**

→ **the raster's discharger is re-pointed to `M-RP-PLATE`**, in the Phase-0, the ROADMAP, `CLAUDE.md` **and in the `skin.css` comment itself**, so nobody opens the appearance pass and spends an afternoon tuning a dot grid that a later milestone throws away. ***This is the J-495 argument that rejected the interim DWM title-bar tint: do not polish what you have already decided to replace.***

**Every other provisional in the arc still points at `M-RP-SKIN`** (fold chevrons · stripe/grip/triangle sizing · the folded strip's form · **the two new tokens**). **None points at nothing** — the countdown rule survives; one arrow just moved.

---

### 5. 🔒 Why it is FILED and not BUILT — and it is a LOCK, not a preference

Joe's sentence contains the blocker: *"…a background widget **which sets it by its own setting**."*

> **THERE IS NO SETTINGS MECHANISM, AND THAT IS DELIBERATE.** **J-513** filed the Ch6 `settings_schema`-vs-plugin-component collision as **explicitly undecided**, binding: ***nothing is built toward either until the grid works.***

So the plate **cannot** be built today without picking the exact thing that is fenced off. **Joe, unprompted, landed on the same conclusion from the other side:**

> ***"That is why i don't want to solve it now. Now has priority widget grid with functional empty regions. Background widget we can create after the grid concept works."***

**Same shape as `M-RP-SKIN`, same reason: you cannot tune — or plug a widget into — an appearance whose mechanics are still moving underneath it.**

> **🔒 THIS ARC RESERVES NOTHING FOR IT.** No prop on `region-shell`, no descriptor key, no store, no manifest slot. *A key nothing writes is a key nobody has round-tripped* (the M-RP6.1k finding). **ZERO impact on M-RP7.2.**

---

### 6. One stale record found and fixed in passing

`docs/ROADMAP.md`'s **M-RP7 arc summary** still read **M-RP7.1b** as *"🟢 design locked J-515"* — stale since **J-516 closed it**. The dedicated entry three lines above it was correct; the summary had not been re-read. **Corrected to ✅ CLOSED (J-516).** *The session brief said the record was consistent and the tree was clean; it was 99% right, and the 1% is why you still look.*

---

### 7. Files

| file | change |
|---|---|
| `ui/assets/skin.css` | `--region-pad: 6px` + `--region-seam: 1px` on `.region-shell`; `padding: var(--region-pad)`; `.region-split` `gap` reads the token; **the raster's discharger comment re-pointed to `M-RP-PLATE`** |
| `docs/xgen-dock-engine-phase0.md` | **v1.4 → v1.5** — new **§4.5.1** (`M-RP-PLATE`) + **§4.5.2** (the two knobs); §11's `M-RP-SKIN` block gains the raster exception; §13 filed list |
| `docs/ROADMAP.md` | **v4.87 → v4.88** — `M-RP-PLATE` filed; `M-RP-SKIN` discharger list corrected; M-RP7.1b marked CLOSED in the arc summary |
| `CLAUDE.md` | PLAY block: `M-RP-PLATE` + the two spacing knobs; `M-RP-SKIN` exception |
| `JOURNAL.md` | this entry |

**No `ui/core/**`, no `ui/client/src/**`, no `ui/sampler/**`, no Rust. Registry unchanged. No new `D` — nothing here is a decision Joe was asked to lock; it is a design filed and a record corrected.**

**Next: M-RP7.2 — splitter resize on the seam.** Design walk delivered; the seven §0 mechanics are taken (`mutate.ts` born at 7.2 with `resizeSplit` · integer scale-up, never floats · live preview / integers on release · seam dead when a neighbour has no main-axis weight · `path: number[]`, no schema change · min-clamp reads the skin · no keyboard resize). **Awaiting Joe's go before any code.**

---

## Entry J-516 — M-RP7.1b CLOSED: the fold axis is the user's, splits shrink-wrap, holes have a floor — and the project's migrate path finally RAN, twelve months after it was first written down

**M-RP7.1b — the fold axis becomes the user's choice; splits shrink-wrap; the hole gets a floor ✅ CLOSED.** Two code commits [Clair]: **`0f25e50`** (9 files, +331/−92) + **`14eb4d8`** (the `.region-title-buttons` wrapper, markup only). One appearance-fix commit [Chat, on Joe's review]. **Zero Rust · zero `ui/sampler` · zero `ui/node`** — verified by diffstat, not asserted.

**Design was locked at J-515. Chat re-drove EVERY leg on the real client (Rule 5); not one number below was taken on report.**

### What shipped

**`collapsed` stopped being a flag and became a DIRECTION.** `FoldAxis = 'width' | 'height'`, **stored** — because it is **user intent, and user intent is not derivable from anything.** `version: 3`. The **along/across mode** stays **derived** (a fact about the tree, so it can go stale; the direction is a fact about the user, so it cannot). **The tile reflects `data-collapsed` · `data-axis` · `data-fold-mode`, and the SKIN does the rest** — `along` ⇒ `flex: 0 0 auto` and siblings absorb; `across` ⇒ the tile **keeps** its inline weight, takes `align-self: flex-start`, and **a hole opens**.

**🔒 THE SPLIT SHRINK-WRAP (§4.4) — the part Joe did not ask for and the part that solved his screenshot.** A split whose children are **all folded ACROSS its own axis** collapses to their folded size and **returns its weight to its siblings by the `flex` they already have.** **Measured: the left column went 215px → 22px, and the freed 2/12 redistributed across the remaining three at EXACTLY `1:7:2`** (`127 / 888 / 254` against a computed `126.9 / 888.3 / 253.8`). **The hole closed to ZERO in the case Joe actually hit.** **The descriptor was never touched** — `rootSizes [1,2,7,2]`, `colSizes [3,1]` throughout. **Weights are not mutated by folding; they are ignored while folded and honoured again on unfold.**

**⚠️ And it deliberately does NOT fire on a MIXED fold** (`[<]` + `[v]`): the column stays 215px wide and a 193px hole opens. **Verified reachable, and KEPT.** *The user asked for two different things and gets a column that fits neither. The raster explains it. **No magic, no guessing what they meant.***

### 🔑 THE MIGRATE RAN. FOR THE FIRST TIME. EVER.

**Grounded before the runbook was written: `migrate` DID NOT EXIST.** The word appeared **only in comments** (`types.ts:16-17`, `layout-default.ts:63`). **`version` had been bumped TWICE and there had never been a migrate FUNCTION at all.** *A path described in three documents and implemented in none is not a path; it is a plan.*

**`migrateLayout` is now real** — `v2 → v3`: each `collapsed: true` leaf reads its parent's `dir` and gets the explicit direction (**the old derived rule, made honest**). **Six vitest cases, fed with hand-built `v2` trees** — both parent kinds, the root case, `false → key deleted`, idempotence, and garbage → default-never-throw.

> **⚠️ Clair's hand-back said the live leg was NOT drivable** (`__XGEN_LAYOUT__.set()` bypasses the migrate). **It IS drivable, and Chat drove it:** plant a `v2` named state, then drive the **REAL Load dialog** — shelf face → combobox → Load → `handleUistateLoad` → `migrateLayout`. **Measured: `version 2→3` · `spaces` (ROW parent) → `'width'` · `rooms` (COL parent) → `'height'` · `self` (no key) → KEY STILL ABSENT, not `false`.** *A branch you cannot reach through the product is a branch you have not tested — and the way to reach it was three clicks, not a new harness.*

**🔑 Two properties fell out of the live run that no test would have shown:**
1. **A migrated v2 layout is HOLE-FREE BY CONSTRUCTION.** Both migrated tiles render `foldMode: "along"` — because the **old derived rule only ever folded ALONG the parent axis.** **Nobody's saved workspace can come back from this schema change with a hole in it.**
2. **The migrate NEVER REWRITES THE STORED STATE.** It is a **read-path transform**: the saved record stays `v2` on disk forever and re-migrates on every load. **Idempotent, non-destructive, and it never silently edits a file the user wrote.**

### ⚠️ THE DEVIATION — FLAGGED BY CLAIR (Rule 6), BLESSED, AND IT WAS THE RUNBOOK'S BUG

`migrateLayout(raw, **fallback**)` — not the runbook §4's bare `migrateLayout(raw)`. **The runbook said it should fall back to `DEFAULT_LAYOUT`** — but **`resolve.ts` is `core` and `DEFAULT_LAYOUT` is SHELL-LOCAL.** Chat's runbook would have made **`core` import the client's default tree: a second source of truth for the default, which is the exact D-067 drift the J-499 grounding killed.** **She injected it from the shell instead and preserved N-095's never-null.** **The deviation is correct and the runbook was wrong.** *(Rule 6 earned its keep: an implementer who absorbs a bad instruction silently ships the architect's mistake.)*

### 🔒 CONVENTION A — and the label was never true of the build (Joe, on review)

Joe locked *"convention B (disclosure)"* at J-515. **He ran it, and found the vertical chevron pointing the wrong way.** He was right, and the reason is sharper than a wrong rotation:

> **WE NEVER SHIPPED CONVENTION B.** The **WIDTH** button was **always convention A** (open `<` = *"I will go LEFT"*; B would have pointed `>`, at the content). **Only the HEIGHT button spoke disclosure** (open `v` = *"my body is below"*) — taken straight from Joe's original `[<][V]` sketch, **which mixes the two conventions in a single stripe.** **And the skin comment sitting directly above the rotations already described convention A, correctly.** ***The label was wrong, the prose was right, and one rotation value contradicted both.***

**🔒 THE RULE NOW, no exceptions: THE CHEVRON ALWAYS POINTS WHERE THE REGION WILL GO IF YOU PRESS IT** — enabled or disabled, folded or not. Unfolded `<` / `^`; folded, the live button points **back** (`>` / `v`); **the disabled button keeps its open glyph, still truthfully naming what it WOULD do.** **The fix was TWO ROTATION VALUES SWAPPED. The width button was never wrong.**

**→ N-116.** *Re-reading the chapter would never have caught this: **the chapter was self-consistent and the CODE was not.** M-RP7.1's arc slot said its purpose was "this is where Joe sees it and corrects the appearance" — and for the second milestone running, that slot has paid for the whole leg.*

**And `.region-title-buttons` got its gap** — `--region-fold-gap: 0` in the tile's tunable-token block. **⚠️ `gap: 0` alone would have been DEAD:** the span shipped `display: inline`, where **`gap` is INERT** — **and an inert knob IS a reservation** (§4.3.1: *RESERVE NOTHING*). **`display: inline-flex` is what makes it a control.** *The test: can Joe change the value today and see the render move? Yes. Then it is a knob with a declared default, not a placeholder.*

### ⏸️ `M-RP-SKIN` — THE APPEARANCE PASS. FILED (Joe, 2026-07-13)

Joe: ***"majority of graphical elements will be changed or updated after ui mechanics completion."***

**🔑 This is the NAMED DISCHARGER for every `PROVISIONAL` marker in the grid arc** — the hole raster, the chevrons, the stripe/grip/triangle sizing, the folded strip's form. **The countdown rule, satisfied at ARC scale instead of per-element: WHO = Joe, WHICH MILESTONE = `M-RP-SKIN`.** *It is not a deferral of a decision. **You cannot tune an appearance whose mechanics are still moving underneath it.** Every provisional in this arc now points somewhere; none of them points at nothing.*

### MEASURED — Chat re-drove all of it

| gate | result |
|---|---|
| `npm test` (**`ui/sampler`**) | **49** (43 → 49; **+6 migrate**) |
| `vite build` (**`ui/client`**) | **168 modules**, clean |
| `cargo test --workspace` | **1517 / 0 / 62 — IDENTICAL.** 56 terminators asserted; **run twice.** *The inverse leg: identical PROVES no Rust landed.* |
| sampler catalogue | **328** — launched and counted, unchanged |
| client registry | **67** — quiescent, empty store, **no selection**, nothing folded |
| ratios | `[1,2,7,2]` **exact at 1291px** (a 7th distinct width) |

**CDP legs, all reproduced:** `[v]` in a `col` → **along** (h:22 stripe, width kept, sibling absorbs to 785, `flex` omitted, no hole) · `[<]` in a `col` → **across** (w:22 strip, h:202 kept, `align-self:flex-start`, **`flex` KEPT**, **193px hole**, raster painted, **tile opaque**) · `[<]` on **both** → **shrink-wrap** (215→22, `flex` omitted, **hole 0**, `1:7:2` exact) · **mixed** → no false shrink-wrap · **buttons** (unused one `aria-disabled="true"` **with `tabIndex:0`** — keyboard-reachable, native `disabled:false`; **clicking it is a true no-op**; the used one's `aria-disabled` is **ABSENT, not `"false"`**; unfold → **exactly 67**, key **deleted**) · **migrate live** · **the bus survives the fold** (self folded ACROSS, inspector still renders `kind:'identity'`, `rowCount:4` — *an unfed branch is an unverified branch, and this is the arc's only proof the mechanics do not lean on their content*) · **accent-neutral** (byte-identical under magenta, `readable:true` asserted first) · **no double-rotation** (stripe `vertical-rl`, glyph angles **identical** inside the rotated strip: 315°/225°).

### 🔑 A FOURTH REGISTRY AXIS — AND IT LIVES ON DISK (N-115)

Found while chasing an unexpected count: **ONE SAVED UI STATE ADDS EXACTLY +4 TO THE REGISTRY.** Measured by id-diff, not inferred:

```
textfield#uistate-load-pick__input · combobox#uistate-load-pick
button#uistate-load-go            · button#uistate-load-del
```

The Load dialog's picker and its two action buttons are **element-absent when there is nothing to pick** — **which is CORRECT** (J-500's own posture) — and materialise the moment one state exists.

> **`67` is only true on a machine with ZERO saved UI states. One click on the diskette and the client reads 71 FOREVER.** Every later baseline would then be off by 4 **on a machine where nothing is wrong** — **N-108's exact shape, except the "data file" is one the USER writes with a button in the UI.**
>
> **⚠️ And `71` is now AMBIGUOUS: selection-active = 71. Zero-selection + one-saved-state = 71. Same number, two causes.**

**Axes now: quiescence (N-105) · store contents (N-108) · selection (N-112) · named-state count (N-115).**

**⚠️ One anomaly, HONESTLY UNRESOLVED and deliberately not chased (Joe: *"we will see if this issue will generate problems in the future. leave it right now"*).** On **one** client launch the registry read **71** with **Joe's real identity auto-selected** (real pubkey — not Chat's probe entity). **Three subsequent fresh launches all rested at 67 with no selection.** `self-panel` selects on `onActivate` (a click), never on mount — so **something activated it, and Chat could not establish what.** **Filed, not explained. No cause is invented here.** *It does, though, VALIDATE N-112 rather than dent it: one click on the self-panel moves the baseline by 4.*

### ⚠️ TWO OF CHAT'S OWN CHECKS LIED THIS SESSION — neither was the build's fault (N-117)

1. A `FAILED|panicked` grep over the cargo log matched **`0 failed`** case-insensitively → **58 phantom failures** against a suite with zero.
2. **The accent-neutrality leg CLICKED and READ in ONE eval.** Svelte had not flushed, so the "before" read was the **PRE-fold DOM** — and the comparison ran across **two different fold states**. **Chat nearly recorded a false accent-leak.** *N-099 says split the state-change and the DOM-read across two evals. This is the near-miss that proves it, and Chat broke its own rule to produce it.*

### Still NOT locked

**⚠️ NO FOLD `D`.** Joe's word at J-515 was *"honestly i have to see it in practice."* **He has now seen it, and has not asked for the lock.** **Phase-0 v1.4 carries the design; `DECISIONS.md` carries what Joe has decided.** **D-116 still stands alone.**

**Filed, not touched:** recursive shrink-wrap (a split of splits) · the rotation-direction user setting · the **`mergeClasses` dedupe sweep** (N-113 — seen again live this session: the load dialog renders `class="combobox combobox"`) · persistence (**M-RP7.5**) · the grid lock (**M-RP7.6**) · **`M-RP-SKIN`**.

**🟢 NEXT-ACTIVE = M-RP7.2 — splitter resize on the seam.**

---

## Entry J-515 — M-RP7.1b DESIGN LOCKED: the fold axis becomes the user's choice; splits shrink-wrap; the hole gets a floor — and the milestone that found it was the one whose whole job was to be looked at

**🟢 M-RP7.1b — the fold axis becomes the user's choice; splits shrink-wrap; the hole gets a floor. DESIGN LOCKED 2026-07-13 (Joe).** Runbook `tasks/M_RP7_1B_FOLD_AXIS.md` · `docs/xgen-dock-engine-phase0.md` → **v1.3** (§4.1 rewritten · §4.3 amended · **new §4.4 + §4.5** · §7.1 amended · §11 renumbered). **⚠️ NO NEW `D`, AND THAT IS THE POINT — see below.**

**Design-only. No code. The runbook goes to Clair.**

### 🔑 The finding was a screenshot

M-RP7.1's own arc table said its purpose was *"this is where Joe sees it and corrects the appearance."* **He did, and it corrected more than the appearance.** He folded two regions, saw the empty band the shipped build leaves behind, and raised three things. **They turned out to be one thing.**

**⚠️ The holes are NOT a consequence of the new design. They are IN THE SHIPPED BUILD.** Phase-0 §4.1 promised *"No hole is ever created"* — **true about a tile's CROSS axis, silent about the MAIN one.** Fold **every** child of a split and the split under-fills. **N-111: a proof about one node is not a proof about the tree** — the §4.1 argument reasoned about a single tile, and **a single tile cannot see its siblings.** *(Sixth in the family: N-091 · N-097 · N-099 · N-109 · N-110 · N-111. The first five are a **check** that saw the wrong subject; this one is a **proof** that saw too small a subject.)*

### 🔒 What Joe locked

**① The fold axis is the USER'S CHOICE — two buttons.** `[<]` folds to the left (collapses **width** → vertical strip); `[v]` folds to the top (collapses **height** → horizontal stripe). **When folded, the unused button is DISABLED and the used one unfolds.** **Two axes, NOT four directions.**

> **What it costs, honestly: §4.1's elegant free property DIES.** *"Drag a folded tile from a column into a row and it re-orients itself"* — gone. **And it is the right trade: that property was elegant for the TREE and SURPRISING for the PERSON.** A user who folds a thing, drags it, and finds it has silently changed shape has not been served by an invariant. **Joe's rule is boring and predictable: I chose left; it stays left.**
>
> **And it finishes a sentence §4.1 only half-wrote.** §4.1's stated goal was *"foldability must not be an accident of placement"* — **it made fold AVAILABLE everywhere and left the DIRECTION an accident of the tree.**

**② A split SHRINK-WRAPS when all its children fold ACROSS it (§4.4).** **This is the part Joe did not ask for and it is the part that actually solves his screenshot.**

> **The mechanical fact he was hitting:** the left column is a `col` split with weight **2 of 12** in the outer `row`. **Fold is a LEAF verb. A split's width is a SPLIT property, in the PARENT's `sizes[]`. A leaf verb cannot reach a split property.** → **fold Rooms and Self any way you like and that column is still 2/12 wide.**
>
> **❌ The obvious fix was rejected on Joe's own lock:** `collapsed` on a **split** node. A leaf gets its title from `CLIENT_PLUGINS.name`; **a split has NO NAME** → it needs a name field, a UI to set it, and **chrome on every split, nested, forever**. ***That is a group container promoted back to a MAIN FORM*** — the exact thing §5 killed (*"group containers will be contained, but not as a main form"*). **It would undo his own lock to solve a problem his other idea already solves.**
>
> **✅ Instead: a split whose children are ALL folded across its own axis shrink-wraps.** Fold Rooms `[<]` and Self `[<]` → every child is strip-wide → **the column shrink-wraps and the `2` goes back to the message stream. And the hole closes with it** — two strips stacked in a strip-wide column leave **no leftover space at all**. **→ the case Joe actually hit produces NO hole under this rule.**
>
> **It costs NOTHING NEW:** no descriptor field (**derived from the children** — a stored flag would go stale the instant a child unfolds, **D-067 in miniature, for the second time in one arc**) · no name, no chrome, splits stay invisible · and the mechanic is **byte-for-byte the one a folded leaf already uses** (`flex: 0 0 auto`, siblings absorb). **One rule, two node types.**
>
> **⚠️ And it deliberately does NOT fire on mixed folds** (`[<]` + `[v]`): the column stays wide and a hole opens. **Kept.** *The user asked for two different things and gets a column that fits neither. **No magic, no guessing what they meant.***

**③ Holes are LEGAL, PAINTED, and INERT (§4.5).** **§7.1's *"no holes, rectangles only"* is AMENDED — by the man who accepted it on 2026-07-12, in the open, on 2026-07-13.** Now: **rectangles only; holes are legal and are painted as a system area.** Mechanically the hole is **flex leftover space inside a split, not an element** → **the raster is a BACKGROUND on the split container. Zero new DOM, one skin rule.** It ships **provisional** — *you cannot tune a raster under holes you have not seen*.

> ### 🔒 **AND THE LOCK THAT MATTERS FOR M-RP7.4: A HOLE IS INERT. IT IS NOT A DROP TARGET.** D-116 says a target tile is an **ADDRESS**; **a hole has no address.** Want a tile there? **Drop on the EDGE of the tile above it.** ***If we let people drop into holes we have quietly built free 2-D placement and retired the tree*** — which means retiring D-103's descriptor, not extending it.
>
> **⚠️ D-116 IS NOT WEAKENED BY ANY OF THIS.** §2 argued *"in a space-filling tree there is no empty space to drop into"* — now only *mostly* true. **But D-116's ground is Joe's constraint (*"never mixing or joining"*), NOT the geometry.** ***Correct the rhetoric; do not touch the decision.*** *(And §7.1's lattice refusal survives untouched for the same reason: **a lattice lets you PUT things in holes; we merely let holes EXIST.**)*

### ⚠️ NO `D` WAS LOCKED, ON JOE'S WORD

Joe: ***"your recomm (c). honestly i have to see it in practice."***

**D-117 was drafted at the Phase-0 walk and is now dead without ever having been written.** And **the replacement is NOT being locked either.** Phase-0 v1.3 carries the design — **that is what a Phase-0 is for.** `DECISIONS.md` carries **what Joe has decided**, and **he has told us he has not.**

> ***A `D` locked for a design its author has said he needs to see first is not a decision. It is a prediction wearing a decision's clothes.*** **→ the fold `D` enters the record after M-RP7.1b ships and Joe has looked at it. D-116 stands alone.**

### 🔑 Two grounded finds that changed the runbook before it was written

**① `migrate` DOES NOT EXIST.** Grepped `ui/**`: the word appears **only in comments** (`types.ts:16-17`, `layout-default.ts:63`). **`version` has been bumped TWICE and there has never been a migrate FUNCTION at all.** → **M-RP7.1b CREATES it, it does not extend it.** *A path described in three documents and implemented in none is not a path; it is a plan.*

**② ⚠️ A LAYOUT CAN ALREADY BE ON DISK — and the Phase-0 said the opposite by accident.** §12 stated *"nothing writes `session.layout`"*, which is **true** — and it was **read as "nothing writes a layout"**, which is **false**. **`app_client.svelte:227` has been persisting layouts inside NAMED UI STATES since M-RP6.1k.** Measured on disk 2026-07-13: `named: {}` — **empty. So migration is free TODAY — BY LUCK, NOT BY DESIGN.** **One click on the diskette lands a `v2` tree with `collapsed` booleans on disk.**

> **→ The answer is not "don't click the diskette." The answer is: WRITE THE REAL MIGRATE.** `v2 → v3`: for each `collapsed: true` leaf, read the parent's `dir` and write the explicit direction — **the old derived rule, made honest.** **~10 lines. And its DoD is that it is EXERCISED in vitest against hand-built `v2` trees under BOTH parent kinds** — **fed, not asserted** (N-091, applied to **the one branch in this codebase that has never once run**).

### The arc, amended (§11)

**✅ M-RP7.1** (CLOSED) · **🟢 M-RP7.1b** (this) · M-RP7.2 — splitter resize on the seam · M-RP7.3 — the mutation algebra (pure) · M-RP7.4 — drag to dock: grip, edge bands · M-RP7.5 — the session layout feeder · **M-RP7.6 — the grid lock: freeze arrangement, keep function** (NEW) · **M-RP7.7 — node app inherits the frame + grid** (was 7.6).

**🔒 M-RP7.1b ships §4.1 AND §4.4 TOGETHER, never apart.** **Two buttons without the shrink-wrap is a thin strip beside a huge hole — STRICTLY WORSE than what ships today.** ***Shipping the disease and the cure a week apart is how a bad appearance gets defended.***

### ⚠️ M-RP7.6 — the grid lock: Joe's 3rd idea, and it CANNOT ship yet

Joe wants a 4th bottom-shelf face that **freezes arrangement while leaving function untouched** — so ordinary use cannot accidentally re-arrange the grid. **Right, and deferred to the END of the arc, because TODAY IT WOULD GUARD NOTHING:** drag does not exist (7.4), resize does not exist (7.2), and **a lock over one verb is a button whose whole meaning is a promise** — the painted-dead chrome this project keeps refusing (J-500 / 6.1j).

**Three costs, grepped not guessed:**
1. **It is the FIRST STATEFUL SHELF FACE.** `shelf-face` has **`active`** (roving) and **`disabled`** (guard) and **NO pressed/toggle concept.** `aria-pressed` is a **real change to a shipped `core`**.
2. **`locked` wants to live in `session`** — where **Rust writes `geometry`** and **the frontend writes `layout`**. **N-107 one level deeper: that object must be merged PER-KEY, never replaced.** → **it lands AFTER M-RP7.5.**
3. **"Lock the top shelf too" locks an EMPTY BOX today** — `app_client.svelte:277` mounts it `items={[]}` and the skin collapses it to height 0. **There is no pinning verb.** → the top shelf joins the day favourites exist.

---

## Entry J-514 — M-RP7.1 — the tile frame: stripe, grip, fold CLOSED — and the tile that can fold opened a hole the chapter said could not exist

**M-RP7.1 — the tile frame: stripe, grip, fold ✅ CLOSED.** One code-only commit [Clair]: **`4c2f886`** (12 files, +433/−93). Design was locked at the dock-engine Phase-0 walk (`docs/xgen-dock-engine-phase0.md` v1.2); this is the implementation, its verification, and **one finding that cost the arc a new milestone before the next one started.**

**Renderer B has a frame.** `region-tile` — the **35th `core`** (`region-shell` 32nd · `shelf` 33rd · `shelf-face` 34th, Joe-locked J-508) — now frames every leaf: a title **stripe** `[move-grip · title · fold]`, a **body slot** (which is now the scroller, ex-`.region-leaf`), and the **reused `status-bar` SE clip-path triangle**. The **eight `Section` roots are unwrapped** — no widget draws its own title any more, which is Joe's *"group containers will be contained, but not as a main form"* arriving as a concrete edit in eight files (Phase-0 §5). `collapsed?: boolean` entered the leaf; `version: 2`; **migrate is a no-op.** The move grip and the resize triangle are **painted and DEAD** (`aria-hidden`, no handler, no role, no cursor — they carry no claim that later becomes false); their dischargers are named — **move → M-RP7.4 — drag to dock: grip, edge bands**, **resize → M-RP7.2 — splitter resize on the seam**.

**🔑 THE FINDING — FOLD OPENED A HOLE, AND THE CHAPTER HAD ALREADY PROMISED IT COULDN'T.** Phase-0 §4.1, one session old, states: *"No hole is ever created: the tile still fills its parent's cross-axis."* **That sentence is true about the cross axis and silent about the main one.** Fold two leaves in a `col` split with no expanded sibling left and **the split under-fills: a hole opens.** It is not hypothetical and it is not a future risk — **it is in the shipped build**, and Joe found it by *looking at the screen*, which is the only place it was ever going to be found. **Nobody wrote the all-siblings-folded case down, because the argument that produced §4.1 was a geometric one about a single tile, and a single tile cannot see its siblings.** *(A proof about one node is not a proof about the tree. The fifth member of the family — N-091 · N-097 · N-099 · N-109 · this: **a verified claim is only ever as wide as the case that was actually run.**)*

**→ The consequence is a new milestone, not a patch: `M-RP7.1b — the fold axis becomes the user's choice; splits shrink-wrap; the hole gets a floor`.** Design locked separately at **J-515**; it is the next thing built.

**⚠️ D-116 LOCKED. D-117 DELIBERATELY NOT LOCKED.** D-116 (*the dock rearranges; it never joins — a target tile is an ADDRESS, not a container; no centre drop-zone, no tabs, no docked/undocked mode*) stands on Joe's own words and nothing this session touched it. **D-117 as drafted — `collapsed` a boolean, the fold axis DERIVED from the parent split — was superseded by Joe before it was ever written to `DECISIONS.md`** (see J-515), and Joe's word on the replacement was: ***"honestly i have to see it in practice."*** → **No fold decision enters the record until it has been built and looked at.** *A `D` locked for a design its author has told you he needs to see first is not a decision; it is a prediction wearing a decision's clothes.*

**MEASURED — Chat re-drove every leg on the real client (Rule 5); no number below was taken on report:**
- Client registry **67** — **quiescent, EMPTY store, NO SELECTION.**
- **⚠️ N-108 EXTENDED — THE REGISTRY BREATHES WITH THE *SELECTION* STATE, NOT ONLY THE STORE.** Measured in one session: **67** (no selection) → **71** (selection active) → **65** (selection active + self folded). N-105 said *assert quiescence before you count*; N-108 said *a baseline that depends on a data file can be wrong on a machine where nothing is wrong*. **This is the third axis: a baseline that does not state its SELECTION state is unreadable.** The inspector renders rows only when the bus is fed, and a folded tile does not render its body at all — so the same healthy client honestly reports three different numbers. **→ every future baseline states store state AND selection state, or it is not a baseline.**
- `cargo test` **1517/0/62 — IDENTICAL to baseline**, which *proves* no Rust landed rather than asserting it. `npm test` **43**. `vite build` **168**. Sampler catalogue **328** (unchanged by scope).
- Split ratios **`[1,2,7,2]` exact at 1242px and 1252px** — a fifth and sixth distinct width.

**⚠️ FILED, NOT FIXED — a real library-wide defect, measured here and deliberately left alone.** **`mergeClasses` does not dedupe**, and `use:envelope` already stamps the type-class from `name` (N-023). **So any component that ALSO writes a literal `class="X"` renders `class="X X"`.** Five registered elements do it today: `region-shell` · `self-panel` · `inspector-panel` · `combobox` · `plugin-list`. **Clair fixed `region-tile` only — correctly; that was her scope.** The sweep (or making `mergeClasses` dedupe, which is a **`$common` base change**) is **its own milestone, NEVER a rider** — the same rule that kept the `dialog` footer-snippet slot out of M-RP6.1k. *A cross-cutting fix smuggled into a component milestone makes that milestone's registry delta unreadable, and the registry delta is the only thing proving the milestone did what it said.*

**TOOLING — measured this session, not folklore. These are new and they cost real time:**
- **⚠️ The CDP bridge WRAPS getters:** `get(id)` returns `{type, state}`. Read **`get(id).state.foo`** — reading `get(id).foo` returns **all nulls, which looks exactly like a broken build.** It isn't.
- Tile class names are `region-tile-stripe` / `-move` / `-title` / `-fold` / `-body` / `-resize` (**not** `-strip` / `-grip`).
- **⚠️ An attribute selector with an unquoted `#` throws a bare `EVAL ERROR`** — `[data-debug-id=region-tile#region-spaces]` is invalid. Quote it, or iterate `querySelectorAll('[data-debug-id]')` and compare. **N-110's family again: assert the subject is READABLE before asserting anything about it.**
- Escaped double-quotes inside `-Expression` break the harness (it mis-binds to `-Ordinal` and throws a type error). Use single quotes inside the JS, or `String.fromCharCode(39)`.
- `npm test` lives in **`ui/sampler`** (`vitest run`) — not `ui/` (no `package.json`) and not `ui/client` (no test script). `vite build` runs in **`ui/client`**.
- **⚠️ `cargo test` EXCEEDS THE MCP TIMEOUT (<45 s). Run it DETACHED** (`Start-Process cmd /c … > log 2>&1 -PassThru`) and poll the PID in **separate short calls** — a long `Start-Sleep` gets the shell killed **and takes the detached run with it.** When that happened here, the truncated log read **1195 / 0 / 60** — plausible, complete-looking, and **wrong**. Only the **missing final `test result:` summary** gave it away. ***A killed detached run leaves a MEASUREMENT-SHAPED ARTIFACT*** — the N-099 shape at the process level.

**Records this entry closes:** `ui/docs/xgen-region-dock-model.md` → **v2.0** · `xgen-widget-tier.md` (W-13's *"may collapse"* finally has a mechanism; a region widget's root is no longer a titled `Section`) · `xgen-ui-components.md` (`region-tile` = 35th) · `xgen-ui-notes.md` · `tasks/M_RP7_1_TILE_FRAME.md` → **COMPLETED** · `docs/ROADMAP.md`. **No new `core` beyond `region-tile`. No Rust.**

**🟢 NEXT-ACTIVE = M-RP7.1b — the fold axis becomes the user's choice; splits shrink-wrap; the hole gets a floor.**

---

## Entry J-513 — M-RP6.1l CLOSED: the plugin list ships — and the milestone's real finding is that it should not have been built yet

**M-RP6.1l ✅ CLOSED.** One code-only commit [Clair]: **`1dc5849`** (6 files, +272/−17). Design was locked design-only at **J-512**; this is the implementation, its verification, and **two corrections that matter more than the milestone**. **The 6.1j countdown is discharged — no shelf face in the app is disabled.**

**What shipped.** The **first plugin registry in the project** — `ui/common/lib/plugins/registry.ts`, a frontend artefact carrying **D-112's three axes (`host` · `delivery` · `surface`) in code for the first time**. `widgetRegistry` now **DERIVES** from it: *a widget is in the grid because it is a registered plugin with `surface: region`.* On top of it, a **read-only pane** (the 5th widget, `kind: system`, `surface: none`) in a shell-local modal, opened by the gear. **Three honest rows, `[system]` badge, no faked verbs.** The grounding held all the way through: **there was no registry to enumerate, so the milestone created the first one** — it did not project one out of Rust (J-499's rule; the inverse of D-114's geometry).

---

## ⚠️ CORRECTION 1 — A COUNTDOWN NAMES *WHO* FLIPS A FACE, NEVER *WHEN*

**This milestone was built too early, and the record is why.** 6.1j wrote: *“no face is enabled before its command exists, **and no milestone closes leaving its own face disabled** — 6.1k flips `diskette`/`load`, 6.1l flips `gear`.”*

**Those are two different rules wearing one sentence.** The first is a real guard: **never enable a control that resolves to nothing.** The second is a **schedule** — and it was smuggled in beside the guard, unexamined, and it **pulled a milestone forward because a face was waiting**, not because the product needed it. Joe, plainly: *“I honestly thought we would grey the widget manager till we have a working grid.”* **He was right, and the schedule was Chat's.**

> ### **A disabled face with a NAMED OWNER is honest indefinitely. A countdown names WHO discharges it — it must never name WHEN, because that turns a guard into a deadline and lets the shelf drive the roadmap.**

**Kept, because it is not nothing:** the derive. `widgetRegistry ← CLIENT_PLUGINS` is the seam every future region drops into, and it is verified. **The sequencing that produced it is not something to be pleased about.**

---

## ⚠️ CORRECTION 2 — D6 IS SUPERSEDED (Joe, 2026-07-12). A GREYED BUTTON IS NOT ONE THING.

J-512's **D6** refused Remove/Disable/Launch/Settings outright (*“no verb exists → the absent slot ships ABSENT, not faked”*). **Too blunt — it collapsed two controls that look identical on screen and are opposites:**

- **Grey because the verb was never built** → a **dead control**. It lies by implying the capability exists.
- **Grey because the plugin's OWN DESCRIPTOR says so** → **not a missing feature — that is W-13, RENDERED.** A disabled **Remove** on a `[system]` row **is the information.** Ch6 §6.8.5 drew exactly that, and it was right.

> ### **🔒 THE RULE (Joe-locked): every button's state is DERIVED FROM THE DESCRIPTOR, never hardcoded — and a control is disabled only for a reason that is TRUE OF THAT PLUGIN and legible to the user.**
> **Remove** → disabled ⇸ `kind === 'system'` (W-13) · confirm-on-click for `custom` (S-6) · **Disable** → disabled ⇸ `kind === 'system'` · **Launch** → rendered **only iff** `surface === 'window'` (element-absent, not greyed — Ch6 §6.8.5's own rule) · **Settings** → disabled ⇸ `!hasSettings`.
>
> **⚠️ And `Settings` is the honest-looking trap:** today it would be grey because **we never built a settings mechanism**, not because these plugins have none. **Same grey, different truth.** → `hasSettings` goes **on the descriptor**, and **the milestone that makes it true owes the button a target, written into its own DoD** (N-109 discipline).

**→ M-RP6.1m (the action row) is FILED and ⏸️ POSTPONED, not next.** Count the feeders: **Settings** — no settings mechanism · **Launch** — no `window` plugin · **Disable** — no disable verb · **Remove** — no `custom` plugin to remove. **Four buttons, ZERO live sources.** That is *precisely* the 6.1k finding (*five of six §4.5 keys have no feeder*) and it takes the same answer: **each control lands with the milestone that creates its source.** *The design is recorded so nobody re-derives it; the build waits.*

---

## 🔍 SURFACED IN SEARCH, NOT IN CODE — two records nobody was reading

**⚠️ N-007 (`ui/docs/xgen-ui-notes.md`, 2026-06-02) — filed, marked *“graduates into Ch6 + the module-framework milestone”*, and IT NEVER GRADUATED.** *“Every module needs a UI representation in both apps — including **system** modules”*: install / enable / disable / select · **status / health** · **a warnings home** (the vanilla EventStore's *“storage heavy — install the engine module”* operator contract *“needs a UI home so it isn't only a log line”*). Written as *“a first-class question, not an afterthought.”* **Neither the taxonomy Phase-0 nor J-512 cited it.** **The shipped plugin list satisfies roughly a third of it.** *A note that files an obligation and is never graduated is a note the project has decided to forget slowly.*

**⚠️ THE SETTINGS-MECHANISM COLLISION — FILED, DELIBERATELY NOT DECIDED.** Two rival answers for *what a plugin's settings UI actually is*, and they have never been put side by side: **(A)** Ch6 §6.8.2/§6.8.5 — `settings_schema`, a JSON-Schema fragment ***“rendered automatically in the module list settings panel”*** (zero lines exist); **(B)** surfaces §3.2 + widget-tier — ***“Settings is a widget, and it hosts other widgets as content”***, i.e. the plugin supplies **its own component** — **and B is SHIPPED** (`substitutions-editor`, M-RP4.3, the first widget ever built). **§6.8 predates what was built, and it loses again — the same species the taxonomy Phase-0 found.** **D-112 fixed classification and placement; it never asked how a plugin's settings get DRAWN.** **Joe: these are visions waiting for real context — it does not need to be yet another widget system.** **BINDING: nothing is built toward either until the grid works. No milestone may quietly pick one.**

---

## MEASURED — Chat re-drove every leg (Rule 5); not one number taken on report

**Every one of Clair's numbers reproduced exactly** — the second handback this arc that did (J-501 the other), *recorded because the ones that did not are also recorded*.

| leg | measured |
|---|---|
| baseline | **67** · `count === unique` · **QUIESCENT** (`menuish: []`) · **EMPTY STORE** — verified **on disk**: `named: {}` (N-108: a baseline that depends on a data file must state which data it stood on) |
| the +12 from 55 | **enumerated, not derived**: 9 × `label#plugin-list__*` + `plugin-list#plugin-list` + `dialog#plugins` + `button#plugins__close` |
| gear | **enabled**, dispatches `widget.manager` → **`:modal` true** (on `:modal`, never the `open` attribute — J-496) |
| getter G | `{count:3, systemCount:3, customCount:0}` |
| mount/close | **67 open === 67 closed** (always-mounted dialog); close → **exact 67**, `open:false` reconciled (the C1 write-back holding) |
| **painted DOM** | **Inspector Panel · Plugin List · Self Panel** — alphabetical (D8), 3 × `[system]`, axes `client · compiled · region\|none`; **`btns: 0`** inside the pane (read-only, proven on the render) |
| **the derive (D2)** | `leafCount:8`, `droppedCount:0`, `unsupportedCount:0`, depth 3; **`self-panel#region-self` + `inspector-panel#region-inspector` still REAL widgets** (the derive did not silently drop them back to placeholders); split ratios **`[1,2,7,2]` EXACT at winW 1727 — a FOURTH distinct width**; `docNoScroll` true |
| accent | 29 elements sampled, **`readable: true`** (the subject was SEEN — not a `null === null` phantom, N-099), **0 diffs** under a magenta `--accent`/`--accent2` inject |
| static (apps DOWN) | **`cargo test` 1517/0/62 — IDENTICAL to baseline** (summed programmatically), exit 0 → ***proves*** the no-Rust claim · `vite build` **168** (from 165) · `npm test` **41** · scope: `git show --stat 1dc5849` = **zero `ui/core/**`, zero `ui/sampler/**`, no Rust** → **sampler catalogue 328 unchanged, by scope** |

**🔑 Client registry baseline is now 67 — quiescent, EMPTY store. The next milestone must cite that, and must say which store state it counted in.**

---

## Deviations — flagged, not absorbed (Rule 6). Both are CHAT'S.

**① The runbook's V2 literal was WRONG, and Clair caught it against the source.** V2 demanded `aria-disabled="false"` on the enabled gear. **`shelf-face.svelte:74` renders `aria-disabled={disabled || undefined}`** — the attribute is **absent** when enabled and `"true"` only when disabled; **it is never `"false"`.** Her measured `ariaDisabled: null` + `nativeDisabled: false` is the correct enabled state, **re-confirmed by Chat against the live DOM**. *Same species as J-501's DM-square slip: a runbook literal that grounding against the source corrects.* The substantive leg (gear → `:modal` dialog) is unaffected.

**② ⚠️ CHAT'S OWN VERIFY DEFECT — A SELECTOR THAT CANNOT SEE ITS SUBJECT REPORTS A CLEAN-LOOKING NOTHING.** Chat twice wrote CDP selectors against **`#app-shelf-bottom`** and **`dialog#plugins`** — but **the components carry `data-debug-id`, NOT `id`**. The first returned **`faces: []`** (an empty array that *looks like a measurement*); the second **threw a bare `EVAL ERROR: Uncaught`**. **Neither entered anything** — both were grounded against the real DOM and re-run, and the throw was treated as **inconclusive, not a failure** (J-496). → **N-110: this is the N-099 family one layer down** — N-099 was *a check whose subject is absent still returns an answer*; this is *a **selector** whose subject is absent does the same, and returns the flattering shape.* **Assert the subject is readable BEFORE you assert anything about it** (the same `readable:true` guard that made the accent leg trustworthy is what the shelf leg lacked). **Harness debt: `cdp-debug.ps1`'s doc must say the registry keys on `data-debug-id`, because the id-selector reflex is going to recur.**

---

## Records

`tasks/M_RP6_1L_PLUGIN_LIST.md` → **COMPLETED** · CLAUDE.md PLAY · `docs/ROADMAP.md` **v4.83** · `ui/docs/xgen-ui-components.md` (**`plugin-list` = the 5th widget**, not a `core` cell; **sampler catalogue unchanged 328**) · `ui/docs/xgen-ui-notes.md` **+N-110**. **No new D. No new `core`.**

**Filed, unchanged:** **M-RP6.1m** ⏸️ (the action row — four buttons, zero feeders) · **M-RP-SETTINGS** · **M-RP-PLUGINS-NODE** · the **`dialog` footer-snippet slot** · **N-007's ungraduated obligation** · the **settings-mechanism collision** · M-RP-ROVING · M-RP-FOCUS · top-shelf pinning · M-RP6.6 · the read-marker gap · `temperature-indicator` ⏸️.

> ### 🟢 **NEXT-ACTIVE = M-RP6.2 — R1 Spaces + R2 Rooms on real `KnownSpace`. BACK TO THE GRID.**
> *The plugin list, its action row, Settings, and the node-module verb all queue behind working regions. **The shelf does not drive the roadmap.***

---

## Entry J-512 — M-RP6.1l DESIGN LOCKED: the plugin list — and the grounding found there is no registry to enumerate

**Design/records-only. NO CODE. Registry unchanged (55, quiescent, empty store).** Joe granted autonomy (*"you have an autonomy in this part. do as you propose"*) and the proposal was adopted as-is. Runbook `tasks/M_RP6_1L_PLUGIN_LIST.md` (ACTIVE) handed to Clair. **No new D — this is a D-112 extension.**

---

## 🔑 THE FINDING: THE PLUGIN LIST HAS NOTHING TO LIST, AND SAYING SO IS THE MILESTONE

The walk opened on the one question the handoff insisted be grounded rather than guessed: **what does the list actually LIST today?** Grepped, not remembered:

- **`xgen-common/src/module.rs`** ships `Descriptor { kind_id, impl_id, name, assurance }` — **no `host`, no `delivery`, no `surface` field.** The **vocabulary** is shipped; the **fields are not** (the taxonomy Phase-0 said so in its own honest-limit note, and it is still true). Its **only** consumer is `xgen-core/src/dag/store.rs` (`STORAGE_ENGINE_KIND_ID` + the `InMemoryEventStore` descriptor) — **node-side, and no Tauri verb exposes it.**
- **Zero `xgen-module.json` anywhere.** No manifest loader, no `modules/` scan, no local WebSocket server in the client.
- The **Auth Module registry** is node-side (`delivery: service`); `temperature.rs` is node-side and `NoOp`. **The client has a verb for none of them.**
- `ui/common/lib/components/widgets/` holds **4 files** — but **only TWO are mounted in the client**: `widgetRegistry` maps 8 region ids → 6 × `RegionPlaceholder`, `self` → `SelfPanel`, `inspector` → `InspectorPanel`. **`substitutions-editor` and `entity-context-menu` are imported NOWHERE in `ui/client`** (sampler-only).
- **W-13's `kind: system` is a spec word with ZERO code.** No widget declares a kind, a name, a version, or a descriptor.

> ### **So M-RP6.1l does not ENUMERATE a registry. It CREATES the first one — in TS, in the frontend — and it lists exactly what is real.**
>
> ***A list that fakes a universal registry is worse than three honest rows.*** **Third instance of one shape:** J-500's *"there is no resident"* · J-502's temperature find · this. **A UI milestone cannot manufacture a source that does not exist.**

---

## The lock

**D1 — the registry becomes an ARTEFACT.** New `ui/common/lib/plugins/registry.ts`: a typed `PluginDescriptor` + `CLIENT_PLUGINS`. **This is D-112's three axes (`host` · `delivery` · `surface`) in code for the first time.** **No Rust** — J-499's rule applied, not excepted: *Rust owns what only Rust can do*, and a plugin descriptor is a type **the webview owns**. (Contrast D-114's geometry: a type **Rust** owns and the webview cannot.)

**D2 — `widgetRegistry` is DERIVED from it.** A widget is in the grid **because it is a registered plugin with `surface: region`**. One source, two readers (the N-096 shape). The 6 unbuilt regions keep `RegionPlaceholder` — **a placeholder is scaffolding, not a plugin, and it is not listed.**

**D3 — the pane's own surface is `none`, and that is not a dodge.** Per surfaces §3.2 + D-112: **Settings** takes `surface: window`; the plugin list is **content inside it** and therefore **spends no surface**. Which means *any* host can mount it **without the pane lying about what it is**.

**🔒 D4 — THE SCOPE FORK, and it was the real decision: SETTINGS DOES NOT EXIST, and 6.1l does not build it.** D-112 says the gear opens a pane *inside Settings' structure* — but there is **no Settings window**. The pane ships with a **shell-local modal host** (the `about-dialog` / `uistate-*-dialog` precedent) as its **first** entry point; **`M-RP-SETTINGS` becomes its SECOND mount** — which is *literally* **S-2's "one component, two mounts"**, so **nothing built here is thrown away**. A grid tile was **rejected by D-112 §9** (*dock the plugin manager into a 200px column, then remove the widget that removes widgets*); a real second Tauri window now is a **frame arc** (own Vite entry, chrome, registry, CDP target, geometry) that would bury the visible part behind plumbing — against Joe's standing brief.

**D5 — THREE honest rows:** `self-panel` · `inspector-panel` · `plugin-list` (**it lists itself**). **NOT** `substitutions-editor` / `entity-context-menu` — **the client never instantiates them**, and registering an unmounted plugin is **the unfed-branch shape (N-091)**. They enter the registry **at the milestone that mounts them**.

**D6 — READ-ONLY rows. No Remove / Disable / Launch / Settings buttons.** There is **no remove verb, no disable verb, no launch verb, no settings schema**. A permanently-disabled control with **no countdown milestone behind it** is exactly what 6.1j forbade — and **J-500's precedent is explicit: the absent slot ships ABSENT, not faked.** What ships is the **`[system]` badge**, which is **Ch6 §6.8.5's own drawing = W-13 made visible**. **S-6 says destruction lives *here*; it does not say it lives here *now*.**

**⚠️ D7 — NO W-8 PHASE-LIMIT NOTE ANYWHERE IN THE PANE. An N-109 pre-empt, written into the runbook as binding.** A read-only list of what is loaded is **not a false statement about anything** → **there is nothing to sweep at close.** And if any leg finds it needs a disclosure, **its removal goes into the DoD of the leg that lifts the limit, in the same edit that adds it.** *(N-109 cost 4 lines to fix and was the only defect in 6.1k that would have reached a human. The cheapest way to honour it is to ship no claim that can go stale.)*

**D9 — the dialog uses the STOCK core footer.** If the `:has()` suppression hack from 6.1k is reached for again, **that is the second independent recurrence** and the `dialog` footer-snippet-slot extraction stops being optional — **its own milestone, never a rider** (a `core` change inside a shell milestone is what makes a registry delta unreadable).

---

## Verify contract set for the build (Rule 5 / N-105 / N-108)

**Baseline: client registry 55 — quiescent, EMPTY STORE, and the milestone must SAY SO.** ⚠️ **The `plugin-list` rows register at MOUNT, not on open** (a closed `<dialog>` is `display:none`, **not unmounted**) → **the baseline moves the moment Leg A lands, and the new one must be MEASURED, not derived.** `cargo test` **MUST stay 1517/0/62 IDENTICAL** — the inverse of 6.1k's leg: *identical proves no Rust landed*. Sampler catalogue **328 unchanged, by scope**. Legs, visible-first per Joe's standing brief: **A** pane + dialog + `commandTable` + **the gear flips** (hand back for Joe's eyes) · **B** the derive · **C** CDP verify → close.

**Its DoD discharges the 6.1j countdown: after 6.1l, no face in the app is disabled.**

---

## Filed, NOT built

**`M-RP-SETTINGS`** (Settings as a `surface: window` plugin — the pane's second mount, and `substitutions-editor`'s first client mount) · **`M-RP-PLUGINS-NODE`** (a read verb exposing `host: node` plugins to the client — **Rust/protocol, the M-RP6.6 shape**; until it lands the list is honestly client-only) · the **`dialog` footer-snippet slot** · M-RP-ROVING · M-RP-FOCUS · top-shelf pinning (surfaces §6 ④) · M-RP6.6 client resident · the read-marker gap · `temperature-indicator` ⏸️.

**Records moved:** `tasks/M_RP6_1L_PLUGIN_LIST.md` (new, ACTIVE) · `CLAUDE.md` (PLAY) · `docs/ROADMAP.md` (v4.82). **No new D. No code.**

---

## Entry J-511 — M-RP6.1k CLOSED: the UI-state store ships — and the milestone's only user-facing defect was an honesty note that had stopped being honest

**M-RP6.1k ✅ CLOSED.** Two code-only commits [Clair]: **`8902efa`** (the milestone — 10 files, +892/−17) + **`fdccdb2`** (the stale-note fix — 3 files, +6/−8). **The D-074 lane held for the third milestone running.** Design was locked design-only at **J-510** (**D-114** + **D-115**); this is the implementation and its verification.

**What shipped.** The client's **UI-state store** — one file, `xgen-client_uistate.json`, two lifecycles (session + **named** states), holding the **grid layout** and the **window geometry**. **M-RP-WINSTATE is absorbed and gone.** The bottom shelf's `diskette` and `load` faces are **enabled**, and their commands do real, persistent work. **`gear` stays disabled — the countdown continues into 6.1l.**

**Legs, in the order Joe asked for them — VISIBLE FIRST.** **A** the two dialogs + both commands into `commandTable` + both faces flipped, against an **in-memory** store (handed back for Joe's eyes, then reshaped on his feedback: comboboxes on both dialogs) · **B** Rust `get_ui_state`/`set_ui_state` + the file + the `loadLayout()` body swap + **N-095 exercised** · **C** window geometry, physical px, save-on-move + `CloseRequested`, restore **before `show()`**, work-area clamp · **D** named states carry the rect through the same clamp → close.

---

## 🔑 THE ENTRY'S REAL FINDING: A PHASE-LIMIT NOTE IS A COUNTDOWN TOO

Leg A shipped a **correct W-8 honesty note**, painted in the Save dialog:

> *“Session-only for now — not yet written to disk.”*

**True at Leg A, and exactly the posture this project demands** — the `substitutions-editor` / `self-panel` precedent: *render the phase-limit honestly rather than fake the capability.*

**Then Legs B, C and D shipped persistence, and nobody swept the note.** The milestone arrived at close-out with a store that survives a relaunch, geometry that saves and clamps, and named states that carry a window rect — **while the app told the user their workspace was not being saved.** Chat caught it in the **painted DOM** during the Rule-5 re-drive, and read the two named states off the disk in the same minute to prove the contradiction rather than argue it.

> ### **A STALE HONESTY NOTE IS STILL A FALSE STATEMENT — and it is WORSE than a missing one, because it was written by someone being careful, so the next reader trusts it.**
>
> **The milestone's own discipline already contained the rule and failed to generalise it.** 6.1j bound: ***the disabled face is a COUNTDOWN, not a resting state*** — no milestone closes leaving its own face disabled. **A W-8 disclosure is the same object.** The face got swept **because the DoD named it**. The note got missed **because nothing named it**.
>
> ### **RULE (N-109): no milestone closes leaving its own UI asserting a phase-limit it has since removed. Sweep the CLAIMS, not just the code. And when a leg ships a disclosure, write its REMOVAL into the DoD of the leg that lifts the limit.**

**Fourth in a family that keeps paying:** N-091 (*“verified” is only as wide as the legs you ran*) · N-097 (a constant `false` stranded a shipped skin rule) · N-099 (a leg that reported green by comparing `null === null`) · and now this. **All four are one species: an earlier state's truth left standing after a later state killed it.**

**Cost of the fix: 4 lines.** *The cheapest defect in the milestone was the only one that would have reached a human.* Fixed in `fdccdb2`; the sweep for surviving Leg-A claims was **re-driven by Chat, not accepted** — the two remaining `in-memory` comments describe the **corrupt-fallback runtime state** (N-095) and are accurate.

---

## The decisions, realised in code — checked against the source, not the report

**D-114 — ONE store, and the carve-out is literal.** `get_ui_state -> String` / `set_ui_state(json: String)`: a **raw-string opaque round-trip**. **There are zero descriptor types in the Rust crate** — the only `widgetId` anywhere in `xgen-client` is a raw string literal *inside* `write_then_read_round_trips_verbatim`, which is the **proof of opacity, not a breach of it**. `WindowGeometry {x, y, width, height, maximized}` is the **only** typed part, reached by a separate typed read-modify-write on `session.geometry`.

> ***Rust owns what only Rust can do, and stays blind to what the webview owns.*** **Held, exactly as locked.**

**D-115 — physical px, clamp, N-095 relocated.** Corroborated independently at verify: 1475 CSS × DPR 1.25 = 1844 inner-physical against 1859 outer on disk (the delta is the window frame). Clamp exercised by writing an off-screen rect and launching. **`tauri-plugin-window-state` was not taken.**

**RESERVE NOTHING — held, and visible on disk.** The store's top-level keys are **exactly** `version`, `session`, `named`, `active`. No `theme`, no `shelf`, no `collapsed`, no `room`. *Five of the six §4.5 keys still have no feeder, and the store does not pretend otherwise.* **D-101 untouched** — `clean_slate_config` still wipes config only; the UI-state store is the project's **first deliberately persistent user-facing state**, by design.

---

## Measured (Rule 5 — Chat re-drove every leg; not one number was taken on report)

| leg | result |
|---|---|
| client registry, **quiescent, empty store** | **55** (`count === unique === 55`) — **MEASURED, not derived** |
| **N-108 breathing** | **55 → save → 59 → delete → 55** — proven as a **transition in ONE session**, exact return, zero leaks |
| `cargo test --workspace` | **1517 / 0 / 62** (from **1507**; **+10**, the ten named tests) — summed programmatically, not hand-counted |
| `vite build` | **165 modules** · `npm test` **41** |
| sampler catalogue | **328 UNCHANGED** — grounded **by scope**: zero `ui/core/**`, zero `ui/sampler/**` across **both** commits |
| faces | `uistate.save` + `uistate.load` **`disabled:false`**; `widget.manager` **still `true`** |
| dialogs | **`:modal` true** — measured on `:modal`, never the `open` attribute (J-496) |
| geometry | split ratios **`[1,2,7,2]` exact at a THIRD distinct width** (1475 CSS); `docNoScroll`; leafCount 8, dropped 0 |
| **N-095 corrupt** | corrupt store → `DEFAULT_LAYOUT`, region-shell **present**, **no blank centre** — the J-499 30→21 failure **proven absent** |
| unknown-key | an injected `probeKey` **survived the app's own write-back** |
| **N-107 two-writer** | emptying `named` left **`session.geometry` byte-intact on disk** |
| skin / accent | zero component-local `<style>`; CSS in `skin.css` (N-090); accent-neutral |

**Two legs Chat ran that were in nobody's matrix:**

1. **Loading a named state with NO `geometry` key.** The disk had grown exactly such a state (`001`, saved before Leg C existed) — *real data outran the fixtures inside one session*. Result: window rect **unchanged**, region-shell present, registry exact. **Both guards held** (`if (s?.layout)` / `if (s?.geometry)`), and the J-499 blank-centre trap was **designed around and cited by name in the comment** before it was ever hit.
2. **Delete is two-step** — the button re-labels to **“Confirm delete”** before it destroys. **S-6's spirit (*destruction is deliberate, never one click*) honoured inside the dialog**, and it was never in the DoD.

**And the “static gates need the apps down” rule stopped being folklore:** `cargo test` with the client up fails in 15 s with `failed to remove file .../xgen-client.exe`. **The running app holds the binary.** Measured, so it never has to be re-litigated.

---

## Deviations (Rule 6 — flagged, not absorbed)

- **Shell-local `.svelte.ts` store**, not an inline `$state` in `app_client` as the runbook said. **The better call** — it is the exact seam Leg B swapped persistence into. Joe accepted.
- **Comboboxes on both dialogs** (Joe, mid-flight; `select` was offered for Load and declined).
- **Autofocus focus-catcher** (`tabindex="-1"` on `.uistate`) so `showModal()` does not land focus on the combobox and pop its dropdown. CDP-confirmed collapsed-on-open.
- **The core `dialog`'s single-button footer is suppressed** for the uistate dialogs via `:has()`, so Save/Delete/Cancel sit on one line. **Consequence, recorded rather than hidden: the core's `__close` button stays mounted-but-hidden and still registers.** → **FILED, NOT BUILT: a footer snippet slot on the `dialog` core** (removes the `:has()` hack *and* the hidden `__close`). **Correctly kept out of 6.1k** — a `core` change inside a shell milestone is exactly what makes a registry delta unreadable. **Its own milestone.**
- **`quit`-command geometry save — a genuine find, made in verification.** `app.exit` (Ctrl+Q / File→Exit) does **not** reliably fire `CloseRequested`, so the window-event saver would have **silently lost the final rect on that path**. The X-button path was always covered. *Found because someone drove the app, not because someone read the code.*

---

## Records + what's next

**+N-107** (two writers, one file → read-modify-write on both sides) · **+N-108** (the registry breathes with **store contents** — extends N-105; **6.1l's baseline is 55**) · **+N-109** (the stale phase-limit note). **No new D** (D-114/D-115 implemented as written). **No new `core` component** — the dialogs are shell-local, so the components registry and the sampler catalogue are untouched.

**NEXT-ACTIVE = M-RP6.1l — the widget manager / plugin list. Its DoD flips `gear`.** Under **D-112** it is **one pane, two entry points** (inside Settings' structure; the shelf gear opens the *same* pane, never a cut-down twin), it lists **built-ins as `[system]` with Remove disabled** (W-13), and it is **the home of destruction** (S-6). **No face is enabled before its command exists, and no milestone closes leaving its own face disabled** — `gear` is the last one standing.

**Still open, named so they stay out:** the `dialog` footer slot · session-**layout** auto-save (**no feeder until renderer B** — RESERVE NOTHING held) · top-shelf pinning (surfaces §6 ④, gates nothing) · M-RP-FOCUS · M-RP-ROVING · M-RP6.6 client resident · the read-marker protocol gap · M-RP7.x · M-RP8 · `temperature-indicator` ⏸️.

---

## Entry J-510 — M-RP6.1k design lock: two specs, one store — and "Rust never learns the shape" turns out to need a carve-out, not an exception

**Design-only. NO CODE.** Joe locked the walk in full ("you have an autonomy in this part — do as you propose"). Records moved: **D-114** + **D-115** · `ROADMAP.md` **v4.80** · `CLAUDE.md` PLAY · `ui/docs/xgen-region-dock-model.md` **v1.8** · `docs/xgen-widget-surfaces-phase0.md` **v1.5** · `ui/docs/xgen-ui-notes.md` **v0.81** (**N-106**) · runbook `tasks/M_RP6_1K_UISTATE_STORE.md` → Clair.

---

## ⚠️ THE COLLISION WAS REAL. IT WAS ALSO NOT A CONTRADICTION.

The kickoff brief named it and said: *do not assume; grep and reconcile.* Two specs described who persists the client's UI:

| | `xgen-region-dock-model.md` §9 | `xgen-widget-surfaces-phase0.md` §4 (Joe-locked J-503) |
|---|---|---|
| object | **layout only** | **UI state** (layout · geometry · shelf · collapsed · theme · room) |
| file | `xgen-client_layout.json` → `xgen-client_layouts.json` | `xgen-client_uistate.json` |
| verbs | `list` / `save` / `load` / `delete` / `rename_layout` (**5**) | one store, two lifecycles |
| when | auto **7.3** · named **7.6** | **6.1k** |

**🔑 They were never two objects. §9 is the EARLIER, NARROWER DRAFT** — written when the layout was the only thing anyone intended to persist, **before** window geometry, the shelf, or a named-workspace concept existed. And surfaces §8 **had already written the outcome down at J-507**: *"M-RP-WINSTATE stays ⏸️ → absorbed by the §4 UI-state store at M-RP6.1k."* The brief's instruction — **look first** — is the whole reason this took a grep instead of an argument.

> ### **A spec written earlier is a spec written with less of the system in view. Read it as a NARROWER TRUE thing, not a COMPETING FALSE one.**
> **And then ask which of its clauses were about SCOPE (they die) and which were about SUBSTANCE (they survive).** §9's identity + reconcile rules — **`widgetId` is durable · drop unknown ids · re-inject missing `system` widgets (W-13) · `version` + migrate on schema change** — are **exactly right**, and they simply become the **`layout` KEY's** rules **inside** the one store. Only the **filenames**, the **five verbs**, and *"the layout manager is itself a widget"* die. **Overturning §9 wholesale would have thrown away four correct rules along with two obsolete filenames.**

**🔒 D-114 — ONE store: `xgen-client_uistate.json`.** Two files = two lifecycles, two clamps, two reconciles, two migration paths **for one user-visible act**. Self-inflicted **D-067**.

---

## 🔑 THE ENTRY'S REAL FINDING: A RULE CAN BE RIGHT AND STILL NOT BE UNIFORM

J-499 D2 established — correctly, and at cost — that ***Rust persists an OPAQUE blob and never learns the node shape***: a `get_layout` command would have **duplicated the descriptor type in Rust**, the exact **D-067** drift this project exists to kill. Surfaces §4 inherited that rule **wholesale**.

**But §4.2's monitor-work-area clamp is MANDATORY — and only Rust can read a work area, or apply a window rect before the webview exists** (apply it later and the window visibly jumps).

**So the rule, applied uniformly, forbids the thing the same document requires.** *A clamp Rust cannot perform is not a clamp.*

**The resolution is not an exception to J-499. It is J-499 applied one level deeper:**

| half of the file | form | why |
|---|---|---|
| **`geometry`** | a **TYPED Rust struct** (`x,y,w,h,maximized`) | a type **Rust already owns** and the **webview cannot** |
| **everything else** | an **opaque `serde_json::Value`**, round-tripped verbatim | a type **the webview owns** and Rust must **never duplicate** |

> ### **RUST OWNS WHAT ONLY RUST CAN DO, AND STAYS BLIND TO WHAT THE WEBVIEW OWNS.**
> **The original rule was never "Rust must not have types." It was "Rust must not have a SECOND COPY of someone else's type."** *A rule quoted without asking which side of it each new thing falls on has stopped being a rule and become a superstition — and the tell is that it starts forbidding things nobody had a reason to forbid.*

**Free, and it is why the blob is the right shape rather than merely a permitted one:** an opaque value **round-trips UNKNOWN KEYS** → every future key is **additive with zero Rust change**, and a key written by a newer binary **survives** a read-write cycle through an older one.

**Verbs: `get_ui_state` / `set_ui_state`** — the shipped `get_substitutions` / `set_substitutions` shape. The frontend `loadLayout()` D2 seam swaps its **body**, never its call shape. *It was written that way at J-499 for exactly this moment, and it paid.*

---

## ⚠️ FIVE OF THE SIX Joe-LOCKED §4.5 KEYS HAVE NO LIVE SOURCE. GREPPED, NOT ASSUMED.

| §4.5 key | feeder in the shipped client? |
|---|---|
| **window geometry** | ✅ **YES** — resize grip + native decorations; the user moves it every session |
| **grid layout** | ⚠️ a **const** (`DEFAULT_LAYOUT`) — no user mutation until renderer B (7.1/7.2) |
| shelf favourites | ❌ pinning (surfaces §6 ④) is **still open**; the top strip mounts empty |
| collapsed / expanded | ❌ **ZERO `collapsible` props anywhere in `ui/client`** — the 8 leaves are non-collapsible `section`s |
| theme | ❌ `theme-*.css` **does not exist** (D-110) |
| last open space + room | ❌ no room selection until **M-RP6.2** |

**→ 6.1k ships `geometry` + `layout`, AND RESERVES NOTHING.**

> **An empty `theme: null` key is the `tabs`-branch / N-091 shape AT FILE SCALE.** *A key nothing writes is a key nobody has round-tripped* — and its wrongness will be discovered by the milestone that finally tries to use it. **§4.5 locked what BELONGS in a UI state; D-065 governs WHEN each one gets built. A design doc listing six keys is not a licence to emit six keys.**

---

## 🔒 D-115 — THE M-RP-WINSTATE CRITERION FIRED, AND NOBODY HAD TO RE-ARGUE IT

Written down at **J-498**, precisely so this moment would cost a grep instead of a debate: *"At kickoff, did the widget grid produce a persistent UI-state store? **YES → B** · **NO → A**."*

**It did. → B.** Own it. **`tauri-plugin-window-state` is NOT taken.** M-RP-WINSTATE → **⬛ SUPERSEDED, absorbed.**

**Unit settled: PHYSICAL px** (N-092b — **measured twice**, J-495 and J-498). **The decisive reason is not tidiness: a rect can only be compared to a monitor work area in the same unit, and Tauri's `work_area()` is physical. A logical rect makes the mandatory clamp UNIMPLEMENTABLE.** (Also: the shipped `tauri.conf.json` already means physical px, so nothing changes meaning mid-flight; and restore happens **before the webview exists**, so there is no DPR to convert with at the moment it is needed.)

---

## ⚠️ A DEFERRAL OUTLIVED ITS PREMISE — AND NEARLY DID SO SILENTLY

**N-095** pinned to **M-RP7.3**: *"a missing / corrupt / schema-stale layout falls back to `DEFAULT_LAYOUT`, never a blank centre — **exercised**, not asserted."* **The reasoning was correct**: `loadLayout()` **could not return null**, so the guard would have been an **unreachable branch in a closed milestone** — the same rule that kept `tabs` out of renderer A.

**M-RP6.1k is the milestone that KILLS that premise.** It is precisely where `loadLayout()` stops returning a constant and starts parsing a real file. **Leaving the DoD at 7.3 would have meant 7.3 closing a hole 6.1k opened.**

> ### **A DEFERRAL IS VALID ONLY AS LONG AS ITS PREMISE HOLDS. WHEN THE PREMISE DIES, THE DEFERRAL DIES WITH IT — IT DOES NOT QUIETLY INHERIT A NEW ONE.**
> **Practical form: a deferral must record its WHY, not just its WHEN** — the *why* is the only part that can later be checked against reality. **N-095 recorded its why**, which is the entire reason this was catchable **by reading** rather than by a bug report. *(The inverse is the real hazard: a deferral that says only "do this at 7.3" survives every premise change, forever, unexamined.)*

---

## The rest of the lock

**Renamed, at a cost of two lines, in the one window where it is free: `layout.save` / `layout.load` → `uistate.save` / `uistate.load`.** The faces shipped at 6.1j carrying `layout.*` — **but the store is not a layout**; it holds geometry, and will hold shelf / theme / room. `layout.*` would be a lie by M-RP6.2. **The commands do not exist yet, so this costs nothing today and an explanation forever after.** *Same instinct as S-5's “pin a face” over “add a widget”.* **No `uistate.saveAs` id** — one diskette, one dialog, two outcomes.

**Legs — VISIBLE FIRST, per Joe's standing brief (*see the UI early, correct it while the milestone is open*):**

| leg | scope |
|---|---|
| **A** | save + load **dialogs** (shell-local, the `about-dialog` precedent) + **both commands enter `commandTable`** + **`diskette`/`load` FLIP ENABLED**, against an **in-memory** store. **The milestone's entire UI is on screen and correctable before one line of plumbing exists.** |
| **B** | Rust `get_ui_state` / `set_ui_state` + `xgen-client_uistate.json` + the `loadLayout()` body swap + **N-095 EXERCISED** (corrupt file → `DEFAULT_LAYOUT`, never a blank centre) |
| **C** | window geometry: save on `CloseRequested` + debounced move/resize; restore + **work-area clamp EXERCISED** (write an off-screen rect, launch, watch it centre) |
| **D** | named states carry geometry (§4.2) + session key + reconcile + CDP verify → close |

**Verification frame.** Real client **9222 only** — no sampler cells; **the sampler catalogue must stay 328, and that is a leg**. Client registry baseline **46**, and per **N-105** it is read **QUIESCENT** or not at all. **`cargo test` MUST MOVE from 1507/0/62** — the *inverse* of the 6.1i/6.1j leg where *identical-to-baseline* **proved** the no-Rust claim. **Rule 5 stands: Chat re-drives every non-destructive leg, no exceptions** — the last five handbacks (J-498, J-499, J-500, J-508, J-509) each had a right conclusion and a wrong figure, and **there has never been a signal telling you which number is the bad one.**

**✅ Debt cleared (Chat's own drift, caught after the J-509 push).** ROADMAP and CLAUDE.md still read *"Still open for Joe: Settings' own surface"*. **It is not open — D-112 closed it** (Settings takes `surface: window`; no `screen` kind was added). Both lines fixed. ***A closed decision that keeps advertising itself as open is a record that trains its readers to distrust it.*** **The only thing still open for Joe is top-shelf pinning (surfaces §6 ④) — and it gates nothing.**

**Not smuggled in, and named so they stay out of the milestone:** top-shelf pinning · M-RP6.6 client resident (the real F-1 close) · the read-marker protocol gap · M-RP-FOCUS · M-RP-ROVING · M-RP6.1e-B1 · M-RP7.x · M-RP8 · M-RP-ICON-ADOPT · `temperature-indicator` ⏸️. **A View/Widgets menu is NOT built here** — S-7 makes it free later, and "free later" is not "now".

**Lanes (D-074):** this is Chat's design walk + canonical runbook. **Clair's commit is code-only (commit 1); Chat's doc-bridge is commit 2. Joe pushes both. Chat never pushes.** *(The lane slipped at J-498 and J-508; it held at J-509. Keep it holding.)*

---

## Entry J-509 — THE REGISTRY BREATHES: the shelves mount in the real client, and a "baseline correction" turns out to be a menu left open

**M-RP6.1j ✅ CLOSED. A MOUNT, not a build — 2 files, +30 (`app_client.svelte` + `app.css`). No `core`, no `skin.css`, no glyph, no sampler, no Rust, no `commandTable`.** Feat **`96a5a60`** [Clair, code-only — **the D-074 lane held this time**, after slipping at J-498 and J-508].

**What shipped.** The `shelf` (J-508) mounted into the real client frame: a **top** strip (**empty** → `[data-empty]` → collapses to height 0, but **registered anyway**, N-053 — so pinning is a **one-line population** later) and a **bottom** strip (three faces: `gear`→`widget.manager`, `diskette`→`layout.save`, `load`→`layout.load`, **all `disabled`**). Frame-column pin `.app-frame > .shelf { flex: 0 0 auto; }` in **`app.css`**, following the shipped `.menu-bar` / `.status-bar` lines.

**🔒 THE DECISION THAT MAKES THE MILESTONE HONEST: `commandTable` WAS NOT TOUCHED.** The three commands **do not exist** — `widget.manager` is 6.1l; `layout.save`/`layout.load` act on the named UI states 6.1k's store creates.

> **A registered command id that resolves to nothing is a WORSE LIE than a disabled button.** It reads as *wired* to every future reader and no-ops forever. **A visibly disabled control is an honest phase-limit (W-8)** — the `self-panel` posture (`registered:false` rendered honestly rather than faked).

**But the `onCommand` seam IS wired live**, so **6.1k and 6.1l are each ONE table entry + ONE `disabled` flip** — and the disabled-but-**keyboard-reachable** state was proven at 6.1i, so this milestone **inherits** it rather than debuting it. *(That is the entire reason `aria-disabled` was chosen over native `disabled`: a natively-disabled `<button>` is not focusable, and an all-disabled bottom shelf would have been **invisible to the keyboard**.)* **Binding, DoD-bound: no face is enabled before its command exists, and no milestone closes leaving its own face disabled.** The disabled state is a **countdown**, not a resting state — and it is visible in the app, which is the best possible reminder.

---

## 🔑 THE ENTRY'S REAL FINDING: A NUMBER THAT DISAGREES WITH THE RECORD IS A HYPOTHESIS, NOT A DISCOVERY

The impl seat measured the client registry at **47**, found **39** pre-existing where the record says **38** (J-501), and — carefully, in good faith, flagging it as her own — **recommended correcting the canonical baseline to 39.**

**Re-driven (Rule 5), the count is 46, and 38 was right all along.** The mechanism was **proven, not argued**:

```
BEFORE    : {"n":46}
MENU OPEN : {"n":47, menuItems:["menu-item#app-menubar__file-exit"]}
AFTER     : {"n":46}
```

**A `menu-item` registers on OPEN and unregisters on CLOSE** — shipped, verified behaviour since **J-492**. Her own **leg 6** roved **File↔Help**. **The registry was read with a popup open, and the phantom was exactly +1.**

> ### THE REGISTRY IS NOT A CONSTANT. IT BREATHES.
> **Assert the UI is QUIESCENT before you count.** Every **owned-popup** component registers children while open — `menu` · `combobox` · `tag-select` · `color-picker` (whose `__hue`/`__alpha` ranges register **only while open**, J-452) · `entity-context-menu`. **Read `openIndex === -1` first, in a SEPARATE eval.**
> **`dialog` is the exception that proves it:** a closed `<dialog>` is `display:none`, **not unmounted** (J-496) — so the About box's **14** entries are *always* there. **Some components leave; some stay. You cannot know the total without knowing which state you are in.**

**⚠️ AND NOTE WHAT THE "CORRECTION" WOULD HAVE COST.** It would have written a **wrong** baseline into a canonical record **and retroactively impugned a correct one** — the worst of both, and the kind of damage that compounds silently, because every later delta would have been measured from a fiction. ***The first thing to test, when a number disagrees with the record, is the measurement.***

**This is the N-099 family, third variant.** N-099 was *a null masquerading as a match*; this is **a transient masquerading as a baseline**. Same root: **a check whose subject is in the wrong state still returns an answer, and the answer is always the plausible one.**

*Her delta — **+8** — was exactly right, and independently confirmed: 2 `shelf#` + 3 `shelf-face#` + 3 `icon#…__icon`. **The conclusion was sound and the figure was not. Recorded as such, because that is now the fifth time this arc, and the pattern is the point.*** → **N-105**.

---

**⚠️ SECOND FINDING — A HARNESS TRAP THAT FAILS QUIETLY, AND IT IS FIXED.** Chat's first eval returned a bare **`EVAL ERROR: Uncaught`**. **Not a broken bridge — the wrong page.**

```
page   DevTools - localhost:5173/   devtools://devtools/bundled/devtools_app.html?…   ← attached HERE
page   XGen Client                  http://localhost:5173/
```

**`cdp-debug.ps1` took the FIRST `page` target — and an open DevTools window is ITSELF a page target**, and it sorts first. It does not fail loudly; **it silently evaluates against the wrong document**, and the symptom reads like a dead debug bridge. **Fixed (one line, its own tooling commit):** the target filter now tests the **scheme** as well as the type (`type -eq 'page' -and url -like 'http*'`). **Proven with DevTools still open** — the same command that failed now attaches to `http://localhost:5173/` and returns `{n:46, quiescent:true, shelfFamily:8}`.

---

**CDP-verified, real client 9222 — Chat re-drove EVERY leg (Rule 5; the sampler has no frame, D-097):**

- **Registry 38 → 46 (+8)**, `count === unique`, **enumerated**, **and read QUIESCENT** (`openIndex === -1` asserted first — the leg this entry exists to teach).
- **⚠️ GEOMETRY — the milestone's real risk, and it came out green.** Two new fixed rows entered the `.app-frame` flex column, which is exactly where the two scars bite: **N-088** (`#app` carried no height rule for the project's entire life — masked until something first had to pin to an edge) and **J-499** (`min-height:0` must ride **every** nested flex level or the blowout puts the scrollbar on the **document**). Measured: **`docNoScroll` true** (864 === 864) · the frame fills the window · the region grid still **fills** `.app-center` · **split ratios `[1,2,7,2]` EXACT at winW 992 — a different width than any prior measurement, which is a stronger proof than reproducing a number** · a leaf still **self-scrolls** while the document stays pinned.
- **The collapsed top strip:** in the registry, `data-empty="true"`, **painted height 0**, and **`borderBottomWidth: 0px`** — **on the computed style, not the attribute** (N-097). *A zero-height strip with a 1px hairline under the menu-bar is precisely the sort of thing that ships.*
- **The bottom strip is real:** 28.8px, right-aligned, three faces, **flush above the status-bar** (botBottom 844 = statusBar top 844).
- **Disabled, in the real frame:** all three `aria-disabled="true"` + **`nativeDisabled: false`** + **keyboard-reachable** (rove lands, `activeElement` moves); clicking is **inert and does not throw** — *D2's proof: the seam is live and the table is honestly empty.*
- **The menu-bar was not shadowed** by two new toolbars entering the document (File↔Help roving intact; the `Ctrl+Q` hint present).
- **Static gates:** `vite build` **160** · `npm test` **41** · `cargo test` **1507 / 0 / 62 — identical to baseline**, *which proves the no-Rust claim rather than asserting it* · `git show --stat` = **2 files**.
- **N-092a honoured — the orphan leg was NOT run and no weaker leg was substituted for it.** The client bridge is **state-only** (no DOM handle), so `domCount`/orphans is a **sampler** capability; and this milestone has **no churn**, so there is not even a return-to-baseline proxy. **Said plainly rather than papered over.**

**Deviations — flagged, not absorbed (Rule 6):**
1. **⚠️ The baseline "correction" (above) — rejected on measurement.** Her delta stands; her baseline does not.
2. **The runbook's folder assumption was WRONG — Chat's error, Clair's catch.** §4 implied `layout/`; the shipped `shelf` lives in **`data-independent/`**, beside `MenuBar`/`StatusBar` — which is **correct** (a shelf is chrome, not a grid node). **The code won.** *The "if the code contradicts me, the code wins" hedge has now caught **four** errors, and two of them were mine.*
3. **N-099 recurred in her own harness — and she caught it.** A same-eval read of the reflected `tabindex` returned a **stale** array (Svelte reflects on the effect flush). Re-driven across two evals. **No wrong number entered the record.** She also caught her own second error: `focus()` moves DOM focus but **does not** sync the shelf's internal `activeIndex` — only arrow keys do.
4. **Live `Ctrl+Q` → quit not re-run** — destructive, proven at J-492, and out of scope by construction (the shelf/menu keydown handlers consume only arrow/Home/End and never `stopPropagation` a `Ctrl+Q`). **Named, not silently skipped.**

**Ordinal (Joe-locked):** `shelf`/`shelf-face` stay **33rd / 34th**. The frame **menu trio** (J-492) was never ordinal-numbered and is **not retro-counted** — it is frame chrome and is not in the sampler catalogue, and renumbering a shipped record to fix a cosmetic count is exactly the churn D-069 exists to prevent.

**Records.** `JOURNAL` J-509 · `ui/docs/xgen-ui-notes.md` **v0.80 +N-105** · `docs/xgen-client-frame-phase0.md` **v2.2** (§6: 6.1i + 6.1j) · `docs/ROADMAP.md` **v4.79** · `CLAUDE.md`. **Components registry UNTOUCHED — no component was built** (D-065: an N-note is not invented for a milestone that has no component lesson, and a registry is not bumped for a mount). **No new D** (D-107 / D-112 extension). Tooling: the `cdp-debug.ps1` target filter, **its own commit**.

**Next-active: M-RP6.1k — the UI-state store** (session state + **named** states + window geometry, absorbing M-RP-WINSTATE) — and **its DoD includes flipping `diskette` + `load` to enabled.** Then **M-RP6.1l** (widget manager; flips `gear`). Still open: **top-shelf pinning** (surfaces §6 ④ — gates nothing; the strip mounts empty) · **M-RP6.6 client resident** (Rust; **no part of it enters a UI milestone**) · the **read-marker protocol gap** · **M-RP-FOCUS** · **M-RP-ROVING** · M-RP6.1e-B1 · M-RP7.x · M-RP8 · M-RP-ICON-ADOPT · `temperature-indicator` ⏸️.

---

## Entry J-508 — M-RP6.1i `shelf` + `shelf-face` core (ordered command strip + faces) — sampler-verified, code + doc-bridge, no Rust

**Two commits (D-074).** Feat = code only (8 files, +353, `git show --stat`-clean: no `ui/client/**`, no `ui/node/**`, no Rust). This doc-bridge carries the records. **Joe pushes both; Chat never pushes.** Every count below is **measured** (Rule 5) — Chat re-drove every non-destructive CDP leg itself in the real sampler under `tauri dev` (9422), both accents.

**What shipped.** The first UNGATED milestone of the M-RP6.1i–l arc (D-112 closed the taxonomy; surfaces §6 ④ top-shelf pinning gates nothing). Two new **`core`** di components — **`shelf`** (the **33rd** `core`) + **`shelf-face`** (the **34th**), continuing the doc's explicit ordinal sequence from `region-shell` (32nd, J-499). *(Recorded, not silently renumbered: the frame menu trio — `menu-bar`/`menu`/`menu-item`, J-492 — was never given ordinals, so the sequence continues at 33/34, not 35/36.)*

- **`shelf.svelte`** — the ordered command **STRIP** framing the widget grid (top = favourites, bottom = system). `role="toolbar"` + `aria-orientation="horizontal"`, **roving tabindex**. Frame chrome OUTSIDE the `Layout` descriptor (S-1). `onCommand` seam (byte-for-byte the `menu-bar` seam — a shelf button and a menu item become one command with two triggers). `data-position` + `data-empty` reflected. **NOT a grid** (no `leaf`/`split`/`tabs`, S-3), **NOT a status-bar** (active command surface, not passive), **NOT a second dock** (S-4).
- **`shelf-face.svelte`** — the leaf: its own native `<button>` composing the `icon` core child, self-registering with its own getter. **Own component, not an `sb-cell`-style non-registering part** — a face carries state (command/disabled/active) and must be independently CDP-readable.
- **The keyboard machine is `menu-bar`'s LINEAR rove, COPIED and nothing else (D3).** `activeIndex` + a `faces[]` ref array + ArrowLeft/Right/Home/End, wrap-around, `preventDefault`, `focus()`. **With this milestone roving-tabindex reaches its FOURTH independent implementation** (`entity-panel` · `menu-bar` · `menu` · `shelf`) — **D-069's four-recurrence bar, MET** — and it is **filed as M-RP-ROVING, deliberately NOT extracted here** (a shared helper touches three closed, CDP-verified components; that is its own milestone). `menu-bar`/`menu`/`entity-panel` were **not touched** (proven by scope).

**🔑 THREE things the runbook was emphatic about, and the code honours each:**
- **`aria-disabled`, NOT native `disabled`** (grounded: `menu-item.svelte` does exactly this). A natively-disabled `<button>` is not focusable; at **M-RP6.1j every bottom-shelf face is disabled**, so native `disabled` would leave the strip **invisible to the keyboard**. The face uses `aria-disabled` + a **guarded `onclick`** (native Enter/Space route through the click, so the ONE guard covers both) + skin hooks on `[aria-disabled="true"]`. **CDP-proven: the disabled face is keyboard-REACHABLE** (rove lands on it, `nativeDisabled:false`) and **activation is inert** (click leaves the probe unchanged).
- **NO badge (D6).** Nothing produces a count, and unread counts have **no protocol mechanism** (the read-marker gap, J-503) — a badge prop with no feeder is the N-097 shape (`.entity-item[data-selected]` shipped unreachable). No prop, no skin rule, no reserved slot.
- **Empty = MOUNTED, not absent (D4/N-053).** `shelf#top-empty` stays in the registry; `data-empty` collapses the painted box to **height 0** (the `status.svelte` mounted-but-empty precedent). This is what the top shelf looks like at 6.1j before pinning.

**Glyphs — three new, sourced NOT from memory (D-108/D-110).** `gear`/`diskette`/`load` = Material Icons **`settings`/`save`/`folder_open`** (Apache-2.0), path `d` copied **byte-for-byte from the real 24×24 SVGs** (curl'd from `google/material-design-icons`), source `.svg` + per-glyph provenance (licence + Material name + source URL) saved to `ui/assets/icons/`. **⚠️ Real trap avoided:** each Material source SVG carries a `fill="none"` bounding rect (`M0 0h24v24H0z`) — `icons.ts` renders **every** path with `fill: currentColor`, so keeping that rect would have painted a **solid 24×24 square over the glyph**. Dropped it; kept the glyph subpath only. **Colour-free geometry** (a baked-in fill fuses colour+shape into one token and makes the D-110 Space-theme ban unenforceable on that glyph). CDP-proven each glyph renders a **real single `<path>` with a non-degenerate `getBBox()`** (gear 18.7×19.2, diskette 18×18, load 20×16 — the leg that catches a mis-copied path).

**CDP-verified (sampler 9422, both accents) — every leg re-driven by Chat:**
- **Registry** — `ids().length` **313 → 328 (+15)**; `count === unique === domCount` (**328/328/328**); **0 orphans both directions**. The +15 = 3 `shelf#` roots + 6 `shelf-face#` (bottom 3 + mixed 3; `top-empty` 0) + 6 composed `icon#…__icon` children.
- **Getters** — `shelf#bottom` `{position:"bottom",itemCount:3,activeIndex:0}`; `shelf#top-empty` `{position:"top",itemCount:0,activeIndex:0}`; `shelf-face#bottom__0` `{command:"widget.manager",hasIcon:true,disabled:false,active:true}`; `#mixed__1` `{command:"layout.save",…,disabled:true,active:false}`; the three `icon#…__icon` children `{name:"gear|diskette|load",size:16,tint:"currentColor",decorative:true}`.
- **Roving** — `["0","-1","-1"]` → ArrowRight → `["-1","0","-1"]` (activeElement moves to face 1, focus is the point) → **wrap BOTH directions** (End→ArrowRight wraps to first; Home→ArrowLeft wraps to last). *(N-099 honoured: click/keypress and the reactive-attribute read are SEPARATE evals — Svelte flushes the tabindex on the effect tick; `focus()` is synchronous, the attribute is not.)*
- **Dispatch** — click face 0/1/2 → probe reads `widget.manager` / `layout.save` / `layout.load` (the commandId, not the click count).
- **Disabled** — `#mixed__1` click → probe **unchanged** (`layout.load`, the guard held); **keyboard-reachable** (rove from `mixed__0` lands on `mixed__1`; `aria-disabled:"true"`, `nativeDisabled:false`).
- **Empty** — `shelf#top-empty` in registry, `data-empty:"true"`, **painted `getBoundingClientRect().height === 0`**, 0 faces.
- **Skin** — all **10** `.shelf*`/`.shelf-face*` rules in cascade incl. `[aria-disabled="true"]`; **zero component `<style>`**; **accent-neutral** — under the accent swap, `shelfBg` (`rgb(28,31,36)` = `--s2`) and `faceColor` (`rgb(138,136,128)` = `--t3`) are **identical** client↔node; **only `--accent2` and the focus-ring's accent segment move** (gold `#c28840` ↔ blue `#3a7ab0`). Eye-checked both screenshots (strips pixel-identical except the focus ring on the focused disabled face).
  - *(Forward note — when **M-RP-FOCUS** moves `--focus-ring` onto a new `--focus` token, this leg becomes "NOTHING moves under an accent swap." That is the fix landing, not a regression; the shelf rides the token unchanged.)*
- **Static gates (apps down):** `vite build` **169 modules** clean · `npm test` **41 passed** · `cargo test --workspace` **1507 / 0 / 62 — IDENTICAL to baseline**, which *proves* the no-Rust claim rather than asserting it.
- **Scope** — `git show --stat` = the 8 §3 files, +353, nothing else.

**Deviations / findings, flagged not absorbed (Rule 6):**
- **The synthetic-Enter path is INCONCLUSIVE, and the record says so.** A synthetic `KeyboardEvent` is untrusted and does **not** trigger a native `<button>`'s Enter/Space→click default (the J-496/J-498 precedent). So the disabled-Enter-inert leg cannot be driven via the eval harness. **The guard is proven at its real convergence point instead:** `onclick` is where mouse-click and native keyboard activation both land, and the click-inert result on the disabled face proves it. One guarded handler, verified via `.click()`.
- **Harness note:** `window.__XGEN_DEBUG__.get(id)` returns `{type, state}` directly (`state` = the getter output), **not** an object with a `.get()` method — an eval that calls `.get()` on the result throws a bare `Uncaught`. Recorded so the next runbook's getter reads use `.state`.
- Runbook match otherwise **exact** — every grounded claim (`menu-item`'s `aria-disabled`, `button` rendering `{label}`-only, `sb-cell`'s no-envelope, `icons.ts`'s `name→d` map) held against the code.

**Records.** components registry **v0.71 → v0.72** (`shelf` = **33rd** `core`, `shelf-face` = **34th**; sampler catalogue **313 → 328**) · ROADMAP **v4.77 → v4.78** (M-RP6.1i ✅) · `ui/docs/xgen-ui-notes.md` **+N-104** (the shelf/shelf-face pattern: toolbar roving-copy, the `aria-disabled`-keeps-a-disabled-face-focusable rule, the drop-the-`fill=none`-rect glyph trap) · **no new D** (D-107/D-112 extension). Feat commit = code only; this doc-bridge = commit 2.

**Next-active: M-RP6.1j — the client mount** (the shelves land in the real client frame; the bottom shelf mounts with its three faces **disabled**, because their commands do not exist yet — the countdown D7 describes, and the disabled-but-reachable state this milestone proved). Still open: **④ top-shelf pinning** (gates nothing) · M-RP6.1k (UI-state store) · M-RP6.1l (widget manager) · M-RP-ROVING (the 4th-recurrence extraction) · M-RP6.6 client resident · M-RP7.x · M-RP8 · M-RP-ICON-ADOPT · `temperature-indicator` ⏸️.

---

## Entry J-507 — THE CODE HAD ALREADY ANSWERED IT: the plugin taxonomy closed (D-112) and the module-UI sandbox floor locked (D-113) — design/records-only, no code

**Design + records only. No code. Registry unchanged (client 38, sampler catalogue 313).** The D-071 taxonomy Phase-0, with **two required outputs** — the taxonomy and the sandbox — **locked together, because you cannot classify a thing while leaving open what the thing you are classifying is allowed to do.** Joe delegated the walk (*"you have autonomy in this part… do as you propose"*), then locked all five recommendations (*"lock all by your recomms"*).

**🔑 THE SESSION'S REAL OUTPUT, AND IT IS THE SAME LESSON AGAIN — BUT THIS TIME THE GREP RAN FIRST.**

> **The reconciliation frame Joe stated on 2026-07-11 — *"module and widget is the plugin in the two areas: system and ui"* — was ALREADY IN THE CODE, VERBATIM, IN A FILE NOBODY IN THIS ARC HAD OPENED.**

`xgen-common/src/module.rs` (**SE-D2**, the Storage-Engine / Plugin-Framework milestone):

> *"There is one unified handshake mechanism; the code term **`kind`** carries the system/ui distinction: a **module** is a *system* plugin (`host = node`), a **plugin** is a *ui* plugin (`host = client`)."*

It also ships **slot/impl identity** (`ModuleKindId` / `ModuleImplId` — **UUIDv4, never `Xgid`**: *"a module GUID is local, developer-assigned, and never crosses the wire"*) and a **trust posture**: *"the descriptor is a **const in the plugin's own code** — there is **no manifest file** … metadata is **authoritative**, location is **never trusted**."*

*After seven grounding misses — and two in J-505/J-506 alone — the rule was applied **before** the design instead of after the pushback. It cost one `search_files` call and it reshaped the entire arc.*

**⚠️ HONEST LIMIT, STATED IN THE RECORD RATHER THAN GLOSSED:** the shipped `Descriptor` struct is `{kind_id, impl_id, name, assurance}` — there is **no `host` field, no `kind` field, no `ui_form` field**, and `assurance` is storage-engine-specific. **The vocabulary is shipped; the fields are not.** D-112 *extends* a real spine — it does not describe fields that already exist.

**🔑 AND THE GREP FOUND A THIRD SPECIES THAT NEITHER SPEC LISTED.** `xgen-core/src/auth/module_registry.rs`: an **Auth Module is a protocol principal** — `AuthModuleXgid` (the 7th XGID flavour) + an **`endpoint_url`** + a Node-side **trusted registry with revocation** (block-only, A2-D1), its key **recovered from the XGID itself** (AMR-D3). **It is a network service.** **Ch6 §6.8.7 calls it *"the reference implementation of a Window-form module"* — the shipped code says otherwise, and §6.8.7 is now corrected in place.** Also grounded: **zero `xgen-module.json` anywhere**; no manifest loader, no `modules/` scan; and `temperature.rs` **itself** leaves its loader open (*"dynamic libraries, WASM, external process"*).

> ### 🔑 SO THE DRIFT WAS NEVER "Ch6 vs D-102". IT WAS **§6.8 vs EVERYTHING THAT WAS ACTUALLY BUILT.**
>
> §6.8 was written **Session 2, April 2026** — before the plugin spine, the Auth Module registry, the widget tier, the region model and `WidgetMount`. **Four later artefacts converged on a different shape and none of them consulted it.** **It is the outlier, and it is the section that moved.** *(The J-502 "first bird" shape a second time — a section named before every convention it would live among existed. **The deferral is why nothing hardened wrong.**)*

**✅ D-112 — ONE PLUGIN, THREE AXES.** **`host`** (`node` = the *system* area · `client` = the *ui* area) · **`delivery`** (`compiled` · `service` · `packaged`) · **`surface`** (`none` · `region` · `shelf` · `window`, at most one). ***"Module" and "widget" are not two species.*** **The `delivery` axis is the one nobody had, and it is where trust lives:** `compiled` (const descriptor, our binary — **everything shipped**) · `service` (own XGID, own endpoint — **the Auth Module**) · `packaged` (third-party code + manifest — **zero lines exist**).

**🔑 THE SLOT INVENTORY DOES NOT RETIRE — AND IT WAS NEVER A RIVAL PLACEMENT MODEL. WE HAD ALREADY BUILT IT.** Split Ch6's table against **this project's own** surfaces §3.2 clause (*content inside another widget is not a surface*): `node.dashboard.widget` / `room.sidebar.*` are **regions** (tiles); `room.toolbar` / `room.message.decorator` / `space.header` / `global.statusbar` are **content anchors inside a host widget**. And the anchor mechanism **already ships**: `message.svelte` takes **`details: WidgetMount[]`**, resolves against a prop-injected registry and **drops unknown ids** (W-13, M-RP5.5 / J-478).

> ### ***`room.message.decorator` IS `message.details` UNDER ANOTHER NAME.***
> **Nobody noticed, because one was written in April and the other in July.** → **ONE placement model** (the D-103 descriptor) **+ ONE containment model** (host-declared `WidgetMount[]` anchors). **A slot is declared by the HOST, never requested by the guest** — that is the anti-drift property, and it is why both mechanisms coexist without a second registry. Ch6's table is marked **STALE** (guessed against a Room view that does not exist); the real anchor inventory is **regenerated from the widgets that actually exist** at M-RP7.4.

**Also settled:** the **manifest is reconciled, not merged** — a compiled `Descriptor` is **authoritative**, a `packaged` manifest is **untrusted input** (it *declares*; the host *enforces*); **they must never become one type.** **Settings takes `surface: window`; NO `screen` kind is added** — the `window` form already exists and has a second consumer, and *Ch6 §6.8.5's "a screen of its own" is prose, not a surface kind*. **⚠️ Foreclosed knowingly:** the Discord full-window overlay shape — **a product choice, reversible on one word, and it must be a lock, never a drift.**

---

## 🔒 D-113 — THE SANDBOX, AND THE REFRAME IS THE WHOLE DECISION

**Ch6 §6.8.8 has asked since Session 2 (April 2026): *"Widget sandboxing: what CSP and iframe sandboxing apply?"***

> ### **But `self-panel` needs no CSP. A compiled Rust storage engine needs no CSP.**
> ### **Nothing about *being a widget* is dangerous. Being `delivery: packaged` is.**
>
> **The question was attached to the WRONG NOUN for three months** — and a widget-shaped rule could never have covered a packaged module's **window**, which is the same risk in a different frame.

**Why it is the largest open surface in the project.** Every other content channel has a **structural foreclosure**, not a filter: **blobs are content-addressed — a hash cannot name a host** (D-111) · **a Space theme is a colour-only allowlist — a colour cannot be a `url()`** (D-110) · **glyphs are banned from Space override** · **fonts are bundled**. **A `packaged` module UI has NONE of them** — arbitrary third-party markup and script **with a network stack**.

**✅ THE FLOOR. Foreclose; do not filter.**

> **S-1 — A PACKAGED MODULE UI IS A WEBVIEW WITH NO NETWORK.** Its only egress is the local IPC channel to its **own backend**, which runs on the Node/Client **we** ship. `default-src 'none'; … connect-src <its own local channel>; frame-ancestors 'none'`. **No `http:` / `https:` scheme is reachable at all.**
>
> **🔑 This makes D-111's beacon UNSAYABLE inside a module — exactly as it is unsayable in a message. The same structural property, taken a second time.**

**S-2** packaged assets, never fetched (*the bundled-fonts rule, generalised — a packaged asset cannot phone home*) · **S-3** own webview, **own origin**, **no Tauri IPC**, no host DOM, no sight of another module · **S-4** **the module never holds the key** — `identity_mode: user` is **consent, not key handover**: the module *requests*, **our Rust signs, per event** (*a module that can sign as you at will is not a module — it is you*) · **S-5** **a module UI may not draw trust chrome** — bounded, attributed frame; **never** the identity / verified / **AI-badge (§6.13)** zone (*icon spoofing does not become acceptable because the spoofer arrived as a package instead of a theme* — **D-110's lesson, generalised beyond CSS**) · **S-6** capabilities **deny-by-default, allowlist never denylist** · **S-7** **no `packaged` plugin loads until S-1…S-6 ship.**

**🔑 LOCKED AGAINST ZERO CODE, DELIBERATELY — BECAUSE THAT IS THE CHEAPEST MOMENT A TRUST BOUNDARY CAN EVER BE SET.** `state.space_theme` was locked before a line of it existed (D-110); `delivery: packaged` is in the same position **today** — no manifest, no loader, no webview. **S-7 therefore costs nothing and forecloses everything**, and **the first packaged module that ever loads will load into a floor that already exists.** *(D-071 paying out **in advance** rather than in arrears — twice in three sessions.)*

**And it closes two more of §6.8.8's five in passing** — **module signing** (mandatory for `packaged` · meaningless for `compiled` · **already solved** for `service`, since an Auth Module **is** its key) and **module permissions** (host-side, deny-by-default). ***Two open questions dissolving the moment an axis exists is the sign the axis is real.*** Only **hot-loading** and **module-to-module** remain — and **S-3 already forecloses the UI half of the latter.**

**⚠️ NAMED AND NOT SOLVED (so it cannot be smuggled in later):** the **`compiled` plugin LOADING mechanism** is still open in `temperature.rs` itself. **A dynamic library is not a sandbox** — if the loader ever becomes `dlopen`, that plugin carries `compiled`-trust with **none** of `compiled`'s review, and this taxonomy would be quietly lying. **Whichever loader is chosen must land on the delivery axis, and if it admits third-party code it inherits D-113.**

---

**✅ VERIFIED, NOT ASSUMED (the briefing asked; the answer is yes).** Surfaces §6 item ⑤ (**glyph licence provenance**) has been **dissolved by D-108** — `docs/xgen-icon-adoption.md` §3f: *"licence + source live in `icons.manifest.json`, **per glyph** — a glyph with no licence entry **fails the build**."* **No audit can forget what the compiler enforces.** What remains is **mechanically sourcing** gear / diskette / load — a task, not a decision. **⑤ STRUCK.**

> ### → **surfaces §6: ① ② ③ CLOSED · ⑤ STRUCK · ONLY ④ (top-shelf pinning) REMAINS — AND IT GATES NOTHING. M-RP6.1i–l ARE UNGATED.**

**Tooling (Rule 3, logged not absorbed).** The Windows-MCP **PowerShell tool timed out and did not recover** for the first half of the session (4-minute hang, no result). The Phase-0 was grounded **entirely through the Filesystem MCP** — **every claim in it comes from a file that was read, none from a `Select-String` that could not be run.** The tool recovered later and the commit was verified with it (`git show --stat` = one file, scope-clean).

**Records.** **`DECISIONS.md` +D-112 +D-113** (appended at the bottom, following the shipped D-099…D-111 pattern — **a 4,000-line record is not silently re-sorted**) · **D-111 amended in place** (its *"open and NOT closed here"* module-widget note now points at D-113) · `docs/xgen-plugin-taxonomy-phase0.md` **v1.1, Status COMPLETED** · `docs/xgen_ch6_client_design.md` **v0.5 → v0.6** (§6.8.3 amended · slot table **STALE** · §6.8.7 **corrected** · §6.8.8 **3 of 5 CLOSED** · Session 11 logged) · `ui/docs/xgen-widget-tier.md` **v1.3** (W-12 amended + the delivery axis; *a widget is a plugin with `host = client`*) · `ui/docs/xgen-region-dock-model.md` **v1.7** (§11 CLOSED) · `docs/xgen-widget-surfaces-phase0.md` **v1.4** (§9 CLOSED · §6 ① CLOSED · ⑤ STRUCK · build-order + records-to-move updated) · `docs/ROADMAP.md` **v4.76**.

**No code moved.** Feat `bfc570a` was the Phase-0 doc alone (1 file, +294, scope-verified by `git show --stat`); this doc-bridge carries the records.

**Next-active: M-RP6.1i — the `shelf` core.** Still open: **④ top-shelf pinning** (gates nothing; the top shelf mounts empty) · **M-RP6.6 client resident** (the real F-1 close — Rust; **no part of it enters a UI milestone**) · the **read-marker protocol gap** (no UI milestone may fake it) · M-RP6.1e-B1 · M-RP7.x · M-RP7.3 (N-095) · M-RP8 · M-RP-ICON-ADOPT · `temperature-indicator` ⏸️.

---

## Entry J-506 — A HASH CANNOT NAME A HOST: the `url()` claim RETRACTED, the real invariant found, D-111 locked — design/records-only, no code

**Design + records only. No code. Registry unchanged (client 38, sampler catalogue 313).** Joe asked **one question** — ***"do you want to say that url() is a security risk?"*** — and the honest answer was **no**. **The claim collapsed under one grep. The collapse is the entry.**

**⚠️ THE RETRACTION, FIRST AND PLAINLY.** J-504, J-505, N-101 and N-102 all listed **"`url()` fetches"** (and font substitution) among the open Space-owner trust surfaces. **NEITHER IS ONE.**
- **Under D-110's OWN colour-only allowlist, every value is validated with `CSS.supports('color', v)` — and `CSS.supports('color', 'url(https://evil/x)')` is FALSE. A `url()` cannot enter a Space theme at all.**
- **Fonts are BUNDLED in the binary** — Ch6 §6.2, explicitly *"without runtime internet dependency."*

**I named a threat that the document I had just written already forecloses.** **This is the SECOND overclaim in two turns on one topic** (the first: calling a key→value token map *"attacker-supplied CSS"*). **Same shape as the J-502 temperature miss — the project had already solved it, and I reasoned about it instead of looking.** *Joe caught both by asking a single flat question rather than accepting the frame.*

**🔑 BUT THE GREP THAT KILLED THE CLAIM FOUND THE REAL INVARIANT — AND FOUND THAT THE PROTOCOL HAD ALREADY WON IT.**

> **Any mechanism where the client fetches a URL chosen by SOMEONE ELSE turns the client into a BEACON** — disclosing the reader's **IP address** and the **exact time they read** to a host of the **sender's** choosing.
>
> **XGen publishes your XGID by design. It does NOT publish your network location.** A fetch primitive silently adds a channel the protocol deliberately excludes — and it does so against a **Space owner or a message sender**, not a distant third party.

**Every avenue is already shut — and shut STRUCTURALLY, not by filtering:**
- **`message.image` / `message.file` carry `xgen://hash/sha256:<64-hex>`.** Grounded in `xgen-core/src/blob_store.rs`: `blob_ref = hash_uri(bytes)`, **the same scheme as `event_id`**; blobs are **federation-native** (M12, CLOSED J-389), **per-blob client-encrypted before upload** (M12-D5), and the store is *"content-blind by construction"*. **It is a CONTENT ADDRESS, NOT A LOCATION.**
- **No HTTP client in any crate** — no `reqwest` / `hyper` / `ureq` in xgen-client, xgen-core, xgen-node.
- **Fonts bundled** (Ch6 §6.2). **Space themes cannot carry a URL** (D-110).

> ### 🔑 A HASH CANNOT NAME A HOST.
>
> **The beacon is not *blocked* by content-addressing — it is made UNSAYABLE.** There is no field in which *"over there"* could be written. **That is a materially stronger property than any filter**, and **it was already there.** *Content-addressing was taken for **integrity**; it bought **privacy** for free — and nobody had written that down. The best finding of the session is a property the project already had.*

**⚠️ THE ONE ROW THAT INVITES IT BACK IN: `link previews`.** Ch2's **"Client decisions — implementation freedom"** table lists *"Markdown flavour, emoji rendering, **link previews**"*. Nothing is built — it is an *example* of what the protocol deliberately does not dictate. **But a client that renders a preview by fetching the URL out of a message hands the SENDER every reader's IP and read-time — invisibly, on every message.** In a system that content-addressed its blobs and bundled its fonts *precisely* to prevent that, **"implementation freedom" was doing quiet work in that row.**

**✅ D-111 (Joe-locked).**

> **Link previews — and ANY rendering that resolves an outbound URL — are fetched NODE-SIDE, NEVER client-side.**

**The Node already talks to the world; the Client deliberately does not.** The Node fetches, strips, caches, serves. **One fetch per link, not one per reader** — and **the sender learns nothing about who read the message or when.** **Implementation freedom is preserved and the table is not weakened:** *whether* to show previews and *how* they look stay entirely the client's business. **Only the fetch LOCATION is fixed** — because *it is not a rendering decision. It is a privacy boundary wearing a rendering decision's clothes.*

**🔑 THE STANDING LESSON, AND IT IS ABOUT LISTS, NOT ABOUT URLs:**

> **A threat list padded with a foreclosed item is WORSE than a short one.** It trains the reader to skim — and the **real** entry gets skimmed along with the fake one. **Retract; do not hedge.**

*The open list is now short, and every item on it is live. And the item that actually matters was the one I had buried at the end of the padded version.*

**⚠️ WHAT ACTUALLY REMAINS — AND IT DWARFS EVERY GLYPH, TOKEN AND BLOB REFERENCE COMBINED: D-036 MODULE WIDGETS.** Third-party HTML in an **isolated webview**, talking to its module backend over a local WebSocket. **CSP and sandboxing are STILL AN OPEN QUESTION** — Ch6 §6.8.8, *filed at Session 2 (April 2026) and untouched since*. **Unlike blobs (content-addressed), themes (allowlisted) and glyphs (banned), a module webview has NO structural foreclosure at all** — it is arbitrary third-party markup with a network stack. **Ch6's, not the UI track's. Filed, not solved — and it should not stay filed much longer.**

**Records.** `DECISIONS.md` **+D-111** · **D-110 amended in place** (its "wider surface" paragraph named `url()` and fonts — **corrected, not deleted**) · `docs/xgen_ch2_architecture.md` **v1.2 → v1.3** (the conformance note under the *Client decisions* table; **its `> Version:` header line was also missing the two mandatory trailing spaces — fixed**) · `ui/docs/xgen-ui-notes.md` **v0.78 +N-103**, with **N-101 and N-102 amended in place** · `ui/docs/xgen-css-layer-model.md` **v1.2** (§7 retraction) · `docs/xgen-icon-adoption.md` **v1.2** · `docs/ROADMAP.md` **v4.75** · `CLAUDE.md`.

**No code moved.**

---

## Entry J-505 — D-110: a Space may re-COLOUR, not re-DRAW. Ch6 §6.2 REWRITTEN, §6.3 ANSWERED after 3 months open; a key allowlist alone is THEATRE — design/records-only, no code

**Design + records only. No code. Registry unchanged (client 38, sampler catalogue 313).** Two Joe-locks, both in one line each. On the ban: *"yes, changing glyphs in space has to be baned, perhaps except color change"*. On the chapter: *"those were concepts waiting for real context to specify more. so it needs to be corrected."* **Both were the right call, and grounding the second one produced a finding sharper than the first.**

**✅ D-110 — THE RULE, AND IT IS A TRUST BOUNDARY, NOT A STYLE PREFERENCE.**

> ### A Space may **re-COLOUR**. A Space may **not re-DRAW**, and may **not re-LAYOUT**.

**Colour** — including the **glyph tint** — **✅ permitted**: *the mark keeps its meaning; only its hue changes.* **Geometry** (`--glyph-*`, `--glyph-*-url`) **❌ banned.** **Layout / metrics ❌ banned** (readability, accessibility, and displacement attacks). **Everything not on the allowlist ❌ banned by default — allowlist, NEVER denylist.**

**Why.** Ch6 §6.3's cascade is XGen default → application theme → **Space theme**. Layers 1-2 are ours and the user's. **Layer 3 is not — it is declared by a Space OWNER and arrives over the wire in a `state.space_theme` Event.** Under **D-108** a theme redraws **any** glyph. **Unrestricted, a Space owner could redraw a lock, a warning, a verified mark, or the AI badge (Ch6 §6.13)** — making a hostile Space look trustworthy, or a human member look like a bot. **Icon spoofing, in a protocol whose entire premise is verified identity.** *Joe's "except color change" carve-out is exactly right and load-bearing: recolouring was what Space theming was **for**, and it never touches meaning.*

**🔑 GROUNDING THE CHAPTER CORRECTED MY OWN RECORD — AND THE CORRECTION FOUND THE REAL HOLE.**

**J-504 and N-101 called Layer 3 *"attacker-supplied CSS arriving over the wire."* That was OVERSTATED.** Ch6 §6.2's event shape is a **key→value token MAP** — named keys, scalar values (`"color_primary": "#4f6ef7"`), **not a stylesheet**. The threat is **narrower** than I wrote. **Corrected in D-110 and N-102 rather than left standing.**

**And the narrowing is what exposed the actual danger:**

> ### ⚠️ A KEY ALLOWLIST ALONE IS THEATRE.

If the client applies the map by **building a stylesheet with string concatenation**, a malicious **VALUE** escapes its own declaration and injects arbitrary CSS — **defeating the key allowlist completely**:

```
"color_primary": "red; } :root { --glyph-lock: path('M0 0h24v24H0z'); } /*"
```

**The key is on the allowlist. The value redraws the lock.** **MANDATORY MITIGATION, all three, any one alone insufficient:** **(1)** allowlist the **key**; **(2)** **validate the value** (`CSS.supports('color', v)`) **and apply via `element.style.setProperty()`** — **the CSSOM cannot break out of a declaration**; **never interpolate a wire-supplied value into a `<style>` text node**; **(3)** **scope** to the active Space's subtree — never `:root`, never app chrome. Written into **new Ch6 §6.3.2**.

**🔑 AND THE BAN REACHES BACK AND CONSTRAINS THE GENERATOR — an unusual shape worth remembering.** **`--glyph-*-url` MUST be emitted COLOUR-FREE** (a `currentColor` mask, colour from a **separate** token). *A data-URI with colour **baked into it fuses colour and geometry into ONE token** — so a Space permitted to change that token's colour would thereby be permitted to **redraw** it, and **the ban would be unenforceable on exactly those glyphs**.* **The re-emit of the seven `%23e6e6e6` glyphs (the 5 `textfield[type=]` insets, the `select` arrow, `--ea-spark`) is now a SECURITY REQUIREMENT, not a Phase-0 tidy-up** — it moves from *"classify per glyph"* to **"mandatory for all seven."**

> **The generalisable lesson: an ACCESS RULE is only real if the DATA MODEL can express the distinction it draws.** A policy that says *"colour yes, shape no"* is vapour if colour and shape live in the same token. **The trust decision rewrote a build step.**

**✅ LOCKED BEFORE IMPLEMENTATION — grounded, not assumed.** Grepped the whole tree: **`state.space_theme` appears in NO Rust, NO TypeScript, NO Svelte.** Ch6 §6.3's cascade is **specified and entirely unbuilt**. **D-110 lands before the first line of it is written** — the cheapest moment a trust boundary can ever be set. *(D-071 — "subsystem audits precede dependent milestones" — paying out **in advance** for once, rather than in arrears.)* **Binding: no milestone may claim theming works, and none may ship a Layer-3 applier that does not implement §6.3.2 in full.**

**✅ Ch6 §6.2 REWRITTEN (Ch6 v0.5, Session 10) — the J-504 drift, closed.** Joe was right that these were first-pass concepts awaiting real context. Phase-1 gave them the context; the chapter now describes the code:
- **`tokens.css` was NEVER BUILT.** A separate "vocabulary, not values" layer earned nothing — tokens live in `skin.css` beside the rules that consume them.
- **`skin-dark.css` → `skin.css`.** One skin; dark/light is a **theme-layer** concern, not a filename.
- **A layer was ADDED the original had no concept of: the glyph bank (L1.5, D-108).**
- **🔑 THE REVERSAL — component `<style>` blocks are FORBIDDEN, not required.** D-058 mandated them. **N-025/N-031/N-090 forbid them: a component ships ZERO CSS.**

> **And it reads backwards until you see it: THE RULE THAT MAKES SKINNING TOTAL IS THE RULE THAT FORBIDS THE COMPONENT FROM PARTICIPATING IN IT.** A component that could style itself would be **a second place appearance lives** — and a skin could then never *fully* re-skin it. **D-058 had it exactly inverted.**

**D-057/D-058 are superseded IN PART, not deleted** — their **intent survives intact** (the minimal reset over a generic normalize; the 13px/1.35 root scale; the 4px spacing unit; no hardcoded values in components). Their **file structure and the component-`<style>` rule do not.** Also corrected in-chapter: the stale `xgen-ui-shared/` folder tree → the **D-095** split, and the component-independence paragraph.

**Records.** `docs/xgen_ch6_client_design.md` **v0.4 → v0.5** (§6.2 CSS Layer Architecture **rewritten** · folder tree + component-independence corrected · §6.2 Theming override-list answered · **NEW §6.3.1** the subset **+ §6.3.2** enforcement · **Session 10** log) · `DECISIONS.md` **+D-110** · `ui/docs/xgen-css-layer-model.md` **v1.0 → v1.1** (§4 drift **closed** · §6 the ban · §2.2 the normative colour-free constraint) · `ui/docs/xgen-ui-notes.md` **v0.77 +N-102** · `docs/xgen-icon-adoption.md` **v1.1** (§5 both items **CLOSED**; Phase-0's re-emit now mandatory) · `docs/ROADMAP.md` **v4.74** · `CLAUDE.md`.

**No code moved.** **Still open, flagged not solved:** the exact colour-token allowlist (enumerated when the theme layer is built) · whether a user may disable Space themes entirely (*rec: yes — cheap, since Layer 3 is a scoped, droppable overlay by construction*) · **the WIDER Space-owner-content trust surface** — `url()` fetches, font substitution, module widgets under **D-036**. **D-110 closes the glyph hole, NOT the category.**

---

## Entry J-504 — The GLYPH BANK: a glyph is a SKIN TOKEN. CSS `d:` PROBED and PROVEN on the real client; the CSS layer model gets a canonical doc; TWO drifts filed — design/probe/records-only, no code

**Design + CDP probe + records only. No code. Registry unchanged (client 38, sampler catalogue 313).** Joe opened with a proposal, not a task: *"we do have couples of them already in component, but they are there hardcoded. i propose a library or a bank"* — modelled on a Java `SvgGlyph.GEAR_ICON` enum. **Grounding turned it into a different question, and then a probe answered it with numbers.**

**🔑 THE FIND — THE BANK ALREADY EXISTED. TWICE. AND THE LOSS IT WAS MEANT TO PREVENT HAD ALREADY HAPPENED.**

**Measured, 21 distinct glyphs, FOUR mechanisms across TWO layers:** **A** `icons.ts` → `icon.svelte`, `<path d>`, fill, `--icon-tint` (**3**) · **B** `skin.css` **`mask-image`** data-URIs, mostly stroke (**11 declarations / 10 distinct**) · **C** `skin.css` **`background-image`** data-URIs with **colour baked into the URI** (**7** — the 5 `textfield[type=]` insets, the `select` arrow, `--ea-spark`) · **D** `img-placeholder.svg` as `src` — **and re-inlined a second time** as a data-URI at `app_sampler.svelte:402`.

**And every skin glyph token was declared INSIDE its own component's class selector — not one at `:root`.** `skin.css` said so deliberately: *"icon-data vars scoped here (no global token)."* **Two consequences, both measured, both bad:**
1. **`--tri` / `--tri-open` are declared TWICE** — `.combobox` (1232-33) and `.section` (1829-30), where the section's own comment says ***"REUSES combobox's masked glyphs"* and then re-declares them.** *The exact failure Joe's question was aimed at, already sitting in the tree.*
2. **🔑 A component-scoped custom property is a PRIVATE VARIABLE, not a theme surface.** A theme author cannot redraw *"the eye"* — they must know **which component scopes it** and redefine each shared glyph **N times**. ***Component-scoping half-defeated the very skinnability it was chosen for.*** That is the sentence that decided the design.

**⚠️ AND `docs/xgen-icon-adoption.md` v0.1 §1 WAS WRONG ABOUT THE *WHERE*.** It said the glyphs were *"currently inline `<svg>` inside their field components."* **Grepped every live `.svelte`: ZERO inline `<svg>` in any field component** (only `icon.svelte`, and one sampler demo URI). **A doc written from memory rather than from the tree** — the arc's standing lesson, landing on a *document* this time instead of a design. **The milestone's shape changed with the correction:** not *"extract inline SVG from components"* but ***"reconcile four mechanisms across two layers."***

**✅ JOE'S PRINCIPLE WAS RIGHT, AND CHAT'S FIRST RECOMMENDATION WAS WRONG.** Joe: *"i think that this is better plan than to hardcode them to source code. they can be redefined by skin change."* Chat had proposed **"registry `d` attribute as the default + skin override as a fallback layer"** — and **withdrew it under push-back**: that is **two defaults for one glyph**, a **second source of truth for geometry** (**D-067 drift wearing a safety vest**), insuring against a browser that **cannot occur**. **Geometry lives in the skin. Only in the skin.** *(Chat's own deviation, flagged rather than absorbed — Rule 6.)*

**✅ THE MODEL — LOCKED (Joe) → D-108.**

> **`core` owns the NAME (identity = content). The skin owns the SHAPE (geometry = appearance).**
> **A glyph is a SKIN TOKEN — the same species as `--accent2`.** A component says *which* glyph; the skin says *what it looks like*. **A component never writes geometry, for the same reason it never writes a colour** (N-025 / N-090, applied to glyphs).

**Source (hand):** `ui/assets/icons/*.svg` + `icons.manifest.json` (**licence per glyph → a glyph with no licence FAILS THE BUILD** — the BSL→GPL gate becomes structural, not a periodic audit). **The `.svg` files never ship.** **Generated:** `glyphs.generated.css` → `:root { --glyph-gear: path('…'); --glyph-gear-url: url("data:…") }` (**the bank, and the runtime default**) + `icons.generated.ts` → `type IconName` (**names only, no geometry** — a typo is a compile error). **Two token forms, and they are NOT redundant:** `path()` is consumable **only** by `d:`, and `<select>`/`<input>` have **no child to hang a `<path>` on** while **N-020 forbids wrapping the root** — so native roots take `--glyph-*-url`.

**🔑 CDP PROBE — REAL CLIENT 9222. Chat drove every leg; non-destructive; exact baseline return (38 → 38, 0 probe nodes, root var cleared).** *Every claim below is a measurement. Rule 5.*
- **CSS `d:` works in WebView2.** `<path>` with **no `d` attribute** + `d: path('M5 5h14v14H5z')` → `getBBox()` **14×14 @ (5,5)**, **`getTotalLength()` = 56** (= 4×14, the true perimeter). **The geometry engine, not merely the computed string** — N-097's rule (*the painted pixel is the leg*) applied to SVG.
- **`d: var(--glyph-x)` resolves** through a `:root` token — identical 14×14 / 56.
- **🔑 `var()` RESOLVES INSIDE A CUSTOM-PROPERTY VALUE.** An inline `style="--g: var(--glyph-gear)"` on the `<svg>` root feeds **ONE generic skin rule** `.icon path { d: var(--g) }` → **one rule serves the entire icon system.** The `data-glyph` per-glyph-rule fallback is **dead**.
- **Multi-path + per-path independent fill, from that one rule:** p1 = 14×14 / len **56** / **magenta** · p2 = 20×20 / len **64.72** / **green**. → **multi-colour marks stay `icon`s. D-096 NOT re-opened** (the palette glyph does not become an `image`).
- **A LATER STYLESHEET REDRAWS THE GLYPH — through the indirection.** p1 → diamond: len **56 → 56.57** (= 4·√200, exact) · p2 **untouched** (64.72) · **the inline style unchanged**. ***Whole-glyph theme replacement, on a real `<path>`, with zero component change.***
- **`-url` form from a `:root` token on a native root:** resolved on a real `<select>` (159-char data-URI).
- **`d` attribute + CSS `d:` together → CSS WINS** (attribute still `"M5 5h14v14H5z"`, rendered geometry the triangle). *This is the leg that made the rejected fallback **possible** — and it is exactly why it had to be **rejected on principle** rather than on capability.*

**Three of the five v0.1 open questions DISSOLVED, none of them by argument:** **§3b** multi-colour → stays an `icon` (a mask can *never* do per-path fills) · **§3c** stroke-vs-fill → an ordinary skin property on `.icon path`, **`icon` gains no new prop** · **§3e** the combo triangle → folds, **and the `--tri` duplicate dies as a side-effect.** **§3f provenance** → structural (manifest + build failure). **§3a** → settled: the *"is per-theme glyph replacement a goal?"* framing was **moot — route (B) was already shipped for 13 glyphs, without a decision.**

**✅ THE LAYER SKETCH — Joe: *"pls write this structure sketch in some record, even in some chapter. it is crucial."*** He is right, and grounding showed **why** it is crucial. → **NEW canonical doc `ui/docs/xgen-css-layer-model.md` v1.0** (the *appearance* sibling of `xgen-region-dock-model.md`, which owns *layout*):

```
theme-*.css            ← custom skin. May redefine --accent2 AND --glyph-gear. Identical mechanism.
───────────────────
skin.css               ← default skin, hand-written  ┐ ONE LAYER,
glyphs.generated.css   ← default skin, machine-made ┘ split by WHO WRITES IT
───────────────────
xgen-normalize / modern-normalize   ← reset, not skin
```

**The split between the two default-skin files is TOOLING, not architecture** — *you never mix a generated block into a 98 KB file a human edits live over HMR.* **A theme overrides a glyph exactly the way it overrides a colour. The cascade IS the mechanism; there is no second machinery.** *(Rejected: putting the bank in `app.css` — **three** `app.css` files → triplication; it loads **after** `skin.css` → cascade inverted; and N-031 scopes it to shell chrome.)*

**⚠️ DRIFT 1 FILED — Ch6 §6.2's CSS LAYER ARCHITECTURE IS STALE vs THE CODE.** D-057/D-058 specify `base.css` → `tokens.css` → `skin-dark.css` → `components/`. **Shipped:** no `tokens.css` exists (tokens live in `skin.css`) · `skin.css`, not `skin-dark.css` · and Ch6's **"each `.svelte` file carries its own `<style>` block"** is the **exact OPPOSITE** of **N-025 / N-031 / N-090**. **A D-067 drift surface sitting in a SPEC CHAPTER** — *and it is why a sketch buried in a note would have lost the argument to Ch6.* **Ch6 NOT amended: a spec-chapter touch is a Joe-lock.** Recorded in the canonical doc §4/§6 so the drift is **visible instead of latent**.

**⚠️ DRIFT 2 / TRUST SURFACE FILED — THE SPACE-THEME GLYPH-OVERRIDE BAN.** Under D-108 a theme redraws **any** glyph — **including a lock, a warning, or a verified mark**. **Ch6 §6.3's Layer 3 is a Space theme declared by the SPACE OWNER via a `state.space_theme` EVENT — attacker-supplied CSS arriving over the wire**, in a protocol whose premise is verified identity. **Recommendation: glyph tokens are EXCLUDED from the Space-overridable subset** (app/user themes may redraw glyphs; **a Space may not**). ***And this is not a new decision surface:*** **Ch6 §6.3 already carries the open question *"Which specific CSS tokens may a Space owner override?"***, and Session 1 filed *"Permitted Space theme override token list"* for the second pass. **This supplies the first entry on a list Ch6 already says must exist.** *(The wider question — what else arbitrary Space-owner CSS can do — is Ch6's, not the bank's. Flagged, not solved.)*

**⚠️ Also recorded, not fixed:** **`theme-*.css` does not exist yet.** Ch6 §6.3's three-layer cascade is **specified but unbuilt**. What is locked is that **the bank is SHAPED so a theme layer can override it when it lands** — **no milestone may claim theming works.** And **`DECISIONS.md` has drifted from newest-first** (D-089, D-091, D-094 and **all** of D-099–D-107 are appended at the *bottom*); D-108/D-109 **follow the de-facto shipped pattern** rather than silently re-sorting a 4,076-line record. **Flagged, not fixed.**

**Records.** **NEW `ui/docs/xgen-css-layer-model.md` v1.0** · `docs/xgen-icon-adoption.md` **v0.1 → v1.0** (§1 inventory **corrected to the measured 21**; §3 settled by measurement) · `ui/docs/xgen-ui-notes.md` **v0.76 +N-101** · `DECISIONS.md` **+D-108** (the glyph bank) **+D-109** (the Chromium/WebView2 `d:` platform dependency — *taken deliberately, named, and NOT hedged; the WebKit exit is the `-url` form the bank **already emits**, so a port is a renderer swap, not a rewrite — the D-103 one-source-two-renderers shape again*) · `docs/ROADMAP.md` v4.73 · `CLAUDE.md`.

**No code moved.** M-RP-ICON-ADOPT stays **gated behind the frame arc — it does not jump the queue.** Its Phase-0 is now **classification + provenance, not re-litigation**: classify all 21 (fill / stroke / multi-colour / native-root), licence-source every one, lock the manifest record + generator contract.

---

## Entry J-503 — surfaces §6.3 CLOSED: what goes into a UI state; scroll position REFUSED; the read-marker protocol gap FILED — design/records-only, no code

**Design + records only. No code. Registry unchanged (client 38, sampler catalogue 313).** Third §6 item closed. **And for the second session running, grounding turned the questions into *different questions than they were asked as*.**

**The test that now decides every future candidate (Joe-locked):**

> **Would you expect it to follow you to another device?** → **NOT UI state** (it is config, or it is protocol).
> **Does it describe *where things sit on screen*, rather than *what you chose*?** → **UI state.**

*Grounded, not assumed: the real `xgen-client_config.toml` carries `node` · `keypair_path` · `logging` · `sync` · `substitutions` — **user intent, zero presentation**. §4.4's config-vs-UI-state split holds.*

**✅ IN:** grid layout (tiles + split sizes) · shelf favourites · window geometry (clamped to the work area) · **collapsed/expanded** panel states · **last open space+room** · **theme**.

**⚠️ Two of those carry conditions, and both conditions came from grounding:**
- **Last open space+room is SESSION state only, and it references PROTOCOL objects** (`SpaceXgid`/`RoomXgid`) → it **needs a reconcile rule**: room gone / left / kicked → **fall back to no room, never crash**. That is the layout's unknown-`widgetId` drop **one level up** — and per **N-095**, **the fallback is EXERCISED, not asserted.**
- **🔑 “Theme” is NOT a free key — Ch6 already specifies a THREE-LAYER theme system.** An **application theme** (dark/light, *operator-configurable*, **D-057**, layered `base.css` → `skin-dark.css`) **and a SPACE THEME declared by the Space owner via a `state.space_theme` EVENT**, overriding only a defined token subset. **Resolution: app default → user choice → Space override.** So the UI-state key is **the user-choice LAYER only**, per-device — calling it *“the theme”* would collide with a protocol event. *(Second session running in which a UI question turned out to already have a protocol answer.)*

**❌ SCROLL POSITION — REFUSED, and NOT because it is hard. It is the WRONG HOME.**

It is **four different things**, and only one is sound: a **pixel offset** · a **ratio** · an **anchor** (an `EventXgid` at the top of the viewport) · the **unread boundary** (a *different concept entirely*).

**A pixel offset is meaningless in THIS stream, for five shipped reasons:** **prepend** (loading older history shifts everything — `message-stream` already compensates *live* with a `scrollHeight`-delta anchor, **J-485**, which is itself proof the offset is unstable **within** a session) · **edit/delete** (a tombstone is a different height) · **grouping recompute** (`grouped` rows + day-dividers change heights; `computeRows` is `$derived`) · **window resize** · **⚠️ and the killer — on relaunch the same messages are not even loaded**: if the client opens with the last N messages and you were 300 back, `scrollTop = 1400` **points at nothing**.

**→ Restoring scroll across a relaunch is a BACKFILL problem, not a UI-state problem:** load history **until the anchor event is in the DOM**, then scroll to it — pagination + sync work, **not a JSON key**. **A stored number would create the illusion of a feature and deliver a wrong scroll**, and it would **fight shipped code** (`message-stream` mounts to bottom **unconditionally**; any restore is an explicit override of a closed, verified machine).

**✅ What IS legitimate, and where it belongs:** **in-session, per-room scroll memory** (A → B → A keeps your place) = an **in-memory `Map<roomId, anchorEventId>`** — no file, no protocol, no persistence, **anchored on an event id even in memory** (prepends shift offsets mid-session too). **Ships with M-RP6.2**, not with the UI-state store. Across-relaunch restore stays **deferred** (needs anchor + backfill-until-found + an **LRU cap** — one entry per room ever visited grows without bound).

**⚠️ NEW FILED GAP — READ / UNREAD MARKERS HAVE NO PROTOCOL MECHANISM.** **Ch6's UI already renders unread counts** (`RoomListItem` = *“Room name, last message preview, **unread count**”*; the Space list carries one too) — **and there is NO read-marker event in the protocol.** Grepped **Ch3 and Ch6**: nothing.

**A read marker is per-identity state a user expects to FOLLOW THEM TO ANOTHER DEVICE**, so by the test above it is **not UI state**. Persisting one in `xgen-client_uistate.json` would ship a **local-only marker that never syncs**, and when a protocol read-marker eventually lands there would be **two sources of truth — D-067 drift, self-inflicted.**

**🔑 And it is what users actually want on relaunch.** Slack and Discord do **not** restore a scroll offset — they restore you to the **unread line**. *That is the evidence: the real problem is the unread boundary, not the pixel — **a protocol gap, not a UI one**.* **This is the same species as the J-502 temperature find: a UI chapter drawing a thing the spec never gave it a mechanism for.** → **Filed as a protocol question. No UI milestone may fake it, and none may quietly persist a local marker to make an unread badge light up.**

**✅ §4.1 AMENDED — the session-vs-named line, stated once:** **a named UI state carries the ARRANGEMENT** (layout · shelf · geometry · theme) and **NOT the open room**. *“Reading” is a **workspace**, not a **place** — Maya restores your panels, not your scene.*

**Records.** `docs/xgen-widget-surfaces-phase0.md` **v1.3** — **§4.5 SETTLED** (the test + the IN/OUT table) · **§4.6** (scroll refused, with the four-candidate breakdown) · **§4.7** (the read-marker gap) · §4.1 amended · §6 item 3 **CLOSED**. `docs/ROADMAP.md` (the read-marker gap filed) · `CLAUDE.md`. **No new D.**

**§6 status: items 2 + 3 CLOSED · item 1 PARTLY closed (the registry half) · STILL OPEN — Settings' own surface (blocked on the §9 taxonomy Phase-0) · top-shelf pinning · glyph provenance. M-RP6.1i–l remain gated.**

---

## Entry J-502 — Vocabulary lock (tile / region / face / slot) + surfaces §6 partial lock; the plugin taxonomy gap FOUND — design/records-only, no code

**Design + records only. No code. Registry unchanged (client 38, sampler catalogue 313).** A §6 walk of `docs/xgen-widget-surfaces-phase0.md` that produced **more corrections than answers** — which is the point of a Phase-0 gate (D-071).

**🔑 THE SESSION'S REAL OUTPUT: A CONCEPT I "RE-DERIVED" WAS ALREADY SPECIFIED, COMPLETE, AND SHIPPED.**

Walking §6.2 (`temperature-indicator`'s identity), Chat reasoned from first principles that heat must be *“a decaying accumulator the widget computes”*, that it *“must never be a protocol field”*, and that per-member heat raised *“a moral question Joe must decide”*. Joe's reply was one line: **“we have already written it — look in the chapters.”**

**Every one of those three claims was false, and the truth was in the tree:**

1. **Temperature IS a protocol property.** Spec **§3.7.13, Status: complete** — reserved `meta_atts` keys `xgen.room_temperature` / `xgen.member_temperature` (floats `[0,1]`), a threshold table, buckets `cool|warm|hot|fiery`.
2. **The client computes NOTHING.** Ch6 **§6.12.1**: the values are **opaque** — *“the client does not know how they were computed, does not attempt to re-derive them, and treats them as authoritative.”* The math is a **plugin on the Room's home Node**, deliberately outside the protocol (**D-061**).
3. **The “moral question” was already answered by the protocol.** §3.7.13.3 ships **`member_temperature_visibility: moderator | everyone | self_only`** on Space state. And member heat is **accumulated overpass of the Space's own pacing rules** (§3.7.12 / **D-060**) — a measure against a rule the Space set for itself, **not** a reputation score. *A materially better design than the one Chat was worried about.*

And there is **shipped Rust on both sides**: `xgen-node/src/plugins/temperature.rs` (the `TemperaturePlugin` trait + `NoOpTemperaturePlugin`) and `xgen-client/src/temperature.rs` (`TemperatureUpdate`, the `__room__` sentinel, bucket derivation, a `temperature_update` Tauri event, **and a DOM contract**: `data-temp-state` + `--xgen-room-temperature` / `--xgen-member-temperature`).

**⚠️ This is Chat's SIXTH grounding miss of this arc and by far the worst.** The others were details inside a runbook (J-497 ×3, J-498, J-499, J-500, J-501). **This one nearly had Joe lock a design decision that contradicts a *complete* spec section, a shipped decision (D-061), and two shipped Rust modules.** The lesson is sharper than *“read the source”*: **a concept I cannot remember is not a concept that does not exist — and “let me re-derive it” is not grounding, it is invention wearing grounding's clothes.** Joe caught it by knowing his own documents.

**✅ §6.2 CLOSED (Joe).** `temperature-indicator` is **a RENDERER of an existing protocol property, not a computation**: a `$common` store fed by the existing `temperature_update` event + the shipped `data-temp-state` skin contract, rendered as **CONTENT inside other widgets** (room heat → R2/R4; member heat → R7/message rows). **Content inside a host is not a surface (§3.2) → it spends NO surface and is NOT a dockable panel.** **The `meter` + W-11-dd-socket framing is WITHDRAWN, not carried forward** — it predates Ch6 §6.12, the widget tier, the dd-socket and the region model. *(Joe: it was **“the first bird”** — named before any of the conventions it was described in existed. **The deferral is why it never hardened wrong.**)* **Gate: live messaging (M-RP6.3 + R5 live) AND a non-no-op node plugin** — `NoOpTemperaturePlugin` returns `None` today, **so there is literally nothing to render**. *That second half is a node/plugin arc — the M-RP6.6 shape again: **a UI milestone cannot manufacture a source that does not exist**.* **Binding: no milestone reserves a heat slot or reads a heat store.** Three stale records **corrected in place, not deleted** (widget-tier §6 · dd-entity-avatar-phase0 · components registry).

**🔑 VOCABULARY LOCKED (Joe) — four words, four things, and we had been using them interchangeably.** Joe: *“you start to use word region systematically, i used it un-systematically”* — then proposed **tile**, and it turned out to be the **missing** word rather than a better synonym:

| term | what it is |
|---|---|
| **tile** | **a PLACE** — a box in the grid; one `leaf` in the D-103 descriptor = one tile |
| **region** | a widget's **full CONTENT surface**, occupying a tile — **it names WHICH widget, not where** |
| **face** | a widget's **compact HANDLE** on a shelf: icon + badge + a `commandId` click (S-4/S-7) |
| **window** | its own OS window |
| **slot** | **Ch6 §6.8.3's named, FIXED attachment point** — a **different placement model**; do not merge |

**`region` is IDENTITY, not location — and the shipped code settles it:** `region-node.svelte` mounts `<W regionId={node.widgetId} />`, so **`regionId === widgetId`**. R3 *is* Self/connection wherever it is docked. **The sentence that proves tile ≠ region:** *a `tabs` node is **ONE TILE holding SEVERAL REGIONS*** (renderer B) — unsayable if they are synonyms. **And `face` is NOT “the static one”** (Chat nearly agreed to that, and it is wrong): S-4/S-7 make a face **a button with a live badge**. **The axis is purpose, not liveness** — a region **IS** the widget rendered; a face is a **handle to** it. *That distinction is load-bearing: §3.3's “one surface per widget” means **a face implies NO tile**, and a “static vs interactive” reading would quietly erode it.* → **N-100**; canonical table = region-dock §0.

**✅ §6.1 PARTIALLY CLOSED (Joe) — the registry half.** **The plugin list is ONE PANE with TWO ENTRY POINTS**: it lives in Settings' structure, and the **bottom-shelf gear opens the same pane** — **not** a cut-down twin. *(That is **S-2's “one component, two mounts”** + **S-7's one-dispatch-two-triggers**, applied to the manager; a simplified popup carrying the destructive buttons with the least context is exactly what **S-6** exists to prevent.)* **NAME = “plugin list”** — Ch6 §6.8.5 calls it the *Module List*, surfaces called it the *Widget Manager*; **same object**, and *plugin* is the better word because it covers **headless** plugins with no UI at all. *(Joe: **“module and widget is the plugin in the two areas: system and ui”**.)* **Built-ins are listed, distinguished, and NOT removable** — and **Ch6 §6.8.5 already drew that**: a `[system]` / `[user]` **mode badge** on every entry. **That is W-13, pre-figured in Ch6 before the widget tier existed.** **Still open: what surface *Settings itself* gets.**

**⚠️ NEW OPEN ITEM — THE PLUGIN TAXONOMY GAP (filed; gates M-RP6.1l).** Grounding Ch6 §6.8 exposed a collision **in the specs, not the code**: **Ch6 §6.8.3 already defines three Module UI Forms — Headless · Widget · Window** — and **Ch6's “widget” is NOT D-102's widget.** Ch6's is **HTML in an isolated webview**, talking to its module backend over a **local WebSocket**, placed by a **named slot** from a fixed inventory (`room.sidebar.bottom`, `global.statusbar`, …). D-102's is a **Svelte component**, in-process, fed by a **`$common` store**, placed by a **dockable region** in the D-103 descriptor. `self-panel` / `inspector-panel` are the latter: **no webview, no socket, no slot, no manifest.** **→ There are TWO PLACEMENT MODELS in the project**, and surfaces-Phase-0's `region | shelf | window | none` is — in hindsight — **a re-derivation of Ch6 §6.8.3 made without consulting it** (they agree on *headless* and *window*, and diverge **exactly where placement lives**). **This is a D-067 drift surface sitting in the specs.** It does **not** block **M-RP6.1i/j** (the `shelf` core + mounts are pure UI); it **does** block **M-RP6.1l** (the plugin list must list both species) and **M-RP7.4**. **Joe's reconciliation frame is the right one — one plugin, one list, several UI forms — so the work is ALIGNMENT, not a choice between them.** → filed as a **taxonomy Phase-0** spanning **D-036 / D-102 / D-103** (region-dock §11 · surfaces §9).

**Records.** `ui/docs/xgen-region-dock-model.md` **v1.6** (**§0 vocabulary — LOCKED** · **§11** the taxonomy gap) · `ui/docs/xgen-ui-notes.md` **+N-100** · `docs/xgen-widget-surfaces-phase0.md` **v1.2** (§6.1 partial · **§6.2 CLOSED** · **§9** new open item) · `ui/docs/xgen-widget-tier.md` + `docs/xgen-dd-entity-avatar-phase0.md` + `ui/docs/xgen-ui-components.md` **v0.71** (the withdrawn `meter`/dd-socket mechanism, corrected in place) · `docs/ROADMAP.md` · `CLAUDE.md`. **No new D** (D-036 / D-102 / D-103 / D-107 extension) — *and a D-number for the vocabulary is Joe's call, deliberately not taken unilaterally.*

**§6 still OPEN — M-RP6.1i–l stay gated:** Settings' own surface (← needs the taxonomy Phase-0: the vocabulary **cannot express Ch6's “a screen of its own”**) · §4.5 UI-state contents · top-shelf pinning · glyph provenance. Also open: **M-RP6.6 client resident** · M-RP6.1e-B1 · M-RP7.x · M-RP7.3 (N-095) · M-RP8 · M-RP-ICON-ADOPT.

---

## Entry J-501 — M-RP6.1h R8 Selection info: the selection bus's first CROSS-REGION reader; **M-RP6.1h CLOSED** — the read loop closes end-to-end

**Doc-bridge (D-074 second commit).** Clair's feat is already pushed = commit `c4346bf` (code-only, **3 files, +143**); this entry + the paired canonical records = commit 2. **M-RP6.1h CLOSED.**

**The loop closes.** R3 writes `{regionId, entity}` → **R8 renders that entity's rows**. The selection bus, which had no reader at all at 6.1f and only a self-reader at 6.1g, now has its **first cross-region reader** — and the strongest proof of that is not the R3 click at all (see V4).

**What landed (3 files).** `ui/common/lib/components/widgets/inspector-panel.svelte` (**new**, 101 lines) — the **second real system widget** (`kind: system`, W-13; **4th widget** overall). `ui/client/src/layout-default.ts` (+2) — `widgetRegistry.inspector → InspectorPanel`, **one map entry**. `ui/assets/skin.css` (+40) — all `.inspector-*` appearance (N-090). **No Rust. No `ui/core/**`. No `ui/node/**`. No sampler.**

**🔑 THE MILESTONE IS THIN BECAUSE THE DATA IS THIN — AND THAT IS THE DESIGN.** `EntityDescriptor` is `{kind, name?, id, flags?, image?}`: five fields, three optional, `image` **reserved-unfed**. An "inspector" over that is a small thing. The design walk **refused** to make it look bigger: a `get_entity_info`-shaped verb was **rejected outright**, because the only selectable entity today is *self*, whose fields already come from `get_self_state` — so a second Rust projection would have been a **second surface (D-067 drift) delivering zero new information**. **Do not invent fields to make a panel look substantial.** When R1/R2 land real spaces and rooms, a richer read can earn its keep *then*.

**Shipped shape.** Rows render as the About `<dl>`: keys as plain `<dt>`, **values as `Label`** (registry-visible) — **Kind · Name · ID · Source**, with **Flags conditional**. `Source` is the bus's own `regionId`, and it is the field that makes R8 *visibly* a cross-region reader. Empty state = the `entity-panel` pattern verbatim (a composed `Paragraph` at `__empty`), root + `section` **always mounted** — which is what makes *clear → exact return to baseline* the honest orphan proxy (N-092a: the client bridge is state-only; there is no `domCount` leg). Getter G `{hasSelection, regionId, kind, rowCount}` — **the XGID and name are deliberately NOT republished** (already readable on the composed children's getters, the N-060 `hasValue` precedent); `rowCount` is **render-truth**, which is what makes the conditional flags row observable. `id = region-${regionId}` (N-096) → a clean **swap in place**. **Zero component `<style>`.**

**🔑 THE N-097 TRAP WAS DESIGNED AROUND, NOT WALKED INTO.** R8's header composes **`entity-avatar`, deliberately NOT `entity-item`** — `entity-item` carries a `selected` prop with a live `[data-selected]` skin rule, and in R8 "selected" is **trivially always true**. That is a constant-valued flag feeding a shipped affordance: *exactly* the shape that stranded `.entity-item[data-selected]` at 6.1g. **One milestone after learning the lesson, the design routed around it.** The avatar renders the same five fields more richly (kind → circle/square/hexagon, name, id → seed colour) — **no new data, no new verb**. *(Also grounded by Clair before build: `variant="labeled"` draws shape + seed + initials only, `<figcaption>` reserved-unused — so the **Name row is not a duplicate**.)*

**CDP verification — REAL CLIENT 9222 (D-097; the sampler has no frame and no bus — it structurally cannot host this). Chat re-drove EVERY leg itself (Rule 5).**
- **V1 registry 36 → 38**, `count === unique === 38`, **enumerated**. `section#region-inspector` **out**; in: `inspector-panel#region-inspector` + `section#region-inspector__section` + `paragraph#region-inspector__empty`. Net **+2**.
- **V2 empty state:** G `{hasSelection:false, regionId:null, kind:null, rowCount:0}`, `__empty` present, **zero** row ids.
- **V3 — 🔑 THE LOOP, AND THE LEG IS THE PAINTED TEXT.** Click R3's real `entity-item` → bus `{regionId:"self", entity:{kind:"identity", id:"xgen://pubkey/ed25519:VtLICf…KHGc", name:"Joe"}}` → the rendered `<dd>` text nodes read **`identity` / `Joe` / the full XGID / `self`**, compared **against the bus payload** (`ddMatchesBus: true`). G `{hasSelection:true, regionId:"self", kind:"identity", rowCount:4}`; registry **42**. **R3's own gold `[data-selected]` bar still paints** (`box-shadow: rgb(154,106,48) 2px 0 0 inset`) — no regression. *(N-097, honoured: a getter field is not a render.)*
- **V4 — R8 READS THE BUS, NOT R3, AND THIS IS THE STRONGER PROOF.** Driving `__XGEN_SEL__` directly: **DM space → circle** + `isDm`, `rowCount 5`, registry 43 · **room → hexagon** (the clip-path polygon really paints), `rowCount 4`, registry 42 · **non-DM space → square** + `e2e`, `rowCount 5`, registry 43. `Source` flips `spaces`/`rooms` each time. **These are regions with NO WRITER AT ALL** — R8 renders them anyway, which the R3 click alone could never have proved. This leg also **exercises the conditional flags row in both directions** (present ↔ absent), so D4's branch ships **verified, not merely reachable**.
- **V5 churn → baseline:** `clear()` → **38 exactly**, the enumerated id set **identical to V2's**, 0 rows, `__empty` back, bus `null`. **Zero leaked registrations across five selection churns.**
- **V6 geometry (the N-091 required leg):** `docNoScroll` true (864 === 864) · members/inspector **409 / 409 = ratio 1.000** against `sizes [1,1]` · the leaf `overflow-y:auto` **self-scrolls to 500 under a 4000px injection while the document stays at 0** · restored, registry 38.
- **V7 skin:** **7** `.inspector-*` rules in cascade (stylesheet-rule inspection, N-042) · **zero component `<style>`** (grepped the **file**, not the DOM) · **accent-neutral** — `dt` `rgb(138,136,128)`, the avatar's seed fill and the value `label` all **byte-identical** under an injected `--accent2: #ff00ff`, then restored.
- **V8 static, re-run by Chat with the apps DOWN** (target-dir contention): `vite build` → **158 modules** (156 → 158) · `npm test` → **41 passed** (3 files) · `cargo test --workspace` → **exit 0 · 1507 passed · 0 failed · 62 ignored** — **identical to the J-500 baseline, which is the no-Rust claim proven rather than asserted** · `git show --stat c4346bf` → **3 files, +143**, the runbook §1 list exactly.

**⚠️ TWO SELF-CAUGHT DEFECTS — both Chat's, both recorded rather than smoothed over.**

1. **The runbook's V4 literal was WRONG (Chat's), and Clair caught it.** §5 asserted that `{kind:'space', flags:{isDm:true}}` renders a **square**. **Grounded against the shipped source** (`entity-avatar.svelte:56-62`) rather than accepted on her word: `room → hexagon` · `space && !isDm → square` · **else circle** — the comment says it outright, *"DM space = circle (people-shaped)"*. **A DM space is a circle.** She drove all three cases anyway, so both sub-claims (shape-flip **and** the D4 flags row) are honestly proven and the milestone is unaffected — **only Chat's label was wrong.** *(A runbook grounding miss, again. The hedge that keeps catching these is the runbook's own instruction: **if the code contradicts me, the code wins and you flag it**.)*

2. **⚠️ CHAT PRODUCED A PHANTOM GREEN AND THREW IT AWAY — this is the milestone's real method lesson.** The first V7 accent probe **set the bus and read the DOM in the same eval**. Svelte flushes on the **effect tick**, so both reads returned **`null`** — and the comparison then reported **`accentNeutral: true` by comparing `null === null`**. **A leg that cannot see the thing it is testing did not pass it.** Re-driven across two evals with a `readable: true` assertion **first**, so a null can never masquerade as a match; the values came back non-null and identical. The same trap produced a spurious `count: 42` immediately after `clear()` — re-read separately → **38**. **Neither phantom entered this record.** → **N-099**.

**Deviations from Clair — flagged, not absorbed (Rule 6).**
- **Her numbers reproduced exactly this time** (38 / 42 / 43 / 38, ratios, accent-neutrality) — the first handback this arc that did. Recorded because the three that did not are also recorded.
- **Her process miss, self-reported:** she overwrote the untracked `inspector-panel.svelte` with a `Write` before reading it. `git diff HEAD` confirms the committed result is the minimal verified 3-file scope, so nothing recoverable was lost — a **session-open discipline note**, not a defect. Logged because a silent overwrite is how a project loses work that *was* recoverable.
- **`.about-grid` and `.inspector-grid` are now two kv-grids.** **No `.kv-grid` extraction** — this is the **second** recurrence and D-069's bar is **four**. Flagged for the fourth, deliberately not acted on.

**Records.** → **N-099** (the same-tick phantom green: a set/click and its DOM read must be **separate evals**, and a leg must assert it can **see** its subject before it compares — a `null === null` match is a false pass). Components registry **v0.70** (`inspector-panel` = the **4th widget**, **2nd system widget**; **not** a `core` catalogue cell; **sampler catalogue unchanged 313**). `docs/xgen-client-frame-phase0.md` §6 (6.1h ✅). `ui/docs/xgen-region-dock-model.md` §5 (**the bus has a cross-region reader**). `docs/ROADMAP.md`. Runbook `tasks/M_RP6_1H_INSPECTOR_PANEL.md` → **COMPLETED**. **No new D** (D-103 / D-107 extension).

**M-RP6.1h CLOSED. The R3 → bus → R8 read loop is closed end-to-end.** **Next: `docs/xgen-widget-surfaces-phase0.md` §6 — 5 items open for Joe (sharpest = Settings' own surface). Nothing in M-RP6.1i–l can start until it locks.** Still open: **M-RP6.6 client resident** (the real F-1 close — Rust, and no part of it enters a UI milestone) · **M-RP6.1e-B1** no-select chrome · **M-RP7.x** node frame inheritance · **M-RP7.3** (N-095's corrupt-layout fallback, **exercised not asserted**) · **M-RP8** title-bar + frameless · **M-RP-ICON-ADOPT** · `temperature-indicator` ⏸️.

---

## Entry J-500 — M-RP6.1g R3 Self/connection: the first real system widget + the selection bus's first writer; **M-RP6.1g CLOSED**

**Doc-bridge (D-074 second commit).** Clair's feat is already pushed = commit `84b482a` (code-only, **7 files, +302/−38**; amended from `967ad51` to carry the post-V5 fix — `967ad51` is orphaned, every record reads `84b482a`); this entry + the paired canonical records = commit 2. **M-RP6.1g CLOSED.**

**🔑 THE MILESTONE'S REAL OUTPUT WAS THE GROUNDING PASS — IT SHRANK THE MILESTONE.** The design in `docs/xgen-client-frame-phase0.md` §6 said *“a `get_self_state` read verb + a scoped `app.emit('self-state', …)` push → closes the F-1 read half.”* **Reading the source before writing the runbook proved three of that sentence's assumptions false**, and all three corrections made the work *smaller and more honest*:

1. **The push ALREADY SHIPS.** `xgen-client/src/desktop.rs::emit_state()` has been firing `app.emit("xgen-client-state-changed", …)` on **every** lifecycle transition, and `app_client.svelte` has been `listen`ing to it and feeding the `status-bar`, since before this arc opened. **A second channel would have been a second surface — the exact D-067 drift this project exists to eliminate. NO NEW EMIT WAS BUILT.**
2. **`ops::whoami` ALREADY EXISTS** (sync, no network, reads `xgen-client_state.json`), and **`session::ClientIdentity::load(keypair_path)` derives the identity XGID from the keypair alone** — no registration, no node. So `get_self_state` is a **thin shell wrapper** over two existing readers (the `get_about_info` shape), **not a new projection**.
3. **⚠️ *“Stop the node → the led flips”* IS NOT RUNNABLE — and saying so is the most valuable line in this entry.** `run_startup` does **one** 2-second `connect_async`, **drops the stream** (`Ok(Ok(_stream))`), emits `Ready`, and never touches the socket again. **There is no resident.** Killing the node changes nothing in a running client. The proof the Phase-0 promised **could not have been run**, and a less careful close would have quietly substituted a weaker one and called it F-1.

*(Grounding the code before writing the runbook is now the **fourth consecutive milestone** where it changed the design — J-497, J-498, J-499, this.)*

**→ Consequence, recorded not smoothed over: 6.1g closes the *read shape* of F-1, NOT its live half.** The live flip needs a sustained WS + reconnect loop, which is a Rust/protocol arc, not a UI one. **Filed as M-RP6.6 — client resident.** *That* is the real F-1 close. **Nothing of it was smuggled into this milestone.**

**What landed (7 files).** `xgen-client/src/desktop.rs` — `SelfStateInfo` + the thin `get_self_state` command (composing `ClientIdentity::load` + `ops::whoami`), registered in `invoke_handler`; **`NodeSelfState` deliberately NOT declared** (the J-497 `NodeAboutInfo` precedent — a node wrapper today would guess fields with no call site to validate against). `ui/common/lib/stores/self-state.svelte.ts` (**new**) — the one channel, two views. `ui/common/lib/components/widgets/self-panel.svelte` (**new**) — the **first real system widget**. `ui/common/lib/stores/selection.svelte.ts` — the W-8 *“no writer”* note **retired**. `ui/client/src/layout-default.ts` — `widgetRegistry.self → SelfPanel`. `ui/client/src/app_client.svelte` — the shell writes the store, the `status-bar` reads it, the local `currentState` mirror + local colour maps **removed**. `ui/assets/skin.css` — 2 `.self-panel*` rules (**N-090: layout included**).

**🔑 THE STORE RELOCATION WAS FORCED, NOT STYLISTIC — and it is the structural lesson.** `region-node` mounts a leaf with **`regionId` and nothing else**: **a widget cannot receive shell props.** Combined with **W-3** (a `common` widget must never import a shell dep), that means **everything a region widget needs must be store-mediated — there is no other channel.** So the shell's `STATE_COLOURS` / `PULSING_STATES` maps **had to** move into `$common`, where the `status-bar` (shell chrome) and the `self-panel` (widget) now read the **same** map. **One map, one signal, two views.** This is not a tidy-up; it is the shape every future region widget inherits. → **N-096**.

**Leaf-id convention (Chat's runbook amendment, applied):** the widget derives `id = \`region-${regionId}\``, so R3 registers as `self-panel#region-self` with children `region-self__section` / `__item` / `__item__avatar` / `__status` / `__status__led` / `__status__label` — the **same** convention the seven remaining placeholders use. The registry delta reads as a **clean swap in place**, which is exactly what renderer A's prop-injected registry was built for (J-499): **one entry replaced, no rewrite.**

**Honest by construction (D-065 / W-8), and it turned out to be the strongest proof available.** The Tauri shell **has never registered** — `xgen-client_state.json` did not exist (measured on the real machine). So the correct render is `registered:false` + the **real keypair-derived XGID** + an explicit *not registered* line. **No fake name, and the Track-A `status` slot ships ABSENT, not faked** (no client-side read path for `state.status/<xgid>` exists at all).

**CDP verification — REAL CLIENT 9222 (D-097; the sampler has no `get_self_state`, no protocol deps, no frame — it structurally cannot host this). Chat re-drove EVERY leg itself (Rule 5).**
- **V1 registry 30 → 36**, `count === unique === 36`, enumerated. `section#region-self` **out**; in: `self-panel#region-self` + the six children above. Net **+6**.
- **V2 / V3 — the same verb, two grounded outcomes, and Chat re-drove V2 *without relaunching*:** `xgen-client_state.json` was moved aside (`whoami` re-reads the file per invoke), leaving the **same process, same keypair** → `{registered:false, identity_id:"xgen://pubkey/ed25519:VtLICf…KHGc", display_name:null, home_node:null, spaces_joined:0}`; restored → `{registered:true, identity_id:**identical**, display_name:"Joe", home_node:"ws://127.0.0.1:8080/xgen", spaces_joined:0}`. **Only the file moved** — which proves the keypair path and the state-file path are genuinely independent, and proves the two XGID sources **agree** (Clair's mismatch `warn!` correctly never fires). *(A one-shot leg was recovered as a repeatable one — the file, not the process, is the variable.)*
- **V4 one channel, two views:** node **up** → bar led AND panel led both `{READY, var(--ok)}`, both labels `"Ready"`, store `{state:"READY"}`; node **down** → both `{DISCONNECTED, var(--err)}`. **One store drives both.** `entity-item` G reports `hasStatus:false` — the Track-A absence is **render-truth**, not a claim.
- **V5 — the selection bus has its FIRST WRITER** (and, after the amendment below, its first reader): `clear → null` → click the **real** `entity-item` → `{regionId:"self", entity:{kind:"identity", id:"xgen://pubkey/ed25519:VtLICf…KHGc", name:"Joe"}}` → `clear → null`.
- **V6 geometry (the N-091 required leg):** `docNoScroll` true (864===864) · panel fills the leaf width · leaf `overflow-y:auto` self-scrolls while the document does not · rooms/self column **613 : 204 = 3.005** against `sizes [3,1]` · restored.
- **V7 relocation purity:** bar led **rgb(45,122,58)** === panel led **rgb(45,122,58)**. A cross-file move *should* be a no-op, and *“should”* is not a verification (N-090).
- **V8 skin:** 2 `.self-panel*` rules in cascade; **zero component `<style>`**.
- **V9 churn → baseline:** **36 → 29 → 36 exactly.** `droppedCount:1`, `leafCount 8→7`, **zero `self` ids leak**, bus clean. *(N-092a: the orphan leg is **not expressible** on the client — the debug bridge is state-only. The client-expressible proxy is the exact return to baseline, N-095b.)*
- **V10 static, all re-run by Chat with the apps down (no target-dir contention):** `cargo test --workspace` → **exit 0 · 1507 passed · 0 failed · 62 ignored** · `npm test` → **41 passed** · `vite build` → **156 modules** (150→156) · `git show --stat 84b482a` → **7 files, +302/−38**, the runbook §3 list exactly, **no `ui/core/**`, no `ui/node/**`, no sampler**.

**⚠️ THE MILESTONE'S SECOND LESSON — V5 FOUND AN UNFED BRANCH, AND IT WAS OURS.** The first V5 pass showed `entity-item.selected: **false** while the bus carried that exact entity`. The panel **wrote** the bus and never **read it back**, so `selected` in its getter was a **constant false** — an unfed field shipped in a milestone about to close. Worse, grounding `skin.css` showed **`.entity-item[data-selected]` already existed (line 2105, shipped at M-RP5.1)**: a **skinned affordance no client code could reach.** **This is the same rule we used to keep the `tabs` branch out of renderer A and to refuse the unreachable null-layout guard — and we do not get to invoke it against others and exempt ourselves.** Fix (Joe-locked, `self-panel.svelte` only, **no skin change** — the rule was already there): `const selected = $derived(selection.current?.entity.id === descriptor.id)`, passed to `entity-item` and published in G. **So the self-panel is the bus's writer AND its first reader.** Re-verified by Chat: cleared → `false`/`[data-selected]` absent → click → `true`/`true`/`[data-selected]="true"` **and the affordance actually PAINTS** (`box-shadow: rgb(154,106,48) 2px 0 0 inset` — the gold `--accent` bar; background lifts `rgb(28,31,36)` → `rgb(42,47,56)`) → cleared → all revert; registry **36** throughout. **Feat amended `967ad51` → `84b482a`.** → **N-097**.

*Note for the record: a click that mutates a store and moves **nothing** on screen is not a working writer — it is an untested one. **The painted pixel is the leg**, not the attribute (N-091's shape, a third time).*

**Deviations — flagged, not absorbed (Rule 6).**
1. **Runbook §5.2's prop shape `{regionId, id}` was WRONG — Chat's error.** `region-node.svelte:36` passes a leaf **only** `regionId`. Clair grounded it (§5.2 had explicitly hedged *“ground what `region-node` actually passes”* — the hedge worked). Resolved to the `region-${regionId}` derivation above.
2. **`tracing::warn!` over `debug_assert!`** for the keypair-vs-cached XGID mismatch (§4.1 said *“a DEV assert”*). **Clair's call is better than the runbook's:** a panic on a legitimate stale-state divergence turns a diagnostic into an outage. §4.1's actual requirement was *“flag it, don't paper over it”* — a `warn!` **is** flagging it. **Never fired** (V3: the two sources agree).
3. **Empty `display_name`/`home_node` → `None` at the Rust boundary.** A projection choice the runbook did not specify. **Confirmed safe:** `registered` is the *state-file-exists* fact and is set **before** the field filters, so an empty name cannot flip it.
4. **`panelClasses:"self-panel self-panel"` (doubled)** — matches the `substitutions-editor` precedent (`mergeClasses` does not dedupe). No CSS effect. **Not a defect**, recorded so it is not re-found.
5. **⚠️ Clair's V9 number does NOT reproduce — conclusion right, evidence wrong. THE THIRD TIME THIS ARC.** Her handback reported *“36 → **23** (self dropped) → 36”* twice. **Chat measured it: dropping the `self` leaf is 36 → 29**, with `droppedCount:1`, `leafCount 8→7`, and zero `self` ids leaking. **23** describes a different, larger churn. **Her conclusion stands** (restore → exactly 36 — reproduced); **her intermediate number does not enter this record.** *(J-499's lesson, verbatim: a canonical record does not inherit unverified numbers. This is precisely why Chat re-drives.)*
6. **The client was closed before the post-amendment V5 could be re-driven**, so Chat **stopped and said so** rather than closing on Clair's numbers; Joe relaunched and every leg above is Chat-measured. **A record that is *probably* true is not the standard.**

**Method notes.** `get_self_state` / `get_state` `invoke` carries **~500–800 ms IPC latency in dev** — the two-step CDP fire→read needs a longer settle than 400 ms. Single-expression `JSON.stringify({…})` evals only (PS 5.1). **Amend-after-push gotcha:** amending an already-pushed commit made GitHub Desktop force-update the remote and then start a **stale merge of the orphaned SHA**, producing a phantom conflict; `git merge --abort` was the fix (the remote was already correct).

**Records.** → **N-096** (the first real system widget; the **forced** store relocation — a widget can receive nothing but `regionId`, so W-3 makes store-mediation the *only* channel; the `region-${regionId}` leaf-id convention → a clean swap-in-place) · **N-097** (a bus **writer** with a matching skin affordance **must read the bus back**, or `selected` is an unfed branch stranding shipped CSS — and **the painted pixel is the leg**) · **N-098** (dev-IPC latency + the amend-after-push merge phantom). Components registry: **`self-panel` = the 3rd widget** (after `substitutions-editor` and `entity-context-menu`) — **not** a `core` catalogue cell; **sampler catalogue unchanged**. `docs/xgen-client-frame-phase0.md` §6 (6.1g ✅ + the three §0 corrections promoted + **M-RP6.6 filed**). `ui/docs/xgen-region-dock-model.md` §5 (**the bus has a writer**). `docs/xgen-widget-surfaces-phase0.md` §7 (the **6.1i–l relabel** — primes deleted). Runbook `tasks/M_RP6_1G_SELF_PANEL.md` → **COMPLETED**. **No new D** (D-103 / D-107 extension).

**M-RP6.1g CLOSED. Next-active = M-RP6.1h** — **R8 inspector**, the selection bus's first *cross-region* reader (its writer is live, and it carries a real XGID). Still open: **surfaces Phase-0 §6** (5 items open for Joe, sharpest = Settings' own surface) → then **M-RP6.1i–l** (shelf core · mounts · UI-state store · widget manager) · **M-RP6.6 client resident** (the real F-1 close) · **M-RP6.1e-B1** no-select chrome · **M-RP7.x** node frame inheritance · **M-RP8** title-bar + frameless · **M-RP-ICON-ADOPT** · `temperature-indicator` ⏸️.

---

## Entry J-499 — M-RP6.1f centre region-shell (renderer A) + selection bus: built + CDP-verified in the real client; **M-RP6.1f CLOSED**

**Doc-bridge (D-074 second commit).** Clair's feat is already pushed = commit `899ff6e` (code-only, **12 files, +540/−8**); this entry + the paired canonical records = commit 2. **M-RP6.1f CLOSED.** The first real step of the widget grid: renderer **A** reads the D-103 `Layout` descriptor, mounts placeholder leaves into the client centre, and the **selection bus** primitive lands. Phase-0 was already locked (J-488 / D-107) — this was a **BUILD walk, not a re-design**.

**What landed.** `ui/core/lib/components/layout/` — `types.ts` (the **full** `leaf|split|tabs` + `Layout {version, root}` contract) · `resolve.ts` (a **pure**, DOM-free walk; never throws) · `resolve.test.ts` (6 cases) · `region-shell.svelte` (renderer A, **one** registered getter G) · `region-node.svelte` (the **internal, non-registering** recursion part — N-064). Plus `ui/common/lib/stores/selection.svelte.ts` (a **new `stores/` folder**) and the shell-local `layout-default.ts` / `region-placeholder.svelte` / the `app_client.svelte` mount / the `app.css` D5 flip / all `.region-*` appearance in `skin.css`.

**Three design calls worth keeping.**
1. **No Rust (D2).** A `get_layout` command returning a hardcoded default would either **duplicate the descriptor type in Rust** — precisely the **D-067 drift surface** the J-497 grounding killed — or return an opaque blob Rust does not own. The seam lives in the frontend as `async loadLayout()`; at **M-RP7.3** only its *body* changes to `invoke('get_layout')`, and Rust persists the tree as an **opaque blob** (the `get_substitutions` shape), so Rust never learns the node shape.
2. **The selection bus is a `$common` store — forced, not preferred (D3).** Both eventual consumers (**R8 inspector**, **`entity-context-menu`**) are widgets in `ui/common/…/widgets/`, and **W-3 forbids a `common` widget importing a shell dep**. A shell-local bus would be **structurally unconsumable**. Shape unchanged: `{regionId, entity} | null`, **one meaning** (the shelf minus-button was killed in `xgen-widget-surfaces-phase0.md` S-6 precisely so a second bus is never needed).
3. **Renderer in `core`, mount in the shell (D4)** — the frame precedent exactly. Grounded, not assumed: `ui/node/vite.config.js` already aliases `$core`/`$common`/`$assets` and the node already imports `$assets/skin.css`, so the **node inherits renderer A free at M-RP7.x**. `core/layout/` imports nothing shell or Tauri.

**🔑 The leaf-resolution machine was NOT invented — it was already shipped.** `message.svelte` has taken a prop-injected `widgets: Record<widgetId, Component>` registry and **dropped unresolvable ids** (W-13 reconcile) since M-RP5.5, with its getter reporting the **resolved** count so the drop is CDP-provable. Renderer A reuses **that exact shape**. No second mechanism. *(Grounding the code before writing the runbook is now the third consecutive milestone where it changed the design — J-497, J-498, this.)*

**⚠️ D5 — this milestone SUPERSEDES a J-495 lock, deliberately.** `.app-center` was `overflow-y: auto` + `padding: 12px 16px`: J-495 locked *“the centre is the ONLY scroller”* (D5), which was right for a single placeholder paragraph and **wrong for a dock layout**. A grid must **fill** and never scroll as a unit; each **leaf** owns its scroll — that is what a docked panel *is*. Flipped to `overflow: hidden; padding: 0; display: flex`, with `min-height: 0` riding every nested flex level (or the classic flexbox blowout puts the scrollbar straight back on the document). The frame chrome still never scrolls. → `docs/xgen-client-frame-phase0.md` §10.3 amended; **do not read its pre-J-499 form as current**.

**CDP verification — REAL CLIENT 9222 (D-097). Chat re-drove every leg itself on a live window (Rule 5).**
- **Registry 22 → 30**, `count === unique === 30`, enumerated. Net **+8**: **−`paragraph#center-placeholder`**, **+`region-shell#region-root`**, **+8 × `section#region-{spaces,rooms,self,room-header,stream,composer,members,inspector}`**. `region-node` registers **nothing** (N-064 held).
- **Getter G** `{version:1, leafCount:8, widgetIds:[8], droppedCount:0, unsupportedCount:0, depth:3}`.
- **Geometry — the N-091 leg, and it is why N-091 exists.** `docNoScroll` **true** (915===915) · `acNoScroll` **true** · `region-shell` **1927×870 fills** `.app-center` 1927×870 · `.app-center` computed `overflow: hidden`, `padding: 0px` (the D5 flip, measured not asserted) · **split ratios exact**: root-row children **[160, 321, 1122, 321]** against `sizes [1,2,7,2]`. **Chat measured this at a window width of 1927px — Clair measured 989px.** The ratio holding across a *different* window is a **stronger proof than reproducing her numbers**, and is the reason ratios (not absolutes) are the right leg. · Leaf self-scroll: inject tall content → that leaf scrolls, document still does not; restored.
- **The one N-090 data-exception, confirmed in the DOM:** a split's `sizes[]` are **descriptor DATA**, not skin — they ride **inline** `flex: 1|2|7|2 1 0px` per child (the `led` `--led-colour` / `meter` `--meter-fill` precedent). `data-dir` drives flex-direction from the skin. Gaps/borders/mins stay in `skin.css`.
- **Drop paths** (driven via the DEV `__XGEN_LAYOUT__` handle): unknown `widgetId` → `droppedCount 0→1`, `leafCount 8→1`, the ghost `section` **never registered**, no crash · `tabs` node → `unsupportedCount 1`, DEV warn, no crash · sizes-mismatch → equal-weight fallback + warn, no crash · restore → G back to 8 leaves, registry back to **30**.
- **Selection bus** (via `__XGEN_SEL__`): `null` → set `{spaces, space#x}` → **replaces** with `{rooms, room#y}` (one selection, **not a list**) → `clear()` → `null`.
- **Skin:** 6 `.region-*` rules in cascade (N-042 method) — split `rgb(52,59,71)` = `--s5`, leaf `rgb(22,24,28)` = `--s`; **accent-neutral** under an injected `--accent2` swap. **Zero component-local `<style>` shipped.**
- **Build/tests (Chat re-ran both):** `npm test` → **41 passed** (35 baseline + **6** new `resolve.test.ts`) · `vite build` → **150 modules** clean · `git show --stat 899ff6e` → 12 files, no Rust, no `ui/node/**`.
- **N-092a honoured:** the **orphan leg was NOT run** — the client's debug bridge is **state-only** (`id → {type,get}`, no DOM handle). The client-expressible proxy is the **clean return to baseline 30 after the drop churn**, which is what was measured.

**🔑 A GAP FOUND BY CHAT'S OWN MISTAKE — and the mistake is the point.** Restoring the layout, Chat **guessed** the DEV handle's shape (`__XGEN_LAYOUT__.default`; it is actually `{current, set}`) and thereby set the live layout to **`null`**. Grounded properly rather than papered over: **a null/absent layout unmounts `region-shell` entirely → a blank centre.** Registry 30→21, shell out of the DOM, `.app-center` empty — **no crash, no document scroll**. So region-dock §9's *“never crash on a stale tree”* half **holds**; its *“fall back to default”* half does **not** — today it falls back to **nothing**.

**Harmless at 6.1f and deliberately NOT fixed here:** `loadLayout()` returns a constant and **cannot** return null, so a `?? DEFAULT_LAYOUT` guard today would be **an unreachable branch shipped in a closed milestone** — exactly the D-065 / N-091 argument used to keep the `tabs` branch out. Chat does not get to invoke that rule against `tabs` and exempt itself. The fallback also belongs to the **loader** (parse a real file, find it missing/corrupt/schema-stale, recover), which is **M-RP7.3's** code and does not exist yet.

**→ Pinned to M-RP7.3's DoD in concrete terms so it cannot evaporate:** *a missing / corrupt / schema-stale layout file falls back to `DEFAULT_LAYOUT`, never to a blank centre — and the fallback is **exercised** (feed it a corrupt file), not asserted.* This is N-091's shape a second time: **the leg nobody ran was “what if there is no layout”.** Written down now, while we know why; run then, when it can actually fail. → **N-095**.

**Deviations — flagged, not absorbed (Rule 6). Two are CHAT'S OWN.**
1. **⚠️ Clair's “first `<style>` in the codebase” flag: conclusion RIGHT, evidence WRONG — and the evidence was checked rather than trusted (Rule 5).** She reported *“shipped core already carries `<style>`: message-stream (80), color-picker (222), separator (22)”*. **Grounded:** the **only** non-empty component `<style>` in the codebase is **`message-stream.svelte` — 32 lines** (257–288, structural: scroll / stacking / pill placement). **`separator.svelte` and `sb-cell.svelte` carry `<style></style>` — EMPTY.** **`color-picker.svelte` has NO `<style>` at all** — its own line-27 comment says so. Her **conclusion stands** (the runbook's “first in the codebase” claim was **false — Chat's error**), but a canonical record does not inherit invented numbers. **The accurate rule: N-031 always permitted an L1 per-component *structural* `<style>` (“rarely used”), and `message-stream` is its one live user; N-090 (J-498) widened *skinnable* to include layout / gaps / tracks, which squeezes L1 to almost nothing.** → **N-094**.
2. **The runbook was internally contradictory — Chat's defect, owned.** §3/D4 put `resolve.test.ts` in `ui/core`, while §5.6 forbade touching `ui/sampler/**` — yet the **only** vitest harness lives in `ui/sampler` and scanned `../common/lib/**` only. The two requirements cannot both hold. Clair's minimal fix (`ui/sampler/vitest.config.js` include gains `../core/lib/**/*.test.ts`, 5 lines) is **correct and belongs in the feat** — without it the tests do not run at all.
3. **The selection store needed a LOAD, which D4's file list omitted.** `__XGEN_SEL__` only exists if the shell *executes* the module (otherwise it is never evaluated), so `app_client.svelte` took a **side-effect import**. W-8 honesty intact: the module is loaded for inspection, and there is still **no writer** — R8 / `entity-context-menu` wire in at 6.1g+. *(Clair's first §5.4 attempt threw `Uncaught`; per §5.7 that is **inconclusive, not a failure** — and treating it as inconclusive is what surfaced the real gap.)*

**Records.** → **N-093** (renderer A: the reused drop-shape; the `sizes[]` inline-flex data-exception to N-090; `region-node` non-registering) · **N-094** (the `<style>` ground truth: N-031 L1 vs N-090) · **N-095** (the null-layout blank centre → M-RP7.3 fallback leg; and the registry-returns-to-baseline proxy for the client's absent orphan leg). Components registry: **`region-shell` = the 32nd `core`**; `region-node` does not catalogue (internal, N-064). `docs/xgen-client-frame-phase0.md` §6 (6.1f ✅) + **§10.3 D5 supersession**. `ui/docs/xgen-region-dock-model.md` (renderer A built, selection bus live). Runbook `tasks/M_RP6_1F_REGION_SHELL.md` → **COMPLETED**. **No new D** (D-103 / D-107 extension).

**M-RP6.1f CLOSED. Next-active = M-RP6.1g** — **R3 Self/connection live** (`get_self_state` read verb + scoped `app.emit` push; closes the **F-1 read half**). **The selection bus is now waiting for its first writer there.** Still open: **shelves / surfaces / UI-state store** (`docs/xgen-widget-surfaces-phase0.md` v1.0 — written, **NOT locked**; 5 items open for Joe, sharpest = Settings' own surface) · **M-RP6.1e-B1** no-select chrome · **M-RP7.x** node frame inheritance · **M-RP8** title-bar + frameless · **M-RP-ICON-ADOPT** · `temperature-indicator` ⏸️.

---

## Entry J-498 — M-RP6.1e-C3 Help→About assembly: built + CDP-verified in the real client; **M-RP6.1e-C CLOSED** (the three-way split complete)

**Doc-bridge (D-074 second commit).** Clair's work is already pushed (commits `782f385` feat · `62b3424` adjustments · `cc08be6` + `1ff7343` CSS refactor) plus Joe's own `2bde134` cosmetic skin pass; this entry + the paired canonical records = the doc commit. **M-RP6.1e-C3 CLOSED → M-RP6.1e-C CLOSED** (C1 `dialog` core ✅ J-496 · C2 `get_about_info` ✅ J-497 · C3 assembly ✅ this).

**⚠️ Process deviation, flagged not absorbed (Rule 6).** Joe pasted the session-open handoff to **Clair** by mistake. She ran the **design walk** and authored the **canonical runbook** — both **Chat's lane** under the three-agent model. No harm to the record (Joe locked the design; the runbook was correct), but it is logged so the lane boundary is not silently eroded by precedent. Chat then **reviewed the runbook against the shipped contracts** and amended it to v1.1 (A1–A5) before it went to Clair — which is where the two real defects were caught (below). She also shipped **four commits**, not the single code-only feat D-074 prescribes; accepted as pushed, noted.

**What landed.** `ui/client/src/about-dialog.svelte` (**new**, shell-local — core stays app-agnostic) wrapping the C1 `dialog`, fed by C2's `get_about_info` **on mount**, rendering the client logo + a `<dl>` metadata grid; a **Help** menu (`help.about`, **no accelerator** — F1 conventionally = Help *contents*) reusing the existing `menus`/`commandTable`/`runCommand` dispatch unchanged; the website link opening the **OS browser** via `tauri-plugin-opener` (four surfaces: `Cargo.toml` · `package.json` · `capabilities/default.json` · `desktop.rs` `.plugin()`); both logo masters copied into `ui/assets/` (`_hda` **wired** ~96px, `_hd_small` **reserved-unwired**, first consumer = M-RP8 title-bar). **`ui/core/` untouched** — verified by `git diff --stat`, not asserted.

**The runbook review earned its keep — two defects caught BEFORE Clair touched the code.** Chat's v1.1 grounded the runbook against the *shipped* component source rather than memory:
1. **A1 — the CSS home was wrong.** The runbook (Clair's v1.0) said "prefer a scoped `<style>` in `about-dialog.svelte`". That would have been the **first component-local CSS in the codebase** — a direct N-023/N-025 violation (appearance lives in a **removable** layer; components carry none).
2. **A2 — the website link would have shipped broken.** `link.svelte` types **both `href` and `text` as REQUIRED** (non-optional, no default), and `text=""` trips its own DEV "no accessible name" warn. The runbook passed `href={c?.link}` straight through → in browser-dev / pre-fetch that is `undefined` → an **href-less `<a>`**. Fixed to a conditional render.

**🔑 THE ARCHITECTURE CORRECTION IS THE MILESTONE'S REAL OUTPUT — and it came from Joe, mid-build, against BOTH Claudes.** A1 put the About CSS in `app.css`. Joe: *"the graphical elements we have in skin. app.css is for block positions etc"* → then, when Clair split it graphics-vs-layout: *"**all skinnable settings need to be in skin.css**"*. **This is broader than A1 and broader than Clair's first correction.** "Skinnable" is **not** "colour and type" — **gaps, sizing, grid tracks and spacing are skin-tunable too**. The About box is a **skinnable UI element**, so its **entire** `.about-*` ruleset belongs in `skin.css` beside the other component skins; **`app.css` is the app-frame skeleton and the per-app accent knob, nothing else**. Both refactors were verified as **pure relocations** — Clair re-measured computed styles and geometry after each move (fonts, gaps, dialog rect 386×428, centring 496,432 — **byte-identical**), which is the correct way to prove a refactor changed nothing. **This supersedes runbook A1** and is recorded as **N-090** (it is a durable layering rule, not an arc-local choice).

**🔑 A REAL C1 GAP, FOUND ONLY IN ASSEMBLY.** `skin.css` gained **`.dialog { margin: auto }`** — and this is **not an About tweak, it is a fix to the shipped `dialog` core's skin, affecting the sampler too** (shared L2). `showModal()` puts the dialog in the top layer with the UA's `position:fixed; inset:0`, but the project's **global margin reset zeroed the UA `margin:auto`** that centres it — so **every dialog pinned top-left**. **C1 was CLOSED as verified and this was still broken**: C1's legs proved `:modal` / focus / backdrop / reopen — **never *position***. The lesson is exact and generalises: **"verified" is only as wide as the legs you actually ran; a component can pass every leg and still render wrong.** → **N-091**.

**Window default 900×600 → 1240×1080** (Clair, `62b3424`). The **J-495 twin-config rule held** — *both* `tauri.conf.json` and `cdp.dev.conf.json` were edited (the whole `windows[0]` object is replaced wholesale by the `--config` overlay). Joe reported it "not working"; **it was working** — Chat measured `outerWidth 993 × 865` CSS at `devicePixelRatio 1.25` → **1241 × 1081 physical**. The config is applied as **physical** px, so at 125% scaling it yields ~992×864 of CSS space. **This is the J-495 DPI finding recurring** (there: 900×600 → 720×480 CSS). It is now **logged as a real open question** (logical-vs-physical), not a curiosity. → **N-092**.

**CDP verification — REAL CLIENT 9222 (D-097 — the sampler has no `get_about_info` and no shell frame). Chat re-drove every non-destructive leg itself (Rule 5).**
- **Client registry 7 → 22**, `count === unique === 22`, enumerated: `menu-bar#app-menubar` + `menu#…__file` + **`menu#…__help`** · the `status-bar` tree · `paragraph#center-placeholder` · **`dialog#about`** · `button#about__close` · `image#about-logo` · `link#about-link` · **9 value `label#about-*`**.
- **`:modal` = true** (the C1 load-bearing leg — `showModal()` reflects the `open` attribute, so only `:modal` discriminates a real modal from the silent non-modal downgrade). `dialog#about → {title:"About XGen Client", open:true}`.
- **Body truth check — all 10 fields REAL, none the `—` guard:** `version 0.10.3` · `built "2026-07-11 11:50:03 UTC · 2bde134"` · `rustc 1.95.0 (59807616e 2026-04-14)` · `tauri 2.11.1` · `svelte 5.55.5` · `windows x86_64` · app_dir · data_dir · config_path. **The rendered SHA `2bde134` IS `HEAD`** — so `build_info` is demonstrably live, not stale (the J-497 §2.4 mechanism, self-proving in the box).
- **D4 Built+commit render as ONE row** (`date · shortSHA`) ✅. **F2** — "Alchemy Dump" present, **personal name absent from the rendered DOM** (regex-grepped, not eyeballed) ✅. **F1 link getter** `{text:"https://www.alchemydump.com", href:…, external:true, disabled:false}` — C2's baked literals confirmed rendering (**KEEP**, Joe).
- **Close → `open:false`, `:modal` gone; registry stays 22** (C1's always-mounted claim holds — a closed `<dialog>` is `display:none`, not unmounted). **Re-open → `:modal` true again** — the reconciliation proof (impossible if the `$bindable` had desynced).
- **FIRST REAL menu-bar roving Left↔Right** — two menus finally exist: `activeIndex` 0→1 (Right) →0 (Left), End→1, Home→0, `items:2` ✅. The M-RP6.1d machine's roving is now *exercised*, not merely *shipped*.

**Three legs NOT claimed as verified (Rule 1 — the discipline is what makes the rest worth reading).**
1. **Esc-close: INCONCLUSIVE, not failed.** A synthetic `KeyboardEvent` dispatched via `eval` is **untrusted** and does **not** trigger the UA's native `<dialog>` cancel/close default. (J-496 hit the same wall from the other side and solved it with a **trusted** `Input.dispatchKeyEvent`; Chat's `eval`-only harness path cannot.) Left to Joe's keyboard.
2. **The registry orphan leg is NOT EXPRESSIBLE on the client.** The debug bridge (`ui/common/lib/components/base/debug.ts`) is a **state-only** registry — `id → {type, get}`, **no DOM handle, no marker attribute**. `count===unique===domCount` + "0 orphans both directions" is a **sampler-harness capability** that the real client's bridge cannot provide. The runbook demanded it anyway; **Chat's own v1.1 review passed that leg through unchallenged — Chat's miss, not Clair's.** Recorded rather than quietly dropped. → **N-092**.
3. **F1 link → OS browser, and Ctrl+Q / File→Exit**: human legs (an OS-browser handoff has **no in-page consequence to synthesise**; File→Exit is destructive). Joe's eye, per the **J-495 grip precedent** (human-verified + measured).

**Rule-6 grounding that paid off (again).** The runbook flagged the exact `tauri-plugin-opener` permission identifier as Clair's grounding point rather than guessing it — the C2 lesson (**ground the permission model per command class**) applied a third time: J-495 `core:window:*` **needed** a grant, J-497 app-defined commands needed **none**, J-498 a **plugin** command needs one again. **The lesson never generalises; it must be re-grounded every time.**

**Deferred / opened.** **M-RP-WINSTATE (window size + position memory) ⏸️ POSTPONED — Joe-locked, gated on the widget grid.** Chat's preference was **B** (own the store: geometry is the first member of a family — pane splits, grid layout, theme, last-open panel all need persistence, and taking `tauri-plugin-window-state` now means two persistence mechanisms later). **Joe's deferral is the better call and was taken:** building geometry persistence *before* the widget grid exists is building against an unknown, and **M-RP8** (frameless + custom title-bar) changes how the window is dragged/resized anyway. His **(a)** preference is noted but **deliberately NOT pre-locked** — the A-vs-B case is *strongest today and weakest exactly when the arc executes*, so locking now would lock at the point of least information. **The deciding criterion is written down instead:** at kickoff, *did the widget grid produce a persistent UI-state store?* **Yes → B** (geometry = five keys in it, no new dependency). **No → A** (`tauri-plugin-window-state`, justified by the **OS-domain vs app-domain** split — the WM owns the frame, the app owns the interior — **not** by convenience). Either way: **clamp restore to the monitor work area** and **settle the logical-vs-physical unit question (N-092) in the same arc**. Consequence: **1240×1080 is a first-launch default that will eventually be overridden — do not tune it.**

**Records.** → **N-090** (all skinnable settings → `skin.css`; `app.css` = frame skeleton + accent knob only — Joe's rule, supersedes runbook A1) · **N-091** (`.dialog { margin:auto }`; the C1 position gap; "verified" is only as wide as the legs run) · **N-092** (the client debug bridge is state-only → no orphan leg; + the physical-px window-config finding). Components registry: `dialog` skin amended. Runbook `tasks/M_RP6_1E_C3_HELP_ABOUT.md` → **COMPLETED** (v1.1, with a delivered-vs-runbook delta section). **No new D** (D-107 extension).

**M-RP6.1e-C CLOSED. Next-active = M-RP6.1f** (centre region-shell scaffold + selection bus). Still open: **M-RP6.1e-B1** no-select chrome · **M-RP7.x** node app inherits the frame (its About will want `NodeAboutInfo`, deliberately still not declared) · **M-RP8** `title-bar` + frameless restore · **M-RP-ICON-ADOPT** · **M-RP-WINSTATE** ⏸️ · `temperature-indicator` ⏸️.

---

## Entry J-497 — M-RP6.1e-C2 `get_about_info` (xgen-common::about + build_info rustc + Svelte lockfile read): built + CDP-verified in the real client; M-RP6.1e-C2 CLOSED

**Doc-bridge (D-074 second commit).** Clair's feat (code-only, 7 files) is already pushed = commit `50b5640` (commit 1); this entry + the paired canonical records = commit 2. **M-RP6.1e-C2 CLOSED** — the second step of the three-way M-RP6.1e-C split (J-496). **The Rust half: no UI.** Verify was the **real client (9222)** — the sampler is a `tauri`+`tauri-build`-only crate with no `xgen-common`/protocol deps and structurally cannot host this command (D-097).

**What landed.** The read path for everything in Joe's About that the frontend cannot see: build date, commit, Rust/Tauri/Svelte versions, platform, app directory, data/config paths. Seven files, scope-clean (`xgen-common/{build.rs, src/build_info.rs, src/about.rs, src/lib.rs}` + `xgen-client/{build.rs, Cargo.toml, src/desktop.rs}`) — **no `ui/**`, no `ops.rs`** (About-info is not a protocol verb; it has no node round-trip, no mutation, no CLI meaning, and must never grow D-092's four arms).

**The headline is what did NOT get built.** Chat's *own* first design for this milestone was **wrong in three places**, and grounding the code before writing the runbook caught all three (§2.0 of the runbook, placed deliberately first so Clair read the corrections before the design):

1. **`xgen-common::build_info` ALREADY EXISTED** — and already emitted `BUILD_TIMESTAMP` + `BUILD_GIT_HASH` (its `build.rs` already shelled `git rev-parse --short HEAD`), already consumed in ~6 places (`--version`, `print_banner`, `ops::StatusResult`, `ClientState.version`/`.build`). The original plan — "add a `build.rs` to `xgen-client` emitting Built + SHA" — would have created a **second build-metadata surface**: precisely the **D-067 drift surface** this project exists to eliminate. **Corrected: reuse it.** The Q4 lock ("Built = date + short SHA") turned out to be **already implemented**.
2. **The Svelte version is not in `package.json`** — it declares a **caret range** (`"svelte": "^5"`), not a version. The original plan ("build.rs reads package.json") would have rendered **`^5`** in the About box. The resolved **`5.55.5`** lives in the **committed** `ui/client/package-lock.json` (confirmed via `git ls-files`). **Corrected: read the lockfile.**
3. **A new `#[tauri::command]` needs NO capability grant.** Capabilities gate **`core:`/plugin** commands, not app-defined ones — `get_state`/`get_pacing_state`/`get_substitutions`/`set_substitutions`/`quit` all run today against a `capabilities/default.json` carrying only `core:default` + `process:default` + `core:window:allow-start-resize-dragging`. **J-495's capability lesson was specific to `core:window:*`**; carrying it forward here would have added a meaningless permission. **Corrected: no capability change.** *(This is the honest inverse of J-495: there, a missing grant was the catch. The lesson is not "always add a grant" — it is "ground the permission model per command class.")*

Also grounded before design: **`tauri::VERSION` exists** (`pub const VERSION: &str = env!("CARGO_PKG_VERSION")`, resolved **2.11.1**) → no build.rs needed for it; **`data_dir` was not managed state** (only `ConfigPath` was).

**Design as shipped (B2, Joe-locked).** `build_info` gained **exactly one** field — `RUSTC_VERSION`, emitted from the **existing** `xgen-common/build.rs` via `rustc -V` (no new dependency; the same `std::process::Command` shape the `git` call already used) — **one build-info surface, and the node inherits it free**. New `xgen-common::about`: a shared **`AboutInfo`** environment block (name · version · link · built · commit · rustc · tauri · svelte · platform · app_dir · data_dir · config_path) with **paths and app facts passed in, never derived inside common**, plus the typed per-app **`ClientAboutInfo { common, … }`** wrapper. **`xgen-common` gained no `tauri` dependency** (it is the protocol-layer crate — the Tauri version is read in the shell as `tauri::VERSION` and passed in; the same rule that keeps `core` app-agnostic). The Tauri command in `desktop.rs` is a **thin wrapper** over one canonical `collect()` — the `get_substitutions` shape.

**The trap that would have passed a lazy verify.** `AboutInfo.version` **must** be the *client's own* `env!("CARGO_PKG_VERSION")`, **not** `build_info::VERSION` (which is **xgen-common's** version). **Both are `0.10.3` today** — so the wrong wiring produces a value that *looks* perfectly correct and proves **nothing**. The runbook therefore demanded proof **from the source line, not the value**. Clair supplied exactly that: the call site passes `env!("CARGO_PKG_VERSION")`, compiler-expanded **in the `xgen-client` crate**. The second named failure signature — a returned `svelte: "^5"` — did not fire either (`5.55.5`).

**CDP verification — REAL CLIENT 9222 (Rule 2, real output). Chat independently re-drove the read on a fresh launch (Rule 5).**
- **The returned JSON** (Chat's own `invoke('get_about_info')`, wrapped under `{"common":{…}}` — the `ClientAboutInfo` seam):
  ```
  {"name":"XGen Client","version":"0.10.3","link":"https://www.alchemydump.com",
   "built":"2026-07-11 08:42:12 UTC","commit":"50b5640",
   "rustc":"rustc 1.95.0 (59807616e 2026-04-14)","tauri":"2.11.1","svelte":"5.55.5",
   "platform":"windows x86_64",
   "app_dir":"C:\\cargo-targets\\XGenProtocol\\debug",
   "data_dir":"C:\\Users\\Joe\\AppData\\Local\\XGenProtocol",
   "config_path":"C:\\Users\\Joe\\AppData\\Local\\XGenProtocol\\xgen-client_config.toml"}
  ```
- **Every field checked against something external** (Clair): `commit` = `git rev-parse --short HEAD` **exact** · `rustc` = real `rustc -V` · `tauri` = `Cargo.lock` · `svelte` = `package-lock.json` · `platform` real · all three paths `Test-Path` **True** (the config file exists).
- **§2.4 demonstrated live, not merely asserted.** Clair's read showed `commit:31d9d0a` / `built:08:25:27`; **Chat's later read showed `commit:50b5640` / `built:08:42:12`.** That is **not a discrepancy** — it is the documented mechanism working: her feat moved HEAD, `.git/HEAD` changed, `build.rs` re-ran, the stamp advanced. **`commit` is the field that exactly identifies a build**; `built` is the last compile. C3 renders the pair together so the two read truthfully.
- **No permission denial** — `errCount:0`, `errs:[]` → the command works with **no capability grant**, confirming correction (3) empirically rather than by argument.
- **Registries (measured, not predicted — Rule 5).** Client's own registry **7** — Chat re-measured the exact ids: `menu-bar#app-menubar`, `menu#app-menubar__file`, `status-bar#app-statusbar`, `status-indicator#…` + its `__led`/`__label`, `paragraph#center-placeholder`. **Sampler catalogue 313, unchanged** — grounded **by scope**: `git show --stat 50b5640` = 7 files, **zero `ui/**`**, so no code reached the sampler (an honest grounding, not a re-measurement theatre).
- **Tests (Clair)** — workspace `cargo test`: **1507 passed, 0 failed, 62 ignored** (5 new `about` tests among them). **`vite build`** clean, 138 modules (unchanged — C2 touched no frontend).

**Joe's observation, answered.** *"I don't see a Help menu in the client's UI"* — **correct, and expected.** C2 is the Rust half and ships **no UI whatsoever** (scope-clean = no `ui/**`). The client registry proves it: `menu#app-menubar__file` is the **only** menu. Help→About is **C3**.

**Four deviations — flagged, not absorbed (Rule 6).**
1. **`AboutParams` struct instead of ~7 positional `String` args to `collect()`.** Every passed-in field is a `String`, so positional call sites could transpose silently. A named struct is the safer shape. **Accepted** — a judgment call squarely inside the runbook's "`collect(params…)`" latitude, and the right one.
2. **`NodeAboutInfo` NOT declared.** §3 left it to Clair's call ("only if it costs nothing"). She left it to **M-RP7.x**, reasoning that a node wrapper today would **guess** the node's extension fields (listen port, peer count, node XGID, federation role) with **no node-side `collect()` call site to validate them against**. **Accepted, and it is the better call** — the client wrapper earns its keep as a *proven* seam; the node's should land with the node's real About.
3. **`ClientAboutInfo` kept as the zero-extension wrapper** — per §2.2's explicit instruction, and Clair concurred: the typed seam keeps the command's return type stable when the first client-only field lands.
4. **`name` / `link` literals were not specified by the runbook.** Clair chose **`"XGen Client"`** + **`"https://www.alchemydump.com"`**, grounded in the global config's App-About convention. **These are shell-supplied and WILL be rendered in the About box — flagged for Joe at C3**, where the final About text is tuned.

**Records.** `docs/xgen-client-frame-phase0.md` (§6 + §10.4 — C2 ✅). `docs/ROADMAP.md`. `CLAUDE.md` PLAY (head J-496→J-497). `tasks/M_RP6_1E_C2_ABOUT_INFO.md` → COMPLETED. This entry. **`ui/docs/*` deliberately untouched** — no component was built and there is **no UI-side lesson** to record; the runbook explicitly said not to invent an N-note for a Rust milestone (D-065). Component registry unchanged (**313**). **No new D** (D-107 extension). Clair feat = commit 1 (pushed `50b5640`); this doc-bridge = commit 2. Not pushed — Joe pushes.

**Next-active.** **M-RP6.1e-C3** — the **Help→About assembly**: the Help menu (`help.about`, **no accelerator** — F1 conventionally means Help *contents*) → the **2nd populated menu**, i.e. the first real `menu-bar` roving Left/Right · the `dialog` (C1) mounted and fed by `get_about_info` (C2) · the logo assets (`_hda` canonical mark, client + node; `_hd_small` reserved ≤16px). **The N-086 shared-W-2 extraction stays NOT-TRIGGERED** (J-496 verdict): Help is a second *instance* of the `menu` machine, not a second *shape*. Verify **real client 9222**; the client's own registry will grow from 7 — **measure it**.


---

## Entry J-496 — M-RP6.1e-C1 `dialog` core (31st `core`; native `<dialog>` + `showModal()`, no W-2 machine): built + CDP-verified in the sampler; M-RP6.1e-C1 CLOSED

**Doc-bridge (D-074 second commit).** Clair's feat (code-only, 3 files) is already pushed = commit `4f35d87` (commit 1); this entry + the paired canonical records = commit 2. **M-RP6.1e-C1 CLOSED** — the first step of the **three-way M-RP6.1e-C split** locked this session (**C1** `dialog` core, sampler, no Rust · **C2** `get_about_info` — `xgen-common::about` + `build.rs` metadata + the Tauri read command, real client · **C3** Help→About assembly, real client). Verify was **sampler 9422** (the component is a pure di container with no shell or window effect — D-097 puts it in the sampler, unlike 6.1e-B).

**What landed.** `dialog` — the **31st `core`**, a **di composite** modal container, flagged as a gap since J-432. Root `<dialog class="dialog">`; header `title` → `children` body slot → footer with one composed `button` (`…__close`, default label **"Close"**, not "OK" — Joe). Three files, scope-clean (`ui/core/…/dialog.svelte`, `ui/assets/skin.css`, `ui/sampler/src/app_sampler.svelte`) — **no `xgen-client/**`, no `ui/client/**`, no Rust** (the C1 fence held).

**The design, in one line: the native element does the whole machine's job.** Every other popup in this library (`combobox`, `tag-select`, `color-picker`, `entity-context-menu`, `menu`) hand-rolls a W-2 owned-popup behaviour machine — **not by preference, but because no native element gave it to them**. `<dialog>` + `showModal()` supplies top-layer stacking, `::backdrop`, focus trap, background-inert and Esc-dismiss **natively**. So `dialog` builds **no machine at all**. That is why this is the smallest component in months, and it is the whole point.

**The trap this milestone was designed to catch — and did.** There are two ways to open a `<dialog>`: `showModal()` (modal: top layer, backdrop, focus trap) and the `open` **attribute** (non-modal: none of those). The attribute path **looks like it works** — it renders, it's visible, and `el.open` reports `true`. Worse: **`showModal()` sets the `open` attribute itself as a side effect**, so `openAttr:true` was measured *while correctly modal* — the attribute proves **nothing** in either direction. The **only** discriminator is **`el.matches(':modal')`**, which the runbook made a non-skippable verify leg for exactly this reason. Had we asserted on the obvious thing (the attribute), we would have shipped a green verify for a silently-downgraded, trap-less, backdrop-less dialog. **`isModal:true` measured.**

**The load-bearing axis — a `$bindable open` lies unless the element writes back.** Native `<dialog>` **owns its own open state**: `showModal()`/`close()` are imperative, and **Esc fires `cancel` → `close` without ever consulting the prop**. The naive binding therefore desynchronises the instant the user presses Esc — element closed, prop still `true`, and the *next* `open = true` is a **no-op because the prop never changed**: the dialog can never be reopened. Closed by three rules: a **guarded** `$effect` prop→element (`if (el.open !== open)`, or effect and listener ping-pong); a **`close`-event listener writing `open = false` back into the binding** (not polish — it is what keeps the bindable honest); `onClose?` from the same listener. **Getter G reads `open` from `el.open` (the DOM), not the prop** — G reports what *rendered*, not what was *intended*.

**CDP verification — SAMPLER 9422, both accents (Rule 2, real output). Clair drove the full 11 legs; Chat independently re-drove the non-destructive ones on a fresh launch (Rule 5 — a registry number I have not measured does not go into a canonical record).**
- **V1 registry (Chat, re-measured)** — `count===unique===domCount` **313/313/313**, **0 orphans both directions** (`orphanRegNotDom:[]`, `orphanDomNotReg:[]`). **309→313 (+4)**, the exact ids:
  ```
  dialog#default   dialog#labelled
  button#default__close   button#labelled__close
  ```
- **V2 getter G (Chat)** — `dialog#default` → `{title:"Confirm action", open:false}`; `dialog#labelled` → `{title:"About XGen", open:false}`. `open` read from `el.open`, `false` on mount.
- **V3 the `:modal` proof (Chat)** — trigger click → `elOpen:true`, **`isModal:true`**. Also measured: `openAttr:true` **while modal** — documenting *why* the attribute is not the discriminator.
- **V4 backdrop (Clair)** — `rgba(0,0,0,0.55)` renders, `painted:true`; grid dimmed under the modal.
- **V5 focus trap (Chat)** — `focusInside:true`, `activeTag:BUTTON` (native autofocus onto Close).
- **V6 Esc honesty — proven from BOTH ends.** Clair drove a **trusted** `Input.dispatchKeyEvent` Escape → `elOpen:false` + `getterOpen:false` + `onClose` fired. Chat cannot synthesise a trusted key event through `eval`, so re-drove the **same code path element-first**: `el.close()` → read after settle → `elOpen:false, getterOpen:false` → **re-click the trigger → `reopened:true, isModal:true`**. Reopening is **impossible** if the prop had lied `true` (the guarded `$effect` would never re-fire) — so the write-back is real.
- **V7 close button (Clair)** — click `…__close` → closed, `onClose` fired once.
- **V8 registry stable across open/close (Chat)** — **313 open === 313 closed**; `…__close` registered in **both** states. Native `<dialog>` closed is `display:none`, **not unmounted** — the **opposite of `menu-item`** (N-086), which mounts on popup-open. There is no open/close delta to hunt.
- **V9 skin (Chat)** — `.dialog` bg **`rgb(28,31,36)`** = `--s2`, radius **`10px`** = `--rad2`; 5 `.dialog*` rules in cascade (Clair).
- **V10 accent (Clair)** — surface **accent-neutral** (bg/title/btn identical client↔node); sole carrier = the inherited `button` focus-ring, gold `rgb(194,136,64)` ↔ blue `rgb(58,122,176)`. Stated, not invented (the N-087 pattern).
- **V11 build (Clair)** — `vite build` clean, **167 modules** (166→167).

**A Rule-6 grounding that mattered.** `::backdrop` is **not** a normal descendant and the CSS-custom-property cascade into it is **engine-uncertain** — Clair did not rely on `var(--…)` inheriting, and used a **literal** `rgba(0,0,0,0.55)` scrim (a modal dim is accent-neutral chrome, so nothing is lost). Grounded, not guessed. `:modal` support confirmed present before it was made load-bearing; `use:envelope` on a `<dialog>` root was a non-event (plain `HTMLElement`, post-N-083 widening).

**Runbook deviation — flagged, not absorbed (Rule 6).** The runbook specified `title` as a required `string`; Clair authored **`title?: string`** defaulting to `''` with a conditional header, matching the **`section` precedent**. **Accepted as the shipped contract** (strictly more robust — a titleless dialog degrades to a bare surface; About is always titled; G still reports `title` verbatim). Recorded here rather than silently retrofitted into the runbook.

**Harness method finding (Chat, for the next CDP driver).** Multi-statement evals combining local `var` declarations with callbacks intermittently return a bare `EVAL ERROR: Uncaught` through the PS 5.1 harness even when the equivalent single-expression eval succeeds — `.click()` and `try/catch` each proved fine **in isolation**, so the fault is the harness/transport, not the page. **Prefer one-expression `JSON.stringify({…})` evals**; and when an eval throws, the *next* read is **inconclusive, not a failure**. One such throw produced a `reopened:false` reading which was **discarded rather than recorded as a phantom regression** (Rule 1). Indexed access (`querySelectorAll("button")[32].click()`) worked where the `forEach`+closure form threw.

**Records.** `ui/docs/xgen-ui-components.md` **v0.65→v0.66** (`dialog` row + the 313 count block; the **N-052 `modal`/`dialog` deferral logged at M-RP2.21 is now closed** in place). `ui/docs/xgen-ui-notes.md` **+N-089** (v0.72→v0.73). `docs/xgen-client-frame-phase0.md` (§6 + §10.4 — C1 ✅, the C-split recorded). `docs/ROADMAP.md`. `CLAUDE.md` PLAY (head J-495→J-496). `tasks/M_RP6_1E_C1_DIALOG.md` → COMPLETED. This entry. **No new D** (D-107 extension). Clair feat = commit 1 (pushed `4f35d87`); this doc-bridge = commit 2. Not pushed — Joe pushes.

**Design locks taken this session (the C2/C3 groundwork, Joe-locked).**
- **`get_about_info` is NOT a D-092 four-armed verb** — it is a **shell read command** (the `get_substitutions` precedent: a thin `#[tauri::command]` in `desktop.rs` over one canonical function). Grounded against the real client: all five existing Tauri commands (`get_state`/`get_pacing_state`/`get_substitutions`/`set_substitutions`/`quit`) are exactly this shape; none is a CLI/batch/aicontrol verb. About-info is build + environment metadata — no node round-trip, no mutation, no CLI meaning.
- **Home = `xgen-common::about` (option B2).** A shared **common env block** (`AboutInfo`: Built · Rust · Tauri · Svelte · Platform · paths **passed in**, never derived) + **typed per-app extensions** (`ClientAboutInfo { common, … }` / `NodeAboutInfo { common, … }`). Joe's constraint drove this: **the node will have its own About with different content** — so the node differs by **addition, not contradiction**, and the shared block is the anti-drift win without a stringly-typed bag.
- **"Built" = date + short git SHA** (a date alone cannot tell two builds apart). Honest caveat: `build.rs` does not rerun on a no-op rebuild, so "Built" means *last actual compile*.
- **The N-086 shared-W-2 extraction: NOT triggered — and that verdict is written down, not left silent.** J-492 deferred the extraction to "the 2nd populated menu or the submenu-flyout", and Help→About *is* the 2nd populated menu — so the trigger nominally fires. Honest reading: **Help is a second *instance* of the `menu` machine, not a second *shape*.** It forces no new abstraction, while `entity-context-menu` still diverges on every axis that made the fresh-minimal build right (portal, dd header, async `onSelect`, variant/purpose). Refactoring a **closed, verified** widget with no forcing function is precisely the wrong-abstraction risk N-086 was written to avoid. **The trigger is re-scoped to its real forcing function: the submenu-flyout, or a menu that needs the portal.**
- **Logo (option A).** Grounded: `ui/assets/` had **only 64px**; the masters in `logo/` are **all 1000×1000**. `_hda` (alpha) is the canonical mark → `ui/assets/logo_client_hda.png` + `logo_node_hda.png`, CSS-sized to ~96–128px in About. `_hd_small` is a **physically simplified ≤16px** asset (details omitted to survive antialiasing — Joe) → also copied, **reserved** for ≤16px contexts (first real consumer: the M-RP8 `title-bar` app icon). The **context rule is recorded so nobody later downscales the wrong master.**

**Next-active.** **M-RP6.1e-C2** — `get_about_info`: `xgen-common::about` (B2) + `build.rs` env vars (Built date + SHA, `rustc -V`, Tauri version, Svelte version read from the client `package.json`) + **`data_dir` promoted to managed state** (only `ConfigPath` is managed today — grounded) + the Tauri command + capability check. **Real Rust work**, verified in the **real client 9222** via CDP `invoke`. Then **C3** Help→About assembly.


---

## Entry J-495 — M-RP6.1e-B real-client frame consolidation: built + CDP-verified in the real client; native-title-bar pivot (frameless DEFERRED to M-RP8); M-RP6.1e-B CLOSED

**Doc-bridge (D-074 second commit).** Clair's feat (code-only, 5 files) is already pushed = commit `8f401bd` (commit 1); this entry + the paired canonical records = commit 2. **M-RP6.1e-B CLOSED** — the second step of the M-RP6.1e split (J-493). **No component was built** — this milestone *assembles* the shipped `core` frame components into the real client shell and retires the legacy hand-rolled chrome. Verify was **real client only (9222)** per D-097; **sampler catalogue registry unchanged 309**.

**What landed.** The client frame is now a real BorderPane: native title bar → full-width `menu-bar` → `.app-center` (the ONLY scroller) → `status-bar` pinned bottom. The hand-rolled `.state-indicator` (local `dotColor()` / `isPulsing()` / `currentState.label`) is **gone**, migrated onto the shipped `status-indicator` (its `led` + `label`); the centre logo and the redundant Quit button are **gone** (File→Exit / Ctrl+Q is the exit — `handleQuit` retained, it is what the command table resolves to); the SE resize-grip is wired to Tauri `startResizeDragging`; the window is resizable at 900×600 / min 640×400. Five files, scope-clean (`tauri.conf.json`, `cdp.dev.conf.json`, `capabilities/default.json`, `app_client.svelte`, `app.css`) — **no `ui/core/**`, no `ui/common/**`, no `ui/sampler/**`, no Rust** (D10 held).

**Two Rule-5 catches the J-493 scope list missed, both caught at runbook-grounding and both real.** (1) **`xgen-client/cdp.dev.conf.json` duplicates the whole `windows[0]` object** — Tauri's `--config` overlay replaces the `windows` array wholesale, so flipping only `tauri.conf.json` would have left the **debug** window (9222, the very surface we verify on) at 420×260 / `resizable:false`. **Both files flipped.** (2) **Capabilities** — `capabilities/default.json` carried `core:default` only, and Tauri v2's `core:window:default` is **getters-only**; `start_resize_dragging` is a mutating command needing an explicit grant. Clair grounded both Rule-6 confirm points against the installed sources rather than guessing: `startResizeDragging('SouthEast')` is a valid plain-string `ResizeDirection` literal (`window.d.ts`), and `core:window:allow-start-resize-dragging` is a valid schema enum value (`desktop-schema.json` + `acl-manifests.json`). No flags needed; the runbook's names were right.

**⚠️ The native-title-bar pivot (Joe's mid-session call — a Phase-0-level change, recorded honestly).** J-493 §10.3 locked the window **frameless** (`decorations:false`), with a custom `data-tauri-drag-region` strip for move (D2) and the SE grip as the **sole** resize affordance (D3). In-session Joe flagged the practical cost: a frameless window during development has **no window controls and no way to move it**. He chose **`decorations: true`** — the native title bar. Consequence: the OS title bar supplies move + minimise/maximise/close + native edge-resize, so **D2 (the drag region) was fully reverted** (wrapper removed, `allow-start-dragging` dropped) and **D3 (the grip) demoted** from sole resize path to a supplementary corner affordance.

**This is NOT a reversal of the frameless design — it is frameless DEFERRED, with a scheduled return.** Joe's stated intent (2026-07-11): native decorations are a **temporary development affordance**, and the Discord-shaped endpoint (custom chrome, extra buttons, immune to OS UI theming) stands. The honest technical correction recorded in the same conversation: a *customised native* bar cannot give either of the two things wanted — extra buttons are impossible on a native caption, and only colours (Win11 DWM attributes, silently ignored on Win10) can be themed. **Discord-shaped IS the custom bar.** So the endpoint is a real `title-bar` `core` component, filed as **M-RP8** (see below), and the interim DWM colour-tint was considered and **rejected** (work we would delete). **Joe's rule, locked:** *no native elements within the window's main pane* — the native chrome is the OS title bar and nothing else. Hence the **grip stays** (`grip=true` + the resize capability retained) even though native edges now resize: it is the component default, the seam is proven, and keeping it wired the whole way through is precisely **why M-RP8 is cheap** — the resize seam never has to be rebuilt. **No new D** (D-107 extension); `docs/xgen-client-frame-phase0.md` §10.3 revised in place (v1.4→v1.5).

**A real latent shell bug, found and fixed (→ N-088).** Joe reported the status-bar floating mid-window. Root cause was **not** the status-bar: the Svelte mount target `<div id="app">` had **no height rule anywhere** (not in `main.js`, `index.html`, or any CSS), so `.app-frame { height: 100% }` resolved against an auto-height parent and **collapsed to content height** — 100px inside a 480px window. The old 420×260 window with a content-sized centre pane **masked** this for the whole project's life; a bottom-pinned status-bar is what finally exposed it. Fix: `html, body, #app { height: 100% }` in `app.css` (in-scope, shell chrome). This is a genuine find, not a workaround — recorded as **N-088**.

**A second, transient find (dead-ended by the pivot, recorded for the M-RP8 builder).** The runbook's D2 assumed the `menu-bar` would sit at intrinsic width leaving a bare strip to drag. It does not: the `core` skin sets `.menu-bar { width: 100% }` — it is *designed* as a full-width bar with its own background + border, so it filled the drag strip entirely and `data-tauri-drag-region` never fired (Tauri drags only when the **event target itself** carries the attribute, never an ancestor). Clair realised D2's intent from `app.css` alone without touching `core`/skin (shrink the bar to `width:auto`, move the bar treatment onto `.frame-top`) — and then the pivot reverted all of it. **The lesson survives the revert:** when M-RP8 builds the custom `title-bar`, it must own its **own** drag-region root; it must not try to drag on the menu-bar's strip.

**CDP verification — REAL CLIENT 9222, decorated window (Rule 2, real output). Chat re-drove all non-destructive legs (V1–V8); Clair owned the destructive ones (V11/V12); Joe supplied the one physical gesture no CDP eval can synthesise (V9).**
- **V1 registry** — `count===unique===domCount` **7/7/7**, **0 orphans both directions** (`orphanRegNotDom:[]`, `orphanDomNotReg:[]`). The client's own registry (**measured, not predicted** — Rule 5):
  ```
  label#app-statusbar__status-indicator__label   led#app-statusbar__status-indicator__led
  menu#app-menubar__file   menu-bar#app-menubar   paragraph#center-placeholder
  status-bar#app-statusbar   status-indicator#app-statusbar__status-indicator
  ```
  `button#quit` **gone** (D8). Delta from J-492's 3: −1 quit, +5 status-bar subtree, +1 centre placeholder.
- **V2 getter G** — `status-bar#app-statusbar` → `{leftCount:1, rightCount:1, hasGrip:true}` — exact (D4: no `secondaryText`, the version tag stays 6.1e-C's).
- **V3 live migration (the D1 proof)** — `status-indicator#…` → `{state:"DISCONNECTED", caption:"Disconnected", hasLink:false}`; `led#…__led` → `{state:"DISCONNECTED", colour:"var(--err)"}`, computed background **`rgb(138, 42, 42)`** — **not black**. `led`'s unknown sentinel is `#000000`, so a black led would mean an unenumerated lifecycle state; all 11 are enumerated in `STATE_COLOURS`. `label#…__label` → `{text:"Disconnected"}` = `currentState.label`.
- **V4 window (the Catch-1 proof)** — `is_resizable` **true**, `is_decorated` **true**, `inner_size` **900×600**. A missed `cdp.dev.conf.json` would have read 420×260. *(Honest wrinkle: DPR is **1.25** and the 900×600 landed as **physical** px → the window is **720×480 CSS px** on screen. Not a defect of this milestone; the design intent of 900×600 **logical** is not literally met at 125% scaling. Logged, not fixed.)*
- **V6 centre-only scroll (never run before, in either config)** — injected height into `.app-center`'s child: centre `scrollTop` **0→500**, `document.documentElement.scrollTop` **0**, `document.body.scrollTop` **0**, `docScrollable:false`; menu-bar `.top` **0** and status-bar `.bottom` **480** **constant** across the scroll (`chromeHeld:true`); `.app-center` `overflow-y:auto`, `.app-frame` `overflow:hidden`. Injected height restored (`childInlineHeight:"(cleared)"`, `centerScroll:0`). **The centre is the only scroller** (the M-RP4.9/J-466 flex-column pattern, now in the real client).
- **V7 menus still open under the new frame** — closed **7** → click File → `menu{label:"File", open:true, itemCount:1, activeIndex:0}` + `menu-bar{items:1, activeIndex:0, openIndex:0}` + the `menu-item` **registers** (count 8, present in DOM *and* registry) → Esc → `menu{open:false, activeIndex:-1}` + item **unregisters** → back to **7/7/7, 0 orphans**. *Method note: Svelte 5 flips state synchronously but tears the popup DOM/registry down on the effect flush — a same-tick CDP read sees the state change without the mount/unmount. Read after settle.*
- **V8 console / permission (the D9 proof)** — error trap installed (`console.error` + `onerror` + `unhandledrejection`); after the real grip drag: `errLog:[]`, `errCount:0`, `permissionDenials:[]`. The resize capability is **granted, not silently failing**.
- **V9 grip resize (Joe's physical gesture — the one leg no CDP eval can fake; an OS drag loop cannot be synthesised)** — grip at the **exact SE corner** (`right:720`/`bottom:480` = the window edges), `cursor:nwse-resize`, `pointer-events:auto`, `aria-hidden:"true"`. Joe grabbed and dragged: window CSS size **720×480 → 743×470**. Joe: *"grip works."* The `onpointerdown → onResizeGrip → startResizeDragging('SouthEast')` seam is live end-to-end.
- **V11 build (Clair)** — `vite build` clean, **138 modules** (129→138: the status-bar subtree + the lazy `window-*.js` chunk for the grip import now enter the client bundle). Only the two pre-existing `icon.svelte` a11y warnings (J-489); none on the touched files.
- **V12 exit (Clair, destructive, once — N-086)** — Ctrl+Q → `defaultPrevented:true` (keymap matched → `app.exit` → `handleQuit`), pid **46932 → gone**, port 9222 down. With the Quit button removed this is the only in-app exit, and it works.
- **V5 (drag region)** — **moot**, dropped with the pivot.

**Records.** `docs/xgen-client-frame-phase0.md` **v1.4→v1.5** (§10.3 revised — native decorations as a temporary development affordance; frameless endpoint intact, sequenced to M-RP8; D2 dormant, D3 grip retained; §6 6.1e-B ✅ DONE). `docs/ROADMAP.md` **v4.63→v4.64** (M-RP6.1e-B ✅ DONE; **+M-RP8 `title-bar` + frameless restore** 🟡 PENDING; **+M-RP6.1e-B1 no-select chrome** 🟡 PENDING; next-active 6.1e-C). `ui/docs/xgen-ui-notes.md` **+N-088** (v0.71→v0.72). `CLAUDE.md` PLAY (head J-494→J-495). `.gitignore` += `*.lnk`. `tasks/M_RP6_1E_B_CLIENT_FRAME.md` → COMPLETED. This entry. **`ui/docs/xgen-ui-components.md` deliberately untouched** — no component was built or changed; the sampler catalogue registry stays **309**. **No new D** (D-107 extension). Clair feat = commit 1 (pushed `8f401bd`); this doc-bridge = commit 2. Not pushed — Joe pushes.

**New backlog filed.**
- **M-RP8 — `title-bar` `core` + frameless restore** 🟡 PENDING, scheduled **after the widget grid is live on BOTH apps**. Drag-region root + app title + minimise/maximise/close seams (the `onResizeGrip` shape) + room for extra buttons + 4 new `icons.ts` glyphs + `decorations:false` in both apps + capabilities re-add. Sequenced last **deliberately**: by then the client and node frames are structurally identical, so it is one component and two mounts rather than building it twice. Known cost: a custom caption loses Windows Snap-Layouts unless re-implemented (the standard price of custom chrome).
- **M-RP6.1e-B1 — no-select chrome** 🟡 PENDING (Joe, this session). Global `user-select: none` in `skin.css` (**L2** — it passes the remove-the-rule litmus: delete the skin and native selection returns, nothing breaks). Opt-in `user-select: text` on **`input` / `textarea` / `[contenteditable]`** (**mandatory, not cosmetic** — an ancestor `none` breaks caret and drag-select inside editable natives in WebView2; this is what keeps `textfield`/`textarea`/`number`/`converter-field`/`combobox`/`tag-select` working) and on **`.paragraph`** (the prose primitive — and therefore the **message body**, since `message` composes `paragraph`: users must be able to copy a message). Everything else is chrome and is not selectable. Extras: `-webkit-user-drag:none` on `img`; `::selection` tied to `--accent2`. Touches `skin.css` (core territory, fenced out of 6.1e-B by D10) → its own commit, sampler-verified.

**Deferred (D-065).** Full-edge four-side resize (SE grip is v1, and native edges cover it while decorated); `secondaryText`/version tag → 6.1e-C; the 900×600-logical-vs-physical DPI wrinkle; the node app's own frame (menu-bar + status-bar, its own milestone, **after 6.1f** so it inherits the region shell rather than building the frame twice).

**Next-active.** **M-RP6.1e-C** — `dialog`/`modal` `core` (flagged since J-432) + **Help→About**. The 2nd populated menu → the first real `menu-bar` roving Left/Right AND the trigger for the deferred shared-W-2/owned-popup extraction (N-086). **About scope grew (Joe, this session, per a reference screenshot):** name · version · a **link** · hi-res logo · **Built** date · Rust/Tauri/Svelte versions · Platform · app directory · data/config paths · a **Close** button (not "OK"). Everything below "Built" needs a **new Tauri read verb** (`get_about_info`) — the frontend cannot see build metadata or filesystem paths. That is **real Rust work in 6.1e-C**, not just a `dialog` component; its runbook must scope it.


---

## Entry J-494 — M-RP6.1e-A `status-bar` core (sb-cell + status-indicator + resize-grip seam): built + CDP-verified, M-RP6.1e-A CLOSED

**Doc-bridge (D-074 second commit).** Clair's feat (code-only, 4 files) is already pushed = commit `afcfaff` (commit 1); this entry + the paired canonical records = commit 2. Per-component design (Joe-locked "lock all by your recomms") built + CDP-verified against the live sampler (9422, both accents). **M-RP6.1e-A CLOSED** — the first step of the M-RP6.1e client-frame consolidation split (J-493) is down; this IS a catalogued `core` cell (unlike the 6.1d menu family — frame chrome). Sampler catalogue registry **299→309**.

**What `status-bar` is.** A **di-composite** `core` component — the fixed bottom-pane strip: side-stacking `sb-cell` groups + `separator`s + an always-visible SE resize-grip. Root `<div class="status-bar">`. Composes the real `status-indicator` (left cell) which itself brings `led` + `label` → a composite cell yields multiple registry entries (the status-indicator/message precedent, matrix multiplies). Imports **no Tauri / no protocol** — the grip exposes an `onResizeGrip?` seam the consuming shell wires at 6.1e-B (`startResizeDragging`).

**Clair's build calls (flagged, Joe-latitude in the runbook — all recorded).** (1) **`sb-cell` = internal, NON-registering layout part** (own file, no `use:envelope`) — a value-less flex group whose entire contribution is CSS positioning (`.sb-cell[data-side]`); a registry getter would be pure ordinal noise → the **N-064 used-internally-without-registration** pattern (chip `register` opt-out precedent). It exists as a file for readability + the "hosts any display component" `children`-snippet contract, not to add a matrix row. (2) **Grip glyph = pure-CSS corner triangle** (`clip-path: polygon(100% 0, 100% 100%, 0 100%)` in `--t4`), not an `icon` — the grip is a positioned corner affordance, not an inline-with-text glyph; a CSS triangle keeps it wholly in `skin.css` with zero `icons.ts` churn and no `Icon` import inside `status-bar` (noted for the icon-adoption backlog; lighter + cleaner fit here). (3) **2nd left cell via `secondaryText?` prop** → renders a vertical `separator` + a `label` in the left cell (exercises the vertical separator; the `#secondary` sampler cell uses `secondaryText="v0.10.3"`). (4) Added a **`grip` boolean prop** (default `true`) so `hasGrip` is honestly prop-driven (also lets a non-resizable host — e.g. the node app — drop it). **A11y honesty:** the grip is pointer-only, `aria-hidden="true"`, `onpointerdown → onResizeGrip?.(e)` — window resize has no keyboard equivalent (OS concern); not faked.

**Getter G** `{ leftCount, rightCount, hasGrip }` — `leftCount = secondaryText ? 2 : 1`, `rightCount = grip ? 1 : 0`, `hasGrip = grip` (colours/captions live on the child entries, no duplication). New skin tokens `--fs-s1: 9px` / `--fs-s2: 8px` below `--fs-0: 10px`; the status-bar caption defaults to `--fs-s1`.

**CDP verification (sampler 9422, both accents — Rule 2, real output). Chat re-drove the loop (D-097; launched detached, harness attach, cleaned up 0-orphan-port).**
- **Registry:** `ids().length` **299→309 (+10)**; `count===unique===domCount` (309/309/309), **0 orphans both directions** (`orphanRegNotDom:[]`, `orphanDomNotReg:[]`). The exact +10 subtree (sorted, matches Clair's structural prediction precisely — `sb-cell` + grip do NOT register):
  ```
  label#default__status-indicator__label   led#default__status-indicator__led   status-bar#default   status-indicator#default__status-indicator
  label#secondary__secondary   label#secondary__status-indicator__label   led#secondary__status-indicator__led   separator#secondary__sep   status-bar#secondary   status-indicator#secondary__status-indicator
  ```
  (`#default` = status-bar + status-indicator + its led + label = 4; `#secondary` = the same 4 + `separator#secondary__sep` + `label#secondary__secondary` = 6.)
- **Getter G:** `status-bar#default → {leftCount:1,rightCount:1,hasGrip:true}`, `status-bar#secondary → {leftCount:2,rightCount:1,hasGrip:true}` — exact.
- **Structure:** root `DIV.status-bar`; 4 `sb-cell` `SPAN`s, `data-side` = `left`/`right`/`left`/`right` (2 status-bars × left+right); grip `SPAN.sb-grip`, `aria-hidden="true"`.
- **Tokens:** `--fs-s1`=**9px**, `--fs-s2`=**8px** (below `--fs-0`=10px); the status-bar caption (`.status-bar .label`) computed `font-size` = **9px**.
- **Grip seam:** a bubbling `pointerdown` dispatched on `.sb-grip` reached the element (capture-probe `probeFired:1`) — since Svelte 5 delegates `pointerdown` at root, the bubbling dispatch invoked `onResizeGrip` (`dispatchThrew:null` — the stub ran clean); grip sits inside `.sb-cell[data-side=right]`; clip-path `polygon(100% 0px, 100% 100%, 0px 100%)` (SE corner). Source seam confirmed in `status-bar.svelte`: `onpointerdown={(e) => onResizeGrip?.(e)}`, imports = envelope/SbCell/StatusIndicator/Separator/Label only (**no Tauri**).
- **Separator:** `separator#secondary__sep` → `role="separator"`, `aria-orientation="vertical"` (the vertical divider between the two left cells).
- **Skin cascade:** `.status-bar` · `.status-bar .label` · `.sb-cell` · `.sb-cell[data-side="right"]` · `.sb-grip` all present in the stylesheets.
- **Accent-neutral (inject-and-restore `--accent2`):** grip bg `--t4` `rgb(88,92,100)`, grip color `rgb(200,196,188)`, separator border `--s5` `rgb(52,59,71)`, led ON `rgb(34,197,94)` — **all unchanged** under an injected `--accent2=#3a7ab0` (`neutralHeld:true`); restored to `#c28840`. This composition has **no accent-carrier** (the status-indicator has no `link`), so both accents render identically — the strip is accent-neutral chrome.
- **Eye-check:** geometry-covered by the structural CDP (grip in the right cell + SE-corner clip-path + caption 9px + vertical separator between two left cells); harness screenshot path flaky (J-489/J-490 precedent), not run.

**Build:** `vite build` clean — **166 modules** (164→166, the two new `.svelte` files); no warnings on any new file (only the pre-existing meter/entity-avatar a11y warnings remain).

**Records.** `ui/docs/xgen-ui-components.md` **v0.64→v0.65** (`status-bar` = 30th `core`, di-composite; `sb-cell` internal non-registering note; sampler registry 299→309). `docs/ROADMAP.md` **v4.62→v4.63** (M-RP6.1e-A ✅ DONE; next-active 6.1e-B). `ui/docs/xgen-ui-notes.md` **+N-087** (v0.70→v0.71). `docs/xgen-client-frame-phase0.md` **§4.5 build-note in-place** (v1.3→v1.4 — the resolved build calls). `CLAUDE.md` PLAY (head J-493→J-494; sampler registry 309; next-active M-RP6.1e-B). `tasks/M_RP6_1E_A_STATUS_BAR.md` → COMPLETED. This entry. **No new D** (D-107 extension). Clair feat = commit 1 (pushed `afcfaff`); this doc-bridge = commit 2. Not pushed — Joe pushes.

**Deferred (D-065):** the real-client mount + `.state-indicator` → `status-indicator` migration + `onResizeGrip` → `startResizeDragging` wiring + window-config flips + center-only scroll + logo/Quit removal (all 6.1e-B); full-edge resize; the node app's own status-bar.

**Next-active.** **M-RP6.1e-B** real-client frame consolidation (9222, no sampler): mount the status-bar bottom, migrate the hand-rolled `.state-indicator` → `status-indicator`, wire the grip → `startResizeDragging`, center-only scroll, remove the center logo + redundant Quit, and the window-config flips (`resizable:true`, menu-bar drag-region, 900×600 / min 640×400). Then 6.1e-C `dialog` + Help→About. `temperature-indicator` (M-RP6.5) stays ⏸️ POSTPONED.


---

## Entry J-493 — M-RP6.1e client frame consolidation split (6.1e-A / -B / -C) LOCKED + window-config grounding — design/records-only

**Design/records-only, no code.** Joe's "consolidation of the app's main UI" resolved into a locked sub-milestone split + the window-config decisions it needs. Grounded against the **real** client files first (Rule 5), then locked "all by your recomms." Extends the frame concept (D-107 / J-488); **no new D**. Component registry unchanged **299** (no component built here).

**Grounding read (real files).** `xgen-client/tauri.conf.json`: the client window is **frameless** (`decorations:false`), **`resizable:false`**, **420×260**, centered — the old lean-chrome window. No native title bar → no native edge-resize, and currently no way to *move* the window either. `ui/client/src/app_client.svelte`: all legacy chrome sits inside `#core-ui-pane` — a hand-rolled `<img id=app-logo>`, a hand-rolled `.state-indicator` (local `dotColor()` state→colour map + `isPulsing()` + `currentState.label`), and the redundant `<Button id=quit>`. The quit seam is `invoke('quit')` (confirmed — the keymap + File→Exit already reuse it). The `.state-indicator` maps **cleanly** onto `status-indicator`: `dotColor` → `led`'s `states` colour-map, `isPulsing` → `led` `pulse`, `currentState.label` → the `label`.

**The split (within the already-Phase-0'd frame arc — a sequence lock, not a new Phase-0).** M-RP6.1e becomes three:
- **6.1e-A `status-bar` core** — the component: `sb-cell` + `separator` + `onResizeGrip?` seam + `--fs-s1`/`--fs-s2` tokens. **Left cell = a `status-indicator`, right cell = the SE resize-grip** (Joe-locked contents). Pure chrome → **sampler-cell + CDP verified** (the grip seam is inert in the sampler; that's fine — the component is data-independent). Fully specced now → **next-active**; runbook `tasks/M_RP6_1E_A_STATUS_BAR.md` written this entry.
- **6.1e-B client frame consolidation** — real-client assembly (9222, no sampler): mount the status-bar in the bottom pane; **migrate** the hand-rolled `.state-indicator` → `status-indicator` in the left cell (reading `currentState`); wire the grip seam → `startResizeDragging` (SE corner); **confine scroll to the center pane only** (the M-RP4.9/J-466 `height:100vh` flex-column pattern — root `overflow:hidden`, chrome panes `flex:0 0 auto`, center `flex:1;min-height:0;overflow-y:auto`); **remove** the center logo + the redundant Quit button (D-065 cleanup — File→Exit is the exit); the center becomes a placeholder leaf until 6.1f. Plus the window-config flips below.
- **6.1e-C `dialog` core + Help→About** — build the `modal`/`dialog` `core` component (flagged since J-432); add a **Help** menu (the **2nd populated menu** → the first real `menu-bar` roving Left/Right AND the trigger for the **deferred shared-W-2/owned-popup extraction**, N-086 — both `menu` + `entity-context-menu` then adopt the extracted module, or `menu`'s fresh-minimal machine proves it generalises); About = app name + version + authors + hi-res logo, **version read from the real build** (Cargo/Tauri), not hardcoded. Real client.

**Window-config decisions (locked "by recomms," settle in 6.1e-B):**
- **`resizable: true`** — flip it on (the whole point).
- **Drag-to-move** — frameless has no title bar, so make the **menu-bar strip a drag region** (`data-tauri-drag-region` on the bar background; the interactive `<button>` triggers override, so clicks still open menus). Gives window-move back.
- **Default + min size** — default **900×600** (a real main window, not 420×260), **min 640×400** (the frame's composition floor). Tunable in the runbook.
- **Resize mechanic v1** — frameless + `decorations:false` means the OS draws no resize borders, so **SE-grip `startResizeDragging` is the resize affordance** (matches Joe's "grip on the right of the status-bar"). Full invisible-edge-drag resize on all four edges is a **deferred polish** (D-065).

**The logo.** Joe's call: the logo does **NOT** live in the frame chrome — it goes in **Help→About** (the classic small modal: name, version, authors, hi-res logo, close button). So the menu-bar gains no logo slot; the About modal (6.1e-C) is its home. This is why 6.1e-C builds `dialog`.

**Records.** `docs/xgen-client-frame-phase0.md` — grounding + the A/B/C split + the window decisions + the About-holds-the-logo call recorded (§4.1 Help menu, §4.5 status-bar contents locked, §6 sub-milestone split, new §10 consolidation grounding); version bump. `docs/ROADMAP.md` — 6.1e → 6.1e-A/-B/-C, 6.1e-A next-active; version bump. `CLAUDE.md` PLAY (head → J-493; next-active 6.1e-A). `tasks/M_RP6_1E_A_STATUS_BAR.md` — the 6.1e-A build runbook (ACTIVE). This entry. **No code, no new D** (D-107 extension). Not pushed — Joe pushes.

**Next-active.** **M-RP6.1e-A `status-bar` core** (Clair build → sampler CDP → close), then 6.1e-B real-client consolidation, then 6.1e-C `dialog` + Help→About.


---

## Entry J-492 — M-RP6.1d `menu-bar` minimal (core trio) + keymap wiring: built + real-client-verified, M-RP6.1d CLOSED

**Doc-bridge (D-074 second commit).** Clair's feat (code-only, 6 files) is pushed = commit `5432d25` (commit 1); this entry + the paired canonical records = commit 2. **First frame step to touch the real client shell** — the menu family is **frame chrome, not sampler cells** (Joe-locked): built as `core`, mounted into the client's fixed top pane, and verified in the **real client (9222)** via the restored CDP harness (M-RP-CDP1), not the sampler. **M-RP6.1d CLOSED.** **Sampler catalogue registry unchanged 299** (no sampler file touched; the menu trio lives in the *client's own* small registry).

**What shipped (feat `5432d25`, code-only, 6 files).** `menu-item.svelte` (`<li role=menuitem>` = optional `icon` slot + label + trailing `Accelerator` hint via `toDisplay`; roving-active flag owned by `menu`) · `menu.svelte` (trigger `<button role=menuitem>` + owned `<ul role=menu>` popup + a **fresh-minimal, self-contained** open/rove/dispatch/dismiss/focus-return/outside-click machine, listener wired-on-open/torn-down-on-close; open is parent-controlled for single-open mutual exclusion — **NOT** a refactor of the closed `entity-context-menu` widget) · `menu-bar.svelte` (`<div role=menubar>`, openIndex/activeIndex ownership + roving Left/Right) · `skin.css` (accent-neutral `.menu-*` L2, inline-absolute dropdown, no portal) · `app_client.svelte` (`KeymapRegistry` singleton + command table `{app.exit: handleQuit}` reusing the existing `invoke('quit')` seam — Rule-5-confirmed — + `Ctrl+Q → app.exit` + one window `keydown` listener; menu-bar mounted in a new top-pane frame, existing Quit button left intact) · `app.css` (full-height column frame: menu-bar top over the content body).

**Verify — real client 9222 (D-097 graduation; Rule 2, real output).** Chat independently re-drove the non-destructive legs against Joe's live client (`-Debug` overlay); the destructive quit + focus-return legs are Clair's in-session verify (the long-running client can't be relaunched via MCP for a Chat re-drive, and re-testing the quit would kill Joe's session).
- **Chat-driven (this session):**
  - Registry (client's own), menu closed: `["menu#app-menubar__file","menu-bar#app-menubar","button#quit"]` — the trio's bar + File menu self-registered, plus the intact Quit button.
  - Getters closed: `menu-bar → {items:1,activeIndex:0,openIndex:-1}`, `menu#…__file → {label:"File",open:false,itemCount:1,activeIndex:-1}` — exact.
  - Open (clicked `.menu-trigger`): `menu#…__file → {label:"File",open:true,itemCount:1,activeIndex:0}` (open + focus-in); the item **registers on open** — `menu-item#app-menubar__file-exit → {label:"Exit",hasIcon:false,accel:"Ctrl+Q",disabled:false}`; `.mi-accel` textContent = **"Ctrl+Q"** (the `Accelerator.toDisplay('win')` hint surfacing in the real client); popup `role="menu"`, item `role="menuitem"`.
  - Esc dismiss: `menu#…__file → {open:false,activeIndex:-1}`, the item **unregisters** — registry back to the 3 closed ids (**0 orphans**).
- **Clair-verified in-session (attributed, not Chat-re-driven):** focus returns to the trigger on Esc (under Chat's *synthetic* Esc, `activeElement` did not report the trigger — programmatic-focus flakiness, the N-029-shape synthetic-event caveat; Clair confirmed real focus-return); **both quit paths end-to-end** — File→Exit select quit the window (launcher exit 0), `Ctrl+Q → defaultPrevented:true` + `xgen-client` process 1→0 + CDP port gone (each quit done once, not looped); `vite build` clean — **129 modules** (two pre-existing `icon.svelte` warnings, J-489); sampler registry unchanged **299**. Screenshots `menu-closed.png` / `menu-open.png`.

**Two session gotchas (→ N-086).** (1) `__TAURI_INTERNALS__.invoke` is **non-configurable** — the quit can't be intercepted non-destructively, so each quit path is proven with one deliberate real quit (never looped). (2) PS 5.1's `ConvertTo-Json` **mangles a bare string** returned to CDP `Runtime.evaluate` (surfaced as a spurious `EVAL ERROR` on a bare-string return whose side effect still fired) — wrap eval returns as a JSON object, not a bare string.

**The machine — fresh-minimal, not a refactor (Rule 6 refinement).** frame-phase0 §4.1 said `menu` "reuses the `entity-context-menu` W-2 machine"; the machine actually lives **entirely inside that closed, verified widget**, interwoven with its dd concerns. Clair built a **parallel minimal machine** in `menu` (no `entity-context-menu` change). §4.1 refined **in-place** (v1.1→v1.2): build minimal now; **extract a shared W-2/owned-popup module when the 2nd populated menu or the submenu-flyout lands** (both `entity-context-menu` + `menu` then adopt it). Arc-local, **no new D** (the J-490/J-491 doc-wording-fix precedent).

**Records.** `ui/docs/xgen-ui-notes.md` **+N-086** (v0.69→v0.70). `docs/xgen-client-frame-phase0.md` **§4.1 refined in-place** (v1.1→v1.2). `ui/docs/xgen-ui-components.md` **v0.63→v0.64** (frame-chrome menu trio note — authored `core`, **not sampler cells**, real-client-verified; sampler registry 299 unchanged). `docs/ROADMAP.md` **v4.60→v4.61** (M-RP6.1d ✅ DONE; next-active 6.1e). `CLAUDE.md` PLAY (head J-491→J-492; the client now carries a top-pane menu-bar + live Ctrl+Q exit; sampler registry 299; next-active M-RP6.1e). `tasks/M_RP6_1D_MENU_BAR.md` → COMPLETED. This entry. **No new D.** Deferred (D-065): submenu-flyout, `menu-separator`, `menu-check-item`, portal, the shared-W-2 extraction, the sampler frame-window incorporation, removing the redundant Quit button, the node app's menu-bar. Clair feat = commit 1 (pushed); this doc-bridge = commit 2. Not pushed — Joe pushes.

**Next-active.** **M-RP6.1e `status-bar`** (`core`; side-stacking `sb-cell` + `separator`s + our own always-visible SE resize-grip via an `onResizeGrip?` seam the shell wires to Tauri; new skin tokens `--fs-s1:9px`/`--fs-s2:8px` below `--fs-0:10px`; the node app needs it too). `temperature-indicator` (M-RP6.5) stays ⏸️ POSTPONED.


---

## Entry J-491 — M-RP6.1c `Accelerator` + `KeymapRegistry`: built + vitest-verified, M-RP6.1c CLOSED

**Doc-bridge (D-074 second commit).** Clair's feat (code-only, 8 files) is already pushed = commit `6993b61` (commit 1); this entry + the paired canonical records = commit 2. The per-component design — Clair's grounded design walk + two Chat tightenings, Joe-locked "as you recommend" — is built and **vitest**-verified (this is the first milestone verified by unit tests, not the CDP loop; the objects are pure/DOM-free with no envelope). **M-RP6.1c CLOSED** — the frame arc's third prerequisite is down. Component registry **unchanged 299** (no envelope, no sampler cell).

**What they are.** Two pure, DOM-free objects in `$common` (`ui/common/lib/keymap/`), **not** visual components. `Accelerator` — the one-definition-two-projections value-object (the `Converter<T>` shape): one authored binding (`accelerator("Ctrl+Q")`) projects to **display** (`toDisplay(platform)` → the future menu-item hint) and **dispatch** (`matches(event, platform)` → the keydown predicate), so display and dispatch can never drift. `KeymapRegistry` — the pure binding table + `resolve(event) → commandId | null`.

**Locked design (built to spec).** `Ctrl` ≡ `Control` = the **platform-accelerator (shortcut) token** (`ctrlKey` on win, `metaKey`/⌘ on mac); deliberately no literal-ctrl token. `Cmd`/`Command`/`Meta`/`Super` → literal `meta`; `Alt`/`Option` → `alt`; `Shift` literal. **Platform is a parameter** (`'win'` default — the only shipped target; no `navigator` read). Canonical `{accel, meta, alt, shift, key}`; key normalization (letters upper-cased, named-key aliases `Esc`/`Del`/`Up`/`Space`… + `F1`–`F24` → `event.key` form). **Exact-modifier match** (Shift held on a plain `Ctrl+Q` suppresses). `toDisplay('win')` = `+`-joined words (literal meta shown `"Win"`), `toDisplay('mac')` = Apple-HIG glyphs `⌃⌥⇧⌘`. `matches` typed against a structural `KeyEventLike` (`Pick` of 5 fields) → DOM-free, unit-testable with a plain object literal. Parse **throws** on malformed (Tier-1 trusted code, fail-fast). `KeymapRegistry`: `register` throws on a duplicate canonical (`Accelerator.toString()`), `resolve` first-match-wins, + `has`/`size`/`list`; bindings carry command **ids** (`"app.exit"`), not handlers. **D3 split:** the pure table lives in `$common`; the shell owns only the singleton instance + binding population + the one `keydown` listener → **6.1d**. Scope narrowed at lock: no `ui/client` touch, no listener/Exit wiring this milestone (Phase-0 §7 schedules only a pure-unit leg here).

**What Clair built (feat `6993b61`, code-only, 8 files).** `ui/common/lib/keymap/accelerator.ts` (the value-object) · `registry.ts` (the pure `KeymapRegistry`) · `accelerator.test.ts` + `registry.test.ts` (the suites) · sampler `vitest.config.js` + `package.json` + `package-lock.json` (the D4-V1 harness) · `ui/common/tsconfig.json` (exclude `lib/**/*.test.ts`). `vite build` clean (sampler, 772ms, no regression — keymap not yet imported by any cell).

**Verify (vitest — Rule 2, real output). Clair ran it green in the feat; Chat independently re-ran `npm test` at verify — also 35/35:**
```
 ✓ ../common/lib/keymap/accelerator.test.ts (27 tests) 4ms
 ✓ ../common/lib/keymap/registry.test.ts (8 tests) 4ms

 Test Files  2 passed (2)
      Tests  35 passed (35)
```
(exit 0; `vitest run` v3.2.7, sampler pkg.) Coverage: parse (Ctrl/Mod-shortcut/Cmd-literal/named-key/F-keys + throws on empty/empty-token/unknown-mod/multi-key), `toDisplay` win-words + mac-glyphs, `matches` exact-modifier + platform projection + case-insensitive key, `KeymapRegistry.resolve` first-match + `null` miss + dup-throw.

**Two Clair Rule-6 flags (beyond the runbook file list).** (1) `vitest.config.js` needed `server.fs.allow: [repo-root]` — the suites live in `../common`, outside the sampler root, and Vite refused to load them until the allow-list widened. (2) `ui/common/tsconfig.json` gained `exclude: ["lib/**/*.test.ts"]` — the test files fell under its `lib/**/*.ts` include, which would drag `vitest` into `ui/common`'s production type graph (unresolvable). Both accepted (sensible hygiene, production config stays production-only).

**Records.** `ui/docs/xgen-ui-notes.md` **+N-085** (v0.68→v0.69). `docs/xgen-client-frame-phase0.md` **§4.4 refined in-place** (v1.0→v1.1 — the D3 split: `KeymapRegistry` pure table + `resolve` in `$common`, only instance+listener shell; the J-487/J-490 doc-wording-fix precedent). `ui/docs/xgen-ui-components.md` **v0.62→v0.63** (keymap value-objects note; **not catalogued** — not `core` components; registry unchanged 299). `docs/ROADMAP.md` **v4.59→v4.60** (M-RP6.1c ✅ DONE; next-active 6.1d). `CLAUDE.md` PLAY (head J-490→J-491; registry 299; next-active M-RP6.1d). `tasks/M_RP6_1C_ACCELERATOR.md` → COMPLETED. This entry. **No new D** (§4.4 concept already under D-107; the D3 refinement is arc-local). Clair feat = commit 1 (pushed); this doc-bridge = commit 2. Not pushed — Joe pushes.

**Next-active.** **M-RP6.1d `menu-bar` minimal** (File→Exit) — where the shell finally wires the keymap: a singleton `KeymapRegistry` + `register(accelerator("Ctrl+Q"), "app.exit")` + the single `keydown` listener → `resolve` → run; reuses the `entity-context-menu` W-2 machine; the `menu-item` accelerator hint reads the same `Accelerator`. `temperature-indicator` (M-RP6.5) stays ⏸️ POSTPONED.


---

## Entry J-490 — M-RP6.1b `separator`: built + CDP-verified, M-RP6.1b CLOSED

**Doc-bridge (D-074 second commit).** Clair's feat (code-only, 3 files) is already pushed = commit 1; this entry + the paired canonical records = commit 2. Per-component design (Joe-locked "go") built + CDP-verified against the live sampler (9422, both accents). **M-RP6.1b CLOSED** — the frame arc's second prerequisite is down. Registry **293→299**.

**What `separator` is.** The **29th `core`** component, a **di** display-kind primitive, and the **first value-less component** in the library (getter is config-only — no value/binding/interaction). One component used everywhere — the menu-divider and the status-bar cell-divider are the same thing (Phase-0 §4.4, built once; D-096 fold cleared, **no new D**). Root **`<div role="separator">`** — deliberately NOT `<hr>`: chosen so the same component is valid both in the flex status-bar AND as a direct child of a future `<ul role="menu">` (an `<hr>` is not a valid `<ul>` child; a `role="separator"` div is) → one root, every context, no branch. Props `orientation?: 'horizontal'|'vertical'` (default horizontal) → `data-orientation` + `aria-orientation`; `variant?: 'line'|'double'|'gap'` (default line) → `data-variant`; `id`. Getter G `{orientation, variant}`.

**N-020 litmus refinement (recorded).** The di `<div>`=composite litmus is about *native atom vs. assembled structure*, not literally “is it a div.” `separator` composes **nothing** (zero children, one registered id, single-leaf getter) → it is **di-atomic**, not composite, despite the `<div>` root. The `<div>` is a fallback because HTML has no inline-divider atom valid in a `<ul role="menu">` (`<hr>` would be the native atom but is invalid there). This is the di analogue of the N-075 dd-root carve-out: the discriminator is **composition**, and a `<div>` root chosen for tag-availability doesn't make an atom a composite. → **N-084**.

**Appearance = `skin.css` only (border-based).** Every visual lives in the `.separator` L2 block; component `<style>` empty. Border-based (not `background`) because `border-style: double` gives the two-line rule natively — horizontal → `border-top`, vertical → `border-left` (+ `align-self:stretch`), `gap` → `border:0` (pure spacing). Colour = the `--s5` hairline token (`#343b47`), confirmed against the live skin as the standard 1px border/groove colour (`.textfield`/`.number`/`.select` borders, `.range`/`.meter` grooves) → accent-neutral chrome.

**What Clair built (feat, code-only, 3 files).** `separator.svelte` (`<div role="separator">`, orientation/variant → `data-*`+`aria-orientation`, envelope getter, empty `<style>`) · `skin.css` `.separator` L2 block · `app_sampler.svelte` 4 DI Atomics cells + import. `vite build` clean (**164 modules**, +1 = separator.svelte, no new `.ts`). Scope-clean (only the 3 files).

**CDP verification (sampler 9422, both accents — all legs green).**
- **Registry:** `ids().length` **293→299 (+6)** — 4 `separator#` cells (`horizontal`/`vertical`/`double`/`gap`) + `label#1`/`label#2` (the two demo labels flanking `#vertical` in its flex row, sampler chrome per the runbook); `count===unique` (299/299), **0 orphans both directions**.
- **Getter G:** `{type:"separator",state:{orientation,variant}}` on every cell (config-only, the first value-less getter) — `#horizontal {horizontal,line}`, `#vertical {vertical,line}`, `#double {horizontal,double}`, `#gap {horizontal,gap}`.
- **Element/attrs:** every cell `tag=div`, `role="separator"`; `aria-orientation` + `data-orientation` + `data-variant` reflected + correct.
- **Variants (computed-style):** `line`→solid on the orientation edge (h `border-top`, v `border-left`); `#double`→`border-top-style:double`; `#gap`→border-width 0 (pure spacing). Widths render 0.8px/2.4px = the authored 1px/3px under a uniform 0.8× WebView device-pixel scaling (cosmetic, not a defect; the double still shows two lines).
- **Accent-neutral:** `--s5 = #343b47`; both `line` + `double` border colour = `rgb(52,59,71)`, **unchanged** under an injected `--accent2=#3a7ab0` swap (no gold↔blue follow — the led/meter chrome precedent). Sampler left clean.
- **Eye-check:** geometry-covered (double = two lines, gap = none, vertical stretches to row height); harness screenshot path flaky, not run.

**Records.** `ui/docs/xgen-ui-components.md` **v0.61→v0.62** (separator = 29th core, di-atomic; registry 293→299 with cause). `docs/ROADMAP.md` **v4.58→v4.59** (M-RP6.1b ✅ DONE; next-active 6.1c). `ui/docs/xgen-ui-notes.md` **+N-084** (separator / leanest di / first value-less component / `<div role=separator>` root for menu+status-bar reuse / N-020 litmus refinement / border-based double via skin / accent-neutral chrome). `CLAUDE.md` PLAY (head J-489→J-490; registry 293→299; next-active M-RP6.1c `Accelerator`). `tasks/M_RP6_1B_SEPARATOR.md` → COMPLETED. This entry. **No new D** (D-096 fold cleared at Phase-0 §4.4). Clair feat = commit 1 (pushed); this doc-bridge = commit 2. Not pushed — Joe pushes.

**Next-active.** **M-RP6.1c `Accelerator`** — ONE `ui/common` value-object (single definition → `toDisplay()` display + `matches(event)` dispatch, pure/DOM-free) + a lean shell keymap registry (starts with one binding Ctrl+Q→Exit). `temperature-indicator` (M-RP6.5) stays ⏸️ POSTPONED.


---

## Entry J-489 — M-RP6.1a `icon`: built + CDP-verified, M-RP6.1a CLOSED

**Doc-bridge (D-074 second commit).** Clair's feat (code-only, 6 files) is already pushed = commit 1; this entry + the paired canonical records = commit 2. The per-component design (Joe-locked "go by recomms") is built and CDP-verified against the live sampler (9422, both accents). **M-RP6.1a CLOSED** — the frame arc's first prerequisite is down. Registry **286→293**.

**What `icon` is.** The **28th `core`** component, a **di** display-kind primitive (no data-dependency — sibling of `label`/`image`/`led`), and the **first shape-definition value-type**. Inline `<svg>` glyph; `name` keys a bundled `icons.ts` registry (`d` string or `d[]`, rendered as `{#each}<path>` — **no `{@html}`**, N-032 lean held) with an optional raw `path` escape hatch (wins over `name`). Props: `size` `16|20|24` (default **16**); `tint` (default `currentColor` via inline `--icon-tint`, override hex/`var(--token)`); `label` — **decorative by default** (`aria-hidden="true"`), set → `role="img"`+`aria-label`. Getter `{name,size,tint,decorative}` (`name:"(path)"` on raw override). D-096-cleared vs `image` on two axes (shape-definition value-type, not `src`; tintable glyph, not raster) — already recorded at frame Phase-0 §4.3, **no new D**.

**Substrate generalized SVG-safe (Clair Rule-6 catch → Joe-locked Option B).** `icon` is the first SVG-rooted `core` component. `use:envelope` stamped the type-class via `node.className = mergeClasses(…)`, but on an `<svg>` root `node.className` is a read-only `SVGAnimatedString` (and `mergeClasses`' `.trim()` has no method there) → TypeError at mount; `vite build` (esbuild, types stripped) wouldn't catch it — it would have burned a CDP cycle. Fix (verified against `envelope.ts`/`logic.ts` before locking): widen the action param `HTMLElement`→`Element` and stamp via `setAttribute('class', mergeClasses(typeClass, node.getAttribute('class')))` — `getAttribute` is `string|null` on HTML+SVG (`mergeClasses` already handles null), `setAttribute` is identical to `.className` on HTML, so behaviorally identical for all 30+ HTML-rooted components; the debug branch already used `setAttribute`. Option A (wrapper `<span>`) was rejected — it contradicts the locked svg-root + fails the verify plan.

**What Clair built (feat, code-only, 6 files).** `icon.svelte` (svg root, name→registry / path override, `d[]` render, envelope getter) · `icons.ts` (registry, 3 fill-based 24-grid seeds: `caret-down`/`dot`/`square`) · 3 source `.svg` under `ui/assets/icons/` (provenance) · `skin.css` one `.icon` L2 rule (`fill: var(--icon-tint, currentColor)`) · `app_sampler.svelte` DI Atomics row (7 cells) · **`envelope.ts`** the SVG-safe generalization. `vite build` clean (**163 modules**, +2 = icon.svelte + icons.ts). Scope-clean.

**CDP verification (sampler 9422, both accents — all legs green).**
- **Registry:** `ids().length` **286→293 (+7)** — exactly the 7 icon cells (`default`/`s16`/`s20`/`s24`/`tinted`/`labelled`/`raw`); `count===unique` (293/293), **0 orphans both directions**.
- **Getter G:** every cell `{type:"icon",state:{name,size,tint,decorative}}`; `icon#default` `{caret-down,16,currentColor,true}` (default size 16 + decorative), `icon#labelled` `decorative:false`, `icon#raw` `name:"(path)"`, `icon#tinted` `tint:"var(--accent2)"`.
- **Element:** every cell `tag=svg`, `class="icon"` (the type-class stamped on an SVG root — the **live proof** the SVG-safe envelope landed; had it thrown, no stamp + no registration), `viewBox="0 0 24 24"`, `paths:1`; width/height **and** bounding box step **16·20·24**.
- **a11y:** decorative cells `aria-hidden="true"`/no role; `labelled` → `role="img"`+`aria-label="collapse"`.
- **Tint:** `icon#default` computed `fill` `rgb(200,196,188)` === its `color` (**follows `currentColor`**, accent-neutral); `icon#tinted` `fill` `rgb(194,136,64)` === `--accent2` `#c28840`; injecting `--accent2=#3a7ab0` → tinted follows to `rgb(58,122,176)` while default holds → **gold↔blue swap follows the token**; override removed → clean restore.
- **Envelope regression (substrate blast-radius):** `label` (`tag=label`,`class="label"`) + `led` (`tag=span`,`class="led"`) still stamp their type-class + register post-change — the attribute-based stamp is behaviorally identical on HTML roots.
- **Eye-check:** covered by geometry (non-zero 16/20/24 boxes + `paths:1` + correct fills); the harness screenshot path is flaky (transient exit-1), not run.

**Records.** `ui/docs/xgen-ui-components.md` **v0.60→v0.61** (icon = 28th core; registry 286→293 with cause). `docs/ROADMAP.md` **v4.57→v4.58** (M-RP6.1a ✅ DONE; + M-RP-ICON-ADOPT backlog pointer). `ui/docs/xgen-ui-notes.md` **+N-083** (icon = first shape-definition value-type / registry-keyed / anti-`{@html}` multi-path / fill-based tint; paired: drove the `envelope` SVG-safe generalization). `CLAUDE.md` PLAY (head J-488→J-489; registry 286→293; next-active M-RP6.1b `separator`). `tasks/M_RP6_1A_ICON.md` → COMPLETED (+§3/§4/§5 amended for the 6-file/envelope reality). Backlog milestone **`docs/xgen-icon-adoption.md`** (M-RP-ICON-ADOPT, PENDING, theory-open — glyph consolidation) filed + committed separately. This entry. **No new D** (D-096 already cleared). Clair feat = commit 1 (pushed); this doc-bridge = commit 2. Not pushed — Joe pushes.

**Next-active.** **M-RP6.1b `separator`** — shared `core`, orientation vertical|horizontal, used between status-bar cells and as `menu-separator` (frame build order step 2). `temperature-indicator` (M-RP6.5) stays ⏸️ POSTPONED.


---

## Entry J-488 — M-RP6.1 client-UI-frame Phase-0 LOCKED (app frame: menu-bar + status-bar)

**Design/records-only, no code (Rule 1/5).** Opened the M-RP6.1 D-071 Phase-0 gate for the client UI panel arc and ran a full brainstorm with Joe on the **app frame** — the fixed menu-bar (top) and status-bar (bottom) around the dockable region layout. Dependency (node↔client surface) confirmed **GO** at M-RP6.0/J-473 (`m-rp6.0-gate-go`); gate result **GO to design + build**. Finding F-1 (no live read verb pre-UI) is this milestone's deliverable, not a defect. Joe locked the frame concept + the component prerequisites + the frame-first build order. Registry unchanged (**286**) — no code this entry.

**Locked (Joe) — the app frame.**
1. **Frame = fixed chrome, NOT dockable regions.** BorderPane: menu-bar top pane + status-bar bottom pane are fixed and live **OUTSIDE** the `Layout` descriptor; only the center is subdivided by the descriptor (renderer A now → dock engine B at M-RP7). Consequence: File→Exit is unconditionally reachable (outside the dock tree — stronger than W-13). → **D-107**.
2. **Frame containers are `core`; window-effects are shell-wired.** The status-bar (and menu family) are reusable `core` components — the node app needs an un-minimalized status-bar too. `core` imports no Tauri, so real-window effects ride seams: status-bar resize-grip via `onResizeGrip?` (shell → `startResizeDragging`); menu Exit via a command callback (shell → existing exit command).
3. **`icon`** = its own `core` component (NOT folded into `image`) — cleared by D-096 on **two** axes: value-type (a shape/path definition, not a `src` reference) and surface (tintable UI glyph vs raster content). Mirrors JavaFX `SVGPath`-vs-`ImageView` (verified: JavaFX `ImageView` is raster-only, SVG is a separate `SVGPath` Shape). Primary path inline `<svg>` (tintable via `currentColor`), raster `<img>` secondary. png/jpg/svg only, no `.ico`.
4. **`separator`** = shared `core` component, orientation vertical | horizontal — used between status-bar cells AND as `menu-separator`. Built once.
5. **Menu-bar** minimal now (File→Exit); JavaFX-standard taxonomy (`menu-bar`/`menu`/`menu-item`/`menu-separator`/`menu-check-item`), fully skinnable / no native; reuses the `entity-context-menu` W-2 behaviour machine. Grows by accretion (separator / check-item / **submenu-flyout** deferred until a 2nd menu needs them, D-065). The menu bar doubles as a live dev-state signal.
6. **`Accelerator`** = ONE `ui/common` value-object, two projections from a single definition (`toDisplay()` display hint + `matches(event)` dispatch) → no display/dispatch drift; pure/DOM-free. Consumed by a **lean** shell-level keymap registry (built full object now, registry starts with one binding Ctrl+Q→Exit).
7. **`status-bar`** = `core` container: side-stacking `sb-cell`s (left/right) + `separator`s + our own always-visible SE resize-grip (our glyph, Tauri mechanism via the seam). Connection cell = `status-indicator` (led+label) reading the SAME reactive `self-state` signal as R3 (single source of truth). Default text `--fs-s1` (9px), tune to `--fs-s2` (8px) if needed.
8. **Font tokens** (verified against `skin.css`, Rule 5 — real scale is `--fs-0:10px; --fs-1:12px; --fs-2:14px`): add **`--fs-s1: 9px`** and **`--fs-s2: 8px`** below `--fs-0`, additive, no rename of the shipped scale. General L2 tokens (dense UI wants sub-10 elsewhere too).

**Decision (D-series).** **D-107** — app frame (menu-bar + status-bar) = fixed chrome outside the dockable region layout; frame containers are `core`, window-effects are shell-wired. Extends D-103 (which had no frame concept).

**Revised M-RP6.1 build order (frame-first; each step gets its own design lock + runbook when it opens).** 6.1a `icon` core · 6.1b `separator` core · 6.1c `Accelerator` + lean keymap registry · 6.1d `menu-bar` minimal (File→Exit) · 6.1e `status-bar` core · 6.1f center region-shell scaffold (renderer A reads descriptor, `get_layout` stub→default, placeholder leaves) + selection bus · 6.1g R3 Self/connection live (`get_self_state` verb + scoped `app.emit` push + `listen`; closes F-1 read half) · 6.1h R8 inspector on the selection bus (generic `EntityDescriptor` rows, self = first inspectable). Verify **graduates from the sampler to the real client app** (the sampler is `tauri`+`tauri-build` only, cannot reach a node — D-097): three-layer = pure unit (vitest) / real-client-offline (fixture self) / real-client + node (stop-node → led flips via the emit push).

**Records.** `docs/xgen-client-frame-phase0.md` **v1.0** (new, the frame Phase-0). `DECISIONS.md` **+D-107**. `ui/docs/xgen-region-dock-model.md` **v1.1→v1.2** (+§10 frame concept). `docs/ROADMAP.md` **v4.56→v4.57** (M-RP6.1 re-sequenced frame-first). CLAUDE.md PLAY (head pointer J-487→J-488, next-active M-RP6.1a `icon` build). This entry. No code, registry unchanged (**286**). Not pushed — Joe pushes.

**Next-active.** **M-RP6.1a `icon` build** — first frame prerequisite (per-component design lock → Clair feat → Chat verify → D-074 close). Submenu-flyout / menu-separator / check-item deferred (D-065). `temperature-indicator` (M-RP6.5) stays ⏸️ POSTPONED.


---

## Entry J-487 — M-RP5.7 grouped-avatar suppression: built + CDP-verified, M-RP5.7 CLOSED

**Doc-bridge (D-074 second commit).** Clair's feat `fd0af23` (code-only, `message.svelte` + `skin.css`) is already pushed = commit 1; this entry + the paired canonical records = commit 2. The grouped-avatar suppression (Phase-0-locked at J-486, D-106) is built and CDP-verified against the live sampler (9422, both accents). **M-RP5.7 CLOSED** — grouping now reads correctly. Registry **296→286**.

**What Clair built (feat `fd0af23`).** `message.svelte`: the grouped-branch guard widened `{#if author}` → `{#if author && !grouped}`, so a `grouped` continuation renders **no** `entity-avatar` child (element absent, not `visibility:hidden`) — the name was already dropped at B, so a grouped cell now registers **neither `__avatar` nor `__name`**; also fixed the stale header doc-comment that still said grouped “keeps the avatar.” `skin.css`: the `.message` avatar grid track was `auto` (collapses to 0 when the avatar element is absent → body left-shift), **pinned to `28px`** (`.message` → `28px 1fr`, `.message[data-own]` → `1fr 28px`; 28px = list-avatar width) so the empty continuation cell reserves its column. Scope-clean (no `MessageDescriptor` / `stream/grouping.ts` / `message-stream.svelte`; `grouped` stays the stream-computed prop); `vite build` clean (161 modules).

**CDP verification (sampler 9422, both accents — all legs green).**
- **Registry:** `ids().length` **296→286 (−10)** — exactly the **10 grouped rows** across the DD·composite panel (7 in `stream-scroll` at seed + `text-grouped` + `text-grouped-edited` + 1 in `stream-basic`), each losing its one `__avatar`; `count===unique` (286/286), **0 orphans both directions**.
- **Grouped suppression:** 10 grouped rows, **0 leaking avatar or name**. `text-grouped` / `text-grouped-edited` cells = only `__body` + root (no `entity-avatar#…__avatar`, no `label#…__name`). The head `text-other` keeps both.
- **Heads keep both:** 0 text-kind heads missing an avatar; the 3 avatar-less heads (`system-notice`, `system-long`, `stream-basic__m-sb-4`) are all `kind=system` (authorless, by design — not a regression).
- **No left-shift:** head `seed-9` and its grouped continuation `seed-8` share an identical grid `28px 311.2px` (1fr resolves to 311.2px in the sampler cell) and body-left **402=402** — the pinned 28px track keeps the empty cell's width. The `seed-4` outlier at body-left 366 is `isOwn:true` (own-side layout, grid `311.2px 28px`), unrelated to the fix.
- **grouped + content-state:** `text-grouped-edited` suppresses avatar+name (av0/nm0/bd1). grouped+deleted has no sampler fixture but rides the same `author && !grouped` positional guard (deleted independently replaces the body).
- **Both accents:** node `#3a7ab0` ↔ client `#c28840`, suppression identical, registry 286 both; grouping is structural / accent-independent.
- **Eye-check GREEN** (Joe-confirmed against the OS screenshot): Alice Ng appears once as the group head with the “AN” avatar, then Seed 8/9/10 render as bare continuation bodies (no avatar, no name, left-aligned under the head). Previously that same avatar repeated on every row — the reason grouping read as invisible.

**Doc-wording fix (Clair Rule-6 flag).** §10 / D-106 illustrated the reserved grid as `28px 288px` / `288px 28px`, but the real code was `auto 1fr` / `1fr auto` — 288px was never literal (illustrative). Clair pinned the avatar track to the real `28px` and kept the content track `1fr` (hardcoding 288px would break responsive width). Intent — reserve the avatar column — is met exactly; the wording in **D-106**, **phase0 §10**, and the components-doc registry note is corrected to `28px 1fr` in-place.

**Records.** `docs/ROADMAP.md` **v4.55→v4.56** (M-RP5.7 🟡 PENDING → ✅ DONE, full CDP results; next-active M-RP6.1). `ui/docs/xgen-ui-components.md` **v0.59→v0.60** (message row grouped-drops-avatar+name; registry note 296→286 with cause; M-RP5.7 build note). `DECISIONS.md` D-106 grid-wording fix. `docs/xgen-dd-message-family-phase0.md` §10 grid-wording fix. CLAUDE.md PLAY (head J-486→J-487; M-RP5.6 B block next-active repointed to M-RP6.1). `tasks/M_RP5_7_GROUPED_AVATAR_SUPPRESSION.md` → COMPLETED (all 7 DoD boxes checked, real-output annotated). This entry. Feat `fd0af23` = commit 1 (Clair, pushed); this doc-bridge = commit 2. Not pushed — Joe pushes.

**Next-active.** **M-RP6.1 client UI panel arc** — region-shell scaffold + selection bus + R3 Self/connection + R8 inspector (self = first inspectable; closes the read half of gate finding F-1). `temperature-indicator` (M-RP6.5 / M-RP5.4) stays ⏸️ POSTPONED until the main window is functional.


---

## Entry J-486 — M-RP5.7 grouped-avatar suppression: Phase-0 LOCKED

**Design/records-only, no code (Rule 1/5).** Ran the D-071 Phase-0 gate for **M-RP5.7**, opened after Joe flagged the M-RP5.6 B screenshot: the `stream-scroll` render showed the identical avatar on every row with nothing dropped, so grouping read as invisible. Root: M-RP5.5 B made `grouped` suppress the **name header** but **keep the avatar** — a same-author run then repeats the avatar, the exact who-is-speaking noise grouping exists to remove. Joe locked the correction “by recomms.” Registry unchanged this entry (**296**); the build will DROP it.

**Locked (Joe, “by recomms”) — all six.**
1. **Symmetric suppression.** A `grouped` row (continuation, `grouped === true`, NOT the group head) renders **neither name NOR avatar**. The group head (`!grouped`) keeps avatar + name. `grouped` stays the stream-computed prop — no stream change.
2. **Element absent, not `visibility:hidden`.** The grouped row does not render the `entity-avatar` child at all → a grouped cell registers **neither `__avatar` nor `__name`**.
3. **Column reserved.** Keep the message grid tracks (`28px 288px` / `288px 28px`); only the avatar **cell content** is empty, so continuation bodies stay aligned under the head (no left-shift). Symmetric for `isOwn`.
4. **Independent of content state.** `grouped` is positional — suppresses avatar + name **regardless of `deleted` / `edited`**.
5. **Formalize as a decision.** The B “keep the avatar” was a build note, not a D; correcting a project-level UI convention warrants a recorded decision → **D-106**.
6. **Registry drops — measure, don’t assume.** Every grouped cell loses `__avatar`; the ripple hits ALL grouped cells (the B `grouped`/`grouped-edited` sampler cells too, not only `stream-scroll`). CDP-measure the real new total at build + record the cause (Rule 5); do NOT retro-rewrite closed milestones’ counts.

**Decision (D-series).** **D-106** — grouped rows drop BOTH name and avatar; the group head carries both; the avatar column stays reserved. (Unlike the M-RP5.6 B implementation choices §9.10, which were code-level and needed no D, this corrects a UI convention.)

**Sub-milestone (locked).** One small Clair feat — `message.svelte` grouped branch drops the `entity-avatar` child (element absent) in addition to the name; + `skin.css` empty-gutter handling. **No** `MessageDescriptor` / `stream/grouping.ts` / `message-stream.svelte` change. Then Chat CDP-verify (grouped cells have no `__avatar`/`__name` in DOM; head cells keep both; body alignment head-vs-continuation via `rect.left`; grouped+deleted / grouped+edited still suppress; real new registry total + cause; both accents; eye-check: one head avatar + bare continuations) → D-074 two-commit close. Closes M-RP5.7 → grouping visually correct → next-active M-RP6.1. Tighter inter-continuation vertical spacing = deferred skin follow-up (D-065).

**Records.** `DECISIONS.md` **+D-106**. `docs/xgen-dd-message-family-phase0.md` **v1.2→v1.3** (+§10 M-RP5.7). `docs/ROADMAP.md` **v4.54→v4.55** (M-RP5.7 PENDING/next-active; M-RP5.6 block’s next-active repointed to M-RP5.7→M-RP6.1). CLAUDE.md PLAY (head J-485→J-486, next-active M-RP5.7). Runbook `tasks/M_RP5_7_GROUPED_AVATAR_SUPPRESSION.md` drafted (ACTIVE, for Clair). This entry. No code, registry unchanged (**296**). GitHub board: M-RP5.7 card PENDING (design-locked; build not started). Not pushed — Joe pushes.

**Next-active.** **M-RP5.7 build** — grouped-avatar suppression on `message.svelte` (Clair feat → Chat CDP-verify → D-074 close), then M-RP6.1.


---

## Entry J-485 — M-RP5.6 B DONE: `message-stream` scroll machine built + CDP-verified (M-RP5.6 CLOSED)

**Doc-bridge (D-074 second commit).** Clair's feat `5cfd718` (the scroll machine) is already pushed = commit 1; this entry + the paired canonical records = commit 2. The `message-stream` **scroll machine** (M-RP5.6 B, Phase-0-locked at J-484) is built and CDP-verified against the live sampler (9422, both accents). **M-RP5.6 CLOSED** — the message dd sub-family (`message` A/B/C + `message-stream` A/B) is complete. Registry **262→296**.

**Built (Clair, `5cfd718`).** On the existing A viewport (`overflow-y:auto`, `max-height:340px`): `atBottom` is now live — a `scroll` listener, **rAF-throttled**, one const `BOTTOM_THRESHOLD_PX = 80` governs both `atBottom` and pill visibility (getter G's `atBottom` drops the A `true` stub). Stick-to-bottom on append@bottom; no-yank when scrolled up. Jump-to-latest pill = **inline chrome** (`<button class="jump-to-latest">`, `{#if !atBottom}`, unregistered — the `day-divider` precedent; state CDP-observable via getter `atBottom`). Preserve-position-on-prepend: `$effect.pre` captures `scrollHeight`, `tick()` re-applies the delta; prepend-detect = count grew AND prev-first-id no longer at index 0. Initial scroll to bottom on mount. No `MessageDescriptor` / `grouping.ts` / `message.svelte` change — grouping/dividers recompute for free via the existing `$derived computeRows`. `vite build` clean (161 modules); scope-clean (`message-stream.svelte` + `app_sampler.svelte` + `skin.css`).

**As-built structural deviation (Clair, sound — flagged and verified against the contract before verifying with it).** §9.10 named the pill as `position:absolute` inside the stream, but an absolute pill INSIDE the `overflow-y:auto` root scrolls away with the content. Clair wrapped the viewport in a non-scrolling `.message-stream-shell` (`position:relative`) and made the pill a **sibling** of the scroll element — the `entity-panel` rooting precedent (outermost DOM = a chrome wrapper). The envelope + `role="log"` still ride the inner `.message-stream` scroll element, so the **registry root is unchanged** and getter G is untouched. Chat confirmed this preserves the contract, then verified against it.

**CDP verification (sampler 9422, live drive; real output quoted, Rule 2).** Joe launched the dev sampler; Chat ran short probes.
- **Registry integrity:** `{count:296, unique:296, orphans:0}` both directions; `count===unique`. The `stream-scroll` seed subtree = **+34** over A's 262 (10 `__avatar` + 10 `__body` + 3 `__name` [7 grouped rows suppress `__name`] + composed roots). *(A transient `306` read appeared for one probe immediately after a reset click — a mid-flush read as the mutated 13-msg subtree tore down to the 10-msg seed; re-measured after settle = `296`, groupedCount 7, names 3. Rule 5: the settled number is authoritative.)*
- **Getter G shape:** `{count, selected, hasEmpty, groupedCount, dividerCount, atBottom, backgroundMountCount, backgroundLive}` — the eight-field contract, `atBottom` now live.
- **Mount-to-bottom:** `scrollTop=398` = max (`scrollHeight 736 − clientHeight 338`).
- **`atBottom` live:** `true` @bottom → `false` scrolled-up (gap 398, pill "↓ Latest" present) → `true` @jump. Pill present ⇔ `!atBottom`.
- **Append@bottom sticks:** count `11→12`, `scrollTop 444→490` (followed the growth).
- **Append scrolled-up = NO yank:** count `10→11`, `scrollTop` held at `0`, `scrollHeight +46`, pill persists.
- **Jump pill:** `scrollTop→444`=max, `atBottom:true`, pill hides.
- **Prepend invariance:** `ΔscrollTop = ΔscrollHeight = 68` (isPrepend branch fired: `200+68=268`) — the anchor holds exactly.
- **Grouping recompute free:** `groupedCount 7→9` across mutations, zero extra code.
- **Both accents:** pill accent-neutral (bg `rgb(42,47,56)` / color `rgb(200,196,188)` identical client↔node) while `--accent2` swaps gold `#c28840` ↔ blue `#3a7ab0`.

**Eye-check screenshot — OS-captured (harness screenshot mode bugged).** The harness `-Mode screenshot` returned exit-1 on this capture (foregrounded, and with `-Seconds 30`). Root-caused: NOT window occlusion and NOT the receive timeout — the failure throws *before* the mode's own error prints, and screenshot is the only mode with a large payload (the base64 PNG), while `eval`/`state` (small payloads) work through the same receive+`ConvertFrom-Json` path. **Windows PowerShell 5.1 `ConvertFrom-Json` chokes on the large screenshot payload** (with `$ErrorActionPreference='Stop'` → throws to `finally` → exit-1). Intermittent across captures because payload size tracks window content — the added `stream-scroll` rows pushed this capture over the limit where the older ~51 KB `cdp-shot-sampler.png` squeaked under. Concrete next-touch for the standing J-483 "harden screenshot mode" flag: parse `result.data` via substring instead of full-object `ConvertFrom-Json`. Joe captured an OS-level screenshot instead (`screenshot_2026-07-09_173150.jpg`), eye-checked GREEN: DD Composites tab, `stream-scroll` scrolled to bottom (Seed 8/9/10 + Appended #1/#2), no pill (at bottom), avatars on every row (grouping suppresses the name header, not the avatar), `stream-bg-unknown` empty box (W-13 unknown-`widgetId` drop), client accent.

**Records (this commit).** `ui/docs/xgen-ui-components.md` **v0.58→v0.59** (message-stream row + scroll machine; registry note 262→**296**; M-RP5.6 C build note). `docs/ROADMAP.md` **v4.53→v4.54** (M-RP5.6 B ✅ DONE, M-RP5.6 CLOSED, next-active M-RP6.1). CLAUDE.md PLAY (head J-484→J-485, M-RP5.6 B ✅ DONE, next-active M-RP6.1). `tasks/M_RP5_6_B_MESSAGE_STREAM_SCROLL.md` → COMPLETED. This entry. No `DECISIONS.md` change (§9.3/§9.10 already carry the machine; the `.message-stream-shell` wrapper is an implementation realisation, not a project-level decision). GitHub board: M-RP5.6 card → DONE. Not pushed — Joe pushes.

**Next-active.** **M-RP6.1 client UI panel arc** — region-shell scaffold + selection bus + R3 Self/connection + R8 inspector (the R5 message-stream system-widget wrap + live node↔client wiring land in the M-RP6.x arc). `temperature-indicator` (M-RP6.5 / M-RP5.4) stays ⏸️ POSTPONED until the main window is functional.


---

## Entry J-484 — M-RP5.6 B Phase-0 LOCKED: `message-stream` scroll machine (implementation refinement)

**Design/records-only, no code (Rule 1/5: nothing built, nothing to verify — stated plainly).** Ran the D-071 Phase-0 gate for M-RP5.6 B (the scroll machine), opened as next-active at J-483. §9.3 of the family Phase-0 already locked the machine at concept level ("full A"); this gate refined the **implementation** — Joe-locked "by recomms" on all four questions + two surfaced sub-points. No `MessageDescriptor` / `grouping.ts` / `message.svelte` change: B is scroll behaviour on the A viewport (`overflow-y:auto`, `max-height:340px`, getter G's `atBottom` currently a `true` stub). Registry unchanged (**262**).

**Locked (Joe, "by recomms").**
- **Q1 — thresholds:** ONE build-time const `BOTTOM_THRESHOLD_PX = 80` governs both `atBottom` and pill visibility (no hysteresis / second band). `atBottom` (`$state`) `= scrollHeight − scrollTop − clientHeight ≤ 80`, recomputed in a `scroll` listener, **rAF-throttled** for a clean CDP read. Getter G's `atBottom` becomes live (drops the A stub).
- **Q2 — jump-pill = inline chrome:** a plain `<button class="jump-to-latest">` in the stream markup (`{#if !atBottom}`), `position:absolute` bottom-right, appearance in `skin.css`. **NOT a component, NOT registered** (the `day-divider` precedent) — its state is already CDP-observable via getter `atBottom`. Fallback if a registry-visible node is later wanted: compose `core` `button` → `<id>__jump`.
- **Q3 — prepend = scrollHeight-delta + first-id heuristic:** prepend-detection = `messages.length` grew AND the previous first-descriptor id is no longer at index 0 (self-contained; distinct from append = count grew, first-id stable). Anchor: capture `prevScrollHeight` in `$effect.pre`, after `tick()` apply `scrollTop += scrollHeight − prevScrollHeight` — the "prepend `scrollTop` invariance" the DoD names. `atBottom` unaffected.
- **Q4 — sampler drive:** ONE new `stream-scroll` live fixture (mutable `$state` array, ~10 seeded `text` messages overflowing 340px) + **append / prepend / reset** buttons in sampler chrome (the existing `streamBgLive`-button pattern). Append pushes `now`; prepend unshifts ~10 min earlier (re-exercises grouping/dividers — free via `$derived computeRows`). Four static fixtures (basic/days/empty/bg) untouched. CDP drives via clicking the real buttons + `el.scrollTop` sets.
- **Sub-point 1 — initial scroll on mount:** on mount `scrollTop = scrollHeight` (chat opens at newest); matches the `atBottom:true` init.
- **Sub-point 2 — grouping recompute is free:** append/prepend mutate `messages` → `computeRows` re-derives grouping + dividers with zero extra code (stated so the runbook doesn't re-solve it).

**No new decision (D-series).** §9.3 already carries the machine; these are implementation choices, not a project-level decision — DECISIONS.md untouched.

**Sub-milestone (locked).** **B** — single Clair feat on the existing `message-stream` root (scroll listener + rAF-throttle + live `atBottom` + inline pill + `$effect.pre`/`tick()` prepend-anchor + mount-to-bottom) + sampler `stream-scroll` live fixture + `skin.css` pill rule; then Chat CDP-verify (`atBottom` true→false→true transitions / append-stick vs append-no-yank / pill visibility / prepend `scrollTop` invariance) → D-074 two-commit close. Closes M-RP5.6 (A+B) → message dd sub-family complete → next-active M-RP6.1 client UI panel arc.

**Records.** `docs/xgen-dd-message-family-phase0.md` **v1.1→v1.2** (+§9.10 M-RP5.6 B implementation refinement). `docs/ROADMAP.md` **v4.52→v4.53** (M-RP5.6 B Phase-0 LOCKED). CLAUDE.md PLAY (head J-483→J-484, next-active flipped to the B build). Runbook `tasks/M_RP5_6_B_MESSAGE_STREAM_SCROLL.md` drafted (ACTIVE, for Clair). This entry. No code, registry unchanged (**262**). GitHub board: M-RP5.6 card stays PENDING (design-locked; build not started). Not pushed — Joe pushes.

**Next-active.** **M-RP5.6 B build** — the scroll machine on the `stream-scroll` live fixture (Clair feat → Chat CDP-verify → D-074 close).

---

## Entry J-483 — M-RP-CDP1: CDP harness RESTORED across all three Tauri apps (dev-only `--config` overlay) + M-RP5.6 A CDP legs CLOSED (registry 219→262)

**One atomic commit (D-074) — infra (overlays + launch scripts) travels with the records it produces.** No Clair feat leg (no Rust/Svelte): the fix is Tauri config + PowerShell + docs, Chat-Claude territory. Everything below is verified with real CDP output on the reliable PowerShell path (Rule 2); the M-RP5.6 A registry count is now the live `ids().length`, no longer withheld (Rule 5).

**Root cause — CORRECTED (the D-104 diagnosis was wrong).** D-104 (and J-482) blamed the Chromium-136 `--user-data-dir` guard: "the port is ignored unless a non-default `--user-data-dir` accompanies it." That is **refuted**. Two facts from the real `msedgewebview2.exe` child command line (captured in a normal shell — `Get-CimInstance` on webview2 hangs the MCP, so Joe ran it): (1) the sampler child **already** carries a non-default `--user-data-dir=…\com.alchemydump.xgensampler\EBWebView` — Tauri **forces** a non-default data dir on Windows by default (`tauri-2.11.1 manager/webview.rs` L534–545), so the guard's precondition was satisfied all along; (2) `--remote-debugging-port` was **absent from every webview2 process**. The port never reached the browser command line. Cause: **wry overrides the `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` env var** with its own programmatic `AdditionalBrowserArguments`, dropping the env-supplied port. (The env-var route happened to work pre-150 by timing/precedent; it is not a contract.) So the guard was a red herring; the real failure was env-var clobbering, and the earlier spikes into `dataDirectory` were a fair-but-null test (Tauri only honours a **relative** config `dataDirectory`, resolved under `%LOCALAPPDATA%\<label>\`; an absolute one is logged-and-ignored — `webview/mod.rs` L392–424).

**Fix (Option A, Joe-locked; verified).** Route the port through Tauri config **`additionalBrowserArgs`** (wry's programmatic channel — where it lands and WebView2 150 honours it), delivered as a **dev-only overlay** so it never ships in release: a per-app `cdp.dev.conf.json` (full window object + `--remote-debugging-port=<port>`) merged via `cargo tauri dev --config cdp.dev.conf.json`; the base `tauri.conf.json` stays **port-free**. `--config` replaces the `app.windows` array (RFC-7396), so the full-window overlay preserves geometry — confirmed (sampler window 960×820 intact, single instance).

**Verified — all three apps (real output).**
- **sampler 9422:** `/json/version` Edg/150.0.4078.48; page `http://localhost:5175/`; `cdp-debug.ps1 -App sampler -Mode eval` → `XGen Sampler | debug=object`.
- **client 9222:** page `http://localhost:5173/`; harness eval → `XGen Client | debug=object`.
- **node 9322:** page `http://localhost:5174/` (window `visible:false`, webview+port still up); harness eval → `XGen Node | debug=object`.
The block is universal (all three are Tauri 2.11.1 / wry 0.55.1); the one fix restores the whole M-RP6.x CDP loop.

**Implemented (B — propagation).** `xgen-sampler/`, `xgen-client/`, `xgen-node/` each gain `cdp.dev.conf.json`; `run-sampler.ps1 -Debug` / `run-client.ps1 -Debug` / `run-node.ps1 -Debug` swap the dead env var for `--config cdp.dev.conf.json`; `cdp-debug.ps1` header note updated. **Flagged (Rule 6, not silently fixed):** `cdp-debug.ps1 -Launch` (built-exe path) still sets the env var — **dead under ≥136** and un-fixable by `--config` (a built exe takes no config overlay); the supported CDP path is attaching to a `run-*.ps1 -Debug` **dev session**. Baking a port into a release config was rejected (would expose CDP in release). See D-104 resolution note + `CDP_DEBUG_HARNESS.md`.

**M-RP5.6 A CDP legs — CLOSED (all §9 checks green against live 9422).**
1. **Registry integrity:** `ids().length` **262**, `count===unique` (262/262), **262 DOM `[data-debug-id]`**, **0 orphans both directions**.
2. **Grouping** (`stream-basic`): getter `groupedCount:1`; DOM grouped row drops `.msg-header` (the 1 `system` row counted separately: 5 rows, 2 header-less = 1 grouped + 1 system).
3. **Dividers** (`stream-days`): getter `dividerCount:4`; the four label bands render with correct live dates — `Today (Jul 9, 2026)` / `Yesterday (Jul 8, 2026)` / `Monday (Jul 6, 2026)` (weekday band) / `Jul 1, 2026` (≥7-day date-only); `groupedCount:0` (dividers break grouping).
4. **Empty:** `stream-empty` `hasEmpty:true` + rendered `.paragraph` "No messages yet".
5. **Background:** `stream-bg` `backgroundMountCount:1` (known) vs `stream-bg-unknown` `0` (W-13 unknown-drop; background-set → no fallback paragraph, spec-consistent); `.message-stream-bg` `position:absolute`, top 0, z-index 0.
6. **Select:** click row → getter `selected:"sb-3"` + exactly one `[data-selected]="true"` mirror.
7. **Both accents:** shell swap gold `#c28840` ↔ blue `#3a7ab0` (accent-neutral component); divider `--t3` identical `rgb(138,136,128)` client↔node.
8. `vite build` clean (**161 modules**); screenshot `temp/cdp-shot-sampler.png` (51 KB).

**Registry: 219 → 262 (+43).** The 5 `message-stream` fixture subtrees: `stream-basic` **18**, `stream-days` **21**, `stream-empty` **2**, `stream-bg` **1**, `stream-bg-unknown` **1** (`_sum:43`) — the stream roots + their composed `message`/`entity-avatar`/`label`/`paragraph` instances (dividers are plain `role="separator"` divs, unregistered). `count===unique`, 0 orphans confirms the retroactive close.

**Records.** `DECISIONS.md` **+D-105** (dev-only `--config` CDP overlay mechanism) + **D-104** resolution note (root-cause correction + exception retired). `docs/ROADMAP.md` **v4.51→v4.52** (M-RP-CDP1 ✅, M-RP5.6 A ✅). `ui/docs/xgen-ui-components.md` **v0.57→0.58** (count 219→262, ⚠️ PENDING note dropped, message-stream CDP-verified build note). `CLAUDE.md` PLAY (count 262, head J-483, next-active M-RP5.6 B). `tasks/M_RP5_6_A_MESSAGE_STREAM_SHELL.md` → **COMPLETED**. `tasks/CDP_DEBUG_HARNESS.md` (overlay mechanism + `-Launch` flag). This entry. One commit, Joe pushes.

**Next-active.** **M-RP5.6 B** — the scroll machine (stick-to-bottom + jump-to-latest pill + prepend `scrollTop` invariance) on sampler fixtures; opens with a D-071 Phase-0. Its DoD is CDP-heavy (`atBottom` transitions, prepend invariance) — now unblocked.

---

## Entry J-482 — M-RP5.6 A: `message-stream` shell BUILT + landed with CDP verification DEFERRED (WebView2 150 harness block; D-104 / M-RP-CDP1 opened)

**Two-commit close (D-074), CDP legs DEFERRED (D-104).** Clair's feat (`message-stream.svelte` + `stream/grouping.ts` + `skin.css` + sampler) + this doc-bridge. **Honest status (Rule 1/2/5):** the code is built and `vite build`-clean, the pure grouping logic is unit-verified 20/20, but the **CDP-only DoD legs are DEFERRED** — the sampler remote-debug harness (9422) is environmentally blocked. Registry count is **NOT** asserted (no live `ids().length`, Rule 5) — it stays at the last-verified **219** with a pending-CDP note.

**Built (`message-stream` dd-composite — step A shell, faithful to phase0 v1.1 §9).**
- **Root** `<div class="message-stream" role="log" use:envelope>` = the scroll viewport (`overflow-y:auto`); background layer + row list are siblings (bg `position:absolute; inset:0; z-index:0` behind, rows `z-index:1`). Structure in the scoped `<style>`; appearance in `skin.css`.
- **`stream/grouping.ts`** (pure, colocated, unit-testable — the `transform.ts`/`clamp.ts` precedent): `GROUP_WINDOW_MS = 5*60*1000` + `computeRows(messages, now)` → an interleaved `StreamRow[]` (`message` carrying its stream-computed `grouped` / `divider` carrying its label) + `formatDayDivider(ts, now)` (fixed en-US `Intl`, DOM-free). Grouping: `text` + same `author.id` + within window + no divider between; breaks on author/`system`/day/first; `deleted` keeps author (doesn't break). Dividers: inserted **between** consecutive messages on local-day change (`toDateString()`), **no leading divider** (spec compares two timestamps — the oldest day heads the stream unlabelled; the four label bands show on day-changes down the stream).
- **Props** = `messages[]` (ordered, not re-sorted) · `background?: WidgetMount[]` + `backgroundLive?` (default true, binding M-RP6.x) · `widgets` registry (widgetId→Component; background + child `message.details`, unknown-id dropped W-13) · `selected?` ($bindable) + `onSelect?` (reserved). Child ids: `<id>__m-<msgid>` prefix chain (clean parent-prefix nesting).
- **Empty (§9.4):** default centered `paragraph` ("No messages yet") ONLY when `count===0` AND no `background` declared; a declared background "shows through" instead.
- **Getter G (§9.6):** `{count, selected, hasEmpty, groupedCount, dividerCount, atBottom, backgroundMountCount, backgroundLive}` — `atBottom` initialised `true` in A (B drives it live).
- **Sampler:** import + 5 DD·composite fixtures (basic / days / empty / bg / bg-unknown) + cells; `skin.css` gains `.message-stream` / `.day-divider` / `.message-stream-row[data-selected]` / `.message-stream-empty`.
- **Scope:** this is **A only** — the root is the scroll viewport but **no scroll behaviour is built** (stick-to-bottom / jump-pill / prepend-preserve = **B**).

**Verified (real output).**
- ✅ `vite build` clean — **161 modules**, no `message-stream` warnings (the two pre-existing meter/entity-avatar notes only).
- ✅ **20/20 pure unit test** on the real `stream/grouping.ts` (ad-hoc `node` type-strip run — reproducible; not a committed suite, no JS runner ships): proves `groupedCount` (1 / 0), `dividerCount` (0 / 4), all four divider bands with real values `["Jun 30, 2026", "Sunday (Jul 5, 2026)", "Yesterday (Jul 7, 2026)", "Today (Jul 8, 2026)"]`, divider-breaks-grouping, system-breaks-a-run, no leading divider — i.e. the exact facts the CDP getter checks would target.

**DEFERRED (D-104) — the CDP legs, blocked by the harness, not by the code.** registry `count===unique` · 0 orphans both directions · getter-G live readout · both-accents. These close retroactively once M-RP-CDP1 restores the harness.

**Why deferred — the harness block (full record).** The sampler CDP harness (9422) worked unchanged J-405→J-480; it is now blocked because the machine's WebView2 Evergreen runtime auto-updated to **150.0.4078.48** (Chromium ≥136), which enforces the Chromium-136 remote-debugging hardening: `--remote-debugging-port` is ignored unless a **non-default `--user-data-dir`** accompanies it on the browser command line. Diagnosed on the reliable PowerShell path with real output — clean verified-zero launch + visible/foregrounded window + ~90s poll → 9422 never opens; runtime pv confirmed `150.0.4078.48`; launcher env var confirmed correct. All levers exhausted: port-only; port + `--user-data-dir` in `AdditionalBrowserArguments` (WebView2 disallows it there); port + `WEBVIEW2_USER_DATA_FOLDER` (wry overrides the data folder). Official fixed-version runtimes exist only for the latest two majors (both ≥136); the only pre-136 source is an untrusted third-party archive — **declined** (supply-chain risk on a dev machine, Joe's call). Full mechanism + fix hierarchy in D-104. (Earlier speculation that "wry overrides the env var per WebView2 precedence" is retracted — the real cause is the Chromium-136 `--user-data-dir` guard.)

**M-RP-CDP1 opened (harness restore, own milestone).** Preferred: an in-repo host change giving the sampler webview an explicit non-default data dir (so port + non-default `--user-data-dir` satisfy the guard); fallback: `cdp-debug.ps1` → `--remote-debugging-pipe`. Universal — the same block will hit client 9222 + node 9322, so the restore fixes all three Tauri apps. No untrusted downloads.

**Records.** `DECISIONS.md` **+D-104** (temporary CDP-deferred close). `docs/ROADMAP.md` **v4.50→v4.51** (M-RP5.6 A = code-landed/CDP-pending; **M-RP-CDP1** added, PENDING). `ui/docs/xgen-ui-components.md` **v0.56→v0.57** (message-stream row + count-pending note; still 219). CLAUDE.md PLAY. `tasks/M_RP5_6_A_MESSAGE_STREAM_SHELL.md` landing note (code COMPLETE, CDP DoD DEFERRED). This entry. Feat + docs = two commits, Joe pushes both.

**Next-active.** **M-RP-CDP1** (harness restore) — then close M-RP5.6 A's deferred CDP legs → **M-RP5.6 B** (scroll machine). If Joe prefers, M-RP5.6 B is buildable now on fixtures too, but its DoD is CDP-heavy (`atBottom` transitions, prepend `scrollTop` invariance), so the harness should come first.

---

## Entry J-481 — M-RP5.6 Phase-0 addendum LOCKED: `message-stream` (grouping / day-dividers / scroll / background / select)

**Design/records-only, no code (Rule 1/5: nothing built, nothing to verify — stated plainly).** Ran the D-071 Phase-0 gate for the `message-stream` (opened as next-active at J-480). Discussion → Joe-locked → the family Phase-0 doc extended **v1.0→v1.1** (§9 addendum). Registry unchanged (**219**).

**`message-stream`** = `core` dd-composite, the `entity-panel` analogue. Root `<div class="message-stream" role="log" use:envelope>` = also the **scroll viewport** (`overflow-y:auto`). Children = `message`s + interleaved day-divider rows, chronological. Sets each child's `grouped` prop (Phase-0 §5). `role="log"`, click-select (not roving).

**Locked (Joe).**
- **Grouping (Q1):** a `text` message renders `grouped` iff the previous *rendered* row is `text`, same `author.id`, within a **5-min** window, no day-divider between. Breaks on different author / any `system` / day boundary / first row. `deleted` tombstone keeps `author.id` → doesn't break a run. `system` never groups. Window = build-time const, Joe-tunable.
- **Day-dividers (Q2):** separator row (`<div class="day-divider" role="separator">`) inserted on local-day change (boundary = local midnight); breaks grouping. Label **always carries the date**: `Today (Jul 8, 2026)` / `Yesterday (Jul 7, 2026)` / `Saturday (Jul 6, 2026)` (2–6 days, weekday+date) / `Jul 1, 2026` (≥7 days, date only). Build-time formatter, Joe-tunable.
- **Scroll (Q3) — full A:** stick-to-bottom (at/near bottom ≤80px → append auto-scrolls; scrolled-up → append shows a **jump-to-latest pill**, no yank) + **preserve-position-on-prepend** (older-load adjusts `scrollTop` by inserted height). Exercised on fixtures via sampler append/prepend controls (no live channel yet, J-476); CDP reads `atBottom` transitions + prepend `scrollTop` invariance.
- **Background/empty (Q6):** `background?: WidgetMount[]` = a **persistent fixed layer** behind the log (`position:absolute; inset:0`, below messages; does NOT scroll — wallpaper style); reuses the `WidgetMount[]`/W-13 socket (static object OR lifecycled/reactive widget). `backgroundLive?: boolean` (default true) = the settings switch (`false` → static/frozen). Fallback: `background` unset AND `count===0` → default composed `paragraph` ("No messages yet"), never bare.
- **Select (Q4):** click → `selected` ($bindable id) + `[data-selected]` row mirror + reserved `onSelect?`. Feeds the future R8 inspector / `entity-context-menu` selection bus. No roving tabindex (§6). Wiring deferred to M-RP6.x; hook reserved now.
- **Getter G:** `{count, selected, hasEmpty, groupedCount, dividerCount, atBottom, backgroundMountCount, backgroundLive}` — mirrors `entity-panel` + stream/background observables (grouping + scroll + background CDP-readable).
- **No `MessageDescriptor` change** — grouping stays a stream-computed prop passed down; stream-level props (`messages[]`, `background?`, `backgroundLive?`, `selected?`, `onSelect?`) are the new surface.

**Two deferrals (Rule 6, recorded).** (1) The `backgroundLive` **settings binding** ($common store / layout field) is M-RP6.x; M-RP5.6 exposes the prop + drives it from a sampler control (same posture as the R5-wrap deferral). (2) The R5 system-widget wrap stays deferred to the M-RP6.x region shell (Phase-0 §1).

**Sub-milestones (locked).** **A** — shell: root `<div role="log">` + ordered N `message`s + grouping computation (sets `grouped`) + day-dividers + empty fallback + background layer render. Static fixtures. Sampler DD·composite cells + CDP. **B** — scroll machine (stick + jump-pill + prepend-preserve) via sampler append/prepend controls. CDP. → D-074 close.

**Records.** `docs/xgen-dd-message-family-phase0.md` **v1.0→v1.1** (+§9 M-RP5.6 addendum). `docs/ROADMAP.md` **v4.49→v4.50** (M-RP5.6 Phase-0 addendum LOCKED + doc pointer). CLAUDE.md PLAY next-active flipped (M-RP5.6 Phase-0 locked → A build). Runbook `tasks/M_RP5_6_A_MESSAGE_STREAM_SHELL.md` drafted (ACTIVE, for Clair). This entry. No code, registry unchanged (**219**). GitHub board: M-RP5.6 card stays PENDING (design-locked; build not started). Not pushed — Joe pushes.

**Next-active.** **M-RP5.6 A** — `message-stream` shell (log + ordered N + grouping-compute + day-dividers + empty + background) on sampler fixtures; sampler DD·composite + CDP-verify (Clair feat → Chat doc-bridge).

---

## Entry J-480 — M-RP5.5 C: `message` `system` kind + `isOwn` verify — message family v1 CLOSED (registry 215→219)

**Two-commit close (D-074).** Clair's feat `09e9cbe` (3 files, pushed) + this doc-bridge. Third + final step of the `message` dd sub-family — the second v1 kind, on sampler fixtures. **Message family v1 is CLOSED.**

**Built (`system` kind).**
- **Top-level `kind` split** in `message.svelte`: `{#if kind === 'system'}` → an authorless centered notice; `{:else}` → the A/B `text` sub-tree, untouched. The two paths are visibly separate so `system` reads NONE of the text-only fields.
- **`system` arm** = its own `<article data-kind="system">` (no `data-own` — a notice has no side) rendering ONE centered `paragraph` (`__body`). No avatar, no `.msg-header`/name, no `details`, no `edited` marker, no tombstone.
- **Getter (Option A — normalized on `system`):** forces the text-only fields off — `{kind:'system', author:null, hasBody, detailsCount:0, isOwn:false, grouped:false, edited:false, deleted:false}`. On `text` the getter is verbatim (unchanged from A/B). Deliberate: the getter tracks RENDER truth, not descriptor truth (the `deleted → detailsCount:0` precedent, J-479); a stray field on a system descriptor never renders, so it never reports. One-line comment marks it.
- **Skin** `.message[data-kind="system"]` collapses the avatar-column grid to `1fr` + `text-align:center`; the notice paragraph is muted `--t3`/`--fs-1` (no new token). Wrapping stays symmetric on the full centered track.

**CDP (9422, both accents) — real output.** `ids()` **215→219** (+4 = 2 system cells × `message` + `paragraph#…__body`; NO `__avatar`/`__name`), `count===219`, `unique===219`, **0 orphans both directions**. System getters both cells exactly the Option-A shape. `system-notice`: `data-kind=system`, single `324px` track, `text-align:center`, no `.msg-header`, no `.entity-avatar`, h=26 (one line). `system-long`: same, h=62 (wraps, centered symmetric). Text mirror re-asserted: `text-other` grid `28px 288px` (own=false) ↔ `text-own` `288px 28px` (own=true, `data-own`). Accent-neutral: system line `rgb(138,136,128)`=`--t3` **identical** client↔node; `--accent2` `#c28840` ↔ `#3a7ab0` (swap live). Screenshot `temp/m-rp5-5-c-system.png`. `vite build` clean (158 modules; the two meter/entity-avatar warnings pre-existing).

**Message family v1 CLOSED.** 2 kinds (`text` / `system`) + `grouped`/`edited`/`deleted` states + the `details` widget socket. `bodyExtras` stays reserved-unfed (D-065); the `message-stream` (M-RP5.6) is the next dd step (it sets B's `grouped` prop).

**Records.** `ui/docs/xgen-ui-components.md` v0.55→v0.56 (message row kinds `text`/`system` + Build note (M-RP5.5 C) + count 215→219). `docs/ROADMAP.md` v4.48→v4.49 (M-RP5.5 ✅ DONE + C ✅ — family v1 CLOSED). CLAUDE.md PLAY. `tasks/M_RP5_5_C_MESSAGE_SYSTEM.md` → COMPLETED (+ close record). This entry. Feat `09e9cbe` already pushed; doc-bridge is a second commit — Joe pushes.

**Next-active.** **M-RP5.6** `message-stream` dd-composite (the `entity-panel` analogue): Phase-0 addendum (grouping algo, day-divider rule, scroll machine, `role="log"`, select hook) → A (section + ordered N `message`s + empty + grouping computation + day-dividers) → B (scroll) → D-074 close. R5 system-widget wrap stays deferred to M-RP6.x.

---

## Entry J-479 — M-RP5.5 B: `message` text states — grouped / edited / deleted (registry 202→215)

**Two-commit close (D-074).** Clair's feat `063aeab` (3 files, pushed) + this doc-bridge. Second build step of the message dd sub-family — the three `text` render-states, on sampler fixtures.

**Built (`text` states).**
- **`grouped`** — a **stream-computed PROP** on `message.svelte` (not a `MessageDescriptor` field; `message-stream` sets it at M-RP5.6). `{#if !grouped}` suppresses the whole `.msg-header` (name + `details`); the avatar + body stay, the reserved column keeps alignment (reader orientation).
- **`edited`** (descriptor field) — a trailing `.message-edited` `(edited)` marker after the body; suppressed under `deleted`.
- **`deleted`** (descriptor field) — tombstone: the body `paragraph` is replaced by an empty `.msg-deleted` span whose copy is a skin `content` var (`--msg-deleted`, the `--caps-hint` precedent); `details`/`edited` dropped, `detailsCount` forced 0, avatar + name kept.
- **Precedence:** `deleted` wins. Getter `+= grouped, edited, deleted`.

**CDP (9422, both accents).** `ids()` **202→215** (+13 = grouped 3 + edited 4 + deleted 3 + grouped-edited 3 — grouped cells drop `__name`, deleted cells drop `__body`), `count===unique`, 0 orphans both directions. grouped/grouped-edited → no `.msg-header` in DOM + `__name` unregistered; edited → `(edited)` present; deleted → `::before` = the `--msg-deleted` copy, no body/details/edited, `__body` unregistered, avatar+name kept. Accent-neutral (`--accent2` `#c28840` ↔ `#3a7ab0`). `vite build` clean (158 modules; the two warnings pre-existing in meter/entity-avatar).

**Records-honesty note (Rule 1).** The B kickoff called all three "existing `MessageDescriptor` fields." Phase-0 §4's descriptor field-list loosely included `grouped`, but Phase-0 §2/§5 lock it as stream-computed ("message only *accepts* it"). Clair implemented it correctly as a **prop**, not a field — `edited`/`deleted` are the descriptor fields. The code is authoritative; no reversal. (Phase-0 §4's loose mention noted here, not rewritten — §5's split was always right.)

**Records.** `ui/docs/xgen-ui-components.md` v0.54→v0.55 (message row getter + Build note (M-RP5.5 B) + count 202→215). `docs/ROADMAP.md` M-RP5.5 B ✅. CLAUDE.md PLAY. `tasks/M_RP5_5_B_MESSAGE_TEXT_STATES.md` → COMPLETED. This entry. Feat `063aeab` already pushed; doc-bridge is a second commit — Joe pushes.

**Next-active.** **M-RP5.5 C** — `system` kind (authorless centered notice) + full `isOwn` both-sides verify → closes message family v1, D-074 close.

---

## Entry J-478 — M-RP5.5 A: `message` dd-composite built — `MessageDescriptor` + `text` full (registry 186→202)

**Two-commit close (D-074).** Clair's feat `166529e` (5 files, pushed) + this doc-bridge. First build step of the message dd sub-family, on sampler fixtures.

**Built (`text` full only).**
- **`MessageDescriptor` dd-socket** (`types.ts`, beside `EntityDescriptor`; source-agnostic, `core` protocol-free): `{kind,id,author?: EntityDescriptor,body?,timestamp,isOwn?,edited?,deleted?,details?: WidgetMount[],bodyExtras?: WidgetMount[]}`; `WidgetMount {widgetId,props?}`.
- **`message.svelte`** (NEW) — `text` full: composes the **real** `entity-avatar` (`__avatar`) + `label` (`__name`) + `paragraph` (`__body`), no re-implement. `details` resolves `WidgetMount[]` through the consumer widgets registry, **drops unknown `widgetId`** (W-13). `isOwn` → `data-own` mirror. Getter `{kind,isOwn,author,hasBody,detailsCount}`.
- **`.message` skin** — grid: avatar column reserved **both** sides + `[data-own]` mirror + header line + body (no new tokens).
- Sampler: fixture stub widget + 4 DD·composite cells (text-other, text-own, text-details, text-unknown-widget).

**CDP (9422, both accents).** `ids()` **186→202** (+16 = 4 cells × 4: message + `__avatar` + `__name` + `__body`), `count===unique`, 0 orphans. `isOwn` false/true exact; `detailsCount` 2 (known) vs 1 (unknown-widget — drop proven, W-13); grid `28px 288px` ↔ `288px 28px` (mirror); accent-neutral geometry. `vite build` ✓.

**Correction (Rule 1, honest-over-polite).** The runbook's "message root + `__avatar`" registry note **under-counted**: because `message` composes the real `label`/`paragraph` too (per its own DoD), each `text` full cell yields **4** entries, not 2 — hence +16, not +8. Recorded correctly here and in the registry doc.

**Records.** `ui/docs/xgen-ui-components.md` v0.53→v0.54 (message built-table row + Build note (M-RP5.5 A) + count 186→202). `docs/ROADMAP.md` M-RP5.5 A ✅. CLAUDE.md PLAY. `tasks/M_RP5_5_A_MESSAGE_TEXT_FULL.md` → COMPLETED. This entry. Feat `166529e` already pushed; doc-bridge is a second commit — Joe pushes.

**Next-active.** **M-RP5.5 B** — states on `text`: grouped (header-line suppressed, avatar stays) / edited marker / deleted tombstone. Sampler cells each + CDP.

---

## Entry J-477 — M-RP5.5/5.6 Phase-0 LOCKED: the `message` family (2 kinds, `MessageDescriptor` dd-socket, `details`/`bodyExtras` widget sockets, `isOwn` flip); doc written

**Design/records-only, no code.** Ran the D-071 Phase-0 on the message family (opened as next-active at J-476). Discussion → Joe-locked → canonical doc. Joe's early mockup (`ui/docs/user-member-message-ui-concept.jpg`) folded in: composition + own/other flip, ID#/IDU placeholders = avatar initials.

**Locked (Joe).**
- **2 kinds:** `text` (avatar both sides, **reserved column**, `isOwn` flip mirrors the row) + `system` (no avatar either side, centered special-adjust line). grouped/edited/deleted = **fields on `text`**, not types (the `EntityDescriptor.revoked`-is-a-flag discipline).
- **`grouped`** suppresses the **header line only** (name + details); the **avatar stays** (reader orientation) — stream-computed.
- **`MessageDescriptor` dd-socket** (source-agnostic, `core` protocol-free): `kind`,`id`,`author?: EntityDescriptor`(reuse),`body?`(text-node),`timestamp`,`isOwn?`(shell-set),`edited?`,`deleted?`,`details?: WidgetMount[]`,`bodyExtras?: WidgetMount[]`; reserved-unfed reply/attachments/reactions (D-065).
- **Widget sockets (new requirement, Joe):** `message` is a **host surface** for system/custom widgets (all-widgets model, W-12) — `details` (header region: time/temperature/badges/icon-buttons; send-status `led` lives here) + `bodyExtras` (below body; attachments/reactions later). Each a `WidgetMount[]` (`{widgetId,props}`; unknown-`widgetId` dropped, W-13 reconcile). Renamed off `meta` to avoid the `meta`-attributes collision.
- **Author self-status** rides `entity-avatar`'s existing `status?` corner-slot (M-RP5.1b) — no message-level `status` wiring.
- **Stream = `core` dd-composite now** (`message-stream`, the `entity-panel` analogue; `role="log"`, click-select not roving); the **R5 system-widget wrap deferred to M-RP6.x** (needs the region shell — same posture as `temperature-indicator`, J-470). Records honesty: J-465 closed the *entity* dd-composite sub-family; `message`+`message-stream` open a new *message* sub-family, not a tier reopen.

**Sub-milestones.** **M-RP5.5** (`message`): A (`MessageDescriptor` + `text` full) · B (grouped/edited/deleted) · C (`system` + `isOwn` flip) → D-074 close. **M-RP5.6** (`message-stream`): A (section + ordered N `message`s + empty + grouping + day-dividers) · B (scroll: auto-bottom/scrollback/jump-to-latest) → D-074 close.

**Records.** New canonical doc `docs/xgen-dd-message-family-phase0.md` (ACTIVE, v1.0). `docs/ROADMAP.md` **v4.45→v4.46** (M-RP5.5/5.6 block: Phase-0 LOCKED + doc pointer + sub-milestone steps; stale "proposed v1" parenthetical corrected to locked fields). CLAUDE.md PLAY next-active flipped (Phase-0 locked → M-RP5.5 A build). This entry. No code, registry unchanged (**186**). GitHub board: M-RP5.5 card stays PENDING (design-locked; build not yet started). Not pushed — Joe pushes.

**Next-active.** **M-RP5.5 A** — `MessageDescriptor` type + `text` full (avatar + name + body + `details` socket) on sampler fixtures; sampler DD·composite + CDP-verify.

---

## Entry J-476 — Reprioritize: `message` component + R5 stream region pulled ahead of the live-wiring arc

**Design/records-only, no code.** Joe reprioritized: build the **`message` component** first, then the **R5 message-stream region** as a system widget — both **before** the M-RP6.x live-wiring arc. Rationale: R5 is the one region with no built component under it (and the app's whole point), and both build on **sampler fixtures** with no node↔client channel needed — so unit-then-container is the honest order, sequenced with the rest of the dd track.

**Locked sequence.** **M-RP5.5** = `message` **dd-composite** (materializes a protocol message → honest HTML, N-075; composes `entity-avatar`/`label`/`paragraph`/`status`/`led`), a small **type family**. **M-RP5.6** = **R5 message-stream** as a **system widget** (the listbox-analogue of `entity-panel`: wraps N `message`s + ordering / grouping / day-dividers / scroll). Both sampler-verified. **M-RP5.5 opens with a D-071 Phase-0** on the message family (enumerate types, pin v1 vs deferred, map each type → composed atomics, define the `MessageDescriptor` dd-socket shape). Proposed v1: text (author+body+timestamp+send-status), system notice, consecutive-grouping, edited/deleted markers; deferred: reply/quote, attachment/media, reactions.

**`temperature-indicator`** stays **⏸️ POSTPONED** (M-RP5.4 / the M-RP6.5 heat slot) until the main window is functional (Joe) — its blocker was exactly "no `message` component," which M-RP5.5 removes, but Joe defers the build itself to post-functional-window.

**Records.** `docs/ROADMAP.md` **v4.44→v4.45** (M-RP5.5/M-RP5.6 block added as next-active; M-RP6.5 amended — R5 built earlier, heat stays POSTPONED). CLAUDE.md PLAY next-active reprioritized. This entry. No code, registry unchanged (186). Not pushed — Joe pushes.

**Next-active.** **M-RP5.5 Phase-0** on the message family (discussion → doc), then build `message` on sampler fixtures.

---

## Entry J-475 — Region/dock model v1.1: layout persistence (§9) + M-RP7.6 named layouts / manager widget

**Design-only, no code.** Follow-on doc amendment to M-RP6.0d (J-474), answering "how is a custom layout saved?".

**Locked.** Layout = the serializable descriptor (§3) written to a **local** file in the client config dir (Tauri `app_config_dir()`), never federated, per-device. **Auto-save-on-exit** (window close hook, + debounced on mutation) + **auto-load-on-start** (`get_layout()` → saved or default) as the baseline; **manual + named layouts** via a small set file + verbs (`list/save/load/delete/rename_layout`), with a **layout-manager widget** on top (fits the all-widgets model). Verb shape mirrors `get_substitutions`/`set_substitutions` — webview owns the live tree, Rust persists the blob.

**Key correction (Joe).** `widgetId` is the **durable identity**; the display name is a mutable label — so **renaming a widget is a non-issue** (layouts reference ids). A widget update MUST keep its id. On-load reconcile then only handles real identity change: drop unknown-`widgetId` nodes, re-inject missing `system` widgets (W-13, Composer can't be lost); `version` bump + migrate only for descriptor **schema** changes; unrecoverable → default, never crash.

**Records.** `ui/docs/xgen-region-dock-model.md` **v1.0→v1.1** (+§9 Layout persistence). `docs/ROADMAP.md` **v4.43→v4.44** (M-RP7.3 enriched + **M-RP7.6** named layouts / manager widget). This entry. No code, registry unchanged (186). Sequencing: contract free now (`version` + verb siblings of M-RP6.1); baseline persistence at **M-RP7.3**, named layouts at **M-RP7.6**. Not pushed — Joe pushes.

---

## Entry J-474 — M-RP6.0d: region / dock model locked — every UI region is a widget (`system`|`custom`); one serializable layout descriptor for both renderers (D-103)

**Design-only, no code (Rule 1/5: nothing built, nothing to verify — stated plainly).** Joe locked, across a short design discussion, the architecture for the whole client UI panel *before* any panel is built. Opened + closed same session as **M-RP6.0d** (a design beat off the M-RP6.0 gate, ahead of the M-RP6.1+ build arc).

**What was locked.**
1. **The main client UI panel is a layout of dockable regions**, movable/rearrangeable like Maya panes ("undock, hover, plug in").
2. **Every region is a widget** — there is no separate "region" concept. Widgets carry `kind`: **`system`** (the built-in surfaces R1–R8 — pre-installed, **non-removable**, but individually configurable + redockable like any widget) or **`custom`** (install/remove; MAY also contribute a region). This makes the client UI a plugin surface end-to-end: a custom widget can ship a brand-new dockable region.
3. **One serializable layout descriptor** (`Layout = {version, root}`; `LayoutNode = leaf | split | tabs`, leaves referencing widgets by id) is read by **both** renderers — a lean **config-grid (A)** now and an owned **Maya-style dock engine (B)** at M-RP7 — so the dock engine is a **renderer upgrade, not a region rewrite**.
4. **Selection bus** — one active selection `{regionId, entity: EntityDescriptor}` across the layout; feeds **R8 Selection-info** (the inspector: whatever's selected exposes its parameter rows) + reuses the same signal `entity-context-menu` reads. A shell primitive arriving with the region shell (M-RP6.1), consumed from M-RP6.2.

**The 8 regions (all system widgets):** R1 Spaces rail (`entity-panel` space) · R2 Rooms (`entity-panel` room→hexagon) · R3 Self/connection (`entity-item`+`status`+`led`) · R4 Room header (`label`/`section` + temperature) · R5 Message stream (`message`, unbuilt) · R6 Composer (`textarea`+`button`) · R7 Members (`entity-panel` identity) · R8 Selection-info (`section`+`label` rows).

**Two widget-tier constraint additions (v1.1→v1.2):** **W-12** — a widget owns exactly one region (the *layout* sibling of the W-11 *data* dd-socket; promotes the earlier "MAY own a region" to the universal rule). **W-13** — `system` widgets are non-removable (always in the default layout; may collapse/redock/retab/configure but never fully close — a user can't lose the Composer).

**Records (D-074 atomic).** New canonical doc `ui/docs/xgen-region-dock-model.md` **v1.0** (region registry + layout-descriptor schema + region-provider seam + W-12/W-13 + renderer roadmap A→B). `DECISIONS.md` **+D-103** (region/dock model; every region a widget; one descriptor for both renderers). `ui/docs/xgen-widget-tier.md` **v1.1→v1.2** (+W-12/W-13 + a v1.2 reframe note). `docs/ROADMAP.md` **v4.42→v4.43** (M-RP6.0d ✅ DONE block + the 🟡 M-RP6.1+ client-UI-panel arc + the 🟡 M-RP7 dock-engine arc). CLAUDE.md PLAY next-active flipped (M-RP6.0d done; M-RP6.1 = region-shell scaffold + selection bus + R3 + R8, closing the read half of gate finding F-1). This entry. No component/registry/`.svelte` touch (registry unchanged at **186**). GitHub board: M-RP6.0d card → DONE. Not pushed — Joe pushes.

**Next-active.** **M-RP6.1** — open with a D-071 Phase-0 auditing the honestly-bindable client-local state (identity / lifecycle / known-spaces on a clean-slate boot), then build the region-shell scaffold + selection bus + **R3 Self/connection** + **R8 Selection-info** (self = first inspectable) on config-grid renderer A, closing the *read* half of F-1 (the shell's current auto-connect discards its WS stream and fakes Ready — needs a real Tauri read verb + reactive `app.emit` push). Renderer B (the Maya dock engine) is the M-RP7 arc on the same descriptor.

---

## Entry J-473 — M-RP6.0 gate CLOSED: all G1–G5 green → GO; node↔client channel confirmed on current binaries

**Execution + verification, no production code.** Ran the M-RP6.0 Pre-UI Node↔Client Functional Gate (opened+locked J-472). **Verdict: GO** — every gate green on the current binaries; the client UI panel arc may open.

**Drive surface (D-071 coverage-map audit first).** Harness-primary: `xgen-mptest` drives the **real** `xgen-node`/`xgen-client` binaries over the `--aicontrol` JSONL channel and observes convergence via `.events`/`state` (black-box). CDP `get_state` is a supplementary witness only. Reshape from the Phase-0 recommendation, surfaced + Joe-approved before running (D-065): the pre-UI client shell has no connect/send affordances and no Tauri verb-commands (M5 carry-over), so CDP can only *read* state — the harness IS the real client↔node drive. Node built `--features harness-control` (mock clock; the feature lives on `xgen-node`, baked into the built exe — not an `xgen-mptest` feature).

**Builds (real output):** `cargo build -p xgen-node --features harness-control` EXIT=0; `cargo build -p xgen-client` EXIT=0; `cargo test -p xgen-mptest --no-run` EXIT=0.

**Gate→scenario map + results (Rule 2, real quoted output):**
- **G3 send/receive + G4 multi-client/one-node** → **MP-C-01** (`mp_c_01_local_fanout_converges`, `xgen-mptest/tests/mp_r1_c5.rs`): `MP-C-01 PASS (single-node local fan-out): 2 nodes converge on membership {owner+member}`; `test result: ok. 1 passed; 0 failed`, EXIT=0, 19.06s.
- **G5 client rebind** → **MP-C-10** (`mp_c_10_leave_and_rejoin_converges`, same file): `MP-C-10 PASS (leave & rejoin, cross-node A↔B): 2 nodes converge`; `1 passed; 0 failed`, EXIT=0, 22.04s.
- **G1 connect + G2 state sync** → witnessed by both scenarios (clients connect + register + converge on shared membership/transcript); the `get_state` UI-read surface itself was proven live in J-469.

**Findings:** F-1 — no CDP-drivable connect/send at pre-UI stage (client Tauri verb-commands absent, M5 carry-over); closed by the client UI panel arc (M-RP6.1+), NOT a channel defect. F-2 — S5 has no single-node harness scenario; cross-node MP-C-10 served as the rebind witness — adequate for the single-node gate.

**Records (D-074 atomic).** Phase-0 doc `docs/tests/PRE_UI_NODE_CLIENT_GATE_phase0.md` ACTIVE→**COMPLETED** v1.0 (Results + GO section). ROADMAP **v4.41→v4.42** (M-RP6.0 🟢 PLAY → ✅ DONE, J-473; result summary + Next→client UI panel arc). CLAUDE.md PLAY flipped M-RP6.0 ✅ DONE with evidence; next-active = **M-RP6.1+ client UI panel arc**. This entry. GitHub board M-RP6.0 → DONE (`92abd4cb`). No DECISIONS/registry/component touch. Not pushed — Joe pushes.

**Next-active.** M-RP6.1+ — the client UI panel arc: assemble the shipped `core`/dd/widget components onto real node↔client state.

**Recovery anchor.** git tag `m-rp6.0-gate-go` marks this commit as the last known-good state (gate CLOSED GO, node↔client channel confirmed). If a later state can't be recovered, `git reset --hard m-rp6.0-gate-go` returns here.

---

## Entry J-472 — M-RP6.0 LOCK: Pre-UI Node↔Client Functional Gate opened (PENDING→PLAY); G-set G1–G5 + name M-RP6.0 locked (Joe)

**Record action, no code.** Joe locked the two open items from the M-RP6.0 Phase-0 (opened PENDING at J-471): the **G-set** and the **milestone name**. Both locked as recommended.

**Locked (Joe):**
- **G-set = G1–G5** (single-node, client-facing; federation topology out): **G1** connect+`get_state` · **G2** state sync · **G3** send/receive round-trip · **G4** multi-client/one-node (reuse `MULTIPARTY_S1`) · **G5** client rebind (reuse `MULTIPARTY_S5`). G1–G3 are the load-bearing gates (what the client UI panel sits on); G4–G5 are the robustness reuses. **No G6/durability** this gate (logged as a candidate for a later federation/durability gate, D-080 family).
- **Name = M-RP6.0** — opens a fresh RP6 arc (`M-RP6.0` = pre-UI gate; `M-RP6.1+` = the client-UI-panel build). Integration is a new concern, not more RP5 component work.

**Method (from Phase-0):** reuse `xgen-mptest` where it covers G1–G5 + live CDP self-drive of the real binaries (9222 client / 9322 node, J-405) for anything harness-external; real output quoted (Rule 2). **DoD:** each G# green with real quoted output + a short findings list + a **GO/NO-GO** verdict for opening the client UI panel arc. **Not** a re-run of the closed Multiparty-tests milestone (that stays CLOSED, J-356; the consolidated ledger was already delivered there — the earlier "MP-R3 owed" note was stale, corrected J-471).

**Records (D-074 atomic).** Phase-0 doc `docs/tests/PRE_UI_NODE_CLIENT_GATE_phase0.md` PENDING→**ACTIVE** (v0.1→v0.2, G-set + name LOCKED banner + scope section flipped to LOCKED). ROADMAP **v4.40→v4.41** (M-RP6.0 🟡 PENDING → 🟢 PLAY, J-472; tail flipped recommended→LOCKED). CLAUDE.md PLAY next-active flipped to M-RP6.0 PLAY (the stale "Next-active = temperature-indicator" line neutralised — M-RP5.4 stays ⏸️ POSTPONED, J-470). This entry. No DECISIONS/registry/component touch. GitHub board M-RP6.0 → PLAY (`3f8e96a8`). Not pushed — Joe pushes.

**Note (Rule 3, honest).** The Filesystem MCP server went unresponsive mid-bundle (two `edit_file` calls on CLAUDE.md timed out at 4 min each; a light read also hung). Joe killed the hung processes and the server recovered. On recovery it was found the two "timed-out" CLAUDE.md edits had in fact applied to disk — the timeout was in the response channel, not the write. All four `.md` files verified consistent post-recovery before this entry was written (Rule 4: journal last).

**Next-active.** M-RP6.0 gate execution: run G1–G5 live, quote real output, produce findings + GO/NO-GO. On GO → the client UI panel arc (M-RP6.1+).

---

## Entry J-471 — records: MP-R3 "owed" note corrected (already delivered J-356) + Pre-UI Node↔Client Functional Gate opened (M-RP6.0, PENDING)

**Two record actions, no code.**

**(1) MP-R3 capstone ledger — correction, not retirement.** A standing note tracked a consolidated R1+R2+R3 multiparty ledger as *owed* at MP-R3 close (carried as `tasks/HANDOFF_MP_R3.md`). On checking: the Multiparty-tests milestone is **fully CLOSED (J-356)** — MP-R1 (J-340) → MP-R2 (J-348) → MP-R3 capstone (J-356), and ROADMAP line 634 states the **consolidated ledger was delivered** at that close. So the obligation was already discharged; the "owed" note was **stale**, not an open debt. No file to retire; no `tasks/HANDOFF_MP_R3.md` exists (content folded into the J-356 close + the `docs/tests/MULTIPARTY_*` findings/matrix). Chat memory corrected accordingly. Round-2 checkpoint audit remains DONE (J-357, GO).

**(2) Pre-UI Node↔Client Functional Gate opened — `M-RP6.0` (PENDING).** Before building the main client UI panel, a lean **live** re-verification of the node↔client surface against the *current* binaries — a **D-071 subsystem audit** (client UI panel = dependent milestone; node↔client channel = dependency) and the live-functional slice of the planned Round-2 whole-codebase audit. **Not** a re-run of the closed multiparty milestone. Recommended G-set (single-node, client-facing; federation out): **G1** connect+`get_state` · **G2** state sync · **G3** send/receive round-trip · **G4** multi-client/one-node (reuse `MULTIPARTY_S1`) · **G5** client rebind (reuse `MULTIPARTY_S5`). Method: reuse `xgen-mptest` where it covers G1–G5 + live CDP self-drive (9222/9322, J-405) for the rest; real output quoted (Rule 2). DoD: each G# green + findings list + a GO/NO-GO for the client UI panel. Scope is **recommended, not locked** — Joe locks the G-set + final name before the gate opens.

**Records.** New Phase-0 doc `docs/tests/PRE_UI_NODE_CLIENT_GATE_phase0.md` (PENDING, v0.1). ROADMAP v4.40 (M-RP6.0 PENDING pointer; MP-R3 already-delivered clarification). No DECISIONS/registry touch. Next-active is unchanged from the code side (widget tier complete; M-RP5.4 POSTPONED, J-470) — this gate is the recommended next real work before the client UI panel.

---

## Entry J-470 — M-RP5.4 `temperature-indicator` ⏸️ POSTPONED: deferred until fully testable end-to-end (needs a `message` representation + a real activity source)

**Decision (Joe).** The last widget-tier item, `temperature-indicator` (M-RP5.4), is **postponed — not cancelled**. Design and build are deferred until it can be verified end-to-end. No code, no Phase-0 this session; roadmap/pointer record only.

**Why.** Clarified this session that `temperature-indicator` is a **room/message activity** widget — "temperature" = conversation heat (activity level), materialized as a `meter` fill through the W-11 dd-socket that `entity-context-menu` (J-469) just proved real. Its UI representation is a **`message`-shaped surface**, and the registry has **no `message` component yet**. Two prerequisites are therefore missing, both downstream of UI integration: (1) a `message` representation component; (2) a real room/message **activity source** to bind heat against. Building it now would be against stubs we can't honestly verify (W-8 honest-phase-limits / D-065 honest-over-polite / "honest longer work over fast shortcuts").

**State.** Widget tier otherwise complete: `substitutions-editor` (M-RP4.3, J-454) ✅ + `entity-context-menu` (M-RP5.3, J-469) ✅. dd tier closed (J-465). Registry unchanged at **186**, 0 orphans (no component touched). Resumes when a `message` representation + a live activity feed exist — likely alongside real-shell entity-UI integration.

**Records.** ROADMAP (M-RP5.4 ⏸️ POSTPONED, v4.39) + CLAUDE.md PLAY next-active. No `DECISIONS.md`/registry/notes touch (a planning deferral, not a build). Standing: MP-R3 capstone ledger (`tasks/HANDOFF_MP_R3.md`) still owed.

---

## Entry J-469 — M-RP5.3 CLOSED: `entity-context-menu` — the SECOND widget + the FIRST real W-11 dd-socket consumer; overlay behaviour machine + portal-to-body/flip-shift; two-layer verify green (sampler 9422 + real client 9222)

**What happened.** Built `entity-context-menu` per the design-locked `docs/entity-context-menu-phase0.md` (A→H). The **second `widget`** (Level-2, D-102) and the **first widget with a real dd-dependency** — the first genuine exercise of the **W-11 dd-socket** (binds an `EntityDescriptor` + a self-status view-model; `core` imports no protocol type). Five build steps (scaffold → machine → header+item → skin+sampler+pure-CDP → portal+effect-CDP), each Joe-gated. Clair (impl seat); full D-074 close. No `DECISIONS.md` touch (a new `common/widgets/` occupant + additive skin; no wire/prop/protocol change). New file `ui/common/lib/components/widgets/entity-context-menu.svelte`; skin block in `ui/assets/skin.css`; sampler WIDGET cell in `ui/sampler/src/app_sampler.svelte`.

**What it is (Phase-0 lock, honoured).** A gesture-agnostic overlay: exposes `open(anchor)`/`close()`, the consumer wires the trigger (the avatar/item reserved `onActivate?`, right-click, long-press). Root `<div class="entity-context-menu" role="menu">`, `data-tier="widget"`. **Behaviour machine D** (the W-2 discriminator, §2 A — not roving-focus, which `entity-panel` already owns): `closed → open(anchored, focus moved in) → navigating(roving) → dispatch(run handler) → closed`; dismiss = Esc · outside-click · select-then-close · focus-leaves; focus returns to the anchor; W-5 listeners wire on open, tear down on close/unmount. **Header** composes `entity-avatar` + name + `status` **full** (the identity-read surface); item rows = widget-owned `<li role="menuitem">`. Base ships exactly the **universal `identity` item** (kind-labelled; space/room labels reserved; flag-gated per-(variant,purpose) slots declared-not-populated, W-8). Getter G `{open,variant,purpose,kind,itemCount,activeIndex}` — task-state only, no payload (N-060); children self-register `entity-avatar#<id>__avatar` + `status#<id>__status`.

**Portal (§2 C).** `portal` (opt-in) relocates the root to `document.body` (escaping every ancestor `overflow` clip) + `position:fixed` + placement against the anchor's viewport rect (flip below→above on bottom overflow, shift to clamp on right/left). One skin, gated by `data-portal`. **Joe-lock override (this session):** the sampler pure layer runs `portal={true}` too (Phase-0 §2 C had scoped the sampler to an inline `position:absolute` popup; Joe locked "sampler also portals" so its demo matches the real shell — the inline popup overlapped the trigger + ellipsized the status line; the portal presents cleanly). The two-layer verify is otherwise unchanged.

**Verify — pure layer, real CDP output (Rule 2), sampler WIDGET tab, port 9422 (portal on).** `vite build` clean (155 modules; the widget = +1 over J-468's 154). Closed → open → rove → dispatch → teardown:
```
closed:  {"n":186,"ecm":{"open":false,"variant":"card","purpose":"default","kind":"identity","itemCount":1,"activeIndex":-1}}
open:    {"open":true,"activeIndex":0,"n":188,"orphans":0,"hasAvatar":true,"hasStatus":true,"parentIsBody":true,"position":"fixed","dataPortal":"true","zIndex":"1000","inViewport":true,"activeText":"View identity","statusText":"🎧 in a meeting updated 6m ago"}
Enter:   {"afterEnter_open":false,"activeIndex":-1,"focusReturnedToTrigger":true}
teardown:{"n":186,"orphans":0,"childrenGone":true,"selectedNote":"selected: identity"}
```
Reading these out: registry **186** closed (185 J-468 baseline + the widget root; the always-mounted root keeps G readable while closed, N-053) → **188** open (`entity-avatar#demo__avatar` + `status#demo__status` self-register) → back to **186** on close (W-5 teardown), `count===unique` throughout → **0 orphans**. Getter G exact. Open moves `activeIndex` −1→0 + focus into the "View identity" menuitem; header renders the avatar + name + `status full`. Portal: root reparented to `document.body`, `position:fixed`, `z-index:1000`, in-viewport. Enter dispatches → `onSelect("identity")` (`selected: identity`) + closes + returns focus to the trigger. Esc + outside-click dismiss were proven in the Step-4 inline pass (`true→false` each); the machine is portal-agnostic. Both accents: `--accent2` swaps gold `#c28840` ↔ blue `#3a7ab0` (live), menu chrome accent-neutral (`bg rgb(34,38,45)=--s3` identical both shells — the item focus ring is the lone accent touch). Screenshot `temp/ecm-sampler-portal.png` (header AN + Alice Ng + "🎧 in a meeting updated 9m ago" + "View identity", clean popup below the trigger).

**Verify — effect layer, real CDP output (Rule 2), REAL client shell, port 9222.** A minimal, clearly-marked **temporary** harness in `app_client.svelte` (an `overflow:hidden` box existing only to prove the clip escape + a real host-injected `onSelect`); reverted at close (lean chrome restored, J-428 — client frontend 126→**122** modules after revert). Portal escape + host round-trip:
```
open:  {"open":true,"parentIsBody":true,"insideHarness":false,"position":"fixed","zIndex":"1000","dataPortal":"true","menuRect":{"t":53,"l":79,"w":235,"h":88},"harnessRect":{"t":136,"l":70,"w":180,"h":60},"menuTallerThanClipBox":true,"inViewport":true}
select:{"afterEnter_open":false,"activeIndex":-1,"handlerResult":"dispatch:identity -> host get_state=DISCONNECTED","focusReturnedToTrigger":true,"childrenGone":true,"menuContentGone":true,"widgetStillRegistered":true}
```
Reading these out: the menu **portals out of the `overflow:hidden` clip box** (`parentIsBody:true`, `insideHarness:false`), renders **88px tall inside a 60px clip box yet fully in-viewport** (an inline popup would be sliced), `position:fixed` `z-index:1000`; the **flip** fired (top:53, above the trigger — the small client window couldn't fit it below). Select → the injected `onSelect` fired **and reached real host I/O**: `handlerResult:"dispatch:identity -> host get_state=DISCONNECTED"` (a real `invoke('get_state')` returning the live client state) — the full W-3 host-injected round-trip; then close + focus-return across the portal boundary + W-5 teardown (portaled children removed from `body`), the always-mounted root persisting. Screenshot `temp/ecm-effect-layer.png` (portaled menu above the trigger, gold focus ring on "View identity", the round-trip line at the foot). **Both verify homes green → done** (widget-tier §5).

**Two honest notes (Rule 6, recorded not hidden).** (1) **ArrowDown clamps, does not advance** — the base ships exactly one item by design (universal `identity`; W-8), so multi-row advance is catalogue-bounded, not a gap; the roving arithmetic is `Math.min(n-1, activeIndex+1)` and open already moves `activeIndex` −1→0. (2) The status-full line **ellipsizes** at the min-width in a cramped container (was visible in the inline sampler popup as "in a me…"; the portal placement gives it natural width so it shows in full) — cosmetic, Joe HMR-tunes.

**Registry.** 185 (J-468) → **186** (the widget root, closed) → **188** open (+`entity-avatar#demo__avatar` +`status#demo__status`). 0 orphans at every state.

**Remaining / next.** The widget tier has **one left**: `temperature-indicator` (M-RP5.4) — the first-conceived widget, binding `meter` through the W-11 dd-socket this widget just exercised for real. Kind 4 `use:render` stays deferred (D-065). Standing: MP-R3 capstone ledger owed (`tasks/HANDOFF_MP_R3.md`).

---

## Entry J-468 — M-RP5.0d CLOSED: hexagon badge-clip fix — the fill-layer refactor; shape moves to an inner `.ea-fill` so the status badge + isAi spark sit on unclipped corners; resolves the M-RP5.0c PROVISIONAL

**What happened.** Fixed the M-RP5.0c PROVISIONAL badge-clip per `tasks/RUNBOOK_HEXAGON_FILL_FIX.md` (option A, fill-layer refactor; design Joe-locked). Clair (impl seat); full D-074 close. No `DECISIONS.md` touch (additive skin/component refactor — no new component, no wire/prop change).

**The bug.** The hexagon `clip-path` lived on the root `<figure class="entity-avatar">`, so its clip region also clipped every descendant — the `status` corner badge and the isAi `::after` spark were **sliced** by the hull. M-RP5.0c only *nudged* the badge inward to hide the slice (a workaround, and the isAi spark had the same latent clip).

**The fix (option A).** Move the SHAPE off the root onto an inner layer:
- `entity-avatar.svelte`: a new absolutely-positioned `<span class="ea-fill" aria-hidden>` is the FIRST child; it carries shape + seed bg/border. The root `<figure>` is now transparent, un-clipped, `overflow:visible`; initials + the `status` badge + the `::after`/`::before` badges stay on the root, above the fill.
- `ui/assets/skin.css`: `background`/`border`/`border-radius` + the hexagon `clip-path` moved from `.entity-avatar` → `.entity-avatar .ea-fill` (`position:absolute; inset:0; z-index:-1`); `[data-shape="square"]`/`[data-shape="hexagon"]` now target `.ea-fill`. The root gains `isolation:isolate` so the fill's `z-index:-1` stays contained (paints behind the in-flow initials + positioned badges). The M-RP5.0c hexagon `.status` nudge is **removed** — the badge returns to the standard `-3px` bottom-right corner, now unclipped.

**Verify — real CDP output (Rule 2), sampler DD·atomic, port 9422.** `vite build` clean (154 modules; only the pre-existing `link.svelte` `state_referenced_locally` notes). Hexagon room-with-status + isAi spark + shapes + registry:
```
{"count":185,"unique":185,"room":{"dataShape":"hexagon","rootClip":"none","fillClip":"polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%)","fillBg":"rgb(188, 230, 207)","justify":"center","align":"center","overflow":"visible","badgePos":"absolute","avRect":{"l":551,"t":540,"r":579,"b":568,"w":28,"h":28},"badgeRect":{"l":567,"t":559,"r":582,"b":571,"w":15,"h":11},"badgeBottomRightCorner":true,"badgeInViewport":true},"circle":{"dataShape":"circle","rootClip":"none","fillRadius":"50%","fillClip":"none","fillBg":"rgb(230, 188, 188)","fillBorder":"0.8px rgb(224, 184, 184)"},"square":{"dataShape":"square","rootClip":"none","fillRadius":"0px","fillClip":"none","fillBg":"rgb(197, 230, 188)"},"dm":{"dataShape":"circle","rootClip":"none","fillRadius":"50%","fillClip":"none"},"hexPlain":{"dataShape":"hexagon","rootClip":"none","fillClip":"polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%)"}}
```
isAi spark (`::after`) + `--rad0` + revoked slash (corrected probe — the earlier `cs(el,'::after')` helper had dropped the pseudo arg):
```
{"rad0":"0","ai":{"hasDataAi":true,"dataAi":"true","rootClip":"none","afterContent":"\"\"","afterTop":"-1px","afterRight":"-1px","afterW":"11.75px","afterBg":"rgb(109, 92, 231)","aiRect":{"l":427,"t":447,"r":455,"b":475,"w":28,"h":28}},"revoked":{"hasDataRevoked":true,"beforeContent":"\"\"","beforeInset":"0px 0px 0px 0px","rootClip":"none"}}
```
Reading these out: **hexagon** root `clip-path:none` + `overflow:visible`; `.ea-fill` carries the hexagon polygon + seed fill `rgb(188,230,207)`; initials `justify/align:center`. **Status badge** `position:absolute`, rect (567,559)–(582,571) — bottom-right of the 28×28 avatar (551,540)–(579,568), `badgeBottomRightCorner:true`, in-viewport → **un-sliced** (no longer inside a clipped ancestor). **isAi spark** `::after` content generated, `top:-1px right:-1px` (top-right), violet `rgb(109,92,231)`, on an unclipped root. **0-regression**: circle/DM `border-radius:50%`, square `0px` (`--rad0` is `0` — pre-existing, not changed here), all with the seed fill + 0.8px ring, root `clip-path:none`. Registry **185**, `count===unique===185` → **0 orphans** (no new component; matches J-467). Screenshot `temp/room-hex-fix.png` (green "GE" hexagons; the room-status dot on the bottom-right corner, unsliced; the `#ai` violet spark top-right; revoked "MA" greyed+slashed).

**Remaining PROVISIONAL (unchanged, flagged before build — Rule 6).** The seed **ring on the diagonal hull** is still absent: a CSS `border` only draws on the rectangular border-box, so the `clip-path` cuts bare fill along the four diagonals regardless of which layer owns the clip (`fillBorder` reads `0.8px` but it only shows on the flat top/bottom). The seed FILL carries the diagonals — same accepted posture as J-467. A true diagonal ring would need a two-layer / drawn-hull technique (out of scope; Joe HMR-tunes if wanted). The badge-slice — the locked deliverable — is fixed.

**Verify finding (Rule 3).** The first measurement pass returned zero rects + a bogus `::after` read: the DD·atomic panel is CSS-hidden when its tab is inactive (M-RP4.9 confined-scroll shell), so `getBoundingClientRect` and pseudo layout are unavailable — clicking the `DD Atomics` tab (`role=tab`) first gave real geometry. (My initial probe also had a helper bug — `cs(el)` ignored the `::after` arg; the corrected `getComputedStyle(el,'::after')` pass is the second block above.) A `run-sampler` dev session launched clean this time (no HMR overlay).

**Records (atomic, D-074).** Edited `ui/core/lib/components/data-dependent/entity-avatar.svelte` (`.ea-fill` layer), `ui/assets/skin.css` (shape/seed → `.ea-fill`, root `isolation:isolate` + un-clipped, hexagon `.status` nudge removed). Docs: `ui/docs/xgen-ui-notes.md` N-081 (v0.65), `ui/docs/xgen-ui-components.md` registry v0.52 (avatar internal note; no surface change), `docs/ROADMAP.md` v4.37 (M-RP5.0d block + tree tail), this PLAY (→ J-468), this entry, runbook → COMPLETED (DoD ticked). No `DECISIONS.md` touch.

**Next-active.** The dd track is complete and the room kind + its badge-clip are settled. Next is the **widget tier**: `entity-context-menu` (M-RP5.3 — the 100% entity read; consumes the reserved `onActivate?`; uses `status full`) → `temperature-indicator` (M-RP5.4 — `meter` via the W-11 dd-socket). Not pushed — Joe pushes.

---

## Entry J-467 — M-RP5.0c CLOSED: the `room` kind — a third entity kind (hexagon avatar), additive to `EntityDescriptor` + `entity-avatar`; ripples free through item/panel

**What happened.** Added **`room`** as a third entity kind per `tasks/RUNBOOK_ROOM_KIND.md` (design Joe-locked A–E, Phase-0 `docs/xgen-dd-room-kind-phase0.md` v1.1). A room is a first-class location entity — a peer to `space` (a room/channel inside a space), NOT a variant of it. Additive to `EntityDescriptor` + the `entity-avatar` shape branch; **ripples free** through `entity-item`/`entity-panel` with zero code change there. Clair (impl seat); full D-074 close.

**What it is.** `kind: 'room'` (Option A — own kind; `flags.isRoom` Option B rejected as it hides a location peer inside `space` and muddies the taxonomy). Kind → shape: `identity` = circle · `space` (non-DM) = rounded-square · DM (`space`+`flags.isDm`) = circle · **`room` = hexagon**. `EntityDescriptor.kind` union `'identity' | 'space'` → `+ 'room'` (source-agnostic; `core` protocol-free). The `entity-avatar` shape derive gained a `kind === 'room' ? 'hexagon'` branch (`data-shape="hexagon"`); ring/seed/initials/status all inherit — no structure change.

**Hexagon skin (`clip-path`, PROVISIONAL).** `.entity-avatar[data-shape="hexagon"] { clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%) }` (a pointy-top hexagon). Two `clip-path` consequences, both surfaced + accepted (Joe HMR-tunes, same posture as `meter`/`range`): (1) the seed **border-ring** is diminished on the diagonal edges (clip clips the border-box) — the seed FILL carries the shape; (2) the status corner badge would be **sliced** by the clip (it is a descendant of the clipped element), so `.entity-avatar[data-shape="hexagon"] .status` is **nudged onto the lower-right hull** (inset `right`/`bottom`, tuned via HMR) instead of the box-corner `-3px` — decision C. Initials stay centered (the base flex `justify/align:center` is untouched by clip — decision D).

**Ripple (free — the reason it's a kind, not a component).** `entity-item` (variant=row) and `entity-panel` compose `entity-avatar`; a `room` descriptor flows straight through with **no `entity-item`/`entity-panel` code change**. Only four files touched: `types.ts` (kind union), `entity-avatar.svelte` (shape branch), `skin.css` (hexagon), `app_sampler.svelte` (cells).

**Verify — real CDP output (Rule 2), sampler DD·atomic + DD·composite, port 9422.** `vite build` clean (154 modules; only the pre-existing `<figure>` a11y note). Room getter + shape + clip + initials + 0-regression:
```
{"roomGetter":{"kind":"room","variant":"list","name":"general","initials":"GE","seed":"hsl(147 45% 82%)","flags":{}},"roomDataShape":"hexagon","roomClipPath":"polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%)","roomInitialsText":"GE","roomJustify":"center","roomAlign":"center","statusPos":"absolute","identityShape":"circle","spaceShape":"square","dmShape":"circle","spaceClip":"none","identityGetterKind":"identity"}
```
Ripple (DD·composite item + panel room avatars):
```
{"itemAvatarShape":"hexagon","panelAvatar1Shape":"hexagon","panelAvatar2Shape":"hexagon","itemAvatarClip":"polygon(50% 0%, 100%…"}
```
Registry:
```
{"count":185,"unique":185,"roomIds":["entity-avatar#room-item__avatar","entity-avatar#room-list","entity-avatar#room-presence","entity-avatar#room-status","entity-avatar#rooms-dev-2b__avatar","entity-avatar#rooms-general-1a__avatar","entity-item#room-item","entity-item#rooms-dev-2b","entity-item#rooms-general-1a","entity-panel#rooms","section#rooms__section","status#room-status__status"]}
```
Reading these out: **kind='room' getter** works, seed-coloured (`hsl(147…)`), initials "GE"; **`data-shape="hexagon"` + clip-path applied**; **initials centered** (`justify/align: center`); status positioned absolute on the hull (nudged, PROVISIONAL). **0-regression**: `identity`=circle, `space`=square (`clip:none`), DM=circle — all unchanged. **Ripple**: `entity-item#room-item`'s avatar + both `entity-panel#rooms` row avatars are `data-shape="hexagon"` with clip-path, **no item/panel code change**. Registry **173→185** (+12 = 3 DD·atomic room avatars + 1 `status#room-status__status` child + item-ripple 2 + panel-ripple 6); `count===unique===185` → **0 orphans**. Screenshot `temp/room-verify.png` (green "GE" hexagons; room-status badge on-hull).

**Verify finding (Rule 3).** A second transient Vite dev-server startup overlay appeared this session ("Can only bind to state or props" at the `entity-panel#rooms` cell). Same class as the M-RP4.9 one: the production build passed, the app was mounted (registry 185, the rooms panel getter `{count:2,…}` read fine — `bind:selected={epSelRooms}` where `epSelRooms = $state()` is valid, identical to the working `epSelDms`), and `location.reload()` cleared it. A dev-server startup race, not a real error — reload to clear, trust the build.

**Records (atomic, D-074).** Edited `ui/core/lib/components/data-dependent/types.ts` (kind union), `entity-avatar.svelte` (shape branch), `ui/assets/skin.css` (hexagon clip-path + status nudge), `ui/sampler/src/app_sampler.svelte` (room cells). Docs: `ui/docs/xgen-ui-notes.md` N-080 (v0.64), `ui/docs/xgen-ui-components.md` registry v0.51 (avatar kind taxonomy += room), `docs/ROADMAP.md` v4.36 (tree tail + M-RP5.0c block), `docs/xgen-dd-room-kind-phase0.md` (Status→COMPLETED v1.2), this PLAY (→ J-467), this entry, runbook → COMPLETED (DoD ticked). No `DECISIONS.md` touch (additive dd-atomic amendment; N-080 is a component note).

**Next-active.** The dd track is complete (avatar/item/status/panel + room kind). Next is the **widget tier**: `entity-context-menu` (M-RP5.3 — the 100% entity read; consumes the reserved `onActivate?`; uses `status full`) → `temperature-indicator` (M-RP5.4 — `meter` via the W-11 dd-socket). Not pushed — Joe pushes.

---

## Entry J-466 — M-RP4.9 CLOSED: sampler static-header + confined scroll + tab rename (test-bed ergonomics; zero component/registry touch)

**What happened.** Reorganized the `xgen-sampler` shell per `tasks/RUNBOOK_SAMPLER_STATIC_HEADER.md` (design Joe-locked, Phase-0 `docs/xgen-sampler-static-header-phase0.md`): the header block became **fixed**, the panel body became the **only** scroller, and the tab labels were renamed. Sampler-infra only (D-097/D-098) — **no `core` component, no registry delta, no components-doc/DECISIONS touch**. Clair (impl seat); full D-074 close.

**What changed.** (1) **Static header + confined scroll** (`ui/sampler/src/app_sampler.svelte` + `ui/sampler/src/app.css` — NOT `ui/assets/skin.css`; the runbook's path is the usual shorthand). `#sampler-root` is now a `height:100%` flex column with `overflow:hidden`; the header block — `.sampler-bar` (title + `client|node` skin-swap) + `.sampler-tabs` — is `flex:0 0 auto` (the bar's old `position:sticky` retired); the five `.sampler-panel`s are wrapped in a new **`.sampler-scroll`** = `flex:1 1 auto; overflow-y:auto; min-height:0`. The `min-height:0` is the flexbox-scroll gotcha (without it the flex child won't shrink below its content and the overflow never engages). The tab bar no longer scrolls away with the content; the document itself never scrolls. (2) **Tab rename** (string-only, `tabs` const labels): `DI · atomic → DI Atomics` · `DI · composite → DI Composites` · `DD · atomic → DD Atomics` · `DD · composite → DD Composites` · `WIDGET → Widgets`. The `id` keys (`di-atomic`/…/`widget`) are unchanged → N-053 keyed routing + CDP tab-indices unaffected.

**Verify — real CDP output (Rule 2), sampler 9422.** `vite build` clean (154 modules, unchanged — no new module). Header-fixed + labels + registry + confinement:
```
{"tabLabels":["DI Atomics","DI Composites","DD Atomics","DD Composites","Widgets"],"registryCount":173,"unique":173,"scrollerExists":true,"canScroll":true,"scrolledTo":400,"barTopBefore":0,"barTopAfter":0,"tabsTopBefore":44.2,"tabsTopAfter":44.2,"headerFixed":true,"docScrollable":false}
```
Routing intact (after clicking DD Composites):
```
{"visiblePanels":1,"activeHasEntityPanel":true,"dmsRendered":true,"itemRendered":true}
```
Reading these out: **five new labels**; **registry `ids().length` unchanged at 173** (`unique:173` → 0 orphans, no component delta). **Header fixed while body scrolls**: `.sampler-scroll.scrollTop` set to `400` while `.sampler-bar` top stays `0` and `.sampler-tabs` top stays `44.2` (`headerFixed:true`); **scroll confined** — `document.documentElement` not scrollable (`docScrollable:false`). **Routing unchanged**: DD Composites → exactly 1 visible panel containing a rendered `entity-panel#dms` + `entity-item#card-space`. Screenshot `temp/static-header-verify.png` (fixed header + tab bar over a scrolled body).

**Verify finding (surfaced, D-065 / Rule 3).** A transient Vite HMR error overlay ("`<div>` was left open at :276") appeared over the running app at startup, but: the production `vite build` **passed** (a genuinely unbalanced file can't compile), the app was mounted (all CDP probes succeeded), and after `location.reload()` the overlay was gone (`overlayPresent:false`, registry 173). A stale HMR startup artifact, not a real parse error — the markup is balanced. Investigated rather than ignored.

**Records (atomic, D-074).** Edited `ui/sampler/src/app_sampler.svelte` (`.sampler-scroll` wrapper + tab-label rename), `ui/sampler/src/app.css` (fixed-header + scroll CSS). Docs: `ui/docs/xgen-ui-notes.md` N-079 (v0.63), `docs/ROADMAP.md` v4.35 (tree tail + M-RP4.9 block), `docs/xgen-sampler-static-header-phase0.md` (Status→COMPLETED v1.1), this PLAY (→ J-466), this entry, runbook → COMPLETED (DoD ticked). **No `ui/docs/xgen-ui-components.md` change (no component/registry delta), no `DECISIONS.md` touch** (sampler ergonomics; N-079 is a sampler note, not a component contract).

**Next-active.** `room` kind (M-RP5.0c — an additive `EntityDescriptor.kind='room'` + a hexagon avatar shape) → the widget tier: `entity-context-menu` (M-RP5.3) → `temperature-indicator` (M-RP5.4). Not pushed — Joe pushes.

---

## Entry J-465 — M-RP5.2 CLOSED: `entity-panel` — the LAST dd-composite (roving-focus listbox); the avatar corner-fix (H); the `entity-item` status-forward amendment ("status once per variant"). The dd-composite tier is CLOSED.

**What happened.** Built `entity-panel` per `tasks/RUNBOOK_ENTITY_PANEL.md` (design Joe-locked A–H, Phase-0 `docs/xgen-dd-spaces-panel-phase0.md` v1.1). The **last dd-composite** — closes the dd-composite tier; the widget tier (M-RP5.3/5.4) is next. Clair (impl seat); explicit handoff for the full D-074 close. One wiring gap surfaced mid-build and was Joe-resolved before the sampler cells (the status-forward, below).

**What it is.** A dd-composite that materializes a LIST of address-book entries as a keyboard-navigable, single-select group: a `section` (group chrome) wrapping a `<ul role="listbox">` of `entity-item` rows, owning the roving focus + selection + empty state (which is why it is its own composite, not a bare `section` with rows). `spaces-panel` is a consumer **preset** (a title + a spaces `items` array), not a separate component — entity-generic. Source-agnostic: `items` is an `EntityItemInput[]` view-model (`EntityDescriptor` + caller secondary/status/meta); `core` imports no protocol type.

**Rooting (the "wrap, not beside" lock).** The outermost DOM is the `<section>` (from the Section child, self-registers `__section`); the panel's own identity + getter G ride the **`<ul class="entity-panel" use:envelope role="listbox">` INSIDE the section body** — the panel IS the listbox, Section frames it (unlike status-indicator/entity-item, whose root is their own `<div>`). The `<ul>` always renders so the envelope anchor is stable — holding the option rows, or (empty) a single `role="presentation"` `<li>` with a composed `paragraph` message. Composes real children (matrix multiplies deeply): `section` + `entity-item ×N` (each → `entity-avatar __avatar` + optional `status __status`).

**Focus (C/D).** Roving tabindex — exactly one row `tabindex=0` (active), the rest `-1`; ArrowUp/Down move, Home/End jump, Enter/Space select+activate, click selects+activates. `selected` ($bindable id) is the persistent choice (→ each row's `aria-selected` + the row's own `[data-selected]` highlight); `activeIndex` is the transient focus position (seeded once at the selected row via a function call — a deliberate one-time capture, restructured to silence the `$state`-init prop-reference warning). Rows are `<li role="option">` wrapping `entity-item variant="row"`: the li owns option semantics + keyboard + click; entity-item is the visual. Getter G `{ count, selected, collapsed, hasEmpty }`; `collapsed` shared with Section (pass-through F). Empty → `emptyText` message + `hasEmpty:true`.

**Avatar corner-fix (H, done first, skin-only).** The `entity-avatar` isAi spark `::after` moved bottom-right → **top-right** (`top:-1px`); the status corner-slot stays bottom-right. So an AI identity that also carries a status no longer overlaps — distinct corners. Resolves the PROVISIONAL note from the M-RP5.1b close.

**`entity-item` status-forward amendment (Joe-lock, same-commit; "status once per variant").** Phase-0 §1 said panel rows inherit the avatar's status corner-slot "for free", but `entity-item` (built M-RP5.1, before the avatar got `status?`, J-464) never forwarded `status` to its avatar child — the wiring the M-RP5.1b slot + corner-fix H were *for*. Added it: a variant shows status **once** — `card` = the inline status LINE (it has room), so its avatar gets **no** corner badge; `row`/`nav`/`inline` (no room for a line) forward `status` to the avatar's **corner badge** (`avatarStatus = variant === 'card' ? undefined : status` on `<EntityAvatar>`). This is completing intended wiring, not a retrofit (D-065-honest) — Phase-0's "for free" was aspirational; runbook Step 2 didn't enumerate it. Recorded as an additive M-RP5.1 amendment; the M-RP5.1 `card-space` cell stays **0-regression** (verified below).

**Verify — real CDP output (Rule 2), sampler DD·composite panel, port 9422.** `vite build` clean (154 modules; zero `entity-panel`/`entity-item` warnings). Getters G exact:
```
"entity-panel#spaces":{"count":3,"selected":"xgen://space/design-7a2","collapsed":false,"hasEmpty":false}
"entity-panel#dms":{"count":3,"selected":null,"collapsed":false,"hasEmpty":false}
"entity-panel#empty":{"count":0,"selected":null,"collapsed":false,"hasEmpty":true}
"entity-panel#collapsed":{"count":2,"selected":null,"collapsed":true,"hasEmpty":false}
```
Registry delta + child registration:
```
{"count":173,"unique":173,"panelCount":4,"sectionChildren":4,"emptyPara":["paragraph#empty__empty"],
 "dmsIds":["entity-avatar#dms-alice-7f3a__avatar","entity-avatar#dms-aria-ai__avatar","entity-avatar#dms-dm-bob-9c04__avatar","entity-item#dms-alice-7f3a","entity-item#dms-aria-ai","entity-item#dms-dm-bob-9c04","entity-panel#dms","section#dms__section","status#dms-aria-ai__avatar__status","status#dms-dm-bob-9c04__avatar__status"]}
```
Roles + roving tabindex + collapse + corner-fix:
```
{"ulRole":"listbox","ulClass":"entity-panel","optCount":3,"optRows":[{"role":"option","tabindex":"-1","ariaSelected":"false"},{"role":"option","tabindex":"0","ariaSelected":"true"},{"role":"option","tabindex":"-1","ariaSelected":"false"}],"tabZeroCount":1,"emptyOptCount":0,"emptyMsgText":"No blocked users.","collDataCollapsed":"true","collBodyDisplay":"none","ariaHasDataAi":"true","ariaAfterTop":"-1px","ariaAfterBottom":"15.65px","statusPosition":"absolute","statusBottom":"-3px","statusRight":"-3px"}
```
Keyboard nav (dms panel — ArrowDown from row 0, then Enter):
```
{"tabsAfterArrow":["-1","0","-1"],"focusedIndex":1}          // ArrowDown → roving 0 moved to row 1, DOM focus at 1
{"selectedAfterEnter":"xgen://identity/alice-7f3a","ariaAfterEnter":["false","true","false"]}   // Enter → selected + aria-selected row 1
```
0-regression + status-once + corner-fix:
```
{"cardHasAvatarStatusChild":false,"cardGetter":{"variant":"card","kind":"space","name":"Dev Team","hasSecondary":true,"hasStatus":true,"selected":false},"cardInlineStatus":"🟢 online","rowHasAvatarStatusChild":true,"aiCellAfterTop":"-1px","avatarWithStatusBottom":"-3px"}
```
Reading these out: registry **146→173** (+27 across 4 panels: spaces 8 + dms 10 + empty 3 + collapsed 6); `count===unique===173` → **0 orphans**. **Child registration**: 4 panels, 4 `section#…__section`, the empty `paragraph#empty__empty`; the DMs subtree = panel + section + 3 `entity-item#dms-<key>` + 3 `entity-avatar#…__avatar` + 2 `status#…__avatar__status` (Bob 🟢 + Aria 🤖 rows forward status to the corner; Alice has none) — **the forward proven**. **Roles**: ul `role=listbox` class `entity-panel`; 3 `role=option`; **roving tabindex `["-1","0","-1"]` (one `0`, on the selected design row), aria-selected mirrors `selected`**. **Keyboard**: ArrowDown → tabindex `["-1","0","-1"]` moved to row 1 + `document.activeElement` at index 1; Enter → `selected="xgen://identity/alice-7f3a"` + `aria-selected ["false","true","false"]` (Enter → selectAt → onActivate fired + selection set). **Empty**: 0 options + "No blocked users." **Collapse**: `section[data-collapsed=true]` + body `display:none` (rows still registered → slot mounted). **Corner-fix**: Aria avatar `data-ai` `::after top:-1px` (top-right, not bottom) + status `position:absolute bottom:-3px right:-3px` (bottom-right) — no overlap on one glyph. **0-regression + status-once**: `card-space` getter unchanged, inline status "🟢 online" intact, **no avatar corner badge** (`cardHasAvatarStatusChild:false`); a DM **row** avatar DOES carry the corner badge (`rowHasAvatarStatusChild:true`); the `with-status` badge (M-RP5.1b) still `bottom:-3px`. Screenshot `temp/entity-panel-verify.png` (spaces w/ Design selected; DMs w/ Bob 🟢 corner + Aria isAi-top/🤖-bottom; "No blocked users."; Archived collapsed).

**Engineering judgment (surfaced, D-065).** (1) The li owns the option semantics + roving tabindex + click/keyboard; `entity-item` is the pure visual (no `onActivate` passed to it → single activate target, no double-fire). (2) The empty message is a composed `paragraph` in a `role="presentation"` li so the listbox has no phantom option. (3) The status-forward gap was caught while writing the sampler cells and surfaced to Joe rather than silently modifying the closed `entity-item` — Joe locked Option 2 (status-once). (4) `activeIndex` init reads props once by design; restructured into a function call to keep the build warning-free.

**Records (atomic, D-074).** New: `ui/core/lib/components/data-dependent/entity-panel.svelte`; edited `entity-item.svelte` (status-forward `avatarStatus`), `ui/assets/skin.css` (isAi `::after` top-right corner-fix + `.entity-panel` block), `ui/sampler/src/app_sampler.svelte` (DD·composite panel cells). Docs: `ui/docs/xgen-ui-notes.md` N-078 (v0.62), `ui/docs/xgen-ui-components.md` registry v0.50 (entity-panel row + spaces-panel BUILT + avatar corner-fix + build note), `docs/ROADMAP.md` v4.34 (tree tail + M-RP5.2 block), `docs/xgen-dd-spaces-panel-phase0.md` (Status→COMPLETED v1.2), this PLAY (→ J-465), this entry, runbook → COMPLETED (DoD ticked). No `DECISIONS.md` touch (N-078 is a component note; the amendment + rooting are arc-local; D-069 bar not met).

**Next-active.** The **dd-composite tier is CLOSED**. Next is the **widget tier**: `entity-context-menu` (M-RP5.3 — the 100% read of an entity; consumes the reserved `onActivate?`; uses `status full`) → `temperature-indicator` (M-RP5.4 — `meter` via the W-11 dd-socket). Kind 4 `use:render` stays deferred (D-065). Not pushed — Joe pushes.

---

## Entry J-464 — M-RP5.1a + 5.1b CLOSED: `status` — the self-status dd-atomic (badge/line/full) + the `entity-avatar` `status?` corner-slot; mounted-but-empty on expiry (runbook wording corrected); the `.status` naming-collision fix

**What happened.** Built `status` (dd-atomic) + the additive `entity-avatar` `status?` corner-slot per `tasks/RUNBOOK_STATUS.md` (design Joe-locked A–G, Phase-0 `docs/xgen-dd-status-phase0.md`). One runbook — the badge IS the avatar slot payload, so they ship together (M-RP5.1a + 5.1b). Clair (impl seat); explicit handoff for the full D-074 atomic close. One interpretation point was Joe-resolved at go: how an *expired* status behaves in the DOM (below).

**What it is.** A dd-atomic that materializes a **self-set status** (Track A `state.status`, J-461) into a visual — personal **EXPRESSION** (emoji + short line), **NOT** presence/connection state (Track A deferred presence, so it never renders here/away). Source-agnostic: consumes a view-model `{ emoji?, text?, updatedAt?, expiresAt? }` (the shell maps Track A's `StatusRecord`); `core` imports no protocol type. **Variants = display density:** `badge` (emoji corner) · `line` (emoji + text) · `full` (+ relative "updated Nm ago"). Root: `badge` = `<span class="status" role="img">` (aria-label = text ?? emoji); `line`/`full` = plain `<span class="status">`. `title` tooltip fallback (E). Getter `{ variant, emoji, hasText, expired }`. Emoji is a single grapheme (Track A cap), taken grapheme-safe.

**Mounted-but-empty on expiry (Joe-locked; corrects the runbook wording).** The runbook DoD said "expired → absent (not rendered)" while also asking for "getter G per cell" — a genuine tension, since a not-rendered instance can't register its getter. Joe's call: the root `<span use:envelope>` **always mounts** (so getter G always registers and `expired:true` stays CDP-readable), but renders **no emoji/text content** when expired (lazy `expiresAt < now`) or content-less; a `data-empty` attribute then collapses it (`.status[data-empty] { display:none }`). "Absent" = empty content, **not** absent DOM. Rationale: matches the envelope-always-registers-while-mounted convention, keeps `expired` testable, gives a **stable registry count** (no cell blinks in/out on expiry), and is the honest "status exists but expired" state. **The runbook's "not rendered" wording is superseded by this** (noted so it doesn't mislead next time).

**Avatar seam (M-RP5.1b).** `entity-avatar` gains a `status?` prop; when passed it renders `<Status status variant="badge" id={cid('status')} />` inside the figure as a bottom-right corner overlay (`.entity-avatar .status` = `position:absolute` on the `position:relative`/`overflow:visible` figure). The child self-registers `<id>__status`; the status atomic owns expiry (no duplication in the avatar). Additive — the avatar's own getter is unchanged (presence/list/labeled/card + entity-item 0-regression).

**The `.status` naming-collision fix (D-065 — surfaced + fixed, not papered).** `status` is a generic class, and `combobox`/`tag-select` popups already render a bare `<span class="status">` for an option-status label (`.combobox-list .status` / `.tag-select-list .status`). The new component's envelope type-class is also `status`, so a bare `.status { display:inline-flex; … }` base rule **leaked** into those closed components. Fix: the component **always** emits `data-variant` (badge/line/full) while the ad-hoc popup `.status` never does, so the base rule is scoped **`.status[data-variant]`** — matches only the component. No closed-component edits; a synthetic bare `.status` probe reads `display:inline` (leak gone). PROVISIONAL note also surfaced: the isAi spark (`.entity-avatar[data-ai]::after`) also sits bottom-right → an AI identity that *also* sets a status overlaps there (left for Joe's HMR tuning; self-status keeps the locked corner).

**Verify — real CDP output (Rule 2), sampler DD·atomic panel, port 9422.** `vite build` clean (153 modules; zero `status.svelte` warnings). Per-cell getters:
```
"status#badge-emoji":{"variant":"badge","emoji":"🎯","hasText":false,"expired":false}
"status#badge-noemoji":{"variant":"badge","emoji":null,"hasText":true,"expired":false}
"status#expired":{"variant":"badge","emoji":"💤","hasText":true,"expired":true}
"status#line":{"variant":"line","emoji":"🌙","hasText":true,"expired":false}
"status#line-noemoji":{"variant":"line","emoji":null,"hasText":true,"expired":false}
"status#full":{"variant":"full","emoji":"🎧","hasText":true,"expired":false}
"status#with-status__status":{"variant":"badge","emoji":"🟢","hasText":true,"expired":false}
"entity-avatar#with-status":{"kind":"identity","variant":"list","name":"Alice Ng","initials":"AN","seed":"hsl(0 45% 82%)","flags":{}}
```
Registry delta + 0-regression counts:
```
{"count":146,"unique":146,"statusCount":7,"statusChild":1,"avatarCount":17,"itemCount":7}
```
Render / layout (DD·atomic tab visible):
```
{"expiredTag":"SPAN","expiredDataEmpty":"true","expiredDisplay":"none","expiredText":"","badgeRole":"img","badgeAria":"🎯","badgeEmoji":"🎯","badgeDisplay":"flex","noemojiTitle":"on vacation","noemojiDataEmpty":null,"noemojiText":"","fullText":"🎧 in a meeting updated 5m ago","lineText":"🌙 sleeping","childPosition":"absolute","childRight":"-3px","childBottom":"-3px","avatarPosition":"relative","childInsideAvatar":true}
```
Collision fix (mine styled, bare `.status` clean):
```
{"mineDisplay":"flex","bareStatusDisplay":"inline","bareStatusGap":"normal","hasScopedRule":true,"hasBareBaseRule":false}
```
Reading these out: registry **138→146** (+8 = 6 standalone `status#` + `entity-avatar#with-status` + its `status#with-status__status` child); `count===unique===146` → **0 orphans**. **`expired` registers with `{expired:true}` on a mounted, empty span** (`data-empty="true"` + `display:none` + empty text) — the always-mount design proven CDP-readable. `badge` `role=img`/`aria-label="🎯"`; `noemoji` `title="on vacation"` (tooltip E) with empty glyph (visible span, not data-empty); `full` = **"🎧 in a meeting updated 5m ago"** (deterministic relative time from render-relative timestamps); `line` = "🌙 sleeping". Avatar seam: corner child `position:absolute` `-3px/-3px` inside the `position:relative` avatar (`childInsideAvatar:true`); avatar getter **unchanged** → **0-regression** (avatars 17 = 9 + 7 item-children + 1 new; items 7). Collision fix: bare `.status` → `display:inline`, no bare base rule, only `.status[data-variant]` in the sheet. Screenshots `temp/status-verify.png` + `temp/status-verify-2.png` (badge 🎯; line "🌙 sleeping"; line-noemoji "on vacation"; full "🎧 in a meeting updated 5m ago"; AN avatar with a 🟢 bottom-right corner badge).

**Engineering judgment (surfaced, D-065).** (1) The avatar delegates expiry to the status child (renders `<Status>` whenever `status` present; the child self-empties) — no duplicated expiry logic. (2) Relative time + expiry are lazy (`Date.now()` at render, no reactive timer) — fits the "lazy" lock; sampler timestamps are render-relative so "5m ago"/expired are deterministic. (3) The naming collision was caught in verify (the `.combobox-list .status` selector surfaced in the cascade probe) and fixed by scoping, not by renaming closed components. (4) The isAi/status bottom-right corner overlap is a known PROVISIONAL-skin item, flagged not silently moved.

**Records (atomic, D-074).** New: `ui/core/lib/components/data-dependent/status.svelte`; edited `entity-avatar.svelte` (`status?` prop + `cid` + corner child + comments), `ui/assets/skin.css` (`.status` block, scoped `.status[data-variant]`), `ui/sampler/src/app_sampler.svelte` (status cells + avatar-with-status). Docs: `ui/docs/xgen-ui-notes.md` N-077 (v0.61), `ui/docs/xgen-ui-components.md` registry v0.49 (status row + avatar `+status?` + build note), `docs/ROADMAP.md` v4.33 (tree tail + M-RP5.1a/5.1b block), `docs/xgen-dd-status-phase0.md` (Status→COMPLETED v1.1), this PLAY (→ J-464), this entry, runbook → COMPLETED (DoD ticked). No `DECISIONS.md` touch (N-077 is a component note; the collision fix + mounted-empty are arc-local; D-069 bar not met).

**Next-active.** `spaces-panel` (dd-composite, composes `section` + `entity-item ×N`; owns roving focus) — now inherits self-status via the avatar corner-slot → `entity-context-menu` widget (consumes `onActivate?`; uses `status full`) → `temperature-indicator` widget (W-11 dd-socket). Track A (J-461) gates the status-bearing avatar variants (M-RP5.2). Not pushed — Joe pushes.

---

## Entry J-463 — M-RP5.1 CLOSED: `entity-item` — the FIRST dd-composite (renamed from `container-list-item`); single-knob variant derive; the global width rule (N-076); `entity-avatar` `labeled`/`card` amendment; sampler DD·composite panel populated

**What happened.** Built `entity-item` per `tasks/RUNBOOK_ENTITY_ITEM.md` (design Joe-locked A–G + width in the Phase-0 walk, `docs/xgen-dd-entity-item-phase0.md` v1.1). The **first data-dependent composite** and the second `data-dependent/` occupant — the dd track's second rung. Clair (impl seat); this arc's handoff explicitly authorized the full atomic records close (D-074). One blocker surfaced at go-request and was Joe-resolved before any writes: the locked derive-map targets avatar variants `labeled`/`card` the shipped `entity-avatar` (M-RP5.0) didn't have — Joe chose **Option A: extend the avatar** (the seam M-RP5.0 pre-announced), tightly scoped, recorded as an additive M-RP5.0 amendment in this same commit.

**What it is.** A dd-**composite** that materializes ONE address-book entry (identity ∪ space ∪ DM) as a **full display unit** — avatar + name + optional secondary line + trailing meta/status — one tier up from `entity-avatar` (the dd-atomic glyph). One composite, **purpose selected by `variant`**: `row` (list entry) · `card` (prominent) · `nav` (sidebar) · `inline` (mention/presence). A genuinely new entity-display need → a **new variant**, not a new component (D-069 bar). Root = **`<div class="entity-item">`** per the N-075 dd-root rule (honest HTML; class×arity from folder + panel + getter). Source-agnostic: consumes the same `EntityDescriptor` seam as the avatar; `core` imports **no** `IdentityRecord`/`SpaceState`.

**Single-knob derive (the composite finding).** The consumer sets **one** `entity-item` `variant`; the composite **derives** the inner `entity-avatar` variant internally (`row→list · card→card · nav→labeled · inline→presence`) — the two variant axes never fight. The slot surface is **derived per variant** (not free props): `row` = name + meta · `card` = name + secondary + status · `nav` = name · `inline` = name. Secondary/status/meta are **caller-supplied slots** (the shell maps protocol + Track A `state.status` → these strings); the composite owns layout, not protocol reads. `onActivate?` + `selected?` are on the root; the list/panel (M-RP5.2) owns roving focus, not the item. Composes the **real** `entity-avatar` child (self-registers `<id>__avatar`, the status-indicator composite precedent — the matrix multiplies). Getter `{ variant, kind, name, hasSecondary, hasStatus, selected }` (`hasSecondary`/`hasStatus` = render truth).

**The global width rule (N-076).** A width-bearing component: **no `width` → 100%** (fills container); **`width` set → that value** (inline style, wins by specificity); **`min-width` = the intrinsic composition floor**. Per-variant floor exceptions allowed + marked (`inline` shrinks to content). Promotes the `meter`/`section` `width?` precedent to a default contract, retro-referenced by both (no code change — they already ship the shape).

**`entity-avatar` amendment (additive, pre-announced at M-RP5.0).** `variant` union widened `'presence' | 'list'` → `+ 'labeled' | 'card'`; they render initials like `list` at ascending glyph sizes (list 28 / labeled 32 / card 40); the name text stays in `entity-item`'s column (`<figcaption>` stays reserved-unused). Not a D-065 retrofit — the seam was named. `presence`/`list` unchanged (0-regression re-verified live).

**Verify — real CDP output (Rule 2), sampler DD·composite panel, port 9422.** `vite build` clean (152 modules; zero warnings on `entity-item`, the pre-existing `entity-avatar` `<figure>` a11y note untouched). Per-cell getters + child avatars (registry excerpt):
```
"entity-avatar#row-identity__avatar":{"kind":"identity","variant":"list","name":"Alice Ng","initials":"AN","seed":"hsl(0 45% 82%)","flags":{}}
"entity-item#row-identity":{"variant":"row","kind":"identity","name":"Alice Ng","hasSecondary":false,"hasStatus":false,"selected":false}
"entity-avatar#card-space__avatar":{"kind":"space","variant":"card","name":"Dev Team","initials":"DT",...}
"entity-item#card-space":{"variant":"card","kind":"space","name":"Dev Team","hasSecondary":true,"hasStatus":true,"selected":false}
"entity-avatar#nav-dm__avatar":{"kind":"space","variant":"labeled","name":"Bob Lee","initials":"BL","flags":{"isDm":true}}
"entity-item#nav-dm":{"variant":"nav","kind":"space","name":"Bob Lee","hasSecondary":false,"hasStatus":false,"selected":false}
"entity-avatar#inline-identity__avatar":{"kind":"identity","variant":"presence",...}
"entity-item#inline-identity":{"variant":"inline","kind":"identity","name":"Alice Ng","hasSecondary":false,"hasStatus":false,"selected":false}
"entity-item#card-plain":{"variant":"card","kind":"space","name":"Dev Team","hasSecondary":false,"hasStatus":false,"selected":false}
"entity-item#selected":{"variant":"row","kind":"identity","name":"Alice Ng","hasSecondary":false,"hasStatus":false,"selected":true}
```
Registry delta + width rule + child registration:
```
{"count":138,"unique":138,"eiCount":7,"avChild":7,"fixedTag":"DIV","fixedClass":"entity-item","fixedWidth":"280px","fixedMinWidth":"180px","rowWidth":"180px","rowMinWidth":"180px","inlineDisplay":"flex","inlineWidth":"158.212px","selectedAttr":"true","cardHasAvatarChild":true,"hasEntityItemRule":true,"hasSelectedRule":true}
```
Set-vs-unset width + inline floor exception:
```
{"inlineVariantAttr":"inline","inlineDisplay":"flex","inlineWidth":"158.212px","inlineMinWidth":"0px","rowInlineStyle":"(none)","fixedInlineStyle":"width: 280px;"}
```
Cascade + per-variant density:
```
{"entityItemSelectors":[".entity-item",".entity-item .ei-body",".entity-item .ei-name",".entity-item .ei-secondary",".entity-item .ei-meta",".entity-item .ei-status",".entity-item .ei-status-emoji",".entity-item .ei-status-text",".entity-item[data-variant=\"card\"]",".entity-item[data-variant=\"nav\"]",".entity-item[data-variant=\"inline\"]",".entity-item[data-variant=\"inline\"] .ei-name",".entity-item:hover",".entity-item[data-selected]"],"cardBorderStyle":"solid","cardBg":"rgb(28, 31, 36)","navPaddingLeft":"4px"}
```
Reading these out: registry **124→138** (+14 = 7 composites + 7 self-registered `__avatar` children); `count===unique===138` → **0 orphans**. **Derive-map literal** — the inner avatars are `list`/`card`/`labeled`/`presence` per the composite's `row`/`card`/`nav`/`inline` (the two new presets `card`/`labeled` exercised). **Slots per C** — `card-space` `hasSecondary:true,hasStatus:true`; `row`/`nav`/`inline` false; `card-plain` (absent-secondary edge) false/false, proving render-truth. **`selected`** `data-selected="true"`. **Width (N-076):** `row` no inline style → used width clamped to the `min-width:180px` floor; `fixed` inline `width:280px`; `inline` `min-width:0` + content width `158px` (the per-variant floor exception). Root `DIV.entity-item`; **14** `.entity-item*` rules in cascade (per-variant density + `:hover` + `[data-selected]`); card density `border:solid` + bg `rgb(28,31,36)`=`--s2`; nav `padding-left:4px`=`--sp-1`. (`inline` computed `display:flex` not `inline-flex` — correct: the `.s-cell` parent is a flex container, so the item's `inline-flex` is blockified per CSS Display; the `min-width:0` confirms the inline rule applied.) Screenshot `temp/entity-item-verify.png` (eye-checked: row AN circle + `3`; card DT rounded-square + secondary + 🟢 online; nav BL circle; inline presence dot + small name; selected gold `--accent` left bar; card-plain name-only; fixed 280px row).

**Engineering judgment (surfaced, D-065).** (1) `onActivate` is on the composite root only, NOT forwarded to the child avatar — one activate target, no double-fire; the avatar's own `onActivate?` stays unused here. (2) `hasSecondary`/`hasStatus` report render-truth (slot-applicable AND value present), so the getter reads exactly what the DOM shows — hence `card-plain` reports false/false. (3) The avatar amendment gives `labeled`/`card` size-only presets (no `<figcaption>`), since the name lives in the composite; the two new variants differ from `list` in glyph size alone. (4) CDP-tooling finding recorded: `cdp-debug.ps1 -Expression` via a `Get-Content -Raw` file avoids the `\"`-vs-`''` quoting trap (escaped `\"` fails; doubled `''` or a file works).

**Records (atomic, D-074).** New: `ui/core/lib/components/data-dependent/entity-item.svelte`; edited `ui/core/lib/components/data-dependent/entity-avatar.svelte` (variant union + initials-render + comments), `ui/assets/skin.css` (`.entity-avatar` labeled/card presets + `.entity-item` block), `ui/sampler/src/app_sampler.svelte` (DD·composite panel). Docs: `ui/docs/xgen-ui-notes.md` N-076 (v0.60), `ui/docs/xgen-ui-components.md` registry v0.48 (entity-item row + avatar variant note + build note; `container-list-item`⬛SUPERSEDED; `spaces-panel` composed-of), `docs/ROADMAP.md` v4.32 (tree tail + M-RP5.1 ✅ DONE block), `docs/xgen-dd-entity-item-phase0.md` (Status→COMPLETED v1.2), this PLAY (→ J-463), this entry, runbook → COMPLETED (DoD ticked). No `DECISIONS.md` touch (N-076 is a component contract/note, arc-local; D-069 bar not met).

**Next-active.** `spaces-panel` (dd-composite, composes `section` + `entity-item ×N`; owns roving focus) → `entity-context-menu` widget (consumes `onActivate?`, the 100% read) → `temperature-indicator` widget (W-11 dd-socket). Track A (J-461) gates the status-bearing avatar variants (M-RP5.2). Kind 4 `use:render` stays deferred (D-065). Not pushed — Joe pushes.

---

## Entry J-462 — M-RP5.0 CLOSED: `entity-avatar` — the FIRST data-dependent component (dd-atomic); the dd-root rule (N-075); shared `seedColour` base helper factored from `chip`; sampler DD·atomic panel populated

**What happened.** Built `entity-avatar` per `tasks/RUNBOOK_ENTITY_AVATAR.md` (design Joe-locked A–H in the J-461 design walk; Phase-0 `docs/xgen-dd-entity-avatar-phase0.md`). The **first data-dependent component** and first `data-dependent/` occupant — the dd track opens for real. Clair (impl seat); this arc's handoff explicitly authorized the atomic records close (D-074).

**What it is.** A dd materializes ONE address-book entry (an identity or a space — the D-071 Phase-0 audit of `IdentityRecord`/`SpaceState`) into a visual; the rendered **shape branches on the data** (the dd ≠ di line): identity + DM space = circle (people-shaped), non-DM space = rounded-square. Consumes a **source-agnostic `EntityDescriptor { kind, name?, id, flags{isAi?,revoked?,isDm?,e2e?}, image? }`** — the W-11 dd-socket payload — NOT the raw protocol type; `core` imports **no** `IdentityRecord`/`SpaceState` (the shell owns the protocol → descriptor map; `image`/`e2e` reserved-unfed, D-065).

**The dd-root rule (N-075, the reusable finding).** dd does NOT inherit the di `<div>`=composite litmus. A dd root is **honest HTML for the materialized thing**; class×arity reads from the folder + sampler panel + getter. `entity-avatar` root = **`<figure class="entity-avatar" role="img">`**, `aria-label={name ?? kind}`, `<figcaption>` reserved (the `labeled`/`card` seam, M-RP5.1) — this corrects the Phase-0 §5-B `<div>` recommendation.

**Mechanics built.** **E** colour = `seedColour(name ?? id)` — the hash + muted band **factored out of `chip`** into a shared base helper `ui/common/lib/components/base/seed-colour.ts` (`{hue,bg,fg,bd}`, byte-identical strings; `chip.svelte` now imports it; no `--accent` dependency). **initials** = 1–2 graphemes of `name` via `Intl.Segmenter` (grapheme-safe), absent name → xgid-tail fallback (last 2 alphanumerics, uppercased). **F** `variant` primary axis (purpose): `presence` (xs, glyph) / `list` (sm, + initials) — size/content derived presets. **D** badges self-drawn (not a nested `led`): `isAi` `::after` fixed-violet spark disc, `revoked` `grayscale(1)`+dim + `::before` diagonal slash. **H** `onActivate?` reserved (menu seam, wired to `onclick`, no menu). **G** getter `{ kind, variant, name, initials, seed, flags }`.

**Verify — real CDP output (Rule 2), sampler DD·atomic panel, port 9422.** `vite build` clean (151 modules). Registry + presence:
```
{"href":"http://localhost:5175/","total":124,"eaCount":9,"ea":["entity-avatar#identity-presence","entity-avatar#identity-list","entity-avatar#space-presence","entity-avatar#space-list","entity-avatar#dm-presence","entity-avatar#dm-list","entity-avatar#absent","entity-avatar#revoked","entity-avatar#ai"]}
```
Per-cell getter + DOM (excerpt — full 9 cells verified):
```
"identity-list":{"g":{"kind":"identity","variant":"list","name":"Alice Ng","initials":"AN","seed":"hsl(0 45% 82%)","flags":{}},"tag":"FIGURE","role":"img","shape":"circle","ai":false,"rev":false,"aria":"Alice Ng"}
"space-list":{"g":{"kind":"space",...,"initials":"DT",...},"shape":"square",...}
"dm-list":{"g":{...,"name":"Bob Lee","initials":"BL","flags":{"isDm":true}},"shape":"circle",...}
"absent":{"g":{"kind":"identity","variant":"list","name":null,"initials":"AZ","seed":"hsl(318 45% 82%)","flags":{}},"shape":"circle",...,"aria":"identity"}
"revoked":{"g":{...,"flags":{"revoked":true}},"rev":true,...}
"ai":{"g":{...,"flags":{"isAi":true}},"ai":true,...}
```
Shell-independence + cascade + pseudos + orphans:
```
{"clientSeed":{"seed":"hsl(0 45% 82%)","bg":"rgb(230, 188, 188)","rad":"50%"},"nodeSeed":{"seed":"hsl(0 45% 82%)","bg":"rgb(230, 188, 188)","rad":"50%"},"seedMatch":true,"squareRad":"10px","eaRuleCount":8,"orphanCount":0,"aiBadge":{"content":"\"\"","bg":"rgb(109, 92, 231)"},"revoked":{"beforeContent":"\"\"","filter":"grayscale(1)","opacity":"0.55"}}
```
Chip 0-regression (Step-1 DoD — per-label seed unchanged, byte-identical band):
```
"chip#default":{"styleBg":"hsl(307 45% 82%)","styleFg":"hsl(307 55% 30%)","styleBd":"hsl(307 40% 80%)",...}
"chip#static":{"styleBg":"hsl(216 45% 82%)",...}  "chip#long":{"styleBg":"hsl(60 45% 82%)",...}   chipCount:3
```
Reading these out: registry **115→124** (+9 cells), all roots `FIGURE`/`role=img`; shape-per-kind (identity/DM circle, non-DM space square `10px`); `absent` → `name:null`/`initials:"AZ"` xgid-tail fallback + `aria-label="identity"`; **seed shell-independence** — `identity-list` bg `rgb(230,188,188)` **identical client↔node** (`seedMatch:true`), no accent swap; isAi `::after` drawn violet `rgb(109,92,231)`; revoked `::before` slash + `grayscale(1)`/`0.55`; **8** `.entity-avatar*` rules in cascade; **0 orphans**; chip 3 cells intact with distinct per-label fills. Screenshot `temp/entity-avatar-verify.png` (eye-checked: circles/rounded-square, AN/DT/BL initials, AZ fallback, MA greyed+slashed, AB with spark badge).

**Engineering judgment (surfaced, D-065).** (1) `seedColour` returns `{hue,bg,fg,bd}` (chip's full triple) so both consumers share one source; chip's output is byte-identical (re-verified). (2) getter `seed` = the fill `bg` hsl string (directly comparable → the shell-independence probe). (3) isAi badge colour is a **fixed** violet literal, deliberately NOT `var(--accent)`, to preserve the whole-avatar shell-independence. (4) `onActivate?` wired to `onclick` with a scoped `svelte-ignore` for the role=img a11y warning (reserve-don't-build). (5) `<figcaption>` reserved as an in-file comment seam (no empty element rendered).

**Records (atomic, D-074).** New: `ui/common/lib/components/base/seed-colour.ts`, `ui/core/lib/components/data-dependent/{types.ts,entity-avatar.svelte}`; edited `chip.svelte` (import helper), `ui/assets/skin.css` (`.entity-avatar` block), `ui/sampler/src/app_sampler.svelte` (DD·atomic panel). Docs: `ui/docs/xgen-ui-notes.md` N-075 (v0.59), `ui/docs/xgen-ui-components.md` registry v0.47 (entity-avatar row + dd-seed BUILT + build note), `docs/ROADMAP.md` v4.31 (tree tail + M-RP5.0 ✅ DONE block), `docs/xgen-dd-entity-avatar-phase0.md` (A–H LOCKED, Status→COMPLETED v1.1), this PLAY (→ J-462), this entry, runbook + handoff → COMPLETED. No `DECISIONS.md` touch (N-075 is registry/note, arc-local; D-069 bar not met).

**Next-active.** `container-list-item` (dd-composite, composes `entity-avatar`; unlocks `labeled`/`card`) → `spaces-panel` → `entity-context-menu` widget (consumes `onActivate?`) → `temperature-indicator` widget (W-11 dd-socket). Track A (J-461) gates the status-bearing avatar variants (M-RP5.2). Not pushed — Joe pushes.

---

## Entry J-461 — PROTO-STATUS.2 CLOSED: self-set status reference impl (`xgen-core/src/status/`) — type + validation + resolution wiring + 19 tests, workspace green

**What happened.** Built the self-set status reference impl per `tasks/RUNBOOK_PROTO_STATUS_2.md` (PROTO-STATUS.0/.1 locked design). New `xgen-core/src/status/mod.rs`: the `StatusRecord` type + validating constructor + `is_expired`, plus the resolution wiring (`status_state_key` + `StatusStore`) and 19 unit tests. Track A (protocol), Clair (impl seat).

**Grounding (D-078 — grep symbol defs, not inference).** Confirmed against the tree before coding: there is no `Timestamp` newtype (timestamps are `String` RFC-3339 on the wire, e.g. `Event.timestamp`, `IdentityRecord.registered_at`); `update_version: u64` is the monotonic per-object counter pattern (`IdentityRecord`, spec §3.6.8); `IdentityXgid` is `#[serde(transparent)]` with `from_xgid`/`as_str`; the `state.*` machinery (`resolution::state_key::state_key_for_event` → `StateKey{category,key_field}` → `resolve`/`derive_resolved`) builds a **`SpaceState`** from a Space's DAG.

**Scope call (surfaced, D-065).** The locked design fixes status as **identity-scoped and global — explicitly NOT under `space/`** (PROTO-STATUS.0 §2). That makes it structurally unable to ride the per-Space DAG resolution (`derive_resolved` yields `SpaceState`; status is not Space state). So "register `state.status/<identity_xgid>` under existing `state.*` machinery" is read as **reuse the conventions**, not thread a new `EventType` through wire/validation/app — which would be a wire-format change (needs its own Joe-lock) and is not enumerated (every runbook test is unit-level on the type/store). Built a self-contained module: `status_state_key(id) -> StateKey{"state.status", <xgid>}` (reuses the `StateKey` namespace) + a `StatusStore` carrying the per-object `update_version`, owner-write guard, clear-by-delete, and lazy-expiry read.

**Type (`StatusRecord`).** `emoji: Option<String>` / `text: Option<String>` / `updated_at: DateTime<Utc>` / `expires_at: Option<DateTime<Utc>>`; absent optionals `skip_serializing_if` (no `null` on the wire). `new(emoji, text, expires_at, now)` stamps `updated_at = now` and validates: emoji = exactly one grapheme cluster (via `unicode-segmentation` — ZWJ/skin-tone are one cluster, `chars().count()` would be wrong); text trimmed, whitespace-only → absent, >128 bytes → reject; `expires_at ∈ [now+60s, now+30d]` inclusive → else reject. `is_expired(now)` = `expires_at` set and strictly < `now`.

**Verify (real output — Rule 2, from `C:/cargo-targets/XGenProtocol`).**
```
cargo test -p xgen-core status::
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 742 filtered out; finished in 0.00s
```
Full workspace, no regressions:
```
cargo test --workspace   →   TOTAL passed=1502 failed=0 ignored=62
```
`cargo clippy -p xgen-core` — 0 warnings/errors (one `map_or(false,…)` → `is_some_and` suggestion applied on `is_expired`). The 19 tests cover the runbook enumeration exactly: emoji 1-grapheme accept (incl. ZWJ family + skin-tone) / 2+ / empty reject; text 128B accept / 129B reject / whitespace→absent / trimmed-on-store; `expires_at` now+60s & now+30d accept, now+59s & now+31d reject; `is_expired` none-never / strict-precede; `status_state_key` category+key; owner-write-only (set + clear reject non-owner); clear=delete + read-absent; lazy-expiry read-absent-without-sweep (len/version intact); monotonic `update_version`; serde omits absent optionals / no null.

**Engineering judgment calls (surfaced for Joe's review before push, D-065).** (1) Scope read above — self-contained module, no new `EventType`/wire change. (2) Promoted `unicode-segmentation` (already compiled in-workspace transitively, v1.13.2) to a **direct** `xgen-core` dep — the grapheme cap needs it. (3) Typed timestamps as `chrono::DateTime<Utc>` (serialises RFC-3339, matches Appendix I "string") rather than raw `String`, for type-safe bounds/expiry — self-contained record, not wire-canonical-hashed. (4) Whitespace-only text → **absent** (per spec §3 + test enumeration "whitespace-only → absent"), not a hard reject (the runbook Type-surface's looser "rejects empty-after-trim" wording; spec + enumeration win).

**Records.** feat commit: `xgen-core/src/status/mod.rs` + `lib.rs` (`pub mod status;`) + `Cargo.toml` (`unicode-segmentation`) + `Cargo.lock`. docs commit: Appendix I §V.4 `StatusRecord` (v1.7→1.8), this entry, runbook → COMPLETED (v1.1). No `DECISIONS.md` touch (arc-local, D-069).

**Deferred + surfaced (D-065, not papered over).** The runbook's close names "ROADMAP PROTO-STATUS.2→DONE" and "CLAUDE.md PLAY", but the prior Track-A commits (PROTO-STATUS.0/.1/.2, `c8c1444`/`f6bc8fc`/`12f5269`/`9a43d7b`) were each **deliberately single-file, doc-only** — neither ROADMAP nor the CLAUDE PLAY block has any PROTO-STATUS / Track-A scaffolding to flip (the PLAY block is entirely RP/UI track, head J-460). Introducing that scaffolding into two session-critical, Chat-owned canonical records is a design-of-record call I did not make unilaterally. Left for Joe/Chat to decide placement; flagged rather than invented. (No false "PLAY" state exists to correct, so the ROADMAP same-commit discipline is not violated by the absence.)

**Next.** PROTO-STATUS.2 gates the status-bearing `entity-avatar` variants (Track B, M-RP5.2).

---

## Entry J-460 — M-RP2.31a CLOSED: `section` `width?` — settable width (additive, meter mechanism) — built, sampler-verified, records closed

**What happened.** Additive amendment to `section` (M-RP2.31) — an optional **`width?`** prop, the `meter` width contract (M-RP2.30). No breaking change.

**Change.** `section.svelte`: `width?: string` → inline `width` on root `<section>`; unset → 100% (fills container). `skin.css` `.section`: `width:100%` + `min-width:160px` (a titled box needs more than meter's 80px) + `box-sizing:border-box`. Getter gains `width` → `{title, badge, collapsible, collapsed, width}` (meter precedent). Sampler: `section#fixed` (`width="320px"`).

**CDP verify** (sampler 9422, real output — Rule 2). `vite build` clean.
```
{"total":115,"fixedGetter":{"title":"Fixed width","collapsible":false,"collapsed":false,"width":"320px"},"plainGetter":{"title":"Spaces","collapsible":false,"collapsed":false},"fixedWidth":"320px","fixedMinWidth":"160px","plainMinWidth":"160px"}
```
`#fixed` getter carries `width:"320px"` + computed `width:320px`; `#plain` has no width key (unset) + `min-width:160px`. Registry **114→115**. 0 orphans.

**Records (atomic, D-074).** N-074 (ui-notes v0.58) + registry v0.46 (section getter gains `width`) + ROADMAP v4.29 (tree tail + M-RP2.31a ✅ DONE) + CLAUDE PLAY (→ J-460) + this entry + runbook → COMPLETED. No DECISIONS touch. **Next-active:** the **dd track opens for real** — Phase-0 on `entity-avatar` (first domain-bound dd; D-071 audit of IdentityRecord/SpaceState + Appendix I) → `container-list-item` → `spaces-panel` (composes `section` + rows) → `temperature-indicator` widget. Kind 4 `use:render` stays deferred (D-065).

---

## Entry J-459 — M-RP2.31 CLOSED: `section` — collapsible disclosure container (di, atomic-ish, root `<section>`); supersedes the seed `section-header` — built, sampler-verified, records closed

**What happened.** Built + closed `section` — the **27th `core`**, root native `<section>`: a collapsible **disclosure container** (optional header over a body slot). **di, binding none; atomic-ish** (self-contained, composes no registering child components — one getter per instance; a nested section is its own instance → its own id). Does NOT open dd.

**Design walk (Joe).** Started from the seed `section-header` (a bare divider). The collapse requirement exposed that a divider must reach out and hide *sibling* rows — awkward. Joe steered to a **container** that owns its own body (clean self-collapse). Reclassified: a binding-none container that wraps children is di (the status-indicator precedent), NOT dd — so `section` does not open the dd track, and it **supersedes** the seed `section-header` (⬛ DEPRECATED). Locks: atomic on `<section>`; `<div>`-body (the honest neutral container — `<article>`/`<p>` carry wrong meaning, `<p>` can't hold block children incl. nested sections); **solid header band** (not a rule line — future bg-colour/picture widget-mods target a filled surface); chevron **reuses combobox's `--tri`/`--tri-open` masked glyphs**; badge is a programmatic string ("2/5" unread/total), not user text. Filter/search explicitly OUT (a data-aware panel/widget concern feeding `badge` + hiding rows — deferred).

**Component (`section.svelte`).** Skeleton `<section class="section">` → `<h{level} class="section-header">title <span class="section-badge">2/5</span></h{level}>` + `<div class="section-body">{@render children}</div>`. Props `title?`/`badge?`/`collapsible?` (def false)/`collapsed?` ($bindable)/`level?` (def 2)/`id`/`children`. Collapsible → header in `<button aria-expanded>` + `.chev`; body hidden via `[data-collapsed]` `display:none` (NEVER `{#if}` — slot stays mounted). Getter `{title, badge, collapsible, collapsed}`. Accent-neutral.

**CDP verify** (sampler 9422, real output — Rule 2). `vite build` clean. Registry + presence:
```
{"total":114,"sec":["section#plain","section#badged","section#collapsible","section#bare","section#nested-inner","section#nested"]}
```
Structure + getters:
```
{"plainGetter":{"title":"Spaces","collapsible":false,"collapsed":false},"badgedGetter":{"title":"Direct messages","badge":"2/5","collapsible":false,"collapsed":false},"collapsibleGetter":{"title":"Rooms","collapsible":true,"collapsed":false},"bareGetter":{"collapsible":false,"collapsed":false},"tag":"SECTION","hasH2":true,"bareHasHeader":false,"badgeText":"2/5","nestedChildIsSection":true,"ariaExpanded":"true","bodyDisplay":"block","ruleCount":12}
```
Collapse (click toggle, settled DOM 2nd round-trip):
```
{"getter":{"title":"Rooms","collapsible":true,"collapsed":true},"dataCollapsed":"true","ariaExpanded":"false","bodyDisplay":"none","chevMask":"url(...tri-open...)"}
```
Registry **108→114** (5 cells + `#nested-inner`, nesting registers its own id). 12 `.section*` rules in cascade. Screenshot `temp/section-verify.png`. 0 orphans. *(Verify note: a stale zombie Vite on 5175 served the pre-edit module for several restarts — the fix was killing all node + confirming the served `/src/app_sampler.svelte` contained `section#plain` BEFORE probing the registry; a lesson for the run-sampler thrash.)*

**Records (atomic, D-074).** N-073 (ui-notes v0.57) + registry v0.45 (section row, 27th core; seed `section-header` → DEPRECATED/superseded) + ROADMAP v4.28 (RP node + tree + M-RP2.31 ✅ DONE) + CLAUDE PLAY (→ J-459) + this entry + runbook → COMPLETED. No DECISIONS touch. **Next-active:** the **dd track opens for real** — Phase-0 on `entity-avatar` (the first domain-bound dd; D-071 subsystem audit of IdentityRecord/SpaceState + Appendix I) → `container-list-item` (the object-backed row composing `entity-avatar`) → `spaces-panel` (composes `section` + rows) → `temperature-indicator` widget. Kind 4 `use:render` stays deferred (D-065).

---

## Entry J-458 — M-RP2.30a CLOSED: `meter` `fill?` — custom bar colour (additive), the led/chip inline-var mechanism — built, sampler-verified, records closed

**What happened.** Additive amendment to `meter` (M-RP2.30) — an optional **`fill?`** prop for a custom bar colour, Joe-lock **option A** (a set `fill` overrides the optimum/sub/over semantics entirely). The led/chip data-coloured-via-inline-var mechanism. No breaking change; unset callers unaffected. Runbook `tasks/M_RP2_30a_METER_FILL.md` (now COMPLETED).

**Change.** `meter.svelte`: `fill?: string` prop (hex or `var(--token)`) → inline `--meter-fill` var, composed with `width?` into one `style` string; getter → `{value,min,max,optimum,fill}`. `skin.css`: the three value-pseudos become `background: var(--meter-fill, var(--ok|--warn|--err))` — set `fill` wins, unset falls back to the semantic fill. Bonus: closes the no-optimum-reads-green gap (`fill="var(--t3)"` = neutral bar). Sampler: `meter#custom` (`fill="var(--accent2)"`) + `meter#neutral2` (`fill="var(--t3)"`).

**CDP verify** (sampler 9422, real output). `vite build` clean.
```
{"total":108,"customGetter":{"value":70,"min":0,"max":100,"optimum":20,"fill":"var(--accent2)"},"neutral2Getter":{"value":50,"min":0,"max":100,"fill":"var(--t3)"},"customVar":"var(--accent2)","pseudoRules":[".meter::-webkit-meter-bar => var(--s5)",".meter::-webkit-meter-optimum-value => var(--meter-fill, var(--ok))",".meter::-webkit-meter-suboptimum-value => var(--meter-fill, var(--warn))",".meter::-webkit-meter-even-less-good-value => var(--meter-fill, var(--err))"]}
```
Getters carry `fill`; the inline `--meter-fill` var is present on `#custom`; all three value-pseudos now read `var(--meter-fill, <semantic>)`. Registry **106→108** (+2 cells). 0 orphans.

**Records (atomic, D-074).** N-072 (ui-notes v0.56) + registry v0.44 (meter getter gains `fill`) + ROADMAP v4.27 (tree tail + M-RP2.30a ✅ DONE) + CLAUDE PLAY (→ J-458) + this entry + runbook → COMPLETED. No DECISIONS touch. **Next-active:** the **dd track opens** — Phase-0 on `section-header` (ungrounded warm-up) → `entity-avatar` (first domain-bound, D-071 audit on IdentityRecord/Appendix I). `temperature-indicator` later consumes meter (fill + semantic) via the widget dd-socket. Kind 4 `use:render` stays deferred (D-065).

---

## Entry J-457 — M-RP2.30 CLOSED: `meter` — the 26th `core` / 5th simple display-di, root `<meter>`, the read-only sibling of `range`; founds the `--warn` L2 token — built, sampler-verified, records closed

**What happened.** Built + closed `meter` — the **26th `core`** and the **5th simple display-di** (after label/paragraph/image/led), root native `<meter>`. The read-only sibling of `range` (same `{value,min,max}` shape, opposite direction: range = editable numeric-in via `bind:value`; meter = read-only value-against-range-out). **di, NOT dd** — during Phase-0 I first framed meter as the first dd-atomic; Joe challenged it and I corrected: a bare value+range is a display primitive, not a domain materialization (dd = IdentityRecord/SpaceState/etc.). So meter stays on the di display family and does NOT open the dd track. Design Joe-locked (surface / prop set / width / semantic-fill+token / skin / classification); runbook `tasks/M_RP2_30_METER.md` (now COMPLETED).

**Component (`meter.svelte`).** Root `<meter>`; `value` plain prop (read-only display-di rule, not `$bindable`); `min`(0)/`max`(1)/`optimum?`/`low?`/`high?`/`width?`/`disabled?`/`id`/`name`. `low`/`high`/`optimum` drive the native semantic fill. Getter `{value,min,max,optimum}`. No caption/readout (the consuming composite/widget adds it — the range/number rule). Width (Joe-lock): **full-width by default** (`.meter` `display:block`+`width:100%`; JavaFX HGrow is native here), optional `width?` pins a fixed width via inline `style`, `min-width:80px` floor.

**Skin + `--warn`.** Founds **`--warn: #ba7517`** in L2 `:root` (amber; XGen had `--ok`/`--err` but no caution colour; reused later for form-warning states). Pseudo-heavy `.meter` (PROVISIONAL, the `range` precedent): `::-webkit-meter-bar` track `--s5` + `::-webkit-meter-optimum-value` `--ok` / `-suboptimum-value` `--warn` / `-even-less-good-value` `--err`; `[aria-disabled]` dims. **Build finding (D-065):** with no `optimum` the UA paints the *optimum* pseudo, so a no-optimum bar reads green, NOT neutral grey — the design-mock grey isn't achievable via pure native pseudos without a per-instance hook (deferred; the `meter#neutral` cell shows it).

**CDP verify** (sampler 9422, real output — Rule 2). `vite build` clean. Registry + presence:
```
{"total":106,"meter":["meter#optimum","meter#caution","meter#danger","meter#neutral","meter#fixed","meter#disabled"]}
```
Getters + computed:
```
{"getters":{"optimum":{"value":35,"min":0,"max":100,"optimum":20},"caution":{"value":65,"min":0,"max":100,"optimum":20},"danger":{"value":94,"min":0,"max":100,"optimum":20},"neutral":{"value":50,"min":0,"max":100},"disabled":{"value":40,"min":0,"max":100}},"base":{"tag":"METER","display":"block","minWidth":"80px","widthPx":96},"fixed":{"width":"120px"},"disabled":{"opacity":"0.45","ariaDisabled":"true"}}
```
Skin rules in cascade + `--warn` token (N-042 stylesheet inspection — `getComputedStyle` returns UA defaults on the shadow-pseudos):
```
{"warnToken":"#ba7517","meterRules":[".meter => var(--s5)",".meter::-webkit-meter-bar => var(--s5)",".meter::-webkit-meter-optimum-value => var(--ok)",".meter::-webkit-meter-suboptimum-value => var(--warn)",".meter::-webkit-meter-even-less-good-value => var(--err)",".meter[aria-disabled=\"true\"]"]}
```
Screenshot at `temp/meter-verify.png`. Registry **100→106** (+6 cells). Accent-neutral (semantic fills, no `--accent`) → no skin-swap. 0 orphans.

**Records (atomic, D-074).** N-071 (ui-notes v0.55) + registry v0.43 (meter row) + ROADMAP v4.26 (RP node + tree + M-RP2.30 ✅ DONE) + CLAUDE PLAY (Entry head → J-457, next-active → dd track open) + this entry + runbook → COMPLETED. No DECISIONS touch (`--warn` recorded in N-071). **Next-active:** the **dd track opens** — `section-header` (ungrounded warm-up) → `entity-avatar` (first domain-bound, D-071 audit on IdentityRecord/Appendix I). `temperature-indicator` later consumes meter as its readout via the widget dd-socket. Kind 4 `use:render` stays deferred (D-065).

---

## Entry J-456 — M-RP4.5 CLOSED: kind-2 converter/bridge — `converter-field`, the one processor kind that is a component; `string ↔ T` via `Converter<T>`/`intlNumber` + a `<generics=T>` two-rep host — built, sampler-verified, records closed

**What happened.** Built + closed **kind 2** of the four-kind processor taxonomy (D-099/N-056): the **converter/bridge** (`string ↔ T`). Unlike kinds 1/3 (same-type in/out, forwarded attachments), kind 2 has TWO representations of DIFFERENT type coexisting — a formatted display string + a typed bound value — which one `bind:value` cannot carry, so it ships as a **real component**, the 25th `core` (di atomic `converter-field`). Pure layer only (no Rust, no effect layer), fully sampler-verifiable. Design Joe-locked Phase-0 (host / config+parse / timing / provenance / getter); runbook `tasks/M_RP4_5_CONVERTER_FIELD.md` (now COMPLETED). Three of four processor kinds now built (1 transformer, 3 filter/guard, 2 converter); only kind 4 (`use:render`) remains codified-not-built.

**Pure core (`transform.ts`, additive, DOM-free).** `PARSE_FAILED = Symbol()` (unique sentinel — `null`/`NaN` stay legitimate `T`); `Converter<T> {toString(v):string; fromString(s): T | PARSE_FAILED; toEditable?(v):string}`; first concrete `intlNumber(opts?, locale?)` over `Intl.NumberFormat` — `toString`=format, `toEditable`=raw `String(v)`, `fromString` discovers the locale's group+decimal glyphs via `formatToParts(11111.1)` (Intl has no parser), strips group, normalises decimal to `.`, `Number()`+finite-check. Still the `logic.ts` posture (Intl = ECMA-402, not DOM; no `window`). The `ProcessorRule` union stays codified-not-declared (D-065).

**Host (`converter-field.svelte`, new di atomic).** `<script lang="ts" generics="T">`, root `<input type="text">`. Two-rep state: `value` ($bindable, typed OUT) + internal `text` ($state, never a `$derived` of value — that clobbers typing) + `invalid`. An `$effect` reformats `text` from `value` on external change **only while unfocused**. Timing (Joe-lock): `focus`→raw `toEditable`; `change`/`blur`→`commit()`; **nothing on `input`** (decoupled → no caret-restore). `commit()`: empty = no-op revert (never "invalid empty"); `PARSE_FAILED` = reject-and-mark (`[data-invalid]`, value held); success = set value + reformat. The component is kind 2's sole framework touch, so the DEV `__XGEN_CONVERT__` hook lives here (not `transform.ts`). Getter `{value,text,valid}`. Skin `.converter-field` assembled from the `.number` vocabulary; parse-fail look keys off `[data-invalid="true"]` (not native `:invalid` — the field holds a free string until commit; the `.led [data-pulse]` attribute-hook precedent). **Provenance = Tier-1 only** (a converter is code, not a user string — no caps/lint).

**Sampler.** New DI·atomic Interactive row; stable converter identity (`const numConv = intlNumber({maximumFractionDigits:2})`, defined once, not inline); cells `#default` (seed 1234.5) + `#disabled` (99.9).

**CDP verify** (sampler 9422, both accents, real output — Rule 2). `npm run build` clean:
```
✓ 147 modules transformed.
✓ built in 711ms
```
(146→147; the `generics="T"` component compiled). Registry + presence:
```
{"total":100,"converter":["converter-field#default","converter-field#disabled"]}
```
Pure core via `__XGEN_CONVERT__`:
```
{"fmt":"1,234,567.5","edit":"1234567.5","parseGrouped":2000000.5,"parsePlain":42.25,"parseBad":"PARSE_FAILED","parseEmpty":"PARSE_FAILED","roundtrip":1234.5}
```
Live `#default` (drive input+change per commit):
```
initial : {"value":1234.5,"text":"1,234.5","valid":true}
afterBad("abc")     : {"value":1234.5,"text":"abc","valid":false}
afterValid("2000000.5"): {"value":2000000.5,"text":"2,000,000.5","valid":true}
afterEmpty("")      : {"value":2000000.5,"text":"2,000,000.5","valid":true}
```
Settled DOM after the bad drive (second round-trip, post-flush):
```
{"dataInvalid":"true","borderColor":"rgb(138, 42, 42)","getter":{"value":2000000.5,"text":"abc","valid":false}}
```
`rgb(138,42,42)` = `#8a2a2a` = `--err` (the `.converter-field[data-invalid="true"]` skin rule applies). Disabled cell + base skin:
```
{"disabled":{"get":{"value":99.9,"text":"99.9","valid":true},"isDisabled":true},"defaultBase":{"tag":"INPUT","type":"text","minHeight":"28px","fontSize":"12px"},"restored":{"value":2000000.5,"text":"2,000,000.5","valid":true}}
```
Registry **98→100** (+2 cells). Field left clean (empty-commit reverted to the current value). **0 orphans.** *(Method note: the getter reads live `$state` synchronously, but the DOM `data-invalid` attribute + `bind:value` display reformat flush a microtask later — the settled attribute is read on a second CDP round-trip, not inline with the drive. This is why the first combined drive showed `dataInvalid:null` while the getter already read `valid:false`.)*

**Records (atomic, D-074).** D-099 amendment (kind 2 built) + N-070 (ui-notes v0.54) + registry v0.42 (converter-field row + processor-kinds banner 3-of-4) + ROADMAP v4.25 (RP node + tree + M-RP4.5 ✅ DONE) + CLAUDE PLAY (Entry head → J-456, next-active → kind 4/dd) + this entry + runbook → COMPLETED. **Next-active:** kind 4 (`use:render`, deferred) → dd-components (unblocks `temperature-indicator` + the widget registry/dynamic-mount layer).

---

## Entry J-455 — M-RP4.1 CLOSED: kind-3 filter/guard — `number` min/max clamp; the 2nd of four processor kinds built (`ClampRule`+`applyClamp` + change-triggered `clamp.ts` attachment + `number` clamp-host) — built, sampler-verified, records closed

**What happened.** Built + closed **kind 3** of the four-kind processor taxonomy (D-099/N-056): the **filter/guard** (`T → T`, idempotent). First consumer = `number` min/max **clamp** on commit (`change`). Pure layer only — no Rust, no effect layer, fully sampler-verifiable (no D-097 blind spot). Design Joe-locked Phase-0 (1–5); runbook `tasks/M_RP4_1_NUMBER_CLAMP.md` (now COMPLETED). Two of four processor kinds are now built (1 transformer, 3 filter/guard).

**Pure core (`transform.ts`).** `ClampRule {min?,max?}` + `applyClamp(n: number|null, rule): number|null` — pure, total, **idempotent**, `null` (empty field) passes through; each bound applied only if present; `min>max` keeps the upper clamp (total, no throw). Co-located with the kind-1 core (both DOM-free, the `logic.ts` posture). The `ProcessorRule` union stays codified-not-declared (D-065).

**Engine (`processor/clamp.ts`).** A **new sibling attachment**, NOT a branch of `processor.ts` — kind 1 is `input`-shaped with caret restore; kind 3 is **`change`-shaped**, numeric-coerce, no caret. `clamp({min?,max?})` → forwardable attachment (`createAttachmentKey`); on `change` reads `valueAsNumber`, coerces via `applyClamp`, writes back + dispatches synthetic `input` to sync `bind:value`; re-entrancy-guarded; DEV hook `__XGEN_CLAMP__`.

**Host (`number.svelte`).** Gains `...rest` + `{...rest}` on `<input>` — the comment's "reserved insertion point", now the first **clamp-host**. Delivery mirrors kind-1 (`<Number {...clamp({min,max})} />`). Additive, no clamp logic in the atomic (D-065).

**CDP verify** (sampler 9422, real output — Rule 2). `vite build` clean 146 modules. Pure core via `__XGEN_CLAMP__`: `applyClamp(99,{0,10})=10`, `(-5)=0`, `(7)=7`, `(null)=null`, idempotent (`99→10→10`). Live `number#clamped` on `change`: drove `99` → DOM+getter `10`; `-5` → `0` (both bind-synced via the synthetic input); in-range `7` → no-op, value preserved (verified with `input`+`change`, since the no-op path intentionally dispatches no `input`). Registry **97→98**; reuses the `.number` skin (kind 3 adds none); 0 orphans.

**Records (atomic, D-074 — code pushed as feat; this is the docs close).** D-099 amendment (kind 3 built) + N-069 (ui-notes v0.53) + registry v0.41 (processor-kinds status) + ROADMAP v4.24 (RP node + tree + M-RP4.1 ✅ DONE) + CLAUDE PLAY (Entry head → J-455, next-active → kind 2) + this entry + runbook → COMPLETED. **Next-active:** kind 2 (converter/bridge field, `Intl`) → kind 4 (`use:render`, deferred) → dd-components.

---

## Entry J-454 — M-RP4.3 CLOSED: `substitutions-editor`, the FIRST widget — one-textarea rules editor → store → live morph, host-injected `set_substitutions` persist (seam-only real-shell verify); W-3/W-8 firmed to spec v1.1; the seed `-->`/`<--`→`->`/`<-` substring-shadowing fix — built, two-layer verified, records closed

**What happened.** Built + closed the **first `widget`** (D-102) — `substitutions-editor`, the settings UI for the substitution rules. Chat built the pure layer (Steps A–E), Clair the effect layer (Steps F–G, commit `f94a138`); one milestone, two verify homes (Lock 2). Design was Joe-locked across Phase-0 (shape 1–7); runbook `tasks/M_RP4_3_SUBSTITUTIONS_EDITOR.md` (now COMPLETED). It dogfooded + firmed the tier.

**What it is.** Not a field — a settings widget that *edits* the `[substitutions]` rules string (it contains a textarea to type rules into), feeds the `$common` `substitutions` store, and the store drives the text-processor (kind-1 transformer) on every processor-host. Home `ui/common/lib/components/widgets/substitutions-editor.svelte` — the **first `widgets/` occupant**. Phase-B.

**Shape (Joe-lock).** One textarea = the raw `" | "` string (D-100 1:1-with-TOML; no per-pair rows, no `stringifyRules`). Explicit **Apply**/**Revert** gated on `dirty && valid`. Owns `draft` (divergent buffer), `dirty`, live Tier-2 validation (`assertSafeRules` → inline warning). Getter `{dirty, valid, count}` — task-state, never payload. Seed via a **Step-A additive** `substitutions.source` (raw text stashed on `setRules`).

**W-3 firming (first-instance build finding).** First build called `invoke` via `import('@tauri-apps/api/core')` inside the widget → **Rollup build-fails**: `common` can't resolve a shell dep. Fix: persistence is a **host-injected `onApply` callback** (the imperative-one-shot seam) — the real client shell passes an `invoke('set_substitutions')`-backed callback, the sampler passes nothing → live-only. The live in-app effect stays store-mediated. **A `common` widget never imports a shell dep; shell I/O is injected.** Firms W-3, not a rewrite. Also firmed W-8 (first-run-no-config caveat: strict write no-ops, swallowed by `try/catch`, until a config exists). Spec → v1.1.

**Two-layer verify.**
- **Pure layer → sampler** (Chat, CDP 9422, both accents, real output — Rule 2). `vite build` clean 145 modules. 5th **WIDGET** tab (mounted-not-`{#if}`, N-053). `ids()` **93→97** (widget + `textarea#demo__rules` + `button#demo__{apply,revert}`). Getter seeded `{dirty:false,valid:true,count:6}` (from the real config source). Drove `aaa BBB | ccc DDD` → `{dirty:true,valid:true,count:2}`, Apply enabled. Looping pair `a aa` → `{valid:false}`, Apply `disabled:true`, inline warn `non-convergent rule: replace "aa" contains find "a"`. Drove `zzz WORKED` + Apply → store `rules:[{find:zzz,replace:WORKED}]`, `dirty:false`; **live cross-widget morph** — `textarea#processed` fed `zzz here` → `WORKED here` (DOM + registry `bind:value`), no file I/O. Revert → draft back to source, `dirty:false`. Skin in cascade (`.substitutions-editor` flex/column/380px, `.subs-note` `--t4`). Accent-swap `--accent2` `#c28840`(client)↔`#3a7ab0`(node), widget body accent-neutral. **0 orphans.**
- **Persistence → real shell, SEAM-ONLY** (Clair, CDP 9222, Joe-lock Option 2). The client shell has **no content layer** (logo + state dot + Quit), so the widget is **not mounted** there (`app_client.svelte` untouched) — the sampler already proves Apply-through-UI. The real shell verifies **persistence only**: baseline get→seed; `set_substitutions`→`<null>`; get read-through→new rules; on-disk `[substitutions] rules` written + other sections intact; relaunch→clean-slate (D-101)→seed (session-only, W-8). 0 orphans. **Split of record: logic/UI → sampler; persistence → real shell.**

**Rust (Clair, F).** `set_substitutions(rules: String)` command (`desktop.rs`) + `write_substitutions_section(config_path, rules)` helper (`app.rs`) — read-mutate-write `ClientConfig`, only `substitutions.rules` replaced, all else preserved; **strict write** (D-065: errors on missing/malformed rather than clobber). +4 write-back unit tests, client-lib 131→135.

**The seed fix (`-->`/`<--` → `->`/`<-`).** A live bug: the transformer rescans the whole field every keystroke, so the `--` rule (`-- ‒`) morphs the `--` prefix the instant it's typed — `-->` never completes (`‒>`, never `→`). `->`/`<-` carry no `--` substring. **New seed:** `-> → | <- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒`. General rule (sibling to the convergence lint): *a rule whose `find` is a substring of a longer rule's `find` shadows it during live keystroke-rescan — prefer collision-free tokens.* → D-100 amendment. Hand-synced across the two Rust seed consts (client `app.rs` + sampler host `main.rs`, Clair) + the sampler placeholder (Chat, this pass).

**W-conformance.** W-1…W-10 held; W-11 N/A (no dd-slot this instance). No clause was wrong; W-3 + W-8 firmed with real-world detail (spec v1.1).

**Records (atomic, D-074 — code pushed as feat `f94a138` + Chat's pure-layer feat; this is the docs close).** N-068 (ui-notes v0.52) + registry v0.40 (first `widgets/` occupant, widget BUILT) + widget-tier spec v1.1 (W-3/W-8 firmed + FIRMED banner) + D-100 seed amendment + sampler placeholder + ROADMAP v4.23 (RP node + tree + M-RP4.3 ✅ DONE) + CLAUDE PLAY (Entry head → J-454, next-active → M-RP4.1) + this entry + runbook → COMPLETED. **Next-active:** M-RP4.1 (kind-3 number-clamp, on `change`) → kind-2 converter field → kind-4 `use:render` (deferred) → dd-components (unblocks `temperature-indicator` + the widget registry/dynamic-mount layer).

---

## Entry J-453 — widget-tier spec: the N-059 concept-lock promoted to a formal, checkable specification (`ui/docs/xgen-widget-tier.md` v1.0 + D-102 + N-067) — the `widget` = a Level-2 UI-plugin; constraint set W-1…W-11; the dd-socket defined ahead of any dd-component; two-layer verify — design-only, no code

**What happened.** Design session (no code, no build). Promoted the N-059 `widget` concept-lock to a formal specification. Walked the two deferred N-059 locks (full constraint set + verify home) plus three questions Joe raised this session — how a widget is represented in code, whether it is "the same mechanism as plugins," and what can be defined for the widget↔dd connection *before* any dd-component exists. All locked by Joe ("locked all by your recomms" + a groups-walk over the read-in state). Authored the canonical spec doc + D-102 + N-067; N-059 gets a forward pointer. Deliverable = a spec doc, not code.

**The unit (locked).** A `widget` is a **UI plugin** — the pluggable **Level-2** tier above the di/dd × atomic/composite grid (Level 0 substrate → Level 1 components → **Level 2 widget**), home `ui/common`. Active (owns state/lifecycle + host I/O) where Level 1 is passive (N-054). Name/placement/home/one-tier-+-Phase were already Joe-locked at N-059/J-445; this session locks the definition.

**Discriminator (1a).** A widget owns state with a **transition-lifecycle persisting across renders** (task progress: draft→dirty→saving→saved) — B primary; "remove it → lose a *behaviour* vs a *layout*" the gloss (A); the closed passive-state family `open`/`revealed`/`hovered`/`dragging` the illustration (C). Settles the N-063 "one flag ≠ widget" correction.

**Plugin inheritance + the one divergence (brainstormed this session, Joe's framing).** A widget is conceptually the **same mechanism as a protocol/auth plugin** — contract-not-hardcoded, capability+Phase declaration, one aggregate getter, clean mount/unmount, swappable behind the interface. The single divergence is the **channel**: a plugin is invocation-shaped (call→return, one-shot, request-scoped); a widget is binding-shaped — a **reactive `$common` store binding** (standing subscription, mount-lifetime, read + optional write-back). *Widget = the plugin contract with the invocation channel swapped for a reactive store binding.*

**Constraint set W-1…W-11 (locked).** composes-down-only · owns state+lifecycle · I/O via declared seam only (Tauri `invoke` + `$common` stores) · one aggregate getter publishing **task-state** (`{dirty,valid,phase}`), never payload/secret (1b) · clean mount/unmount (0-orphans, no cross-widget coupling) · skin-L2 pure/effect-separable · scoped home + Phase (A/B/C) · honest phase-limits · **W-9 representation** · **W-10 plugin contract** · **W-11 dd-socket**.

**Representation (W-9, Joe's code question).** No `<component>` tag — a widget is an ordinary Svelte component (`.svelte`) in `ui/common/lib/components/widgets/`, marked Level-2 via `envelope` (tier arg / `data-tier="widget"`) so `ids()` + the sampler WIDGET tab partition it from composites. **Connection v1 = static import + placement**; a widget **registry + dynamic mount** (the true plugin-discovery layer) is **reserved** until dd-components give it a first consumer (D-065/D-069). The contract is plugin-shaped day one; only discovery is deferred.

**dd-socket (W-11, the crucial ahead-of-dd move).** No dd-component exists yet, so the widget↔dd **socket** is defined now: a widget MAY expose typed dd-slots, each a `$common` **store handle** (read + write-back) + a **named mount point**, source-agnostic (N-057). The dd binds to the store, never to widget internals — the plugin contract **+ a reactive data port** (the same binding-not-invocation divergence applied to the dd connection). `temperature-indicator` later provides a `temperature` store socket → a dd temp-gauge binds to it; the widget frame doesn't change when the dd arrives, only the slot fills.

**I/O seam (1c).** Store-mediated default (the N-057/N-058 substitutions precedent); callback/prop injection for a genuinely imperative one-shot action; a DEV hook (N-056 `__XGEN_PROC__`) for a pure-compute core. No new mechanism.

**Verify home — two layers (2 + 2a + 2b).** Pure/presentational layer (I/O stubbed) → sampler, a 5th **WIDGET** tab (mounted-not-`{#if}`, N-053) — the component-DoD extended. Effect layer (real config read/write, command round-trip, session-vs-persistent) → real shell (client/node, CDP 9222/9322 — D-097's host-real home) — real `invoke` chain incl. write-back, honest phase-limit demonstrated, real output (Rule 2). One milestone, two verify homes; a widget isn't done until both are green.

**First widgets.** First **buildable** = `substitutions-editor` (M-RP4.3; composes core-di only, no dd; Phase-B, session-only write-back under D-101) — dogfoods the spec. `temperature-indicator` = first **conceived** but **dd-blocked** (conceptually defined; nothing to plug into until a dd-component exists). The spec therefore defines the frame + connection generically (proven by the dd-free editor), instance-independent. **Provisional (D-065):** v1.0 **first-instance-provisional** — drawn against the six closed di composites, not a built widget; M-RP4.3 may amend a constraint (the tag-select→N-064 precedent).

**Doc home (3A).** New focused spec doc `ui/docs/xgen-widget-tier.md` (beside the ui-notes + components registry it extends) + **D-102** naming the decision + **N-067**; N-059 gets a `→ D-102` forward pointer.

**Records (atomic, D-074 — design-only, no feat).** `ui/docs/xgen-widget-tier.md` v1.0 (new) + D-102 (DECISIONS) + N-067 + N-059 forward pointer (ui-notes v0.51) + components registry v0.39 (Level-2 widget-tier pointer) + ROADMAP v4.22 (RP-node prose + tree + widget-spec ✅ DONE) + CLAUDE PLAY (Entry head → J-453, next-active → M-RP4.3) + this entry. No code, no build; working tree otherwise clean. **Next:** M-RP4.3 (`substitutions-editor`, the first widget — dogfoods + firms the spec) → M-RP4.1 (kind-3 number-clamp) → kind-2 converter field → kind-4 `use:render` (deferred) → dd-components (unblocks `temperature-indicator` + the widget registry/dynamic-mount layer).

---

## Entry J-452 — M-RP2.29 CLOSED: `color-picker`, the 7th di composite — compact themed picker (`#rrggbbaa`), owned-popup reuse, float-HSV lossless round-trip, open-only children correct the matrix to 93 — built, CDP-verified, records closed

**What happened.** Built `color-picker`, the **24th `core`** and the **7th di composite** — the themeable, compact answer to native `<input type=color>`, whose popup dialog is OS/Chromium-painted and **unreachable by CSS *or* JS** past the swatch (N-047, reconfirmed live in DevTools this session: the dialog is a UA-native window, not DOM — no shadow root, nothing to query; dense/compact layout is therefore only possible in an owned popup). Design walked + Joe-locked ("go by your recomms"); runbook `tasks/M_RP2_29_COLOR_PICKER.md` (now COMPLETED). Steps A–E + one fidelity fix + a cosmetic pass surfaced/driven at CDP.

**Shape.** Combobox-shaped **owned-popup** (N-063): anchor `textfield` (`__hex`, editable) + a live swatch + a **palette** icon in the chevron slot; passive di, owns only `open`; closes on outside-pointerdown (not blur — avoids racing the in-popup sliders/SV drag). Popup body: a CSS-gradient **SV surface** (`<div>` + positioned thumb, pointer x→S y→V — **not `<canvas>`**, CDP-readable per N-042) + `range`×2 (`__hue` 0–360 / `__alpha` 0–255) + a **HEXA/RGBA/HSVA** model selector (swaps the numeric row only; SV/hue/alpha are HSV-native, identical across models) + a native **`EyeDropper`** button (recycled; `#rrggbb`, keeps alpha; hidden when the API is absent) + **8 recent slots** (commit-on-close, dedup, most-recent-first).

**Value + float-HSV (the fidelity fix).** Value = canonical `#rrggbbaa` (8-digit, lowercase, always-valued; **more capable than native** — native emits no alpha). Internal source of truth = HSVA. First build rounded HSV to integers → `rgb→hsv→rgb` was lossy (seed `#9a6a30ff` came back `#99692fff`), and the buggy mount wrote the drift **back into the bound sampler state** (sticky until a fresh page load). Fix: keep `h`/`s`/`v` as **floats**, round only at display — lossless round-trip; seeds now preserved exactly. Two guarded `$effect`s (commit hsva→value/hexDraft/`lastHexa`; parse user hex edits, gated by `lastHexa`) keep the anchor field and sliders in sync without a feedback loop. Hex field accepts 6-digit (pads `ff`); invalid → `:invalid`, no commit.

**Matrix — open-only children (correcting the runbook's 97, Rule 5/6).** The `__hue`/`__alpha` ranges live **inside the `{#if open}` popup**, so they register **only while open** — a **live sub-state, like focus**, not a static cell. Stable closed-state count = **+2/cell** (composite + `textfield __hex`) → 2 cells → **89→93**; opening a cell adds its two ranges (verified live at **95**). Numeric-row inputs + SV + recents are **raw** elements (no atomic), so the count stays +2. First composite where a registered child is conditionally mounted — recorded as a live-verified state, not baseline matrix (N-066).

**Skin (the `<style>` scare).** All appearance in `skin.css` (30 `.color-picker*`/`.cp-*` rules); the only inline `style=` are **data-driven values that cannot live in a stylesheet** (swatch colour, live SV hue `--cp-hue`, thumb x/y %, alpha `--cp-solid`, each recent's colour). Zero component `<style>`. Joe saw a `<style>` badge in DevTools — that is **Vite dev** injecting global CSS into one `<style>` tag for HMR; prod `vite build` extracts it to the external `.css` (30 KB). Not hardcoding. Cosmetic pass (Joe): field sized to content (`12ch`), palette icon **outside** the field on the right (password-field pattern), tight popup line spacing (`--sp-1`), centered model-button text.

**Verify (all real, Rule 2 — Chat self-drove, sampler + CDP 9422, both accents).** `vite build` clean (144 modules). `ids()===93`; children `textfield#{default,disabled}__hex`, no collision. Seed fidelity `#9a6a30ff`/`#2a6090ff` exact. Open → `data-open`, popup mounts (SV+hue+alpha+models×3+8 recents+eyedropper), total→**95** (`range#default__{hue,alpha}` register). Hue 120 + alpha 128 drive → `#309a3080`. Hex `#00ff00`→`#00ff00ff` (6-digit pad); `#zz`→`:invalid`, value unchanged. Model swap HEXA→RGBA → 4 numeric fields, value unchanged. Recents: Escape-close then reopen → slot 0 `#00ff00ff`, dedup. Eyedropper present in WebView2. `#disabled`: no open, `aria-disabled`, input disabled. Accent swap active-model bg gold `rgb(154,106,48)` ↔ blue `rgb(42,96,144)`. 30 rules in cascade; screenshots both accents eye-checked; 0 orphans on 9422/5175.

**Records (atomic, D-074).** N-066 (ui-notes v0.50) + registry v0.38 (24th `core`, 7th di composite, color-picker ✅ BUILT, matrix 93) + ROADMAP v4.21 (RP node + narrative M-RP2.29 ✅) + this entry + CLAUDE PLAY + runbook → COMPLETED. **D-069 7th-composite watch: no promotion.** **Next-active:** the `widget` tier definition (N-059→spec) → M-RP4.3 (first widget, in-app TOML editor + write-back) → M-RP4.1 (kind-3 number-clamp).

---

## Entry J-451 — M-RP2.28 CLOSED: `tag-select`, the 6th di composite (the chip consumer) — multi-select `string[]`, owned-popup reuse, `chip` `register` opt-out makes N-064 real, a general width system — built, CDP-verified, records closed

**What happened.** Built `tag-select`, the **23rd `core`** and the **6th di composite** — the last of the N-054 di-composite backlog. A **completely new component** (own file/logic) that *reuses the owned-popup pattern* (N-063, from combobox) and *composes* two existing components as children: `Textfield` (the `__filter` query buffer) + `Chip` (the tags). Not built on combobox (no shared code). Design walked + Joe-locked across the session; runbook `tasks/M_RP2_28_TAG_SELECT.md` (now COMPLETED). Steps A–E + two in-arc additions (gear, width) + two mid-arc fixes surfaced by CDP.

**Model + registration.** `value: string[]` `$bindable` (empty `[]`), getter `{values,count}` (select-multiple precedent). The query is LOCAL `$state` bound to the child textfield (suffix `__filter` — 3rd distinct textfield suffix after `__field`/`__input`, collision-safe), cleared on pick, NOT the model. Matrix **+2/cell** (composite + `__filter`) → 3 cells → **83→89**.

**N-064 made real (the headline correction).** The recorded N-064 ("chips used internally without per-instance registration") did **not** hold mechanically — `envelope` registers whenever a debug getter is present, so id-less chips got ordinal ids (`chip#1..4`, caught at CDP: total 91 not 89). Fixed with an **additive `register` prop** on `chip` (default true; `register={false}` omits the getter → renders + stamps `.chip` but does not register). tag-select renders chips `register={false}` → 0 stray chips, matrix deterministic at 89. (Rule 6 stop-and-surface: the fix edits the closed `chip` atomic, so it was Joe-locked before landing.)

**Popup + behaviour.** Own `<ul role=listbox aria-multiselectable>`, TWO sections: top "Selected (N)" (reveals all, reachable even when the row collapses) + "Options" (`notSelected && matchesQuery`, hide-selected). Pick stays open + clears query + refocus. `allowCreate?` (Enter, no exact match → value===label), silent case-insensitive dedup, `max?` (no-op + `[data-full]` dim + input disabled + no open), Backspace-last.

**Structure + two additions (Joe).** Password-field layout: root `.tag-select` = flex row = `.tag-field` box (anchors popup) + an **outside** manage gear (`.tag-manage`, transparent cog mask, fires `onManage?` — keyword-set editor is widget-tier, deferred). **Width system:** no `width` → `.tag-field` `max-content` + cap `DEFAULT_CAP=3`; `width` set → a hidden mirror row gives natural chip widths + a `ResizeObserver` tracks the field → only fully-fitting chips shown, rest → `+N` (no half-clipped chips). Two CDP-caught fixes: gear was clipped inside the field (moved outside, sibling); a stale Svelte-HMR state showed a wrong fit count until a clean reload (recorded honestly, Rule 1).

**Candidate collection (deferred).** `options` is source-agnostic (N-057). The persistent vocabulary will live in the client TOML `[tags] keywords = [...]` (seed `["important","work","personal"]`) → loader → `get_tags` command → store; write-back = widget-tier (M-RP4.3). NOT built — sampler passes a literal.

**Verify (all real, Rule 2 — Chat self-drove, sampler + CDP 9422, both accents).** `vite build` clean. `ids()===89`; children `textfield#{default,max,create}__filter` (no collision); 0 stray chips. Getters default `{count:4}`/max `{count:2}`/create `{count:0}`. Freeform create `zzz`→value===label; dedup `ZZZ`→no-op; Backspace-last→0. Popup `["Selected (4)","Options"]`, hide-selected; pick `later`→count 5, stays open, query cleared. Width: fixed 260→2 chips + `+2` **no clipping**; auto max 295 / create 155 fit content. max cap → no-op + `[data-full]` + disabled. Gear outside `.tag-field` all 3 cells, mask set, click-safe. `✓` mark gold `rgb(194,136,64)` ↔ blue `rgb(58,122,176)`. 0 orphans.

**Records (atomic, D-074).** N-064 amend + N-065 (ui-notes v0.49) + registry v0.37 (23rd `core`, 6th di composite, tag-select ✅ BUILT, matrix 89) + ROADMAP v4.20 (RP node M-RP2.28 ✅) + this entry + CLAUDE PLAY + runbook → COMPLETED + `tasks/HANDOFF_UI_TIER_DISCUSSION.md` ACTIVE→DEPRECATED (deliverable met by J-445). **Next-active:** the N-054 di-composite backlog is CLOSED → `color-picker` (reuses owned-popup, N-047) → the `widget` tier definition (N-059→spec) → M-RP4.3 (first widget) → M-RP4.1.

---

## Entry J-450 — M-RP2.27 CLOSED: `chip`, a standalone di token (22nd `core`) — self-computed colour (first di), the used-internally-without-registration pattern (N-064); prerequisite for `tag-select` — built, CDP-verified, records closed

**What happened.** Built `chip`, the **22nd `core`** — a standalone di token (atomic-ish `<span class="chip">`, no self-registering children; the `×` is a raw `<button>`). Not a composite: built standalone as the **prerequisite for `tag-select`** (M-RP2.28) and because it recurs downstream (dd facets, tier/`is_ai` badges, entity tokens). Design Joe-locked across the session (chip standalone; `×` right / `×`-only; uppercase; self-computed colour); runbook `tasks/M_RP2_27_CHIP.md` (now COMPLETED). Steps A–D.

**Self-computed colour (the headline).** First di whose colour comes from its own content: `hash(label)`→hue at a fixed muted S/L band (fill `hsl(h 45% 82%)`, text `hsl(h 55% 30%)`, border `hsl(h 40% 80%)` — never white), injected as inline `--chip-bg/fg/bd` the `.chip` skin reads (the `--led-colour` mechanism). `led` was caller-supplied; every other di is accent-derived — `chip` computes its own, so it is **shell-independent** (identical under gold/blue).

**N-064 pattern.** A standalone component (self-registers in its own sampler cells) can be **used internally without per-instance registration** when instances are dynamic — `tag-select` will render chips via `{#each}` with no `envelope` per chip, so the consumer's matrix stays predictable (+2/cell, chips don't multiply).

**Contract.** `label` (raw stored value; uppercase + **bold** display-only) / `removable?` (default true, `×` right, `×`-only, body inert-selectable) / `onRemove?` / `id`; getter `{label, removable}`; ellipsis truncation; `×` masked stroke glyph (`--chip-x`, N-052).

**Verify (all real, Rule 2 — Chat self-drove, sampler + CDP 9422, both accents).** `vite build` 142 modules. `ids()===83`; `#default {label:"rust",removable:true}` / `#static {removable:false}` (no `×`) / `#long` ellipsis-truncated. Computed fills differ per label — rust `rgb(244,225,242)` ≠ svelte `rgb(225,233,244)`, both muted; rust fill **identical** under node shell (self-computed proof). `×` present/default, absent/static; `×` click fires bound `onRemove` (internal spy counter, not surfaced), no throw. Screenshot eye-checked. **Joe cosmetic (as-shipped):** fill L 92→82 (−10% brightness), label `font-weight:700`.

**Records (atomic, D-074).** N-064 (ui-notes v0.48) + registry v0.36 (22nd `core`, chip build note, matrix 80→83, tag-select schema stub updated) + ROADMAP v4.19 (RP node M-RP2.27 ✅) + this entry + CLAUDE PLAY + runbook → COMPLETED. **Next-active:** **M-RP2.28 `tag-select`** (the chip consumer) → `color-picker` → widget definition → M-RP4.3. **Housekeeping still open:** `tasks/HANDOFF_UI_TIER_DISCUSSION.md` ACTIVE→SUPERSEDED (deliverable met by J-445) — flip next touch.

---

## Entry J-449 — M-RP2.26 CLOSED: `combobox`, the 5th di composite — native `<datalist>` tried + reverted, rebuilt as a rich **owned-popup** (own `<ul role=listbox>`, passive; owned-popup pattern → color-picker) — built, CDP-verified, records closed

**What happened.** Built `combobox`, the **21st `core`** and the **5th di composite**, 5th backlog pick (N-054). A two-phase round: started native, reverted, rebuilt owned. Design walked + Joe-locked live (6 locks); runbook `tasks/M_RP2_26_COMBOBOX.md` (now COMPLETED).

**Native tried → reverted (the pivot).** First built Path A: `textfield` + native `<datalist>` via a Step-A additive `list?` prop, decorative ▼. Passive and cheap, but the native datalist popup is **OS/WebView-drawn** — rows are text-only and completely unstyleable (no compact spacing, no left-align, no rich content). Joe's usage needs **rich rows** (icon/status), which native can't render at all. So the native version (incl. the `textfield` `list?` prop) was **reverted from a clean slate** — not kept as a baseline (documenting a dead-end wasn't worth the second component). N-063.

**Owned-popup rebuild.** `combobox` now renders its own `<ul role="listbox">`: compact, left-aligned, no balloon, rich `options` `{value,label,status?,disabled?,icon?}[]` (back-compat `string[]`; `icon?` declared but **unwired** until an icon primitive lands). **Passive di** — owns exactly one UI flag, `open` (same order as password-field `revealed`). Settled a live disagreement: an earlier over-rigid "owns `open` → widget" was **corrected** — the `widget` bar (N-059) is a *behaviour contract*, not *any state*; a styled dropdown with one open flag stays a passive di-composite. Establishes the reusable **owned-popup pattern** → `color-picker` will reuse it (native colour popup is also unstyleable, N-047).

**Collision caught at CDP.** The child `textfield` first used suffix `__field` — which **collided** with password-field's child key (`textfield#default__field`) when both share instance id "default": registry came up **78, not 80** (two child getters shadowing). Fixed by giving combobox its own child suffix `__input` (`textfield#default__input`) — collision-safe; each textfield-bearing composite now owns a distinct child suffix.

**Icon + affordance.** ▼ is a stroke-only masked glyph (N-052 lineage — the eye/drop precedent: outline comes from a `fill=none` SVG under a mask, not an inline SVG). Chevron collapsed, closed-triangle on `[data-open]` (the real `open` makes the swap honest). Rendered both candidates for Joe (chevron vs closed-triangle) before he picked chevron-collapsed. The glyph lives on a real `.chev` span (not `::after`) so it can carry a **finger cursor** + a click that focuses the field/opens (Joe's cosmetic add).

**Verify (all real, Rule 2 — Chat self-drove, sampler + CDP 9422, both accents).** `ids()===80`; children `textfield#*__input` (no collision). Open-on-focus sets `data-open` + mounts `<ul>` + ▼→triangle; filter narrows (`"on"`→Online); select sets value + closes + ▼→chevron; disabled composite inert (aria-disabled, input disabled, no open); disabled row (Offline) unselectable; `.chev` cursor `pointer`, click focuses+opens. Focus-retention across CDP evals is a harness artifact (blur closes next-tick) — drove disabled-row + open-state via synthetic `focusin`. Node-accent shot confirmed (combobox uses neutral `--s3/--s4`, so it reads the same under both accents by design). 0 orphans on 9422/5175.

**Records (atomic, D-074).** N-063 (ui-notes v0.47) + registry v0.35 (21st `core`, 5th di composite, Build note M-RP2.26, combobox schema updated) + ROADMAP v4.18 (RP node M-RP2.26 ✅, reorder) + this entry + CLAUDE PLAY + runbook → COMPLETED. **Housekeeping flagged:** `tasks/HANDOFF_UI_TIER_DISCUSSION.md` is still `Status: ACTIVE` but its deliverable (widget-tier concept-lock) was met by J-445/N-059 — flip to COMPLETED/SUPERSEDED next touch. **Next-active:** the remaining di-composite backlog (`tag-select` → `color-picker`, N-054, passive-purity order; color-picker reuses the owned-popup pattern) → the `widget` tier definition → M-RP4.3.

---

## Entry J-448 — M-RP2.25 CLOSED: `file-field`, the 4th di composite (Shape A: child-composite — hidden `file` atomic + drop-zone + list); passive slice only (no remove, no progress); outline drop-icon — built, CDP-verified, records closed

**What happened.** Built `file-field`, the **20th `core`** and the **4th di composite** (after status-indicator, password-field, star-rating), 4th backlog pick (N-054). Design Joe-locked this session (Shape A + scope + Locks 1–3); runbook `tasks/M_RP2_25_FILE_FIELD.md` (now COMPLETED). Last round of the session.

**Shape A + scope (Rule 6).** Composes the real `file` atomic as a **hidden** child input (`__input`, self-registers) driven by a styled drop-zone + a file-list — the child-composite model, contrast to star-rating's Shape B. Matrix multiplies **+2/cell** (composite + child) → 3 cells → **68→74**. Passive slice only: **no remove** (a `FileList` is immutable — remove needs a `File[]` model + `DataTransfer` write-back, tag-select territory; follow-up) and **no progress/upload** (host I/O = widget-tier, N-059, deferred). Drop/pick **replaces** the selection; stays FileList-native + passive.

**Mechanics.** `files` `$bindable` (FileList|null); zone `role=button`/`tabindex`/Enter-Space → picker via a queried input ref (no atomic change); drop builds a `DataTransfer` (respects `multiple`, keeps first when single), sets `input.files`, dispatches `change` so the child `bind:files` syncs up to the composite. `data-dragging` highlight; `disabled`. Getter `{count, files:[{name,size,type}]}`.

**Name clash caught at build.** `vite build` failed: `file.svelte` is already aliased `FileField` in the sampler → renamed the composite import `FileFieldComposite`. Fixed, rebuilt clean (140 modules).

**Drop-icon (approved in-milestone touch-up).** Outline (stroked, `fill=none`) folder + short down-arrow centered in the folder rect — **info-only, no accent**. Skin-only: `--drop` mask var + `::before` on `.drop-zone` left of the label, fixed `--t3` (stays neutral even while the zone goes accent on drag). Same mask mechanism as eye/star (N-052 lineage). Joe confirmed the outline design + placement before it landed.

**Verify (all real, Rule 2 — Chat self-drove, sampler + CDP 9422, both accents).** `ids()===74`; composite + child baseline `{count:0,files:[]}`. Drop → `{count:1,files:[{name:"a.txt",size:1,type:"text/plain"}]}`; `!multiple` drop of 2 keeps 1; `#multiple` keeps 2. Enter triggers hidden input (click spy). `dragover` → `data-dragging="true"`, border `--accent2`. Disabled: `tabindex=-1`, `aria-disabled=true`, drop no-op. `::before` 18×18, mask set, bg `--t3 rgb(138,136,128)`. Accent gold `rgb(194,136,64)` ↔ blue `rgb(58,122,176)`; 4 `.file-field` rules in cascade. Screenshots eye-checked (icon left of label). 0 orphans.

**Records (atomic, D-074).** N-062 (ui-notes v0.46) + registry v0.34 (20th `core`, 4th di composite, Build note M-RP2.25) + ROADMAP v4.17 (RP node M-RP2.25 ✅, reorder) + this entry + CLAUDE PLAY + runbook → COMPLETED. **Next-active:** the remaining di-composite backlog (`combobox` → `tag-select` → `color-picker`, N-054, passive-purity order) → the `widget` tier definition → M-RP4.3.

---

## Entry J-447 — M-RP2.24 CLOSED: `star-rating`, the 3rd di composite (Shape B: self-contained, composes no child components — refines the composite definition); discrete-value + roving-radiogroup + hover-preview — built, CDP-verified, records closed

**What happened.** Built `star-rating`, the **19th `core` component** and the **3rd di composite** (after `status-indicator`, `password-field`), and the third di-composite backlog pick (N-054). Design Joe-locked this session (Shape B + Locks 1–3 + all-next); runbook `tasks/M_RP2_24_STAR_RATING.md` (now COMPLETED). Steps A–E.

**Shape B — the taxonomy refinement (the headline).** Unlike the first two composites, which compose real child atomic *components* that self-register and multiply the matrix, `star-rating` is a `<div class="star-rating">` (composite root-marker via `envelope`) that renders its stars **internally** in an `{#each}` of `<span role="radio">` — composing no child components. It registers **one** aggregate getter, so the matrix multiplies **flat +1 per cell** (3 cells → **+3**, 65→68), not the child-multiply of status-indicator/password-field. This **refines the composite definition** (N-061): *a di-composite is a `<div class="type">` assembly; composing child atomics is the common case, not a requirement.* → **D-069 promotion-watch** (note only unless a 4th composes-nothing composite recurs).

**di + passive.** Caller supplies `max`/`value`; interprets no domain (di). Hover-preview is transient presentational `$state` (button `:active` precedent), not host-I/O → clears the widget bar (N-059); the deliberate first backlog pick as the one unambiguously-passive candidate.

**Mechanics.** `value: number` `$bindable` (0=unrated), `max` (5), getter `{value,max}`; `role=radiogroup` + per-star `role=radio`/`aria-checked` + roving `tabindex`; arrows move+select (selection-follows-focus) with `stars[next-1].focus()`, Home=1/End=max; hover-preview (`hovered`, restores on `mouseleave`) + `clearable` (click active star → 0); `readonly` (full-colour display) + `disabled` (dims). Glyph = ★ currentColor `mask` placeholder (N-052), filled `--accent2` (gold/blue) / empty `--t4`; whole-star v1.

**Correction owned (Rule 5).** The runbook DoD first wrote the matrix as `65→66` (a `+1`-total slip); Shape B adds 1 entry × 3 cells = **+3**, so `65→68` is the correct, verified count. Runbook fixed at close before commit.

**Joe cosmetic edit.** Post-build, Joe tuned the shipped skin: star `18px` (from 20px), `gap: var(--sp-0)` (from `--sp-1`). Recorded as-shipped (if `--sp-0` is undefined in `:root`, gap resolves to 0/touching — flagged, his call).

**Verify (all real, Rule 2 — Chat self-drove, sampler + CDP 9422, both accents).** Static `vite build` 139 modules. `ids().length===68`; `#default {value:0,max:5}` / `#rated {value:3,max:5}` / `#readonly {value:4,max:5}`. Click star 4 → `4`; again → `0` (clearable). Hover split-read (N-053 Svelte-5 flush): `filled:5` while `value:0`; `mouseleave` → `filled:0` (restore). Keyboard `#rated`: `3→Right→4→Left×2→2→Home→1→End→5`. a11y: `role=radiogroup` / star `role=radio` / `checkedIdx=2`. readonly: `tabindex=null`, click no-op (stays 4), `data-readonly="true"`, `aria-disabled=null`. Colour: filled gold `rgb(194,136,64)` ↔ blue `rgb(58,122,176)`, empty `--t4 rgb(88,92,100)`; 4 `.star-rating` rules in cascade. Screenshots both accents eye-checked. Teardown 0 orphans.

**Records (this close, one atomic per D-074).** N-061 (ui-notes v0.44) + components registry v0.33 (19th `core`, 3rd di composite, Build note M-RP2.24) + ROADMAP v4.16 (RP node M-RP2.24 ✅, next-active reordered) + this entry + CLAUDE PLAY (next-active flip) + runbook → COMPLETED. **Next-active (J-445/N-059 order):** the remaining di-composite backlog (`file-field`/`combobox`/`tag-select`/`color-picker`, N-054, passive-purity order — `file-field` next) → the `widget` tier definition → M-RP4.3.

---

## Entry J-446 — M-RP2.23 CLOSED: `password-field`, the 2nd di composite (redact + reveal + caps-lock; transparent icon toggle, no-reflow skin lessons) — built, CDP-verified, records closed

**What happened.** Built `password-field`, the **18th `core` component** and the **2nd di composite** (after `status-indicator`). Root `<div class="password-field">` = `textfield`(`__field`) + a transparent icon-only `button` toggle-mode reveal(`__reveal`); owns `revealed` + `capsLock`; getter `{revealed, hasValue, capsLock}` — boolean `hasValue`, never the value. Five steps A→E per the runbook (`tasks/M_RP2_23_PASSWORD_FIELD.md`, now COMPLETED).

**Step A (own commit).** Two additive props on `textfield`, both default-absent so the 44-cell registry is behaviour-unchanged (D-065): `redactValue?` (getter returns `value:null` when true — a `password-field` child never publishes the live secret into `window.__XGEN_DEBUG__`) + `autocomplete?` (native pass-through).

**Steps B–D.** Composite mirrors the N-054 registration model (composite root + children self-registering under `<childtype>#<id>__<slot>`); matrix **56→65** (flat +9). Reveal = the toggle-mode button + reflected `aria-pressed`; child `type = revealed ? 'text' : 'password'`. Caps-lock read composite-level via bubbled `getModifierState` (no textfield touch). Sampler DI·composite panel +3 cells (default/disabled/revealed); CDP self-driven both accents.

**Revision round (Joe cosmetic — the lessons).** (1) Caps warning moved from an optional `label` child to a skin treatment: `data-caps` → red `--err-bright` field border + an overlaid `::after` "Caps Lock is on!" hint (absolute, no reflow); dropping the child flattens the matrix (no conditional entry) — *state feedback belongs in the skin via a reflected data-attr, not an injected element*. (2) Transparent icon-only reveal: eye/eye-off currentColor `mask-image` swapped on `aria-pressed` (scoped `--eye`/`--eye-off`, placeholder until the `icon` primitive, N-052); 18px, 3px gap. (3) **Width-jump** on toggle: root cause was **not** `::-ms-reveal` but the `textfield`'s reserved `padding-right:24px` in password mode (N-039 `***` inset space) — suppressing the glyph left the padding; normalizing to `--sp-2` gave identical width (CDP 155/155, jump 0) — *suppressing a per-type inset means dropping its reserved padding too*.

**Verify (all real, Rule 2).** CDP (sampler, both accents): matrix 65 flat; `textfield#default__field` = `{type:"password", value:null}` (redact) while composite `hasValue:true`; reveal click → inner `type` text + `aria-pressed:"true"` + eye→eye-off mask; caps → `data-caps:"true"` + border `rgb(230,67,67)` + `::after` "Caps Lock is on!"; transparent button `bg rgba(0,0,0,0)`/border 0/18px; no `***`; field width 155/155 jump 0. Teardown 0 orphans each pass.

**D-069 2nd-composite watch: no promotion.** The N-054 registration model held clean across a second composite — stays a note, not promoted to a decision.

**Records (this close, one atomic per D-074).** N-060 (ui-notes v0.43) + components registry v0.32 (18th `core`, Build note M-RP2.23) + ROADMAP v4.15 (RP node M-RP2.23 ✅, next-active reordered) + this entry + CLAUDE PLAY (next-active flip) + runbook → COMPLETED. **Next-active (J-445/N-059 order):** the remaining di-composite backlog (`color-picker`/`file-field`/`combobox`/`tag-select`/`star-rating`, N-054, Joe's pick) → the `widget` tier definition → M-RP4.3.

---

## Entry J-445 — Design discussion: the `widget` tier concept-locked (Level-2 app assembly above the di/dd × atomic/composite grid); M-RP4.3 reordered after the di-composite backlog + the widget definition — no code

**What happened.** Design discussion only (no code, no build). M-RP4.3 (in-app `[substitutions]` TOML editor) is the first UI unit that is **assembly + behaviour + host I/O**; the component taxonomy stops at di/dd × atomic/composite — all passive (N-054) — with no tier for a behaviour-carrying assembly. Talked it through with Joe and locked the concept + name + home for a new tier; recorded as **N-059**. Full definition (constraint set + verify home) deferred until the di-composite backlog is built.

**Locked (J-445).** (1) The unit is a **`widget`** — "ui-module" in the generic/CS sense, named to avoid the protocol/CLI-module + Tier-1-auth collision (the term is new to the record, locked not recalled — D-065). (2) It is a new **Level 2** above the di/dd × atomic/composite grid (Level 0 substrate → Level 1 components → Level 2 widget), **not** a rung wedged between atomic and composite — the grid stays pure. (3) Discriminator = **passive** (composite: props → DOM + getter, no side effects) vs **active** (widget: owns state/lifecycle, validates, host I/O). (4) **One tier, not two** — behaviour-only vs I/O-carrying is the existing **Phase** axis (A/B/C, N-028), not a class branch. (5) Home = **`ui/common`** (node apps will use some widgets).

**Verify-home lean (provisional).** A widget's defining trait (host I/O + integration) is the sampler's declared blind spot (D-097). Lean: effectful layer verifies in the real shell; the pure/presentational layer (I/O stubbed) stays sampler-tunable — the N-056 processor precedent. To be locked at full definition.

**Roadmap reorder.** M-RP4.3 is the first widget → it now waits on the widget definition. New order: di-composite backlog (passive, N-054) → widget definition (N-059 promoted to a spec) → **M-RP4.3** (first widget) → M-RP4.1 (kind-3 clamp); processor kinds 2/4 slot around as before.

**State.** No milestone state change (design only). ui-notes v0.42 (N-059); ROADMAP v4.14 (RP-node reorder clause); CLAUDE PLAY next-active reordered. No code; working tree otherwise clean. **Next:** pick a di-composite from the N-054 backlog (Joe's selection).

---

## Entry J-444 — M-RP4.4 Chat half + CLOSE: `app_sampler.svelte` swapped to the real `get_substitutions` load path (frontend literal retired) + the sampler CDP §5 pass (two fresh launches, real output) — M-RP4.4 CLOSED

**What happened.** Completed the **Chat half** of M-RP4.4 (runbook §4a/§5/§6) and **closed the milestone**. The sampler frontend stops seeding a literal and hydrates the `substitutions` store from the sampler host's real `get_substitutions` command (Clair's J-443 Rust half) — the full generate→file→load→command→setRules chain now runs live in the workbench. CDP-verified across two fresh launches; ROADMAP flipped, task COMPLETED.

**Frontend dep surfaced first (Rule 6).** The invoke swap needs `@tauri-apps/api` — the sampler frontend never called `invoke` (it seeded a literal), so it wasn't a dependency. Surfaced to Joe rather than adding silently; go given. Added `"@tauri-apps/api": "^2"` to `ui/sampler/package.json` (matching the client exactly) + `npm install` (added 1 package). The Rust host stays minimal (D-098 unaffected — this is a frontend dep).

**The swap (`app_sampler.svelte`).** Removed the module-scope `substitutions.setRules('--> → | …')` literal + its comment; made `onMount` async — `applyShell('client')` then `try { const { invoke } = await import('@tauri-apps/api/core'); substitutions.setRules(await invoke('get_substitutions')); } catch (_) {}` (outside-Tauri no-op — the plain-browser Vite preview leaves the store empty; the morph's canonical home is the Tauri window, D-098). Mirrors `app_client.svelte` (J-437). The seed now lands *after* mount, so the processor-host cell starts with empty rules and re-attaches when `setRules` resolves (source-agnostic store + attachment lifecycle, D-099). Static gate: `vite build` ✓ **137 modules** (was 122; +the `@tauri-apps/api/core` dynamic chunk).

**Verify — CDP §5 (Chat self-drove, sampler + CDP 9422, real output, Rule 2).** exe_dir = `C:\cargo-targets\XGenProtocol\debug`; config = `debug\xgen-sampler_config.toml`.
- **Launch 1 (config absent → first-run generation).** The host generated the config on start — subset only: `[substitutions] rules = "--> → | <-- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒"`. `ids().length===56`. Live morph from the loaded rules: input `x --> y :) z -- w <3 q <-- r :(` → `x → y 🙂 z ‒ w ❤️ q ← r 🙁` (all six seed pairs). Registry `textarea#processed` → `{"value":"x → y 🙂 z ‒ w ❤️ q ← r 🙁"}` — the morphed value reached `bind:value`, not just the DOM. Teardown 0 orphans.
- **Launch 2 (delete-on-start).** Pre-seeded a **sentinel** config (`rules = "zzz SENTINEL-WAS-HERE | qqq WIPED-PROOF"`); relaunched → the file was **wiped + regenerated to the seed** (sentinel gone). Live store reflected the seed, not the sentinel: input `zzz --> qqq :) end` → `zzz → qqq 🙂 end` (seed pairs morph; `zzz`/`qqq` stay literal — the sentinel's pair is absent). Teardown 0 orphans.

**What this closes.** The N-057 frontend-literal seam is closed — the sampler loads, it no longer seeds (N-058). The seed const still lives in two hand-synced Rust places (client `app.rs` + sampler host `main.rs`); a shared-const crate collapsing them is out of scope (documented). D-101's clean-slate-on-start is now proven live end-to-end in the sampler (the client/node halves were Rust-tested in J-443).

**State.** M-RP4.4 ✅ CLOSED. ROADMAP v4.13, ui-notes v0.41 (N-058), task COMPLETED. **Next:** M-RP4.3 (in-app TOML editor + write-back) → M-RP4.1 (kind-3 number-clamp).

---

## Entry J-443 — M-RP4.4 Rust half (Clair): clean-slate-on-start wired into all three binaries (client + node + sampler host) + the sampler subset-config real load path (subset-gen + minimal loader + `get_substitutions`); +7 Rust tests (client 129→131, node 306→308, sampler +3); still staged, NOT closed — Chat's `app_sampler.svelte` invoke swap + the sampler CDP pass are the open handoff (D-065)

**What happened.** Built the **Clair half** of M-RP4.4 (runbook §3/§4, Joe-locked design + D-101): the phase-scoped **clean-slate-on-start** discipline in all three binaries, plus the sampler host's **real config-load path** (the positive half of D-101). No frontend, no CDP — that's Chat's half, and the milestone stays **staged** (ROADMAP not flipped, task ACTIVE) until it + the sampler CDP pass land.

**Phase-0 grounding (D-071/D-078, before authoring).** Read the actual config-read paths, not the runbook's line hints. **Client** (`desktop.rs::run_startup`): config path derived ~154, first-run SETUP detection ~158, the seed generator is `cmd_init`'s config-birth branch (`app.rs`). **Node** (Joe's explicit flag — M-RP4.2 never touched the node): desktop `run()` reads config in the order `maybe_write_default_config` (writes if absent) → `init_logging` (reads `[logging].level`) → `run_node` (real config read), so delete-on-start goes **before** `maybe_write_default_config` and reuses it as the generator — grounded, matches the runbook exactly. **Sampler host** (`xgen-sampler/src/main.rs`): confirmed truly bare — `tauri::Builder::default().run()`, no data-dir, no serde/toml. D-101 sanctions a "tiny fs+toml capability," so serde+toml added (workspace-matched: serde 1 / toml 0.8), no protocol deps.

**Rule-6 decision surfaced to Joe (not decided silently).** The runbook left the **sampler instance data-dir path** open. Surfaced it; **Joe locked exe_dir** (parent of `current_exe()` — mirrors client/node default-instance, zero Tauri-path/bundle-id dependency, findable for Chat's CDP). The subset file is named **`xgen-sampler_config.toml`** (D-025 binary-prefix), NOT `xgen-client_config.toml`, so it can't collide with a real client config generated in the same exe_dir.

**Client (`app.rs` + `desktop.rs`).** Extracted `cmd_init`'s config-birth generation into a shared `write_fresh_config(config, keypair, ai)` (behaviour-preserving — the two existing seed tests still pass), and added `pub fn clean_slate_config(config, keypair)`: **if the config exists**, wipe + regenerate from seed; a genuine first run (no config) is **left untouched** so `run_startup`'s `!config.exists() && !keypair.exists()` SETUP detection still fires (the load-bearing reading — regeneration is conditional on prior existence). Wired `app::clean_slate_config(&config_path, &keypair_path)` into `run_startup` before the first-run read. Delete-site doc-comment carries the D-101 *why* (config ephemeral this phase) + *until-when* (persistent settings exit condition) + the **J-438 seed-once suspension** (cleared pairs reappear on relaunch, intended now).

**Node (`desktop.rs`).** Added `clean_slate_config(data_dir, port)` = wipe `xgen-node_config.toml` if present, then `maybe_write_default_config` regenerates the default from seed; wired into `run()` before `init_logging`/`run_node` read it. Node has no substitutions consumer — this keeps the discipline uniform. Same D-101 comment at the delete site.

**Sampler host (`main.rs`, net-new capability).** Minimal `SamplerConfig { substitutions: { rules } }` (contract-shape parity with the client's `SubstitutionsSection`, D-098 — NOT code reuse); `write_subset_config` (generator: writes ONLY `[substitutions] rules` from seed), `clean_slate_config` (wipe+regen on start, called in `main()`), `load_substitutions` (minimal loader), and a `#[tauri::command] get_substitutions` returning the loaded string (mirrors the client command). `DEFAULT_SUBSTITUTIONS_SEED` is **hand-synced** with the client's const — the third copy of the seed; a shared-const crate is explicitly out of scope this arc (documented at the const + owed to N-058 at close).

**Verify — Rust (real output, Rule 2).**
- Client lib (`cargo test -p xgen-client --lib`) — 129 → **131** (+2 clean-slate tests):
  ```
  test app::m_rp4_2_substitutions_tests::clean_slate_wipes_and_reseeds_existing_config ... ok
  test app::m_rp4_2_substitutions_tests::first_run_config_seeds_starter_pack ... ok
  test app::m_rp4_2_substitutions_tests::seed_is_not_resurrected_after_user_clears ... ok
  test result: ok. 131 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.72s
  ```
  (`clean_slate_leaves_first_run_untouched` also passed — count 131 = 129 + 2.)
- Node lib (`cargo test -p xgen-node --lib`) — 306 → **308** (+2):
  ```
  running 4 tests
  test desktop::tests::default_config_honours_port_override ... ok
  test desktop::tests::default_config_roundtrips_through_nodeconfig ... ok
  test desktop::tests::clean_slate_creates_config_on_first_run ... ok
  test desktop::tests::clean_slate_wipes_and_regenerates_node_config ... ok
  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 304 filtered out; finished in 0.00s
  ```
  Full node lib: `test result: ok. 308 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.53s`.
- Sampler (`cargo test -p xgen-sampler`) — **+3** (new):
  ```
  running 3 tests
  test tests::load_absent_config_is_empty ... ok
  test tests::write_then_load_round_trips_seed ... ok
  test tests::clean_slate_regenerates_subset_config_from_seed ... ok
  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```

**Pending — Chat's half + the close (NOT run, not fabricated — Rule 1).** (1) `app_sampler.svelte` drops the literal seed and hydrates via `invoke('get_substitutions')` (mirroring `app_client.svelte`, J-437). (2) The sampler CDP pass (§5: config regenerated on start, command feeds the store, live morph from the loaded rules, delete-on-start proven, count 56). (3) The close records — N-058, ROADMAP M-RP4.4 ✅, CLAUDE PLAY, task → COMPLETED. The client/node delete-on-start is Rust-tested + Joe's live check; not sampler-CDP'able.

**Canonical (this work).** `xgen-client/src/app.rs` (+`write_fresh_config`/`clean_slate_config` + `cmd_init` refactor + 2 tests) + `xgen-client/src/desktop.rs` (clean-slate call in `run_startup`) + `xgen-node/src/desktop.rs` (+`clean_slate_config` wired into `run()` + 2 tests) + `xgen-sampler/Cargo.toml` (+serde/toml) + `xgen-sampler/src/main.rs` (subset-gen + loader + `get_substitutions` + clean-slate + 3 tests) [commit, feat]; this JOURNAL J-443 + `tasks/M_RP4_4_SAMPLER_CONFIG_LOAD.md` (Rust DoD ticked, Status stays ACTIVE) [same commit — D-074]. N-058/ROADMAP ✅/CLAUDE/task→COMPLETED deferred to the true close after Chat's half + CDP. Joe pushes.

---

## Entry J-442 — M-RP4.4 design locked + runbook/D-101 authored (Chat): sampler real config-load path + clean-slate-on-start discipline; the two design tensions resolved by Joe (every-binary wipe this phase; sampler config = subset snippets); NOT built — build gated on Joe's explicit go

**What happened.** Design-walked the sampler config-load arc (M-RP4.4) and authored its deliverables: the runbook (`tasks/M_RP4_4_SAMPLER_CONFIG_LOAD.md`, PENDING) + **D-101**. No code — the build is gated on Joe's explicit go.

**The principle (Joe-locked).** Config-backed components run in the sampler through the **real** generate→file→load→command→setRules chain (contract-shape fidelity, not code reuse — D-098), so a component drops into the rewritten client/node UIs with zero reprogramming. First instance: substitutions — closes the N-057 two-hand-synced-seeds literal seam. Sets the precedent for every future config-backed component.

**Two tensions resolved by Joe.** (1) Clean-slate-on-start applies to **every binary** (client, node, sampler) this phase — config is ephemeral/deprecatable while the settings logic is in development; this **suspends J-438 seed-once** for the phase (cleared pairs reappear on relaunch — intended now, resumes at the exit condition; D-101 carries the *why* + *until-when* at the delete site and in DECISIONS). (2) The sampler config = **subset snippets** (only the sections it needs, e.g. `[substitutions]`), not the whole client/node config.

**Scope note.** The arc grew: it now touches `xgen-client` + `xgen-node` Rust (delete-on-start) — Clair's domain — alongside the sampler host (Clair) + the sampler frontend invoke swap + CDP (Chat).

**Next.** Joe's go → build M-RP4.4 (then N-058 + ROADMAP ✅ at close) → M-RP4.3 (editor + write-back) → M-RP4.1 (kind-3 clamp).

**Canonical (this work).** `tasks/M_RP4_4_SAMPLER_CONFIG_LOAD.md` (new, PENDING) + `DECISIONS.md` (D-101) + this JOURNAL J-442 + `CLAUDE.md` (PLAY next-active pointer) [commit, docs]. Joe pushes.

---

## Entry J-441 — M-RP4.2 canonical close (Chat): user-owned substitution pairs shipped + verified; N-057 + D-100 written, ROADMAP ✅ / M-RP4.3 🟡, `configs.ts` deleted, task → COMPLETED. The belated M-RP4.0 docs residue landed first (commit 1), then feat (configs.ts delete) + docs (the records)

**What happened.** Closed M-RP4.2 (user-owned substitution pairs). Both halves had already shipped and been verified: Chat's `$common` `parseRules` + source-agnostic `substitutions` store + sampler rewire (CDP-verified, J-436) and Clair's Rust `SubstitutionsSection`/`load_substitutions_section` + `get_substitutions` Tauri command + client boot hydration + six-pair first-run seed (J-437→J-440; lib 129 green; Joe-verified live). This entry is the canonical close — the records owed per §0 decision 7 / D-074.

**Tree reconciled first (JOB 0).** Seven uncommitted files were identified as the belated second half of the J-435 M-RP4.0 close (its feat + JOURNAL J-435 + CLAUDE committed at J-435; DECISIONS D-099, ROADMAP, N-056, components, the M_RP4_0 task, `textarea.svelte` host, and `configs.ts` never committed). Landed as a standalone **belated-M-RP4.0-docs commit** (commit 1) on the untouched tree, restoring HEAD coherence, before any 4.2 record was written — so no close commit inherited unknown edits.

**The close records (commit 3, same-commit — D-074).** N-057 (the source-agnostic rule store; the ` | `/first-space grammar; presets retired as the live source; Tier-2 on config data; the Chat/Clair source duality; the two-hand-synced-seeds seam). D-100 (the grammar + single-string TOML home + source-agnostic store — a **new** decision, not a D-099 amendment: the TOML home + the store-as-delivery-contract are cross-cutting choices every config-backed component inherits). ROADMAP M-RP4.2 ✅ + M-RP4.3 🟡 (v4.12). CLAUDE PLAY → M-RP4.2 closed, next-active M-RP4.3. components: `textarea` source-note (the user list, not a preset; v0.31). task → COMPLETED.

**`configs.ts` deleted (commit 2, feat).** The `arrowMorph`/`emojiMorph` presets — orphaned once the store became the source (zero importers confirmed across `ui/`) — deleted. Sample data, never architecture (D-099/N-056).

**Verify (honest — no new CDP run this close).** The close rests on prior evidence, not a fresh pass: J-436 CDP (`ids().length===56`, store-sourced morph + `bind:value` sync, Tier-2 guard); J-437→J-439 Rust (lib 129 green); Joe's live real-client check (session brief); J-440 static gate on the unified sampler seed. A formal close-time sampler-CDP pass of the six-pair canonical pack was **not** run — Joe eyes the live morph in the running sampler (J-440). Flagged, not fabricated (Rule 1/2); a formal pass remains Chat's loop if wanted.

**Next.** M-RP4.4 (sampler real config-load path + clean-slate-on-start discipline — Joe-locked design this session: **every binary** wipes any found config on start this phase, sampler config = the needed subset of the client/node sections; closes the two-hand-synced-seeds seam) → M-RP4.3 (editor + write-back) → M-RP4.1 (kind-3 clamp).

**Canonical (this work).** Commit 1 (docs, belated M-RP4.0): `DECISIONS.md` (D-099) + `docs/ROADMAP.md` + `tasks/M_RP4_0_PROCESSOR_ENGINE.md` + `textarea.svelte` + `ui/docs/xgen-ui-components.md` + `ui/docs/xgen-ui-notes.md` (N-056) + `ui/common/lib/components/processor/configs.ts`. Commit 2 (feat): delete `configs.ts`. Commit 3 (docs): `DECISIONS.md` (D-100) + `ui/docs/xgen-ui-notes.md` (N-057) + `docs/ROADMAP.md` + this JOURNAL J-441 + `CLAUDE.md` + `ui/docs/xgen-ui-components.md` + `tasks/M_RP4_2_SUBSTITUTIONS.md` → COMPLETED. Joe pushes.

---

## Entry J-440 — sampler unified to the client starter pack (M-RP4.2), Clair (Joe-directed, crosses into the sampler seat): the sampler now seeds the SAME six-pair canonical pack as the client, so the workbench mirrors shipped behaviour; supersedes §4d's "deliberately different" note; static-gated, live morph is Joe's to eye in the running sampler; still staged, NOT closed

**What happened.** Joe's model: the **sampler is the canonical component workbench** where components are built, seen, and tuned in a shared view; client/node UIs are deprecated structure to be rewritten *after* components are sharp. So every component needs its sharp/definitive form **in the sampler**. Symptom that surfaced it: `:(` and `--` didn't morph in the sampler even after the client const gained them (J-438/J-439) — because the sampler holds a **separate** seed (D-097: it's a minimal host with no client config, can't read the Rust const), and that literal was still the old `:((( 🙁🙁🙁` demo. The two seeds were never wired together; nothing about the client change could reach the sampler.

**The fix.** Updated the sampler's seed literal (`ui/sampler/src/app_sampler.svelte`) from the old `--> → | <-- ← | :) 🙂 | <3 ❤️ | :((( 🙁🙁🙁` to the **same canonical pack the client ships**: `--> → | <-- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒`. Also fixed the stale processor-cell placeholder (dropped the retired `=>` token → `type --> <-- :) <3 :( -- to morph`). The two seeds are kept in sync **by hand** (Rust `DEFAULT_SUBSTITUTIONS_SEED` in `app.rs`, TS literal in the sampler) — the alternative (wire the sampler host to read a config file) is the deprecated-UI plumbing Joe explicitly set aside.

**Records reconciled (this change reverses a prior lock, so the old claims had to go).** §4d originally locked the sampler as *deliberately different* (a `:(((` multi-char exhibit); Joe superseded that — both the `app.rs` const doc-comment and the runbook §4d bullet rewritten to "sampler mirrors the client pack". The matrix is unchanged (56) — this is demo *data*, no new cell, no registry change.

**Seat + verify (honest).** The sampler is **Chat's seat** (§0.7: sampler rewires are Chat's, CDP-verified there); this edit was made by Clair at Joe's direct, repeated instruction. Static-gated only — `npx vite build` (sampler) `✓ 134 modules transformed` / `✓ built in 515ms`. The **live morph** (type `--` / `:(` → `‒` / `🙁` in `textarea#processed`) was **not** CDP-driven by me; Joe is running the sampler and can eye it live (Vite HMR re-seeds on reload). A formal sampler-CDP pass remains Chat's loop if wanted.

**Still staged, NOT closed (D-065).** ROADMAP not flipped, task ACTIVE; canonical close (Chat: N-057/D-100/ROADMAP/components/task→COMPLETED) pending Joe's verification.

**Canonical (this work).** `ui/sampler/src/app_sampler.svelte` (seed → canonical pack + placeholder) + `xgen-client/src/app.rs` (const doc-comment: "differs from sampler" → "sampler mirrors") [commit, feat]; this JOURNAL J-440 + `tasks/M_RP4_2_SUBSTITUTIONS.md` (§4d bullet reconciled) [commit, docs]. Joe pushes.

---

## Entry J-439 — substitution starter pack gains a sixth pair (M-RP4.2 §4d tuning), Clair: `-- ‒` (double-hyphen → figure dash) appended to `DEFAULT_SUBSTITUTIONS_SEED`; const + round-trip test updated together, suite stays green (129); still staged, NOT closed (D-065)

**What happened.** Joe-directed one-pair tuning of the first-run starter pack (J-438): appended `-- ‒` (a double-hyphen `--` → figure dash `‒`, U+2012) so the seed is now six pairs. The const is the single source — `cmd_init` and the round-trip test both ride it — so the change is one literal edit plus the test's expected-pairs vec.

**The seed (now six pairs, Joe-locked):**
```
--> → | <-- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒
```
Ordering note: `--> → ` precedes `-- ‒`, so the arrow rule wins on `-->` and only a bare `--` (not part of `-->`) becomes `‒` — the literal in-order `applyRules` pass does the right thing, no convergence risk (`‒` doesn't contain `--`).

**Verify — Rust (real output, Rule 2).**
- `m_rp4_2_substitutions_tests` (the round-trip test now asserts the six expected pairs incl. `("--","‒")`):
  ```
  running 6 tests
  test app::m_rp4_2_substitutions_tests::absent_file_defaults_to_empty ... ok
  test app::m_rp4_2_substitutions_tests::malformed_toml_defaults_to_empty ... ok
  test app::m_rp4_2_substitutions_tests::present_section_round_trips_raw_string ... ok
  test app::m_rp4_2_substitutions_tests::absent_section_defaults_to_empty ... ok
  test app::m_rp4_2_substitutions_tests::seed_is_not_resurrected_after_user_clears ... ok
  test app::m_rp4_2_substitutions_tests::first_run_config_seeds_starter_pack ... ok
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 123 filtered out; finished in 1.08s
  ```
  Suite unchanged at 129 (same count, the round-trip test just asserts one more pair).

**Records synced.** `tasks/M_RP4_2_SUBSTITUTIONS.md` §4d (the seed `toml` block + the DoD tick) updated to the six-pair string; J-438's quoted seed is left as the contemporaneous record (not rewritten). Still staged — Joe's regenerate-from-scratch verify + the canonical close (Chat: N-057/D-100/ROADMAP/components/task→COMPLETED) pending. Note (separate, still open per Joe's direction): unifying the **sampler** demo string to the canonical pack — Chat's `app_sampler.svelte` seat, not yet actioned.

**Canonical (this work).** `xgen-client/src/app.rs` (const +`-- ‒`; round-trip test → six pairs) [commit, feat]; this JOURNAL J-439 + `tasks/M_RP4_2_SUBSTITUTIONS.md` (§4d seed sync) [commit, docs]. Joe pushes.

---

## Entry J-438 — substitution pairs first-run seed (M-RP4.2 §4d), Clair: a fresh client config ships a locked starter pack instead of an empty `[substitutions]` — seeded once at config birth (`cmd_init`), never resurrected after the user clears it; +2 tests (lib 127→129); still staged, NOT closed (Joe's regenerate-from-scratch verify gate + Chat canonical close pending — D-065)

**What happened.** Small follow-on to the M-RP4.2 arc (J-437): seeded the client's first-run config generator so a brand-new client ships with a working starter substitution list instead of an empty `[substitutions]`. This upgrades Joe's end-to-end verify gate — delete the dev config, let the client regenerate at SETUP, and confirm a processor-host morphs from the **auto-seeded** pack (generation → load → Tauri command → store → morph), strictly stronger than hand-adding the TOML line. Milestone stays staged: ROADMAP not flipped, task ACTIVE, canonical records (N-057/D-100/components/task→COMPLETED) are Chat's after Joe verifies.

**The seed (Joe-locked, verbatim §4d).**
```toml
[substitutions]
rules = "--> → | <-- ← | :) 🙂 | <3 ❤️ | :( 🙁"
```
Five pairs: `-->`→`→`, `<--`→`←`, `:)`→`🙂`, `<3`→`❤️`, `:(`→`🙁`. Intentionally differs from the sampler demo string — the sampler keeps `:((( → 🙁🙁🙁` as a multi-char-replace exhibit; the shipped starter pack uses the cleaner `:( → 🙁`.

**Where it lives (a grounding correction).** The §4d brief named `default_config_toml()` / `maybe_write_default_config` (the node's J-080 path). The **client has no such function** — its first-run config generator is `cmd_init` (`app.rs`), which serialises a `ClientConfig` to TOML and writes it. `cmd_init` has exactly one config-*birth* path: the `!config_file.exists()` branch (builds from `ClientConfig::default()`, writes). The other two paths operate on an existing config — the `--ai` re-init branch reads-modifies-`[ai]`-writes (round-tripping and thus preserving the user's existing `[substitutions]`), and the non-AI "already exists" path doesn't write at all. So the seed is applied **only** in the config-birth branch.

**Seed-once semantics (the load-bearing locked behaviour).** Seeded `cfg.substitutions.rules = DEFAULT_SUBSTITUTIONS_SEED` in the config-birth branch only. `ClientConfig::default()` stays **empty** and `load_substitutions_section` falls back to `SubstitutionsSection::default()` (empty) — so there is **no** code path that resurrects the seed: a user who clears their pairs (edits `rules` in the existing file) keeps them cleared, because every later launch reads the file, never the default. Deleting the *whole* config file is a legitimate fresh birth and re-seeds (that's Joe's verify gate, by design). "Clear pairs" ≠ "delete config".

**Literal vs shared const (the handoff open question).** Chose a named **`pub const DEFAULT_SUBSTITUTIONS_SEED: &str`** (next to `SubstitutionsSection`) over an inline literal — one definition referenced by `cmd_init`, with the round-trip test asserting the five expected pairs *independently* (not `== const`) so a wrong const is caught, not rubber-stamped.

**Verify — Rust (real output, Rule 2).**
- New `m_rp4_2_substitutions_tests` cases (`first_run_config_seeds_starter_pack`: real `cmd_init` in a tempdir → `load_substitutions_section` → seed string round-trips AND a Rust mirror of the grammar parses it to the exact five `(find, replace)` pairs; `seed_is_not_resurrected_after_user_clears`: cmd_init seeds → blank `rules` + write-back → re-run cmd_init (config exists → not overwritten) → load returns `""`):
  ```
  running 6 tests
  test app::m_rp4_2_substitutions_tests::absent_file_defaults_to_empty ... ok
  test app::m_rp4_2_substitutions_tests::malformed_toml_defaults_to_empty ... ok
  test app::m_rp4_2_substitutions_tests::present_section_round_trips_raw_string ... ok
  test app::m_rp4_2_substitutions_tests::absent_section_defaults_to_empty ... ok
  test app::m_rp4_2_substitutions_tests::first_run_config_seeds_starter_pack ... ok
  test app::m_rp4_2_substitutions_tests::seed_is_not_resurrected_after_user_clears ... ok
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 123 filtered out; finished in 1.09s
  ```
- Full `xgen-client` suite (lib 127 → **129**, the +2 above): `cargo build -p xgen-client` → `Finished dev profile ... in 0.32s`; `cargo test -p xgen-client` lib → `test result: ok. 129 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`, all integration binaries `0 failed`.

**Pending — Joe's gate (NOT run, not fabricated — D-065/Rule 1).** Delete the dev `xgen-client_config.toml` → launch `xgen-client` (→ SETUP regenerates the config with the seeded pack) → confirm a processor-host morphs from it. Once Joe confirms, the canonical close (Chat) writes N-057 + D-100 + ROADMAP M-RP4.2 ✅ + the components note + task → COMPLETED. `configs.ts` deletion stays a separate close item (not now).

**Canonical (this work).** `xgen-client/src/app.rs` (+`DEFAULT_SUBSTITUTIONS_SEED` const + seed in `cmd_init` config-birth branch + 2 seed tests) [commit, feat]; this JOURNAL J-438 + `tasks/M_RP4_2_SUBSTITUTIONS.md` (§4d DoD ticked) [commit, docs]. Joe pushes.

---

## Entry J-437 — user-owned substitution pairs (M-RP4.2), Clair Rust half: the client TOML `[substitutions] rules` string → loader → `get_substitutions` Tauri command → `ui/client` boot hydration of the source-agnostic store; Rust loader landed + unit-tested (127 pass), real-client morph verification is Joe's remaining gate before the canonical close (still staged, NOT closed — D-065)

**What happened.** Built the **Clair half** of M-RP4.2: fed the source-agnostic `$common` substitution store (Chat's J-436) from the real client's settings. The user's ONE list of pairs now flows from `xgen-client_config.toml [substitutions] rules` → a Rust loader → a Tauri command → the Svelte store on boot. Verbatim `[sync]` / `load_sync_section` / `get_state` precedent — no engine change, no wire change, client-only (the sampler is a minimal host with no client config, D-097, so this half is not sampler-CDP'able). The milestone stays **staged**: it does not close until Joe verifies the morph in the real client and Chat writes the canonical records (N-057 / D-100 / ROADMAP ✅ / components / task→COMPLETED).

**The Rust landing (three files, one feat).**
- `xgen-client/src/app.rs` — `SubstitutionsSection { #[serde(default)] rules: String }` (the one-string TOML home, mirroring the future one-textarea editor 1:1, not an array); a `#[serde(default)] substitutions` field on `ClientConfig`; the `Default for ClientConfig` arm; and `pub fn load_substitutions_section(config_path: &Path) -> SubstitutionsSection` — read file → `toml::from_str::<ClientConfig>` → `.substitutions`, defaulting to empty on absent file / absent section / parse error. The Rust side carries the raw string **verbatim** — all parsing of the ` | ` + first-space grammar happens in the Svelte store (the engine stays source-agnostic, D-099).
- `xgen-client/src/desktop.rs` — `#[tauri::command] fn get_substitutions(config: tauri::State<ConfigPath>) -> String` returning `load_substitutions_section(&config.0).rules`, registered in `invoke_handler(tauri::generate_handler![get_state, get_pacing_state, quit, get_substitutions])`.
- `ui/client/src/app_client.svelte` — on `onMount` (alongside the existing `get_state` invoke), `substitutions.setRules(await invoke('get_substitutions'))`, importing `substitutions` from `$common/components/processor/store.svelte`.

**Config-path access decision (handoff open question).** Chose **managed Tauri state** over recompute: a `struct ConfigPath(PathBuf)` `.manage()`d in `run()` as `data_dir.join("xgen-client_config.toml")`, mirroring the existing `CurrentState`/`Pacing`/`PipeShutdown` managed-state pattern. Recompute was the worse option — the data-dir is resolved from the `--instance` label at launch, so re-deriving it inside the command would have to re-thread that resolution; the managed path is the same one `run_startup` uses (line ~139). The command reads `tauri::State<ConfigPath>` exactly as `get_state` reads `tauri::State<CurrentState>`.

**Verify — Rust (real output quoted, Rule 2).**
- New loader unit tests (`m_rp4_2_substitutions_tests`, four cases: present section round-trips the raw string incl. emoji + internal-space `replace`; absent section → empty; absent file → empty; malformed TOML → empty):
  ```
  running 4 tests
  test app::m_rp4_2_substitutions_tests::absent_file_defaults_to_empty ... ok
  test app::m_rp4_2_substitutions_tests::malformed_toml_defaults_to_empty ... ok
  test app::m_rp4_2_substitutions_tests::present_section_round_trips_raw_string ... ok
  test app::m_rp4_2_substitutions_tests::absent_section_defaults_to_empty ... ok
  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 123 filtered out; finished in 0.00s
  ```
- Full `xgen-client` suite (lib 123 → **127**, the +4 above; integration unchanged): `cargo build -p xgen-client` → `Finished dev profile ... in 6.98s`; `cargo test -p xgen-client` lib → `test result: ok. 127 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`, all integration binaries `0 failed`.
- `ui/client` static gate (`npx vite build`, the project's gate — no svelte-check in the shells): `✓ 122 modules transformed.` / `✓ built in 449ms` — the `$common/components/processor/store.svelte` import resolves.

**Pending — Joe's gate (NOT yet run, not fabricated — D-065/Rule 1).** The real-client morph (launch `xgen-client`, put a `[substitutions]` line in `xgen-client_config.toml`, confirm a processor-host morphs from it) is Joe's verification step and has **not** been performed. There is no on-disk dev client config in the repo (generated at first-run SETUP), so Joe adds the line by hand, e.g.:
```toml
[substitutions]
rules = "--> → | <-- ← | :) 🙂 | <3 ❤️ | :((( 🙁🙁🙁"
```
Once Joe confirms the morph, the canonical close (Chat) writes N-057 + D-100 + ROADMAP M-RP4.2 ✅ + the components note + task → COMPLETED.

**Canonical (this work).** `xgen-client/src/app.rs` (+`SubstitutionsSection`/field/Default/`load_substitutions_section`/loader tests) + `xgen-client/src/desktop.rs` (+`ConfigPath` state/`get_substitutions`/registration) + `ui/client/src/app_client.svelte` (store import + boot hydration) [commit, feat]; this JOURNAL J-437 + `tasks/M_RP4_2_SUBSTITUTIONS.md` (Clair DoD items, honest staged state) [commit, docs]. N-057/D-100/ROADMAP/components/task→COMPLETED stay deferred to the true close after Joe verifies. Joe pushes.

---

## Entry J-436 — user-owned substitution pairs (M-RP4.2), Chat half: one user list replaces the demo presets — `parseRules` + a source-agnostic `$common` store + sampler rewire, CDP-verified; Clair Rust half (config struct + Tauri command + client hydration) is the open handoff (staged, NOT closed — D-065)

**What happened.** Built the **Chat half** of M-RP4.2: retired `arrowMorph`/`emojiMorph` as the *live source* and replaced them with **one user-owned list** of substitution pairs, fed through the unchanged kind-1 engine. Added `parseRules` to `transform.ts`, a new source-agnostic reactive store `processor/store.svelte.ts` in `common`, and rewired the sampler `textarea#processed` cell to read the store. Self-drove CDP verification in the sampler, recorded honestly as a **staged** milestone. The runbook is `tasks/M_RP4_2_SUBSTITUTIONS.md` (design-walked + Joe-locked this session). This resolves Joe's original "arrows worked, emoji didn't" confusion: the presets were never architecture — there is **one list**, and arrows + emoji now morph together from it.

**The locked grammar (literal, no regex — Joe-specified).** Whole list = one string; pairs split on the literal ` | ` (space-pipe-space); within a pair, split on the **first space** → `find` (before) | `replace` (everything after). `find` = any string with no whitespace; `replace` = any string at all (multi-char, emoji, internal spaces, a lone `|` e.g. `:| 😐`, a phrase like `brb be right back`); the only forbidden token substring is ` | ` itself. The engine needed **zero change** — `applyRules` already does literal multi-char replace-all via split/join.

**Decisions applied.** TOML home = a single `rules` string in a new `[substitutions]` section (mirrors the future one-textarea editor 1:1, not an array). Store is **source-agnostic** (the real client feeds it via a Tauri command; the sampler seeds a literal) — the engine stays source-agnostic per D-099 decision 9. Config-file rules are **Tier-2** (`trusted:false`): `setRules` runs `assertSafeRules` so the caps + convergence lint protect the user from a self-authored looping pair, failing safe to empty on a bad set (per-pair partition + inline warnings deferred to the M-RP4.3 editor).

**Phase-0 groundings (Rule 6, this session).** Read the real Rust config path: `xgen-client/src/app.rs` — `ClientConfig` composes per-section structs each `#[serde(default)]` with a `load_sync_section` loader (the `[substitutions]` section is a verbatim `[sync]` copy); `xgen-client/src/desktop.rs` — the Svelte shell gets data via Tauri commands in `invoke_handler![get_state, get_pacing_state, quit]` (a new `get_substitutions` registers there; `config_path` already derived in `run_startup`). The **sampler** is a minimal host (D-097) with no client config and no Tauri commands → the read-from-TOML half can't run there, which is exactly why the parser + store live source-agnostic in `common` and the sampler seeds a literal.

**Verify (Chat self-drove, sampler + CDP 9422, fresh detached launch; real output quoted, Rule 2; dispatch/read split by a tick per J-433).**
- **Parser (DEV hook) + store seed + count:** `parseRules("--> → | :| 😐 | brb be right back")` → `[{"find":"-->","replace":"→"},{"find":":|","replace":"😐"},{"find":"brb","replace":"be right back"}]` (the lone-`|` + internal-spaces-replace proof); the seeded store held all 5 demo pairs (`-->`,`<--`,`:)`,`<3`,`:(((`); `count:56` (no new cell).
- **Live morph sourced from the store:** typed `x --> y :) z <3 q :(((` + dispatched → next tick `{"dom":"x → y 🙂 z ❤️ q 🙁🙁🙁","registry":{"value":"x → y 🙂 z ❤️ q 🙁🙁🙁"}}` — **arrows AND emoji morph together from the one list**, registry synced via `bind:value`.
- **Store update re-sources:** `setRules("foo BAR")` → store `[{"find":"foo","replace":"BAR"}]`; a fresh morph of `foo and --> :)` → `BAR and --> :)` — `foo` morphs, the old `-->`/`:)` rules are gone (the store is the single live source; the attachment re-attached on the new list).
- **Tier-2 guard:** `setRules("a aa")` → `{"beforeLen":1,"afterRules":[],"rejectedToEmpty":true}` — the convergence lint rejected the self-authored loop, store fell safe to empty.
- **Screenshot (eye-checked):** re-seeded demo → `textarea#processed` renders `→ ← 🙂 ❤️ 🙁🙁🙁`. (An earlier screenshot caught the cell un-morphed — a test-sequencing artifact from combining `setRules`+dispatch in one eval before the re-attach flushed, J-433; re-firing in a fresh eval rendered correctly.)
- **Teardown:** `orphans -> 9422: False  5175: False` (0 orphans).

**Staged — NOT closed (D-065).** This is the Chat half only. The milestone does **not** close until Clair's Rust half lands and Joe verifies it in the real client; ROADMAP is **not** flipped to ✅ and the task stays ACTIVE. **OPEN HANDOFF → Clair** (runbook §4, exact `[sync]`/`get_state` precedent): `SubstitutionsSection { rules: String }` + field on `ClientConfig` + `Default` + `load_substitutions_section()` in `app.rs`; `#[tauri::command] get_substitutions -> String` registered in `desktop.rs`; `ui/client` boot `invoke('get_substitutions')` → `substitutions.setRules(...)`; a Rust unit test on the loader. The TRUE close (when Clair's half lands) then writes N-057 + D-100 (the grammar + TOML-single-string decision) + ROADMAP M-RP4.2 ✅ + the components note + task → COMPLETED.

**Canonical (this commit).** `ui/common/lib/components/processor/transform.ts` (+`parseRules`) + `processor.ts` (DEV hook +`parseRules`) + `processor/store.svelte.ts` (new) + `ui/sampler/src/app_sampler.svelte` (store import + seed + `#processed` reads store) [commit, feat]; `CLAUDE.md` (PLAY → M-RP4.2 staged, Entry pointer → J-436 + active task) + this JOURNAL J-436 + `tasks/M_RP4_2_SUBSTITUTIONS.md` (Status ACTIVE, Chat-half DoD ticked) [commit, docs]. `configs.ts` is now unimported (kept as reference, runbook decision 1). N-057/D-100/ROADMAP/components/task-COMPLETED deferred to the true close. Joe pushes.

---

## Entry J-435 — text-processor kind-1 transformer built (M-RP4.0): the edit-side engine ships as a forwarded Svelte 5 attachment in `common`; `textarea` is the first processor-host; the four-kind taxonomy on two engines codified (D-099/N-056), only kind 1 built (D-065)

**What happened.** Built the text-processor **kind 1** (transformer) — authored the new `ui/common/lib/components/processor/` folder (`transform.ts` + `configs.ts` + `processor.ts`), retrofitted `textarea.svelte` to a processor-host (one-line `{...rest}` spread + header rewrite), added a `textarea#processed` sampler cell, self-drove CDP verification in the sampler, recorded. This discharges the longest-deferred UI seam (reserved since N-029/N-032/N-038/N-040), per the design-locked runbook `tasks/M_RP4_0_PROCESSOR_ENGINE.md` v1.1. The design walk had resolved the processor into a **four-kind taxonomy on two engines**; the honest scope (D-065) was **codify all four, build only kind 1**.

**Phase-0 groundings (confirmed before authoring, Rule 6).** (1) Installed Svelte is **5.56.4** — well above the 5.29 floor for `svelte/attachments` + `createAttachmentKey`, so the P-1a attachment approach compiles (the one stop-and-flag risk; it cleared). (2) `textarea.svelte` carried no `...rest`; adding it to `$props()` + spreading on `<textarea>` does not shadow `bind:value`/`use:envelope` (directives) or explicit attrs (no collision). (3) `envelope.ts` DEV idiom mirrored for the `__XGEN_PROC__` hook; the attachment adds no registry entry.

**Decisions applied (all Joe-locked in the runbook).** P-1a edit seam = a forwarded **attachment** (not `use:`); P-2 pure sink-agnostic core (`transform.ts`); P-3 two provenance tiers (Tier-1 trusted bypass; Tier-2 serializable literal pairs only — caps `CAP_RULES=100`/`CAP_LEN=200` + convergence lint); P-4 caret-preserving value sink (transformed-prefix length) + re-entrancy-guarded synthetic `input`. Forward-clean naming: only `TransformRule` in code, `ProcessorRule` union reserved (D-099); `reversible` declared-not-implemented. Kinds 2/3/4 records-only.

**Verify (Chat self-drove, sampler + CDP 9422, fresh detached launch; real output quoted, Rule 2).**
- **Count:** `{"n":56,"hasProcessed":true}` — 55→56; the attachment adds no registry entry (the host `textarea#processed` is the one id).
- **Transform + binding-sync (split across two evals, tick gap per J-433):** eval A set `"a --> b => c"` + dispatched `input` → `{"domAfterDispatch":"a → b ⇒ c"}`; eval B (next tick) read `{"dom":"a → b ⇒ c","registry":{"value":"a → b ⇒ c"}}` — the registry value proves the synthetic `input` synced `bind:value`, not just the DOM.
- **Pure core via the DEV hook:** `{"single":"→","seq":"a → b ⇒ c ← d","noop":"plain text"}` — `applyRules("-->", …)==="→"`, sequential literal replace-all, no-op unchanged.
- **Provenance guard:** `{"loopUntrusted":"throw:ProcessorRuleError","loopTrusted":"pass","arrowUntrusted":"pass"}` — untrusted `{find:"x",replace:"xx"}` throws (convergence: `xx` contains `x`), trusted bypasses, convergent `arrowMorph` passes untrusted.
- **No-op safety:** `{"value":"no tokens here","inputEvents":1}` — a token-free value round-trips with exactly one input event (the guard suppresses a synthetic dispatch on no-op).
- **Screenshot (eye-checked):** the `textarea#processed` cell renders `1 → 2 ← 3 ⇒ 4` after a multi-token dispatch.
- **Negative control (honest, D-065):** the sibling `textarea#default` — which has **no** processor attached — held externally-typed `"line one\nline two :) --> "` with the tokens **un-morphed** (registry confirmed `{"value":"line one\nline two :) --> "}`). Not produced by this code; it proves the attachment is scoped to the one cell and does not leak onto sibling textareas.
- **Caret (P-4) — the one machine-unverifiable item:** CDP cannot drive `:focus`+caret, so the caret-stays-at-typing-point behaviour is recorded as eyeball-only (the transformed-prefix-length math is the implementation; a token straddling the caret is the documented limitation).
- **Teardown:** `orphans -> 9422: False  5175: False` (0 orphans).

**Canonical (D-074).** `ui/common/lib/components/processor/transform.ts` + `configs.ts` + `processor.ts` (new) + `ui/core/lib/components/data-independent/textarea.svelte` (`{...rest}` spread + processor-host header) + `ui/sampler/src/app_sampler.svelte` (`textarea#processed` cell) [commit 1, feat]; `ui/docs/xgen-ui-notes.md` (N-056 + the four-kind table, v0.39) + `DECISIONS.md` (D-099) + `docs/ROADMAP.md` (M-RP4 arc opened, v4.11) + `CLAUDE.md` (PLAY → M-RP4.0, prior-PLAY → J-435) + this JOURNAL J-435 + `ui/docs/xgen-ui-components.md` (`textarea` noted as processor-host, v0.30) + `tasks/M_RP4_0_PROCESSOR_ENGINE.md` Status → COMPLETED [commit 2, docs]. Two-commit close (feat → docs); Joe pushes.

**Next-active:** **M-RP4.1** — kind-3 number-clamp (fires on `change`/blur), `number` as the proving consumer (second processor-host) → the kind-2 converter field (needs a decoupled text field; `toString` may delegate to `Intl`) → kind-4 `use:render` (deferred, the render-side sanitiser arc) → dd-components. Round-2 whole-codebase audit still gates the UI milestone.

---

## Entry J-434 — `status-indicator` built (M-RP2.22): the seventeenth `core` component and the FIRST di composite (`<div class="status-indicator">` = led + label + optional link); founds the composite build pattern — aggregate getter + children self-register under stable ids (zero atomic changes); first sampler cell to multiply the registry count (44→55)

**What happened.** Built `status-indicator` — authored `status-indicator.svelte`, added the `.status-indicator` skin block, replaced the DI·composite panel's empty-state with a `status-indicator` row (3 cells), self-drove CDP verification in the sampler, recorded. The **seventeenth** `core` component and the **first di composite** — root is `<div class="status-indicator">` (the N-020/N-022 composite marker, vs an atomic's native root) composing three already-built atomics: `led` (required) + `label` (required) + an optional trailing `link`. Binding none; **di** (the caller supplies the state→colour map, caption, link target; the composite interprets no domain structure). N-049 deferred its prop surface to a build-time walk; that walk happened (Joe locked SI-1…SI-6 by recommendation), runbook `tasks/M_RP2_22_status_indicator.md` authored + pushed ahead of the build.

**Decisions (Joe-locked SI-1…SI-6 + two build-time groundings).** (1) **API** flat pass-through: `states`/`state`/`pulse?`→led, `caption`→label, `linkHref?`/`linkText?`(default `"Details →"`)/`linkExternal?`/`onLinkClick?`→link. (2) **Registration model** (the first-composite precedent): the composite registers ONE aggregate getter; the **real child atomics self-register** (grounded — `led`/`label`/`link` each pass their `debug` getter unconditionally, so `envelope` registers them whenever mounted, keyed `id ?? ordinal`) under composite-supplied **stable ids** `<id>__led`/`__label`/`__link`. **Zero changes to the three closed atomics** (D-065). (3) **SI-1 refinement:** aggregate getter is `{state, caption, hasLink}` — **`colour` dropped** (it would duplicate `led`'s `?? "#000000"` sentinel; colour is verified on the led child instead). (4) **Optional link** rendered `{#if linkHref}` — a genuine absent sub-element, **distinct from N-053** (the never-`{#if}` rule is about keeping COMPONENTS mounted for registry completeness across tabs; an absent optional link is correctly absent). (5) **Matrix accounting changes** — a composite row registers composite + each child, so the count is no longer 1-per-cell: 3 cells → **11 new entries** (3 composite + 8 children) → **44→55**. (6) **Skin** `.status-indicator` flex row, trailing `.link` `margin-left:auto`; PROVISIONAL; no new token.

**Verify (Chat self-drove, sampler + CDP 9422, fresh launch; real output quoted, Rule 2).**
- **Count:** `ids().length === 55` (44→55, the matrix-multiplication confirmed).
- **Aggregate + child getters (one eval):** `{"di":{"status-indicator#default":{"state":"ON","caption":"Connected","hasLink":false},"status-indicator#withlink":{"state":"OFF","caption":"Disconnected","hasLink":true},"status-indicator#pulse":{"state":"ERR","caption":"Error","hasLink":true},"led#default__led":{"state":"ON","colour":"#22c55e"},"label#default__label":{"text":"Connected"},"link#withlink__link":{"text":"Status page","href":"https://xgen.example/status","external":true,"disabled":false}},"rootTag":"DIV","rootClass":"status-indicator","kids":["SPAN.led","LABEL.label"],"hasDefaultLink":false,"hasWithlink":true,"hasPulseLink":true}` — the composition proof (children registered under stable ids carrying their own getters), the link-iff-href proof (`#default` has no `link#default__link` and its DOM root holds only `[SPAN.led, LABEL.label]`; `#withlink`/`#pulse` do have the link), and the composite-root-is-`<div class="status-indicator">` proof.
- **Skin + combined accent (one eval, DI·composite tab active):** `{"rowDisplay":"flex","rowAlign":"center","rowGap":"8px","linkMarginLeft":"0px","skinRules":[".status-indicator",".status-indicator > .link"],"linkClient":"rgb(194, 136, 64)","linkNode":"rgb(58, 122, 176)","ledClient":"rgb(34, 197, 94)","ledNode":"rgb(34, 197, 94)"}` — both `.status-indicator` rules in cascade, the row computes `flex`/`center`/`gap:8px`, and the **combined accent proof**: the link colour **swaps** gold↔blue while the led background `rgb(34,197,94)` is **identical** across shells (caller colour, not accent). **Honest note:** `link margin-left` computes `0px` — the `margin-left:auto` rule IS applied + in cascade, but the sampler cell hugs content so there is no free space for `auto` to absorb; the right-push manifests only when `.status-indicator` is a full-width row (its real use), not in a content-sized test cell (D-065 — recorded as-is, not faked with a demo width).
- **Screenshot (eye-checked):** the DI·composite tab shows the COMPOSITE section with three rows — green "Connected" (no link), grey "Disconnected" + gold "Status page", red "Error" + gold "View logs →"; leds show caller colours, links ride the accent.
- **Teardown:** `0 orphans - 9422/5175 free`.

**Canonical (D-074).** `ui/core/lib/components/data-independent/status-indicator.svelte` + `ui/assets/skin.css` (`.status-indicator`; **+ an unrelated Joe-requested type tune in the same file** — founded `--fs-0: 10px` and set `.paragraph` + `.link` default font-size to it; CDP-verified `--fs-0`/paragraph/link all compute `10px`, `.label` stays `--fs-1` 12px so the composite's link now reads one step smaller than its caption) + `ui/sampler/src/app_sampler.svelte` (DI·composite row) [commit 1, feat]; `ui/docs/xgen-ui-notes.md` (N-054) + `ui/docs/xgen-ui-components.md` (status-indicator promoted, first composite row + schema) + `docs/ROADMAP.md` (RP node + Present) + `CLAUDE.md` (PLAY → M-RP2.22, prior-PLAY → J-434) + this JOURNAL J-434 + `tasks/M_RP2_22_status_indicator.md` Status → COMPLETED [commit 2, docs]. **No `DECISIONS.md` touch** — a composite, applies D-096 (no amendment); the composite-registration model is recorded as N-054 (D-069 promotion-watch when the second composite reuses it). Two-commit close (feat → docs); Joe pushes.

**Next-active:** the di catalogue's atomic axis is closed and the first composite is in; ahead — the **text-processor engine** (two consumers waiting: textarea, number), then **dd-components**, with further composites (`password-field` reveal, `color-picker`, `file-field`, `combobox`, `tag-select`, `star-rating`) reusing the N-054 composite pattern. Round-2 whole-codebase audit still gates the UI milestone.

---

## Entry J-433 — sampler tabbed by class×arity (M-RP3.2): four-panel container (DI/DD × atomic/composite); all panels MOUNTED + CSS-hidden, never `{#if}`, to preserve the CDP registry-completeness invariant; pure sampler chrome

**What happened.** Restructured the sampler host (`ui/sampler/`, D-098) from one long vertical scroll into a **four-panel tab container** keyed by the catalogue's class×arity axes — edited `app_sampler.svelte` (tab state + tab bar + four panels) + `app.css` (tab/panel/empty-state chrome), self-drove CDP verification, recorded. **Pure sampler chrome** — no `core`/`common` component touched, `skin.css` untouched; matrix unchanged at **44** cells. Its own M-RP3.x sampler milestone (sibling to M-RP3.0 scaffold / M-RP3.1 populate) so the next component-build (M-RP2.22 `status-indicator`) drops cleanly into the already-tabbed di·composite panel. Joe locked the design (the tab taxonomy + the all-mounted call + refinements) ahead of the runbook; runbook `tasks/M_RP3_2_sampler_tabs.md` authored + pushed ahead of the build.

**Decisions (Joe-locked).** (1) Four panels by class×arity: **DI · atomic** (all 16 components / 44 cells), **DI · composite** (empty; first occupant `status-indicator`), **DD · atomic** (empty), **DD · composite** (empty). (2) **All panels stay MOUNTED; inactive hidden via CSS `display:none` (`class:hidden`), NEVER `{#if}`** — the load-bearing call: `envelope` registers into `window.__XGEN_DEBUG__` only while mounted (grounded in `envelope.ts`), so `{#if}`-gating inactive panels would drop their ids and break the CDP matrix-count invariant (D-097). CSS-hidden ≠ unmounted → registry stays complete → self-drive unchanged. (3) Client/node skin-swap stays **global** tool chrome above the tabs. (4) In-panel kind sub-headers promoted to **INTERACTIVE / DISPLAY / NAVIGATION** (link moved from Display to its own NAVIGATION header — aligns with the three di kinds, N-049/N-052; no cell added/removed/re-bound). (5) Empty panels carry an explicit `No components yet` placeholder. (6) Canonical labels (DI/DD · atomic/composite, the index's *atomic* vocabulary).

**Verify (Chat self-drove, sampler + CDP 9422, fresh launch; real output quoted, Rule 2).**
- Registry complete on the default tab (`-Mode state`): **all 44** instances enumerate with the other three panels mounted-but-hidden.
- Eval 1 (load): `{"n":44,"panels":["block","none","none","none"],"titles":["Interactive","Display","Navigation"],"empties":3,"tabs":["DI · atomic*","DI · composite","DD · atomic","DD · composite"],"navNext":"link"}` — DI·atomic visible, the other three `display:none`; the three kind sub-headers present; `link` sits directly under Navigation; 3 empty-states.
- Eval 2 (**the anti-`{#if}` proof**, after clicking the DI·composite tab + a reactive-flush tick): `{"nAfterSwitch":44,"panels":["none","block","none","none"],"activeTab":"DI · composite","activeEmptyText":"No components yet","activeEmptyCode":"status-indicator"}` — **`ids().length` STILL 44** through the switch (DI·atomic now hidden but mounted) while the visible panel flips to DI·composite; empty-state renders. This is the proof `{#if}` was correctly avoided.
- Eval 3 (skin-swap re-themes the active tab): `{"tabActiveClient":"rgb(154, 106, 48)","tabActiveNode":"rgb(42, 96, 144)"}` — active tab gold `#9a6a30` (client) ↔ blue `#2a6090` (node).
- Screenshots (eye-checked): DI·atomic shows the skin-swap bar + the 4-tab bar (DI·atomic active/gold) + the INTERACTIVE grid; DI·composite shows the `No components yet` placeholder naming `status-indicator` (M-RP2.22).
- Teardown: `0 orphans - ports 9422/5175 free`.

**Harness finding (recorded).** Svelte 5 reactivity flushes effects **after** the current synchronous task, so a same-eval read of `getComputedStyle`/`:not(.hidden)` immediately after `.click()` returns the **pre-update** DOM (the first eval-2 attempt read `["block","none","none","none"]` post-click — stale, not a defect). Protocol: drive the tab switch in one `eval`, read panel state in a **separate** `eval` (next CDP call = next tick, effect flushed). Sibling-shape to the N-050 stale-HMR finding but distinct cause (intra-session reactive flush, not stale dev-state).

**Canonical (D-074).** `ui/sampler/src/app_sampler.svelte` + `ui/sampler/src/app.css` [commit 1, feat]; `ui/docs/xgen-ui-notes.md` (N-053, v0.36) + `ui/docs/xgen-ui-components.md` (test-bed callout updated, v0.28) + `docs/ROADMAP.md` (RP node + Present narrative, v4.09) + `CLAUDE.md` (PLAY → M-RP3.2, prior-PLAY → J-433) + this JOURNAL J-433 + `tasks/M_RP3_2_sampler_tabs.md` Status → COMPLETED [commit 2, docs]. **No `DECISIONS.md` touch** — sampler chrome, arc-local (D-097/D-098); the all-mounted/never-`{#if}` invariant is recorded as N-053 (D-069 promotion-watch only if it recurs). Two-commit close (feat → docs); Joe pushes.

**Next-active:** `status-indicator` (M-RP2.22, the first di composite — `led` + `label` + optional `link`, all in hand) drops into the di·composite panel → the text-processor engine → dd-components.

---

## Entry J-432 — `link` built (M-RP2.21): the sixteenth `core` component and the FIRST navigation-kind di (atomic `<a href>`); commits the `<a>`-vs-`<button>` split; synthesised `disabled`; bundled-safe `external` rel; returns to accent-derived colour

**What happened.** Built `link` (di·A, atomic `<a href>`) — authored `link.svelte`, added the `.link` skin block, wired a sampler row (3 cells), self-drove CDP verification in the sampler, recorded. The **sixteenth** `core` component and the **first navigation-kind di** (a new kind alongside interactive and display). N-049 had deferred `link`'s prop surface to a build-time design walk; that walk happened this session (Joe locked all by my recommendations, plus a Q&A on icon-button / leave-app / open-modal mapping), then runbook `tasks/M_RP2_21_link.md` → build.

**A new kind + the `<a>`-vs-`<button>` commit.** `link` neither binds an editable value (interactive) nor is read-only (display): it **acts** (navigates) while carrying a `text` label. The N-049 tension was navigation-`<a>` vs an action that only *looks* like a link (a `<button>` link-styled shape). `link` **is** an `<a>` with a real `href`; the look-alike stays `button`. Never conflated.

**Three mechanics.** (1) **Synthesised `disabled`** — an `<a>` has no native `disabled`; `disabled` drops `href` (non-navigating), sets `aria-disabled="true"` + `tabindex=-1`, blocks `onclick`; skin greys via `[aria-disabled]`. First component to fake a native-absent state. (2) **Bundled-safe `external`** — `external` auto-sets `target="_blank"` + `rel="noopener noreferrer"` (no raw `target`/`rel` props). (3) **Returns to accent-derived colour** — `.link` `color: var(--accent2)` re-themes gold/blue, confirming `led`'s caller-supplied colour (N-051) was the one-off, not a turn.

**Component.** `ui/core/lib/components/data-independent/link.svelte` (`lang="ts"`). Props `href`(req)/`text`(req, `""` for icon-only)/`onclick?`/`external?`/`disabled?`/`ariaLabel?`/`id`; derived `effectiveHref`/`target`/`rel`; getter `{text,href,external,disabled}` (carries the prop `href` even when disabled drops it from the element); DEV-warn `text===""` && no `ariaLabel`; no `$bindable`, no processor seam, **no Tauri/router import**. Consumer-wiring (notes): `shell.open` for OS-browser (a raw `_blank` in a Tauri WebView can spawn a blank in-app webview), `onclick`→router for SPA, **modal-open = `button` not `link`**.

**Verify (Chat self-drove, sampler + CDP 9422, fresh launch; real output quoted, Rule 2).**
- Registry: `n_ids=44`; `link#default {"text":"Settings","href":"#settings","external":false,"disabled":false}`; `link#external {"text":"xgen.example","href":"https://xgen.example","external":true,"disabled":false}`; `link#disabled {"text":"Unavailable","href":"#x","external":false,"disabled":true}`.
- Attributes / skin / accent (one eval): `{"def":{"tag":"A","href":"#settings","target":null,"rel":null,"aria":null},"ext":{"href":"https://xgen.example","target":"_blank","rel":"noopener noreferrer","al":"XGen site (opens externally)"},"dis":{"href":null,"aria":"true","tab":"-1","deco":"none","col":"rgb(88, 92, 100)"},"linkRules":[".link",".link:hover",".link:focus-visible",".link[aria-disabled]"],"accent":{"client":"rgb(194, 136, 64)","node":"rgb(58, 122, 176)"}}`. **`dis.href===null`** = the synthesised-disabled proof (href dropped) + `aria-disabled`/`tabindex=-1`/greyed `--t4`/no underline; external carries `_blank` + safe `rel` + `aria-label`; all 4 `.link` rules in cascade; **accent: `#default` gold `rgb(194,136,64)` (client) ↔ blue `rgb(58,122,176)` (node)** — `link` rides the accent (the contrast to `led`).
- Screenshot (eye-checked): three links — accent "Settings", accent "xgen.example", greyed "Unavailable".
- Teardown: `0 orphans - ports 9422/5175 free`.

**Canonical.** `ui/core/lib/components/data-independent/link.svelte` (new) + `ui/assets/skin.css` (`.link` block) + `ui/sampler/src/app_sampler.svelte` (row + 3 cells, matrix 41→44) [commit 1, feat]; `ui/docs/xgen-ui-notes.md` (N-052, v0.35) + `ui/docs/xgen-ui-components.md` (M-RP2.21 build-note + `link` promoted from Planned + `modal`/`dialog` logged, v0.27) + `docs/ROADMAP.md` (RP node + Present narrative, v4.08) + `CLAUDE.md` (PLAY → M-RP2.21, prior-PLAY → J-432) + this JOURNAL J-432 + `tasks/M_RP2_21_link.md` Status → COMPLETED [commit 2, docs]. No `DECISIONS.md` touch (a new di kind/own atomic; applies D-096). Two-commit close (feat → docs); Joe pushes.

**Next-active:** the `status-indicator` di composite — `led` + `label` + optional trailing `link`, all three now in hand (its own design walk at build time) → the text-processor engine → dd-components.

---

## Entry J-431 — `led` built (M-RP2.20): the fifteenth `core` component and the FOURTH simple display-di; the FIRST caller-supplied-colour-map + the FIRST data-coloured atomic (colour rides an inline CSS var, not the accent); `#000000` unknown-sentinel contract

**What happened.** Built `led` (di·A, atomic inline `<span class="led">` status light) — authored `led.svelte`, added the `.led` skin block, wired a sampler row (4 cells), self-drove CDP verification in the sampler, recorded. The **fifteenth** `core` component and the **fourth simple display-di** (after label/paragraph/image, N-032). Fully locked at N-049 (J-429), so straight to runbook + build (no design walk); runbook `tasks/M_RP2_20_led.md` authored + pushed ahead of the build, build-time micro-decisions Joe-flagged (state plain prop, inline `--led-colour` var, `data-pulse` attribute-hook, 4 cells).

**Two firsts.** (1) **Caller-supplied colour map** — the `select` options-prop shape (N-034) applied to a display-di: the atomic carries a `states: Record<string,string>` map it does **not** interpret, picking a colour by the `state` key. The shells' bespoke `.state-dot` + `dotColor(state)` switch becomes this, generalised. (2) **First data-coloured atomic** — every prior component's colour came from the skin (`--accent*`/`--t*`/`--err`); `led`'s comes from the **prop**, injected as an inline `--led-colour` custom property the `.led` skin reads. The skin owns **shape only** — so `led` is the **first component whose colour is NOT accent-derived** (verified below).

**Contract (lands on the caller).** `colour = states[state] ?? "#000000"` — **full black `#000000` is the reserved unknown/undefined sentinel** (always-visible solid; a transparent dot would disappear). **Consumers must never map a real state to `#000000`** (written into the `.svelte` header). `title = state ?? "?"` (native tooltip; a set-but-unmapped key still shows — diagnostic). `role="img"` + `aria-label={title}`. Values accept hex OR `var(--token)`. `pulse?` via a reflected `data-pulse` attribute (the `.toggle[role="switch"]` attribute-hook precedent). Plain props, no `$bindable` (display-di), no processor seam.

**Verify (Chat self-drove, sampler + CDP 9422, fresh launch; real output quoted, Rule 2).**
- Registry (`-Mode state`): `n_ids=41`; `led#default = {"state":"ON","colour":"#22c55e"}`; `led#off = {"state":"OFF","colour":"var(--t4)"}` (the token reference travels in the getter); `led#pulse = {"state":"ERR","colour":"var(--err)"}`; `led#unknown = {"state":"???","colour":"#000000"}` (**the black sentinel for an unmapped key — the contract proof**).
- Computed / pulse / a11y / skin / no-accent (one eval): `{"bg":{"def":"rgb(34, 197, 94)","off":"rgb(88, 92, 100)","unk":"rgb(0, 0, 0)"},"pulse":{"pAnim":"led-pulse","dAnim":"none","pData":"true","dData":null},"a11y":{"tag":"SPAN","role":"img","aDef":"ON","aUnk":"???","title":"ON","radius":"50%","disp":"block"},"ledRules":[…,".led",".led[data-pulse]"],"kf":true,"noAccent":{"client":"rgb(34, 197, 94)","node":"rgb(34, 197, 94)"}}`. The inline `--led-colour` drives `.led` background incl. the `var(--t4)` token path (`rgb(88,92,100)`); the sentinel renders `rgb(0,0,0)`; `data-pulse` + `led-pulse` keyframes applied (`kf:true`); `.led` + `.led[data-pulse]` in cascade. **`noAccent`: `#default` bg `rgb(34,197,94)` identical client↔node** — the no-accent-dependency proof.
- Screenshot (eye-checked): four round dots — green / grey / red (pulsing) / **black** (`#unknown` visibly renders, does not vanish).
- Teardown: `0 orphans - ports 9422/5175 free`.

**Finding (expected, not a defect).** Computed `display` is `block`, not the skin's `inline-block` — flex-item blockification from the sampler cell's flex layout (the same `label` finding, N-035).

**Canonical.** `ui/core/lib/components/data-independent/led.svelte` (new) + `ui/assets/skin.css` (`.led` block) + `ui/sampler/src/app_sampler.svelte` (row + 4 cells, matrix 37→41) [commit 1, feat]; `ui/docs/xgen-ui-notes.md` (N-051, v0.34) + `ui/docs/xgen-ui-components.md` (M-RP2.20 build-note + `led` promoted from Planned, v0.26) + `docs/ROADMAP.md` (RP node + Present narrative, v4.07) + `CLAUDE.md` (PLAY → M-RP2.20, prior-PLAY → J-431) + this JOURNAL J-431 + `tasks/M_RP2_20_led.md` Status → COMPLETED [commit 2, docs]. No `DECISIONS.md` touch (new simple display-di; applies D-096, the map is the N-034 precedent). Two-commit close (feat → docs); Joe pushes.

**Next-active:** `link` (navigation `<a href>` atomic — its own design walk at build time: `href`/`text`/`target`/`rel`/external-vs-in-app/inert) → the `status-indicator` di composite (led + label + optional link — both constituents now in hand) → the text-processor engine → dd-components.

---

## Entry J-430 — `select-multiple` built (M-RP2.19): the fourteenth `core` component and the LAST input-family atomic di; the FIRST plain-array value-type (`bind:value` → `string[]`, the 5th binding shape), empty model `[]` not `null`, getter `{values,count}`; own atomic under the sharpened D-096

**What happened.** Built `select-multiple` (di·A, atomic `<select multiple>`) — authored `select-multiple.svelte`, added the `.select-multiple` skin block, wired a sampler row (3 cells), self-drove CDP verification in the sampler, recorded. The **fourteenth** `core` component and the **last input-family atomic di** (N-038); the input-family atomic axis is now closed. Joe-locked the six design decisions at the design walk (D-a..D-f); runbook `tasks/M_RP2_19_select_multiple.md` authored + pushed ahead of the build.

**Own atomic under the sharpened D-096.** Shares the `<select>` tag with `select` but fails **two** of the three sharpened-criterion clauses (root + value-type + shared skin/surface, N-042): value-type diverges (**`string[]`** vs scalar `string`) **and** skin-surface diverges (scrolling list-box vs dropdown). The `range`-vs-`number` logic, doubled. Applies D-096, **no amendment** (no `DECISIONS.md` touch).

**The headline — first plain-array value-type.** `bind:value` on `<select multiple>` yields a native **`string[]`** (no `bind:group`) — the **5th binding shape** after boolean-in/event-out/string-in/number/FileList. Empty model is **`[]`, not `null`** (set-absent vs the single-select's scalar-null) — the N-038 array landing. A plain array is `$state.snapshot`-serialisable, so the getter is trivial: `{ values: $state.snapshot(value), count: value.length }` (the `{count,…}` shape mirrors `file`). `options` carried over unchanged from `select` (the dual `string[]`/`{value,label?,disabled?}[]` shape + the same `$derived items`, N-034); `size?` default 4; `multiple` hardcoded; no `placeholder`; no processor seam. Own `.select-multiple` skin — list-box surface (`--s4`/`--s5`), accent-tinted `option:checked`, focus/disabled; no arrow, no `--ctl-h`.

**Verify (Chat self-drove, sampler + CDP 9422, both accents via skin-swap; real output quoted, Rule 2).**
- Registry seed (`cdp-debug.ps1 -App sampler -Mode state`, fresh launch): `n_ids=37`; `sm#default = {"values":[],"count":0}` (the `[]` empty-model proof, **not** `null`); `sm#seeded = {"values":["a","c"],"count":2}`; `sm#disabled = {"values":["b"],"count":1}`.
- Bind round-trip (eval; select a+b on `#default` + dispatch `change`): `EVAL RESULT: {"tag":"SELECT","mult":true,"sz":4,"roundtrip":{"values":["a","b"],"count":2}}` — a `string[]` round-trips through `bind:value` (the substrate's first plain-array value-type, proven); `multiple=true`, `size=4` (default).
- Skin / disabled / accent (eval): `EVAL RESULT: {"disabledFlag":true,"smRules":[".select-multiple",".select-multiple option",".select-multiple option:checked",".select-multiple:focus-visible",".select-multiple:disabled"],"accentClient":"#c28840","accentNode":"#3a7ab0"}` — all 5 `.select-multiple` rules in cascade (incl `option:checked`), `#disabled` inert, accent flips gold↔blue.
- Screenshots (both shells eye-checked): three dark-surface list-boxes with **accent-tinted selected rows** (gold client / blue node); `#seeded` shows Alpha highlighted + Gamma below the 4-row fold; `#disabled` greyed.
- Teardown: `0 orphans - ports 9422/5175 free`.

**Verify-harness note.** A fresh launch was required: a **stale HMR** session first reported `#default`/`#seeded` both as `['a','b']` (neither matched its seed). Teardown + relaunch gave the correct seeds — confirming the prior read was stale dev-state, not a binding defect (the `range`/`color` minimized-window finding family, N-047 shape). Recorded so the next interactive verify expects it.

**Canonical.** `ui/core/lib/components/data-independent/select-multiple.svelte` (new) + `ui/assets/skin.css` (`.select-multiple` block) + `ui/sampler/src/app_sampler.svelte` (row + 3 cells, matrix 34→37) [commit 1, feat]; `ui/docs/xgen-ui-notes.md` (N-050, v0.33) + `ui/docs/xgen-ui-components.md` (M-RP2.19 build-note, v0.25) + `docs/ROADMAP.md` (RP node + Present narrative, v4.06) + `CLAUDE.md` (PLAY → M-RP2.19, prior-PLAY pointer → J-430) + this JOURNAL J-430 + `tasks/M_RP2_19_select_multiple.md` Status → COMPLETED [commit 2, docs]. No `DECISIONS.md` touch (applies sharpened D-096). Two-commit close (feat → docs); Joe pushes.

**Next-active:** `led` + `link` (di catalogue additions, locked N-049) — `led` is runbook-and-go (fully locked); then the `status-indicator` composite (once `led` + `label` are in hand), then the text-processor engine, then dd-components.

---

## Entry J-429 — di catalogue extended (planning, nothing built): `led` (display-di status light, caller-supplied colour map) + `link` (navigation atomic, `<a href>`) + `status-indicator` (di composite = led + label + optional link)

**What happened.** A design conversation with Joe added three components to the data-independent catalogue. **Planning/concept-lock only — no code, no build.** Recorded in `ui/docs/xgen-ui-components.md` (v0.24: a `navigation` row + a Planned note + the `status-indicator` Composites row) + `ui/docs/xgen-ui-notes.md` (**N-049**, the concept-lock the catalogue rows point to). No `DECISIONS.md` touch (arc-local di-vocabulary; D-069 threshold not met).

**The three.**
- **`led`** — di·A **simple display-di** (the 4th, after label/paragraph/image), atomic inline `<span class="led">` status light. Joe's vision: **not hardcoded states/colours** but a **caller-supplied `states: Record<string,string>` map** (`{ "ON":"#ff0000", "OFF":"var(--t4)" }` — hex or `var(--token)`) + a `state` key (the `select` options-prop pattern, N-034 → fully di). `pulse?` orthogonal boolean. Unknown/undefined → **reserved full-black `#000000`** (always-visible sentinel; consumers never map a real state to black). `title = state ?? "?"` native tooltip. Getter `{state, colour}`; `role="img"`. `.led` skin = shape only, colour via inline CSS var.
- **`link`** — di·A **navigation** atomic, root `<a href>` (surfaced by the status-indicator's optional "details →" slot). Distinct from the existing *button link-styled shape* (a `<button>` that looks like a link). Full prop surface is its own build-time design walk.
- **`status-indicator`** — di **composite** (`<div class="status-indicator">` = led + label + optional trailing link). **Classification corrected by Joe: di, not dd** — the caller supplies the state→colour map + caption, the component binds no domain structure (a settings/overview panel = N of these rows). Generalises the shells' bespoke `.state-dot` + `dotColor`/`isPulsing`; becomes the shells' state-indicator when they move to lib components. (A future *domain-bound* health panel that derives states itself would be the dd version — separate, later.)

**Context.** Prompted by Joe noticing the real shells' bespoke status light (the `.state-dot`) after the J-428 shell revert, and by the broader plan to swap the shells from bespoke chrome to lib components once the di set is complete.

**Canonical.** `ui/docs/xgen-ui-components.md` (v0.24) + `ui/docs/xgen-ui-notes.md` (N-049) + this JOURNAL J-429 + a light `CLAUDE.md` PLAY Next touch (the di queue now reads `select multiple` → `led`/`link` → `status-indicator`). No `DECISIONS`/`ROADMAP` touch. Not pushed — Joe pushes.

**Next-active:** unchanged build order — `select multiple` (the last input-family atomic), then `led` + `link`, then `status-indicator`.

---

## Entry J-428 — Real client + node shells reverted to original chrome: the throwaway component demos (M-RP2.3–2.15) removed from both apps; the M-RP3.0 "revert deferred" debt discharged

**What happened.** Stripped the throwaway `core`-component demo stack from both real shells — `ui/client/src/app_client.svelte` and `ui/node/src/app_node.svelte` — returning each to its **original cosmetic state**: logo + state-indicator (status dot + label) + the one real window control (client `Quit` → `quit`, node `Shut Down` → `shut_down`). Cosmetic only; **no functionality touched** (Rule 5) — the Tauri state listener (`onMount`/`onDestroy`, `get_state`/`get_node_state`, the `xgen-*-state-changed` subscription), the `handleQuit`/`handleShutDown` invokers, and `dotColor`/`isPulsing` are byte-for-byte unchanged.

**Why it was there / why now.** During M-RP2.3–2.15 each new `core` component was wired into **both** real shells as a live registry-verification probe (the pre-sampler "verified live in both apps" method). Every instance was explicitly tagged in-source *“throwaway demo … Not a real affordance”*. When the **sampler** became the component test-bed (M-RP3.0, D-097), those probes went redundant — but M-RP3.0 left the shells *“frozen as-is (revert deferred)”* and the revert never ran. Joe saw the leftover stack on running `run-client.ps1` / `run-node.ps1` (screenshots) and called it: cosmetic debt, fix before anything else.

**Removed from each shell** (symmetric): the 11 demo instances (`Toggle`, `Textfield`, `Textfield type=search`, `Select`, `Textarea`, `NumberField`, `Range`, `Label`, `Paragraph`, `Image`, and the `demo-toggle` `Button`), their 8 `demo*` `$state` vars + the M-RP2.x comment blocks, and the now-unused imports (`Toggle`/`Textfield`/`Select`/`Label`/`Paragraph`/`Image`/`Textarea`/`NumberField`/`Range`/`Placeholder`). **Kept:** the `Button` import + the real `quit`/`shutdown` instance (the legit M-RP2.4 close-affordance, still `envelope`-registered as `button#quit` / `button#shutdown`), `onMount`/`onDestroy`, `AppLogo`.

**Verification.** Pre-checked both shells' `app.css` — no demo/component selectors (chrome only: `#core-ui-pane`/`#app-logo`/`.state-indicator`/`.state-dot`/`.state-label`/button); removal leaves no dangling CSS. Post-edit grep for leftover component refs returned only case-insensitive false positives on *“label”* (`state-label`) and *“placeholder”* (the dev-preview comment); zero real references remain. Both files end cleanly at `<main>` = logo + state-indicator + the single window button, matching Joe's reference images (client gold `Quit`, node blue `Shut Down`). Not full-app-run-verified here (pure frontend revert; if a shell was open, Vite HMR hot-reloaded it live).

**Canonical.** Two shell files + `CLAUDE.md` (M-RP3.0 PLAY note flipped *“frozen as-is (revert deferred)”* → *“reverted to original chrome … (J-428)”*) + this JOURNAL J-428. No `DECISIONS.md`, no ROADMAP, no `ui/docs/` touch (a chore/debt-discharge, not a milestone or a technique). Not pushed — Joe pushes.

**Next-active:** unchanged — the last remaining atomic di per N-038, `select multiple`.

---

## Entry J-427 — M-RP2.18 CLOSED: `file` — the thirteenth `core` component; the FIRST non-`value` binding (`bind:files` / FileList), the 4th binding shape and the first value-type `$state.snapshot` can't serialise; a `file-field` composite is logged

**What happened.** Built `file` — the **thirteenth** `core` component, an atomic `<input type="file">` (native picker button). Authored + skinned in one pass, built/tuned/verified **in the sampler** (D-097). Frontend + skin only; no protocol/data change (Rule 5).

**The headline is the binding shape, not the fold.** Own atomic is obvious — `<input type="file">` binds a **FileList**, not a string/number/boolean; no fold candidate. Applies D-096, no amendment. The real event: `file` is the **first non-`value` binding** in the library — `bind:files`, the **4th binding shape** after boolean-in (`checked`, toggle) / event-out (`onclick`, button) / string-in (`value`, the whole input family incl. date/color). The substrate (`envelope`/`debug`, N-023/N-024) had been proven across all of those, but every prior binding rode `value`/`checked`/`onclick`. It is also the first value-type **`$state.snapshot` cannot serialise** (a FileList is a live host object, not a plain object/proxy).

**Getter — de-FileList (the design point).** The getter returns a **plain** view, not the FileList: `{ count, files: [{ name, size, type }] }` (`count: files?.length ?? 0`, `files` via `Array.from`). The **bindable prop carries the live FileList** (the consumer's real value); the getter is the serialisable projection for the N-024 registry / CDP `returnByValue`. `$state.snapshot(files)` would not flatten a host FileList — the explicit map is required.

**Component + skin.** `ui/core/lib/components/data-independent/file.svelte`: root `<input type="file" bind:files use:envelope>`, `files = $bindable(null)` (`FileList | null`, empty = `null`), zero `<style>`. Props: `files`/`accept`/`multiple`/`disabled`/`id`/`name`; dropped `value` (**unsettable programmatically** — browser security; consumers read via the binding, never write `.value`), `placeholder`/`pattern`/`readonly`/min/max/step (n/a), `type` (fixed). `capture` reserved. No processor seam. Selection fires **`change`**, not `input`. Own `.file` skin styles the file-button pseudo to match `.button` (`--ctl-h`/`--sp-1 --sp-4`/`--s4`/`--s5`/`--rad`/`--t2`/`--fs-1`/pointer) in **both** spellings — `::file-selector-button` (standard) + `::-webkit-file-upload-button` (legacy) — as **separate rules** (a selector list with an unknown pseudo drops the whole rule, so they can't be comma-combined). `:disabled` greyed; the UA "No file chosen" text accepted.

**Composite logged.** A custom drag-drop file row (zone + selected-file list + remove + upload progress) is the deferred **`file-field` / `dropzone` composite** — logged in the registry Composites + ROADMAP, not built.

**Sampler.** A `file` row + **3** cells — `file#default` (single) + `file#multiple` (`multiple`) + `file#disabled`. All start `null` (a file is unsettable from markup; honest-empty). Matrix **31 → 34**.

**Verification (Chat self-drove, real `tauri dev` + CDP 9422, both accents via skin-swap).** `ids().length === 34` (3 `file#`), baselines `{count:0,files:[]}`. **Bind path (the FileList round-trip — the headline proof):** `value` is unsettable, so injected a real file via `DataTransfer` + dispatched **`change`** (file inputs fire `change`, not `input`):
```
const dt = new DataTransfer(); dt.items.add(new File(['x'],'test.txt',{type:'text/plain'})); el.files = dt.files; el.dispatchEvent(new Event('change',{bubbles:true}))
```
→ bound getter:
```
"file#default":{"type":"file","state":{"count":1,"files":[{"name":"test.txt","size":1,"type":"text/plain"}]}}
```
**A FileList round-trips through `bind:files`, de-FileLuted to plain metadata** — the substrate's first non-`value` binding, proven. `file#multiple`/`file#disabled` stayed `{count:0,files:[]}`. Element + props + skin (one eval):
```
{"defMultiple":false,"mulMultiple":true,"disDisabled":true,"style":{"tag":"INPUT","type":"file","fontSize":"12px","color":"rgb(200, 196, 188)","cursor":"pointer"},"disabledStyle":{"cursor":"not-allowed","opacity":"0.5"},"fileRules":[".file",".file::file-selector-button",".file::-webkit-file-upload-button",".file::file-selector-button:hover",".file:focus-visible",".file:focus-visible::file-selector-button",".file:disabled",".file:disabled::file-selector-button"],"accent2":"#c28840"}
```
All **8** `.file` rules parsed + in cascade (stylesheet-rule inspection, N-042 method — `getComputedStyle` won't surface `::file-selector-button`). **Skin-swap:** `--accent2` `#c28840` (client) → after `data-shell="node"` `#3a7ab0`. Screenshot (client) eye-checked: `file#default` renders the `.button`-styled "Choose File" + **"test.txt"** (the round-tripped name), `file#multiple` "Choose **Files**" (native plural), `file#disabled` greyed. (Incidental reconfirm: `color#default` showed its exact `#9a6a30` seed on this fresh load — closing N-047's churn as instance-state, not a defect.) Clean teardown (5175/9422 free, 0 orphans).

**Canonical (two commits, UI pattern `feat` then `docs`).** Commit 1 (impl): `file.svelte` (new) + `ui/assets/skin.css` (`.file`) + `ui/sampler/src/app_sampler.svelte` (row). Commit 2 (records): `ui/docs/xgen-ui-notes.md` (N-048) + `docs/ROADMAP.md` (M-RP2.18 ✅, v4.05, file-field horizon) + `CLAUDE.md` (PLAY → M-RP2.18, J-426→J-427) + `ui/docs/xgen-ui-components.md` (file row promoted + file-field composite logged, v0.23) + this JOURNAL J-427 + `tasks/M_RP2_18_FILE.md` (→ COMPLETED). **No `DECISIONS.md` touch** (applies D-096, no amendment). Not pushed — Joe pushes.

**Next-active:** the **last** remaining atomic di per N-038 — `select multiple` — then the text-processor engine, then dd-components.

---

## Entry J-426 — M-RP2.17 CLOSED: `color` — the twelfth `core` component; a SINGLETON that stands alone (the `range` case, not a fold); the native picker is OS-painted (not skinnable) → a `color-picker` composite is logged (#2)

**What happened.** Built `color` — the **twelfth** `core` component, an atomic `<input type="color">` (native swatch + picker). Authored + skinned in one pass, built/tuned/verified **in the sampler** (D-097). Frontend + skin only; no protocol/data change (Rule 5).

**Why own atomic (the design decision).** `color` has **no siblings** (unlike date's five), so the fold test is *sideways* — `color` vs `date`/`range`. Sharpened D-096 (root + value-type + **shared skin/surface**, N-042): it shares the `<input>` root **and** value-type (a **string**, `#rrggbb`) with `date` — root + value-type alone would pull toward a date fold, the trap the sharpened criterion exists for — but diverges on **skin/surface** (a **swatch**, `::-webkit-color-swatch*`, nothing shared with date's text box + calendar indicator) and prop surface (no min/max/step, no `:invalid`). So **own atomic: the `range` case, not the `textfield` case**. **Applies D-096, no amendment** (no `DECISIONS.md` touch).

**Component + skin.** `ui/core/lib/components/data-independent/color.svelte`: root `<input type="color" bind:value use:envelope>`, `value = $bindable('#000000')` (string, **always a valid `#rrggbb`** — the native control has no empty state, never `''`/`null`: the always-valued shape, like `range`), getter `{value}` (**no `type`** — singleton), zero `<style>`. The **leanest prop surface yet**: keep `value`/`disabled`/`id`/`name`; drop `placeholder`/`pattern` (n/a), `readonly` (native no-op), min/max/step (n/a), `:invalid` (always valid). No processor seam. `alpha`/`colorspace` reserved, not built. Own `.color` swatch skin (pseudo-element-heavy like `.range`): `appearance:none` + `::-webkit-color-swatch-wrapper{padding:0}` + `::-webkit-color-swatch{border:none; border-radius:calc(--rad - 1px)}`; compact 36×24 (**no `--ctl-h`**; Joe may swap to `--ctl-h` for row parity); `:disabled` → `not-allowed` + `opacity:0.5`.

**The native picker is not skinnable → composite #2 logged.** The OPEN dialog (saturation square / hue slider / eyedropper / hex+RGB fields / swatches) is OS/Chromium-painted; `.color` styles only the closed-state swatch. A themed custom palette is the deferred **`color-picker` composite** (the `password-field`-off-`textfield` shape) — logged in the components registry's Composites section + the ROADMAP, not built. (Joe confirmed the intent: the atomic is the non-programmed native version; a programmed themed palette is composite #2.)

**Sampler.** A `color` row + **2** cells — `color#default` (seed `#9a6a30`) + `color#disabled` (seed `#2a6090`); no invalid/type variants exist. Matrix **29 → 31**.

**Verification (Chat self-drove, real `tauri dev` + CDP 9422, both accents via skin-swap).** `ids().length === 31` (2 `color#`); both report `"type":"color"`:
```
"color#default":{"type":"color","state":{"value":"#123456"}},"color#disabled":{"type":"color","state":{"value":"#2a6090"}}
```
**Bind path:** dispatched a real `input` (`"#123456"`, N-029) on `color#default` → bound getter `{value:"#123456"}` (a string round-trips through `bind:value`). Element + skin (one eval):
```
{"elValAfterDispatch":"#123456","style":{"tag":"INPUT","type":"color","appearance":"none","webkitAppearance":"none","width":"36px","height":"24px","borderRadius":"6px","cursor":"pointer"},"disabled":{"isDisabled":true,"cursor":"not-allowed","opacity":"0.5"},"colorRules":[".color",".color:focus-visible",".color:disabled",".color::-webkit-color-swatch-wrapper",".color::-webkit-color-swatch"],"shell":"client","accent2":"#c28840"}
```
All **5** `.color` rules parsed + in cascade (stylesheet-rule inspection — `getComputedStyle` doesn't surface the swatch pseudos, N-042 method). **Skin-swap:** `--accent2` `#c28840` (client) → after `data-shell="node"` `#3a7ab0` (blue). Screenshot (client) eye-checked: both swatches render (`#disabled` dimmed) **and** incidentally caught the native Chromium picker open (saturation square / hue / eyedropper / RGB) — the real native dialog. Clean teardown (5175/9422 free, 0 orphans).

**Verify finding (honest — the first interactive native-popup control exposes it).** `color#default`'s value **drifted** from its `#9a6a30` seed during the minimized-window session (read `#419584`, then the dispatched `#123456`, then a stray green at screenshot) — stray pointer events on the swatch kept opening the native picker and changing the colour. `color#disabled` (non-interactive) held its **exact** `#2a6090` seed throughout. That asymmetry **proves seeding + bind are correct** (a seeding bug would have broken disabled too); a fresh user load gets `#9a6a30`. Recorded (N-047): for interactive native-popup controls (`color`, the `date` pickers), read the non-interactive cell for the seed proof + the dispatched round-trip for the bind proof, not the post-session swatch value. (Sibling-shape to the J-422 harness false-negative — the verify *method*, not the component, is what needs the care.)

**Canonical (two commits, UI pattern `feat` then `docs`).** Commit 1 (impl): `ui/core/lib/components/data-independent/color.svelte` (new) + `ui/assets/skin.css` (`.color`) + `ui/sampler/src/app_sampler.svelte` (row). Commit 2 (records): `ui/docs/xgen-ui-notes.md` (N-047) + `docs/ROADMAP.md` (M-RP2.17 ✅, v4.04, color-picker horizon) + `CLAUDE.md` (PLAY → M-RP2.17, J-425→J-426) + `ui/docs/xgen-ui-components.md` (color row promoted + color-picker composite logged, v0.22) + this JOURNAL J-426 + `tasks/M_RP2_17_COLOR.md` (→ COMPLETED). **No `DECISIONS.md` touch** (applies D-096, no amendment). Not pushed — Joe pushes.

**Next-active:** the di track continues, built/tuned in the sampler (D-097) — `file` (new `bind:files` shape) / `select multiple` per N-038 — then the text-processor engine, then dd-components.

---

## Entry J-425 — M-RP2.16 CLOSED: `date` — the eleventh `core` component; the date-input family (`date`/`time`/`datetime-local`/`month`/`week`) FOLDS into one atomic (the `textfield` fold again, not `range`); the sampler-DoD becomes a standing rule

**What happened.** Resumed the di-atomic component track (paused since M-RP3.0) and built `date` — the **eleventh** `core` component, an atomic `<input>` covering the whole date-input family. The five siblings — `date` / `time` / `datetime-local` / `month` / `week` — **fold into one component** via a constrained `type` prop (default `date`). Built, skinned, tuned, and CDP-verified entirely **in the sampler** (D-097); the real shells were not touched. Frontend + skin only; no protocol/data change (Rule 5).

**The fold (the design decision).** This is the **`textfield` fold again, not the `range` case.** Through the *sharpened* D-096 (root + value-type + **shared skin/surface**, the N-042 amendment): (1) **root** — all five are `<input type=…>`; (2) **value-type** — plain `bind:value` binds the element's `.value` **string** for every one (`"2026-06-28"` / `"13:45"` / `"2026-06-28T13:45"` / `"2026-06"` / `"2026-W26"`), all **string** (the numeric discriminator that kept `number` separate does not bite); (3) **skin/surface** — identical authored box + prop surface, differing only in UA picker chrome (calendar/clock/both), exactly the textfield situation. Passes cleanly → fold. **Applies D-096, no amendment** (N-046). The honest counter — each type's string is a different *format* — is resolved as textfield did: the getter carries `{ type, value }`, so `type` travels with the value (the N-024 registry). D-096 already pre-named `date` among own atomics *vs* `textfield`; the **new** question resolved here is the *intra-family* one.

**Component + skin.** `ui/core/lib/components/data-independent/date.svelte`: root `<input {type} bind:value use:envelope>`, `value = $bindable('')` (string, **empty=`''`** — the clean divergence from `number`'s `null`), getter `{type,value}`, zero `<style>`; prop surface keeps `value`/`disabled`/`readonly`/`id`/`name`, adds `min`/`max`/`step` (native shaping), drops `placeholder`/`pattern`; plain `bind:value` not `valueAsDate` (reserved); no processor seam. `readonly` is a native pass-through, **not sampler-exercised** (engine-variable) — a flagged build caveat. Own `.date` skin assembled from the `.number` box (keeps `--ctl-h` + `:invalid`→`--err`), `:read-only`, and a recoloured `::-webkit-calendar-picker-indicator`; the picker popup + glyph render dark via the global `color-scheme: dark` (N-043, added *for* this family — now exercised). Sampler: a `date` row + **7** cells (the five types + `#disabled` + `#invalid`), matrix **22 → 29**.

**Verification (Chat self-drove, real `tauri dev` + CDP 9422, both accents via skin-swap).** `ids().length === 29` (7 `date#`). **Fold proof, registry** — every `date#…` reports component `"type":"date"` (one component) carrying its own input type, value a string each:
```
"date#default":{"type":"date","state":{"type":"date","value":"2026-12-31"}},"date#time":{"type":"date","state":{"type":"time","value":"13:45"}},"date#datetime":{"type":"date","state":{"type":"datetime-local","value":"2026-06-28T13:45"}},"date#month":{"type":"date","state":{"type":"month","value":"2026-06"}},"date#week":{"type":"date","state":{"type":"week","value":"2026-W26"}},"date#disabled":{"type":"date","state":{"type":"date","value":"2026-06-28"}},"date#invalid":{"type":"date","state":{"type":"date","value":"2030-01-01"}}
```
**Fold proof, bind path** — dispatched a real `input` (`"2026-12-31"`, N-029) on `date#default`; the bound getter went `value:"2026-06-28"` → `"2026-12-31"` (a JSON **string** round-trips through `bind:value`, the analogue of `number`'s 42/7). Element + skin (one eval):
```
{"elValAfterDispatch":"2026-12-31","style":{"tag":"INPUT","type":"date","minHeight":"28px","fontSize":"12px","color":"rgb(236, 233, 225)","borderRadius":"6px"},"invalid":{"invalidBorder":"rgb(138, 42, 42)","invalidMatches":true,"defaultBorder":"rgb(52, 59, 71)","defaultMatches":false},"dateRules":[".date",".date:focus-visible",".date:disabled",".date:read-only",".date:invalid",".date::-webkit-calendar-picker-indicator"],"shell":"client","accent2":"#c28840"}
```
Computed-style = `--ctl-h`(28px)/`--fs-1`(12px)/`--t`(rgb(236,233,225))/`--rad`(6px). `:invalid` specific — `date#invalid` border `rgb(138,42,42)` (`--err`) + matches; `date#default` stays `rgb(52,59,71)` (`--s5`), no match. All **6** `.date` rules parsed + in cascade (stylesheet-rule inspection — `getComputedStyle` doesn't surface the `::-webkit-calendar-picker-indicator` pseudo, N-042 method). **Skin-swap** — `--accent2` `#c28840` (client) → after `data-shell="node"` `{"shell":"node","accent2":"#3a7ab0"}` (blue). Screenshot (client, scrolled to the row) eye-checked: all five native pickers render (`31/12/2026`, `13:45`, `28/06/2026 13:45`, `June 2026`, `Week 26, 2026`), `#disabled` greyed, `#invalid` red-bordered, indicators dark. Clean teardown (5175/9422 free, 0 orphans).

**Harness note (honest).** The first state-read loop printed “state not ready after retries” while every attempt's output was in fact the full correct registry — a `-match` scoping artifact in my retry wrapper (sibling to the J-422 false-negative), **not** a read failure. The data above is the real returned output.

**Standing rule (the sampler-DoD).** From `date` onward a component milestone is **not done** until its sampler row + applicable-state cells are added and CDP-verified in the sampler — this **replaces** the dual-shell demo-wiring step entirely. Recorded as a one-line closing note on **D-097** (its canonical home) + N-046.

**Canonical (two commits, D-074 UI pattern: `feat` code then `docs` records).** Commit 1 (impl): `ui/core/lib/components/data-independent/date.svelte` (new) + `ui/assets/skin.css` (`.date`) + `ui/sampler/src/app_sampler.svelte` (row). Commit 2 (records): `ui/docs/xgen-ui-notes.md` (N-046) + `docs/ROADMAP.md` (M-RP2.16 ✅, v4.03) + `CLAUDE.md` (PLAY → M-RP2.16, J-424→J-425) + `ui/docs/xgen-ui-components.md` (date row promoted, v0.21) + `DECISIONS.md` (D-097 closing note) + this JOURNAL J-425 + `tasks/M_RP2_16_DATE.md` (→ COMPLETED). Applies D-096 (no fold amendment); the only `DECISIONS.md` touch is the one-line D-097 note. Not pushed — Joe pushes.

**Next-active:** the di track continues, built/tuned in the sampler (D-097) — `color` (own atomic, native chrome) / `file` (new `bind:files` shape) / `select multiple` per N-038 — then the text-processor engine, then dd-components.

---

## Entry J-424 — post-M-RP3.1 tweak: sampler `image#default` shows the real placeholder artwork (inlined data-URI, no file linkage)

**What happened.** A one-line cosmetic swap (Joe-noticed: the sampler's `image#default` showed my plain grey “image” box, not the real placeholder). The bundled `ui/assets/img-placeholder.svg` (the M-RP2.11/J-416 mountains+sun glyph) is now **inlined as a data-URI `src`** in `ui/sampler/src/app_sampler.svelte` — hardcoded per Joe's call (no file import/linkage; the sampler stays self-contained), with explicit `width/height='72'` added because the source SVG has a `viewBox` but no intrinsic size (a sizeless SVG as `<img src>` would default to ~300×150). Faithful usage either way — a data-URI is a normal `src` value; what it skips dogfooding is the bundled-asset path specifically, acceptable for a tuning exhibit. Not a milestone; frontend-only; no protocol/data change (Rule 5).

**Verify (Chat self-drove, CDP 9422).** `image#default`: `complete:true`, `naturalWidth/Height: 72`, `src` head = the inlined `data:image/svg+xml,%3Csvg viewBox='0 0 120 120'…`. Screenshot (scrolled to the row) confirms the real placeholder glyph renders with the `.image` `--rad` corners. Clean teardown (5175/9422 free, 0 orphans).

**Canonical.** `ui/sampler/src/app_sampler.svelte` (the `imgSrc` const); this JOURNAL J-424; `CLAUDE.md` prior-PLAY pointer J-423→J-424. No other records (cosmetic demo-data swap — no N/DECISIONS/ROADMAP/components touch). Its own commit; not pushed — Joe pushes.

**Next-active unchanged:** the component di track resumes — `date` (own atomic, native picker) per N-038, built/tuned in the sampler (D-097).

---

## Entry J-423 — M-RP3.1 CLOSED: Sampler populated — all 10 `core` components live in a 22-cell semantic-group×state grid + polished skin-swap; surfaced an atomic gap (`toggle` has no `disabled`)

**What happened.** Turned the M-RP3.0 scaffold (one `button#smoke`) into the actual tuning surface: all **10 built `core` components** mounted live, **22 `envelope`-registered instances** in a semantic-group×state grid, with a polished client↔node segmented skin-swap. Frontend-only — `ui/sampler/src/app_sampler.svelte` rewrite + `app.css` grid; the `xgen-sampler` crate untouched. No protocol/data change (Rule 5); no `DECISIONS.md` touch (applies D-097/D-098/N-028).

**IA = semantic-group×state, not class×phase.** Phase-0: all 10 are di·A today (no dd, no Phase B/C), so N-028's class×phase axes are degenerate. v1 groups by **Interactive** (toggle/button/textfield/select/textarea/number/range) and **Display** (label/paragraph/image), each a row, its **applicable** states as cells. Class/phase columns activate later when dd/B/C exist.

**Ragged state-map (honest).** default — all 10; disabled — interactive only; invalid — only `textfield` (bad email) + `number` (out-of-range); teaching variants (toggle checked/switch, button toggle-mode, textfield password, textarea `\n`). **No focus column** — focus is transient; a static focus cell would be a lie (verified live instead).

**Atomic gap surfaced (the sampler doing its job).** `toggle` exposes only `checked`/`id`/`shape` — **no `disabled` prop** — so `toggle#disabled` is impossible from the sampler without component work (paused). That cell became **`toggle#switch`** (the switch shape) and the gap is logged (N-045) for the di resume: `toggle` likely wants a `disabled` pass-through for parity. Final matrix = **22** cells, not 23. This is exactly the coverage hole a dedicated exhibit surfaces that demos-in-shells did not.

**Skin-swap = polished segmented control, TOOL CHROME.** A `client | node` segmented control in the bar (styled in `app.css`, active segment uses live `--accent`), deliberately NOT a sampled `core` component — preserves the N-028 tool-vs-sampled line. Flips `:root[data-shell]`; with accent-prominent cells now present (toggle `accent-color`, the latched toggle-mode button), the two shell screenshots **genuinely differ** (unlike the smoke-only scaffold).

**Detail confirmed.** `envelope` keys the registry by `data-debug-id = "type#id"` and does NOT stamp the raw DOM `id` (read of `envelope.ts`), so reusing `id="default"` across component types is collision-free (22 unique `type#id` keys, e.g. `toggle#default` vs `button#default`). `image#default` uses an inline data-URI SVG (no network fetch).

**Verification (Chat self-drove, real `tauri dev` + CDP 9422).** `ids().length === 22`, full list exactly the designed matrix:
```
["toggle#default","toggle#checked","toggle#switch","button#default","button#disabled","button#toggle","textfield#default","textfield#disabled","textfield#invalid","textfield#password","select#default","select#disabled","textarea#default","textarea#disabled","number#default","number#disabled","number#invalid","range#default","range#disabled","label#default","paragraph#default","image#default"]
```
Invalid: `number#invalid` + `textfield#invalid` border-color = `--err` `rgb(138,42,42)`, while `number#default` stays `--s5` `rgb(52,59,71)` (invalid is specific, not blanket). Disabled: `number#disabled`/`button#disabled` `cursor:not-allowed`. Skin-swap: toggle `accent-color` `rgb(154,106,48)` (gold/`--pr`, client) → `rgb(42,96,144)` (blue/`--inf`, node), `--accent` `#9a6a30`↔`#2a6090`. Screenshots both shells render the grid correctly (states + accents read right — checked toggle gold, latched toggle-button gold, invalid red borders + email icon, password dots+icon, disabled greyed, switch shape) and differ in bytes (46225 vs 46182). Clean teardown (5175/9422 free, 0 orphans).

**Canonical (D-074).** `ui/docs/xgen-ui-notes.md` **N-045**; `docs/ROADMAP.md` (UI subtree + RP-node M-RP3.1 ✅, v**4.02**); `CLAUDE.md` PLAY → M-RP3.1, pointer J-422→J-423; `ui/docs/xgen-ui-components.md` test-bed note (populated); `tasks/M_RP3_1_SAMPLER_POPULATE.md` → COMPLETED. No DECISIONS touch. Implementation commit (`app_sampler.svelte` + `app.css`) then records-only. Not pushed — Joe pushes.

**Next-active:** the component **di track RESUMES** — **`date`** (own atomic, structured value / native picker) per N-038, built/tuned **in the sampler** (D-097), then `color` / `file` / `select multiple`, then the text-processor engine, then dd-components.

---

## Entry J-422 — M-RP3.0 CLOSED: Sampler scaffold — `xgen-sampler`, a third Tauri/WebView2 app as the component test-bed; component di track PAUSED (D-097, D-098)

**What happened.** Stood up the **Sampler** — a third, standalone Tauri/WebView2 app whose sole job is to host, tune, and CDP-verify the `core` component library in isolation, with a live client↔node skin-swap. New arc **M-RP3**; this is its scaffold milestone (M-RP3.1 populates the full matrix). The di **component track is paused** (resumable — `date`/`color`/`file`/`select multiple`, then text-processor, then dd-components). Triggered by Joe's call to stop wiring throwaway component demos into both real shells and instead build a dedicated test-bed; the real shells are **frozen as-is** (revert deferred). No protocol/data change; Rust protocol baseline untouched (Rule 5) — the new crate carries no protocol deps.

**Two decisions (the arc's foundation).** **D-097 — test-bed split:** component appearance/state/per-shell theming → the sampler (the skin-swap covers gold-vs-blue, replacing demos-in-both-shells); a component in a real composed feature → the real app at integration; the two shells *running with each other* (federation/handshakes/MP-R) → both real apps together — the sampler's **structural blind spot** (one window, one runtime). **D-098 — sampler runtime = full Tauri/WebView2 sibling (option A), not Vite-in-Chrome:** runs in real WebView2 via a **minimal** host (no protocol deps), same Blink/quirks the skin rests on, same CDP harness; D-095-mirror-exempt, and it diverges on skin *load mechanism* (edits the canonical `ui/assets/skin.css` live via Vite HMR — the killer feature).

**Built (implementation commit).** Frontend `ui/sampler/` — own Vite+Svelte app: `index.html`, `package.json` (`xgen-sampler-ui`, deps svelte/vite/@sveltejs/vite-plugin-svelte; dropped @tauri-apps/* — no IPC), `vite.config.js` (port **5175**, same `$core`/`$common`/`$assets` aliases, outDir `sampler-dist`), `src/main.js` (normalize → skin.css → app.css → mount), `src/app.css` (the two `[data-shell="client"|"node"]` accent-alias blocks + a minimal dev layout, default client), `src/app_sampler.svelte` (plain-JS shell, bare `$state`; a title bar + a `swap skin` flip control + one smoke `Button id="smoke"`). Crate `xgen-sampler/` — minimal host: `Cargo.toml` (`tauri` + `tauri-build` ONLY, no `xgen-common`/`xgen-core`/tokio/etc.), `build.rs` (`tauri_build::build()`), `tauri.conf.json` (devUrl 5175, frontendDist `sampler-dist`, decorated/resizable 960×820 window, identifier `com.alchemydump.xgensampler`), `src/main.rs` (the bare `tauri::Builder::default().run(generate_context!())` — vs `xgen-client`'s heavyweight `desktop::run()` with lifecycle/pipes/WS), `capabilities/default.json` (core-only), icons copied from client. Plumbing: root `Cargo.toml` workspace `members += "xgen-sampler"` (Cargo.lock updated on build); `run-sampler.ps1` (Vite 5175, `cargo tauri dev` in `xgen-sampler`, `-Debug` → `--remote-debugging-port=9422`); `cdp-debug.ps1` taught `sampler`→9422 (ValidateSet + basePort + exe path).

**Skin-swap mechanism.** Per-shell accent is three vars (`--accent`/`--accent2`/`--accent-ink` → `--pr*` client / `--inf*` node) over the shared `skin.css`. The sampler's `app.css` defines BOTH keyed by `:root[data-shell="client"|"node"]` (default client) and flips `document.documentElement.dataset.shell` at runtime — one grid, flip accent, re-theme live.

**Live skin editing (Joe's intent).** No fs-plugin / refresh button in dev: `run-sampler.ps1` runs `tauri dev`, so Vite **HMR** hot-applies every save to the canonical `ui/assets/skin.css` instantly in the WebView2 window. Joe edits the file directly and watches it live; Chat is out of the inner tuning loop, only doing records + commit once a look is settled. A standalone-exe live-reload is a deferred follow-on (D-098).

**Verification (Chat self-drove, real `tauri dev` + CDP on 9422).** v0 mounts ONE smoke component to prove the chain end-to-end. The proof:
```
EVAL RESULT: {"href":"http://localhost:5175/","dbg":"object","ids":["button#smoke"]}
```
— the page loaded from the sampler Vite server **in WebView2**, `window.__XGEN_DEBUG__` exists, and `ids()` lists `button#smoke`: the `$core` import + `envelope` + the debug registry work end-to-end in the brand-new app (the scaffold's load-bearing proof). Note: registry id is **`button#smoke`**, not the runbook's `sampler#smoke` — `envelope` keys by component *type* (`name:'button'`), not by app (naming slip in the runbook, corrected in records). Skin-swap, computed `--accent` on the smoke button:
```
client (default): {"shell":"client","accent":"#9a6a30","bg":"rgb(42, 47, 56)"}
after data-shell=node: {"shell":"node","accent":"#2a6090","bg":"rgb(42, 47, 56)"}
```
— `--accent` flips **gold `#9a6a30`** (`--pr`) ↔ **blue `#2a6090`** (`--inf`) live via the `[data-shell]` attribute. (The button's base `bg` is `--s4` in both — not accent-driven — so the var-resolution is the swap proof, not the button's fill; the screenshots are consequently pixel-identical between shells. M-RP3.1's matrix adds accent-prominent components, e.g. the toggle's `accent-color`, for the visual.) Screenshots both states eye-checked: the bar (title + `accent:` tag + `swap skin` control) + the `button#smoke` cell render cleanly in the 960×820 decorated window. Clean teardown: ports 5175/9422 free, 0 orphans.

**Honest verify note.** The readiness poll loops printed false negatives (a `-match` over the harness output array never set the `$ok` flag, and a `Test-NetConnection` on 5175 raced to `False`); both were harness-loop artifacts, not real failures — the actual CDP reads above (and `location.href`) prove Vite + WebView2 + the registry were all live. The first `tauri dev` was fast because the `tauri`/`wry` deps were already compiled in the shared `CARGO_TARGET_DIR` from the client/node builds; only the tiny `xgen-sampler` crate needed compiling.

**Canonical (D-074).** `DECISIONS.md` **D-097** + **D-098**; `ui/docs/xgen-ui-notes.md` **N-044**; `docs/ROADMAP.md` (UI subtree + RP-node M-RP3.0 ✅ + paused marker, v**4.01**); `CLAUDE.md` PLAY → M-RP3.0 (component track paused), pointer J-421→J-422; `ui/docs/xgen-ui-components.md` test-bed note (no registry change); `tasks/M_RP3_0_SAMPLER_SCAFFOLD.md` → COMPLETED. **DECISIONS.md touched** (D-097/D-098). Implementation commit (frontend + crate + workspace + Cargo.lock + run-sampler.ps1 + cdp-debug.ps1) then records-only commit. Not pushed — Joe pushes.

**Next-active:** **M-RP3.1** — populate the sampler's class×phase matrix with all 10 built components + state columns (default/disabled/invalid/focus) + the polished skin-swap control. Then the component di track resumes (`date` first), built/tuned in the sampler per D-097.

---

## Entry J-421 — post-M-RP2.15 skin fix: `color-scheme: dark` on `:root` — UA-painted native control internals (number spinner, scrollbars, future date/color/file chrome) now render dark; number spinner sized 25% smaller

**What happened.** A one-line skin fix (Joe-reported: the `number` spinner arrows stayed light even in the dark theme). Not a new component, not a milestone. Root cause: the skin styles each control's **box** (bg/border/text), but the UA paints control *internals* — the spinner arrows, scrollbars, and the `date`/`color`/`file` picker chrome still ahead — from the document's **color-scheme**, which defaults to `light` and ignores our box styling. Fix: one declaration **`color-scheme: dark`** added to `:root` in `ui/assets/skin.css` (the L2 token block). It inherits, so it governs every native control at once; idiomatic dark-app fix; keeps the spinner (the M-RP2.14 no-suppression lock holds). **Global, not `.number`-scoped** (Joe's call): the problem is document-level ("native chrome paints light in our dark app"), so it is fixed once at the vocabulary level rather than re-tripped on each native-chromed atomic ahead (forward-looking for `date`/`color`/`file`). No protocol/data change; Rust baseline untouched (Rule 5).

**Verify (both apps, real `tauri dev` + CDP; Chat self-drove).** `getComputedStyle(document.documentElement).colorScheme` → `"dark"` both apps. `toggle` accent-color unaffected: client `rgb(154,106,48)` (=`--pr` gold), node `rgb(42,96,144)` (=`--inf` blue) — the `accent-color`-driven checkbox shape is independent of `color-scheme`, eye-check confirmed it still looks right. The number **spinner** is hover-only-painted by Chromium, so it is not visible at rest in a CDP screenshot (no synthetic hover), but the **scrollbar** — the same UA-painted-chrome mechanism `color-scheme` governs — renders **dark** in both shells in the screenshots, the observable proof the declaration reaches native internals. Clean teardown: `9222=False 9322=False 5173=False 5174=False`, `xgen=0`.

**Canonical.** `ui/assets/skin.css` (`color-scheme: dark` in `:root`; `.number::-webkit-inner-spin-button { transform: scale(0.75) }` — spinner 25% smaller, a follow-on request, same change); `ui/docs/xgen-ui-notes.md` N-043; this JOURNAL J-421; `CLAUDE.md` prior-PLAY pointer J-420→J-421. No `DECISIONS.md` touch (a skin vocabulary addition on the N-031 stack, not a new principle); no `components`/`ROADMAP` touch (no component, no milestone state change). Its own commit; not pushed — Joe pushes.

**Next-active (UI/RP track), unchanged:** the **remaining atomic di** per N-038 — `date` (own atomic, structured value / native picker) next, then `color` / `file` / `select multiple` — then the text-processor engine, then dd-components.

---

## Entry J-420 — M-RP2.15 CLOSED: `range` — tenth `core` component, a stand-alone atomic `<input type="range">` (bounded numeric slider); own atomic on the SHARPENED D-096 fold criterion (→ D-096 amendment); first pseudo-element-heavy skin

**What happened.** Authored the tenth `core` component `range` and skinned it in the same pass — the next atomic di after `number` per the locked N-038 track order (catalogue row *numeric (bounded)*). Mechanically the same `<input>` root **and** the same value-type (number) as `number`, so by the *literal* D-096 criterion (root + value-type) it would fold into `number`. It does **not** — `range` is the case that proves the criterion **necessary but not sufficient**, and the milestone's load-bearing decision is the **D-096 amendment**: the fold test is sharpened to root + value-type + **shared skin/surface** (genuine interchangeability). Implementation (`range.svelte` + `skin.css` + both shells) is its own commit; this is the records-only close. No protocol/data change; Rust baseline untouched (Rule 5).

**Built.** `ui/core/lib/components/data-independent/range.svelte`: root `<input type="range" use:envelope>`; `value = $bindable(0)` typed `number` (always present — the clean divergence from `number`'s `number | null`); getter `() => $state.snapshot({ value })` (always a number); zero `<style>`. Prop surface = the numeric control, slider-shaped: keep `value`/`min`/`max`/`step`/`disabled`/`id`/`name`; **drop** `placeholder` (never empty), `pattern`, `readonly` (native no-op on `type=range`), `type` (fixed). **No clamping** in the atomic — a consumer setting `min > 0` passes an in-range initial (documented consumer responsibility, exactly as `number` does not clamp). **No processor seam** — `range` is a bounded drag, not free-text/free-number entry, so there are no typed digits to reformat (the numeric-formatting consumer is `number`); this is **not** a third defer-per-consumer instance.

**Why own atomic (D-096 amendment, the design point).** `range` shares both halves of D-096's criterion as originally written — root `<input>` AND value-type `number` (same as `number`) — so the *literal* criterion would fold it in. It must not, because the fold's whole value is *genuine interchangeability* (the string-input family shared one skin + one prop surface, switched by a thin `type`). `range` diverges on three axes the fold cannot absorb: (1) **skin** — track/thumb `::-webkit-slider-*` pseudo-elements, **zero** shared appearance with `number`'s text box + spinner; (2) **prop surface** — no `placeholder`, no live `:invalid` (the thumb is clamped, can't go out of range), no `readonly`; bounds are the *defining* attribute; (3) **interaction/empty model** — clamped drag, **always-valued** vs `number`'s empty=`null`. Folding would put two disjoint skins behind one class and a prop that swaps the whole rendering — the polymorphic-contract problem D-096 prevents, on the *appearance* axis. So D-096 gains an amendment clause (criterion = root + value-type + shared skin/surface); the string-input fold still passes the sharpened test, so it is **not** reopened.

**Skin (N-042, first pseudo-element-heavy skin — PROVISIONAL).** Own `.range` key in `skin.css` (after `.number`, before `.select`): `appearance:none` + `-webkit-appearance:none` on the input, then `::-webkit-slider-runnable-track` (a 4px `--s5` groove, pill radius) + `::-webkit-slider-thumb` (16px circle, `margin-top:-6px` to centre on the 4px track, `background: var(--accent, var(--pr))` → per-shell gold/blue, `border: var(--accent2, var(--pr2))`). Vendor-prefixed is fine (single-engine WebView2/Chromium — the toggle-switch `::before` / select-arrow precedent). `:focus-visible` → `--focus-ring`; `:disabled` greys thumb (`--s4`) + track (`--s3`). **No `:invalid`** (clamped), **no `--ctl-h`**, **no new `:root` token**. The **accent fill** (tinted track left of the thumb) is **deferred** — WebKit gives no free fill; a future value-driven `linear-gradient` skin shape (D-065). Demo: one `<Range bind:value={demoRange} id="demo" min={0} max={100} step={1}>` added to both shells (`let demoRange = $state(50)` bare — plain-JS shells, the N-041 gotcha); imported as `Range` (no global shadowing, unlike `number`→`NumberField`).

**Verification (Chat self-drove both apps, real `tauri dev` + CDP).** Registry both apps — `ids()` includes `range#demo`:
```
CLIENT ids: ["toggle#demo","textfield#demo","textfield#demo-search","select#demo","textarea#demo","number#demo","range#demo","label#demo","paragraph#demo","image#demo","button#demo-toggle","button#quit"]
CLIENT range#demo: {"type":"range","state":{"value":56}}   (typeof value === "number")
NODE   range#demo: {"type":"range","state":{"value":50}}
```
Baseline is **always-valued, never `null`** (node read the demo seed `50`; client read `56` — a stray hover/drag landed on the minimized window during the long mount-poll, still a number in range — re-driven cleanly below). Dispatched a real **`input`** event (N-029; range fires `input` on drag) → the registry carried a **JSON number** (not a string):
```
CLIENT readback: {"value":75,"t":"number"}
NODE   readback: {"value":25,"t":"number"}
```
The number-distinguishing proof on the slider bind path (the analogue of `number`'s 42/7). Element computed-style both apps:
```
CLIENT: {"tag":"INPUT","type":"range","appearance":"none","webkitAppearance":"none","width":"160px","cursor":"pointer"}
NODE:   {"tag":"INPUT","type":"range","appearance":"none","webkitAppearance":"none","width":"160px","cursor":"pointer"}
```
Screenshots both apps eye-checked: the slider renders — track groove + **per-shell accent thumb** (gold/`--pr` client at ~75%, blue/`--inf` node at ~25%, matching the dispatched values) + per-shell chrome. Clean teardown: `9222=False 9322=False 5173=False 5174=False`, `xgen=0`.

**Verify finding (N-042 — method, the first slider exposes it; Rule 1/3 recorded honestly).** The planned pseudo-element computed-style probe **did not work**: `getComputedStyle(el, '::-webkit-slider-thumb')` / `'::-webkit-slider-runnable-track'` returned UA defaults, not the authored styles — `{"thumbBg":"rgba(0, 0, 0, 0)","thumbW":"160px","thumbH":"4px","thumbRadius":"0px","trackBg":"rgba(0, 0, 0, 0)","trackH":"4px"}` (thumb width `160px` = the *element* box, not the 16px thumb; bg transparent). These are UA shadow-DOM pseudo-elements; Chromium does not surface author styles on them via `getComputedStyle` (a shadow-pseudo limitation, **not** a timing issue like N-039/N-041). The skin was verified instead by **stylesheet-rule inspection** (walk `document.styleSheets` → `cssRules`, read `.style.cssText` for each `.range…` selector) — all **7** `.range` rules confirmed parsed + in the cascade in both apps (base + track + thumb + focus + disabled + 2 disabled-pseudo, thumb carrying `background: var(--accent, var(--pr))`) — **plus the screenshot** (the accent thumb renders). Going forward, pseudo-element skins are verified via stylesheet-rule presence + screenshot, not `getComputedStyle`.

**Canonical (D-074), records-only.** `DECISIONS.md` D-096 **amendment** (the sharpened criterion: root + value-type + shared skin/surface; Last-updated → 2026-06-27); `ui/docs/xgen-ui-notes.md` N-042 (v0.33); `ui/docs/xgen-ui-components.md` Built `range` row (`{value}`, ref N-022/N-024/N-038/N-042) + detail paragraph + di-catalogue build-note (v0.20); `docs/ROADMAP.md` RP node M-RP2.15 ✅ + both chains + Present clause + frontier (v4.00); `CLAUDE.md` PLAY → M-RP2.15 ✅ CLOSED, pointer J-419→J-420; `tasks/M_RP2_15_RANGE.md` → COMPLETED. **DECISIONS.md IS touched this milestone** (the D-096 amendment) — unlike M-RP2.13/M-RP2.14. Frontier M-RP2.14→M-RP2.15. Implementation in its own commit; records not pushed — Joe pushes.

**Records-commit note (honest record).** The first records commit (`e19ed8e`, pushed) inadvertently went out **without this J-420 entry** — the JOURNAL edit timed out (local MCP server unresponsive) after the other six record files had already been written, and the commit was made before the journal write was reconfirmed. J-420 + the JOURNAL header bump land in a **follow-up records commit**. All other M-RP2.15 records (DECISIONS/notes/components/ROADMAP/CLAUDE/task) were correct in `e19ed8e`.

**Next-active (UI/RP track), per N-038 track order:** the **remaining atomic di** — `date` (own atomic, structured value / native picker) next, then `color` / `file` / `select multiple` — then the text-processor engine (own arc, all consumers in hand), then dd-components. Composites (incl. `password-field` reveal) are the later composite track. `range` is the tenth built `core` component.

---

## Entry J-419 — M-RP2.14 CLOSED: `number` — ninth `core` component, a stand-alone atomic `<input type="number">` (numeric free-entry); first non-string/non-boolean registry value; processor kept DEFERRED (2nd consumer)

**What happened.** Authored the ninth `core` component `number` and skinned it in the same pass — the next atomic di after `textarea` per the locked N-038 track order. Mechanically the same `<input>` root as `textfield`, but a **distinct atomic, NOT a member of the `textfield` `type` fold** (D-096 **held**, not amended): the boundary D-096 drew is *same root + same VALUE-TYPE*, and `number` breaks the second half — Svelte's `bind:value` on `type="number"` coerces to a **number** (`null` when empty), not a string. Folding it in would force `textfield`'s `value` prop polymorphic (`string | number | null`) and defeat the single-typed contract the fold exists to give. So `number` stays its own atomic — the first registry value that is neither boolean (toggle) nor string (everything since). Implementation (`number.svelte` + `skin.css` + both shells) is its own commit; this is the records-only close. No protocol/data change; Rust baseline untouched (Rule 5).

**Built.** `ui/core/lib/components/data-independent/number.svelte`: root `<input type="number" use:envelope>`; `value = $bindable(null)` (type from the lang=ts prop annotation `value?: number | null`); getter `() => $state.snapshot({ value })`; zero `<style>`. Prop surface = the control vocabulary with the numeric bits swapped: keeps `value`/`placeholder`/`disabled`/`readonly`/`id`/`name`; **drops `type`** (fixed) and **`pattern`** (ignored on `type=number`); **adds `min`/`max`/`step`** (native shaping attributes; `step` drives the native-spinner increment — config, not state, not in the getter). The **native spinner is kept** — the UA up/down arrows ARE the atomic's affordance; the custom-button **stepper** is a separate composite (later track), so no `::-webkit-*-spin-button` suppression.

**Processor seam — reserved, NOT built; the second defer-per-consumer instance.** N-038 names `number` as the processor's **numeric-formatting** consumer. Deferred on the same two grounds as `textarea` (N-040): the N-038 sequence builds the engine in its own arc after *all* atomic di (every consumer in hand), and D-065 keeps the atomic free of empty machinery. Header reserves the edit-side `use:processor` insertion point; nothing built. Second reserve-and-defer after `textarea` — a **D-069 promotion-watch**, not yet at the four-recurrence bar (two instances).

**Skin (N-041).** Own `.number` key in `skin.css`, assembled from the M-RP2.7 L2 vocabulary like `.textfield` (the `.select`/`.textarea` per-class precedent). Single-line control — so it **keeps `min-height: --ctl-h`** (unlike `.textarea`) and **keeps `:invalid` → `--err`** (meaningful via native numeric constraint validation: out-of-`min`/`max`, bad `step`). No icon machinery, no `resize`, no spinner suppression, no new `:root` token. Demo: one `<NumberField bind:value={demoNumber} id="demo" placeholder="0" min={0} max={100} step={1}>` added to both shells (imported as `NumberField` to avoid shadowing the global `Number`).

**Verification (Chat self-drove both apps, real `tauri dev` + CDP).** Registry baseline both apps: `number#demo` → `{"value":null}` — **empty input snapshots as `null`** (the Lock-2 expectation, confirmed at runtime, not assumed; matches the locked default). Dispatched a real `input` event (N-029) with a numeric value → registry carried a **JSON number** (parsed Int32, `isNumber=True`): client `value=42`, node `value=7` — NOT the string `"42"`. The number-distinguishing proof (the analogue of textarea's `\n`-survives-the-rune). `:invalid` probe on the live `.number` (`min 0`/`max 100`): out-of-range `999` → `:invalid` true, computed `border-top-color` `rgb(138, 42, 42)` (=`--err`); in-range `42` → valid, `rgb(52, 59, 71)` (=`--s5`). Computed-style both apps: `{"tag":"INPUT","type":"number","minHeight":"28px","fontSize":"12px","color":"rgb(236, 233, 225)","radius":"6px"}` (= `--ctl-h`/`--fs-1`/`--t`/`--rad`). Screenshots both apps — number box renders (value 42; native spinner is UA hover/focus-revealed in Chromium). Clean teardown: `9222=False 9322=False 5173=False 5174=False`, `xgen=0`.

**Build/verify snags caught + fixed live (Rule 1/3 — recorded honestly).** (1) **First mount failed entirely** (empty body, no `__XGEN_DEBUG__`, all components missing): the Vite overlay read `app_client.svelte:51 rune_missing_parentheses`, then after a first fix `js_parse_error` at the `:`. Cause: `$state<number | null>(null)` and a TS type annotation `let x: number | null = …` are both invalid in the **plain-JS app shells** (`app_client.svelte`/`app_node.svelte` `<script>` is not `lang=ts`). Fix: bare `$state(null)` in the shells; in the lang=ts component `$bindable(null)` with the prop-type annotation carrying the type. (2) **Pseudo-class computed-style must be read in a separate CDP task** — a same-task `getComputedStyle` right after the `input` event returned the pre-recalc border (`--s5`) even though `matches(':invalid')` was already true; a second eval round-trip (post-flush) returned the correct `--err`. Sibling to N-039's mid-flush caveat. (3) Vite parse-error overlays don't auto-dismiss on fix — a `location.reload()` over CDP was needed to recover the apps. All three captured in N-041.

**Canonical (D-074), records-only.** `ui/docs/xgen-ui-notes.md` N-041 (v0.32); `ui/docs/xgen-ui-components.md` Built `number` row (`{value}`, ref N-022/N-024/N-038/N-041) + detail paragraph + di-catalogue build-note (v0.19); `docs/ROADMAP.md` RP node M-RP2.14 ✅ + both chains + Present clause + frontier (v3.99); `CLAUDE.md` PLAY → M-RP2.14 ✅ CLOSED, pointer J-418→J-419; `tasks/M_RP2_14_NUMBER.md` → COMPLETED. **No `DECISIONS.md` touch** — D-096 held (applied, not amended); the processor-defer is the application of the existing N-038 sequence + D-065. Frontier M-RP2.13→M-RP2.14. Implementation in its own commit; records not pushed — Joe pushes.

**Next-active (UI/RP track), per N-038 track order:** the **remaining atomic di** — `range` (own atomic, bounded numeric `bind:value`, slider) next **in a new session** (Joe's call), then `date` / `color` / `file` / `select multiple` — then the text-processor engine (own arc, all consumers in hand), then dd-components. Composites (incl. `password-field` reveal) are the later composite track. `number` is the ninth built `core` component.

---

## Entry J-418 — M-RP2.13 CLOSED: `textarea` — eighth `core` component, a stand-alone atomic `<textarea>` (multi-line free-text); the processor seam kept DEFERRED

**What happened.** Authored the eighth `core` component `textarea` and skinned it in the same pass — the next atomic di per the locked N-038 track order. Root tag is `<textarea>`, not `<input>`, so by the N-020 root-tag discriminator this is a **new atomic component, not a `textfield` fold**. It is the **edit-side** multi-line counterpart to `paragraph`'s render-side single prose string (N-032 EDIT-vs-RENDER axis): `paragraph` wraps one read-only string visually; `textarea` holds literal `\n`-bearing editable free text. The walk's load-bearing decision: the milestone **keeps the text-processor seam deferred** — `textarea` ships processor-**ready**, it is **not** the processor's trigger. Implementation (`textarea.svelte` + `skin.css` + both shells) is its own commit; this is the records-only close. No protocol/data change; Rust baseline untouched (Rule 5).

**Built.** `ui/core/lib/components/data-independent/textarea.svelte`: root `<textarea use:envelope>` (N-020); string `bind:value`; getter `() => $state.snapshot({ value })`; zero `<style>`. Prop surface = the `textfield` string-input vocabulary **minus** what `<textarea>` can't carry, **plus** `rows`: keeps `value`($bindable string)/`placeholder`/`disabled`/`readonly`/`id`/`name`; **drops `type`** (no such attribute) and **`pattern`** (`<input>`-only — no `:invalid`-via-pattern path); **adds `rows`** (numeric, default `3` — initial visible height, the one textarea-specific prop). Getter is **value-only** — `rows` is static config, not user-mutable state (`textfield` didn't snapshot `placeholder`). `maxlength` deliberately omitted (mirrors `textfield`).

**Processor seam — DEFERRED, the design decision.** N-038 named `textarea`/`number` as the processor's "earliest natural trigger"; the walk resolved that to **defer** on two locked grounds: (1) the N-038 sequence is locked — *finish ALL atomic di → engine (own arc, all consumers in hand) → dd* — and `textarea` is not the last atomic (`number`/`range`/`date`/`color`/`file`/`select multiple` follow), so building here would over-fit the seam to one of the three named consumers; (2) D-065 — the *atomic* is function-complete without it, exactly as `textfield` shipped processor-ready. The header reserves the **edit-side `use:processor`** insertion point (the counterpart to `paragraph`'s render-side `use:render`); nothing built. **auto-grow** (`field-sizing: content`) is reserved as a future skin shape (like `select`'s `appearance:base-select`), not authored (D-065).

**Skin (N-040).** Own `.textarea` key in `skin.css`, **assembled** from the M-RP2.7 L2 vocabulary like `.textfield` (per-class clarity > DRY, the `.select` precedent — not a shared `.textfield, .textarea` group). Same box (`--s`/`--s5`/`--rad`/`--t`/`--fs-1`/`--lh`/padding/`:focus-visible`/`:disabled`/`:read-only`); differs in **no `min-height: --ctl-h`** (rows drives height), **`resize: vertical`** (horizontal would break the flex-column width), **no per-type icons**, **no `:invalid`**. No new `:root` token. Demo: one `<Textarea bind:value={demoTextarea} id="demo" placeholder="Multi-line text">` added to both shells.

**Verification (Chat self-drove both apps, real `tauri dev` + CDP).** Registry baseline both apps: `textarea#demo` → `{"type":"textarea","state":{"value":""}}`. Dispatched a real `input` event (textarea fires `input`, not `change` — N-029) with a newline-bearing string → client `{"type":"textarea","state":{"value":"line one\nline two"}}` (`hasNewline=True lineCount=2`), node `{"type":"textarea","state":{"value":"node line A\nnode line B"}}` (`lineCount=2`) — the literal `\n` survives the bind rune to the registry snapshot, the proof that distinguishes it from `textfield`. Computed-style both apps: `{"tag":"TEXTAREA","fontSize":"12px","color":"rgb(236, 233, 225)","resize":"vertical","radius":"6px","bg":"rgb(22, 24, 28)","border":"rgb(52, 59, 71)"}` (= `--fs-1`/`--t`/`resize:vertical`/`--rad`/`--s`/`--s5`). Screenshots both apps eye-checked — multi-line box renders with the second line visible + the vertical resize grabber present + per-shell chrome. Clean teardown: `9222=False 9322=False 5173=False 5174=False`, `0 xgen-* remaining`.

**Canonical (D-074), records-only.** `ui/docs/xgen-ui-notes.md` N-040 (v0.31); `ui/docs/xgen-ui-components.md` Built `textarea` row (`{value}`, ref N-022/N-024/N-038/N-040) + detail paragraph + di-catalogue build-note (v0.18); `docs/ROADMAP.md` RP node M-RP2.13 ✅ + both chains + Present clause + frontier (v3.98); `CLAUDE.md` PLAY → M-RP2.13 ✅ CLOSED, pointer J-417→J-418; `tasks/M_RP2_13_TEXTAREA.md` → COMPLETED. **No `DECISIONS.md` touch** — the processor-defer is the *application* of the existing N-038 sequence + D-065, not a new principle (if the defer-per-consumer pattern recurs to the four-recurrence bar it graduates then, D-069). Frontier M-RP2.12→M-RP2.13. Implementation in its own commit; records not pushed — Joe pushes.

**Next-active (UI/RP track), per N-038 track order:** the **remaining atomic di** — `number` (own atomic, numeric `bind:value`) next, then `range` / `date` / `color` / `file` / `select multiple` — then the text-processor engine (own arc, all consumers in hand), then dd-components. Composites (incl. `password-field` reveal) are the later composite track. `textarea` is the eighth built `core` component.

---

## Entry J-417 — M-RP2.12 CLOSED: `textfield` gains a constrained `type` prop — the string-input family folds into one component (reverses N-029 → D-096) + per-type inset icons

**What happened.** Gave `textfield` a constrained `type` prop, folding the structurally-identical string-input family (`text|search|email|url|tel|password`) into the one component, and added a per-type very-weak-grey inset icon (skin). This **reverses N-029** ("type is fixed") — the reversal was pre-authorised by N-038's scoping and now lands with code as **D-096**. Implementation (`textfield.svelte` + `skin.css` + both shells) is its own commit; this is the records-only close. No protocol/data change; Rust baseline untouched (Rule 5).

**Built.** `ui/core/lib/components/data-independent/textfield.svelte`: added `type?: 'text'|'search'|'email'|'url'|'tel'|'password'` (TS union, default `'text'`), root `<input {type}>`, getter now `() => $state.snapshot({ type, value })`, header comment rewritten for the fold; zero `<style>` preserved. Enforcement is the **TS union alone** — no runtime guard, no DEV-warn: an out-of-whitelist value degrades safely (browser → `text`), so a guard would be empty machinery (D-065), and unlike image's `alt` the type system has a safe native fallback. `maxlength` deliberately NOT added (orthogonal to the fold). Password reveal stays OUT of the atomic (interactive chrome → the `password-field` composite, deferred).

**Skin (N-039).** Five per-type inset icons in `skin.css`, keyed `.textfield[type="…"]`, inline-SVG `background-image` right-inset — same mechanism as the `select` arrow; colour literal `%23e6e6e6` (the `img-placeholder` very-weak grey) inside each SVG, no `:root` token. Glyphs: `search` magnifier · `email` envelope · `url` link · `tel` rotary-ish phone · `password` `***`; `text` none. Iconed types carry a right-padding bump (`calc(--sp-4 + --sp-1)`). The native `search` clear-"x" is suppressed (`::-webkit-search-cancel-button { appearance:none }`) so it doesn't collide with the magnifier. Demo: one `<Textfield type="search" id="demo-search">` added to both shells.

**Verification (Chat self-drove both apps, real `tauri dev` + CDP).** Registry both apps: `textfield#demo` → `{"type":"text","value":""}` (default holds, getter now carries `type`), `textfield#demo-search` → `{"type":"search","value":""}`. `el.type` sweep — all six round-trip exactly: `{text,search,email,url,tel,password}` each read back identical (browser accepts each). Per-type computed `background-image`: `text` → `none`, the other five present (lengths 302/349/369/301/220). `bind:value` delta on `type=search` (dispatched `input`) → `{type:"search",value:"find me"}` (client) / `{value:"node find"}` (node) — string bind path holds on a non-text type. `.textfield:invalid` → `--err` `rgb(138, 42, 42)` for **both** native email-type-validation and `pattern` on clean detached elements; valid email + plain text stay `--s5` `rgb(52, 59, 71)`. Screenshots both apps: plain `#demo` iconless, `#demo-search` shows the right-inset magnifier (clear-x gone); client also showed the email state red-border + envelope. Clean teardown (ports 9222/9322/5173/5174 free, 0 orphans).

**Verify finding (N-039).** Probing per-`type` native behaviour by mutating `el.type` on a Svelte `bind:value`-owned `<input>` and reading across an event flush is unreliable — reconciliation + the bind round-trip fight the manual mutation, and one `getComputedStyle(:invalid)` read returned the base border mid-flush (the rendered screenshot + a detached-element test both showed the correct red). Probe per-type via a detached element (or an instance authored with that type) + screenshot; synchronous `el.type`/computed reads with no event dispatched are safe.

**Canonical (D-074), records-only.** `DECISIONS.md` D-096 (the fold decision; Last-updated → 2026-06-25); `ui/docs/xgen-ui-notes.md` N-039 (v0.30, + `→ D-096` on N-038); `ui/docs/xgen-ui-components.md` Built row (`{type,value}`, ref +N-038/N-039) + detail rewrite + di-catalogue build-note (v0.17); `docs/ROADMAP.md` RP node M-RP2.12 ✅ + chain + Present + frontier (v3.97); `CLAUDE.md` PLAY → M-RP2.12 ✅ CLOSED, pointer J-416→J-417; `tasks/M_RP2_12_TEXTFIELD_TYPE.md` → COMPLETED; this JOURNAL J-417. Frontier M-RP2.11→M-RP2.12. Implementation in its own commit; records not pushed — Joe pushes.

**Next-active (UI/RP track), per N-038 track order:** the **remaining atomic di** — `textarea` / `number` / `range` / `date` / `color` / `file` / `select multiple` — then the text-processor engine, then dd-components. Composites (incl. `password-field` with the reveal toggle) are the later composite track. `textfield` now covers six input types; it is still the seventh built component (a fold, not a new one).

---

## Entry J-416 — M-RP2.11 CLOSED: `image` — seventh `core` component, the third/final display-di (atomic `<img>`, `src` + required `alt`); **display-di trio complete**

**What happened.** Authored the seventh `core` component `image` and skinned it in the same pass — the **third and final display-di**. The trio (label/paragraph/image) is now complete. Implementation in its own commit (new asset `img-placeholder.svg` + `image.svelte` + both shells + `skin.css`); this is the records-only close. No protocol/data change; Rust baseline untouched (Rule 5).

**Built.** `ui/core/lib/components/data-independent/image.svelte`: atomic `<img use:envelope>` (N-020); props `src: string` + `alt: string` (**both required, no default**) + `id`; getter `() => $state.snapshot({ src, alt })`; zero `<style>`. **Structural novelty:** `<img>` is a **void element** — the first display-di whose value lives in an **attribute** (`src`), not a text-node body (label/paragraph put the value in their content). The read-only pattern otherwise carries over verbatim.

**Required `alt` (the design point).** `alt: string` typed non-optional, no default — the consumer must consciously pass it, including `alt=""` for a deliberately decorative image (valid + conscious). The requirement forces the a11y decision, not forbid empty. A DEV-only `console.warn` fires if `alt === undefined`; no prod throw. `src` likewise required. Getter carries `{src, alt}` (two fields — `alt` is part of the contract; precedent: a display-di getter carries what the semantic demands, not always one).

**Skin.** `.image { border-radius: var(--rad); }` — an image's look is intrinsic, the skin just frames; **no new token**; sizing is a consumer concern. New bundled asset `ui/assets/img-placeholder.svg` (Joe-approved neutral grey placeholder: grey square + light frame/sun/two-peaks glyph), imported via `$assets` in both shells — the first asset-backed demo; Vite inlined the sub-threshold SVG as a data-URI.

**Verification (Chat self-drove both apps, real `tauri dev` + CDP).** Registry: `image#demo` → `{"type":"image","state":{"src":"data:image/svg+xml,%3csvg…","alt":"Image placeholder"}}` (client 9222 / node 9322). Computed-style (both): `tag IMG`, `border-radius 6px` (=`--rad`), `display block`, `complete true`, `alt "Image placeholder"`. Screenshots both apps eye-checked — the placeholder renders (grey square, light glyph, rounded corners; stretched to column width — no width constraint, sizing deferred). Clean teardown (ports 9222/9322/5173/5174 free, 0 orphans).

**Canonical (D-074), records-only.** `ui/docs/xgen-ui-notes.md` N-037 (v0.28); `ui/docs/xgen-ui-components.md` Built row + detail (v0.16); `docs/ROADMAP.md` RP node M-RP2.11 ✅ + frontier (v3.96) — display-di trio complete; `CLAUDE.md` PLAY → M-RP2.11 ✅ CLOSED, Next → first composites, pointer J-415→J-416; `tasks/M_RP2_11_IMAGE.md` → COMPLETED; this JOURNAL J-416. Frontier M-RP2.10→M-RP2.11. Implementation in its own commit; records not pushed — Joe pushes.

**Next-active (UI/RP track):** the **first composites** — `textfield-group` (label + textfield, where `for`/association lands) and `combobox` (`textfield` + `datalist`) — the di→composite transition. The seven atomic `core` components (toggle/button/textfield/select/label/paragraph/image) are the building blocks the composites assemble.

---

## Entry J-415 — M-RP2.10 CLOSED: `paragraph` — sixth `core` component (second display-di, atomic `<p>` prose); the `--fs-*` type scale founded; render-side formatter seam reserved

**What happened.** Authored the sixth `core` component `paragraph` and skinned it in the same pass, AND founded the `--fs-*` type-size scale (deferred here from M-RP2.9). Second of the display-di trio (after `label`) — reuses the read-only display-di pattern verbatim. Implementation in its own commit (`paragraph.svelte` + both shells + `skin.css` tokens/retro-key/skin); this is the records-only close. No protocol/data change; Rust baseline untouched (Rule 5).

**Built.** `ui/core/lib/components/data-independent/paragraph.svelte`: atomic `<p use:envelope>` (N-020); value prop **`text`** (plain, the display-di semantic name shared with label); `id`; getter `() => $state.snapshot({ text })`; body the **text node** `{text}`; zero `<style>`. Identical shape to `label` — the read-only pattern generalizes unchanged to a second tag. Demo wired both shells (no `$state` var, read-only).

**Formatter seam (reserved, not built — the design point).** Body is a plain text node today (`{text}`), never `{@html}` — safe by default. The future inline-mark formatter (`_x_`/`*x*`, whitelist `<strong>`/`<em>`/`<br>`, escape char) lands as a `common` `use:render` action — the render-side counterpart to the edit-side `use:processor` (EDIT-vs-RENDER axis, N-032); that action owns the delimiter map + whitelist + sanitisation and rewrites node content only when applied. Not built now (D-065); documented insertion point.

**`--fs-*` type scale founded.** Until now every component hardcoded `font-size: 12px`. Founded in `skin.css`: `--fs-1: 12px` (control/caption) + `--fs-2: 14px` (body prose) + `--lh: 1.5`. **Pair only, no `--fs-3`/`--fs-4` seed** (D-065). The four shipped skins retro-keyed in the same pass (`12px`→`var(--fs-1)`, `1.5`→`var(--lh)`, 8 substitutions); all stay zero-`<style>`. `.paragraph` = `font-size: var(--fs-2)`, `color: var(--t)` (brightest — prose is content, vs label's caption `--t2`), `line-height: var(--lh)`, `margin-block-end: var(--sp-3)`.

**Verification (Chat self-drove both apps, real `tauri dev` + CDP).** Registry: `paragraph#demo` → `{"type":"paragraph","state":{"text":"Demo paragraph of prose."}}` (client 9222 / node 9322). Computed-style (both): `.paragraph 14px/21px rgb(236, 233, 225)` (=`--fs-2` / `--t`). **Retro-key re-verified non-regressive** — all four at 12px/18px: `.button` rgb(200,196,188), `.textfield` rgb(236,233,225), `.select` rgb(236,233,225), `.label` rgb(200,196,188). Screenshots both apps eye-checked — the paragraph renders visibly larger + brighter than the label caption above it. Clean teardown (ports 9222/9322/5173/5174 free, 0 orphans).

**Canonical (D-074), records-only.** `ui/docs/xgen-ui-notes.md` N-036 (v0.27); `ui/docs/xgen-ui-components.md` Built row + detail (v0.15); `docs/ROADMAP.md` RP node M-RP2.10 ✅ + frontier (v3.95); `CLAUDE.md` PLAY → M-RP2.10 ✅ CLOSED, Next → `image`, pointer J-414→J-415; `tasks/M_RP2_10_PARAGRAPH.md` → COMPLETED; this JOURNAL J-415. Frontier M-RP2.9→M-RP2.10. Implementation in its own commit; records not pushed — Joe pushes.

**Next-active (UI/RP track):** `image` (root `<img>`, `src` + required `alt`, Phase A — N-032) — completes the display-di trio — then the first composites (`textfield-group` = label + textfield; `combobox` = `textfield` + `datalist`).

---

## Entry J-414 — M-RP2.9 CLOSED: `label` — fifth `core` component, the first DISPLAY-kind di (atomic `<label>`, read-only caption), authored + skinned in one pass

**What happened.** Authored the fifth `core` component `label` and skinned it in the same pass — the **first display-kind di**. The four built so far (toggle/button/textfield/select) are *interactive* (input/event, live getter delta); `label` is **value-carrying but read-only**, the display half of the di model (N-032). Founds the read-only display-di pattern `paragraph`/`image` inherit. Implementation in its own commit (`label.svelte` + both shells + `skin.css`); this is the records-only close. No protocol/data change; Rust baseline untouched (Rule 5).

**Built.** `ui/core/lib/components/data-independent/label.svelte`: atomic `<label use:envelope>` (N-020); value prop **`text`** (plain, **not** `$bindable`); `id`; debug getter `() => $state.snapshot({ text })`; body `{text}`; zero `<style>`. Demo `label` wired into both shells (beside demo toggle/textfield/select/button) — no `$state` var (read-only). `.label` skinned in `skin.css` from L2 (`color: var(--t2)`, `font-size: 12px`, `line-height: 1.5`) — **no new token**; all five built components remain zero-`<style>`.

**The five walk locks (Joe, design walk this session).** (1) value prop **`text`** not `value` — `value` is the editable/`$bindable` marker; display-di take a *semantic* value-name (label/paragraph = `text`, image = `src`). (2) **No `for`** — association is a composite concern (N-032), wired by `textfield-group`; standalone label valid-but-inert. (3) **getter registered anyway** — registry stays uniform (N-030 §4); founds the display-di **verify pattern**: no event to dispatch, verify = snapshot returns the value + computed-style probe. (4) **skin assembles, no new token** — the `--fs-*` type scale deferred to `paragraph` (M-RP2.10), where two text components justify founding it + retro-keying the shipped skins. (5) **`use:envelope` unchanged** — content-agnostic substrate reused verbatim; the substrate generalizes across the interactive/display fault line.

**Verification (Chat self-drove both apps, real `tauri dev` + CDP).** Registry: `label#demo` → `{"type":"label","state":{"text":"Demo label"}}` (client 9222 / node 9322) — the first display-di registered cleanly beside the four interactive di + the demo-toggle button. Computed-style probe (both apps): `color: rgb(200, 196, 188)` (=`--t2` #c8c4bc), `font-size: 12px`, `line-height: 18px`. Screenshots both apps eye-checked — the dim caption renders between the select and the toggle button. Clean teardown (ports 9222/9322/5173/5174 free, 0 orphans).

**Finding (Rule 1 — surfaced in verify).** The probe returned `display:block` on `.label`; investigation showed both `<body>` (flex-row) and `<main#core-ui-pane>` (flex-column) are flex containers, so the label is a **flex item** — CSS blockifies a flex item's computed `display` to `block` regardless of its own value (a bare `<label>` appended to the flex `<body>` reported `block` too). The `.label` skin sets **no `display`**; the UA inline default and the N-032 "inline default" framing both stand — the block is **environmental** (the shell's flex layout), not the component. Recorded so it is not misread when `paragraph`/`image` are verified the same way.

**Canonical (D-074), records-only.** `ui/docs/xgen-ui-notes.md` N-035 (v0.26); `ui/docs/xgen-ui-components.md` Built row + detail (v0.14); `docs/ROADMAP.md` RP node M-RP2.9 ✅ + frontier advance (v3.94); `CLAUDE.md` PLAY → M-RP2.9 ✅ CLOSED, Next → `paragraph`, pointer J-413→J-414; `tasks/M_RP2_9_LABEL.md` → COMPLETED; this JOURNAL J-414. Frontier M-RP2.8→M-RP2.9. Implementation in its own commit; records not pushed — Joe pushes.

**Next-active (UI/RP track):** `paragraph` (root `<p>`, single-paragraph prose, inline-mark formatter seam reserved — N-032; founds the `--fs-*` type scale), then `image` (root `<img>`, `src`+`alt`) — completing the display-di trio — then the first composites (`textfield-group` = label + textfield; `combobox` = `textfield` + `datalist`).

---

## Entry J-413 — M-RP2.8 CLOSED: `select` — fourth `core` component (di·A, single-select, atomic `<select>`), authored + skinned in one pass; first content-carrying di

**What happened.** Authored the fourth `core` component `select` and **skinned it in the same pass** — the first author-and-skin-in-one-pass milestone (the L2 vocabulary founded at M-RP2.7/N-033 made it possible) and the first **content-carrying** di component. Implementation in its own commit (`select.svelte` + both shells + `skin.css`); this is the records-only close. No protocol/data change; Rust baseline untouched (Rule 5).

**Built.** `ui/core/lib/components/data-independent/select.svelte`: atomic `<select use:envelope>` (N-020); `options` prop accepting `string[]` **or** `{value,label,disabled?}[]` normalized internally to one shape; optional `placeholder` → leading disabled `<option value="">`; `bind:value` (string); native-state `disabled`/`id`/`name`/`required`; debug getter `() => $state.snapshot({ value })`; zero `<style>`. Demo `select` wired into both shells (beside demo toggle/textfield/button). `.select` skinned in `skin.css` from L2 tokens (`--s`/`--s5` box, `--rad`, `--ctl-h`, `--sp-*` padding, accent-tinted focus ring, disabled grey, `:invalid`→`--err`) + `appearance:none` + inline-SVG `background-image` arrow (root stays `<select>`, L1 empty — all four built components remain zero-`<style>`). The open option-list popup is left native (Q3).

**The content-carrying precedent.** Where toggle/button/textfield are pure native-state, `select` is the first di component carrying *list content*. Locked shape: a normalized `options` prop (not slotted children) — keeps the root atomic and the component data-independent, and is the same surface the dd layer will later feed. This is the pattern future content-carrying di (and dd) components follow.

**Verification (Chat self-drove both apps, real `tauri dev` + CDP).** `select#demo` baseline `{value:""}` → set value + dispatched a real `change` event → `{value:"beta"}` (client 9222) / `{value:"gamma"}` (node 9322): the bind-in live-reactive read on a content-carrying control. `optionCount:4` (placeholder + alpha/beta/gamma); computed style `appearance:none`, radius `6px` (`--rad`), inline-SVG arrow present; both apps eye-checked (chevron, box matches the textfield, per-shell chrome). Clean teardown (ports 9222/9322/5173/5174 free, 0 orphans). N-029 finding restated for `change`: driving `bind:value` over CDP needs a dispatched `change` event, not a bare `el.value=`.

**Canonical (D-074), records-only.** `ui/docs/xgen-ui-notes.md` N-034 (v0.25); `ui/docs/xgen-ui-components.md` Built row + detail (v0.13); `docs/ROADMAP.md` RP node M-RP2.8 ✅ + frontier advance (v3.93); `CLAUDE.md` PLAY → M-RP2.8 ✅ CLOSED, Next → display-di `label`, pointer J-412→J-413; `tasks/M_RP2_8_SELECT.md` → COMPLETED; this JOURNAL J-413. Frontier M-RP2.7→M-RP2.8. Implementation in its own commit; records not pushed — Joe pushes.

**Next-active (UI/RP track):** display-di `label` (root `<label>`, caption — first of the display-di trio, identities locked N-032), then `paragraph`/`image`, then first composites (`combobox` = `textfield` + `datalist`; `textfield-group`).

---

## Entry J-412 — M-RP2.7 CLOSED: first skin pass — N-031 CSS stack stood up + L2 vocabulary founded; `button{}` wrinkle closed; per-shell accent; switch skin-only — eye- + CDP-verified BOTH apps

**What happened.** Chat self-drove the full M-RP2.7 implementation (Ms Design seat) + the live verification loop in both apps per the runbook `tasks/M_RP2_7_FIRST_SKIN_PASS.md`. The first skin pass stood up the N-031 CSS source stack and founded the L2 token+treatment vocabulary; the N-028/N-029 global `button{}` wrinkle is closed. No protocol/data change; Rust test baseline unchanged (~1466/0, not re-run — Rule 5). Implementation landed in its own commit (skin stack + `cdp-debug.ps1` screenshot mode + the accent bug fix); this is the records-only close.

**Built (Ms Design, Phase 1–2).** Relocated `ui/modern-normalize.css` → `ui/assets/modern-normalize.css` (pristine L0, never edited). New `ui/assets/xgen-normalize.css` (L0 adapted floor: `*`/`main`/`p`/`img` floor + the native-`button`-flattening reset, migrated out of `app.css`). New `ui/assets/skin.css` (L2: semantic palette canonical here, `@font-face` from shared `ui/assets/fonts/`, radius/spacing scale, accent-tinted focus ring, `.button`/`.toggle`/`.textfield` keyed appearance, `[aria-pressed="true"]` accent + `:active` bevel, switch via `.toggle[role="switch"]` `appearance:none` + `::before` thumb). Added `$assets` Vite alias to both shells; rewired both `main.js` import chains (modern-normalize → xgen-normalize → skin → app.css). Gutted both `app.css` to shell chrome + per-shell `--accent*` alias (client gold/`--pr`, node blue/`--inf`). All zero-`<style>` invariant on the three components preserved (Q5 **skin-only**, locked).

**Verification (Chat self-drove real Vite + `tauri dev` + CDP, both apps; added `-Mode screenshot` to `cdp-debug.ps1`).** Wrinkle-clearance computed-style probe (client):
```
bareButton:    {bg:rgba(0,0,0,0), bw:0px,   rad:0px, pad:0px,  app:none}   <- normalize-flat
skinnedButton(.button #quit): {bg:rgb(42,47,56)=--s4, rad:6px, pad:20px}   <- skinned
tabSize:"4"  (UA default 8 -> modern-normalize loaded)   sheetCount:4
```
Accent resolves per-shell (forced-substitution probe): client `var(--accent)` → `rgb(154,106,48)`=`--pr` gold; node → `rgb(42,96,144)`=`--inf` blue; `rootRules:2` (skin + app.css `:root` both in cascade). `[aria-pressed="true"]` shows accent fill (node demo-toggle latched → blue, screenshot-confirmed); `.toggle[role="switch"]` → `appearance:none`, 40px track, visible pill+thumb in both apps. Both apps eye-checked via captured PNGs — coherent render (logo, state dot, switch, accent focus ring, accent-latched toggle, base-grey terminal button). Clean teardown: ports 9222/9322/5173/5174 all free, 0 orphans.

**Finding (Rule 1/3 — caught in verify, fixed before close).** Both `app.css` header comments contained `(--s*/--t*/--pr*/--inf*/--ok/--err)`; the `--s*/` substring forms `*/`, **closing the C-style comment early** and dropping the leading `:root{--accent…}` rule (parser recovered at `html,body`). Result: `--accent` undefined at runtime — the client's initial "gold" was actually the skin's `var(--accent, var(--pr))` *fallback*, not real accent. Diagnosed via the stylesheet map (app.css sheet started at `html,body`, not `:root`) + a `var(--accent, rgb(1,2,3))` sentinel returning the sentinel. Comment corrected in both shells; re-verified resolving (gold/blue). A real bug surfaced only because the skin pass was verified rather than assumed.

**Q5 locked — switch skin-only.** The `appearance:none` + `::before`-thumb switch renders cleanly as a pill+thumb in both apps (single-engine WebView2/Chromium target removes the historical pseudo-on-form-control risk). No L1 scaffold needed; `toggle.svelte` stays `<style>`-free — all three built components remain zero-L1.

**Canonical (D-074), records-only.** `ui/docs/xgen-ui-notes.md` (N-033 skin vocabulary founded + the `*/`-comment finding, v0.23→0.24); `ui/docs/xgen-ui-components.md` (skin-shipped note on the three components, v0.11→0.12); `docs/ROADMAP.md` (RP node M-RP2.7 ✅, frontier advance, v3.91→3.92); `CLAUDE.md` (PLAY head → M-RP2.7 ✅ CLOSED, Next → `select`, entry pointer J-411→J-412); `tasks/M_RP2_7_FIRST_SKIN_PASS.md` (Status ACTIVE→COMPLETED, DoD checked); this JOURNAL J-412. Frontier advances M-RP2.6→M-RP2.7. Implementation in its own commit; records not pushed — Joe pushes.

**Next-active (UI/RP track):** `select` (di·A, atomic `<select>`, pick-only) — the next basic; assembles its skin from the now-founded L2 vocabulary. Then display-di `label`/`paragraph`/`image` (N-032) → first composites (`combobox` = `textfield` + `datalist`; `textfield-group`).

---

## Entry J-411 — Records-only (conceptual): display-di identities locked — `label` / `paragraph` / `image` + the edit-vs-render processor axis (N-032)

**What happened.** Records-only design-capture, conceptual — **none built**, no code, no protocol/data change, arc order unchanged (M-RP2.7 still next). A design conversation settled the **display half** of the di model and some naming. Captured as N-032.

**Display-di trio (identity by root tag, N-030).** The three built components are *interactive* (input/event, live getter state); these three are **display-kind di** — value-carrying but **read-only**:
- **`label`** — root `<label>` — short caption naming another control. Over `<span>` (inline, semantically empty) and `<p>` (a different component). Association (`for`/nesting) is a **composite** concern, not the atom — lean implicit nesting at `textfield-group`; standalone-without-control is a tolerated inert edge case; block-level = skin, not tag.
- **`paragraph`** — root `<p>` — a **single** paragraph of prose (not multi-paragraph). Named over `text` (too generic) / `textblock` ("block" oversells it). Scalar string value; renders through an **inline-mark formatter seam reserved but pass-through today** (WordStar/markdown-lineage `_x_`/`*x*`; delimiter map + escape char TBD; inline whitelist only → trivial sanitisation; links the one risky add).
- **`image`** — root `<img>` — value = `src`, with **required `alt`**. `<img>` for content images; decorative imagery = CSS `background-image` (skin, not a component).

**Two text processors on the EDIT-vs-RENDER axis** (not "dynamic/static"): edit-side (live, bound, `textarea`/`textfield` — the earmarked `use:processor`) vs render-side (read-only, `paragraph`). Both reusable `common` actions, thin seams now.

**Parked (explicitly not these):** multi-paragraph `textblock`/`richtext` (root `<div>` + sanitiser); the edit-side `textarea` processor. Naming `paragraph` frees `textblock` for the parked richtext.

**Earlier in-thread (no record needed):** reaffirmed the 0/1 switch = `toggle` switch-shape (root `<input type=checkbox>`, not its own component); a multi-position selector = a **`select`** segmented-shape (root `<select>`), not `combobox` (which is `textfield` + `datalist`, free-text-with-suggestions).

**Canonical (D-074), records-only.** `ui/docs/xgen-ui-notes.md` (N-032, v0.22→0.23); `ui/docs/xgen-ui-components.md` (queued display-di prose named with roots, v0.10→0.11); `docs/ROADMAP.md` (RP node display-di mention); CLAUDE PLAY entry pointer J-410→J-411; this JOURNAL J-411. No frontier/arc change. Not pushed — Joe pushes.

---

## Entry J-410 — M-RP2.6 CLOSED: `button` retrofit (`ariaLabel` + `pressed`/toggle-mode) + `toggle` `shape` (switch `role`) — additive, skin-free, CDP-verified in BOTH apps

**What happened.** Clair implemented the M-RP2.6 runbook (`tasks/M_RP2_6_BUTTON_RETROFIT.md`); Chat self-drove the full `tauri dev` + CDP verification in both apps. Purely **additive** retrofit of two shipped `core` components — the first reopen of shipped components (N-030) — and **skin-free** (all shapes/looks remain M-RP2.7). No `$common` change, no protocol/data change; Rust test baseline unchanged (~1466/0, not re-run — Rule 5). Code landed in `c1e2f44`; this is the records-only close.

**Built (Clair, additive).** `button.svelte`: +`ariaLabel` (→ `aria-label`), +`mode` (`'momentary'` default / `'toggle'`), +`pressed` (`$bindable(false)`); `handleClick` flips `pressed` in toggle-mode then fires `onclick`; getter → `$state.snapshot({ clicks, disabled, pressed })`; root gains `aria-label={ariaLabel||undefined}` + `aria-pressed={mode==='toggle' ? pressed : undefined}`. `toggle.svelte`: +`shape` (`'checkbox'` default / `'switch'`); root gains `role={shape==='switch'?'switch':undefined}` + `aria-checked={shape==='switch'?checked:undefined}`; getter unchanged (`{checked}` — shape is a static prop). Both shells: existing `toggle#demo` set `shape="switch"`; one throwaway `button#demo-toggle` (`mode="toggle"`, `bind:pressed`) added; real Quit/Shut-Down untouched. Static gate: no `svelte-check`/`tsc` in this toolchain (shells are plain JS) — Clair ran `vite build` (the Svelte compiler over every module) clean in both shells, 119 modules each, 0 errors/0 warnings.

**Live verification (real Vite + `tauri dev` + CDP; Chat self-drove per the N-028 working mode).** Launched both detached (`run-{client,node}.ps1 -Debug`), polled ports (client 9222 / node 9322), dumped registry, drove real events, cleaned up. Baseline registry held all four components incl. the new `button#demo-toggle` carrying `pressed`.

*Pressed-latch (headline — the event-driven self-redump the terminal Quit/Shut-Down could not do, N-028 finding 1):* one `b.click()` then another, registry re-read each time —
```
client: {"clicks":4,"disabled":false,"pressed":false} || {"clicks":5,"disabled":false,"pressed":true} || {"clicks":6,"disabled":false,"pressed":false}
node:   {"clicks":2,"disabled":false,"pressed":false} || {"clicks":3,"disabled":false,"pressed":true} || {"clicks":4,"disabled":false,"pressed":false}
```
Each click: `clicks` +1 AND `pressed` latches true→false on alternate clicks — self-redumped live on the same instance. (Honest note, Rule 1: the non-zero baselines — client 4, node 2 — are stray clicks on the live window during the ~150 s first-build/poll wait; the per-click **delta** is the proof, and it is clean.)

*Switch role/aria (`toggle#demo`):*
```
client/node BEFORE: {"role":"switch","ariaChecked":"false","checked":false}
client/node AFTER : {"role":"switch","ariaChecked":"true","checked":true,"reg":{"checked":true}}
```
`role="switch"` persists across the state change; `aria-checked` correctly **reflects** the `checked` bool (false→true), matching the registry — N-030 §4 one-source/many-projections.

*Momentary regression (DOM read — not clicked, since Quit/Shut-Down are terminal):*
```
client: {"id":"button#quit","ariaPressed":null,"hasAttr":false}
node:   {"id":"button#shutdown","ariaPressed":null,"hasAttr":false}
```
Momentary buttons carry **no** `aria-pressed` — correct (it would be a lie on a non-latching button).

*Cleanup:* ports 9222/9322/5173/5174 all closed afterward; 0 `xgen-client`/`xgen-node` orphans.

**Working-tree note (Rule 3).** A benign `xgen-node/Cargo.toml` modification surfaced post-build — `git diff` shows **no content delta**, only an `LF→CRLF` normalization warning (a line-ending churn artifact of `tauri dev`/cargo touching the file). Not staged, not part of this close; Joe can `git checkout` it or ignore.

**Canonical (D-074), records-only.** `CLAUDE.md` (PLAY head → M-RP2.6 ✅ CLOSED, Next-active → M-RP2.7 first skin pass, entry pointer J-409→J-410); `ui/docs/xgen-ui-components.md` (button getter → `{clicks,disabled,pressed}` + mode note, toggle shape/role note, shape-family prose → built; v0.9→0.10); `docs/ROADMAP.md` (RP node Shipped + M-RP2.6 ✅, Next → M-RP2.7; v3.89→3.90); `tasks/M_RP2_6_BUTTON_RETROFIT.md` (Status ACTIVE→COMPLETED, DoD checked); this JOURNAL J-410. Frontier advances M-RP2.5→M-RP2.6. Code in `c1e2f44`; records not pushed — Joe pushes.

**Next-active (UI/RP track):** M-RP2.7 — first skin pass (N-031 CSS source stack). Needs a design walk (skin.css home/path, accent gold/blue one-skin-vs-two, normalize wiring) → runbook before code.

---

## Entry J-409 — Records-only: CSS source stack locked (N-031) + leading arc locked (M-RP2.6 `button` retrofit → M-RP2.7 first skin pass)

**What happened.** Records-only. A design conversation following J-408/N-030 settled the CSS architecture ahead of the first skin pass, and locked the leading UI/RP arc. No code, no protocol/data change; test baseline unchanged (~1466/0, not re-run — Rule 5). Captured as N-031; PLAY + ROADMAP + registry updated to match.

**CSS source stack locked (N-031).** Four-source ordered cascade — three global files + one per-component channel: **L0 `modern-normalize.css`** (pristine upstream cleaner, per-tag, never edited, version-bumpable) → **L0 `xgen-normalize.css`** (our adapted element-generic floor, per-tag, deviations recorded in-file) → **L1 scoped `<style>` in each `.svelte`** (construction/structural, per-component, as-needed, appearance-neutral, frequently empty) → **L2 one `skin.css`** (all appearance, keyed by type-class, the single removable layer + live-swap target). This **refines N-021** (its single "normalize.css" becomes the two-file split — pristine import + adapted deltas; "trim" becomes "override," never deletion) and **operationalises N-025** (structural-vs-skin restated as the remove-the-rule litmus + the baseline generic→L0 / specific→L1 second cut). Saturation (Joe): L0 saturates by tag-count (~15 native tags), L1 barely grows (mostly empty; reuse not re-rule), L2 saturates by **vocabulary** (tokens + shared treatments defined once, then assembled) — N-019 write-once applied to styling. Consequence: the first skin pass is a **vocabulary-founding** pass and reconciles the N-028/N-029 global `input{}`/`button{}` wrinkle (generic reset → xgen-normalize; `:root` tokens + appearance → skin.css; `app.css` gutted to shell chrome).

**Leading arc locked.** Per Joe's J-408 reframe (shipped components before the next basic), the arc is **M-RP2.6 `button` retrofit** (additive `ariaLabel`→`aria-label`; `pressed`/toggle-mode — one inner bool, momentary default / toggle latches bind-out `pressed`, `aria-pressed` toggle-mode-only; debug getter → `{clicks,disabled,pressed}`; `toggle` gains a conditional `role="switch"`; Quit/Shut-Down stay momentary) **→ M-RP2.7 first skin pass** (stand up the N-031 stack, reconcile the wrinkle, render the icon/switch/pressed shapes, found the L2 vocabulary). Then `select` (di·A) → `label`/`image` (display-di) → first composites (`combobox` = `textfield` + `datalist`; `textfield-group`). Sequenced retrofit→skin: the skin keys on attributes the retrofit introduces, and the retrofit is independently CDP-verifiable (a toggle-mode button self-redumps its `pressed` delta — the event-driven self-redump the terminal buttons could not, N-028 finding 1).

**Open for the M-RP2.6/2.7 design walk / runbook:** `toggle` `role="switch"` needs a component signal (a `shape`/`variant` prop — role is semantic, not skin-drivable); `skin.css` home/path (candidate `ui/assets/`, per the `skin-*.css`+`tokens.css` precedent in `ui/templates/skeleton/` + `ui/backup/run_1.0/`); whether `modern-normalize.css` is wired today (the shells' `app.css` hand-rolled reset currently does the L0 job); accent gold/blue as one skin + shell-set token vs two skins. No code before a Joe-approved runbook (D-071).

**Canonical (D-074).** `ui/docs/xgen-ui-notes.md` (N-031 + in-place pointers on N-021/N-025, v0.21→0.22); `ui/docs/xgen-ui-components.md` (CSS-source-stack pointer on the shape-family prose, v0.8→0.9); `docs/ROADMAP.md` (RP node Next → locked arc + N-030/N-031 in records, v3.88→3.89); `CLAUDE.md` (PLAY Next-active → arc locked, entry pointer J-408→J-409); this JOURNAL J-409. No ROADMAP frontier change (M-RP2.5 ✅ unchanged; the lock is within the open RP queue). Not pushed — Joe pushes.

---

## Entry J-408 — Design-capture (N-030) + reprioritization: shape families on the built components; `button` retrofit + first skin file now lead the queue

**What happened.** Records-only. A design conversation after J-407 (ahead of `select`) settled several component-model questions and surfaced that two **shipped** components gain additive surface. No code, no protocol/data change; test baseline unchanged (~1466/0, not re-run — Rule 5). Captured as N-030; registry + PLAY updated to match. All conclusions driven by the root-tag lens (N-020) + shape-is-skin (N-019/N-025).

**Conclusions recorded (N-030):**
- **Boolean family splits by root tag.** `<input type=checkbox>` → `toggle` (shapes: checkbox / switch = skin). `<button>` → `button` toggle-mode. A button-style boolean toggle is **not** a shape of `toggle` (different tag) — it lives on `button`. UX-identical; the cut is form-semantics (form value → checkbox; in-app pressed action → button toggle-mode).
- **`button` retrofit (additive, on a shipped component):** optional `ariaLabel` (→ `aria-label`; makes the **icon shape** — a skin variant, not a new component — accessible) + `pressed`/toggle-mode (one inner "is-down" bool; momentary pulses it as today, toggle latches it as bind-out `pressed`; `aria-pressed` only in toggle-mode). Existing Quit / Shut-Down stay momentary, untouched.
- **`toggle`** gains no structural change — its checkbox / switch shapes are named for the catalogue, land as skin with the first skin file.
- **Read/drive model:** one inner bool = single source of truth; `bind:` (app) / `__XGEN_DEBUG__` (tooling) / `aria-*` (assistive tech) / skin (eye) are all projections, never copies. ARIA is reflected from the bool, never hand-managed.
- **`label` & `image` are display-kind di components** (correcting an earlier miscategorisation): they carry a value (`text` / `src`) + universal props but are read-only (no event-out). Data-independent → di family, not a separate "no-value" bucket. `image` = Phase-A primitive; a local-path / blob source pulls a usage to Phase B (Tauri).
- **Combobox = `textfield` + `datalist` composite** (di·A), not `textfield` + `select`. `select` = pick-only atomic; list-box = a `select` shape; rich list-view = a later `<div>` data-derived composite.

**Reprioritization (Joe's reframe — "address changes to done components before opening the next one").** Next-active reordered: the **`button` retrofit + first skin file** now lead, ahead of `select`. Rationale — the retrofit's shapes (icon / switch / pressed) only render once a skin file exists, and the first skin file also reconciles the global `input{}`/`button{}` pre-skin wrinkle (N-028/N-029), so the retrofit and the skin pass are naturally one arc. `select`, then `label`/`image`, then the first composites follow. Still Joe's call to lock the actual arc.

**Canonical (D-074).** `ui/docs/xgen-ui-notes.md` (N-030, v0.20→0.21); `ui/docs/xgen-ui-components.md` (N-030 pointers on the toggle/button rows + shape-family prose, v0.7→0.8); `CLAUDE.md` (PLAY Next-active reframed → button retrofit + first skin file; entry pointer J-407→J-408); this JOURNAL J-408. No ROADMAP change — the frontier milestone is unchanged (M-RP2.5 ✅); the reorder is within the open RP queue. Not pushed — Joe pushes.

---

## Entry J-407 — M-RP2.5 CLOSED: third `core` component (`textfield`) built + live CDP-verified in BOTH apps — string bind-in path, three binding shapes complete

**What happened.** Authored the third real `core` component, `textfield` (the Joe-locked M-RP2.5 next move), wired a throwaway demo into both shells, and Chat self-drove the full `tauri dev` + CDP loop in both apps. `textfield` is the **string bind-in** path (`bind:value`), completing the three envelope binding shapes the substrate now demonstrably generalizes across: toggle (boolean bind-in), button (event-out), textfield (string bind-in).

**Built + wired.** NEW `ui/core/lib/components/data-independent/textfield.svelte` — Svelte-5 runes, N-022 free-text single-line, N-020 atomic (root native `<input type="text">`), type-class via `use:envelope` (not hardcoded), no local CSS. `type` is **fixed, not a prop** (one-semantic-per-component — email/url/tel/password/number are separate components; search is a deferred shape variant). Native-state surface: `value` (`$bindable`, `bind:value`), `placeholder`, `disabled`, `readonly` (distinct from disabled — shown/selectable, not greyed), `id`, `pattern` (native `:invalid`; consumer's rule + skin's look, no bespoke validator), `name`. Debug getter `() => $state.snapshot({ value })`. Both shells (`app_client.svelte`, `app_node.svelte`) import `Textfield` via `$core` and mount a throwaway `<Textfield bind:value={demoText} id="demo" />` alongside the existing `toggle#demo` and the real Quit / Shut-Down buttons.

**Live verification (real-registry path, both apps; Chat self-drove the loop per N-028 working mode).** `run-{client,node}.ps1 -Debug` launched detached → poll CDP port → retry `snapshot()` until non-null (N-028 race fix) → dispatch a real `input` event via `cdp-debug.ps1 -Mode eval` → re-dump:
```
client (9222): BASELINE {"toggle#demo":{...checked:false},"textfield#demo":{"type":"textfield","state":{"value":""}},"button#quit":{...clicks:0,disabled:false}}
               EVAL RESULT: hello
               AFTER    {...,"textfield#demo":{"type":"textfield","state":{"value":"hello"}},...}
node   (9322): BASELINE {...,"textfield#demo":{"type":"textfield","state":{"value":""}},"button#shutdown":{...}}
               EVAL RESULT: world
               AFTER    {...,"textfield#demo":{"type":"textfield","state":{"value":"world"}},...}
```
The registry holds **three** components across **three** binding shapes side by side. The `value` ""→"hello"/"world" delta **re-lands the live-reactive-read proof on the bind-in path** — the proof the terminal-action button could not self-redump (N-028 finding 1). Cleanup clean: ports 9222/9322/5173/5174 all closed, zero orphans. Race-fix reconfirmed (client retry=2, node retry=1).

**Finding (N-029) — CDP input-dispatch subtlety.** Driving `bind:value` from CDP needs a **real dispatched `input` event** (`el.value="…"; el.dispatchEvent(new Event("input",{bubbles:true}))`), not a bare `el.value=` assignment — Svelte reads the value in the `input` handler, so a silent property set leaves the rune (and registry) stale. A correctness detail in the verify *procedure*, not the component (sibling-shape to the N-028 poll-race).

**Design boundaries recorded (N-029).** `type` fixed (not a prop); processor-**ready**, not processor-bearing (a text processor lives once in `common` as a `use:` action shared with `<textarea>` — DRY by composition, not duplication; deferred); the clear/copy-button version is a future `<div class="textfield-group">` composite (root-tag boundary, the legitimate split), not a "stateful twin" of this atomic. Pre-skin wrinkle noted: shells likely carry a global `input {}` rule, reconcile at the skin pass (N-025).

**No Rust, no protocol/data change.** Pure UI-layer + canonical-record work; test baseline unchanged from J-401 (~1466/0) — `cargo test` not re-run (Rule 5).

**Canonical (D-074).** NEW `ui/core/lib/components/data-independent/textfield.svelte`; `ui/client/src/app_client.svelte` + `ui/node/src/app_node.svelte` ($core Textfield demo); `ui/docs/xgen-ui-components.md` (textfield row + prose + Status line, v0.6→0.7); `ui/docs/xgen-ui-notes.md` (N-029, v0.19→0.20); `CLAUDE.md` (PLAY head → M-RP2.5, entry pointer J-406→J-407); `docs/ROADMAP.md` (RP node + M-RP2.5 + chain tail + Present + sampler reclass, v3.87→3.88); this JOURNAL J-407. Next-active (UI/RP track): Joe's call — `select` / `textfield-group` composite / `use:processor` action / sampler. Not pushed — Joe pushes.

---

## Entry J-406 — Closed the two N-028-routed items: D-095 dev-tooling-exemption footnote landed + GPL-overview question resolved (no decision); M-RP2.5 locked = `textfield`

**What happened.** Records-only follow-up to J-405. Joe locked the two items N-028 had routed to him, plus the next UI/RP move. No code, no protocol/data change; test baseline unchanged (~1466/0, not re-run — Rule 5).

**(1) D-095 footnote — landed.** Added a one-line **Dev-tooling exemption** to `DECISIONS.md` D-095: dev-only tool dirs under `ui/` (e.g. `ui/sampler/`, the component-exhibition app) are **not** part of the 1:1 crate-mirror — they sit alongside the mirrored tiers with no crate counterpart; the mirror governs the shipped substrate / library / shell tiers, not dev scaffolding. Keeps the tier map clean when the sampler dir lands.

**(2) GPL-overview question — resolved, no decision needed (Joe).** No GPL question arises during development: created code is locked under the single development license (BSL 1.1, per every file header), and GPL-2.0-or-later becomes effective on project handover per the fundamental records (the BSL→GPL conversion). The `ui/core/` = GPL boundary is a *future-state* property, not a present catalogue-duty trigger → no `DECISIONS.md` touch. The N-028 flag is marked **→ Resolved** in place (notes are append-only); the registry-as-catalogue / sampler-as-visual-face framing stands on its own (D-065). The CLAUDE PLAY "Owed to Joe" line flipped to CLOSED.

**(3) Next move locked — M-RP2.5 = `textfield`.** Joe took the recommended path (b): continue data-independent component authoring rather than stand up the sampler now. Next component = `textfield` (di · A, string-value `bind:value`) — the third binding shape after the toggle's boolean-in and the button's event-out, and the highest-reuse composite constituent (combobox / tag-select / password-field all compose it). Sampler Phase-A deferred behind catalogue growth + the first skin file. **Authoring not yet started — discuss-first; design vision surfaced, awaiting Joe's "go".**

**Canonical (D-074).** `DECISIONS.md` (D-095 dev-tooling-exemption footnote + Last updated 2026-06-22); `ui/docs/xgen-ui-notes.md` (N-028 GPL flag → Resolved, v0.18→0.19); `CLAUDE.md` (PLAY: Owed→CLOSED, Next-active→`textfield` lock, entry pointer J-405→J-406); this JOURNAL J-406. Not pushed — Joe pushes.

---

## Entry J-405 — M-RP2.4 CLOSED: second `core` component (`button`) built + live CDP-verified in BOTH apps; throwaway `Button.svelte` retired; CLAUDE PLAY caught up to UI/RP frontier (D-094); sampler design recorded (N-028)

**What happened.** Closed the button pipeline-tuning pass — authored the second real `core` component, swapped both shells onto it via `$core`, retired the pre-N-020 throwaway `Button.svelte`, and drove the full `tauri dev` + CDP loop in **both** apps. Also caught `CLAUDE.md`'s PLAY head up to the UI/RP frontier (it had drifted a full arc behind) and recorded the session's sampler design.

**Built + wired.** NEW `ui/core/lib/components/data-independent/button.svelte` — Svelte-5 runes, N-022 action-trigger, N-020 atomic (root native `<button>`), type-class supplied by `use:envelope` from `$common` (not hardcoded), event-**OUT** (`onclick`, no `bind`) — the complementary envelope path to the toggle's event-in `bind:checked`. Props `label`/`onclick`/`disabled`/`id`; honest internal `clicks` `$state` so the N-024 registry has a live observable; debug getter `() => $state.snapshot({ clicks, disabled })`. Both shells (`app_client.svelte`, `app_node.svelte`) swap `import Button` to `$core/components/data-independent/button.svelte` and replace `<Button text=.. app=.. onAction=..>` with `<Button label=.. onclick=.. id=..>`; `handleQuit`/`handleShutDown` untouched. Both throwaway `ui/{client,node}/src/lib/Button.svelte` (Svelte-4, pre-N-020) deleted.

**Live verification (real-registry path, both apps).** `run-{client,node}.ps1 -Debug` → `cdp-debug.ps1 -App {client,node} -Mode state` (separate terminal):
```
client (9222/:5173): {"toggle#demo":{"type":"toggle","state":{"checked":false}},"button#quit":{"type":"button","state":{"clicks":0,"disabled":false}}}
node   (9322/:5174): {"toggle#demo":{"type":"toggle","state":{"checked":false}},"button#shutdown":{"type":"button","state":{"clicks":0,"disabled":false}}}
```
DOM carried `class="button"` + `data-debug-id="button#{quit,shutdown}"`; clicking Quit / Shut-Down closed each window — the close affordance restored via the `core` button. The registry holding two components across two semantics (bind-in toggle + event-out button) confirms the N-023/N-024 substrate **generalizes**.

**Findings (recorded in N-028):** (1) terminal-action button can't self-redump a `clicks` delta — firing it exits the app; the live-reactive-read proof is inherited from `toggle` (same envelope path). (2) Pre-skin the button is not bare — it inherits a global `button {}` rule in each shell (N-025 wrinkle). (3) Node `-Debug` close prints a benign WebView2 teardown log `Failed to unregister class Chrome_WidgetWin_0. Error = 1412` (watch-item). (4) `run-* -Debug` blocks the terminal — run `cdp-debug.ps1` in a separate one.

**Sampler design-of-record + phase taxonomy (N-028).** Settled this session, implementation deferred (M-RP2.5+): a separate dev app at `ui/sampler/` (dev-tool dir, D-095-mirror-exempt — one-line footnote owed, routed to Joe); three build-phases A/B/C, trigger-driven; both the read (N-024) and write/synthetic-feed sides designed-for; IA = class×phase matrix-of-record, tabbed-by-phase (gated phases = locked tabs), di/dd = in-pane `[All|di|dd]` segmented filter (sub-tabs a volume-triggered per-pane upgrade), Combined tab = skinned together-gallery; skin+size global chrome; index-driven; live skin-swap the killer feature. New component **layer-phase taxonomy (A/B/C)**, orthogonal to di/dd, recorded as a Phase column in the components registry. **GPL-overview flag routed to Joe** (`ui/core/` = GPL → the registry is the licensed-corpus catalogue; candidate `DECISIONS.md` touch).

**CLAUDE.md PLAY catch-up (D-094).** The PLAY head had drifted a full arc behind — still reading the AFI/F17 frontier (last touched 2026-06-18); the J-399→J-404 UI/RP arc updated JOURNAL + ROADMAP + ui-docs but never flipped the PLAY head (the named ROADMAP-vs-CLAUDE drift). Caught up here: the stale AFI/F17 PLAY block lifted **verbatim** to `CLAUDE_HISTORY.md` (D-094, a move not a rewrite), and a fresh UI/RP PLAY head written (M-RP2.3 ✅ J-403 → M-RP2.4 ✅ this entry → sampler horizon).

**No Rust, no protocol/data change.** Pure UI-layer + canonical-record work; test baseline unchanged from J-401 (~1466/0) — `cargo test` not re-run (Rule 5: not claimed beyond baseline).

**Canonical (D-074).** NEW `ui/core/lib/components/data-independent/button.svelte`; `ui/client/src/app_client.svelte` + `ui/node/src/app_node.svelte` ($core button swap); DELETED `ui/client/src/lib/Button.svelte` + `ui/node/src/lib/Button.svelte`; `ui/docs/xgen-ui-components.md` (button row + Phase column, v0.5→0.6); `ui/docs/xgen-ui-notes.md` (N-028, v0.17→0.18); `CLAUDE.md` (PLAY head AFI/F17 → fresh UI/RP head + pointer line); `CLAUDE_HISTORY.md` (AFI/F17 block archived, v1.0→1.1); `docs/ROADMAP.md` (RP node + M-RP2.4 + sampler rows + chain tail + Present, v3.86→3.87); this JOURNAL J-405. Next-active (UI/RP track): sampler Phase-A (M-RP2.5) or continue data-independent component authoring — Joe's call. Not pushed — Joe pushes.

---

## Entry J-404 — ROADMAP reconciliation (drift sweep, 3 altitudes) + 4 May-era UI-mockup docs DEPRECATED

**What happened.** Reconciled `docs/ROADMAP.md` against actual state before opening UI-component work — it had drifted since 2026-06-18 (≈J-398-era; everything J-399→J-403 was unrepresented), at three altitudes:
- **Road tree:** added `✅ Appendix F/I audit` (J-397/J-398) + `✅ AI-F17` (J-401) nodes (the tree jumped doc-opt→UI, missing both — a tree/prose drift the doc itself calls a discipline failure); restructured the UI subtree — `⬛ mockup stock-take` + `🟢 UI component-library/substrate (RP)` + `🟡 clean-table UI`.
- **"How to use this view":** the "playing now"/"live frontier" narrative bullets were frozen at ~J-256/J-258 (pre-dating M8.5→M12, multiparty, M10/M11/M12, doc-opt, AFI) and contradicted the J-390 ASCII diagrams beside them. Added a reconciliation note marking the narrative historical; reframed both bullet leads to the real frontier (close-ledger kept as labeled history); fixed diagram markers `M8.7 [NEXT-ACTIVE]→[CLOSED J-302]`, `M9 [pending]→[CLOSED J-307]`, and the post-gate chain tail.
- **Present/Near/Far:** Present now leads with the RP/UI track ACTIVE (doc-opt → ✅ COMPLETE); Pre-UI chain line rebuilt; Near mockup stock-take `🟢 NEXT-ACTIVE → ⬛ DEPRECATED`; Far/UI RP-groundwork note. Header v3.85→v3.86, Last updated 2026-06-21.

**Mockup stock-take retired (Joe's call).** The planned "reconcile the May-era `ui/docs/` mockups to as-built" step is **superseded** (not performed) by the component-library-first build — recorded ⬛ DEPRECATED throughout ROADMAP. The four still-`ACTIVE` mockup docs flipped to DEPRECATED (replacement named): `xgan-ui-overview.md`, `xgen-ui-chat-briefing.md`, `xgen-ui-run-2_BRIEFING.md`, `xgan-ui-debug-console-questions.md` (`xgen-ui-design-brainstorm.md` was already DEPRECATED; `xgan-ui-run-1_SUMMARY.md` stays COMPLETED — a frozen session record).

**No code, no protocol/data change.** Doc-hygiene only; test baseline unchanged (~1466/0). Sibling-shape to DO-3 (J-394 ROADMAP refresh).

**Canonical.** `docs/ROADMAP.md` (v3.86) shipped as a prior standalone reconciliation commit (already pushed); this JOURNAL J-404 + the four UI-mockup doc deprecations travel in this commit. Next-active (UI/RP track): first `core` button + retire throwaway `Button.svelte`.

---

## Entry J-403 — M-RP2.3 CLOSED: substrate proof — first `core` component (`toggle`) built + live CDP registry verified in BOTH apps

**What happened.** Closed the substrate proof. Built the first real `core` component and drove the full `tauri dev` + CDP debug loop end-to-end in **both** UI apps — establishing that the N-023/N-024 base substrate and the D-095 tier wiring work in a real Vite build, not just under `tsc --noEmit`.

**Built + wired.** NEW `ui/core/lib/components/data-independent/toggle.svelte` — Svelte-5 runes, N-022 boolean-toggle, N-020 atomic (root native `<input type="checkbox">`), type-class supplied by `use:envelope` from `$common` (not hardcoded), N-024 opt-in debug getter `() => $state.snapshot({ checked })`. Mounted as a throwaway `<Toggle id="demo">` demo instance in both shells via `$core` (`app_client.svelte`, `app_node.svelte`). The pre-N-020 throwaway `Button.svelte` (Quit / Shut-Down) was kept — both windows are `decorations:false`, so retiring the only close affordance waits for the first `core` button (next step).

**Live verification (real-registry path, not transport-only).** `run-{client,node}.ps1 -Debug` (Vite + `tauri dev` + remote-debug port) → `cdp-debug.ps1 -App {client,node} -Mode state` (attach):
- client (9222 / :5173) and node (9322 / :5174) both attached over the WS;
- `$common`/`$core` aliases resolved in real builds; DOM carried `class="toggle"` + `data-debug-id="toggle#demo"`;
- `window.__XGEN_DEBUG__.snapshot()` returned real `{"toggle#demo":{"type":"toggle","state":{"checked":false}}}`; flipping the toggle then re-dumping returned `{checked:true}` — confirming the getter reads **live reactive scope** (the N-024 claim), not a mount-time snapshot.

**Tooling fixes (arc-local, not promoted).** `run-client.ps1` + `run-node.ps1` both pointed `$TauriDir` at a non-existent `…/src-tauri` (the Tauri crate is the `xgen-client` / `xgen-node` root) — fixed; added a dev-only `-Debug` switch injecting `--remote-debugging-port=<9222|9322>` (client = clean param block; node = `$args` read, to preserve `--service` arg-forwarding). `cdp-debug.ps1` `state` mode brought from the bare `JSON.stringify(window.__XGEN_DEBUG__)` (which stringifies the singleton's methods to `{}`) to `…snapshot()` — the script had drifted from harness-doc v1.1. Harness DoD's last UI-gated (state-dump) box ticked for both apps; the release-inert box left open (no release build was run — honest, not claimed).

**No Rust, no protocol/data change.** Test baseline unchanged from J-401 (~1466/0) — pure UI-layer + dev-tooling work; `cargo test` not re-run (Rule 5: not claimed beyond the J-401 baseline).

**Canonical (D-074).** NEW `ui/core/lib/components/data-independent/toggle.svelte`; `ui/client/src/app_client.svelte` + `ui/node/src/app_node.svelte` (Toggle demo via `$core`); `run-client.ps1` + `run-node.ps1` (TauriDir + `-Debug`); `cdp-debug.ps1` (`.snapshot()` fix); `ui/docs/xgen-ui-components.md` (NEW Built-components registry + Status PENDING→ACTIVE, v0.4→v0.5); `ui/docs/xgen-ui-notes.md` (N-027, v0.16→v0.17); `tasks/CDP_DEBUG_HARNESS.md` (DoD tick + §Mechanism two-path note + script-fix note, v1.1→v1.2); this JOURNAL J-403. Next-active (UI/RP track): first `core` button + retire throwaway `Button.svelte`; then continue data-independent component authoring. Not pushed — Joe pushes.

---

## Entry J-402 — Settled finding (do-not-re-research): `is_ai`/`ai_capabilities` were never deferred — omission-when-human is intentional back-compat

**What happened.** During a client-UI modelling session the question resurfaced — why does the wire identity record appear to "omit" `is_ai`/`ai_capabilities`? Verified against code + Appendix I §IV.1 to settle it permanently. The AI fields are fully implemented and load-bearing: present on `IdentityMessage::Register` **and** (post-F17, J-401) `::Record`; shape-validated (3040); immutable after registration (3041); part of the **canonical signing form** (stripping `is_ai` breaks verification); federation-replicated; and they drive pacing (§3.7.12.1). The apparent "omission" is a deliberate **serialization rule**: when `is_ai = false` (human), both fields are serde-skipped from canonical form so pre-AI human signatures stay **byte-identical**. AI identities serialize *with* them. Nothing was deferred at the model level; F17 (Record-variant exposure) closed code-side at J-401 and the doc (§IV.1) was already aligned.

**Why recorded.** Marks the understanding **settled** so the omission-when-human shape is not re-investigated as a gap in future sessions. UI consequence: `is_ai` reaches the client on identity-record lookups of AI identities → the AI-badge is wire-backed (D-059), not aspirational.

**Canonical.** Doc-only, no code. This JOURNAL J-402; `tasks/HANDOFF_AFI_AUDIT.md` §9 annotated with the F17 post-close pointer. Next-active unchanged: mockup stock-take + reconcile-to-as-built (Chat seat).

---

## Entry J-401 — F17 CLOSED code-side (Clair seat): `IdentityMessage::Record` gains `is_ai`/`ai_capabilities`, populated on `identity.get`

**What happened.** Implemented the Joe-LOCKED (J-400) F17 code fix. Added `is_ai: bool` (`#[serde(default, skip_serializing_if = "is_false")]`) and `ai_capabilities: Option<AiCapabilities>` (`#[serde(default, skip_serializing_if = "Option::is_none")]`) to `IdentityMessage::Record` (`xgen-core/src/wire/types.rs`), mirroring the discipline already on `identity.register`/`IdentityRecord`. The `identity.get` responder (`xgen-node/src/app.rs`) now populates both from the stored `IdentityRecord`. Additive + backward-compatible: human-record lookups stay byte-identical (both fields serde-skipped when false/None); a peer/client lookup of an AI Identity now sees its AI status — the §3.6.10 transparency requirement. Three tests added in `wire/types.rs`: `identity_record_with_ai_round_trip` (AI carries `is_ai=true` + caps), `identity_record_human_omits_ai_fields_in_canonical_form` (human omits both), `identity_record_legacy_without_ai_fields_deserialises` (pre-F17 wire JSON still parses → human record). No Appendix I edit — §IV.1 already matched the target shape (per J-400).

**Verification.** `cargo test --workspace` → `passed=1466 failed=0 ignored=62`. New F17 tests confirmed passing:
```
test wire::types::tests::identity_record_human_omits_ai_fields_in_canonical_form ... ok
test wire::types::tests::identity_record_legacy_without_ai_fields_deserialises ... ok
test wire::types::tests::identity_record_with_ai_round_trip ... ok
```

**Canonical (D-074).** `xgen-core/src/wire/types.rs` (Record fields + 3 tests); `xgen-node/src/app.rs` (responder populates the fields); `tasks/ROUTE_F17_identity_record_ai_fields.md` → COMPLETED; this JOURNAL J-401. Next-active: mockup stock-take + reconcile-to-as-built (Chat seat).

---

## Entry J-400 — F17 direction Joe-LOCKED: code fix (Clair seat); ROUTE_F17 → ACTIVE; next-active flipped to F17 (then mockup stock-take)

**What happened.** Joe locked the **F17** resolution direction — the **code fix** (the Chat recommendation from J-399) — to be implemented by Clair in a new session before the mockup stock-take. F17 = the wire `identity.record` (`IdentityMessage::Record`) omits `is_ai`/`ai_capabilities` that Appendix I §IV.1 documents and §3.6.10 transparency expects. The fix is additive + backward-compatible: add both fields to `IdentityMessage::Record` (serde-skip when false/none), populated from the stored `IdentityRecord` on `identity.get`; Appendix I §IV.1 already matches the target so no doc edit is needed (a one-line JOURNAL close note when the code lands). Doc-only this entry; the code is Clair's next session.

**Bridges prepared for the new session.** `tasks/ROUTE_F17_identity_record_ai_fields.md` flipped PENDING→ACTIVE (v1.0→v1.1), §2 marked Joe-LOCKED code fix (the doc-fix alternative retained as NOT-chosen context). CLAUDE PLAY next-active flipped to F17 (Clair seat) → then Chat resumes the mockup stock-take. Clair's Rule-0 reads on open: CLAUDE PLAY → this J-400 → `tasks/ROUTE_F17_identity_record_ai_fields.md` (the spec + acceptance criteria).

**Canonical (D-074).** `tasks/ROUTE_F17_identity_record_ai_fields.md` (ACTIVE, v1.1); `CLAUDE.md` PLAY (F17 next-active); this JOURNAL J-400. Next-active: F17 code fix (Clair), then mockup stock-take + reconcile-to-as-built.

---

## Entry J-399 — Post-AFI close-up: F17 routed to Clair (`tasks/ROUTE_F17_identity_record_ai_fields.md`); DO-5 handoff flipped COMPLETED + archived

**What happened.** Two close-up items after the AFI arc close (J-398), doc-only, no code. (1) **F17 routing** — authored `tasks/ROUTE_F17_identity_record_ai_fields.md` (PENDING, Clair seat): the suspected code gap where the wire `identity.record` (`IdentityMessage::Record`) omits `is_ai`/`ai_capabilities` that Appendix I §IV.1 documents and §3.6.10 transparency expects. Recommendation locked-in-doc: additive code fix — add both fields to `IdentityMessage::Record` (serde-skip when false/none, populated from the stored `IdentityRecord` on `identity.get`); no Appendix I edit needed since the doc already matches the target. Awaits Joe-lock on direction (code fix vs. trim §IV.1). (2) **§8 hygiene** — `tasks/HANDOFF_DO5_JOURNAL_WINDOWING.md` flipped ACTIVE→COMPLETED and archived to `tasks/archive/` (DO-2 convention); the deferred close-up noted in the AFI handoff §8.

**Canonical (D-074).** NEW `tasks/ROUTE_F17_identity_record_ai_fields.md` (PENDING); `tasks/HANDOFF_DO5_JOURNAL_WINDOWING.md` → COMPLETED + git-mv to `tasks/archive/`; this JOURNAL J-399; `CLAUDE.md` PLAY annotation. Next-active unchanged: mockup stock-take + reconcile-to-as-built.

---

## Entry J-398 — AFI audit AI sub-pass CLOSED: Appendix I reconciled to as-built (v1.7); new fundamentals appendices M/N/O authored + event_trace folded into Appendix G (v1.2); F17 (identity.record missing AI fields) Joe-routed as suspected code gap — AFI arc CLOSED

**What happened.** Closed the AI sub-pass of the AFI audit (handoff `tasks/HANDOFF_AFI_AUDIT.md`), reconciling Appendix I (data structures) to the as-built serializable types in `xgen-common`/`xgen-core` + the protocol event catalog. Code as ground truth (Q4). Doc-only; no code. Phase-0 was a full read-only inventory diffed both directions (D-077): no forward-drift in the wire enums' variant sets; the drift was backward-coherence (the doc lagged the M8→M12+ backend) plus one forward-drift surfaced on field-level read (F17).

**Appendix I reconcile (v1.6 → v1.7), AI-F01–F16, all doc-side.** Thread model added — `thread.create`/`resolved`/`archived` event rows (§I.2) + `ThreadState`/`ThreadStatus` (new §VI.9); `SpaceState` +`jurisdiction`/`e2e_encryption`/`threads`; `RoomState` +`permission_overrides`/`mls_commit_tip`; `PendingInvite` (new §VI.7, +`valid_until`); `RoomPermission`/`Effect` (new §VI.8); eight `TransportMessage` variants (`sync_complete`, `invite_bootstrap_request`, `blob_upload_begin/chunk/upload_end/upload_ok`, `blob_fetch_request/fetch_end`) + `sync_request.limit` + `auth_ok.node_id`; `identity.register` +`re_registration`; `IdentityRecord` +`revoked`/`revoked_at`/`revocation_reason`; Part IX honesty note (file/reaction/redact + thread.* content is handler-defined, not tabulated). F15 (`transport.error.event_id`) was already documented — no-op.

**Three fundamentals promoted to their own appendices (single source of truth per topic).** Rather than cross-ref scattered mentions, each earned a proper data-structure article: **Appendix M** — Trust Assertions & Auth-Tier evidence (`trust_assertion.rs`: TrustAssertion/TrustClaims/ModulePolicy/Erasability/Retention/ModuleKind + reserved claims keys). **Appendix N** — Auth-Module / Plugin framework descriptors (`module.rs`: ModuleKindId/ModuleImplId/AssuranceClass/Descriptor). **Appendix O** — `--aicontrol` control-plane structures (`aicontrol/*`: Command/Reply/ErrorBody/Category/ControlCode catalogue/cmd-resolution/Filter/Bindings/IdempotencyStore/TimeoutTier/token). Appendix I's Overview now points at M/N/O and scopes out observability/registry/clock types (S1/S4/S5). aicontrol behaviour stays in `xgen_aicontrol_implementation.md` (sibling split); L/C/ch3 remain the homes for storage/primitive/normative rules.

**event_trace → Appendix G (Joe-locked, option c).** The reconcile found `event_trace.rs` (EventDirection/LocalAction/ExitReason/SpaceRole/SessionContext) is the structured-logging subsystem, not control-plane — its enums emit Appendix G log values. Folded the typed enums + session-context into a new **Source Types** section in Appendix G (v1.1→v1.2), mapping each Rust variant to its log string without duplicating the existing value tables.

**F17 — Joe-routed (suspected code gap, NOT a doc edit).** Field-level read showed the wire `IdentityMessage::Record` (`identity.record`) carries only `protocol_version/identity_id/display_name?/registered_at/devices/home_node` — it omits `is_ai`/`ai_capabilities` that Appendix I §IV.1 documents and that §3.6.10 AI-transparency expects (the full `IdentityRecord` and the replication path both carry them). Per Q4 this is a suspected backend gap, not a doc drift: §IV.1 left intact; routed to Joe/Clair for a code decision (expose AI status on the lookup response vs. trim the doc).

**Scope calls (Joe-locked).** S1 observability + event_trace value-strings → handled (G / out). S2 aicontrol → Appendix O. S3 module descriptors → Appendix N (M10 closed J-375, so shipped surface, not deferred). S4 clock.rs → out (D-090 home). S5 internal registries/helpers → out (implementation docs). Reverse cross-refs into C/L/ch3/aicontrol-impl deliberately skipped (option B) — M/N/O+I+G already establish authority; keeps the commit focused.

**Canonical (D-074, doc-only).** NEW `docs/xgen_appendix_m_en.md` / `_n_` / `_o_` (v1.0); `docs/xgen_appendix_i_en.md` v1.6→v1.7; `docs/xgen_appendix_g_en.md` v1.1→v1.2; `tasks/HANDOFF_AFI_AUDIT.md` → consolidated AF+AI ledger, COMPLETED; `docs/ROADMAP.md` (AFI CLOSED + mockup next + v3.84→v3.85); `CLAUDE.md` head (AFI CLOSED, mockup next-active); this JOURNAL J-398.

**Next-active.** Mockup stock-take + reconcile-to-as-built (the last pre-UI step), then UI clean-table → Streams. Open item for Joe: F17 routing.

---

## Entry J-397 — AFI audit AF sub-pass CLOSED: Appendix F reconciled to the as-built CLI (5 findings); `federate` reframed N/A-only-node-concept + ch2 node-to-node note; §F.2.1 node-admin cross-ref

**What happened.** Opened the post-doc-opt pre-UI arc — the Appendix F/I audit-against-code (handoff `tasks/HANDOFF_AFI_AUDIT.md`; Phase-0 + AF + AI; code as ground truth, D-071). Phase-0 inventoried the as-built surface (client: 31 leaf verbs from `ClientCommand`/`ThreadCommand`/`AiCommand`; node foreground `NodeCommand`; node admin `AdminCommand` tree). AF sub-pass diffed Appendix F (F.0.2 / F.0.4 / F.2 / F.3) against code and reconciled. Doc-only; no code.

**AF findings (all closed, doc-side; Q4 code-as-truth).** AF-F01 `create-dm-space` and AF-F02 `leave` were in code but absent from F.0.4 + F.3 — rows added. AF-F04 `members` carried a stale "deferred / no local data source" annotation + Network=No, but the verb ships (WS DAG replay, covers DM Spaces) — de-staled + Network No→Yes. AF-F06 node `whoami` (real `NodeCommand` verb) was missing from F.2 — added. All four D-092 dispatch arms verified present for every drift verb.

**`federate` (AF-F03) — reframed, not removed.** `federate` is listed client-side only for client↔node CLI vocabulary symmetry; the capability is a Node concept (federation is node-to-node, operator-governed — production verb `xgen-node federation initiate`, plus list/defederate/accept/reject/set-policy/show-policy; `add-peer` is a fenced harness seam). The stale "Deferred to M6 Phase 7" wording (a false-positive implying a pending feature) was replaced with `N/A — only node concept` + a by-design note, and the architectural boundary stated once in ch2 (Architecture): a client has no federate action by nature.

**Symmetry audit.** The shared CLI vocabulary is the small fundamental core (`init`/`whoami`/`status`/`version`, already F.0.2) + `spaces` (scope-collision, F.0.5). `federate` is the single deliberate cross-concept mention; the rest of the client/node asymmetry is structural (different jobs), not a gap — symmetry stays a one-off, no systematic N/A cross-listing.

**Node-admin surface (fork A).** The `AdminCommand` tree (federation / identity / space / audit / log / auth-module / bootstrap / migration / plugin) is documented authoritatively in `xgen_node_admin_ops_design.md` and was absent from F.2 with zero cross-reference. Rather than duplicate ~35 verbs (a second source of truth), F.2 gained a new §F.2.1 group-summary + pointer.

**Canonical (D-074, doc-only, checkpoint).** `docs/xgen_appendix_f_en.md` v1.12→v1.13; `docs/xgen_ch2_architecture.md` v1.1→v1.2; `tasks/HANDOFF_AFI_AUDIT.md` NEW; `docs/ROADMAP.md` (AFI marker + version); `CLAUDE.md` head (AFI OPENED, AF done, AI next); this JOURNAL J-397.

**Next-active.** AI sub-pass — diff Appendix I (data structures) ↔ the as-built serializable types (xgen-common, 57 structs/enums) + the protocol event catalog (D-077 both directions).

---

## Entry J-396 — DO-5: JOURNAL.md windowed — J-375 and older relocated to JOURNAL_ARCHIVE.md (D-094); documentation-optimization phase COMPLETE

**What happened.** DO-5, the final and riskiest documentation-optimization sub-step, windowed this development journal. The file had grown to 17,761 lines / 2.38 MB / 378 entries. The recent arc (M11 → M12 → Round-2 pre-UI gate → doc-opt; entries J-395 … J-376, 20 entries) was kept live here; entries J-375 and older (358 entries) were relocated verbatim to a new `JOURNAL_ARCHIVE.md` (ARCHIVED), with a forward pointer at the cut here and a back-pointer at the archive top (D-094 convention).

**Cut point.** Milestone-clean boundary, Joe-locked: the live window begins at J-376 (M11 OPENED); the archive begins at J-375 (M10 CLOSED). No content was edited — a pure byte-exact relocation. Guard: live entry-count (incl. this J-396) + archived entry-count == 379; both files re-counted and asserted; first and last entry of each file spot-checked intact.

**Convention.** D-094 (the windowing convention already applied to `CLAUDE_HISTORY.md` in DO-1) covers JOURNAL windowing; a one-line note was added to it rather than minting a new decision ID.

**Canonical (D-074).** `JOURNAL.md` (windowed live window + forward pointer + this J-396) and new `JOURNAL_ARCHIVE.md` (358 entries, ARCHIVED); `DECISIONS.md` (D-094 one-line note); `docs/ROADMAP.md` (DO-5 marker + doc-opt node/arrow done + Present advanced to the Appendix F/I audit step + version bump); `CLAUDE.md` head (DO-5 done; documentation-optimization phase COMPLETE; next frontier = Appendix F/I audit-against-code). Doc-only; no code.

**Next-active.** Documentation-optimization phase closes. Per the Joe-locked pre-UI chain: Appendix F/I audit-against-code → mockup stock-take + reconcile-to-as-built → UI → Streams.

---

## Entry J-395 — DO-4: DECISIONS.md duplicate-ID resolution (R2G-F03) — 7 collided D-numbers split a/b; ~30 references repointed

**What happened.** DO-4 resolved R2G-F03, the duplicate decision IDs in DECISIONS.md. The defect was larger than the named D-030/D-031: seven numbers (030, 031, 037, 038, 039, 055, 056) were each used for two unrelated decisions — a legacy of the file being two roughly-sorted blocks (a descending D-093→D-000 block, then an ascending D-001→D-094 block) that collided on those numbers.

**Convention.** Per the established `F-10a` precedent (glued lowercase letter suffix), each collided number was split into `D-NNNa` / `D-NNNb`, earlier file occurrence = `a`. No renumbering (would break cross-references) and ordering left as-is — lookup is by ID, not reading order (suffix-not-renumber, Joe-locked).

**Reference repointing.** Each live bare `D-NNN` citation across ch1–ch6, the appendices, the lifecycle/admin design docs, ROADMAP, and CLAUDE was read in context and repointed to the sense it meant. Most pairs were single-sense (e.g., every reference to the MLS decision → the `a` entry; the unreferenced Phase-1-config decision → `b`). Two pairs — D-037 (Tier-1 identity vs Node deployment) and D-056 (recv() routing fix vs one-binary-per-role) — were cited heavily in both senses and were classified per-reference. A guard (zero bare two-digit decision ref of the seven remaining in any live doc) gated the close; JOURNAL and tasks/archive are append-only history and were left untouched.

**Canonical (D-074).** `DECISIONS.md` (14 headings suffixed + internal refs repointed), the cross-doc reference updates, `docs/ROADMAP.md` (DO-4 marker + v3.81→v3.82), `CLAUDE.md` (head: DO-4 ✅), this JOURNAL J-395. Doc-only; no code.

**Next-active.** DO-5 — JOURNAL.md windowing (the riskiest sub-step; last / deferrable).

---

## Entry J-394 — DO-3: ROADMAP prose refresh — completed milestones relocated to Past; Present rewritten to the doc-opt phase; tree untouched

**What happened.** DO-3 of the documentation-optimization phase: refreshed the ROADMAP's prose sections, which had drifted badly out of sync with the codebase while the backend sprinted M6→M12. The tree/chain "Visual structure" section was left untouched (Joe-lock). `docs/ROADMAP.md` v3.80→v3.81, 914→752 lines.

**The staleness found.** The "Present — playing now" section was a ~180-line chronological stack of ~50 already-CLOSED entries (J-180→J-357); the actual present (the doc-opt phase) was absent and the newest entry still read "next-active = M10". "Far future" was tangled with M7/M8/M9/multiparty/M10/M11/M12 all marked done-or-scheduled, and "Past — settled" had stopped being maintained at the XGID/federation era (~J-148).

**The refresh (Option A — condense, not archive; the detailed record already lives in JOURNAL).** Past — settled gained terse one-liners for everything M6→M12 + multiparty + Round-2, grouped by family (M6 admin verbs, M7 family, Storage subsystem, the Round-1 D-071 arc pack, Round-2 first, M8 family, M9 + Multiparty-tests, M10–M12, the Round-2 final pre-UI gate). Present — playing now was rewritten to describe only the doc-opt phase (DO-1✅/DO-2✅/DO-2b✅/DO-3 now/DO-4–5 pending) + the pre-UI chain. Near future = the real pre-UI remainder (Appendix F/I audit → mockup stock-take) + identity→home-node discovery. Far future = UI + Streams + routed topics, stripped of completed-milestone clutter. Parallel-workstreams / Open-areas / Cross-cutting / How-to-read were current and left intact.

**Canonical (D-074).** `docs/ROADMAP.md` (the rewrite + v3.81 + the doc-opt marker bumped to DO-3-done in the tree node/arrow), `CLAUDE.md` (head: DO-3 ✅, next-active DO-4), this JOURNAL J-394. No DECISIONS change. Doc-only; no code.

**Next-active.** DO-4 — DECISIONS.md R2G-F03 (D-030/D-031 duplicates + non-monotonic ordering; suffix-not-renumber).

---

## Entry J-393 — DO-2b: 43 stale-ACTIVE task docs triaged + archived; NODE_ADMIN_PASS2 kept live (PENDING)

**What happened.** DO-2b cleared the 44 stale `Status: ACTIVE` task docs surfaced in DO-2 (J-392). All 44 mapped to CLOSED milestones — their headers were never flipped at arc close. Triage outcome: 43 archived under D-094, 1 kept live.

**Disposition.** 42 → `Status: COMPLETED` (header-only flip, original `Last updated` dates preserved — Option Y, honest about real work dates) then `git mv` to `tasks/archive/`. `BATCH_FLAG_review.md` → `DEPRECATED` (its subject, the `--batch` flag, was descoped) then archived. `NODE_ADMIN_PASS2_PROPOSALS.md` → `PENDING`, kept in live `tasks/` — it feeds the still-pending "M6 (new) Node admin write path". Flips were surgical (status line only; rest of each file byte-identical).

**State.** Live `tasks/` now holds 3 files, all PENDING: `FEDERATION_PROPAGATION_PHASE_9_SURVEY.md`, `M6_CLIENT_MEMBERS_DESIGN.md`, `NODE_ADMIN_PASS2_PROPOSALS.md`. `tasks/archive/` holds 250 docs + the README pointer. Stale-ACTIVE count in live `tasks/` = 0.

**Canonical (D-074).** `CLAUDE.md` (head: DO-2b ✅), `docs/ROADMAP.md` (doc-opt node + chain arrow), this JOURNAL J-393, the 43 renames (+1-line status edits) + the NODE_ADMIN PENDING flip. No DECISIONS change (D-094 governs). Doc-only; no code.

**Next-active.** DO-3 (ROADMAP Present-prose refresh — stale at the J-256/Arc-H era).

---

## Entry J-392 — DO-2: 207 terminal task docs archived to tasks/archive/ (D-094 applied); 44 stale-ACTIVE labels surfaced → DO-2b

**What happened.** DO-2 of the documentation-optimization phase: `tasks/` held 253 `.md` files (~4.86 MB) and no archive subfolder. The 207 terminal documents — 204 COMPLETED + 1 COMPLETE (spelling variant) + 2 DEPRECATED — were relocated to a new `tasks/archive/` via `git mv` (history preserved), applying the D-094 archive-with-forward-pointers convention first established in DO-1. A `tasks/archive/README.md` (ARCHIVED) is the forward pointer. Live `tasks/` now holds 46 files.

**Finding surfaced (D-065) → DO-2b.** Of the 46 remaining, 44 still carry `Status: ACTIVE` and 2 `PENDING`. With the entire M8→M12 chain and the multiparty milestone all CLOSED, the 44 ACTIVE are almost certainly stale labels never flipped at arc close — a documentation-hygiene gap, not genuine in-flight work. Rather than sweep by label (which would either freeze a lie or risk archiving live work), the gap is routed as **DO-2b**: a bounded triage pass — confirm each of the 44 is terminal, flip Status to COMPLETED (header-only; no content rewrite), then archive under the same D-094 pattern. The 2 PENDING stay live.

**Canonical (D-074).** `CLAUDE.md` (live head: DO-2 done, next-active DO-3); `docs/ROADMAP.md` (doc-opt node + chain arrow updated); this JOURNAL J-392; the 207 renames + `tasks/archive/README.md`. No DECISIONS change (D-094 already governs). Doc-only; no code.

**Next-active.** DO-3 (ROADMAP Present-prose refresh — stale at the J-256/Arc-H era), with DO-2b (the 44-ACTIVE triage) as a near-term bounded follow-on. Not pushed — Joe pushes.

---

## Entry J-391 — Documentation-optimization phase OPENED: chain reconciled (doc-opt → F/I → mockups → UI); DO-1 CLAUDE.md split + D-094 archive-convention landed

**What happened.** Opened the pre-UI documentation-optimization phase (doc-only, no code). The phase exists to discharge the documentation bloat accumulated while the backend sprinted M8→M12 and the reference/operational records were maintained reactively. Two moves landed in this opening commit: DO-1 (the CLAUDE.md split) and the D-094 archive convention.

**Chain reconciliation.** The J-390 bridge recorded "next-active = UI" across PLAY / ROADMAP / JOURNAL — one step ahead of the plan locked with Joe. The true pre-UI chain is: documentation-optimization → Appendix F/I audit-against-code → mockup stock-take + reconcile-to-as-built → UI (clean-table build) → Streams (standalone, post-UI). This entry, the PLAY head, and ROADMAP (the tree node + the chain arrow) flip the record to that chain, atomically (D-074).

**DO-1 — CLAUDE.md split.** CLAUDE.md had grown to ~898 KB (2,769 lines) and could no longer be read in a single call — a Rule-0 cost on every session open. The file is three zones: the live PLAY head, a ~2,200-line stack of superseded PLAY blocks (the J-389 M12-close block back through M6 / XGID-retrofit / Phase-9), and a stable briefing tail (MANDATORY Behaviour rules → Build Commands). The superseded stack was lifted verbatim into a new `CLAUDE_HISTORY.md` (Status: ARCHIVED); CLAUDE.md now holds the live head + the stable briefing + a forward pointer to the archive. No content was rewritten — a move, not an edit.

**D-094 — archive convention.** Promoted the archive-with-forward-pointers discipline to a standing decision so the next bloat cycle does not re-litigate it: the four canonical operational records keep a small live head and relocate superseded content to a frozen ARCHIVED sibling with a forward pointer; archiving is a move, never a rewrite; D-074 atomicity and the append-only / no-retroactive-rewrite conventions are preserved. First exercised by DO-1.

**Phase plan.** DO-1 CLAUDE split (this commit) · DO-2 tasks/ archive (~204 COMPLETED/ARCHIVED docs → `tasks/archive/`) · DO-3 ROADMAP Present-prose refresh (stale at the J-256/Arc-H era) · DO-4 DECISIONS.md R2G-F03 (D-030/D-031 duplicates + non-monotonic order — suffix, not renumber) · DO-5 JOURNAL.md windowing (range-archives + a recent live window; riskiest, scheduled last / deferrable).

**Canonical (D-074).** `CLAUDE.md` (new J-391 head + the split); `CLAUDE_HISTORY.md` NEW (ARCHIVED); `DECISIONS.md` D-094 NEW; `docs/ROADMAP.md` v3.79→v3.80 (chain flip at the tree node + the chain arrow); this JOURNAL J-391. No code; no Appendix change (no CLI verb touched).

**Next-active.** DO-2 (tasks/ archive). **Entry (Rule 0):** CLAUDE.md PLAY → JOURNAL J-391 → `docs/ROADMAP.md` (doc-opt chain) → `DECISIONS.md` D-094 → `CLAUDE_HISTORY.md`. Not pushed — Joe pushes.

---

## Entry J-390 — Round-2 final pre-UI gate CLOSED: whole-codebase Pass-2 audit GO; M10/M11/M12 + cross-arc clean; R2G-F01..F04 routed; next-active = UI

**What happened.** Clair authored the Round-2 final pre-UI gate audit (audit seat) — the single whole-codebase Pass-2 sweep that gates UI — and committed it audit-only (`145d2c8`, pushed). Chat independently cross-verified the load-bearing hinges in code on `main` (D-065), opened the triage discussion on the four new findings + the carried/routed items, recommended a routing on each; Joe locked **GO by-recomms**. Chat landed this close bridge. **The Round-2 final pre-UI gate is CLOSED COMPLETE — verdict GO. The pre-UI chain is fully discharged; next-active = UI.** HEAD `145d2c8` (audit, pushed); this bridge is the next commit.

**Why a Pass 2 (not a re-run).** `tasks/ROUND_2_AUDIT.md` (COMPLETE, 2026-06-05) certified GO for an M8/M9/multiparty-era tree; its chain ended "M10 -> UI" and it put M10 out of scope. Three substantial arcs shipped after it — M10 (auth module ref set, J-375), M11 (`self`/D-021, J-378), M12 (attachments, J-379->J-389). The J-357 reconciliation re-inserted a second Round-2 gate after M12 for exactly this reason. The new doc `tasks/ROUND_2_FINAL_GATE_AUDIT.md` covers that delta + the cross-arc surface now that all three arcs exist; it **supersedes Pass-1's §6 verdict by reference** and leaves Pass-1 a clean COMPLETE record (structural lock: new doc, not a reopen — Joe).

**The five axes (all grounded file:line on `main`).** (1) New surfaces M10/M11/M12 as-built, clean — zero new state-mutation conflict domain. (2) Cross-arc coherence redux — coherent end-to-end. (3) Wire-code register — domain-10 (10001–10004) clean + typed; RC-F-01 discharged by M10. (4) Carried-forward Pass-1 register (R2-F01/F07/F09) holds (R2-F09 strengthened-as-anticipated: M11 self-thread + M12 same-identity fetch are the multi-device UI-prototype motivators the PULL named). (5) Routed-open sweep — caught that MP-F2-followon is only partially discharged.

**Chat cross-verify on `main` (D-065, not by faith).** **P3:** `message.file`/`message.redact` have no apply arm -> `_ => Ok(())` (state.rs:655). **P4:** `author_is_retained` (runtime.rs:623/636) reads exactly `module_policy().erasability.retention` — the chain M10.1 wrote. **P6 (the fresh positive claim, scrutinized hardest):** `federation_fetch_blob` (xgen-node/src/app.rs:2855) resolves holders ∩ live sessions and injects `BlobFetchRequest` — it never calls `connect_url`; the only production dial sites (establish app.rs:3857, reconnect reconnect.rs:379) sit below the `policy_permits` (3163) / `jurisdiction_permits` (3194) gate, so a blocked node has no live session -> can't be a holder. Arc-G containment transitive: real. **R2G-F03:** D-030 ×2 (L1833/L2241), D-031 ×2 (L1806/L2285). The MP-F2-followon split (auth half discharged by M10; the 7 event-validation codes still generic 4000, non-auth-band) is a genuine precision catch — J-357 conflated them.

**Four new findings (all S3/S4, none gates UI) — routings Joe-LOCKED by-recomms.** **R2G-F01** (S3, MP-F2-followon event-validation half: 7 codes -> generic 4000) -> carried into the UI error-surfacing pass (not a pre-UI arc; info already batch-observable via MP-F5). **R2G-F02** (S4, D-093 c3 forward-constraint) -> named UI build-constraint: attachment-forward/re-share MUST re-encrypt (fresh `blob_ref`), never reuse a descriptor. **R2G-F03** (S4, DECISIONS.md D-030/D-031 duplicates + non-monotonic order) -> a doc-hygiene pass, any time; **suffix the duplicates, do not renumber** (preserve cross-refs; respect no-retroactive-rewrite). **R2G-F04** (S4, `create-dm-space` undocumented in Appendix F, already self-recorded) -> fold into the same doc-hygiene pass. **Routed-open left on their named homes:** MP-F12 (departed-signer), MP-F13 (identity->home-node discovery), MP-F16 (federation endpoint), M12.3 throwaway `pending_fetches` (self-heals). **UI inherits explicitly** (named, not blockers): the R2-F01 residual A+thin-fetch (flagged-UNBUILT, node authoritative) + R2G-F02.

**Canonical (D-074).** `tasks/ROUND_2_FINAL_GATE_AUDIT.md` ACTIVE v1.0 -> COMPLETED v1.1 (Clair authored the body `145d2c8`; Chat's close note + Status flip ride this bridge); `CLAUDE.md` PLAY head (gate CLOSED, next-active = UI; the M12-close block retitled historical); `docs/ROADMAP.md` v3.78 -> v3.79 (gate done at tree / chain / detail; next-active = UI); this JOURNAL J-390. **No DECISIONS change** (all findings S3/S4; no new principle — the round-close discipline stays a standing promotion candidate). **No Appendix F change** (the gate touched no CLI verb).

**Next-active: UI** (clean-table build — the post-M12 chain's endpoint) -> Streams (standalone real-time plane, post-UI). All pre-UI work is now closed: Multiparty (J-356) + Round-2 checkpoint (J-357) + M10 (J-375) + M11 (J-378) + M12 (J-389) + this Round-2 final pre-UI gate (J-390). **Entry (Rule 0): CLAUDE.md PLAY -> JOURNAL J-390 -> `docs/ROADMAP.md` (UI next-active) -> `tasks/ROUND_2_FINAL_GATE_AUDIT.md` (COMPLETED) -> `tasks/ROUND_2_AUDIT.md` (Pass-1 baseline).** Audit `145d2c8` pushed; this bridge handed to Joe.

---

## Entry J-389 — M12.4 SHIPPED + CLOSED = M12 (attachments) CLOSED: erasure (redact + blob-delete + Retained-refusal); box-gated e2e green; next-active = Round-2 final pre-UI gate

**What happened.** Clair implemented M12.4 spine-first across three commits on top of the runbook `e2f7d42`; Joe pushed; the box-gated real-binary e2e RUN went green; Chat cross-verified on `main` (D-065) and landed this close bridge. **M12.4 (erasure) is SHIPPED + CLOSED — and with it the whole M12 (attachments) milestone CLOSES.** HEAD `8315f72`, tree clean, in sync with origin.

**What shipped (erasure, made real against attachments).** `message.redact` was a bare validated kind with no applier; M12.4 gives it one. On a redact, the node resolves the target `message.file`'s descriptor blob_refs and — gated on the **original content author's** `Retention` — either **deletes the blob bytes** (a real, complete erasure of that node's copy: the content-addressed `BlobStore` is mutable, separate from the DAG) or **refuses** on a Retained (T4) author (the legal-hold floor, `10004`). The redact event itself always converges; only the byte-delete side-effect is gated. The DAG-resident residue (event existence + plaintext descriptor key + text) is **not** erased — that's the D3 crypto-shred arc (WE6 boundary). The GDPR win lands on the mutable layer; the immutable residue is marked-and-reserved — the M12.3 "build the mechanism, fence the maturity" pattern.

**Commits (spine-first, per-commit DoD-gated).** runbook `e2f7d42` · **C1** `ccc267c` — `BlobStore::delete` (idempotent: `Ok(true)` removed / `Ok(false)` absent-no-op / `Err` malformed ref) + the typed `BlobError::ErasureRefusedRetained → 10004` (blob_store.rs:98; RC-F-01 re-grep confirmed 10004 free) · **C2** `16cb81f` — the spine: `build_message_redact_event` ({target_event_id}) + `NodeRuntime::resolve_redact_erasure` (a pure xgen-core decision fn at node/runtime.rs:574 — D1 target-id, V4 resolve target from the Space store, read `attachments` blob_refs, **D2/F2b read `target.sender`'s `Retention`** via `author_is_retained`, lenient: only explicit module-declared `Retained` blocks, absent/Erasable/no-module → erase per D-088 T1-max-erasable; **M12's first production `Retention` reader**) + the `process_inbound` Accepted-arm hook `apply_redact_erasure` (app.rs:3004; **D3 gate the side-effect not admission**; `RefusedRetained` → `10004` via `reject_signal` to a `LocallySubmitted` redactor; origin-agnostic → B's federated redact deletes B's cached copy = WE4) · **C3** `8315f72` — the redact CLI verb (`ops::redact` + 4-arm D-092: main.rs / app.rs / batch.rs / aicontrol.rs) + the minimal client tombstone (D6/V8: `fetch_attachments` skips a redacted `message.file`'s blob) + Appendix F v1.11→1.12 (Session 8) + the box-gated e2e (`#[ignore]`). **DoD each:** build 0-error · clippy `--all-targets --all-features -D warnings` clean · in-suite `cargo test --workspace` → **1463/0** (1448 + C1's 4 + C2's 10 + C3's parse test; e2e `#[ignore]`). **RED-on-revert recorded** for the C2 spine: neuter `author_is_retained` → WE2 fails (Delete not RefusedRetained) + WE3 fails (Retained bytes deleted = legal-hold breach); restored → green.

**Box-gated e2e RUN — green (Joe ran it).** `cargo test -p xgen-mptest --test m12_4_self_thread_redact_e2e -- --ignored`: `w_redact_erases_attachment_e2e ... ok` — "redacted attachment gone, non-redacted retrievable (real binaries)," 16.97s, on the real two-same-identity-client path.

**Chat cross-verify on `main` (D-065, not by faith).** `delete` + `10004` typed (blob_store.rs:98/192); `resolve_redact_erasure` verified line-by-line (node/runtime.rs:574-623 — reads `target.sender`'s Retention, lenient, only explicit Retained blocks); the hook fires in the Accepted arm after store + before fanout (app.rs:3325, side-effect not admission); the D8 hinge verified in code — `encrypt_blob` (blob.rs:59) calls `OsRng.fill_bytes` for both key + nonce per call, the `fresh_key_and_ciphertext_each_call` test asserts `c1 != c2` → `blob_ref` per-send-unique by construction; local/remote in sync.

**Witnesses WE1–WE6.** WE1 blob-bytes erased · WE2 F2b Retained refusal + `10004` (RED-on-revert) · WE3 convergence / Retained-bytes-kept (RED-on-revert) · WE4 federated erasure (origin-agnostic hook → B's cached copy) · WE5 shared-fate safety (fresh-key-per-send → distinct `blob_ref`) · WE6 D3 boundary (**close-claim, not a test**: bytes erased + refusal + tombstone, NOT the DAG-residue crypto-shred — that's D3).

**D8 / D-093 c3 honored with ZERO storage reshape.** `blob_ref = hash_uri(ciphertext)` is per-send-unique by construction (the deletable handle); `plaintext_hash` (exchange.rs:937) is the separate content-identity metadata — the invariant the crypto already guarantees, no salt, no dedup change. **Forward-constraint documented in code:** any future re-attach/forward must re-encrypt (fresh `blob_ref`), never reuse a descriptor (else the A-09 cross-fate-share hazard returns).

**Canonical (D-074).** design doc + runbook + audit `tasks/M12_4_ERASURE_{DESIGN,IMPL,PHASE0_AUDIT}.md` v1.0→1.1 → **COMPLETED**; master `tasks/M12_ATTACHMENTS_DESIGN.md` v1.2→1.3 → **COMPLETED** (M12 done); Appendix F v1.12 (carried by C3 / Session 8); `CLAUDE.md` PLAY head (M12 → CLOSED, live head); `docs/ROADMAP.md` **v3.77→v3.78** (M12 🟢→✅ at tree / chain / detail); this JOURNAL J-389. Code commits C1–C3 + runbook already on `main` (Clair authored, Joe pushed). **No DECISIONS change** (M12.4-D# arc-local D-069; **D-093 already authored at J-388**, all three clauses honored — esp. c3, the D8 zero-reshape).

**M12 (attachments) CLOSED — the four sub-arcs:** M12.1 (J-382) blob mechanism + self-thread slice · M12.2 (J-385) fetch/`--attach`/F6 size-gate + F9 data-root · M12.3 (J-387) federation fetch-blob-by-hash · M12.4 (J-389) erasure. The full attachment feature — send, fetch, size-gate, cross-home, erase — is shipped, pre-UI as planned.

**Next-active: the Round-2 final pre-UI gate** — the single whole-codebase audit (the Round-2 two-pass strategy's second pass) that gates UI, per the locked post-M12 chain. Then **UI** (clean-table build) → **Streams** (standalone real-time plane, post-UI). The pre-UI work is now complete: Multiparty tests (J-356) + Round-2 checkpoint (J-357) + M10 (J-375) + M11 (J-378) + M12 (J-389) all closed. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-389 → `docs/ROADMAP.md` (M12 closed + the post-M12 chain) → `tasks/M12_ATTACHMENTS_DESIGN.md` (COMPLETED) → `DECISIONS.md` D-093.** Pushed (Joe).

---

## Entry J-388 — M12.4 OPENED-grounded + Phase-0 audit DONE + design Joe-LOCKED: erasure (redact + blob-delete + Retained-refusal); M12.4-D1..D9; M12-D6 promoted → D-093; next-active = Clair runbook

**What happened.** Clair authored the M12.4 D-071 Phase-0 audit (audit seat) and committed it (`31e17aa`, pushed) — verdict GO, findings M12.4-A-01..09, six+ grounding facts, eight forks FK-1..FK-8 with recommendations. Chat opened the M12.4 design discussion on the eight forks + the M12-D6 promotion, recommended on each; Joe locked **all by-recomms** (two explicit Joe-locks: FK-2 the Retention-read target, and the M12-D6 promotion). Before authoring, Chat independently re-grounded the load-bearing claims on `main @ 31e17aa` (D-065) and authored `tasks/M12_4_ERASURE_DESIGN.md` (v1.0 ACTIVE) + the `DECISIONS.md` D-093 promotion + this bridge. M12.4 is the **fourth + final** M12 sub-arc; **M12.4 close = M12 close**. Doc-only; no code.

**The central enabling fact.** Attachments split content across **two stores of opposite mutability**: blob ciphertext lives in the content-addressed `BlobStore` (separate from the DAG, **mutable** → deleting bytes is a real, complete erasure of that node's copy, **buildable now**); the `message.file` event residue (existence + the plaintext per-blob key + any text) is **DAG-immutable** → crypto-shred, **D3-gated** (per M12-D10 + D-088's cascade). M12.4 builds the GDPR win on the mutable layer and marks-and-reserves the DAG residue to D3 — the M12.3 "build the mechanism, fence the maturity" pattern. This is why M12.4 is GO-able and not a repeat of Arc I's design-only close (J-253): there is a concrete mutable surface to actually erase.

**Chat re-grounding (D-065, on `main @ 31e17aa`).** `message.redact` is a bare validated kind (permission arm exchange.rs:792 → `SendMessages`, no-op validation exchange.rs:846 — no builder/schema/applier; F2a all net-new). `message.*` never mutates `SpaceState` node-side (`_ => Ok(())` state.rs:655 — the node's erasure job is the blob-delete; the display tombstone is client-side). `BlobStore` has **NO delete** (`new`/`put`/`get`/`contains` only, blob_store.rs:100/120/135/151 — `BlobStore::delete` is the load-bearing missing primitive). The author's `Retention` is reachable without new plumbing (`IdentityRecord.trust_assertion` registry.rs:45 → `module_policy().erasability.retention`), with **zero production `Retention` readers** (J-380/J-387 hold — F2b is M12's first). B caches federated blobs (`store.put(&bytes)` app.rs:1945 — a redact reaching B must delete B's cached copy). **No `blob_ref → events` reverse index exists** (the M12.3-audit finding holds — decisive for FK-8).

**Nine decisions Joe-LOCKED (M12.4-D1..D9, arc-local D-069 except D9).** **D1 (FK-1) redact schema = `{ target_event_id }` only** — the node resolves blob_refs from the target `message.file`'s descriptor (single source of truth); net-new builder + content shape. **D2 (FK-2, load-bearing, Joe-lock) F2b reads the ORIGINAL CONTENT AUTHOR's `Retention`** (the target message's sender), not the redactor's — per D-088/D-093 c2, retention is per-record: a Retained (T4) author's content can't be erased by anyone (including a lower-tier redactor), and a redactor can't elevate erasure power by being lower-tier; the *permission* to redact stays the separate, existing `SendMessages`/moderation gate. **D3 (FK-3, convergence spine, RED-on-revert) gate the SIDE-EFFECT, not admission** — the redact is always admitted/stored/fanned-out (a valid signed event); only the blob-erasure side-effect is `Retention`-gated (Retained → keep the bytes = legal-hold floor; Erasable/absent → delete); the display tombstone applies regardless; the INV-EXP/M8.6 lesson (tier-conditioned admission diverges, side-effect gates don't). **D4 (FK-4, exceeds the literal F2 "tombstone-only" lock, Joe-confirmed) tombstone AND delete-the-bytes** — tombstone the DAG event (the residue it can't erase — D3) + delete the blob bytes (the content it can erase now); F2a's spirit. **D5 (FK-5, mechanical) hook in `process_inbound`** after validate+store, sibling to M12.3's `BlobUploadEnd → store.put`; the runbook grounds the exact site. **D6 (FK-6) minimal client tombstone in M12.4** — a redacted message must not render its descriptor / fetch its blob (the user-visible witness); richer redaction UI deferred to UI. **D7 (FK-7, re-grep at build) reject/refusal codes in domain 10** — reserve **`10004 erasure_refused_retained`** (a typed `BlobError` variant, the M12-D9 parallel error type, not `ExchangeError`; mirrors M12.3's 10003) for the F2b refusal signal to the redactor; reuse existing codes for not-found/malformed; RC-F-01 re-grep the register before emitting. **D8 (FK-8 — SUPERSEDES the audit's scan/accept-the-hazard rec; resolved by D-093 c3) NO shared physical copy across erasure-fate** — the audit predates the T4-reasoning that produced D-093 c3. Reasoning: retention/erasability is per-record (D-093 c2); a single shared physical copy lets one record's policy silently override another's (a lower-tier erasure deleting bytes a T4 record holds = durability-floor breach; a T4 hold blocking a lower-tier record's valid erasure = right-to-erasure breach). A refcount/reverse-index manages shared-copy bookkeeping but **cannot resolve the tier collision** (can't honor "A held, B erasable" on one physical blob) — heavy AND insufficient. **Lock:** an attachment blob's physical copy may be shared only among references that share the same erasure-fate; **M12.4 v1 = no attachment dedup** (one physical copy per `message.file` send, each with its own deletable storage handle; the **content-hash retained as descriptor metadata**, not the storage key, so identical-file detection / policy-keyed dedup-within-a-shared-fate-set stays a future optimization — not a correctness fix); a redact deletes only that reference's own copy → the A-09 hazard can't arise. **Runbook-bound mechanism (grounded, not a Joe-lock):** how `blob_ref` is derived today (pure `hash(bytes)` doubling as the storage key?) decides clean-handle vs a small additive per-send-salt on the storage handle (content-hash stays pure for the metadata) — Clair grounds + picks; the invariant is the lock. **D9 (Joe-lock) M12-D6 PROMOTED → `DECISIONS.md` D-093.** Three bound clauses: (1) universal E2E / no protocol escrow; (2) "Retained (T4)" = ciphertext durability-floor + erasure-refusal, retain-and-produce reserved to the operator/module layer; (3) no shared physical blob copy across erasure-fate (the D8 corollary). M12.4 is the arc that first exercises it (D2/D3 = first enforcement read; D4/D8 = first mechanism); past the 3-recurrence bar (AH-D1 + D-088 lineage, carried J-381→J-387). M12-D6's design-doc flag flips to "promoted → D-093."

**Witnesses (M12.4; RED-on-revert; runbook firms).** WE1 blob-bytes erasure (redact → ciphertext gone from `blobs_dir` → fetch `10003`) · WE2 F2b Retained refusal (Retained author → bytes kept + `10004`; Erasable/absent → deleted; first `Retention` reader) · WE3 convergence (redact stored + fanned identically regardless of side-effect outcome) · WE4 federated erasure (B deletes its cached copy on the redact) · WE5 shared-fate safety (identical bytes → two physical copies → redact one, the other untouched) · WE6 D3 boundary (close-claim: M12.4 erases bytes + refuses-on-Retained + tombstones, NOT the DAG-residue crypto-shred — that's D3).

**Canonical (D-074).** audit `tasks/M12_4_ERASURE_PHASE0_AUDIT.md` (Clair, `31e17aa`, pushed); design doc NEW `tasks/M12_4_ERASURE_DESIGN.md` (v1.0 ACTIVE); **`DECISIONS.md` D-093 NEW** (the M12-D6 promotion, three clauses); `CLAUDE.md` PLAY head (M12.4 design-lock, live head); `docs/ROADMAP.md` **v3.76→v3.77** (M12.4 opened + designed at tree / chain / detail; M12-D6 → promoted); this JOURNAL J-388; `tasks/M12_ATTACHMENTS_DESIGN.md` M12-D6 flag → "promoted → D-093." **No Appendix F** (no CLI verb/flag changed at the design-lock; the redact verb — if the runbook adds one — carries its 4-arm D-092 + Appendix F obligations at impl).

**Next-active: Clair authors the M12.4 runbook** (`tasks/M12_4_*_IMPL.md`, the §3 scope, spine-first — re-grounds the anchors, grounds the `blob_ref` derivation + picks the D8 mechanism, picks the `10004` variant, defines the witness set) → Joe-lock the runbook values → implement spine-first, per-commit, Joe pushes each → Chat close-bridge → **M12.4 close = M12 close** → Round-2 final pre-UI gate → UI → Streams. No code until the M12.4 runbook lands + Joe-locks. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-388 → `tasks/M12_4_ERASURE_DESIGN.md` → `tasks/M12_4_ERASURE_PHASE0_AUDIT.md` → `tasks/M12_ATTACHMENTS_DESIGN.md` (M12-D6/D10) → `DECISIONS.md` D-093/D-088/AH-D1 → `docs/ROADMAP.md` (M12).** Not pushed — Joe pushes.

---

## Entry J-387 — M12.3 SHIPPED + CLOSED: federation fetch-blob-by-hash (D1 β-multiplex / D4 sync / D5 typed 10003); box-gated e2e green; next-active = M12.4 (erasure)

**What happened.** Clair implemented M12.3 spine-first across three commits on top of the runbook `e24bef9`; Joe pushed; the box-gated real-binary two-home e2e RUN went green; Chat cross-verified on `main` (D-065) and landed this close bridge. M12.3 — federation **fetch-blob-by-hash** — is **SHIPPED + CLOSED**. HEAD `709811f`, tree clean, in sync with origin.

**What shipped (the D1 β multiplex, taken head-on per audit A-02).** The `message.file` descriptor already federates (eager push); the **bytes** did not — the federation steady-state loop dropped blob `TransportMessage`s on its `Ok(_)` catch-all. M12.3 closes that gap: on a client↔node fetch **miss**, home B resolves the Space's federated holders (`home_node ∪ federation_nodes ∩ live sessions`, home_node-first, **serialize-one-fetch-per-peer** — P2, no `BlobChunk` wire change) and fetches the ciphertext **across homes over the established federation session** (no new conn, no re-handshake): an `OutboundMsg::BlobFetchRequest` injector + a shared **`PendingFederationFetches = Arc<Mutex<HashMap<NodeXgid, FetchSlot>>>`** registry (forced by `OutboundMsg: #[derive(Clone)]`, V5 — a `oneshot` can't ride a Clone enum). The federation loop gained **serve + collect + inject arms ahead of the `Ok(_)` catch-all** (the clean arm-add the design predicted — no Event-ordering entanglement; blob fetch/reply is its own request/response, concurrent with the Event push), threaded through both session-driver chains (receiver + reconnect-initiator) + the in-process harness. B serves the bytes (cached content-blind) or the typed **`10003 blob_unavailable`** on unreachable/timeout (D4 **synchronous**: B blocks the client fetch during the round-trip; inner timeout = `[sync].completion_timeout_seconds` 5s, outer client `fetch_blob` 2× — P3). `space_id` rides as an **additive `Option`** on the wire `BlobFetchRequest` (no existing-caller break — P1).

**Commits (spine-first, per-commit DoD-gated).** runbook `e24bef9` · **C1** `9a2332d` — type the `10003` reject (D5): `BlobError::Unavailable` → `to_wire_code` `Some((10003, "blob_unavailable"))` (blob_store.rs:86), replacing the defensive literal at app.rs:1919 (W2 baseline; byte-identical wire) · **C2** `df053f7` — the federation fetch path (D1/D4, the spine bulk) + the in-process spine witness · **C3** `709811f` — the box-gated real-binary two-home e2e (W1 headline, `#[ignore]`, sibling to `m12_2a_self_thread_e2e`). **DoD each:** build 0-error · clippy `--all-targets --all-features -D warnings` clean · in-suite `cargo test --workspace` → **1448/0** (1445 baseline + C1 unit + C2's two in-process witnesses; the e2e `#[ignore]` keeps the in-suite count unchanged). **RED-on-revert recorded:** C1 (drop the `to_wire_code` arm → fails) + C2 (neuter the collect arm → W1 `Err(())`, restored to green).

**Box-gated e2e RUN — green (Joe ran it).** `cargo test -p xgen-mptest --test m12_3_federation_fetch_e2e -- --ignored`: `w1_federated_blob_fetch_roundtrip_byte_identical ... ok` — "B fetched a federated blob byte-identical across homes from A," 21.09s, on the real two-home `harness-control` binaries. This is the W1 headline executed end-to-end.

**Chat cross-verify on `main` (D-065, not by faith).** Confirmed directly: C1's typed `10003` arm at blob_store.rs:86 + the unit `unavailable_wire_code_is_10003` passing; the app.rs:1919 defensive literal **removed** (no `blob_err(10003` remains); the federated-fetch miss path (`Some(sid) => federation_fetch_blob(...)` / `None => Err(BlobError::Unavailable)`, app.rs:1929/1948); the **serve arm ahead of the `Ok(_)` catch-all** (serve app.rs:2630 vs catch-all :2739); the `PendingFederationFetches` registry forced by `OutboundMsg: Clone` (fanout.rs:119); the **additive** `space_id: Option` on the wire `BlobFetchRequest` (wire/types.rs:235); both in-process witnesses green (`federated_blob_fetch_round_trip` W1-spine + `federation_fetch_unknown_space_is_unavailable` W4); the e2e `#[ignore]` (m12_3_federation_fetch_e2e.rs:60); local/remote in sync.

**Witnesses W1–W5.** W1 federated round-trip byte-identical (in-process `federated_blob_fetch_round_trip` + box-gated C3 RUN green) · W2 typed `10003` (C1 unit + the miss path) · W3 content-blind across homes (ciphertext at rest on B) · W4 never-leak (unknown/non-federated Space → `Unavailable`, never a cross-home query) · W5 self stays federation-free (M11/D-021 intact; the federation surface is additive, not a weakening of the self path).

**Named M12.3 carry-over (flagged, non-blocking).** The operator-manual **`federation initiate` admin verb passes a throwaway `pending_fetches`** (admin_ops.rs:1825-1830): a session it initiates **serves** blobs correctly, but a client-miss fetch routed over *that specific* session serves a graceful `10003` until the reconnect scheduler re-establishes the session with the shared registry (self-heals). Mechanism-first-scoped; reserved fix = thread the shared `PendingFederationFetches` through `AdminContext` like `federation_policy`. Flagged here for a future close.

**Canonical (D-074).** design doc `tasks/M12_3_FEDERATION_FETCH_DESIGN.md` v1.0→1.1 → **COMPLETED**; runbook `tasks/M12_3_FEDERATION_FETCH_IMPL.md` v1.0→1.1 → **COMPLETED**; `CLAUDE.md` PLAY head (M12.3 → CLOSED, live head); `docs/ROADMAP.md` **v3.75→v3.76** (M12.3 closed at tree / chain / detail); this JOURNAL J-387. Code commits C1–C3 + runbook already on `main` (Clair authored, Joe pushed). **No DECISIONS change** (M12.3-D# arc-local D-069; **M12-D6 stays a flagged DECISIONS.md promotion candidate — Joe's explicit call**). **No Appendix F** (no CLI verb/flag changed — the arc was wire-internal; `--attach`/`fetch` unchanged).

**Next-active: M12.4 (erasure)** — the `message.redact` **content applier** (F2a, net-new — `message.redact` is a validated kind today with no applier), the F2b sender `Retention` read (**M12's first production `Retention` reader** — zero readers exist today; the M12.3↔M12.4 boundary D3 reserved this to here), crypto-shred destroy-to-erase (D3-gated), the reserved WORM/legal-hold operator hook (F7). Opens with its own **D-071 Phase-0** (Clair's audit seat). Then M12 closes → Round-2 final pre-UI gate → UI → Streams. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-387 → `tasks/M12_3_FEDERATION_FETCH_DESIGN.md` (COMPLETED) → `tasks/M12_3_FEDERATION_FETCH_IMPL.md` (COMPLETED) → `tasks/M12_ATTACHMENTS_DESIGN.md` (M12-D8/D9/D10; M12.4 = the erasure scope) → `docs/ROADMAP.md` (M12).** Pushed (Joe).

---

## Entry J-386 — M12.3 OPENED-grounded + Phase-0 audit DONE + design Joe-LOCKED: federation fetch-blob-by-hash; M12.3-D1..D6; next-active = Clair runbook

**What happened.** Clair authored the M12.3 D-071 Phase-0 audit (audit seat) and committed it (`fee9271`, pushed) — verdict GO, findings M12.3-A-01..06, grounding ledger, six forks FK-1..FK-6 with recommendations, grounded to file:line on `main`. Chat opened the M12.3 design discussion on the six forks, recommended on each; Joe locked **all by-recomms**. Before authoring, Chat independently re-grounded the load-bearing claims on `main @ fee9271` (D-065 — the J-381..J-385 precedent of not building on Clair's grounding by faith) and then authored `tasks/M12_3_FEDERATION_FETCH_DESIGN.md` (v1.0 ACTIVE) + this bridge. Doc-only; no code; no DECISIONS change.

**Chat re-grounding (D-065, on `main @ fee9271`).** Four load-bearing claims verified directly: (1) the **10003 defensive literal** is at `app.rs:1919` — `blob_err(10003, "blob_unavailable")` on the client↔node fetch path's `BlobStore::get → Ok(None)`, with the comment that the typed `BlobError::Unavailable` + lazy fetch land at M12.3; `to_wire_code` maps only 10001/10002 today (10003 reserved-not-emitted in the enum, `blob_store.rs:74`). (2) the **federation steady-state loop catch-all** `Ok(_) =>` "silently ignore" is at `app.rs:2561`, inside a `tokio::select!` whose inbound `match` has explicit `Event`-bearing-fanout (→ `apply_federation_push`) + `IdentityReplicate` arms, with the outbound side (`OutboundMsg`/`out_rx`) at `app.rs:2567+` — confirming blob `TransportMessage`s are dropped on a federation session today, AND that **routing blob variants in is a clean arm-add ahead of the catch-all, not an Event-ordering entanglement** (the FK-1/β hinge). (3) **zero production `Retention` readers** (J-380 holds; the M12.3↔M12.4 boundary is real). (4) domain-10 used only by the blob band, **10003 free**, RC-F-01 clean.

**Six decisions Joe-LOCKED (M12.3-D1..D6, arc-local D-069).** **D1 (FK-1) fetch transport = β multiplex the established federation session** — add the fetch protocol as new inbound arms on the federation loop's `match` (replacing part of the `Ok(_)` drop), reusing the M12.1 `TransportMessage` blob variants (`BlobFetchRequest`/`BlobChunk`/`BlobFetchEnd`) on the same channel (relationship + transport + auth already exist; no re-handshake, no new conn lifecycle); replies ride the existing `OutboundMsg`/`out_rx`, correlated by `blob_ref`, concurrent-with not interleaved-into the Event push. **α ephemeral side-conn = the documented fallback** if multiplexing entangles with ordering at implementation. The **"which peer holds it"** resolution (Space `home_node`/`federation_nodes` vs the event `sender`'s home) is a **runbook-bound open item, NOT a Joe-lock** (a resolution-source detail like the M12.1 chunk-size). **D2 (FK-2, load-bearing; = the M12-D8 deferred lock) F3 = lazy-default** — B fetches the ciphertext on demand at first read-miss, not eager at descriptor-receipt; the M12-D8 lean made formal here. The **Retained(T4) eager/replicated override** stays coupled to the F7 durability floor but is a **named, unbuilt reserved hook** (no eager-replicate path built this arc). Honest caveat (recorded): eager would reuse the existing push machinery + is the materially smaller build (no fetch protocol, no miss signal) at the cost of every-home-stores-every-blob; lazy takes the F3 storage-efficiency posture deliberately. **D3 (FK-4, load-bearing; the M12.3↔M12.4 boundary) Retained floor = α reserve-the-hook, NO `Retention` read in M12.3** — the first production `Retention` reader stays at M12.4 (M12-D10). Resolves the surfaced M12-D8↔M12-D10 tension in favour of M12-D10 (mechanism-first; F7 "mark + reserve the hook, not build-the-vault"); the protocol-layer "Retained = ciphertext durability floor + erasure refusal" (M12-D6) is unchanged — M12.3 simply does not *read* `Retention` to drive replication. **D4 (FK-3) miss-signal = α synchronous** — B blocks the client's fetch during the federation round-trip; on timeout/genuine-absence serves the typed `10003`; **no new client wire**. `PendingBuffer` is event-shaped (keyed by `EventXgid`, node-side DAG admission) — a blob-miss is a different unit/waiter, so a held-pending signal would **mirror not extend** it; **β async held-pending reserved**. **D5 (FK-5, a confirm) type `10003`** — add `BlobError::Unavailable` → `to_wire_code` `Some((10003, "blob_unavailable"))`, replace the `app.rs:1919` literal; a **variant-add to the existing parallel `BlobError`** (M12-D9, not `ExchangeError`); first typed emission = the federated-read miss; RC-F-01 re-grep at build. **D6 (FK-6) single arc** — no M12.3a/b milestone split (one coherent mechanism); spine-first commits within. Witness posture = a **box-gated real-binary two-home federation witness** (headline, sibling to `m12_2a_self_thread_e2e`; the F2/F9 harness-control `add-peer`/`initiate` seam already drives federation) **+ an in-suite in-process witness** (spine) — the M12.2a split; the runbook firms the exact set.

**Witnesses (M12.3; RED-on-revert; runbook firms).** W1 federated round-trip (A authors `message.file` → descriptor federates to B → B miss → B federation-fetches from A → byte-identical; neuter the fetch arm → `10003`) · W2 typed-unavailable `10003` (replaces the defensive literal) · W3 content-blind across homes (ciphertext at rest on B; inherits M12.1 W2) · W4 never-leak (a third home C not sharing the Space/federation never receives or serves the blob) · W5 self stays federation-free (M11/D-021 intact; M12.3's federation surface is additive).

**Canonical (D-074).** audit `tasks/M12_3_FEDERATION_FETCH_PHASE0_AUDIT.md` (Clair, `fee9271`, pushed); design doc NEW `tasks/M12_3_FEDERATION_FETCH_DESIGN.md` (v1.0 ACTIVE); `CLAUDE.md` PLAY head; `docs/ROADMAP.md` **v3.74→v3.75** (M12.3 opened + designed, at tree / chain / detail); this JOURNAL J-386. **No DECISIONS change** (M12.3-D# arc-local D-069; **M12-D6 stays a flagged DECISIONS.md promotion candidate — Joe's explicit call**). No Appendix F (no CLI verb changed this arc — doc-only design-lock).

**Next-active: Clair authors the M12.3 runbook** (`tasks/M12_3_*_IMPL.md`, the §3 scope, spine-first — re-grounds the anchors, resolves the "which peer holds it" source, picks values, defines the witness set) → Joe-lock the values → implement spine-first, per-commit, Joe pushes each → Chat doc-bridge → **M12.3 close** → M12.4 (erasure) → M12 close → Round-2 final pre-UI gate → UI → Streams. No code until the M12.3 runbook lands + Joe-locks. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-386 → `tasks/M12_3_FEDERATION_FETCH_DESIGN.md` → `tasks/M12_3_FEDERATION_FETCH_PHASE0_AUDIT.md` → `tasks/M12_ATTACHMENTS_DESIGN.md` (M12-D8/D9/D10) → `docs/ROADMAP.md` (M12).** Not pushed — Joe pushes.

---

## Entry J-385 — M12.2 SHIPPED + CLOSED: M12.2b F9 data-root posture shift (D5+D6, both binaries) closes M12.2; next-active = M12.3 (federation fetch-by-hash)

**What happened.** Clair implemented the M12.2b F9 data-root posture shift spine-first per the J-383 design (D5+D6) + the runbook locks (VA..VF), gated green, and handed the close to Chat. Joe pushed; `main` HEAD `3dff95d`, tree clean. This is the Chat close bridge (two-seat: Clair commits her own code; Chat lands the canonical-record flips + the Appendix F flag rows). M12.2b is the last M12.2 sub-arc — **its close closes M12.2** (M12.2a J-384 + M12.2b J-385).

**Three commits on `main`.** runbook `94d0539` (Clair's deliverable, locks recorded) · **C1** `e391e22` **resolution shift (D5)** — a shared `xgen-common` data-dir module: the hand-rolled platform-default chain (`%LOCALAPPDATA%/XGenProtocol` on Windows, `$XDG_DATA_HOME` / `~/.local/share/XGenProtocol` on Unix; **fail-fast if no base resolves, never silently fall back to `exe_dir`** — VB), `--data-dir` flag + `XGEN_DATA_DIR` env (flag > env > default — VC; no config equivalent since it precedes config load), `--instance` rebases under the resolved root (VF), wired into **both binaries'** `resolve_data_dir` (**VA — the scope fork locked both**, shared logic in `xgen-common`, no drift); **+ the S-2 fix** — `cmd_init` now roots `spaces_dir`+`blobs_dir` at `data_dir` (app.rs:3819-3820), not `exe_dir` (Chat-verified on `main`): without it the written config kept the event DAG + blobs in the install folder, **silently defeating F9's whole point** — a pre-existing asymmetry corrected; **+ the S-3 pin** — the mptest `base_command` pins `--data-dir <exe_dir>` (process.rs:447, Chat-verified), the single chokepoint every real-binary spawn/restart inherits, so the breaking default change can't strand the box-gated tests' `exe_dir/instances/<label>` expectation (also covers the `m9_2_f2_add_peer` `exe_dir/spaces` reliance) · **C2** `3dff95d` **startup validation (D5/VD, the RED-on-revert spine)** — `xgen-common::validate_data_dir`: creatable (`create_dir_all`) + writable (write-probe) + not-tmp (reject under `temp_dir()`, canonicalized) → `DataDirError::{NotCreatable,NotWritable,UnderTemp}`, fail-fast `exit(1)` in both binaries; **+ the D6 leave-as-legacy notice (VE = built, not doc-only)** — `legacy_data_notice` fires only when the fresh platform default is used (no `--data-dir`/`XGEN_DATA_DIR`) **and** an old `exe_dir` layout (keypair or `instances/`) holds data, printing one stderr line naming the `--data-dir=<exe_dir>` escape; **no auto-migration**.

**Gate (every commit).** build 0-error · clippy `--all-targets --all-features -D warnings` clean · in-suite `cargo test --workspace` green → **1445/0** (1443 + W3 validate + W5 notice; the in-process `phase9_harness` uses tempdirs and never calls `resolve_data_dir`, so the default change leaves the in-suite count untouched). **C2 RED-on-revert recorded:** neutering the not-tmp branch → W3 fails at "a dir under the system temp dir must be rejected"; restored → 1445/0. **Box-gated e2e re-run 2/2** — the M12.2a real-binary self-thread e2e survives the breaking default change (the harness's `exe_dir` root passes create/not-tmp/writable; the S-3 `base_command` pin holds). **D-065 note (Clair, Rule 1):** Clair flagged a mid-session scare — she couldn't recall editing the harness/client/cmd_init this session — and chased it down rather than assuming done: a first grep was truncated, the second found the S-3 pin in `base_command` (the chokepoint), confirming all C1 scope landed. A false alarm correctly verified, not papered.

**Witnesses W1–W5 (in-suite; no box-gated RUN beyond the e2e re-run, unlike M12.2a):** W1 fresh-default resolves to the platform dir (not `exe_dir`); W2 `--data-dir`/env honored + precedence; W3 validation fail-fast (the spine, RED-on-revert); W4 `--instance` rebases under the resolved root; W5 legacy `exe_dir` layout reachable via `--data-dir=<old>`.

**Canonical (D-074).** Appendix F `docs/xgen_appendix_f_en.md` **v1.10→1.11** (new `--data-dir` row in F.0.1 fundamental flags + the F.0.6 precedence table; the `--instance` rows F.0.1 + F.8.1 re-worded from `<exe dir>/instances/<label>` to `<resolved data root>/instances/<label>`; Session 7 log); runbook `tasks/M12_2b_DATAROOT_IMPL.md` → **COMPLETED**; design doc `tasks/M12_2_DESIGN.md` **v1.1→1.2 → COMPLETED** (§4 M12.2b ✅ SHIPPED; the doc now covers a fully-closed M12.2); `CLAUDE.md` PLAY head; `docs/ROADMAP.md` **v3.73→v3.74** (M12.2 closed at tree / chain / detail); this JOURNAL J-385. **No DECISIONS change** (M12.2-D# arc-local D-069; **M12-D6 stays a flagged DECISIONS.md promotion candidate — Joe's explicit call**).

**M12 progress.** M12.1 (J-382, blob store + self-thread slice) + M12.2 (J-385, fetch verb + `--attach` polish + F6 gate + F9 data-root) done; **M12.3** (federation) and **M12.4** (erasure) remain. **Next-active: M12.3** — the federation **fetch-blob-by-hash** protocol (push is eager today; lazy fetch is net-new) + the **F3 lazy/eager lock** (M12-D8, deferred to here) + the **Retained(T4) durability floor** (F7) + the **held-pending / unavailable client signal** (extend `HeldPending`/`PendingBuffer`) + the reserved `BlobError` **`10003 blob_unavailable`** — its own D-071 Phase-0 → design → Joe-lock → runbook → implement. No code until the M12.3 design is Joe-locked. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-385 → `tasks/M12_ATTACHMENTS_DESIGN.md` v1.1 (M12-D8 F3 lazy-lean, M12-D10 M12.3 scope) → `tasks/M12_2_DESIGN.md` (COMPLETED, context) → `docs/ROADMAP.md` (M12).** Pushed (Joe).

---

## Entry J-384 — M12.2a SHIPPED + CLOSED: fetch verb + --attach polish + F6 gate (D2+D3+D4); the M12.1 self-thread e2e boundary discharged; next-active = Clair M12.2b runbook (F9)

**What happened.** Clair implemented the M12.2a blob-feature trio spine-first per the J-383 design + the runbook locks (C1→C4), gated green at every commit, ran the box-gated e2e on the box, and handed the close to Chat. Joe pushed; `main` HEAD `64b1e2c`, tree clean. This is the Chat close bridge (two-seat: Clair commits her own code + the box RUN; Chat lands the canonical-record flips + the Appendix F verb rows). M12.2a = the second M12 sub-arc; it **discharges the M12.1 honest boundary** (the full ops/CLI-layer self-thread attachment e2e now works through the real binaries).

**Five commits on `main`.** runbook `1170f85` (Clair's deliverable, locks recorded) · **C1** `ea609d5` **fetch verb (D2)** — `ops::fetch_attachments` was built-but-uncalled (M12.1 C4), so a thin 4-arm verb-add, no core/wire change: `ClientCommand::Fetch` (+alias `fetch-attachments`), `FetchAttachmentsArgs` retrofitted with `#[derive(clap::Args)]`, `--space`/`--room`/`--out-dir`(required), `cmd_fetch` summary (twin of `cmd_history`), all four D-092 arms (CLI / run-path / batch / aicontrol) · **C2** `b73efc8` **`--attach` polish (D3)** — `SendArgs.attach: Option<String>`→`Vec<String>` (multi-file, repeatable) + `text: String`→`Option<String>` (optional); a pure `validate_send_args` guard (require-one; **combined `--text`+`--attach` → error**, per the VC lock-change, D-065 no-quiet-data-loss); multi-file loop encrypt+upload → `Vec<Descriptor>` → the already-plural `build_message_file_event`; **`reconstruct_argv` gains a `Value::Array` arm (S-2** — verified on `main` that the aicontrol 4th arm had `Bool`/`Null`/`String` arms only, so multi-file `--attach` over aicontrol was silently broken → one bogus `--attach '["a","b"]'`); mime stays hardcoded (S-1, the dropped audit rec, not pulled in) · **C3** `da0e7e9` **F6 size gate (D4, the spine commit, RED-on-revert)** — `BlobError::TooLarge { size, limit }`→`to_wire_code` `(10002, blob_too_large)` (RC-F-01 re-grepped: 10002 was reserved-not-emitted, no collision) + `DEFAULT_MAX_BLOB_BYTES` = **16 MiB** shared const in `xgen-core` (VB) + `[node].max_blob_bytes` on `NodeSection` (serde-default; `NodeConfig::default` literal + the `config_reload` fixture re-rooted to avoid a 0-ceiling-rejects-everything trap) threaded `run_node`→`handle_connection` (+ both harness call sites + the respawn fn); **node-authoritative reject without buffering at `BlobUploadBegin.size` + a per-chunk accumulate-check** (defends a lying Begin); **one terminal reply at `BlobUploadEnd`** via a `blob_oversized` flag — a first-cut that replied at *both* Begin and End left a leftover `10001` frame that poisoned the connection, caught in test and redesigned to the single-reply form (witnessed by the under-ceiling round-trip reusing the same conn); client pre-check (S-3, conservative on the compile-time const; the node gate is authoritative) · **C4** `64b1e2c` **box-gated real-binary e2e** — `xgen-mptest/tests/m12_2a_self_thread_e2e.rs` (two `#[ignore]` tests): **W-e2e** `register`→`self`→`send --attach <300 KiB>` (client A) → `fetch --out-dir` (client B, same identity via the `[[rehome]]` `to="a"` = `spawn_client_reusing_keypair`) → fetched file **byte-identical** (multi-chunk); **W-multi** two `--attach` files → both fetched byte-identical (exercises D3 multi-file + the S-2 array arm end-to-end).

**Gate (every commit).** build 0-error · clippy `--all-targets --all-features -D warnings` clean · in-suite `cargo test --workspace` green → **1440/0** (1429 baseline + 11: C1 +4 clap/alias/required-out-dir/reconstruct-routing, C2 +6 validate-guard ×5 + array-reconstruct, C3 +1 `w_toolarge`; the C4 e2e are `#[ignore]`, not in the count). **C3 RED-on-revert recorded:** neutering both gate halves → the witness fails at "over-ceiling upload is rejected"; restored → 1440/0. Bonus finding: neutering *only* the Begin check still rejected via the accumulate-check — genuine defense-in-depth. **Box-gated RUN (Clair, on the box): W-e2e + W-multi green 2/2** — not box-free-claimed. A real first-run bug was caught and fixed by actually running it: both tests hard-coded node port 8560, so parallel `--ignored` execution collided on bind → the second node's pipe never started; fixed by per-test port parameterization (the mp_c06 convention). **The M12.1 ops-level / self-thread e2e boundary is discharged** — the thing M12.1 explicitly named as untested now passes through the real binaries with the fetch verb.

**Honest scope (D-065, flagged not papered).** W-toolarge-e2e (a real-binary node-gate too-large witness) deferred: `run_scenario` exposes no per-node ceiling knob, and with the 16 MiB default the client pre-check fires before the node gate — a faithful real-binary node-gate witness would need a manifest ceiling knob (heavy) or a 16 MiB+ file, for low marginal value over C3's in-suite `w_toolarge` (which has genuine RED-on-revert). The node gate is witnessed; only the *real-binary* variant of that witness is deferred.

**Canonical (D-074).** Appendix F `docs/xgen_appendix_f_en.md` **v1.9→1.10** (new `fetch` rows in F.0.4 + F.3; the `send` rows updated to the final M12.2a surface — `--text` optional, `--attach` repeatable, exactly-one-of-the-two; Session 6 log); runbook `tasks/M12_2a_FETCH_ATTACH_F6_IMPL.md` → **COMPLETED**; design doc `tasks/M12_2_DESIGN.md` **v1.0→1.1** (§3 M12.2a ✅ SHIPPED with the commit list; the doc stays ACTIVE for M12.2b); `CLAUDE.md` PLAY head; `docs/ROADMAP.md` **v3.72→v3.73** (M12.2a closed at tree / chain / detail); this JOURNAL J-384. **No DECISIONS change** (M12.2-D# arc-local D-069; **M12-D6 stays a flagged DECISIONS.md promotion candidate — Joe's explicit call**).

**Next-active: Clair authors the M12.2b runbook** (`tasks/M12_2b_*_IMPL.md`, the F9 data-root posture shift — D5: `--data-dir` flag + env override + a hand-rolled platform-dir default + startup validation; D6: leave-as-legacy + named `--data-dir=<old>` escape, no auto-migration) → Joe-lock → implement → Chat doc-bridge → **M12.2b close** → **M12.2 close** → M12.3 (federation) → M12.4 (erasure). No code until the M12.2b runbook lands + Joe-locks. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-384 → `tasks/M12_2_DESIGN.md` v1.1 (§4 M12.2b) → the M12.2 audit (FK-4/FK-5, the F9 groundings) → `docs/ROADMAP.md` (M12).** Pushed (Joe).

---

## Entry J-383 — M12.2 OPENED + Phase-0 audit DONE + design Joe-LOCKED: M12.2-D1..D6; split M12.2a (trio+e2e) / M12.2b (F9 data-root); next-active = Clair M12.2a runbook

**What happened.** Clair authored the M12.2 D-071 Phase-0 audit (audit seat) and committed it (`58c3a0b`, pushed) — verdict GO, findings M12.2-A-01..05, grounding ledger L1–L25, six forks FK-1..FK-6 with recommendations, grounded to file:line on `main @ 60cfd8f` (re-confirmed by reading code; the M12.1 anchors drifted across C1–C5). Chat opened the M12.2 design discussion on the six forks, recommended on each; Joe locked **by-recomms**. Before authoring, Chat re-verified the one load-bearing hinge (D-065): `data_dir` is resolved **before** config load (`try_load_config` runs with `data_dir` already passed in; only sub-dirs like `spaces_dir` are config-overridable), so a root-`data_dir` override must be a flag/env — confirmed. Chat then authored `tasks/M12_2_DESIGN.md` (v1.0 ACTIVE) + this bridge. The formal M12.2 OPEN rides this bridge (kickoff-sanctioned: open-and-design in one motion). Doc-only; no code; no DECISIONS change.

**Six decisions Joe-LOCKED (M12.2-D1..D6, arc-local D-069).** **D1 (FK-6) split.** M12.2 → **M12.2a** (the blob-feature trio — fetch verb + `--attach` polish + F6 gate — which lands the **full self-thread e2e** and discharges the M12.1 honest boundary; high-value, low-risk, client-feature-shaped) then **M12.2b** (the F9 data-root posture shift — a breaking node-ops default change + startup validation + legacy handling, isolated because it's orthogonal to the trio and shouldn't delay the e2e). M12.2a first; M12.2 closes when both close. **D2 (FK-1) fetch verb.** `ops::fetch_attachments` is **built but has zero callers** (C4), so this is a thin **4-arm verb-add, no core/wire change**: a new `fetch`/`fetch-attachments` `ClientCommand` with a by-message/thread selector (mirror `history`; reuses the built op, which already loops all attachments in scope), **output to a path/dir** named from the `Descriptor` filename (binary → never stdout), all four D-092 arms (CLI `main.rs` / run-path `app.rs` / batch `batch.rs` / aicontrol `aicontrol.rs` via `reconstruct_argv`) + an Appendix F entry; `FetchAttachmentsArgs` gains a clap derive (hand-built today). **D3 (FK-2) `--attach` polish.** **Surface-only** (the C2 builder takes `&[Descriptor]`; the fetch reader loops; mime is client-side): lock **both** multi-file `--attach` + attach-only sends (make `text` optional on `SendArgs`); client-side only, no core/wire change. **D4 (FK-3) F6 gate.** Placement = **both, node-authoritative**: the node rejects at **`BlobUploadBegin.size`** (carried-but-discarded today — the fail-fast hook), returning the reserved `BlobError` **`10002 blob_too_large`** *before* accepting chunks; the client also pre-checks for UX (the node gate is the real one — can't trust the client). Ceiling source = **a flat operator node-config ceiling now** (the `[sync].batch_size` precedent shape); F6's full Pattern-A **tier→size table + tighter per-Space immutable override** are **reserved as the named Pattern-A enrichment, NOT built in M12.2** (no tier→size map and no immutable-Space type exist today; the gate *mechanism* — enforce a ceiling, reject `10002` at `BlobUploadBegin` — does not reshape when the source later grows flat→tier-keyed; mechanism-first). **D5 (FK-4) F9 default + override.** Override = a **`--data-dir` flag + env var** (forced by the verified before-config-load ordering); default = a **platform data dir** (cleanly outside the install folder — `%LOCALAPPDATA%` on Windows / `$XDG_DATA_HOME` (or `~/.local/share`) on Linux + documented fallbacks), implemented **hand-rolled to avoid a new `dirs`-crate dependency**; + **startup validation** (net-new: present-or-creatable / writable / not-tmp, fail fast); `--instance` (`<root>/instances/<label>`) rebases under the resolved root. **D6 (FK-5) F9 existing-data handling.** **Leave-as-legacy + named** (the M10.4-D5 precedent): pre-existing `exe_dir()`-rooted deployments stay put, documented, with a `--data-dir=<old path>` named escape; **no auto-migration** (moving a live node's data is disproportionate risk for a reference impl); the new default applies to fresh deployments.

**Witness (M12.2a — discharges the M12.1 boundary).** The full self-thread e2e via `xgen-mptest` driving the **real binaries** over `.aicontrol` (`self` → `send --attach <file>` → `fetch` retrieves it **byte-identical** by a second same-identity client) — reachable once the fetch verb is a `ClientCommand` (no new crate edge: `xgen-node` doesn't dep `xgen-client`, but `xgen-mptest` already spawns both binaries); + a multi-file round-trip + the F6 `10002` reject, RED-on-revert.

**Canonical (D-074).** audit `tasks/M12_2_FETCH_GATE_DATAROOT_PHASE0_AUDIT.md` (Clair, `58c3a0b`, pushed); design doc NEW `tasks/M12_2_DESIGN.md` (v1.0 ACTIVE); `CLAUDE.md` PLAY head; `docs/ROADMAP.md` **v3.71→v3.72** (M12.2 opened + designed, split M12.2a/b, at tree / chain / detail); this JOURNAL J-383. **No DECISIONS change** (M12.2-D# arc-local D-069; **M12-D6 stays a flagged DECISIONS.md promotion candidate — Joe's explicit call**).

**Next-active: Clair authors the M12.2a runbook** (`tasks/M12_2a_*_IMPL.md`, the trio D2+D3+D4) → implement → Chat doc-bridge → M12.2a close (the e2e witnessed) → Clair authors the M12.2b runbook (F9, D5+D6) → implement → Chat doc-bridge → M12.2b close → **M12.2 close** → M12.3 (federation) → M12.4 (erasure). No code until the M12.2a runbook lands. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-383 → `tasks/M12_2_DESIGN.md` → the M12.2 audit → `tasks/M12_ATTACHMENTS_DESIGN.md` v1.1 → `docs/ROADMAP.md` (M12).** Not pushed — Joe pushes.

---

## Entry J-382 — M12.1 SHIPPED + CLOSED: blob store + per-blob crypto + chunked-base64 WS transfer + --attach/fetch + W1–W5; R-1/R-2 recorded; design doc v1.1; next-active = M12.2

**What happened.** Clair implemented the M12.1 slice spine-first per the locked §5 (C1→C5), gated green at every commit, and handed the close to Chat. Joe pushed; `main` HEAD `d4c1cc4`, tree clean. This is the Chat close bridge (two-seat: Clair commits her own code; Chat lands the canonical-record flips + the design-doc framing corrections). M12.1 = the first M12 sub-arc, the **federation-free self-thread blob slice**.

**Six commits on `main`.** runbook `d4143e1` (Clair's deliverable) · **C1** `49748d3` content-addressed `BlobStore` (content-blind; hash-keyed put/get/contains) + `BlobError` (net-new parallel type, **domain 10 = 10000–10999**; `10001 blob_hash_mismatch` live, 10002/10003 reserved M12.2/M12.3 — the M12-D9 principle, band grounded collision-free vs the ≤∼6xxx register) + `blobs_dir` on `PathsSection` (`#[serde(default)]` for pre-M12 configs; default `<data_dir>/blobs` under today's `exe_dir()` data_dir per M12-D7's M12.1 decoupling) · **C2** `da72bd7` `xgen-core/src/encryption/blob.rs` (fresh per-blob **ChaCha20Poly1305** key — the Arc-H Phase-2 primitive, D-052-consistent) + `Descriptor` + `build_message_file_event` (reuses `EventType::MessageFile`, **no new event kind** = the F5/M12-D2 answer) · **C3** `6d38360` the long pole — chunked-base64 transfer over **WebSocket** (R-2): 6 `TransportMessage` blob variants (begin/chunk/end + fetch) + `Connection::upload_blob/fetch_blob` + node WS arms before the `Inbound::Transport(_)` catch-all + `BLOB_CHUNK_BYTES`=**128 KiB** (V1; base64 ≈ 174 KB < the 256 KB frame ceiling) · **C4** `7eb6cf4` `--attach` on `SendArgs`/`ops::send` (write) + `ops::fetch_attachments` (read); D-092 4-arm coverage free because `Send` is clap-parsed in all four arms · **C5** `d4c1cc4` W1–W5 witnesses against the **real node via a real WS client** (incl. a genuine second-same-identity-client fetch for W1's locked framing).

**Gate (every commit).** build 0-error · clippy `--all-targets --all-features -D warnings` clean · `cargo test --workspace` green → **1429/0** (1405 baseline + 24: C1 +8, C2 +9, C3 +3, C4 +1, C5 +3). RED-on-revert recorded for the spine witnesses W2 (content-blindness) / W3 (on-disk corruption → hash-mismatch) / W4 (chunked reassembly).

**Two grounding findings recorded (D-065; Chat independently re-grounded both on `main` before the lock at J-381+ε; neither overturns a locked decision — each corrected an imprecise grounding the design carried from the audit).** **R-1 — crypto-maturity boundary.** `ops::send` ships text as **plaintext** content today; client-side `enc:` live-encryption is D3-fenced (the client holds no epoch key). So honouring M12-D3's "descriptor inside `enc:`" *now* would make attachments **stronger than text**, violating M12-D5 flag-1. Resolution (locked): M12.1 ships per-blob **byte encryption real** (ciphertext at rest), but the `Descriptor` — **including the per-blob key** — rides as **plaintext `message.file` content**, matching `message.text`; the `enc:`-wrap of the descriptor activates for text *and* blob descriptors **together** at the shared D3/M8.7 cutover (zero blob-store rework). **W2** sharpened: *"ciphertext-at-rest + store content-blind-by-construction"* — **not** *"the node can't get the key"* (the key is in the plaintext descriptor today, exactly as text is plaintext; both go node-blind together at D3). **R-2 — transfer channel = WebSocket, not the named pipe.** The client reaches the node **only over WS** (`home_node` is a `ws://` URL; sends ride `Connection::send_event_confirmed`); there is no client→node pipe — the named pipe is the control-mode↔resident **driver** channel (the `--attach` CLI invocation path, not the byte path; the 7 client-side "pipe" hits are all the resident pipe-server / control-mode drivers per D-056). M12-A-01's "client→home over the pipe" was grounded-imprecise. The **M12-D1 chunked-base64 *encoding* lock is unchanged** (WS frames are JSON/text too — raw bytes can't ride without base64); only the framing surface moved to WS `TransportMessage` variants, and the binary-frame rejection still holds (transfer stays inside the uniform JSON `TransportMessage` model). → Design doc corrected **v1.0→1.1** (M12-D1 channel correction + M12-D5 R-1 maturity boundary + M12-D3 forward-pointer).

**Honest boundary (D-065 — named + routed to M12.2, not papered over).** C5 witnesses the **load-bearing mechanism** (per-blob crypto + chunked-base64 WS transfer + content-blind store) end-to-end against the real node + real on-disk `blobs_dir` + a real second-same-identity-client fetch. The thin `xgen-client` **ops glue** (`ops::send --attach` / `ops::fetch_attachments`) is component- + clap-parse-tested but has **no full ops-level / self-thread e2e**: no crate links both `xgen-client` and `xgen-node` in-process (`xgen-mptest` drives the binaries), and the real-binary fetch path needs the **M12.2 fetch CLI verb**. W5 / self-DM federation wall = M11/D-021 inherited unchanged; **M12.1 adds no federation surface**. This is the expressible M12.1 witness given the grounded crate constraint; the full self-thread e2e rides M12.2 once the fetch verb lands.

**Canonical (D-074).** design doc `tasks/M12_ATTACHMENTS_DESIGN.md` **v1.0→1.1**; runbook `tasks/M12_1_BLOB_STORE_ATTACH_IMPL.md` → **COMPLETED**; `CLAUDE.md` PLAY head; `docs/ROADMAP.md` **v3.70→v3.71** (M12.1 closed at tree / chain / detail); this JOURNAL J-382. **No DECISIONS change** (M12-D# arc-local D-069; **M12-D6 stays a flagged DECISIONS.md promotion candidate — Joe's explicit call, not this arc**).

**Next-active: M12.2** — `--attach` surface polish + the **fetch CLI verb** (unblocks the full self-thread e2e the M12.1 boundary named) + the **F6 blob size gate** (`blob_too_large`, the reserved `10002`) + the **F9 default-outside-install data-root posture shift** (M12-D7: `--data-dir` override + startup validation, touching every node file + `--instance`) — its own D-071 Phase-0 → design → Joe-lock → runbook → implement. No code until the M12.2 design is Joe-locked. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-382 → `tasks/M12_ATTACHMENTS_DESIGN.md` v1.1 → the M12.1 runbook (COMPLETED) → `docs/ROADMAP.md` (M12).** Pushed (Joe).

---

## Entry J-381 — M12 design Joe-LOCKED: M12-D1..D10; chunked-base64 / reuse message.file + Descriptor / universal-E2E protocol-layer; next-active = Clair M12.1 runbook

**What happened.** With the M12 Phase-0 audit done (J-380, GO), Chat opened the design discussion on the five audit-teed inputs — sequenced pipe-shape-first (the gate) then the E2E philosophy lock — recommending on each. Joe locked all five + the reject-type principle **by-recomms**. Before authoring, Chat honoured a stated precondition of its own 4b recommendation (D-065): the universal-E2E call hinged on the Arc-H *text* path being universal-no-escrow, so Chat grounded that on `main` first (confirmed — see D6). Chat then authored `tasks/M12_ATTACHMENTS_DESIGN.md` (v1.0 ACTIVE) + this bridge. Doc-only; no code; no DECISIONS change (M12-D# arc-local, D-069; D6 a flagged promotion candidate).

**Grounding confirmation (D-065, J-381).** Arc-H text encryption is universal E2E with no escrow / no tier-conditioning / no recovery key: zero escrow/tier/retention hits across `xgen-core/src/encryption/`, and the defended invariant `erasing_wrapped_key_defeats_epoch_holder` (AH-D1 constraint 2, `client_mls.rs`) makes the per-message content key random and never epoch-derived — destroying the wrapped CK is permanent even for the epoch holder. The 4b recommendation's precondition holds; the universal-E2E lock is faithful to AH-D1, not a new posture.

**Ten decisions Joe-LOCKED (M12-D1..D10, arc-local D-069 unless marked).** **D1** pipe byte-transfer = **chunked base64** (the gate, M12-A-01) — a net-new transfer sub-protocol *inside* the existing text-line model (base64-chunked bounded lines + begin/size/sentinel framing, symmetric upload/fetch); chosen over length-prefixed binary (breaks the UTF-8-line invariant both pipe surfaces depend on) and single-field base64 (multi-MB single line + whole-file-in-memory, a later reshape against F1). **D2** message seam = **reuse `message.file`** (already validation-wired; no new event kind = the F5 answer; `file.upload`/`message.attachment-meta` doc-only; `stream.*`/`media.*` clean doc reservation); build the missing `build_message_file_event` carrying `attachments: [Descriptor]` (F1 plural; rides DAG+fanout like `message.text`, no `SpaceState` apply arm). **D3** net-new **`Descriptor`** = `blob_ref` (ciphertext hash = content-address / store key) + `plaintext_hash` (post-decrypt integrity) + `key` (per-blob) + `filename`/`mime`/`size`, carried **inside** the `enc:` E2E content. **D4** **blob store** = content-addressed `blobs_dir` `PathsSection` sibling, default `<data_dir>/blobs` (events + referenced blobs back up / snapshot / tier as one unit — the T4 requirement); content-blind (ciphertext only); `EventStore` untouched (event-only). **D5** blob encryption = **same Arc-H envelope as text** (fresh per-blob key → encrypt → upload ciphertext; node content-blind, content-addresses by ciphertext hash; key + plaintext-hash ride the descriptor inside `enc:`; same group reads text + blob, no new key distribution). *Flag 1:* inherits the text path's D3 crypto maturity (demonstration-grade now; production RFC 9420 HPKE = the M8.7 S+L openmls arc) — interface now, prod crypto when text's D3 lands; not stronger/weaker than text. **D6 (DECISIONS.md PROMOTION CANDIDATE)** E2E philosophy = **universal at the protocol layer** — every tier crypto-shreddable, **no protocol escrow key** (grounded above). T4 **retain-and-produce** (F7/F8 WORM / legal-hold) is **NOT** a protocol escrow; it is **reserved to the operator/module layer** (the F7 WORM hook). Resolves the F2/F7/F8 ripple in one move: F2 crypto-shred = a real protocol guarantee everywhere (destroy the per-blob key → every replica, incl. unreachable federated homes, permanently unreadable); F7/F8 "Retained" at protocol = a **ciphertext durability floor + erasure refusal**, with plaintext producibility an operator/module-tier concern — consistent with institutional-independence + "mark + reserve the hook, don't build the vault." Principle-shaped (reinforces AH-D1 + D-088); **flagged for Joe's explicit DECISIONS.md promotion**, not auto-promoted (the established no-DECISIONS-change-unless-Joe-promotes pattern). **D7** F9 data-root posture = **adopt but decouple from M12.1** — today `data_dir = exe_dir()` with no override; M12.1 uses `blobs_dir` as a sibling under today's `data_dir`; the full default-outside-install + `--data-dir` override + startup-validation shift (touches every node file + `--instance`) lands at M12.2 / a named node-config step. **D8** F3 federation = lazy-lean provisional, **lock deferred to the M12.3 grounding** (M12.1/M12.2 never federate; lazy blob fetch-by-hash is net-new — push is eager today); Retained(T4) eager/replicated override coupled to the F7 durability floor; `HeldPending`/`PendingBuffer` = the model to extend for the lazy-miss / unavailable client signal. **D9** blob rejects (blob-too-large F6 / blob-unavailable F3 / hash-mismatch) = a **new parallel error type at the transfer/ingest gate**, NOT `ExchangeError` (which gates the signed envelope; the small descriptor event still rides it) and NOT `StoreError`; the **code band is picked at build**, grounded against the register (RC-F-01 / M10.1 collision discipline). **D10** F4 sub-arc split confirmed: **M12.1** (blob store + Descriptor + `build_message_file_event` + chunked-base64 transfer + same-Arc-H encryption + `--attach` into the `self` thread = headline witness, intra-home, **never federation** → M11/D-021 intact) → **M12.2** (`--attach` surface polish + 4 D-092 arms + F6 size gate + the F9 posture shift) → **M12.3** (fetch-blob-by-hash + the F3 lazy/eager lock + Retained durability floor + held-pending/unavailable signal) → **M12.4** (build the `message.redact` content applier [none today, F2a] + the F2b sender-`Retention` read [M12's first production reader of dormant AI-D8; T4/`Retained` refuses] + crypto-shred destroy-to-erase [D3-gated] + the reserved WORM/legal-hold operator/module hook).

**M12.1 is shovel-ready** = the next runbook. Design §3 gives the end-to-end witness path (`--attach`→`self_open` → per-blob key + encrypt + hash → chunked-base64 upload to the content-blind store → `message.file` descriptor event → second same-identity client fetches + decrypts + verifies). §4 witnesses (RED-on-revert): W1 byte-identical round-trip / W2 content-blindness (ciphertext at rest) / W3 hash-integrity / W4 chunked-transfer fidelity / W5 never-federates. `--attach` threads through `ops::send` once → inherits all four D-092 arms.

**Canonical (D-074).** design doc NEW (`tasks/M12_ATTACHMENTS_DESIGN.md` v1.0 ACTIVE); `CLAUDE.md` PLAY head; `docs/ROADMAP.md` v3.69→v3.70 (M12 design-lock annotation at tree / chain / detail); this JOURNAL J-381. **No DECISIONS change** (M12-D# arc-local; D6 flagged promotion candidate, not promoted).

**Next-active.** Clair authors the M12.1 runbook (`tasks/M12_1_*_IMPL.md`, the design §3 scope) → implement → Chat doc-bridge → M12.1 close → M12.2 → M12.3 → M12.4 → M12 close → Round-2 final pre-UI gate → UI → Streams. No code until the runbook lands. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-381 → `tasks/M12_ATTACHMENTS_DESIGN.md` → the audit → the brief → `docs/ROADMAP.md` (M12).** Not pushed — Joe pushes.

---

## Entry J-380 — M12 Phase-0 audit ✅ DONE (Clair, GO): forks F1–F9 grounded; Chat doc-bridge records audit-done; next-active = design

**What happened.** Clair shipped the M12 D-071 Phase-0 audit `tasks/M12_ATTACHMENTS_PHASE0_AUDIT.md` (v1.0 ACTIVE, findings M12-A-01..09), committed audit-only at `504a808` (pushed; `main` @ `fdbaa8d` at audit time, clean tree). This is the Chat doc-bridge recording audit-done in the canonical records (CLAUDE PLAY / JOURNAL / ROADMAP) — the two-seat separation (Clair commits the audit, Chat records it), the M10 J-359 precedent. Doc-only; no DECISIONS change.

**Verdict: GO.** Every fork F1–F9 grounded to file:line against `main`. The minimal **M12.1 self-thread slice** = four small, self-contained, net-new pieces (A-01/02/03/08) and **never touches federation** → M11/D-021 intact; shovel-ready once the one load-bearing fork is decided.

**Independently re-grounded (D-065, J-352 precedent).** Chat re-verified the three material claims before recording: (A-01) pipe = 22 `read_line` text reads, **0** binary reads in the pipe layer; (A-02) `MessageFile` / `"message.file"` present (4 wire.rs hits), `file.upload` / `message.attachment-meta` / `build_message_file_event` = **0** code hits; (A-03) `resolve_data_dir` → `exe_dir()`, no data-root override flag (only `--instance` segregating *under* `exe_dir`). All confirmed.

**Load-bearing finding (M12-A-01, agenda item 3).** The pipe is **line-delimited UTF-8 text** (`read_line`→String + `write_all(as_bytes())`) on both surfaces (`pipe.rs` `__BATCH__` + `aicontrol.rs` JSONL). No binary / length-prefix / chunked transfer exists → **raw file bytes cannot ride it** (contain `\n`, not UTF-8). Gates **even the federation-free M12.1 slice** (bytes still move client→home over this pipe). Three design candidates: base64-in-JSONL / length-prefixed binary frame / chunked base64. **The design's first decision and the M12.1 long pole.**

**Two brief-refinements + one (D-065, routed to design).** (1) Attachment kind = **`message.file`** (validation-wired but unbuilt — no `build_message_file_event`; the descriptor attaches as `attachments: [Descriptor]` content); the brief's `file.upload` / `message.attachment-meta` are doc-only. Also the **F5 answer: reuse `message.file`, no new kind**; `stream.*` / `media.*` are doc-only → clean reservation. (2) **F9 "default outside the install folder" is a genuine new convention** — today `data_dir = exe_dir()` (the install folder), no override flag; a deliberate posture shift to adopt, not an extension. (3) Blob rejects → a **new parallel error type**, not `ExchangeError` (fold the RC-F-01 / M10.1 wire-code-collision discipline into the band choice). None contradicts a locked fork.

**Everything else net-new, as F1–F9 anticipated.** Erasure: `message.redact` validation-wired but **no applier** (F2 builds it); zero production readers of `Retention` / `module_policy` → **F2b would be the first** (M12-A-05). GC/TTL: append-only store, **no lifecycle** → F7/F8 net-new (M12-A-07). Blob store: `EventStore` event-only → net-new, a clean `PathsSection` / `blobs_dir` sibling under `data_dir` (M12-A-03). Federation: push **eager**, no fetch-by-hash → F3 lazy net-new; `HeldPending` / `PendingBuffer` is the lazy-miss UX seam (M12-A-06). Size: flat 256 KB frame ceiling only; §3.1.1 tier-table + `max_event_size` **unwired** → the blob gate is a parallel transfer/ingest gate (M12-A-04).

**Canonical (D-074).** audit doc NEW (Clair, `504a808`, already pushed); this JOURNAL J-380; `CLAUDE.md` PLAY head (audit-done); `docs/ROADMAP.md` v3.68→v3.69 (M12 audit-done annotation at tree / chain / horizon). No DECISIONS change.

**Next-active: design (Chat/Joe).** The three load-bearing decisions the audit teed up — (1) the pipe byte-transfer shape (M12-A-01, the long pole), (2) adopt `message.file` + the `Descriptor` schema (M12-A-02), (3) the F9 data-root posture shift (M12-A-03) — plus the F3 federation lean (confirmed at the M12.3 grounding). → Joe-lock → Clair runbook → implement (sub-arc'd M12.1–M12.4) → Chat doc-bridge per arc → close. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-380 → the audit → the brief → `docs/ROADMAP.md` (M12).** Not pushed — Joe pushes.

---

## Entry J-379 — M12 (attachments) OPENED: concept + Phase-0 scope + forks F1–F9 Joe-LOCKED; next-active = Clair D-071 Phase-0 audit

**What happened.** M12 opened cold (M11 closed at J-378; `main` clean, tree green 1405/0). After a full concept discussion, Joe LOCKED the attachment concept, the Phase-0 grounding scope, and **nine forks (F1–F9)**. Chat authored the framing brief `tasks/M12_ATTACHMENTS_PHASE0_BRIEF.md` (v1.0 ACTIVE). Doc-only; no code; no DECISIONS change (M12 design not started; J-357 + D-021/D-056/D-088 are the standing sources).

**Provenance grounded (D-065, J-379).** J-357 was a planning lock (existence + placement), not a scope lock. Grep of `main`: **no blob store today** (event-only `EventStore` over a SQLite `events` table — every "blob" hit is MLS / Arc-H content-blindness); **§3.1.1's size model is envelope-only and mostly unwired** (only the 256 KB frame ceiling enforced; the per-tier table + Space `max_event_size` are spec/stored-but-not-enforced); **`ModulePolicy`** (`claims.extra["module_policy"]`, `erasability` first member) is the dormant extensible switch-bag with **zero production readers**; `file.upload` / `message.attachment-meta` are event-kind breadcrumbs.

**Concept Joe-LOCKED.** Metadata-by-value / content-by-reference: the event carries a **descriptor** (hash + name + mime + size); the bytes live in a **net-new content-addressed blob store**. Bytes never ride the event — three structural reasons: the §3.1.1 size ceiling; the immutable signed DAG (`event_id` = hash of the event's bytes); erasability impossibility + federation cost. **Send = upload-and-persist** to the home node (not link-to-source, not Skype-style direct device→device) — the source device is irrelevant after upload, download needs no second device online. The **`self` thread is M12's front door**: intra-home multi-device sharing rides blob store + pipe upload + pipe fetch and **never touches federation**, so M11/D-021's never-federated guarantee stays intact; it is the headline witness for M12.1.

**Nine forks Joe-LOCKED (F1–F9, arc-local D-069).** **F1** descriptor = a multi-file list from day one (`attachments: [Descriptor]`; schema plural, build single-first if useful). **F2** GDPR/deletion = **F2a** tombstone-only lean (inherit the Arc-I shape + D-088 + AI-D8 erasability; cross-federation erasure is request-not-guarantee) with **F2b** read-sender-retention escape (T4/`Retained` refuses erasure — would make M12 the first reader of the dormant AI-D8 enforcement). **F3** federation = lazy lean, **audit-grounded-not-locked** (decided at design after the pipe/federation grounding), **overridden toward eager/replicated for Retained (T4)**. **F4** open monolithic; anticipated split **M12.1** local store+descriptor+pipe round-trip (self-thread witness) → **M12.2** pipe transfer + `--attach` (4 D-092 arms) → **M12.3** federation → **M12.4** erasure/tombstone. **F5** namespace reservation (attachment event-kind, steer clear of the streams band; ground `file.upload` / `message.attachment-meta` first). **F6** blob size = **Pattern A** tier-derived spec ceiling (MB-scale) + tighter-only immutable Space override, mirroring §3.1.1; enforcement at transfer/ingest; values Phase-0. **F7** storage model + retention-aware lifecycle = upload-to-content-addressed-store; reclaim/GC tier-retention-aware (T1 reclaimable+erasable; T4 pinned/legal-held/undeletable, WORM-shaped); retention sets a durability floor overriding F3's lazy lean for retained tiers; tiering/offload (Retained blob moves to a cheaper archived store to reclaim primary space while staying immutable) = a **reserved operator/module hook** — M12 marks + reserves, does not build the WORM vault. **F8** lifetime = Pattern-A tier-set TTL, two modes — lower tiers a reclaim deadline (kept *at most*), T4 a retention minimum / legal-hold (kept *at least*, undeletable, overrides erasure); values Phase-0. **F9** blob store rooted as an **event-log sibling under one durable node data root** (`<data_root>/events.db` + `<data_root>/blobs/` — events + referenced blobs back up / snapshot / tier as a unit, the T4 requirement), **defaults outside the install/system folder**, operator-overridable to any absolute path/volume, startup-validated (durable, writable, not-tmp); **node config, never tier-module, never assertion-set**; reserved archive/offload path/hook (pairs F7); per-object size/lifetime stays per-blob tier-driven (shared root ≠ shared retention). **Pattern-A spine** across F6/F8/F9: size/lifetime/placement keyed by tier (spec/operator), NOT the `ModulePolicy` switch-bag — a module attests the tier, the spec maps tier→hard ceilings, the operator owns placement; a module must not loosen a hard ceiling, shorten a legal-hold, or set a local path.

**Real-world grounding (J-379 web check).** Joe's T4 model = **WORM + legal hold + tiered archival**, validated against government records practice (NARA / Federal Records Act retention; WORM admin-proof immutability; time-based interval vs legal-hold-until-cleared, the hold overriding an expired interval; immutability independent of storage tier → tier-to-archive to reclaim space). Reference-impl posture = mark + reserve the offload hook, not build the WORM vault.

**Routed (survive beyond M12).** **Pattern-B "module-as-policy-bearer"** — reconsider §3.1.1 message-size (and other hard limits) as a tier-auth-module-defined limitation carried on the `ModulePolicy` switch-bag; sibling of the parked **erasure-via-general-setting** idea — named on the ROADMAP horizon, **not invented in M12**. The WORM/archival backend itself = operator/module responsibility (M12 reserves the hook). Carry-over (pre-existing): client UX for federation-derived held-pending/unavailable signals (a lazy-fetch miss surfaces here, F3); federation-under-load stress measurement.

**Canonical (D-074).** brief NEW (`tasks/M12_ATTACHMENTS_PHASE0_BRIEF.md` v1.0 ACTIVE); `CLAUDE.md` PLAY head; `docs/ROADMAP.md` v3.67→v3.68 (M12 🟡→🟢 at tree / chain / horizon + a new Pattern-B horizon line); this JOURNAL J-379. No DECISIONS change (M12 design not started; J-357 + D-021/D-056/D-088 standing).

**Next-active.** Clair opens the **M12 D-071 Phase-0 audit** (the brief = its agenda) → design → Joe-lock → runbook → implement (likely sub-arc'd M12.1–M12.4) → Chat doc-bridge per arc → close → Round-2 final pre-UI gate → UI → Streams. No code until the design is Joe-locked. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-379 → `tasks/M12_ATTACHMENTS_PHASE0_BRIEF.md` → `docs/ROADMAP.md` (M12) → ch3 §3.1.1.** Not pushed — Joe pushes.

---

## Entry J-378 — M11 (`self` thread, D-021) SHIPPED + CLOSED: 3 Clair code commits + the doc-bridge close (D4 ch6 §6.16 note + canonical flips); next-active = M12

**What happened.** Clair shipped the M11 build in three code commits on `main`, gate green; Chat ran this doc-bridge close (the D4 ch6 note + the canonical-record flips, one-writer atomic per D-074). Doc-only on Chat's side; no DECISIONS change (M11-D1..D5 arc-local, D-069); D-021 reconciled.

**What shipped (Clair, 3 commits, pushed).** **`2c2bc4c`** — D1 guard at both DM constructors (`from_dm_space_create` / `from_dm_space_create_node`, `xgen-core`): skip the auto self-invite when `invitee == creator`; constructor-only, no `apply_join` belt-and-suspenders (the `AlreadyMember` short-circuit already neutralizes a stray self-invite). The entire applier/protocol delta — two sites, a few lines, no wire/event/reject change. Witnesses W1a/W1b (no `pending_invites[self]`, RED-on-revert), W2 (creator Owner + dm-Room member), W3 (`DmFederationNotAllowed`); non-self regression held. **`092ddb9`** — D5 `self` verb + label + D2 wording (`xgen-client`): `ClientCommand::SelfThread` (clap `"self"`), `ops::self_open` (resolve session identity → create-if-absent scan for the `"self"`-labelled KnownSpace → open, or run the create chain when absent), all four D-092 dispatch arms (CLI / run-path / batch / aicontrol). `create_dm_space` labels the KnownSpace `"self"` when `invitee == creator`, so the raw `--invitee <own-id>` floor and the verb converge on one core / one label, no drift. Witnesses V-idempotent / V-autotarget. **`ebc2bf6`** — W4 reach witness (`xgen-node`, test-only): the self-DM is served via member-gated `collect_sync_history`; a second sync as the same identity sees it (the D2 reach property).

**Gate (final).** `cargo build --workspace --all-targets` 0 · `cargo clippy --workspace --lib --tests --all-features -- -D warnings` clean · `cargo test --workspace` **1405/0** (+6 over the 1399 baseline = W1a/W1b/W2/W3 + V-idempotent/V-autotarget + W4).

**Joe-lock checkpoints resolved by-recommendation (runbook §2, C1–C6).** C1 label-based create-if-absent detection · C2 one create-core with the label parameterized (floor + verb converge) · C3 the swallowed wire self-invite left as the named-benign D1 residue (suppressing it is out of D1's two-site scope) · C4 `name="self"` storage key / "Saved Messages" display · C5 W4 its own test-only commit · C6 variant `SelfThread` / clap `"self"`. One grounding resolved an open design question: D5 needs **all four** dispatch arms (a `self` verb is a real new `ClientCommand`, sibling-shape to `CreateDmSpace`).

**Doc-bridge close (this entry, D-074 atomic).** D4 ch6 note authored: `docs/xgen_ch6_client_design.md` v0.3→0.4, new **§6.16 "The `self` thread (Saved Messages)"** (self-DM shape; reuses-existing-identity anchor; never-federated / never-broadcast by `DmFederationNotAllowed`; reach = any client authenticated as the user — their own devices, Node-resident not device-local; attachments inherited at M12; boundary = no new wire/event/reject surface) + a Session 9 log entry; ch6 header brought to the full mandated structure (Date / Language / License added). **Canonical flips:** `tasks/M11_SELF_THREAD_{DESIGN,PHASE0_AUDIT,IMPL}.md` → COMPLETED (v1.1; IMPL carries the close banner); this PLAY/JOURNAL; `docs/ROADMAP.md` v3.66→v3.67 (M11 🟢→✅ at all three sites: tree / index / detail); `docs/xgen_appendix_f_en.md` v1.8→v1.9 (the `self` verb added to the F.0.4 Client-only list + the F.3 detailed reference, Session 5 — a missing close deliverable caught by Joe at close, per the thin-verb-arc Appendix-F convention J-334; the runbook §6 close list had omitted it). No DECISIONS change. D-021 reconciled (registered-via-existing-identity / never-federated spirit preserved; the pre-machinery "never registered" clause relaxed).

**Next-active.** **M12 — attachments** (the pre-UI mechanic; opens its own D-071 Phase-0), then the Round-2 final pre-UI gate → UI → Streams (the J-357 reconciled chain). **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-378 → `docs/ROADMAP.md` (M12) → `docs/xgen_ch6_client_design.md` §6.16.** Not pushed — Joe pushes.

---

## Entry J-377 — M11 (`self` thread, D-021) design Joe-LOCKED: shape B self-DM; D1 guard-at-construction (the entire applier delta) + D2–D5; next-active = Clair runbook

**What happened.** Phase-0 audit (`tasks/M11_SELF_THREAD_PHASE0_AUDIT.md` v1.0, read-only, grounded @ `345a461`) returned the verdict **B admissible — C not needed**. Chat authored `tasks/M11_SELF_THREAD_DESIGN.md` (v1.0 ACTIVE); Joe locked M11-D1..D5 by-recomms. Doc-only; no code; no DECISIONS change (M11-D# arc-local, D-069).

**Phase-0 verdict (audit-grounded).** `from_dm_space_create` (state.rs:342) + `from_dm_space_create_node` (state.rs:487) both admit `invitee == creator` — no guard, no error, no break. The creator is sole Owner + dm-Room member the instant the create chain lands (`apply_room_create` auto-inserts the sender, state.rs:792); **no `membership.join` is ever needed.** The only artifact is a vestigial `pending_invites[self]`, inert (`apply_join` short-circuits `AlreadyMember`, state.rs:1000-1001). The creator passes every step-11 gate (registered 629 / Space member 672 / Room member 676) → posts + reads. `create-dm-space --invitee <own-id>` works on current `main` (app.rs:543); the auto-invite's swallowed reject is every DM's accept-either behaviour (ops.rs:786-787, 740-742) — self introduces no new failure. Reach = member-gated `collect_sync_history` (fanout.rs:457). Non-federation doubly contained (`DmFederationNotAllowed` state.rs:660-661 + degenerate `{this_node}` party set runtime.rs:2101). Identity reuse, zero new registration (ops.rs:670-677).

**Decisions Joe-LOCKED (M11-D1..D5, arc-local D-069).** **D1** vestigial self-invite = **guard at construction** — skip the auto-invite when `invitee == creator` in both constructors; creator still seated via `apply_room_create`; **constructor-only, no `apply_join` belt-and-suspenders** (the `AlreadyMember` short-circuit already neutralizes a stray self-invite). **This is the entire applier/protocol delta for M11** — a few lines, two sites, no wire/event/reject change. **D2** reach wording locked precisely: "any client authenticated as the user (their own devices)," Node-resident not device-local (D-065 honesty; no code). **D3** client surface = a thin `xgen-client` convenience + "self"/"Saved Messages" label over existing `create-dm-space` + `Send`/`History`; no new wire, no applier change beyond D1. **D4** the ch6 descriptive note = the close deliverable (reuses-existing-identity anchor + attachments-inherited-M12 + "not an account / no new protocol surface"); NOT a ch3 normative edit. **D5** self-target UX = **(a) a `self` convenience verb** (create-if-absent → open; auto-resolves the session identity, no typed id); the raw `--invitee <own-id>` form stays the documented floor.

**Witness set (RED-on-revert).** W1 no `pending_invites[self]` on self-DM create (revert the guard → vestigial entry returns) · W2 creator Owner + dm-Room member + can post/read · W3 never-federates · W4 a second client authenticated as the same user sees the thread.

**Why B (faithfulness, recorded).** Reuse the convergence-proven primitive (zero protocol/applier delta) · `self` = the user's real registered identity on both endpoints (no-anonymity pillar) · the **hard** `DmFederationNotAllowed` wall (privacy as a structural property, not a default). C (single-member regular Space) would need a new creation path + has only a default non-federation posture; its lone conceptual edge is satisfied by the keypair-reuse + the D1 guard.

**Out.** Attachments → M12 (inherited). Operator-confidentiality → moot for B (audit §7). Renaming the internal DM primitive for the one-party case → named-not-fixed (D-069). Any new wire/event/reject/ch3 edit → none.

**Canonical (D-074).** design doc NEW (`tasks/M11_SELF_THREAD_DESIGN.md` v1.0 ACTIVE); `CLAUDE.md` PLAY head; `docs/ROADMAP.md` v3.65→v3.66 (M11 design-lock marker); this JOURNAL J-377. No DECISIONS change. (The Phase-0 audit is already on `main`, pushed at its own commit.)

**Next-active.** Clair authors `tasks/M11_SELF_THREAD_IMPL.md` — the D1 guard (two sites) + the D5 `self` verb + W1–W4 witnesses + the ch6 note as the close deliverable → implement → Chat doc-bridge → close. No code until the runbook lands. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-377 → `tasks/M11_SELF_THREAD_DESIGN.md` → `tasks/M11_SELF_THREAD_PHASE0_AUDIT.md`.** Not pushed — Joe pushes.

---

## Entry J-376 — M11 (`self` thread, D-021) OPENED: concept Joe-LOCKED (Node-side never-federated self-DM, reuses the user's existing keypair); Phase-0 scope locked; next-active = Clair D-071 Phase-0 audit

**What happened.** M11 opened cold (M10 closed at J-375; `main` clean). After a full concept discussion, Joe LOCKED the `self`-thread concept and the Phase-0 grounding scope. Chat authored the framing brief `tasks/M11_SELF_THREAD_PHASE0_BRIEF.md` (v1.0 ACTIVE) and superseded `tasks/HANDOFF_M11.md` (→ COMPLETED). Doc-only; no code; no DECISIONS change (M11 design not started; D-021 is the standing source).

**Provenance grounded (D-065).** A fresh grep of ch0–ch6 + appendices A–L returns ZERO detailed `self` grounding (all "self" hits unrelated). The sole source is D-021 (a terse 2026-04-28 deferral note, "Spec reference: —"); ROADMAP carries name-only placeholders. M11 builds something the spec never described — which legitimizes relaxing D-021's pre-machinery wording (no specified mechanism is overridden).

**Concept Joe-LOCKED.** `self` = a Node-side, never-federated, never-broadcast personal thread reusing the user's EXISTING keypair (single identity — `self` is *you*, not a second account); streams omitted; realized on the existing Space/Room/Event/DAG + DM-non-federation machinery; chronological history + "reachable from any client on the Node" are free properties of the Space apparatus; M12 attachments inherited (text-first at M11). Reconciles D-021 by relaxing the single pre-machinery clause "never registered" → registered via the already-registered identity, keeping the spirit (own keypair, never federated, private, Node-mediated reach). Chose **B (reuse keypair)** over A (own synthetic key — would re-shape "my space" into "a separate account" + itself need registration) and C (single-member regular Space — the named fallback; lacks the DM path's hard `DmFederationNotAllowed` guarantee).

**Grounding findings (verified main tree, J-376).** (1) a client Identity is a device-local keypair file (`ClientIdentity::load`, session.rs:60) — Node-side `self` makes reach free via Space-sync. (2) `validate_event` step 11 rejects an unregistered signer (`UnknownSender`, exchange.rs:202-209) — B reuses the registered identity, satisfying the gate. (3) DM never federates (`DmFederationNotAllowed`, runtime.rs:2105) — the non-broadcast property is already built; BUT `from_dm_space_create` (state.rs:342) has NO `invitee == creator` guard (verified — only wrong-type / missing-field errors), so a self-DM's admissibility is unproven — the one real unknown.

**Phase-0 scope (Joe-LOCKED).** Headline = the self-DM admissibility edge (`from_dm_space_create` → `apply_join` when invitee == creator), which decides B vs the C fallback. Supporting grounding in order: (1) registration cost (expect zero new registration under B); (2) DM-creation entry point; (3) reach (Space-sync to any client); (4) client surface in `xgen-client`/ch6. Named deliverable = a short ch6 descriptive note (reuses-existing-identity anchor line + attachments-as-inherited-M12 + "not an account / no new protocol surface"), authored at close — NOT a ch3 normative edit.

**Forks Joe-LOCKED.** F1 shape = target B, fallback C. F2 registration = none (provisional, Phase-0 confirms). F3 scope = text-first, attachments at M12. The keypair fork and the data-shape fork fold into one unknown: whether the DM machinery tolerates invitee == creator decides self-DM (B) vs single-member Space (C).

**Canonical correction.** ROADMAP's "M11 identity-layer; rides M10 auth-module work" tag is corrected at this bridge — under B, M11 is a client/Space feature reusing an existing identity; it does not touch the auth-module work.

**Canonical (D-074).** brief `tasks/M11_SELF_THREAD_PHASE0_BRIEF.md` NEW (v1.0 ACTIVE); `tasks/HANDOFF_M11.md` ACTIVE→COMPLETED (v1.1); `CLAUDE.md` PLAY head; `docs/ROADMAP.md` v3.64→v3.65 (M11 🟡→🟢 + tag correction); this JOURNAL J-376. No DECISIONS change.

**Next-active.** Clair opens the M11 D-071 Phase-0 audit against the locked B shape (this brief is the agenda) → design → Joe-lock → runbook → implement (text-first) → Chat doc-bridge → close. No code until the design is Joe-locked. Then the J-357 chain: M11 → M12 → Round-2 pre-UI gate → UI → Streams. **Entry (Rule 0): CLAUDE.md PLAY → JOURNAL J-376 → `tasks/M11_SELF_THREAD_PHASE0_BRIEF.md` → DECISIONS.md D-021.** Not pushed — Joe pushes.

---

## Archive pointer (D-094)

Entries **J-375 and older** have been relocated to **`JOURNAL_ARCHIVE.md`** (ARCHIVED)
to keep this live window on the recent arc (M11 → M12 → Round-2 → doc-opt).
Live window: **J-395 … J-376** (20 entries). Archive: **J-375 … J-046** (358 entries).
Pure byte-exact relocation — no entry altered, reordered, or lost.
