/** Floats a rigid transform takes: its three basis columns, then its translation, as a bone's is stored. */
export const RIGID_TRANSFORM_FLOATS: number = 12;

/**
 * A rigid transform's inverse: its basis transposed, its translation turned back through it.
 *
 * @param transform - A rotation and translation, {@link RIGID_TRANSFORM_FLOATS} floats.
 * @returns What undoes it.
 */
export function invertRigidTransform(transform: ArrayLike<number>): Float32Array {
  const [ix, iy, iz, jx, jy, jz, kx, ky, kz, cx, cy, cz] = Array.from(transform);

  return new Float32Array([
    ix,
    jx,
    kx,
    iy,
    jy,
    ky,
    iz,
    jz,
    kz,
    -(ix * cx + iy * cy + iz * cz),
    -(jx * cx + jy * cy + jz * cz),
    -(kx * cx + ky * cy + kz * cz),
  ]);
}

/**
 * `outer` after `inner`: a point through `inner`, then through `outer`.
 *
 * @param outer - The transform applied second.
 * @param inner - The transform applied first.
 * @returns The two as one.
 */
export function composeRigidTransforms(outer: ArrayLike<number>, inner: ArrayLike<number>): Float32Array {
  const result: Float32Array = new Float32Array(RIGID_TRANSFORM_FLOATS);

  for (let axis: number = 0; axis < 3; axis += 1) {
    rotateByRigidTransform(outer, inner[axis * 3], inner[axis * 3 + 1], inner[axis * 3 + 2], result, axis * 3);
  }

  rotateByRigidTransform(outer, inner[9], inner[10], inner[11], result, 9);
  result[9] += outer[9];
  result[10] += outer[10];
  result[11] += outer[11];

  return result;
}

/**
 * A direction through a transform's basis, written into `out` so a loop over many allocates nothing.
 *
 * @param transform - The transform, whose translation a direction does not take.
 * @param x - The direction's first component.
 * @param y - Its second.
 * @param z - Its third.
 * @param out - Where the turned direction is written.
 * @param at - The offset in `out` it is written at.
 */
export function rotateByRigidTransform(
  transform: ArrayLike<number>,
  x: number,
  y: number,
  z: number,
  out: Float32Array,
  at: number
): void {
  out[at] = transform[0] * x + transform[3] * y + transform[6] * z;
  out[at + 1] = transform[1] * x + transform[4] * y + transform[7] * z;
  out[at + 2] = transform[2] * x + transform[5] * y + transform[8] * z;
}
