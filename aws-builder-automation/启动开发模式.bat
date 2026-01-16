@echo off
echo ========================================
echo AWS Builder ID 自动注册系统 - 开发模式
echo ========================================
echo.
echo 正在启动开发服务器...
echo.

cd /d "%~dp0"
npm run tauri dev

pause
