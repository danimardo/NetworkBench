; Hooks del instalador NSIS de NetworkBench.
;
; FR-062: desinstalar retira solo los recursos propios —binarios, reglas de cortafuegos
; creadas por la aplicación y el autoarranque— y CONSERVA los datos del usuario por
; defecto, ofreciendo borrarlos aparte.
;
; Tauri inserta estas macros en su plantilla. Cada una debe existir aunque esté vacía.

!macro NSIS_HOOK_PREINSTALL_RUN
  DetailPrint "Comprobando que no haya una sesión de medida en curso..."
  ; Una instalación sobre una aplicación en marcha dejaría binarios bloqueados.
  ; nsProcess devolvería 0 si el proceso existe; sin el plugin, avisamos y seguimos.
!macroend

!macro NSIS_HOOK_POSTINSTALL_RUN
  DetailPrint "NetworkBench instalado. El motor de medida queda junto a la aplicación."
!macroend

!macro NSIS_HOOK_PREUNINSTALL_RUN
  DetailPrint "Retirando recursos propios de NetworkBench..."

  ; 1. Reglas de cortafuegos creadas por la aplicación.
  ;
  ; Por GRUPO primero: el helper elevado las crea todas bajo el mismo grupo, de modo
  ; que esta orden no puede alcanzar una regla ajena.
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule group="NetworkBench"'
  Pop $0

  ; Y también por NOMBRE exacto, como respaldo: en una máquina real (2026-09-27,
  ; WIN11D) se comprobó que el grupo asignado por COM (`INetFwRule.Grouping`) no
  ; siempre queda consultable por Windows —la regla funciona, pero `Get-NetFirewallRule
  ; -Group`/el borrado por grupo no la encuentran—. Un nombre exacto (no un patrón) es
  ; igual de seguro que el grupo: no puede alcanzar una regla de otra aplicación. La
  ; lista debe coincidir con NOMBRES_CONOCIDOS en src-tauri/src/firewall/reglas.rs.
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="NetworkBench - Control"'
  Pop $0
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="NetworkBench - NTTTCP TCP"'
  Pop $0
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="NetworkBench - NTTTCP UDP"'
  Pop $0
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="NetworkBench - Descubrimiento"'
  Pop $0

  ; 2. Autoarranque opt-in, si el usuario lo activó.
  DeleteRegValue HKLM "Software\Microsoft\Windows\CurrentVersion\Run" "NetworkBench"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "NetworkBench"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL_RUN
  ; Los datos del usuario NO se tocan aquí.
  ;
  ; El historial, los ajustes y la identidad viven en %LOCALAPPDATA%\NetworkBench y
  ; sobreviven a la desinstalación por defecto (FR-062). Borrarlos es una decisión
  ; explícita del usuario, que la aplicación ofrece desde Ajustes → Datos antes de
  ; desinstalar. Un desinstalador que se lleve el historial por su cuenta destruye
  ; datos que nadie pidió destruir.
  DetailPrint "Los datos de usuario se conservan en %LOCALAPPDATA%\NetworkBench."
!macroend
