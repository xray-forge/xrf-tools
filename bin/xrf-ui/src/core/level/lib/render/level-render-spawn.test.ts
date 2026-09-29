import { describe, expect, it } from "@jest/globals";
import { ERendererDraw, IRendererGeometry, IRendererSurface, TRendererColor } from "@xrf/renderer";

import { EXrayBumpMode, XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { EXrayResolution } from "@/core/ipc/types/xrf-vfs";
import { toLevelSpawnSurface, toPosedGeometry } from "@/core/level/lib/render/level-render-spawn";
import { IVisualSubmeshViews } from "@/core/visuals/lib/visual-views";
import { mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";

/** One vertex a metre and a half up, facing `+x`, hung entirely from bone zero. */
function createSubmesh(): IVisualSubmeshViews {
  return {
    binormals: new Float32Array([0, 0, 1]),
    clusters: null,
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

describe("toLevelSpawnSurface", () => {
  const color: TRendererColor = [0.2, 0.4, 0.6];

  it("shades a part with the lighting model its descriptor sets, as a sector's surface is", () => {
    const surface: IRendererSurface = toLevelSpawnSurface({
      color,
      descriptor: mockSurfaceDescriptor({ material: 3 }),
      texture: "lamp",
    });

    expect(surface.material).toBe(3);
    expect(surface.textures?.base).toBe("lamp");
  });

  it("carries its model's colour beside its base, which the renderer draws while textures are off", () => {
    const surface: IRendererSurface = toLevelSpawnSurface({ color, descriptor: null, texture: "lamp" });

    expect(surface.color).toEqual(color);
    expect(surface.textures?.base).toBe("lamp");
  });

  it("binds the bump pair and the detail its descriptor declares beside its base", () => {
    const descriptor: XraySurfaceDescriptor = mockSurfaceDescriptor({
      bump: {
        bump: { reference: "lamp_bump", resolution: { kind: EXrayResolution.MISSING, roots: [] } },
        companion: { reference: "lamp_bump#", resolution: { kind: EXrayResolution.MISSING, roots: [] } },
        mode: EXrayBumpMode.USE,
        virtualHeight: null,
      },
      detail: {
        bump: {
          bump: { reference: "detail\\detail_metal_bump", resolution: { kind: EXrayResolution.MISSING, roots: [] } },
          companion: {
            reference: "detail\\detail_metal_bump#",
            resolution: { kind: EXrayResolution.MISSING, roots: [] },
          },
          mode: EXrayBumpMode.USE,
          virtualHeight: null,
        },
        reference: "detail\\detail_metal",
        scale: 4,
      },
    });
    const surface: IRendererSurface = toLevelSpawnSurface({ color, descriptor, texture: "lamp" });

    expect(surface.textures).toEqual({
      base: "lamp",
      bump: "lamp_bump",
      bumpCompanion: "lamp_bump#",
      detail: "detail\\detail_metal",
      detailBump: "detail\\detail_metal_bump",
      detailBumpCompanion: "detail\\detail_metal_bump#",
    });
    expect(surface.detailScale).toBe(4);
  });
});
