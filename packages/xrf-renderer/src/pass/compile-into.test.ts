import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { Object3D, PerspectiveCamera, RenderTarget, WebGPURenderer } from "three/webgpu";

import { compileInto } from "#/pass/compile-into";

interface ICompileSeen {
  target: Nullable<RenderTarget>;
  depth: boolean;
  stencil: boolean;
}

function createRenderer(seen: Array<ICompileSeen>): WebGPURenderer {
  const renderer = {
    compileAsync: (): Promise<void> => {
      seen.push({ depth: renderer.depth, stencil: renderer.stencil, target: renderer.target });

      return Promise.resolve();
    },
    depth: true,
    getRenderTarget: (): Nullable<RenderTarget> => renderer.target,
    setRenderTarget: (target: Nullable<RenderTarget>): void => {
      renderer.target = target;
    },
    stencil: false,
    target: null as Nullable<RenderTarget>,
  };

  return renderer as unknown as WebGPURenderer;
}

describe("compileInto", () => {
  // Built with the renderer's depth, a pipeline for a target without one is one no draw into it uses.
  it("compiles with the target's depth and stencil, and leaves the renderer's and its target as they were", async () => {
    const seen: Array<ICompileSeen> = [];
    const renderer: WebGPURenderer = createRenderer(seen);
    const previous: RenderTarget = new RenderTarget();
    const target: RenderTarget = new RenderTarget(1, 1, { depthBuffer: false, stencilBuffer: true });

    renderer.setRenderTarget(previous);
    await compileInto(renderer, target, new Object3D(), new PerspectiveCamera());
    await compileInto(renderer, null, new Object3D(), new PerspectiveCamera());

    expect(seen).toEqual([
      { depth: false, stencil: true, target },
      { depth: true, stencil: false, target: null },
    ]);
    expect(renderer.depth).toBe(true);
    expect(renderer.stencil).toBe(false);
    expect(renderer.getRenderTarget()).toBe(previous);
  });
});
