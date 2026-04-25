import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/analysis/allocation-optimizer")({
  component: AllocationOptimizerPage,
});

function AllocationOptimizerPage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={9}
      feature="Allocation Optimizer"
      description="Optimize a portfolio's target allocations and rebalancing trades against backtest history, tax constraints, and a chosen rebalancing strategy. Pairs with the Optimizer (efficient frontier, mean-variance) and the Backtest engine."
    />
  );
}
