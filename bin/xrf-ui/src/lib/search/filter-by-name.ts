/**
 * The entries of a collection whose name carries what was typed.
 *
 * A plain substring fold rather than a matcher: a library or a bank is browsed by remembering part of a name, and
 * every one of these lists is filtered the same way.
 *
 * @param items - Entries in the order the collection carries them.
 * @param filter - What the filter field holds, already the user's whole query.
 * @param getName - Names one entry.
 * @returns The matching entries, or every entry for an empty query, as the same list.
 */
export function filterByName<T>(items: Array<T>, filter: string, getName: (item: T) => string): Array<T>;
export function filterByName<T>(
  items: ReadonlyArray<T>,
  filter: string,
  getName: (item: T) => string
): ReadonlyArray<T>;
export function filterByName<T>(
  items: ReadonlyArray<T>,
  filter: string,
  getName: (item: T) => string
): ReadonlyArray<T> {
  const needle: string = filter.trim().toLowerCase();

  return needle ? items.filter((item: T) => getName(item).toLowerCase().includes(needle)) : items;
}
