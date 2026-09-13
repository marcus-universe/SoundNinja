Unicode true
ManifestDPIAware true
; Add in `dpiAwareness` `PerMonitorV2` to manifest for Windows 10 1607+ (note this should not affect lower versions since they should be able to ignore this and pick up `dpiAware` `true` set by `ManifestDPIAware true`)
; Currently undocumented on NSIS's website but is in the Docs folder of source tree, see
; https://github.com/kichik/nsis/blob/5fc0b87b819a9eec006df4967d08e522ddd651c9/Docs/src/attributes.but#L286-L300
; https://github.com/tauri-apps/tauri/pull/10106
ManifestDPIAwareness PerMonitorV2

!if "{{compression}}" == "none"
 SetCompress off
!else
 ; Set the compression algorithm. We default to LZMA.
 SetCompressor /SOLID "{{compression}}"
!endif

; Keep above !include to stay ahead of any plugin command
; see https://github.com/tauri-apps/tauri/pull/15422#discussion_r3289239624
{{#if signed_plugins_path}}
!addplugindir "{{signed_plugins_path}}"
{{/if}}

; Always-dark installer to match the app. Must be set before MUI2.nsh
; or MUI bakes in the default white background.
!define MUI_BGCOLOR "222831"
!define MUI_TEXTCOLOR "EEEEEE"
!define MUI_HEADER_TRANSPARENT_TEXT
!define MUI_LICENSEPAGE_BGCOLOR "222831"
!define MUI_DIRECTORYPAGE_BGCOLOR "222831"
!define MUI_INSTFILESPAGE_COLORS "EEEEEE 222831"
!include MUI2.nsh
!include FileFunc.nsh
!include x64.nsh
!include WordFunc.nsh
!include nsDialogs.nsh
!include LogicLib.nsh
!include "utils.nsh"
!include "FileAssociation.nsh"
!include "Win\COM.nsh"
!include "Win\Propkey.nsh"
!include "StrFunc.nsh"
!include WinMessages.nsh
${StrCase}
${StrLoc}

{{#if installer_hooks}}
!include "{{installer_hooks}}"
{{/if}}

!define WEBVIEW2APPGUID "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"

!define MANUFACTURER "{{manufacturer}}"
!define PRODUCTNAME "{{product_name}}"
!define VERSION "{{version}}"
!define VERSIONWITHBUILD "{{version_with_build}}"
!define HOMEPAGE "{{homepage}}"
!define INSTALLMODE "{{install_mode}}"
!define LICENSE "{{license}}"
!define INSTALLERICON "{{installer_icon}}"
!define SIDEBARIMAGE "{{sidebar_image}}"
!define HEADERIMAGE "{{header_image}}"
!define UNINSTALLERICON "{{uninstaller_icon}}"
!define UNINSTALLERHEADERIMAGE "{{uninstaller_header_image}}"
!define MAINBINARYNAME "{{main_binary_name}}"
!define MAINBINARYSRCPATH "{{main_binary_path}}"
!define BUNDLEID "{{bundle_id}}"
!define COPYRIGHT "{{copyright}}"
!define OUTFILE "{{out_file}}"
!define ARCH "{{arch}}"
!define ADDITIONALPLUGINSPATH "{{additional_plugins_path}}"
!define ALLOWDOWNGRADES "{{allow_downgrades}}"
!define DISPLAYLANGUAGESELECTOR "{{display_language_selector}}"
!define INSTALLWEBVIEW2MODE "{{install_webview2_mode}}"
!define WEBVIEW2INSTALLERARGS "{{webview2_installer_args}}"
!define WEBVIEW2BOOTSTRAPPERPATH "{{webview2_bootstrapper_path}}"
!define WEBVIEW2INSTALLERPATH "{{webview2_installer_path}}"
!define MINIMUMWEBVIEW2VERSION "{{minimum_webview2_version}}"
; Stable ARP key so upgrades from productName "soundninja" do not create a second Installed Apps row.
!define UNINSTKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\soundninja"
!define MANUKEY "Software\${MANUFACTURER}"
!define MANUPRODUCTKEY "${MANUKEY}\${PRODUCTNAME}"
; Pre-rename productName was "soundninja"; old installs wrote this key.
!define LEGACYMANUPRODUCTKEY "${MANUKEY}\soundninja"
!define UNINSTALLERSIGNCOMMAND "{{uninstaller_sign_cmd}}"
!define ESTIMATEDSIZE "{{estimated_size}}"
!define STARTMENUFOLDER "{{start_menu_folder}}"

Var PassiveMode
Var UpdateMode
Var NoShortcutMode
Var WixMode
Var OldMainBinaryName
Var StemsDialog
Var StemsCheckbox
Var StemsCheckboxState
Var LangDialog
Var LangCombo
Var LangChoice

Name "${PRODUCTNAME}"
BrandingText "${PRODUCTNAME} ${VERSION}"
OutFile "${OUTFILE}"

; We don't actually use this value as default install path,
; it's just for nsis to append the product name folder in the directory selector
; https://nsis.sourceforge.io/Reference/InstallDir
!define PLACEHOLDER_INSTALL_DIR "placeholder\${PRODUCTNAME}"
InstallDir "${PLACEHOLDER_INSTALL_DIR}"

VIProductVersion "${VERSIONWITHBUILD}"
VIAddVersionKey "ProductName" "${PRODUCTNAME}"
VIAddVersionKey "FileDescription" "${PRODUCTNAME}"
VIAddVersionKey "LegalCopyright" "${COPYRIGHT}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"

# additional plugins
!addplugindir "${ADDITIONALPLUGINSPATH}"

; Uninstaller signing command
!if "${UNINSTALLERSIGNCOMMAND}" != ""
 !uninstfinalize '${UNINSTALLERSIGNCOMMAND}'
!endif

; Handle install mode, `perUser`, `perMachine` or `both`
!if "${INSTALLMODE}" == "perMachine"
 RequestExecutionLevel admin
!endif

!if "${INSTALLMODE}" == "currentUser"
 RequestExecutionLevel user
!endif

!if "${INSTALLMODE}" == "both"
 !define MULTIUSER_MUI
 !define MULTIUSER_INSTALLMODE_INSTDIR "${PRODUCTNAME}"
 !define MULTIUSER_INSTALLMODE_COMMANDLINE
 !if "${ARCH}" == "x64"
 !define MULTIUSER_USE_PROGRAMFILES64
 !else if "${ARCH}" == "arm64"
 !define MULTIUSER_USE_PROGRAMFILES64
 !endif
 !define MULTIUSER_INSTALLMODE_DEFAULT_REGISTRY_KEY "${UNINSTKEY}"
 !define MULTIUSER_INSTALLMODE_DEFAULT_REGISTRY_VALUENAME "CurrentUser"
 !define MULTIUSER_INSTALLMODEPAGE_SHOWUSERNAME
 !define MULTIUSER_INSTALLMODE_FUNCTION RestorePreviousInstallLocation
 !define MULTIUSER_EXECUTIONLEVEL Highest
 !include MultiUser.nsh
!endif

; Installer icon
!if "${INSTALLERICON}" != ""
 !define MUI_ICON "${INSTALLERICON}"
!endif

; Installer sidebar image
!if "${SIDEBARIMAGE}" != ""
 !define MUI_WELCOMEFINISHPAGE_BITMAP "${SIDEBARIMAGE}"
 !define MUI_WELCOMEFINISHPAGE_BITMAP_STRETCH AspectFitHeight
!endif

; Enable header images for installer and uninstaller pages when either image is configured.
!if "${HEADERIMAGE}" != ""
 !define MUI_HEADERIMAGE
!else if "${UNINSTALLERHEADERIMAGE}" != ""
 !define MUI_HEADERIMAGE
!endif

; Installer header image
!if "${HEADERIMAGE}" != ""
 !define MUI_HEADERIMAGE_BITMAP "${HEADERIMAGE}"
 !define MUI_HEADERIMAGE_BITMAP_NOSTRETCH
!endif

; Uninstaller header image
!if "${UNINSTALLERHEADERIMAGE}" != ""
 !define MUI_HEADERIMAGE_UNBITMAP "${UNINSTALLERHEADERIMAGE}"
!endif

; Uninstaller icon
!if "${UNINSTALLERICON}" != ""
 !define MUI_UNICON "${UNINSTALLERICON}"
!endif

; Define registry key to store installer language
!define MUI_LANGDLL_REGISTRY_ROOT "HKCU"
!define MUI_LANGDLL_REGISTRY_KEY "${MANUPRODUCTKEY}"
!define MUI_LANGDLL_REGISTRY_VALUENAME "Installer Language"

; Untheme a control so SetCtlColors sticks (NSIS bug #443 / themed BUTTON/STATIC).
!macro DarkenHwnd HWND
 Push $9
 StrCpy $9 ${HWND}
 ${If} $9 != 0
  System::Call 'uxtheme::SetWindowTheme(p r9, w " ", w " ")'
  SetCtlColors $9 "EEEEEE" "222831"
 ${EndIf}
 Pop $9
!macroend

!macro DarkenNavButton ID
 GetDlgItem $1 $HWNDPARENT ${ID}
 ${If} $1 != 0
  System::Call 'uxtheme::SetWindowTheme(p r1, w " ", w " ")'
  SetCtlColors $1 "EEEEEE" "363f4d"
 ${EndIf}
!macroend

; Recolor every child except SS_BITMAP (sidebar/header art).
!macro DarkenChildWindows PARENT
 Push $2
 Push $3
 Push $5
 Push $6
 StrCpy $2 ${PARENT}
 ${If} $2 != 0
  System::Call 'user32::GetWindow(p r2, i 5) p .r3'
  ${While} $3 != 0
   System::Call 'user32::GetClassName(p r3, t .r5, i 64)'
   System::Call 'user32::GetWindowLong(p r3, i -16) i .r6'
   IntOp $6 $6 & 0xF
   ${If} $6 != 14
    ${If} $5 == "Button"
     System::Call 'uxtheme::SetWindowTheme(p r3, w " ", w " ")'
    ${EndIf}
    SetCtlColors $3 "EEEEEE" "222831"
   ${EndIf}
   System::Call 'user32::GetWindow(p r3, i 2) p .r3'
  ${EndWhile}
 ${EndIf}
 Pop $6
 Pop $5
 Pop $3
 Pop $2
!macroend

!macro ApplyDarkUiImpl
 SetCtlColors $HWNDPARENT "EEEEEE" "222831"
 ; Dark titlebar (20 = Win10 20H1+, 19 = older 1809 builds)
 System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 20, *i 1, i 4)'
 System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 19, *i 1, i 4)'

 FindWindow $0 "#32770" "" $HWNDPARENT
 SetCtlColors $0 "EEEEEE" "222831"
 !insertmacro DarkenChildWindows $HWNDPARENT
 !insertmacro DarkenChildWindows $0

 GetDlgItem $1 $HWNDPARENT 1034
 SetCtlColors $1 "EEEEEE" "222831"
 GetDlgItem $1 $HWNDPARENT 1037
 SetCtlColors $1 "EEEEEE" "222831"
 GetDlgItem $1 $HWNDPARENT 1038
 SetCtlColors $1 "EEEEEE" "222831"
 GetDlgItem $1 $HWNDPARENT 1028
 SetCtlColors $1 "EEEEEE" "222831"

 ; Classic InstallOptions welcome IDs (no-op on nsDialogs MUI2)
 GetDlgItem $1 $HWNDPARENT 1201
 SetCtlColors $1 "EEEEEE" "222831"
 GetDlgItem $1 $HWNDPARENT 1202
 SetCtlColors $1 "EEEEEE" "222831"
 ${If} $0 != 0
  GetDlgItem $1 $0 1201
  SetCtlColors $1 "EEEEEE" "222831"
  GetDlgItem $1 $0 1202
  SetCtlColors $1 "EEEEEE" "222831"
 ${EndIf}

 ; Next / Cancel / Back
 !insertmacro DarkenNavButton 1
 !insertmacro DarkenNavButton 2
 !insertmacro DarkenNavButton 3
!macroend

Function ApplyDarkUi
 !insertmacro ApplyDarkUiImpl
FunctionEnd
Function un.ApplyDarkUi
 !insertmacro ApplyDarkUiImpl
FunctionEnd

!macro StripOuterQuotes VAR
 Push $7
 Push $8
 Push $9
 StrCpy $9 ${VAR}
 StrCpy $8 $9 1
 StrCpy $7 "$\""
 ${If} $8 == $7
  StrLen $8 $9
  IntOp $8 $8 - 2
  ${If} $8 > 0
   StrCpy $9 $9 $8 1
  ${Else}
   StrCpy $9 ""
  ${EndIf}
  StrCpy ${VAR} $9
 ${EndIf}
 Pop $9
 Pop $8
 Pop $7
!macroend

!macro TryReadInstallDir ROOT KEY NAME
 ${If} $4 == ""
  ClearErrors
  ReadRegStr $4 ${ROOT} "${KEY}" "${NAME}"
  ${If} ${Errors}
   StrCpy $4 ""
  ${Else}
   !insertmacro StripOuterQuotes $4
  ${EndIf}
 ${EndIf}
!macroend

; $4 = previous install directory. Survives productName rename (soundninja -> Sound Ninja).
Function ResolveOldInstallDir
 StrCpy $4 ""
 !insertmacro TryReadInstallDir SHCTX "${MANUPRODUCTKEY}" ""
 !insertmacro TryReadInstallDir SHCTX "${LEGACYMANUPRODUCTKEY}" ""
 !insertmacro TryReadInstallDir HKLM "${MANUPRODUCTKEY}" ""
 !insertmacro TryReadInstallDir HKLM "${LEGACYMANUPRODUCTKEY}" ""
 !insertmacro TryReadInstallDir SHCTX "${UNINSTKEY}" "InstallLocation"
 !insertmacro TryReadInstallDir HKLM "${UNINSTKEY}" "InstallLocation"
 ${If} $4 == ""
  ReadRegStr $5 SHCTX "${UNINSTKEY}" "UninstallString"
  ${If} $5 == ""
   ReadRegStr $5 HKLM "${UNINSTKEY}" "UninstallString"
  ${EndIf}
  !insertmacro StripOuterQuotes $5
  ${If} $5 != ""
   ${GetParent} "$5" $4
  ${EndIf}
 ${EndIf}
 ${If} $4 == ""
  StrCpy $4 "$INSTDIR"
 ${EndIf}
FunctionEnd

; Installer pages, must be ordered as they appear
; 1. Welcome Page
!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
!define MUI_PAGE_CUSTOMFUNCTION_SHOW WelcomeShow
!insertmacro MUI_PAGE_WELCOME

Function WelcomeShow
 Call ApplyDarkUi
 Push $0
 Push $1
 Push $9
 StrCpy $1 0
 ; MUI2 nsDialogs handle (declared by MUI_PAGE_WELCOME above)
 StrCpy $1 $mui.WelcomePage.Text
 ${If} $1 == 0
  FindWindow $0 "#32770" "" $HWNDPARENT
  GetDlgItem $1 $0 1202
 ${EndIf}
 ${If} $1 == 0
  GetDlgItem $1 $HWNDPARENT 1202
 ${EndIf}
 ${If} $1 != 0
  System::Call 'user32::GetWindowText(p r1, t .r0, i ${NSIS_MAX_STRLEN})'
  ${StrLoc} $9 $0 "$(installingVersion)" ">"
  ${If} $9 == ""
   StrCpy $0 "$0$\r$\n$\r$\n$(installingVersion)"
   SendMessage $1 ${WM_SETTEXT} 0 "STR:$0"
  ${EndIf}
  SetCtlColors $1 "EEEEEE" "222831"
 ${EndIf}
 ${If} $mui.WelcomePage.Title != 0
  SetCtlColors $mui.WelcomePage.Title "EEEEEE" "222831"
 ${EndIf}
 Pop $9
 Pop $1
 Pop $0
FunctionEnd

; 2. License Page (if defined)
!if "${LICENSE}" != ""
 !define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
 !define MUI_PAGE_CUSTOMFUNCTION_SHOW ApplyDarkUi
 !insertmacro MUI_PAGE_LICENSE "${LICENSE}"
!endif

; 3. Install mode (if it is set to `both`)
!if "${INSTALLMODE}" == "both"
 !define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
 !define MUI_PAGE_CUSTOMFUNCTION_SHOW ApplyDarkUi
 !insertmacro MULTIUSER_PAGE_INSTALLMODE
!endif

; 4. Custom page to ask user if he wants to reinstall/uninstall
; only if a previous installation was detected
Var ReinstallPageCheck
Page custom PageReinstall PageLeaveReinstall
Function PageReinstall
 ; Uninstall previous WiX installation if exists.
 ;
 ; A WiX installer stores the installation info in registry
 ; using a UUID and so we have to loop through all keys under
 ; `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`
 ; and check if `DisplayName` and `Publisher` keys match ${PRODUCTNAME} and ${MANUFACTURER}
 ;
 ; This has a potential issue that there maybe another installation that matches
 ; our ${PRODUCTNAME} and ${MANUFACTURER} but wasn't installed by our WiX installer,
 ; however, this should be fine since the user will have to confirm the uninstallation
 ; and they can chose to abort it if doesn't make sense.
 StrCpy $0 0
 wix_loop:
 EnumRegKey $1 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall" $0
 StrCmp $1 "" wix_loop_done ; Exit loop if there is no more keys to loop on
 IntOp $0 $0 + 1
 ReadRegStr $R0 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$1" "DisplayName"
 ReadRegStr $R1 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$1" "Publisher"
 StrCmp "$R0$R1" "${PRODUCTNAME}${MANUFACTURER}" 0 wix_loop
 ReadRegStr $R0 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$1" "UninstallString"
 ${StrCase} $R1 $R0 "L"
 ${StrLoc} $R0 $R1 "msiexec" ">"
 StrCmp $R0 0 0 wix_loop_done
 StrCpy $WixMode 1
 StrCpy $R6 "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$1"
 Goto compare_version
 wix_loop_done:

 ; Check if there is an existing installation, if not, abort the reinstall page
 ReadRegStr $R0 SHCTX "${UNINSTKEY}" ""
 ReadRegStr $R1 SHCTX "${UNINSTKEY}" "UninstallString"
 ${IfThen} "$R0$R1" == "" ${|} Abort ${|}

 ; Compare this installar version with the existing installation
 ; and modify the messages presented to the user accordingly
 compare_version:
 StrCpy $R4 "$(older)"
 ${If} $WixMode = 1
 ReadRegStr $R0 HKLM "$R6" "DisplayVersion"
 ${Else}
 ReadRegStr $R0 SHCTX "${UNINSTKEY}" "DisplayVersion"
 ${EndIf}
 ${IfThen} $R0 == "" ${|} StrCpy $R4 "$(unknown)" ${|}

 nsis_tauri_utils::SemverCompare "${VERSION}" $R0
 Pop $R0
 ; Reinstalling the same version
 ${If} $R0 = 0
 StrCpy $R1 "$(alreadyInstalledLong)"
 StrCpy $R2 "$(addOrReinstall)"
 StrCpy $R3 "$(uninstallApp)"
 !insertmacro MUI_HEADER_TEXT "$(alreadyInstalled)" "$(chooseMaintenanceOption)"
 ; Upgrading
 ${ElseIf} $R0 = 1
 StrCpy $R1 "$(olderOrUnknownVersionInstalled)"
 StrCpy $R2 "$(uninstallBeforeInstalling)"
 StrCpy $R3 "$(dontUninstall)"
 !insertmacro MUI_HEADER_TEXT "$(alreadyInstalled)" "$(choowHowToInstall)"
 ; Downgrading
 ${ElseIf} $R0 = -1
 StrCpy $R1 "$(newerVersionInstalled)"
 StrCpy $R2 "$(uninstallBeforeInstalling)"
 !if "${ALLOWDOWNGRADES}" == "true"
 StrCpy $R3 "$(dontUninstall)"
 !else
 StrCpy $R3 "$(dontUninstallDowngrade)"
 !endif
 !insertmacro MUI_HEADER_TEXT "$(alreadyInstalled)" "$(choowHowToInstall)"
 ${Else}
 Abort
 ${EndIf}

 ; Skip showing the page if passive
 ;
 ; Note that we don't call this earlier at the beginning
 ; of this function because we need to populate some variables
 ; related to current installed version if detected and whether
 ; we are downgrading or not.
 ${If} $PassiveMode = 1
 Call PageLeaveReinstall
 ${Else}
 nsDialogs::Create 1018
 Pop $R4
 ${IfThen} $(^RTL) = 1 ${|} nsDialogs::SetRTL $(^RTL) ${|}

 ${NSD_CreateLabel} 0 0 100% 24u $R1
 Pop $R1
 SetCtlColors $R4 "EEEEEE" "222831"
 SetCtlColors $R1 "EEEEEE" "222831"

 ${NSD_CreateRadioButton} 30u 50u -30u 8u $R2
 Pop $R2
 !insertmacro DarkenHwnd $R2
 ${NSD_OnClick} $R2 PageReinstallUpdateSelection

 ${NSD_CreateRadioButton} 30u 70u -30u 8u $R3
 Pop $R3
 !insertmacro DarkenHwnd $R3
 ; Disable this radio button if downgrading and downgrades are disabled
 !if "${ALLOWDOWNGRADES}" == "false"
 ${IfThen} $R0 = -1 ${|} EnableWindow $R3 0 ${|}
 !endif
 ${NSD_OnClick} $R3 PageReinstallUpdateSelection

 ; Check the first radio button if this the first time
 ; we enter this page or if the second button wasn't
 ; selected the last time we were on this page
 ${If} $ReinstallPageCheck <> 2
 SendMessage $R2 ${BM_SETCHECK} ${BST_CHECKED} 0
 ${Else}
 SendMessage $R3 ${BM_SETCHECK} ${BST_CHECKED} 0
 ${EndIf}

 ${NSD_SetFocus} $R2
 Call ApplyDarkUi
 nsDialogs::Show
 ${EndIf}
FunctionEnd
Function PageReinstallUpdateSelection
 ${NSD_GetState} $R2 $R1
 ${If} $R1 == ${BST_CHECKED}
 StrCpy $ReinstallPageCheck 1
 ${Else}
 StrCpy $ReinstallPageCheck 2
 ${EndIf}
FunctionEnd
Function PageLeaveReinstall
 ${NSD_GetState} $R2 $R1

 ; If migrating from Wix, always uninstall
 ${If} $WixMode = 1
 Goto reinst_uninstall
 ${EndIf}

 ; In update mode, always proceeds without uninstalling
 ${If} $UpdateMode = 1
 Goto reinst_done
 ${EndIf}

 ; $R0 holds whether same(0)/upgrading(1)/downgrading(-1) version
 ; $R1 holds the radio buttons state:
 ; 1 => first choice was selected
 ; 0 => second choice was selected
 ${If} $R0 = 0 ; Same version, proceed
 ${If} $R1 = 1 ; User chose to add/reinstall
 Goto reinst_done
 ${Else} ; User chose to uninstall
 Goto reinst_uninstall
 ${EndIf}
 ${ElseIf} $R0 = 1 ; Upgrading
 ${If} $R1 = 1 ; User chose to uninstall
 Goto reinst_uninstall
 ${Else}
 Goto reinst_done ; User chose NOT to uninstall
 ${EndIf}
 ${ElseIf} $R0 = -1 ; Downgrading
 ${If} $R1 = 1 ; User chose to uninstall
 Goto reinst_uninstall
 ${Else}
 Goto reinst_done ; User chose NOT to uninstall
 ${EndIf}
 ${EndIf}

 reinst_uninstall:
 ${If} $WixMode = 1
 HideWindow
 ClearErrors
 ReadRegStr $R1 HKLM "$R6" "UninstallString"
 ExecWait '$R1' $0
 BringToFront
 ${IfThen} ${Errors} ${|} StrCpy $0 2 ${|}
 ${If} $0 = 1602
 Abort
 ${EndIf}
 ${If} $0 = 1
 Abort
 ${EndIf}
 ${If} $0 <> 0
 ${OrIf} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
 MessageBox MB_ICONEXCLAMATION "$(unableToUninstall)"
 Abort
 ${EndIf}
 Goto reinst_done
 ${EndIf}

 Call ResolveOldInstallDir

 StrCpy $5 ""
 ${If} ${FileExists} "$4\uninstall.exe"
 StrCpy $5 "$4\uninstall.exe"
 ${Else}
 ReadRegStr $5 SHCTX "${UNINSTKEY}" "UninstallString"
 ${If} $5 == ""
 ReadRegStr $5 HKLM "${UNINSTKEY}" "UninstallString"
 ${EndIf}
 !insertmacro StripOuterQuotes $5
 ${EndIf}

 ; Missing uninstaller: in-place upgrade instead of blocking
 ${If} $5 == ""
 ${OrIfNot} ${FileExists} "$5"
 Goto reinst_done
 ${EndIf}

 HideWindow
 ClearErrors
 InitPluginsDir
 CopyFiles /SILENT "$5" "$PLUGINSDIR\old-uninstall.exe"
 ${IfNot} ${FileExists} "$PLUGINSDIR\old-uninstall.exe"
 BringToFront
 Goto reinst_done
 ${EndIf}

 StrCpy $R1 "$\"$PLUGINSDIR\old-uninstall.exe$\""
 ${IfThen} $UpdateMode = 1 ${|} StrCpy $R1 '$R1 /UPDATE' ${|}
 ${IfThen} $PassiveMode = 1 ${|} StrCpy $R1 '$R1 /P' ${|}
 StrCpy $R1 '$R1 _?=$4'
 ExecWait '$R1' $0
 BringToFront

 ${IfThen} ${Errors} ${|} StrCpy $0 2 ${|}

 ; User cancelled NSIS uninstaller? return to select un/reinstall page
 ${If} $0 = 1
 Abort
 ${EndIf}

 ${If} $0 <> 0
 ${OrIf} ${FileExists} "$4\${MAINBINARYNAME}.exe"
 MessageBox MB_ICONEXCLAMATION "$(unableToUninstall)"
 Abort
 ${EndIf}
 reinst_done:
FunctionEnd

; 5. Choose install directory page
!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
!define MUI_PAGE_CUSTOMFUNCTION_SHOW ApplyDarkUi
!insertmacro MUI_PAGE_DIRECTORY

; 5a. Default app language
Page custom LangPageCreate LangPageLeave

Function LangPageCreate
 ${If} $PassiveMode = 1
 ${OrIf} ${Silent}
 Abort
 ${EndIf}
 !insertmacro MUI_HEADER_TEXT "Language" "Choose the default language for Sound Ninja"
 nsDialogs::Create 1018
 Pop $LangDialog
 ${If} $LangDialog == error
 Abort
 ${EndIf}
 ${NSD_CreateLabel} 0 0 100% 28u "Select the language used when Sound Ninja first starts. You can change this later in Settings."
 Pop $0
 SetCtlColors $LangDialog "EEEEEE" "222831"
 SetCtlColors $0 "EEEEEE" "222831"

 ${NSD_CreateDropList} 0 40u 70% 12u ""
 Pop $LangCombo
 ${NSD_CB_AddString} $LangCombo "English"
 ${NSD_CB_AddString} $LangCombo "Deutsch"
 ${NSD_CB_AddString} $LangCombo "Español"
 ${NSD_CB_AddString} $LangCombo "Français"
 ${NSD_CB_AddString} $LangCombo "日本語"
 ${NSD_CB_AddString} $LangCombo "简体中文"

 ${If} $LangChoice == "de"
  ${NSD_CB_SelectString} $LangCombo "Deutsch"
 ${ElseIf} $LangChoice == "es"
  ${NSD_CB_SelectString} $LangCombo "Español"
 ${ElseIf} $LangChoice == "fr"
  ${NSD_CB_SelectString} $LangCombo "Français"
 ${ElseIf} $LangChoice == "ja"
  ${NSD_CB_SelectString} $LangCombo "日本語"
 ${ElseIf} $LangChoice == "zh-Hans"
  ${NSD_CB_SelectString} $LangCombo "简体中文"
 ${Else}
  ${NSD_CB_SelectString} $LangCombo "English"
 ${EndIf}

 Call ApplyDarkUi
 nsDialogs::Show
FunctionEnd

Function LangPageLeave
 ${NSD_GetText} $LangCombo $0
 ${If} $0 == "Deutsch"
  StrCpy $LangChoice "de"
 ${ElseIf} $0 == "Español"
  StrCpy $LangChoice "es"
 ${ElseIf} $0 == "Français"
  StrCpy $LangChoice "fr"
 ${ElseIf} $0 == "日本語"
  StrCpy $LangChoice "ja"
 ${ElseIf} $0 == "简体中文"
  StrCpy $LangChoice "zh-Hans"
 ${Else}
  StrCpy $LangChoice "en"
 ${EndIf}
FunctionEnd

; 5b. Optional AI stem model download (on first app launch)
Page custom StemsPageCreate StemsPageLeave

Function StemsPageCreate
 ${If} $PassiveMode = 1
 ${OrIf} ${Silent}
 Abort ; skip page in silent/passive mode — default checked via $StemsCheckboxState
 ${EndIf}
 !insertmacro MUI_HEADER_TEXT "AI Stem Separation" "Optional model download on first launch"
 nsDialogs::Create 1018
 Pop $StemsDialog
 ${If} $StemsDialog == error
 Abort
 ${EndIf}
 ${NSD_CreateLabel} 0 0 100% 36u "Sound Ninja can separate vocals from music in the Record Editor using an AI model (BS-RoFormer, ~158 MB). The model is downloaded the first time you open the app — not during this install."
 Pop $0
 SetCtlColors $StemsDialog "EEEEEE" "222831"
 SetCtlColors $0 "EEEEEE" "222831"

 ${NSD_CreateCheckbox} 0 50u 100% 12u "Download the AI stem separation model on first launch (~158 MB)"
 Pop $StemsCheckbox
 !insertmacro DarkenHwnd $StemsCheckbox
 ; Default: checked
 ${NSD_Check} $StemsCheckbox
 StrCpy $StemsCheckboxState 1

 Call ApplyDarkUi
 nsDialogs::Show
FunctionEnd

Function StemsPageLeave
 ${NSD_GetState} $StemsCheckbox $StemsCheckboxState
FunctionEnd

; 6. Start menu shortcut page
Var AppStartMenuFolder
!if "${STARTMENUFOLDER}" != ""
 !define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
 !define MUI_STARTMENUPAGE_DEFAULTFOLDER "${STARTMENUFOLDER}"
!else
 !define MUI_PAGE_CUSTOMFUNCTION_PRE Skip
!endif
!define MUI_PAGE_CUSTOMFUNCTION_SHOW ApplyDarkUi
!insertmacro MUI_PAGE_STARTMENU Application $AppStartMenuFolder

; 7. Installation page
!define MUI_PAGE_CUSTOMFUNCTION_SHOW ApplyDarkUi
!insertmacro MUI_PAGE_INSTFILES

; 8. Finish page
;
; Don't auto jump to finish page after installation page,
; because the installation page has useful info that can be used debug any issues with the installer.
!define MUI_FINISHPAGE_NOAUTOCLOSE
; Use show readme button in the finish page as a button create a desktop shortcut
!define MUI_FINISHPAGE_SHOWREADME
!define MUI_FINISHPAGE_SHOWREADME_TEXT "$(createDesktop)"
!define MUI_FINISHPAGE_SHOWREADME_FUNCTION CreateOrUpdateDesktopShortcut
; Show run app after installation.
!define MUI_FINISHPAGE_RUN
!define MUI_FINISHPAGE_RUN_FUNCTION RunMainBinary
!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
!define MUI_PAGE_CUSTOMFUNCTION_SHOW FinishShow
!insertmacro MUI_PAGE_FINISH

Function FinishShow
 Call ApplyDarkUi
 !insertmacro DarkenHwnd $mui.FinishPage.Title
 !insertmacro DarkenHwnd $mui.FinishPage.Text
 !insertmacro DarkenHwnd $mui.FinishPage.Run
 !insertmacro DarkenHwnd $mui.FinishPage.ShowReadme
FunctionEnd

Function RunMainBinary
 nsis_tauri_utils::RunAsUser "$INSTDIR\${MAINBINARYNAME}.exe" ""
FunctionEnd

; Uninstaller Pages
; 1. Confirm uninstall page
Var DeleteAppDataCheckbox
Var DeleteAppDataCheckboxState
!define /ifndef WS_EX_LAYOUTRTL 0x00400000
!define MUI_PAGE_CUSTOMFUNCTION_SHOW un.ConfirmShow
Function un.ConfirmShow ; Add add a `Delete app data` check box
 Call un.ApplyDarkUi
 ; $1 inner dialog HWND
 ; $2 window DPI
 ; $3 style
 ; $4 x
 ; $5 y
 ; $6 width
 ; $7 height
 FindWindow $1 "#32770" "" $HWNDPARENT ; Find inner dialog
 System::Call "user32::GetDpiForWindow(p r1) i .r2"
 ${If} $(^RTL) = 1
 StrCpy $3 "${__NSD_CheckBox_EXSTYLE} | ${WS_EX_LAYOUTRTL}"
 IntOp $4 50 * $2
 ${Else}
 StrCpy $3 "${__NSD_CheckBox_EXSTYLE}"
 IntOp $4 0 * $2
 ${EndIf}
 IntOp $5 100 * $2
 IntOp $6 400 * $2
 IntOp $7 25 * $2
 IntOp $4 $4 / 96
 IntOp $5 $5 / 96
 IntOp $6 $6 / 96
 IntOp $7 $7 / 96
 System::Call 'user32::CreateWindowEx(i r3, w "${__NSD_CheckBox_CLASS}", w "$(deleteAppData)", i ${__NSD_CheckBox_STYLE}, i r4, i r5, i r6, i r7, p r1, i0, i0, i0) i .s'
 Pop $DeleteAppDataCheckbox
 SendMessage $HWNDPARENT ${WM_GETFONT} 0 0 $1
 SendMessage $DeleteAppDataCheckbox ${WM_SETFONT} $1 1
 !insertmacro DarkenHwnd $DeleteAppDataCheckbox
FunctionEnd
!define MUI_PAGE_CUSTOMFUNCTION_LEAVE un.ConfirmLeave
Function un.ConfirmLeave
 SendMessage $DeleteAppDataCheckbox ${BM_GETCHECK} 0 0 $DeleteAppDataCheckboxState
FunctionEnd
!define MUI_PAGE_CUSTOMFUNCTION_PRE un.SkipIfPassive
!insertmacro MUI_UNPAGE_CONFIRM

; 2. Uninstalling Page
!define MUI_PAGE_CUSTOMFUNCTION_SHOW un.ApplyDarkUi
!insertmacro MUI_UNPAGE_INSTFILES

;Languages
{{#each languages}}
!insertmacro MUI_LANGUAGE "{{this}}"
{{/each}}
!insertmacro MUI_RESERVEFILE_LANGDLL
{{#each language_files}}
 !include "{{this}}"
{{/each}}

!ifdef LANG_ENGLISH
 LangString installingVersion ${LANG_ENGLISH} "This will install ${PRODUCTNAME} ${VERSION}."
!endif
!ifdef LANG_GERMAN
 LangString installingVersion ${LANG_GERMAN} "Diese Installation richtet ${PRODUCTNAME} ${VERSION} ein."
!endif
!ifdef LANG_SPANISH
 LangString installingVersion ${LANG_SPANISH} "Se instalará ${PRODUCTNAME} ${VERSION}."
!endif
!ifdef LANG_FRENCH
 LangString installingVersion ${LANG_FRENCH} "Ceci installera ${PRODUCTNAME} ${VERSION}."
!endif
!ifdef LANG_JAPANESE
 LangString installingVersion ${LANG_JAPANESE} "${PRODUCTNAME} ${VERSION} をインストールします。"
!endif
!ifdef LANG_SIMPCHINESE
 LangString installingVersion ${LANG_SIMPCHINESE} "将安装 ${PRODUCTNAME} ${VERSION}。"
!endif
!ifdef LANG_TRADCHINESE
 LangString installingVersion ${LANG_TRADCHINESE} "將安裝 ${PRODUCTNAME} ${VERSION}。"
!endif
!ifdef LANG_SPANISHINTERNATIONAL
 LangString installingVersion ${LANG_SPANISHINTERNATIONAL} "Se instalará ${PRODUCTNAME} ${VERSION}."
!endif
; Fallback when current MUI language has no LangString
LangString installingVersion 0 "This will install ${PRODUCTNAME} ${VERSION}."

Function .onInit
 ; Default: download stem model on first launch (checkbox page can uncheck).
 StrCpy $StemsCheckboxState 1
 StrCpy $LangChoice "en"
 ${If} $LANGUAGE = 1031
  StrCpy $LangChoice "de"
 ${ElseIf} $LANGUAGE = 1034
  StrCpy $LangChoice "es"
 ${ElseIf} $LANGUAGE = 3082
  StrCpy $LangChoice "es"
 ${ElseIf} $LANGUAGE = 1036
  StrCpy $LangChoice "fr"
 ${ElseIf} $LANGUAGE = 1041
  StrCpy $LangChoice "ja"
 ${ElseIf} $LANGUAGE = 2052
  StrCpy $LangChoice "zh-Hans"
 ${ElseIf} $LANGUAGE = 1028
  StrCpy $LangChoice "zh-Hans"
 ${EndIf}

 ${GetOptions} $CMDLINE "/P" $PassiveMode
 ${IfNot} ${Errors}
 StrCpy $PassiveMode 1
 ${EndIf}

 ${GetOptions} $CMDLINE "/NS" $NoShortcutMode
 ${IfNot} ${Errors}
 StrCpy $NoShortcutMode 1
 ${EndIf}

 ${GetOptions} $CMDLINE "/UPDATE" $UpdateMode
 ${IfNot} ${Errors}
 StrCpy $UpdateMode 1
 ${EndIf}

 !if "${DISPLAYLANGUAGESELECTOR}" == "true"
 !insertmacro MUI_LANGDLL_DISPLAY
 !endif

 !insertmacro SetContext

 ${If} $INSTDIR == "${PLACEHOLDER_INSTALL_DIR}"
 ; Set default install location
 !if "${INSTALLMODE}" == "perMachine"
 ${If} ${RunningX64}
 !if "${ARCH}" == "x64"
 StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCTNAME}"
 !else if "${ARCH}" == "arm64"
 StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCTNAME}"
 !else
 StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"
 !endif
 ${Else}
 StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"
 ${EndIf}
 !else if "${INSTALLMODE}" == "currentUser"
 StrCpy $INSTDIR "$LOCALAPPDATA\${PRODUCTNAME}"
 !endif

 Call RestorePreviousInstallLocation
 ${EndIf}

 !if "${INSTALLMODE}" == "both"
 !insertmacro MULTIUSER_INIT
 !endif
FunctionEnd

Section EarlyChecks
 ; Abort silent installer if downgrades is disabled
 !if "${ALLOWDOWNGRADES}" == "false"
 ${If} ${Silent}
 ; If downgrading
 ${If} $R0 = -1
 System::Call 'kernel32::AttachConsole(i -1)i.r0'
 ${If} $0 <> 0
 System::Call 'kernel32::GetStdHandle(i -11)i.r0'
 System::call 'kernel32::SetConsoleTextAttribute(i r0, i 0x0004)' ; set red color
 FileWrite $0 "$(silentDowngrades)"
 ${EndIf}
 Abort
 ${EndIf}
 ${EndIf}
 !endif

SectionEnd

Section WebView2
 ; Check if Webview2 is already installed and skip this section
 ${If} ${RunningX64}
 ReadRegStr $4 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
 ${Else}
 ReadRegStr $4 HKLM "SOFTWARE\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
 ${EndIf}
 ${If} $4 == ""
 ReadRegStr $4 HKCU "SOFTWARE\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
 ${EndIf}

 ${If} $4 == ""
 ; Webview2 installation
 ;
 ; Skip if updating
 ${If} $UpdateMode <> 1
 !if "${INSTALLWEBVIEW2MODE}" == "downloadBootstrapper"
 Delete "$TEMP\MicrosoftEdgeWebview2Setup.exe"
 DetailPrint "$(webview2Downloading)"
 NSISdl::download "https://go.microsoft.com/fwlink/p/?LinkId=2124703" "$TEMP\MicrosoftEdgeWebview2Setup.exe"
 Pop $0
 ${If} $0 == "success"
 DetailPrint "$(webview2DownloadSuccess)"
 ${Else}
 DetailPrint "$(webview2DownloadError)"
 Abort "$(webview2AbortError)"
 ${EndIf}
 StrCpy $6 "$TEMP\MicrosoftEdgeWebview2Setup.exe"
 Goto install_webview2
 !endif

 !if "${INSTALLWEBVIEW2MODE}" == "embedBootstrapper"
 Delete "$TEMP\MicrosoftEdgeWebview2Setup.exe"
 File "/oname=$TEMP\MicrosoftEdgeWebview2Setup.exe" "${WEBVIEW2BOOTSTRAPPERPATH}"
 DetailPrint "$(installingWebview2)"
 StrCpy $6 "$TEMP\MicrosoftEdgeWebview2Setup.exe"
 Goto install_webview2
 !endif

 !if "${INSTALLWEBVIEW2MODE}" == "offlineInstaller"
 Delete "$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe"
 File "/oname=$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe" "${WEBVIEW2INSTALLERPATH}"
 DetailPrint "$(installingWebview2)"
 StrCpy $6 "$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe"
 Goto install_webview2
 !endif

 Goto webview2_done

 install_webview2:
 DetailPrint "$(installingWebview2)"
 ; $6 holds the path to the webview2 installer
 ExecWait "$6 ${WEBVIEW2INSTALLERARGS} /install" $1
 ${If} $1 = 0
 DetailPrint "$(webview2InstallSuccess)"
 ${Else}
 DetailPrint "$(webview2InstallError)"
 Abort "$(webview2AbortError)"
 ${EndIf}
 webview2_done:
 ${EndIf}
 ${Else}
 !if "${MINIMUMWEBVIEW2VERSION}" != ""
 ${VersionCompare} "${MINIMUMWEBVIEW2VERSION}" "$4" $R0
 ${If} $R0 = 1
 update_webview:
 DetailPrint "$(installingWebview2)"
 ${If} ${RunningX64}
 ReadRegStr $R1 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate" "path"
 ${Else}
 ReadRegStr $R1 HKLM "SOFTWARE\Microsoft\EdgeUpdate" "path"
 ${EndIf}
 ${If} $R1 == ""
 ReadRegStr $R1 HKCU "SOFTWARE\Microsoft\EdgeUpdate" "path"
 ${EndIf}
 ${If} $R1 != ""
 ; Chromium updater docs: https://source.chromium.org/chromium/chromium/src/+/main:docs/updater/user_manual.md
 ; Modified from "HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Microsoft EdgeWebView\ModifyPath"
 ExecWait `"$R1" /install appguid=${WEBVIEW2APPGUID}&needsadmin=true` $1
 ${If} $1 = 0
 DetailPrint "$(webview2InstallSuccess)"
 ${Else}
 MessageBox MB_ICONEXCLAMATION|MB_ABORTRETRYIGNORE "$(webview2InstallError)" IDIGNORE ignore IDRETRY update_webview
 Quit
 ignore:
 ${EndIf}
 ${EndIf}
 ${EndIf}
 !endif
 ${EndIf}
SectionEnd

Section Install
 SetOutPath $INSTDIR

 !ifmacrodef NSIS_HOOK_PREINSTALL
 !insertmacro NSIS_HOOK_PREINSTALL
 !endif

 !insertmacro CheckIfAppIsRunning "${MAINBINARYNAME}.exe" "${PRODUCTNAME}"

 ; Copy main executable
 File "${MAINBINARYSRCPATH}"

 ; Copy resources
 {{#each resources_dirs}}
 CreateDirectory "$INSTDIR\\{{this}}"
 {{/each}}
 {{#each resources}}
 File /a "/oname={{this.[1]}}" "{{no-escape @key}}"
 {{/each}}

 ; Copy external binaries
 {{#each binaries}}
 File /a "/oname={{this}}" "{{no-escape @key}}"
 {{/each}}

 ; Create file associations
 {{#each file_associations as |association| ~}}
 {{#each association.ext as |ext| ~}}
 !insertmacro APP_ASSOCIATE "{{ext}}" "{{or association.name ext}}" "{{association-description association.description ext}}" "$INSTDIR\${MAINBINARYNAME}.exe,0" "Open with ${PRODUCTNAME}" "$INSTDIR\${MAINBINARYNAME}.exe $\"%1$\""
 {{/each}}
 {{/each}}

 ; Register deep links
 {{#each deep_link_protocols as |protocol| ~}}
 WriteRegStr SHCTX "Software\Classes\\{{protocol}}" "URL Protocol" ""
 WriteRegStr SHCTX "Software\Classes\\{{protocol}}" "" "URL:${BUNDLEID} protocol"
 WriteRegStr SHCTX "Software\Classes\\{{protocol}}\DefaultIcon" "" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\",0"
 WriteRegStr SHCTX "Software\Classes\\{{protocol}}\shell\open\command" "" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" $\"%1$\""
 {{/each}}

 ; Create uninstaller
 WriteUninstaller "$INSTDIR\uninstall.exe"

 ; Save $INSTDIR in registry for future installations
 WriteRegStr SHCTX "${MANUPRODUCTKEY}" "" $INSTDIR

 ; Stem model install intent (read by the app on first launch).
 ; Always write under HKCU so per-user app can read it without elevation.
 ${If} $StemsCheckboxState == 1
  WriteRegDWORD HKCU "Software\com.soundninja.dev" "WantStemsModel" 1
 ${Else}
  WriteRegDWORD HKCU "Software\com.soundninja.dev" "WantStemsModel" 0
 ${EndIf}

 ${If} $LangChoice == ""
  StrCpy $LangChoice "en"
 ${EndIf}
 WriteRegStr HKCU "Software\com.soundninja.dev" "DefaultLanguage" $LangChoice

 ; Silent/passive installs: default to wanting the model if state was never set
 ${If} $PassiveMode = 1
 ${OrIf} ${Silent}
  ${If} $StemsCheckboxState == ""
   WriteRegDWORD HKCU "Software\com.soundninja.dev" "WantStemsModel" 1
  ${EndIf}
 ${EndIf}

 !if "${INSTALLMODE}" == "both"
 ; Save install mode to be selected by default for the next installation such as updating
 ; or when uninstalling
 WriteRegStr SHCTX "${UNINSTKEY}" $MultiUser.InstallMode 1
 !endif

 ; Remove old main binary if it doesn't match new main binary name
 ReadRegStr $OldMainBinaryName SHCTX "${UNINSTKEY}" "MainBinaryName"
 ${If} $OldMainBinaryName != ""
 ${AndIf} $OldMainBinaryName != "${MAINBINARYNAME}.exe"
 Delete "$INSTDIR\$OldMainBinaryName"
 ${EndIf}

 ; Save current MAINBINARYNAME for future updates
 WriteRegStr SHCTX "${UNINSTKEY}" "MainBinaryName" "${MAINBINARYNAME}.exe"

 ; Registry information for add/remove programs
 WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "${PRODUCTNAME}"
 WriteRegStr SHCTX "${UNINSTKEY}" "DisplayIcon" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\""
 WriteRegStr SHCTX "${UNINSTKEY}" "DisplayVersion" "${VERSION}"
 WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "${MANUFACTURER}"
 WriteRegStr SHCTX "${UNINSTKEY}" "InstallLocation" "$\"$INSTDIR$\""
 WriteRegStr SHCTX "${UNINSTKEY}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
 WriteRegDWORD SHCTX "${UNINSTKEY}" "NoModify" "1"
 WriteRegDWORD SHCTX "${UNINSTKEY}" "NoRepair" "1"

 ${GetSize} "$INSTDIR" "/M=uninstall.exe /S=0K /G=0" $0 $1 $2
 IntOp $0 $0 + ${ESTIMATEDSIZE}
 IntFmt $0 "0x%08X" $0
 WriteRegDWORD SHCTX "${UNINSTKEY}" "EstimatedSize" "$0"

 !if "${HOMEPAGE}" != ""
 WriteRegStr SHCTX "${UNINSTKEY}" "URLInfoAbout" "${HOMEPAGE}"
 WriteRegStr SHCTX "${UNINSTKEY}" "URLUpdateInfo" "${HOMEPAGE}"
 WriteRegStr SHCTX "${UNINSTKEY}" "HelpLink" "${HOMEPAGE}"
 !endif

 ; Create start menu shortcut
 !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
 Call CreateOrUpdateStartMenuShortcut
 !insertmacro MUI_STARTMENU_WRITE_END

 ; Create desktop shortcut for silent and passive installers
 ; because finish page will be skipped
 ${If} $PassiveMode = 1
 ${OrIf} ${Silent}
 Call CreateOrUpdateDesktopShortcut
 ${EndIf}

 !ifmacrodef NSIS_HOOK_POSTINSTALL
 !insertmacro NSIS_HOOK_POSTINSTALL
 !endif

 ; Auto close this page for passive mode
 ${If} $PassiveMode = 1
 SetAutoClose true
 ${EndIf}
SectionEnd

Function .onInstSuccess
 ; Check for `/R` flag only in silent and passive installers because
 ; GUI installer has a toggle for the user to (re)start the app
 ${If} $PassiveMode = 1
 ${OrIf} ${Silent}
 ${GetOptions} $CMDLINE "/R" $R0
 ${IfNot} ${Errors}
 ${GetOptions} $CMDLINE "/ARGS" $R0
 nsis_tauri_utils::RunAsUser "$INSTDIR\${MAINBINARYNAME}.exe" "$R0"
 ${EndIf}
 ${EndIf}
FunctionEnd

Function un.onInit
 !insertmacro SetContext

 !if "${INSTALLMODE}" == "both"
 !insertmacro MULTIUSER_UNINIT
 !endif

 !insertmacro MUI_UNGETLANGUAGE

 ${GetOptions} $CMDLINE "/P" $PassiveMode
 ${IfNot} ${Errors}
 StrCpy $PassiveMode 1
 ${EndIf}

 ${GetOptions} $CMDLINE "/UPDATE" $UpdateMode
 ${IfNot} ${Errors}
 StrCpy $UpdateMode 1
 ${EndIf}
FunctionEnd

Section Uninstall

 !ifmacrodef NSIS_HOOK_PREUNINSTALL
 !insertmacro NSIS_HOOK_PREUNINSTALL
 !endif

 !insertmacro CheckIfAppIsRunning "${MAINBINARYNAME}.exe" "${PRODUCTNAME}"

 ; Delete the app directory and its content from disk
 ; Copy main executable
 Delete "$INSTDIR\${MAINBINARYNAME}.exe"

 ; Delete resources
 {{#each resources}}
 Delete "$INSTDIR\\{{this.[1]}}"
 {{/each}}

 ; Delete external binaries
 {{#each binaries}}
 Delete "$INSTDIR\\{{this}}"
 {{/each}}

 ; Delete app associations
 {{#each file_associations as |association| ~}}
 {{#each association.ext as |ext| ~}}
 !insertmacro APP_UNASSOCIATE "{{ext}}" "{{or association.name ext}}"
 {{/each}}
 {{/each}}

 ; Delete deep links
 {{#each deep_link_protocols as |protocol| ~}}
 ReadRegStr $R7 SHCTX "Software\Classes\\{{protocol}}\shell\open\command" ""
 ${If} $R7 == "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" $\"%1$\""
 DeleteRegKey SHCTX "Software\Classes\\{{protocol}}"
 ${EndIf}
 {{/each}}

 ; Delete uninstaller
 Delete "$INSTDIR\uninstall.exe"

 {{#each resources_ancestors}}
 RMDir /REBOOTOK "$INSTDIR\\{{this}}"
 {{/each}}
 RMDir "$INSTDIR"

 ; Remove shortcuts if not updating
 ${If} $UpdateMode <> 1
 !insertmacro DeleteAppUserModelId

 ; Remove start menu shortcut
 !insertmacro MUI_STARTMENU_GETFOLDER Application $AppStartMenuFolder
 !insertmacro IsShortcutTarget "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
 Pop $0
 ${If} $0 = 1
 !insertmacro UnpinShortcut "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
 Delete "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
 RMDir "$SMPROGRAMS\$AppStartMenuFolder"
 ${EndIf}
 !insertmacro IsShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
 Pop $0
 ${If} $0 = 1
 !insertmacro UnpinShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk"
 Delete "$SMPROGRAMS\${PRODUCTNAME}.lnk"
 ${EndIf}

 ; Remove desktop shortcuts
 !insertmacro IsShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
 Pop $0
 ${If} $0 = 1
 !insertmacro UnpinShortcut "$DESKTOP\${PRODUCTNAME}.lnk"
 Delete "$DESKTOP\${PRODUCTNAME}.lnk"
 ${EndIf}
 Call un.DeleteLegacySoundninjaShortcuts
 ${EndIf}

 ; Remove registry information for add/remove programs
 !if "${INSTALLMODE}" == "both"
 DeleteRegKey SHCTX "${UNINSTKEY}"
 !else if "${INSTALLMODE}" == "perMachine"
 DeleteRegKey HKLM "${UNINSTKEY}"
 !else
 DeleteRegKey HKCU "${UNINSTKEY}"
 !endif

 ; Removes the Autostart entry for ${PRODUCTNAME} from the HKCU Run key if it exists.
 ; This ensures the program does not launch automatically after uninstallation if it exists.
 ; If it doesn't exist, it does nothing.
 ; We do this when not updating (to preserve the registry value on updates)
 ${If} $UpdateMode <> 1
 DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}"
 ${EndIf}

 ; Delete app data if the checkbox is selected
 ; and if not updating
 ${If} $DeleteAppDataCheckboxState = 1
 ${AndIf} $UpdateMode <> 1
 ; Clear the install location $INSTDIR from registry
 DeleteRegKey SHCTX "${MANUPRODUCTKEY}"
 DeleteRegKey /ifempty SHCTX "${MANUKEY}"

 ; Clear the install language from registry
 DeleteRegValue HKCU "${MANUPRODUCTKEY}" "Installer Language"
 DeleteRegKey /ifempty HKCU "${MANUPRODUCTKEY}"
 DeleteRegKey /ifempty HKCU "${MANUKEY}"

 SetShellVarContext current
 RmDir /r "$APPDATA\${BUNDLEID}"
 RmDir /r "$LOCALAPPDATA\${BUNDLEID}"
 ${EndIf}

 !ifmacrodef NSIS_HOOK_POSTUNINSTALL
 !insertmacro NSIS_HOOK_POSTUNINSTALL
 !endif

 ; Auto close if passive mode or updating
 ${If} $PassiveMode = 1
 ${OrIf} $UpdateMode = 1
 SetAutoClose true
 ${EndIf}
SectionEnd

Function RestorePreviousInstallLocation
 Call ResolveOldInstallDir
 ${If} $4 != ""
  StrCpy $INSTDIR $4
 ${EndIf}
FunctionEnd

!macro DeleteLegacySoundninjaShortcutsImpl
 ${If} ${FileExists} "$SMPROGRAMS\soundninja.lnk"
  !insertmacro UnpinShortcut "$SMPROGRAMS\soundninja.lnk"
  Delete "$SMPROGRAMS\soundninja.lnk"
 ${EndIf}
 ${If} ${FileExists} "$DESKTOP\soundninja.lnk"
  !insertmacro UnpinShortcut "$DESKTOP\soundninja.lnk"
  Delete "$DESKTOP\soundninja.lnk"
 ${EndIf}
 ${If} "$AppStartMenuFolder" != ""
  ${If} ${FileExists} "$SMPROGRAMS\$AppStartMenuFolder\soundninja.lnk"
   !insertmacro UnpinShortcut "$SMPROGRAMS\$AppStartMenuFolder\soundninja.lnk"
   Delete "$SMPROGRAMS\$AppStartMenuFolder\soundninja.lnk"
  ${EndIf}
 ${EndIf}
!macroend

Function DeleteLegacySoundninjaShortcuts
 !insertmacro DeleteLegacySoundninjaShortcutsImpl
FunctionEnd
Function un.DeleteLegacySoundninjaShortcuts
 !insertmacro DeleteLegacySoundninjaShortcutsImpl
FunctionEnd

Function Skip
 Abort
FunctionEnd

Function SkipIfPassive
 ${IfThen} $PassiveMode = 1 ${|} Abort ${|}
FunctionEnd
Function un.SkipIfPassive
 ${IfThen} $PassiveMode = 1 ${|} Abort ${|}
FunctionEnd

Function CreateOrUpdateStartMenuShortcut
 ; We used to use product name as MAINBINARYNAME
 ; migrate old shortcuts to target the new MAINBINARYNAME
 StrCpy $R0 0

 !insertmacro IsShortcutTarget "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
 Pop $0
 ${If} $0 = 1
 !insertmacro SetShortcutTarget "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
 StrCpy $R0 1
 ${EndIf}

 !insertmacro IsShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
 Pop $0
 ${If} $0 = 1
 !insertmacro SetShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
 StrCpy $R0 1
 ${EndIf}

 ${If} $R0 = 1
 Call DeleteLegacySoundninjaShortcuts
 Return
 ${EndIf}

 ; Skip creating shortcut if in update mode or no shortcut mode
 ; but always create if migrating from wix
 ${If} $WixMode = 0
 ${If} $UpdateMode = 1
 ${OrIf} $NoShortcutMode = 1
 Return
 ${EndIf}
 ${EndIf}

 !if "${STARTMENUFOLDER}" != ""
 CreateDirectory "$SMPROGRAMS\$AppStartMenuFolder"
 CreateShortcut "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
 !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
 !else
 CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
 !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\${PRODUCTNAME}.lnk"
 !endif
 Call DeleteLegacySoundninjaShortcuts
FunctionEnd

Function CreateOrUpdateDesktopShortcut
 ; We used to use product name as MAINBINARYNAME
 ; migrate old shortcuts to target the new MAINBINARYNAME
 !insertmacro IsShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
 Pop $0
 ${If} $0 = 1
 !insertmacro SetShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
 Call DeleteLegacySoundninjaShortcuts
 Return
 ${EndIf}

 ; Skip creating shortcut if in update mode or no shortcut mode
 ; but always create if migrating from wix
 ${If} $WixMode = 0
 ${If} $UpdateMode = 1
 ${OrIf} $NoShortcutMode = 1
 Return
 ${EndIf}
 ${EndIf}

 CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
 !insertmacro SetLnkAppUserModelId "$DESKTOP\${PRODUCTNAME}.lnk"
 Call DeleteLegacySoundninjaShortcuts
FunctionEnd
