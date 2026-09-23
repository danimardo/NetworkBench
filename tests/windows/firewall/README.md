# Arnés de Pruebas Windows: Firewall y Elevación UAC (AC-NB-09, V-08)

Este arnés documenta y automatiza la verificación del subsistema de cortafuegos de Windows y elevación controlada UAC según ADR-002, ADR-005 y `Historias.md` §14.

## Escenarios Cubiertos

1. **Regla ausente (`RuleStatus::Missing`)**:
   - Se consulta una regla no existente.
   - El inspector retorna `Missing` y el preflight diagnostica `NB-FW-002` si el canal de control está activo.
2. **Regla modificada (`RuleStatus::Modified`)**:
   - Una regla existe pero tiene puertos o protocolo distintos a los esperados.
   - Retorna `Modified`.
3. **Regla deshabilitada (`RuleStatus::Disabled`)**:
   - La regla existe pero `Enabled = False`.
   - El inspector detecta el estado inactivo y propone reconfiguración.
4. **Rechazo de UAC (`NB-FW-004`)**:
   - El usuario pulsa «No» en el diálogo UAC o se cancela el proceso elevado.
   - El sistema captura el código de cancelación (Win32 1223 `ERROR_CANCELLED`) y presenta `NB-FW-004` con opciones de reintento o instrucciones manuales copiables.
5. **Validación de Lista Blanca (Allowlist)**:
   - El helper elevado rechaza de forma estricta cualquier regla cuyo prefijo no sea `NetworkBench - `, o ejecutables ajenos a la instalación, o rangos de puertos que excedan 64 puertos.

## Ejecución del Arnés

```powershell
powershell -ExecutionPolicy Bypass -File tests/windows/firewall/firewall_harness.ps1
```
