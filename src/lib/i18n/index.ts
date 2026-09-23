import esCatalog from "../../../locales/es.json";
import enCatalog from "../../../locales/en.json";

export type SupportedLocale = "es" | "en";

export interface TranslationCatalogs {
  [key: string]: unknown;
}

const catalogs: Record<SupportedLocale, TranslationCatalogs> = {
  es: esCatalog,
  en: enCatalog,
};

let currentLocale: SupportedLocale = "es";
const subscribers = new Set<(locale: SupportedLocale) => void>();

function resolveKey(obj: unknown, path: string): string | undefined {
  const parts = path.split(".");
  let curr: unknown = obj;
  for (const part of parts) {
    if (curr && typeof curr === "object" && part in curr) {
      curr = (curr as Record<string, unknown>)[part];
    } else {
      return undefined;
    }
  }
  return typeof curr === "string" ? curr : undefined;
}

export function getLocale(): SupportedLocale {
  return currentLocale;
}

export function setLocale(locale: SupportedLocale): void {
  if (locale !== "es" && locale !== "en") {
    locale = "es";
  }
  currentLocale = locale;
  for (const sub of subscribers) {
    sub(currentLocale);
  }
}

export function subscribeLocale(callback: (locale: SupportedLocale) => void): () => void {
  subscribers.add(callback);
  callback(currentLocale);
  return () => {
    subscribers.delete(callback);
  };
}

export function t(key: string, params?: Record<string, string | number>): string {
  let template = resolveKey(catalogs[currentLocale], key);
  if (template === undefined && currentLocale !== "es") {
    template = resolveKey(catalogs.es, key);
  }
  if (template === undefined) {
    return key;
  }
  if (!params) {
    return template;
  }
  return template.replace(/\{(\w+)\}/g, (_, k) => {
    return k in params ? String(params[k]) : `{${k}}`;
  });
}

export function formatNumber(
  val: number | bigint,
  decimals = 2,
  locale: SupportedLocale = currentLocale,
): string {
  const num = typeof val === "bigint" ? Number(val) : val;
  return new Intl.NumberFormat(locale === "es" ? "es-ES" : "en-US", {
    useGrouping: true,
    minimumFractionDigits: 0,
    maximumFractionDigits: decimals,
  }).format(num);
}

export function formatThroughput(
  bps: number | bigint,
  locale: SupportedLocale = currentLocale,
): string {
  const b = typeof bps === "bigint" ? Number(bps) : bps;
  if (b >= 1_000_000_000) {
    const gbps = b / 1_000_000_000;
    return `${formatNumber(gbps, 2, locale)} Gbps`;
  }
  if (b >= 1_000_000) {
    const mbps = b / 1_000_000;
    return `${formatNumber(mbps, 2, locale)} Mbps`;
  }
  if (b >= 1_000) {
    const kbps = b / 1_000;
    return `${formatNumber(kbps, 2, locale)} Kbps`;
  }
  return `${formatNumber(b, 0, locale)} bps`;
}

export function formatBytes(
  bytes: number | bigint,
  locale: SupportedLocale = currentLocale,
): string {
  const b = typeof bytes === "bigint" ? Number(bytes) : bytes;
  if (b >= 1024 * 1024 * 1024) {
    const gb = b / (1024 * 1024 * 1024);
    return `${formatNumber(gb, 2, locale)} GB`;
  }
  if (b >= 1024 * 1024) {
    const mb = b / (1024 * 1024);
    return `${formatNumber(mb, 2, locale)} MB`;
  }
  if (b >= 1024) {
    const kb = b / 1024;
    return `${formatNumber(kb, 2, locale)} KB`;
  }
  return `${formatNumber(b, 0, locale)} B`;
}

export function formatDateTime(
  date: Date | string | number,
  locale: SupportedLocale = currentLocale,
): string {
  const d = typeof date === "object" ? date : new Date(date);
  return new Intl.DateTimeFormat(locale === "es" ? "es-ES" : "en-US", {
    timeZone: "Europe/Madrid",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).format(d);
}
