import { PortfolioSearchSchema, zodValidator } from "@chrdfin/types";
import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/tracking/portfolio")({
  validateSearch: zodValidator(PortfolioSearchSchema),
  component: PortfolioPage,
});

function PortfolioPage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={5}
      feature="Portfolio Tracker"
      description="Holdings, real-time P&L, allocation views, and watchlists driven by manual entry."
    />
  );
}
