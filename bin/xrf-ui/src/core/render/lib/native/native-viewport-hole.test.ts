import { afterEach, describe, expect, it } from "@jest/globals";

import { NativeViewportHole, toOpaqueColor } from "@/core/render/lib/native/native-viewport-hole";
import { getWashStops } from "@/core/theme/surface";

/** A panel painted by the scheme shown, as the theme paints the shell, holding the viewport's element. */
function mockSchemedPage(): { panel: HTMLElement; element: HTMLElement } {
  const style: HTMLStyleElement = document.createElement("style");
  const panel: HTMLElement = document.createElement("div");
  const element: HTMLElement = document.createElement("div");

  style.textContent =
    '[data-color-scheme="light"] .panel { background-color: rgb(240, 240, 240); }\n' +
    '[data-color-scheme="dark"] .panel { background-color: rgb(20, 20, 20); }';
  panel.className = "panel";
  panel.appendChild(element);
  document.head.appendChild(style);
  document.body.appendChild(panel);
  document.documentElement.setAttribute("data-color-scheme", "dark");

  return { element, panel };
}

async function flush(): Promise<void> {
  await Promise.resolve();
}

describe("NativeViewportHole", () => {
  afterEach(() => {
    document.head.innerHTML = "";
    document.body.innerHTML = "";
    document.documentElement.removeAttribute("data-color-scheme");
  });

  // A theme switched while a viewport is open repaints the page's backdrop under it, not the colour it opened with.
  it("reads the page again when its colour scheme changes", async () => {
    const { element, panel } = mockSchemedPage();
    const hole: NativeViewportHole = new NativeViewportHole(element);

    expect(hole.getBackdrop(1).color).toEqual({ b: 20, g: 20, r: 20 });
    expect(panel.style.background).toBe("transparent");

    document.documentElement.setAttribute("data-color-scheme", "light");
    await flush();

    expect(hole.getBackdrop(1).color).toEqual({ b: 240, g: 240, r: 240 });
    expect(panel.style.background).toBe("transparent");

    hole.dispose();

    expect(panel.style.background).toBe("");
  });

  it("lays the theme's wash over the box of the ancestor painting it, in device pixels", () => {
    const panel: HTMLElement = document.createElement("div");
    const element: HTMLElement = document.createElement("div");

    panel.style.backgroundColor = "rgb(1, 2, 3)";
    panel.style.backgroundImage = "linear-gradient(135deg, red, blue)";
    panel.getBoundingClientRect = () => ({ bottom: 60, left: 10, right: 110, top: 20 }) as DOMRect;
    panel.appendChild(element);
    document.body.appendChild(panel);
    document.documentElement.setAttribute("data-color-scheme", "light");

    const hole: NativeViewportHole = new NativeViewportHole(element);
    const stops = getWashStops("light");

    expect(hole.getBackdrop(2)).toEqual({
      color: { b: 3, g: 2, r: 1 },
      wash: { angle: stops.angle, from: stops.first, rect: { height: 80, width: 200, x: 20, y: 40 }, to: stops.last },
    });

    hole.dispose();
  });
});

describe("toOpaqueColor", () => {
  it("reads a painted colour and refuses a transparent one", () => {
    expect(toOpaqueColor("rgb(1, 2, 3)")).toEqual({ b: 3, g: 2, r: 1 });
    expect(toOpaqueColor("rgba(1, 2, 3, 0.5)")).toEqual({ b: 3, g: 2, r: 1 });
    expect(toOpaqueColor("rgba(0, 0, 0, 0)")).toBeNull();
    expect(toOpaqueColor("transparent")).toBeNull();
  });
});
