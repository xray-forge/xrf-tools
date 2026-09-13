import { ReactElement } from "react";

import { IArchiveStatisticsView } from "@/applications/archives-explorer/components/editor/statistics/archive-statistics-section";
import { ArchiveBreakdownSection } from "@/applications/archives-explorer/components/editor/statistics/ArchiveBreakdownSection";
import { ArchiveLargestEntry } from "@/core/ipc/types/xrf-archive-stats";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IArchiveLargestSectionProps extends BaseComponentProps {
  largest: Array<ArchiveLargestEntry>;
  view: IArchiveStatisticsView;
}

/**
 * The heaviest entries, and the only section that ends at a file rather than at a figure.
 */
export function ArchiveLargestSection({
  "data-testid": dataTestId = "archive-largest-section",
  id,
  className,
  largest,
  view,
}: IArchiveLargestSectionProps): ReactElement {
  return (
    <ArchiveBreakdownSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Largest files"}
      description={"The heaviest entries of the subject, largest first."}
      fact={`top ${largest.length}`}
      rows={largest.map((entry: ArchiveLargestEntry) => ({
        files: 1,
        id: entry.name,
        label: entry.name,
        sizeReal: entry.sizeReal,
      }))}
      view={view}
      isPreordered
    />
  );
}
