import { IPanZoomState, PAN_ZOOM_FIT } from "./pan-zoom";

/** What a viewport does when the camera it is looking through moves. */
export type TPanZoomListener = () => void;

/**
 * The camera one or more viewports look through, held outside React.
 *
 * Shared rather than owned by a viewport because a comparison is two pictures of the same texels: they can only move
 * together if neither of them owns the camera.
 */
export class PanZoomController {
  private state: IPanZoomState = PAN_ZOOM_FIT;

  private readonly listeners: Set<TPanZoomListener> = new Set();

  /**
   * @returns The camera every viewport on this controller is currently looking through.
   */
  public get(): IPanZoomState {
    return this.state;
  }

  /**
   * Moves the camera and tells every viewport watching it.
   *
   * @param next - The camera to look through, or how to derive it from the current one.
   */
  public set(next: IPanZoomState | ((current: IPanZoomState) => IPanZoomState)): void {
    const resolved: IPanZoomState = typeof next === "function" ? next(this.state) : next;

    if (resolved === this.state) {
      return;
    }

    this.state = resolved;

    for (const listener of this.listeners) {
      listener();
    }
  }

  /**
   * Watches the camera.
   *
   * @param listener - Called after every move, with nothing: a viewport reads the camera back itself.
   * @returns The function that stops watching.
   */
  public subscribe(listener: TPanZoomListener): () => void {
    this.listeners.add(listener);

    return () => {
      this.listeners.delete(listener);
    };
  }
}
