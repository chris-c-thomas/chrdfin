import { featureFlags } from "@chrdfin/config";
import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/tools/")({
  beforeLoad: () => {
    if (featureFlags.calculators) throw redirect({ to: "/tools/calculators" });
    if (featureFlags.backtest) throw redirect({ to: "/tools/compare" });
    throw redirect({ to: "/" });
  },
});
