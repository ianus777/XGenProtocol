# roadmap-format-gate.ps1 - the standing guard D-136 says a sweep never leaves behind.
#
# J-662 deleted 261 unofficial icons and 62 prose state symbols. J-663 deleted 168
# narrative annotation lines (61 KB) after the file regrew 68,890 -> 119,889 bytes in
# eight days. BOTH WERE SWEEPS, NOT GUARDS - and the tree regrew again across five
# commits between J-675 and J-682 because nothing checked a line at write time.
#
# Run before every commit that touches docs/ROADMAP.md. Non-zero exit = do not commit.
#
# Rules enforced, all quoted from the file's own "Symbol discipline" and
# "How to read this view" sections:
#   R-ICON  L38  the six legend symbols and nothing else
#   R-POS   L40  state symbols on NODE lines only; a a-annotation line carries none
#   R-ANN   L402 a-annotation lines admit ONLY "trigger:" and "Owes:"
#   R-LEN   L411 node description under ~160 chars (R-5a: the link chain is exempt)

param([string]$Path = 'docs/ROADMAP.md')

$ErrorActionPreference = 'Stop'
$lines = [System.IO.File]::ReadAllLines((Resolve-Path $Path))

# Tree fence: only the nested view is governed.
$start = 0; $end = 0
for ($i = 0; $i -lt $lines.Count; $i++) {
  if ($lines[$i] -match '^## Visual structure') { $start = $i }
  if ($start -gt 0 -and $lines[$i] -match '^### How to read this view') { $end = $i; break }
}
if ($start -eq 0 -or $end -eq 0) { Write-Host 'GATE ERROR: tree fence not found'; exit 2 }

$ARROW  = [char]0x21B3
$GLYPHS = "$([char]0x2502)$([char]0x2500)$([char]0x251C)$([char]0x2514)"
# The six, built via surrogate pairs - [char]0x1F7E2 overflows char and corrupts silently.
$SIX = @(
  [string][char]0x2705,                      # DONE
  [string][char]0x274C,                      # CANCELLED
  [string][char]0x2B1B,                      # DEPRECATED
  [string][char]0x23F8,                      # POSTPONED
  ([char]0xD83D + [char]0xDFE2),             # PLAY
  ([char]0xD83D + [char]0xDFE1)              # PENDING
)

$fail = @()

for ($i = $start; $i -lt $end; $i++) {
  $n = $i + 1
  $s = $lines[$i]
  if ($s -notmatch "[$GLYPHS]") { continue }

  $stripped = $s -replace "^[\s$GLYPHS]*", ''
  $isAnnotation = $stripped.StartsWith($ARROW)

  # R-ICON - any pictographic codepoint outside the six. Enumeration marks and
  # arrows are punctuation per L42 and are NOT flagged.
  for ($j = 0; $j -lt $s.Length; $j++) {
    $c = [int]$s[$j]; $cp = $c
    if ($c -ge 0xD800 -and $c -le 0xDBFF -and $j + 1 -lt $s.Length) {
      $cp = 0x10000 + (($c - 0xD800) -shl 10) + ([int]$s[$j + 1] - 0xDC00); $j++
    }
    if ($cp -eq 0xFE0F) { continue }
    $pictographic = ($cp -ge 0x1F300 -and $cp -le 0x1FAFF) -or ($cp -ge 0x2600 -and $cp -le 0x27BF) -or $cp -eq 0x2B1B -or $cp -eq 0x23F8
    if (-not $pictographic) { continue }
    $ch = if ($cp -gt 0xFFFF) { [char]::ConvertFromUtf32($cp) } else { [string][char]$cp }
    if ($SIX -notcontains $ch) { $fail += "R-ICON  L$n  illegal icon U+{0:X4} - write it as words (L41)" -f $cp }
  }

  if ($isAnnotation) {
    # R-POS - a state symbol on an annotation line reads as a state that does not exist.
    foreach ($sym in $SIX) {
      if ($s.Contains($sym)) { $fail += "R-POS   L$n  state symbol on an annotation line (L40)"; break }
    }
    # R-ANN - only two annotations are admitted.
    $body = $stripped.Substring(1).TrimStart()
    if ($body -notmatch '^(\*\*)?`?(trigger:|Owes:)') {
      $fail += "R-ANN   L$n  narrative annotation - only 'trigger:' and 'Owes:' are legal (L402); this belongs in JOURNAL.md behind the node's J-nnn"
    }
  }
  else {
    # R-LEN - description only, link chain exempt (R-5a). The chain starts at the
    # first middot; measure what precedes it.
    $desc = ($stripped -split [char]0x00B7)[0]
    if ($desc.Length -gt 200) { $fail += "R-LEN   L$n  node description $($desc.Length) chars, bound is ~160 (R-5)" }
  }
}

if ($fail.Count -eq 0) {
  Write-Host "ROADMAP FORMAT GATE: PASS - tree lines $($start + 2)..$end clean"
  exit 0
}
Write-Host "ROADMAP FORMAT GATE: FAIL - $($fail.Count) violation(s)"
$fail | ForEach-Object { Write-Host "  $_" }
exit 1
