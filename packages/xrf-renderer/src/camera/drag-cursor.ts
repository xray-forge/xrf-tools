import { Nullable } from "@xrf/types";

import { ICursorTarget } from "#/camera/cursor-target";
import { IDragControls } from "#/camera/drag-controls";

/** What the pointer shows while it is dragging a scene, matching the picture viewport's own drag cursor. */
const DRAG_CURSOR: string = "grabbing";

/**
 * A scene's drag shown on the cursor, for as long as the drag lasts, and the cursor given back after.
 */
export class DragCursor {
  private readonly element: ICursorTarget;
  /** The cursor the element had before a drag took it, or null while nothing drags. */
  private resting: Nullable<string> = null;

  /**
   * @param element - Whatever carries the cursor.
   */
  public constructor(element: ICursorTarget) {
    this.element = element;
  }

  public start(): void {
    this.resting ??= this.element.style.cursor;
    this.element.style.cursor = DRAG_CURSOR;
  }

  public end(): void {
    if (this.resting !== null) {
      this.element.style.cursor = this.resting;
      this.resting = null;
    }
  }
}

/**
 * Shows a scene's drag on the cursor, for as long as the drag lasts.
 *
 * @param controls - Controls whose drag the cursor answers.
 * @param element - Whatever carries the cursor, which is the element those controls listen to.
 * @returns Unbinds the listeners and leaves the cursor as it was found.
 */
export function bindDragCursor(controls: IDragControls, element: ICursorTarget): () => void {
  const cursor: DragCursor = new DragCursor(element);

  function onStart(): void {
    cursor.start();
  }

  function onEnd(): void {
    cursor.end();
  }

  controls.addEventListener("start", onStart);
  controls.addEventListener("end", onEnd);

  return () => {
    controls.removeEventListener("start", onStart);
    controls.removeEventListener("end", onEnd);
    cursor.end();
  };
}
