import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import ErrorResolution from "./ErrorResolution.svelte";
import type { AppError } from "../../lib/contracts/errors";

describe("ErrorResolution Component Tests (T068 / US3)", () => {
  it("muestra el título humano y el código de referencia secundario", () => {
    const error: AppError = {
      code: "NB-FW-002",
      severity: "error",
      messageKey: "errors.NB-FW-002",
      issues: [],
      actions: ["configure_firewall", "show_firewall_instructions"],
    };

    render(ErrorResolution, { props: { error } });

    // Código secundario visible
    expect(screen.getByText("Ref. NB-FW-002")).toBeTruthy();
    // Botones de acción
    expect(screen.getByText("Configurar automáticamente")).toBeTruthy();
    expect(screen.getByText("Ver instrucciones")).toBeTruthy();
  });

  it("permite alternar y copiar las instrucciones manuales de firewall", async () => {
    const error: AppError = {
      code: "NB-FW-002",
      severity: "error",
      messageKey: "errors.NB-FW-002",
      issues: [],
      actions: ["show_firewall_instructions"],
    };

    render(ErrorResolution, { props: { error } });

    const instructionsBtn = screen.getByText("Ver instrucciones");
    await fireEvent.click(instructionsBtn);

    // Debe mostrar la caja de instrucciones
    expect(screen.getByText(/PowerShell como Administrador/i)).toBeTruthy();
    expect(screen.getByText(/New-NetFirewallRule/i)).toBeTruthy();
  });

  it("invoca la acción seleccionada al pulsar el botón correspondiente", async () => {
    const onAction = vi.fn();
    const error: AppError = {
      code: "NB-PORT-001",
      severity: "error",
      messageKey: "errors.NB-PORT-001",
      issues: [],
      actions: ["change_port"],
    };

    render(ErrorResolution, { props: { error, onAction } });

    const changePortBtn = screen.getByText("Cambiar puerto");
    await fireEvent.click(changePortBtn);

    expect(onAction).toHaveBeenCalledWith("change_port");
  });

  it("muestra el estado de severidad advertencia correctamente para NB-DISK-001", () => {
    const error: AppError = {
      code: "NB-DISK-001",
      severity: "warning",
      messageKey: "errors.NB-DISK-001",
      issues: [],
      actions: ["free_space"],
    };

    render(ErrorResolution, { props: { error } });

    expect(screen.getByText("Ref. NB-DISK-001")).toBeTruthy();
    expect(screen.getByText("WARNING")).toBeTruthy();
    expect(screen.getByText("Liberar espacio en disco")).toBeTruthy();
  });
});
