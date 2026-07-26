# XGen Node - dev and release launcher.
# Usage:
#   .\run-node.ps1            - dev mode (hot-reload, no install needed)
#   .\run-node.ps1 -Debug     - dev mode + WebView2 remote-debug port 9322 (CDP harness, dev-only)
#   .\run-node.ps1 release    - build standalone .exe
#   .\run-node.ps1 --service  - headless service mode (no window, no systray)
#
# NB: -Debug is read from $args (not a param block) to preserve --service arg
# forwarding (--instance/--port). Mirrors run-client.ps1's -Debug intent; that script
# has no --service, so it uses a clean param block instead.

$Root        = $PSScriptRoot
$FrontendDir = "$Root\ui\node"
$TauriDir    = "$Root\xgen-node"
$WantDebug   = $args -contains "-Debug"
$env:CARGO_TARGET_DIR = "C:/cargo-targets/XGenProtocol"
$VitePort    = 5174
$InstanceLbl = if ($args -contains "--instance") { $args[([Array]::IndexOf($args, "--instance")) + 1] } else { $null }
$host.UI.RawUI.WindowTitle = "XGen Node"

# ── M-RP-DEVSERVER-GUARD — this launcher OWNS the Vite it starts ──────────────
# THE DEFECT THIS REPLACES: `$vite` is the cmd.exe handle, and the process tree
# under it is FOUR levels deep (MEASURED, not assumed):
#     cmd.exe (ours) -> node npm-cli.js -> cmd.exe /c vite -> node vite.js
# The listener is the LAST of those. `Stop-Process $vite` killed the FIRST and
# nothing else, so npm survived and RESPAWNED vite. Every run leaked a server.
#
# The leak was invisible because the readiness probe asked the wrong question:
# `Invoke-WebRequest localhost:$VitePort` cannot tell "MY server started" from
# "A server is listening". Vite runs strictPort:true, so the next run's own Vite
# DIES on the taken port while the probe is answered by the LEAK and reports
# ready. The app then runs a STALE BUNDLE with no error anywhere.
#
# NB 5174 IS THE NODE'S PORT BY DESIGN - it is not a stray client server. A leak
# here has already cost one session to a wrong diagnosis.
#
# THREE GUARANTEES:
#   1. PRE-FLIGHT REFUSAL  - port already held => abort and NAME the holder.
#   2. OWNERSHIP ASSERTION - the listener must be a DESCENDANT of the process we
#                            spawned. Presence of a listener proves nothing.
#   3. TREE KILL in finally - taskkill /T /F from our handle. Verified to reap
#                            all six processes and release the port.
# RESIDUAL GAP: closing the console window mid-run skips `finally`. Guarantee 1
# then makes the NEXT run refuse loudly instead of silently serving a stale
# bundle - the two halves cover each other.
# NB --service mode never starts a Vite (it goes straight to `cargo run`), so it
# is untouched by all of this.

function Get-PortOwnerPid {
    param([int]$Port)
    $c = Get-NetTCPConnection -State Listen -LocalPort $Port -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($c) { return [int]$c.OwningProcess }
    return $null
}

function Test-IsDescendantOf {
    param([int]$ChildPid, [int]$AncestorPid)
    $seen = @{}
    $cur  = $ChildPid
    while ($cur -and -not $seen.ContainsKey($cur)) {
        if ($cur -eq $AncestorPid) { return $true }
        $seen[$cur] = $true
        $p = Get-CimInstance Win32_Process -Filter "ProcessId=$cur" -ErrorAction SilentlyContinue
        if (-not $p) { return $false }
        $cur = [int]$p.ParentProcessId
    }
    return $false
}

# One-time npm install if node_modules is missing
if (-not (Test-Path "$FrontendDir\node_modules")) {
    Write-Host "Installing frontend dependencies..."
    Push-Location $FrontendDir
    npm install
    Pop-Location
}

Set-Location $TauriDir

