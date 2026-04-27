import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import type { SyncMode, SyncProgress, SyncStatus, SyncSummary } from "@chrdfin/types";

import { qk } from "@/lib/queryKeys.js";

const PROGRESS_EVENT = "sync:progress";

/**
 * Poll the latest sync_log row + last successful sync timestamp. Stale
 * time is 0 so the dashboard always shows fresh state on focus, but a
 * 30s refetch covers idle tabs too.
 */
export function useSyncStatus() {
  return useQuery<SyncStatus, Error>({
    queryKey: qk.syncStatus(),
    queryFn: async () => invoke<SyncStatus>("get_sync_status"),
    staleTime: 0,
    refetchInterval: 30_000,
    refetchOnWindowFocus: true,
  });
}

interface SyncDataInput {
  readonly mode: SyncMode;
}

/**
 * Run a manual full or incremental sync. On success, invalidate every
 * data root so any visible chart, screener, or detail view refetches
 * against fresh DuckDB rows.
 */
export function useSyncDataMutation() {
  const qc = useQueryClient();
  return useMutation<SyncSummary, Error, SyncDataInput>({
    mutationFn: async ({ mode }) => invoke<SyncSummary>("sync_data", { input: { mode } }),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: qk.syncStatus() });
      void qc.invalidateQueries({ queryKey: qk.prices() });
      void qc.invalidateQueries({ queryKey: qk.macro() });
      void qc.invalidateQueries({ queryKey: qk.asset() });
    },
  });
}

/**
 * Subscribe to the orchestrator's `sync:progress` events. Returns the
 * most recent payload, or `null` while no sync is in flight. The status
 * query becomes the source of truth once a run completes — this hook
 * exists for live UI feedback during the run itself.
 */
export function useSyncProgress(): SyncProgress | null {
  const [progress, setProgress] = useState<SyncProgress | null>(null);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let cancelled = false;

    listen<SyncProgress>(PROGRESS_EVENT, (event) => {
      setProgress(event.payload);
    })
      .then((fn) => {
        if (cancelled) {
          fn();
          return;
        }
        unlisten = fn;
      })
      .catch((err: unknown) => {
        console.warn("failed to subscribe to sync:progress", err);
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return progress;
}
