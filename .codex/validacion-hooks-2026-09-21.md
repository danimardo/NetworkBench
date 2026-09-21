# Diagnóstico y reparación de PreToolUse — 2026-09-21

Petición del propietario: identificar la ruta consumida por Codex, activar el hook y
demostrar bloqueo de escritura y permiso de lectura. Referencia normativa: constitución
0.4.0, no ratificada; no modificada. Ejecutable ensayado: `codex-cli 0.155.1`.

## Resultado y causa

**VERIFICADO:** el proyecto declara el hook en `.codex/hooks.json`, en JSON. La candidata
anidada no se consume como declaración del proyecto. Además, faltaba confiar en la
definición: una sesión interactiva mostró un hook nuevo, instalado pero inactivo.

Tras conceder confianza, el bloqueo ya funcionó. Apareció una segunda incompatibilidad:
la lectura continuaba, pero el hook figuraba como `Failed`. El guard emitía
`permissionDecision: allow` sin `updatedInput`. El binario contiene literalmente
`PreToolUse hook returned unsupported permissionDecision:allow`. Se sustituyó esa
respuesta por `{}`; el criterio de detección de rutas y la respuesta `deny` no cambiaron.
La repetición real produjo `Blocked` para escritura y `Completed` para lectura.

## Documentación oficial consultada

