# XGen Sampler - dev and release launcher (M-RP3.0).
# Usage:
#   .\run-sampler.ps1            - dev mode (hot-reload; live skin.css editing via Vite HMR)
#   .\run-sampler.ps1 -Debug     - dev mode + WebView2 remote-debug port 9422 (CDP harness, dev-only)
#   .\run-sampler.ps1 release    - build standalone .exe
#
# The sampler is the component test-bed (D-097): build/tune components here, in
# the real WebView2 runtime, with live client<->node skin-swap. Editing the
# canonical ui/assets/skin.css while this runs hot-applies instantly (Vite HMR).

param(
    [string]$Mode = "",
    [switch]$Debug
)

$Root        = $PSScriptRoot
$FrontendDir = "$Root\ui\sampler"
$TauriDir    = "$Root\xgen-sampler"
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
} else {
    Write-Host "Starting Vite dev server (port 5175)..."
    $viteCmd = "npm --prefix `"$FrontendDir`" run dev"
    $vite = Start-Process -FilePath "cmd.exe" -ArgumentList "/c", $viteCmd -PassThru -WindowStyle Hidden

    Write-Host "Waiting for Vite on port 5175..."
    $ready = $false
    for ($i = 0; $i -lt 30; $i++) {
        Start-Sleep -Milliseconds 500
        try {
            $r = Invoke-WebRequest -Uri "http://localhost:5175" -UseBasicParsing -TimeoutSec 1 -ErrorAction Stop
            $ready = $true
            break
        } catch { }
    }

    if (-not $ready) {
        Write-Host "Vite did not start within 15 s - aborting."
        $vite | Stop-Process -Force -ErrorAction SilentlyContinue
        exit 1
    }

    Write-Host "Vite ready. Starting XGen Sampler (dev)..."
    $env:TAURI_SKIP_DEVSERVER_CHECK = "true"
    if ($Debug) {
        $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9422"
        Write-Host "[-Debug] WebView2 remote-debugging port 9422 enabled (dev-only)."
    }
    cargo tauri dev

    $vite | Stop-Process -Force -ErrorAction SilentlyContinue
}
