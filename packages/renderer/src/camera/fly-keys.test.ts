import { describe, expect, it } from "@jest/globals";

import { EFlyKey, getFlyKey } from "#/camera/fly-keys";

describe("getFlyKey", () => {
  it("answers both WASD and the arrows, with rising and sinking on E and Q", () => {
    expect(["KeyW", "KeyS", "KeyA", "KeyD", "KeyE", "KeyQ"].map(getFlyKey)).toEqual([
      EFlyKey.FORWARD,
      EFlyKey.BACK,
      EFlyKey.LEFT,
      EFlyKey.RIGHT,
      EFlyKey.UP,
      EFlyKey.DOWN,
    ]);
    expect(["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].map(getFlyKey)).toEqual([
      EFlyKey.FORWARD,
      EFlyKey.BACK,
      EFlyKey.LEFT,
      EFlyKey.RIGHT,
    ]);
  });

  it("boosts on either shift, and ignores every other key", () => {
    expect(getFlyKey("ShiftLeft")).toBe(EFlyKey.FAST);
    expect(getFlyKey("ShiftRight")).toBe(EFlyKey.FAST);
    expect(getFlyKey("KeyZ")).toBeUndefined();
  });
});
