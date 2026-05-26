#define MyAppName "Entropy"
#ifndef MyAppVersion
#define MyAppVersion "1.12.93"
#endif
#define MyAppPublisher "Ergohaven"
#define MyAppURL "https://github.com/ergohaven/entropy"
#define MyAppExeName "entropy.exe"

[Setup]
AppId={{F1E55C49-6BE0-4E42-A918-4F2A689E2B5B}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={autopf}\Ergohaven\Entropy
DefaultGroupName=Entropy
DisableProgramGroupPage=yes
LicenseFile=..\LICENSE
OutputDir=..\dist
OutputBaseFilename=entropy-setup-{#MyAppVersion}
SetupIconFile=..\assets\entropy.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\target\x86_64-pc-windows-gnu\release\entropy.exe"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion

[Icons]
Name: "{group}\Entropy"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\Entropy"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
