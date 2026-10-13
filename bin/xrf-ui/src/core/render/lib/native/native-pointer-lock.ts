/**
 * Locks the pointer to an element, hiding it and reporting only how far the mouse moves.
 *
 * Raw motion is the mouse's own counts, free of the system's acceleration, as the game reads it; a browser refusing it
 * falls back to the system's adjusted motion. Locking needs the user activation of a recent press, so it can be refused
 * outright, and the caller then drags unlocked.
 *
 * @param element - What the pointer is locked to.
 * @param isRaw - Whether to ask for raw motion first.
 * @returns Whether the pointer is locked.
 */
export async function requestNativePointerLock(element: HTMLElement, isRaw: boolean): Promise<boolean> {
  if (isRaw) {
    try {
      await element.requestPointerLock({ unadjustedMovement: true });

      return true;
    } catch {
      // Raw motion refused; the adjusted lock below may still be granted.
    }
  }

  try {
    await element.requestPointerLock();

    return true;
  } catch {
    return false;
  }
}
