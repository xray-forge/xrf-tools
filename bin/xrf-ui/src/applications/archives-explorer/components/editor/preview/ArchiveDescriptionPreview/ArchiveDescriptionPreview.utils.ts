import { assertExhaustive, Nullable } from "@xrf/types";

import {
  ArchiveBounds,
  ArchiveDescribeScope,
  ArchiveReference,
  EArchiveDescribeScope,
  EArchiveReferenceStatus,
} from "@/core/ipc/types/xrf-app";
import { formatHex } from "@/lib/format/hex";
import { formatNumber } from "@/lib/format/number";

/** Shown where a file declares no value at all, which is not the same as declaring a default one. */
export const NOT_DECLARED: string = "Not declared";

/** Shown for a piece of a level holding nothing whose extent could be taken, rather than a box of no size. */
export const NOTHING_TO_MEASURE: string = "Nothing to measure";

/**
 * What became of a reference, worded in the terms of the subject that was searched.
 *
 * @param reference - Resolved reference to describe.
 * @param scope - What the lookup behind it searched.
 * @returns A phrase for the reference's caption, or null when the reference resolved and needs no qualifying.
 */
export function describeReferenceStatus(reference: ArchiveReference, scope: ArchiveDescribeScope): Nullable<string> {
  switch (reference.status) {
    case EArchiveReferenceStatus.PRESENT:
      return reference.path ?? null;
    case EArchiveReferenceStatus.ABSENT:
      return scope.kind === EArchiveDescribeScope.WORLD
        ? "Not found in the mounted tree"
        : `Not in ${scope.volumes === 1 ? "this volume" : `these ${scope.volumes} volumes`}`;
    case EArchiveReferenceStatus.UNKNOWN:
      return "Not a name that can be looked up";
    default:
      return assertExhaustive(reference.status);
  }
}

/**
 * A texture param colour as the word it is stored as.
 *
 * @param value - Colour word from the descriptor.
 * @returns The word in the spelling an author would compare against.
 */
export function formatColorWord(value: number): string {
  return formatHex(value, 8);
}

/**
 * A chunk id in the spelling `ETextureParams.h` gives it.
 *
 * @param id - Chunk id as read.
 * @returns The id as four hex digits.
 */
export function formatChunkId(id: number): string {
  return formatHex(id, 4);
}

/**
 * How much world a piece of a level covers.
 *
 * @param bounds - Extents the piece declares, absent for a piece carrying nothing to measure.
 * @returns The three extents in engine units, which are metres, or the phrase for a piece with no extent at all.
 */
export function formatLevelBounds(bounds: Nullable<ArchiveBounds>): string {
  if (!bounds) {
    return NOTHING_TO_MEASURE;
  }

  const width: string = formatNumber(bounds.width, 1);
  const height: string = formatNumber(bounds.height, 1);
  const depth: string = formatNumber(bounds.depth, 1);

  return `${width} × ${height} × ${depth} m`;
}
