# Sortie Universal PowerShell Installer
# Automatically verifies dependencies (Visual C++ 2015-2022 Redistributable x64),
# installs sortie.exe to local programs folder, creates Desktop & Start Menu shortcuts,
# and starts the application with a clean, empty workspace.

$ErrorActionPreference = "Stop"
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "             SORTIE UNIVERSAL SETUP INSTALLER             " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Check Visual C++ 2015-2022 Redistributable x64 Dependency
Write-Host "`n[1/4] Checking system dependencies (Visual C++ Runtime x64)..." -ForegroundColor Yellow
$vcRegPath = "HKLM:\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64"
$vcInstalled = $false
if (Test-Path $vcRegPath) {
    $val = (Get-ItemProperty -Path $vcRegPath -Name "Installed" -ErrorAction SilentlyContinue).Installed
    if ($val -eq 1) {
        $vcInstalled = $true
    }
}

if ($vcInstalled) {
    Write-Host "  -> Visual C++ 2015-2022 Redistributable x64 is already installed! [OK]" -ForegroundColor Green
} else {
    Write-Host "  -> Visual C++ Redistributable not detected. Downloading and installing..." -ForegroundColor Magenta
    $vcTemp = Join-Path $env:TEMP "vc_redist.x64.exe"
    $url = "https://aka.ms/vs/17/release/vc_redist.x64.exe"
    try {
        Invoke-WebRequest -Uri $url -OutFile $vcTemp -UseBasicParsing
        Write-Host "  -> Installing Visual C++ Redistributable (quiet mode)..." -ForegroundColor Magenta
        $proc = Start-Process -FilePath $vcTemp -ArgumentList "/install /quiet /norestart" -Wait -PassThru
        if ($proc.ExitCode -in 0, 1638, 3010) {
            Write-Host "  -> Visual C++ Redistributable installed successfully! [OK]" -ForegroundColor Green
        } else {
            Write-Warning "Visual C++ setup returned exit code $($proc.ExitCode). Continuing installation..."
        }
    } catch {
        Write-Warning "Could not download or install VC++ runtime automatically: $_. Continuing..."
    }
}

# 2. Locate sortie.exe
Write-Host "`n[2/4] Locating Sortie application binary..." -ForegroundColor Yellow
$sourceExe = ""
$possiblePaths = @(
    (Join-Path $PSScriptRoot "..\target\release\sortie.exe"),
    (Join-Path $PSScriptRoot "sortie.exe"),
    (Join-Path (Get-Location) "target\release\sortie.exe"),
    (Join-Path (Get-Location) "sortie.exe")
)

foreach ($p in $possiblePaths) {
    if (Test-Path $p) {
        $sourceExe = (Resolve-Path $p).Path
        break
    }
}

if (-not $sourceExe -or -not (Test-Path $sourceExe)) {
    Write-Host "`n[!] Could not find 'sortie.exe'." -ForegroundColor Red
    Write-Host "Please build Sortie first using 'cargo build --release' or place sortie.exe beside this script." -ForegroundColor Yellow
    exit 1
}

Write-Host "  -> Found binary at: $sourceExe [OK]" -ForegroundColor Green

# 3. Install Sortie to Local AppData Programs Directory
Write-Host "`n[3/4] Installing Sortie to local directory..." -ForegroundColor Yellow
$installDir = Join-Path $env:LOCALAPPDATA "Programs\Sortie"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$destExe = Join-Path $installDir "sortie.exe"
Copy-Item -Path $sourceExe -Destination $destExe -Force
Write-Host "  -> Installed application binary to: $destExe [OK]" -ForegroundColor Green

# 4. Create Shortcuts (Desktop & Start Menu)
Write-Host "`n[4/4] Creating Windows Shortcuts (Desktop & Start Menu)..." -ForegroundColor Yellow
$wsh = New-Object -ComObject WScript.Shell

# Desktop Shortcut
$desktopPath = [System.Environment]::GetFolderPath("Desktop")
$desktopLnk = Join-Path $desktopPath "Sortie.lnk"
$shortcut = $wsh.CreateShortcut($desktopLnk)
$shortcut.TargetPath = $destExe
$shortcut.WorkingDirectory = $installDir
$shortcut.Description = "Sortie Grid Launcher & Project Bundler"
$shortcut.Save()
Write-Host "  -> Created Desktop shortcut: $desktopLnk [OK]" -ForegroundColor Green

# Start Menu Shortcut
$startMenuDir = Join-Path ([System.Environment]::GetFolderPath("Programs")) "Sortie"
if (-not (Test-Path $startMenuDir)) {
    New-Item -ItemType Directory -Path $startMenuDir -Force | Out-Null
}
$startMenuLnk = Join-Path $startMenuDir "Sortie.lnk"
$shortcutSM = $wsh.CreateShortcut($startMenuLnk)
$shortcutSM.TargetPath = $destExe
$shortcutSM.WorkingDirectory = $installDir
$shortcutSM.Description = "Sortie Grid Launcher & Project Bundler"
$shortcutSM.Save()
Write-Host "  -> Created Start Menu shortcut: $startMenuLnk [OK]" -ForegroundColor Green

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host "      SORTIE INSTALLATION COMPLETED SUCCESSFULLY!         " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "`nLaunching Sortie now with a fresh, clean dashboard...`n" -ForegroundColor Cyan

# Launch Sortie cleanly
Start-Process -FilePath $destExe
