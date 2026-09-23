import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import ResultScreen from "./ResultScreen.svelte";
import type { SessionResult } from "../../lib/contracts/session-result";

describe("ResultScreen Component Tests (T053)", () => {
  const mockResult: SessionResult = {
    schemaVersion: 1,
    sessionId: "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    startedAt: "2026-09-22T05:00:00.000Z",
    finishedAt: "2026-09-22T05:01:00.000Z",
    status: "completed",
    initiator: {
      instanceId: "3fa85f64-5717-4562-b3fc-2c963f66afa6",
      displayName: "DESKTOP-LOCAL",
      fingerprint: "sha256:11223344",
      address: "192.168.1.10",
    },
    responder: {
      instanceId: "4fa85f64-5717-4562-b3fc-2c963f66afa7",
      displayName: "SERVER-REMOTE",
      fingerprint: "sha256:55667788",
      address: "192.168.1.20",
    },
    plan: {
      protocol: "tcp",
      direction: "both",
      measureSeconds: 20,
      warmupSeconds: 2,
      cooldownSeconds: 2,
      streams: 4,
      port: 5001,
    },
    capacity: {
      refBps: 1000000000,
      refSource: "negotiated",
      capABps: 1000000000,
      capBBps: 1000000000,
    },
    directions: [
      {
        direction: "forward",
        status: "completed",
        officialBps: "950000000",
        utilization: 0.95,
        samplesCount: 40,
        gapsCount: 0,
        stability: {
          level: "veryStable",
          meanBps: 950000000,
          stdDevBps: 5000000,
          cv: 0.005,
          minBps: 940000000,
          maxBps: 960000000,
          p5Bps: 945000000,
          p50Bps: 950000000,
          p95Bps: 955000000,
          dropsCount: 0,
          samplesCount: 40,
          gapsCount: 0,
        },
        retransmission: {
          level: "normal",
          ratio: 0.0001,
          packetsSent: 50000,
          packetsRetransmitted: 5,
        },
        cpuReceiver: 12.0,
      },
      {
        direction: "reverse",
        status: "completed",
        officialBps: "940000000",
        utilization: 0.94,
        samplesCount: 40,
        gapsCount: 0,
        stability: {
          level: "veryStable",
          meanBps: 940000000,
          stdDevBps: 6000000,
          cv: 0.006,
          minBps: 930000000,
          maxBps: 950000000,
          p5Bps: 935000000,
          p50Bps: 940000000,
          p95Bps: 945000000,
          dropsCount: 0,
          samplesCount: 40,
          gapsCount: 0,
        },
        retransmission: {
          level: "normal",
          ratio: 0.0001,
          packetsSent: 50000,
          packetsRetransmitted: 4,
        },
        cpuReceiver: 14.0,
      },
    ],
    asymmetry: {
      ratio: 0.01,
      isAsymmetric: false,
    },
    verdict: {
      rulesVersion: "1.0.0",
      level: "ok",
      titleKey: "verdict.ok_title",
      performanceLevel: "ok",
      stabilityLevel: "veryStable",
      asymmetryLevel: "ok",
      retransmissionLevel: "normal",
      cpuLevel: "low",
      facts: [
        {
          ruleId: "FACT_SPEED",
          messageKey: "facts.speed_measured",
          safeParams: { speedBps: "940000000" },
          evidenceRefs: ["officialBps"],
        },
      ],
      observations: [
        {
          ruleId: "OBS_NONE",
          messageKey: "observations.stability_variable",
          safeParams: null,
          evidenceRefs: [],
        },
      ],
      possibleCauses: [
        {
          ruleId: "CAUSE_NONE",
          messageKey: "causes.background_traffic",
          safeParams: null,
          evidenceRefs: [],
        },
      ],
      actions: [
        {
          ruleId: "ACT_NONE",
          messageKey: "actions.check_cable",
          safeParams: null,
          evidenceRefs: [],
        },
      ],
    },
    resultSource: "initiator",
    versions: {
      appVersion: "0.1.0",
      protocolVersion: 1,
      engineVersion: "5.40",
      schemaVersion: 1,
      thresholdsHash: "b65f73615249f16c22fa3d63685f792b7f94661eed4f7e0eea454d1dba8923c0",
    },
  };

  it("renderiza la tarjeta de veredicto nivel 1 y la velocidad mínima oficial", () => {
    render(ResultScreen, { props: { result: mockResult } });

    // La velocidad mínima de las dos direcciones es 940 Mbit/s (940000000)
    expect(screen.getByTestId("main-speed").textContent).toContain("940.0 Mbit/s");
    expect(screen.getByTestId("verdict-card")).toBeTruthy();
  });

  it("renderiza las cuatro secciones diagnósticas estructuradas: hechos, observaciones, causas y acciones", () => {
    render(ResultScreen, { props: { result: mockResult } });

    expect(screen.getByTestId("facts-block")).toBeTruthy();
    expect(screen.getByTestId("observations-block")).toBeTruthy();
    expect(screen.getByTestId("causes-block")).toBeTruthy();
    expect(screen.getByTestId("actions-block")).toBeTruthy();
  });

  it("renderiza la gráfica ThroughputChart y las métricas de dirección", () => {
    render(ResultScreen, { props: { result: mockResult } });

    expect(screen.getByTestId("throughput-chart")).toBeTruthy();
    expect(screen.getByTestId("metric-card-directions")).toBeTruthy();
    expect(screen.getByTestId("metric-card-stability")).toBeTruthy();
  });

  it("ejecuta los callbacks onRetest y onNewTest al pulsar los botones correspondientes", async () => {
    const onRetest = vi.fn();
    const onNewTest = vi.fn();

    render(ResultScreen, { props: { result: mockResult, onRetest, onNewTest } });

    const retestBtn = screen.getByTestId("retest-btn");
    const newTestBtn = screen.getByTestId("new-test-btn");

    retestBtn.click();
    expect(onRetest).toHaveBeenCalledTimes(1);

    newTestBtn.click();
    expect(onNewTest).toHaveBeenCalledTimes(1);
  });
});
