import { Nullable } from "@xrf/types";

import { ArchiveStatistics } from "@/core/ipc/types/xrf-archive-stats";

import {
  defineArchiveStatisticsSection,
  EArchiveStatisticsSection,
  IArchiveStatisticsSection,
} from "./archive-statistics-section";
import { ArchiveCompressionSection } from "./sections/ArchiveCompressionSection";
import { ArchiveExtensionsSection } from "./sections/ArchiveExtensionsSection";
import { ArchiveFoldersSection } from "./sections/ArchiveFoldersSection";
import { ArchiveLargestSection } from "./sections/ArchiveLargestSection";
import { ArchiveOriginsSection } from "./sections/ArchiveOriginsSection";
import { ArchiveOverviewSection } from "./sections/ArchiveOverviewSection";
import { ArchiveSizesSection } from "./sections/ArchiveSizesSection";
import { ArchiveVolumesSection } from "./sections/ArchiveVolumesSection";

/**
 * Every section the dialog can show, in the order the rail lists them.
 */
export const ARCHIVE_STATISTICS_SECTIONS: ReadonlyArray<IArchiveStatisticsSection> = [
  defineArchiveStatisticsSection({
    id: EArchiveStatisticsSection.OVERVIEW,
    label: "Overview",
    select: (statistics: ArchiveStatistics) => statistics.overview,
    render: (overview) => <ArchiveOverviewSection overview={overview} />,
  }),
  defineArchiveStatisticsSection({
    id: EArchiveStatisticsSection.EXTENSIONS,
    label: "Extensions",
    select: (statistics: ArchiveStatistics) => statistics.extensions,
    render: (extensions, view) => <ArchiveExtensionsSection extensions={extensions} view={view} />,
  }),
  defineArchiveStatisticsSection({
    id: EArchiveStatisticsSection.FOLDERS,
    label: "Folders",
    select: (statistics: ArchiveStatistics) => statistics.folders,
    render: (folders, view) => <ArchiveFoldersSection folders={folders} view={view} />,
  }),
  defineArchiveStatisticsSection({
    id: EArchiveStatisticsSection.SIZES,
    label: "Sizes",
    select: (statistics: ArchiveStatistics) => statistics.sizes,
    render: (sizes, view) => <ArchiveSizesSection sizes={sizes} view={view} />,
  }),
  defineArchiveStatisticsSection({
    id: EArchiveStatisticsSection.LARGEST,
    label: "Largest files",
    select: (statistics: ArchiveStatistics) => statistics.largest,
    render: (largest, view) => <ArchiveLargestSection largest={largest} view={view} />,
  }),
  defineArchiveStatisticsSection({
    id: EArchiveStatisticsSection.COMPRESSION,
    label: "Compression",
    // A volume set only: a loose file has no stored size, so a world leaves this absent rather than empty.
    select: (statistics: ArchiveStatistics) =>
      statistics.compression === null
        ? null
        : { compression: statistics.compression, extensions: statistics.extensions },
    render: (selected) => (
      <ArchiveCompressionSection compression={selected.compression} extensions={selected.extensions} />
    ),
  }),
  defineArchiveStatisticsSection({
    id: EArchiveStatisticsSection.VOLUMES,
    label: "Volumes",
    select: (statistics: ArchiveStatistics) => statistics.volumes,
    render: (volumes, view) => <ArchiveVolumesSection volumes={volumes} view={view} />,
  }),
  defineArchiveStatisticsSection({
    id: EArchiveStatisticsSection.ORIGINS,
    label: "Origins",
    // A world only: a merged name table cannot say what it folded away.
    select: (statistics: ArchiveStatistics) => statistics.origins,
    render: (origins, view) => <ArchiveOriginsSection origins={origins} view={view} />,
  }),
];

/**
 * The sections the open report can answer, in rail order.
 *
 * @param statistics - The report, or null before it arrives.
 * @returns What the rail should list.
 */
export function listArchiveStatisticsSections(
  statistics: Nullable<ArchiveStatistics>
): ReadonlyArray<IArchiveStatisticsSection> {
  return statistics === null ? [] : ARCHIVE_STATISTICS_SECTIONS.filter((it) => it.isAnswerable(statistics));
}
