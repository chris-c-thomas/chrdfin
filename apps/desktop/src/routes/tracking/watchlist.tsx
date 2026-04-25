import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/tracking/watchlist")({
  component: WatchlistPage,
});

function WatchlistPage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={5}
      feature="Watchlists"
      description="Configurable real-time quote tables grouped into named lists."
    />
  );
}
