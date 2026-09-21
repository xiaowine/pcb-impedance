@echo off
chcp 65001 > nul
echo [PCB 阻抗匹配系统 - Web 原型服务]
echo 正在启动 Node.js 原生零依赖代理服务...
echo 访问地址: http://localhost:3000
echo 按 Ctrl+C 可停止服务
echo.
cd /d "%~dp0"
node server.js
pause
