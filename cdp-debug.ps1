# XGen UI - CDP Debug Harness (WebView2 remote-debug read loop).
# DEV-ONLY. Reads console output and live UI state from a running XGen WebView2
# window over the Chrome DevTools Protocol. Never target a release build
# (release closes the devtools feature and the port). Spec: docs/CDP_DEBUG_HARNESS.md.
#
# Usage:
#   .\cdp-debug.ps1 -App client -Launch -Mode eval -Expression "1+1"  # launch, eval, clean up
#   .\cdp-debug.ps1 -App node   -Launch -Mode console -Seconds 8       # launch node, tail console 8 s
#   .\cdp-debug.ps1 -App node -Mode state                              # attach to a RUNNING node, dump registry
#   .\cdp-debug.ps1 -App client -Mode screenshot                       # attach to a RUNNING client, save PNG to temp\
#   .\cdp-debug.ps1 -App client -Ordinal 1 -Mode eval -Expression "location.href"
#
# TRUSTED INPUT (M-RP7.2). A synthetic MouseEvent from `eval` is UNTRUSTED: `isTrusted:false`, and it
# fires NO native defaults (J-496, proven the hard way). `Input.dispatchMouseEvent` is injected at the
# BROWSER level, so it is trusted and drives real hover, focus, capture and drag:
#   .\cdp-debug.ps1 -App client -Mode click -At "320,140"
#   .\cdp-debug.ps1 -App client -Mode drag  -From "215,400" -To "300,400" -Steps 12
#   .\cdp-debug.ps1 -App client -Mode drag  -From "215,400" -To "300,400" ^
#        -MidExpression "JSON.stringify(__XGEN_LAYOUT__.current)" -Expression "JSON.stringify(__XGEN_LAYOUT__.current)"
#
# ** -MidExpression is evaluated WHILE THE BUTTON IS STILL DOWN.** It is not a convenience: a design that
# previews live but only writes the descriptor on release is INDISTINGUISHABLE from one that writes on
# every move, if you can only read after mouseReleased. The mid-drag read IS the proof.
#
# COORDINATES ARE CSS PIXELS relative to the layout viewport - the same space `getBoundingClientRect()`
# returns, so a rect centre can be handed straight to -At/-From. It is NOT device pixels; do NOT scale by
# devicePixelRatio. Verified by calibration, not assumed (see docs/CDP_DEBUG_HARNESS.md).
#
# ** INTEGER COORDINATES ONLY, AND THAT IS DELIBERATE.** PowerShell renders a [double] with the CURRENT
# CULTURE's decimal separator - on a sk-SK box `123.5` stringifies to `123,5`, which is not JSON, and the
# CDP frame is rejected with an error that looks nothing like a locale bug. Coords are [int] end to end.
#
# Port = base + Ordinal; base is 9222 (client) or 9322 (node), so client and node
# never collide and each instance gets a unique port. Override with -Port / -Exe.
# To debug a `tauri dev` session, launch it with a dev-only Tauri config OVERLAY that adds
# the port (WebView2 >=136 IGNORES the WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS env var — wry
# overrides it with its own AdditionalBrowserArguments; D-104). The sampler's
# run-sampler.ps1 -Debug does this via `cargo tauri dev --config cdp.dev.conf.json` (the
# base tauri.conf.json stays port-free so RELEASE never exposes CDP).

param(
    [ValidateSet('client','node','sampler')] [string]$App = 'client',
    [int]$Ordinal = 0,
    [ValidateSet('console','state','eval','screenshot','click','drag')] [string]$Mode = 'state',
    [string]$Expression = '',
    [string]$MidExpression = '',
    [string]$At = '',
    [string]$From = '',
    [string]$To = '',
    [int]$Steps = 12,
    # ** A harness that can only PASS is a weak harness. ** -KeepSelection suppresses the pre-gesture
    # selection clear so N-118 can be REPRODUCED on demand, not merely avoided. Use it to prove a
    # splitter is genuinely immune (user-select:none + preventDefault) rather than protected by the
    # instrument. If a gesture behaves differently under this switch, the guard is the harness's, not
    # the code's - and the code will fail for a real user who never had a harness clearing up after them.
    [switch]$KeepSelection,
    [int]$Seconds = 8,
    [switch]$Launch,
    [string]$Exe = '',
    [int]$Port = 0,
    [string]$OutFile = ''
)

$ProgressPreference = 'SilentlyContinue'
$ErrorActionPreference = 'Stop'

# Resolve per-app defaults (overridable by -Exe / -Port / -OutFile).
if (-not $Exe)     { $Exe = if ($App -eq 'node') { "$PSScriptRoot\bin\xgen-node.exe" } elseif ($App -eq 'sampler') { "$PSScriptRoot\bin\xgen-sampler.exe" } else { "$PSScriptRoot\bin\xgen-client.exe" } }
$basePort = if ($App -eq 'node') { 9322 } elseif ($App -eq 'sampler') { 9422 } else { 9222 }
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

# --- Trusted input (M-RP7.2) ---------------------------------------------------------------

