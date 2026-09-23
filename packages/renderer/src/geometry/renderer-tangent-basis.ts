import { IRendererGeometry } from "#/contract/scene/renderer-geometry";

/** Three components of a vector. */
type TVector = [number, number, number];

/**
 * Adds the tangent basis a bump pair rotates through, derived from the uvs, for geometry that carries none.
 *
 * @param geometry - Geometry with positions, normals and uvs.
 * @returns The same geometry, with `tangent` and `binormal` set.
 */
export function withRendererTangentBasis(geometry: IRendererGeometry): IRendererGeometry {
  const { position, normal, uv } = geometry;

  if (!normal || !uv) {
    throw new Error("A tangent basis is derived from normals and uvs, and this geometry lacks one of them.");
  }

  const count: number = position.length / 3;
  const corners: ArrayLike<number> = geometry.index ?? Array.from({ length: count }, (_, at: number) => at);
  const tangentSum: Float64Array = new Float64Array(count * 3);
  const binormalSum: Float64Array = new Float64Array(count * 3);

  for (let at = 0; at + 2 < corners.length; at += 3) {
    const triangle: TVector = [corners[at], corners[at + 1], corners[at + 2]];
    const edge1: TVector = toVector((axis) => position[triangle[1] * 3 + axis] - position[triangle[0] * 3 + axis]);
    const edge2: TVector = toVector((axis) => position[triangle[2] * 3 + axis] - position[triangle[0] * 3 + axis]);
    const du1: number = uv[triangle[1] * 2] - uv[triangle[0] * 2];
    const dv1: number = uv[triangle[1] * 2 + 1] - uv[triangle[0] * 2 + 1];
    const du2: number = uv[triangle[2] * 2] - uv[triangle[0] * 2];
    const dv2: number = uv[triangle[2] * 2 + 1] - uv[triangle[0] * 2 + 1];
    const determinant: number = du1 * dv2 - du2 * dv1;

    // A triangle collapsed in uv space, as a sphere's pole rows are, carries no direction to add.
    if (determinant === 0) {
      continue;
    }

    for (const corner of triangle) {
      for (let axis = 0; axis < 3; axis += 1) {
        tangentSum[corner * 3 + axis] += (edge1[axis] * dv2 - edge2[axis] * dv1) / determinant;
        binormalSum[corner * 3 + axis] += (edge2[axis] * du1 - edge1[axis] * du2) / determinant;
      }
    }
  }

  const tangent: Float32Array = new Float32Array(count * 3);
  const binormal: Float32Array = new Float32Array(count * 3);

  for (let vertex = 0; vertex < count; vertex += 1) {
    const n: TVector = toVector((axis) => normal[vertex * 3 + axis]);
    const sum: TVector = toVector((axis) => tangentSum[vertex * 3 + axis]);
    const along: number = dot(n, sum);
    let t: TVector = toVector((axis) => sum[axis] - n[axis] * along);

    if (dot(t, t) === 0) {
      // Nothing accumulated here, so any direction in the surface will do rather than a zero basis.
      t = cross([n[2], n[0], n[1]], n);
    }

    t = normalise(t);

    let b: TVector = cross(n, t);

    if (
      dot(
        b,
        toVector((axis) => binormalSum[vertex * 3 + axis])
      ) < 0
    ) {
      b = toVector((axis) => -b[axis]);
    }

    tangent.set(t, vertex * 3);
    binormal.set(b, vertex * 3);
  }

  return { ...geometry, binormal, tangent };
}

function toVector(component: (axis: number) => number): TVector {
  return [component(0), component(1), component(2)];
}

function dot(a: TVector, b: TVector): number {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

function cross(a: TVector, b: TVector): TVector {
  return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
}

function normalise(vector: TVector): TVector {
  const length: number = Math.hypot(vector[0], vector[1], vector[2]) || 1;

  return [vector[0] / length, vector[1] / length, vector[2] / length];
}
