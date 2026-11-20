import { IUseRankedSearch, IUseRankedSearchOptions, useRankedSearch } from "@xrf/react/ranked-search";
import { KeyboardEvent, ReactElement, ReactNode, useCallback, useMemo } from "react";

import { EditorSearchResults, IEditorSearchResultRow } from "#/editor/search-results";
import { EditorSideMenu, IEditorSideMenuItem } from "#/editor/side-menu";
import { BaseComponentProps } from "#/element-types";

/** What a search menu's heading draws: what it lists, and the field that filters it. */
export interface IEditorSearchMenuHeading {
  /** What the menu lists. */
  title: string;
  /** Items listed before any filtering. */
  count: number;
  query: string;
  placeholder: string;
  /** Names the field for a screen reader, which the placeholder alone does not. */
  ariaLabel: string;
  onClear: () => void;
  /** Drives the result list from the field, set only while there are results to drive. */
  onKeyDown?: (event: KeyboardEvent<HTMLElement>) => void;
  onQueryChange: (query: string) => void;
}

type TEditorSearchMenuRow<T> = IEditorSearchResultRow & { item: T };

export interface IEditorSearchMenuProps<T> extends BaseComponentProps, IUseRankedSearchOptions<T> {
  title: string;
  searchLabel: string;
  resultsLabel: string;
  placeholder?: string;
  /** Blocks keyboard and pointer activation of search results while keeping filtering and navigation available. */
  isActivationDisabled?: boolean;
  /** Describes a result without discarding the item that activation opens. */
  toRow: (item: T) => IEditorSearchResultRow;
  onSelect: (item: T) => void;
  /** Draws the heading band and its filter field, which the owner places in its own panel chrome. */
  renderHeading: (heading: IEditorSearchMenuHeading) => ReactNode;
  /** Filters or controls beneath the heading. */
  header?: ReactNode;
  /** Default list or content, displayed while the query is empty. */
  sections?: Array<IEditorSideMenuItem>;
  children?: ReactNode;
}

/**
 * A searchable side menu: its heading's field ranks the items, the arrow keys and Enter drive the results, and the
 * default content returns once the query is cleared.
 */
export function EditorSearchMenu<T>({
  "data-testid": dataTestId,
  id,
  className,
  title,
  searchLabel,
  resultsLabel,
  placeholder = searchLabel,
  isActivationDisabled = false,
  items,
  toSearchText,
  toSecondaryText,
  limit,
  toRow,
  onSelect,
  renderHeading,
  header,
  sections,
  children,
}: IEditorSearchMenuProps<T>): ReactElement {
  const onSelectResult = useCallback(
    (item: T) => {
      if (!isActivationDisabled) {
        onSelect(item);
      }
    },
    [isActivationDisabled, onSelect]
  );

  const search: IUseRankedSearch<T> = useRankedSearch({
    items,
    toSearchText,
    toSecondaryText,
    limit,
    onSelect: onSelectResult,
  });
  const rows: Array<TEditorSearchMenuRow<T>> = useMemo(
    () => search.results.map((item: T) => ({ ...toRow(item), item })),
    [search.results, toRow]
  );

  const onSelectRow = useCallback((row: TEditorSearchMenuRow<T>) => onSelectResult(row.item), [onSelectResult]);

  return (
    <EditorSideMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      header={
        <>
          {renderHeading({
            title,
            count: items.length,
            query: search.query,
            placeholder,
            ariaLabel: searchLabel,
            onClear: search.clear,
            onKeyDown: search.isSearching ? search.onInputKeyDown : undefined,
            onQueryChange: search.setQuery,
          })}
          {header}
        </>
      }
      sections={search.isSearching ? undefined : sections}
    >
      {search.isSearching ? (
        <EditorSearchResults
          ariaLabel={resultsLabel}
          emptyLabel={`No ${title.toLowerCase()} match ${search.query.trim()}.`}
          rows={rows}
          total={search.total}
          activeIndex={search.activeIndex}
          isStale={search.isStale}
          isDisabled={isActivationDisabled}
          onHoverIndex={search.setActiveIndex}
          onSelect={onSelectRow}
        />
      ) : (
        children
      )}
    </EditorSideMenu>
  );
}
