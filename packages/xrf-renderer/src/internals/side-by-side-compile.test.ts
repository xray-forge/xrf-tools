import { describe, expect, it } from "@jest/globals";
import { WebGPURenderer } from "three/webgpu";

import { compileSideBySide } from "#/internals/side-by-side-compile";

interface IFakeNodes {
  getForRender(renderObject: unknown): unknown;
  getForCompute(computeNode: unknown): unknown;
  getForRenderAsync?: (renderObject: unknown) => Promise<unknown>;
  getForComputeAsync?: (computeNode: unknown) => Promise<unknown>;
}

function createRenderer(built: Array<string>): [WebGPURenderer, IFakeNodes] {
  const nodes: IFakeNodes = {
    getForCompute: (node: unknown): unknown => built.push(`compute:${node}`),
    getForRender: (object: unknown): unknown => built.push(`render:${object}`),
  };

  return [{ _nodes: nodes } as unknown as WebGPURenderer, nodes];
}

describe("compileSideBySide", () => {
  // Built part way while another build runs, a shader corrupts the node state the builds share.
  it("has three build each shader whole as a compile reaches it, and yield part way again after", async () => {
    const built: Array<string> = [];
    const [renderer, nodes] = createRenderer(built);
    const started: Array<string> = [];

    const compiled: Promise<void> = compileSideBySide(renderer, [
      async (): Promise<void> => {
        started.push("a");
        await nodes.getForRenderAsync?.("a");
      },
      async (): Promise<void> => {
        started.push("b");
        await nodes.getForComputeAsync?.("b");
      },
    ]);

    // Both started at once, each building as it asked, before either waited.
    expect(started).toEqual(["a", "b"]);
    expect(built).toEqual(["render:a", "compute:b"]);

    await compiled;

    expect("getForRenderAsync" in nodes).toBe(false);
    expect("getForComputeAsync" in nodes).toBe(false);
  });

  it("waits for every compile before failing with the first failure", async () => {
    const [renderer, nodes] = createRenderer([]);
    let isLateDone: boolean = false;

    const failure: unknown = await compileSideBySide(renderer, [
      () => Promise.reject(new Error("first")),
      async (): Promise<void> => {
        await Promise.resolve();
        isLateDone = true;
      },
    ]).catch((error: unknown) => error);

    expect(failure).toEqual(new Error("first"));
    expect(isLateDone).toBe(true);
    expect("getForRenderAsync" in nodes).toBe(false);
  });
});
