<#
.SYNOPSIS
  Harness de rendimiento V-04 / V-12 para NetworkBench y WebView2 (T159, T168).
.DESCRIPTION
  Su primera versión no lanzaba la aplicación: muestreaba procesos llamados
  "NetworkBench"/"msedgewebview2" que no existían, y nunca calculaba CPU.

  Esta versión sigue el método que ya fija la constitución (`.specify/memory/constitution.md`,
  principio VII), no un valor inventado aquí: presupuesto "< 5 % de UN procesador lógico"
  —fórmula `100 × suma(delta CPU usuario+sistema) / delta tiempo real`, SIN dividir por el
  número de núcleos—, sumando la app y los procesos WebView2 atribuibles, **excluyendo
  NTTTCP**, con el hardware documentado y varias ejecuciones por escenario (con/sin
  animación). T168 pedía fijar un "núcleo de referencia": no hacía falta inventar un
  modelo de CPU — la constitución ya normaliza a "un procesador lógico" y exige documentar
  el hardware real de cada ejecución en vez de fijar un modelo cerrado.

  Mide la aplicación **en reposo** (ventana abierta, sin sesión `RUNNING_*` activa):
  lanzar una medición real necesita un segundo equipo o al menos otra instancia local
  emparejada, y este script no lo hace. Sigue siendo una comprobación parcial de V-04 (el
  presupuesto es "durante la prueba") y de V-12.
.PARAMETER ExePath
  Ruta al ejecutable de NetworkBench.
.PARAMETER DurationSeconds
  Ventana de medición de CPU/memoria por ejecución.
.PARAMETER Runs
  Ejecuciones por escenario (la constitución pide cinco).
.PARAMETER ReduceMotion
  Si se pasa, fuerza `reduceMotion: true` en `settings.json` antes de medir (escenario
  "sin animación"). Sin el switch, mide con el valor por defecto (`false`, "con animación").
#>

param(
  [string]$ExePath = "src-tauri/target/x86_64-pc-windows-msvc/debug/NetworkBench.exe",
  [int]$DurationSeconds = 10,
  [int]$StartupWaitSeconds = 5,
  [int]$Runs = 5,
  [switch]$ReduceMotion
)

$ErrorActionPreference = "Stop"

function Get-ProcessTree($rootId) {
  $all = @($rootId)
  $children = Get-CimInstance Win32_Process -Filter "ParentProcessId=$rootId" -ErrorAction SilentlyContinue
  foreach ($c in $children) { $all += Get-ProcessTree $c.ProcessId }
  return $all
}

function Get-SettingsPath {
  if ($env:LOCALAPPDATA) { return Join-Path $env:LOCALAPPDATA "NetworkBench\settings.json" }
  return Join-Path $env:TEMP "NetworkBench\settings.json"
}

function Set-ReduceMotionSetting([bool]$value) {
  $path = Get-SettingsPath
  if (-not (Test-Path $path)) {
    Write-Host "AVISO: no existe '$path' todavía; arrancando una vez para generarlo..." -ForegroundColor Yellow
    $warm = Start-Process -FilePath (Resolve-Path $ExePath) -PassThru
    Start-Sleep -Seconds 3
    Stop-Process -Id $warm.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
  }
  if (-not (Test-Path $path)) {
    Write-Host "PENDIENTE: '$path' sigue sin existir; no se puede fijar reduceMotion." -ForegroundColor Yellow
    return $false
  }
  $json = Get-Content $path -Raw | ConvertFrom-Json
  $json.reduceMotion = $value
  ($json | ConvertTo-Json -Depth 10) | Set-Content -Path $path -Encoding utf8
  return $true
}

function Get-EnvironmentReport {
  $cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
  $gpu = Get-CimInstance Win32_VideoController | Select-Object -First 1
  $os = Get-CimInstance Win32_OperatingSystem
  $ramGB = [Math]::Round($os.TotalVisibleMemorySize / 1MB, 1)
  $nic = Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object Status -eq "Up" | Select-Object -First 1
  $powerScheme = (powercfg /getactivescheme 2>$null)
  $webview2Version = $null
  try {
    $key = "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
    $webview2Version = (Get-ItemProperty -Path $key -ErrorAction Stop).pv
  } catch {}
  $screen = Get-CimInstance Win32_VideoController | Select-Object -First 1 -ExpandProperty CurrentHorizontalResolution
  $screenV = Get-CimInstance Win32_VideoController | Select-Object -First 1 -ExpandProperty CurrentVerticalResolution

  [PSCustomObject]@{
    Cpu             = "$($cpu.Name) ($($cpu.NumberOfCores) nucleos fisicos / $($cpu.NumberOfLogicalProcessors) logicos, $($cpu.MaxClockSpeed) MHz)"
    Gpu             = $gpu.Name
    RamGB           = $ramGB
    Nic             = if ($nic) { "$($nic.InterfaceDescription) ($($nic.LinkSpeed))" } else { "no detectada" }
    Windows         = "$($os.Caption) $($os.Version)"
    WebView2        = if ($webview2Version) { $webview2Version } else { "no detectado" }
    Resolucion      = "${screen}x${screenV}"
    PlanDeEnergia   = ($powerScheme -join " ").Trim()
  }
}

Write-Host "=== Harness de rendimiento V-04/V-12 NetworkBench (T159/T168) ===" -ForegroundColor Cyan

if (-not (Test-Path $ExePath)) {
  Write-Host "PENDIENTE: no existe '$ExePath'." -ForegroundColor Yellow
  Write-Host "Compila primero con 'cargo build --manifest-path src-tauri/Cargo.toml' (o --release),"
  Write-Host "y comprueba que 'pnpm build' generó dist/, o pasa -ExePath a otro binario."
  exit 2
}

