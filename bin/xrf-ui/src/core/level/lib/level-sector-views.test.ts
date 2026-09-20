import { describe, expect, it } from "@jest/globals";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import {
  countSectorDraws,
  countSectorTriangles,
  createSectorViews,
  hasAlphaSurfaces,
  ISectorViews,
  listSectorTextures,
} from "@/core/level/lib/level-sector-views";
import { OPAQUE_RENDER_SURFACE } from "@/core/render/lib/render-surface";
import {
  mockSectorDescription,
  mockSectorInstanceGroup,
  mockSectorSection,
  mockSectorSurface,
} from "@/fixtures/mocks/level.mocks";
import { mockAlphaSurfaceDescriptor, MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

describe("level sector views", () => {
  it("builds typed array views over the packed sections", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer);

    const views: ISectorViews = createSectorViews(description, buffer.toArrayBuffer());

    expect(Array.from(views.geometry.positions)).toEqual([0, 0, 0, 1, 0, 0, 0, 1, 0]);
    expect(Array.from(views.geometry.indices)).toEqual([0, 1, 2]);
    expect(views.geometry.vertexCount).toBe(3);
  });

  // A sector reaches past what sixteen bits address, so its indices are read through a `Uint32Array`. Reading them as
  // sixteen would pass on a fixture whose indices are all small, so the assertion is on the view's own type.
  it("reads a sector's indices as thirty-two bit elements", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer);

    const views: ISectorViews = createSectorViews(description, buffer.toArrayBuffer());

    expect(views.geometry.indices).toBeInstanceOf(Uint32Array);
    expect(views.geometry.indices.byteLength).toBe(3 * Uint32Array.BYTES_PER_ELEMENT);
  });

  // An attribute is present only when a declaration in the sector carried it, so a positions-only sector has to
  // report nothing rather than an empty view a renderer would bind.
  it("leaves out an attribute the sector declared nothing for", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer);

    const views: ISectorViews = createSectorViews(description, buffer.toArrayBuffer());

    expect(views.geometry.normals).toBeNull();
    expect(views.geometry.lightmapUvs).toBeNull();
    expect(views.geometry.uvs).toBeNull();
  });

  it("carries an attribute the sector did declare", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer);
    const lightmapUvs = buffer.pushFloats([0, 0, 1, 0, 0, 1]);

    const views: ISectorViews = createSectorViews(
      {
        ...description,
        bufferLength: buffer.byteLength,
        geometry: { ...description.geometry, lightmapUvs },
      },
      buffer.toArrayBuffer()
    );

    expect(Array.from(views.geometry.lightmapUvs as Float32Array)).toEqual([0, 0, 1, 0, 0, 1]);
  });

  it("keeps each section's range and the surface that draws it", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer, {
      sections: [
        mockSectorSection({ draw: { start: 0, count: 3 }, surface: mockSectorSurface({ shaderId: 1 }) }),
        mockSectorSection({
          draw: { start: 3, count: 6 },
          surface: mockSectorSurface({ shaderId: 2, shaderName: "glass", textureName: "window" }),
        }),
      ],
    });

    const views: ISectorViews = createSectorViews(description, buffer.toArrayBuffer());

    expect(views.sections).toHaveLength(2);
    expect(views.sections[0]).toMatchObject({ count: 3, start: 0, surface: { shaderId: 1 }, triangleCount: 1 });
    expect(views.sections[1]).toMatchObject({
      count: 6,
      start: 3,
      surface: { shaderId: 2, textureName: "window" },
      triangleCount: 2,
    });
    expect(countSectorTriangles(views)).toBe(3);
  });

  // The sector's own geometry and a mesh it stands in many places are packed the same way, so they are read the same
  // way. Two spellings of it would be two places to forget an attribute.
  it("reads an instanced mesh as the same geometry a sector's own is", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer, {
      instances: [mockSectorInstanceGroup(buffer, [0, 100])],
    });

    const views: ISectorViews = createSectorViews(
      { ...description, bufferLength: buffer.byteLength },
      buffer.toArrayBuffer()
    );

    expect(Object.keys(views.instances[0]?.geometry ?? {}).sort()).toEqual(Object.keys(views.geometry).sort());
    expect(views.instances[0]?.instanceCount).toBe(2);
  });

  // The description says where everything sits in the buffer, so a buffer of another size is another pack, and the
  // views would point at whatever happened to be at those offsets.
  it("refuses a buffer its description does not cover", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer);

    expect(() => createSectorViews(description, new ArrayBuffer(8))).toThrow(
      "The description and the buffer came from different packs."
    );
  });
});

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

  it("counts a draw for each surface and each instanced mesh", () => {
    expect(countSectorDraws(viewsOf(sectorDrawing([]), {}))).toBe(2);
  });
});
