import { reactConfig } from "@chrdfin/eslint-config/react";

const config = [
  ...reactConfig,
  {
    ignores: ["src/routeTree.gen.ts", "dist/**", "src-tauri/**"],
  },
];

export default config;
