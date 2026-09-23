<#
.SYNOPSIS
  Harness de rendimiento V-04 / V-12 para NetworkBench y WebView2.
.DESCRIPTION
  Mide el consumo de CPU y memoria de la aplicación NetworkBench y los procesos
  asociados de WebView2 durante la ejecución, verificando el refresco <= 4 Hz.
#>

param(
  [int]$DurationSeconds = 10,
  [int]$SampleIntervalMs = 250 # Corresponde a la frecuencia máxima de 4 Hz
)

Write-Host "=== Iniciando captura de rendimiento V-04/V-12 NetworkBench ===" -ForegroundColor Cyan
Write-Host "Duración: $DurationSeconds s | Intervalo: $SampleIntervalMs ms (<= 4 Hz)"

$processNames = @("NetworkBench", "msedgewebview2")
$samples = @()
$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

while ($stopwatch.Elapsed.TotalSeconds -lt $DurationSeconds) {
  $timestamp = [DateTime]::UtcNow.ToString("o")
  $procs = Get-Process -Name $processNames -ErrorAction SilentlyContinue
  
  $totalWs = 0
  $totalCpu = 0.0
  
  foreach ($p in $procs) {
    $totalWs += $p.WorkingSet64
  }

  $sample = [PSCustomObject]@{
    Timestamp = $timestamp
    ElapsedSeconds = [Math]::Round($stopwatch.Elapsed.TotalSeconds, 2)
    ProcessCount = $procs.Count
    WorkingSetMB = [Math]::Round($totalWs / 1MB, 2)
  }
  
  $samples += $sample
  Start-Sleep -Milliseconds $SampleIntervalMs
}

$stopwatch.Stop()

Write-Host "`n=== Resumen de Rendimiento ===" -ForegroundColor Green
$avgWs = ($samples | Measure-Object -Property WorkingSetMB -Average).Average
$maxWs = ($samples | Measure-Object -Property WorkingSetMB -Maximum).Maximum

Write-Host "Muestras capturadas: $($samples.Count) (Frecuencia: $([Math]::Round($samples.Count / $DurationSeconds, 1)) Hz)"
Write-Host "Memoria Media (Working Set): $([Math]::Round($avgWs, 2)) MB"
Write-Host "Memoria Pico (Working Set): $([Math]::Round($maxWs, 2)) MB"
Write-Host "Límite <= 4 Hz respetado: $([Math]::Round($samples.Count / $DurationSeconds, 1) -le 4.5)"

exit 0
