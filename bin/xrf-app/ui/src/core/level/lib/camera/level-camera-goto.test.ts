import { describe, expect, it } from "@jest/globals";
import { toXrayHeading, toXraySpace } from "@xrf/math";

import { formatLevelFacing, formatLevelPosition, ILevelCamera } from "@/core/level/lib/camera/level-camera";
import {
  formatLevelGoTo,
  ILevelGoTo,
  isLevelGoTo,
  parseLevelGoTo,
  toLevelGoTo,
  toLevelGoToViewpoint,
  toRoundedLevelGoTo,
} from "@/core/level/lib/camera/level-camera-goto";
import { ILevelViewpoint } from "@/core/level/lib/camera/level-viewpoint";

const HERE: ILevelGoTo = { heading: 0, pitch: 0, x: 0, y: 0, z: 0 };

describe("isLevelGoTo", () => {
  it("takes only a whole place of finite numbers", () => {
    expect(isLevelGoTo({ heading: 90, pitch: -4, x: 1, y: 2, z: 3 })).toBe(true);
    expect(isLevelGoTo({ heading: 90, pitch: -4, x: 1, y: 2 })).toBe(false);
    expect(isLevelGoTo({ heading: 90, pitch: -4, x: 1, y: 2, z: Number.NaN })).toBe(false);
    expect(isLevelGoTo({ heading: "90", pitch: -4, x: 1, y: 2, z: 3 })).toBe(false);
  });
});

describe("toRoundedLevelGoTo", () => {
  it("rounds every value to the decimals asked", () => {
    expect(toRoundedLevelGoTo({ heading: 252.80049, pitch: -4.0004, x: 344.70051, y: -2.3, z: 247.7 }, 3)).toEqual({
      heading: 252.8,
      pitch: -4,
      x: 344.701,
      y: -2.3,
      z: 247.7,
    });
  });
});

describe("parseLevelGoTo", () => {
  it("reads the readout as it is copied, degree signs and all", () => {
    expect(parseLevelGoTo("x -394.9 y 4.6 z 167.7\nh 102.2° p -21.7°", HERE)).toEqual({
      heading: 102.2,
      pitch: -21.7,
      x: -394.9,
      y: 4.6,
      z: 167.7,
    });
  });

  it("keeps what the text leaves out, and reads nothing from text naming none of it", () => {
    expect(parseLevelGoTo("y 12", { ...HERE, x: 5 })).toEqual({ ...HERE, x: 5, y: 12 });
    expect(parseLevelGoTo("somewhere", HERE)).toBeNull();
  });

  // A config writes a position as bare numbers, `position = -394.9, 4.6, 167.7`, in the readout's own order.
  it("reads bare numbers in the readout's order, keeping what they leave out", () => {
    expect(parseLevelGoTo("position = -394.9, 4.6, 167.7", { ...HERE, heading: 30 })).toEqual({
      heading: 30,
      pitch: 0,
      x: -394.9,
      y: 4.6,
      z: 167.7,
    });
    expect(parseLevelGoTo("1 2 3 90 -10 77", HERE)).toEqual({ heading: 90, pitch: -10, x: 1, y: 2, z: 3 });
  });
});

describe("toLevelGoToViewpoint", () => {
  // What the readout says of a camera stood where it says is what it said.
  it("stands the camera where the readout would read it back", () => {
    const goTo: ILevelGoTo = { heading: 102.2, pitch: -21.7, x: -394.9, y: 4.6, z: 167.7 };
    const { position, target }: ILevelViewpoint = toLevelGoToViewpoint(goTo);
    const camera: ILevelCamera = {
      ...toXrayHeading({ x: target.x - position.x, y: target.y - position.y, z: target.z - position.z }),
      position: toXraySpace(position),
    };

    expect(formatLevelPosition(camera)).toBe("x -394.9 y 4.6 z 167.7");
    expect(formatLevelFacing(camera)).toBe(formatLevelFacing({ ...camera, ...toLevelGoToCameraAngles(goTo) }));
    expect(toLevelGoTo(camera).heading).toBeCloseTo(102.2);
    expect(toLevelGoTo(camera).pitch).toBeCloseTo(-21.7);
  });

  it("faces short of straight down, which would leave the camera no heading", () => {
    const { position, target }: ILevelViewpoint = toLevelGoToViewpoint({ ...HERE, heading: 30, pitch: -90 });

    expect(Math.hypot(target.x - position.x, target.z - position.z)).toBeGreaterThan(0);
  });
});

/** A go-to's angles as the camera states them, in radians. */
function toLevelGoToCameraAngles(goTo: ILevelGoTo): Pick<ILevelCamera, "heading" | "pitch"> {
  return { heading: (goTo.heading * Math.PI) / 180, pitch: (goTo.pitch * Math.PI) / 180 };
}

describe("formatLevelGoTo", () => {
  it("states a place as the camera readout does, which parseLevelGoTo reads back to a decimal", () => {
    const goTo: ILevelGoTo = { heading: 102.24, pitch: -21.66, x: -394.92, y: 4.6, z: 167.7 };
    const camera: ILevelCamera = { ...toLevelGoToCameraAngles(goTo), position: { x: goTo.x, y: goTo.y, z: goTo.z } };

    expect(formatLevelGoTo(goTo)).toBe(`${formatLevelPosition(camera)} ${formatLevelFacing(camera)}`);
    expect(parseLevelGoTo(formatLevelGoTo(goTo), HERE)).toEqual({
      heading: 102.2,
      pitch: -21.7,
      x: -394.9,
      y: 4.6,
      z: 167.7,
    });
  });
});
