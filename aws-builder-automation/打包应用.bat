@echo off
echo ========================================
echo AWS Builder ID 自动注册系统 - 构建打包
echo ========================================
echo.
echo 正在构建桌面应用程序...
echo 这可能需要几分钟时间...
echo.

cd /d "%~dp0"
npm run tauri build

echo.
echo ========================================
echo 构建完成！
echo ========================================
echo.
echo 安装包位置：
echo MSI:  src-tauri\target\release\bundle\msi\
echo NSIS: src-tauri\target\release\bundle\nsis\
echo.

pause
