import { defineWorkspace } from "vitest/config";

export default defineWorkspace([
  "apps/desktop/vitest.config.ts",
  "packages/charts/vitest.config.ts",
  "packages/config/vitest.config.ts",
  "packages/types/vitest.config.ts",
  "packages/ui/vitest.config.ts",
]);
