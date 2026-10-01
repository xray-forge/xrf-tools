import { describe, expect, it } from "@jest/globals";
import { Camera, Object3D, PerspectiveCamera, Scene, WebGPURenderer } from "three/webgpu";

import { drawTogether } from "#/pass/drawn-together";

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

describe("drawTogether", () => {
  // Each render call is an encoder, a command buffer and a pass, freed only at the worker's next major collection.
  it("draws every part in one render call, unsorted and in order, and leaves them as they were", () => {
    const seen: Array<ISeen> = [];
    const renderer: WebGPURenderer = createRenderer(seen);
    const holder: Scene = new Scene();
    const parts: Array<Object3D> = [new Object3D(), new Scene(), new Scene()];
    const camera: PerspectiveCamera = new PerspectiveCamera();

    drawTogether(renderer, holder, parts, camera);

    expect(seen).toHaveLength(1);
    expect(seen[0].children).toEqual(parts);
    expect(seen[0].camera).toBe(camera);
    expect(seen[0].isSorted).toBe(false);
    expect(renderer.sortObjects).toBe(true);
    expect(holder.children).toEqual([]);
    expect(parts.map((part: Object3D) => part.parent)).toEqual([null, null, null]);
  });
});
