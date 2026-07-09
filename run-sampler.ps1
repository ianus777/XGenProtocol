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
        # WebView2 Evergreen >=136 (Chromium-136 remote-debug guard) IGNORES the
        # WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS env var (wry overrides it with its own
        # programmatic AdditionalBrowserArguments), so the old env-var route no longer opens
        # the port. The port now rides a dev-only Tauri config OVERLAY (cdp.dev.conf.json)
        # merged via --config; the base tauri.conf.json stays port-free so RELEASE builds
        # never expose CDP. See D-104 / tasks/CDP_DEBUG_HARNESS.md.
        Write-Host "[-Debug] CDP remote-debugging port 9422 via cdp.dev.conf.json overlay (dev-only)."
        cargo tauri dev --config cdp.dev.conf.json
    } else {
        cargo tauri dev
    }

    $vite | Stop-Process -Force -ErrorAction SilentlyContinue
}
