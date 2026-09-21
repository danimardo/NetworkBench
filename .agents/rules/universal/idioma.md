# Idioma

**Ámbito:** universal. **Estado:** VERIFICADO (instrucción explícita del propietario).

## Reglas

- La comunicación con el propietario es **siempre en español**.
- Están en español: informes, propuestas, planes, reglas, skills, workflows, descripciones
  de agentes, listas de comprobación, mensajes de commit y documentación del sistema.
- La ortografía española es obligatoria, **con tildes, eñes y signos de apertura**.
  Nunca sustituir un carácter acentuado por su equivalente ASCII: se escribe «configuración»,
  no «configuracion»; «diseño», no «diseno».
- La interfaz de la aplicación es bilingüe español/inglés (`Historias.md` §4, hito H1).
  Eso afecta a las cadenas de producto, no al idioma de trabajo.

## Qué respeta la convención del repositorio, no el español

Código, identificadores, nombres de clases, funciones, variables, componentes, módulos,
API, rutas, nombres de paquetes, claves de configuración, claves de i18n y términos
técnicos establecidos.

Los componentes existentes en `Design/src/lib/` usan nombres en inglés (`Button`, `Card`,
`StatusPill`). Esa es la convención real: mantenerla.

## Excepción

Los ficheros de `.agents/skills/speckit-*/` están en inglés porque los genera el CLI de
SpecKit y están cubiertos por sus manifiestos SHA-256. No traducirlos.
