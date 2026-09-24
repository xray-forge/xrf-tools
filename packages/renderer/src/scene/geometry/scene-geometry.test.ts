import { describe, expect, it } from "@jest/globals";
import { BufferAttribute } from "three/webgpu";

import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { EVertexAttribute } from "#/shader/vertex-attribute";

/** A triangle whose vertices come packed, its coordinate two shorts a vertex or a tree's four. */
function createPacked(uvComponents: number = 2): IRendererGeometry {
  return {
    groups: [],
    packed: {
      binormal: new Uint8Array(12),
      normal: new Uint8Array(12),
      tangent: new Uint8Array(12),
      uv: new Int16Array(3 * uvComponents),
      uv1: new Int16Array(6),
    },
    position: new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]),
  };
}

describe("SceneGeometry", () => {
  it("binds the packed vertex under its own names, directions as normalized bytes and coordinates as words", () => {
    const { buffer } = new SceneGeometry(createPacked());
    const normal = buffer.getAttribute(EVertexAttribute.PACKED_NORMAL) as BufferAttribute;
    const uv = buffer.getAttribute(EVertexAttribute.PACKED_UV) as BufferAttribute;

    expect([normal.itemSize, normal.normalized]).toEqual([4, true]);
    // Two shorts a word, so three passes them as they are rather than widening each to 32 bits.
    expect([uv.itemSize, uv.normalized, uv.array.constructor]).toEqual([1, false, Uint32Array]);
    expect(buffer.getAttribute(EVertexAttribute.PACKED_UV1).itemSize).toBe(1);
    // Three's float normal is neither carried nor made up from the faces.
    expect(buffer.hasAttribute("normal")).toBe(false);
  });

  it("takes a tree's coordinate as four shorts a vertex, two words, over the bytes that crossed", () => {
    const geometry: IRendererGeometry = createPacked(4);
    const uv = new SceneGeometry(geometry).buffer.getAttribute(EVertexAttribute.PACKED_UV) as BufferAttribute;

    expect(uv.itemSize).toBe(2);
    expect((uv.array as Uint32Array).buffer).toBe(geometry.packed?.uv?.buffer);
  });

  it("refuses a geometry carrying its vertices packed and as floats both", () => {
    expect(() => new SceneGeometry({ ...createPacked(), uv: new Float32Array(6) })).toThrow(/packed and as floats/);
  });
});
