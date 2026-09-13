import { describe, expect, it } from "@jest/globals";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { VisualSubmesh, VisualTextureDependency } from "@/core/ipc/types/xrf-visual";
import {
  createVisualSurfaces,
  isAlphaVisualSurface,
  IVisualSurface,
  OPAQUE_VISUAL_SURFACE,
  toAlphaTexturePaths,
  toVisualSurface,
} from "@/core/visuals/lib/visual-surface";
import {
  mockAlphaSurfaceDescriptor,
  mockPackedSubmesh,
  mockSurfaceDescriptor,
  mockTextureDependency,
  MockVisualBuffer,
} from "@/fixtures/mocks/visual.mocks";

describe("toVisualSurface", () => {
  it("draws a surface with no answer the way the engine draws an unresolved shader", () => {
    expect(toVisualSurface(null)).toEqual(OPAQUE_VISUAL_SURFACE);
    expect(toVisualSurface(mockSurfaceDescriptor({ declaration: { kind: "noLibrary" } }))).toEqual(
      OPAQUE_VISUAL_SURFACE
    );
  });

  it("cuts out against the resolved reference without compositing", () => {
    // A cut-out is opaque everywhere it is not discarded, so it keeps writing depth and stays out of the sorted pass.
    const surface: IVisualSurface = toVisualSurface(mockAlphaSurfaceDescriptor());

    expect(surface).toEqual({ alphaTest: 200 / 255, isDepthWritten: true, isTransparent: false });
  });

  it("composites a blended surface and stops it writing depth", () => {
    const surface: IVisualSurface = toVisualSurface(
      mockAlphaSurfaceDescriptor({ draw: { kind: "blended", reference: 32 } })
    );

    expect(surface).toEqual({ alphaTest: 32 / 255, isDepthWritten: false, isTransparent: true });
  });

  it("keeps a blended surface with no reference sampling everything it draws", () => {
    // `models\window` and every other `MODELEbB`: the class passes no reference, so nothing is discarded.
    expect(toVisualSurface(mockAlphaSurfaceDescriptor({ draw: { kind: "blended", reference: 0 } }))).toEqual({
      alphaTest: 0,
      isDepthWritten: false,
      isTransparent: true,
    });
  });
});

describe("isAlphaVisualSurface", () => {
  it("counts a blended surface that discards nothing", () => {
    expect(isAlphaVisualSurface(OPAQUE_VISUAL_SURFACE)).toBe(false);
    expect(isAlphaVisualSurface({ alphaTest: 0, isDepthWritten: false, isTransparent: true })).toBe(true);
    expect(isAlphaVisualSurface({ alphaTest: 0.5, isDepthWritten: true, isTransparent: false })).toBe(true);
  });
});

describe("createVisualSurfaces", () => {
  it("joins each submesh on the shader name it declares", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const submeshes: Array<VisualSubmesh> = [
      mockPackedSubmesh(buffer, { index: 0, shaderName: "models\\model" }),
      mockPackedSubmesh(buffer, { index: 1, shaderName: "models\\model_aref" }),
      mockPackedSubmesh(buffer, { index: 2, shaderName: null }),
      mockPackedSubmesh(buffer, { index: 3, shaderName: "models\\never_answered" }),
    ];
    const surfaces: Record<string, XraySurfaceDescriptor> = {
      "models\\model": mockSurfaceDescriptor(),
      "models\\model_aref": mockAlphaSurfaceDescriptor(),
    };

    const states: Map<number, IVisualSurface> = createVisualSurfaces(submeshes, surfaces);

    expect(states.get(0)).toEqual(OPAQUE_VISUAL_SURFACE);
    expect(states.get(1)!.alphaTest).toBeCloseTo(200 / 255);
    // A submesh naming no shader, and one whose name the map has no answer for, are both drawn opaque rather than
    // left without a state.
    expect(states.get(2)).toEqual(OPAQUE_VISUAL_SURFACE);
    expect(states.get(3)).toEqual(OPAQUE_VISUAL_SURFACE);
  });

  it("answers for every submesh of a model opened before surfaces existed", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const states: Map<number, IVisualSurface> = createVisualSurfaces([mockPackedSubmesh(buffer, { index: 7 })]);

    expect(states.get(7)).toEqual(OPAQUE_VISUAL_SURFACE);
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

    const surfaces: Map<number, IVisualSurface> = createVisualSurfaces(submeshes, {
      "models\\model": mockSurfaceDescriptor(),
      "models\\model_aref": mockAlphaSurfaceDescriptor(),
    });

    expect([...toAlphaTexturePaths(surfaces, textures)]).toEqual(["textures\\veg\\veg_fluff.dds"]);
  });

  it("names nothing for a reference that located no file", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const surfaces: Map<number, IVisualSurface> = createVisualSurfaces(
      [mockPackedSubmesh(buffer, { index: 0, shaderName: "models\\model_aref" })],
      { "models\\model_aref": mockAlphaSurfaceDescriptor() }
    );

    expect(
      toAlphaTexturePaths(surfaces, [mockTextureDependency({ resolution: { kind: "noScope" }, submeshIndex: 0 })])
    ).toEqual(new Set());
  });

  it("names nothing when no surface reads alpha", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const surfaces: Map<number, IVisualSurface> = createVisualSurfaces([mockPackedSubmesh(buffer, { index: 0 })]);

    expect(toAlphaTexturePaths(surfaces, [mockTextureDependency({ submeshIndex: 0 })])).toEqual(new Set());
  });
});
