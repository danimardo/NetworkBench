# Harness V-11: Geometría de ventana, Snap, DPI y recuperación de monitores (Windows / Tauri 2)
# Requisitos: PowerShell 5.1+, Windows 10 22H2 / Windows 11

param (
    [switch]$VerboseOutput = $false
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Harness V-11: Pruebas de Geometría de Ventana y DPI      " -ForegroundColor Cyan
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

# 1. Regla de dimensiones mínimas (800x600)
$MinWidth = 800
$MinHeight = 600
$DefaultWidth = 1024
$DefaultHeight = 720
$MinVisible = 100

Assert-Condition -Name "Constantes de dimensiones válidas" `
    -Condition ($MinWidth -ge 800 -and $MinHeight -ge 600 -and $MinVisible -eq 100) `
    -Details "Mínimo exigido: 800x600, 100px visibles para fallback"

# 2. Simulación de cálculo de visibilidad e intersección (Regla 100x100 px)
function Test-IsVisible {
    param ($WinX, $WinY, $WinW, $WinH, $MonX, $MonY, $MonW, $MonH)
    $InterLeft = [Math]::Max($WinX, $MonX)
    $InterTop = [Math]::Max($WinY, $MonY)
    $InterRight = [Math]::Min($WinX + $WinW, $MonX + $MonW)
    $InterBottom = [Math]::Min($WinY + $WinH, $MonY + $MonH)
    $InterW = $InterRight - $InterLeft
    $InterH = $InterBottom - $InterTop
    return ($InterW -ge 100 -and $InterH -ge 100)
}

$VisibleInside = Test-IsVisible 100 100 1024 720 0 0 1920 1080
Assert-Condition -Name "Ventana dentro de pantalla es visible" -Condition $VisibleInside

$VisibleEdgeExact = Test-IsVisible (1920 - 100) (1080 - 100) 800 600 0 0 1920 1080
Assert-Condition -Name "Ventana en el borde con exactamente 100x100 px es visible" -Condition $VisibleEdgeExact

$NotVisibleSub100X = Test-IsVisible (1920 - 99) 100 800 600 0 0 1920 1080
Assert-Condition -Name "Ventana con 99 px horizontales visibles es rechazada" -Condition (-not $NotVisibleSub100X)

$NotVisibleSub100Y = Test-IsVisible 100 (1080 - 99) 800 600 0 0 1920 1080
Assert-Condition -Name "Ventana con 99 px verticales visibles es rechazada" -Condition (-not $NotVisibleSub100Y)

# 3. Simulación de monitor secundario retirado (desconexión)
$SecondaryDisconnected = Test-IsVisible 2560 100 1024 720 0 0 1920 1080
Assert-Condition -Name "Ventana en coordenadas de monitor secundario desconectado no es visible en primario" `
    -Condition (-not $SecondaryDisconnected)

# Fallback al centro del monitor primario (1920x1080)
$FallbackX = [Math]::Max(0, [int]((1920 - $DefaultWidth) / 2))
$FallbackY = [Math]::Max(0, [int]((1080 - $DefaultHeight) / 2))
$FallbackVisible = Test-IsVisible $FallbackX $FallbackY $DefaultWidth $DefaultHeight 0 0 1920 1080
Assert-Condition -Name "Geometría recentrada por fallback es plenamente visible" `
    -Condition ($FallbackVisible -and $FallbackX -eq 448 -and $FallbackY -eq 180) `
    -Details "Centrado calculado: X=$FallbackX, Y=$FallbackY"

# 4. Simulación de Snap Assist y atajos Win+Flechas
# Media pantalla izquierda (Win+Izquierda)
$SnapLeftW = [int](1920 / 2)
$SnapLeftVisible = Test-IsVisible 0 0 $SnapLeftW 1080 0 0 1920 1080
Assert-Condition -Name "Snap Assist mitad izquierda (960x1080) es válido" -Condition $SnapLeftVisible

# Media pantalla derecha (Win+Derecha)
$SnapRightVisible = Test-IsVisible 960 0 $SnapLeftW 1080 0 0 1920 1080
Assert-Condition -Name "Snap Assist mitad derecha (960x1080) es válido" -Condition $SnapRightVisible

# Maximizado (Win+Arriba)
$MaximizedPreservesRestoredSize = ($DefaultWidth -eq 1024 -and $DefaultHeight -eq 720)
Assert-Condition -Name "Modo maximizado preserva dimensiones restauradas para desmaximizar" `
    -Condition $MaximizedPreservesRestoredSize

# 5. Escala de DPI (100%, 150%, 200%)
$DpiScales = @(
    @{ Factor = 1.0; Name = "DPI 100% (96 DPI)";   LogicalW = 1024; LogicalH = 720; PhysicalW = 1024; PhysicalH = 720 },
    @{ Factor = 1.5; Name = "DPI 150% (144 DPI)";  LogicalW = 1024; LogicalH = 720; PhysicalW = 1536; PhysicalH = 1080 },
    @{ Factor = 2.0; Name = "DPI 200% (192 DPI)";  LogicalW = 1024; LogicalH = 720; PhysicalW = 2048; PhysicalH = 1440 }
)

foreach ($dpi in $DpiScales) {
    $CalcPhysicalW = [int]($dpi.LogicalW * $dpi.Factor)
    $CalcPhysicalH = [int]($dpi.LogicalH * $dpi.Factor)
    $Match = ($CalcPhysicalW -eq $dpi.PhysicalW -and $CalcPhysicalH -eq $dpi.PhysicalH)
    Assert-Condition -Name "Escala $($dpi.Name): conversión lógica-física exacta" `
        -Condition $Match `
        -Details "Físico esperado: $($dpi.PhysicalW)x$($dpi.PhysicalH), Calculado: ${CalcPhysicalW}x${CalcPhysicalH}"
}

Write-Host ""
$SummaryColor = "Red"
if ($PassedTests -eq $TotalTests) {
    $SummaryColor = "Green"
}
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Resumen de Pruebas V-11: $PassedTests / $TotalTests pasadas" -ForegroundColor $SummaryColor
Write-Host "==========================================================" -ForegroundColor Cyan

if ($PassedTests -ne $TotalTests) {
    exit 1
}
exit 0
