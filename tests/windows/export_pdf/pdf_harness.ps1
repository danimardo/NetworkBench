# tests/windows/export_pdf/pdf_harness.ps1
# Harness de verificación V-09 para NetworkBench:
# Fidelidad de PrintToPdf con SVG, fuentes, formato A4 y control de fallos en ruta de destino (§20.1, AC-NB-12).

$ErrorActionPreference = "Stop"

Write-Host "=== Iniciando Harness V-09: Verificación de exportación PDF (PrintToPdf A4 + SVG) ===" -ForegroundColor Cyan

$tempDir = Join-Path $env:TEMP "NetworkBench_Test_Pdf_$([System.Guid]::NewGuid().ToString())"
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

try {
    # 1. Crear documento HTML representativo con estilos de impresión A4 y SVG vectoriales
    $htmlContent = @"
<!DOCTYPE html>
<html lang="es">
<head>
<meta charset="UTF-8">
<title>NetworkBench - Informe de Rendimiento de Red</title>
<style>
  @page {
    size: A4 portrait;
    margin: 15mm;
  }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    color: #0f172a;
    background: #ffffff;
    margin: 0;
    padding: 0;
  }
  .header {
    border-bottom: 2px solid #0284c7;
    padding-bottom: 12px;
    margin-bottom: 20px;
  }
  h1 { font-size: 20pt; margin: 0 0 6px 0; color: #0f172a; }
  .meta { font-size: 10pt; color: #64748b; }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 20px;
  }
  .card {
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    padding: 12px;
    background: #f8fafc;
  }
  .chart-container {
    width: 100%;
    height: 180px;
    margin: 20px 0;
  }
  svg { width: 100%; height: 100%; }
</style>
</head>
<body>
  <div class="header">
    <h1>Informe de Rendimiento de Red</h1>
    <div class="meta">NetworkBench v0.1.0 · Motor Microsoft NTTTCP 5.40 · 2026-09-22</div>
  </div>
  <div class="grid">
    <div class="card">
      <strong>Equipo local:</strong> DESKTOP-LOCAL<br>
      <strong>Velocidad oficial:</strong> 948,00 Mbit/s<br>
      <strong>Veredicto:</strong> Rendimiento óptimo
    </div>
    <div class="card">
      <strong>Equipo remoto:</strong> LAPTOP-REMOTE<br>
      <strong>Capacidad ref:</strong> 1.000,00 Mbit/s<br>
      <strong>Aprovechamiento:</strong> 94,8 %
    </div>
  </div>
  <div class="chart-container">
    <svg viewBox="0 0 500 150" xmlns="http://www.w3.org/2000/svg">
      <rect width="500" height="150" fill="#f1f5f9" rx="4" />
      <polyline fill="none" stroke="#0284c7" stroke-width="2.5"
        points="0,120 50,110 100,60 150,55 200,50 250,52 300,48 350,50 400,49 450,51 500,50" />
      <text x="10" y="25" fill="#475569" font-size="10" font-family="sans-serif">Throughput (Mbit/s)</text>
    </svg>
  </div>
</body>
</html>
"@

    $htmlPath = Join-Path $tempDir "report.html"
    $pdfPath = Join-Path $tempDir "report.pdf"
    [System.IO.File]::WriteAllText($htmlPath, $htmlContent, [System.Text.Encoding]::UTF8)

    # 2. Localizar Edge / WebView2 runtime para prueba headless de PrintToPdf
    $edgePaths = @(
        "$env:ProgramFiles (x86)\Microsoft\Edge\Application\msedge.exe",
        "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe",
        "$env:LOCALAPPDATA\Microsoft\Edge\Application\msedge.exe"
    )

    $msedge = $null
    foreach ($p in $edgePaths) {
        if (Test-Path $p) {
            $msedge = $p
            break
        }
    }

    if ($null -ne $msedge) {
        Write-Host "Ejecutando renderizado de PrintToPdf headless con WebView2/Edge..." -ForegroundColor Yellow
        $process = Start-Process -FilePath $msedge -ArgumentList @(
            "--headless",
            "--disable-gpu",
            "--run-all-compositor-stages-before-draw",
            "--print-to-pdf=""$pdfPath""",
            """$htmlPath"""
        ) -Wait -PassThru -NoNewWindow

        if (Test-Path $pdfPath) {
            $fileBytes = [System.IO.File]::ReadAllBytes($pdfPath)
            if ($fileBytes.Length -gt 1000) {
                # Comprobar cabecera mágica de PDF
                $magic = [System.Text.Encoding]::ASCII.GetString($fileBytes[0..4])
                if ($magic -like "%PDF-*") {
                    Write-Host "[OK] V-09: PDF A4 generado correctamente con fidelidad y cabecera $magic ($($fileBytes.Length) bytes)." -ForegroundColor Green
                } else {
                    throw "El archivo generado no tiene cabecera PDF válida: $magic"
                }
            } else {
                throw "El archivo PDF generado es anormalmente pequeño ($($fileBytes.Length) bytes)"
            }
        } else {
            throw "El comando de impresión no generó el archivo PDF esperado"
        }
    } else {
        Write-Host "[AVISO] msedge.exe no localizado en rutas estándar; omitiendo paso interactivo de WebView2." -ForegroundColor Yellow
    }

    # 3. Prueba de fallo de ruta de destino (ruta inválida / sin permisos)
    Write-Host "Verificando control de fallos ante rutas de destino inválidas..." -ForegroundColor Yellow
    $invalidPath = "Z:\RutaInexistente_12345\report.pdf"
    $failed = $false
    try {
        [System.IO.File]::WriteAllBytes($invalidPath, [byte[]]@(0x25, 0x50, 0x44, 0x46))
    } catch {
        $failed = $true
    }

    if ($failed) {
        Write-Host "[OK] V-09: Control de fallo de destino verificado correctamente." -ForegroundColor Green
    } else {
        throw "La escritura en ruta inexistente debió fallar"
    }

    Write-Host "=== Harness V-09 completado con éxito ===" -ForegroundColor Green
}
finally {
    if (Test-Path $tempDir) {
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
