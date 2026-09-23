import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import HistoryScreen from "./HistoryScreen.svelte";
import * as historyApi from "../../lib/api/history";

vi.mock("../../lib/api/history", () => ({
  listHistory: vi.fn(),
  getHistoryDetail: vi.fn(),
  compareCohort: vi.fn(),
  deletePreview: vi.fn(),
  deleteConfirm: vi.fn(),
  repeatPlan: vi.fn(),
  getPeerTrend: vi.fn(),
}));

describe("T079 - HistoryScreen UI Component Tests (US4)", () => {
  const sampleItems = [
    {
      id: "11111111-1111-4111-8111-111111111111",
      createdAt: "2026-09-22T06:00:00Z",
      peerId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      peerName: "Servidor-Local",
      peerFingerprint: "sha256:abc",
      status: "completed",
      protocol: "tcp",
      streams: 1,
      durationSeconds: 10,
      forwardBps: "9420000000",
      reverseBps: "9310000000",
      isPartial: false,
      diagnosticVerdict: '{"level":"ok"}',
      clientInterface: "Ethernet 10G",
      serverInterface: "Ethernet 10G",
    },
    {
      id: "22222222-2222-4222-8222-222222222222",
      createdAt: "2026-09-21T14:30:00Z",
      peerId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
      peerName: "Portatil-Wifi",
      peerFingerprint: "sha256:def",
      status: "completed",
      protocol: "udp",
      streams: 2,
      durationSeconds: 15,
      forwardBps: "450000000",
      reverseBps: null,
      isPartial: true,
      diagnosticVerdict: '{"level":"warn"}',
      clientInterface: "Wi-Fi 6",
      serverInterface: "Ethernet 1G",
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("muestra el estado vacío cuando no hay sesiones en el historial", async () => {
    vi.mocked(historyApi.listHistory).mockResolvedValueOnce({
      items: [],
      totalCount: 0,
      hasMore: false,
    });

    render(HistoryScreen);

    await waitFor(() => {
      expect(screen.getByText("No hay pruebas registradas todavía")).toBeTruthy();
    });
  });

  it("renderiza la lista de sesiones agrupadas con nombres y velocidades", async () => {
    vi.mocked(historyApi.listHistory).mockResolvedValueOnce({
      items: sampleItems,
      totalCount: 2,
      hasMore: false,
    });

    render(HistoryScreen);

    await waitFor(() => {
      expect(screen.getByText("Servidor-Local")).toBeTruthy();
      expect(screen.getByText("Portatil-Wifi")).toBeTruthy();
      expect(screen.getAllByText("UDP").length).toBeGreaterThanOrEqual(1);
      expect(screen.getByText("Incompleta")).toBeTruthy();
    });
  });

  it("permite abrir el detalle de una sesión al pulsar sobre ella", async () => {
    vi.mocked(historyApi.listHistory).mockResolvedValueOnce({
      items: sampleItems,
      totalCount: 2,
      hasMore: false,
    });

    vi.mocked(historyApi.getHistoryDetail).mockResolvedValueOnce({
      id: "11111111-1111-4111-8111-111111111111",
      createdAt: "2026-09-22T06:00:00Z",
      protocol: "tcp",
      status: "completed",
      streams: 1,
      durationSeconds: 10,
      forwardBps: "9420000000",
      reverseBps: "9310000000",
      isPartial: false,
      clientInterface: "Ethernet 10G",
      serverInterface: "Ethernet 10G",
      samples: [],
    });

    vi.mocked(historyApi.compareCohort).mockResolvedValueOnce({
      sampleCount: 5,
      averageForwardBps: 9300000000,
      currentForwardBps: 9420000000,
      differencePercent: 1.2,
      isSignificantlyLower: false,
      isSignificantlyHigher: false,
      observationText: null,
    });

    render(HistoryScreen);

    await waitFor(() => {
      expect(screen.getByText("Servidor-Local")).toBeTruthy();
    });

    const itemBtn = screen.getByText("Servidor-Local");
    await fireEvent.click(itemBtn);

    await waitFor(() => {
      expect(screen.getByText(/Volver al historial/i)).toBeTruthy();
      expect(screen.getByText("Repetir esta prueba")).toBeTruthy();
    });
  });

  it("permite solicitar el borrado de todo el historial y muestra diálogo de confirmación", async () => {
    vi.mocked(historyApi.listHistory).mockResolvedValueOnce({
      items: sampleItems,
      totalCount: 2,
      hasMore: false,
    });

    vi.mocked(historyApi.deletePreview).mockResolvedValueOnce({
      target: { type: "all" },
      affectedSessions: 2,
      affectedSamples: 40,
      confirmationToken: "valid-tok-123",
      expiresInSeconds: 60,
    });

    render(HistoryScreen);

    await waitFor(() => {
      expect(screen.getByText("Eliminar todo el historial")).toBeTruthy();
    });

    const deleteAllBtn = screen.getByText("Eliminar todo el historial");
    await fireEvent.click(deleteAllBtn);

    await waitFor(() => {
      expect(screen.getByText("¿Eliminar prueba del historial?")).toBeTruthy();
      expect(screen.getByText("40")).toBeTruthy();
    });
  });
});
