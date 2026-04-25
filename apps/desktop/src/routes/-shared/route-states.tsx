import { Link, type ErrorComponentProps } from "@tanstack/react-router";
import { type JSX } from "react";

export function RoutePending(): JSX.Element {
  return (
    <div className="flex h-full items-center justify-center text-xs text-muted-foreground">
      Loading…
    </div>
  );
}

export function RouteErrorBoundary({ error, reset }: ErrorComponentProps): JSX.Element {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-6">
      <div className="text-md font-medium text-destructive">Something went wrong</div>
      <pre className="max-w-2xl whitespace-pre-wrap font-mono text-xs text-muted-foreground">
        {error instanceof Error ? error.message : String(error)}
      </pre>
      <div className="flex gap-2 pt-2">
        <button
          type="button"
          onClick={reset}
          className="border border-border px-3 py-1 text-xs hover:bg-accent"
        >
          Reset
        </button>
        <Link to="/" className="border border-border px-3 py-1 text-xs hover:bg-accent">
          Home
        </Link>
      </div>
    </div>
  );
}

export function SectionErrorBoundary({
  error,
  reset,
}: ErrorComponentProps): JSX.Element {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-6">
      <div className="text-md font-medium text-destructive">Section unavailable</div>
      <pre className="max-w-2xl whitespace-pre-wrap font-mono text-xs text-muted-foreground">
        {error instanceof Error ? error.message : String(error)}
      </pre>
      <button
        type="button"
        onClick={reset}
        className="border border-border px-3 py-1 text-xs hover:bg-accent"
      >
        Retry
      </button>
    </div>
  );
}

export function RouteNotFound(): JSX.Element {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-6">
      <div className="text-md font-medium">Route not found</div>
      <Link to="/" className="border border-border px-3 py-1 text-xs hover:bg-accent">
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
      <div className="text-md font-medium">{feature}</div>
      {description && (
        <div className="max-w-md text-xs text-muted-foreground">{description}</div>
      )}
      <div className="text-xs text-muted-foreground">Coming in Phase {phase}.</div>
    </div>
  );
}
