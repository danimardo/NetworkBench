import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import PeersScreen from "./PeersScreen.svelte";
import { setLocale } from "../../lib/i18n";
import type { Peer } from "../../lib/contracts/peer";

const equipo: Peer = {
  instanceId: "11111111-1111-4111-8111-111111111111",
  displayName: "WIN11D",
  fingerprint: "1".repeat(64),
  addresses: ["192.168.1.50:7411"],
  trustState: "trusted",
  autoAccept: false,
  lastSeen: "2026-09-30T10:00:00Z",
};

afterEach(() => setLocale("es"));

async function abrirMenu(props: Record<string, unknown> = {}, peer: Peer = equipo) {
  render(PeersScreen, { props: { peers: [peer], ...props } });
  await fireEvent.contextMenu(screen.getByRole("listitem"), { clientX: 40, clientY: 40 });
  return screen.findByRole("menu");
}

describe("Inicio: dirección de un equipo", () => {
  it("un equipo guardado sin dirección no inventa 127.0.0.1", () => {
    render(PeersScreen, { props: { peers: [{ ...equipo, addresses: [] }] } });

    expect(screen.queryByText("127.0.0.1")).toBeNull();
    expect(screen.getByText("Dirección desconocida")).toBeTruthy();
  });
});

describe("Inicio: menú contextual de un equipo", () => {
  it("el clic derecho abre un menú con las cinco acciones", async () => {
    await abrirMenu();

    const items = screen.getAllByRole("menuitem").map((i) => i.textContent?.trim());
    expect(items).toEqual([
      "Marcar como favorito",
      "Copiar IP",
      "Comprobar ahora",
      "Quitar marca de confianza",
      "Eliminar equipo",
    ]);
  });

  it("«Quitar marca de confianza» avisa con el equipo y cierra el menú", async () => {
    const onRevokeTrust = vi.fn();
    await abrirMenu({ onRevokeTrust });

    await fireEvent.click(screen.getByRole("menuitem", { name: "Quitar marca de confianza" }));

    expect(onRevokeTrust).toHaveBeenCalledWith(equipo);
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("«Marcar como favorito» avisa con el equipo", async () => {
    const onToggleFavorite = vi.fn();
    await abrirMenu({ onToggleFavorite });

    await fireEvent.click(screen.getByRole("menuitem", { name: "Marcar como favorito" }));

    expect(onToggleFavorite).toHaveBeenCalledWith(equipo);
  });

  it("un equipo favorito ofrece quitarlo de favoritos", async () => {
    await abrirMenu({}, { ...equipo, favorite: true });

    expect(screen.getByRole("menuitem", { name: "Quitar de favoritos" })).toBeTruthy();
  });

  it("«Eliminar equipo» pide confirmación y no elimina hasta confirmar", async () => {
    const onForget = vi.fn();
    await abrirMenu({ onForget });

    await fireEvent.click(screen.getByRole("menuitem", { name: "Eliminar equipo" }));
    expect(onForget).not.toHaveBeenCalled();
    expect(await screen.findByText("¿Eliminar WIN11D?")).toBeTruthy();

    await fireEvent.click(screen.getByTestId("forget-confirm-btn"));
    expect(onForget).toHaveBeenCalledWith(equipo);
  });

  it("cancelar la confirmación no elimina nada", async () => {
    const onForget = vi.fn();
    await abrirMenu({ onForget });

    await fireEvent.click(screen.getByRole("menuitem", { name: "Eliminar equipo" }));
    await fireEvent.click(await screen.findByTestId("forget-cancel-btn"));

    expect(onForget).not.toHaveBeenCalled();
  });

  it("Escape cierra el menú sin hacer nada", async () => {
    const onRevokeTrust = vi.fn();
    const menu = await abrirMenu({ onRevokeTrust });

    await fireEvent.keyDown(menu, { key: "Escape" });

    expect(screen.queryByRole("menu")).toBeNull();
    expect(onRevokeTrust).not.toHaveBeenCalled();
  });

  it("ArrowDown mueve el foco entre las acciones activas", async () => {
    const menu = await abrirMenu();
    const [primero, segundo] = screen.getAllByRole("menuitem");
    expect(document.activeElement).toBe(primero);

    await fireEvent.keyDown(menu, { key: "ArrowDown" });

    expect(document.activeElement).toBe(segundo);
  });

  it("un equipo no guardado no permite quitar confianza ni eliminar", async () => {
    await abrirMenu({ isSaved: () => false }, { ...equipo, trustState: "unknown" });

    for (const nombre of ["Marcar como favorito", "Quitar marca de confianza", "Eliminar equipo"]) {
      expect((screen.getByRole("menuitem", { name: nombre }) as HTMLButtonElement).disabled).toBe(
        true,
      );
    }
  });

  it("un equipo sin dirección no permite copiar la IP", async () => {
    await abrirMenu({}, { ...equipo, addresses: [] });

    expect(
      (screen.getByRole("menuitem", { name: "Copiar IP" }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  it("«Copiar IP» copia solo el host, sin el puerto de control", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    await abrirMenu();

    await fireEvent.click(screen.getByRole("menuitem", { name: "Copiar IP" }));

    expect(writeText).toHaveBeenCalledWith("192.168.1.50");
  });
});

describe("Inicio: equipos inalcanzables", () => {
  const caido = { ...equipo, lastSeen: new Date(Date.now() - 2 * 3600 * 1000).toISOString() };

  it("un equipo inalcanzable dice «No accesible» y cuánto hace que se vio", () => {
    render(PeersScreen, {
      props: {
        peers: [caido],
        statusOf: () => ({ availability: "unreachable", compatible: true }),
      },
    });

    expect(screen.getByText("No accesible")).toBeTruthy();
    expect(screen.getByText(/Visto hace 2 horas/)).toBeTruthy();
  });

  it("un equipo disponible no muestra «Visto hace»", () => {
    render(PeersScreen, { props: { peers: [caido] } });

    expect(screen.queryByText(/Visto/)).toBeNull();
  });

  it("un equipo que se está comprobando lo dice", () => {
    render(PeersScreen, {
      props: { peers: [equipo], statusOf: () => ({ availability: "checking", compatible: true }) },
    });

    expect(screen.getByText("Comprobando…")).toBeTruthy();
  });

  it("el menú ofrece «Comprobar ahora» y avisa con el equipo", async () => {
    const onCheckNow = vi.fn();
    await abrirMenu({ onCheckNow });

    await fireEvent.click(screen.getByRole("menuitem", { name: "Comprobar ahora" }));

    expect(onCheckNow).toHaveBeenCalledWith(equipo);
  });
});
