# Independent transparency probe: fullscreen 30%-alpha black window via WinForms.
# If DWM compositing is sane, screen mean should read ~0.7x of baseline.
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$form = New-Object System.Windows.Forms.Form
$form.Text = "DIM-PROBE"
$form.FormBorderStyle = "None"
$form.StartPosition = "Manual"
$form.Location = New-Object System.Drawing.Point(0, 0)
$form.Size = New-Object System.Drawing.Size(1920, 1080)
$form.BackColor = [System.Drawing.Color]::Black
$form.Opacity = 0.3
$form.TopMost = $true
$form.Show()
"probe up" | Out-File C:\Users\Mark\AppData\Local\Temp\dimprobe.log
[System.Threading.Thread]::Sleep(15000)
$form.Close()
"probe down" | Out-File C:\Users\Mark\AppData\Local\Temp\dimprobe.log -Append
