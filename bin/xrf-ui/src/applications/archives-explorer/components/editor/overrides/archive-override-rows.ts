import { Nullable } from "@xrf/types";

import { getAssetContainerSource } from "@/core/assets/lib";
import { ArchiveShadowedCopy, ArchiveWorldEntry } from "@/core/ipc/types/xrf-app";
import { XrayAssetContainer } from "@/core/ipc/types/xrf-vfs";

/** Height every row of the listing draws at, so the virtualizer never has to measure one. */
export const ARCHIVE_OVERRIDE_ROW_HEIGHT = 28;

/** Every kind of row the listing draws, named where a switch or a comparison has to pick one. */
export enum EArchiveOverrideRow {
  /** The contested engine path, and what the contest costs. */
  PATH = "path",
  /** One copy of that path, at its place in the search order. */
  COPY = "copy",
}

/** One contested engine path, standing over the copies that claim it. */
export interface IArchiveOverridePathRow {
  kind: EArchiveOverrideRow.PATH;
  id: string;
  entry: ArchiveWorldEntry;
  /** Copies claiming this path, the winner included, which is one more than it hides. */
  copies: number;
  /** Unpacked bytes held by the copies no lookup reaches. */
  hiddenSize: number;
}

/** One copy of a contested path, ranked where the search order puts it. */
export interface IArchiveOverrideCopyRow {
  kind: EArchiveOverrideRow.COPY;
  id: string;
  container: XrayAssetContainer;
  sizeReal: number;
  /** One-based place in the search order, so the row reads as a rank rather than a position in a list. */
  rank: number;
  /** Whether this is the copy the engine loads. */
  isWinner: boolean;
}

export type TArchiveOverrideRow = IArchiveOverridePathRow | IArchiveOverrideCopyRow;

/**
 * Flattens contested paths into the one fixed-height row list the virtualizer scrolls.
 *
 * @param entries - Contested paths, winner first, as the backend folded them.
 * @returns Rows in display order.
 */
export function flattenOverrides(entries: ReadonlyArray<ArchiveWorldEntry>): Array<TArchiveOverrideRow> {
  const rows: Array<TArchiveOverrideRow> = [];

  for (const entry of entries) {
    rows.push({
      copies: entry.shadowed.length + 1,
      entry,
      hiddenSize: entry.shadowed.reduce((total: number, copy: ArchiveShadowedCopy) => total + copy.sizeReal, 0),
      id: entry.name,
      kind: EArchiveOverrideRow.PATH,
    });

    rows.push({
      container: entry.container,
      id: `${entry.name}:0`,
      isWinner: true,
      kind: EArchiveOverrideRow.COPY,
      rank: 1,
      sizeReal: entry.sizeReal,
    });

    entry.shadowed.forEach((copy: ArchiveShadowedCopy, index: number) => {
      rows.push({
        container: copy.container,
        id: `${entry.name}:${index + 1}`,
        isWinner: false,
        kind: EArchiveOverrideRow.COPY,
        rank: index + 2,
        sizeReal: copy.sizeReal,
      });
    });
  }

  return rows;
}

/**
 * Narrows contested paths to those a person is looking for.
 *
 * @param entries - Contested paths to narrow.
 * @param filter - Text to find in the engine path, already lower-cased; empty matches everything.
 * @param source - Source that must hold one of the copies, or null for any.
 * @returns The entries that match both.
 */
export function filterOverrides(
  entries: ReadonlyArray<ArchiveWorldEntry>,
  filter: string,
  source: Nullable<string>
): Array<ArchiveWorldEntry> {
  return entries.filter((entry: ArchiveWorldEntry) => {
    if (filter && !entry.name.toLowerCase().includes(filter)) {
      return false;
    }

    if (!source) {
      return true;
    }

    return (
      getAssetContainerSource(entry.container) === source ||
      entry.shadowed.some((copy: ArchiveShadowedCopy) => getAssetContainerSource(copy.container) === source)
    );
  });
}
