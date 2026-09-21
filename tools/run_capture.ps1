# run_capture.ps1
$env:RUST_BACKTRACE = "1"
& "C:\Users\xiaow\Desktop\pcb-impedance-gpui\target\debug\pcb-impedance-gpui.exe" 2>&1 | Out-File -FilePath "C:\Users\xiaow\Desktop\gpui_error.log" -Encoding utf8
