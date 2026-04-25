import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/analysis/optimizer")({
  component: OptimizerPage,
});

function OptimizerPage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={9}
      feature="Portfolio Optimizer"
      description="Mean-variance, efficient frontier, risk parity, and Black-Litterman allocation tools."
    />
  );
}
