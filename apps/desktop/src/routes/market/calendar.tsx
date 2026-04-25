import { CalendarSearchSchema, zodValidator } from "@chrdfin/types";
import { createFileRoute } from "@tanstack/react-router";
import { type JSX } from "react";

import { PhasePlaceholder } from "@/routes/-shared/route-states.js";

export const Route = createFileRoute("/market/calendar")({
  validateSearch: zodValidator(CalendarSearchSchema),
  component: CalendarPage,
});

function CalendarPage(): JSX.Element {
  return (
    <PhasePlaceholder
      phase={8}
      feature="Calendar"
      description="Earnings, economic events, IPOs, and stock splits."
    />
  );
}
