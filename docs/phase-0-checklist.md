# Phase 0: Foundation & Tooling — Implementation Checklist

**Goal:** A fully configured monorepo with a running Tauri v2 desktop app, DuckDB schema initialized, CI pipeline, and all package stubs. The platform shell (sidebar, header) renders with navigation to all feature domains. Every TypeScript package builds and passes lint/typecheck. The Rust workspace compiles. A placeholder Tauri command round-trips data between the frontend and backend.

**Approach:** Work through tasks sequentially. Each task builds on the previous. Verify each step compiles/passes before moving on.

---

## Task 0.1: Initialize Monorepo Root

Create the root `chrdfin/` directory with pnpm workspaces and Turborepo.

### Files to create

**`package.json`** (root):

```jsonc
{
  "name": "chrdfin",
  "private": true,
  "packageManager": "pnpm@9.15.4",
  "engines": {
    "node": ">=22.0.0"
  },
  "scripts": {
    "dev": "turbo run dev",
    "build": "turbo run build",
    "typecheck": "turbo run typecheck",
    "lint": "turbo run lint",
    "lint:fix": "turbo run lint:fix",
    "test": "turbo run test",
    "test:watch": "turbo run test:watch",
    "format": "prettier --write .",
    "format:check": "prettier --check .",
    "tauri": "pnpm --filter desktop tauri",
    "tauri:dev": "pnpm --filter desktop tauri dev",
    "tauri:build": "pnpm --filter desktop tauri build",
    "clean": "turbo run clean && rm -rf node_modules"
  }
}
```

**`pnpm-workspace.yaml`**:

```yaml
packages:
  - "apps/*"
  - "packages/*"
```

**`turbo.json`**:

```jsonc
{
  "$schema": "https://turbo.build/schema.json",
  "globalDependencies": [".env"],
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": ["dist/**"]
    },
    "dev": {
      "cache": false,
      "persistent": true
    },
    "lint": {
      "dependsOn": ["^build"]
    },
    "lint:fix": {
      "dependsOn": ["^build"]
    },
    "typecheck": {
      "dependsOn": ["^build"]
    },
    "test": {
      "dependsOn": ["^build"]
    },
    "test:watch": {
      "cache": false,
      "persistent": true
    },
    "clean": {
      "cache": false
    }
  }
}
```

**`Cargo.toml`** (workspace root):

```toml
[workspace]
members = [
    "crates/chrdfin-core",
    "apps/desktop/src-tauri",
]
resolver = "2"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
```

**`.env.example`**:

```bash
# Data Providers
TIINGO_API_KEY=your_tiingo_api_key
FRED_API_KEY=your_fred_api_key

# Optional: Real-time quotes and options data
POLYGON_API_KEY=your_polygon_api_key

# App
NODE_ENV=development
```

**`.gitignore`**: See `docs/_gitignore` (renamed to `.gitignore` in the repo).

**`.prettierrc`**:

```json
{
  "semi": true,
  "singleQuote": true,
  "trailingComma": "all",
  "printWidth": 100,
  "tabWidth": 2,
  "plugins": ["prettier-plugin-tailwindcss"]
}
```

**`.prettierignore`**:

```
node_modules
dist
target
coverage
pnpm-lock.yaml
Cargo.lock
```

**`.nvmrc`**:

```
22
```

**`rust-toolchain.toml`**:

```toml
[toolchain]
channel = "stable"
```

### Verification

- [ ] `pnpm install` runs without errors

---

## Task 0.2: Shared TypeScript Configuration (`@chrdfin/tsconfig`)

Create the shared tsconfig package.

### Directory: `packages/tsconfig/`

**`packages/tsconfig/package.json`**:

```json
{
  "name": "@chrdfin/tsconfig",
  "version": "0.0.0",
  "private": true,
  "license": "MIT",
  "files": ["*.json"]
}
```

