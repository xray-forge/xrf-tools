import { describe, expect, it } from "@jest/globals";

import {
  RENDER_FRAME_RATE_OPTIONS,
  RENDER_RESOLUTION_OPTIONS,
} from "@/core/render/lib/features/render-display-choices";
import { FRAME_RATE_LIMITS } from "@/core/render/lib/settings/render-frame-rate-limit";
import { RENDER_RESOLUTIONS } from "@/core/render/lib/settings/render-resolution";

describe("render display options", () => {
  it("offers every frame rate limit and resolution this build takes, in its order", () => {
    expect(RENDER_FRAME_RATE_OPTIONS.map((it) => it.value)).toEqual(FRAME_RATE_LIMITS);
    expect(RENDER_FRAME_RATE_OPTIONS[0]?.label).toBe("No cap");
    expect(RENDER_RESOLUTION_OPTIONS.map((it) => it.value)).toEqual(RENDER_RESOLUTIONS);
    expect(RENDER_RESOLUTION_OPTIONS.map((it) => it.label)).toEqual(["Auto", "720p", "1080p", "1440p", "4K"]);
  });
});