if ($args[0] -eq "release") {
    $host.UI.RawUI.WindowTitle = "XGen Node - release build"
    Write-Host "Building release .exe..."
    cargo tauri build
    $exe = "C:\cargo-targets\XGenProtocol\release\xgen-node-app.exe"
    if (Test-Path $exe) {
        Write-Host "Done. Binary: $exe"
        Copy-Item $exe "$Root\bin\xgen-node-app.exe" -Force
        Write-Host "Copied to bin\xgen-node-app.exe"
    }
} elseif ($args -contains "--service") {
    # Forward all args to the binary (supports --instance, --port alongside --service)
    $forwardArgs = $args -join " "
    $host.UI.RawUI.WindowTitle = if ($InstanceLbl) { "XGen Node - SERVICE (headless), instance $InstanceLbl" } else { "XGen Node - SERVICE (headless), DEFAULT DATA DIR" }
    Write-Host "Starting XGen Node in service mode (headless)..."
    $argList = ($args | ForEach-Object { $_ }) -join " "
    Invoke-Expression "cargo run -- $argList"
} else {
    # GUARANTEE 1 - PRE-FLIGHT REFUSAL. Never start on a port someone else holds.
    $holder = Get-PortOwnerPid -Port $VitePort
    if ($holder) {
        $hp = Get-CimInstance Win32_Process -Filter "ProcessId=$holder" -ErrorAction SilentlyContinue
        $host.UI.RawUI.WindowTitle = "XGen Node - REFUSED (port $VitePort in use)"
        Write-Host "REFUSING TO START: port $VitePort is already held by PID $holder."
        if ($hp) { Write-Host "  $($hp.Name) :: $($hp.CommandLine)" }
        Write-Host "  Vite is strictPort:true, so our server would die and the readiness probe"
        Write-Host "  would be answered by THAT process - the app would run a STALE BUNDLE with"
        Write-Host "  no error anywhere. Kill it and re-run:"
        Write-Host "    taskkill /PID $holder /T /F"
        exit 1
    }

    Write-Host "Starting Vite dev server on port $VitePort..."
    $viteCmd = "npm --prefix `"$FrontendDir`" run dev"
    $vite = Start-Process -FilePath "cmd.exe" -ArgumentList "/c", $viteCmd -PassThru -WindowStyle Hidden

    try {
        # GUARANTEE 2 - OWNERSHIP ASSERTION. "My server started", not "a server is listening".
        Write-Host "Waiting for Vite on port $VitePort (PID $($vite.Id) and its tree)..."
        $ready = $false
        for ($i = 0; $i -lt 30; $i++) {
            Start-Sleep -Milliseconds 500
            $owner = Get-PortOwnerPid -Port $VitePort
            if ($owner) {
                if (Test-IsDescendantOf -ChildPid $owner -AncestorPid $vite.Id) {
                    $ready = $true
                    break
                }
                Write-Host "REFUSING TO PROCEED: port $VitePort answered, but by PID $owner,"
                Write-Host "  which is NOT a descendant of the Vite we started (PID $($vite.Id))."
                Write-Host "  A stale bundle would have been served and every probe would have passed."
                exit 1
            }
            if ($vite.HasExited) {
                Write-Host "Vite exited before the port opened (exit code $($vite.ExitCode)) - aborting."
                exit 1
            }
        }

        if (-not $ready) {
            Write-Host "Vite did not start within 15 s - aborting."
            exit 1
        }

        $host.UI.RawUI.WindowTitle = if ($WantDebug) { "XGen Node - dev, Vite $VitePort, CDP 9322" } else { "XGen Node - dev, Vite $VitePort" }
        Write-Host "Vite ready on $VitePort (PID $(Get-PortOwnerPid -Port $VitePort), ours). Starting XGen Node (dev)..."
        $env:TAURI_SKIP_DEVSERVER_CHECK = "true"
        if ($WantDebug) {
            # WebView2 >=136 ignores the env-var route (wry overrides it); the port now rides a
            # dev-only Tauri config OVERLAY (cdp.dev.conf.json), base stays release-safe. See D-104.
            Write-Host "[-Debug] CDP remote-debugging port 9322 via cdp.dev.conf.json overlay (dev-only)."
            cargo tauri dev --config cdp.dev.conf.json
        } else {
            cargo tauri dev
        }
    }
    finally {
        # GUARANTEE 3 - TREE KILL. `exit` inside try still runs this block, so the
        # abort paths above clean up too. Stop-Process on the handle killed cmd ONLY.
        if ($vite -and -not $vite.HasExited) {
            & taskkill /PID $vite.Id /T /F 2>&1 | Out-Null
        }
        Start-Sleep -Milliseconds 400
        $left = Get-PortOwnerPid -Port $VitePort
        if ($left) {
            Write-Host "WARNING: port $VitePort STILL HELD by PID $left after cleanup - the next run will refuse."
        } else {
            Write-Host "Vite stopped; port $VitePort released."
        }
    }
}
