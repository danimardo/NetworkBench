# Validación de Seguridad, Secretos y Licencias (T131)

**Fecha**: 2026-09-22  
**Entorno**: Windows 11 Pro (build 26200)  
**Estado**: `VERIFICADO` (mediante escaneo estático `scripts/security/scan-security.mjs` y revisión de `THIRD_PARTY_NOTICES.md`)

---

## 1. Alcance de la Auditoría

1. **Escaneo de Secretos**:
   - Inspección recursiva de `src/`, `src-tauri/src/`, `tests/` y `scripts/`.
   - Búsqueda de claves privadas (PEM, RSA, EC), tokens de GitHub/AWS y credenciales en texto plano.
   - **Resultado**: 0 secretos detectados. Toda clave criptográfica efímera o persistente se deriva de DPAPI de Windows (`CryptProtectData`) o Ed25519 en memoria, sin serialización de claves privadas en logs ni frontend.
2. **Revisión de Licencias y Dependencias**:
   - `THIRD_PARTY_NOTICES.md` enumera de forma exhaustiva las dependencias de Rust, frontend y el motor NTTTCP de Microsoft.
   - Compatibilidad con licencia GPL-3.0-or-later del proyecto: todas las bibliotecas consumidas cuentan con licencias permisivas (MIT, Apache-2.0, ISC, Unlicense).
3. **Privilegio Mínimo y Superficie de Ataque**:
   - La aplicación principal se ejecuta como usuario estándar no elevado.
   - Solo el binario auxiliar `networkbench-firewall-helper` requiere UAC previa solicitud explícita del usuario y cuenta con allowlist estricta de prefijos y argumentos.

---

## 2. Evidencia de Ejecución

```text
$ node scripts/security/scan-security.mjs
=== Escaneo de Seguridad, Secretos y Licencias (T131) ===

✓ Escaneo de secretos: ningún secreto hardcodeado detectado en el código fuente.
✓ Atribuciones de terceros y licencias verificadas en THIRD_PARTY_NOTICES.md.

✓ Auditoría de seguridad superada sin incidencias.
```

Resultado: **PASS**. Sin bloqueantes de seguridad ni riesgos de licenciamiento.
