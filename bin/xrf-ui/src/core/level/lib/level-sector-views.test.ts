import { describe, expect, it } from "@jest/globals";

import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import { countSectorTriangles, createSectorViews, ISectorViews } from "@/core/level/lib/level-sector-views";
import { mockSectorDescription, mockSectorSection } from "@/fixtures/mocks/level.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

describe("level sector views", () => {
  it("builds typed array views over the packed sections", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer);

    const views: ISectorViews = createSectorViews(description, buffer.toArrayBuffer());

    expect(Array.from(views.positions)).toEqual([0, 0, 0, 1, 0, 0, 0, 1, 0]);
    expect(Array.from(views.indices)).toEqual([0, 1, 2]);
    expect(views.vertexCount).toBe(3);
  });

  // A sector reaches past what sixteen bits address, so its indices are read through a `Uint32Array`. Reading them as
  // sixteen would pass on a fixture whose indices are all small, so the assertion is on the view's own type.
  it("reads a sector's indices as thirty-two bit elements", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer);

    const views: ISectorViews = createSectorViews(description, buffer.toArrayBuffer());

    expect(views.indices).toBeInstanceOf(Uint32Array);
    expect(views.indices.byteLength).toBe(3 * Uint32Array.BYTES_PER_ELEMENT);
  });

  // An attribute is present only when a declaration in the sector carried it, so a positions-only sector has to
  // report nothing rather than an empty view a renderer would bind.
  it("leaves out an attribute the sector declared nothing for", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer);

    const views: ISectorViews = createSectorViews(description, buffer.toArrayBuffer());

    expect(views.normals).toBeNull();
    expect(views.lightmapUvs).toBeNull();
    expect(views.uvs).toBeNull();
  });

  it("carries an attribute the sector did declare", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer);
    const lightmapCoordinates = buffer.pushFloats([0, 0, 1, 0, 0, 1]);

    const views: ISectorViews = createSectorViews(
      { ...description, lightmapCoordinates, bufferLength: buffer.byteLength },
      buffer.toArrayBuffer()
    );

    expect(Array.from(views.lightmapUvs as Float32Array)).toEqual([0, 0, 1, 0, 0, 1]);
  });

  it("keeps each section's range and the surface that draws it", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: SectorDescription = mockSectorDescription(buffer, {
      sections: [
        mockSectorSection({ shaderId: 1, draw: { start: 0, count: 3 } }),
        mockSectorSection({ shaderId: 2, draw: { start: 3, count: 6 }, shaderName: "glass", textureName: "window" }),
      ],
    });

    const views: ISectorViews = createSectorViews(description, buffer.toArrayBuffer());

    expect(views.sections).toHaveLength(2);
    expect(views.sections[0]).toMatchObject({ shaderId: 1, start: 0, count: 3, triangleCount: 1 });
    expect(views.sections[1]).toMatchObject({
      shaderId: 2,
      start: 3,
      count: 6,
      triangleCount: 2,
      textureName: "window",
    });
    expect(countSectorTriangles(views)).toBe(3);
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
