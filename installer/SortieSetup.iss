; Sortie Inno Setup Wizard Script
; Generates Sortie-v0.1.0-Setup.exe installer with automatic dependencies (VC++ 2015-2022 Redistributable x64) check & installation.

[Setup]
AppName=Sortie
AppVersion=0.1.0
AppPublisher=photonsX
AppPublisherURL=https://github.com/photonsX/Sortie
AppSupportURL=https://github.com/photonsX/Sortie/issues
AppUpdatesURL=https://github.com/photonsX/Sortie/releases
DefaultDirName={autopf}\Sortie
DefaultGroupName=Sortie
AllowNoIcons=yes
OutputDir=..\target\installer
OutputBaseFilename=Sortie-v0.1.0-Setup
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
; Source binary from Cargo release build
Source: "..\target\release\sortie.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Sortie"; Filename: "{app}\sortie.exe"
Name: "{group}\Uninstall Sortie"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Sortie"; Filename: "{app}\sortie.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\sortie.exe"; Description: "{cm:LaunchProgram,Sortie}"; Flags: nowait postinstall skipifsilent

[Code]
var
  VCRedistPage: TOutputProgressWizardPage;

function IsVCRedistInstalled(): Boolean;
var
  Installed: Cardinal;
begin
  Result := False;
  // Check Visual C++ 2015-2022 Redistributable x64 registry key
  if RegQueryDWordValue(HKEY_LOCAL_MACHINE, 'SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64', 'Installed', Installed) then
  begin
    if Installed = 1 then
      Result := True;
  end;
end;

function DownloadAndInstallVCRedist(): Boolean;
var
  DownloadUrl, TempPath: String;
  ResultCode: Integer;
begin
  Result := True;
  if IsVCRedistInstalled() then
    Exit;

  DownloadUrl := 'https://aka.ms/vs/17/release/vc_redist.x64.exe';
  TempPath := ExpandConstant('{tmp}\vc_redist.x64.exe');

  VCRedistPage := CreateOutputProgressPage('Installing Dependencies', 'Downloading Visual C++ 2015-2022 Redistributable (x64)...');
  VCRedistPage.Show;
  try
    if idpDownloadFile(DownloadUrl, TempPath) then
    begin
      VCRedistPage.SetText('Installing Visual C++ Redistributable...', 'Please wait...');
      if Exec(TempPath, '/install /quiet /norestart', '', SW_SHOW, ewWaitUntilTerminated, ResultCode) then
      begin
        if (ResultCode = 0) or (ResultCode = 1638) or (ResultCode = 3010) then
          Result := True
        else
          MsgBox('Visual C++ Redistributable installation finished with code ' + IntToStr(ResultCode) + '. Sortie may require manual VC++ runtime setup.', mbInformation, MB_OK);
      end else begin
        MsgBox('Failed to run Visual C++ Redistributable installer.', mbError, MB_OK);
      end;
    end else begin
      MsgBox('Could not download Visual C++ Redistributable. Please check internet connection if Sortie fails to launch.', mbInformation, MB_OK);
    end;
  finally
    VCRedistPage.Hide;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssInstall then
  begin
    // Check and install VC++ dependency before copying files
    DownloadAndInstallVCRedist();
  end;
end;
