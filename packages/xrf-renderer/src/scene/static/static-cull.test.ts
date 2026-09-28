import { describe, expect, it, jest } from "@jest/globals";
import { ComputeNode, DepthTexture, Mesh, PerspectiveCamera, Scene, Vector4, WebGPURenderer } from "three/webgpu";

import { DEFAULT_RENDERER_TREE_WIND } from "#/lighting/default-lighting";
import { StaticCull } from "#/scene/static/static-cull";
import { IStaticPools } from "#/scene/static/static-pools";
import { LIGHT_SHADOW_FACE_BUDGET } from "#/uniforms/lights-uniforms";
import { STATIC_LIGHT_VIEW_START, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";
import { StorageRetirement } from "#/uniforms/storage-retirement";
import { TreeWindUniforms } from "#/uniforms/tree-wind-uniforms";
import { CullView } from "#/visibility/cull-view";
import { IShadowFrustum } from "#/visibility/shadow-frustum";

const VIEW_PASSES: number = 3;

interface IPoolsStub extends IStaticPools {
  version: number;
  candidateExtent: number;
  isSwaying: boolean;
}

function createCull(): {
  buffers: StaticDrawBuffers;
  cull: StaticCull;
  early: Scene;
  pools: IPoolsStub;
  renderer: WebGPURenderer;
  rendered: Array<Scene>;
  submissions: Array<Array<ComputeNode>>;
  wind: TreeWindUniforms;
} {
  const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), {
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
    candidateExtent: 1,
    clusterExtent: 1,
    flush: () => {},
    isSwaying: false,
    lodExtent: 1,
    rowExtent: 1,
    version: 0,
  };
  const wind: TreeWindUniforms = new TreeWindUniforms();
  const early: Scene = new Scene();
  const cull: StaticCull = new StaticCull(buffers, pools, wind, early, new Scene());
  const submissions: Array<Array<ComputeNode>> = [];
  const rendered: Array<Scene> = [];
  // The tests record dispatches and draws; WebGPU execution and the resulting lists are checked in the app.
  const renderer = {
    compute: (nodes: ComputeNode | Array<ComputeNode>): void => {
      submissions.push(Array.isArray(nodes) ? [...nodes] : [nodes]);
    },
    render: (scene: Scene): void => void rendered.push(scene),
  } as unknown as WebGPURenderer;

  return { buffers, cull, early, pools, renderer, rendered, submissions, wind };
}

/**
 * @returns A camera standing still until moved, its view, and what draws a frame of it the way the frame's passes do
 *   and says whether its first cull ran.
 */
function createFrames(
  cull: StaticCull,
  renderer: WebGPURenderer,
  submissions: ReadonlyArray<Array<ComputeNode>>
): { camera: PerspectiveCamera; view: CullView; frame: () => boolean } {
  const camera: PerspectiveCamera = new PerspectiveCamera();
  const view: CullView = new CullView();
  const depth: DepthTexture = new DepthTexture(4, 4);

  function frame(): boolean {
    cull.take(view, camera);

    const before: number = submissions.length;

    cull.dispatch(renderer);

    const isCulled: boolean = submissions.length > before;

    cull.cullLate(renderer, depth, 4, 4);
    cull.finish(renderer, depth, 4, 4);

    return isCulled;
  }

  camera.updateMatrixWorld();
  view.take(camera);

  return { camera, frame, view };
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

describe("StaticCull occlusion", () => {
  // What the first phase kept last was kept against the depth, so a view that stands still is culled again.
  it("culls the view again, occluding nothing, once what the depth hides is no longer culled", () => {
    const { buffers, cull, renderer, submissions } = createCull();

    buffers.occlusion.previous.isTaken.value = 1;
    cull.setOccluding(false);
    cull.dispatch(renderer);
    cull.setOccluding(false);
    cull.dispatch(renderer);

    expect(buffers.occlusion.previous.isTaken.value).toBe(0);
    expect(submissions).toHaveLength(1);
    cull.dispose();
  });

  // The last cull before the camera stops tests the depth of the pose before: it is culled once more against its own.
  it("culls a view that stopped once more against its own depth, and then not again", () => {
    const { cull, renderer, submissions } = createCull();
    const { camera, frame, view } = createFrames(cull, renderer, submissions);
    const still: Array<boolean> = [frame(), frame(), frame()];

    camera.position.x += 1;
    camera.updateMatrixWorld();
    view.take(camera);

    expect([...still, frame(), frame(), frame()]).toEqual([true, true, false, true, true, false]);
    cull.dispose();
  });

  // The depth holds the static draws alone, which stand still while nothing they list changes: a plain draw walking
  // past a still camera never hid anything, so its lists stay right however long it stands.
  it("keeps a still view's cull however long it stands while the trees stand still", () => {
    const { cull, pools, renderer, submissions, wind } = createCull();
    const { frame } = createFrames(cull, renderer, submissions);

    pools.isSwaying = true;
    wind.take({ ...DEFAULT_RENDERER_TREE_WIND, amplitude: 0 });

    expect(Array.from({ length: 6 }, frame)).toEqual([true, true, false, false, false, false]);
    cull.dispose();
  });

  // A swaying tree moves the depth it wrote with no version saying so, as it moves a light face's map.
  it("culls a still view every frame while the wind sways a batch it draws, and stops once the wind drops", () => {
    const { cull, pools, renderer, submissions, wind } = createCull();
    const { frame } = createFrames(cull, renderer, submissions);

    pools.isSwaying = true;
    wind.take(DEFAULT_RENDERER_TREE_WIND);

    const windy: Array<boolean> = [frame(), frame(), frame()];

    wind.take(null);

    expect([...windy, frame(), frame()]).toEqual([true, true, true, false, false]);
    cull.dispose();
  });

  it("keeps a still view's cull in the wind while no batch it draws sways", () => {
    const { cull, renderer, submissions, wind } = createCull();
    const { frame } = createFrames(cull, renderer, submissions);

    wind.take(DEFAULT_RENDERER_TREE_WIND);

    expect([frame(), frame(), frame()]).toEqual([true, true, false]);
    cull.dispose();
  });

  it("draws the first phase's batches, and nothing while none stands", () => {
    const { cull, early, renderer, rendered } = createCull();
    const camera: PerspectiveCamera = new PerspectiveCamera();

    cull.drawEarly(renderer, camera);
    early.add(new Mesh());
    cull.drawEarly(renderer, camera);

    expect(rendered).toEqual([early]);
    cull.dispose();
  });

  it("runs the second phase only as far as the camera's regions reach", () => {
    const { cull, pools, renderer, submissions } = createCull();
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const view: CullView = new CullView();

    camera.updateMatrixWorld();
    view.take(camera);
    pools.candidateExtent = 3;
    cull.take(view, camera);
    cull.dispatch(renderer);
    cull.cullLate(renderer, new DepthTexture(4, 4), 4, 4);

    const [late] = submissions[submissions.length - 1];

    expect(late.count).toBe(3);
    cull.dispose();
  });
});
