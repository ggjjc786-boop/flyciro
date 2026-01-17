@echo off
chcp 65001 >nul
echo ========================================
echo   Unified Account Manager - 开发模式
echo ========================================
echo.
echo 正在启动开发服务器...
echo.

cd /d "%~dp0"

if not exist "node_modules" (
    echo 检测到未安装依赖，正在安装...
    call npm install
    if errorlevel 1 (
        echo.
        echo 依赖安装失败！请检查网络连接。
        pause
        exit /b 1
    )
)

echo.
echo 启动中...
echo.
call npm run tauri dev

pause
