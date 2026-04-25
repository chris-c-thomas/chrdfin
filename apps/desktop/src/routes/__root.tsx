import { SidebarProvider } from "@chrdfin/ui";
import { createRootRoute, Outlet, ScrollRestoration } from "@tanstack/react-router";
import { useState, type JSX } from "react";

import { CommandPalette } from "@/components/shell/command-palette.js";
import { AppHeader } from "@/components/shell/header.js";
import { AppSidebar } from "@/components/shell/sidebar.js";

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout(): JSX.Element {
  const [paletteOpen, setPaletteOpen] = useState(false);

  return (
    <SidebarProvider defaultOpen>
      <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
        <AppSidebar />
        <div className="flex flex-1 flex-col overflow-hidden">
          <AppHeader onOpenCommandPalette={() => setPaletteOpen(true)} />
          <main className="flex-1 overflow-auto">
            <Outlet />
          </main>
        </div>
        <CommandPalette open={paletteOpen} onOpenChange={setPaletteOpen} />
        <ScrollRestoration />
      </div>
    </SidebarProvider>
  );
}
