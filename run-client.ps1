# Run the XGen Client UI in development mode.
# Usage: .\run-client.ps1
# First run: installs npm dependencies automatically.

$Root    = $PSScriptRoot
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

Write-Host "Starting XGen Client..."
Set-Location $TauriDir
cargo tauri dev
