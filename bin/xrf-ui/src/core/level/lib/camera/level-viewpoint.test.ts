import { describe, expect, it } from "@jest/globals";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { mockVisualBounds } from "@/fixtures/mocks/visual.mocks";

function level(): VisualBounds {
  return mockVisualBounds({
    boundingBox: { max: { x: 600, y: 90, z: -100 }, min: { x: -400, y: -10, z: -600 } },
  });
}

describe("toLevelStartViewpoint", () => {
  // Opened from outside, a level is a diorama: every surface past the distance at which anything is legible, and the
  // streamer reading in the sectors furthest from where somebody is about to fly.
  it("stands in the middle of the level's extent", () => {
    const viewpoint: ILevelViewpoint = toLevelStartViewpoint(level());

    expect(viewpoint.position).toEqual({ x: 100, y: 40, z: -350 });
  });

  it("looks level with the horizon, down the longer side", () => {
    const viewpoint: ILevelViewpoint = toLevelStartViewpoint(level());

    // The same height as the camera, so the view opens on the place rather than on the ground under it.
    expect(viewpoint.target.y).toBe(viewpoint.position.y);
    // 1000 across against 500 deep, so the camera faces along x.
    expect(viewpoint.target.x).toBeGreaterThan(viewpoint.position.x);
    expect(viewpoint.target.z).toBe(viewpoint.position.z);
  });

  it("faces the other way for a level that is deeper than it is wide", () => {
    const viewpoint: ILevelViewpoint = toLevelStartViewpoint(
      mockVisualBounds({ boundingBox: { max: { x: 100, y: 10, z: 500 }, min: { x: -100, y: -10, z: -500 } } })
    );

    expect(viewpoint.target.z).toBeLessThan(viewpoint.position.z);
    expect(viewpoint.target.x).toBe(viewpoint.position.x);
  });

  it("opens at the origin for a level that reports no extent", () => {
    const viewpoint: ILevelViewpoint = toLevelStartViewpoint(null);

    expect(viewpoint.position).toEqual({ x: 0, y: 0, z: 0 });
    // Still a direction to look in, because a camera told to look at where it stands keeps whatever it had.
    expect(viewpoint.target).not.toEqual(viewpoint.position);
  });

  // Every float is optional on the wire, and a corner that arrives half stated is not a reason to place the camera
  // somewhere arbitrary.
  it("reads a corner missing a component as that component being zero", () => {
    const viewpoint: ILevelViewpoint = toLevelStartViewpoint(
      mockVisualBounds({ boundingBox: { max: { x: 10, y: null, z: 10 }, min: { x: -10, y: null, z: -10 } } })
    );

    expect(viewpoint.position).toEqual({ x: 0, y: 0, z: 0 });
  });
});
