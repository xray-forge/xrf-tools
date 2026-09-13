import { useCallback, useMemo, useState } from "react";

import { EApplicationGroupId, IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { ISearchResult, IUseRankedSearch, useRankedSearch } from "@/core/search/lib";
import { Nullable } from "@/lib/types/general";

import {
  getCatalogEntries,
  getCatalogSummary,
  ICatalogEntry,
  ICatalogGroupFilter,
  ICatalogSection,
  toCatalogGroupFilters,
  toCatalogSearchText,
  toCatalogSecondaryText,
  toCatalogSections,
  toRankedSections,
} from "./catalog";

export interface IUseApplicationCatalogOptions {
  applications: ReadonlyArray<IApplicationDescriptor>;
  groups: ReadonlyArray<IApplicationGroup>;
  /** Invoked when a result is accepted with the enter key; opening one is the surface's business. */
  onSelect: (entry: ICatalogEntry) => void;
}

export interface IUseApplicationCatalog {
  /** One chip per group the catalog offers, counted before any narrowing. */
  filters: Array<ICatalogGroupFilter>;
  /** `null` is every group rather than none. */
  selectedGroupId: Nullable<EApplicationGroupId>;
  onSelectGroup: (groupId: Nullable<EApplicationGroupId>) => void;
  /** What the body draws: one section per visible group, or the single ranked one. */
  sections: Array<ICatalogSection>;
  summary: string;
  search: IUseRankedSearch<ICatalogEntry>;
}

/**
 * Narrows the catalog down to what the launcher should be showing.
 *
 * @param options - Catalog inputs and selection behavior.
 * @param options.applications - Every application the catalog offers.
 * @param options.groups - Groups in the order the catalog presents them.
 * @param options.onSelect - Receives the accepted result.
 * @returns The narrowed catalog, its summary, and the controls that narrow it.
 */
export function useApplicationCatalog({
  applications,
  groups,
  onSelect,
}: IUseApplicationCatalogOptions): IUseApplicationCatalog {
  const [selectedGroupId, setSelectedGroupId] = useState<Nullable<EApplicationGroupId>>(null);

  const catalogSections: Array<ICatalogSection> = useMemo(
    () => toCatalogSections(applications, groups),
    [applications, groups]
  );

  const visibleSections: Array<ICatalogSection> = useMemo(
    () =>
      selectedGroupId
        ? catalogSections.filter((section: ICatalogSection) => section.group?.id === selectedGroupId)
        : catalogSections,
    [catalogSections, selectedGroupId]
  );

  const searchable: Array<ICatalogEntry> = useMemo(() => getCatalogEntries(visibleSections), [visibleSections]);

  const search: IUseRankedSearch<ICatalogEntry> = useRankedSearch({
    items: searchable,
    toSearchText: toCatalogSearchText,
    toSecondaryText: toCatalogSecondaryText,
    onSelect,
  });

  const sections: Array<ICatalogSection> = search.isSearching
    ? toRankedSections(search.results.map(({ item }: ISearchResult<ICatalogEntry>) => item))
    : visibleSections;

  const onSelectGroup = useCallback((groupId: Nullable<EApplicationGroupId>) => setSelectedGroupId(groupId), []);

  return {
    filters: toCatalogGroupFilters(catalogSections),
    selectedGroupId,
    onSelectGroup,
    sections,
    summary: getCatalogSummary(visibleSections, selectedGroupId),
    search,
  };
}
