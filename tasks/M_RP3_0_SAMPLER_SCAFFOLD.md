# M-RP3.0 — Sampler scaffold (standalone arc M-RP3: the component test-bed)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-27  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Goal

Stand up the **Sampler** — a third, standalone Tauri/WebView2 app whose sole job is to host, tune, and CDP-verify the `core` component library in isolation, with a live client↔node skin-swap. This is the **scaffold** milestone of the M-RP3 arc: prove the pipeline end-to-end (Tauri+Vite+WebView2 boots → `$core` import resolves → `envelope` registers in `window.__XGEN_DEBUG__` → CDP attaches on a dedicated port → skin-swap re-themes), mounting **one** smoke component to prove the chain. Populating the full class×phase matrix is the next milestone (M-RP3.1).

Direction (Joe-locked): the **component track is paused** (the di queue — `date`/`color`/`file`/`select multiple` — is resumable, not cancelled); both **real shells are frozen as-is** (no revert, no new wiring); the sampler is its **own arc**, populated with the 10 already-built components.

## Locks (decisions to record this milestone)

**D-097 — test-bed split (the build-vs-interaction rule).** Component appearance / state / per-shell theming is built and tuned in the **sampler** (the skin-swap covers gold-vs-blue, fully replacing the practice of wiring demos into both real shells). A component inside a real composed feature is exercised in the **real app at integration**. The two shells **running with each other** (federation/protocol plane, handshakes, the MP-R scenarios) is the sampler's structural blind spot — one window, one runtime — and stays the job of **both real apps run together**, at interaction/integration milestones (MP-R3, Tier-1 auth rebuild, streams). From `date` onward, components are built/tuned in the sampler, CDP-verified there; the real apps are not run for *component* reasons.

**D-098 — sampler runtime = full Tauri/WebView2 sibling (option A), not Vite-only (B).** The sampler runs in **WebView2** via its own minimal Tauri host, identical runtime to the real shells (same Blink/Chromium, same quirks the skin rests on — vendor-prefixed pseudo-elements, `color-scheme`, the hover-only spinner paint), driven by the **same CDP self-drive harness**. The host crate is deliberately **minimal** — `tauri` + `tauri-build` only, **no protocol deps** (it does NOT mirror `xgen-client`, which pulls `xgen-common`/`xgen-core`/tokio/websockets/crypto/CLI). Rejected (B) Vite-in-Chrome: lighter but a different engine than the real runtime and a divergent toolchain — reintroduces the false-confidence the sampler exists to remove. The sampler is **D-095-mirror-exempt** (it does not mirror the client/node source-tree shape; already footnoted under D-095).

**Scaffold-scope lock:** v0 mounts exactly **one** smoke component (`sampler#smoke`) to prove the `$core`+envelope+registry chain. The matrix IA, state columns, and the polished skin-swap control are **M-RP3.1**. v0's skin-swap is the *mechanism* (the `[data-shell]` attribute + both accent-alias blocks + a bare flip control) — enough to prove re-theming, not the final UI.

## Architecture (from Phase-0)

Each real app = a Vite+Svelte frontend (`ui/<x>`, e.g. port 5173) **+** a Tauri Rust crate (`xgen-<x>/`) that opens a WebView2 window at the Vite `devUrl`; `run-<x>.ps1` starts Vite then `cargo tauri dev`; `-Debug` sets `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=<p>`. The sampler mirrors this shape with its **own** frontend + a **minimal** crate.

Ports (new pair): **Vite 5175 · CDP 9422** (client 5173/9222, node 5174/9322).

Per-shell accent (the skin-swap hinge) is just three vars in each shell's `app.css` over the shared `skin.css`:
`--accent / --accent2 / --accent-ink` → `--pr* ` (client gold) / `--inf*` (node blue). The sampler defines **both** blocks keyed by `:root[data-shell="client"|"node"]` and flips the attribute at runtime.

## Files to create / edit

