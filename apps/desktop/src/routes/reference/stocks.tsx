import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/reference/stocks")({
  component: StocksReferencePage,
});

function StocksReferencePage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={12}
      feature="Reference · Stocks"
      description="Bundled guides on equity instruments — order types, corporate actions, dividends, splits, ADRs, ETFs, mutual funds, and how chrdfin models each."
    />
  );
}
