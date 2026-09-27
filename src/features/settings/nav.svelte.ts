/**
 * Petición de abrir Ajustes en una pestaña concreta desde fuera (p. ej. el aviso de
 * cortafuegos del arranque). `SettingsScreen` la consume al montarse y la vacía.
 */
export const settingsNav = $state<{ requestedTab: string | null }>({ requestedTab: null });
