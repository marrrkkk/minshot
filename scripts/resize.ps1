# Resize the minshot window and screenshot it, to check the preview follows.
# Usage: powershell -File scripts\resize.ps1 -Width 900 -Height 400 -Out file.png
param([int]$Width = 900, [int]$Height = 400, [string]$Out = "$env:TEMP\minshot-resized.png")

. "$PSScriptRoot\w32.ps1"

$c = @"
using System; using System.Runtime.InteropServices;
public static class M {
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hgt, bool repaint);
}
"@
Add-Type -TypeDefinition $c

$proc = Get-Process minshot -ErrorAction Stop | Select-Object -First 1
$hwnd = $proc.MainWindowHandle
[M]::MoveWindow([IntPtr]$hwnd, 200, 200, $Width, $Height, $true) | Out-Null
Start-Sleep -Seconds 2
foreach ($line in [W32]::All()) { if ($line -match "minshot\|") { "  $_" } }
& powershell -File "$PSScriptRoot\windowshot.ps1" -Hwnd $hwnd -Out $Out
