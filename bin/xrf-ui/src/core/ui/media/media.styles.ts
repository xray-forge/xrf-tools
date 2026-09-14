import { Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";

/** Side of one checkerboard square, in css pixels. */
const CHECKERBOARD_SQUARE: number = 10;

/**
 * Alpha checkerboard, so a transparent region reads as transparent rather than as black.
 */
export const IMAGE_CHECKERBOARD: SystemStyleObject<Theme> = {
  backgroundImage: [
    "linear-gradient(45deg, #707070 25%, transparent 25%)",
    "linear-gradient(-45deg, #808080 25%, transparent 25%)",
    "linear-gradient(45deg, transparent 75%, #808080 75%)",
    "linear-gradient(-45deg, transparent 75%, #808080 75%)",
  ].join(","),
  backgroundSize: `${CHECKERBOARD_SQUARE * 2}px ${CHECKERBOARD_SQUARE * 2}px`,
  backgroundPosition: `0 0, 0 ${CHECKERBOARD_SQUARE}px, ${CHECKERBOARD_SQUARE}px -${CHECKERBOARD_SQUARE}px, -${CHECKERBOARD_SQUARE}px 0px`,
};

/**
 * The checkerboard, sized so its squares stay the same size on screen whatever the content is scaled to.
 *
 * @param scale - Content scale the board is being drawn at.
 * @returns Background sizing to write onto the scaled element.
 */
export function toCheckerboardSizing(scale: number): { backgroundSize: string; backgroundPosition: string } {
  const square: number = CHECKERBOARD_SQUARE / scale;

  return {
    backgroundSize: `${square * 2}px ${square * 2}px`,
    backgroundPosition: `0 0, 0 ${square}px, ${square}px -${square}px, -${square}px 0px`,
  };
}
