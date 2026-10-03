import { saturate } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { Vector3d } from "@/core/ipc/types/xrf-math";
import {
  VisualBone,
  VisualBounds,
  VisualDescription,
  VisualDrawRange,
  VisualSubmesh,
} from "@/core/ipc/types/xrf-visual";
import { IRenderSurfaceDraw, OPAQUE_RENDER_SURFACE_DRAW } from "@/core/render/lib/surface/render-surface-draw";

/** Framing values a camera needs, derived from what the model actually spans. */
export interface IVisualCameraFit {
  center: [number, number, number];
  radius: number;
}

/** One drawable range of a submesh, already validated by the packer. */
export interface IVisualSubmeshLevel {
  start: number;
  count: number;
  triangleCount: number;
}

/** One submesh as the viewer reports it: its name, the ranges that draw it, and how its shader draws it. */
export interface IVisualSubmeshViews {
  index: number;
  label: string;
  /** Finest first, never empty. A submesh with one entry has no choice to offer. */
  levels: Array<IVisualSubmeshLevel>;
  /** The material state its shader compiles to: whether alpha is read, and how. */
  surface: IRenderSurfaceDraw;
}

/** What the viewer's controls and readouts need of a model; the renderer reads its geometry itself. */
export interface IVisualModelViews {
  submeshes: Array<IVisualSubmeshViews>;
  fit: IVisualCameraFit;
  /** Whether any bone carries a bind pose and a parent, which is what draws a skeleton. */
  hasSkeleton: boolean;
  vertexCount: number;
  /** Longest collapse chain any submesh carries, which is how many distinct steps the detail control can reach. */
  levelCount: number;
}

/**
 * Radius used when a model reports no usable extent, so a camera still has somewhere to stand.
 */
const FALLBACK_FIT_RADIUS: number = 1;

/**
 * Reads one coordinate triple, or null when any component is absent.
 *
 * @param vector - Coordinate triple received from the backend.
 * @returns Finite coordinates, or `null` when any component is absent or non-finite.
 */
function toFiniteTriple(vector: Vector3d): Nullable<[number, number, number]> {
  const { x, y, z } = vector;

  if (x === null || y === null || z === null) {
    return null;
  }

  return Number.isFinite(x) && Number.isFinite(y) && Number.isFinite(z) ? [x, y, z] : null;
}

/**
 * Framing for a model, preferring what its geometry spans over what its header claims.
 *
 * @param description - Packed visual description containing declared and computed bounds.
 * @returns Finite camera framing with a non-zero fallback radius.
 */
export function createVisualCameraFit(description: VisualDescription): IVisualCameraFit {
  const bounds: Nullable<VisualBounds> = description.computedBounds ?? description.declaredBounds ?? null;
  const center: Nullable<[number, number, number]> = bounds ? toFiniteTriple(bounds.boundingSphere.center) : null;
  const radius: Nullable<number> = bounds?.boundingSphere.radius ?? null;

  return {
    center: center ?? [0, 0, 0],
    radius: radius !== null && Number.isFinite(radius) && radius > 0 ? radius : FALLBACK_FIT_RADIUS,
  };
}

/**
 * Turn a description into what the viewer's controls and readouts read of it.
 *
 * @param description - What the backend said the model contains.
 * @param surfaces - Material state per submesh index, as `createVisualSurfaces` joined it. A submesh with no entry
 *   is drawn opaque, which is how a model opened without a shader library is drawn.
 * @returns Per submesh draw ranges and material states, and camera framing.
 */
export function createVisualViews(
  description: VisualDescription,
  surfaces: ReadonlyMap<number, IRenderSurfaceDraw> = new Map()
): IVisualModelViews {
  const submeshes: Array<IVisualSubmeshViews> = [];

  let vertexCount: number = 0;
  let levelCount: number = 0;

  for (const submesh of description.submeshes as Array<VisualSubmesh>) {
    if (submesh.content.kind !== "packed") {
      continue;
    }

    const { geometry } = submesh.content;
    const levels: Array<IVisualSubmeshLevel> = geometry.detailLevels.map((range: VisualDrawRange) => ({
      start: range.start,
      count: range.count,
      triangleCount: range.count / 3,
    }));

    vertexCount += geometry.vertexCount;
    levelCount = Math.max(levelCount, levels.length);

    submeshes.push({
      index: submesh.index,
      label: submesh.textureName ?? `submesh ${submesh.index}`,
      levels,
      surface: surfaces.get(submesh.index) ?? OPAQUE_RENDER_SURFACE_DRAW,
    });
  }

  return {
    submeshes,
    fit: createVisualCameraFit(description),
    hasSkeleton: description.bones.some(
      (bone: VisualBone) =>
        bone.bindTransform && bone.parentIndex !== null && description.bones[bone.parentIndex]?.bindTransform
    ),
    vertexCount,
    levelCount,
  };
}

/**
 * The range one submesh draws at a chosen point along its collapse chain.
 *
 * @param submesh - Submesh whose range is being resolved.
 * @param detail - How far down the chain to go: 0 is full detail, 1 is the coarsest level the submesh has.
 * @returns The range to draw, never undefined because a packed submesh always has at least one level.
 */
export function getVisualSubmeshLevel(submesh: IVisualSubmeshViews, detail: number): IVisualSubmeshLevel {
  const coarsest: number = submesh.levels.length - 1;

  return submesh.levels[Math.round(saturate(detail) * coarsest)];
}

/**
 * Triangles a model draws at a chosen detail fraction, summed over its submeshes.
 *
 * @param model - Views of the loaded model.
 * @param detail - How far down each collapse chain to go, 0 to 1.
 * @returns Total triangle count at that setting.
 */
export function countVisualTriangles(model: IVisualModelViews, detail: number): number {
  return model.submeshes.reduce(
    (total: number, submesh: IVisualSubmeshViews) => total + getVisualSubmeshLevel(submesh, detail).triangleCount,
    0
  );
}
