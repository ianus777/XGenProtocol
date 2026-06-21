# XGen UI - CDP Debug Harness (WebView2 remote-debug read loop).
# DEV-ONLY. Reads console output and live UI state from a running XGen WebView2
# window over the Chrome DevTools Protocol. Never target a release build
# (release closes the devtools feature and the port). Spec: tasks/CDP_DEBUG_HARNESS.md.
#
# Usage:
#   .\cdp-debug.ps1 -App client -Launch -Mode eval -Expression "1+1"  # launch, eval, clean up
#   .\cdp-debug.ps1 -App node   -Launch -Mode console -Seconds 8       # launch node, tail console 8 s
#   .\cdp-debug.ps1 -App node -Mode state                              # attach to a RUNNING node, dump registry
#   .\cdp-debug.ps1 -App client -Ordinal 1 -Mode eval -Expression "location.href"
#
# Port = base + Ordinal; base is 9222 (client) or 9322 (node), so client and node
# never collide and each instance gets a unique port. Override with -Port / -Exe.
# To debug a `tauri dev` session, launch it with the env var first, e.g.:
#   $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9222'

param(
    [ValidateSet('client','node')] [string]$App = 'client',
    [int]$Ordinal = 0,
    [ValidateSet('console','state','eval')] [string]$Mode = 'state',
    [string]$Expression = '',
    [int]$Seconds = 8,
    [switch]$Launch,
    [string]$Exe = '',
    [int]$Port = 0,
    [string]$OutFile = ''
)

$ProgressPreference = 'SilentlyContinue'
$ErrorActionPreference = 'Stop'

# Resolve per-app defaults (overridable by -Exe / -Port / -OutFile).
if (-not $Exe)     { $Exe = if ($App -eq 'node') { "$PSScriptRoot\bin\xgen-node.exe" } else { "$PSScriptRoot\bin\xgen-client.exe" } }
$basePort = if ($App -eq 'node') { 9322 } else { 9222 }
$port = if ($Port -gt 0) { $Port } else { $basePort + $Ordinal }
if (-not $OutFile) { $OutFile = "$PSScriptRoot\temp\cdp-debug-$App.txt" }

$launchedPid = $null
$ws = $null

function Receive-CdpMessage {
    param($Ws, $Token)
    $buf = New-Object byte[] 65536
    $sb  = New-Object System.Text.StringBuilder
    do {
        $seg = New-Object System.ArraySegment[byte] (,$buf)
        $r = $Ws.ReceiveAsync($seg, $Token); $r.Wait()
        [void]$sb.Append([System.Text.Encoding]::UTF8.GetString($buf, 0, $r.Result.Count))
    } while (-not $r.Result.EndOfMessage)
    return $sb.ToString()
}

function Send-Cdp {
    param($Ws, $Token, $Json)
    $b = [System.Text.Encoding]::UTF8.GetBytes($Json)
    $seg = New-Object System.ArraySegment[byte] (,$b)
    $Ws.SendAsync($seg, [System.Net.WebSockets.WebSocketMessageType]::Text, $true, $Token).Wait()
}

function Format-ConsoleArgs {
    param($Params)
    $parts = @()
    foreach ($a in $Params.args) {
        if     ($null -ne $a.value)       { $parts += [string]$a.value }
        elseif ($null -ne $a.description) { $parts += [string]$a.description }
        elseif ($null -ne $a.preview)     { $parts += [string]$a.preview.description }
        else                              { $parts += [string]$a.type }
    }
    return ($parts -join ' ')
}

