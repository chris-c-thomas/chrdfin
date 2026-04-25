import { Link, type ErrorComponentProps } from "@tanstack/react-router";
import { type JSX } from "react";

export function RoutePending(): JSX.Element {
  return (
    <div className="text-muted-foreground flex h-full items-center justify-center text-xs">
      Loading…
    </div>
  );
}

export function RouteErrorBoundary({ error, reset }: ErrorComponentProps): JSX.Element {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-6">
      <div className="text-destructive text-lg font-medium">Something went wrong</div>
      <pre className="text-muted-foreground max-w-2xl whitespace-pre-wrap font-mono text-xs">
        {error instanceof Error ? error.message : String(error)}
      </pre>
      <div className="flex gap-2 pt-2">
        <button
          type="button"
          onClick={reset}
          className="border-border hover:bg-accent border px-3 py-1 text-xs"
        >
          Reset
        </button>
        <Link to="/" className="border-border hover:bg-accent border px-3 py-1 text-xs">
          Home
        </Link>
      </div>
    </div>
  );
}

export function SectionErrorBoundary({ error, reset }: ErrorComponentProps): JSX.Element {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-6">
      <div className="text-destructive text-lg font-medium">Section unavailable</div>
      <pre className="text-muted-foreground max-w-2xl whitespace-pre-wrap font-mono text-xs">
        {error instanceof Error ? error.message : String(error)}
      </pre>
      <button
        type="button"
        onClick={reset}
        className="border-border hover:bg-accent border px-3 py-1 text-xs"
      >
        Retry
      </button>
    </div>
  );
}

export function RouteNotFound(): JSX.Element {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-6">
      <div className="text-lg font-medium">Route not found</div>
      <Link to="/" className="border-border hover:bg-accent border px-3 py-1 text-xs">
        Home
      </Link>
    </div>
  );
}

export interface PhasePlaceholderProps {
  phase: number;
  feature: string;
  description?: string;
}

export function PhasePlaceholder({
  phase,
  feature,
  description,
}: PhasePlaceholderProps): JSX.Element {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 p-6 text-center">
      <div className="text-lg font-medium">{feature}</div>
      {description && <div className="text-muted-foreground max-w-md text-xs">{description}</div>}
      <div className="text-muted-foreground text-xs">Coming in Phase {phase}.</div>
    </div>
  );
}
