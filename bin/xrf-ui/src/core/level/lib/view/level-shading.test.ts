import { describe, expect, it } from "@jest/globals";
import { ERenderAntialiasing, ERenderDebugView } from "@xrf/ipc/types/xrf-renderer";

import {
  ELevelShading,
  getLevelShadings,
  LEVEL_SHADINGS,
  toShownLevelShading,
} from "@/core/level/lib/view/level-shading";

describe("level shadings", () => {
  // A view the renderer gains has to be listed, or the Shading menu never offers it.
  it("offers every debug view the renderer draws", () => {
    const offered: Set<string> = new Set(LEVEL_SHADINGS.map((choice) => choice.debugView));

    expect([...Object.values(ERenderDebugView)].filter((view) => !offered.has(view))).toEqual([]);
  });

  it("lists FSR's views only while FSR draws the frame, and shows the final frame for one otherwise", () => {
    expect(getLevelShadings(ERenderAntialiasing.FSR).map((choice) => choice.value)).toContain(ELevelShading.FSR_LOCKS);
    expect(getLevelShadings(ERenderAntialiasing.TAA).map((choice) => choice.value)).not.toContain(
      ELevelShading.FSR_LOCKS
    );
    expect(toShownLevelShading(ELevelShading.FSR_LOCKS, ERenderAntialiasing.TAA)).toBe(ELevelShading.FINAL);
    expect(toShownLevelShading(ELevelShading.OVERDRAW, ERenderAntialiasing.TAA)).toBe(ELevelShading.OVERDRAW);
  });
});
