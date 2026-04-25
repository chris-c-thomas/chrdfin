import { ScreenerSearchSchema, zodValidator } from "@chrdfin/types";
import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/market/screener")({
  validateSearch: zodValidator(ScreenerSearchSchema),
  component: ScreenerPage,
});

function ScreenerPage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={7}
      feature="Stock Screener"
      description="Filter and sort thousands of equities and ETFs by sector, market cap, yield, P/E, and more."
    />
  );
}
