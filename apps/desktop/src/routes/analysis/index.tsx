import { featureFlags } from "@chrdfin/config";
import { BacktestSearchSchema, MonteCarloSearchSchema } from "@chrdfin/types";
import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/analysis/")({
  beforeLoad: () => {
    if (featureFlags.backtest) {
      throw redirect({
        to: "/analysis/backtest",
        search: BacktestSearchSchema.parse({}),
      });
    }
    if (featureFlags.monteCarlo) {
      throw redirect({
        to: "/analysis/monte-carlo",
        search: MonteCarloSearchSchema.parse({}),
      });
    }
    if (featureFlags.optimizer) throw redirect({ to: "/analysis/optimizer" });
    throw redirect({ to: "/" });
  },
});
