import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, renderHook } from "@testing-library/react";

import { IPanelWidth, usePanelWidth } from "@/core/shell/panel/use-panel-width";

function setWindowWidth(width: number): void {
  act(() => {
    Object.defineProperty(window, "innerWidth", { configurable: true, value: width, writable: true });
    window.dispatchEvent(new Event("resize"));
  });
}

describe("usePanelWidth", () => {
  beforeEach(() => {
    window.localStorage.clear();
    Object.defineProperty(window, "innerWidth", { configurable: true, value: 1600, writable: true });
  });

  it("starts at the stored preference when the window has room for it", () => {
    window.localStorage.setItem("xrf.panels.left.width", "640");

    expect(renderHook(() => usePanelWidth("left", 1)).result.current.width).toBe(640);
  });

  it("renders a stored preference the window cannot afford at the budget instead", () => {
    window.localStorage.setItem("xrf.panels.right.width", "640");
    Object.defineProperty(window, "innerWidth", { configurable: true, value: 900, writable: true });

    const { result } = renderHook(() => usePanelWidth("right", 1));

    expect(result.current.width).toBe(450);
    // Reconciling with the window is not the user restating a preference, so the stored value stands.
    expect(window.localStorage.getItem("xrf.panels.right.width")).toBe("640");
  });

  it("re-clamps as the window narrows and gives the preference back when it widens", () => {
    window.localStorage.setItem("xrf.panels.left.width", "640");

    const { result } = renderHook(() => usePanelWidth("left", 1));

    expect(result.current.width).toBe(640);

    setWindowWidth(900);
    expect(result.current.width).toBe(450);

    setWindowWidth(1600);
    expect(result.current.width).toBe(640);
    expect(window.localStorage.getItem("xrf.panels.left.width")).toBe("640");
  });

  it("halves the share when a second panel opens and restores it when that panel closes", () => {
    window.localStorage.setItem("xrf.panels.left.width", "640");

    const { result, rerender } = renderHook(({ openCount }) => usePanelWidth("left", openCount), {
      initialProps: { openCount: 1 },
    });

    expect(result.current.width).toBe(640);

    rerender({ openCount: 2 });
    expect(result.current.width).toBe(400);

    rerender({ openCount: 1 });
    expect(result.current.width).toBe(640);
  });

  it("stores a dragged width, clamped, because a drag is the user stating a preference", () => {
    Object.defineProperty(window, "innerWidth", { configurable: true, value: 900, writable: true });

    const { result }: { result: { current: IPanelWidth } } = renderHook(() => usePanelWidth("left", 1));

    act(() => result.current.onResize(1200));

    expect(result.current.width).toBe(450);
    expect(window.localStorage.getItem("xrf.panels.left.width")).toBe("450");
  });
});
