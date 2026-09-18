import { describe, expect, it, jest } from "@jest/globals";
import { act, renderHook } from "@testing-library/react";
import { StrictMode, useEffect } from "react";

import { usePathSeed } from "@/core/ui/form/use-path-seed";
import { noop } from "@/lib/callbacks/noop";

describe("usePathSeed", () => {
  it("does not deliver a pending seed after unmount", async () => {
    let resolve: (path: string) => void = noop;

    const pending = new Promise<string>((settle) => {
      resolve = settle;
    });

    const onSeeded = jest.fn<(path: string) => void>();
    const { result, unmount } = renderHook(() => usePathSeed({ seed: () => pending, onSeeded }));

    act(() => result.current.request());

    unmount();

    await act(async () => {
      resolve("C:/late");
    });

    expect(onSeeded).not.toHaveBeenCalled();
  });

  it("delivers only the current seed after Strict Mode replays effects", async () => {
    let resolveOriginal: (path: string) => void = noop;

    const original = new Promise<string>((settle) => {
      resolveOriginal = settle;
    });

    let resolveCurrent: (path: string) => void = noop;

    const current = new Promise<string>((settle) => {
      resolveCurrent = settle;
    });

    const seed = jest.fn<() => Promise<string>>().mockReturnValueOnce(original).mockReturnValueOnce(current);
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
      resolveOriginal("C:/stale");
    });

    expect(onSeeded).not.toHaveBeenCalled();

    await act(async () => {
      resolveCurrent("C:/current");
    });

    expect(onSeeded).toHaveBeenCalledTimes(1);
    expect(onSeeded).toHaveBeenCalledWith("C:/current");
  });
});
