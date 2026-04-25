import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Merge Tailwind class names with conflict resolution.
 *
 * Combines `clsx` (conditional classes) with `tailwind-merge` (resolves
 * conflicting utility classes — `cn("p-2", "p-4")` returns `"p-4"`).
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
