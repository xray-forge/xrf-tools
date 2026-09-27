import { describe, expect, it } from "@jest/globals";

import { formatLevelFacing, formatLevelPosition, ILevelCamera } from "@/core/level/lib/camera/level-camera";
import {
  ILevelGoTo,
  isLevelGoToText,
  parseLevelGoTo,
  parseLevelGoToTexts,
  toLevelGoTo,
  toLevelGoToTexts,
  toLevelGoToViewpoint,
} from "@/core/level/lib/camera/level-camera-goto";
import { ILevelViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { toXrayHeading, toXraySpace } from "@/core/render/lib/scene/render-space";

const HERE: ILevelGoTo = { heading: 0, pitch: 0, x: 0, y: 0, z: 0 };

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

describe("parseLevelGoToTexts", () => {
  it("reads back what toLevelGoToTexts shows, a decimal each", () => {
    const goTo: ILevelGoTo = { heading: 102.24, pitch: -21.66, x: -394.92, y: 4.6, z: 167.7 };

    expect(toLevelGoToTexts(goTo)).toEqual({ heading: "102.2", pitch: "-21.7", x: "-394.9", y: "4.6", z: "167.7" });
    expect(parseLevelGoToTexts(toLevelGoToTexts(goTo))).toEqual({
      heading: 102.2,
      pitch: -21.7,
      x: -394.9,
      y: 4.6,
      z: 167.7,
    });
  });

  it("reads nothing while a value is empty or not a number", () => {
    const texts: Record<keyof ILevelGoTo, string> = { heading: "0", pitch: "0", x: "1", y: "2", z: "3" };

    expect(isLevelGoToText(" ")).toBe(false);
    expect(isLevelGoToText("-4.5")).toBe(true);
    expect(parseLevelGoToTexts({ ...texts, y: "" })).toBeNull();
    expect(parseLevelGoToTexts({ ...texts, z: "abc" })).toBeNull();
  });
});
