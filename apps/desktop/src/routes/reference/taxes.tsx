import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/reference/taxes")({
  component: TaxesReferencePage,
});

function TaxesReferencePage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={12}
      feature="Reference · Taxes"
      description="Guides on capital gains (short vs long), wash sales, qualified vs ordinary dividends, tax-loss harvesting, lot selection methods (FIFO, LIFO, specific ID), AMT, and state-specific considerations."
    />
  );
}
