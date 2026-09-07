import { ReactElement, ReactNode, useMemo } from "react";

import { IUseRankedSearchOptions, useRankedSearch } from "@/core/search/lib";
import { EditorSearchHeader } from "@/core/shell/editor/EditorSearchHeader";
import { EditorSearchResults, IEditorSearchResultRow } from "@/core/shell/editor/EditorSearchResults";
import { EditorSideMenu, IEditorSideMenuItem } from "@/core/shell/editor/EditorSideMenu";
import { StyledComponentProps } from "@/lib/dom/element-types";

export interface IEditorSearchMenuProps<T> extends StyledComponentProps, IUseRankedSearchOptions<T> {
  title: string;
  searchLabel: string;
  resultsLabel: string;
  placeholder?: string;
  /** Describes a result without discarding the item that activation opens. */
  toRow: (item: T) => IEditorSearchResultRow;
  onSelect: (item: T) => void;
  /** Filters or controls beneath the search field. */
  header?: ReactNode;
  /** Default list or content, displayed while the query is empty. */
  sections?: Array<IEditorSideMenuItem>;
  children?: ReactNode;
}

/** Combines a searchable menu's field, result navigation, and default content. */
export function EditorSearchMenu<T>({
  "data-testid": dataTestId,
  id,
  className,
  sx,
  title,
  searchLabel,
  resultsLabel,
  placeholder = searchLabel,
  items,
  toSearchText,
  toSecondaryText,
  limit,
  toRow,
  onSelect,
  header,
  sections,
  children,
}: IEditorSearchMenuProps<T>): ReactElement {
  const search = useRankedSearch({ items, toSearchText, toSecondaryText, limit, onSelect });
  const rows = useMemo(() => search.results.map(({ item }) => ({ ...toRow(item), item })), [search.results, toRow]);

  return (
    <EditorSideMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={sx}
      header={
        <>
          <EditorSearchHeader
            title={title}
            count={items.length}
            query={search.query}
            placeholder={placeholder}
            ariaLabel={searchLabel}
            onClear={search.clear}
            onKeyDown={search.isSearching ? search.onInputKeyDown : undefined}
            onQueryChange={search.setQuery}
          />
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
          onHoverIndex={search.setActiveIndex}
          onSelect={(row) => onSelect(row.item)}
        />
      ) : (
        children
      )}
    </EditorSideMenu>
  );
}
