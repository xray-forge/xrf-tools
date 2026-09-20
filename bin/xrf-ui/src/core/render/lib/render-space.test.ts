import { describe, expect, it } from "@jest/globals";

import { IXrayHeading, toRendererSpace, toXrayHeading, toXraySpace } from "@/core/render/lib/render-space";

describe("render space", () => {
  // The packer negates `z` on the way out, so anything read back for a person has to negate it again or it disagrees
  // with the level's own data - a spawn point, a config coordinate - by a sign on one axis only.
  it("negates the axis the packer negated, and leaves the other two", () => {
    expect(toXraySpace({ x: -243.75, y: 12.5, z: 87.25 })).toEqual({ x: -243.75, y: 12.5, z: -87.25 });
    expect(toRendererSpace({ x: -243.75, y: 12.5, z: -87.25 })).toEqual({ x: -243.75, y: 12.5, z: 87.25 });
  });

  it("round trips", () => {
    const point = { x: 1.5, y: -2.25, z: 3.75 };

    expect(toRendererSpace(toXraySpace(point))).toEqual(point);
  });

  it("leaves the origin where it is, which is the one place both spaces agree on", () => {
    expect(toXraySpace({ x: 0, y: 0, z: 0 })).toEqual({ x: 0, y: 0, z: 0 });
  });
});

describe("toXrayHeading", () => {
  /** What `Fvector::setHP` builds from a heading and a pitch, in renderer space for the reader to hand back. */
  function facing(heading: number, pitch: number) {
    return toRendererSpace({
      x: -Math.cos(pitch) * Math.sin(heading),
      y: Math.sin(pitch),
      z: Math.cos(pitch) * Math.cos(heading),
    });
  }

  /** The same direction the engine names, counted round one turn. */
  function turned(radians: number): number {
    return ((radians % (Math.PI * 2)) + Math.PI * 2) % (Math.PI * 2);
  }

  // The inverse of the engine's own `setHP`, which is the only definition of a heading that matches the game.
  it("reads back what the engine's setHP writes", () => {
    for (const heading of [0, Math.PI / 4, Math.PI / 2, -Math.PI / 2, 2.5, 3.9]) {
      for (const pitch of [0, 0.4, -0.4, 1.2]) {
        const read: IXrayHeading = toXrayHeading(facing(heading, pitch));

        expect(read.heading).toBeCloseTo(turned(heading), 5);
        expect(read.pitch).toBeCloseTo(pitch, 5);
      }
    }
  });

  // Not the engine's own range, which runs from -pi/2 to 3pi/2 and so gives one direction two signs.
  it("counts a heading round a full turn rather than about zero", () => {
    expect(toXrayHeading(facing(-0.5, 0)).heading).toBeCloseTo(Math.PI * 2 - 0.5, 5);
  });

  it("faces +z at a heading of zero, in the level's own axes", () => {
    // Renderer -z is the level's +z, which is exactly the trap this conversion exists to close.
    expect(toXrayHeading({ x: 0, y: 0, z: -1 }).heading).toBeCloseTo(0);
    expect(toXrayHeading({ x: 0, y: 0, z: 1 }).heading).toBeCloseTo(Math.PI);
  });

  it("reads straight up and straight down without a heading it cannot know", () => {
    expect(toXrayHeading({ x: 0, y: 1, z: 0 })).toEqual({ heading: 0, pitch: Math.PI / 2 });
    expect(toXrayHeading({ x: 0, y: -1, z: 0 })).toEqual({ heading: 0, pitch: -Math.PI / 2 });
  });

  it("does not need a normalized direction", () => {
    expect(toXrayHeading({ x: 0, y: 0, z: -8 }).heading).toBeCloseTo(toXrayHeading({ x: 0, y: 0, z: -1 }).heading);
  });
});
