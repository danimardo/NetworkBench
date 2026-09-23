import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  instanceIdSchema,
  decimalU64Schema,
  createMetricValueSchema,
  throughputBpsSchema,
} from "../../src/lib/contracts/common";

const fixturesPath = resolve("tests/contracts/fixtures/common-fixtures.json");
const fixtures = JSON.parse(readFileSync(fixturesPath, "utf-8"));

describe("Contratos comunes: Esquemas Zod y Fixtures", () => {
  it("valida UUIDv4 canónicos", () => {
    for (const validId of fixtures.valid_instances) {
      const parsed = instanceIdSchema.safeParse(validId);
      expect(parsed.success).toBe(true);
    }
  });

  it("rechaza UUIDs inválidos o no v4", () => {
    for (const invalidId of fixtures.invalid_instances) {
      const parsed = instanceIdSchema.safeParse(invalidId);
      expect(parsed.success).toBe(false);
    }
  });

  it("valida y preserva precisión en decimalU64 sin desbordamiento de Number", () => {
    for (const validU64 of fixtures.valid_u64_decimals) {
      const parsed = decimalU64Schema.safeParse(validU64);
      expect(parsed.success).toBe(true);
      if (parsed.success) {
        expect(parsed.data).toBe(validU64);
      }
    }
  });

  it("rechaza decimalU64 con formatos inválidos o negativos", () => {
    for (const invalidU64 of fixtures.invalid_u64_decimals) {
      const parsed = decimalU64Schema.safeParse(invalidU64);
      expect(parsed.success).toBe(false);
    }
  });

  it("valida la unión discriminada MetricValue en sus cuatro variantes", () => {
    const bpsMetricSchema = createMetricValueSchema(throughputBpsSchema);

    const av = bpsMetricSchema.safeParse(fixtures.metric_values.available);
    expect(av.success).toBe(true);
    if (av.success) {
      expect(av.data.status).toBe("available");
      if (av.data.status === "available") {
        expect(av.data.value).toBe("10000000000");
      }
    }

    const na = bpsMetricSchema.safeParse(fixtures.metric_values.not_available);
    expect(na.success).toBe(true);
    if (na.success) {
      expect(na.data.status).toBe("notAvailable");
    }

    const ne = bpsMetricSchema.safeParse(fixtures.metric_values.not_evaluable);
    expect(ne.success).toBe(true);
    if (ne.success) {
      expect(ne.data.status).toBe("notEvaluable");
      if (ne.data.status === "notEvaluable") {
        expect(ne.data.reason).toBeDefined();
      }
    }

    const inv = bpsMetricSchema.safeParse(fixtures.metric_values.invalid);
    expect(inv.success).toBe(true);
    if (inv.success) {
      expect(inv.data.status).toBe("invalid");
      if (inv.data.status === "invalid") {
        expect(inv.data.reason).toBeDefined();
      }
    }
  });
});
