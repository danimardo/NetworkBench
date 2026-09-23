# Arnés de Pruebas E2E de Dos Equipos (T134)

Este directorio contiene las pruebas y arneses de integración End-to-End (E2E) para NetworkBench sobre Windows, simulando y validando la interacción completa entre dos extremos (Peer Local y Peer Remoto).

## Componentes

- `e2e_two_peers_harness.ps1`: Script PowerShell de ejecución de pruebas E2E integradas que comprueba:
  1. **Generación e intercambio de identidades**: Criptografía Ed25519 con huellas SHA-256 de 64 caracteres hexadecimales.
  2. **Conectividad dual-stack**: Resolución y enlace de direcciones IPv4 (`192.168.1.100:7411`) e IPv6 con identificador de zona / zone index (`[fe80::1ff:fe23:4567%12]:7411`).
  3. **Emparejamiento simétrico**: Negociación con código numérico de 6 dígitos con expiración y mitigación de fuerza bruta.
  4. **Inspección de cortafuegos**: Verificación de reglas de Windows Defender Firewall para puerto de control (TCP 7411) y puertos de datos NTTTCP sin elevación forzada en modo usuario.
  5. **Pruebas de rendimiento TCP**: Secuencial unidireccional (forward/reverse) y bidireccional simultáneo (`RunningBoth`).
  6. **Pruebas de rendimiento UDP**: Diagnóstico determinista de datagramas con métricas de tasa recibida, paquetes perdidos y porcentaje de pérdida.
  7. **Cancelación limpia**: Interrupción atómica e idempotente del benchmark garantizada mediante Windows Job Object (cero procesos huérfanos).
  8. **Persistencia SQLite**: Registro transaccional en modo WAL con backups automáticos y retención por antigüedad.
  9. **Exportación sin canarios**: Generación y verificación de artefactos CSV (con BOM UTF-8 y delimitador por locale), JSON (unidades base) y PDF A4.

## Ejecución

```powershell
powershell -File tests\windows\e2e\e2e_two_peers_harness.ps1
```

O con salida detallada:

```powershell
powershell -File tests\windows\e2e\e2e_two_peers_harness.ps1 -VerboseOutput
```
