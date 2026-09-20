import { describe, expect, it } from "@jest/globals";
import { BufferGeometry } from "three";

import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import {
  createSectorGeometry,
  HEMI_ATTRIBUTE,
  LIGHTMAP_ATTRIBUTE,
} from "@/core/level/lib/sector/level-sector-geometry";
import { createSectorViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import { mockSectorDescription, mockSectorSection, mockSectorSurface } from "@/fixtures/mocks/level.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

function createViews(
  overrides: Partial<SectorDescription> = {},
  extend?: (buffer: MockVisualBuffer) => void
): ISectorViews {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const description: SectorDescription = mockSectorDescription(buffer, overrides);

  extend?.(buffer);

  return createSectorViews({ ...description, ...overrides, bufferLength: buffer.byteLength }, buffer.toArrayBuffer());
}

describe("level sector geometry", () => {
  it("binds the attributes the sector carries", () => {
    const geometry: BufferGeometry = createSectorGeometry(createViews());

    expect(geometry.getAttribute("position").count).toBe(3);
    expect(geometry.getIndex()?.count).toBe(3);
  });

  // Binding an attribute the sector never packed would hand the renderer a view over nothing.
  it("binds nothing for an attribute the sector did not declare", () => {
    const geometry: BufferGeometry = createSectorGeometry(createViews());

    expect(geometry.getAttribute("normal")).toBeUndefined();
    expect(geometry.getAttribute(LIGHTMAP_ATTRIBUTE)).toBeUndefined();
    expect(geometry.getAttribute(HEMI_ATTRIBUTE)).toBeUndefined();
  });

  // One geometry with a group per surface, not a mesh per surface: the drawables of a sector share the vertex array,
  // so splitting them would upload the same vertices once per surface.
  it("draws each surface as a group of the one shared geometry", () => {
    const geometry: BufferGeometry = createSectorGeometry(
      createViews({
        sections: [
          mockSectorSection({ draw: { start: 0, count: 3 }, surface: mockSectorSurface({ shaderId: 1 }) }),
          mockSectorSection({ draw: { start: 3, count: 6 }, surface: mockSectorSurface({ shaderId: 2 }) }),
        ],
      })
    );

    expect(geometry.groups).toEqual([
      { start: 0, count: 3, materialIndex: 0 },
      { start: 3, count: 6, materialIndex: 1 },
    ]);
  });

  it("measures the sector so a viewer can cull it", () => {
    const geometry: BufferGeometry = createSectorGeometry(createViews());

    expect(geometry.boundingSphere).not.toBeNull();
    expect(geometry.boundingSphere?.radius).toBeGreaterThan(0);
  });
});
