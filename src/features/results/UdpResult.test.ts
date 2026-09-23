import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import UdpResult from "./UdpResult.svelte";
import type { UdpDiagnosticResult } from "../../lib/contracts/plan";

describe("T099 - UdpResult UI Component Tests (US5)", () => {
  const sampleLowLossResult: UdpDiagnosticResult = {
    targetRateBps: 100_000_000,
    emittedRateBps: 100_000_000,
    receivedRateBps: 99_950_000,
    packetsSent: 100_000,
    packetsReceived: 99_950,
    packetsLost: 50,
    lossRatio: 0.0005,
    lossPercent: 0.05,
    lossLevel: "low",
    titleKey: "diagnostics.udp.loss_low",
    observations: [],
    isTargetExceeded: false,
  };

  it("renderiza el resultado UDP con pérdidas bajas de forma óptima", () => {
    render(UdpResult, { result: sampleLowLossResult });

    expect(screen.getByText("Resultado de prueba UDP")).toBeDefined();
    expect(screen.getByText("Sin pérdidas apreciables (< 0,1 %)")).toBeDefined();
    expect(screen.getByText("0.05 %")).toBeDefined();
    expect(screen.getAllByText("100.0 Mbit/s").length).toBeGreaterThan(0);
    expect(
      screen.getByText(
        "En pruebas UDP y simultáneas no aplica el análisis de asimetría secuencial.",
      ),
    ).toBeDefined();
  });

  it("muestra advertencia cuando la tasa objetivo superaba la capacidad", () => {
    const exceededResult: UdpDiagnosticResult = {
      ...sampleLowLossResult,
      lossLevel: "moderate",
      lossPercent: 0.5,
      isTargetExceeded: true,
      observations: ["diagnostics.udp.target_exceeded_capacity"],
    };

    render(UdpResult, { result: exceededResult });

    expect(screen.getByText("Pérdidas ligeras (0,1 % – 1 %)")).toBeDefined();
    expect(screen.getByText("La tasa objetivo superaba la capacidad del enlace")).toBeDefined();
  });
});
