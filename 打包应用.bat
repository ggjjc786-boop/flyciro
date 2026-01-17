@echo off
chcp 65001 >nul
echo ========================================
echo   Unified Account Manager - 打包应用
echo ========================================
echo.
echo 正在打包应用程序...
echo 这可能需要几分钟时间，请耐心等待...
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
echo 开始打包...
echo.
call npm run tauri build

if errorlevel 1 (
    echo.
    echo 打包失败！请查看上方错误信息。
    pause
    exit /b 1
)

echo.
echo ========================================
echo   打包完成！
echo ========================================
echo.
echo 安装包位置:
echo   MSI: src-tauri\target\release\bundle\msi\
echo   NSIS: src-tauri\target\release\bundle\nsis\
echo.
pause
