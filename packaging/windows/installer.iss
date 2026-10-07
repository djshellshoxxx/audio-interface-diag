; Inno Setup script for Audio Interface Diag. Compile with:
;   iscc /DAppVersion=0.0.1-beta /DSrcDir=..\..\target\bundled /DOutDir=..\..\dist installer.iss
#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef SrcDir
  #define SrcDir "..\..\target\bundled"
#endif
#ifndef OutDir
  #define OutDir "..\..\dist"
#endif

[Setup]
AppId={{6C1D7A52-2B0E-4C8E-9A61-A1D000000001}
AppName=Audio Interface Diag
AppVersion={#AppVersion}
AppPublisher=Circuit Drift Labs
DefaultDirName={autopf}\Audio Interface Diag
DefaultGroupName=Audio Interface Diag
OutputDir={#OutDir}
OutputBaseFilename=AudioInterfaceDiag-{#AppVersion}-windows-x64-setup
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin

[Types]
Name: "full"; Description: "Standalone, CLAP and VST3"
Name: "custom"; Description: "Custom"; Flags: iscustom

[Components]
Name: "standalone"; Description: "Standalone application"; Types: full
Name: "clap"; Description: "CLAP plugin"; Types: full
Name: "vst3"; Description: "VST3 plugin"; Types: full

[Files]
Source: "{#SrcDir}\Audio Interface Diag.exe"; DestDir: "{app}"; Components: standalone; Flags: ignoreversion
Source: "{#SrcDir}\Audio Interface Diag.clap"; DestDir: "{commoncf64}\CLAP"; Components: clap; Flags: ignoreversion
Source: "{#SrcDir}\Audio Interface Diag.vst3\*"; DestDir: "{commoncf64}\VST3\Audio Interface Diag.vst3"; Components: vst3; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\Audio Interface Diag"; Filename: "{app}\Audio Interface Diag.exe"; Components: standalone
Name: "{group}\Uninstall Audio Interface Diag"; Filename: "{uninstallexe}"

[Run]
Filename: "{app}\Audio Interface Diag.exe"; Description: "Launch Audio Interface Diag"; Flags: nowait postinstall skipifsilent; Components: standalone
