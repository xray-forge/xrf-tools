import { describe, expect, it } from "@jest/globals";

import {
  describeLevelSurfaceGeometry,
  describeLevelSurfaceSpan,
  NO_LEVEL_SURFACE_GEOMETRY,
} from "./level-surface-geometry";

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
