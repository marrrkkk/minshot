Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$form = New-Object System.Windows.Forms.Form
$form.Text = "DROP-PROBE"
$form.Size = New-Object System.Drawing.Size(500, 400)
$form.StartPosition = "Manual"
$form.Location = New-Object System.Drawing.Point(1100, 300)
$form.TopMost = $true
$form.AllowDrop = $true
$log = "C:\Users\Mark\AppData\Local\Temp\drop_probe.log"
"probe started" | Out-File $log
$form.Add_DragEnter({ param($s, $e)
  $fmts = ($e.Data.GetFormats() -join ",")
  "ENTER formats=$fmts effect=$($e.Effect)" | Out-File $log -Append
  if ($e.Data.GetDataPresent([System.Windows.Forms.DataFormats]::FileDrop)) { $e.Effect = "Copy" } else { $e.Effect = "None" }
})
$form.Add_DragDrop({ param($s, $e)
  $files = @()
  if ($e.Data.GetDataPresent([System.Windows.Forms.DataFormats]::FileDrop)) { $files = $e.Data.GetData([System.Windows.Forms.DataFormats]::FileDrop) }
  $fmts = ($e.Data.GetFormats() -join ",")
  "DROP files=$($files -join ';') formats=$fmts" | Out-File $log -Append
})
$form.Add_Shown({ $form.Activate() })
[void]$form.ShowDialog()
