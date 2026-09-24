import { describe, expect, it } from "@jest/globals";

import {
  ISectorGeometryViews,
  ISectorInstanceViews,
  ISectorSectionViews,
  ISectorViews,
} from "@/core/level/lib/sector/level-sector-views";
import { toLevelSurfaceRender } from "@/core/level/lib/surface/level-surface-render";
import { mockSectorSurface } from "@/fixtures/mocks/level.mocks";

import { countSectorSurfaceGeometry, mergeLevelSurfaceGeometry } from "./level-surface-count";
import {
  describeLevelSurfaceGeometry,
  describeLevelSurfaceSpan,
  NO_LEVEL_SURFACE_GEOMETRY,
} from "./level-surface-geometry";

function section(shaderId: number, drawables: number, triangleCount: number): ISectorSectionViews {
  return {
    bounds: null,
    count: triangleCount * 3,
    drawables: Array.from({ length: drawables }, (_, index: number) => index),
    render: toLevelSurfaceRender(null),
    start: 0,
    surface: mockSectorSurface({ shaderId }),
    triangleCount,
  };
}

/** Baked coordinates as a packed sector carries them: shorts over 1024, no low bytes. */
function withCoordinates(uvs: Array<number>): ISectorGeometryViews {
  return {
    binormals: null,
    indices: Uint32Array.from({ length: uvs.length / 2 }, (_, index: number) => index),
    tangents: null,
    uvComponents: 2,
    uvs: Int16Array.from(uvs, (value: number) => value * 1024),
  } as unknown as ISectorGeometryViews;
}

function instance(shaderId: number, places: number, indexCount: number): ISectorInstanceViews {
  return {
    drawables: Array.from({ length: places }, (_, index: number) => index),
    // Only the index count is read here, so the rest of the views are left off rather than invented.
    geometry: { indexCount } as ISectorGeometryViews,
    hemi: new Float32Array(),
    instanceCount: places,
    render: toLevelSurfaceRender(null),
    surface: mockSectorSurface({ shaderId }),
    transforms: new Float32Array(),
  };
}

function sector(
  sections: Array<ISectorSectionViews>,
  instances: Array<ISectorInstanceViews> = [],
  geometry: ISectorGeometryViews = withCoordinates([])
): ISectorViews {
  return { geometry, instances, sections } as unknown as ISectorViews;
}

function countLevelSurfaceGeometry(...sectors: Array<ISectorViews>) {
  return mergeLevelSurfaceGeometry(sectors.map(countSectorSurfaceGeometry));
}

function resident(
  sections: Array<ISectorSectionViews>,
  instances: Array<ISectorInstanceViews> = [],
  geometry: ISectorGeometryViews = withCoordinates([])
): ISectorViews {
  return sector(sections, instances, geometry);
}

describe("countSectorSurfaceGeometry", () => {
  it("counts the drawables and triangles of one entry", () => {
    const counted = countLevelSurfaceGeometry(resident([section(99, 70, 4807)]));

    expect(counted.get(99)).toMatchObject({ drawables: 70, triangles: 4807 });
  });

  it("keeps entries apart", () => {
    const counted = countLevelSurfaceGeometry(resident([section(99, 1, 10), section(3, 2, 40)]));

    expect(counted.get(99)).toMatchObject({ drawables: 1, triangles: 10 });
    expect(counted.get(3)).toMatchObject({ drawables: 2, triangles: 40 });
  });

  it("adds up the sections of one entry", () => {
    const counted = countLevelSurfaceGeometry(resident([section(99, 3, 30), section(99, 4, 40)]));

    expect(counted.get(99)).toMatchObject({ drawables: 7, triangles: 70 });
  });

  // Each sector is counted as it arrives, since its bytes move to the renderer after; the panel totals them.
  it("adds up one entry across sectors, widening its range and keeping its narrowest draw", () => {
    const counted = countLevelSurfaceGeometry(
      sector([section(99, 1, 1)], [], withCoordinates([0, 0, 1, 1, 0.5, 0.5])),
      sector([section(99, 2, 1)], [], withCoordinates([2, 2, 2.5, 2.5, 2, 2]))
    );

    expect(counted.get(99)).toMatchObject({ drawables: 3, triangles: 2 });
    expect(counted.get(99)?.span).toEqual({ uMax: 2.5, uMin: 0, vMax: 2.5, vMin: 0 });
    expect(counted.get(99)?.narrowest?.uMin).toBeCloseTo(2);
  });

  // A mesh stood in many places draws its triangles once per place, which is what the frame really costs.
  it("counts an instanced mesh once per place it stands", () => {
    const counted = countLevelSurfaceGeometry(resident([], [instance(5, 4, 30)]));

    expect(counted.get(5)).toMatchObject({ drawables: 4, triangles: 40 });
  });

  // The second half of the shape question: a decal's fan is written over its own footprint, so its coordinates run
  // the whole way in both axes. One covering a sliver of that samples a sliver of its texture over its whole area.
  it("measures what the base coordinate covers", () => {
    const geometry: ISectorGeometryViews = withCoordinates([0, 0, 1, 0.5, 0.25, 1]);
    const counted = countLevelSurfaceGeometry(resident([section(99, 1, 1)], [], geometry));

    expect(counted.get(99)?.span).toEqual({ uMax: 1, uMin: 0, vMax: 1, vMin: 0 });
  });

  it("says there is no range where the geometry carries no coordinates", () => {
    const geometry: ISectorGeometryViews = { indices: Uint32Array.of(0, 1, 2), uvs: null } as ISectorGeometryViews;

    expect(countLevelSurfaceGeometry(resident([section(99, 1, 1)], [], geometry)).get(99)?.span).toBeNull();
  });

  it("says nothing about an entry no resident sector draws", () => {
    expect(countLevelSurfaceGeometry(resident([section(99, 1, 10)])).get(3)).toBeUndefined();
  });

  it("counts nothing when nothing is resident", () => {
    expect(countLevelSurfaceGeometry().size).toBe(0);
  });
});

describe("describeLevelSurfaceGeometry", () => {
  // The number worth reading: a baked decal is a clipped fan of dozens of triangles, and a quad laid over the same
  // place is two. The average per drawable is what tells them apart.
  it("says how many triangles each drawable averages", () => {
    expect(describeLevelSurfaceGeometry({ drawables: 70, narrowest: null, span: null, triangles: 4807 })).toBe(
      "70 drawables · 4807 triangles · 68.7 each"
    );
  });

  it("names a quad for what it is", () => {
    expect(describeLevelSurfaceGeometry({ drawables: 70, narrowest: null, span: null, triangles: 140 })).toBe(
      "70 drawables · 140 triangles · 2.0 each"
    );
  });

  it("says when nothing resident draws the entry", () => {
    expect(describeLevelSurfaceGeometry(NO_LEVEL_SURFACE_GEOMETRY)).toBe("nothing resident draws it");
  });
});

describe("describeLevelSurfaceSpan", () => {
  it("names the range in both axes", () => {
    expect(describeLevelSurfaceSpan({ uMax: 1, uMin: 0, vMax: 0.998, vMin: -0.001 })).toBe(
      "u 0.00..1.00 · v -0.00..1.00"
    );
  });

  it("says when the geometry carries no coordinates", () => {
    expect(describeLevelSurfaceSpan(null)).toBe("its geometry carries no base coordinate");
  });
});
