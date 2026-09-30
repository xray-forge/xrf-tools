import { describe, expect, it } from "@jest/globals";

import { EFlyKey, getFlyKey } from "#/camera/fly-key";

describe("getFlyKey", () => {
  it("moves on WASD, rising and sinking on E and Q, and turns on the arrows", () => {
    expect(["KeyW", "KeyS", "KeyA", "KeyD", "KeyE", "KeyQ"].map(getFlyKey)).toEqual([
      EFlyKey.FORWARD,
      EFlyKey.BACK,
      EFlyKey.LEFT,
      EFlyKey.RIGHT,
      EFlyKey.UP,
      EFlyKey.DOWN,
    ]);
    expect(["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].map(getFlyKey)).toEqual([
      EFlyKey.LOOK_UP,
      EFlyKey.LOOK_DOWN,
      EFlyKey.TURN_LEFT,
      EFlyKey.TURN_RIGHT,
    ]);
  });

  it("boosts on either shift, and ignores every other key", () => {
    expect(getFlyKey("ShiftLeft")).toBe(EFlyKey.FAST);
    expect(getFlyKey("ShiftRight")).toBe(EFlyKey.FAST);
    expect(getFlyKey("KeyZ")).toBeUndefined();
  });
});
