import { afterEach, describe, expect, it, jest } from "@jest/globals";

import { watchDevicePixelRatio } from "@/lib/dom/device-pixel-ratio";

interface IMockQuery {
  media: string;
  change: () => void;
  isStopped: boolean;
}

/** Answers every media query with one whose change is fired by hand, keeping each in order. */
function mockMatchMedia(): Array<IMockQuery> {
  const queries: Array<IMockQuery> = [];

  window.matchMedia = ((media: string): MediaQueryList => {
    const query: IMockQuery = { change: () => undefined, isStopped: false, media };

    queries.push(query);

    return {
      addEventListener: (_: string, listener: () => void) => {
        query.change = listener;
      },
      removeEventListener: () => {
        query.isStopped = true;
      },
    } as unknown as MediaQueryList;
  }) as typeof window.matchMedia;

  return queries;
}

function setDevicePixelRatio(ratio: number): void {
  Object.defineProperty(window, "devicePixelRatio", { configurable: true, value: ratio });
}

describe("watchDevicePixelRatio", () => {
  const { matchMedia } = window;

  afterEach(() => {
    window.matchMedia = matchMedia;
    setDevicePixelRatio(1);
  });

  it("tells each new ratio, watching for the next from the one it changed to, until stopped", () => {
    const queries: Array<IMockQuery> = mockMatchMedia();
    const listener = jest.fn<(ratio: number) => void>();

    setDevicePixelRatio(1);

    const stop: () => void = watchDevicePixelRatio(listener);

    setDevicePixelRatio(1.5);
    queries[0]?.change();

    expect(listener).toHaveBeenCalledWith(1.5);
    expect(queries.map(({ media }) => media)).toEqual(["(resolution: 1dppx)", "(resolution: 1.5dppx)"]);

    stop();

    expect(queries[1]?.isStopped).toBe(true);
  });

  it("tells nothing where the page cannot query its display", () => {
    window.matchMedia = undefined as unknown as typeof window.matchMedia;

    expect(() => watchDevicePixelRatio(() => undefined)()).not.toThrow();
  });
});
