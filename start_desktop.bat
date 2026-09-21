@echo off
chcp 65001 > nul
echo [PCB 阻抗匹配系统 - GPUI 原生极速版]
echo 正在启动 GPU 硬件加速桌面应用...
cd /d "%~dp0desktop"
if exist "target\debug\pcb-impedance-gpui.exe" (
    start "" "target\debug\pcb-impedance-gpui.exe"
) else (
    echo 未找到预编译二进制文件，正在自动编译并启动...
    cargo run
)
