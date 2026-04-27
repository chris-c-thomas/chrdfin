import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
  cn,
} from "@chrdfin/ui";
import { Link, useLocation } from "@tanstack/react-router";
import { Moon, Search, Sun } from "lucide-react";
import { Fragment, type JSX } from "react";

import { MarketStatusIndicator } from "./market-status-indicator.js";
import { SyncStatusBadge } from "./sync-status-badge.js";

import { useTheme } from "@/components/providers/theme-provider.js";

interface BreadcrumbSegment {
  label: string;
  href?: string;
}

/**
 * Translate a `/foo/bar` pathname into capitalized breadcrumb segments,
 * with a home root and the final segment marked as the current page.
 */
function pathToBreadcrumbs(pathname: string): BreadcrumbSegment[] {
  const parts = pathname.split("/").filter(Boolean);
  if (parts.length === 0) return [{ label: "Dashboard" }];

  const segments: BreadcrumbSegment[] = [{ label: "chrdfin", href: "/" }];
  let acc = "";
  parts.forEach((part, i) => {
    acc += `/${part}`;
    const label = part
      .split("-")
      .map((s) => s.charAt(0).toUpperCase() + s.slice(1))
      .join(" ");
    segments.push(i === parts.length - 1 ? { label } : { label, href: acc });
  });
  return segments;
}

export interface AppHeaderProps {
  onOpenCommandPalette: () => void;
}

export function AppHeader({ onOpenCommandPalette }: AppHeaderProps): JSX.Element {
  const { pathname } = useLocation();
  const segments = pathToBreadcrumbs(pathname);
  const { resolvedTheme, setTheme } = useTheme();

  return (
    <header className="border-border bg-background flex h-14 shrink-0 items-center gap-4 border-b px-4">
      <Breadcrumb className="flex-1">
        <BreadcrumbList>
          {segments.map((segment, i) => {
            const isLast = i === segments.length - 1;
            return (
              <Fragment key={`${segment.label}-${i}`}>
                <BreadcrumbItem>
                  {isLast || !segment.href ? (
                    <BreadcrumbPage>{segment.label}</BreadcrumbPage>
                  ) : (
                    <Link to={segment.href} className="hover:text-foreground transition-colors">
                      {segment.label}
                    </Link>
                  )}
                </BreadcrumbItem>
                {!isLast && <BreadcrumbSeparator />}
              </Fragment>
            );
          })}
        </BreadcrumbList>
      </Breadcrumb>

      <button
        type="button"
        onClick={onOpenCommandPalette}
        className={cn(
          "border-border bg-card text-muted-foreground flex h-7 w-80 items-center gap-2 rounded-sm border px-2 text-xs transition-colors",
          "hover:border-muted-foreground/40 hover:text-foreground",
          "focus-visible:ring-ring focus-visible:outline-none focus-visible:ring-2",
        )}
      >
        <Search className="size-3.5" />
        <span className="flex-1 text-left">Search tickers, portfolios, tools…</span>
        <kbd className="text-muted-foreground font-mono text-xs">⌘K</kbd>
      </button>

      <SyncStatusBadge />

      <MarketStatusIndicator />

      <button
        type="button"
        onClick={() => setTheme(resolvedTheme === "dark" ? "light" : "dark")}
        aria-label={`Switch to ${resolvedTheme === "dark" ? "light" : "dark"} theme`}
        className={cn(
          "text-muted-foreground flex size-7 items-center justify-center rounded-sm transition-colors",
          "hover:bg-accent hover:text-foreground",
          "focus-visible:ring-ring focus-visible:outline-none focus-visible:ring-2",
        )}
      >
        {resolvedTheme === "dark" ? <Sun className="size-4" /> : <Moon className="size-4" />}
      </button>
    </header>
  );
}