**`packages/tsconfig/base.json`**:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "incremental": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "noUncheckedIndexedAccess": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "exactOptionalPropertyTypes": false,
    "verbatimModuleSyntax": true
  },
  "exclude": ["node_modules", "dist", "coverage"]
}
```

**`packages/tsconfig/react-library.json`**:

```json
{
  "extends": "./base.json",
  "compilerOptions": {
    "jsx": "react-jsx",
    "lib": ["ES2022", "DOM", "DOM.Iterable"]
  }
}
```

**`packages/tsconfig/react-app.json`**:

```json
{
  "extends": "./base.json",
  "compilerOptions": {
    "jsx": "react-jsx",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  }
}
```

**`packages/tsconfig/library.json`**:

```json
{
  "extends": "./base.json",
  "compilerOptions": {
    "outDir": "./dist",
    "rootDir": "./src"
  }
}
```

---

## Task 0.3: Shared ESLint Configuration (`@chrdfin/eslint-config`)

### Directory: `packages/eslint-config/`

Create three flat config files: `base.js`, `react.js`. Each exports a flat config array.

**Implementation notes:**

- ESLint 9 flat config format.
- `@typescript-eslint/eslint-plugin` and `@typescript-eslint/parser`.
- `eslint-plugin-import-x` for import sorting.
- `@typescript-eslint/consistent-type-imports: error`.
- Import boundary rules: packages must not import from apps. Domain route directories must not import from other domain route directories.

---

## Task 0.4: Leaf Packages (`@chrdfin/types`, `@chrdfin/config`)

These are the two leaf packages with no internal dependencies.

### `packages/types/`

Shared TypeScript interfaces and Zod schemas. See `docs/type-definitions-reference.md` for the complete type inventory.

**Key files to create:**

- `package.json` — name: `@chrdfin/types`
- `tsconfig.json` — extends `@chrdfin/tsconfig/library.json`
- `src/index.ts` — barrel export
- All type files from `docs/type-definitions-reference.md`

### `packages/config/`

Shared configuration, constants, and feature flags.

**`src/features.ts`**:

```typescript
export const FEATURES = {
  backtest: true,
  monteCarlo: true,
  tracker: true,
  optimizer: false,
  calculators: true,
  marketData: true,
  news: true,
  research: false,
} as const satisfies Record<string, boolean>;

export type FeatureId = keyof typeof FEATURES;

export function isFeatureEnabled(id: FeatureId): boolean {
  return FEATURES[id] ?? false;
}
```

**`src/constants.ts`**:

```typescript
export const APP_NAME = 'chrdfin';
export const APP_DESCRIPTION = 'Personal Financial Intelligence Platform';

export const DEFAULT_BENCHMARK = 'SPY';
export const DEFAULT_RISK_FREE_RATE_SERIES = 'DGS3MO';
export const DEFAULT_INFLATION_SERIES = 'CPIAUCSL';

export const MARKET_HOURS = {
  open: { hour: 9, minute: 30 },
  close: { hour: 16, minute: 0 },
  timezone: 'America/New_York',
} as const;

export const POLLING_INTERVALS = {
  realTimeQuotes: 15_000,
  newsSync: 900_000,
} as const;

export const DATA_LIMITS = {
  maxTickersPerQuery: 50,
  maxTickersPerBatch: 100,
  maxScreenerResults: 500,
  maxBacktestAssets: 50,
  maxMCIterations: 1_000_000,  // Much higher than web — native Rust handles this
  maxNewsResults: 100,
} as const;
```

---

## Task 0.5: UI and Charts Packages (`@chrdfin/ui`, `@chrdfin/charts`)

### `@chrdfin/ui`

- Depends on: `@chrdfin/types`, `@chrdfin/config`
- Initialize shadcn/ui with the default theme.
- Must include: `Button`, `Input`, `Card`, `Badge`, `Tabs`, `Sheet`, `Dialog`, `Select`, `Separator` as minimum primitives.
- Must include: `lib/utils.ts` with `cn()` utility (clsx + tailwind-merge).

### `@chrdfin/charts`

- Depends on: `@chrdfin/types`, `@chrdfin/ui`
- Stub: placeholder wrapper components for Lightweight Charts and Recharts.

---

## Task 0.6: Tauri v2 Application (`apps/desktop`)

This is the core application scaffold. The Tauri app has two parts: a Vite + React frontend and a Rust backend.

### Frontend Setup (Vite + React)

1. Initialize with Vite: `pnpm create vite apps/desktop --template react-ts`
2. Install Tauri dependencies: `@tauri-apps/api`, `@tauri-apps/plugin-*`
3. Install routing: `@tanstack/react-router`
4. Install data fetching: `@tanstack/react-query`
5. Install form handling: `react-hook-form`, `@hookform/resolvers`
6. Install UI: reference `@chrdfin/ui` via workspace protocol
7. Configure Tailwind CSS 4

**`vite.config.ts`**:

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
});
```

### Rust Backend Setup (Tauri)

**`apps/desktop/src-tauri/Cargo.toml`**:

```toml
[package]
name = "chrdfin-desktop"
version = "0.1.0"
edition = "2021"

[lib]
name = "chrdfin_desktop_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
chrdfin-core = { path = "../../../crates/chrdfin-core" }
duckdb = { version = "1", features = ["bundled"] }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { version = "1", features = ["full"] }
thiserror = { workspace = true }
tracing = "0.1"
```

