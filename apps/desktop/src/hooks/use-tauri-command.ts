import {
  useMutation,
  useQuery,
  type UseMutationOptions,
  type UseQueryOptions,
} from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

/**
 * Invoke a Tauri command as a TanStack Query.
 *
 * The query key is `[command, args]` — the query refetches when args change.
 * Pass `enabled: false` to defer until args are ready.
 */
export function useTauriQuery<
  TResult,
  TArgs extends Record<string, unknown> = Record<string, never>,
>(
  command: string,
  args: TArgs = {} as TArgs,
  options?: Omit<UseQueryOptions<TResult, Error>, "queryKey" | "queryFn">,
) {
  return useQuery<TResult, Error>({
    queryKey: [command, args],
    queryFn: async () => {
      try {
        return await invoke<TResult>(command, args);
      } catch (err) {
        throw err instanceof Error ? err : new Error(String(err));
      }
    },
    ...options,
  });
}

/** Invoke a Tauri command as a TanStack Mutation. */
export function useTauriMutation<TResult, TArgs extends Record<string, unknown>>(
  command: string,
  options?: UseMutationOptions<TResult, Error, TArgs>,
) {
  return useMutation<TResult, Error, TArgs>({
    mutationFn: async (args) => {
      try {
        return await invoke<TResult>(command, args);
      } catch (err) {
        throw err instanceof Error ? err : new Error(String(err));
      }
    },
    ...options,
  });
}
