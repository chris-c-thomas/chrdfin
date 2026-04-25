import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/tools/calculators")({
  component: CalculatorsPage,
});

function CalculatorsPage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={6}
      feature="Financial Calculators"
      description="Compound growth, retirement, withdrawal, options payoff, tax-loss, position sizing."
    />
  );
}
