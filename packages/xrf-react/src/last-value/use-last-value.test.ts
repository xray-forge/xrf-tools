import { describe, expect, it } from "@jest/globals";
import { renderHook } from "@testing-library/react";
import { Nullable } from "@xrf/types";

import { useLastValue } from "./use-last-value";

interface IRenderProps {
  current: Nullable<string>;
  isStale: boolean;
}

function renderLastValue(initial: IRenderProps) {
  return renderHook(({ current, isStale }: IRenderProps) => useLastValue(current, isStale), {
    initialProps: initial,
  });
}

describe("useLastValue", () => {
  it("shows what it is given", () => {
    expect(renderLastValue({ current: "first", isStale: false }).result.current).toBe("first");
  });

  it("holds the previous value while the next one loads", () => {
    const { result, rerender } = renderLastValue({ current: "first", isStale: false });

    // What a selection actually does: clears the value, then fills it a few milliseconds later. Without this the
    // view unmounts in between and loses whatever it was holding.
    rerender({ current: null, isStale: true });
    expect(result.current).toBe("first");

    rerender({ current: "second", isStale: false });
    expect(result.current).toBe("second");
  });

  it("forgets the previous value once the load settles on nothing", () => {
    const { result, rerender } = renderLastValue({ current: "first", isStale: false });

    // An empty result is an answer, not a gap. Keeping the old value here would report it as the current selection.
    rerender({ current: null, isStale: false });

    expect(result.current).toBeNull();
  });

  it("holds an empty value as a value, not as a gap", () => {
    const { result, rerender } = renderLastValue({ current: "first", isStale: false });

    rerender({ current: "", isStale: true });

    expect(result.current).toBe("");
  });

  it("has nothing to hold before the first value arrives", () => {
    expect(renderLastValue({ current: null, isStale: true }).result.current).toBeNull();
  });
});
