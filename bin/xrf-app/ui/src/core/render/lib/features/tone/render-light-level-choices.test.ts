import { describe, expect, it } from "@jest/globals";
import { ERenderLightLevel } from "@xrf/ipc/types/xrf-renderer";

import { RENDER_LIGHT_LEVEL_OPTIONS } from "@/core/render/lib/features/tone/render-light-level-choices";

describe("RENDER_LIGHT_LEVEL_OPTIONS", () => {
  it("offers every light level, none disabled", () => {
    expect(RENDER_LIGHT_LEVEL_OPTIONS.map((option) => option.value)).toEqual([
      ERenderLightLevel.AUTHORED,
      ERenderLightLevel.ANCHORED,
      ERenderLightLevel.PHYSICAL,
    ]);
    expect(RENDER_LIGHT_LEVEL_OPTIONS.some((option) => option.isDisabled)).toBe(false);
  });
});
