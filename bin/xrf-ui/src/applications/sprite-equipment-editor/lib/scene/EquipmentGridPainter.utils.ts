import { IPanZoomRect, IPanZoomSize } from "@/lib/media/pan-zoom";
import { Nullable } from "@/lib/types/general";

/**
 * Sizes a canvas to its pane and hands back a context ready to draw in css pixels.
 *
 * @param canvas - Canvas to prepare, or null before it is mounted.
 * @param viewport - Pane size in css pixels.
 * @returns The cleared context, or null when there is nowhere to draw yet.
 */
export function prepareCanvas(
  canvas: Nullable<HTMLCanvasElement>,
  viewport: IPanZoomSize
): Nullable<CanvasRenderingContext2D> {
  if (!canvas || viewport.width <= 0 || viewport.height <= 0) {
    return null;
  }

  const ratio: number = window.devicePixelRatio || 1;
  const backingWidth: number = Math.round(viewport.width * ratio);
  const backingHeight: number = Math.round(viewport.height * ratio);

  // Guarded because assigning either dimension clears the canvas even when the value is unchanged, which would throw
  // away a frame on every redraw that did not resize.
  if (canvas.width !== backingWidth || canvas.height !== backingHeight) {
    canvas.width = backingWidth;
    canvas.height = backingHeight;
  }

  const context: Nullable<CanvasRenderingContext2D> = canvas.getContext("2d");

  if (!context) {
    return null;
  }

  context.setTransform(ratio, 0, 0, ratio, 0, 0);
  context.clearRect(0, 0, viewport.width, viewport.height);

  return context;
}

/**
 * Shades a rectangle.
 *
 * @param context - Canvas to draw into.
 * @param rect - Rectangle in viewport pixels.
 * @param colour - Fill to shade it with.
 */
export function fillRect(context: CanvasRenderingContext2D, rect: IPanZoomRect, colour: string): void {
  context.fillStyle = colour;
  context.fillRect(rect.x, rect.y, rect.width, rect.height);
}

/**
 * Outlines a rectangle, inset so the stroke sits inside the bounds it describes.
 *
 * @param context - Canvas to draw into.
 * @param rect - Rectangle in viewport pixels.
 * @param colour - Stroke to outline it with.
 * @param lineWidth - Stroke width in viewport pixels.
 */
export function strokeRect(
  context: CanvasRenderingContext2D,
  rect: IPanZoomRect,
  colour: string,
  lineWidth: number
): void {
  context.strokeStyle = colour;
  context.lineWidth = lineWidth;
  context.strokeRect(toHairline(rect.x), toHairline(rect.y), rect.width - 1, rect.height - 1);
}

/**
 * Puts a one pixel line on a pixel instead of across two of them at half intensity.
 *
 * @param position - Where the line falls, in css pixels.
 * @returns The nearest position a hairline is crisp at.
 */
export function toHairline(position: number): number {
  return Math.round(position) + 0.5;
}
