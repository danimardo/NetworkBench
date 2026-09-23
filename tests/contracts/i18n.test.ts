import { describe, it, expect, beforeEach } from "vitest";
import {
  getLocale,
  setLocale,
  subscribeLocale,
  t,
  formatThroughput,
  formatBytes,
  formatNumber,
  formatDateTime,
} from "../../src/lib/i18n";

describe("i18n Unit & Contract Tests", () => {
  beforeEach(() => {
    setLocale("es");
  });

  it("permite cambiar y consultar el idioma activo", () => {
    expect(getLocale()).toBe("es");
    setLocale("en");
    expect(getLocale()).toBe("en");
    setLocale("es");
    expect(getLocale()).toBe("es");
  });

  it("notifica a los suscriptores cuando cambia el idioma", () => {
    const history: string[] = [];
    const unsubscribe = subscribeLocale((loc) => history.push(loc));

    setLocale("en");
    setLocale("es");
    unsubscribe();
    setLocale("en");

    expect(history).toEqual(["es", "en", "es"]);
  });

  it("traduce claves anidadas en español y en inglés", () => {
    setLocale("es");
    expect(t("nav.peers")).toBe("Equipos");
    expect(t("errors.NB-CONN-001")).toContain("No se puede alcanzar");

    setLocale("en");
    expect(t("nav.peers")).toBe("Peers");
    expect(t("errors.NB-CONN-001")).toContain("Cannot reach target");
  });

  it("hace fallback seguro a la clave si no existe", () => {
    expect(t("non.existent.key")).toBe("non.existent.key");
  });

  it("formatea números según la configuración regional", () => {
    expect(formatNumber(1234.56, 2, "es")).toBe("1.234,56");
    expect(formatNumber(1234.56, 2, "en")).toBe("1,234.56");
  });

  it("formatea throughput correctamente en distintas magnitudes", () => {
    expect(formatThroughput(500, "es")).toBe("500 bps");
    expect(formatThroughput(10_000, "es")).toBe("10 Kbps");
    expect(formatThroughput(945_500_000, "es")).toMatch(/945[,.]5 Mbps/);
    expect(formatThroughput(1_250_000_000n, "en")).toBe("1.25 Gbps");
  });

  it("formatea bytes correctamente en magnitudes binarias", () => {
    expect(formatBytes(512, "es")).toBe("512 B");
    expect(formatBytes(2048, "es")).toBe("2 KB");
    expect(formatBytes(50 * 1024 * 1024, "es")).toBe("50 MB");
    expect(formatBytes(2n * 1024n * 1024n * 1024n, "en")).toBe("2 GB");
  });

  it("formatea fechas con zona Europe/Madrid", () => {
    const d = new Date("2026-07-15T10:30:00Z");
    // Verano CEST (UTC+2) -> 12:30:00
    const str = formatDateTime(d, "es");
    expect(str).toMatch(/12:30:00/);
  });
});
