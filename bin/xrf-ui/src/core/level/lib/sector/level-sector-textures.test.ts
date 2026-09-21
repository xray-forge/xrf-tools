import { describe, expect, it } from "@jest/globals";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import {
  hasDetailedSurfaces,
  ISectorTextureRequest,
  listSectorTextures,
} from "@/core/level/lib/sector/level-sector-textures";
import { createSectorViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import { OPAQUE_RENDER_SURFACE } from "@/core/render/lib/surface/render-surface";
import {
  mockSectorDescription,
  mockSectorInstanceGroup,
  mockSectorSection,
  mockSectorSurface,
} from "@/fixtures/mocks/level.mocks";
import {
  mockAlphaSurfaceDescriptor,
  mockBlendedSurfaceDescriptor,
  mockSurfaceDescriptor,
  MockVisualBuffer,
} from "@/fixtures/mocks/visual.mocks";

describe("level sector surfaces", () => {
  const SHADER_ID: number = 3;

  /** A sector whose one section and one instanced mesh are both drawn by the row at `SHADER_ID`. */
  function sectorDrawing(lightmaps: Array<string>): SectorDescription {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const surface = mockSectorSurface({
      // What the renderer binds is the row's third texture, and only when it is spelled `lmap`.
      hemi: lightmaps[1] ?? null,
      shaderId: SHADER_ID,
      shaderName: "levels\\aref",
      textureName: "veg\veg_reed",
    });
    const description: SectorDescription = mockSectorDescription(buffer, {
      instances: [mockSectorInstanceGroup(buffer, [0], { surface })],
      sections: [mockSectorSection({ surface })],
    });

    return { ...description, bufferLength: buffer.byteLength };
  }

  function table(descriptor: XraySurfaceDescriptor): Array<XraySurfaceDescriptor> {
    const rows: Array<XraySurfaceDescriptor> = [];

    rows[SHADER_ID] = descriptor;

    return rows;
  }

  function viewsOf(description: SectorDescription, surfaces: Array<XraySurfaceDescriptor>): ISectorViews {
    return createSectorViews(description, new ArrayBuffer(description.bufferLength), surfaces);
  }

  it("joins each surface on the shader id it declares", () => {
    const views: ISectorViews = viewsOf(sectorDrawing([]), table(mockAlphaSurfaceDescriptor()));

    expect(views.sections[0]?.render.alphaTest).toBeCloseTo(200 / 255);
    expect(views.instances[0]?.render.alphaTest).toBeCloseTo(200 / 255);
  });

  it("draws a surface the table has no answer for opaque rather than leaving it without a state", () => {
    const views: ISectorViews = viewsOf(sectorDrawing([]), []);

    expect(views.sections[0]?.render).toEqual(OPAQUE_RENDER_SURFACE);
    expect(hasDetailedSurfaces(views)).toBe(false);
  });

  // The upload is per file while alpha is per surface: a DXT1 base drawn by a cut-out surface has to keep the alpha
  // bit its blocks carry, and its lightmap is sampled for light rather than tested for coverage.
  it("asks for alpha on a cut-out surface's base texture and never on its occlusion map", () => {
    const views: ISectorViews = viewsOf(sectorDrawing(["lmap#1_1", "lmap#1_2"]), table(mockAlphaSurfaceDescriptor()));
    const requests: Array<ISectorTextureRequest> = listSectorTextures(views);

    expect(requests).toContainEqual({ isAlphaRead: true, isMipped: true, reference: "veg\veg_reed" });
    expect(requests).toContainEqual({ isAlphaRead: false, isMipped: true, reference: "lmap#1_2" });
  });

  // Only the one the renderer samples. Reading the other half of every pair was a megabyte a lightmap for a texture
  // nothing binds.
  it("does not ask for the half of the lightmap pair no deferred shader reads", () => {
    const views: ISectorViews = viewsOf(sectorDrawing(["lmap#1_1", "lmap#1_2"]), table(mockSurfaceDescriptor()));

    expect(listSectorTextures(views).map((it) => it.reference)).not.toContain("lmap#1_1");
  });

  // Nothing in the level names it: a sector fetching only what its shader table spells would draw its ground as the
  // bare whole-level photograph its base texture is.
  it("asks for the detail texture a surface is modulated with, without its alpha", () => {
    const detailed: XraySurfaceDescriptor = mockSurfaceDescriptor({
      detail: { reference: "detail\\detail_grnd_earth", scale: 150 },
    });
    const views: ISectorViews = viewsOf(sectorDrawing([]), table(detailed));

    expect(listSectorTextures(views)).toContainEqual({
      isAlphaRead: false,
      isMipped: true,
      reference: "detail\\detail_grnd_earth",
    });
    expect(hasDetailedSurfaces(views)).toBe(true);
  });

  it("names a reference once, keeping the alpha any surface naming it asked for", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer, {
      sections: [
        mockSectorSection({ surface: mockSectorSurface({ shaderId: 0, textureName: "shared" }) }),
        mockSectorSection({ surface: mockSectorSurface({ shaderId: SHADER_ID, textureName: "shared" }) }),
      ],
    });

    const views: ISectorViews = createSectorViews(
      { ...description, bufferLength: buffer.byteLength },
      buffer.toArrayBuffer(),
      table(mockAlphaSurfaceDescriptor())
    );

    expect(listSectorTextures(views)).toEqual([{ isAlphaRead: true, isMipped: true, reference: "shared" }]);
  });

  // The engine binds a wall mark's decal through `smp_rtlinear`, which filters nothing between mips
  // (`Blender_Recorder_R3.cpp`). A decal is dark marks on a neutral field, so mipping one spreads the marks over
  // the field as soon as it is minified, and the footprint fills with a grey the engine never draws.
  it("asks for a wall mark's decal without the mip chain", () => {
    const wallmark: XraySurfaceDescriptor = mockBlendedSurfaceDescriptor({
      declaration: {
        function: "normal",
        isAlphaTested: true,
        isBlended: true,
        isDepthWritten: false,
        isWallmark: true,
        kind: "scripted",
        script: "shaders\\r2\\effects_wallmarkmult.s",
      },
      draw: { isDoubled: true, kind: "multiplied" },
    });
    const [base] = listSectorTextures(viewsOf(sectorDrawing([]), table(wallmark)));

    expect(base.isMipped).toBe(false);
    expect(base.isAlphaRead).toBe(true);
  });

  it("keeps the mip chain for every surface that is not one", () => {
    const views: ISectorViews = viewsOf(sectorDrawing([]), table(mockSurfaceDescriptor()));

    expect(listSectorTextures(views).every((it: ISectorTextureRequest) => it.isMipped)).toBe(true);
  });
});
