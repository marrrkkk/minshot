# Drive the running minshot: trigger a capture, drag a region, screenshot.
# Usage: powershell -File scripts/drive.ps1 -DragX0 400 -DragY0 300 -DragX1 900 -DragY1 800 -Out file.png
param(
  [int]$DragX0 = 400, [int]$DragY0 = 300, [int]$DragX1 = 900, [int]$DragY1 = 800,
  [string]$Out = "$env:TEMP\minshot-shot.png",
  [switch]$ByButton
)

. "$PSScriptRoot\w32.ps1"

$kbd = @"
using System; using System.Runtime.InteropServices;
public static class Keys {
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static void Combo(params byte[] vks) {
    foreach (var vk in vks) keybd_event(vk, 0, 0, UIntPtr.Zero);
    foreach (var vk in vks) keybd_event(vk, 0, 2, UIntPtr.Zero);
  }
}
"@
Add-Type -TypeDefinition $kbd

$proc = Get-Process minshot -ErrorAction Stop | Select-Object -First 1
$hwnd = $proc.MainWindowHandle
"hwnd=$hwnd"

function Find-Rect([int]$h) {
  foreach ($line in [W32]::All()) {
    if ($line -match "^\s*(\d+)\|") {
      if ([int]$Matches[1] -eq $h) { return (MsRect $line) }
    }
  }
  return $null
}
function Show-Windows([string]$tag) {
  "windows ($tag):"
  [W32]::All() | Where-Object { $_ -match "minshot" } | ForEach-Object { "  $_" }
}

if ($ByButton) {
  # Click the primary button (client centre of the launcher).
  $rect = Find-Rect $hwnd
  "winrect=$($rect -join ',')"
  [W32]::SetForegroundWindow($hwnd) | Out-Null
  Start-Sleep -Milliseconds 500
  [W32]::Click($rect[0] + 132, $rect[1] + 94)
} else {
  # The global shortcut.
  [Keys]::Combo(0x11, 0x10, 0x34)
}
Start-Sleep -Seconds 2
Show-Windows "after trigger"

# The overlay is up: drag the region.
[W32]::Drag($DragX0, $DragY0, $DragX1, $DragY1)
Start-Sleep -Seconds 3
Show-Windows "after drag"

$proc.Refresh()
$hwnd2 = $proc.MainWindowHandle
$outRect = Find-Rect $hwnd2
"hwnd2=$hwnd2 outrect=$($outRect -join ',')"

& powershell -File "$PSScriptRoot\windowshot.ps1" -Hwnd $hwnd2 -Out $Out
