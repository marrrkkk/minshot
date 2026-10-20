# Look for a bright line on the edges of a captured PNG: the selection
# outline must not be in the picture.
# Usage: powershell -File scripts\edges.ps1 shot.png
param([string]$File)

Add-Type -AssemblyName System.Drawing
$bmp = [System.Drawing.Bitmap]::FromFile((Resolve-Path $File))
"image: $($bmp.Width)x$($bmp.Height)"

function Brightness($b) { ($b.R + $b.G + $b.B) / 3 }

# The first and last row and column of the image.
$top = @(); $bottom = @(); $left = @(); $right = @()
for ($x = 0; $x -lt $bmp.Width; $x++) {
  $top += Brightness $bmp.GetPixel($x, 0)
  $bottom += Brightness $bmp.GetPixel($x, ($bmp.Height - 1))
}
for ($y = 0; $y -lt $bmp.Height; $y++) {
  $left += Brightness $bmp.GetPixel(0, $y)
  $right += Brightness $bmp.GetPixel(($bmp.Width - 1), $y)
}
function Stat($name, $values) {
  $avg = ($values | Measure-Object -Average).Average
  $max = ($values | Measure-Object -Maximum).Maximum
  "{0,-8} mean={1,7:N1}  max={2,7:N1}" -f $name, $avg, $max
}
Stat "top" $top
Stat "bottom" $bottom
Stat "left" $left
Stat "right" $right

# The middle of the image, for comparison: what the screen looks like.
$mid = @()
for ($y = [int]($bmp.Height / 2) - 5; $y -lt [int]($bmp.Height / 2) + 5; $y++) {
  for ($x = [int]($bmp.Width / 2) - 5; $x -lt [int]($bmp.Width / 2) + 5; $x++) {
    $mid += Brightness $bmp.GetPixel($x, $y)
  }
}
Stat "middle" $mid
$bmp.Dispose()
