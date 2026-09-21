# Hooks de Git

Barrera **independiente de la herramienta de agentes**. Git los ejecuta pase lo que pase
con la política de permisos de Claude Code o de Codex, y también cuando el commit lo haces
tú a mano. Es la única garantía del repositorio que no depende de que un agente coopere.

## Instalación

Una vez por clon:

```
git config core.hooksPath scripts/git-hooks
```

No se instala solo: `core.hooksPath` es configuración local de cada clon y Git no la
versiona a propósito, porque un hook versionado que se activara solo sería código
ejecutándose sin que nadie lo haya aceptado.

Comprobar que está activo:

```
git config --get core.hooksPath      # debe imprimir scripts/git-hooks
```

## `pre-commit`

1. `check-protected-paths.mjs --staged` — rechaza cambios en `.specify/`,
   `.agents/skills/speckit-*`, `Design/`, `Historias.md` y los documentos históricos.
   La constitución no se rechaza: se avisa, porque su cambio debe declararse.
2. `verify.mjs` — coherencia del sistema de instrucciones.

## Salida de emergencia

```
git commit --no-verify
```

Solo con autorización del propietario y **declarándolo en el mensaje del commit**.
Un `--no-verify` sin explicación es un cambio en silencio, que es exactamente lo que
el hook existe para impedir.
