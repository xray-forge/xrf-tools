import { Vector3d } from "@/core/bindings/types/xrf-db";
import {
  VisualBone,
  VisualBounds,
  VisualDescription,
  VisualDrawRange,
  VisualSection,
  VisualSubmesh,
} from "@/core/bindings/types/xrf-visual";
import { Nullable, Optional } from "@/lib/types/general";

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

/** One submesh's attributes as views over the shared buffer, plus the ranges that draw it. */
export interface IVisualSubmeshViews {
  index: number;
  label: string;
  positions: Float32Array;
  normals: Float32Array;
  uvs: Float32Array;
  indices: Uint16Array;
  /** Finest first, never empty. A submesh with one entry has no choice to offer. */
  levels: Array<IVisualSubmeshLevel>;
}

/** Segment endpoints of a skeleton, and which bones each segment joins. */
export interface IVisualSkeletonViews {
  positions: Nullable<Float32Array>;
  pairs: Nullable<Uint16Array>;
}

/** Everything the scene needs to build meshes, and nothing it does not. */
export interface IVisualModelViews {
  submeshes: Array<IVisualSubmeshViews>;
  fit: IVisualCameraFit;
  /**
   * Bind pose joints as line-segment endpoints, or null when the model carries no bind data.
   *
   * One pair of positions per bone that has a placed parent, ready for `LineSegments` without further arithmetic. The
   * positions are already in renderer space: the backend composed the chain and mirrored it the same way the mesh is,
   * so the skeleton sits inside the geometry rather than beside it.
   */
  skeleton: Nullable<Float32Array>;
  /**
   * Bone and parent index of each segment the skeleton draws, in the order `skeleton` lays them out.
   *
   * What lets a posed frame reuse the same buffer: a motion arrives as joint positions per bone, and these say which
   * two joints each drawn segment joins. Null exactly when `skeleton` is.
   */
  skeletonPairs: Nullable<Uint16Array>;
  vertexCount: number;
  /**
   * Longest collapse chain any submesh carries, which is how many distinct steps the detail control can reach.
   *
   * One means every submesh is static and there is nothing to decimate.
   */
  levelCount: number;
}

/**
 * Radius used when a model reports no usable extent, so a camera still has somewhere to stand.
 */
const FALLBACK_FIT_RADIUS: number = 1;

/**
 * Reads one coordinate triple, or null when any component is absent.
 *
 * Rust `f32` crosses as `number | null` because a non-finite float serialises to null, and such values
 * do occur: two visuals in the reference trees declare bounds of `f32::MAX`. Treating null as zero would
 * quietly place a model at the origin, so it is treated as no value at all.
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
 * Builds a typed array view over one packed section.
 *
 * Views rather than copies: the whole point of transferring one buffer is that the attributes are used
 * where they landed. Byte offsets are aligned by the packer, which is what makes these constructors
 * legal at all.
 *
 * @param buffer - Packed geometry buffer.
 * @param section - Byte range containing `f32` values.
 * @returns A view over the section without copying its bytes.
 */
function toFloatView(buffer: ArrayBuffer, section: VisualSection): Float32Array {
  return new Float32Array(buffer, section.byteOffset, section.byteLength / Float32Array.BYTES_PER_ELEMENT);
}

function toIndexView(buffer: ArrayBuffer, section: VisualSection): Uint16Array {
  return new Uint16Array(buffer, section.byteOffset, section.byteLength / Uint16Array.BYTES_PER_ELEMENT);
}

/**
 * Framing for a model, preferring what its geometry spans over what its header claims.
 *
 * Measured bounds are the honest ones. Declared bounds are the fallback for a model that produced no
 * geometry, so an empty viewport still frames where the model says it is.
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
 * Turn a description and its buffer into the views a scene uploads.
 *
 * Deliberately pure and free of three.js, because the offset arithmetic here is the riskiest code in the
 * viewer and the only kind of mistake that renders as a plausible but wrong mesh rather than as an
 * error. Keeping it a function means it is tested without a gpu.
 *
 * @param description - What the backend said the buffer contains.
 * @param buffer - The packed attribute bytes.
 * @returns Per submesh views, draw ranges and camera framing.
 */
export function createVisualViews(description: VisualDescription, buffer: ArrayBuffer): IVisualModelViews {
  if (buffer.byteLength !== description.bufferLength) {
    throw new Error(
      `Geometry buffer is ${buffer.byteLength} bytes but its description covers ${description.bufferLength}. ` +
        "The description and the buffer came from different reads."
    );
  }

  const skeleton: IVisualSkeletonViews = createVisualSkeleton(description.bones);
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
      positions: toFloatView(buffer, geometry.positions),
      normals: toFloatView(buffer, geometry.normals),
      uvs: toFloatView(buffer, geometry.uvs),
      indices: toIndexView(buffer, geometry.indices),
      levels,
    });
  }

  return {
    submeshes,
    fit: createVisualCameraFit(description),
    skeleton: skeleton.positions,
    skeletonPairs: skeleton.pairs,
    vertexCount,
    levelCount,
  };
}

/**
 * Turn a bone hierarchy into the line segments that draw it.
 *
 * A segment per bone that has a placed parent, so a root contributes nothing and a chain draws as a connected run.
 * Returns null rather than an empty buffer when nothing can be drawn - a model with no IK chunk, or a single bone with
 * no parent to reach - so the caller can tell "no skeleton to show" from "a skeleton of no bones".
 *
 * @param bones - Bones the backend reported, with composed bind positions.
 * @returns Segment endpoints for `LineSegments` and the bone pairs they join, both null when nothing is drawable.
 */
export function createVisualSkeleton(bones: Array<VisualBone>): IVisualSkeletonViews {
  const segments: Array<number> = [];
  const pairs: Array<number> = [];

  for (const [index, bone] of bones.entries()) {
    const parent: Optional<VisualBone> = bone.parentIndex === null ? undefined : bones[bone.parentIndex];

    if (!bone.bindPosition || !parent?.bindPosition || bone.parentIndex === null) {
      continue;
    }

    segments.push(
      bone.bindPosition.x ?? 0,
      bone.bindPosition.y ?? 0,
      bone.bindPosition.z ?? 0,
      parent.bindPosition.x ?? 0,
      parent.bindPosition.y ?? 0,
      parent.bindPosition.z ?? 0
    );
    pairs.push(index, bone.parentIndex);
  }

  return segments.length
    ? { positions: new Float32Array(segments), pairs: new Uint16Array(pairs) }
    : { positions: null, pairs: null };
}

/**
 * The range one submesh draws at a chosen point along its collapse chain.
 *
 * Detail is a fraction rather than a level index because an X-Ray slide-window table is one entry per edge collapse,
 * not a handful of authored LODs: a measured `stalker_bandit_1` carries 230 entries on one submesh and 948 on the
 * other, each step shedding about two triangles. A shared index would drive the first submesh to its coarsest while
 * the second was a quarter of the way down, so the same fraction of each chain is what keeps a model decimating
 * evenly — and what makes the setting mean the same thing across models with different chain lengths.
 *
 * @param submesh - Submesh whose range is being resolved.
 * @param detail - How far down the chain to go: 0 is full detail, 1 is the coarsest level the submesh has.
 * @returns The range to draw, never undefined because a packed submesh always has at least one level.
 */
export function getVisualSubmeshLevel(submesh: IVisualSubmeshViews, detail: number): IVisualSubmeshLevel {
  const coarsest: number = submesh.levels.length - 1;

  return submesh.levels[Math.round(Math.min(Math.max(detail, 0), 1) * coarsest)];
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
