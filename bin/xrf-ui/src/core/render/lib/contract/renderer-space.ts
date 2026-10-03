import { TVector3 } from "@xrf/math";

import { TRendererVector } from "@/core/render/lib/contract/renderer-vector";

/**
 * A direction or a point the engine states, in renderer space: its `z` negated, leaving zero as zero.
 *
 * @param vector - The vector in engine space.
 * @returns The same vector in renderer space.
 */
export function toRendererVector(vector: TVector3): TRendererVector {
  const [x, y, z] = vector;

  // So a readout at the origin says `0.0` rather than `-0.0`.
  return [x, y, z === 0 ? 0 : -z];
}

/**
 * The inverse of {@link toRendererVector}, which is the same flip.
 *
 * @param vector - The vector in renderer space.
 * @returns The same vector as the engine states it.
 */
export function toEngineVector(vector: TRendererVector): TVector3 {
  return toRendererVector(vector);
}
