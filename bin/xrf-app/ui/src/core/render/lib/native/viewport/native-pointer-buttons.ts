import { WorldInputButtons } from "@xrf/ipc/types/xrf-world";

/** No button held. */
export const NO_POINTER_BUTTONS: WorldInputButtons = { auxiliary: false, main: false, secondary: false };

/**
 * @param buttons - `PointerEvent.buttons`: a bit a button held, 1 the main, 2 the secondary and 4 the auxiliary.
 * @returns The buttons held among those a drag tells the world of.
 */
export function toPointerButtons(buttons: number): WorldInputButtons {
  return { auxiliary: (buttons & 4) !== 0, main: (buttons & 1) !== 0, secondary: (buttons & 2) !== 0 };
}

/**
 * @param buttons - The buttons held.
 * @param button - `PointerEvent.button`, the one that went down or up: 0 the main, 1 the auxiliary, 2 the secondary.
 * @returns The buttons held once that one changed; the same for a button a drag does not tell of.
 */
export function togglePointerButton(buttons: WorldInputButtons, button: number): WorldInputButtons {
  switch (button) {
    case 0:
      return { ...buttons, main: !buttons.main };

    case 1:
      return { ...buttons, auxiliary: !buttons.auxiliary };

    case 2:
      return { ...buttons, secondary: !buttons.secondary };

    default:
      return buttons;
  }
}

/** @returns Whether two sets hold the same buttons. */
export function isSamePointerButtons(first: WorldInputButtons, second: WorldInputButtons): boolean {
  return first.main === second.main && first.secondary === second.secondary && first.auxiliary === second.auxiliary;
}

/** @returns Whether a set holds no button. */
export function isNoPointerButton(buttons: WorldInputButtons): boolean {
  return isSamePointerButtons(buttons, NO_POINTER_BUTTONS);
}

/** @returns Whether the main button is held and no other. */
export function isMainPointerButtonOnly(buttons: WorldInputButtons): boolean {
  return buttons.main && !buttons.secondary && !buttons.auxiliary;
}

/** @returns Whether the secondary button is held and no other. */
export function isSecondaryPointerButtonOnly(buttons: WorldInputButtons): boolean {
  return buttons.secondary && !buttons.main && !buttons.auxiliary;
}
