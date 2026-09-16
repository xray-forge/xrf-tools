import {
  ArchiveBounds,
  ArchiveDescribeScope,
  ArchiveReference,
  EArchiveDescribeScope,
  EArchiveReferenceStatus,
} from "@/core/ipc/types/xrf-app";
import { formatNumber } from "@/lib/format/number";
import { assertExhaustive } from "@/lib/types/exhaustive";
import { Nullable } from "@/lib/types/general";

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
  return `0x${(value >>> 0).toString(16).padStart(8, "0").toUpperCase()}`;
}

/**
 * A chunk id in the spelling `ETextureParams.h` gives it.
 *
 * @param id - Chunk id as read.
 * @returns The id as four hex digits.
 */
export function formatChunkId(id: number): string {
  return `0x${(id >>> 0).toString(16).padStart(4, "0").toUpperCase()}`;
}

/**
 * The entries of a described collection whose name carries what was typed.
 *
 * A plain substring fold rather than a matcher: a library or a bank is browsed by remembering part of a name, and
 * every one of these lists is filtered the same way.
 *
 * @param items - Entries in the order the description carries them.
 * @param filter - What the filter field holds, already the user's whole query.
 * @param getName - Names one entry.
 * @returns The matching entries, or every entry for an empty query.
 */
export function filterByName<T>(items: Array<T>, filter: string, getName: (item: T) => string): Array<T> {
  const needle: string = filter.trim().toLowerCase();

  return needle ? items.filter((item: T) => getName(item).toLowerCase().includes(needle)) : items;
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
