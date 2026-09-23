# Validación de Arquitectura y Dependencias (T130)

**Fecha**: 2026-09-22  
**Entorno**: Windows 11 Pro (build 26200)  
**Estado**: `VERIFICADO` (evidencia mediante scripts deterministas `check-imports.mjs` y `check-logger.mjs`)

---

## 1. Reglas y Fronteras Arquitectónicas

De acuerdo con la constitución (artículo II y IV) y [ADR-002](../../docs/governance/ADRS.md#L25):

1. **Autoridad de Negocio en Rust**:
   - Todo cálculo diagnóstico, veredicto, control de sesión, autorización y persistencia reside exclusivamente en Rust (`src-tauri/`).
   - El frontend Svelte solo consume snapshots monótonos inmutables y eventos a través del cliente tipado IPC (`src/lib/api/`).
2. **Modularidad Feature-First**:
   - Cada feature (`peers`, `session`, `results`, `history`, `export`, `settings`) cuenta con un `index.ts` que define su API pública.
   - Prohibición de imports profundos entre features. Se verificó con `scripts/architecture/check-imports.mjs`: cero infracciones detectadas.
3. **Uso Exclusivo de Logger Saneado**:
   - Prohibido el uso de `console.log`, `console.warn`, `console.error` o `println!` en código de producción.
   - Todo registro se canaliza a través de `src-tauri/src/logging` y `src/lib/logging`. Se verificó con `scripts/architecture/check-logger.mjs`: cero llamadas prohibidas.
4. **Sin APIs Legacy**:
   - No se emplean dependencias obsoletas ni crates no justificados.

---

## 2. Evidencia de Ejecución

```text
$ node scripts/architecture/check-imports.mjs
✓ Comprobación de imports y fronteras de arquitectura superada.

$ node scripts/architecture/check-logger.mjs
✓ Comprobación de logger único superada.
```

Resultado: **PASS**. Arquitectura limpia, desacoplada y alineada con ADR-001 a ADR-007.
