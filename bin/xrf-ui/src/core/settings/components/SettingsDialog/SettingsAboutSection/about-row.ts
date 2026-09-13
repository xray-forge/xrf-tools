import { Nullable } from "@/lib/types/general";

/** One stated fact: what it is, what it says, and where it can be followed to. */
export interface IAboutRow {
  label: string;
  value: string;
  href?: string;
}

/**
 * Drops what a build or a machine could not answer.
 *
 * @param rows - Label, value and optional link of every row a description considered stating.
 * @returns Only the rows that have something to say, in the order they were given.
 */
export function statedRows(rows: Array<[string, Nullable<string>, string?]>): Array<IAboutRow> {
  return rows.flatMap(([label, value, href]) => (value ? [{ label, value, href }] : []));
}
