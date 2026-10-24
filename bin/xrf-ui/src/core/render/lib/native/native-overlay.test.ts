import { describe, expect, it } from "@jest/globals";
import { ERenderOverlay, RenderOverlaySegment } from "@xrf/ipc/types/xrf-renderer";

import { toNativeLines } from "@/core/render/lib/native/native-overlay";

const SEGMENT: RenderOverlaySegment = {
  colors: [
    [1, 0, 0],
    [0, 1, 0],
  ],
  ends: [
    [0, 0, 0],
    [1, 2, 3],
  ],
};

describe("toNativeLines", () => {
  it("sends the segments as a line overlay, hidden behind the scene or drawn over it as asked", () => {
    expect(toNativeLines([SEGMENT], true)).toEqual({
      isDepthTested: true,
      kind: ERenderOverlay.LINES,
      segments: [SEGMENT],
    });
    expect(toNativeLines([SEGMENT], false)).toEqual({
      isDepthTested: false,
      kind: ERenderOverlay.LINES,
      segments: [SEGMENT],
    });
  });
});
