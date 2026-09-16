import { ArchiveBounds } from "@/core/ipc/types/xrf-app";
import { formatNumber } from "@/lib/format/number";
import { Nullable } from "@/lib/types/general";

/** Decimals an extent keeps. A level is measured in tens of metres, so one is plenty and three is noise. */
const EXTENT_DIGITS: number = 1;

/** Decimals a node spacing keeps, which is a fraction of a metre and needs them. */
const SPACING_DIGITS: number = 2;

/**
 * How much world a level piece covers.
 *
 * @param bounds - Extents the piece declares.
 * @returns The three extents in engine units, which are metres.
 */
export function formatLevelBounds(bounds: ArchiveBounds): string {
  const width: string = formatNumber(bounds.width, EXTENT_DIGITS);
  const height: string = formatNumber(bounds.height, EXTENT_DIGITS);
  const depth: string = formatNumber(bounds.depth, EXTENT_DIGITS);

  return `${width} × ${height} × ${depth} m`;
}

/**
 * The grid one navigation node covers.
 *
 * @param nodeSize - Spacing between nodes on the ground plane.
 * @param nodeHeight - Height one node spans.
 * @returns A phrase for the node's own size.
 */
export function formatNodeSize(nodeSize: Nullable<number>, nodeHeight: Nullable<number>): string {
  return `${formatNumber(nodeSize, SPACING_DIGITS)} m apart, ${formatNumber(nodeHeight, SPACING_DIGITS)} m tall`;
}

/**
 * A count with thousands separated, because these run to millions and a bare run of digits is not read.
 *
 * @param value - Count to render.
 * @returns The count, grouped.
 */
export function formatCount(value: number): string {
  return value.toLocaleString("en-US");
}
