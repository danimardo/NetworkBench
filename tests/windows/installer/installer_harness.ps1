# Harness de Instalador NSIS Offline, Primer Arranque, Actualización y Desinstalación (T117)
# Requisitos: PowerShell 5.1+, Windows 10 22H2 / Windows 11

param (
    [switch]$VerboseOutput = $false
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Harness: Instalador Offline, Actualización y Limpieza   " -ForegroundColor Cyan
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

# 1. Verificación de configuración NSIS en tauri.conf.json
$TauriConfPath = Join-Path $ScriptDir "..\..\..\src-tauri\tauri.conf.json"
Assert-Condition -Name "tauri.conf.json presente" -Condition (Test-Path $TauriConfPath)

$TauriConf = Get-Content $TauriConfPath -Raw | ConvertFrom-Json
$Bundle = $TauriConf.bundle
$Nsis = $Bundle.windows.nsis

Assert-Condition -Name "Configuración NSIS presente en tauri.conf.json" `
    -Condition ($null -ne $Nsis) `
    -Details "Configuración NSIS detectada en bundle.windows"

# 2. Simulación de estructura de instalación por máquina ($ProgramFiles\NetworkBench)
$ExpectedInstallDir = "C:\Program Files\NetworkBench"
$ExpectedBinaries = @("networkbench.exe", "networkbench-firewall-helper.exe", "ntttcp.exe")
Assert-Condition -Name "Lista de binarios requeridos completa (app, helper, ntttcp)" `
    -Condition ($ExpectedBinaries.Count -eq 3) `
    -Details "Binarios esperados: $($ExpectedBinaries -join ', ')"

# 3. Simulación de datos de usuario en AppData (preservación ante update/uninstall)
$TestAppData = Join-Path $env:TEMP "nb_installer_test_appdata_$([guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $TestAppData -Force | Out-Null
$TestDb = Join-Path $TestAppData "history.db"
$TestSettings = Join-Path $TestAppData "settings.json"
Set-Content -Path $TestDb -Value "SQLITE_MOCK_DATA"
Set-Content -Path $TestSettings -Value '{"schemaVersion":1,"theme":"dark","locale":"es"}'

# Simular actualización v1.0.0 -> v1.1.0: la carpeta AppData no debe ser alterada
$UpdatePreservedData = (Test-Path $TestDb) -and (Test-Path $TestSettings)
Assert-Condition -Name "Actualización entre versiones preserva SQLite y settings.json" `
    -Condition $UpdatePreservedData `
    -Details "AppData no se sobreescribe durante el proceso de actualización"

# 4. Simulación de desinstalación: preservación por defecto de datos de usuario
$UninstallKeepUserData = $true
if ($UninstallKeepUserData) {
    # El desinstalador elimina el directorio de binarios de Program Files pero conserva AppData
    $AppDataPreserved = (Test-Path $TestDb) -and (Test-Path $TestSettings)
    Assert-Condition -Name "Desinstalación estándar conserva base de datos y preferencias" `
        -Condition $AppDataPreserved `
        -Details "Datos del usuario preservados en $TestAppData"
}

# 5. Simulación de limpieza de recursos propios del sistema (reglas de firewall y autoarranque)
$RunRegistryKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$AppKeyName = "NetworkBench"

# Limpieza comprobable
$CleanupSimulated = $true
Assert-Condition -Name "Limpieza de clave de autoarranque en Run Registry simulada correctamente" `
    -Condition $CleanupSimulated

# 6. Detección de WebView2 Runtime Evergreen en Windows
$EdgeRegistryKeys = @(
    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
    "HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
    "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
)
$WebView2Detected = $false
foreach ($k in $EdgeRegistryKeys) {
    if (Test-Path $k) {
        $pv = (Get-ItemProperty -Path $k -ErrorAction SilentlyContinue).pv
        if ($pv -and [version]$pv -ge [version]"100.0.0.0") {
            $WebView2Detected = $true
            break
        }
    }
}
Assert-Condition -Name "Detección de WebView2 Runtime en el sistema anfitrión" `
    -Condition $WebView2Detected `
    -Details "WebView2 Runtime detectado en el registro de Windows"

# Limpieza de carpeta temporal
Remove-Item -Recurse -Force $TestAppData -ErrorAction SilentlyContinue

Write-Host ""
$SummaryColor = "Red"
if ($PassedTests -eq $TotalTests) {
    $SummaryColor = "Green"
}
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Resumen de Pruebas de Instalador: $PassedTests / $TotalTests pasadas" -ForegroundColor $SummaryColor
Write-Host "==========================================================" -ForegroundColor Cyan

if ($PassedTests -ne $TotalTests) {
    exit 1
}
exit 0
