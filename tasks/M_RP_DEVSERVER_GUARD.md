# M-RP-DEVSERVER-GUARD — the dev launcher owns the Vite it starts
> **Status**: COMPLETED  
> Version: 1.0  
> Date: July 2026  
> **Last updated**: 2026-07-26  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — WHY THIS RAN BEFORE LEG B, NOT AFTER

M-RP-MEMBERS Leg B is frontend work. Under the defect below it would have been verified
against a bundle hours older than the code under test, with **every probe reporting success**.
Fixing after Leg B makes Leg B's evidence retroactively suspect; fixing during means changing
the instrument mid-measurement. **Before was the only clean slot.**

Scope: **4 `.ps1` files at repo root.** Zero Rust · zero `ui/**` · zero `.md` under `docs/`
other than records. **No floor moves** — nothing compiled, nothing bundled.

---

## §2 — THE DEFECT, MEASURED

`run-client.ps1` / `run-node.ps1` / `run-sampler.ps1` were byte-identical in shape:

```
$vite = Start-Process cmd.exe /c "npm --prefix <dir> run dev" -PassThru   # the handle
...
Invoke-WebRequest http://localhost:<port>                                 # the probe
...
$vite | Stop-Process -Force                                               # the cleanup
```

**① THE PROCESS TREE IS FOUR LEVELS DEEP.** Measured live, not inferred:

```
cmd.exe  (ours, the $vite handle)   /c npm --prefix ui\client run dev
 └─ node                            npm-cli.js
     └─ cmd.exe                     /d /s /c vite        <- a SECOND cmd.exe
         └─ node                    vite.js              <- THE LISTENER
```

`Stop-Process $vite` killed **the first of six processes**. npm survived and **respawned vite**.
⚠️ The inherited claim was *"kill the npm parent"* — measurement says the script's own handle
sits **one level above npm again**, and the reaped tree is six processes, not two.

**② THE PROBE ASKED THE WRONG QUESTION.** `Invoke-WebRequest localhost:<port>` cannot
distinguish *"my server started"* from *"a server is listening."* All three Vite configs are
`strictPort: true` (5173 client · 5174 node · 5175 sampler — read from the three
`vite.config.js`), so the next run's own Vite **dies on the taken port** while the probe is
**answered by the leak** and reports ready. 🔑 **The app then runs a stale bundle with no error
anywhere.** This is the D-071 defect class: *a check that passes because something else supplied
what was missing.*

**③ CLEANUP WAS NOT IN A `finally`.** Ctrl-C or a closed console skipped it entirely.

**④ NOT IN THE INHERITED BRIEF: `$env:TAURI_SKIP_DEVSERVER_CHECK = "true"`** disables Tauri's
own devserver check — the second alarm that would have caught this was switched off by hand.
📌 **FLAGGED, NOT CHANGED. It may be there for a reason. 🔓 Joe's.**

---

## §3 — THE FIX: THREE GUARANTEES THAT COVER EACH OTHER

| # | Guarantee | Mechanism |
|---|---|---|
| 1 | **PRE-FLIGHT REFUSAL** | Port already listening ⇒ abort, **naming the holder's PID and command line**, and print the `taskkill` line to clear it |
| 2 | **OWNERSHIP ASSERTION** | The listener must be a **descendant of the process we spawned** (`Test-IsDescendantOf`, PPID walk with a cycle guard). Presence of a listener proves nothing |
| 3 | **TREE KILL IN `finally`** | `taskkill /PID $vite.Id /T /F`. `exit` inside `try` still runs `finally`, so the abort paths clean up too |

🔑 **RESIDUAL GAP, NAMED:** closing the console window mid-run skips `finally`. **Guarantee 1
then makes the next run refuse loudly** instead of silently serving a stale bundle. *The two
halves cover each other; neither alone is sufficient.* Guarantee 1 alone leaves the
check-then-start race; Guarantee 2 alone wastes the 15 s wait.

