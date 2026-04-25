import { createRouter } from "@tanstack/react-router";

import { RouteErrorBoundary, RouteNotFound, RoutePending } from "./routes/-shared/route-states.js";
import { routeTree } from "./routeTree.gen.js";

export const router = createRouter({
  routeTree,
  defaultPreload: "intent",
  defaultPreloadStaleTime: 0,
  defaultPendingComponent: RoutePending,
  defaultErrorComponent: RouteErrorBoundary,
  defaultNotFoundComponent: RouteNotFound,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
