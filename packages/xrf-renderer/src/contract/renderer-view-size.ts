/**
 * How big the canvas showing frames is, which only the page can measure.
 */
export interface IRendererViewSize {
  /** Css pixels. */
  width: number;
  /** Css pixels. */
  height: number;
  /** Device pixels drawn for each css pixel. */
  pixelRatio: number;
}
