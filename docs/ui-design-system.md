# UI Design System & Reference Implementations — chrdfin

## Purpose

This document is the canonical specification for the chrdfin user interface: design philosophy, color tokens, typography, density rules, platform shell, component patterns, and three reference views. It is the visual companion to `docs/technical-blueprint.md`.

Claude Code agents implementing UI work in this repository follow this document. The color system, typography stack, and component patterns defined here are non-negotiable across all feature domains.

**Scope of this document:**

- Phase 0 platform shell (sidebar, header, command palette, route placeholders)
- Color tokens and theming for `apps/desktop` and `@chrdfin/ui`
- Reference implementations for Phases 2 (Backtesting), 5 (Portfolio Tracker), and 7 (Screener) — provided as concrete targets to anchor design decisions made during the foundation phase

**Out of scope:**

- Implementation logic for any feature domain beyond Phase 0
- Chart configuration internals (see `@chrdfin/charts` package docs once authored)
- Domain-specific business rules (see per-phase checklists)

---

## Design Philosophy

chrdfin is a desktop financial workstation, not a SaaS dashboard. It targets a single power user who values information density, repeatability, and low-friction navigation over decoration. Every UI decision is governed by the following principles, ordered by priority:

1. **Information density over whitespace.** A typical screen displays 50–200 numerical values. Padding, line-height, and component spacing are tuned for scanning, not for breathing room. Default row height for tabular data is 28–32px. Default control height is 28–32px.

2. **Muted palette as the default surface, color as signal.** Chrome (sidebars, headers, table borders, panel backgrounds) is grayscale. Color is reserved for actionable signal: gain/loss tinting on numbers, the active interactive state, focus rings, and chart series.

3. **Tabular numerals everywhere.** All numeric output uses `font-variant-numeric: tabular-nums` so columns of figures align without manual tracking. This is enforced globally in `globals.css` on `*`.

4. **Typographic hierarchy through size and weight, not color variety.** Most UI text is 12–13px. Labels are 11px muted; values are 12–14px primary; emphasized data is 14–16px medium-weight. Display-size text (≥18px) is reserved for the single largest figure on a view (e.g. portfolio total value).

5. **Borders and surface stepping over shadows.** Layering uses Carbon's stepped-surface model: layer-01, layer-02, layer-03 each one step lighter (dark theme) or alternating (light theme). No drop shadows on internal panels. Shadows are reserved for floating overlays only (popovers, dropdowns, dialogs).

6. **Flat geometry, minimal radius.** Border radius is `0` for tabular cells, `2px` for small controls (inputs, buttons, badges), `4px` for cards and panels. There is no `12px+` rounding anywhere. No gradients. No glows.

7. **Keyboard-first interaction.** Every primary action has a keyboard shortcut. The command palette (`Cmd/Ctrl+K`) is the primary navigation method for power users. Tab order is logical and visible focus rings are mandatory.

8. **Consistent information architecture.** Every view follows the same skeleton: top metrics strip → main content (table, chart, or form) → secondary panels. Users should never have to learn a new layout language between views.

---

## Color System

