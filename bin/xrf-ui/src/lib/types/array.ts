/**
 * The one empty list, shared by everything that needs a stable "nothing here".
 *
 * Typed `never` so it satisfies a `ReadonlyArray<T>` of any element type, and frozen so a caller that tries to push
 * to it fails loudly rather than quietly filling in everyone else's default.
 */
export const EMPTY_ARRAY: ReadonlyArray<never> = Object.freeze([]);

/**
 * The list without the entry at `index`.
 *
 * @param items - List to copy.
 * @param index - Position to drop; a position outside the list copies it unchanged.
 * @returns A new list.
 */
export function withoutAt<T>(items: Array<T>, index: number): Array<T> {
  return items.filter((_, at) => at !== index);
}

/**
 * The list with the entry at `index` replaced.
 *
 * @param items - List to copy.
 * @param index - Position to replace; a position outside the list copies it unchanged.
 * @param value - Replacement value.
 * @returns A new list.
 */
export function withValueAt(items: Array<string>, index: number, value: string): Array<string> {
  return items.map((item, at) => (at === index ? value : item));
}
