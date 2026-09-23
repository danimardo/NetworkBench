import { describe, it, expect, vi } from "vitest";
import { render, fireEvent, screen } from "@testing-library/svelte";
import PeersScreen from "./PeersScreen.svelte";
import type { Peer } from "../../lib/contracts/peer";

describe("T034 - PeersScreen UI & Accessibility", () => {
  const samplePeers: Peer[] = [
    {
      instanceId: "11111111-1111-4111-8111-111111111111",
      displayName: "PC-Sobremesa",
      fingerprint: "1111111111111111111111111111111111111111111111111111111111111111",
      addresses: ["192.168.1.10:7411"],
      trustState: "trusted",
      autoAccept: false,
      lastSeen: "2026-09-21T20:00:00Z",
    },
    {
      instanceId: "22222222-2222-4222-8222-222222222222",
      displayName: "Portatil-Dani",
      fingerprint: "2222222222222222222222222222222222222222222222222222222222222222",
      addresses: ["192.168.1.20:7411"],
      trustState: "unknown",
      autoAccept: false,
      lastSeen: "2026-09-21T20:05:00Z",
    },
  ];

  it("muestra el estado vacío accesible cuando no hay equipos detectados", () => {
    render(PeersScreen, {
      peers: [],
      isScanning: false,
    });

    expect(screen.getByText("No se han detectado equipos")).toBeTruthy();
    expect(screen.getByRole("status")).toBeTruthy();
  });

  it("renderiza la lista de peers con su información correspondiente", () => {
    render(PeersScreen, {
      peers: samplePeers,
      isScanning: false,
    });

    expect(screen.getByText("PC-Sobremesa")).toBeTruthy();
    expect(screen.getByText("Portatil-Dani")).toBeTruthy();
    expect(screen.getByText("192.168.1.10:7411")).toBeTruthy();
  });

  it("permite abrir el diálogo de conexión manual y valida el puerto", async () => {
    const onManualConnect = vi.fn();
    render(PeersScreen, {
      peers: samplePeers,
      onManualConnect,
    });

    const manualBtn = screen.getByTestId("manual-connect-btn");
    await fireEvent.click(manualBtn);

    // Diálogo abierto
    expect(screen.getByText("Conexión manual por IP")).toBeTruthy();

    const hostInput = screen.getByLabelText("Dirección IP o Hostname");
    const portInput = screen.getByLabelText("Puerto de control");
    const submitBtn = screen.getByText("Conectar");

    // Probar puerto inválido < 1024
    await fireEvent.input(hostInput, { target: { value: "192.168.1.50" } });
    await fireEvent.input(portInput, { target: { value: "80" } });
    await fireEvent.click(submitBtn);

    expect(screen.getByText("El puerto debe ser un número entre 1024 y 65535")).toBeTruthy();
    expect(onManualConnect).not.toHaveBeenCalled();

    // Probar puerto válido
    await fireEvent.input(portInput, { target: { value: "7412" } });
    await fireEvent.click(submitBtn);

    expect(onManualConnect).toHaveBeenCalledWith("192.168.1.50", 7412);
  });

  it("muestra el diálogo de pairing con código de 6 dígitos y botón Cancelar accesible", async () => {
    const onCancelPairing = vi.fn();
    const onConfirmPairing = vi.fn();

    render(PeersScreen, {
      peers: samplePeers,
      activePairingPeer: samplePeers[1],
      pairingCode: "458921",
      pairingSecondsLeft: 55,
      onCancelPairing,
      onConfirmPairing,
    });

    expect(screen.getByText("Emparejar equipo nuevo")).toBeTruthy();
    expect(screen.getByLabelText(/código de verificación 458921/i)).toBeTruthy();
    expect(screen.getByText("Caduca en 55 s")).toBeTruthy();

    const cancelBtn = screen.getByTestId("pairing-cancel-btn");
    const confirmBtn = screen.getByTestId("pairing-confirm-btn");

    await fireEvent.click(confirmBtn);
    expect(onConfirmPairing).toHaveBeenCalledWith(samplePeers[1], "458921");

    await fireEvent.click(cancelBtn);
    expect(onCancelPairing).toHaveBeenCalled();
  });
});
