import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/tools/compare")({
  component: ComparePage,
});

function ComparePage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={10}
      feature="Comparison Tool"
      description="Side-by-side comparison of portfolios, backtests, and strategies."
    />
  );
}
