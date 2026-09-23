import { TRendererColor } from "@xrf/renderer";

import { toRenderGridStep } from "@/core/render/lib/scene/render-grid-step";

/** Line segments as the renderer's line overlay takes them. */
export interface IRenderLines {
  /** Three floats a vertex, two vertices a segment. */
  positions: Float32Array;
  /** Three floats a vertex. */
  colors: Float32Array;
}

/** What a grid is drawn with. */
export interface IRenderGridLinesOptions {
  cells: number;
  /** Hex colour of the lines. */
  color: number;
  /** Hex colour of the two lines through the origin. */
  originColor: number;
}

/** Three's `AxesHelper` colours at each end, as the canvas shows them: x red, y green, z blue, fading as they go. */
const AXES_COLORS: ReadonlyArray<[TRendererColor, TRendererColor]> = [
  [
    [1, 0, 0],
    [1, 0.797, 0],
  ],
  [
    [0, 1, 0],
    [0.797, 1, 0],
  ],
  [
    [0, 0, 1],
    [0, 0.797, 1],
  ],
];

/**
 * A ground grid on the xz plane around the origin, as three's `GridHelper` lays one out, at a step that suits what it
 * frames.
 *
 * @param extent - How far out what it frames reaches.
 * @param options - Cells across, and the two colours.
 * @returns The grid's lines.
 */
export function toRenderGridLines(extent: number, options: IRenderGridLinesOptions): IRenderLines {
  const { cells, color, originColor } = options;
  const step: number = toRenderGridStep(extent * 2, cells);
  const half: number = (step * cells) / 2;
  const positions: Array<number> = [];
  const colors: Array<number> = [];

  for (let line = 0; line <= cells; line += 1) {
    const at: number = -half + line * step;
    const shade: TRendererColor = toRawColor(line * 2 === cells ? originColor : color);

    positions.push(-half, 0, at, half, 0, at, at, 0, -half, at, 0, half);
    colors.push(...shade, ...shade, ...shade, ...shade);
  }

  return { colors: new Float32Array(colors), positions: new Float32Array(positions) };
}

/**
 * The three axes from the origin, each `size` long.
 *
 * @param size - How long each is.
 * @returns The axes' lines.
 */
export function toRenderAxesLines(size: number): IRenderLines {
  const positions: Array<number> = [];
  const colors: Array<number> = [];

  AXES_COLORS.forEach(([start, end], axis: number) => {
    const tip: [number, number, number] = [0, 0, 0];

    tip[axis] = size;
    positions.push(0, 0, 0, ...tip);
    colors.push(...start, ...end);
  });

  return { colors: new Float32Array(colors), positions: new Float32Array(positions) };
}

/**
 * @param hex - A hex colour as a page writes one.
 * @returns Its bytes as raw values, which is what the renderer's overlays draw.
 */
export function toRawColor(hex: number): TRendererColor {
  return [((hex >> 16) & 0xff) / 255, ((hex >> 8) & 0xff) / 255, (hex & 0xff) / 255];
}
