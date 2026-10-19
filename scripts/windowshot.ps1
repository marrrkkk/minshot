# Capture a window's own content with PrintWindow (works for WebView2 hosts).
# Usage: powershell -File scripts/windowshot.ps1 -Hwnd 123456 -Out file.png
param([int]$Hwnd = 0, [string]$Out = "$env:TEMP\window.png")

Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @"
using System; using System.Drawing; using System.Drawing.Imaging; using System.Runtime.InteropServices;
public static class WinShot {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L,T,R,B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  public static void Capture(IntPtr hwnd, string path) {
    RECT r; GetWindowRect(hwnd, out r);
    int w = r.R - r.L, hh = r.B - r.T;
    using (Bitmap bmp = new Bitmap(w, hh, PixelFormat.Format32bppArgb))
    using (Graphics g = Graphics.FromImage(bmp)) {
      IntPtr hdc = g.GetHdc();
      PrintWindow(hwnd, hdc, 2);
      g.ReleaseHdc(hdc);
      bmp.Save(path, ImageFormat.Png);
    }
  }
}
"@ -ReferencedAssemblies System.Drawing
[WinShot]::Capture([IntPtr]$Hwnd, $Out)
Write-Output "saved $Out"