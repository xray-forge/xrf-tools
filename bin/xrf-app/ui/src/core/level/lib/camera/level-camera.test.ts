import { describe, expect, it } from "@jest/globals";

import {
  formatLevelFacing,
  formatLevelPoint,
  formatLevelPosition,
  ILevelCamera,
} from "@/core/level/lib/camera/level-camera";

const CAMERA: ILevelCamera = {
  heading: Math.PI / 2,
  pitch: -Math.PI / 12,
  position: { x: -243.75, y: 12.5, z: 87.25 },
};

describe("level camera readout", () => {
  it("states a point a decimal an axis, in the engine's own axis order", () => {
    expect(formatLevelPoint({ x: 1, y: -2.04, z: 3.96 })).toBe("x 1.0 y -2.0 z 4.0");
  });

  it("states where the camera stands as a point", () => {
    expect(formatLevelPosition(CAMERA)).toBe("x -243.8 y 12.5 z 87.3");
  });

  it("states which way the camera faces in degrees, heading then pitch", () => {
    expect(formatLevelFacing(CAMERA)).toBe("h 90.0° p -15.0°");
  });
});
