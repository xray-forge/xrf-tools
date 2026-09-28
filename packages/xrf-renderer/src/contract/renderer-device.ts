/**
 * The GPU a renderer is drawing with, as far as the browser tells.
 */
export interface IRendererDevice {
  /** GPU vendor, such as `nvidia`; browsers withhold the model. */
  vendor: string;
  /** GPU architecture, such as `blackwell`, or empty when the browser withholds it. */
  architecture: string;
  /** Device features the renderer was granted, sorted. */
  features: ReadonlyArray<string>;
}
