param([switch]$Apply)

# legd-move.ps1 -- M-DOC-ROADTREE Leg D, section C: the B2 archive move.
# ASCII-only by design: a BOM-less UTF-8 .ps1 is read as ANSI by PS 5.1, so every
# non-ASCII string this script writes comes from legd-annotations.json instead.
# Default is a dry run. Pass -Apply to write.

$ErrorActionPreference = 'Stop'
$root = 'E:\Projects\XGenProtocol'
$claudePath  = Join-Path $root 'CLAUDE.md'
$histPath    = Join-Path $root 'CLAUDE_HISTORY.md'
$dataPath    = Join-Path $root 'legd-annotations.json'
$utf8 = New-Object System.Text.UTF8Encoding($false)

function Fail($msg) { throw "ASSERTION FAILED: $msg" }
function Ok($msg)   { Write-Host ("  ok   " + $msg) }

$d = [IO.File]::ReadAllText($dataPath, [Text.Encoding]::UTF8) | ConvertFrom-Json

$claudeRaw = [IO.File]::ReadAllText($claudePath, [Text.Encoding]::UTF8)
$histRaw   = [IO.File]::ReadAllText($histPath,   [Text.Encoding]::UTF8)
if ($claudeRaw.Contains("`r")) { Fail 'CLAUDE.md contains CR; expected LF-only' }
if ($histRaw.Contains("`r"))   { Fail 'CLAUDE_HISTORY.md contains CR; expected LF-only' }

$L = $claudeRaw -split "`n"
$H = $histRaw   -split "`n"

Write-Host ''
Write-Host '=== PRE-STATE (V8: whole-document invariants) ==='
$preLineCount   = $L.Count
$preFirst       = $L[0]
$preLast        = $L[$L.Count-1]
$preL29Len      = $L[28].Length
$preHistCount   = $H.Count
$preHistHeads   = ($H | Where-Object { $_ -like '## *' }).Count
Write-Host ("  CLAUDE.md lines={0}  L29len={1}" -f $preLineCount, $preL29Len)
Write-Host ("  HISTORY   lines={0}  '## ' headings={1}" -f $preHistCount, $preHistHeads)
if ($preL29Len -ne 124299) { Fail "L29 length is $preL29Len, expected 124299" }
if ($preHistHeads -ne 185) { Fail "history heading count is $preHistHeads, expected 185" }

# ---- build block ranges over B2 only -------------------------------------
$b2s = [int]$d.b2Start
$b2e = [int]$d.b2End
$heads = @()
for ($i = $b2s - 1; $i -le $b2e - 1; $i++) {
  if ($L[$i].StartsWith('> ###')) { $heads += ($i + 1) }
}
if ($heads.Count -ne [int]$d.expectedBlocks) { Fail ("block count is {0}, expected {1}" -f $heads.Count, $d.expectedBlocks) }
Ok ("block count = " + $heads.Count)

# guard the boundary that would have swallowed the standing items
if (-not $L[$b2e].StartsWith('> **')) { Fail ("line " + ($b2e+1) + " is not the first standing item; B2 end is wrong") }
Ok ("B2 ends at L$b2e; L" + ($b2e+1) + " is a standing '> **' item and is out of scope")

$blocks = @()
for ($k = 0; $k -lt $heads.Count; $k++) {
  $s = $heads[$k]
  $e = if ($k -lt $heads.Count - 1) { $heads[$k+1] - 1 } else { $b2e }
  $txt = ($L[($s-1)..($e-1)] -join "`n")
  $cp  = [Char]::ConvertToUtf32(($L[$s-1] -replace '^> ###\s*',''), 0)
  $blocks += [pscustomobject]@{ Start=$s; End=$e; Sym=$cp; Text=$txt }
}

$census = @{}
foreach ($b in $blocks) { if ($census.ContainsKey($b.Sym)) { $census[$b.Sym]++ } else { $census[$b.Sym] = 1 } }
$tot = 0
foreach ($k in ($census.Keys | Sort-Object)) { Write-Host ("  U+{0:X}  {1}" -f $k, $census[$k]); $tot += $census[$k] }
if ($tot -ne $heads.Count) { Fail "census total $tot != block count" }
Ok "symbol census totals to the block count"

