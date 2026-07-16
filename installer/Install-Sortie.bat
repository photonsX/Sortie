@echo off
:: Sortie Universal Installer Bootstrap
:: Double-click this batch file to automatically install Sortie and all required dependencies.
title Sortie Setup Installer
echo Starting Sortie Universal Setup...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0Install-Sortie.ps1"
if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Installation encountered an issue. Please check the logs above.
    pause
)
