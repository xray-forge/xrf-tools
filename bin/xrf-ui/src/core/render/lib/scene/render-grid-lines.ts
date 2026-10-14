import { IPoint3, toRawColor, TRawColor } from "@xrf/math";

import { RenderOverlaySegment } from "@/core/ipc/types/xrf-renderer";
import { toRenderGridStep } from "@/core/render/lib/scene/render-grid-step";

/** What a grid is drawn with. */
export interface IRenderGridLinesOptions {
  /** Roughly how many cells the grid spans across; the step itself is rounded to a readable one. */
  cells: number;
  /** Hex colour of the lines. */
  color: number;
  /** Hex colour of the two lines through the origin. */
  originColor: number;
  /** Where its middle stands, the origin when left out. */
  center?: IPoint3;
  /** Times each round cell is split, reaching as far with that many more lines; one when left out. */
  subdivision?: number;
}

/** Three's `AxesHelper` colours at each end, as the canvas shows them: x red, y green, z blue, fading as they go. */
const AXES_COLORS: ReadonlyArray<[TRawColor, TRawColor]> = [
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
 * @returns The grid's segments.
 */
export function toRenderGridLines(extent: number, options: IRenderGridLinesOptions): Array<RenderOverlaySegment> {
  const { color, originColor } = options;
  const { x, y, z }: IPoint3 = options.center ?? { x: 0, y: 0, z: 0 };
  const subdivision: number = Math.max(1, Math.round(options.subdivision ?? 1));
  const cells: number = options.cells * subdivision;
  const step: number = toRenderGridStep(extent * 2, options.cells) / subdivision;
  const half: number = (step * cells) / 2;
  const segments: Array<RenderOverlaySegment> = [];

  for (let line = 0; line <= cells; line += 1) {
    const at: number = -half + line * step;
    const shade: TRawColor = toRawColor(line * 2 === cells ? originColor : color);

    segments.push(
      {
        colors: [shade, shade],
        ends: [
          [x - half, y, z + at],
          [x + half, y, z + at],
        ],
      },
      {
        colors: [shade, shade],
        ends: [
          [x + at, y, z - half],
          [x + at, y, z + half],
        ],
      }
    );
  }

  return segments;
}

/**
 * The twelve edges of a box.
 *
 * @param min - Its lowest corner.
 * @param max - Its highest.
 * @param color - Hex colour of the edges.
 * @returns The box's segments.
 */
export function toRenderBoxLines(min: IPoint3, max: IPoint3, color: number): Array<RenderOverlaySegment> {
  const corners: Array<[number, number, number]> = [0, 1, 2, 3, 4, 5, 6, 7].map((bits: number) => [
    bits & 1 ? max.x : min.x,
    bits & 2 ? max.y : min.y,
    bits & 4 ? max.z : min.z,
  ]);
  // Each pair of corners differing in exactly one bit is an edge.
  const edges: Array<[number, number]> = [];

  for (let from = 0; from < 8; from += 1) {
    for (const bit of [1, 2, 4]) {
      if (!(from & bit)) {
        edges.push([from, from | bit]);
      }
    }
  }

  const shade: TRawColor = toRawColor(color);

  return edges.map(([from, to]) => ({ colors: [shade, shade], ends: [corners[from], corners[to]] }));
}

/**
 * The three axes from the origin, each `size` long.
 *
 * @param size - How long each is.
 * @returns The axes' segments.
 */
export function toRenderAxesLines(size: number): Array<RenderOverlaySegment> {
  return AXES_COLORS.map(([start, end], axis: number) => {
    const tip: [number, number, number] = [0, 0, 0];

    tip[axis] = size;

    return { colors: [start, end], ends: [[0, 0, 0], tip] };
  });
}
