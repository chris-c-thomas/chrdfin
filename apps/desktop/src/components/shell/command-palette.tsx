import { CommandDialog, CommandEmpty, CommandInput, CommandList } from "@chrdfin/ui";
import { useEffect, type JSX } from "react";

export interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/**
 * Phase 0 command palette — empty shell wired to Cmd/Ctrl+K.
 * Result groups (Navigate, Tickers, Portfolios, Tools, Actions) populate
 * in later phases as their underlying Tauri commands ship.
 */
export function CommandPalette({ open, onOpenChange }: CommandPaletteProps): JSX.Element {
  useEffect(() => {
    function handler(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        onOpenChange(!open);
      }
    }
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onOpenChange]);

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <CommandInput placeholder="Search tickers, portfolios, tools…" />
      <CommandList>
        <CommandEmpty>No results.</CommandEmpty>
      </CommandList>
    </CommandDialog>
  );
}
