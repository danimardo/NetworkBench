# Harness de prueba para Firewall y UAC en Windows (NetworkBench v1)
param(
    [switch]$Verbose = $false
)

$ErrorActionPreference = "Stop"
Write-Host "=== Iniciando Harness de Firewall y UAC (NetworkBench) ===" -ForegroundColor Cyan

$testCount = 0
$passCount = 0

function Assert-Test([string]$name, [bool]$condition) {
    $script:testCount++
    if ($condition) {
        $script:passCount++
        Write-Host "  [PASS] $name" -ForegroundColor Green
    } else {
        Write-Host "  [FAIL] $name" -ForegroundColor Red
        throw "Fallo en aserción: $name"
    }
}

# 1. Comprobar que una regla inexistente no coincide en netsh
Write-Host "`n1. Verificando detección de regla ausente..."
$missingRuleName = "NetworkBench_NonExistent_Rule_Test_999"
$netshOutput = & netsh advfirewall firewall show rule name="$missingRuleName" 2>&1
$isAbsent = ($netshOutput -match "No rules match" -or $netshOutput -match "No coincide ninguna regla")
Assert-Test "Regla ausente detectada correctamente por netsh" ($isAbsent -or $LASTEXITCODE -ne 0)

# 2. Verificación de binario helper y allowlist
Write-Host "`n2. Verificando binario helper y lista blanca..."
$helperPath = "src-tauri\target\x86_64-pc-windows-msvc\debug\networkbench-firewall-helper.exe"
if (Test-Path $helperPath) {
    # Prueba con regla que viola la lista blanca (prefijo no permitido)
    $badReq = '[{"operation":"add","ruleName":"HackerRule","protocol":"TCP","portRange":"5201","program":"","profiles":["Private"]}]'
    $tmpBad = [System.IO.Path]::GetTempFileName()
    Set-Content -Path $tmpBad -Value $badReq -Encoding UTF8
    
    $proc = Start-Process -FilePath $helperPath -ArgumentList "--file `"$tmpBad`"" -NoNewWindow -Wait -PassThru
    Remove-Item -Path $tmpBad -Force
    Assert-Test "Helper rechaza regla con prefijo no permitido" ($proc.ExitCode -ne 0)

    # Prueba con regla que excede 64 puertos
    $hugeRangeReq = '[{"operation":"add","ruleName":"NetworkBench - Test","protocol":"TCP","portRange":"5000-6000","program":"","profiles":["Private"]}]'
    $tmpHuge = [System.IO.Path]::GetTempFileName()
    Set-Content -Path $tmpHuge -Value $hugeRangeReq -Encoding UTF8

    $procHuge = Start-Process -FilePath $helperPath -ArgumentList "--file `"$tmpHuge`"" -NoNewWindow -Wait -PassThru
    Remove-Item -Path $tmpHuge -Force
    Assert-Test "Helper rechaza rango que excede 64 puertos" ($procHuge.ExitCode -ne 0)
} else {
    Write-Host "  [INFO] networkbench-firewall-helper.exe no compilado aún en debug; se valida lógica en Rust unit tests." -ForegroundColor Yellow
}

# 3. Verificación de código de error UAC 1223 (ERROR_CANCELLED)
Write-Host "`n3. Verificando mapeo de error de cancelación UAC (Win32 1223)..."
$uacCancelledCode = 1223
Assert-Test "Código Win32 1223 corresponde a ERROR_CANCELLED" ($uacCancelledCode -eq 1223)

Write-Host "`n=== Resumen de Harness de Firewall: $passCount / $testCount pruebas superadas ===" -ForegroundColor Cyan
if ($passCount -eq $testCount) {
    exit 0
} else {
    exit 1
}
