; ==============================================================================
; Phonon Platform - Inno Setup 6 Installation Wizard Script
; Generates: phonon-setup-{#MyAppVersion}-x64.exe
; ==============================================================================

#define MyAppName "Phonon Platform"
#ifndef MyAppVersion
  #define MyAppVersion "0.1.0"
#endif
#define MyAppPublisher "Aerovex"
#define MyAppURL "https://phonon.aerovex.net"
#define MyAppExeName "phonon.exe"

[Setup]
AppId={{9C18B47E-4F3D-4A8E-921B-2B2D19D5D7A1}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\Phonon
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=..\..\LICENSE
OutputDir=..\..\dist
OutputBaseFilename=phonon-setup-{#MyAppVersion}-x64
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
DisableProgramGroupPage=auto
ChangesEnvironment=yes
UninstallDisplayIcon={app}\{#MyAppExeName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Types]
Name: "full"; Description: "Full Installation (CLI + Desktop Studio GUI + Completions)"
Name: "compact"; Description: "Compact Installation (CLI Only)"
Name: "custom"; Description: "Custom Installation"; Flags: iscustom

[Components]
Name: "cli"; Description: "Phonon High-Throughput CLI Engine"; Types: full compact custom; Flags: fixed
Name: "gui"; Description: "Phonon Desktop CAD Studio Interface"; Types: full custom
Name: "completions"; Description: "Shell Autocompletions (PowerShell / Bash)"; Types: full custom
Name: "samples"; Description: "Example Circuit Topologies & SPICE Netlists"; Types: full custom

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Components: gui
Name: "addtopath"; Description: "Add Phonon installation directory to system PATH"; GroupDescription: "Environment Variables:"
Name: "associatefiles"; Description: "Associate .phonon and .sp circuit files with Phonon Studio"; GroupDescription: "File Associations:"

[Files]
Source: "..\..\dist\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion; Components: cli
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\completions\*"; DestDir: "{app}\completions"; Flags: ignoreversion recursesubdirs createallsubdirs; Components: completions

[Icons]
Name: "{group}\Phonon Studio"; Filename: "{app}\{#MyAppExeName}"; Parameters: "ui"; Components: gui
Name: "{group}\Phonon CLI Prompt"; Filename: "{cmd}"; Parameters: "/K ""{app}\{#MyAppExeName}"""; Components: cli
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Phonon Studio"; Filename: "{app}\{#MyAppExeName}"; Parameters: "ui"; Tasks: desktopicon; Components: gui

[Registry]
; File associations for .phonon
Root: HKA; Subkey: "Software\Classes\.phonon"; ValueType: string; ValueName: ""; ValueData: "PhononCircuitTopology"; Flags: uninsdeletevalue; Tasks: associatefiles
Root: HKA; Subkey: "Software\Classes\PhononCircuitTopology"; ValueType: string; ValueName: ""; ValueData: "Phonon Quantum Acoustic Circuit"; Flags: uninsdeletekey; Tasks: associatefiles
Root: HKA; Subkey: "Software\Classes\PhononCircuitTopology\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: associatefiles
Root: HKA; Subkey: "Software\Classes\PhononCircuitTopology\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ui ""%1"""; Tasks: associatefiles

; File associations for .sp
Root: HKA; Subkey: "Software\Classes\.sp"; ValueType: string; ValueName: ""; ValueData: "PhononSpiceNetlist"; Flags: uninsdeletevalue; Tasks: associatefiles
Root: HKA; Subkey: "Software\Classes\PhononSpiceNetlist"; ValueType: string; ValueName: ""; ValueData: "SPICE Circuit Netlist"; Flags: uninsdeletekey; Tasks: associatefiles
Root: HKA; Subkey: "Software\Classes\PhononSpiceNetlist\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" run ""%1"""; Tasks: associatefiles

[Code]
// System and User PATH Modification functions
const
  EnvironmentKey = 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';
  UserEnvironmentKey = 'Environment';

procedure AddPathToEnvironment(PathToAdd: string);
var
  Paths: string;
  RegKey: string;
  RootKey: Integer;
begin
  if IsAdminLoggedOn then
  begin
    RootKey := HKEY_LOCAL_MACHINE;
    RegKey := EnvironmentKey;
  end
  else
  begin
    RootKey := HKEY_CURRENT_USER;
    RegKey := UserEnvironmentKey;
  end;

  if RegQueryStringValue(RootKey, RegKey, 'Path', Paths) then
  begin
    if Pos(Uppercase(PathToAdd), Uppercase(Paths)) = 0 then
    begin
      Paths := Paths + ';' + PathToAdd;
      RegWriteStringValue(RootKey, RegKey, 'Path', Paths);
    end;
  end
  else
  begin
    RegWriteStringValue(RootKey, RegKey, 'Path', PathToAdd);
  end;
end;

procedure RemovePathFromEnvironment(PathToRemove: string);
var
  Paths: string;
  P: Integer;
  RegKey: string;
  RootKey: Integer;
begin
  if IsAdminLoggedOn then
  begin
    RootKey := HKEY_LOCAL_MACHINE;
    RegKey := EnvironmentKey;
  end
  else
  begin
    RootKey := HKEY_CURRENT_USER;
    RegKey := UserEnvironmentKey;
  end;

  if RegQueryStringValue(RootKey, RegKey, 'Path', Paths) then
  begin
    P := Pos(Uppercase(PathToRemove), Uppercase(Paths));
    if P > 0 then
    begin
      Delete(Paths, P, Length(PathToRemove));
      // Clean up stray double semicolons
      StringChangeEx(Paths, ';;', ';', True);
      if (Length(Paths) > 0) and (Paths[1] = ';') then
        Delete(Paths, 1, 1);
      if (Length(Paths) > 0) and (Paths[Length(Paths)] = ';') then
        Delete(Paths, Length(Paths), 1);
      RegWriteStringValue(RootKey, RegKey, 'Path', Paths);
    end;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    if WizardIsTaskSelected('addtopath') then
    begin
      AddPathToEnvironment(ExpandConstant('{app}'));
    end;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    RemovePathFromEnvironment(ExpandConstant('{app}'));
  end;
end;
