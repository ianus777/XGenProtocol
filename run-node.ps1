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

# One-time npm install if node_modules is missing
if (-not (Test-Path "$FrontendDir\node_modules")) {
    Write-Host "Installing frontend dependencies..."
    Push-Location $FrontendDir
    npm install
    Pop-Location
}

Set-Location $TauriDir

if ($args[0] -eq "release") {
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
    Write-Host "Starting XGen Node in service mode (headless)..."
    $argList = ($args | ForEach-Object { $_ }) -join " "
    Invoke-Expression "cargo run -- $argList"
} else {
    Write-Host "Starting Vite dev server..."
    $viteCmd = "npm --prefix `"$FrontendDir`" run dev"
    $vite = Start-Process -FilePath "cmd.exe" -ArgumentList "/c", $viteCmd -PassThru -WindowStyle Hidden

    Write-Host "Waiting for Vite on port 5174..."
    $ready = $false
    for ($i = 0; $i -lt 30; $i++) {
        Start-Sleep -Milliseconds 500
        try {
            $r = Invoke-WebRequest -Uri "http://localhost:5174" -UseBasicParsing -TimeoutSec 1 -ErrorAction Stop
            $ready = $true
            break
        } catch { }
    }

    if (-not $ready) {
        Write-Host "Vite did not start within 15 s - aborting."
        $vite | Stop-Process -Force -ErrorAction SilentlyContinue
        exit 1
    }

    Write-Host "Vite ready. Starting XGen Node (dev)..."
    $env:TAURI_SKIP_DEVSERVER_CHECK = "true"
    if ($WantDebug) {
        $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9322"
        Write-Host "[-Debug] WebView2 remote-debugging port 9322 enabled (dev-only)."
    }
    cargo tauri dev

    $vite | Stop-Process -Force -ErrorAction SilentlyContinue
}
