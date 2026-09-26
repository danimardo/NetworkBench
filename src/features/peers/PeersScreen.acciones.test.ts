import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import { tick } from "svelte";
import PeersScreen from "./PeersScreen.svelte";
import { setLocale } from "../../lib/i18n";
import type { Peer } from "../../lib/contracts/peer";

const equipo: Peer = {
  instanceId: "11111111-1111-4111-8111-111111111111",
  displayName: "PC-Sobremesa",
  fingerprint: "1".repeat(64),
  addresses: ["192.168.1.10:7411"],
  trustState: "trusted",
  autoAccept: false,
  lastSeen: "2026-09-26T10:00:00Z",
};

afterEach(() => setLocale("es"));

describe("Inicio: conectar por IP y volver a buscar", () => {
  it("con la lista vacía hay un único «Conectar por IP» (el del centro) y ninguno en la cabecera", () => {
    render(PeersScreen, { props: { peers: [], isScanning: false } });

    expect(screen.getAllByRole("button", { name: /Conectar por IP/ })).toHaveLength(1);
    expect(screen.queryByTestId("manual-connect-btn")).toBeNull();
    expect(screen.queryByText("Conectar manualmente")).toBeNull();
  });

  it("con equipos en la lista, «Conectar por IP» pasa a la cabecera y el del centro desaparece", () => {
    render(PeersScreen, { props: { peers: [equipo], isScanning: false } });

    const botones = screen.getAllByRole("button", { name: /Conectar por IP/ });
    expect(botones).toHaveLength(1);
    expect(botones[0]!.getAttribute("data-testid")).toBe("manual-connect-btn");
    expect(screen.queryByTestId("manual-connect-empty-btn")).toBeNull();
  });

  it("el icono del botón de la cabecera va a la izquierda del texto, en la misma línea", () => {
    render(PeersScreen, { props: { peers: [equipo], isScanning: false } });

    const boton = screen.getByTestId("manual-connect-btn");
    const hijos = [...boton.children].map((h) => h.className);
    // El icono (span.nb-button-icon) precede a la etiqueta; antes iba dentro del texto y se apilaba.
    expect(hijos[0]).toContain("nb-button-icon");
    expect(hijos[1]).toContain("nb-button-label");
    expect(boton.querySelector(".nb-button-label svg")).toBeNull();
  });

  it("la lupa del estado vacío es un botón que vuelve a buscar", async () => {
    const onRescan = vi.fn();
    render(PeersScreen, { props: { peers: [], isScanning: false, onRescan } });

    const lupa = screen.getByTestId("rescan-empty-btn");
    expect(lupa.tagName).toBe("BUTTON");
    expect(lupa.getAttribute("aria-label")).toBe("Buscar de nuevo");
    await fireEvent.click(lupa);
    expect(onRescan).toHaveBeenCalledTimes(1);
  });

  it("el botón de refrescar de la cabecera existe también con equipos en la lista", async () => {
    const onRescan = vi.fn();
    render(PeersScreen, { props: { peers: [equipo], isScanning: false, onRescan } });

    await fireEvent.click(screen.getByTestId("rescan-btn"));
    expect(onRescan).toHaveBeenCalledTimes(1);
  });

  it("mientras busca, los botones de buscar quedan desactivados y no piden otra búsqueda", async () => {
    const onRescan = vi.fn();
    render(PeersScreen, { props: { peers: [], isScanning: true, onRescan } });

    const cabecera = screen.getByTestId("rescan-btn") as HTMLButtonElement;
    const lupa = screen.getByTestId("rescan-empty-btn") as HTMLButtonElement;
    expect(cabecera.disabled).toBe(true);
    expect(lupa.disabled).toBe(true);
    expect(screen.getByText("Buscando equipos en la red local...")).toBeTruthy();
    // Con «Buscando…» tampoco se ofrece la conexión por IP del centro.
    expect(screen.queryByTestId("manual-connect-empty-btn")).toBeNull();
    await fireEvent.click(cabecera);
    expect(onRescan).not.toHaveBeenCalled();
  });

  it("toda la pantalla de Inicio se traduce al inglés sin volver a montarla", async () => {
    render(PeersScreen, { props: { peers: [], isScanning: false } });
    expect(screen.getByText("Equipos disponibles")).toBeTruthy();

    setLocale("en");
    await tick();

    expect(screen.getByText("Available devices")).toBeTruthy();
    expect(screen.getByText("No devices detected")).toBeTruthy();
    expect(screen.getByRole("button", { name: /Connect by IP/ })).toBeTruthy();
    // La cabecera y la lupa del estado vacío: los dos vuelven a buscar.
    expect(screen.getAllByRole("button", { name: "Search again" })).toHaveLength(2);
    expect(screen.queryByText("Equipos disponibles")).toBeNull();
  });

  it("el diálogo de conexión manual también está en inglés", async () => {
    setLocale("en");
    render(PeersScreen, { props: { peers: [], isScanning: false } });

    await fireEvent.click(screen.getByRole("button", { name: /Connect by IP/ }));
    expect(await screen.findByText("Manual connection by IP")).toBeTruthy();
    expect(screen.getByLabelText("IP address or hostname")).toBeTruthy();
    expect(screen.getByLabelText("Control port")).toBeTruthy();
  });
});
