import { cn } from "@chrdfin/ui";
import { Loader2 } from "lucide-react";
import { type JSX } from "react";

import { useSyncDataMutation, useSyncProgress, useSyncStatus } from "@/lib/queries/sync.js";

type BadgeState = "syncing" | "failed" | "up-to-date" | "never-run" | "loading";

interface BadgeView {
  readonly dotClass: string;
  readonly label: string;
}

function describe(state: BadgeState, label: string): BadgeView {
  switch (state) {
    case "syncing":
      return { dotClass: "bg-warning animate-pulse", label };
    case "failed":
      return { dotClass: "bg-warning", label };
    case "up-to-date":
      return { dotClass: "bg-gain", label };
    case "never-run":
      return { dotClass: "bg-muted-foreground", label };
    case "loading":
      return { dotClass: "bg-muted-foreground/40", label };
  }
}

function formatRelative(iso: string | null): string {
  if (!iso) return "never";
  const t = new Date(iso).getTime();
  if (Number.isNaN(t)) return "unknown";
  const diffMs = Date.now() - t;
  const sec = Math.max(0, Math.round(diffMs / 1000));
  if (sec < 60) return `${sec}s ago`;
  const min = Math.round(sec / 60);
  if (min < 60) return `${min}m ago`;
  const hr = Math.round(min / 60);
  if (hr < 24) return `${hr}h ago`;
  return `${Math.round(hr / 24)}d ago`;
}

/**
 * Header indicator for the data layer. Click to kick off an incremental
 * sync. While in flight, shows live progress driven by the orchestrator's
 * `sync:progress` events.
 */
export function SyncStatusBadge(): JSX.Element {
  const status = useSyncStatus();
  const progress = useSyncProgress();
  const mutation = useSyncDataMutation();

  const isSyncing = mutation.isPending || status.data?.latest?.status === "started";

  let view: BadgeView;
  if (status.isLoading) {
    view = describe("loading", "Loading…");
  } else if (isSyncing) {
    const phase = progress?.phase ? `${progress.phase} ` : "";
    const counter = progress && progress.total > 0 ? `${progress.current}/${progress.total}` : "";
    view = describe("syncing", `Syncing ${phase}${counter}`.trim());
  } else if (status.data?.latest?.status === "failed") {
    view = describe("failed", "Last sync failed — click to retry");
  } else if (status.data?.lastSuccessfulSync) {
    view = describe("up-to-date", `Synced ${formatRelative(status.data.lastSuccessfulSync)}`);
  } else {
    view = describe("never-run", "No sync yet — click to seed");
  }

  return (
    <button
      type="button"
      onClick={() => mutation.mutate({ mode: "incremental" })}
      disabled={isSyncing}
      title={view.label}
      className={cn(
        "flex items-center gap-2 rounded-sm px-2 text-xs transition-colors",
        "hover:bg-accent hover:text-foreground",
        "focus-visible:ring-ring focus-visible:outline-none focus-visible:ring-2",
        "disabled:cursor-default",
      )}
    >
      {isSyncing ? (
        <Loader2 className="text-warning size-3.5 animate-spin" aria-hidden="true" />
      ) : (
        <span className={cn("size-2 rounded-full", view.dotClass)} aria-hidden="true" />
      )}
      <span className="text-muted-foreground max-w-[16rem] truncate">{view.label}</span>
    </button>
  );
}
