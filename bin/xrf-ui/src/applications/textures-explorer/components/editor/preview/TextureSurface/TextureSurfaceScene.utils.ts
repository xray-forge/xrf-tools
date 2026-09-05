import { BoxGeometry, BufferAttribute, BufferGeometry, SphereGeometry, Vector2, Vector3 } from "three";

import { ETextureSurfaceShape } from "@/applications/textures-explorer/lib/texture-surface";
import { XRAY_BINORMAL_ATTRIBUTE, XRAY_TANGENT_ATTRIBUTE } from "@/core/visuals/lib/visual-bump";
import { Nullable } from "@/lib/types/general";

/** How large each body is drawn, chosen so all three frame alike under one camera fit. */
const SHAPE_EXTENT: number = 2;

/** How thick the flat body is, as a fraction of its extent: enough to see it turn, too little to read as a box. */
const SLAB_THICKNESS: number = 0.02;

/**
 * Builds the body a texture is laid on, with the tangent basis the X-Ray bump shader rotates through.
 *
 * @param shape - Body to build.
 * @returns Geometry carrying `xrayTangent` and `xrayBinormal` beside its normals and uvs.
 */
export function createTextureSurfaceGeometry(shape: ETextureSurfaceShape): BufferGeometry {
  return withXrayTangentBasis(createShapeGeometry(shape));
}

/**
 * Adds the tangent basis the bump shader expects, derived from the uvs the geometry already carries.
 *
 * Derived here rather than taken from three.js, which dropped `computeTangents` in favour of a mikktspace binding: a
 * wasm dependency is a heavy thing to add for three generated primitives, and the derivation itself is short. It is
 * Lengyel's: accumulate a per-triangle tangent weighted by the uv winding, orthogonalise it against the normal, and
 * take the handedness from whether the accumulated binormal agrees with the cross product.
 *
 * @param geometry - Indexed geometry with positions, normals and uvs.
 * @returns The same geometry, with `xrayTangent` and `xrayBinormal` added.
 */
export function withXrayTangentBasis(geometry: BufferGeometry): BufferGeometry {
  const positions: BufferAttribute = geometry.getAttribute("position") as BufferAttribute;
  const normals: BufferAttribute = geometry.getAttribute("normal") as BufferAttribute;
  const uvs: BufferAttribute = geometry.getAttribute("uv") as BufferAttribute;
  const indices: ReadonlyArray<number> = toTriangleIndices(geometry, positions.count);
  const count: number = positions.count;

  const accumulatedTangent: Array<Vector3> = createVectors(count);
  const accumulatedBinormal: Array<Vector3> = createVectors(count);

  const positionAt: [Vector3, Vector3, Vector3] = [new Vector3(), new Vector3(), new Vector3()];
  const uvAt: [Vector2, Vector2, Vector2] = [new Vector2(), new Vector2(), new Vector2()];

  for (let at = 0; at < indices.length; at += 3) {
    for (let corner = 0; corner < 3; corner += 1) {
      positionAt[corner].fromBufferAttribute(positions, indices[at + corner]);
      uvAt[corner].fromBufferAttribute(uvs, indices[at + corner]);
    }

    const edge1: Vector3 = positionAt[1].clone().sub(positionAt[0]);
    const edge2: Vector3 = positionAt[2].clone().sub(positionAt[0]);
    const deltaUv1: Vector2 = uvAt[1].clone().sub(uvAt[0]);
    const deltaUv2: Vector2 = uvAt[2].clone().sub(uvAt[0]);
    const determinant: number = deltaUv1.x * deltaUv2.y - deltaUv2.x * deltaUv1.y;

    // A triangle collapsed in uv space - the seam rows of a sphere are exactly this - carries no direction to add.
    if (determinant === 0) {
      continue;
    }

    const scale: number = 1 / determinant;
    const tangent: Vector3 = edge1.clone().multiplyScalar(deltaUv2.y).addScaledVector(edge2, -deltaUv1.y);
    const binormal: Vector3 = edge2.clone().multiplyScalar(deltaUv1.x).addScaledVector(edge1, -deltaUv2.x);

    for (let corner = 0; corner < 3; corner += 1) {
      accumulatedTangent[indices[at + corner]].addScaledVector(tangent, scale);
      accumulatedBinormal[indices[at + corner]].addScaledVector(binormal, scale);
    }
  }

  const tangents: Float32Array = new Float32Array(count * 3);
  const binormals: Float32Array = new Float32Array(count * 3);

  const normalAt: Vector3 = new Vector3();
  const tangentAt: Vector3 = new Vector3();
  const binormalAt: Vector3 = new Vector3();

  for (let index = 0; index < count; index += 1) {
    normalAt.fromBufferAttribute(normals, index);

    // Gram-Schmidt: the tangent lies in the surface, so whatever of it points along the normal is not tangent.
    tangentAt.copy(accumulatedTangent[index]).addScaledVector(normalAt, -normalAt.dot(accumulatedTangent[index]));

    if (tangentAt.lengthSq() === 0) {
      // Nothing was accumulated here, so any direction in the surface will do rather than a zero basis, which would
      // shade the vertex black.
      tangentAt.set(normalAt.z, normalAt.x, normalAt.y).cross(normalAt);
    }

    tangentAt.normalize();

    binormalAt.crossVectors(normalAt, tangentAt);

    // Negative where the uv winding is mirrored, which is what tells a flipped shell from a plain one.
    if (binormalAt.dot(accumulatedBinormal[index]) < 0) {
      binormalAt.negate();
    }

    tangents.set([tangentAt.x, tangentAt.y, tangentAt.z], index * 3);
    binormals.set([binormalAt.x, binormalAt.y, binormalAt.z], index * 3);
  }

  geometry.setAttribute(XRAY_TANGENT_ATTRIBUTE, new BufferAttribute(tangents, 3));
  geometry.setAttribute(XRAY_BINORMAL_ATTRIBUTE, new BufferAttribute(binormals, 3));

  return geometry;
}

