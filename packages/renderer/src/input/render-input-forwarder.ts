import { ERenderInput, IRenderInputEvent, toRenderInputEvent } from "#/input/render-input";

/** What the forwarder does with a gesture it has taken. */
export type TRenderInputSink = (event: IRenderInputEvent) => void;

/** Gestures taken on the canvas itself, which is what a scene's controls listen to. */
const ELEMENT_INPUT: ReadonlyArray<ERenderInput> = [
  ERenderInput.CONTEXT_MENU,
  ERenderInput.POINTER_CANCEL,
  ERenderInput.POINTER_DOWN,
  ERenderInput.WHEEL,
];

/**
 * Gestures taken from the whole window, so a drag that leaves the canvas is still a drag.
 */
const WINDOW_INPUT: ReadonlyArray<ERenderInput> = [ERenderInput.POINTER_MOVE, ERenderInput.POINTER_UP];

/**
 * Sends what is done to a canvas to whatever is drawing on it.
 */
export class RenderInputForwarder {
  private readonly canvas: HTMLCanvasElement;
  private readonly sink: TRenderInputSink;
  private readonly cursor: string;

  public constructor(canvas: HTMLCanvasElement, sink: TRenderInputSink) {
    this.canvas = canvas;
    this.cursor = canvas.style.cursor;
    this.sink = sink;

    // Scrolling and the browser's own menu belong to the gesture, and only this side can refuse them: a frame
    // spent asking the other thread whether to is a frame the page has already scrolled.
    canvas.style.touchAction = "none";

    for (const type of ELEMENT_INPUT) {
      canvas.addEventListener(type, this.onElementInput, { passive: false });
    }

    for (const type of WINDOW_INPUT) {
      window.addEventListener(type, this.onWindowInput);
    }
  }

  /**
   * Shows the cursor whatever is drawing asked for.
   *
   * @param cursor - A css cursor, as the controls on the far side wrote it.
   */
  public setCursor(cursor: string): void {
    this.canvas.style.cursor = cursor;
  }

  /** Stops forwarding, and leaves the canvas as it was found. */
  public dispose(): void {
    for (const type of ELEMENT_INPUT) {
      this.canvas.removeEventListener(type, this.onElementInput);
    }

    for (const type of WINDOW_INPUT) {
      window.removeEventListener(type, this.onWindowInput);
    }

    this.canvas.style.cursor = this.cursor;
    this.canvas.style.touchAction = "";
  }

  private readonly onElementInput = (event: Event): void => {
    event.preventDefault();

    this.sink(toRenderInputEvent(event.type as ERenderInput, event as PointerEvent));
  };

  private readonly onWindowInput = (event: Event): void => {
    this.sink(toRenderInputEvent(event.type as ERenderInput, event as PointerEvent));
  };
}
