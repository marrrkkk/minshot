# Capture the true composited screen (BitBlt + CAPTUREBLT, so transparent
# layered windows composite like the eye sees them) and report mean/std of a
# brightness grid. Usage: powershell -File scripts/snapcheck.ps1 -Out file.png
param([string]$Out = "$env:TEMP\snapcheck.png")

$csharp = @"
using System;
using System.Drawing;
using System.Runtime.InteropServices;
public static class SnapCap {
  [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr h);
  [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr h, IntPtr dc);
  [DllImport("gdi32.dll")] public static extern IntPtr CreateCompatibleDC(IntPtr dc);
  [DllImport("gdi32.dll")] public static extern IntPtr CreateCompatibleBitmap(IntPtr dc, int w, int h);
  [DllImport("gdi32.dll")] public static extern IntPtr SelectObject(IntPtr dc, IntPtr o);
  [DllImport("gdi32.dll")] public static extern bool DeleteObject(IntPtr o);
  [DllImport("gdi32.dll")] public static extern bool DeleteDC(IntPtr dc);
  [DllImport("gdi32.dll")] public static extern bool BitBlt(IntPtr d, int x, int y, int w, int h, IntPtr s, int sx, int sy, int rop);
  public static void Snap(string f) {
    IntPtr src = GetDC(IntPtr.Zero);
    IntPtr mem = CreateCompatibleDC(src);
    IntPtr hb = CreateCompatibleBitmap(src, 1920, 1080);
    SelectObject(mem, hb);
    BitBlt(mem, 0, 0, 1920, 1080, src, 0, 0, unchecked((int)0x40CC0020));
    ReleaseDC(IntPtr.Zero, src);
    using (Bitmap b = Bitmap.FromHbitmap(hb)) { b.Save(f); }
    DeleteObject(hb); DeleteDC(mem);
  }
}
"@
Add-Type -TypeDefinition $csharp -ReferencedAssemblies System.Drawing
[SnapCap]::Snap($Out)

$img = [System.Drawing.Image]::FromFile($Out)
$bmp = New-Object System.Drawing.Bitmap($img)
$sum = 0; $sum2 = 0; $n = 0
for ($x = 100; $x -lt 1920; $x += 80) {
  for ($y = 100; $y -lt 1080; $y += 80) {
    $p = $bmp.GetPixel($x, $y)
    $v = ($p.R + $p.G + $p.B) / 3
    $sum += $v; $sum2 += $v * $v; $n++
  }
}
$mean = $sum / $n
$var = $sum2 / $n - $mean * $mean
$bmp.Dispose(); $img.Dispose()
"samples=$n mean=$([math]::Round($mean,1)) std=$([math]::Round([math]::Sqrt($var),1))"
