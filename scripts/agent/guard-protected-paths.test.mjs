import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

// No escribe rutas protegidas: entrega solicitudes sintéticas al hook por stdin.
const guard = fileURLToPath(new URL("./guard-protected-paths.mjs", import.meta.url));
const decision = (tool_name, tool_input) => {
  const result = spawnSync(process.execPath, [guard], {
    input: JSON.stringify({ tool_name, tool_input }),
    encoding: "utf8",
    timeout: 5000,
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  const response = JSON.parse(result.stdout);
  return response.hookSpecificOutput?.permissionDecision ?? "allow";
};
const patch = (...lines) => ["*** Begin Patch", ...lines, "*** End Patch"].join("\n");

test("permite documentación que menciona rutas protegidas", () => {
  assert.equal(decision("apply_patch", patch(
    "*** Add File: F:/Apps/NetBench/ARCHITECTURE.md",
    "+Consultar .specify/memory/constitution.md, Historias.md y Design/.",
    "+No editar .agents/skills/speckit-plan/SKILL.md.",
  )), "allow");
});

test("no interpreta cabeceras citadas dentro del contenido como destinos", () => {
  assert.equal(decision("functions.apply_patch", patch(
    "*** Update File: docs/governance/README.md", "@@",
    "-Referencia a Historias.md",
    "+*** Update File: .specify/memory/constitution.md",
    " Referencia a Design/README.md",
    "*** End of File",
  )), "allow");
});

for (const target of [
  ".agents/skills/speckit-plan/SKILL.md",
  "Design/README.md",
  "Especificacion.md",
  "AUDITORIA_DISENO_V3.md",
  "F:/Apps/NetBench/.specify/templates/plan-template.md",
  "F:\\Apps\\NetBench\\.specify\\templates\\plan-template.md",
]) {
  for (const operation of ["Add File", "Update File", "Delete File"]) {
    test(`bloquea ${operation}: ${target}`, () => {
      const body = operation === "Add File" ? ["+contenido"]
        : operation === "Update File" ? ["@@", "-antes", "+después"] : [];
      assert.equal(decision("apply_patch", patch(`*** ${operation}: ${target}`, ...body)), "deny");
    });
  }
}

test("bloquea mover desde una ruta protegida", () => {
  assert.equal(decision("apply_patch", patch(
    "*** Update File: Design/README.md", "*** Move to: docs/copia.md",
    "@@", "-antes", "+después",
  )), "deny");
});

test("bloquea mover hacia una ruta protegida", () => {
  assert.equal(decision("apply_patch", patch(
    "*** Update File: docs/copia.md", "*** Move to: Design/README.md",
    "@@", "-antes", "+después",
  )), "deny");
});

test("bloquea un parche mixto si cualquiera de sus destinos está protegido", () => {
  assert.equal(decision("apply_patch", patch(
    "*** Add File: ARCHITECTURE.md", "+documentación",
    "*** Update File: Design/README.md", "@@", "-antes", "+después",
  )), "deny");
});

test("acepta payload estructurado y CRLF", () => {
  const input = patch("*** Add File: docs/nota.md", "+Historias.md").replaceAll("\n", "\r\n");
  assert.equal(decision("apply_patch", { patch: input }), "allow");
});

test("conserva fallback ante parche desconocido o truncado", () => {
  assert.equal(decision("apply_patch", "*** Update File: Design/README.md"), "deny");
  assert.equal(decision("apply_patch", patch(
    "*** Formato desconocido: Design/README.md",
  )), "deny");
});

test("mantiene los campos de ruta de herramientas de edición", () => {
  assert.equal(decision("Edit", { file_path: "Design/README.md", new_string: "texto" }), "deny");
  assert.equal(decision("Write", { file_path: "docs/nota.md", content: "Historias.md" }), "allow");
});

test("mantiene lecturas permitidas y escrituras shell bloqueadas", () => {
  assert.equal(decision("exec_command", { cmd: "Get-Content Design/README.md" }), "allow");
  assert.equal(decision("exec_command", { cmd: "Set-Content Design/README.md 'texto'" }), "deny");
});

test("Historias.md dejó de estar protegido (decisión del propietario, 2026-09-21)", () => {
  assert.equal(decision("Edit", { file_path: "Historias.md", new_string: "texto" }), "allow");
  assert.equal(decision("exec_command", { cmd: "Set-Content Historias.md 'texto'" }), "allow");
  assert.equal(decision("apply_patch", patch("*** Update File: Historias.md", "@@", "-a", "+b")), "allow");
});

test("la constitución está exenta del guard: el permiso lo pide settings.json (2026-09-21)", () => {
  const c = ".specify/memory/constitution.md";
  assert.equal(decision("Edit", { file_path: c, new_string: "texto" }), "allow");
  assert.equal(decision("Write", { file_path: "F:/Apps/NetBench/" + c, content: "x" }), "allow");
  assert.equal(decision("exec_command", { cmd: `Set-Content ${c} 'texto'` }), "allow");
  assert.equal(decision("apply_patch", patch(`*** Update File: ${c}`, "@@", "-a", "+b")), "allow");
});

test("la exención no alcanza al resto de .specify ni a un comando mixto", () => {
  assert.equal(decision("Edit", { file_path: ".specify/templates/plan-template.md", new_string: "x" }), "deny");
  assert.equal(decision("Edit", { file_path: ".specify/integrations/codex.manifest.json", new_string: "x" }), "deny");
  assert.equal(decision("exec_command", {
    cmd: "cp .specify/memory/constitution.md .specify/templates/constitution-template.md",
  }), "deny");
});
