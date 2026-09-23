import { describe, expect, it } from "@jest/globals";
import { ERendererDraw } from "@xrf/renderer";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import {
  hasDetailedSurfaces,
  listDescriptionTextures,
  listSectorTextures,
} from "@/core/level/lib/sector/level-sector-textures";
import { createSectorViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";
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

    expect(views.sections[0]?.render.draw).toBe(ERendererDraw.CUT_OUT);
    expect(views.instances[0]?.render.alphaReference).toBeCloseTo(200 / 255);
  });

  it("draws a surface the table has no answer for opaque rather than leaving it without a state", () => {
    const views: ISectorViews = viewsOf(sectorDrawing([]), []);

    expect(views.sections[0]?.render).toMatchObject({ detail: null, draw: ERendererDraw.OPAQUE, isWallmark: false });
    expect(hasDetailedSurfaces(views)).toBe(false);
  });

  it("asks for a surface's base texture and its occlusion map", () => {
    const views: ISectorViews = viewsOf(sectorDrawing(["lmap#1_1", "lmap#1_2"]), table(mockAlphaSurfaceDescriptor()));

    expect(listSectorTextures(views)).toEqual([{ reference: "veg\veg_reed" }, { reference: "lmap#1_2" }]);
  });

  // Only the one the renderer samples. Reading the other half of every pair was a megabyte a lightmap for a texture
  // nothing binds.
  it("does not ask for the half of the lightmap pair no deferred shader reads", () => {
    const views: ISectorViews = viewsOf(sectorDrawing(["lmap#1_1", "lmap#1_2"]), table(mockSurfaceDescriptor()));

    expect(listSectorTextures(views).map((it) => it.reference)).not.toContain("lmap#1_1");
  });

  // Nothing in the level names it: a sector fetching only what its shader table spells would draw its ground as the
  // bare whole-level photograph its base texture is.
  it("asks for the detail texture a surface is modulated with", () => {
    const detailed: XraySurfaceDescriptor = mockSurfaceDescriptor({
      detail: { reference: "detail\\detail_grnd_earth", scale: 150 },
    });
    const views: ISectorViews = viewsOf(sectorDrawing([]), table(detailed));

    expect(listSectorTextures(views)).toContainEqual({ reference: "detail\\detail_grnd_earth" });
    expect(hasDetailedSurfaces(views)).toBe(true);
  });

  it("names a reference once, however many surfaces name it", () => {
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

    expect(listSectorTextures(views)).toEqual([{ reference: "shared" }]);
  });

  // The loader reads what a sector names before it reads the sector, so the description alone has to answer.
  it("reads the same references off the description as off the views", () => {
    const description: SectorDescription = sectorDrawing(["lmap#1_1", "lmap#1_2"]);
    const surfaces: Array<XraySurfaceDescriptor> = table(mockSurfaceDescriptor());

    expect(listDescriptionTextures(description, surfaces)).toEqual(listSectorTextures(viewsOf(description, surfaces)));
  });

  it("says which surfaces are wall marks, which the renderer composites into the albedo", () => {
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
    const views: ISectorViews = viewsOf(sectorDrawing([]), table(wallmark));

    expect(views.sections[0]?.render).toMatchObject({
      draw: ERendererDraw.MULTIPLIED_2X,
      isLit: false,
      isWallmark: true,
    });
  });
});
