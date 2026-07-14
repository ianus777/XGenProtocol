# XGen UI — CDP Debug Harness (WebView2 remote-debug read loop)
> **Status**: ACTIVE  
> Version: 1.5  
> Date: Jun 2026  
> **Last updated**: 2026-07-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## Purpose

An automated way to inspect the running XGen UI — read its console output and its live state — without a manual copy-paste loop and without the Chrome extension (which cannot reach a Tauri WebView). The mechanism is the **Chrome DevTools Protocol (CDP)** exposed by the Windows WebView2 engine the Tauri shell embeds. This doc records the **resolved mechanism** (decisions locked 2026-06-20) and the **harness**, now **built and verified** as `cdp-debug.ps1` at repo root (2026-06-20). Remaining verification (real app logs, real registry content) is gated on the UI existing under `tauri dev`.

## Mechanism — RESOLVED (decisions locked 2026-06-20)

> **⚠️ UPDATE (2026-07-09, J-483 / M-RP-CDP1) — port enablement changed for WebView2 ≥136.** Item 1 below (the `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` env var) is **SUPERSEDED** for the dev-session path. WebView2 Evergreen ≥136 (runtime 150.0.4078.48) stopped opening the port via the env var — **wry overrides `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`** with its own programmatic `AdditionalBrowserArguments` (confirmed: the port was absent from every `msedgewebview2.exe` child cmdline; it is NOT the Chromium-136 `--user-data-dir` guard, which Tauri already satisfies by forcing a non-default data dir). **New mechanism (D-105):** the port rides Tauri config `additionalBrowserArgs`, delivered as a **dev-only overlay** `cdp.dev.conf.json` (per app: `xgen-sampler/`, `xgen-client/`, `xgen-node/`) merged via `cargo tauri dev --config cdp.dev.conf.json`. `run-{sampler,client,node}.ps1 -Debug` now do exactly this; the base `tauri.conf.json` stays **port-free** so RELEASE never exposes CDP (item 6 still holds). Verified on all three (sampler 9422 / client 9222 / node 9322, harness attach + `__XGEN_DEBUG__` live). **`cdp-debug.ps1 -Launch` (bare built-exe) is DEAD under ≥136** — a built exe takes no `--config` and its env-var route is clobbered; the supported path is attach-to-a-dev-session (`run-*.ps1 -Debug`, then `cdp-debug.ps1 -App <app> -Mode …` WITHOUT `-Launch`). A debug-build-with-port-baked, or dropping `-Launch`, is a future decision.

1. **Port enablement** — set `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=<port>` in the launching process's environment. Dev-only by convention; no code change required. Formalize a launch flag only if the harness later needs it.
2. **WebView content** — debug runs happen under `tauri dev` (Vite serving the Svelte app on `localhost:5173`), so the real UI and registry are loaded. A production-bundled build loads its own assets; either way the CDP target exists.
   - **Two launch paths (keep distinct):** `cdp-debug.ps1 -Launch` spawns the *bare* `bin\xgen-*.exe` (dev-wired to `:5173`/`:5174` but no Vite) — this proves **transport** only (endpoint/attach/evaluate/console), page is blank. The **real-registry** path is `run-client.ps1 -Debug` / `run-node.ps1 -Debug` (which set the port env-var, start Vite, and run `tauri dev`) **then** `cdp-debug.ps1 -App <app> -Mode state` in *attach* mode (no `-Launch`). The latter is what verifies actual `window.__XGEN_DEBUG__` content.