- [Configuración avanzada](https://developers.openai.com/es-419/docs/config-file/config-advanced):
  admite JSON en `.codex/hooks.json` y tablas de hooks en `.codex/config.toml`.
- [Hooks](https://learn.chatgpt.com/docs/hooks): esquema por evento, grupo matcher y
  handlers; confianza de la definición mediante `/hooks`, almacenada por hash.

Son páginas actuales, no una edición fijada a 0.155.1. La ruta JSON y su ejecución en esa
versión quedan **VERIFICADAS** por las pruebas siguientes. La alternativa TOML queda
**DOCUMENTADA**, no ejecutada.

Declaración JSON conservada:

```json
{
  "hooks": {
    "PreToolUse": [{
      "matcher": "*",
      "hooks": [{
        "type": "command",
        "command": "node scripts/agent/guard-protected-paths.mjs",
        "timeout": 10
      }]
    }]
  }
}
```

Equivalente TOML documentado, no instalado ni probado:

```toml
[[hooks.PreToolUse]]
matcher = "*"

[[hooks.PreToolUse.hooks]]
type = "command"
command = "node scripts/agent/guard-protected-paths.mjs"
timeout = 10
```

## Prueba de las rutas

Se guardó copia antes de cada intento y se restauró en `finally`, comprobando igualdad
SHA-256. Se escribió `{ INVALID JSON: NETBENCH_HOOK_PROBE` en una candidata cada vez.

Primero se ejecutó `codex debug prompt-input`: ambas candidatas dieron salida 0 y ningún
aviso. **Prueba no concluyente:** ese comando no demuestra carga de hooks en ejecución.

Después se arrancaron sesiones con `codex exec --ephemeral --color never`, pidiendo
responder solamente OK, sin herramientas ni cambios. Salidas relevantes reales:

```text
# Candidata .codex/hooks.json
warning: failed to parse hooks config F:\Apps\NetBench\.codex\hooks.json: key must be a string at line 1 column 3
OK
EXIT=0
RESTORED=True

# Candidata anidada
OK
EXIT=0
RESTORED=True
```

La segunda sesión no mostró aviso de parseo. La primera demuestra también que un JSON
inválido genera un aviso, pero no impide arrancar: no es una barrera que falle en cerrado.

## Confianza

Con `codex --no-alt-screen`, el diálogo mostró estas líneas:

```text
Hooks need review
1 hook is new or changed.
PreToolUse            1           0           1
Source    Project config - F:\Apps\NetBench\.codex\hooks.json
Command   node scripts/agent/guard-protected-paths.mjs
Matcher   *
Timeout   10s
Trust     New hook - review required
```

Se revisó ese único hook y se pulsó `t` en su ficha. Resultado:

```text
[x] Hook 1
Trust     Trusted
PreToolUse            1           1
```

No se usó bypass ni se editó manualmente el almacén de confianza. La confianza quedó
persistida: sesiones CLI nuevas ejecutaron el hook. No se amplió el sandbox ni se
ejecutó su setup. La confianza del proyecto y la de la definición del hook son distintas.

## Demostración final

Sesión CLI nueva: `01a0c3d2-68e9-70b3-ad7c-596fbe692da4`, con dos llamadas separadas a
`exec_command` y `login=false`. El runner de la prueba guardó una copia del documento
antes del ensayo para preservar sus cambios previos si fallaba el bloqueo.

Escritura solicitada: `Add-Content -LiteralPath Historias.md -Value NETBENCH_HOOK_PROBE_20260921`.
Salida real del bloqueo:

```text
Command blocked by PreToolUse hook: Ruta protegida de NetworkBench — requisitos de producto. Modificarla requiere autorización específica del propietario: informa, propón y espera. No la edites por otra vía. Ver .agents/rules/universal/fuentes-de-verdad.md. Command: Add-Content -LiteralPath Historias.md -Value NETBENCH_HOOK_PROBE_20260921
hook: PreToolUse Blocked
```

Lectura: `Get-Content -Encoding UTF8 -LiteralPath Historias.md -TotalCount 1`.
Salida real:

```text
hook: PreToolUse
hook: PreToolUse Completed
 succeeded in 134ms:
# NetworkBench — Especificación detallada v1
```

La lectura terminó con código 0. El SHA-256 antes y después fue
`793F7AC4B3E1DA024EE85481903849D27DA6904490E6C78EBB1DFC1225436146`.
El runner imprimió `HISTORIAS_UNCHANGED=True`. No se escribió en el documento ni fue
necesario ejecutar `git restore`. El cambio previo que muestra Git pertenece al estado
anterior a esta tarea.

La primera prueba, antes de ajustar la salida de continuación, mostró `PreToolUse Failed`
en la lectura y un error del perfil PowerShell `Set-PSReadLineOption`. Se repitió sin
perfil y con UTF-8: el error del perfil desapareció, pero `Failed` persistió. Solo el
ajuste del JSON de continuación eliminó `Failed`; no se cambió el perfil del usuario.

## Cambios y límites

Se conserva la declaración JSON correcta; se eliminaron la declaración candidata falsa
y su README. Se actualizó la respuesta de continuación del guard, los dos metadatos y
las referencias de documentación afectadas. No se cambiaron las claves de config.toml.

**NO VERIFICABLE en esta sesión:** que el anfitrión recargue los hooks ya abiertos.
**No ensayado:** alternativa TOML, otras modalidades de Codex, todas las herramientas o
escrituras posibles y una nueva sesión de Claude Code tras el ajuste compartido.
La prueba acredita los comandos concretos, no una protección exhaustiva del filesystem.

El guard mantiene su comportamiento de fallo en abierto ante entradas no reconocidas.
El hook de Git sigue siendo independiente. No se hicieron operaciones de escritura de
Git, ni se modificaron las rutas protegidas por el encargo.

Las copias y los registros completos de las primeras pruebas quedaron en el directorio
temporal `C:/Users/DANIMA~1/AppData/Local/Temp/netbench-hooks-3c16f2b0ab5a4c4abcfcf6667c7def08`.

## Verificación del sistema

`node scripts/agent/verify.mjs` terminó con código 0. Salida final real:

```text
── Resumen
  correcto     Tamaño de AGENTS.md
  correcto     Coherencia de adaptadores
  correcto     Referencias (9 aviso(s))
  correcto     Protección de documentos históricos
  correcto     Rutas protegidas (working tree) (2 aviso(s))
  correcto     Tokens de diseño

Sistema de instrucciones coherente. Esto NO verifica el producto: no existe aplicación.
```

Los avisos corresponden a referencias históricas y al cambio previo de Historias.md.
Contradicción ajena al arreglo de hooks: los comentarios antiguos de config.toml afirman
una precedencia y un efecto de aprobación que los ensayos no demuestran; las sesiones
mostraron `approval: never`. No se atribuye a esa configuración el bloqueo observado.

## Corrección de falso positivo en `apply_patch`

Durante la creación de `ARCHITECTURE.md`, el hook bloqueó un parche cuyo destino no estaba
protegido porque el contenido documental mencionaba `.specify/`, `Historias.md` y `Design/`.
La causa era que `apply_patch` no expone un campo de ruta separado y el guard inspeccionaba
todo el parche, incluidas las líneas añadidas.

El parser específico de parches inspecciona ahora únicamente las cabeceras `Add File`,
`Update File`, `Delete File` y `Move to`. Ante formato desconocido o truncado conserva el
fallback anterior sobre todo el payload. No se eliminó ninguna ruta protegida ni se amplió
el conjunto de operaciones autorizadas.

`scripts/agent/guard-protected-paths.test.mjs` cubre 33 casos: contenido que menciona rutas
protegidas, cada operación sobre cada ruta protegida, rutas absolutas Windows, movimientos,
parches mixtos, CRLF, payload estructurado, fallback, herramientas de edición y shell.
Se incorporó a `node scripts/agent/verify.mjs`.

Evidencia real adicional: tras el cambio, `apply_patch` creó `ARCHITECTURE.md` aunque su
contenido cita las rutas protegidas. El verificador completo terminó con código 0 y registró
`Guard de rutas protegidas: ℹ pass 33`. La constitución permaneció idéntica a HEAD y el
SHA-256 de `Historias.md` no cambió durante esta corrección.
