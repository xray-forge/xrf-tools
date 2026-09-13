import {
  IEquipmentGrid,
  IEquipmentLayout,
  IEquipmentSectionDescriptor,
  TEquipmentCell,
  toCellRect,
  toDescriptorRect,
  toOutsideSheetRects,
} from "@/core/sprite-equipment/lib";
import { IPanZoomRect, IPanZoomSize, IPanZoomTransform, toViewportRect } from "@/lib/media/pan-zoom";
import { Nullable } from "@/lib/types/general";

import { IEquipmentGridPalette } from "./equipment-grid-palette";
import { fillRect, strokeRect, toHairline } from "./EquipmentGridCanvas.utils";

/** Below this many viewport pixels a cell is narrower than the lines around it, so the lattice stops being drawn. */
const MINIMUM_LEGIBLE_CELL: number = 4;

/** Everything one frame of the overlay is drawn from. */
export interface IEquipmentGridFrame {
  /** The lattice and its occupants. */
  layout: IEquipmentLayout;
  /** Where the sheet sits in the viewport. */
  transform: IPanZoomTransform;
  /** Viewport size in css pixels. */
  viewport: IPanZoomSize;
  palette: IEquipmentGridPalette;
  /** Whether the lattice lines are drawn at all. Occupancy and selection are shown either way. */
  isGridVisible: boolean;
  hoveredCell: Nullable<TEquipmentCell>;
  selectedCell: Nullable<TEquipmentCell>;
}

/**
 * Draws one frame of the grid overlay, back to front.
 *
 * @param context - Prepared context, already cleared and scaled to css pixels.
 * @param frame - What to draw.
 */
export function paintEquipmentGrid(context: CanvasRenderingContext2D, frame: IEquipmentGridFrame): void {
  paintOutsideSheet(context, frame);
  paintOccupancy(context, frame);
  paintLattice(context, frame);
  paintHoveredCell(context, frame);
  paintSelectedCell(context, frame);
}

/**
 * Shades the part of the lattice the picture cannot cover.
 *
 * @param context - Context to draw into.
 * @param frame - What to draw.
 */
function paintOutsideSheet(context: CanvasRenderingContext2D, frame: IEquipmentGridFrame): void {
  for (const rect of toOutsideSheetRects(frame.layout.grid)) {
    fillRect(context, toViewportRect(frame.transform, rect), frame.palette.outside);
  }
}

/**
 * Shades what the configuration claims, then outlines whatever claims more than the picture holds.
 *
 * Per rectangle rather than per cell: walking the lattice is thirteen thousand lookups a frame on an Anomaly sheet to
 * shade two thousand rectangles. Overlapping claims stack their alpha, which is how a shared slot shows.
 *
 * @param context - Context to draw into.
 * @param frame - What to draw.
 */
function paintOccupancy(context: CanvasRenderingContext2D, frame: IEquipmentGridFrame): void {
  const { layout, palette } = frame;

  for (const descriptor of layout.descriptors) {
    fillRect(context, toDescriptorViewportRect(frame, descriptor), palette.occupied);
  }

  for (const descriptor of layout.outside) {
    strokeRect(context, toDescriptorViewportRect(frame, descriptor), palette.outsideEdge, 1);
  }
}

/**
 * Draws the lattice lines, clipped to what the viewport can see.
 *
 * By line index rather than by walking cells, and only the indices on screen are visited: at a fitted Anomaly sheet
 * that is a few hundred lines instead of thirteen thousand cells. One path for all of them, because a stroke per line
 * is a state change per line.
 *
 * @param context - Context to draw into.
 * @param frame - What to draw.
 */
function paintLattice(context: CanvasRenderingContext2D, frame: IEquipmentGridFrame): void {
  const { isGridVisible, palette, transform, viewport } = frame;
  const grid: IEquipmentGrid = frame.layout.grid;
  const step: number = grid.size * transform.scale;

  if (!isGridVisible || step < MINIMUM_LEGIBLE_CELL) {
    return;
  }

  const firstColumn: number = Math.max(0, Math.floor(-transform.offsetX / step));
  const lastColumn: number = Math.min(grid.columns, Math.ceil((viewport.width - transform.offsetX) / step));
  const firstRow: number = Math.max(0, Math.floor(-transform.offsetY / step));
  const lastRow: number = Math.min(grid.rows, Math.ceil((viewport.height - transform.offsetY) / step));

  const top: number = transform.offsetY + firstRow * step;
  const bottom: number = transform.offsetY + lastRow * step;
  const left: number = transform.offsetX + firstColumn * step;
  const right: number = transform.offsetX + lastColumn * step;

  context.beginPath();

  for (let column = firstColumn; column <= lastColumn; column++) {
    const x: number = toHairline(transform.offsetX + column * step);

    context.moveTo(x, top);
    context.lineTo(x, bottom);
  }

  for (let row = firstRow; row <= lastRow; row++) {
    const y: number = toHairline(transform.offsetY + row * step);

    context.moveTo(left, y);
    context.lineTo(right, y);
  }

  context.strokeStyle = palette.line;
  context.lineWidth = 1;
  context.stroke();
}

/**
 * Marks the cell under the pointer.
 *
 * @param context - Context to draw into.
 * @param frame - What to draw.
 */
function paintHoveredCell(context: CanvasRenderingContext2D, frame: IEquipmentGridFrame): void {
  if (frame.hoveredCell) {
    fillRect(context, toCellViewportRect(frame, frame.hoveredCell), frame.palette.hover);
  }
}

/**
 * Marks the cell whose occupants are being shown.
 *
 * @param context - Context to draw into.
 * @param frame - What to draw.
 */
function paintSelectedCell(context: CanvasRenderingContext2D, frame: IEquipmentGridFrame): void {
  if (!frame.selectedCell) {
    return;
  }

  const rect: IPanZoomRect = toCellViewportRect(frame, frame.selectedCell);

  fillRect(context, rect, frame.palette.selected);
  strokeRect(context, rect, frame.palette.selectedEdge, 2);
}

/**
 * @param frame - Frame being drawn.
 * @param cell - Cell as `[row, column]`.
 * @returns Where that cell lands in the viewport.
 */
function toCellViewportRect(frame: IEquipmentGridFrame, cell: TEquipmentCell): IPanZoomRect {
  return toViewportRect(frame.transform, toCellRect(frame.layout.grid, cell));
}

/**
 * @param frame - Frame being drawn.
 * @param descriptor - Rectangle in cells.
 * @returns Where that rectangle lands in the viewport.
 */
function toDescriptorViewportRect(frame: IEquipmentGridFrame, descriptor: IEquipmentSectionDescriptor): IPanZoomRect {
  return toViewportRect(frame.transform, toDescriptorRect(frame.layout.grid, descriptor));
}
