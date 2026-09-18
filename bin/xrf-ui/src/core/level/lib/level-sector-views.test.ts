import { describe, expect, it } from "@jest/globals";

import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import { countSectorTriangles, createSectorViews, ISectorViews } from "@/core/level/lib/level-sector-views";
import {
  mockSectorDescription,
  mockSectorInstanceGroup,
  mockSectorSection,
  mockSectorSurface,
} from "@/fixtures/mocks/level.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

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
