# Agent Handoff — chrdfin

## Purpose

You are Claude Code, working on chrdfin. This document is your map of the documentation tree. Read this file first, then the documents it routes you to based on the task at hand.

The goal is simple: do not invent patterns when patterns already exist. Every architectural and stylistic decision in this codebase has a written contract. Find the contract, follow it.

---

## What is chrdfin?

A personal financial intelligence desktop application: portfolio backtesting, Monte Carlo simulation, portfolio tracking, optimization, calculators, market data, screener, and news. Built as a Tauri v2 native desktop app — **not** a web app — with a Vite + React 19 SPA frontend and a Rust computation/storage backend using DuckDB.

Single-user. Personal use. No live trading. No multi-user concerns. Open-source intent.

The platform aesthetic is Bloomberg Terminal: dense, tabular, information-rich, no decoration.

For full architectural context: read `docs/technical-blueprint.md` and `CLAUDE.md` (root). Both are required reading before any non-trivial work.

---

## Document Inventory

| Document | What it covers | When to read |
|---|---|---|
| `CLAUDE.md` (root) | Repository structure, code conventions, hard rules ("do NOT use localStorage," etc.), tech stack versions | **Always read first** — this overrides any other doc on conflicts |
| `docs/technical-blueprint.md` | Full system architecture, package boundaries, Tauri command catalog, Rust crate structure | Before architectural work; whenever a Tauri command is added or modified |
| `docs/phase-0-checklist.md` | Step-by-step Phase 0 implementation tasks | When scaffolding the monorepo or completing Phase 0 deliverables |
| `docs/database-schema-reference.md` | DuckDB schema definitions, table relationships, migration strategy | Before any schema change; when adding a Tauri command that reads/writes new tables |
| `docs/type-definitions-reference.md` | Shared TypeScript types and Zod schemas in `@chrdfin/types` | Before adding a new shared type; when changing types that cross the Rust↔TS boundary |
| `docs/ui-design-system.md` | Color tokens (Carbon-derived), typography, density rules, platform shell, the three reference views | All UI work — every component must respect these tokens |
| `docs/ui-component-recipes.md` | Production code for primitives: `useTauriCommand`, `useThemeColors`, `<DeltaValue>`, `<MetricsStrip>`, `<DataTable>`, `<Sparkline>`, `<ThemeProvider>`, `<MarketStatusIndicator>` | Whenever a new component is needed — check if a primitive already exists |
| `docs/chart-recipes.md` | Recharts patterns: `<EquityCurveWithDrawdown>`, `<AllocationDonut>`, `<ReturnsHistogram>`, `<AnnualReturnsBar>`, `<PerformanceArea>`, `<MonteCarloCone>`, plus chart foundations | Any chart implementation; before adding a new visualization type |
| `docs/route-conventions.md` | TanStack Router file-based routing, search param schemas, lazy loading, error/pending boundaries, feature flag integration | Any new route, change to URL structure, or search param handling |
| `docs/data-fetching-patterns.md` | TanStack Query + Tauri command patterns: cache keys, invalidation graph, optimistic mutations, real-time event subscriptions | Any data fetching from Rust; mutations; live quote handling; sync coordination |
| `docs/form-patterns.md` | React Hook Form + Zod conventions, field primitives, search-param-synced forms, field arrays, async validation, mutation integration | Any form, calculator, filter bar, or config UI |
| `docs/agent-handoff.md` | This file — the router | First — every session |

The `docs/` directory is the canonical specification. Code is the implementation; these documents are the design.

---

## Reading Order by Task

Identify the task type, read the listed documents in order, then proceed. If a task spans multiple types, read the union.

### Scaffolding Phase 0

1. `CLAUDE.md`
2. `docs/technical-blueprint.md`
3. `docs/phase-0-checklist.md`
4. `docs/ui-design-system.md`
5. `docs/ui-component-recipes.md` (sections 1, 2, 4, 10, 11)
6. `docs/route-conventions.md` (sections 1-3, 5-6)

