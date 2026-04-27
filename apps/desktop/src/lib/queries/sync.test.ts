/**
 * Smoke test for `useSyncProgress`: verifies the hook subscribes to the
 * Tauri `sync:progress` event and surfaces the latest payload through
 * its return value. The Tauri event API is mocked so the test runs
 * without a Tauri runtime.
 */
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

type ProgressHandler = (event: { payload: unknown }) => void;

const handlers: ProgressHandler[] = [];
const unlisten = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (event: string, handler: ProgressHandler) => {
    if (event === "sync:progress") {
      handlers.push(handler);
    }
    return unlisten;
  }),
}));

beforeEach(() => {
  handlers.length = 0;
  unlisten.mockClear();
});

afterEach(() => {
  vi.clearAllMocks();
});

describe("useSyncProgress", () => {
  it("returns the latest sync:progress payload after the listener fires", async () => {
    const { useSyncProgress } = await import("./sync.js");
    const { result } = renderHook(() => useSyncProgress());

    expect(result.current).toBeNull();

    await waitFor(() => expect(handlers).toHaveLength(1));

    act(() => {
      handlers[0]?.({
        payload: {
          phase: "prices",
          current: 3,
          total: 10,
          message: "SPY",
        },
      });
    });

    expect(result.current).toEqual({
      phase: "prices",
      current: 3,
      total: 10,
      message: "SPY",
    });
  });

  it("unsubscribes on unmount", async () => {
    const { useSyncProgress } = await import("./sync.js");
    const { unmount } = renderHook(() => useSyncProgress());

    await waitFor(() => expect(handlers).toHaveLength(1));
    unmount();
    await waitFor(() => expect(unlisten).toHaveBeenCalledTimes(1));
  });
});
