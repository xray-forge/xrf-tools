/**
 * Whether this browser will hand a canvas's drawing to another thread.
 *
 * @returns Whether `transferControlToOffscreen` exists to be called.
 */
export function canRenderOffscreen(): boolean {
  return typeof HTMLCanvasElement !== "undefined" && "transferControlToOffscreen" in HTMLCanvasElement.prototype;
}
