import { describe, expect, it } from "vitest";
import { APP_NAME } from "../src/lib";

describe("Smoke test", () => {
  it("exports APP_NAME correctly", () => {
    expect(APP_NAME).toBe("NetworkBench");
  });
});