/** The triangle corners of a geometry, indexed or not, so the accumulation reads one shape either way. */
function toTriangleIndices(geometry: BufferGeometry, count: number): ReadonlyArray<number> {
  const index: Nullable<BufferAttribute> = geometry.getIndex();

  return index ? Array.from(index.array) : Array.from({ length: count }, (_, at: number) => at);
}

/** One zero vector per vertex, to accumulate into. */
function createVectors(count: number): Array<Vector3> {
  return Array.from({ length: count }, () => new Vector3());
}

/**
 * Where a light sits for one pair of angles, at a fixed distance from the body.
 *
 * Spherical rather than a position, because what a person drags is a direction: the distance a directional light sits
 * at changes nothing it does, and letting the drag change it would only move the highlight for no stated reason.
 *
 * @param azimuth - Angle around the body, in radians.
 * @param elevation - Angle above its equator, in radians.
 * @returns Where to put the light.
 */
export function toLightPosition(azimuth: number, elevation: number): Vector3 {
  const radius: number = SHAPE_EXTENT * 3;

  return new Vector3(
    radius * Math.cos(elevation) * Math.sin(azimuth),
    radius * Math.sin(elevation),
    radius * Math.cos(elevation) * Math.cos(azimuth)
  );
}

/** The body itself, before it is given a tangent basis. */
function createShapeGeometry(shape: ETextureSurfaceShape): BufferGeometry {
  switch (shape) {
    case ETextureSurfaceShape.PLANE:
      return withTopFirstUvs(new BoxGeometry(SHAPE_EXTENT, SHAPE_EXTENT, SHAPE_EXTENT * SLAB_THICKNESS));

    case ETextureSurfaceShape.SPHERE:
      return withTopFirstUvs(new SphereGeometry(SHAPE_EXTENT / 2, 96, 64));

    case ETextureSurfaceShape.CUBE:
      return withTopFirstUvs(new BoxGeometry(SHAPE_EXTENT * 0.8, SHAPE_EXTENT * 0.8, SHAPE_EXTENT * 0.8));
  }
}

/**
 * Turns three.js' bottom-first uvs into the top-first ones X-Ray authors against.
 *
 * The texture loaders never flip a row: X-Ray stores images top first, and the compressed path could not flip one if
 * it wanted to. three.js generates uvs with `v = 0` at the bottom, so a body carrying them shows every texture upside
 * down against the flat preview of the same file, which is the one comparison this surface exists to support.
 *
 * @param geometry - Freshly generated geometry.
 * @returns The same geometry, with `v` mirrored.
 */
function withTopFirstUvs(geometry: BufferGeometry): BufferGeometry {
  const uvs: BufferAttribute = geometry.getAttribute("uv") as BufferAttribute;

  for (let index = 0; index < uvs.count; index += 1) {
    uvs.setY(index, 1 - uvs.getY(index));
  }

  uvs.needsUpdate = true;

  return geometry;
}
