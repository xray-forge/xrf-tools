import { describe, expect, it } from "@jest/globals";

import {
  getSharedCorners,
  ICssBox,
  ICssCorners,
  intersectBoxes,
  isClipping,
  isCovering,
  maxCorners,
  readBorderRadii,
  readBorderWidths,
  toPaddingBox,
  toPaddingRadii,
} from "@/lib/dom/css-box";

const ROUNDED: ICssCorners = { bottomLeft: 8, bottomRight: 8, topLeft: 8, topRight: 8 };

function styled(css: Record<string, string>): CSSStyleDeclaration {
  const element: HTMLElement = document.createElement("div");

  for (const [property, value] of Object.entries(css)) {
    element.style.setProperty(property, value);
  }

  return element.style;
}

describe("css-box", () => {
  it("reads border widths and radii from a style", () => {
    const style: CSSStyleDeclaration = styled({
      "border-bottom-left-radius": "4px",
      "border-left-width": "2px",
      "border-top-width": "1px",
      "border-top-left-radius": "8px",
    });

    expect(readBorderWidths(style)).toEqual({ bottom: 0, left: 2, right: 0, top: 1 });
    expect(readBorderRadii(style)).toEqual({ bottomLeft: 4, bottomRight: 0, topLeft: 8, topRight: 0 });
  });

  it("clips for every overflow but visible", () => {
    expect(isClipping(styled({ "overflow-x": "hidden" }))).toBe(true);
    expect(isClipping(styled({ "overflow-y": "auto" }))).toBe(true);
    expect(isClipping(styled({ "overflow-x": "visible", "overflow-y": "visible" }))).toBe(false);
    expect(isClipping(styled({}))).toBe(false);
  });

  // A bordered, rounded box clips inside its border, its inner corners rounded the border's width less.
  it("takes the padding box and its radii inside the borders", () => {
    const borders = { bottom: 1, left: 1, right: 1, top: 1 };

    expect(toPaddingBox({ bottom: 100, left: 10, right: 110, top: 0 }, borders)).toEqual({
      bottom: 99,
      left: 11,
      right: 109,
      top: 1,
    });
    expect(toPaddingRadii(ROUNDED, borders)).toEqual({ bottomLeft: 7, bottomRight: 7, topLeft: 7, topRight: 7 });
    expect(toPaddingRadii(ROUNDED, { ...borders, left: 10 })).toEqual({
      bottomLeft: 0,
      bottomRight: 7,
      topLeft: 0,
      topRight: 7,
    });
  });

  it("intersects boxes and tells one covering another within half a pixel", () => {
    const view: ICssBox = { bottom: 100, left: 0, right: 200, top: 0 };

    expect(intersectBoxes({ bottom: 150, left: -10, right: 100, top: 20 }, view)).toEqual({
      bottom: 100,
      left: 0,
      right: 100,
      top: 20,
    });
    expect(isCovering({ bottom: 99.6, left: 0.4, right: 200, top: 0 }, view)).toBe(true);
    expect(isCovering({ bottom: 100, left: 1, right: 200, top: 0 }, view)).toBe(false);
  });

  // Content beside a side panel reaches only the right corners of the rounded box holding both.
  it("shares only the corners an inner box reaches", () => {
    const outer: ICssBox = { bottom: 100, left: 0, right: 200, top: 0 };

    expect(getSharedCorners({ bottom: 100, left: 60, right: 200, top: 0 }, outer, ROUNDED)).toEqual({
      bottomLeft: 0,
      bottomRight: 8,
      topLeft: 0,
      topRight: 8,
    });
    expect(maxCorners(ROUNDED, { bottomLeft: 12, bottomRight: 0, topLeft: 0, topRight: 0 })).toEqual({
      ...ROUNDED,
      bottomLeft: 12,
    });
  });
});
