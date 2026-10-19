import { afterEach, describe, expect, it } from "@jest/globals";

import { INativePagePlanes, readNativePagePlanes, toOpaqueColor } from "@/core/render/lib/native/native-page-planes";

/** Nests the elements, outermost first, under the document's body, each styled by its own CSS longhands. */
function mockNested(...styles: Array<Record<string, string>>): Array<HTMLElement> {
  const elements: Array<HTMLElement> = styles.map((css: Record<string, string>) => {
    const element: HTMLElement = document.createElement("div");

    for (const [property, value] of Object.entries(css)) {
      element.style.setProperty(property, value);
    }

    return element;
  });

  elements.reduce((parent: HTMLElement, child: HTMLElement) => parent.appendChild(child), document.body);

  return elements;
}

describe("readNativePagePlanes", () => {
  afterEach(() => {
    document.body.innerHTML = "";
  });

  // The shell's frame around a rounded, clipping box holding the content plane a viewer sits on.
  it("reads each painted ancestor, outermost first, clipped by the clipping ancestors around it", () => {
    const [frame, clip, content, element] = mockNested(
      { "background-color": "rgb(33, 33, 33)" },
      { "border-top-left-radius": "8px", "border-top-width": "1px", "overflow-x": "hidden" },
      { "background-attachment": "fixed", "background-color": "rgb(23, 23, 23)", "background-image": "none" },
      {}
    );
    const page: INativePagePlanes = readNativePagePlanes(element);

    expect(page.planes.map((it) => [it.element, it.color])).toEqual([
      [frame, { b: 33, g: 33, r: 33 }],
      [content, { b: 23, g: 23, r: 23 }],
    ]);
    expect(page.clips).toEqual([
      {
        borders: { bottom: 0, left: 0, right: 0, top: 1 },
        element: clip,
        radii: { bottomLeft: 0, bottomRight: 0, topLeft: 7, topRight: 0 },
      },
    ]);
    expect(page.planes[0].clips).toEqual([]);
    expect(page.planes[1].clips).toEqual(page.clips);
    expect(page.planes[1].isWashFixed).toBe(true);
  });

  it("shows the outer colour through a plane painting only an image, and clips no plane by itself", () => {
    const [, washed, element] = mockNested(
      { "background-color": "rgb(1, 2, 3)" },
      { "background-image": "linear-gradient(red, blue)", "overflow-y": "hidden" },
      {}
    );
    const page: INativePagePlanes = readNativePagePlanes(element);

    expect(page.planes[1]).toMatchObject({ clips: [], color: { b: 3, g: 2, r: 1 }, element: washed, isWashed: true });
    expect(page.clips.map((it) => it.element)).toEqual([washed]);
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
