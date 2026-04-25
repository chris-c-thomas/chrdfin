import { featureFlags, type FeatureFlag } from "@chrdfin/config";
import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { type JSX } from "react";

import { SectionErrorBoundary } from "@/routes/-shared/route-states.js";

const SECTION_FLAGS: ReadonlyArray<FeatureFlag> = ["backtest", "monteCarlo", "optimizer"];

export const Route = createFileRoute("/analysis")({
  beforeLoad: () => {
    if (!SECTION_FLAGS.some((f) => featureFlags[f])) throw redirect({ to: "/" });
  },
  component: AnalysisLayout,
  errorComponent: SectionErrorBoundary,
});

function AnalysisLayout(): JSX.Element {
  return <Outlet />;
}
