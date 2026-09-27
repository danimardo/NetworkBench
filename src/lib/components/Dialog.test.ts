import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import { createRawSnippet } from "svelte";
import Dialog from "./Dialog.svelte";

const cuerpo = createRawSnippet(() => ({ render: () => `<p>cuerpo</p>` }));

describe("Dialog: usa la utilidad compartida .nb-pop, no una animación propia", () => {
  it("el panel lleva la clase nb-pop", () => {
    render(Dialog, { props: { title: "Título", children: cuerpo } });
    const panel = screen.getByRole("dialog");
    expect(panel.classList.contains("nb-pop")).toBe(true);
  });
});
