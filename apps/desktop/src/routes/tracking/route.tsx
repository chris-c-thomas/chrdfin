import { featureFlags } from "@chrdfin/config";
import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { type JSX } from "react";

import { SectionErrorBoundary } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/tracking")({
  beforeLoad: () => {
    if (!featureFlags.tracker) throw redirect({ to: "/" });
  },
  component: TrackingLayout,
  errorComponent: SectionErrorBoundary,
});

function TrackingLayout(): JSX.Element {
  return <Outlet />;
}
