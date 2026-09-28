/**
 * How large the frame is: the output shown, and the scene as drawn, smaller while it is upscaled.
 */
export interface IRendererFrameSize {
  /** The output's width, in device pixels. */
  readonly width: number;
  readonly height: number;
  /** The scene's width as drawn. */
  readonly renderWidth: number;
  readonly renderHeight: number;
  /** The output's side over the drawing's. */
  readonly upscale: number;
}

/**
 * @param width - The output's width, in device pixels.
 * @param height - And its height.
 * @param upscale - The output's side over the drawing's.
 * @returns The frame's size, each side of the drawing at least a pixel.
 */
export function toRendererFrameSize(width: number, height: number, upscale: number): IRendererFrameSize {
  return {
    height,
    renderHeight: Math.max(1, Math.round(height / upscale)),
    renderWidth: Math.max(1, Math.round(width / upscale)),
    upscale,
    width,
  };
}

/**
 * @param a - A frame's size.
 * @param b - Another.
 * @returns Whether the two draw and show at the same sizes.
 */
export function isSameRendererFrameSize(a: IRendererFrameSize, b: IRendererFrameSize): boolean {
  return (
    a.width === b.width && a.height === b.height && a.renderWidth === b.renderWidth && a.renderHeight === b.renderHeight
  );
}
