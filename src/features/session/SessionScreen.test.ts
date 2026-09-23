import { describe, it, expect, vi } from "vitest";
import { render, fireEvent, screen } from "@testing-library/svelte";
import SessionScreen from "./SessionScreen.svelte";
import type { Peer } from "../../lib/contracts/peer";
import type { BenchmarkPlan } from "../../lib/contracts/plan";

describe("T034 - SessionScreen UI & Accessibility", () => {
  const samplePeer: Peer = {
    instanceId: "11111111-1111-4111-8111-111111111111",
    displayName: "PC-Sobremesa",
    fingerprint: "1111111111111111111111111111111111111111111111111111111111111111",
    addresses: ["192.168.1.10:7411"],
    trustState: "trusted",
    autoAccept: false,
    lastSeen: "2026-09-21T20:00:00Z",
  };

  const samplePlan: BenchmarkPlan = {
    protocol: "tcp",
    direction: "forward",
    streams: 1,
    warmupSeconds: 1,
    measureSeconds: 10,
    cooldownSeconds: 1,
    port: 7412,
  };

  it("muestra el botón Cancelar siempre visible durante la fase activa y reacciona al click", async () => {
    const onCancel = vi.fn();
    render(SessionScreen, {
      peer: samplePeer,
      plan: samplePlan,
      phase: "runningSend",
      progressPercent: 45,
      currentThroughputBps: "948000000",
      onCancel,
    });

    expect(screen.getByText("PC-Sobremesa")).toBeTruthy();
    expect(screen.getByText("Midiendo envío (A → B)")).toBeTruthy();

    const cancelBtn = screen.getByTestId("session-cancel-btn");
    expect(cancelBtn).toBeTruthy();

    await fireEvent.click(cancelBtn);
    expect(onCancel).toHaveBeenCalled();
  });

  it("muestra el throughput actual formateado en tiempo real", () => {
    render(SessionScreen, {
      peer: samplePeer,
      plan: samplePlan,
      phase: "runningSend",
      progressPercent: 50,
      currentThroughputBps: "950000000",
    });

    const metric = screen.getByTestId("live-throughput");
    expect(metric.textContent).toContain("950.0 Mbit/s");
  });

  it("muestra acciones finales al completar la prueba con éxito", async () => {
    const onViewResults = vi.fn();
    render(SessionScreen, {
      peer: samplePeer,
      plan: samplePlan,
      phase: "completed",
      progressPercent: 100,
      onViewResults,
    });

    expect(screen.getByText("Prueba completada con éxito")).toBeTruthy();
    const viewBtn = screen.getByTestId("view-results-btn");
    await fireEvent.click(viewBtn);
    expect(onViewResults).toHaveBeenCalled();
  });

  it("muestra la opción de reintentar si la prueba fue cancelada o falló", async () => {
    const onRetry = vi.fn();
    render(SessionScreen, {
      peer: samplePeer,
      plan: samplePlan,
      phase: "cancelled",
      onRetry,
    });

    expect(screen.getByText("Prueba cancelada")).toBeTruthy();
    const retryBtn = screen.getByTestId("retry-btn");
    await fireEvent.click(retryBtn);
    expect(onRetry).toHaveBeenCalled();
  });
});
