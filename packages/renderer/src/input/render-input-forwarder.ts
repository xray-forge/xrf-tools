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

/** Keys and focus, heard by the canvas once a press has focused it. */
const FOCUS_INPUT: ReadonlyArray<ERenderInput> = [ERenderInput.KEY_DOWN, ERenderInput.KEY_UP, ERenderInput.BLUR];

/** Keys whose own action scrolls the page, which a focused canvas refuses; every other key keeps its own. */
const SCROLLING_KEYS: ReadonlySet<string> = new Set([
  "ArrowDown",
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "PageDown",
  "PageUp",
  "Space",
]);

/**
 * Sends what is done to a canvas to whatever is drawing on it.
 */
export class RenderInputForwarder {
  private readonly canvas: HTMLCanvasElement;
  private readonly sink: TRenderInputSink;
  private readonly cursor: string;
  private readonly tabIndex: number;

  public constructor(canvas: HTMLCanvasElement, sink: TRenderInputSink) {
    this.canvas = canvas;
    this.cursor = canvas.style.cursor;
    this.tabIndex = canvas.tabIndex;
    this.sink = sink;

    // Scrolling and the browser's own menu belong to the gesture, and only this side can refuse them: a frame
    // spent asking the other thread whether to is a frame the page has already scrolled.
    canvas.style.touchAction = "none";
    // Focusable, so the keys a camera flies by reach it; a press focuses it, and the ring would frame the scene.
    canvas.tabIndex = 0;
    canvas.style.outline = "none";

    for (const type of ELEMENT_INPUT) {
      canvas.addEventListener(type, this.onElementInput, { passive: false });
    }

    for (const type of FOCUS_INPUT) {
      canvas.addEventListener(type, this.onFocusInput);
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

    for (const type of FOCUS_INPUT) {
      this.canvas.removeEventListener(type, this.onFocusInput);
    }

    this.canvas.style.cursor = this.cursor;
    this.canvas.style.touchAction = "";
    this.canvas.style.outline = "";
    this.canvas.tabIndex = this.tabIndex;
  }

  private readonly onElementInput = (event: Event): void => {
    event.preventDefault();

    // Prevented, a press no longer focuses on its own.
    if (event.type === ERenderInput.POINTER_DOWN) {
      this.canvas.focus({ preventScroll: true });
    }

    this.sink(toRenderInputEvent(event.type as ERenderInput, event));
  };

  private readonly onFocusInput = (event: Event): void => {
    if (event.type === ERenderInput.KEY_DOWN && SCROLLING_KEYS.has((event as KeyboardEvent).code)) {
      event.preventDefault();
    }

    this.sink(toRenderInputEvent(event.type as ERenderInput, event));
  };

  private readonly onWindowInput = (event: Event): void => {
    this.sink(toRenderInputEvent(event.type as ERenderInput, event));
  };
}
