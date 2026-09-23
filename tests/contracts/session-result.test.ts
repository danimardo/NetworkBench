import { describe, it, expect } from "vitest";
import {
  sessionResultSchema,
  directionResultSchema,
  type SessionResult,
} from "../../src/lib/contracts/session-result";

describe("SessionResult Contract Tests (T052)", () => {
  it("valida un SessionResult completo con veredicto OK", () => {
    const rawResult: SessionResult = {
      schemaVersion: 1,
      sessionId: "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
      startedAt: "2026-09-22T05:00:00.000Z",
      finishedAt: "2026-09-22T05:01:00.000Z",
      status: "completed",
      initiator: {
        instanceId: "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
        displayName: "DESKTOP-LOCAL",
        fingerprint: "sha256:11223344556677889900aabbccddeeff11223344556677889900aabbccddeeff",
        address: "192.168.1.50",
      },
      responder: {
        instanceId: "7c9e6679-7425-40de-944b-e07fc1f90ae7",
        displayName: "SERVER-REMOTE",
        fingerprint: "sha256:ffeeddccbbaa00998877665544332211ffeeddccbbaa00998877665544332211",
        address: "192.168.1.60",
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
          sender: {
            role: "sender",
            totalBytes: "2375000000",
            realtimeSeconds: 20.0,
            throughputBps: "950000000",
            cpuPercent: 12.5,
            buffersCount: 36240,
            errorsCount: 0,
          },
          receiver: {
            role: "receiver",
            totalBytes: "2370000000",
            realtimeSeconds: 20.0,
            throughputBps: "948000000",
            cpuPercent: 8.2,
            buffersCount: 36160,
            errorsCount: 0,
          },
          officialBps: "948000000",
          utilization: 0.948,
          stability: {
            level: "veryStable",
            meanBps: 948000000,
            stdDevBps: 9480000,
            cv: 0.01,
            minBps: 940000000,
            maxBps: 955000000,
            p5Bps: 942000000,
            p50Bps: 948000000,
            p95Bps: 954000000,
            dropsCount: 0,
            samplesCount: 40,
            gapsCount: 0,
          },
          retransmission: {
            level: "normal",
            ratio: 0.0001,
            packetsSent: 160000,
            packetsRetransmitted: 16,
          },
          cpuSender: 12.5,
          cpuReceiver: 8.2,
          samplesCount: 40,
          gapsCount: 0,
        },
      ],
      asymmetry: {
        ratio: 0.02,
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
            safeParams: { speedBps: "948000000" },
            evidenceRefs: ["officialBps"],
          },
        ],
        observations: [],
        possibleCauses: [],
        actions: [],
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

    // parse should succeed
    const parsed = sessionResultSchema.parse(rawResult);
    expect(parsed.status).toBe("completed");
    expect(parsed.directions[0]?.officialBps).toBe("948000000");
    expect(parsed.verdict?.level).toBe("ok");
  });

  it("valida un SessionResult incompleto con veredicto notEvaluable", () => {
    const rawResult: SessionResult = {
      schemaVersion: 1,
      sessionId: "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
      startedAt: "2026-09-22T05:00:00.000Z",
      finishedAt: "2026-09-22T05:00:15.000Z",
      status: "incomplete",
      initiator: {
        instanceId: "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
        displayName: "DESKTOP-LOCAL",
        fingerprint: "sha256:11223344556677889900aabbccddeeff11223344556677889900aabbccddeeff",
        address: "192.168.1.50",
      },
      responder: {
        instanceId: "7c9e6679-7425-40de-944b-e07fc1f90ae7",
        displayName: "SERVER-REMOTE",
        fingerprint: "sha256:ffeeddccbbaa00998877665544332211ffeeddccbbaa00998877665544332211",
        address: "192.168.1.60",
      },
      plan: {
        protocol: "tcp",
        direction: "forward",
        measureSeconds: 20,
        warmupSeconds: 2,
        cooldownSeconds: 2,
        streams: 4,
        port: 5001,
      },
      capacity: null,
      directions: [
        {
          direction: "forward",
          status: "incomplete",
          samplesCount: 0,
          gapsCount: 0,
        },
      ],
      asymmetry: null,
      verdict: {
        rulesVersion: "1.0.0",
        level: "notEvaluable",
        titleKey: "verdict.incomplete",
        performanceLevel: null,
        stabilityLevel: "notEvaluable",
        asymmetryLevel: null,
        retransmissionLevel: "notAvailable",
        cpuLevel: "notAvailable",
        facts: [],
        observations: [],
        possibleCauses: [],
        actions: [
          {
            ruleId: "SESSION_REPEAT",
            messageKey: "actions.repeat_test",
            safeParams: null,
            evidenceRefs: [],
          },
        ],
      },
      resultSource: "local",
      versions: {
        appVersion: "0.1.0",
        protocolVersion: 1,
        engineVersion: "5.40",
        schemaVersion: 1,
        thresholdsHash: "b65f73615249f16c22fa3d63685f792b7f94661eed4f7e0eea454d1dba8923c0",
      },
    };

    const parsed = sessionResultSchema.parse(rawResult);
    expect(parsed.status).toBe("incomplete");
    expect(parsed.directions[0]?.officialBps).toBeUndefined();
    expect(parsed.verdict?.level).toBe("notEvaluable");
  });

  it("rechaza direcciones con throughput official que no sea string u64 seguro", () => {
    const invalidDirection = {
      direction: "forward",
      status: "completed",
      officialBps: "-100", // negativo inválido
      samplesCount: 10,
      gapsCount: 0,
    };
    expect(() => directionResultSchema.parse(invalidDirection)).toThrow();
  });
});
