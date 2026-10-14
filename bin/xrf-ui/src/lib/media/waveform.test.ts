import { describe, expect, it } from "@jest/globals";

import { extractPeaks } from "@/lib/media/waveform";

describe("extractPeaks", () => {
  it("keeps the loudest magnitude in each bucket", () => {
    // Averaging instead would smear a single transient into the quiet around it, which is exactly the
    // thing someone opens a waveform to find.
    const samples: Float32Array = new Float32Array([0, 0, 0.9, 0, 0.1, 0.2, 0, 0]);

    // Compared loosely because a Float32Array cannot hold 0.9 exactly.
    expect(Array.from(extractPeaks(samples, 2))).toEqual([expect.closeTo(0.9), expect.closeTo(0.2)]);
  });

  it("treats magnitude as absolute, so troughs count as loudly as peaks", () => {
    expect(Array.from(extractPeaks(new Float32Array([-1, 0.5]), 1))).toEqual([1]);
  });

  it("covers the whole signal when buckets do not divide it evenly", () => {
    const samples: Float32Array = new Float32Array([0.1, 0.2, 0.3, 0.4, 0.5]);
    const peaks: Float32Array = extractPeaks(samples, 3);

    // The last sample must land in some bucket rather than being dropped by rounding.
    expect(peaks).toHaveLength(3);
    expect(Math.max(...Array.from(peaks))).toBeCloseTo(0.5);
  });

  it("never produces fewer buckets than asked for, even for a very short sound", () => {
    expect(extractPeaks(new Float32Array([0.5]), 8)).toHaveLength(8);
  });

  it("returns silence for empty input", () => {
    expect(Array.from(extractPeaks(new Float32Array(), 4))).toEqual([0, 0, 0, 0]);
    expect(extractPeaks(new Float32Array([1]), 0)).toHaveLength(0);
  });
});
