import { describe, expect, it } from "@jest/globals";
import { BufferAttribute, BufferGeometry, Vector3 } from "three";

import { createTextureSurfaceGeometry, ETextureSurfaceShape, toLightPosition } from "./texture-surface.utils";

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

  it("should orient the plane so u runs right and v runs down, as X-Ray stores rows", () => {
    const geometry: BufferGeometry = createTextureSurfaceGeometry(ETextureSurfaceShape.PLANE);
    const uvs: BufferAttribute = geometry.getAttribute("uv") as BufferAttribute;

    // Top-left of the file at the top-left corner of the quad: three.js generates the opposite, and a texture drawn
    // that way is mirrored against the flat preview of the same file.
    expect(uvs.getY(0)).toBe(0);
    expect(vectorAt(geometry, "position", 0).y).toBeGreaterThan(0);

    const tangent: Vector3 = vectorAt(geometry, "xrayTangent", 0);
    const binormal: Vector3 = vectorAt(geometry, "xrayBinormal", 0);

    // u to the right and v downwards, which is the handedness a DirectX-era engine packs its normals against: get the
    // binormal's sign wrong here and every bumped surface is lit from the opposite side of its own detail.
    expect(tangent.x).toBeCloseTo(1, 5);
    expect(binormal.y).toBeCloseTo(-1, 5);
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
