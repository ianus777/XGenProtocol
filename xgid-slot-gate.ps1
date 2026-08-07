# xgid-slot-gate.ps1 - the D-137 enforcement gate.
#
# WHY THIS EXISTS: D-136 says a completed sweep is not a standing rule. D-137 says
# a promotion-watch with a named trigger is worth nothing if nobody owns the trigger.
# This script is the owner. Run it from the session-open reading order.
#
# WHAT IT DOES: sweeps every .rs in the four crates for identifier-shaped String slots
# written AFTER the XGID retrofit arc closed (7ed4e30), and compares the result against
# xgid-slot-manifest.tsv keyed on File+Struct+Field.
#   NEW slot not on the manifest  -> FAIL. Classify it, then add it.
#   Manifest row no longer found  -> WARN. Expected while Legs B/C retype; never silent.
#
# WHAT IT CANNOT DO (stated, not hidden): it catches a slot that is String and should not
# be. It CANNOT catch a slot correctly String at a boundary and wrongly consumed as
# internal state one function away - which is SeenRecord's actual shape. Only a read sees
# that. The gate clears the desk; it does not retire the read.

[CmdletBinding()]
param(
    [string]$RepoRoot = 'E:\Projects\XGenProtocol',
    [string]$Manifest = 'xgid-slot-manifest.tsv',
    [string]$ArcClose = '7ed4e30',
    # Only the plant test should pass this. A dirty tree silently changes the baseline:
    # a planted slot in a TRACKED file was counted as real and moved 255 to 256 (J-661).
    [switch]$AllowDirty
)

Set-Location $RepoRoot
$manPath = Join-Path $RepoRoot $Manifest
if (-not (Test-Path $manPath)) { Write-Host "GATE ERROR: manifest not found at $manPath" -ForegroundColor Red; exit 2 }

# Check 0 - the gate must not measure work that is not in the repo.
# Two separate contamination vectors, and ls-files alone only closes the first:
#   UNTRACKED .rs   -> excluded by sourcing the file set from git ls-files below.
#   MODIFIED .rs    -> still reads dirty from disk, so it is refused here.
$dirty = @(git status --porcelain -- 'xgen-common' 'xgen-core' 'xgen-client' 'xgen-node' |
           Where-Object { $_ -match '\.rs$' })
if ($dirty.Count) {
    if (-not $AllowDirty) {
        Write-Host "GATE FAIL: $($dirty.Count) uncommitted .rs change(s) - the baseline is not reproducible." -ForegroundColor Red
        $dirty | ForEach-Object { Write-Host "  $_" }
        Write-Host "Commit or stash, then re-run. Pass -AllowDirty only to exercise the gate itself." -ForegroundColor Red
        exit 1
    }
    Write-Host "GATE WARNING: -AllowDirty is set and $($dirty.Count) .rs file(s) are uncommitted." -ForegroundColor Yellow
    Write-Host "This run measures the WORKING TREE, not the repo. Do not quote its numbers." -ForegroundColor Yellow
}

$rx = '^\s*(pub\s+)?([a-z_]*(_id|_xgid|_node|_identity|_sender|_space|_room|_event|_thread|_author|_owner|_actor|_by)|id|xgid|sender|identity|node|space|room|event|thread|author|owner|actor|home_node|target)\s*:\s*(Option<)?String'

# KNOWN BLIND SPOT, measured 2026-08-06 (M-RP-MEMBER-ACT Leg A/B read, Clair's Finding 2).
# This regex is NAME-KEYED, so an XGID-bearing String whose identifier matches none of the
# suffixes above is INVISIBLE to the gate. Concrete live example: `KnownSpace.counterpart`
# (an identity XGID) does NOT match, while `counterpart_id` WOULD. A PASS therefore means
# "nothing NAMED like a slot is untyped" - it does NOT mean "no untyped XGID slot exists".
# Do not read a pass as classification. Whether D-137's mechanism should catch this is
# open and Joe's; recorded here rather than silently widened (D-145).

# File set comes from the INDEX, not the filesystem: Get-ChildItem would sweep untracked
# scratch files and count them as production slots (J-661).
$tracked = @(git ls-files -- 'xgen-common/*.rs' 'xgen-core/*.rs' 'xgen-client/*.rs' 'xgen-node/*.rs' |
             Where-Object { $_ -notmatch '(^|/)(target|\.claude)/' })
if (-not $tracked.Count) { Write-Host 'GATE ERROR: git ls-files matched no .rs files.' -ForegroundColor Red; exit 2 }
$hits = $tracked | ForEach-Object { Select-String -Path $_ -Pattern $rx }

$found = @()
foreach ($g in ($hits | Group-Object Path)) {
    $rel = (Resolve-Path -Relative $g.Name) -replace '^\.\\','' -replace '\\','/'
    $bl = git blame --line-porcelain -- $rel 2>$null
    $sha = @{}; $ln = 0
    for ($i = 0; $i -lt $bl.Count; $i++) {
        if ($bl[$i] -match '^([0-9a-f]{40}) \d+ (\d+)') { $ln = [int]$Matches[2]; $sha[$ln] = $Matches[1] }
    }
    $lines = Get-Content $g.Name -Encoding UTF8
    foreach ($h in $g.Group) {
        $i = $h.LineNumber - 1; $st = 'ENUM-VARIANT'; $dv = ''
        for ($j = $i; $j -ge [Math]::Max(0, $i - 30); $j--) {
            if ($lines[$j] -match '^\s*(pub )?struct\s+(\w+)') {
                $st = $Matches[2]
                for ($k = $j; $k -ge [Math]::Max(0, $j - 8); $k--) {
                    if ($lines[$k] -match 'derive\((.*)\)') { $dv = $Matches[1]; break }
                }
                break
            }
            if ($lines[$j] -match '^\s*(pub\s+)?(async\s+)?fn\s') { $st = '<fn-param>'; break }
        }
        $fld = ($h.Line.Trim() -replace '^pub\s+','' -replace '\s*:.*$','')
        $found += [pscustomobject]@{ File=$rel; Struct=$st; Field=$fld; Line=$h.LineNumber; Sha=$sha[$h.LineNumber]; Derive=$dv }
    }
}

