# Arnés E2E de dos equipos (T134).
#
# ESTE ARNÉS NO MIDE NADA TODAVÍA.
#
# Su versión anterior imprimía «9/9 pasadas» evaluando variables que el propio script
# fijaba a $true unas líneas antes. Eso no es una prueba: es una afirmación disfrazada
# de resultado, y así llegó a VALIDACION.md como evidencia de un ciclo completo entre
# dos equipos que nunca se ejecutó.
#
# Ahora enumera lo que haría falta comprobar y sale con código 2, que significa
# PENDIENTE. Un arnés que no puede ejecutar su escenario debe decirlo, no aprobarlo.
#
# Para ejecutarlo de verdad hacen falta dos equipos Windows en la misma red, con el
# instalador puesto en ambos, y los parámetros -PeerRemoto y -PeerLocal.

param (
    [string]$PeerRemoto = "",
    [string]$PeerLocal = "",
    [switch]$VerboseOutput = $false
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Arnés E2E de dos equipos (T134)                          " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$Escenarios = @(
    "Emparejamiento con verificación humana de la huella (FR-012)",
    "Identidad cambiada que invalida la confianza previa (FR-013)",
    "Conectividad IPv4 e IPv6, incluida dirección con zona",
    "Inspección y creación de reglas de cortafuegos con UAC aceptado y rechazado",
    "Medición TCP secuencial en ambos sentidos con resultado común (US1)",
    "Medición UDP con pérdida, tasa objetivo y tasa real (US5)",
    "Cancelación desde cualquier estado, sin procesos huérfanos (FR-023)",
    "Persistencia SQLite en WAL y reapertura tras reinicio (FR-045)",
    "Exportación PDF, JSON y CSV sin fuga de identificadores (US6)"
)

if (-not $PeerRemoto -or -not $PeerLocal) {
    Write-Host ""
    Write-Host "PENDIENTE: este escenario exige dos equipos y no se ha ejecutado." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Comprobaciones que quedan sin cubrir:" -ForegroundColor Yellow
    foreach ($e in $Escenarios) {
        Write-Host "  [PENDIENTE] $e" -ForegroundColor Yellow
    }
    Write-Host ""
    Write-Host "Uso:" -ForegroundColor Gray
    Write-Host "  .\e2e_two_peers_harness.ps1 -PeerLocal <ip-local> -PeerRemoto <ip-remota>" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Ninguna de estas líneas es un resultado. No las registres como evidencia." -ForegroundColor Yellow
    exit 2
}

Write-Host ""
Write-Host "ERROR: la ejecución real contra dos equipos no está implementada." -ForegroundColor Red
Write-Host "Se recibieron -PeerLocal '$PeerLocal' y -PeerRemoto '$PeerRemoto', pero el" -ForegroundColor Red
Write-Host "arnés todavía no sabe conducir la aplicación. Implementarlo es parte de T134." -ForegroundColor Red
exit 1
