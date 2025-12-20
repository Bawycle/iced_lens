; SPDX-License-Identifier: MPL-2.0
; Inno Setup Script for IcedLens Windows Installer
; Build with: iscc build-windows-installer.iss
;
; Prerequisites:
;   1. Build the release binary: cargo build --release
;   2. Ensure FFmpeg DLLs are available (see FFmpeg section below)
;   3. Run Inno Setup Compiler on this script
;
; Output: target/release/IcedLens-{version}-x86_64-setup.exe

#define MyAppName "IcedLens"
#define MyAppVersion GetStringFileInfo("..\target\release\iced_lens.exe", "ProductVersion")
#define MyAppPublisher "Bawycle"
#define MyAppURL "https://codeberg.org/Bawycle/iced_lens"
#define MyAppExeName "iced_lens.exe"

; Fallback version if exe doesn't exist yet
#ifndef MyAppVersion
  #define MyAppVersion "0.4.1"
#endif

[Setup]
; Application identity
AppId={{A7B8C9D0-E1F2-4A5B-6C7D-8E9F0A1B2C3D}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases

; Installation directories
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes

; Output settings
OutputDir=..\target\release
OutputBaseFilename=IcedLens-{#MyAppVersion}-x86_64-setup
SetupIconFile=..\assets\branding\iced_lens.ico
UninstallDisplayIcon={app}\{#MyAppExeName}

; Compression
Compression=lzma2/ultra64
SolidCompression=yes
LZMAUseSeparateProcess=yes

; Windows version requirements (Windows 10 1809+)
MinVersion=10.0.17763

; Architecture
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

; Privileges
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

; Misc
WizardStyle=modern
DisableWelcomePage=no
ShowLanguageDialog=auto

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "italian"; MessagesFile: "compiler:Languages\Italian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "associateimages"; Description: "Associate with image files (JPEG, PNG, GIF, WebP, BMP, TIFF)"; GroupDescription: "File associations:"; Flags: unchecked
Name: "associatevideos"; Description: "Associate with video files (MP4, AVI, MOV, MKV, WebM)"; GroupDescription: "File associations:"; Flags: unchecked

[Files]
; Main executable
Source: "..\target\release\iced_lens.exe"; DestDir: "{app}"; Flags: ignoreversion

; Translations
Source: "..\assets\i18n\*.ftl"; DestDir: "{app}\assets\i18n"; Flags: ignoreversion recursesubdirs

; Icon files (for shell integration)
Source: "..\assets\branding\iced_lens.ico"; DestDir: "{app}"; Flags: ignoreversion

; FFmpeg DLLs - These must be obtained separately and placed in target/release/
; Required: avcodec-*.dll, avformat-*.dll, avutil-*.dll, swscale-*.dll, swresample-*.dll
; Optional: avdevice-*.dll, avfilter-*.dll
; Download from: https://github.com/BtbN/FFmpeg-Builds/releases (shared build)
Source: "..\target\release\*.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

; Licenses
Source: "..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion
Source: "..\THIRD_PARTY_LICENSES.md"; DestDir: "{app}"; DestName: "THIRD_PARTY_LICENSES.txt"; Flags: ignoreversion

[Icons]
; Start Menu
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"

; Desktop (optional)
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; App Paths registration for command-line access
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\App Paths\iced_lens.exe"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName}"; Flags: uninsdeletekey

; Image file associations
Root: HKCU; Subkey: "Software\Classes\.jpg\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Image"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associateimages
Root: HKCU; Subkey: "Software\Classes\.jpeg\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Image"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associateimages
Root: HKCU; Subkey: "Software\Classes\.png\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Image"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associateimages
Root: HKCU; Subkey: "Software\Classes\.gif\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Image"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associateimages
Root: HKCU; Subkey: "Software\Classes\.webp\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Image"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associateimages
Root: HKCU; Subkey: "Software\Classes\.bmp\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Image"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associateimages
Root: HKCU; Subkey: "Software\Classes\.tiff\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Image"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associateimages
Root: HKCU; Subkey: "Software\Classes\.tif\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Image"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associateimages
Root: HKCU; Subkey: "Software\Classes\.ico\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Image"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associateimages
Root: HKCU; Subkey: "Software\Classes\.svg\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Image"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associateimages

; Video file associations
Root: HKCU; Subkey: "Software\Classes\.mp4\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Video"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associatevideos
Root: HKCU; Subkey: "Software\Classes\.avi\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Video"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associatevideos
Root: HKCU; Subkey: "Software\Classes\.mov\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Video"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associatevideos
Root: HKCU; Subkey: "Software\Classes\.mkv\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Video"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associatevideos
Root: HKCU; Subkey: "Software\Classes\.webm\OpenWithProgids"; ValueType: string; ValueName: "IcedLens.Video"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associatevideos

; ProgID for images
Root: HKCU; Subkey: "Software\Classes\IcedLens.Image"; ValueType: string; ValueName: ""; ValueData: "Image File"; Flags: uninsdeletekey; Tasks: associateimages
Root: HKCU; Subkey: "Software\Classes\IcedLens.Image\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\iced_lens.ico,0"; Tasks: associateimages
Root: HKCU; Subkey: "Software\Classes\IcedLens.Image\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: associateimages

; ProgID for videos
Root: HKCU; Subkey: "Software\Classes\IcedLens.Video"; ValueType: string; ValueName: ""; ValueData: "Video File"; Flags: uninsdeletekey; Tasks: associatevideos
Root: HKCU; Subkey: "Software\Classes\IcedLens.Video\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\iced_lens.ico,0"; Tasks: associatevideos
Root: HKCU; Subkey: "Software\Classes\IcedLens.Video\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: associatevideos

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Clean up user data directory (optional - commented out to preserve settings)
; Type: filesandordirs; Name: "{localappdata}\IcedLens"

[Code]
// SHChangeNotify import - must be declared before use
procedure SHChangeNotify(wEventId: Integer; uFlags: Integer; dwItem1: Integer; dwItem2: Integer);
  external 'SHChangeNotify@shell32.dll stdcall';

const
  SHCNE_ASSOCCHANGED = $08000000;
  SHCNF_IDLIST = $0000;

// Notify Windows Explorer of file association changes
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    // Refresh shell icons
    SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, 0, 0);
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    // Refresh shell icons after uninstall
    SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, 0, 0);
  end;
end;
