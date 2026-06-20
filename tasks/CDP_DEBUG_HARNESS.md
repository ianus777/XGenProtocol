# XGen UI — CDP Debug Harness (WebView2 remote-debug read loop)
> **Status**: PENDING  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-20  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## Purpose

An automated way to inspect the running XGen UI — read its console output and its live state — without a manual copy-paste loop and without the Chrome extension (which cannot reach a Tauri WebView). The mechanism is the **Chrome DevTools Protocol (CDP)** exposed by the Windows WebView2 engine the Tauri shell embeds. This doc records the **resolved mechanism** (decisions locked 2026-06-20) and **specifies the harness** to build at RP-3 — when the UI and the `window.__XGEN_DEBUG__` registry exist. No harness code is written yet.

## Mechanism — RESOLVED (decisions locked 2026-06-20)

1. **Port enablement** — set `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=<port>` in the launching process's environment. Dev-only by convention; no code change required. Formalize a launch flag only if the harness later needs it.
2. **WebView content** — debug runs happen under `tauri dev` (Vite serving the Svelte app on `localhost:5173`), so the real UI and registry are loaded. A production-bundled build loads its own assets; either way the CDP target exists.
3. **Multi-instance** — one port per instance: `port = 9222 + instance_ordinal`. The harness discovers the correct page target via the HTTP `/json` list (match on `url` / title), then attaches to that target's `webSocketDebuggerUrl`.
4. **What is read** — two capabilities: (a) **console-tail** via `Runtime.consoleAPICalled` — usable now, pre-UI; (b) **state dump** via `Runtime.evaluate` on `window.__XGEN_DEBUG__` — once the registry exists. Console-tail ships first.
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

## Harness specification (build at RP-3)

A single dev-only script. Responsibilities, in order:

1. **Resolve target** — input: instance label/ordinal (default 0). Compute `port = 9222 + ordinal`. `GET /json`; pick the `page` target (match expected `url`/title); read its `webSocketDebuggerUrl`. If the app is not yet running, optionally launch it with the env-var set; otherwise attach to the running one.
2. **Connect** — open the ws; assert `State = Open`.
3. **Enable domains** — send `Runtime.enable` (required for console events).
4. **Capability: console-tail** — receive-loop on `Runtime.consoleAPICalled`; render each to a line (type · args). Bounded by a frame budget / timeout; write to a capture file the operator (or Claude) reads.
5. **Capability: state-dump** — send `Runtime.evaluate` with `returnByValue:true` on `JSON.stringify(window.__XGEN_DEBUG__)` (or a single component by `data-debug-id`); capture the value to file.
6. **Cleanup** — dispose the ws; if the harness launched the process, kill the tree (`taskkill /PID <pid> /T /F`) and clear the env-var.

**Conventions to honour:** `$ProgressPreference='SilentlyContinue'`; absolute paths; verification reads as separate calls from writes; UTF-8 no-BOM if it emits any project file.

## Open / parked

- Exact target-match heuristic for multi-instance (url vs. window title) — settle when more than one instance is debugged in practice.
- Whether `Console.enable` (legacy) adds anything over `Runtime.consoleAPICalled` — default to Runtime only.
- Deep-object rendering policy for the state dump (shallow + expand) — a presentation choice, deferred.
- In-grain alternative: a `debug-snapshot` verb on the existing `--aicontrol` / D-056 command layer (Appendix O) could carry app state without CDP. CDP stays the tool for console/DOM; the aicontrol path is the long-term option for app *state* specifically. Recorded, not chosen.

## Definition of Done (for the RP-3 build)

- [ ] Harness resolves a target by instance ordinal and attaches via `/json` (verified against a running `tauri dev`).
- [ ] `Runtime.enable` succeeds; console-tail captures at least one real app `console.log` line with structured args.
- [ ] State-dump returns `window.__XGEN_DEBUG__` content by value to a capture file (once the registry exists).
- [ ] Cleanup leaves no orphan process and no lingering env-var.
- [ ] Confirmed inert against a release build (no port, no devtools) — documented, not just assumed.
- [ ] Dev-only gating documented where the harness lives.
