import { featureFlags } from "@chrdfin/config";
import { NewsSearchSchema, ScreenerSearchSchema } from "@chrdfin/types";
import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/market/")({
  beforeLoad: () => {
    if (featureFlags.marketData) {
      throw redirect({
        to: "/market/screener",
        search: ScreenerSearchSchema.parse({}),
      });
    }
    if (featureFlags.news) {
      throw redirect({
        to: "/market/news",
        search: NewsSearchSchema.parse({}),
      });
    }
    throw redirect({ to: "/" });
  },
});
