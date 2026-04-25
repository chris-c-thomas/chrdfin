import { NewsSearchSchema, zodValidator } from "@chrdfin/types";
import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/market/news")({
  validateSearch: zodValidator(NewsSearchSchema),
  component: NewsPage,
});

function NewsPage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={8}
      feature="News Feed"
      description="Aggregated financial news from Tiingo News and curated RSS feeds."
    />
  );
}
