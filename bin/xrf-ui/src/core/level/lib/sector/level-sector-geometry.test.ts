import { describe, expect, it } from "@jest/globals";
import { BufferGeometry, Vector3 } from "three";

import { SectorDescription, VisualBounds } from "@/core/ipc/types/xrf-visual";
import {
  createSectorGeometry,
  HEMI_ATTRIBUTE,
  LIGHTMAP_ATTRIBUTE,
} from "@/core/level/lib/sector/level-sector-geometry";
import { createSectorViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import { mockSectorDescription, mockSectorSection, mockSectorSurface } from "@/fixtures/mocks/level.mocks";
import { mockVisualBounds, MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

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

  // The packer measures the sphere from the same positions by the same method three.js would - the centre of the
  // box, and the furthest position from it - on a thread that is not the one drawing. Recomputing it here walked
  // every vertex twice for an answer already on the wire.
  it("takes the enclosing sphere from the pack rather than measuring it again", () => {
    const views: ISectorViews = createViews();
    const measured: BufferGeometry = createSectorGeometry({ ...views, bounds: null });

    measured.computeBoundingSphere();

    const declared: VisualBounds = mockVisualBounds({
      boundingSphere: {
        center: measured.boundingSphere?.center as Vector3,
        radius: measured.boundingSphere?.radius as number,
      },
    });

    const geometry: BufferGeometry = createSectorGeometry({ ...views, bounds: declared });

    // The same answer, which is the only reason taking it is safe: a sphere that did not enclose the sector would
    // cull it the moment the camera looked along its edge.
    expect(geometry.boundingSphere?.center.toArray()).toEqual(measured.boundingSphere?.center.toArray());
    expect(geometry.boundingSphere?.radius).toBe(measured.boundingSphere?.radius);
  });

  it("uses what the pack declares rather than what the vertices say", () => {
    const views: ISectorViews = createViews();
    const declared: VisualBounds = mockVisualBounds({
      boundingSphere: { center: { x: 7, y: 8, z: 9 }, radius: 40 },
    });

    const geometry: BufferGeometry = createSectorGeometry({ ...views, bounds: declared });

    expect(geometry.boundingSphere?.center.toArray()).toEqual([7, 8, 9]);
    expect(geometry.boundingSphere?.radius).toBe(40);
  });

  // A sector that packed nothing declares no extent, and a geometry with no sphere at all is culled always.
  it("measures the sphere itself where the pack declares none", () => {
    const geometry: BufferGeometry = createSectorGeometry({ ...createViews(), bounds: null });

    expect(geometry.boundingSphere?.radius).toBeGreaterThan(0);
  });
});
