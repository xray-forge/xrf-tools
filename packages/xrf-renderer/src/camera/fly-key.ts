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
  TURN_LEFT = "turnLeft",
  TURN_RIGHT = "turnRight",
  LOOK_UP = "lookUp",
  LOOK_DOWN = "lookDown",
}

/** Keyboard codes each key answers to: WASD, `E` and `Q` move the camera, and the arrow keys turn it. */
const FLY_KEYS: Readonly<Record<string, EFlyKey>> = {
  ArrowDown: EFlyKey.LOOK_DOWN,
  ArrowLeft: EFlyKey.TURN_LEFT,
  ArrowRight: EFlyKey.TURN_RIGHT,
  ArrowUp: EFlyKey.LOOK_UP,
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
