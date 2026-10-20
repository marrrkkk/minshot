# Check the overlay: capture a region, then open the overlay again and see
# whether the rectangle of the last shot is still on the screen.
# Usage: powershell -File scripts\overlay-check.ps1
param([string]$Exe = "src-tauri\target\release\minshot.exe")

. "$PSScriptRoot\w32.ps1"

$fg = @"
using System;
using System.Runtime.InteropServices;
public static class Fg {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
}
"@
Add-Type -TypeDefinition $fg

# The click only happens when our own window really is in front, so the
# test never clicks on whatever the person is doing.
function Click-Ours([int]$x, [int]$y) {
  $h = [Fg]::GetForegroundWindow()
  if ($h -ne [IntPtr]$script:hwnd) {
    "  (skipped: another window is in front)"
    return $false
  }
  [W32]::Click($x, $y)
  return $true
}

$proc = Start-Process -FilePath $Exe -PassThru
"started: $Exe  pid=$($proc.Id)"
$line = $null
for ($i = 0; $i -lt 40 -and -not $line; $i++) {
  Start-Sleep -Milliseconds 250
  $line = [W32]::All() | Where-Object { $_ -match "\|$($proc.Id)\|V\|minshot" } | Select-Object -First 1
}
if (-not $line) {
  "windows of that process:"
  [W32]::All() | Where-Object { $_ -match "minshot" } | ForEach-Object { "  $_" }
  "the launcher never showed up"
  $proc | Stop-Process -Force
  exit
}
$script:hwnd = [int](($line -split '\|')[0])
$rect = MsRect $line
"launcher: $line"

# 1. the overlay comes up and a region is captured
[W32]::SetForegroundWindow([IntPtr]$script:hwnd) | Out-Null
Start-Sleep -Milliseconds 700
if (-not (Click-Ours ($rect[0] + 124) ($rect[1] + 61))) { $proc | Stop-Process -Force; exit }
Start-Sleep -Seconds 2
"overlay up: $(([W32]::All() | Where-Object { $_ -match "\|$($proc.Id)\|V\|minshot overlay" }).Count -gt 0)"
[W32]::Drag(600, 200, 1000, 400)
Start-Sleep -Seconds 3
"captured: $((Get-ChildItem "$env:TEMP\minshot" -Filter *.png -ErrorAction SilentlyContinue).Count) file(s)"

# 2. the overlay comes up again: the old rectangle must be gone.
# The app is in the shot view now, so its "New capture" button is in the
# toolbar at the top right.
$line = [W32]::All() | Where-Object { $_ -match "\|$($proc.Id)\|V\|minshot" } | Select-Object -First 1
$rect = MsRect $line
[W32]::SetForegroundWindow([IntPtr]$script:hwnd) | Out-Null
Start-Sleep -Milliseconds 700
if (-not (Click-Ours ($rect[2] - 60) ($rect[1] + 51))) { $proc | Stop-Process -Force; exit }
Start-Sleep -Milliseconds 600
& powershell -File "$PSScriptRoot\screen.ps1" -Out "$env:TEMP\overlay-again.png"
"picture of the overlay the second time: $env:TEMP\overlay-again.png"
$proc | Stop-Process -Force
