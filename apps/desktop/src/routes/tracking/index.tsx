import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/tracking/")({
  beforeLoad: () => {
    throw redirect({ to: "/tracking/portfolio" });
  },
});
