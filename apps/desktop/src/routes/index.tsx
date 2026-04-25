import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@chrdfin/ui";
import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import {
  Briefcase,
  Calendar,
  LayoutDashboard,
  LineChart,
  Newspaper,
  Wallet,
  type LucideIcon,
} from "lucide-react";
import { useEffect, useState, type JSX } from "react";

interface HealthCheckResponse {
  status: string;
  db_initialized: boolean;
  version: string;
  core_version: string;
}

interface PlannedWidget {
  icon: LucideIcon;
  label: string;
  description: string;
}

/**
 * The dashboard is the application's home screen. In its eventual form it
 * will be a customizable grid of widgets pulling from every feature
 * domain — markets, portfolio, backtests, accounts, news, calendar.
 * Until the widget framework lands the page renders an intent placeholder
 * plus the Phase 0 IPC health check.
 */
const PLANNED_WIDGETS: readonly PlannedWidget[] = [
  {
    icon: Briefcase,
    label: "Portfolio Summary",
    description:
      "Total value, day change, allocation breakdown, and top movers across tracked portfolios.",
  },
  {
    icon: LineChart,
    label: "Market Overview",
    description: "Indices, sector performance, and watchlist quotes during market hours.",
  },
  {
    icon: LayoutDashboard,
    label: "Recent Backtests",
    description: "Last-run portfolio simulations with CAGR, max drawdown, and Sharpe at a glance.",
  },
  {
    icon: Wallet,
    label: "Accounts",
    description: "Aggregated balances and holdings across linked accounts and tracked portfolios.",
  },
  {
    icon: Newspaper,
    label: "News",
    description: "Top headlines and ticker-tagged stories from Tiingo News and curated RSS.",
  },
  {
    icon: Calendar,
    label: "Earnings & Calendar",
    description: "Upcoming earnings releases and economic events for tracked tickers.",
  },
];

export const Route = createFileRoute("/")({
  component: DashboardPage,
});

function DashboardPage(): JSX.Element {
  const [health, setHealth] = useState<string>("checking…");

  useEffect(() => {
    invoke<HealthCheckResponse>("health_check")
      .then((result) => setHealth(JSON.stringify(result, null, 2)))
      .catch((err: unknown) => {
        const message = err instanceof Error ? err.message : String(err);
        setHealth(`Tauri command failed: ${message}`);
      });
  }, []);

  return (
    <div className="flex flex-col gap-6 p-6">
      <div className="flex flex-col gap-1">
        <h1 className="flex items-center gap-2 text-lg font-medium">
          <LayoutDashboard className="size-5" />
          Dashboard
        </h1>
        <p className="text-muted-foreground max-w-3xl text-sm">
          Your starting view. This page will become a customizable grid of widgets giving you an
          at-a-glance overview of the markets, your portfolio, recent backtests, accounts, news, and
          upcoming events. Layout, widget selection, and refresh cadence will all be
          user-configurable.
        </p>
      </div>

      <section className="flex flex-col gap-3">
        <div className="text-muted-foreground text-xs font-medium uppercase tracking-wider">
          Planned widgets
        </div>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {PLANNED_WIDGETS.map((widget) => {
            const Icon = widget.icon;
            return (
              <Card key={widget.label} className="p-4">
                <div className="flex flex-col gap-2">
                  <div className="flex items-center gap-2">
                    <Icon className="text-muted-foreground size-4" />
                    <span className="text-sm font-medium">{widget.label}</span>
                  </div>
                  <p className="text-muted-foreground text-xs">{widget.description}</p>
                </div>
              </Card>
            );
          })}
        </div>
      </section>

      <Card className="max-w-2xl">
        <CardHeader>
          <CardTitle>Phase 0 health check</CardTitle>
          <CardDescription>
            Round-trip from React → Tauri command → chrdfin-core → DuckDB. Kept visible during Phase
            1 wiring; will be removed once the widget grid lands.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <pre className="bg-muted overflow-x-auto rounded-sm p-3 font-mono text-xs">{health}</pre>
        </CardContent>
      </Card>
    </div>
  );
}
