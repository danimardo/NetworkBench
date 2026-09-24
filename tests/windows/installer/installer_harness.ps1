# Arnés del instalador NSIS (T117, T128).
#
# Comprueba lo que se puede comprobar sin elevar: que el instalador existe, que lleva
# lo que debe llevar y que su configuración dice lo que debe decir.
#
# Su versión anterior afirmaba «7/7 pasadas» sobre variables fijadas a $true en el
# propio script, incluida la preservación de datos de usuario al desinstalar. Nada de
# eso se ejecutaba. Lo que de verdad exige instalar y desinstalar se declara PENDIENTE
# y sale con código 2, no con éxito.

param ([switch]$VerboseOutput = $false)

$ErrorActionPreference = "Stop"
# tests/windows/installer -> raíz del repositorio
$Raiz = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
Set-Location $Raiz

$Triple = "x86_64-pc-windows-msvc"
$Release = "src-tauri\target\$Triple\release"

$Total = 0
$Ok = 0
$Pendientes = @()

function Comprobar {
    param([string]$Nombre, [scriptblock]$Prueba, [string]$Detalle = "")
    $script:Total++
    $resultado = $false
    try { $resultado = & $Prueba } catch { $resultado = $false }
    if ($resultado) {
        $script:Ok++
        Write-Host "  [OK]   $Nombre" -ForegroundColor Green
    } else {
        Write-Host "  [FALLO] $Nombre" -ForegroundColor Red
        if ($Detalle) { Write-Host "          $Detalle" -ForegroundColor Yellow }
    }
}

function Pendiente {
    param([string]$Nombre, [string]$Motivo)
    $script:Pendientes += $Nombre
    Write-Host "  [PENDIENTE] $Nombre" -ForegroundColor Yellow
    Write-Host "              $Motivo" -ForegroundColor Gray
}

Write-Host "=== Arnés del instalador NSIS ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Comprobable sin elevar:" -ForegroundColor Cyan

$instalador = Get-ChildItem "$Release\bundle\nsis\*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1

Comprobar "El instalador existe" { $null -ne $instalador } `
    "Constrúyelo con: node scripts/package/release.mjs"

if ($instalador) {
    # WebView2 offline pesa unos 150 MB. Un instalador de pocos MB significa que se
    # coló un bootstrapper que exigiría red al instalar (FR-062, SC-014).
    $mib = [math]::Round($instalador.Length / 1MB, 1)
    Comprobar "Lleva el runtime WebView2 embebido ($mib MiB)" { $instalador.Length -gt 100MB } `
        "Con menos de 100 MiB la instalación exigiría red"
}

Comprobar "El motor queda junto a la aplicación" { Test-Path "$Release\ntttcp.exe" }
Comprobar "La aplicación principal se construyó" { Test-Path "$Release\NetworkBench.exe" }
Comprobar "El helper elevado está empaquetado como recurso" {
    (Test-Path "$Release\resources\networkbench-firewall-helper.exe") -or
    (Test-Path "$Release\networkbench-firewall-helper.exe")
}

if (Test-Path "$Release\ntttcp.exe") {
    $esperado = (Get-Content "engine\SHA256" -Raw).Trim()
    $real = (Get-FileHash -Algorithm SHA256 "$Release\ntttcp.exe").Hash.ToLower()
    Comprobar "La integridad del motor empaquetado coincide" { $real -eq $esperado } `
        "esperado $esperado, obtenido $real"
}

$conf = Get-Content "src-tauri\tauri.conf.json" -Raw
Comprobar "Instalación por máquina (perMachine)" { $conf -match '"installMode"\s*:\s*"perMachine"' }
Comprobar "WebView2 en modo offline" { $conf -match '"type"\s*:\s*"offlineInstaller"' }
Comprobar "Instalador bilingüe español e inglés" {
    ($conf -match '"Spanish"') -and ($conf -match '"English"')
}

$hooks = Get-Content "src-tauri\nsis\hooks.nsh" -Raw -ErrorAction SilentlyContinue
Comprobar "Los hooks de desinstalación existen" { $null -ne $hooks }
if ($hooks) {
    # Por grupo, no por nombre suelto: así la orden no puede alcanzar una regla ajena.
    Comprobar "Las reglas de cortafuegos se retiran por grupo" { $hooks -match 'delete rule group="NetworkBench"' }
    Comprobar "El autoarranque se retira" { $hooks -match 'CurrentVersion\\Run' }
    Comprobar "Los datos de usuario NO se borran al desinstalar" {
        ($hooks -notmatch 'RMDir\s+/r.*LOCALAPPDATA') -and ($hooks -match 'conservan')
    }
}

Write-Host ""
Write-Host "Requiere elevación y una máquina limpia:" -ForegroundColor Cyan
Pendiente "Instalación real sin red" "Exige ejecutar el instalador con privilegios de administrador"
Pendiente "Primer arranque tras instalar" "Exige una sesión de escritorio"
Pendiente "Actualización sobre una versión anterior" "Exige dos artefactos y una instalación previa"
Pendiente "Desinstalación preservando %LOCALAPPDATA%\NetworkBench" "Exige instalar primero"
Pendiente "Matriz Windows 10 22H2" "No hay una segunda máquina en este entorno"

Write-Host ""
Write-Host "=== Resumen: $Ok/$Total comprobables superadas, $($Pendientes.Count) pendientes ===" -ForegroundColor Cyan
Write-Host "Las pendientes NO cuentan como aprobadas." -ForegroundColor Yellow

if ($Ok -ne $Total) { exit 1 }
if ($Pendientes.Count -gt 0) { exit 2 }
exit 0
