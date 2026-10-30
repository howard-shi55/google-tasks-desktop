!include "LogicLib.nsh"

!macro NSIS_HOOK_POSTUNINSTALL
  ; Remove only our startup value on uninstall; updater-mode reinstalls retain it.
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "GoogleTasksDesktop"
  ${EndIf}
!macroend
