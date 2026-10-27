import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { getWashStops, IWashStops } from "@xrf/ui/theme";

import { readNativePageBackdrop } from "@/core/render/lib/native/native-page-backdrop";
import { INativePageClip, INativePagePlane } from "@/core/render/lib/native/native-page-planes";
import { ICssBox } from "@/lib/dom/css-box";

const STOPS: IWashStops = getWashStops("dark");
const SQUARE = { bottomLeft: 0, bottomRight: 0, topLeft: 0, topRight: 0 };

function mockElement(box: ICssBox): HTMLElement {
  const element: HTMLElement = document.createElement("div");

  element.getBoundingClientRect = () => box as DOMRect;

  return element;
}

function mockPlane(box: ICssBox, gray: number, overrides: Partial<INativePagePlane> = {}): INativePagePlane {
  return {
    clips: [],
    color: { b: gray, g: gray, r: gray },
    element: mockElement(box),
    isWashFixed: false,
    isWashed: false,
    radii: SQUARE,
    ...overrides,
  };
}

describe("readNativePageBackdrop", () => {
  beforeEach(() => {
    Object.defineProperty(document.documentElement, "clientWidth", { configurable: true, value: 1024 });
    Object.defineProperty(document.documentElement, "clientHeight", { configurable: true, value: 768 });
  });

  afterEach(() => {
    Object.defineProperty(document.documentElement, "clientWidth", { configurable: true, value: 0 });
    Object.defineProperty(document.documentElement, "clientHeight", { configurable: true, value: 0 });
  });

  // The frame keeps its own colour around the content plane, which is rounded where it fills the bordered clip's
  // corners: beside an open side panel, the right ones only.
  it("paints each plane in its box, rounded at the corners it shares with a rounded clip", () => {
    const clip: INativePageClip = {
      borders: { bottom: 1, left: 1, right: 1, top: 1 },
      element: mockElement({ bottom: 740, left: 40, right: 990, top: 30 }),
      radii: { bottomLeft: 7, bottomRight: 7, topLeft: 7, topRight: 7 },
    };
    const frame: INativePagePlane = mockPlane({ bottom: 768, left: 0, right: 1024, top: 0 }, 33);
    const content: INativePagePlane = mockPlane({ bottom: 800, left: 241, right: 989, top: 31 }, 23, { clips: [clip] });

    expect(readNativePageBackdrop({ clips: [clip], planes: [frame, content] }, STOPS, 2)).toEqual({
      planes: [
        { color: frame.color, isWashed: false, radii: [0, 0, 0, 0], rect: { height: 1536, width: 2048, x: 0, y: 0 } },
        {
          color: content.color,
          isWashed: false,
          radii: [0, 14, 14, 0],
          rect: { height: 1416, width: 1496, x: 482, y: 62 },
        },
      ],
      wash: null,
    });
  });

  it("starts from the innermost plane filling the page, which hides the ones outside it", () => {
    const body: INativePagePlane = mockPlane({ bottom: 768, left: 0, right: 1024, top: 0 }, 10);
    const frame: INativePagePlane = mockPlane({ bottom: 768, left: 0, right: 1024, top: 0 }, 33);
    const content: INativePagePlane = mockPlane({ bottom: 700, left: 50, right: 1000, top: 30 }, 23);

    expect(
      readNativePageBackdrop({ clips: [], planes: [body, frame, content] }, STOPS, 1).planes.map((it) => it.color)
    ).toEqual([frame.color, content.color]);
  });

  // A fixed wash is laid over the page's viewport, as the shell's surfaces lay it; another over its own box.
  it("lays the wash over the innermost washed plane's box, or the page for a fixed one", () => {
    const frame: INativePagePlane = mockPlane({ bottom: 768, left: 0, right: 1024, top: 0 }, 33, {
      isWashFixed: true,
      isWashed: true,
    });
    const panel: INativePagePlane = mockPlane({ bottom: 60, left: 10, right: 110, top: 20 }, 23, { isWashed: true });
    const wash = { angle: STOPS.angle, from: STOPS.first, to: STOPS.last };

    expect(readNativePageBackdrop({ clips: [], planes: [frame] }, STOPS, 1).wash).toEqual({
      ...wash,
      rect: { height: 768, width: 1024, x: 0, y: 0 },
    });
    expect(readNativePageBackdrop({ clips: [], planes: [frame, panel] }, STOPS, 2).wash).toEqual({
      ...wash,
      rect: { height: 80, width: 200, x: 20, y: 40 },
    });
  });
});
