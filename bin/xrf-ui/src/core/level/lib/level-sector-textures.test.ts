import { describe, expect, it } from "@jest/globals";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import { hasAlphaSurfaces, listSectorTextures } from "@/core/level/lib/level-sector-textures";
import { createSectorViews, ISectorViews } from "@/core/level/lib/level-sector-views";
import { OPAQUE_RENDER_SURFACE } from "@/core/render/lib/render-surface";
import {
  mockSectorDescription,
  mockSectorInstanceGroup,
  mockSectorSection,
  mockSectorSurface,
} from "@/fixtures/mocks/level.mocks";
import { mockAlphaSurfaceDescriptor, MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

describe("level sector surfaces", () => {
  const SHADER: string = "levels\\aref";

  /** A sector whose one section and one instanced mesh are both drawn by `SHADER`. */
  function sectorDrawing(lightmaps: Array<string>): SectorDescription {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const surface = mockSectorSurface({ lightmaps, shaderName: SHADER, textureName: "veg\veg_reed" });
    const description: SectorDescription = mockSectorDescription(buffer, {
      instances: [mockSectorInstanceGroup(buffer, [0], { surface })],
      sections: [mockSectorSection({ surface })],
    });

    return { ...description, bufferLength: buffer.byteLength };
  }

  function viewsOf(description: SectorDescription, surfaces: Record<string, XraySurfaceDescriptor>): ISectorViews {
    return createSectorViews(description, new ArrayBuffer(description.bufferLength), surfaces);
  }

  it("joins each surface on the shader name it declares", () => {
    const views: ISectorViews = viewsOf(sectorDrawing([]), { [SHADER]: mockAlphaSurfaceDescriptor() });

    expect(views.sections[0]?.render.alphaTest).toBeCloseTo(200 / 255);
    expect(views.instances[0]?.render.alphaTest).toBeCloseTo(200 / 255);
    expect(hasAlphaSurfaces(views)).toBe(true);
  });

  it("draws a surface the table has no answer for opaque rather than leaving it without a state", () => {
    const views: ISectorViews = viewsOf(sectorDrawing([]), {});

    expect(views.sections[0]?.render).toEqual(OPAQUE_RENDER_SURFACE);
    expect(hasAlphaSurfaces(views)).toBe(false);
  });

  // The upload is per file while alpha is per surface: a DXT1 base drawn by a cut-out surface has to keep the alpha
  // bit its blocks carry, and its lightmap is sampled for light rather than tested for coverage.
  it("asks for alpha on a cut-out surface's base texture and never on its lightmap", () => {
    const views: ISectorViews = viewsOf(sectorDrawing(["lmap#1_1"]), { [SHADER]: mockAlphaSurfaceDescriptor() });
    const requests = listSectorTextures(views);

    expect(requests).toContainEqual({ isAlphaRead: true, reference: "veg\veg_reed" });
    expect(requests).toContainEqual({ isAlphaRead: false, reference: "lmap#1_1" });
  });

  it("names a reference once, keeping the alpha any surface naming it asked for", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer, {
      sections: [
        mockSectorSection({ surface: mockSectorSurface({ shaderName: "levels\\solid", textureName: "shared" }) }),
        mockSectorSection({ surface: mockSectorSurface({ shaderName: SHADER, textureName: "shared" }) }),
      ],
    });

    const views: ISectorViews = createSectorViews(
      { ...description, bufferLength: buffer.byteLength },
      buffer.toArrayBuffer(),
      { [SHADER]: mockAlphaSurfaceDescriptor() }
    );

    expect(listSectorTextures(views)).toEqual([{ isAlphaRead: true, reference: "shared" }]);
  });
});
