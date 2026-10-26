import { describe, expect, it, jest } from "@jest/globals";
import { renderHook } from "@testing-library/react";

import { useMountEffect } from "@/lib/react/use-mount-effect";

describe("useMountEffect", () => {
  it("runs once, however many times the component renders", () => {
    const effect = jest.fn<() => void>();
    const { rerender } = renderHook(() => useMountEffect(effect));

    rerender();
    rerender();

    expect(effect).toHaveBeenCalledTimes(1);
  });

  it("runs the cleanup it returns when the component unmounts", () => {
    const cleanup = jest.fn<() => void>();
    const { unmount } = renderHook(() => useMountEffect(() => cleanup));

    expect(cleanup).not.toHaveBeenCalled();

    unmount();

    expect(cleanup).toHaveBeenCalledTimes(1);
  });

  it("tolerates an effect that returns no cleanup", () => {
    const { unmount } = renderHook(() => useMountEffect(() => undefined));

    expect(() => unmount()).not.toThrow();
  });

  it("runs the closure as it stood at mount", () => {
    // Stale by design: a later render's closure is never the one that runs, which is what makes the
    // empty dependency list honest rather than a lie the linter is told to ignore.
    const seen: Array<number> = [];
    const { rerender } = renderHook((value: number) => useMountEffect(() => void seen.push(value)), {
      initialProps: 1,
    });

    rerender(2);

    expect(seen).toEqual([1]);
  });
});
