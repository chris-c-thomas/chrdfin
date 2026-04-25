import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/reference/")({
  component: ReferenceIndexPage,
});

function ReferenceIndexPage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={12}
      feature="Reference Library"
      description="A bundled knowledge base of guides covering Stocks, Options, Retirement Accounts, Estate Planning, Taxes, and other financial topics. Picks up new sections as the platform matures."
    />
  );
}