3. **Multi-app / multi-instance** — `port = base + instance_ordinal`, with base 9222 (client) / 9322 (node) selected by `-App`, so client and node never collide and each instance gets a unique port (`-Port` / `-Exe` override). The harness discovers the page target via the HTTP `/json` list, then attaches to its `webSocketDebuggerUrl`.
4. **What is read** — two capabilities: (a) **console-tail** via `Runtime.consoleAPICalled` — usable now, pre-UI; (b) **state dump** via `Runtime.evaluate` on `window.__XGEN_DEBUG__.snapshot()` (registry verbs `snapshot()` / `get(id)` / `ids()`, per the **N-024** producer contract) — once the registry exists. Console-tail ships first.
5. **Carrier** — PowerShell v1 (`System.Net.WebSockets.ClientWebSocket`). Proven end-to-end, in-session, zero dependencies. Revisit only if it becomes unwieldy (small Rust/Node helper is the fallback carrier).
6. **Release safety (non-negotiable)** — the WebView2 `devtools` feature and the remote-debug port are **dev-only**. Production builds close both. The harness never targets a release build.

## Proven evidence (this session, 2026-06-20)

Against `bin\xgen-client.exe` (bare launch opens a Tauri window titled "XGen Client"; `devtools` compiled in):

- **Endpoint opens** — `GET http://localhost:9222/json` returned a live `page` target with a `webSocketDebuggerUrl` (`ws://localhost:9222/devtools/page/<id>`). Page `url` was `http://localhost:5173/` (dev-wired build; Vite not running, so the page was blank).
- **Evaluate works** — over the ws, `Runtime.evaluate` returned by value:
  `{"id":1,"result":{"result":{"type":"string","value":"{\"title\":\"\",\"url\":\"about:blank\",...}"}}}`
- **Attach to a running instance** — the page target was discovered through `/json` and attached without relaunching; the harness can attach to an externally-launched window.
- **Console capture** — `Runtime.enable` then an injected `console.log('XGEN_PROBE_MARKER', 42, {a:1})` arrived as a `Runtime.consoleAPICalled` event with **structured args** (string, number, and object with a property preview `{a:1}`) — typed data, not flattened text.

**Honest scope of the proof:** the page was blank (no Vite), so these results prove the **transport** (endpoint, attach, evaluate, console-event delivery) — not real app logs or real registry content. Those require the UI served under `tauri dev`.

## Real-registry proof (2026-06-21, M-RP2.3)

The UI-gated remainder is now closed for the **state-dump** path, in **both** apps, via the real-registry launch path (`run-*  -Debug` + attach). With the first instrumented `core` component (`toggle`, N-024 debug getter) mounted as a demo instance in each shell:

- Client (port 9222, page `http://localhost:5173/`) and node (port 9322, page `http://localhost:5174/`) each attached over the WS.
- `Runtime.evaluate` on `window.__XGEN_DEBUG__.snapshot()` returned real content: `{"toggle#demo":{"type":"toggle","state":{"checked":false}}}`.
- After flipping the toggle (DOM `change`), a re-dump returned `{checked:true}` — proving the registry reads **live reactive scope**, not a mount-time snapshot.
- DOM carried the envelope stamp `class="toggle"` + `data-debug-id="toggle#demo"`; the `get('<id>')` path keys on that same string.

