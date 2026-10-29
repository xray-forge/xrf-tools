import { describe, expect, it } from "@jest/globals";
import { ERenderLightLevel } from "@xrf/ipc/types/xrf-renderer";

import {
  getRenderLightLevelOptions,
  toDrawnRenderLightLevel,
} from "@/core/render/lib/features/render-light-level-choices";

describe("getRenderLightLevelOptions", () => {
  it("offers the dev levels only where they are available, shown either way", () => {
    expect(getRenderLightLevelOptions(true).map((option) => option.isDisabled ?? false)).toEqual([false, false, false]);
    expect(getRenderLightLevelOptions(false).map((option) => option.isDisabled ?? false)).toEqual([false, true, true]);
  });
});

describe("toDrawnRenderLightLevel", () => {
  it("draws a parked dev level as the authored one where it is not available", () => {
    expect(toDrawnRenderLightLevel(ERenderLightLevel.PHYSICAL, false)).toBe(ERenderLightLevel.AUTHORED);
    expect(toDrawnRenderLightLevel(ERenderLightLevel.ANCHORED, false)).toBe(ERenderLightLevel.AUTHORED);
    expect(toDrawnRenderLightLevel(ERenderLightLevel.PHYSICAL, true)).toBe(ERenderLightLevel.PHYSICAL);
    expect(toDrawnRenderLightLevel(ERenderLightLevel.ANCHORED, true)).toBe(ERenderLightLevel.ANCHORED);
    expect(toDrawnRenderLightLevel(ERenderLightLevel.AUTHORED, false)).toBe(ERenderLightLevel.AUTHORED);
  });
});
