import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

/** What the pointer shows while it is dragging a scene, matching the picture viewport's own drag cursor. */
export const DRAG_CURSOR: string = "grabbing";

/** Whatever carries a cursor: an element on a page, or what stands in for one on a thread without a page. */
export interface ICursorTarget {
  style: { cursor: string };
}

/**
 * Shows a scene's drag on the cursor, for as long as the drag lasts.
 *
 * @param controls - Controls whose drag the cursor answers.
 * @param element - Whatever carries the cursor, which is the element those controls listen to.
 * @returns Unbinds the listeners and leaves the cursor as it was found.
 */
export function bindDragCursor(controls: OrbitControls, element: ICursorTarget): () => void {
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
