import { describe, expect, it, jest } from "@jest/globals";
import { act, renderHook, waitFor } from "@testing-library/react";

import { noop } from "@/lib/callbacks/noop";
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
        await jest.advanceTimersByTimeAsync(2000);
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

  it("skips ticks while a read is pending and resumes after it settles", async () => {
    jest.useFakeTimers();

    try {
      let resolve: (value: string) => void = noop;
      const pending = new Promise<string>((settle) => {
        resolve = settle;
      });
      const read = jest.fn<() => Promise<string>>().mockReturnValueOnce(pending).mockResolvedValue("next");
      const { result } = renderHook(() => usePolledValue(read, 1000));

      await act(async () => {
        await jest.advanceTimersByTimeAsync(3000);
      });

      expect(read).toHaveBeenCalledTimes(1);
      expect(result.current).toBeNull();

      await act(async () => {
        resolve("first");
      });

      expect(result.current).toBe("first");

      await act(async () => {
        await jest.advanceTimersByTimeAsync(1000);
      });

      expect(read).toHaveBeenCalledTimes(2);
      expect(result.current).toBe("next");
    } finally {
      jest.useRealTimers();
    }
  });

  it("ignores a pending response after unmount and does not resume polling", async () => {
    jest.useFakeTimers();

    try {
      let resolve: (value: string) => void = noop;
      const pending = new Promise<string>((settle) => {
        resolve = settle;
      });
      const read = jest.fn<() => Promise<string>>().mockReturnValue(pending);
      const { result, unmount } = renderHook(() => usePolledValue(read, 1000));

      expect(read).toHaveBeenCalledTimes(1);
      expect(result.current).toBeNull();

      unmount();

      await act(async () => {
        resolve("late");
      });

      await act(async () => {
        await jest.advanceTimersByTimeAsync(5000);
      });

      expect(result.current).toBeNull();
      expect(read).toHaveBeenCalledTimes(1);
    } finally {
      jest.useRealTimers();
    }
  });

  it("ignores the previous interval's pending response and polls at the new interval", async () => {
    jest.useFakeTimers();

    try {
      let resolve: (value: string) => void = noop;
      const pending = new Promise<string>((settle) => {
        resolve = settle;
      });
      const read = jest
        .fn<() => Promise<string>>()
        .mockReturnValueOnce(pending)
        .mockResolvedValueOnce("current")
        .mockResolvedValue("next");
      const { result, rerender } = renderHook((intervalMs: number) => usePolledValue(read, intervalMs), {
        initialProps: 1000,
      });

      expect(read).toHaveBeenCalledTimes(1);

      rerender(2000);

      await waitFor(() => expect(result.current).toBe("current"));

      expect(read).toHaveBeenCalledTimes(2);

      await act(async () => {
        resolve("stale");
      });

      expect(result.current).toBe("current");

      await act(async () => {
        await jest.advanceTimersByTimeAsync(1000);
      });

      expect(read).toHaveBeenCalledTimes(2);

      await act(async () => {
        await jest.advanceTimersByTimeAsync(1000);
      });

      expect(read).toHaveBeenCalledTimes(3);
      expect(result.current).toBe("next");
    } finally {
      jest.useRealTimers();
    }
  });

  it("keeps the last value after rejection and resumes on the next tick", async () => {
    jest.useFakeTimers();

    try {
      const read = jest
        .fn<() => Promise<string>>()
        .mockResolvedValueOnce("ok")
        .mockRejectedValueOnce(new Error("gone"))
        .mockResolvedValue("recovered");
      const { result } = renderHook(() => usePolledValue(read, 1000));

      await waitFor(() => expect(result.current).toBe("ok"));

      await act(async () => {
        await jest.advanceTimersByTimeAsync(1000);
      });

      expect(result.current).toBe("ok");

      await act(async () => {
        await jest.advanceTimersByTimeAsync(1000);
      });

      expect(result.current).toBe("recovered");
    } finally {
      jest.useRealTimers();
    }
  });

  it("resumes polling after a synchronous reader throws", async () => {
    jest.useFakeTimers();

    try {
      const read = jest
        .fn<() => string>()
        .mockImplementationOnce(() => {
          throw new Error("gone");
        })
        .mockReturnValue("recovered");
      const { result } = renderHook(() => usePolledValue(read, 1000));

      expect(result.current).toBeNull();

      await act(async () => {
        await jest.advanceTimersByTimeAsync(1000);
      });

      expect(read).toHaveBeenCalledTimes(2);
      expect(result.current).toBe("recovered");
    } finally {
      jest.useRealTimers();
    }
  });
});
