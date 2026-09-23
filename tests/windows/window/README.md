# Harness V-11: Geometría de Ventana, Monitores y DPI

Valida las garantías de geometría de ventana exigidas por la constitución y los requisitos de producto:

1. **Dimensiones Mínimas**: 800×600 px físicos/lógicos.
2. **Regla de Visibilidad**: La ventana debe intersectar al menos 100×100 px en un monitor activo.
3. **Monitor Retirado**: Si las coordenadas almacenadas caen fuera de los monitores actuales (ej. pantalla secundaria desconectada), se normaliza y recentra automáticamente en el monitor principal.
4. **Snap Assist / Atajos de Teclado**: Soporte para mitades de pantalla (Win+Izquierda, Win+Derecha) y maximizado (Win+Arriba), conservando las dimensiones restauradas.
5. **Factores DPI**: Verificación de escala sin deformación en 100% (96 DPI), 150% (144 DPI) y 200% (192 DPI).

## Ejecución

```powershell
powershell -File tests/windows/window/window_harness.ps1
```
