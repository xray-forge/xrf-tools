import { describe, expect, it } from "@jest/globals";
import { BundleGroup, NodeFrame, NodeMaterialObserver, StorageBufferNode, WebGPURenderer } from "three/webgpu";

import { RenderObjectRefreshType } from "#/internals/render-object-refresh";
import { getRendererBackend, IRendererBackend } from "#/internals/renderer-backend";

/**
 * Every read of three the renderer makes past its types, pinned against the installed three: an upgrade that moves
 * one fails here, before it fails on the GPU.
 */
describe("three's internals, as the renderer reads them", () => {
  it("numbers its refresh types as the observer answers in them", () => {
    // `StaticDrawObserver.needsRefresh` answers in these.
    expect(RenderObjectRefreshType).toEqual({ FULL: 2, NONE: 0, SHARED: 1 });
  });

  it("decides a render object's refresh on the observer's prototype, from the object and the frame", () => {
    // `callBaseNeedsRefresh` calls it for every draw outside a recorded bundle.
    const { needsRefresh } = NodeMaterialObserver.prototype as unknown as { needsRefresh: unknown };

    expect(needsRefresh).toBeInstanceOf(Function);
    expect(needsRefresh).toHaveLength(2);
  });

  it("counts render calls on the frame and versions a bundle group each time it records", () => {
    const bundle: BundleGroup = new BundleGroup();
    const version: number = bundle.version;

    bundle.needsUpdate = true;

    // `StaticDrawObserver` refreshes fully once a recording and the shared groups once a render call.
    expect(bundle.version).toBe(version + 1);
    expect(new NodeFrame().renderId).toBe(0);
  });

  it("has a backend that says it is WebGPU and times queries by their uid", () => {
    // `getRendererBackend` and `RendererGpuTimings` read these.
    const backend: IRendererBackend = getRendererBackend(new WebGPURenderer({ canvas: {} as HTMLCanvasElement }));

    expect(backend.isWebGPUBackend).toBe(true);
    expect(backend.hasTimestampQuery).toBeInstanceOf(Function);
    expect(backend.getTimestamp).toBeInstanceOf(Function);
  });

  it("builds a reversed depth renderer and a storage buffer node", () => {
    // Reversed depth rests on the option; `StaticDrawBuffers` shares one storage node per buffer.
    expect(new WebGPURenderer({ canvas: {} as HTMLCanvasElement, reversedDepthBuffer: true }).reversedDepthBuffer).toBe(
      true
    );
    expect(typeof StorageBufferNode).toBe("function");
  });
});
