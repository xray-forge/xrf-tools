import { describe, expect, it } from "@jest/globals";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { VisualSubmesh, VisualTextureDependency } from "@/core/ipc/types/xrf-visual";
import { IRendererSurfaceDraw, OPAQUE_RENDERER_SURFACE_DRAW } from "@/core/render/lib/surface/renderer-surface-draw";
import { createVisualSurfaces, toAlphaTexturePaths } from "@/core/visuals/lib/visual-surface";
import {
  mockAlphaSurfaceDescriptor,
  mockPackedSubmesh,
  mockSurfaceDescriptor,
  mockTextureDependency,
  MockVisualBuffer,
} from "@/fixtures/mocks/visual.mocks";

describe("createVisualSurfaces", () => {
  it("joins each submesh on its own position, which is what the backend answered in", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const submeshes: Array<VisualSubmesh> = [
      mockPackedSubmesh(buffer, { index: 0, shaderName: "models\\model" }),
      mockPackedSubmesh(buffer, { index: 1, shaderName: "models\\model_aref" }),
      mockPackedSubmesh(buffer, { index: 2, shaderName: null }),
      mockPackedSubmesh(buffer, { index: 3, shaderName: "models\\never_answered" }),
    ];
    const surfaces: Array<XraySurfaceDescriptor> = [mockSurfaceDescriptor(), mockAlphaSurfaceDescriptor()];

    const states: Map<number, IRendererSurfaceDraw> = createVisualSurfaces(submeshes, surfaces);

    expect(states.get(0)).toEqual(OPAQUE_RENDERER_SURFACE_DRAW);
    expect(states.get(1)!.alphaReference).toBeCloseTo(200 / 255);
    // A submesh naming no shader, and one whose name the map has no answer for, are both drawn opaque rather than
    // left without a state.
    expect(states.get(2)).toEqual(OPAQUE_RENDERER_SURFACE_DRAW);
    expect(states.get(3)).toEqual(OPAQUE_RENDERER_SURFACE_DRAW);
  });

  it("answers for every submesh of a model opened before surfaces existed", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const states: Map<number, IRendererSurfaceDraw> = createVisualSurfaces([mockPackedSubmesh(buffer, { index: 7 })]);

    expect(states.get(7)).toEqual(OPAQUE_RENDERER_SURFACE_DRAW);
  });
});

describe("toAlphaTexturePaths", () => {
  it("names the files an alpha reading surface samples, and no others", () => {
    // The decision is per file because the upload is: two submeshes sharing a texture upload it once, and the one
    // that reads alpha decides the format.
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const submeshes: Array<VisualSubmesh> = [
      mockPackedSubmesh(buffer, { index: 0, shaderName: "models\\model" }),
      mockPackedSubmesh(buffer, { index: 1, shaderName: "models\\model_aref" }),
    ];
    const textures: Array<VisualTextureDependency> = [
      mockTextureDependency({ submeshIndex: 0 }),
      mockTextureDependency({
        reference: "veg\\veg_fluff",
        resolution: {
          kind: "resolved",
          step: "asset root",
          assets: [
            {
              container: { kind: "directory", relativePath: "textures\\veg\\veg_fluff.dds", root: "C:\\gamedata" },
              logicalPath: "textures\\veg\\veg_fluff.dds",
            },
          ],
        },
        submeshIndex: 1,
      }),
    ];

    const surfaces: Map<number, IRendererSurfaceDraw> = createVisualSurfaces(submeshes, [
      mockSurfaceDescriptor(),
      mockAlphaSurfaceDescriptor(),
    ]);

    expect([...toAlphaTexturePaths(surfaces, textures)]).toEqual(["textures\\veg\\veg_fluff.dds"]);
  });

  it("names nothing for a reference that located no file", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const surfaces: Map<number, IRendererSurfaceDraw> = createVisualSurfaces(
      [mockPackedSubmesh(buffer, { index: 0, shaderName: "models\\model_aref" })],
      [mockAlphaSurfaceDescriptor()]
    );

    expect(
      toAlphaTexturePaths(surfaces, [mockTextureDependency({ resolution: { kind: "noScope" }, submeshIndex: 0 })])
    ).toEqual(new Set());
  });

  it("names nothing when no surface reads alpha", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const surfaces: Map<number, IRendererSurfaceDraw> = createVisualSurfaces([mockPackedSubmesh(buffer, { index: 0 })]);

    expect(toAlphaTexturePaths(surfaces, [mockTextureDependency({ submeshIndex: 0 })])).toEqual(new Set());
  });
});
