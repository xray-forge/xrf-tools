import { describe, expect, it, jest } from "@jest/globals";
import { BufferGeometry } from "three/webgpu";

import { ERendererOverlay } from "#/contract/scene/renderer-overlay";
import { IRendererSkeleton } from "#/contract/scene/renderer-skeleton";
import { SkeletonOverlay } from "#/scene/overlay/skeleton-overlay";
import { RendererSkeletons } from "#/scene/skeleton/renderer-skeletons";

function createChain(bones: number): IRendererSkeleton {
  return {
    binds: Float32Array.from(
      { length: bones * 12 },
      (_: unknown, at: number) => [1, 0, 0, 0, 1, 0, 0, 0, 1, Math.floor(at / 12), 0, 0][at % 12]
    ),
    pairs: Uint16Array.from({ length: (bones - 1) * 2 }, (_: unknown, at: number) =>
      at % 2 ? Math.floor(at / 2) : Math.floor(at / 2) + 1
    ),
  };
}

function createOverlay(skeletons: RendererSkeletons): SkeletonOverlay {
  return new SkeletonOverlay(
    { color: [1, 1, 1], isDepthTested: false, kind: ERendererOverlay.SKELETON, skeleton: "npc" },
    skeletons
  );
}

function toSegments(overlay: SkeletonOverlay): Array<number> {
  return Array.from(overlay.objects[0].geometry.getAttribute("position").array);
}

describe("SkeletonOverlay", () => {
  it("draws a skeleton put again, whatever its pose's version", () => {
    const skeletons: RendererSkeletons = new RendererSkeletons(() => {});
    const overlay: SkeletonOverlay = createOverlay(skeletons);
    const moved: IRendererSkeleton = createChain(2);

    skeletons.putSkeleton("npc", createChain(2));
    overlay.update();

    expect(toSegments(overlay)).toEqual([1, 0, 0, 0, 0, 0]);

    moved.binds[21] = 5;
    skeletons.putSkeleton("npc", moved);
    overlay.update();

    expect(toSegments(overlay)).toEqual([5, 0, 0, 0, 0, 0]);
  });

  it("lets its segments' geometry go for another count of segments", () => {
    const skeletons: RendererSkeletons = new RendererSkeletons(() => {});
    const overlay: SkeletonOverlay = createOverlay(skeletons);

    skeletons.putSkeleton("npc", createChain(2));
    overlay.update();

    const before: BufferGeometry = overlay.objects[0].geometry;
    const disposed = jest.fn();

    before.addEventListener("dispose", disposed);
    skeletons.putSkeleton("npc", createChain(3));
    overlay.update();

    expect(disposed).toHaveBeenCalledTimes(1);
    expect(overlay.objects[0].geometry).not.toBe(before);
    expect(toSegments(overlay)).toHaveLength(12);
  });
});
