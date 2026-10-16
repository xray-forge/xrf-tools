import { describe, expect, it } from "@jest/globals";

import { ERenderAntialiasing } from "@/core/ipc/types/xrf-renderer";
import { isTemporalAntialiasing } from "@/core/render/lib/features/render-antialiasing-choices";

describe("temporal antialiasing", () => {
  it("tells the modes that resolve frames over time, which sharpening applies to at any size", () => {
    expect(Object.values(ERenderAntialiasing).filter(isTemporalAntialiasing)).toEqual([
      ERenderAntialiasing.TAA,
      ERenderAntialiasing.FSR2,
      ERenderAntialiasing.DLSS,
    ]);
  });
});
