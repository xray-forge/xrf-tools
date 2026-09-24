import { describe, expect, it } from "@jest/globals";

import {
  IRendererImpostors,
  RENDERER_IMPOSTOR_CORNER_FLOATS,
  RENDERER_IMPOSTOR_CORNERS,
  RENDERER_IMPOSTOR_FACETS,
} from "#/contract/scene/renderer-impostors";
import { StaticLods } from "#/scene/static/static-lods";
import { STATIC_LOD_CORNER_COLUMNS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/** One impostor whose corners count up: each float is its corner's index times ten plus its own. */
function createImpostors(): IRendererImpostors {
  return {
    corners: Float32Array.from(
      { length: RENDERER_IMPOSTOR_CORNERS * RENDERER_IMPOSTOR_CORNER_FLOATS },
      (_, index: number) =>
        Math.floor(index / RENDERER_IMPOSTOR_CORNER_FLOATS) * 10 + (index % RENDERER_IMPOSTOR_CORNER_FLOATS)
    ),
    factors: new Float32Array([0.5]),
    normals: new Float32Array(RENDERER_IMPOSTOR_FACETS * 4),
    spheres: new Float32Array([1, 2, 3, 4]),
  };
}

describe("StaticLods", () => {
  it("writes a corner as two columns: position and hemi, then its coordinates and sun", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers();
    const lods: StaticLods = new StaticLods(buffers);
    const start: number = lods.allocate(1) as number;
    const corners = buffers.lodCorners.array as Float32Array;
    const at: number = (start * STATIC_LOD_CORNER_COLUMNS + 2 * 2) * 4;

    lods.write(start, createImpostors());

    // The third corner: position 20, 21, 22, u 23, v 24, hemi 25, sun 26.
    expect(Array.from(corners.subarray(at, at + 8))).toEqual([20, 21, 22, 25, 23, 24, 26, 0]);
    expect(Array.from((buffers.lodSpheres.array as Float32Array).subarray(start * 4, start * 4 + 4))).toEqual([
      1, 2, 3, 4,
    ]);
  });

  it("leaves a freed impostor with no sphere, for the LOD cull to pass over", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers();
    const lods: StaticLods = new StaticLods(buffers);
    const start: number = lods.allocate(1) as number;

    lods.write(start, createImpostors());
    lods.free(start, 1);

    expect((buffers.lodSpheres.array as Float32Array)[start * 4 + 3]).toBe(-1);
    expect(lods.use.used).toBe(0);
  });
});
