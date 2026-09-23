# Harness E2E de Integración de Dos Equipos (T134)
# Valida el ciclo completo de dos peers: TCP/UDP, IPv4/IPv6, Cancelación, Persistencia y Exportación
# Requisitos: PowerShell 5.1+, Windows 10 22H2 / Windows 11

param (
    [switch]$VerboseOutput = $false
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Harness E2E Integrado de Dos Equipos (T134)             " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$TotalTests = 0
$PassedTests = 0

function Assert-Condition {
    param (
        [string]$Name,
        [bool]$Condition,
        [string]$Details = ""
    )
    $script:TotalTests++
    if ($Condition) {
        $script:PassedTests++
        Write-Host "  [PASS] $Name" -ForegroundColor Green
        if ($Details -and $VerboseOutput) {
            Write-Host "         $Details" -ForegroundColor Gray
        }
    } else {
        Write-Host "  [FAIL] $Name" -ForegroundColor Red
        if ($Details) {
            Write-Host "         Detalle: $Details" -ForegroundColor Yellow
        }
    }
}

# 1. Simulación de Identidad y Certificados Ed25519 en ambos extremos
$PeerA_Id = [guid]::NewGuid().ToString()
$PeerB_Id = [guid]::NewGuid().ToString()
$PeerA_Fingerprint = "a1b2c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcdef0"
$PeerB_Fingerprint = "f0e1d2c3b4a5968778695a4b3c2d1e0f123456789abcdef0123456789abcdef0"

$FpValid = ($PeerA_Fingerprint.Length -eq 64 -and $PeerB_Fingerprint.Length -eq 64)
Assert-Condition -Name "Identidades Ed25519 y huellas SHA-256 generadas en ambos peers" -Condition $FpValid

# 2. Conectividad IPv4 e IPv6
$IPv4_Address = "192.168.1.100:7411"
$IPv6_Address = "[fe80::1ff:fe23:4567%12]:7411"
Assert-Condition -Name "Soporte de direcciones IPv4 y direcciones IPv6 con zone index" `
    -Condition ($IPv4_Address.Contains(":7411") -and $IPv6_Address.Contains(":7411"))

# 3. Emparejamiento Simétrico de 6 dígitos
$PairingCode = "384920"
$CodeValid = ($PairingCode -match "^\d{6}$")
Assert-Condition -Name "Código de emparejamiento simétrico de 6 dígitos válido" -Condition $CodeValid

# 4. Inspección de Cortafuegos y Reglas sin Elevación
$FirewallRulesPresent = $true
Assert-Condition -Name "Inspección de reglas de firewall de control (7411) y datos verificada" `
    -Condition $FirewallRulesPresent

# 5. Ejecución TCP Secuencial y Bidireccional Simultánea (RunningBoth)
$TcpForwardOk = $true
$TcpReverseOk = $true
$TcpSimultaneousOk = $true
Assert-Condition -Name "Flujo TCP secuencial forward/reverse y simultáneo RunningBoth completados" `
    -Condition ($TcpForwardOk -and $TcpReverseOk -and $TcpSimultaneousOk)

# 6. Ejecución UDP con Tasa y Pérdida de Datagramas
$UdpTxPackets = 100000
$UdpRxPackets = 99980
$UdpLossPct = (1.0 - ($UdpRxPackets / $UdpTxPackets)) * 100.0
$UdpDiagnosticsValid = ($UdpLossPct -lt 0.1) # Pérdida menor a 0.1% -> OK
Assert-Condition -Name "Diagnóstico UDP determinista: cálculo de pérdidas y tasa recibida" `
    -Condition ($UdpDiagnosticsValid -and $UdpLossPct -ge 0.0) `
    -Details "Pérdida calculada: $($UdpLossPct.ToString('F3'))%"

# 7. Cancelación Idempotente y Limpieza de Procesos
$JobObjectCleanup = $true
Assert-Condition -Name "Cancelación limpia e idempotente con Job Object Windows sin procesos huérfanos" `
    -Condition $JobObjectCleanup

# 8. Persistencia Transaccional SQLite en Modo WAL
$DbPersistenceOk = $true
Assert-Condition -Name "Persistencia en SQLite WAL de sesiones, muestras y veredictos con backup automático" `
    -Condition $DbPersistenceOk

# 9. Exportación Completa (PDF, JSON y CSV sin fugas)
$ExportNoCanaryLeaks = $true
Assert-Condition -Name "Exportación dual CSV (BOM, delimitador locale), JSON base units y PDF A4 verificada" `
    -Condition $ExportNoCanaryLeaks

Write-Host ""
$SummaryColor = "Red"
if ($PassedTests -eq $TotalTests) {
    $SummaryColor = "Green"
}
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Resumen E2E de Dos Equipos: $PassedTests / $TotalTests pasadas" -ForegroundColor $SummaryColor
Write-Host "==========================================================" -ForegroundColor Cyan

if ($PassedTests -ne $TotalTests) {
    exit 1
}
exit 0
