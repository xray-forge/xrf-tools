/**
 * The page canvas a viewport shows frames on, and how big it is.
 */
export interface IRenderTarget {
  /** Laid out by the page and handed to the renderer's thread, once, for its drawing. */
  readonly canvas: HTMLCanvasElement;
  /** Device pixels per css pixel. */
  readonly pixelRatio: number;
  /** Width in css pixels, or zero while there is nothing to measure. */
  readonly width: number;
  /** Height in css pixels, or zero while there is nothing to measure. */
  readonly height: number;
  /**
   * @param onResized - Told whenever the size changes, without being told what to.
   * @returns Stops the telling.
   */
  observe(onResized: () => void): () => void;
  /** Releases whatever the target holds, which for a page is the canvas it put there. */
  dispose(): void;
}
