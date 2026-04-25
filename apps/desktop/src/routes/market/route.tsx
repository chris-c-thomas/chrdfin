import { featureFlags } from "@chrdfin/config";
import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { type JSX } from "react";

import { SectionErrorBoundary } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/market")({
  beforeLoad: () => {
    if (!featureFlags.marketData && !featureFlags.news) throw redirect({ to: "/" });
  },
  component: MarketLayout,
  errorComponent: SectionErrorBoundary,
});

function MarketLayout(): JSX.Element {
  return <Outlet />;
}
