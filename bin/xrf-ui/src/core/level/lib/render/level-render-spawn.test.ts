import { describe, expect, it } from "@jest/globals";
import { ERendererDraw, IRendererGeometry } from "@xrf/renderer";

import { toPosedGeometry } from "@/core/level/lib/render/level-render-spawn";
import { IVisualSubmeshViews } from "@/core/visuals/lib/visual-views";

/** One vertex a metre and a half up, facing `+x`, hung entirely from bone zero. */
function createSubmesh(): IVisualSubmeshViews {
  return {
    binormals: new Float32Array([0, 0, 1]),
    index: 0,
    indices: new Uint16Array([0, 0, 0]),
    label: "lamp",
    levels: [{ count: 3, start: 0, triangleCount: 1 }],
    normals: new Float32Array([1, 0, 0]),
    positions: new Float32Array([0, 1.5, 0]),
    skinIndices: new Uint16Array([0, 0, 0, 0]),
    skinWeights: new Float32Array([1, 0, 0, 0]),
    surface: { draw: ERendererDraw.OPAQUE, isLit: true },
    tangents: new Float32Array([0, 1, 0]),
    uvs: new Float32Array([0, 0]),
  };
}

describe("toPosedGeometry", () => {
  it("moves a vertex from its bone's bind to its rest, and turns its directions with the bone", () => {
    // Bound a metre up; at rest a metre across too, and turned a quarter about `y`: `+x` to `-z`.
    const bind: Array<number> = [1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0];
    const rest: Array<number> = [0, 0, -1, 0, 1, 0, 1, 0, 0, 1, 1, 0];
    const geometry: IRendererGeometry = toPosedGeometry(createSubmesh(), new Float32Array(bind), rest);

    // Half a metre over the bone, which now stands at (1, 1, 0).
    expect(Array.from(geometry.position ?? [])).toEqual([1, 1.5, 0]);
    expect(Array.from(geometry.normal ?? []).map((it) => Math.round(it) + 0)).toEqual([0, 0, -1]);
    expect(Array.from(geometry.tangent ?? []).map((it) => Math.round(it) + 0)).toEqual([0, 1, 0]);
    expect(geometry.skinIndices).toBeUndefined();
  });

  it("stands a submesh as authored without a rest pose", () => {
    const geometry: IRendererGeometry = toPosedGeometry(createSubmesh(), null, null);

    expect(Array.from(geometry.position ?? [])).toEqual([0, 1.5, 0]);
  });
});
