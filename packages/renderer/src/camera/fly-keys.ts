import { Maybe } from "@xrf/types";

/**
 * What a key does to a flying camera.
 */
export enum EFlyKey {
  FORWARD = "forward",
  BACK = "back",
  LEFT = "left",
  RIGHT = "right",
  UP = "up",
  DOWN = "down",
  /** Held to move faster, which a level the size of Zaton needs to cross at all. */
  FAST = "fast",
}

/** Keyboard codes each direction answers to, laid out for both WASD and the arrow keys. */
const FLY_KEYS: Readonly<Record<string, EFlyKey>> = {
  ArrowDown: EFlyKey.BACK,
  ArrowLeft: EFlyKey.LEFT,
  ArrowRight: EFlyKey.RIGHT,
  ArrowUp: EFlyKey.FORWARD,
  KeyA: EFlyKey.LEFT,
  KeyD: EFlyKey.RIGHT,
  KeyE: EFlyKey.UP,
  KeyQ: EFlyKey.DOWN,
  KeyS: EFlyKey.BACK,
  KeyW: EFlyKey.FORWARD,
  ShiftLeft: EFlyKey.FAST,
  ShiftRight: EFlyKey.FAST,
};

/**
 * @param code - `KeyboardEvent.code`, which is layout independent so `W` is the same key on azerty.
 * @returns What that key does, or nothing for a key the camera ignores.
 */
export function getFlyKey(code: string): Maybe<EFlyKey> {
  return FLY_KEYS[code];
}
