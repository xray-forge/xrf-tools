import { ERendererOverlay, TRendererOverlay } from "@xrf/renderer";

import { ILevelBox, toBoxFloor, toBoxReach, toOriginReach } from "@/core/level/lib/extent/level-extent";
import { ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import {
  IRenderLines,
  toRawColor,
  toRenderAxesLines,
  toRenderBoxLines,
  toRenderGridLines,
} from "@/core/render/lib/scene/render-grid-lines";
import { toRenderGridStep } from "@/core/render/lib/scene/render-grid-step";

/** Cells of the grid the axis marker spans, so which way is which is legible without dwarfing the level. */
const AXES_CELLS: number = 2;

/**
 * The ground grid, centred on the origin and reaching the level: a level lying a kilometre off zero needs a grid that
 * reaches it, not one the size of it. Tested against depth, so the level's own ground hides it.
 *
 * @param box - The level's extent.
 * @param config - The grid's cells and colours.
 * @returns The overlay.
 */
export function toLevelGridOverlay(box: ILevelBox, config: ILevelRenderConfig): TRendererOverlay {
  return toLines(
    toRenderGridLines(toOriginReach(box), {
      cells: config.gridCells,
      color: config.gridColor,
      originColor: config.gridOriginColor,
    }),
    true
  );
}

/**
 * The same grid over the level's own footprint in the extent's colour, so where the level ends is a line.
 *
 * @param box - The level's extent.
 * @param config - The grid's cells and the extent's colour.
 * @returns The overlay.
 */
export function toLevelExtentGridOverlay(box: ILevelBox, config: ILevelRenderConfig): TRendererOverlay {
  return toLines(
    toRenderGridLines(toBoxReach(box), {
      cells: config.gridCells,
      center: toBoxFloor(box),
      color: config.boundsColor,
      originColor: config.boundsColor,
    }),
    true
  );
}

/**
 * The box the level claims. Tested against depth: it is around everything drawn, so drawn through the level it lays
 * four bright lines across every view of it.
 *
 * @param box - The level's extent.
 * @param config - The extent's colour.
 * @returns The overlay.
 */
export function toLevelExtentBoxOverlay(box: ILevelBox, config: ILevelRenderConfig): TRendererOverlay {
  return toLines(toRenderBoxLines(box.min, box.max, config.boundsColor), true);
}

/**
 * The axis marker at the level's own origin, two grid cells long, drawn through the level: a marker is a point, and
 * marsh puts its origin under its terrain.
 *
 * @param box - The level's extent, which sizes the grid the marker is measured in.
 * @param config - The grid's cells.
 * @returns The overlay.
 */
export function toLevelAxesOverlay(box: ILevelBox, config: ILevelRenderConfig): TRendererOverlay {
  const step: number = toRenderGridStep(toOriginReach(box) * 2, config.gridCells);

  return toLines(toRenderAxesLines(step * AXES_CELLS), false);
}

/**
 * The sun as a disc where the light comes from, following the camera, since a light draws nothing.
 *
 * @param config - Its colour and size.
 * @returns The overlay.
 */
export function toLevelSunOverlay(config: ILevelRenderConfig): TRendererOverlay {
  return { color: toRawColor(config.sunColor), kind: ERendererOverlay.SUN, size: config.sunSize };
}

function toLines({ positions, colors }: IRenderLines, isDepthTested: boolean): TRendererOverlay {
  return { colors, isDepthTested, kind: ERendererOverlay.LINES, positions };
}
