---
name: "cierre-de-tarea"
description: "Comprobar y declarar honestamente el cierre de una tarea en NetworkBench: qué se hizo, qué se verificó ejecutándolo, qué quedó fuera y por qué. Usar antes de dar cualquier trabajo por terminado."
compatibility: "NetworkBench. Requiere AGENTS.md y .agents/ en la raíz del repositorio."
metadata:
  author: "sistema de agentes NetworkBench"
  canonical: ".agents/skills/cierre-de-tarea/SKILL.md"
---

# Cierre de tarea

Una tarea no termina cuando el código está escrito, sino cuando **queda claro qué es
cierto sobre ella**. Esta skill produce esa declaración.

## 1. Alcance

Contrasta lo entregado con lo pedido, no con lo que fue cómodo hacer.

- ¿Está hecho **todo** el alcance? Si una parte quedó bloqueada, ¿está el resto completo?
- ¿Se amplió el alcance sin pedirlo? Reducirlo es decisión del propietario, no tuya.
- ¿Se corrigió algo no relacionado? No debería.
- ¿Quedó algo a medias sin declararlo? Eso no es un cierre, es un abandono silencioso.

## 2. Verificación

Para cada afirmación que vayas a hacer, asigna un estado y ten la evidencia a mano:

`VERIFICADO` · `DOCUMENTADO` · `INFERIDO` · `NO VERIFICABLE` · `NO PRESENTE`

Reglas duras:

- **Inspeccionar no es probar.** Si no lo ejecutaste, no está VERIFICADO.
- **No inventes comandos.** En este repositorio hoy solo existen
  `node Design/scripts/verify-tokens.mjs` y `node scripts/agent/verify.mjs`.
  No hay build, ni tests, ni lint, ni type-check. Ver `.agents/rules/proyecto/estado.md`.
- Si tocaste `AGENTS.md`, `.agents/`, `CLAUDE.md` o `.claude/`:
  ejecuta `node scripts/agent/verify.mjs` y pega el resultado real.
- Si tocaste `Design/src/`: ejecuta `node Design/scripts/verify-tokens.mjs`.
  (Antes comprueba que estaba permitido tocarlo: `.agents/rules/proyecto/diseno.md` lo prohíbe.)
- Un test fallido, omitido o intermitente **no cuenta como aprobado**. Reintentar no borra
  la primera incidencia.

## 3. Reglas que pudiste romper sin darte cuenta

- ¿Hiciste alguna operación de Git? Requiere autorización específica cada vez.
- ¿Tocaste `.specify/`, las skills `speckit-*`, la constitución, `Historias.md` o `Design/`?
  Todas están protegidas.
- ¿Ampliaste permisos, capabilities o alcance de sandbox para desbloquearte? Prohibido.
- ¿Aparece algún secreto en lo que vas a entregar o en los logs?
- ¿Está todo en español, con tildes?

## 4. Trazabilidad

Para un cambio no trivial, deja constancia de: qué petición lo originó, qué requisito de
`Historias.md` o principio de la constitución lo define, qué ficheros cambiaron, qué se
verificó, qué no pudo verificarse y **contra qué versión de la constitución** se trabajó
(hoy: 0.4.0, no ratificada).

No fuerces este aparato para un cambio trivial.

## 5. Declaración final

Entrega, en este orden:

1. **Qué se hizo**, en una o dos frases.
2. **Ficheros**: creados, modificados, generados, no tocados por falta de autorización.
3. **Verificaciones ejecutadas**, con el comando y su resultado real.
4. **Qué NO se verificó y por qué**, y cómo verificarlo más adelante.
5. **Contradicciones o riesgos** detectados y no resueltos.
6. **Qué queda pendiente** y de quién depende.

Si algo salió mal, dilo con su salida. Si un paso se saltó, dilo. Si está hecho y
comprobado, dilo sin rodeos ni coletillas defensivas.
