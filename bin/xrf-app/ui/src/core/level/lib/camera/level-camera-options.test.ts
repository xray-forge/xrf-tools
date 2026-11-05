import { describe, expect, it } from "@jest/globals";

import {
  DEFAULT_LEVEL_CAMERA_OPTIONS,
  ILevelCameraOptions,
  stepLevelCameraSpeed,
  toLevelCameraOptions,
} from "@/core/level/lib/camera/level-camera-options";

describe("stepLevelCameraSpeed", () => {
  it("steps by a quarter, at least a metre a second a step, within the toolbar's bounds", () => {
    expect(stepLevelCameraSpeed(12, 1)).toBe(15);
    expect(stepLevelCameraSpeed(12, -1)).toBe(10);
    expect(stepLevelCameraSpeed(12, 2)).toBe(19);
    expect(stepLevelCameraSpeed(1, 1)).toBe(2);
    expect(stepLevelCameraSpeed(2, -1)).toBe(1);
    expect(stepLevelCameraSpeed(1, -1)).toBe(1);
    expect(stepLevelCameraSpeed(110, 1)).toBe(120);
  });
});

describe("toLevelCameraOptions", () => {
  it("reads stored options back as they were set", () => {
    const options: ILevelCameraOptions = { boost: 8, fieldOfView: 90, sensitivity: 0.004, speed: 25 };

    expect(toLevelCameraOptions(JSON.parse(JSON.stringify(options)))).toEqual(options);
  });

  it("holds each value to the bounds the control offers, and defaults what is not a number", () => {
    expect(toLevelCameraOptions({ boost: "fast", fieldOfView: 500, speed: 0 })).toEqual({
      ...DEFAULT_LEVEL_CAMERA_OPTIONS,
      fieldOfView: 120,
      speed: 1,
    });
    expect(toLevelCameraOptions(null)).toEqual(DEFAULT_LEVEL_CAMERA_OPTIONS);
  });
});
