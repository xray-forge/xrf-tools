import {
  EApplicationGroupId,
  EApplicationStatus,
  IApplicationDescriptor,
  IApplicationGroup,
} from "@/core/routing/application";
import { Nullable } from "@/lib/types/general";

/** One application together with the group it was found under, which a ranked result no longer implies. */
export interface ICatalogEntry {
  application: IApplicationDescriptor;
  group: IApplicationGroup;
}

/** Tools the catalog draws together, under the group heading they share. */
export interface ICatalogSection {
  /** `null` for ranked results, where a heading would claim a grouping the order does not have. */
  group: Nullable<IApplicationGroup>;
  entries: Array<ICatalogEntry>;
}

/** One group chip: the group and how much of the catalog choosing it would leave. */
export interface ICatalogGroupFilter {
  group: IApplicationGroup;
  count: number;
}

/**
 * Files the catalog under its groups.
 *
 * @param applications - Every application the catalog offers.
 * @param groups - Groups in the order the catalog presents them.
 * @returns One section per group that has tools, in group order.
 */
export function toCatalogSections(
  applications: ReadonlyArray<IApplicationDescriptor>,
  groups: ReadonlyArray<IApplicationGroup>
): Array<ICatalogSection> {
  return groups
    .map((group: IApplicationGroup): ICatalogSection => ({
      group,
      entries: applications
        .filter((application: IApplicationDescriptor) => application.group === group.id)
        .map((application: IApplicationDescriptor): ICatalogEntry => ({ application, group })),
    }))
    .filter((section: ICatalogSection) => section.entries.length > 0);
}

/**
 * Gathers ranked results into the one section that carries no heading.
 *
 * @param entries - Results in rank order.
 * @returns The single headless section, or no sections at all when nothing matched.
 */
export function toRankedSections(entries: Array<ICatalogEntry>): Array<ICatalogSection> {
  return entries.length ? [{ group: null, entries }] : [];
}

/**
 * @param sections - Sections to offer as chips.
 * @returns One filter per headed section, in section order.
 */
export function toCatalogGroupFilters(sections: ReadonlyArray<ICatalogSection>): Array<ICatalogGroupFilter> {
  return sections.flatMap(({ group, entries }: ICatalogSection) => (group ? [{ group, count: entries.length }] : []));
}

/**
 * @param sections - Sections to read across.
 * @returns Every entry, in the order the sections hold them.
 */
export function getCatalogEntries(sections: ReadonlyArray<ICatalogSection>): Array<ICatalogEntry> {
  return sections.flatMap((section: ICatalogSection) => section.entries);
}

/**
 * Counts the catalog in a line, narrowed the same way the chips narrow it.
 *
 * @param sections - Sections currently on offer.
 * @param selectedGroupId - Group the catalog was narrowed to, or `null` for all of them.
 * @returns The summary line shown beside the page title.
 */
export function getCatalogSummary(
  sections: ReadonlyArray<ICatalogSection>,
  selectedGroupId: Nullable<EApplicationGroupId>
): string {
  const entries: Array<ICatalogEntry> = getCatalogEntries(sections);
  const readyCount: number = entries.filter(
    ({ application }: ICatalogEntry) => application.status === EApplicationStatus.READY
  ).length;

  const parts: Array<string> = [`${entries.length} ${entries.length === 1 ? "tool" : "tools"}`, `${readyCount} ready`];

  // While one group is chosen this could only ever read "1 group", which its own chip already says.
  if (!selectedGroupId) {
    parts.push(`${sections.length} ${sections.length === 1 ? "group" : "groups"}`);
  }

  return parts.join(" · ");
}

/**
 * @param entry - Catalog entry being indexed.
 * @returns The text a search ranks the entry by.
 */
export function toCatalogSearchText(entry: ICatalogEntry): string {
  return entry.application.label;
}

/**
 * The description and the group name match too, so "icons" still finds the six sprite tools whose
 * labels only say "sprite".
 *
 * @param entry - Catalog entry being indexed.
 * @returns Text that matches without outranking a label match.
 */
export function toCatalogSecondaryText(entry: ICatalogEntry): string {
  return `${entry.application.description} ${entry.group.label}`;
}
