import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/reference/retirement")({
  component: RetirementReferencePage,
});

function RetirementReferencePage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={12}
      feature="Reference · Retirement Accounts"
      description="Guides on tax-advantaged accounts — Traditional IRA, Roth IRA, 401(k), 403(b), 457, HSA, SEP, SIMPLE, Solo 401(k). Contribution limits, RMDs, conversions, withdrawals."
    />
  );
}
