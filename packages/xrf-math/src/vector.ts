/** Three components, in whichever space the caller holds them. */
export type TVector3 = readonly [number, number, number];

/**
 * @param a - A vector.
 * @param b - Another.
 * @returns Their sum.
 */
export function addVectors(a: TVector3, b: TVector3): [number, number, number] {
  return [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
}

/**
 * @param vector - A vector.
 * @param by - How much to scale it by.
 * @returns The vector, scaled.
 */
export function scaleVector(vector: TVector3, by: number): [number, number, number] {
  return [vector[0] * by, vector[1] * by, vector[2] * by];
}

/**
 * @param a - A vector.
 * @param b - Another.
 * @returns Their dot product.
 */
export function dotProduct(a: TVector3, b: TVector3): number {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

/**
 * @param a - A vector.
 * @param b - Another.
 * @returns Their cross product, `a` then `b`.
 */
export function crossProduct(a: TVector3, b: TVector3): [number, number, number] {
  return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
}

/**
 * @param vector - A vector.
 * @returns The vector at length one, or a zero vector unchanged, since it points nowhere.
 */
export function normalise(vector: TVector3): [number, number, number] {
  const length: number = Math.hypot(vector[0], vector[1], vector[2]) || 1;

  return [vector[0] / length, vector[1] / length, vector[2] / length];
}
