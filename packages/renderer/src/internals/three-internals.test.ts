import { describe, expect, it } from "@jest/globals";
import {
  BundleGroup,
  Camera,
  NodeFrame,
  NodeMaterialObserver,
  PerspectiveCamera,
  StorageBufferNode,
  WebGPURenderer,
} from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
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

  it("keeps a camera's reversed depth in a private field its getter reads, which its projection follows", () => {
    const camera: PerspectiveCamera = new PerspectiveCamera(90, 1, 1, 100);

    // `adoptRendererConventions` writes the field, since `reversedDepth` has no setter.
    expect(Object.getOwnPropertyDescriptor(camera, "_reversedDepth")?.value).toBe(false);
    expect(Object.getOwnPropertyDescriptor(Camera.prototype, "reversedDepth")?.get).toBeInstanceOf(Function);

    adoptRendererConventions(camera);

    // Reversed: the near plane maps to one and the far plane to zero.
    expect(camera.reversedDepth).toBe(true);
    expect(camera.projectionMatrix.elements[10]).toBeCloseTo(1 / 99);
    expect(camera.projectionMatrix.elements[14]).toBeCloseTo(100 / 99);
  });

  it("builds a reversed depth renderer and a storage buffer node", () => {
    // `RendererDevice.open` asks for reversed depth; `StaticDrawBuffers` shares one storage node per buffer.
    expect(new WebGPURenderer({ canvas: {} as HTMLCanvasElement, reversedDepthBuffer: true }).reversedDepthBuffer).toBe(
      true
    );
    expect(typeof StorageBufferNode).toBe("function");
  });
});
