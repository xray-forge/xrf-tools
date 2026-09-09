import {
  ArchivePatchChange,
  ArchivePatchClass,
  ArchivePatchOrigin,
  ArchivePatchResult,
  ArchivePatchSide,
} from "@/core/bindings/types/xrf-pack";
import { Nullable } from "@/lib/types/general";

/** One classified entry, flattened into the columns a findings grid shows. */
export interface IPatchChangeRow {
  /**
   * Classification, kept as the raw class rather than as its label.
   */
  class: ArchivePatchClass;
  name: string;
  /**
   * Unpacked bytes this row accounts for.
   */
  size: number;
  /**
   * Volume set or loose root the size was read from.
   */
  origin: string;
}

/**
 * Lists added, modified, then removed entries.
 *
 * Origins arrive as a table listed once per report and an index on each side, so rows sharing an origin end up
 * holding the same string instance rather than each parsing its own copy of the path.
 *
 * @param result - Comparison report.
 * @returns One row per changed entry, preserving order within each class.
 */
export function toPatchChangeRows(result: ArchivePatchResult): Array<IPatchChangeRow> {
  const origins: Array<string> = result.origins.map(describeOrigin);

  return [...result.added, ...result.modified, ...result.removed].map((change: ArchivePatchChange) => {
    const side: Nullable<ArchivePatchSide> = change.target ?? change.base;

    return {
      class: change.class,
      name: change.name,
      size: side?.size ?? 0,
      origin: side ? (origins[side.origin] ?? "") : "",
    };
  });
}

/**
 * Names where entries were read from.
 *
 * @param origin - One entry of the report's origin table.
 * @returns The volume set path for an archived origin, or the mount root for a loose one.
 */
function describeOrigin(origin: ArchivePatchOrigin): string {
  return origin.kind === "archive" ? origin.path : origin.root;
}
