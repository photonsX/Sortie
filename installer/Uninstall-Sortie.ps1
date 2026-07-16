# Sortie Universal PowerShell Uninstaller
# Safely closes Sortie if running, deletes application binaries, removes shortcuts,
# and asks whether to remove saved user configuration (tiles & projects).

$ErrorActionPreference = "Continue"
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "            SORTIE UNIVERSAL UNINSTALLER                  " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Close Sortie if currently running
Write-Host "`n[1/4] Checking if Sortie is currently running..." -ForegroundColor Yellow
$running = Get-Process -Name "sortie" -ErrorAction SilentlyContinue
if ($running) {
    Write-Host "  -> Sortie is currently running. Closing application safely..." -ForegroundColor Magenta
    Stop-Process -Name "sortie" -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
    Write-Host "  -> Closed Sortie process [OK]" -ForegroundColor Green
} else {
    Write-Host "  -> Sortie is not running [OK]" -ForegroundColor Green
}

# 2. Remove Shortcuts (Desktop & Start Menu)
Write-Host "`n[2/4] Removing Windows Shortcuts..." -ForegroundColor Yellow

# Desktop Shortcut
$desktopLnk1 = Join-Path ([System.Environment]::GetFolderPath("Desktop")) "Sortie.lnk"
if (Test-Path $desktopLnk1) {
    Remove-Item -Path $desktopLnk1 -Force
    Write-Host "  -> Removed Desktop shortcut: $desktopLnk1 [OK]" -ForegroundColor Green
}

# OneDrive Desktop Shortcut (if applicable)
$oneDriveDesktop = Join-Path $env:USERPROFILE "OneDrive\Desktop\Sortie.lnk"
if (Test-Path $oneDriveDesktop) {
    Remove-Item -Path $oneDriveDesktop -Force
    Write-Host "  -> Removed OneDrive Desktop shortcut: $oneDriveDesktop [OK]" -ForegroundColor Green
}

# Start Menu Shortcut Folder
$startMenuDir = Join-Path ([System.Environment]::GetFolderPath("Programs")) "Sortie"
if (Test-Path $startMenuDir) {
    Remove-Item -Path $startMenuDir -Recurse -Force
    Write-Host "  -> Removed Start Menu folder: $startMenuDir [OK]" -ForegroundColor Green
}

# 3. Remove Application Binaries
Write-Host "`n[3/4] Removing Application Directory & Binaries..." -ForegroundColor Yellow
$installDir = Join-Path $env:LOCALAPPDATA "Programs\Sortie"
if (Test-Path $installDir) {
    Remove-Item -Path $installDir -Recurse -Force
    Write-Host "  -> Removed directory: $installDir [OK]" -ForegroundColor Green
} else {
    Write-Host "  -> Application directory not found ($installDir) [Skipped]" -ForegroundColor DarkGray
}

# 4. Optional Removal of Saved Configuration & Tiles (`state.json`)
Write-Host "`n[4/4] Saved User Configuration (Tiles & Projects)..." -ForegroundColor Yellow
$configDir = Join-Path $env:APPDATA "Sortie"
if (Test-Path $configDir) {
    Write-Host "`nWe found your saved tiles and project settings at:" -ForegroundColor Cyan
    Write-Host "  $configDir" -ForegroundColor White
    $response = Read-Host "`nDo you want to permanently delete your saved tiles & project configuration as well? (Y/N)"
    if ($response -eq 'Y' -or $response -eq 'y') {
        Remove-Item -Path $configDir -Recurse -Force
        Write-Host "  -> Deleted saved configuration directory [OK]" -ForegroundColor Green
    } else {
        Write-Host "  -> Kept saved configuration! (If you reinstall Sortie later, your tiles will return automatically) [OK]" -ForegroundColor Green
    }
} else {
    Write-Host "  -> No saved configuration directory found [OK]" -ForegroundColor Green
}

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host "       SORTIE HAS BEEN SUCCESSFULLY UNINSTALLED!          " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "`nYou can safely close this window now.`n" -ForegroundColor White
Read-Host "Press Enter to exit..." | Out-Null
