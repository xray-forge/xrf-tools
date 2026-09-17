/**
 * Rank buckets, best first. The numbers are only compared to each other.
 */
export enum ESearchRank {
  EXACT = 0,
  /** The file name without its extension is exactly the query - typing "dialogs" for dialogs.xml. */
  EXACT_LEAF = 1,
  /** The file name starts with the query, which is what someone typing a partial name means. */
  LEAF_PREFIX = 2,
  /** Starts a directory, or a dot-separated word inside the name. */
  SEGMENT = 3,
  SUBSTRING = 4,
  /** Matched only in secondary text such as documentation, never in the name itself. */
  SECONDARY = 5,
}

export interface ISearchIndexEntry<T> {
  item: T;
  /** Lowercased once when the index is built, never per keystroke. */
  text: string;
  /** Where the last path segment starts, so a file name match can outrank a directory match. */
  leafAt: number;
  /** Where the extension begins, so an exact stem can outrank a prefix of some longer name. */
  stemEnd: number;
  /** Also searched, but never ranked - documentation, descriptions, anything not the identity. */
  secondary: string;
}

export interface IRankedSearchOutcome<T> {
  results: Array<T>;
  /** Every match, not just the returned ones, so the caller can say how many were left out. */
  total: number;
}

/** Only these end a path segment. A dot separates words inside a name, it does not start a new leaf. */
const PATH_SEPARATORS: string = "\\/";
const SEGMENT_SEPARATORS: string = "\\/.";

/**
 * Prepare items for repeated searching.
 *
 * The whole point of the index is that `toLowerCase` and segment scanning happen once per dataset
 * rather than once per item per keystroke.
 *
 * @param items - Items to index for repeated searching.
 * @param toSearchText - Returns each item's searchable identity text.
 * @param toSecondaryText - Optionally returns searchable text that does not affect rank.
 * @returns Lowercased search entries with path metadata.
 */
export function buildSearchIndex<T>(
  items: ReadonlyArray<T>,
  toSearchText: (item: T) => string,
  toSecondaryText?: (item: T) => string
): Array<ISearchIndexEntry<T>> {
  return items.map((item: T) => {
    const text: string = toSearchText(item).toLowerCase();

    let leafAt: number = 0;

    for (let index = text.length - 1; index >= 0; index -= 1) {
      if (PATH_SEPARATORS.includes(text[index])) {
        leafAt = index + 1;
        break;
      }
    }

    const dotAt: number = text.lastIndexOf(".");
    const stemEnd: number = dotAt > leafAt ? dotAt : text.length;

    return { item, text, leafAt, stemEnd, secondary: toSecondaryText?.(item).toLowerCase() ?? "" };
  });
}

function getRank(entry: ISearchIndexEntry<unknown>, query: string, matchAt: number): ESearchRank {
  if (entry.text.length === query.length) {
    return ESearchRank.EXACT;
  }

  // A directory may match before the file name, but the file name still determines the stronger rank.
  if (entry.text.startsWith(query, entry.leafAt)) {
    return entry.stemEnd - entry.leafAt === query.length ? ESearchRank.EXACT_LEAF : ESearchRank.LEAF_PREFIX;
  }

  // Starting right after a separator reads as naming a directory or a word, which still beats landing in
  // the middle of one - but never beats naming the file itself.
  for (let at: number = matchAt; at !== -1; at = entry.text.indexOf(query, at + 1)) {
    if (at === 0 || SEGMENT_SEPARATORS.includes(entry.text[at - 1])) {
      return ESearchRank.SEGMENT;
    }
  }

  return ESearchRank.SUBSTRING;
}

/**
 * Filter and rank in a single pass, returning at most `limit` results.
 *
 * Everything is visited even though only `limit` come back: the count has to be truthful, and a result
 * cannot be known to be the best match until the rest have been looked at.
 *
 * @param index - Precomputed entries to search.
 * @param rawQuery - Search text before whitespace trimming and case normalization.
 * @param limit - Maximum number of ranked results to return.
 * @returns Ranked results and the total number of matches.
 */
export function rankedSearch<T>(
  index: ReadonlyArray<ISearchIndexEntry<T>>,
  rawQuery: string,
  limit: number
): IRankedSearchOutcome<T> {
  const query: string = rawQuery.trim().toLowerCase();

  if (!query) {
    return { results: [], total: 0 };
  }

  const matches: Array<{ entry: ISearchIndexEntry<T>; rank: ESearchRank }> = [];

  for (const entry of index) {
    const matchAt: number = entry.text.indexOf(query);

    if (matchAt !== -1) {
      matches.push({ entry, rank: getRank(entry, query, matchAt) });
    } else if (entry.secondary.includes(query)) {
      // Found in documentation rather than in the name, so it sorts below every name match instead of
      // competing with them on an offset that means nothing in the label.
      matches.push({ entry, rank: ESearchRank.SECONDARY });
    }
  }

  matches.sort((first, second) => first.rank - second.rank || first.entry.text.localeCompare(second.entry.text));

  return {
    results: matches.slice(0, limit).map((match) => match.entry.item),
    total: matches.length,
  };
}
