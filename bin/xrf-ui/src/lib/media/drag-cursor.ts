import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

/** What the pointer shows while it is dragging a scene, matching the picture viewport's own drag cursor. */
const DRAG_CURSOR: string = "grabbing";

/**
 * Shows a scene's drag on the cursor, for as long as the drag lasts.
 *
 * Not a css rule, and it cannot be one: `OrbitControls` writes `cursor: auto` on the element it is given, so nothing
 * inherited from an ancestor ever reaches the canvas, and it captures the pointer on press, so no `:active` rule
 * matches either. The controls say it themselves - `start` when a gesture takes hold, `end` when it lets go - and a
 * wheel notch dispatches both in one task, so zooming never flashes the hand. Three's own `cursorStyle` is not this:
 * it offers `auto` or `grab`, and `grab` stands a hand on the canvas whether or not anything is being dragged.
 *
 * @param controls - Controls whose drag the cursor answers.
 * @param element - Element to write the cursor on, which is the one those controls listen to.
 * @returns Unbinds the listeners and leaves the cursor as it was found.
 */
export function bindDragCursor(controls: OrbitControls, element: HTMLElement): () => void {
  const original: string = element.style.cursor;

  function onStart(): void {
    element.style.cursor = DRAG_CURSOR;
  }

  function onEnd(): void {
    element.style.cursor = original;
  }

  controls.addEventListener("start", onStart);
  controls.addEventListener("end", onEnd);

  return () => {
    controls.removeEventListener("start", onStart);
    controls.removeEventListener("end", onEnd);
    element.style.cursor = original;
  };
}
