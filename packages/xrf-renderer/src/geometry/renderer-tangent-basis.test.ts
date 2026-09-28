import { describe, expect, it } from "@jest/globals";

import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { createRendererBox } from "#/geometry/renderer-box-geometry";
import { createRendererSphere } from "#/geometry/renderer-sphere-geometry";
import { withRendererTangentBasis } from "#/geometry/renderer-tangent-basis";

type TVector = [number, number, number];

function vectorAt(array: Float32Array, vertex: number): TVector {
  return [array[vertex * 3], array[vertex * 3 + 1], array[vertex * 3 + 2]];
}

function dot(a: TVector, b: TVector): number {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

const BODIES: ReadonlyArray<[string, IRendererGeometry]> = [
  ["box", withRendererTangentBasis(createRendererBox(2, 2, 0.04))],
  ["sphere", withRendererTangentBasis(createRendererSphere(1, 96, 64))],
];

describe("withRendererTangentBasis", () => {
  it.each(BODIES)("keeps the %s's basis unit length and square to the normal, poles included", (_, geometry) => {
    for (let vertex: number = 0; vertex < geometry.position.length / 3; vertex += 1) {
      const normal: TVector = vectorAt(geometry.normal!, vertex);
      const tangent: TVector = vectorAt(geometry.tangent!, vertex);
      const binormal: TVector = vectorAt(geometry.binormal!, vertex);

      // A zero-length basis shades a vertex black, which a sphere's collapsed pole triangles would give.
      expect(Math.sqrt(dot(tangent, tangent))).toBeCloseTo(1, 5);
      expect(Math.sqrt(dot(binormal, binormal))).toBeCloseTo(1, 5);
      expect(dot(tangent, normal)).toBeCloseTo(0, 5);
      expect(dot(binormal, normal)).toBeCloseTo(0, 5);
    }
  });

  it("runs u right and v down on a face looking at +z, as X-Ray stores rows and packs its normals", () => {
    const [, box] = BODIES[0];
    // The +z face is slot 4; its first vertex is the file's first texel.
    const front: number = box.index![box.groups[4].start];
    const corner: number = Math.min(front, box.index![box.groups[4].start + 2], box.index![box.groups[4].start + 1]);

    expect([box.uv![corner * 2], box.uv![corner * 2 + 1]]).toEqual([0, 0]);
    expect(box.position[corner * 3]).toBeLessThan(0);
    expect(box.position[corner * 3 + 1]).toBeGreaterThan(0);
    // Get the binormal's sign wrong and every bumped surface is lit from the far side of its own detail.
    expect(vectorAt(box.tangent!, corner)[0]).toBeCloseTo(1, 5);
    expect(vectorAt(box.binormal!, corner)[1]).toBeCloseTo(-1, 5);
  });

  // A vertex whose triangles are collapsed in uv sums no tangent; one with every component equal once gave a zero one.
  it.each([
    ["an equal-component normal", [1, 1, 1]],
    ["a normal along x", [1, 0, 0]],
  ])("finds a basis in the surface for %s that summed no tangent", (_: string, direction: Array<number>) => {
    const length: number = Math.hypot(direction[0], direction[1], direction[2]);
    const normal: TVector = [direction[0] / length, direction[1] / length, direction[2] / length];
    const geometry: IRendererGeometry = withRendererTangentBasis({
      groups: [],
      normal: new Float32Array([...normal, ...normal, ...normal]),
      position: new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]),
      uv: new Float32Array(6),
    });
    const tangent: TVector = vectorAt(geometry.tangent!, 0);
    const binormal: TVector = vectorAt(geometry.binormal!, 0);

    expect(Math.sqrt(dot(tangent, tangent))).toBeCloseTo(1, 5);
    expect(Math.sqrt(dot(binormal, binormal))).toBeCloseTo(1, 5);
    expect(dot(tangent, normal)).toBeCloseTo(0, 5);
  });

  it("refuses geometry with nothing to derive a basis from", () => {
    expect(() => withRendererTangentBasis({ groups: [], position: new Float32Array(9) })).toThrow();
  });
});
