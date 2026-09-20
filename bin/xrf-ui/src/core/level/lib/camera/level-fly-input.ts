import { Nullable } from "@/lib/types/general";

export interface ILevelFlyInput {
  forward: boolean;
  back: boolean;
  left: boolean;
  right: boolean;
  up: boolean;
  down: boolean;
  /** Held to move faster, which a level the size of Zaton needs to cross at all. */
  fast: boolean;
}

export const EMPTY_LEVEL_FLY_INPUT: ILevelFlyInput = {
  back: false,
  down: false,
  fast: false,
  forward: false,
  left: false,
  right: false,
  up: false,
};

/** Keyboard codes each direction answers to, laid out for both WASD and the arrow keys. */
const KEY_BINDINGS: Readonly<Record<string, keyof ILevelFlyInput>> = {
  ArrowDown: "back",
  ArrowLeft: "left",
  ArrowRight: "right",
  ArrowUp: "forward",
  KeyA: "left",
  KeyD: "right",
  KeyE: "up",
  KeyQ: "down",
  KeyS: "back",
  KeyW: "forward",
  ShiftLeft: "fast",
  ShiftRight: "fast",
};

/**
 * @param code - `KeyboardEvent.code`, which is layout independent so `W` is the same key on azerty.
 * @returns The direction that key drives, or null for a key the camera ignores.
 */
export function getFlyBinding(code: string): Nullable<keyof ILevelFlyInput> {
  return KEY_BINDINGS[code] ?? null;
}
