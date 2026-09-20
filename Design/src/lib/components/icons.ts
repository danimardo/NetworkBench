/**
 * Trazos de icono compartidos (viewBox 0 0 24 24, trazo `currentColor`).
 * Nunca emojis como iconos (§16.14). Los que necesitan relleno (estrella)
 * lo indican en `fill`.
 *
 * Uso: <Icon name="home" size={19} />
 */
export type IconName =
  | "home"
  | "history"
  | "settings"
  | "wifi"
  | "star"
  | "star-outline"
  | "desktop"
  | "laptop"
  | "connect"
  | "link"
  | "check"
  // --- añadidos para H1 (§3-§7 del correo de ampliación) ---
  | "ethernet"
  | "adapter-other"
  | "shield"
  | "alert-triangle"
  | "x-circle"
  | "edit"
  | "trash"
  | "search"
  | "chevron-down"
  | "close"
  | "copy"
  | "external-link"
  | "refresh"
  | "arrow-right"
  | "clock"
  | "info"
  | "wifi-off"
  | "filter";

interface IconDef {
  /** Uno o más elementos <path>/<circle> como fragmento de markup SVG. */
  paths: string;
  strokeWidth?: number;
  fill?: "none" | "currentColor";
}

export const ICONS: Record<IconName, IconDef> = {
  home: {
    fill: "none",
    strokeWidth: 1.9,
    paths: `<path d="M4 11.5L12 4l8 7.5"/><path d="M6 10v9.2a.8.8 0 00.8.8H10v-6h4v6h3.2a.8.8 0 00.8-.8V10"/>`,
  },
  history: {
    fill: "none",
    strokeWidth: 1.9,
    paths: `<circle cx="12" cy="12" r="8.2"/><path d="M12 7.6v4.6l3.2 2"/>`,
  },
  settings: {
    fill: "none",
    strokeWidth: 1.7,
    paths: `<path d="M12 15.2a3.2 3.2 0 100-6.4 3.2 3.2 0 000 6.4z"/><path d="M19.4 15a1.7 1.7 0 00.34 1.87l.06.06a2.06 2.06 0 11-2.9 2.9l-.07-.06a1.7 1.7 0 00-1.87-.34 1.7 1.7 0 00-1.03 1.56V21.4a2.06 2.06 0 11-4.12 0v-.1a1.7 1.7 0 00-1.03-1.56 1.7 1.7 0 00-1.87.34l-.07.06a2.06 2.06 0 11-2.9-2.9l.06-.06A1.7 1.7 0 004.6 15a1.7 1.7 0 00-1.56-1.03H2.94a2.06 2.06 0 110-4.12h.1A1.7 1.7 0 004.6 8.82a1.7 1.7 0 00-.34-1.87l-.06-.07a2.06 2.06 0 112.9-2.9l.07.06A1.7 1.7 0 008.94 4.4h.09A1.7 1.7 0 0010.56 3V2.94a2.06 2.06 0 114.12 0v.1A1.7 1.7 0 0016 4.4a1.7 1.7 0 001.87-.34l.07-.06a2.06 2.06 0 112.9 2.9l-.06.07a1.7 1.7 0 00-.34 1.87v.09c.24.68.82 1.18 1.56 1.24h.1a2.06 2.06 0 110 4.12h-.1A1.7 1.7 0 0019.4 15z"/>`,
  },
  wifi: {
    fill: "none",
    strokeWidth: 2,
    paths: `<path d="M2.5 8.8a15 15 0 0119 0"/><path d="M5.5 12.3a10 10 0 0113 0"/><path d="M9 15.8a5 5 0 016 0"/>`,
  },
  star: {
    fill: "currentColor",
    strokeWidth: 0,
    paths: `<path d="M12 3l2.6 5.6 6.1.6-4.6 4.1 1.3 6-5.4-3.1-5.4 3.1 1.3-6-4.6-4.1 6.1-.6L12 3z"/>`,
  },
  "star-outline": {
    fill: "none",
    strokeWidth: 1.6,
    paths: `<path d="M12 3l2.6 5.6 6.1.6-4.6 4.1 1.3 6-5.4-3.1-5.4 3.1 1.3-6-4.6-4.1 6.1-.6L12 3z"/>`,
  },
  desktop: {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<rect x="3" y="4.5" width="18" height="11" rx="1.5"/><path d="M8 19.5h8M12 15.5v4"/>`,
  },
  connect: {
    fill: "none",
    strokeWidth: 2,
    paths: `<circle cx="4.5" cy="12" r="2.1" fill="currentColor" stroke="none"/><circle cx="19.5" cy="12" r="2.1" fill="currentColor" stroke="none"/><path d="M7 12h9"/><path d="M13 8.6l3.4 3.4-3.4 3.4"/>`,
  },
  link: {
    fill: "none",
    strokeWidth: 1.9,
    paths: `<path d="M8.5 15.5L15.5 8.5"/><path d="M7.8 12L5.4 14.4a3 3 0 004.2 4.2L12 16.2"/><path d="M16.2 12l2.4-2.4a3 3 0 00-4.2-4.2L12 7.8"/>`,
  },
  check: {
    fill: "none",
    strokeWidth: 2.4,
    paths: `<path d="M4 12.5l5 5L20 6.5"/>`,
  },
  laptop: {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<rect x="4" y="4.5" width="16" height="10.5" rx="1.3"/><path d="M2.5 19.5h19l-1.3-2.7a1 1 0 00-.9-.6H4.7a1 1 0 00-.9.6L2.5 19.5z"/>`,
  },
  /** Tipo de adaptador: Ethernet (§15 `type: "ethernet"`) — conector RJ45. */
  ethernet: {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<path d="M7 10V6.5a1 1 0 011-1h8a1 1 0 011 1V10"/><rect x="4.5" y="10" width="15" height="8" rx="1.3"/><path d="M9 10v3M12 10v3M15 10v3"/>`,
  },
  /** Tipo de adaptador genérico: virtual / VPN / loopback / other (§15). */
  "adapter-other": {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<rect x="3.5" y="6" width="17" height="12" rx="2"/><path d="M8 11.2h8M8 14.2h5"/>`,
  },
  /** Confianza (§9.2) — deliberadamente distinto de la estrella de favorito
      para no confundir "confío en este equipo" con "marcado como favorito". */
  shield: {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<path d="M12 3.2l7 2.6v5.4c0 4.5-3 7.6-7 9.6-4-2-7-5.1-7-9.6V5.8l7-2.6z"/><path d="M9 12l2.2 2.2L15.5 9.5"/>`,
  },
  "alert-triangle": {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<path d="M12 4l9.5 16.5H2.5L12 4z"/><path d="M12 10.2v4.2"/><circle cx="12" cy="17.3" r="0.9" fill="currentColor" stroke="none"/>`,
  },
  "x-circle": {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<circle cx="12" cy="12" r="8.5"/><path d="M9 9l6 6M15 9l-6 6"/>`,
  },
  edit: {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<path d="M4 20l.9-3.9L16.6 4.4a1.5 1.5 0 012.1 0l1 1a1.5 1.5 0 010 2.1L8 19.2 4 20z"/><path d="M14.5 6.5l3 3"/>`,
  },
  trash: {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<path d="M4.5 7h15"/><path d="M9 7V5a1 1 0 011-1h4a1 1 0 011 1v2"/><path d="M6.5 7l.7 12.1a1.5 1.5 0 001.5 1.4h6.6a1.5 1.5 0 001.5-1.4L17.5 7"/><path d="M10 11v6M14 11v6"/>`,
  },
  search: {
    fill: "none",
    strokeWidth: 1.9,
    paths: `<circle cx="10.8" cy="10.8" r="6.3"/><path d="M19.5 19.5l-4.3-4.3"/>`,
  },
  "chevron-down": {
    fill: "none",
    strokeWidth: 2,
    paths: `<path d="M5.5 8.5l6.5 7 6.5-7"/>`,
  },
  close: {
    fill: "none",
    strokeWidth: 1.9,
    paths: `<path d="M6 6l12 12M18 6L6 18"/>`,
  },
  copy: {
    fill: "none",
    strokeWidth: 1.7,
    paths: `<rect x="8.5" y="8.5" width="11" height="11" rx="1.5"/><path d="M15 8.5V6a1.5 1.5 0 00-1.5-1.5H6A1.5 1.5 0 004.5 6v7.5A1.5 1.5 0 006 15h2.5"/>`,
  },
  "external-link": {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<path d="M9.5 5.5h-4a1 1 0 00-1 1v12a1 1 0 001 1h12a1 1 0 001-1v-4"/><path d="M13 4.5h6.5V11"/><path d="M19 5l-9 9"/>`,
  },
  refresh: {
    fill: "none",
    strokeWidth: 1.9,
    paths: `<path d="M4.5 12a7.5 7.5 0 0112.9-5.2M19.5 12a7.5 7.5 0 01-12.9 5.2"/><path d="M17 4.5v3.5h-3.5M7 19.5V16h3.5"/>`,
  },
  "arrow-right": {
    fill: "none",
    strokeWidth: 2,
    paths: `<path d="M4.5 12h14.5"/><path d="M13.5 6.5L19.5 12l-6 5.5"/>`,
  },
  clock: {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<circle cx="12" cy="12" r="8.3"/><path d="M12 7.5V12l3.3 2"/>`,
  },
  info: {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<circle cx="12" cy="12" r="8.3"/><path d="M12 11v5.3" stroke-linecap="round"/><circle cx="12" cy="8.2" r="0.9" fill="currentColor" stroke="none"/>`,
  },
  "wifi-off": {
    fill: "none",
    strokeWidth: 2,
    paths: `<path d="M2.5 8.8a15 15 0 013.6-2.5M9 15.8a5 5 0 016 0M5.5 12.3a10 10 0 013-2.1M21.5 8.8a15 15 0 00-4.4-3" stroke-linecap="round"/><path d="M2.5 3l19 19" stroke-linecap="round"/>`,
  },
  filter: {
    fill: "none",
    strokeWidth: 1.8,
    paths: `<path d="M4 5.5h16l-6 7v6l-4 2v-8l-6-7z"/>`,
  },
};