try {
    if ($Launch) {
        if (-not (Test-Path $Exe)) { throw "Exe not found: $Exe" }
        $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$port"
        $proc = Start-Process -FilePath $Exe -PassThru
        $launchedPid = $proc.Id
        Write-Host "Launched $Exe (PID $launchedPid), remote-debugging-port=$port"
    }

    # Resolve the page target. Fast TCP probe first (a refused port fails in ~ms,
    # not the ~2 s an HTTP request waits), then fetch /json once the port is live.
    $wsUrl = $null; $pageUrl = $null
    for ($i = 0; $i -lt 60; $i++) {
        $up = $false
        try {
            $tcp = New-Object System.Net.Sockets.TcpClient
            $iar = $tcp.BeginConnect('127.0.0.1', $port, $null, $null)
            if ($iar.AsyncWaitHandle.WaitOne(200)) { $tcp.EndConnect($iar); $up = $true }
            $tcp.Close()
        } catch { }
        if ($up) {
            try {
                $targets = (Invoke-WebRequest -Uri "http://127.0.0.1:$port/json" -UseBasicParsing -TimeoutSec 3).Content | ConvertFrom-Json
                $page = $targets | Where-Object { $_.type -eq 'page' } | Select-Object -First 1
                if ($page) { $wsUrl = $page.webSocketDebuggerUrl; $pageUrl = $page.url; break }
            } catch { }
        }
        Start-Sleep -Milliseconds 300
    }
    if (-not $wsUrl) { throw "No CDP page target on port $port. Is the app running with remote-debugging-port=$port?" }
    Write-Host "Target: $pageUrl"
    Write-Host "WS:     $wsUrl"

    $ws  = New-Object System.Net.WebSockets.ClientWebSocket
    $cts = New-Object System.Threading.CancellationTokenSource
    $cts.CancelAfter([Math]::Max($Seconds, 10) * 1000)
    $ws.ConnectAsync([Uri]$wsUrl, $cts.Token).Wait()
    if ($ws.State -ne 'Open') { throw "WebSocket not Open: $($ws.State)" }

    New-Item -ItemType Directory -Force -Path (Split-Path $OutFile) | Out-Null

    switch ($Mode) {
        'console' {
            Send-Cdp $ws $cts.Token '{"id":1,"method":"Runtime.enable"}'
            Write-Host "Console-tail for $Seconds s (-> $OutFile). Ctrl+C to stop early."
            "" | Set-Content -Path $OutFile -Encoding UTF8
            $tcts = New-Object System.Threading.CancellationTokenSource
            $tcts.CancelAfter($Seconds * 1000)
            try {
                while ($true) {
                    $msg = Receive-CdpMessage $ws $tcts.Token
                    $obj = $msg | ConvertFrom-Json
                    if ($obj.method -eq 'Runtime.consoleAPICalled') {
                        $line = "[{0}] {1}" -f $obj.params.type, (Format-ConsoleArgs $obj.params)
                        Write-Host $line
                        Add-Content -Path $OutFile -Value $line -Encoding UTF8
                    }
                }
            } catch { }  # cancellation ends the tail
        }
        'state' {
            $expr = 'window.__XGEN_DEBUG__ ? JSON.stringify(window.__XGEN_DEBUG__.snapshot()) : null'
            Send-Cdp $ws $cts.Token ('{"id":1,"method":"Runtime.evaluate","params":{"expression":' + (ConvertTo-Json $expr) + ',"returnByValue":true}}')
            for ($i = 0; $i -lt 20; $i++) {
                $obj = (Receive-CdpMessage $ws $cts.Token) | ConvertFrom-Json
                if ($obj.id -eq 1) {
                    $val = $obj.result.result.value
                    $out = if ($null -eq $val) { 'null' } else { [string]$val }
                    $out | Set-Content -Path $OutFile -Encoding UTF8
                    if ($out -eq 'null') { Write-Host "window.__XGEN_DEBUG__ is null/undefined (no registry yet)." }
                    Write-Host "State -> ${OutFile}:"; Write-Host $out
                    break
                }
            }
        }
        'eval' {
            if ([string]::IsNullOrWhiteSpace($Expression)) { throw "-Mode eval requires -Expression" }
            Send-Cdp $ws $cts.Token ('{"id":1,"method":"Runtime.evaluate","params":{"expression":' + (ConvertTo-Json $Expression) + ',"returnByValue":true}}')
            for ($i = 0; $i -lt 20; $i++) {
                $obj = (Receive-CdpMessage $ws $cts.Token) | ConvertFrom-Json
                if ($obj.id -eq 1) {
                    if ($obj.result.exceptionDetails) { Write-Host "EVAL ERROR: $($obj.result.exceptionDetails.text)" }
                    else { Write-Host "EVAL RESULT: $($obj.result.result.value)" }
                    break
                }
            }
        }
    }
}
finally {
    if ($ws) { try { $ws.Dispose() } catch { } }
    if ($launchedPid) {
        & taskkill /PID $launchedPid /T /F 2>&1 | Out-Null
        Write-Host "Cleaned up launched process tree (PID $launchedPid)."
    }
    Remove-Item Env:\WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ErrorAction SilentlyContinue
}
