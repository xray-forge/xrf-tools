import { describe, expect, it } from "@jest/globals";
import { ELevelObjectShape, ELevelRestriction, LevelObject } from "@xrf/ipc/types/xrf-app-levels";

import { mockLevelObject, mockLevelPatrol } from "@/core/level/fixtures/level.mocks";
import {
  describeLevelColor,
  describeLevelRestriction,
  describeLevelShape,
  describeLevelSoundWindow,
  listLevelPatrolLeads,
} from "@/tools/level-viewer/lib/outliner/level-object-describe";

describe("describeLevelSoundWindow", () => {
  // `SStaticSound` reads a window of noughts as the sound's own default rather than as nothing.
  it("says a window of noughts as its default, an active one as the game day's hours and others in seconds", () => {
    expect(describeLevelSoundWindow({ from: 0, to: 0 }, "active")).toBe("All day");
    expect(describeLevelSoundWindow({ from: 0, to: 0 }, "play")).toBe("The whole sound");
    expect(describeLevelSoundWindow({ from: 0, to: 0 }, "pause")).toBe("Looped without a pause");
    expect(describeLevelSoundWindow({ from: 6 * 3_600_000, to: 21 * 3_600_000 + 30 * 60_000 }, "active")).toBe(
      "06:00 to 21:30"
    );
    expect(describeLevelSoundWindow({ from: 1500, to: 4000 }, "pause")).toBe("1.5 s to 4 s");
  });
});

describe("describeLevelColor", () => {
  it("writes each channel with two decimals", () => {
    expect(describeLevelColor([1, 0.5, 0.125])).toBe("1.00 0.50 0.13");
  });
});

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
