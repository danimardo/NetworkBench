# Estado del proyecto

**Ámbito:** este repositorio. **Estado:** VERIFICADO el 2026-09-21.
**Revísalo** en cuanto aparezca el primer manifiesto real: esta regla caduca entonces.

## La aplicación no existe

NO PRESENTE, comprobado: `package.json`, cualquier lockfile JS, `Cargo.toml`,
`rust-toolchain.toml`, `tsconfig.json`, `vite.config.*`, `svelte.config.*`,
`vitest.config.*`, `playwright.config.*`, `eslint.config.*`, `tauri.conf.*`,
`src/`, `src-tauri/`, `static/`, `tests/`, `e2e/`, `.github/`.

Lo que sí existe: documentación normativa (`Historias.md`, constitución), un sistema de
diseño de referencia (`Design/`, 18 componentes Svelte + tokens, sin build) y el sistema
de instrucciones para agentes.

## Consecuencias operativas

- **No hay comandos de build, test, lint ni type-check del proyecto.** No los inventes,
  no los ejecutes y no los documentes como disponibles. La propia constitución dice que
  `pnpm check` aún no existe y que crearlo es requisito previo a cerrar la primera
  implementación afectada.
- El **único** comando verificado hoy es `node Design/scripts/verify-tokens.mjs` (pasa,
  salida `0`). Comprueba ausencia de literales de color fuera de `tokens.css`. **No**
  certifica accesibilidad, corrección funcional, ni la tokenización de dimensiones,
  tipografía o animación.
- `node scripts/agent/verify.mjs` comprueba el sistema de instrucciones, no el producto.
- No hay CI ni hooks: **ninguna comprobación se ejecuta sola.**

## No declares verificado lo que no has ejecutado

Inspeccionar no es probar. Que una versión exista en un registro no demuestra que
compile. Que un script exista no demuestra que pase. Usa los estados de `AGENTS.md`
y di siempre qué no pudiste comprobar y por qué.

## Decisiones abiertas que condicionan cualquier plan

- **Q1** matriz Windows · **Q2** umbrales de cobertura · **Q3** paquete de alcance
  (offline, persistencia H1, 32 streams simultáneos por sentido, accesibilidad desde H1) ·
  **Q4** tema inicial.
- **G1** motor NTTTCP · **G2** protocolo · **G3** resultados · **G4** interpretación ·
  **G5** entorno y toolchain · **G6** requisitos perdidos (§§29-54).
- Las 17 discrepancias `Historias.md` ↔ constitución sin trasladar a los artefactos.

Antes de planificar, comprueba cuáles siguen abiertas y decláralo en el plan.

## Hitos

`[H1]` canal de control, descubrimiento, emparejamiento, TCP bidireccional, UI básica ·
`[H2]` firewall, interpretación, gráficas, errores completos, temas ·
`[H3]` historial, exportación, UDP, ajustes, actualizador.

Cada requisito de `Historias.md` lleva su hito entre corchetes. No implementes
anticipadamente lo que está fuera de la v1.