**Script fix applied this session:** the `state` mode evaluated the pre-v1.1 bare `JSON.stringify(window.__XGEN_DEBUG__)` (which stringifies the singleton's *methods* to `{}` once the registry exists). Corrected to the guarded `…snapshot()` form this doc already specified at v1.1 — only the script had drifted.

**Still transport-only:** real *app* `console.log` capture (vs the injected-marker probe) was not exercised this session; the console-tail path remains proven by transport, not by a real app log line.

## Harness specification

**Built + verified 2026-06-20** — `cdp-debug.ps1` at repo root (sibling of `run-client.ps1`). Modes: `eval` (arbitrary expression), `state` (dumps `window.__XGEN_DEBUG__`), `console` (tails `Runtime.consoleAPICalled` for N seconds). `-App client|node` selects the exe + base port (client 9222 / node 9322); `-Launch` spawns the exe with the port env-var and kills its tree on exit; without it, the harness attaches to a running instance. Verified green for **both apps**: client (port 9222, dev :5173) and node (port 9322, dev :5174) — eval returned `2`; state reports `null` gracefully (no registry yet); console tailed and exited cleanly. End-to-end ~1 s.

**Resolve fix (root cause of an early hang):** an `Invoke-WebRequest` to a not-yet-listening port waits the full ~2 s timeout under Windows PowerShell, so HTTP-only polling overran on cold start. The harness now does a fast **TCP-connect probe** (a closed port fails in ~ms) before fetching `/json`.

A single dev-only script. Responsibilities, in order:

1. **Resolve target** — input: `-App` (client/node) + ordinal. Compute `port = base + ordinal` (base 9222 client / 9322 node). **TCP-probe** the port (fast fail if closed), then `GET /json`; pick the `page` target; read its `webSocketDebuggerUrl`. If the app is not yet running, optionally launch it with the env-var set; otherwise attach to the running one.
2. **Connect** — open the ws; assert `State = Open`.
3. **Enable domains** — send `Runtime.enable` (required for console events).
4. **Capability: console-tail** — receive-loop on `Runtime.consoleAPICalled`; render each to a line (type · args). Bounded by a frame budget / timeout; write to a capture file the operator (or Claude) reads.
5. **Capability: state-dump** — send `Runtime.evaluate` with `returnByValue:true` on a **guarded** expression so the read stays graceful before the registry installs: `window.__XGEN_DEBUG__ ? JSON.stringify(window.__XGEN_DEBUG__.snapshot()) : null` (whole dump), `….get('<data-debug-id>')` (single component), or `….ids()` (enumeration). Capture the value to file. Registry shape + verbs are defined by **N-024** (`ui/docs/xgen-ui-notes.md`); the singleton isolates a throwing component so one bad getter cannot blind the whole dump.
6. **Cleanup** — dispose the ws; if the harness launched the process, kill the tree (`taskkill /PID <pid> /T /F`) and clear the env-var.

**Conventions to honour:** `$ProgressPreference='SilentlyContinue'`; absolute paths; verification reads as separate calls from writes; UTF-8 no-BOM if it emits any project file.

## Trusted input — `-Mode click` / `-Mode drag` (added 2026-07-14, M-RP7.2 leg 0)

**Why it exists.** A synthetic `MouseEvent` from `Runtime.evaluate` is **untrusted** (`isTrusted:false`) and **fires no native defaults** — J-496 proved this the hard way. **`Input.dispatchMouseEvent` is injected at the browser level, so it is trusted**, and it drives real hover, focus, pointer capture and drag. **M-RP7.4's drag cannot be proven without it.**

```
.\cdp-debug.ps1 -App client -Mode click -At "206,42"
.\cdp-debug.ps1 -App client -Mode drag -From "215,400" -To "300,400" -Steps 12 `
     -MidExpression "JSON.stringify(__XGEN_LAYOUT__.current)" `
     -Expression    "JSON.stringify(__XGEN_LAYOUT__.current)"
```

> ### 🔑 **`-MidExpression` IS EVALUATED WHILE THE BUTTON IS STILL DOWN, AND IT IS NOT A CONVENIENCE.**
> A design that **previews live but only writes the descriptor on release** is **indistinguishable** from one that writes on every move — *if you can only read after `mouseReleased`.* **The mid-drag read IS the proof.** Verified: MID sees the moves and **no `mouseup`**; AFTER sees the `mouseup`.

**MEASURED on the real client (2026-07-14), not assumed:**

- **Coordinates are CSS pixels** relative to the layout viewport — **the same space `getBoundingClientRect()` returns.** **DPR 1.25 does NOT apply; do not scale.** *Calibrated by clicking a fold button at its measured rect centre and watching `collapsed` flip — a wrong coordinate space simply misses.*
- `isTrusted = true` on every event; `buttons = 1` across the moves; **three consecutive drags byte-identical.**
- **⚠️ `button` MUST be `"none"` on a button-up move.** Sending `button:"left"` with `buttons:0` makes Chromium report **`buttons=1` on a HOVER**. Harmless to a splitter; **it would silently poison M-RP7.4**, whose drop-band hover must be readable with the button up. *The instrument lied before the code had a chance to.*
- **⚠️ INTEGER COORDINATES ONLY.** PowerShell renders a `[double]` with the **current culture's** decimal separator — on a sk-SK box `123.5` becomes `123,5`, which is not JSON, and the CDP frame is rejected with an error that looks nothing like a locale bug.
- **⚠️ Every read sits behind a double-`requestAnimationFrame` barrier** (`awaitPromise`). A CDP ack means the *browser* accepted the event, not that the renderer ran the handler — and Svelte's flush is a microtask on top (N-117). **The barrier is a READ barrier: do not put it inside the move loop.**
- **⚠️ THE SELECTION TRAP — see N-118.** Every gesture is preceded by `getSelection().removeAllRanges()`. Without it, the second drag from the same point presses on the selection the first one left, **Chromium opens a native HTML5 drag, and every subsequent `mousemove` and the `mouseup` are swallowed in silence.** *This is not a harness quirk — it is a real bug waiting for any splitter that is not `user-select: none`.*
- **🧪 `-KeepSelection` (J-519) — because A HARNESS THAT CAN ONLY PASS IS A WEAK HARNESS.** It **suppresses** that clear, so N-118 can be **REPRODUCED on demand** rather than merely avoided. **Use it to prove a gesture is genuinely immune rather than protected by the instrument.** **Proven with it:** two drags on selectable tile body → `mousedown, mousemove, **dragstart**` — and then **nothing, no `mouseup`** (the native drag, finally *visible* rather than inferred). The **seam**, same conditions with a live selection → **both drags commit, no `dragstart`.** ***If a gesture behaves differently under this switch, the guard is the HARNESS's, not the CODE's — and the code will fail for a real user, who has no harness tidying up after them.***
- **⚠️ RE-MEASURE COORDINATES BEFORE EVERY GESTURE — a rect is not a constant.** Folding **moves** a tile's buttons (they rotate into the side strip); a resize **moves** every seam to its right. **Chat dragged a stale coordinate during the J-519 re-drive and selected text instead of grabbing the seam.** *Measure, then gesture. Never gesture on a coordinate measured before the last gesture.*

## Open / parked

- Exact target-match heuristic for multi-instance (url vs. window title) — settle when more than one instance is debugged in practice.
- Whether `Console.enable` (legacy) adds anything over `Runtime.consoleAPICalled` — default to Runtime only.
- Deep-object rendering policy for the state dump (shallow + expand) — a presentation choice, deferred.
- In-grain alternative: a `debug-snapshot` verb on the existing `--aicontrol` / D-056 command layer (Appendix O) could carry app state without CDP. CDP stays the tool for console/DOM; the aicontrol path is the long-term option for app *state* specifically. Recorded, not chosen.

## Definition of Done

- [x] Harness resolves a target by `-App` + ordinal and attaches via `/json` — verified 2026-06-20 for **both** client (9222) and node (9322) bin builds (eval `2`; resolve via TCP probe). Against `tauri dev` specifically: pending a UI run.
- [x] `Runtime.enable` + console-tail loop runs and exits cleanly — verified. Capturing a **real app** `console.log` line is deferred until the UI emits logs (the transport is already proven via the injected-marker probe).
- [x] State-dump path verified — guarded `….snapshot()` read returns `null` gracefully pre-registry, and returns **real content in both apps** with the registry present (M-RP2.3, 2026-06-21): `{"toggle#demo":{"type":"toggle","state":{"checked":false}}}`, flip → `{checked:true}`. Script brought in line with the v1.1 `…snapshot()` spec.
- [x] Cleanup leaves no orphan process and no lingering env-var — verified (`taskkill /T` + env-var removal in `finally`).
- [ ] Confirmed inert against a release build (no port, no devtools) — pending a release build.
- [x] Dev-only gating documented where the harness lives — header comment in `cdp-debug.ps1`.

**UI-gated remainder:** the **state-dump** path is now verified against the real registry in both apps (M-RP2.3). Real *app* `console.log` capture (beyond the injected-marker probe) remains outstanding — exercised when a shell emits real log lines.
