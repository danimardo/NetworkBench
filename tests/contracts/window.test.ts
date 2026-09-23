import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  windowGeometrySchema,
  getWindowGeometry,
  saveWindowGeometry,
  restoreAndShowWindow,
} from "../../src/lib/window";
import { setTransportMock } from "../../src/lib/api/transport";

describe("Window Geometry Contract Tests (T121)", () => {
  beforeEach(() => {
    setTransportMock(null);
    vi.restoreAllMocks();
  });

  it("valida una geometría de ventana conforme con mínimos 800x600", () => {
    const valid = {
      x: 100,
      y: 100,
      width: 1024,
      height: 720,
      isMaximized: false,
    };
    expect(windowGeometrySchema.parse(valid)).toEqual(valid);

    // Dimensiones menores a 800x600 deben ser rechazadas
    expect(() =>
      windowGeometrySchema.parse({
        ...valid,
        width: 799,
      }),
    ).toThrow();

    expect(() =>
      windowGeometrySchema.parse({
        ...valid,
        height: 599,
      }),
    ).toThrow();
  });

  it("getWindowGeometry devuelve la geometría o null", async () => {
    setTransportMock(async (cmd) => {
      if (cmd === "window_get_geometry") {
        return {
          ok: true,
          value: {
            x: 200,
            y: 150,
            width: 1200,
            height: 800,
            isMaximized: false,
          },
        };
      }
      return { ok: true, value: null };
    });

    const geom = await getWindowGeometry();
    expect(geom?.width).toBe(1200);
    expect(geom?.height).toBe(800);
    expect(geom?.x).toBe(200);
  });

  it("saveWindowGeometry persiste la geometría", async () => {
    let savedGeom: unknown = null;
    setTransportMock(async (cmd, args) => {
      if (cmd === "window_save_geometry") {
        savedGeom = args?.request;
        return { ok: true, value: null };
      }
      return { ok: true, value: null };
    });

    await saveWindowGeometry({
      x: 50,
      y: 50,
      width: 1024,
      height: 768,
      isMaximized: false,
    });

    expect(savedGeom).toEqual({
      x: 50,
      y: 50,
      width: 1024,
      height: 768,
      isMaximized: false,
    });
  });

  it("restoreAndShowWindow devuelve la geometría aplicada", async () => {
    setTransportMock(async (cmd) => {
      if (cmd === "window_restore_and_show") {
        return {
          ok: true,
          value: {
            x: 100,
            y: 100,
            width: 1024,
            height: 720,
            isMaximized: false,
          },
        };
      }
      return { ok: true, value: null };
    });

    const applied = await restoreAndShowWindow();
    expect(applied?.width).toBe(1024);
    expect(applied?.height).toBe(720);
  });
});
