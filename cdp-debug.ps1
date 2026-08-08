# XGen UI - CDP Debug Harness (WebView2 remote-debug read loop).
# DEV-ONLY. Reads console output and live UI state from a running XGen WebView2
# window over the Chrome DevTools Protocol. Never target a release build
# (release closes the devtools feature and the port). Spec: docs/CDP_DEBUG_HARNESS.md.
#
# Usage: THIS SCRIPT ATTACHES. IT DOES NOT LAUNCH. Start the app with the matching
# run-*.ps1 -Debug (the cdp.dev.conf.json overlay is the only route that opens the
# port on WebView2 >=136 - D-104), then attach here. -Launch is RETIRED and refuses.
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
# TRUSTED KEYS (M-TOOL-CDP-KEY). `Input.dispatchKeyEvent` is the keyboard half of the same argument.
# Until it existed the harness could CLICK but not PRESS, so every keyboard assertion shipped with a
# stated limit - M-RP-SELECT-ORIENT's L-12 activation gate was driven with a synthetic `keydown`,
# which runs the Svelte listener but does NOT prove the browser routes a physical key there.
#   .\cdp-debug.ps1 -App client -Mode key -Key Enter -At "340,114"     # focus that row, then press
#   .\cdp-debug.ps1 -App client -Mode key -Key Tab -Repeat 12          # WALK the tab order
#   .\cdp-debug.ps1 -App client -Mode key -Key Tab -Modifier Shift     # and walk it backwards
#   .\cdp-debug.ps1 -App client -Mode key -Key ArrowDown -Expression "..."
#
# ** KEYS GO TO document.activeElement, NOT TO A COORDINATE **, so this mode always prints the focused
# element BEFORE and AFTER (tag, own + owner data-debug-id, role, tabindex, aria-selected, text).
# Without that pair, a key hitting a dead handler and a key hitting <body> because nothing was focused
# look IDENTICAL. -Repeat prints every intermediate stop, which is what makes a Tab walk a measurement
# rather than a before/after.
#
# ** -At FOCUSES BY CLICKING, SO IT CANNOT TEST ACTIVATION. ** On any component where a click is
# itself an activation (entity-panel, menu-item, shelf-face - i.e. most of this UI), -At -Key Enter
# passes because of the CLICK and proves nothing about the KEY. Found on this harness's own first
# gate. To test a key path, put focus there WITHOUT a click (`el.focus()` via -Mode eval, or Tab into
# it) and press with NO -At. Use -At only when the click is incidental to what you are asserting.
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
    [ValidateSet('console','state','eval','screenshot','click','drag','key')] [string]$Mode = 'state',
    [ValidateSet('Enter','Space','Tab','Escape','ArrowUp','ArrowDown','ArrowLeft','ArrowRight','Home','End')] [string]$Key = '',
    [ValidateSet('None','Shift','Ctrl','Alt')] [string]$Modifier = 'None',
    [int]$Repeat = 1,
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

# ** A KEY IS FOUR FIELDS, NOT ONE - AND THE MISSING ONES FAIL SILENTLY. **
# `Input.dispatchKeyEvent` accepts `{type,key}` alone, ACKs it, and the page does NOTHING: Chromium
# routes on `windowsVirtualKeyCode`, and a text-producing key additionally needs `text` or no
# `keypress` is generated at all. An event that is accepted and ignored is the N-139 family again -
# the instrument reports success and the assertion reads clean against a dead key.
# ** SO THE SUPPORTED KEYS ARE A TABLE, NOT A PARSER. ** A generic string->key mapper looks general
# and is wrong at exactly the edges that matter (Space's text is " " but its `code` is "Space";
# arrows carry NO text at all; Enter's text is CR, not LF). Ten keys, each measured. Add the eleventh
# when something needs it, with its four fields, rather than guessing a rule that covers it.
function Get-KeySpec {
    param([string]$Name)
    switch ($Name) {
        'Enter'      { return @{ code = 'Enter';      vk = 13; text = "`r" } }
        'Space'      { return @{ code = 'Space';      vk = 32; text = ' '   } }
        'Tab'        { return @{ code = 'Tab';        vk = 9;  text = "`t" } }
        'Escape'     { return @{ code = 'Escape';     vk = 27; text = ''    } }
        'ArrowUp'    { return @{ code = 'ArrowUp';    vk = 38; text = ''    } }
        'ArrowDown'  { return @{ code = 'ArrowDown';  vk = 40; text = ''    } }
        'ArrowLeft'  { return @{ code = 'ArrowLeft';  vk = 37; text = ''    } }
        'ArrowRight' { return @{ code = 'ArrowRight'; vk = 39; text = ''    } }
        'Home'       { return @{ code = 'Home';       vk = 36; text = ''    } }
        'End'        { return @{ code = 'End';        vk = 35; text = ''    } }
        default      { throw "Unsupported -Key '$Name'. Add it to Get-KeySpec with its four fields." }
    }
}

# CDP modifier bitmask: Alt=1, Ctrl=2, Meta=4, Shift=8. Shift+Tab is the one that earns this param -
# a tab-order walk that can only go forwards cannot tell a TRAP from a one-way door.
function Get-ModifierMask {
    param([string]$Name)
    switch ($Name) {
        'Shift' { return 8 } 'Ctrl' { return 2 } 'Alt' { return 1 } default { return 0 }
    }
}

