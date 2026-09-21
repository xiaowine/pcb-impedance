# capture_top.ps1
Add-Type @"
  using System;
  using System.Runtime.InteropServices;
  public class Win32 {
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")]
    public static extern bool SetProcessDPIAware();
  }
"@

[Win32]::SetProcessDPIAware()

$shell = New-Object -ComObject "Shell.Application"
$shell.MinimizeAll()
Start-Sleep -Milliseconds 600

$proc = Get-Process -Name "pcb-impedance-gpui" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($proc -and $proc.MainWindowHandle -ne [IntPtr]::Zero) {
  Write-Output "Found Process HWND: $($proc.MainWindowHandle)"
  [Win32]::ShowWindow($proc.MainWindowHandle, 9) # SW_RESTORE
  [Win32]::SetForegroundWindow($proc.MainWindowHandle)
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

Write-Output "Clean capture complete to $outputPath ($($screen.Width)x$($screen.Height))"
