import { ScreenerSearchSchema } from "@chrdfin/types";
import { createFileRoute, redirect } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/market/options/$symbol")({
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
  component: OptionsChainPage,
});

function OptionsChainPage(): JSX.Element {
  const { symbol } = Route.useParams();
  return (
    <PhasePlaceholder
      phase={7}
      feature={`Options · ${symbol}`}
      description="Options chain with implied volatility, Greeks, and payoff visualization."
    />
  );
}