$exeDir = Split-Path -Parent (Resolve-Path $ExePath)
if (-not (Test-Path (Join-Path $exeDir "ntttcp.exe"))) {
  Write-Host "AVISO: no hay ntttcp.exe junto al ejecutable ($exeDir)." -ForegroundColor Yellow
}

$envInfo = Get-EnvironmentReport
Write-Host "`n--- Entorno (constitución, principio VII: hardware documentado) ---"
Write-Host "CPU:            $($envInfo.Cpu)"
Write-Host "GPU:            $($envInfo.Gpu)"
Write-Host "RAM:            $($envInfo.RamGB) GB"
Write-Host "NIC activa:     $($envInfo.Nic)"
Write-Host "Windows:        $($envInfo.Windows)"
Write-Host "WebView2:       $($envInfo.WebView2)"
Write-Host "Resolucion:     $($envInfo.Resolucion)"
Write-Host "Plan energia:   $($envInfo.PlanDeEnergia)"

$escenario = if ($ReduceMotion) { "sin animacion (reduceMotion=true)" } else { "con animacion (reduceMotion=false)" }
Write-Host "`nEscenario: $escenario -- $Runs ejecuciones de $DurationSeconds s`n"

if (-not (Set-ReduceMotionSetting([bool]$ReduceMotion))) {
  Write-Host "Continuando con el valor de reduceMotion que ya tuviera settings.json." -ForegroundColor Yellow
}

$resultados = @()

for ($i = 1; $i -le $Runs; $i++) {
  Write-Host "--- Ejecucion $i/$Runs ---"
  $proc = Start-Process -FilePath (Resolve-Path $ExePath) -PassThru
  Start-Sleep -Seconds $StartupWaitSeconds

  $ids = Get-ProcessTree $proc.Id | Sort-Object -Unique
  $vivos = Get-Process -Id $ids -ErrorAction SilentlyContinue
  if (-not $vivos -or -not ($vivos | Where-Object Id -eq $proc.Id)) {
    Write-Host "FALLO: el proceso no sigue vivo tras el arranque en la ejecucion $i." -ForegroundColor Red
    continue
  }

  # NTTTCP queda excluido del presupuesto (constitución, principio VII); en reposo no
  # debería aparecer, pero si un proceso previo lo dejó vivo no debe colarse en la suma.
  $idsNtttcp = @($vivos | Where-Object ProcessName -eq "ntttcp" | Select-Object -ExpandProperty Id)
  $idsMedidos = $ids | Where-Object { $_ -notin $idsNtttcp }

  $before = @{}
  foreach ($id in $idsMedidos) {
    $p = Get-Process -Id $id -ErrorAction SilentlyContinue
    if ($p) { $before[$id] = @{ Cpu = $p.TotalProcessorTime; Ws = $p.WorkingSet64 } }
  }
  $t0 = Get-Date
  Start-Sleep -Seconds $DurationSeconds
  $t1 = Get-Date
  $elapsed = ($t1 - $t0).TotalSeconds

  $totalCpuSeconds = 0.0
  $totalWsBytes = 0.0
  foreach ($id in $idsMedidos) {
    $p2 = Get-Process -Id $id -ErrorAction SilentlyContinue
    if ($p2 -and $before.ContainsKey($id)) {
      $totalCpuSeconds += ($p2.TotalProcessorTime - $before[$id].Cpu).TotalSeconds
      $totalWsBytes += $p2.WorkingSet64
    }
  }

  foreach ($id in $ids) { Stop-Process -Id $id -Force -ErrorAction SilentlyContinue }
  Start-Sleep -Milliseconds 500

  # Fórmula exacta de la constitución: 100 * cpu / tiempo, sin dividir por núcleos.
  $pctUnProcesadorLogico = [Math]::Round(($totalCpuSeconds / $elapsed) * 100, 3)
  $wsMB = [Math]::Round($totalWsBytes / 1MB, 1)
  Write-Host ("  CPU: {0} % de un procesador lógico | Memoria: {1} MB | procesos: {2}" -f $pctUnProcesadorLogico, $wsMB, $idsMedidos.Count)

  $resultados += [PSCustomObject]@{ Run = $i; PctCpu = $pctUnProcesadorLogico; WsMB = $wsMB }
}

if ($resultados.Count -eq 0) {
  Write-Host "`nFALLO: ninguna ejecucion produjo datos." -ForegroundColor Red
  exit 1
}

$media = [Math]::Round(($resultados | Measure-Object -Property PctCpu -Average).Average, 3)
$pico = [Math]::Round(($resultados | Measure-Object -Property PctCpu -Maximum).Maximum, 3)
$mediaWs = [Math]::Round(($resultados | Measure-Object -Property WsMB -Average).Average, 1)

Write-Host "`n=== Resultado ($escenario, reposo, sin sesion RUNNING_* activa) ===" -ForegroundColor Green
Write-Host "Ejecuciones validas: $($resultados.Count)/$Runs"
Write-Host "CPU media: $media % de un procesador logico (presupuesto V-04: < 5 %)"
Write-Host "CPU pico:  $pico %"
Write-Host "Memoria media: $mediaWs MB"
if ($media -lt 5) {
  Write-Host "Dentro del presupuesto de V-04." -ForegroundColor Green
} else {
  Write-Host "FUERA del presupuesto de V-04." -ForegroundColor Red
}
Write-Host "`nEsto NO cierra V-04/V-12 por completo: mide la app en reposo, no durante una" -ForegroundColor Yellow
Write-Host "medicion RUNNING_* real (hace falta un segundo equipo o instancia emparejada)." -ForegroundColor Yellow

exit 0