# ---- move set -------------------------------------------------------------
$moveLines = New-Object System.Collections.Generic.HashSet[int]
foreach ($b in $blocks) { if ($b.Sym -eq 0x2705) { [void]$moveLines.Add($b.Start) } }
$doneCount = $moveLines.Count
if ($doneCount -ne 42) { Fail "DONE-symbol blocks = $doneCount, expected 42" }
foreach ($n in $d.moveExtra) { if (-not $moveLines.Add([int]$n)) { Fail "line $n listed twice in the move set" } }
if ($moveLines.Count -ne [int]$d.expectedMove) { Fail ("move set = {0}, expected {1}" -f $moveLines.Count, $d.expectedMove) }
Ok ("move set = " + $moveLines.Count + " (42 by symbol + " + $d.moveExtra.Count + " by reading)")

$stay = @($blocks | Where-Object { -not $moveLines.Contains($_.Start) })
if ($stay.Count -ne [int]$d.expectedStay) { Fail ("stay set = {0}, expected {1}" -f $stay.Count, $d.expectedStay) }
Ok ("stay set = " + $stay.Count + "; move + stay = " + ($moveLines.Count + $stay.Count))

# ---- annotations ----------------------------------------------------------
$work = $L.Clone()
$annCount = 0
foreach ($p in $d.annotations.PSObject.Properties) {
  $n = [int]$p.Name
  if (-not $moveLines.Contains($n)) { Fail "annotation targets L$n which is not in the move set" }
  if (-not $work[$n-1].StartsWith('> ###')) { Fail "annotation target L$n is not a headline" }
  $work[$n-1] = $work[$n-1] + $p.Value
  $annCount++
}
foreach ($p in $d.stayAnnotations.PSObject.Properties) {
  $n = [int]$p.Name
  if ($moveLines.Contains($n)) { Fail "stay-annotation targets L$n which IS in the move set" }
  if (-not $work[$n-1].StartsWith('> ###')) { Fail "stay-annotation target L$n is not a headline" }
  $work[$n-1] = $work[$n-1] + $p.Value
  $annCount++
}
Ok "annotations applied = $annCount"

# rebuild block texts from the annotated copy
$moved = @()
$kept  = @()
foreach ($b in $blocks) {
  $t = ($work[($b.Start-1)..($b.End-1)] -join "`n")
  if ($moveLines.Contains($b.Start)) { $moved += [pscustomobject]@{ Start=$b.Start; End=$b.End; Text=$t } }
  else { $kept += $b.Start }
}
if ($moved.Count -ne [int]$d.expectedMove) { Fail "moved block count mismatch" }

# ---- V9: no closure block separated from its phase-0 ----------------------
$pairs = @{ '123'='125'; '145'='173'; '151'='157'; '155'='157'; '161'='173'; '165'='169'; '167'='169'; '191'='253'; '219'='213'; '259'='189'; '261'='173' }
foreach ($k in $pairs.Keys) {
  if (-not $moveLines.Contains([int]$k))          { Fail "V9: phase-0 L$k not in move set" }
  if (-not $moveLines.Contains([int]$pairs[$k]))  { Fail ("V9: closure L" + $pairs[$k] + " for phase-0 L$k not in move set") }
}
Ok "V9: all 11 phase-0 blocks travel with their closure block"

# ---- build the new files --------------------------------------------------
$drop = New-Object System.Collections.Generic.HashSet[int]
foreach ($m in $moved) { for ($i = $m.Start; $i -le $m.End; $i++) { [void]$drop.Add($i) } }
$newClaude = @()
for ($i = 1; $i -le $work.Count; $i++) { if (-not $drop.Contains($i)) { $newClaude += $work[$i-1] } }
$newClaudeText = ($newClaude -join "`n")

$batch = @()
$batch += $d.batchHeading
$batch += ''
foreach ($m in $moved) {
  $t = $m.Text.TrimEnd("`n")
  $batch += ($t -split "`n")
  $batch += ''
}

