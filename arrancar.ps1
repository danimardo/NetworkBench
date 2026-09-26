<#
.SYNOPSIS
  Arranca NetworkBench en modo de desarrollo, matando antes cualquier instancia zombi.
.DESCRIPTION
  `pnpm tauri dev` falla si queda un `NetworkBench.exe` colgado de una ejecucion anterior,
  o si el puerto del servidor de Vite (5173, ver vite.config.ts) sigue ocupado por un
  proceso Node huerfano — el propio "Port 5173 is already in use" que para en seco el
  arranque. Este script comprueba los dos casos y los limpia antes de arrancar, sin tocar
  ningun otro proceso del sistema.
#>

param(
  [int]$DevPort = 5173
)

$ErrorActionPreference = "Stop"

function Get-ProcessTree($rootId) {
  $all = @($rootId)
  $children = Get-CimInstance Win32_Process -Filter "ParentProcessId=$rootId" -ErrorAction SilentlyContinue
  foreach ($c in $children) { $all += Get-ProcessTree $c.ProcessId }
  return $all
}

Write-Host "=== arrancar.ps1: NetworkBench en modo de desarrollo ===" -ForegroundColor Cyan

# 1. Matar cualquier NetworkBench.exe zombi, con todo su arbol de WebView2.
$appProcs = Get-Process -Name "NetworkBench" -ErrorAction SilentlyContinue
if ($appProcs) {
  Write-Host "Encontrado NetworkBench.exe en ejecucion (PID $(($appProcs.Id) -join ', ')). Cerrando..." -ForegroundColor Yellow
  foreach ($p in $appProcs) {
    $ids = Get-ProcessTree $p.Id | Sort-Object -Unique
    foreach ($id in $ids) { Stop-Process -Id $id -Force -ErrorAction SilentlyContinue }
  }
  Start-Sleep -Milliseconds 500
  Write-Host "Cerrado." -ForegroundColor Green
} else {
  Write-Host "No habia ninguna instancia de NetworkBench en ejecucion."
}

# 2. Liberar el puerto del servidor de Vite si quedo un proceso huerfano de una sesion
#    anterior de 'pnpm dev'/'pnpm tauri dev' que nunca se cerro bien.
$conn = Get-NetTCPConnection -LocalPort $DevPort -State Listen -ErrorAction SilentlyContinue
if ($conn) {
  $pids = $conn | Select-Object -ExpandProperty OwningProcess -Unique
  foreach ($procId in $pids) {
    $proc = Get-Process -Id $procId -ErrorAction SilentlyContinue
    if ($proc) {
      Write-Host "Puerto $DevPort ocupado por '$($proc.ProcessName)' (PID $procId). Cerrando..." -ForegroundColor Yellow
      Stop-Process -Id $procId -Force -ErrorAction SilentlyContinue
    }
  }
  Start-Sleep -Milliseconds 500
} else {
  Write-Host "Puerto $DevPort libre."
}

# 3. Arrancar de verdad. Bloquea la consola con el log en vivo de Vite + Cargo/Tauri;
#    Ctrl+C lo detiene limpiamente.
Write-Host "`nArrancando 'pnpm tauri dev'...`n" -ForegroundColor Cyan
pnpm tauri dev
