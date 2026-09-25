/**
 * What one frame cost, as the renderer itself counted it.
 */
export interface IRenderFrameCost {
  /** Mean frame time over the window, in milliseconds. */
  frameTime: number;
  /** The longest frame of the window, which is what a stutter actually is. */
  worstFrameTime: number;
  /** Mean of what `render` itself took, which is where uploads and shader compiles land. */
  drawTime: number;
  /** The longest draw of the window: a spike here is inside `render`, and a spike only in the frame is not. */
  worstDrawTime: number;
  framesPerSecond: number;
  /** Draw calls the last frame issued. */
  draws: number;
  /** Triangles the last frame drew, instanced geometry counted once for every place it stood. */
  triangles: number;
  /** Width of the buffer drawn into, in device pixels rather than the css pixels the canvas occupies. */
  drawnWidth: number;
  /** Height of the same buffer, which with the width is what every per-pixel cost is paid over. */
  drawnHeight: number;
  /** Width the scene was drawn at: the buffer's, or less while TAA upscales it. */
  renderedWidth: number;
  /** And its height: what the scene's per-pixel passes are paid over. */
  renderedHeight: number;
}

/** Nothing drawn yet, which is what a viewport costs before its first frame. */
export const EMPTY_RENDER_FRAME_COST: IRenderFrameCost = {
  drawnHeight: 0,
  drawnWidth: 0,
  drawTime: 0,
  draws: 0,
  frameTime: 0,
  framesPerSecond: 0,
  renderedHeight: 0,
  renderedWidth: 0,
  triangles: 0,
  worstDrawTime: 0,
  worstFrameTime: 0,
};
