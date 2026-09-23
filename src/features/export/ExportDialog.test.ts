import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import ExportDialog from "./ExportDialog.svelte";
import * as exportApi from "../../lib/api/export";

vi.mock("../../lib/api/export", () => ({
  previewExport: vi.fn().mockResolvedValue({
    token: "mock-export-token-1234",
    fileNames: ["NetworkBench_LAPTOP_2026-09-22_0800.pdf"],
    disclosureKeys: [
      "export.disclosure.equipment_names",
      "export.disclosure.ip_addresses",
      "export.disclosure.mac_adapters",
      "export.disclosure.test_results",
    ],
    totalSessions: 1,
  }),
  executeExport: vi.fn().mockResolvedValue({
    exportedFiles: ["C:\\Users\\Mock\\Downloads\\NetworkBench_LAPTOP_2026-09-22_0800.pdf"],
    bytesWritten: 60400,
  }),
}));

describe("T105 - ExportDialog UI Tests (US6)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renderiza el diálogo con opciones de formato y aviso de contenido", async () => {
    render(ExportDialog, {
      open: true,
      sessionIds: ["4a72d3f0-8c9e-4b2a-a1b2-c3d4e5f6a7b8"],
      peerName: "LAPTOP",
      onclose: vi.fn(),
    });

    expect(screen.getByText("Exportar resultados")).toBeDefined();
    expect(screen.getByText("PDF")).toBeDefined();
    expect(screen.getByText("JSON")).toBeDefined();
    expect(screen.getByText("CSV")).toBeDefined();
    expect(screen.getByLabelText("Ocultar direcciones IP y MAC")).toBeDefined();

    // Comprobar llamada inicial de preview
    await waitFor(() => {
      expect(exportApi.previewExport).toHaveBeenCalledWith({
        sessionIds: ["4a72d3f0-8c9e-4b2a-a1b2-c3d4e5f6a7b8"],
        format: "pdf",
        anonymize: false,
      });
    });
  });

  it("actualiza la previsualización al cambiar la casilla de anonimización", async () => {
    render(ExportDialog, {
      open: true,
      sessionIds: ["4a72d3f0-8c9e-4b2a-a1b2-c3d4e5f6a7b8"],
      peerName: "LAPTOP",
      onclose: vi.fn(),
    });

    const checkbox = screen.getByLabelText("Ocultar direcciones IP y MAC");
    await fireEvent.click(checkbox);

    await waitFor(() => {
      expect(exportApi.previewExport).toHaveBeenCalledWith({
        sessionIds: ["4a72d3f0-8c9e-4b2a-a1b2-c3d4e5f6a7b8"],
        format: "pdf",
        anonymize: true,
      });
    });
  });

  it("cambia el formato seleccionado y actualiza preview", async () => {
    render(ExportDialog, {
      open: true,
      sessionIds: ["4a72d3f0-8c9e-4b2a-a1b2-c3d4e5f6a7b8"],
      peerName: "LAPTOP",
      onclose: vi.fn(),
    });

    const csvButton = screen.getByText("CSV");
    await fireEvent.click(csvButton);

    await waitFor(() => {
      expect(exportApi.previewExport).toHaveBeenCalledWith({
        sessionIds: ["4a72d3f0-8c9e-4b2a-a1b2-c3d4e5f6a7b8"],
        format: "csv",
        anonymize: false,
      });
    });
  });

  it("ejecuta la exportación y notifica los ficheros guardados", async () => {
    const handleExported = vi.fn();
    render(ExportDialog, {
      open: true,
      sessionIds: ["4a72d3f0-8c9e-4b2a-a1b2-c3d4e5f6a7b8"],
      peerName: "LAPTOP",
      onclose: vi.fn(),
      onexported: handleExported,
    });

    await waitFor(() => {
      expect(screen.getByText("NetworkBench_LAPTOP_2026-09-22_0800.pdf")).toBeDefined();
    });

    const exportButton = screen.getByRole("button", { name: "Exportar" });
    await fireEvent.click(exportButton);

    await waitFor(() => {
      expect(exportApi.executeExport).toHaveBeenCalled();
      expect(handleExported).toHaveBeenCalledWith([
        "C:\\Users\\Mock\\Downloads\\NetworkBench_LAPTOP_2026-09-22_0800.pdf",
      ]);
    });
  });
});
