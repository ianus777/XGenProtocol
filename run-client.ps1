# XGen Client — dev and release launcher.
# Usage:
#   .\run-client.ps1           — dev mode (hot-reload, no install needed)
#   .\run-client.ps1 release   — build standalone .exe

$Root        = $PSScriptRoot
$FrontendDir = "$Root\ui\dev_core_ui\client_ui"
$TauriDir    = "$Root\xgen-client\src-tauri"
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
    $exe = "C:\cargo-targets\XGenProtocol\release\xgen-client-app.exe"
    if (Test-Path $exe) {
        Write-Host "Done. Binary: $exe"
        Copy-Item $exe "$Root\bin\xgen-client-app.exe" -Force
        Write-Host "Copied to bin\xgen-client-app.exe"
    }
} else {
    Write-Host "Starting XGen Client (dev)..."
    cargo tauri dev
}
