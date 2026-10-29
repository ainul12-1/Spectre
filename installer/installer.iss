; Inno Setup Script for Spectre Stealth Browser
; Generates Spectre-Windows-Setup.exe (Single executable installer & launcher)

#define MyAppName "Spectre"
#define MyAppVersion "1.0.10"
#define MyAppPublisher "Ainul"
#define MyAppURL "https://github.com/ainul12-1/Spectre"
#define MyAppExeName "Spectre.exe"

[Setup]
AppId={{D3E8B541-692B-4F8A-A5E3-9118FC5E8D21}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={userappdata}\Spectre
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputBaseFilename=Spectre-Windows-Setup
OutputDir=.
Compression=lzma2/max
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64
WizardStyle=modern
PrivilegesRequired=lowest
SetupIconFile=..\src-tauri\icons\icon.ico
VersionInfoVersion=1.0.10.0
VersionInfoCompany=Ainul
VersionInfoDescription=Spectre Stealth Safari Browser
VersionInfoCopyright=Copyright (C) 2024 Ainul
VersionInfoProductName=Spectre
VersionInfoProductVersion=1.0.10.0
UninstallDisplayName={#MyAppName}
CloseApplications=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\src-tauri\target\release\spectre.exe"; DestDir: "{app}"; DestName: "Spectre.exe"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
