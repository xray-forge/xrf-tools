import { describe, expect, it, jest } from "@jest/globals";
import { ComputeNode, Scene, Vector4, WebGPURenderer } from "three/webgpu";

import { StaticCull } from "#/scene/static/static-cull";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { StaticLods } from "#/scene/static/static-lods";
import { StaticPlaces } from "#/scene/static/static-places";
import { LIGHT_SHADOW_FACE_BUDGET } from "#/uniforms/lights-uniforms";
import { EStaticPool, STATIC_LIGHT_VIEW_START, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { IShadowFrustum } from "#/visibility/shadow-frustum";

function createCull(): {
  buffers: StaticDrawBuffers;
  cull: StaticCull;
  pool: StaticDrawPool;
  renderer: WebGPURenderer;
  submissions: Array<Array<ComputeNode>>;
} {
  const buffers: StaticDrawBuffers = new StaticDrawBuffers({
    [EStaticPool.LODS]: 2,
    [EStaticPool.PLACES]: 4,
    [EStaticPool.PYRAMID]: 4,
    [EStaticPool.ROWS]: 4,
    [EStaticPool.SLOTS]: 4,
  });
  const pool: StaticDrawPool = new StaticDrawPool(buffers);
  const cull: StaticCull = new StaticCull(
    buffers,
    pool,
    new StaticPlaces(buffers),
    new StaticLods(buffers),
    new Scene()
  );
  const submissions: Array<Array<ComputeNode>> = [];
  // The tests record dispatches; WebGPU execution and the resulting instance lists are checked in the app.
  const renderer = {
    compute: (nodes: Array<ComputeNode>): void => {
      submissions.push([...nodes]);
    },
  } as unknown as WebGPURenderer;

  return { buffers, cull, pool, renderer, submissions };
}

function createFrustum(version: number = 1): IShadowFrustum {
  return { planes: Array.from({ length: 6 }, () => new Vector4(1, 0, 0, 1)), version };
}

describe("StaticCull shadow batches", () => {
  it("submits dirty faces together and retains unchanged results across frames and an empty batch", () => {
    const { cull, renderer, submissions } = createCull();
    const faces: Array<IShadowFrustum> = Array.from({ length: LIGHT_SHADOW_FACE_BUDGET }, () => createFrustum());

    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, []);
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);

    expect(submissions).toHaveLength(1);
    expect(submissions[0]).toHaveLength(2 * faces.length);
    expect(new Set(submissions[0]).size).toBe(2 * faces.length);
    cull.dispose();
  });

  it("invalidates a moved frustum or a replacement face even when its version matches the previous owner", () => {
    const { cull, renderer, submissions } = createCull();
    const moving = { ...createFrustum() };
    const faces: Array<IShadowFrustum> = [moving, createFrustum()];

    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    moving.version += 1;
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    faces[1] = createFrustum();
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, [faces[1], moving]);

    expect(submissions.map((nodes) => nodes.length)).toEqual([4, 2, 2, 4]);
    cull.dispose();
  });

  it("reculls retained faces when camera-dependent detail or resident draws change", () => {
    const { buffers, cull, pool, renderer, submissions } = createCull();
    const faces: Array<IShadowFrustum> = [createFrustum(), createFrustum()];

    pool.isEnabled = true;
    pool.allocate();
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    buffers.lod.camera.value.x += 10;
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    buffers.lod.glodStart.value += 1;
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    pool.release(0);
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);

    expect(submissions.map((nodes) => nodes.length)).toEqual([4, 4, 4, 4]);
    cull.dispose();
  });

  it("disposes shaders after growth, reculls into the new buffers and releases the replacement shaders", () => {
    const { buffers, cull, renderer, submissions } = createCull();
    const faces: Array<IShadowFrustum> = [createFrustum(), createFrustum()];
    const retired = jest.fn();
    const disposed = jest.fn();

    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    submissions[0].forEach((node: ComputeNode) => node.addEventListener("dispose", retired));
    buffers.grow(EStaticPool.SLOTS, 8);
    buffers.grow(EStaticPool.ROWS, 8);
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);

    expect(retired).toHaveBeenCalledTimes(4);
    expect(submissions).toHaveLength(2);
    expect(submissions[1].every((node) => !submissions[0].includes(node))).toBe(true);
    submissions[1].forEach((node: ComputeNode) => node.addEventListener("dispose", disposed));
    cull.dispose();
    expect(disposed).toHaveBeenCalledTimes(4);
  });

  it("rejects a batch larger than the reserved slots before submitting any work", () => {
    const { cull, renderer, submissions } = createCull();
    const faces: Array<IShadowFrustum> = Array.from({ length: LIGHT_SHADOW_FACE_BUDGET + 1 }, () => createFrustum());

    expect(() => cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces)).toThrow(RangeError);
    expect(submissions).toHaveLength(0);
    cull.dispose();
  });
});
