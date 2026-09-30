/// <reference types="node" />

import { readFileSync } from "node:fs";

import { describe, expect, it } from "@jest/globals";
import {
  BufferAttribute,
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

  it("keeps each drawn geometry's dispose handler and wireframe index on its record of geometries", () => {
    // `disposeSharingGeometry` takes the handler off, which frees through the first render object's cached list.
    const init: string = String(Object.getPrototypeOf(WebGPURenderer.prototype).init);
    const initGeometry: string = readThreeMethod("renderers/common/Geometries.js", "initGeometry");
    const getIndex: string = readThreeMethod("renderers/common/Geometries.js", "getIndex");

    expect(init).toContain("this._geometries = new Geometries(");
    expect(initGeometry).toContain("for ( const attribute of renderObject.getAttributes() )");
    expect(initGeometry).toContain("this._geometryDisposeListeners.set( geometry, onDispose )");
    expect(initGeometry).toContain("this.info.memory.geometries --");
    expect(getIndex).toContain("wireframes.set( geometry, wireframeAttribute )");
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
    // `compileInto` sets the target around the call alone, and the compile lane chains the calls.
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

  it("makes an attribute's buffer only for one its backend holds none for, sized by its array", () => {
    // `createStorageBuffer` makes an arena's buffer first, of the arena's size, over a one-element array.
    const create: string = readThreeMethod("renderers/webgpu/utils/WebGPUAttributeUtils.js", "createAttribute");

    expect(create).toContain("let buffer = bufferData.buffer;");
    expect(create).toContain("if ( buffer === undefined ) {");
    expect(create).toContain("const byteLength = array.byteLength;");
    expect(create).toContain("bufferData.buffer = buffer;");
  });

  it("makes a storage attribute's buffer for storage, vertices and copies both ways", () => {
    // `createStorageBuffer` asks for the same usage, which a growth's `copyBufferToBuffer` needs at both ends.
    const create: string = readThreeMethod("renderers/webgpu/WebGPUBackend.js", "createStorageAttribute");

    expect(create).toContain(
      "GPUBufferUsage.STORAGE | GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST"
    );
  });

  it("reads an attribute's array only to make its buffer, and again only once its version moves", () => {
    // `releaseStorageArray` lets the array go once made, and nothing marks such an attribute for upload after.
    const update: string = readThreeMethod("renderers/common/Attributes.js", "update");

    expect(update).toContain("if ( data.version === undefined ) {");
    expect(update).toContain("data.version < bufferAttribute.version || bufferAttribute.usage === DynamicDrawUsage");
  });

  it("keeps an attribute's count apart from its array, set as it is made", () => {
    // `releaseStorageArray` swaps the array for one element's, and whatever reads the count still reads the buffer's.
    const attribute: BufferAttribute = new BufferAttribute(new Uint32Array(8), 2);

    attribute.array = new Uint32Array(2);

    expect(attribute.count).toBe(4);
  });

  it("binds a storage buffer whole, from its attribute's backend data", () => {
    // An arena's buffer is larger than its attribute's array, and a grown one is found by the attribute alone.
    const bind: string = readThreeMethod("renderers/webgpu/utils/WebGPUBindingUtils.js", "createBindGroup");
    const uniforms: string = readThreeMethod("renderers/webgpu/nodes/WGSLNodeBuilder.js", "getUniforms");

    expect(bind).toContain("const buffer = backend.get( binding.attribute ).buffer;");
    expect(bind).toContain("resource: { buffer: buffer }");
    // A storage buffer's array has no length in its shader, whatever its node counts.
    expect(uniforms).toContain("bufferCount > 0 && uniform.type === 'buffer'");
  });

  it("names a buffer of no name of its own after its uniform's id, as its builder hands the uniform out", () => {
    // `adoptStableBufferNames` names such a buffer again by its order in the shader, recognising it by this name.
    const uniform: string = readThreeMethod("renderers/webgpu/nodes/WGSLNodeBuilder.js", "getUniformFromNode");

    expect(uniform).toContain("uniformNode.name = name ? name : 'NodeBuffer_' + uniformNode.id;");
  });

  it("sends a texture's bytes only as it first makes it or its version moves, and none while its source is not ready", () => {
    // `releaseTextureData` lets them go once it is up, and marks its source not ready for a texture made again.
    const update: string = readThreeMethod("renderers/common/Textures.js", "updateTexture");

    expect(update).toContain(
      "if ( textureData.initialized === true && textureData.version === texture.version ) return;"
    );
    expect(update).toContain("if ( texture.source.dataReady === true ) backend.updateTexture( texture, options );");
  });

  it("makes a texture it let go anew with nothing in it, a copy's destination, while its source is not ready", () => {
    // `SurfaceBatching.restore` has three make an evicted texture again, its bytes gone, then copies its layer into it.
    const update: string = readThreeMethod("renderers/common/Textures.js", "updateTexture");
    const destroy: string = readThreeMethod("renderers/common/Textures.js", "_destroyTexture");
    const create: string = readThreeMethod("renderers/webgpu/utils/WebGPUTextureUtils.js", "createTexture");

    expect(destroy).toContain("this.delete( texture );");
    expect(update).toContain(
      "if ( textureData.isDefaultTexture === undefined || textureData.isDefaultTexture === true ) {"
    );
    expect(update).toContain("backend.createTexture( texture, options );");
    expect(create).toContain(
      "let usage = GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC;"
    );
  });

  it("makes a binding's texture again as the binding is made, disposed or not", () => {
    // The resurrection trap: a binding cloned from a template keeps the texture it was built with, gone or not, and
    // three makes it again from its bytes, which a texture that let them go does not read.
    const create: string = readThreeMethod("renderers/common/Bindings.js", "_createBindings");

    expect(create).toContain("this.textures.updateTexture( binding.texture );");
  });

  it("gives a compile's render context the renderer's depth and stencil, where a draw gives it its target's", () => {
    // `compileInto` hands the compile the target's, or a target without a depth compiles a pipeline no draw uses.
    const compile: string = readThreeMethod("renderers/common/Renderer.js", "async compileAsync");
    const render: string = readThreeMethod("renderers/common/Renderer.js", "_renderScene");

    expect(compile).toContain("renderContext.depth = this.depth;");
    expect(compile).toContain("renderContext.stencil = this.stencil;");
    expect(render).toContain("renderContext.depth = renderTarget.depthBuffer;");
    expect(render).toContain("renderContext.stencil = renderTarget.stencilBuffer;");
  });

  it("builds a compile's shaders through the node manager's asynchronous builds, beside whole ones of the same state", () => {
    // `compileSideBySide` stands the whole builds in for the asynchronous ones while a batch compiles side by side.
    const render: string = readThreeMethod("renderers/common/Renderer.js", "async compileAsync");
    const compute: string = readThreeMethod("renderers/common/Renderer.js", "async compileComputeAsync");
    const forRender: string = readThreeMethod("renderers/common/nodes/NodeManager.js", "getForRenderAsync");
    const forCompute: string = readThreeMethod("renderers/common/nodes/NodeManager.js", "getForComputeAsync");

    expect(render).toContain("await this._nodes.getForRenderAsync( renderObject );");
    expect(compute).toContain("await nodes.getForComputeAsync( computeNode );");
    expect(forRender).toContain("const result = this.getForRender( renderObject, true );");
    expect(forCompute).toContain("const result = this.getForCompute( computeNode, true );");
    expect(readThreeMethod("renderers/common/nodes/NodeManager.js", "getForRender")).toContain(
      "getForRender( renderObject, useAsync = false )"
    );
    expect(readThreeMethod("renderers/common/nodes/NodeManager.js", "getForCompute")).toContain(
      "getForCompute( computeNode, useAsync = false )"
    );
  });

  it("makes a storage and an indirect attribute's buffer through the backend, with the usages it states", () => {
    // `adoptZeroedStorage` makes a marked attribute's buffer first, with the same usage, for three to find and keep.
    const storage: string = readThreeMethod("renderers/webgpu/WebGPUBackend.js", "createStorageAttribute");
    const indirect: string = readThreeMethod("renderers/webgpu/WebGPUBackend.js", "createIndirectStorageAttribute");
    const create: string = readThreeMethod("renderers/webgpu/utils/WebGPUAttributeUtils.js", "createAttribute");

    expect(storage).toContain(
      "GPUBufferUsage.STORAGE | GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST"
    );
    expect(indirect).toContain(
      "GPUBufferUsage.STORAGE | GPUBufferUsage.INDIRECT | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST"
    );
    expect(create).toContain("_bufferDescriptor.mappedAtCreation = true;");
    expect(create).toContain("if ( buffer === undefined ) {");
  });

  it("compiles compute kernels with each pipeline made asynchronously, which its types leave out", () => {
    // `compileComputeAsync` in `internals/compute-compile.ts` calls it for the frame's kernels and a staged build's.
    const compile: string = readThreeMethod("renderers/common/Renderer.js", "async compileComputeAsync");

    expect(compile).toContain("pipelines.getForCompute( computeNode, computeBindings, compilationPromises );");
    expect(compile).toContain("await Promise.all( compilationPromises );");
  });

  it("keys a render context by its target's attachments, so targets alike share their draws' render objects", () => {
    // `FullScreenDraw` compiles for its own target and draws into another of the same attachments with nothing built
    // again; a compile takes the context a top-level render takes, at the call depth that render draws at.
    const get: string = readThreeMethod("renderers/common/RenderContexts.js", "get");
    const compile: string = readThreeMethod("renderers/common/Renderer.js", "async compileAsync");

    expect(get).toContain(
      "attachmentState = `${ count }:${ format }:${ type }:${ renderTarget.samples }:${ renderTarget.depthBuffer }:${ renderTarget.stencilBuffer }`;"
    );
    expect(get).toContain("get( renderTarget = null, mrt = null, callDepth = 0 )");
    expect(compile).toContain("this._renderContexts.get( renderTarget, this._mrt );");
  });

  it("calls a `Fn` through a node holding its function, whose body builds its nodes only as the call is built", () => {
    // `isNodeReading` runs a body taking nothing itself, to see what the graph under it reads.
    const call: string = readThreeMethod("nodes/tsl/TSLCore.js", "call");
    const source: string = readFileSync(require.resolve("three/src/nodes/tsl/TSLCore.js"), "utf8");

    expect(source).toContain("this.isShaderCallNodeInternal = true;");
    expect(call).toContain("const jsFunc = shaderNode.jsFunc;");
  });
});
