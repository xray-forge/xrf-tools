import { describe, expect, it } from "@jest/globals";

import { ERenderOverlay, RenderOverlay } from "@/core/ipc/types/xrf-renderer";
import { toNativeLines } from "@/core/render/lib/native/native-overlay";

describe("toNativeLines", () => {
  it("sends each segment with its two ends and their colours", () => {
    const overlay: RenderOverlay = toNativeLines(
      {
        colors: new Float32Array([1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1]),
        positions: new Float32Array([0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 2, 3]),
      },
      true
    );

    expect(overlay).toEqual({
      isDepthTested: true,
      kind: ERenderOverlay.LINES,
      segments: [
        {
          colors: [
            [1, 0, 0],
            [0, 1, 0],
          ],
          ends: [
            [0, 0, 0],
            [1, 0, 0],
          ],
        },
        {
          colors: [
            [0, 0, 1],
            [1, 1, 1],
          ],
          ends: [
            [0, 2, 0],
            [0, 2, 3],
          ],
        },
      ],
    });
  });
});
