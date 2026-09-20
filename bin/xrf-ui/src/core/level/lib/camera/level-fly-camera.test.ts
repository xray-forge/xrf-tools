import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera, Vector3 } from "three";

import {
  DEFAULT_LEVEL_FLY_OPTIONS,
  EMPTY_LEVEL_FLY_INPUT,
  getFlyBinding,
  ILevelFlyInput,
  LevelFlyCamera,
} from "@/core/level/lib/camera/level-fly-camera";

function input(overrides: Partial<ILevelFlyInput> = {}): ILevelFlyInput {
  return { ...EMPTY_LEVEL_FLY_INPUT, ...overrides };
}

describe("level fly camera", () => {
  it("binds the movement keys by code so the layout does not matter", () => {
    expect(getFlyBinding("KeyW")).toBe("forward");
    expect(getFlyBinding("ArrowUp")).toBe("forward");
    expect(getFlyBinding("ShiftLeft")).toBe("fast");
    expect(getFlyBinding("KeyZ")).toBeNull();
  });

  it("moves along the direction it is facing", () => {
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const fly: LevelFlyCamera = new LevelFlyCamera();

    fly.update(camera, input({ forward: true }), 1);

    // Facing down negative z by default, so a second of forward is a second of speed that way.
    expect(camera.position.z).toBeCloseTo(-DEFAULT_LEVEL_FLY_OPTIONS.speed);
    expect(camera.position.x).toBeCloseTo(0);
  });

  // A level the size of Zaton is kilometres across; walking it end to end is not a viable way to look at anything.
  it("multiplies the distance while the boost is held", () => {
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const fly: LevelFlyCamera = new LevelFlyCamera();

    fly.update(camera, input({ fast: true, forward: true }), 1);

    expect(camera.position.z).toBeCloseTo(-DEFAULT_LEVEL_FLY_OPTIONS.speed * DEFAULT_LEVEL_FLY_OPTIONS.boost);
  });

  // Scaled by elapsed time, so a level that streams and stutters still travels at one speed.
  it("travels the same distance however the frames fall", () => {
    const steady: PerspectiveCamera = new PerspectiveCamera();
    const stuttering: PerspectiveCamera = new PerspectiveCamera();
    const fly: LevelFlyCamera = new LevelFlyCamera();

    fly.update(steady, input({ forward: true }), 1);

    for (let index = 0; index < 10; index += 1) {
      fly.update(stuttering, input({ forward: true }), 0.1);
    }

    expect(stuttering.position.z).toBeCloseTo(steady.position.z, 5);
  });

  it("rises along world up rather than where it is looking", () => {
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const fly: LevelFlyCamera = new LevelFlyCamera();

    fly.look(0, 400);
    fly.update(camera, input({ up: true }), 1);

    expect(camera.position.y).toBeCloseTo(DEFAULT_LEVEL_FLY_OPTIONS.speed);
  });

  // Pitching past vertical would roll the horizon over, which is disorienting and has no use.
  it("stops the pitch just short of straight up and straight down", () => {
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const fly: LevelFlyCamera = new LevelFlyCamera();

    fly.look(0, -100_000);
    fly.update(camera, input(), 1);

    const up: Vector3 = new Vector3(0, 1, 0).applyQuaternion(camera.quaternion);

    expect(up.y).toBeGreaterThan(0);
  });

  it("reports whether the camera actually moved", () => {
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const fly: LevelFlyCamera = new LevelFlyCamera();

    expect(fly.update(camera, input(), 1)).toBe(false);
    expect(fly.update(camera, input({ forward: true }), 1)).toBe(true);
  });

  it("aims at a target without moving to it", () => {
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const fly: LevelFlyCamera = new LevelFlyCamera();

    camera.position.set(0, 0, 10);
    fly.lookAt(camera, new Vector3(0, 0, 0));

    const ahead: Vector3 = new Vector3(0, 0, -1).applyQuaternion(camera.quaternion);

    expect(camera.position.z).toBe(10);
    expect(ahead.z).toBeCloseTo(-1);
  });
});
