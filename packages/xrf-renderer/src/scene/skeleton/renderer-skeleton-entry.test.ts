import { describe, expect, it } from "@jest/globals";

import { IRendererMotion } from "#/contract/scene/renderer-motion";
import { RendererSkeletonEntry } from "#/scene/skeleton/renderer-skeleton-entry";

function toBone(height: number): Array<number> {
  return [1, 0, 0, 0, 1, 0, 0, 0, 1, 0, height, 0];
}

const MOTION: IRendererMotion = { floatsPerBone: 12, transforms: new Float32Array([...toBone(1), ...toBone(2)]) };

function toPosedHeight(frame: number): number {
  const entry: RendererSkeletonEntry = new RendererSkeletonEntry({ binds: new Float32Array(toBone(0)) });

  entry.pose(MOTION, { frame, hiddenBones: [], motion: "walk" });

  return entry.skeleton.bones[0].matrixWorld.elements[13];
}

describe("RendererSkeletonEntry", () => {
  it("stands its bones where the motion's frame puts them", () => {
    expect(toPosedHeight(0)).toBe(1);
    expect(toPosedHeight(1)).toBe(2);
  });

  it("shows the bind pose for a frame outside the motion: past it, before it, or between two", () => {
    expect([2, -1, 0.5].map(toPosedHeight)).toEqual([0, 0, 0]);
  });
});
