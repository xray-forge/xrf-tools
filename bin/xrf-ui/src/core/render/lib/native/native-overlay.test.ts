import { describe, expect, it } from "@jest/globals";

import { ERenderOverlay, RenderOverlaySegment } from "@/core/ipc/types/xrf-renderer";
import { toNativeLines } from "@/core/render/lib/native/native-overlay";
import { toRenderAxesLines } from "@/core/render/lib/scene/render-grid-lines";

describe("toNativeLines", () => {
  it("sends each segment with its two ends and their colours", () => {
    const segments: Array<RenderOverlaySegment> = toRenderAxesLines(2);

    expect(toNativeLines(segments, true)).toEqual({ isDepthTested: true, kind: ERenderOverlay.LINES, segments });
    expect(segments[1]).toEqual({
      colors: [
        [0, 1, 0],
        [0.797, 1, 0],
      ],
      ends: [
        [0, 0, 0],
        [0, 2, 0],
      ],
    });
  });
});
