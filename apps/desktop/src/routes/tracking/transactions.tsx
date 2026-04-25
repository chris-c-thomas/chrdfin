import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/tracking/transactions")({
  component: TransactionsPage,
});

function TransactionsPage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={5}
      feature="Transactions"
      description="Full audit trail of buys, sells, dividends, splits, and transfers."
    />
  );
}