**Why all three scripts and not just the client** (Joe's question, answered): two of the three
sanctioned launchers start a Vite — `run-client_debug.lnk` → 5173 and `run-node_debug.lnk` →
5174. (`run-node_service.lnk` routes to `cargo run` at `run-node.ps1:37` and **starts no Vite at
all** — a hypothesis of Chat's that measurement killed.) 5174 has **already cost one session** to
the wrong diagnosis *"an orphaned client vite"*. And a rule that holds for two of three scripts
is not a rule.

---

## §4 — THE `cdp-debug.ps1 -Launch` RIDER: RETIRED, NOT REPAIRED

`-Launch` could not work and **failed convincingly**: it set
`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` (`:188`), which WebView2 ≥136 ignores per **D-104** —
contradicting the script's own header 150 lines above — and it ran `$Exe`, defaulting to
`bin\xgen-client.exe`, **the 21-May release binary**. Output was two reassuring lines plus
`exit 1`, and it put a plausible XGen window on screen.

Under D-104 the only CDP route is the `cdp.dev.conf.json` overlay, which `run-*.ps1 -Debug`
already takes ⇒ **there is nothing to repair.** `-Launch` now **refuses and names the correct
route.** The usage header was corrected in the same pass — a doc that contradicts its own code
is the same defect class as the probe.

---

## §5 — VERIFICATION

All functions were **extracted from the file on disk and executed**, never retyped.

| Check | Expect | Result |
|---|---|---|
| Parse, all 4 scripts | 0 errors | ✅ `PSParser` clean |
| Line endings preserved | LF (all 4 were LF) | ✅ CR=0 on all 4 |
| Live chain: listener → our handle | 4 hops, chain intact | ✅ traced |
| G1 on a genuinely held port | refuse, name PID + cmdline | ✅ named PID + `vite.js` path |
| G2 listener vs real ancestor | True | ✅ |
| G2 vs unrelated live process | False | ✅ |
| G2 vs non-existent PID | False | ✅ |
| G2 reversed direction | False | ✅ |
| `Get-PortOwnerPid` on free port | null | ✅ |
| G3 `taskkill /T` from our handle | tree reaped, port released | ✅ 6 processes, released |

⚠️ **ONE CONTROL FAILED AND THE CONTROL WAS WRONG, NOT THE CODE.** The first negative control
asked whether the listener was a descendant of `$PID` — but `$PID` was the shell that **spawned
the whole chain**, so `True` was the correct answer to a question that tested nothing. Re-run
against a genuinely unrelated process (explorer), a non-existent PID, and the reversed
direction. 🔑 **A negative control that shares an ancestor with the positive control is not a
negative control** — N-163's requirement is not satisfied by merely *having* one.

---

## §6 — DEFINITION OF DONE

- [x] All three `run-*.ps1` carry the three guarantees, with the measured tree in a comment
- [x] `cdp-debug.ps1 -Launch` refuses and names the overlay route; usage header corrected
- [x] All four scripts parse clean; LF preserved (CR=0)
- [x] G1/G2/G3 driven live with positive **and** valid negative controls
- [x] `--service` confirmed Vite-free and untouched
- [x] JOURNAL + CLAUDE.md PLAY + ROADMAP + this doc in one commit (D-074)
- [x] N-166 recorded

---

## §7 — CARRIED FORWARD

- 🔓 **`TAURI_SKIP_DEVSERVER_CHECK`** — flagged in §2④, unchanged. Joe's.
- 📌 **Floors still stale for Leg B:** `svelte-check` (last known 0 err / 34 warn / 15 files) and
  the client registry (last known 149). Neither was re-measured here — this milestone touched no
  `ui/**` and no Rust, so it cannot have moved them, but **the staleness predates this work** and
  must be discharged before Leg B.
- 📌 **`run-sampler.ps1`'s `try` body keeps the original indentation** to hold the diff to the
  logic. Cosmetic only; parses clean.
