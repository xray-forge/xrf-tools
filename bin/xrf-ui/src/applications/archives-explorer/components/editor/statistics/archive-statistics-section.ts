import { ReactElement } from "react";

import { ArchiveStatistics } from "@/core/ipc/types/xrf-archive-stats";
import { EStatMeasure } from "@/core/ui/stats/stat-measure";
import { Nullable } from "@/lib/types/general";

/** Stable identity of a section, so a rail selection survives a re-render and a test can name one. */
export enum EArchiveStatisticsSection {
  OVERVIEW = "overview",
  EXTENSIONS = "extensions",
  FOLDERS = "folders",
  SIZES = "sizes",
  LARGEST = "largest",
  COMPRESSION = "compression",
  VOLUMES = "volumes",
  ORIGINS = "origins",
}

/**
 * What every section reads besides the report: the view state the dialog owns.
 */
export interface IArchiveStatisticsView {
  /** Which measurement orders the rows and draws the bars. */
  measure: EStatMeasure;
  onMeasureChange: (measure: EStatMeasure) => void;
}

/**
 * One section of the statistics dialog, as the rail and the content area both read it.
 */
export interface IArchiveStatisticsSection {
  id: EArchiveStatisticsSection;
  label: string;
  /** Whether the open report can answer this section at all. */
  isAnswerable: (statistics: ArchiveStatistics) => boolean;
  render: (statistics: ArchiveStatistics, view: IArchiveStatisticsView) => Nullable<ReactElement>;
}

/**
 * Declares a section in terms of the one part of the report it needs.
 *
 * @param section - The declaration.
 * @param section.id - Stable identity, which the rail selection is kept as.
 * @param section.label - What the rail calls it.
 * @param section.select - The part of the report this section needs, or null when the subject cannot answer it.
 * @param section.render - How that part is shown.
 * @returns The section as the dialog consumes it, with its selection erased.
 */
export function defineArchiveStatisticsSection<T>(section: {
  id: EArchiveStatisticsSection;
  label: string;
  select: (statistics: ArchiveStatistics) => Nullable<T>;
  render: (selected: T, view: IArchiveStatisticsView) => ReactElement;
}): IArchiveStatisticsSection {
  return {
    id: section.id,
    label: section.label,
    isAnswerable: (statistics: ArchiveStatistics) => section.select(statistics) !== null,
    render: (statistics: ArchiveStatistics, view: IArchiveStatisticsView) => {
      const selected: Nullable<T> = section.select(statistics);

      return selected === null ? null : section.render(selected, view);
    },
  };
}
