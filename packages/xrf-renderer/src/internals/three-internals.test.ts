/// <reference types="node" />

import { readFileSync } from "node:fs";

import { describe, expect, it } from "@jest/globals";
import {
  BundleGroup,
  Camera,
  NodeFrame,
  NodeMaterialObserver,
  Object3D,
  PerspectiveCamera,
  RenderTarget,
  StorageBufferAttribute,
  StorageBufferNode,
  WebGPUBackend,
  WebGPURenderer,
} from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
import { RenderObjectRefreshType } from "#/internals/render-object-refresh-type";
import { getRendererBackend, IRendererBackend } from "#/internals/renderer-backend";

function readThreeMethod(file: string, method: string): string {
  const source: string = readFileSync(require.resolve(`three/src/${file}`), "utf8");
  const start: number = source.indexOf(`\n\t${method}(`);

  return start < 0 ? "" : source.slice(start, source.indexOf("\n\t}\n", start));
}

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

  it("gives the render context each render call's camera, and a kept render object its camera only as it records", () => {
    // `adoptRenderCamera` reads the first, and makes up for the second on a bundle's replay.
    const source: string = String((WebGPURenderer.prototype as unknown as { _renderScene: unknown })._renderScene);

    expect(source).toContain("renderContext.camera = camera");
  });

  it("has a backend that says it is WebGPU and times queries by their uid", () => {
    // `getRendererBackend` and `RendererGpuTimings` read these.
    const backend: IRendererBackend = getRendererBackend(new WebGPURenderer({ canvas: {} as HTMLCanvasElement }));

    expect(backend.isWebGPUBackend).toBe(true);
    // `setRendererTimestamps` turns timing over by the flag three reads as every pass begins.
    expect(new WebGPURenderer({ canvas: {} as HTMLCanvasElement, trackTimestamp: true }).backend).toHaveProperty(
      "trackTimestamp",
      true
    );
    expect(backend.hasTimestampQuery).toBeInstanceOf(Function);
    expect(backend.getTimestamp).toBeInstanceOf(Function);
  });

  it("keeps the device it opened on its backend, which the limits, the pacing and the device's description read", () => {
    // `toStorageLimit`, `whenSubmittedWorkDone` and `RendererDevice.describe` read `backend.device`.
    const backend: object = getRendererBackend(new WebGPURenderer({ canvas: {} as HTMLCanvasElement }));
    const init: string = String((backend as { init?: unknown }).init);

    expect(backend).toHaveProperty("device", null);
    expect(init).toContain("this.device = device");
  });

  it("frees an attribute's GPU buffer through its backend, from the attribute alone, and knows one it never had", () => {
    // `destroyStorageAttribute` frees what a static draw pool's growth replaced.
    const backend: IRendererBackend = getRendererBackend(new WebGPURenderer({ canvas: {} as HTMLCanvasElement }));

    expect(backend.destroyAttribute).toBeInstanceOf(Function);
    expect(backend.destroyAttribute).toHaveLength(1);
    // An attribute never uploaded has no backend data, which `destroyStorageAttribute` checks first.
    expect(backend.has?.(new StorageBufferAttribute(new Float32Array(4), 4))).toBe(false);
  });

  it("keeps the record of every attribute it uploaded on the renderer, which frees a geometry's buffers", () => {
    // `destroyGeometryAttribute` deletes through it, as three's own geometry disposal does.
    const renderer: WebGPURenderer = new WebGPURenderer({ canvas: {} as HTMLCanvasElement });
    const init: string = String(Object.getPrototypeOf(WebGPURenderer.prototype).init);

    expect(renderer).toHaveProperty("_attributes", null);
    expect(init).toContain("this._attributes = new Attributes(backend");
    expect(init).toContain("new Geometries(this._attributes");
  });

  it("lets an object go by an event of its own, which is what its render objects are freed on", () => {
    // `disposeObject`: three's render objects keep an object's geometry until the object or its material is disposed.
    const object: Object3D = new Object3D();
    let events: number = 0;

    object.addEventListener("dispose" as never, () => (events += 1));
    (object as unknown as { dispose(): void }).dispose();

    expect(events).toBe(1);
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

  it("clears a target's depth on its first draw unless its data says it was cleared", () => {
    // `initPreservedDepthTarget` writes the flag this reads, for the targets whose depth another draw fills.
    const render = (WebGPURenderer.prototype as unknown as { _renderScene: () => void })._renderScene;

    expect(render.toString()).toContain("renderTargetData.depthInitialized !== true");
    expect(new WebGPURenderer({ canvas: {} as HTMLCanvasElement })).toHaveProperty("_textures");
  });

  it("frees every colour a target holds as it is sized or let go, and its depth only where it claimed it first", () => {
    // `RendererTargets.resize`, `ResolvedTarget` and `resizeBorrowedDepthTarget` size a target sharing a texture only
    // with its owner, before either is allocated; a borrowed depth stays its lender's.
    const destroy: string = readThreeMethod("renderers/common/Textures.js", "_destroyRenderTarget");
    const claim: unknown = Object.getOwnPropertyDescriptor(RenderTarget.prototype, "depthTexture")?.set;

    expect(destroy).toContain("this._destroyTexture( textures[ i ] )");
    expect(destroy).toContain("depthTexture.renderTarget === renderTarget");
    expect(String(RenderTarget.prototype.setSize)).toContain("this.dispose()");
    expect(String(claim)).toContain("current.renderTarget === null");
  });

  it("allocates every colour of a target allocated for the first time again, and keeps its pass views until resized", () => {
    // `RendererTargets.resize`: the water's target shares the frame's texture, so it joins or leaves with every
    // target over that texture freed, since a target left allocated keeps views of what the join reallocated.
    const update: string = readThreeMethod("renderers/common/Textures.js", "updateRenderTarget");
    const descriptor: string = String(
      (WebGPUBackend.prototype as unknown as { _getRenderPassDescriptor: unknown })._getRenderPassDescriptor
    );

    expect(update).toContain("if ( renderTargetData.width !== size.width || size.height !== renderTargetData.height )");
    expect(update).toContain("if ( textureNeedsUpdate ) texture.needsUpdate = true;");
    expect(descriptor).toContain("renderTargetData.width !== renderTarget.width");
  });

  it("stands a shared placeholder in for a texture with nothing to upload, and says so in its data", () => {
    // `copyTextures` copies nothing into or out of a texture in that state, and finds each end's own on the backend.
    const update: string = readThreeMethod("renderers/common/Textures.js", "updateTexture");

    expect(update).toContain("backend.createDefaultTexture( texture );");
    expect(update).toContain("textureData.isDefaultTexture = true;");
    expect(getRendererBackend(new WebGPURenderer({ canvas: {} as HTMLCanvasElement }))).toHaveProperty("get");
    expect((WebGPUBackend.prototype as unknown as { get: unknown }).get).toBeInstanceOf(Function);
  });

  it("takes the target a compile builds for as the compile starts, before it first waits on anything but its init", () => {
    // `RendererSceneCompiler` sets each pass's target around the call alone, and chains the calls.
    const compile: string = String(Object.getPrototypeOf(WebGPURenderer.prototype).compileAsync);
    const prefix: string = compile.slice(0, compile.indexOf("this._renderTarget"));

    expect(prefix.length).toBeGreaterThan(0);
    expect(prefix.match(/await /g)).toEqual(["await "]);
    expect(prefix).toContain("await this.init()");
  });

  it("builds a reversed depth renderer and a storage buffer node", () => {
    // `RendererDevice.open` asks for reversed depth; `StaticDrawBuffers` shares one storage node per buffer.
    expect(new WebGPURenderer({ canvas: {} as HTMLCanvasElement, reversedDepthBuffer: true }).reversedDepthBuffer).toBe(
      true
    );
    expect(typeof StorageBufferNode).toBe("function");
  });
});
