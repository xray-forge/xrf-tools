/**
 * @jest-environment jsdom
 */

import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera } from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

import { bindDragCursor } from "#/camera/drag-cursor";

function bind(): { controls: OrbitControls; element: HTMLElement; unbind: () => void } {
  const element: HTMLElement = document.createElement("div");
  const controls: OrbitControls = new OrbitControls(new PerspectiveCamera(), element);

  return { controls, element, unbind: bindDragCursor(controls, element) };
}

describe("bindDragCursor", () => {
  it("marks the drag and only the drag", () => {
    const { controls, element } = bind();

    expect(element.style.cursor).toBe("auto");

    controls.dispatchEvent({ type: "start" });

    expect(element.style.cursor).toBe("grabbing");

    controls.dispatchEvent({ type: "end" });

    expect(element.style.cursor).toBe("auto");
  });

  it("leaves nothing behind when a gesture starts and ends in one task", () => {
    const { controls, element } = bind();

    controls.dispatchEvent({ type: "start" });
    controls.dispatchEvent({ type: "end" });

    expect(element.style.cursor).toBe("auto");
  });

  it("stops answering once unbound, and gives back the cursor it was handed", () => {
    const { controls, element, unbind } = bind();

    controls.dispatchEvent({ type: "start" });

    expect(element.style.cursor).toBe("grabbing");

    unbind();

    expect(element.style.cursor).toBe("auto");

    controls.dispatchEvent({ type: "start" });

    expect(element.style.cursor).toBe("auto");
  });
});
