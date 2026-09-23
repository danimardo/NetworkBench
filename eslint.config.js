import ts from "typescript-eslint";
import svelte from "eslint-plugin-svelte";

export default ts.config(
  ...ts.configs.recommended,
  ...svelte.configs["flat/recommended"],
  {
    files: ["**/*.ts", "**/*.svelte.ts"],
    languageOptions: {
      parser: ts.parser,
    },
  },
  {
    files: ["*.svelte", "**/*.svelte"],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
      },
    },
    rules: {
      "svelte/no-at-html-tags": "warn",
    },
  },
  {
    rules: {
      "no-console": ["error", { allow: ["warn", "error"] }],
    },
  },
  {
    ignores: [
      "dist/",
      "node_modules/",
      "src-tauri/",
      ".specify/",
      "Design/",
      "scripts/",
      "coverage/",
    ],
  },
);
