@echo off
chcp 65001 >nul
title EnvHub 跨平台开发环境交互式向导
cd /d "%~dp0"

echo 正在启动 EnvHub 交互式向导...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0setup_interactive.ps1"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo 执行过程中遇到异常，请右键选择“以管理员身份运行”后重试。
    pause
)
