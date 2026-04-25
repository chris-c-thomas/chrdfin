import type { z } from "zod";

/**
 * Adapt a Zod schema to TanStack Router's `validateSearch` signature.
 *
 * Throws on validation failure; TanStack Router's error boundary catches and
 * surfaces the message. Bookmarked URLs with stale params land on the
 * section's error component with a descriptive message.
 *
 * @example
 *   export const Route = createFileRoute("/analysis/backtest")({
 *     validateSearch: zodValidator(BacktestSearchSchema),
 *   });
 */
export function zodValidator<T extends z.ZodTypeAny>(schema: T) {
  return (input: Record<string, unknown>): z.infer<T> => schema.parse(input);
}
