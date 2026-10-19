# Watch the minshot window from the moment the process starts: when does it
# appear, and does it ever move afterwards?
# Usage: powershell -File scripts/startup.ps1
param([string]$Exe = "src-tauri\target\release\minshot.exe")

. "$PSScriptRoot\w32.ps1"

$proc = Start-Process -FilePath $Exe -PassThru
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$seen = @()
$last = ""
while ($sw.ElapsedMilliseconds -lt 4000) {
  foreach ($line in [W32]::All()) {
    if ($line -notmatch "\|$($proc.Id)\|") { continue }
    $parts = $line -split '\|'
    $key = "$($parts[2]) $($parts[4])"
    if ($key -ne $last) {
      $last = $key
      $seen += "{0,6} ms  visible={1}  rect={2}" -f $sw.ElapsedMilliseconds, $parts[2], $parts[4]
    }
  }
  Start-Sleep -Milliseconds 10
}
$seen | ForEach-Object { $_ }
$proc | Stop-Process -Force
