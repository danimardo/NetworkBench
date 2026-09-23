import { describe, it, expect } from "vitest";
import {
  peerSchema,
  trustStateSchema,
  sha256FingerprintSchema,
  peerDisplayNameSchema,
  type Peer,
  type TrustState,
} from "../../src/lib/contracts/peer";
import { benchmarkPlanSchema, type BenchmarkPlan } from "../../src/lib/contracts/plan";
import {
  protocolEnvelopeSchema,
  helloPayloadSchema,
  pairRequestPayloadSchema,
  pairResultPayloadSchema,
  requestPayloadSchema,
  responsePayloadSchema,
  cancelPayloadSchema,
  heartbeatPayloadSchema,
  MAX_FRAME_SIZE_BYTES,
} from "../../src/lib/contracts/protocol";

describe("T029 - Peer Contract & Protocol Tests", () => {
  describe("TrustState enum", () => {
    it("reconoce exactamente los cuatro estados de confianza requeridos", () => {
      const validStates: TrustState[] = ["unknown", "known", "trusted", "trustedAutoAccept"];

      for (const st of validStates) {
        expect(trustStateSchema.parse(st)).toBe(st);
      }

      // Estados inválidos o mal escritos
      expect(() => trustStateSchema.parse("untrusted")).toThrow();
      expect(() => trustStateSchema.parse("autoAccept")).toThrow();
      expect(() => trustStateSchema.parse("")).toThrow();
    });
  });

  describe("Peer validation", () => {
    const validPeer: Peer = {
      instanceId: "a1b2c3d4-e5f6-4a1b-8c2d-3e4f5a6b7c8d",
      displayName: "Sobremesa-Laboratorio",
      fingerprint: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      addresses: ["192.168.1.50:7411"],
      trustState: "unknown",
      autoAccept: false,
      lastSeen: "2026-09-21T20:00:00Z",
      alias: "PC Principal",
    };

    it("valida un peer correcto y normaliza fingerprint a minúsculas", () => {
      const parsed = peerSchema.parse({
        ...validPeer,
        fingerprint: "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
      });
      expect(parsed.fingerprint).toBe(validPeer.fingerprint);
      expect(parsed.trustState).toBe("unknown");
    });

    it("rechaza nombres vacíos, > 48 caracteres o con caracteres de control/bidi", () => {
      expect(() => peerDisplayNameSchema.parse("")).toThrow();
      expect(() => peerDisplayNameSchema.parse("a".repeat(49))).toThrow();
      expect(() => peerDisplayNameSchema.parse("Equipo\u0000Invalido")).toThrow();
      expect(() => peerDisplayNameSchema.parse("Equipo\u202EInvalido")).toThrow(); // RLO bidi
    });

    it("rechaza fingerprints con longitud distinta de 64 o caracteres no hex", () => {
      expect(() => sha256FingerprintSchema.parse("abcdef")).toThrow();
      expect(() => sha256FingerprintSchema.parse("g".repeat(64))).toThrow();
    });
  });

  describe("BenchmarkPlan validation", () => {
    it("acepta planes TCP estándar válidos", () => {
      const validPlan: BenchmarkPlan = {
        protocol: "tcp",
        direction: "forward",
        streams: 1,
        warmupSeconds: 1,
        measureSeconds: 10,
        cooldownSeconds: 1,
        port: 7412,
        bufferSizeBytes: "65536",
      };
      const parsed = benchmarkPlanSchema.parse(validPlan);
      expect(parsed.protocol).toBe("tcp");
      expect(parsed.streams).toBe(1);
    });

    it("rechaza streams fuera del rango 1..64 en secuencial", () => {
      expect(() =>
        benchmarkPlanSchema.parse({
          protocol: "tcp",
          direction: "forward",
          streams: 0,
          warmupSeconds: 1,
          measureSeconds: 10,
          cooldownSeconds: 1,
          port: 7412,
        }),
      ).toThrow();

      expect(() =>
        benchmarkPlanSchema.parse({
          protocol: "tcp",
          direction: "forward",
          streams: 65,
          warmupSeconds: 1,
          measureSeconds: 10,
          cooldownSeconds: 1,
          port: 7412,
        }),
      ).toThrow();
    });

    it("rechaza streams > 32 en simultáneo ('both')", () => {
      expect(() =>
        benchmarkPlanSchema.parse({
          protocol: "tcp",
          direction: "both",
          streams: 33,
          warmupSeconds: 1,
          measureSeconds: 10,
          cooldownSeconds: 1,
          port: 7412,
        }),
      ).toThrow();

      expect(
        benchmarkPlanSchema.parse({
          protocol: "tcp",
          direction: "both",
          streams: 32,
          warmupSeconds: 1,
          measureSeconds: 10,
          cooldownSeconds: 1,
          port: 7412,
        }),
      ).toBeDefined();
    });

    it("rechaza puertos reservados < 1024 o > 65535", () => {
      expect(() =>
        benchmarkPlanSchema.parse({
          protocol: "tcp",
          direction: "forward",
          streams: 1,
          warmupSeconds: 1,
          measureSeconds: 10,
          cooldownSeconds: 1,
          port: 80,
        }),
      ).toThrow();
    });

    it("rechaza duraciones de medición < 5 s o > 300 s", () => {
      expect(() =>
        benchmarkPlanSchema.parse({
          protocol: "tcp",
          direction: "forward",
          streams: 1,
          warmupSeconds: 1,
          measureSeconds: 4,
          cooldownSeconds: 1,
          port: 7412,
        }),
      ).toThrow();

      expect(() =>
        benchmarkPlanSchema.parse({
          protocol: "tcp",
          direction: "forward",
          streams: 1,
          warmupSeconds: 1,
          measureSeconds: 301,
          cooldownSeconds: 1,
          port: 7412,
        }),
      ).toThrow();
    });
  });

  describe("Protocol Envelope and Payloads", () => {
    it("valida el envelope estándar con id UUID y ts ISO 8601", () => {
      const envelope = {
        type: "HELLO",
        id: "b2c3d4e5-f6a1-4b2c-8d3e-4f5a6b7c8d9e",
        sessionId: null,
        ts: "2026-09-21T20:00:00Z",
        payload: {
          protocolVersion: 1,
          protocolMin: 1,
          appVersion: "0.1.0",
          instanceId: "a1b2c3d4-e5f6-4a1b-8c2d-3e4f5a6b7c8d",
          displayName: "Portatil-Dani",
          platform: "windows",
          isBusy: false,
        },
      };

      const parsed = protocolEnvelopeSchema.parse(envelope);
      expect(parsed.type).toBe("HELLO");

      const hello = helloPayloadSchema.parse(parsed.payload);
      expect(hello.protocolVersion).toBe(1);
      expect(hello.displayName).toBe("Portatil-Dani");
    });

    it("valida el código de emparejamiento de exactamente 6 dígitos en PAIR_REQUEST", () => {
      expect(pairRequestPayloadSchema.parse({ pairingCode: "123456" })).toEqual({
        pairingCode: "123456",
      });

      expect(() => pairRequestPayloadSchema.parse({ pairingCode: "12345" })).toThrow();
      expect(() => pairRequestPayloadSchema.parse({ pairingCode: "1234567" })).toThrow();
      expect(() => pairRequestPayloadSchema.parse({ pairingCode: "abcdef" })).toThrow();
    });

    it("valida respuestas de PAIR_RESULT y REQUEST/RESPONSE", () => {
      const pairRes = pairResultPayloadSchema.parse({
        accepted: true,
      });
      expect(pairRes.accepted).toBe(true);

      const reqPayload = requestPayloadSchema.parse({
        plan: {
          protocol: "tcp",
          direction: "forward",
          streams: 4,
          warmupSeconds: 1,
          measureSeconds: 10,
          cooldownSeconds: 1,
          port: 7412,
        },
        estimateSeconds: 12,
        suggestedInterface: "Ethernet 1",
      });
      expect(reqPayload.plan.streams).toBe(4);

      const respPayload = responsePayloadSchema.parse({
        accepted: false,
        reason: "Usuario rechazó la prueba",
      });
      expect(respPayload.accepted).toBe(false);
      expect(respPayload.reason).toBe("Usuario rechazó la prueba");
    });

    it("verifica el límite máximo de trama de 1 MiB", () => {
      expect(MAX_FRAME_SIZE_BYTES).toBe(1048576);
    });

    it("valida payloads de CANCEL y HEARTBEAT", () => {
      const cancel = cancelPayloadSchema.parse({
        reason: "Cancelado por el usuario",
        code: "NB-CONN-004",
      });
      expect(cancel.code).toBe("NB-CONN-004");

      const hb = heartbeatPayloadSchema.parse({ nonce: 42 });
      expect(hb.nonce).toBe(42);
    });
  });
});