**Frontend `ui/sampler/`** (mirror `ui/client`, D-095-exempt layout):
1. `index.html` — title "XGen Sampler", `<div id="sampler-root">`, `<script type="module" src="/src/main.js">`.
2. `package.json` — name `xgen-sampler-ui`; deps mirror `ui/client` (svelte, vite, @sveltejs/vite-plugin-svelte, @tauri-apps/cli); `dev`/`build` scripts.
3. `vite.config.js` — `server.port = 5175`, `strictPort: true`; aliases `$core`/`$common`/`$assets` resolved identically to `ui/client/vite.config.js`; svelte plugin.
4. `src/main.js` — mirror `ui/client/src/main.js` import order (xgen-normalize → skin.css → app.css) + mount `app_sampler.svelte` into `#sampler-root`.
5. `src/app.css` — the **two accent-alias blocks** keyed by `:root[data-shell="client"|"node"]` (lifted from client/node `app.css`); a minimal sampler layout (not the 320px card — its own dev layout). No new tokens.
6. `src/app_sampler.svelte` — v0: a title, a **skin-swap flip control** (bare button toggling `document.documentElement.dataset.shell`), and **one** smoke `core` instance (e.g. `<Button>` or `<Toggle>`) via `use:envelope` → registry id `sampler#smoke`. Default `data-shell="client"`.

