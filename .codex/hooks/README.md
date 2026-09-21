# Ubicación del fichero de hooks de Codex — sin resolver

El mismo contenido está en **dos rutas** a propósito:

- `.codex/hooks.json`
- `.codex/hooks/hooks.json`

## Por qué

El 2026-09-21 se probó `.codex/hooks/hooks.json` en una sesión real de Codex 0.155.1 y
**el hook no llegó a dispararse**: Codex editó una ruta protegida sin pedir nada y sin
solicitar confianza para los hooks del repositorio.

Lo que sí está VERIFICADO:

- La característica `hooks` está en estado `stable` y **activada** (`codex features list`).
  El problema no es que los hooks estén apagados.
- El binario menciona «Failed to read project hooks config file» junto a las rutas
  relativas al repositorio `.codex`, `.codex/config.toml`, `.codex/agents`, `.codex/hooks`,
  `.agents` y `.agents/skills`. Existe un fichero de hooks por proyecto; **cuál de las dos
  rutas es, no se ha determinado.**
- Los hooks exigen confianza: el estado guarda un `trusted_hash`. Si Codex nunca pidió
  confiar en ellos, es señal de que ni siquiera encontró el fichero.

## Lo que NO se ha hecho

No se ha declarado el hook dentro de `.codex/config.toml` como tabla TOML, aunque el
binario contiene una estructura `HookEventsToml` que lo sugiere. Motivo: una clave TOML
inventada puede impedir que Codex lea toda su configuración de proyecto, y ya hubo una
avería así el mismo día con `sandbox_mode`. **No se añaden claves TOML sin verificarlas.**

## Estado

`NO FUNCIONAL` mientras no se demuestre lo contrario. La barrera real frente a Codex es
`scripts/git-hooks/pre-commit`, que sí está verificada y no depende de ninguna herramienta.
