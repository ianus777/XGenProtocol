# XGen Client - dev and release launcher.
# Usage:
#   .\run-client.ps1            - dev mode (hot-reload, no install needed)
#   .\run-client.ps1 -Debug     - dev mode + WebView2 remote-debug port 9222 (CDP harness, dev-only)
#   .\run-client.ps1 release    - build standalone .exe

param(
    [string]$Mode = "",
    [switch]$Debug
)

$Root        = $PSScriptRoot
$FrontendDir = "$Root\ui\client"
$TauriDir    = "$Root\xgen-client"
$env:CARGO_TARGET_DIR = "C:/cargo-targets/XGenProtocol"

# One-time npm install if node_modules is missing
if (-not (Test-Path "$FrontendDir\node_modules")) {
    Write-Host "Installing frontend dependencies..."
    Push-Location $FrontendDir
    npm install
    Pop-Location
}

Set-Location $TauriDir

if ($Mode -eq "release") {
    Write-Host "Building release .exe..."
    cargo tauri build
    $exe = "C:\cargo-targets\XGenProtocol\release\xgen-client-app.exe"
    if (Test-Path $exe) {
        Write-Host "Done. Binary: $exe"
        Copy-Item $exe "$Root\bin\xgen-client-app.exe" -Force
        Write-Host "Copied to bin\xgen-client-app.exe"
    }
} else {
    Write-Host "Starting Vite dev server..."
    $viteCmd = "npm --prefix `"$FrontendDir`" run dev"
    $vite = Start-Process -FilePath "cmd.exe" -ArgumentList "/c", $viteCmd -PassThru -WindowStyle Hidden

    Write-Host "Waiting for Vite on port 5173..."
    $ready = $false
    for ($i = 0; $i -lt 30; $i++) {
        Start-Sleep -Milliseconds 500
        try {
            $r = Invoke-WebRequest -Uri "http://localhost:5173" -UseBasicParsing -TimeoutSec 1 -ErrorAction Stop
            $ready = $true
            break
        } catch { }
    }

    if (-not $ready) {
        Write-Host "Vite did not start within 15 s - aborting."
        $vite | Stop-Process -Force -ErrorAction SilentlyContinue
        exit 1
    }

    Write-Host "Vite ready. Starting XGen Client (dev)..."
    $env:TAURI_SKIP_DEVSERVER_CHECK = "true"
    if ($Debug) {
        $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9222"
        Write-Host "[-Debug] WebView2 remote-debugging port 9222 enabled (dev-only)."
    }
    cargo tauri dev

    $vite | Stop-Process -Force -ErrorAction SilentlyContinue
}
