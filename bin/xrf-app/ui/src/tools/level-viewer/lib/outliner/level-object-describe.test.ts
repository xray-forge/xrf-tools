import { describe, expect, it } from "@jest/globals";
import { ELevelObjectShape, ELevelRestriction, LevelObject } from "@xrf/ipc/types/xrf-app-levels";

import { mockLevelObject, mockLevelPatrol } from "@/core/level/fixtures/level.mocks";
import {
  describeLevelRestriction,
  describeLevelShape,
  listLevelPatrolLeads,
} from "@/tools/level-viewer/lib/outliner/level-object-describe";

describe("listLevelPatrolLeads", () => {
  it("names the points a patrol point leads on to, with how likely each way is", () => {
    const objects: Array<LevelObject> = mockLevelPatrol(0, "walk", 3);

    expect(listLevelPatrolLeads(objects[1] as LevelObject, objects)).toEqual(["wp01 (1)"]);
    expect(listLevelPatrolLeads(objects[3] as LevelObject, objects)).toEqual([]);
    expect(listLevelPatrolLeads(mockLevelObject(), objects)).toEqual([]);
  });
});

describe("describeLevelShape", () => {
  it("states a sphere's radius and a box's sides in metres", () => {
    expect(describeLevelShape({ center: { x: 0, y: 0, z: 0 }, kind: ELevelObjectShape.SPHERE, radius: 2.5 })).toBe(
      "Sphere, 2.5 m radius"
    );
    expect(
      describeLevelShape({
        axes: [
          { x: 2, y: 0, z: 0 },
          { x: 0, y: 3, z: 0 },
          { x: 0, y: 0, z: 4.125 },
        ],
        center: { x: 0, y: 0, z: 0 },
        kind: ELevelObjectShape.BOX,
      })
    ).toBe("Box, 2 m × 3 m × 4.13 m");
    expect(describeLevelRestriction(ELevelRestriction.OUT)).toBe("Keeps out");
  });
});
