import { describe, expect, it } from "@jest/globals";
import { Camera, Object3D, PerspectiveCamera, Scene, WebGPURenderer } from "three/webgpu";

import { drawCleared } from "#/pass/cleared-draw";

interface ISeen {
  children: Array<Object3D>;
  camera: Camera;
  isSorted: boolean;
  clears: [boolean, boolean, boolean, boolean];
}

function createRenderer(seen: Array<ISeen>): WebGPURenderer {
  const renderer = {
    autoClear: false,
    autoClearColor: true,
    autoClearDepth: true,
    autoClearStencil: true,
    render: (scene: Scene, camera: Camera): void => {
      seen.push({
        camera,
        children: [...scene.children],
        clears: [renderer.autoClear, renderer.autoClearColor, renderer.autoClearDepth, renderer.autoClearStencil],
        isSorted: renderer.sortObjects,
      });
    },
    sortObjects: true,
  };

  return renderer as unknown as WebGPURenderer;
}

describe("drawCleared", () => {
  it("clears as the draw's pass begins what is asked alone, and leaves the renderer's clears as they were", () => {
    const seen: Array<ISeen> = [];
    const renderer: WebGPURenderer = createRenderer(seen);

    drawCleared(renderer, false, true, () => renderer.render(new Scene(), new PerspectiveCamera()));

    expect(seen[0].clears).toEqual([true, false, true, false]);
    expect([renderer.autoClear, renderer.autoClearColor, renderer.autoClearDepth, renderer.autoClearStencil]).toEqual([
      false,
      true,
      true,
      true,
    ]);
  });
});
