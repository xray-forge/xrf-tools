import { ArchiveDetailEntry, ArchiveDetailLibraryDescription } from "@/core/ipc/types/xrf-app";
import { formatNumber } from "@/lib/format/number";

import { formatCount } from "../ArchiveDescriptionPreview.utils";

/** Metres one slot of the grid covers, `DETAIL_SLOT_SIZE`, which is fixed by the format. */
export const DETAIL_SLOT_METERS: number = 2;

/**
 * How much of the level the layer actually dresses.
 *
 * @param description - Layer to describe.
 * @returns The planted count with the share it is of the grid.
 */
export function describePlanting(description: ArchiveDetailLibraryDescription): string {
  if (!description.slots) {
    return "No slots at all";
  }

  const share: string = formatNumber((description.plantedSlots / description.slots) * 100, 1);

  return `${formatCount(description.plantedSlots)} slots · ${share}%`;
}

/**
 * How much ground the grid spans.
 *
 * @param description - Layer to describe.
 * @returns The two extents in engine units, which are metres.
 */
export function describeCoverage(description: ArchiveDetailLibraryDescription): string {
  const x: string = formatNumber(description.coversX, 1);
  const z: string = formatNumber(description.coversZ, 1);

  return `Covering ${x} × ${z} m of ground`;
}

/**
 * How widely one object of the library is planted.
 *
 * Corners rather than slots: a slot plants up to four objects, so the counts across a library sum to more than the
 * number of planted slots.
 *
 * @param entry - Entry to describe.
 * @returns The corner count, or a phrase for an object nothing plants.
 */
export function describeEntryPlanting(entry: ArchiveDetailEntry): string {
  return entry.plantedCorners
    ? `${formatCount(entry.plantedCorners)} ${entry.plantedCorners === 1 ? "corner" : "corners"}`
    : "Never planted";
}
