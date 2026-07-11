# M-RP6.1e-C3 — Help→About assembly (Help menu + About dialog + logo) build runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

For Clair. **Third and final step** of M-RP6.1e-C (Phase-0 J-488 / D-107 / `docs/xgen-client-frame-phase0.md` §4.1, §6, §10.4). Design **locked by Joe** this session (design walk; F1 + F2 confirmed "by your recomm").

**The C-split:** C1 `dialog` core ✅ CLOSED (J-496) · C2 `get_about_info` ✅ CLOSED (J-497) · **C3 Help→About assembly ← this runbook.**

**C3 is the UI half — plus a small opener footprint.** It renders the C2 data (`get_about_info`) inside the C1 `dialog`, reached from a new **Help** menu, with the client logo. It is *mostly* `ui/**`, but F1 (the website link opening the OS browser) adds a Tauri plugin + capability + a `desktop.rs` `.plugin()` line — so C3 is **not** scope-clean-of-Rust. That footprint is deliberate and bounded (§2.6).

---

## 1. Goal

Assemble the shipped pieces into the working About experience:

- a **Help** menu with one item **About** (no accelerator),
- the C1 `dialog` mounted in the frame, opened by `help.about`,
- an About **body** (shell-local) rendering the C2 `get_about_info` payload,
- the **client logo** (canonical `_hda` master),
- the website **link** opening the **OS browser** (not an in-app blank webview).

**Verify in the real client 9222** (D-097 — the sampler has no `get_about_info` and cannot host the shell frame).

---

## 2. Locked design

### 2.0 What already exists (ground it, Rule 5)

