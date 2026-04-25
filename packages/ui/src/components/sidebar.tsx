import { Slot } from "@radix-ui/react-slot";
import {
  createContext,
  forwardRef,
  useCallback,
  useContext,
  useState,
  type AnchorHTMLAttributes,
  type ButtonHTMLAttributes,
  type HTMLAttributes,
  type JSX,
  type ReactNode,
} from "react";

import { cn } from "../lib/utils.js";

/**
 * Minimal sidebar primitives for chrdfin's platform shell.
 *
 * Owns its own collapse state via React context (not localStorage). The
 * `apps/desktop` shell wraps this with persistence via Tauri/DuckDB later.
 */

interface SidebarContextValue {
  open: boolean;
  setOpen: (open: boolean) => void;
  toggle: () => void;
  collapsible: "icon" | "none";
}

const SidebarContext = createContext<SidebarContextValue | null>(null);

export function useSidebar(): SidebarContextValue {
  const ctx = useContext(SidebarContext);
  if (!ctx) {
    throw new Error("useSidebar must be used inside <SidebarProvider>");
  }
  return ctx;
}

export interface SidebarProviderProps {
  children: ReactNode;
  defaultOpen?: boolean;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  collapsible?: "icon" | "none";
}

export function SidebarProvider({
  children,
  defaultOpen = true,
  open: controlledOpen,
  onOpenChange,
  collapsible = "icon",
}: SidebarProviderProps): JSX.Element {
  const [internalOpen, setInternalOpen] = useState(defaultOpen);
  const isControlled = controlledOpen !== undefined;
  const open = isControlled ? controlledOpen : internalOpen;

  const setOpen = useCallback(
    (next: boolean) => {
      if (!isControlled) setInternalOpen(next);
      onOpenChange?.(next);
    },
    [isControlled, onOpenChange],
  );

  const toggle = useCallback(() => setOpen(!open), [open, setOpen]);

  return (
    <SidebarContext.Provider value={{ open, setOpen, toggle, collapsible }}>
      {children}
    </SidebarContext.Provider>
  );
}

export interface SidebarProps extends HTMLAttributes<HTMLDivElement> {
  collapsible?: "icon" | "none";
}

export const Sidebar = forwardRef<HTMLDivElement, SidebarProps>(
  ({ className, collapsible: _collapsible, children, ...props }, ref) => {
    const { open } = useSidebar();
    return (
      <aside
        ref={ref}
        data-state={open ? "expanded" : "collapsed"}
        className={cn(
          "flex h-screen flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground transition-[width] duration-150",
          open ? "w-60" : "w-12",
          className,
        )}
        {...props}
      >
        {children}
      </aside>
    );
  },
);
Sidebar.displayName = "Sidebar";

export const SidebarHeader = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn(
        "flex h-12 shrink-0 items-center border-b border-sidebar-border px-3",
        className,
      )}
      {...props}
    />
  ),
);
SidebarHeader.displayName = "SidebarHeader";

export const SidebarContent = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn("flex-1 overflow-y-auto overflow-x-hidden", className)}
      {...props}
    />
  ),
);
SidebarContent.displayName = "SidebarContent";

export const SidebarFooter = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn("shrink-0 border-t border-sidebar-border p-2", className)}
      {...props}
    />
  ),
);
SidebarFooter.displayName = "SidebarFooter";

export const SidebarGroup = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn("flex flex-col gap-0.5 px-2 py-2", className)}
      {...props}
    />
  ),
);
SidebarGroup.displayName = "SidebarGroup";

export const SidebarGroupLabel = forwardRef<
  HTMLDivElement,
  HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => {
  const { open } = useSidebar();
  if (!open) return null;
  return (
    <div
      ref={ref}
      className={cn(
        "px-2 pb-1 pt-0.5 text-xs font-medium uppercase tracking-wider text-muted-foreground",
        className,
      )}
      {...props}
    />
  );
});
SidebarGroupLabel.displayName = "SidebarGroupLabel";

export const SidebarMenu = forwardRef<HTMLUListElement, HTMLAttributes<HTMLUListElement>>(
  ({ className, ...props }, ref) => (
    <ul
      ref={ref}
      className={cn("flex w-full flex-col gap-0.5", className)}
      {...props}
    />
  ),
);
SidebarMenu.displayName = "SidebarMenu";

export const SidebarMenuItem = forwardRef<
  HTMLLIElement,
  HTMLAttributes<HTMLLIElement>
>(({ className, ...props }, ref) => (
  <li ref={ref} className={cn("relative", className)} {...props} />
));
SidebarMenuItem.displayName = "SidebarMenuItem";

export interface SidebarMenuButtonProps
  extends ButtonHTMLAttributes<HTMLButtonElement> {
  asChild?: boolean;
  isActive?: boolean;
}

export const SidebarMenuButton = forwardRef<
  HTMLButtonElement,
  SidebarMenuButtonProps
>(({ asChild, isActive, className, ...props }, ref) => {
  const Comp = asChild ? Slot : "button";
  return (
    <Comp
      ref={ref}
      data-active={isActive ? "true" : undefined}
      className={cn(
        "flex h-8 w-full items-center gap-2 rounded-sm px-2 text-sm transition-colors",
        "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sidebar-ring",
        "data-[active=true]:bg-sidebar-accent data-[active=true]:text-sidebar-accent-foreground data-[active=true]:font-medium",
        "data-[active=true]:before:absolute data-[active=true]:before:inset-y-0 data-[active=true]:before:left-0 data-[active=true]:before:w-0.5 data-[active=true]:before:bg-sidebar-primary",
        className,
      )}
      {...props}
    />
  );
});
SidebarMenuButton.displayName = "SidebarMenuButton";

export interface SidebarMenuLinkProps
  extends AnchorHTMLAttributes<HTMLAnchorElement> {
  isActive?: boolean;
}

export const SidebarMenuLink = forwardRef<
  HTMLAnchorElement,
  SidebarMenuLinkProps
>(({ isActive, className, ...props }, ref) => (
  <a
    ref={ref}
    data-active={isActive ? "true" : undefined}
    className={cn(
      "flex h-8 w-full items-center gap-2 rounded-sm px-2 text-sm transition-colors",
      "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
      "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sidebar-ring",
      "data-[active=true]:bg-sidebar-accent data-[active=true]:text-sidebar-accent-foreground data-[active=true]:font-medium",
      "data-[active=true]:before:absolute data-[active=true]:before:inset-y-0 data-[active=true]:before:left-0 data-[active=true]:before:w-0.5 data-[active=true]:before:bg-sidebar-primary",
      className,
    )}
    {...props}
  />
));
SidebarMenuLink.displayName = "SidebarMenuLink";
