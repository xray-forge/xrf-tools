import { describe, expect, it } from "@jest/globals";
import { IRendererObject } from "@xrf/renderer";

import { LEVEL_RENDER_KEYS } from "@/core/level/lib/render/level-render-keys";
import { toLevelImpostorObject } from "@/core/level/lib/render/level-render-sector";
import { ISectorImpostorViews } from "@/core/level/lib/sector/level-sector-views";
import { getLevelSurfaceRender } from "@/core/level/lib/surface/level-surface-render";
import { mockSectorSurface } from "@/fixtures/mocks/level.mocks";

/** Three impostors of one sector, the last two drawn with a second surface. */
function createImpostors(): ISectorImpostorViews {
  const surface = mockSectorSurface({ shaderId: 4, shaderName: "details\\lod", textureName: "lod\\level_lods" });

  return {
    corners: new Float32Array(3 * 32 * 8),
    count: 3,
    factors: new Float32Array([0.5, 0.5, 0.5]),
    groups: [
      { count: 1, render: getLevelSurfaceRender([], 3), start: 0, surface: { ...surface, shaderId: 3 } },
      { count: 2, render: getLevelSurfaceRender([], 4), start: 1, surface },
    ],
    normals: new Float32Array(3 * 8 * 4),
    spheres: new Float32Array([0, 0, 0, 1, 10, 20, 30, 5, -10, 0, 4, 2]),
  };
}

describe("level render impostors", () => {
  it("stands the quad in a place per impostor of a run, scaled to its sphere and naming its index in the set", () => {
    const object: IRendererObject = toLevelImpostorObject(7, createImpostors(), 1);
    const transforms = object.instances?.transforms as Float32Array;

    expect(object.geometry).toBe(LEVEL_RENDER_KEYS.impostorQuad);
    expect(object.surfaces).toEqual([LEVEL_RENDER_KEYS.surface(4)]);
    expect(object.instances?.impostors?.key).toBe(LEVEL_RENDER_KEYS.impostors(7));
    expect(Array.from(object.instances?.impostors?.indices ?? [])).toEqual([1, 2]);
    expect([transforms[0], transforms[5], transforms[10]]).toEqual([5, 5, 5]);
    expect(Array.from(transforms.subarray(12, 16))).toEqual([10, 20, 30, 1]);
    expect(Array.from(transforms.subarray(16 + 12, 16 + 16))).toEqual([-10, 0, 4, 1]);
  });
});
