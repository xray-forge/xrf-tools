import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act, render, RenderResult } from "@testing-library/react";
import { ReactElement } from "react";

import { IElementSize, useElementSize } from "@/lib/react/use-element-size";

const ORIGINAL_RESIZE_OBSERVER = global.ResizeObserver;

/** What every mounted element reports, since jsdom lays nothing out and answers zero for both dimensions. */
let laidOut: IElementSize = { width: 0, height: 0 };

let observers: Array<TestResizeObserver> = [];
let sizes: WeakMap<HTMLElement, IElementSize> = new WeakMap();

/** Every measurement the probe has been handed, so a test can tell a re-measure from a re-publish. */
let published: Array<IElementSize> = [];

class TestResizeObserver {
  public readonly observe = jest.fn<(element: unknown) => void>();
  public readonly disconnect = jest.fn<() => void>();

  public constructor(public readonly notify: () => void) {
    observers.push(this);
  }
}

function TestComponent({ elementKey = 0, testId = "probe" }: { elementKey?: number; testId?: string }): ReactElement {
  const [ref, size] = useElementSize<HTMLDivElement>();

  if (size) {
    published.push(size);
  }

  return (
    <div key={elementKey} ref={ref} data-testid={testId}>
      {size ? `${size.width}x${size.height}` : "unmeasured"}
    </div>
  );
}

describe("useElementSize", () => {
  beforeEach(() => {
    laidOut = { width: 800, height: 600 };
    observers = [];
    sizes = new WeakMap();
    published = [];

    jest.spyOn(HTMLElement.prototype, "clientWidth", "get").mockImplementation(function (this: HTMLElement) {
      return (sizes.get(this) ?? laidOut).width;
    });
    jest.spyOn(HTMLElement.prototype, "clientHeight", "get").mockImplementation(function (this: HTMLElement) {
      return (sizes.get(this) ?? laidOut).height;
    });

    global.ResizeObserver = TestResizeObserver as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    jest.restoreAllMocks();
    global.ResizeObserver = ORIGINAL_RESIZE_OBSERVER;
  });

  it("measures the element before anything is painted against it", () => {
    const { getByTestId }: RenderResult = render(<TestComponent />);

    // Measured in a layout effect, so the size is there on the first painted frame rather than one frame late, which
    // is a picture placed against a box it does not have.
    expect(getByTestId("probe").textContent).toBe("800x600");
  });

  it("reports every later size the element takes", () => {
    const { getByTestId }: RenderResult = render(<TestComponent />);

    act(() => {
      laidOut = { width: 1200, height: 700 };
      observers[0].notify();
    });

    expect(getByTestId("probe").textContent).toBe("1200x700");
  });

  it("keeps the last real size when the element reports none", () => {
    const { getByTestId }: RenderResult = render(<TestComponent />);

    act(() => {
      laidOut = { width: 0, height: 0 };
      observers[0].notify();
    });

    // A closed panel is still observed and reports zero. Publishing that would leave the caller with nothing to lay
    // out against, and the element comes back the size it went away at.
    expect(getByTestId("probe").textContent).toBe("800x600");
  });

  it("publishes the same measurement as the same value", () => {
    render(<TestComponent />);

    act(() => observers[0].notify());

    // An observer fires for the observation itself and for either dimension. A fresh object each time would be a new
    // dependency for everything derived from the size, so an unchanged box has to stay the value it already was.
    expect(published.length).toBeGreaterThan(0);
    expect(new Set(published).size).toBe(1);
  });

  it("stops observing when the element goes away", () => {
    const { unmount }: RenderResult = render(<TestComponent />);

    expect(observers).toHaveLength(1);
    expect(observers[0].disconnect).not.toHaveBeenCalled();

    unmount();

    expect(observers[0].disconnect).toHaveBeenCalledTimes(1);
  });

  it("disconnects the old element and measures its replacement", () => {
    const { getByTestId, rerender, unmount } = render(<TestComponent />);
    const original: HTMLElement = getByTestId("probe");
    const originalObserver: TestResizeObserver = observers[0];

    expect(originalObserver.observe).toHaveBeenCalledWith(original);

    laidOut = { width: 1200, height: 700 };
    rerender(<TestComponent elementKey={1} />);

    const replacement: HTMLElement = getByTestId("probe");

    expect(replacement).not.toBe(original);
    expect(originalObserver.disconnect).toHaveBeenCalledTimes(1);
    expect(observers).toHaveLength(2);
    expect(observers[1].observe).toHaveBeenCalledWith(replacement);
    expect(replacement.textContent).toBe("1200x700");

    act(() => {
      sizes.set(replacement, { width: 900, height: 500 });
      observers[1].notify();
    });

    expect(replacement.textContent).toBe("900x500");

    unmount();

    expect(observers[1].disconnect).toHaveBeenCalledTimes(1);
    expect(originalObserver.disconnect).toHaveBeenCalledTimes(1);
  });

  it("resizes two instances independently and keeps the survivor observing", () => {
    const first = render(<TestComponent testId={"first"} />);
    const second = render(<TestComponent testId={"second"} />);
    const firstElement: HTMLElement = first.getByTestId("first");
    const secondElement: HTMLElement = second.getByTestId("second");

    expect(observers).toHaveLength(2);
    expect(observers[0].observe).toHaveBeenCalledWith(firstElement);
    expect(observers[1].observe).toHaveBeenCalledWith(secondElement);

    act(() => {
      sizes.set(firstElement, { width: 400, height: 300 });
      observers[0].notify();
    });

    expect(firstElement.textContent).toBe("400x300");
    expect(secondElement.textContent).toBe("800x600");

    act(() => {
      sizes.set(secondElement, { width: 1000, height: 700 });
      observers[1].notify();
    });

    expect(firstElement.textContent).toBe("400x300");
    expect(secondElement.textContent).toBe("1000x700");

    first.unmount();

    expect(observers[0].disconnect).toHaveBeenCalledTimes(1);
    expect(observers[1].disconnect).not.toHaveBeenCalled();

    act(() => {
      sizes.set(secondElement, { width: 600, height: 400 });
      observers[1].notify();
    });

    expect(secondElement.textContent).toBe("600x400");
    expect(observers).toHaveLength(2);

    second.unmount();

    expect(observers[1].disconnect).toHaveBeenCalledTimes(1);
  });
});