**Crate `xgen-sampler/`** (minimal Tauri host):
7. `Cargo.toml` — `name = "xgen-sampler"`, `[[bin]]`, `build = "build.rs"`; `[build-dependencies] tauri-build = { version = "2" }`; `[dependencies] tauri = { version = "2" }` (+ `tauri-plugin-process` only if a quit command is wanted — default: omit, the decorated window's OS close suffices). **No** xgen-common/core/tokio/etc.
8. `build.rs` — `fn main() { tauri_build::build() }`.
9. `tauri.conf.json` — `productName "XGen Sampler"`, `identifier "com.alchemydump.xgensampler"`, `build.devUrl "http://localhost:5175"`, `frontendDist "C:/cargo-targets/XGenProtocol/sampler-dist"`, `beforeBuildCommand "npm --prefix ../ui/sampler run build"`; window `decorations: true`, `resizable: true`, ~`960×820`, `center: true` (a dev tool — needs to scroll the matrix); `security.csp null`; `bundle.active false`.
10. `src/main.rs` — minimal: `tauri::Builder::default().run(tauri::generate_context!()).expect(...)` (~6 lines + license header).
11. `capabilities/default.json` — mirror `xgen-client/capabilities` default (core window perms; + `process:allow-exit` only if a quit command is added).
12. `icons/icon.png` — reuse a placeholder (copy `xgen-client/icons/icon.png`) so `tauri-build` resolves.

**Root plumbing:**
13. `run-sampler.ps1` — mirror `run-client.ps1`: `$FrontendDir = ui/sampler`, `$TauriDir = xgen-sampler`, Vite port **5175**, `-Debug` → `--remote-debugging-port=9422`.
14. Root `Cargo.toml` — add `"xgen-sampler"` to `[workspace] members`.
15. `cdp-debug.ps1` — add the `sampler` app → port **9422** mapping (so `-App sampler` works).

## Phases

**Phase 1 — frontend** (`ui/sampler/*`): create files 1–6. Confirm aliases resolve against the existing `$core`/`$common`/`$assets`. `app_sampler.svelte` imports the smoke component from `$core/components/data-independent/…` and `envelope` from `$common/components/base/envelope` (the proven paths).

**Phase 2 — crate** (`xgen-sampler/*`): create files 7–12 (copy-adapt from `xgen-client`, stripping all protocol/CLI/lifecycle — keep only the bare Tauri builder). Add to workspace members (file 14).

**Phase 3 — run script + harness** (files 13, 15): `run-sampler.ps1`; teach `cdp-debug.ps1` the `sampler`→9422 mapping.

**Phase 4 — CDP verify (the sampler itself; Chat self-drives):**
- `run-sampler.ps1 -Debug` (detached) → Vite **5175** up, WebView2 window opens, CDP attaches on **9422**.
- Registry: `window.__XGEN_DEBUG__.ids()` includes **`sampler#smoke`** — proves `$core` import + `envelope` + the debug registry work end-to-end in the new app (the scaffold's load-bearing proof).
- Skin-swap: flip `document.documentElement.dataset.shell` client↔node → computed-style `--accent` resolves **gold `rgb(154,106,48)`** (`--pr`) ↔ **blue `rgb(42,96,144)`** (`--inf`) on a skinned element. Real output captured.
- Screenshot: the smoke component renders with the client accent; second screenshot after swap shows the node accent.
- Clean teardown (ports 5175/9422 free, 0 orphans).

**Phase 5 — records (D-074 atomic):**
- `DECISIONS.md` **D-097** (test-bed split) + **D-098** (sampler runtime A); `Last updated` bumped.
- `ui/docs/xgen-ui-notes.md` **N-044** — sampler scaffold: the minimal-host decision, ports 5175/9422, the `[data-shell]` skin-swap mechanism, the `sampler#smoke` registry proof.
- `docs/ROADMAP.md` — new **M-RP3 arc** node + **M-RP3.0 ✅**; the component-track **paused** marker; frontier → M-RP3.0. Version bump.
- `CLAUDE.md` PLAY → M-RP3.0 (new arc; component track paused; sampler is the new test bed); pointer → J-422.
- `ui/docs/xgen-ui-components.md` — **no Built-registry change** (no new component); one line noting the sampler is the test bed from `date` onward.
- `tasks/M_RP3_0_SAMPLER_SCAFFOLD.md` → **COMPLETED**.
- `JOURNAL.md` **J-422** (written last, real CDP output quoted).
- Two commits: implementation (frontend + crate + plumbing), then records-only. Joe pushes.

## Definition of Done

- [x] `ui/sampler/` created (index.html, package.json, vite.config.js port 5175, src/main.js, src/app.css with both `[data-shell]` accent blocks, src/app_sampler.svelte with skin-swap flip + one smoke component)
- [x] `xgen-sampler/` created (minimal Cargo.toml — tauri + tauri-build, no protocol deps; build.rs; tauri.conf.json devUrl 5175 / decorated resizable 960×820 window; src/main.rs ~6-line builder; capabilities/default.json core-only; icons copied from client)
- [x] root `Cargo.toml` workspace members += `xgen-sampler`; workspace resolved (Cargo.lock updated on build)
- [x] `run-sampler.ps1` (Vite 5175, cargo tauri dev in xgen-sampler, -Debug → CDP 9422); `cdp-debug.ps1` knows `sampler`→9422
- [x] CDP: `run-sampler.ps1 -Debug` booted WebView2, CDP attached on 9422 (J-422)
- [x] CDP: `__XGEN_DEBUG__.ids()` = `["button#smoke"]` — $core+envelope+registry chain proven in the new app (`location.href=http://localhost:5175/`, `typeof __XGEN_DEBUG__==="object"`)
- [x] CDP: skin-swap flips `--accent` `#9a6a30`(gold/`--pr`,client) ↔ `#2a6090`(blue/`--inf`,node) via `[data-shell]`
- [x] screenshots: smoke component + bar render; client accent then node accent after swap (smoke button base is `--s4`, so the two are pixel-identical — var-resolution is the proof; matrix M-RP3.1 adds accent-prominent components)
- [x] clean teardown (ports 5175/9422 free, 0 orphans)
- [x] records (DECISIONS D-097 + D-098, N-044, ROADMAP v4.01 M-RP3 arc + M-RP3.0 + paused marker, CLAUDE PLAY, components test-bed note, JOURNAL J-422, task COMPLETED)

**Naming correction (from verify):** the smoke instance registers as **`button#smoke`**, not `sampler#smoke` — `envelope` keys by component *type* (`name: 'button'`), not by app. The DoD/verify above use the actual id.
