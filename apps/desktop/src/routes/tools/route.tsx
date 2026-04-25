import { featureFlags } from "@chrdfin/config";
import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { type JSX } from "react";

import { SectionErrorBoundary } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/tools")({
  beforeLoad: () => {
    if (!featureFlags.calculators && !featureFlags.backtest) throw redirect({ to: "/" });
  },
  component: ToolsLayout,
  errorComponent: SectionErrorBoundary,
});

function ToolsLayout(): JSX.Element {
  return <Outlet />;
}