**`apps/desktop/src-tauri/tauri.conf.json`**:

```jsonc
{
  "$schema": "https://raw.githubusercontent.com/nicktomlin/tauri/refs/heads/dev/crates/tauri-cli/schema.json",
  "productName": "chrdfin",
  "version": "0.1.0",
  "identifier": "com.chrdfin.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build"
  },
  "app": {
    "windows": [
      {
        "title": "chrdfin",
        "width": 1440,
        "height": 900,
        "minWidth": 1024,
        "minHeight": 680,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

**`apps/desktop/src-tauri/src/main.rs`** — Placeholder:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod state;
mod error;

use state::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .setup(|app| {
            let db = db::initialize_db(app.handle())?;
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**`apps/desktop/src-tauri/src/commands/mod.rs`** — Placeholder:

```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub db_initialized: bool,
    pub version: String,
}

#[tauri::command]
pub fn health_check() -> HealthCheckResponse {
    HealthCheckResponse {
        status: "ok".to_string(),
        db_initialized: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}
```

### Platform Shell Components

**`src/components/shell/Sidebar.tsx`**:

- Collapsible sidebar with navigation sections: Analysis, Tracking, Tools, Market & Research.
- Each section lists navigation items from domain manifests.
- Items conditionally rendered based on feature flags from `@chrdfin/config`.
- Collapse/expand toggle.

**`src/components/shell/Header.tsx`**:

- Breadcrumbs, global ticker search placeholder, market status indicator placeholder.

**`src/routes/__root.tsx`**:

- Root layout: platform shell (Sidebar + Header) wrapping `<Outlet />`.

### TanStack Router Routes

Create ALL domain placeholder routes. Each placeholder should render the page title, description, and phase number.

```
src/routes/
├── __root.tsx               # Root layout with platform shell
├── index.tsx                # Dashboard home
├── analysis/
│   ├── backtest.tsx         # "Backtesting — Phase 3"
│   ├── backtest.$id.tsx     # Backtest results (placeholder)
│   ├── monte-carlo.tsx      # "Monte Carlo — Phase 4"
│   ├── monte-carlo.$id.tsx  # MC results (placeholder)
│   └── optimizer.tsx        # "Optimizer — Phase 9"
├── tracking/
│   ├── portfolio.tsx        # "Portfolio Tracker — Phase 5"
│   ├── portfolio.$id.tsx    # Portfolio detail (placeholder)
│   ├── transactions.tsx     # "Transactions — Phase 5"
│   └── watchlist.tsx        # "Watchlists — Phase 5"
├── tools/
│   ├── calculators.tsx                   # "Calculators — Phase 6"
│   ├── calculators.compound-growth.tsx   # Placeholder
│   ├── calculators.retirement.tsx        # Placeholder
│   ├── calculators.withdrawal.tsx        # Placeholder
│   ├── calculators.options-payoff.tsx    # Placeholder
│   ├── calculators.tax-loss.tsx          # Placeholder
│   └── compare.tsx                       # "Compare — Phase 10"
└── market/
    ├── screener.tsx          # "Screener — Phase 7"
    ├── ticker.$symbol.tsx    # "Ticker Detail — Phase 7"
    ├── options.$symbol.tsx   # "Options — Phase 7"
    ├── news.tsx              # "News — Phase 8"
    └── calendar.tsx          # "Calendar — Phase 8"
```

### Placeholder route pattern

```tsx
// Example: src/routes/analysis/backtest.tsx
import { createFileRoute } from '@tanstack/react-router';
import { Card, CardContent, CardHeader, CardTitle } from '@chrdfin/ui';

export const Route = createFileRoute('/analysis/backtest')({
  component: BacktestPage,
});

function BacktestPage() {
  return (
    <div className="p-6">
      <Card>
        <CardHeader>
          <CardTitle>Portfolio Backtesting</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">
            Historical portfolio simulation with configurable rebalancing strategies.
            Implementation planned for Phase 3.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
```

---

## Task 0.7: Rust Crate (`chrdfin-core`)

### Directory: `crates/chrdfin-core/`

**`Cargo.toml`**:

```toml
[package]
name = "chrdfin-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
rayon = "1"
rand = "0.8"
statrs = "0.17"
nalgebra = "0.33"
chrono = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
approx = "0.5"
```

**`src/lib.rs`** — Placeholder with module stubs:

```rust
pub mod backtest;
pub mod monte_carlo;
pub mod optimizer;
pub mod stats;
pub mod portfolio;
pub mod matrix;
pub mod calculators;
pub mod types;

/// Placeholder: validates the crate compiles and links.
pub fn health_check() -> String {
    "chrdfin-core loaded successfully".to_string()
}
```

Create stub files for each module:

- `src/backtest.rs` — `// TODO: Phase 2 — Backtesting logic`
- `src/monte_carlo.rs` — `// TODO: Phase 4 — Monte Carlo simulation`
- `src/optimizer.rs` — `// TODO: Phase 9 — Portfolio optimization`
- `src/stats.rs` — `// TODO: Phase 2 — Statistical functions`
- `src/portfolio.rs` — `// TODO: Phase 2 — Portfolio math`
- `src/matrix.rs` — `// TODO: Phase 9 — Linear algebra`
- `src/calculators.rs` — `// TODO: Phase 6 — Financial calculators`
- `src/types.rs` — Shared Rust types with serde Serialize/Deserialize

---

## Task 0.8: DuckDB Schema Initialization

Inside `apps/desktop/src-tauri/src/`:

**`schema.sql`** — All `CREATE TABLE IF NOT EXISTS` statements from `docs/database-schema-reference.md`.

**`db.rs`** — DuckDB connection management:

```rust
use duckdb::{Connection, Result};
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn initialize(app_handle: &tauri::AppHandle) -> Result<Self> {
        let data_dir = app_handle
            .path()
            .app_data_dir()
            .expect("failed to resolve app data dir");
        std::fs::create_dir_all(&data_dir).expect("failed to create data directory");
        let db_path = data_dir.join("chrdfin.duckdb");

        let conn = Connection::open(db_path)?;
        conn.execute_batch(include_str!("schema.sql"))?;

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }
}
```

### Verification

- [ ] `cargo build` compiles the Tauri app with DuckDB schema initialization.

---

## Task 0.9: Vitest Configuration

Configure Vitest for unit testing across TypeScript packages.

### Root `vitest.workspace.ts`

```typescript
import { defineWorkspace } from 'vitest/config';

export default defineWorkspace([
  'packages/*/vitest.config.ts',
]);
```

Create at least one passing test per testable package:

- `packages/types/src/__tests__/schemas.test.ts` — Test a Zod schema parse.
- `packages/config/src/__tests__/features.test.ts` — Test `isFeatureEnabled()`.
- `packages/ui/src/__tests__/utils.test.ts` — Test `cn()` utility.

---

## Task 0.10: GitHub Actions CI Pipeline

**`.github/workflows/ci.yml`**:

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  typescript:
    name: TypeScript Quality
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm typecheck
      - run: pnpm lint
      - run: pnpm format:check
      - run: pnpm test

  rust:
    name: Rust Quality
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
      - run: cargo check --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo test --workspace
```

---

## Task 0.11: Verify Frontend-Backend Round Trip

After all setup is complete, verify that the Tauri IPC works end-to-end:

**Frontend test** (add to the dashboard home route):

```typescript
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

function DashboardPage() {
  const [health, setHealth] = useState<string>('checking...');

  useEffect(() => {
    invoke('health_check').then((result) => {
      setHealth(JSON.stringify(result, null, 2));
    });
  }, []);

  return (
    <div className="p-6">
      <h1>chrdfin Dashboard</h1>
      <pre className="mt-4 rounded bg-muted p-4 text-sm">{health}</pre>
    </div>
  );
}
```

### Verification

- [ ] `pnpm tauri dev` launches the app.
- [ ] The health check response displays in the dashboard.

---

## Completion Checklist

Run these in order. All must pass:

```bash
pnpm install                 # No errors
pnpm typecheck               # All packages pass
pnpm lint                    # Zero warnings
pnpm test                    # All stub tests pass
cargo check --workspace      # Rust compiles
cargo clippy --workspace -- -D warnings  # No clippy warnings
cargo test --workspace       # Rust tests pass
pnpm tauri dev               # App launches
```

Additionally verify:

- [ ] All feature domain routes have placeholder pages.
- [ ] Platform shell (sidebar with navigation sections, header) renders.
- [ ] Sidebar shows: Analysis (Backtesting, Monte Carlo, Optimizer), Tracking (Portfolio, Transactions, Watchlists), Tools (Calculators, Compare), Market (Screener, News, Calendar).
- [ ] Feature flags gate navigation items: Optimizer and Research do not appear.
- [ ] DuckDB schema initializes on app launch with all tables from `database-schema-reference.md`.
- [ ] `health_check` Tauri command returns a response to the frontend.
- [ ] `@chrdfin/types` exports types for all domains.
- [ ] `Cargo.lock` is tracked in git.
- [ ] GitHub Actions CI workflow file exists and is valid YAML.

When all pass, Phase 0 is complete. Proceed to Phase 1: Data Layer.
