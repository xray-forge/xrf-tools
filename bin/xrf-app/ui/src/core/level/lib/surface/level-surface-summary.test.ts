import { describe, expect, it } from "@jest/globals";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";

import {
  findLevelSurface,
  ILevelSurfaceSummary,
  listLevelSurfaces,
  listNamedLevelSurfaces,
  UNNAMED_LEVEL_SURFACE,
} from "@/core/level/lib/surface/level-surface-summary";
import { mockSurfaceDescriptor } from "@/core/visuals/fixtures/visual.mocks";

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

describe("findLevelSurface", () => {
  it("finds an entry by its shader and every texture, in order, the first that matches", () => {
    const entry: XraySurfaceDescriptor = mockSurfaceDescriptor({ shader: "default", textures: ["wall", "wall_bump"] });
    const table: Array<XraySurfaceDescriptor> = [
      mockSurfaceDescriptor({ shader: "default", textures: ["wall"] }),
      mockSurfaceDescriptor({ shader: "default", textures: ["wall_bump", "wall"] }),
      mockSurfaceDescriptor({ shader: "effects\\water", textures: ["wall", "wall_bump"] }),
      mockSurfaceDescriptor({ shader: "default", textures: ["wall", "wall_bump"] }),
      mockSurfaceDescriptor({ shader: "default", textures: ["wall", "wall_bump"] }),
    ];

    expect(findLevelSurface(entry, table)).toBe(3);
    expect(findLevelSurface(entry, table.slice(0, 3))).toBeNull();
  });
});