# Partition by ANCESTRY against the arc-close commit, never by author date:
# 7ed4e30 landed at 12:24:02 on 2026-05-29 and 36 in-arc slots carry that same date,
# so a day-granularity test cannot separate them (J-659).
$uniq = $found.Sha | Sort-Object -Unique
$anc = @{}
foreach ($c in $uniq) { git merge-base --is-ancestor $c $ArcClose 2>$null; $anc[$c] = ($LASTEXITCODE -eq 0) }
$post = $found | Where-Object { -not $anc[$_.Sha] -and $_.Struct -ne '<fn-param>' -and $_.Derive -notmatch 'clap' }

$manLines = Get-Content $manPath -Encoding UTF8 | Where-Object { $_ -and -not $_.StartsWith('#') }
$manKeys = @{}
$manRows = @()
foreach ($l in $manLines) {
    $p = $l -split "`t"
    if ($p.Count -ge 4) { $manKeys["$($p[1])|$($p[2])|$($p[3])"] = $p[0]; $manRows += [pscustomobject]@{Class=$p[0];Key="$($p[1])|$($p[2])|$($p[3])"} }
}

$expLine = (Get-Content $manPath -Encoding UTF8 | Where-Object { $_ -match '^# EXPECTED:' })
$expected = @{}
if ($expLine -match 'BOUNDARY=(\d+)\s+DESCRIPTIVE=(\d+)\s+INTERNAL=(\d+)\s+UNREAD=(\d+)\s+TOTAL=(\d+)') {
    $expected = @{ BOUNDARY=[int]$Matches[1]; DESCRIPTIVE=[int]$Matches[2]; INTERNAL=[int]$Matches[3]; UNREAD=[int]$Matches[4]; TOTAL=[int]$Matches[5] }
}

$fail = 0

# Check 1 - the manifest must reconcile against its own EXPECTED header. This is what
# stops the manifest and section 3a drifting apart, which is this arc's failure mode.
$tally = $manRows | Group-Object Class
foreach ($k in 'BOUNDARY','DESCRIPTIVE','INTERNAL','UNREAD') {
    $got = ($tally | Where-Object Name -eq $k | Select-Object -First 1).Count
    if (-not $got) { $got = 0 }
    if ($expected.Count -and $got -ne $expected[$k]) {
        Write-Host "GATE FAIL: manifest tally $k = $got but EXPECTED header says $($expected[$k])." -ForegroundColor Red
        $fail = 1
    }
}
if ($expected.Count -and $manRows.Count -ne $expected.TOTAL) {
    Write-Host "GATE FAIL: manifest has $($manRows.Count) rows but EXPECTED header says $($expected.TOTAL)." -ForegroundColor Red
    $fail = 1
}

# Check 2 - any post-arc identifier slot not on the manifest.
$new = @()
foreach ($r in $post) {
    $key = "$($r.File)|$($r.Struct)|$($r.Field)"
    if (-not $manKeys.ContainsKey($key)) { $new += $r }
}
if ($new.Count) {
    Write-Host "GATE FAIL: $($new.Count) post-arc identifier slot(s) not on the manifest." -ForegroundColor Red
    Write-Host "Classify each as BOUNDARY / DESCRIPTIVE / INTERNAL per D-137, then add it." -ForegroundColor Red
    $new | Sort-Object File,Line | ForEach-Object { Write-Host ("  {0}:{1}  {2}.{3}" -f $_.File, $_.Line, $_.Struct, $_.Field) }
    $fail = 1
}

# Check 3 - manifest rows no longer present. Expected during Legs B/C. Never silent.
$foundKeys = @{}
foreach ($r in $post) { $foundKeys["$($r.File)|$($r.Struct)|$($r.Field)"] = $true }
$gone = $manRows | Where-Object { -not $foundKeys.ContainsKey($_.Key) }
if ($gone.Count) {
    Write-Host "GATE WARN: $($gone.Count) manifest row(s) no longer found (retyped or removed?):" -ForegroundColor Yellow
    $gone | ForEach-Object { Write-Host ("  [{0}] {1}" -f $_.Class, ($_.Key -replace '\|','  ')) }
    Write-Host "If a retype landed, drop the row and update the EXPECTED header in the same commit." -ForegroundColor Yellow
}

if ($fail) { Write-Host "`nXGID SLOT GATE: FAIL" -ForegroundColor Red; exit 1 }
Write-Host ("`nXGID SLOT GATE: PASS - {0} post-arc slots, all on the manifest ({1} BOUNDARY / {2} DESCRIPTIVE / {3} INTERNAL / {4} UNREAD)." -f $post.Count, $expected.BOUNDARY, $expected.DESCRIPTIVE, $expected.INTERNAL, $expected.UNREAD) -ForegroundColor Green
exit 0