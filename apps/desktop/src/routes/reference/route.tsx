import { featureFlags } from "@chrdfin/config";
import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { type JSX } from "react";

import { SectionErrorBoundary } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/reference")({
  beforeLoad: () => {
    if (!featureFlags.reference) throw redirect({ to: "/" });
  },
  component: ReferenceLayout,
  errorComponent: SectionErrorBoundary,
});

function ReferenceLayout(): JSX.Element {
  return <Outlet />;
}
