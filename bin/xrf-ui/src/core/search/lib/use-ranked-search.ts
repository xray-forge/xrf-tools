import { KeyboardEvent, useCallback, useDeferredValue, useMemo, useState } from "react";

import {
  buildSearchIndex,
  IRankedSearchOutcome,
  ISearchIndexEntry,
  ISearchResult,
  rankedSearch,
} from "@/core/search/lib/ranked-search";

/** Past a couple of screens nobody reads results, and the renderer pays for every one of them. */
export const DEFAULT_SEARCH_LIMIT: number = 200;

export interface IUseRankedSearchOptions<T> {
  items: ReadonlyArray<T>;
  /** The identity of an item - what ranking is computed from. Keep it cheap rather than clever. */
  toSearchText: (item: T) => string;
  /** Optional extra text that should match but never outrank a name match. */
  toSecondaryText?: (item: T) => string;
  limit?: number;
  /** Invoked when the active result is accepted with the enter key. */
  onSelect?: (item: T) => void;
}

export interface IUseRankedSearch<T> {
  query: string;
  setQuery: (query: string) => void;
  clear: () => void;
  results: Array<ISearchResult<T>>;
  /** Matches found, which is not the same as results returned once the limit applies. */
  total: number;
  isSearching: boolean;
  /** True while the visible results belong to an older query than the one in the field. */
  isStale: boolean;
  activeIndex: number;
  setActiveIndex: (index: number) => void;
  onInputKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
}

/**
 * Searches a large list without blocking the input field.
 *
 * Three things make this fast, and all three matter: the searchable text is lowercased once per dataset
 * rather than once per keystroke, the query is deferred so react can abandon a filter that a newer
 * keystroke has already invalidated, and only `limit` results reach the renderer. The last is the one
 * that counts - filtering tens of thousands of rows costs a couple of milliseconds, while rendering
 * them costs seconds.
 *
 * @param options - Search inputs and behavior.
 * @param options.items - Items to index and search.
 * @param options.toSearchText - Extracts the primary searchable text.
 * @param options.toSecondaryText - Extracts optional lower-priority searchable text.
 * @param options.limit - Maximum number of results exposed to the renderer.
 * @param options.onSelect - Receives the active result when Enter is pressed.
 * @returns The query state, ranked results, and keyboard interaction callbacks.
 */
export function useRankedSearch<T>({
  items,
  toSearchText,
  toSecondaryText,
  limit = DEFAULT_SEARCH_LIMIT,
  onSelect,
}: IUseRankedSearchOptions<T>): IUseRankedSearch<T> {
  const [query, setQueryValue] = useState<string>("");
  const [activeIndex, setActiveIndex] = useState<number>(0);

  // Typing updates the field immediately; the filter runs against this at a lower priority.
  const deferredQuery: string = useDeferredValue(query);

  const index: Array<ISearchIndexEntry<T>> = useMemo(
    () => buildSearchIndex(items, toSearchText, toSecondaryText),
    // `toSearchText` is a stable accessor at every call site; re-indexing on identity would defeat it.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [items]
  );

  const outcome: IRankedSearchOutcome<T> = useMemo(
    () => rankedSearch(index, deferredQuery, limit),
    [index, deferredQuery, limit]
  );

  const currentIndex: number = Math.min(activeIndex, Math.max(0, outcome.results.length - 1));

  const setQuery = useCallback((next: string) => {
    setQueryValue(next);
    setActiveIndex(0);
  }, []);

  const clear = useCallback(() => {
    setQueryValue("");
    setActiveIndex(0);
  }, []);

  const onInputKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>) => {
      const { results } = outcome;

      if (!results.length) {
        return;
      }

      if (event.key === "ArrowDown") {
        event.preventDefault();
        setActiveIndex((current: number) => (Math.min(current, results.length - 1) + 1) % results.length);
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        setActiveIndex(
          (current: number) => (Math.min(current, results.length - 1) - 1 + results.length) % results.length
        );
      } else if (event.key === "Enter") {
        event.preventDefault();
        onSelect?.(results[currentIndex].item);
      }
    },
    [currentIndex, onSelect, outcome]
  );

  return {
    query,
    setQuery,
    clear,
    results: outcome.results,
    total: outcome.total,
    isSearching: Boolean(query.trim()),
    isStale: query !== deferredQuery,
    activeIndex: currentIndex,
    setActiveIndex,
    onInputKeyDown,
  };
}
