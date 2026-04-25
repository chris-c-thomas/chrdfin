import { featureFlags, type FeatureFlag } from "@chrdfin/config";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  cn,
  useSidebar,
} from "@chrdfin/ui";
import { Link, useLocation } from "@tanstack/react-router";
import {
  Activity,
  BarChart2,
  BookOpen,
  Briefcase,
  Calculator,
  Calendar,
  ChevronLeft,
  FileText,
  GitCompare,
  Layers,
  LayoutDashboard,
  LineChart,
  List,
  Newspaper,
  PiggyBank,
  Receipt,
  Scale,
  Sigma,
  Star,
  TrendingUp,
  type LucideIcon,
} from "lucide-react";
import { type JSX } from "react";

interface NavItem {
  label: string;
  to: string;
  icon: LucideIcon;
  flag: FeatureFlag;
}

interface NavSection {
  label: string;
  items: NavItem[];
}

/**
 * Section order: Tracking → Analysis & Tools → Market → Reference.
 *
 * Plural labels (`Portfolios`, `Watchlists`, `Screeners`, `Calendars`) signal
 * that each domain supports multiple saved instances per user. The list/detail
 * UX and per-instance dropdowns are deferred to the respective phases — see
 * `docs/technical-blueprint.md` § Multi-instance domains.
 */
const SECTIONS: NavSection[] = [
  {
    label: "Tracking",
    items: [
      { label: "Portfolios", to: "/tracking/portfolio", icon: Briefcase, flag: "tracker" },
      { label: "Transactions", to: "/tracking/transactions", icon: List, flag: "tracker" },
      { label: "Watchlists", to: "/tracking/watchlist", icon: Star, flag: "tracker" },
    ],
  },
  {
    label: "Analysis & Tools",
    items: [
      { label: "Backtesting", to: "/analysis/backtest", icon: LineChart, flag: "backtest" },
      { label: "Monte Carlo", to: "/analysis/monte-carlo", icon: Sigma, flag: "monteCarlo" },
      { label: "Optimizer", to: "/analysis/optimizer", icon: Activity, flag: "optimizer" },
      {
        label: "Allocation Optimizer",
        to: "/analysis/allocation-optimizer",
        icon: Scale,
        flag: "allocationOptimizer",
      },
      { label: "Calculators", to: "/tools/calculators", icon: Calculator, flag: "calculators" },
      { label: "Compare", to: "/tools/compare", icon: GitCompare, flag: "backtest" },
    ],
  },
  {
    label: "Market",
    items: [
      { label: "Screeners", to: "/market/screener", icon: Layers, flag: "marketData" },
      { label: "News", to: "/market/news", icon: Newspaper, flag: "news" },
      { label: "Calendars", to: "/market/calendar", icon: Calendar, flag: "news" },
    ],
  },
  {
    label: "Reference",
    items: [
      { label: "Stocks", to: "/reference/stocks", icon: TrendingUp, flag: "reference" },
      { label: "Options", to: "/reference/options", icon: BarChart2, flag: "reference" },
      {
        label: "Retirement Accounts",
        to: "/reference/retirement",
        icon: PiggyBank,
        flag: "reference",
      },
      {
        label: "Estate Planning",
        to: "/reference/estate",
        icon: FileText,
        flag: "reference",
      },
      { label: "Taxes", to: "/reference/taxes", icon: Receipt, flag: "reference" },
      { label: "Guides", to: "/reference", icon: BookOpen, flag: "reference" },
    ],
  },
];

export function AppSidebar(): JSX.Element {
  const location = useLocation();
  const { open, toggle } = useSidebar();
  const isDashboardActive = location.pathname === "/";

  return (
    <Sidebar>
      <SidebarHeader>
        <span className={cn("font-mono font-semibold", open ? "text-base" : "text-sm")}>
          {open ? "chrdfin" : "CHRD"}
        </span>
      </SidebarHeader>
      <SidebarContent>
        {/*
         * Dashboard is rendered above the section groups as a top-level entry
         * point. Always visible (no feature flag) and uses an exact pathname
         * match so it is only highlighted on `/`.
         */}
        <SidebarGroup>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton asChild isActive={isDashboardActive}>
                <Link to="/" aria-label="Dashboard">
                  <LayoutDashboard className="size-4 shrink-0" />
                  {open && <span className="truncate">Dashboard</span>}
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroup>

        {SECTIONS.map((section) => {
          const visible = section.items.filter((item) => featureFlags[item.flag]);
          if (visible.length === 0) return null;
          return (
            <SidebarGroup key={section.label}>
              <SidebarGroupLabel>{section.label}</SidebarGroupLabel>
              <SidebarMenu>
                {visible.map((item) => {
                  const Icon = item.icon;
                  const isActive =
                    item.to === "/reference"
                      ? location.pathname === "/reference"
                      : location.pathname.startsWith(item.to);
                  return (
                    <SidebarMenuItem key={item.to}>
                      <SidebarMenuButton asChild isActive={isActive}>
                        <Link to={item.to} aria-label={item.label}>
                          <Icon className="size-4 shrink-0" />
                          {open && <span className="truncate">{item.label}</span>}
                        </Link>
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                  );
                })}
              </SidebarMenu>
            </SidebarGroup>
          );
        })}
      </SidebarContent>
      <SidebarFooter>
        <button
          type="button"
          onClick={toggle}
          aria-label={open ? "Collapse sidebar" : "Expand sidebar"}
          className={cn(
            "text-muted-foreground flex h-8 w-full items-center gap-2 rounded-sm px-2 text-xs transition-colors",
            "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
            "focus-visible:ring-sidebar-ring focus-visible:outline-none focus-visible:ring-2",
          )}
        >
          <ChevronLeft
            className={cn("size-4 shrink-0 transition-transform", !open && "rotate-180")}
          />
          {open && <span>Collapse</span>}
        </button>
      </SidebarFooter>
    </Sidebar>
  );
}
