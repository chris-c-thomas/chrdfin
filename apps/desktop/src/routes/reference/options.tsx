import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/reference/options")({
  component: OptionsReferencePage,
});

function OptionsReferencePage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={12}
      feature="Reference · Options"
      description="Guides on option contracts — calls, puts, multi-leg strategies, Greeks (delta/gamma/theta/vega/rho), implied volatility, assignment, expiration mechanics, and tax treatment."
    />
  );
}
