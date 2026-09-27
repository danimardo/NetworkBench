import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import { createRawSnippet } from "svelte";
import Toast from "./Toast.svelte";

const texto = createRawSnippet(() => ({ render: () => `<span>aviso</span>` }));

describe("Toast: variante ancha (wide)", () => {
  it("sin wide, no lleva la clase de ancho extra", () => {
    render(Toast, { props: { children: texto } });
    const toast = screen.getByText("aviso").closest(".nb-toast")!;
    expect(toast.classList.contains("nb-toast-wide")).toBe(false);
  });

  it("con wide, lleva la clase que amplía el ancho máximo", () => {
    render(Toast, { props: { wide: true, children: texto } });
    const toast = screen.getByText("aviso").closest(".nb-toast")!;
    expect(toast.classList.contains("nb-toast-wide")).toBe(true);
  });
});
