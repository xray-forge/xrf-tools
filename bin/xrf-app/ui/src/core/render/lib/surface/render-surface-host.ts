/**
 * Anything a view can hand a drawing surface to.
 */
export interface IRenderSurfaceHost {
  /**
   * Takes somewhere to draw, and starts.
   *
   * @param container - The element the drawing fills.
   */
  attach(container: HTMLElement): void;
  /** Releases whatever was drawing, and stops. */
  detach(): void;
}
