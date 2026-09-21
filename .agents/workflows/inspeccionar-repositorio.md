# Workflow — inspeccionar el repositorio

**Cuándo:** antes de planificar una feature, tras un parón largo, o cuando sospeches que
la documentación no describe el estado real.

**Modo:** solo lectura. No crear, modificar ni borrar nada. No instalar. No usar la red
salvo para consultar documentación.

## 1. Situarse

```
git rev-parse --show-toplevel
git branch --show-current
git status --porcelain
git remote -v
git worktree list
git log --oneline | head
```

No presupongas que el directorio de inicio es la raíz del repositorio. Anota el working
tree sucio: es estado de trabajo del propietario, **no lo limpies**.

## 2. Descubrir, no asumir

Busca solo señales de alto valor: manifiestos, lockfiles, ficheros de configuración,
estructura de directorios, extensiones de código, CI, contenedores, configuración de
agentes y documentación raíz.

Hoy el resultado esperado es: **ningún manifiesto de proyecto**. Si aparece alguno
(`package.json`, `Cargo.toml`, `rust-toolchain.toml`), el estado del repositorio ha
cambiado: `.agents/rules/proyecto/estado.md` ha caducado y hay que informar.

## 3. Inspección progresiva

```
descubrir manifiestos y estructura
        ↓
identificar tecnologías candidatas
        ↓
verificar las candidatas
        ↓
auditar en profundidad solo las confirmadas
```

Una dependencia transitiva no prueba que una tecnología forme parte de la arquitectura.
No audites React, Electron o Gradle porque existan: audita lo que haya evidencia de que existe.

## 4. Leer lo normativo, en orden

1. `.specify/memory/constitution.md` — anota versión, si está ratificada, Q abiertas y G abiertas.
2. `Historias.md` — requisitos, con su hito `[H1]/[H2]/[H3]`.
3. `Design/README.md` y el resto de `Design/*.md`.
4. `.agents/rules/` del ámbito que vayas a tocar.

## 5. Comprobar el entorno

```
claude --version ; codex --version ; node --version ; pnpm --version ; rustc --version ; git --version
```

Compara con `.agents/meta/detected-stack.yaml` y con la línea base de la constitución.
Las herramientas de la máquina **no** son las versiones del proyecto.

## 6. Comprobar el sistema de agentes

```
node scripts/agent/verify.mjs
```

Y, si alguna herramienta se ha actualizado, sigue
`.agents/skills/verificar-sistema-agentes/SKILL.md`.

## 7. Informar

Con estados por afirmación (`VERIFICADO`, `DOCUMENTADO`, `INFERIDO`, `NO VERIFICABLE`,
`NO PRESENTE`), y una lista explícita de contradicciones, huecos y limitaciones del entorno.

Si el resultado contradice `.agents/meta/detected-stack.yaml` o una regla de
`.agents/rules/proyecto/`, **dilo y propón la actualización**. No la apliques en silencio.
