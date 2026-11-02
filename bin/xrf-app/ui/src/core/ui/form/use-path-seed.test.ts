import { describe, expect, it, jest } from "@jest/globals";
import { act, renderHook } from "@testing-library/react";
import { StrictMode, useEffect } from "react";

import { usePathSeed } from "@/core/ui/form/use-path-seed";
import { deferred } from "@/fixtures/utils/promise";

describe("usePathSeed", () => {
  it("does not deliver a pending seed after unmount", async () => {
    const pending = deferred<string>();
    const onSeeded = jest.fn<(path: string) => void>();
    const { result, unmount } = renderHook(() => usePathSeed({ seed: () => pending.promise, onSeeded }));

    act(() => result.current.request());

    unmount();

    await act(async () => {
      pending.resolve("C:/late");
    });

    expect(onSeeded).not.toHaveBeenCalled();
  });

  it("delivers only the current seed after Strict Mode replays effects", async () => {
    const original = deferred<string>();
    const current = deferred<string>();
    const seed = jest
      .fn<() => Promise<string>>()
      .mockReturnValueOnce(original.promise)
      .mockReturnValueOnce(current.promise);
    const onSeeded = jest.fn<(path: string) => void>();

    renderHook(
      () => {
        const { request } = usePathSeed({ seed, onSeeded });

        useEffect(() => request(), [request]);
      },
      { wrapper: StrictMode }
    );

    expect(seed).toHaveBeenCalledTimes(2);

    await act(async () => {
      original.resolve("C:/stale");
    });

    expect(onSeeded).not.toHaveBeenCalled();

    await act(async () => {
      current.resolve("C:/current");
    });

    expect(onSeeded).toHaveBeenCalledTimes(1);
    expect(onSeeded).toHaveBeenCalledWith("C:/current");
  });
});
