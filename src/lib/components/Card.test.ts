import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import Card from "./Card.svelte";
import { createRawSnippet } from "svelte";

// `enterIndex` es lo único nuevo aquí (la entrada escalonada, `.nb-enter` en tokens.css):
// sin él, una tarjeta no debe llevar ni la clase ni la variable --nb-i, para no animar nada
// donde no se ha pedido (p. ej. un diálogo o una tarjeta suelta fuera de una lista).

const contenido = createRawSnippet(() => ({
  render: () => `<span>contenido</span>`,
}));

describe("Card: entrada escalonada (enterIndex)", () => {
  it("sin enterIndex, no lleva la clase ni la variable de la animación", () => {
    render(Card, { props: { children: contenido } });
    const tarjeta = screen.getByText("contenido").closest(".nb-card")!;
    expect(tarjeta.classList.contains("nb-enter")).toBe(false);
    expect(tarjeta.getAttribute("style") ?? "").not.toContain("--nb-i");
  });

  it("con enterIndex, lleva la clase y --nb-i con ese valor exacto", () => {
    render(Card, { props: { enterIndex: 3, children: contenido } });
    const tarjeta = screen.getByText("contenido").closest(".nb-card")!;
    expect(tarjeta.classList.contains("nb-enter")).toBe(true);
    expect((tarjeta as HTMLElement).style.getPropertyValue("--nb-i")).toBe("3");
  });

  it('con enterIndex={0} también anima (0 es un índice válido, no "sin índice")', () => {
    render(Card, { props: { enterIndex: 0, children: contenido } });
    const tarjeta = screen.getByText("contenido").closest(".nb-card")!;
    expect(tarjeta.classList.contains("nb-enter")).toBe(true);
    expect((tarjeta as HTMLElement).style.getPropertyValue("--nb-i")).toBe("0");
  });

  it("una tarjeta clicable (con onclick) también lleva la clase y la variable", () => {
    render(Card, { props: { enterIndex: 1, onclick: () => {}, children: contenido } });
    const tarjeta = screen.getByRole("button");
    expect(tarjeta.classList.contains("nb-enter")).toBe(true);
    expect((tarjeta as HTMLElement).style.getPropertyValue("--nb-i")).toBe("1");
  });
});
