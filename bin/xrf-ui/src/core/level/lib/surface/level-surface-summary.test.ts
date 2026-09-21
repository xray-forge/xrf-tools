import { describe, expect, it } from "@jest/globals";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import {
  ILevelSurfaceSummary,
  listLevelSurfaces,
  listNamedLevelSurfaces,
  UNNAMED_LEVEL_SURFACE,
} from "@/core/level/lib/surface/level-surface-summary";
import { mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";

describe("listLevelSurfaces", () => {
  // The position **is** the shader id: it is what every surface of the level refers to its entry by.
  it("numbers the rows by their place in the table", () => {
    const summaries: Array<ILevelSurfaceSummary> = listLevelSurfaces([
      mockSurfaceDescriptor({ shader: "default" }),
      mockSurfaceDescriptor({ shader: "effects\\wallmarkmult" }),
    ]);

    expect(summaries.map((it) => [it.shaderId, it.shader])).toEqual([
      [0, "default"],
      [1, "effects\\wallmarkmult"],
    ]);
  });

  it("says so for an entry that names no shader", () => {
    const summaries: Array<ILevelSurfaceSummary> = listLevelSurfaces([mockSurfaceDescriptor({ shader: null })]);

    expect(summaries[0].shader).toBe(UNNAMED_LEVEL_SURFACE);
  });
});

describe("listNamedLevelSurfaces", () => {
  // A level's table carries entries holding the place of every later one, and a reader scanning how surfaces are
  // drawn is not looking for the gaps.
  it("drops the entries holding a place and keeps the rest in order", () => {
    const table: Array<XraySurfaceDescriptor> = [
      mockSurfaceDescriptor({ shader: "default" }),
      mockSurfaceDescriptor({ shader: null }),
      mockSurfaceDescriptor({ shader: "effects\\water" }),
    ];

    const named: Array<ILevelSurfaceSummary> = listNamedLevelSurfaces(listLevelSurfaces(table));

    expect(named.map((it) => it.shaderId)).toEqual([0, 2]);
  });
});