# Parse "x,y" -> [int[]]. Integers only: see the locale note in the header.
function ConvertTo-Point {
    param([string]$Text, [string]$Name)
    if ($Text -notmatch '^\s*(-?\d+)\s*,\s*(-?\d+)\s*$') { throw "-$Name must be `"x,y`" with INTEGER CSS pixels (got: '$Text')" }
    return @([int]$Matches[1], [int]$Matches[2])
}

# Send a CDP method and drain until its id comes back. Runtime events interleave, so matching on
# the id is the only safe read; taking the next frame off the wire is how you get a phantom result.
$script:CdpId = 100
function Invoke-CdpMethod {
    param($Ws, $Token, [string]$Method, [string]$ParamsJson = '{}')
    $script:CdpId++
    $id = $script:CdpId
    Send-Cdp $Ws $Token ('{"id":' + $id + ',"method":"' + $Method + '","params":' + $ParamsJson + '}')
    for ($i = 0; $i -lt 60; $i++) {
        $obj = (Receive-CdpMessage $Ws $Token) | ConvertFrom-Json
        if ($obj.id -eq $id) { return $obj }
    }
    throw "No CDP reply for $Method (id $id)"
}

function Send-MouseEvent {
    param($Ws, $Token, [string]$Type, [int]$X, [int]$Y, [int]$Buttons = 0, [int]$ClickCount = 0)
    # ** `button` MUST be "none" on a button-up move, or Chromium reports buttons=1 on it. **
    # Measured, not assumed: sending button:"left" with buttons:0 on a mouseMoved makes the listener
    # see a DRAG-move where a HOVER happened. Harmless to a splitter (it only listens after
    # pointerdown) but it would silently poison M-RP7.4, whose drop-band hover must be readable
    # with the button UP. The instrument lied before the code had a chance to.
    $btn = if ($Type -eq 'mouseMoved' -and $Buttons -eq 0) { 'none' } else { 'left' }
    $p = '{"type":"' + $Type + '","x":' + $X + ',"y":' + $Y +
         ',"button":"' + $btn + '","buttons":' + $Buttons + ',"clickCount":' + $ClickCount + '}'
    [void](Invoke-CdpMethod $Ws $Token 'Input.dispatchMouseEvent' $p)
}

# ** CLEAR THE SELECTION BEFORE EVERY GESTURE. THIS IS NOT HYGIENE - IT IS THE DIFFERENCE BETWEEN A
#    DRAG THAT WORKS AND ONE THAT HALF-EXECUTES. **
# A drag across selectable text fires `selectstart` and leaves a SELECTION behind. The NEXT drag from
# the same point presses on that selection - and Chromium treats a selection as DRAGGABLE CONTENT, so
# it opens a native HTML5 drag session, which takes over the mouse and SWALLOWS every subsequent
# mousemove AND the mouseup. The gesture then half-executes in total silence: press lands, one move
# lands, release never does. Cost me three wrong diagnoses ("the barrier kills the stream", "the events
# arrive late") before a page reload made it vanish and named the real cause.
# The same trap is waiting for the SEAM: a splitter must be `user-select: none`, or the first drag
# selects the text under it and the second one sticks the tile to the cursor.
function Clear-Selection {
    param($Ws, $Token)
    $js = 'if(window.getSelection){var s=getSelection();if(s.removeAllRanges)s.removeAllRanges()}1'
    $p  = '{"expression":' + (ConvertTo-Json $js) + ',"returnByValue":true}'
    [void](Invoke-CdpMethod $Ws $Token 'Runtime.evaluate' $p)
}

# ** A CDP ACK IS NOT A GUARANTEE THAT THE DOM HAS MOVED. **
# `Input.dispatchMouseEvent` returns when the BROWSER accepts the event, not when the renderer has run
# the handler - and Svelte's flush is a MICROTASK on top of that (N-117: a click and a read in one eval
# return the PRE-change DOM, and it produced a false accent-leak once already). So every read goes
# behind a DOUBLE requestAnimationFrame with awaitPromise: two frames clears the input dispatch and the
# microtask queue both.
# ** Do NOT put this barrier inside the move loop.** It is a read barrier, not a gesture pacer.
function Wait-Frame {
    param($Ws, $Token)
    $js = 'new Promise(function(r){requestAnimationFrame(function(){requestAnimationFrame(function(){r(1)})})})'
    $p  = '{"expression":' + (ConvertTo-Json $js) + ',"awaitPromise":true,"returnByValue":true}'
    [void](Invoke-CdpMethod $Ws $Token 'Runtime.evaluate' $p)
}

# Evaluate and print, ALWAYS behind the frame barrier above.
function Show-Eval {
    param($Ws, $Token, [string]$Label, [string]$Js)
    Wait-Frame $Ws $Token
    $p = '{"expression":' + (ConvertTo-Json $Js) + ',"returnByValue":true}'
    $obj = Invoke-CdpMethod $Ws $Token 'Runtime.evaluate' $p
    if ($obj.result.exceptionDetails) { Write-Host "$Label ERROR: $($obj.result.exceptionDetails.text)" }
    else { Write-Host "$Label $($obj.result.result.value)" }
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
                # Filter on the SCHEME as well as the type (N-105): an open DevTools window is
                # ITSELF a `page` target (devtools://…), and it sorts FIRST. Attaching to it does not
                # fail loudly — it silently evaluates against the wrong document, and every
                # `window.__XGEN_DEBUG__` read comes back as a bare `EVAL ERROR: Uncaught`, which
                # reads like a broken bridge rather than a wrong target. Only http(s) pages are ours.
                $page = $targets | Where-Object { $_.type -eq 'page' -and $_.url -like 'http*' } | Select-Object -First 1
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
                    # N-142: an UNCAUGHT EXCEPTION is not a console API call, so subscribing to
                    # consoleAPICalled alone renders a crashing app INVISIBLE — measured at M-RP6.9:
                    # 265 tailed lines, zero matches, across a real crash. An instrument that reads
                    # clean during a failure is worse than one that reads nothing, because clean
                    # output looks like evidence (the N-139 family).
                    elseif ($obj.method -eq 'Runtime.exceptionThrown') {
                        $d = $obj.params.exceptionDetails
                        $what = if ($d.exception.description) { $d.exception.description } else { $d.text }
                        $line = "[exception] {0} (line {1}, col {2})" -f $what, $d.lineNumber, $d.columnNumber
                        Write-Host $line -ForegroundColor Red
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
        'click' {
            $p = ConvertTo-Point $At 'At'
            if (-not $KeepSelection) { Clear-Selection $ws $cts.Token }
            Write-Host "click @ $($p[0]),$($p[1])  (trusted, CSS px)"
            Send-MouseEvent $ws $cts.Token 'mouseMoved'    $p[0] $p[1] 0 0
            Send-MouseEvent $ws $cts.Token 'mousePressed'  $p[0] $p[1] 1 1
            Send-MouseEvent $ws $cts.Token 'mouseReleased' $p[0] $p[1] 0 1
            if ($Expression) { Show-Eval $ws $cts.Token 'AFTER:' $Expression }
        }
        'drag' {
            $a = ConvertTo-Point $From 'From'
            $b = ConvertTo-Point $To   'To'
            if ($Steps -lt 1) { $Steps = 1 }
            if (-not $KeepSelection) { Clear-Selection $ws $cts.Token }
            Write-Host "drag $($a[0]),$($a[1]) -> $($b[0]),$($b[1])  in $Steps steps  (trusted, CSS px)"
            Send-MouseEvent $ws $cts.Token 'mouseMoved'   $a[0] $a[1] 0 0
            Send-MouseEvent $ws $cts.Token 'mousePressed' $a[0] $a[1] 1 1
            for ($s = 1; $s -le $Steps; $s++) {
                # Integer interpolation: see the locale note. Round, never emit a fraction.
                $x = [int][Math]::Round($a[0] + ($b[0] - $a[0]) * $s / $Steps)
                $y = [int][Math]::Round($a[1] + ($b[1] - $a[1]) * $s / $Steps)
                Send-MouseEvent $ws $cts.Token 'mouseMoved' $x $y 1 0
                # 16 ms = one frame. Chromium coalesces mousemove per frame; firing faster drops moves.
                Start-Sleep -Milliseconds 16
            }
            # THE MID-DRAG READ - the button is STILL DOWN. This is the only place a live preview
            # can be told apart from a descriptor written on every move.
            if ($MidExpression) { Show-Eval $ws $cts.Token 'MID (button down):' $MidExpression }
            Send-MouseEvent $ws $cts.Token 'mouseReleased' $b[0] $b[1] 0 1
            if ($Expression) { Show-Eval $ws $cts.Token 'AFTER (released):' $Expression }
        }
        'screenshot' {
            # Page.captureScreenshot -> base64 PNG in result.data. The app window must be
            # rendered (do NOT minimise when capturing). CDP doesn't 'see CSS' semantically,
            # but the screenshot is the rendered cascade — the eye-check surface for a skin pass.
            $shot = if ($OutFile -like '*.png') { $OutFile } else { "$PSScriptRoot\temp\cdp-shot-$App.png" }
            New-Item -ItemType Directory -Force -Path (Split-Path $shot) | Out-Null
            Send-Cdp $ws $cts.Token '{"id":1,"method":"Page.captureScreenshot","params":{"format":"png"}}'
            for ($i = 0; $i -lt 40; $i++) {
                $obj = (Receive-CdpMessage $ws $cts.Token) | ConvertFrom-Json
                if ($obj.id -eq 1) {
                    if ($obj.result.exceptionDetails) { Write-Host "SHOT ERROR: $($obj.result.exceptionDetails.text)"; break }
                    $data = $obj.result.data
                    if ([string]::IsNullOrEmpty($data)) { Write-Host 'No screenshot data returned.'; break }
                    [System.IO.File]::WriteAllBytes($shot, [System.Convert]::FromBase64String($data))
                    Write-Host "Screenshot -> $shot ($((Get-Item $shot).Length) bytes)"
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
