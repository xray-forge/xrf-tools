import { Nullable } from "@xrf/types";

import { ERenderOverlay, RenderOverlay } from "@/core/ipc/types/xrf-renderer";
import { ILevelBox, toBoxFloor, toBoxReach, toOriginReach } from "@/core/level/lib/extent/level-extent";
import { ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { toNativeLines } from "@/core/render/lib/native/native-overlay";
import { toRawColor } from "@/core/render/lib/scene/render-color";
import { toRenderAxesLines, toRenderBoxLines, toRenderGridLines } from "@/core/render/lib/scene/render-grid-lines";
import { toRenderGridStep } from "@/core/render/lib/scene/render-grid-step";

/** Cells of the grid the axis marker spans, so which way is which is legible without dwarfing the level. */
const AXES_CELLS: number = 2;

/**
 * The grid, the extent, the axes and the sun, sized to the level and shown as the toolbar asks.
 *
 * @param box - The level's extent, or null while none is open.
 * @param options - Which of them the toolbar shows.
 * @param config - Their cells, colours and sizes.
 * @returns The overlays, in the order drawn.
 */
export function toLevelFrameOverlays(
  box: Nullable<ILevelBox>,
  options: Pick<ILevelViewOptions, "isAxesVisible" | "isGridVisible" | "isSunVisible">,
  config: ILevelRenderConfig
): Array<RenderOverlay> {
  const overlays: Array<RenderOverlay> = [];

  if (box && options.isGridVisible) {
    overlays.push(toLevelGridOverlay(box, config));

    if (!box.isEmpty) {
      overlays.push(toLevelExtentGridOverlay(box, config), toLevelExtentBoxOverlay(box, config));
    }
  }

  if (box && options.isAxesVisible) {
    overlays.push(toLevelAxesOverlay(box, config));
  }

  if (options.isSunVisible) {
    overlays.push(toLevelSunOverlay(config));
  }

  return overlays;
}

/**
 * The ground grid, centred on the origin and reaching the level: a level lying a kilometre off zero needs a grid that
 * reaches it, not one the size of it. Tested against depth, so the level's own ground hides it.
 *
 * @param box - The level's extent.
 * @param config - The grid's cells and colours.
 * @returns The overlay.
 */
export function toLevelGridOverlay(box: ILevelBox, config: ILevelRenderConfig): RenderOverlay {
  return toNativeLines(
    toRenderGridLines(toOriginReach(box), {
      cells: config.gridCells,
      color: config.gridColor,
      originColor: config.gridOriginColor,
      subdivision: config.gridSubdivision,
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
export function toLevelExtentGridOverlay(box: ILevelBox, config: ILevelRenderConfig): RenderOverlay {
  return toNativeLines(
    toRenderGridLines(toBoxReach(box), {
      cells: config.gridCells,
      center: toBoxFloor(box),
      color: config.boundsColor,
      originColor: config.boundsColor,
      subdivision: config.gridSubdivision,
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
export function toLevelExtentBoxOverlay(box: ILevelBox, config: ILevelRenderConfig): RenderOverlay {
  return toNativeLines(toRenderBoxLines(box.min, box.max, config.boundsColor), true);
}

/**
 * The axis marker at the level's own origin, two grid cells long, drawn through the level: a marker is a point, and
 * marsh puts its origin under its terrain.
 *
 * @param box - The level's extent, which sizes the grid the marker is measured in.
 * @param config - The grid's cells.
 * @returns The overlay.
 */
export function toLevelAxesOverlay(box: ILevelBox, config: ILevelRenderConfig): RenderOverlay {
  const step: number = toRenderGridStep(toOriginReach(box) * 2, config.gridCells);

  return toNativeLines(toRenderAxesLines(step * AXES_CELLS), false);
}

/**
 * The sun as a disc where the light comes from, following the camera, since a light draws nothing.
 *
 * @param config - Its colour and size.
 * @returns The overlay.
 */
export function toLevelSunOverlay(config: ILevelRenderConfig): RenderOverlay {
  return { color: toRawColor(config.sunColor), kind: ERenderOverlay.SUN, size: config.sunSize };
}
