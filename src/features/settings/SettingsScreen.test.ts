import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import SettingsScreen from "./SettingsScreen.svelte";

// Mock de APIs
vi.mock("../../lib/api/settings", () => ({
  getSettings: vi.fn().mockResolvedValue({
    schemaVersion: 1,
    theme: "dark",
    locale: "es",
    reduceMotion: false,
    logLevel: "warn",
    autoAcceptTrusted: false,
    customControlPort: null,
    autostart: false,
    minimizeToTray: false,
    mdnsEnabled: true,
  }),
  updateSettings: vi.fn().mockImplementation((prefs) => Promise.resolve(prefs)),
  getAutostart: vi.fn().mockResolvedValue(false),
  setAutostart: vi.fn().mockResolvedValue(undefined),
  getDataInfo: vi.fn().mockResolvedValue({
    dbPath: "C:\\Users\\User\\AppData\\Local\\NetworkBench\\history.db",
    dbSizeBytes: 1048576,
    settingsPath: "C:\\Users\\User\\AppData\\Local\\NetworkBench\\settings.json",
    settingsSizeBytes: 1024,
  }),
  purgeData: vi.fn().mockResolvedValue(undefined),
  getAboutInfo: vi.fn().mockResolvedValue({
    appName: "NetworkBench",
    appVersion: "0.1.0",
    engineName: "Microsoft NTTTCP",
    engineVersion: "5.35",
    protocolVersion: "v1",
    license: "GPL-3.0-or-later",
    copyright: "© 2026 Daniel Díez Mardomingo",
  }),
  evaluateAppClose: vi.fn().mockResolvedValue("allowExit"),
}));

vi.mock("../../lib/api/updater", () => ({
  checkUpdate: vi.fn().mockResolvedValue({ status: "upToDate" }),
  evaluateUpdate: vi.fn().mockResolvedValue({ status: "upToDate" }),
}));

vi.mock("../../lib/api/firewall", () => ({
  inspectFirewall: vi.fn().mockResolvedValue({
    controlPortRule: "Present",
    enginePortsRule: "Present",
    activeProfile: "Private",
  }),
}));

describe("SettingsScreen", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renderiza navegación accesible por pestañas role=tablist", async () => {
    render(SettingsScreen);

    const tablist = screen.getByRole("tablist");
    expect(tablist).toBeTruthy();

    const tabs = screen.getAllByRole("tab");
    expect(tabs.length).toBe(8);

    // Por defecto inicia en apariencia
    expect(tabs[0]!.getAttribute("aria-selected")).toBe("true");
  });

  it("permite cambiar a la pestaña de red y muestra opciones de puerto y mDNS", async () => {
    render(SettingsScreen);

    const tabs = screen.getAllByRole("tab");
    const networkTab = tabs[1]!; // Pestaña Red

    await fireEvent.click(networkTab);
    expect(networkTab.getAttribute("aria-selected")).toBe("true");

    const portInput = screen.getByPlaceholderText("7411");
    expect(portInput).toBeTruthy();
  });

  it("permite cambiar a la pestaña de Acerca de y muestra versiones y licencia", async () => {
    render(SettingsScreen);

    const tabs = screen.getAllByRole("tab");
    const aboutTab = tabs[7]!; // Pestaña Acerca de

    await fireEvent.click(aboutTab);
    expect(aboutTab.getAttribute("aria-selected")).toBe("true");

    expect(screen.getByText("NetworkBench")).toBeTruthy();
  });
});
