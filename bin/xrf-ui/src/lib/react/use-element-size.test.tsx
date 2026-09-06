import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { act, render, RenderResult } from "@testing-library/react";
import { ReactElement } from "react";

import { IElementSize, useElementSize } from "@/lib/react/use-element-size";
import { Nullable } from "@/lib/types/general";

/** What every mounted element reports, since jsdom lays nothing out and answers zero for both dimensions. */
let laidOut: IElementSize = { width: 0, height: 0 };

/** The observer's callback, so a test can resize the element the way a browser would. */
let notify: Nullable<() => void> = null;

/** Every measurement the probe has been handed, so a test can tell a re-measure from a re-publish. */
let published: Array<IElementSize> = [];

class TestResizeObserver {
  public constructor(callback: () => void) {
    notify = callback;
  }

  public observe(): void {}

  public unobserve(): void {}

  public disconnect(): void {
    notify = null;
  }
}

function Probe(): ReactElement {
  const [ref, size] = useElementSize<HTMLDivElement>();

  if (size) {
    published.push(size);
  }

  return (
    <div ref={ref} data-testid={"probe"}>
      {size ? `${size.width}x${size.height}` : "unmeasured"}
    </div>
  );
}

describe("useElementSize", () => {
  beforeEach(() => {
    laidOut = { width: 800, height: 600 };
    notify = null;
    published = [];

    Object.defineProperty(HTMLElement.prototype, "clientWidth", { configurable: true, get: () => laidOut.width });
    Object.defineProperty(HTMLElement.prototype, "clientHeight", { configurable: true, get: () => laidOut.height });

    global.ResizeObserver = TestResizeObserver as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    Reflect.deleteProperty(HTMLElement.prototype, "clientWidth");
    Reflect.deleteProperty(HTMLElement.prototype, "clientHeight");
  });

  it("measures the element before anything is painted against it", () => {
    const { getByTestId }: RenderResult = render(<Probe />);

    // Measured in a layout effect, so the size is there on the first painted frame rather than one frame late, which
    // is a picture placed against a box it does not have.
    expect(getByTestId("probe").textContent).toBe("800x600");
  });

  it("reports every later size the element takes", () => {
    const { getByTestId }: RenderResult = render(<Probe />);

    act(() => {
      laidOut = { width: 1200, height: 700 };
      notify?.();
    });

    expect(getByTestId("probe").textContent).toBe("1200x700");
  });

  it("keeps the last real size when the element reports none", () => {
    const { getByTestId }: RenderResult = render(<Probe />);

    act(() => {
      laidOut = { width: 0, height: 0 };
      notify?.();
    });

    // A closed panel is still observed and reports zero. Publishing that would leave the caller with nothing to lay
    // out against, and the element comes back the size it went away at.
    expect(getByTestId("probe").textContent).toBe("800x600");
  });

  it("publishes the same measurement as the same value", () => {
    render(<Probe />);

    act(() => notify?.());

    // An observer fires for the observation itself and for either dimension. A fresh object each time would be a new
    // dependency for everything derived from the size, so an unchanged box has to stay the value it already was.
    expect(published.length).toBeGreaterThan(0);
    expect(new Set(published).size).toBe(1);
  });

  it("stops observing when the element goes away", () => {
    const { unmount }: RenderResult = render(<Probe />);

    expect(notify).not.toBeNull();

    unmount();

    expect(notify).toBeNull();
  });
});