- **`dialog` core (C1, J-496)** — `ui/core/lib/components/data-independent/dialog.svelte`. Props `{ title?, open?=$bindable(false), closeLabel?='Close', onClose?, id?, children }`. Root is the native `<dialog>` + `showModal()`; **modal-only**, no `open` *attribute* ever set; composes its own Close `button` (self-registers `…__close`). `open` is reconciled both ways (guarded `$effect` prop→element + a `close` listener element→prop). Getter reads `open` from `el.open` (the DOM). **Children are always mounted** (closed = `display:none`), so the About body registers on mount and stays registered across open/close.
- **`get_about_info` (C2, J-497)** — `xgen-client/src/desktop.rs`, returns `ClientAboutInfo { common: AboutInfo }`. `AboutInfo` fields: `name, version, link, built, commit, rustc, tauri, svelte, platform, app_dir, data_dir, config_path` (all `String`). Baked literals: `name:"XGen Client"`, `link:"https://www.alchemydump.com"` (Joe-confirmed KEEP, this session). `built` is a commit-triggered stamp (last compile), `commit` is the exact short SHA (C2 §2.4).
- **`app_client.svelte`** — `ui/client/src/app_client.svelte`. Drives the frame: `<MenuBar {menus} onCommand={runCommand}/>` over `<main class="app-center">` over `<StatusBar/>`. `menus` is `[{ label:'File', items:[{label:'Exit', accelerator:accelerator('Ctrl+Q'), command:'app.exit'}] }]`. Dispatch is `onCommand → runCommand(commandId) → commandTable[commandId]?.()`. `commandTable = { 'app.exit': handleQuit }`. Tauri calls are **lazy-imported inside handlers** (`handleQuit`, `handleResizeGrip`) so the browser-dev preview (no Tauri) keeps working.
- **`$assets`** → `ui/assets/` (all three shells, `vite.config.js`). CSS is imported in `main.js`; a bundled image is `import url from '$assets/<file>'` → Vite returns the hashed URL string.
- **Logo masters** — `logo/logo_proto_01_client_hda.png` (1000×1000, alpha, canonical) + `logo/logo_proto_01_client_hd_small.png` (physically simplified ≤16px). `ui/assets/` today holds only the old `logo_client_64.png` (leave it — likely the window icon; not C3's concern).
- **Core display components** — `image` (`{ src, alt }`, both required), `link` (`{ href, text, onclick?, external?, disabled?, ariaLabel?, id? }`), `label` (`{ text, id? }`). All under `$core/components/data-independent/`.

### 2.1 About body lives shell-local (D1)

New **`ui/client/src/about-dialog.svelte`** — a client component wrapping the core `dialog`. It renders client-specific data (`ClientAboutInfo`, `"XGen Client"`, the *client* logo), so it is **not** a `core` component (core stays app-agnostic — the `link`/menu-bar rule). Keeping it a dedicated file (not inlined into `app_client.svelte`) keeps the shell lean.

Shape:

```svelte
<script>
  import Dialog from '$core/components/data-independent/dialog.svelte';
  import Image from '$core/components/data-independent/image.svelte';
  import Link  from '$core/components/data-independent/link.svelte';
  import Label from '$core/components/data-independent/label.svelte';
  import logoUrl from '$assets/logo_client_hda.png';

  let { open = $bindable(false), info = null, onOpenLink } = $props();
  // info = ClientAboutInfo | null (null in browser-dev / pre-fetch).
  const c = $derived(info?.common ?? null);
</script>

<Dialog bind:open title="About XGen Client" id="about">
  <!-- logo · name · version · "Developed by Alchemy Dump" (F2) · website link -->
  <!-- then the metadata grid (§2.3) -->
</Dialog>
```

`title="About XGen Client"` renders in the dialog header (the C1 `<h2 class="dialog-title">`). The Close button is C1's composed child — **do not** add another.

### 2.2 Rows: core components where they fit + shell CSS grid (D2)

- **logo** → `<Image src={logoUrl} alt="XGen" id="about-logo" />`, CSS-sized ~96–128px (about-dialog skin, not `image`'s job).
- **website** → `<Link href={c?.link} text={c?.link ?? ''} external={true} onclick={handleLink} id="about-link" />` (F1 wiring in §2.5).
- **"Developed by Alchemy Dump"** (F2) → a **static shell literal** (plain skin text), NOT a field on `AboutInfo`. Joe-confirmed: company, never the personal name. Do **not** reopen C2's Rust to add a company field.
- **metadata values** (dynamic) → each a `<Label>` with a **stable id** so verify reads it from the registry getter (the UI-side analogue of C2's field-by-field truth check). Keys ("Built", "Rust", …) are **plain skin text** (static chrome, not data — they don't register).
- **layout** → a 2-column CSS grid (`key | value`) in the about-dialog skin (`<style>` local, or a keyed `.about-*` rule — match the shell's existing style approach; `app_client.svelte` uses `app.css`/skin tokens, so prefer a scoped `<style>` in `about-dialog.svelte` for the About-specific grid).

Null-guard every value: `{c?.version ?? '—'}` (browser-dev / pre-fetch shows em-dashes, never crashes — the `get_state`-failure precedent).

### 2.3 Field list + the Built/commit pairing (D4)

Render, in this order (label id in parens):

| Key (plain text) | Value (`<Label>`) | id |
|---|---|---|
| Version | `c.version` | `about-version` |
| Built | `c.built` **· `c.commit`** (one row, middot separator) | `about-built` |
| Rust | `c.rustc` | `about-rustc` |
| Tauri | `c.tauri` | `about-tauri` |
| Svelte | `c.svelte` | `about-svelte` |
| Platform | `c.platform` | `about-platform` |
| App dir | `c.app_dir` | `about-app-dir` |
| Data dir | `c.data_dir` | `about-data-dir` |
| Config | `c.config_path` | `about-config` |

**Built + commit render together** (frame §6 / C2 §2.4): `built` is the last compile, `commit` is what exactly identifies the build. One "Built" row = `2026-07-11 08:42 · 50b5640` (a single `<Label>` whose `text` is the composed string, so it stays one registry entry). Paths are long — let the grid value cell wrap / `overflow-wrap: anywhere` (they must be readable, not truncated silently).

### 2.4 invoke timing — on mount (D3)

`app_client.svelte` fetches `get_about_info` **once on mount** (alongside the existing `get_state` / `get_substitutions` invokes, inside the same `try`), stores it in `$state`, and passes it as the `info` prop to `<AboutDialog>`. Rationale: About data is static per session; the 3rd startup invoke is sub-ms local; the dialog stays synchronous (no loading/null-race state beyond the `?? '—'` guard). On invoke failure (browser-dev) `aboutInfo` stays `null` → em-dashes.

```js
// inside the existing onMount try { … }
aboutInfo = await invoke('get_about_info');
```

### 2.5 Menu + command wiring (D6)

In `app_client.svelte`:

1. **Second menu** — append to `menus`:
   ```js
   { label: 'Help', items: [{ label: 'About', command: 'help.about' }] }  // NO accelerator (F1 = Help contents conventionally)
   ```
2. **Command** — `commandTable['help.about'] = () => (aboutOpen = true);` (a new `let aboutOpen = $state(false);`).
3. **Mount** the dialog in the frame (inside `.app-frame`, a sibling of `<main>`/`<StatusBar>`):
   ```svelte
   <AboutDialog bind:open={aboutOpen} info={aboutInfo} onOpenLink={handleAboutLink} />
   ```

Opening a modal via a command that flips a bound `open` state is exactly "a button flipping an open state" — link.svelte's own doc says modal-open is a command/button action, **not** a link. No new mechanism; reuses `onCommand → runCommand → commandTable`.

### 2.6 F1 — the website link opens the OS browser (opener plugin)

`<Link external>` sets `target="_blank"`, but inside the Tauri webview that spawns a **blank in-app webview**, not the OS browser (link.svelte documents this). Correct behavior = `onclick` → open the URL in the OS browser via the **Tauri opener plugin**. This is C3's one dependency/capability cost (Joe: "by your recomm", wants to see it function).

Add on **four** surfaces:

1. **`xgen-client/Cargo.toml`** — `tauri-plugin-opener = "2"` under `[dependencies]`.
2. **`ui/client/package.json`** — `"@tauri-apps/plugin-opener": "^2"` under `dependencies` (then `npm install` so `package-lock.json` resolves — note: this bumps the committed lockfile, which is also what C2's `build.rs` reads for the Svelte version; the Svelte entry is unaffected, but **re-verify `get_about_info().svelte` still returns `5.55.5`** after the install, §4).
3. **`xgen-client/capabilities/default.json`** — add the opener permission to `permissions`. **⚠️ Rule-6 grounding point:** the exact identifier is version-sensitive — `tauri-plugin-opener` exposes `opener:default` (allows `open-url`/`open-path` under a default scope) and a narrower `opener:allow-open-url`. **Confirm the correct identifier + whether a URL scope entry is needed on contact** (do not guess; `pnpm tauri add opener` / the plugin README is the source of truth). Prefer the **narrowest** grant that lets `openUrl(external https URL)` work.
4. **`xgen-client/src/desktop.rs`** — register `.plugin(tauri_plugin_opener::init())` in the `tauri::Builder` chain (next to `tauri_plugin_process::init()`).

Wiring, in `app_client.svelte` (lazy-import inside the handler — the `handleQuit`/`handleResizeGrip` pattern, so browser-dev keeps working):

```js
async function handleAboutLink(e) {
  e?.preventDefault?.();               // stop the <a> target=_blank in-app-webview path
  try {
    const { openUrl } = await import('@tauri-apps/plugin-opener');
    await openUrl('https://www.alchemydump.com');  // or info.common.link
  } catch (err) { console.error('Open link failed:', err); }
}
```

The `<Link onclick={handleAboutLink}>` preventDefaults and routes to `openUrl` — the exact consumer-wiring link.svelte prescribes. Keep `external={true}` (retains the correct `rel`/right-click-open a11y; the onclick overrides the in-app-webview behaviour).

*(Confirm the JS import path — `@tauri-apps/plugin-opener`'s `openUrl` — against the installed package on contact; older snippets use `open` from `@tauri-apps/plugin-shell`. Use the **opener** plugin, matching the Rust crate.)*

### 2.7 Logo assets (D5)

- Copy `logo/logo_proto_01_client_hda.png` → **`ui/assets/logo_client_hda.png`** (canonical mark). Import via `$assets` (§2.1), render through `image` (§2.2).
- Copy `logo/logo_proto_01_client_hd_small.png` → **`ui/assets/logo_client_hd_small.png`** — **reserved, NOT wired** (frame §4.1-C3; first consumer = M-RP8 title-bar app icon). Copying it now keeps the "don't downscale the wrong master later" rule enforceable.
- **Node** masters (`_node_hda` / `_node_hd_small`) are **left to M-RP7.x** (the node inherits the frame there). C3 is client-only.
- Match the existing `logo_client_*` naming; do **not** touch `logo_client_64.png`.

---

## 3. Files to touch (indicative — Clair confirms exact paths)

**UI:**
1. `ui/client/src/about-dialog.svelte` — **new**: the About body (§2.1–2.3), wraps core `dialog`.
2. `ui/client/src/app_client.svelte` — Help menu + `help.about` command + `aboutOpen` state + mount `<AboutDialog>` + `get_about_info` on mount + `handleAboutLink` (§2.4–2.6).
3. `ui/assets/logo_client_hda.png` — **new** (copied master, canonical).
4. `ui/assets/logo_client_hd_small.png` — **new** (copied master, reserved).
5. `ui/client/package.json` (+`package-lock.json`) — `@tauri-apps/plugin-opener` (§2.6).

**Rust/config (F1 footprint):**
6. `xgen-client/Cargo.toml` — `tauri-plugin-opener = "2"`.
7. `xgen-client/capabilities/default.json` — opener permission (§2.6, ground the identifier).
8. `xgen-client/src/desktop.rs` — `.plugin(tauri_plugin_opener::init())`.

**NOT this milestone / do NOT touch:** `get_about_info` / `about.rs` / `build_info` (C2 — closed; no new field, no company field — F2 is a shell literal) · `dialog.svelte` (C1 — closed) · any `ops.rs` · the node app · the sampler.

### The `ui/docs/*` question (N-note)

C3 builds **no new component** (`dialog` shipped at C1; About-dialog is a shell-local *consumer*). Per D-065, **do not invent an N-note** if there is no genuine component-level lesson. If C3 surfaces a real reusable UI lesson (e.g. the opener-plugin consumer-wiring pattern for `link`, or the always-mounted-dialog-body registry consequence in practice), record it as **N-090**; otherwise leave `ui/docs/*` untouched. Chat decides at close (Rule 6 — flag, don't invent).

---

## 4. Verify plan — REAL CLIENT 9222 (D-097; Rule 2, quote real output)

Run `run-client.ps1 -Debug`. The sampler cannot host this (no `get_about_info`, no frame).

1. **`cargo build` / `cargo test`** — workspace green. **Quote the real test count** (Rule 5 — do not carry a number forward; the opener plugin + no new tests should leave it at C2's 1507, but **measure**).
2. **`vite build`** — clean (**quote module count**; it will change from C2's 138 — new component + logo import + opener import).
3. **Help→About opens the dialog** — CDP: resolve `help.about` (or click the About item) → the dialog is a **real modal**: `el.matches(':modal') === true` (the C1 load-bearing leg — `showModal()` reflects the `open` attribute, so only `:modal` proves it). Quote it.
4. **Body renders the C2 data** — read each `about-*` label getter over CDP and check the value is the **real** `get_about_info` field (not `—`): `version` 0.10.3, `built · commit` pair, `rustc`/`tauri` 2.11.1/`svelte` 5.55.5/`platform`/the three paths. This is the UI-side field-by-field truth check.
5. **F1 — the website link opens the OS browser** — click `about-link`; confirm the **OS default browser** navigates to `alchemydump.com` (eye-check / process check), **not** an in-app blank webview. Quote the `errCount:0` / no `permissionDenials` (the opener capability is correctly granted). *This is the "see it function" Joe asked for.*
6. **F2 — "Developed by Alchemy Dump"** renders; the **personal name does NOT appear** anywhere in the box (grep the rendered DOM text — Rule 2).
7. **Close paths** — the Close button AND Esc both close (`open:false`, `:modal` gone), and **About re-opens** afterward (the C1 reconciliation proof — if the prop had lied, re-open would be a no-op).
8. **First real menu-bar roving Left↔Right** — two menus now exist (File, Help): ArrowRight/ArrowLeft move roving focus between the triggers; Home/End jump. Quote the `menu-bar` getter `activeIndex` moving 0↔1.
9. **Client registry grew — MEASURE it** (Rule 5, never predict). Enumerate what registered (`about` + `about__close` + `about-logo` + `about-link` + the `about-*` value labels) and quote `count===unique===domCount`, **0 orphans both directions**. Note: because the dialog body is always mounted (C1), the count is **stable across open/close** — measure once. Sampler catalogue **unchanged 313** (grounded by scope — no sampler file touched).

**PS 5.1 / harness (N-086, N-089):** wrap CDP eval returns as a **JSON object**; single-expression evals are the reliable form; a read after a thrown eval is **inconclusive, not a failure**. `__TAURI_INTERNALS__.invoke` is non-configurable — don't stub it. **Window-config unchanged** this milestone, so no `cdp.dev.conf.json` twin-edit needed — but if anything touches the window block, both files.

---

## 5. Rule-6 confirm points (ground it, don't guess)

- **Opener permission identifier** (§2.6 #3) — `opener:default` vs `opener:allow-open-url` vs a scoped entry. Confirm on contact; prefer the narrowest that works. If the plugin needs a URL scope in the capability, add exactly the `https://www.alchemydump.com` (or a minimal https scope) — do not open a wildcard.
- **Opener JS API** — `openUrl` from `@tauri-apps/plugin-opener`. Confirm the export name against the installed package (older docs say `open` from `plugin-shell`).
- **`package-lock.json` bump** — after `npm install`, re-verify `get_about_info().svelte` still returns **`5.55.5`** (C2's `build.rs` reads the lockfile; the opener add shouldn't move Svelte, but prove it — Rule 5).
- **Logo copy** — the `_hda` PNG is ~1000×1000; confirm it bundles + displays at the CSS-clamped size without layout blowout. If it's heavy, note it (no resize this milestone — CSS-scale the master, per the frame lock).
- If **any** grounded fact above is wrong on contact, **stop and report** (Rule 3).

---

## 6. Definition of Done

- [ ] `ui/client/src/about-dialog.svelte` — new; wraps core `dialog`; logo (`image`) + website (`link`) + "Developed by Alchemy Dump" literal + the §2.3 metadata grid; every value a `<Label>` with a stable `about-*` id; null-guarded.
- [ ] Help menu (`help.about`, **no accelerator**) + `commandTable['help.about']` flips `aboutOpen`; `<AboutDialog>` mounted in the frame; `get_about_info` fetched **on mount**.
- [ ] **Built + commit render together** in one row (`date · shortSHA`).
- [ ] F1 — website link opens the **OS browser** via `tauri-plugin-opener` (Cargo + package.json + capability + `desktop.rs` `.plugin()`); narrowest capability grant; `errCount:0`. **Demonstrated functioning** (Joe's ask).
- [ ] F2 — "Developed by Alchemy Dump" renders; **no personal name** anywhere in the box.
- [ ] Logo masters copied to `ui/assets/` (`logo_client_hda.png` wired, `logo_client_hd_small.png` reserved-unwired). Node masters untouched.
- [ ] Workspace `cargo test` green — **count quoted from real output**. `vite build` clean — **module count quoted**.
- [ ] All §4 verify legs run against the **real client 9222**, actual quoted output: `:modal` open, field-by-field body truth check, link→OS-browser, Close+Esc+re-open, File↔Help roving, registry measured (count===unique===domCount, 0 orphans both ways), sampler catalogue 313 unchanged.
- [ ] Client registry **measured**, not predicted; enumerated. C2's baked `name`/`link` literals confirmed rendering (KEEP, Joe).
- [ ] `ui/docs/*` untouched unless a genuine N-090 UI lesson surfaced (Rule 6 — don't invent).
- [ ] Any deviation **flagged, not absorbed** (Rule 6).

*(Per the task-file DoD rule: "commit pushed" is deliberately NOT a checklist item. `Status: COMPLETED` in the header is the real signal.)*

---

## 7. Close (D-074, two commits)

1. **Clair — feat commit** (code only): the files in §3 (UI + the F1 Rust/config footprint + the two copied logo assets).
2. **Chat — doc-bridge commit**: `JOURNAL.md` (J-series) · `CLAUDE.md` PLAY · `docs/ROADMAP.md` · `docs/xgen-client-frame-phase0.md` (§6 / §10.4 — C3 ✅, **M-RP6.1e-C CLOSED**) · `ui/docs/*` (N-090 **only if** earned) · this file → **COMPLETED**.

Joe pushes both. Chat never pushes.

---

*End of M-RP6.1e-C3 runbook.*
