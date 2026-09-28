import { describe, expect, it } from "@jest/globals";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { VisualSubmesh } from "@/core/ipc/types/xrf-visual";
import { IRendererSurfaceDraw, OPAQUE_RENDERER_SURFACE_DRAW } from "@/core/render/lib/surface/renderer-surface-draw";
import { createVisualSurfaces } from "@/core/visuals/lib/visual-surface";
import {
  mockAlphaSurfaceDescriptor,
  mockPackedSubmesh,
  mockSurfaceDescriptor,
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
