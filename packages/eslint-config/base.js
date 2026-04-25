import js from "@eslint/js";
import importX from "eslint-plugin-import-x";
import tseslint from "typescript-eslint";
import globals from "globals";

/**
 * Shared base ESLint config for all chrdfin packages.
 *
 * - ESLint 9 flat config.
 * - TypeScript via typescript-eslint.
 * - Import boundary enforcement via eslint-plugin-import-x.
 * - Forbids: default exports, `any`, console (except warn/error), localStorage,
 *   sessionStorage, raw color hex literals (warn).
 */
export const baseConfig = [
  {
    ignores: [
      "**/dist/**",
      "**/build/**",
      "**/coverage/**",
      "**/node_modules/**",
      "**/target/**",
      "**/.turbo/**",
      "**/routeTree.gen.ts",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
      globals: {
        ...globals.es2022,
        ...globals.node,
      },
    },
    plugins: {
      "import-x": importX,
    },
    rules: {
      // Named-export discipline (CLAUDE.md hard rule).
      "import-x/no-default-export": "error",
      "import-x/no-anonymous-default-export": "error",

      // Import ordering & hygiene.
      "import-x/order": [
        "error",
        {
          groups: ["builtin", "external", "internal", ["parent", "sibling", "index"], "type"],
          "newlines-between": "always",
          alphabetize: { order: "asc", caseInsensitive: true },
        },
      ],
      "import-x/no-cycle": ["error", { maxDepth: 4 }],

      // Type-only imports must be marked.
      "@typescript-eslint/consistent-type-imports": [
        "error",
        { prefer: "type-imports", fixStyle: "separate-type-imports" },
      ],

      // No `any` (CLAUDE.md hard rule).
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],

      // Storage forbidden — must go through Tauri commands + DuckDB.
      "no-restricted-globals": [
        "error",
        {
          name: "localStorage",
          message: "Use Tauri commands + DuckDB for persistent state (CLAUDE.md).",
        },
        {
          name: "sessionStorage",
          message: "Use Tauri commands + DuckDB for persistent state (CLAUDE.md).",
        },
      ],
      "no-restricted-properties": [
        "error",
        {
          object: "window",
          property: "localStorage",
          message: "Use Tauri commands + DuckDB.",
        },
        {
          object: "window",
          property: "sessionStorage",
          message: "Use Tauri commands + DuckDB.",
        },
      ],

      // Console hygiene.
      "no-console": ["warn", { allow: ["warn", "error"] }],

      // Exhaustive switch via never.
      "@typescript-eslint/switch-exhaustiveness-check": "off",
    },
  },
  {
    // Allow default exports in config files & route metadata where required by tooling.
    files: [
      "**/*.config.{js,ts,mjs,cjs}",
      "**/vite.config.*",
      "**/vitest.config.*",
      "**/tailwind.config.*",
      "**/eslint.config.*",
      "**/postcss.config.*",
    ],
    rules: {
      "import-x/no-default-export": "off",
    },
  },
];

export default baseConfig;