### Adding a new Tauri command

1. `CLAUDE.md` (sections on Tauri, Rust conventions)
2. `docs/technical-blueprint.md` (existing command catalog)
3. `docs/database-schema-reference.md` (if reading/writing DuckDB)
4. `docs/type-definitions-reference.md` (define shared input/output types)
5. `docs/data-fetching-patterns.md` (cache key entry, invalidation graph entry)

### Implementing a new UI page

1. `docs/ui-design-system.md` (philosophy, layout)
2. `docs/route-conventions.md` (route file structure, lazy loading)
3. `docs/ui-component-recipes.md` (reusable primitives)
4. `docs/data-fetching-patterns.md` (if the page fetches data)
5. `docs/form-patterns.md` (if the page accepts user input)
6. `docs/chart-recipes.md` (if the page shows charts)

### Adding a chart

1. `docs/chart-recipes.md` (find an existing recipe first; only invent if novel)
2. `docs/ui-design-system.md` (chart palette, density)
3. `docs/ui-component-recipes.md` (the `useThemeColors` hook + formatters)

### Adding or modifying a form

1. `docs/form-patterns.md`
2. `docs/type-definitions-reference.md` (schema location)
3. `docs/route-conventions.md` (if the form syncs to URL)
4. `docs/data-fetching-patterns.md` (if the form submits a mutation)

### Adding a new route

1. `docs/route-conventions.md`
2. `docs/ui-design-system.md` (loading/empty/error treatment)
3. `docs/data-fetching-patterns.md` (if loaders or queries needed)

### Real-time / live data work

1. `docs/data-fetching-patterns.md` (event subscription patterns, the `useTauriEvent` hook)
2. `docs/technical-blueprint.md` (Rust-side event emission)
3. `docs/type-definitions-reference.md` (event payload types)

### Schema or data model changes

1. `docs/database-schema-reference.md`
2. `docs/type-definitions-reference.md` (TS mirror of the schema)
3. `docs/data-fetching-patterns.md` (cache invalidation if affected)
4. `docs/form-patterns.md` (if forms consume the schema)

### Theme, color, or typography work

1. `docs/ui-design-system.md`
2. `docs/ui-component-recipes.md` (the `<ThemeProvider>` and `useThemeColors` recipes)
3. `docs/chart-recipes.md` (if charts are affected)

---

## Cross-Cutting Hard Rules

These overrides apply regardless of task. They appear in `CLAUDE.md` and individual docs but are listed here for anchor reference.

### Storage and persistence

- **Never** use `localStorage` or `sessionStorage`. All persistent state goes through Tauri commands and DuckDB.
- **Never** commit `.env` files. Only `.env.example` is tracked.
- **Never** store API keys in the database or in plaintext config files.

### Architecture

- **Never** use Next.js, server components, API routes, or any SSR pattern. This is a Tauri SPA.
- **Never** use Drizzle, PostgreSQL, Neon, or any external database. DuckDB is the database.
- **Never** use WASM, wasm-pack, wasm-bindgen, or WebWorkers. Computation is native Rust.
- **Never** use SWR — TanStack Query for Tauri command data fetching.
- **Never** use `nuqs` — TanStack Router search params.
- **Never** use `Comlink` — there are no WebWorkers to communicate with.

### Code quality

- **Never** use `any`. Use `unknown` and narrow.
- **Never** use default exports. Named exports throughout.
- **Never** add CSS files. Tailwind utility classes only (one exception: `globals.css`).
- **Never** use `f32` in Rust computation code. `f64` everywhere for numerical precision.
- **Never** panic in Rust computation hot paths. `Result<T, E>` everywhere.
- **Never** import between feature domains. Cross-domain data flows through Tauri commands and `@chrdfin/types`.

### UI

