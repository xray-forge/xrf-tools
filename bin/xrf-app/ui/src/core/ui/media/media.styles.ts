import { VIEWPORT } from "@/core/theme/tokens";

/**
 * The checkerboard, sized so its squares stay the same size on screen whatever the content is scaled to.
 *
 * @param scale - Content scale the board is being drawn at.
 * @returns Background sizing to write onto the scaled element.
 */
export function toCheckerboardSizing(scale: number): { backgroundSize: string; backgroundPosition: string } {
  const square: number = VIEWPORT.checkerboardSquare / scale;

  return {
    backgroundSize: `${square * 2}px ${square * 2}px`,
    backgroundPosition: `0 0, 0 ${square}px, ${square}px -${square}px, -${square}px 0px`,
  };
}
