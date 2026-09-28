import { describe, expect, it } from "@jest/globals";
import { ComputeNode, DepthTexture, Vector4, WebGPURenderer } from "three/webgpu";

import { StaticDepthPyramid } from "#/scene/static/static-depth-pyramid";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";
import { StorageRetirement } from "#/uniforms/storage-retirement";

function createPyramid(): {
  buffers: StaticDrawBuffers;
  pyramid: StaticDepthPyramid;
  renderer: WebGPURenderer;
  builds: Array<Array<ComputeNode>>;
} {
  const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), { [EStaticPool.PYRAMID]: 4 });
  const builds: Array<Array<ComputeNode>> = [];
  // The tests record dispatches; the reduction itself is checked in the app.
  const renderer = {
    compute: (nodes: Array<ComputeNode>): void => void builds.push([...nodes]),
  } as unknown as WebGPURenderer;

  return { buffers, builds, pyramid: new StaticDepthPyramid(buffers), renderer };
}

function toLevels(buffers: StaticDrawBuffers): Array<Array<number>> {
  return buffers.occlusion.levels.slice(0, buffers.occlusion.levelCount.value).map((level: Vector4) => level.toArray());
}

describe("StaticDepthPyramid", () => {
  it("lays its levels out one after another, a quarter of the last each, down to a single texel", () => {
    const { buffers, builds, pyramid, renderer } = createPyramid();

    expect(pyramid.build(renderer, new DepthTexture(64, 16), 64, 16)).toBe(true);
    // Where each starts, its width and height, and the pixels a texel of it spans.
    expect(toLevels(buffers)).toEqual([
      [0, 16, 4, 4],
      [64, 4, 1, 16],
      [68, 1, 1, 64],
    ]);
    expect(builds[0].map((level: ComputeNode) => level.count)).toEqual([64, 4, 1]);
    expect(buffers.occlusion.size.value.toArray()).toEqual([64, 16]);
    expect(buffers.capacity(EStaticPool.PYRAMID)).toBe(69);
    pyramid.dispose();
  });

  it("lays itself out again only for a drawing of another size", () => {
    const { pyramid, renderer } = createPyramid();
    const depth: DepthTexture = new DepthTexture(16, 16);

    expect(pyramid.build(renderer, depth, 16, 16)).toBe(true);
    expect(pyramid.build(renderer, depth, 16, 16)).toBe(false);
    expect(pyramid.build(renderer, depth, 32, 16)).toBe(true);
    pyramid.dispose();
  });

  // A test of a larger rectangle is skipped, not wrong.
  it("leaves the levels past a buffer its limit kept short untested", () => {
    const { buffers, pyramid, renderer } = createPyramid();

    buffers.storageLimit = 66 * 4;
    pyramid.build(renderer, new DepthTexture(64, 16), 64, 16);

    expect(buffers.capacity(EStaticPool.PYRAMID)).toBe(66);
    expect(toLevels(buffers)).toEqual([[0, 16, 4, 4]]);
    pyramid.dispose();
  });
});