# ** KEYS GO TO document.activeElement, NOT TO A COORDINATE. ** That is why this mode reports the
# focused element BEFORE and AFTER every press: without it, a key delivered to a dead handler and a
# key delivered to <body> because nothing was focused are INDISTINGUISHABLE - both look like "nothing
# happened". The before/after pair is the deliverable, not a courtesy.
function Send-KeyEvent {
    param($Ws, $Token, [string]$Name, [string]$Modifier = 'None')
    $spec = Get-KeySpec $Name
    $mods = Get-ModifierMask $Modifier
    # A key that produces text uses `keyDown` (which generates keypress); one that does not uses
    # `rawKeyDown`. Sending `keyDown` WITHOUT text suppresses keypress silently - measured, not assumed.
    $downType = if ($spec.text -ne '') { 'keyDown' } else { 'rawKeyDown' }
    $common = '"key":' + (ConvertTo-Json $Name) + ',"code":' + (ConvertTo-Json $spec.code) +
              ',"windowsVirtualKeyCode":' + $spec.vk + ',"nativeVirtualKeyCode":' + $spec.vk +
              ',"modifiers":' + $mods
    $textPart = if ($spec.text -ne '') { ',"text":' + (ConvertTo-Json $spec.text) } else { '' }
    [void](Invoke-CdpMethod $Ws $Token 'Input.dispatchKeyEvent' ('{"type":"' + $downType + '",' + $common + $textPart + '}'))
    [void](Invoke-CdpMethod $Ws $Token 'Input.dispatchKeyEvent' ('{"type":"keyUp",' + $common + '}'))
}

# One expression, JSON.stringify'd - PS 5.1 cannot take a multi-statement eval (N-101).
# ** THE PROBE CLIMBS TO THE NEAREST data-debug-id ANCESTOR, AND THAT IS THE POINT (N-110). **
# Focusable elements are mostly UNREGISTERED leaves - an `li[role=option]` inside `entity-panel`
# carries no id of its own, so a probe reading only the focused node reports `id: null` and tells you
# the tag but never WHERE you are. A tab walk whose every stop reads "LI, null" is not a measurement.
# `own` is kept separate from `owner` so the two can never be confused for one another.
$FocusProbeJs = 'JSON.stringify((function(){var e=document.activeElement;if(!e)return null;' +
    'var o=e.closest?e.closest("[data-debug-id]"):null;' +
    'return{tag:e.tagName,own:e.getAttribute("data-debug-id"),' +
    'owner:o?o.getAttribute("data-debug-id"):null,role:e.getAttribute("role"),' +
    'tab:e.getAttribute("tabindex"),aria:e.getAttribute("aria-selected"),' +
    'text:(e.textContent||"").trim().slice(0,28)}})())'

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
        # -Launch IS RETIRED, AND IT REFUSES RATHER THAN PRETENDING (M-RP-DEVSERVER-GUARD).
        # It could not work and it FAILED CONVINCINGLY: it set
        # WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS, which WebView2 >=136 IGNORES (D-104, and
        # this script's own header says so 150 lines above), and it ran $Exe, which
        # defaults to bin\xgen-*.exe - a STALE RELEASE BINARY, not the dev build. The
        # output was two reassuring lines ("Launched..." / "Cleaned up...") plus exit 1,
        # and it put a plausible XGen window on screen. That is worse than no feature.
        # THE ONLY CDP ROUTE IS THE cdp.dev.conf.json OVERLAY, which the run-*.ps1
        # launchers already take. Launch there, then attach with this script.
        Write-Host "-Launch IS RETIRED AND CANNOT WORK. WebView2 >=136 ignores the env-var"
        Write-Host "route (D-104); the only CDP route is the cdp.dev.conf.json overlay."
        Write-Host "Launch the app first, in its own window:"
        Write-Host "    .\run-client.ps1 -Debug     (CDP 9222)"
        Write-Host "    .\run-node.ps1 -Debug       (CDP 9322)"
        Write-Host "    .\run-sampler.ps1 -Debug    (CDP 9422)"
        Write-Host "then re-run this script WITHOUT -Launch to attach."
        exit 1
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
        'key' {
            if ([string]::IsNullOrWhiteSpace($Key)) { throw "-Mode key requires -Key" }
            # -At is OPTIONAL and focuses by TRUSTED CLICK first, so "focus this row, press Enter" is one
            # command. Without it the key goes wherever focus already is - which is exactly what a Tab
            # walk wants, and exactly what makes a stray key look like a dead handler if you forget.
            if ($At) {
                $p = ConvertTo-Point $At 'At'
                if (-not $KeepSelection) { Clear-Selection $ws $cts.Token }
                Write-Host "focus-click @ $($p[0]),$($p[1])  (trusted, CSS px)"
                Send-MouseEvent $ws $cts.Token 'mouseMoved'    $p[0] $p[1] 0 0
                Send-MouseEvent $ws $cts.Token 'mousePressed'  $p[0] $p[1] 1 1
                Send-MouseEvent $ws $cts.Token 'mouseReleased' $p[0] $p[1] 0 1
            }
            Show-Eval $ws $cts.Token 'FOCUS BEFORE:' $FocusProbeJs
            $label = if ($Modifier -eq 'None') { $Key } else { "$Modifier+$Key" }
            for ($k = 0; $k -lt $Repeat; $k++) {
                Write-Host "key $label  (trusted, Input.dispatchKeyEvent)"
                Send-KeyEvent $ws $cts.Token $Key $Modifier
                # -Repeat exists for tab-order WALKS, so each step must be readable, not just the last.
                if ($Repeat -gt 1) { Show-Eval $ws $cts.Token "  FOCUS [$($k+1)/$Repeat]:" $FocusProbeJs }
            }
            Show-Eval $ws $cts.Token 'FOCUS AFTER: ' $FocusProbeJs
            if ($Expression) { Show-Eval $ws $cts.Token 'AFTER:' $Expression }
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
