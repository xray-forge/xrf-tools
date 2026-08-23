import { describe, expect, it } from "@jest/globals";
import { renderHook } from "@testing-library/react";

import { useLastContent } from "@/core/archive/use-last-content";
import { Nullable } from "@/lib/types/general";

interface IRenderProps {
  current: Nullable<string>;
  isStale: boolean;
}

function renderLastContent(initial: IRenderProps) {
  return renderHook(({ current, isStale }: IRenderProps) => useLastContent(current, isStale), {
    initialProps: initial,
  });
}

describe("useLastContent", () => {
  it("shows what it is given", () => {
    expect(renderLastContent({ current: "first", isStale: false }).result.current).toBe("first");
  });

  it("holds the previous value while the next one loads", () => {
    const { result, rerender } = renderLastContent({ current: "first", isStale: false });

    // What a selection actually does: clears the content, then fills it a few milliseconds later. Without this the
    // panel unmounts in between, which is how the player used to lose its volume and its element.
    rerender({ current: null, isStale: true });
    expect(result.current).toBe("first");

    rerender({ current: "second", isStale: false });
    expect(result.current).toBe("second");
  });

  it("forgets the previous value once the load settles on nothing", () => {
    const { result, rerender } = renderLastContent({ current: "first", isStale: false });

    // An empty result is an answer, not a gap. Keeping the old file here would report it as the current selection.
    rerender({ current: null, isStale: false });

    expect(result.current).toBeNull();
  });

  it("has nothing to hold before the first value arrives", () => {
    expect(renderLastContent({ current: null, isStale: true }).result.current).toBeNull();
  });
});
