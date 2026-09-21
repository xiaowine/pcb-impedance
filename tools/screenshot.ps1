# screenshot.ps1
Get-Process -Name "pcb-impedance-gpui" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 500

Start-Process "C:\Users\xiaow\Desktop\pcb-impedance\desktop\target\debug\pcb-impedance-gpui.exe"

Add-Type @"
  using System;
  using System.Runtime.InteropServices;
  public class Win32 {
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll", EntryPoint = "FindWindow", SetLastError = true)]
    public static extern IntPtr FindWindowByCaption(IntPtr ZeroOnly, string lpWindowName);
  }
"@

$hWnd = [IntPtr]::Zero
for ($i = 0; $i -lt 15; $i++) {
  Start-Sleep -Milliseconds 500
  $hWnd = [Win32]::FindWindowByCaption([IntPtr]::Zero, "PCB 阻抗匹配与叠层设计系统 (GPUI 原生极速版)")
  if ($hWnd -ne [IntPtr]::Zero) {
    break
  }
}

if ($hWnd -ne [IntPtr]::Zero) {
  [Win32]::ShowWindow($hWnd, 9) # SW_RESTORE
  [Win32]::SetForegroundWindow($hWnd)
  Start-Sleep -Seconds 1
}

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bitmap = New-Object System.Drawing.Bitmap $screen.Width, $screen.Height
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($screen.Location, [System.Drawing.Point]::Empty, $screen.Size)
$outputPath = "C:\Users\xiaow\Desktop\pcb-impedance\tools\gpui_screenshot.png"
$bitmap.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose()
$bitmap.Dispose()

Write-Output "Found hWnd: $hWnd, screenshot saved to $outputPath"
