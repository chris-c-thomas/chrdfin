import { cn } from "@chrdfin/ui";
import { type JSX } from "react";

import { useMarketStatus, type MarketStatus } from "@/hooks/use-market-status.js";

const STATUS_LABEL: Record<MarketStatus, string> = {
  open: "Market Open",
  "pre-market": "Pre-Market",
  "after-market": "After-Market",
  closed: "Market Closed",
  weekend: "Weekend",
  holiday: "Holiday",
};

const STATUS_DOT_CLASS: Record<MarketStatus, string> = {
  open: "bg-gain",
  "pre-market": "bg-warning",
  "after-market": "bg-warning",
  closed: "bg-muted-foreground",
  weekend: "bg-muted-foreground",
  holiday: "bg-muted-foreground",
};

export function MarketStatusIndicator(): JSX.Element {
  const { status, etTime } = useMarketStatus();
  return (
    <div className="flex items-center gap-3 text-xs">
      <div className="flex items-center gap-2">
        <span
          className={cn(
            "size-2 rounded-full",
            STATUS_DOT_CLASS[status],
            status === "open" && "animate-pulse",
          )}
          aria-hidden="true"
        />
        <span className="text-muted-foreground">{STATUS_LABEL[status]}</span>
      </div>
      <span className="font-mono tabular-nums text-muted-foreground">{etTime} ET</span>
    </div>
  );
}
