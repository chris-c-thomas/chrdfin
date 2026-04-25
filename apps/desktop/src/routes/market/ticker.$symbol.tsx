import { ScreenerSearchSchema } from "@chrdfin/types";
import { createFileRoute, redirect } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/market/ticker/$symbol")({
  parseParams: ({ symbol }) => {
    const upper = symbol.toUpperCase();
    if (!/^[A-Z0-9.-]{1,10}$/.test(upper)) {
      throw redirect({
        to: "/market/screener",
        search: ScreenerSearchSchema.parse({}),
      });
    }
    return { symbol: upper };
  },
  stringifyParams: ({ symbol }) => ({ symbol: symbol.toUpperCase() }),
  component: TickerDetailPage,
});

function TickerDetailPage(): JSX.Element {
  const { symbol } = Route.useParams();
  return (
    <PhasePlaceholder
      phase={7}
      feature={`Ticker · ${symbol}`}
      description="Real-time quote, price chart, fundamentals, and news for a single ticker."
    />
  );
}
