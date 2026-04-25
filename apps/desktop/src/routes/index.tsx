import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@chrdfin/ui";
import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState, type JSX } from "react";

interface HealthCheckResponse {
  status: string;
  db_initialized: boolean;
  version: string;
  core_version: string;
}

export const Route = createFileRoute("/")({
  component: DashboardPage,
});

function DashboardPage(): JSX.Element {
  const [health, setHealth] = useState<string>("checking…");

  useEffect(() => {
    invoke<HealthCheckResponse>("health_check")
      .then((result) => setHealth(JSON.stringify(result, null, 2)))
      .catch((err: unknown) => {
        const message = err instanceof Error ? err.message : String(err);
        setHealth(`Tauri command failed: ${message}`);
      });
  }, []);

  return (
    <div className="p-6">
      <div className="mb-6 flex flex-col gap-1">
        <h1 className="text-lg font-medium">Dashboard</h1>
        <p className="text-muted-foreground text-xs">
          Phase 0 — platform shell, schema initialization, and IPC round-trip verification.
        </p>
      </div>

      <Card className="max-w-2xl">
        <CardHeader>
          <CardTitle>Health check</CardTitle>
          <CardDescription>
            Round-trip from React → Tauri command → chrdfin-core → DuckDB.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <pre className="bg-muted overflow-x-auto rounded-sm p-3 font-mono text-xs">{health}</pre>
        </CardContent>
      </Card>
    </div>
  );
}
