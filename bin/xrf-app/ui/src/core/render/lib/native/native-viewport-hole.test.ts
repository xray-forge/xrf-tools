import { afterEach, describe, expect, it } from "@jest/globals";

import { NativeViewportHole } from "@/core/render/lib/native/native-viewport-hole";
import { flush } from "@/fixtures/utils/promise";

/** A panel painted by the scheme shown, as the theme paints the shell, holding the viewport's element. */
function mockSchemedPage(): { panel: HTMLElement; element: HTMLElement } {
  const style: HTMLStyleElement = document.createElement("style");
  const panel: HTMLElement = document.createElement("div");
  const element: HTMLElement = document.createElement("div");

  style.textContent =
    '[data-color-scheme="light"] .panel { background-color: rgb(240, 240, 240); }\n' +
    '[data-color-scheme="dark"] .panel { background-color: rgb(20, 20, 20); }';
  panel.className = "panel";
  panel.style.background = "red";
  panel.appendChild(element);
  document.head.appendChild(style);
  document.body.appendChild(panel);
  document.documentElement.setAttribute("data-color-scheme", "dark");

  return { element, panel };
}

describe("NativeViewportHole", () => {
  afterEach(() => {
    document.head.innerHTML = "";
    document.body.innerHTML = "";
    document.documentElement.removeAttribute("data-color-scheme");
  });

  // Until the renderer has drawn the window it is bare, black while the renderer starts its device.
  it("tells the planes at once but keeps the page painted until opened, and puts it back when disposed", () => {
    const { element, panel } = mockSchemedPage();
    const hole: NativeViewportHole = new NativeViewportHole(element);

    expect(hole.getBackdrop(1).planes.map((it) => it.color)).toEqual([{ b: 0, g: 0, r: 255 }]);
    expect(panel.style.background).toBe("red");

    hole.open();
    hole.open();

    expect(panel.style.background).toBe("transparent");

    hole.dispose();

    expect(panel.style.background).toBe("red");
  });

  // A theme switched while a viewport is open repaints the page's backdrop under it, not the colour it opened with.
  it("reads the page again when its colour scheme changes, staying open", async () => {
    const { element, panel } = mockSchemedPage();

    panel.style.background = "";

    const hole: NativeViewportHole = new NativeViewportHole(element);

    hole.open();

    expect(hole.getBackdrop(1).planes[0].color).toEqual({ b: 20, g: 20, r: 20 });

    document.documentElement.setAttribute("data-color-scheme", "light");
    await flush();

    expect(hole.getBackdrop(1).planes[0].color).toEqual({ b: 240, g: 240, r: 240 });
    expect(panel.style.background).toBe("transparent");

    hole.dispose();

    expect(panel.style.background).toBe("");
  });
});
