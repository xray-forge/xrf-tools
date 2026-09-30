import { Vector3d } from "@/core/ipc/types/xrf-math";

/**
 * A vector as the backend sends it, as three numbers the renderer takes.
 *
 * @param vector - The vector, whose non-finite components crossed as null.
 * @returns Its components, a missing one read as zero.
 */
export function toRenderVector(vector: Vector3d): [number, number, number] {
  return [vector.x ?? 0, vector.y ?? 0, vector.z ?? 0];
}
