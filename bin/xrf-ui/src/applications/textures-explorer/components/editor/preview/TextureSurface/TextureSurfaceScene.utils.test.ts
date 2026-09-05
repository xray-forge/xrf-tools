import { describe, expect, it } from "@jest/globals";
import { BufferAttribute, BufferGeometry, Vector3 } from "three";

import { ETextureSurfaceShape } from "@/applications/textures-explorer/lib/texture-surface";

import { createTextureSurfaceGeometry, toLightPosition } from "./TextureSurfaceScene.utils";

/**
 * The first vertex of the face the camera starts in front of, which is the one carrying the texture.
 *
 * @param geometry - Geometry to search.
 * @returns Index of a vertex whose normal points at the camera.
 */
function frontFaceVertex(geometry: BufferGeometry): number {
  const normals: BufferAttribute = geometry.getAttribute("normal") as BufferAttribute;

  for (let index = 0; index < normals.count; index += 1) {
    if (normals.getZ(index) === 1) {
      return index;
    }
  }

  throw new Error("The body has no face pointing at the camera.");
}

/**
 * Reads one vertex out of a named attribute.
 *
 * @param geometry - Geometry to read.
 * @param name - Attribute to read from.
 * @param index - Vertex to read.
 * @returns The vector at that vertex.
 */
function vectorAt(geometry: BufferGeometry, name: string, index: number): Vector3 {
  return new Vector3().fromBufferAttribute(geometry.getAttribute(name) as BufferAttribute, index);
}

describe("createTextureSurfaceGeometry", () => {
  it("should carry a tangent basis on every body", () => {
    for (const shape of Object.values(ETextureSurfaceShape)) {
      const geometry: BufferGeometry = createTextureSurfaceGeometry(shape);

      expect(geometry.getAttribute("xrayTangent").count).toBe(geometry.getAttribute("position").count);
      expect(geometry.getAttribute("xrayBinormal").count).toBe(geometry.getAttribute("position").count);
    }
  });

  it("should keep the basis unit length and square to the normal, poles included", () => {
    for (const shape of Object.values(ETextureSurfaceShape)) {
      const geometry: BufferGeometry = createTextureSurfaceGeometry(shape);

      for (let index = 0; index < geometry.getAttribute("position").count; index += 1) {
        const normal: Vector3 = vectorAt(geometry, "normal", index);
        const tangent: Vector3 = vectorAt(geometry, "xrayTangent", index);
        const binormal: Vector3 = vectorAt(geometry, "xrayBinormal", index);

        // A zero-length basis shades a vertex black, which is what a sphere's collapsed pole triangles would give.
        expect(tangent.length()).toBeCloseTo(1, 5);
        expect(binormal.length()).toBeCloseTo(1, 5);
        expect(tangent.dot(normal)).toBeCloseTo(0, 5);
        expect(binormal.dot(normal)).toBeCloseTo(0, 5);
      }
    }
  });

  it("should orient the flat body so u runs right and v runs down, as X-Ray stores rows", () => {
    const geometry: BufferGeometry = createTextureSurfaceGeometry(ETextureSurfaceShape.PLANE);
    const front: number = frontFaceVertex(geometry);
    const uvs: BufferAttribute = geometry.getAttribute("uv") as BufferAttribute;
    const position: Vector3 = vectorAt(geometry, "position", front);

    // Top-left of the file at the top-left corner of the face: three.js generates the opposite, and a texture drawn
    // that way is mirrored against the flat preview of the same file.
    expect(uvs.getX(front)).toBe(0);
    expect(uvs.getY(front)).toBe(0);
    expect(position.x).toBeLessThan(0);
    expect(position.y).toBeGreaterThan(0);

    const tangent: Vector3 = vectorAt(geometry, "xrayTangent", front);
    const binormal: Vector3 = vectorAt(geometry, "xrayBinormal", front);

    // u to the right and v downwards, which is the handedness a DirectX-era engine packs its normals against: get the
    // binormal's sign wrong here and every bumped surface is lit from the opposite side of its own detail.
    expect(tangent.x).toBeCloseTo(1, 5);
    expect(binormal.y).toBeCloseTo(-1, 5);
  });

  it("should give the flat body a depth, so turning it away shows an edge rather than nothing", () => {
    const geometry: BufferGeometry = createTextureSurfaceGeometry(ETextureSurfaceShape.PLANE);

    geometry.computeBoundingBox();

    const depth: number = geometry.boundingBox!.max.z - geometry.boundingBox!.min.z;

    expect(depth).toBeGreaterThan(0);
    // Thin enough that the face a person is reading is not competing with the sides of a box.
    expect(depth).toBeLessThan(0.2);
  });
});

describe("toLightPosition", () => {
  it("should put the light in front at the origin of both angles", () => {
    const position: Vector3 = toLightPosition(0, 0);

    expect(position.x).toBeCloseTo(0, 5);
    expect(position.y).toBeCloseTo(0, 5);
    expect(position.z).toBeGreaterThan(0);
  });

  it("should keep the light at one distance whichever way it is swung", () => {
    const distance: number = toLightPosition(0, 0).length();

    expect(toLightPosition(Math.PI / 3, Math.PI / 7).length()).toBeCloseTo(distance, 5);
    expect(toLightPosition(-2, -1).length()).toBeCloseTo(distance, 5);
  });
});
