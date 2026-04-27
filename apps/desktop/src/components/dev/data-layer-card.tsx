import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle, cn } from "@chrdfin/ui";
import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { type JSX } from "react";

import { useSyncDataMutation, useSyncProgress, useSyncStatus } from "@/lib/queries/sync.js";

interface SyncRunRow {
  readonly id: string;
  readonly syncType: string;
  readonly status: string;
  readonly tickersSynced: number | null;
  readonly rowsUpserted: number | null;
  readonly errorMessage: string | null;
  readonly startedAt: string;
  readonly completedAt: string | null;
}

/**
 * Phase-1 developer surface for the data layer. Run sync, watch live
 * progress, browse the recent sync_log. Replaced by proper Settings UI
 * in Phase 10.
 */
export function DataLayerCard(): JSX.Element {
  const status = useSyncStatus();
  const progress = useSyncProgress();
  const mutation = useSyncDataMutation();

  const recent = useQuery<readonly SyncRunRow[], Error>({
    queryKey: ["sync", "recent"],
    queryFn: async () => invoke<readonly SyncRunRow[]>("get_recent_sync_runs"),
    refetchInterval: 30_000,
  });

  const isSyncing = mutation.isPending || status.data?.latest?.status === "started";
  const pct =
    progress && progress.total > 0
      ? Math.min(100, Math.round((progress.current / progress.total) * 100))
      : 0;

  return (
    <Card className="max-w-2xl">
      <CardHeader>
        <CardTitle>Data layer (developer view)</CardTitle>
        <CardDescription>
          Drives the Phase 1 sync orchestrator. Replaced by proper Settings UI in Phase 10.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            size="sm"
            variant="default"
            disabled={isSyncing}
            onClick={() => mutation.mutate({ mode: "incremental" })}
          >
            Run Incremental
          </Button>
          <Button
            type="button"
            size="sm"
            variant="outline"
            disabled={isSyncing}
            onClick={() => mutation.mutate({ mode: "full" })}
          >
            Run Full
          </Button>
          <span className="text-muted-foreground ml-auto text-xs">
            Last successful: {status.data?.lastSuccessfulSync ?? "never"}
          </span>
        </div>

        {isSyncing && (
          <div className="flex flex-col gap-1">
            <div className="text-muted-foreground flex items-center justify-between text-xs">
              <span>
                {progress?.phase ?? "starting"}
                {progress?.message ? ` — ${progress.message}` : ""}
              </span>
              <span className="font-mono tabular-nums">
                {progress ? `${progress.current}/${progress.total}` : "…"}
              </span>
            </div>
            <div className="bg-muted h-1.5 w-full overflow-hidden rounded-sm">
              <div
                className={cn("bg-warning h-full transition-[width] duration-200")}
                style={{ width: `${pct}%` }}
              />
            </div>
          </div>
        )}

        {mutation.isError && (
          <div className="text-destructive text-xs">Sync failed: {mutation.error.message}</div>
        )}

        {status.data?.latest && (
          <div className="border-border rounded-sm border">
            <div className="border-border bg-muted/40 border-b px-3 py-2 text-xs font-medium">
              Latest sync log entry
            </div>
            <dl className="grid grid-cols-2 gap-x-4 gap-y-1 px-3 py-2 text-xs">
              <dt className="text-muted-foreground">Type</dt>
              <dd>{status.data.latest.syncType}</dd>
              <dt className="text-muted-foreground">Status</dt>
              <dd>{status.data.latest.status}</dd>
              <dt className="text-muted-foreground">Tickers synced</dt>
              <dd>{status.data.latest.tickersSynced ?? "—"}</dd>
              <dt className="text-muted-foreground">Rows upserted</dt>
              <dd>{status.data.latest.rowsUpserted ?? "—"}</dd>
              <dt className="text-muted-foreground">Started</dt>
              <dd className="font-mono">{status.data.latest.startedAt}</dd>
              <dt className="text-muted-foreground">Completed</dt>
              <dd className="font-mono">{status.data.latest.completedAt ?? "—"}</dd>
              {status.data.latest.errorMessage && (
                <>
                  <dt className="text-muted-foreground">Error</dt>
                  <dd className="text-destructive">{status.data.latest.errorMessage}</dd>
                </>
              )}
            </dl>
          </div>
        )}

        {recent.data && recent.data.length > 0 && (
          <div className="overflow-x-auto text-xs">
            <table className="w-full">
              <thead className="text-muted-foreground text-left">
                <tr>
                  <th className="py-1 pr-2">Started</th>
                  <th className="py-1 pr-2">Type</th>
                  <th className="py-1 pr-2">Status</th>
                  <th className="py-1 pr-2 text-right">Rows</th>
                </tr>
              </thead>
              <tbody>
                {recent.data.map((r) => (
                  <tr key={r.id} className="border-border border-t">
                    <td className="py-1 pr-2 font-mono">{r.startedAt}</td>
                    <td className="py-1 pr-2">{r.syncType}</td>
                    <td className="py-1 pr-2">{r.status}</td>
                    <td className="py-1 pr-2 text-right tabular-nums">{r.rowsUpserted ?? "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
