import { MonteCarloSearchSchema, zodValidator } from "@chrdfin/types";
import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/analysis/monte-carlo")({
  validateSearch: zodValidator(MonteCarloSearchSchema),
  component: MonteCarloPage,
});

function MonteCarloPage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={4}
      feature="Monte Carlo Simulation"
      description="Forward-looking probabilistic analysis via parametric, historical bootstrap, and block bootstrap methods."
    />
  );
}
