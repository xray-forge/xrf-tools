import { Nullable } from "@xrf/types";

import { LtxFileStructure, LtxResolvedIndexEntry, LtxStructureEntry } from "@/core/ipc/types/xrf-ltx-inspect";
import { ROOT_SECTION, ROOT_SECTION_LABEL } from "@/core/ltx/lib/resolved";
import { ITreeNode } from "@/core/ui/tree/tree-node";

/**
 * One row of the index, and where clicking it goes.
 */
export type TConfigsIndexRow = { kind: "section"; name: string } | { kind: "entry"; name: string; line: number };

/**
 * Whether the open config is a list: names declared outside any section, and no section at all.
 *
 * @param structure - The open config as the parser read it.
 * @returns Whether it declares keys and no section.
 */
export function isListConfig(structure: Nullable<LtxFileStructure>): boolean {
  return Boolean(structure && structure.sections.length === 0 && structure.rootEntries.length > 0);
}

/**
 * What the panel lists for the file as written: the headers it declares, or the keys a list declares instead of them.
 *
 * @param structure - The open config as the parser read it.
 * @returns One row per section, or one per entry for a list config.
 */
export function toAuthoredIndex(structure: Nullable<LtxFileStructure>): Array<ITreeNode<TConfigsIndexRow>> {
  if (!structure) {
    return [];
  }

  if (isListConfig(structure)) {
    return structure.rootEntries.map((entry: LtxStructureEntry) => ({
      id: `entry:${entry.line}`,
      label: entry.name,
      payload: { kind: "entry", line: entry.line, name: entry.name },
    }));
  }

  return structure.sections.map((section) => ({
    id: `section:${section.name}`,
    label: section.name,
    payload: { kind: "section", name: section.name },
  }));
}

/**
 * What the panel lists for the resolved document: one row per section it holds.
 *
 * @param sections - The sections the view is showing, already narrowed by whoever narrowed them.
 * @returns One row per section, the unnamed one named.
 */
export function toResolvedIndex(sections: ReadonlyArray<LtxResolvedIndexEntry>): Array<ITreeNode<TConfigsIndexRow>> {
  return sections.map((section: LtxResolvedIndexEntry) => ({
    id: `section:${section.name}`,
    label: section.name === ROOT_SECTION ? ROOT_SECTION_LABEL : section.name,
    payload: { kind: "section", name: section.name },
  }));
}

/**
 * The rows whose name holds the query, in the order they were given.
 *
 * @param rows - Rows to narrow.
 * @param query - What was typed in the filter field.
 * @returns The rows that match.
 */
export function filterConfigsIndex(
  rows: ReadonlyArray<ITreeNode<TConfigsIndexRow>>,
  query: string
): Array<ITreeNode<TConfigsIndexRow>> {
  const wanted: string = query.trim().toLowerCase();

  return wanted
    ? rows.filter((row: ITreeNode<TConfigsIndexRow>) => row.payload?.name.toLowerCase().includes(wanted))
    : [...rows];
}
