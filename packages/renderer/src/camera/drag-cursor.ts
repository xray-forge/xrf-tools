import { Nullable } from "@xrf/types";

/** What the pointer shows while it is dragging a scene, matching the picture viewport's own drag cursor. */
export const DRAG_CURSOR: string = "grabbing";

/** Whatever carries a cursor: an element on a page, or what stands in for one on a thread without a page. */
export interface ICursorTarget {
  style: { cursor: string };
}

/** Controls that say when a drag starts and ends, as three's `OrbitControls` do. */
export interface IDragControls {
  addEventListener(type: "start" | "end", listener: () => void): void;
  removeEventListener(type: "start" | "end", listener: () => void): void;
}

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
