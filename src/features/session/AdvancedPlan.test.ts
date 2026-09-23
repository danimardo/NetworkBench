import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import AdvancedPlan from "./AdvancedPlan.svelte";
import type { BenchmarkPlan } from "../../lib/contracts/plan";

describe("T092 - AdvancedPlan UI Component Tests (US5)", () => {
  const defaultPlan: BenchmarkPlan = {
    protocol: "tcp",
    direction: "both_sequential",
    streams: 1,
    warmupSeconds: 2,
    measureSeconds: 20,
    cooldownSeconds: 2,
    port: 5001,
    bufferSizeBytes: "65536",
  };

  it("renderiza correctamente las opciones avanzadas y la vista previa del plan", () => {
    render(AdvancedPlan, { plan: defaultPlan });

    expect(screen.getByText("Opciones avanzadas")).toBeDefined();
    expect(screen.getByText("Ambas (secuencial)")).toBeDefined();
    expect(screen.getByText("PLAN VÁLIDO")).toBeDefined();
    expect(screen.getByText(/5001\.\.5001 \(1 puerto\)/)).toBeDefined();
  });

  it("muestra el aviso explicativo de UDP al seleccionar el protocolo UDP", async () => {
    render(AdvancedPlan, { plan: defaultPlan });

    const udpRadio = screen.getByRole("radio", { name: "UDP" });
    await fireEvent.click(udpRadio);

    // Debe mostrar la nota explicativa de dominio de UDP (§18)
    expect(screen.getByText(/TCP y UDP responden a preguntas distintas/i)).toBeDefined();

    // Debe mostrar campos de tasa y tamaño de datagrama
    expect(screen.getByText(/Tasa objetivo UDP/i)).toBeDefined();
    expect(screen.getByText(/Tamaño de datagrama UDP/i)).toBeDefined();
  });

  it("ajusta y valida límites en modo simultáneo (máximo 32 streams por sentido)", async () => {
    render(AdvancedPlan, {
      plan: {
        ...defaultPlan,
        streams: 40,
      },
    });

    // Cambiar a "Ambas a la vez" (simultáneo)
    const simBtn = screen.getByRole("radio", { name: "Ambas a la vez (simultáneo)" });
    await fireEvent.click(simBtn);

    // Debe autoajustar streams a 32 máximo y reflejarlo en la vista previa
    expect(screen.getByText(/5001\.\.5032 \(envío\) y 5033\.\.5064 \(recepción\)/)).toBeDefined();
    expect(screen.getByText("PLAN VÁLIDO")).toBeDefined();
  });

  it("restaura los valores recomendados al pulsar el botón correspondiente", async () => {
    render(AdvancedPlan, {
      plan: {
        protocol: "udp",
        direction: "forward",
        streams: 16,
        warmupSeconds: 5,
        measureSeconds: 60,
        cooldownSeconds: 5,
        port: 6000,
      },
    });

    const restoreBtn = screen.getByText("Restaurar valores recomendados");
    await fireEvent.click(restoreBtn);

    // Al restaurar vuelve a TCP y both_sequential
    const tcpRadio = screen.getByRole("radio", { name: "TCP" });
    expect(tcpRadio.classList.contains("active")).toBe(true);
    expect(screen.getByText(/5001\.\.5001 \(1 puerto\)/)).toBeDefined();
  });

  it("invoca onApply con el plan configurado cuando es válido", async () => {
    const onApply = vi.fn();
    render(AdvancedPlan, { plan: defaultPlan, onApply });

    const applyBtn = screen.getByText("Aplicar configuración");
    await fireEvent.click(applyBtn);

    expect(onApply).toHaveBeenCalledTimes(1);
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        protocol: "tcp",
        direction: "both_sequential",
        streams: 1,
        port: 5001,
      }),
    );
  });
});
