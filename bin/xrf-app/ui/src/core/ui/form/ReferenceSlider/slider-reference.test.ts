import { describe, expect, it } from "@jest/globals";

import {
  describeSliderRecommendation,
  SLIDER_ABOVE_RECOMMENDED,
  SLIDER_BELOW_RECOMMENDED,
  SLIDER_WASTED,
} from "@/core/ui/form/ReferenceSlider/slider-reference";

describe("describeSliderRecommendation", () => {
  it("says nothing within the range or for an unrated setting, and which side a value outside it stands", () => {
    expect(describeSliderRecommendation(32, [32, 96])).toBeNull();
    expect(describeSliderRecommendation(96, [32, 96])).toBeNull();
    expect(describeSliderRecommendation(500, undefined)).toBeNull();
    expect(describeSliderRecommendation(16, [32, 96])).toBe(SLIDER_BELOW_RECOMMENDED);
    expect(describeSliderRecommendation(128, [32, 96])).toBe(SLIDER_ABOVE_RECOMMENDED);
  });

  it("names a value past the measured useful end as wasted, ranged or not, and the end itself as not", () => {
    expect(describeSliderRecommendation(8, [4, 8], 8)).toBeNull();
    expect(describeSliderRecommendation(12, [4, 8], 8)).toBe(SLIDER_WASTED);
    expect(describeSliderRecommendation(4, undefined, 2)).toBe(SLIDER_WASTED);
    expect(describeSliderRecommendation(2, [4, 8], 8)).toBe(SLIDER_BELOW_RECOMMENDED);
  });
});
