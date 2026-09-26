# Harness de Rendimiento V-04 / V-12 (NetworkBench)

Este directorio documenta el arnés de medición de rendimiento de la interfaz gráfica y procesos de NetworkBench en Windows.

## Estado (T159, 2026-09-25)

La versión anterior de `scripts/test/performance.ps1` **no lanzaba la aplicación**:
muestreaba procesos llamados `NetworkBench`/`msedgewebview2` que nunca existían, así que
`ProcessCount` era siempre 0 y el resumen ("Límite <= 4 Hz respetado") medía la frecuencia
del propio bucle del script, no de la aplicación. Tampoco medía CPU (variable declarada y
nunca calculada).

La versión actual lanza el ejecutable real (`NetworkBench.exe`), localiza todo su árbol de
procesos (la app más los `msedgewebview2.exe` que arranca), mide CPU real por delta de
`TotalProcessorTime` y memoria real (`WorkingSet64`), y limpia solo ese árbol al terminar.
**Verificado el 2026-09-25** en Windows 11 Pro 26200 x64 (AMD Ryzen 5 2600X, 6 núcleos
físicos / 12 lógicos): la app completa en reposo (ventana abierta, sin sesión activa)
consume entre 0,3 % y 1,4 % de un núcleo según la ventana medida, muy por debajo del 5 %
de V-04, y ~360 MB de memoria de trabajo entre los 8 procesos del árbol.

**Esto NO cierra V-04 ni V-12 por completo**: mide la app en **reposo**, no «durante la
prueba» (`RUNNING_*`), que es lo que V-04 presupuesta. Medir durante una sesión real
necesita un segundo equipo (o al menos otra instancia local emparejada) que este script no
levanta. Sigue sin definirse el «núcleo de referencia» de SC-010 en ningún artefacto del
proyecto (T168): el script usa y declara la CPU real de la máquina donde corre, no una
certificada contra un hardware acordado.

## Requisitos de rendimiento evaluados

1. **V-04 (Presupuesto de CPU/GPU durante `RUNNING_*`)**:
   - Refresco de muestras en pantalla $\le$ 4 Hz (intervalo mínimo 250 ms entre lotes).
   - Animación de flujo en GPU sin layout trashing.
   - Respeto de `prefers-reduced-motion`.
   - Presupuesto de CPU: < 5 % de un núcleo de referencia (sin definir, T168).

2. **V-12 (Consumo total de recursos app + WebView2)**:
   - Medición de memoria RAM (Working Set) y porcentaje de CPU consumido por `NetworkBench.exe` y los procesos hijos de `msedgewebview2.exe`.

## Script de ejecución

Para ejecutar la captura de métricas en Windows (compila primero el backend si no existe
el ejecutable):

```powershell
powershell -File scripts/test/performance.ps1
```

Parámetros: `-ExePath` (por defecto el binario de depuración MSVC), `-DurationSeconds`
(ventana de medición, 10 s por defecto), `-StartupWaitSeconds` (espera antes de medir para
dejar que WebView2 arranque, 5 s por defecto). Sale con código 2 y sin medir nada si no
encuentra el ejecutable — no da un resultado por bueno sin haberlo comprobado.
