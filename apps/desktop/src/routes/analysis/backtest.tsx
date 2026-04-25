import { BacktestSearchSchema, zodValidator } from "@chrdfin/types";
import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/analysis/backtest")({
  validateSearch: zodValidator(BacktestSearchSchema),
  component: BacktestPage,
});

function BacktestPage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={3}
      feature="Portfolio Backtesting"
      description="Historical portfolio simulation with configurable rebalancing, dividends, and benchmark comparison."
    />
  );
}
