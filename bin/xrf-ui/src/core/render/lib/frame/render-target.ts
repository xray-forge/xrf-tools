/**
 * Where a viewport draws, and how big it is.
 */
export interface IRenderTarget {
  /** The surface the renderer draws on. */
  readonly canvas: HTMLCanvasElement | OffscreenCanvas;
  /** Device pixels per css pixel. */
  readonly pixelRatio: number;
  /**
   * Whether the canvas has a style to keep in step with its size. A canvas on a page is laid out by css and has
   * one; a canvas handed to another thread is laid out by nobody and has none, and writing to it there throws.
   */
  readonly isStyled: boolean;
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
