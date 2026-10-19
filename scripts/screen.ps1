# Take a picture of the whole screen, the way the eye sees it.
# Usage: powershell -File scripts/screen.ps1 -Out file.png
param([string]$Out = "$env:TEMP\screen.png")

Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @"
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
public static class ScreenCap {
  [DllImport("user32.dll")] public static extern IntPtr GetDesktopWindow();
  [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr h);
  [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr h, IntPtr dc);
  [DllImport("gdi32.dll")] public static extern IntPtr CreateCompatibleDC(IntPtr dc);
  [DllImport("gdi32.dll")] public static extern IntPtr CreateCompatibleBitmap(IntPtr dc, int w, int h);
  [DllImport("gdi32.dll")] public static extern IntPtr SelectObject(IntPtr dc, IntPtr o);
  [DllImport("gdi32.dll")] public static extern bool DeleteObject(IntPtr o);
  [DllImport("gdi32.dll")] public static extern bool DeleteDC(IntPtr dc);
  [DllImport("gdi32.dll")] public static extern bool BitBlt(IntPtr d, int x, int y, int w, int h, IntPtr s, int sx, int sy, int rop);
  public static void Shot(string path) {
    // CAPTUREBLT (0x40000000) so layered/transparent windows are included,
    // the way the eye sees them.
    IntPtr src = GetDC(GetDesktopWindow());
    int w = 1920, h = 1080;
    IntPtr mem = CreateCompatibleDC(src);
    IntPtr hb = CreateCompatibleBitmap(src, w, h);
    SelectObject(mem, hb);
    BitBlt(mem, 0, 0, w, h, src, 0, 0, unchecked((int)0x40CC0020));
    ReleaseDC(GetDesktopWindow(), src);
    using (Bitmap bmp = new Bitmap(w, h, PixelFormat.Format32bppArgb)) {
      using (Graphics g = Graphics.FromImage(bmp)) {
        IntPtr hdc = g.GetHdc();
        BitBlt(hdc, 0, 0, w, h, mem, 0, 0, 0x00CC0020);
        g.ReleaseHdc(hdc);
      }
      bmp.Save(path, ImageFormat.Png);
    }
    DeleteObject(hb); DeleteDC(mem);
  }
}
"@ -ReferencedAssemblies System.Drawing
[ScreenCap]::Shot($Out)
Write-Output "wrote $Out"
