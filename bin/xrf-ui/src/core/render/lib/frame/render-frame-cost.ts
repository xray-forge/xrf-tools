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
}
