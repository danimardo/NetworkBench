# Harness de Rendimiento V-04 / V-12 (NetworkBench)

Este directorio documenta el arnés de medición de rendimiento de la interfaz gráfica y procesos de NetworkBench en Windows.

## Requisitos de rendimiento evaluados

1. **V-04 (Presupuesto de CPU/GPU durante `RUNNING_*`)**:
   - Refresco de muestras en pantalla $\le$ 4 Hz (intervalo mínimo 250 ms entre lotes).
   - Animación de flujo en GPU sin layout trashing.
   - Respeto de `prefers-reduced-motion`.

2. **V-12 (Consumo total de recursos app + WebView2)**:
   - Medición de memoria RAM (Working Set) y porcentaje de CPU consumido por `NetworkBench.exe` y los procesos hijos de `msedgewebview2.exe`.

## Script de ejecución

Para ejecutar la captura de métricas en Windows:

```powershell
powershell -File scripts/test/performance.ps1
```
