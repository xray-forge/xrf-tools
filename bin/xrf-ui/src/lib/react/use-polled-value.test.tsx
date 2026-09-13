import { describe, expect, it, jest } from "@jest/globals";
import { act, renderHook, waitFor } from "@testing-library/react";

import { usePolledValue } from "@/lib/react/use-polled-value";

describe("usePolledValue", () => {
  it("reads once immediately rather than after the first interval", async () => {
    const { result } = renderHook(() => usePolledValue(() => "first", 10_000));

    await waitFor(() => expect(result.current).toBe("first"));
  });

  it("re-reads on the interval", async () => {
    jest.useFakeTimers();

    try {
      let reads: number = 0;
      const { result } = renderHook(() => usePolledValue(() => ++reads, 1000));

      await waitFor(() => expect(result.current).toBe(1));

      await act(async () => {
        jest.advanceTimersByTime(2000);
      });

      expect(result.current).toBe(3);
    } finally {
      jest.useRealTimers();
    }
  });

  it("stops reading once unmounted", async () => {
    jest.useFakeTimers();

    try {
      const read = jest.fn(() => 1);
      const { unmount } = renderHook(() => usePolledValue(read, 1000));

      await waitFor(() => expect(read).toHaveBeenCalledTimes(1));

      unmount();

      await act(async () => {
        jest.advanceTimersByTime(5000);
      });

      expect(read).toHaveBeenCalledTimes(1);
    } finally {
      jest.useRealTimers();
    }
  });

  it("keeps the last value when a read fails, because only the reader knows what to say about it", async () => {
    jest.useFakeTimers();

    try {
      let attempt: number = 0;
      const { result } = renderHook(() =>
        usePolledValue(() => (++attempt === 1 ? Promise.resolve("ok") : Promise.reject(new Error("gone"))), 1000)
      );

      await waitFor(() => expect(result.current).toBe("ok"));

      await act(async () => {
        jest.advanceTimersByTime(1000);
      });

      expect(result.current).toBe("ok");
    } finally {
      jest.useRealTimers();
    }
  });
});