if ($H[2]  -notlike '> Version: 1.1*')            { Fail 'history L3 is not the expected Version line' }
if ($H[4]  -notlike '> **Last updated**: 2026-06-22*') { Fail 'history L5 is not the expected Last updated line' }
if ($H[12] -ne '---')                              { Fail 'history L13 is not the --- separator' }
$H[2] = '> Version: 1.2  '
$H[4] = '> **Last updated**: 2026-07-29  '

$newHist = @()
$newHist += $H[0..10]
$newHist += ''
$newHist += $d.preambleAdd
$newHist += $H[11]
$newHist += $H[12]
$newHist += $H[13]
$newHist += $batch
$newHist += $H[14..($H.Count-1)]
$newHistText = ($newHist -join "`n")

Write-Host ''
Write-Host '=== POST-STATE (asserted before any write) ==='
$movedLineTotal = ($moved | Measure-Object -Property Start -Sum).Count
$droppedLines = $drop.Count
if ($newClaude.Count -ne ($preLineCount - $droppedLines)) { Fail 'CLAUDE.md line arithmetic does not close' }
Ok ("CLAUDE.md " + $preLineCount + " -> " + $newClaude.Count + " lines (" + $droppedLines + " removed)")
if ($newClaude[0] -ne $preFirst) { Fail 'first line changed' }
if ($newClaude[$newClaude.Count-1] -ne $preLast) { Fail 'LAST LINE CHANGED -- tail lost' }
Ok 'first and last line intact'
if ($newClaude[28].Length -ne 124299) { Fail 'L29 changed' }
Ok 'L29 still 124299 chars'

$newHeads = ($newClaude | Where-Object { $_ -like '> ###*' }).Count
if ($newHeads -ne $stay.Count) { Fail "remaining headline count $newHeads != stay set" }
Ok ("V3: " + $moved.Count + " archived + " + $newHeads + " remaining = " + ($moved.Count + $newHeads))

foreach ($m in $moved) {
  if (-not $newHistText.Contains($m.Text.TrimEnd("`n"))) { Fail ("V1: block L" + $m.Start + " not byte-identical in history") }
  if ($newClaudeText.Contains($m.Text.TrimEnd("`n")))    { Fail ("block L" + $m.Start + " still present in CLAUDE.md") }
}
Ok 'V1: every moved block is byte-identical in history and absent from the head'

$newHistHeads = ($newHist | Where-Object { $_ -like '## *' }).Count
if ($newHistHeads -ne ($preHistHeads + 1)) { Fail "history headings $newHistHeads, expected $($preHistHeads+1)" }
Ok ("history headings " + $preHistHeads + " -> " + $newHistHeads + " (one batch heading, F1c)")

$preLastReal = ($L | Where-Object { $_.Trim() -ne '' })[-1]
foreach ($s in @('TRUSTED-MOUSE','M-RP-FOCUS','Track A')) {
  if (-not $newClaudeText.Contains($s)) { Fail "standing item '$s' lost from the head" }
}
Ok 'the four standing items past L262 are still in the head'
$postLastReal = ($newClaude | Where-Object { $_.Trim() -ne '' })[-1]
if ($postLastReal -ne $preLastReal) { Fail 'LAST NON-EMPTY LINE CHANGED -- tail lost' }
Ok 'last non-empty line intact (the file ends with a newline, so the raw last element is empty and proves nothing)'

Write-Host ''
if ($Apply) {
  [IO.File]::WriteAllText($claudePath, $newClaudeText, $utf8)
  [IO.File]::WriteAllText($histPath,   $newHistText,   $utf8)
  Write-Host '=== APPLIED ==='
  Write-Host ("  CLAUDE.md         " + (Get-Item $claudePath).Length + " bytes")
  Write-Host ("  CLAUDE_HISTORY.md " + (Get-Item $histPath).Length + " bytes")
} else {
  Write-Host '=== DRY RUN -- nothing written. Re-run with -Apply. ==='
}