The color system is derived from IBM's Carbon Design System. Carbon's neutral gray palette and Blue 60 (#0f62fe) form the foundation; gain/loss semantics are layered on top using Carbon's Green and Red palettes calibrated for accessibility on each theme's surfaces.

The system is implemented as CSS custom properties on `:root` (light theme, default) and `.dark` (dark theme), exposed to Tailwind v4 via `@theme inline`. This is the current shadcn/ui convention as of February 2025.

### Token Methodology

Two layers of tokens:

| Layer | Purpose | Examples |
|---|---|---|
| **Reference tokens** | Raw palette values from Carbon. Never used directly by components. | `--blue-60`, `--gray-100` |
| **Semantic tokens** | Role-based aliases that components consume. Map to reference tokens per theme. | `--background`, `--primary`, `--gain` |

Components only ever reference semantic tokens. This is what allows the entire app to retheme by toggling a single class on `<html>`.

### Light Theme Palette

The light theme is Carbon's "White" theme. Background is pure white; the first content layer drops to Gray-10 for visual stratification.

| Semantic Token | Hex | Carbon Reference | Role |
|---|---|---|---|
| `--background` | `#ffffff` | white | Application root background |
| `--foreground` | `#161616` | gray-100 | Primary text on background |
| `--card` | `#f4f4f4` | gray-10 | Layer-01: panels, cards |
| `--card-foreground` | `#161616` | gray-100 | Text on card |
| `--popover` | `#ffffff` | white | Floating surfaces (dropdowns, dialogs) |
| `--popover-foreground` | `#161616` | gray-100 | Text on popover |
| `--primary` | `#0f62fe` | blue-60 | Primary interactive (filled buttons, active nav indicator) |
| `--primary-foreground` | `#ffffff` | white | Text on primary surfaces |
| `--secondary` | `#e0e0e0` | gray-20 | Secondary buttons, neutral chips |
| `--secondary-foreground` | `#161616` | gray-100 | Text on secondary |
| `--muted` | `#f4f4f4` | gray-10 | Subtle background fills (table zebra, hover states) |
| `--muted-foreground` | `#525252` | gray-70 | De-emphasized text (labels, helper text) |
| `--accent` | `#e8e8e8` | gray-15 (interpolated) | Hover background on neutral elements |
| `--accent-foreground` | `#161616` | gray-100 | Text on accent |
| `--destructive` | `#da1e28` | red-60 | Destructive action (delete, danger) |
| `--destructive-foreground` | `#ffffff` | white | Text on destructive |
| `--border` | `#e0e0e0` | gray-20 | Default borders, table dividers |
| `--input` | `#e0e0e0` | gray-20 | Form field borders |
| `--ring` | `#0f62fe` | blue-60 | Focus ring |
| **Layering extensions** | | | |
| `--layer-01` | `#f4f4f4` | gray-10 | First nested surface |
| `--layer-02` | `#ffffff` | white | Second nested surface |
| `--layer-03` | `#f4f4f4` | gray-10 | Third nested surface |
| **Financial semantics** | | | |
| `--gain` | `#198038` | green-60 | Positive value text/icon (WCAG AA on white) |
| `--loss` | `#da1e28` | red-60 | Negative value text/icon (WCAG AA on white) |
| `--neutral` | `#525252` | gray-70 | Zero/unchanged value |
| `--warning` | `#f1c21b` | yellow-30 | Pre/after-market, caution states |
| **Chart series** | | | |
| `--chart-1` | `#0f62fe` | blue-60 | Portfolio / primary series |
| `--chart-2` | `#009d9a` | teal-60 | Benchmark / secondary series |
| `--chart-3` | `#8a3ffc` | purple-60 | Tertiary |
| `--chart-4` | `#1192e8` | cyan-60 | Quaternary |
| `--chart-5` | `#ee5396` | magenta-60 | Quinary |

### Dark Theme Palette

The dark theme is Carbon's "Gray 100" theme. Background is Gray-100; nested layers progress one full step lighter on the gray scale (Gray-90 → Gray-80 → Gray-70). This is the inverse of the light theme's alternation pattern but the same conceptual layering model.

| Semantic Token | Hex | Carbon Reference | Role |
|---|---|---|---|
| `--background` | `#161616` | gray-100 | Application root background |
| `--foreground` | `#f4f4f4` | gray-10 | Primary text on background |
| `--card` | `#262626` | gray-90 | Layer-01: panels, cards |
| `--card-foreground` | `#f4f4f4` | gray-10 | Text on card |
| `--popover` | `#262626` | gray-90 | Floating surfaces |
| `--popover-foreground` | `#f4f4f4` | gray-10 | Text on popover |
| `--primary` | `#0f62fe` | blue-60 | Primary interactive (filled) |
| `--primary-foreground` | `#ffffff` | white | Text on primary |
| `--secondary` | `#393939` | gray-80 | Secondary buttons, neutral chips |
| `--secondary-foreground` | `#f4f4f4` | gray-10 | Text on secondary |
| `--muted` | `#262626` | gray-90 | Subtle fills (zebra, hover) |
| `--muted-foreground` | `#a8a8a8` | gray-40 | De-emphasized text |
| `--accent` | `#393939` | gray-80 | Hover background |
| `--accent-foreground` | `#f4f4f4` | gray-10 | Text on accent |
| `--destructive` | `#fa4d56` | red-50 | Destructive (lighter for dark surface) |
| `--destructive-foreground` | `#ffffff` | white | Text on destructive |
| `--border` | `#393939` | gray-80 | Default borders, table dividers |
| `--input` | `#393939` | gray-80 | Form field borders |
| `--ring` | `#ffffff` | white | Focus ring (Carbon dark convention) |
| **Layering extensions** | | | |
| `--layer-01` | `#262626` | gray-90 | First nested surface |
| `--layer-02` | `#393939` | gray-80 | Second nested surface |
| `--layer-03` | `#525252` | gray-70 | Third nested surface |
| **Financial semantics** | | | |
| `--gain` | `#42be65` | green-40 | Positive (WCAG AA on gray-100/90) |
| `--loss` | `#fa4d56` | red-50 | Negative (WCAG AA on gray-100/90) |
| `--neutral` | `#a8a8a8` | gray-40 | Zero/unchanged |
| `--warning` | `#f1c21b` | yellow-30 | Pre/after-market |
| **Chart series** | | | |
| `--chart-1` | `#4589ff` | blue-50 | Portfolio (lifted for dark contrast) |
| `--chart-2` | `#08bdba` | teal-40 | Benchmark |
| `--chart-3` | `#a56eff` | purple-50 | Tertiary |
| `--chart-4` | `#33b1ff` | cyan-40 | Quaternary |
| `--chart-5` | `#ff7eb6` | magenta-40 | Quinary |

### `apps/desktop/src/globals.css` — Full Theme Declaration

This is the canonical theme file. Save this verbatim as `apps/desktop/src/globals.css`:

```css
@import "tailwindcss";
@import "tw-animate-css";

@custom-variant dark (&:is(.dark *));

/* ============================================================
   Light theme (Carbon "White" theme)
   ============================================================ */
:root {
  /* Surface */
  --background: #ffffff;
  --foreground: #161616;
  --card: #f4f4f4;
  --card-foreground: #161616;
  --popover: #ffffff;
  --popover-foreground: #161616;

  /* Layering (Carbon stepped surface model) */
  --layer-01: #f4f4f4;
  --layer-02: #ffffff;
  --layer-03: #f4f4f4;

  /* Interactive */
  --primary: #0f62fe;
  --primary-foreground: #ffffff;
  --secondary: #e0e0e0;
  --secondary-foreground: #161616;
  --muted: #f4f4f4;
  --muted-foreground: #525252;
  --accent: #e8e8e8;
  --accent-foreground: #161616;
  --destructive: #da1e28;
  --destructive-foreground: #ffffff;

  /* Structure */
  --border: #e0e0e0;
  --input: #e0e0e0;
  --ring: #0f62fe;

  /* Sidebar (shadcn sidebar tokens) */
  --sidebar: #f4f4f4;
  --sidebar-foreground: #161616;
  --sidebar-primary: #0f62fe;
  --sidebar-primary-foreground: #ffffff;
  --sidebar-accent: #e8e8e8;
  --sidebar-accent-foreground: #161616;
  --sidebar-border: #e0e0e0;
  --sidebar-ring: #0f62fe;

  /* Financial semantics */
  --gain: #198038;
  --loss: #da1e28;
  --neutral: #525252;
  --warning: #f1c21b;

  /* Chart series */
  --chart-1: #0f62fe;
  --chart-2: #009d9a;
  --chart-3: #8a3ffc;
  --chart-4: #1192e8;
  --chart-5: #ee5396;

  /* Shape */
  --radius: 0.25rem; /* 4px — used for cards/panels; controls override to 2px */
}

/* ============================================================
   Dark theme (Carbon "Gray 100" theme)
   ============================================================ */
.dark {
  --background: #161616;
  --foreground: #f4f4f4;
  --card: #262626;
  --card-foreground: #f4f4f4;
  --popover: #262626;
  --popover-foreground: #f4f4f4;

  --layer-01: #262626;
  --layer-02: #393939;
  --layer-03: #525252;

  --primary: #0f62fe;
  --primary-foreground: #ffffff;
  --secondary: #393939;
  --secondary-foreground: #f4f4f4;
  --muted: #262626;
  --muted-foreground: #a8a8a8;
  --accent: #393939;
  --accent-foreground: #f4f4f4;
  --destructive: #fa4d56;
  --destructive-foreground: #ffffff;

  --border: #393939;
  --input: #393939;
  --ring: #ffffff;

  --sidebar: #161616;
  --sidebar-foreground: #f4f4f4;
  --sidebar-primary: #0f62fe;
  --sidebar-primary-foreground: #ffffff;
  --sidebar-accent: #393939;
  --sidebar-accent-foreground: #f4f4f4;
  --sidebar-border: #393939;
  --sidebar-ring: #ffffff;

  --gain: #42be65;
  --loss: #fa4d56;
  --neutral: #a8a8a8;
  --warning: #f1c21b;

  --chart-1: #4589ff;
  --chart-2: #08bdba;
  --chart-3: #a56eff;
  --chart-4: #33b1ff;
  --chart-5: #ff7eb6;
}

/* ============================================================
   Tailwind v4 token mapping (@theme inline)
   ============================================================ */
@theme inline {
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-card: var(--card);
  --color-card-foreground: var(--card-foreground);
  --color-popover: var(--popover);
  --color-popover-foreground: var(--popover-foreground);
  --color-primary: var(--primary);
  --color-primary-foreground: var(--primary-foreground);
  --color-secondary: var(--secondary);
  --color-secondary-foreground: var(--secondary-foreground);
  --color-muted: var(--muted);
  --color-muted-foreground: var(--muted-foreground);
  --color-accent: var(--accent);
  --color-accent-foreground: var(--accent-foreground);
  --color-destructive: var(--destructive);
  --color-destructive-foreground: var(--destructive-foreground);
  --color-border: var(--border);
  --color-input: var(--input);
  --color-ring: var(--ring);

  --color-layer-01: var(--layer-01);
  --color-layer-02: var(--layer-02);
  --color-layer-03: var(--layer-03);

  --color-sidebar: var(--sidebar);
  --color-sidebar-foreground: var(--sidebar-foreground);
  --color-sidebar-primary: var(--sidebar-primary);
  --color-sidebar-primary-foreground: var(--sidebar-primary-foreground);
  --color-sidebar-accent: var(--sidebar-accent);
  --color-sidebar-accent-foreground: var(--sidebar-accent-foreground);
  --color-sidebar-border: var(--sidebar-border);
  --color-sidebar-ring: var(--sidebar-ring);

  --color-gain: var(--gain);
  --color-loss: var(--loss);
  --color-neutral: var(--neutral);
  --color-warning: var(--warning);

  --color-chart-1: var(--chart-1);
  --color-chart-2: var(--chart-2);
  --color-chart-3: var(--chart-3);
  --color-chart-4: var(--chart-4);
  --color-chart-5: var(--chart-5);

  --radius-sm: 2px;
  --radius-md: 4px;
  --radius-lg: 8px;

  --font-sans: "IBM Plex Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif;
  --font-mono: "IBM Plex Mono", "JetBrains Mono", "SF Mono", "Cascadia Code", "Fira Code", Consolas, monospace;
}

/* ============================================================
   Base styles
   ============================================================ */
@layer base {
  * {
    border-color: var(--border);
    font-variant-numeric: tabular-nums;
  }

  html,
  body {
    background-color: var(--background);
    color: var(--foreground);
    font-family: var(--font-sans);
    font-size: 13px;
    line-height: 1.4;
    -webkit-font-smoothing: antialiased;
  }

  /* Default focus ring for keyboard nav */
  :focus-visible {
    outline: 2px solid var(--ring);
    outline-offset: 1px;
  }

  /* Scrollbars: thin, theme-aware */
  ::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }
  ::-webkit-scrollbar-track {
    background: var(--background);
  }
  ::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;
  }
  ::-webkit-scrollbar-thumb:hover {
    background: var(--muted-foreground);
  }
}
```

### Usage Guidance

- **Components reference utilities, not raw tokens.** Use `bg-card text-card-foreground` instead of inline `style={{ background: 'var(--card)' }}`. The `@theme inline` mapping makes this work.
- **Gain/loss are utilities.** Use `text-gain` for positive values, `text-loss` for negative, `text-neutral` for zero. Do not parameterize via inline color or className concatenation in business logic — use a `<DeltaValue value={n} />` component (in `@chrdfin/ui`) that picks the class.
- **Layer utilities are for nested panels.** When a panel inside a card needs visual distinction, use `bg-layer-02`. Do not stack four `bg-card` panels — that defeats the layering model.
- **Charts read tokens via JS.** Recharts and other chart libraries don't inherit CSS variables in stroke/fill props. Pass them via `getComputedStyle(document.documentElement).getPropertyValue('--chart-1')` or use a small `useThemeColors()` hook in `@chrdfin/charts`.
- **OKLCH conversion is supported.** If migrating to perceptually uniform color manipulation later, the hex values can be converted to OKLCH using `oklch()` wrappers without changing token names. Hex is used here for direct verifiability against carbondesignsystem.com.

---

## Typography

The platform uses **IBM Plex** to align with Carbon's typography system. Both Plex Sans and Plex Mono are open-source and available via Google Fonts. Plex Mono provides perfectly uniform digit widths, which is essential for the platform's tabular data emphasis.

### Font Stack

| Stack | Use |
|---|---|
| `IBM Plex Sans` | All UI text: labels, headers, body |
| `IBM Plex Mono` | All numeric data, tickers, code, command palette inputs |

Fallbacks defined in `--font-sans` and `--font-mono` above ensure graceful degradation.

### Type Scale

A compressed 6-step scale, optimized for desktop density. Avoid introducing intermediate sizes.

| Token | Size | Line height | Weight | Use |
|---|---|---|---|---|
| `text-xs` | 11px | 1.3 | 400 | Column labels, helper text, secondary metadata |
| `text-sm` | 12px | 1.4 | 400 | Default body text, table cells |
| `text-base` | 13px | 1.4 | 400 | Form inputs, sidebar nav labels |
| `text-md` | 14px | 1.4 | 500 | Section headers, primary metric values |
| `text-lg` | 16px | 1.3 | 500 | View titles, prominent values |
| `text-xl` | 18px | 1.2 | 600 | Display values (portfolio total, single hero number per view) |

Maximum size in any production view: 18px. There are no headings larger than 18px anywhere in the application.

### Numeric Conventions

- All numeric output uses Plex Mono (`font-mono`) by default. UI labels stay in Plex Sans.
- All cells with numeric values are right-aligned in tables.
- Currency: ISO format with thousands separators, e.g. `$847,293.14`. Never `$847k` in primary data; abbreviated forms are reserved for secondary metadata or chart axes.
- Percentages: two decimal places by default (`+0.34%`), one decimal for ranges and aggregates (`+34.7%`).
- Negative values: parenthesized OR signed-prefix, never both. Default is signed-prefix (`-$1,234.56`). Reserve parentheses `($1,234.56)` for accounting contexts (transaction logs).
- Sign discipline: always show `+` on positive values when they appear next to negative values for parity (gain/loss columns). Standalone positive values omit the `+`.

---

## Density & Spacing

Spacing follows a 4px grid. Components compose from this base.

| Token | Pixels | Use |
|---|---|---|
| `--space-1` | 4px | Inline gaps (icon to label) |
| `--space-2` | 8px | Tight component padding |
| `--space-3` | 12px | Default component padding |
| `--space-4` | 16px | Panel padding |
| `--space-6` | 24px | View-level padding (page edges) |
| `--space-8` | 32px | Section separation in vertical layouts |

Tailwind's default `0.25rem` step matches `--space-1` exactly, so `p-2`, `gap-3`, `mt-4` etc. map cleanly to this system.

### Component Heights

Standard heights for stacked control consistency:

| Element | Height |
|---|---|
| Sidebar collapsed width | 48px |
| Sidebar expanded width | 240px |
| Top header bar | 40px |
| Tab bar | 36px |
| Filter chip / pill | 28px |
| Default button | 32px |
| Compact button | 28px |
| Default input | 32px |
| Compact input (filter row) | 28px |
| Table row (default) | 32px |
| Table row (compact) | 28px |
| Table header | 32px |

---

## Platform Shell

The platform shell is the persistent application chrome rendered by `apps/desktop/src/routes/__root.tsx`. It wraps every route. There is exactly one shell; views never render their own sidebars or headers.

### Sidebar (Left)

Implemented in `apps/desktop/src/components/shell/sidebar.tsx`. Built on shadcn/ui's `<Sidebar>` primitive with the collapsible variant.

- **Width:** 240px expanded, 48px collapsed. Toggle persists across sessions (stored via Tauri `app_data_dir` config file, not localStorage — see CLAUDE.md).
- **Background:** `bg-sidebar` (matches `--card` in light, `--background` in dark for visual integration with the title bar).
- **Logo block:** 48px tall. "CHRD" wordmark in Plex Mono, weight 600, when collapsed; "chrdfin" lowercase in Plex Sans when expanded.
- **Sections:** four groups with subtle dividers (`border-sidebar-border`). When collapsed, dividers persist but section titles are hidden.

| Section | Items |
|---|---|
| **Analysis** | Backtesting, Monte Carlo, Optimizer |
| **Tracking** | Portfolio, Transactions, Watchlist |
| **Tools** | Calculators, Compare |
| **Market** | Screener, News, Calendar |

- **Nav item:** 32px height, icon (16px Lucide) + label. Active state is a 2px left border accent in `--primary` plus a subtle `bg-sidebar-accent` fill (5% blue tint). Hover state is `bg-sidebar-accent` only.
- **Feature flags:** Items whose feature flag in `@chrdfin/config` is `false` are not rendered. This is checked at component render time, not at route definition time, so toggling a flag in config and HMR-reloading immediately reflects in the sidebar.
- **Collapse toggle:** Bottom of sidebar, 32px button with chevron icon. Keyboard shortcut `Cmd/Ctrl+B`.

### Header (Top)

Implemented in `apps/desktop/src/components/shell/header.tsx`. 40px tall, full width minus sidebar.

- **Left — Breadcrumbs:** Reflects active route. E.g., `Analysis / Backtesting / Results`. Built on shadcn `<Breadcrumb>`. Last segment is `text-foreground`, prior segments `text-muted-foreground`. Separator is `/` in `text-muted-foreground`.
- **Center — Command palette trigger:** 320px-wide read-only-looking input. Placeholder: `Search tickers, portfolios, tools...`. Magnifier icon left, `⌘K` shortcut hint right. Click or `Cmd/Ctrl+K` opens the actual command palette dialog (shadcn `<Command>` inside a `<Dialog>`).
- **Right — Status indicators:**
  - 8px dot + label: `Market Open` (green dot, `--gain`), `Pre-Market` / `After-Market` (amber, `--warning`), `Closed` (gray, `--muted-foreground`). Logic in a `useMarketStatus()` hook driven by ET clock and NYSE trading hours.
  - ET clock: HH:MM in Plex Mono, `text-muted-foreground`. Updates every second via a single `setInterval` in the hook.
  - Theme toggle: 28px icon button, sun/moon. Toggles `.dark` class on `<html>`.

### Command Palette

Single global instance, mounted in `__root.tsx`. Built on shadcn `<CommandDialog>`.

- Sources: route navigation, ticker search (Tiingo via Tauri command, debounced 200ms), portfolio names, calculator presets, theme toggle.
- Result groups: `Navigate`, `Tickers`, `Portfolios`, `Tools`, `Actions`.
- Keyboard: `Cmd/Ctrl+K` to open, `Esc` to close, arrows to navigate, `Enter` to execute.

---

## Component Patterns

The following patterns are referenced by the three example views below and any future feature domain. Each lives in `@chrdfin/ui` once authored.

### Data Table

The single most important component in the application. Built on TanStack Table v8 wrapping shadcn `<Table>` primitives, exported as `<DataTable>` from `@chrdfin/ui`.

- Compact row height (32px default, 28px in dense mode).
- Header row sticky on scroll with `bg-card` and `border-b border-border`.
- Sortable columns: click header to cycle `asc → desc → none`. Active sort indicator is a 12px Lucide chevron in `text-foreground`; inactive columns show no indicator (not even a faded one — keeps headers clean).
- Zebra striping: `even:bg-muted/40` (light theme), `even:bg-muted/30` (dark). Subtle, not banded.
- Hover row: `hover:bg-accent` — non-dramatic, just a one-step surface lift.
- Selected row: `bg-accent` with a 2px left border in `--primary`.
- Numeric cells: right-aligned, Plex Mono, gain/loss tinting via `<DeltaValue>` component.
- No vertical column dividers. Horizontal row dividers are 1px `border-border` only on every nth row, or omitted entirely if zebra striping is enabled.
- Empty state: single line, `text-muted-foreground`, 12px, vertically centered. No illustrations.

### Metrics Strip

Horizontal row of key-value pairs separated by typographic spacing, no borders or cards. Used at the top of every analytical view.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  CAGR    Total Return   Max DD     Sharpe   Sortino  Vol      Best   Worst   │
│  7.82%   +312.4%        -32.1%     0.71     0.94     9.8%     +26.3% -22.1%  │
└──────────────────────────────────────────────────────────────────────────────┘
```

- 48–56px tall row. Background `bg-card` or `bg-background`.
- Each cell: `text-xs text-muted-foreground` label on top, `text-md` value below.
- Equal horizontal spacing via `flex` with `gap-6` or `grid` with auto columns.
- No borders between cells — separation is typographic.
- Gain/loss tinting on relevant values (Total Return, Max DD, Best/Worst Year, etc.).

### Chart Container

Wraps Recharts via `@chrdfin/charts`. All charts share consistent treatment:

- No grid lines by default; opt-in with a `gridLines` prop for charts that genuinely need them (equity curves do, sparklines never do).
- No axis lines visible at chart edges.
- Tick labels: Plex Mono, 10px, `text-muted-foreground`.
- Tooltips: shadcn `<HoverCard>`-styled, 11px content, `bg-popover border border-border`.
- Crosshair: 1px dashed `border-muted-foreground`, only on hover.
- Legend: only when more than one series. Inline above chart, not floating.
- Series colors pulled from `--chart-1` through `--chart-5` via `useThemeColors()` hook.
- Linked charts (e.g. equity curve + drawdown) share an x-axis and crosshair via Recharts' `<ComposedChart>` or stacked containers with synchronized state.

### Sparkline

Tiny inline charts in table rows. Built on Recharts `<LineChart>` with no axes, no tooltips, no grid.

- Width: 60px. Height: 20px.
- Stroke: `--gain` if first-to-last is positive, `--loss` if negative, 1.5px width.
- No fill, no markers.
- Implemented as `<Sparkline data={number[]} />` in `@chrdfin/charts`.

### Filter Bar

Horizontal row of filter controls used in screener and search views. 56px tall, `border-b border-border`.

- Each filter: shadcn `<Select>` or `<Popover>` with custom range slider. 28px height, pill-shaped (`rounded-md`).
- Active filters show value in `text-foreground`; unset filters show placeholder in `text-muted-foreground`.
- "Clear filters" text button (no border, `text-muted-foreground hover:text-foreground`) at the end of the active group.
- Result count on the far right: `142 results` in `text-muted-foreground`.

### Form Patterns

All forms use React Hook Form + Zod resolver per CLAUDE.md.

- Field height 32px. Label above field, 11px `text-muted-foreground`.
- Error message below field, 11px `text-destructive`. Field border switches to `border-destructive` on error.
- Submit button is `<Button>` default size, primary variant.
- Compound fields (e.g. ticker + weight) use `grid grid-cols-[1fr_80px] gap-2` for visual grouping.

---

## Reference Implementations

Three concrete views are specified below as visual targets. They will be built in their respective phases:

| View | Phase | Route |
|---|---|---|
| Portfolio Dashboard | Phase 5 | `/tracking/portfolio` |
| Backtesting Results | Phase 2–3 | `/analysis/backtest` |
| Market Screener | Phase 7 | `/market/screener` |

These specifications are the visual contract. Implementation logic, data flow, and Tauri command bindings are defined in the respective phase checklists.

### Reference View 1 — Portfolio Dashboard

The default home view. Dense multi-panel layout, no hero card.

**Top metrics strip (48px):**

A single row spanning the content width. Content from left to right:

- Portfolio name dropdown (e.g. `Core Holdings ▾`) — switches active portfolio
- `Total Value` label + `$847,293.14` value (Plex Mono, `text-lg`)
- `Day Change` label + `+$2,841.07 / +0.34%` (gain-tinted)
- `Total Return` label + `+34.7% / +$218,441.07` (gain-tinted) with `since inception` qualifier in `text-muted-foreground text-xs`

No cards, no borders between metrics — typographic spacing only.

**Main grid (below strip):**

```
┌─────────────────────────────────────────┬──────────────────────┐
│                                         │  Allocation Donut    │
│         Holdings DataTable              │                      │
│         (60% width, 12-15 rows)         │  Performance         │
│                                         │  Sparkline (12mo)    │
│                                         │                      │
│                                         │  Quick Stats Grid    │
│                                         │  (2x3 cells)         │
└─────────────────────────────────────────┴──────────────────────┘
```

- **Holdings table columns:** Ticker | Name (truncated, `text-muted-foreground`) | Shares | Avg Cost | Current Price | Market Value | Day Chg ($/% combined, gain/loss tinted) | Total Return % (tinted) | Weight %.
- **Allocation donut:** 160px diameter, no legend. Inline labels around the donut: `Tech 34%`, `Health 18%`, etc. Sectors color-coded from the chart palette.
- **Performance sparkline:** 12-month area chart. No grid, no axis labels. Start value annotated at left, end value at right. 1px line + 5%-opacity fill.
- **Quick stats:** 2×3 grid. Each cell: 11px label + 13px value. Cells: Beta, Sharpe, Div Yield, Day's Volume, 52w High/Low (visual indicator bar), Cash.

### Reference View 2 — Backtesting Results

Output of a completed backtest (e.g. 60% SPY / 40% AGG, 2005–2024).

**Top metrics strip:**

Eight equally-spaced cells across the full width: CAGR | Total Return | Max Drawdown | Sharpe | Sortino | Volatility | Best Year | Worst Year. Format per metrics-strip pattern above.

**Main chart area (full width, ~50% viewport height):**

A single composed chart with two stacked panels sharing an x-axis:

```
┌────────────────────────────────────────────────────────────┐
│ Equity Curve                                               │
│   Portfolio (──── chart-1 solid)                           │
│   S&P 500 benchmark (- - - chart-2 dashed)                 │
│   Y-axis right side, log scale toggle in chart header      │
├────────────────────────────────────────────────────────────┤
│ Drawdown                                                   │
│   Negative bars in --loss at 30% opacity                   │
│   Y-axis right side, % scale                               │
└────────────────────────────────────────────────────────────┘
   2005    2008    2011    2014    2017    2020    2023
```

Shared crosshair, shared tooltip listing all values for the hovered date.

**Bottom panels (50/50 split):**

Left — Annual Returns Table: columns Year | Portfolio | Benchmark | Excess. Years 2015–2024. Gain/loss tinting on return columns. Compact rows.

Right — Rolling Returns Distribution: small histogram of 12-month rolling returns. Bars in `--chart-1` at 60% opacity. Mean line marked at the median bucket. No grid.

### Reference View 3 — Market Screener

Stock screening interface. Filter-driven table.

**Filter bar (56px, top):**

Horizontal row of compact filter controls per the Filter Bar pattern:

| Filter | Type |
|---|---|
| Asset Type | Single-select (`Stocks` selected) |
| Sector | Multi-select (`All` shown when none selected) |
| Market Cap | Range slider with min/max numeric inputs (`$10B – $1T`) |
| Dividend Yield | Range slider (`> 2%`) |
| P/E Ratio | Range slider (`5 – 25`) |

`Clear filters` text button at the end of the chip group. `142 results` count on the far right.

**Results table (full remaining height):**

| Column | Notes |
|---|---|
| Ticker | Bold, Plex Mono |
| Company | Truncated, `text-muted-foreground` |
| Sector | Sector tag (small uppercase, `text-xs`) |
| Market Cap | Right-aligned, abbreviated (`$2.4T`) |
| Price | Right-aligned, Plex Mono |
| Day Change % | Right-aligned, gain/loss tinted |
| YTD % | Right-aligned, gain/loss tinted |
| Div Yield | Right-aligned |
| P/E | Right-aligned |
| Avg Volume | Right-aligned, abbreviated (`12.4M`) |
| Sparkline | 60px wide, last 30 days price action, gain/loss colored |

20+ rows visible. Sortable by every column. Click ticker to navigate to `/market/ticker/$symbol`.

---

## Phase 0 Implementation Scope

Phase 0 implements **only** the platform shell and design system foundation. The three reference views above are not part of Phase 0.

Phase 0 deliverables for UI:

1. **`apps/desktop/src/globals.css`** — exactly as specified above. Includes Tailwind v4 imports, both theme blocks, `@theme inline` mapping, and base styles.
2. **`apps/desktop/src/components/shell/sidebar.tsx`** — full implementation per spec, with all four sections and all 13 nav items, gated by feature flags from `@chrdfin/config`. Routes navigate via TanStack Router.
3. **`apps/desktop/src/components/shell/header.tsx`** — breadcrumbs, command palette trigger (opens an empty palette in Phase 0, populated in later phases), market status, ET clock, theme toggle.
4. **`apps/desktop/src/components/shell/command-palette.tsx`** — empty `<CommandDialog>` with placeholder "No results" message. Wired to `Cmd/Ctrl+K`.
5. **`apps/desktop/src/components/providers/theme-provider.tsx`** — manages `.dark` class on `<html>`. Default to dark theme on first launch. Persist preference via Tauri command (DuckDB `settings` table per schema reference), not localStorage.
6. **Route placeholders for all 13 feature domains** — each renders a centered "Coming in Phase N" message with the phase number per the table in CLAUDE.md.
7. **`packages/@chrdfin/ui`** — initialize with shadcn/ui CLI in Tailwind v4 mode, copy `globals.css` content, and stub-export the components needed for Phase 0 (`Button`, `Sidebar`, `Breadcrumb`, `Command`, `CommandDialog`, `Dialog`).
8. **Font loading** — install `@fontsource/ibm-plex-sans` and `@fontsource/ibm-plex-mono`, import in `main.tsx`. No CDN font loading (offline-first desktop app).

What Phase 0 does NOT implement:

- Any data tables (no holdings, no screener results)
- Any charts (Recharts not installed yet)
- Any forms (Zod schemas exist in `@chrdfin/types` but no form pages)
- Any Tauri commands beyond `health_check` per the existing checklist
- Settings UI (persisted via Tauri commands, no UI page in Phase 0)

---

## What NOT to Do

These are non-negotiable. Any deviation requires explicit approval and a documented exception in this file.

- Do NOT use `localStorage` or `sessionStorage` for theme persistence, sidebar state, or any other state. All persistent UI state goes through Tauri commands and DuckDB. (CLAUDE.md rule.)
- Do NOT introduce additional color palettes outside the Carbon-derived tokens defined here. If a future need arises, add the token to `:root` and `.dark`, map it in `@theme inline`, and document it in this file.
- Do NOT use Tailwind's built-in palette utilities (`bg-blue-500`, `text-red-600`, etc.) anywhere. Always use semantic tokens (`bg-primary`, `text-loss`).
- Do NOT use raw color values (hex, rgb, hsl) inline in JSX or component styles. The only place hex values appear is in `globals.css`.
- Do NOT add CSS files beyond `globals.css`. Use Tailwind utility classes. (CLAUDE.md rule.)
- Do NOT use rounded-2xl, rounded-3xl, or any radius beyond `--radius-lg` (8px). Cards are 4px, controls are 2px.
- Do NOT use drop shadows on internal panels. Shadows are reserved for popovers, dropdowns, and dialogs (where they appear via shadcn defaults).
- Do NOT create gradient backgrounds anywhere.
- Do NOT use emoji in production UI strings or any user-facing copy.
- Do NOT use illustrations, mascots, or decorative imagery anywhere.
- Do NOT add "Welcome back" greeting banners, onboarding modals, or first-run tutorials. The user is the developer.
- Do NOT use color as the primary differentiator for non-financial state (e.g. don't make a button red just because it's important — use weight and position). Color is for financial signal (gain/loss/warning) and the single primary accent.
- Do NOT add status badges to non-financial elements. The market status pill in the header is the only persistent status indicator in the platform shell.
- Do NOT exceed 18px for any text size in production views.
- Do NOT use serif fonts.
- Do NOT use `font-variant-numeric: oldstyle-nums` or `proportional-nums` anywhere — tabular nums are global.
- Do NOT install icon libraries beyond `lucide-react`. All iconography is Lucide.
- Do NOT install chart libraries beyond Recharts. Charts that Recharts can't handle (e.g. WebGL-accelerated tick charts, if ever needed) get an exception in this file first.
- Do NOT install component libraries beyond shadcn/ui + Radix primitives. Any third component library requires architectural review.

---

## References

- **Carbon Design System color tokens:** <https://carbondesignsystem.com/elements/color/overview/>
- **Carbon color palette source:** <https://github.com/carbon-design-system/carbon/blob/main/packages/colors/docs/sass.md>
- **shadcn/ui Tailwind v4 guide:** <https://ui.shadcn.com/docs/tailwind-v4>
- **shadcn/ui theming:** <https://ui.shadcn.com/docs/theming>
- **Tailwind CSS v4 theme directive:** <https://tailwindcss.com/docs/theme>
- **IBM Plex on Google Fonts:** <https://fonts.google.com/specimen/IBM+Plex+Sans>
- **TanStack Router:** <https://tanstack.com/router>
- **TanStack Table:** <https://tanstack.com/table>
- **Recharts:** <https://recharts.org>

---

## Document Maintenance

This document is the source of truth for chrdfin's UI. Any change to color tokens, typography, density rules, or component patterns must update this file in the same commit. Reference views may be expanded as more phases come online; do not delete prior reference views even after they ship.

When in doubt, the rule is: **density, restraint, and consistency over novelty.**
