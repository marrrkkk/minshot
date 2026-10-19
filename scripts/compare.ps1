# Compare two PNGs pixel by pixel.
# Usage: powershell -File scripts/compare.ps1 -A a.png -B b.png
param([string]$A, [string]$B)

Add-Type -AssemblyName System.Drawing
$ba = [System.Drawing.Bitmap]::FromFile((Resolve-Path $A))
$bb = [System.Drawing.Bitmap]::FromFile((Resolve-Path $B))
"a: $($ba.Width)x$($ba.Height)   b: $($bb.Width)x$($bb.Height)"
$w = [Math]::Min($ba.Width, $bb.Width)
$h = [Math]::Min($ba.Height, $bb.Height)
$diff = 0
$maxDiff = 0
$sumA = 0.0
$sumB = 0.0
for ($y = 0; $y -lt $h; $y += 1) {
  for ($x = 0; $x -lt $w; $x += 1) {
    $pa = $ba.GetPixel($x, $y)
    $pb = $bb.GetPixel($x, $y)
    $d = [Math]::Abs($pa.R - $pb.R) + [Math]::Abs($pa.G - $pb.G) + [Math]::Abs($pa.B - $pb.B)
    if ($d -gt 0) { $diff += 1 }
    if ($d -gt $maxDiff) { $maxDiff = $d }
    $sumA += $pa.R + $pa.G + $pa.B
    $sumB += $pb.R + $pb.G + $pb.B
  }
}
"pixels compared: $($w * $h)"
"pixels that differ: $diff"
"largest difference (r+g+b): $maxDiff"
"mean brightness a: $([math]::Round($sumA / ($w * $h * 3), 2))  b: $([math]::Round($sumB / ($w * $h * 3), 2))"
$ba.Dispose()
$bb.Dispose()
