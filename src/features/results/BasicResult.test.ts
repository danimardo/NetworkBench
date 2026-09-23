import { describe, it, expect, vi } from "vitest";
import { render, fireEvent, screen } from "@testing-library/svelte";
import BasicResult from "./BasicResult.svelte";
import type { BasicResultModel } from "./model";

describe("T049 - BasicResult View Component", () => {
  const sampleResult: BasicResultModel = {
    sessionId: "33333333-3333-4333-8333-333333333333",
    peer: {
      instanceId: "11111111-1111-4111-8111-111111111111",
      displayName: "Sobremesa-Laboratorio",
      fingerprint: "1111111111111111111111111111111111111111111111111111111111111111",
      addresses: ["192.168.1.10:7411"],
      trustState: "trusted",
      autoAccept: false,
      lastSeen: "2026-09-21T20:00:00Z",
    },
    plan: {
      protocol: "tcp",
      direction: "forward",
      streams: 1,
      warmupSeconds: 1,
      measureSeconds: 10,
      cooldownSeconds: 1,
      port: 7412,
    },
    forwardOfficialBps: "948000000",
    reverseOfficialBps: "942000000",
    durationSeconds: 10,
    completedAt: "2026-09-21 21:30:00",
  };

  it("renderiza correctamente las velocidades oficiales de ambos sentidos y no menciona NTTTCP en títulos", () => {
    render(BasicResult, {
      result: sampleResult,
    });

    expect(screen.getByText("Resultado de la medición")).toBeTruthy();
    expect(
      screen.getByText("Sobremesa-Laboratorio (192.168.1.10:7411).", { exact: false }),
    ).toBeTruthy();

    const forward = screen.getByTestId("forward-throughput");
    expect(forward.textContent).toContain("948.0 Mbit/s");

    const reverse = screen.getByTestId("reverse-throughput");
    expect(reverse.textContent).toContain("942.0 Mbit/s");

    // Verificar que el título principal no contiene NTTTCP
    expect(document.body.textContent).not.toContain("ntttcp");
  });

  it("permite invocar los callbacks de repetir y volver a equipos", async () => {
    const onRepeat = vi.fn();
    const onBackToPeers = vi.fn();

    render(BasicResult, {
      result: sampleResult,
      onRepeat,
      onBackToPeers,
    });

    const repeatBtn = screen.getByTestId("result-repeat-btn");
    const backBtn = screen.getByTestId("result-back-btn");

    await fireEvent.click(repeatBtn);
    expect(onRepeat).toHaveBeenCalled();

    await fireEvent.click(backBtn);
    expect(onBackToPeers).toHaveBeenCalled();
  });
});