- **Never** use Tailwind palette utilities (`bg-blue-500`, `text-red-600`). Use semantic tokens (`bg-primary`, `text-loss`).
- **Never** use raw color values (hex, rgb, hsl) inline in JSX. Hex appears only in `globals.css`.
- **Never** use drop shadows on internal panels. Shadows are reserved for popovers, dropdowns, dialogs.
- **Never** use icon libraries beyond `lucide-react`.
- **Never** use chart libraries beyond Recharts (without architectural review).
- **Never** use emoji in production UI strings.
- **Never** use illustrations, mascots, or decorative imagery.

### Scope discipline

- **Never** add features outside the current phase's deliverable list without explicit approval.
- **Never** implement broker integrations, live trading, or order execution.

---

## When to Ask vs When to Proceed

The phase checklist and design system documents are designed to be unambiguous. In most cases, proceed.

**Proceed without asking when:**

- The task is in a phase checklist or has a documented recipe
- A primitive in `ui-component-recipes.md` or `chart-recipes.md` already covers the need
- The decision is purely typographic, density, or color and a token exists for it
- A test is missing and you can write one in the documented test pattern

**Ask the user when:**

- The task implies a new feature outside the current phase
- The implementation requires a new dependency not listed in tech stack tables
- A documented constraint and a user request appear to conflict
- An architectural decision affects more than one feature domain
- The fix to a documented bug pattern requires changing the contract itself

When asking, be specific: cite the document, line, or constraint that prompted the question. Do not ask broad framing questions when narrow ones suffice.

---

## Handling Stale or Contradictory Context

The documentation is the source of truth. When code disagrees with documentation:

1. Treat the documentation as authoritative. The code is wrong (or out of date), not the document.
2. If you're certain the code is right, surface the discrepancy to the user before changing the document. Documentation drift is a real issue and silent realignment makes it worse.
3. If multiple documents conflict, the priority order is:
   1. `CLAUDE.md` (root) — overrides everything
   2. `docs/technical-blueprint.md` — overrides specialized docs on architecture
   3. The specialized doc most directly relevant to the task
4. If the user's instructions conflict with documentation, ask. Do not silently override the contract.

---

## Conventions for This Document

This document is small on purpose. It does not duplicate content from the specialized docs — it points to them.

When adding a new specialized doc:

1. Add an entry to the **Document Inventory** table.
2. Add or extend a row in **Reading Order by Task**.
3. If the new doc introduces a hard rule, add it to **Cross-Cutting Hard Rules** and link the doc as the source.

When deprecating a doc:

1. Remove the inventory entry.
2. Remove from all reading-order rows.
3. Add a redirect note to whichever doc supersedes it.

---

## Document Tree at a Glance

```
chrdfin/
├── CLAUDE.md                              ← always-loaded agent context
└── docs/
    ├── agent-handoff.md                   ← THIS DOCUMENT (the router)
    │
    ├── technical-blueprint.md             ← system architecture
    ├── phase-0-checklist.md               ← Phase 0 step-by-step tasks
    ├── database-schema-reference.md       ← DuckDB tables & relationships
    ├── type-definitions-reference.md      ← @chrdfin/types catalog
    │
    ├── ui-design-system.md                ← tokens, philosophy, layout
    ├── ui-component-recipes.md            ← primitives & hooks
    ├── chart-recipes.md                   ← Recharts patterns
    ├── route-conventions.md               ← TanStack Router structure
    ├── data-fetching-patterns.md          ← TanStack Query + Tauri events
    └── form-patterns.md                   ← RHF + Zod conventions
```

The four "UI handoff" docs (design system, component recipes, chart recipes, route conventions) plus the two "interaction layer" docs (data fetching, forms) form a self-contained front-end specification. The three "system" docs (technical blueprint, schema, types) form the cross-cutting backbone. CLAUDE.md is the root contract.

Read what's relevant. Implement what's specified. Ask when something genuinely doesn't fit.
