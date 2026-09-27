import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import CloseFlow from "./CloseFlow.svelte";
import App from "./App.svelte";
import { setTransportMock } from "../lib/api/transport";
import { setLocale } from "../lib/i18n";

// Cierre de la ventana (Historias.md §5.2). El backend intercepta toda vía de cierre, decide
// y solo cuando hay que preguntar avisa por un evento; aquí se comprueba que la interfaz
// muestra la pregunta, devuelve lo elegido y no hace nada en los demás casos.
//
// Antes el cierre dependía de que la interfaz llamara a `window.destroy()`; sin ese permiso
// el botón de cerrar no hacía nada.

const escuchas = vi.hoisted(() => ({
  manejadores: new Map<string, (e: { payload: unknown }) => void>(),
  dejaron: [] as string[],
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (evento: string, cb: (e: { payload: unknown }) => void) => {
    escuchas.manejadores.set(evento, cb);
    return () => {
      escuchas.dejaron.push(evento);
      escuchas.manejadores.delete(evento);
    };
  }),
}));

function emitir(evento: string, payload: unknown = null) {
  const m = escuchas.manejadores.get(evento);
  if (!m) throw new Error(`nadie escucha ${evento}`);
  m({ payload });
}

let comandos: { cmd: string; args?: Record<string, unknown> }[] = [];

beforeEach(() => {
  comandos = [];
  escuchas.manejadores.clear();
  escuchas.dejaron.length = 0;
  setTransportMock(async (cmd, args) => {
    comandos.push({ cmd, args });
    return { ok: true, value: null };
  });
});
afterEach(() => {
  setTransportMock(null);
  setLocale("es");
});

async function montado() {
  const r = render(CloseFlow);
  await waitFor(() => expect(escuchas.manejadores.has("app://close-ask")).toBe(true));
  return r;
}

describe("CloseFlow: «¿Qué quieres hacer?»", () => {
  it("al pedir cerrar el backend, muestra la pregunta con las dos opciones y la casilla", async () => {
    await montado();
    expect(screen.queryByRole("dialog")).toBeNull();

    emitir("app://close-ask", { sessionActive: false });

    expect(await screen.findByText("¿Qué quieres hacer?")).toBeTruthy();
    expect(screen.getByText(/seguir disponible en segundo plano/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Minimizar a la bandeja" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Cerrar NetworkBench" })).toBeTruthy();
    expect(screen.getByLabelText("Recordar mi decisión")).toBeTruthy();
  });

  it("«Minimizar a la bandeja» devuelve minimize sin recordar", async () => {
    await montado();
    emitir("app://close-ask", { sessionActive: false });
    await fireEvent.click(await screen.findByRole("button", { name: "Minimizar a la bandeja" }));

    await waitFor(() => expect(comandos).toHaveLength(1));
    expect(comandos[0]).toEqual({
      cmd: "app_close_apply",
      args: { choice: "minimize", remember: false },
    });
    expect(screen.queryByText("¿Qué quieres hacer?")).toBeNull();
  });

  it("«Cerrar NetworkBench» con «Recordar mi decisión» devuelve exit y recordar", async () => {
    await montado();
    emitir("app://close-ask", { sessionActive: false });
    await fireEvent.click(await screen.findByLabelText("Recordar mi decisión"));
    await fireEvent.click(screen.getByRole("button", { name: "Cerrar NetworkBench" }));

    await waitFor(() => expect(comandos).toHaveLength(1));
    expect(comandos[0]).toEqual({
      cmd: "app_close_apply",
      args: { choice: "exit", remember: true },
    });
  });

  it("Escape no hace nada: cierra el diálogo y no manda ningún comando", async () => {
    await montado();
    emitir("app://close-ask", { sessionActive: false });
    await screen.findByText("¿Qué quieres hacer?");

    await fireEvent.keyDown(document.body, { key: "Escape" });
    await waitFor(() => expect(screen.queryByText("¿Qué quieres hacer?")).toBeNull());
    expect(comandos).toEqual([]);
  });

  it("sale en inglés con el idioma activo", async () => {
    setLocale("en");
    await montado();
    emitir("app://close-ask", { sessionActive: false });

    expect(await screen.findByText("What do you want to do?")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Minimize to tray" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Close NetworkBench" })).toBeTruthy();
    expect(screen.getByLabelText("Remember my choice")).toBeTruthy();
  });
});

describe("CloseFlow: salir con una prueba en curso", () => {
  it("avisa de que se cancelará la prueba y solo sale si se confirma", async () => {
    await montado();
    await waitFor(() => expect(escuchas.manejadores.has("app://close-confirm-session")).toBe(true));

    emitir("app://close-confirm-session");
    expect(await screen.findByText(/prueba en curso/i)).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Salir y cancelar" }));
    await waitFor(() => expect(comandos).toHaveLength(1));
    expect(comandos[0]!.cmd).toBe("app_close_confirmed");
  });

  it("«Seguir con la prueba» no cancela ni sale", async () => {
    await montado();
    await waitFor(() => expect(escuchas.manejadores.has("app://close-confirm-session")).toBe(true));
    emitir("app://close-confirm-session");

    await fireEvent.click(await screen.findByRole("button", { name: "Seguir con la prueba" }));
    expect(comandos).toEqual([]);
  });
});

describe("CloseFlow: ciclo de vida", () => {
  it("al desmontarse deja de escuchar los dos eventos", async () => {
    const { unmount } = await montado();
    await waitFor(() => expect(escuchas.manejadores.has("app://close-confirm-session")).toBe(true));
    unmount();
    expect([...escuchas.dejaron].sort()).toEqual([
      "app://close-ask",
      "app://close-confirm-session",
    ]);
  });

  it("un payload raro no rompe la pregunta", async () => {
    await montado();
    emitir("app://close-ask", "basura");
    expect(await screen.findByText("¿Qué quieres hacer?")).toBeTruthy();
  });
});

describe("App: cableado del cierre", () => {
  it("la aplicación entera muestra la pregunta cuando el backend la pide", async () => {
    setTransportMock(async (cmd) => {
      comandos.push({ cmd });
      const ok = (value: unknown) => ({ ok: true, value });
      if (cmd === "app_get_snapshot")
        return ok({
          revision: 1,
          appVersion: "0.1.0",
          locale: "es",
          theme: "dark",
          instanceId: "x",
          instanceName: "PC",
          isSessionActive: false,
          activeSessionId: null,
          peersCount: 0,
        });
      if (cmd === "firewall_rules_status")
        return ok({
          reglas: [],
          redes: [],
          permitirPublico: false,
          puertoControl: 7411,
          ayudanteDisponible: true,
        });
      if (
        [
          "peers_list",
          "peers_discovered_list",
          "session_incoming_list",
          "peers_pairing_incoming_list",
        ].includes(cmd)
      )
        return ok([]);
      return ok(null);
    });
    render(App);
    await waitFor(() => expect(escuchas.manejadores.has("app://close-ask")).toBe(true), {
      timeout: 4000,
    });

    emitir("app://close-ask", { sessionActive: false });
    expect(await screen.findByText("¿Qué quieres hacer?")).toBeTruthy();
  });
});
