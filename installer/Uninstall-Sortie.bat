@echo off
:: Sortie Universal Uninstaller Bootstrap
:: Double-click this batch file to automatically uninstall Sortie and remove shortcuts.
title Sortie Uninstaller
echo Starting Sortie Uninstaller...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0Uninstall-Sortie.ps1"
