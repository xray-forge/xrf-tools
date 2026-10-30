import { describe, expect, it } from "@jest/globals";
import { toRadians } from "@xrf/math";

import { toLevelFramedGoTo } from "@/core/level/lib/camera/level-camera-frame";
import { ILevelGoTo } from "@/core/level/lib/camera/level-camera-goto";

describe("toLevelFramedGoTo", () => {
  // Facing `+z` and down, the camera stands back along `-z` and up; the sphere's `z` is renderer space, so the engine's
  // is +10.
  it("keeps the camera's rotation and stands it back until the sphere fills the view, with room round it", () => {
    const goTo: ILevelGoTo = toLevelFramedGoTo(
      { heading: 0, pitch: toRadians(-30), position: { x: 0, y: 0, z: 0 } },
      { center: { x: 1, y: 2, z: -10 }, radius: 4 },
      60
    );

    expect(goTo.heading).toBe(0);
    expect(goTo.pitch).toBeCloseTo(-30);
    expect(goTo.x).toBeCloseTo(1);
    // Four metres and a quarter more room, over the sine of half of sixty degrees: ten metres back from the centre.
    expect(goTo.y).toBeCloseTo(2 + 10 * Math.sin(toRadians(30)));
    expect(goTo.z).toBeCloseTo(10 - 10 * Math.cos(toRadians(30)));
  });

  // A path lying on the ground, framed level or from below, would stand the camera inside the terrain behind it.
  it("looks down at least twenty degrees, so it stands above what it frames", () => {
    for (const pitch of [0, 30]) {
      const goTo: ILevelGoTo = toLevelFramedGoTo(
        { heading: toRadians(45), pitch: toRadians(pitch), position: { x: 0, y: 0, z: 0 } },
        { center: { x: 0, y: 40, z: 0 }, radius: 4 },
        60
      );

      expect(goTo.heading).toBeCloseTo(45);
      expect(goTo.pitch).toBeCloseTo(-20);
      expect(goTo.y).toBeCloseTo(40 + 10 * Math.sin(toRadians(20)));
    }
  });

  it("stands no nearer than two metres to something small", () => {
    const goTo: ILevelGoTo = toLevelFramedGoTo(
      { heading: toRadians(90), pitch: toRadians(-30), position: { x: 0, y: 0, z: 0 } },
      { center: { x: 0, y: 0, z: 0 }, radius: 0.05 },
      60
    );

    expect(Math.hypot(goTo.x, goTo.y, goTo.z)).toBeCloseTo(2);
    expect(goTo.heading).toBeCloseTo(90);
    expect(goTo.pitch).toBeCloseTo(-30);
    // Looking down, it stands above.
    expect(goTo.y).toBeCloseTo(1);
  });

  it("faces level and a little down along +z before the camera has reported", () => {
    const goTo: ILevelGoTo = toLevelFramedGoTo(null, { center: { x: 0, y: 0, z: 0 }, radius: 1 }, 60);

    expect(goTo.heading).toBe(0);
    expect(goTo.pitch).toBeCloseTo(-20);
  });
});
