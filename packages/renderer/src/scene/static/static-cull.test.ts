import { describe, expect, it, jest } from "@jest/globals";
import { ComputeNode, Scene, Vector4, WebGPURenderer } from "three/webgpu";

import { StaticCull } from "#/scene/static/static-cull";
import { IStaticPools } from "#/scene/static/static-pools";
import { LIGHT_SHADOW_FACE_BUDGET } from "#/uniforms/lights-uniforms";
import { EStaticPool, STATIC_LIGHT_VIEW_START, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { IShadowFrustum } from "#/visibility/shadow-frustum";

/** Passes a shadow view's cull dispatches: its arguments cleared, its single draws' clusters, then its rows'. */
const VIEW_PASSES: number = 3;

/** Pools of one of everything, whose version a test bumps as the draws would change. */
interface IPoolsStub extends IStaticPools {
  version: number;
}

function createCull(): {
  buffers: StaticDrawBuffers;
  cull: StaticCull;
  pools: IPoolsStub;
  renderer: WebGPURenderer;
  submissions: Array<Array<ComputeNode>>;
} {
  const buffers: StaticDrawBuffers = new StaticDrawBuffers({
    [EStaticPool.BATCHES]: 4,
    [EStaticPool.CLUSTERS]: 4,
    [EStaticPool.LODS]: 2,
    [EStaticPool.PLACES]: 4,
    [EStaticPool.PYRAMID]: 4,
    [EStaticPool.ROWS]: 4,
    [EStaticPool.SHADOW_LIST]: 4,
    [EStaticPool.SLOTS]: 4,
    [EStaticPool.SURFACE_LIST]: 4,
  });
  const pools: IPoolsStub = {
    batchExtent: 1,
    clusterExtent: 1,
    flush: () => {},
    lodExtent: 1,
    rowExtent: 1,
    version: 0,
  };
  const cull: StaticCull = new StaticCull(buffers, pools, new Scene());
  const submissions: Array<Array<ComputeNode>> = [];
  // The tests record dispatches; WebGPU execution and the resulting lists are checked in the app.
  const renderer = {
    compute: (nodes: ComputeNode | Array<ComputeNode>): void => {
      submissions.push(Array.isArray(nodes) ? [...nodes] : [nodes]);
    },
  } as unknown as WebGPURenderer;

  return { buffers, cull, pools, renderer, submissions };
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
    expect(submissions[0]).toHaveLength(VIEW_PASSES * faces.length);
    expect(new Set(submissions[0]).size).toBe(VIEW_PASSES * faces.length);
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

    expect(submissions.map((nodes) => nodes.length / VIEW_PASSES)).toEqual([2, 1, 1, 2]);
    cull.dispose();
  });

  it("reculls kept faces when the draws change, not when the camera moves: a face casts every tree at its finest", () => {
    const { buffers, cull, pools, renderer, submissions } = createCull();
    const faces: Array<IShadowFrustum> = [createFrustum(), createFrustum()];

    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    buffers.lod.camera.value.x += 10;
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    buffers.lod.glodStart.value += 1;
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);
    pools.version += 1;
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);

    expect(submissions.map((nodes) => nodes.length / VIEW_PASSES)).toEqual([2, 2]);
    cull.dispose();
  });

  it("reculls a kept cascade when the camera moves, which picks its trees' bands", () => {
    const { buffers, cull, renderer, submissions } = createCull();
    const cascade: Array<IShadowFrustum> = [createFrustum()];

    cull.cullViews(renderer, 0, cascade);
    cull.cullViews(renderer, 0, cascade);
    buffers.lod.camera.value.x += 10;
    cull.cullViews(renderer, 0, cascade);
    buffers.lod.glodStart.value += 1;
    cull.cullViews(renderer, 0, cascade);

    expect(submissions.map((nodes) => nodes.length / VIEW_PASSES)).toEqual([1, 1, 1]);
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
    buffers.grow(EStaticPool.CLUSTERS, 8);
    cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces);

    expect(retired).toHaveBeenCalledTimes(2 * VIEW_PASSES);
    expect(submissions).toHaveLength(2);
    expect(submissions[1].every((node) => !submissions[0].includes(node))).toBe(true);
    submissions[1].forEach((node: ComputeNode) => node.addEventListener("dispose", disposed));
    cull.dispose();
    expect(disposed).toHaveBeenCalledTimes(2 * VIEW_PASSES);
  });

  it("rejects a batch larger than the reserved slots before submitting any work", () => {
    const { cull, renderer, submissions } = createCull();
    const faces: Array<IShadowFrustum> = Array.from({ length: LIGHT_SHADOW_FACE_BUDGET + 1 }, () => createFrustum());

    expect(() => cull.cullViews(renderer, STATIC_LIGHT_VIEW_START, faces)).toThrow(RangeError);
    expect(submissions).toHaveLength(0);
    cull.dispose();
  });
});
