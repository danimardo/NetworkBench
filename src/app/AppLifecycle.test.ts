import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import CloseDialog from "./CloseDialog.svelte";

describe("AppLifecycle and CloseDialog", () => {
  it("renderiza el diálogo modal cuando open es true", () => {
    render(CloseDialog, {
      props: {
        open: true,
        oncancel: vi.fn(),
        onconfirm: vi.fn(),
      },
    });

    const dialog = screen.getByRole("dialog");
    expect(dialog).toBeTruthy();
  });

  it("invoca oncancel al hacer clic en cancelar", async () => {
    const oncancel = vi.fn();
    const onconfirm = vi.fn();

    render(CloseDialog, {
      props: {
        open: true,
        oncancel,
        onconfirm,
      },
    });

    const buttons = screen.getAllByRole("button");
    const cancelButton = buttons.find(
      (b) =>
        b.textContent?.includes("cancel") ||
        b.textContent?.includes("Seguir") ||
        b.textContent?.includes("Cancelar"),
    );
    expect(cancelButton).toBeTruthy();

    if (cancelButton) {
      await fireEvent.click(cancelButton);
      expect(oncancel).toHaveBeenCalledTimes(1);
      expect(onconfirm).not.toHaveBeenCalled();
    }
  });

  it("invoca onconfirm al hacer clic en salir y cancelar prueba", async () => {
    const oncancel = vi.fn();
    const onconfirm = vi.fn();

    render(CloseDialog, {
      props: {
        open: true,
        oncancel,
        onconfirm,
      },
    });

    const buttons = screen.getAllByRole("button");
    const confirmButton = buttons.find(
      (b) =>
        b.textContent?.includes("Salir") ||
        b.textContent?.includes("Exit") ||
        b.textContent?.includes("confirm"),
    );
    expect(confirmButton).toBeTruthy();

    if (confirmButton) {
      await fireEvent.click(confirmButton);
      expect(onconfirm).toHaveBeenCalledTimes(1);
      expect(oncancel).not.toHaveBeenCalled();
    }
  });
});
