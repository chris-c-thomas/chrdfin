import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/reference/estate")({
  component: EstateReferencePage,
});

function EstateReferencePage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={12}
      feature="Reference · Estate Planning"
      description="Guides on wills, trusts, beneficiary designations, gift and estate tax exemptions, step-up in basis, and how to model multi-generational portfolios."
    />
  );
}
