# Run several captures against the live app and save the window after each.
# Usage: powershell -File scripts/cases.ps1
param([string]$Dir = "C:\Users\Mark\AppData\Local\Temp\opencode\cases")

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

New-Item -ItemType Directory -Force -Path $Dir | Out-Null
$proc = Get-Process minshot -ErrorAction Stop | Select-Object -First 1

function Find-Rect([int]$h) {
  foreach ($line in [W32]::All()) {
    if ($line -match "^\s*(\d+)\|") { if ([int]$Matches[1] -eq $h) { return (MsRect $line) } }
  }
  return $null
}

# name, x0, y0, x1, y1
$cases = @(
  @("wide",   20,  40, 920, 140),
  @("portrait", 300, 100, 700, 900),
  @("small",  100, 100, 220, 170),
  @("full",   0,   42, 1920, 1080)
)

foreach ($c in $cases) {
  $name = $c[0]
  [Keys]::Combo(0x11, 0x10, 0x34)
  Start-Sleep -Milliseconds 1200
  [W32]::Drag([int]$c[1], [int]$c[2], [int]$c[3], [int]$c[4])
  Start-Sleep -Milliseconds 2500
  $proc.Refresh()
  $h = $proc.MainWindowHandle
  $r = Find-Rect $h
  "$name -> rect=$($r -join ',')"
  & powershell -File "$PSScriptRoot\windowshot.ps1" -Hwnd $h -Out "$Dir\$name.png" | Out-Null
}
"done"
